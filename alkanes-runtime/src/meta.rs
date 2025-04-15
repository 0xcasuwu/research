pub mod auth;
#[cfg(feature = "panic-hook")]
pub mod compat;
pub mod imports;
pub mod runtime;
pub mod stdio;
pub mod storage;
pub mod token;
pub use crate::stdio::stdout;

use metashrew_support::{compat::{to_arraybuffer_layout, to_passback_ptr}};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub name: String,
String}

#[derive(Serialize, Deserialize)] 
pub struct ApiMetadata {
    pub functions: Vec<FunctionMetadata>
}

