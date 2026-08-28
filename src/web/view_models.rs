// Shared row shapes used between SQLite handlers and HTML renderers.

pub type CategoryResourceRow = (i64, String, String, String, String, f64, i64, i64, i64);
pub type FavoriteResourceRow = (i64, String, String, String, String, f64, i64, i64, i64);
pub type NotificationRow = (i64, Option<i64>, String, String, String, i64, i64);
pub type ContactRequestRow = (i64, i64, String, String, String, String, String, i64, i64);
pub type ConversationRow = (i64, i64, String, String, String, String, i64, i64);
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
