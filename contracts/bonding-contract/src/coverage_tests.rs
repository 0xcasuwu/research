//! Tests to cover gaps in test coverage for the bonding contract
//! 
//! These tests focus on:
//! 1. Context handling when no mock context is set
//! 2. Bonding curve n=0 case (constant price)
//! 3. Scaling factor edge cases
//! 4. Error propagation

use super::*;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use crate::mock_context::{set_mock_context, get_mock_context};
use crate::reset_mock_environment::reset_mock_environment;

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

// Helper function to initialize a contract for testing with specific parameters
fn init_test_contract(initial_diesel_reserve: u128, k_factor: u128, n_exponent: u128) -> BondingContractAlkane {
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
    // Create fixed-size arrays for u128::from_le_bytes
    let mut name_bytes = [0u8; 16];
    let name_str = b"TestToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"TEST";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    // Set up a default context
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
    
    contract
}

#[test]
fn test_no_mock_context() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
    // Clear the mock context to test the default context path
    crate::mock_context::clear_mock_context();
    
    // Try to get the context - should return the default context
    let context_result = contract.context();
    assert!(context_result.is_ok(), "Context should be available even without mock context");
    
    let context = context_result.unwrap();
    assert_eq!(context.caller, AlkaneId::default(), "Default context should have default caller");
    assert_eq!(context.myself, AlkaneId::default(), "Default context should have default myself");
    assert_eq!(context.incoming_alkanes.0.len(), 0, "Default context should have empty incoming alkanes");
    
    // Try to initialize the contract with the default context
    let mut name_bytes = [0u8; 16];
    let name_str = b"TestToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"TEST";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let initial_diesel_reserve = 1_000_000;
    let k_factor = 1;
    let n_exponent = 1;
    
    // Initialize the contract with default context
    let result = contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve);
    assert!(result.is_ok(), "Contract should initialize with default context");
    
    // Verify the contract was initialized correctly
    assert_eq!(contract.name(), "TestToken");
    assert_eq!(contract.symbol(), "TEST");
    assert_eq!(contract.diesel_reserve(), initial_diesel_reserve);
    assert_eq!(contract.alkane_supply(), initial_diesel_reserve);
    assert_eq!(contract.k_factor(), k_factor);
    assert_eq!(contract.n_exponent(), n_exponent);
    
    // Reset the mock context for other tests
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
}

#[test]
fn test_constant_price_curve() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a contract with n=0 (constant price curve)
    let contract = init_test_contract(1_000_000, 100, 0);
    
    // Verify the price is constant (k) regardless of reserve
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    assert_eq!(price, 100, "Price should be equal to k_factor for n=0");
    
    // Buy some alkane
    let diesel_amount = 10_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(diesel_amount);
    assert!(buy_result.is_ok(), "Buy operation should succeed with n=0");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    // For n=0, the price is constant, so the alkane amount should be diesel_amount * SCALING_FACTOR / k
    let expected_alkane = diesel_amount * 1_000_000 / 100;
    assert_eq!(alkane_amount, expected_alkane, "Alkane amount should match expected for n=0");
    
    // Verify the price is still constant after buying
    let price_after_buy_response = contract.current_price().unwrap();
    let price_after_buy = u128::from_le_bytes(price_after_buy_response.data.try_into().unwrap());
    assert_eq!(price_after_buy, 100, "Price should still be equal to k_factor after buying");
    
    // Sell some alkane
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: alkane_amount / 2, // Sell half
        }
    ]);
    set_mock_context(context);
    
    let sell_result = contract.sell_alkane(alkane_amount / 2);
    assert!(sell_result.is_ok(), "Sell operation should succeed with n=0");
    
    let sell_response = sell_result.unwrap();
    let diesel_returned = if sell_response.alkanes.0.is_empty() { 0 } else { sell_response.alkanes.0[0].value };
    
    // For n=0, the price is constant, so the diesel amount should be alkane_amount * k / SCALING_FACTOR
    let expected_diesel = (alkane_amount / 2) * 100 / 1_000_000;
    assert_eq!(diesel_returned, expected_diesel, "Diesel amount should match expected for n=0");
    
    // Verify the price is still constant after selling
    let price_after_sell_response = contract.current_price().unwrap();
    let price_after_sell = u128::from_le_bytes(price_after_sell_response.data.try_into().unwrap());
    assert_eq!(price_after_sell, 100, "Price should still be equal to k_factor after selling");
}

