//! Test binary for the bonding contract

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

// Thread-local storage for mock context
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

fn main() {
    println!("Testing bonding contract...");
    
    // Reset the mock environment
    bonding_contract::reset_mock_environment::reset();
    
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
    let diesel_amount = 50_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    // Use buy_alkane instead of provide
    match contract.buy_alkane(diesel_amount) {
        Ok(response) => {
            println!("Bought alkane with {} diesel", diesel_amount);
            println!("Received {} alkanes", response.alkanes.0[0].value);
            
            // Save the alkane amount for the next test
            let alkane_amount = response.alkanes.0[0].value;
            
            // Test selling alkanes for diesel
            let context = create_context_with_alkanes(vec![
                AlkaneTransfer {
                    id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
                    value: alkane_amount / 2, // Sell half of the alkanes
                }
            ]);
            set_mock_context(context);
            
            // Use sell_alkane instead of redeem
            match contract.sell_alkane(alkane_amount / 2) {
                Ok(response) => {
                    println!("Sold {} alkanes", alkane_amount / 2);
                    println!("Received {} diesel", response.alkanes.0[0].value);
                    
                    // Test getting the current price using the BondingContract trait
                    match BondingContract::current_price(&contract) {
                        Ok(price_response) => {
                            // Convert the response data to u128
                            let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
                            println!("Current price: {} diesel per alkane", price);
                        },
                        Err(e) => println!("Failed to get current price: {}", e),
                    }
                },
                Err(e) => println!("Failed to sell alkanes: {}", e),
            }
        },
        Err(e) => println!("Failed to buy alkane: {}", e),
    }
    
    println!("Testing completed!");
}
