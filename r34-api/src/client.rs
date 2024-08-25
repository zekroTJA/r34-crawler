use crate::{
    errors::{Error, Result},
    models::Post,
};
use reqwest::{IntoUrl, Url};

pub const API_ROOT_URL: &str = "https://api.rule34.xxx/index.php";

static DEFAULT_QUERY_PARAMS: [(&str, &str); 4] = [
    ("page", "dapi"),
    ("s", "post"),
    ("q", "index"),
    ("json", "1"),
];

pub struct Client {
    root_url: Url,
    client: reqwest::blocking::Client,
}

impl Default for Client {
    fn default() -> Self {
        Self::new(API_ROOT_URL).expect("default client construction")
    }
}

impl Client {
    pub fn new<R: IntoUrl>(root_url: R) -> Result<Self> {
        let root_url = root_url.into_url().map_err(Error::InvalidRootUrl)?;
        let client = reqwest::blocking::Client::default();
        Ok(Self { root_url, client })
    }

    pub fn posts<T: AsRef<str>>(
        &self,
        tags: &[T],
        limit: Option<usize>,
        page: Option<usize>,
    ) -> Result<Vec<Post>> {
        let mut req = self
            .client
            .get(self.root_url.as_ref())
            .query(&DEFAULT_QUERY_PARAMS);

        let tags_joined: String = tags.iter().map(|v| v.as_ref()).intersperse(" ").collect();
        req = req.query(&[("tags", tags_joined)]);

        if let Some(limit) = limit {
            req = req.query(&[("limit", limit)])
        }

        if let Some(page) = page {
            req = req.query(&[("pid", page)])
        }

        Ok(req.send()?.error_for_status()?.json()?)
    }
}
