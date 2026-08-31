// Shared row shapes used between SQLite handlers and HTML renderers.

pub type CategoryResourceRow = (i64, String, String, String, String, f64, i64, i64, i64);
pub type FavoriteResourceRow = (i64, String, String, String, String, f64, i64, i64, i64);
pub type NotificationRow = (i64, Option<i64>, String, String, String, i64, i64);
pub type ContactRequestRow = (i64, i64, String, String, String, String, String, i64, i64);

pub struct ConversationRow {
    #[allow(dead_code)]
    pub id: i64,
    pub other_user_id: i64,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub last_message: String,
    pub unread_count: i64,
    pub updated_at: i64,
}

pub struct UserSessionRow {
    pub session_public_id: String,
    pub ip_address: String,
    pub user_agent: String,
    #[allow(dead_code)]
    pub created_at: i64,
    #[allow(dead_code)]
    pub last_seen_at: i64,
    pub is_current: bool,
}

pub struct ChatMessageRow {
    pub id: i64,
    pub sender_user_id: i64,
    pub message: String,
    pub is_read: i64,
    pub created_at: i64,
    pub delivered_at: i64,
    pub read_at: i64,
    pub reply_to_message_id: i64,
    pub reply_sender_user_id: i64,
    pub reply_message: String,
    pub edited_at: i64,
    pub deleted_at: i64,
    pub attachment_kind: String,
    pub attachment_url: String,
}

pub type PublicProfileResourceRow = (i64, String, String, String, f64, i64, i64, i64);
pub type MyResourceRow = (
    i64,
    String,
    String,
    String,
    f64,
    i64,
    i64,
    i64,
    String,
    String,
    i64,
);
pub type AdminReportRow = (
    i64,
    i64,
    i64,
    String,
    String,
    i64,
    String,
    String,
    String,
    i64,
);
pub type AdminResourceRow = (
    i64,
    String,
    String,
    String,
    f64,
    i64,
    i64,
    i64,
    i64,
    String,
    String,
);
pub type SearchResourceRow = (
    i64,
    String,
    String,
    String,
    String,
    f64,
    i64,
    i64,
    i64,
    usize,
    usize,
    usize,
);
pub type SearchPersonRow = (String, String, String, String, i64, String, i64, i64);
