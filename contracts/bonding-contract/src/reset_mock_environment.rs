//! Reset mock environment module
//!
//! This module provides functionality to reset the mock environment for testing.

use crate::mock_runtime;
use crate::mock_storage;
use crate::mock_context;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Once;

// Static flag to track if we're in a test environment
static IS_TEST_ENVIRONMENT: AtomicBool = AtomicBool::new(false);
static INIT: Once = Once::new();

// Counter to track test runs and ensure unique environments
pub static TEST_RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Reset the mock environment
///
/// This function resets the mock runtime, storage, and context for testing.
/// It ensures proper cleanup of resources to prevent memory corruption between tests.
pub fn reset() {
    // Use a mutex to ensure only one test is resetting the environment at a time
    static RESET_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _lock = RESET_MUTEX.lock().unwrap();
    
    // Increment the test run counter to ensure a fresh environment
    TEST_RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Initialize test environment flag if not already done
    INIT.call_once(|| {
        IS_TEST_ENVIRONMENT.store(true, Ordering::SeqCst);
    });

    // First, clear all mock contexts to ensure any references are dropped
    mock_context::clear_mock_context();
    mock_runtime::clear_mock_context();
    
    // Then clear all storage to ensure any references are dropped
    mock_storage::clear_mock_storage();
    mock_runtime::clear_mock_storage();
    
    // Force a garbage collection cycle if possible
    #[cfg(feature = "gc")]
    unsafe {
        std::alloc::System.gc();
    }
    
    // Add a small delay to ensure all resources are properly released
    std::thread::sleep(std::time::Duration::from_millis(50));
}

/// Check if we're in a test environment
pub fn is_test_environment() -> bool {
    IS_TEST_ENVIRONMENT.load(Ordering::SeqCst)
}
