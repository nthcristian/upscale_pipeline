use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub polling_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoRequest {
    pub model: String,
    pub prompt: String,
    pub duration: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<AspectRatio>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_references: Option<Vec<InputReference>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Reference {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputReference {
    #[serde(rename = "type")]
    pub _type: String,
    pub image_url: Reference,
}

impl InputReference {
    pub fn from_url(url: String) -> Self {
        Self {
            _type: String::from("image_url"),
            image_url: Reference { url },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Resolution {
    #[serde(rename = "1080p")]
    FHD,
    #[serde(rename = "720p")]
    HD,
    #[serde(rename = "480p")]
    SD,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AspectRatio {
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "9:16")]
    Portrait,
    #[serde(rename = "16:9")]
    Landscape,
}
