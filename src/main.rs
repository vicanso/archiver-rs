use async_compression::tokio::bufread::GzipEncoder;
use glob::glob;
use snafu::prelude::*;
use std::io::Cursor;
// use std::io::Write;
use std::path::Path;
use std::{env, str::FromStr};
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt as _;
use tokio_tar::Builder;
use tracing::Level;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Arg is invalid {path}"))]
    InvalidArg { path: String },
    #[snafu(display("Io error {source}"))]
    Io { source: std::io::Error },
    #[snafu(display("Strp prefix {source}"))]
    StripPrefix { source: std::path::StripPrefixError },
    #[snafu(display("Pattern {source}"))]
    Pattern { source: glob::PatternError },
    #[snafu(display("Glob {source}"))]
    Glob { source: glob::GlobError },
    #[snafu(display("Compression is invalid {compression}"))]
    InvalidCompression { compression: String },
}

const GZIP: &str = "gz";

fn init_logger() {
    let mut level = Level::INFO;
    if let Ok(log_level) = env::var("LOG_LEVEL") {
        if let Ok(value) = Level::from_str(log_level.as_str()) {
            level = value;
        }
    }
    let timer = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339().unwrap_or_else(|_| {
        tracing_subscriber::fmt::time::OffsetTime::new(
            time::UtcOffset::from_hms(0, 0, 0).unwrap(),
            time::format_description::well_known::Rfc3339,
        )
    });
    let env = std::env::var("RUST_ENV").unwrap_or_default();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_timer(timer)
        .with_ansi(env != "production")
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[tokio::main]
async fn run() -> Result<(), Error> {
    let mut original = "".to_string();
    let mut target = "".to_string();
    for (index, argument) in env::args().enumerate() {
        if index == 1 {
            original = argument;
        } else if index == 2 {
            target = argument;
        }
    }
    if original.is_empty() {
        return Err(Error::InvalidArg { path: original });
    }
    if target.is_empty() {
        return Err(Error::InvalidArg {
            path: target.clone(),
        });
    }
    let filename = Path::new(&target)
        .file_name()
        .ok_or(Error::InvalidArg {
            path: target.clone(),
        })?
        .to_string_lossy();
    let arr: Vec<&str> = filename.split('.').collect();
    if arr.len() < 3 || !arr.contains(&"tar") {
        return Err(Error::InvalidArg {
            path: target.clone(),
        });
    }
    // compress each file first
    let mut compress_first = false;
    let mut compress_type = arr[arr.len() - 2];
    if [GZIP].contains(&compress_type) {
        compress_first = true;
    }
    if !compress_first {
        compress_type = arr[arr.len() - 1];
    }

    println!("{:?}", arr.clone());
    println!("{compress_type}");
    println!("{compress_first}");

    let file = File::create(target.clone()).await.context(IoSnafu {})?;
    let mut a = Builder::new(file);

    // let mut reader = read_dir(original.clone()).await.context(IoSnafu {})?;
    // println!("{original:?}");
    for suffix in ["/*", "/*/*"] {
        for entry in glob(&(original.clone() + suffix)).context(PatternSnafu {})? {
            let file_path = entry.context(GlobSnafu {})?;
            if file_path.is_dir() {
                continue;
            }
            let filename = file_path
                .strip_prefix(&original)
                .context(StripPrefixSnafu {})?;
            // compress file first
            if compress_first {
                match compress_type {
                    GZIP => {
                        let buf = fs::read(&file_path).await.context(IoSnafu {})?;
                        let mut w = GzipEncoder::new(Cursor::new(Vec::new()));
                        w.get_mut().write_all(&buf).await.context(IoSnafu {})?;
                        w.get_mut().shutdown().await.context(IoSnafu {})?;
                        let mut tmp = tempfile::tempfile().context(IoSnafu {})?;
                        // tmp.write_all(&w.into_inner().into_inner())
                        // .context(IoSnafu {})?;
                        // a.append_file(filename, &mut tmp)
                        //     .await
                        //     .context(IoSnafu {})?;

                        // fs::write(path, contents)
                        // tmp.write_all(w.into_inner().bytes());
                        // tmp.write_all(&w.into_inner().into()));
                        // println!("{tmp:?}", ;

                        // tmp.write_all(w.into_inner().into()).context(IoSnafu {})?;
                        // std::fs::write(tmp, w.into_inner().into()).context(IoSnafu {})?;
                        // fs::write(tmp, w.into_inner().into())
                        // .await
                        // .context(IoSnafu {})?;
                        // write!(tmp, w.into_inner()).context(IoSnafu {})?;
                        // let buf = w.into_inner().bytes();
                        // println!("{buf:?}");
                        // w.write_all(buf);
                    }
                    _ => {
                        return Err(Error::InvalidCompression {
                            compression: compress_type.to_string(),
                        })
                    }
                }
                continue;
            }
            a.append_file(
                filename,
                &mut File::open(&file_path).await.context(IoSnafu {})?,
            )
            .await
            .context(IoSnafu {})?;
        }
    }

    Ok(())
}

fn main() {
    // Because we need to get the local offset before Tokio spawns any threads, our `main`
    // function cannot use `tokio::main`.
    std::panic::set_hook(Box::new(|e| {
        error!(category = "panic", message = e.to_string(),);
        std::process::exit(1);
    }));
    init_logger();
    if let Err(err) = run() {
        println!("{err:?}");
    }
}
