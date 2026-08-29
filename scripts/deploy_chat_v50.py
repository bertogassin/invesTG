#!/usr/bin/env python3
"""Deploy Chat V5 (typing + presence + dedupe) on production."""

from __future__ import annotations

import datetime
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path("/root/resursmap")
BUNDLE = pathlib.Path(__file__).resolve().parent / "chat_v50_bundle"
CACHE_VERSION = "4.8.0"

FILES = [
    "src/state/app_state.rs",
    "src/web/handlers/chat_realtime.rs",
    "src/web/handlers/chat_api.rs",
    "src/web/handlers.rs",
    "src/web/routes/communication.rs",
    "src/web/templates/communication.rs",
    "static/chat-v2.js",
    "static/chat-v2.css",
    "static/chat-sounds.js",
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

    if not BUNDLE.is_dir():
        fail(f"missing bundle directory {BUNDLE}")

    head = run(["git", "rev-parse", "--short", "HEAD"]).stdout.strip()
    marker("HEAD_BEFORE", head)

    status = run(["git", "status", "--short"]).stdout.strip()
    if status:
        fail(f"worktree is dirty:\n{status}")

    backup_dir = pathlib.Path(
        f"/root/resursmap-backups/chat-v50-before-{datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
    )
    backup_dir.mkdir(parents=True, exist_ok=True)

    for relative in FILES:
        source = BUNDLE / relative
        target = ROOT / relative
        if not source.is_file():
            fail(f"bundle file missing: {source}")
        if not target.is_file():
            fail(f"target file missing: {target}")
        shutil.copy2(target, backup_dir / relative.replace("/", "__"))
        shutil.copy2(source, target)
        print("UPDATED", relative)

    marker("BACKUP_DIR", str(backup_dir))

    run(["cargo", "fmt"])
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

    node = shutil.which("node")
    if node:
        run([node, "--check", "static/chat-v2.js"])
        run([node, "--check", "static/chat-sounds.js"])
    else:
        print("NODE_CHECK=SKIPPED")

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

    service = run(["systemctl", "is-active", "resursmap"]).stdout.strip()
    marker("SERVICE", service)
    marker("HEALTH", "ok")
    marker("CACHE_VERSION", CACHE_VERSION)

    run(["git", "add", *FILES])
    run(["git", "diff", "--cached", "--check"])
    run(
        [
            "git",
            "commit",
            "-m",
            "chat-v5.1: premium chat UX with sounds, haptics and visual polish",
        ]
    )
    run(["git", "push", "origin", "main"])

    head_after = run(["git", "rev-parse", "--short", "HEAD"]).stdout.strip()
    marker("HEAD_AFTER", head_after)
    marker("PUSH", "YES")
    marker("DEPLOY", "COMPLETE")


if __name__ == "__main__":
    main()
