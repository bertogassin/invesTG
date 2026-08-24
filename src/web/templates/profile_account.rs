use super::common::{base_style, escape_html, icon};

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
