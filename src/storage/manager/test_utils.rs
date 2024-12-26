use std::io::{self, Write};

use bytes::Bytes;
use futures_util::Stream;
use rand::RngCore;
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use tokio::{fs::File, io::copy};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::utils::crypto::HashRead;

use super::*;

pub struct TempHolder {
    pub data_dir: TempDir,
    pub temp_dir: TempDir,
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

pub async fn test_store(repo: impl Manager, holder: TempHolder) {
    const SIZE: usize = 3;

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

pub async fn test_delete(repo: impl Manager, holder: TempHolder) {
    const SIZE: usize = 1;

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