#[test]
fn test_scaling_factor_edge_cases() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Test case 1: Very small k_factor with large diesel_reserve
    let contract = init_test_contract(1_000_000_000, 1, 1);
    
    // Calculate expected price: k * reserve / SCALING_FACTOR
    let expected_price = 1 * 1_000_000_000 / 1_000_000;
    
    // Get the actual price
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    assert_eq!(price, expected_price, "Price should match expected with small k and large reserve");
    
    // Test buying with a very small amount
    let tiny_amount = 1;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: tiny_amount,
        }
    ]);
    set_mock_context(context);
    
    // Calculate expected alkane amount
    // For n=1: alkane = k * diesel * (reserve + diesel/2) / SCALING_FACTOR
    let expected_alkane = 1 * tiny_amount * (1_000_000_000 + tiny_amount/2) / 1_000_000;
    
    let buy_result = contract.buy_alkane(tiny_amount);
    assert!(buy_result.is_ok(), "Buy operation should succeed with small amount");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    assert_eq!(alkane_amount, expected_alkane, "Alkane amount should match expected for small purchase");
    
    // Test case 2: Very large k_factor with small diesel_reserve
    reset_mock_environment();
    let contract = init_test_contract(1000, 1_000_000, 1);
    
    // Calculate expected price: k * reserve / SCALING_FACTOR
    let expected_price = 1_000_000 * 1000 / 1_000_000;
    
    // Get the actual price
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    assert_eq!(price, expected_price, "Price should match expected with large k and small reserve");
    
    // Test buying with a moderate amount
    let amount = 100;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: amount,
        }
    ]);
    set_mock_context(context);
    
    // Calculate expected alkane amount
    // For n=1: alkane = k * diesel * (reserve + diesel/2) / SCALING_FACTOR
    let expected_alkane = 1_000_000 * amount * (1000 + amount/2) / 1_000_000;
    
    let buy_result = contract.buy_alkane(amount);
    assert!(buy_result.is_ok(), "Buy operation should succeed with moderate amount");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    assert_eq!(alkane_amount, expected_alkane, "Alkane amount should match expected for moderate purchase");
    
    // Test case 3: Very small values all around
    reset_mock_environment();
    let contract = init_test_contract(10, 10, 1);
    
    // Calculate expected price: k * reserve / SCALING_FACTOR
    let expected_price = 10 * 10 / 1_000_000;
    
    // Get the actual price
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    assert_eq!(price, expected_price, "Price should match expected with small values");
    
    // Test buying with a small amount
    let amount = 5;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: amount,
        }
    ]);
    set_mock_context(context);
    
    // Calculate expected alkane amount
    // For n=1: alkane = k * diesel * (reserve + diesel/2) / SCALING_FACTOR
    let expected_alkane = 10 * amount * (10 + amount/2) / 1_000_000;
    
    // With very small values, the result might be 0 due to integer division
    // In this case, the contract should return at least 1
    let expected_alkane = if expected_alkane == 0 { 1 } else { expected_alkane };
    
    let buy_result = contract.buy_alkane(amount);
    assert!(buy_result.is_ok(), "Buy operation should succeed with small values");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    assert_eq!(alkane_amount, expected_alkane, "Alkane amount should match expected for small values");
}

