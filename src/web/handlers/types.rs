use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddResourceForm {
    pub title: String,
    pub description: String,
    pub contact: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessageForm {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct EditResourceForm {
    pub title: String,
    pub description: String,
    pub contact: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct ReportResourcePayload {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ContactRequestPayload {
    pub public_id: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct RejectResourceForm {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct PromotionRequestForm {
    pub target_id: i64,
}
