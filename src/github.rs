use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    pub url: String,
    pub name: String,
}
