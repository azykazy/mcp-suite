# mcp-tools TODO

Rust 製 MCP ツールの拡充計画。出力サイズ制御によるトークン削減を主目的とする。

---

## 高優先度

### `git_log`
- [x] 実装
- コンパクト形式（hash・日付・author・件名）に絞った出力
- パラメータ: `repo`, `path`（ファイル絞り込み）, `limit`（件数）, `from`/`to`（範囲）
- `git log` のそのまま出力に比べてトークンを大幅削減

### `file_outline`
- [x] 実装
- 関数・クラス・型定義のシグネチャだけを返す（ファイル全体 Read 不要）
- tree-sitter を使って多言語対応（Rust, TypeScript, Python, Go など）
- コード全体把握のコストを最も削減できる

### `directory_tree`
- [x] 実装
- `.gitignore` を尊重したコンパクトなツリー表示
- パラメータ: `path`, `max_depth`, `ignore`（追加除外パターン）
- ディレクトリ構造の把握を1回のツール呼び出しで完結させる

---

## 中優先度

### `read_range`
- [ ] 実装
- ファイルの指定行範囲だけを返す
- パラメータ: `path`, `start`, `end`
- 大きなファイルで `Read` ツールより細かく制御できる

### `git_log_file`
- [ ] 実装
- 特定ファイルの変更履歴をコンパクトに返す
- `git_log` の `path` 絞り込み特化版（`git log -- <file>` 相当）

---

## 低優先度

- 外部コマンドをほぼそのままラップするだけになるもの（`cat`, `wc` 等）は効果薄のため保留

---

## 完了済み

- [x] `grep` — コンパクト `path:line: content` 形式、`.gitignore` 対応
- [x] `find` — glob/部分一致でファイル探索
- [x] `diff` — ファイル間差分
- [x] `git_diff` — ワーキングツリー・ステージ・コミット範囲の差分
- [x] `git_log` — コンパクト形式のコミット履歴（hash・日付・author・件名）
