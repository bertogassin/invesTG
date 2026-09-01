use super::common::{
    bottom_nav, empty_state_card, escape_html, guest_mode_hint, icon, intent_kind_chips,
    navigation_card, page_document, page_shell, premium_badge_html, profession_label,
    resource_listing_label, resource_result_card, search_form_hero, search_people_cards,
    section_head, simple_hero, static_asset, topbar, verified_badge_html,
};
use crate::geography::world;
use std::collections::BTreeMap;

fn is_intent_category(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "work" | "job" | "jobs" | "business" | "services" | "service" | "community"
    )
}

fn json_string_literal(value: &str) -> String {
    let mut out = String::from('"');

    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }

    out.push('"');
    out
}

fn push_explore_entry(
    parts: &mut Vec<String>,
    kind: &str,
    label: &str,
    subtitle: &str,
    href: &str,
) {
    parts.push(format!(
        "{{\"k\":{},\"l\":{},\"s\":{},\"h\":{},\"q\":{}}}",
        json_string_literal(kind),
        json_string_literal(label),
        json_string_literal(subtitle),
        json_string_literal(href),
        json_string_literal(label),
    ));
}

fn build_home_explore_index(
    resource_categories: &[(String, i64)],
    people_categories: &[(String, i64)],
) -> String {
    let mut parts = Vec::new();
    let world_data = world();

    for (ci, (continent, countries)) in world_data.iter().enumerate() {
        push_explore_entry(
            &mut parts,
            "continent",
            continent,
            "Континент",
            &format!("/app/{ci}"),
        );

        for (si, (country, cities)) in countries.iter().enumerate() {
            push_explore_entry(
                &mut parts,
                "country",
                country,
                &format!("{continent} · страна"),
                &format!("/app/{ci}/{si}"),
            );

            for (zi, city) in cities.iter().enumerate() {
                push_explore_entry(
                    &mut parts,
                    "city",
                    city,
                    &format!("{country} · город"),
                    &format!("/app/{ci}/{si}/{zi}"),
                );
            }
        }
    }

    let mut professions: BTreeMap<String, i64> = BTreeMap::new();

    for (category, count) in resource_categories {
        let key = category.trim().to_string();

        if key.is_empty() || is_intent_category(&key) {
            continue;
        }

        *professions.entry(key).or_insert(0) += count;
    }

    for (category, count) in people_categories {
        let key = category.trim().to_string();

        if key.is_empty() || is_intent_category(&key) {
            continue;
        }

        *professions.entry(key).or_insert(0) += count;
    }

    push_explore_entry(
        &mut parts,
        "work",
        "Работа",
        "Вакансии и предложения работы",
        "/app/search?kind=work",
    );
    push_explore_entry(
        &mut parts,
        "workers",
        "Работники",
        "Профессии и объявления тех, кто ищет работу",
        "/app/search?kind=workers",
    );
    push_explore_entry(
        &mut parts,
        "business",
        "Бизнес",
        "Компании, услуги и специалисты",
        "/app/search?kind=business",
    );

    for (category, count) in professions {
        let label = profession_label(&category);
        let subtitle = if count > 0 {
            format!("Профессия · {count}")
        } else {
            "Профессия".to_string()
        };
        let href = format!("/app/search?q={}", urlencoding::encode(category.trim()));

        parts.push(format!(
            "{{\"k\":{},\"l\":{},\"s\":{},\"h\":{},\"q\":{}}}",
            json_string_literal("profession"),
            json_string_literal(&label),
            json_string_literal(&subtitle),
            json_string_literal(&href),
            json_string_literal(&format!("{} {}", category, label)),
        ));
    }

    format!("[{}]", parts.join(","))
}

// ============================================================
// LUCIDE SVG
// ============================================================

