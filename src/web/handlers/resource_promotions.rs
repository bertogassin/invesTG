use super::auth::verify_authenticated_user;
use super::common::{rate_limit_retry_after, request_is_cross_site, unix_now};
use super::types::PromotionRequestForm;
use crate::resource_publisher::{
    mark_promotion_paid, promotion_price_label, promotion_price_minor, try_publish_promotion,
};
use crate::resource_screening::{listing_type_label, screen_listing_content};
use crate::state::app_state::AppState;
use crate::web::handlers::admin::is_resource_moderation_session;
use crate::web::templates;
use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};

struct PromotionResource {
    id: i64,
    owner_client_id: String,
    continent_index: i64,
    country_index: i64,
    city_index: i64,
    category: String,
    title: String,
    description: String,
    address: String,
    moderation_status: String,
    is_active: i64,
    listing_type: String,
}

struct PromotionTarget {
    id: i64,
    city_name: String,
    target_name: String,
}

fn resource_is_eligible(moderation_status: &str, is_active: i64) -> bool {
    moderation_status == "approved" && is_active == 1
}

fn target_matches_resource(
    resource: &PromotionResource,
    target_continent: i64,
    target_country: i64,
    target_city: i64,
) -> bool {
    resource.continent_index == target_continent
        && resource.country_index == target_country
        && resource.city_index == target_city
}

fn load_owned_resource(
    connection: &rusqlite::Connection,
    resource_id: i64,
    owner_client_id: &str,
) -> Option<PromotionResource> {
    connection
        .query_row(
            "SELECT
                id,
                client_id,
                continent_index,
                country_index,
                city_index,
                category,
                title,
                description,
                address,
                moderation_status,
                is_active,
                listing_type
             FROM resources
             WHERE id = ?1
               AND client_id = ?2
             LIMIT 1",
            rusqlite::params![resource_id, owner_client_id,],
            |row| {
                Ok(PromotionResource {
                    id: row.get(0)?,
                    owner_client_id: row.get(1)?,
                    continent_index: row.get(2)?,
                    country_index: row.get(3)?,
                    city_index: row.get(4)?,
                    category: row.get(5)?,
                    title: row.get(6)?,
                    description: row.get(7)?,
                    address: row.get(8)?,
                    moderation_status: row.get(9)?,
                    is_active: row.get(10)?,
                    listing_type: row.get(11)?,
                })
            },
        )
        .ok()
}

fn load_target_for_resource(
    connection: &rusqlite::Connection,
    resource: &PromotionResource,
) -> Option<PromotionTarget> {
    connection
        .query_row(
            "SELECT
                id,
                city_name,
                target_name
             FROM city_publication_targets
             WHERE continent_index = ?1
               AND country_index = ?2
               AND city_index = ?3
               AND telegram_chat_id < 0
               AND is_active = 1
             ORDER BY id ASC
             LIMIT 1",
            rusqlite::params![
                resource.continent_index,
                resource.country_index,
                resource.city_index,
            ],
            |row| {
                Ok(PromotionTarget {
                    id: row.get(0)?,
                    city_name: row.get(1)?,
                    target_name: row.get(2)?,
                })
            },
        )
        .ok()
}

fn status_response(
    title: &str,
    eyebrow: &str,
    heading: &str,
    description: &str,
    resource_id: i64,
) -> Response {
    Html(templates::status_page(
        title,
        eyebrow,
        heading,
        description,
        &templates::navigation_card(
            &format!("/app/resource/{resource_id}"),
            "map-pin",
            "Вернуться к объявлению",
            "Открыть ресурс",
        ),
    ))
    .into_response()
}

