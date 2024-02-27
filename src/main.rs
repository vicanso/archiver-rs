use clap::Parser;
use path_absolutize::*;
use std::path::Path;
use substring::Substring;

use std::{env, str::FromStr};
use tracing::error;
use tracing::Level;

use tracing_subscriber::FmtSubscriber;

use crate::error::Error;

mod archiver;
mod compression;
mod error;

/// A tool for archive file as tar, but it will compress each file first.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to archive
    #[arg(short, long)]
    source: String,
    /// Archive file save as
    #[arg(short, long)]
    target: String,
    /// Level of compress
    #[arg(short, long, default_value_t = 9)]
    level: i32,
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

#[tokio::main]
async fn run() -> Result<(), Error> {
    let args = Args::parse();
    let source = resolve_path(&args.source);
    let target = resolve_path(&args.target);

    archiver::archive(&source, &target, args.level).await
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
