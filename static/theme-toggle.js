// Переключатель темы ResursMap
(function() {
    const savedTheme = localStorage.getItem('resursmap-theme');
    if (savedTheme === 'light') {
        document.body.classList.add('light-theme');
    }
})();
