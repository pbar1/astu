use std::borrow::BorrowMut;
use std::path::Path;
use std::pin::Pin;

use anyhow::Result;
use futures::task::Context;
use futures::task::Poll;
use futures::Sink;
use futures::Stream;
use tokio::fs::File;
use tokio_util::codec::Framed;
use tokio_util::codec::LinesCodec;

pub struct FileStore {
    framed: Framed<File, LinesCodec>,
}

/// Constructors
impl FileStore {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path).await?;
        let codec = LinesCodec::new();
        let framed = Framed::new(file, codec);
        Ok(Self { framed })
    }

    pub async fn stdin() -> Result<Self> {
        Self::new("/dev/fd/0").await
    }

    pub async fn stdout() -> Result<Self> {
        Self::new("/dev/fd/1").await
    }

    pub async fn stderr() -> Result<Self> {
        Self::new("/dev/fd/2").await
    }
}

impl Stream for FileStore {
    type Item = Result<String>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let framed = self.get_mut().framed.borrow_mut();
        let framed: Pin<&mut Framed<tokio::fs::File, LinesCodec>> = Pin::new(framed);
        <Framed<tokio::fs::File, LinesCodec> as futures::Stream>::poll_next(framed, cx)
            .map_err(From::from)
    }
}

impl<T> Sink<T> for FileStore
where
    T: AsRef<str>,
{
    type Error = anyhow::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let framed = self.get_mut().framed.borrow_mut();
        let framed: Pin<&mut Framed<tokio::fs::File, LinesCodec>> = Pin::new(framed);
        <Framed<tokio::fs::File, LinesCodec> as futures::Sink<String>>::poll_ready(framed, cx)
            .map_err(From::from)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<()> {
        let framed = self.get_mut().framed.borrow_mut();
        let framed: Pin<&mut Framed<tokio::fs::File, LinesCodec>> = Pin::new(framed);
        <Framed<tokio::fs::File, LinesCodec> as futures::Sink<T>>::start_send(framed, item)
            .map_err(From::from)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let framed = self.get_mut().framed.borrow_mut();
        let framed: Pin<&mut Framed<tokio::fs::File, LinesCodec>> = Pin::new(framed);
        <Framed<tokio::fs::File, LinesCodec> as futures::Sink<String>>::poll_flush(framed, cx)
            .map_err(From::from)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let framed = self.get_mut().framed.borrow_mut();
        let framed: Pin<&mut Framed<tokio::fs::File, LinesCodec>> = Pin::new(framed);
        <Framed<tokio::fs::File, LinesCodec> as futures::Sink<String>>::poll_close(framed, cx)
            .map_err(From::from)
    }
}

#[cfg(test)]
mod tests {

    use futures::SinkExt;

    use super::*;

    #[tokio::test]
    async fn sink_works() {
        let mut store = FileStore::stdout().await.unwrap();
        store.send("hello").await.unwrap();
        store.send("world").await.unwrap();
    }
}
