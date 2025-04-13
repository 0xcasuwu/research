//! End-to-end tests for the AMM contract

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

/// This test demonstrates the end-to-end flow of a user swapping diesel for alkane
#[test]
fn test_e2e_swap_diesel_for_alkane() {
    // Step 1: Initialize the contract
    let contract = BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"AMMToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"AMM";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let initial_reserve_diesel = 1_000_000;
    let initial_reserve_alkane = 1_000_000;
    
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
    let init_result = contract.init_contract(name, symbol, initial_reserve_diesel, initial_reserve_alkane);
    assert!(init_result.is_ok(), "Contract initialization failed: {:?}", init_result.err());
    
    // Step 2: Verify the contract was initialized correctly
    assert_eq!(contract.name(), "AMMToken");
    assert_eq!(contract.symbol(), "AMM");
    assert_eq!(contract.reserve_diesel(), initial_reserve_diesel);
    assert_eq!(contract.reserve_alkane(), initial_reserve_alkane);
    
    // Step 3: User 1 swaps diesel for alkane
    let user1_diesel_amount = 50_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: user1_diesel_amount,
        }
    ]);
    
    // Set the context for User 1
    set_mock_context(context);
    
    // User 1 swaps diesel for alkane
    let swap_result = contract.swap_diesel_for_alkane(user1_diesel_amount);
    assert!(swap_result.is_ok(), "Swap operation failed: {:?}", swap_result.err());
    
    // Get the response
    let swap_response = swap_result.unwrap();
    
    // Step 4: Verify User 1 received alkanes
    assert_eq!(swap_response.alkanes.0.len(), 1, "User should receive exactly one alkane transfer");
    assert_eq!(swap_response.alkanes.0[0].id, AlkaneId { block: 3, tx: 3 }, "User should receive contract alkanes");
    
    let user1_alkanes_received = swap_response.alkanes.0[0].value;
    assert!(user1_alkanes_received > 0, "User should receive a positive amount of alkanes");
    
    println!("User 1 swapped {} diesel and received {} alkanes", user1_diesel_amount, user1_alkanes_received);
    
    // Step 5: User 2 swaps diesel for alkane
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
    
    // User 2 swaps diesel for alkane
    let swap_result = contract.swap_diesel_for_alkane(user2_diesel_amount);
    assert!(swap_result.is_ok(), "Swap operation failed: {:?}", swap_result.err());
    
    // Get the response
    let swap_response = swap_result.unwrap();
    
    // Step 6: Verify User 2 received alkanes
    assert_eq!(swap_response.alkanes.0.len(), 1, "User should receive exactly one alkane transfer");
    assert_eq!(swap_response.alkanes.0[0].id, AlkaneId { block: 3, tx: 3 }, "User should receive contract alkanes");
    
    let user2_alkanes_received = swap_response.alkanes.0[0].value;
    assert!(user2_alkanes_received > 0, "User should receive a positive amount of alkanes");
    
    println!("User 2 swapped {} diesel and received {} alkanes", user2_diesel_amount, user2_alkanes_received);
    
    // Step 7: Verify the contract state after both users swapped
    let new_reserve_diesel = contract.reserve_diesel();
    let new_reserve_alkane = contract.reserve_alkane();
    
    assert_eq!(new_reserve_diesel, initial_reserve_diesel + user1_diesel_amount + user2_diesel_amount, 
               "Reserve diesel should be updated correctly");
    assert_eq!(new_reserve_alkane, initial_reserve_alkane - user1_alkanes_received - user2_alkanes_received, 
               "Reserve alkane should be updated correctly");
    
    // Step 8: User 1 swaps alkane for diesel
    let user1_alkane_swap_amount = user1_alkanes_received / 2;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: user1_alkane_swap_amount,
        }
    ]);
    
    // Set the context back to User 1
    let mut user1_context = context;
    user1_context.caller = AlkaneId { block: 1, tx: 1 };
    set_mock_context(user1_context);
    
    // User 1 swaps alkane for diesel
    let swap_result = contract.swap_alkane_for_diesel(user1_alkane_swap_amount);
    assert!(swap_result.is_ok(), "Swap operation failed: {:?}", swap_result.err());
    
    // Get the response
    let swap_response = swap_result.unwrap();
    
    // Step 9: Verify User 1 received diesel in return
    assert_eq!(swap_response.alkanes.0.len(), 1, "User should receive exactly one alkane transfer");
    assert_eq!(swap_response.alkanes.0[0].id, AlkaneId { block: 2, tx: 0 }, "User should receive diesel");
    
    let user1_diesel_received = swap_response.alkanes.0[0].value;
    assert!(user1_diesel_received > 0, "User should receive a positive amount of diesel");
    
    println!("User 1 swapped {} alkanes and received {} diesel", user1_alkane_swap_amount, user1_diesel_received);
    
    // Step 10: Verify the contract state after User 1 swapped back
    let final_reserve_diesel = contract.reserve_diesel();
    let final_reserve_alkane = contract.reserve_alkane();
    
    assert_eq!(final_reserve_diesel, new_reserve_diesel - user1_diesel_received, 
               "Reserve diesel should be updated correctly after swap");
    assert_eq!(final_reserve_alkane, new_reserve_alkane + user1_alkane_swap_amount, 
               "Reserve alkane should be updated correctly after swap");
    
    // Step 11: Calculate the current price
    let current_price = contract.current_price().unwrap();
    println!("Current price: {} diesel per alkane", current_price);
    
    // Step 12: Verify the price is reasonable
    assert!(current_price > 0, "Price should be positive");
    
    // Step 13: Verify the price has increased from the initial price
    let initial_price = initial_reserve_diesel / initial_reserve_alkane;
    assert!(current_price > initial_price, 
            "Price should increase as more diesel is swapped for alkane (initial: {}, current: {})", 
            initial_price, current_price);
    
    println!("E2E test completed successfully!");
    println!("Initial state: {} diesel, {} alkane reserve", initial_reserve_diesel, initial_reserve_alkane);
    println!("Final state: {} diesel, {} alkane reserve", final_reserve_diesel, final_reserve_alkane);
    println!("Price change: {} -> {} diesel per alkane", initial_price, current_price);
}