pub fn render_continents(
    users_count: i64,
    online_count: i64,
    resources_count: i64,
    _categories: Vec<(String, i64)>,
    people_by_category: Vec<(String, i64)>,
    guest_mode: bool,
) -> String {
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

    let head_extra = r####"<style id="resursmap-home-layout-v1">
    .rm-home-section {
        margin-top:18px;
    }

    .rm-guest-hint {
        margin-top:12px;
        padding:10px 12px;
        border-radius:14px;
        border:1px solid rgba(224,196,138,.24);
        background:linear-gradient(135deg, rgba(224,196,138,.10), rgba(114,196,212,.06));
        color:var(--muted);
        font-size:13px;
        line-height:1.5;
        box-shadow:0 0 24px rgba(224,196,138,.06);
    }

    .rm-stats-row {
        display:grid;
        grid-template-columns:repeat(3,minmax(0,1fr));
        gap:10px;
        margin-top:22px;
        position:relative;
        z-index:2;
    }

    .rm-stat {
        min-width:0;
        padding:14px 10px;
        border-radius:16px;
        border:1px solid rgba(232,204,150,.24);
        background:
            linear-gradient(145deg, rgba(232,204,150,.12), rgba(126,212,228,.06));
        box-shadow:
            inset 0 1px 0 rgba(255,255,255,.07),
            0 10px 28px rgba(0,0,0,.20);
        text-align:center;
    }

    .rm-stat strong {
        display:block;
        font-size:clamp(18px,4vw,24px);
        font-weight:850;
        letter-spacing:-.03em;
        color:var(--gold-light);
        line-height:1;
    }

    .rm-stat-online strong {
        color:var(--success);
        text-shadow:0 0 20px rgba(111,232,184,.35);
    }

    .rm-stat span {
        display:block;
        margin-top:6px;
        color:var(--muted);
        font-size:10px;
        font-weight:700;
        letter-spacing:.04em;
        text-transform:uppercase;
    }

    .rm-home-explorer {
        margin-top:20px;
        padding:16px 16px 14px;
        border:1px solid rgba(232,204,150,.28);
        background:
            radial-gradient(circle at 100% 0%, rgba(126,212,228,.10), transparent 42%),
            radial-gradient(circle at 0% 100%, rgba(232,204,150,.08), transparent 40%),
            rgba(255,255,255,.02);
        box-shadow:
            0 18px 44px rgba(0,0,0,.24),
            inset 0 1px 0 rgba(255,255,255,.06);
    }

    .rm-home-explorer-head {
        margin-bottom:12px;
    }

    .rm-home-explorer-title {
        font-size:16px;
        line-height:1.25;
    }

    .rm-home-explorer-copy {
        margin-top:4px;
        color:var(--muted);
        font-size:12px;
        line-height:1.45;
    }

    .rm-home-explorer-field {
        display:grid;
        grid-template-columns:auto 1fr auto;
        align-items:center;
        gap:10px;
        padding:4px 4px 4px 14px;
        border-radius:16px;
        border:1px solid rgba(232,204,150,.32);
        background:rgba(0,0,0,.18);
        transition:
            border-color .18s ease,
            box-shadow .18s ease;
    }

    .rm-home-explorer-field:focus-within {
        border-color:rgba(232,204,150,.55);
        box-shadow:0 0 0 4px rgba(232,204,150,.10);
    }

    .rm-home-explorer-icon {
        color:var(--gold-light);
        font-size:16px;
        line-height:1;
    }

    .rm-home-explorer-input {
        width:100%;
        min-height:44px;
        border:0;
        outline:none;
        background:transparent;
        color:var(--text);
        font:inherit;
        font-size:15px;
    }

    .rm-home-explorer-input::placeholder {
        color:var(--muted);
    }

    .rm-home-explorer-clear {
        width:36px;
        height:36px;
        border:0;
        border-radius:12px;
        background:rgba(255,255,255,.06);
        color:var(--muted);
        font-size:18px;
        cursor:pointer;
    }

    .rm-home-explorer-results {
        display:grid;
        gap:8px;
        margin-top:10px;
        max-height:min(52vh, 420px);
        overflow:auto;
    }

    .rm-explore-hit {
        display:grid;
        grid-template-columns:auto 1fr;
        gap:12px;
        align-items:center;
        padding:11px 12px;
        border-radius:14px;
        border:1px solid rgba(255,255,255,.06);
        background:rgba(255,255,255,.03);
        text-decoration:none;
        color:inherit;
        transition:
            transform .16s ease,
            border-color .16s ease,
            background .16s ease;
    }

    .rm-explore-hit:hover,
    .rm-explore-hit.is-active {
        transform:translateY(-1px);
        border-color:rgba(232,204,150,.34);
        background:rgba(232,204,150,.08);
    }

    .rm-explore-hit-icon {
        width:34px;
        height:34px;
        display:flex;
        align-items:center;
        justify-content:center;
        border-radius:11px;
        background:rgba(232,204,150,.10);
        font-size:16px;
    }

    .rm-explore-hit-body strong {
        display:block;
        font-size:14px;
        line-height:1.25;
    }

    .rm-explore-hit-body small {
        display:block;
        margin-top:2px;
        color:var(--muted);
        font-size:11px;
        line-height:1.35;
    }

    .rm-explore-empty {
        padding:12px;
        border-radius:12px;
        color:var(--muted);
        font-size:13px;
        text-align:center;
    }

    .rm-home-explorer .rm-kind-chips {
        margin:12px 0 0;
    }

    @media (min-width: 860px) {
        main.page {
            max-width:1180px;
        }
    }

    @media (max-width: 620px) {
        .rm-stats-row {
            grid-template-columns:repeat(3,minmax(0,1fr));
            gap:8px;
        }

        .rm-stat {
            padding:12px 6px;
        }

        .rm-stat span {
            font-size:9px;
        }
    }
</style>"####;

    let body_before_main = r####""####;

    let guest_hint = if guest_mode { guest_mode_hint() } else { "" };

    let explore_index = build_home_explore_index(&_categories, &people_by_category);

    let hero = format!(
        r#"<section class="hero">
    <div class="eyebrow">
        {globe_icon}
        ResursMap
    </div>

    <h1>Карта ресурсов</h1>

    <p>Работа, работники и бизнес — найдите нужное рядом.</p>

    {guest_hint}

    <section class="rm-home-explorer card" id="rm-home-explorer">
        <div class="rm-home-explorer-head">
            <div class="card-title rm-home-explorer-title">
                Поиск
            </div>
            <div class="card-meta rm-home-explorer-copy">
                Город, профессия, вакансия или компания
            </div>
        </div>

        <div class="rm-home-explorer-field">
            <span class="rm-home-explorer-icon" aria-hidden="true">⌕</span>
            <input id="rm-home-explorer-input"
                   class="rm-home-explorer-input"
                   type="search"
                   inputmode="search"
                   autocomplete="off"
                   autocapitalize="off"
                   spellcheck="false"
                   placeholder="Ницца, электрик, вакансия…"
                   aria-label="Поиск работы, работников и бизнеса">
            <button id="rm-home-explorer-clear"
                    class="rm-home-explorer-clear"
                    type="button"
                    hidden
                    aria-label="Очистить">×</button>
        </div>

        <div id="rm-home-explorer-results"
             class="rm-home-explorer-results"
             hidden></div>

        {kind_chips}
    </section>

    <div class="rm-stats-row">
        <div class="rm-stat">
            <strong>{users_count}</strong>
            <span>участников</span>
        </div>
        <div class="rm-stat rm-stat-online">
            <strong>{online_count}</strong>
            <span>онлайн</span>
        </div>
        <div class="rm-stat">
            <strong>{resources_count}</strong>
            <span>объявлений</span>
        </div>
    </div>
</section>"#,
        globe_icon = icon("globe"),
        guest_hint = guest_hint,
        kind_chips = intent_kind_chips(
            "",
            false,
            "/app/search",
            "/app/search?kind=work",
            "/app/search?kind=workers",
            "/app/search?kind=business",
        ),
        users_count = users_count,
        online_count = online_count,
        resources_count = resources_count,
    );

    let main_html = format!(
        r####"
{topbar}

{hero}

{section_head_countries}

<div class="grid">
    {cards}
</div>
"####,
        topbar = topbar("Карта", "globe"),
        hero = hero,
        cards = cards,
    );

    let body_after = format!(
        r#"<script type="application/json" id="rm-home-explore-data">{explore_index}</script>
<script src="{home_explorer_js}" defer></script>"#,
        explore_index = explore_index,
        home_explorer_js = static_asset("home-explorer.js"),
    );

    page_document(
        "ResursMap",
        head_extra,
        body_before_main,
        &main_html,
        &bottom_nav("map"),
        &body_after,
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

        let content = format!(
            r#"
{section_head_countries}

<div class="grid">
    {cards}
</div>
"#,
            section_head_countries = section_head_countries,
            cards = cards,
        );

        return page_shell(
            name,
            &topbar("Карта", "globe"),
            &simple_hero(
                "map",
                "Регион",
                name,
                "Выберите страну, чтобы открыть доступные города и ресурсы.",
            ),
            &content,
            &bottom_nav("map"),
        );
    }

    render_continents(0, 0, 0, Vec::new(), Vec::new(), false)
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

            let content = format!(
                r#"
{section_head_cities}

<div class="grid">
    {cards}
</div>
"#,
                section_head_cities = section_head_cities,
                cards = cards,
            );

            return page_shell(
                country,
                &topbar("Карта", "globe"),
                &simple_hero(
                    "map-pin",
                    cname,
                    country,
                    "Выберите город и откройте его карту ресурсов.",
                ),
                &content,
                &bottom_nav("map"),
            );
        }
    }

    render_continents(0, 0, 0, Vec::new(), Vec::new(), false)
}

