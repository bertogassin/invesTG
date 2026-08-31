(function () {
    "use strict";

    if (window.__resursmapSoundsReady) {
        return;
    }

    window.__resursmapSoundsReady = true;

    var audioContext = null;
    var unlocked = false;

    function soundsEnabled() {
        if (
            window.matchMedia &&
            window.matchMedia("(prefers-reduced-motion: reduce)").matches
        ) {
            return false;
        }

        try {
            return localStorage.getItem("resursmap-chat-sounds") !== "off";
        } catch (_) {
            return true;
        }
    }

    function hapticsEnabled() {
        if (
            window.matchMedia &&
            window.matchMedia("(prefers-reduced-motion: reduce)").matches
        ) {
            return false;
        }

        try {
            return localStorage.getItem("resursmap-chat-haptics") !== "off";
        } catch (_) {
            return true;
        }
    }

    function getContext() {
        if (!soundsEnabled()) {
            return null;
        }

        var AudioContextCtor =
            window.AudioContext || window.webkitAudioContext;

        if (!AudioContextCtor) {
            return null;
        }

        if (!audioContext) {
            audioContext = new AudioContextCtor();
        }

        if (audioContext.state === "suspended") {
            audioContext.resume().catch(function () {});
        }

        return audioContext;
    }

    function unlockOnGesture() {
        if (unlocked) {
            return;
        }

        var ctx = getContext();

        if (ctx && ctx.state === "running") {
            unlocked = true;
        }
    }

    function tone(options) {
        var ctx = getContext();

        if (!ctx) {
            return;
        }

        var start = options.start || ctx.currentTime;
        var duration = options.duration || 0.18;
        var oscillator = ctx.createOscillator();
        var gain = ctx.createGain();
        var filter = ctx.createBiquadFilter();

        filter.type = options.filterType || "lowpass";
        filter.frequency.setValueAtTime(
            options.filter || 1800,
            start
        );

        if (options.filterTo) {
            filter.frequency.exponentialRampToValueAtTime(
                options.filterTo,
                start + duration * 0.85
            );
        }

        oscillator.type = options.type || "sine";
        oscillator.frequency.setValueAtTime(
            options.from || 520,
            start
        );

        if (options.to) {
            oscillator.frequency.exponentialRampToValueAtTime(
                Math.max(options.to, 40),
                start + duration * 0.72
            );
        }

        gain.gain.setValueAtTime(0.0001, start);
        gain.gain.exponentialRampToValueAtTime(
            options.peak || 0.08,
            start + 0.018
        );
        gain.gain.exponentialRampToValueAtTime(
            0.0001,
            start + duration
        );

        oscillator.connect(filter);
        filter.connect(gain);
        gain.connect(ctx.destination);
        oscillator.start(start);
        oscillator.stop(start + duration + 0.03);
    }

    function playSequence(notes) {
        var ctx = getContext();

        if (!ctx) {
            return;
        }

        var cursor = ctx.currentTime + 0.01;

        notes.forEach(function (note) {
            tone({
                start: cursor,
                from: note.from,
                to: note.to,
                peak: note.peak || 0.05,
                duration: note.duration || 0.14,
                type: note.type || "sine",
                filter: note.filter || 2000,
                filterTo: note.filterTo
            });
            cursor += note.gap || 0.11;
        });
    }

    function playAppNotification() {
        playSequence([
            { from: 659.25, to: 783.99, peak: 0.042, duration: 0.12, filter: 2400 },
            { from: 783.99, to: 987.77, peak: 0.034, duration: 0.14, filter: 2600, gap: 0.09 },
            { from: 987.77, peak: 0.028, duration: 0.2, filter: 2800, gap: 0.1 }
        ]);
    }

    function playSplashSound() {
        tone({
            from: 440,
            to: 660,
            peak: 0.028,
            duration: 0.35,
            type: "sine",
            filter: 1400,
            filterTo: 2200
        });
    }

    function playChatReceive() {
        playSequence([
            { from: 587.33, to: 739.99, peak: 0.048, duration: 0.16, type: "triangle", filter: 2100 },
            { from: 739.99, to: 880, peak: 0.032, duration: 0.18, type: "sine", filter: 2300, gap: 0.08 }
        ]);
        chatHaptic("receive");
    }

    function playChatSend() {
        tone({
            from: 493.88,
            to: 659.25,
            peak: 0.036,
            duration: 0.12,
            type: "sine",
            filter: 1500
        });
        chatHaptic("send");
    }

    function playChatError() {
        tone({
            from: 277.18,
            to: 220,
            peak: 0.04,
            duration: 0.2,
            type: "triangle",
            filter: 800
        });
        chatHaptic("error");
    }

    function chatHaptic(kind) {
        if (!hapticsEnabled() || !navigator.vibrate) {
            return;
        }

        if (kind === "send") {
            navigator.vibrate(6);
        } else if (kind === "receive") {
            navigator.vibrate([5, 24, 6]);
        } else if (kind === "error") {
            navigator.vibrate([10, 36, 10]);
        }
    }

    function setChatSoundsEnabled(enabled) {
        try {
            localStorage.setItem(
                "resursmap-chat-sounds",
                enabled ? "on" : "off"
            );
        } catch (_) {}

        document.dispatchEvent(
            new CustomEvent("resursmap:sounds-changed", {
                detail: { enabled: enabled }
            })
        );
    }

    function setChatHapticsEnabled(enabled) {
        try {
            localStorage.setItem(
                "resursmap-chat-haptics",
                enabled ? "on" : "off"
            );
        } catch (_) {}

        document.dispatchEvent(
            new CustomEvent("resursmap:haptics-changed", {
                detail: { enabled: enabled }
            })
        );
    }

    window.playAppNotification = playAppNotification;
    window.resursmapPlayNotificationSound = playAppNotification;
    window.playNotificationSound = playAppNotification;
    window.resursmapPlaySplashSound = playSplashSound;
    window.playChatReceive = playChatReceive;
    window.playChatSend = playChatSend;
    window.playChatError = playChatError;
    window.chatHaptic = chatHaptic;
    window.setChatSoundsEnabled = setChatSoundsEnabled;
    window.setChatHapticsEnabled = setChatHapticsEnabled;
    window.chatSoundsAreEnabled = soundsEnabled;
    window.chatHapticsAreEnabled = hapticsEnabled;

    ["pointerdown", "keydown", "touchstart"].forEach(function (eventName) {
        document.addEventListener(eventName, unlockOnGesture, {
            once: false,
            passive: true
        });
    });
})();
