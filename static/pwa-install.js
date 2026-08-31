(function () {
    "use strict";

    var deferredPrompt = null;

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

    function isStandaloneMode() {
        return (
            window.matchMedia(
                "(display-mode: standalone)"
            ).matches ||
            window.navigator.standalone === true
        );
    }

    function isIOSDevice() {
        return /iphone|ipad|ipod/i.test(
            navigator.userAgent
        );
    }

    function isTelegramBrowser() {
        return /telegram/i.test(
            navigator.userAgent
        );
    }

    function assetVersion() {
        var meta = document.querySelector(
            'meta[name="resursmap-asset-version"]'
        );

        if (meta && meta.content) {
            return meta.content;
        }

        var scripts = document.getElementsByTagName("script");

        for (var i = 0; i < scripts.length; i++) {
            var src = scripts[i].src || "";

            if (src.indexOf("pwa-install.js") !== -1) {
                var match = src.match(/[?&]v=([^&]+)/);

                if (match) {
                    return match[1];
                }
            }
        }

        return "4.9.11";
    }

    function registerServiceWorker() {
        if (!("serviceWorker" in navigator)) {
            return;
        }

        window.addEventListener(
            "load",
            function () {
                navigator.serviceWorker.register(
                    "/static/resursmap-sw.js?v=" + assetVersion(),
                    {
                        scope: "/"
                    }
                ).catch(function () {
                    // Installation UI provides fallback guidance.
                });
            },
            { once: true }
        );
    }

    registerServiceWorker();

    window.addEventListener(
        "beforeinstallprompt",
        function (event) {
            event.preventDefault();
            deferredPrompt = event;

            document.dispatchEvent(
                new CustomEvent(
                    "resursmap:pwa-install-ready"
                )
            );
        }
    );

    window.addEventListener(
        "appinstalled",
        function () {
            deferredPrompt = null;

            document.dispatchEvent(
                new CustomEvent(
                    "resursmap:pwa-installed"
                )
            );
        }
    );

    ready(function () {
        var androidButton =
            document.getElementById(
                "resursmap-install-android"
            );

        var iosButton =
            document.getElementById(
                "resursmap-install-ios"
            );

        var hint =
            document.getElementById(
                "resursmap-install-hint"
            );

        if (
            !androidButton &&
            !iosButton &&
            !hint
        ) {
            return;
        }

        function setHint(text) {
            if (hint) {
                hint.textContent = text;
            }
        }

        function markInstalled() {
            setHint(
                "ResursMap уже установлен на этом устройстве."
            );

            if (androidButton) {
                androidButton.disabled = true;
                androidButton.setAttribute(
                    "aria-disabled",
                    "true"
                );
            }

            if (iosButton) {
                iosButton.disabled = true;
                iosButton.setAttribute(
                    "aria-disabled",
                    "true"
                );
            }
        }

        if (isStandaloneMode()) {
            markInstalled();
            return;
        }

        if (isIOSDevice()) {
            setHint(
                "На iPhone установка выполняется через Safari."
            );
        } else if (isTelegramBrowser()) {
            setHint(
                "Для установки откройте эту страницу в Chrome."
            );
        } else {
            setHint(
                "Установите ResursMap на главный экран телефона."
            );
        }

        document.addEventListener(
            "resursmap:pwa-install-ready",
            function () {
                if (!isIOSDevice()) {
                    setHint(
                        "Приложение готово к установке."
                    );
                }
            }
        );

        document.addEventListener(
            "resursmap:pwa-installed",
            markInstalled
        );

        if (androidButton) {
            androidButton.addEventListener(
                "click",
                async function () {
                    if (isStandaloneMode()) {
                        markInstalled();
                        return;
                    }

                    if (deferredPrompt) {
                        androidButton.disabled = true;

                        try {
                            await deferredPrompt.prompt();
                            await deferredPrompt.userChoice;
                        } finally {
                            deferredPrompt = null;
                            androidButton.disabled = false;
                        }

                        return;
                    }

                    if (isTelegramBrowser()) {
                        alert(
                            "Откройте меню Telegram и выберите " +
                            "«Открыть в Chrome». Затем снова " +
                            "нажмите «Установить»."
                        );
                        return;
                    }

                    alert(
                        "Если окно установки не появилось:\n\n" +
                        "1. Откройте меню браузера ⋮.\n" +
                        "2. Выберите «Установить приложение» " +
                        "или «Добавить на главный экран»."
                    );
                }
            );
        }

        if (iosButton) {
            iosButton.addEventListener(
                "click",
                function () {
                    if (isStandaloneMode()) {
                        markInstalled();
                        return;
                    }

                    alert(
                        "На iPhone откройте ResursMap в Safari.\n\n" +
                        "1. Нажмите «Поделиться».\n" +
                        "2. Выберите «На экран Домой».\n" +
                        "3. Нажмите «Добавить»."
                    );
                }
            );
        }
    });
})();
