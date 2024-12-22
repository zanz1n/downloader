use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::Stream;
use jsonwebtoken::{DecodingKey, EncodingKey};
use pin_project_lite::pin_project;
use sha2::{digest::Output, Digest};
use sqlx::error::BoxDynError;
use tokio::io::AsyncRead;

pin_project! {
    pub struct HashRead<T, H> {
        #[pin]
        read: T,
        hasher: H,
    }
}

impl<T, H: Digest> HashRead<T, H> {
    pub fn new(read: T) -> Self {
        let hasher = H::new();
        Self { read, hasher }
    }

    #[inline]
    pub fn hash(self) -> Output<H> {
        self.hasher.finalize()
    }

    #[inline]
    pub fn hash_into<I: From<Output<H>>>(self) -> I {
        self.hash().into()
    }
}

impl<T: AsyncRead, H: Digest> AsyncRead for HashRead<T, H> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.project();
        let before_len = buf.filled().len();

        match this.read.poll_read(cx, buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Ready(Ok(())) => {
                let filled = buf.filled();
                let after_len = filled.len();

                if after_len > before_len {
                    let new = &filled[before_len..];
                    this.hasher.update(new);
                }

                Poll::Ready(Ok(()))
            }
        }
    }
}

pin_project! {
    pub struct HashStream<S, H> {
        #[pin]
        stream: S,
        hasher: H,
    }
}

impl<S, H: Digest> HashStream<S, H> {
    pub fn new(stream: S) -> Self {
        let hasher = H::new();
        Self { stream, hasher }
    }

    #[inline]
    pub fn hash(self) -> Output<H> {
        self.hasher.finalize()
    }

    #[inline]
    pub fn hash_into<I: From<Output<H>>>(self) -> I {
        self.hash().into()
    }
}

impl<S, H, E> Stream for HashStream<S, H>
where
    S: Stream<Item = Result<Bytes, E>>,
    H: Digest,
{
    type Item = Result<Bytes, E>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let poll = this.stream.poll_next(cx);
        if let Poll::Ready(Some(Ok(v))) = &poll {
            this.hasher.update(&v);
        }
        poll
    }
}

pub async fn fetch_jwt_key_files(
    public_key: &str,
    private_key: &str,
) -> Result<(EncodingKey, DecodingKey), BoxDynError> {
    let public_key = tokio::fs::read(public_key).await?;
    let public_key = DecodingKey::from_ed_pem(&public_key)?;

    let private_key = tokio::fs::read(private_key).await?;
    let private_key = EncodingKey::from_ed_pem(&private_key)?;

    Ok((private_key, public_key))
}
