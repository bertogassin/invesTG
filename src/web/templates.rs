use std::collections::BTreeMap;

pub fn world() -> BTreeMap<&'static str, BTreeMap<&'static str, Vec<&'static str>>> {
    let mut w = BTreeMap::new();

    let mut eu = BTreeMap::new();

    eu.insert(
        "Франция",
        vec!["Париж", "Марсель", "Лион", "Тулуза", "Ницца"],
    );

    eu.insert("Германия", vec!["Берлин", "Гамбург", "Мюнхен", "Кёльн"]);

    eu.insert("Италия", vec!["Рим", "Милан", "Неаполь", "Турин"]);

    w.insert("Европа", eu);

    w
}

// ============================================================
// LUCIDE SVG
// ============================================================

pub(crate) fn icon(name: &str) -> &'static str {
    match name {
        "globe" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3c2.5 2.5 3.5 5.5 3.5 9s-1 6.5-3.5 9c-2.5-2.5-3.5-5.5-3.5-9S9.5 5.5 12 3Z"/></svg>"#
        }

        "map" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="m3 6 6-3 6 3 6-3v15l-6 3-6-3-6 3V6Z"/><path d="M9 3v15"/><path d="M15 6v15"/></svg>"#
        }

        "search" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg>"#
        }

        "user" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><circle cx="12" cy="8" r="4"/><path d="M4 21c.8-4 3.5-6 8-6s7.2 2 8 6"/></svg>"#
        }

        "star" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="m12 3 2.8 5.7 6.2.9-4.5 4.4 1.1 6.2-5.6-3-5.6 3 1.1-6.2L3 9.6l6.2-.9L12 3Z"/></svg>"#
        }

        "map-pin" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z"/><circle cx="12" cy="10" r="2.5"/></svg>"#
        }

        "chevron" => {
            r#"<svg class="icon small-icon" viewBox="0 0 24 24"><path d="m9 18 6-6-6-6"/></svg>"#
        }

        "briefcase" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><rect x="3" y="7" width="18" height="13" rx="2"/><path d="M8 7V5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M3 12h18"/></svg>"#
        }

        "building" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M4 21V4l8-2 8 2v17"/><path d="M8 7h1M15 7h1M8 11h1M15 11h1M8 15h1M15 15h1"/><path d="M10 21v-3h4v3"/></svg>"#
        }

        "heart" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M20.8 8.8c0 5.5-8.8 10.2-8.8 10.2S3.2 14.3 3.2 8.8A4.8 4.8 0 0 1 12 6a4.8 4.8 0 0 1 8.8 2.8Z"/></svg>"#
        }

        "menu" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M4 6h16M4 12h16M4 18h16"/></svg>"#
        }

        _ => "",
    }
}

// ============================================================
// GLOBAL DESIGN
// ============================================================

pub(crate) fn base_style() -> &'static str {
    r#"
:root {
    --bg: #080a0d;
    --bg-soft: #0e1116;
    --card: rgba(22, 25, 31, .78);
    --card-hover: rgba(29, 33, 40, .92);
    --line: rgba(255,255,255,.08);

    --text: #f3f0e9;
    --muted: #9298a3;

    --gold: #d6b77a;
    --gold-light: #f0d69c;

    --sea: #65b8c9;
    --sea-light: #8bd8e4;

    --radius: 22px;
}

* {
    box-sizing: border-box;
}

html {
    background: var(--bg);
    color-scheme: dark;
}

body {
    margin: 0;
    min-height: 100vh;

    font-family:
        Inter,
        -apple-system,
        BlinkMacSystemFont,
        "Segoe UI",
        sans-serif;

    color: var(--text);

    background:
        radial-gradient(
            circle at 15% 0%,
            rgba(101,184,201,.10),
            transparent 34%
        ),
        radial-gradient(
            circle at 85% 15%,
            rgba(214,183,122,.09),
            transparent 30%
        ),
        linear-gradient(
            145deg,
            #080a0d 0%,
            #0b0e12 45%,
            #080a0d 100%
        );
}

body::before {
    content: "";
    position: fixed;
    inset: 0;
    pointer-events: none;

    background:
        linear-gradient(
            120deg,
            transparent 0%,
            rgba(255,255,255,.018) 50%,
            transparent 100%
        );

    opacity: .5;
}

.page {
    width: min(100% - 32px, 900px);
    margin: 0 auto;
    padding: 28px 0 110px;
}

.icon {
    width: 22px;
    height: 22px;

    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;

    flex: 0 0 auto;
}

.small-icon {
    width: 18px;
    height: 18px;
}

/* -----------------------------------------------------------
   HEADER
----------------------------------------------------------- */

.topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;

    margin-bottom: 32px;
}

.brand {
    display: flex;
    align-items: center;
    gap: 12px;

    text-decoration: none;
    color: var(--text);
}

.brand-mark {
    width: 44px;
    height: 44px;

    display: grid;
    place-items: center;

    border: 1px solid rgba(214,183,122,.28);
    border-radius: 14px;

    color: var(--gold-light);

    background:
        linear-gradient(
            145deg,
            rgba(214,183,122,.15),
            rgba(255,255,255,.025)
        );

    box-shadow:
        0 10px 35px rgba(0,0,0,.28),
        inset 0 1px 0 rgba(255,255,255,.08);
}

.brand-name {
    font-size: 17px;
    font-weight: 700;
    letter-spacing: .08em;
}

.brand-sub {
    margin-top: 2px;
    color: var(--muted);
    font-size: 11px;
    letter-spacing: .08em;
}

/* -----------------------------------------------------------
   HERO
----------------------------------------------------------- */

.hero {
    position: relative;
    overflow: hidden;

    padding: 30px;

    border: 1px solid var(--line);
    border-radius: 30px;

    background:
        radial-gradient(
            circle at 100% 0%,
            rgba(214,183,122,.13),
            transparent 36%
        ),
        radial-gradient(
            circle at 0% 100%,
            rgba(101,184,201,.09),
            transparent 38%
        ),
        rgba(17,20,25,.78);

    box-shadow:
        0 30px 80px rgba(0,0,0,.32),
        inset 0 1px 0 rgba(255,255,255,.06);
}

.hero::after {
    content: "";
    position: absolute;
    width: 220px;
    height: 220px;

    right: -100px;
    bottom: -120px;

    border-radius: 50%;

    background: rgba(214,183,122,.08);
    filter: blur(10px);
}

.eyebrow {
    display: flex;
    align-items: center;
    gap: 8px;

    color: var(--gold-light);
    font-size: 12px;
    font-weight: 600;

    letter-spacing: .12em;
    text-transform: uppercase;
}

.eyebrow-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--gold);
    box-shadow: 0 0 14px rgba(214,183,122,.7);
}

.hero h1 {
    margin: 14px 0 10px;

    font-size: clamp(32px, 7vw, 56px);
    line-height: 1.02;
    letter-spacing: -.045em;
}

.hero p {
    max-width: 620px;

    margin: 0;

    color: var(--muted);
    font-size: 15px;
    line-height: 1.7;
}

/* -----------------------------------------------------------
   SEARCH
----------------------------------------------------------- */

.search {
    display: flex;
    align-items: center;
    gap: 12px;

    margin-top: 24px;
    padding: 15px 17px;

    color: var(--muted);

    border: 1px solid var(--line);
    border-radius: 16px;

    background: rgba(255,255,255,.035);
}

.search input {
    width: 100%;

    border: 0;
    outline: 0;

    color: var(--text);
    background: transparent;

    font: inherit;
}

.search input::placeholder {
    color: #666d78;
}

/* -----------------------------------------------------------
   SECTION
----------------------------------------------------------- */

.section-head {
    display: flex;
    align-items: end;
    justify-content: space-between;

    margin: 34px 4px 14px;
}

.section-title {
    margin: 0;

    font-size: 18px;
    letter-spacing: -.02em;
}

.section-caption {
    margin: 5px 0 0;

    color: var(--muted);
    font-size: 12px;
}

/* -----------------------------------------------------------
   CARDS
----------------------------------------------------------- */

.grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 13px;
}

.card {
    position: relative;

    display: flex;
    align-items: center;
    gap: 15px;

    min-height: 92px;
    padding: 18px;

    color: var(--text);
    text-decoration: none;

    border: 1px solid var(--line);
    border-radius: var(--radius);

    background:
        linear-gradient(
            145deg,
            rgba(255,255,255,.055),
            rgba(255,255,255,.018)
        ),
        var(--card);

    box-shadow:
        0 16px 40px rgba(0,0,0,.18),
        inset 0 1px 0 rgba(255,255,255,.035);

    transition:
        transform .2s ease,
        border-color .2s ease,
        background .2s ease,
        box-shadow .2s ease;
}

.card:hover {
    transform: translateY(-2px);

    border-color: rgba(214,183,122,.24);

    background: var(--card-hover);

    box-shadow:
        0 20px 48px rgba(0,0,0,.28),
        0 0 0 1px rgba(214,183,122,.04);
}

.card-icon {
    width: 48px;
    height: 48px;

    display: grid;
    place-items: center;

    flex: 0 0 auto;

    color: var(--gold-light);

    border: 1px solid rgba(214,183,122,.20);
    border-radius: 16px;

    background:
        linear-gradient(
            145deg,
            rgba(214,183,122,.13),
            rgba(214,183,122,.035)
        );
}

.card-content {
    min-width: 0;
    flex: 1;
}

.card-title {
    font-size: 15px;
    font-weight: 650;
}

.card-meta {
    margin-top: 5px;

    color: var(--muted);
    font-size: 12px;
}

.card-arrow {
    color: #656c76;
}

/* -----------------------------------------------------------
   FEATURE CARDS
----------------------------------------------------------- */

.feature-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
}

.feature {
    min-height: 125px;
    padding: 18px;

    border: 1px solid var(--line);
    border-radius: 20px;

    background: rgba(255,255,255,.025);
}

.feature .icon {
    color: var(--sea-light);
}

.feature strong {
    display: block;
    margin-top: 18px;

    font-size: 14px;
}

.feature span {
    display: block;
    margin-top: 5px;

    color: var(--muted);
    font-size: 11px;
    line-height: 1.5;
}

/* -----------------------------------------------------------
   BOTTOM NAV
----------------------------------------------------------- */

.bottom-nav {
    position: fixed;
    z-index: 20;

    left: 50%;
    bottom: 16px;

    width: min(calc(100% - 28px), 520px);

    transform: translateX(-50%);

    display: grid;
    grid-template-columns: repeat(4, 1fr);

    padding: 8px;

    border: 1px solid rgba(255,255,255,.09);
    border-radius: 22px;

    background: rgba(13,16,21,.88);

    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);

    box-shadow:
        0 20px 60px rgba(0,0,0,.5),
        inset 0 1px 0 rgba(255,255,255,.06);
}

.nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;

    gap: 5px;

    min-height: 52px;

    color: #777e88;
    text-decoration: none;

    border-radius: 16px;

    font-size: 10px;

    transition: .2s ease;
}

.nav-item:hover,
.nav-item.active {
    color: var(--gold-light);
    background: rgba(214,183,122,.08);
}

.nav-item .icon {
    width: 19px;
    height: 19px;
}

/* -----------------------------------------------------------
   MOBILE
----------------------------------------------------------- */


/* ============================================================
   RESURSMAP LUXURY LAYER
   ============================================================ */

.topbar {
    position: relative;
    z-index: 5;
}

.brand {
    transition: transform .25s ease, opacity .25s ease;
}

.brand:hover {
    transform: translateY(-1px);
    opacity: .92;
}

.brand-mark {
    background:
        radial-gradient(circle at 30% 25%,
            rgba(240,214,156,.28),
            transparent 48%),
        linear-gradient(145deg,
            rgba(214,183,122,.18),
            rgba(101,184,201,.08));
    border: 1px solid rgba(214,183,122,.28);
    box-shadow:
        0 8px 30px rgba(0,0,0,.35),
        inset 0 1px 0 rgba(255,255,255,.10);
    backdrop-filter: blur(18px);
}

.hero {
    position: relative;
    overflow: hidden;
    border: 1px solid rgba(255,255,255,.08);
    box-shadow:
        0 24px 70px rgba(0,0,0,.28),
        inset 0 1px 0 rgba(255,255,255,.05);
}

.hero::before {
    content: "";
    position: absolute;
    width: 280px;
    height: 280px;
    top: -160px;
    right: -100px;
    border-radius: 50%;
    background: rgba(101,184,201,.10);
    filter: blur(45px);
    pointer-events: none;
}

.hero::after {
    content: "";
    position: absolute;
    width: 220px;
    height: 220px;
    bottom: -150px;
    left: -80px;
    border-radius: 50%;
    background: rgba(214,183,122,.08);
    filter: blur(45px);
    pointer-events: none;
}

.search {
    position: relative;
    z-index: 2;
    border: 1px solid rgba(255,255,255,.10);
    background: rgba(255,255,255,.045);
    box-shadow:
        inset 0 1px 0 rgba(255,255,255,.05),
        0 12px 35px rgba(0,0,0,.18);
    transition:
        border-color .25s ease,
        box-shadow .25s ease,
        background .25s ease;
}

.search:focus-within {
    border-color: rgba(214,183,122,.45);
    background: rgba(255,255,255,.065);
    box-shadow:
        0 0 0 4px rgba(214,183,122,.06),
        0 16px 45px rgba(0,0,0,.25);
}

.card {
    position: relative;
    overflow: hidden;
    border: 1px solid rgba(255,255,255,.075);
    background:
        linear-gradient(
            145deg,
            rgba(255,255,255,.055),
            rgba(255,255,255,.018)
        );
    box-shadow:
        0 12px 35px rgba(0,0,0,.18),
        inset 0 1px 0 rgba(255,255,255,.045);
    backdrop-filter: blur(14px);
    transition:
        transform .25s ease,
        border-color .25s ease,
        box-shadow .25s ease,
        background .25s ease;
}

.card::before {
    content: "";
    position: absolute;
    inset: 0;
    background:
        linear-gradient(
            120deg,
            transparent 20%,
            rgba(255,255,255,.035) 50%,
            transparent 80%
        );
    transform: translateX(-100%);
    transition: transform .55s ease;
    pointer-events: none;
}

.card:hover {
    transform: translateY(-4px);
    border-color: rgba(214,183,122,.25);
    background:
        linear-gradient(
            145deg,
            rgba(255,255,255,.075),
            rgba(255,255,255,.025)
        );
    box-shadow:
        0 20px 50px rgba(0,0,0,.30),
        0 0 35px rgba(214,183,122,.055),
        inset 0 1px 0 rgba(255,255,255,.07);
}

.card:hover::before {
    transform: translateX(100%);
}

.card-icon {
    color: var(--gold-light);
    transition: transform .25s ease, color .25s ease;
}

.card:hover .card-icon {
    transform: scale(1.08);
    color: var(--sea-light);
}

.card-arrow {
    color: rgba(214,183,122,.70);
    transition: transform .25s ease, color .25s ease;
}

.card:hover .card-arrow {
    transform: translateX(4px);
    color: var(--gold-light);
}

.feature {
    border: 1px solid rgba(255,255,255,.07);
    background:
        linear-gradient(
            145deg,
            rgba(255,255,255,.045),
            rgba(255,255,255,.015)
        );
    box-shadow:
        0 12px 35px rgba(0,0,0,.16),
        inset 0 1px 0 rgba(255,255,255,.04);
    backdrop-filter: blur(12px);
    transition:
        transform .25s ease,
        border-color .25s ease,
        box-shadow .25s ease;
}

.feature:hover {
    transform: translateY(-3px);
    border-color: rgba(101,184,201,.22);
    box-shadow:
        0 18px 45px rgba(0,0,0,.25),
        0 0 30px rgba(101,184,201,.045);
}

.feature .icon {
    color: var(--gold-light);
}

.bottom-nav {
    border-top: 1px solid rgba(255,255,255,.08);
    background: rgba(8,10,13,.78);
    box-shadow:
        0 -12px 40px rgba(0,0,0,.25),
        inset 0 1px 0 rgba(255,255,255,.04);
    backdrop-filter: blur(22px);
}

.nav-item {
    transition:
        color .2s ease,
        transform .2s ease;
}

.nav-item:hover {
    transform: translateY(-2px);
}

.nav-item.active {
    color: var(--gold-light);
}

.nav-item.active .icon {
    filter: drop-shadow(0 0 8px rgba(214,183,122,.25));
}

.section-title {
    letter-spacing: -.02em;
}

.section-caption {
    color: rgba(255,255,255,.48);
}

@media (max-width: 620px) {
    .page {
        width: min(100% - 20px, 900px);
        padding-top: 18px;
    }

    .hero {
        padding: 24px 20px;
        border-radius: 24px;
    }

    .hero h1 {
        font-size: 36px;
    }

    .grid {
        grid-template-columns: 1fr;
    }

    .feature-grid {
        grid-template-columns: 1fr;
    }

    .feature {
        min-height: auto;
    }

    .brand-mark {
        width: 40px;
        height: 40px;
    }
}
"#
}

