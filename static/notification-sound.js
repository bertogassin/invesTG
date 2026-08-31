// Legacy alias — global audio lives in chat-sounds.js
(function () {
    "use strict";

    if (typeof window.playNotificationSound === "function") {
        return;
    }

    window.playNotificationSound = function () {};
})();
