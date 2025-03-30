use std::future::Future;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;

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
