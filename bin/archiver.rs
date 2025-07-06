#!/usr/bin/env cargo run

// Copyright 2025 Tree xie.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use archiver::Error;
use clap::Parser;
use path_absolutize::*;
use std::path::Path;
use std::{env, str::FromStr};
use substring::Substring;
use tracing::error;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const LS_MODE: &str = "ls";
const UNARCHIVE_MODE: &str = "unarchive";

/// A tool for archive file as tar, but it will compress each file first.
/// Simple way for gz.tar, archiver ~/files ~/files.gz.tar.
/// Simple way for ls, archiver ~/files.gz.tar
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source path to archive
    #[arg(short, long)]
    source: Option<String>,
    /// Archive file
    #[arg(short, long)]
    tar: Option<String>,
    /// Level of compress
    #[arg(short, long, default_value_t = 9)]
    level: i32,
    /// Glob file pattern
    #[arg(short, long, default_value = "/**/*")]
    pattern: String,
    /// Run mode, "archive", "ls", "unarchive"
    #[arg(short, long, default_value = "archive")]
    mode: String,
    /// Unarchive all files to output directory
    #[arg(short, long)]
    output: Option<String>,
    /// Unarchive filter file
    #[arg(short, long)]
    file: Option<String>,
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
    if args.output.is_some() || args.file.is_some() {
        args.mode = UNARCHIVE_MODE.to_string();
    }
    if args.mode != UNARCHIVE_MODE && args.source.clone().unwrap_or_default().is_empty() {
        args.mode = LS_MODE.to_string()
    }
    args
}

#[tokio::main]
async fn run() -> Result<(), Error> {
    let args = parse_args();
    let source = resolve_path(&args.source.unwrap_or_default());
    let target = resolve_path(&args.tar.unwrap_or_default());
    let output = resolve_path(&args.output.unwrap_or_default());

    match args.mode.as_str() {
        LS_MODE => archiver::ls(&target).await,
        UNARCHIVE_MODE => {
            archiver::unarchive(archiver::UnarchiveParams {
                source: target,
                target: output,
                file: args.file.unwrap_or_default(),
            })
            .await
        }
        _ => {
            archiver::archive(archiver::ArchiveParams {
                source,
                target,
                level: args.level,
                pattern: args.pattern,
            })
            .await
        }
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
        error!(message = e.to_string());
    }
}
