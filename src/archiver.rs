use chrono::{DateTime, Local};
use glob::glob;
use pad::{Alignment, PadStr};
use std::path::Path;
use std::time::SystemTime;
use tokio::fs::File;
use tokio_stream::StreamExt;
use tokio_tar::{Archive, Builder};
use tracing::{debug, info};
use uuid::{NoContext, Timestamp, Uuid};

use super::compression;
use super::error::Error;

const GZIP: &str = "gz";
const ZSTD: &str = "zst";
const BROTLI: &str = "br";
const LZ4: &str = "lz4";
const SNAPPY: &str = "sz";
const DEFLATE: &str = "zip";
const XZ: &str = "xz";

fn uuid() -> String {
    let ts = Timestamp::now(NoContext);
    Uuid::new_v7(ts).to_string()
}

pub struct ArchiveParams {
    pub source: String,
    pub target: String,
    pub level: i32,
    pub pattern: String,
}
pub struct UnarchiveParams {
    pub source: String,
    pub target: String,
    pub file: String,
}

pub async fn ls(target: &str) -> Result<(), Error> {
    if target.is_empty() {
        return Err(Error::InvalidArg {
            path: target.to_string(),
        });
    }
    let file = File::open(target).await?;
    let mut r = Archive::new(file);
    let mut entries = r.entries()?;
    let mut lines = vec![];
    while let Some(file) = entries.next().await {
        let f = file?;
        let size = if let Ok(size) = f.header().size() {
            bytesize::ByteSize(size).to_string()
        } else {
            "--".to_string()
        }
        .pad_to_width_with_alignment(8, Alignment::Right);
        let mode = if let Ok(mode) = f.header().mode() {
            unix_mode::to_string(mode)
        } else {
            "--".to_string()
        };
        let mtime = if let Ok(mtime) = f.header().mtime() {
            let mtime: DateTime<Local> = DateTime::from_timestamp(mtime as i64, 0)
                .unwrap_or_default()
                .into();
            mtime.naive_local().to_string()
        } else {
            "--".to_string()
        }
        .pad_to_width_with_alignment(19, Alignment::Right);

        lines.push(format!("{mode}  {size}  {mtime}  {}", f.path()?.display(),));
    }

    println!("total {}", lines.len());

    for line in lines {
        println!("{line}");
    }

    Ok(())
}

pub async fn unarchive(params: UnarchiveParams) -> Result<(), Error> {
    if params.source.is_empty() {
        return Err(Error::InvalidArg {
            path: params.source,
        });
    }
    let arr: Vec<&str> = params.source.split('.').collect();
    if arr.len() < 3 {
        return Err(Error::InvalidArg {
            path: params.source,
        });
    }
    let compress_type = arr[arr.len() - 2];

    let file = File::open(&params.source).await?;
    let mut r = Archive::new(file);
    let mut entries = r.entries()?;
    let output = if params.target.is_empty() {
        Path::new(&params.source)
            .parent()
            .ok_or(Error::InvalidArg {
                path: params.source.clone(),
            })?
    } else {
        Path::new(&params.target)
    };
    let mut file_count = 0;
    let start = SystemTime::now();

    while let Some(file) = entries.next().await {
        let mut f = file?;
        let path = f.path()?;
        if !params.file.is_empty() && params.file != path.to_string_lossy() {
            continue;
        }
        file_count += 1;

        let file_path = output.join(path);
        debug!(
            file = file_path.to_string_lossy().to_string(),
            "start to decode"
        );
        let filename = &Some(file_path);
        let buf = match compress_type {
            GZIP => compression::gzip_decode(&mut f, filename).await,
            ZSTD => compression::zstd_decode(&mut f, filename).await,
            BROTLI => compression::brotli_decode(&mut f, filename).await,
            SNAPPY => compression::snappy_decode(&mut f, filename).await,
            LZ4 => compression::lz4_decode(&mut f, filename).await,
            DEFLATE => compression::deflate_decode(&mut f, filename).await,
            XZ => compression::xz_decode(&mut f, filename).await,
            _ => Err(Error::InvalidCompression {
                compression: compress_type.to_string(),
            }),
        }?;
        if !params.file.is_empty() {
            println!("{}", std::string::String::from_utf8_lossy(&buf));
        }
    }
    let mut duration = None;
    if let Ok(d) = SystemTime::now().duration_since(start) {
        duration = Some(humantime::format_duration(d).to_string());
    };
    if params.file.is_empty() {
        info!(
            path = output.to_string_lossy().to_string(),
            file_count, duration
        );
    }

    Ok(())
}

pub async fn archive(params: ArchiveParams) -> Result<(), Error> {
    if params.target.is_empty() {
        return Err(Error::InvalidArg {
            path: params.target,
        });
    }
    if params.source.is_empty() {
        return Err(Error::InvalidArg {
            path: params.source,
        });
    }
    let dir = tempfile::tempdir()?;
    let source = params.source;
    let target = params.target;
    let level = params.level;

    let filename = Path::new(&target)
        .file_name()
        .ok_or(Error::InvalidArg {
            path: target.clone(),
        })?
        .to_string_lossy();
    let arr: Vec<&str> = filename.split('.').collect();
    if arr.len() < 3 || arr[2] != "tar" {
        return Err(Error::InvalidArg {
            path: target.clone(),
        });
    }
    let compress_type = arr[1];

    let file = File::create(&target).await?;
    let mut a = Builder::new(file);
    let mut file_count = 0;
    let start = SystemTime::now();

    for entry in glob(&format!("{source}{}", params.pattern))
        .map_err(|err| Error::Pattern { source: err })?
    {
        let file_path = entry.map_err(|err| Error::Glob { source: err })?;
        let filename = file_path
            .strip_prefix(&source)
            .map_err(|err| Error::StripPrefix { source: err })?;
        if file_path.is_dir() {
            continue;
        }

        let file = dir.path().join(uuid());
        debug!(
            file = filename.to_string_lossy().to_string(),
            "start to encode"
        );

        let size = match compress_type {
            GZIP => compression::gzip_encode(&file_path, &file, level).await,
            ZSTD => compression::zstd_encode(&file_path, &file, level).await,
            BROTLI => compression::brotli_encode(&file_path, &file, level).await,
            SNAPPY => compression::snappy_encode(&file_path, &file).await,
            LZ4 => compression::lz4_encode(&file_path, &file).await,
            DEFLATE => compression::deflate_encode(&file_path, &file, level).await,
            XZ => compression::xz_encode(&file_path, &file, level).await,
            _ => Err(Error::InvalidCompression {
                compression: compress_type.to_string(),
            }),
        }?;
        debug!(
            file = filename.to_string_lossy().to_string(),
            size = bytesize::ByteSize(size as u64).to_string(),
            "encode done"
        );
        a.append_file(filename, &mut File::open(&file).await?)
            .await?;
        file_count += 1;
    }
    a.finish().await?;
    let mut duration = None;
    if let Ok(d) = SystemTime::now().duration_since(start) {
        duration = Some(humantime::format_duration(d).to_string());
    };
    let mut file_size = None;
    if let Ok(file) = File::open(&target).await {
        if let Ok(meta) = file.metadata().await {
            file_size = Some(bytesize::ByteSize(meta.len()).to_string());
        }
    }
    info!(
        file = target,
        file_size,
        compression = compress_type,
        level,
        file_count,
        duration,
    );

    Ok(())
}
