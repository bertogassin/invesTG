#!/usr/bin/env bash
# Unified production deploy entry point for ResursMap VPS.
set -euo pipefail

ROOT="${RESURSMAP_ROOT:-/root/resursmap}"
cd "$ROOT"

echo "DEPLOY=START"
echo "ROOT=$ROOT"

git fetch origin main
git pull --ff-only origin main

bash scripts/run_after_git_pull.sh