// ============================================================
// MAIN APP
// ============================================================

pub fn render_continents() -> String {
    let w = world();
    let mut cards = String::new();

    for (ci, (continent, countries)) in w.iter().enumerate() {
        for (si, country) in countries.keys().enumerate() {
            cards.push_str(&format!(
                r#"
<a class="card" href="/app/{ci}/{si}">
    <div class="card-icon">{building}</div>

    <div class="card-content">
        <div class="card-title">{country}</div>
        <div class="card-meta">{continent} · страна</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                building = icon("building"),
                arrow = icon("chevron"),
            ));
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>ResursMap</title>
<style>{style}</style>
</head>

<body>
<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">
    <div class="eyebrow">
        <span class="eyebrow-dot"></span>
        ResursMap
    </div>

    <h1>Карта ресурсов</h1>

    <p>
        Люди, города, услуги и возможности —
        всё необходимое рядом с вами.
    </p>

    <div class="search">
        {search_icon}
        <input
            type="text"
            placeholder="Найти город, услугу или ресурс..."
            onkeydown="if(event.key==='Enter') window.location='/app/search?q='+encodeURIComponent(this.value)"
        >
    </div>
</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Страны</h2>
        <p class="section-caption">Выберите страну для продолжения</p>
    </div>
</div>

<div class="grid">
    {cards}
</div>

<div class="section-head">
    <div>
        <h2 class="section-title">Возможности</h2>
        <p class="section-caption">Всё необходимое в одном месте</p>
    </div>
</div>

<div class="feature-grid">
    <div class="feature">
        {map_icon}
        <strong>Ресурсы</strong>
        <span>Находите полезные места и услуги.</span>
    </div>

    <div class="feature">
        {heart_icon}
        <strong>Сообщество</strong>
        <span>Люди и возможности вашего города.</span>
    </div>

    <div class="feature">
        {user_icon}
        <strong>Профиль</strong>
        <span>Ваши данные, голоса и активность.</span>
    </div>
</div>

</main>

<nav class="bottom-nav">
    <a class="nav-item active" href="/app">
        {nav_map}
        <span>Карта</span>
    </a>

    <a class="nav-item" href="/app/search">
        {nav_search}
        <span>Поиск</span>
    </a>

    <a class="nav-item" href="/app/me">
        {nav_user}
        <span>Профиль</span>
    </a>

    <a class="nav-item" href="/app">
        {nav_menu}
        <span>Меню</span>
    </a>
</nav>

</body>
</html>"#,
        style = base_style(),
        logo = icon("globe"),
        search_icon = icon("search"),
        cards = cards,
        map_icon = icon("map"),
        heart_icon = icon("heart"),
        user_icon = icon("user"),
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
    )
}

// ============================================================
// CONTINENT
// ============================================================

pub fn render_continent(ci: usize) -> String {
    let w = world();

    if let Some((name, countries)) = w.iter().nth(ci) {
        let mut cards = String::new();

        for (si, country) in countries.keys().enumerate() {
            cards.push_str(&format!(
                r#"
<a class="card" href="/app/{ci}/{si}">
    <div class="card-icon">
        {icon}
    </div>

    <div class="card-content">
        <div class="card-title">{country}</div>
        <div class="card-meta">Открыть города</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                icon = icon("building"),
                arrow = icon("chevron"),
            ));
        }

        return format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{name} · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">
    <div class="eyebrow">
        {}
        Регион
    </div>

    <h1>{name}</h1>

    <p>Выберите страну, чтобы открыть доступные города и ресурсы.</p>
</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Страны</h2>
        <p class="section-caption">{count} доступных</p>
    </div>
</div>

<div class="grid">
    {cards}
</div>

</main>

<nav class="bottom-nav">
    <a class="nav-item active" href="/app">
        {}
        <span>Карта</span>
    </a>

    <a class="nav-item" href="/app/search">
        {}
        <span>Поиск</span>
    </a>

    <a class="nav-item" href="/app/me">
        {}
        <span>Профиль</span>
    </a>

    <a class="nav-item" href="/app">
        {count}
        <span>Меню</span>
    </a>
</nav>

</body>
</html>"#,
            base_style(),
            icon("globe"),
            icon("map"),
            icon("search"),
            icon("user"),
            icon("menu"),
            count = countries.len(),
        );
    }

    render_continents()
}

// ============================================================
// COUNTRY
// ============================================================

pub fn render_country(ci: usize, si: usize) -> String {
    let w = world();

    if let Some((cname, countries)) = w.iter().nth(ci) {
        if let Some((country, cities)) = countries.iter().nth(si) {
            let mut cards = String::new();

            for (zi, city) in cities.iter().enumerate() {
                cards.push_str(&format!(
                    r#"
<a class="card" href="/app/{ci}/{si}/{zi}">
    <div class="card-icon">
        {pin}
    </div>

    <div class="card-content">
        <div class="card-title">{city}</div>
        <div class="card-meta">{country}</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                    pin = icon("map-pin"),
                    arrow = icon("chevron"),
                ));
            }

            return format!(
                r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{country} · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">
    <div class="eyebrow">
        {}
        {cname}
    </div>

    <h1>{country}</h1>

    <p>Выберите город и откройте его карту ресурсов.</p>
</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Города</h2>
        <p class="section-caption">{count} городов</p>
    </div>
</div>

<div class="grid">
    {cards}
</div>

</main>

<nav class="bottom-nav">
    <a class="nav-item active" href="/app">
        {}
        <span>Карта</span>
    </a>

    <a class="nav-item" href="/app/search">
        {}
        <span>Поиск</span>
    </a>

    <a class="nav-item" href="/app/me">
        {}
        <span>Профиль</span>
    </a>

    <a class="nav-item" href="/app">
        {count}
        <span>Меню</span>
    </a>
</nav>

</body>
</html>"#,
                base_style(),
                icon("globe"),
                icon("map-pin"),
                icon("search"),
                icon("user"),
                icon("menu"),
                count = cities.len(),
            );
        }
    }

    render_continents()
}

// ============================================================
// CITY
// ============================================================

pub fn render_city(ci: usize, si: usize, zi: usize) -> String {
    let w = world();

    if let Some((cname, countries)) = w.iter().nth(ci) {
        if let Some((country, cities)) = countries.iter().nth(si) {
            if let Some(city) = cities.get(zi) {
                return format!(
                    r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{city} · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {}
        {country}
    </div>

    <h1>{city}</h1>

    <p>
        {cname} · {country}
        <br>
        Карта ресурсов города.
    </p>

</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Разделы</h2>
        <p class="section-caption">Выберите направление</p>
    </div>
</div>

<div class="grid">

    <a class="card" href="/app/{ci}/{si}/{zi}/cat/work">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Работа</div>
            <div class="card-meta">Вакансии и предложения</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

    <a class="card" href="/app/{ci}/{si}/{zi}/cat/business">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Бизнес</div>
            <div class="card-meta">Компании и услуги</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

    <a class="card" href="/app/{ci}/{si}/{zi}/cat/services">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Услуги</div>
            <div class="card-meta">Помощь и специалисты</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

    <a class="card" href="/app/{ci}/{si}/{zi}/cat/community">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Сообщество</div>
            <div class="card-meta">Люди и контакты</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

</div>

</main>

<nav class="bottom-nav">

    <a class="nav-item active" href="/app">
        {}
        <span>Карта</span>
    </a>

    <a class="nav-item" href="/app/search">
        {}
        <span>Поиск</span>
    </a>

    <a class="nav-item" href="/app/me">
        {}
        <span>Профиль</span>
    </a>

    <a class="nav-item" href="/app">
        {}
        <span>Меню</span>
    </a>

</nav>

</body>
</html>"#,
                    base_style(),
                    icon("globe"),
                    icon("map-pin"),
                    icon("briefcase"),
                    icon("chevron"),
                    icon("building"),
                    icon("chevron"),
                    icon("map"),
                    icon("chevron"),
                    icon("user"),
                    icon("chevron"),
                    icon("map"),
                    icon("search"),
                    icon("user"),
                    icon("menu"),
                );
            }
        }
    }

    render_continents()
}

// ============================================================
// SEARCH
// ============================================================

