use crate::db::pool::DbPool;
use crate::resource_screening::listing_type_label;
use crate::state::app_state::AppState;
use rusqlite::{params, Connection};

pub fn promotion_price_minor() -> i64 {
    std::env::var("PROMOTION_PRICE_MINOR")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(499)
}

pub fn promotion_price_label() -> String {
    let minor = promotion_price_minor();
    format!("{:.2} €", minor as f64 / 100.0)
}

struct PromotionPublishRow {
    _request_id: i64,
    requester_user_id: i64,
    status: String,
    payment_status: String,
    bot_check_status: String,
    resource_id: i64,
    title: String,
    description: String,
    address: String,
    category: String,
    listing_type: String,
    telegram_chat_id: i64,
    city_name: String,
}

fn insert_promotion_notification(
    connection: &Connection,
    user_id: i64,
    resource_id: i64,
    kind: &str,
    title: &str,
    message: &str,
) {
    if user_id <= 0 {
        return;
    }

    let _ = connection.execute(
        "INSERT INTO user_notifications (
            user_id,
            resource_id,
            kind,
            title,
            message,
            is_read,
            created_at
         )
         VALUES (?1, ?2, ?3, ?4, ?5, 0, strftime('%s','now'))",
        params![user_id, resource_id, kind, title, message],
    );
}

fn load_publish_row(connection: &Connection, request_id: i64) -> Option<PromotionPublishRow> {
    connection
        .query_row(
            "SELECT
                pr.id,
                pr.requester_user_id,
                pr.status,
                pr.payment_status,
                COALESCE(pr.bot_check_status, 'unknown'),
                r.id,
                r.title,
                r.description,
                r.address,
                r.category,
                COALESCE(r.listing_type, 'general'),
                t.telegram_chat_id,
                t.city_name
             FROM resource_promotion_requests pr
             JOIN resources r ON r.id = pr.resource_id
             JOIN city_publication_targets t ON t.id = pr.target_id
             WHERE pr.id = ?1
             LIMIT 1",
            params![request_id],
            |row| {
                Ok(PromotionPublishRow {
                    _request_id: row.get(0)?,
                    requester_user_id: row.get(1)?,
                    status: row.get(2)?,
                    payment_status: row.get(3)?,
                    bot_check_status: row.get(4)?,
                    resource_id: row.get(5)?,
                    title: row.get(6)?,
                    description: row.get(7)?,
                    address: row.get(8)?,
                    category: row.get(9)?,
                    listing_type: row.get(10)?,
                    telegram_chat_id: row.get(11)?,
                    city_name: row.get(12)?,
                })
            },
        )
        .ok()
}

fn format_group_message(row: &PromotionPublishRow) -> String {
    let kind = listing_type_label(&row.listing_type);
    let address = if row.address.trim().is_empty() {
        String::new()
    } else {
        format!("\n📍 {}", row.address.trim())
    };

    format!(
        "📢 ResursMap · {}\n{} · {}\n\n{}\n\n{}{}\n\n🔗 {}/app/resource/{}",
        row.city_name.trim(),
        kind,
        row.category.trim(),
        row.title.trim(),
        row.description.trim(),
        address,
        crate::stripe_payments::public_base_url(),
        row.resource_id,
    )
}

#[allow(clippy::too_many_arguments)]
fn record_promotion_event(
    connection: &Connection,
    request_id: i64,
    actor_user_id: i64,
    event_kind: &str,
    previous_status: &str,
    new_status: &str,
    details: &str,
    now: i64,
) {
    let _ = connection.execute(
        "INSERT INTO resource_promotion_events (
            promotion_request_id,
            actor_user_id,
            event_kind,
            previous_status,
            new_status,
            details,
            created_at
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            request_id,
            actor_user_id,
            event_kind,
            previous_status,
            new_status,
            details,
            now,
        ],
    );
}

pub async fn try_publish_promotion(
    state: &AppState,
    request_id: i64,
    actor_user_id: i64,
) -> Result<(), String> {
    if request_id <= 0 {
        return Err("invalid_request".into());
    }

    let connection = state
        .db_pool
        .get()
        .map_err(|_| "database_unavailable".to_string())?;

    let row = load_publish_row(&connection, request_id)
        .ok_or_else(|| "promotion_not_found".to_string())?;

    if row.payment_status != "paid" {
        return Err("payment_required".into());
    }

    if row.telegram_chat_id >= 0 {
        return Err("group_not_connected".into());
    }

    let can_auto_publish = row.bot_check_status == "passed";
    let admin_approved = row.status == "approved";

    if row.status == "published" {
        return Ok(());
    }

    if row.status != "pending"
        && row.status != "approved"
        && row.status != "publishing"
        && row.status != "failed"
    {
        return Err("invalid_status".into());
    }

    if !can_auto_publish && !admin_approved {
        return Err("moderation_required".into());
    }

    let now = chrono::Utc::now().timestamp();
    let previous_status = row.status.clone();
    let message_text = format_group_message(&row);

    if connection
        .execute(
            "UPDATE resource_promotion_requests
             SET status = 'publishing',
                 updated_at = ?2,
                 failure_reason = ''
             WHERE id = ?1
               AND status IN ('pending', 'approved', 'failed')",
            params![request_id, now],
        )
        .unwrap_or(0)
        != 1
    {
        return Err("publish_busy".into());
    }

    drop(connection);

    let sent = crate::telegram_notify::publish_to_telegram_group(
        state.bot_token.as_deref(),
        row.telegram_chat_id,
        &message_text,
    )
    .await;

    let connection = state
        .db_pool
        .get()
        .map_err(|_| "database_unavailable".to_string())?;

    match sent {
        Ok(message_id) => {
            connection
                .execute(
                    "UPDATE resource_promotion_requests
                     SET status = 'published',
                         telegram_message_id = ?2,
                         published_at = ?3,
                         updated_at = ?3
                     WHERE id = ?1",
                    params![request_id, message_id, now],
                )
                .map_err(|_| "publish_update_failed".to_string())?;

            record_promotion_event(
                &connection,
                request_id,
                actor_user_id,
                "published",
                &previous_status,
                "published",
                "telegram_group",
                now,
            );

            insert_promotion_notification(
                &connection,
                row.requester_user_id,
                row.resource_id,
                "promotion_published",
                "Объявление опубликовано",
                &format!(
                    "«{}» опубликовано в Telegram-группе {}.",
                    row.title.trim(),
                    row.city_name.trim()
                ),
            );

            Ok(())
        }
        Err(error) => {
            let reason = error.to_string();
            let _ = connection.execute(
                "UPDATE resource_promotion_requests
                 SET status = 'failed',
                     failure_reason = ?2,
                     updated_at = ?3
                 WHERE id = ?1",
                params![request_id, reason, now],
            );
            record_promotion_event(
                &connection,
                request_id,
                actor_user_id,
                "publish_failed",
                "publishing",
                "failed",
                &reason,
                now,
            );

            insert_promotion_notification(
                &connection,
                row.requester_user_id,
                row.resource_id,
                "promotion_publish_failed",
                "Публикация отложена",
                &format!(
                    "«{}» оплачено, но отправка в группу {} не удалась. Администратор поможет завершить публикацию.",
                    row.title.trim(),
                    row.city_name.trim()
                ),
            );

            Err(reason)
        }
    }
}

