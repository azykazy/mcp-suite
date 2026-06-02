---
title: Claude Code グローバルガイドライン
description: Claude Code 全プロジェクト共通のブランチ戦略・MCPツール・Terraform設定
type: config
tags: [claude-code, guidelines, global]
path: claude/CLAUDE.md
---

# Claude Code グローバルガイドライン

## Markdown ファイル作成ルール

新しい `.md` ファイルを作成する際は、必ずファイル先頭に以下の frontmatter を付与すること。

```yaml
---
title: "表示用タイトル"
description: "ファイルの用途の一言説明"
type: readme | doc | config | agent
tags: [タグ1, タグ2]
path: リポジトリルートからの相対パス（例: docs/example.md）
---
```

| フィールド | 必須 | 説明 |
|---|---|---|
| `title` | ✓ | 人間が読みやすい表示用タイトル |
| `description` | ✓ | ファイルの用途・目的の一言説明 |
| `type` | ✓ | `readme` / `doc` / `config` / `agent` のいずれか |
| `tags` | ✓ | 内容を表すキーワードのリスト |
| `path` | ✓ | リポジトリルートからの相対パス |

---

## インフラプラットフォームの質問ルール

AWS・GCP・Azure・Terraform・Kubernetes・Cloudflare 等の主要インフラプラットフォームに関する質問には、**必ず回答前に公式ドキュメントの最新情報を取得すること**。

- `mcp-tools: web_fetch` または `context7` で最新ドキュメントを参照する
- 学習データのみによる回答は禁止（仕様・料金・制限は頻繁に変わるため）
- 回答には必ず参照元 URL を明示する

対象キーワード例: EC2, S3, RDS, Lambda, CloudFront, EKS, ECS, DynamoDB, SQS, IAM, VPC, BigQuery, GKE, Azure, Terraform, Kubernetes など

---

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
2. `main` / `master` 以外にいる場合は `git checkout main` で切り替える
3. リモートがある場合は `git pull` で最新状態を取得する
4. タスクに対応した新しいブランチを命名規則に従って作成し、そのブランチに切り替えてから作業を開始する
5. 直接 `main` / `master` ブランチにはコミットしない
6. ブランチ作成は作業の**最初のステップ**であり、後回しにしない

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

`tf_apply` で `plan_file` を指定した場合は、apply 成功後にそのファイルを削除すること。

---

## ドキュメント更新

タスクが完了したら、コミット・PR 作成の前に関連ドキュメントを探して更新する。

1. `README.md`・`docs/` ディレクトリ・その他ドキュメントファイルを検索する
2. 変更内容を反映すべきドキュメントがあれば更新する
3. 更新対象がなければスキップしてよい

---

## ドキュメント記録

以下の3種類のドキュメントをメインエージェントが直接作成する。IDは対象ディレクトリの既存ファイルを確認してから当日の連番で採番する。

| type | ID形式 | 保存先 | 作成タイミング |
|------|-------|--------|--------------|
| `task` | `TASK-YYYYMMDD-NNN` | `docs/tasks/` | コミットまたはPR作成完了後 |
| `decision` | `DEC-YYYYMMDD-NNN` | `docs/decisions/` | 技術的な意思決定をした直後 |
| `knowledge` | `KNW-YYYYMMDD-NNN` | `docs/knowledge/` | 調査・検証で得た知見が生まれた直後 |

**task テンプレート:**

```markdown
---
title: "<タスクの概要>"
description: "<一行説明>"
type: doc
tags: [task]
path: docs/tasks/<ID>.md
doc_id: "<ID>"
doc_type: task
status: completed
---

# <タスクの概要>

## 実行内容
<何をしたか>

## 結果
<コミットハッシュまたは PR URL>
```

**decision テンプレート:**

```markdown
---
title: "<意思決定のタイトル>"
description: "<一行説明>"
type: doc
tags: [decision]
path: docs/decisions/<ID>.md
doc_id: "<ID>"
doc_type: decision
status: accepted
---

# <意思決定のタイトル>

## 状況
<どんな背景・課題があったか>

## 決定
<何を選んだか>

## 理由
<なぜそう決めたか>

## トレードオフ
<失ったもの・懸念点>
```

**knowledge テンプレート:**

```markdown
---
title: "<ナレッジのタイトル>"
description: "<一行説明>"
type: doc
tags: [knowledge]
path: docs/knowledge/<ID>.md
doc_id: "<ID>"
doc_type: knowledge
source: "<情報源URL（あれば）>"
---

# <ナレッジのタイトル>

## 概要
<要点>

## 詳細
<調査・検証内容>

## 活用場面
<いつ使えるか>
```

---

## Issue 管理（git-bug）

issue・チケットの管理には **git-bug** を使うこと。TODO.md への追記は行わない。メインエージェントが直接コマンドを実行する。

### 基本コマンド

```bash
# issue 作成
git-bug bug new --title "タイトル" --message "詳細"

# issue 詳細表示（状態確認は ls ではなく必ず show を使う）
git-bug bug show <id>

# ステータスをクローズに変更
git-bug bug status close <id>

# コメント追加
git-bug bug comment add <id> --message "コメント"

# GitHub Issues から同期（pull）
git-bug bridge pull github

# ローカル → GitHub Issues に反映（push）
git-bug bridge push github
```

### GitHub bridge 認証

bridge 設定時は手動 PAT を発行せず、**`gh auth token` で取得したトークンを使うこと**。

```bash
git-bug bridge new \
    --name=github \
    --target=github \
    --owner=<owner> \
    --project=<repo> \
    --token="$(gh auth token)" \
    --non-interactive
```

