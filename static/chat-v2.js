(function () {
    "use strict";

    function ready(callback) {
        if (document.readyState === "loading") {
            document.addEventListener(
                "DOMContentLoaded",
                callback,
                { once: true }
            );
        } else {
            callback();
        }
    }

    ready(function () {
        var history = document.getElementById(
            "chat-messages"
        );
        var form = document.getElementById("chat-form");
        var input = document.getElementById("chat-input");
        var send = document.getElementById("chat-send");
        var clear = document.getElementById("chat-clear");
        var counter = document.getElementById(
            "chat-counter"
        );
        var sendState = document.getElementById(
            "chat-send-state"
        );
        var connectionState = document.getElementById(
            "chat-connection-state"
        );
        var peerState = document.getElementById(
            "chat-peer-state"
        );
        var headerStatus = document.getElementById(
            "chat-header-status"
        );
        var headerPresenceDot = document.getElementById(
            "chat-header-presence-dot"
        );
        var scrollBottomButton = document.getElementById(
            "chat-scroll-bottom"
        );
        var loadOlder = document.getElementById(
            "chat-load-older"
        );
        var historyStart = document.getElementById(
            "chat-history-start"
        );
        var historyEnd = document.getElementById(
            "chat-end"
        );

        if (
            !history ||
            !form ||
            !input ||
            !send ||
            !clear ||
            !counter ||
            !connectionState ||
            !loadOlder
        ) {
            return;
        }

        var otherUserId = String(
            history.dataset.otherUserId || ""
        ).trim();

        if (
            !/^[1-9][0-9]{0,18}$/.test(otherUserId)
        ) {
            return;
        }

        var firstMessageId = Number(
            history.dataset.firstMessageId || "0"
        );
        var lastMessageId = Number(
            history.dataset.lastMessageId || "0"
        );
        var mayHaveOlder =
            history.dataset.mayHaveOlder === "1";
        var polling = false;
        var sending = false;
        var loadingOlder = false;
        var pollTimer = null;
        var draftKey =
            "resursmap-chat-draft:" + otherUserId;
        var pendingSendKey =
            "resursmap-chat-outbox:" + otherUserId;
        var legacyPendingSendKey =
            "resursmap-chat-pending:" + otherUserId;
        var pendingQueue = [];
        var peerOnline = false;
        var peerLastSeenAt = 0;
        var peerTyping = false;
        var peerTypingTimer = null;
        var typingStopTimer = null;
        var presenceTimer = null;
        var lastTypingEmitAt = 0;

        function createClientMessageId() {
            if (
                window.crypto &&
                typeof window.crypto.randomUUID ===
                    "function"
            ) {
                return window.crypto.randomUUID();
            }

            return (
                "fallback_" +
                Date.now().toString(36) +
                "_" +
                Math.random()
                    .toString(36)
                    .slice(2, 14)
            );
        }

        function savePendingQueue() {
            try {
                if (pendingQueue.length > 0) {
                    localStorage.setItem(
                        pendingSendKey,
                        JSON.stringify(pendingQueue)
                    );
                } else {
                    localStorage.removeItem(
                        pendingSendKey
                    );
                }

                localStorage.removeItem(
                    legacyPendingSendKey
                );
            } catch (_) {
                // Storage may be disabled.
            }
        }

        function setConnection(text, className) {
            connectionState.textContent = text;
            connectionState.classList.remove(
                "is-online",
                "is-error"
            );

            if (className) {
                connectionState.classList.add(className);
            }

            updatePeerState();
        }

        function formatLastSeen(timestamp) {
            if (!Number.isFinite(timestamp) || timestamp <= 0) {
                return "давно";
            }

            var now = Math.floor(Date.now() / 1000);
            var delta = Math.max(0, now - timestamp);

            if (delta < 60) {
                return "только что";
            }

            if (delta < 3600) {
                return Math.floor(delta / 60) + " мин назад";
            }

            if (delta < 86400) {
                return Math.floor(delta / 3600) + " ч назад";
            }

            return Math.floor(delta / 86400) + " д назад";
        }

        function typingDotsHtml(label) {
            return (
                '<span class="chat-typing-dots" aria-hidden="true">' +
                "<i></i><i></i><i></i></span> " +
                label
            );
        }

        function syncHeaderPresence() {
            if (headerPresenceDot) {
                headerPresenceDot.hidden = !(peerOnline || peerTyping);
                headerPresenceDot.classList.toggle(
                    "is-typing",
                    peerTyping
                );
            }

            if (!headerStatus) {
                return;
            }

            headerStatus.classList.remove(
                "is-online",
                "is-typing"
            );

            if (peerTyping) {
                headerStatus.innerHTML = typingDotsHtml("печатает…");
                headerStatus.classList.add("is-typing");
                return;
            }

            if (peerOnline) {
                headerStatus.textContent = "онлайн";
                headerStatus.classList.add("is-online");
                return;
            }

            var defaultSubtitle =
                headerStatus.dataset.defaultSubtitle || "";

            if (peerLastSeenAt > 0) {
                headerStatus.textContent =
                    "был(а) " + formatLastSeen(peerLastSeenAt);
                return;
            }

            headerStatus.textContent =
                defaultSubtitle || "не в сети";
        }

        function updatePeerState() {
            if (peerState) {
                peerState.hidden = false;
                peerState.classList.remove(
                    "is-online",
                    "is-typing"
                );

                if (peerTyping) {
                    peerState.innerHTML = typingDotsHtml("печатает…");
                    peerState.classList.add("is-typing");
                } else if (peerOnline) {
                    peerState.textContent = "онлайн";
                    peerState.classList.add("is-online");
                } else {
                    peerState.textContent =
                        "был(а) " + formatLastSeen(peerLastSeenAt);
                }
            }

            syncHeaderPresence();
        }

        function setPeerTyping(active) {
            peerTyping = Boolean(active);

            if (peerTypingTimer) {
                window.clearTimeout(peerTypingTimer);
                peerTypingTimer = null;
            }

            if (peerTyping) {
                peerTypingTimer = window.setTimeout(function () {
                    peerTyping = false;
                    updatePeerState();
                }, 5200);
            }

            updatePeerState();
        }

        function reconcilePendingForMessage(message) {
            var clientId = String(
                message.client_message_id || ""
            ).trim();

            if (!clientId) {
                return;
            }

            pendingQueue = pendingQueue.filter(function (item) {
                return item.clientMessageId !== clientId;
            });

            removePendingRow(clientId);
            savePendingQueue();
        }

        function emitTypingSignal(kind) {
            document.dispatchEvent(
                new CustomEvent(
                    "resursmap:chat-realtime-send",
                    {
                        detail: {
                            type: kind,
                            other_user_id: otherUserId
                        }
                    }
                )
            );
        }

        function scheduleTypingSignals() {
            var now = Date.now();

            if (now - lastTypingEmitAt >= 1800) {
                lastTypingEmitAt = now;
                emitTypingSignal("typing.start");
            }

            if (typingStopTimer) {
                window.clearTimeout(typingStopTimer);
            }

            typingStopTimer = window.setTimeout(function () {
                typingStopTimer = null;
                emitTypingSignal("typing.stop");
            }, 2800);
        }

        async function refreshPeerPresence() {
            try {
                var data = await fetchJson(
                    "/api/chat/" + otherUserId + "/peer"
                );

                peerOnline = Boolean(data.online);
                peerLastSeenAt = Number(data.last_seen_at || 0);
                updatePeerState();
            } catch (_) {
                // Presence is best-effort.
            }
        }

        function updateViewportHeight() {
            var viewport = window.visualViewport;
            var height = viewport
                ? viewport.height
                : window.innerHeight;

            var offsetTop = viewport
                ? viewport.offsetTop
                : 0;

            var bottomInset = viewport
                ? Math.max(
                    0,
                    window.innerHeight -
                        viewport.height -
                        viewport.offsetTop
                )
                : 0;

            document.documentElement.style.setProperty(
                "--chat-viewport-height",
                Math.round(height) + "px"
            );

            document.documentElement.style.setProperty(
                "--chat-viewport-offset-top",
                Math.round(offsetTop) + "px"
            );

            document.documentElement.style.setProperty(
                "--chat-keyboard-inset",
                Math.round(bottomInset) + "px"
            );
        }

        function autoResize() {
            input.style.height = "auto";

            var nextHeight = Math.min(
                Math.max(input.scrollHeight, 48),
                150
            );

            input.style.height = nextHeight + "px";
        }

        function updateComposer() {
            var length = Array.from(input.value).length;

            counter.textContent = length + " / 2000";
            counter.classList.toggle(
                "is-near-limit",
                length >= 1800 && length < 2000
            );
            counter.classList.toggle(
                "is-limit",
                length >= 2000
            );

            clear.hidden = length === 0;
            send.disabled =
                input.value.trim().length === 0 ||
                length > 2000;

            autoResize();

            try {
                if (input.value) {
                    localStorage.setItem(
                        draftKey,
                        input.value
                    );
                } else {
                    localStorage.removeItem(draftKey);
                }
            } catch (_) {
                // Storage may be disabled.
            }
        }

        function updateScrollBottomButton() {
            if (!scrollBottomButton) {
                return;
            }

            scrollBottomButton.hidden = nearBottom();
        }

        var markReadTimer = null;

        function markReadAtBottomDebounced() {
            if (!nearBottom() || lastMessageId <= 0) {
                return;
            }

            if (markReadTimer) {
                window.clearTimeout(markReadTimer);
            }

            markReadTimer = window.setTimeout(function () {
                fetchJson(
                    "/api/chat/" +
                    otherUserId +
                    "/messages?limit=1&mark_read=1&read_through_id=" +
                    lastMessageId
                )
                    .then(function (data) {
                        updateReadStatuses(
                            Number(data.peer_read_through_id || 0)
                        );
                    })
                    .catch(function () {
                        // Best-effort read sync.
                    });
            }, 350);
        }

        function scrollToBottom(behavior) {
            history.scrollTo({
                top: history.scrollHeight,
                behavior: behavior || "auto"
            });
            updateScrollBottomButton();
        }

        function nearBottom() {
            return (
                history.scrollHeight -
                history.scrollTop -
                history.clientHeight
            ) < 130;
        }

        function formatTime(timestamp) {
            var date = new Date(timestamp * 1000);

            if (Number.isNaN(date.getTime())) {
                return "";
            }

            return new Intl.DateTimeFormat("ru", {
                hour: "2-digit",
                minute: "2-digit"
            }).format(date);
        }

        function createMessageRow(message, animate) {
            var row = document.createElement("div");
            var bubble = document.createElement("div");
            var body = document.createElement("div");
            var meta = document.createElement("div");
            var time = document.createElement("span");
            var status = document.createElement("span");
            var mine = Boolean(message.is_mine);

            row.className = "chat-message-row" +
                (mine ? " is-mine" : " is-theirs");

            row.dataset.messageId = String(message.id);
            row.dataset.mine = mine ? "1" : "0";

            if (animate) {
                row.classList.add("is-new");
            }

            bubble.className = "chat-bubble";
            body.className = "chat-message-body";
            body.textContent = String(message.message || "");
            if (message.attachment_kind === "voice" && message.attachment_url) {
                body.textContent = "";
                body.classList.add("chat-message-body--voice");
                var voicePlayer = document.createElement("div");
                voicePlayer.className = "chat-voice-player";
                var voiceAudio = document.createElement("audio");
                voiceAudio.className = "chat-voice-audio";
                voiceAudio.controls = true;
                voiceAudio.preload = "metadata";
                voiceAudio.src = String(message.attachment_url);
                voicePlayer.appendChild(voiceAudio);
                body.appendChild(voicePlayer);
            } else if (message.attachment_kind === "image" && message.attachment_url) {
                var img = document.createElement("img");
                img.className = "chat-message-image";
                img.src = String(message.attachment_url);
                img.alt = "Фото";
                img.loading = "lazy";
                img.setAttribute("role", "button");
                img.tabIndex = 0;
                body.textContent = "";
                body.appendChild(img);
                if (message.message) {
                    var cap = document.createElement("div");
                    cap.className = "chat-message-caption";
                    cap.textContent = String(message.message);
                    body.appendChild(cap);
                }
            }
            meta.className = "chat-message-meta";
            time.textContent = formatTime(
                Number(message.created_at)
            );

            status.className = "chat-message-status";

            if (mine) {
                if (Number(message.read_at) > 0) {
                    status.textContent = "✓✓";
                    status.classList.add("is-read");
                } else if (
                    Number(message.delivered_at) > 0
                ) {
                    status.textContent = "✓✓";
                } else {
                    status.textContent = "✓";
                }
            }

            meta.appendChild(time);
            meta.appendChild(status);
            bubble.appendChild(body);
            bubble.appendChild(meta);
            row.appendChild(bubble);

            return row;
        }

        function pendingRow(clientMessageId) {
            var found = null;

            history.querySelectorAll(
                ".chat-message-row[data-client-message-id]"
            ).forEach(function (row) {
                if (
                    row.dataset.clientMessageId ===
                    clientMessageId
                ) {
                    found = row;
                }
            });

            return found;
        }

        function removePendingRow(clientMessageId) {
            var row = pendingRow(clientMessageId);

            if (row) {
                row.remove();
            }
        }

        function renderPendingItem(item) {
            var row = pendingRow(
                item.clientMessageId
            );

            if (!row) {
                row = createMessageRow(
                    {
                        id:
                            "pending-" +
                            item.clientMessageId,
                        message: item.message,
                        is_mine: true,
                        delivered_at: 0,
                        read_at: 0,
                        created_at: item.createdAt
                    },
                    true
                );

                row.dataset.clientMessageId =
                    item.clientMessageId;
                row.classList.add("is-pending");

                history.insertBefore(row, historyEnd);
            }

            var status = row.querySelector(
                ".chat-message-status"
            );

            if (!status) {
                return;
            }

            status.classList.remove(
                "is-read",
                "is-error"
            );
            status.classList.add("is-pending");

            if (item.state === "error" || item.state === "failed") {
                row.classList.add("is-send-error");
                row.classList.remove("is-sending");
                status.classList.add("is-error");
                status.textContent = "!";
                status.title =
                    "Не отправлено · нажмите повторить";
                status.setAttribute(
                    "role",
                    "button"
                );
                status.tabIndex = 0;

                status.onclick = function (event) {
                    event.stopPropagation();
                    item.state = "queued";
                    savePendingQueue();
                    renderPendingItem(item);
                    flushPendingQueue();
                };

                status.onkeydown = function (event) {
                    if (
                        event.key === "Enter" ||
                        event.key === " "
                    ) {
                        event.preventDefault();
                        status.click();
                    }
                };
            } else {
                row.classList.remove(
                    "is-send-error"
                );
                row.classList.add("is-sending");
                status.textContent = "…";
                status.title = "Отправляется";
                status.removeAttribute("role");
                status.removeAttribute("tabindex");
                status.onclick = null;
                status.onkeydown = null;
            }
        }

        function renderPendingQueue() {
            pendingQueue.forEach(
                renderPendingItem
            );

            if (pendingQueue.length > 0) {
                scrollToBottom("auto");
            }
        }

        function messageExists(id) {
            return Boolean(history.querySelector(
                '.chat-message-row[data-message-id="' +
                String(id) +
                '"]'
            ));
        }

        function updateReadStatuses(readThroughId) {
            if (!Number.isSafeInteger(readThroughId) ||
                readThroughId <= 0) {
                return;
            }

            history.querySelectorAll(
                '.chat-message-row[data-mine="1"]'
            ).forEach(function (row) {
                var id = Number(row.dataset.messageId);

                if (id <= readThroughId) {
                    var status = row.querySelector(
                        ".chat-message-status"
                    );

                    if (status) {
                        status.textContent = "✓✓";
                        status.classList.add("is-read");
                    }
                }
            });
        }

        function notifyMessagesRendered(messages) {
            if (!messages || !messages.length) {
                return;
            }

            document.dispatchEvent(
                new CustomEvent(
                    "resursmap:chat-messages-render",
                    {
                        detail: { messages: messages }
                    }
                )
            );
        }

        function appendMessages(messages) {
            var shouldStick = nearBottom();
            var incomingCount = 0;

            messages.forEach(function (message) {
                var id = Number(message.id);

                if (!Number.isSafeInteger(id) ||
                    id <= 0 ||
                    messageExists(id)) {
                    return;
                }

                reconcilePendingForMessage(message);

                history.insertBefore(
                    createMessageRow(message, true),
                    historyEnd
                );

                firstMessageId =
                    firstMessageId > 0
                        ? Math.min(firstMessageId, id)
                        : id;
                lastMessageId =
                    Math.max(lastMessageId, id);

                if (!message.is_mine) {
                    incomingCount += 1;
                }
            });

            history.dataset.firstMessageId =
                String(firstMessageId);
            history.dataset.lastMessageId =
                String(lastMessageId);

            if (shouldStick || incomingCount > 0) {
                scrollToBottom(
                    incomingCount > 0 ? "smooth" : "auto"
                );
            }

            if (
                incomingCount > 0 &&
                document.visibilityState === "visible"
            ) {
                if (typeof window.playChatReceive === "function") {
                    window.playChatReceive();
                } else if (
                    typeof window.playNotificationSound === "function"
                ) {
                    window.playNotificationSound();
                }
            }

            updateScrollBottomButton();
            notifyMessagesRendered(messages);
        }

        function prependMessages(messages) {
            if (!messages.length) {
                return;
            }

            var previousHeight = history.scrollHeight;
            var fragment = document.createDocumentFragment();

            messages.forEach(function (message) {
                var id = Number(message.id);

                if (!Number.isSafeInteger(id) ||
                    id <= 0 ||
                    messageExists(id)) {
                    return;
                }

                reconcilePendingForMessage(message);

                fragment.appendChild(
                    createMessageRow(message, false)
                );

                firstMessageId =
                    firstMessageId > 0
                        ? Math.min(firstMessageId, id)
                        : id;
                lastMessageId =
                    Math.max(lastMessageId, id);
            });

            history.insertBefore(fragment, historyStart.nextSibling);
            history.dataset.firstMessageId =
                String(firstMessageId);

            history.scrollTop +=
                history.scrollHeight - previousHeight;

            notifyMessagesRendered(messages);
        }

        async function fetchJson(url, options) {
            var response = await fetch(url, {
                credentials: "same-origin",
                cache: "no-store",
                headers: Object.assign(
                    {
                        "Accept": "application/json"
                    },
                    options && options.headers
                        ? options.headers
                        : {}
                ),
                method:
                    options && options.method
                        ? options.method
                        : "GET",
                body:
                    options && options.body
                        ? options.body
                        : undefined
            });

            var data = await response.json().catch(function () {
                return {
                    ok: false,
                    error: "invalid_response"
                };
            });

            if (!response.ok || !data.ok) {
                var error = new Error(
                    data.error || "request_failed"
                );

                error.status = response.status;
                error.retryAfter =
                    Number(data.retry_after || 0);

                throw error;
            }

            return data;
        }

        async function pollMessages(force) {
            if (
                polling ||
                document.visibilityState === "hidden" ||
                (
                    force !== true &&
                    document.documentElement.dataset
                        .chatRealtime === "online"
                )
            ) {
                return;
            }

            polling = true;

            try {
                var markReadQuery = nearBottom()
                    ? (
                        "&mark_read=1&read_through_id=" +
                        Math.max(lastMessageId, 0)
                    )
                    : "&mark_read=0";

                var data = await fetchJson(
                    "/api/chat/" +
                    otherUserId +
                    "/messages?after_id=" +
                    Math.max(lastMessageId, 0) +
                    "&limit=100" +
                    markReadQuery
                );

                appendMessages(data.messages || []);
                updateReadStatuses(
                    Number(data.peer_read_through_id || 0)
                );
                setConnection("В сети", "is-online");
            } catch (error) {
                if (error.status === 401) {
                    setConnection(
                        "Требуется повторный вход",
                        "is-error"
                    );
                    window.clearInterval(pollTimer);
                } else {
                    setConnection(
                        "Связь восстанавливается…",
                        "is-error"
                    );
                }
            } finally {
                polling = false;
                window.__resursmapChatLastPollAt = Date.now();
            }
        }

        async function loadOlderMessages() {
            if (
                loadingOlder ||
                !mayHaveOlder ||
                firstMessageId <= 0
            ) {
                return;
            }

            loadingOlder = true;
            loadOlder.disabled = true;
            loadOlder.textContent = "Загрузка…";

            try {
                var data = await fetchJson(
                    "/api/chat/" +
                    otherUserId +
                    "/messages?before_id=" +
                    firstMessageId +
                    "&limit=50"
                );

                prependMessages(data.messages || []);
                mayHaveOlder = Boolean(data.has_more);
                loadOlder.hidden = !mayHaveOlder;
                loadOlder.textContent =
                    mayHaveOlder
                        ? "Загрузить предыдущие сообщения"
                        : "Начало переписки";
                setConnection("В сети", "is-online");
            } catch (_) {
                loadOlder.textContent =
                    "Не удалось загрузить · Повторить";
                setConnection(
                    "Ошибка загрузки истории",
                    "is-error"
                );
            } finally {
                loadingOlder = false;
                loadOlder.disabled = false;
            }
        }

        var MAX_AUTO_RETRIES = 5;
        var RETRY_BASE_DELAY_MS = 1000;
        var RETRY_MAX_DELAY_MS = 20000;
        var retryTimers = {};

        function clearItemRetryTimer(clientMessageId) {
            if (retryTimers[clientMessageId]) {
                window.clearTimeout(retryTimers[clientMessageId]);
                delete retryTimers[clientMessageId];
            }
        }

        function isRecoverableError(error) {
            if (!error || typeof error.status !== "number") {
                return true;
            }
            if (error.status === 429) {
                return true;
            }
            return error.status >= 500;
        }

        function scheduleRetryWithDelay(item, delayMs) {
            clearItemRetryTimer(item.clientMessageId);
            item.state = "retrying";
            savePendingQueue();
            renderPendingItem(item);
            retryTimers[item.clientMessageId] = window.setTimeout(function () {
                delete retryTimers[item.clientMessageId];
                item.state = "queued";
                savePendingQueue();
                renderPendingItem(item);
                flushPendingQueue();
            }, delayMs);
        }

        function scheduleRetry(item) {
            var delay = Math.min(
                RETRY_BASE_DELAY_MS * Math.pow(2, item.attempts),
                RETRY_MAX_DELAY_MS
            );
            delay = delay + Math.floor(Math.random() * 300);
            scheduleRetryWithDelay(item, delay);
        }

        async function flushPendingQueue() {
        if (sending || navigator.onLine === false) {
            return;
        }
        sending = true;
        updateComposer();
        try {
            var index = 0;
            while (index < pendingQueue.length) {
                var item = pendingQueue[index];
                if (item.state !== "queued") {
                    index += 1;
                    continue;
                }
                item.attempts = (item.attempts || 0) + 1;
                item.state = "sending";
                savePendingQueue();
                renderPendingItem(item);
                sendState.textContent =
                    pendingQueue.length > 1
                        ? "Отправка \u00b7 осталось " + pendingQueue.length
                        : "Отправка\u2026";
                try {
                    var data = await fetchJson(
                        "/api/chat/" + otherUserId + "/send",
                        {
                            method: "POST",
                            headers: { "Content-Type": "application/json" },
                            body: JSON.stringify({
                                message: item.message,
                                reply_to_message_id: item.replyToMessageId,
                                client_message_id: item.clientMessageId
                            })
                        }
                    );
                    if (data.message) {
                        if (
                            !data.message.reply_to_message_id &&
                            item.replyToMessageId
                        ) {
                            data.message.reply_to_message_id =
                                item.replyToMessageId;
                        }

                        data.message.reply_message =
                            item.replyMessage || "";
                        data.message.reply_sender_user_id =
                            item.replySenderUserId || null;
                    }
                    clearItemRetryTimer(item.clientMessageId);
                    removePendingRow(item.clientMessageId);
                    var sentIndex = pendingQueue.indexOf(item);
                    if (sentIndex !== -1) {
                        pendingQueue.splice(sentIndex, 1);
                    }
                    savePendingQueue();
                    if (data.message) {
                        appendMessages([data.message]);
                    }
                    if (typeof window.playChatSend === "function") {
                        window.playChatSend();
                    }
                    window.dispatchEvent(new CustomEvent("resursmap:chat-message-sent"));
                    setConnection("\u0412 \u0441\u0435\u0442\u0438", "is-online");
                    index = 0;
                } catch (error) {
                    var recoverable = isRecoverableError(error);
                    var exhausted = item.attempts >= MAX_AUTO_RETRIES;
                    if (recoverable && !exhausted) {
                        if (error.status === 429 && error.retryAfter > 0) {
                            sendState.textContent =
                                "\u041b\u0438\u043c\u0438\u0442 \u00b7 \u043f\u043e\u0432\u0442\u043e\u0440 \u0447\u0435\u0440\u0435\u0437 " + error.retryAfter + " \u0441\u0435\u043a.";
                            scheduleRetryWithDelay(item, error.retryAfter * 1000);
                        } else {
                            sendState.textContent = "\u041f\u043e\u0432\u0442\u043e\u0440 \u0447\u0435\u0440\u0435\u0437 \u043d\u0435\u0441\u043a\u043e\u043b\u044c\u043a\u043e \u0441\u0435\u043a\u0443\u043d\u0434\u2026";
                            scheduleRetry(item);
                        }
                        setConnection("\u041e\u0448\u0438\u0431\u043a\u0430 \u043e\u0442\u043f\u0440\u0430\u0432\u043a\u0438", "is-error");
                    } else {
                        item.state = "failed";
                        savePendingQueue();
                        renderPendingItem(item);
                        if (error.status === 401) {
                            sendState.textContent = "\u0421\u0435\u0441\u0441\u0438\u044f \u0438\u0441\u0442\u0435\u043a\u043b\u0430";
                        } else {
                            sendState.textContent = "\u041d\u0435 \u043e\u0442\u043f\u0440\u0430\u0432\u043b\u0435\u043d\u043e \u00b7 \u043d\u0430\u0436\u043c\u0438\u0442\u0435 !";
                        }
                        setConnection("\u041e\u0448\u0438\u0431\u043a\u0430 \u043e\u0442\u043f\u0440\u0430\u0432\u043a\u0438", "is-error");
                        if (typeof window.playChatError === "function") {
                            window.playChatError();
                        }
                    }
                    index += 1;
                }
            }
            if (pendingQueue.length === 0) {
                sendState.textContent = "\u041e\u0442\u043f\u0440\u0430\u0432\u043b\u0435\u043d\u043e \u00b7 Enter \u2014 \u043e\u0442\u043f\u0440\u0430\u0432\u0438\u0442\u044c";
            }
        } finally {
            sending = false;
            updateComposer();
        }
    }

        function sendMessage() {
            if (
                window.ResursMapChatForward &&
                typeof window.resursmapSendChatForward === "function"
            ) {
                window.resursmapSendChatForward();
                return;
            }

            var message = input.value.trim();

            if (!message) {
                return;
            }

            var reply =
                window.ResursMapChatReply || null;

            var item = {
                clientMessageId:
                    createClientMessageId(),
                message: message,
                replyToMessageId:
                    reply ? reply.id : null,
                replyMessage:
                    reply ? reply.message : "",
                replySenderUserId:
                    reply ? reply.senderUserId : null,
                createdAt:
                    Math.floor(Date.now() / 1000),
                state: "queued",
            attempts: 0
            };

            pendingQueue.push(item);
            savePendingQueue();
            renderPendingItem(item);

            input.value = "";
            updateComposer();
            input.focus();

            window.dispatchEvent(
                new CustomEvent(
                    "resursmap:chat-message-queued"
                )
            );

            scrollToBottom("smooth");
            flushPendingQueue();
        }

        var imageInput = document.getElementById("chat-image-input");
        var imageBtn = document.getElementById("chat-image-btn");

        if (!imageInput) {
            imageInput = document.createElement("input");
            imageInput.type = "file";
            imageInput.accept = "image/jpeg,image/png,image/webp";
            imageInput.id = "chat-image-input";
            imageInput.hidden = true;
            form.appendChild(imageInput);
        }

        if (!imageBtn) {
            imageBtn = document.createElement("button");
            imageBtn.type = "button";
            imageBtn.id = "chat-image-btn";
            imageBtn.textContent = "📷";
            imageBtn.className = "chat-image-btn";
            if (send && send.parentNode) {
                send.parentNode.insertBefore(imageBtn, send);
            }
        }

        imageBtn.addEventListener("click", function () {
            imageInput.click();
        });

        imageInput.addEventListener("change", function () {
            var file = imageInput.files && imageInput.files[0];
            imageInput.value = "";
            if (!file) return;
            if (file.size > 8 * 1024 * 1024) {
                setConnection("Фото больше 8 МБ", "is-error");
                return;
            }
            var clientMessageId = createClientMessageId();
            var formData = new FormData();
            formData.append("image", file);
            formData.append("client_message_id", clientMessageId);
            formData.append("caption", input.value.trim());
            var reply = window.ResursMapChatReply || null;
            if (reply && reply.id) {
                formData.append("reply_to_message_id", String(reply.id));
            }
            sendState.textContent = "Отправка фото…";
            fetch("/api/chat/" + otherUserId + "/send-image", {
                method: "POST",
                body: formData,
                credentials: "same-origin"
            }).then(function (res) { return res.json().then(function (data) { return { res: res, data: data }; }); })
              .then(function (pack) {
                if (!pack.res.ok || !pack.data || !pack.data.ok) {
                    throw new Error((pack.data && pack.data.error) || "send_failed");
                }
                input.value = "";
                updateComposer();
                if (pack.data.message) {
                    appendMessages([pack.data.message]);
                }
                window.ResursMapChatReply = null;
                var replyBarEl =
                    document.getElementById("chat-reply-bar");
                if (replyBarEl) {
                    replyBarEl.hidden = true;
                }
                setConnection("В сети", "is-online");
                sendState.textContent = "Отправлено · Enter — отправить";
              }).catch(function () {
                setConnection("Ошибка фото", "is-error");
                sendState.textContent = "Фото не отправлено";
              });
        });

        var voiceBtn = document.getElementById("chat-voice-btn");
        var voiceRecording = false;
        var voiceRecorder = null;
        var voiceChunks = [];
        var voiceStartedAt = 0;
        var voiceTimerId = null;
        var voiceStopTimerId = null;
        var voiceOverlay = document.createElement("div");
        voiceOverlay.id = "chat-voice-recording";
        voiceOverlay.className = "chat-voice-recording";
        voiceOverlay.hidden = true;
        voiceOverlay.innerHTML =
            '<div class="chat-voice-recording-inner">' +
                '<div class="chat-voice-recording-pulse"></div>' +
                '<span class="chat-voice-recording-label">Запись…</span>' +
                '<span class="chat-voice-recording-timer">0:00</span>' +
            '</div>';
        if (form) {
            form.appendChild(voiceOverlay);
        }

        function formatVoiceDuration(ms) {
            var total = Math.max(0, Math.floor(ms / 1000));
            var minutes = Math.floor(total / 60);
            var seconds = total % 60;
            return minutes + ":" + String(seconds).padStart(2, "0");
        }

        function updateVoiceTimer() {
            var timer = voiceOverlay.querySelector(
                ".chat-voice-recording-timer"
            );
            if (timer) {
                timer.textContent = formatVoiceDuration(
                    Date.now() - voiceStartedAt
                );
            }
        }

        function hideVoiceRecording() {
            voiceRecording = false;
            voiceOverlay.hidden = true;
            if (voiceBtn) {
                voiceBtn.classList.remove("is-recording");
            }
            if (voiceTimerId) {
                window.clearInterval(voiceTimerId);
                voiceTimerId = null;
            }
            if (voiceStopTimerId) {
                window.clearTimeout(voiceStopTimerId);
                voiceStopTimerId = null;
            }
        }

        function sendVoiceBlob(blob, mimeType) {
            if (!blob || !blob.size) {
                return;
            }
            var clientMessageId = createClientMessageId();
            var formData = new FormData();
            formData.append("voice", blob, "voice.webm");
            formData.append("client_message_id", clientMessageId);
            var reply = window.ResursMapChatReply || null;
            if (reply && reply.id) {
                formData.append("reply_to_message_id", String(reply.id));
            }
            sendState.textContent = "Отправка голосового…";
            fetch("/api/chat/" + otherUserId + "/send-voice", {
                method: "POST",
                body: formData,
                credentials: "same-origin"
            }).then(function (res) {
                return res.json().then(function (data) {
                    return { res: res, data: data };
                });
            }).then(function (pack) {
                if (!pack.res.ok || !pack.data || !pack.data.ok) {
                    throw new Error(
                        (pack.data && pack.data.error) || "send_failed"
                    );
                }
                if (pack.data.message) {
                    appendMessages([pack.data.message]);
                }
                window.ResursMapChatReply = null;
                var replyBarEl =
                    document.getElementById("chat-reply-bar");
                if (replyBarEl) {
                    replyBarEl.hidden = true;
                }
                setConnection("В сети", "is-online");
                sendState.textContent = "Отправлено · Enter — отправить";
                if (typeof window.playChatSend === "function") {
                    window.playChatSend();
                }
            }).catch(function () {
                setConnection("Ошибка голосового", "is-error");
                sendState.textContent = "Голосовое не отправлено";
                if (typeof window.playChatError === "function") {
                    window.playChatError();
                }
            });
        }

        function stopVoiceRecording(sendIt) {
            if (!voiceRecording || !voiceRecorder) {
                hideVoiceRecording();
                return;
            }

            var recorder = voiceRecorder;
            var duration = Date.now() - voiceStartedAt;
            voiceRecorder = null;
            hideVoiceRecording();

            recorder.onstop = function () {
                var mimeType =
                    recorder.mimeType || "audio/webm";
                var blob = new Blob(voiceChunks, {
                    type: mimeType
                });
                voiceChunks = [];
                if (sendIt && duration >= 600 && blob.size > 0) {
                    sendVoiceBlob(blob, mimeType);
                } else if (sendIt && duration < 600) {
                    sendState.textContent = "Слишком короткая запись";
                }
            };

            if (recorder.state !== "inactive") {
                recorder.stop();
            }

            if (recorder.stream) {
                recorder.stream.getTracks().forEach(function (track) {
                    track.stop();
                });
            }
        }

        function startVoiceRecording() {
            if (voiceRecording || !voiceBtn) {
                return;
            }

            if (
                !navigator.mediaDevices ||
                typeof navigator.mediaDevices.getUserMedia !== "function" ||
                typeof MediaRecorder === "undefined"
            ) {
                sendState.textContent =
                    "Запись голоса недоступна в этом браузере";
                return;
            }

            navigator.mediaDevices.getUserMedia({ audio: true })
                .then(function (stream) {
                    voiceChunks = [];
                    var mimeType = "";
                    if (MediaRecorder.isTypeSupported(
                        "audio/webm;codecs=opus"
                    )) {
                        mimeType = "audio/webm;codecs=opus";
                    } else if (MediaRecorder.isTypeSupported("audio/webm")) {
                        mimeType = "audio/webm";
                    } else if (MediaRecorder.isTypeSupported("audio/ogg")) {
                        mimeType = "audio/ogg";
                    }

                    voiceRecorder = mimeType
                        ? new MediaRecorder(stream, { mimeType: mimeType })
                        : new MediaRecorder(stream);

                    voiceRecorder.ondataavailable = function (event) {
                        if (event.data && event.data.size > 0) {
                            voiceChunks.push(event.data);
                        }
                    };

                    voiceRecorder.start(250);
                    voiceRecording = true;
                    voiceStartedAt = Date.now();
                    voiceBtn.classList.add("is-recording");
                    voiceOverlay.hidden = false;
                    updateVoiceTimer();
                    voiceTimerId = window.setInterval(
                        updateVoiceTimer,
                        500
                    );
                    voiceStopTimerId = window.setTimeout(function () {
                        stopVoiceRecording(true);
                    }, 120000);
                    sendState.textContent = "Запись… отпустите для отправки";
                })
                .catch(function () {
                    sendState.textContent =
                        "Нет доступа к микрофону";
                    if (typeof window.playChatError === "function") {
                        window.playChatError();
                    }
                });
        }

        if (voiceBtn) {
            voiceBtn.addEventListener("pointerdown", function (event) {
                event.preventDefault();
                startVoiceRecording();
            });

            ["pointerup", "pointerleave", "pointercancel"].forEach(
                function (eventName) {
                    voiceBtn.addEventListener(eventName, function () {
                        if (voiceRecording) {
                            stopVoiceRecording(true);
                        }
                    });
                }
            );
        }

        var soundToggle = document.getElementById("chat-sound-toggle");
        if (soundToggle) {
            function syncSoundToggle() {
                var enabled = true;
                try {
                    enabled =
                        localStorage.getItem("resursmap-chat-sounds") !==
                        "off";
                } catch (_) {}
                soundToggle.setAttribute(
                    "aria-pressed",
                    enabled ? "true" : "false"
                );
                soundToggle.textContent = enabled ? "🔊" : "🔇";
                soundToggle.classList.toggle("is-muted", !enabled);
            }

            syncSoundToggle();
            soundToggle.addEventListener("click", function () {
                var enabled =
                    soundToggle.getAttribute("aria-pressed") !== "true";
                try {
                    localStorage.setItem(
                        "resursmap-chat-sounds",
                        enabled ? "on" : "off"
                    );
                } catch (_) {}
                syncSoundToggle();
            });
        }

        var hapticToggle = document.getElementById("chat-haptic-toggle");
        if (hapticToggle) {
            function syncHapticToggle() {
                var enabled = true;
                try {
                    enabled =
                        localStorage.getItem("resursmap-chat-haptics") !==
                        "off";
                } catch (_) {}
                hapticToggle.setAttribute(
                    "aria-pressed",
                    enabled ? "true" : "false"
                );
                hapticToggle.textContent = enabled ? "📳" : "🔕";
                hapticToggle.classList.toggle("is-muted", !enabled);
            }

            syncHapticToggle();
            hapticToggle.addEventListener("click", function () {
                var enabled =
                    hapticToggle.getAttribute("aria-pressed") !== "true";
                try {
                    localStorage.setItem(
                        "resursmap-chat-haptics",
                        enabled ? "on" : "off"
                    );
                } catch (_) {}
                syncHapticToggle();
            });
        }

        form.addEventListener(
            "submit",
            function (event) {
                event.preventDefault();
                sendMessage();
            }
        );

        input.addEventListener("input", function () {
            updateComposer();

            if (input.value.trim()) {
                scheduleTypingSignals();
            } else if (typingStopTimer) {
                window.clearTimeout(typingStopTimer);
                typingStopTimer = null;
                emitTypingSignal("typing.stop");
            }
        });

        input.addEventListener(
            "keydown",
            function (event) {
                if (
                    event.key === "Enter" &&
                    !event.shiftKey &&
                    !event.isComposing
                ) {
                    event.preventDefault();
                    sendMessage();
                }
            }
        );

        clear.addEventListener("click", function () {
            input.value = "";
            updateComposer();
            input.focus();
        });

        loadOlder.addEventListener(
            "click",
            loadOlderMessages
        );

        if (scrollBottomButton) {
            scrollBottomButton.addEventListener(
                "click",
                function () {
                    scrollToBottom("smooth");
                }
            );
        }

        history.addEventListener(
            "scroll",
            function () {
                updateScrollBottomButton();
                markReadAtBottomDebounced();
            },
            { passive: true }
        );

        document.addEventListener(
            "visibilitychange",
            function () {
                if (
                    document.visibilityState === "visible"
                ) {
                    pollMessages();
                }
            }
        );

        window.addEventListener(
            "online",
            function () {
                pollMessages();
                flushPendingQueue();
            }
        );

        document.addEventListener(
            "resursmap:chat-typing",
            function (event) {
                var detail = event.detail || {};
                var actorId = String(
                    detail.actor_user_id || ""
                ).trim();

                if (actorId !== otherUserId) {
                    return;
                }

                if (detail.kind === "typing.start") {
                    setPeerTyping(true);
                } else if (detail.kind === "typing.stop") {
                    setPeerTyping(false);
                }
            }
        );

        document.addEventListener(
            "resursmap:chat-read-update",
            function (event) {
                var detail = event.detail || {};
                updateReadStatuses(Number(detail.message_id || 0));
            }
        );

        document.addEventListener(
            "resursmap:chat-realtime-sync",
            function () {
                pollMessages(true);
            }
        );

        document.addEventListener(
            "resursmap:chat-sync-messages",
            function (event) {
                var detail = event.detail || {};

                appendMessages(detail.messages || []);
                updateReadStatuses(
                    Number(detail.peer_read_through_id || 0)
                );
            }
        );

        if (window.visualViewport) {
            window.visualViewport.addEventListener(
                "resize",
                updateViewportHeight,
                { passive: true }
            );
        }

        window.addEventListener(
            "resize",
            updateViewportHeight
        );

        try {
            input.value =
                localStorage.getItem(draftKey) || "";

            var storedQueue =
                localStorage.getItem(pendingSendKey);
            var legacyPending =
                localStorage.getItem(
                    legacyPendingSendKey
                );

            if (storedQueue) {
                var parsedQueue =
                    JSON.parse(storedQueue);

                if (Array.isArray(parsedQueue)) {
                    pendingQueue =
                        parsedQueue.filter(
                            function (item) {
                                return Boolean(
                                    item &&
                                    typeof item.message ===
                                        "string" &&
                                    typeof item
                                        .clientMessageId ===
                                        "string"
                                );
                            }
                        );
                }
            } else if (legacyPending) {
                var parsedLegacy =
                    JSON.parse(legacyPending);

                if (
                    parsedLegacy &&
                    typeof parsedLegacy.message ===
                        "string" &&
                    typeof parsedLegacy
                        .clientMessageId === "string"
                ) {
                    pendingQueue = [{
                        clientMessageId:
                            parsedLegacy
                                .clientMessageId,
                        message:
                            parsedLegacy.message,
                        replyToMessageId:
                            parsedLegacy
                                .replyToMessageId ||
                            null,
                        replyMessage: "",
                        replySenderUserId: null,
                        createdAt:
                            Math.floor(
                                Date.now() / 1000
                            ),
                        state: "queued"
                    }];
                }
            }

            pendingQueue.forEach(function (item) {
                if (item.state === "sending" || item.state === "retrying") {
                    item.state = "queued";
                }
            });

            savePendingQueue();
        } catch (_) {
            pendingQueue = [];
        }

        loadOlder.hidden = !mayHaveOlder;
        updateViewportHeight();
        updateComposer();
        scrollToBottom("auto");
        setConnection("В сети", "is-online");

        pollTimer = window.setInterval(
            pollMessages,
            5000
        );

        var safetyPollTimer = window.setInterval(
            function () {
                pollMessages(true);
            },
            15000
        );

        presenceTimer = window.setInterval(
            refreshPeerPresence,
            20000
        );

        window.addEventListener(
            "pagehide",
            function () {
                window.clearInterval(pollTimer);
                window.clearInterval(safetyPollTimer);
                window.clearInterval(presenceTimer);

                if (typingStopTimer) {
                    window.clearTimeout(typingStopTimer);
                    emitTypingSignal("typing.stop");
                }
            },
            { once: true }
        );

        window.setTimeout(refreshPeerPresence, 400);

        window.setTimeout(pollMessages, 500);
        window.setTimeout(markReadAtBottomDebounced, 700);

        renderPendingQueue();

        if (
            pendingQueue.length > 0 &&
            navigator.onLine !== false
        ) {
            window.setTimeout(
                flushPendingQueue,
                700
            );
        }

        window.addEventListener("online", function () {
            if (
                pendingQueue.length > 0 &&
                navigator.onLine !== false
            ) {
                flushPendingQueue();
            }
        });
    });
})();

