use std::{env, str::FromStr};
use tracing::Level;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

use crate::error::Error;

mod archiver;
mod compression;
mod error;

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
    archiver::archive(&original, &target, 9).await
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
