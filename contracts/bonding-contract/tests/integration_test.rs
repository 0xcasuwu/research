//! Integration tests for the bonding contract

use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::context::Context;
// Import BondingContract trait to access current_price method
use bonding_contract::BondingContract;
// Import BondContract trait to access bond functionality
use bonding_contract::BondContract;
// Import the mock_context module from the bonding-contract crate
use bonding_contract::mock_context::set_mock_context;
use bonding_contract::reset_mock_environment;
use std::sync::Once;

// Used to ensure reset is called only once per test
static INIT: Once = Once::new();

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

// Helper function to ensure environment is reset before each test
fn setup_test() {
    // Reset the mock environment at the beginning of each test
    reset_mock_environment::reset();
}

// Simple test to verify the bonding contract functionality
#[test]
fn test_bonding_contract_e2e() {
    // Setup the test environment
    setup_test();
    
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

// Test bond functionality
#[test]
fn test_bond_functionality() {
    // Setup the test environment
    setup_test();
    
    // Create a new bonding contract
    let mut contract = bonding_contract::BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"BondToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"BT";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    // Bond parameters
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 2_000_000;
    let half_life = 86400; // 1 day in seconds
    let level_bips = 100; // 1%
    let term = 604800; // 1 week in seconds
    
    // Set up a default context for initialization
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Initialize the bond contract
    let init_result = contract.init_bond_contract(
        name, 
        symbol, 
        virtual_input_reserves, 
        virtual_output_reserves, 
        half_life, 
        level_bips, 
        term
    );
    assert!(init_result.is_ok(), "Bond contract initialization failed: {:?}", init_result.err());
    
    // We can't directly access private methods, so we'll use the public API
    // For testing purposes, we'll use the BondContract trait methods
    
    // Test purchasing a bond
    let diesel_amount = 10_000;
    let min_output = 1;
    let to = 1; // Address 1
    
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    // Purchase a bond
    let purchase_result = contract.purchase_bond(to, min_output);
    assert!(purchase_result.is_ok(), "Bond purchase failed: {:?}", purchase_result.err());
    
    // Check the position count
    let position_count = contract.position_count_of(to);
    assert_eq!(position_count, 1, "User should have exactly one bond position");
    
    println!("Bond functionality test completed successfully!");
}
