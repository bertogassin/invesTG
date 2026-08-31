#!/usr/bin/env bash
# Verify STATIC_ASSET_VERSION is synchronized across the repo.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CANONICAL="$(grep 'pub const STATIC_ASSET_VERSION' src/web/templates/common.rs | sed -n 's/.*"\([^"]*\)".*/\1/p')"

if [[ -z "$CANONICAL" ]]; then
  echo "VERSION_CHECK=FAIL (missing STATIC_ASSET_VERSION)"
  exit 1
fi

echo "CANONICAL=$CANONICAL"
FAIL=0

check_file() {
  local file="$1"
  local pattern="$2"
  if [[ ! -f "$file" ]]; then
    echo "MISSING $file"
    FAIL=1
    return
  fi
  if ! grep -q "$pattern" "$file"; then
    echo "MISMATCH $file (expected $pattern)"
    FAIL=1
  else
    echo "OK $file"
  fi
}

check_file "static/resursmap-sw.js" "resursmap-shell-v${CANONICAL}"
check_file "scripts/deploy_simple.sh" "CACHE_VERSION=${CANONICAL}"
check_file "scripts/run_after_git_pull.sh" "CACHE_VERSION=${CANONICAL}"
check_file "scripts/run_chat_v50_production.sh" "CACHE_VERSION=${CANONICAL}"
check_file "scripts/deploy_chat_v50.py" "CACHE_VERSION = \"${CANONICAL}\""

if [[ "$FAIL" -ne 0 ]]; then
  echo "VERSION_CHECK=FAIL"
  exit 1
fi

echo "VERSION_CHECK=OK"
