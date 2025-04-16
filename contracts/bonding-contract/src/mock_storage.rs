//! Mock storage module for testing

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use crate::reset_mock_environment;

// Mock storage implementation with safety checks
thread_local! {
    static MOCK_STORAGE: RefCell<HashMap<Vec<u8>, Arc<Vec<u8>>>> = RefCell::new(HashMap::new());
}

// Helper function to clear the mock storage
pub fn clear_mock_storage() {
    MOCK_STORAGE.with(|storage| {
        // First, explicitly drop all Arc references
        let mut old_storage = HashMap::new();
        std::mem::swap(&mut *storage.borrow_mut(), &mut old_storage);
        
        // Explicitly drop each Arc reference
        for (_, value) in old_storage.into_iter() {
            drop(value);
        }
        
        // Create a new empty HashMap
        *storage.borrow_mut() = HashMap::new();
    });
}

// Helper function to get a value from mock storage
pub fn get_mock_storage(key: &[u8]) -> Option<Arc<Vec<u8>>> {
    // Check if we're in a test environment
    if reset_mock_environment::is_test_environment() {
        // Create a prefixed key with the test run counter to ensure isolation between tests
        let mut prefixed_key = Vec::with_capacity(8 + key.len());
        
        // Get the current test run counter
        let counter = unsafe {
            std::ptr::read_volatile(&crate::reset_mock_environment::TEST_RUN_COUNTER as *const _ as *const u64)
        };
        
        // Add the counter as a prefix to the key
        prefixed_key.extend_from_slice(&counter.to_le_bytes());
        prefixed_key.extend_from_slice(key);
        
        MOCK_STORAGE.with(|storage| {
            storage.borrow().get(&prefixed_key).cloned()
        })
    } else {
        None
    }
}

// Helper function to set a value in mock storage
pub fn set_mock_storage(key: Vec<u8>, value: Vec<u8>) {
    // Create a prefixed key with the test run counter to ensure isolation between tests
    let mut prefixed_key = Vec::with_capacity(8 + key.len());
    
    // Get the current test run counter
    let counter = unsafe {
        std::ptr::read_volatile(&crate::reset_mock_environment::TEST_RUN_COUNTER as *const _ as *const u64)
    };
    
    // Add the counter as a prefix to the key
    prefixed_key.extend_from_slice(&counter.to_le_bytes());
    prefixed_key.extend_from_slice(&key);
    
    MOCK_STORAGE.with(|storage| {
        storage.borrow_mut().insert(prefixed_key, Arc::new(value));
    });
}

// Mock implementation of __load_storage with enhanced safety checks
#[no_mangle]
pub extern "C" fn __load_storage(key_ptr: *const u8, key_len: usize) -> *const u8 {
    // Safety check for null pointer
    if key_ptr.is_null() {
        // Return an empty vector for null pointers
        let empty = Arc::new(Vec::new());
        let ptr = empty.as_ptr();
        // Store the Arc in the storage to prevent it from being dropped
        MOCK_STORAGE.with(|storage| {
            storage.borrow_mut().insert(vec![0], empty);
        });
        return ptr;
    }
    
    // Safety check for key_len
    if key_len > isize::MAX as usize {
        // Return an empty vector for invalid lengths
        let empty = Arc::new(Vec::new());
        let ptr = empty.as_ptr();
        // Store the Arc in the storage to prevent it from being dropped
        MOCK_STORAGE.with(|storage| {
            storage.borrow_mut().insert(vec![0], empty);
        });
        return ptr;
    }
    
    // Safely create a Vec<u8> from the raw pointer
    let key = unsafe {
        if key_ptr.is_null() || key_len == 0 {
            vec![0]
        } else {
            let slice = std::slice::from_raw_parts(key_ptr, key_len);
            slice.to_vec()
        }
    };
    
    // Create a prefixed key with the test run counter to ensure isolation between tests
    let mut prefixed_key = Vec::with_capacity(8 + key.len());
    
    // Get the current test run counter
    let counter = unsafe {
        std::ptr::read_volatile(&crate::reset_mock_environment::TEST_RUN_COUNTER as *const _ as *const u64)
    };
    
    // Add the counter as a prefix to the key
    prefixed_key.extend_from_slice(&counter.to_le_bytes());
    prefixed_key.extend_from_slice(&key);
    
    let result = MOCK_STORAGE.with(|storage| {
        storage.borrow().get(&prefixed_key).cloned()
    });
    
    match result {
        Some(value) => {
            // Return a pointer to the data
            value.as_ptr()
        }
        None => {
            // Return an empty vector but store it in the storage to prevent it from being dropped
            let empty = Arc::new(Vec::new());
            let ptr = empty.as_ptr();
            // Store the Arc in the storage to prevent it from being dropped
            MOCK_STORAGE.with(|storage| {
                storage.borrow_mut().insert(prefixed_key, empty.clone());
            });
            ptr
        }
    }
}

// Mock implementation of __request_storage with enhanced safety checks
#[no_mangle]
pub extern "C" fn __request_storage(key_ptr: *const u8, key_len: usize, value_ptr: *const u8, value_len: usize) {
    // Safety check for null pointers
    if key_ptr.is_null() || value_ptr.is_null() {
        return;
    }
    
    // Safety check for lengths
    if key_len > isize::MAX as usize || value_len > isize::MAX as usize {
        return;
    }
    
    // Safely create Vec<u8>s from the raw pointers
    let key = unsafe {
        if key_ptr.is_null() || key_len == 0 {
            vec![0]
        } else {
            let slice = std::slice::from_raw_parts(key_ptr, key_len);
            slice.to_vec()
        }
    };
    
    // Create a prefixed key with the test run counter to ensure isolation between tests
    let mut prefixed_key = Vec::with_capacity(8 + key.len());
    
    // Get the current test run counter
    let counter = unsafe {
        std::ptr::read_volatile(&crate::reset_mock_environment::TEST_RUN_COUNTER as *const _ as *const u64)
    };
    
    // Add the counter as a prefix to the key
    prefixed_key.extend_from_slice(&counter.to_le_bytes());
    prefixed_key.extend_from_slice(&key);
    
    let value = unsafe {
        if value_ptr.is_null() || value_len == 0 {
            vec![0]
        } else {
            let slice = std::slice::from_raw_parts(value_ptr, value_len);
            slice.to_vec()
        }
    };
    
    // Remove any existing entry first to ensure proper cleanup
    MOCK_STORAGE.with(|storage| {
        let mut storage_ref = storage.borrow_mut();
        if let Some(_) = storage_ref.remove(&prefixed_key) {
            // Old value is dropped here
        }
        storage_ref.insert(prefixed_key, Arc::new(value));
    });
}