#[test]
fn test_error_propagation() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a contract
    let contract = BondingContractAlkane::default();
    
    // Set up a context
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Try to buy alkane without initializing the contract
    let diesel_amount = 1000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    // This should fail because the contract is not initialized
    let buy_result = contract.buy_alkane(diesel_amount);
    assert!(buy_result.is_err(), "Buy operation should fail on uninitialized contract");
    
    // Try to sell alkane without initializing the contract
    let alkane_amount = 1000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: alkane_amount,
        }
    ]);
    set_mock_context(context);
    
    // This should fail because the contract is not initialized
    let sell_result = contract.sell_alkane(alkane_amount);
    assert!(sell_result.is_err(), "Sell operation should fail on uninitialized contract");
    
    // Try to get the current price without initializing the contract
    let price_result = contract.current_price();
    assert!(price_result.is_err(), "Current price should fail on uninitialized contract");
    
    // Initialize the contract
    let mut name_bytes = [0u8; 16];
    let name_str = b"TestToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"TEST";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let initial_diesel_reserve = 1_000_000;
    let k_factor = 1;
    let n_exponent = 1;
    
    // Set up a default context
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
    
    // Now try to buy alkane with wrong diesel ID
    let diesel_amount = 1000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 1, tx: 1 }, // Wrong ID (not diesel)
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    // This should fail because no diesel was received
    let buy_result = contract.buy_alkane(diesel_amount);
    assert!(buy_result.is_err(), "Buy operation should fail with wrong diesel ID");
    assert_eq!(buy_result.unwrap_err().to_string(), "no diesel received", 
               "Error message should indicate no diesel received");
    
    // Try to sell alkane with wrong alkane ID
    let alkane_amount = 1000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 1, tx: 1 }, // Wrong ID (not contract alkane)
            value: alkane_amount,
        }
    ]);
    set_mock_context(context);
    
    // This should fail because no alkane was received
    let sell_result = contract.sell_alkane(alkane_amount);
    assert!(sell_result.is_err(), "Sell operation should fail with wrong alkane ID");
    assert_eq!(sell_result.unwrap_err().to_string(), "no alkane received", 
               "Error message should indicate no alkane received");
}

#[test]
fn test_bonding_curve_direct_n0() {
    // Test the BondingCurve struct directly with n=0
    
    // Create a curve with n=0
    let mut curve = BondingCurve::new(1_000_000, 1_000_000, 100, 0);
    
    // Verify the price is constant (k)
    let price = curve.get_current_price();
    assert_eq!(price, 100, "Price should be equal to k_factor for n=0");
    
    // Buy some alkane
    let diesel_amount = 10_000;
    let alkane_amount = curve.buy_alkane(diesel_amount);
    
    // For n=0, the price is constant, so the alkane amount should be diesel_amount * SCALING_FACTOR / k
    let expected_alkane = diesel_amount * 1_000_000 / 100;
    assert_eq!(alkane_amount, expected_alkane, "Alkane amount should match expected for n=0");
    
    // Verify the price is still constant after buying
    let price_after_buy = curve.get_current_price();
    assert_eq!(price_after_buy, 100, "Price should still be equal to k_factor after buying");
    
    // Sell some alkane
    let diesel_returned = curve.sell_alkane(alkane_amount / 2);
    
    // For n=0, the price is constant, so the diesel amount should be alkane_amount * k / SCALING_FACTOR
    let expected_diesel = (alkane_amount / 2) * 100 / 1_000_000;
    assert_eq!(diesel_returned, expected_diesel, "Diesel amount should match expected for n=0");
    
    // Verify the price is still constant after selling
    let price_after_sell = curve.get_current_price();
    assert_eq!(price_after_sell, 100, "Price should still be equal to k_factor after selling");
}

#[test]
fn test_get_buy_amount_zero_input() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let contract = init_test_contract(1_000_000, 1, 1);
    
    // Test get_buy_amount with zero input
    let diesel_amount = 0;
    
    // Get the response
    let response = contract.get_buy_amount(diesel_amount).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&response.data);
    let alkane_amount = u128::from_le_bytes(bytes);
    
    // Verify the response contains zero
    assert_eq!(alkane_amount, 0, "get_buy_amount should return 0 for zero input");
}

#[test]
fn test_get_sell_amount_zero_input() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let contract = init_test_contract(1_000_000, 1, 1);
    
    // Test get_sell_amount with zero input
    let alkane_amount = 0;
    
    // Get the response
    let response = contract.get_sell_amount(alkane_amount).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&response.data);
    let diesel_amount = u128::from_le_bytes(bytes);
    
    // Verify the response contains zero
    assert_eq!(diesel_amount, 0, "get_sell_amount should return 0 for zero input");
}

#[test]
fn test_get_sell_amount_too_much() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let contract = init_test_contract(1_000_000, 1, 1);
    
    // Test get_sell_amount with more alkane than the supply
    let alkane_amount = 2_000_000; // More than the supply (1_000_000)
    
    // Get the response
    let response = contract.get_sell_amount(alkane_amount).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&response.data);
    let diesel_amount = u128::from_le_bytes(bytes);
    
    // Verify the response contains zero
    assert_eq!(diesel_amount, 0, "get_sell_amount should return 0 for amount > supply");
}
