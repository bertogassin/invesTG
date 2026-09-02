use super::common::{
    back_hero, back_link, bottom_nav, empty_state_action, empty_state_card_with_actions,
    escape_html, guest_locked_section, icon, kind_chip, my_resource_moderation_badge,
    navigation_card, page_document, page_shell, premium_badge_html, profession_label,
    resource_card_link_class, resource_detail_section_class, resource_listing_label,
    search_people_cards, section_head, topbar, verified_badge_html,
};

pub struct RenderCategoryParams<'a> {
    pub ci: usize,
    pub si: usize,
    pub zi: usize,
    pub category: &'a str,
    pub listing_type: Option<&'a str>,
    pub active_rubric: Option<&'a str>,
    pub sort: &'a str,
    pub resources: Vec<crate::web::view_models::CategoryResourceRow>,
    pub people: Vec<crate::web::view_models::SearchPersonRow>,
}

fn category_list_href(
    ci: usize,
    si: usize,
    zi: usize,
    category: &str,
    listing_type: Option<&str>,
    rubric: Option<&str>,
    sort: &str,
) -> String {
    let base = if category.eq_ignore_ascii_case("all") {
        format!("/app/{ci}/{si}/{zi}/all")
    } else {
        format!(
            "/app/{ci}/{si}/{zi}/cat/{}",
            urlencoding::encode(category)
        )
    };
    let mut parts = Vec::new();
    match listing_type {
        Some("seeker") => parts.push("type=seeker".to_string()),
        Some("offer") => parts.push("type=offer".to_string()),
        _ => {}
    }
    if let Some(rubric) = rubric {
        parts.push(format!("rubric={}", urlencoding::encode(rubric)));
    }
    if sort == "new" {
        parts.push("sort=new".to_string());
    }
    if parts.is_empty() {
        base
    } else {
        format!("{base}?{}", parts.join("&"))
    }
}

