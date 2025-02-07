use crate::github::Release;
use http::{header, StatusCode};
use thiserror::Error;
use ureq::Error as HttpError;

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
    pub fn fetch(repo: &str) -> Result<Self, Error> {
        let url = format!("https://api.github.com/repos/{}/releases", repo);
        let mut res = ureq::get(&url)
            .header(header::USER_AGENT, "reqwest 0.11.3")
            .call()?;
        if res.status() == StatusCode::NOT_FOUND {
            return Err(Error::RepoNotFound(repo.to_owned()));
        }
        let releases = res.body_mut().read_json::<Vec<Release>>()?;
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
