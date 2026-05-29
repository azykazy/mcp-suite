---
title: mcp-tools TODO
description: Rust 製 MCP ツールの拡充計画・実装状況トラッキング
type: doc
tags: [todo, mcp-tools, rust]
path: TODO.md
---

# mcp-tools TODO

Rust 製 MCP ツールの拡充計画。出力サイズ制御によるトークン削減を主目的とする。

---

## 高優先度

（現在なし）

---

## 中優先度

（現在なし）

---

## キャンセル済み

- `read_range` — `Read` ツールの `offset`/`limit` で代替可能なため不要
- `git_log_file` — `git_log` の `path` パラメータで既にカバー済みのため不要

---

## 完了済み

- [x] `grep` — コンパクト `path:line: content` 形式、`.gitignore` 対応
- [x] `find` — glob/部分一致でファイル探索
- [x] `diff` — ファイル間差分
- [x] `git_diff` — ワーキングツリー・ステージ・コミット範囲の差分
- [x] `git_log` — コンパクト形式のコミット履歴（hash・日付・author・件名）
- [x] `file_outline` — 関数・クラス・型定義のシグネチャ抽出（.rs .py .ts .js .go 対応）
- [x] `directory_tree` — `.gitignore` を尊重したコンパクトなツリー表示
