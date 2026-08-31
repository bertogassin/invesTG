pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub const STATIC_ASSET_VERSION: &str = "4.9.24";

pub fn profession_label(raw: &str) -> String {
    match raw.trim().to_lowercase().as_str() {
        "work" | "job" | "jobs" => "Работа".to_string(),
        "business" => "Бизнес".to_string(),
        "services" | "service" => "Услуги".to_string(),
        "community" => "Сообщество".to_string(),
        _ => raw.trim().to_string(),
    }
}

pub fn static_asset(path: &str) -> String {
    let normalized = if path.starts_with("/static/") {
        path.to_string()
    } else {
        format!("/static/{}", path.trim_start_matches('/'))
    };

    format!("{}?v={}", normalized, STATIC_ASSET_VERSION)
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

        "message-circle" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M21 11.5c0 4.7-4 8.5-9 8.5-1 0-2-.2-2.9-.5L4 21l1.5-4.5C4.5 15.4 3 13.6 3 11.5 3 6.8 7 3 12 3s9 3.8 9 8.5Z"/></svg>"#
        }

        "menu" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M4 6h16M4 12h16M4 18h16"/></svg>"#
        }

        "logo" | "map-pin-brand" => {
            r#"<svg class="icon brand-logo-icon" viewBox="0 0 24 24"><path d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z"/><circle cx="12" cy="10" r="2.5"/></svg>"#
        }

        "arrow-left" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>"#
        }

        "chevron-left" => {
            r#"<svg class="icon small-icon" viewBox="0 0 24 24"><path d="m15 18-6-6 6-6"/></svg>"#
        }

        "shield" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z"/></svg>"#
        }

        "users" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>"#
        }

        "bell" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/></svg>"#
        }

        "settings" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>"#
        }

        "alert-triangle" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>"#
        }

        "plus" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M12 5v14"/><path d="M5 12h14"/></svg>"#
        }

        "edit" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>"#
        }

        "check" => r#"<svg class="icon" viewBox="0 0 24 24"><path d="m20 6-11 11-5-5"/></svg>"#,

        "x" => {
            r#"<svg class="icon" viewBox="0 0 24 24"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>"#
        }

        _ => "",
    }
}

pub fn brand_logo() -> &'static str {
    icon("logo")
}

pub(crate) fn site_head_links() -> String {
    format!(
        r##"<meta name="theme-color" content="#080a0d">
<link rel="icon" href="{icon}" type="image/svg+xml">
<link rel="apple-touch-icon" href="{icon}">
<link rel="manifest" href="{manifest}">"##,
        icon = static_asset("app-icon.svg"),
        manifest = static_asset("manifest.webmanifest"),
    )
}

// ============================================================
// GLOBAL DESIGN
// ============================================================

pub(crate) fn base_style() -> &'static str {
    r#"
:root {
    --bg: #080a0d;
    --bg-soft: #10141b;
    --surface: #141922;
    --card: rgba(24, 28, 36, .86);
    --card-hover: rgba(32, 37, 46, .94);
    --line: rgba(232, 204, 150, .26);

    --text: #f8f5ef;
    --muted: #a3a9b4;

    --gold: #e8cc96;
    --gold-light: #ffe4b8;
    --gold-glow: rgba(232, 204, 150, .42);

    --sea: #7ed4e4;
    --sea-light: #a8e8f4;
    --sea-glow: rgba(126, 212, 228, .28);

    --success: #6fe8b8;
    --warning: #ffb85c;
    --danger: #ff8f98;
    --info: #8ec5ff;

    --radius: 22px;
    --radius-sm: 14px;
    --radius-lg: 26px;
    --theme-color: #080a0d;
}

/* Светлая тема - активируется добавлением класса light-theme к body */
body.light-theme {
    --bg: #f7f5f0;
    --bg-soft: #efebe2;
    --card: rgba(255, 255, 255, .82);
    --card-hover: rgba(255, 255, 255, .95);
    --line: rgba(0,0,0,.08);

    --text: #1a1d21;
    --muted: #5c636e;

    --gold: #b88932;
    --gold-light: #8f6b1e;

    --sea: #3d8b9c;
    --sea-light: #2d6e7d;

    --success: #2f9b74;
    --warning: #c47d1a;
    --danger: #dc4b58;
    --info: #3f7ec9;
}

body.light-theme::before {
    background:
        linear-gradient(
            120deg,
            transparent 0%,
            rgba(0,0,0,.015) 50%,
            transparent 100%
        );
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

@keyframes pulseGlow {
    0%, 100% {
        opacity: .55;
        transform: scale(1);
    }
    50% {
        opacity: .85;
        transform: scale(1.04);
    }
}

@keyframes shimmerSlide {
    from {
        transform: translateX(-120%);
    }
    to {
        transform: translateX(120%);
    }
}

html {
    background: var(--bg);
    color-scheme: dark;
}

body {
    margin: 0;
    min-height: 100vh;
    padding-top: env(safe-area-inset-top, 0px);
    padding-left: env(safe-area-inset-left, 0px);
    padding-right: env(safe-area-inset-right, 0px);

    font-family:
        Inter,
        -apple-system,
        BlinkMacSystemFont,
        "Segoe UI",
        sans-serif;

    color: var(--text);

    background:
        radial-gradient(
            circle at 12% 0%,
            rgba(126, 212, 228, .20),
            transparent 40%
        ),
        radial-gradient(
            circle at 88% 8%,
            rgba(232, 204, 150, .18),
            transparent 36%
        ),
        radial-gradient(
            circle at 50% 100%,
            rgba(111, 232, 184, .07),
            transparent 42%
        ),
        linear-gradient(
            145deg,
            var(--bg) 0%,
            var(--bg-soft) 45%,
            var(--bg) 100%
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
            rgba(0,0,0,.018) 50%,
            transparent 100%
        );

    opacity: .5;
}

.page {
    width: min(100% - 32px, 900px);
    margin: 0 auto;
    padding: 28px 0 calc(110px + env(safe-area-inset-bottom, 0px));
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

    border: 1px solid rgba(224, 196, 138, .38);
    border-radius: 14px;

    color: var(--gold-light);

    background:
        radial-gradient(
            circle at 30% 25%,
            rgba(245, 219, 168, .32),
            transparent 52%
        ),
        linear-gradient(
            145deg,
            rgba(224, 196, 138, .18),
            rgba(114, 196, 212, .10)
        );

    box-shadow:
        0 10px 35px rgba(0, 0, 0, .28),
        0 0 28px rgba(224, 196, 138, .12),
        inset 0 1px 0 rgba(255, 255, 255, .08);
}

.brand-mark .brand-logo-icon {
    width: 24px;
    height: 24px;
    color: var(--gold-light);
}

.brand-name {
    font-size: 17px;
    font-weight: 800;
    letter-spacing: .08em;
    background: linear-gradient(135deg, var(--text) 0%, var(--gold-light) 100%);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
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
            rgba(214,183,122,.12),
            transparent 36%
        ),
        radial-gradient(
            circle at 0% 100%,
            rgba(101,184,201,.09),
            transparent 38%
        ),
        var(--card);

    box-shadow:
        0 30px 80px rgba(0,0,0,.32),
        inset 0 1px 0 rgba(0,0,0,.06);
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
    box-shadow: 0 0 14px rgba(214,183,122,.70);
}

.hero h1 {
    margin: 14px 0 10px;

    font-size: clamp(32px, 7vw, 56px);
    line-height: 1.02;
    letter-spacing: -.045em;
    text-shadow: 0 0 48px rgba(232, 204, 150, .14);
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
    padding: 16px 18px;
    border-radius: 16px;

    color: var(--muted);

    border: 1px solid var(--line);
    border-radius: 16px;

    background: rgba(214,183,122,.06);
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
    color: var(--muted);
}

/* -----------------------------------------------------------
   SECTION
----------------------------------------------------------- */

.section-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    margin: 34px 4px 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--line);
}

.section-title {
    margin: 0;
    padding-left: 12px;
    border-left: 3px solid var(--gold-light);
    font-size: 18px;
    letter-spacing: -.02em;
    box-shadow: -8px 0 24px rgba(232, 204, 150, .12);
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

.grid .card:nth-child(1) { animation-delay: .04s; }
.grid .card:nth-child(2) { animation-delay: .08s; }
.grid .card:nth-child(3) { animation-delay: .12s; }
.grid .card:nth-child(4) { animation-delay: .16s; }
.grid .card:nth-child(5) { animation-delay: .20s; }
.grid .card:nth-child(6) { animation-delay: .24s; }

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
            rgba(0,0,0,.055),
            rgba(0,0,0,.018)
        ),
        var(--card);

    box-shadow:
        0 16px 40px rgba(0,0,0,.18),
        inset 0 1px 0 rgba(0,0,0,.035);

    transition:
        transform .2s ease,
        border-color .2s ease,
        background .2s ease,
        box-shadow .2s ease;

    -webkit-tap-highlight-color: transparent;
}

.card:active {
    transform: scale(.98);
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
    width: 52px;
    height: 52px;

    display: grid;
    place-items: center;

    flex: 0 0 auto;

    color: var(--gold-light);

    border: 1px solid rgba(232, 204, 150, .28);
    border-radius: 16px;

    background:
        radial-gradient(
            circle at 30% 20%,
            rgba(255, 228, 184, .18),
            transparent 55%
        ),
        linear-gradient(
            145deg,
            rgba(232, 204, 150, .16),
            rgba(126, 212, 228, .08)
        );

    box-shadow:
        0 8px 22px rgba(0, 0, 0, .22),
        inset 0 1px 0 rgba(255, 255, 255, .08);
}

.card-content {
    min-width: 0;
    flex: 1;
}

.card-title {
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -.01em;
    line-height: 1.3;
}

.card-meta {
    margin-top: 5px;

    color: var(--muted);
    font-size: 13px;
    line-height: 1.5;
}

.card-arrow {
    color: var(--muted);
}

.rm-back-link {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
    text-decoration: none;
    margin-bottom: 20px;
}

.card--result {
    text-decoration: none;
    color: inherit;
    margin-bottom: 14px;
    align-items: flex-start;
}

.card-title--lg {
    font-size: 17px;
    min-width: 0;
}

.card-title--people {
    font-size: 17px;
    line-height: 1.3;
    overflow-wrap: anywhere;
    display: flex;
    align-items: center;
    gap: 8px;
}

.card-meta--mt-4 { margin-top: 4px; }
.card-meta--mt-8 { margin-top: 8px; }
.card-meta--mt-9 { margin-top: 9px; }
.card-meta--wrap { overflow-wrap: anywhere; }

.card-meta--desc {
    margin-top: 8px;
    line-height: 1.45;
    overflow-wrap: anywhere;
}

.rm-card-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-top: 10px;
}

.rm-card-row--badges {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    margin-top: 10px;
}

.rm-card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 10px;
}

.rm-card-rating {
    flex: 0 0 auto;
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
}

.rm-ready-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #22c55e;
    box-shadow: 0 0 6px rgba(34, 197, 94, .5);
    flex: 0 0 auto;
}

.section-head--mt-24 {
    margin-top: 24px;
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

    background: rgba(0,0,0,.025);
}

.feature .icon {
    color: var(--gold-light);
    filter: drop-shadow(0 0 8px rgba(224, 196, 138, .16));
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
    bottom: calc(16px + env(safe-area-inset-bottom, 0px));

    width: min(calc(100% - 28px), 520px);

    transform: translateX(-50%);

    display: grid;
    grid-template-columns: repeat(4, 1fr);

    padding: 8px;

    border: 1px solid var(--line);
    border-radius: 22px;

    background: var(--card);

    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);

    box-shadow:
        0 20px 60px rgba(0,0,0,.5),
        0 0 0 1px rgba(224, 196, 138, .08),
        0 0 40px rgba(224, 196, 138, .06),
        inset 0 1px 0 rgba(255, 255, 255, .05);
}

.nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;

    gap: 6px;

    min-height: 56px;

    color: var(--muted);
    text-decoration: none;

    border-radius: 16px;

    font-size: 11px;
    font-weight: 600;
    letter-spacing: .02em;

    transition:
        color .2s ease,
        background .2s ease,
        transform .2s ease;
}

.nav-item:hover,
.nav-item.active {
    color: var(--gold-light);
    background: rgba(224, 196, 138, .12);
    box-shadow: inset 0 0 24px rgba(224, 196, 138, .08);
}

