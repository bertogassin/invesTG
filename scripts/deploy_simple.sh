#!/usr/bin/env bash
set -euo pipefail

cd /root/resursmap

echo "=== SIMPLE DEPLOY ==="
echo "HEAD=$(git rev-parse --short HEAD)"

git pull --ff-only origin main

cargo fmt -- --check
cargo check
cargo test
cargo build --release

if command -v node >/dev/null 2>&1; then
  node --check static/chat-v2.js
  node --check static/chat-sounds.js
  node --check static/inbox.js
fi

sqlite3 data/votes.db 'PRAGMA integrity_check;'

systemctl restart resursmap

health_ok=false
for _ in $(seq 1 30); do
  if curl -fsS http://127.0.0.1:3000/health | grep -qx ok; then
    health_ok=true
    break
  fi
  sleep 1
done

if [[ "$health_ok" != true ]]; then
  echo "DEPLOY_ABORTED=health_failed"
  exit 1
fi

echo "HEAD_AFTER=$(git rev-parse --short HEAD)"
echo "SERVICE=$(systemctl is-active resursmap)"
echo "HEALTH=ok"
echo "CACHE_VERSION=4.9.33"
echo "DEPLOY=COMPLETE"
