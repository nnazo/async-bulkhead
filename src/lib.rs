#![feature(doc_cfg)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod bulkhead;
pub use bulkhead::*;