.nav-item:active {
    transform: scale(.94);
}

.nav-item .icon {
    pointer-events: none;
}

.nav-item .icon {
    width: 22px;
    height: 22px;
}

.nav-badge {
    position: absolute;
    top: 4px;
    right: 4px;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    border-radius: 999px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #dc2626;
    color: #fff;
    font-size: 10px;
    font-weight: 800;
    box-sizing: border-box;
}

.nav-item {
    position: relative;
}

.nav-item.active .icon {
    stroke-width: 2.2;
    filter: drop-shadow(0 0 12px rgba(232, 204, 150, .45));
}

.nav-item.active::after {
    content: "";
    width: 5px;
    height: 5px;
    margin-top: 1px;
    border-radius: 50%;
    background: var(--gold-light);
    box-shadow: 0 0 12px var(--gold-glow);
}

/* -----------------------------------------------------------
   MOBILE
----------------------------------------------------------- */



.topbar-account {
    display:inline-flex;
    align-items:center;
    justify-content:center;
    gap:7px;
    min-height:42px;
    padding:0 13px;
    border:1px solid rgba(214,183,122,.24);
    border-radius:14px;
    background:
        linear-gradient(
            145deg,
            rgba(214,183,122,.12),
            rgba(214,183,122,.04)
        );
    color:var(--text);
    text-decoration:none;
    font-size:12px;
    font-weight:850;
    transition:
        border-color .16s ease,
        background .16s ease,
        transform .16s ease;
}

.topbar-account:hover {
    border-color:rgba(214,183,122,.48);
    background:rgba(214,183,122,.14);
}

.topbar-account:active {
    transform:scale(.97);
}

.topbar-account-icon {
    display:grid;
    place-items:center;
    width:18px;
    height:18px;
    color:var(--gold);
}

.topbar-account-icon .icon {
    width:18px;
    height:18px;
}

@media (max-width:420px) {
    .topbar-account {
        width:42px;
        padding:0;
    }

    .topbar-account-label {
        position:absolute;
        width:1px;
        height:1px;
        overflow:hidden;
        clip:rect(0 0 0 0);
        white-space:nowrap;
    }
}

/* ============================================================
   RESURSMAP LUXURY LAYER
   ============================================================ */

.rm-guest-panel {
    display: block;
    padding: 22px 20px;
    margin-bottom: 18px;
    border-color: rgba(232, 204, 150, .24);
    background:
        linear-gradient(145deg, rgba(232, 204, 150, .08), rgba(126, 212, 228, .05));
}

.rm-guest-title {
    font-size: 18px;
    line-height: 1.25;
}

.rm-guest-copy {
    margin-top: 8px;
    line-height: 1.55;
}

.rm-guest-actions {
    display: grid;
    gap: 10px;
    margin-top: 16px;
}

.rm-sessions-panel {
    margin-top: 18px;
    padding: 16px;
    border: 1px solid var(--line);
    border-radius: 18px;
    background: rgba(0, 0, 0, .03);
}

.rm-session-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 0;
    border-bottom: 1px solid var(--line);
}

.rm-session-row:last-child {
    border-bottom: 0;
    padding-bottom: 0;
}

.rm-session-current {
    color: var(--gold-light);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: .06em;
    text-transform: uppercase;
}

.topbar {
    position: relative;
    z-index: 5;

    animation: fadeIn .4s ease both;
}

.theme-toggle-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;

    min-width: 40px;
    min-height: 40px;

    padding: 0 10px;

    border: 1px solid var(--line);
    border-radius: 12px;

    background: rgba(0,0,0,.04);

    color: var(--muted);
    font-size: 13px;
    font-weight: 600;

    cursor: pointer;

    transition:
        color .2s ease,
        background .2s ease,
        border-color .2s ease;
}

.theme-toggle-btn:hover {
    color: var(--gold-light);
    border-color: rgba(214,183,122,.24);
    background: rgba(214,183,122,.06);
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
            rgba(245, 219, 168, .34),
            transparent 48%),
        linear-gradient(145deg,
            rgba(224, 196, 138, .20),
            rgba(114, 196, 212, .10));
    border: 1px solid rgba(224, 196, 138, .36);
    box-shadow:
        0 8px 30px rgba(0,0,0,.35),
        0 0 32px rgba(224, 196, 138, .14),
        inset 0 1px 0 rgba(255,255,255,.08);
    backdrop-filter: blur(18px);
}

.hero {
    position: relative;
    overflow: hidden;
    border: 1px solid var(--line);
    border-radius: 28px;
    padding: 36px 28px;
    box-shadow:
        0 24px 70px rgba(0,0,0,.28),
        inset 0 1px 0 rgba(0,0,0,.05);

    animation: fadeInUp .5s ease both;
}

.hero h1 {
    font-size: 42px;
    font-weight: 800;
    letter-spacing: -.02em;
    line-height: 1.1;
    margin: 0 0 12px 0;
}

.hero p {
    font-size: 16px;
    line-height: 1.6;
    color: var(--muted);
    margin: 0 0 24px 0;
    max-width: 560px;
}

.hero .eyebrow {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: .06em;
    text-transform: uppercase;
    color: var(--gold-light);
}

.hero::before {
    content: "";
    position: absolute;
    width: 320px;
    height: 320px;
    top: -180px;
    right: -120px;
    border-radius: 50%;
    background: rgba(126, 212, 228, .20);
    filter: blur(52px);
    pointer-events: none;
    animation: pulseGlow 8s ease-in-out infinite;
}

.hero::after {
    content: "";
    position: absolute;
    width: 260px;
    height: 260px;
    bottom: -160px;
    left: -100px;
    border-radius: 50%;
    background: rgba(232, 204, 150, .18);
    filter: blur(52px);
    pointer-events: none;
    animation: pulseGlow 10s ease-in-out infinite reverse;
}

.search {
    position: relative;
    z-index: 2;
    border: 1px solid var(--line);
    background: rgba(0,0,0,.045);
    box-shadow:
        inset 0 1px 0 rgba(0,0,0,.05),
        0 12px 35px rgba(0,0,0,.18);
    transition:
        border-color .25s ease,
        box-shadow .25s ease,
        background .25s ease;
}

.search:focus-within {
    border-color: rgba(214,183,122,.45);
    background: rgba(0,0,0,.065);
    box-shadow:
        0 0 0 4px rgba(214,183,122,.06),
        0 16px 45px rgba(0,0,0,.25);
}

.card {
    position: relative;
    overflow: hidden;
    border: 1px solid var(--line);

    animation: fadeInUp .45s ease both;
    background:
        linear-gradient(
            145deg,
            rgba(0,0,0,.055),
            rgba(0,0,0,.018)
        );
    box-shadow:
        0 12px 35px rgba(0,0,0,.18),
        inset 0 1px 0 rgba(0,0,0,.045);
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
            rgba(0,0,0,.035) 50%,
            transparent 80%
        );
    transform: translateX(-100%);
    transition: transform .55s ease;
    pointer-events: none;
}

.card:hover {
    transform: translateY(-4px);
    border-color: rgba(232, 204, 150, .38);
    background:
        linear-gradient(
            145deg,
            rgba(232, 204, 150, .11),
            rgba(126, 212, 228, .06)
        );
    box-shadow:
        0 22px 56px rgba(0,0,0,.32),
        0 0 48px rgba(232, 204, 150, .10),
        inset 0 1px 0 rgba(255,255,255,.08);
}

.card:hover::before {
    transform: translateX(100%);
}

.card-icon {
    color: var(--gold-light);
    filter: drop-shadow(0 0 10px rgba(224, 196, 138, .18));
    transition: transform .25s ease, color .25s ease, filter .25s ease;
}

.card:hover .card-icon {
    transform: scale(1.08);
    color: #fff1d0;
    filter: drop-shadow(0 0 14px rgba(224, 196, 138, .32));
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
    border: 1px solid rgba(0,0,0,.07);
    background:
        linear-gradient(
            145deg,
            rgba(0,0,0,.045),
            rgba(0,0,0,.015)
        );
    box-shadow:
        0 12px 35px rgba(0,0,0,.16),
        inset 0 1px 0 rgba(0,0,0,.04);
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
    border-top: 1px solid var(--line);
    background: var(--card);
    box-shadow:
        0 -12px 40px rgba(0,0,0,.25),
        inset 0 1px 0 rgba(0,0,0,.04);
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
    filter: drop-shadow(0 0 10px rgba(224, 196, 138, .36));
}

.section-head {
    border-bottom: 1px solid var(--line);
}

.section-caption {
    color: var(--muted);
}


/* RESURSMAP_STAGE9_UI_SYSTEM */

.ui-form {
    width: 100%;
    box-sizing: border-box;
}

.ui-form-stack {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.ui-field {
    display: block;
}

.ui-field-label {
    display: block;
    margin-bottom: 7px;
    font-weight: 600;
    color: var(--text);
}

.ui-form-note {
    font-size: 12px;
    color: var(--muted);
    line-height: 1.5;
}

.rm-auth-wrap {
    width: min(100%, 520px);
    margin: 0 auto;
    padding: 18px;
}

.rm-auth-card {
    display: block;
    padding: 26px 22px;
    border-color: rgba(214, 183, 122, .24);
}

.rm-auth-title {
    margin: 0 0 10px;
    color: var(--text);
    font-size: clamp(30px, 8vw, 42px);
}

.rm-auth-subtitle {
    margin: 0 0 24px;
    color: var(--muted);
    font-size: 16px;
    line-height: 1.58;
}

.rm-auth-label {
    display: block;
    margin-bottom: 8px;
    color: var(--text);
    font-size: 14px;
    font-weight: 750;
}

.rm-auth-label + .rm-auth-input + .rm-auth-label,
.rm-auth-step .rm-auth-label {
    margin-top: 16px;
}

.rm-auth-input {
    width: 100%;
    min-height: 52px;
    padding: 0 15px;
    border-radius: 14px;
}

.rm-auth-input--code {
    text-align: center;
    letter-spacing: .24em;
    font-size: 20px;
}

.rm-auth-password-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
}

.rm-auth-password-toggle {
    min-height: 52px;
    padding: 0 14px;
    border-radius: 14px;
    border: 1px solid rgba(214, 183, 122, .24);
    background: rgba(255, 255, 255, .03);
    color: var(--text);
    font-size: 14px;
    font-weight: 650;
    cursor: pointer;
}

.rm-auth-links {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin-top: 10px;
    font-size: 13px;
}

.rm-auth-links a {
    text-decoration: none;
}

.rm-auth-links a:first-child {
    color: var(--gold-light);
}

.rm-auth-links a:last-child {
    color: var(--muted);
}

.rm-auth-button {
    width: 100%;
    min-height: 52px;
    margin-top: 16px;
    border-radius: 14px;
    font-size: 16px;
    font-weight: 850;
    cursor: pointer;
}

.rm-auth-button--compact {
    margin-top: 12px;
}

.rm-auth-status {
    min-height: 22px;
    margin: 15px 0 0;
    color: var(--muted);
    font-size: 14px;
}

.rm-auth-status.is-error {
    color: #ef6b72;
}

.rm-auth-footer {
    margin: 18px 0 0;
    text-align: center;
    color: var(--muted);
    font-size: 14px;
}

.rm-auth-footer a {
    color: var(--gold-light);
}

.rm-auth-back {
    margin-top: 18px;
    text-align: center;
}

.rm-auth-back a {
    color: var(--muted);
    text-decoration: none;
    font-size: 14px;
}

.rm-auth-step {
    margin-top: 20px;
    padding-top: 20px;
    border-top: 1px solid rgba(255, 255, 255, .08);
}

.rm-premium-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 9px;
    border-radius: 999px;
    border: 1px solid rgba(214, 183, 122, .45);
    background: rgba(214, 183, 122, .12);
    color: var(--gold-light);
    font-size: 10px;
    font-weight: 800;
    letter-spacing: .08em;
}

.rm-premium-badge--compact {
    padding: 0;
    border: 0;
    background: transparent;
    font-size: 10px;
}

.rm-premium-badge--admin {
    padding: 0;
    border: 0;
    background: transparent;
    color: #b88932;
    font-size: inherit;
    letter-spacing: normal;
}

a.feature {
    text-decoration: none;
    color: inherit;
}

