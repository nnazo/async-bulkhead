use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(all(not(any(feature = "tokio", feature = "smol")), feature = "async-std"))]
mod async_std;

#[cfg(all(not(any(feature = "tokio", feature = "async-std")), feature = "smol"))]
mod smol;

#[cfg(test)]
#[cfg(feature = "tokio")]
mod tests;

#[derive(Debug, Error)]
pub enum BulkheadError {
    #[cfg(feature = "tokio")]
    #[error("the bulkhead semaphore has been closed")]
    Acquire(#[from] tokio1::sync::AcquireError),
    #[error("the maximum number of concurrent calls is met")]
    Timeout(
        #[cfg(feature = "tokio")]
        #[from]
        tokio1::time::error::Elapsed,
        #[cfg(all(not(feature = "tokio"), feature = "async-std"))]
        #[from]
        async_std1::future::TimeoutError,
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
            max_concurrent_calls: Arc::new(tokio1::sync::Semaphore::new(self.max_concurrent_calls)),
            #[cfg(all(not(feature = "tokio"), any(feature = "async-std", feature = "smol")))]
            max_concurrent_calls: Arc::new(async_lock1::Semaphore::new(self.max_concurrent_calls)),
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
    max_concurrent_calls: Arc<tokio1::sync::Semaphore>,
    #[cfg(all(not(feature = "tokio"), any(feature = "async-std", feature = "smol")))]
    max_concurrent_calls: Arc<async_lock1::Semaphore>,
    max_wait_duration: Duration,
}

impl Bulkhead {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> BulkheadBuilder {
        BulkheadBuilder::default()
    }

    pub fn remaining_calls(&self) -> usize {
        self.max_concurrent_calls.available_permits()
    }
}

impl Default for Bulkhead {
    fn default() -> Self {
        let max_concurrent_calls = 25;
        Self {
            #[cfg(feature = "tokio")]
            max_concurrent_calls: Arc::new(tokio1::sync::Semaphore::new(max_concurrent_calls)),
            #[cfg(all(not(feature = "tokio"), any(feature = "async-std", feature = "smol")))]
            max_concurrent_calls: Arc::new(async_lock1::Semaphore::new(max_concurrent_calls)),
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
