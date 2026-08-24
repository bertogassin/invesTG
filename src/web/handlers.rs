mod favorites;
pub use favorites::*;

mod notifications;
pub use notifications::*;

mod contacts;
pub use contacts::*;

mod chat;
pub use chat::*;

mod profiles;
pub use profiles::*;

mod resources;
pub use resources::*;

mod admin;
pub use admin::*;

mod navigation;
pub use navigation::*;

mod types;
pub use types::*;

mod health;
pub use health::*;

mod common;
use common::{
    csrf_rejected_response, input_text_is_valid, rate_limit_retry_after, request_is_cross_site,
    telegram_owner_user_id, unix_now,
};

mod auth;
pub use auth::{app_auth, app_auth_page};
use auth::{
    create_admin_session, is_admin_session, verify_telegram_init_data, verify_user_session,
};

use super::templates;
use crate::state::app_state::AppState;
