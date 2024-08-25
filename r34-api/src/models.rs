use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Post {
    pub change: usize,
    pub comment_count: usize,
    pub directory: usize,
    pub file_url: String,
    pub has_notes: bool,
    pub hash: String,
    pub height: usize,
    pub id: usize,
    pub image: String,
    pub owner: String,
    pub parent_id: usize,
    pub preview_url: String,
    pub rating: String,
    pub sample: bool,
    pub sample_height: usize,
    pub sample_url: String,
    pub sample_width: usize,
    pub score: usize,
    pub source: String,
    pub status: String,
    pub tags: String,
    pub width: usize,
}
