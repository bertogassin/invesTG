use super::common::{
    back_hero, back_link, bottom_nav, bottom_nav_with_badge, empty_state_action, empty_state_card,
    empty_state_card_with_actions, escape_html, guest_locked_section, guest_mode_panel, icon,
    moderator_level_badge, navigation_card, page_document, page_shell, premium_badge_html,
    profession_label, profile_resource_card, section_head, simple_hero, topbar, verified_badge_html,
};

pub struct RenderMeParams<'a> {
    pub authenticated: bool,
    pub user_id: i64,
    pub username: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub resources_count: i64,
    pub approved_count: i64,
    pub pending_count: i64,
    pub rejected_count: i64,
    pub favorites_count: i64,
    pub unread_notifications_count: i64,
    pub pending_contact_requests_count: i64,
    pub unread_messages_count: i64,
    pub moderator_level: i64,
    pub intent_text: &'a str,
    pub intent_until: i64,
    pub category: &'a str,
    pub home_continent_index: i64,
    pub home_country_index: i64,
    pub home_city_index: i64,
    pub user_sessions: Vec<crate::web::view_models::UserSessionRow>,
}

fn home_city_select_html(continent: i64, country: i64, city: i64) -> String {
    let world_data = crate::geography::world();
    let mut options = String::from(r#"<option value="">Город не указан</option>"#);

    for (ci, (continent_name, countries)) in world_data.iter().enumerate() {
        let ci = ci as i64;
        options.push_str(&format!(
            r#"<optgroup label="{}">"#,
            escape_html(continent_name)
        ));

        for (si, (country_name, cities)) in countries.iter().enumerate() {
            let si = si as i64;
            for (zi, city_name) in cities.iter().enumerate() {
                let zi = zi as i64;
                let selected = if continent == ci && country == si && city == zi {
                    " selected"
                } else {
                    ""
                };
                options.push_str(&format!(
                    r#"<option value="{ci}:{si}:{zi}"{selected}>{} · {}</option>"#,
                    escape_html(city_name),
                    escape_html(country_name),
                ));
            }
        }

        options.push_str("</optgroup>");
    }

    format!(
        r#"
    <label class="rm-profile-field rm-profile-field--spaced">
        <div class="rm-profile-field-label">
            Ваш город
        </div>
        <select id="profile-home-city" class="ui-select">
            {options}
        </select>
        <div class="card-meta">
            Нужен, чтобы вас находили в разделе «Работники» этого города.
        </div>
    </label>
"#
    )
}

fn session_device_label(user_agent: &str) -> &'static str {
    let agent = user_agent.to_lowercase();

    if agent.contains("telegram") {
        "Telegram"
    } else if agent.contains("android") {
        "Android"
    } else if agent.contains("iphone") || agent.contains("ipad") {
        "iPhone / iPad"
    } else if agent.contains("windows") {
        "Windows"
    } else if agent.contains("mac os") || agent.contains("macintosh") {
        "Mac"
    } else {
        "Браузер"
    }
}

