use chrono::{DateTime, Local};
use color_eyre::Result;
use github_update::{download, GithubRelease};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

static APP_NAME: &str = "github-update";
static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::config_dir()
        .map(|path| path.join(APP_NAME))
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(format!(".{}", APP_NAME)))
});

static EXAMPLE_CONFIG: &str = include_str!("config.example.toml");

#[derive(Deserialize, Serialize)]
struct Repo {
    repo: String,
    #[serde(with = "serde_regex")]
    asset: Regex,
    output: Option<String>,
    #[serde(default)]
    pre_release: bool,
    #[serde(default)]
    stable_only: bool,
    last_version: Option<String>,
    last_update: Option<DateTime<Local>>,
}

#[derive(Deserialize, Serialize)]
struct Config {
    output: PathBuf,
    repos: Vec<Repo>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    fs::create_dir_all(&*CONFIG_DIR)?;
    let config_path = CONFIG_DIR.join("config.toml");
    if !config_path.exists() {
        fs::write(&config_path, EXAMPLE_CONFIG)?;
        println!("Please fill up config at {}", config_path.display());
    } else {
        let config = fs::read_to_string(&config_path)?;
        let mut config = toml::from_str::<Config>(&config).expect("Malformed config");
        let output = expand_tilde(&config.output).unwrap();
        env::set_current_dir(&output)?;
        for repo in config.repos.iter_mut() {
            fetch_last_update(repo)?;
        }
        fs::write(&config_path, toml::to_string_pretty(&config).unwrap())?;
    }
    Ok(())
}

fn fetch_last_update(target: &mut Repo) -> Result<()> {
    if let Some(ref time) = target.last_update {
        let interval = Local::now() - *time;
        if interval.num_days() < 30 {
            println!("Skip {}", target.repo);
            return Ok(());
        }
    }
    println!("Checking {}", target.repo);
    let releases = GithubRelease::fetch(&target.repo)?;
    let release = releases.latest(target.pre_release, target.stable_only);
    match release {
        Some(release) => {
            if let Some(ref version) = target.last_version {
                if version == &release.tag_name {
                    println!("Update to date {}", target.repo);
                    return Ok(());
                }
            }
            let asset = release
                .assets
                .iter()
                .find(|asset| target.asset.find(&asset.name).is_some());
            if let Some(asset) = asset {
                let output_name = target
                    .output
                    .as_ref()
                    .map(String::as_str)
                    .unwrap_or(&asset.name);
                download(&asset.url, &asset.name, output_name)?;
                target.last_version = Some(release.tag_name.clone());
                target.last_update = Some(Local::now());
                println!("Download update {} as {}", target.repo, output_name);
            }
        }
        None => {
            println!("Update to date {}", target.repo);
        }
    }
    Ok(())
}

fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();
    if p.starts_with("~") {
        if p == Path::new("~") {
            dirs::home_dir()
        } else {
            dirs::home_dir().map(|mut h| {
                if h == Path::new("/") {
                    // Corner case: `h` root directory;
                    // don't prepend extra `/`, just drop the tilde.
                    p.strip_prefix("~").unwrap().to_path_buf()
                } else {
                    h.push(p.strip_prefix("~/").unwrap());
                    h
                }
            })
        }
    } else {
        Some(p.to_path_buf())
    }
}
