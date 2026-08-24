use super::common::{base_style, bottom_nav, icon, topbar};
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
            cards.push_str(&format!(
                r#"
<a class="card" href="/app/{ci}/{si}">
    <div class="card-icon">{building}</div>

    <div class="card-content">
        <div class="card-title">{country}</div>
        <div class="card-meta">{continent} · страна</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                building = icon("building"),
                arrow = icon("chevron"),
            ));
        }
    }

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

<section class="hero">
    <div class="eyebrow">
        <span class="eyebrow-dot"></span>
        ResursMap
    </div>

    <h1>Карта ресурсов</h1>

    <p>
        Люди, города, услуги и возможности —
        всё необходимое рядом с вами.
    </p>

    <div class="search">
        {search_icon}
        <input
            type="text"
            placeholder="Найти город, услугу или ресурс..."
            onkeydown="if(event.key==='Enter') window.location='/app/search?q='+encodeURIComponent(this.value)"
        >
    </div>
</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Страны</h2>
        <p class="section-caption">Выберите страну для продолжения</p>
    </div>
</div>

<div class="grid">
    {cards}
</div>

<div class="section-head">
    <div>
        <h2 class="section-title">Возможности</h2>
        <p class="section-caption">Всё необходимое в одном месте</p>
    </div>
</div>

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
        search_icon = icon("search"),
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
            cards.push_str(&format!(
                r#"
<a class="card" href="/app/{ci}/{si}">
    <div class="card-icon">
        {icon}
    </div>

    <div class="card-content">
        <div class="card-title">{country}</div>
        <div class="card-meta">Открыть города</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                icon = icon("building"),
                arrow = icon("chevron"),
            ));
        }

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

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">
    <div class="eyebrow">
        {}
        Регион
    </div>

    <h1>{name}</h1>

    <p>Выберите страну, чтобы открыть доступные города и ресурсы.</p>
</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Страны</h2>
        <p class="section-caption">{count} доступных</p>
    </div>
</div>

<div class="grid">
    {cards}
</div>

</main>

{bottom_nav}

</body>
</html>"#,
            base_style(),
            icon("globe"),
            icon("map"),
            bottom_nav = bottom_nav("map"),
            count = countries.len(),
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
                cards.push_str(&format!(
                    r#"
<a class="card" href="/app/{ci}/{si}/{zi}">
    <div class="card-icon">
        {pin}
    </div>

    <div class="card-content">
        <div class="card-title">{city}</div>
        <div class="card-meta">{country}</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                    pin = icon("map-pin"),
                    arrow = icon("chevron"),
                ));
            }

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

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">
    <div class="eyebrow">
        {}
        {cname}
    </div>

    <h1>{country}</h1>

    <p>Выберите город и откройте его карту ресурсов.</p>
</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Города</h2>
        <p class="section-caption">{count} городов</p>
    </div>
</div>

<div class="grid">
    {cards}
</div>

</main>

{bottom_nav}

</body>
</html>"#,
                base_style(),
                icon("globe"),
                icon("map-pin"),
                bottom_nav = bottom_nav("map"),
                count = cities.len(),
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

<header class="topbar">
    <a class="brand" href="/app">
        <div class="brand-mark">{}</div>
        <div>
            <div class="brand-name">RESURSMAP</div>
            <div class="brand-sub">RESOURCE NETWORK</div>
        </div>
    </a>
</header>

<section class="hero">

    <div class="eyebrow">
        {}
        {country}
    </div>

    <h1>{city}</h1>

    <p>
        {cname} · {country}
        <br>
        Карта ресурсов города.
    </p>

</section>

<div class="section-head">
    <div>
        <h2 class="section-title">Разделы</h2>
        <p class="section-caption">Выберите направление</p>
    </div>
