#!/usr/bin/env bash
# Wire the three homes this project is published to.
#
# Codeberg is canonical (the repository exists there and ownership is joint).
# GitHub carries two mirrors for reach: the 8b-is org and the personal account.
# All three receive the same tags. URLs come from OWNERS.toml, which is the single
# source of identity; this script never hardcodes them.
#
# Usage: scripts/setup-remotes.sh [--ssh]
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

canonical=$(sed -n 's/^repo = "\(.*\)"$/\1/p' OWNERS.toml | head -1)
scheme=https
while [[ $# -gt 0 ]]; do
  case "$1" in
    --ssh) scheme=git ;;
    *) printf 'unknown flag %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

# Turn a URL into an SSH equivalent when asked: host/path -> git@host:path
to_ssh() {
  printf '%s' "$1" | sed -E 's#^https://([^/]+)/#git@\1:#'
}

name_for() {
  case "$1" in
    *codeberg.org*) printf 'origin' ;;
    */8b-is/*) printf '8b-is' ;;
    */peterlodri-sec/*) printf 'peter' ;;
    *) printf 'mirror' ;;
  esac
}

add_remote() {
  local name="$1" url="$2"
  if [[ "$scheme" == git ]]; then url="$(to_ssh "$url").git"; else url="$url.git"; fi
  if git remote get-url "$name" >/dev/null 2>&1; then
    git remote set-url "$name" "$url"
    printf 'updated %-8s %s\n' "$name" "$url"
  else
    git remote add "$name" "$url"
    printf 'added   %-8s %s\n' "$name" "$url"
  fi
}

add_remote "$(name_for "$canonical")" "$canonical"
while IFS= read -r url; do
  [[ -n "$url" ]] && add_remote "$(name_for "$url")" "$url"
done < <(awk '/^\[\[mirror\]\]/{f=1;next} /^\[/{f=0} f && /^url = /{sub(/^url = "/,""); sub(/"$/,""); print}' OWNERS.toml)

cat <<EOF

Remotes:
EOF
git remote -v | sed 's/^/  /'
cat <<EOF

Publish order — canonical first so the mirrors never lead:

  git push -u origin main
  git push 8b-is main
  git push peter main
  git push origin --tags      # tags only after CI is green on the canonical host

This script wires URLs; it does not create repositories or push anything.
EOF