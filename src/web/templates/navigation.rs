use super::common::{
    base_style, bottom_nav, empty_state_card, escape_html, icon, navigation_card,
    people_result_card, resource_result_card, search_form_hero, search_hero, section_head,
    simple_hero, topbar,
};
use std::collections::BTreeMap;

pub fn world() -> BTreeMap<&'static str, BTreeMap<&'static str, Vec<&'static str>>> {
    let mut w = BTreeMap::new();

    let mut eu = BTreeMap::new();

    eu.insert(
        "Франция",
        vec!["Париж", "Марсель", "Лион", "Тулуза", "Ницца"],
    );

    eu.insert("Германия", vec!["Берлин", "Гамбург", "Мюнхен", "Кёльн"]);

    eu.insert("Италия", vec!["Рим", "Милан", "Неаполь", "Турин"]);

    w.insert("Европа", eu);

    w
}

// ============================================================
// LUCIDE SVG
// ============================================================

pub fn render_continents() -> String {
    let w = world();
    let mut cards = String::new();

    for (ci, (continent, countries)) in w.iter().enumerate() {
        for (si, country) in countries.keys().enumerate() {
            cards.push_str(&navigation_card(
                &format!("/app/{}/{}", ci, si),
                "building",
                country,
                &format!("{} · страна", continent),
            ));
        }
    }

    let section_head_countries = section_head("Страны", "Выберите страну для продолжения", None);
    let section_head_features = section_head("Возможности", "Всё необходимое в одном месте", None);

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>ResursMap</title>
<style>{style}</style>
</head>

<body>
<main class="page">

{topbar}

{hero}

{section_head_countries}

<div class="grid">
    {cards}
</div>

{section_head_features}

<div class="feature-grid">
    <div class="feature">
        {map_icon}
        <strong>Ресурсы</strong>
        <span>Находите полезные места и услуги.</span>
    </div>

    <div class="feature">
        {heart_icon}
        <strong>Сообщество</strong>
        <span>Люди и возможности вашего города.</span>
    </div>

    <div class="feature">
        {user_icon}
        <strong>Профиль</strong>
        <span>Ваши данные, голоса и активность.</span>
    </div>
</div>

</main>

{bottom_nav}

</body>
</html>"#,
        style = base_style(),
        topbar = topbar("RESOURCE NETWORK", "globe"),
        hero = search_hero(
            "ResursMap",
            "Карта ресурсов",
            "Люди, города, услуги и возможности — всё необходимое рядом с вами.",
            "Найти город, услугу или ресурс...",
        ),
        cards = cards,
        map_icon = icon("map"),
        heart_icon = icon("heart"),
        user_icon = icon("user"),
        bottom_nav = bottom_nav("map"),
    )
}

// ============================================================
// CONTINENT
// ============================================================

pub fn render_continent(ci: usize) -> String {
    let w = world();

    if let Some((name, countries)) = w.iter().nth(ci) {
        let mut cards = String::new();

        for (si, country) in countries.keys().enumerate() {
            cards.push_str(&navigation_card(
                &format!("/app/{}/{}", ci, si),
                "building",
                country,
                "Открыть города",
            ));
        }

        let section_head_countries =
            section_head("Страны", &format!("{} доступных", countries.len()), None);

        return format!(
            r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{name} · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

{}

{}

{section_head_countries}

<div class="grid">
    {cards}
</div>

</main>

{bottom_nav}

</body>
</html>"#,
            base_style(),
            topbar("RESOURCE NETWORK", "globe"),
            simple_hero(
                "map",
                "Регион",
                name,
                "Выберите страну, чтобы открыть доступные города и ресурсы.",
            ),
            bottom_nav = bottom_nav("map"),
        );
    }

    render_continents()
}

// ============================================================
// COUNTRY
// ============================================================

pub fn render_country(ci: usize, si: usize) -> String {
    let w = world();

    if let Some((cname, countries)) = w.iter().nth(ci) {
        if let Some((country, cities)) = countries.iter().nth(si) {
            let mut cards = String::new();

            for (zi, city) in cities.iter().enumerate() {
                cards.push_str(&navigation_card(
                    &format!("/app/{}/{}/{}", ci, si, zi),
                    "map-pin",
                    city,
                    country,
                ));
            }

            let section_head_cities =
                section_head("Города", &format!("{} городов", cities.len()), None);

            return format!(
                r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{country} · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

{}

{}

{section_head_cities}

<div class="grid">
    {cards}
</div>

</main>

{bottom_nav}

</body>
</html>"#,
                base_style(),
                topbar("RESOURCE NETWORK", "globe"),
                simple_hero(
                    "map-pin",
                    cname,
                    country,
                    "Выберите город и откройте его карту ресурсов.",
                ),
                bottom_nav = bottom_nav("map"),
            );
        }
    }

    render_continents()
}

