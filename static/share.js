(function () {
    "use strict";

    document.addEventListener("click", function (event) {
        var button = event.target.closest("[data-share]");
        if (!button) {
            return;
        }

        event.preventDefault();

        var title = button.getAttribute("data-share-title") || document.title;
        var text = button.getAttribute("data-share-text") || "";
        var url = button.getAttribute("data-share-url") || window.location.href;
        var status = document.getElementById(button.getAttribute("data-share-status") || "");

        function done(message) {
            if (status) {
                status.textContent = message;
            } else {
                button.textContent = message;
            }
        }

        if (navigator.share) {
            navigator
                .share({ title: title, text: text, url: url })
                .catch(function () {});
            return;
        }

        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard
                .writeText(url)
                .then(function () {
                    done("Ссылка скопирована");
                })
                .catch(function () {
                    done("Скопируйте адрес страницы");
                });
            return;
        }

        done("Скопируйте адрес страницы");
    });
})();
