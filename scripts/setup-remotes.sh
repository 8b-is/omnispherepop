#!/usr/bin/env bash
# Wire the three homes this project is published to.
#
# GitHub carries two mirrors because ownership is joint: the org is canonical,
# the personal account is a co-owner mirror. Codeberg mirrors the repository for
# anyone who wants a non-US host. All three receive the same tags.
#
# Usage: scripts/setup-remotes.sh [--with-codeberg] [--ssh]
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

org=$(sed -n 's/^org = "\(.*\)"$/\1/p' OWNERS.toml | head -1)
scheme=https
with_codeberg=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --with-codeberg) with_codeberg=1; shift ;;
    --ssh) scheme=git; shift ;;
    *) printf 'unknown flag %s\n' "$1" >&2; exit 2 ;;
  esac
done

if [[ "$scheme" == ssh ]]; then
  base="git@github.com"
  cb="git@codeberg.org"
else
  base="https://github.com"
  cb="https://codeberg.org"
fi

add_remote() {
  local name="$1" url="$2"
  if git remote get-url "$name" >/dev/null 2>&1; then
    git remote set-url "$name" "$url"
    printf 'updated %s -> %s\n' "$name" "$url"
  else
    git remote add "$name" "$url"
    printf 'added   %s -> %s\n' "$name" "$url"
  fi
}

add_remote origin "$base/$org/omnispherepop.git"
add_remote peter "$base/peterlodri-sec/omnispherepop.git"
if [[ $with_codeberg -eq 1 ]]; then
  add_remote codeberg "$cb/$org/omnispherepop.git"
fi

cat <<EOF

Remotes:
$(git remote -v)

Publishing order matters. The org repository is canonical, so push there first:

  git push -u origin main --tags
  git push peter main --tags
EOF
[[ $with_codeberg -eq 1 ]] && printf '  git push codeberg main --tags\n'
cat <<'EOF'

Each remote must exist before you push. Creating them is an owner action: the org
repo needs an owner of the GitHub organization, and the Codeberg mirror needs an
owner account there. This script only wires URLs; it never creates repositories.
EOF