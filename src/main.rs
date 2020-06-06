use reqwest::{header, Client, StatusCode};
use std::{
    fs::File,
    io::{self, Cursor},
    process,
};
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

fn main() {
    let opt = Opt::from_args();
    let client = Client::new();
    let url = format!("https://api.github.com/repos/{}/releases", opt.repo);
    let mut res = client.get(&url).send().unwrap();
    if res.status() == StatusCode::NOT_FOUND {
        abort!("No such repo {:?}", opt.repo);
    }
    let releases = res.json::<Vec<github::Release>>().unwrap();
    let release = match opt.release.as_str() {
        "latest" => find_latest_release(&releases, &opt),
        tag => find_specific_release(&releases, tag),
    };
    let release = release.or_else(|| {
        select_one(releases.iter().map(|release| &release.tag_name)).map(|index| &releases[index])
    });
    match release {
        Some(release) => match select_one(release.assets.iter().map(|asset| &asset.name)) {
            Some(index) => {
                let asset = &release.assets[index];
                let output_name = opt
                    .output
                    .as_ref()
                    .map(String::as_str)
                    .unwrap_or(&asset.name);
                download(&client, &asset.url, &asset.name, output_name);
            }
            None => (),
        },
        None => {
            abort!("No matched release to download");
        }
    }
}

fn download(client: &Client, url: &str, display: &str, name: &str) {
    let mut res = client
        .get(url)
        .header(header::ACCEPT, "application/octet-stream")
        .send()
        .unwrap();
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
            io::copy(&mut progress.wrap_read(&mut res), &mut f).unwrap();
        }
        None => {
            println!("will download: {}", display);
            res.copy_to(&mut f).unwrap();
        }
    }
}

fn select_one<'a, T: AsRef<str> + 'a, I: IntoIterator<Item = &'a T>>(items: I) -> Option<usize> {
    let input = items
        .into_iter()
        .map(<_>::as_ref)
        .collect::<Vec<&str>>()
        .join("\n");
    let items = skim::Skim::run_with(
        &skim::SkimOptions::default(),
        Some(Box::new(Cursor::new(input))),
    )
    .map(|output| output.selected_items)
    .unwrap_or_else(Vec::new);
    items.first().map(|item| item.get_index())
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
