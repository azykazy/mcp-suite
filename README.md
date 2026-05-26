# mcp-suite

自作 & OSS MCP と Claude Code / Codex 設定をまとめた個人用セットアップリポジトリ。

## クイックスタート

```bash
git clone https://github.com/azykazy/mcp-suite.git
cd mcp-suite
bash setup.sh
```

Claude Code / Codex を再起動するとMCP・サブエージェントが有効になります。

## 構成

```
mcp-suite/
├── setup.sh              # セットアップスクリプト（1本で完結）
├── config/
│   └── mcp_settings.json # MCPサーバー設定テンプレート
├── codex/                # Codex 設定・サブエージェント・hooks
├── custom/               # 自作MCPサーバー（ソースコード管理）
│   └── mcp-tools/        # grep / diff / find（Rust実装）
├── oss/                  # OSSのMCP情報・参照
│   ├── context7/         # ドキュメント取得MCP
│   └── playwright/       # ブラウザ操作MCP
└── agents/               # Claude Code サブエージェント定義
    ├── code-reviewer-ja.md  # PR前の品質・設計レビュー（読み取り専用）
    ├── security-reviewer.md # セキュリティレビュー（読み取り専用）
    ├── debugger.md          # バグ・エラー原因調査（Bash可・編集不可）
    ├── test-writer.md       # テストケース設計・実装
    └── git-workflow-ja.md   # Gitワークフロー担当
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

## 含まれるサブエージェント

| エージェント | 権限 | 用途 |
|------------|------|------|
| code-reviewer-ja | 読み取り専用 | PR前の品質・設計レビュー（日本語） |
| security-reviewer | 読み取り専用 | OWASP Top 10 ベースのセキュリティレビュー |
| debugger | Read + Bash | エラー・バグの原因調査 |
| test-writer | フルアクセス | テストケースの設計・実装 |
| git-workflow-ja | フルアクセス | ブランチ作成〜PRのGitワークフロー |

詳細は [agents/README.md](agents/README.md) を参照。

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
