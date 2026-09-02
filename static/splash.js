// Заставка ResursMap — только при первом заходе на /app
(function() {
    var path = window.location.pathname || "";
    if (path !== "/app" && !path.startsWith("/app/")) {
        return;
    }

    function assetVersion() {
        var meta = document.querySelector(
            'meta[name="resursmap-asset-version"]'
        );

        if (meta && meta.content) {
            return meta.content;
        }

        return "";
    }

    try {
        if (localStorage.getItem('resursmap-splash-shown') === '1') {
            return;
        }
    } catch(e) {}

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
        'background:',
        'radial-gradient(circle at 20% 0%, rgba(126,212,228,.18), transparent 42%),',
        'radial-gradient(circle at 80% 10%, rgba(232,204,150,.16), transparent 38%),',
        'linear-gradient(160deg,#080a0d,#0e1116)',
        'transition:opacity .5s ease,visibility .5s ease',
        'opacity:1',
        'visibility:visible'
    ].join(';');

    var logo = document.createElement('img');
    logo.src = "/static/app-icon.svg?v=" + assetVersion();
    logo.alt = 'ResursMap';
    logo.style.cssText = [
        'width:104px',
        'height:104px',
        'border-radius:26px',
        'box-shadow:',
        '0 16px 48px rgba(0,0,0,.55),',
        '0 0 60px rgba(232,204,150,.18)',
        'animation:fadeInUp .6s ease both, pulseGlow 2.4s ease-in-out .6s infinite'
    ].join(';');

    var name = document.createElement('div');
    name.textContent = 'ResursMap';
    name.style.cssText = [
        'font-size:30px',
        'font-weight:900',
        'letter-spacing:.06em',
        'background:linear-gradient(135deg,#f8f5ef,#ffe4b8)',
        '-webkit-background-clip:text',
        'background-clip:text',
        '-webkit-text-fill-color:transparent',
        'animation:fadeInUp .6s ease .2s both'
    ].join(';');

    var subtitle = document.createElement('div');
    subtitle.textContent = 'Работа, люди и бизнес';
    subtitle.style.cssText = [
        'font-size:11px',
        'font-weight:800',
        'letter-spacing:.14em',
        'text-transform:uppercase',
        'color:#e8cc96',
        'animation:fadeInUp .6s ease .3s both'
    ].join(';');

    splash.appendChild(logo);
    splash.appendChild(name);
    splash.appendChild(subtitle);
    document.body.appendChild(splash);

    function playSoftSound() {
        if (typeof window.resursmapPlaySplashSound === "function") {
            window.resursmapPlaySplashSound();
        }
    }

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

    setTimeout(playSoftSound, 200);
})();
