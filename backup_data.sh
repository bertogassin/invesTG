#!/bin/bash
set -e
DIR=/root/resursmap
STAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p "$DIR/backups"
tar -czf "$DIR/backups/data_$STAMP.tar.gz" -C "$DIR" data
# keep last 20
ls -1t "$DIR/backups"/data_*.tar.gz 2>/dev/null | tail -n +21 | xargs -r rm -f
echo "backup ok $STAMP"
