use super::{Bulkhead, BulkheadError};
use async_std::future::{timeout, TimeoutError};
use futures_lite::future::Future;

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

impl From<TimeoutError> for BulkheadError {
    fn from(_err: TimeoutError) -> Self {
        Self::Timeout
    }
}
