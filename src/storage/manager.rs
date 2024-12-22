use std::{
    io::{self, ErrorKind},
    path::PathBuf,
    time::Instant,
};

use axum::http::StatusCode;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use sha2::Sha256;
use tokio::{
    fs::{remove_file, rename, File},
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    config::StorageConfig,
    utils::{
        crypto::HashStream,
        fmt::{fmt_hex, fmt_since},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum ObjectError {
    #[error("io error in file system: {0}")]
    IoError(#[from] io::Error),
    #[error("file not found")]
    NotFound,
}

impl ObjectError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            ObjectError::IoError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ObjectError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    #[inline]
    pub fn custom_code(&self) -> u8 {
        match self {
            ObjectError::IoError(..) => 1,
            ObjectError::NotFound => 2,
        }
    }
}

pub struct ObjectManager {
    data_dir: PathBuf,
    temp_dir: PathBuf,
}

impl ObjectManager {
    pub fn new(cfg: &StorageConfig) -> Self {
        Self {
            data_dir: PathBuf::from(cfg.data_dir.as_str()),
            temp_dir: PathBuf::from(cfg.temp_dir.as_str()),
        }
    }
}

impl ObjectManager {
    #[instrument(target = "object_fs", name = "store", skip(self, stream))]
    pub async fn store(
        &self,
        id: Uuid,
        stream: impl Stream<Item = Result<Bytes, io::Error>> + Unpin,
    ) -> Result<(u64, [u8; 32]), ObjectError> {
        let mut stream = HashStream::<_, Sha256>::new(stream);

        let start = Instant::now();

        tracing::info!(target: "object_fs", "starting store");

        let id = id.to_string();
        let temp_dir = self.temp_dir.join(format!("{id}-incomplete"));

        let file = File::create(&temp_dir).await.inspect_err(|error| {
            tracing::error!(
                target: "object_fs",
                %error,
                path = ?temp_dir,
                took = %fmt_since(start),
                "create file failed",
            );
        })?;

        let mut file = BufWriter::with_capacity(1024 * 1024, file);

        let size = match copy_impl(&mut stream, &mut file).await {
            Ok(v) => v,
            Err(error) => {
                tracing::warn!(
                    target: "object_fs",
                    %error,
                    took = %fmt_since(start),
                    "interrupted by IO",
                );

                let _ = remove_file(&temp_dir).await.map_err(|error| {
                    tracing::error!(
                        target: "object_fs",
                        %error,
                        path = ?temp_dir,
                        took = %fmt_since(start),
                        "delete file after IO interruption failed",
                    );
                });

                return Err(error.into());
            }
        };

        let def_dir = self.data_dir.join(&id);

        if let Err(error) = rename(&temp_dir, &def_dir).await {
            tracing::error!(
                target: "object_fs",
                %error,
                took = %fmt_since(start),
                "move file failed",
            );

            let _ = remove_file(&temp_dir).await.map_err(|error| {
                tracing::error!(
                    target: "object_fs",
                    %error,
                    path = ?temp_dir,
                    took = %fmt_since(start),
                    "delete file after move failed",
                );
            });

            return Err(error.into());
        }

        let hash: [u8; 32] = stream.hash_into();

        tracing::info!(
            target: "object_fs",
            took = %fmt_since(start),
            written_bytes = size,
            hash = %fmt_hex(&hash),
            "finished store",
        );

        Ok((size, hash))
    }

    #[instrument(target = "object_fs", name = "fetch", skip(self))]
    pub async fn fetch(
        &self,
        id: Uuid,
    ) -> Result<impl AsyncRead + Unpin, ObjectError> {
        let start = Instant::now();

        tracing::info!(target: "object_fs", "starting fetch");

        let id = id.to_string();
        let path = self.data_dir.join(&id);

        let file = File::open(&path).await.map_err(|error| {
            if error.kind() == ErrorKind::NotFound {
                ObjectError::NotFound
            } else {
                tracing::error!(
                    target: "object_fs",
                    %error,
                    took = %fmt_since(start),
                    path = ?path,
                    "open file failed",
                );
                ObjectError::IoError(error)
            }
        })?;

        let file_size = file
            .metadata()
            .await
            .map(|meta| meta.len())
            .inspect_err(|error| {
                tracing::error!(
                    target: "object_fs",
                    %error,
                    took = %fmt_since(start),
                    path = ?path,
                    "fetch file metadata failed",
                );
            })
            .ok();

        debug_assert_ne!(file_size, None);

        tracing::info!(
            target: "object_fs",
            took = %fmt_since(start),
            "fetched file stream",
        );

        let buf_cap = buffer_cap(file_size) as usize;

        Ok(BufReader::with_capacity(buf_cap, file))
    }

    #[instrument(target = "object_fs", name = "delete", skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), ObjectError> {
        let start = Instant::now();

        tracing::info!(target: "object_fs", "starting delete");

        let id = id.to_string();
        let path = self.data_dir.join(&id);

        remove_file(&path).await.map_err(|error| {
            tracing::error!(
                target: "object_fs",
                %error,
                took = %fmt_since(start),
                path = ?path,
                "delete file failed",
            );
            if error.kind() == ErrorKind::NotFound {
                ObjectError::NotFound
            } else {
                ObjectError::IoError(error)
            }
        })?;

        Ok(())
    }
}

#[inline]
const fn buffer_cap(file_size: Option<u64>) -> u64 {
    const DEFAULT_BUFFER_CAP: u64 = 8 * 1024;

    if let Some(file_size) = file_size {
        if file_size >= 1024 * 1024 * 1024 {
            8 * 1024 * 1024
        } else if file_size >= 8 * 1024 * 1024 {
            1024 * 1024
        } else if file_size >= 1024 * 1024 {
            128 * 1024
        } else {
            DEFAULT_BUFFER_CAP
        }
    } else {
        DEFAULT_BUFFER_CAP
    }
}

pub(super) async fn copy_impl<S, W>(
    stream: &mut S,
    writer: &mut W,
) -> io::Result<u64>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut n = 0;
    while let Some(res) = stream.next().await {
        match res {
            Ok(v) => {
                writer.write_all(&v).await?;
                n += v.len();
            }
            Err(err) => return Err(err),
        }
    }

    writer.flush().await?;
    Ok(n as u64)
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};

    use bytes::Bytes;
    use futures_util::Stream;
    use rand::RngCore;
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;
    use test_log::test;
    use tokio::{fs::File, io::copy};
    use tokio_util::io::ReaderStream;
    use uuid::Uuid;

    use crate::utils::crypto::HashRead;

    use super::*;

    #[allow(dead_code, reason = "this is a struct to hold ownership of data")]
    struct TempHolder {
        data_dir: TempDir,
        temp_dir: TempDir,
    }

    fn repository() -> (ObjectManager, TempHolder) {
        let data_dir = tempfile::tempdir().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();

        (
            ObjectManager {
                data_dir: data_dir.path().to_owned(),
                temp_dir: temp_dir.path().to_owned(),
            },
            TempHolder { data_dir, temp_dir },
        )
    }

    /// size is in MB
    async fn create_rand_file(
        holder: &TempHolder,
        size: usize,
    ) -> (
        impl Stream<Item = Result<Bytes, io::Error>> + Unpin,
        [u8; 32],
    ) {
        // Intentionally not 1024 * 1024
        // To detect wrong offsets while copying IO data
        let mut buf = vec![0u8; 1000 * 1000];

        let path = holder.temp_dir.path().join(Uuid::new_v4().to_string());
        let mut file = std::fs::File::create(&path).unwrap();

        let mut thread_rng = rand::thread_rng();
        let mut hash = Sha256::new();

        for _ in 0..size {
            thread_rng.fill_bytes(&mut buf);
            hash.update(&buf);

            file.write(&buf).unwrap();
        }

        let file = File::open(path).await.unwrap();
        let hash: [u8; 32] = hash.finalize().into();

        (ReaderStream::with_capacity(file, 8192), hash)
    }

    #[test(tokio::test)]
    async fn test_store() {
        const SIZE: usize = 3;

        let (repo, holder) = repository();

        let (reader, reader_hash) = create_rand_file(&holder, SIZE).await;
        let id = Uuid::new_v4();
        let (written, store_hash) = repo.store(id, reader).await.unwrap();

        assert!(
            reader_hash.iter().eq(store_hash.iter()),
            "generated incorrect sha256 hash for input"
        );
        assert_eq!(
            written,
            (SIZE as u64) * 1000 * 1000,
            "returned incorrect number of written bytes"
        );

        let reader = repo.fetch(id).await.unwrap();
        let mut reader = HashRead::<_, Sha256>::new(reader);

        let mut dev_null = File::from_std(tempfile::tempfile().unwrap());

        let written = copy(&mut reader, &mut dev_null).await.unwrap();
        let fetch_hash: [u8; 32] = reader.hash_into();

        assert_eq!(
            written,
            (SIZE as u64) * 1000 * 1000,
            "returned incorrect number of written bytes"
        );
        assert!(
            reader_hash.iter().eq(fetch_hash.iter()),
            "stream hash mismatches the created file one",
        );
    }

    #[test(tokio::test)]
    async fn test_delete() {
        const SIZE: usize = 1;

        let (repo, holder) = repository();

        let id = Uuid::new_v4();

        let file_res = repo.fetch(id).await;
        assert!(
            matches!(file_res, Err(e) if matches!(e, ObjectError::NotFound)),
            "expected ObjectError::NotFound for inexistent file",
        );

        let (reader, _) = create_rand_file(&holder, SIZE).await;
        repo.store(id, reader).await.unwrap();

        repo.fetch(id).await.expect("could not fetch created file");
        repo.delete(id)
            .await
            .expect("could not delete created file");

        let file_res = repo.fetch(id).await;
        assert!(
            matches!(file_res, Err(e) if matches!(e, ObjectError::NotFound)),
            "expected ObjectError::NotFound for deleted file",
        );
    }
}
