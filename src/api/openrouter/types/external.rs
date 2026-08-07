use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JobRespose {
    pub id: Option<String>,
    pub polling_url: Option<String>,
    pub status: Option<String>,
    pub error: Option<ResponseError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PollingResponse {
    pub id: Option<String>,
    pub polling_url: Option<String>,
    pub status: Option<PollStatus>,
    pub unsigned_urls: Option<Vec<String>>,
    pub usage: Option<UsageInfo>,
    pub error: Option<ResponseError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadResponse {
    pub error: Option<ResponseError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UsageInfo {
    pub cost: f64,
    pub is_byok: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PollStatus {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "expired")]
    Expired,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ResponseError {
    Simple(String),
    Complex { code: i32, message: String },
}
