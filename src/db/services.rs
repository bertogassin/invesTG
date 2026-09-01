use rusqlite::{params, Connection, Result};

const CATEGORIES: &[(&str, &str, &str, &str)] = &[
    (
        "home-repair",
        "Ремонт и дом",
        "Home repair",
        "Travaux et maison",
    ),
    ("cleaning", "Уборка", "Cleaning", "Nettoyage"),
    (
        "moving",
        "Переезды и доставка",
        "Moving and delivery",
        "Déménagement et livraison",
    ),
    (
        "auto",
        "Авто и транспорт",
        "Auto and transport",
        "Auto et transport",
    ),
    (
        "digital",
        "Компьютеры и цифровые услуги",
        "Computer and digital services",
        "Informatique et services numériques",
    ),
    (
        "education",
        "Обучение и языки",
        "Education and languages",
        "Formation et langues",
    ),
    (
        "beauty",
        "Красота и уход",
        "Beauty and care",
        "Beauté et soins",
    ),
    (
        "family",
        "Семья и помощь",
        "Family and assistance",
        "Famille et aide",
    ),
    (
        "business",
        "Бизнес-услуги",
        "Business services",
        "Services aux entreprises",
    ),
    (
        "events",
        "Фото, видео и мероприятия",
        "Photo, video and events",
        "Photo, vidéo et événements",
    ),
];

