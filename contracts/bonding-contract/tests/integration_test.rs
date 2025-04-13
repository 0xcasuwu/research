//! Integration tests for the bonding contract

use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::context::Context;
use alkanes_support::response::CallResponse;
// Import BondingCurve directly from the crate root
use bonding_contract::BondingCurve;
// Import BondingContract trait to access current_price method
use bonding_contract::BondingContract;
use std::cell::RefCell;
use std::sync::Arc;
use std::collections::HashMap;

// Mock storage implementation with safety checks
thread_local! {
    static MOCK_STORAGE: RefCell<HashMap<Vec<u8>, Arc<Vec<u8>>>> = RefCell::new(HashMap::new());
}

// Mock implementation of __load_storage with safety checks
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

// Mock implementation of __request_storage with safety checks
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
fn clear_mock_storage() {
    MOCK_STORAGE.with(|storage| {
        storage.borrow_mut().clear();
    });
}

// Mock context implementation
thread_local! {
    static MOCK_CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

// Helper function to set the mock context
fn set_mock_context(context: Context) {
    MOCK_CONTEXT.with(|c| {
        *c.borrow_mut() = Some(context);
    });
}

// Helper function to create a context with incoming alkanes
fn create_context_with_alkanes(alkanes: Vec<AlkaneTransfer>) -> Context {
    Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: AlkaneTransferParcel(alkanes),
        vout: 0,
        inputs: vec![],
    }
}

// Simple test to verify the bonding contract functionality
#[test]
fn test_bonding_contract_e2e() {
    // Clear the mock storage
    clear_mock_storage();
    
    // Create a new bonding contract
    let mut contract = bonding_contract::BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"BondingToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"BND";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let initial_supply = 1_000_000;
    let initial_reserve = 1_000_000;
    let k_factor = 1;
    let n_exponent = 1;
    
    // Set up a default context for initialization
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Initialize the contract using the public init_contract method
    let init_result = contract.init_contract(name, symbol, k_factor, n_exponent, initial_reserve);
    assert!(init_result.is_ok(), "Contract initialization failed: {:?}", init_result.err());
    
    // Test buying alkane with diesel
    let user1_diesel_amount = 50_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: user1_diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    // Use buy_alkane instead of provide
    let buy_result = contract.buy_alkane(user1_diesel_amount);
    assert!(buy_result.is_ok(), "Buy alkane operation failed: {:?}", buy_result.err());
    
    let buy_response = buy_result.unwrap();
    assert_eq!(buy_response.alkanes.0.len(), 1, "User should receive exactly one alkane transfer");
    assert_eq!(buy_response.alkanes.0[0].id, AlkaneId { block: 3, tx: 3 }, "User should receive contract alkanes");
    
    let user1_alkanes_received = buy_response.alkanes.0[0].value;
    assert!(user1_alkanes_received > 0, "User should receive a positive amount of alkanes");
    
    println!("User 1 provided {} diesel and received {} alkanes", user1_diesel_amount, user1_alkanes_received);
    
    // Test selling alkanes for diesel
    let user1_sell_amount = user1_alkanes_received / 2;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: user1_sell_amount,
        }
    ]);
    set_mock_context(context);
    
    // Use sell_alkane instead of redeem
    let sell_result = contract.sell_alkane(user1_sell_amount);
    assert!(sell_result.is_ok(), "Sell alkane operation failed: {:?}", sell_result.err());
    
    let sell_response = sell_result.unwrap();
    assert_eq!(sell_response.alkanes.0.len(), 1, "User should receive exactly one alkane transfer");
    assert_eq!(sell_response.alkanes.0[0].id, AlkaneId { block: 2, tx: 0 }, "User should receive diesel");
    
    let user1_diesel_received = sell_response.alkanes.0[0].value;
    assert!(user1_diesel_received > 0, "User should receive a positive amount of diesel");
    assert!(user1_diesel_received < user1_diesel_amount, "User should receive less diesel than provided due to slippage");
    
    println!("User 1 sold {} alkanes and received {} diesel", user1_sell_amount, user1_diesel_received);
    
    // Test getting the current price using the BondingContract trait
    let current_price_result = BondingContract::current_price(&contract);
    assert!(current_price_result.is_ok(), "Failed to get current price: {:?}", current_price_result.err());
    
    let current_price_response = current_price_result.unwrap();
    let current_price = u128::from_le_bytes(current_price_response.data.try_into().unwrap());
    println!("Current price: {} diesel per alkane", current_price);
    
    // Verify the price is reasonable
    assert!(current_price > 0, "Price should be positive");
    
    // Verify the price has increased from the initial price
    let initial_price = initial_reserve;  // For n=1, price = k * reserve, and k=1
    assert!(current_price > initial_price, 
            "Price should increase as more diesel is provided (initial: {}, current: {})", 
            initial_price, current_price);
    
    println!("E2E test completed successfully!");
}
