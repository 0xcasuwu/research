//! Tests for precision and rounding issues in the bonding contract
//! 
//! These tests focus on:
//! 1. Small value tests to detect precision loss
//! 2. Rounding error accumulation tests
//! 3. Scaling factor tests

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
fn test_small_values() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a curve with standard values
    let contract = init_test_contract(1_000_000, 1, 1);
    
    // Try to buy with minimal diesel
    let tiny_amount = 1;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: tiny_amount,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(tiny_amount);
    
    // The operation should complete without panicking
    assert!(buy_result.is_ok(), "Buy operation should not panic with tiny amount");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    // Due to the 1_000_000 scaling factor, we might get 0 alkane for a tiny diesel amount
    println!("Alkane received for 1 diesel: {}", alkane_amount);
    
    // If we get 0, it indicates precision loss
    if alkane_amount == 0 {
        println!("WARNING: Precision loss detected - received 0 alkane for 1 diesel");
    }
    
    // Try with a slightly larger amount
    let small_amount = 1000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: small_amount,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(small_amount);
    assert!(buy_result.is_ok(), "Buy operation should not panic with small amount");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    // This should give a non-zero amount
    assert!(alkane_amount > 0, "Should receive non-zero alkane for small diesel amount");
    println!("Alkane received for 1000 diesel: {}", alkane_amount);
    
    // Try to sell a tiny amount of alkane
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: 1,
        }
    ]);
    set_mock_context(context);
    
    let sell_result = contract.sell_alkane(1);
    
    // The operation should complete without panicking
    assert!(sell_result.is_ok(), "Sell operation should not panic with tiny amount");
    
    let sell_response = sell_result.unwrap();
    let diesel_returned = if sell_response.alkanes.0.is_empty() { 0 } else { sell_response.alkanes.0[0].value };
    
    // Due to the 1_000_000 scaling factor, we might get 0 diesel for a tiny alkane amount
    println!("Diesel received for 1 alkane: {}", diesel_returned);
    
    // If we get 0, it indicates precision loss
    if diesel_returned == 0 {
        println!("WARNING: Precision loss detected - received 0 diesel for 1 alkane");
    }
}

#[test]
fn test_division_precision_loss() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a curve with values that would cause division precision loss
    // High reserve, low k_factor
    let contract = init_test_contract(1_000_000_000, 1, 1);
    
    // Get the current price
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    println!("Price with high reserve: {}", price);
    
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
    assert!(sell_result.is_ok(), "Sell operation should not panic with small amount and high price");
    
    let sell_response = sell_result.unwrap();
    let diesel_returned = if sell_response.alkanes.0.is_empty() { 0 } else { sell_response.alkanes.0[0].value };
    
    // Due to integer division with a high price, we might get 0 diesel
    println!("Diesel received for 1 alkane with price {}: {}", price, diesel_returned);
    
    // If we get 0, it indicates precision loss
    if diesel_returned == 0 {
        println!("WARNING: Precision loss detected - received 0 diesel for 1 alkane with high price");
    }
    
    // Now test with low reserve, high k_factor
    let contract = init_test_contract(1000, 1000, 1);
    
    // Get the current price
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    
    println!("Price with low reserve, high k_factor: {}", price);
    
    // Try to buy a small amount of diesel
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: 1,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(1);
    
    // The operation should complete without panicking
    assert!(buy_result.is_ok(), "Buy operation should not panic with small amount and high k_factor");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    // Due to the 1_000_000 scaling factor and high k_factor, we might get 0 alkane
    println!("Alkane received for 1 diesel with high k_factor: {}", alkane_amount);
    
    // If we get 0, it indicates precision loss
    if alkane_amount == 0 {
        println!("WARNING: Precision loss detected - received 0 alkane for 1 diesel with high k_factor");
    }
}