(function () {
    "use strict";

    function ready(callback) {
        if (document.readyState === "loading") {
            document.addEventListener(
                "DOMContentLoaded",
                callback,
                { once: true }
            );
        } else {
            callback();
        }
    }

    ready(function () {
        var history =
            document.getElementById("chat-messages");
        var form =
            document.getElementById("chat-form");
        var input =
            document.getElementById("chat-input");

        if (!history || !form || !input) {
            return;
        }

        var otherUserId = String(
            history.dataset.otherUserId || ""
        ).trim();

        if (
            !/^[1-9][0-9]{0,18}$/.test(otherUserId)
        ) {
            return;
        }

        var messageCache = new Map();
        var selectedMessage = null;
        var refreshDebounceTimer = null;
        var refreshFallbackTimer = null;
        var lastTapAt = 0;
        var lastTapMessageId = 0;

        var replyBar = document.getElementById("chat-reply-bar");
        var replyText =
            document.getElementById("chat-reply-text");

        if (!replyBar) {
            replyBar = document.createElement("div");
            replyBar.id = "chat-reply-bar";
            replyBar.className = "chat-reply-bar";
            replyBar.hidden = true;
            replyBar.innerHTML =
                '<div class="chat-reply-accent"></div>' +
                '<div class="chat-reply-copy">' +
                    '<strong>Ответ</strong>' +
                    '<span id="chat-reply-text"></span>' +
                '</div>' +
                '<button id="chat-reply-close" ' +
                    'type="button" aria-label="Отменить ответ">' +
                    '×' +
                '</button>';

            form.insertBefore(replyBar, form.firstChild);
            replyText =
                document.getElementById("chat-reply-text");
        }

        var sheet = document.createElement("div");
        sheet.id = "chat-action-sheet";
        sheet.className = "chat-action-sheet";
        sheet.hidden = true;
        sheet.innerHTML =
            '<button class="chat-sheet-backdrop" ' +
                'type="button" data-close-sheet></button>' +
            '<section class="chat-sheet-panel" ' +
                'role="dialog" aria-modal="true">' +
                '<div class="chat-sheet-handle"></div>' +
                '<div class="chat-sheet-preview" ' +
                    'id="chat-sheet-preview"></div>' +
                '<div class="chat-sheet-actions">' +
                    '<div class="chat-sheet-reactions" id="chat-sheet-reactions">' +
                        '<button type="button" data-chat-action="react" data-emoji="❤️" aria-label="❤️">❤️</button>' +
                        '<button type="button" data-chat-action="react" data-emoji="👍" aria-label="👍">👍</button>' +
                        '<button type="button" data-chat-action="react" data-emoji="😂" aria-label="😂">😂</button>' +
                        '<button type="button" data-chat-action="react" data-emoji="😮" aria-label="😮">😮</button>' +
                        '<button type="button" data-chat-action="react" data-emoji="😢" aria-label="😢">😢</button>' +
                        '<button type="button" data-chat-action="react" data-emoji="🙏" aria-label="🙏">🙏</button>' +
                    '</div>' +
                    '<button type="button" ' +
                        'data-chat-action="reply">' +
                        '<span>↩</span>Ответить' +
                    '</button>' +
                    '<button type="button" ' +
                        'data-chat-action="copy">' +
                        '<span>⧉</span>Копировать' +
                    '</button>' +
                    '<button type="button" ' +
                        'data-chat-action="forward">' +
                        '<span>↪</span>Переслать' +
                    '</button>' +
                    '<button type="button" ' +
                        'data-chat-action="edit">' +
                        '<span>✎</span>Изменить' +
                    '</button>' +
                    '<button type="button" ' +
                        'class="is-danger" ' +
                        'data-chat-action="delete">' +
                        '<span>⌫</span>Удалить' +
                    '</button>' +
                '</div>' +
            '</section>';

        document.body.appendChild(sheet);

        var editor = document.createElement("div");
        editor.id = "chat-editor";
        editor.className = "chat-editor";
        editor.hidden = true;
        editor.innerHTML =
            '<button class="chat-sheet-backdrop" ' +
                'type="button" data-close-editor></button>' +
            '<section class="chat-editor-panel" ' +
                'role="dialog" aria-modal="true">' +
                '<div class="chat-sheet-handle"></div>' +
                '<div class="chat-editor-title">' +
                    'Редактировать сообщение' +
                '</div>' +
                '<textarea id="chat-editor-input" ' +
                    'maxlength="2000" rows="4"></textarea>' +
                '<div class="chat-editor-footer">' +
                    '<button type="button" ' +
                        'data-close-editor>Отмена</button>' +
                    '<button type="button" ' +
                        'class="is-primary" ' +
                        'id="chat-editor-save">Сохранить</button>' +
                '</div>' +
            '</section>';

        document.body.appendChild(editor);

        var confirmBox = document.createElement("div");
        confirmBox.id = "chat-delete-confirm";
        confirmBox.className = "chat-editor";
        confirmBox.hidden = true;
        confirmBox.innerHTML =
            '<button class="chat-sheet-backdrop" ' +
                'type="button" data-close-delete></button>' +
            '<section class="chat-editor-panel chat-delete-panel" ' +
                'role="dialog" aria-modal="true">' +
                '<div class="chat-sheet-handle"></div>' +
                '<div class="chat-delete-icon">⌫</div>' +
                '<div class="chat-editor-title">' +
                    'Удалить сообщение?' +
                '</div>' +
                '<p>У собеседников вместо текста появится ' +
                    'отметка «Сообщение удалено».</p>' +
                '<div class="chat-editor-footer">' +
                    '<button type="button" ' +
                        'data-close-delete>Отмена</button>' +
                    '<button type="button" ' +
                        'class="is-danger" ' +
                        'id="chat-delete-apply">Удалить</button>' +
                '</div>' +
            '</section>';

        document.body.appendChild(confirmBox);

        var forwardPicker = document.createElement("div");
        forwardPicker.id = "chat-forward-picker";
        forwardPicker.className = "chat-editor";
        forwardPicker.hidden = true;
        forwardPicker.innerHTML =
            '<button class="chat-sheet-backdrop" ' +
                'type="button" data-close-forward></button>' +
            '<section class="chat-editor-panel chat-forward-panel" ' +
                'role="dialog" aria-modal="true">' +
                '<div class="chat-sheet-handle"></div>' +
                '<div class="chat-editor-title">Переслать сообщение</div>' +
                '<div class="chat-forward-preview" ' +
                    'id="chat-forward-picker-preview"></div>' +
                '<div class="chat-forward-list" ' +
                    'id="chat-forward-list"></div>' +
            '</section>';

        document.body.appendChild(forwardPicker);

        if (!replyText) {
            replyText =
                document.getElementById("chat-reply-text");
        }

        var editorInput =
            document.getElementById("chat-editor-input");
        var editorSave =
            document.getElementById("chat-editor-save");
        var deleteApply =
            document.getElementById("chat-delete-apply");
        var sheetPreview =
            document.getElementById("chat-sheet-preview");
        var forwardBar =
            document.getElementById("chat-forward-bar");
        var forwardText =
            document.getElementById("chat-forward-text");
        var forwardPickerPreview =
            document.getElementById("chat-forward-picker-preview");
        var forwardList =
            document.getElementById("chat-forward-list");
        var forwardDraftKey =
            "resursmap-chat-forward:" + otherUserId;

        function forwardPreviewLabel(message) {
            if (Number(message.deleted_at) > 0) {
                return "Сообщение удалено";
            }

            if (
                message.attachment_kind === "voice" &&
                message.attachment_url
            ) {
                return "🎤 Голосовое сообщение";
            }

            if (
                message.attachment_kind === "image" &&
                message.attachment_url
            ) {
                return message.message
                    ? "📷 " + shortText(message.message, 90)
                    : "📷 Фото";
            }

            return shortText(messageText(message), 110);
        }

        function buildForwardPayload(message) {
            return {
                text: messageText(message),
                attachment_kind:
                    String(message.attachment_kind || ""),
                attachment_url:
                    String(message.attachment_url || ""),
                preview: forwardPreviewLabel(message)
            };
        }

        function showForwardBar(payload) {
            window.ResursMapChatForward = payload;

            if (forwardBar && forwardText) {
                forwardText.textContent =
                    payload.preview || payload.text || "Сообщение";
                forwardBar.hidden = false;
            }

            if (replyBar) {
                clearReply();
            }

            input.removeAttribute("required");
            input.focus();
        }

        function clearForward() {
            window.ResursMapChatForward = null;

            if (forwardBar) {
                forwardBar.hidden = true;
            }

            if (forwardText) {
                forwardText.textContent = "";
            }

            input.setAttribute("required", "required");
        }

        function buildForwardCaption(payload, comment) {
            var core =
                String(payload.text || "").trim() ||
                "📷 Фото";
            var block = "[Переслано]\n" + core;

            if (comment) {
                return comment + "\n\n" + block;
            }

            return block;
        }

        function closeForwardPicker() {
            forwardPicker.hidden = true;
            document.body.classList.remove("chat-overlay-open");
        }

        function openForwardPicker(message) {
            if (Number(message.deleted_at) > 0) {
                return;
            }

            closeSheet();
            forwardPickerPreview.textContent =
                forwardPreviewLabel(message);
            forwardList.innerHTML =
                '<div class="chat-forward-empty">Загрузка диалогов…</div>';
            forwardPicker.hidden = false;
            document.body.classList.add("chat-overlay-open");

            requestJson("/api/chat/conversations")
                .then(function (data) {
                    var conversations =
                        Array.isArray(data.conversations)
                            ? data.conversations
                            : [];

                    conversations = conversations.filter(
                        function (conversation) {
                            return String(
                                conversation.other_user_id
                            ) !== otherUserId;
                        }
                    );

                    if (!conversations.length) {
                        forwardList.innerHTML =
                            '<div class="chat-forward-empty">Нет других диалогов для пересылки.</div>';
                        return;
                    }

                    forwardList.innerHTML = conversations
                        .map(function (conversation) {
                            var userId = String(
                                conversation.other_user_id
                            );
                            var label = String(
                                conversation.display_name || userId
                            );
                            var meta = String(
                                conversation.last_message || ""
                            );

                            return (
                                '<button type="button" ' +
                                'class="chat-forward-target" ' +
                                'data-forward-target="' +
                                userId +
                                '"><span class="chat-forward-target-name">' +
                                label +
                                '</span><span class="chat-forward-target-meta">' +
                                meta +
                                "</span></button>"
                            );
                        })
                        .join("");

                    forwardList
                        .querySelectorAll("[data-forward-target]")
                        .forEach(function (button) {
                            button.addEventListener(
                                "click",
                                function () {
                                    var targetId =
                                        button.dataset.forwardTarget;
                                    var payload =
                                        buildForwardPayload(message);

                                    closeForwardPicker();

                                    if (targetId === otherUserId) {
                                        showForwardBar(payload);
                                        return;
                                    }

                                    try {
                                        sessionStorage.setItem(
                                            "resursmap-chat-forward:" +
                                                targetId,
                                            JSON.stringify(payload)
                                        );
                                    } catch (_) {
                                        return;
                                    }

                                    window.location.href =
                                        "/app/chat/" +
                                        encodeURIComponent(targetId) +
                                        "#chat-end";
                                }
                            );
                        });
                })
                .catch(function () {
                    forwardList.innerHTML =
                        '<div class="chat-forward-empty">Не удалось загрузить диалоги.</div>';
                });
        }

        function createForwardClientMessageId() {
            if (
                window.crypto &&
                typeof window.crypto.randomUUID === "function"
            ) {
                return window.crypto.randomUUID();
            }

            return (
                "fwd_" +
                Date.now().toString(36) +
                "_" +
                Math.random().toString(36).slice(2, 10)
            );
        }

        function setSendState(text) {
            var node = document.getElementById("chat-send-state");

            if (node) {
                node.textContent = text;
            }
        }

        window.resursmapSendChatForward = function () {
            var payload = window.ResursMapChatForward;

            if (!payload) {
                return;
            }

            var comment = input.value.trim();
            var caption = buildForwardCaption(payload, comment);

            clearForward();
            input.value = "";
            input.dispatchEvent(new Event("input", { bubbles: true }));

            if (
                payload.attachment_kind === "voice" &&
                payload.attachment_url
            ) {
                setSendState("Пересылка голосового…");

                fetch(payload.attachment_url, {
                    credentials: "same-origin"
                })
                    .then(function (response) {
                        if (!response.ok) {
                            throw new Error("forward_fetch_failed");
                        }

                        return response.blob();
                    })
                    .then(function (blob) {
                        var formData = new FormData();
                        formData.append("voice", blob, "forward.webm");
                        formData.append(
                            "client_message_id",
                            createForwardClientMessageId()
                        );

                        return fetch(
                            "/api/chat/" + otherUserId + "/send-voice",
                            {
                                method: "POST",
                                body: formData,
                                credentials: "same-origin"
                            }
                        );
                    })
                    .then(function (response) {
                        return response.json().then(function (data) {
                            return { response: response, data: data };
                        });
                    })
                    .then(function (pack) {
                        if (!pack.response.ok || !pack.data.ok) {
                            throw new Error("forward_send_failed");
                        }

                        if (pack.data.message) {
                            document.dispatchEvent(
                                new CustomEvent(
                                    "resursmap:chat-sync-messages",
                                    {
                                        detail: {
                                            messages: [pack.data.message],
                                            peer_read_through_id: 0
                                        }
                                    }
                                )
                            );
                        }

                        if (comment) {
                            return requestJson(
                                "/api/chat/" + otherUserId + "/send",
                                {
                                    method: "POST",
                                    headers: {
                                        "Content-Type": "application/json"
                                    },
                                    body: JSON.stringify({
                                        message: comment,
                                        client_message_id:
                                            createForwardClientMessageId()
                                    })
                                }
                            ).then(function (data) {
                                if (data.message) {
                                    document.dispatchEvent(
                                        new CustomEvent(
                                            "resursmap:chat-sync-messages",
                                            {
                                                detail: {
                                                    messages: [data.message],
                                                    peer_read_through_id: 0
                                                }
                                            }
                                        )
                                    );
                                }
                            });
                        }
                    })
                    .then(function () {
                        setSendState("Переслано · Enter — отправить");
                    })
                    .catch(function () {
                        setSendState("Не удалось переслать голосовое");
                    });

                return;
            }

            if (
                payload.attachment_kind === "image" &&
                payload.attachment_url
            ) {
                setSendState("Пересылка фото…");

                fetch(payload.attachment_url, {
                    credentials: "same-origin"
                })
                    .then(function (response) {
                        if (!response.ok) {
                            throw new Error("forward_fetch_failed");
                        }

                        return response.blob();
                    })
                    .then(function (blob) {
                        var formData = new FormData();
                        formData.append("image", blob, "forward.jpg");
                        formData.append(
                            "client_message_id",
                            createForwardClientMessageId()
                        );
                        formData.append("caption", caption);

                        return fetch(
                            "/api/chat/" + otherUserId + "/send-image",
                            {
                                method: "POST",
                                body: formData,
                                credentials: "same-origin"
                            }
                        );
                    })
                    .then(function (response) {
                        return response.json().then(function (data) {
                            return { response: response, data: data };
                        });
                    })
                    .then(function (pack) {
                        if (!pack.response.ok || !pack.data.ok) {
                            throw new Error("forward_send_failed");
                        }

                        if (pack.data.message) {
                            document.dispatchEvent(
                                new CustomEvent(
                                    "resursmap:chat-sync-messages",
                                    {
                                        detail: {
                                            messages: [pack.data.message],
                                            peer_read_through_id: 0
                                        }
                                    }
                                )
                            );
                        }

                        setSendState("Переслано · Enter — отправить");
                    })
                    .catch(function () {
                        setSendState("Не удалось переслать фото");
                    });

                return;
            }

            setSendState("Пересылка…");

            requestJson("/api/chat/" + otherUserId + "/send", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    message: caption,
                    client_message_id: createForwardClientMessageId()
                })
            })
                .then(function (data) {
                    if (data.message) {
                        document.dispatchEvent(
                            new CustomEvent(
                                "resursmap:chat-sync-messages",
                                {
                                    detail: {
                                        messages: [data.message],
                                        peer_read_through_id: 0
                                    }
                                }
                            )
                        );
                    }

                    setSendState("Переслано · Enter — отправить");
                })
                .catch(function () {
                    setSendState("Не удалось переслать сообщение");
                });
        };

        function restoreForwardDraft() {
            var raw = null;

            try {
                raw = sessionStorage.getItem(forwardDraftKey);
            } catch (_) {
                return;
            }

            if (!raw) {
                return;
            }

            try {
                sessionStorage.removeItem(forwardDraftKey);
            } catch (_) {
                // Ignore storage cleanup errors.
            }

            try {
                showForwardBar(JSON.parse(raw));
            } catch (_) {
                // Ignore invalid draft payloads.
            }
        }

        restoreForwardDraft();

        if (document.getElementById("chat-forward-close")) {
            document
                .getElementById("chat-forward-close")
                .addEventListener("click", clearForward);
        }

        forwardPicker.addEventListener("click", function (event) {
            if (event.target.closest("[data-close-forward]")) {
                closeForwardPicker();
            }
        });

        function requestJson(url, options) {
            return fetch(url, {
                credentials: "same-origin",
                cache: "no-store",
                method: options && options.method
                    ? options.method
                    : "GET",
                headers: Object.assign(
                    { "Accept": "application/json" },
                    options && options.headers
                        ? options.headers
                        : {}
                ),
                body: options && options.body
                    ? options.body
                    : undefined
            }).then(function (response) {
                return response.json()
                    .catch(function () {
                        return {
                            ok: false,
                            error: "invalid_response"
                        };
                    })
                    .then(function (data) {
                        if (!response.ok || !data.ok) {
                            var error = new Error(
                                data.error ||
                                "request_failed"
                            );
                            error.status = response.status;
                            throw error;
                        }

                        return data;
                    });
            });
        }

        function messageText(message) {
            if (Number(message.deleted_at) > 0) {
                return "Сообщение удалено";
            }

            return String(message.message || "");
        }

        function messageIsTextOnly(message) {
            var kind = String(message.attachment_kind || "");

            return !kind || kind === "text";
        }

        function messageCanBeEdited(message) {
            if (!message.is_mine || Number(message.deleted_at) > 0) {
                return false;
            }

            if (!messageIsTextOnly(message)) {
                return false;
            }

            var createdAt = Number(message.created_at || 0);

            if (!createdAt) {
                return true;
            }

            return (
                Math.floor(Date.now() / 1000) - createdAt <= 86400
            );
        }

        function shortText(value, limit) {
            var text = String(value || "")
                .replace(/\s+/g, " ")
                .trim();

            if (text.length > limit) {
                return text.slice(0, limit) + "…";
            }

            return text;
        }

        function renderReactions(row, reactions) {
            var bubble = row.querySelector(".chat-bubble");

            if (!bubble) {
                return;
            }

            bubble.querySelectorAll(".chat-message-reactions")
                .forEach(function (element) {
                    element.remove();
                });

            if (!Array.isArray(reactions) || !reactions.length) {
                return;
            }

            var bar = document.createElement("div");
            bar.className = "chat-message-reactions";

            reactions.forEach(function (reaction) {
                var pill = document.createElement("button");
                pill.type = "button";
                pill.className =
                    "chat-reaction-pill" +
                    (reaction.mine ? " is-mine" : "");
                pill.dataset.emoji = reaction.emoji;
                pill.textContent =
                    String(reaction.emoji || "") +
                    " " +
                    String(reaction.count || 0);
                bar.appendChild(pill);
            });

            bubble.appendChild(bar);
        }

        function applyMessageBody(body, message) {
            body.classList.remove("is-deleted");
            body.classList.remove("chat-message-body--voice");
            body.innerHTML = "";

            if (Number(message.deleted_at) > 0) {
                body.classList.add("is-deleted");
                body.textContent = "Сообщение удалено";
                return;
            }

            if (
                message.attachment_kind === "voice" &&
                message.attachment_url
            ) {
                body.classList.add("chat-message-body--voice");
                var voiceWrap = document.createElement("div");
                voiceWrap.className = "chat-voice-player";
                var voiceEl = document.createElement("audio");
                voiceEl.className = "chat-voice-audio";
                voiceEl.controls = true;
                voiceEl.preload = "metadata";
                voiceEl.src = String(message.attachment_url);
                voiceWrap.appendChild(voiceEl);
                body.appendChild(voiceWrap);
                return;
            }

            if (
                message.attachment_kind === "image" &&
                message.attachment_url
            ) {
                var img = document.createElement("img");
                img.className = "chat-message-image";
                img.src = String(message.attachment_url);
                img.alt = "Фото";
                img.loading = "lazy";
                img.setAttribute("role", "button");
                img.tabIndex = 0;
                body.appendChild(img);

                if (message.message) {
                    var caption = document.createElement("div");
                    caption.className = "chat-message-caption";
                    caption.textContent = String(message.message);
                    body.appendChild(caption);
                }

                return;
            }

            body.textContent = String(message.message || "");
        }

        function copyMessageText(message) {
            var text = messageText(message);

            if (!text || text === "Сообщение удалено") {
                return Promise.reject();
            }

            if (
                navigator.clipboard &&
                typeof navigator.clipboard.writeText === "function"
            ) {
                return navigator.clipboard.writeText(text);
            }

            return Promise.reject();
        }

        function reactToMessage(message, emoji) {
            if (!message || Number(message.deleted_at) > 0) {
                return Promise.resolve();
            }

            return requestJson(
                "/api/chat/" +
                otherUserId +
                "/messages/" +
                message.id +
                "/react",
                {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json"
                    },
                    body: JSON.stringify({ emoji: emoji })
                }
            ).then(function (data) {
                if (data.message) {
                    renderMessage(data.message);
                }
            });
        }

        function renderMessage(message) {
            var id = Number(message.id);
            var row = history.querySelector(
                '.chat-message-row[data-message-id="' +
                String(id) +
                '"]'
            );

            if (!row) {
                return;
            }

            var renderSignature = [
                String(message.message || ""),
                Number(message.reply_to_message_id || 0),
                String(message.reply_message || ""),
                Number(message.edited_at || 0),
                Number(message.deleted_at || 0),
                Number(message.read_at || 0),
                Number(message.delivered_at || 0),
                String(message.attachment_kind || ""),
                String(message.attachment_url || ""),
                JSON.stringify(message.reactions || [])
            ].join("|");

            if (
                row.dataset.renderSignature ===
                renderSignature
            ) {
                messageCache.set(id, message);
                return;
            }

            row.dataset.renderSignature =
                renderSignature;

            messageCache.set(id, message);

            row.dataset.mine =
                message.is_mine ? "1" : "0";
            row.dataset.messageText =
                String(message.message || "");
            row.dataset.deleted =
                Number(message.deleted_at) > 0
                    ? "1"
                    : "0";

            var bubble =
                row.querySelector(".chat-bubble");

            if (!bubble) {
                return;
            }

            bubble.classList.toggle(
                "is-deleted",
                Number(message.deleted_at) > 0
            );

            bubble.querySelectorAll(
                ".chat-reply-quote, .chat-edited-label"
            ).forEach(function (element) {
                element.remove();
            });

            var body =
                bubble.querySelector(".chat-message-body");

            if (!body) {
                body = bubble.firstElementChild;

                if (!body ||
                    body.classList.contains(
                        "chat-message-meta"
                    )) {
                    body = document.createElement("div");
                    bubble.insertBefore(
                        body,
                        bubble.firstChild
                    );
                }

                body.classList.add("chat-message-body");
            }

            applyMessageBody(body, message);

            if (
                Number(message.reply_to_message_id) > 0
            ) {
                var quote = document.createElement("button");
                quote.type = "button";
                quote.className = "chat-reply-quote";
                quote.dataset.targetMessageId =
                    String(message.reply_to_message_id);

                var replySender =
                    Number(message.reply_sender_user_id || 0);
                var author = "Сообщение";

                if (replySender > 0) {
                    author =
                        String(replySender) === otherUserId
                            ? "Собеседник"
                            : "Вы";
                }

                var authorNode =
                    document.createElement("strong");
                var textNode =
                    document.createElement("span");

                authorNode.textContent = author;
                textNode.textContent = shortText(
                    message.reply_message ||
                        "Исходное сообщение",
                    120
                );

                quote.appendChild(authorNode);
                quote.appendChild(textNode);
                bubble.insertBefore(quote, body);
            }

            if (
                Number(message.edited_at) > 0 &&
                Number(message.deleted_at) === 0
            ) {
                var edited =
                    document.createElement("span");
                edited.className = "chat-edited-label";
                edited.textContent = "изменено";
                bubble.appendChild(edited);
            }

            renderReactions(row, message.reactions || []);
        }

        function messageFromRow(row) {
            var id = Number(row.dataset.messageId);

            if (!Number.isSafeInteger(id) || id <= 0) {
                return null;
            }

            var cached = messageCache.get(id);

            if (cached) {
                return cached;
            }

            var body = row.querySelector(".chat-message-body");

            return {
                id: id,
                sender_user_id:
                    row.dataset.mine === "1"
                        ? 0
                        : Number(otherUserId),
                message:
                    row.dataset.messageText ||
                    (body ? body.textContent : ""),
                is_mine: row.dataset.mine === "1",
                deleted_at:
                    row.dataset.deleted === "1" ? 1 : 0,
                edited_at:
                    Number(row.dataset.editedAt || 0),
                reply_to_message_id:
                    Number(row.dataset.replyTo || 0) || null,
                reply_message:
                    row.dataset.replyMessage || "",
                reply_sender_user_id:
                    Number(row.dataset.replySender || 0) ||
                    null,
                read_at:
                    Number(row.dataset.readAt || 0),
                delivered_at:
                    Number(row.dataset.deliveredAt || 0),
                created_at:
                    Number(row.dataset.createdAt || 0),
                attachment_kind:
                    row.dataset.attachmentKind || "",
                attachment_url:
                    row.dataset.attachmentUrl || ""
            };
        }

        function refreshRecentDebounced() {
            if (refreshDebounceTimer) {
                window.clearTimeout(refreshDebounceTimer);
            }

            refreshDebounceTimer = window.setTimeout(
                refreshRecent,
                350
            );
        }

        function hydrateSsrMessages() {
            history.querySelectorAll(
                ".chat-message-row[data-message-id]"
            ).forEach(function (row) {
                var id = Number(row.dataset.messageId);

                if (!Number.isSafeInteger(id) || id <= 0) {
                    return;
                }

                var body = row.querySelector(".chat-message-body");
                var message = {
                    id: id,
                    sender_user_id:
                        row.dataset.mine === "1"
                            ? 0
                            : Number(otherUserId),
                    message:
                        row.dataset.messageText ||
                        (body ? body.textContent : ""),
                    is_mine: row.dataset.mine === "1",
                    deleted_at:
                        row.dataset.deleted === "1" ? 1 : 0,
                    edited_at:
                        Number(row.dataset.editedAt || 0),
                    reply_to_message_id:
                        Number(row.dataset.replyTo || 0) || null,
                    reply_message:
                        row.dataset.replyMessage || "",
                    reply_sender_user_id:
                        Number(row.dataset.replySender || 0) ||
                        null,
                    read_at:
                        Number(row.dataset.readAt || 0),
                    delivered_at:
                        Number(row.dataset.deliveredAt || 0),
                    created_at:
                        Number(row.dataset.createdAt || 0)
                };

                messageCache.set(id, message);
                renderMessage(message);
            });
        }

        function refreshRecent() {
            return requestJson(
                "/api/chat/" +
                otherUserId +
                "/messages?limit=100&mark_read=0"
            )
                .then(function (data) {
                    document.dispatchEvent(
                        new CustomEvent(
                            "resursmap:chat-sync-messages",
                            {
                                detail: {
                                    messages: data.messages || [],
                                    peer_read_through_id:
                                        Number(
                                            data.peer_read_through_id || 0
                                        )
                                }
                            }
                        )
                    );

                    (data.messages || []).forEach(
                        renderMessage
                    );
                })
                .catch(function () {
                    // Existing realtime status handles errors.
                });
        }

        function closeSheet() {
            sheet.hidden = true;
            document.body.classList.remove(
                "chat-overlay-open"
            );
        }

        function openSheet(message) {
            selectedMessage = message;
            sheetPreview.textContent =
                shortText(messageText(message), 180);

            var editButton = sheet.querySelector(
                '[data-chat-action="edit"]'
            );
            var deleteButton = sheet.querySelector(
                '[data-chat-action="delete"]'
            );
            var forwardButton = sheet.querySelector(
                '[data-chat-action="forward"]'
            );

            var mine = Boolean(message.is_mine);
            var deleted =
                Number(message.deleted_at) > 0;

            editButton.hidden = !messageCanBeEdited(message);
            deleteButton.hidden = !mine || deleted;

            if (forwardButton) {
                forwardButton.hidden = deleted;
            }

            var reactionsRow =
                document.getElementById("chat-sheet-reactions");

            if (reactionsRow) {
                reactionsRow.hidden = deleted;
            }

            sheet.hidden = false;
            document.body.classList.add(
                "chat-overlay-open"
            );
        }

        function selectReply(message) {
            if (Number(message.deleted_at) > 0) {
                return;
            }

            window.ResursMapChatReply = {
                id: Number(message.id),
                senderUserId:
                    Number(message.sender_user_id),
                message: String(message.message || "")
            };

            replyText.textContent =
                shortText(message.message, 110);
            replyBar.hidden = false;
            closeSheet();
            input.focus();
        }

        function clearReply() {
            window.ResursMapChatReply = null;
            replyBar.hidden = true;
            replyText.textContent = "";
        }

        function openEditor(message) {
            selectedMessage = message;
            closeSheet();
            editorInput.value =
                String(message.message || "");
            editor.hidden = false;
            document.body.classList.add(
                "chat-overlay-open"
            );
            editorInput.focus();
            editorInput.setSelectionRange(
                editorInput.value.length,
                editorInput.value.length
            );
        }

        function closeEditor() {
            editor.hidden = true;
            document.body.classList.remove(
                "chat-overlay-open"
            );
        }

        function openDelete(message) {
            selectedMessage = message;
            closeSheet();
            confirmBox.hidden = false;
            document.body.classList.add(
                "chat-overlay-open"
            );
        }

        function closeDelete() {
            confirmBox.hidden = true;
            document.body.classList.remove(
                "chat-overlay-open"
            );
        }

        history.addEventListener(
            "click",
            function (event) {
                var reaction = event.target.closest(
                    ".chat-reaction-pill"
                );

                if (reaction) {
                    var reactionRow = event.target.closest(
                        ".chat-message-row"
                    );

                    if (!reactionRow) {
                        return;
                    }

                    var reactionMessage =
                        messageFromRow(reactionRow);

                    if (reactionMessage) {
                        reactToMessage(
                            reactionMessage,
                            reaction.dataset.emoji || "❤️"
                        );
                    }

                    return;
                }

                var quote = event.target.closest(
                    ".chat-reply-quote"
                );

                if (quote) {
                    var target = history.querySelector(
                        '.chat-message-row[data-message-id="' +
                        quote.dataset.targetMessageId +
                        '"]'
                    );

                    if (target) {
                        target.scrollIntoView({
                            behavior: "smooth",
                            block: "center"
                        });
                        target.classList.add(
                            "is-highlighted"
                        );
                        window.setTimeout(function () {
                            target.classList.remove(
                                "is-highlighted"
                            );
                        }, 1400);
                    }

                    return;
                }

                var row = event.target.closest(
                    ".chat-message-row"
                );

                if (!row) {
                    return;
                }

                if (
                    event.target.closest(
                        ".chat-message-meta, .chat-message-status, .chat-message-reactions"
                    )
                ) {
                    return;
                }

                var id = Number(row.dataset.messageId);
                var message = messageFromRow(row);

                if (!message) {
                    return;
                }

                var now = Date.now();

                if (
                    lastTapMessageId === id &&
                    now - lastTapAt < 320
                ) {
                    lastTapAt = 0;
                    lastTapMessageId = 0;
                    reactToMessage(message, "❤️");
                    return;
                }

                lastTapAt = now;
                lastTapMessageId = id;
                openSheet(message);
            }
        );

        sheet.addEventListener(
            "click",
            function (event) {
                if (
                    event.target.closest(
                        "[data-close-sheet]"
                    )
                ) {
                    closeSheet();
                    return;
                }

                var action = event.target.closest(
                    "[data-chat-action]"
                );

                if (!action || !selectedMessage) {
                    return;
                }

                if (action.dataset.chatAction === "reply") {
                    selectReply(selectedMessage);
                } else if (action.dataset.chatAction === "copy") {
                    copyMessageText(selectedMessage)
                        .then(function () {
                            closeSheet();
                        })
                        .catch(function () {
                            closeSheet();
                        });
                } else if (action.dataset.chatAction === "forward") {
                    openForwardPicker(selectedMessage);
                } else if (action.dataset.chatAction === "react") {
                    reactToMessage(
                        selectedMessage,
                        action.dataset.emoji || "❤️"
                    ).finally(function () {
                        closeSheet();
                    });
                } else if (
                    action.dataset.chatAction === "edit"
                ) {
                    openEditor(selectedMessage);
                } else if (
                    action.dataset.chatAction === "delete"
                ) {
                    openDelete(selectedMessage);
                }
            }
        );

        document.getElementById(
            "chat-reply-close"
        ).addEventListener("click", clearReply);

        input.addEventListener("keydown", function (event) {
            if (
                event.key === "Escape" &&
                replyBar &&
                !replyBar.hidden
            ) {
                event.preventDefault();
                clearReply();
                return;
            }

            if (
                event.key === "Escape" &&
                forwardBar &&
                !forwardBar.hidden
            ) {
                event.preventDefault();
                clearForward();
            }
        });

        editor.addEventListener(
            "click",
            function (event) {
                if (
                    event.target.closest(
                        "[data-close-editor]"
                    )
                ) {
                    closeEditor();
                }
            }
        );

        confirmBox.addEventListener(
            "click",
            function (event) {
                if (
                    event.target.closest(
                        "[data-close-delete]"
                    )
                ) {
                    closeDelete();
                }
            }
        );

        editorSave.addEventListener(
            "click",
            function () {
                if (!selectedMessage) {
                    return;
                }

                var value = editorInput.value.trim();

                if (!value ||
                    Array.from(value).length > 2000) {
                    editorInput.focus();
                    return;
                }

                editorSave.disabled = true;
                editorSave.textContent = "Сохранение…";

                requestJson(
                    "/api/chat/" +
                    otherUserId +
                    "/messages/" +
                    selectedMessage.id +
                    "/edit",
                    {
                        method: "POST",
                        headers: {
                            "Content-Type":
                                "application/json"
                        },
                        body: JSON.stringify({
                            message: value
                        })
                    }
                )
                    .then(function (data) {
                        selectedMessage.message =
                            data.message;
                        selectedMessage.edited_at =
                            data.edited_at;
                        renderMessage(selectedMessage);
                        closeEditor();
                    })
                    .catch(function () {
                        editorInput.classList.add(
                            "is-error"
                        );
                    })
                    .finally(function () {
                        editorSave.disabled = false;
                        editorSave.textContent =
                            "Сохранить";
                    });
            }
        );

        deleteApply.addEventListener(
            "click",
            function () {
                if (!selectedMessage) {
                    return;
                }

                deleteApply.disabled = true;
                deleteApply.textContent = "Удаление…";

                requestJson(
                    "/api/chat/" +
                    otherUserId +
                    "/messages/" +
                    selectedMessage.id +
                    "/delete",
                    { method: "POST" }
                )
                    .then(function (data) {
                        selectedMessage.message = "";
                        selectedMessage.deleted_at =
                            data.deleted_at;
                        selectedMessage.edited_at = 0;
                        renderMessage(selectedMessage);
                        closeDelete();
                    })
                    .catch(function () {
                        deleteApply.classList.add("is-error");
                    })
                    .finally(function () {
                        deleteApply.disabled = false;
                        deleteApply.textContent =
                            "Удалить";
                    });
            }
        );

        window.addEventListener(
            "resursmap:chat-message-sent",
            function () {
                clearReply();
                window.setTimeout(refreshRecent, 250);
            }
        );

        document.addEventListener(
            "resursmap:chat-messages-render",
            function (event) {
                var messages =
                    event.detail && event.detail.messages
                        ? event.detail.messages
                        : [];

                messages.forEach(renderMessage);
            }
        );

        document.addEventListener(
            "resursmap:chat-realtime-sync",
            function () {
                var lastPoll =
                    Number(window.__resursmapChatLastPollAt || 0);

                if (Date.now() - lastPoll < 900) {
                    return;
                }

                refreshRecentDebounced();
            }
        );

        document.addEventListener(
            "visibilitychange",
            function () {
                if (
                    document.visibilityState === "visible"
                ) {
                    refreshRecentDebounced();
                }
            }
        );

        hydrateSsrMessages();

        refreshFallbackTimer = window.setInterval(
            refreshRecent,
            30000
        );

        window.addEventListener(
            "pagehide",
            function () {
                if (refreshDebounceTimer) {
                    window.clearTimeout(refreshDebounceTimer);
                }

                if (refreshFallbackTimer) {
                    window.clearInterval(refreshFallbackTimer);
                }
            },
            { once: true }
        );
    });
})();

