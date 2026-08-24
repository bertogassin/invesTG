mod communication;
pub use communication::*;

mod navigation;
pub use navigation::*;

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

use std::collections::BTreeMap;

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
    let safe_category = escape_html(category);
    let category_url = urlencoding::encode(category);

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
            ci, si, zi, category_url
        )
    } else {
        resources
            .iter()
            .map(|(id, title, description, contact, address, rating, votes, verified, premium)| {
                let safe_title = escape_html(title);
                let safe_description = escape_html(description);
                let safe_contact = escape_html(contact);
                let safe_address = escape_html(address);

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
                    safe_title,
                    premium_badge,
                    safe_description,
                    rating,
                    votes,
                    safe_address,
                    safe_contact,
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
        category = safe_category,
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

    let public_id_js = serde_json::to_string(public_id).unwrap_or_else(|_| "\"\"".to_string());

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
        public_id_js = public_id_js,
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
    let safe_title = escape_html(title);
    let safe_description = escape_html(description);
    let safe_contact = escape_html(contact);
    let safe_address = escape_html(address);
    let safe_category = escape_html(category);

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
            public_id = urlencoding::encode(owner_public_id),
        )
    };

    let map_query = urlencoding::encode(address.trim());

    let map_href = format!(
        "https://www.google.com/maps/search/?api=1&query={}",
        map_query
    );

    let safe_contact_href = escape_html(&contact_href);
    let safe_map_href = escape_html(&map_href);

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
        category = safe_category,
        title = safe_title,
        description = safe_description,
        owner_profile_html = owner_profile_html,
        rating = rating,
        votes = votes,
        address = safe_address,
        contact = safe_contact,
        contact_href = safe_contact_href,
        map_href = safe_map_href,
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
                let safe_title = escape_html(title);
                let safe_category = escape_html(category);
                let safe_description = escape_html(description);
                let safe_rejection_reason = escape_html(rejection_reason);

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
                            safe_rejection_reason
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
                    title = safe_title,
                    category = safe_category,
                    description = safe_description,
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
    let safe_title = escape_html(title);
    let safe_description = escape_html(description);
    let safe_contact = escape_html(contact);
    let safe_address = escape_html(address);
    let safe_category = escape_html(category);

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
        title = safe_title,
        description = safe_description,
        contact = safe_contact,
        address = safe_address,
        category = safe_category,
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