// stable key, category, RU, EN, FR, searchable aliases separated by '|'.
const SERVICES: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("plumbing-repair", "home-repair", "Сантехнические работы", "Plumbing services", "Travaux de plomberie", "сантехник|ремонт крана|устранение протечки|установка смесителя|plumber|plomberie|fuite d'eau"),
    ("electrical-work", "home-repair", "Электромонтажные работы", "Electrical work", "Travaux d'électricité", "электрик|розетки|проводка|освещение|electrician|wiring|électricien|prise électrique"),
    ("painting", "home-repair", "Покраска и малярные работы", "Painting services", "Travaux de peinture", "маляр|покраска стен|обои|painter|painting|peintre|peinture"),
    ("tiling", "home-repair", "Укладка плитки", "Tiling", "Pose de carrelage", "плиточник|кафель|ванная|tiler|tiles|carreleur|carrelage"),
    ("drywall", "home-repair", "Гипсокартон и перегородки", "Drywall and partitions", "Placo et cloisons", "гипсокартон|перегородка|drywall|plasterboard|placo|cloison"),
    ("flooring", "home-repair", "Укладка напольных покрытий", "Flooring installation", "Pose de revêtements de sol", "ламинат|паркет|линолеум|flooring|parquet|sol"),
    ("furniture-assembly", "home-repair", "Сборка мебели", "Furniture assembly", "Montage de meubles", "мебель|шкаф|кухня|ikea|furniture assembly|montage meuble"),
    ("door-window", "home-repair", "Установка дверей и окон", "Door and window installation", "Pose de portes et fenêtres", "двери|окна|door|window|porte|fenêtre"),
    ("appliance-repair", "home-repair", "Ремонт бытовой техники", "Appliance repair", "Dépannage électroménager", "стиральная машина|холодильник|посудомойка|appliance repair|électroménager"),
    ("home-cleaning", "cleaning", "Уборка квартир и домов", "Home cleaning", "Ménage à domicile", "уборка|клининг|генеральная уборка|cleaning|cleaner|ménage|nettoyage"),
    ("office-cleaning", "cleaning", "Уборка офисов", "Office cleaning", "Nettoyage de bureaux", "офисная уборка|office cleaning|nettoyage bureau"),
    ("window-cleaning", "cleaning", "Мытьё окон", "Window cleaning", "Nettoyage de vitres", "окна|мойка окон|window cleaning|vitres"),
    ("post-construction-cleaning", "cleaning", "Уборка после ремонта", "Post-construction cleaning", "Nettoyage après travaux", "после ремонта|строительная уборка|after builders cleaning|après travaux"),
    ("moving-service", "moving", "Квартирный и офисный переезд", "Home and office moving", "Déménagement particulier et professionnel", "переезд|грузчики|déménagement|movers|moving"),
    ("furniture-delivery", "moving", "Доставка мебели и покупок", "Furniture and purchase delivery", "Livraison de meubles et achats", "доставка|мебель|курьер|delivery|livraison"),
    ("cargo-transport", "moving", "Грузовые перевозки", "Cargo transport", "Transport de marchandises", "грузоперевозки|фургон|truck|cargo|transport marchandises"),
    ("car-repair", "auto", "Ремонт автомобилей", "Car repair", "Réparation automobile", "автомеханик|автосервис|car repair|mechanic|garage|mécanicien"),
    ("car-diagnostics", "auto", "Диагностика автомобиля", "Car diagnostics", "Diagnostic automobile", "автодиагностика|диагностика|car diagnostics|diagnostic auto"),
    ("tire-service", "auto", "Шиномонтаж", "Tire service", "Service pneumatiques", "шины|колеса|tire|tyre|pneu|pneumatique"),
    ("car-wash", "auto", "Мойка и детейлинг автомобиля", "Car wash and detailing", "Lavage et detailing automobile", "автомойка|детейлинг|car wash|detailing|lavage auto"),
    ("passenger-transport", "auto", "Пассажирские перевозки", "Passenger transport", "Transport de personnes", "такси|трансфер|водитель|taxi|transfer|chauffeur|vtc"),
    ("computer-repair", "digital", "Ремонт компьютеров", "Computer repair", "Dépannage informatique", "компьютерный мастер|ноутбук|pc repair|computer repair|dépannage informatique"),
    ("software-installation", "digital", "Установка программ и Windows", "Software and Windows installation", "Installation de logiciels et Windows", "windows|программы|переустановка|software installation|installation windows"),
    ("website-development", "digital", "Создание сайтов", "Website development", "Création de sites web", "сайт|веб разработка|website|web development|site internet"),
    ("mobile-app-development", "digital", "Разработка мобильных приложений", "Mobile app development", "Développement d'applications mobiles", "приложение|android|ios|mobile app|application mobile"),
    ("graphic-design", "digital", "Графический дизайн", "Graphic design", "Design graphique", "логотип|баннер|дизайнер|logo|graphic design|graphiste"),
    ("social-media", "digital", "Ведение социальных сетей", "Social media management", "Gestion des réseaux sociaux", "smm|соцсети|instagram|social media|réseaux sociaux"),
    ("language-lessons", "education", "Уроки иностранных языков", "Foreign language lessons", "Cours de langues", "репетитор|английский|французский|language lessons|cours anglais|cours français"),
    ("school-tutoring", "education", "Школьные занятия и репетиторство", "School tutoring", "Soutien scolaire", "математика|школа|домашнее задание|tutor|school support|soutien scolaire"),
    ("translation", "education", "Письменный и устный перевод", "Written and oral translation", "Traduction et interprétariat", "переводчик|перевод документов|translator|interpreter|traducteur|interprète"),
    ("haircut", "beauty", "Парикмахерские услуги", "Hairdressing", "Coiffure", "парикмахер|барбер|стрижка|hairdresser|barber|coiffeur"),
    ("manicure", "beauty", "Маникюр и педикюр", "Manicure and pedicure", "Manucure et pédicure", "ногти|маникюр|nails|manicure|ongles|manucure"),
    ("makeup", "beauty", "Макияж и визаж", "Makeup services", "Maquillage", "визажист|макияж|makeup|maquillage"),
    ("massage", "beauty", "Массаж", "Massage", "Massage", "массажист|massage therapist|masseur"),
    ("babysitting", "family", "Услуги няни", "Babysitting", "Garde d'enfants", "няня|дети|babysitter|nanny|garde enfants"),
    ("elderly-care", "family", "Помощь пожилым и уход", "Elderly care", "Aide aux personnes âgées", "сиделка|уход|caregiver|elderly care|auxiliaire de vie"),
    ("pet-care", "family", "Уход за животными", "Pet care", "Garde d'animaux", "собаки|кошки|выгул|pet sitter|dog walker|garde animaux"),
    ("legal-help", "business", "Юридическая помощь", "Legal services", "Services juridiques", "юрист|адвокат|документы|lawyer|legal|avocat|juridique"),
    ("accounting", "business", "Бухгалтерские услуги", "Accounting services", "Services comptables", "бухгалтер|налоги|accountant|accounting|comptable"),
    ("business-consulting", "business", "Бизнес-консультации", "Business consulting", "Conseil aux entreprises", "бизнес план|консультант|business consulting|conseil entreprise"),
    ("photography", "events", "Фотосъёмка", "Photography", "Photographie", "фотограф|фото|photographer|photography|photographe"),
    ("video-production", "events", "Видеосъёмка и монтаж", "Video production and editing", "Vidéo et montage", "видеограф|монтаж видео|video|video editing|vidéaste|montage vidéo"),
    ("event-organization", "events", "Организация мероприятий", "Event organization", "Organisation d'événements", "свадьба|праздник|event planner|wedding|organisation événement"),
];

