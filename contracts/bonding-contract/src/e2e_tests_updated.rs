//! End-to-end tests for the Bonding Curve contract

use super::*;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use std::cell::RefCell;

// Import the thread_local mock context from tests.rs
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

/// This test demonstrates the end-to-end flow of a user buying and selling alkane
#[test]
fn test_e2e_buy_sell_alkane() {
    // Step 1: Initialize the contract
    let contract = BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"BondingToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"BOND";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let initial_diesel_reserve = 1_000_000;
    let k_factor = 1;
    let n_exponent = 1; // Linear bonding curve
    
    // Set up a default context for initialization
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Initialize the contract
    let init_result = contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve);
    assert!(init_result.is_ok(), "Contract initialization failed: {:?}", init_result.err());
    
    // Step 2: Verify the contract was initialized correctly
    assert_eq!(contract.name(), "BondingToken");
    assert_eq!(contract.symbol(), "BOND");
    assert_eq!(contract.diesel_reserve(), initial_diesel_reserve);
    assert_eq!(contract.alkane_supply(), initial_diesel_reserve);
    
    // Step 3: User 1 buys alkane with diesel
    let user1_diesel_amount = 50_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: user1_diesel_amount,
        }
    ]);
    
    // Set the context for User 1
    set_mock_context(context);
    
    // User 1 buys alkane with diesel
    let buy_result = contract.buy_alkane(user1_diesel_amount);
    assert!(buy_result.is_ok(), "Buy operation failed: {:?}", buy_result.err());
    
    // Get the response
    let buy_response = buy_result.unwrap();
    
    // Step 4: Verify User 1 received alkanes
    assert_eq!(buy_response.alkanes.0.len(), 1, "User should receive exactly one alkane transfer");
    assert_eq!(buy_response.alkanes.0[0].id, AlkaneId { block: 3, tx: 3 }, "User should receive contract alkanes");
    
    let user1_alkanes_received = buy_response.alkanes.0[0].value;
    assert!(user1_alkanes_received > 0, "User should receive a positive amount of alkanes");
    
    println!("User 1 bought alkane with {} diesel and received {} alkanes", user1_diesel_amount, user1_alkanes_received);
    
    // Step 5: User 2 buys alkane with diesel
    let user2_diesel_amount = 75_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: user2_diesel_amount,
        }
    ]);
    
    // Update the context for User 2 (different caller)
    let mut user2_context = context;
    user2_context.caller = AlkaneId { block: 1, tx: 2 };
    set_mock_context(user2_context);
    
    // User 2 buys alkane with diesel
    let buy_result = contract.buy_alkane(user2_diesel_amount);
    assert!(buy_result.is_ok(), "Buy operation failed: {:?}", buy_result.err());
    
    // Get the response
    let buy_response = buy_result.unwrap();
    
    // Step 6: Verify User 2 received alkanes
    assert_eq!(buy_response.alkanes.0.len(), 1, "User should receive exactly one alkane transfer");
    assert_eq!(buy_response.alkanes.0[0].id, AlkaneId { block: 3, tx: 3 }, "User should receive contract alkanes");
    
    let user2_alkanes_received = buy_response.alkanes.0[0].value;
    assert!(user2_alkanes_received > 0, "User should receive a positive amount of alkanes");
    
    println!("User 2 bought alkane with {} diesel and received {} alkanes", user2_diesel_amount, user2_alkanes_received);
    
    // Step 7: Verify the contract state after both users bought alkane
    let new_diesel_reserve = contract.diesel_reserve();
    let new_alkane_supply = contract.alkane_supply();
    
    assert_eq!(new_diesel_reserve, initial_diesel_reserve + user1_diesel_amount + user2_diesel_amount, 
               "Diesel reserve should be updated correctly");
    assert_eq!(new_alkane_supply, initial_diesel_reserve + user1_alkanes_received + user2_alkanes_received, 
               "Alkane supply should be updated correctly");
    
    // Step 8: User 1 sells alkane for diesel
    let user1_alkane_sell_amount = user1_alkanes_received / 2;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: user1_alkane_sell_amount,
        }
    ]);
    
    // Set the context back to User 1
    let mut user1_context = context;
    user1_context.caller = AlkaneId { block: 1, tx: 1 };
    set_mock_context(user1_context);
    
    // User 1 sells alkane for diesel
    let sell_result = contract.sell_alkane(user1_alkane_sell_amount);
    assert!(sell_result.is_ok(), "Sell operation failed: {:?}", sell_result.err());
    
    // Get the response
    let sell_response = sell_result.unwrap();
    
    // Step 9: Verify User 1 received diesel in return
    assert_eq!(sell_response.alkanes.0.len(), 1, "User should receive exactly one alkane transfer");
    assert_eq!(sell_response.alkanes.0[0].id, AlkaneId { block: 2, tx: 0 }, "User should receive diesel");
    
    let user1_diesel_received = sell_response.alkanes.0[0].value;
    assert!(user1_diesel_received > 0, "User should receive a positive amount of diesel");
    
    println!("User 1 sold {} alkanes and received {} diesel", user1_alkane_sell_amount, user1_diesel_received);
    
    // Step 10: Verify the contract state after User 1 sold alkane
    let final_diesel_reserve = contract.diesel_reserve();
    let final_alkane_supply = contract.alkane_supply();
    
    assert_eq!(final_diesel_reserve, new_diesel_reserve - user1_diesel_received, 
               "Diesel reserve should be updated correctly after sell");
    assert_eq!(final_alkane_supply, new_alkane_supply - user1_alkane_sell_amount, 
               "Alkane supply should be updated correctly after sell");
    
    // Step 11: Get the current price
    let price_response = contract.current_price().unwrap();
    let current_price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    println!("Current price: {} diesel per alkane", current_price);
    
    // Step 12: Verify the price is reasonable
    assert!(current_price > 0, "Price should be positive");
    
    // Step 13: Verify the price has increased from the initial price
    let initial_price = k_factor * initial_diesel_reserve;
    assert!(current_price > initial_price, 
            "Price should increase as more diesel is added to the reserve (initial: {}, current: {})", 
            initial_price, current_price);
    
    println!("E2E test completed successfully!");
    println!("Initial state: {} diesel reserve, {} alkane supply", initial_diesel_reserve, initial_diesel_reserve);
    println!("Final state: {} diesel reserve, {} alkane supply", final_diesel_reserve, final_alkane_supply);
    println!("Price change: {} -> {} diesel per alkane", initial_price, current_price);
}