fn render_user_sessions_panel(sessions: &[crate::web::view_models::UserSessionRow]) -> String {
    if sessions.is_empty() {
        return String::new();
    }

    let rows = sessions
        .iter()
        .map(|session| {
            let device = session_device_label(&session.user_agent);
            let ip = if session.ip_address.is_empty() {
                "IP не определён".to_string()
            } else {
                escape_html(&session.ip_address)
            };
            let current = if session.is_current {
                r#"<span class="rm-session-current">Это устройство</span>"#
            } else {
                ""
            };
            let revoke_form = if session.is_current {
                String::new()
            } else {
                format!(
                    r#"<form method="post" action="/app/sessions/revoke" class="rm-session-form">
    <input type="hidden" name="session_public_id" value="{session_id}">
    <button type="submit" class="ui-button rm-session-revoke-btn">
        Завершить
    </button>
</form>"#,
                    session_id = escape_html(&session.session_public_id),
                )
            };

            format!(
                r#"<div class="rm-session-row">
    <div>
        <strong>{device}</strong>
        <div class="card-meta rm-session-ip">{ip}</div>
        {current}
    </div>
    {revoke_form}
</div>"#,
                device = device,
                ip = ip,
                current = current,
                revoke_form = revoke_form,
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"<section class="rm-sessions-panel">
    <div class="card-title rm-sessions-title">
        Активные сессии
    </div>
    <div class="card-meta rm-sessions-copy">
        Устройства, где вы вошли в ResursMap.
    </div>
    {rows}
    <form method="post" action="/app/sessions/revoke-others" class="rm-sessions-revoke-all">
        <button type="submit" class="ui-button rm-sessions-revoke-all-btn">
            Выйти на других устройствах
        </button>
    </form>
</section>"#,
        rows = rows,
    )
}

fn count_badge(count: i64) -> String {
    if count <= 0 {
        return String::new();
    }

    format!(r#"<span class="rm-command-badge">{count}</span>"#)
}

pub fn render_me(params: RenderMeParams<'_>) -> String {
    let RenderMeParams {
        authenticated,
        user_id,
        username,
        first_name,
        last_name,
        resources_count,
        approved_count,
        pending_count,
        rejected_count,
        favorites_count,
        unread_notifications_count,
        pending_contact_requests_count,
        unread_messages_count,
        moderator_level,
        intent_text,
        intent_until,
        category,
        home_continent_index,
        home_country_index,
        home_city_index,
        user_sessions,
    } = params;
    let safe_username = escape_html(username);
    let safe_first_name = escape_html(first_name);
    let safe_last_name = escape_html(last_name);
    let safe_intent_text = escape_html(intent_text);
    let selected_rubric = crate::catalog::resolve(category)
        .map(|rubric| rubric.id)
        .unwrap_or("");
    let work_options = crate::catalog::by_kind(crate::catalog::RubricKind::Work)
        .map(|rubric| {
            let selected = if selected_rubric == rubric.id {
                " selected"
            } else {
                ""
            };
            format!(
                r#"                <option value="{id}"{selected}>{label}</option>"#,
                id = escape_html(rubric.id),
                selected = selected,
                label = escape_html(rubric.label),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let business_options = crate::catalog::by_kind(crate::catalog::RubricKind::Business)
        .map(|rubric| {
            let selected = if selected_rubric == rubric.id {
                " selected"
            } else {
                ""
            };
            format!(
                r#"                <option value="{id}"{selected}>{label}</option>"#,
                id = escape_html(rubric.id),
                selected = selected,
                label = escape_html(rubric.label),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let intent_status_text = if safe_intent_text.is_empty() {
        "Статус не указан".to_string()
    } else if intent_until > 0 {
        safe_intent_text.to_string()
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
        "Пользователь".to_string()
    } else {
        "Гость".to_string()
    };

    let moderator_badge = if moderator_level > 0 {
        moderator_level_badge(moderator_level)
    } else {
        String::new()
    };

    let username_html = if !safe_username.is_empty() {
        format!(r#"<div class="rm-me-username">@{}</div>"#, safe_username)
    } else if authenticated {
        r#"<div class="rm-me-username rm-me-username--guest">Аккаунт</div>"#.to_string()
    } else {
        String::new()
    };

    let telegram_id_html = if authenticated {
        format!(
            r#"<div class="rm-me-account-id">ID аккаунта · {}</div>"#,
            user_id
        )
    } else {
        String::new()
    };

    let account_header = if authenticated {
        format!(
            r#"
<div class="card rm-me-account-card">

    <div class="rm-me-account-row">

        <div class="rm-me-avatar">
            {user_icon}
        </div>

        <div class="rm-me-name-wrap">

            <div class="rm-me-name">
                {display_name}
            </div>

            {username_html}
            {moderator_badge}

            {telegram_id_html}

        </div>

    </div>

</div>
"#,
            user_icon = icon("user"),
            display_name = display_name,
            username_html = username_html,
            telegram_id_html = telegram_id_html,
            moderator_badge = moderator_badge,
        )
    } else {
        format!(
            "{}{}",
            guest_mode_panel("/app/me"),
            navigation_card(
                "/app/search",
                "search",
                "Сначала поиск",
                "Найдите людей и ресурсы"
            ),
        )
    };

    let account_header = if authenticated {
        format!(
            r#"{account_header}
<form method="post"
      action="/app/logout"
      class="rm-me-logout-form">
    <button type="submit"
            class="ui-button rm-me-logout-btn">
        Выйти из аккаунта
    </button>
</form>
{sessions_panel}"#,
            account_header = account_header,
            sessions_panel = render_user_sessions_panel(&user_sessions),
        )
    } else {
        account_header
    };

    let statistics = if authenticated {
        format!(
            r#"
<div class="rm-me-stats">

    <div class="card rm-me-stat-card">
        <div class="rm-me-stat-value">
            {resources_count}
        </div>
        <div class="card-meta rm-me-stat-meta">
            Мои ресурсы
        </div>
    </div>

    <div class="card rm-me-stat-card">
        <div class="rm-me-stat-value">
            {favorites_count}
        </div>
        <div class="card-meta rm-me-stat-meta">
            Избранное
        </div>
    </div>

    <div class="card rm-me-stat-card">
        <div class="rm-me-stat-value rm-me-stat-value--md rm-me-stat-value--ok">
            {approved_count}
        </div>
        <div class="card-meta rm-me-stat-meta">
            Одобрено
        </div>
    </div>

    <div class="card rm-me-stat-card">
        <div class="rm-me-stat-value rm-me-stat-value--md rm-me-stat-value--warn">
            {pending_count}
        </div>
        <div class="card-meta rm-me-stat-meta">
            На проверке
        </div>
    </div>

</div>

<div class="card rm-me-rejected-row">

    <div class="card-content">
        <div class="card-title rm-me-rejected-title">
            Отклонено
        </div>

        <div class="card-meta rm-me-rejected-copy">
            Ресурсы, которым требуется исправление
        </div>
    </div>

    <div class="rm-me-rejected-count">
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

    let personal_center = if authenticated {
        let attention_count = pending_count
            .saturating_add(rejected_count)
            .saturating_add(unread_notifications_count)
            .saturating_add(pending_contact_requests_count)
            .saturating_add(unread_messages_count);

        let availability_class = "available";
        let availability_text = "Внутренние сообщения доступны";

        let category_display = crate::catalog::by_id(selected_rubric)
            .map(|rubric| rubric.label)
            .unwrap_or("Направление не выбрано");
        let category_text = category_display;

        let admin_navigation = if moderator_level > 0 {
            format!(
                r#"<a class="rm-command-card rm-admin-command"
                       href="/app/center">
                    <span class="rm-command-icon">{}</span>
                    <span class="rm-command-copy">
                        <strong>Центр управления</strong>
                        <small>
                            Административный уровень {}
                        </small>
                    </span>
                    <span class="rm-command-arrow">{}</span>
                </a>"#,
                icon("shield"),
                moderator_level,
                icon("chevron"),
            )
        } else {
            String::new()
        };

        format!(
            r#"
<style id="resursmap-personal-center-v1">
.rm-personal-center {{
    --center-gold: var(--gold);
    --center-green: var(--success);
    --center-blue: var(--info);
    --center-red: var(--danger);
    position:relative;
    overflow:hidden;
    margin-bottom:24px;
    padding:24px;
    border:1px solid rgba(232,204,150,.30);
    border-radius:26px;
    background:
        radial-gradient(circle at 100% 0%,
            rgba(126,212,228,.16),transparent 34%),
        radial-gradient(circle at 0% 100%,
            rgba(232,204,150,.14),transparent 36%),
        linear-gradient(145deg,
            rgba(20,23,30,.98),
            rgba(10,12,17,.98));
    box-shadow:
        0 24px 68px rgba(0,0,0,.30),
        0 0 48px rgba(232,204,150,.06);
}}
.rm-personal-center::after {{
    content:"";
    position:absolute;
    width:230px;
    height:230px;
    top:-145px;
    right:-115px;
    border:1px solid rgba(214,183,122,.17);
    border-radius:50%;
    box-shadow:
        0 0 0 35px rgba(214,183,122,.025),
        0 0 0 72px rgba(119,87,185,.025);
    pointer-events:none;
}}
.rm-center-kicker {{
    position:relative;
    z-index:1;
    color:var(--center-gold);
    font-size:10px;
    font-weight:950;
    letter-spacing:.18em;
}}
.rm-center-heading {{
    position:relative;
    z-index:1;
    margin:10px 0 7px;
    font-size:clamp(27px,7vw,43px);
    line-height:1;
    letter-spacing:-.04em;
}}
.rm-center-subtitle {{
    position:relative;
    z-index:1;
    max-width:620px;
    margin:0;
    color:var(--muted);
    font-size:13px;
    line-height:1.6;
}}
.rm-center-status {{
    position:relative;
    z-index:1;
    display:flex;
    flex-wrap:wrap;
    gap:8px;
    margin-top:17px;
}}
.rm-status-pill {{
    min-height:30px;
    display:inline-flex;
    align-items:center;
    gap:7px;
    padding:0 11px;
    border:1px solid rgba(255,255,255,.09);
    border-radius:999px;
    color:var(--muted);
    background:rgba(255,255,255,.025);
    font-size:10px;
    font-weight:850;
}}
.rm-status-pill::before {{
    content:"";
    width:7px;
    height:7px;
    border-radius:50%;
    background:var(--center-red);
}}
.rm-status-pill.available::before {{
    background:var(--center-green);
    box-shadow:0 0 12px rgba(98,224,173,.55);
}}
.rm-status-pill.private::before {{
    background:var(--center-red);
}}
.rm-center-metrics {{
    position:relative;
    z-index:1;
    display:grid;
    grid-template-columns:repeat(4,minmax(0,1fr));
    gap:9px;
    margin-top:20px;
}}
.rm-center-metric {{
    min-width:0;
    padding:14px;
    border:1px solid rgba(255,255,255,.09);
    border-radius:16px;
    background:
        linear-gradient(145deg, rgba(255,255,255,.04), rgba(255,255,255,.015));
    box-shadow:inset 0 1px 0 rgba(255,255,255,.05);
}}
.rm-center-metric strong {{
    display:block;
    font-size:23px;
    line-height:1;
    color:var(--gold-light);
    text-shadow:0 0 24px rgba(232,204,150,.12);
}}
.rm-center-metric span {{
    display:block;
    margin-top:7px;
    color:var(--muted);
    font-size:10px;
}}
.rm-center-metric.attention strong {{
    color:#ffc26f;
}}
.rm-command-section {{
    margin-bottom:24px;
}}
.rm-command-title {{
    display:flex;
    justify-content:space-between;
    align-items:end;
    gap:12px;
    margin-bottom:11px;
}}
.rm-command-title h2 {{
    margin:0;
    font-size:21px;
}}
.rm-command-title span {{
    color:var(--muted);
    font-size:11px;
}}
.rm-command-grid {{
    display:grid;
    grid-template-columns:repeat(2,minmax(0,1fr));
    gap:10px;
}}
.rm-command-card {{
    min-width:0;
    min-height:90px;
    display:flex;
    align-items:center;
    gap:12px;
    padding:15px;
    border:1px solid rgba(255,255,255,.08);
    border-radius:18px;
    color:var(--text);
    background:
        linear-gradient(145deg,
            rgba(255,255,255,.035),
            rgba(255,255,255,.015));
    text-decoration:none;
    transition:
        transform .18s ease,
        border-color .18s ease;
}}
.rm-command-card:hover {{
    transform:translateY(-3px);
    border-color:rgba(232,204,150,.36);
    background:
        linear-gradient(145deg,
            rgba(232,204,150,.10),
            rgba(126,212,228,.05));
    box-shadow:
        0 14px 36px rgba(0,0,0,.24),
        0 0 32px rgba(232,204,150,.08);
}}
.rm-command-icon {{
    flex:0 0 43px;
    height:43px;
    display:grid;
    place-items:center;
    border:1px solid rgba(232,204,150,.28);
    border-radius:14px;
    color:var(--center-gold);
    background:
        radial-gradient(circle at 30% 20%, rgba(255,228,184,.14), transparent 55%),
        rgba(232,204,150,.09);
    box-shadow:inset 0 1px 0 rgba(255,255,255,.06);
}}
.rm-command-icon svg {{
    width:20px;
    height:20px;
}}
.rm-command-copy {{
    min-width:0;
    flex:1;
}}
.rm-command-copy strong,
.rm-command-copy small {{
    display:block;
}}
.rm-command-copy strong {{
    overflow-wrap:anywhere;
    font-size:14px;
}}
.rm-command-copy small {{
    margin-top:5px;
    color:var(--muted);
    font-size:10px;
    line-height:1.4;
}}
.rm-command-arrow {{
    flex:0 0 auto;
    color:var(--muted);
}}
.rm-command-arrow svg {{
    width:17px;
    height:17px;
}}
.rm-command-badge {{
    min-width:25px;
    height:25px;
    display:grid;
    place-items:center;
    padding:0 7px;
    border:1px solid rgba(214,183,122,.32);
    border-radius:999px;
    color:var(--center-gold);
    background:rgba(214,183,122,.09);
    font-size:10px;
    font-weight:950;
}}
.rm-admin-command {{
    border-color:rgba(214,183,122,.24);
    background:
        linear-gradient(135deg,
            rgba(214,183,122,.075),
            rgba(119,87,185,.055));
}}
.rm-future-panel {{
    margin-bottom:24px;
    padding:18px;
    border:1px dashed rgba(214,183,122,.22);
    border-radius:19px;
    background:rgba(214,183,122,.025);
}}
.rm-future-panel strong {{
    display:block;
    color:var(--center-gold);
    font-size:14px;
}}
.rm-future-panel p {{
    margin:7px 0 0;
    color:var(--muted);
    font-size:11px;
    line-height:1.55;
}}
@media (max-width:680px) {{
    .rm-personal-center {{
        padding:20px;
    }}
    .rm-center-metrics {{
        grid-template-columns:repeat(2,minmax(0,1fr));
    }}
    .rm-command-grid {{
        grid-template-columns:1fr;
    }}
}}
@media (max-width:390px) {{
    .rm-center-metric {{
        padding:12px;
    }}
    .rm-command-card {{
        min-height:82px;
    }}
}}
@media (prefers-reduced-motion:reduce) {{
    .rm-command-card {{
        transition:none;
    }}
}}
html.light-theme .rm-status-pill,
body.light-theme .rm-status-pill {{
    background:#fff;
    border-color:rgba(26,29,33,.10);
}}
html.light-theme .rm-center-metric,
body.light-theme .rm-center-metric {{
    background:#fff;
    border-color:rgba(26,29,33,.10);
    box-shadow:0 8px 20px rgba(26,29,33,.05);
}}
html.light-theme .rm-command-icon,
body.light-theme .rm-command-icon {{
    background:rgba(165,118,31,.08);
}}
</style>

<section class="rm-personal-center">
    <h1 class="rm-center-heading">
        Обзор
    </h1>

    <p class="rm-center-subtitle">
        Ваш личный штурвал: ресурсы, связи,
        сообщения, активность и возможности
        в одном защищённом пространстве.
    </p>

    <div class="rm-center-status">
        <span class="rm-status-pill {availability_class}">
            {availability_text}
        </span>
        <span class="rm-status-pill">
            {category_text}
        </span>
    </div>

    <div class="rm-center-metrics">
        <div class="rm-center-metric">
            <strong>{resources_count}</strong>
            <span>моих ресурсов</span>
        </div>
        <div class="rm-center-metric">
            <strong>{approved_count}</strong>
            <span>опубликовано</span>
        </div>
        <div class="rm-center-metric">
            <strong>{favorites_count}</strong>
            <span>в избранном</span>
        </div>
        <div class="rm-center-metric attention">
            <strong>{attention_count}</strong>
            <span>требуют внимания</span>
        </div>
    </div>
</section>

<section class="rm-command-section">
    <div class="rm-command-title">
        <h2>Мои направления</h2>
        <span>Реальные разделы аккаунта</span>
    </div>

    <div class="rm-command-grid">
        <a class="rm-command-card"
           href="/app/add">
            <span class="rm-command-icon">
                {plus_icon}
            </span>
            <span class="rm-command-copy">
                <strong>Добавить объявление</strong>
                <small>
                    Откроется последний город
                </small>
            </span>
            <span class="rm-command-arrow">
                {arrow}
            </span>
        </a>

        <a class="rm-command-card"
           href="/app/my-resources">
            <span class="rm-command-icon">
                {resources_icon}
            </span>
            <span class="rm-command-copy">
                <strong>Мои ресурсы</strong>
                <small>
                    Опубликовано: {approved_count} ·
                    На проверке: {pending_count} ·
                    Отклонено: {rejected_count}
                </small>
            </span>
            <span class="rm-command-arrow">
                {arrow}
            </span>
        </a>

        <a class="rm-command-card"
           href="/app/contact-requests">
            <span class="rm-command-icon">
                {contacts_icon}
            </span>
            <span class="rm-command-copy">
                <strong>Запросы на связь</strong>
                <small>
                    Решайте, кто сможет связаться с вами
                </small>
            </span>
            {contacts_badge}
            <span class="rm-command-arrow">
                {arrow}
            </span>
        </a>

        <a class="rm-command-card"
           href="/app/favorites">
            <span class="rm-command-icon">
                {favorites_icon}
            </span>
            <span class="rm-command-copy">
                <strong>Избранное</strong>
                <small>
                    Сохранённые ресурсы: {favorites_count}
                </small>
            </span>
            <span class="rm-command-arrow">
                {arrow}
            </span>
        </a>

        <a class="rm-command-card"
           href="/app/notifications">
            <span class="rm-command-icon">
                {notifications_icon}
            </span>
            <span class="rm-command-copy">
                <strong>Уведомления</strong>
                <small>
                    События аккаунта и сообщества
                </small>
            </span>
            {notifications_badge}
            <span class="rm-command-arrow">
                {arrow}
            </span>
        </a>

        <a class="rm-command-card"
           href="/app/search">
            <span class="rm-command-icon">
                {search_icon}
            </span>
            <span class="rm-command-copy">
                <strong>Найти возможности</strong>
                <small>
                    Работа, работники и бизнес
                </small>
            </span>
            <span class="rm-command-arrow">
                {arrow}
            </span>
        </a>

        {admin_navigation}
    </div>
</section>


"#,
            availability_class = availability_class,
            availability_text = availability_text,
            category_text = category_text,
            resources_count = resources_count,
            approved_count = approved_count,
            pending_count = pending_count,
            rejected_count = rejected_count,
            favorites_count = favorites_count,
            attention_count = attention_count,
            resources_icon = icon("map"),
            plus_icon = icon("plus"),
            contacts_icon = icon("users"),
            favorites_icon = icon("heart"),
            notifications_icon = icon("bell"),
            search_icon = icon("search"),
            arrow = icon("chevron"),
            contacts_badge = count_badge(pending_contact_requests_count),
            notifications_badge = count_badge(unread_notifications_count),
            admin_navigation = admin_navigation,
        )
    } else {
        String::new()
    };

    let content_html = format!(
        r####"{personal_center}

{account_header}


{statistics}


<section class="card rm-profile-section">

    <div class="rm-profile-section-head">

        <div>
            <div class="card-title rm-profile-section-title">
                Мой статус
            </div>

            <div class="card-meta rm-profile-section-copy">
                Расскажите сообществу, что вы ищете
                или что можете предложить.
            </div>
        </div>

        <div class="rm-profile-icon-box">
            {settings_icon}
        </div>

    </div>


    <div class="rm-profile-intent-box">

        <div class="rm-profile-intent-kicker">
            Сейчас
        </div>

        <div id="intent-current" class="rm-profile-intent-text">
            {intent_status_text}
        </div>

    </div>





    <div class="rm-profile-settings-block">
        <div class="rm-profile-settings-kicker">
            Настройки
        </div>

        <a href="/app/menu"
           class="ui-button rm-profile-settings-link">
            ⚙️ Тема, звук и ярлык
        </a>
    </div>


    <label class="rm-profile-field">

        <div class="rm-profile-field-label">
            Что вы ищете или предлагаете
        </div>

        <textarea
            id="profile-intent"
            maxlength="300"
            rows="4"
            placeholder="Например: ищу электрика в Ницце или предлагаю грузоперевозки..."
         class="ui-textarea">{safe_intent_text}</textarea>

    </label>


    <label class="rm-profile-field rm-profile-field--spaced">

        <div class="rm-profile-field-label">
            Профессия или направление
        </div>

        <select
            id="profile-category"
            class="ui-input"
        >
            <option value="">Не выбрано</option>
            <optgroup label="Я работаю как">
{work_options}
            </optgroup>
            <optgroup label="Мой бизнес">
{business_options}
            </optgroup>
        </select>

    </label>

    {home_city_select}

    <label class="rm-profile-field rm-profile-field--spaced">

        <div class="rm-profile-field-label">
            Срок актуальности
        </div>

        <select id="profile-duration" class="ui-select">
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
     class="ui-button rm-profile-save-btn">
        Сохранить статус
    </button>


    <div id="profile-save-status" class="ui-status rm-profile-save-status">
    </div>

</section>"####,
        account_header = account_header,
        statistics = statistics,
        settings_icon = icon("settings"),
        intent_status_text = intent_status_text,
        safe_intent_text = safe_intent_text,
        work_options = work_options,
        business_options = business_options,
        home_city_select =
            home_city_select_html(home_continent_index, home_country_index, home_city_index,),
    );

    let body_after_html = r####"
<script>
(function () {
    "use strict";

    const DRAFT_KEY = "rm_profile_draft";

    const saveButton =
        document.getElementById("profile-save");

    const intent =
        document.getElementById("profile-intent");

    const categoryInput =
        document.getElementById(
            "profile-category"
        );

    const homeCity =
        document.getElementById(
            "profile-home-city"
        );

    const duration =
        document.getElementById(
            "profile-duration"
        );

    const status =
        document.getElementById(
            "profile-save-status"
        );

    const current =
        document.getElementById(
            "intent-current"
        );

    if (
        !saveButton ||
        !intent ||
        !categoryInput ||
        !duration
    ) {
        return;
    }

    function profileErrorText(code) {
        switch (code) {
            case "invalid_category":
                return "Выберите профессию из списка.";
            case "invalid_intent":
                return "Текст статуса слишком длинный.";
            case "invalid_duration":
                return "Выберите срок показа статуса.";
            case "login_required":
                return "Войдите в аккаунт.";
            case "rate_limited":
                return "Слишком часто. Подождите немного.";
            case "database_error":
            case "database_unavailable":
                return "Не удалось сохранить. Попробуйте позже.";
            default:
                return "Не удалось сохранить.";
        }
    }

    function collectDraft() {
        return {
            intent: intent.value,
            category: categoryInput.value,
            home_city: homeCity ? homeCity.value : "",
            duration: duration.value
        };
    }

    function persistDraft() {
        try {
            localStorage.setItem(
                DRAFT_KEY,
                JSON.stringify(collectDraft())
            );
        } catch (_) {}
    }

    function restoreDraft() {
        try {
            const raw = localStorage.getItem(DRAFT_KEY);
            if (!raw) {
                return;
            }

            const draft = JSON.parse(raw);
            if (typeof draft.intent === "string") {
                intent.value = draft.intent;
            }
            if (typeof draft.category === "string") {
                categoryInput.value = draft.category;
            }
            if (homeCity && typeof draft.home_city === "string") {
                homeCity.value = draft.home_city;
            }
            if (typeof draft.duration === "string") {
                duration.value = draft.duration;
            }
        } catch (_) {}
    }

    restoreDraft();

    intent.addEventListener("input", persistDraft);
    categoryInput.addEventListener("change", persistDraft);
    duration.addEventListener("change", persistDraft);
    if (homeCity) {
        homeCity.addEventListener("change", persistDraft);
    }
    window.addEventListener("beforeunload", persistDraft);

    saveButton.addEventListener(
        "click",
        async function () {
            saveButton.disabled = true;
            persistDraft();

            if (status) {
                status.textContent = "Сохраняем...";
            }

            try {
                const response = await fetch(
                    "/api/profile",
                    {
                        method: "POST",
                        headers: {
                            "Content-Type":
                                "application/json"
                        },
                        body: JSON.stringify({
                            intent_text:
                                intent.value.trim(),
                            duration_days:
                                Number(duration.value),
                            category:
                                categoryInput.value.trim(),
                            home_city:
                                homeCity
                                    ? homeCity.value.trim()
                                    : ""
                        })
                    }
                );

                const data = await response.json();

                if (response.status === 401 || data.error === "login_required") {
                    if (status) {
                        status.textContent =
                            "Войдите в аккаунт.";
                    }

                    return;
                }

                if (!response.ok || !data.ok) {
                    if (status) {
                        status.textContent =
                            profileErrorText(data.error);
                    }

                    return;
                }

                try {
                    localStorage.removeItem(DRAFT_KEY);
                } catch (_) {}

                if (current) {
                    current.textContent =
                        data.intent_text ||
                        "Статус не указан";
                }

                if (status) {
                    status.textContent =
                        "✓ Статус сохранён";
                }
            } catch (_) {
                if (status) {
                    status.textContent =
                        "Ошибка соединения.";
                }
            } finally {
                saveButton.disabled = false;
            }
        }
    );
})();
</script>"####
        .to_string();

    let main_html = format!(
        "{topbar}\n\n{hero}\n\n{content}",
        topbar = topbar("Профиль", "user"),
        hero = simple_hero(
            "user",
            "ResursMap",
            "Профиль",
            "Ваши ресурсы, сохранённые места и активность.",
        ),
        content = content_html,
    );

    page_document(
        "Профиль · ResursMap",
        "",
        "",
        &main_html,
        &bottom_nav_with_badge("menu", unread_messages_count),
        &body_after_html,
    )
}

// ============================================================
// USER NOTIFICATIONS
// ============================================================

pub fn render_notifications(
    notifications: Vec<crate::web::view_models::NotificationRow>,
    authenticated: bool,
) -> String {
    let cards = if !authenticated {
        guest_locked_section("Уведомления", "/app/notifications")
    } else if notifications.is_empty() {
        empty_state_card(
            "Уведомлений нет",
            "Здесь появятся результаты модерации и важные изменения ваших ресурсов.",
        )
    } else {
        notifications
            .iter()
            .map(
                |(_notification_id, resource_id, kind, title, message, is_read, _created_at)| {
                    let safe_title = escape_html(title);

                    let safe_message = escape_html(message);

                    let (icon_html, card_class, icon_class) = match kind.as_str() {
                        "resource_approved" => (icon("check"), "rm-notif-card--approved", "rm-notif-icon--approved"),
                        "resource_rejected" => (icon("x"), "rm-notif-card--rejected", "rm-notif-icon--rejected"),
                        "promotion_published" => (icon("star"), "rm-notif-card--approved", "rm-notif-icon--approved"),
                        "promotion_moderation" => (icon("clock"), "rm-notif-card--contact", "rm-notif-icon--contact"),
                        "promotion_publish_failed" => (icon("alert-triangle"), "rm-notif-card--rejected", "rm-notif-icon--rejected"),
                        "promotion_rejected" => (icon("x"), "rm-notif-card--rejected", "rm-notif-icon--rejected"),
                        "admin_assignment" => (icon("shield"), "rm-notif-card--contact", "rm-notif-icon--contact"),
                        "chat_message" => (icon("message-circle"), "rm-notif-card--chat", "rm-notif-icon--chat"),
                        "contact_accepted" => (icon("check"), "rm-notif-card--contact", "rm-notif-icon--contact"),
                        "contact_rejected" => (icon("x"), "rm-notif-card--rejected", "rm-notif-icon--rejected"),
                        _ => (icon("bell"), "", "rm-notif-icon--default"),
                    };

                    let unread_badge = if *is_read == 0 {
                        r#"<span class="rm-notif-new">Новое</span>"#
                    } else {
                        ""
                    };

                    let open_link = if kind == "chat_message" {
                        if let Some(id) = resource_id {
                            format!(
                                r#"<a href="/app/chat/{id}" class="rm-notif-action rm-notif-action--gold">Открыть чат</a>"#,
                                id = id
                            )
                        } else {
                            r#"<a href="/app/messages" class="rm-notif-action rm-notif-action--gold">Открыть сообщения</a>"#
                                .to_string()
                        }
                    } else if kind == "contact_accepted" {
                        r#"<a href="/app/messages" class="rm-notif-action rm-notif-action--gold">Открыть сообщения</a>"#
                            .to_string()
                    } else if let Some(id) = resource_id {
                        format!(
                            r#"<a href="/app/resource/{id}" class="rm-notif-action rm-notif-action--neutral">Открыть ресурс</a>"#,
                            id = id
                        )
                    } else {
                        String::new()
                    };

                    format!(
                        r#"
<article class="card rm-notif-card {card_class}">
    <div class="rm-notif-layout">
        <div class="rm-notif-icon {icon_class}">{icon_html}</div>
        <div class="rm-notif-body">
            <div class="rm-notif-head">
                <div class="card-title">{title}</div>
                {unread_badge}
            </div>
            <div class="card-meta rm-notif-message">{message}</div>
            {open_link}
        </div>
    </div>
</article>
"#,
                        card_class = card_class,
                        icon_class = icon_class,
                        icon_html = icon_html,
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

    let content = format!(
        r#"<section>
    {cards}
</section>"#,
        cards = cards,
    );

    page_shell(
        "Уведомления · ResursMap",
        &topbar("Уведомления", "bell"),
        &back_hero(
            &back_link("/app/me", "Профиль", "arrow-left"),
            "user",
            "Уведомления",
            "Центр уведомлений",
            "Статусы модерации и важные изменения ваших ресурсов.",
        ),
        &content,
        &bottom_nav("menu"),
    )
}

// ============================================================
// CATEGORY
// ============================================================

pub struct RenderPublicUserProfileParams<'a> {
    pub public_id: &'a str,
    pub username: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub intent_text: &'a str,
    pub category: &'a str,
    pub chat_user_id: Option<i64>,
    pub resources: Vec<crate::web::view_models::PublicProfileResourceRow>,
}

pub fn render_public_user_profile(params: RenderPublicUserProfileParams<'_>) -> String {
    let RenderPublicUserProfileParams {
        public_id,
        username,
        first_name,
        last_name,
        intent_text,
        category,
        chat_user_id,
        resources,
    } = params;
    let profession = {
        let label = profession_label(category);
        if label.is_empty()
            || matches!(
                label.as_str(),
                "Работа" | "Бизнес" | "Услуги" | "Сообщество"
            )
        {
            "Специалист".to_string()
        } else {
            label
        }
    };
    let hero_full_name = format!("{} {}", first_name.trim(), last_name.trim(),)
        .trim()
        .to_string();

    let person_name = if !hero_full_name.is_empty() {
        hero_full_name
    } else if !username.trim().is_empty() {
        format!("@{}", username.trim())
    } else {
        String::new()
    };

    let safe_username = escape_html(username);
    let safe_first_name = escape_html(first_name);
    let safe_last_name = escape_html(last_name);
    let safe_intent = escape_html(intent_text);

    let full_name = format!("{} {}", safe_first_name, safe_last_name)
        .trim()
        .to_string();

    let display_name = profession.clone();
    let person_line = if !full_name.is_empty() {
        full_name
    } else if !safe_username.is_empty() {
        format!("@{}", safe_username)
    } else {
        person_name
    };

    let contact_html = String::new();

    let public_id_js = serde_json::to_string(public_id).unwrap_or_else(|_| "\"\"".to_string());

    let internal_contact_html = if let Some(chat_user_id) = chat_user_id {
        format!(
            r#"
<section class="card rm-public-section">

    <div class="rm-public-kicker">
        Написать
    </div>

    <div class="card-meta rm-public-copy">
        Открыть личный диалог.
    </div>

    <a href="/app/chat/{chat_user_id}" class="rm-public-chat-link">
        Открыть чат
    </a>
    <button type="button" class="ui-button" data-share data-share-title="Профиль ResursMap" data-share-status="share-status">Поделиться</button>
    <div id="share-status" class="ui-status"></div>

</section>
"#,
            chat_user_id = chat_user_id,
        )
    } else {
        r#"
<section class="card rm-public-section">

    <div class="rm-public-kicker">
        Написать
    </div>

    <div class="card-meta rm-public-copy">
        Отправьте запрос.
    </div>

    <button
        id="contact-request-open"
        type="button"
        class="ui-button rm-public-contact-btn">
        Написать
    </button>

    <div id="contact-request-panel" class="rm-public-contact-panel">

        <textarea
            id="contact-request-message"
            maxlength="500"
            rows="5"
            placeholder="Напишите короткое сообщение..."
            class="ui-textarea"></textarea>

        <div class="rm-public-contact-actions">

            <button
                id="contact-request-send"
                type="button"
                class="ui-button rm-public-contact-send">
                Отправить
            </button>

            <button
                id="contact-request-cancel"
                type="button"
                class="ui-button rm-public-contact-cancel">
                Отмена
            </button>

        </div>

        <div id="contact-request-status" class="ui-status rm-public-contact-status">
        </div>

    </div>

    <button type="button" class="ui-button" data-share data-share-title="Профиль ResursMap" data-share-status="share-status">Поделиться</button>
    <div id="share-status" class="ui-status"></div>

</section>
"#
        .to_string()
    };

    let intent_html = if safe_intent.is_empty() {
        String::new()
    } else {
        format!(
            r#"
<section class="card rm-public-section rm-public-section--intent">

    <div class="rm-public-kicker">
        Актуальный статус
    </div>

    <div class="rm-public-intent-body">{intent}</div>

</section>
"#,
            intent = safe_intent
        )
    };

    let resource_count = resources.len();

    let cards = if resources.is_empty() {
        empty_state_card(
            "Ресурсы не опубликованы",
            "В профиле нет опубликованных ресурсов.",
        )
    } else {
        resources
            .iter()
            .map(
                |(id, title, category, description, rating, votes, verified, premium)| {
                    let title = escape_html(title);
                    let category = escape_html(category);
                    let description = escape_html(description);

                    let verified_badge = if *verified != 0 {
                        verified_badge_html(true)
                    } else {
                        ""
                    };

                    let premium_badge = if *premium != 0 {
                        premium_badge_html("compact")
                    } else {
                        ""
                    };

                    profile_resource_card(super::common::ProfileResourceCardParams {
                        href: &format!("/app/resource/{}", id),
                        icon_name: "map-pin",
                        title: &title,
                        category: &category,
                        description: &description,
                        address: None,
                        rating: *rating,
                        votes: *votes,
                        premium_badge_html: premium_badge,
                        verified_badge_html: verified_badge,
                    })
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let section_head_resources =
        section_head("Ресурсы участника", "Только активные и одобренные", None);

    let main_html = format!(
        r####"<section class="card rm-public-profile-card">

    <div class="rm-public-profile-row">

        <div class="rm-me-avatar">
            {profile_icon}
        </div>

        <div class="rm-me-name-wrap">

            <div class="rm-public-name">
                {display_name}
            </div>

            <div class="card-meta rm-public-resource-meta">
                {person_line}
            </div>

            <div class="card-meta rm-public-resource-meta">
                {resource_count} ресурсов
            </div>

        </div>

    </div>

    {contact_html}

</section>


{intent_html}

{internal_contact_html}


{section_head_resources}


<section>
    {cards}
</section>"####,
        profile_icon = icon("user"),
        display_name = display_name,
        person_line = if person_line.is_empty() {
            String::new()
        } else {
            person_line.clone()
        },
        resource_count = resource_count,
        contact_html = contact_html,
        intent_html = intent_html,
        internal_contact_html = internal_contact_html,
        section_head_resources = section_head_resources,
        cards = cards,
    );

    let body_after = format!(
        r####"



<script>
(function() {{
    const publicId = {public_id_js};

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
                    window.location.href =
                        "/login?next="
                        + encodeURIComponent(
                            window.location.pathname
                                + window.location.search
                        );
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
                            "Диалог уже создаётся.";
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
                            "Диалог уже открыт.";
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

                if (data.status === "pending") {{
                    if (status) {{
                        status.textContent =
                            "Сообщение отправлено. Открываем чат…";
                    }}

                    if (data.chat_url) {{
                        window.location.href = data.chat_url;
                        return;
                    }}
                }}

                if (status) {{
                    status.textContent =
                        "Открываем диалог…";
                }}

                if (message) {{
                    message.value = "";
                }}

                openButton.disabled = false;
                openButton.textContent =
                    "Открыть чат";

                if (data.chat_url) {{
                    window.location.href =
                        data.chat_url;
                    return;
                }}

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
</script>"####,
        public_id_js = public_id_js,
    );

    page_document(
        &format!("{} · ResursMap", display_name),
        r#"<meta name="robots" content="noindex, nofollow">"#,
        "",
        &format!(
            "{topbar}\n\n{hero}\n\n{content}",
            topbar = topbar("Профессия", "user"),
            hero = back_hero(
                &back_link("/app/search", "Назад", "arrow-left",),
                "user",
                "Профессия",
                &profession,
                if person_line.is_empty() {
                    "Объявления и статус по выбранной рубрике."
                } else {
                    person_line.as_str()
                },
            ),
            content = main_html,
        ),
        &bottom_nav("search"),
        &body_after,
    )
}

pub fn render_public_user_not_found() -> String {
    let back_to_map = navigation_card("/app", "map", "Вернуться к городам", "");

    let content = format!(
        r#"<section>
    {back_to_map}
</section>"#,
        back_to_map = back_to_map,
    );

    page_shell(
        "Профиль не найден · ResursMap",
        "",
        &simple_hero(
            "alert-triangle",
            "ResursMap",
            "Профиль не найден",
            "Пользователь недоступен или публичный профиль ещё не создан.",
        ),
        &content,
        "",
    )
}

// ============================================================
// ПРОФИЛЬ РЕСУРСА
// ============================================================

pub fn render_favorites(
    resources: Vec<crate::web::view_models::FavoriteResourceRow>,
    authenticated: bool,
) -> String {
    let cards = if !authenticated {
        guest_locked_section("Избранное", "/app/favorites")
    } else if resources.is_empty() {
        empty_state_card_with_actions(
            "Избранное пока пустое",
            "Откройте любое объявление и сохраните его в избранное.",
            &format!(
                "{}{}",
                empty_state_action("/app/search", "Найти ресурсы"),
                empty_state_action("/app", "На карту"),
            ),
        )
    } else {
        resources
            .iter()
            .map(
                |(id, title, category, description, address, rating, votes, verified, premium)| {
                    let premium_badge = if *premium != 0 {
                        premium_badge_html("compact")
                    } else {
                        ""
                    };

                    let verified_badge = if *verified != 0 {
                        verified_badge_html(true)
                    } else {
                        ""
                    };

                    profile_resource_card(super::common::ProfileResourceCardParams {
                        href: &format!("/app/resource/{}", id),
                        icon_name: "heart",
                        title,
                        category,
                        description,
                        address: Some(address.as_str()),
                        rating: *rating,
                        votes: *votes,
                        premium_badge_html: premium_badge,
                        verified_badge_html: verified_badge,
                    })
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let content = format!(
        r#"<section>
    {cards}
</section>"#,
        cards = cards,
    );

    page_shell(
        "Избранное · ResursMap",
        &topbar("Избранное", "heart"),
        &back_hero(
            &back_link("/app/me", "Профиль", "arrow-left"),
            "heart",
            "Избранное",
            "Сохранённые ресурсы",
            "Всё, что вы отметили сердцем.",
        ),
        &content,
        &bottom_nav("menu"),
    )
}

#[cfg(test)]
mod personal_center_tests {
    use super::*;

    fn params(authenticated: bool) -> RenderMeParams<'static> {
        RenderMeParams {
            authenticated,
            user_id: 42,
            username: "captain",
            first_name: "Amir",
            last_name: "",
            resources_count: 7,
            approved_count: 4,
            pending_count: 2,
            rejected_count: 1,
            favorites_count: 3,
            unread_notifications_count: 5,
            pending_contact_requests_count: 2,
            unread_messages_count: 6,
            moderator_level: 0,
            intent_text: "Ищу партнёров",
            intent_until: 0,
            category: "Бизнес",
            home_continent_index: -1,
            home_country_index: -1,
            home_city_index: -1,
            user_sessions: vec![],
        }
    }

    #[test]
    fn personal_center_is_rendered_for_authenticated_user() {
        let html = render_me(params(true));

        assert!(html.contains("Обзор"));
        assert!(html.contains("/app/my-resources"));
        assert!(html.contains("/app/contact-requests"));
        assert!(html.contains("/app/favorites"));
        assert!(html.contains("/app/notifications"));
        assert!(html.contains("Реальные разделы аккаунта"));
    }

    #[test]
    fn personal_center_is_hidden_from_guest() {
        let html = render_me(params(false));

        assert!(!html.contains("RESURSMAP · PERSONAL COMMAND"));
        assert!(html.contains("Войдите в аккаунт"));
    }

    #[test]
    fn personal_center_escapes_profile_data() {
        let mut unsafe_params = params(true);

        unsafe_params.category = "<script>alert(1)</script>";

        let html = render_me(unsafe_params);

        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("Направление не выбрано"));
    }

    #[test]
    fn public_profile_leads_with_profession() {
        let html = render_public_user_profile(RenderPublicUserProfileParams {
            public_id: "abc",
            username: "ivan",
            first_name: "Иван",
            last_name: "Петров",
            intent_text: "",
            category: "security",
            chat_user_id: None,
            resources: vec![],
        });

        let profession = html.find("Охрана").expect("profession");
        let name = html.find("Иван").expect("name");
        assert!(profession < name);
        assert!(html.contains("Профессия"));
        assert!(html.contains("noindex"));
    }
}
