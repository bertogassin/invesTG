(function () {
    "use strict";

    function ready(callback) {
        if (document.readyState === "loading") {
            document.addEventListener("DOMContentLoaded", callback, { once: true });
        } else {
            callback();
        }
    }

    function telegramWebApp() {
        return window.Telegram && window.Telegram.WebApp
            ? window.Telegram.WebApp
            : null;
    }

    function hasInitData() {
        var tg = telegramWebApp();
        return Boolean(tg && tg.initData);
    }

    function authEndpoint() {
        return "/app/auth";
    }

    function setBusy(isBusy) {
        document.documentElement.dataset.telegramAuthBusy = isBusy ? "1" : "0";
    }

    function notifyStatus(message, isError) {
        document.dispatchEvent(
            new CustomEvent("resursmap:telegram-auth-status", {
                detail: { message: message, error: Boolean(isError) },
            })
        );
    }

    function telegramAuthError(error) {
        var messages = {
            invalid_telegram_data: "Не удалось проверить Telegram. Откройте приложение заново.",
            telegram_auth_disabled: "Вход через Telegram временно недоступен.",
            rate_limited: "Слишком много попыток. Подождите немного.",
            database_unavailable: "Сервис временно недоступен.",
        };

        return messages[error] || "Не удалось выполнить вход через Telegram.";
    }

    function loginWithTelegram(options) {
        options = options || {};

        var tg = telegramWebApp();

        if (!tg || !tg.initData) {
            if (!options.silent) {
                notifyStatus(
                    "Откройте ResursMap через Telegram, чтобы войти одним нажатием.",
                    true
                );
            }
            return Promise.resolve(false);
        }

        tg.ready();

        setBusy(true);

        if (!options.silent) {
            notifyStatus("Входим через Telegram…", false);
        }

        return fetch(authEndpoint(), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ init_data: tg.initData }),
        })
            .then(function (response) {
                return response.json().catch(function () {
                    return { ok: false, error: "invalid_response" };
                });
            })
            .then(function (data) {
                if (!data.ok) {
                    if (!options.silent) {
                        notifyStatus(telegramAuthError(data.error), true);
                    }
                    return false;
                }

                if (!options.silent) {
                    notifyStatus("Вход выполнен", false);
                }

                var redirectTarget =
                    options.redirectTarget ||
                    document.documentElement.dataset.authRedirect ||
                    "/app";

                window.location.replace(redirectTarget);
                return true;
            })
            .catch(function () {
                if (!options.silent) {
                    notifyStatus("Ошибка соединения. Попробуйте ещё раз.", true);
                }
                return false;
            })
            .finally(function () {
                setBusy(false);
            });
    }

    window.resursmapTelegramAuth = {
        login: loginWithTelegram,
        hasInitData: hasInitData,
    };

    ready(function () {
        if (document.documentElement.dataset.telegramAutoAuth !== "1") {
            return;
        }

        if (document.documentElement.dataset.authenticated === "1") {
            return;
        }

        if (!hasInitData()) {
            return;
        }

        loginWithTelegram({ silent: true });
    });
})();
