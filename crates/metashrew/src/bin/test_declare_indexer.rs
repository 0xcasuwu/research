//! Test binary for the declare_indexer macro

use metashrew::indexer::{Indexer, KeyValueStore};
use anyhow::Result;
use serde::{Serialize, Deserialize};

// Define Protocol Buffer messages for testing declare_indexer
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetHeightRequest {
    pub dummy: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetHeightResponse {
    pub height: u32,
}

// Define a simple indexer for testing declare_indexer
struct ProtoIndexer {
    store: KeyValueStore,
}

impl Default for ProtoIndexer {
    fn default() -> Self {
        Self {
            store: KeyValueStore::new(),
        }
    }
}

impl Indexer for ProtoIndexer {
    fn index_block(&mut self, height: u32, _block: &[u8]) -> Result<()> {
        // Store the height
        self.store.set(b"height".to_vec(), height.to_le_bytes().to_vec());
        Ok(())
    }

    fn flush(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        Ok(self.store.pairs())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ProtoIndexer {
    fn get_proto_height(&self, _request: GetHeightRequest) -> Result<GetHeightResponse> {
        let height = if let Some(height_bytes) = self.store.get(b"height") {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&height_bytes[0..4]);
            u32::from_le_bytes(bytes)
        } else {
            0
        };
        
        Ok(GetHeightResponse { height })
    }
}

// Test the declare_indexer macro
metashrew::declare_indexer! {
    struct ProtoIndexerProgram {
        indexer: ProtoIndexer,
        views: {
            "get_proto_height" => {
                fn get_proto_height(&self, request: GetHeightRequest) -> Result<GetHeightResponse> {
                    self.get_proto_height(request)
                }
            }
        }
    }
}

fn main() {
    std::println!("Testing declare_indexer macro...");
    std::println!("ProtoIndexerProgram defined successfully");
    std::println!("Test completed successfully!");
}