pub fn normalize(value: &str) -> String {
    crate::db::professions::normalize(value)
}

pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS service_categories (
            stable_key TEXT PRIMARY KEY,
            name_ru TEXT NOT NULL,
            name_en TEXT NOT NULL,
            name_fr TEXT NOT NULL,
            normalized_ru TEXT NOT NULL,
            normalized_en TEXT NOT NULL,
            normalized_fr TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS services (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            stable_key TEXT NOT NULL UNIQUE,
            category_key TEXT NOT NULL REFERENCES service_categories(stable_key),
            name_ru TEXT NOT NULL,
            name_en TEXT NOT NULL,
            name_fr TEXT NOT NULL,
            normalized_ru TEXT NOT NULL,
            normalized_en TEXT NOT NULL,
            normalized_fr TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1
        );
        CREATE TABLE IF NOT EXISTS service_aliases (
            service_id INTEGER NOT NULL REFERENCES services(id) ON DELETE CASCADE,
            alias TEXT NOT NULL,
            normalized_alias TEXT NOT NULL,
            UNIQUE(service_id, normalized_alias)
        );
        CREATE INDEX IF NOT EXISTS idx_services_category ON services(category_key, name_ru);
        CREATE INDEX IF NOT EXISTS idx_services_ru ON services(normalized_ru);
        CREATE INDEX IF NOT EXISTS idx_service_aliases_normalized ON service_aliases(normalized_alias);",
    )?;

    for (position, category) in CATEGORIES.iter().enumerate() {
        conn.execute(
            "INSERT INTO service_categories(stable_key,name_ru,name_en,name_fr,normalized_ru,normalized_en,normalized_fr,position)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(stable_key) DO UPDATE SET name_ru=excluded.name_ru,name_en=excluded.name_en,name_fr=excluded.name_fr,normalized_ru=excluded.normalized_ru,normalized_en=excluded.normalized_en,normalized_fr=excluded.normalized_fr,position=excluded.position",
            params![category.0, category.1, category.2, category.3, normalize(category.1), normalize(category.2), normalize(category.3), position as i64],
        )?;
    }

    for service in SERVICES {
        conn.execute(
            "INSERT INTO services(stable_key,category_key,name_ru,name_en,name_fr,normalized_ru,normalized_en,normalized_fr)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(stable_key) DO UPDATE SET category_key=excluded.category_key,name_ru=excluded.name_ru,name_en=excluded.name_en,name_fr=excluded.name_fr,normalized_ru=excluded.normalized_ru,normalized_en=excluded.normalized_en,normalized_fr=excluded.normalized_fr,is_active=1",
            params![service.0,service.1,service.2,service.3,service.4,normalize(service.2),normalize(service.3),normalize(service.4)],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM services WHERE stable_key=?1",
            [service.0],
            |row| row.get(0),
        )?;
        for alias in service
            .5
            .split('|')
            .chain([service.2, service.3, service.4])
        {
            let normalized = normalize(alias);
            if !normalized.is_empty() {
                conn.execute("INSERT OR IGNORE INTO service_aliases(service_id,alias,normalized_alias) VALUES(?1,?2,?3)", params![id, alias.trim(), normalized])?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_catalog_is_stable_and_multilingual() {
        let conn = Connection::open_in_memory().expect("memory database");
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .expect("foreign keys");
        initialize(&conn).expect("service schema");
        let services: i64 = conn
            .query_row("SELECT COUNT(*) FROM services", [], |row| row.get(0))
            .expect("services");
        let aliases: i64 = conn.query_row("SELECT COUNT(*) FROM service_aliases WHERE normalized_alias IN ('ремонт крана','plumber','plomberie')", [], |row| row.get(0)).expect("aliases");
        assert!(services >= 40);
        assert_eq!(aliases, 3);
    }
}
