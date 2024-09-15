use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use astu_resolve::Target;
use futures::Sink;
use futures::SinkExt;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
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

impl FileStore {
    pub fn get_stream(self) -> impl Stream<Item = Result<Target>> {
        self.framed
            .map_ok(|x| Target::from_str(&x))
            .map(futures::stream::iter)
            .flatten()
            .map_err(anyhow::Error::from)
    }

    pub fn get_sink(self) -> impl Sink<Result<Target>> {
        async fn inner(res: Result<Target>) -> Result<String> {
            res.map(|target| target.to_string())
        }
        self.framed.with(inner).sink_map_err(anyhow::Error::from)
    }

    pub fn get_sink_2(self) -> impl Sink<Target> {
        async fn inner(t: Target) -> Result<String> {
            Ok(t.to_string())
        }
        self.framed.with(inner).sink_map_err(anyhow::Error::from)
    }
}
