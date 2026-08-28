// Проверка новых уведомлений и звук
(function() {
    var lastCount = 0;

    function checkNotifications() {
        fetch('/api/notifications/unread-count')
            .then(function(r) { return r.json(); })
            .then(function(data) {
                if (data.count > lastCount) {
                    playNotificationSound();
                }
                lastCount = data.count;
            })
            .catch(function() {});
    }

    // Проверяем каждые 30 секунд
    setInterval(checkNotifications, 30000);

    // Первая проверка через 5 секунд после загрузки
    setTimeout(checkNotifications, 5000);
})();
