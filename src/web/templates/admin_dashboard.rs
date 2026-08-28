use super::common::{escape_html, page_document};

pub(crate) struct AdminDashboardData<'a> {
    pub owner_name: &'a str,
    pub level: i64,
    pub level_title: &'a str,
    pub territory: &'a str,
    pub assignment_id: i64,
    pub scope_id: i64,
    pub enabled_permissions: i64,
    pub users: i64,
    pub resources: i64,
    pub pending_resources: i64,
    pub premium_resources: i64,
    pub complaints: i64,
    pub unread_notifications: i64,
    pub active_sessions: i64,
    pub security_warnings: i64,
    pub audit_events: i64,
    pub level_counts: [i64; 5],
}

pub(crate) fn render_admin_dashboard(data: AdminDashboardData<'_>) -> String {
    let owner_name = escape_html(data.owner_name);
    let level_title = escape_html(data.level_title);
    let territory = escape_html(data.territory);

    let head = r#"
<style>
.admin-v2 {
    --owner-gold:#d6b77a;
    --owner-gold-soft:#f0d69c;
    --owner-emerald:#2ac785;
    --owner-violet:#8974ff;
    --owner-blue:#64a8ff;
    --owner-danger:#ef5964;
    --owner-panel:rgba(17,20,25,.86);
    --owner-line:rgba(214,183,122,.18);
    max-width:1180px;
    margin:0 auto;
    padding-bottom:50px;
}
.admin-owner-hero {
    position:relative;
    overflow:hidden;
    padding:28px;
    border:1px solid rgba(214,183,122,.30);
    border-radius:26px;
    background:
        radial-gradient(circle at 85% 15%,
            rgba(137,116,255,.16),transparent 34%),
        radial-gradient(circle at 10% 100%,
            rgba(42,199,133,.12),transparent 35%),
        linear-gradient(145deg,
            rgba(27,28,31,.98),rgba(8,10,13,.98));
    box-shadow:
        0 28px 80px rgba(0,0,0,.36),
        inset 0 1px rgba(255,255,255,.05);
}
.admin-owner-kicker {
    color:var(--owner-gold);
    font-size:11px;
    font-weight:900;
    letter-spacing:.18em;
}
.admin-owner-title {
    max-width:760px;
    margin:13px 0 10px;
    font-size:clamp(30px,7vw,58px);
    line-height:.98;
    letter-spacing:-.045em;
}
.admin-owner-subtitle {
    max-width:700px;
    color:var(--muted);
    font-size:14px;
    line-height:1.6;
}
.admin-owner-badges {
    display:flex;
    flex-wrap:wrap;
    gap:9px;
    margin-top:22px;
}
.admin-owner-badge {
    min-height:36px;
    display:inline-flex;
    align-items:center;
    padding:0 13px;
    border-radius:999px;
    border:1px solid var(--owner-line);
    background:rgba(255,255,255,.035);
    color:var(--owner-gold-soft);
    font-size:12px;
    font-weight:850;
}
.admin-owner-badge.safe {
    border-color:rgba(42,199,133,.30);
    color:#69e6ae;
}
.admin-section-title {
    margin:28px 0 13px;
    display:flex;
    justify-content:space-between;
    align-items:flex-end;
    gap:16px;
}
.admin-section-title h2 {
    margin:0;
    font-size:20px;
}
.admin-section-title span {
    color:var(--muted);
    font-size:12px;
}
.admin-stat-grid {
    display:grid;
    grid-template-columns:repeat(2,minmax(0,1fr));
    gap:12px;
}
.admin-stat {
    min-height:132px;
    padding:18px;
    border:1px solid var(--owner-line);
    border-radius:21px;
    background:
        linear-gradient(145deg,
            rgba(255,255,255,.045),
            rgba(255,255,255,.018));
    box-shadow:0 14px 34px rgba(0,0,0,.14);
}
.admin-stat-icon {
    font-size:20px;
    margin-bottom:19px;
}
.admin-stat-value {
    font-size:28px;
    font-weight:950;
    line-height:1;
}
.admin-stat-label {
    margin-top:8px;
    color:var(--muted);
    font-size:12px;
    line-height:1.4;
}
.admin-command-grid {
    display:grid;
    gap:12px;
}
.admin-command-card {
    padding:20px;
    border:1px solid var(--owner-line);
    border-radius:22px;
    background:var(--owner-panel);
}
.admin-command-header {
    display:flex;
    justify-content:space-between;
    gap:12px;
    align-items:center;
}
.admin-command-name {
    font-size:16px;
    font-weight:900;
}
.admin-command-status {
    color:#69e6ae;
    font-size:12px;
    font-weight:850;
}
.admin-command-meta {
    margin-top:8px;
    color:var(--muted);
    font-size:12px;
    line-height:1.5;
}
.admin-levels {
    display:grid;
    gap:9px;
}
.admin-level-row {
    display:grid;
    grid-template-columns:45px 1fr auto;
    align-items:center;
    gap:12px;
    min-height:58px;
    padding:10px 14px;
    border:1px solid rgba(255,255,255,.08);
    border-radius:16px;
    background:rgba(255,255,255,.025);
}
.admin-level-number {
    width:36px;
    height:36px;
    display:flex;
    align-items:center;
    justify-content:center;
    border-radius:12px;
    background:rgba(214,183,122,.10);
    color:var(--owner-gold-soft);
    font-weight:950;
}
.admin-level-name {
    font-size:13px;
    font-weight:850;
}
.admin-level-count {
    color:var(--muted);
    font-size:13px;
    font-weight:850;
}
.admin-warning {
    border-color:rgba(239,89,100,.28);
}
@media (min-width:760px) {
    .admin-owner-hero { padding:42px; }
    .admin-stat-grid {
        grid-template-columns:repeat(4,minmax(0,1fr));
    }
    .admin-command-grid {
        grid-template-columns:repeat(3,minmax(0,1fr));
    }
}
@media (prefers-reduced-motion:no-preference) {
    .admin-owner-hero,
    .admin-stat,
    .admin-command-card {
        animation:adminReveal .45s ease both;
    }
    @keyframes adminReveal {
        from { opacity:0; transform:translateY(8px); }
        to { opacity:1; transform:none; }
    }
}

.admin-v2 {
    padding:
        max(10px,env(safe-area-inset-top))
        0
        calc(100px + env(safe-area-inset-bottom));
}
.admin-command-nav {
    position:sticky;
    top:8px;
    z-index:20;
    display:flex;
    gap:7px;
    margin:0 0 14px;
    padding:7px;
    overflow-x:auto;
    scrollbar-width:none;
    border:1px solid rgba(214,183,122,.16);
    border-radius:18px;
    background:rgba(8,10,13,.84);
    box-shadow:0 16px 40px rgba(0,0,0,.28);
    backdrop-filter:blur(20px);
}
.admin-command-nav::-webkit-scrollbar {
    display:none;
}
.admin-command-nav a {
    min-height:40px;
    display:inline-flex;
    align-items:center;
    padding:0 14px;
    flex:0 0 auto;
    border-radius:12px;
    color:var(--muted);
    text-decoration:none;
    font-size:12px;
    font-weight:850;
}
.admin-command-nav a:first-child {
    color:#f4ddb0;
    background:rgba(214,183,122,.10);
}
.admin-owner-hero::before {
    content:"";
    position:absolute;
    inset:0;
    pointer-events:none;
    opacity:.22;
    background-image:
        linear-gradient(rgba(214,183,122,.10) 1px,transparent 1px),
        linear-gradient(90deg,rgba(214,183,122,.10) 1px,transparent 1px);
    background-size:42px 42px;
    mask-image:linear-gradient(to bottom,black,transparent 78%);
}
.admin-owner-hero::after {
    content:"";
    position:absolute;
    width:280px;
    height:280px;
    right:-125px;
    top:-130px;
    border:1px solid rgba(214,183,122,.22);
    border-radius:50%;
    box-shadow:
        0 0 0 25px rgba(214,183,122,.025),
        0 0 0 55px rgba(137,116,255,.025);
}
.admin-owner-hero > * {
    position:relative;
    z-index:2;
}
.admin-global-visual {
    position:relative;
    min-height:230px;
    margin-top:26px;
    display:flex;
    align-items:center;
    justify-content:center;
    overflow:hidden;
    border:1px solid rgba(214,183,122,.14);
    border-radius:22px;
    background:
        radial-gradient(circle,
            rgba(42,199,133,.10),transparent 55%),
        rgba(0,0,0,.16);
}
.admin-globe {
    position:relative;
    width:150px;
    height:150px;
    border:1px solid rgba(214,183,122,.55);
    border-radius:50%;
    box-shadow:
        0 0 45px rgba(214,183,122,.10),
        inset 0 0 34px rgba(42,199,133,.08);
}
.admin-globe::before,
.admin-globe::after {
    content:"";
    position:absolute;
    inset:17px 0;
    border:1px solid rgba(214,183,122,.28);
    border-radius:50%;
}
.admin-globe::after {
    inset:0 48px;
}
.admin-globe-equator {
    position:absolute;
    left:3px;
    right:3px;
    top:50%;
    height:1px;
    background:rgba(214,183,122,.35);
}
.admin-globe-core {
    position:absolute;
    inset:46px;
    display:flex;
    align-items:center;
    justify-content:center;
    border-radius:50%;
    color:#f0d69c;
    font-size:17px;
    font-weight:950;
    letter-spacing:.08em;
    background:rgba(214,183,122,.09);
    box-shadow:0 0 30px rgba(214,183,122,.14);
}
.admin-orbit {
    position:absolute;
    width:205px;
    height:95px;
    border:1px solid rgba(137,116,255,.30);
    border-radius:50%;
    transform:rotate(-18deg);
}
.admin-orbit-dot {
    position:absolute;
    width:9px;
    height:9px;
    top:50%;
    left:calc(50% + 96px);
    border-radius:50%;
    background:#69e6ae;
    box-shadow:0 0 18px rgba(105,230,174,.8);
}
.admin-visual-label {
    position:absolute;
    left:16px;
    bottom:14px;
    color:var(--muted);
    font-size:10px;
    font-weight:850;
    letter-spacing:.12em;
}
.admin-visual-state {
    position:absolute;
    right:16px;
    top:14px;
    color:#69e6ae;
    font-size:10px;
    font-weight:900;
}
.admin-stat {
    position:relative;
    overflow:hidden;
    transition:
        transform .22s ease,
        border-color .22s ease,
        background .22s ease;
}
.admin-stat::after {
    content:"";
    position:absolute;
    width:70px;
    height:70px;
    right:-30px;
    bottom:-35px;
    border:1px solid rgba(214,183,122,.12);
    border-radius:50%;
}
.admin-stat:nth-child(3n+1) {
    border-top-color:rgba(100,168,255,.55);
}
.admin-stat:nth-child(3n+2) {
    border-top-color:rgba(214,183,122,.60);
}
.admin-stat:nth-child(3n+3) {
    border-top-color:rgba(42,199,133,.55);
}
.admin-stat:hover {
    transform:translateY(-2px);
    border-color:rgba(214,183,122,.32);
}
.admin-command-card {
    position:relative;
    overflow:hidden;
}
.admin-command-card::before {
    content:"";
    position:absolute;
    left:0;
    top:18px;
    bottom:18px;
    width:2px;
    border-radius:4px;
    background:linear-gradient(
        to bottom,
        transparent,
        rgba(214,183,122,.75),
        transparent
    );
}
.admin-level-row {
    transition:
        transform .2s ease,
        border-color .2s ease;
}
.admin-level-row:hover {
    transform:translateX(3px);
    border-color:rgba(214,183,122,.28);
}
.admin-level-row:nth-child(5) {
    border-color:rgba(214,183,122,.35);
    background:
        linear-gradient(90deg,
            rgba(214,183,122,.09),
            rgba(137,116,255,.04));
}
@media (max-width:420px) {
    .admin-owner-hero {
        padding:24px 20px;
    }
    .admin-owner-title {
        font-size:35px;
    }
    .admin-stat {
        min-height:118px;
        padding:16px;
    }
    .admin-stat-value {
        font-size:25px;
    }
}
@media (prefers-reduced-motion:no-preference) {
    .admin-orbit {
        animation:ownerOrbit 14s linear infinite;
    }
    .admin-globe {
        animation:ownerPulse 4s ease-in-out infinite;
    }
    @keyframes ownerOrbit {
        to { transform:rotate(342deg); }
    }
    @keyframes ownerPulse {
        50% {
            box-shadow:
                0 0 60px rgba(214,183,122,.16),
                inset 0 0 38px rgba(42,199,133,.12);
        }
    }
}
</style>
"#;

    let main = format!(
        r##"
<div class="admin-v2">
    <nav class="admin-command-nav"
         aria-label="Разделы центра">
        <a href="#global-overview">Обзор</a>
        <a href="#global-indicators">Показатели</a>
        <a href="#system-state">Система</a>
        <a href="#admin-hierarchy">Иерархия</a>
        <a href="/app/center/administrators">Администраторы</a>
    </nav>

    <section class="admin-owner-hero"
             id="global-overview">
        <div class="admin-owner-kicker">
            RESURSMAP · GLOBAL COMMAND CENTER
        </div>

        <h1 class="admin-owner-title">
            Глобальное управление ResursMap
        </h1>

        <div class="admin-owner-subtitle">
            Защищённый обзор международной платформы.
            Все показатели ниже получены из действующей базы данных.
        </div>

        <div class="admin-owner-badges">
            <span class="admin-owner-badge">
                👑 Уровень {level} · {level_title}
            </span>

            <span class="admin-owner-badge">
                🌍 {territory}
            </span>

            <span class="admin-owner-badge safe">
                ● Система работает
            </span>

            <span class="admin-owner-badge">
                Назначение #{assignment_id}
            </span>

            <span class="admin-owner-badge">
                Территория #{scope_id}
            </span>

            <span class="admin-owner-badge safe">
                Разрешения · {enabled_permissions}
            </span>
        </div>

        <div class="admin-global-visual"
             role="img"
             aria-label="Глобальная сеть ResursMap работает">
            <div class="admin-orbit">
                <span class="admin-orbit-dot"></span>
            </div>

            <div class="admin-globe">
                <span class="admin-globe-equator"></span>
                <span class="admin-globe-core">RM</span>
            </div>

            <div class="admin-visual-state">
                ● GLOBAL NETWORK ONLINE
            </div>

            <div class="admin-visual-label">
                WORLD CONTROL SURFACE · OWNER ACCESS
            </div>
        </div>
    </section>

    <div class="admin-section-title"
         id="global-indicators">
        <h2>Глобальные показатели</h2>
        <span>Действующие данные</span>
    </div>

    <section class="admin-stat-grid">
        {stats}
    </section>

    <div class="admin-section-title"
         id="system-state">
        <h2>Состояние управления</h2>
        <span>Admin V2</span>
    </div>

    <section class="admin-command-grid">
        <article class="admin-command-card">
            <div class="admin-command-header">
                <div class="admin-command-name">База данных</div>
                <div class="admin-command-status">● ONLINE</div>
            </div>
            <div class="admin-command-meta">
                SQLite подключена. Миграция Admin V2 активна.
            </div>
        </article>

        <article class="admin-command-card">
            <div class="admin-command-header">
                <div class="admin-command-name">Owner-защита</div>
                <div class="admin-command-status">● ACTIVE</div>
            </div>
            <div class="admin-command-meta">
                Единственный Global Owner · {owner_name}
            </div>
        </article>

        <article class="admin-command-card {warning_class}">
            <div class="admin-command-header">
                <div class="admin-command-name">Безопасность</div>
                <div class="admin-command-status">
                    {security_status}
                </div>
            </div>
            <div class="admin-command-meta">
                Критические события за последние 24 часа: {security_warnings}
            </div>
        </article>
    </section>

    <div class="admin-section-title"
         id="admin-hierarchy">
        <h2>Иерархия администраторов</h2>
        <span>Уровни 1–5</span>
    </div>

    <section class="admin-levels">
        {level_rows}
    </section>
</div>
"##,
        level = data.level,
        level_title = level_title,
        territory = territory,
        assignment_id = data.assignment_id,
        scope_id = data.scope_id,
        enabled_permissions = data.enabled_permissions,
        owner_name = owner_name,
        security_warnings = data.security_warnings,
        security_status = if data.security_warnings == 0 {
            "● НОРМА"
        } else {
            "● ВНИМАНИЕ"
        },
        warning_class = if data.security_warnings == 0 {
            ""
        } else {
            "admin-warning"
        },
        stats = render_stats(&data),
        level_rows = render_levels(data.level_counts),
    );

    page_document("Global Command Center · ResursMap", head, "", &main, "", "")
}