pub fn render_category(params: RenderCategoryParams<'_>) -> String {
    let RenderCategoryParams {
        ci,
        si,
        zi,
        category,
        listing_type,
        active_rubric,
        sort,
        resources,
        people,
    } = params;
    let city_url = format!("/app/{}/{}/{}", ci, si, zi);
    let category_url = urlencoding::encode(category);
    let type_query = match listing_type {
        Some("seeker") => "type=seeker",
        Some("offer") => "type=offer",
        _ => "",
    };
    let add_url = if category.eq_ignore_ascii_case("all") {
        city_url.clone()
    } else {
        let mut href = format!("/app/{ci}/{si}/{zi}/cat/{category_url}/add");
        let mut parts = Vec::new();
        if !type_query.is_empty() {
            parts.push(type_query.to_string());
        }
        if let Some(rubric) = active_rubric {
            parts.push(format!("rubric={}", urlencoding::encode(rubric)));
        }
        if !parts.is_empty() {
            href.push('?');
            href.push_str(&parts.join("&"));
        }
        href
    };
    let cards = if resources.is_empty() {
        if people.is_empty() {
            empty_state_card_with_actions(
                "Пока пусто",
                if category.eq_ignore_ascii_case("all") {
                    "В этом городе пока нет опубликованных объявлений."
                } else if listing_type == Some("seeker") {
                    "Добавьте объявление или укажите профессию в профиле."
                } else {
                    "Добавьте первое объявление в этом разделе."
                },
                &empty_state_action(
                    &add_url,
                    if category.eq_ignore_ascii_case("all") {
                        "К разделам"
                    } else {
                        "Добавить"
                    },
                ),
            )
        } else {
            String::new()
        }
    } else {
        resources
            .iter()
            .map(
                |(
                    id,
                    title,
                    description,
                    contact,
                    address,
                    rating,
                    votes,
                    verified,
                    premium,
                    row_listing_type,
                    row_rubric,
                )| {
                    let safe_title = escape_html(title);
                    let safe_description = escape_html(description);
                    let safe_contact = escape_html(contact);
                    let safe_address = escape_html(address);
                    let rubric_label = profession_label(row_rubric);
                    let listing_label = {
                        let mut parts = Vec::new();
                        if listing_type.is_none() {
                            parts.push(resource_listing_label(row_listing_type).to_string());
                        }
                        if !rubric_label.is_empty()
                            && !matches!(
                                rubric_label.as_str(),
                                "Работа" | "Бизнес" | "Услуги" | "Сообщество"
                            )
                        {
                            parts.push(rubric_label);
                        }
                        if parts.is_empty() {
                            String::new()
                        } else {
                            format!(
                                r#"<div class="card-meta">{}</div>"#,
                                escape_html(&parts.join(" · "))
                            )
                        }
                    };

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

                            {listing_label}

                            <div class="card-meta">
                                {description}
                            </div>

                            <div class="card-meta">
                                Оценка {rating:.1} · {votes} голосов
                            </div>

                            <div class="card-meta">
                                {address}
                            </div>

                            <div class="card-meta">
                                {contact}
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
                        listing_label = listing_label,
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
    let people_count = people.len();
    let people_section = if people.is_empty() {
        String::new()
    } else {
        format!(
            r#"{head}
<div>{cards}</div>"#,
            head = section_head("По профессии", &format!("Найдено: {people_count}"), None,),
            cards = search_people_cards(&people),
        )
    };

    let work_chips = if category.eq_ignore_ascii_case("work") {
        format!(
            r#"<nav class="rm-kind-chips" aria-label="Что искать">
    {work}{workers}
</nav>"#,
            work = kind_chip(
                listing_type == Some("offer"),
                &category_list_href(ci, si, zi, category, Some("offer"), active_rubric, sort),
                "Работа",
            ),
            workers = kind_chip(
                listing_type == Some("seeker"),
                &category_list_href(ci, si, zi, category, Some("seeker"), active_rubric, sort),
                "Работники",
            ),
        )
    } else {
        String::new()
    };

    let rubric_chips = if category.eq_ignore_ascii_case("all") {
        String::new()
    } else {
        let rubric_kind = if category.eq_ignore_ascii_case("work") {
            crate::catalog::RubricKind::Work
        } else {
            crate::catalog::RubricKind::Business
        };
        let mut chips = String::from(r#"<nav class="rm-kind-chips" aria-label="Рубрика">"#);
        chips.push_str(&kind_chip(
            active_rubric.is_none(),
            &category_list_href(ci, si, zi, category, listing_type, None, sort),
            "Все",
        ));
        for rubric in crate::catalog::by_kind(rubric_kind) {
            chips.push_str(&kind_chip(
                active_rubric == Some(rubric.id),
                &category_list_href(ci, si, zi, category, listing_type, Some(rubric.id), sort),
                rubric.label,
            ));
        }
        chips.push_str("</nav>");
        chips
    };

    let sort_chips = format!(
        r#"<nav class="rm-kind-chips" aria-label="Сортировка">
    {rating}{newest}
</nav>"#,
        rating = kind_chip(
            sort != "new",
            &category_list_href(ci, si, zi, category, listing_type, active_rubric, "rating"),
            "По рейтингу",
        ),
        newest = kind_chip(
            sort == "new",
            &category_list_href(ci, si, zi, category, listing_type, active_rubric, "new"),
            "Новые",
        ),
    );

    let section_head_resources = if resources.is_empty() {
        String::new()
    } else {
        section_head("Объявления", &format!("Найдено: {}", count), None)
    };

    let content = format!(
        r####"{work_chips}

{rubric_chips}

{sort_chips}

{section_head_resources}

<div>
    {cards}
</div>

{people_section}"####,
        work_chips = work_chips,
        rubric_chips = rubric_chips,
        sort_chips = sort_chips,
        section_head_resources = section_head_resources,
        cards = cards,
        people_section = people_section,
    );

    let heading = if let Some(rubric) = active_rubric.and_then(crate::catalog::by_id) {
        rubric.label
    } else {
        match (category.to_ascii_lowercase().as_str(), listing_type) {
            ("all", _) => "Все объявления",
            ("work", Some("offer")) => "Работа",
            ("work", Some("seeker")) => "Работники",
            ("work", _) => "Работа",
            ("business" | "services", _) => "Бизнес",
            _ => category,
        }
    };
    let heading_copy = if category.eq_ignore_ascii_case("all") {
        "Все опубликованные объявления в городе."
    } else {
        match listing_type {
            Some("offer") => "Вакансии и предложения работы в городе.",
            Some("seeker") => "Объявления и специалисты по профессии в городе.",
            _ => "Объявления города в этом разделе.",
        }
    };

    page_shell(
        &format!("{} · ResursMap", heading),
        &topbar("Категория", "globe"),
        &back_hero(
            &back_link(&city_url, "Вернуться к городу", "chevron"),
            "map",
            "Раздел",
            heading,
            heading_copy,
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
    pub listing_type: &'a str,
    pub continent_index: i64,
    pub country_index: i64,
    pub city_index: i64,
    pub _created_at: i64,
    pub owner_public_id: &'a str,
    pub rubric: &'a str,
    pub owner_preview: bool,
    pub moderation_status: &'a str,
    pub is_active: i64,
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
        listing_type,
        continent_index,
        country_index,
        city_index,
        _created_at,
        owner_public_id,
        rubric,
        owner_preview,
        moderation_status,
        is_active,
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

    let listing_label = resource_listing_label(listing_type);
    let rubric_label = if crate::catalog::by_id(rubric).is_some() {
        profession_label(rubric)
    } else {
        profession_label(category)
    };
    let category_url = urlencoding::encode(category);
    let type_query = match listing_type.trim() {
        "seeker" => "?type=seeker",
        "offer" => "?type=offer",
        _ => "",
    };
    let (back_url, back_label) = if owner_preview {
        ("/app/my-resources".to_string(), "Мои ресурсы")
    } else {
        (
            format!(
                "/app/{continent_index}/{country_index}/{city_index}/cat/{category_url}{type_query}"
            ),
            "Вернуться к разделу",
        )
    };
    let moderation_banner = if !owner_preview {
        String::new()
    } else if moderation_status == "rejected" {
        r#"<section class="card rm-resource-moderation-banner">
    <div class="card-title">Объявление отклонено</div>
    <div class="card-meta">Исправьте текст и сохраните снова — оно уйдёт на повторную проверку.</div>
    <a href="/app/resource/{id}/edit" class="ui-button rm-auth-button">Редактировать</a>
</section>"#
            .replace("{id}", &id.to_string())
    } else if is_active == 0 {
        String::from(
            r#"<section class="card rm-resource-moderation-banner">
    <div class="card-title">Объявление скрыто</div>
    <div class="card-meta">Другие участники его не видят.</div>
</section>"#,
        )
    } else {
        String::from(
            r#"<section class="card rm-resource-moderation-banner">
    <div class="card-title">На проверке</div>
    <div class="card-meta">Другие участники это объявление пока не видят. После одобрения оно появится в поиске и в городе.</div>
</section>"#,
        )
    };

    let hero_description = format!(
        r#"<span class="rm-resource-hero-badges">
            {premium_badge}
            {verified_badge}
        </span>

        <span class="rm-resource-listing-label">{listing_label}</span>

        <span id="rating-summary" class="rm-resource-rating-summary">
            Оценка <strong>{rating:.1}</strong> · {votes} голосов
        </span>"#,
        premium_badge = premium_badge,
        verified_badge = verified_badge,
        listing_label = listing_label,
        rating = rating,
        votes = votes,
    );
    let public_actions_html = if owner_preview {
        String::new()
    } else {
        r#"<button
        id="favorite-button"
        type="button"
        class="ui-button rm-resource-favorite-btn">
        В избранное
    </button>

    <button
        type="button"
        class="ui-button"
        data-share
        data-share-title="Объявление ResursMap"
        data-share-status="share-status">
        Поделиться
    </button>
    <div id="share-status" class="ui-status"></div>

    <div id="favorite-status" class="ui-status rm-resource-favorite-status">
    </div>

    <button
        id="report-button"
        type="button"
        class="ui-button rm-resource-report-btn">
        Пожаловаться
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
    </div>"#
            .to_string()
    };

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
        r####"{moderation_banner}
<section>
{public_actions}
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
        {address}
    </div>

    <div class="card-meta rm-resource-contact-line rm-resource-contact-line--last">
        {contact}
    </div>

    <div class="rm-resource-contact-actions">

        <a href="{contact_href}" class="rm-resource-contact-btn rm-resource-contact-btn--gold">
            Связаться
        </a>

        <a href="{map_href}"
           target="_blank"
           rel="noopener noreferrer"
           class="rm-resource-contact-btn rm-resource-contact-btn--neutral">
            На карте
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
        moderation_banner = moderation_banner,
        public_actions = public_actions_html,
        description = safe_description,
        owner_profile_html = owner_profile_html,
        address = safe_address,
        contact = safe_contact,
        contact_href = safe_contact_href,
        map_href = safe_map_href,
        detail_section_class = detail_section_class,
        id = id,
    );

    let body_after = if owner_preview {
        String::new()
    } else {
        format!(
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
                ? "В избранном"
                : "В избранное";
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
                    `Оценка <strong>${{Number(data.rating).toFixed(1)}}</strong> · ${{data.votes}} голосов`;

                status.textContent = "✓ Ваша оценка сохранена";
            }} catch (_) {{
                status.textContent = "Ошибка соединения.";
            }}
        }});
    }});
}})();
</script>"####,
        id = id,
    )
    };

    page_document(
        &format!("{} · ResursMap", title),
        if owner_preview {
            r#"<meta name="robots" content="noindex, nofollow">"#
        } else {
            ""
        },
        "",
        &format!(
            "{topbar}\n\n{hero}\n\n{content}",
            topbar = topbar("Ресурс", "map"),
            hero = back_hero(
                &back_link(&back_url, back_label, "arrow-left"),
                "map-pin",
                &rubric_label,
                title,
                &hero_description,
            ),
            content = main_html,
        ),
        &bottom_nav(if owner_preview { "menu" } else { "map" }),
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
    pub telegram_ready: bool,
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

    let telegram_warning_html = if params.telegram_ready {
        String::new()
    } else {
        r#"
<div class="card rm-promo-pending">
    <div class="card-title">Telegram не настроен</div>
    <div class="card-meta rm-promo-pending-copy">
        Оплата и заявка возможны, но автоматическая публикация в группу
        заработает только после настройки TELEGRAM_BOT_TOKEN на сервере.
    </div>
</div>
"#
        .to_string()
    };

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
                format!(r#"<div class="card-meta rm-promo-bot-reason">Причина: {reason}</div>"#)
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
            {address}
        </div>

        <div class="rm-promo-preview-footer">
            ResursMap — работа, работники и бизнес рядом
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

{telegram_warning_html}

{action_html}
"#,
        city_name = city_name,
        listing_type = listing_type,
        category = category,
        title = title,
        description = description,
        address = address,
        target_name = target_name,
        telegram_warning_html = telegram_warning_html,
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
    stripe_enabled: bool,
    mock_allowed: bool,
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

    let (payment_form, payment_footnote) = if stripe_enabled {
        (
            format!(
                r#"<form method="post"
          action="/app/resource/{resource_id}/promote/pay/{request_id}"
          class="ui-form rm-promo-form">
        <button type="submit" class="ui-button rm-promo-submit">
            Перейти к оплате Stripe · {price}
        </button>
    </form>"#,
                resource_id = resource_id,
                request_id = request_id,
                price = price,
            ),
            "Оплата проходит через Stripe Checkout. После успешной оплаты объявление будет опубликовано автоматически.",
        )
    } else if mock_allowed {
        (
            format!(
                r#"<form method="post"
          action="/app/resource/{resource_id}/promote/pay/{request_id}"
          class="ui-form rm-promo-form">
        <button type="submit" class="ui-button rm-promo-submit">
            Подтвердить оплату · {price}
        </button>
    </form>"#,
                resource_id = resource_id,
                request_id = request_id,
                price = price,
            ),
            "Тестовый контур: кнопка фиксирует оплату без Stripe. Для продакшена задайте STRIPE_SECRET_KEY.",
        )
    } else {
        (
            r#"<div class="card-meta rm-promo-payment-unavailable">
        Оплата временно недоступна. Платёжный сервис не настроен на этом сервере.
    </div>"#.to_string(),
            "Для production требуется STRIPE_SECRET_KEY или явный ALLOW_MOCK_PROMOTION_PAYMENT=1 на localhost.",
        )
    };

    let content = format!(
        r#"
<section class="card rm-promo-payment-card">
    <div class="card-title">Оплата публикации в группе</div>
    <div class="card-meta rm-promo-price-note">
        К оплате: <strong>{price}</strong>
    </div>
    <div class="card-meta">{note}</div>
    {reason_html}
    {payment_form}
    <div class="card-meta rm-promo-payment-footnote">
        {payment_footnote}
    </div>
</section>
"#,
        price = price,
        note = note,
        reason_html = reason_html,
        payment_form = payment_form,
        payment_footnote = escape_html(payment_footnote),
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
    notice: Option<&str>,
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
            <button type="submit" class="ui-button rm-admin-reject-btn">Отклонить и вернуть оплату</button>
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

    let notice_html = notice
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            format!(
                r#"<div class="card rm-admin-promo-notice" role="status">{}</div>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();

    page_shell(
        "Продвижение · Админ",
        &topbar("Продвижение", "megaphone"),
        &format!(
            r#"<section class="hero"><h1>Очередь продвижения</h1><p>Оплаченные заявки, требующие решения администратора.</p></section>{notice_html}"#,
            notice_html = notice_html,
        ),
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
            &format!(
                "{}{}",
                empty_state_action("/app/add", "Добавить объявление"),
                empty_state_action("/app", "Открыть города"),
            ),
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
                is_active,
                listing_type,
                row_rubric,
            )| {
                let safe_title = escape_html(title);
                let safe_description = escape_html(description);
                let safe_rejection_reason = escape_html(rejection_reason);
                let rubric_label = profession_label(row_rubric);
                let category_label = if !rubric_label.is_empty()
                    && !matches!(rubric_label.as_str(), "Работа" | "Бизнес")
                {
                    rubric_label
                } else {
                    profession_label(category)
                };
                let category_line = format!(
                    "{} · {}",
                    resource_listing_label(listing_type),
                    escape_html(&category_label)
                );

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
                                        Оценка {rating:.1} · {votes}
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
                    category = category_line,
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
            "Ваши объявления по работе и бизнесу.",
        ),
        &content,
        &bottom_nav("menu"),
    )
}

pub struct RenderEditResourceParams<'a> {
    pub id: i64,
    pub title: &'a str,
    pub description: &'a str,
    pub contact: &'a str,
    pub address: &'a str,
    pub category: &'a str,
    pub listing_type: &'a str,
    pub rubric: &'a str,
}

pub fn render_edit_resource(params: RenderEditResourceParams<'_>) -> String {
    let RenderEditResourceParams {
        id,
        title,
        description,
        contact,
        address,
        category,
        listing_type,
        rubric,
    } = params;
    let safe_title = escape_html(title);
    let safe_description = escape_html(description);
    let safe_contact = escape_html(contact);
    let safe_address = escape_html(address);
    let listing_type_field = if crate::catalog::by_id(rubric)
        .map(|item| item.kind == crate::catalog::RubricKind::Work)
        .unwrap_or_else(|| category.eq_ignore_ascii_case("work"))
    {
        let offer_selected = if listing_type != "seeker" {
            " selected"
        } else {
            ""
        };
        let seeker_selected = if listing_type == "seeker" {
            " selected"
        } else {
            ""
        };

        format!(
            r#"
    <label class="ui-field">
        <span class="ui-field-label">Тип объявления</span>
        <select name="listing_type" class="ui-input">
            <option value="offer"{offer_selected}>Предложение работы</option>
            <option value="seeker"{seeker_selected}>Ищу работу</option>
        </select>
    </label>
"#
        )
    } else {
        String::new()
    };
    let rubric_field = rubric_select_html(
        rubric,
        if category.eq_ignore_ascii_case("work") {
            Some(crate::catalog::RubricKind::Work)
        } else {
            Some(crate::catalog::RubricKind::Business)
        },
    );
    let content = format!(
        r####"<form method="post"
      action="/app/resource/{id}/edit"
      class="ui-form ui-form-stack">

    {rubric_field}

    {listing_type_field}

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
        listing_type_field = listing_type_field,
        rubric_field = rubric_field,
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
            &format!("Рубрика: {}", profession_label(rubric)),
        ),
        &content,
        &bottom_nav("menu"),
    )
}

