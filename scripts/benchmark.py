#!/usr/bin/env python3
"""
自作MCPツール vs 標準Bashコマンドのベンチマーク

測定項目:
  - 実行時間 (ms): avg, min, p95, stdev
  - 出力バイト数（トークン数の近似: bytes / 4）

テストシナリオ:
  A. 小規模: mcp-tools/src（5ファイル）
  B. 大規模: .cargo/registry（600+ファイル）
  C. ウォームサーバー: MCP永続プロセスvs都度Bashプロセス起動
"""

import argparse
import datetime
import json
import subprocess
import time
import os
import statistics

MCP_BINARY = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "custom/mcp-tools/target/release/mcp-tools",
)
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(REPO, "custom/mcp-tools/src")
LARGE_DIR = os.path.expanduser("~/.cargo/registry/src")


def _percentile(sorted_data: list, pct: float) -> float:
    if not sorted_data:
        return 0.0
    k = (len(sorted_data) - 1) * pct / 100
    lo, hi = int(k), min(int(k) + 1, len(sorted_data) - 1)
    return sorted_data[lo] + (sorted_data[hi] - sorted_data[lo]) * (k - lo)


def call_mcp(tool: str, arguments: dict, runs: int = 5) -> dict:
    """MCPバイナリをコールドスタートして計測（各呼び出しで新しいプロセスを起動）"""
    init_msg = json.dumps({
        "jsonrpc": "2.0", "id": 0, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                   "clientInfo": {"name": "bench", "version": "0"}}
    })
    call_msg = json.dumps({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": tool, "arguments": arguments}
    })
    stdin_data = (init_msg + "\n" + call_msg + "\n").encode()

    elapsed_list = []
    output_text = ""

    for _ in range(runs):
        t0 = time.perf_counter()
        proc = subprocess.run([MCP_BINARY], input=stdin_data, capture_output=True)
        t1 = time.perf_counter()
        elapsed_list.append((t1 - t0) * 1000)

        lines = proc.stdout.decode(errors="replace").strip().split("\n")
        for line in reversed(lines):
            try:
                resp = json.loads(line)
                output_text = (
                    resp.get("result", {}).get("content", [{}])[0].get("text", "")
                )
                break
            except json.JSONDecodeError:
                continue

    sorted_elapsed = sorted(elapsed_list)
    output_bytes = len(output_text.encode("utf-8"))

    return {
        "avg_ms": sum(elapsed_list) / len(elapsed_list),
        "min_ms": min(elapsed_list),
        "p95_ms": _percentile(sorted_elapsed, 95),
        "stdev_ms": statistics.stdev(elapsed_list) if len(elapsed_list) > 1 else 0.0,
        "output_bytes": output_bytes,
        "approx_tokens": max(1, output_bytes // 4),
        "output_preview": output_text[:200].replace("\n", "\\n"),
    }


def call_mcp_warm(tool: str, arguments_list: list, repeat: int = 5) -> dict:
    """MCPバイナリを1プロセス起動して複数リクエストを連続送信（ウォームサーバー）"""
    init_msg = json.dumps({
        "jsonrpc": "2.0", "id": 0, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                   "clientInfo": {"name": "bench", "version": "0"}}
    })

    call_msgs = []
    for i, arguments in enumerate(arguments_list * repeat):
        call_msgs.append(json.dumps({
            "jsonrpc": "2.0", "id": i + 1, "method": "tools/call",
            "params": {"name": tool, "arguments": arguments}
        }))

    stdin_data = (init_msg + "\n" + "\n".join(call_msgs) + "\n").encode()

    t0 = time.perf_counter()
    proc = subprocess.run([MCP_BINARY], input=stdin_data, capture_output=True)
    t1 = time.perf_counter()

    total_ms = (t1 - t0) * 1000
    n = len(arguments_list) * repeat
    avg_per_request_ms = total_ms / n

    lines = [l for l in proc.stdout.decode(errors="replace").strip().split("\n") if l.strip()]
    output_bytes = 0
    for line in lines[1:]:  # skip initialize response
        try:
            resp = json.loads(line)
            text = resp.get("result", {}).get("content", [{}])[0].get("text", "")
            output_bytes += len(text.encode("utf-8"))
        except json.JSONDecodeError:
            continue

    avg_output_bytes = output_bytes // n if n > 0 else 0

    return {
        "total_ms": total_ms,
        "avg_ms": avg_per_request_ms,
        "n_requests": n,
        "avg_output_bytes": avg_output_bytes,
        "approx_tokens": max(1, avg_output_bytes // 4),
    }


def call_bash(cmd: list, runs: int = 5) -> dict:
    """Bashコマンドを都度起動して計測"""
    elapsed_list = []
    output_text = ""

    for _ in range(runs):
        t0 = time.perf_counter()
        proc = subprocess.run(cmd, capture_output=True, cwd=REPO)
        t1 = time.perf_counter()
        elapsed_list.append((t1 - t0) * 1000)
        output_text = proc.stdout.decode(errors="replace")

    sorted_elapsed = sorted(elapsed_list)
    output_bytes = len(output_text.encode("utf-8"))

    return {
        "avg_ms": sum(elapsed_list) / len(elapsed_list),
        "min_ms": min(elapsed_list),
        "p95_ms": _percentile(sorted_elapsed, 95),
        "stdev_ms": statistics.stdev(elapsed_list) if len(elapsed_list) > 1 else 0.0,
        "output_bytes": output_bytes,
        "approx_tokens": max(1, output_bytes // 4),
        "output_preview": output_text[:200].replace("\n", "\\n"),
    }


def ratio(mcp_val, bash_val):
    if bash_val == 0 or mcp_val == 0:
        return "N/A"
    r = bash_val / mcp_val
    return f"{r:.2f}x"


def print_result(label, mcp, bash):
    print(f"\n#### {label}")
    print(f"{'指標':<22} {'MCP':>10} {'Bash':>10} {'Bash/MCP':>10}")
    print("-" * 56)
    print(f"{'平均実行時間(ms)':<22} {mcp['avg_ms']:>10.1f} {bash['avg_ms']:>10.1f} {ratio(mcp['avg_ms'], bash['avg_ms']):>10}")
    print(f"{'最小実行時間(ms)':<22} {mcp['min_ms']:>10.1f} {bash['min_ms']:>10.1f} {ratio(mcp['min_ms'], bash['min_ms']):>10}")
    print(f"{'p95実行時間(ms)':<22} {mcp['p95_ms']:>10.1f} {bash['p95_ms']:>10.1f} {ratio(mcp['p95_ms'], bash['p95_ms']):>10}")
    print(f"{'標準偏差(ms)':<22} {mcp['stdev_ms']:>10.2f} {bash['stdev_ms']:>10.2f} {'':>10}")
    print(f"{'出力バイト数':<22} {mcp['output_bytes']:>10,} {bash['output_bytes']:>10,} {ratio(mcp['output_bytes'], bash['output_bytes']):>10}")
    print(f"{'概算トークン数':<22} {mcp['approx_tokens']:>10,} {bash['approx_tokens']:>10,} {ratio(mcp['approx_tokens'], bash['approx_tokens']):>10}")


def main():
    parser = argparse.ArgumentParser(description="MCP Tools vs Bash ベンチマーク")
    parser.add_argument("--runs", type=int, default=5, metavar="N",
                        help="各ケースの計測回数 (デフォルト: 5)")
    parser.add_argument("--output", default=None, metavar="FILE",
                        help="結果JSONの出力先 (デフォルト: scripts/benchmark_results.json)")
    cli = parser.parse_args()
    runs = cli.runs

    print("=" * 70)
    print(f"MCP Tools ベンチマーク (各{runs}回計測の平均)")
    print(f"対象リポジトリ: {REPO}")
    print("=" * 70)

    results = {}

    # ── シナリオA: 小規模（src/）──────────────────────────────────────
    print("\n\n## シナリオA: 小規模（mcp-tools/src/ - 5ファイル）")

    print("\n[A-1] grep: 'fn ' を src/ で検索")
    a1_mcp = call_mcp("grep", {"pattern": "fn ", "paths": [SRC]}, runs)
    a1_bash = call_bash(["grep", "-rn", "fn ", SRC], runs)
    print_result("grep (小規模)", a1_mcp, a1_bash)
    results["A1_grep_small"] = {"mcp": a1_mcp, "bash": a1_bash}

    print("\n[A-2] find: src/ 以下のファイルを列挙")
    a2_mcp = call_mcp("find", {"path": SRC, "type": "f"}, runs)
    a2_bash = call_bash(["find", SRC, "-type", "f"], runs)
    print_result("find (小規模)", a2_mcp, a2_bash)
    results["A2_find_small"] = {"mcp": a2_mcp, "bash": a2_bash}

    print("\n[A-3] diff: main.rs vs grep.rs")
    file_a = os.path.join(SRC, "main.rs")
    file_b = os.path.join(SRC, "grep.rs")
    a3_mcp = call_mcp("diff", {"a": file_a, "b": file_b}, runs)
    a3_bash = call_bash(["diff", "-u", file_a, file_b], runs)
    print_result("diff (main.rs vs grep.rs)", a3_mcp, a3_bash)
    results["A3_diff"] = {"mcp": a3_mcp, "bash": a3_bash}

    print("\n[A-4] git_diff: HEAD~1..HEAD")
    a4_mcp = call_mcp("git_diff", {"repo": REPO, "from": "HEAD~1", "to": "HEAD"}, runs)
    a4_bash = call_bash(["git", "diff", "HEAD~1", "HEAD"], runs)
    print_result("git_diff (HEAD~1..HEAD)", a4_mcp, a4_bash)
    results["A4_git_diff"] = {"mcp": a4_mcp, "bash": a4_bash}

    print("\n[A-5] file_outline: main.rs のシンボル一覧")
    a5_mcp = call_mcp("file_outline", {"path": file_a}, runs)
    # Bash相当: Rustの関数・構造体・列挙型・トレイト・impl をgrepで抽出
    a5_bash = call_bash([
        "grep", "-En",
        r"^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]|"
        r"^[[:space:]]*(pub[[:space:]]+)?struct[[:space:]]|"
        r"^[[:space:]]*(pub[[:space:]]+)?enum[[:space:]]|"
        r"^[[:space:]]*(pub[[:space:]]+)?trait[[:space:]]|"
        r"^[[:space:]]*impl[[:space:]]",
        file_a,
    ], runs)
    print_result("file_outline (main.rs)", a5_mcp, a5_bash)
    results["A5_file_outline"] = {"mcp": a5_mcp, "bash": a5_bash}

    print("\n[A-6] git_log: 直近20コミット")
    a6_mcp = call_mcp("git_log", {"repo": REPO, "limit": 20}, runs)
    a6_bash = call_bash([
        "git", "-C", REPO, "log", "-20",
        "--pretty=format:%h  %ad  %an  %s", "--date=short",
    ], runs)
    print_result("git_log (直近20件)", a6_mcp, a6_bash)
    results["A6_git_log"] = {"mcp": a6_mcp, "bash": a6_bash}

    # ── シナリオB: 大規模（cargo registry）──────────────────────────
    print("\n\n## シナリオB: 大規模（.cargo/registry - 600+ファイル）")
    print("※ MCP grep はデフォルトで max_matches=100 で打ち切り")

    print("\n[B-1] grep: 'fn ' を .cargo/registry で検索")
    b1_mcp = call_mcp("grep", {"pattern": "fn ", "paths": [LARGE_DIR], "max_matches": 100}, runs)
    b1_bash = call_bash(["grep", "-rn", "--include=*.rs", "fn ", LARGE_DIR], runs)
    print_result("grep (大規模 / 100件上限 vs 無制限)", b1_mcp, b1_bash)
    results["B1_grep_large"] = {"mcp": b1_mcp, "bash": b1_bash}

    print("\n[B-2] find: .cargo/registry 以下の .rs ファイルを列挙")
    b2_mcp = call_mcp("find", {"path": LARGE_DIR, "type": "f", "pattern": "*.rs"}, runs)
    b2_bash = call_bash(["find", LARGE_DIR, "-type", "f", "-name", "*.rs"], runs)
    print_result("find (大規模 / 200件上限 vs 無制限)", b2_mcp, b2_bash)
    results["B2_find_large"] = {"mcp": b2_mcp, "bash": b2_bash}

    print("\n[B-3] file_outline: 大規模Rustファイル（syn/expr.rs 4179行）")
    large_rs = os.path.expanduser(
        "~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f"
        "/syn-2.0.117/src/expr.rs"
    )
    b3_mcp = call_mcp("file_outline", {"path": large_rs}, runs)
    b3_bash = call_bash([
        "grep", "-En",
        r"^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]|"
        r"^[[:space:]]*(pub[[:space:]]+)?struct[[:space:]]|"
        r"^[[:space:]]*(pub[[:space:]]+)?enum[[:space:]]|"
        r"^[[:space:]]*(pub[[:space:]]+)?trait[[:space:]]|"
        r"^[[:space:]]*impl[[:space:]]",
        large_rs,
    ], runs)
    print_result("file_outline (syn/expr.rs 4179行)", b3_mcp, b3_bash)
    results["B3_file_outline_large"] = {"mcp": b3_mcp, "bash": b3_bash}

    print("\n[B-4] git_log: 107コミットのリポジトリ（limit=20 vs 無制限）")
    seat_manager = os.path.expanduser("~/seat_manager")
    b4_mcp = call_mcp("git_log", {"repo": seat_manager, "limit": 20}, runs)
    b4_bash = call_bash([
        "git", "-C", seat_manager, "log",
        "--pretty=format:%h  %ad  %an  %s", "--date=short",
    ], runs)
    print_result("git_log (107コミット / limit=20 vs 全件)", b4_mcp, b4_bash)
    results["B4_git_log_large"] = {"mcp": b4_mcp, "bash": b4_bash}

    # ── シナリオC: ウォームサーバー（MCP永続プロセス）──────────────
    n = 10
    print(f"\n\n## シナリオC: ウォームサーバー比較")
    print(f"（MCP: 1プロセス起動 → {n}リクエスト連続 vs Bash: {n}回別プロセス起動）")

    warm = call_mcp_warm("grep", [{"pattern": "fn ", "paths": [SRC]}], repeat=n)
    cold_bash_times = []
    for _ in range(n):
        t0 = time.perf_counter()
        subprocess.run(["grep", "-rn", "fn ", SRC], capture_output=True)
        t1 = time.perf_counter()
        cold_bash_times.append((t1 - t0) * 1000)
    bash_avg = sum(cold_bash_times) / len(cold_bash_times)

    print(f"\n#### grep × {n}回連続実行")
    print(f"{'指標':<30} {'MCP(ウォーム)':>16} {'Bash(都度起動)':>16}")
    print("-" * 66)
    print(f"{'1リクエストあたり平均(ms)':<30} {warm['avg_ms']:>16.2f} {bash_avg:>16.2f}")
    print(f"{'合計時間(ms)':<30} {warm['total_ms']:>16.1f} {sum(cold_bash_times):>16.1f}")
    print(f"{'速度改善(Bash/MCP)':<30} {bash_avg / warm['avg_ms']:>16.2f}x {'':>16}")

    results["C_warm_server"] = {
        "mcp": {"avg_ms": warm["avg_ms"], "total_ms": warm["total_ms"]},
        "bash": {"avg_ms": bash_avg, "total_ms": sum(cold_bash_times)},
    }

    # ── 最終サマリー ─────────────────────────────────────────────────
    print("\n\n" + "=" * 70)
    print("最終サマリー")
    print("=" * 70)
    summary_rows = [
        ("A1: grep (小規模)",       "A1_grep_small"),
        ("A2: find (小規模)",       "A2_find_small"),
        ("A3: diff",                "A3_diff"),
        ("A4: git_diff",            "A4_git_diff"),
        ("A5: file_outline",        "A5_file_outline"),
        ("A6: git_log",             "A6_git_log"),
        ("B1: grep (大規模)",           "B1_grep_large"),
        ("B2: find (大規模)",           "B2_find_large"),
        ("B3: file_outline (大規模)",   "B3_file_outline_large"),
        ("B4: git_log (大規模)",        "B4_git_log_large"),
    ]
    print(f"{'ケース':<28} {'速度(Bash/MCP)':>16} {'トークン削減(Bash/MCP)':>22}")
    print("-" * 68)
    for label, key in summary_rows:
        r = results[key]
        speed  = r["bash"]["avg_ms"]        / r["mcp"]["avg_ms"]        if r["mcp"]["avg_ms"]        > 0 else 0
        tokens = r["bash"]["approx_tokens"]  / r["mcp"]["approx_tokens"] if r["mcp"]["approx_tokens"] > 0 else 0
        print(f"  {label:<26} {speed:>14.2f}x {tokens:>20.2f}x")

    warm_speed = results["C_warm_server"]["bash"]["avg_ms"] / results["C_warm_server"]["mcp"]["avg_ms"]
    print(f"  {'C: ウォームサーバー(×10)':<26} {warm_speed:>14.2f}x {'(計測外)':>22}")

    out_path = cli.output or os.path.join(REPO, "scripts/benchmark_results.json")
    results["_meta"] = {
        "date": datetime.date.today().isoformat(),
        "runs": runs,
        "repo": REPO,
    }
    with open(out_path, "w") as f:
        json.dump(results, f, ensure_ascii=False, indent=2)
    print(f"\n生データ保存先: {out_path}")

    return results


if __name__ == "__main__":
    main()
