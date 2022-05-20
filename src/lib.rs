#[cfg(feature = "async-std")]
use async_std1 as async_std;
use futures_lite::future::Future;
#[cfg(feature = "smol")]
use smol_timeout::TimeoutExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
#[cfg(feature = "tokio")]
use tokio1 as tokio;

#[derive(Debug, Error)]
pub enum BulkheadError {
    #[cfg(feature = "tokio")]
    #[error("the bulkhead semaphore has been closed")]
    Acquire(#[from] tokio::sync::AcquireError),
    #[error("the maximum number of concurrent calls is met")]
    Timeout(
        #[cfg(feature = "tokio")]
        #[from]
        tokio::time::error::Elapsed,
        #[cfg(all(not(feature = "tokio"), feature = "async-std"))]
        #[from]
        async_std::future::TimeoutError,
    ),
}

#[derive(Debug, Copy, Clone)]
pub struct BulkheadBuilder {
    max_concurrent_calls: usize,
    max_wait_duration: Duration,
}

impl BulkheadBuilder {
    pub fn max_concurrent_calls(mut self, max_concurrent_calls: usize) -> Self {
        self.max_concurrent_calls = max_concurrent_calls;
        self
    }

    pub fn max_wait_duration(mut self, max_wait_duration: Duration) -> Self {
        self.max_wait_duration = max_wait_duration;
        self
    }

    pub fn build(self) -> Bulkhead {
        Bulkhead {
            #[cfg(feature = "tokio")]
            max_concurrent_calls: Arc::new(tokio::sync::Semaphore::new(self.max_concurrent_calls)),
            #[cfg(all(not(feature = "tokio"), any(feature = "async-std", feature = "smol")))]
            max_concurrent_calls: Arc::new(async_lock::Semaphore::new(self.max_concurrent_calls)),
            max_wait_duration: self.max_wait_duration,
        }
    }
}

impl Default for BulkheadBuilder {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 25,
            max_wait_duration: Duration::from_millis(1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bulkhead {
    #[cfg(feature = "tokio")]
    max_concurrent_calls: Arc<tokio::sync::Semaphore>,
    #[cfg(all(not(feature = "tokio"), any(feature = "async-std", feature = "smol")))]
    max_concurrent_calls: Arc<async_lock::Semaphore>,
    max_wait_duration: Duration,
}

impl Bulkhead {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> BulkheadBuilder {
        BulkheadBuilder::default()
    }

    #[cfg(feature = "tokio")]
    pub async fn limit<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = tokio::time::timeout(self.max_wait_duration, permit_fut).await??;
        Ok(f.await)
    }

    #[cfg(all(not(any(feature = "tokio", feature = "smol")), feature = "async-std"))]
    pub async fn limit<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = async_std::future::timeout(self.max_wait_duration, permit_fut).await?;
        Ok(f.await)
    }

    #[cfg(all(not(any(feature = "tokio", feature = "async-std")), feature = "smol"))]
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

impl Default for Bulkhead {
    fn default() -> Self {
        let max_concurrent_calls = 25;
        Self {
            #[cfg(feature = "tokio")]
            max_concurrent_calls: Arc::new(tokio::sync::Semaphore::new(max_concurrent_calls)),
            #[cfg(all(not(feature = "tokio"), any(feature = "async-std", feature = "smol")))]
            max_concurrent_calls: Arc::new(async_lock::Semaphore::new(max_concurrent_calls)),
            max_wait_duration: Duration::from_millis(1),
        }
    }
}

#[derive(Debug)]
pub struct Bulkheads(HashMap<String, Bulkhead>);

impl Bulkheads {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn register(&mut self, name: String, bulkhead: Bulkhead) -> &mut Self {
        self.0.insert(name, bulkhead);
        self
    }

    pub fn get(&self, name: &str) -> Option<&Bulkhead> {
        self.0.get(name)
    }
}

impl Default for Bulkheads {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(feature = "tokio")]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use tokio1 as tokio;

    async fn two_calls_test_helper(bulkhead: Bulkhead) -> Result<(), BulkheadError> {
        let bulkhead_clone = bulkhead.clone();
        let handle = tokio::spawn(async move {
            let sleep_fut = tokio::time::sleep(Duration::from_millis(50));
            bulkhead_clone.limit(sleep_fut).await
        });
        let result_fut = async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            bulkhead.limit(async {}).await
        };
        let (result, _) = tokio::join!(result_fut, handle);
        result
    }

    #[tokio::test]
    pub async fn times_out() {
        let bulkhead = Bulkhead::builder().max_concurrent_calls(1).build();
        let result = two_calls_test_helper(bulkhead).await;
        assert_matches!(result, Err(BulkheadError::Timeout(_)));
    }

    #[tokio::test]
    pub async fn doesnt_time_out() {
        let bulkhead = Bulkhead::default();
        let result = bulkhead.limit(async {}).await;
        assert_matches!(result, Ok(_));
    }

    #[tokio::test]
    pub async fn doesnt_time_out_long() {
        let bulkhead = Bulkhead::builder()
            .max_concurrent_calls(1)
            .max_wait_duration(Duration::from_secs(2))
            .build();
        let result = two_calls_test_helper(bulkhead).await;
        assert_matches!(result, Ok(_));
    }
}