(function () {
    "use strict";

    function installChatFallback() {
        var form =
            document.getElementById("chat-form");
        var state =
            document.getElementById(
                "chat-connection-state"
            );

        if (!form) {
            return;
        }

        function publishComposerHeight() {
            document.documentElement.style.setProperty(
                "--chat-composer-height",
                Math.ceil(
                    form.getBoundingClientRect().height
                ) + "px"
            );
        }

        publishComposerHeight();

        if ("ResizeObserver" in window) {
            var composerObserver =
                new ResizeObserver(
                    publishComposerHeight
                );

            composerObserver.observe(form);

            window.addEventListener(
                "pagehide",
                function () {
                    composerObserver.disconnect();
                },
                { once: true }
            );
        } else {
            window.addEventListener(
                "resize",
                publishComposerHeight,
                { passive: true }
            );
        }

        // Запрещаем браузеру переходить на сырой JSON/текст
        // при submit. Основной обработчик Chat V2 продолжает
        // получать событие и отправляет сообщение через fetch.
        form.addEventListener(
            "submit",
            function (event) {
                event.preventDefault();
            },
            true
        );
    }

    if (document.readyState === "loading") {
        document.addEventListener(
            "DOMContentLoaded",
            installChatFallback,
            { once:true }
        );
    } else {
        installChatFallback();
    }
})();

