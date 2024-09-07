#![warn(clippy::pedantic)]

use futures::FutureExt;
use futures::Stream;
use futures::StreamExt;
use futures::TryFuture;
use futures::TryFutureExt;

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
