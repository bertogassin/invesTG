use super::auth::verify_authenticated_user;
use super::common::{rate_limit_retry_after, request_is_cross_site, unix_now};
use super::types::PromotionRequestForm;
use crate::state::app_state::AppState;
use crate::web::templates;
use axum::{
    extract::{Form, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
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
                is_active
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
                    "/app/auth?next=/app/my-resources",
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

    let existing_status: Option<String> = connection
        .query_row(
            "SELECT status
             FROM resource_promotion_requests
             WHERE resource_id = ?1
               AND target_id = ?2
               AND status IN (
                   'pending',
                   'approved',
                   'publishing'
               )
             ORDER BY id DESC
             LIMIT 1",
            rusqlite::params![resource.id, target.id,],
            |row| row.get(0),
        )
        .ok();

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
            existing_status: existing_status.as_deref(),
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

    let now = unix_now();

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
            created_at,
            updated_at
         )
         VALUES (
            ?1, ?2, ?3,
            lower(hex(randomblob(16))),
            'pending',
            'not_required',
            0,
            'EUR',
            ?4,
            ?4
         )",
        rusqlite::params![resource.id, authenticated.user_id, form.target_id, now,],
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
                'web_request',
                ?3
             )",
            rusqlite::params![request_id, authenticated.user_id, now,],
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

    status_response(
        "Заявка создана · ResursMap",
        "✓ ResursMap",
        "Заявка отправлена",
        "Публикация появится в городской группе после проверки администратором.",
        resource.id,
    )
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
