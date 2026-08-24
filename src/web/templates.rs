mod resources;
pub use resources::*;

mod profile_account;
pub use profile_account::*;

mod communication;
pub use communication::*;

mod navigation;
pub use navigation::*;

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

use std::collections::BTreeMap;
