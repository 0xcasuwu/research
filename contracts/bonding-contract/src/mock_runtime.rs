//! Mock runtime implementation for testing

use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::context::Context;
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::reset_mock_environment;

// Thread-local storage for mock context
thread_local! {
    pub static MOCK_CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
    pub static MOCK_STORAGE: RefCell<HashMap<Vec<u8>, Vec<u8>>> = RefCell::new(HashMap::new());
}

// Helper function to get the mock context
pub fn get_mock_context() -> Option<Context> {
    // Check if we're in a test environment
    if reset_mock_environment::is_test_environment() {
        MOCK_CONTEXT.with(|c| c.borrow().clone())
    } else {
        None
    }
}

// Helper function to get the context (used by lib.rs)
pub fn get_context() -> Option<Context> {
    get_mock_context()
}

// Helper function to set the mock context
pub fn set_mock_context(context: Context) {
    MOCK_CONTEXT.with(|c| {
        // First clear any existing context to ensure proper cleanup
        if c.borrow().is_some() {
            *c.borrow_mut() = None;
        }
        
        // Then set the new context
        *c.borrow_mut() = Some(context);
    });
}

// Helper function to clear the mock context
pub fn clear_mock_context() {
    MOCK_CONTEXT.with(|c| {
        // Explicitly drop the current context if it exists
        if c.borrow().is_some() {
            let mut context_ref = c.borrow_mut();
            // Take ownership of the context and drop it
            let _ = context_ref.take();
        }
    });
}

// Helper function to clear the mock storage
pub fn clear_mock_storage() {
    MOCK_STORAGE.with(|s| {
        // Create a new empty HashMap and swap it with the current one
        let mut old_storage = HashMap::new();
        std::mem::swap(&mut *s.borrow_mut(), &mut old_storage);
        
        // Explicitly drop the old storage
        drop(old_storage);
    });
}

// Helper function to get a value from mock storage
pub fn get_mock_storage(key: &[u8]) -> Vec<u8> {
    // Check if we're in a test environment
    if reset_mock_environment::is_test_environment() {
        MOCK_STORAGE.with(|s| {
            s.borrow().get(key).cloned().unwrap_or_default()
        })
    } else {
        Vec::new()
    }
}

// Helper function to set a value in mock storage
pub fn set_mock_storage(key: &[u8], value: Vec<u8>) {
    MOCK_STORAGE.with(|s| {
        // Remove any existing entry first to ensure proper cleanup
        let mut storage_ref = s.borrow_mut();
        if let Some(_) = storage_ref.remove(&key.to_vec()) {
            // Old value is dropped here
        }
        storage_ref.insert(key.to_vec(), value);
    });
}

// Mock implementation of AlkaneResponder
pub struct MockAlkaneResponder;

impl AlkaneResponder for MockAlkaneResponder {
    fn context(&self) -> Result<Context> {
        if let Some(context) = get_mock_context() {
            return Ok(context);
        }
        Err(anyhow!("No mock context set"))
    }

    fn execute(&self) -> Result<CallResponse> {
        Err(anyhow!("Not implemented"))
    }
}

// Mock runtime functions
pub fn create_mock_runtime() -> MockAlkaneResponder {
    // Ensure we're in a test environment
    if !reset_mock_environment::is_test_environment() {
        // Initialize the test environment
        reset_mock_environment::reset();
    }
    
    MockAlkaneResponder
}
