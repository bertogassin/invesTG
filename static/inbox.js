(function () {
    "use strict";

    function ready(callback) {
        if (document.readyState === "loading") {
            document.addEventListener("DOMContentLoaded", callback, { once: true });
        } else {
            callback();
        }
    }

    ready(function () {
        var list = document.getElementById("chat-dialog-list");

        if (!list || list.dataset.inboxLive !== "1") {
            return;
        }

        var caption = document.getElementById("inbox-unread-caption");
        var liveBadge = document.getElementById("inbox-live-badge");
        var fetching = false;
        var pollTimer = null;
        var syncTimer = null;
        var socket = null;
        var retryTimer = null;
        var heartbeatTimer = null;
        var retryAttempt = 0;
        var stopped = false;
        var lastSnapshot = "";

        function escapeHtml(value) {
            return String(value || "")
                .replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;")
                .replace(/"/g, "&quot;");
        }

        function setLiveState(online) {
            if (!liveBadge) {
                return;
            }

            liveBadge.hidden = false;
            liveBadge.dataset.state = online ? "online" : "offline";
            liveBadge.textContent = online ? "live" : "sync";
        }

        function conversationKey(conversation) {
            return [
                conversation.other_user_id,
                conversation.updated_at,
                conversation.unread_count,
                conversation.last_message,
                conversation.last_time,
            ].join("|");
        }

    var MESSAGE_ICON =
        '<svg class="icon" viewBox="0 0 24 24"><path d="M21 11.5c0 4.7-4 8.5-9 8.5-1 0-2-.2-2.9-.5L4 21l1.5-4.5C4.5 15.4 3 13.6 3 11.5 3 6.8 7 3 12 3s9 3.8 9 8.5Z"/></svg>';
    var CHEVRON_ICON =
        '<svg class="icon small-icon" viewBox="0 0 24 24"><path d="m9 18 6-6-6-6"/></svg>';

    function renderConversation(conversation) {
            var username = String(conversation.username || "").trim();
            var usernameHtml = username
                ? '<div class="card-meta" style="margin-top:3px;overflow-wrap:anywhere;">@'
                  + escapeHtml(username)
                  + "</div>"
                : "";
            var unreadHtml =
                Number(conversation.unread_count) > 0
                    ? '<span class="chat-dialog-unread">'
                      + escapeHtml(conversation.unread_count)
                      + "</span>"
                    : "";
            var lastTime = conversation.last_time
                ? '<div class="chat-dialog-time">'
                  + escapeHtml(conversation.last_time)
                  + "</div>"
                : '<div class="chat-dialog-time"></div>';

            return (
                '<a href="/app/chat/'
                + encodeURIComponent(conversation.other_user_id)
                + '#chat-end" class="card chat-dialog-card" data-other-user-id="'
                + escapeHtml(conversation.other_user_id)
                + '"><div class="card-icon">'
                + MESSAGE_ICON
                + '</div><div class="card-content"><div class="card-title">'
                + escapeHtml(conversation.display_name)
                + "</div>"
                + usernameHtml
                + '<div class="card-meta">'
                + escapeHtml(conversation.last_message)
                + '</div></div><div class="chat-dialog-side">'
                + lastTime
                + unreadHtml
                + '<div class="card-arrow">'
                + CHEVRON_ICON
                + "</div></div></a>"
            );
        }

        function renderEmptyState() {
            return (
                '<div class="card empty-state-card"><div class="card-content"><div class="card-title">Диалогов пока нет</div><div class="card-meta">После принятия запроса на связь здесь появится внутренний чат.</div></div></div>'
            );
        }

        function applySnapshot(data) {
            var conversations = Array.isArray(data.conversations)
                ? data.conversations
                : [];
            var snapshot = conversations.map(conversationKey).join("\n");

            if (snapshot === lastSnapshot) {
                return;
            }

            lastSnapshot = snapshot;

            if (caption) {
                caption.textContent =
                    "Непрочитанных: " + String(data.total_unread || 0);
            }

            if (typeof window.resursmapRefreshAttentionBadge === "function") {
                window.resursmapRefreshAttentionBadge();
            }

            if (!conversations.length) {
                list.innerHTML = renderEmptyState();
                return;
            }

            list.innerHTML = conversations.map(renderConversation).join("");
        }

        async function refreshInbox() {
            if (fetching || stopped) {
                return;
            }

            fetching = true;

            try {
                var response = await fetch("/api/chat/conversations", {
                    credentials: "same-origin",
                    headers: {
                        Accept: "application/json",
                    },
                });

                if (response.status === 401) {
                    stopped = true;
                    setLiveState(false);
                    return;
                }

                var data = await response.json();

                if (!response.ok || !data.ok) {
                    setLiveState(false);
                    return;
                }

                applySnapshot(data);
            } catch (_) {
                setLiveState(false);
            } finally {
                fetching = false;
            }
        }

        function scheduleSync() {
            window.clearTimeout(syncTimer);

            syncTimer = window.setTimeout(function () {
                syncTimer = null;
                refreshInbox();
            }, 350);
        }

        function websocketUrl() {
            var scheme =
                window.location.protocol === "https:" ? "wss:" : "ws:";

            return (
                scheme + "//" + window.location.host + "/api/chat/realtime"
            );
        }

        function clearSocketTimers() {
            window.clearTimeout(retryTimer);
            window.clearInterval(heartbeatTimer);
            retryTimer = null;
            heartbeatTimer = null;
        }

        function scheduleReconnect() {
            if (stopped || retryTimer) {
                return;
            }

            var delay = Math.min(1000 * Math.pow(2, retryAttempt), 15000);
            retryAttempt = Math.min(retryAttempt + 1, 4);

            retryTimer = window.setTimeout(function () {
                retryTimer = null;
                connectRealtime();
            }, delay);
        }

        function connectRealtime() {
            if (stopped || !window.WebSocket) {
                setLiveState(false);
                return;
            }

            clearSocketTimers();

            if (
                socket &&
                (socket.readyState === WebSocket.OPEN ||
                    socket.readyState === WebSocket.CONNECTING)
            ) {
                return;
            }

            try {
                socket = new WebSocket(websocketUrl());
            } catch (_) {
                setLiveState(false);
                scheduleReconnect();
                return;
            }

            socket.addEventListener("open", function () {
                retryAttempt = 0;
                setLiveState(true);

                heartbeatTimer = window.setInterval(function () {
                    if (!socket || socket.readyState !== WebSocket.OPEN) {
                        return;
                    }

                    socket.send(JSON.stringify({ type: "ping" }));
                }, 20000);
            });

            socket.addEventListener("message", function (event) {
                var payload;

                try {
                    payload = JSON.parse(event.data);
                } catch (_) {
                    return;
                }

                if (
                    payload.type === "chat_event" ||
                    payload.type === "sync_required" ||
                    payload.type === "ready"
                ) {
                    scheduleSync();
                }
            });

            socket.addEventListener("close", function () {
                clearSocketTimers();
                socket = null;
                setLiveState(false);

                if (!stopped) {
                    scheduleReconnect();
                }
            });

            socket.addEventListener("error", function () {
                setLiveState(false);
            });
        }

        function startPollingFallback() {
            window.clearInterval(pollTimer);

            pollTimer = window.setInterval(function () {
                if (
                    socket &&
                    socket.readyState === WebSocket.OPEN
                ) {
                    return;
                }

                refreshInbox();
            }, 60000);
        }

        window.addEventListener("visibilitychange", function () {
            if (document.visibilityState === "visible") {
                refreshInbox();
            }
        });

        window.addEventListener("pagehide", function () {
            stopped = true;
            window.clearInterval(pollTimer);
            window.clearTimeout(syncTimer);
            clearSocketTimers();

            if (socket) {
                socket.close();
                socket = null;
            }
        });

        refreshInbox();
        connectRealtime();
        startPollingFallback();
    });
})();
