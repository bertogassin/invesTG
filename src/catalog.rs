//! Единый справочник рубрик для поиска, профиля и публикации.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RubricKind {
    Work,
    Business,
}

#[derive(Clone, Copy, Debug)]
pub struct Rubric {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: RubricKind,
    pub aliases: &'static [&'static str],
}

const RUBRICS: &[Rubric] = &[
    Rubric {
        id: "security",
        label: "Охрана",
        kind: RubricKind::Work,
        aliases: &["охранник", "охранники", "security", "guard"],
    },
    Rubric {
        id: "electrician",
        label: "Электрик",
        kind: RubricKind::Work,
        aliases: &["электрика", "электромонтаж", "электрики"],
    },
    Rubric {
        id: "plumber",
        label: "Сантехник",
        kind: RubricKind::Work,
        aliases: &["сантехника", "сантехники", "plumber"],
    },
    Rubric {
        id: "builder",
        label: "Строитель",
        kind: RubricKind::Work,
        aliases: &["строительство", "отделка", "маляр", "штукатур", "каменщик"],
    },
    Rubric {
        id: "welder",
        label: "Сварщик",
        kind: RubricKind::Work,
        aliases: &["сварка", "сварщики", "welder"],
    },
    Rubric {
        id: "tiler",
        label: "Плиточник",
        kind: RubricKind::Work,
        aliases: &["плитка", "плиточники", "кафель"],
    },
    Rubric {
        id: "carpenter",
        label: "Плотник и столяр",
        kind: RubricKind::Work,
        aliases: &["плотник", "столяр", "столярка"],
    },
    Rubric {
        id: "mechanic",
        label: "Автомеханик",
        kind: RubricKind::Work,
        aliases: &["автомеханик", "слесарь"],
    },
    Rubric {
        id: "driver",
        label: "Водитель",
        kind: RubricKind::Work,
        aliases: &["такси", "курьер", "доставка", "шофер", "водитель"],
    },
    Rubric {
        id: "cook",
        label: "Повар",
        kind: RubricKind::Work,
        aliases: &["кухня", "пекарь", "кондитер", "повар"],
    },
    Rubric {
        id: "cleaner",
        label: "Уборка",
        kind: RubricKind::Work,
        aliases: &["клининг", "уборщица", "уборщик", "горничная"],
    },
    Rubric {
        id: "caregiver",
        label: "Уход и няня",
        kind: RubricKind::Work,
        aliases: &["няня", "сиделка", "уход", "companion"],
    },
    Rubric {
        id: "warehouse",
        label: "Склад и грузчики",
        kind: RubricKind::Work,
        aliases: &["грузчик", "грузчики", "склад", "логистика"],
    },
    Rubric {
        id: "waiter",
        label: "Ресторан и зал",
        kind: RubricKind::Work,
        aliases: &["официант", "бармен", "хостес", "официантка"],
    },
    Rubric {
        id: "beauty",
        label: "Красота",
        kind: RubricKind::Work,
        aliases: &["парикмахер", "маникюр", "косметолог", "визажист"],
    },
    Rubric {
        id: "sales",
        label: "Продажи",
        kind: RubricKind::Work,
        aliases: &["продавец", "консультант", "кассир"],
    },
    Rubric {
        id: "office-admin",
        label: "Администратор",
        kind: RubricKind::Work,
        aliases: &["администратор", "секретарь", "ресепшн", "офис-менеджер"],
    },
    Rubric {
        id: "accountant",
        label: "Бухгалтер",
        kind: RubricKind::Work,
        aliases: &["бухгалтерия", "бухгалтер"],
    },
    Rubric {
        id: "it",
        label: "IT и компьютеры",
        kind: RubricKind::Work,
        aliases: &["программист", "айти", "it", "компьютер", "разработчик"],
    },
    Rubric {
        id: "medical",
        label: "Медицина",
        kind: RubricKind::Work,
        aliases: &["врач", "медсестра", "медбрат", "доктор"],
    },
    Rubric {
        id: "legal",
        label: "Юрист",
        kind: RubricKind::Work,
        aliases: &["адвокат", "юристконсульт"],
    },
    Rubric {
        id: "teacher",
        label: "Образование",
        kind: RubricKind::Work,
        aliases: &["учитель", "репетитор", "преподаватель", "педагог"],
    },
    Rubric {
        id: "translator",
        label: "Переводчик",
        kind: RubricKind::Work,
        aliases: &["перевод", "переводчики", "переводчица"],
    },
    Rubric {
        id: "photographer",
        label: "Фотограф",
        kind: RubricKind::Work,
        aliases: &["фото", "съемка", "съёмка", "фотограф"],
    },
    Rubric {
        id: "trainer",
        label: "Тренер",
        kind: RubricKind::Work,
        aliases: &["фитнес", "тренер", "инструктор"],
    },
    Rubric {
        id: "gardener",
        label: "Сад и участок",
        kind: RubricKind::Work,
        aliases: &["садовник", "озеленение", "сад"],
    },
    Rubric {
        id: "tailor",
        label: "Швея",
        kind: RubricKind::Work,
        aliases: &["швея", "портной", "ателье"],
    },
    Rubric {
        id: "handyman",
        label: "Разнорабочий",
        kind: RubricKind::Work,
        aliases: &["подсобник", "разнорабочие", "разнорабочий"],
    },
    Rubric {
        id: "cafe",
        label: "Кафе и рестораны",
        kind: RubricKind::Business,
        aliases: &["кафе", "ресторан", "пекарня"],
    },
    Rubric {
        id: "shop",
        label: "Магазин",
        kind: RubricKind::Business,
        aliases: &["магазин", "торговля"],
    },
    Rubric {
        id: "home-services",
        label: "Услуги для дома",
        kind: RubricKind::Business,
        aliases: &["ремонт квартир", "мастер на час"],
    },
    Rubric {
        id: "transport",
        label: "Перевозки",
        kind: RubricKind::Business,
        aliases: &["грузоперевозки", "переезд"],
    },
    Rubric {
        id: "autoservice",
        label: "Автосервис",
        kind: RubricKind::Business,
        aliases: &["сто", "шиномонтаж"],
    },
    Rubric {
        id: "rent-housing",
        label: "Аренда квартир",
        kind: RubricKind::Business,
        aliases: &["квартира", "жилье", "жильё", "аренда квартир"],
    },
    Rubric {
        id: "rent-auto",
        label: "Аренда авто",
        kind: RubricKind::Business,
        aliases: &["прокат авто", "аренда машины", "аренда авто"],
    },
    Rubric {
        id: "hotel",
        label: "Отель",
        kind: RubricKind::Business,
        aliases: &["гостиница", "отель", "хостел"],
    },
    Rubric {
        id: "real-estate",
        label: "Недвижимость",
        kind: RubricKind::Business,
        aliases: &["агентство недвижимости", "риелтор", "риэлтор"],
    },
    Rubric {
        id: "beauty-biz",
        label: "Салон и красота",
        kind: RubricKind::Business,
        aliases: &["салон", "барбершоп"],
    },
    Rubric {
        id: "construction-biz",
        label: "Строительная компания",
        kind: RubricKind::Business,
        aliases: &["стройка", "подряд"],
    },
    Rubric {
        id: "education-biz",
        label: "Курсы и школа",
        kind: RubricKind::Business,
        aliases: &["курсы", "школа"],
    },
    Rubric {
        id: "legal-biz",
        label: "Юридические услуги",
        kind: RubricKind::Business,
        aliases: &["нотариус"],
    },
    Rubric {
        id: "it-biz",
        label: "IT и веб",
        kind: RubricKind::Business,
        aliases: &["студия", "веб-студия"],
    },
];

