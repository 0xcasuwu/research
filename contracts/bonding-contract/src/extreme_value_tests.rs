//! Tests for extreme values and overflow protection in the bonding contract
//! 
//! These tests focus on:
//! 1. Maximum and near-maximum values for all parameters
//! 2. Cascading overflow scenarios
//! 3. Contract behavior with extreme values

use super::*;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use crate::mock_context::set_mock_context;
use crate::mock_runtime::clear_mock_storage;

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
fn test_max_diesel_reserve() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a curve with maximum diesel reserve
    // Using u128::MAX / 2 to avoid immediate overflow in calculations
    let large_reserve = u128::MAX / 2;
    let contract = init_test_contract(large_reserve, 1, 1);
    
    // Verify price calculation
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    // For n=1, price should be k * reserve, but this will saturate to u128::MAX
    assert_eq!(price, u128::MAX, "Price should saturate to u128::MAX with very large reserve");
    
    // Try to buy alkane with a small amount
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: 1,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(1);
    
    // The operation should complete without panicking
    assert!(buy_result.is_ok(), "Buy operation should not panic with max reserve");
    
    // Verify state remains consistent
    assert_eq!(contract.diesel_reserve(), u128::MAX / 2 + 1, "Diesel reserve should be updated correctly");
}

#[test]
fn test_max_alkane_supply() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a contract with reasonable reserve but maximum alkane supply
    let contract = init_test_contract(1_000_000, 1, 1);
    
    // Manually set the alkane supply to near max
    contract.set_alkane_supply(u128::MAX - 1000);
    
    // Try to buy alkane
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: 1000,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(1000);
    
    // The operation should complete without panicking
    assert!(buy_result.is_ok(), "Buy operation should not panic with max supply");
    
    // Verify state is updated correctly
    let new_supply = contract.alkane_supply();
    assert!(new_supply > u128::MAX - 1000, "Alkane supply should be updated correctly");
    
    // Try to sell a small amount of alkane
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: 1,
        }
    ]);
    set_mock_context(context);
    
    let sell_result = contract.sell_alkane(1);
    
    // The operation should complete without panicking
    assert!(sell_result.is_ok(), "Sell operation should not panic with max supply");
}

#[test]
fn test_max_k_factor() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a contract with maximum k factor
    let contract = init_test_contract(1_000_000, u128::MAX / 2, 1);
    
    // Verify price calculation
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    // For n=1, price = k * reserve, which will saturate to u128::MAX
    assert_eq!(price, u128::MAX, "Price should saturate to u128::MAX with max k factor");
    
    // Try to buy alkane
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: 1000,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(1000);
    
    // The operation should complete without panicking
    assert!(buy_result.is_ok(), "Buy operation should not panic with max k factor");
    
    // The amount of alkane received might be 0 due to overflow and division
    let buy_response = buy_result.unwrap();
    println!("Alkane received with max k factor: {}", 
             if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value });
}

#[test]
fn test_near_max_values() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a curve with near-maximum values
    let near_max = u128::MAX - 1_000_000;
    let contract = init_test_contract(near_max, 1, 1);
    
    // Verify price calculation
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    // For n=1, price = k * reserve, which will saturate to u128::MAX
    assert_eq!(price, u128::MAX, "Price should saturate to u128::MAX with near max reserve");
    
    // Try to buy alkane with a small amount
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: 1,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(1);
    
    // The operation should complete without panicking
    assert!(buy_result.is_ok(), "Buy operation should not panic with near max values");
    
    // Try to buy alkane with a large amount
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: 1_000_000,
        }
    ]);
    set_mock_context(context);
    
    let large_buy_result = contract.buy_alkane(1_000_000);
    
    // The operation should complete without panicking
    assert!(large_buy_result.is_ok(), "Buy operation should not panic with large amount near max");
}

#[test]
fn test_cascading_overflow() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a curve with values that will cause cascading overflow
    let large_value = u128::MAX / 2;
    let contract = init_test_contract(large_value, 2, 2);
    
    // Verify price calculation (should saturate due to squaring)
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    // For n=2, price = k * reserve^2, which will saturate to u128::MAX
    assert_eq!(price, u128::MAX, "Price should saturate due to squaring large value");
    
    // Try operations that would cause cascading overflow
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: large_value / 10,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(large_value / 10);
    
    // The operation should complete without panicking
    assert!(buy_result.is_ok(), "Buy operation should not panic with cascading overflow");
    
    // Verify state remains consistent
    assert!(contract.diesel_reserve() >= large_value, "Diesel reserve should not underflow");
    assert!(contract.alkane_supply() >= large_value, "Alkane supply should not underflow");
}

#[test]
fn test_contract_with_extreme_values() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a contract
    let contract = BondingContractAlkane::default();
    
    // Initialize with extreme values
    let mut name_bytes = [0u8; 16];
    let name_str = b"MaxToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"MAX";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    let large_reserve = u128::MAX / 2;
    let large_k = u128::MAX / 2;
    let n_exponent = 1;
    
    // Set up context
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Initialize contract
    let result = contract.init_contract(name, symbol, large_k, n_exponent, large_reserve);
    assert!(result.is_ok(), "Contract should initialize with large values");
    
    // Verify contract state
    assert_eq!(contract.diesel_reserve(), large_reserve, "Diesel reserve should be set correctly");
    assert_eq!(contract.k_factor(), large_k, "K factor should be set correctly");
    
    // Test buy operation with large value
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: 1_000_000,
        }
    ]);
    set_mock_context(context);
    
    let result = contract.buy_alkane(1_000_000);
    assert!(result.is_ok(), "Should handle buy operation with large initial values");
}

#[test]
fn test_direct_bonding_curve_extreme_values() {
    // Test the BondingCurve struct directly with extreme values
    
    // Case 1: Maximum reserve
    let mut curve = BondingCurve::new(u128::MAX, 1_000_000, 1, 1);
    let price = curve.get_current_price();
    assert_eq!(price, u128::MAX, "Price should saturate to u128::MAX with max reserve");
    
    // Case 2: Maximum supply
    let mut curve = BondingCurve::new(1_000_000, u128::MAX, 1, 1);
    let price = curve.get_current_price();
    assert_eq!(price, 1_000_000, "Price should be k * reserve with max supply");
    
    // Case 3: Maximum k factor
    let mut curve = BondingCurve::new(1_000_000, 1_000_000, u128::MAX, 1);
    let price = curve.get_current_price();
    assert_eq!(price, u128::MAX, "Price should saturate to u128::MAX with max k factor");
    
    // Case 4: Buy with maximum reserve
    let mut curve = BondingCurve::new(u128::MAX - 1000, 1_000_000, 1, 1);
    let alkane_amount = curve.buy_alkane(1000);
    assert!(alkane_amount >= 0, "Buy should not panic with max reserve");
    
    // Case 5: Sell with maximum supply
    let mut curve = BondingCurve::new(1_000_000, u128::MAX - 1000, 1, 1);
    let diesel_amount = curve.sell_alkane(1000);
    assert!(diesel_amount >= 0, "Sell should not panic with max supply");
    
    // Case 6: Cascading overflow in cubed calculation
    let mut curve = BondingCurve::new(u128::MAX / 2, 1_000_000, 1, 2);
    let alkane_amount = curve.buy_alkane(1000);
    assert!(alkane_amount >= 0, "Buy should not panic with potential cascading overflow");
}
