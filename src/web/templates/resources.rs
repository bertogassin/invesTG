use super::common::{
    back_hero, back_link, bottom_nav, empty_state_action, empty_state_card,
    empty_state_card_with_actions, escape_html, guest_locked_section, icon,
    my_resource_moderation_badge, navigation_card, page_document, page_shell, premium_badge_html,
    resource_card_link_class, resource_detail_section_class, section_head, topbar,
    verified_badge_html,
};

pub fn render_category(
    ci: usize,
    si: usize,
    zi: usize,
    category: &str,
    resources: Vec<crate::web::view_models::CategoryResourceRow>,
) -> String {
    let city_url = format!("/app/{}/{}/{}", ci, si, zi);
    let category_url = urlencoding::encode(category);

    let cards = if resources.is_empty() {
        format!(
            r#"{empty}
            <a class="feature rm-feature-add"
               href="/app/{ci}/{si}/{zi}/cat/{category_url}/add">
                <div class="card-icon">+</div>
                <strong>Добавить ресурс</strong>
                <span>Будьте первым в этой категории.</span>
            </a>"#,
            ci = ci,
            si = si,
            zi = zi,
            category_url = category_url,
            empty = empty_state_card(
                "Пока ресурсов нет",
                "Будьте первым — добавьте ресурс в эту категорию.",
            ),
        )
    } else {
        resources
            .iter()
            .map(
                |(id, title, description, contact, address, rating, votes, verified, premium)| {
                    let safe_title = escape_html(title);
                    let safe_description = escape_html(description);
                    let safe_contact = escape_html(contact);
                    let safe_address = escape_html(address);

                    let verified_badge = if *verified != 0 {
                        verified_badge_html(false)
                    } else {
                        ""
                    };

                    let premium_badge = if *premium != 0 {
                        premium_badge_html("default")
                    } else {
                        ""
                    };

                    let card_class = resource_card_link_class(*premium != 0);
                    let premium_shine = if *premium != 0 {
                        r#"<div class="rm-resource-card-shine"></div>"#
                    } else {
                        ""
                    };

                    format!(
                        r#"
                    <a href="/app/resource/{id}" class="{card_class}">
                        {premium_shine}
                        <div class="card-icon">{map_icon}</div>

                        <div class="card-content">

                            <div class="rm-resource-title-row">
                                <div class="card-title">
                                    {title}
                                </div>
                                {premium_badge}
                            </div>

                            <div class="card-meta">
                                {description}
                            </div>

                            <div class="card-meta">
                                ⭐ {rating:.1} · {votes} голосов
                            </div>

                            <div class="card-meta">
                                📍 {address}
                            </div>

                            <div class="card-meta">
                                📞 {contact}
                            </div>

                            <div class="rm-resource-verified-row">
                                {verified_badge}
                            </div>

                        </div>

                        <div class="card-arrow">›</div>
                    </a>
                    "#,
                        id = id,
                        card_class = card_class,
                        premium_shine = premium_shine,
                        map_icon = icon("map-pin"),
                        title = safe_title,
                        premium_badge = premium_badge,
                        description = safe_description,
                        rating = rating,
                        votes = votes,
                        address = safe_address,
                        contact = safe_contact,
                        verified_badge = verified_badge,
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let count = resources.len();

    let section_head_resources = section_head("Ресурсы", &format!("Найдено: {}", count), None);
    let section_head_city = section_head("Ваш город", "Все направления и категории", Some(34));
    let city_navigation_card = navigation_card(
        &city_url,
        "map",
        "Открыть карту города",
        "Вернуться ко всем категориям",
    );

    let content = format!(
        r####"{section_head_resources}

<div>
    {cards}
</div>

{section_head_city}

{city_navigation_card}"####,
        section_head_resources = section_head_resources,
        section_head_city = section_head_city,
        city_navigation_card = city_navigation_card,
        cards = cards,
    );

    page_shell(
        &format!("{} · ResursMap", category),
        &topbar("Категория", "globe"),
        &back_hero(
            &back_link(&city_url, "Вернуться к городу", "chevron"),
            "map",
            "Категория",
            category,
            "Ресурсы города в выбранной категории.",
        ),
        &content,
        &bottom_nav("map"),
    )
}

// ============================================================
// ADD RESOURCE
// ============================================================

// ============================================================
// PUBLIC USER PROFILE
// ============================================================

pub struct RenderResourceProfileParams<'a> {
    pub id: i64,
    pub title: &'a str,
    pub description: &'a str,
    pub contact: &'a str,
    pub address: &'a str,
    pub rating: f64,
    pub votes: i64,
    pub premium: i64,
    pub verified: i64,
    pub category: &'a str,
    pub _created_at: i64,
    pub owner_public_id: &'a str,
}

pub fn render_resource_profile(params: RenderResourceProfileParams<'_>) -> String {
    let RenderResourceProfileParams {
        id,
        title,
        description,
        contact,
        address,
        rating,
        votes,
        premium,
        verified,
        category,
        _created_at,
        owner_public_id,
    } = params;
    let safe_description = escape_html(description);
    let safe_contact = escape_html(contact);
    let safe_address = escape_html(address);

    let premium_badge = if premium != 0 {
        premium_badge_html("default")
    } else {
        ""
    };

    let verified_badge = if verified != 0 {
        verified_badge_html(false)
    } else {
        ""
    };

    let hero_description = format!(
        r#"<span class="rm-resource-hero-badges">
            {premium_badge}
            {verified_badge}
        </span>

        <span id="rating-summary" class="rm-resource-rating-summary">
            ⭐ <strong>{rating:.1}</strong> · {votes} голосов
        </span>"#,
        premium_badge = premium_badge,
        verified_badge = verified_badge,
        rating = rating,
        votes = votes,
    );

    let detail_section_class = resource_detail_section_class(premium != 0);

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
<section class="card rm-resource-owner-card">

    <div class="card-icon rm-resource-owner-icon">
        {owner_icon}
    </div>

    <div class="card-content rm-resource-owner-content">

        <div class="rm-resource-owner-kicker">
            Владелец ресурса
        </div>

        <div class="card-title">
            Профиль участника
        </div>

        <div class="card-meta rm-resource-owner-meta">
            Другие ресурсы и актуальный статус
        </div>

    </div>

    <a href="/app/user/{public_id}" class="rm-resource-owner-link">
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

    let main_html = format!(
        r####"<section>
<button
        id="favorite-button"
        type="button"
        class="ui-button rm-resource-favorite-btn">
        ♡ В избранное
    </button>

    <div id="favorite-status" class="ui-status rm-resource-favorite-status">
    </div>

    <button
        id="report-button"
        type="button"
        class="ui-button rm-resource-report-btn">
        ⚑ Пожаловаться
    </button>

    <div id="report-panel" class="rm-resource-report-panel">

        <div class="rm-resource-report-label">
            Причина жалобы
        </div>

        <textarea
            id="report-reason"
            maxlength="500"
            rows="4"
            placeholder="Коротко опишите проблему..."
            class="ui-textarea"></textarea>

        <div class="rm-resource-report-actions">

            <button
                id="report-submit"
                type="button"
                class="ui-button rm-resource-report-submit">
                Отправить жалобу
            </button>

            <button
                id="report-cancel"
                type="button"
                class="ui-button rm-resource-report-cancel">
                Отмена
            </button>

        </div>

        <div id="report-status" class="ui-status rm-resource-report-status">
        </div>

    </div>

    <div class="rm-resource-rating-block">
        <div class="rm-resource-rating-kicker">
            ОЦЕНИТЬ РЕСУРС
        </div>

        <div id="rating-stars" class="rm-resource-stars">
            <button type="button" data-score="1" class="ui-button rm-resource-star-btn">☆</button>
            <button type="button" data-score="2" class="ui-button rm-resource-star-btn">☆</button>
            <button type="button" data-score="3" class="ui-button rm-resource-star-btn">☆</button>
            <button type="button" data-score="4" class="ui-button rm-resource-star-btn">☆</button>
            <button type="button" data-score="5" class="ui-button rm-resource-star-btn">☆</button>
        </div>

        <div id="vote-status" class="ui-status rm-resource-vote-status"></div>
    </div>
</section>

<section class="{detail_section_class}">

    <div class="rm-resource-section-kicker">
        О ресурсе
    </div>

    <div class="rm-resource-description">
        {description}
    </div>

</section>

{owner_profile_html}

<section class="card rm-resource-section">

    <div class="rm-resource-section-kicker rm-resource-section-kicker--contacts">
        Контакты
    </div>

    <div class="card-meta rm-resource-contact-line">
        📍 {address}
    </div>

    <div class="card-meta rm-resource-contact-line rm-resource-contact-line--last">
        📞 {contact}
    </div>

    <div class="rm-resource-contact-actions">

        <a href="{contact_href}" class="rm-resource-contact-btn rm-resource-contact-btn--gold">
            📞 Связаться
        </a>

        <a href="{map_href}"
           target="_blank"
           rel="noopener noreferrer"
           class="rm-resource-contact-btn rm-resource-contact-btn--neutral">
            📍 На карте
        </a>

    </div>

</section>

<section class="card rm-resource-section">

    <div class="rm-resource-section-kicker rm-resource-section-kicker--contacts">
        ID ресурса
    </div>

    <div class="rm-resource-id-value">
        #{id}
    </div>

</section>"####,
        description = safe_description,
        owner_profile_html = owner_profile_html,
        address = safe_address,
        contact = safe_contact,
        contact_href = safe_contact_href,
        map_href = safe_map_href,
        detail_section_class = detail_section_class,
        id = id,
    );

    let body_after = format!(
        r####"<script>
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
                            "Войдите в аккаунт ResursMap.";
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
                            "Войдите в аккаунт ResursMap.";
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
                        "Войдите в аккаунт ResursMap, чтобы поставить оценку.";
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
</script>"####,
        id = id,
    );

    page_document(
        &format!("{} · ResursMap", title),
        "",
        "",
        &format!(
            "{topbar}\n\n{hero}\n\n{content}",
            topbar = topbar("Ресурс", "map"),
            hero = back_hero(
                &back_link("/app", "Вернуться к карте", "arrow-left"),
                "map-pin",
                category,
                title,
                &hero_description,
            ),
            content = main_html,
        ),
        &bottom_nav("map"),
        &body_after,
    )
}

pub struct RenderResourcePromotionParams<'a> {
    pub resource_id: i64,
    pub title: &'a str,
    pub category: &'a str,
    pub description: &'a str,
    pub address: &'a str,
    pub city_name: &'a str,
    pub target_name: &'a str,
    pub target_id: i64,
    pub listing_type_label: &'a str,
    pub price_label: &'a str,
    pub existing_status: Option<&'a str>,
    pub existing_payment_status: Option<&'a str>,
    pub existing_request_id: Option<i64>,
    pub existing_bot_status: Option<&'a str>,
    pub existing_failure_reason: Option<&'a str>,
}

pub fn render_resource_promotion(params: RenderResourcePromotionParams<'_>) -> String {
    let title = escape_html(params.title);
    let category = escape_html(params.category);
    let description = escape_html(params.description);
    let address = escape_html(params.address);
    let city_name = escape_html(params.city_name);
    let target_name = escape_html(params.target_name);

    let listing_type = escape_html(params.listing_type_label);
    let price_label = escape_html(params.price_label);

    let action_html = if params.existing_status == Some("published") {
        r#"
<div class="card rm-promo-pending rm-promo-published">
    <div class="card-title">Опубликовано в группе</div>
    <div class="card-meta rm-promo-pending-copy">
        Объявление уже отправлено в Telegram-группу города.
    </div>
</div>
"#
        .to_string()
    } else if params.existing_status == Some("failed")
        && params.existing_payment_status == Some("paid")
        && params.existing_bot_status == Some("passed")
    {
        if let Some(request_id) = params.existing_request_id {
            let reason = params
                .existing_failure_reason
                .map(escape_html)
                .unwrap_or_default();
            let reason_html = if reason.is_empty() {
                String::new()
            } else {
                format!(
                    r#"<div class="card-meta rm-promo-bot-reason">Причина: {reason}</div>"#
                )
            };
            format!(
                r#"
<div class="card rm-promo-pending rm-promo-failed">
    <div class="card-title">Публикация не удалась</div>
    {reason_html}
    <div class="card-meta rm-promo-pending-copy">
        Оплата принята, автопроверка пройдена. Можно повторить отправку в группу.
    </div>
</div>
<form method="post"
      action="/app/resource/{}/promote/retry/{}"
      class="ui-form rm-promo-form">
    <button type="submit" class="ui-button rm-promo-submit">
        Повторить публикацию
    </button>
</form>
"#,
                params.resource_id, request_id
            )
        } else {
            String::new()
        }
    } else if params.existing_payment_status == Some("pending") {
        if let Some(request_id) = params.existing_request_id {
            format!(
                r#"
<a class="ui-button rm-promo-submit"
   href="/app/resource/{}/promote/pay/{}">
    Перейти к оплате · {}
</a>
"#,
                params.resource_id, request_id, price_label
            )
        } else {
            String::new()
        }
    } else if params.existing_status.is_some() {
        let status_note = match params.existing_status {
            Some("pending") if params.existing_payment_status == Some("paid") => {
                "Оплата принята. Заявка ожидает проверки администратором перед публикацией в группе."
            }
            Some("failed") => {
                "Публикация не удалась. Администратор может помочь завершить размещение."
            }
            _ => {
                "Повторная заявка не требуется. Дождитесь публикации или решения администратора."
            }
        };
        format!(
            r#"
<div class="card rm-promo-pending">
    <div class="card-title">Заявка в обработке</div>
    <div class="card-meta rm-promo-pending-copy">{status_note}</div>
</div>
"#
        )
    } else {
        format!(
            r#"
<form method="post"
      action="/app/resource/{resource_id}/promote/request"
      class="ui-form rm-promo-form">
    <input type="hidden"
           name="target_id"
           value="{target_id}">

    <div class="card-meta rm-promo-price-note">
        Стоимость публикации в группе: <strong>{price_label}</strong>
    </div>

    <button type="submit" class="ui-button rm-promo-submit">
        Продвинуть в Telegram-группу
    </button>
</form>
"#,
            resource_id = params.resource_id,
            target_id = params.target_id,
            price_label = price_label,
        )
    };

    let content = format!(
        r#"
<section class="card rm-promo-preview">

    <div class="rm-promo-preview-head">
        RESURSMAP · {city_name}
    </div>

    <div class="rm-promo-preview-body">

        <div class="rm-promo-preview-category">
            {listing_type} · {category}
        </div>

        <h2 class="rm-promo-preview-title">
            {title}
        </h2>

        <div class="rm-promo-preview-text">
            {description}
        </div>

        <div class="rm-promo-preview-address">
            📍 {address}
        </div>

        <div class="rm-promo-preview-footer">
            ResursMap — люди, услуги и возможности рядом
        </div>

        <div class="rm-promo-preview-domain">
            resursmap.de
        </div>

    </div>
</section>

<section class="card rm-promo-target-card">
    <div class="card-title">
        Место публикации
    </div>

    <div class="card-meta rm-promo-target-copy">
        {target_name} · {city_name}
    </div>

    <div class="card-meta rm-promo-target-note">
        Сначала оплата, затем — автопубликация (если проверка пройдена)
        или модерация администратором. Контакты остаются на странице объявления.
    </div>
</section>

{action_html}
"#,
        city_name = city_name,
        listing_type = listing_type,
        category = category,
        title = title,
        description = description,
        address = address,
        target_name = target_name,
        action_html = action_html,
    );

    page_shell(
        "Продвижение · ResursMap",
        &topbar("Продвижение", "map-pin"),
        &back_hero(
            &back_link(
                &format!("/app/resource/{}", params.resource_id,),
                "Объявление",
                "arrow-left",
            ),
            "map-pin",
            "Городская публикация",
            "Продвижение",
            "Предварительный просмотр публикации ResursMap.",
        ),
        &content,
        "",
    )
}

pub fn render_promotion_payment(
    resource_id: i64,
    request_id: i64,
    price_label: &str,
    bot_note: &str,
    bot_reason: Option<&str>,
) -> String {
    let price = escape_html(price_label);
    let note = escape_html(bot_note);
    let reason_html = bot_reason
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            format!(
                r#"<div class="card-meta rm-promo-bot-reason">Причина проверки: {}</div>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();

    let content = format!(
        r#"
<section class="card rm-promo-payment-card">
    <div class="card-title">Оплата публикации в группе</div>
    <div class="card-meta rm-promo-price-note">
        К оплате: <strong>{price}</strong>
    </div>
    <div class="card-meta">{note}</div>
    {reason_html}
    <form method="post"
          action="/app/resource/{resource_id}/promote/pay/{request_id}"
          class="ui-form rm-promo-form">
        <button type="submit" class="ui-button rm-promo-submit">
            Подтвердить оплату · {price}
        </button>
    </form>
    <div class="card-meta rm-promo-payment-footnote">
        Платёжный шлюз подключается отдельно. Сейчас кнопка фиксирует оплату для тестового контура.
    </div>
</section>
"#,
        price = price,
        note = note,
        reason_html = reason_html,
        resource_id = resource_id,
        request_id = request_id,
    );

    page_shell(
        "Оплата · ResursMap",
        &topbar("Оплата", "credit-card"),
        &back_hero(
            &back_link(
                &format!("/app/resource/{resource_id}/promote"),
                "Назад",
                "arrow-left",
            ),
            "credit-card",
            "Публикация в группе",
            "Оплата",
            "После оплаты объявление отправится в Telegram-группу города.",
        ),
        &content,
        "",
    )
}

#[allow(clippy::type_complexity)]
pub fn render_admin_promotion_queue(
    rows: &[(i64, i64, String, String, String, String, String, i64)],
) -> String {
    fn listing_kind(raw: &str) -> &'static str {
        match raw.trim() {
            "seeker" => "Ищу работу",
            "offer" => "Предложение",
            _ => "Объявление",
        }
    }

    let cards = if rows.is_empty() {
        r#"<div class="card"><div class="card-meta">Нет оплаченных заявок, ожидающих модерации.</div></div>"#
            .to_string()
    } else {
        rows.iter()
            .map(
                |(
                    request_id,
                    resource_id,
                    title,
                    category,
                    listing_type,
                    bot_status,
                    bot_reason,
                    _created_at,
                )| {
                    let safe_title = escape_html(title);
                    let safe_category = escape_html(category);
                    let kind = escape_html(listing_kind(listing_type));
                    let safe_reason = escape_html(bot_reason);
                    format!(
                        r#"
<section class="card rm-admin-promo-card">
    <div class="card-title">{safe_title}</div>
    <div class="card-meta">{kind} · {safe_category} · ID {resource_id}</div>
    <div class="card-meta">Проверка бота: {bot_status}{reason}</div>
    <div class="rm-admin-promo-actions">
        <form method="post" action="/app/admin/promotion/{request_id}/approve">
            <button type="submit" class="ui-button">Одобрить и опубликовать</button>
        </form>
        <form method="post" action="/app/admin/promotion/{request_id}/reject">
            <button type="submit" class="ui-button rm-admin-reject-btn">Отклонить</button>
        </form>
    </div>
</section>
"#,
                        safe_title = safe_title,
                        kind = kind,
                        safe_category = safe_category,
                        resource_id = resource_id,
                        bot_status = escape_html(bot_status),
                        reason = if safe_reason.is_empty() {
                            String::new()
                        } else {
                            format!(" · {safe_reason}")
                        },
                        request_id = request_id,
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    page_shell(
        "Продвижение · Админ",
        &topbar("Продвижение", "megaphone"),
        r#"<section class="hero"><h1>Очередь продвижения</h1><p>Оплаченные заявки, требующие решения администратора.</p></section>"#,
        &cards,
        "",
    )
}

// ============================================================
// МОИ РЕСУРСЫ
// ============================================================

pub fn render_my_resources(
    client_id: &str,
    resources: Vec<crate::web::view_models::MyResourceRow>,
) -> String {
    let cards = if client_id.is_empty() {
        guest_locked_section("Мои ресурсы", "/app/my-resources")
    } else if resources.is_empty() {
        empty_state_card_with_actions(
            "Нет опубликованных ресурсов",
            "Добавьте ресурс в выбранном городе и категории.",
            &empty_state_action("/app", "Открыть карту"),
        )
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
                    premium_badge_html("compact")
                } else {
                    ""
                };

                let moderation_badge =
                    my_resource_moderation_badge(*is_active, moderation_status);

                let hidden_html =
                    if *is_active == 0
                        && moderation_status != "rejected"
                    {
                        r#"<div class="rm-my-resource-note rm-my-resource-note--hidden"><strong>Ресурс скрыт.</strong> Публикация недоступна другим участникам.</div>"#
                            .to_string()
                    } else {
                        String::new()
                    };

                let promotion_button =
                    if moderation_status == "approved"
                        && *is_active == 1
                    {
                        format!(
                            r#"<a href="/app/resource/{id}/promote" class="rm-my-resource-action rm-my-resource-action--gold">Продвинуть</a>"#,
                            id = id,
                        )
                    } else {
                        String::new()
                    };

                let rejection_html =
                    if moderation_status == "rejected"
                        && !rejection_reason.trim().is_empty()
                    {
                        format!(
                            r#"<div class="rm-my-resource-note rm-my-resource-note--rejected"><strong>Причина отказа:</strong> {}</div>"#,
                            safe_rejection_reason
                        )
                    } else {
                        String::new()
                    };

                format!(
                    r#"
                    <article class="card rm-my-resource-card">

                        <div class="rm-my-resource-layout">

                            <div class="card-icon rm-my-resource-icon">
                                {icon}
                            </div>

                            <div class="rm-my-resource-body">

                                <div class="rm-my-resource-head">
                                    <div class="rm-my-resource-title-wrap">
                                        <div class="card-title rm-my-resource-title">
                                            {title}
                                        </div>

                                        <div class="card-meta rm-my-resource-category">
                                            {category}
                                        </div>
                                    </div>

                                    <div class="rm-my-resource-rating">
                                        ⭐ {rating:.1} · {votes}
                                    </div>
                                </div>

                                <div class="card-meta rm-my-resource-desc">
                                    {description}
                                </div>

                                <div class="rm-my-resource-badges">
                                    {premium_badge}
                                    {moderation_badge}
                                </div>

                                {rejection_html}
                                {hidden_html}

                                <div class="rm-my-resource-actions">

                                    <a href="/app/resource/{id}/edit"
                                       class="rm-my-resource-action rm-my-resource-action--edit">
                                        ✎ Редактировать
                                    </a>

                                    {promotion_button}

                                    <a href="/app/resource/{id}"
                                       class="rm-my-resource-action rm-my-resource-action--neutral">
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
                    promotion_button = promotion_button,
                )
            })
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
        "Мои ресурсы · ResursMap",
        &topbar("Мои ресурсы", "map"),
        &back_hero(
            &back_link("/app/me", "Профиль", "arrow-left"),
            "user",
            "Управление",
            "Мои ресурсы",
            "Ваши объявления, компании, услуги и другие ресурсы.",
        ),
        &content,
        &bottom_nav("profile"),
    )
}

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
    let content = format!(
        r####"<form method="post"
      action="/app/resource/{id}/edit"
      class="ui-form ui-form-stack">

    <label class="ui-field">
        <span class="ui-field-label">Название</span>
        <input
            name="title"
            required
            maxlength="120"
            value="{title}"
            class="ui-input">
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Описание</span>
        <textarea
            name="description"
            required
            maxlength="1000"
            rows="6"
            class="ui-textarea">{description}</textarea>
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Телефон или Telegram</span>
        <input
            name="contact"
            maxlength="120"
            value="{contact}"
            class="ui-input">
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Адрес</span>
        <input
            name="address"
            maxlength="250"
            value="{address}"
            class="ui-input">
    </label>

    <button type="submit" class="ui-button rm-auth-button">
        Сохранить изменения
    </button>

    <div class="ui-form-note">
        После сохранения ресурс автоматически вернётся на повторную модерацию.
    </div>

</form>"####,
        id = id,
        title = safe_title,
        description = safe_description,
        contact = safe_contact,
        address = safe_address,
    );

    page_shell(
        "Редактировать ресурс · ResursMap",
        &topbar("Редактирование", "map"),
        &back_hero(
            &back_link(
                &format!("/app/resource/{}", id),
                "Назад к ресурсу",
                "arrow-left",
            ),
            "edit",
            "Редактирование",
            "Редактировать ресурс",
            &format!("Категория: {}", category),
        ),
        &content,
        &bottom_nav("profile"),
    )
}

pub fn render_add_resource(ci: usize, si: usize, zi: usize, category: &str) -> String {
    let back_url = format!("/app/{}/{}/{}", ci, si, zi);
    let listing_type_field = if category.eq_ignore_ascii_case("work") {
        r#"
    <label class="ui-field">
        <span class="ui-field-label">Тип объявления</span>
        <select name="listing_type" class="ui-input">
            <option value="offer">Предложение работы / услуги</option>
            <option value="seeker">Ищу работу</option>
        </select>
    </label>
"#
    } else {
        ""
    };

    let content = format!(
        r####"<form method="post"
      action="/app/{}/{}/{}/cat/{}/add"
      class="ui-form ui-form-stack">

    {listing_type_field}

    <label class="ui-field">
        <span class="ui-field-label">Название</span>
        <input
            name="title"
            required
            maxlength="120"
            placeholder="Например: Охранная компания"
            class="ui-input">
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Описание</span>
        <textarea
            name="description"
            required
            maxlength="1000"
            rows="5"
            placeholder="Расскажите о ресурсе..."
            class="ui-textarea"></textarea>
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Телефон или Telegram</span>
        <input
            name="contact"
            maxlength="120"
            placeholder="+33... или @username"
            class="ui-input">
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Адрес</span>
        <input
            name="address"
            maxlength="200"
            placeholder="Город, улица..."
            class="ui-input">
    </label>

    <button type="submit" class="ui-button rm-auth-button">
        ➕ Добавить ресурс
    </button>

</form>"####,
        ci,
        si,
        zi,
        category,
        listing_type_field = listing_type_field,
    );

    let main_html = format!(
        "{topbar}\n\n{hero}\n\n{content}",
        topbar = topbar("Новый ресурс", "globe"),
        hero = back_hero(
            &back_link(&back_url, "Вернуться", "chevron"),
            "plus",
            "Добавление ресурса",
            "Добавить ресурс",
            &format!("Категория: {}", category),
        ),
        content = content,
    );

    page_document(
        "Добавить ресурс · ResursMap",
        "",
        "",
        &main_html,
        &bottom_nav("profile"),
        "",
    )
}