(function () {
    "use strict";

    function ready(callback) {
        if (document.readyState === "loading") {
            document.addEventListener(
                "DOMContentLoaded",
                callback,
                { once: true }
            );
        } else {
            callback();
        }
    }

    ready(function () {
        var history =
            document.getElementById("chat-messages");

        if (!history || !window.WebSocket) {
            return;
        }

        var socket = null;
        var retryTimer = null;
        var heartbeatTimer = null;
        var stopped = false;
        var retryAttempt = 0;

        function websocketUrl() {
            var scheme =
                window.location.protocol === "https:"
                    ? "wss:"
                    : "ws:";

            return (
                scheme +
                "//" +
                window.location.host +
                "/api/chat/realtime"
            );
        }

        function clearTimers() {
            window.clearTimeout(retryTimer);
            window.clearInterval(heartbeatTimer);
            retryTimer = null;
            heartbeatTimer = null;
        }

        function requestSync() {
            document.dispatchEvent(
                new CustomEvent(
                    "resursmap:chat-realtime-sync"
                )
            );
        }

        function scheduleReconnect() {
            if (stopped || retryTimer) {
                return;
            }

            var delay = Math.min(
                1000 * Math.pow(2, retryAttempt),
                15000
            );

            retryAttempt = Math.min(
                retryAttempt + 1,
                4
            );

            retryTimer = window.setTimeout(
                function () {
                    retryTimer = null;
                    connect();
                },
                delay
            );
        }

        function connect() {
            if (
                stopped ||
                document.visibilityState === "hidden" ||
                (
                    socket &&
                    (
                        socket.readyState ===
                            WebSocket.OPEN ||
                        socket.readyState ===
                            WebSocket.CONNECTING
                    )
                )
            ) {
                return;
            }

            try {
                socket = new WebSocket(websocketUrl());
            } catch (_) {
                scheduleReconnect();
                return;
            }

            socket.addEventListener(
                "open",
                function () {
                    retryAttempt = 0;

                    document.documentElement.dataset
                        .chatRealtime = "online";

                    requestSync();

                    heartbeatTimer =
                        window.setInterval(
                            function () {
                                if (
                                    socket &&
                                    socket.readyState ===
                                        WebSocket.OPEN
                                ) {
                                    socket.send(
                                        JSON.stringify({
                                            type: "ping"
                                        })
                                    );
                                }
                            },
                            20000
                        );
                }
            );

            socket.addEventListener(
                "message",
                function (event) {
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
                        if (
                            payload.type === "chat_event" &&
                            payload.event &&
                            payload.event.kind === "message.read"
                        ) {
                            document.dispatchEvent(
                                new CustomEvent(
                                    "resursmap:chat-read-update",
                                    {
                                        detail: {
                                            message_id: Number(
                                                payload.event.message_id || 0
                                            ),
                                        },
                                    }
                                )
                            );
                            return;
                        }

                        requestSync();
                    }

                    if (
                        payload.type === "typing_event" &&
                        payload.event
                    ) {
                        document.dispatchEvent(
                            new CustomEvent(
                                "resursmap:chat-typing",
                                {
                                    detail: payload.event
                                }
                            )
                        );
                    }
                }
            );

            socket.addEventListener(
                "close",
                function () {
                    clearTimers();

                    delete document.documentElement.dataset
                        .chatRealtime;

                    socket = null;
                    requestSync();
                    scheduleReconnect();
                }
            );

            socket.addEventListener(
                "error",
                function () {
                    if (socket) {
                        socket.close();
                    }
                }
            );
        }

        document.addEventListener(
            "resursmap:chat-realtime-send",
            function (event) {
                var detail = event.detail || {};

                if (
                    !socket ||
                    socket.readyState !== WebSocket.OPEN ||
                    !detail.type
                ) {
                    return;
                }

                try {
                    socket.send(JSON.stringify(detail));
                } catch (_) {
                    // Socket may close mid-send.
                }
            }
        );

        document.addEventListener(
            "visibilitychange",
            function () {
                if (
                    document.visibilityState === "visible"
                ) {
                    connect();
                    requestSync();
                } else if (socket) {
                    socket.close();
                }
            }
        );

        window.addEventListener(
            "online",
            connect
        );

        window.addEventListener(
            "pagehide",
            function () {
                stopped = true;
                clearTimers();

                if (socket) {
                    socket.close();
                }
            },
            { once: true }
        );

        connect();
    });
})();

