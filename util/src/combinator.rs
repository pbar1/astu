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
