use super::auth::verify_user_session;
use crate::state::app_state::AppState;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
};
use rusqlite::params;

fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn display_name(username: &str, first_name: &str, last_name: &str, fallback: i64) -> String {
    let full = format!("{} {}", first_name.trim(), last_name.trim())
        .trim()
        .to_string();
    if !full.is_empty() {
        full
    } else if !username.trim().is_empty() {
        format!("@{}", username.trim_start_matches('@'))
    } else {
        format!("Участник {}", fallback)
    }
}

fn shell(title: &str, body: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
<meta name="theme-color" content="#0b1220">
<title>{title}</title>
<link rel="stylesheet" href="/static/chat-next.css?v=3">
</head>
<body>
{body}
<script src="/static/chat-next.js?v=3" defer></script>
</body>
</html>"##,
        title = esc(title),
        body = body
    )
}

pub async fn messages_page(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return Html(shell(
            "Сообщения",
            r#"<main class="mn-shell"><section class="mn-empty">
                <div class="mn-empty-icon">🔐</div>
                <h1>Нужен вход</h1>
                <p>Войдите в аккаунт, чтобы открыть сообщения.</p>
                <a class="mn-primary" href="/app/login">Войти</a>
            </section></main>"#,
        ));
    };

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html(shell(
                "Сообщения",
                r#"<main class="mn-shell"><section class="mn-empty"><h1>Сервис временно недоступен</h1><p>Попробуйте ещё раз через несколько секунд.</p></section></main>"#,
            ));
        }
    };

    let mut rows_html = String::new();

    let rows = db
        .prepare(
            r#"SELECT
                c.id,
                CASE WHEN c.user1_id = ?1 THEN c.user2_id ELSE c.user1_id END AS peer_id,
                COALESCE(p.username, ''),
                COALESCE(p.first_name, ''),
                COALESCE(p.last_name, ''),
                COALESCE((
                    SELECT CASE
                        WHEN m.deleted_at > 0 THEN 'Сообщение удалено'
                        WHEN m.attachment_kind = 'image' THEN '📷 Фото'
                        WHEN m.attachment_kind = 'voice' THEN '🎤 Голосовое сообщение'
                        ELSE m.message
                    END
                    FROM messages m
                    WHERE m.conversation_id = c.id
                    ORDER BY m.id DESC
                    LIMIT 1
                ), ''),
                COALESCE((
                    SELECT COUNT(*)
                    FROM messages m
                    WHERE m.conversation_id = c.id
                      AND m.sender_user_id <> ?1
                      AND m.read_at = 0
                ), 0),
                c.updated_at
            FROM conversations c
            LEFT JOIN profiles p
              ON p.user_id = CASE WHEN c.user1_id = ?1 THEN c.user2_id ELSE c.user1_id END
            WHERE c.user1_id = ?1 OR c.user2_id = ?1
            ORDER BY c.updated_at DESC, c.id DESC
            LIMIT 250"#,
        )
        .and_then(|mut stmt| {
            stmt.query_map(params![user_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    for (_conversation_id, peer_id, username, first, last, last_message, unread, updated_at) in rows
    {
        let name = display_name(&username, &first, &last, peer_id);
        let initial = name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .collect::<String>();
        let badge = if unread > 0 {
            format!(r#"<span class="mn-unread">{}</span>"#, unread.min(99))
        } else {
            String::new()
        };

        rows_html.push_str(&format!(
            r#"<a class="mn-thread" href="/app/chat/{peer}">
                <div class="mn-avatar">{initial}</div>
                <div class="mn-thread-main">
                    <div class="mn-thread-top">
                        <strong>{name}</strong>
                        <time data-unix="{updated}"></time>
                    </div>
                    <div class="mn-thread-bottom">
                        <span>{preview}</span>
                        {badge}
                    </div>
                </div>
            </a>"#,
            peer = peer_id,
            initial = esc(&initial),
            name = esc(&name),
            updated = updated_at,
            preview = esc(if last_message.is_empty() {
                "Начните диалог"
            } else {
                &last_message
            }),
            badge = badge,
        ));
    }

    if rows_html.is_empty() {
        rows_html = r#"<section class="mn-empty mn-empty-compact">
            <div class="mn-empty-icon">💬</div>
            <h2>Диалогов пока нет</h2>
            <p>Откройте профиль участника и начните общение.</p>
        </section>"#
            .to_string();
    }

    let body = format!(
        r#"<main class="mn-shell mn-inbox" data-messenger-inbox data-user-id="{user_id}">
            <header class="mn-topbar">
                <a class="mn-icon-btn" href="/app" aria-label="Назад">‹</a>
                <div>
                    <div class="mn-kicker">RESURSMAP</div>
                    <h1>Сообщения</h1>
                </div>
                <button class="mn-icon-btn" type="button" data-refresh-inbox aria-label="Обновить">↻</button>
            </header>
            <div class="mn-search-wrap">
                <span>⌕</span>
                <input data-inbox-search type="search" placeholder="Поиск диалогов" autocomplete="off">
            </div>
            <section class="mn-thread-list">{rows_html}</section>
        </main>"#,
        user_id = user_id,
        rows_html = rows_html,
    );

    Html(shell("Сообщения", &body))
}

pub async fn chat_page(
    State(state): State<AppState>,
    Path(other_user_id): Path<i64>,
    headers: HeaderMap,
) -> Html<String> {
    let Some(user_id) = verify_user_session(&state, &headers) else {
        return Html(shell(
            "Чат",
            r#"<main class="mn-shell"><section class="mn-empty"><div class="mn-empty-icon">🔐</div><h1>Нужен вход</h1><a class="mn-primary" href="/app/login">Войти</a></section></main>"#,
        ));
    };

    if other_user_id <= 0 || other_user_id == user_id {
        return Html(shell(
            "Чат",
            r#"<main class="mn-shell"><section class="mn-empty"><h1>Диалог недоступен</h1><a class="mn-primary" href="/app/messages">К сообщениям</a></section></main>"#,
        ));
    }

    let db = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(db) => db,
        Err(_) => {
            return Html(shell(
                "Чат",
                r#"<main class="mn-shell"><section class="mn-empty"><h1>Сервис временно недоступен</h1></section></main>"#,
            ));
        }
    };

    let peer = db
        .query_row(
            r#"SELECT
                COALESCE(username, ''),
                COALESCE(first_name, ''),
                COALESCE(last_name, ''),
                COALESCE(last_seen_at, 0)
            FROM profiles
            WHERE user_id = ?1
            LIMIT 1"#,
            params![other_user_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )
        .ok();

    let (username, first, last, last_seen_at) =
        peer.unwrap_or_else(|| (String::new(), String::new(), String::new(), 0));

    let name = display_name(&username, &first, &last, other_user_id);
    let initial = name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .collect::<String>();

    let body = format!(
        r#"<main class="mn-shell mn-chat"
              data-messenger-chat
              data-user-id="{user_id}"
              data-peer-id="{peer_id}"
              data-peer-last-seen="{last_seen_at}">
            <header class="mn-chatbar">
                <a class="mn-icon-btn" href="/app/messages" aria-label="Назад">‹</a>
                <div class="mn-avatar mn-avatar-small">{initial}</div>
                <div class="mn-peer">
                    <strong>{name}</strong>
                    <span data-peer-status>подключение…</span>
                </div>
                <button class="mn-icon-btn" type="button" data-chat-menu aria-label="Меню">⋯</button>
            </header>

            <section class="mn-messages" data-message-list aria-live="polite">
                <div class="mn-day-chip">Сегодня</div>
                <div class="mn-loading" data-chat-loading>Загрузка сообщений…</div>
            </section>

            <button class="mn-jump-bottom" data-jump-bottom hidden type="button">↓</button>

            <section class="mn-reply-strip" data-reply-strip hidden>
                <div>
                    <strong data-reply-author>Ответ</strong>
                    <span data-reply-text></span>
                </div>
                <button type="button" data-reply-cancel>×</button>
            </section>

            <form class="mn-composer" data-chat-form>
                <label class="mn-attach" title="Фото">
                    <input data-image-input type="file" accept="image/jpeg,image/png,image/webp" hidden>
                    ＋
                </label>
                <div class="mn-input-shell">
                    <textarea data-chat-input rows="1" maxlength="2000" placeholder="Сообщение" autocomplete="off"></textarea>
                    <button class="mn-emoji-btn" type="button" data-emoji-toggle aria-label="Эмодзи">☺</button>
                </div>
                <button class="mn-send" type="submit" data-send-button aria-label="Отправить">➤</button>
            </form>

            <div class="mn-emoji-panel" data-emoji-panel hidden>
                <button type="button">👍</button><button type="button">❤️</button>
                <button type="button">😂</button><button type="button">🔥</button>
                <button type="button">👏</button><button type="button">🙏</button>
                <button type="button">✅</button><button type="button">🤝</button>
            </div>

            <div class="mn-toast" data-chat-toast hidden></div>
        </main>"#,
        user_id = user_id,
        peer_id = other_user_id,
        last_seen_at = last_seen_at,
        initial = esc(&initial),
        name = esc(&name),
    );

    Html(shell(&format!("Чат — {}", name), &body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_escape_is_safe() {
        assert_eq!(esc(r#"<a "x">&"#), "&lt;a &quot;x&quot;&gt;&amp;");
    }
}
