use std::{fs::File, io, path::Path};

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

pub struct Credentials {
    user_id: String,
    api_key: String,
}

impl Credentials {
    pub fn new(user_id: impl Into<String>, api_key: impl Into<String>) -> Credentials {
        Self {
            user_id: user_id.into(),
            api_key: api_key.into(),
        }
    }
}

impl<U, A> From<(U, A)> for Credentials
where
    U: Into<String>,
    A: Into<String>,
{
    fn from((user_id, api_key): (U, A)) -> Self {
        Self::new(user_id, api_key)
    }
}

pub struct Client {
    root_url: Url,
    credentíals: Credentials,
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new<R: IntoUrl, C: Into<Credentials>>(root_url: R, credentíals: C) -> Result<Self> {
        let root_url = root_url.into_url().map_err(Error::InvalidRootUrl)?;
        let client = reqwest::blocking::Client::default();
        let credentíals = credentíals.into();
        Ok(Self {
            root_url,
            client,
            credentíals,
        })
    }

    pub fn posts<T: AsRef<str>>(
        &self,
        tags: &[T],
        limit: Option<usize>,
        page: Option<usize>,
        last_id: Option<usize>,
    ) -> Result<Vec<Post>> {
        let mut req = self
            .client
            .get(self.root_url.as_ref())
            .query(&DEFAULT_QUERY_PARAMS)
            .query(&[
                ("user_id", &self.credentíals.user_id),
                ("api_key", &self.credentíals.api_key),
            ]);

        let tags_joined: String = tags.iter().map(|v| v.as_ref()).intersperse(" ").collect();
        req = req.query(&[("tags", tags_joined)]);

        if let Some(limit) = limit {
            req = req.query(&[("limit", limit)])
        }

        if let Some(page) = page {
            req = req.query(&[("pid", page)])
        }

        if let Some(last_id) = last_id {
            req = req.query(&[("last_id", last_id)])
        }

        Ok(req.send()?.error_for_status()?.json()?)
    }

    pub fn download_post(
        &self,
        post: &Post,
        out_dir: &Path,
        force_overwrite: bool,
    ) -> Result<bool> {
        let file_name = out_dir.join(post.get_file_name());
        if !force_overwrite && file_name.exists() {
            return Ok(false);
        }

        let mut res = self.client.get(&post.file_url).send()?.error_for_status()?;

        let mut file = File::create(file_name)?;

        io::copy(&mut res, &mut file)?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_from_tuple() {
        let creds = Credentials::from(("user123", "api_key_abc"));
        assert_eq!(creds.user_id, "user123");
        assert_eq!(creds.api_key, "api_key_abc");

        let creds2 = Credentials::from((String::from("user456"), String::from("api_key_def")));
        assert_eq!(creds2.user_id, "user456");
        assert_eq!(creds2.api_key, "api_key_def");
    }
}