// ============================================================
// CITY
// ============================================================

pub fn render_city(ci: usize, si: usize, zi: usize) -> String {
    let w = world();

    if let Some((_cname, countries)) = w.iter().nth(ci) {
        if let Some((country, cities)) = countries.iter().nth(si) {
            if let Some(city) = cities.get(zi) {
                let section_head_sections = section_head("Разделы", "Выберите направление", None);

                let content = format!(
                    r#"
{section_head_sections}

<div class="grid">

    {work_card}

    {workers_card}

    {business_card}

</div>
"#,
                    section_head_sections = section_head_sections,
                    work_card = navigation_card(
                        &format!("/app/{}/{}/{}/cat/work?type=offer", ci, si, zi),
                        "briefcase",
                        "Работа",
                        "Вакансии рядом",
                    ),
                    workers_card = navigation_card(
                        &format!("/app/{}/{}/{}/cat/work?type=seeker", ci, si, zi),
                        "user",
                        "Работники",
                        "Профессии и кто ищет работу",
                    ),
                    business_card = navigation_card(
                        &format!("/app/{}/{}/{}/cat/business", ci, si, zi),
                        "building",
                        "Бизнес",
                        "Компании и услуги",
                    ),
                );

                return page_shell(
                    city,
                    &topbar("Карта", "globe"),
                    &simple_hero(
                        "map-pin",
                        country,
                        city,
                        "Работа, работники и бизнес в городе.",
                    ),
                    &content,
                    &bottom_nav("map"),
                );
            }
        }
    }

    render_continents(0, 0, 0, Vec::new(), Vec::new(), false)
}

