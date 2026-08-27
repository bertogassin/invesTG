pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

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

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(12px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

@keyframes fadeIn {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
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

    animation: fadeIn .4s ease both;
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

    animation: fadeInUp .5s ease both;
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

    animation: fadeInUp .45s ease both;
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


/* RESURSMAP_STAGE9_UI_SYSTEM */

.ui-form {
    width: 100%;
    box-sizing: border-box;
}

.ui-button {
    font-family: inherit;
    -webkit-tap-highlight-color: transparent;
    touch-action: manipulation;
    transition:
        transform .14s ease,
        opacity .14s ease,
        border-color .14s ease,
        background-color .14s ease;
}

.ui-button:active {
    transform: scale(.985);
}

.ui-button:disabled {
    cursor: not-allowed;
    opacity: .58;
}

.ui-input,
.ui-textarea,
.ui-select {
    font-family: inherit;
    box-sizing: border-box;
    max-width: 100%;
}

.ui-input:not([type="checkbox"]):not([type="radio"]),
.ui-textarea,
.ui-select {
    width: 100%;
}

.ui-input:not([type="checkbox"]):not([type="radio"]),
.ui-select {
    min-height: 46px;
}

.ui-textarea {
    min-height: 96px;
}

.ui-input:focus-visible,
.ui-textarea:focus-visible,
.ui-select:focus-visible,
.ui-button:focus-visible {
    outline: 2px solid rgba(214,183,122,.72);
    outline-offset: 2px;
}

.ui-badge {
    display: inline-flex;
    align-items: center;
    max-width: 100%;
    box-sizing: border-box;
}

.ui-status {
    min-height: 18px;
    overflow-wrap: anywhere;
}

@media (hover:hover) {
    .ui-button:hover {
        opacity: .94;
    }
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

pub(crate) fn page_document(
    title: &str,
    head_extra_html: &str,
    body_before_main_html: &str,
    main_html: &str,
    bottom_nav_html: &str,
    body_after_html: &str,
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>{style}</style>
{head_extra}
</head>

<body>

{body_before_main}

<main class="page">

{main}

</main>

{bottom_nav}

{body_after}

</body>
</html>"#,
        title = escape_html(title),
        style = base_style(),
        head_extra = head_extra_html,
        body_before_main = body_before_main_html,
        main = main_html,
        bottom_nav = bottom_nav_html,
        body_after = body_after_html,
    )
}

pub(crate) fn page_shell(
    title: &str,
    topbar_html: &str,
    hero_html: &str,
    content_html: &str,
    bottom_nav_html: &str,
) -> String {
    let main_html = format!(
        r#"{topbar}

{hero}

{content}"#,
        topbar = topbar_html,
        hero = hero_html,
        content = content_html,
    );

    page_document(title, "", "", &main_html, bottom_nav_html, "")
}

pub(crate) fn bottom_nav(active: &str) -> String {
    let item_class = |name: &str| {
        if active == name {
            "nav-item active"
        } else {
            "nav-item"
        }
    };

    format!(
        r#"
<nav class="bottom-nav">

    <a class="{map_class}" href="/app">
        {nav_map}
        <span>Карта</span>
    </a>

    <a class="{search_class}" href="/app/search">
        {nav_search}
        <span>Поиск</span>
    </a>

    <a class="{profile_class}" href="/app/me">
        {nav_user}
        <span>Профиль</span>
    </a>

    <a class="{menu_class}" href="/app">
        {nav_menu}
        <span>Меню</span>
    </a>

</nav>
"#,
        map_class = item_class("map"),
        search_class = item_class("search"),
        profile_class = item_class("profile"),
        menu_class = item_class("menu"),
        nav_map = icon("map"),
        nav_search = icon("search"),
        nav_user = icon("user"),
        nav_menu = icon("menu"),
    )
}

pub(crate) fn topbar(subtitle: &str, icon_name: &str) -> String {
    format!(
        r#"
<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">{subtitle}</div>
        </div>
    </a>
</header>
"#,
        logo = icon(icon_name),
        subtitle = escape_html(subtitle),
    )
}

pub(crate) fn back_link(href: &str, label: &str, icon_name: &str) -> String {
    format!(
        r#"<a href="{href}"
   style="display:inline-flex;align-items:center;gap:8px;
          color:var(--muted);text-decoration:none;margin-bottom:20px;">
    {icon}
    <span>{label}</span>
</a>"#,
        href = escape_html(href),
        label = escape_html(label),
        icon = icon(icon_name),
    )
}

pub(crate) struct ExtendedNavigationCardParams<'a> {
    pub id: Option<&'a str>,
    pub href: &'a str,
    pub icon_html: &'a str,
    pub title: &'a str,
    pub meta: &'a str,
    pub trailing_html: Option<&'a str>,
}

pub(crate) fn extended_navigation_card(params: ExtendedNavigationCardParams<'_>) -> String {
    let ExtendedNavigationCardParams {
        id,
        href,
        icon_html,
        title,
        meta,
        trailing_html,
    } = params;

    let id_html = id
        .filter(|value| !value.is_empty())
        .map(|value| format!(r#" id="{}""#, escape_html(value),))
        .unwrap_or_default();

    let trailing = trailing_html
        .map(str::to_string)
        .unwrap_or_else(|| format!(r#"<div class="card-arrow">{}</div>"#, icon("chevron"),));

    format!(
        r#"
<a{id_html} class="card" href="{href}">
    <div class="card-icon">
        {icon_html}
    </div>

    <div class="card-content">
        <div class="card-title">{title}</div>
        <div class="card-meta">{meta}</div>
    </div>

    {trailing}
</a>
"#,
        id_html = id_html,
        href = escape_html(href),
        icon_html = icon_html,
        title = escape_html(title),
        meta = escape_html(meta),
        trailing = trailing,
    )
}

pub(crate) fn navigation_card(href: &str, icon_name: &str, title: &str, meta: &str) -> String {
    extended_navigation_card(ExtendedNavigationCardParams {
        id: None,
        href,
        icon_html: icon(icon_name),
        title,
        meta,
        trailing_html: None,
    })
}

pub(crate) fn quick_navigation_card(
    href: &str,
    icon_char: &str,
    title: &str,
    meta: &str,
) -> String {
    format!(
        r#"<a href="{href}" class="card rm-quick-card">
    <div class="rm-quick-icon">{icon_char}</div>
    <div class="card-title">{title}</div>
    <div class="card-meta" style="margin-top:5px;">
        {meta}
    </div>
</a>"#,
        href = escape_html(href),
        icon_char = escape_html(icon_char),
        title = escape_html(title),
        meta = escape_html(meta),
    )
}

pub(crate) fn people_result_card(
    href: &str,
    display_name_html: &str,
    username_html: &str,
    intent_html: &str,
    contact_html: &str,
) -> String {
    format!(
        r#"
<a href="{href}"
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
        href = escape_html(href),
        user_icon = icon("user"),
        display_name = display_name_html,
        username_html = username_html,
        intent_html = intent_html,
        contact_html = contact_html,
        arrow = icon("chevron"),
    )
}

pub(crate) struct ResourceResultCardParams<'a> {
    pub href: &'a str,
    pub title_html: &'a str,
    pub category_html: &'a str,
    pub description_html: &'a str,
    pub rating: f64,
    pub votes: i64,
    pub location_html: &'a str,
    pub address_html: &'a str,
    pub premium_badge_html: &'a str,
    pub verified_badge_html: &'a str,
}

pub(crate) fn resource_result_card(params: ResourceResultCardParams<'_>) -> String {
    let ResourceResultCardParams {
        href,
        title_html,
        category_html,
        description_html,
        rating,
        votes,
        location_html,
        address_html,
        premium_badge_html,
        verified_badge_html,
    } = params;
    format!(
        r#"
<a href="{href}"
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
        href = escape_html(href),
        resource_icon = icon("map-pin"),
        title = title_html,
        category = category_html,
        description = description_html,
        rating = rating,
        votes = votes,
        location = location_html,
        address = address_html,
        premium_badge = premium_badge_html,
        verified_badge = verified_badge_html,
        arrow = icon("chevron"),
    )
}

pub(crate) struct ProfileResourceCardParams<'a> {
    pub href: &'a str,
    pub icon_name: &'a str,
    pub title: &'a str,
    pub category: &'a str,
    pub description: &'a str,
    pub address: Option<&'a str>,
    pub rating: f64,
    pub votes: i64,
    pub premium_badge_html: &'a str,
    pub verified_badge_html: &'a str,
}

pub(crate) fn profile_resource_card(params: ProfileResourceCardParams<'_>) -> String {
    let ProfileResourceCardParams {
        href,
        icon_name,
        title,
        category,
        description,
        address,
        rating,
        votes,
        premium_badge_html,
        verified_badge_html,
    } = params;

    let address_html = match address {
        Some(addr) if !addr.is_empty() => format!(
            r#"<div class="card-meta" style="margin-top:8px;overflow-wrap:anywhere;">
            📍 {addr}
        </div>
"#,
            addr = escape_html(addr),
        ),
        _ => String::new(),
    };

    format!(
        r#"<a href="{href}" class="card" style="text-decoration:none;color:inherit;margin-bottom:14px;align-items:flex-start;">
    <div class="card-icon">{resource_icon}</div>

    <div class="card-content" style="min-width:0;">
        <div class="card-title">{title}</div>

        <div class="card-meta" style="margin-top:4px;">{category}</div>

        <div class="card-meta" style="margin-top:8px;line-height:1.45;overflow-wrap:anywhere;">{description}</div>

        {address_html}
        <div class="card-meta" style="margin-top:8px;">⭐ {rating:.1} · {votes} голосов</div>

        <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-top:9px;">
            {premium_badge}
            {verified_badge}
        </div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>"#,
        href = escape_html(href),
        resource_icon = icon(icon_name),
        title = escape_html(title),
        category = escape_html(category),
        description = escape_html(description),
        address_html = address_html,
        rating = rating,
        votes = votes,
        premium_badge = premium_badge_html,
        verified_badge = verified_badge_html,
        arrow = icon("chevron"),
    )
}

pub(crate) fn status_page(
    title: &str,
    eyebrow: &str,
    heading: &str,
    description: &str,
    action_html: &str,
) -> String {
    let content = format!(
        r#"<section class="hero">
    <div class="eyebrow">{eyebrow}</div>
    <h1>{heading}</h1>
    <p>{description}</p>
    {action}
</section>"#,
        eyebrow = escape_html(eyebrow),
        heading = escape_html(heading),
        description = escape_html(description),
        action = action_html,
    );

    page_document(title, "", "", &content, "", "")
}

pub(crate) fn empty_state_card(title: &str, description_html: &str) -> String {
    format!(
        r#"
<div class="card" style="display:block;margin-top:18px;">
    <div class="card-content">
        <div class="card-title">{title}</div>
        <div class="card-meta" style="margin-top:5px;">
            {description}
        </div>
    </div>
</div>
"#,
        title = escape_html(title),
        description = description_html,
    )
}

pub(crate) fn section_head(title: &str, caption: &str, margin_top: Option<u32>) -> String {
    let style = match margin_top {
        Some(px) => format!(r#" style="margin-top:{px}px;""#),
        None => String::new(),
    };

    format!(
        r#"<div class="section-head"{style}>
    <div>
        <h2 class="section-title">{title}</h2>
        <p class="section-caption">{caption}</p>
    </div>
</div>"#,
        style = style,
        title = escape_html(title),
        caption = escape_html(caption),
    )
}

pub(crate) fn simple_hero(
    icon_name: &str,
    eyebrow: &str,
    title: &str,
    description_html: &str,
) -> String {
    format!(
        r#"<section class="hero">
    <div class="eyebrow">
        {icon}
        {eyebrow}
    </div>

    <h1>{title}</h1>

    <p>{description}</p>
</section>"#,
        icon = icon(icon_name),
        eyebrow = escape_html(eyebrow),
        title = escape_html(title),
        description = description_html,
    )
}

pub(crate) fn search_hero(
    eyebrow: &str,
    title: &str,
    description: &str,
    placeholder: &str,
) -> String {
    format!(
        r#"<section class="hero">
    <div class="eyebrow">
        <span class="eyebrow-dot"></span>
        {eyebrow}
    </div>

    <h1>{title}</h1>

    <p>{description}</p>

    <div class="search">
        {search_icon}
        <input
            type="text"
            placeholder="{placeholder}"
            onkeydown="if(event.key==='Enter') window.location='/app/search?q='+encodeURIComponent(this.value)"
        >
    </div>
</section>"#,
        eyebrow = escape_html(eyebrow),
        title = escape_html(title),
        description = escape_html(description),
        search_icon = icon("search"),
        placeholder = escape_html(placeholder),
    )
}

pub(crate) fn search_form_hero(
    eyebrow: &str,
    title: &str,
    description: &str,
    query: &str,
    placeholder: &str,
) -> String {
    format!(
        r#"<section class="hero">

    <div class="eyebrow">
        {search_icon}
        {eyebrow}
    </div>

    <h1>{title}</h1>

    <p>{description}</p>

    <form method="get"
          action="/app/search"
          class="search">
        {search_input_icon}

        <input
            name="q"
            autofocus
            value="{query}"
            placeholder="{placeholder}"
        >
    </form>

</section>"#,
        search_icon = icon("search"),
        search_input_icon = icon("search"),
        eyebrow = escape_html(eyebrow),
        title = escape_html(title),
        description = escape_html(description),
        query = escape_html(query),
        placeholder = escape_html(placeholder),
    )
}

pub(crate) fn back_hero(
    back_html: &str,
    icon_name: &str,
    eyebrow: &str,
    title: &str,
    description_html: &str,
) -> String {
    format!(
        r#"<section class="hero">
    {back}

    <div class="eyebrow">
        {icon}
        {eyebrow}
    </div>

    <h1>{title}</h1>

    <p>{description}</p>
</section>"#,
        back = back_html,
        icon = icon(icon_name),
        eyebrow = escape_html(eyebrow),
        title = escape_html(title),
        description = description_html,
    )
}
