use super::common::{
    back_hero, back_link, bottom_nav, empty_state_card, escape_html, guest_locked_section, icon,
    navigation_card, page_document, page_shell, section_head, topbar,
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
            <a class="feature"
               href="/app/{ci}/{si}/{zi}/cat/{category_url}/add"
               style="text-decoration:none;color:inherit;margin-top:12px;display:block;">
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
            .map(|(id, title, description, contact, address, rating, votes, verified, premium)| {
                            let safe_title = escape_html(title);
                let safe_description = escape_html(description);
                let safe_contact = escape_html(contact);
                let safe_address = escape_html(address);

                let verified_badge = if *verified != 0 {
                    r#"<span style="
                        color:#16a34a;
                        font-size:12px;
                        font-weight:700;
                    ">✓ Проверен</span>"#
                } else {
                    ""
                };

                let premium_badge = if *premium != 0 {
                    r#"<span style="
                        display:inline-flex;
                        align-items:center;
                        gap:5px;
                        padding:4px 9px;
                        border-radius:999px;
                        background:rgba(214,183,122,.12);
                        border:1px solid rgba(214,183,122,.45);
                        color:var(--gold-light);
                        font-size:10px;
                        font-weight:800;
                        letter-spacing:.10em;
                    ">★ PREMIUM</span>"#
                } else {
                    ""
                };

                let card_style = if *premium != 0 {
                    "margin-bottom:16px;                    border:1px solid rgba(214,183,122,.55);                    background:linear-gradient(145deg,var(--card),var(--card-hover));                    box-shadow:0 10px 32px rgba(214,183,122,.14),0 0 0 1px rgba(214,183,122,.06);                    position:relative;                    overflow:hidden;"
                } else {
                    "margin-bottom:14px;"
                };

                format!(
                    r#"
                    <a href="/app/resource/{}" class="card" style="{}text-decoration:none;color:inherit;">
                        {}
                        <div class="card-icon">{}</div>

                        <div class="card-content">

                            <div style="
                                display:flex;
                                align-items:center;
                                gap:8px;
                                flex-wrap:wrap;
                                margin-bottom:6px;
                            ">
                                <div class="card-title">
                                    {}
                                </div>

                                {}
                            </div>

                            <div class="card-meta">
                                {}
                            </div>

                            <div class="card-meta">
                                ⭐ {:.1} · {} голосов
                            </div>

                            <div class="card-meta">
                                📍 {}
                            </div>

                            <div class="card-meta">
                                📞 {}
                            </div>

                            <div style="margin-top:6px;">
                                {}
                            </div>

                        </div>

                        <div class="card-arrow">›</div>
                    </a>
                    "#,
                    id,
                    card_style,
                    if *premium != 0 {
                        r#"<div style="position:absolute;top:0;left:0;right:0;height:2px;background:linear-gradient(90deg,transparent,var(--gold),transparent);"></div>"#
                    } else {
                        ""
                    },
                    icon("map-pin"),
                    safe_title,
                    premium_badge,
                    safe_description,
                    rating,
                    votes,
                    safe_address,
                    safe_contact,
                    verified_badge
                )
            })
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
        r#"<span style="
            display:inline-flex;
            align-items:center;
            gap:6px;
            padding:6px 11px;
            border-radius:999px;
            background:rgba(214,183,122,.12);
            border:1px solid rgba(214,183,122,.45);
            color:var(--gold-light);
            font-size:11px;
            font-weight:800;
            letter-spacing:.08em;
        ">★ PREMIUM</span>"#
    } else {
        ""
    };

    let verified_badge = if verified != 0 {
        r#"<span style="
            display:inline-flex;
            align-items:center;
            gap:5px;
            color:#16a34a;
            font-size:12px;
            font-weight:700;
        ">✓ Проверен</span>"#
    } else {
        ""
    };

    let hero_description = format!(
        r#"<span style="
            display:flex;
            align-items:center;
            gap:9px;
            flex-wrap:wrap;
            margin-bottom:10px;
        ">
            {premium_badge}
            {verified_badge}
        </span>

        <span id="rating-summary"
              style="display:block;">
            ⭐ <strong>{rating:.1}</strong> · {votes} голосов
        </span>"#,
        premium_badge = premium_badge,
        verified_badge = verified_badge,
        rating = rating,
        votes = votes,
    );

    let premium_style = if premium != 0 {
        "border:1px solid rgba(214,183,122,.55);background:linear-gradient(145deg,var(--card),var(--card-hover));box-shadow:0 12px 38px rgba(214,183,122,.14);"
    } else {
        "border:1px solid rgba(0,0,0,.07);"
    };

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
<section class="card"
         style="
             display:flex;
             width:100%;
             box-sizing:border-box;
             margin-bottom:16px;
             padding:18px;
             text-decoration:none;
         ">

    <div class="card-icon"
         style="flex:0 0 auto;">
        {owner_icon}
    </div>

    <div class="card-content"
         style="min-width:0;">

        <div style="
            font-size:11px;
            text-transform:uppercase;
            letter-spacing:.07em;
            color:var(--muted);
            font-weight:800;
            margin-bottom:5px;
        ">
            Владелец ресурса
        </div>

        <div class="card-title">
            Профиль участника
        </div>

        <div class="card-meta"
             style="margin-top:4px;">
            Другие ресурсы и актуальный статус
        </div>

    </div>

    <a href="/app/user/{public_id}"
       style="
           flex:0 0 auto;
           align-self:center;
           min-height:40px;
           display:inline-flex;
           align-items:center;
           justify-content:center;
           padding:0 13px;
           border-radius:12px;
           text-decoration:none;
           color:var(--text);
           font-size:12px;
           font-weight:800;
           border:1px solid rgba(214,183,122,.32);
           background:rgba(214,183,122,.08);
       ">
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
        style="
            margin-top:12px;
            min-height:44px;
            padding:0 15px;
            border-radius:14px;
            border:1px solid rgba(220,38,38,.25);
            background:rgba(220,38,38,.06);
            color:var(--text);
            font-weight:800;
            cursor:pointer;
        " class="ui-button">
        ♡ В избранное
    </button>

    <div
        id="favorite-status"
        style="
            margin-top:7px;
            font-size:12px;
            color:var(--muted);
        " class="ui-status">
    </div>

    <button
        id="report-button"
        type="button"
        style="
            margin-top:10px;
            min-height:42px;
            padding:0 14px;
            border-radius:14px;
            border:1px solid rgba(217,119,6,.28);
            background:rgba(217,119,6,.06);
            color:var(--text);
            font-weight:700;
            cursor:pointer;
        " class="ui-button">
        ⚑ Пожаловаться
    </button>

    <div
        id="report-panel"
        style="
            display:none;
            margin-top:12px;
            padding:14px;
            border-radius:16px;
            border:1px solid rgba(217,119,6,.18);
            background:rgba(0,0,0,.03);
        ">

        <div style="
            font-size:13px;
            font-weight:800;
            margin-bottom:8px;
        ">
            Причина жалобы
        </div>

        <textarea
            id="report-reason"
            maxlength="500"
            rows="4"
            placeholder="Коротко опишите проблему..."
            style="
                width:100%;
                box-sizing:border-box;
                padding:12px 13px;
                border-radius:12px;
                border:1px solid var(--line);
                background:rgba(0,0,0,.04);
                color:var(--text);
                resize:vertical;
                font-size:14px;
            " class="ui-textarea"></textarea>

        <div style="
            display:flex;
            gap:8px;
            margin-top:10px;
            flex-wrap:wrap;
        ">

            <button
                id="report-submit"
                type="button"
                style="
                    min-height:40px;
                    padding:0 14px;
                    border-radius:12px;
                    border:1px solid rgba(220,38,38,.30);
                    background:rgba(220,38,38,.08);
                    color:var(--text);
                    font-weight:800;
                    cursor:pointer;
                " class="ui-button">
                Отправить жалобу
            </button>

            <button
                id="report-cancel"
                type="button"
                style="
                    min-height:40px;
                    padding:0 14px;
                    border-radius:12px;
                    border:1px solid var(--line);
                    background:rgba(0,0,0,.03);
                    color:var(--text);
                    font-weight:700;
                    cursor:pointer;
                " class="ui-button">
                Отмена
            </button>

        </div>

        <div
            id="report-status"
            style="
                margin-top:9px;
                font-size:12px;
                color:var(--muted);
            " class="ui-status">
        </div>

    </div>

    <div style="margin-top:18px;">
        <div style="
            font-size:12px;
            color:var(--muted);
            margin-bottom:8px;
            font-weight:700;
            letter-spacing:.05em;
        ">
            ОЦЕНИТЬ РЕСУРС
        </div>

        <div id="rating-stars" style="
            display:flex;
            gap:8px;
            align-items:center;
        ">
            <button type="button" data-score="1" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;" class="ui-button">☆</button>
            <button type="button" data-score="2" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;" class="ui-button">☆</button>
            <button type="button" data-score="3" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;" class="ui-button">☆</button>
            <button type="button" data-score="4" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;" class="ui-button">☆</button>
            <button type="button" data-score="5" style="font-size:28px;background:none;border:0;cursor:pointer;padding:2px;" class="ui-button">☆</button>
        </div>

        <div id="vote-status" style="
            margin-top:7px;
            font-size:12px;
            color:var(--muted);
        " class="ui-status"></div>
    </div>
