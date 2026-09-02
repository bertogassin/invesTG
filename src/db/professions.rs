use rusqlite::{params, Connection, OptionalExtension, Result};
use std::collections::BTreeSet;

const SECTORS: &[(&str, &str, &str, &str)] = &[
    (
        "construction",
        "Строительство и ремонт",
        "Construction and repair",
        "Bâtiment et rénovation",
    ),
    (
        "transport",
        "Транспорт и логистика",
        "Transport and logistics",
        "Transport et logistique",
    ),
    (
        "technology",
        "IT и технологии",
        "IT and technology",
        "Informatique et technologie",
    ),
    (
        "health",
        "Здоровье и уход",
        "Health and care",
        "Santé et soins",
    ),
    ("education", "Образование", "Education", "Éducation"),
    (
        "hospitality",
        "Гостеприимство и питание",
        "Hospitality and food",
        "Hôtellerie et restauration",
    ),
    (
        "business",
        "Бизнес и управление",
        "Business and management",
        "Commerce et gestion",
    ),
    (
        "finance",
        "Финансы и право",
        "Finance and law",
        "Finance et droit",
    ),
    (
        "creative",
        "Творчество и медиа",
        "Creative and media",
        "Création et médias",
    ),
    (
        "personal",
        "Бытовые и персональные услуги",
        "Personal services",
        "Services à la personne",
    ),
    (
        "industry",
        "Промышленность и производство",
        "Industry and manufacturing",
        "Industrie et production",
    ),
    (
        "agriculture",
        "Сельское хозяйство и природа",
        "Agriculture and nature",
        "Agriculture et nature",
    ),
];