pub fn all() -> &'static [Rubric] {
    RUBRICS
}

pub fn by_kind(kind: RubricKind) -> impl Iterator<Item = &'static Rubric> {
    RUBRICS.iter().filter(move |rubric| rubric.kind == kind)
}

pub fn by_id(id: &str) -> Option<&'static Rubric> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }

    RUBRICS.iter().find(|rubric| rubric.id == id)
}

pub fn normalize(value: &str) -> String {
    value.trim().replace(['ё', 'Ё'], "е").to_lowercase()
}

fn rubric_matches(rubric: &Rubric, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }

    if normalize(rubric.id) == needle || normalize(rubric.label) == needle {
        return true;
    }

    rubric
        .aliases
        .iter()
        .any(|alias| normalize(alias) == needle)
}

pub fn resolve(raw: &str) -> Option<&'static Rubric> {
    let needle = normalize(raw);
    if needle.is_empty() {
        return None;
    }

    if let Some(rubric) = RUBRICS
        .iter()
        .find(|rubric| rubric_matches(rubric, &needle))
    {
        return Some(rubric);
    }

    let mut best: Option<&'static Rubric> = None;
    let mut best_len = 0usize;

    for token in needle.split(|ch: char| !ch.is_alphanumeric()) {
        if token.chars().count() < 3 {
            continue;
        }

        if let Some(rubric) = RUBRICS.iter().find(|rubric| rubric_matches(rubric, token)) {
            if token.chars().count() > best_len {
                best = Some(rubric);
                best_len = token.chars().count();
            }
        }
    }

    best
}

