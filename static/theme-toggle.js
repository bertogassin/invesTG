// Переключатель темы ResursMap — работает в любом месте
(function() {
    // Восстановим сохранённую тему
    try {
        var savedTheme = localStorage.getItem('resursmap-theme');
        if (savedTheme === 'light') {
            document.body.classList.add('light-theme');
        }
    } catch(e) {}

    // Делегированный обработчик клика
    document.addEventListener('click', function(event) {
        var btn = event.target.closest('.theme-toggle-btn');
        if (!btn) return;

        event.preventDefault();

        var isLight = document.body.classList.toggle('light-theme');

        try {
            localStorage.setItem('resursmap-theme', isLight ? 'light' : 'dark');
        } catch(e) {}

        // Обновим все кнопки темы на странице
        var allBtns = document.querySelectorAll('.theme-toggle-btn');
        for (var i = 0; i < allBtns.length; i++) {
            allBtns[i].textContent = isLight ? '🌙 Тёмная тема' : '☀️ Светлая тема';
        }
    });

    // Установим начальный текст на всех кнопках
    document.addEventListener('DOMContentLoaded', function() {
        var allBtns = document.querySelectorAll('.theme-toggle-btn');
        var isLight = document.body.classList.contains('light-theme');
        for (var i = 0; i < allBtns.length; i++) {
            allBtns[i].textContent = isLight ? '🌙 Тёмная тема' : '☀️ Светлая тема';
        }
    });
})();
