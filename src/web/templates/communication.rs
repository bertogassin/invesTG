use super::common::{
    back_hero, back_link, bottom_nav, contact_request_status_badge, empty_state_action,
    empty_state_card, empty_state_card_with_actions, escape_html, guest_locked_section, icon,
    page_shell, section_head, static_asset, topbar,
};

pub fn render_contact_requests(
    requests: Vec<crate::web::view_models::ContactRequestRow>,
    authenticated: bool,
) -> String {
    let pending_count = requests.iter().filter(|r| r.3 == "pending").count();

    let cards = if !authenticated {
        guest_locked_section("Запросы на связь", "/app/contact-requests")
    } else if requests.is_empty() {
        empty_state_card_with_actions(
            "Нет входящих запросов",
            "Новые запросы будут отображаться здесь.",
            &empty_state_action("/app/search", "Найти участников"),
        )
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
                        format!("Участник · {:06}", sender_user_id.rem_euclid(1_000_000))
                    };

                    let username_html = if !safe_username.is_empty() {
                        format!(
                            r#"<div class="card-meta rm-contact-username">@{username}</div>"#,
                            username = safe_username,
                        )
                    } else {
                        String::new()
                    };

                    let profile_link = if !public_id.trim().is_empty() {
                        format!(
                            r#"<a href="/app/user/{public_id}" class="rm-contact-profile-link">Открыть профиль</a>"#,
                            public_id = public_id,
                        )
                    } else {
                        String::new()
                    };

                    let status_badge = contact_request_status_badge(status);

                    let actions = if status == "pending" {
                        format!(
                            r#"
<div class="rm-contact-actions">
    <form method="post" action="/app/contact-request/{id}/accept" class="ui-form">
        <button type="submit" class="ui-button rm-contact-btn rm-contact-btn--accept">✓ Принять</button>
    </form>
    <form method="post" action="/app/contact-request/{id}/reject" class="ui-form">
        <button type="submit" class="ui-button rm-contact-btn rm-contact-btn--reject">✕ Отклонить</button>
    </form>
</div>
"#,
                            id = request_id,
                        )
                    } else if status == "accepted" {
                        format!(
                            r#"<a href="/app/chat/{sender_user_id}" class="rm-contact-open-chat">💬 Открыть чат</a>"#,
                            sender_user_id = sender_user_id,
                        )
                    } else {
                        String::new()
                    };

                    format!(
                        r#"
<article class="card rm-contact-card">
    <div class="rm-contact-layout">
        <div class="card-icon">{user_icon}</div>
        <div class="rm-contact-body">
            <div class="rm-contact-head">
                <div>
                    <div class="card-title rm-contact-title">{display_name}</div>
                    {username_html}
                </div>
                <div>{status_badge}</div>
            </div>
            <div class="rm-contact-message">{message}</div>
            {profile_link}
            {actions}
            <div class="card-meta rm-contact-meta">REQUEST #{request_id} · {created_at}</div>
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

    let content_html = format!(
        r####"<div class="card rm-contact-summary">

    <div class="card-content">

        <div class="card-title">
            Новые запросы
        </div>

        <div class="card-meta rm-contact-summary-copy">
            Ожидают вашего решения
        </div>

    </div>

    <div class="rm-contact-summary-count">
        {pending_count}
    </div>

</div>


<section>

    {cards}

</section>"####,
        pending_count = pending_count,
        cards = cards,
    );

    page_shell(
        "Запросы · ResursMap",
        &topbar("Запросы контактов", "users"),
        &back_hero(
            &back_link("/app/me", "Профиль", "arrow-left"),
            "user",
            "Связи",
            "Запросы на связь",
            "Управление ранее полученными запросами.",
        ),
        &content_html,
        &bottom_nav("profile"),
    )
}

// ============================================================
// TASK 7.22G-C — MESSAGES LIST
// ============================================================

pub(crate) fn conversation_display_name(
    other_user_id: i64,
    username: &str,
    first_name: &str,
    last_name: &str,
) -> String {
    let safe_username = escape_html(username);
    let safe_first_name = escape_html(first_name);
    let safe_last_name = escape_html(last_name);

    let full_name = format!("{safe_first_name} {safe_last_name}")
        .trim()
        .to_string();

    if !full_name.is_empty() {
        full_name
    } else if !safe_username.is_empty() {
        format!("@{safe_username}")
    } else {
        format!("Участник · {:06}", other_user_id.rem_euclid(1_000_000))
    }
}

pub(crate) fn format_inbox_time(updated_at: i64) -> String {
    if updated_at <= 0 {
        return String::new();
    }

    let paris = chrono_tz::Europe::Paris;
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(updated_at, 0)
        .map(|dt| dt.with_timezone(&paris));

    let Some(dt) = datetime else {
        return String::new();
    };

    let today = chrono::Utc::now().with_timezone(&paris).date_naive();
    let date = dt.date_naive();

    if date == today {
        dt.format("%H:%M").to_string()
    } else if date == today - chrono::Duration::days(1) {
        "Вчера".to_string()
    } else if today.signed_duration_since(date).num_days() < 7 {
        dt.format("%a").to_string()
    } else {
        dt.format("%d.%m").to_string()
    }
}

pub fn render_messages(
    authenticated: bool,
    conversations: Vec<crate::web::view_models::ConversationRow>,
) -> String {
    let total_unread: i64 = conversations.iter().map(|c| c.unread_count).sum();

    let content = if !authenticated {
        guest_locked_section("Сообщения", "/app/messages")
    } else if conversations.is_empty() {
        empty_state_card_with_actions(
            "Нет диалогов",
            "Откройте профиль участника, чтобы начать диалог.",
            &empty_state_action("/app/search", "Найти участников"),
        )
    } else {
        conversations
            .iter()
            .map(|conversation| {
                let other_user_id = conversation.other_user_id;
                let username = &conversation.username;
                let first_name = &conversation.first_name;
                let last_name = &conversation.last_name;
                let last_message = &conversation.last_message;
                let unread_count = conversation.unread_count;
                let updated_at = conversation.updated_at;
                let safe_username = escape_html(username);

                let display_name =
                    conversation_display_name(other_user_id, username, first_name, last_name);

                let safe_last_message = escape_html(last_message);

                let username_html = if !safe_username.is_empty() {
                    format!(
                        r#"<div class="card-meta rm-dialog-username">@{username}</div>"#,
                        username = safe_username,
                    )
                } else {
                    String::new()
                };

                let has_last_message = !safe_last_message.is_empty();

                let last_time = if has_last_message {
                    format_inbox_time(updated_at)
                } else {
                    String::new()
                };

                let last_message_html = if has_last_message {
                    safe_last_message
                } else {
                    "Новый диалог".to_string()
                };

                let unread_html = if unread_count > 0 {
                    format!(
                        r#"<span class="chat-dialog-unread">{count}</span>"#,
                        count = unread_count,
                    )
                } else {
                    String::new()
                };

                format!(
                    r#"
<a href="/app/chat/{other_user_id}#chat-end"
   class="card chat-dialog-card"
   data-other-user-id="{other_user_id}">

    <div class="card-icon chat-dialog-avatar">
        {chat_icon}
    </div>

    <div class="card-content">

        <div class="card-title">
            {display_name}
        </div>

        {username_html}

        <div class="card-meta chat-dialog-preview">
            {last_message}
        </div>

    </div>

    <div class="chat-dialog-side">

        <div class="chat-dialog-time">
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
            })
            .collect::<Vec<_>>()
            .join("")
    };

    let section_head_dialogs = if authenticated {
        format!(
            r#"<div class="section-head" id="inbox-section-head">
    <div>
        <h2 class="section-title">Диалоги</h2>
        <p class="section-caption" id="inbox-unread-caption">Непрочитанных: {total_unread}</p>
    </div>
    <span class="inbox-live-badge" id="inbox-live-badge" hidden aria-hidden="true">live</span>
</div>"#,
            total_unread = total_unread,
        )
    } else {
        section_head("Диалоги", &format!("Непрочитанных: {}", total_unread), None)
    };

    let list_attributes = if authenticated {
        r#" id="chat-dialog-list" data-inbox-live="1""#
    } else {
        ""
    };

    let inbox_script = if authenticated {
        format!(
            r#"<script src="{inbox_js}" defer></script>"#,
            inbox_js = static_asset("inbox.js"),
        )
    } else {
        String::new()
    };

    let content_html = format!(
        r####"<link rel="stylesheet"
      href="{chat_css}">

{section_head_dialogs}


<section class="chat-dialog-list"{list_attributes}>

    {content}

</section>

{inbox_script}"####,
        chat_css = static_asset("chat-v2.css"),
        section_head_dialogs = section_head_dialogs,
        list_attributes = list_attributes,
        content = content,
        inbox_script = inbox_script,
    );

    page_shell(
        "Сообщения · ResursMap",
        &topbar("Сообщения", "message-circle"),
        &back_hero(
            &back_link("/app/me", "Назад", "chevron-left"),
            "message-circle",
            "Внутренняя связь",
            "Сообщения",
            "Личные сообщения и активные диалоги.",
        ),
        &content_html,
        &bottom_nav("profile"),
    )
}

// ============================================================
// TASK 7.22F-E — CHAT
// ============================================================

fn chat_message_body_html(message: &crate::web::view_models::ChatMessageRow) -> String {
    if message.deleted_at > 0 {
        return r#"<div class="chat-message-body is-deleted">Сообщение удалено</div>"#.to_string();
    }

    if message.attachment_kind == "voice" && !message.attachment_url.is_empty() {
        return format!(
            r#"<div class="chat-message-body chat-message-body--voice"><div class="chat-voice-player"><audio controls preload="metadata" class="chat-voice-audio" src="{url}"></audio></div></div>"#,
            url = escape_html(&message.attachment_url),
        );
    }

    if message.attachment_kind == "image" && !message.attachment_url.is_empty() {
        let caption_html = if message.message.is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="chat-message-caption">{}</div>"#,
                escape_html(&message.message)
            )
        };

        return format!(
            r#"<div class="chat-message-body"><img class="chat-message-image" src="{url}" alt="Фото" loading="lazy" role="button" tabindex="0">{caption}</div>"#,
            url = escape_html(&message.attachment_url),
            caption = caption_html,
        );
    }

    format!(
        r#"<div class="chat-message-body">{}</div>"#,
        escape_html(&message.message)
    )
}