// ============================================================
// SEARCH
// ============================================================

pub fn render_search(
    q: &str,
    kind: &str,
    resources: Vec<crate::web::view_models::SearchResourceRow>,
    people: Vec<crate::web::view_models::SearchPersonRow>,
    guest_mode: bool,
) -> String {
    let guest_hint = if guest_mode { guest_mode_hint() } else { "" };
    let world_data = world();

    let query_lower = q.trim().to_lowercase();

    let mut location_results = String::new();
    let mut location_count = 0usize;

    if !query_lower.is_empty() {
        for (ci, (continent, countries)) in world_data.iter().enumerate() {
            if continent.to_lowercase().contains(&query_lower) {
                location_count += 1;

                location_results.push_str(&navigation_card(
                    &format!("/app/{ci}"),
                    "globe",
                    continent,
                    "Континент",
                ));
            }

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

    let kind = kind.trim();
    let q_query = if q.trim().is_empty() {
        String::new()
    } else {
        format!("q={}", urlencoding::encode(q.trim()))
    };
    let search_href = |kind_value: &str| -> String {
        let mut href = String::from("/app/search?");
        if !q.trim().is_empty() {
            href.push_str(&q_query);
            href.push('&');
        }
        if !kind_value.is_empty() {
            href.push_str("kind=");
            href.push_str(&urlencoding::encode(kind_value));
        } else if href.ends_with('&') {
            href.pop();
        }
        if href.ends_with('?') {
            href.pop();
        }
        href
    };
    let kind_chips = intent_kind_chips(
        kind,
        true,
        &search_href(""),
        &search_href("work"),
        &search_href("workers"),
        &search_href("business"),
    );

    let people_count = people.len();
    let people_section = if people.is_empty() {
        String::new()
    } else {
        format!(
            r#"
{people_head}

<section>
    {people_cards}
</section>
"#,
            people_head = section_head(
                "По профессии",
                &format!("Найдено: {}", people_count),
                Some(24),
            ),
            people_cards = search_people_cards(&people),
        )
    };

    let result_count = resources.len();

    let results = if q.trim().is_empty() && kind.is_empty() {
        String::new()
    } else if resources.is_empty() && people.is_empty() && location_results.is_empty() {
        empty_state_card(
            "Ничего не найдено",
            &format!(
                "По запросу «{}» пока ничего не найдено.",
                escape_html(if q.trim().is_empty() {
                    match kind {
                        "work" => "работа",
                        "workers" => "работники",
                        "business" => "бизнес",
                        _ => q,
                    }
                } else {
                    q
                }),
            ),
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
                    listing_type,
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

                    let category_line = {
                        let label = profession_label(category);
                        match listing_type.as_str() {
                            "seeker" | "offer" => {
                                format!("{} · {}", label, resource_listing_label(listing_type))
                            }
                            _ => label,
                        }
                    };

                    let description_preview = {
                        let trimmed = description.trim();
                        let mut chars = trimmed.chars();
                        let preview: String = chars.by_ref().take(140).collect();
                        if chars.next().is_some() {
                            format!("{preview}…")
                        } else {
                            preview
                        }
                    };

                    let premium_badge = if *premium != 0 {
                        premium_badge_html("default")
                    } else {
                        ""
                    };

                    let verified_badge = if *verified != 0 {
                        verified_badge_html(true)
                    } else {
                        ""
                    };

                    resource_result_card(crate::web::templates::common::ResourceResultCardParams {
                        href: &format!("/app/resource/{}", id),
                        title_html: &escape_html(title),
                        category_html: &escape_html(&category_line),
                        description_html: &escape_html(&description_preview),
                        rating: *rating,
                        votes: *votes,
                        location_html: &escape_html(&location),
                        address_html: &escape_html(address),
                        premium_badge_html: premium_badge,
                        verified_badge_html: verified_badge,
                    })
                },
            )
            .collect::<Vec<_>>()
            .join("")
    };

    let location_section = if q.trim().is_empty() || location_results.is_empty() || !kind.is_empty()
    {
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

    let result_header = if resources.is_empty() {
        String::new()
    } else {
        section_head(
            match kind {
                "work" => "Вакансии",
                "workers" => "Объявления",
                "business" => "Бизнес",
                _ => "Объявления",
            },
            &format!("Найдено: {}", result_count),
            Some(24),
        )
    };

    let listings_block = if result_header.is_empty() && results.is_empty() {
        String::new()
    } else {
        format!(
            r#"
{result_header}

<section>
    {results}
</section>
"#,
            result_header = result_header,
            results = results,
        )
    };

    let content = format!(
        r#"
{location_section}

{listings_block}

{people_section}
"#,
        location_section = location_section,
        listings_block = listings_block,
        people_section = people_section,
    );

    let hero_extra = format!("{guest_hint}{kind_chips}");

    page_shell(
        "Поиск · ResursMap",
        &topbar("Поиск", "search"),
        &search_form_hero(
            "Поиск",
            match kind {
                "work" => "Найти работу",
                "workers" => "Найти работников",
                "business" => "Найти бизнес",
                _ => "Найти рядом",
            },
            "Работа, работники, бизнес, город или профессия.",
            q,
            "Например: электрик, Ницца, вакансия...",
            kind,
            &hero_extra,
        ),
        &content,
        &bottom_nav("search"),
    )
}

// ============================================================
// CONTACT REQUESTS
// ============================================================

pub fn render_menu() -> String {
    let pwa_install_js = super::common::static_asset("pwa-install.js");

    let content = format!(
        r#"<section>
    {section_head_settings}

    <section id="resursmap-install-panel"
             class="card rm-pwa-panel rm-pwa-panel--compact">

        <div class="rm-pwa-compact-row">
            <div class="rm-pwa-compact-copy">
                <div class="card-title rm-pwa-compact-title">
                    На главный экран
                </div>
                <div id="resursmap-install-hint"
                     class="card-meta rm-pwa-hint">
                    Ярлык сайта — не загрузка из магазина
                </div>
            </div>

            <button id="resursmap-install-pwa"
                    type="button"
                    class="ui-button rm-pwa-install-btn">
                ＋ Добавить
            </button>
        </div>
    </section>

    <div class="card rm-settings-card">
        <div class="card-title rm-settings-title">Звук и вибрация</div>

        <div class="rm-settings-toggle-stack">
            <button id="rm-menu-sound-toggle"
                    type="button"
                    class="rm-settings-toggle-btn">
                🔔 Звук включён
            </button>

            <button id="rm-menu-haptics-toggle"
                    type="button"
                    class="rm-settings-toggle-btn">
                📳 Вибрация включена
            </button>
        </div>

        <button class="ui-button sound-test-btn rm-settings-sound-btn"
                type="button">
            ▶ Прослушать уведомление
        </button>
    </div>

    <div class="card rm-settings-card rm-settings-card--spaced">
        <div class="card-title rm-settings-title">Тема оформления</div>

        <button class="theme-toggle-btn rm-settings-theme-btn" type="button">
            ☀️ Светлая тема
        </button>
    </div>

    <div class="card rm-settings-card rm-settings-card--spaced">
        <div class="card-title rm-settings-title--sm">Язык</div>
        <div class="card-meta">Русский</div>
    </div>
</section>"#,
        section_head_settings = section_head("Меню", "Настройки приложения", None),
    );

    let main_html = format!(
        r#"{topbar}

{hero}

{content}"#,
        topbar = topbar("Меню", "menu"),
        hero = simple_hero(
            "settings",
            "ResursMap",
            "Меню",
            "Тема, звук и ярлык на главном экране.",
        ),
        content = content,
    );

    let body_after = format!(
        r#"<script src="{pwa_install_js}" defer></script>
<script src="{menu_settings_js}" defer></script>"#,
        pwa_install_js = pwa_install_js,
        menu_settings_js = super::common::static_asset("menu-settings.js"),
    );

    page_document(
        "Меню · ResursMap",
        "",
        "",
        &main_html,
        &bottom_nav("menu"),
        &body_after,
    )
}
