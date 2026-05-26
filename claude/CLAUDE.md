# Claude Code グローバルガイドライン

## ブランチ戦略

### 命名規則

| プレフィックス | 用途 |
|---|---|
| `feat/<topic>` | 新機能追加 |
| `fix/<topic>` | バグ修正 |
| `hotfix/<topic>` | 本番緊急修正 |
| `chore/<topic>` | 保守・依存更新・設定変更 |
| `refactor/<topic>` | リファクタリング（機能変更なし） |

`<topic>` はケバブケース（例: `feat/sqs-checkin-notification`）。

### 作業開始前のルール

新しいタスクを受けたら、**コードを一切変更する前に**必ず以下の手順を実行する。

1. `git branch --show-current` で現在のブランチを確認する
2. 現在のブランチに関わらず、タスクに対応した新しいブランチを命名規則に従って作成し、そのブランチに切り替えてから作業を開始する
3. 直接 `main` / `master` ブランチにはコミットしない
4. ブランチ作成は作業の**最初のステップ**であり、後回しにしない

### コミット・マージ前の確認ルール

コミットまたはマージを実行する前に、必ず以下の内容をテキストとしてユーザーに提示する。

**出力フォーマット:**

```
## 変更サマリー

### 変更ファイル
- `<ファイルパス>` — <変更の概要>

### 変更の意図
<なぜその変更が必要か、何を解決・実現するかを簡潔に説明>

### コミットメッセージ（案）
<type>(<scope>): <日本語での説明>
```

ユーザーの確認・承認を得てからコミット・マージを実行する。

---

### main / master へのマージルール

マージ方法はリモートリポジトリの有無で切り替える。

**リモートリポジトリがある場合（`git remote` で origin 等が存在する）**

- `main` / `master` へのマージは必ず Pull Request 経由で行う
- **squash merge のみ許可**（コミット履歴を1つにまとめてmainを綺麗に保つ）
- PR タイトルは変更内容を簡潔に表した日本語で記載する
- ブランチのマージ後は作業ブランチを削除する

**ローカルのみの場合（`git remote` が空、またはリモートなし）**

- PR は作成しない
- `git merge <branch>` でローカルマージしてコミットする
- ブランチのマージ後は作業ブランチを削除する

### 作業終了後のルール

作業が完了したら（マージ・ブランチ削除を含む）、必ず `main` / `master` ブランチに戻る。

```bash
git checkout main   # または git checkout master
```

- 次のタスクを受けるときは常に `main` / `master` が起点になるようにする
- 作業ブランチに留まったまま次のタスクを開始しない

---

## MCP ツールの優先使用

以下の操作は Bash の標準コマンドではなく、**mcp-tools の MCP 版を優先して使うこと**。低トークン・高速で動作し、大量出力による文脈汚染を防ぐ。

| 操作 | 非推奨 | MCP ツール（推奨） |
|------|--------------|-----------------|
| ファイル内文字列検索 | `grep` | `mcp-tools: grep` |
| ファイル・ディレクトリ探索 | `find` | `mcp-tools: find` |
| ファイル差分確認 | `diff` | `mcp-tools: diff` |
| Git 差分確認 | `git diff` | `mcp-tools: git_diff` |
| ディレクトリ構造確認 | `tree` | `mcp-tools: directory_tree` |
| ファイルシンボル一覧 | `grep`（関数・クラス抽出） | `mcp-tools: file_outline` |
| Git コミット履歴確認 | `git log` | `mcp-tools: git_log` |
| URL コンテンツ取得 | `WebFetch` | `mcp-tools: web_fetch` |

Bash を使ってよいのは、MCP ツールでは対応できない操作（ビルド・テスト実行・パイプ処理など）に限る。

### web_fetch の使い方

`WebFetch` の代わりに常に `mcp-tools: web_fetch` を使うこと。HTML を Markdown に変換して返すため、トークン数を削減しつつリンク・コードブロック・テーブル構造を保持できる。

```
# ページ全体を取得
web_fetch(url="https://example.com")

# 特定セクションのみ取得（さらにトークン削減）
web_fetch(url="https://docs.rs/anyhow/latest/anyhow/", selector="main")

# 文字数上限を指定
web_fetch(url="https://example.com", max_chars=4000)
```

`selector` には CSS セレクタを指定できる（例: `"main"`, `"article"`, `".content"`）。大規模なリファレンスページでは必ず `selector` で対象を絞ること。

---

## Terraform

`tf_plan` を実行した後は、プラン出力に続けて変更内容を日本語で簡潔に説明すること。

- 追加・変更・削除・再作成されるリソースをリスト形式で列挙する
- 変更がない場合（No changes）は説明不要

---

## サブエージェント

`~/.claude/agents/` に以下のサブエージェントが登録されている。タスクの性質に応じて積極的に活用すること。

| name | 権限 | 使うべき場面 |
|------|------|-------------|
| `code-reviewer-ja` | 読み取り専用 | PR作成前・コード変更後のレビュー依頼 |
| `security-reviewer` | 読み取り専用 | 認証・認可・入力検証・API実装後のセキュリティ確認 |
| `debugger` | Read + Bash | エラー・テスト失敗・予期しない挙動の原因調査 |
| `test-writer` | フルアクセス | 新機能・バグ修正に対するテストケースの作成 |
| `git-workflow-ja` | フルアクセス | ブランチ作成・コミット・PR作成のGitワークフロー全般 |

### 呼び出し方

- **自然言語**: 「debugger サブエージェントを使ってこのエラーを調査して」
- **@メンション**: `@debugger このスタックトレースの原因を調べて`
- **セッション全体**: `claude --agent debugger`

### 使い分けの原則

- 調査・レビュー系（`code-reviewer-ja`・`security-reviewer`・`debugger`）は読み取り専用または Bash 限定で安全に実行できる
- 実装系（`test-writer`・`git-workflow-ja`）はファイル編集が必要なため、事前に計画をユーザーに提示してから実行する

### 管理

エージェント定義は `~/mcp-suite/agents/` で管理されており、`bash ~/mcp-suite/setup.sh` で `~/.claude/agents/` へ反映される。
