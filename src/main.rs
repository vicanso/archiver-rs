use clap::Parser;
use path_absolutize::*;
use std::path::Path;
use substring::Substring;

use std::{env, str::FromStr};
use tracing::error;
use tracing::Level;

use tracing_subscriber::FmtSubscriber;

use crate::error::Error;

static LS_MODE: &str = "ls";

mod archiver;
mod compression;
mod error;

/// A tool for archive file as tar, but it will compress each file first.
/// Simple way for gz.tar, archiver ~/files ~/files.gz.tar.
/// Simple way for ls, archiver ~/files.gz.tar
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to archive
    #[arg(short, long)]
    source: Option<String>,
    /// Archive file save as
    #[arg(short, long)]
    target: Option<String>,
    /// Level of compress
    #[arg(short, long, default_value_t = 9)]
    level: i32,
    /// Glob file pattern
    #[arg(short, long, default_value = "/**/*")]
    pattern: String,
    /// Run mode, "archive", "ls"
    #[arg(short, long, default_value = "archive")]
    mode: String,
}

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
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_timer(timer)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn resolve_path(path: &str) -> String {
    if path.is_empty() {
        return "".to_string();
    }
    let mut p = path.to_string();
    if p.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            p = home.to_string_lossy().to_string() + p.substring(1, p.len());
        };
    }
    if let Ok(p) = Path::new(&p).absolutize() {
        p.to_string_lossy().to_string()
    } else {
        p
    }
}

fn parse_args() -> Args {
    let arguments: Vec<String> = env::args().collect();
    let mut args = vec![];
    for (index, item) in arguments.iter().enumerate() {
        if index != 0 && !item.starts_with('-') {
            // 如果上一个参数不是以-开始，而且没有=
            let prev = arguments[index - 1].clone();
            if !prev.starts_with('-') && !prev.contains('=') {
                if item.ends_with(".tar") {
                    args.push("-t");
                } else {
                    args.push("-s");
                }
            }
        }
        args.push(item)
    }
    let mut args = Args::parse_from(args);
    if args.source.clone().unwrap_or_default().is_empty() {
        args.mode = LS_MODE.to_string()
    }
    args
}

#[tokio::main]
async fn run() -> Result<(), Error> {
    let args = parse_args();
    let source = resolve_path(&args.source.unwrap_or_default());
    let target = resolve_path(&args.target.unwrap_or_default());

    if args.mode == LS_MODE {
        archiver::ls(&target).await
    } else {
        archiver::archive(archiver::ArchiveParams {
            source,
            target,
            level: args.level,
            pattern: args.pattern,
        })
        .await
    }
}

fn main() {
    // Because we need to get the local offset before Tokio spawns any threads, our `main`
    // function cannot use `tokio::main`.
    std::panic::set_hook(Box::new(|e| {
        error!(category = "panic", message = e.to_string(),);
        std::process::exit(1);
    }));
    init_logger();
    if let Err(e) = run() {
        error!(message = e.to_string(), "archive fail");
    }
}
