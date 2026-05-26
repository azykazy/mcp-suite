#!/usr/bin/env python3
import json
import os
import re
import subprocess
import sys


HOME = os.path.expanduser("~")
CODEX_HOME = os.path.join(HOME, ".codex")


def deny(reason: str) -> None:
    print(
        json.dumps(
            {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason,
                }
            },
            ensure_ascii=False,
        )
    )
    sys.exit(0)


def git_root(cwd: str) -> str:
    try:
        out = subprocess.check_output(
            ["git", "-C", cwd, "rev-parse", "--show-toplevel"],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        return out or cwd
    except Exception:
        return cwd


def current_branch(cwd: str) -> str:
    try:
        return subprocess.check_output(
            ["git", "-C", cwd, "branch", "--show-current"],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
    except Exception:
        return ""


def normalize_path(token: str) -> str:
    token = token.strip("\"'")
    token = token.rstrip("),;]")
    return os.path.abspath(os.path.expanduser(token))


def is_allowed_home_path(path: str, project_root: str) -> bool:
    path = os.path.abspath(path)
    project_root = os.path.abspath(project_root)
    return (
        path == project_root
        or path.startswith(project_root + os.sep)
        or path == CODEX_HOME
        or path.startswith(CODEX_HOME + os.sep)
    )


def check_bash(command: str, cwd: str) -> None:
    project_root = git_root(cwd)

    for match in re.finditer(r"(?:~|%s)/\S+" % re.escape(HOME), command):
        path = normalize_path(match.group(0))
        if path.startswith(HOME + os.sep) and not is_allowed_home_path(path, project_root):
            deny(
                "[アクセス制限] プロジェクト外へのアクセスは禁止です: "
                f"{path} (許可範囲: {project_root}, {CODEX_HOME})"
            )

    if re.search(r"\bgit\s+(commit|push)\b", command):
        branch = current_branch(cwd)
        if branch in {"main", "master"}:
            deny(
                "[ブランチ保護] main/master への直接 commit/push は禁止です。"
                "feat/fix/hotfix/chore/refactor プレフィックスのブランチを作成してから作業してください。"
            )

    tool_match = re.search(r"(?:^|[|;&\s])(grep|find|tree)(?=\s)", command)
    if tool_match:
        tool = tool_match.group(1)
        deny(
            f"[MCP優先ルール] Bash の '{tool}' は禁止です。"
            "mcp-tools の対応ツール、または rg/rg --files を使用してください。"
        )


def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return

    if payload.get("tool_name") != "Bash":
        return

    tool_input = payload.get("tool_input") or {}
    command = tool_input.get("command") or ""
    cwd = payload.get("cwd") or os.getcwd()

    if command:
        check_bash(command, cwd)


if __name__ == "__main__":
    main()
