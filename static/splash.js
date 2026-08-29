// Заставка ResursMap — показывается только при первом заходе
(function() {
    // Проверим, была ли заставка показана
    try {
        if (localStorage.getItem('resursmap-splash-shown') === '1') {
            return;
        }
    } catch(e) {}

    // Создадим заставку
    var splash = document.createElement('div');
    splash.id = 'resursmap-splash';
    splash.style.cssText = [
        'position:fixed',
        'inset:0',
        'z-index:9999',
        'display:flex',
        'flex-direction:column',
        'align-items:center',
        'justify-content:center',
        'gap:18px',
        'background:linear-gradient(160deg,#080a0d,#0e1116)',
        'transition:opacity .5s ease,visibility .5s ease',
        'opacity:1',
        'visibility:visible'
    ].join(';');

    // Логотип
    var logo = document.createElement('img');
    logo.src = "/static/app-icon.svg";
    logo.alt = 'ResursMap';
    logo.style.cssText = [
        'width:96px',
        'height:96px',
        'border-radius:24px',
        'box-shadow:0 12px 40px rgba(0,0,0,.5)',
        'animation:fadeInUp .6s ease both'
    ].join(';');

    // Название
    var name = document.createElement('div');
    name.textContent = 'ResursMap';
    name.style.cssText = [
        'font-size:28px',
        'font-weight:900',
        'letter-spacing:.05em',
        'color:#f3f0e9',
        'animation:fadeInUp .6s ease .2s both'
    ].join(';');

    // Подпись
    var subtitle = document.createElement('div');
    subtitle.textContent = 'Resource Network';
    subtitle.style.cssText = [
        'font-size:11px',
        'font-weight:800',
        'letter-spacing:.15em',
        'text-transform:uppercase',
        'color:#d6b77a',
        'animation:fadeInUp .6s ease .3s both'
    ].join(';');

    splash.appendChild(logo);
    splash.appendChild(name);
    splash.appendChild(subtitle);
    document.body.appendChild(splash);

    // Мягкий звук через Web Audio API
    function playSoftSound() {
        try {
            var AudioContext = window.AudioContext || window.webkitAudioContext;
            if (!AudioContext) return;

            var ctx = new AudioContext();
            var oscillator = ctx.createOscillator();
            var gainNode = ctx.createGain();

            oscillator.type = 'sine';
            oscillator.frequency.setValueAtTime(440, ctx.currentTime);
            oscillator.frequency.exponentialRampToValueAtTime(880, ctx.currentTime + 0.15);

            gainNode.gain.setValueAtTime(0, ctx.currentTime);
            gainNode.gain.linearRampToValueAtTime(0.15, ctx.currentTime + 0.05);
            gainNode.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.8);

            oscillator.connect(gainNode);
            gainNode.connect(ctx.destination);

            oscillator.start(ctx.currentTime);
            oscillator.stop(ctx.currentTime + 0.8);
        } catch(e) {}
    }

    // Покажем заставку 1.5 секунды
    setTimeout(function() {
        splash.style.opacity = '0';
        splash.style.visibility = 'hidden';

        try {
            localStorage.setItem('resursmap-splash-shown', '1');
        } catch(e) {}

        setTimeout(function() {
            if (splash.parentNode) {
                splash.parentNode.removeChild(splash);
            }
        }, 500);
    }, 1500);

    // Звук после клика пользователя (браузеры блокируют autoplay)
    document.addEventListener('click', function() {
        playSoftSound();
    }, { once: true });

    // Если пользователь уже кликал — играем сразу
    if (document.readyState === 'complete') {
        playSoftSound();
    }
})();
