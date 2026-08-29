#!/usr/bin/env bash
set -euo pipefail

cd /root/resursmap

echo "=== CHAT V5.1 PRODUCTION DEPLOY ==="
echo "PWD=$(pwd)"
echo "DATE_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"

if [[ ! -d scripts/chat_v50_bundle ]]; then
  echo "DEPLOY_ABORTED=missing scripts/chat_v50_bundle"
  exit 1
fi

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
sqlite3 data/votes.db 'PRAGMA integrity_check;'
echo "CHAT_V51=COMPLETE"
