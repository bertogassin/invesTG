#!/usr/bin/env bash
set -euo pipefail

cd /root/resursmap
BUNDLE=scripts/chat_v50_bundle

echo "=== SIMPLE DEPLOY ==="
echo "HEAD=$(git rev-parse --short HEAD)"

cp "$BUNDLE/src/state/app_state.rs" src/state/
cp "$BUNDLE/src/web/handlers/chat_realtime.rs" src/web/handlers/
cp "$BUNDLE/src/web/handlers/chat_api.rs" src/web/handlers/
cp "$BUNDLE/src/web/handlers.rs" src/web/
cp "$BUNDLE/src/web/routes/communication.rs" src/web/routes/
cp "$BUNDLE/src/web/templates/communication.rs" src/web/templates/
cp "$BUNDLE/static/chat-v2.js" static/
cp "$BUNDLE/static/chat-v2.css" static/
cp "$BUNDLE/static/chat-sounds.js" static/

echo "FILES=UPDATED"

cargo fmt
cargo check
cargo test
cargo build --release

node --check static/chat-v2.js
node --check static/chat-sounds.js

sqlite3 data/votes.db 'PRAGMA integrity_check;'

systemctl restart resursmap

for i in $(seq 1 30); do
  curl -fsS http://127.0.0.1:3000/health && break
  sleep 1
done

git add \
  src/state/app_state.rs \
  src/web/handlers/chat_realtime.rs \
  src/web/handlers/chat_api.rs \
  src/web/handlers.rs \
  src/web/routes/communication.rs \
  src/web/templates/communication.rs \
  static/chat-v2.js \
  static/chat-v2.css \
  static/chat-sounds.js

git commit -m "chat-v5.1: premium chat UX with sounds, haptics and visual polish"
git push origin main

echo "HEAD_AFTER=$(git rev-parse --short HEAD)"
echo "SERVICE=$(systemctl is-active resursmap)"
echo "DEPLOY=COMPLETE"
