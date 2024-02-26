use async_compression::tokio::write::{GzipEncoder, ZstdEncoder};
use async_compression::Level;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::copy;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::error::Error;

async fn write_file(target: &PathBuf, data: &[u8]) -> Result<(), Error> {
    let mut file = File::create(target)
        .await
        .map_err(|err| Error::Io { source: err })?;
    file.write_all(data)
        .await
        .map_err(|err| Error::Io { source: err })?;
    file.shutdown()
        .await
        .map_err(|err| Error::Io { source: err })?;
    Ok(())
}

async fn compress<'a, W>(file: &PathBuf, writer: &'a mut W) -> Result<u64, Error>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    let mut r = File::open(file)
        .await
        .map_err(|err| Error::Io { source: err })?;
    let size = copy(&mut r, writer)
        .await
        .map_err(|err| Error::Io { source: err })?;
    Ok(size)
}

pub async fn gzip(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = GzipEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    write_file(target, &w.into_inner()).await
}

pub async fn zstd(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = ZstdEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    write_file(target, &w.into_inner()).await
}
