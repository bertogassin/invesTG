use std::collections::BTreeMap;

pub fn world() -> BTreeMap<&'static str, BTreeMap<&'static str, Vec<&'static str>>> {
    let mut w = BTreeMap::new();

    let mut eu = BTreeMap::new();

    eu.insert(
        "Франция",
        vec!["Париж", "Марсель", "Лион", "Тулуза", "Ницца"],
    );

    eu.insert(
        "Германия",
        vec!["Берлин", "Гамбург", "Мюнхен", "Кёльн"],
    );

    eu.insert(
        "Италия",
        vec!["Рим", "Милан", "Неаполь", "Турин"],
    );

    w.insert("Европа", eu);

    w
}

pub fn render_continents() -> String {
    let w = world();
    let mut items = String::new();

    for (ci, (continent, countries)) in w.iter().enumerate() {
        let country_list: Vec<String> = countries
            .keys()
            .enumerate()
            .map(|(si, country)| {
                format!("<a href='/app/{ci}/{si}'>{country}</a>")
            })
            .collect();

        items.push_str(&format!(
            "<div class='card'><h2>{continent}</h2><p>{}</p></div>",
            country_list.join(" • ")
        ));
    }

    format!(
        "<!DOCTYPE html>
<html>
<head>
<meta charset='utf-8'>
<meta name='viewport' content='width=device-width, initial-scale=1'>
<title>Карта ресурсов</title>
<style>
body {{
    font-family: sans-serif;
    background: #0d0d0d;
    color: #eee;
    padding: 1rem;
    max-width: 800px;
    margin: 0 auto;
}}
.card {{
    background: #1a1a22;
    padding: 1rem;
    border-radius: 12px;
    margin: 1rem 0;
}}
a {{
    color: #4fc3f7;
    text-decoration: none;
    padding: 0.2rem 0.5rem;
}}
a:hover {{
    background: #2a2a32;
    border-radius: 6px;
}}
</style>
</head>
<body>
<h1>🌍 Карта ресурсов</h1>
<p>Выберите регион</p>
{}
</body>
</html>",
        items
    )
}

pub fn render_continent(ci: usize) -> String {
    let w = world();

    if let Some((name, countries)) = w.iter().nth(ci) {
        let list: String = countries
            .keys()
            .enumerate()
            .map(|(si, country)| {
                format!(
                    "<li><a href='/app/{ci}/{si}'>{country}</a></li>"
                )
            })
            .collect();

        return format!(
            "<!DOCTYPE html>
<html>
<head>
<meta charset='utf-8'>
<meta name='viewport' content='width=device-width, initial-scale=1'>
<title>{name}</title>
<style>
body {{
    font-family: sans-serif;
    background: #0d0d0d;
    color: #eee;
    padding: 1rem;
}}
a {{
    color: #4fc3f7;
    text-decoration: none;
}}
</style>
</head>
<body>
<h1>{name}</h1>
<ul>{list}</ul>
<a href='/app'>← Назад</a>
</body>
</html>"
        );
    }

    render_continents()
}

pub fn render_country(ci: usize, si: usize) -> String {
    let w = world();

    if let Some((cname, countries)) = w.iter().nth(ci) {
        if let Some((country, cities)) = countries.iter().nth(si) {
            let list: String = cities
                .iter()
                .enumerate()
                .map(|(zi, city)| {
                    format!(
                        "<li><a href='/app/{ci}/{si}/{zi}'>{city}</a></li>"
                    )
                })
                .collect();

            return format!(
                "<!DOCTYPE html>
<html>
<head>
<meta charset='utf-8'>
<meta name='viewport' content='width=device-width, initial-scale=1'>
<title>{country}</title>
<style>
body {{
    font-family: sans-serif;
    background: #0d0d0d;
    color: #eee;
    padding: 1rem;
}}
a {{
    color: #4fc3f7;
    text-decoration: none;
}}
</style>
</head>
<body>
<h1>{cname} · {country}</h1>
<ul>{list}</ul>
<a href='/app/{ci}'>← Назад</a>
</body>
</html>"
            );
        }
    }

    render_continents()
}

pub fn render_city(ci: usize, si: usize, zi: usize) -> String {
    let w = world();

    if let Some((cname, countries)) = w.iter().nth(ci) {
        if let Some((country, cities)) = countries.iter().nth(si) {
            if let Some(city) = cities.get(zi) {
                return format!(
                    "<!DOCTYPE html>
<html>
<head>
<meta charset='utf-8'>
<meta name='viewport' content='width=device-width, initial-scale=1'>
<title>{city}</title>
<style>
body {{
    font-family: sans-serif;
    background: #0d0d0d;
    color: #eee;
    padding: 1rem;
}}
a {{
    color: #4fc3f7;
    text-decoration: none;
}}
</style>
</head>
<body>
<h1>📍 {city}</h1>
<p>
Страна: {country}<br>
Регион: {cname}
</p>
<a href='/app/{ci}/{si}'>← Назад</a>
</body>
</html>"
                );
            }
        }
    }

    render_continents()
}

pub fn render_search(q: &str) -> String {
    format!(
        "<!DOCTYPE html>
<html>
<head>
<meta charset='utf-8'>
<meta name='viewport' content='width=device-width, initial-scale=1'>
<title>Поиск</title>
</head>
<body style='font-family:sans-serif;background:#0d0d0d;color:#eee;padding:1rem'>
<h1>🔎 Поиск</h1>
<p>Запрос: {}</p>
<a href='/app'>← Назад</a>
</body>
</html>",
        q
    )
}

pub fn render_me() -> String {
    "<!DOCTYPE html>
<html>
<head>
<meta charset='utf-8'>
<meta name='viewport' content='width=device-width, initial-scale=1'>
<title>Мой профиль</title>
</head>
<body style='font-family:sans-serif;background:#0d0d0d;color:#eee;padding:1rem'>
<h1>👤 Мой профиль</h1>
<p>Профиль пользователя ResursMap.</p>
<a href='/app'>← Назад</a>
</body>
</html>"
        .to_string()
}

pub fn render_category(
    ci: usize,
    si: usize,
    zi: usize,
    category: &str,
) -> String {
    let city_page = render_city(ci, si, zi);

    format!(
        "<!DOCTYPE html>
<html>
<head>
<meta charset='utf-8'>
<meta name='viewport' content='width=device-width, initial-scale=1'>
<title>{category}</title>
</head>
<body style='font-family:sans-serif;background:#0d0d0d;color:#eee;padding:1rem'>
<h1>📂 {category}</h1>
<p>Категория ресурсов города.</p>
{}
</body>
</html>",
        city_page
    )
}
