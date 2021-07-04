use color_eyre::Result;
use reqwest::{header, Client};
use std::{
    fs::File,
    io::{self, Cursor, Write},
};

pub async fn download(client: &Client, url: &str, display: &str, name: &str) -> Result<()> {
    let display = display.to_owned();
    let res = client
        .get(url)
        .header(header::USER_AGENT, "reqwest 0.11.3")
        .header(header::ACCEPT, "application/octet-stream")
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
            let mut cursor = Cursor::new(res.bytes().await?);
            io::copy(&mut progress.wrap_read(&mut cursor), &mut f).unwrap();
        }
        None => {
            println!("will download: {}", display);
            let content = res.bytes().await?;
            f.write_all(&content)?;
        }
    }
    Ok(())
}
