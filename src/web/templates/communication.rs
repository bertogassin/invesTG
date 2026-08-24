use super::common::{base_style, icon};

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
