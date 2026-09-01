(function () {
    "use strict";

    function escapeHtml(value) {
        return String(value || "")
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
    }

    async function loadMore(button, grid) {
        if (button.disabled) return;
        button.disabled = true;
        button.textContent = "Загружаем…";

        try {
            var response = await fetch(
                "/api/map/countries/" +
                    encodeURIComponent(button.dataset.countryId) +
                    "/cities?offset=" +
                    encodeURIComponent(button.dataset.offset || "0"),
                { headers: { Accept: "application/json" } }
            );
            var data = await response.json();
            if (!response.ok || !data.ok || !Array.isArray(data.items)) {
                throw new Error("load_failed");
            }

            data.items.forEach(function (item) {
                var link = document.createElement("a");
                link.className = "card";
                link.href = item.href;
                link.innerHTML =
                    '<div class="card-icon"><svg class="icon" viewBox="0 0 24 24"><path d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z"></path><circle cx="12" cy="10" r="2.5"></circle></svg></div>' +
                    '<div class="card-content"><div class="card-title">' +
                    escapeHtml(item.name) +
                    '</div><div class="card-meta">Город</div></div>' +
                    '<div class="card-arrow">›</div>';
                grid.appendChild(link);
            });

            button.dataset.offset = String(data.next_offset || 0);
            if (!data.has_more) {
                button.remove();
                return;
            }
            button.disabled = false;
            button.textContent = "Показать следующие города";
        } catch (_) {
            button.disabled = false;
            button.textContent = "Повторить загрузку городов";
        }
    }

    document.addEventListener("DOMContentLoaded", function () {
        var button = document.getElementById("rm-map-more");
        var grid = document.getElementById("rm-map-city-grid");
        if (!button || !grid) return;

        button.addEventListener("click", function () {
            loadMore(button, grid);
        });

        if ("IntersectionObserver" in window) {
            var observer = new IntersectionObserver(function (entries) {
                if (entries.some(function (entry) { return entry.isIntersecting; })) {
                    loadMore(button, grid);
                }
            }, { rootMargin: "240px" });
            observer.observe(button);
        }
    }, { once: true });
})();
