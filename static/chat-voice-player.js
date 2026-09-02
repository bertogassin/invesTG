(function () {
    "use strict";

    var activeAudio = null;

    function finiteDuration(audio) {
        return Number.isFinite(audio.duration) &&
            audio.duration > 0
            ? audio.duration
            : 0;
    }

    function formatTime(seconds) {
        var safe = Number.isFinite(seconds)
            ? Math.max(0, Math.floor(seconds))
            : 0;
        var minutes = Math.floor(safe / 60);
        var rest = safe % 60;

        return minutes + ":" + String(rest).padStart(2, "0");
    }

    function update(player) {
        var audio = player.querySelector("audio");
        var button = player.querySelector(".chat-voice-play");
        var progress = player.querySelector(
            ".chat-voice-progress"
        );
        var time = player.querySelector(".chat-voice-time");

        if (!audio || !button || !progress || !time) {
            return;
        }

        var duration = finiteDuration(audio);
        var current = Number.isFinite(audio.currentTime)
            ? Math.max(0, audio.currentTime)
            : 0;

        progress.max = duration > 0
            ? String(duration)
            : "1";
        progress.value = duration > 0
            ? String(Math.min(current, duration))
            : "0";
        progress.disabled = duration <= 0;

        button.textContent = audio.paused ? "▶" : "Ⅱ";
        button.setAttribute(
            "aria-label",
            audio.paused
                ? "Воспроизвести голосовое"
                : "Приостановить голосовое"
        );

        time.textContent =
            formatTime(current) +
            " / " +
            formatTime(duration);
    }

    function recoverWebmDuration(audio, player) {
        if (finiteDuration(audio) > 0) {
            update(player);
            return;
        }

        var restoredTime = Number.isFinite(audio.currentTime)
            ? audio.currentTime
            : 0;

        function recovered() {
            audio.removeEventListener(
                "timeupdate",
                recovered
            );

            if (finiteDuration(audio) > 0) {
                try {
                    audio.currentTime = Math.min(
                        restoredTime,
                        audio.duration
                    );
                } catch (_) {}
            }

            update(player);
        }

        audio.addEventListener(
            "timeupdate",
            recovered,
            { once: true }
        );

        try {
            audio.currentTime = 1e10;
        } catch (_) {
            audio.removeEventListener(
                "timeupdate",
                recovered
            );
            update(player);
        }
    }

    function enhance(player) {
        if (
            !player ||
            player.dataset.voiceEnhanced === "1"
        ) {
            return;
        }

        var audio = player.querySelector("audio");

        if (!audio) {
            return;
        }

        player.dataset.voiceEnhanced = "1";
        audio.controls = false;
        audio.preload = "metadata";

        var button = document.createElement("button");
        var progress = document.createElement("input");
        var time = document.createElement("span");

        button.type = "button";
        button.className = "chat-voice-play";
        button.textContent = "▶";
        button.setAttribute(
            "aria-label",
            "Воспроизвести голосовое"
        );

        progress.type = "range";
        progress.className = "chat-voice-progress";
        progress.min = "0";
        progress.max = "1";
        progress.step = "0.01";
        progress.value = "0";
        progress.disabled = true;
        progress.setAttribute(
            "aria-label",
            "Позиция голосового сообщения"
        );

        time.className = "chat-voice-time";
        time.textContent = "0:00 / 0:00";

        player.insertBefore(button, audio);
        player.insertBefore(progress, audio);
        player.insertBefore(time, audio);

        button.addEventListener("click", function () {
            if (activeAudio && activeAudio !== audio) {
                activeAudio.pause();
            }

            if (audio.paused) {
                activeAudio = audio;
                audio.play().catch(function () {
                    update(player);
                });
            } else {
                audio.pause();
            }

            if (navigator.vibrate) {
                navigator.vibrate(8);
            }
        });

        progress.addEventListener("input", function () {
            var duration = finiteDuration(audio);

            if (duration <= 0) {
                return;
            }

            audio.currentTime = Math.min(
                duration,
                Math.max(0, Number(progress.value))
            );
            update(player);
        });

        [
            "loadedmetadata",
            "durationchange",
            "timeupdate",
            "play",
            "pause",
            "ended",
            "seeked",
            "emptied"
        ].forEach(function (eventName) {
            audio.addEventListener(
                eventName,
                function () {
                    update(player);
                }
            );
        });

        audio.addEventListener(
            "loadedmetadata",
            function () {
                recoverWebmDuration(audio, player);
            }
        );

        update(player);
    }

    function enhanceAll(root) {
        var scope = root && root.querySelectorAll
            ? root
            : document;

        if (
            scope.matches &&
            scope.matches(".chat-voice-player")
        ) {
            enhance(scope);
        }

        scope.querySelectorAll(".chat-voice-player")
            .forEach(enhance);
    }

    function start() {
        enhanceAll(document);

        var observer = new MutationObserver(
            function (records) {
                records.forEach(function (record) {
                    record.addedNodes.forEach(
                        function (node) {
                            if (node.nodeType === 1) {
                                enhanceAll(node);
                            }
                        }
                    );
                });
            }
        );

        observer.observe(document.body, {
            childList: true,
            subtree: true
        });
    }

    if (document.readyState === "loading") {
        document.addEventListener(
            "DOMContentLoaded",
            start,
            { once: true }
        );
    } else {
        start();
    }
})();
