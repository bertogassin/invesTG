#[derive(Debug)]
pub struct AdminAdministratorRow {
    pub assignment_id: i64,
    pub user_id: i64,
    pub level: i64,
    pub scope_type: String,
    pub territory: String,
    pub permission_count: u32,
    pub status: String,
    pub valid_from: i64,
    pub valid_until: Option<i64>,
    pub display_name: String,
    pub username: String,
    pub active_sessions: i64,
    pub audit_events: i64,
}

#[derive(Debug)]
pub struct AdminSessionRow {
    pub public_id: String,
    pub user_id: i64,
    pub assignment_id: i64,
    pub ip_address: String,
    pub user_agent: String,
    pub device_label: String,
    pub two_factor_verified: bool,
    pub created_at: i64,
    pub last_seen_at: i64,
    pub expires_at: i64,
    pub display_name: String,
}

#[derive(Debug)]
pub struct AdminAdministratorsData {
    pub viewer_name: String,
    pub administrators: Vec<AdminAdministratorRow>,
    pub sessions: Vec<AdminSessionRow>,
    pub active_assignments: usize,
    pub expiring_assignments: usize,
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn level_title(level: i64) -> &'static str {
    match level {
        1 => "Помощник группы",
        2 => "Администратор города",
        3 => "Администратор страны",
        4 => "Администратор континента",
        5 => "Global Owner",
        _ => "Неизвестный уровень",
    }
}

fn scope_title(scope_type: &str) -> &'static str {
    match scope_type {
        "group" => "Группа",
        "city" => "Город",
        "country" => "Страна",
        "continent" => "Континент",
        "world" => "Мир",
        _ => "Территория",
    }
}

fn lifecycle_actions(administrator: &AdminAdministratorRow) -> String {
    if administrator.level == 5 {
        return r#"<div class="owner-lock">
            Global Owner защищён от изменения и отзыва.
        </div>"#
            .to_string();
    }

    let id = administrator.assignment_id;
    let mut actions = String::new();

    if administrator.status == "active" {
        actions.push_str(&format!(
            r#"<form class="lifecycle-form"
                     method="post"
                     action="/app/center/administrators/{id}/suspend">
                <input name="reason"
                       minlength="5"
                       maxlength="500"
                       required
                       placeholder="Причина приостановки">
                <button class="warning-action"
                        type="submit">
                    Приостановить
                </button>
            </form>"#
        ));
    }

    if administrator.status == "suspended" {
        actions.push_str(&format!(
            r#"<form class="lifecycle-form"
                     method="post"
                     action="/app/center/administrators/{id}/restore">
                <input name="reason"
                       minlength="5"
                       maxlength="500"
                       required
                       placeholder="Причина восстановления">
                <button class="restore-action"
                        type="submit">
                    Восстановить
                </button>
            </form>"#
        ));
    }

    if administrator.status == "active" || administrator.status == "suspended" {
        actions.push_str(&format!(
            r#"<form class="lifecycle-form expiry-form"
                     method="post"
                     action="/app/center/administrators/{id}/change-expiry">
                <input type="number"
                       name="duration_days"
                       min="1"
                       max="365"
                       value="90"
                       required
                       aria-label="Новый срок в днях">
                <input name="reason"
                       minlength="5"
                       maxlength="500"
                       required
                       placeholder="Причина изменения срока">
                <button type="submit">
                    Изменить срок
                </button>
            </form>

            <form class="lifecycle-form"
                  method="post"
                  action="/app/center/administrators/{id}/revoke">
                <input name="reason"
                       minlength="5"
                       maxlength="500"
                       required
                       placeholder="Причина окончательного отзыва">
                <button class="danger-action"
                        type="submit">
                    Отозвать назначение
                </button>
            </form>"#
        ));
    }

    if actions.is_empty() {
        return r#"<div class="owner-lock">
            Для этого назначения действия недоступны.
        </div>"#
            .to_string();
    }

    format!(
        r#"<div class="lifecycle-actions">
            <div class="lifecycle-title">
                Управление назначением
            </div>
            {actions}
        </div>"#
    )
}