a.feature.rm-feature-add {
    margin-top: 12px;
    display: block;
}

.rm-empty-state {
    display: block;
    margin-top: 18px;
    padding: 28px 24px;
    text-align: center;
}

.rm-empty-state .card-title {
    font-size: 17px;
    margin-bottom: 8px;
}

.rm-empty-state .card-meta {
    margin-top: 5px;
    line-height: 1.6;
}

.rm-empty-state-actions {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 18px;
}

.rm-empty-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 44px;
    padding: 0 16px;
    border-radius: 14px;
    text-decoration: none;
    font-size: 14px;
    font-weight: 850;
}

.rm-empty-professions {
    margin: 0;
    padding: 16px 18px;
    border: 1px dashed var(--line);
    border-radius: 16px;
    color: var(--muted);
    font-size: 14px;
    line-height: 1.55;
}

.rm-guest-hint {
    margin: 14px 0 0;
    padding: 12px 14px;
    border-radius: 12px;
    border: 1px solid rgba(214, 183, 122, .24);
    background: rgba(214, 183, 122, .08);
    color: var(--muted);
    font-size: 14px;
    line-height: 1.5;
}

.rm-guest-hint a {
    color: var(--gold-light);
}

.rm-verified-badge {
    font-size: 11px;
    font-weight: 800;
    color: #16a34a;
}

.rm-verified-badge--compact {
    font-size: 10px;
}

.rm-notif-card {
    display: block;
    padding: 18px;
    margin-bottom: 14px;
    border-left: 3px solid var(--gold-light);
}

.rm-notif-card--approved,
.rm-notif-card--contact {
    border-left-color: #16a34a;
}

.rm-notif-card--rejected {
    border-left-color: #dc2626;
}

.rm-notif-card--chat {
    border-left-color: var(--gold);
}

.rm-notif-layout {
    display: flex;
    gap: 13px;
    align-items: flex-start;
}

.rm-notif-icon {
    width: 42px;
    height: 42px;
    border-radius: 13px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    font-size: 19px;
    font-weight: 900;
    border: 1px solid rgba(214, 183, 122, .20);
    background: rgba(0, 0, 0, .03);
}

.rm-notif-icon--approved,
.rm-notif-icon--contact {
    color: #16a34a;
}

.rm-notif-icon--rejected {
    color: #dc2626;
}

.rm-notif-icon--chat {
    color: var(--gold);
}

.rm-notif-icon--default {
    color: var(--gold-light);
}

.rm-notif-body {
    min-width: 0;
    flex: 1;
}

.rm-notif-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 10px;
}

.rm-notif-message {
    margin-top: 7px;
    line-height: 1.55;
    overflow-wrap: anywhere;
}

.rm-notif-new {
    font-size: 10px;
    font-weight: 900;
    color: var(--gold);
    text-transform: uppercase;
    letter-spacing: .06em;
}

.rm-notif-action {
    display: inline-flex;
    align-items: center;
    min-height: 40px;
    padding: 0 13px;
    border-radius: 12px;
    text-decoration: none;
    color: var(--text);
    font-size: 13px;
    font-weight: 800;
    margin-top: 13px;
}

.rm-notif-action--gold {
    border: 1px solid rgba(214, 183, 122, .28);
    background: rgba(214, 183, 122, .07);
}

.rm-notif-action--neutral {
    border: 1px solid var(--line);
    background: rgba(0, 0, 0, .03);
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
    padding: 13px 16px;
    border: 1px solid var(--line);
    border-radius: 13px;
    background: rgba(0,0,0,.045);
    color: var(--text);
    font-size: 15px;
    transition:
        border-color .25s ease,
        background .25s ease,
        box-shadow .25s ease;
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
    outline: 2px solid rgba(214,183,122,.70);
    outline-offset: 2px;
}

.ui-input:focus,
.ui-textarea:focus,
.ui-select:focus {
    border-color: rgba(214,183,122,.45);
    background: rgba(0,0,0,.065);
    box-shadow: 0 0 0 4px rgba(214,183,122,.06);
    outline: none;
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

.rm-session-form {
    margin: 0;
}

.rm-session-revoke-btn {
    min-height: 34px;
    padding: 0 12px;
    font-size: 11px;
}

.rm-session-ip {
    margin-top: 4px;
}

.rm-sessions-title {
    font-size: 16px;
    margin-bottom: 8px;
}

.rm-sessions-copy {
    margin-bottom: 10px;
    line-height: 1.5;
}

.rm-sessions-revoke-all {
    margin-top: 14px;
}

.rm-sessions-revoke-all-btn {
    width: 100%;
    min-height: 42px;
}

.rm-mod-level-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 800;
    letter-spacing: .04em;
    text-decoration: none;
}

.rm-mod-level-badge--1 {
    background: rgba(42, 199, 133, .10);
    border: 1px solid rgba(42, 199, 133, .34);
    color: #69e6ae;
}

.rm-mod-level-badge--2 {
    background: rgba(100, 168, 255, .10);
    border: 1px solid rgba(100, 168, 255, .36);
    color: #8fc2ff;
}

.rm-mod-level-badge--3 {
    background: linear-gradient(90deg, rgba(100, 168, 255, .09), rgba(214, 183, 122, .08));
    border: 1px solid rgba(214, 183, 122, .38);
    color: #e6d09f;
}

.rm-mod-level-badge--4 {
    background: linear-gradient(90deg, rgba(137, 116, 255, .12), rgba(214, 183, 122, .08));
    border: 1px solid rgba(137, 116, 255, .42);
    color: #c2b7ff;
}

.rm-mod-level-badge--5 {
    background: linear-gradient(90deg, rgba(214, 183, 122, .15), rgba(137, 116, 255, .10));
    border: 1px solid rgba(214, 183, 122, .48);
    color: #f0d69c;
    box-shadow: 0 0 24px rgba(214, 183, 122, .14);
}

.rm-me-account-card {
    display: block;
    margin-bottom: 20px;
    padding: 20px;
    border: 1px solid rgba(214, 183, 122, .22);
}

.rm-me-account-row {
    display: flex;
    align-items: center;
    gap: 15px;
}

.rm-me-avatar {
    width: 58px;
    height: 58px;
    border-radius: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    background: rgba(214, 183, 122, .08);
    border: 1px solid rgba(214, 183, 122, .28);
}

.rm-me-name-wrap {
    min-width: 0;
}

.rm-me-name {
    font-size: 21px;
    line-height: 1.2;
    font-weight: 850;
    overflow-wrap: anywhere;
}

.rm-me-username {
    margin-top: 5px;
    color: var(--muted);
    font-size: 14px;
}

.rm-me-username--guest {
    font-size: 13px;
}

.rm-me-account-id {
    margin-top: 10px;
    font-size: 11px;
    color: var(--muted);
    letter-spacing: .04em;
}

.rm-me-logout-form {
    margin-top: 12px;
}

.rm-me-logout-btn {
    width: 100%;
    min-height: 44px;
}

.rm-me-stats {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    margin-bottom: 24px;
}

.rm-me-stat-card {
    display: block;
    padding: 16px;
}

.rm-me-stat-value {
    font-size: 26px;
    font-weight: 900;
    line-height: 1;
}

.rm-me-stat-value--md {
    font-size: 24px;
}

.rm-me-stat-value--ok {
    color: #16a34a;
}

.rm-me-stat-value--warn {
    color: #d97706;
}

.rm-me-stat-meta {
    margin-top: 7px;
}

.rm-me-rejected-row {
    display: flex;
    margin-bottom: 20px;
    padding: 14px 16px;
    border: 1px solid rgba(220, 38, 38, .14);
}

.rm-me-rejected-title {
    font-size: 14px;
}

.rm-me-rejected-copy {
    margin-top: 3px;
}

.rm-me-rejected-count {
    font-size: 22px;
    font-weight: 900;
    color: #dc2626;
}

.rm-profile-section {
    display: block;
    padding: 20px;
    margin-bottom: 24px;
}

.rm-profile-section--compact {
    margin-bottom: 18px;
}

.rm-profile-section-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 14px;
    margin-bottom: 18px;
}

.rm-profile-section-title {
    font-size: 18px;
}

.rm-profile-section-copy {
    margin-top: 5px;
    line-height: 1.45;
}

.rm-profile-icon-box {
    width: 42px;
    height: 42px;
    border-radius: 13px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    background: rgba(214, 183, 122, .08);
    border: 1px solid rgba(214, 183, 122, .24);
}

.rm-profile-intent-box {
    padding: 12px 14px;
    border-radius: 13px;
    background: rgba(0, 0, 0, .03);
    border: 1px solid rgba(0, 0, 0, .07);
    margin-bottom: 16px;
}

.rm-profile-intent-kicker {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: .06em;
    font-weight: 800;
    margin-bottom: 5px;
}

.rm-profile-intent-text {
    font-size: 14px;
    line-height: 1.5;
    overflow-wrap: anywhere;
}

.rm-profile-settings-block {
    margin-top: 20px;
    padding-top: 18px;
    border-top: 1px solid var(--line);
}

.rm-profile-settings-kicker {
    font-size: 11px;
    font-weight: 900;
    letter-spacing: .08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 12px;
}

.theme-toggle-btn.rm-profile-theme-btn {
    width: 100%;
    min-height: 48px;
    gap: 8px;
    padding: 0 16px;
    border-color: rgba(214, 183, 122, .28);
    border-radius: 14px;
    background: rgba(214, 183, 122, .06);
    color: var(--gold-light);
    font-size: 14px;
    font-weight: 700;
}

.rm-profile-field {
    display: block;
}

.rm-profile-field--spaced {
    margin-top: 15px;
}

.rm-profile-field-label {
    margin-bottom: 7px;
    font-size: 13px;
    font-weight: 800;
}

.rm-profile-save-btn {
    width: 100%;
    min-height: 50px;
    margin-top: 17px;
    border: 0;
    border-radius: 15px;
    font-size: 15px;
    font-weight: 900;
    color: #111;
    background: linear-gradient(135deg, var(--gold), var(--gold-light));
}

.rm-profile-save-status {
    margin-top: 9px;
    font-size: 12px;
    color: var(--muted);
    line-height: 1.4;
}

.rm-profile-nav-grid {
    display: grid;
    gap: 12px;
}

.rm-profile-trailing {
    display: flex;
    align-items: center;
    gap: 10px;
}

.rm-profile-trailing-count {
    min-width: 28px;
    height: 28px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 900;
    background: rgba(214, 183, 122, .12);
    border: 1px solid rgba(214, 183, 122, .28);
    color: var(--gold-light);
}

.rm-public-section {
    display: block;
    padding: 20px;
    margin-bottom: 18px;
}

.rm-public-section--intent {
    border: 1px solid rgba(214, 183, 122, .24);
}

.rm-public-kicker {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: .07em;
    font-weight: 800;
    margin-bottom: 8px;
}

.rm-public-copy {
    line-height: 1.5;
    margin-bottom: 14px;
}

.rm-public-chat-link {
    width: 100%;
    min-height: 48px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 14px;
    border: 1px solid rgba(22, 163, 74, .38);
    background: rgba(22, 163, 74, .10);
    color: var(--text);
    text-decoration: none;
    font-weight: 850;
}

.rm-public-contact-btn {
    width: 100%;
    min-height: 48px;
}

.rm-public-contact-panel {
    display: none;
    margin-top: 14px;
}

.rm-public-contact-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-top: 10px;
}

.rm-public-contact-send {
    min-height: 44px;
    border-color: rgba(22, 163, 74, .35);
    background: rgba(22, 163, 74, .10);
}

.rm-public-contact-cancel {
    min-height: 44px;
    background: rgba(0, 0, 0, .03);
    font-weight: 750;
}

.rm-public-contact-status {
    margin-top: 9px;
    font-size: 12px;
    color: var(--muted);
    line-height: 1.4;
}

.rm-public-profile-card {
    display: block;
    margin-bottom: 20px;
    padding: 20px;
    border: 1px solid rgba(214, 183, 122, .22);
}

.rm-public-profile-row {
    display: flex;
    align-items: center;
    gap: 15px;
}

.rm-public-intent-text {
    margin: 0;
    line-height: 1.55;
    white-space: pre-wrap;
}

.rm-public-name {
    font-size: 20px;
    line-height: 1.25;
    font-weight: 900;
    overflow-wrap: anywhere;
}

