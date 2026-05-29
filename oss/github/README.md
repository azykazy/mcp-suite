---
title: GitHub MCP
description: GitHub API を操作する OSS MCP サーバー
type: readme
tags: [mcp, github, oss]
path: oss/github/README.md
---

# GitHub MCP

GitHub APIを操作するMCPサーバー。

- パッケージ: `@modelcontextprotocol/server-github`
- 実行方式: `npx -y @modelcontextprotocol/server-github`
- 環境変数: `GITHUB_PERSONAL_ACCESS_TOKEN`（必須）

## トークンの取得

GitHub Settings → Developer settings → Personal access tokens で生成。  
必要スコープ: `repo`, `read:org`

## 参考

- https://github.com/modelcontextprotocol/servers/tree/main/src/github
