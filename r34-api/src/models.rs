use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl Post {
    pub fn get_file_name(&self) -> String {
        let ext = get_file_ext(&self.file_url)
            .or_else(|| get_file_ext(&self.image))
            .unwrap_or("unknown");
        format!("{}_{}x{}.{}", self.id, self.width, self.height, ext)
    }
}

fn get_file_ext(v: &str) -> Option<&str> {
    v.chars()
        .rev()
        .enumerate()
        .find(|(_, c)| *c == '.')
        .map(|(i, _)| &v[v.len() - i..])
        .and_then(|v| if v.is_empty() { None } else { Some(v) })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_file_ext() {
        assert_eq!(get_file_ext("foo.bar"), Some("bar"));
        assert_eq!(get_file_ext("foo.bar.baz"), Some("baz"));
        assert_eq!(get_file_ext("foo.bar."), None);
        assert_eq!(get_file_ext("foobarbaz"), None);
        assert_eq!(get_file_ext(""), None);
    }
}
