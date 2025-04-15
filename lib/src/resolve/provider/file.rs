use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use async_stream::try_stream;
use futures::stream::BoxStream;
use futures::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tracing::debug;

use crate::resolve::Resolve;
use crate::resolve::Target;

// FIXME: Possibly treat FileResolver specially by injecting a chain
/// Reads targets from lines in a file.
#[derive(Debug, Default, Clone, Copy)]
pub struct FileResolver {
    // FIXME: Use PhantomData to force usage of constructors
}

impl Resolve for FileResolver {
    fn resolve_fallible(&self, target: Target) -> BoxStream<Result<Target>> {
        match target.path() {
            Some(path) => self.resolve_path(path.into()),
            _unsupported => futures::stream::empty().boxed(),
        }
    }
}

impl FileResolver {
    #[allow(clippy::unused_self)]
    fn resolve_path(&self, path: PathBuf) -> BoxStream<Result<Target>> {
        try_stream! {
            let file = File::open(path).await?;
            let file = BufReader::new(file);
            let mut lines = file.lines();

            while let Some(line) = lines.next_line().await? {
                if line.is_empty() {
                    continue;
                }
                match Target::from_str(&line) {
                    Ok(target) => yield target,
                    Err(error) => debug!(?error, %line, "FileResolver: error parsing line"),
                }
            }
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::resolve::ResolveExt;

    fn testfile(file: &str) -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        format!("{manifest_dir}/test/{file}")
    }

    #[rstest]
    #[case(testfile("file.txt"), 8)]
    #[tokio::test]
    async fn resolve_works(#[case] query: String, #[case] num: usize) {
        let target = Target::from_str(&query).unwrap();
        let resolver = FileResolver::default();
        let targets = resolver.resolve_set(target).await;
        assert_eq!(targets.len(), num);
    }
}
