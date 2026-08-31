#!/usr/bin/env python3
"""Post-pull production verify for ResursMap (git pull is the source of truth)."""

from __future__ import annotations

import datetime
import pathlib
import subprocess
import sys

ROOT = pathlib.Path("/root/resursmap")
REPO = pathlib.Path(__file__).resolve().parent.parent
CACHE_VERSION = "4.9.11"

FILES = [
    "src/state/app_state.rs",
    "src/web/handlers/chat_realtime.rs",
    "src/web/handlers/chat_api.rs",
    "src/web/handlers.rs",
    "src/web/routes/communication.rs",
    "src/web/templates/communication.rs",
    "src/web/templates/common.rs",
    "static/chat-v2.js",
    "static/chat-v2.css",
    "static/chat-sounds.js",
    "static/inbox.js",
]


def run(cmd: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    print("+", " ".join(cmd))
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        check=check,
        capture_output=True,
    )


def fail(message: str) -> None:
    print("DEPLOY_ABORTED:", message)
    sys.exit(1)


def marker(name: str, value: str) -> None:
    print(f"{name}={value}")


def main() -> None:
    if not ROOT.is_dir():
        fail(f"missing project root {ROOT}")

    for relative in FILES:
        if not (ROOT / relative).is_file():
            fail(f"missing deployed file: {relative}")
        if not (REPO / relative).is_file():
            fail(f"missing repo file: {relative}")

    head = run(["git", "rev-parse", "--short", "HEAD"]).stdout.strip()
    marker("HEAD_BEFORE", head)

    status = run(["git", "status", "--short"]).stdout.strip()
    if status:
        fail(f"worktree is dirty:\n{status}")

    backup_dir = pathlib.Path(
        f"/root/resursmap-backups/verify-before-{datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
    )
    backup_dir.mkdir(parents=True, exist_ok=True)
    marker("BACKUP_DIR", str(backup_dir))

    run(["cargo", "fmt", "--", "--check"])
    run(["cargo", "check"])
    run(["cargo", "test"])
    run(
        [
            "cargo",
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ]
    )
    run(["cargo", "build", "--release"])
    run(["git", "diff", "--check"])

    run(["sqlite3", "data/votes.db", "PRAGMA integrity_check;"])
    run(["sqlite3", "data/votes.db", "PRAGMA foreign_key_check;"])

    run(["systemctl", "restart", "resursmap"])

    health_ok = False
    for _ in range(30):
        probe = run(["curl", "-fsS", "http://127.0.0.1:3000/health"], check=False)
        if probe.returncode == 0 and probe.stdout.strip() == "ok":
            health_ok = True
            break
        run(["sleep", "1"], check=False)

    if not health_ok:
        fail("health check did not return ok within 30 seconds")

    marker("SERVICE", run(["systemctl", "is-active", "resursmap"]).stdout.strip())
    marker("HEALTH", "ok")
    marker("CACHE_VERSION", CACHE_VERSION)
    marker("DEPLOY", "COMPLETE")


if __name__ == "__main__":
    main()
