use super::{Bulkhead, BulkheadError};
use futures_lite::future::Future;
use tokio1 as tokio;

#[doc(cfg(feature = "tokio"))]
impl Bulkhead {
    pub async fn limit<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = tokio::time::timeout(self.max_wait_duration, permit_fut).await??;
        Ok(f.await)
    }
}
