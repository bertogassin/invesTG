(function () {
    "use strict";

    var CITY_KEY = "resursmap-last-city";
    var SEARCH_KEY = "resursmap-recent-searches";

    function ready(callback) {
        if (document.readyState === "loading") {
            document.addEventListener("DOMContentLoaded", callback, { once: true });
        } else {
            callback();
        }
    }

    function cityFromPath(path) {
        var match = String(path || "").match(/^\/app\/(\d+)\/(\d+)\/(\d+)(?:\/|$)/);
        if (!match) {
            return null;
        }

        return {
            ci: Number(match[1]),
            si: Number(match[2]),
            zi: Number(match[3]),
            href: "/app/" + match[1] + "/" + match[2] + "/" + match[3],
        };
    }

    function saveCity(city) {
        if (!city) {
            return;
        }

        try {
            localStorage.setItem(CITY_KEY, JSON.stringify(city));
        } catch (e) {}

        document.cookie =
            "rm_last_city=" +
            city.ci +
            "." +
            city.si +
            "." +
            city.zi +
            "; Path=/; Max-Age=31536000; SameSite=Lax";
    }

    function loadCity() {
        try {
            return JSON.parse(localStorage.getItem(CITY_KEY) || "null");
        } catch (e) {
            return null;
        }
    }

    function loadSearches() {
        try {
            var rows = JSON.parse(localStorage.getItem(SEARCH_KEY) || "[]");
            return Array.isArray(rows) ? rows.slice(0, 6) : [];
        } catch (e) {
            return [];
        }
    }

    function saveSearch(entry) {
        if (!entry || (!entry.q && !entry.kind && !entry.rubric)) {
            return;
        }

        var rows = loadSearches().filter(function (row) {
            return row.href !== entry.href;
        });
        rows.unshift(entry);

        try {
            localStorage.setItem(SEARCH_KEY, JSON.stringify(rows.slice(0, 6)));
        } catch (e) {}
    }

    function renderRecent() {
        var host = document.getElementById("rm-recent-searches");
        if (!host) {
            return;
        }

        var rows = loadSearches();
        var city = loadCity();
        var html = "";

        if (city && city.href) {
            html +=
                '<a class="rm-kind-chip" href="' +
                city.href +
                '">Последний город</a>';
        }

        rows.forEach(function (row) {
            if (!row.href || !row.label) {
                return;
            }

            html +=
                '<a class="rm-kind-chip" href="' +
                row.href +
                '">' +
                row.label +
                "</a>";
        });

        host.innerHTML = html;
        host.hidden = !html;
    }

    ready(function () {
        saveCity(cityFromPath(window.location.pathname));

        var params = new URLSearchParams(window.location.search || "");
        if (window.location.pathname === "/app/search") {
            var q = (params.get("q") || "").trim();
            var kind = (params.get("kind") || "").trim();
            var rubric = (params.get("rubric") || "").trim();
            var href = "/app/search" + (window.location.search || "");
            var label = q || rubric || kind;

            if (label) {
                saveSearch({
                    q: q,
                    kind: kind,
                    rubric: rubric,
                    href: href,
                    label: label,
                });
            }

            renderRecent();
        }

        var addLinks = document.querySelectorAll("[data-add-listing], a[href=\"/app/add\"]");
        var lastCity = loadCity();
        for (var i = 0; i < addLinks.length; i++) {
            if (lastCity && lastCity.href) {
                addLinks[i].setAttribute("href", lastCity.href);
            }
        }
    });
})();
