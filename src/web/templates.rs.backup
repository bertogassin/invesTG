use std::collections::BTreeMap;

pub fn world() -> BTreeMap<&'static str, BTreeMap<&'static str, Vec<&'static str>>> {
    let mut w = BTreeMap::new();

    let mut eu = BTreeMap::new();

    eu.insert(
        "Франция",
        vec!["Париж", "Марсель", "Лион", "Тулуза", "Ницца"],
    );

    eu.insert(
        "Германия",
        vec!["Берлин", "Гамбург", "Мюнхен", "Кёльн"],
    );

    eu.insert(
        "Италия",
        vec!["Рим", "Милан", "Неаполь", "Турин"],
    );

    w.insert("Европа", eu);

    w
}

// ============================================================
// LUCIDE SVG
// ============================================================

fn icon(name: &str) -> &'static str {
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

fn base_style() -> &'static str {
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
    <div class="card-icon">
        {building}
    </div>

    <div class="card-content">
        <div class="card-title">{country}</div>
        <div class="card-meta">{continent} · страна</div>
    </div>

    <div class="card-arrow">
        {arrow}
    </div>
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
<style>{}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">
            {}
        </div>

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
        {}
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
    {}
</div>

<div class="section-head">
    <div>
        <h2 class="section-title">Возможности</h2>
        <p class="section-caption">Всё необходимое в одном месте</p>
    </div>
</div>

<div class="feature-grid">

    <div class="feature">
        {}
        <strong>Ресурсы</strong>
        <span>Находите полезные места и услуги.</span>
    </div>

    <div class="feature">
        {}
        <strong>Сообщество</strong>
        <span>Люди и возможности вашего города.</span>
    </div>

    <div class="feature">
        {}
        <strong>Профиль</strong>
        <span>Ваши данные, голоса и активность.</span>
    </div>

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
        icon("search"),
        cards,

        icon("map"),
        icon("heart"),
        icon("user"),

        icon("map"),
        icon("search"),
        icon("user"),
        icon("menu"),
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
        {}
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
        {}
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

pub fn render_search(q: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Поиск · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">SEARCH</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {}
        Поиск
    </div>

    <h1>Найти ресурс</h1>

    <p>Введите город, услугу, бизнес или другое направление.</p>

    <div class="search">
        {}
        <input
            autofocus
            value="{q}"
            placeholder="Например: строитель, Ницца..."
            onkeydown="if(event.key==='Enter') window.location='/app/search?q='+encodeURIComponent(this.value)"
        >
    </div>

</section>

</main>

<nav class="bottom-nav">

    <a class="nav-item" href="/app">
        {}
        <span>Карта</span>
    </a>

    <a class="nav-item active" href="/app/search">
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
        icon("search"),
        icon("search"),
        icon("search"),
        icon("map"),
        icon("search"),
        icon("user"),
        icon("menu"),
        q = q,
    )
}

// ============================================================
// PROFILE
// ============================================================

pub fn render_me() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Профиль · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">PROFILE</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {}
        Личный кабинет
    </div>

    <h1>Мой профиль</h1>

    <p>
        Здесь будут ваши данные, активность,
        избранное и ресурсы.
    </p>

</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Ваш ResursMap</h2>
        <p class="section-caption">Всё в одном месте</p>
    </div>
</div>

<div class="grid">

    <a class="card" href="/app">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Мои ресурсы</div>
            <div class="card-meta">Ваши публикации и активность</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

    <a class="card" href="/app">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Избранное</div>
            <div class="card-meta">Сохранённые ресурсы</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

</div>

</main>

<nav class="bottom-nav">

    <a class="nav-item" href="/app">
        {}
        <span>Карта</span>
    </a>

    <a class="nav-item" href="/app/search">
        {}
        <span>Поиск</span>
    </a>

    <a class="nav-item active" href="/app/me">
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
        icon("user"),
        icon("user"),
        icon("map"),
        icon("chevron"),
        icon("heart"),
        icon("chevron"),
        icon("map"),
        icon("search"),
        icon("user"),
        icon("menu"),
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
) -> String {
    let city_page = render_city(ci, si, zi);

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{category} · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">CATEGORY</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {}
        Категория
    </div>

    <h1>{category}</h1>

    <p>
        Ресурсы города в выбранной категории.
    </p>

</section>

<div style="margin-top:24px">
    {city_page}
</div>

</main>

</body>
</html>"#,
        base_style(),
        icon("map"),
        icon("map"),
    )
}
