(function () {
    "use strict";

    function ready(callback) {
        if (document.readyState === "loading") {
            document.addEventListener("DOMContentLoaded", callback, { once: true });
        } else {
            callback();
        }
    }

    function normalize(value) {
        return (value || "")
            .toString()
            .trim()
            .toLowerCase()
            .replace(/ё/g, "е");
    }

    function escapeHtml(value) {
        return (value || "")
            .toString()
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#39;");
    }

    function iconForKind(kind) {
        if (kind === "continent") {
            return "🌍";
        }
        if (kind === "country") {
            return "🏛";
        }
        if (kind === "city") {
            return "📍";
        }
        if (kind === "work") {
            return "💼";
        }
        if (kind === "workers") {
            return "👷";
        }
        if (kind === "business") {
            return "🏢";
        }
        return "👤";
    }

    ready(function () {
        var root = document.getElementById("rm-home-explorer");
        var input = document.getElementById("rm-home-explorer-input");
        var results = document.getElementById("rm-home-explorer-results");
        var clearBtn = document.getElementById("rm-home-explorer-clear");
        var allLink = document.getElementById("rm-home-explorer-all");
        var dataNode = document.getElementById("rm-home-explore-data");

        if (!root || !input || !results || !dataNode) {
            return;
        }

        var index = [];

        try {
            index = JSON.parse(dataNode.textContent || "[]");
        } catch (_) {
            index = [];
        }

        var defaultHits = index.filter(function (hit) {
            return (
                hit.k === "work" ||
                hit.k === "workers" ||
                hit.k === "business"
            );
        });
        var activeIndex = -1;
        var visibleHits = [];

        function setClearVisible() {
            if (!clearBtn) {
                return;
            }

            clearBtn.hidden = input.value.trim().length === 0;
        }

        function updateAllLink(query) {
            if (!allLink) {
                return;
            }

            var trimmed = query.trim();

            if (!trimmed) {
                allLink.href = "/app/search";
                return;
            }

            allLink.href =
                "/app/search?q=" + encodeURIComponent(trimmed);
        }

        function renderHits(hits) {
            visibleHits = hits;
            activeIndex = hits.length > 0 ? 0 : -1;

            if (hits.length === 0) {
                if (input.value.trim().length >= 2) {
                    results.innerHTML =
                        '<div class="rm-explore-empty">Ничего не найдено. Попробуйте полный поиск.</div>';
                    results.hidden = false;
                } else {
                    results.innerHTML = "";
                    results.hidden = true;
                }
                return;
            }

            results.innerHTML = hits
                .map(function (hit, index) {
                    return (
                        '<a class="rm-explore-hit' +
                        (index === activeIndex ? " is-active" : "") +
                        '" href="' +
                        hit.h +
                        '" data-index="' +
                        index +
                        '">' +
                        '<span class="rm-explore-hit-icon" aria-hidden="true">' +
                        iconForKind(hit.k) +
                        "</span>" +
                        '<span class="rm-explore-hit-body">' +
                        "<strong>" +
                        escapeHtml(hit.l) +
                        "</strong>" +
                        "<small>" +
                        escapeHtml(hit.s) +
                        "</small>" +
                        "</span>" +
                        "</a>"
                    );
                })
                .join("");

            results.hidden = false;
        }

        function scoreHit(hit, query) {
            var label = normalize(hit.l);
            var subtitle = normalize(hit.s);
            var tokens = normalize(hit.q || "");
            var combined = label + " " + subtitle + " " + tokens;

            if (label === query) {
                return 1000;
            }
            if (label.indexOf(query) === 0) {
                return 900 - label.length;
            }
            if (subtitle.indexOf(query) !== -1) {
                return 700;
            }
            if (combined.indexOf(query) !== -1) {
                return 500;
            }

            var words = query.split(/\s+/).filter(Boolean);
            var wordScore = 0;

            words.forEach(function (word) {
                if (label.indexOf(word) === 0) {
                    wordScore += 120;
                } else if (combined.indexOf(word) !== -1) {
                    wordScore += 60;
                }
            });

            return wordScore;
        }

        function search(queryRaw) {
            var query = normalize(queryRaw);

            updateAllLink(queryRaw);
            setClearVisible();

            if (query.length < 1) {
                renderHits(defaultHits);
                return;
            }

            var hits = index
                .map(function (hit) {
                    return {
                        hit: hit,
                        score: scoreHit(hit, query),
                    };
                })
                .filter(function (entry) {
                    return entry.score > 0;
                })
                .sort(function (a, b) {
                    return b.score - a.score;
                })
                .slice(0, 8)
                .map(function (entry) {
                    return entry.hit;
                });

            renderHits(hits);
        }

        function setActive(index) {
            if (!visibleHits.length) {
                return;
            }

            activeIndex = Math.max(
                0,
                Math.min(index, visibleHits.length - 1)
            );

            var links = results.querySelectorAll(".rm-explore-hit");

            for (var i = 0; i < links.length; i++) {
                links[i].classList.toggle("is-active", i === activeIndex);
            }

            var activeLink = links[activeIndex];

            if (activeLink && typeof activeLink.scrollIntoView === "function") {
                activeLink.scrollIntoView({ block: "nearest" });
            }
        }

        input.addEventListener("input", function () {
            search(input.value);
        });

        input.addEventListener("keydown", function (event) {
            if (event.key === "ArrowDown") {
                event.preventDefault();
                setActive(activeIndex + 1);
                return;
            }

            if (event.key === "ArrowUp") {
                event.preventDefault();
                setActive(activeIndex - 1);
                return;
            }

            if (event.key === "Enter") {
                if (activeIndex >= 0 && visibleHits[activeIndex]) {
                    event.preventDefault();
                    window.location.href = visibleHits[activeIndex].h;
                    return;
                }

                if (input.value.trim()) {
                    event.preventDefault();
                    window.location.href =
                        "/app/search?q=" +
                        encodeURIComponent(input.value.trim());
                }
            }

            if (event.key === "Escape") {
                input.value = "";
                search("");
                input.blur();
            }
        });

        if (clearBtn) {
            clearBtn.addEventListener("click", function () {
                input.value = "";
                search("");
                input.focus();
            });
        }

        document.addEventListener("click", function (event) {
            if (!root.contains(event.target)) {
                results.hidden = true;
            }
        });

        input.addEventListener("focus", function () {
            search(input.value);
        });

        setClearVisible();
        updateAllLink("");
        renderHits(defaultHits);
    });
})();