</section>

<section
    class="card"
    style="
        {premium_style}
        display:block;
        width:100%;
        box-sizing:border-box;
        margin-bottom:16px;
        padding:20px;
        position:relative;
        overflow:hidden;
    "
>

    <div style="
        font-size:12px;
        color:var(--muted);
        margin-bottom:8px;
        text-transform:uppercase;
        letter-spacing:.08em;
        font-weight:700;
    ">
        О ресурсе
    </div>

    <div style="
        width:100%;
        min-width:0;
        box-sizing:border-box;
        font-size:16px;
        line-height:1.6;
        white-space:pre-wrap;
        overflow-wrap:anywhere;
        word-break:break-word;
    ">
        {description}
    </div>

</section>

{owner_profile_html}

<section
    class="card"
    style="
        display:block;
        width:100%;
        box-sizing:border-box;
        margin-bottom:16px;
        padding:20px;
    "
>

    <div style="
        font-size:12px;
        color:var(--muted);
        margin-bottom:12px;
        text-transform:uppercase;
        letter-spacing:.08em;
        font-weight:700;
    ">
        Контакты
    </div>

    <div class="card-meta"
         style="
             display:block;
             width:100%;
             margin-bottom:10px;
             line-height:1.5;
             overflow-wrap:anywhere;
         ">
        📍 {address}
    </div>

    <div class="card-meta"
         style="
             display:block;
             width:100%;
             line-height:1.5;
             overflow-wrap:anywhere;
         ">
        📞 {contact}
    </div>

    <div style="
        display:grid;
        grid-template-columns:repeat(2,minmax(0,1fr));
        width:100%;
        gap:10px;
        margin-top:18px;
    ">

        <a href="{contact_href}"
           style="
               display:flex;
               align-items:center;
               justify-content:center;
               gap:8px;
               min-width:0;
               min-height:48px;
               padding:0 10px;
               box-sizing:border-box;
               border-radius:14px;
               text-decoration:none;
               font-weight:800;
               color:var(--text);
               border:1px solid rgba(214,183,122,.38);
               background:rgba(214,183,122,.08);
           ">
            📞 Связаться
        </a>

        <a href="{map_href}"
           target="_blank"
           rel="noopener noreferrer"
           style="
               display:flex;
               align-items:center;
               justify-content:center;
               gap:8px;
               min-width:0;
               min-height:48px;
               padding:0 10px;
               box-sizing:border-box;
               border-radius:14px;
               text-decoration:none;
               font-weight:800;
               color:var(--text);
               border:1px solid var(--line);
               background:rgba(0,0,0,.035);
           ">
            📍 На карте
        </a>

    </div>

