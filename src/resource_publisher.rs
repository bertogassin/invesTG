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

fn load_publish_row(connection: &Connection, request_id: i64) -> Option<PromotionPublishRow> {
    connection
        .query_row(
            "SELECT
                pr.id,
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
                    status: row.get(1)?,
                    payment_status: row.get(2)?,
                    bot_check_status: row.get(3)?,
                    resource_id: row.get(4)?,
                    title: row.get(5)?,
                    description: row.get(6)?,
                    address: row.get(7)?,
                    category: row.get(8)?,
                    listing_type: row.get(9)?,
                    telegram_chat_id: row.get(10)?,
                    city_name: row.get(11)?,
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
        "📢 ResursMap · {}\n{} · {}\n\n{}\n\n{}{}\n\n🔗 https://resursmap.de/app/resource/{}",
        row.city_name.trim(),
        kind,
        row.category.trim(),
        row.title.trim(),
        row.description.trim(),
        address,
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

    if row.status != "pending" && row.status != "approved" && row.status != "publishing" {
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
               AND status IN ('pending', 'approved')",
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
            Err(reason)
        }
    }
}

pub fn mark_promotion_paid(
    pool: &DbPool,
    request_id: i64,
    owner_user_id: i64,
) -> Result<bool, String> {
    let connection = pool.get().map_err(|_| "database_unavailable".to_string())?;

    let now = chrono::Utc::now().timestamp();
    let updated = connection
        .execute(
            "UPDATE resource_promotion_requests
             SET payment_status = 'paid',
                 updated_at = ?3
             WHERE id = ?1
               AND requester_user_id = ?2
               AND payment_status = 'pending'",
            params![request_id, owner_user_id, now],
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
        "paid",
        now,
    );

    Ok(true)
}
