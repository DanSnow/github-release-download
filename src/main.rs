use color_eyre::Result;
use reqwest::{header, Client, StatusCode};
use skim::prelude::{bounded, Skim, SkimItem};
use std::{fs::File, io, process, sync::Arc, thread};
use structopt::StructOpt;

mod github;

macro_rules! abort {
    ($($args:tt)*) => {
        eprintln!($($args)*);
        process::exit(1);
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, help = "Allow pre-release version")]
    pre_release: bool,
    #[structopt(short, long, help = "Only select stable version")]
    stable_only: bool,
    #[structopt(
        short,
        long,
        help = "Set the download file name, or it will be same as asset name"
    )]
    output: Option<String>,
    #[structopt(help = r#"Repoistory name, in "<user>/<repo>" form"#)]
    repo: String,
    #[structopt(
        help = r#"Tag name you want to download, or use "latest" to download lastest release"#,
        default_value = "latest"
    )]
    release: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    let client = Client::new();
    let url = format!("https://api.github.com/repos/{}/releases", opt.repo);
    let res = client
        .get(&url)
        .header(header::USER_AGENT, "reqwest 0.11.3")
        .send()
        .await?;
    if res.status() == StatusCode::NOT_FOUND {
        abort!("No such repo {:?}", opt.repo);
    }
    let releases = res.json::<Vec<github::Release>>().await?;
    let release = match opt.release.as_str() {
        "latest" => find_latest_release(&releases, &opt),
        tag => find_specific_release(&releases, tag),
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

async fn download(client: &Client, url: &str, display: &str, name: &str) -> Result<()> {
    let display = display.to_owned();
    let res = client
        .get(url)
        .header(header::ACCEPT, "application/octet-stream")
        .header(header::USER_AGENT, "reqwest 0.11.3")
        .send()
        .await?;
    let len = res.headers().get(header::CONTENT_LENGTH).and_then(|value| {
        value
            .to_str()
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
    });
    let mut f = File::create(name).unwrap();
    match len {
        Some(len) => {
            let progress = indicatif::ProgressBar::new(len);
            progress.set_prefix(display);
            let mut body = io::Cursor::new(res.bytes().await?);
            io::copy(&mut progress.wrap_read(&mut body), &mut f)?;
        }
        None => {
            println!("will download: {}", display);
            let mut body = io::Cursor::new(res.bytes().await?);
            io::copy(&mut body, &mut f)?;
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

fn find_latest_release<'a>(
    releases: &'a [github::Release],
    opt: &Opt,
) -> Option<&'a github::Release> {
    if opt.pre_release {
        return releases.first();
    }
    let first_stable = releases.iter().find(|release| !release.prerelease);
    if opt.stable_only {
        return first_stable;
    }
    first_stable.or_else(|| releases.first())
}

fn find_specific_release<'a>(
    releases: &'a [github::Release],
    tag: &str,
) -> Option<&'a github::Release> {
    releases.iter().find(|release| release.tag_name == tag)
}