.rm-public-resource-meta {
    margin-top: 5px;
}

.rm-public-intent-body {
    font-size: 16px;
    line-height: 1.55;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
}

.rm-mod-login-wrap {
    max-width: 520px;
    margin: auto;
}

.rm-mod-nav {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 20px;
}

.rm-mod-chip {
    padding: 9px 12px;
    border-radius: 999px;
    text-decoration: none;
    border: 1px solid rgba(255, 255, 255, .14);
    color: inherit;
    font-weight: 700;
}

.rm-mod-chip--reports {
    border-color: rgba(217, 119, 6, .38);
    background: rgba(217, 119, 6, .08);
    font-weight: 800;
}

.rm-mod-chip--reports-alert {
    border-color: rgba(220, 38, 38, .38);
    background: rgba(220, 38, 38, .08);
    font-weight: 800;
}

.rm-mod-chip--pending {
    border-color: rgba(217, 119, 6, .35);
}

.rm-mod-chip--verified {
    border-color: rgba(22, 163, 74, .35);
}

.rm-mod-chip--rejected {
    border-color: rgba(220, 38, 38, .35);
}

.rm-mod-chip--premium {
    border-color: rgba(214, 183, 122, .45);
}

.rm-mod-chip--hidden {
    border-color: rgba(220, 38, 38, .32);
}

.rm-mod-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 10px;
    margin-bottom: 24px;
}

.rm-mod-metrics--2 {
    grid-template-columns: repeat(2, minmax(0, 1fr));
}

.rm-mod-card {
    border: 1px solid rgba(214, 183, 122, .22);
    border-radius: 20px;
    padding: 18px;
    margin-bottom: 16px;
    background: rgba(255, 255, 255, .035);
}

.rm-mod-card--resource {
    margin: 0 0 16px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, .12);
}

.rm-mod-card-head {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    align-items: flex-start;
}

.rm-mod-card-head--resource {
    gap: 14px;
}

.rm-mod-card-kicker {
    font-size: 11px;
    color: #8f96a3;
    margin-bottom: 6px;
}

.rm-mod-card-title {
    margin: 0 0 6px;
    font-size: 20px;
}

.rm-mod-card-title--resource {
    margin: 0 0 8px;
}

.rm-mod-card-category {
    color: #9ca3af;
    font-size: 13px;
}

.rm-mod-card-desc {
    color: #9ca3af;
    line-height: 1.5;
    max-width: 620px;
}

.rm-mod-badges {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    font-size: 12px;
}

.rm-mod-badges--resource {
    gap: 12px;
    margin-top: 14px;
}

.rm-mod-badge--pending {
    color: #d97706;
    font-weight: 800;
}

.rm-mod-badge--ok {
    color: #16a34a;
    font-weight: 800;
}

.rm-mod-badge--bad {
    color: #dc2626;
    font-weight: 800;
}

.rm-mod-badge--active {
    color: #16a34a;
}

.rm-mod-badge--hidden {
    color: #dc2626;
}

.rm-mod-rating {
    font-weight: 800;
    white-space: nowrap;
}

.rm-mod-reason {
    margin-top: 16px;
    padding: 14px;
    border-radius: 14px;
    background: rgba(217, 119, 6, .07);
    border: 1px solid rgba(217, 119, 6, .20);
    line-height: 1.5;
}

.rm-mod-rejection {
    margin-top: 12px;
    padding: 11px 13px;
    border-radius: 12px;
    border: 1px solid rgba(220, 38, 38, .22);
    background: rgba(220, 38, 38, .07);
    color: #dc2626;
    font-size: 13px;
    line-height: 1.45;
}

.rm-mod-meta {
    margin-top: 10px;
    font-size: 11px;
    color: var(--muted);
}

.rm-mod-actions {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(145px, 1fr));
    gap: 9px;
    margin-top: 18px;
}

.rm-mod-actions--bulk {
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 8px;
}

.rm-mod-btn {
    width: 100%;
    min-height: 44px;
    border-radius: 12px;
    color: inherit;
    font-weight: 800;
    cursor: pointer;
    font-family: inherit;
}

.rm-mod-btn--ok {
    border: 1px solid rgba(22, 163, 74, .35);
    background: rgba(22, 163, 74, .10);
}

.rm-mod-btn--warn {
    border: 1px solid rgba(217, 119, 6, .35);
    background: rgba(217, 119, 6, .08);
}

.rm-mod-btn--danger {
    border: 1px solid rgba(220, 38, 38, .30);
    background: rgba(220, 38, 38, .07);
}

.rm-mod-btn--danger-strong {
    border: 1px solid rgba(220, 38, 38, .40);
    background: rgba(220, 38, 38, .12);
}

.rm-mod-btn--gold {
    border: 1px solid rgba(214, 183, 122, .42);
    background: rgba(214, 183, 122, .10);
}

.rm-mod-btn--neutral {
    border: 1px solid rgba(220, 38, 38, .28);
    background: rgba(220, 38, 38, .07);
}

.rm-mod-link {
    min-height: 42px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, .12);
    display: flex;
    align-items: center;
    justify-content: center;
    text-decoration: none;
    color: inherit;
    font-weight: 800;
}

.rm-mod-search {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
}

.rm-mod-search-input {
    flex: 1;
    min-width: 0;
    padding: 13px 14px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, .14);
    background: rgba(255, 255, 255, .04);
    color: inherit;
    font-size: 15px;
    font-family: inherit;
}

.rm-mod-search-btn {
    min-width: 92px;
    border-radius: 14px;
    border: 1px solid rgba(214, 183, 122, .35);
    background: rgba(214, 183, 122, .10);
    color: inherit;
    font-weight: 800;
    cursor: pointer;
    font-family: inherit;
}

.rm-mod-reject-form {
    grid-column: 1 / -1;
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 9px;
}

.rm-mod-reject-input {
    min-width: 0;
    min-height: 44px;
    box-sizing: border-box;
    padding: 0 13px;
    border-radius: 12px;
    border: 1px solid rgba(220, 38, 38, .30);
    background: rgba(255, 255, 255, .04);
    color: inherit;
    font-size: 14px;
    font-family: inherit;
}

.rm-mod-reject-btn {
    min-height: 44px;
    padding: 0 16px;
    border-radius: 12px;
    border: 1px solid rgba(220, 38, 38, .38);
    background: rgba(220, 38, 38, .10);
    color: inherit;
    font-weight: 800;
    cursor: pointer;
    font-family: inherit;
}

.rm-mod-bulk-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 12px;
}

.rm-mod-bulk-note {
    font-size: 12px;
    color: var(--muted);
}

.rm-mod-empty.card {
    display: block;
}

.rm-mod-result.card {
    display: block;
    padding: 20px;
}

.rm-mod-result-meta {
    margin-top: 8px;
}

.rm-mod-quick-card {
    border: 1px solid rgba(214, 183, 122, .22);
    border-radius: 18px;
    padding: 16px;
    margin-bottom: 12px;
    background: rgba(214, 183, 122, .05);
    box-shadow: 0 8px 24px rgba(0, 0, 0, .15);
}

.rm-mod-quick-title {
    font-size: 16px;
    font-weight: 800;
    color: #f0d69c;
    margin-bottom: 6px;
}

.rm-mod-quick-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
    flex-wrap: wrap;
}

.rm-mod-quick-btn {
    padding: 8px 16px;
    border: none;
    border-radius: 10px;
    font-size: 12px;
    font-weight: 800;
    cursor: pointer;
    font-family: inherit;
}

.rm-mod-quick-btn--approve {
    background: #16a34a;
    color: #fff;
}

.rm-mod-quick-btn--reject {
    background: #dc2626;
    color: #fff;
}

.rm-mod-empty-dashed {
    padding: 30px;
    text-align: center;
    color: #8f96a3;
    border: 1px dashed rgba(214, 183, 122, .25);
    border-radius: 18px;
}

.rm-status-badge {
    font-size: 11px;
    font-weight: 850;
}

.rm-status-badge--ok {
    color: #16a34a;
}

.rm-status-badge--bad {
    color: #dc2626;
}

.rm-status-badge--pending {
    color: #d97706;
}

.rm-contact-summary {
    display: flex;
    margin-bottom: 20px;
    padding: 15px 16px;
}

.rm-contact-summary-count {
    font-size: 24px;
    font-weight: 900;
}

.rm-contact-summary-copy {
    margin-top: 3px;
}

.rm-contact-card {
    display: block;
    padding: 18px;
    margin-bottom: 14px;
}

.rm-contact-layout {
    display: flex;
    align-items: flex-start;
    gap: 13px;
}

.rm-contact-body {
    flex: 1;
    min-width: 0;
}

.rm-contact-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 10px;
}

.rm-contact-title {
    font-size: 17px;
    overflow-wrap: anywhere;
}

.rm-contact-username {
    margin-top: 3px;
}

.rm-contact-message {
    margin-top: 12px;
    padding: 12px 13px;
    border-radius: 12px;
    background: rgba(0, 0, 0, .035);
    border: 1px solid rgba(0, 0, 0, .07);
    font-size: 14px;
    line-height: 1.5;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
}

.rm-contact-meta {
    margin-top: 10px;
    font-size: 10px;
}

.rm-contact-actions {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    margin-top: 16px;
}

.rm-contact-btn {
    width: 100%;
    min-height: 44px;
    border-radius: 12px;
    color: inherit;
    font-weight: 850;
    cursor: pointer;
    font-family: inherit;
}

.rm-contact-btn--accept {
    border: 1px solid rgba(22, 163, 74, .35);
    background: rgba(22, 163, 74, .10);
}

.rm-contact-btn--reject {
    border: 1px solid rgba(220, 38, 38, .30);
    background: rgba(220, 38, 38, .08);
}

.rm-contact-open-chat {
    margin-top: 16px;
    min-height: 46px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 15px;
    border-radius: 13px;
    text-decoration: none;
    color: var(--text);
    font-size: 13px;
    font-weight: 850;
    border: 1px solid rgba(214, 183, 122, .38);
    background: rgba(214, 183, 122, .08);
}

.rm-contact-profile-link {
    margin-top: 12px;
    min-height: 40px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0 13px;
    border-radius: 12px;
    border: 1px solid var(--line);
    text-decoration: none;
    color: inherit;
    font-size: 12px;
    font-weight: 800;
}

.rm-dialog-username {
    margin-top: 3px;
    overflow-wrap: anywhere;
}

.rm-home-professions-kicker {
    margin-top: 14px;
    font-size: 11px;
    font-weight: 900;
    letter-spacing: .08em;
    text-transform: uppercase;
    color: var(--muted);
}

.rm-pwa-panel {
    display: block;
    padding: 17px;
    margin-bottom: 16px;
}

.rm-pwa-hint {
    margin-top: 5px;
    line-height: 1.45;
}

.rm-home-start-card {
    display: block;
    padding: 18px;
}

.rm-home-start-title {
    font-size: 18px;
    line-height: 1.3;
}

.rm-home-start-copy {
    margin-top: 8px;
    line-height: 1.5;
}

.rm-home-search-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 46px;
    margin-top: 15px;
    border-radius: 14px;
    text-decoration: none;
    font-weight: 850;
}

.rm-flow-title {
    margin-top: 7px;
}

.rm-flow-copy {
    margin-top: 5px;
}

.rm-settings-card {
    display: block;
    padding: 20px;
}

.rm-settings-card--spaced {
    margin-top: 12px;
}

.rm-settings-title {
    margin-bottom: 12px;
}

.rm-settings-title--sm {
    margin-bottom: 6px;
}

.rm-settings-theme-btn,
.rm-settings-sound-btn {
    width: 100%;
    min-height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border: 1px solid rgba(214, 183, 122, .28);
    border-radius: 14px;
    background: rgba(214, 183, 122, .06);
    color: var(--gold-light);
    font-size: 14px;
    font-weight: 700;
}

.rm-settings-sound-btn {
    margin-top: 12px;
}

.rm-search-person-username {
    margin-top: 4px;
}

.rm-search-intent-box {
    margin-top: 10px;
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid rgba(214, 183, 122, .20);
    background: rgba(214, 183, 122, .07);
    font-size: 13px;
    line-height: 1.45;
    overflow-wrap: anywhere;
}

.rm-presence-badge {
    font-size: 10px;
    font-weight: 850;
}