#[test]
fn test_rounding_error_accumulation() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a curve with standard values
    let contract = init_test_contract(1_000_000, 1, 1);
    
    // Record initial state
    let initial_reserve = contract.diesel_reserve();
    let initial_supply = contract.alkane_supply();
    
    // Perform many small buy operations
    let num_operations = 100;
    let small_amount = 10;
    let mut total_diesel_spent = 0;
    let mut total_alkane_received = 0;
    
    for i in 0..num_operations {
        let context = create_context_with_alkanes(vec![
            AlkaneTransfer {
                id: AlkaneId { block: 2, tx: 0 }, // Diesel
                value: small_amount,
            }
        ]);
        set_mock_context(context);
        
        let buy_result = contract.buy_alkane(small_amount);
        assert!(buy_result.is_ok(), "Buy operation should not panic in iteration {}", i);
        
        let buy_response = buy_result.unwrap();
        let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
        
        total_diesel_spent += small_amount;
        total_alkane_received += alkane_amount;
    }
    
    println!("Total diesel spent: {}", total_diesel_spent);
    println!("Total alkane received: {}", total_alkane_received);
    
    // Now sell all received alkane
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: total_alkane_received,
        }
    ]);
    set_mock_context(context);
    
    let sell_result = contract.sell_alkane(total_alkane_received);
    assert!(sell_result.is_ok(), "Sell operation should not panic");
    
    let sell_response = sell_result.unwrap();
    let diesel_returned = if sell_response.alkanes.0.is_empty() { 0 } else { sell_response.alkanes.0[0].value };
    
    // Calculate the difference (slippage + rounding errors)
    let difference = total_diesel_spent as i128 - diesel_returned as i128;
    let percentage = difference as f64 / total_diesel_spent as f64 * 100.0;
    
    println!("Total diesel returned: {}", diesel_returned);
    println!("Difference: {} ({}%)", difference, percentage);
    
    // Verify the difference is reasonable (some difference is expected due to slippage)
    assert!(percentage < 10.0, "Difference should be less than 10% of total spent");
    
    // Verify final state is close to initial state
    let final_reserve = contract.diesel_reserve();
    let final_supply = contract.alkane_supply();
    
    let reserve_diff = final_reserve as i128 - initial_reserve as i128;
    let supply_diff = final_supply as i128 - initial_supply as i128;
    
    println!("Initial reserve: {}, Final reserve: {}, Difference: {}", 
             initial_reserve, final_reserve, reserve_diff);
    println!("Initial supply: {}, Final supply: {}, Difference: {}", 
             initial_supply, final_supply, supply_diff);
    
    // Some difference is expected due to rounding, but it should be small
    assert!(reserve_diff.abs() < 1000, "Reserve should be close to initial value");
    assert!(supply_diff.abs() < 1000, "Supply should be close to initial value");
}

#[test]
fn test_scaling_factor_impact() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a curve with standard values
    let contract = init_test_contract(1_000_000, 1, 1);
    
    // Get the bonding curve directly to calculate expected amounts
    let curve = contract.get_bonding_curve();
    
    // Buy a large amount to test the scaling factor's impact
    let large_amount = 1_000_000_000; // 1 billion
    
    // Calculate expected amount without the scaling factor
    // For n=1: alkane = k * diesel * (reserve + diesel/2)
    let reserve = 1_000_000;
    let k = 1;
    let expected_without_scaling = k * large_amount * (reserve + large_amount/2);
    
    // Calculate expected amount with the scaling factor
    let expected_with_scaling = expected_without_scaling / 1_000_000;
    
    // Get the actual amount from the contract
    let buy_amount_response = contract.get_buy_amount(large_amount).unwrap();
    let actual_amount = u128::from_le_bytes(buy_amount_response.data.try_into().unwrap());
    
    println!("Expected without scaling: {}", expected_without_scaling);
    println!("Expected with scaling: {}", expected_with_scaling);
    println!("Actual amount: {}", actual_amount);
    
    // Verify the scaling factor is applied correctly
    assert_eq!(actual_amount, expected_with_scaling, 
               "Alkane amount should match expected amount with scaling factor");
    
    // Verify the impact of the scaling factor
    let ratio_without_scaling = expected_without_scaling as f64 / large_amount as f64;
    let ratio_with_scaling = actual_amount as f64 / large_amount as f64;
    
    println!("Ratio without scaling: {}", ratio_without_scaling);
    println!("Ratio with scaling: {}", ratio_with_scaling);
    
    // The ratio with scaling should be 1/1_000_000 of the ratio without scaling
    assert!((ratio_without_scaling / ratio_with_scaling - 1_000_000.0).abs() < 1.0,
            "Scaling factor should reduce ratio by factor of 1_000_000");
    
    // Test the impact on small values
    let small_amount = 1;
    let buy_amount_response = contract.get_buy_amount(small_amount).unwrap();
    let actual_small_amount = u128::from_le_bytes(buy_amount_response.data.try_into().unwrap());
    
    println!("Amount for 1 diesel: {}", actual_small_amount);
    
    // For very small amounts, the scaling factor might cause the result to be 0
    // This is a precision issue that should be documented
    if actual_small_amount == 0 {
        println!("WARNING: Scaling factor causes precision loss for small amounts");
    }
}