pub async fn resource_promotion_page(
    State(state): State<AppState>,
    Path(resource_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if resource_id <= 0 {
        return status_response(
            "Продвижение · ResursMap",
            "⚠ ResursMap",
            "Объявление не найдено",
            "Проверьте ссылку и повторите попытку.",
            resource_id,
        );
    }

    let authenticated = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,

        None => {
            return Html(templates::status_page(
                "Требуется вход · ResursMap",
                "Авторизация",
                "Войдите в аккаунт",
                "Продвижение доступно владельцу объявления.",
                &templates::navigation_card(
                    "/login?next=/app/my-resources",
                    "user",
                    "Войти",
                    "Личный кабинет",
                ),
            ))
            .into_response();
        }
    };

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,

        Err(_) => {
            return status_response(
                "Продвижение · ResursMap",
                "⚠ ResursMap",
                "Сервис недоступен",
                "Повторите попытку позже.",
                resource_id,
            );
        }
    };

    let Some(resource) = load_owned_resource(&connection, resource_id, &authenticated.client_id)
    else {
        return status_response(
            "Продвижение · ResursMap",
            "⚠ Доступ",
            "Нет доступа",
            "Продвигать объявление может только его владелец.",
            resource_id,
        );
    };

    if !resource_is_eligible(&resource.moderation_status, resource.is_active) {
        return status_response(
            "Продвижение · ResursMap",
            "Модерация",
            "Продвижение недоступно",
            "Сначала объявление должно быть одобрено и опубликовано.",
            resource_id,
        );
    }

    let Some(target) = load_target_for_resource(&connection, &resource) else {
        return status_response(
            "Продвижение · ResursMap",
            "Городская группа",
            "Группа ещё не подключена",
            "Продвижение станет доступно после подключения официальной группы города.",
            resource_id,
        );
    };

    let existing_row: Option<(String, String, i64, String, String)> = connection
        .query_row(
            "SELECT status, payment_status, id,
                    COALESCE(bot_check_status, 'unknown'),
                    COALESCE(failure_reason, '')
             FROM resource_promotion_requests
             WHERE resource_id = ?1
               AND target_id = ?2
               AND status IN (
                   'pending',
                   'approved',
                   'publishing',
                   'failed',
                   'published'
               )
             ORDER BY id DESC
             LIMIT 1",
            rusqlite::params![resource.id, target.id,],
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

    let (
        existing_status,
        existing_payment_status,
        existing_request_id,
        existing_bot_status,
        existing_failure_reason,
    ) = match existing_row {
        Some((status, payment, id, bot_status, failure_reason)) => (
            Some(status),
            Some(payment),
            Some(id),
            Some(bot_status),
            Some(failure_reason),
        ),
        None => (None, None, None, None, None),
    };

    drop(connection);

    Html(templates::render_resource_promotion(
        templates::RenderResourcePromotionParams {
            resource_id: resource.id,
            title: &resource.title,
            category: &resource.category,
            description: &resource.description,
            address: &resource.address,
            city_name: &target.city_name,
            target_name: &target.target_name,
            target_id: target.id,
            listing_type_label: listing_type_label(&resource.listing_type),
            price_label: &promotion_price_label(),
            existing_status: existing_status.as_deref(),
            existing_payment_status: existing_payment_status.as_deref(),
            existing_request_id,
            existing_bot_status: existing_bot_status.as_deref(),
            existing_failure_reason: existing_failure_reason.as_deref(),
        },
    ))
    .into_response()
}