/// This test demonstrates adding liquidity to the AMM
#[test]
fn test_add_liquidity() {
    // Step 1: Initialize the contract
    let contract = BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"LiquidityToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"LIQ";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let initial_reserve_diesel = 1_000_000;
    let initial_reserve_alkane = 1_000_000;
    
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
    contract.init_contract(name, symbol, initial_reserve_diesel, initial_reserve_alkane).unwrap();
    
    // Step 2: User adds liquidity
    let diesel_amount = 50_000;
    let alkane_amount = 50_000;
    
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        },
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: alkane_amount,
        }
    ]);
    
    set_mock_context(context);
    
    // User adds liquidity
    let add_liquidity_result = contract.add_liquidity(diesel_amount, alkane_amount);
    assert!(add_liquidity_result.is_ok(), "Add liquidity operation failed: {:?}", add_liquidity_result.err());
    
    // Step 3: Verify the contract state after adding liquidity
    let new_reserve_diesel = contract.reserve_diesel();
    let new_reserve_alkane = contract.reserve_alkane();
    
    assert_eq!(new_reserve_diesel, initial_reserve_diesel + diesel_amount, 
               "Reserve diesel should be updated correctly");
    assert_eq!(new_reserve_alkane, initial_reserve_alkane + alkane_amount, 
               "Reserve alkane should be updated correctly");
    
    println!("User added {} diesel and {} alkane as liquidity", diesel_amount, alkane_amount);
    println!("New reserves: {} diesel, {} alkane", new_reserve_diesel, new_reserve_alkane);
    
    // Step 4: Verify the price remains the same after adding balanced liquidity
    let price = contract.current_price().unwrap();
    let initial_price = initial_reserve_diesel / initial_reserve_alkane;
    
    assert_eq!(price, initial_price, 
               "Price should remain the same after adding balanced liquidity");
    
    println!("Price after adding liquidity: {} diesel per alkane", price);
    println!("Add liquidity test completed successfully!");
}

