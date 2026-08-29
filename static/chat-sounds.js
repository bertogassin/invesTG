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
            start + 0.018
        );
        gain.gain.exponentialRampToValueAtTime(
            0.0001,
            start + duration
        );

        oscillator.connect(gain);
        gain.connect(ctx.destination);
        oscillator.start(start);
        oscillator.stop(start + duration + 0.02);
    }

    function playChatReceive() {
        tone({
            from: 740,
            to: 980,
            peak: 0.07,
            duration: 0.22,
            type: "triangle"
        });
        chatHaptic("receive");
    }

    function playChatSend() {
        tone({
            from: 560,
            to: 720,
            peak: 0.05,
            duration: 0.14,
            type: "sine"
        });
        chatHaptic("send");
    }

    function playChatError() {
        tone({
            from: 280,
            to: 220,
            peak: 0.06,
            duration: 0.2,
            type: "square"
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

    window.playChatReceive = playChatReceive;
    window.playChatSend = playChatSend;
    window.playChatError = playChatError;
    window.chatHaptic = chatHaptic;

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
