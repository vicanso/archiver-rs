use async_compression::tokio::write::{BrotliEncoder, DeflateEncoder, GzipEncoder, ZstdEncoder};
use async_compression::Level;
use lz4_flex::block::compress_prepend_size;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;
use tokio::io::copy;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::error::Error;

async fn write_file(target: &PathBuf, data: &[u8]) -> Result<(), Error> {
    let mut file = File::create(target).await?;
    file.write_all(data).await?;
    file.flush().await?;
    Ok(())
}

async fn compress<'a, W>(file: &PathBuf, writer: &'a mut W) -> Result<u64, Error>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    let mut r = File::open(file).await?;
    let size = copy(&mut r, writer).await?;
    Ok(size)
}

pub async fn gzip_encode(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = GzipEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    w.shutdown().await?;
    write_file(target, &w.into_inner()).await
}

pub async fn zstd_encode(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = ZstdEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    w.shutdown().await?;
    write_file(target, &w.into_inner()).await
}

pub async fn brotli_encode(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = BrotliEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    w.shutdown().await?;
    write_file(target, &w.into_inner()).await
}

pub async fn deflate_encode(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = DeflateEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    w.shutdown().await?;
    write_file(target, &w.into_inner()).await
}

pub async fn snappy_encode(file: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    let buf = fs::read(file).await?;
    let mut w = snap::raw::Encoder::new();
    write_file(target, &w.compress_vec(&buf)?).await
}

pub async fn lz4_encode(file: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    let buf = fs::read(file).await?;
    write_file(target, &compress_prepend_size(&buf)).await
}