pub fn label_for(raw: &str) -> String {
    resolve(raw)
        .map(|rubric| rubric.label.to_string())
        .unwrap_or_else(|| raw.trim().to_string())
}

pub fn search_text(rubric: &Rubric) -> String {
    let mut parts = Vec::with_capacity(2 + rubric.aliases.len());
    parts.push(rubric.id);
    parts.push(rubric.label);
    parts.extend(rubric.aliases.iter().copied());
    parts.join(" ")
}

pub fn search_text_for(raw: &str) -> String {
    by_id(raw)
        .or_else(|| resolve(raw))
        .map(search_text)
        .unwrap_or_else(|| raw.trim().to_string())
}

pub fn resource_category_for(rubric: &Rubric) -> &'static str {
    match rubric.kind {
        RubricKind::Work => "work",
        RubricKind::Business => "business",
    }
}

pub fn listing_type_for_intent(kind: RubricKind, listing_type: Option<&str>) -> &'static str {
    match kind {
        RubricKind::Work => match listing_type.map(str::trim) {
            Some("seeker") => "seeker",
            _ => "offer",
        },
        RubricKind::Business => "general",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_guard_aliases() {
        assert_eq!(resolve("Охрана").unwrap().id, "security");
        assert_eq!(resolve("охранник").unwrap().id, "security");
        assert_eq!(resolve("security").unwrap().id, "security");
    }

    #[test]
    fn resolves_housing_from_query() {
        assert_eq!(resolve("Аренда квартир").unwrap().id, "rent-housing");
        assert_eq!(resolve("квартира в Ницце").unwrap().id, "rent-housing");
    }

    #[test]
    fn resolves_trade_inside_city_query() {
        assert_eq!(resolve("сантехник в Ницце").unwrap().id, "plumber");
        assert!(search_text_for("plumber").contains("сантехник"));
    }

    #[test]
    fn rejects_empty_and_names() {
        assert!(resolve("").is_none());
        assert!(resolve("Иван").is_none());
    }

    #[test]
    fn resolves_common_trades() {
        assert_eq!(resolve("сварщик в Лионе").unwrap().id, "welder");
        assert_eq!(resolve("бухгалтер").unwrap().id, "accountant");
        assert_eq!(resolve("фотограф").unwrap().id, "photographer");
        assert_eq!(resolve("администратор").unwrap().id, "office-admin");
        assert_eq!(resolve("плиточник").unwrap().id, "tiler");
        assert_eq!(resolve("автомеханик").unwrap().id, "mechanic");
        assert_eq!(resolve("автосервис").unwrap().id, "autoservice");
    }

    #[test]
    fn rubric_ids_are_unique() {
        let mut ids = std::collections::BTreeSet::new();
        for rubric in RUBRICS {
            assert!(ids.insert(rubric.id), "duplicate rubric id: {}", rubric.id);
        }
    }
}