pub fn render_add_rubric_picker(
    ci: usize,
    si: usize,
    zi: usize,
    category: &str,
    listing_type: Option<&str>,
) -> String {
    let category_url = urlencoding::encode(category);
    let back_url = match listing_type {
        Some("seeker") => format!("/app/{ci}/{si}/{zi}/cat/{category_url}?type=seeker"),
        Some("offer") => format!("/app/{ci}/{si}/{zi}/cat/{category_url}?type=offer"),
        _ => format!("/app/{ci}/{si}/{zi}/cat/{category_url}"),
    };
    let rubric_kind = if category.eq_ignore_ascii_case("work") {
        crate::catalog::RubricKind::Work
    } else {
        crate::catalog::RubricKind::Business
    };
    let type_query = match listing_type {
        Some("seeker") => "type=seeker&",
        Some("offer") => "type=offer&",
        _ => "",
    };
    let mut cards = String::new();
    for rubric in crate::catalog::by_kind(rubric_kind) {
        cards.push_str(&navigation_card(
            &format!(
                "/app/{ci}/{si}/{zi}/cat/{category_url}/add?{type_query}rubric={}",
                urlencoding::encode(rubric.id)
            ),
            if rubric_kind == crate::catalog::RubricKind::Work {
                "briefcase"
            } else {
                "building"
            },
            rubric.label,
            "Выберите, затем заполните объявление",
        ));
    }

    let heading = match listing_type {
        Some("seeker") => "Кем вы хотите работать?",
        Some("offer") => "Какая вакансия?",
        _ => "Выберите рубрику",
    };

    page_shell(
        "Выбор рубрики · ResursMap",
        &topbar("Новое объявление", "globe"),
        &back_hero(
            &back_link(&back_url, "Назад", "chevron"),
            "map",
            "Сначала рубрика",
            heading,
            "Один список для поиска и публикации — без свободного ввода.",
        ),
        &format!(
            r#"<div class="grid">{cards}</div>"#,
            cards = cards
        ),
        &bottom_nav("map"),
    )
}

