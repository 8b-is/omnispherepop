#!/usr/bin/env bash
# Enforce that OWNERS.toml is the only place project identity lives.
# Usage: scripts/sync-owners.sh           # report drift, still exit 0
#        scripts/sync-owners.sh --check   # exit 1 on drift (CI gate)
set -uo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

check_only=0
[[ "${1:-}" == "--check" ]] && check_only=1

drift=0
note() { printf 'drift: %s\n' "$1"; drift=1; }

field() { sed -n "s/^$1 = \"\(.*\)\"\$/\1/p" OWNERS.toml | head -1; }

org=$(field org)
repo=$(field repo)
license_line=$(field license)
copyright_holders=$(sed -n 's/^copyright = "Copyright [0-9-]* //p' OWNERS.toml | head -1 | sed 's/"$//')

handles=()
names=()
set +u
while IFS= read -r line; do
  case "$line" in
    *github*) handles+=("$(printf '%s' "$line" | sed 's/.*"\(.*\)"/\1/')") ;;
    name:*) names+=("$(printf '%s' "$line" | sed 's/.*"\(.*\)"/\1/')") ;;
  esac
done < <(awk '/^\[\[owner\]\]/{f=1;next} /^\[/{f=0} f' OWNERS.toml)

# 1. Workspace metadata.
grep -q "repository = \"$repo\"" Cargo.toml || note "Cargo.toml repository != OWNERS.toml"
grep -q "^license = \"$license_line\"$" Cargo.toml || note "Cargo.toml license != OWNERS.toml"
for n in "${names[@]}"; do
  grep -q "\"$n\"" Cargo.toml || note "Cargo.toml authors missing \"$n\""
done

# 2. CODEOWNERS must name every owner exactly once, in OWNERS.toml order.
expected="* $(printf '@%s ' "${handles[@]}" | sed 's/ $//')"
actual=$(cat .github/CODEOWNERS 2>/dev/null || echo "<missing>")
[[ "$actual" == "$expected" ]] || note ".github/CODEOWNERS expected '$expected', got '$actual'"

# 3. No personal-account references anywhere in tracked sources.
if grep -rq "peterlodrisec" --include='*.rs' --include='*.toml' --include='*.md' --include='*.html' --include='*.yml' --exclude-dir=superpowers . 2>/dev/null; then
  note "personal-account references found (grep peterlodrisec)"
fi

# 4. Both license files must carry the joint copyright.
for f in LICENSE-MIT LICENSE-APACHE; do
  grep -qF "$copyright_holders" "$f" || note "$f missing joint copyright holders ($copyright_holders)"
done

# 5. README must point at the canonical repo, and every mirror must be named.
grep -qF "$repo" README.md || note "README missing canonical repo URL ($repo)"
while IFS= read -r url; do
  [[ -n "$url" ]] || continue
  grep -qF "$url" README.md || note "README missing mirror URL $url"
done < <(awk '/^\[\[mirror\]\]/{f=1;next} /^\[/{f=0} f && /^url = /{sub(/^url = "/,""); sub(/"$/,""); print}' OWNERS.toml)

# 6. The submodule must point at the jointly owned ultra-graph, not a fork.
grep -q "8b-is/ultra-graph" .gitmodules || note ".gitmodules does not point at 8b-is/ultra-graph"

set -u

if [[ $drift -ne 0 ]]; then
  [[ $check_only -eq 1 ]] && { printf 'sync-owners: drift detected\n' >&2; exit 1; }
  printf 'sync-owners: drift detected\n' >&2
  exit 0
fi
printf 'sync-owners: ok (%s, %d owners, %s)\n' "$org" "${#handles[@]}" "$license_line"