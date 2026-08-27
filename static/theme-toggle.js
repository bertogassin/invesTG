// Переключатель темы ResursMap
(function() {
    // Восстановим сохранённую тему
    try {
        var savedTheme = localStorage.getItem('resursmap-theme');
        if (savedTheme === 'light') {
            document.body.classList.add('light-theme');
        }
    } catch(e) {}

    // Обработчик клика
    document.addEventListener('click', function(event) {
        var btn = event.target.closest('.theme-toggle-btn');
        if (!btn) return;

        event.preventDefault();

        var isLight = document.body.classList.toggle('light-theme');

        try {
            localStorage.setItem('resursmap-theme', isLight ? 'light' : 'dark');
        } catch(e) {}

        btn.textContent = isLight ? '🌙' : '☀️';
    });

    // Установим начальную иконку
    document.addEventListener('DOMContentLoaded', function() {
        var btn = document.querySelector('.theme-toggle-btn');
        if (btn) {
            var isLight = document.body.classList.contains('light-theme');
            btn.textContent = isLight ? '🌙' : '☀️';
        }
    });
})();
