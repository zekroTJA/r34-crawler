use crate::errors::{Error, Result};
use crate::models::Post;
use reqwest::{IntoUrl, Url};
use std::fs::File;
use std::io;
use std::path::Path;

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

impl TryFrom<&str> for Credentials {
    type Error = Error;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        let value = value.trim_start_matches("&");
        let mut user_id = None;
        let mut api_key = None;
        for kv in value.split("&") {
            let (k, v) = kv.split_once("=").ok_or(Error::MalformedCredentials)?;
            match k {
                "user_id" => user_id = Some(v),
                "api_key" => api_key = Some(v),
                _ => (),
            }
        }
        Ok(Credentials::new(
            user_id.ok_or(Error::MissingCredentials("user_id"))?,
            api_key.ok_or(Error::MissingCredentials("api_key"))?,
        ))
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
    credentials: Credentials,
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new<R: IntoUrl, C: Into<Credentials>>(root_url: R, credentials: C) -> Result<Self> {
        let root_url = root_url.into_url().map_err(Error::InvalidRootUrl)?;
        let client = reqwest::blocking::Client::default();
        let credentials = credentials.into();
        Ok(Self {
            root_url,
            client,
            credentials,
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
                ("user_id", &self.credentials.user_id),
                ("api_key", &self.credentials.api_key),
            ]);

        let mut tags_joined = String::new();
        for (i, tag) in tags.iter().enumerate() {
            if i > 0 {
                tags_joined.push(' ');
            }
            tags_joined.push_str(tag.as_ref());
        }
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

        let creds = Credentials::from((String::from("user456"), String::from("api_key_def")));
        assert_eq!(creds.user_id, "user456");
        assert_eq!(creds.api_key, "api_key_def");
    }

    #[test]
    fn test_credentials_from_string() {
        // OK - no trailing &
        let creds = Credentials::try_from("api_key=foobar&user_id=baz").unwrap();
        assert_eq!(creds.user_id, "baz");
        assert_eq!(creds.api_key, "foobar");

        // OK - trailing &
        let creds = Credentials::try_from("&api_key=foobar&user_id=baz").unwrap();
        assert_eq!(creds.user_id, "baz");
        assert_eq!(creds.api_key, "foobar");

        // OK - extra query args
        let creds = Credentials::try_from("api_key=foobar&extra=asd&user_id=baz").unwrap();
        assert_eq!(creds.user_id, "baz");
        assert_eq!(creds.api_key, "foobar");

        // ERROR - malformed
        let err = Credentials::try_from("api_key&extra=asd&user_id=baz");
        assert!(matches!(err, Err(Error::MalformedCredentials)));

        // ERROR - empty
        let err = Credentials::try_from("");
        assert!(matches!(err, Err(Error::MalformedCredentials)));

        // ERROR - empty with leading &
        let err = Credentials::try_from("&");
        assert!(matches!(err, Err(Error::MalformedCredentials)));

        // ERROR - trailing &
        let err = Credentials::try_from("api_key=foo&user_id=asd&");
        assert!(matches!(err, Err(Error::MalformedCredentials)));

        // ERROR - missing user_id
        let err = Credentials::try_from("api_key=foo");
        assert!(matches!(err, Err(Error::MissingCredentials("user_id"))));
        let err = Credentials::try_from("extra=asd&api_key=foo");
        assert!(matches!(err, Err(Error::MissingCredentials("user_id"))));

        // ERROR - missing api_key
        let err = Credentials::try_from("user_id=foo");
        assert!(matches!(err, Err(Error::MissingCredentials("api_key"))));
        let err = Credentials::try_from("extra=asd&user_id=foo");
        assert!(matches!(err, Err(Error::MissingCredentials("api_key"))));
    }
}
