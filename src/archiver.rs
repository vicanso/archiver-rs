use glob::glob;
use std::path::Path;
use std::time::SystemTime;
use tokio::fs::File;
use tokio_tar::Builder;
use tracing::info;
use uuid::{NoContext, Timestamp, Uuid};

use super::compression;
use super::error::Error;

const GZIP: &str = "gz";
const ZSTD: &str = "zst";
const BROTLI: &str = "br";
const LZ4: &str = "lz4";
const SNAPPY: &str = "sz";
const DEFLATE: &str = "zip";

fn uuid() -> String {
    let ts = Timestamp::now(NoContext);
    Uuid::new_v7(ts).to_string()
}

pub async fn archive(original: &str, target: &str, level: i32) -> Result<(), Error> {
    let dir = tempfile::tempdir()?;

    let filename = Path::new(&target)
        .file_name()
        .ok_or(Error::InvalidArg {
            path: target.to_string(),
        })?
        .to_string_lossy();
    let arr: Vec<&str> = filename.split('.').collect();
    if arr.len() < 3 || arr[2] != "tar" {
        return Err(Error::InvalidArg {
            path: target.to_string(),
        });
    }
    let compress_type = arr[1];

    let file = File::create(target).await?;
    let mut a = Builder::new(file);
    let mut file_count = 0;
    let start = SystemTime::now();

    for entry in
        glob(&(original.to_string() + "/**/*")).map_err(|err| Error::Pattern { source: err })?
    {
        let file_path = entry.map_err(|err| Error::Glob { source: err })?;
        let filename = file_path
            .strip_prefix(original)
            .map_err(|err| Error::StripPrefix { source: err })?;
        if file_path.is_dir() {
            continue;
        }

        let file = dir.path().join(uuid());

        match compress_type {
            GZIP => compression::gzip_encode(&file_path, &file, level).await,
            ZSTD => compression::zstd_encode(&file_path, &file, level).await,
            BROTLI => compression::brotli_encode(&file_path, &file, level).await,
            SNAPPY => compression::snappy_encode(&file_path, &file).await,
            LZ4 => compression::lz4_encode(&file_path, &file).await,
            DEFLATE => compression::deflate_encode(&file_path, &file, level).await,
            _ => Err(Error::InvalidCompression {
                compression: compress_type.to_string(),
            }),
        }?;
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
    if let Ok(file) = File::open(target).await {
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