fn rubric_select_html(selected: &str, kind: Option<crate::catalog::RubricKind>) -> String {
    let mut options = String::from(r#"<option value="">Выберите из списка</option>"#);
    let groups = match kind {
        Some(crate::catalog::RubricKind::Work) => {
            vec![(crate::catalog::RubricKind::Work, "Работа и работники")]
        }
        Some(crate::catalog::RubricKind::Business) => {
            vec![(crate::catalog::RubricKind::Business, "Бизнес")]
        }
        None => vec![
            (crate::catalog::RubricKind::Work, "Работа и работники"),
            (crate::catalog::RubricKind::Business, "Бизнес"),
        ],
    };

    for (group_kind, group_label) in groups {
        options.push_str(&format!(
            r#"<optgroup label="{}">"#,
            escape_html(group_label)
        ));
        for rubric in crate::catalog::by_kind(group_kind) {
            let selected_attr = if selected == rubric.id { " selected" } else { "" };
            options.push_str(&format!(
                r#"<option value="{id}"{selected_attr}>{label}</option>"#,
                id = escape_html(rubric.id),
                selected_attr = selected_attr,
                label = escape_html(rubric.label),
            ));
        }
        options.push_str("</optgroup>");
    }

    format!(
        r#"
    <label class="ui-field">
        <span class="ui-field-label">Рубрика</span>
        <select name="rubric" required class="ui-input">
            {options}
        </select>
    </label>
"#
    )
}

pub struct AddResourceDraft<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub contact: &'a str,
    pub address: &'a str,
}

