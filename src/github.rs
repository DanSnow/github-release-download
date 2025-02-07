use nucleo_picker::Render;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Debug, Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Asset {
    pub url: String,
    pub name: String,
}

pub struct GithubRender;

impl Render<Release> for GithubRender {
    type Str<'a> = Cow<'a, str>;

    fn render<'a>(&self, release: &'a Release) -> Self::Str<'a> {
        Cow::from(&release.tag_name)
    }
}

impl Render<Asset> for GithubRender {
    type Str<'a> = Cow<'a, str>;

    fn render<'a>(&self, asset: &'a Asset) -> Self::Str<'a> {
        Cow::from(&asset.name)
    }
}

pub trait Renderable {
    fn get_render() -> impl Render<Self>
    where
        Self: Sized;
}

impl Renderable for Release {
    fn get_render() -> impl Render<Self>
    where
        Self: Sized,
    {
        GithubRender
    }
}
impl Renderable for Asset {
    fn get_render() -> impl Render<Self>
    where
        Self: Sized,
    {
        GithubRender
    }
}
