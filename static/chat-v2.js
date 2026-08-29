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

        var otherUserId = Number(
            history.dataset.otherUserId || "0"
        );

        if (!Number.isSafeInteger(otherUserId) ||
            otherUserId <= 0) {
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

        function setConnection(text, className) {
            connectionState.textContent = text;
            connectionState.classList.remove(
                "is-online",
                "is-error"
            );

            if (className) {
                connectionState.classList.add(className);
            }
        }

        function updateViewportHeight() {
            var viewport = window.visualViewport;
            var height = viewport
                ? viewport.height
                : window.innerHeight;

            document.documentElement.style.setProperty(
                "--chat-viewport-height",
                Math.round(height) + "px"
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
                sending ||
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

        function scrollToBottom(behavior) {
            historyEnd.scrollIntoView({
                block: "end",
                behavior: behavior || "auto"
            });
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

            row.className = "chat-message-row";
            row.dataset.messageId = String(message.id);
            row.dataset.mine = mine ? "1" : "0";
            row.style.width = "100%";
            row.style.display = "flex";
            row.style.justifyContent = mine
                ? "flex-end"
                : "flex-start";
            row.style.marginBottom = "10px";

            if (animate) {
                row.classList.add("is-new");
            }

            bubble.className = "chat-bubble";
            bubble.style.minWidth = "70px";
            bubble.style.padding = "11px 13px 8px";
            bubble.style.borderRadius = "16px";
            bubble.style.boxSizing = "border-box";
            bubble.style.background = mine
                ? "rgba(214,183,122,.14)"
                : "rgba(0,0,0,.045)";
            bubble.style.border = mine
                ? "1px solid rgba(214,183,122,.30)"
                : "1px solid rgba(0,0,0,.08)";

            body.style.fontSize = "14px";
            body.style.lineHeight = "1.48";
            body.style.whiteSpace = "pre-wrap";
            body.style.overflowWrap = "anywhere";
            body.style.wordBreak = "break-word";
            body.textContent = String(message.message || "");

            meta.style.marginTop = "5px";
            meta.style.display = "flex";
            meta.style.justifyContent = "flex-end";
            meta.style.gap = "6px";
            meta.style.fontSize = "9px";
            meta.style.color = "var(--muted)";

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
                typeof window.playNotificationSound
                    === "function" &&
                document.visibilityState === "visible"
            ) {
                window.playNotificationSound();
            }
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

        async function pollMessages() {
            if (
                polling ||
                document.visibilityState === "hidden"
            ) {
                return;
            }

            polling = true;

            try {
                var data = await fetchJson(
                    "/api/chat/" +
                    otherUserId +
                    "/messages?after_id=" +
                    Math.max(lastMessageId, 0) +
                    "&limit=100"
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

        async function sendMessage() {
            var message = input.value.trim();

            if (!message || sending) {
                return;
            }

            sending = true;
            sendState.textContent = "Отправка…";
            updateComposer();

            try {
                var data = await fetchJson(
                    "/api/chat/" +
                    otherUserId +
                    "/send",
                    {
                        method: "POST",
                        headers: {
                            "Content-Type":
                                "application/json"
                        },
                        body: JSON.stringify({
                            message: message
                        })
                    }
                );

                appendMessages([data.message]);
                input.value = "";
                updateComposer();
                input.focus();
                sendState.textContent =
                    "Отправлено · Enter — отправить";
                setConnection("В сети", "is-online");
            } catch (error) {
                if (
                    error.status === 429 &&
                    error.retryAfter > 0
                ) {
                    sendState.textContent =
                        "Лимит сообщений · повторите через " +
                        error.retryAfter +
                        " сек.";
                } else if (error.status === 401) {
                    sendState.textContent =
                        "Сессия истекла · войдите снова";
                } else {
                    sendState.textContent =
                        "Не отправлено · проверьте соединение";
                }

                setConnection(
                    "Ошибка отправки",
                    "is-error"
                );
            } finally {
                sending = false;
                updateComposer();
            }
        }

        form.addEventListener(
            "submit",
            function (event) {
                event.preventDefault();
                sendMessage();
            }
        );

        input.addEventListener("input", updateComposer);

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
            pollMessages
        );

        if (window.visualViewport) {
            window.visualViewport.addEventListener(
                "resize",
                updateViewportHeight
            );
            window.visualViewport.addEventListener(
                "scroll",
                updateViewportHeight
            );
        }

        window.addEventListener(
            "resize",
            updateViewportHeight
        );

        try {
            input.value =
                localStorage.getItem(draftKey) || "";
        } catch (_) {
            input.value = "";
        }

        loadOlder.hidden = !mayHaveOlder;
        updateViewportHeight();
        updateComposer();
        scrollToBottom("auto");
        setConnection("В сети", "is-online");

        pollTimer = window.setInterval(
            pollMessages,
            3000
        );

        window.setTimeout(pollMessages, 500);
    });
})();