pub async fn request_resource_promotion(
    State(state): State<AppState>,
    Path(resource_id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<PromotionRequestForm>,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён").into_response();
    }

    if resource_id <= 0 || form.target_id <= 0 {
        return (StatusCode::BAD_REQUEST, "Некорректный запрос").into_response();
    }

    let authenticated = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,

        None => {
            return (StatusCode::UNAUTHORIZED, "Требуется вход").into_response();
        }
    };

    if let Some(retry_after) = rate_limit_retry_after(
        &state,
        authenticated.user_id,
        "resource_promotion_request",
        5,
        3600,
    )
    .await
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "Слишком много заявок. Повторите попытку позже.",
        )
            .into_response();
    }

    let mut connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,

        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Сервис недоступен").into_response();
        }
    };

    let Some(resource) = load_owned_resource(&connection, resource_id, &authenticated.client_id)
    else {
        return (StatusCode::FORBIDDEN, "Нет доступа к объявлению").into_response();
    };

    if resource.owner_client_id != authenticated.client_id
        || !resource_is_eligible(&resource.moderation_status, resource.is_active)
    {
        return (StatusCode::FORBIDDEN, "Продвижение недоступно").into_response();
    }

    let target_location: Option<(i64, i64, i64)> = connection
        .query_row(
            "SELECT
                continent_index,
                country_index,
                city_index
             FROM city_publication_targets
             WHERE id = ?1
               AND telegram_chat_id < 0
               AND is_active = 1
             LIMIT 1",
            rusqlite::params![form.target_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();

    let Some((target_continent, target_country, target_city)) = target_location else {
        return (StatusCode::BAD_REQUEST, "Городская группа недоступна").into_response();
    };

    if !target_matches_resource(&resource, target_continent, target_country, target_city) {
        return (
            StatusCode::BAD_REQUEST,
            "Группа не соответствует городу объявления",
        )
            .into_response();
    }

    let screening = screen_listing_content(&resource.title, &resource.description, "");
    let bot_check_status = if screening.passed { "passed" } else { "failed" };
    let price_minor = promotion_price_minor();

    let transaction = match connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
    {
        Ok(transaction) => transaction,

        Err(_) => {
            return (StatusCode::CONFLICT, "Очередь занята. Повторите попытку.").into_response();
        }
    };

    let inserted = transaction.execute(
        "INSERT INTO resource_promotion_requests (
            resource_id,
            requester_user_id,
            target_id,
            client_request_id,
            status,
            payment_status,
            price_minor,
            currency,
            bot_check_status,
            bot_check_reason,
            created_at,
            updated_at
         )
         VALUES (
            ?1, ?2, ?3,
            lower(hex(randomblob(16))),
            'pending',
            'pending',
            ?4,
            'EUR',
            ?5,
            ?6,
            ?7,
            ?7
         )",
        rusqlite::params![
            resource.id,
            authenticated.user_id,
            form.target_id,
            price_minor,
            bot_check_status,
            screening.reason,
            unix_now(),
        ],
    );

    if inserted.is_err() {
        return status_response(
            "Продвижение · ResursMap",
            "Заявка",
            "Заявка уже создана",
            "Объявление уже ожидает подтверждения для этой городской группы.",
            resource.id,
        );
    }

    let request_id = transaction.last_insert_rowid();

    if transaction
        .execute(
            "INSERT INTO resource_promotion_events (
                promotion_request_id,
                actor_user_id,
                event_kind,
                previous_status,
                new_status,
                details,
                created_at
             )
             VALUES (
                ?1, ?2, 'created',
                '', 'pending',
                ?3,
                ?4
             )",
            rusqlite::params![
                request_id,
                authenticated.user_id,
                unix_now(),
                bot_check_status,
            ],
        )
        .unwrap_or(0)
        != 1
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось сохранить журнал заявки",
        )
            .into_response();
    }

    if transaction.commit().is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Не удалось завершить заявку",
        )
            .into_response();
    }

    Redirect::temporary(&format!(
        "/app/resource/{}/promote/pay/{}",
        resource.id, request_id
    ))
    .into_response()
}

