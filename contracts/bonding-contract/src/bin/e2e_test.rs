//! End-to-end test for the bonding contract

use bonding_contract::BondingContractAlkane;
// Import BondingCurve directly from the crate root
use bonding_contract::BondingCurve;
// Import BondingContract trait to access current_price method
use bonding_contract::BondingContract;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::context::Context;
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

// Helper function to clear the mock storage
fn clear_mock_storage() {
    MOCK_STORAGE.with(|storage| {
        storage.borrow_mut().clear();
    });
}

// Import the mock context module
use bonding_contract::mock_context;

// Helper function to set the mock context
fn set_mock_context(context: Context) {
    mock_context::set_mock_context(context);
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

fn main() {
    println!("Running end-to-end test for the bonding contract...");
    
    // Clear the mock storage
    clear_mock_storage();
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"BondingToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"BND";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
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
    match contract.init_contract(name, symbol, k_factor, n_exponent, initial_reserve) {
        Ok(_) => println!("Contract initialized successfully"),
        Err(e) => {
            println!("Failed to initialize contract: {}", e);
            return;
        }
    }
    
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
    match contract.buy_alkane(user1_diesel_amount) {
        Ok(response) => {
            let user1_alkanes_received = response.alkanes.0[0].value;
            println!("User 1 bought {} alkanes with {} diesel", user1_alkanes_received, user1_diesel_amount);
            
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
            match contract.sell_alkane(user1_sell_amount) {
                Ok(response) => {
                    let user1_diesel_received = response.alkanes.0[0].value;
                    println!("User 1 sold {} alkanes and received {} diesel", user1_sell_amount, user1_diesel_received);
                    
                    // Test getting the current price using the BondingContract trait
                    match BondingContract::current_price(&contract) {
                        Ok(price_response) => {
                            // Convert the response data to u128
                            let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
                            println!("Current price: {} diesel per alkane", price);
                            println!("End-to-end test completed successfully!");
                        },
                        Err(e) => println!("Failed to get current price: {}", e),
                    }
                },
                Err(e) => println!("Failed to sell alkanes: {}", e),
            }
        },
        Err(e) => println!("Failed to buy alkanes: {}", e),
    }
}
