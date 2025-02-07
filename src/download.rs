use color_eyre::Result;
use http::header;
use std::{
    fs::File,
    io::{self, Read, Write},
};

pub fn download(url: &str, display: &str, name: &str) -> Result<()> {
    let display = display.to_owned();
    let res = ureq::get(url)
        .header(header::USER_AGENT, "ureq 3.0.4")
        .header(header::ACCEPT, "application/octet-stream")
        .call()?;
    let len = res.headers().get(header::CONTENT_LENGTH).and_then(|value| {
        value
            .to_str()
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
    });
    let mut f = File::create(name).unwrap();
    let mut body = res.into_body().into_reader();
    match len {
        Some(len) => {
            let progress = indicatif::ProgressBar::new(len);
            progress.set_prefix(display);
            let mut buf = vec![0; 4096];
            loop {
                let size = body.read(&mut buf)?;
                if size == 0 {
                    break;
                }
                f.write_all(&buf[..size])?;
                progress.inc(size as u64);
            }
            progress.finish();
        }
        None => {
            println!("will download: {}", display);
            io::copy(&mut body, &mut f)?;
        }
    }
    Ok(())
}
