#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLAUDE_SETTINGS="$HOME/.claude/settings.json"
CLAUDE_JSON="$HOME/.claude.json"
MCP_CONFIG="$REPO_DIR/config/mcp_settings.json"
ENV_FILE="$REPO_DIR/.env"
export MCP_SUITE_DIR="$REPO_DIR"

echo "=== mcp-suite setup ==="

# 前提チェック
check_deps() {
  local missing=()
  command -v node >/dev/null 2>&1 || missing+=("node")
  command -v npx  >/dev/null 2>&1 || missing+=("npx")
  command -v jq   >/dev/null 2>&1 || missing+=("jq")
  if [ ${#missing[@]} -ne 0 ]; then
    echo "ERROR: 以下のツールが見つかりません: ${missing[*]}"
    echo "  node/npx: https://nodejs.org"
    echo "  jq: sudo apt install jq  または  brew install jq"
    exit 1
  fi
  echo "[OK] 依存ツール確認済み (node=$(node -v), jq=$(jq --version))"
}

# .env の読み込み
load_env() {
  if [ -f "$ENV_FILE" ]; then
    set -a
    # shellcheck disable=SC1090
    source "$ENV_FILE"
    set +a
    echo "[OK] .env を読み込みました"
  else
    echo "[WARN] .env が見つかりません。.env.example をコピーして編集してください:"
    echo "  cp $REPO_DIR/.env.example $ENV_FILE"
    echo "  vi $ENV_FILE"
    echo ""
    echo "  続行しますが、環境変数が未設定のMCPは動作しません。"
  fi
}

# Claude Code v2: ~/.claude.json に mcpServers をマージ
# （v1 では ~/.claude/settings.json だったが v2 で移動）
configure_claude() {
  # 環境変数を展開した一時ファイルを作成（node使用でmacOS互換）
  local tmp_config
  tmp_config=$(mktemp)
  node -e "
    const c = require('fs').readFileSync(process.argv[1], 'utf8');
    process.stdout.write(c.replace(/\\\${(\w+)}/g, (_, k) => process.env[k] ?? ''));
  " "$MCP_CONFIG" > "$tmp_config"

  if [ ! -f "$CLAUDE_JSON" ]; then
    echo '{}' > "$CLAUDE_JSON"
  fi

  # 既存設定をバックアップしてからマージ
  local backup="$CLAUDE_JSON.bak.$(date +%Y%m%d%H%M%S)"
  cp "$CLAUDE_JSON" "$backup"
  echo "[OK] 設定バックアップ: $backup"

  jq --slurpfile mcp "$tmp_config" '.mcpServers = $mcp[0].mcpServers' \
    "$CLAUDE_JSON" > "$tmp_config.merged"
  mv "$tmp_config.merged" "$CLAUDE_JSON"
  rm -f "$tmp_config"

  echo "[OK] mcpServers を $CLAUDE_JSON に設定しました"
}

# カスタムMCPのビルド
build_custom_mcps() {
  local custom_dir="$REPO_DIR/custom"
  if [ ! -d "$custom_dir" ] || [ -z "$(ls -A "$custom_dir" 2>/dev/null)" ]; then
    echo "[SKIP] custom/ にMCPが見つかりません"
    return
  fi

  for mcp_dir in "$custom_dir"/*/; do
    [ -d "$mcp_dir" ] || continue
    local name
    name=$(basename "$mcp_dir")
    echo "  ビルド: $name"

    if [ -f "$mcp_dir/package.json" ]; then
      (cd "$mcp_dir" && npm install && npm run build 2>/dev/null || true)
    elif [ -f "$mcp_dir/Cargo.toml" ]; then
      if command -v cargo >/dev/null 2>&1; then
        (cd "$mcp_dir" && cargo build --release)
        echo "  [OK] $name: Rustビルド完了"
      else
        echo "  [SKIP] $name: cargo が見つかりません。rustup で Rust をインストールしてください。"
      fi
    elif [ -f "$mcp_dir/go.mod" ]; then
      (cd "$mcp_dir" && go build ./...)
    else
      echo "  [SKIP] $name: ビルド方法が不明（package.json / Cargo.toml / go.mod なし）"
    fi
  done
}

# サブエージェントを ~/.claude/agents/ へインストール
install_agents() {
  local agents_dir="$REPO_DIR/agents"
  local target_dir="$HOME/.claude/agents"

  if [ ! -d "$agents_dir" ] || [ -z "$(ls -A "$agents_dir"/*.md 2>/dev/null)" ]; then
    echo "[SKIP] agents/ にエージェント定義が見つかりません"
    return
  fi

  mkdir -p "$target_dir"

  for agent_file in "$agents_dir"/*.md; do
    [ -f "$agent_file" ] || continue
    local name
    name=$(basename "$agent_file")
    [ "$name" = "README.md" ] && continue
    cp "$agent_file" "$target_dir/$name"
    echo "  [OK] エージェント導入: $name"
  done

  echo "[OK] サブエージェントを $target_dir に設定しました"
}

main() {
  check_deps
  load_env
  build_custom_mcps
  configure_claude
  install_agents
  echo ""
  echo "=== セットアップ完了 ==="
  echo "Claude Code を再起動してMCP・サブエージェントを有効化してください。"
}

main "$@"