// stable_key, sector, ru, en, fr, aliases (RU/EN/FR; separated by |)
const PROFESSIONS: &[(&str, &str, &str, &str, &str, &str)] = &[
    (
        "electrician",
        "construction",
        "Электрик",
        "Electrician",
        "Électricien",
        "электромонтажник|инженер-электрик|electrician|électricien",
    ),
    (
        "auto-electrician",
        "construction",
        "Автомобильный электрик",
        "Auto electrician",
        "Électricien automobile",
        "автоэлектрик|car electrician|électricien auto",
    ),
    (
        "plumber",
        "construction",
        "Сантехник",
        "Plumber",
        "Plombier",
        "сантехнические работы|plumber|plombier",
    ),
    (
        "builder",
        "construction",
        "Строитель",
        "Construction worker",
        "Ouvrier du bâtiment",
        "строительные работы|builder|bâtiment",
    ),
    (
        "mason",
        "construction",
        "Каменщик",
        "Mason",
        "Maçon",
        "кладка|bricklayer|maçon",
    ),
    (
        "carpenter",
        "construction",
        "Плотник",
        "Carpenter",
        "Charpentier",
        "столяр|carpenter|menuisier|charpentier",
    ),
    (
        "painter",
        "construction",
        "Маляр",
        "Painter",
        "Peintre en bâtiment",
        "маляр-штукатур|painter|peintre",
    ),
    (
        "tiler",
        "construction",
        "Плиточник",
        "Tiler",
        "Carreleur",
        "кафельщик|tiler|carreleur",
    ),
    (
        "roofer",
        "construction",
        "Кровельщик",
        "Roofer",
        "Couvreur",
        "крыша|roofer|couvreur",
    ),
    (
        "welder",
        "construction",
        "Сварщик",
        "Welder",
        "Soudeur",
        "сварка|welder|soudeur",
    ),
    (
        "architect",
        "construction",
        "Архитектор",
        "Architect",
        "Architecte",
        "architect|architecte",
    ),
    (
        "civil-engineer",
        "construction",
        "Инженер-строитель",
        "Civil engineer",
        "Ingénieur génie civil",
        "прораб|civil engineer|ingénieur bâtiment",
    ),
    (
        "driver",
        "transport",
        "Водитель",
        "Driver",
        "Chauffeur",
        "шофёр|водитель категории b|driver|chauffeur",
    ),
    (
        "truck-driver",
        "transport",
        "Водитель грузовика",
        "Truck driver",
        "Chauffeur poids lourd",
        "дальнобойщик|truck driver|chauffeur routier",
    ),
    (
        "courier",
        "transport",
        "Курьер",
        "Courier",
        "Coursier",
        "доставщик|delivery driver|livreur|coursier",
    ),
    (
        "taxi-driver",
        "transport",
        "Водитель такси",
        "Taxi driver",
        "Chauffeur de taxi",
        "таксист|taxi|vtc|chauffeur vtc",
    ),
    (
        "logistician",
        "transport",
        "Логист",
        "Logistics specialist",
        "Logisticien",
        "логистика|supply chain|logisticien",
    ),
    (
        "warehouse-worker",
        "transport",
        "Работник склада",
        "Warehouse worker",
        "Magasinier",
        "кладовщик|комплектовщик|warehouse|magasinier",
    ),
    (
        "mover",
        "transport",
        "Грузчик",
        "Mover",
        "Déménageur",
        "переезды|搬家|mover|déménageur",
    ),
    (
        "software-developer",
        "technology",
        "Разработчик программного обеспечения",
        "Software developer",
        "Développeur logiciel",
        "программист|developer|software engineer|développeur|programmeur",
    ),
    (
        "rust-developer",
        "technology",
        "Rust-разработчик",
        "Rust developer",
        "Développeur Rust",
        "rust programmer|rust engineer|développeur rust",
    ),
    (
        "web-developer",
        "technology",
        "Веб-разработчик",
        "Web developer",
        "Développeur web",
        "frontend|backend|fullstack|веб программист|développeur web",
    ),
    (
        "mobile-developer",
        "technology",
        "Разработчик мобильных приложений",
        "Mobile developer",
        "Développeur mobile",
        "android developer|ios developer|flutter developer",
    ),
    (
        "system-administrator",
        "technology",
        "Системный администратор",
        "System administrator",
        "Administrateur système",
        "сисадмин|devops|sysadmin|administrateur système",
    ),
    (
        "data-analyst",
        "technology",
        "Аналитик данных",
        "Data analyst",
        "Analyste de données",
        "data analytics|bi analyst|аналитик данных",
    ),
    (
        "cybersecurity-specialist",
        "technology",
        "Специалист по кибербезопасности",
        "Cybersecurity specialist",
        "Spécialiste cybersécurité",
        "информационная безопасность|cybersecurity|sécurité informatique",
    ),
    (
        "computer-technician",
        "technology",
        "Компьютерный мастер",
        "Computer technician",
        "Technicien informatique",
        "ремонт компьютеров|установка windows|pc repair|dépannage informatique",
    ),
    (
        "doctor",
        "health",
        "Врач",
        "Doctor",
        "Médecin",
        "доктор|physician|médecin",
    ),
    (
        "nurse",
        "health",
        "Медсестра / медбрат",
        "Nurse",
        "Infirmier",
        "медицинская сестра|nurse|infirmière|infirmier",
    ),
    (
        "dentist",
        "health",
        "Стоматолог",
        "Dentist",
        "Dentiste",
        "зубной врач|dentist|dentiste",
    ),
    (
        "pharmacist",
        "health",
        "Фармацевт",
        "Pharmacist",
        "Pharmacien",
        "аптекарь|pharmacist|pharmacien",
    ),
    (
        "psychologist",
        "health",
        "Психолог",
        "Psychologist",
        "Psychologue",
        "psychologist|psychologue",
    ),
    (
        "physiotherapist",
        "health",
        "Физиотерапевт",
        "Physiotherapist",
        "Kinésithérapeute",
        "реабилитолог|physio|kiné|kinésithérapeute",
    ),
    (
        "caregiver",
        "health",
        "Сиделка",
        "Caregiver",
        "Auxiliaire de vie",
        "помощник по уходу|caregiver|aide à domicile|auxiliaire de vie",
    ),
    (
        "teacher",
        "education",
        "Учитель",
        "Teacher",
        "Enseignant",
        "преподаватель|teacher|enseignant|professeur",
    ),
    (
        "tutor",
        "education",
        "Репетитор",
        "Tutor",
        "Professeur particulier",
        "частный преподаватель|tutor|cours particulier",
    ),
    (
        "translator",
        "education",
        "Переводчик",
        "Translator",
        "Traducteur",
        "устный переводчик|interpreter|translator|interprète|traducteur",
    ),
    (
        "language-teacher",
        "education",
        "Преподаватель языков",
        "Language teacher",
        "Professeur de langues",
        "английский язык|французский язык|language tutor",
    ),
    (
        "chef",
        "hospitality",
        "Повар",
        "Cook",
        "Cuisinier",
        "шеф-повар|cook|chef|cuisinier",
    ),
    (
        "baker",
        "hospitality",
        "Пекарь",
        "Baker",
        "Boulanger",
        "кондитер|baker|boulanger|pâtissier",
    ),
    (
        "waiter",
        "hospitality",
        "Официант",
        "Waiter",
        "Serveur",
        "официантка|waitress|serveuse|serveur",
    ),
    (
        "barista",
        "hospitality",
        "Бариста",
        "Barista",
        "Barista",
        "кофе|coffee|café",
    ),
    (
        "hotel-worker",
        "hospitality",
        "Работник гостиницы",
        "Hotel worker",
        "Employé d'hôtel",
        "горничная|receptionist|réceptionniste|femme de chambre",
    ),
    (
        "manager",
        "business",
        "Менеджер",
        "Manager",
        "Gestionnaire",
        "руководитель|управляющий|manager|gestionnaire",
    ),
    (
        "project-manager",
        "business",
        "Менеджер проектов",
        "Project manager",
        "Chef de projet",
        "project management|руководитель проекта|chef de projet",
    ),
    (
        "sales-specialist",
        "business",
        "Специалист по продажам",
        "Sales specialist",
        "Commercial",
        "продавец|sales manager|vendeur|commercial",
    ),
    (
        "marketing-specialist",
        "business",
        "Маркетолог",
        "Marketing specialist",
        "Spécialiste marketing",
        "маркетинг|marketing|digital marketing",
    ),
    (
        "hr-specialist",
        "business",
        "HR-специалист",
        "HR specialist",
        "Spécialiste RH",
        "кадровик|рекрутер|recruiter|ressources humaines",
    ),
    (
        "entrepreneur",
        "business",
        "Предприниматель",
        "Entrepreneur",
        "Entrepreneur",
        "бизнесмен|business owner|chef d'entreprise",
    ),
    (
        "accountant",
        "finance",
        "Бухгалтер",
        "Accountant",
        "Comptable",
        "бухучёт|accounting|comptable",
    ),
    (
        "financial-analyst",
        "finance",
        "Финансовый аналитик",
        "Financial analyst",
        "Analyste financier",
        "финансы|financial analyst|analyste financier",
    ),
    (
        "lawyer",
        "finance",
        "Юрист",
        "Lawyer",
        "Juriste",
        "адвокат|legal adviser|avocat|juriste",
    ),
    (
        "insurance-agent",
        "finance",
        "Страховой агент",
        "Insurance agent",
        "Agent d'assurance",
        "страхование|insurance|assurance",
    ),
    (
        "graphic-designer",
        "creative",
        "Графический дизайнер",
        "Graphic designer",
        "Graphiste",
        "дизайнер|graphic design|graphiste",
    ),
    (
        "photographer",
        "creative",
        "Фотограф",
        "Photographer",
        "Photographe",
        "фотосъёмка|photography|photographe",
    ),
    (
        "video-editor",
        "creative",
        "Видеомонтажёр",
        "Video editor",
        "Monteur vidéo",
        "монтаж видео|videographer|monteur vidéo",
    ),
    (
        "content-creator",
        "creative",
        "Создатель контента",
        "Content creator",
        "Créateur de contenu",
        "блогер|smm|content creator|créateur de contenu",
    ),
    (
        "interior-designer",
        "creative",
        "Дизайнер интерьера",
        "Interior designer",
        "Décorateur d'intérieur",
        "интерьер|interior design|décoration intérieure",
    ),
    (
        "cleaner",
        "personal",
        "Уборщик / клинер",
        "Cleaner",
        "Agent d'entretien",
        "уборка|клининг|cleaning|ménage|agent de nettoyage",
    ),
    (
        "hairdresser",
        "personal",
        "Парикмахер",
        "Hairdresser",
        "Coiffeur",
        "барбер|barber|hair stylist|coiffeur",
    ),
    (
        "beautician",
        "personal",
        "Косметолог",
        "Beautician",
        "Esthéticien",
        "визажист|маникюр|beautician|esthéticienne",
    ),
    (
        "security-guard",
        "personal",
        "Охранник",
        "Security guard",
        "Agent de sécurité",
        "безопасность|security|vigile|agent de sécurité",
    ),
    (
        "mechanic",
        "personal",
        "Автомеханик",
        "Auto mechanic",
        "Mécanicien automobile",
        "ремонт автомобилей|car mechanic|mécanicien auto",
    ),
    (
        "gardener",
        "personal",
        "Садовник",
        "Gardener",
        "Jardinier",
        "ландшафт|gardening|jardinier",
    ),
    (
        "seamstress",
        "personal",
        "Швея",
        "Seamstress",
        "Couturier",
        "портной|tailor|couturier|couturière",
    ),
    (
        "babysitter",
        "personal",
        "Няня",
        "Babysitter",
        "Garde d'enfants",
        "уход за детьми|nanny|baby-sitter|garde d'enfants",
    ),
    (
        "factory-worker",
        "industry",
        "Рабочий производства",
        "Factory worker",
        "Ouvrier de production",
        "завод|production worker|ouvrier usine",
    ),
    (
        "machine-operator",
        "industry",
        "Оператор станка",
        "Machine operator",
        "Opérateur de machine",
        "станочник|cnc operator|opérateur cn",
    ),
    (
        "industrial-engineer",
        "industry",
        "Промышленный инженер",
        "Industrial engineer",
        "Ingénieur industriel",
        "инженер производства|industrial engineer",
    ),
    (
        "quality-controller",
        "industry",
        "Контролёр качества",
        "Quality controller",
        "Contrôleur qualité",
        "quality assurance|контроль качества|qualité",
    ),
    (
        "farmer",
        "agriculture",
        "Фермер",
        "Farmer",
        "Agriculteur",
        "сельское хозяйство|farmer|agriculteur",
    ),
    (
        "agronomist",
        "agriculture",
        "Агроном",
        "Agronomist",
        "Agronome",
        "агрономия|agronomist|agronome",
    ),
    (
        "veterinarian",
        "agriculture",
        "Ветеринар",
        "Veterinarian",
        "Vétérinaire",
        "ветврач|vet|vétérinaire",
    ),
    (
        "landscaper",
        "agriculture",
        "Ландшафтный специалист",
        "Landscaper",
        "Paysagiste",
        "ландшафтный дизайнер|landscaper|paysagiste",
    ),
];