pub async fn promotion_payment_page(
    State(state): State<AppState>,
    Path((resource_id, request_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Response {
    let authenticated = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => {
            return Redirect::temporary(&format!(
                "/login?next={}",
                urlencoding::encode(&format!(
                    "/app/resource/{resource_id}/promote/pay/{request_id}"
                ))
            ))
            .into_response();
        }
    };

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => {
            return status_response(
                "Оплата · ResursMap",
                "⚠ ResursMap",
                "Сервис недоступен",
                "Повторите попытку позже.",
                resource_id,
            );
        }
    };

    let row: Option<(String, String, String, i64)> = connection
        .query_row(
            "SELECT pr.payment_status,
                    pr.bot_check_status,
                    COALESCE(pr.bot_check_reason, ''),
                    pr.price_minor
             FROM resource_promotion_requests pr
             JOIN resources r ON r.id = pr.resource_id
             WHERE pr.id = ?1
               AND pr.resource_id = ?2
               AND pr.requester_user_id = ?3
             LIMIT 1",
            rusqlite::params![request_id, resource_id, authenticated.user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .ok();

    drop(connection);

    let Some((payment_status, bot_status, bot_reason, price_minor)) = row else {
        return status_response(
            "Оплата · ResursMap",
            "⚠ Доступ",
            "Заявка не найдена",
            "Проверьте ссылку или создайте новую заявку.",
            resource_id,
        );
    };

    if payment_status == "paid" {
        return status_response(
            "Оплата · ResursMap",
            "✓ ResursMap",
            "Оплата уже подтверждена",
            if bot_status == "passed" {
                "Объявление будет опубликовано в группе автоматически после проверки системы."
            } else {
                "Заявка передана администратору для модерации перед публикацией в группе."
            },
            resource_id,
        );
    }

    let price_label = format!("{:.2} €", price_minor as f64 / 100.0);
    let bot_note = if bot_status == "passed" {
        "Автопроверка пройдена: после оплаты публикация в группе выполняется сразу."
    } else {
        "Автопроверка не пройдена: после оплаты заявка уйдёт администратору, затем — в группу."
    };

    Html(templates::render_promotion_payment(
        resource_id,
        request_id,
        &price_label,
        bot_note,
        if bot_reason.trim().is_empty() {
            None
        } else {
            Some(bot_reason.as_str())
        },
    ))
    .into_response()
}

pub async fn confirm_promotion_payment(
    State(state): State<AppState>,
    Path((resource_id, request_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён").into_response();
    }

    let authenticated = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, "Требуется вход").into_response(),
    };

    let paid = match mark_promotion_paid(&state.db_pool, request_id, authenticated.user_id) {
        Ok(paid) => paid,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Не удалось подтвердить оплату",
            )
                .into_response();
        }
    };

    if !paid {
        return status_response(
            "Оплата · ResursMap",
            "⚠ ResursMap",
            "Оплата уже подтверждена",
            "Повторная оплата не требуется.",
            resource_id,
        );
    }

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => {
            return (StatusCode::SERVICE_UNAVAILABLE, "Сервис недоступен").into_response();
        }
    };

    let bot_status: String = connection
        .query_row(
            "SELECT COALESCE(bot_check_status, 'unknown')
             FROM resource_promotion_requests
             WHERE id = ?1
             LIMIT 1",
            rusqlite::params![request_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "unknown".to_string());
    drop(connection);

    if bot_status == "passed" {
        if let Err(error) = try_publish_promotion(&state, request_id, authenticated.user_id).await {
            return status_response(
                "Публикация · ResursMap",
                "⚠ ResursMap",
                "Оплата принята, публикация отложена",
                &format!(
                    "Оплата прошла успешно, но отправка в группу временно недоступна ({error}). Администратор поможет завершить публикацию."
                ),
                resource_id,
            );
        }

        return status_response(
            "Опубликовано · ResursMap",
            "✓ ResursMap",
            "Объявление опубликовано",
            "Платёж принят, объявление отправлено в городскую Telegram-группу.",
            resource_id,
        );
    }

    status_response(
        "Модерация · ResursMap",
        "✓ ResursMap",
        "Оплата принята",
        "Заявка передана администратору. После проверки объявление будет опубликовано в группе.",
        resource_id,
    )
}

#[allow(clippy::type_complexity)]
pub async fn admin_promotion_queue(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if !is_resource_moderation_session(&state, &headers) {
        return Redirect::temporary("/login?next=%2Fapp%2Fadmin%2Fpromotions").into_response();
    }

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => {
            return Html("<h1>503</h1><p>База данных недоступна.</p>".to_string()).into_response();
        }
    };

    let rows: Vec<(i64, i64, String, String, String, String, String, i64)> = connection
        .prepare(
            "SELECT
                pr.id,
                pr.resource_id,
                r.title,
                r.category,
                COALESCE(r.listing_type, 'general'),
                pr.bot_check_status,
                COALESCE(pr.bot_check_reason, ''),
                pr.created_at
             FROM resource_promotion_requests pr
             JOIN resources r ON r.id = pr.resource_id
             WHERE pr.payment_status = 'paid'
               AND pr.status IN ('pending', 'failed')
             ORDER BY pr.created_at ASC, pr.id ASC
             LIMIT 100",
        )
        .and_then(|mut stmt| {
            stmt.query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();

    drop(connection);

    Html(templates::render_admin_promotion_queue(&rows)).into_response()
}

pub async fn admin_approve_promotion(
    State(state): State<AppState>,
    Path(request_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён").into_response();
    }
    if !is_resource_moderation_session(&state, &headers) {
        return (StatusCode::FORBIDDEN, "Недостаточно прав").into_response();
    }

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, "DB error").into_response(),
    };

    let now = unix_now();
    let updated = connection.execute(
        "UPDATE resource_promotion_requests
         SET status = 'approved',
             updated_at = ?2
         WHERE id = ?1
           AND payment_status = 'paid'
           AND status IN ('pending', 'failed')",
        rusqlite::params![request_id, now],
    );
    drop(connection);

    if updated.unwrap_or(0) != 1 {
        return Redirect::temporary("/app/admin/promotions").into_response();
    }

    let _ = try_publish_promotion(&state, request_id, 0).await;

    Redirect::temporary("/app/admin/promotions").into_response()
}

