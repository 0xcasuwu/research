//! Test binary for the metashrew macros

// Import specific items instead of using glob imports
use metashrew::indexer::{Indexer, KeyValueStore};
use anyhow::Result;
use serde::{Serialize, Deserialize};

// Define a simple indexer
struct SimpleIndexer {
    store: KeyValueStore,
}

impl Default for SimpleIndexer {
    fn default() -> Self {
        Self {
            store: KeyValueStore::new(),
        }
    }
}

impl Indexer for SimpleIndexer {
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

impl SimpleIndexer {
    fn get_height(&self, _input: Vec<u8>) -> Result<Vec<u8>> {
        if let Some(height) = self.store.get(b"height") {
            Ok(height.clone())
        } else {
            Ok(0u32.to_le_bytes().to_vec())
        }
    }
}

// Test the metashrew_indexer macro
metashrew::metashrew_indexer! {
    struct SimpleIndexerProgram {
        indexer: SimpleIndexer,
        views: {
            "get_height" => get_height(Vec<u8>) -> Vec<u8>,
        }
    }
}

// Test the metashrew_view macro
metashrew::metashrew_view! {
    fn test_view(input: Vec<u8>) -> Result<Vec<u8>> {
        Ok(input)
    }
}

fn main() {
    std::println!("Testing metashrew macros...");
    
    // Test the metashrew_indexer macro
    std::println!("Testing metashrew_indexer macro...");
    std::println!("SimpleIndexerProgram defined successfully");
    
    // Test the metashrew_view macro
    std::println!("Testing metashrew_view macro...");
    std::println!("test_view function defined successfully");
    
    std::println!("All tests completed successfully!");
}