pub fn normalize(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace('ё', "е")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS profession_sectors (
            stable_key TEXT PRIMARY KEY,
            name_ru TEXT NOT NULL,
            name_en TEXT NOT NULL,
            name_fr TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS professions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            stable_key TEXT NOT NULL UNIQUE,
            sector_key TEXT NOT NULL REFERENCES profession_sectors(stable_key),
            name_ru TEXT NOT NULL,
            name_en TEXT NOT NULL,
            name_fr TEXT NOT NULL,
            normalized_ru TEXT NOT NULL,
            normalized_en TEXT NOT NULL,
            normalized_fr TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1
        );
        CREATE TABLE IF NOT EXISTS profession_aliases (
            profession_id INTEGER NOT NULL REFERENCES professions(id) ON DELETE CASCADE,
            alias TEXT NOT NULL,
            normalized_alias TEXT NOT NULL,
            UNIQUE(profession_id, normalized_alias)
        );
        CREATE INDEX IF NOT EXISTS idx_professions_sector ON professions(sector_key, name_ru);
        CREATE INDEX IF NOT EXISTS idx_professions_ru ON professions(normalized_ru);
        CREATE INDEX IF NOT EXISTS idx_profession_aliases_normalized ON profession_aliases(normalized_alias);
        CREATE TABLE IF NOT EXISTS profile_professions (
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            profession_id INTEGER NOT NULL REFERENCES professions(id),
            is_primary INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            PRIMARY KEY(user_id, profession_id)
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_profile_professions_primary
        ON profile_professions(user_id) WHERE is_primary = 1;
        CREATE TABLE IF NOT EXISTS profession_suggestions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            label TEXT NOT NULL,
            normalized_label TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            UNIQUE(user_id, normalized_label)
        );",
    )?;

    for (position, sector) in SECTORS.iter().enumerate() {
        conn.execute(
            "INSERT INTO profession_sectors(stable_key,name_ru,name_en,name_fr,position)
             VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(stable_key) DO UPDATE SET name_ru=excluded.name_ru,name_en=excluded.name_en,name_fr=excluded.name_fr,position=excluded.position",
            params![sector.0, sector.1, sector.2, sector.3, position as i64],
        )?;
    }

    for item in PROFESSIONS {
        conn.execute(
            "INSERT INTO professions(stable_key,sector_key,name_ru,name_en,name_fr,normalized_ru,normalized_en,normalized_fr)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(stable_key) DO UPDATE SET sector_key=excluded.sector_key,name_ru=excluded.name_ru,name_en=excluded.name_en,name_fr=excluded.name_fr,normalized_ru=excluded.normalized_ru,normalized_en=excluded.normalized_en,normalized_fr=excluded.normalized_fr,is_active=1",
            params![item.0,item.1,item.2,item.3,item.4,normalize(item.2),normalize(item.3),normalize(item.4)],
        )?;
        let id: i64 = conn.query_row(
            "SELECT id FROM professions WHERE stable_key=?1",
            [item.0],
            |row| row.get(0),
        )?;
        for alias in item.5.split('|').chain([item.2, item.3, item.4]) {
            let normalized = normalize(alias);
            if !normalized.is_empty() {
                conn.execute("INSERT OR IGNORE INTO profession_aliases(profession_id,alias,normalized_alias) VALUES(?1,?2,?3)", params![id, alias.trim(), normalized])?;
            }
        }
    }

    conn.execute(
        "INSERT OR IGNORE INTO profile_professions(user_id,profession_id,is_primary)
         SELECT p.user_id,profession.id,1
         FROM profiles p
         JOIN professions profession
           ON profession.normalized_ru = lower(replace(trim(p.category),'ё','е'))
           OR profession.normalized_en = lower(replace(trim(p.category),'ё','е'))
           OR profession.normalized_fr = lower(replace(trim(p.category),'ё','е'))
         WHERE trim(p.category) <> ''",
        [],
    )?;
    Ok(())
}

