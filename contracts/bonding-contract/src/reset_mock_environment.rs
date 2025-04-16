//! Reset mock environment module
//!
//! This module provides functionality to reset the mock environment for testing.

use crate::mock_runtime;
use crate::mock_storage;
use crate::mock_context;
use crate::Bond;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Once;
use std::collections::HashMap;
use std::sync::Mutex;

// Static flag to track if we're in a test environment
static IS_TEST_ENVIRONMENT: AtomicBool = AtomicBool::new(false);
static INIT: Once = Once::new();

// Counter to track test runs and ensure unique environments
pub static TEST_RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

// Mock bond registry for testing
lazy_static::lazy_static! {
    static ref MOCK_BOND_REGISTRY: Mutex<HashMap<u128, HashMap<u128, Bond>>> = Mutex::new(HashMap::new());
    static ref MOCK_POSITION_COUNTS: Mutex<HashMap<u128, u128>> = Mutex::new(HashMap::new());
}

/// Add a bond to the mock registry
pub fn add_bond(address: u128, bond_id: u128, bond: Bond) {
    let mut registry = MOCK_BOND_REGISTRY.lock().unwrap();
    let address_bonds = registry.entry(address).or_insert_with(HashMap::new);
    address_bonds.insert(bond_id, bond);
    
    // Update position count
    let mut counts = MOCK_POSITION_COUNTS.lock().unwrap();
    *counts.entry(address).or_insert(0) += 1;
}

/// Get a bond from the mock registry
pub fn get_bond(address: u128, bond_id: u128) -> Option<Bond> {
    let registry = MOCK_BOND_REGISTRY.lock().unwrap();
    if let Some(address_bonds) = registry.get(&address) {
        address_bonds.get(&bond_id).cloned()
    } else {
        None
    }
}

/// Update a bond in the mock registry
pub fn update_bond(address: u128, bond_id: u128, bond: Bond) {
    let mut registry = MOCK_BOND_REGISTRY.lock().unwrap();
    if let Some(address_bonds) = registry.get_mut(&address) {
        address_bonds.insert(bond_id, bond);
    }
}

/// Get the position count for an address
pub fn get_position_count(address: u128) -> u128 {
    let counts = MOCK_POSITION_COUNTS.lock().unwrap();
    *counts.get(&address).unwrap_or(&0)
}

/// Set the position count for an address
pub fn set_position_count(address: u128, count: u128) {
    let mut counts = MOCK_POSITION_COUNTS.lock().unwrap();
    counts.insert(address, count);
}

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
    
    // Reset the mock bond registry
    {
        let mut registry = MOCK_BOND_REGISTRY.lock().unwrap();
        registry.clear();
        
        let mut counts = MOCK_POSITION_COUNTS.lock().unwrap();
        counts.clear();
    }
    
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
