use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::db::pool::DbPool;
use crate::db::security::{
    claim_critical_escalation, claim_warning_slot, record_moderation_audit, update_persistent_risk,
    NewModerationAuditEvent, DEFAULT_WARNING_COOLDOWN_SECONDS,
};

use teloxide::prelude::*;
use teloxide::types::{ChatKind, ChatPermissions};

const FLOOD_WINDOW: Duration = Duration::from_secs(8);
const FLOOD_LIMIT: usize = 7;

const DUPLICATE_WINDOW: Duration = Duration::from_secs(30);

const RISK_DECAY_WINDOW: Duration = Duration::from_secs(600);
const RISK_HISTORY_WINDOW: Duration = Duration::from_secs(3600);

const RISK_LOW_THRESHOLD: u32 = 2;
const RISK_MEDIUM_THRESHOLD: u32 = 5;
const RISK_HIGH_THRESHOLD: u32 = 9;
const RISK_CRITICAL_THRESHOLD: u32 = 14;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityReason {
    Flood,
    DuplicateMessage,
    ExcessiveLinks,
    SuspiciousLink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Clean,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
struct LastMessage {
    text: String,
    at: Instant,
}

#[derive(Debug, Clone)]
struct ViolationEvent {
    at: Instant,
    weight: u32,
}

#[derive(Debug, Clone)]
struct RiskRecord {
    events: VecDeque<ViolationEvent>,
    last_decay_at: Instant,
}

static FLOOD_STATE: OnceLock<Mutex<HashMap<(i64, u64), VecDeque<Instant>>>> = OnceLock::new();

static DUPLICATE_STATE: OnceLock<Mutex<HashMap<(i64, u64), LastMessage>>> = OnceLock::new();

static RISK_STATE: OnceLock<Mutex<HashMap<(i64, u64), RiskRecord>>> = OnceLock::new();

fn flood_state() -> &'static Mutex<HashMap<(i64, u64), VecDeque<Instant>>> {
    FLOOD_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn duplicate_state() -> &'static Mutex<HashMap<(i64, u64), LastMessage>> {
    DUPLICATE_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn risk_state() -> &'static Mutex<HashMap<(i64, u64), RiskRecord>> {
    RISK_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[allow(clippy::too_many_arguments)]
fn audit_moderation_action(
    db_pool: &DbPool,
    chat_id: i64,
    user_id: i64,
    message_id: i32,
    action: &str,
    reason: &SecurityReason,
    risk_score: u32,
    risk_level: RiskLevel,
    critical_occurrence: u32,
    mute_seconds: i64,
    success: bool,
    created_at: i64,
) {
    let reason_text = format!("{:?}", reason);
    let risk_level_text = format!("{:?}", risk_level);

    let event = NewModerationAuditEvent {
        chat_id,
        user_id,
        message_id,
        action,
        reason: &reason_text,
        risk_score,
        risk_level: &risk_level_text,
        critical_occurrence,
        mute_seconds,
        success,
        created_at,
    };

    match record_moderation_audit(db_pool, &event) {
        Ok(audit_id) => {
            println!(
                "SECURITY_AUDIT chat_id={} user_id={} action={} success={} audit_id={}",
                chat_id, user_id, action, success, audit_id
            );
        }

        Err(err) => {
            eprintln!(
                "SECURITY_AUDIT_ERROR chat_id={} user_id={} action={} success={} error={:?}",
                chat_id, user_id, action, success, err
            );
        }
    }
}

fn is_group_message(msg: &Message) -> bool {
    matches!(msg.chat.kind, ChatKind::Public(_))
}

fn count_links(text: &str) -> usize {
    text.split_whitespace()
        .filter(|token| {
            let lower = token.to_ascii_lowercase();

            lower.starts_with("http://")
                || lower.starts_with("https://")
                || lower.starts_with("www.")
                || lower.starts_with("t.me/")
        })
        .count()
}

fn contains_suspicious_link(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();

    const SUSPICIOUS: &[&str] = &[
        "bit.ly/",
        "tinyurl.com/",
        "cutt.ly/",
        "tiny.one/",
        "rb.gy/",
        "rebrand.ly/",
    ];

    SUSPICIOUS.iter().any(|needle| lower.contains(needle))
}

fn detect_static_reason(text: &str) -> Option<SecurityReason> {
    if count_links(text) >= 4 {
        return Some(SecurityReason::ExcessiveLinks);
    }

    if contains_suspicious_link(text) {
        return Some(SecurityReason::SuspiciousLink);
    }

    None
}

fn reason_weight(reason: &SecurityReason) -> u32 {
    match reason {
        SecurityReason::DuplicateMessage => 1,
        SecurityReason::Flood => 2,
        SecurityReason::ExcessiveLinks => 3,
        SecurityReason::SuspiciousLink => 4,
    }
}

fn risk_level(score: u32) -> RiskLevel {
    if score >= RISK_CRITICAL_THRESHOLD {
        RiskLevel::Critical
    } else if score >= RISK_HIGH_THRESHOLD {
        RiskLevel::High
    } else if score >= RISK_MEDIUM_THRESHOLD {
        RiskLevel::Medium
    } else if score >= RISK_LOW_THRESHOLD {
        RiskLevel::Low
    } else {
        RiskLevel::Clean
    }
}

fn detect_flood(chat_id: i64, user_id: u64, now: Instant) -> bool {
    let Ok(mut state) = flood_state().lock() else {
        return false;
    };

    let queue = state
        .entry((chat_id, user_id))
        .or_insert_with(VecDeque::new);

    while let Some(front) = queue.front() {
        if now.duration_since(*front) > FLOOD_WINDOW {
            queue.pop_front();
        } else {
            break;
        }
    }

    queue.push_back(now);

    queue.len() >= FLOOD_LIMIT
}

fn detect_duplicate(chat_id: i64, user_id: u64, text: &str, now: Instant) -> bool {
    let normalized = text.trim();

    if normalized.len() < 6 {
        return false;
    }

    let Ok(mut state) = duplicate_state().lock() else {
        return false;
    };

    let key = (chat_id, user_id);

    let duplicate = state
        .get(&key)
        .map(|previous| {
            previous.text == normalized && now.duration_since(previous.at) <= DUPLICATE_WINDOW
        })
        .unwrap_or(false);

    state.insert(
        key,
        LastMessage {
            text: normalized.to_string(),
            at: now,
        },
    );

    duplicate
}

fn update_risk(
    chat_id: i64,
    user_id: u64,
    reason: &SecurityReason,
    now: Instant,
) -> (u32, RiskLevel) {
    let Ok(mut state) = risk_state().lock() else {
        return (0, RiskLevel::Clean);
    };

    let key = (chat_id, user_id);

    let record = state.entry(key).or_insert_with(|| RiskRecord {
        events: VecDeque::new(),
        last_decay_at: now,
    });

    while let Some(front) = record.events.front() {
        if now.duration_since(front.at) > RISK_HISTORY_WINDOW {
            record.events.pop_front();
        } else {
            break;
        }
    }

    if now.duration_since(record.last_decay_at) >= RISK_DECAY_WINDOW {
        if !record.events.is_empty() {
            record.events.pop_front();
        }

        record.last_decay_at = now;
    }

    record.events.push_back(ViolationEvent {
        at: now,
        weight: reason_weight(reason),
    });

    let score = record.events.iter().map(|event| event.weight).sum::<u32>();

    (score, risk_level(score))
}

async fn is_group_admin(bot: &Bot, msg: &Message, user_id: u64) -> bool {
    let Ok(admins) = bot.get_chat_administrators(msg.chat.id).await else {
        return false;
    };

    admins.iter().any(|member| member.user.id.0 == user_id)
}

async fn is_whitelisted(bot: &Bot, msg: &Message, user_id: u64) -> bool {
    if is_group_admin(bot, msg, user_id).await {
        return true;
    }

    false
}

pub fn is_security_candidate(msg: &Message) -> bool {
    if !is_group_message(msg) {
        return false;
    }

    msg.from.is_some() && msg.text().is_some()
}

fn should_send_warning(db_pool: &DbPool, chat_id: i64, user_id: u64, now_unix: i64) -> bool {
    match claim_warning_slot(
        db_pool,
        chat_id,
        user_id as i64,
        now_unix,
        DEFAULT_WARNING_COOLDOWN_SECONDS,
    ) {
        Ok(true) => true,

        Ok(false) => {
            println!(
                "SECURITY_ACTION chat_id={} user_id={} action=WARNING_SUPPRESSED_COOLDOWN cooldown_seconds={}",
                chat_id,
                user_id,
                DEFAULT_WARNING_COOLDOWN_SECONDS
            );

            false
        }

        Err(err) => {
            eprintln!(
                "SECURITY_DB_ERROR chat_id={} user_id={} operation=CLAIM_WARNING_SLOT fallback=ALLOW_WARNING error={:?}",
                chat_id,
                user_id,
                err
            );

            // Fail-safe for moderation visibility:
            // DB failure must never suppress a legitimate warning.
            true
        }
    }
}

pub async fn security_observer(bot: Bot, msg: Message, db_pool: DbPool) -> ResponseResult<()> {
    let Some(user) = msg.from.as_ref() else {
        return Ok(());
    };

    if user.is_bot {
        return Ok(());
    }

    let Some(text) = msg.text() else {
        return Ok(());
    };

    if is_whitelisted(&bot, &msg, user.id.0).await {
        return Ok(());
    }

    let now = Instant::now();

    let reason = if let Some(reason) = detect_static_reason(text) {
        Some(reason)
    } else if detect_flood(msg.chat.id.0, user.id.0, now) {
        Some(SecurityReason::Flood)
    } else if detect_duplicate(msg.chat.id.0, user.id.0, text, now) {
        Some(SecurityReason::DuplicateMessage)
    } else {
        None
    };

    if let Some(reason) = reason {
        let now_unix = chrono::Utc::now().timestamp();

        let (score, level) = match update_persistent_risk(
            &db_pool,
            msg.chat.id.0,
            user.id.0 as i64,
            &format!("{:?}", reason),
            reason_weight(&reason),
            msg.id.0,
            now_unix,
        ) {
            Ok(score) => {
                println!(
                    "SECURITY_DB chat_id={} user_id={} operation=PERSIST_RISK status=OK risk_score={}",
                    msg.chat.id,
                    user.id.0,
                    score
                );

                (score, risk_level(score))
            }

            Err(err) => {
                eprintln!(
                    "SECURITY_DB_ERROR chat_id={} user_id={} operation=PERSIST_RISK fallback=RAM error={:?}",
                    msg.chat.id,
                    user.id.0,
                    err
                );

                update_risk(msg.chat.id.0, user.id.0, &reason, now)
            }
        };

        println!(
            "SECURITY_EVENT chat_id={} user_id={} reason={:?} weight={} risk_score={} risk_level={:?} message_id={}",
            msg.chat.id,
            user.id.0,
            reason,
            reason_weight(&reason),
            score,
            level,
            msg.id
        );

        match level {
            RiskLevel::Clean | RiskLevel::Low => {}

            RiskLevel::Medium => {
                let warning = format!(
                    "⚠️ Пожалуйста, соблюдайте правила группы. Система обнаружила повторяющуюся или подозрительную активность. Уровень риска: {}.",
                    score
                );

                if should_send_warning(&db_pool, msg.chat.id.0, user.id.0, now_unix) {
                    match bot.send_message(msg.chat.id, warning).await {
                        Ok(_) => {
                            println!(
                                "SECURITY_ACTION chat_id={} user_id={} action=WARNING risk_score={} risk_level={:?}",
                                msg.chat.id,
                                user.id.0,
                                score,
                                level
                            );

                            audit_moderation_action(
                                &db_pool,
                                msg.chat.id.0,
                                user.id.0 as i64,
                                msg.id.0,
                                "WARNING",
                                &reason,
                                score,
                                level,
                                0,
                                0,
                                true,
                                now_unix,
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "SECURITY_ACTION_ERROR chat_id={} user_id={} action=WARNING error={:?}",
                                msg.chat.id, user.id.0, err
                            );

                            audit_moderation_action(
                                &db_pool,
                                msg.chat.id.0,
                                user.id.0 as i64,
                                msg.id.0,
                                "WARNING",
                                &reason,
                                score,
                                level,
                                0,
                                0,
                                false,
                                now_unix,
                            );
                        }
                    }
                }
            }

            RiskLevel::High => {
                match bot.delete_message(msg.chat.id, msg.id).await {
                    Ok(_) => {
                        println!(
                            "SECURITY_ACTION chat_id={} user_id={} action=DELETE_MESSAGE risk_score={} risk_level={:?} message_id={}",
                            msg.chat.id,
                            user.id.0,
                            score,
                            level,
                            msg.id
                        );

                        audit_moderation_action(
                            &db_pool,
                            msg.chat.id.0,
                            user.id.0 as i64,
                            msg.id.0,
                            "DELETE_MESSAGE",
                            &reason,
                            score,
                            level,
                            0,
                            0,
                            true,
                            now_unix,
                        );
                    }
                    Err(err) => {
                        eprintln!(
                            "SECURITY_ACTION_ERROR chat_id={} user_id={} action=DELETE_MESSAGE risk_score={} risk_level={:?} message_id={} error={:?}",
                            msg.chat.id,
                            user.id.0,
                            score,
                            level,
                            msg.id,
                            err
                        );

                        audit_moderation_action(
                            &db_pool,
                            msg.chat.id.0,
                            user.id.0 as i64,
                            msg.id.0,
                            "DELETE_MESSAGE",
                            &reason,
                            score,
                            level,
                            0,
                            0,
                            false,
                            now_unix,
                        );
                    }
                }

                let warning = format!(
                    "⚠️ Сообщение удалено системой защиты группы. Пожалуйста, прекратите спам или подозрительную активность. Уровень риска: {}.",
                    score
                );

                if should_send_warning(&db_pool, msg.chat.id.0, user.id.0, now_unix) {
                    match bot.send_message(msg.chat.id, warning).await {
                        Ok(_) => {
                            println!(
                                "SECURITY_ACTION chat_id={} user_id={} action=WARNING_AFTER_DELETE risk_score={} risk_level={:?}",
                                msg.chat.id,
                                user.id.0,
                                score,
                                level
                            );

                            audit_moderation_action(
                                &db_pool,
                                msg.chat.id.0,
                                user.id.0 as i64,
                                msg.id.0,
                                "WARNING_AFTER_DELETE",
                                &reason,
                                score,
                                level,
                                0,
                                0,
                                true,
                                now_unix,
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "SECURITY_ACTION_ERROR chat_id={} user_id={} action=WARNING_AFTER_DELETE error={:?}",
                                msg.chat.id,
                                user.id.0,
                                err
                            );

                            audit_moderation_action(
                                &db_pool,
                                msg.chat.id.0,
                                user.id.0 as i64,
                                msg.id.0,
                                "WARNING_AFTER_DELETE",
                                &reason,
                                score,
                                level,
                                0,
                                0,
                                false,
                                now_unix,
                            );
                        }
                    }
                }
            }

            RiskLevel::Critical => {
                match bot.delete_message(msg.chat.id, msg.id).await {
                    Ok(_) => {
                        println!(
                            "SECURITY_ACTION chat_id={} user_id={} action=DELETE_MESSAGE risk_score={} risk_level={:?} message_id={}",
                            msg.chat.id,
                            user.id.0,
                            score,
                            level,
                            msg.id
                        );

                        audit_moderation_action(
                            &db_pool,
                            msg.chat.id.0,
                            user.id.0 as i64,
                            msg.id.0,
                            "DELETE_MESSAGE",
                            &reason,
                            score,
                            level,
                            0,
                            0,
                            true,
                            now_unix,
                        );
                    }
                    Err(err) => {
                        eprintln!(
                            "SECURITY_ACTION_ERROR chat_id={} user_id={} action=DELETE_MESSAGE risk_score={} risk_level={:?} message_id={} error={:?}",
                            msg.chat.id,
                            user.id.0,
                            score,
                            level,
                            msg.id,
                            err
                        );

                        audit_moderation_action(
                            &db_pool,
                            msg.chat.id.0,
                            user.id.0 as i64,
                            msg.id.0,
                            "DELETE_MESSAGE",
                            &reason,
                            score,
                            level,
                            0,
                            0,
                            false,
                            now_unix,
                        );
                    }
                }

                let escalation = match claim_critical_escalation(
                    &db_pool,
                    msg.chat.id.0,
                    user.id.0 as i64,
                    now_unix,
                ) {
                    Ok(escalation) => {
                        println!(
                            "SECURITY_DB chat_id={} user_id={} operation=CRITICAL_ESCALATION status=OK critical_occurrence={} mute_seconds={}",
                            msg.chat.id,
                            user.id.0,
                            escalation.occurrence,
                            escalation.mute_seconds
                        );

                        escalation
                    }

                    Err(err) => {
                        eprintln!(
                            "SECURITY_DB_ERROR chat_id={} user_id={} operation=CRITICAL_ESCALATION fallback=MUTE_10_MINUTES error={:?}",
                            msg.chat.id,
                            user.id.0,
                            err
                        );

                        crate::db::security::CriticalEscalation {
                            occurrence: 1,
                            mute_seconds: 10 * 60,
                        }
                    }
                };

                let until = msg.date + chrono::Duration::seconds(escalation.mute_seconds);

                let action = match escalation.mute_seconds {
                    600 => "MUTE_10_MINUTES",
                    3600 => "MUTE_1_HOUR",
                    _ => "MUTE_24_HOURS",
                };

                match bot
                    .restrict_chat_member(msg.chat.id, user.id, ChatPermissions::empty())
                    .until_date(until)
                    .await
                {
                    Ok(_) => {
                        println!(
                            "SECURITY_ACTION chat_id={} user_id={} action={} critical_occurrence={} mute_seconds={} risk_score={} risk_level={:?}",
                            msg.chat.id,
                            user.id.0,
                            action,
                            escalation.occurrence,
                            escalation.mute_seconds,
                            score,
                            level
                        );

                        audit_moderation_action(
                            &db_pool,
                            msg.chat.id.0,
                            user.id.0 as i64,
                            msg.id.0,
                            action,
                            &reason,
                            score,
                            level,
                            escalation.occurrence,
                            escalation.mute_seconds,
                            true,
                            now_unix,
                        );
                    }

                    Err(err) => {
                        eprintln!(
                            "SECURITY_ACTION_ERROR chat_id={} user_id={} action={} critical_occurrence={} mute_seconds={} risk_score={} risk_level={:?} error={:?}",
                            msg.chat.id,
                            user.id.0,
                            action,
                            escalation.occurrence,
                            escalation.mute_seconds,
                            score,
                            level,
                            err
                        );

                        audit_moderation_action(
                            &db_pool,
                            msg.chat.id.0,
                            user.id.0 as i64,
                            msg.id.0,
                            action,
                            &reason,
                            score,
                            level,
                            escalation.occurrence,
                            escalation.mute_seconds,
                            false,
                            now_unix,
                        );
                    }
                }

                let duration_text = match escalation.mute_seconds {
                    600 => "10 минут",
                    3600 => "1 час",
                    _ => "24 часа",
                };

                let warning = format!(
                    "🚨 Система защиты временно ограничила участника на {} из-за повторяющейся подозрительной активности. Уровень риска: {}.",
                    duration_text,
                    score
                );

                if should_send_warning(&db_pool, msg.chat.id.0, user.id.0, now_unix) {
                    match bot.send_message(msg.chat.id, warning).await {
                        Ok(_) => {
                            println!(
                                "SECURITY_ACTION chat_id={} user_id={} action=WARNING_AFTER_MUTE risk_score={} risk_level={:?}",
                                msg.chat.id,
                                user.id.0,
                                score,
                                level
                            );

                            audit_moderation_action(
                                &db_pool,
                                msg.chat.id.0,
                                user.id.0 as i64,
                                msg.id.0,
                                "WARNING_AFTER_MUTE",
                                &reason,
                                score,
                                level,
                                escalation.occurrence,
                                escalation.mute_seconds,
                                true,
                                now_unix,
                            );
                        }
                        Err(err) => {
                            eprintln!(
                                "SECURITY_ACTION_ERROR chat_id={} user_id={} action=WARNING_AFTER_MUTE error={:?}",
                                msg.chat.id,
                                user.id.0,
                                err
                            );

                            audit_moderation_action(
                                &db_pool,
                                msg.chat.id.0,
                                user.id.0 as i64,
                                msg.id.0,
                                "WARNING_AFTER_MUTE",
                                &reason,
                                score,
                                level,
                                escalation.occurrence,
                                escalation.mute_seconds,
                                false,
                                now_unix,
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_weights_are_stable() {
        assert_eq!(reason_weight(&SecurityReason::DuplicateMessage), 1);

        assert_eq!(reason_weight(&SecurityReason::Flood), 2);

        assert_eq!(reason_weight(&SecurityReason::ExcessiveLinks), 3);

        assert_eq!(reason_weight(&SecurityReason::SuspiciousLink), 4);
    }

    #[test]
    fn risk_levels_are_correct() {
        assert_eq!(risk_level(0), RiskLevel::Clean);
        assert_eq!(risk_level(2), RiskLevel::Low);
        assert_eq!(risk_level(5), RiskLevel::Medium);
        assert_eq!(risk_level(9), RiskLevel::High);
        assert_eq!(risk_level(14), RiskLevel::Critical);
    }

    #[test]
    fn counts_links() {
        assert_eq!(count_links("a https://example.com b http://test.com"), 2);
    }

    #[test]
    fn detects_excessive_links() {
        let text = concat!(
            "https://a.example ",
            "https://b.example ",
            "https://c.example ",
            "https://d.example"
        );

        assert_eq!(
            detect_static_reason(text),
            Some(SecurityReason::ExcessiveLinks)
        );
    }

    #[test]
    fn detects_shortener() {
        assert_eq!(
            detect_static_reason("Смотри https://bit.ly/example"),
            Some(SecurityReason::SuspiciousLink)
        );
    }

    #[test]
    fn normal_text_is_clean() {
        assert_eq!(
            detect_static_reason("Добрый день. Ищу электрика в Ницце."),
            None
        );
    }

    #[test]
    fn duplicate_detection_works() {
        let now = Instant::now();

        assert!(!detect_duplicate(-700, 701, "одинаковое сообщение", now));

        assert!(detect_duplicate(
            -700,
            701,
            "одинаковое сообщение",
            now + Duration::from_secs(2)
        ));
    }

    #[test]
    fn flood_detection_works() {
        let chat = -800;
        let user = 801;
        let now = Instant::now();

        for i in 0..(FLOOD_LIMIT - 1) {
            assert!(!detect_flood(
                chat,
                user,
                now + Duration::from_millis(i as u64 * 100)
            ));
        }

        assert!(detect_flood(chat, user, now + Duration::from_secs(1)));
    }

    #[test]
    fn risk_accumulates() {
        let chat = -900;
        let user = 901;
        let now = Instant::now();

        let (score1, level1) = update_risk(chat, user, &SecurityReason::DuplicateMessage, now);

        assert_eq!(score1, 1);
        assert_eq!(level1, RiskLevel::Clean);

        let (score2, level2) = update_risk(
            chat,
            user,
            &SecurityReason::SuspiciousLink,
            now + Duration::from_secs(1),
        );

        assert_eq!(score2, 5);
        assert_eq!(level2, RiskLevel::Medium);
    }
}