pub fn build_fts_query(conn: &Connection, query: &str) -> String {
    let mut groups = Vec::new();
    for raw_term in query.split_whitespace() {
        let term: String = raw_term.chars().filter(|c| c.is_alphanumeric()).collect();
        if term.chars().count() < 2 {
            continue;
        }
        let normalized = normalize(&term);
        let mut alternatives = BTreeSet::from([normalized.clone()]);
        if let Ok(mut stmt) = conn.prepare(
            "SELECT DISTINCT p.name_ru,p.name_en,p.name_fr
             FROM professions p LEFT JOIN profession_aliases a ON a.profession_id=p.id
             WHERE p.is_active=1 AND (p.normalized_ru LIKE ?1 OR p.normalized_en LIKE ?1 OR p.normalized_fr LIKE ?1 OR a.normalized_alias LIKE ?1)
             LIMIT 5",
        ) {
            let prefix = format!("{normalized}%");
            if let Ok(rows) = stmt.query_map([prefix], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            }) {
                for (ru, en, fr) in rows.filter_map(std::result::Result::ok) {
                    for label in [ru, en, fr] {
                        for word in label.split_whitespace() {
                            let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
                            if clean.chars().count() >= 2 {
                                alternatives.insert(normalize(&clean));
                            }
                        }
                    }
                }
            }
        }
        let expanded = alternatives
            .into_iter()
            .map(|value| format!("{value}*"))
            .collect::<Vec<_>>();
        groups.push(if expanded.len() == 1 {
            expanded[0].clone()
        } else {
            format!("({})", expanded.join(" OR "))
        });
    }
    groups.join(" ")
}

