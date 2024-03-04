use async_compression::tokio::write::{
    BrotliEncoder, DeflateEncoder, GzipDecoder, GzipEncoder, ZstdEncoder,
};
use async_compression::Level;
use filetime::{set_file_mtime, FileTime};
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

async fn copy_mtime(file: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    let f = File::open(file).await?;
    let meta = f.metadata().await?;
    set_file_mtime(target, FileTime::from_last_modification_time(&meta))?;
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
    let _ = write_file(target, &w.into_inner()).await;
    copy_mtime(file, target).await
}

pub async fn zstd_encode(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = ZstdEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    w.shutdown().await?;
    let _ = write_file(target, &w.into_inner()).await;
    copy_mtime(file, target).await
}

pub async fn brotli_encode(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = BrotliEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    w.shutdown().await?;
    let _ = write_file(target, &w.into_inner()).await;
    copy_mtime(file, target).await
}

pub async fn deflate_encode(file: &PathBuf, target: &PathBuf, level: i32) -> Result<(), Error> {
    let mut w = DeflateEncoder::with_quality(Vec::new(), Level::Precise(level));
    compress(file, &mut w).await?;
    w.shutdown().await?;
    let _ = write_file(target, &w.into_inner()).await;
    copy_mtime(file, target).await
}

pub async fn snappy_encode(file: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    let buf = fs::read(file).await?;
    let mut w = snap::raw::Encoder::new();
    let _ = write_file(target, &w.compress_vec(&buf)?).await;
    copy_mtime(file, target).await
}

pub async fn lz4_encode(file: &PathBuf, target: &PathBuf) -> Result<(), Error> {
    let buf = fs::read(file).await?;
    let _ = write_file(target, &compress_prepend_size(&buf)).await;
    copy_mtime(file, target).await
}

pub async fn gzip_decode(
    file: &mut tokio_tar::Entry<tokio_tar::Archive<tokio::fs::File>>,
    target: &PathBuf,
) -> Result<(), Error> {
    let mut w = GzipDecoder::new(Vec::new());
    let _ = copy(file, &mut w).await?;
    let _ = write_file(target, &w.into_inner()).await;
    // let meta = file.header().g.await?;
    // set_file_mtime(target, FileTime::from_last_modification_time(&meta))?;

    // compress(file, &mut w).await?;
    // w.shutdown().await?;
    // let _ = write_file(target, &w.into_inner()).await;
    // copy_mtime(file, target).await
    Ok(())
}
