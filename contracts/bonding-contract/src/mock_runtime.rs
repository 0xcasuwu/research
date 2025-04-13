//! Mock runtime functions for testing

use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;

// Thread-local storage for mock storage
thread_local! {
    static MOCK_STORAGE: RefCell<HashMap<Vec<u8>, Arc<Vec<u8>>>> = RefCell::new(HashMap::new());
}

// Mock implementation of __load_storage
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
    
    let key = unsafe { std::slice::from_raw_parts(key_ptr, key_len).to_vec() };
    
    let result = MOCK_STORAGE.with(|storage| {
        storage.borrow().get(&key).cloned()
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
                storage.borrow_mut().insert(key, empty);
            });
            ptr
        }
    }
}

// Mock implementation of __request_storage
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
    
    let key = unsafe { std::slice::from_raw_parts(key_ptr, key_len).to_vec() };
    let value = unsafe { std::slice::from_raw_parts(value_ptr, value_len).to_vec() };
    
    MOCK_STORAGE.with(|storage| {
        storage.borrow_mut().insert(key, Arc::new(value));
    });
}

// Helper function to clear the mock storage (useful between tests)
pub fn clear_mock_storage() {
    MOCK_STORAGE.with(|storage| {
        storage.borrow_mut().clear();
    });
}

// Helper function to get a value from the mock storage
pub fn get_mock_storage(key: &[u8]) -> Option<Arc<Vec<u8>>> {
    MOCK_STORAGE.with(|storage| {
        storage.borrow().get(key).cloned()
    })
}

// Helper function to set a value in the mock storage
pub fn set_mock_storage(key: &[u8], value: Vec<u8>) {
    MOCK_STORAGE.with(|storage| {
        storage.borrow_mut().insert(key.to_vec(), Arc::new(value));
    });
}