pub fn synchronize_profile(conn: &Connection, user_id: i64, raw: &str) -> Result<String> {
    let normalized = normalize(raw);
    conn.execute(
        "DELETE FROM profile_professions WHERE user_id=?1",
        [user_id],
    )?;
    if normalized.is_empty() {
        return Ok(String::new());
    }
    let found: Option<(i64, String)> = conn
        .query_row(
            "SELECT p.id,p.name_ru FROM professions p
             LEFT JOIN profession_aliases a ON a.profession_id=p.id
             WHERE p.is_active=1 AND (p.normalized_ru=?1 OR p.normalized_en=?1 OR p.normalized_fr=?1 OR a.normalized_alias=?1)
             ORDER BY CASE WHEN p.normalized_ru=?1 THEN 0 ELSE 1 END LIMIT 1",
            [&normalized],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    if let Some((profession_id, canonical)) = found {
        conn.execute(
            "INSERT INTO profile_professions(user_id,profession_id,is_primary) VALUES(?1,?2,1)",
            params![user_id, profession_id],
        )?;
        Ok(canonical)
    } else {
        conn.execute("INSERT OR IGNORE INTO profession_suggestions(user_id,label,normalized_label) VALUES(?1,?2,?3)", params![user_id, raw.trim(), normalized])?;
        Ok(raw.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> Connection {
        let conn = Connection::open_in_memory().expect("memory database");
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
             CREATE TABLE users(id INTEGER PRIMARY KEY);
             CREATE TABLE profiles(
                user_id INTEGER NOT NULL,
                category TEXT NOT NULL DEFAULT ''
             );
             INSERT INTO users(id) VALUES(1);",
        )
        .expect("base schema");
        initialize(&conn).expect("profession schema");
        conn
    }

    #[test]
    fn normalization_is_language_safe() {
        assert_eq!(normalize("  Инженёр   ЭЛЕКТРИК  "), "инженер электрик");
        assert_eq!(normalize("Chauffeur VTC"), "chauffeur vtc");
    }

    #[test]
    fn alias_maps_to_canonical_profession() {
        let conn = database();
        let canonical = synchronize_profile(&conn, 1, "chauffeur").expect("sync");
        assert_eq!(canonical, "Водитель");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM profile_professions WHERE user_id=1 AND is_primary=1",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(count, 1);
    }

    #[test]
    fn unknown_profession_is_preserved_for_review() {
        let conn = database();
        let label = synchronize_profile(&conn, 1, "Редкая новая профессия").expect("sync");
        assert_eq!(label, "Редкая новая профессия");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM profession_suggestions", [], |row| {
                row.get(0)
            })
            .expect("count");
        assert_eq!(count, 1);
    }

    #[test]
    fn multilingual_alias_expands_search() {
        let conn = database();
        let query = build_fts_query(&conn, "chauffeur Nice");
        assert!(query.contains("водитель*"));
        assert!(query.contains("driver*"));
        assert!(query.contains("nice*"));
    }
}
