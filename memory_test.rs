use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;

// Mock storage implementation with safety checks
thread_local! {
    static MOCK_STORAGE: RefCell<HashMap<Vec<u8>, Arc<Vec<u8>>>> = RefCell::new(HashMap::new());
}

// Helper function to clear the mock storage
fn clear_mock_storage() {
    MOCK_STORAGE.with(|storage| {
        // First, create a new empty HashMap
        let new_storage = HashMap::new();
        
        // Then replace the old one with the new one
        // This ensures all Arc references are properly dropped
        *storage.borrow_mut() = new_storage;
    });
}

// Helper function to get a value from mock storage
fn get_mock_storage(key: &[u8]) -> Option<Arc<Vec<u8>>> {
    MOCK_STORAGE.with(|storage| {
        storage.borrow().get(key).cloned()
    })
}

// Helper function to set a value in mock storage
fn set_mock_storage(key: Vec<u8>, value: Vec<u8>) {
    MOCK_STORAGE.with(|storage| {
        storage.borrow_mut().insert(key, Arc::new(value));
    });
}

// Mock implementation of __load_storage with enhanced safety checks
fn __load_storage(key_ptr: *const u8, key_len: usize) -> *const u8 {
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
        let slice = std::slice::from_raw_parts(key_ptr, key_len);
        slice.to_vec()
    };
    
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

// Mock implementation of __request_storage with enhanced safety checks
fn __request_storage(key_ptr: *const u8, key_len: usize, value_ptr: *const u8, value_len: usize) {
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
        let slice = std::slice::from_raw_parts(key_ptr, key_len);
        slice.to_vec()
    };
    
    let value = unsafe {
        let slice = std::slice::from_raw_parts(value_ptr, value_len);
        slice.to_vec()
    };
    
    MOCK_STORAGE.with(|storage| {
        storage.borrow_mut().insert(key, Arc::new(value));
    });
}

// Test function to verify memory safety
fn test_memory_safety() {
    // Clear the storage first
    clear_mock_storage();
    
    // Create a key and value
    let key = vec![1, 2, 3, 4];
    let value = vec![5, 6, 7, 8];
    
    // Store the value
    set_mock_storage(key.clone(), value.clone());
    
    // Retrieve the value
    let retrieved = get_mock_storage(&key).unwrap();
    assert_eq!(*retrieved, value);
    
    // Test __load_storage
    let key_ptr = key.as_ptr();
    let key_len = key.len();
    let ptr = __load_storage(key_ptr, key_len);
    
    // Convert the pointer back to a slice
    let retrieved_slice = unsafe {
        std::slice::from_raw_parts(ptr, value.len())
    };
    
    // Verify the retrieved value
    assert_eq!(retrieved_slice, value.as_slice());
    
    // Test __request_storage
    let new_value = vec![9, 10, 11, 12];
    let value_ptr = new_value.as_ptr();
    let value_len = new_value.len();
    
    __request_storage(key_ptr, key_len, value_ptr, value_len);
    
    // Verify the updated value
    let updated = get_mock_storage(&key).unwrap();
    assert_eq!(*updated, new_value);
    
    // Test null pointer handling
    let null_ptr = std::ptr::null();
    let empty_ptr = __load_storage(null_ptr, 0);
    
    // This should not crash
    let empty_slice = unsafe {
        std::slice::from_raw_parts(empty_ptr, 0)
    };
    
    assert_eq!(empty_slice, &[]);
    
    // Test invalid length handling
    let invalid_len = isize::MAX as usize + 1;
    let empty_ptr2 = __load_storage(key_ptr, invalid_len);
    
    // This should not crash
    let empty_slice2 = unsafe {
        std::slice::from_raw_parts(empty_ptr2, 0)
    };
    
    assert_eq!(empty_slice2, &[]);
    
    // Test null pointer handling for __request_storage
    __request_storage(null_ptr, 0, value_ptr, value_len);
    __request_storage(key_ptr, key_len, null_ptr, 0);
    
    // Test invalid length handling for __request_storage
    __request_storage(key_ptr, invalid_len, value_ptr, value_len);
    __request_storage(key_ptr, key_len, value_ptr, invalid_len);
    
    // Clear the storage at the end
    clear_mock_storage();
    
    // Verify the storage is empty
    assert!(get_mock_storage(&key).is_none());
    
    println!("All memory safety tests passed!");
}

fn main() {
    test_memory_safety();
}