(function () {
    "use strict";

    function ensureLightbox() {
        var existing = document.getElementById("chat-image-lightbox");

        if (existing) {
            return existing;
        }

        var lightbox = document.createElement("div");
        lightbox.id = "chat-image-lightbox";
        lightbox.className = "chat-image-lightbox";
        lightbox.hidden = true;
        lightbox.innerHTML =
            '<button type="button" class="chat-lightbox-backdrop" aria-label="Закрыть"></button>' +
            '<img class="chat-lightbox-image" alt="">';

        document.body.appendChild(lightbox);
        return lightbox;
    }

    function closeLightbox() {
        var lightbox = document.getElementById("chat-image-lightbox");

        if (!lightbox) {
            return;
        }

        lightbox.hidden = true;
        document.body.classList.remove("chat-lightbox-open");

        var image = lightbox.querySelector(".chat-lightbox-image");

        if (image) {
            image.removeAttribute("src");
        }
    }

    function openLightbox(src) {
        var lightbox = ensureLightbox();
        var image = lightbox.querySelector(".chat-lightbox-image");

        if (!image || !src) {
            return;
        }

        image.src = src;
        lightbox.hidden = false;
        document.body.classList.add("chat-lightbox-open");
    }

    document.addEventListener("click", function (event) {
        var thumb = event.target.closest(".chat-message-image");

        if (!thumb || !thumb.src) {
            return;
        }

        event.preventDefault();
        openLightbox(thumb.src);
    });

    document.addEventListener("keydown", function (event) {
        if (event.key === "Escape") {
            closeLightbox();
        }
    });

    document.addEventListener("click", function (event) {
        var lightbox = document.getElementById("chat-image-lightbox");

        if (!lightbox || lightbox.hidden) {
            return;
        }

        if (
            event.target.closest(".chat-lightbox-backdrop") ||
            event.target.classList.contains("chat-lightbox-image")
        ) {
            closeLightbox();
        }
    });
})();