/// This test demonstrates the full lifecycle of the AMM contract:
/// 1. Contract initialization
/// 2. Multiple users swapping
/// 3. Price discovery and slippage effects
/// 4. Adding liquidity
/// 5. Final state verification
#[test]
fn test_amm_contract_lifecycle() {
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
    
    let initial_reserve_diesel = 1_000_000;
    let initial_reserve_alkane = 1_000_000;
    
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
    contract.init_contract(name, symbol, initial_reserve_diesel, initial_reserve_alkane).unwrap();
    
    // Step 2: Define multiple users
    let users = vec![
        AlkaneId { block: 1, tx: 1 }, // Creator
        AlkaneId { block: 1, tx: 2 }, // User 2
        AlkaneId { block: 1, tx: 3 }, // User 3
        AlkaneId { block: 1, tx: 4 }, // User 4
        AlkaneId { block: 1, tx: 5 }, // User 5
    ];
    
    // Step 3: Users swap diesel for alkane in sequence
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
        
        // User swaps diesel for alkane
        let swap_response = contract.swap_diesel_for_alkane(*diesel_amount).unwrap();
        
        // Record the alkanes received
        user_alkanes[i] = swap_response.alkanes.0[0].value;
        
        println!("User {} swapped {} diesel and received {} alkanes", 
                 i + 1, diesel_amount, user_alkanes[i]);
        
        // Check the price after each swap
        let price = contract.current_price().unwrap();
        println!("Price after User {}: {} diesel per alkane", i + 1, price);
    }
    
    // Step 4: Verify the total state
    let total_diesel_provided: u128 = diesel_amounts.iter().sum();
    let total_alkanes_received: u128 = user_alkanes.iter().sum();
    
    assert_eq!(contract.reserve_diesel(), initial_reserve_diesel + total_diesel_provided,
               "Reserve diesel should match the total diesel provided");
    assert_eq!(contract.reserve_alkane(), initial_reserve_alkane - total_alkanes_received,
               "Reserve alkane should match the initial minus total alkanes received");
    
    // Step 5: Users swap alkane for diesel in reverse order
    let mut total_diesel_received = 0;
    
    for (i, (user, alkanes)) in users.iter().zip(user_alkanes.iter()).enumerate().rev() {
        // Only swap if the user has alkanes
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
            
            // User swaps alkane for diesel
            let swap_response = contract.swap_alkane_for_diesel(*alkanes).unwrap();
            
            // Record the diesel received
            let diesel_received = swap_response.alkanes.0[0].value;
            total_diesel_received += diesel_received;
            
            println!("User {} swapped {} alkanes and received {} diesel", 
                     i + 1, alkanes, diesel_received);
            
            // Check the price after each swap
            let price = contract.current_price().unwrap();
            println!("Price after User {} swap back: {} diesel per alkane", i + 1, price);
        }
    }
    
    // Step 6: Verify the final state
    let final_reserve_diesel = contract.reserve_diesel();
    let final_reserve_alkane = contract.reserve_alkane();
    
    println!("Final state: {} diesel, {} alkane reserve", final_reserve_diesel, final_reserve_alkane);
    println!("Total diesel provided: {}", total_diesel_provided);
    println!("Total diesel received: {}", total_diesel_received);
    
    // The final diesel reserve should be less than the initial reserve plus the total provided minus the total received
    // due to slippage and fees
    assert!(final_reserve_diesel < initial_reserve_diesel + total_diesel_provided - total_diesel_received,
            "Final diesel reserve should reflect slippage and fees");
    
    // The final alkane reserve should be equal to the initial reserve
    // (all received alkanes were swapped back)
    assert_eq!(final_reserve_alkane, initial_reserve_alkane,
               "Final alkane reserve should equal initial reserve after all swaps");
    
    println!("Lifecycle test completed successfully!");
}