.rm-presence-badge--online {
    color: #22c55e;
}

.rm-presence-badge--open {
    color: #16a34a;
}

.rm-presence-badge--closed {
    color: var(--muted);
    font-weight: 750;
}

.rm-resource-card {
    text-decoration: none;
    color: inherit;
    margin-bottom: 14px;
}

.rm-resource-card--premium {
    margin-bottom: 16px;
    border: 1px solid rgba(214, 183, 122, .55);
    background: linear-gradient(145deg, var(--card), var(--card-hover));
    box-shadow:
        0 10px 32px rgba(214, 183, 122, .14),
        0 0 0 1px rgba(214, 183, 122, .06);
    position: relative;
    overflow: hidden;
}

.rm-resource-card-shine {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, transparent, var(--gold), transparent);
}

.rm-resource-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 6px;
}

.rm-resource-verified-row {
    margin-top: 6px;
}

.rm-resource-hero-badges {
    display: flex;
    align-items: center;
    gap: 9px;
    flex-wrap: wrap;
    margin-bottom: 10px;
}

.rm-resource-rating-summary {
    display: block;
}

.rm-resource-owner-card {
    display: flex;
    width: 100%;
    box-sizing: border-box;
    margin-bottom: 16px;
    padding: 18px;
    text-decoration: none;
}

.rm-resource-owner-icon {
    flex: 0 0 auto;
}

.rm-resource-owner-content {
    min-width: 0;
}

.rm-resource-owner-kicker {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: .07em;
    color: var(--muted);
    font-weight: 800;
    margin-bottom: 5px;
}

.rm-resource-owner-meta {
    margin-top: 4px;
}

.rm-resource-owner-link {
    flex: 0 0 auto;
    align-self: center;
    min-height: 40px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0 13px;
    border-radius: 12px;
    text-decoration: none;
    color: var(--text);
    font-size: 12px;
    font-weight: 800;
    border: 1px solid rgba(214, 183, 122, .32);
    background: rgba(214, 183, 122, .08);
}

.rm-resource-favorite-btn {
    margin-top: 12px;
    min-height: 44px;
    padding: 0 15px;
    border-radius: 14px;
    border: 1px solid rgba(220, 38, 38, .25);
    background: rgba(220, 38, 38, .06);
    color: var(--text);
    font-weight: 800;
}

.rm-resource-favorite-status {
    margin-top: 7px;
    font-size: 12px;
    color: var(--muted);
}

.rm-resource-report-btn {
    margin-top: 10px;
    min-height: 42px;
    padding: 0 14px;
    border-radius: 14px;
    border: 1px solid rgba(217, 119, 6, .28);
    background: rgba(217, 119, 6, .06);
    color: var(--text);
    font-weight: 700;
}

.rm-resource-report-panel {
    display: none;
    margin-top: 12px;
    padding: 14px;
    border-radius: 16px;
    border: 1px solid rgba(217, 119, 6, .18);
    background: rgba(0, 0, 0, .03);
}

.rm-resource-report-label {
    font-size: 13px;
    font-weight: 800;
    margin-bottom: 8px;
}

.rm-resource-report-actions {
    display: flex;
    gap: 8px;
    margin-top: 10px;
    flex-wrap: wrap;
}

.rm-resource-report-submit {
    min-height: 40px;
    padding: 0 14px;
    border-radius: 12px;
    border: 1px solid rgba(220, 38, 38, .30);
    background: rgba(220, 38, 38, .08);
    color: var(--text);
    font-weight: 800;
}

.rm-resource-report-cancel {
    min-height: 40px;
    padding: 0 14px;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: rgba(0, 0, 0, .03);
    color: var(--text);
    font-weight: 700;
}

.rm-resource-report-status {
    margin-top: 9px;
    font-size: 12px;
    color: var(--muted);
}

.rm-resource-rating-block {
    margin-top: 18px;
}

.rm-resource-rating-kicker {
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 8px;
    font-weight: 700;
    letter-spacing: .05em;
}

.rm-resource-stars {
    display: flex;
    gap: 8px;
    align-items: center;
}

.rm-resource-star-btn {
    font-size: 28px;
    background: none;
    border: 0;
    cursor: pointer;
    padding: 2px;
}

.rm-resource-vote-status {
    margin-top: 7px;
    font-size: 12px;
    color: var(--muted);
}

.rm-resource-section {
    display: block;
    width: 100%;
    box-sizing: border-box;
    margin-bottom: 16px;
    padding: 20px;
    position: relative;
    overflow: hidden;
}

.rm-resource-section--premium {
    border: 1px solid rgba(214, 183, 122, .55);
    background: linear-gradient(145deg, var(--card), var(--card-hover));
    box-shadow: 0 12px 38px rgba(214, 183, 122, .14);
}

.rm-resource-section--plain {
    border: 1px solid rgba(0, 0, 0, .07);
}

.rm-resource-section-kicker {
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: .08em;
    font-weight: 700;
}

.rm-resource-section-kicker--contacts {
    margin-bottom: 12px;
}

.rm-resource-description {
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    font-size: 16px;
    line-height: 1.6;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    word-break: break-word;
}

.rm-resource-contact-line {
    display: block;
    width: 100%;
    margin-bottom: 10px;
    line-height: 1.5;
    overflow-wrap: anywhere;
}

.rm-resource-contact-line--last {
    margin-bottom: 0;
}

.rm-resource-contact-actions {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    width: 100%;
    gap: 10px;
    margin-top: 18px;
}

.rm-resource-contact-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-width: 0;
    min-height: 48px;
    padding: 0 10px;
    box-sizing: border-box;
    border-radius: 14px;
    text-decoration: none;
    font-weight: 800;
    color: var(--text);
}

.rm-resource-contact-btn--gold {
    border: 1px solid rgba(214, 183, 122, .38);
    background: rgba(214, 183, 122, .08);
}

.rm-resource-contact-btn--neutral {
    border: 1px solid var(--line);
    background: rgba(0, 0, 0, .035);
}

.rm-resource-id-value {
    font-size: 18px;
    font-weight: 700;
}

.rm-my-resource-card {
    display: block;
    margin-bottom: 16px;
    padding: 18px;
}

.rm-my-resource-layout {
    display: flex;
    align-items: flex-start;
    gap: 14px;
}

.rm-my-resource-icon {
    flex: 0 0 auto;
}

.rm-my-resource-body {
    flex: 1;
    min-width: 0;
}

.rm-my-resource-head {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: flex-start;
}

.rm-my-resource-title-wrap {
    min-width: 0;
}

.rm-my-resource-title {
    font-size: 18px;
    line-height: 1.25;
    margin-bottom: 4px;
}

.rm-my-resource-category {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: .06em;
}

.rm-my-resource-rating {
    flex: 0 0 auto;
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
}

.rm-my-resource-desc {
    margin-top: 10px;
    line-height: 1.5;
    overflow-wrap: anywhere;
}

.rm-my-resource-badges {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
    margin-top: 12px;
}

.rm-my-resource-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    margin-top: 14px;
}

.rm-my-resource-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 42px;
    padding: 0 14px;
    border-radius: 12px;
    text-decoration: none;
    font-size: 13px;
    font-weight: 800;
    color: var(--text);
}

.rm-my-resource-action--gold {
    font-weight: 850;
    color: var(--gold-light);
    border: 1px solid rgba(214, 183, 122, .48);
    background: rgba(214, 183, 122, .10);
}

.rm-my-resource-action--edit {
    border: 1px solid rgba(214, 183, 122, .35);
    background: rgba(214, 183, 122, .08);
}

.rm-my-resource-action--neutral {
    font-weight: 700;
    border: 1px solid var(--line);
    background: rgba(0, 0, 0, .03);
}

.rm-my-resource-note {
    margin-top: 10px;
    padding: 10px 12px;
    border-radius: 12px;
    font-size: 12px;
    line-height: 1.5;
}

.rm-my-resource-note--hidden {
    border: 1px solid rgba(220, 38, 38, .18);
    background: rgba(220, 38, 38, .05);
    color: #dc2626;
}

.rm-my-resource-note--rejected {
    border: 1px solid rgba(220, 38, 38, .22);
    background: rgba(220, 38, 38, .07);
    color: #dc2626;
}

.rm-resource-mod-badge {
    font-size: 11px;
    font-weight: 800;
}

.rm-resource-mod-badge--hidden {
    color: #dc2626;
}

.rm-resource-mod-badge--approved {
    color: #16a34a;
}

.rm-resource-mod-badge--rejected {
    color: #dc2626;
}

.rm-resource-mod-badge--pending {
    color: #d97706;
}

.rm-promo-pending {
    display: block;
    padding: 18px;
    margin-top: 16px;
    border: 1px solid rgba(214, 183, 122, .32);
    background: rgba(214, 183, 122, .07);
}

.rm-promo-pending-copy {
    margin-top: 6px;
}

.rm-promo-form {
    margin-top: 16px;
}

.rm-promo-submit {
    width: 100%;
    min-height: 52px;
    border-radius: 15px;
    border: 1px solid rgba(214, 183, 122, .55);
    background: linear-gradient(135deg, rgba(214, 183, 122, .20), rgba(214, 183, 122, .08));
    color: var(--text);
    font-weight: 900;
}

.rm-promo-preview {
    display: block;
    overflow: hidden;
    padding: 0;
    border: 1px solid rgba(214, 183, 122, .52);
    background:
        radial-gradient(circle at 100% 0%, rgba(126, 212, 228, .15), transparent 34%),
        linear-gradient(145deg, rgba(20, 23, 30, .99), rgba(10, 12, 17, .99));
    box-shadow:
        0 22px 60px rgba(0, 0, 0, .30),
        0 0 38px rgba(214, 183, 122, .08);
}

.rm-promo-preview-head {
    padding: 15px 20px;
    border-bottom: 1px solid rgba(214, 183, 122, .24);
    color: var(--gold-light);
    font-size: 12px;
    font-weight: 900;
    letter-spacing: .12em;
    text-transform: uppercase;
}

.rm-promo-preview-body {
    padding: 22px 20px;
}

.rm-promo-preview-category {
    color: var(--muted);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: .08em;
    text-transform: uppercase;
}

.rm-promo-preview-title {
    margin: 8px 0 12px;
    font-size: 24px;
    line-height: 1.18;
    overflow-wrap: anywhere;
}

.rm-promo-preview-text {
    color: var(--text);
    font-size: 14px;
    line-height: 1.55;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
}

.rm-promo-preview-address {
    margin-top: 16px;
    color: var(--muted);
    font-size: 13px;
    overflow-wrap: anywhere;
}

.rm-promo-preview-footer {
    margin-top: 20px;
    padding-top: 15px;
    border-top: 1px solid rgba(214, 183, 122, .18);
    color: var(--gold-light);
    font-size: 12px;
    font-weight: 850;
}

.rm-promo-preview-domain {
    margin-top: 7px;
    color: var(--muted);
    font-size: 11px;
}

.rm-promo-target-card {
    display: block;
    padding: 18px;
    margin-top: 16px;
}

.rm-promo-target-copy {
    margin-top: 7px;
}

.rm-promo-target-note {
    margin-top: 10px;
    line-height: 1.5;
}

@media (hover:hover) {
    .ui-button:hover {
        opacity: .94;
    }
}

@media (prefers-reduced-motion: reduce) {
    .hero::before,
    .hero::after {
        animation: none;
    }

    .grid .card {
        animation-delay: 0s !important;
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
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="resursmap-asset-version" content="{asset_version}">
{site_head}
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:;">
<meta name="referrer" content="no-referrer">
<meta http-equiv="X-Content-Type-Options" content="nosniff">
<meta http-equiv="X-Frame-Options" content="DENY">
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

<script src="{splash_js}" defer></script>
<script src="{notification_sound_js}" defer></script>
<script src="{nav_badge_js}" defer></script>
<script src="{theme_toggle_js}" defer></script>

</body>
</html>"#,
        title = escape_html(title),
        asset_version = STATIC_ASSET_VERSION,
        site_head = site_head_links(),
        style = base_style(),
        head_extra = head_extra_html,
        body_before_main = body_before_main_html,
        main = main_html,
        bottom_nav = bottom_nav_html,
        body_after = body_after_html,
        splash_js = static_asset("splash.js"),
        notification_sound_js = static_asset("notification-sound.js"),
        nav_badge_js = static_asset("nav-badge.js"),
        theme_toggle_js = static_asset("theme-toggle.js"),
    )
}