</section>

<section
    class="card"
    style="
        display:block;
        width:100%;
        box-sizing:border-box;
        margin-bottom:16px;
        padding:20px;
    "
>

    <div style="
        font-size:12px;
        color:var(--muted);
        margin-bottom:12px;
        text-transform:uppercase;
        letter-spacing:.08em;
        font-weight:700;
    ">
        ID ресурса
    </div>

    <div style="font-size:18px;font-weight:700;">
        #{id}
    </div>

</section>"####,
        description = safe_description,
        owner_profile_html = owner_profile_html,
        address = safe_address,
        contact = safe_contact,
        contact_href = safe_contact_href,
        map_href = safe_map_href,
        premium_style = premium_style,
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
    pub existing_status: Option<&'a str>,
}

pub fn render_resource_promotion(params: RenderResourcePromotionParams<'_>) -> String {
    let title = escape_html(params.title);
    let category = escape_html(params.category);
    let description = escape_html(params.description);
    let address = escape_html(params.address);
    let city_name = escape_html(params.city_name);
    let target_name = escape_html(params.target_name);

    let action_html = if params.existing_status.is_some() {
        r#"
<div class="card"
     style="
         display:block;
         padding:18px;
         margin-top:16px;
         border:1px solid rgba(214,183,122,.32);
         background:rgba(214,183,122,.07);
     ">
    <div class="card-title">
        Заявка ожидает подтверждения
    </div>
    <div class="card-meta"
         style="margin-top:6px;">
        Повторная заявка не требуется.
    </div>
</div>
"#
        .to_string()
    } else {
        format!(
            r#"
<form method="post"
      action="/app/resource/{resource_id}/promote/request"
      class="ui-form"
      style="margin-top:16px;">
    <input type="hidden"
           name="target_id"
           value="{target_id}">

    <button type="submit"
            class="ui-button"
            style="
                width:100%;
                min-height:52px;
                border-radius:15px;
                border:1px solid rgba(214,183,122,.55);
                background:
                    linear-gradient(
                        135deg,
                        rgba(214,183,122,.20),
                        rgba(214,183,122,.08)
                    );
                color:var(--text);
                font-weight:900;
                cursor:pointer;
            ">
        Отправить на модерацию
    </button>
</form>
"#,
            resource_id = params.resource_id,
            target_id = params.target_id,
        )
    };

    let content = format!(
        r#"
<section class="card"
         style="
             display:block;
             overflow:hidden;
             padding:0;
             border:1px solid rgba(214,183,122,.52);
             background:
                 radial-gradient(
                     circle at 100% 0%,
                     rgba(126,212,228,.15),
                     transparent 34%
                 ),
                 linear-gradient(
                     145deg,
                     rgba(20,23,30,.99),
                     rgba(10,12,17,.99)
                 );
             box-shadow:
                 0 22px 60px rgba(0,0,0,.30),
                 0 0 38px rgba(214,183,122,.08);
         ">

    <div style="
        padding:15px 20px;
        border-bottom:1px solid rgba(214,183,122,.24);
        color:var(--gold-light);
        font-size:12px;
        font-weight:900;
        letter-spacing:.12em;
        text-transform:uppercase;
    ">
        RESURSMAP · {city_name}
    </div>

    <div style="padding:22px 20px;">

        <div style="
            color:var(--muted);
            font-size:11px;
            font-weight:800;
            letter-spacing:.08em;
            text-transform:uppercase;
        ">
            {category}
        </div>

        <h2 style="
            margin:8px 0 12px;
            font-size:24px;
            line-height:1.18;
            overflow-wrap:anywhere;
        ">
            {title}
        </h2>

        <div style="
            color:var(--text);
            font-size:14px;
            line-height:1.55;
            white-space:pre-wrap;
            overflow-wrap:anywhere;
        ">
            {description}
        </div>

        <div style="
            margin-top:16px;
            color:var(--muted);
            font-size:13px;
            overflow-wrap:anywhere;
        ">
            📍 {address}
        </div>

        <div style="
            margin-top:20px;
            padding-top:15px;
            border-top:1px solid rgba(214,183,122,.18);
            color:var(--gold-light);
            font-size:12px;
            font-weight:850;
        ">
            ResursMap — люди, услуги и возможности рядом
        </div>

        <div style="
            margin-top:7px;
            color:var(--muted);
            font-size:11px;
        ">
            resursmap.de
        </div>

    </div>
</section>

<section class="card"
         style="
             display:block;
             padding:18px;
             margin-top:16px;
         ">
    <div class="card-title">
        Место публикации
    </div>

    <div class="card-meta"
         style="margin-top:7px;">
        {target_name} · {city_name}
    </div>

    <div class="card-meta"
         style="
             margin-top:10px;
             line-height:1.5;
         ">
        Перед публикацией заявка проверяется администратором.
        Контактные данные остаются на странице объявления.
    </div>
</section>

{action_html}
"#,
        city_name = city_name,
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
        empty_state_card(
            "Нет опубликованных ресурсов",
            "Добавьте ресурс в выбранном городе и категории.",
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
                    r#"<span style="font-size:11px;font-weight:800;color:var(--gold-light);">★ PREMIUM</span>"#
                } else {
                    ""
                };

                let moderation_badge = if *is_active == 0
                    && moderation_status != "rejected"
                {
                    r#"<span style="font-size:11px;font-weight:800;color:#dc2626;">⚫ Скрыт</span>"#
                } else {
                    match moderation_status.as_str() {
                        "approved" => {
                            r#"<span style="font-size:11px;font-weight:800;color:#16a34a;">🟢 Одобрен</span>"#
                        }
                        "rejected" => {
                            r#"<span style="font-size:11px;font-weight:800;color:#dc2626;">🔴 Отклонён</span>"#
                        }
                        _ => {
                            r#"<span style="font-size:11px;font-weight:800;color:#d97706;">🟡 На проверке</span>"#
                        }
                    }
                };

                let hidden_html =
                    if *is_active == 0
                        && moderation_status != "rejected"
                    {
                        r#"<div style="
                            margin-top:10px;
                            padding:10px 12px;
                            border-radius:12px;
                            border:1px solid rgba(220,38,38,.18);
                            background:rgba(220,38,38,.05);
                            color:#dc2626;
                            font-size:12px;
                            line-height:1.5;
                        "><strong>Ресурс скрыт.</strong> Публикация недоступна другим участникам.</div>"#
                            .to_string()
                    } else {
                        String::new()
                    };

                let promotion_button =
                    if moderation_status == "approved"
                        && *is_active == 1
                    {
                        format!(
                            r#"<a
                                href="/app/resource/{id}/promote"
                                style="
                                    display:inline-flex;
                                    align-items:center;
                                    justify-content:center;
                                    min-height:42px;
                                    padding:0 14px;
                                    border-radius:12px;
                                    text-decoration:none;
                                    font-size:13px;
                                    font-weight:850;
                                    color:var(--gold-light);
                                    border:1px solid rgba(214,183,122,.48);
                                    background:rgba(214,183,122,.10);
                                ">
                                Продвинуть
                            </a>"#,
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
                            r#"<div style="
                                margin-top:10px;
                                padding:10px 12px;
                                border-radius:12px;
                                border:1px solid rgba(220,38,38,.22);
                                background:rgba(220,38,38,.07);
                                color:#dc2626;
                                font-size:12px;
                                line-height:1.5;
                            "><strong>Причина отказа:</strong> {}</div>"#,
                            safe_rejection_reason
                        )
                    } else {
                        String::new()
                    };

                format!(
                    r#"
                    <article class="card"
                       style="
                           display:block;
                           margin-bottom:16px;
                           padding:18px;
                       ">

                        <div style="
                            display:flex;
                            align-items:flex-start;
                            gap:14px;
                        ">

                            <div class="card-icon"
                                 style="flex:0 0 auto;">
                                {icon}
                            </div>

                            <div style="
                                flex:1;
                                min-width:0;
                            ">

                                <div style="
                                    display:flex;
                                    justify-content:space-between;
                                    gap:12px;
                                    align-items:flex-start;
                                ">
                                    <div style="min-width:0;">
                                        <div class="card-title"
                                             style="
                                                 font-size:18px;
                                                 line-height:1.25;
                                                 margin-bottom:4px;
                                             ">
                                            {title}
                                        </div>

                                        <div class="card-meta"
                                             style="
                                                 font-size:12px;
                                                 text-transform:uppercase;
                                                 letter-spacing:.06em;
                                             ">
                                            {category}
                                        </div>
                                    </div>

                                    <div style="
                                        flex:0 0 auto;
                                        font-size:12px;
                                        color:var(--muted);
                                        white-space:nowrap;
                                    ">
                                        ⭐ {rating:.1} · {votes}
                                    </div>
                                </div>

                                <div class="card-meta"
                                     style="
                                         margin-top:10px;
                                         line-height:1.5;
                                         overflow-wrap:anywhere;
                                     ">
                                    {description}
                                </div>

                                <div style="
                                    display:flex;
                                    gap:8px;
                                    flex-wrap:wrap;
                                    align-items:center;
                                    margin-top:12px;
                                ">
                                    {premium_badge}
                                    {moderation_badge}
                                </div>

                                {rejection_html}
                                {hidden_html}

                                <div style="
                                    display:flex;
                                    gap:10px;
                                    flex-wrap:wrap;
                                    margin-top:14px;
                                ">

                                    <a
                                        href="/app/resource/{id}/edit"
                                        style="
                                            display:inline-flex;
                                            align-items:center;
                                            justify-content:center;
                                            min-height:42px;
                                            padding:0 14px;
                                            border-radius:12px;
                                            text-decoration:none;
                                            font-size:13px;
                                            font-weight:800;
                                            color:var(--text);
                                            border:1px solid rgba(214,183,122,.35);
                                            background:rgba(214,183,122,.08);
                                        "
                                    >
                                        ✎ Редактировать
                                    </a>

                                    {promotion_button}

                                    <a
                                        href="/app/resource/{id}"
                                        style="
                                            display:inline-flex;
                                            align-items:center;
                                            justify-content:center;
                                            min-height:42px;
                                            padding:0 14px;
                                            border-radius:12px;
                                            text-decoration:none;
                                            font-size:13px;
                                            font-weight:700;
                                            color:var(--text);
                                            border:1px solid var(--line);
                                            background:rgba(0,0,0,.03);
                                        "
                                    >
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
      style="display:flex;flex-direction:column;gap:16px;" class="ui-form">

    <label>
        <div style="margin-bottom:7px;font-weight:600;">Название</div>
        <input
            name="title"
            required
            maxlength="120"
            value="{title}"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;" class="ui-input">
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">Описание</div>
        <textarea
            name="description"
            required
            maxlength="1000"
            rows="6"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;
                   box-sizing:border-box;resize:vertical;" class="ui-textarea">{description}</textarea>
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">Телефон или Telegram</div>
        <input
            name="contact"
            maxlength="120"
            value="{contact}"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;" class="ui-input">
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">Адрес</div>
        <input
            name="address"
            maxlength="250"
            value="{address}"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;" class="ui-input">
    </label>

    <button type="submit"
            style="min-height:52px;border:0;border-radius:16px;
                   font-size:16px;font-weight:800;cursor:pointer;
                   background:linear-gradient(135deg,var(--gold),var(--gold-light));
                   color:#111;" class="ui-button">
        Сохранить изменения
    </button>

    <div style="font-size:12px;color:var(--muted);line-height:1.5;">
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

    let content = format!(
        r####"<form method="post"
      action="/app/{}/{}/{}/cat/{}/add"
      style="display:flex;flex-direction:column;gap:16px;" class="ui-form">

    <label>
        <div style="margin-bottom:7px;font-weight:600;">
            Название
        </div>

        <input
            name="title"
            required
            maxlength="120"
            placeholder="Например: Охранная компания"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;" class="ui-input">
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">
            Описание
        </div>

        <textarea
            name="description"
            required
            maxlength="1000"
            rows="5"
            placeholder="Расскажите о ресурсе..."
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;
                   box-sizing:border-box;resize:vertical;" class="ui-textarea"></textarea>
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">
            Телефон или Telegram
        </div>

        <input
            name="contact"
            maxlength="120"
            placeholder="+33... или @username"
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;" class="ui-input">
    </label>

    <label>
        <div style="margin-bottom:7px;font-weight:600;">
            Адрес
        </div>

        <input
            name="address"
            maxlength="200"
            placeholder="Город, улица..."
            style="width:100%;padding:15px;border-radius:14px;
                   border:1px solid #ddd;font-size:16px;box-sizing:border-box;" class="ui-input">
    </label>

    <button
        type="submit"
        style="margin-top:8px;padding:16px;border:0;border-radius:16px;
               font-size:17px;font-weight:700;cursor:pointer;" class="ui-button">
        ➕ Добавить ресурс
    </button>

</form>"####,
        ci, si, zi, category,
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