pub fn render_admin_administrators(data: AdminAdministratorsData) -> String {
    let administrators_html = if data.administrators.is_empty() {
        r#"<div class="empty">
                Административные назначения не найдены.
            </div>"#
            .to_string()
    } else {
        data.administrators
            .iter()
            .map(|administrator| {
                let status_class = if administrator.status == "active" {
                    "active"
                } else {
                    "inactive"
                };

                let valid_until = administrator
                    .valid_until
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "Бессрочно".to_string());

                let lifecycle_controls = lifecycle_actions(administrator);

                let username = if administrator.username.is_empty() {
                    "Вход связан с профилем".to_string()
                } else {
                    format!("@{}", escape_html(&administrator.username))
                };

                format!(
                    r#"
                        <article class="administrator-card level-{level}">
                            <div class="administrator-top">
                                <div class="level-mark">{level}</div>
                                <div class="administrator-identity">
                                    <h2>{display_name}</h2>
                                    <p>{username} · User #{user_id}</p>
                                </div>
                                <span class="status {status_class}">
                                    {status}
                                </span>
                            </div>

                            <div class="role-name">
                                Уровень {level} · {level_title}
                            </div>

                            <div class="territory">
                                <span>{scope_title}</span>
                                <strong>{territory}</strong>
                            </div>

                            <div class="administrator-metrics">
                                <div>
                                    <strong>{permission_count}</strong>
                                    <span>разрешений</span>
                                </div>
                                <div>
                                    <strong>{active_sessions}</strong>
                                    <span>активных сессий</span>
                                </div>
                                <div>
                                    <strong>{audit_events}</strong>
                                    <span>событий аудита</span>
                                </div>
                            </div>

                            <div class="administrator-foot">
                                <span>Назначение #{assignment_id}</span>
                                <span>Начало: {valid_from}</span>
                                <span>Окончание: {valid_until}</span>
                                <span>Scope: {scope_type}</span>
                            </div>

                            {lifecycle_controls}
                        </article>
                        "#,
                    level = administrator.level,
                    display_name = escape_html(&administrator.display_name),
                    username = username,
                    user_id = administrator.user_id,
                    status_class = status_class,
                    status = escape_html(&administrator.status),
                    level_title = level_title(administrator.level),
                    scope_title = scope_title(&administrator.scope_type),
                    territory = escape_html(&administrator.territory),
                    permission_count = administrator.permission_count,
                    active_sessions = administrator.active_sessions,
                    audit_events = administrator.audit_events,
                    assignment_id = administrator.assignment_id,
                    valid_from = administrator.valid_from,
                    valid_until = valid_until,
                    scope_type = escape_html(&administrator.scope_type),
                    lifecycle_controls = lifecycle_controls,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let sessions_html = if data.sessions.is_empty() {
        r#"<div class="empty">
            Активных административных сессий пока нет.
        </div>"#
            .to_string()
    } else {
        data.sessions
            .iter()
            .map(|session| {
                let short_id = session.public_id.chars().take(12).collect::<String>();

                let ip = if session.ip_address.is_empty() {
                    "IP не передан прокси"
                } else {
                    &session.ip_address
                };

                let user_agent = if session.user_agent.is_empty() {
                    "Устройство не определено"
                } else {
                    &session.user_agent
                };

                let factor = if session.two_factor_verified {
                    "2FA подтверждена"
                } else {
                    "2FA ещё не подтверждена"
                };

                format!(
                    r#"
                    <article class="session-card">
                        <div class="session-head">
                            <div>
                                <strong>{display_name}</strong>
                                <span>#{short_id}</span>
                            </div>
                            <span class="live-dot">Активна</span>
                        </div>

                        <div class="session-grid">
                            <p>
                                <span>Пользователь</span>
                                User #{user_id}
                            </p>
                            <p>
                                <span>Назначение</span>
                                #{assignment_id}
                            </p>
                            <p>
                                <span>IP</span>
                                {ip}
                            </p>
                            <p>
                                <span>Защита</span>
                                {factor}
                            </p>
                            <p>
                                <span>Создана</span>
                                {created_at}
                            </p>
                            <p>
                                <span>Последняя активность</span>
                                {last_seen_at}
                            </p>
                            <p>
                                <span>Истекает</span>
                                {expires_at}
                            </p>
                            <p>
                                <span>Устройство</span>
                                {device_label}
                            </p>
                        </div>

                        <div class="user-agent">{user_agent}</div>

                        <form class="revoke-form"
                              method="post"
                              action="/app/center/sessions/{session_id}/revoke">
                            <label>
                                Причина завершения сессии
                                <input name="reason"
                                       minlength="5"
                                       maxlength="500"
                                       required
                                       placeholder="Например: неизвестное устройство">
                            </label>
                            <button type="submit">
                                Завершить эту сессию
                            </button>
                        </form>
                    </article>
                    "#,
                    display_name = escape_html(&session.display_name),
                    short_id = escape_html(&short_id),
                    user_id = session.user_id,
                    assignment_id = session.assignment_id,
                    ip = escape_html(ip),
                    factor = factor,
                    created_at = session.created_at,
                    last_seen_at = session.last_seen_at,
                    expires_at = session.expires_at,
                    device_label = escape_html(&session.device_label),
                    user_agent = escape_html(user_agent),
                    session_id = escape_html(&session.public_id),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };

    format!(
        r##"<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport"
      content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="color-scheme" content="dark">
<title>Все администраторы · ResursMap</title>
<style>
:root {{
    --bg:#07090d;
    --surface:#101319;
    --surface-2:#151923;
    --gold:#dfc07f;
    --gold-soft:rgba(223,192,127,.16);
    --green:#62e0ad;
    --red:#ff6d78;
    --blue:#7ab9ff;
    --violet:#a893ff;
    --text:#f5f2eb;
    --muted:#9298a6;
    --line:rgba(223,192,127,.18);
}}
* {{ box-sizing:border-box; }}
html {{ scroll-behavior:smooth; }}
body {{
    margin:0;
    min-height:100vh;
    padding:
        max(18px,env(safe-area-inset-top))
        16px
        max(32px,env(safe-area-inset-bottom));
    color:var(--text);
    background:
        radial-gradient(circle at 10% 0%,rgba(30,98,73,.18),transparent 34%),
        radial-gradient(circle at 100% 8%,rgba(92,65,155,.18),transparent 32%),
        linear-gradient(160deg,#090b10,#05070a 68%);
    font-family:Inter,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;
}}
.page {{
    width:min(1080px,100%);
    margin:0 auto;
}}
.topbar {{
    display:flex;
    align-items:center;
    justify-content:space-between;
    gap:14px;
    margin-bottom:18px;
}}
.back {{
    min-height:44px;
    display:inline-flex;
    align-items:center;
    padding:0 16px;
    border:1px solid var(--line);
    border-radius:14px;
    color:var(--text);
    background:rgba(255,255,255,.025);
    text-decoration:none;
    font-weight:800;
}}
.protected {{
    color:var(--green);
    font-size:12px;
    font-weight:900;
    letter-spacing:.08em;
}}
.topbar-actions {{
    display:flex;
    align-items:center;
    gap:10px;
}}
.create-assignment {{
    min-height:44px;
    display:inline-flex;
    align-items:center;
    padding:0 15px;
    border:1px solid rgba(223,192,127,.34);
    border-radius:14px;
    color:var(--gold);
    background:var(--gold-soft);
    text-decoration:none;
    font-size:12px;
    font-weight:950;
}}
.hero {{
    position:relative;
    overflow:hidden;
    padding:28px;
    border:1px solid rgba(223,192,127,.28);
    border-radius:26px;
    background:
        linear-gradient(135deg,rgba(223,192,127,.10),transparent 42%),
        linear-gradient(145deg,rgba(18,22,29,.96),rgba(10,12,17,.96));
    box-shadow:0 24px 70px rgba(0,0,0,.32);
}}
.hero::after {{
    content:"";
    position:absolute;
    width:240px;
    height:240px;
    right:-100px;
    top:-120px;
    border:1px solid rgba(223,192,127,.18);
    border-radius:50%;
    box-shadow:
        0 0 0 34px rgba(223,192,127,.025),
        0 0 0 70px rgba(168,147,255,.025);
}}
.kicker {{
    color:var(--gold);
    font-size:12px;
    font-weight:950;
    letter-spacing:.18em;
}}
h1 {{
    max-width:720px;
    margin:12px 0 10px;
    font-size:clamp(32px,7vw,58px);
    line-height:.98;
    letter-spacing:-.045em;
}}
.hero p {{
    max-width:720px;
    margin:0;
    color:var(--muted);
    font-size:16px;
    line-height:1.65;
}}
.summary {{
    display:grid;
    grid-template-columns:repeat(3,minmax(0,1fr));
    gap:12px;
    margin-top:22px;
}}
.summary-card {{
    padding:16px;
    border:1px solid var(--line);
    border-radius:18px;
    background:rgba(255,255,255,.025);
}}
.summary-card strong {{
    display:block;
    color:var(--gold);
    font-size:28px;
}}
.summary-card span {{
    color:var(--muted);
    font-size:12px;
}}
.section-title {{
    display:flex;
    justify-content:space-between;
    align-items:end;
    gap:16px;
    margin:34px 0 14px;
}}
.section-title h2 {{
    margin:0;
    font-size:25px;
}}
.section-title span {{
    color:var(--muted);
    font-size:13px;
}}
.administrator-list,
.session-list {{
    display:grid;
    gap:14px;
}}
.administrator-card,
.session-card,
.empty {{
    padding:20px;
    border:1px solid rgba(255,255,255,.09);
    border-radius:22px;
    background:
        linear-gradient(145deg,rgba(20,23,30,.95),rgba(12,14,19,.96));
    box-shadow:0 15px 42px rgba(0,0,0,.20);
}}
.administrator-card.level-5 {{
    border-color:rgba(223,192,127,.34);
    background:
        linear-gradient(135deg,rgba(223,192,127,.09),transparent 38%),
        linear-gradient(145deg,#17191e,#0c0e13);
}}
.administrator-top {{
    display:flex;
    align-items:center;
    gap:14px;
}}
.level-mark {{
    flex:0 0 54px;
    height:54px;
    display:grid;
    place-items:center;
    border-radius:17px;
    color:var(--gold);
    background:var(--gold-soft);
    font-size:24px;
    font-weight:950;
}}
.administrator-identity {{
    min-width:0;
    flex:1;
}}
.administrator-identity h2 {{
    margin:0;
    overflow-wrap:anywhere;
    font-size:20px;
}}
.administrator-identity p {{
    margin:5px 0 0;
    color:var(--muted);
    font-size:12px;
}}
.status {{
    padding:7px 10px;
    border-radius:999px;
    font-size:10px;
    font-weight:950;
    text-transform:uppercase;
}}
.status.active {{
    color:var(--green);
    border:1px solid rgba(98,224,173,.30);
    background:rgba(98,224,173,.08);
}}
.status.inactive {{
    color:var(--red);
    border:1px solid rgba(255,109,120,.30);
    background:rgba(255,109,120,.08);
}}
.role-name {{
    margin-top:17px;
    color:var(--gold);
    font-size:14px;
    font-weight:900;
}}
.territory {{
    display:flex;
    justify-content:space-between;
    gap:12px;
    margin-top:12px;
    padding:14px;
    border-radius:15px;
    background:rgba(255,255,255,.025);
}}
.territory span {{
    color:var(--muted);
}}
.administrator-metrics {{
    display:grid;
    grid-template-columns:repeat(3,minmax(0,1fr));
    gap:9px;
    margin-top:12px;
}}
.administrator-metrics div {{
    padding:13px;
    border:1px solid rgba(255,255,255,.065);
    border-radius:15px;
}}
.administrator-metrics strong {{
    display:block;
    font-size:20px;
}}
.administrator-metrics span {{
    color:var(--muted);
    font-size:11px;
}}
.administrator-foot {{
    display:flex;
    flex-wrap:wrap;
    gap:8px 16px;
    margin-top:15px;
    color:var(--muted);
    font-size:11px;
}}
.lifecycle-actions {{
    display:grid;
    gap:10px;
    margin-top:17px;
    padding-top:17px;
    border-top:1px solid rgba(255,255,255,.07);
}}
.lifecycle-title {{
    color:var(--gold);
    font-size:12px;
    font-weight:900;
}}
.lifecycle-form {{
    display:grid;
    grid-template-columns:minmax(0,1fr) auto;
    gap:9px;
}}
.lifecycle-form.expiry-form {{
    grid-template-columns:90px minmax(0,1fr) auto;
}}
.lifecycle-form input {{
    min-width:0;
    min-height:43px;
    padding:0 12px;
    border:1px solid rgba(255,255,255,.12);
    border-radius:12px;
    outline:none;
    color:var(--text);
    background:rgba(255,255,255,.035);
    font:inherit;
}}
.lifecycle-form button {{
    min-height:43px;
    padding:0 14px;
    border:1px solid var(--line);
    border-radius:12px;
    color:var(--gold);
    background:var(--gold-soft);
    font-weight:900;
    cursor:pointer;
}}
.lifecycle-form .warning-action {{
    color:#ffc26f;
    border-color:rgba(255,194,111,.30);
    background:rgba(255,194,111,.08);
}}
.lifecycle-form .restore-action {{
    color:var(--green);
    border-color:rgba(98,224,173,.30);
    background:rgba(98,224,173,.08);
}}
.lifecycle-form .danger-action {{
    color:var(--red);
    border-color:rgba(255,109,120,.32);
    background:rgba(255,109,120,.08);
}}
.owner-lock {{
    margin-top:17px;
    padding:13px;
    border:1px solid rgba(223,192,127,.14);
    border-radius:14px;
    color:var(--muted);
    background:rgba(223,192,127,.035);
    font-size:11px;
}}
.session-head {{
    display:flex;
    align-items:center;
    justify-content:space-between;
    gap:12px;
}}
.session-head strong,
.session-head span {{
    display:block;
}}
.session-head div span {{
    margin-top:4px;
    color:var(--muted);
    font-size:11px;
}}
.live-dot {{
    color:var(--green);
    font-size:11px;
    font-weight:900;
}}
.live-dot::before {{
    content:"";
    display:inline-block;
    width:7px;
    height:7px;
    margin-right:6px;
    border-radius:50%;
    background:var(--green);
    box-shadow:0 0 14px rgba(98,224,173,.5);
}}
.session-grid {{
    display:grid;
    grid-template-columns:repeat(4,minmax(0,1fr));
    gap:9px;
    margin-top:15px;
}}
.session-grid p {{
    min-width:0;
    margin:0;
    padding:12px;
    overflow-wrap:anywhere;
    border-radius:14px;
    background:rgba(255,255,255,.025);
    font-size:12px;
}}
.session-grid span {{
    display:block;
    margin-bottom:5px;
    color:var(--muted);
    font-size:10px;
}}
.user-agent {{
    margin-top:10px;
    color:var(--muted);
    overflow-wrap:anywhere;
    font-size:10px;
}}
.revoke-form {{
    display:grid;
    grid-template-columns:minmax(0,1fr) auto;
    align-items:end;
    gap:10px;
    margin-top:16px;
    padding-top:16px;
    border-top:1px solid rgba(255,255,255,.07);
}}
.revoke-form label {{
    color:var(--muted);
    font-size:11px;
    font-weight:750;
}}
.revoke-form input {{
    width:100%;
    min-height:44px;
    display:block;
    margin-top:7px;
    padding:0 13px;
    border:1px solid rgba(255,255,255,.12);
    border-radius:13px;
    outline:none;
    color:var(--text);
    background:rgba(255,255,255,.035);
    font:inherit;
}}
.revoke-form button {{
    min-height:44px;
    padding:0 15px;
    border:1px solid rgba(255,109,120,.30);
    border-radius:13px;
    color:#ff8992;
    background:rgba(255,109,120,.08);
    font-weight:900;
    cursor:pointer;
}}
.empty {{
    color:var(--muted);
    text-align:center;
}}
@media (max-width:720px) {{
    .hero {{ padding:22px; }}
    .summary {{
        grid-template-columns:repeat(2,minmax(0,1fr));
    }}
    .summary-card:first-child {{
        grid-column:1/-1;
    }}
    .administrator-metrics {{
        grid-template-columns:repeat(2,minmax(0,1fr));
    }}
    .administrator-metrics div:first-child {{
        grid-column:1/-1;
    }}
    .session-grid {{
        grid-template-columns:repeat(2,minmax(0,1fr));
    }}
    .revoke-form {{
        grid-template-columns:1fr;
    }}
    .lifecycle-form,
    .lifecycle-form.expiry-form {{
        grid-template-columns:1fr;
    }}
}}
@media (max-width:430px) {{
    body {{ padding-left:12px;padding-right:12px; }}
    .topbar {{
        align-items:flex-start;
    }}
    .topbar-actions {{
        flex-direction:column;
        align-items:flex-end;
    }}
    .protected {{
        display:none;
    }}
    .create-assignment {{
        max-width:170px;
        min-height:40px;
        text-align:center;
    }}
    .administrator-card,
    .session-card {{ padding:16px; }}
    .status {{ align-self:flex-start; }}
    .administrator-top {{ align-items:flex-start; }}
    .level-mark {{
        flex-basis:46px;
        height:46px;
    }}
}}
@media (prefers-reduced-motion:reduce) {{
    html {{ scroll-behavior:auto; }}
}}
</style>
</head>
<body>
<main class="page">
    <div class="topbar">
        <a class="back" href="/app/center">← Центр управления</a>
        <div class="topbar-actions">
            <a class="create-assignment"
               href="/app/center/administrators/new">
                + Назначить администратора
            </a>
            <span class="protected">
                ЗАЩИЩЁННЫЙ РАЗДЕЛ
            </span>
        </div>
    </div>

    <section class="hero">
        <div class="kicker">
            RESURSMAP · ADMINISTRATOR CONTROL
        </div>
        <h1>Центр администраторов</h1>
        <p>
            Реальные назначения, территориальные области,
            разрешения и активные административные сессии.
            Просмотр выполняется от имени {viewer_name}.
        </p>

        <div class="summary">
            <div class="summary-card">
                <strong>{total}</strong>
                <span>всего назначений</span>
            </div>
            <div class="summary-card">
                <strong>{active}</strong>
                <span>активных назначений</span>
            </div>
            <div class="summary-card">
                <strong>{sessions}</strong>
                <span>активных сессий</span>
            </div>
        </div>
    </section>

    <div class="section-title">
        <h2>Административная иерархия</h2>
        <span>{expiring} ограничены сроком</span>
    </div>

    <section class="administrator-list">
        {administrators_html}
    </section>

    <div class="section-title">
        <h2>Активные сессии</h2>
        <span>Последние 100</span>
    </div>

    <section class="session-list">
        {sessions_html}
    </section>
</main>
</body>
</html>"##,
        viewer_name = escape_html(&data.viewer_name),
        total = data.administrators.len(),
        active = data.active_assignments,
        sessions = data.sessions.len(),
        expiring = data.expiring_assignments,
        administrators_html = administrators_html,
        sessions_html = sessions_html,
    )
}
