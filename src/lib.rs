//! An async semaphore-based bulkhead implementation.
//!
//! The [`Bulkhead`] struct provides a `limit` method for wrapping
//! futures with the bulkhead client-side resiliency pattern.
//!
//! ### Example
//! ```rust
//! use async_bulkhead::Bulkhead;
//! use async_bulkhead::BulkheadError;
//! use std::time::Duration;
//!
//! let bulkhead = Bulkhead::builder()
//!     .max_concurrent_calls(10)
//!     .max_wait_duration(Duration::from_millis(100))
//!     .build()
//!     .expect("max concurrent calls not > 0");
//!
//! async {
//!     let value = bulkhead.limit(async { 10 }).await?;
//!     println!("{value}");
//!     Ok::<_, BulkheadError>(())
//! };
//! ```
//!
//! ### Features
//! Note that the runtime features of this crate are mutually exclusive.
//! You should only use one of the following:
//! 1. `rt-tokio` (default)
//! 1. `rt-async-std`
//! 1. `rt-smol`

mod bulkhead;
pub use bulkhead::*;
