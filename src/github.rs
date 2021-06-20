use serde::Deserialize;
use skim::SkimItem;
use std::borrow::Cow;

#[derive(Debug, Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

impl SkimItem for Release {
    fn text(&self) -> Cow<str> {
        Cow::from(&self.tag_name)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub url: String,
    pub name: String,
}

impl SkimItem for Asset {
    fn text(&self) -> Cow<str> {
        Cow::from(&self.name)
    }
}
