---
title: サブエージェント一覧
description: ~/.claude/agents/ に同期されるサブエージェント定義の管理ディレクトリ
type: readme
tags: [agents, claude-code]
path: agents/README.md
---

# agents/

`.claude/agents/` に配置するサブエージェント定義ファイルを管理するディレクトリ。

`bash setup.sh` を実行すると `~/.claude/agents/` へコピーされ、Claude Code の `Agent` ツールや `/agents` で利用可能になる。

## ファイル形式

各エージェントは Markdown ファイル（`.md`）で定義する。

```markdown
---
name: agent-name          # 英小文字・ハイフン区切り
description: >            # いつ使うかの説明（Claude が自動選択する際の判断基準）
  ここに説明を書く
model: claude-sonnet-4-6  # 省略可（省略時は親から継承）
---

エージェントへのシステムプロンプト（指示・役割・制約など）
```

## エージェント一覧

| ファイル | name | 権限 | 用途 |
|---------|------|------|------|
| [code-reviewer-ja.md](code-reviewer-ja.md) | code-reviewer-ja | 読み取り専用 | PR前の品質・設計レビュー（日本語） |
| [security-reviewer.md](security-reviewer.md) | security-reviewer | 読み取り専用 | 認証・認可・入力検証・暗号化のセキュリティレビュー |
| [debugger.md](debugger.md) | debugger | Read + Bash | エラー・バグの原因調査（実行可・編集不可） |
| [test-writer.md](test-writer.md) | test-writer | フルアクセス | テストケースの設計・実装 |
| [git-workflow-ja.md](git-workflow-ja.md) | git-workflow-ja | フルアクセス | ブランチ作成〜PRまでのGitワークフロー |
| [issue-manager.md](issue-manager.md) | issue-manager | Read + Bash | git-bug を使った issue 作成・管理・GitHub 同期 |
| [doc-manager.md](doc-manager.md) | doc-manager | フルアクセス | タスク・意思決定・ナレッジをドキュメント化し、グラフで管理 |

## エージェントの追加

1. このディレクトリに `.md` ファイルを追加
2. `bash setup.sh` を再実行
3. Claude Code を再起動
