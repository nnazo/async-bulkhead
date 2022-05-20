use super::{Bulkhead, BulkheadError};
use futures_lite::future::Future;
use smol_timeout::TimeoutExt;

#[doc(cfg(feature = "smol"))]
impl Bulkhead {
    pub async fn limit<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = permit_fut
            .timeout(self.max_wait_duration)
            .await
            .ok_or(BulkheadError::Timeout())?;
        Ok(f.await)
    }
}