#[allow(clippy::too_many_arguments)]
pub fn render_add_resource(
    ci: usize,
    si: usize,
    zi: usize,
    category: &str,
    listing_type: Option<&str>,
    rubric: &crate::catalog::Rubric,
    draft: Option<AddResourceDraft<'_>>,
    error: Option<&str>,
) -> String {
    let category_url = urlencoding::encode(category);
    let picker_url = match listing_type {
        Some("seeker") => format!("/app/{ci}/{si}/{zi}/cat/{category_url}/add?type=seeker"),
        Some("offer") => format!("/app/{ci}/{si}/{zi}/cat/{category_url}/add?type=offer"),
        _ => format!("/app/{ci}/{si}/{zi}/cat/{category_url}/add"),
    };
    let listing_hidden = match listing_type {
        Some("seeker") => r#"<input type="hidden" name="listing_type" value="seeker">"#,
        Some("offer") => r#"<input type="hidden" name="listing_type" value="offer">"#,
        _ => "",
    };
    let heading = match listing_type {
        Some("seeker") => "Ищу работу",
        Some("offer") => "Предлагаю работу",
        _ => "Новое объявление",
    };

    let error_html = error
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            format!(
                r#"<p class="ui-status is-error" role="alert">{}</p>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();
    let title_value = draft.as_ref().map(|value| value.title).unwrap_or("");
    let description_value = draft.as_ref().map(|value| value.description).unwrap_or("");
    let contact_value = draft.as_ref().map(|value| value.contact).unwrap_or("");
    let address_value = draft.as_ref().map(|value| value.address).unwrap_or("");

    let content = format!(
        r####"{error_html}
<form method="post"
      action="/app/{ci}/{si}/{zi}/cat/{category_url}/add"
      class="ui-form ui-form-stack">

    {listing_hidden}
    <input type="hidden" name="rubric" value="{rubric_id}">

    <div class="ui-field">
        <span class="ui-field-label">Рубрика</span>
        <div class="card-title">{rubric_label}</div>
        <a class="card-meta" href="{picker_url}">Изменить рубрику</a>
    </div>

    <label class="ui-field">
        <span class="ui-field-label">Название</span>
        <input
            name="title"
            required
            maxlength="120"
            placeholder="Кратко, без лишнего"
            class="ui-input"
            value="{title_value}">
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Описание</span>
        <textarea
            name="description"
            required
            maxlength="1000"
            rows="5"
            placeholder="Условия, опыт, что предлагаете или ищете"
            class="ui-textarea">{description_value}</textarea>
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Телефон или Telegram</span>
        <input
            name="contact"
            required
            maxlength="120"
            placeholder="+33... или @username"
            class="ui-input"
            value="{contact_value}">
    </label>

    <label class="ui-field">
        <span class="ui-field-label">Адрес</span>
        <input
            name="address"
            maxlength="200"
            placeholder="Город, район, улица"
            class="ui-input"
            value="{address_value}">
    </label>

    <button type="submit" class="ui-button rm-auth-button">
        Опубликовать
    </button>

</form>"####,
        error_html = error_html,
        ci = ci,
        si = si,
        zi = zi,
        category_url = category_url,
        listing_hidden = listing_hidden,
        rubric_id = escape_html(rubric.id),
        rubric_label = escape_html(rubric.label),
        picker_url = escape_html(&picker_url),
        title_value = escape_html(title_value),
        description_value = escape_html(description_value),
        contact_value = escape_html(contact_value),
        address_value = escape_html(address_value),
    );

    page_document(
        "Добавить объявление · ResursMap",
        "",
        "",
        &format!(
            "{topbar}\n\n{hero}\n\n{content}",
            topbar = topbar("Новое объявление", "globe"),
            hero = back_hero(
                &back_link(&picker_url, "К рубрикам", "chevron"),
                "briefcase",
                heading,
                rubric.label,
                "Рубрика уже выбрана. Осталось заполнить детали.",
            ),
            content = content,
        ),
        &bottom_nav("map"),
        "",
    )
}

#[cfg(test)]
mod catalog_publish_tests {
    use super::*;

    #[test]
    fn add_flow_starts_with_fixed_work_rubrics() {
        let html = render_add_rubric_picker(0, 0, 0, "work", Some("offer"));

        assert!(html.contains("Охрана"));
        assert!(html.contains("rubric=security"));
        assert!(html.contains("Какая вакансия?"));
    }

    #[test]
    fn add_form_locks_selected_rubric() {
        let rubric = crate::catalog::by_id("security").expect("security rubric");
        let html = render_add_resource(0, 0, 0, "work", Some("offer"), rubric, None, None);

        assert!(html.contains(r#"name="rubric""#));
        assert!(html.contains("security"));
        assert!(html.contains("Охрана"));
        assert!(html.contains("Изменить рубрику"));
    }

    #[test]
    fn owner_preview_explains_pending_and_hides_public_actions() {
        let html = render_resource_profile(RenderResourceProfileParams {
            id: 7,
            title: "Охранник в Ницце",
            description: "Ночная смена",
            contact: "@owner",
            address: "Nice",
            rating: 0.0,
            votes: 0,
            premium: 0,
            verified: 0,
            category: "work",
            listing_type: "seeker",
            continent_index: 0,
            country_index: 0,
            city_index: 0,
            _created_at: 0,
            owner_public_id: "abc",
            rubric: "security",
            owner_preview: true,
            moderation_status: "pending",
            is_active: 1,
        });

        assert!(html.contains("На проверке"));
        assert!(html.contains("Другие участники это объявление пока не видят"));
        assert!(html.contains("/app/my-resources"));
        assert!(!html.contains("id=\"favorite-button\""));
        assert!(!html.contains("В избранное"));
        assert!(!html.contains("Пожаловаться"));
        assert!(html.contains("noindex"));
    }
}
