use std::str::FromStr;

use anyhow::bail;
use camino::Utf8PathBuf;
use futures::FutureExt;
use futures::Stream;
use futures::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio_stream::wrappers::LinesStream;

use crate::Resolve;
use crate::ResolveResult;
use crate::Target;

pub struct FileResolver;

impl Resolve for FileResolver {
    fn resolve(&self, target: Target) -> ResolveResult {
        let path = match target {
            Target::File(path) => path,
            unsupported => bail!("FileResolver: unsupported target: {unsupported}"),
        };

        // TODO: The flattens drop errors
        let stream = stream_nonempty_lines(path)
            .into_stream()
            .map(futures::stream::iter)
            .flatten()
            .flatten()
            .map(|x| Target::from_str(&x))
            .boxed();

        Ok(stream)
    }
}

async fn stream_nonempty_lines(path: Utf8PathBuf) -> anyhow::Result<impl Stream<Item = String>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let lines = LinesStream::new(reader.lines());

    let stream = lines
        .map(futures::stream::iter)
        .flatten()
        .filter(|x| futures::future::ready(!x.is_empty()));

    Ok(stream)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::ResolveExt;

    fn testfile(file: &str) -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        format!("{manifest_dir}/test/{file}")
    }

    #[rstest]
    #[case(testfile("file.txt"), 8)]
    #[tokio::test]
    async fn resolve_works(#[case] query: String, #[case] num: usize) {
        let target = Target::from_str(&query).unwrap();
        let resolver = FileResolver;
        let targets: BTreeSet<Target> = resolver.resolve_infallible(target).collect().await;
        assert_eq!(targets.len(), num);
    }
}
