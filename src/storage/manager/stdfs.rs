use std::{
    io::{self, ErrorKind},
    path::PathBuf,
    time::Instant,
};

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use sha2::Sha256;
use tokio::{
    fs::{remove_file, rename, File},
    io::{AsyncRead, AsyncWriteExt},
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

use super::{Manager, ObjectError};

pub struct SyncFsManager {
    data_dir: PathBuf,
    temp_dir: PathBuf,
}

impl SyncFsManager {
    pub fn new(cfg: &StorageConfig) -> Self {
        Self {
            data_dir: PathBuf::from(cfg.data_dir.as_str()),
            temp_dir: PathBuf::from(cfg.temp_dir.as_str()),
        }
    }
}

impl Manager for SyncFsManager {
    #[instrument(target = "object_fs", name = "store", skip(self, stream))]
    async fn store(
        &self,
        id: Uuid,
        stream: impl Stream<Item = Result<Bytes, io::Error>> + Unpin + Send,
    ) -> Result<(u64, [u8; 32]), ObjectError> {
        let mut stream = HashStream::<_, Sha256>::new(stream);

        let start = Instant::now();

        tracing::info!(target: "object_fs", "starting store");

        let id = id.to_string();
        let temp_dir = self.temp_dir.join(format!("{id}-incomplete"));

        let mut file = File::create(&temp_dir).await.inspect_err(|error| {
            tracing::error!(
                target: "object_fs",
                %error,
                path = ?temp_dir,
                took = %fmt_since(start),
                "create file failed",
            );
        })?;

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
    async fn fetch(
        &self,
        id: Uuid,
    ) -> Result<impl AsyncRead + Unpin + Send + 'static, ObjectError> {
        let start = Instant::now();

        tracing::info!(target: "object_fs", "starting fetch");

        let id = id.to_string();
        let path = self.data_dir.join(&id);

        let mut file = File::open(&path).await.map_err(|error| {
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

        let max_buf_size = buffer_cap(file_size.unwrap_or_default()) as usize;
        file.set_max_buf_size(max_buf_size);

        Ok(file)
    }

    #[instrument(target = "object_fs", name = "delete", skip(self))]
    async fn delete(&self, id: Uuid) -> Result<(), ObjectError> {
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
const fn buffer_cap(file_size: u64) -> usize {
    const DEFAULT_BUFFER_CAP: usize = 2 * 1024 * 1024;

    if file_size >= 1024 * 1024 * 1024 {
        8 * 1024 * 1024
    } else {
        DEFAULT_BUFFER_CAP
    }
}

pub async fn copy_impl<S>(mut stream: S, file: &mut File) -> io::Result<u64>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Unpin,
{
    let mut n = 0;
    while let Some(res) = stream.next().await {
        match res {
            Ok(v) => {
                file.write_all(&v).await?;
                n += v.len();
            }
            Err(err) => return Err(err),
        }
    }

    file.flush().await?;
    file.sync_all().await?;
    Ok(n as u64)
}

#[cfg(test)]
mod tests {
    use crate::storage::manager::test_utils::TempHolder;

    use super::*;

    fn repository() -> (SyncFsManager, TempHolder) {
        let data_dir = tempfile::tempdir().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();

        (
            SyncFsManager {
                data_dir: data_dir.path().to_owned(),
                temp_dir: temp_dir.path().to_owned(),
            },
            TempHolder { data_dir, temp_dir },
        )
    }

    macro_rules! impl_test {
        ($name: ident) => {
            #[test_log::test(tokio::test)]
            async fn $name() {
                let (repo, holder) = repository();
                crate::storage::manager::test_utils::$name(repo, holder).await;
            }
        };
    }

    impl_test!(test_store);
    impl_test!(test_delete);
}
