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
            "resursmap-chat-pending:" + otherUserId;
        var pendingSend = null;

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

        function savePendingSend(value) {
            pendingSend = value;

            try {
                if (value) {
                    localStorage.setItem(
                        pendingSendKey,
                        JSON.stringify(value)
                    );
                } else {
                    localStorage.removeItem(
                        pendingSendKey
                    );
                }
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
            history.scrollTo({
                top: history.scrollHeight,
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

            if (
                !pendingSend ||
                pendingSend.message !== message
            ) {
                savePendingSend({
                    clientMessageId:
                        createClientMessageId(),
                    message: message,
                    replyToMessageId:
                        window.ResursMapChatReply
                            ? window.ResursMapChatReply.id
                            : null
                });
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
                            message: pendingSend.message,
                            reply_to_message_id:
                                pendingSend.replyToMessageId,
                            client_message_id:
                                pendingSend.clientMessageId
                        })
                    }
                );

                if (
                    window.ResursMapChatReply &&
                    data.message
                ) {
                    data.message.reply_message =
                        window.ResursMapChatReply.message;
                    data.message.reply_sender_user_id =
                        window.ResursMapChatReply.senderUserId;
                }

                appendMessages([data.message]);

                window.dispatchEvent(
                    new CustomEvent(
                        "resursmap:chat-message-sent"
                    )
                );

                savePendingSend(null);
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
            function () {
                pollMessages();

                if (pendingSend && !sending) {
                    sendMessage();
                }
            }
        );

        document.addEventListener(
            "resursmap:chat-realtime-sync",
            function () {
                pollMessages(true);
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

            var storedPending =
                localStorage.getItem(pendingSendKey);

            if (storedPending) {
                var parsedPending =
                    JSON.parse(storedPending);

                if (
                    parsedPending &&
                    typeof parsedPending.message ===
                        "string" &&
                    typeof parsedPending
                        .clientMessageId === "string"
                ) {
                    pendingSend = parsedPending;
                    input.value =
                        parsedPending.message;
                }
            }
        } catch (_) {
            input.value = "";
            pendingSend = null;
        }

        loadOlder.hidden = !mayHaveOlder;
        updateViewportHeight();
        updateComposer();
        scrollToBottom("auto");
        setConnection("В сети", "is-online");

        pollTimer = window.setInterval(
            pollMessages,
            30000
        );

        window.setTimeout(pollMessages, 500);

        if (
            pendingSend &&
            navigator.onLine !== false
        ) {
            window.setTimeout(sendMessage, 700);
        }
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
        var refreshTimer = null;

        var replyBar = document.createElement("div");
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

        form.insertBefore(replyBar, input);

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
                    '<button type="button" ' +
                        'data-chat-action="reply">' +
                        '<span>↩</span>Ответить' +
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

        var replyText =
            document.getElementById("chat-reply-text");
        var editorInput =
            document.getElementById("chat-editor-input");
        var editorSave =
            document.getElementById("chat-editor-save");
        var deleteApply =
            document.getElementById("chat-delete-apply");
        var sheetPreview =
            document.getElementById("chat-sheet-preview");

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

        function shortText(value, limit) {
            var text = String(value || "")
                .replace(/\s+/g, " ")
                .trim();

            if (text.length > limit) {
                return text.slice(0, limit) + "…";
            }

            return text;
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
                Number(message.delivered_at || 0)
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

            body.textContent = messageText(message);

            if (Number(message.deleted_at) > 0) {
                body.classList.add("is-deleted");
            } else {
                body.classList.remove("is-deleted");
            }

            if (
                Number(message.reply_to_message_id) > 0
            ) {
                var quote = document.createElement("button");
                quote.type = "button";
                quote.className = "chat-reply-quote";
                quote.dataset.targetMessageId =
                    String(message.reply_to_message_id);

                var author =
                    Number(message.reply_sender_user_id) > 0 &&
                    Number(message.reply_sender_user_id) ===
                        Number(message.sender_user_id)
                        ? "Сообщение"
                        : "Ответ";

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
        }

        function refreshRecent() {
            return requestJson(
                "/api/chat/" +
                otherUserId +
                "/messages?limit=100"
            )
                .then(function (data) {
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

            var mine = Boolean(message.is_mine);
            var deleted =
                Number(message.deleted_at) > 0;

            editButton.hidden = !mine || deleted;
            deleteButton.hidden = !mine || deleted;

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

                var id = Number(row.dataset.messageId);
                var message = messageCache.get(id);

                if (message) {
                    openSheet(message);
                }
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
            "visibilitychange",
            function () {
                if (
                    document.visibilityState === "visible"
                ) {
                    refreshRecent();
                }
            }
        );

        refreshRecent();

        refreshTimer = window.setInterval(
            refreshRecent,
            30000
        );

        window.addEventListener(
            "pagehide",
            function () {
                window.clearInterval(refreshTimer);
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

        window.addEventListener(
            "error",
            function () {
                if (state) {
                    state.textContent =
                        "Переподключение…";
                    state.classList.add("is-error");
                }
            }
        );

        window.addEventListener(
            "unhandledrejection",
            function () {
                if (state) {
                    state.textContent =
                        "Связь восстанавливается…";
                    state.classList.add("is-error");
                }
            }
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
                        requestSync();
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
