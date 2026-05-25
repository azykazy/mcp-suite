#!/usr/bin/env python3
"""
web_fetch MCP ツールのベンチマーク

測定項目:
  - 実行時間 (ms): avg, min, p95, stdev
  - 出力バイト数とトークン概算 (bytes / 4)
  - トークン削減率: curl 生 HTML vs web_fetch 抽出テキスト

テストシナリオ:
  W-1: 小規模 HTML (example.com)
  W-2: 中規模ドキュメントページ (docs.rs/anyhow)
  W-3: 大規模ページ (doc.rust-lang.org/std/string)
  W-4: セレクター指定での部分抽出 (selector="main")
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

SCENARIOS = [
    {
        "id": "W1",
        "label": "小規模 HTML (example.com)",
        "url": "https://example.com",
        "selector": None,
    },
    {
        "id": "W2",
        "label": "中規模ドキュメント (docs.rs/anyhow)",
        "url": "https://docs.rs/anyhow/latest/anyhow/",
        "selector": None,
    },
    {
        "id": "W3",
        "label": "大規模ページ (doc.rust-lang.org/String)",
        "url": "https://doc.rust-lang.org/std/string/struct.String.html",
        "selector": None,
    },
    {
        "id": "W4",
        "label": "セレクター指定 main (docs.rs/anyhow)",
        "url": "https://docs.rs/anyhow/latest/anyhow/",
        "selector": "main",
    },
]


def _percentile(sorted_data: list, pct: float) -> float:
    if not sorted_data:
        return 0.0
    k = (len(sorted_data) - 1) * pct / 100
    lo, hi = int(k), min(int(k) + 1, len(sorted_data) - 1)
    return sorted_data[lo] + (sorted_data[hi] - sorted_data[lo]) * (k - lo)


def call_mcp_web_fetch(url: str, selector: str | None, max_chars: int, runs: int) -> dict:
    """MCP web_fetch ツールを呼び出して計測"""
    init_msg = json.dumps({
        "jsonrpc": "2.0", "id": 0, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                   "clientInfo": {"name": "bench", "version": "0"}},
    })
    args: dict = {"url": url}
    if max_chars is not None:
        args["max_chars"] = max_chars
    if selector:
        args["selector"] = selector
    call_msg = json.dumps({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": "web_fetch", "arguments": args},
    })
    stdin_data = (init_msg + "\n" + call_msg + "\n").encode()

    elapsed_list = []
    output_text = ""
    is_error = False

    for _ in range(runs):
        t0 = time.perf_counter()
        proc = subprocess.run([MCP_BINARY], input=stdin_data, capture_output=True)
        t1 = time.perf_counter()
        elapsed_list.append((t1 - t0) * 1000)

        for line in reversed(proc.stdout.decode(errors="replace").strip().split("\n")):
            try:
                resp = json.loads(line)
                content = resp.get("result", {}).get("content", [{}])
                output_text = content[0].get("text", "") if content else ""
                is_error = resp.get("result", {}).get("isError", False)
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
        "output_preview": output_text[:300].replace("\n", "\\n"),
        "is_error": is_error,
    }


def call_curl(url: str, runs: int) -> dict:
    """curl -sL で生 HTML を取得して計測"""
    elapsed_list = []
    output_text = ""

    for _ in range(runs):
        t0 = time.perf_counter()
        proc = subprocess.run(
            ["curl", "-sL", "--max-time", "15", url],
            capture_output=True,
        )
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
    }


def ratio(a, b) -> str:
    if a == 0 or b == 0:
        return "N/A"
    return f"{b / a:.2f}x"


def print_result(scenario: dict, mcp: dict, curl: dict) -> None:
    sel = f" [selector={scenario['selector']}]" if scenario["selector"] else ""
    print(f"\n#### {scenario['id']}: {scenario['label']}{sel}")
    if mcp["is_error"]:
        print(f"  [!] MCP エラー: {mcp['output_preview']}")
        return

    print(f"{'指標':<26} {'web_fetch':>12} {'curl (生HTML)':>14} {'削減率':>10}")
    print("-" * 64)
    print(f"{'平均実行時間(ms)':<26} {mcp['avg_ms']:>12.1f} {curl['avg_ms']:>14.1f} {'':>10}")
    print(f"{'最小実行時間(ms)':<26} {mcp['min_ms']:>12.1f} {curl['min_ms']:>14.1f} {'':>10}")
    print(f"{'p95実行時間(ms)':<26} {mcp['p95_ms']:>12.1f} {curl['p95_ms']:>14.1f} {'':>10}")
    print(f"{'標準偏差(ms)':<26} {mcp['stdev_ms']:>12.2f} {curl['stdev_ms']:>14.2f} {'':>10}")
    print(f"{'出力バイト数':<26} {mcp['output_bytes']:>12,} {curl['output_bytes']:>14,} {ratio(mcp['output_bytes'], curl['output_bytes']):>10}")
    print(f"{'概算トークン数':<26} {mcp['approx_tokens']:>12,} {curl['approx_tokens']:>14,} {ratio(mcp['approx_tokens'], curl['approx_tokens']):>10}")
    print(f"  出力プレビュー: {mcp['output_preview'][:120]}...")


def main() -> None:
    parser = argparse.ArgumentParser(description="web_fetch MCP ツール ベンチマーク")
    parser.add_argument("--runs", type=int, default=3, metavar="N",
                        help="各ケースの計測回数 (デフォルト: 3)")
    parser.add_argument("--max-chars", type=int, default=None, metavar="N",
                        help="web_fetch の max_chars (省略時は全文返却)")
    parser.add_argument("--output", default=None, metavar="FILE",
                        help="結果 JSON の出力先 (デフォルト: scripts/benchmark_web_fetch_results.json)")
    cli = parser.parse_args()
    runs = cli.runs
    max_chars = cli.max_chars

    print("=" * 70)
    print(f"web_fetch ベンチマーク  (各 {runs} 回計測)")
    print(f"比較: MCP web_fetch (テキスト抽出) vs curl (生 HTML)")
    print(f"max_chars={'制限なし' if max_chars is None else max_chars}")
    print("=" * 70)

    results: dict = {}

    for scenario in SCENARIOS:
        sid = scenario["id"]
        print(f"\n[{sid}] {scenario['label']}")
        print(f"  URL: {scenario['url']}")

        mcp = call_mcp_web_fetch(
            scenario["url"], scenario["selector"], max_chars, runs
        )
        # curl は selector なしで常に全体を取得（比較対象）
        curl = call_curl(scenario["url"], runs)

        print_result(scenario, mcp, curl)
        results[sid] = {
            "scenario": scenario,
            "mcp": mcp,
            "curl": curl,
        }

    # ── 最終サマリー ──────────────────────────────────────────────────
    print("\n\n" + "=" * 70)
    print("最終サマリー: トークン削減効果")
    print("=" * 70)
    print(f"{'ケース':<36} {'web_fetchトークン':>18} {'curlトークン':>14} {'削減率':>10}")
    print("-" * 80)

    for scenario in SCENARIOS:
        sid = scenario["id"]
        r = results[sid]
        if r["mcp"]["is_error"]:
            continue
        mcp_tok = r["mcp"]["approx_tokens"]
        curl_tok = r["curl"]["approx_tokens"]
        red = ratio(mcp_tok, curl_tok)
        label = f"{sid}: {scenario['label']}"
        if scenario["selector"]:
            label += f" [{scenario['selector']}]"
        print(f"  {label:<34} {mcp_tok:>18,} {curl_tok:>14,} {red:>10}")

    out_dir = os.path.dirname(os.path.abspath(__file__))
    out_path = cli.output or os.path.join(out_dir, "benchmark_web_fetch_results.json")
    results["_meta"] = {
        "date": datetime.date.today().isoformat(),
        "runs": runs,
        "max_chars": max_chars,
    }
    with open(out_path, "w") as f:
        json.dump(results, f, ensure_ascii=False, indent=2, default=str)
    print(f"\n生データ保存先: {out_path}")


if __name__ == "__main__":
    main()
