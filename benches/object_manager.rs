use std::{io, task::Poll};

use bytes::{Bytes, BytesMut};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use downloader::{
    config::StorageConfig,
    storage::manager::{Manager, ObjectManager},
    utils::serde::ResolvedPath,
};
use futures_util::Stream;
use rand::RngCore;
use tempfile::TempDir;
use uuid::Uuid;

#[derive(Clone)]
struct InMemStream {
    chunk_size: usize,
    data: Bytes,
}

impl Stream for InMemStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let res = {
            if self.data.len() == 0 {
                None
            } else if self.data.len() <= self.chunk_size {
                let len = self.data.len();
                Some(Ok(self.data.split_to(len)))
            } else {
                let cs = self.chunk_size;
                Some(Ok(self.data.split_to(cs)))
            }
        };
        Poll::Ready(res)
    }
}

#[allow(dead_code, reason = "this is a struct to hold ownership of data")]
struct TempHolder {
    data_dir: TempDir,
    temp_dir: TempDir,
}

fn repository() -> (ObjectManager, TempHolder) {
    let _data_dir = tempfile::tempdir().unwrap();
    let _temp_dir = tempfile::tempdir().unwrap();

    let data_dir =
        ResolvedPath::new(_data_dir.path().to_string_lossy().into_owned())
            .unwrap();

    let temp_dir =
        ResolvedPath::new(_temp_dir.path().to_string_lossy().into_owned())
            .unwrap();

    let state_dir = data_dir.clone();

    (
        ObjectManager::new(&StorageConfig {
            state_dir,
            data_dir,
            temp_dir,
        }),
        TempHolder {
            data_dir: _data_dir,
            temp_dir: _temp_dir,
        },
    )
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
}

fn create_stream(size: usize) -> InMemStream {
    const MAX_CHUNK: usize = 8 * 1024;
    let mut buf = BytesMut::zeroed(size);
    rand::thread_rng().fill_bytes(&mut buf);

    let data = buf.freeze();

    InMemStream {
        data,
        chunk_size: MAX_CHUNK,
    }
}

fn benchmark_store_with_size(c: &mut Criterion, size: usize) {
    let stream = create_stream(size);
    let (repo, _holder) = repository();
    let id = Uuid::new_v4();

    let size_fmt = format!("{} MiB", size as f64 / (1024.0 * 1024.0));
    let bench_id = BenchmarkId::new("store", size_fmt);

    c.bench_with_input(bench_id, &repo, |b, r| {
        b.to_async(runtime()).iter(|| async {
            r.store(id, stream.clone()).await.unwrap();
        });
    });
}

fn benchmark_store(c: &mut Criterion) {
    benchmark_store_with_size(c, 1024);
    benchmark_store_with_size(c, 1024 * 1024);
    // benchmark_store_with_size(c, 128 * 1024 * 1024);
}

criterion_group!(
    name = object_manager;
    config = Criterion::default();
    targets = benchmark_store
);
criterion_main!(object_manager);
