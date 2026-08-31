(function () {
    "use strict";

    function notifySound() {
        if (typeof window.resursmapPlayNotificationSound === "function") {
            window.resursmapPlayNotificationSound();
        }
    }

    var lastAttention = 0;
    var lastNotifications = 0;

    function formatCount(value) {
        var count = Number(value) || 0;

        if (count <= 0) {
            return "";
        }

        return count > 99 ? "99+" : String(count);
    }

    function attentionLink() {
        return document.querySelector("[data-nav-attention-link]");
    }

    function updateAttentionBadge(count) {
        var link = attentionLink();

        if (!link) {
            return;
        }

        var badge = link.querySelector(".nav-badge");
        var label = formatCount(count);

        if (!label) {
            if (badge) {
                badge.remove();
            }

            link.classList.remove("has-attention");
            return;
        }

        if (!badge) {
            badge = document.createElement("span");
            badge.className = "nav-badge";
            link.insertBefore(badge, link.querySelector("span"));
        }

        badge.textContent = label;
        link.classList.add("has-attention");
    }

    function refreshAttention() {
        return fetch("/api/account/attention-count", {
            credentials: "same-origin",
            headers: { Accept: "application/json" },
        })
            .then(function (response) {
                if (!response.ok) {
                    throw new Error("attention_count_failed");
                }

                return response.json();
            })
            .then(function (data) {
                var total = Number(data.count) || 0;
                var notifications = Number(data.notifications) || 0;

                if (notifications > lastNotifications && lastNotifications > 0) {
                    notifySound();
                }

                lastNotifications = notifications;
                lastAttention = total;
                updateAttentionBadge(total);

                if (typeof window.resursmapOnAttentionCount === "function") {
                    window.resursmapOnAttentionCount(data);
                }
            })
            .catch(function () {});
    }

    window.resursmapRefreshAttentionBadge = refreshAttention;

    document.addEventListener("visibilitychange", function () {
        if (!document.hidden) {
            refreshAttention();
        }
    });

    setInterval(refreshAttention, 30000);
    setTimeout(refreshAttention, 1200);
})();
