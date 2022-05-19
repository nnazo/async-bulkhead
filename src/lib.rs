#[cfg(feature = "async-std")]
use async_std1 as async_std;
use futures_lite::future::Future;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "tokio")]
use std::time::Instant;
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
        #[cfg(not(feature = "tokio"))]
        #[cfg(feature = "async-std")]
        #[from]
        async_std::future::TimeoutError,
    ),
}

#[derive(Debug, Copy, Clone)]
pub struct BulkheadConfig {
    max_concurrent_calls: usize,
    max_wait_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct Bulkhead {
    #[cfg(feature = "tokio")]
    max_concurrent_calls: Arc<tokio::sync::Semaphore>,
    #[cfg(not(feature = "tokio"))]
    #[cfg(feature = "async-std")]
    max_concurrent_calls: Arc<async_lock::Semaphore>,
    max_wait_duration: Duration,
}

impl Bulkhead {
    pub fn new(max_concurrent_calls: usize, max_wait_duration: Duration) -> Self {
        Self {
            #[cfg(feature = "tokio")]
            max_concurrent_calls: Arc::new(tokio::sync::Semaphore::new(max_concurrent_calls)),
            #[cfg(not(feature = "tokio"))]
            #[cfg(feature = "async-std")]
            max_concurrent_calls: Arc::new(async_lock::Semaphore::new(max_concurrent_calls)),
            max_wait_duration,
        }
    }

    pub fn from_config(config: BulkheadConfig) -> Self {
        let BulkheadConfig {
            max_concurrent_calls,
            max_wait_duration,
        } = config;
        Self::new(max_concurrent_calls, max_wait_duration)
    }

    #[cfg(feature = "tokio")]
    pub async fn limit<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let now = Instant::now();
        let until = (now + self.max_wait_duration).into();
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = tokio::time::timeout_at(until, permit_fut).await??;
        Ok(f.await)
    }

    #[cfg(not(feature = "tokio"))]
    #[cfg(feature = "async-std")]
    pub async fn limit<F, R>(&self, f: F) -> Result<R, BulkheadError>
    where
        F: Future<Output = R>,
    {
        let permit_fut = self.max_concurrent_calls.acquire();
        let _permit = async_std::future::timeout(self.max_wait_duration, permit_fut).await?;
        Ok(f.await)
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

#[cfg(test)]
mod tests {}
