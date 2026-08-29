pub struct AdminSecurityData {
    pub masked_email: String,
    pub verified: bool,
    pub remaining_seconds: i64,
    pub message: String,
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn render_admin_security(data: AdminSecurityData) -> String {
    let state_class = if data.verified { "verified" } else { "pending" };

    let state_title = if data.verified {
        "Сессия подтверждена"
    } else {
        "Требуется подтверждение"
    };

    let state_text = if data.verified {
        format!(
            "Разрешение критических действий действует ещё около {} минут.",
            (data.remaining_seconds + 59) / 60
        )
    } else {
        "Критические действия пока заблокированы.".to_string()
    };

    let message = if data.message.is_empty() {
        String::new()
    } else {
        format!(
            r#"<div class="message">{}</div>"#,
            escape_html(&data.message)
        )
    };

    format!(
        r#"<!doctype html>
<html lang="ru">
<head>
<meta charset="utf-8">
<meta name="viewport"
 content="width=device-width,initial-scale=1,viewport-fit=cover">
<meta name="color-scheme" content="dark">
<title>Безопасность владельца · ResursMap</title>
<style>
:root {{
 --gold:#dfc07f;--green:#62e0ad;--orange:#f3a94f;
 --text:#f5f2eb;--muted:#959ba8;--line:rgba(223,192,127,.2);
}}
*{{box-sizing:border-box}}
body{{
 margin:0;min-height:100vh;
 padding:max(18px,env(safe-area-inset-top)) 16px
 max(34px,env(safe-area-inset-bottom));
 color:var(--text);
 background:
 radial-gradient(circle at 10% 0%,rgba(37,116,85,.18),transparent 36%),
 radial-gradient(circle at 100% 8%,rgba(97,70,160,.18),transparent 34%),
 linear-gradient(155deg,#090b10,#050609);
 font-family:Inter,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif
}}
.page{{width:min(700px,100%);margin:auto}}
.back{{
 min-height:44px;display:inline-flex;align-items:center;
 padding:0 16px;margin-bottom:18px;border:1px solid var(--line);
 border-radius:14px;color:var(--text);text-decoration:none;font-weight:850
}}
.hero,.card{{
 border:1px solid var(--line);border-radius:25px;
 background:linear-gradient(145deg,rgba(21,24,32,.97),rgba(10,12,17,.98));
 box-shadow:0 22px 65px rgba(0,0,0,.3)
}}
.hero{{
 padding:27px;
 background:
 linear-gradient(135deg,rgba(223,192,127,.1),transparent 44%),
 linear-gradient(145deg,#181a21,#0b0d12)
}}
.kicker{{
 color:var(--gold);font-size:11px;font-weight:950;letter-spacing:.18em
}}
h1{{
 margin:13px 0 10px;font-size:clamp(34px,8vw,55px);
 line-height:1;letter-spacing:-.045em
}}
p{{color:var(--muted);line-height:1.6}}
.state{{margin-top:22px;padding:18px;border-radius:18px}}
.state.verified{{
 color:var(--green);border:1px solid rgba(98,224,173,.3);
 background:rgba(98,224,173,.07)
}}
.state.pending{{
 color:var(--orange);border:1px solid rgba(243,169,79,.3);
 background:rgba(243,169,79,.07)
}}
.state strong{{display:block;font-size:18px}}
.state span{{display:block;margin-top:6px;color:var(--muted);font-size:13px}}
.card{{margin-top:15px;padding:22px}}
.card h2{{margin:0;font-size:21px}}
.email{{
 margin:15px 0;padding:14px;border-radius:14px;
 color:var(--gold);background:rgba(223,192,127,.07);font-weight:850
}}
button,input{{
 width:100%;min-height:48px;border-radius:14px;font:inherit
}}
button{{
 border:1px solid rgba(223,192,127,.36);
 color:#17120a;background:linear-gradient(135deg,#f0d99f,#c9a65f);
 font-weight:950;cursor:pointer
}}
input{{
 margin-bottom:10px;padding:0 16px;
 border:1px solid rgba(255,255,255,.13);
 outline:none;color:var(--text);background:rgba(255,255,255,.035);
 text-align:center;font-size:24px;font-weight:900;letter-spacing:.28em
}}
.message{{
 margin-top:15px;padding:14px;border-radius:14px;
 color:var(--green);border:1px solid rgba(98,224,173,.25);
 background:rgba(98,224,173,.06)
}}
.notice{{margin-top:15px;color:var(--muted);font-size:12px;line-height:1.55}}
</style>
</head>
<body>
<main class="page">
<a class="back" href="/app/center">← Центр управления</a>

<section class="hero">
 <div class="kicker">ResursMap · Безопасность</div>
 <h1>Защищённая сессия</h1>
 <p>
  Повторное подтверждение личности перед управлением
  администраторами, финансами и аварийными функциями.
 </p>
 <div class="state {state_class}">
  <strong>{state_title}</strong>
  <span>{state_text}</span>
 </div>
 {message}
</section>

<section class="card">
 <h2>Получить одноразовый код</h2>
 <p>Код будет отправлен на подтверждённый адрес владельца.</p>
 <div class="email">{masked_email}</div>
 <form method="post" action="/app/center/security/request">
  <button type="submit">Отправить защищённый код</button>
 </form>
</section>

<section class="card">
 <h2>Подтвердить сессию</h2>
 <p>Введите шестизначный код. Доступно не более пяти попыток.</p>
 <form method="post" action="/app/center/security/verify">
  <input name="code" inputmode="numeric"
   autocomplete="one-time-code" pattern="[0-9]{{6}}"
   maxlength="6" required aria-label="Шестизначный код">
  <button type="submit">Подтвердить сессию</button>
 </form>
 <div class="notice">
  Код действует 10 минут и привязан только к текущей
  admin-сессии. Подтверждение критических действий действует
  15 минут.
 </div>
</section>
</main>
</body>
</html>"#,
        state_class = state_class,
        state_title = state_title,
        state_text = escape_html(&state_text),
        message = message,
        masked_email = escape_html(&data.masked_email),
    )
}
