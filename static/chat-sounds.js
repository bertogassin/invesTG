(function () {
    "use strict";

    var audioContext = null;
    var unlocked = false;

    function soundsEnabled() {
        if (window.matchMedia &&
            window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
            return false;
        }

        try {
            return localStorage.getItem("resursmap-chat-sounds") !== "off";
        } catch (_) {
            return true;
        }
    }

    function hapticsEnabled() {
        if (window.matchMedia &&
            window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
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

        filter.type = "lowpass";
        filter.frequency.setValueAtTime(
            options.filter || 1800,
            start
        );

        oscillator.type = options.type || "sine";
        oscillator.frequency.setValueAtTime(
            options.from || 520,
            start
        );

        if (options.to) {
            oscillator.frequency.exponentialRampToValueAtTime(
                options.to,
                start + duration * 0.72
            );
        }

        gain.gain.setValueAtTime(0.0001, start);
        gain.gain.exponentialRampToValueAtTime(
            options.peak || 0.08,
            start + 0.022
        );
        gain.gain.exponentialRampToValueAtTime(
            0.0001,
            start + duration
        );

        oscillator.connect(filter);
        filter.connect(gain);
        gain.connect(ctx.destination);
        oscillator.start(start);
        oscillator.stop(start + duration + 0.02);
    }

    function playChatReceive() {
        tone({
            from: 620,
            to: 880,
            peak: 0.065,
            duration: 0.24,
            type: "triangle",
            filter: 2200
        });
        window.setTimeout(function () {
            tone({
                from: 880,
                to: 980,
                peak: 0.04,
                duration: 0.16,
                type: "sine",
                filter: 2400
            });
        }, 90);
        chatHaptic("receive");
    }

    function playChatSend() {
        tone({
            from: 520,
            to: 760,
            peak: 0.045,
            duration: 0.16,
            type: "sine",
            filter: 1600
        });
        chatHaptic("send");
    }

    function playChatError() {
        tone({
            from: 320,
            to: 240,
            peak: 0.055,
            duration: 0.22,
            type: "triangle",
            filter: 900
        });
        chatHaptic("error");
    }

    function chatHaptic(kind) {
        if (!hapticsEnabled() || !navigator.vibrate) {
            return;
        }

        if (kind === "send") {
            navigator.vibrate(8);
        } else if (kind === "receive") {
            navigator.vibrate([6, 28, 8]);
        } else if (kind === "error") {
            navigator.vibrate([12, 40, 12]);
        }
    }

    function setChatSoundsEnabled(enabled) {
        try {
            localStorage.setItem(
                "resursmap-chat-sounds",
                enabled ? "on" : "off"
            );
        } catch (_) {}
    }

    window.playChatReceive = playChatReceive;
    window.playChatSend = playChatSend;
    window.playChatError = playChatError;
    window.chatHaptic = chatHaptic;
    window.setChatSoundsEnabled = setChatSoundsEnabled;
    window.chatSoundsAreEnabled = soundsEnabled;

    if (!window.playNotificationSound) {
        window.playNotificationSound = playChatReceive;
    }

    ["pointerdown", "keydown", "touchstart"].forEach(function (eventName) {
        document.addEventListener(eventName, unlockOnGesture, {
            once: false,
            passive: true
        });
    });
})();