fn render_stats(data: &AdminDashboardData<'_>) -> String {
    let values = [
        ("👥", data.users, "Активные пользователи"),
        ("◈", data.resources, "Активные ресурсы"),
        ("⌛", data.pending_resources, "Ожидают проверки"),
        ("★", data.premium_resources, "Premium-ресурсы"),
        ("⚑", data.complaints, "Открытые жалобы"),
        ("🔔", data.unread_notifications, "Ваши уведомления"),
        ("🔐", data.active_sessions, "Admin-сессии"),
        ("▤", data.audit_events, "События аудита"),
    ];

    values
        .iter()
        .map(|(icon, value, label)| {
            format!(
                r#"
<article class="admin-stat">
    <div class="admin-stat-icon">{icon}</div>
    <div class="admin-stat-value">{value}</div>
    <div class="admin-stat-label">{label}</div>
</article>
"#
            )
        })
        .collect()
}

fn render_levels(levels: [i64; 5]) -> String {
    let names = [
        "Помощники групп",
        "Администраторы городов",
        "Администраторы стран",
        "Администраторы континентов",
        "Global Owner",
    ];

    names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            format!(
                r#"
<div class="admin-level-row">
    <div class="admin-level-number">{level}</div>
    <div class="admin-level-name">{name}</div>
    <div class="admin-level-count">{count}</div>
</div>
"#,
                level = index + 1,
                count = levels[index],
            )
        })
        .collect()
}