pub(crate) struct AuthPageParams<'a> {
    pub document_title: &'a str,
    pub heading: &'a str,
    pub subtitle: &'a str,
    pub body_html: &'a str,
    pub footer_html: &'a str,
    pub script_html: &'a str,
}

pub(crate) fn auth_support_scripts() -> String {
    format!(
        r#"<script src="{auth_forms_js}" defer></script>"#,
        auth_forms_js = static_asset("auth-forms.js"),
    )
}

pub(crate) fn render_auth_page(params: AuthPageParams<'_>) -> String {
    let main_html = format!(
        r#"<div class="rm-auth-wrap">
    <section class="card rm-auth-card">
        <h1 class="rm-auth-title">{heading}</h1>
        <p class="rm-auth-subtitle">{subtitle}</p>
        {body}
        <p id="auth-status" class="rm-auth-status" role="status" aria-live="polite"></p>
        {footer}
    </section>
    <div class="rm-auth-back">
        <a href="/app">&larr; Вернуться на карту</a>
    </div>
</div>"#,
        heading = escape_html(params.heading),
        subtitle = escape_html(params.subtitle),
        body = params.body_html,
        footer = params.footer_html,
    );

    page_document(
        params.document_title,
        "",
        "",
        &main_html,
        "",
        &format!(
            "{support_scripts}{script_html}",
            support_scripts = auth_support_scripts(),
            script_html = params.script_html,
        ),
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
    bottom_nav_with_badge(active, 0)
}

pub(crate) fn bottom_nav_with_badge(active: &str, unread_count: i64) -> String {
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

    <a class="{profile_class}" href="/app/me" data-nav-attention-link>
        {nav_user}
        {unread_badge}
        <span>Профиль</span>
    </a>

    <a class="{menu_class}" href="/app/menu">
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
        unread_badge = if unread_count > 0 {
            let label = if unread_count > 99 {
                "99+".to_string()
            } else {
                unread_count.to_string()
            };
            format!(r#"<span class="nav-badge">{label}</span>"#)
        } else {
            String::new()
        },
    )
}

pub(crate) fn topbar(subtitle: &str, _icon_name: &str) -> String {
    format!(
        r#"
<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{logo}</div>
        <div>
            <div class="brand-name">ResursMap</div>
            <div class="brand-sub">{subtitle}</div>
        </div>
    </a>

    <a class="topbar-account"
       href="/app/me"
       aria-label="Открыть профиль">
        <span class="topbar-account-icon">
            {user_icon}
        </span>
        <span class="topbar-account-label">
            Профиль
        </span>
    </a>
</header>
"#,
        logo = brand_logo(),
        user_icon = icon("user"),
        subtitle = escape_html(subtitle),
    )
}

pub(crate) fn back_link(href: &str, label: &str, icon_name: &str) -> String {
    format!(
        r#"<a href="{href}" class="rm-back-link">
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

pub(crate) fn people_result_card(
    href: &str,
    display_name_html: &str,
    username_html: &str,
    intent_html: &str,
    contact_html: &str,
    is_ready: bool,
) -> String {
    format!(
        r#"
<a href="{href}" class="card card--result">

    <div class="card-icon">
        {user_icon}
    </div>

    <div class="card-content">

        <div class="card-title card-title--people">
            {ready_dot}
            {display_name}
        </div>

        {username_html}

        {intent_html}

        <div class="rm-card-row">
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
        ready_dot = if is_ready {
            r#"<span class="rm-ready-dot"></span>"#
        } else {
            ""
        },
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
<a href="{href}" class="card card--result">

    <div class="card-icon">{resource_icon}</div>

    <div class="card-content">

        <div class="rm-card-header">
            <div class="card-title card-title--lg">
                {title}
            </div>

            <div class="rm-card-rating">
                ⭐ {rating:.1} · {votes}
            </div>
        </div>

        <div class="card-meta card-meta--mt-4">
            {category}
        </div>

        <div class="card-meta card-meta--desc">
            {description}
        </div>

        <div class="card-meta card-meta--mt-9">
            📍 {location}
        </div>

        <div class="card-meta card-meta--mt-4 card-meta--wrap">
            {address}
        </div>

        <div class="rm-card-row--badges">
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
            r#"<div class="card-meta card-meta--mt-8 card-meta--wrap">
            📍 {addr}
        </div>
"#,
            addr = escape_html(addr),
        ),
        _ => String::new(),
    };

    format!(
        r#"<a href="{href}" class="card card--result">
    <div class="card-icon">{resource_icon}</div>

    <div class="card-content">
        <div class="card-title">{title}</div>

        <div class="card-meta card-meta--mt-4">{category}</div>

        <div class="card-meta card-meta--desc">{description}</div>

        {address_html}
        <div class="card-meta card-meta--mt-8">⭐ {rating:.1} · {votes} голосов</div>

        <div class="rm-card-row--badges">
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

pub(crate) fn premium_badge_html(variant: &str) -> &'static str {
    match variant {
        "compact" => r#"<span class="rm-premium-badge rm-premium-badge--compact">★ Премиум</span>"#,
        "admin" => r#"<span class="rm-premium-badge rm-premium-badge--admin">★ Премиум</span>"#,
        _ => r#"<span class="rm-premium-badge">★ Премиум</span>"#,
    }
}

pub(crate) fn verified_badge_html(compact: bool) -> &'static str {
    if compact {
        r#"<span class="rm-verified-badge rm-verified-badge--compact">✓ Проверен</span>"#
    } else {
        r#"<span class="rm-verified-badge">✓ Проверен</span>"#
    }
}

pub(crate) fn empty_state_action(href: &str, label: &str) -> String {
    format!(
        r#"<a class="rm-empty-action ui-button" href="{href}">{label}</a>"#,
        href = escape_html(href),
        label = escape_html(label),
    )
}

pub(crate) fn empty_state_card(title: &str, description_html: &str) -> String {
    empty_state_card_with_actions(title, description_html, "")
}

pub(crate) fn empty_state_card_with_actions(
    title: &str,
    description_html: &str,
    actions_html: &str,
) -> String {
    let actions = if actions_html.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="rm-empty-state-actions">{actions_html}</div>"#)
    };

    format!(
        r#"
<div class="card rm-empty-state">
    <div class="card-content">
        <div class="card-title">{title}</div>
        <div class="card-meta">{description}</div>
        {actions}
    </div>
</div>
"#,
        title = escape_html(title),
        description = description_html,
        actions = actions,
    )
}

pub(crate) fn workflow_status_label(status: &str) -> &'static str {
    match status {
        "pending" => "Ожидает",
        "closed" => "Закрыта",
        "escalated" => "Передана выше",
        "rejected" => "Отклонена",
        "approved" => "Одобрен",
        "active" => "Активна",
        "suspended" => "Приостановлена",
        _ => "",
    }
}

pub(crate) fn workflow_status_label_or_raw(status: &str) -> String {
    let label = workflow_status_label(status);
    if label.is_empty() {
        status.to_string()
    } else {
        label.to_string()
    }
}

pub(crate) fn moderation_queue_badge(status: &str) -> &'static str {
    match status {
        "approved" => r#"<span class="rm-mod-badge rm-mod-badge--ok">✓ Одобрен</span>"#,
        "rejected" => r#"<span class="rm-mod-badge rm-mod-badge--bad">✕ Отклонён</span>"#,
        _ => r#"<span class="rm-mod-badge rm-mod-badge--pending">● Ожидает проверки</span>"#,
    }
}

pub(crate) fn report_queue_badge(status: &str) -> &'static str {
    match status {
        "pending" => r#"<span class="rm-mod-badge rm-mod-badge--pending">● Ожидает</span>"#,
        _ => r#"<span class="rm-mod-badge rm-mod-badge--ok">✓ Закрыта</span>"#,
    }
}

pub(crate) fn resource_visibility_badge(active: i64) -> &'static str {
    if active == 1 {
        r#"<span class="rm-mod-badge rm-mod-badge--active">● Активен</span>"#
    } else {
        r#"<span class="rm-mod-badge rm-mod-badge--hidden">● Скрыт</span>"#
    }
}

pub(crate) fn resource_visibility_with_status(active: i64, status: &str) -> String {
    let safe_status = escape_html(status);
    if active == 1 {
        format!(
            r#"<span class="rm-mod-badge rm-mod-badge--active">● Активен · {safe_status}</span>"#
        )
    } else {
        format!(
            r#"<span class="rm-mod-badge rm-mod-badge--hidden">● Скрыт · {safe_status}</span>"#
        )
    }
}

pub(crate) fn moderator_level_badge(level: i64) -> String {
    let (text, class) = match level {
        1 => ("Уровень 1 · Помощник группы", "rm-mod-level-badge--1"),
        2 => ("Уровень 2 · Администратор города", "rm-mod-level-badge--2"),
        3 => ("Уровень 3 · Администратор страны", "rm-mod-level-badge--3"),
        4 => ("Уровень 4 · Администратор континента", "rm-mod-level-badge--4"),
        5 => ("Уровень 5 · Владелец ResursMap", "rm-mod-level-badge--5"),
        _ => return String::new(),
    };

    format!(
        r#"<a href="/app/center" class="rm-mod-level-badge {class}">{text}</a>"#,
        class = class,
        text = text,
    )
}

pub(crate) fn contact_request_status_badge(status: &str) -> &'static str {
    match status {
        "accepted" => r#"<span class="rm-status-badge rm-status-badge--ok">✓ Принят</span>"#,
        "rejected" => r#"<span class="rm-status-badge rm-status-badge--bad">✕ Отклонён</span>"#,
        _ => r#"<span class="rm-status-badge rm-status-badge--pending">● Ожидает ответа</span>"#,
    }
}

pub(crate) fn my_resource_moderation_badge(is_active: i64, status: &str) -> &'static str {
    if is_active == 0 && status != "rejected" {
        r#"<span class="rm-resource-mod-badge rm-resource-mod-badge--hidden">⚫ Скрыт</span>"#
    } else {
        match status {
            "approved" => {
                r#"<span class="rm-resource-mod-badge rm-resource-mod-badge--approved">🟢 Одобрен</span>"#
            }
            "rejected" => {
                r#"<span class="rm-resource-mod-badge rm-resource-mod-badge--rejected">🔴 Отклонён</span>"#
            }
            _ => r#"<span class="rm-resource-mod-badge rm-resource-mod-badge--pending">🟡 На проверке</span>"#,
        }
    }
}

pub(crate) fn resource_card_link_class(premium: bool) -> &'static str {
    if premium {
        "card rm-resource-card rm-resource-card--premium"
    } else {
        "card rm-resource-card"
    }
}

pub(crate) fn resource_detail_section_class(premium: bool) -> &'static str {
    if premium {
        "card rm-resource-section rm-resource-section--premium"
    } else {
        "card rm-resource-section rm-resource-section--plain"
    }
}

fn admin_ops_styles() -> &'static str {
    r#"