pub async fn admin_reject_promotion(
    State(state): State<AppState>,
    Path(request_id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён").into_response();
    }
    if !is_resource_moderation_session(&state, &headers) {
        return (StatusCode::FORBIDDEN, "Недостаточно прав").into_response();
    }

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, "DB error").into_response(),
    };

    let _ = connection.execute(
        "UPDATE resource_promotion_requests
         SET status = 'rejected',
             updated_at = strftime('%s','now')
         WHERE id = ?1
           AND status IN ('pending', 'failed')",
        rusqlite::params![request_id],
    );

    Redirect::temporary("/app/admin/promotions").into_response()
}

pub async fn retry_promotion_publish(
    State(state): State<AppState>,
    Path((resource_id, request_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Response {
    if request_is_cross_site(&headers) {
        return (StatusCode::FORBIDDEN, "Запрос отклонён").into_response();
    }

    let authenticated = match verify_authenticated_user(&state, &headers) {
        Some(user) => user,
        None => return (StatusCode::UNAUTHORIZED, "Требуется вход").into_response(),
    };

    let connection = match crate::db::pool::get_connection(&state.db_pool) {
        Ok(connection) => connection,
        Err(_) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Сервис недоступен",
            )
                .into_response();
        }
    };

    let row: Option<(String, String, String)> = connection
        .query_row(
            "SELECT pr.status, pr.payment_status, COALESCE(pr.bot_check_status, 'unknown')
             FROM resource_promotion_requests pr
             JOIN resources r ON r.id = pr.resource_id
             WHERE pr.id = ?1
               AND pr.resource_id = ?2
               AND pr.requester_user_id = ?3
               AND r.client_id = ?4
             LIMIT 1",
            rusqlite::params![
                request_id,
                resource_id,
                authenticated.user_id,
                authenticated.client_id,
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok();
    drop(connection);

    let Some((status, payment_status, bot_status)) = row else {
        return status_response(
            "Продвижение · ResursMap",
            "⚠ Доступ",
            "Заявка не найдена",
            "Проверьте ссылку или создайте новую заявку.",
            resource_id,
        );
    };

    if payment_status != "paid" || status != "failed" || bot_status != "passed" {
        return status_response(
            "Продвижение · ResursMap",
            "⚠ ResursMap",
            "Повтор недоступен",
            "Повторная отправка доступна только для оплаченных заявок с пройденной автопроверкой.",
            resource_id,
        );
    }

    match try_publish_promotion(&state, request_id, authenticated.user_id).await {
        Ok(()) => status_response(
            "Опубликовано · ResursMap",
            "✓ ResursMap",
            "Объявление опубликовано",
            "Публикация в Telegram-группе выполнена успешно.",
            resource_id,
        ),
        Err(error) => status_response(
            "Публикация · ResursMap",
            "⚠ ResursMap",
            "Публикация не удалась",
            &format!(
                "Повторная отправка не удалась ({error}). Попробуйте позже или обратитесь к администратору."
            ),
            resource_id,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource(moderation_status: &str, is_active: i64) -> PromotionResource {
        PromotionResource {
            id: 1,
            owner_client_id: "owner".to_string(),
            continent_index: 0,
            country_index: 2,
            city_index: 4,
            category: "Услуги".to_string(),
            title: "Тест".to_string(),
            description: "Описание".to_string(),
            address: "Ницца".to_string(),
            moderation_status: moderation_status.to_string(),
            is_active,
            listing_type: "offer".to_string(),
        }
    }

    #[test]
    fn only_approved_active_resource_is_eligible() {
        assert!(resource_is_eligible("approved", 1));
        assert!(!resource_is_eligible("pending", 1));
        assert!(!resource_is_eligible("rejected", 1));
        assert!(!resource_is_eligible("approved", 0));
    }

    #[test]
    fn target_must_match_resource_city() {
        let resource = resource("approved", 1);

        assert!(target_matches_resource(&resource, 0, 2, 4,));

        assert!(!target_matches_resource(&resource, 0, 2, 3,));

        assert!(!target_matches_resource(&resource, 0, 1, 4,));
    }
}
