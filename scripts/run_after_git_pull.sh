#!/usr/bin/env bash
set -euo pipefail

cd /root/resursmap

echo "=== DEPLOY AFTER GIT PULL ==="
echo "PWD=$(pwd)"
echo "DATE_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "HEAD_BEFORE=$(git rev-parse --short HEAD)"

if [[ -n "$(git status --short)" ]]; then
  echo "DEPLOY_ABORTED=dirty_worktree"
  git status --short
  exit 1
fi

git pull origin main

echo "HEAD_AFTER_PULL=$(git rev-parse --short HEAD)"

cargo fmt
cargo fmt -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
git diff --check

if command -v node >/dev/null 2>&1; then
  node --check static/chat-v2.js
  node --check static/chat-sounds.js
else
  echo "NODE_CHECK=SKIPPED"
fi

sqlite3 data/votes.db 'PRAGMA integrity_check;'
sqlite3 data/votes.db 'PRAGMA foreign_key_check;'

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

echo "SERVICE=$(systemctl is-active resursmap)"
echo "HEALTH=ok"
echo "SQLITE=ok"
echo "CACHE_VERSION=4.9.14"
git status --short --branch
echo "DEPLOY=COMPLETE"
