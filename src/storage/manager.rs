use std::{
    io::{self, ErrorKind},
    path::PathBuf,
    time::Instant,
};

use axum::http::StatusCode;
use sha2::Sha256;
use tokio::{
    fs::{remove_file, rename, File},
    io::{copy, AsyncRead},
};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    config::StorageConfig,
    utils::{crypto::HashRead, fmt::fmt_since},
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
    #[instrument(target = "object_fs", name = "store", skip(self, reader))]
    pub async fn store(
        &self,
        id: Uuid,
        reader: impl AsyncRead + Unpin,
    ) -> Result<(u64, [u8; 32]), ObjectError> {
        let mut reader = HashRead::<_, Sha256>::new(reader);

        let start = Instant::now();

        tracing::info!(target: "object_fs", "starting store");

        let id = id.to_string();
        let temp_dir = self.temp_dir.join(format!("{id}-incomplete"));

        let mut file = File::create(&temp_dir).await.map_err(|error| {
            tracing::error!(
                target: "object_fs",
                %error,
                path = ?temp_dir,
                took = %fmt_since(start),
                "create file failed",
            );
            error
        })?;

        let size = copy(&mut reader, &mut file).await.map_err(|error| {
            tracing::warn!(
                target: "object_fs",
                %error,
                took = %fmt_since(start),
                "interrupted by IO",
            );
            error
        })?;

        let def_dir = self.data_dir.join(&id);

        let rename_res = rename(&temp_dir, &def_dir).await.map_err(|error| {
            tracing::error!(
                target: "object_fs",
                %error,
                took = %fmt_since(start),
                "move file failed",
            );
            error
        });

        if let Err(err) = rename_res {
            let _ = remove_file(&temp_dir).await.map_err(|error| {
                tracing::error!(
                    target: "object_fs",
                    %error,
                    path = ?temp_dir,
                    took = %fmt_since(start),
                    "delete file after move failed",
                );
            });

            return Err(err.into());
        }

        let hash: [u8; 32] = reader.hash_into();

        tracing::info!(
            target: "object_fs",
            took = %fmt_since(start),
            written_bytes = size,
            hash = %format!("{hash:02x?}"),
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
            tracing::error!(
                target: "object_fs",
                %error,
                took = %fmt_since(start),
                path = ?path,
                "open file failed",
            );
            if error.kind() == ErrorKind::NotFound {
                ObjectError::NotFound
            } else {
                ObjectError::IoError(error)
            }
        })?;

        tracing::info!(
            target: "object_fs",
            took = %fmt_since(start),
            "fetched file stream",
        );

        Ok(file)
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use rand::RngCore;
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;
    use tokio::{
        fs::File,
        io::{copy, AsyncRead},
    };
    use uuid::Uuid;

    use crate::{storage::manager::ObjectError, utils::crypto::HashRead};

    use super::ObjectManager;

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
    ) -> (impl AsyncRead + Unpin, [u8; 32]) {
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

        (file, hash)
    }

    #[tokio::test]
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

    #[tokio::test]
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
