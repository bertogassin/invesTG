use axum::{
    extract::{Path, Query, State},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;
use rusqlite::{params, Connection};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

fn world() -> BTreeMap<&'static str, BTreeMap<&'static str, Vec<&'static str>>> {
    let mut w = BTreeMap::new();

    let mut eu = BTreeMap::new();
    eu.insert("Франция", vec!["Париж", "Марсель", "Лион", "Тулуза", "Ницца", "Нант", "Монпелье", "Страсбург", "Бордо", "Лилль", "Ренн", "Реимс", "Орлеан"]);
    eu.insert("Германия", vec!["Берлин", "Гамбург", "Мюнхен", "Кёльн", "Франкфурт", "Штутгарт", "Дюссельдорф", "Лейпциг", "Дортмунд", "Эссен", "Бремен", "Дрезден"]);
    eu.insert("Великобритания", vec!["Лондон", "Бирмингем", "Манчестер", "Глазго", "Ливерпуль", "Бристоль", "Лидс", "Эдинбург", "Шеффилд", "Кардифф"]);
    eu.insert("Италия", vec!["Рим", "Милан", "Неаполь", "Турин", "Палермо", "Генуя", "Болонья", "Флоренция", "Бари", "Катания"]);
    eu.insert("Испания", vec!["Мадрид", "Барселона", "Валенсия", "Севилья", "Сарагоса", "Малага", "Бильбао", "Мурсия", "Пальма"]);
    eu.insert("Польша", vec!["Варшава", "Краков", "Лодзь", "Вроцлав", "Познань", "Гданьск", "Щецин", "Быдгощ", "Люблин"]);
    eu.insert("Нидерланды", vec!["Амстердам", "Роттердам", "Гаага", "Утрехт", "Эйндховен", "Гронинген"]);
    eu.insert("Бельгия", vec!["Брюссель", "Антверпен", "Гент", "Шарлеруа", "Льеж", "Брюгге"]);
    eu.insert("Австрия", vec!["Вена", "Грац", "Линц", "Зальцбург", "Инсбрук"]);
    eu.insert("Швейцария", vec!["Цюрих", "Женева", "Базель", "Берн", "Лозанна"]);
    eu.insert("Португалия", vec!["Лиссабон", "Порту", "Брага", "Коимбра"]);
    eu.insert("Швеция", vec!["Стокгольм", "Гётеборг", "Мальмё", "Уппсала"]);
    eu.insert("Норвегия", vec!["Осло", "Берген", "Тронхейм", "Ставангер"]);
    eu.insert("Дания", vec!["Копенгаген", "Орхус", "Оденсе"]);
    eu.insert("Финляндия", vec!["Хельсинки", "Эспоо", "Тампере", "Турку"]);
    eu.insert("Чехия", vec!["Прага", "Брно", "Острава", "Пльзень"]);
    eu.insert("Румыния", vec!["Бухарест", "Клуж-Напока", "Тимишоара", "Яссы", "Констанца"]);
    eu.insert("Греция", vec!["Афины", "Салоники", "Патра", "Ираклион"]);
    eu.insert("Венгрия", vec!["Будапешт", "Дебрецен", "Сегед", "Мишкольц"]);
    eu.insert("Ирландия", vec!["Дублин", "Корк", "Лимерик", "Голуэй"]);
    eu.insert("Украина", vec!["Киев", "Харьков", "Одесса", "Днепр", "Львов"]);
    eu.insert("Беларусь", vec!["Минск", "Гомель", "Могилёв", "Витебск", "Гродно"]);
    eu.insert("Молдова", vec!["Кишинёв", "Бельцы", "Тирасполь"]);
    eu.insert("Сербия", vec!["Белград", "Нови-Сад", "Ниш"]);
    eu.insert("Хорватия", vec!["Загреб", "Сплит", "Риека"]);
    eu.insert("Болгария", vec!["София", "Пловдив", "Варна", "Бургас"]);
    w.insert("Европа", eu);

    let mut kau = BTreeMap::new();
    kau.insert("Россия", vec!["Москва", "Санкт-Петербург", "Новосибирск", "Екатеринбург", "Казань", "Нижний Новгород", "Челябинск", "Самара", "Ростов-на-Дону", "Уфа", "Краснодар", "Воронеж", "Пермь", "Волгоград", "Красноярск", "Тюмень", "Сочи", "Астрахань"]);
    kau.insert("Грузия", vec!["Тбилиси", "Батуми", "Кутаиси"]);
    kau.insert("Армения", vec!["Ереван", "Гюмри", "Ванадзор"]);
    kau.insert("Азербайджан", vec!["Баку", "Гянджа", "Сумгайыт"]);
    kau.insert("Чечня", vec!["Грозный"]);
    kau.insert("Дагестан", vec!["Махачкала", "Дербент", "Хасавюрт"]);
    kau.insert("Ингушетия", vec!["Магас", "Назрань"]);
    kau.insert("Северная Осетия", vec!["Владикавказ"]);
    kau.insert("Кабардино-Балкария", vec!["Нальчик"]);
    kau.insert("Карачаево-Черкесия", vec!["Черкесск"]);
    kau.insert("Адыгея", vec!["Майкоп"]);
    kau.insert("Калмыкия", vec!["Элиста"]);
    w.insert("Кавказ и РФ", kau);

    let mut as_ = BTreeMap::new();
    as_.insert("Турция", vec!["Стамбул", "Анкара", "Измир", "Бурса", "Анталья", "Адана"]);
    as_.insert("Казахстан", vec!["Алматы", "Астана", "Шымкент", "Караганда"]);
    as_.insert("Узбекистан", vec!["Ташкент", "Самарканд", "Наманган", "Бухара"]);
    as_.insert("Кыргызстан", vec!["Бишкек", "Ош"]);
    as_.insert("Таджикистан", vec!["Душанбе", "Худжанд"]);
    as_.insert("ОАЭ", vec!["Дубай", "Абу-Даби", "Шарджа"]);
    as_.insert("Таиланд", vec!["Бангкок", "Чиангмай", "Пхукет"]);
    as_.insert("Вьетнам", vec!["Ханой", "Хошимин", "Дананг"]);
    as_.insert("Индия", vec!["Дели", "Мумбаи", "Бангалор", "Ченнаи", "Хайдарабад"]);
    as_.insert("Китай", vec!["Пекин", "Шанхай", "Гуанчжоу", "Шэньчжэнь"]);
    as_.insert("Япония", vec!["Токио", "Осака", "Иокогама", "Нагоя"]);
    as_.insert("Южная Корея", vec!["Сеул", "Пусан", "Инчхон"]);
    w.insert("Азия", as_);

    let mut am = BTreeMap::new();
    am.insert("США", vec!["Нью-Йорк", "Лос-Анджелес", "Чикаго", "Хьюстон", "Майами", "Сан-Франциско", "Сиэтл"]);
    am.insert("Канада", vec!["Торонто", "Монреаль", "Ванкувер", "Калгари", "Оттава"]);
    am.insert("Мексика", vec!["Мехико", "Гвадалахара", "Монтеррей", "Канкун"]);
    am.insert("Бразилия", vec!["Сан-Паулу", "Рио-де-Жанейро", "Бразилиа", "Салвадор"]);
    am.insert("Аргентина", vec!["Буэнос-Айрес", "Кордова", "Росарио"]);
    am.insert("Колумбия", vec!["Богота", "Медельин", "Кали"]);
    am.insert("Чили", vec!["Сантьяго", "Вальпараисо"]);
    w.insert("Америка", am);

    let mut oth = BTreeMap::new();
    oth.insert("Египет", vec!["Каир", "Александрия", "Гиза"]);
    oth.insert("Марокко", vec!["Касабланка", "Рабат", "Марракеш"]);
    oth.insert("ЮАР", vec!["Йоханнесбург", "Кейптаун", "Дурбан"]);
    oth.insert("Австралия", vec!["Сидней", "Мельбурн", "Брисбен", "Перт"]);
    oth.insert("Новая Зеландия", vec!["Окленд", "Веллингтон"]);
    w.insert("Другие", oth);

    w
}

