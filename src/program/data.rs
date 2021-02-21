use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ScriptRequest {
    pub vod: String,
    pub ima: String,
    pub product: i64,
    pub channel: String,
    #[serde(rename = "vodThumb")]
    pub vod_thumb: String,
}
