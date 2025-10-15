use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRepack {
    pub title: String,
    pub genres_tags: Option<String>,
    pub company: Option<String>,
    pub languages: Option<String>,
    pub original_size: Option<String>,
    pub repack_size: Option<String>,
    pub url: String,
    pub date: Option<String>,
    pub image_url: Option<String>,
    pub magnet_links: Vec<MagnetLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagnetLink {
    pub source: String,
    pub magnet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularRepackEntry {
    pub url: String,
    pub title: String,
    pub image_url: Option<String>,
}

#[derive(Default)]
pub(crate) struct GameDetails {
    pub genres_tags: Option<String>,
    pub company: Option<String>,
    pub languages: Option<String>,
    pub original_size: Option<String>,
    pub repack_size: Option<String>,
}