.rm-admin-ops {
    --ops-green: #46d39a;
    --ops-gold: #d6b77a;
    --ops-red: #ff7d7d;
    --ops-blue: #72aaff;
    --ops-line: rgba(255, 255, 255, .10);
    --ops-muted: #9aaba2;
    width: min(1100px, 100%);
    margin: 0 auto;
}
.rm-admin-ops .topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
}
.rm-admin-ops .back {
    color: var(--text);
    text-decoration: none;
    font-weight: 850;
}
.rm-admin-ops .protected {
    padding: 7px 10px;
    border: 1px solid rgba(70, 211, 154, .32);
    border-radius: 999px;
    color: var(--ops-green);
    font-size: 11px;
    font-weight: 900;
}
.rm-admin-ops .hero {
    padding: 26px;
    border: 1px solid rgba(70, 211, 154, .26);
    border-radius: 25px;
    background:
        linear-gradient(135deg, rgba(70, 211, 154, .12), rgba(16, 26, 22, .96) 54%),
        var(--card);
    box-shadow: 0 24px 65px rgba(0, 0, 0, .30);
}
.rm-admin-ops .kicker {
    color: var(--ops-green);
    font-size: 12px;
    font-weight: 900;
    letter-spacing: .12em;
    text-transform: uppercase;
}
.rm-admin-ops h1 {
    margin: 9px 0;
    font-size: clamp(30px, 6vw, 54px);
    line-height: 1;
    letter-spacing: -.04em;
}
.rm-admin-ops .hero p {
    max-width: 680px;
    margin: 0;
    color: #c4d1ca;
    line-height: 1.55;
}
.rm-admin-ops .group-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 16px;
}
.rm-admin-ops .group-meta span {
    padding: 6px 9px;
    border: 1px solid var(--ops-line);
    border-radius: 999px;
    color: var(--ops-muted);
    font-size: 11px;
}
.rm-admin-ops .telegram {
    display: inline-flex;
    margin-top: 15px;
    color: var(--ops-blue);
    text-decoration: none;
    font-weight: 850;
}
.rm-admin-ops .metrics {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
    margin: 16px 0 26px;
}
.rm-admin-ops .metric {
    padding: 17px;
    border: 1px solid var(--ops-line);
    border-radius: 18px;
    background: rgba(16, 26, 22, .88);
}
.rm-admin-ops .metric strong {
    display: block;
    color: var(--ops-green);
    font-size: 25px;
}
.rm-admin-ops .metric span {
    color: var(--ops-muted);
    font-size: 12px;
}
.rm-admin-ops .section-head {
    display: flex;
    justify-content: space-between;
    align-items: end;
    gap: 12px;
    margin: 24px 2px 12px;
    border-bottom: 0;
}
.rm-admin-ops .section-head h2 {
    margin: 0;
    font-size: 20px;
}
.rm-admin-ops .section-head span {
    color: var(--ops-muted);
    font-size: 12px;
}
.rm-admin-ops .grid {
    display: grid;
    gap: 11px;
}
.rm-admin-ops .event-card,
.rm-admin-ops .report-card,
.rm-admin-ops .risk-row {
    padding: 16px;
    border: 1px solid var(--ops-line);
    border-radius: 17px;
    background: linear-gradient(145deg, rgba(20, 34, 28, .96), rgba(11, 20, 16, .96));
}
.rm-admin-ops .event-head {
    display: flex;
    justify-content: space-between;
    gap: 12px;
}
.rm-admin-ops .event-card p,
.rm-admin-ops .report-card p {
    margin: 10px 0;
    color: #cad5cf;
}
.rm-admin-ops .report-actions {
    margin-top: 14px;
    padding-top: 13px;
    border-top: 1px solid var(--ops-line);
}
.rm-admin-ops .report-actions form {
    display: grid;
    gap: 9px;
}
.rm-admin-ops .report-actions input {
    width: 100%;
    min-height: 43px;
    padding: 0 12px;
    border: 1px solid var(--ops-line);
    border-radius: 11px;
    color: var(--text);
    background: #0b1511;
    font: inherit;
}
.rm-admin-ops .action-buttons {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 7px;
}
.rm-admin-ops .action-buttons button {
    min-height: 41px;
    padding: 7px;
    border-radius: 10px;
    font-weight: 850;
    cursor: pointer;
}
.rm-admin-ops .action-buttons .close {
    color: var(--text);
    border: 1px solid var(--ops-line);
    background: rgba(255, 255, 255, .05);
}
.rm-admin-ops .action-buttons .escalate {
    color: var(--ops-gold);
    border: 1px solid rgba(214, 183, 122, .32);
    background: rgba(214, 183, 122, .08);
}
.rm-admin-ops .action-buttons .reject {
    color: var(--ops-red);
    border: 1px solid rgba(255, 125, 125, .30);
    background: rgba(255, 125, 125, .08);
}
.rm-admin-ops .risk,
.rm-admin-ops .report-status {
    padding: 5px 8px;
    border: 1px solid var(--ops-line);
    border-radius: 999px;
    color: var(--ops-gold);
    font-size: 10px;
    font-weight: 900;
}
.rm-admin-ops .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
}
.rm-admin-ops .meta span {
    color: var(--ops-muted);
    font-size: 11px;
}
.rm-admin-ops .risk-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
}
.rm-admin-ops .risk-row small {
    display: block;
    margin-top: 4px;
    color: var(--ops-muted);
}
.rm-admin-ops .risk-values {
    display: flex;
    gap: 7px;
}
.rm-admin-ops .risk-values span {
    padding: 6px 8px;
    border: 1px solid var(--ops-line);
    border-radius: 10px;
    color: var(--ops-gold);
    font-size: 11px;
}
.rm-admin-ops .empty {
    padding: 28px 16px;
    border: 1px dashed var(--ops-line);
    border-radius: 17px;
    color: var(--ops-muted);
    text-align: center;
}
@media (max-width: 700px) {
    .rm-admin-ops .hero { padding: 21px 17px; }
    .rm-admin-ops .metrics { grid-template-columns: 1fr; }
    .rm-admin-ops .event-head,
    .rm-admin-ops .risk-row {
        align-items: flex-start;
        flex-direction: column;
    }
    .rm-admin-ops .action-buttons { grid-template-columns: 1fr; }
}
.rm-admin-ops.rm-admin-ops--city {
    width: min(1180px, 100%);
}
.rm-admin-ops--city .hero {
    border-color: rgba(85, 228, 154, .28);
    background: linear-gradient(145deg, rgba(21, 50, 38, .98), rgba(8, 20, 15, .98));
}
.rm-admin-ops--city .eyebrow {
    color: #55e49a;
    font-size: 12px;
    letter-spacing: .14em;
    text-transform: uppercase;
}
.rm-admin-ops--city .hero-top,
.rm-admin-ops--city .row,
.rm-admin-ops--city .list-row {
    display: flex;
    justify-content: space-between;
    gap: 14px;
}
.rm-admin-ops--city .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 9px;
    margin-top: 20px;
}
.rm-admin-ops--city .button {
    padding: 11px 14px;
    border: 1px solid var(--ops-line);
    border-radius: 13px;
    color: var(--text);
    background: rgba(17, 37, 28, .88);
    text-decoration: none;
}
.rm-admin-ops--city .button.primary {
    color: #062015;
    background: #55e49a;
    border-color: #55e49a;
    font-weight: 800;
}
.rm-admin-ops--city .metrics {
    grid-template-columns: repeat(6, minmax(0, 1fr));
}
.rm-admin-ops--city .metric strong {
    color: #55e49a;
}
.rm-admin-ops--city .grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
}
.rm-admin-ops--city .card,
.rm-admin-ops--city .panel {
    padding: 16px;
    border: 1px solid var(--ops-line);
    border-radius: 18px;
    background: linear-gradient(145deg, rgba(18, 38, 29, .96), rgba(8, 19, 14, .96));
}
.rm-admin-ops--city .panel {
    padding: 0;
    overflow: hidden;
}
.rm-admin-ops--city .list-row {
    align-items: center;
    padding: 14px 16px;
    border-bottom: 1px solid var(--ops-line);
}
.rm-admin-ops--city .list-row:last-child {
    border-bottom: 0;
}
.rm-admin-ops--city .list-row small {
    display: block;
    margin-top: 5px;
}
.rm-admin-ops--city .right {
    text-align: right;
}
.rm-admin-ops--city .badge {
    display: inline-block;
    padding: 5px 8px;
    border: 1px solid var(--ops-line);
    border-radius: 999px;
    color: #55e49a;
    font-size: 12px;
}
.rm-admin-ops--city .badge.warning {
    color: var(--ops-gold);
}
.rm-admin-ops--city .badge.danger {
    color: var(--ops-red);
}
.rm-admin-ops--city .meta a {
    color: #55e49a;
}
.rm-admin-ops--city footer {
    margin-top: 28px;
    color: var(--ops-muted);
    font-size: 12px;
}
.rm-admin-ops.rm-admin-ops--geo {
    width: min(1180px, 100%);
}
.rm-admin-ops--geo .protected {
    border-color: rgba(214, 183, 122, .35);
    color: var(--ops-gold);
    letter-spacing: .08em;
}
.rm-admin-ops--geo .hero {
    position: relative;
    overflow: hidden;
    border-color: rgba(214, 183, 122, .25);
    background:
        linear-gradient(135deg, rgba(214, 183, 122, .14), rgba(17, 21, 28, .94) 48%),
        var(--card);
}
.rm-admin-ops--geo .kicker {
    color: var(--ops-gold);
}
.rm-admin-ops--geo .metrics {
    grid-template-columns: repeat(5, minmax(0, 1fr));
}
.rm-admin-ops--geo .metric strong {
    color: var(--ops-gold);
}
.rm-admin-ops--geo .search {
    position: sticky;
    top: 8px;
    z-index: 10;
    display: flex;
    gap: 10px;
    padding: 12px;
    margin: 0 0 18px;
    border: 1px solid var(--ops-line);
    border-radius: 18px;
    background: rgba(7, 9, 13, .86);
    backdrop-filter: blur(18px);
}
.rm-admin-ops--geo .search input {
    flex: 1;
    min-width: 0;
    min-height: 48px;
    padding: 0 16px;
    border: 1px solid var(--ops-line);
    border-radius: 14px;
    color: var(--text);
    background: rgba(22, 27, 36, .92);
    font: inherit;
}
.rm-admin-ops--geo .search button {
    min-height: 48px;
    padding: 0 20px;
    border: 0;
    border-radius: 14px;
    color: #17120a;
    background: linear-gradient(135deg, #ead29f, var(--ops-gold));
    font-weight: 900;
    cursor: pointer;
}
.rm-admin-ops--geo .city-list {
    display: grid;
    gap: 12px;
}
.rm-admin-ops--geo .city-card {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 260px;
    gap: 16px;
    padding: 18px;
    border: 1px solid var(--ops-line);
    border-radius: 20px;
    background: linear-gradient(145deg, rgba(22, 27, 36, .95), rgba(13, 17, 23, .96));
}
.rm-admin-ops--geo .city-title-row {
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: 12px;
}
.rm-admin-ops--geo .city-card h3 {
    margin: 0;
    font-size: 20px;
}
.rm-admin-ops--geo .native {
    display: block;
    margin-top: 3px;
    color: var(--ops-muted);
    font-size: 13px;
}
.rm-admin-ops--geo .location {
    margin-top: 8px;
    color: #c7ccd5;
}
.rm-admin-ops--geo .city-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
    margin-top: 12px;
}
.rm-admin-ops--geo .city-meta span {
    padding: 5px 8px;
    border: 1px solid var(--ops-line);
    border-radius: 999px;
    color: var(--ops-muted);
    font-size: 11px;
}
.rm-admin-ops--geo .group-status {
    white-space: nowrap;
    padding: 6px 9px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 900;
}
.rm-admin-ops--geo .group-status.connected {
    color: var(--ops-green);
    background: rgba(70, 211, 154, .10);
    border: 1px solid rgba(70, 211, 154, .30);
}
.rm-admin-ops--geo .group-status.disabled {
    color: var(--ops-red);
    background: rgba(255, 125, 125, .09);
    border: 1px solid rgba(255, 125, 125, .28);
}
.rm-admin-ops--geo .group-status.missing {
    color: var(--ops-muted);
    background: rgba(255, 255, 255, .04);
    border: 1px solid var(--ops-line);
}
.rm-admin-ops--geo .group-panel {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 14px;
    border: 1px solid var(--ops-line);
    border-radius: 15px;
    background: rgba(0, 0, 0, .18);
}
.rm-admin-ops--geo .group-panel small {
    margin-top: 5px;
    color: var(--ops-muted);
}
.rm-admin-ops--geo .group-editor {
    margin-top: 14px;
    padding-top: 12px;
    border-top: 1px solid var(--ops-line);
}
.rm-admin-ops--geo .group-editor summary {
    cursor: pointer;
    color: var(--ops-gold);
    font-size: 13px;
    font-weight: 850;
}
.rm-admin-ops--geo .group-form {
    display: grid;
    gap: 10px;
    margin-top: 12px;
}
.rm-admin-ops--geo .group-form label {
    display: grid;
    gap: 5px;
    color: var(--ops-muted);
    font-size: 11px;
    font-weight: 750;
}
.rm-admin-ops--geo .group-form input:not([type="hidden"]):not([type="checkbox"]) {
    width: 100%;
    min-height: 42px;
    padding: 0 11px;
    border: 1px solid var(--ops-line);
    border-radius: 11px;
    color: var(--text);
    background: #0c1016;
    font: inherit;
}
.rm-admin-ops--geo .group-form .active-control {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text);
}
.rm-admin-ops--geo .group-form button {
    min-height: 43px;
    border: 0;
    border-radius: 11px;
    color: #17120a;
    background: linear-gradient(135deg, #ead29f, var(--ops-gold));
    font-weight: 900;
    cursor: pointer;
}
.rm-admin-ops--geo .telegram-link {
    margin-top: 12px;
    color: var(--ops-blue);
    text-decoration: none;
    font-size: 13px;
    font-weight: 800;
}
.rm-admin-ops--geo .empty-state {
    padding: 42px 20px;
    border: 1px dashed var(--ops-line);
    border-radius: 22px;
    text-align: center;
    color: var(--ops-muted);
}
.rm-admin-ops--geo .empty-state h2 {
    margin: 8px 0;
    color: var(--text);
}
.rm-admin-ops--geo .empty-state p {
    margin: 0;
}
.rm-admin-ops--geo .empty-icon {
    font-size: 38px;
}
@media (max-width: 900px) {
    .rm-admin-ops--city .metrics { grid-template-columns: repeat(3, minmax(0, 1fr)); }
}
@media (max-width: 760px) {
    .rm-admin-ops--geo .metrics { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .rm-admin-ops--geo .metric:last-child { grid-column: 1 / -1; }
    .rm-admin-ops--geo .search { flex-direction: column; align-items: stretch; }
    .rm-admin-ops--geo .city-card { grid-template-columns: 1fr; }
}
@media (max-width: 650px) {
    .rm-admin-ops--city .metrics,
    .rm-admin-ops--city .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
    .rm-admin-ops--city .list-row,
    .rm-admin-ops--city .hero-top { align-items: flex-start; }
}
@media (max-width: 430px) {
    .rm-admin-ops--city .metrics,
    .rm-admin-ops--city .grid { grid-template-columns: 1fr; }
    .rm-admin-ops--city .list-row { flex-direction: column; }
    .rm-admin-ops--city .right { text-align: left; }
}
.rm-admin-ops.rm-admin-ops--helpers {
    width: min(900px, 100%);
}
.rm-admin-ops--helpers .create {
    margin: 15px 0;
    padding: 18px;
    border: 1px solid var(--ops-line);
    border-radius: 20px;
    background: linear-gradient(145deg, rgba(19, 40, 30, .96), rgba(9, 23, 16, .96));
}
.rm-admin-ops--helpers .fields {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
}
.rm-admin-ops--helpers label {
    display: grid;
    gap: 6px;
    color: var(--ops-muted);
    font-size: 12px;
}
.rm-admin-ops--helpers input,
.rm-admin-ops--helpers select,
.rm-admin-ops--helpers button {
    width: 100%;
    border: 1px solid var(--ops-line);
    border-radius: 12px;
    padding: 11px;
    background: #0b1912;
    color: var(--text);
    font: inherit;
}
.rm-admin-ops--helpers button {
    cursor: pointer;
    color: #062015;
    background: #58e59e;
    font-weight: 800;
}
.rm-admin-ops--helpers button.warning {
    color: #231900;
    background: #e8bd62;
}
.rm-admin-ops--helpers button.danger {
    color: #260707;
    background: #ff7777;
}
.rm-admin-ops--helpers .head {
    display: flex;
    justify-content: space-between;
    gap: 12px;
}
.rm-admin-ops--helpers .head small {
    display: block;
    margin-top: 4px;
    color: var(--ops-muted);
}
.rm-admin-ops--helpers .head span {
    color: #58e59e;
    font-size: 12px;
}
.rm-admin-ops--helpers .card {
    padding: 18px;
    border: 1px solid var(--ops-line);
    border-radius: 20px;
    background: linear-gradient(145deg, rgba(19, 40, 30, .96), rgba(9, 23, 16, .96));
}
.rm-admin-ops--helpers .card p {
    color: var(--ops-muted);
}
.rm-admin-ops--helpers .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 9px;
}
.rm-admin-ops--helpers .actions form {
    display: grid;
    gap: 7px;
}
.rm-admin-ops.rm-admin-ops--assign {
    width: min(780px, 100%);
}
.rm-admin-ops--assign .hero {
    border-color: rgba(214, 183, 122, .25);
    background:
        linear-gradient(135deg, rgba(214, 183, 122, .12), rgba(17, 21, 28, .94) 48%),
        var(--card);
}
.rm-admin-ops--assign .kicker {
    color: var(--ops-gold);
}
.rm-admin-ops--assign .protected {
    color: var(--ops-green);
    font-size: 11px;
    font-weight: 950;
}
.rm-admin-ops--assign .policy {
    margin-top: 15px;
    padding: 17px;
    border: 1px solid var(--ops-line);
    border-radius: 18px;
    color: var(--ops-muted);
    font-size: 13px;
    line-height: 1.65;
    background: linear-gradient(145deg, rgba(20, 24, 32, .96), rgba(10, 12, 17, .97));
}
.rm-admin-ops--assign .form-card {
    display: grid;
    gap: 18px;
    margin-top: 18px;
    padding: 24px;
    border: 1px solid var(--ops-line);
    border-radius: 24px;
    background: linear-gradient(145deg, rgba(20, 24, 32, .96), rgba(10, 12, 17, .97));
    box-shadow: 0 24px 70px rgba(0, 0, 0, .28);
}
.rm-admin-ops--assign label {
    display: grid;
    gap: 8px;
    color: var(--ops-muted);
    font-size: 12px;
    font-weight: 850;
}
.rm-admin-ops--assign input,
.rm-admin-ops--assign select,
.rm-admin-ops--assign textarea {
    width: 100%;
    min-height: 49px;
    padding: 11px 14px;
    border: 1px solid rgba(255, 255, 255, .12);
    border-radius: 14px;
    color: var(--text);
    background: rgba(255, 255, 255, .04);
    font: inherit;
}
.rm-admin-ops--assign textarea {
    min-height: 115px;
    resize: vertical;
}
.rm-admin-ops--assign .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
}
.rm-admin-ops--assign button[type="submit"] {
    min-height: 52px;
    border: 1px solid rgba(214, 183, 122, .38);
    border-radius: 15px;
    color: #17130b;
    background: linear-gradient(135deg, #efd49a, #cda85f);
    font-size: 14px;
    font-weight: 950;
    cursor: pointer;
}
.rm-admin-ops--assign .warning {
    margin: 0;
    color: var(--ops-red);
    font-size: 12px;
    line-height: 1.55;
}
@media (max-width: 620px) {
    .rm-admin-ops--helpers .fields,
    .rm-admin-ops--helpers .actions { grid-template-columns: 1fr; }
    .rm-admin-ops--assign .grid { grid-template-columns: 1fr; }
    .rm-admin-ops--assign .protected { display: none; }
}
"#
}

