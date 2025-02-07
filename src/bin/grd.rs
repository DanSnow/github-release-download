use clap::Parser;
use color_eyre::Result;
use github_update::{download, github::Renderable, GithubRelease};
use nucleo_picker::PickerOptions;
use std::process;

macro_rules! abort {
    ($($args:tt)*) => {
        eprintln!($($args)*);
        process::exit(1);
    }
}

#[derive(Debug, Parser)]
struct Opt {
    #[arg(short, long, help = "Allow pre-release version")]
    pre_release: bool,
    #[arg(short, long, help = "Only select stable version")]
    stable_only: bool,
    #[arg(
        short,
        long,
        help = "Set the download file name, or it will be same as asset name"
    )]
    output: Option<String>,
    #[arg(help = r#"Repository name, in "<user>/<repo>" form"#)]
    repo: String,
    #[arg(
        help = r#"Tag name you want to download, or use "latest" to download latest release"#,
        default_value = "latest"
    )]
    release: String,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let releases = GithubRelease::fetch(&opt.repo)?;
    let release = match opt.release.as_str() {
        "latest" => releases.latest(opt.pre_release, opt.stable_only),
        tag => releases.find_tag(tag),
    };
    match release {
        Some(release) => {
            let asset = select_one(release.assets.clone());
            match asset {
                Some(asset) => {
                    let output_name = opt
                        .output
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or(&asset.name);
                    download(&asset.url, &asset.name, output_name)?;
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

fn select_one<T: Renderable + Clone + Send + Sync + 'static, I: IntoIterator<Item = T>>(
    items: I,
) -> Option<T> {
    let render = T::get_render();
    let mut picker = PickerOptions::default().picker(render);
    let injector = picker.injector();

    for item in items {
        injector.push(item)
    }
    picker.pick().expect("Fail to pick").cloned()
}
