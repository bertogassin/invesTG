use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct StripeCheckoutSession {
    pub id: String,
    pub url: String,
}

pub fn stripe_secret_key() -> Option<String> {
    std::env::var("STRIPE_SECRET_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| value.starts_with("sk_"))
}

pub fn stripe_webhook_secret() -> Option<String> {
    std::env::var("STRIPE_WEBHOOK_SECRET")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| value.starts_with("whsec_"))
}

pub fn stripe_configured() -> bool {
    stripe_secret_key().is_some()
}

pub fn mock_promotion_payment_allowed() -> bool {
    if stripe_configured() {
        return false;
    }

    match std::env::var("ALLOW_MOCK_PROMOTION_PAYMENT")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
    {
        Some(value) if matches!(value.as_str(), "1" | "true" | "yes") => true,
        Some(value) if matches!(value.as_str(), "0" | "false" | "no") => false,
        _ => {
            let base = public_base_url().to_ascii_lowercase();
            base.contains("localhost") || base.contains("127.0.0.1")
        }
    }
}

pub fn public_base_url() -> String {
    std::env::var("PUBLIC_BASE_URL")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| value.starts_with("http"))
        .unwrap_or_else(|| "https://resursmap.de".to_string())
}

pub async fn create_promotion_checkout_session(
    resource_id: i64,
    request_id: i64,
    user_id: i64,
    price_minor: i64,
    currency: &str,
    product_name: &str,
) -> Result<StripeCheckoutSession, String> {
    let secret = stripe_secret_key().ok_or_else(|| "stripe_not_configured".to_string())?;
    if price_minor <= 0 {
        return Err("invalid_price".into());
    }

    let base = public_base_url();
    let success_url = format!(
        "{base}/app/resource/{resource_id}/promote/paid/{request_id}?session_id={{CHECKOUT_SESSION_ID}}"
    );
    let cancel_url = format!("{base}/app/resource/{resource_id}/promote/pay/{request_id}");

    let params = [
        ("mode", "payment".to_string()),
        ("success_url", success_url),
        ("cancel_url", cancel_url),
        ("client_reference_id", format!("promo-{request_id}")),
        ("metadata[promotion_request_id]", request_id.to_string()),
        ("metadata[resource_id]", resource_id.to_string()),
        ("metadata[user_id]", user_id.to_string()),
        (
            "line_items[0][price_data][currency]",
            currency.to_ascii_lowercase(),
        ),
        (
            "line_items[0][price_data][product_data][name]",
            product_name.to_string(),
        ),
        (
            "line_items[0][price_data][unit_amount]",
            price_minor.to_string(),
        ),
        ("line_items[0][quantity]", "1".to_string()),
    ];

    let response = reqwest::Client::new()
        .post("https://api.stripe.com/v1/checkout/sessions")
        .bearer_auth(secret)
        .form(&params)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("stripe_api_error:{body}"));
    }

    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|error| error.to_string())?;

    let id = payload
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "stripe_missing_session_id".to_string())?
        .to_string();

    let url = payload
        .get("url")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "stripe_missing_checkout_url".to_string())?
        .to_string();

    Ok(StripeCheckoutSession { id, url })
}

pub async fn fetch_checkout_session(session_id: &str) -> Result<serde_json::Value, String> {
    let secret = stripe_secret_key().ok_or_else(|| "stripe_not_configured".to_string())?;
    let url = format!(
        "https://api.stripe.com/v1/checkout/sessions/{}",
        urlencoding::encode(session_id)
    );

    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(secret)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err("stripe_session_lookup_failed".into());
    }

    response.json().await.map_err(|error| error.to_string())
}

pub fn checkout_session_is_paid(session: &serde_json::Value) -> bool {
    session
        .get("payment_status")
        .and_then(|value| value.as_str())
        == Some("paid")
}

pub fn checkout_session_request_id(session: &serde_json::Value) -> Option<i64> {
    session
        .get("metadata")
        .and_then(|metadata| metadata.get("promotion_request_id"))
        .and_then(|value| value.as_str())
        .and_then(|value| value.parse::<i64>().ok())
}

pub fn checkout_session_user_id(session: &serde_json::Value) -> Option<i64> {
    session
        .get("metadata")
        .and_then(|metadata| metadata.get("user_id"))
        .and_then(|value| value.as_str())
        .and_then(|value| value.parse::<i64>().ok())
}

pub fn checkout_session_payment_reference(session: &serde_json::Value) -> String {
    session
        .get("payment_intent")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            session
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string()
        })
}

pub fn verify_webhook_signature(payload: &[u8], signature_header: &str, secret: &str) -> bool {
    let mut timestamp = "";
    let mut signature = "";

    for part in signature_header.split(',') {
        if let Some(value) = part.strip_prefix("t=") {
            timestamp = value;
        } else if let Some(value) = part.strip_prefix("v1=") {
            signature = value;
        }
    }

    if timestamp.is_empty() || signature.is_empty() {
        return false;
    }

    let timestamp_secs: i64 = match timestamp.parse() {
        Ok(value) => value,
        Err(_) => return false,
    };

    let now = chrono::Utc::now().timestamp();
    if (now - timestamp_secs).abs() > 300 {
        return false;
    }

    let signed_payload = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };

    mac.update(signed_payload.as_bytes());

    let expected = hex::encode(mac.finalize().into_bytes());
    expected == signature
}
