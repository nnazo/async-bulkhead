use super::{Bulkhead, BulkheadError};
use async_std1 as async_std;
use futures_lite::future::Future;

#[doc(cfg(feature = "async-std"))]
impl Bulkhead {
    pub async fn limit<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = async_std::future::timeout(self.max_wait_duration, permit_fut).await?;
        Ok(f.await)
    }

    pub async fn limit_io<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = async_std::io::timeout(self.max_wait_duration, permit_fut).await?;
        Ok(f.await)
    }
}
