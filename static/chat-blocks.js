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
        var send =
            document.getElementById("chat-send");
        var toggle =
            document.getElementById("chat-block-toggle");
        var sendState =
            document.getElementById("chat-send-state");

        if (!history || !form || !input || !send || !toggle) {
            return;
        }

        var otherUserId =
            Number(history.dataset.otherUserId || 0);

        if (!Number.isSafeInteger(otherUserId) ||
            otherUserId <= 0) {
            return;
        }

        var state = {
            blocked: false,
            blockedByMe: false,
            busy: false
        };

        function applyState() {
            form.classList.toggle(
                "is-chat-blocked",
                state.blocked
            );

            input.disabled = state.blocked;
            send.disabled = state.blocked;

            toggle.hidden = false;
            toggle.disabled = state.busy;

            if (state.blockedByMe) {
                toggle.textContent = "Разблокировать";
                toggle.classList.add("is-unblock");
            } else {
                toggle.textContent = "Заблокировать";
                toggle.classList.remove("is-unblock");
            }

            if (sendState) {
                sendState.textContent = state.blocked
                    ? "Обмен сообщениями недоступен"
                    : "Enter — отправить · Shift+Enter — новая строка";
            }
        }

        async function request(url, method) {
            var response = await fetch(url, {
                method: method || "GET",
                headers: {
                    "Accept": "application/json"
                },
                credentials: "same-origin"
            });

            var data = await response.json().catch(function () {
                return {
                    ok: false,
                    error: "invalid_response"
                };
            });

            if (!response.ok || !data.ok) {
                throw new Error(
                    data.error || "request_failed"
                );
            }

            return data;
        }

        async function refresh() {
            try {
                var data = await request(
                    "/api/chat/" +
                    otherUserId +
                    "/block"
                );

                state.blocked = Boolean(data.blocked);
                state.blockedByMe =
                    Boolean(data.blocked_by_me);

                applyState();
            } catch (_) {
                // Серверная и SQLite-защита продолжают работать.
            }
        }

        toggle.addEventListener("click", async function () {
            if (state.busy) {
                return;
            }

            if (!state.blockedByMe) {
                var confirmed = window.confirm(
                    "Заблокировать пользователя?\n\n" +
                    "Он больше не сможет отправлять вам сообщения. " +
                    "Старый диалог сохранится."
                );

                if (!confirmed) {
                    return;
                }
            }

            state.busy = true;
            applyState();

            try {
                var endpoint = state.blockedByMe
                    ? "/api/chat/" +
                        otherUserId +
                        "/unblock"
                    : "/api/chat/" +
                        otherUserId +
                        "/block";

                var data = await request(
                    endpoint,
                    "POST"
                );

                state.blocked = Boolean(data.blocked);
                state.blockedByMe =
                    Boolean(data.blocked_by_me);
            } catch (_) {
                window.alert(
                    "Не удалось изменить блокировку. " +
                    "Попробуйте ещё раз."
                );
            }

            state.busy = false;
            applyState();
        });

        refresh();
    });
})();
