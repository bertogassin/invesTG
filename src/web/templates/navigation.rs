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