fn categories() -> Vec<(&'static str, &'static str, Vec<&'static str>)> {
    vec![
        ("Профессии", "/static/icons/work.svg", vec!["Водитель", "Охранник", "Электрик", "Сантехник", "Строитель", "Повар", "Программист", "Дизайнер", "Маркетолог", "Переводчик", "Юрист", "Врач", "Медсестра", "Учитель", "Парикмахер", "Сварщик", "Механик", "Логист", "Продавец", "Администратор"]),
        ("Бизнес", "/static/icons/biz.svg", vec!["Есть бизнес", "Ищу партнёра", "Готов открыть точку", "Франшиза", "Онлайн-проект"]),
        ("Финансы", "/static/icons/money.svg", vec!["До 1 000 €", "1–5 000 €", "5–10 000 €", "10 000 €+", "Готов инвестировать", "Ищу инвестора"]),
        ("Жильё", "/static/icons/home.svg", vec!["Сдаю жильё", "Сниму жильё", "Комната", "Квартира", "Дом", "Посуточно / туризм"]),
        ("Транспорт", "/static/icons/car.svg", vec!["Легковой", "Грузовой", "Микроавтобус", "Мото", "Готов возить", "Ищу водителя"]),
        ("Команда", "/static/icons/team.svg", vec!["Готов в команду", "Собираю команду", "Ищу единомышленников", "Удалённо", "На месте"]),
        ("Услуги", "/static/icons/service.svg", vec!["Ремонт", "Уборка", "Доставка", "Красота", "IT-помощь", "Юр. консультация"]),
    ]
}



#[derive(Clone)]
struct AppState {
    db: std::sync::Arc<Mutex<Connection>>,
}

fn db_open() -> Connection {
    let conn = Connection::open("data/votes.db").expect("db open");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS votes (
            client_id TEXT NOT NULL,
            city TEXT NOT NULL,
            category TEXT NOT NULL,
            item TEXT NOT NULL,
            PRIMARY KEY (client_id, city, category, item)
        );
        CREATE INDEX IF NOT EXISTS idx_votes_city ON votes(city);
CREATE TABLE IF NOT EXISTS balances (
  client_id TEXT PRIMARY KEY,
  points_locked INTEGER NOT NULL DEFAULT 0,
  points_available INTEGER NOT NULL DEFAULT 0,
  lifetime_earned INTEGER NOT NULL DEFAULT 0,
  unlocked_main INTEGER NOT NULL DEFAULT 0,
  unlocked_city INTEGER NOT NULL DEFAULT 0,
  unlocked_marks INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS profiles (
  client_id TEXT PRIMARY KEY,
  username TEXT NOT NULL DEFAULT '',
  open_contact INTEGER NOT NULL DEFAULT 0,
  updated_at INTEGER NOT NULL DEFAULT 0,
  intent_text TEXT NOT NULL DEFAULT '',
  intent_until INTEGER NOT NULL DEFAULT 0
);"  
    ).ok();
    conn
}


fn webapp_user_id(init_data: &str, bot_token: &str) -> Option<i64> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    if init_data.is_empty() || bot_token.is_empty() {
        return None;
    }
    let mut hash = None;
    let mut pairs: Vec<(String, String)> = Vec::new();
    for part in init_data.split('&') {
        let mut kv = part.splitn(2, '=');
        let k = kv.next()?.to_string();
        let v = urlencoding_decode(kv.next().unwrap_or(""));
        if k == "hash" {
            hash = Some(v);
        } else {
            pairs.push((k, v));
        }
    }
    let hash = hash?;
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let data_check = pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut secret = HmacSha256::new_from_slice(b"WebAppData").ok()?;
    secret.update(bot_token.as_bytes());
    let secret_key = secret.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&secret_key).ok()?;
    mac.update(data_check.as_bytes());
    let expect = mac.finalize().into_bytes();
    let expect_hex = hex::encode(expect);
    if expect_hex != hash {
        return None;
    }
    // parse user json
    let user_v = pairs.into_iter().find(|(k, _)| k == "user").map(|(_, v)| v)?;
    let u: serde_json::Value = serde_json::from_str(&user_v).ok()?;
    u.get("id").and_then(|x| x.as_i64())
}

fn urlencoding_decode(s: &str) -> String {
    // minimal decode
    let mut out = String::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let h = std::str::from_utf8(&b[i+1..i+3]).ok();
            if let Some(h) = h {
                if let Ok(v) = u8::from_str_radix(h, 16) {
                    out.push(v as char);
                    i += 3;
                    continue;
                }
            }
        }
        if b[i] == b'+' {
            out.push(' ');
        } else {
            out.push(b[i] as char);
        }
        i += 1;
    }
    out
}

fn city_chat(city: &str) -> Option<String> {
    let data = std::fs::read_to_string("data/city_chats.json").ok()?;
    let map: std::collections::HashMap<String, String> =
        serde_json::from_str(&data).ok()?;
    map.get(city).cloned()
}


const STARTER_LOCKED: i64 = 100;
const UNLOCK_MARKS: i64 = 40;
const UNLOCK_CITY: i64 = 40;

fn ensure_balance(conn: &Connection, client_id: &str) {
    let _ = conn.execute(
        "INSERT OR IGNORE INTO balances (client_id, points_locked, points_available, lifetime_earned)
         VALUES (?1, ?2, 0, ?2)",
        rusqlite::params![client_id, STARTER_LOCKED],
    );
}

