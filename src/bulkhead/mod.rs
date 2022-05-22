use async_lock::Semaphore;
use cfg_if::cfg_if;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

cfg_if!(
    if #[cfg(all(not(any(feature = "rt-async-std", feature = "rt-smol")), feature = "rt-tokio"))] {
        mod tokio;
    } else if #[cfg(all(not(any(feature = "rt-tokio", feature = "rt-smol")), feature = "rt-async-std"))] {
        mod async_std;
    } else if #[cfg(all(not(any(feature = "rt-tokio", feature = "rt-async-std")), feature = "rt-smol"))] {
        mod smol;
    } else {
        compile_error!("you must enable one feature between `rt-tokio`, `rt-async-std` and `rt-smol`");
    }
);

#[cfg(all(test, feature = "rt-tokio"))]
mod tests;

/// The error type for operations with [`Bulkhead`].
#[derive(Debug, Error)]
pub enum BulkheadError {
    /// The error returned when the bulkhead semaphore permit could not be acquired before the
    /// specified maximum wait duration.
    #[error("the maximum number of concurrent calls is met")]
    Timeout,
    /// The error returned when a non-positive maximum number of concurrent calls is specified
    #[error("max concurrent calls must be at least 1")]
    InvalidConcurrentCalls,
}

/// A builder type for a [`Bulkhead`]
#[derive(Debug, Copy, Clone)]
pub struct BulkheadBuilder {
    max_concurrent_calls: usize,
    max_wait_duration: Duration,
}

impl BulkheadBuilder {
    /// Specifies the maximum number of concurrent calls the bulkhead will allow.
    ///
    /// Defaults to 25.
    pub fn max_concurrent_calls(mut self, max_concurrent_calls: usize) -> Self {
        self.max_concurrent_calls = max_concurrent_calls;
        self
    }

    /// Specifies the maximum wait duration for the bulkhead's semaphore guard to be acquired.
    ///
    /// Defaults to `Duration::from_millis(1)`.
    pub fn max_wait_duration(mut self, max_wait_duration: Duration) -> Self {
        self.max_wait_duration = max_wait_duration;
        self
    }

    /// Builds the [`Bulkhead`]. This returns an `Err(BulkheadError::InvalidConcurrentCalls)`
    /// value if the number of concurrent calls is not positive.
    pub fn build(self) -> Result<Bulkhead, BulkheadError> {
        if self.max_concurrent_calls > 0 {
            Ok(Bulkhead {
                max_concurrent_calls: Arc::new(Semaphore::new(self.max_concurrent_calls)),
                max_wait_duration: self.max_wait_duration,
            })
        } else {
            Err(BulkheadError::InvalidConcurrentCalls)
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

/// A semaphore-based bulkhead for limiting the number of concurrent
/// calls to a resource.
///
/// This type can be safely cloned and sent across threads while
/// maintaining the correct number of allowed concurrent calls.
#[derive(Debug, Clone)]
pub struct Bulkhead {
    max_concurrent_calls: Arc<Semaphore>,
    max_wait_duration: Duration,
}

impl Bulkhead {
    /// Creates a new bulkhead with the default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`BulkheadBuilder`] containing the default configuration
    pub fn builder() -> BulkheadBuilder {
        BulkheadBuilder::default()
    }
}

impl Default for Bulkhead {
    fn default() -> Self {
        let BulkheadBuilder {
            max_concurrent_calls,
            max_wait_duration,
        } = BulkheadBuilder::default();
        Self {
            max_concurrent_calls: Arc::new(Semaphore::new(max_concurrent_calls)),
            max_wait_duration,
        }
    }
}

/// A structure for tracking multiple bulkheads for different resources.
///
/// This type can be safely cloned and sent across threads while
/// maintaining the correct number of allowed concurrent calls in each
/// resource's corresponding bulkhead.
#[derive(Debug, Clone)]
pub struct BulkheadRegistry(HashMap<String, Bulkhead>);

impl BulkheadRegistry {
    /// Creates an empty registry
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Adds a new bulkhead for the specified resource to the registry
    pub fn register(&mut self, resource: String, bulkhead: Bulkhead) -> &mut Self {
        self.0.insert(resource, bulkhead);
        self
    }

    /// Retrieves the requested resource bulkhead from the registry
    pub fn get(&self, resource: &str) -> Option<&Bulkhead> {
        self.0.get(resource)
    }
}

impl Default for BulkheadRegistry {
    fn default() -> Self {
        Self::new()
    }
}
