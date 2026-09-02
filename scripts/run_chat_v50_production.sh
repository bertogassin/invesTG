#!/usr/bin/env bash
set -euo pipefail

resursmap_sqlite_path() {
  local url="${DATABASE_URL:-sqlite:data/votes.db}"
  printf '%s\n' "${url#sqlite:}"
}

cd /root/resursmap

echo "=== RESURSMAP PRODUCTION DEPLOY ==="
echo "PWD=$(pwd)"
echo "DATE_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"

git pull --ff-only origin main

if [[ ! -f scripts/deploy_chat_v50.py ]]; then
  echo "DEPLOY_ABORTED=missing scripts/deploy_chat_v50.py"
  exit 1
fi

python3 scripts/deploy_chat_v50.py

echo "=== POST DEPLOY MARKERS ==="
git rev-parse --short HEAD
git status --short --branch
systemctl is-active resursmap
curl -fsS http://127.0.0.1:3000/health
sqlite3 "$(resursmap_sqlite_path)" 'PRAGMA integrity_check;'
echo "CACHE_VERSION=4.9.54"
echo "DEPLOY=COMPLETE"