#[test]
fn test_consistent_scaling() {
    // Clear any previous state
    clear_mock_storage();
    
    // Create a curve with standard values
    let contract = init_test_contract(1_000_000, 1, 1);
    
    // Buy a moderate amount of alkane
    let diesel_amount = 100_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(diesel_amount);
    assert!(buy_result.is_ok(), "Buy operation should not panic");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    println!("Diesel spent: {}", diesel_amount);
    println!("Alkane received: {}", alkane_amount);
    
    // Now sell the alkane
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: alkane_amount,
        }
    ]);
    set_mock_context(context);
    
    let sell_result = contract.sell_alkane(alkane_amount);
    assert!(sell_result.is_ok(), "Sell operation should not panic");
    
    let sell_response = sell_result.unwrap();
    let diesel_returned = if sell_response.alkanes.0.is_empty() { 0 } else { sell_response.alkanes.0[0].value };
    
    println!("Diesel returned: {}", diesel_returned);
    
    // Calculate the slippage
    let slippage = diesel_amount as i128 - diesel_returned as i128;
    let slippage_percentage = slippage as f64 / diesel_amount as f64 * 100.0;
    
    println!("Slippage: {} ({}%)", slippage, slippage_percentage);
    
    // Some slippage is expected due to the bonding curve design
    // But it should be reasonable
    assert!(slippage_percentage > 0.0, "Slippage should be positive");
    assert!(slippage_percentage < 50.0, "Slippage should be less than 50%");
    
    // Test with n=2 to see if scaling is consistent
    let contract = init_test_contract(1_000_000, 1, 2);
    
    // Buy a moderate amount of alkane
    let diesel_amount = 100_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let buy_result = contract.buy_alkane(diesel_amount);
    assert!(buy_result.is_ok(), "Buy operation should not panic with n=2");
    
    let buy_response = buy_result.unwrap();
    let alkane_amount = if buy_response.alkanes.0.is_empty() { 0 } else { buy_response.alkanes.0[0].value };
    
    println!("Diesel spent (n=2): {}", diesel_amount);
    println!("Alkane received (n=2): {}", alkane_amount);
    
    // Now sell the alkane
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: alkane_amount,
        }
    ]);
    set_mock_context(context);
    
    let sell_result = contract.sell_alkane(alkane_amount);
    assert!(sell_result.is_ok(), "Sell operation should not panic with n=2");
    
    let sell_response = sell_result.unwrap();
    let diesel_returned = if sell_response.alkanes.0.is_empty() { 0 } else { sell_response.alkanes.0[0].value };
    
    println!("Diesel returned (n=2): {}", diesel_returned);
    
    // Calculate the slippage for n=2
    let slippage_n2 = diesel_amount as i128 - diesel_returned as i128;
    let slippage_percentage_n2 = slippage_n2 as f64 / diesel_amount as f64 * 100.0;
    
    println!("Slippage (n=2): {} ({}%)", slippage_n2, slippage_percentage_n2);
    
    // Slippage should be different for different curve types
    // For n=2, slippage should be higher than for n=1
    assert!(slippage_percentage_n2 > slippage_percentage, 
            "Slippage for n=2 should be higher than for n=1");
}