fn chat_reply_author_label(
    reply_sender_user_id: i64,
    viewer_user_id: i64,
    other_user_id: i64,
) -> &'static str {
    if reply_sender_user_id <= 0 {
        return "Сообщение";
    }

    if reply_sender_user_id == viewer_user_id {
        "Вы"
    } else if reply_sender_user_id == other_user_id {
        "Собеседник"
    } else {
        "Сообщение"
    }
}

fn render_chat_message_row(
    message: &crate::web::view_models::ChatMessageRow,
    viewer_user_id: i64,
    other_user_id: i64,
    last_date_key: &mut String,
) -> String {
    let mine = message.sender_user_id != other_user_id;
    let mine_attribute = if mine { "1" } else { "0" };
    let deleted = message.deleted_at > 0;

    let status = if mine {
        if message.read_at > 0 {
            ("✓✓", " is-read")
        } else if message.delivered_at > 0 {
            ("✓✓", "")
        } else if message.is_read != 0 {
            ("✓✓", " is-read")
        } else {
            ("✓", "")
        }
    } else {
        ("", "")
    };

    let paris = chrono_tz::Europe::Paris;
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(message.created_at, 0)
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

    let date_separator = if !date_key.is_empty() && date_key != *last_date_key {
        *last_date_key = date_key.clone();

        format!(
            r#"
<div class="chat-date-chip-wrap">
    <span class="chat-date-chip">{date_label}</span>
</div>
"#
        )
    } else {
        String::new()
    };

    let row_class = if mine {
        "chat-message-row is-mine"
    } else {
        "chat-message-row is-theirs"
    };

    let bubble_class = if deleted {
        "chat-bubble is-deleted"
    } else {
        "chat-bubble"
    };

    let reply_html = if message.reply_to_message_id > 0 {
        let reply_preview = escape_html(&message.reply_message);
        let reply_author =
            chat_reply_author_label(message.reply_sender_user_id, viewer_user_id, other_user_id);

        format!(
            r#"
        <button type="button"
                class="chat-reply-quote"
                data-target-message-id="{reply_to}">
            <strong>{reply_author}</strong>
            <span>{reply_preview}</span>
        </button>"#,
            reply_to = message.reply_to_message_id,
            reply_author = reply_author,
            reply_preview = reply_preview,
        )
    } else {
        String::new()
    };

    let edited_html = if message.edited_at > 0 && !deleted {
        r#"<span class="chat-edited-label">изменено</span>"#.to_string()
    } else {
        String::new()
    };

    let display_body = chat_message_body_html(message);
    let safe_message_text = escape_html(&message.message);
    let attachment_kind = escape_html(&message.attachment_kind);
    let attachment_url = escape_html(&message.attachment_url);

    let reactions_html = if message.reactions.is_empty() {
        String::new()
    } else {
        let pills = message
            .reactions
            .iter()
            .map(|reaction| {
                let mine_class = if reaction.mine {
                    " is-mine"
                } else {
                    ""
                };

                format!(
                    r#"<button type="button" class="chat-reaction-pill{mine_class}" data-emoji="{emoji}">{emoji} <span>{count}</span></button>"#,
                    mine_class = mine_class,
                    emoji = escape_html(&reaction.emoji),
                    count = reaction.count,
                )
            })
            .collect::<Vec<_>>()
            .join("");

        format!(r#"<div class="chat-message-reactions">{pills}</div>"#)
    };

    format!(
        r#"
{date_separator}

<div class="{row_class}"
     data-message-id="{message_id}"
     data-mine="{mine_attribute}"
     data-deleted="{deleted_flag}"
     data-edited-at="{edited_at}"
     data-reply-to="{reply_to}"
     data-reply-message="{reply_message}"
     data-reply-sender="{reply_sender}"
     data-read-at="{read_at}"
     data-delivered-at="{delivered_at}"
     data-created-at="{created_at}"
     data-message-text="{message_text}"
     data-attachment-kind="{attachment_kind}"
     data-attachment-url="{attachment_url}">

    <div class="{bubble_class}">
        {reply_html}
        {display_body}
        {reactions_html}
        {edited_html}
        <div class="chat-message-meta">
            <span>{chat_time}</span>
            <span class="chat-message-status{status_class}">{status_mark}</span>
        </div>
    </div>

</div>
"#,
        row_class = row_class,
        message_id = message.id,
        mine_attribute = mine_attribute,
        deleted_flag = if deleted { "1" } else { "0" },
        edited_at = message.edited_at,
        reply_to = message.reply_to_message_id,
        reply_message = escape_html(&message.reply_message),
        reply_sender = message.reply_sender_user_id,
        read_at = message.read_at,
        delivered_at = message.delivered_at,
        created_at = message.created_at,
        message_text = safe_message_text,
        bubble_class = bubble_class,
        reply_html = reply_html,
        display_body = display_body,
        reactions_html = reactions_html,
        attachment_kind = attachment_kind,
        attachment_url = attachment_url,
        edited_html = edited_html,
        chat_time = chat_time,
        status_class = status.1,
        status_mark = status.0,
        date_separator = date_separator,
    )
}

pub struct ChatContactRequestState {
    pub id: i64,
    pub sender_user_id: i64,
    pub status: String,
}

#[allow(clippy::too_many_arguments)]
pub fn render_chat(
    authenticated: bool,
    viewer_user_id: i64,
    other_user_id: i64,
    username: &str,
    first_name: &str,
    last_name: &str,
    messages: Vec<crate::web::view_models::ChatMessageRow>,
    contact_request: Option<&ChatContactRequestState>,
) -> String {
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
        format!("Участник · {:06}", other_user_id.rem_euclid(1_000_000))
    } else {
        "Чат".to_string()
    };

    let subtitle = if !safe_username.is_empty() {
        format!("@{}", safe_username)
    } else {
        "Личный диалог".to_string()
    };

    let content = if !authenticated {
        guest_locked_section("Чат", "/app/messages")
    } else if messages.is_empty()
        && username.is_empty()
        && first_name.is_empty()
        && last_name.is_empty()
    {
        empty_state_card("Чат недоступен", "Диалог недоступен.")
    } else {
        let first_message_id = messages.first().map(|message| message.id).unwrap_or(0);

        let last_message_id = messages.last().map(|message| message.id).unwrap_or(0);

        let may_have_older = if messages.len() >= 100 { "1" } else { "0" };

        let message_cards = if messages.is_empty() {
            r#"
<div class="chat-empty-thread">
    <div class="chat-empty-thread-icon" aria-hidden="true"></div>
    <strong>Диалог открыт</strong>
    <p>Напишите первое сообщение — Enter для отправки.</p>
</div>
"#
            .to_string()
        } else {
            let mut last_date_key = String::new();

            messages
                .iter()
                .map(|message| {
                    render_chat_message_row(
                        message,
                        viewer_user_id,
                        other_user_id,
                        &mut last_date_key,
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };

        let contact_gate = contact_request
            .map(|request| {
                if request.status == "pending" && request.sender_user_id != viewer_user_id {
                    format!(
                        r#"<aside class="chat-request-gate">
<strong>Запрос на общение</strong>
<p>Примите запрос, чтобы продолжить переписку.</p>
<div class="chat-request-actions">
<form method="post" action="/app/contact-request/{id}/accept"><button type="submit">Принять</button></form>
<form method="post" action="/app/contact-request/{id}/reject"><button type="submit">Отклонить</button></form>
</div>
</aside>"#,
                        id = request.id
                    )
                } else if request.status == "pending" {
                    r#"<aside class="chat-request-gate"><strong>Сообщение отправлено</strong><p>Ожидаем решения пользователя. Второе сообщение пока недоступно.</p></aside>"#.to_string()
                } else if request.status == "rejected" {
                    r#"<aside class="chat-request-gate is-rejected"><strong>Запрос отклонён</strong><p>Продолжить этот диалог сейчас нельзя.</p></aside>"#.to_string()
                } else {
                    String::new()
                }
            })
            .unwrap_or_default();

        let composer_locked = contact_request
            .is_some_and(|request| request.status == "pending" || request.status == "rejected");

        let composer = if composer_locked {
            String::new()
        } else {
            r#"
<form id="chat-form"
      class="ui-form chat-composer">

    <div id="chat-reply-bar"
         class="chat-reply-bar"
         hidden>
        <div class="chat-reply-accent"></div>
        <div class="chat-reply-copy">
            <strong>Ответ</strong>
            <span id="chat-reply-text"></span>
        </div>
        <button id="chat-reply-close"
                type="button"
                aria-label="Отменить ответ">
            ×
        </button>
    </div>

    <div id="chat-forward-bar"
         class="chat-forward-bar"
         hidden>
        <div class="chat-forward-accent"></div>
        <div class="chat-reply-copy">
            <strong>Переслать</strong>
            <span id="chat-forward-text"></span>
        </div>
        <button id="chat-forward-close"
                type="button"
                aria-label="Отменить пересылку">
            ×
        </button>
    </div>

    <div class="chat-composer-main">
        <textarea id="chat-input" name="message" rows="1" maxlength="2000" required autocomplete="off" enterkeyhint="send" aria-label="Текст сообщения" placeholder="Сообщение…" class="ui-textarea chat-input"></textarea>
        <button id="chat-clear" type="button" class="chat-clear-button" aria-label="Очистить сообщение" hidden>×</button>
    </div>
    <input type="file" id="chat-image-input" accept="image/jpeg,image/png,image/webp" hidden>
    <button id="chat-voice-btn" type="button" class="chat-voice-btn">Голос</button>
    <button id="chat-image-btn" type="button" class="chat-image-btn">Фото</button>
    <button id="chat-send" type="submit" class="ui-button chat-send-button">Отправить</button>
    <div class="chat-composer-footer"><span id="chat-send-state">Enter — отправить · Shift+Enter — новая строка</span><span id="chat-counter">0 / 2000</span></div>
</form>
"#.to_string()
        };

        format!(
            r#"
<link rel="stylesheet"
      href="{chat_css}">

<section class="card chat-shell">

    <div class="chat-history-toolbar">
        <button id="chat-load-older"
                type="button"
                class="chat-secondary-button"
                hidden>
            Загрузить предыдущие сообщения
        </button>

        <div class="chat-toolbar-status">
            <span id="chat-connection-state"
                  class="chat-connection-state">
                Подключение…
            </span>

            <span id="chat-peer-state"
                  class="chat-peer-state"
                  hidden></span>

            <button id="chat-sound-toggle"
                    type="button"
                    class="chat-sound-toggle"
                    aria-label="Звуки чата"
                    aria-pressed="true"
                    title="Звуки чата">
                Звук
            </button>

            <button id="chat-haptic-toggle"
                    type="button"
                    class="chat-sound-toggle chat-haptic-toggle"
                    aria-label="Вибрация чата"
                    aria-pressed="true"
                    title="Вибрация чата">
                Вибро
            </button>

            <button id="chat-block-toggle"
                    type="button"
                    class="chat-block-toggle"
                    hidden>
                Заблокировать
            </button>
        </div>
    </div>

    <div id="chat-messages"
         data-other-user-id="{other_user_id}"
         data-first-message-id="{first_message_id}"
         data-last-message-id="{last_message_id}"
         data-may-have-older="{may_have_older}"
         aria-live="polite"
         class="chat-messages-panel">
        <div id="chat-history-start"></div>
        {message_cards}
        <div id="chat-end"></div>
    </div>

    <button id="chat-scroll-bottom"
            type="button"
            class="chat-scroll-bottom"
            hidden
            aria-label="К новым сообщениям">
        ↓
    </button>

</section>

{contact_gate}

{composer}

<!-- legacy composer removed
<form id="chat-form"
      class="ui-form chat-composer">

    <div id="chat-reply-bar"
         class="chat-reply-bar"
         hidden>
        <div class="chat-reply-accent"></div>
        <div class="chat-reply-copy">
            <strong>Ответ</strong>
            <span id="chat-reply-text"></span>
        </div>
        <button id="chat-reply-close"
                type="button"
                aria-label="Отменить ответ">
            ×
        </button>
    </div>

    <div id="chat-forward-bar"
         class="chat-forward-bar"
         hidden>
        <div class="chat-forward-accent"></div>
        <div class="chat-reply-copy">
            <strong>Переслать</strong>
            <span id="chat-forward-text"></span>
        </div>
        <button id="chat-forward-close"
                type="button"
                aria-label="Отменить пересылку">
            ×
        </button>
    </div>

    <div class="chat-composer-main">
        <textarea
            id="chat-input"
            name="message"
            rows="1"
            maxlength="2000"
            required
            autocomplete="off"
            enterkeyhint="send"
            aria-label="Текст сообщения"
            placeholder="Сообщение…"
            class="ui-textarea chat-input"></textarea>

        <button id="chat-clear"
                type="button"
                class="chat-clear-button"
                aria-label="Очистить сообщение"
                hidden>
            ×
        </button>
    </div>

    <input type="file"
           id="chat-image-input"
           accept="image/jpeg,image/png,image/webp"
           hidden>

    <button id="chat-voice-btn"
            type="button"
            class="chat-voice-btn"
            aria-label="Голосовое сообщение"
            title="Удерживайте для записи">
        Голос
    </button>

    <button id="chat-image-btn"
            type="button"
            class="chat-image-btn"
            aria-label="Отправить фото">
        Фото
    </button>

    <button id="chat-send"
            type="submit"
            class="ui-button chat-send-button"
            aria-label="Отправить сообщение">
        Отправить
    </button>

    <div class="chat-composer-footer">
        <span id="chat-send-state">
            Enter — отправить · Shift+Enter — новая строка
        </span>
        <span id="chat-counter">0 / 2000</span>
    </div>
</form>
-->

<script src="{chat_js}" defer></script>
<script src="{chat_blocks_js}" defer></script>

"#,
            chat_css = static_asset("chat-v2.css"),
            chat_js = static_asset("chat-v2.js"),
            chat_blocks_js = static_asset("chat-blocks.js"),
            other_user_id = other_user_id,
            first_message_id = first_message_id,
            last_message_id = last_message_id,
            may_have_older = may_have_older,
            message_cards = message_cards,
            contact_gate = contact_gate,
            composer = composer,
        )
    };

    let content_html = format!(
        r####"<section class="hero chat-header-premium">

    {back_link}

    <div class="chat-header-main">

        <div class="chat-avatar-ring" aria-hidden="true">
            <div class="card-icon chat-header-avatar">
                {user_icon}
            </div>
            <span id="chat-header-presence-dot"
                  class="chat-header-presence-dot"
                  hidden></span>
        </div>

        <div class="chat-header-copy">

            <h1 class="chat-header-title">
                {display_name}
            </h1>

            <div id="chat-header-status"
                 class="chat-header-status"
                 data-default-subtitle="{subtitle}">
                {subtitle}
            </div>

        </div>

    </div>

</section>

{content}"####,
        back_link = back_link("/app/messages", "Назад", "chevron"),
        user_icon = icon("user"),
        display_name = display_name,
        subtitle = subtitle,
        content = content,
    );

    page_shell(
        "Чат · ResursMap",
        &topbar("Чат", "message-circle"),
        "",
        &content_html,
        &bottom_nav("profile"),
    )
}

// ============================================================
// PROFILE
// ============================================================
