use std::str::FromStr;

use async_stream::stream;
use camino::Utf8PathBuf;
use futures::Stream;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio_stream::wrappers::LinesStream;

use crate::Resolve;
use crate::Target;

pub struct FileResolver;

impl Resolve for FileResolver {
    fn resolve(&self, target: Target) -> impl Stream<Item = Target> {
        stream! {
            match target {
                Target::File(path) => {
                    for await target in targets_from_file(path) {
                        yield target;
                    }
                }
                _unsupported => return,
            }
        }
    }
}

fn targets_from_file(path: Utf8PathBuf) -> impl Stream<Item = Target> {
    stream! {
        let Ok(file) = File::open(path).await else {
            return;
        };
        let reader = BufReader::new(file);
        let lines = LinesStream::new(reader.lines());

        for await line in lines {
            let Ok(line) = line else {
                continue;
            };
            if let Ok(t) = Target::from_str(&line.trim()) {
                yield t;
            }
        }
    }
}