pub fn render_search(
    q: &str,
    resources: Vec<(
        i64,
        String,
        String,
        String,
        String,
        f64,
        i64,
        i64,
        i64,
        usize,
        usize,
        usize,
    )>,
    people: Vec<(String, String, String, String, i64, String, i64)>,
) -> String {
    let world_data = world();

    let query_lower = q.trim().to_lowercase();

    let mut location_results = String::new();
    let mut location_count = 0usize;

    if !query_lower.is_empty() {
        for (ci, (continent, countries)) in world_data.iter().enumerate() {
            for (si, (country, cities)) in countries.iter().enumerate() {
                let country_match = country.to_lowercase().contains(&query_lower);

                if country_match {
                    location_count += 1;

                    location_results.push_str(&format!(
                        r#"
<a href="/app/{ci}/{si}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:12px;
   ">
    <div class="card-icon">{icon}</div>

    <div class="card-content">
        <div class="card-title">{country}</div>
        <div class="card-meta">{continent} · страна</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                        ci = ci,
                        si = si,
                        icon = icon("building"),
                        country = country,
                        continent = continent,
                        arrow = icon("chevron"),
                    ));
                }

                for (zi, city) in cities.iter().enumerate() {
                    if city.to_lowercase().contains(&query_lower) {
                        location_count += 1;

                        location_results.push_str(&format!(
                            r#"
<a href="/app/{ci}/{si}/{zi}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:12px;
   ">
    <div class="card-icon">{icon}</div>

    <div class="card-content">
        <div class="card-title">{city}</div>
        <div class="card-meta">{country} · город</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                            ci = ci,
                            si = si,
                            zi = zi,
                            icon = icon("map-pin"),
                            city = city,
                            country = country,
                            arrow = icon("chevron"),
                        ));
                    }
                }
            }
        }
    }

    let people_count = people.len();

    let people_section = if q.trim().is_empty() || people.is_empty() {
        String::new()
    } else {
        let people_cards = people
            .iter()
            .map(
                |(
                    public_id,
                    username,
                    first_name,
                    last_name,
                    open_contact,
                    intent_text,
                    intent_until,
                )| {
                    let escape_html = |value: &str| -> String {
                        value
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;")
                            .replace('\'', "&#39;")
                    };

                    let safe_username = escape_html(username);
                    let safe_first_name = escape_html(first_name);
                    let safe_last_name = escape_html(last_name);
                    let safe_intent = escape_html(intent_text);

                    let full_name = format!("{} {}", safe_first_name, safe_last_name)
                        .trim()
                        .to_string();

                    let display_name = if !full_name.is_empty() {
                        full_name
                    } else if !safe_username.is_empty() {
                        format!("@{}", safe_username)
                    } else {
                        "Участник ResursMap".to_string()
                    };

                    let username_html = if !safe_username.is_empty()
                        && display_name != format!("@{}", safe_username)
                    {
                        format!(
                            r#"<div class="card-meta"
                                     style="margin-top:4px;">
                                    @{username}
                                </div>"#,
                            username = safe_username
                        )
                    } else {
                        String::new()
                    };

                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);

                    let intent_is_active = !safe_intent.trim().is_empty()
                        && (*intent_until == 0 || *intent_until >= now);

                    let intent_html = if intent_is_active {
                        format!(
                            r#"
<div style="
    margin-top:10px;
    padding:10px 12px;
    border-radius:12px;
    border:1px solid rgba(214,183,122,.20);
    background:rgba(214,183,122,.07);
    font-size:13px;
    line-height:1.45;
    overflow-wrap:anywhere;
">
    {intent}
</div>
"#,
                            intent = safe_intent
                        )
                    } else {
                        String::new()
                    };

                    let contact_html = if *open_contact != 0 {
                        r#"<span style="
                                font-size:10px;
                                font-weight:850;
                                color:#16a34a;
                            ">● Контакт открыт</span>"#
                    } else {
                        r#"<span style="
                                font-size:10px;
                                font-weight:750;
                                color:var(--muted);
                            ">Контакт закрыт</span>"#
                    };

                    format!(
                        r#"
<a href="/app/user/{public_id}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:14px;
       align-items:flex-start;
   ">

    <div class="card-icon">
        {user_icon}
    </div>

    <div class="card-content"
         style="min-width:0;">

        <div class="card-title"
             style="
                 font-size:17px;
                 line-height:1.3;
                 overflow-wrap:anywhere;
             ">
            {display_name}
        </div>

        {username_html}

        {intent_html}

        <div style="
            display:flex;
            gap:8px;
            flex-wrap:wrap;
            margin-top:10px;
        ">
            {contact_html}
        </div>

    </div>

    <div class="card-arrow">
        {arrow}
    </div>

</a>
"#,
                        public_id = public_id,
                        user_icon = icon("user"),
                        display_name = display_name,
                        username_html = username_html,
                        intent_html = intent_html,
                        contact_html = contact_html,
                        arrow = icon("chevron"),
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("");

        format!(
            r#"
<div class="section-head"
     style="margin-top:24px;">
    <div>
        <h2 class="section-title">Участники</h2>
        <p class="section-caption">
            Найдено: {people_count}
        </p>
    </div>
</div>

<section>
    {people_cards}
</section>
"#,
            people_count = people_count,
            people_cards = people_cards,
        )
    };

    let result_count = resources.len();

    let results = if q.trim().is_empty() {
        r#"
        <div class="card" style="display:block;margin-top:18px;">
            <div class="card-content">
                <div class="card-title">Начните поиск</div>
                <div class="card-meta" style="margin-top:5px;">
                    Введите название, услугу, категорию или адрес.
                </div>
            </div>
        </div>
        "#
        .to_string()
    } else if resources.is_empty() && people.is_empty() && location_results.is_empty() {
        format!(
            r#"
        <div class="card" style="display:block;margin-top:18px;">
            <div class="card-content">
                <div class="card-title">Ничего не найдено</div>
                <div class="card-meta" style="margin-top:5px;">
                    По запросу «{}» пока ничего не найдено.
                </div>
            </div>
        </div>
        "#,
            q
        )
    } else if resources.is_empty() {
        String::new()
    } else {
        resources
            .iter()
            .map(
                |(
                    id,
                    title,
                    category,
                    description,
                    address,
                    rating,
                    votes,
                    verified,
                    premium,
                    ci,
                    si,
                    zi,
                )| {
                    let location = world_data
                        .iter()
                        .nth(*ci)
                        .and_then(|(_, countries)| {
                            countries.iter().nth(*si).and_then(|(country, cities)| {
                                cities
                                    .get(*zi)
                                    .map(|city| format!("{} · {}", city, country))
                            })
                        })
                        .unwrap_or_else(|| "Местоположение не указано".to_string());

                    let premium_badge = if *premium != 0 {
                        r#"<span style="
                            display:inline-flex;
                            align-items:center;
                            padding:4px 8px;
                            border-radius:999px;
                            border:1px solid rgba(214,183,122,.38);
                            background:rgba(214,183,122,.10);
                            color:#d6b77a;
                            font-size:10px;
                            font-weight:800;
                        ">★ PREMIUM</span>"#
                    } else {
                        ""
                    };

                    let verified_badge = if *verified != 0 {
                        r#"<span style="
                            color:#16a34a;
                            font-size:11px;
                            font-weight:800;
                        ">✓ Проверен</span>"#
                    } else {
                        ""
                    };

                    format!(
                        r#"
<a href="/app/resource/{id}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:14px;
       align-items:flex-start;
   ">

    <div class="card-icon">{resource_icon}</div>

    <div class="card-content" style="min-width:0;">

        <div style="
            display:flex;
            justify-content:space-between;
            align-items:flex-start;
            gap:10px;
        ">
            <div class="card-title"
                 style="font-size:17px;min-width:0;">
                {title}
            </div>

            <div style="
                flex:0 0 auto;
                font-size:12px;
                color:var(--muted);
                white-space:nowrap;
            ">
                ⭐ {rating:.1} · {votes}
            </div>
        </div>

        <div class="card-meta"
             style="margin-top:5px;">
            {category}
        </div>

        <div class="card-meta"
             style="
                 margin-top:8px;
                 line-height:1.45;
                 overflow-wrap:anywhere;
             ">
            {description}
        </div>

        <div class="card-meta"
             style="margin-top:9px;">
            📍 {location}
        </div>

        <div class="card-meta"
             style="
                 margin-top:4px;
                 overflow-wrap:anywhere;
             ">
            {address}
        </div>

        <div style="
            display:flex;
            gap:8px;
            align-items:center;
            flex-wrap:wrap;
            margin-top:10px;
        ">
            {premium_badge}
            {verified_badge}
        </div>

    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                        id = id,
                        resource_icon = icon("map-pin"),
                        arrow = icon("chevron"),
                        title = title,
                        category = category,
                        description = description,
                        rating = rating,
                        votes = votes,
                        location = location,
                        address = address,
                        premium_badge = premium_badge,
                        verified_badge = verified_badge,
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let location_section = if q.trim().is_empty() || location_results.is_empty() {
        String::new()
    } else {
        format!(
            r#"
<div class="section-head"
     style="margin-top:24px;">
    <div>
        <h2 class="section-title">Места</h2>
        <p class="section-caption">
            Найдено: {location_count}
        </p>
    </div>
</div>

<section>
    {location_results}
</section>
"#,
            location_count = location_count,
            location_results = location_results,
        )
    };

    let result_header = if q.trim().is_empty() || resources.is_empty() {
        String::new()
    } else {
        format!(
            r#"
<div class="section-head"
     style="margin-top:24px;">
    <div>
        <h2 class="section-title">Результаты</h2>
        <p class="section-caption">
            Найдено: {}
        </p>
    </div>
</div>
"#,
            result_count
        )
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Поиск · ResursMap</title>
<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">SEARCH</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {search_icon}
        Поиск
    </div>

    <h1>Найти ресурс</h1>

    <p>
        Ищите услуги, компании, специалистов и другие ресурсы.
    </p>

    <form method="get"
          action="/app/search"
          class="search">
        {search_input_icon}

        <input
            name="q"
            autofocus
            value="{q}"
            placeholder="Например: охранник, бизнес, улица..."
        >
    </form>

</section>

{location_section}

{people_section}

{result_header}

<section>
    {results}
</section>

</main>

<nav class="bottom-nav">

    <a class="nav-item" href="/app">
        {nav_map}
        <span>Карта</span>
    </a>

    <a class="nav-item active" href="/app/search">
        {nav_search}
        <span>Поиск</span>
    </a>

    <a class="nav-item" href="/app/me">
        {nav_user}
        <span>Профиль</span>
    </a>

    <a class="nav-item" href="/app">
        {nav_menu}
        <span>Меню</span>
    </a>

</nav>

</body>
</html>"#,
        style = base_style(),
        logo = icon("search"),
        search_icon = icon("search"),
        search_input_icon = icon("search"),
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
        q = q,
        location_section = location_section,
        people_section = people_section,
        result_header = result_header,
        results = results,
    )
}

// ============================================================
// CONTACT REQUESTS
// ============================================================

pub fn render_contact_requests(
    requests: Vec<(i64, i64, String, String, String, String, String, i64, i64)>,
    authenticated: bool,
) -> String {
    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let pending_count = requests.iter().filter(|r| r.3 == "pending").count();

    let cards = if !authenticated {
        r#"
<div class="card"
     style="
         display:block;
         padding:20px;
     ">

    <div class="card-title">
        Откройте ResursMap через Telegram
    </div>

    <div class="card-meta"
         style="
             margin-top:6px;
             line-height:1.5;
         ">
        После входа здесь будут ваши запросы на связь.
    </div>

</div>
"#
        .to_string()
    } else if requests.is_empty() {
        r#"
<div class="card"
     style="
         display:block;
         padding:20px;
     ">

    <div class="card-title">
        Входящих запросов пока нет
    </div>

    <div class="card-meta"
         style="
             margin-top:6px;
             line-height:1.5;
         ">
        Когда кто-то захочет связаться через ResursMap,
        запрос появится здесь.
    </div>

</div>
"#
        .to_string()
    } else {
        requests
            .iter()
            .map(
                |(
                    request_id,
                    sender_user_id,
                    message,
                    status,
                    public_id,
                    username,
                    first_name,
                    created_at,
                    _sort,
                )| {
                    let safe_message = escape_html(message);

                    let safe_username = escape_html(username);

                    let safe_first_name = escape_html(first_name);

                    let display_name = if !safe_first_name.is_empty() {
                        safe_first_name
                    } else if !safe_username.is_empty() {
                        format!("@{}", safe_username)
                    } else {
                        format!("Участник #{}", sender_user_id)
                    };

                    let username_html = if !safe_username.is_empty() {
                        format!(
                            r#"<div class="card-meta"
                                         style="margin-top:3px;">
                                        @{username}
                                    </div>"#,
                            username = safe_username,
                        )
                    } else {
                        String::new()
                    };

                    let profile_link = if !public_id.trim().is_empty() {
                        format!(
                            r#"
<a href="/app/user/{public_id}"
   style="
       margin-top:12px;
       min-height:40px;
       display:inline-flex;
       align-items:center;
       justify-content:center;
       padding:0 13px;
       border-radius:12px;
       border:1px solid rgba(255,255,255,.10);
       text-decoration:none;
       color:inherit;
       font-size:12px;
       font-weight:800;
   ">
    Открыть профиль
</a>
"#,
                            public_id = public_id,
                        )
                    } else {
                        String::new()
                    };

                    let status_badge = match status.as_str() {
                        "accepted" => {
                            r#"<span style="
                                        color:#16a34a;
                                        font-size:11px;
                                        font-weight:850;
                                    ">✓ Принят</span>"#
                        }

                        "rejected" => {
                            r#"<span style="
                                        color:#dc2626;
                                        font-size:11px;
                                        font-weight:850;
                                    ">✕ Отклонён</span>"#
                        }

                        _ => {
                            r#"<span style="
                                        color:#d97706;
                                        font-size:11px;
                                        font-weight:850;
                                    ">● Ожидает ответа</span>"#
                        }
                    };

                    let actions = if status == "pending" {
                        format!(
                            r#"
<div style="
    display:grid;
    grid-template-columns:repeat(2,minmax(0,1fr));
    gap:10px;
    margin-top:16px;
">

    <form method="post"
          action="/app/contact-request/{id}/accept">

        <button type="submit"
                style="
                    width:100%;
                    min-height:44px;
                    border-radius:12px;
                    border:1px solid rgba(22,163,74,.35);
                    background:rgba(22,163,74,.10);
                    color:inherit;
                    font-weight:850;
                    cursor:pointer;
                ">
            ✓ Принять
        </button>

    </form>

    <form method="post"
          action="/app/contact-request/{id}/reject">

        <button type="submit"
                style="
                    width:100%;
                    min-height:44px;
                    border-radius:12px;
                    border:1px solid rgba(220,38,38,.30);
                    background:rgba(220,38,38,.08);
                    color:inherit;
                    font-weight:850;
                    cursor:pointer;
                ">
            ✕ Отклонить
        </button>

    </form>

</div>
"#,
                            id = request_id,
                        )
                    } else if status == "accepted" {
                        format!(
                            r#"
<a href="/app/chat/{sender_user_id}"
   style="
       margin-top:16px;
       min-height:46px;
       display:flex;
       align-items:center;
       justify-content:center;
       padding:0 15px;
       border-radius:13px;
       text-decoration:none;
       color:var(--text);
       font-size:13px;
       font-weight:850;
       border:1px solid rgba(214,183,122,.38);
       background:rgba(214,183,122,.10);
   ">
    💬 Открыть чат
</a>
"#,
                            sender_user_id = sender_user_id,
                        )
                    } else {
                        String::new()
                    };

                    format!(
                        r#"
<article class="card"
         style="
             display:block;
             padding:18px;
             margin-bottom:14px;
         ">

    <div style="
        display:flex;
        align-items:flex-start;
        gap:13px;
    ">

        <div class="card-icon">
            {user_icon}
        </div>

        <div style="
            flex:1;
            min-width:0;
        ">

            <div style="
                display:flex;
                justify-content:space-between;
                align-items:flex-start;
                gap:10px;
            ">

                <div>

                    <div class="card-title"
                         style="
                             font-size:17px;
                             overflow-wrap:anywhere;
                         ">
                        {display_name}
                    </div>

                    {username_html}

                </div>

                <div style="
                    flex:0 0 auto;
                ">
                    {status_badge}
                </div>

            </div>

            <div style="
                margin-top:12px;
                padding:12px 13px;
                border-radius:12px;
                background:rgba(255,255,255,.035);
                border:1px solid rgba(255,255,255,.07);
                font-size:14px;
                line-height:1.5;
                overflow-wrap:anywhere;
                white-space:pre-wrap;
            ">
                {message}
            </div>

            {profile_link}

            {actions}

            <div class="card-meta"
                 style="
                     margin-top:10px;
                     font-size:10px;
                 ">
                REQUEST #{request_id}
                · {created_at}
            </div>

        </div>

    </div>

</article>
"#,
                        user_icon = icon("user"),
                        display_name = display_name,
                        username_html = username_html,
                        status_badge = status_badge,
                        message = safe_message,
                        profile_link = profile_link,
                        actions = actions,
                        request_id = request_id,
                        created_at = created_at,
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">

<head>

<meta charset="utf-8">

<meta name="viewport"
      content="width=device-width, initial-scale=1">

<title>Запросы · ResursMap</title>

<style>{style}</style>

</head>

<body>

<main class="page">

<header class="topbar">

    <a class="brand"
       href="/app">

        <div class="brand-mark">
            {logo}
        </div>

        <div>

            <div class="brand-name">
                RESURSMAP
            </div>

            <div class="brand-sub">
                CONTACT REQUESTS
            </div>

        </div>

    </a>

</header>


<section class="hero">

    <a href="/app/me"
       style="
           display:inline-flex;
           align-items:center;
           gap:8px;
           color:var(--muted);
           text-decoration:none;
           margin-bottom:20px;
       ">

        {back}

        <span>
            Профиль
        </span>

    </a>


    <div class="eyebrow">
        {user_icon}
        Связи
    </div>

    <h1>
        Запросы на связь
    </h1>

    <p>
        Здесь вы решаете,
        кто сможет начать общение с вами
        внутри ResursMap.
    </p>

</section>


<div class="card"
     style="
         display:flex;
         margin-bottom:20px;
         padding:15px 16px;
     ">

    <div class="card-content">

        <div class="card-title">
            Новые запросы
        </div>

        <div class="card-meta"
             style="margin-top:3px;">
            Ожидают вашего решения
        </div>

    </div>

    <div style="
        font-size:24px;
        font-weight:900;
    ">
        {pending_count}
    </div>

</div>


<section>

    {cards}

</section>


</main>


<nav class="bottom-nav">

    <a class="nav-item"
       href="/app">

        {nav_map}

        <span>
            Карта
        </span>

    </a>


    <a class="nav-item"
       href="/app/search">

        {nav_search}

        <span>
            Поиск
        </span>

    </a>


    <a class="nav-item active"
       href="/app/me">

        {nav_user}

        <span>
            Профиль
        </span>

    </a>


    <a class="nav-item"
       href="/app">

        {nav_menu}

        <span>
            Меню
        </span>

    </a>

</nav>


</body>

</html>"#,
        style = base_style(),
        logo = icon("user"),
        back = icon("arrow-left"),
        user_icon = icon("user"),
        pending_count = pending_count,
        cards = cards,
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
    )
}

// ============================================================
// TASK 7.22G-C — MESSAGES LIST
// ============================================================

pub fn render_messages(
    authenticated: bool,
    conversations: Vec<(i64, i64, String, String, String, String, i64, i64)>,
) -> String {
    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let total_unread: i64 = conversations.iter().map(|c| c.6).sum();

    let content = if !authenticated {
        r#"
<div class="card"
     style="
         display:block;
         padding:20px;
     ">

    <div class="card-title">
        Откройте ResursMap через Telegram
    </div>

    <div class="card-meta"
         style="
             margin-top:7px;
             line-height:1.5;
         ">
        После входа здесь появятся ваши личные сообщения.
    </div>

</div>
"#
        .to_string()
    } else if conversations.is_empty() {
        r#"
<div class="card"
     style="
         display:block;
         padding:20px;
     ">

    <div class="card-title">
        Диалогов пока нет
    </div>

    <div class="card-meta"
         style="
             margin-top:7px;
             line-height:1.5;
         ">
        После принятия запроса на связь здесь появится внутренний чат.
    </div>

</div>
"#
        .to_string()
    } else {
        conversations
            .iter()
            .map(
                |(
                    _conversation_id,
                    other_user_id,
                    username,
                    first_name,
                    last_name,
                    last_message,
                    unread_count,
                    updated_at,
                )| {
                    let safe_username = escape_html(username);

                    let safe_first_name = escape_html(first_name);

                    let safe_last_name = escape_html(last_name);

                    let safe_last_message = escape_html(last_message);

                    let full_name = format!("{} {}", safe_first_name, safe_last_name)
                        .trim()
                        .to_string();

                    let display_name = if !full_name.is_empty() {
                        full_name
                    } else if !safe_username.is_empty() {
                        format!("@{}", safe_username)
                    } else {
                        format!("Участник #{}", other_user_id)
                    };

                    let username_html = if !safe_username.is_empty() {
                        format!(
                            r#"
<div class="card-meta"
     style="
         margin-top:3px;
         overflow-wrap:anywhere;
     ">
    @{username}
</div>
"#,
                            username = safe_username,
                        )
                    } else {
                        String::new()
                    };

                    let has_last_message = !safe_last_message.is_empty();

                    let last_time = if has_last_message {
                        chrono::DateTime::<chrono::Utc>::from_timestamp(*updated_at, 0)
                            .map(|dt| {
                                dt.with_timezone(&chrono_tz::Europe::Paris)
                                    .format("%H:%M")
                                    .to_string()
                            })
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };

                    let last_message_html = if has_last_message {
                        safe_last_message
                    } else {
                        "Чат открыт. Сообщений пока нет.".to_string()
                    };

                    let unread_html = if *unread_count > 0 {
                        format!(
                            r#"
<div style="
    min-width:28px;
    height:28px;
    padding:0 8px;
    display:flex;
    align-items:center;
    justify-content:center;
    border-radius:999px;
    box-sizing:border-box;
    background:rgba(214,183,122,.16);
    border:1px solid rgba(214,183,122,.38);
    font-size:11px;
    font-weight:900;
">
    {count}
</div>
"#,
                            count = unread_count,
                        )
                    } else {
                        String::new()
                    };

                    format!(
                        r#"
<a href="/app/chat/{other_user_id}#chat-end"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:12px;
       align-items:flex-start;
   ">

    <div class="card-icon">
        {chat_icon}
    </div>

    <div class="card-content"
         style="
             min-width:0;
             flex:1;
         ">

        <div class="card-title"
             style="
                 font-size:16px;
                 overflow-wrap:anywhere;
             ">
            {display_name}
        </div>

        {username_html}

        <div class="card-meta"
             style="
                 margin-top:8px;
                 line-height:1.45;
                 overflow:hidden;
                 display:-webkit-box;
                 -webkit-line-clamp:2;
                 -webkit-box-orient:vertical;
             ">
            {last_message}
        </div>

    </div>

    <div style="
        display:flex;
        align-items:center;
        gap:9px;
        flex:0 0 auto;
    ">

        <div style="
            font-size:10px;
            color:var(--muted);
            white-space:nowrap;
        ">
            {last_time}
        </div>

        {unread_html}

        <div class="card-arrow">
            {arrow}
        </div>

    </div>

</a>
"#,
                        other_user_id = other_user_id,
                        chat_icon = icon("message-circle"),
                        display_name = display_name,
                        username_html = username_html,
                        last_message = last_message_html,
                        last_time = last_time,
                        unread_html = unread_html,
                        arrow = icon("chevron"),
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">

<head>

<meta charset="utf-8">

<meta name="viewport"
      content="width=device-width, initial-scale=1">

<title>Сообщения · ResursMap</title>

<style>{style}</style>

</head>

<body>

<main class="page">

<header class="topbar">

    <a class="brand"
       href="/app">

        <div class="brand-mark">
            {logo}
        </div>

        <div>
            <div class="brand-name">
                RESURSMAP
            </div>

            <div class="brand-sub">
                MESSAGES
            </div>
        </div>

    </a>

</header>


<section class="hero">

    <a href="/app/me"
       style="
           display:inline-flex;
           align-items:center;
           gap:8px;
           color:var(--muted);
           text-decoration:none;
           margin-bottom:20px;
       ">
        {back}
        Назад
    </a>

    <div class="eyebrow">
        {message_icon}
        Внутренняя связь
    </div>

    <h1>
        Сообщения
    </h1>

    <p>
        Ваши личные диалоги внутри ResursMap.
    </p>

</section>


<div class="section-head">

    <div>

        <h2 class="section-title">
            Диалоги
        </h2>

        <p class="section-caption">
            Непрочитанных: {total_unread}
        </p>

    </div>

</div>


<section>

    {content}

</section>


</main>

</body>

</html>"#,
        style = base_style(),
        logo = icon("search"),
        back = icon("chevron-left"),
        message_icon = icon("message-circle"),
        total_unread = total_unread,
        content = content,
    )
}

// ============================================================
// TASK 7.22F-E — CHAT
// ============================================================

pub fn render_chat(
    authenticated: bool,
    other_user_id: i64,
    username: &str,
    first_name: &str,
    last_name: &str,
    messages: Vec<(i64, i64, String, i64, i64)>,
) -> String {
    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let safe_username = escape_html(username);
    let safe_first_name = escape_html(first_name);
    let safe_last_name = escape_html(last_name);

    let full_name = format!("{} {}", safe_first_name, safe_last_name)
        .trim()
        .to_string();

    let display_name = if !full_name.is_empty() {
        full_name
    } else if !safe_username.is_empty() {
        format!("@{}", safe_username)
    } else if other_user_id > 0 {
        format!("Участник #{}", other_user_id)
    } else {
        "Чат ResursMap".to_string()
    };

    let subtitle = if !safe_username.is_empty() {
        format!("@{}", safe_username)
    } else {
        "Внутренний чат ResursMap".to_string()
    };

    let content = if !authenticated {
        r#"
<div class="card"
     style="
         display:block;
         padding:20px;
     ">

    <div class="card-title">
        Откройте ResursMap через Telegram
    </div>

    <div class="card-meta"
         style="
             margin-top:7px;
             line-height:1.5;
         ">
        Для доступа к внутренним сообщениям
        требуется подтверждённая сессия.
    </div>

</div>
"#
        .to_string()
    } else if messages.is_empty()
        && username.is_empty()
        && first_name.is_empty()
        && last_name.is_empty()
    {
        r#"
<div class="card"
     style="
         display:block;
         padding:20px;
     ">

    <div class="card-title">
        Чат недоступен
    </div>

    <div class="card-meta"
         style="
             margin-top:7px;
             line-height:1.5;
         ">
        Между этими пользователями ещё нет
        подтверждённого контакта.
    </div>

</div>
"#
        .to_string()
    } else {
        let message_cards = if messages.is_empty() {
            r#"
<div style="
    padding:30px 14px;
    text-align:center;
    color:var(--muted);
    font-size:13px;
    line-height:1.5;
">
    Чат открыт.<br>
    Напишите первое сообщение.
</div>
"#
            .to_string()
        } else {
            let mut last_date_key = String::new();

            messages
                .iter()
                .map(
                    |(_message_id, sender_user_id, message, is_read, created_at)| {
                        let safe_message = escape_html(message);

                        let mine = *sender_user_id != other_user_id;

                        let align = if mine { "flex-end" } else { "flex-start" };

                        let bubble_bg = if mine {
                            "rgba(214,183,122,.14)"
                        } else {
                            "rgba(255,255,255,.045)"
                        };

                        let border = if mine {
                            "rgba(214,183,122,.30)"
                        } else {
                            "rgba(255,255,255,.08)"
                        };

                        let status = if mine && *is_read != 0 {
                            "✓✓"
                        } else if mine {
                            "✓"
                        } else {
                            ""
                        };

                        // Время и дата полностью формируются Rust.
                        // Часовой пояс: Europe/Paris.
                        let paris = chrono_tz::Europe::Paris;

                        let datetime =
                            chrono::DateTime::<chrono::Utc>::from_timestamp(*created_at, 0)
                                .map(|dt| dt.with_timezone(&paris));

                        let chat_time = datetime
                            .as_ref()
                            .map(|dt| dt.format("%H:%M").to_string())
                            .unwrap_or_default();

                        let date_key = datetime
                            .as_ref()
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_default();

                        let date_label = if let Some(dt) = datetime.as_ref() {
                            let date = dt.date_naive();

                            let today = chrono::Utc::now().with_timezone(&paris).date_naive();

                            if date == today {
                                "Сегодня".to_string()
                            } else if date == today - chrono::Duration::days(1) {
                                "Вчера".to_string()
                            } else {
                                dt.format("%d.%m.%Y").to_string()
                            }
                        } else {
                            String::new()
                        };

                        let date_separator = if !date_key.is_empty() && date_key != last_date_key {
                            last_date_key = date_key.clone();

                            format!(
                                r#"
<div style="
    display:flex;
    justify-content:center;
    margin:16px 0 12px;
">
    <span style="
        padding:5px 11px;
        border-radius:999px;
        font-size:10px;
        font-weight:750;
        color:var(--muted);
        background:rgba(255,255,255,.045);
        border:1px solid rgba(255,255,255,.07);
    ">
        {}
    </span>
</div>
"#,
                                date_label
                            )
                        } else {
                            String::new()
                        };

                        format!(
                            r#"
{date_separator}

<div style="
    width:100%;
    display:flex;
    justify-content:{align};
    margin-bottom:10px;
">

    <div style="
        max-width:82%;
        min-width:70px;
        padding:11px 13px 8px;
        border-radius:16px;
        background:{bubble_bg};
        border:1px solid {border};
        box-sizing:border-box;
    ">

        <div style="
            font-size:14px;
            line-height:1.48;
            white-space:pre-wrap;
            overflow-wrap:anywhere;
            word-break:break-word;
        ">{message}</div>

        <div style="
            margin-top:5px;
            display:flex;
            justify-content:flex-end;
            gap:6px;
            font-size:9px;
            color:var(--muted);
        ">
            <span>{chat_time}</span>
            <span>{status}</span>
        </div>

    </div>

</div>
"#,
                            align = align,
                            bubble_bg = bubble_bg,
                            border = border,
                            message = safe_message,
                            date_separator = date_separator,
                            chat_time = chat_time,
                            status = status,
                        )
                    },
                )
                .collect::<Vec<_>>()
                .join("")
        };

        format!(
            r#"
<section class="card"
         style="
             display:block;
             padding:14px;
             margin-bottom:14px;
         ">

    <div id="chat-messages"
         style="
             max-height:58vh;
             overflow-y:auto;
             padding:4px 1px;
         ">
        {message_cards}
        <div id="chat-end"></div>
    </div>

</section>

<form method="post"
      action="/app/chat/{other_user_id}/send"
      style="
          position:sticky;
          bottom:10px;
          display:flex;
          gap:9px;
          align-items:flex-end;
          padding:10px;
          border-radius:18px;
          background:rgba(10,12,15,.95);
          border:1px solid rgba(255,255,255,.08);
          backdrop-filter:blur(14px);
      ">

    <textarea
        name="message"
        rows="1"
        maxlength="2000"
        required
        placeholder="Сообщение..."
        style="
            flex:1;
            min-height:46px;
            max-height:130px;
            resize:vertical;
            box-sizing:border-box;
            border-radius:14px;
            border:1px solid rgba(255,255,255,.10);
            background:rgba(255,255,255,.04);
            color:var(--text);
            padding:12px 13px;
            font:inherit;
            line-height:1.4;
            outline:none;
        "
    ></textarea>

    <button type="submit"
            style="
                width:48px;
                height:48px;
                border-radius:14px;
                border:1px solid rgba(214,183,122,.40);
                background:rgba(214,183,122,.13);
                color:var(--text);
                font-size:20px;
                font-weight:900;
                cursor:pointer;
                flex:0 0 auto;
            ">
        ➤
    </button>

</form>
"#,
            other_user_id = other_user_id,
            message_cards = message_cards,
        )
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">

<head>
<meta charset="utf-8">
<meta name="viewport"
      content="width=device-width, initial-scale=1">

<title>Чат · ResursMap</title>

<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">

    <a class="brand"
       href="/app">

        <div class="brand-mark">
            {logo}
        </div>

        <div>
            <div class="brand-name">
                RESURSMAP
            </div>

            <div class="brand-sub">
                CHAT
            </div>
        </div>

    </a>

</header>

<section class="hero"
         style="
             padding-bottom:14px;
         ">

    <a href="/app/messages"
       style="
           display:inline-flex;
           align-items:center;
           gap:7px;
           color:var(--muted);
           text-decoration:none;
           margin-bottom:18px;
       ">
        {back}
        Назад
    </a>

    <div style="
        display:flex;
        align-items:center;
        gap:13px;
    ">

        <div class="card-icon"
             style="
                 width:48px;
                 height:48px;
                 flex:0 0 48px;
             ">
            {user_icon}
        </div>

        <div style="min-width:0;">

            <h1 style="
                margin:0;
                font-size:22px;
                overflow-wrap:anywhere;
            ">
                {display_name}
            </h1>

            <div style="
                margin-top:4px;
                color:var(--muted);
                font-size:12px;
                overflow-wrap:anywhere;
            ">
                {subtitle}
            </div>

        </div>

    </div>

</section>

{content}

</main>






</body>
</html>"#,
        style = base_style(),
        logo = icon("search"),
        back = icon("chevron"),
        user_icon = icon("user"),
        display_name = display_name,
        subtitle = subtitle,
        content = content,
    )
}

// ============================================================
// PROFILE
// ============================================================

pub fn render_me(
    authenticated: bool,
    user_id: i64,
    username: &str,
    first_name: &str,
    last_name: &str,
    resources_count: i64,
    approved_count: i64,
    pending_count: i64,
    rejected_count: i64,
    favorites_count: i64,
    unread_notifications_count: i64,
    pending_contact_requests_count: i64,
    unread_messages_count: i64,
    open_contact: bool,
    intent_text: &str,
    intent_until: i64,
) -> String {
    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let safe_username = escape_html(username);
    let safe_first_name = escape_html(first_name);
    let safe_last_name = escape_html(last_name);
    let safe_intent_text = escape_html(intent_text);

    let unread_messages_badge = if unread_messages_count > 0 {
        format!(
            r#"<span style="
                    min-width:28px;
                    height:28px;
                    padding:0 8px;
                    border-radius:999px;
                    display:flex;
                    align-items:center;
                    justify-content:center;
                    box-sizing:border-box;
                    font-size:12px;
                    font-weight:900;
                    background:rgba(214,183,122,.16);
                    border:1px solid rgba(214,183,122,.38);
                ">{}</span>"#,
            unread_messages_count
        )
    } else {
        String::new()
    };

    let contact_checked = if open_contact { "checked" } else { "" };

    let intent_status_text = if safe_intent_text.is_empty() {
        "Статус пока не указан".to_string()
    } else if intent_until > 0 {
        format!("{}", safe_intent_text)
    } else {
        safe_intent_text.clone()
    };

    let full_name = format!("{} {}", safe_first_name, safe_last_name,)
        .trim()
        .to_string();

    let display_name = if !full_name.is_empty() {
        full_name
    } else if !safe_username.is_empty() {
        format!("@{}", safe_username)
    } else if authenticated {
        "Пользователь Telegram".to_string()
    } else {
        "Гость".to_string()
    };

    let username_html = if !safe_username.is_empty() {
        format!(
            r#"<div style="
                margin-top:5px;
                color:var(--muted);
                font-size:14px;
            ">@{}</div>"#,
            safe_username
        )
    } else if authenticated {
        r#"<div style="
            margin-top:5px;
            color:var(--muted);
            font-size:13px;
        ">Telegram аккаунт</div>"#
            .to_string()
    } else {
        String::new()
    };

    let telegram_id_html = if authenticated {
        format!(
            r#"<div style="
                margin-top:10px;
                font-size:11px;
                color:var(--muted);
                letter-spacing:.04em;
            ">
                TELEGRAM ID · {}
            </div>"#,
            user_id
        )
    } else {
        String::new()
    };

    let account_header = if authenticated {
        format!(
            r#"
<div class="card"
     style="
         display:block;
         margin-bottom:20px;
         padding:20px;
         border:1px solid rgba(214,183,122,.22);
     ">

    <div style="
        display:flex;
        align-items:center;
        gap:15px;
    ">

        <div style="
            width:58px;
            height:58px;
            border-radius:18px;
            display:flex;
            align-items:center;
            justify-content:center;
            flex:0 0 auto;
            background:rgba(214,183,122,.10);
            border:1px solid rgba(214,183,122,.28);
        ">
            {user_icon}
        </div>

        <div style="min-width:0;">

            <div style="
                font-size:21px;
                line-height:1.2;
                font-weight:850;
                overflow-wrap:anywhere;
            ">
                {display_name}
            </div>

            {username_html}

            {telegram_id_html}

        </div>

    </div>

</div>
"#,
            user_icon = icon("user"),
            display_name = display_name,
            username_html = username_html,
            telegram_id_html = telegram_id_html,
        )
    } else {
        r#"
<div class="card"
     style="
         display:block;
         margin-bottom:20px;
         padding:20px;
     ">

    <div class="card-title">
        Откройте ResursMap через Telegram
    </div>

    <div class="card-meta"
         style="
             margin-top:7px;
             line-height:1.5;
         ">
        После входа здесь появятся ваш профиль,
        ресурсы, избранное и статистика.
    </div>

</div>
"#
        .to_string()
    };

    let statistics = if authenticated {
        format!(
            r#"
<div style="
    display:grid;
    grid-template-columns:repeat(2,minmax(0,1fr));
    gap:10px;
    margin-bottom:24px;
">

    <div class="card"
         style="display:block;padding:16px;">
        <div style="
            font-size:26px;
            font-weight:900;
            line-height:1;
        ">
            {resources_count}
        </div>
        <div class="card-meta"
             style="margin-top:7px;">
            Мои ресурсы
        </div>
    </div>

    <div class="card"
         style="display:block;padding:16px;">
        <div style="
            font-size:26px;
            font-weight:900;
            line-height:1;
        ">
            {favorites_count}
        </div>
        <div class="card-meta"
             style="margin-top:7px;">
            Избранное
        </div>
    </div>

    <div class="card"
         style="display:block;padding:16px;">
        <div style="
            font-size:24px;
            font-weight:900;
            line-height:1;
            color:#16a34a;
        ">
            {approved_count}
        </div>
        <div class="card-meta"
             style="margin-top:7px;">
            Одобрено
        </div>
    </div>

    <div class="card"
         style="display:block;padding:16px;">
        <div style="
            font-size:24px;
            font-weight:900;
            line-height:1;
            color:#d97706;
        ">
            {pending_count}
        </div>
        <div class="card-meta"
             style="margin-top:7px;">
            На проверке
        </div>
    </div>

</div>

<div class="card"
     style="
         display:flex;
         margin-bottom:20px;
         padding:14px 16px;
         border:1px solid rgba(220,38,38,.14);
     ">

    <div class="card-content">
        <div class="card-title"
             style="font-size:14px;">
            Отклонено
        </div>

        <div class="card-meta"
             style="margin-top:3px;">
            Ресурсы, которым требуется исправление
        </div>
    </div>

    <div style="
        font-size:22px;
        font-weight:900;
        color:#dc2626;
    ">
        {rejected_count}
    </div>

</div>
"#,
            resources_count = resources_count,
            favorites_count = favorites_count,
            approved_count = approved_count,
            pending_count = pending_count,
            rejected_count = rejected_count,
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">

<head>
<meta charset="utf-8">
<meta name="viewport"
      content="width=device-width, initial-scale=1">

<script src="https://telegram.org/js/telegram-web-app.js"></script>

<title>Профиль · ResursMap</title>

<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">

    <a class="brand"
       href="/app">

        <div class="brand-mark">
            {logo}
        </div>

        <div>
            <div class="brand-name">
                RESURSMAP
            </div>

            <div class="brand-sub">
                PROFILE
            </div>
        </div>

    </a>

</header>


<section class="hero">

    <div class="eyebrow">
        {profile_icon}
        Личный кабинет
    </div>

    <h1>Мой профиль</h1>

    <p>
        Ваши ресурсы, сохранённые места
        и активность в ResursMap.
    </p>

</section>


{account_header}


{statistics}


<section class="card"
         style="
             display:block;
             padding:20px;
             margin-bottom:24px;
         ">

    <div style="
        display:flex;
        align-items:flex-start;
        justify-content:space-between;
        gap:14px;
        margin-bottom:18px;
    ">

        <div>
            <div class="card-title"
                 style="font-size:18px;">
                Мой статус
            </div>

            <div class="card-meta"
                 style="
                     margin-top:5px;
                     line-height:1.45;
                 ">
                Расскажите сообществу, что вы ищете
                или что можете предложить.
            </div>
        </div>

        <div style="
            width:42px;
            height:42px;
            border-radius:13px;
            display:flex;
            align-items:center;
            justify-content:center;
            flex:0 0 auto;
            background:rgba(214,183,122,.10);
            border:1px solid rgba(214,183,122,.25);
        ">
            {settings_icon}
        </div>

    </div>


    <div style="
        padding:12px 14px;
        border-radius:13px;
        background:rgba(255,255,255,.03);
        border:1px solid rgba(255,255,255,.07);
        margin-bottom:16px;
    ">

        <div style="
            font-size:11px;
            color:var(--muted);
            text-transform:uppercase;
            letter-spacing:.06em;
            font-weight:800;
            margin-bottom:5px;
        ">
            Сейчас
        </div>

        <div id="intent-current"
             style="
                 font-size:14px;
                 line-height:1.5;
                 overflow-wrap:anywhere;
             ">
            {intent_status_text}
        </div>

    </div>


    <label style="
        display:flex;
        align-items:center;
        gap:11px;
        margin-bottom:17px;
        cursor:pointer;
    ">

        <input
            id="profile-open-contact"
            type="checkbox"
            {contact_checked}
            style="
                width:20px;
                height:20px;
                flex:0 0 auto;
            "
        >

        <div>
            <div style="
                font-size:14px;
                font-weight:800;
            ">
                Показывать мой контакт
            </div>

            <div style="
                font-size:12px;
                color:var(--muted);
                margin-top:3px;
                line-height:1.4;
            ">
                Другие пользователи смогут понять,
                что вы открыты для связи.
            </div>
        </div>

    </label>


    <label style="display:block;">

        <div style="
            margin-bottom:7px;
            font-size:13px;
            font-weight:800;
        ">
            Что вы ищете или предлагаете
        </div>

        <textarea
            id="profile-intent"
            maxlength="300"
            rows="4"
            placeholder="Например: ищу электрика в Ницце или предлагаю грузоперевозки..."
            style="
                width:100%;
                box-sizing:border-box;
                resize:vertical;
                padding:14px;
                border-radius:14px;
                border:1px solid rgba(255,255,255,.12);
                background:rgba(255,255,255,.035);
                color:var(--text);
                font:inherit;
                line-height:1.5;
            "
        >{safe_intent_text}</textarea>

    </label>


    <label style="
        display:block;
        margin-top:15px;
    ">

        <div style="
            margin-bottom:7px;
            font-size:13px;
            font-weight:800;
        ">
            Срок актуальности
        </div>

        <select
            id="profile-duration"
            style="
                width:100%;
                min-height:48px;
                box-sizing:border-box;
                padding:0 13px;
                border-radius:14px;
                border:1px solid rgba(255,255,255,.12);
                background:rgba(255,255,255,.035);
                color:var(--text);
                font:inherit;
            "
        >
            <option value="0">
                Без срока
            </option>

            <option value="1">
                1 день
            </option>

            <option value="3">
                3 дня
            </option>

            <option value="7" selected>
                7 дней
            </option>

            <option value="30">
                30 дней
            </option>
        </select>

    </label>


    <button
        id="profile-save"
        type="button"
        style="
            width:100%;
            min-height:50px;
            margin-top:17px;
            border:0;
            border-radius:15px;
            cursor:pointer;
            font-size:15px;
            font-weight:900;
            color:#111;
            background:linear-gradient(
                135deg,
                #d6b77a,
                #b88932
            );
        "
    >
        Сохранить статус
    </button>


    <div id="profile-save-status"
         style="
             min-height:18px;
             margin-top:9px;
             font-size:12px;
             color:var(--muted);
             line-height:1.4;
         ">
    </div>

</section>


<div class="section-head">

    <div>
        <h2 class="section-title">
            Ваш ResursMap
        </h2>

        <p class="section-caption">
            Управление аккаунтом
        </p>
    </div>

</div>


<div style="
    display:grid;
    gap:12px;
">

<a class="card"
   href="/app/contact-requests"
   style="
       text-decoration:none;
       color:inherit;
   ">

    <div class="card-icon">
        👥
    </div>

    <div class="card-content">

        <div class="card-title">
            Запросы на связь
        </div>

        <div class="card-meta">
            Входящие запросы от участников ResursMap
        </div>

    </div>

    <div style="
        display:flex;
        align-items:center;
        gap:10px;
    ">

        <span style="
            min-width:28px;
            height:28px;
            padding:0 8px;
            border-radius:999px;
            display:flex;
            align-items:center;
            justify-content:center;
            box-sizing:border-box;
            font-size:12px;
            font-weight:900;
            background:rgba(214,183,122,.12);
            border:1px solid rgba(214,183,122,.30);
        ">
            {pending_contact_requests_count}
        </span>

        <div class="card-arrow">
            {notification_arrow}
        </div>

    </div>

</a>



<a class="card"
   href="/app/messages"
   style="
       text-decoration:none;
       color:inherit;
   ">

    <div class="card-icon">
        💬
    </div>

    <div class="card-content">

        <div class="card-title">
            Сообщения
        </div>

        <div class="card-meta">
            Личные диалоги внутри ResursMap
        </div>

    </div>

    <div style="
        display:flex;
        align-items:center;
        gap:10px;
    ">
        {unread_messages_badge}

        <div class="card-arrow">
            ›
        </div>
    </div>

</a>


<a class="card"
   href="/app/notifications"
   style="
       text-decoration:none;
       color:inherit;
   ">

    <div class="card-icon">
        🔔
    </div>

    <div class="card-content">

        <div class="card-title">
            Уведомления
        </div>

        <div class="card-meta">
            Модерация и изменения ваших ресурсов
        </div>

    </div>

    <div style="
        display:flex;
        align-items:center;
        gap:10px;
    ">

        <span style="
            min-width:28px;
            height:28px;
            padding:0 8px;
            border-radius:999px;
            display:flex;
            align-items:center;
            justify-content:center;
            box-sizing:border-box;
            font-size:12px;
            font-weight:900;
            background:rgba(214,183,122,.12);
            border:1px solid rgba(214,183,122,.30);
        ">
            {unread_notifications_count}
        </span>

        <div class="card-arrow">
            {notification_arrow}
        </div>

    </div>

</a>


<a id="my-resources-link"
   class="card"
   href="/app/my-resources"
   style="
       text-decoration:none;
       color:inherit;
   ">

    <div class="card-icon">
        {resource_icon}
    </div>

    <div class="card-content">

        <div class="card-title">
            Мои ресурсы
        </div>

        <div class="card-meta">
            Публикации, статусы и редактирование
        </div>

    </div>

    <div class="card-arrow">
        {arrow1}
    </div>

</a>


<a class="card"
   href="/app/favorites"
   style="
       text-decoration:none;
       color:inherit;
   ">

    <div class="card-icon">
        {heart}
    </div>

    <div class="card-content">

        <div class="card-title">
            Избранное
        </div>

        <div class="card-meta">
            Сохранённые ресурсы
        </div>

    </div>

    <div class="card-arrow">
        {arrow2}
    </div>

</a>


<a class="card"
   href="/app/search"
   style="
       text-decoration:none;
       color:inherit;
   ">

    <div class="card-icon">
        {search_icon}
    </div>

    <div class="card-content">

        <div class="card-title">
            Найти ресурс
        </div>

        <div class="card-meta">
            Глобальный поиск по ResursMap
        </div>

    </div>

    <div class="card-arrow">
        {arrow3}
    </div>

</a>


</div>


</main>


<nav class="bottom-nav">

    <a class="nav-item"
       href="/app">
        {nav_map}
        <span>Карта</span>
    </a>

    <a class="nav-item"
       href="/app/search">
        {nav_search}
        <span>Поиск</span>
    </a>

    <a class="nav-item active"
       href="/app/me">
        {nav_user}
        <span>Профиль</span>
    </a>

    <a class="nav-item"
       href="/app">
        {nav_menu}
        <span>Меню</span>
    </a>

</nav>


<script>
(function() {{
    const saveButton =
        document.getElementById("profile-save");

    const openContact =
        document.getElementById("profile-open-contact");

    const intent =
        document.getElementById("profile-intent");

    const duration =
        document.getElementById("profile-duration");

    const status =
        document.getElementById("profile-save-status");

    const current =
        document.getElementById("intent-current");


    if (
        saveButton &&
        openContact &&
        intent &&
        duration
    ) {{
        saveButton.addEventListener(
            "click",
            async () => {{
                saveButton.disabled = true;

                if (status) {{
                    status.textContent =
                        "Сохраняем...";
                }}

                try {{
                    const response =
                        await fetch(
                            "/api/profile",
                            {{
                                method: "POST",

                                headers: {{
                                    "Content-Type":
                                        "application/json"
                                }},

                                body: JSON.stringify({{
                                    open_contact:
                                        openContact.checked,

                                    intent_text:
                                        intent.value.trim(),

                                    duration_days:
                                        Number(
                                            duration.value
                                        )
                                }})
                            }}
                        );

                    const data =
                        await response.json();

                    if (
                        response.status === 401
                    ) {{
                        if (status) {{
                            status.textContent =
                                "Откройте ResursMap через Telegram.";
                        }}

                        saveButton.disabled = false;
                        return;
                    }}

                    if (!data.ok) {{
                        if (status) {{
                            status.textContent =
                                "Не удалось сохранить.";
                        }}

                        saveButton.disabled = false;
                        return;
                    }}

                    if (current) {{
                        current.textContent =
                            data.intent_text ||
                            "Статус пока не указан";
                    }}

                    if (status) {{
                        status.textContent =
                            "✓ Статус сохранён";
                    }}

                }} catch (_) {{
                    if (status) {{
                        status.textContent =
                            "Ошибка соединения.";
                    }}
                }}

                saveButton.disabled = false;
            }}
        );
    }}


}})();
</script>


</body>
</html>"#,
        style = base_style(),
        logo = icon("user"),
        profile_icon = icon("user"),
        account_header = account_header,
        statistics = statistics,
        settings_icon = icon("user"),
        intent_status_text = intent_status_text,
        safe_intent_text = safe_intent_text,
        contact_checked = contact_checked,
        unread_notifications_count = unread_notifications_count,
        pending_contact_requests_count = pending_contact_requests_count,
        unread_messages_badge = unread_messages_badge,
        notification_arrow = icon("chevron"),
        resource_icon = icon("map-pin"),
        heart = icon("heart"),
        search_icon = icon("search"),
        arrow1 = icon("chevron"),
        arrow2 = icon("chevron"),
        arrow3 = icon("chevron"),
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
    )
}

// ============================================================
// USER NOTIFICATIONS
// ============================================================

pub fn render_notifications(
    notifications: Vec<(i64, Option<i64>, String, String, String, i64, i64)>,
    authenticated: bool,
) -> String {
    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let cards = if !authenticated {
        r#"
        <div class="card"
             style="display:block;padding:20px;">

            <div class="card-title">
                Откройте ResursMap через Telegram
            </div>

            <div class="card-meta"
                 style="margin-top:7px;line-height:1.5;">
                Уведомления доступны после подтверждения Telegram.
            </div>

        </div>
        "#
        .to_string()
    } else if notifications.is_empty() {
        r#"
        <div class="card"
             style="display:block;padding:20px;">

            <div style="
                font-size:30px;
                margin-bottom:12px;
            ">
                🔔
            </div>

            <div class="card-title">
                Уведомлений пока нет
            </div>

            <div class="card-meta"
                 style="margin-top:7px;line-height:1.5;">
                Здесь появятся результаты модерации
                и важные изменения ваших ресурсов.
            </div>

        </div>
        "#
        .to_string()
    } else {
        notifications
            .iter()
            .map(
                |(_notification_id, resource_id, kind, title, message, is_read, _created_at)| {
                    let safe_title = escape_html(title);

                    let safe_message = escape_html(message);

                    let (icon_text, accent) = match kind.as_str() {
                        "resource_approved" => ("✓", "#16a34a"),

                        "resource_rejected" => ("!", "#dc2626"),

                        "chat_message" => ("💬", "#d6b77a"),

                        "contact_accepted" => ("✓", "#16a34a"),

                        "contact_rejected" => ("×", "#dc2626"),

                        _ => ("🔔", "#b88932"),
                    };

                    let unread_badge = if *is_read == 0 {
                        r#"
                            <span style="
                                font-size:10px;
                                font-weight:900;
                                color:#d6b77a;
                                text-transform:uppercase;
                                letter-spacing:.06em;
                            ">
                                Новое
                            </span>
                            "#
                    } else {
                        ""
                    };

                    let open_link = if kind == "chat_message" || kind == "contact_accepted" {
                        r#"
                            <a href="/app/messages"
                               style="
                                   display:inline-flex;
                                   align-items:center;
                                   min-height:40px;
                                   padding:0 13px;
                                   border-radius:12px;
                                   text-decoration:none;
                                   color:var(--text);
                                   font-size:13px;
                                   font-weight:800;
                                   border:1px solid rgba(214,183,122,.28);
                                   background:rgba(214,183,122,.07);
                                   margin-top:13px;
                               ">
                                Открыть сообщения
                            </a>
                            "#
                        .to_string()
                    } else if let Some(id) = resource_id {
                        format!(
                            r#"
                                <a href="/app/resource/{id}"
                                   style="
                                       display:inline-flex;
                                       align-items:center;
                                       min-height:40px;
                                       padding:0 13px;
                                       border-radius:12px;
                                       text-decoration:none;
                                       color:var(--text);
                                       font-size:13px;
                                       font-weight:800;
                                       border:1px solid rgba(255,255,255,.10);
                                       background:rgba(255,255,255,.03);
                                       margin-top:13px;
                                   ">
                                    Открыть ресурс
                                </a>
                                "#,
                            id = id
                        )
                    } else {
                        String::new()
                    };

                    format!(
                        r#"
<article class="card"
         style="
             display:block;
             padding:18px;
             margin-bottom:14px;
             border-left:3px solid {accent};
         ">

    <div style="
        display:flex;
        gap:13px;
        align-items:flex-start;
    ">

        <div style="
            width:42px;
            height:42px;
            border-radius:13px;
            display:flex;
            align-items:center;
            justify-content:center;
            flex:0 0 auto;
            font-size:19px;
            font-weight:900;
            color:{accent};
            border:1px solid rgba(214,183,122,.20);
            background:rgba(255,255,255,.03);
        ">
            {icon_text}
        </div>

        <div style="
            min-width:0;
            flex:1;
        ">

            <div style="
                display:flex;
                justify-content:space-between;
                align-items:flex-start;
                gap:10px;
            ">

                <div class="card-title">
                    {title}
                </div>

                {unread_badge}

            </div>

            <div class="card-meta"
                 style="
                     margin-top:7px;
                     line-height:1.55;
                     overflow-wrap:anywhere;
                 ">
                {message}
            </div>

            {open_link}

        </div>

    </div>

</article>
"#,
                        accent = accent,
                        icon_text = icon_text,
                        title = safe_title,
                        message = safe_message,
                        unread_badge = unread_badge,
                        open_link = open_link,
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">

<head>

<meta charset="utf-8">

<meta name="viewport"
      content="width=device-width, initial-scale=1">

<title>Уведомления · ResursMap</title>

<style>{style}</style>

</head>


<body>

<main class="page">


<header class="topbar">

    <a class="brand"
       href="/app">

        <div class="brand-mark">
            {logo}
        </div>

        <div>
            <div class="brand-name">
                RESURSMAP
            </div>

            <div class="brand-sub">
                NOTIFICATIONS
            </div>
        </div>

    </a>

</header>


<section class="hero">

    <a href="/app/me"
       style="
           display:inline-flex;
           align-items:center;
           gap:8px;
           color:var(--muted);
           text-decoration:none;
           margin-bottom:20px;
       ">
        {back}
        <span>Профиль</span>
    </a>


    <div class="eyebrow">
        🔔
        Уведомления
    </div>


    <h1>
        Центр уведомлений
    </h1>


    <p>
        Статусы модерации и важные изменения
        ваших ресурсов.
    </p>

</section>


<section>
    {cards}
</section>


</main>


<nav class="bottom-nav">

    <a class="nav-item"
       href="/app">
        {nav_map}
        <span>Карта</span>
    </a>

    <a class="nav-item"
       href="/app/search">
        {nav_search}
        <span>Поиск</span>
    </a>

    <a class="nav-item active"
       href="/app/me">
        {nav_user}
        <span>Профиль</span>
    </a>

    <a class="nav-item"
       href="/app">
        {nav_menu}
        <span>Меню</span>
    </a>

</nav>


</body>
</html>"#,
        style = base_style(),
        logo = icon("user"),
        back = icon("arrow-left"),
        cards = cards,
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
    )
}

// ============================================================
// CATEGORY
// ============================================================

pub fn render_category(
    ci: usize,
    si: usize,
    zi: usize,
    category: &str,
    resources: Vec<(i64, String, String, String, String, f64, i64, i64, i64)>,
) -> String {
    let city_url = format!("/app/{}/{}/{}", ci, si, zi);

    let cards = if resources.is_empty() {
        format!(
            r#"
        <a class="feature"
           href="/app/{}/{}/{}/cat/{}/add"
           style="text-decoration:none;color:inherit;">
            <div class="card-icon">+</div>
            <strong>Пока ресурсов нет</strong>
            <span>Будьте первым — добавьте ресурс в эту категорию.</span>
        </a>
        "#,
            ci, si, zi, category
        )
    } else {
        resources
            .iter()
            .map(|(id, title, description, contact, address, rating, votes, verified, premium)| {
                let verified_badge = if *verified != 0 {
                    r#"<span style="
                        color:#16a34a;
                        font-size:12px;
                        font-weight:700;
                    ">✓ Проверен</span>"#
                } else {
                    ""
                };

                let premium_badge = if *premium != 0 {
                    r#"<span style="
                        display:inline-flex;
                        align-items:center;
                        gap:5px;
                        padding:4px 9px;
                        border-radius:999px;
                        background:rgba(214,183,122,.12);
                        border:1px solid rgba(214,183,122,.45);
                        color:#b88932;
                        font-size:10px;
                        font-weight:800;
                        letter-spacing:.10em;
                    ">★ PREMIUM</span>"#
                } else {
                    ""
                };

                let card_style = if *premium != 0 {
                    "margin-bottom:16px;                    border:1px solid rgba(214,183,122,.55);                    background:linear-gradient(145deg,rgba(255,255,255,1),rgba(250,246,238,.98));                    box-shadow:0 10px 32px rgba(214,183,122,.14),0 0 0 1px rgba(214,183,122,.06);                    position:relative;                    overflow:hidden;"
                } else {
                    "margin-bottom:14px;"
                };

                format!(
                    r#"
                    <a href="/app/resource/{}" class="card" style="{}text-decoration:none;color:inherit;">
                        {}
                        <div class="card-icon">{}</div>

                        <div class="card-content">

                            <div style="
                                display:flex;
                                align-items:center;
                                gap:8px;
                                flex-wrap:wrap;
                                margin-bottom:6px;
                            ">
                                <div class="card-title">
                                    {}
                                </div>

                                {}
                            </div>

                            <div class="card-meta">
                                {}
                            </div>

                            <div class="card-meta">
                                ⭐ {:.1} · {} голосов
                            </div>

                            <div class="card-meta">
                                📍 {}
                            </div>

                            <div class="card-meta">
                                📞 {}
                            </div>

                            <div style="margin-top:6px;">
                                {}
                            </div>

                        </div>

                        <div class="card-arrow">›</div>
                    </a>
                    "#,
                    id,
                    card_style,
                    if *premium != 0 {
                        r#"<div style="position:absolute;top:0;left:0;right:0;height:2px;background:linear-gradient(90deg,transparent,#d6b77a,transparent);"></div>"#
                    } else {
                        ""
                    },
                    icon("map-pin"),
                    title,
                    premium_badge,
                    description,
                    rating,
                    votes,
                    address,
                    contact,
                    verified_badge
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let count = resources.len();

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">

<title>{category} · ResursMap</title>

<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>

        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">

    <a href="{city_url}"
       style="display:inline-flex;align-items:center;gap:8px;
              color:var(--muted);text-decoration:none;margin-bottom:20px;">
        {back}
        <span>Вернуться к городу</span>
    </a>

    <div class="eyebrow">
        {category_icon}
        Категория
    </div>

    <h1>{category}</h1>

    <p>
        Ресурсы города в выбранной категории.
    </p>

</section>

<div class="section-head">

    <div>
        <h2 class="section-title">Ресурсы</h2>
        <p class="section-caption">
            Найдено: {count}
        </p>
    </div>

</div>

<div>
    {cards}
</div>

<div class="section-head" style="margin-top:34px;">

    <div>
        <h2 class="section-title">Ваш город</h2>

        <p class="section-caption">
            Все направления и категории
        </p>
    </div>

</div>

<a class="card"
   href="{city_url}"
   style="text-decoration:none;">

    <div class="card-icon">
        {city_icon}
    </div>

    <div class="card-content">

        <div class="card-title">
            Открыть карту города
        </div>

        <div class="card-meta">
            Вернуться ко всем категориям
        </div>

    </div>

    <div class="card-arrow">
        ›
    </div>

</a>

</main>

<nav class="bottom-nav">

    <a class="nav-item active" href="/app">
        {nav_map}
        <span>Карта</span>
    </a>

    <a class="nav-item" href="/app/search">
        {nav_search}
        <span>Поиск</span>
    </a>

    <a class="nav-item" href="/app/me">
        {nav_user}
        <span>Профиль</span>
    </a>

    <a class="nav-item" href="/app">
        {nav_menu}
        <span>Меню</span>
    </a>

</nav>

</body>
</html>"#,
        style = base_style(),
        logo = icon("globe"),
        back = icon("chevron"),
        category_icon = icon("map"),
        city_icon = icon("map"),
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
        category = category,
        city_url = city_url,
        count = count,
        cards = cards,
    )
}

// ============================================================
// ADD RESOURCE
// ============================================================

// ============================================================
// PUBLIC USER PROFILE
// ============================================================

pub fn render_public_user_profile(
    public_id: &str,
    username: &str,
    first_name: &str,
    last_name: &str,
    open_contact: bool,
    intent_text: &str,
    chat_user_id: Option<i64>,
    resources: Vec<(i64, String, String, String, f64, i64, i64, i64)>,
) -> String {
    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    let safe_username = escape_html(username);
    let safe_first_name = escape_html(first_name);
    let safe_last_name = escape_html(last_name);
    let safe_intent = escape_html(intent_text);

    let full_name = format!("{} {}", safe_first_name, safe_last_name)
        .trim()
        .to_string();

    let display_name = if !full_name.is_empty() {
        full_name
    } else if !safe_username.is_empty() {
        format!("@{}", safe_username)
    } else {
        "Участник ResursMap".to_string()
    };

    let contact_html = if open_contact && !safe_username.is_empty() {
        format!(
            r#"
<a href="https://t.me/{username}"
   target="_blank"
   rel="noopener noreferrer"
   style="
       margin-top:14px;
       min-height:46px;
       display:inline-flex;
       align-items:center;
       justify-content:center;
       padding:0 16px;
       border-radius:14px;
       text-decoration:none;
       color:var(--text);
       font-weight:850;
       border:1px solid rgba(214,183,122,.38);
       background:rgba(214,183,122,.10);
   ">
    Telegram · @{username}
</a>
"#,
            username = safe_username
        )
    } else {
        r#"
<div style="
    margin-top:12px;
    font-size:12px;
    color:var(--muted);
">
    Контакт пользователя закрыт.
</div>
"#
        .to_string()
    };

    let safe_public_id = escape_html(public_id);

    let internal_contact_html = if let Some(chat_user_id) = chat_user_id {
        format!(
            r#"
<section class="card"
         style="
             display:block;
             padding:20px;
             margin-bottom:18px;
         ">

    <div style="
        font-size:11px;
        color:var(--muted);
        text-transform:uppercase;
        letter-spacing:.07em;
        font-weight:800;
        margin-bottom:8px;
    ">
        Связаться
    </div>

    <div class="card-meta"
         style="
             line-height:1.5;
             margin-bottom:14px;
         ">
        Контакт установлен. Вы можете продолжить общение
        во внутреннем чате ResursMap.
    </div>

    <a href="/app/chat/{chat_user_id}"
       style="
           width:100%;
           min-height:48px;
           box-sizing:border-box;
           display:flex;
           align-items:center;
           justify-content:center;
           border-radius:14px;
           border:1px solid rgba(22,163,74,.38);
           background:rgba(22,163,74,.10);
           color:var(--text);
           text-decoration:none;
           font-weight:850;
       ">
        Открыть чат
    </a>

</section>
"#,
            chat_user_id = chat_user_id,
        )
    } else {
        format!(
            r#"
<section class="card"
         style="
             display:block;
             padding:20px;
             margin-bottom:18px;
         ">

    <div style="
        font-size:11px;
        color:var(--muted);
        text-transform:uppercase;
        letter-spacing:.07em;
        font-weight:800;
        margin-bottom:8px;
    ">
        Связаться
    </div>

    <div class="card-meta"
         style="
             line-height:1.5;
             margin-bottom:14px;
         ">
        Отправьте запрос через ResursMap.
    </div>

    <button
        id="contact-request-open"
        type="button"
        style="
            width:100%;
            min-height:48px;
            border-radius:14px;
            border:1px solid rgba(214,183,122,.38);
            background:rgba(214,183,122,.10);
            color:var(--text);
            font-weight:850;
            cursor:pointer;
        ">
        Связаться через ResursMap
    </button>

    <div id="contact-request-panel"
         style="
             display:none;
             margin-top:14px;
         ">

        <textarea
            id="contact-request-message"
            maxlength="500"
            rows="5"
            placeholder="Напишите короткое сообщение..."
            style="
                width:100%;
                box-sizing:border-box;
                resize:vertical;
                padding:13px;
                border-radius:14px;
                border:1px solid rgba(255,255,255,.12);
                background:rgba(255,255,255,.035);
                color:var(--text);
                font:inherit;
                line-height:1.45;
            "></textarea>

        <div style="
            display:grid;
            grid-template-columns:1fr 1fr;
            gap:10px;
            margin-top:10px;
        ">

            <button
                id="contact-request-send"
                type="button"
                style="
                    min-height:44px;
                    border-radius:12px;
                    border:1px solid rgba(22,163,74,.35);
                    background:rgba(22,163,74,.10);
                    color:var(--text);
                    font-weight:800;
                    cursor:pointer;
                ">
                Отправить
            </button>

            <button
                id="contact-request-cancel"
                type="button"
                style="
                    min-height:44px;
                    border-radius:12px;
                    border:1px solid rgba(255,255,255,.12);
                    background:rgba(255,255,255,.03);
                    color:var(--text);
                    font-weight:750;
                    cursor:pointer;
                ">
                Отмена
            </button>

        </div>

        <div
            id="contact-request-status"
            style="
                margin-top:9px;
                font-size:12px;
                color:var(--muted);
                line-height:1.4;
            ">
        </div>

    </div>

</section>
"#,
        )
    };

    let intent_html = if safe_intent.is_empty() {
        String::new()
    } else {
        format!(
            r#"
<section class="card"
         style="
             display:block;
             padding:20px;
             margin-bottom:18px;
             border:1px solid rgba(214,183,122,.25);
         ">

    <div style="
        font-size:11px;
        color:var(--muted);
        text-transform:uppercase;
        letter-spacing:.07em;
        font-weight:800;
        margin-bottom:8px;
    ">
        Актуальный статус
    </div>

    <div style="
        font-size:16px;
        line-height:1.55;
        overflow-wrap:anywhere;
        white-space:pre-wrap;
    ">{intent}</div>

</section>
"#,
            intent = safe_intent
        )
    };

    let resource_count = resources.len();

    let cards = if resources.is_empty() {
        r#"
<div class="card"
     style="display:block;padding:20px;">

    <div class="card-title">
        Публичных ресурсов пока нет
    </div>

    <div class="card-meta"
         style="margin-top:6px;">
        У этого участника пока нет опубликованных ресурсов.
    </div>

</div>
"#
        .to_string()
    } else {
        resources
            .iter()
            .map(
                |(id, title, category, description, rating, votes, verified, premium)| {
                    let title = escape_html(title);
                    let category = escape_html(category);
                    let description = escape_html(description);

                    let verified_badge = if *verified != 0 {
                        r#"<span style="
                                font-size:11px;
                                font-weight:800;
                                color:#16a34a;
                            ">✓ Проверен</span>"#
                    } else {
                        ""
                    };

                    let premium_badge = if *premium != 0 {
                        r#"<span style="
                                font-size:10px;
                                font-weight:900;
                                color:#b88932;
                            ">★ PREMIUM</span>"#
                    } else {
                        ""
                    };

                    format!(
                        r#"
<a href="/app/resource/{id}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:14px;
       align-items:flex-start;
   ">

    <div class="card-icon">
        {resource_icon}
    </div>

    <div class="card-content"
         style="min-width:0;">

        <div class="card-title">
            {title}
        </div>

        <div class="card-meta"
             style="margin-top:4px;">
            {category}
        </div>

        <div class="card-meta"
             style="
                 margin-top:8px;
                 line-height:1.45;
                 overflow-wrap:anywhere;
             ">
            {description}
        </div>

        <div class="card-meta"
             style="margin-top:8px;">
            ⭐ {rating:.1} · {votes} голосов
        </div>

        <div style="
            display:flex;
            align-items:center;
            gap:8px;
            flex-wrap:wrap;
            margin-top:9px;
        ">
            {premium_badge}
            {verified_badge}
        </div>

    </div>

    <div class="card-arrow">
        {arrow}
    </div>

</a>
"#,
                        id = id,
                        resource_icon = icon("map-pin"),
                        title = title,
                        category = category,
                        description = description,
                        rating = rating,
                        votes = votes,
                        premium_badge = premium_badge,
                        verified_badge = verified_badge,
                        arrow = icon("chevron"),
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">

<head>
<meta charset="utf-8">
<meta name="viewport"
      content="width=device-width, initial-scale=1">

<title>{display_name} · ResursMap</title>

<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">

    <a class="brand"
       href="/app">

        <div class="brand-mark">
            {logo}
        </div>

        <div>
            <div class="brand-name">
                RESURSMAP
            </div>

            <div class="brand-sub">
                MEMBER
            </div>
        </div>

    </a>

</header>


<section class="hero">

    <a href="javascript:history.back()"
       style="
           display:inline-flex;
           align-items:center;
           gap:8px;
           color:var(--muted);
           text-decoration:none;
           margin-bottom:20px;
       ">
        {back}
        <span>Назад</span>
    </a>

    <div class="eyebrow">
        {user_icon}
        Участник ResursMap
    </div>

    <h1>{display_name}</h1>

    <p>
        Публичный профиль участника сообщества.
    </p>

</section>


<section class="card"
         style="
             display:block;
             padding:20px;
             margin-bottom:18px;
         ">

    <div style="
        display:flex;
        align-items:center;
        gap:15px;
    ">

        <div style="
            width:58px;
            height:58px;
            display:flex;
            align-items:center;
            justify-content:center;
            flex:0 0 auto;
            border-radius:18px;
            background:rgba(214,183,122,.10);
            border:1px solid rgba(214,183,122,.28);
        ">
            {profile_icon}
        </div>

        <div style="min-width:0;">

            <div style="
                font-size:20px;
                line-height:1.25;
                font-weight:900;
                overflow-wrap:anywhere;
            ">
                {display_name}
            </div>

            <div class="card-meta"
                 style="margin-top:5px;">
                {resource_count} ресурсов
            </div>

        </div>

    </div>

    {contact_html}

</section>


{intent_html}

{internal_contact_html}


<div class="section-head">

    <div>

        <h2 class="section-title">
            Ресурсы участника
        </h2>

        <p class="section-caption">
            Только активные и одобренные
        </p>

    </div>

</div>


<section>
    {cards}
</section>


</main>


<nav class="bottom-nav">

    <a class="nav-item"
       href="/app">
        {nav_map}
        <span>Карта</span>
    </a>

    <a class="nav-item"
       href="/app/search">
        {nav_search}
        <span>Поиск</span>
    </a>

    <a class="nav-item"
       href="/app/me">
        {nav_user}
        <span>Профиль</span>
    </a>

    <a class="nav-item"
       href="/app">
        {nav_menu}
        <span>Меню</span>
    </a>

</nav>



<script>
(function() {{
    const publicId = "{public_id}";

    const openButton =
        document.getElementById("contact-request-open");

    const panel =
        document.getElementById("contact-request-panel");

    const message =
        document.getElementById("contact-request-message");

    const sendButton =
        document.getElementById("contact-request-send");

    const cancelButton =
        document.getElementById("contact-request-cancel");

    const status =
        document.getElementById("contact-request-status");

    if (openButton && panel) {{
        openButton.addEventListener("click", () => {{
            panel.style.display = "block";

            if (message) {{
                message.focus();
            }}
        }});
    }}

    if (cancelButton && panel) {{
        cancelButton.addEventListener("click", () => {{
            panel.style.display = "none";

            if (status) {{
                status.textContent = "";
            }}
        }});
    }}

    if (sendButton) {{
        sendButton.addEventListener("click", async () => {{
            const text =
                message
                    ? message.value.trim()
                    : "";

            if (text.length < 2) {{
                if (status) {{
                    status.textContent =
                        "Напишите сообщение.";
                }}

                return;
            }}

            sendButton.disabled = true;

            if (status) {{
                status.textContent =
                    "Отправляем...";
            }}

            try {{
                const response = await fetch(
                    "/api/contact/request",
                    {{
                        method: "POST",
                        headers: {{
                            "Content-Type": "application/json"
                        }},
                        body: JSON.stringify({{
                            public_id: publicId,
                            message: text
                        }})
                    }}
                );

                const data =
                    await response.json();

                if (response.status === 401) {{
                    if (status) {{
                        status.textContent =
                            "Откройте ResursMap через Telegram.";
                    }}

                    sendButton.disabled = false;
                    return;
                }}

                if (
                    response.status === 400 &&
                    data.error === "cannot_contact_self"
                ) {{
                    if (status) {{
                        status.textContent =
                            "Нельзя отправить запрос самому себе.";
                    }}

                    sendButton.disabled = false;
                    return;
                }}

                if (
                    response.status === 409 &&
                    data.error === "request_already_pending"
                ) {{
                    if (status) {{
                        status.textContent =
                            "Запрос уже отправлен и ожидает ответа.";
                    }}

                    sendButton.disabled = false;
                    return;
                }}

                if (
                    response.status === 409 &&
                    data.error === "already_connected"
                ) {{
                    if (status) {{
                        status.textContent =
                            "Контакт уже установлен.";
                    }}

                    sendButton.disabled = false;
                    return;
                }}

                if (!data.ok) {{
                    if (status) {{
                        status.textContent =
                            "Не удалось отправить запрос.";
                    }}

                    sendButton.disabled = false;
                    return;
                }}

                if (status) {{
                    status.textContent =
                        "✓ Запрос отправлен";
                }}

                if (message) {{
                    message.value = "";
                }}

                openButton.disabled = true;
                openButton.textContent =
                    "Запрос отправлен";

            }} catch (_) {{
                if (status) {{
                    status.textContent =
                        "Ошибка соединения.";
                }}
            }}

            sendButton.disabled = false;
        }});
    }}
}})();
</script>

</body>
</html>"#,
        style = base_style(),
        logo = icon("user"),
        back = icon("arrow-left"),
        user_icon = icon("user"),
        profile_icon = icon("user"),
        display_name = display_name,
        resource_count = resource_count,
        contact_html = contact_html,
        intent_html = intent_html,
        internal_contact_html = internal_contact_html,
        public_id = safe_public_id,
        cards = cards,
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
    )
}

pub fn render_public_user_not_found() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ru">

<head>
<meta charset="utf-8">
<meta name="viewport"
      content="width=device-width, initial-scale=1">

<title>Профиль не найден · ResursMap</title>

<style>{style}</style>
</head>

<body>

<main class="page">

<section class="hero">

    <div class="eyebrow">
        ⚠ ResursMap
    </div>

    <h1>
        Профиль не найден
    </h1>

    <p>
        Пользователь недоступен
        или публичный профиль ещё не создан.
    </p>

    <a class="card"
       href="/app"
       style="
           text-decoration:none;
           margin-top:20px;
       ">

        <div class="card-icon">
            {map}
        </div>

        <div class="card-content">

            <div class="card-title">
                Вернуться на карту
            </div>

        </div>

        <div class="card-arrow">
            {arrow}
        </div>

    </a>

</section>

</main>

</body>
</html>"#,
        style = base_style(),
        map = icon("map"),
        arrow = icon("chevron"),
    )
}

// ============================================================
// ПРОФИЛЬ РЕСУРСА
// ============================================================
pub fn render_resource_profile(
    id: i64,
    title: &str,
    description: &str,
    contact: &str,
    address: &str,
    rating: f64,
    votes: i64,
    premium: i64,
    verified: i64,
    category: &str,
    _created_at: i64,
    owner_public_id: &str,
) -> String {
    let premium_badge = if premium != 0 {
        r#"<span style="
            display:inline-flex;
            align-items:center;
            gap:6px;
            padding:6px 11px;
            border-radius:999px;
            background:rgba(214,183,122,.12);
            border:1px solid rgba(214,183,122,.45);
            color:#b88932;
            font-size:11px;
            font-weight:800;
            letter-spacing:.08em;
        ">★ PREMIUM</span>"#
    } else {
        ""
    };

    let verified_badge = if verified != 0 {
        r#"<span style="
            display:inline-flex;
            align-items:center;
            gap:5px;
            color:#16a34a;
            font-size:12px;
            font-weight:700;
        ">✓ Проверен</span>"#
    } else {
        ""
    };

    let premium_style = if premium != 0 {
        "border:1px solid rgba(214,183,122,.55);background:linear-gradient(145deg,#fff,#faf6ee);box-shadow:0 12px 38px rgba(214,183,122,.14);"
    } else {
        "border:1px solid rgba(0,0,0,.07);"
    };

    let contact_clean = contact.trim();

    let contact_href = if contact_clean.starts_with('@') {
        format!("https://t.me/{}", contact_clean.trim_start_matches('@'))
    } else if contact_clean.starts_with("http://") || contact_clean.starts_with("https://") {
        contact_clean.to_string()
    } else {
        let phone: String = contact_clean
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '+')
            .collect();

        format!("tel:{}", phone)
    };

    let owner_profile_html = if owner_public_id.trim().is_empty() {
        String::new()
    } else {
        format!(
            r#"
<section class="card"
         style="
             display:flex;
             width:100%;
             box-sizing:border-box;
             margin-bottom:16px;
             padding:18px;
             text-decoration:none;
         ">

    <div class="card-icon"
         style="flex:0 0 auto;">
        {owner_icon}
    </div>

    <div class="card-content"
         style="min-width:0;">

        <div style="
            font-size:11px;
            text-transform:uppercase;
            letter-spacing:.07em;
            color:var(--muted);
            font-weight:800;
            margin-bottom:5px;
        ">
            Владелец ресурса
        </div>

        <div class="card-title">
            Профиль участника
        </div>

        <div class="card-meta"
             style="margin-top:4px;">
            Другие ресурсы и актуальный статус
        </div>

    </div>

    <a href="/app/user/{public_id}"
       style="
           flex:0 0 auto;
           align-self:center;
           min-height:40px;
           display:inline-flex;
           align-items:center;
           justify-content:center;
           padding:0 13px;
           border-radius:12px;
           text-decoration:none;
           color:var(--text);
           font-size:12px;
           font-weight:800;
           border:1px solid rgba(214,183,122,.32);
           background:rgba(214,183,122,.08);
       ">
        Открыть
    </a>

</section>
"#,
            owner_icon = icon("user"),
            public_id = owner_public_id,
        )
    };

    let map_query = address.trim().replace(' ', "+");

    let map_href = format!(
        "https://www.google.com/maps/search/?api=1&query={}",
        map_query
    );

    format!(
        r##"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">

<title>{title} · ResursMap</title>

<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">

    <a href="/app"
       style="display:inline-flex;align-items:center;gap:8px;
              color:var(--muted);text-decoration:none;margin-bottom:20px;">
        {back}
        <span>Вернуться к карте</span>
    </a>

    <div class="eyebrow">
        {category_icon}
        {category}
    </div>

    <div style="
        margin-top:14px;
        display:flex;
        align-items:center;
        gap:9px;
        flex-wrap:wrap;
    ">
        {premium_badge}
        {verified_badge}
    </div>

    <h1 style="margin-top:14px;">{title}</h1>

    <p id="rating-summary">
        ⭐ <strong>{rating:.1}</strong> · {votes} голосов
    </p>

    <button
        id="favorite-button"
        type="button"
        style="
            margin-top:12px;
            min-height:44px;
            padding:0 15px;
            border-radius:14px;
            border:1px solid rgba(220,38,38,.25);
            background:rgba(220,38,38,.06);
            color:var(--text);
            font-weight:800;
            cursor:pointer;
        ">
        ♡ В избранное
    </button>

    <div
        id="favorite-status"
        style="
            margin-top:7px;
            font-size:12px;
            color:var(--muted);
        ">
    </div>

    <button
        id="report-button"
        type="button"
        style="
            margin-top:10px;
            min-height:42px;
            padding:0 14px;
            border-radius:14px;
            border:1px solid rgba(217,119,6,.28);
            background:rgba(217,119,6,.06);
            color:var(--text);
            font-weight:700;
            cursor:pointer;
        ">
        ⚑ Пожаловаться
    </button>

    <div
        id="report-panel"
        style="
            display:none;
            margin-top:12px;
            padding:14px;
            border-radius:16px;
            border:1px solid rgba(217,119,6,.18);
            background:rgba(255,255,255,.03);
        ">

        <div style="
            font-size:13px;
            font-weight:800;
            margin-bottom:8px;
        ">
            Причина жалобы
        </div>

        <textarea
            id="report-reason"
            maxlength="500"
            rows="4"
            placeholder="Коротко опишите проблему..."
            style="
                width:100%;
                box-sizing:border-box;
                padding:12px 13px;
                border-radius:12px;
                border:1px solid rgba(255,255,255,.12);
                background:rgba(255,255,255,.04);
                color:var(--text);
                resize:vertical;
                font-size:14px;
            "></textarea>

        <div style="
            display:flex;
            gap:8px;
            margin-top:10px;
            flex-wrap:wrap;
        ">

            <button
                id="report-submit"
                type="button"
                style="
                    min-height:40px;
                    padding:0 14px;
                    border-radius:12px;
                    border:1px solid rgba(220,38,38,.30);
                    background:rgba(220,38,38,.08);
                    color:var(--text);
                    font-weight:800;
                    cursor:pointer;
                ">
                Отправить жалобу
            </button>

            <button
                id="report-cancel"
                type="button"
                style="
                    min-height:40px;
                    padding:0 14px;
                    border-radius:12px;
                    border:1px solid rgba(255,255,255,.10);
                    background:rgba(255,255,255,.03);
                    color:var(--text);
                    font-weight:700;
                    cursor:pointer;
                ">
                Отмена
            </button>

        </div>

        <div
            id="report-status"
            style="
                margin-top:9px;
                font-size:12px;
                color:var(--muted);
            ">
        </div>

    </div>

    <div style="margin-top:18px;">
        <div style="
            font-size:12px;
            color:var(--muted);
            margin-bottom:8px;
            font-weight:700;
            letter-spacing:.05em;
        ">
            ОЦЕНИТЬ РЕСУРС
        </div>

        <div id="rating-stars" style="
            display:flex;
            gap:8px;
            align-items:center;
        ">
            <button type="button" data-score="1" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;">☆</button>
            <button type="button" data-score="2" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;">☆</button>
            <button type="button" data-score="3" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;">☆</button>
            <button type="button" data-score="4" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;">☆</button>
            <button type="button" data-score="5" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;">☆</button>
        </div>

        <div id="vote-status" style="
            margin-top:7px;
            font-size:12px;
            color:var(--muted);
        "></div>
    </div>

</section>

<section
    class="card"
    style="
        {premium_style}
        display:block;
        width:100%;
        box-sizing:border-box;
        margin-bottom:16px;
        padding:20px;
        position:relative;
        overflow:hidden;
    "
>

    <div style="
        font-size:12px;
        color:var(--muted);
        margin-bottom:8px;
        text-transform:uppercase;
        letter-spacing:.08em;
        font-weight:700;
    ">
        О ресурсе
    </div>

    <div style="
        width:100%;
        min-width:0;
        box-sizing:border-box;
        font-size:16px;
        line-height:1.6;
        white-space:pre-wrap;
        overflow-wrap:anywhere;
        word-break:break-word;
    ">
        {description}
    </div>

</section>

{owner_profile_html}

<section
    class="card"
    style="
        display:block;
        width:100%;
        box-sizing:border-box;
        margin-bottom:16px;
        padding:20px;
    "
>

    <div style="
        font-size:12px;
        color:var(--muted);
        margin-bottom:12px;
        text-transform:uppercase;
        letter-spacing:.08em;
        font-weight:700;
    ">
        Контакты
    </div>

    <div class="card-meta"
         style="
             display:block;
             width:100%;
             margin-bottom:10px;
             line-height:1.5;
             overflow-wrap:anywhere;
         ">
        📍 {address}
    </div>

    <div class="card-meta"
         style="
             display:block;
             width:100%;
             line-height:1.5;
             overflow-wrap:anywhere;
         ">
        📞 {contact}
    </div>

    <div style="
        display:grid;
        grid-template-columns:repeat(2,minmax(0,1fr));
        width:100%;
        gap:10px;
        margin-top:18px;
    ">

        <a href="{contact_href}"
           style="
               display:flex;
               align-items:center;
               justify-content:center;
               gap:8px;
               min-width:0;
               min-height:48px;
               padding:0 10px;
               box-sizing:border-box;
               border-radius:14px;
               text-decoration:none;
               font-weight:800;
               color:var(--text);
               border:1px solid rgba(214,183,122,.38);
               background:rgba(214,183,122,.10);
           ">
            📞 Связаться
        </a>

        <a href="{map_href}"
           target="_blank"
           rel="noopener noreferrer"
           style="
               display:flex;
               align-items:center;
               justify-content:center;
               gap:8px;
               min-width:0;
               min-height:48px;
               padding:0 10px;
               box-sizing:border-box;
               border-radius:14px;
               text-decoration:none;
               font-weight:800;
               color:var(--text);
               border:1px solid var(--line);
               background:rgba(255,255,255,.035);
           ">
            📍 На карте
        </a>

    </div>

</section>

<section
    class="card"
    style="
        display:block;
        width:100%;
        box-sizing:border-box;
        margin-bottom:16px;
        padding:20px;
    "
>

    <div style="
        font-size:12px;
        color:var(--muted);
        margin-bottom:12px;
        text-transform:uppercase;
        letter-spacing:.08em;
        font-weight:700;
    ">
        ResursMap ID
    </div>

    <div style="font-size:18px;font-weight:700;">
        #{id}
    </div>

</section>

</main>

<script>
(function () {{
    const resourceId = {id};

    const favoriteButton =
        document.getElementById("favorite-button");

    const favoriteStatus =
        document.getElementById("favorite-status");

    function renderFavorite(value) {{
        if (!favoriteButton) return;

        favoriteButton.textContent =
            value
                ? "♥ В избранном"
                : "♡ В избранное";
    }}

    async function loadFavorite() {{
        try {{
            const response = await fetch(
                `/api/resource/${{resourceId}}/favorite`
            );

            const data = await response.json();

            if (data.ok) {{
                renderFavorite(Boolean(data.favorite));
            }}
        }} catch (_) {{}}
    }}

    if (favoriteButton) {{
        favoriteButton.addEventListener("click", async () => {{
            favoriteButton.disabled = true;

            if (favoriteStatus) {{
                favoriteStatus.textContent = "Сохраняем...";
            }}

            try {{
                const response = await fetch(
                    `/api/resource/${{resourceId}}/favorite`,
                    {{
                        method: "POST"
                    }}
                );

                if (response.status === 401) {{
                    if (favoriteStatus) {{
                        favoriteStatus.textContent =
                            "Откройте ResursMap через Telegram.";
                    }}

                    favoriteButton.disabled = false;
                    return;
                }}

                const data = await response.json();

                if (data.ok) {{
                    renderFavorite(Boolean(data.favorite));

                    if (favoriteStatus) {{
                        favoriteStatus.textContent =
                            data.favorite
                                ? "✓ Добавлено в избранное"
                                : "Удалено из избранного";
                    }}
                }}
            }} catch (_) {{
                if (favoriteStatus) {{
                    favoriteStatus.textContent =
                        "Ошибка соединения.";
                }}
            }}

            favoriteButton.disabled = false;
        }});
    }}

    loadFavorite();

    const reportButton =
        document.getElementById("report-button");

    const reportPanel =
        document.getElementById("report-panel");

    const reportReason =
        document.getElementById("report-reason");

    const reportSubmit =
        document.getElementById("report-submit");

    const reportCancel =
        document.getElementById("report-cancel");

    const reportStatus =
        document.getElementById("report-status");

    if (reportButton && reportPanel) {{
        reportButton.addEventListener("click", () => {{
            reportPanel.style.display = "block";

            if (reportReason) {{
                reportReason.focus();
            }}
        }});
    }}

    if (reportCancel && reportPanel) {{
        reportCancel.addEventListener("click", () => {{
            reportPanel.style.display = "none";

            if (reportStatus) {{
                reportStatus.textContent = "";
            }}
        }});
    }}

    if (reportSubmit) {{
        reportSubmit.addEventListener("click", async () => {{
            const reason =
                reportReason
                    ? reportReason.value.trim()
                    : "";

            if (reason.length < 3) {{
                if (reportStatus) {{
                    reportStatus.textContent =
                        "Опишите причину жалобы.";
                }}

                return;
            }}

            reportSubmit.disabled = true;

            if (reportStatus) {{
                reportStatus.textContent =
                    "Отправляем...";
            }}

            try {{
                const response = await fetch(
                    `/api/resource/${{resourceId}}/report`,
                    {{
                        method: "POST",
                        headers: {{
                            "Content-Type": "application/json"
                        }},
                        body: JSON.stringify({{
                            reason: reason
                        }})
                    }}
                );

                if (response.status === 401) {{
                    if (reportStatus) {{
                        reportStatus.textContent =
                            "Откройте ResursMap через Telegram.";
                    }}

                    reportSubmit.disabled = false;
                    return;
                }}

                const data = await response.json();

                if (data.ok) {{
                    if (reportStatus) {{
                        reportStatus.textContent =
                            "✓ Жалоба отправлена на проверку.";
                    }}

                    if (reportReason) {{
                        reportReason.value = "";
                    }}

                    reportButton.textContent =
                        "✓ Жалоба отправлена";
                }} else {{
                    if (reportStatus) {{
                        reportStatus.textContent =
                            "Не удалось отправить жалобу.";
                    }}
                }}
            }} catch (_) {{
                if (reportStatus) {{
                    reportStatus.textContent =
                        "Ошибка соединения.";
                }}
            }}

            reportSubmit.disabled = false;
        }});
    }}
    const stars = Array.from(
        document.querySelectorAll("#rating-stars button")
    );

    const status = document.getElementById("vote-status");
    const summary = document.getElementById("rating-summary");

    function paint(score) {{
        stars.forEach((star) => {{
            const value = Number(star.dataset.score);
            star.textContent = value <= score ? "★" : "☆";
        }});
    }}

    stars.forEach((star) => {{
        star.addEventListener("mouseenter", () => {{
            paint(Number(star.dataset.score));
        }});

        star.addEventListener("mouseleave", () => {{
            paint(0);
        }});

        star.addEventListener("click", async () => {{
            const score = Number(star.dataset.score);

            status.textContent = "Сохраняем оценку...";

            try {{
                const response = await fetch(
                    `/api/resource/${{resourceId}}/vote`,
                    {{
                        method: "POST",
                        headers: {{
                            "Content-Type": "application/json"
                        }},
                        body: JSON.stringify({{
                            
                            score: score
                        }})
                    }}
                );

                const data = await response.json();

                if (response.status === 401) {{
                    status.textContent =
                        "Откройте ResursMap через Telegram, чтобы поставить оценку.";
                    return;
                }}

                if (!data.ok) {{
                    status.textContent = "Не удалось сохранить оценку.";
                    return;
                }}

                paint(score);

                summary.innerHTML =
                    `⭐ <strong>${{Number(data.rating).toFixed(1)}}</strong> · ${{data.votes}} голосов`;

                status.textContent = "✓ Ваша оценка сохранена";
            }} catch (_) {{
                status.textContent = "Ошибка соединения.";
            }}
        }});
    }});
}})();
</script>

</body>
</html>"##,
        style = base_style(),
        logo = icon("map"),
        back = icon("arrow-left"),
        category_icon = icon("map-pin"),
        category = category,
        title = title,
        description = description,
        owner_profile_html = owner_profile_html,
        rating = rating,
        votes = votes,
        address = address,
        contact = contact,
        contact_href = contact_href,
        map_href = map_href,
        premium_badge = premium_badge,
        verified_badge = verified_badge,
        premium_style = premium_style,
        id = id,
    )
}

// ============================================================
// МОИ РЕСУРСЫ
// ============================================================

pub fn render_favorites(
    resources: Vec<(i64, String, String, String, String, f64, i64, i64, i64)>,
    authenticated: bool,
) -> String {
    let cards = if !authenticated {
        r#"
        <div class="card" style="display:block;">
            <div class="card-content">
                <div class="card-title">Откройте через Telegram</div>
                <div class="card-meta" style="margin-top:6px;">
                    Чтобы видеть избранное, откройте ResursMap через кнопку бота.
                </div>
            </div>
        </div>
        "#
        .to_string()
    } else if resources.is_empty() {
        r#"
        <div class="card" style="display:block;">
            <div class="card-content">
                <div class="card-title">Избранное пока пустое</div>
                <div class="card-meta" style="margin-top:6px;">
                    Откройте любой ресурс и нажмите ♡.
                </div>
            </div>
        </div>
        "#
        .to_string()
    } else {
        resources
            .iter()
            .map(
                |(
                    id,
                    title,
                    category,
                    description,
                    address,
                    rating,
                    votes,
                    verified,
                    premium,
                )| {
                    let premium_badge = if *premium != 0 {
                        r#"<span style="font-size:10px;font-weight:800;color:#b88932;">★ PREMIUM</span>"#
                    } else {
                        ""
                    };

                    let verified_badge = if *verified != 0 {
                        r#"<span style="font-size:11px;font-weight:800;color:#16a34a;">✓ Проверен</span>"#
                    } else {
                        ""
                    };

                    format!(
                        r#"
<a href="/app/resource/{id}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:14px;
       align-items:flex-start;
   ">

    <div class="card-icon">{resource_icon}</div>

    <div class="card-content" style="min-width:0;">

        <div class="card-title">{title}</div>

        <div class="card-meta" style="margin-top:4px;">
            {category}
        </div>

        <div class="card-meta"
             style="
                 margin-top:8px;
                 line-height:1.45;
                 overflow-wrap:anywhere;
             ">
            {description}
        </div>

        <div class="card-meta"
             style="
                 margin-top:8px;
                 overflow-wrap:anywhere;
             ">
            📍 {address}
        </div>

        <div class="card-meta" style="margin-top:7px;">
            ⭐ {rating:.1} · {votes} голосов
        </div>

        <div style="
            display:flex;
            gap:8px;
            flex-wrap:wrap;
            margin-top:9px;
        ">
            {premium_badge}
            {verified_badge}
        </div>

    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                        id = id,
                        resource_icon = icon("heart"),
                        title = title,
                        category = category,
                        description = description,
                        address = address,
                        rating = rating,
                        votes = votes,
                        premium_badge = premium_badge,
                        verified_badge = verified_badge,
                        arrow = icon("chevron"),
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Избранное · ResursMap</title>
<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">FAVORITES</div>
        </div>
    </a>
</header>

<section class="hero">

    <a href="/app/me"
       style="
           display:inline-flex;
           align-items:center;
           gap:8px;
           color:var(--muted);
           text-decoration:none;
           margin-bottom:20px;
       ">
        {back}
        <span>Профиль</span>
    </a>

    <div class="eyebrow">
        {heart}
        Избранное
    </div>

    <h1>Сохранённые ресурсы</h1>

    <p>Всё, что вы отметили сердцем.</p>

</section>

<section>
    {cards}
</section>

</main>

<nav class="bottom-nav">

    <a class="nav-item" href="/app">
        {nav_map}
        <span>Карта</span>
    </a>

    <a class="nav-item" href="/app/search">
        {nav_search}
        <span>Поиск</span>
    </a>

    <a class="nav-item active" href="/app/me">
        {nav_user}
        <span>Профиль</span>
    </a>

    <a class="nav-item" href="/app">
        {nav_menu}
        <span>Меню</span>
    </a>

</nav>

</body>
</html>"#,
        style = base_style(),
        logo = icon("heart"),
        back = icon("arrow-left"),
        heart = icon("heart"),
        cards = cards,
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
    )
}

pub fn render_my_resources(
    client_id: &str,
    resources: Vec<(
        i64,
        String,
        String,
        String,
        f64,
        i64,
        i64,
        i64,
        String,
        String,
        i64,
    )>,
) -> String {
    let cards = if client_id.is_empty() {
        r#"
        <div class="card" style="margin-top:20px;">
            <div class="card-content">
                <div class="card-title">Профиль владельца не определён</div>
                <div class="card-meta">
                    Откройте ResursMap через Telegram или этот браузер ещё раз.
                </div>
            </div>
        </div>
        "#
        .to_string()
    } else if resources.is_empty() {
        r#"
        <div class="card" style="margin-top:20px;">
            <div class="card-content">
                <div class="card-title">У вас пока нет ресурсов</div>
                <div class="card-meta">
                    Добавьте первый ресурс через нужный город и категорию.
                </div>
            </div>
        </div>
        "#
        .to_string()
    } else {
        resources
            .iter()
            .map(|(
                id,
                title,
                category,
                description,
                rating,
                votes,
                _verified,
                premium,
                moderation_status,
                rejection_reason,
                is_active
            )| {
                let premium_badge = if *premium != 0 {
                    r#"<span style="font-size:11px;font-weight:800;color:#b88932;">★ PREMIUM</span>"#
                } else {
                    ""
                };

                let moderation_badge = if *is_active == 0
                    && moderation_status != "rejected"
                {
                    r#"<span style="font-size:11px;font-weight:800;color:#dc2626;">⚫ Скрыт</span>"#
                } else {
                    match moderation_status.as_str() {
                        "approved" => {
                            r#"<span style="font-size:11px;font-weight:800;color:#16a34a;">🟢 Одобрен</span>"#
                        }
                        "rejected" => {
                            r#"<span style="font-size:11px;font-weight:800;color:#dc2626;">🔴 Отклонён</span>"#
                        }
                        _ => {
                            r#"<span style="font-size:11px;font-weight:800;color:#d97706;">🟡 На проверке</span>"#
                        }
                    }
                };

                let hidden_html =
                    if *is_active == 0
                        && moderation_status != "rejected"
                    {
                        r#"<div style="
                            margin-top:10px;
                            padding:10px 12px;
                            border-radius:12px;
                            border:1px solid rgba(220,38,38,.18);
                            background:rgba(220,38,38,.05);
                            color:#dc2626;
                            font-size:12px;
                            line-height:1.5;
                        "><strong>Ресурс скрыт.</strong> Он временно не показывается пользователям.</div>"#
                            .to_string()
                    } else {
                        String::new()
                    };

                let rejection_html =
                    if moderation_status == "rejected"
                        && !rejection_reason.trim().is_empty()
                    {
                        format!(
                            r#"<div style="
                                margin-top:10px;
                                padding:10px 12px;
                                border-radius:12px;
                                border:1px solid rgba(220,38,38,.22);
                                background:rgba(220,38,38,.07);
                                color:#dc2626;
                                font-size:12px;
                                line-height:1.5;
                            "><strong>Причина отказа:</strong> {}</div>"#,
                            rejection_reason
                        )
                    } else {
                        String::new()
                    };

                format!(
                    r#"
                    <article class="card"
                       style="
                           display:block;
                           margin-bottom:16px;
                           padding:18px;
                       ">

                        <div style="
                            display:flex;
                            align-items:flex-start;
                            gap:14px;
                        ">

                            <div class="card-icon"
                                 style="flex:0 0 auto;">
                                {icon}
                            </div>

                            <div style="
                                flex:1;
                                min-width:0;
                            ">

                                <div style="
                                    display:flex;
                                    justify-content:space-between;
                                    gap:12px;
                                    align-items:flex-start;
                                ">
                                    <div style="min-width:0;">
                                        <div class="card-title"
                                             style="
                                                 font-size:18px;
                                                 line-height:1.25;
                                                 margin-bottom:4px;
                                             ">
                                            {title}
                                        </div>

                                        <div class="card-meta"
                                             style="
                                                 font-size:12px;
                                                 text-transform:uppercase;
                                                 letter-spacing:.06em;
                                             ">
                                            {category}
                                        </div>
                                    </div>

                                    <div style="
                                        flex:0 0 auto;
                                        font-size:12px;
                                        color:var(--muted);
                                        white-space:nowrap;
                                    ">
                                        ⭐ {rating:.1} · {votes}
                                    </div>
                                </div>

                                <div class="card-meta"
                                     style="
                                         margin-top:10px;
                                         line-height:1.5;
                                         overflow-wrap:anywhere;
                                     ">
                                    {description}
                                </div>

                                <div style="
                                    display:flex;
                                    gap:8px;
                                    flex-wrap:wrap;
                                    align-items:center;
                                    margin-top:12px;
                                ">
                                    {premium_badge}
                                    {moderation_badge}
                                </div>

                                {rejection_html}
                                {hidden_html}

                                <div style="
                                    display:flex;
                                    gap:10px;
                                    flex-wrap:wrap;
                                    margin-top:14px;
                                ">

                                    <a
                                        href="/app/resource/{id}/edit"
                                        style="
                                            display:inline-flex;
                                            align-items:center;
                                            justify-content:center;
                                            min-height:42px;
                                            padding:0 14px;
                                            border-radius:12px;
                                            text-decoration:none;
                                            font-size:13px;
                                            font-weight:800;
                                            color:var(--text);
                                            border:1px solid rgba(214,183,122,.35);
                                            background:rgba(214,183,122,.08);
                                        "
                                    >
                                        ✎ Редактировать
                                    </a>

                                    <a
                                        href="/app/resource/{id}"
                                        style="
                                            display:inline-flex;
                                            align-items:center;
                                            justify-content:center;
                                            min-height:42px;
                                            padding:0 14px;
                                            border-radius:12px;
                                            text-decoration:none;
                                            font-size:13px;
                                            font-weight:700;
                                            color:var(--text);
                                            border:1px solid rgba(255,255,255,.10);
                                            background:rgba(255,255,255,.03);
                                        "
                                    >
                                        Открыть
                                    </a>

                                </div>
                            </div>
                        </div>

                    </article>
                    "#,
                    id = id,
                    icon = icon("map-pin"),
                    title = title,
                    category = category,
                    description = description,
                    rating = rating,
                    votes = votes,
                    premium_badge = premium_badge,
                    moderation_badge = moderation_badge,
                    rejection_html = rejection_html,
                    hidden_html = hidden_html,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Мои ресурсы · ResursMap</title>
<style>{style}</style>
</head>

<body>
<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">MY RESOURCES</div>
        </div>
    </a>
</header>

<section class="hero">
    <a href="/app/me"
       style="display:inline-flex;align-items:center;gap:8px;
              color:var(--muted);text-decoration:none;margin-bottom:20px;">
        {back}
        <span>Профиль</span>
    </a>

    <div class="eyebrow">
        {user_icon}
        Управление
    </div>

    <h1>Мои ресурсы</h1>

    <p>Ваши объявления, компании, услуги и другие ресурсы.</p>
</section>

<section>
    {cards}
</section>

</main>
</body>
</html>"#,
        style = base_style(),
        logo = icon("map"),
        back = icon("arrow-left"),
        user_icon = icon("user"),
        cards = cards,
    )
}

// ============================================================
// РЕДАКТИРОВАНИЕ РЕСУРСА
// ============================================================
pub fn render_edit_resource(
    id: i64,
    title: &str,
    description: &str,
    contact: &str,
    address: &str,
    category: &str,
) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Редактировать ресурс · ResursMap</title>
<style>{style}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">EDIT RESOURCE</div>
        </div>
    </a>
</header>

<section class="hero">
    <a href="/app/resource/{id}"
       style="display:inline-flex;align-items:center;gap:8px;
              color:var(--muted);text-decoration:none;margin-bottom:20px;">
        {back}
        <span>Назад к ресурсу</span>
    </a>

    <div class="eyebrow">
        {edit_icon}
        Редактирование
    </div>

    <h1>Редактировать ресурс</h1>
    <p>Категория: <strong>{category}</strong></p>
</section>

<form method="post"
      action="/app/resource/{id}/edit"
      style="display:flex;flex-direction:column;gap:16px;">

    <label>
        <div style="margin-bottom:7px;font-weight:600;">Название</div>
        <input
            name="title"
            required
            maxlength="120"
            value="{title}"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;">
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">Описание</div>
        <textarea
            name="description"
            required
            maxlength="1000"
            rows="6"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;
                   box-sizing:border-box;resize:vertical;">{description}</textarea>
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">Телефон или Telegram</div>
        <input
            name="contact"
            maxlength="120"
            value="{contact}"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;">
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">Адрес</div>
        <input
            name="address"
            maxlength="250"
            value="{address}"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;">
    </label>

    <button type="submit"
            style="min-height:52px;border:0;border-radius:16px;
                   font-size:16px;font-weight:800;cursor:pointer;
                   background:linear-gradient(135deg,#d6b77a,#b88932);
                   color:#111;">
        Сохранить изменения
    </button>

    <div style="font-size:12px;color:var(--muted);line-height:1.5;">
        После сохранения ресурс автоматически вернётся на повторную модерацию.
    </div>

</form>

</main>

</body>
</html>"##,
        style = base_style(),
        logo = icon("map"),
        back = icon("arrow-left"),
        edit_icon = icon("user"),
        id = id,
        title = title,
        description = description,
        contact = contact,
        address = address,
        category = category,
    )
}

pub fn render_add_resource(ci: usize, si: usize, zi: usize, category: &str) -> String {
    let back_url = format!("/app/{}/{}/{}", ci, si, zi);

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">

<title>Добавить ресурс · ResursMap</title>
<script src="https://telegram.org/js/telegram-web-app.js"></script>

<style>{}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">ADD RESOURCE</div>
        </div>
    </a>
</header>

<section class="hero">

    <a href="{}"
       style="display:inline-flex;align-items:center;gap:8px;
              color:var(--muted);text-decoration:none;margin-bottom:20px;">
        {}
        <span>Вернуться</span>
    </a>

    <div class="eyebrow">
        {}
        Добавление ресурса
    </div>

    <h1>Добавить ресурс</h1>

    <p>
        Категория: <strong>{}</strong>
    </p>

</section>

<form method="post"
      action="/app/{}/{}/{}/cat/{}/add"
      style="display:flex;flex-direction:column;gap:16px;">

    <input type="hidden" name="init_data" id="telegram-init-data" value="">



    <label>
        <div style="margin-bottom:7px;font-weight:600;">
            Название
        </div>

        <input
            name="title"
            required
            maxlength="120"
            placeholder="Например: Охранная компания"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;">
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">
            Описание
        </div>

        <textarea
            name="description"
            required
            maxlength="1000"
            rows="5"
            placeholder="Расскажите о ресурсе..."
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;
                   box-sizing:border-box;resize:vertical;"></textarea>
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">
            Телефон или Telegram
        </div>

        <input
            name="contact"
            maxlength="120"
            placeholder="+33... или @username"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;">
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">
            Адрес
        </div>

        <input
            name="address"
            maxlength="200"
            placeholder="Город, улица..."
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;">
    </label>

    <button
        type="submit"
        style="margin-top:8px;padding:16px;border:0;border-radius:16px;
               font-size:17px;font-weight:700;cursor:pointer;">
        ➕ Добавить ресурс
    </button>

</form>

</main>

<script>
(function() {{
    const initDataField = document.getElementById("telegram-init-data");

    // Передаём Telegram initData как безопасный fallback,
    // если пользовательская session cookie ещё не создана.
    try {{
        const tg =
            window.Telegram && window.Telegram.WebApp
                ? window.Telegram.WebApp
                : null;

        if (tg) {{
            tg.ready();

            if (initDataField && tg.initData) {{
                initDataField.value = tg.initData;
            }}
        }}
    }} catch (_) {{}}
}})();
</script>

</body>
</html>"#,
        base_style(),
        icon("globe"),
        back_url,
        icon("chevron"),
        icon("plus"),
        category,
        ci,
        si,
        zi,
        category,
    )
}