fn unlock_for_activity(conn: &Connection, client_id: &str) {
    ensure_balance(conn, client_id);
    let marks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM votes WHERE client_id=?1",
            [client_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let cities: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT city) FROM votes WHERE client_id=?1",
            [client_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let (mut locked, mut avail, mut um, mut uc, _uk): (i64, i64, i64, i64, i64) = conn
        .query_row(
            "SELECT points_locked, points_available, unlocked_marks, unlocked_city, unlocked_main
             FROM balances WHERE client_id=?1",
            [client_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap_or((STARTER_LOCKED, 0, 0, 0, 0));

    let mut moved = 0i64;
    if marks >= 3 && um == 0 {
        let take = UNLOCK_MARKS.min(locked);
        locked -= take;
        avail += take;
        um = 1;
        moved += take;
    }
    if cities >= 1 && uc == 0 {
        let take = UNLOCK_CITY.min(locked);
        locked -= take;
        avail += take;
        uc = 1;
        moved += take;
    }
    let _ = moved;
    let _ = conn.execute(
        "UPDATE balances SET points_locked=?1, points_available=?2,
         unlocked_marks=?3, unlocked_city=?4 WHERE client_id=?5",
        rusqlite::params![locked, avail, um, uc, client_id],
    );
}

fn balance_html(conn: &Connection, client_id: &str) -> String {
    ensure_balance(conn, client_id);
    unlock_for_activity(conn, client_id);
    let (locked, avail, um, uc): (i64, i64, i64, i64) = conn
        .query_row(
            "SELECT points_locked, points_available, unlocked_marks, unlocked_city FROM balances WHERE client_id=?1",
            [client_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap_or((100, 0, 0, 0));
    let t1 = if um == 1 { "✓" } else { "○" };
    let t2 = if uc == 1 { "✓" } else { "○" };
    format!(
        r#"<div class="card" style="margin:12px 0;padding:12px;border-radius:12px;background:#1a1a22">
        <div style="font-weight:600;margin-bottom:6px">⚡ Ресурсы</div>
        <div>Доступно: <b>{avail}</b> · Ещё за задания: <b>{locked}</b></div>
        <div class="sub" style="margin-top:8px;font-size:13px;opacity:.85">
        {t1} 3 отметки в категориях<br>
        {t2} выбран город (есть отметки)
        </div></div>"#,
        avail = avail,
        locked = locked,
        t1 = t1,
        t2 = t2,
    )
}


fn shell(title: &str, body: &str) -> Html<String> {
    Html(format!(
        r##"<!DOCTYPE html>
<html lang="ru"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no"/>
<title>{title}</title>
<script src="https://telegram.org/js/telegram-web-app.js"></script>
<style>
body{{margin:0;font-family:system-ui,-apple-system,Segoe UI,Roboto,sans-serif;background:#0e1621;color:#e7ecf3;line-height:1.45}}
header{{position:sticky;top:0;z-index:20;background:rgba(14,22,33,.95);border-bottom:1px solid #1e2d3d;padding:10px 14px}}
main{{padding:14px 14px 40px;max-width:640px;margin:0 auto}}
.card{{background:#15202b;border:1px solid #1e2d3d;border-radius:14px;padding:14px;margin:12px 0}}
.item{{display:block;padding:12px 10px;border-bottom:1px solid #1e2d3d;color:#e7ecf3;text-decoration:none}}
.item:last-child{{border-bottom:0}}
.cta{{display:block;text-align:center;background:#2a6df4;color:#fff;text-decoration:none;padding:12px;border-radius:12px;margin:8px 0;font-weight:600}}
.cta2{{display:block;text-align:center;background:#1a2836;color:#c5d0dc;text-decoration:none;padding:11px;border-radius:12px;margin:8px 0;border:1px solid #2a3f55}}
h1{{font-size:1.35rem;margin:8px 0}}
.sub{{color:#8b9bb0;margin:0 0 10px;font-size:.92rem}}
.nav a{{color:#9ec1ff;margin-right:12px;text-decoration:none;font-size:.9rem}}
.tag{{color:#8b9bb0;font-size:.85rem}}
.muted{{color:#8b9bb0;font-size:.85rem}}
img.ic{{vertical-align:middle;margin-right:6px}}
input[type=checkbox]{{width:18px;height:18px}}
</style>
</head>
<body>
<header class="nav">
<a href="/app/">Карта</a>
<a href="/app/search">Поиск</a>
<a href="/app/me">Профиль</a>
</header>
<main>
{body}
</main>
<script>
try{{if(window.Telegram&&Telegram.WebApp){{Telegram.WebApp.ready();Telegram.WebApp.expand(); try{{if(window.rmMigrateAll)rmMigrateAll();}}catch(e){{}}}}}}catch(e){{}}

window.RM_CACHE_VER="v2";
window.rmClient=function(){{
  try{{
    var u=window.Telegram&&Telegram.WebApp&&Telegram.WebApp.initDataUnsafe&&Telegram.WebApp.initDataUnsafe.user;
    if(u&&u.id){{
      var tid="tg:"+String(u.id);
      try{{localStorage.setItem("rm_cid",tid);}}catch(e){{}}
      return tid;
    }}
  }}catch(e){{}}
  var id=null;
  try{{id=localStorage.getItem("rm_cid");}}catch(e){{}}
  if(!id){{
    id="c"+Math.random().toString(36).slice(2)+Date.now().toString(36);
    try{{localStorage.setItem("rm_cid",id);}}catch(e){{}}
  }}
  return id;
}};
window.rmKey=function(city,cat){{
  return "rm:"+window.RM_CACHE_VER+":"+city+"|"+cat;
}};
window.rmReadArr=function(key){{
  try{{
    var raw=localStorage.getItem(key);
    if(!raw) return [];
    var arr=JSON.parse(raw);
    return Array.isArray(arr)?arr.filter(function(x){{return typeof x==="string"&&x.length>0;}}):[];
  }}catch(e){{ return []; }}
}};
window.rmWriteArr=function(key,arr){{
  try{{
    if(arr&&arr.length) localStorage.setItem(key,JSON.stringify(arr));
    else localStorage.removeItem(key);
  }}catch(e){{}}
}};
window.rmToggle=function(name,on){{
  var city=window.__rmCity||""; var cat=window.__rmCat||"";
  if(!city||!cat||!name) return;
  try{{localStorage.setItem("rm_last_city",city);}}catch(e){{}}
  var k=rmKey(city,cat);
  var arr=rmReadArr(k);
  if(on){{ if(arr.indexOf(name)<0) arr.push(name); }}
  else arr=arr.filter(function(x){{return x!==name;}});
  rmWriteArr(k,arr);
  try{{
    fetch("/api/vote",{{method:"POST",headers:{{"Content-Type":"application/json"}},
      body:JSON.stringify({{
        client_id:rmClient(),city:city,category:cat,item:name,on:!!on,
        init_data:(window.Telegram&&Telegram.WebApp&&Telegram.WebApp.initData)||""
      }})}}).then(function(){{
        if(window.rmLoadStats) try{{rmLoadStats(city);}}catch(e){{}}
      }}).catch(function(){{}});
  }}catch(e){{}}
}};
window.rmRestore=function(){{
  var city=window.__rmCity||""; var cat=window.__rmCat||"";
  if(!city||!cat) return;
  var k=rmKey(city,cat);
  var arr=rmReadArr(k);
  // migrate old keys without version
  if(!arr.length){{
    var old="rm:"+city+"|"+cat;
    arr=rmReadArr(old);
    if(arr.length){{ rmWriteArr(k,arr); try{{localStorage.removeItem(old);}}catch(e){{}} }}
  }}
  document.querySelectorAll("input[data-item]").forEach(function(el){{
    el.checked = arr.indexOf(el.dataset.item)>=0;
  }});
}};
window.rmAllLocal=function(){{
  var rows=[];
  try{{
    for(var i=0;i<localStorage.length;i++){{
      var key=localStorage.key(i);
      if(!key) continue;
      var rest=null;
      if(key.indexOf("rm:"+window.RM_CACHE_VER+":")===0) rest=key.slice(("rm:"+window.RM_CACHE_VER+":").length);
      else if(key.indexOf("rm:")===0 && key.indexOf("rm:v")!==0) rest=key.slice(3);
      if(rest==null) continue;
      var p=rest.indexOf("|");
      if(p<0) continue;
      var city=rest.slice(0,p), cat=rest.slice(p+1);
      rmReadArr(key).forEach(function(it){{
        rows.push({{city:city,cat:cat,item:it}});
      }});
    }}
  }}catch(e){{}}
  return rows;
}};
window.rmClearLocal=function(){{
  try{{
    var keys=[];
    for(var i=0;i<localStorage.length;i++){{
      var k=localStorage.key(i);
      if(k&&k.indexOf("rm:")===0) keys.push(k);
    }}
    keys.forEach(function(k){{localStorage.removeItem(k);}});
  }}catch(e){{}}
}};
window.rmMigrateAll=function(){{
  try{{
    var keys=[];
    for(var i=0;i<localStorage.length;i++){{
      var k=localStorage.key(i);
      if(k) keys.push(k);
    }}
    keys.forEach(function(key){{
      if(key.indexOf("rm:")!==0) return;
      if(key.indexOf("rm:"+window.RM_CACHE_VER+":")===0) return;
      if(key==="rm_cid"||key==="rm_last_city") return;
      if(key.indexOf("rm:v")===0) return; // other version prefixes leave for now
      // old form rm:City|Cat
      var rest=key.slice(3);
      if(rest.indexOf("|")<0) return;
      var arr=rmReadArr(key);
      if(!arr.length){{ try{{localStorage.removeItem(key);}}catch(e){{}} return; }}
      var nk="rm:"+window.RM_CACHE_VER+":"+rest;
      var cur=rmReadArr(nk);
      arr.forEach(function(it){{ if(cur.indexOf(it)<0) cur.push(it); }});
      rmWriteArr(nk,cur);
      try{{localStorage.removeItem(key);}}catch(e){{}}
    }});
  }}catch(e){{}}
}};


window.rmLoadStats=function(city){{
  var box=document.getElementById("stats");
  if(!box||!city) return;
  fetch("/api/stats?city="+encodeURIComponent(city))
    .then(function(r){{return r.json();}})
    .then(function(rows){{
      if(!rows||!rows.length){{
        box.innerHTML="<p class=\\"muted\\">Пока нет отметок в этом городе</p>";
        return;
      }}
      var by={{}};
      rows.forEach(function(x){{
        var c=x.category||"";
        if(!by[c]) by[c]=[];
        by[c].push(x);
      }});
      var html="";
      Object.keys(by).forEach(function(cat){{
        html+="<div style=\\"margin:10px 0 6px;color:#9ec1ff;font-weight:600\\">"+cat+"</div>";
        by[cat].sort(function(a,b){{return (b.n||0)-(a.n||0);}}).forEach(function(x){{
          html+="<div class=\\"item\\" style=\\"display:flex;justify-content:space-between;gap:8px\\"><span>"+
            (x.item||"")+"</span><b>"+(x.n||0)+"</b></div>";
        }});
      }});
      box.innerHTML=html;
    }})
    .catch(function(){{ box.innerHTML="<p class=\\"muted\\">Не удалось загрузить</p>"; }});
}};

</script>
</body></html>"##,
        title = title,
        body = body,
    ))
}


async fn home() -> Html<String> {
    shell(
        "Карта ресурсов",
        r#"<h1>Карта людей и ресурсов</h1>
<p class="sub">Города → профессии, жильё, капитал, команды. Сначала карта — потом проекты.</p>
<a class="cta" href="/app/">Открыть карту</a>
<a class="cta2" href="https://t.me/ResursWork_bot">Telegram-бот</a>
<a class="cta2" href="https://t.me/+3t_HTJT51Hs4ODM0">Главный чат</a>
<div class="card"><span class="chip">Европа</span><span class="chip">Кавказ</span><span class="chip">Азия</span><span class="chip">Америка</span>
<p class="muted" style="margin:10px 0 0">Демо для презентации. Голоса и чаты городов подключим к общей базе.</p></div>"#,
    )
}

async fn app_root() -> Html<String> {
    let w = world();
    let mut items = String::new();
    for (i, name) in w.keys().enumerate() {
        let n = w[name].len();
        items.push_str(&format!(
            r#"<a class="item" href="/app/{i}"><span class="row"><span>{name}</span><span class="muted">{n} стран</span></span></a>"#
        ));
    }
    shell(
        "Континенты",
        &format!(
            r#"<h1>Континенты</h1>
<p class="sub">Выберите регион</p>
<a class="cta2" href="/app/search">🔎 Найти город</a>
{items}"#
        ),
    )
}

async fn app_continent(Path(ci): Path<usize>) -> Html<String> {
    let w = world();
    let Some((cname, countries)) = w.iter().nth(ci) else {
        return shell("Ошибка", r#"<p>Не найдено</p><a class="cta2" href="/app/">Назад</a>"#);
    };
    let mut items = String::new();
    for (si, (sname, cities)) in countries.iter().enumerate() {
        items.push_str(&format!(
            r#"<a class="item" href="/app/{ci}/{si}"><span class="row"><span>{sname}</span><span class="muted">{} городов</span></span></a>"#,
            cities.len()
        ));
    }
    shell(
        cname,
        &format!(
            r#"<p class="tag"><a href="/app/" style="color:#6ab2f2;text-decoration:none">← Континенты</a></p>
<h1>{cname}</h1>
<p class="sub">Страны</p>
{items}"#
        ),
    )
}

async fn app_country(Path((ci, si)): Path<(usize, usize)>) -> Html<String> {
    let w = world();
    let Some((cname, countries)) = w.iter().nth(ci) else {
        return shell("Ошибка", r#"<a class="cta2" href="/app/">Назад</a>"#);
    };
    let Some((sname, cities)) = countries.iter().nth(si) else {
        return shell("Ошибка", &format!(r#"<a class="cta2" href="/app/{ci}">Назад</a>"#));
    };
    let mut items = String::new();
    for (zi, city) in cities.iter().enumerate() {
        items.push_str(&format!(
            r#"<a class="item" href="/app/{ci}/{si}/{zi}"><img class="ic" src="/static/icons/pin.svg" width="18" height="18" alt=""/> {city}</a>"#
        ));
    }
    shell(
        sname,
        &format!(
            r#"<p class="tag"><a href="/app/{ci}" style="color:#6ab2f2;text-decoration:none">← {cname}</a></p>
<h1>{sname}</h1>
<p class="sub">Города</p>
{items}"#
        ),
    )
}

async fn app_city(Path((ci, si, zi)): Path<(usize, usize, usize)>) -> Html<String> {
    let w = world();
    let Some((cname, countries)) = w.iter().nth(ci) else {
        return shell("Ошибка", r#"<a class="cta2" href="/app/">Назад</a>"#);
    };
    let Some((sname, cities)) = countries.iter().nth(si) else {
        return shell("Ошибка", &format!(r#"<a class="cta2" href="/app/{ci}">Назад</a>"#));
    };
    let Some(city) = cities.get(zi) else {
        return shell("Ошибка", &format!(r#"<a class="cta2" href="/app/{ci}/{si}">Назад</a>"#));
    };

    let mut cats = String::new();
    for (k, (title, icon, _)) in categories().iter().enumerate() {
        cats.push_str(&format!(
            r#"<a class="item" href="/app/{ci}/{si}/{zi}/cat/{k}"><img class="ic" src="{icon}" width="18" height="18" alt=""/> {title}</a>"#,
            ci = ci, si = si, zi = zi, k = k, icon = icon, title = title
        ));
    }

    let chat = match city_chat(city) {
        Some(u) => format!(
            r#"<a class="cta" href="{u}" target="_blank" rel="noopener">💬 Открыть чат города</a>"#
        ),
        None => r#"<p class="muted" style="margin:8px 0">💬 Чат этого города пока не привязан</p>"#.to_string(),
    };

    shell(
        city,
        &format!(
            r#"<p class="tag"><a href="/app/{ci}/{si}" style="color:#6ab2f2;text-decoration:none">← {sname}</a> · {cname}</p>
<h1 style="display:flex;align-items:center;gap:8px"><img src="/static/icons/pin.svg" width="22" height="22" alt=""/> {city}</h1>
<script>window.__rmCity="{city}";try{{localStorage.setItem("rm_last_city","{city}");}}catch(e){{}}</script>
<p class="sub">Категории ниже · сводку можно фильтровать · шаблон объявления → в чат города</p>
{chat}
<a class="cta2" href="https://t.me/+3t_HTJT51Hs4ODM0">🏠 Главный чат сообщества</a>
<div class="card" id="openBox" style="margin:12px 0">
  <p class="tag" style="margin:0">Связь в городе: <b id="openN">…</b> открыты к контакту</p>
  <p class="muted" style="font-size:.85rem;margin:6px 0 10px">Договорённости — в чате города. Список людей не публикуем.</p>
  <p class="tag" style="margin:0 0 6px">Шаблоны объявления</p>
  <div style="display:flex;flex-wrap:wrap;gap:6px;margin-bottom:8px">
    <button type="button" class="cta2 adTpl" data-tpl="job" style="flex:1;min-width:42%;padding:8px;font-size:.85rem">Ищу работу</button>
    <button type="button" class="cta2 adTpl" data-tpl="svc" style="flex:1;min-width:42%;padding:8px;font-size:.85rem">Предлагаю услугу</button>
    <button type="button" class="cta2 adTpl" data-tpl="rent" style="flex:1;min-width:42%;padding:8px;font-size:.85rem">Ищу жильё</button>
    <button type="button" class="cta2 adTpl" data-tpl="offer" style="flex:1;min-width:42%;padding:8px;font-size:.85rem">Сдаю / транспорт</button>
  </div>
  <p class="muted" id="copyMsg" style="margin:0"></p>
</div>
<script>
(function(){{
  var city={city_js};
  fetch("/api/open_count?city="+encodeURIComponent(city))
    .then(function(r){{return r.json();}})
    .then(function(x){{ var n=document.getElementById("openN"); if(n) n.textContent=String(x.n||0); }})
    .catch(function(){{ var n=document.getElementById("openN"); if(n) n.textContent="0"; }});
  function copyTxt(txt){{
    var m=document.getElementById("copyMsg");
    function ok(){{ if(m) m.textContent="Скопировано — вставьте в чат города"; }}
    if(navigator.clipboard&&navigator.clipboard.writeText){{
      navigator.clipboard.writeText(txt).then(ok).catch(function(){{ prompt("Скопируйте:", txt); }});
    }} else {{ prompt("Скопируйте:", txt); }}
  }}
  var tpls={{
    job: "["+city+"] Ищу работу / подработку: …\\nОпыт: …\\nГрафик: …\\nПишите в чат города или в личку.",
    svc: "["+city+"] Предлагаю услугу: …\\nУсловия: …\\nПишите в чат города.",
    rent: "["+city+"] Ищу жильё: комната/квартира, срок …\\nБюджет: …\\nПишите в чат города.",
    offer: "["+city+"] Предлагаю: жильё / транспорт / перевозка\\nДетали: …\\nПишите в чат города."
  }};
  document.querySelectorAll(".adTpl").forEach(function(b){{
    b.onclick=function(){{ copyTxt(tpls[b.getAttribute("data-tpl")]||tpls.job); }};
  }});
}})();
</script>
<div class="card" id="city-stats" style="margin:12px 0">
  <p class="tag" style="margin:0 0 8px">Сводка по городу</p>
  <div id="statFilters" style="display:flex;flex-wrap:wrap;gap:6px;margin-bottom:10px">
    <button type="button" class="cta2 sf" data-f="all" style="padding:6px 10px;font-size:.8rem">Все</button>
    <button type="button" class="cta2 sf" data-f="Профессии" style="padding:6px 10px;font-size:.8rem">Работа</button>
    <button type="button" class="cta2 sf" data-f="Жильё" style="padding:6px 10px;font-size:.8rem">Жильё</button>
    <button type="button" class="cta2 sf" data-f="Транспорт" style="padding:6px 10px;font-size:.8rem">Транспорт</button>
    <button type="button" class="cta2 sf" data-f="Услуги" style="padding:6px 10px;font-size:.8rem">Услуги</button>
    <button type="button" class="cta2 sf" data-f="Бизнес" style="padding:6px 10px;font-size:.8rem">Бизнес</button>
    <button type="button" class="cta2 sf" data-f="Финансы" style="padding:6px 10px;font-size:.8rem">Финансы</button>
  </div>
  <div id="statsBody"><p class="muted">Загрузка…</p></div>
</div>
<script>
(function(){{
  var el=document.getElementById("statsBody");
  var city={city_js};
  var allRows=[];
  var filter="all";
  if(!el||!city) return;
  function paint(){{
    var rows=allRows;
    if(filter!=="all"){{
      rows=allRows.filter(function(x){{ return (x.category||"")===filter; }});
    }}
    if(!rows.length){{
      el.innerHTML="<p class=\"muted\">Нет отметок в этом фильтре. Откройте категорию ниже и отметьте пункты.</p>";
      return;
    }}
    var html="";
    rows.slice(0,20).forEach(function(x){{
      html+="<div class=\"item\" style=\"display:flex;justify-content:space-between;gap:8px\"><span><b>"+
        (x.item||"")+"</b><div class=\"muted\">"+(x.category||"")+"</div></span><span style=\"opacity:.9\">"+x.n+"</span></div>";
    }});
    el.innerHTML=html;
  }}
  document.querySelectorAll(".sf").forEach(function(b){{
    b.onclick=function(){{ filter=b.getAttribute("data-f")||"all"; paint(); }};
  }});
  fetch("/api/stats?city="+encodeURIComponent(city))
    .then(function(r){{return r.json();}})
    .then(function(rows){{
      allRows=rows||[];
      paint();
    }})
    .catch(function(){{
      el.innerHTML="<p class=\"muted\">Не удалось загрузить сводку</p>";
    }});
}})();
</script>
<p class="tag" style="margin-top:16px">Категории</p>
<div class="card">{cats}</div>
<a class="cta2" href="/app/me">👤 Профиль</a>"#,
            ci = ci,
            si = si,
            sname = sname,
            cname = cname,
            city = city,
            chat = chat,
            cats = cats,
            city_js = format!("{:?}", city),
        ),
    )
}


async fn app_cat(Path((ci, si, zi, k)): Path<(usize, usize, usize, usize)>) -> Html<String> {
    let w = world();
    let Some((cname, countries)) = w.iter().nth(ci) else {
        return shell("Ошибка", r#"<a class="cta2" href="/app/">Назад</a>"#);
    };
    let Some((sname, cities)) = countries.iter().nth(si) else {
        return shell("Ошибка", &format!(r#"<a class="cta2" href="/app/{ci}">Назад</a>"#));
    };
    let Some(city) = cities.get(zi).copied() else {
        return shell("Ошибка", &format!(r#"<a class="cta2" href="/app/{ci}/{si}">Назад</a>"#));
    };
    let cats = categories();
    let Some((title, icon, items)) = cats.get(k) else {
        return shell("Ошибка", &format!(r#"<a class="cta2" href="/app/{ci}/{si}/{zi}">Назад</a>"#));
    };

    let mut list = String::new();
    for it in items {
        list.push_str(&format!(
            r#"<label class="item" style="cursor:pointer;display:flex;align-items:center;gap:10px">
<input type="checkbox" data-item="{it}" onchange="rmToggle(this.dataset.item,this.checked)"/> {it}</label>"#,
            it = it
        ));
    }

    shell(
        title,
        &format!(
            r#"<p class="tag"><a href="/app/{ci}/{si}/{zi}" style="color:#6ab2f2;text-decoration:none">← {city}</a> · {sname}</p>
<script>window.__rmCity="{city}";window.__rmCat="{title}";document.addEventListener("DOMContentLoaded",function(){{try{{rmRestore();}}catch(e){{}}}});</script>
<h1 style="display:flex;align-items:center;gap:8px"><img src="{icon}" width="22" height="22" alt=""/> {title}</h1>
<p class="sub">Отметьте, что актуально для вас в этом городе</p>
<div class="card">{list}</div>
<a class="cta2" href="/app/me">👤 Профиль</a>"#,
            ci = ci,
            si = si,
            zi = zi,
            city = city,
            sname = sname,
            title = title,
            icon = icon,
            list = list,
        ),
    )
}

#[derive(Deserialize)]
struct SearchQ {
    q: Option<String>,
}

async fn app_search(Query(q): Query<SearchQ>) -> Html<String> {
    let qstr = q.q.unwrap_or_default();
    let qn = qstr.trim().to_lowercase();
    let mut hits = String::new();
    if qn.chars().count() >= 2 {
        let w = world();
        for (ci, (cname, countries)) in w.iter().enumerate() {
            for (si, (sname, cities)) in countries.iter().enumerate() {
                for (zi, city) in cities.iter().enumerate() {
                    if city.to_lowercase().contains(&qn) {
                        hits.push_str(&format!(
                            r#"<a class="item" href="/app/{ci}/{si}/{zi}">📍 {city}<div class="muted">{sname} · {cname}</div></a>"#
                        ));
                    }
                }
            }
        }
        if hits.is_empty() {
            hits = r#"<p class="muted">Ничего не найдено. Попробуйте «Париж», «Берлин», «Грозный».</p>"#.into();
        }
    } else if !qstr.is_empty() {
        hits = r#"<p class="muted">Введите минимум 2 буквы</p>"#.into();
    } else {
        hits = r#"<p class="muted">Например: Ницца, Баку, Стамбул</p>"#.into();
    }
    shell(
        "Поиск",
        &format!(
            r#"<h1>Поиск города</h1>
<form action="/app/search" method="get">
<input class="search" type="search" name="q" value="{qstr}" placeholder="Название города…" autofocus/>
<button class="cta" type="submit">Найти</button>
</form>
{hits}"#
        ),
    )
}

async fn health(State(st): State<AppState>) -> String {
    let db_ok = st.db.lock().map(|c| c.execute_batch("SELECT 1").is_ok()).unwrap_or(false);
    if db_ok {
        "ok".into()
    } else {
        "degraded".into()
    }
}


async fn app_me() -> Html<String> {
    shell(
        "Профиль",
        r##"<div class="card" style="background:linear-gradient(145deg,#1a2a3c,#15202b)">
  <div style="display:flex;align-items:center;gap:12px">
    <div style="width:52px;height:52px;border-radius:14px;background:#2a6df4;display:flex;align-items:center;justify-content:center;font-size:1.5rem">👤</div>
    <div>
      <h1 style="margin:0;font-size:1.25rem" id="pname">Профиль</h1>
      <p class="tag" style="margin:4px 0 0" id="who">…</p>
    </div>
  </div>
  <div id="pointsBox" style="margin-top:12px;padding:12px;border-radius:12px;background:#1a1a22">
    <div style="font-weight:600">⚡ Ресурсы</div>
    <div style="margin-top:4px">Доступно: <b id="ptsAvail">—</b> · За задания: <b id="ptsLock">—</b></div>
    <div class="muted" style="font-size:.8rem;margin-top:6px" id="ptsTasks"></div>
  </div>
  <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-top:14px">
    <div style="background:#0e1621;border-radius:10px;padding:10px;text-align:center">
      <div style="font-size:1.3rem;font-weight:700" id="cntMarks">0</div>
      <div class="muted" style="font-size:.8rem">отметок</div>
    </div>
    <div style="background:#0e1621;border-radius:10px;padding:10px;text-align:center">
      <div style="font-size:1.3rem;font-weight:700" id="cntCities">0</div>
      <div class="muted" style="font-size:.8rem">городов</div>
    </div>
  </div>
</div>

<div class="card">
  <div style="font-weight:600;margin-bottom:8px">⚡ Быстрые действия</div>
  <a class="item" href="/app/">🌍 Континенты / карта</a>
  <a class="item" href="/app/search">🔍 Поиск города</a>
  <a class="item" href="/app/me" onclick="location.reload();return false;">🔄 Обновить профиль</a>
</div>

<div class="card">
  <div style="font-weight:600;margin-bottom:8px">🤝 Связь (по желанию)</div>
  <p class="muted" style="font-size:.85rem;margin:0 0 10px">Ник не светится всем списком. В городе виден только счётчик «открыты к связи». Договорённости — в чате города.</p>
  <label style="display:flex;gap:8px;align-items:center;margin:8px 0">
    <input type="checkbox" id="openContact"/> Открыт к связи
  
  <label class="muted" style="display:block;margin-top:12px">Сейчас актуально (до 14 дней)</label>
  <input id="intentText" maxlength="120" placeholder="Напр.: ищу работу водителем / сдаю комнату" style="width:100%;padding:10px;border-radius:10px;border:1px solid #333;background:#111;color:#eee;margin-top:4px"/>
  <p class="muted" id="intentUntil" style="margin:6px 0 0;font-size:.8rem"></p>
</label>
  <input id="tgUser" placeholder="@username" maxlength="32"
    style="width:100%;padding:10px;border-radius:10px;border:1px solid #333;background:#111;color:#eee;box-sizing:border-box"/>
  <button type="button" class="cta2" id="saveProfile" style="margin-top:8px;width:100%;text-align:center">Сохранить</button>
  <p class="muted" id="profMsg" style="margin-top:6px"></p>
</div>
<script>
(function(){
  function cid(){
    try{
      var id=window.Telegram&&Telegram.WebApp&&Telegram.WebApp.initDataUnsafe&&Telegram.WebApp.initDataUnsafe.user&&Telegram.WebApp.initDataUnsafe.user.id;
      if(id) return "tg:"+id;
    }catch(e){}
    var k="rm_cid", c=localStorage.getItem(k);
    if(!c){ c="web_"+Math.random().toString(36).slice(2)+Date.now().toString(36); localStorage.setItem(k,c); }
    return c;
  }
  function initData(){ try{ return (window.Telegram&&Telegram.WebApp&&Telegram.WebApp.initData)||""; }catch(e){ return ""; } }
  var openEl=document.getElementById("openContact");
  var userEl=document.getElementById("tgUser");
  var msg=document.getElementById("profMsg");
  fetch("/api/profile?client_id="+encodeURIComponent(cid())+"&init_data="+encodeURIComponent(initData()))
    .then(function(r){return r.json();})
    .then(function(p){
      if(openEl) openEl.checked=!!p.open_contact;
      if(userEl&&p.username) userEl.value="@"+p.username;
      var it=document.getElementById("intentText");
      var iu=document.getElementById("intentUntil");
      if(it) it.value=p.intent_text||"";
      if(iu){
        if(p.intent_until){
          var d=new Date(p.intent_until*1000);
          iu.textContent="Актуально до: "+d.toLocaleDateString();
        } else iu.textContent="";
      }
    }).catch(function(){});
  var btn=document.getElementById("saveProfile");
  if(btn) btn.onclick=function(){
    fetch("/api/profile",{
      method:"POST",
      headers:{"Content-Type":"application/json"},
      body:JSON.stringify({
        client_id:cid(),
        init_data:initData(),
        username:(userEl&&userEl.value)||"",
        open_contact:!!(openEl&&openEl.checked),
        intent_text:(document.getElementById("intentText")&&document.getElementById("intentText").value)||""
      })
    }).then(function(r){return r.json();})
      .then(function(x){
        if(msg) msg.textContent=x.ok?"Сохранено":"Ошибка";
        if(x.ok){
          var it=document.getElementById("intentText");
          var iu=document.getElementById("intentUntil");
          if(iu){
            var txt=(it&&it.value||"").trim();
            if(txt){
              var d=new Date(Date.now()+14*24*3600*1000);
              iu.textContent="Актуально до: "+d.toLocaleDateString();
            } else iu.textContent="";
          }
        }
      })
      .catch(function(){ if(msg) msg.textContent="Ошибка сети"; });
  };
})();
</script>

<div class="card" id="lastCityBox" style="display:none">
  <div style="font-weight:600;margin-bottom:6px">📍 Последний город</div>
  <div id="lastCity" class="muted">—</div>
  <div id="lastCityLink" style="margin-top:8px"></div>
</div>

<div class="card">
  <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;gap:8px;flex-wrap:wrap">
    <div style="font-weight:600">✅ Мои отметки</div>
    <button type="button" id="btnClear" style="background:#3a2030;color:#ffb4b4;border:1px solid #5a3040;border-radius:8px;padding:6px 10px;font-size:.85rem;cursor:pointer">Очистить всё</button>
  </div>
  <p class="sub" style="margin-top:0">Снимите галочку, чтобы убрать один пункт. «Очистить всё» — после подтверждения.</p>
  <div id="list"><p class="muted">Загрузка…</p></div>
</div>

<div class="card">
  <div style="font-weight:600;margin-bottom:6px">ℹ️ Подсказка</div>
  <p class="muted" style="margin:0">Отметки ставятся в городе → категория. Они видны в статистике города и здесь в кабинете.</p>
</div>

<script>
(function(){
  function esc(s){return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/"/g,"&quot;");}
  var cid = (typeof rmClient==="function") ? rmClient() : "";
  try{
    document.getElementById("who").textContent = "ID: " + cid;
    fetch("/api/points?client_id="+encodeURIComponent(cid)).then(function(r){return r.json();}).then(function(p){
      var a=document.getElementById("ptsAvail"); if(a) a.textContent=String(p.available);
      var l=document.getElementById("ptsLock"); if(l) l.textContent=String(p.locked);
      var t=document.getElementById("ptsTasks");
      if(t){
        var s="";
        s+=(p.unlocked_marks? "✓":"○")+" Поставьте 3 отметки в категориях (+40)\n";
        s+=(p.unlocked_city? "✓":"○")+" Отметьте что-то в городе (+40)\n";
        s+="○ Вступить в главный чат — скоро (+20)";
        t.textContent=s;
      }
    }).catch(function(){});
    var u = window.Telegram && Telegram.WebApp && Telegram.WebApp.initDataUnsafe && Telegram.WebApp.initDataUnsafe.user;
    if(u){
      var nm = [u.first_name, u.last_name].filter(Boolean).join(" ") || ("@"+(u.username||""));
      if(nm) document.getElementById("pname").textContent = nm;
    }
  }catch(e){}

  function fromLocal(){
    var rows=[];
    try{
      for(var i=0;i<localStorage.length;i++){
        var key=localStorage.key(i);
        if(!key||key.indexOf("rm:")!==0) continue;
        var rest=key.slice(3);
        var p=rest.indexOf("|");
        if(p<0) continue;
        var city=rest.slice(0,p), cat=rest.slice(p+1);
        var arr=[]; try{arr=JSON.parse(localStorage.getItem(key)||"[]");}catch(e){}
        if(!Array.isArray(arr)) continue;
        arr.forEach(function(it){ if(it) rows.push({city:city,cat:cat,item:String(it)}); });
      }
    }catch(e){}
    return rows;
  }

  function render(rows){
    var el=document.getElementById("list");
    var cities={};
    rows.forEach(function(x){ cities[x.city]=1; });
    document.getElementById("cntMarks").textContent = String(rows.length);
    document.getElementById("cntCities").textContent = String(Object.keys(cities).length);

    var last = localStorage.getItem("rm_last_city") || (rows[0] && rows[0].city) || "";
    if(last){
      document.getElementById("lastCityBox").style.display="block";
      document.getElementById("lastCity").textContent = last;
      document.getElementById("lastCityLink").innerHTML =
        "<a class=\"cta2\" href=\"/app/search?q="+encodeURIComponent(last)+"\">Найти «"+esc(last)+"» в поиске</a>";
    }

    if(!rows.length){
      el.innerHTML="<p class=\"muted\">Пока пусто. Откройте город → категорию → поставьте галочки.</p>";
      return;
    }
    // group by city
    var by={};
    rows.forEach(function(x){
      if(!by[x.city]) by[x.city]=[];
      by[x.city].push(x);
    });
    var html="";
    Object.keys(by).sort().forEach(function(city){
      html+="<div style=\"margin:12px 0 6px;color:#9ec1ff;font-weight:600\">📍 "+esc(city)+"</div>";
      by[city].forEach(function(x){
        html+="<label class=\"item\" style=\"cursor:pointer;display:flex;gap:12px;align-items:flex-start\">"+
          "<input type=\"checkbox\" checked style=\"width:20px;height:20px;margin-top:3px;flex-shrink:0\" "+
          "data-city=\""+esc(x.city)+"\" data-cat=\""+esc(x.cat)+"\" data-item=\""+esc(x.item)+"\"/>"+
          "<span><b>"+esc(x.item)+"</b><div class=\"muted\">"+esc(x.cat)+"</div></span></label>";
      });
    });
    el.innerHTML=html;
    el.querySelectorAll("input[type=checkbox]").forEach(function(cb){
      cb.onchange=function(){
        if(cb.checked) return;
        window.__rmCity=cb.dataset.city;
        window.__rmCat=cb.dataset.cat;
        if(typeof rmToggle==="function") rmToggle(cb.dataset.item,false);
        var lab=cb.closest("label"); if(lab) lab.remove();
        // recount
        var left=[];
        el.querySelectorAll("input[type=checkbox]").forEach(function(x){
          left.push({city:x.dataset.city,cat:x.dataset.cat,item:x.dataset.item});
        });
        if(!left.length) el.innerHTML="<p class=\"muted\">Пусто</p>";
        document.getElementById("cntMarks").textContent=String(left.length);
        var c={}; left.forEach(function(x){c[x.city]=1;});
        document.getElementById("cntCities").textContent=String(Object.keys(c).length);
      };
    });
  }

  function clearAll(){
    if(!confirm("Удалить ВСЕ ваши отметки? Это нельзя отменить.")) return;
    var rows = fromLocal();
    rows.forEach(function(x){
      window.__rmCity=x.city; window.__rmCat=x.cat;
      if(typeof rmToggle==="function") rmToggle(x.item,false);
    });
    // wipe keys
    if(typeof rmClearLocal==='function') rmClearLocal();
    render([]);
  }

  document.getElementById("btnClear").onclick = clearAll;

  var localRows=(typeof rmAllLocal==='function')?rmAllLocal():fromLocal();
  if(localRows.length) render(localRows);
  fetch("/api/my?client_id="+encodeURIComponent(cid))
    .then(function(r){return r.json();})
    .then(function(rows){
      if(!rows||!rows.length){ if(!localRows.length) render([]); return; }
      render(rows.map(function(x){ return {city:x.city,cat:x.category,item:x.item}; }));
    })
    .catch(function(){ if(!localRows.length) render([]); });
})();
</script>"##,
    )
}


#[derive(Deserialize)]
struct VoteIn {
    client_id: String,
    city: String,
    category: String,
    item: String,
    on: bool,
    #[serde(default)]
    init_data: String,
}

#[derive(Serialize)]
struct OkMsg { ok: bool }


#[derive(serde::Serialize)]
struct PointsOut {
    locked: i64,
    available: i64,
    unlocked_marks: i64,
    unlocked_city: i64,
}

async fn api_points(State(st): State<AppState>, Query(q): Query<std::collections::HashMap<String, String>>) -> Json<PointsOut> {
    let cid = q.get("client_id").cloned().unwrap_or_default();
    if cid.is_empty() {
        return Json(PointsOut { locked: 0, available: 0, unlocked_marks: 0, unlocked_city: 0 });
    }
    let conn = st.db.lock().expect("db");
    ensure_balance(&conn, &cid);
    unlock_for_activity(&conn, &cid);
    let (locked, available, um, uc): (i64, i64, i64, i64) = conn
        .query_row(
            "SELECT points_locked, points_available, unlocked_marks, unlocked_city FROM balances WHERE client_id=?1",
            [&cid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap_or((100, 0, 0, 0));
    Json(PointsOut { locked, available, unlocked_marks: um, unlocked_city: uc })
}

async fn api_vote(State(st): State<AppState>, Json(v): Json<VoteIn>) -> Json<OkMsg> {
    let token = std::env::var("TELEGRAM_BOT_TOKEN").or_else(|_| std::env::var("BOT_TOKEN")).unwrap_or_default();
    let cid = if let Some(uid) = webapp_user_id(&v.init_data, &token) {
        format!("tg:{uid}")
    } else {
        // without valid initData only allow anonymous client_id that does NOT pretend to be tg:
        let c = v.client_id.chars().take(64).collect::<String>();
        if c.starts_with("tg:") {
            return Json(OkMsg { ok: false });
        }
        c
    };
    if cid.len() < 8 || v.city.is_empty() || v.item.is_empty() {
        return Json(OkMsg { ok: false });
    }
    let db = st.db.lock().expect("db");
    if v.on {
        let _ = db.execute(
            "INSERT OR IGNORE INTO votes (client_id, city, category, item) VALUES (?1,?2,?3,?4)",
            params![cid, v.city, v.category, v.item],
        );
    } else {
        let _ = db.execute(
            "DELETE FROM votes WHERE client_id=?1 AND city=?2 AND category=?3 AND item=?4",
            params![cid, v.city, v.category, v.item],
        );
    }
    unlock_for_activity(&db, &cid);
    Json(OkMsg { ok: true })
}

#[derive(Deserialize)]
struct StatsQ { city: String }

#[derive(Serialize)]
struct StatRow { category: String, item: String, n: i64 }

async fn api_stats(State(st): State<AppState>, Query(q): Query<StatsQ>) -> Json<Vec<StatRow>> {
    let city = q.city.chars().take(80).collect::<String>();
    if city.is_empty() {
        return Json(vec![]);
    }
    let db = match st.db.lock() {
        Ok(g) => g,
        Err(_) => return Json(vec![]),
    };
    let mut stmt = match db.prepare(
        "SELECT category, item, COUNT(*) FROM votes WHERE city=?1 GROUP BY category, item ORDER BY COUNT(*) DESC LIMIT 100",
    ) {
        Ok(s) => s,
        Err(_) => return Json(vec![]),
    };
    let rows = stmt
        .query_map(params![city], |r| {
            Ok(StatRow {
                category: r.get(0)?,
                item: r.get(1)?,
                n: r.get(2)?,
            })
        })
        .ok()
        .map(|it| it.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    Json(rows)
}



#[derive(Deserialize)]
struct MyQ {
    client_id: String,
}

#[derive(Serialize)]
struct MyRow {
    city: String,
    category: String,
    item: String,
}

async fn api_my(State(st): State<AppState>, Query(q): Query<MyQ>) -> Json<Vec<MyRow>> {
    let cid = q.client_id.chars().take(64).collect::<String>();
    if cid.len() < 4 {
        return Json(vec![]);
    }
    let db = st.db.lock().expect("db");
    let mut stmt = match db.prepare(
        "SELECT city, category, item FROM votes WHERE client_id=?1 ORDER BY city, category, item",
    ) {
        Ok(s) => s,
        Err(_) => return Json(vec![]),
    };
    let rows = stmt
        .query_map(params![cid], |r| {
            Ok(MyRow {
                city: r.get(0)?,
                category: r.get(1)?,
                item: r.get(2)?,
            })
        })
        .ok()
        .map(|it| it.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();
    Json(rows)
}

#[tokio::main]
async fn main() {
    let state = AppState { db: std::sync::Arc::new(Mutex::new(db_open())) };
    
#[derive(Deserialize)]
struct ProfileQ {
    client_id: Option<String>,
    init_data: Option<String>,
}

#[derive(Serialize)]
struct ProfileOut {
    username: String,
    open_contact: bool,
    #[serde(default)]
    intent_text: String,
    #[serde(default)]
    intent_until: i64,
}

fn resolve_cid(client_id: &str, init_data: &str) -> String {
    let token = std::env::var("TELEGRAM_BOT_TOKEN")
        .or_else(|_| std::env::var("BOT_TOKEN"))
        .unwrap_or_default();
    if let Some(uid) = webapp_user_id(init_data, &token) {
        return format!("tg:{uid}");
    }
    let c: String = client_id.chars().take(64).collect();
    if c.starts_with("tg:") {
        return String::new();
    }
    c
}

async fn api_profile_get(State(st): State<AppState>, Query(q): Query<ProfileQ>) -> Json<ProfileOut> {
    let cid = resolve_cid(
        q.client_id.as_deref().unwrap_or(""),
        q.init_data.as_deref().unwrap_or(""),
    );
    if cid.len() < 8 {
        return Json(ProfileOut { username: String::new(), open_contact: false, intent_text: String::new(), intent_until: 0 });
    }
    let db = st.db.lock().expect("db");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    match db.query_row(
        "SELECT username, open_contact, intent_text, intent_until FROM profiles WHERE client_id=?1",
        [&cid],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?)),
    ) {
        Ok((u, o, it, until)) => {
            let (it, until) = if until > now {
                (it, until)
            } else {
                (String::new(), 0)
            };
            Json(ProfileOut {
                username: u,
                open_contact: o != 0,
                intent_text: it,
                intent_until: until,
            })
        }
        Err(_) => Json(ProfileOut {
            username: String::new(),
            open_contact: false,
            intent_text: String::new(),
            intent_until: 0,
        }),
    }
}

#[derive(Deserialize)]
struct ProfileIn {
    client_id: String,
    #[serde(default)]
    init_data: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    open_contact: bool,
    #[serde(default)]
    intent_text: String,
}

async fn api_profile_set(State(st): State<AppState>, Json(v): Json<ProfileIn>) -> Json<OkMsg> {
    let cid = resolve_cid(&v.client_id, &v.init_data);
    if cid.len() < 8 {
        return Json(OkMsg { ok: false });
    }
    let mut user: String = v.username.trim().trim_start_matches('@').chars().take(32).collect();
    user = user.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
    let open = if v.open_contact { 1i64 } else { 0 };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let db = st.db.lock().expect("db");
    let intent: String = v
        .intent_text
        .chars()
        .take(120)
        .collect::<String>()
        .trim()
        .to_string();
    let intent_until = if intent.is_empty() { 0i64 } else { now + 14 * 24 * 3600 };
    let _ = db.execute(
        "INSERT INTO profiles (client_id, username, open_contact, updated_at, intent_text, intent_until)
         VALUES (?1,?2,?3,?4,?5,?6)
         ON CONFLICT(client_id) DO UPDATE SET
           username=excluded.username,
           open_contact=excluded.open_contact,
           updated_at=excluded.updated_at,
           intent_text=excluded.intent_text,
           intent_until=excluded.intent_until",
        rusqlite::params![cid, user, open, now, intent, intent_until],
    );
    Json(OkMsg { ok: true })
}

#[derive(Serialize)]
struct OpenCountOut { n: i64 }

#[derive(Deserialize)]
struct OpenCountQ { city: String }

async fn api_open_count(State(st): State<AppState>, Query(q): Query<OpenCountQ>) -> Json<OpenCountOut> {
    let city: String = q.city.chars().take(80).collect();
    if city.is_empty() {
        return Json(OpenCountOut { n: 0 });
    }
    let db = st.db.lock().expect("db");
    let n: i64 = db
        .query_row(
            "SELECT COUNT(DISTINCT v.client_id) FROM votes v
             INNER JOIN profiles p ON p.client_id = v.client_id
             WHERE v.city = ?1 AND p.open_contact = 1",
            [&city],
            |r| r.get(0),
        )
        .unwrap_or(0);
    Json(OpenCountOut { n })
}


    let app = Router::new()
        .route("/", get(home))
        .route("/app", get(app_root))
        .route("/app/", get(app_root))
        .route("/app/search", get(app_search))
        .route("/app/me", get(app_me))
        .route("/app/{ci}", get(app_continent))
        .route("/app/{ci}/{si}", get(app_country))
        .route("/app/{ci}/{si}/{zi}", get(app_city))
        .route("/app/{ci}/{si}/{zi}/cat/{k}", get(app_cat))
        .route("/health", get(health))
        .route("/api/vote", post(api_vote))
        .route("/api/points", get(api_points))
        .route("/api/my", get(api_my))
        .route("/api/stats", get(api_stats))
        .route("/api/profile", get(api_profile_get).post(api_profile_set))
        .route("/api/open_count", get(api_open_count))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("resursmap listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("db");
    axum::serve(listener, app).await.expect("db");
}