// ============================================================
// CITY
// ============================================================

pub fn render_city(ci: usize, si: usize, zi: usize) -> String {
    let w = world();

    if let Some((cname, countries)) = w.iter().nth(ci) {
        if let Some((country, cities)) = countries.iter().nth(si) {
            if let Some(city) = cities.get(zi) {
                let section_head_sections = section_head("Разделы", "Выберите направление", None);

                return format!(
                    r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{city} · ResursMap</title>
<style>{}</style>
</head>

<body>

<main class="page">

{}

{}

{section_head_sections}

<div class="grid">

    <a class="card" href="/app/{ci}/{si}/{zi}/cat/work">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Работа</div>
            <div class="card-meta">Вакансии и предложения</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

    <a class="card" href="/app/{ci}/{si}/{zi}/cat/business">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Бизнес</div>
            <div class="card-meta">Компании и услуги</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

    <a class="card" href="/app/{ci}/{si}/{zi}/cat/services">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Услуги</div>
            <div class="card-meta">Помощь и специалисты</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

    <a class="card" href="/app/{ci}/{si}/{zi}/cat/community">
        <div class="card-icon">{}</div>
        <div class="card-content">
            <div class="card-title">Сообщество</div>
            <div class="card-meta">Люди и контакты</div>
        </div>
        <div class="card-arrow">{}</div>
    </a>

</div>

</main>

{bottom_nav}

</body>
</html>"#,
                    base_style(),
                    topbar("RESOURCE NETWORK", "globe"),
                    simple_hero(
                        "map-pin",
                        country,
                        city,
                        &format!(
                            "{} · {}<br>Карта ресурсов города.",
                            escape_html(cname),
                            escape_html(country),
                        ),
                    ),
                    icon("briefcase"),
                    icon("chevron"),
                    icon("building"),
                    icon("chevron"),
                    icon("map"),
                    icon("chevron"),
                    icon("user"),
                    icon("chevron"),
                    bottom_nav = bottom_nav("map"),
                );
            }
        }
    }

    render_continents()
}

// ============================================================
// SEARCH
// ============================================================

