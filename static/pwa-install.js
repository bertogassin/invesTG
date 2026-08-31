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

    function isMobileLike() {
        if (window.matchMedia("(max-width: 900px)").matches) {
            return true;
        }

        return (
            "ontouchstart" in window ||
            navigator.maxTouchPoints > 0
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

        return "";
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
        var panel = document.getElementById(
            "resursmap-install-panel"
        );

        var installButton = document.getElementById(
            "resursmap-install-pwa"
        );

        var hint = document.getElementById(
            "resursmap-install-hint"
        );

        if (!panel && !installButton && !hint) {
            return;
        }

        function setHint(text) {
            if (hint) {
                hint.textContent = text;
            }
        }

        function hidePanel() {
            if (panel) {
                panel.hidden = true;
            }
        }

        function markInstalled() {
            setHint(
                "ResursMap уже на главном экране."
            );

            if (installButton) {
                installButton.disabled = true;
                installButton.setAttribute(
                    "aria-disabled",
                    "true"
                );
                installButton.textContent = "✓ Добавлено";
            }
        }

        if (isStandaloneMode()) {
            markInstalled();
            return;
        }

        if (!isMobileLike()) {
            hidePanel();
            return;
        }

        if (isIOSDevice()) {
            setHint(
                "Safari → Поделиться → На экран «Домой»."
            );
        } else if (isTelegramBrowser()) {
            setHint(
                "Откройте страницу в Chrome, затем добавьте ярлык."
            );
        } else {
            setHint(
                "Создаётся ярлык сайта, не загрузка из Play/App Store."
            );
        }

        document.addEventListener(
            "resursmap:pwa-install-ready",
            function () {
                if (!isIOSDevice()) {
                    setHint(
                        "Можно добавить одним нажатием."
                    );
                }
            }
        );

        document.addEventListener(
            "resursmap:pwa-installed",
            markInstalled
        );

        if (installButton) {
            installButton.addEventListener(
                "click",
                async function () {
                    if (isStandaloneMode()) {
                        markInstalled();
                        return;
                    }

                    if (isIOSDevice()) {
                        alert(
                            "На iPhone:\n\n" +
                            "1. Откройте ResursMap в Safari.\n" +
                            "2. Нажмите «Поделиться».\n" +
                            "3. Выберите «На экран Домой».\n" +
                            "4. Нажмите «Добавить»."
                        );
                        return;
                    }

                    if (deferredPrompt) {
                        installButton.disabled = true;

                        try {
                            await deferredPrompt.prompt();
                            await deferredPrompt.userChoice;
                        } finally {
                            deferredPrompt = null;
                            installButton.disabled = false;
                        }

                        return;
                    }

                    if (isTelegramBrowser()) {
                        alert(
                            "Откройте меню Telegram → «Открыть в Chrome», " +
                            "затем снова нажмите «Добавить»."
                        );
                        return;
                    }

                    alert(
                        "Если окно не появилось:\n\n" +
                        "1. Меню браузера ⋮.\n" +
                        "2. «Установить приложение» или " +
                        "«Добавить на главный экран».\n\n" +
                        "Это ярлык сайта, не APK из магазина."
                    );
                }
            );
        }
    });
})();
