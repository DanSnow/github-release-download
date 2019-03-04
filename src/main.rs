use reqwest::{header, Client};
use std::{
    fs::File,
    io::{self, Cursor},
};
use structopt::StructOpt;

mod github;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "p", long = "pre-release")]
    pre_release: bool,
    #[structopt(short = "s", long = "stable-only")]
    stable_only: bool,
    repo: String,
    #[structopt(default_value = "latest")]
    release: String,
}

fn main() {
    let opt = Opt::from_args();
    let client = Client::new();
    let url = format!("https://api.github.com/repos/{}/releases", opt.repo);
    let mut res = client.get(&url).send().unwrap();
    let releases = res.json::<Vec<github::Release>>().unwrap();
    let release = find_latest_release(&releases, &opt);
    match release {
        Some(release) => {
            let names = release
                .assets
                .iter()
                .map(|asset| asset.name.as_str())
                .collect::<Vec<&str>>()
                .join("\n");
            let items = skim::Skim::run_with(
                &skim::SkimOptions::default(),
                Some(Box::new(Cursor::new(names))),
            )
            .map(|output| output.selected_items)
            .unwrap_or_else(Vec::new);
            let item = items.first().unwrap();
            let asset = &release.assets[item.get_index()];
            let mut res = client
                .get(&asset.url)
                .header(header::ACCEPT, "application/octet-stream")
                .send()
                .unwrap();
            let len = res.headers().get(header::CONTENT_LENGTH).and_then(|value| {
                value
                    .to_str()
                    .ok()
                    .and_then(|value| value.parse::<u64>().ok())
            });
            let mut f = File::create(&asset.name).unwrap();
            match len {
                Some(len) => {
                    let progress = indicatif::ProgressBar::new(len);
                    progress.set_prefix(&asset.name);
                    io::copy(&mut progress.wrap_read(&mut res), &mut f).unwrap();
                }
                None => {
                    println!("will download: {}", asset.name);
                    res.copy_to(&mut f).unwrap();
                }
            }
        }
        None => {
            eprintln!("No matched release to download");
        },
    }
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
