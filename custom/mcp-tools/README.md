# mcp-tools

Rustで実装した軽量MCPサーバー。grep・diff・findをトークン効率よく提供します。

## ツール一覧

### grep
ファイルまたはディレクトリからregexパターンを検索します。

```json
{
  "pattern": "fn main",
  "paths": ["src"],
  "context": 2,
  "ignore_case": false,
  "max_matches": 100
}
```

出力形式（context=0）:
```
src/main.rs:1: fn main() {
```

出力形式（context>0）:
```
 src/main.rs:1: use std::io;
>src/main.rs:2: fn main() {
 src/main.rs:3:     let x = 1;
```

### diff
2ファイルを比較してunified diff形式で差分を返します。

```json
{
  "a": "path/to/file_a.rs",
  "b": "path/to/file_b.rs",
  "context": 3
}
```

### find
ディレクトリ以下のファイル・ディレクトリを検索します。

```json
{
  "path": "src",
  "pattern": "*.rs",
  "type": "f",
  "max_depth": 3,
  "max_results": 200
}
```

## ビルド

```bash
cargo build --release
# バイナリ: target/release/mcp-tools
```

## 設計方針

- **低トークン**: デフォルト context=0 で一致行のみ出力
- **軽量**: 外部プロセスなし、シングルバイナリ（~2MB）
- **高速**: Rustネイティブバイナリ、起動コスト最小
- **バイナリファイル自動スキップ**: null バイト検出で除外
