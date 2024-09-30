use std::task::Poll;

use pin_project_lite::pin_project;
use sha2::{digest::Output, Digest};
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