pub fn mark_promotion_paid(
    pool: &DbPool,
    request_id: i64,
    owner_user_id: i64,
) -> Result<bool, String> {
    mark_promotion_paid_with_reference(pool, request_id, owner_user_id, "")
}

pub fn mark_promotion_paid_with_reference(
    pool: &DbPool,
    request_id: i64,
    owner_user_id: i64,
    payment_reference: &str,
) -> Result<bool, String> {
    let connection = pool.get().map_err(|_| "database_unavailable".to_string())?;

    let now = chrono::Utc::now().timestamp();
    let updated = connection
        .execute(
            "UPDATE resource_promotion_requests
             SET payment_status = 'paid',
                 stripe_payment_reference = CASE
                     WHEN ?4 != '' THEN ?4
                     ELSE stripe_payment_reference
                 END,
                 updated_at = ?3
             WHERE id = ?1
               AND requester_user_id = ?2
               AND payment_status = 'pending'",
            params![request_id, owner_user_id, now, payment_reference],
        )
        .unwrap_or(0);

    if updated != 1 {
        return Ok(false);
    }

    record_promotion_event(
        &connection,
        request_id,
        owner_user_id,
        "payment_confirmed",
        "pending",
        "pending",
        if payment_reference.is_empty() {
            "paid"
        } else {
            payment_reference
        },
        now,
    );

    Ok(true)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionFinalizeOutcome {
    Published,
    AwaitingModeration,
    AlreadyPublished,
}

pub async fn finalize_paid_promotion(
    state: &AppState,
    request_id: i64,
    actor_user_id: i64,
    notify_user: bool,
) -> Result<PromotionFinalizeOutcome, String> {
    let connection = state
        .db_pool
        .get()
        .map_err(|_| "database_unavailable".to_string())?;

    let row: Option<(String, String, i64, i64, String)> = connection
        .query_row(
            "SELECT pr.status,
                    COALESCE(pr.bot_check_status, 'unknown'),
                    pr.requester_user_id,
                    pr.resource_id,
                    r.title
             FROM resource_promotion_requests pr
             JOIN resources r ON r.id = pr.resource_id
             WHERE pr.id = ?1
               AND pr.payment_status = 'paid'
             LIMIT 1",
            params![request_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .ok();

    drop(connection);

    let Some((status, bot_status, requester_user_id, resource_id, title)) = row else {
        return Err("promotion_not_paid".into());
    };

    if status == "published" {
        return Ok(PromotionFinalizeOutcome::AlreadyPublished);
    }

    if bot_status == "passed" {
        try_publish_promotion(state, request_id, actor_user_id).await?;
        Ok(PromotionFinalizeOutcome::Published)
    } else {
        if notify_user {
            let connection = state
                .db_pool
                .get()
                .map_err(|_| "database_unavailable".to_string())?;

            insert_promotion_notification(
                &connection,
                requester_user_id,
                resource_id,
                "promotion_moderation",
                "Продвижение на модерации",
                &format!(
                    "Оплата за «{}» принята. Администратор проверит объявление перед публикацией в группе.",
                    title.trim()
                ),
            );
        }

        Ok(PromotionFinalizeOutcome::AwaitingModeration)
    }
}

pub fn store_checkout_session_id(
    pool: &DbPool,
    request_id: i64,
    owner_user_id: i64,
    session_id: &str,
) -> Result<bool, String> {
    let connection = pool.get().map_err(|_| "database_unavailable".to_string())?;
    let updated = connection
        .execute(
            "UPDATE resource_promotion_requests
             SET stripe_checkout_session_id = ?3,
                 updated_at = ?4
             WHERE id = ?1
               AND requester_user_id = ?2
               AND payment_status = 'pending'",
            params![
                request_id,
                owner_user_id,
                session_id,
                chrono::Utc::now().timestamp()
            ],
        )
        .unwrap_or(0);

    Ok(updated == 1)
}
