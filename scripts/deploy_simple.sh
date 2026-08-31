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

for i in $(seq 1 30); do
  curl -fsS http://127.0.0.1:3000/health && break
  sleep 1
done

echo "HEAD_AFTER=$(git rev-parse --short HEAD)"
echo "SERVICE=$(systemctl is-active resursmap)"
echo "CACHE_VERSION=4.9.19"
echo "DEPLOY=COMPLETE"
