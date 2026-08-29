#!/usr/bin/env python3
"""Apply Chat V5 changes directly on production worktree."""

from __future__ import annotations

import datetime
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path("/root/resursmap")


def fail(msg: str) -> None:
    print("APPLY_ABORTED:", msg)
    sys.exit(1)


def read(rel: str) -> str:
    path = ROOT / rel
    if not path.is_file():
        fail(f"missing file {rel}")
    return path.read_text(encoding="utf-8")


def write(rel: str, content: str, backups: pathlib.Path) -> None:
    path = ROOT / rel
    backup = backups / rel.replace("/", "__")
    shutil.copy2(path, backup)
    path.write_text(content, encoding="utf-8")
    print("UPDATED", rel)


def replace_once(content: str, old: str, new: str, label: str) -> str:
    if old not in content:
        fail(f"anchor missing for {label}")
    count = content.count(old)
    if count != 1:
        fail(f"anchor for {label} found {count} times")
    return content.replace(old, new, 1)


def main() -> None:
    if not (ROOT / "Cargo.toml").is_file():
        fail(f"not in project root: {ROOT}")

    head = subprocess.check_output(
        ["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, text=True
    ).strip()
    print("HEAD_BEFORE=" + head)

    status = subprocess.check_output(
        ["git", "status", "--short"], cwd=ROOT, text=True
    ).strip()
    if status:
        fail("worktree is dirty")

    backups = pathlib.Path(
        "/root/resursmap-backups/chat-v50-apply-"
        + datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    )
    backups.mkdir(parents=True, exist_ok=True)

    # --- app_state.rs ---
    app_state = read("src/state/app_state.rs")
    if "ChatTypingEvent" not in app_state:
        app_state = replace_once(
            app_state,
            "use std::{\n    collections::{HashMap, VecDeque},\n    sync::{\n        atomic::{AtomicU64, Ordering},\n        Arc,\n    },\n};",
            "use std::{\n    collections::{HashMap, VecDeque},\n    sync::{\n        atomic::{AtomicU64, Ordering},\n        Arc, Mutex as StdMutex,\n    },\n};",
            "app_state imports",
        )
        app_state = replace_once(
            app_state,
            "impl ChatRealtimeEvent {\n    pub fn includes_user(&self, user_id: i64) -> bool {\n        self.user1_id == user_id || self.user2_id == user_id\n    }\n}\n\n#[derive(Clone)]",
            """impl ChatRealtimeEvent {
    pub fn includes_user(&self, user_id: i64) -> bool {
        self.user1_id == user_id || self.user2_id == user_id
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatTypingEvent {
    pub event_id: u64,
    pub kind: String,
    pub actor_user_id: i64,
    pub other_user_id: i64,
    pub user1_id: i64,
    pub user2_id: i64,
}

impl ChatTypingEvent {
    pub fn includes_user(&self, user_id: i64) -> bool {
        self.user1_id == user_id || self.user2_id == user_id
    }

    pub fn is_visible_to(&self, viewer_user_id: i64) -> bool {
        self.includes_user(viewer_user_id) && self.actor_user_id != viewer_user_id
    }
}

#[derive(Clone)]""",
            "typing event struct",
        )
        app_state = replace_once(
            app_state,
            "    pub chat_events: broadcast::Sender<ChatRealtimeEvent>,\n    pub chat_event_sequence: Arc<AtomicU64>,\n}",
            """    pub chat_events: broadcast::Sender<ChatRealtimeEvent>,
    pub chat_event_sequence: Arc<AtomicU64>,

    // Chat V5 typing bus (ephemeral, process-local).
    pub chat_typing_events: broadcast::Sender<ChatTypingEvent>,
    pub chat_typing_sequence: Arc<AtomicU64>,
    pub chat_typing_rate: Arc<StdMutex<HashMap<String, i64>>>,
}""",
            "app_state fields",
        )
        app_state = replace_once(
            app_state,
            "        let (chat_events, _) = broadcast::channel(2_048);\n\n        Self {",
            "        let (chat_events, _) = broadcast::channel(2_048);\n        let (chat_typing_events, _) = broadcast::channel(1_024);\n\n        Self {",
            "typing channel init",
        )
        app_state = replace_once(
            app_state,
            "            chat_events,\n            chat_event_sequence: Arc::new(AtomicU64::new(0)),\n        }\n    }",
            """            chat_events,
            chat_event_sequence: Arc::new(AtomicU64::new(0)),
            chat_typing_events,
            chat_typing_sequence: Arc::new(AtomicU64::new(0)),
            chat_typing_rate: Arc::new(StdMutex::new(HashMap::new())),
        }
    }""",
            "app_state constructor tail",
        )
        app_state = replace_once(
            app_state,
            "        });\n    }\n}\n\n#[cfg(test)]",
            """        });
    }

    pub fn publish_typing_event(
        &self,
        kind: &str,
        actor_user_id: i64,
        other_user_id: i64,
    ) -> bool {
        if actor_user_id <= 0
            || other_user_id <= 0
            || actor_user_id == other_user_id
            || (kind != "typing.start" && kind != "typing.stop")
        {
            return false;
        }

        let now = crate::web::handlers::common::unix_now();
        let rate_key = format!("typing:{actor_user_id}:{other_user_id}");

        {
            let mut rate_limits = match self.chat_typing_rate.lock() {
                Ok(rate_limits) => rate_limits,
                Err(_) => return false,
            };

            let last_sent = rate_limits.get(&rate_key).copied().unwrap_or(0);
            if now.saturating_sub(last_sent) < 2 {
                return false;
            }

            rate_limits.insert(rate_key, now);
        }

        let (user1_id, user2_id) = if actor_user_id < other_user_id {
            (actor_user_id, other_user_id)
        } else {
            (other_user_id, actor_user_id)
        };

        let event_id = self
            .chat_typing_sequence
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);

        let _ = self.chat_typing_events.send(ChatTypingEvent {
            event_id,
            kind: kind.to_string(),
            actor_user_id,
            other_user_id,
            user1_id,
            user2_id,
        });

        true
    }
}

#[cfg(test)]""",
            "publish typing",
        )
        app_state = replace_once(
            app_state,
            "    use super::ChatRealtimeEvent;\n\n    #[test]\n    fn realtime_event_scope_is_strict() {",
            """    use super::{ChatRealtimeEvent, ChatTypingEvent};

    #[test]
    fn typing_event_is_hidden_from_actor() {
        let event = ChatTypingEvent {
            event_id: 1,
            kind: "typing.start".to_string(),
            actor_user_id: 3,
            other_user_id: 9,
            user1_id: 3,
            user2_id: 9,
        };

        assert!(event.is_visible_to(9));
        assert!(!event.is_visible_to(3));
        assert!(!event.is_visible_to(4));
    }

    #[test]
    fn realtime_event_scope_is_strict() {""",
            "typing test",
        )
    write("src/state/app_state.rs", app_state, backups)

    # --- chat_realtime.rs full replace from bundled content file if present ---
    realtime_path = pathlib.Path(__file__).resolve().parent / "chat_v50_bundle" / "src/web/handlers/chat_realtime.rs"
    if realtime_path.is_file():
        write("src/web/handlers/chat_realtime.rs", realtime_path.read_text(encoding="utf-8"), backups)
    else:
        fail("bundle file required: scripts/chat_v50_bundle/src/web/handlers/chat_realtime.rs")

    handlers = read("src/web/handlers.rs")
    if "api_chat_peer" not in handlers:
        handlers = replace_once(
            handlers,
            "pub use chat_api::{api_chat_delete, api_chat_edit, api_chat_messages, api_chat_send};",
            "pub use chat_api::{api_chat_delete, api_chat_edit, api_chat_messages, api_chat_peer, api_chat_send};",
            "handlers export",
        )
        write("src/web/handlers.rs", handlers, backups)

    routes = read("src/web/routes/communication.rs")
    if "api_chat_peer" not in routes:
        routes = replace_once(
            routes,
            "    accept_contact_request, api_chat_delete, api_chat_edit, api_chat_messages, api_chat_realtime,\n    api_chat_send, api_start_direct_chat, chat_page, contact_requests_page,\n    messages_page, reject_contact_request, send_chat_message,",
            "    accept_contact_request, api_chat_delete, api_chat_edit, api_chat_messages, api_chat_peer,\n    api_chat_realtime, api_chat_send, api_start_direct_chat, chat_page, contact_requests_page,\n    messages_page, reject_contact_request, send_chat_message,",
            "routes import",
        )
        routes = replace_once(
            routes,
            '        .route("/api/chat/{other_user_id}/messages", get(api_chat_messages))\n        .route("/api/chat/realtime", get(api_chat_realtime))',
            '        .route("/api/chat/{other_user_id}/messages", get(api_chat_messages))\n        .route("/api/chat/{other_user_id}/peer", get(api_chat_peer))\n        .route("/api/chat/realtime", get(api_chat_realtime))',
            "routes path",
        )
        write("src/web/routes/communication.rs", routes, backups)

    bundle = pathlib.Path(__file__).resolve().parent / "chat_v50_bundle"
    for rel in [
        "src/web/handlers/chat_api.rs",
        "src/web/templates/communication.rs",
        "static/chat-v2.js",
        "static/chat-v2.css",
    ]:
        source = bundle / rel
        if not source.is_file():
            fail(f"bundle file missing: {source}")
        write(rel, source.read_text(encoding="utf-8"), backups)

    print("BACKUP_DIR=" + str(backups))
    print("APPLY_OK")


if __name__ == "__main__":
    main()
