use std::future::Future;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use futures::FutureExt;
use futures::Stream;
use futures::StreamExt;
use futures::TryFuture;
use futures::TryFutureExt;
use futures::TryStream;
use futures::TryStreamExt;

impl<T: ?Sized> AstuTryFutureExt for T where T: TryFuture {}

/// Adapters specific to [`Result`]-returning futures.
pub trait AstuTryFutureExt: TryFuture {
    /// Converts this future into a single element stream of the [`Result::Ok`]
    /// variant. The [`Result::Err`] variant will not be yielded, and in that
    /// case the stream will complete immediately.
    ///
    /// Accepts an inspector to, for example, log the error.
    fn into_stream_infallible<I>(self, inspector: I) -> impl Stream<Item = Self::Ok>
    where
        Self: Sized,
        I: FnOnce(&Self::Error),
    {
        self.inspect_err(inspector)
            .into_stream()
            .map(futures::stream::iter)
            .flatten()
    }
}

impl<T: ?Sized> AstuTryStreamExt for T where T: TryStream {}

/// Adapters specific to [`Result`]-returning streams.
pub trait AstuTryStreamExt: TryStream {
    /// Inspects and drops the `Error` variant, returning only the `Ok` variant.
    fn flatten_err<I>(self, inspector: I) -> impl Stream<Item = Self::Ok>
    where
        Self: Sized,
        I: FnMut(&Self::Error),
    {
        self.inspect_err(inspector)
            .map(futures::stream::iter)
            .flatten()
    }
}

/// Spawns a future as a Tokio task and applies a timeout to it.
///
/// # Errors
///
/// - If the timeout is reached
/// - If the task fails to join
pub async fn spawn_timeout<T>(
    duration: Duration,
    future: impl Future<Output = T> + Send + 'static,
) -> Result<T>
where
    T: Send + 'static,
{
    let task = tokio::spawn(future);
    let timeout = tokio::time::timeout(duration, task);
    let t = timeout
        .await
        .context("tokio task timed out")?
        .context("tokio task failed to join")?;
    Ok(t)
}