/// This test demonstrates the full lifecycle of the bonding curve contract:
/// 1. Contract initialization
/// 2. Multiple users buying alkane
/// 3. Price discovery and slippage effects
/// 4. Multiple users selling alkane
/// 5. Final state verification
#[test]
fn test_bonding_contract_lifecycle() {
    // Step 1: Initialize the contract
    let contract = BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"LifecycleToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"LIFE";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let initial_diesel_reserve = 1_000_000;
    let k_factor = 1;
    let n_exponent = 1; // Linear bonding curve
    
    // Set up a default context for initialization
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Initialize the contract
    contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve).unwrap();
    
    // Step 2: Define multiple users
    let users = vec![
        AlkaneId { block: 1, tx: 1 }, // Creator
        AlkaneId { block: 1, tx: 2 }, // User 2
        AlkaneId { block: 1, tx: 3 }, // User 3
        AlkaneId { block: 1, tx: 4 }, // User 4
        AlkaneId { block: 1, tx: 5 }, // User 5
    ];
    
    // Step 3: Users buy alkane with diesel in sequence
    let diesel_amounts = vec![10_000, 20_000, 30_000, 40_000, 50_000];
    let mut user_alkanes = vec![0; users.len()];
    
    for (i, (user, diesel_amount)) in users.iter().zip(diesel_amounts.iter()).enumerate() {
        // Create context for this user
        let context = create_context_with_alkanes(vec![
            AlkaneTransfer {
                id: AlkaneId { block: 2, tx: 0 }, // Diesel
                value: *diesel_amount,
            }
        ]);
        
        // Update the context for this user
        let mut user_context = context;
        user_context.caller = user.clone();
        set_mock_context(user_context);
        
        // User buys alkane with diesel
        let buy_response = contract.buy_alkane(*diesel_amount).unwrap();
        
        // Record the alkanes received
        user_alkanes[i] = buy_response.alkanes.0[0].value;
        
        println!("User {} bought alkane with {} diesel and received {} alkanes", 
                 i + 1, diesel_amount, user_alkanes[i]);
        
        // Check the price after each buy
        let price_response = contract.current_price().unwrap();
        let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
        println!("Price after User {}: {} diesel per alkane", i + 1, price);
    }
    
    // Step 4: Verify the total state
    let total_diesel_provided: u128 = diesel_amounts.iter().sum();
    let total_alkanes_received: u128 = user_alkanes.iter().sum();
    
    assert_eq!(contract.diesel_reserve(), initial_diesel_reserve + total_diesel_provided,
               "Diesel reserve should match the total diesel provided");
    assert_eq!(contract.alkane_supply(), initial_diesel_reserve + total_alkanes_received,
               "Alkane supply should match the initial plus total alkanes received");
    
    // Step 5: Users sell alkane for diesel in reverse order
    let mut total_diesel_received = 0;
    
    for (i, (user, alkanes)) in users.iter().zip(user_alkanes.iter()).enumerate().rev() {
        // Only sell if the user has alkanes
        if *alkanes > 0 {
            // Create context for this user
            let context = create_context_with_alkanes(vec![
                AlkaneTransfer {
                    id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
                    value: *alkanes,
                }
            ]);
            
            // Update the context for this user
            let mut user_context = context;
            user_context.caller = user.clone();
            set_mock_context(user_context);
            
            // User sells alkane for diesel
            let sell_response = contract.sell_alkane(*alkanes).unwrap();
            
            // Record the diesel received
            let diesel_received = sell_response.alkanes.0[0].value;
            total_diesel_received += diesel_received;
            
            println!("User {} sold {} alkanes and received {} diesel", 
                     i + 1, alkanes, diesel_received);
            
            // Check the price after each sell
            let price_response = contract.current_price().unwrap();
            let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
            println!("Price after User {} sell: {} diesel per alkane", i + 1, price);
        }
    }
    
    // Step 6: Verify the final state
    let final_diesel_reserve = contract.diesel_reserve();
    let final_alkane_supply = contract.alkane_supply();
    
    println!("Final state: {} diesel reserve, {} alkane supply", final_diesel_reserve, final_alkane_supply);
    println!("Total diesel provided: {}", total_diesel_provided);
    println!("Total diesel received: {}", total_diesel_received);
    
    // The final diesel reserve should be less than the initial reserve plus the total provided minus the total received
    // due to slippage and fees
    assert!(final_diesel_reserve < initial_diesel_reserve + total_diesel_provided - total_diesel_received,
            "Final diesel reserve should reflect slippage and fees");
    
    // The final alkane supply should be equal to the initial supply
    // (all received alkanes were sold back)
    assert_eq!(final_alkane_supply, initial_diesel_reserve,
               "Final alkane supply should equal initial supply after all sells");
    
    println!("Lifecycle test completed successfully!");
}
