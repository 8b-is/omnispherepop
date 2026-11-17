#!/usr/bin/env bash
# Generate the Academy pages from the CLI, so the site cannot drift from the code.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cargo build -q -p ospop-cli

cargo run -q -p ospop-cli -- learn --json > /tmp/ospop-curriculum.json
mkdir -p site/academy site/assets
cp /tmp/ospop-curriculum.json site/assets/curriculum.json

node scripts/gen-academy.mjs site/assets/curriculum.json site/academy
printf 'generated %s lesson pages\n' "$(ls site/academy/*.html 2>/dev/null | wc -l | tr -d ' ')"