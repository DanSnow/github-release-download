use crate::github::Release;
use reqwest::{header, Client, Error as HttpError, StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("repo `{0}` not found")]
    RepoNotFound(String),
    #[error("http error")]
    HttpError(#[from] HttpError),
}

pub struct GithubRelease {
    releases: Vec<Release>,
}

impl GithubRelease {
    pub async fn fetch(client: &Client, repo: &str) -> Result<Self, Error> {
        let url = format!("https://api.github.com/repos/{}/releases", repo);
        let res = client
            .get(&url)
            .header(header::USER_AGENT, "reqwest 0.11.3")
            .send()
            .await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Err(Error::RepoNotFound(repo.to_owned()));
        }
        let releases = res.json::<Vec<Release>>().await?;
        Ok(Self { releases })
    }

    pub fn latest(&self, pre_release: bool, stable_only: bool) -> Option<&Release> {
        if pre_release {
            return self.releases.first();
        }
        let first_stable = self.releases.iter().find(|release| !release.prerelease);
        if stable_only {
            return first_stable;
        }
        first_stable.or_else(|| self.releases.first())
    }

    pub fn find_tag(&self, tag: &str) -> Option<&Release> {
        self.releases.iter().find(|release| release.tag_name == tag)
    }
}
