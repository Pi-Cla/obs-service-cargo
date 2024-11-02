#![allow(clippy::unwrap_used)]

use libroast::common::Compression;
use obs_service_cargo::cli;
use std::{io, path::PathBuf};
use tokio::fs;
use tokio_test::task::spawn;
use tracing::{error, info};
use tracing_test::traced_test;
use rand::prelude::*;

async fn vendor_source(source: &str, filter: bool) -> io::Result<PathBuf> {
    let mut rng = rand::thread_rng();
    let random_tag: u8 = rng.gen();
    let random_tag = random_tag.to_string();
    let response = reqwest::get(source).await.unwrap();
    let fname = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("balls");
    info!("Source file: {}", &fname);
    let outfile = format!("/{}/{}", "tmp", &fname);
    info!("Downloaded to: '{:?}'", &outfile);
    fs::File::create(&outfile).await.unwrap();
    let outfile = PathBuf::from(&outfile);
    let data = response.bytes().await.unwrap();
    let data = data.to_vec();
    fs::write(&outfile, data).await.unwrap();
    let outdir = PathBuf::from("/tmp");
    let opt = cli::Opts {
        src: cli::Src {
            src: outfile.to_path_buf(),
        },
        compression: Compression::default(),
        tag: Some(random_tag),
        cargotoml: [].to_vec(),
        update: true,
        filter,
        outdir,
        color: clap::ColorChoice::Auto,
        i_accept_the_risk: [].to_vec(),
        respect_lockfile: false,
        versioned_dirs: true,
    };

    let res = opt.src.run_vendor(&opt).map_err(|err| {
        error!(?err);
        io::Error::new(io::ErrorKind::Interrupted, err.to_string())
    });
    assert!(res.is_ok());
    Ok(outfile)
}

#[traced_test]
#[tokio::test]
async fn no_filter_vendor_sources() -> io::Result<()> {
    let sources = [
        "https://github.com/elliot40404/bonk/archive/refs/tags/v0.3.2.tar.gz",
        "https://github.com/openSUSE-Rust/roast/archive/refs/tags/v5.1.2.tar.gz",
    ];
    for src in sources {
        let _ = spawn(async move {
            vendor_source(src, false).await.unwrap();
            src
        })
        .await;
    }
    Ok(())
}

#[tokio::test]
async fn filter_vendor_sources() -> io::Result<()> {
    let sources: &[&str] = &[
        "https://github.com/wez/wezterm/archive/refs/tags/20240203-110809-5046fc22.tar.gz",
        "https://github.com/alacritty/alacritty/archive/refs/tags/v0.14.0.tar.gz"
    ];
    for src in sources {
        let _ = spawn(async move {
            filter_vendor_source(src, true).await.unwrap();
            src
        })
        .await;
    }
    Ok(())
}

