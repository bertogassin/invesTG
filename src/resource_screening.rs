#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreeningResult {
    pub passed: bool,
    pub reason: String,
}

fn count_links(text: &str) -> usize {
    text.split_whitespace()
        .filter(|token| {
            let lower = token.to_ascii_lowercase();
            lower.starts_with("http://")
                || lower.starts_with("https://")
                || lower.starts_with("www.")
        })
        .count()
}

fn contains_suspicious_link(text: &str) -> bool {
    const SUSPICIOUS: &[&str] = &[
        "bit.ly",
        "tinyurl",
        "t.me/joinchat",
        "clck.ru",
        "cutt.ly",
        "goo.gl",
        "rb.gy",
        "is.gd",
        "0day",
        "casino",
        "crypto-airdrop",
    ];

    let lower = text.to_ascii_lowercase();
    SUSPICIOUS.iter().any(|needle| lower.contains(needle))
}

pub fn screen_listing_content(title: &str, description: &str, contact: &str) -> ScreeningResult {
    let combined = format!("{title}\n{description}\n{contact}");

    if count_links(&combined) >= 4 {
        return ScreeningResult {
            passed: false,
            reason: "Слишком много ссылок в тексте".to_string(),
        };
    }

    if contains_suspicious_link(&combined) {
        return ScreeningResult {
            passed: false,
            reason: "Подозрительная ссылка в объявлении".to_string(),
        };
    }

    ScreeningResult {
        passed: true,
        reason: String::new(),
    }
}

pub fn listing_type_label(listing_type: &str) -> &'static str {
    match listing_type.trim() {
        "seeker" => "Ищу работу",
        "offer" => "Предложение",
        _ => "Объявление",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_job_seeker_passes() {
        let result = screen_listing_content(
            "Ищу работу электриком",
            "Добрый день. Ищу электрика в Ницце.",
            "@username",
        );
        assert!(result.passed);
    }

    #[test]
    fn suspicious_link_fails() {
        let result = screen_listing_content(
            "Test",
            "Смотри https://bit.ly/example",
            "",
        );
        assert!(!result.passed);
    }
}
