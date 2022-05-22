use super::{Bulkhead, BulkheadError};
use futures_lite::future::Future;
use tokio::time::{error, timeout};

impl Bulkhead {
    /// Limits the number of concurrent calls using a semaphore with the
    /// specified maximum concurrent calls and semaphore wait duration.
    ///
    /// When the semaphore permit can't be acquired before the specified duration,
    /// the `Err(BulkheadError::Timeout)` value is returned.
    pub async fn limit<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = timeout(self.max_wait_duration, permit_fut).await?;
        Ok(f.await)
    }
}

impl From<error::Elapsed> for BulkheadError {
    fn from(_err: error::Elapsed) -> Self {
        Self::Timeout
    }
}