</div>

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
                    icon("globe"),
                    icon("map-pin"),
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
    resources: Vec<(
        i64,
        String,
        String,
        String,
        String,
        f64,
        i64,
        i64,
        i64,
        usize,
        usize,
        usize,
    )>,
    people: Vec<(String, String, String, String, i64, String, i64)>,
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

                    location_results.push_str(&format!(
                        r#"
<a href="/app/{ci}/{si}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:12px;
   ">
    <div class="card-icon">{icon}</div>

    <div class="card-content">
        <div class="card-title">{country}</div>
        <div class="card-meta">{continent} · страна</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                        ci = ci,
                        si = si,
                        icon = icon("building"),
                        country = country,
                        continent = continent,
                        arrow = icon("chevron"),
                    ));
                }

                for (zi, city) in cities.iter().enumerate() {
                    if city.to_lowercase().contains(&query_lower) {
                        location_count += 1;

                        location_results.push_str(&format!(
                            r#"
<a href="/app/{ci}/{si}/{zi}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:12px;
   ">
    <div class="card-icon">{icon}</div>

    <div class="card-content">
        <div class="card-title">{city}</div>
        <div class="card-meta">{country} · город</div>
    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                            ci = ci,
                            si = si,
                            zi = zi,
                            icon = icon("map-pin"),
                            city = city,
                            country = country,
                            arrow = icon("chevron"),
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

                    format!(
                        r#"
<a href="/app/user/{public_id}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:14px;
       align-items:flex-start;
   ">

    <div class="card-icon">
        {user_icon}
    </div>

    <div class="card-content"
         style="min-width:0;">

        <div class="card-title"
             style="
                 font-size:17px;
                 line-height:1.3;
                 overflow-wrap:anywhere;
             ">
            {display_name}
        </div>

        {username_html}

        {intent_html}

        <div style="
            display:flex;
            gap:8px;
            flex-wrap:wrap;
            margin-top:10px;
        ">
            {contact_html}
        </div>

    </div>

    <div class="card-arrow">
        {arrow}
    </div>

</a>
"#,
                        public_id = public_id,
                        user_icon = icon("user"),
                        display_name = display_name,
                        username_html = username_html,
                        intent_html = intent_html,
                        contact_html = contact_html,
                        arrow = icon("chevron"),
                    )
                },
            )
            .collect::<Vec<_>>()
            .join("");

        format!(
            r#"
<div class="section-head"
     style="margin-top:24px;">
    <div>
        <h2 class="section-title">Участники</h2>
        <p class="section-caption">
            Найдено: {people_count}
        </p>
    </div>
</div>

<section>
    {people_cards}
</section>
"#,
            people_count = people_count,
            people_cards = people_cards,
        )
    };

    let result_count = resources.len();

    let results = if q.trim().is_empty() {
        r#"
        <div class="card" style="display:block;margin-top:18px;">
            <div class="card-content">
                <div class="card-title">Начните поиск</div>
                <div class="card-meta" style="margin-top:5px;">
                    Введите название, услугу, категорию или адрес.
                </div>
            </div>
        </div>
        "#
        .to_string()
    } else if resources.is_empty() && people.is_empty() && location_results.is_empty() {
        format!(
            r#"
        <div class="card" style="display:block;margin-top:18px;">
            <div class="card-content">
                <div class="card-title">Ничего не найдено</div>
                <div class="card-meta" style="margin-top:5px;">
                    По запросу «{}» пока ничего не найдено.
                </div>
            </div>
        </div>
        "#,
            q
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

                    format!(
                        r#"
<a href="/app/resource/{id}"
   class="card"
   style="
       text-decoration:none;
       color:inherit;
       margin-bottom:14px;
       align-items:flex-start;
   ">

    <div class="card-icon">{resource_icon}</div>

    <div class="card-content" style="min-width:0;">

        <div style="
            display:flex;
            justify-content:space-between;
            align-items:flex-start;
            gap:10px;
        ">
            <div class="card-title"
                 style="font-size:17px;min-width:0;">
                {title}
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
             style="margin-top:5px;">
            {category}
        </div>

        <div class="card-meta"
             style="
                 margin-top:8px;
                 line-height:1.45;
                 overflow-wrap:anywhere;
             ">
            {description}
        </div>

        <div class="card-meta"
             style="margin-top:9px;">
            📍 {location}
        </div>

        <div class="card-meta"
             style="
                 margin-top:4px;
                 overflow-wrap:anywhere;
             ">
            {address}
        </div>

        <div style="
            display:flex;
            gap:8px;
            align-items:center;
            flex-wrap:wrap;
            margin-top:10px;
        ">
            {premium_badge}
            {verified_badge}
        </div>

    </div>

    <div class="card-arrow">{arrow}</div>
</a>
"#,
                        id = id,
                        resource_icon = icon("map-pin"),
                        arrow = icon("chevron"),
                        title = title,
                        category = category,
                        description = description,
                        rating = rating,
                        votes = votes,
                        location = location,
                        address = address,
                        premium_badge = premium_badge,
                        verified_badge = verified_badge,
                    )
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
<div class="section-head"
     style="margin-top:24px;">
    <div>
        <h2 class="section-title">Места</h2>
        <p class="section-caption">
            Найдено: {location_count}
        </p>
    </div>
</div>

<section>
    {location_results}
</section>
"#,
            location_count = location_count,
            location_results = location_results,
        )
    };

    let result_header = if q.trim().is_empty() || resources.is_empty() {
        String::new()
    } else {
        format!(
            r#"
<div class="section-head"
     style="margin-top:24px;">
    <div>
        <h2 class="section-title">Результаты</h2>
        <p class="section-caption">
            Найдено: {}
        </p>
    </div>
</div>
"#,
            result_count
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

<section class="hero">

    <div class="eyebrow">
        {search_icon}
        Поиск
    </div>

    <h1>Найти ресурс</h1>

    <p>
        Ищите услуги, компании, специалистов и другие ресурсы.
    </p>

    <form method="get"
          action="/app/search"
          class="search">
        {search_input_icon}

        <input
            name="q"
            autofocus
            value="{q}"
            placeholder="Например: охранник, бизнес, улица..."
        >
    </form>

</section>

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
        search_icon = icon("search"),
        search_input_icon = icon("search"),
        bottom_nav = bottom_nav("search"),
        q = q,
        location_section = location_section,
        people_section = people_section,
        result_header = result_header,
        results = results,
    )
}

// ============================================================
// CONTACT REQUESTS
// ============================================================
