# mcp-suite

自作 & OSS MCP と Claude Code サブエージェントをまとめた個人用セットアップリポジトリ。

## クイックスタート

```bash
git clone https://github.com/azykazy/mcp-suite.git
cd mcp-suite
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
│   └── mcp-tools/        # grep / diff / find（Rust実装）
├── oss/                  # OSSのMCP情報・参照
│   ├── context7/         # ドキュメント取得MCP
│   └── playwright/       # ブラウザ操作MCP
└── agents/               # Claude Code サブエージェント定義
    ├── code-reviewer-ja.md  # 日本語コードレビュー専門エージェント
    └── git-workflow-ja.md   # Gitワークフロー専門エージェント
```

## 含まれるMCP

### OSS

| MCP | 用途 | パッケージ |
|-----|------|-----------|
| context7 | ライブラリドキュメント取得 | `@upstash/context7-mcp` |
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

## サブエージェントの追加

1. `agents/<agent-name>.md` をエージェント定義フォーマットで作成（[詳細](agents/README.md)）
2. `bash setup.sh` を再実行（`~/.claude/agents/` へコピーされる）
3. Claude Code を再起動

## 前提条件

- Node.js 18+
- jq（`sudo apt install jq` または `brew install jq`）
- Rust / cargo（mcp-tools ビルド用、`curl https://sh.rustup.rs -sSf | sh` または `mise use rust`）