pub(crate) fn admin_ops_page(title: &str, content_html: &str) -> String {
    admin_ops_page_themed(title, "", content_html)
}

pub(crate) fn admin_ops_page_themed(title: &str, theme_modifier: &str, content_html: &str) -> String {
    admin_ops_page_styled(title, theme_modifier, content_html, "")
}

pub(crate) fn admin_ops_page_styled(
    title: &str,
    theme_modifier: &str,
    content_html: &str,
    extra_styles: &str,
) -> String {
    let wrapper_class = if theme_modifier.is_empty() {
        "rm-admin-ops".to_string()
    } else {
        format!("rm-admin-ops {theme_modifier}")
    };

    let head_extra = format!(
        r#"<meta name="robots" content="noindex,nofollow">
<style>{styles}{extra}</style>"#,
        styles = admin_ops_styles(),
        extra = extra_styles,
    );

    page_document(
        title,
        &head_extra,
        "",
        &format!(r#"<div class="{wrapper_class}">{content_html}</div>"#),
        "",
        "",
    )
}

pub(crate) fn guest_mode_hint() -> &'static str {
    r#"<p class="rm-guest-hint">Вы смотрите как гость. Карта и поиск доступны без регистрации. <a href="/login">Войти</a> · <a href="/register">Регистрация</a></p>"#
}

pub(crate) fn guest_mode_panel(next_path: &str) -> String {
    let login_href = if next_path.is_empty() {
        "/login".to_string()
    } else {
        format!("/login?next={}", urlencoding::encode(next_path),)
    };

    let register_href = if next_path.is_empty() {
        "/register".to_string()
    } else {
        format!("/register?next={}", urlencoding::encode(next_path),)
    };

    format!(
        r#"
<div class="rm-guest-panel card">
    <div class="card-content">
        <div class="card-title rm-guest-title">
            Вы в гостевом режиме
        </div>
        <div class="card-meta rm-guest-copy">
            Смотрите карту и ресурсы без регистрации.
            Войдите, когда понадобятся сообщения, избранное или публикации.
        </div>
    </div>

    <div class="rm-guest-actions">
        {map_card}
        {search_card}
        {login_card}
        {register_card}
    </div>
</div>"#,
        map_card = navigation_card("/app", "globe", "Карта ресурсов", "Страны и города"),
        search_card = navigation_card("/app/search", "search", "Поиск", "Люди и ресурсы"),
        login_card = navigation_card(
            &login_href,
            "user",
            "Войти",
            "Email и пароль",
        ),
        register_card = navigation_card(
            &register_href,
            "edit",
            "Регистрация",
            "Создать аккаунт на сайте",
        ),
    )
}

pub(crate) fn guest_locked_section(feature: &str, next_path: &str) -> String {
    format!(
        "{note}{panel}",
        note = empty_state_card(
            "Нужен аккаунт",
            &format!(
                "Раздел «{feature}» доступен после входа. Карта и поиск работают без регистрации.",
                feature = escape_html(feature),
            ),
        ),
        panel = guest_mode_panel(next_path),
    )
}

pub(crate) fn section_head(title: &str, caption: &str, margin_top: Option<u32>) -> String {
    let class_extra = match margin_top {
        Some(24) => " section-head--mt-24",
        _ => "",
    };

    format!(
        r#"<div class="section-head{class_extra}">
    <div>
        <h2 class="section-title">{title}</h2>
        <p class="section-caption">{caption}</p>
    </div>
</div>"#,
        class_extra = class_extra,
        title = escape_html(title),
        caption = escape_html(caption),
    )
}

/// HTML shell for transactional emails (login codes, password reset, admin OTP).
/// Inline styles are required for email client compatibility.
pub fn transactional_code_email_html(
    heading: &str,
    intro: &str,
    code: &str,
    expiry_note: &str,
    footer: &str,
) -> String {
    format!(
        "<div style=\"font-family:Arial,sans-serif;max-width:520px;margin:auto;padding:32px\">\
            <h2 style=\"margin:0 0 18px\">{heading}</h2>\
            <p>{intro}</p>\
            <div style=\"font-size:34px;font-weight:800;letter-spacing:8px;margin:24px 0\">{code}</div>\
            <p>{expiry_note}</p>\
            <p style=\"color:#777;font-size:13px\">{footer}</p>\
        </div>",
        heading = escape_html(heading),
        intro = escape_html(intro),
        code = escape_html(code),
        expiry_note = escape_html(expiry_note),
        footer = escape_html(footer),
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

#[cfg(test)]
mod public_entry_tests {
    use super::*;

    #[test]
    fn registered_icons_render_svg() {
        let icons = [
            "globe",
            "map",
            "search",
            "user",
            "star",
            "map-pin",
            "chevron",
            "briefcase",
            "building",
            "heart",
            "message-circle",
            "menu",
            "logo",
            "arrow-left",
            "chevron-left",
            "shield",
            "users",
            "bell",
            "settings",
            "alert-triangle",
            "plus",
            "edit",
            "check",
            "x",
        ];

        for name in icons {
            let markup = icon(name);
            assert!(!markup.is_empty(), "missing icon: {name}");
            assert!(markup.contains("<svg"), "icon not svg: {name}");
        }
    }

    #[test]
    fn topbar_exposes_account_inside_site() {
        let html = topbar("Карта", "map");

        assert!(html.contains("href=\"/app\""));
        assert!(html.contains("href=\"/app/me\""));
        assert!(html.contains("Профиль"));
        assert!(html.contains("brand-logo-icon"));
        assert!(html.contains("ResursMap"));
        assert!(!html.contains("/app/auth"));
    }
}
