//! # Metashrew Library
//!
//! A library for building Metashrew indexer programs with convenient macros and primitives.
//! This library simplifies the development of WASM modules for the Metashrew Bitcoin indexer framework.

extern crate log;

pub mod macros;
pub mod index_pointer;
pub mod host;
pub mod indexer;
pub mod stdio;
pub mod view;
pub mod utils;
pub mod wasm;
pub mod wasm_exports;
#[cfg(feature = "native")]
pub mod native;

/// Re-export key components for easier access
pub use host::*;
pub use indexer::*;
pub use view::*;
pub use wasm::{get, set, input, flush, initialize, reset, clear};
// wasm_exports is already available as a module

// Note: The macros (metashrew_indexer, metashrew_view, declare_indexer) are 
// automatically exported at the crate root by #[macro_export] and don't need to be re-exported
