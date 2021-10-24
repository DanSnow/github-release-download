use clap::Parser;
use color_eyre::Result;
use github_update::{download, github, GithubRelease};
use reqwest::Client;
use skim::prelude::{bounded, Skim, SkimItem};
use std::{process, sync::Arc, thread};

macro_rules! abort {
    ($($args:tt)*) => {
        eprintln!($($args)*);
        process::exit(1);
    }
}

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, about = "Allow pre-release version")]
    pre_release: bool,
    #[clap(short, long, about = "Only select stable version")]
    stable_only: bool,
    #[clap(
        short,
        long,
        about = "Set the download file name, or it will be same as asset name"
    )]
    output: Option<String>,
    #[clap(about = r#"Repoistory name, in "<user>/<repo>" form"#)]
    repo: String,
    #[clap(
        about = r#"Tag name you want to download, or use "latest" to download lastest release"#,
        default_value = "latest"
    )]
    release: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();
    let client = Client::new();
    let releases = GithubRelease::fetch(&client, &opt.repo).await?;
    let release = match opt.release.as_str() {
        "latest" => releases.latest(opt.pre_release, opt.stable_only),
        tag => releases.find_tag(tag),
    };
    match release {
        Some(release) => {
            let asset = select_one(release.assets.clone()).map(|item| {
                (*item)
                    .as_any()
                    .downcast_ref::<github::Asset>()
                    .expect("Fail to downcast asset")
                    .clone()
            });
            match asset {
                Some(asset) => {
                    let output_name = opt
                        .output
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or(&asset.name);
                    download(&client, &asset.url, &asset.name, output_name).await?;
                }
                None => (),
            }
        }
        None => {
            abort!("No matched release to download");
        }
    }
    Ok(())
}

fn select_one<T: SkimItem, I: IntoIterator<Item = T>>(items: I) -> Option<Arc<dyn SkimItem>> {
    let items = items
        .into_iter()
        .map(|x| Arc::new(x) as Arc<dyn SkimItem>)
        .collect::<Vec<Arc<dyn SkimItem>>>();
    let (tx, rx) = bounded(10240);
    thread::spawn(move || {
        for item in items {
            if tx.send(item).is_err() {
                break;
            }
        }
    });
    let items = Skim::run_with(&skim::SkimOptions::default(), Some(rx))
        .map(|output| output.selected_items)
        .unwrap_or_else(Vec::new);
    items.into_iter().next()
}
