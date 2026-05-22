# mcp-suite

自作 & OSS MCP をまとめた個人用MCPセットアップリポジトリ。

## クイックスタート

```bash
git clone https://github.com/azykazy/mcp-suite.git
cd mcp-suite
cp .env.example .env
vi .env          # GITHUB_PERSONAL_ACCESS_TOKEN など必要な値を設定
bash setup.sh
```

Claude Code を再起動するとMCPが有効になります。

## 構成

```
mcp-suite/
├── setup.sh              # セットアップスクリプト（1本で完結）
├── config/
│   └── mcp_settings.json # MCPサーバー設定テンプレート
├── custom/               # 自作MCPサーバー（ソースコード管理）
│   └── <mcp-name>/
├── oss/                  # OSSのMCP情報・参照
│   ├── context7/         # ドキュメント取得MCP
│   ├── github/           # GitHub操作MCP
│   └── playwright/       # ブラウザ操作MCP
└── .env.example          # 環境変数テンプレート
```

## 含まれるMCP

### OSS

| MCP | 用途 | パッケージ |
|-----|------|-----------|
| context7 | ライブラリドキュメント取得 | `@upstash/context7-mcp` |
| GitHub | GitHub API操作 | `@modelcontextprotocol/server-github` |
| Playwright | ブラウザ操作 | `@playwright/mcp` |

### 自作

| MCP | 用途 | 実装 |
|-----|------|------|
| [mcp-tools](custom/mcp-tools/) | grep / diff / find（低トークン・高速） | Rust |

詳細は各ディレクトリの README を参照。

## 自作MCPの追加

1. `custom/<mcp-name>/` にソースを配置
2. `config/mcp_settings.json` に `mcpServers` エントリを追加
3. `bash setup.sh` を再実行

## 必要な環境変数

| 変数 | 用途 | 必須 |
|------|------|------|
| `GITHUB_PERSONAL_ACCESS_TOKEN` | GitHub MCP認証 | GitHub MCP使用時 |

## 前提条件

- Node.js 18+
- jq（`sudo apt install jq` または `brew install jq`）
- Rust / cargo（mcp-tools ビルド用、`curl https://sh.rustup.rs -sSf | sh` または `mise use rust`）
