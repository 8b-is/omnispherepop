#!/usr/bin/env bash
# Install the Omnispherepop agent plugin: skills into the user skill directory,
# MCP wiring printed for the harness to merge.
#
# Usage: scripts/install-agent-plugin.sh [--skills-dir DIR] [--print-only]
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
skills_dir="${HOME}/.agents/skills"
print_only=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skills-dir) skills_dir="$2"; shift 2 ;;
    --print-only) print_only=1; shift ;;
    *) printf 'unknown flag %s\n' "$1" >&2; exit 2 ;;
  esac
done

if ! command -v ospop-mcp >/dev/null 2>&1; then
  printf 'note: ospop-mcp not on PATH; build it with `cargo build --release -p ospop-mcp`\n' >&2
fi

if [[ $print_only -eq 0 ]]; then
  mkdir -p "$skills_dir"
  for skill in "$root"/agent-plugin/skills/*/; do
    name="$(basename "$skill")"
    target="$skills_dir/$name"
    rm -rf "$target"
    cp -R "$skill" "$target"
    printf 'installed skill: %s\n' "$target"
  done
fi

cat <<'EOF'

MCP wiring — merge this into your harness config (example: agent-plugin/.mcp.json):

  {
    "mcpServers": {
      "omnispherepop": { "command": "ospop-mcp", "args": [], "env": {} }
    }
  }

Then restart the agent session so the tools register. Verify with:

  printf '%s\n' \
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
    '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
    | ospop-mcp | tail -1

EOF