pub fn render_search(
    q: &str,
    resources: Vec<crate::web::view_models::SearchResourceRow>,
    people: Vec<crate::web::view_models::SearchPersonRow>,
) -> String {
    let world_data = world();

    let query_lower = q.trim().to_lowercase();

    let mut location_results = String::new();
    let mut location_count = 0usize;

    if !query_lower.is_empty() {
        for (ci, (continent, countries)) in world_data.iter().enumerate() {
            for (si, (country, cities)) in countries.iter().enumerate() {
                let country_match = country.to_lowercase().contains(&query_lower);

                if country_match {
                    location_count += 1;

                    location_results.push_str(&navigation_card(
                        &format!("/app/{}/{}", ci, si),
                        "building",
                        country,
                        &format!("{} · страна", continent),
                    ));
                }

                for (zi, city) in cities.iter().enumerate() {
                    if city.to_lowercase().contains(&query_lower) {
                        location_count += 1;

                        location_results.push_str(&navigation_card(
                            &format!("/app/{}/{}/{}", ci, si, zi),
                            "map-pin",
                            city,
                            &format!("{} · город", country),
                        ));
                    }
                }
            }
        }
    }

    let people_count = people.len();

    let people_section = if q.trim().is_empty() || people.is_empty() {
        String::new()
    } else {
        let people_cards = people
            .iter()
            .map(
                |(
                    public_id,
                    username,
                    first_name,
                    last_name,
                    open_contact,
                    intent_text,
                    intent_until,
                )| {
                    let escape_html = |value: &str| -> String {
                        value
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;")
                            .replace('\'', "&#39;")
                    };

                    let safe_username = escape_html(username);
                    let safe_first_name = escape_html(first_name);
                    let safe_last_name = escape_html(last_name);
                    let safe_intent = escape_html(intent_text);

                    let full_name = format!("{} {}", safe_first_name, safe_last_name)
                        .trim()
                        .to_string();

                    let display_name = if !full_name.is_empty() {
                        full_name
                    } else if !safe_username.is_empty() {
                        format!("@{}", safe_username)
                    } else {
                        "Участник ResursMap".to_string()
                    };

                    let username_html = if !safe_username.is_empty()
                        && display_name != format!("@{}", safe_username)
                    {
                        format!(
                            r#"<div class="card-meta"
                                     style="margin-top:4px;">
                                    @{username}
                                </div>"#,
                            username = safe_username
                        )
                    } else {
                        String::new()
                    };

                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);

                    let intent_is_active = !safe_intent.trim().is_empty()
                        && (*intent_until == 0 || *intent_until >= now);

                    let intent_html = if intent_is_active {
                        format!(
                            r#"
<div style="
    margin-top:10px;
    padding:10px 12px;
    border-radius:12px;
    border:1px solid rgba(214,183,122,.20);
    background:rgba(214,183,122,.07);
    font-size:13px;
    line-height:1.45;
    overflow-wrap:anywhere;
">
    {intent}
</div>
"#,
                            intent = safe_intent
                        )
                    } else {
                        String::new()
                    };

                    let contact_html = if *open_contact != 0 {
                        r#"<span style="
                                font-size:10px;
                                font-weight:850;
                                color:#16a34a;
                            ">● Контакт открыт</span>"#
                    } else {
                        r#"<span style="
                                font-size:10px;
                                font-weight:750;
                                color:var(--muted);
                            ">Контакт закрыт</span>"#
                    };

                    people_result_card(
                        &format!("/app/user/{}", public_id),
                        &display_name,
                        &username_html,
                        &intent_html,
                        contact_html,
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("");

        format!(
            r#"
{people_head}

<section>
    {people_cards}
</section>
"#,
            people_head =
                section_head("Участники", &format!("Найдено: {}", people_count), Some(24),),
            people_cards = people_cards,
        )
    };

    let result_count = resources.len();

    let results = if q.trim().is_empty() {
        empty_state_card(
            "Начните поиск",
            "Введите название, услугу, категорию или адрес.",
        )
    } else if resources.is_empty() && people.is_empty() && location_results.is_empty() {
        empty_state_card(
            "Ничего не найдено",
            &format!("По запросу «{}» пока ничего не найдено.", escape_html(q),),
        )
    } else if resources.is_empty() {
        String::new()
    } else {
        resources
            .iter()
            .map(
                |(
                    id,
                    title,
                    category,
                    description,
                    address,
                    rating,
                    votes,
                    verified,
                    premium,
                    ci,
                    si,
                    zi,
                )| {
                    let location = world_data
                        .iter()
                        .nth(*ci)
                        .and_then(|(_, countries)| {
                            countries.iter().nth(*si).and_then(|(country, cities)| {
                                cities
                                    .get(*zi)
                                    .map(|city| format!("{} · {}", city, country))
                            })
                        })
                        .unwrap_or_else(|| "Местоположение не указано".to_string());

                    let premium_badge = if *premium != 0 {
                        r#"<span style="
                            display:inline-flex;
                            align-items:center;
                            padding:4px 8px;
                            border-radius:999px;
                            border:1px solid rgba(214,183,122,.38);
                            background:rgba(214,183,122,.10);
                            color:#d6b77a;
                            font-size:10px;
                            font-weight:800;
                        ">★ PREMIUM</span>"#
                    } else {
                        ""
                    };

                    let verified_badge = if *verified != 0 {
                        r#"<span style="
                            color:#16a34a;
                            font-size:11px;
                            font-weight:800;
                        ">✓ Проверен</span>"#
                    } else {
                        ""
                    };

                    resource_result_card(crate::web::templates::common::ResourceResultCardParams {
                        href: &format!("/app/resource/{}", id),
                        title_html: title,
                        category_html: category,
                        description_html: description,
                        rating: *rating,
                        votes: *votes,
                        location_html: &location,
                        address_html: address,
                        premium_badge_html: premium_badge,
                        verified_badge_html: verified_badge,
                    })
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let location_section = if q.trim().is_empty() || location_results.is_empty() {
        String::new()
    } else {
        format!(
            r#"
{location_head}

<section>
    {location_results}
</section>
"#,
            location_head =
                section_head("Места", &format!("Найдено: {}", location_count), Some(24),),
            location_results = location_results,
        )
    };

    let result_header = if q.trim().is_empty() || resources.is_empty() {
        String::new()
    } else {
        section_head(
            "Результаты",
            &format!("Найдено: {}", result_count),
            Some(24),
        )
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Поиск · ResursMap</title>
<style>{style}</style>
</head>

<body>

<main class="page">

{topbar}

{hero}

{location_section}

{people_section}

{result_header}

<section>
    {results}
</section>

</main>

{bottom_nav}

</body>
</html>"#,
        style = base_style(),
        topbar = topbar("SEARCH", "search"),
        hero = search_form_hero(
            "Поиск",
            "Найти ресурс",
            "Ищите услуги, компании, специалистов и другие ресурсы.",
            q,
            "Например: охранник, бизнес, улица...",
        ),
        bottom_nav = bottom_nav("search"),
        location_section = location_section,
        people_section = people_section,
        result_header = result_header,
        results = results,
    )
}

// ============================================================
// CONTACT REQUESTS
// ============================================================
