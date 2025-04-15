//! Unit tests for the Bonding Curve contract

use super::*;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use crate::mock_context::{get_mock_context, set_mock_context};
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

#[test]
fn test_init_contract() {
    // Reset the mock environment
    reset_mock_environment();
    
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
    
    let initial_diesel_reserve = 1_000_000;
    let k_factor = 1;
    let n_exponent = 1; // Linear bonding curve
    
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
    let result = contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve);
    assert!(result.is_ok(), "Contract initialization failed: {:?}", result.err());
    
    // Verify the contract was initialized correctly
    assert_eq!(contract.name(), "TestToken");
    assert_eq!(contract.symbol(), "TEST");
    assert_eq!(contract.diesel_reserve(), initial_diesel_reserve);
    assert_eq!(contract.alkane_supply(), initial_diesel_reserve); // Initial supply is 1:1 with reserve
    assert_eq!(contract.k_factor(), k_factor);
    assert_eq!(contract.n_exponent(), n_exponent);
}

#[test]
fn test_buy_alkane() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
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
    let n_exponent = 1; // Linear bonding curve
    
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
    
    // Set up a context with diesel
    let diesel_amount = 10_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    // Buy alkane with diesel - note that we're not using the diesel_amount parameter
    // since the function ignores it and uses the context's incoming alkanes instead
    let response = contract.buy_alkane(0).unwrap();
    
    // Verify the response
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain exactly one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, AlkaneId { block: 3, tx: 3 }, "Response should contain contract alkane");
    
    let alkane_amount = response.alkanes.0[0].value;
    assert!(alkane_amount > 0, "Alkane amount should be positive");
    
    // Verify the contract state
    assert_eq!(contract.diesel_reserve(), initial_diesel_reserve + diesel_amount, 
               "Diesel reserve should be updated correctly");
    
    // For a linear bonding curve, the alkane supply should increase
    let initial_alkane_supply = initial_diesel_reserve;
    assert_eq!(contract.alkane_supply(), initial_alkane_supply + alkane_amount, 
               "Alkane supply should be updated correctly");
    
    // Verify the price
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    let initial_price = k_factor * initial_diesel_reserve;
    assert!(price > initial_price, "Price should increase after buying alkane");
}

#[test]
fn test_sell_alkane() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
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
    let n_exponent = 1; // Linear bonding curve
    
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
    
    // First, buy alkane with diesel to get some alkane
    let diesel_amount = 10_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let response = contract.buy_alkane(0).unwrap();
    let alkane_amount = response.alkanes.0[0].value;
    
    // Now, sell alkane for diesel
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: alkane_amount,
        }
    ]);
    set_mock_context(context);
    
    let response = contract.sell_alkane(alkane_amount).unwrap();
    
    // Verify the response
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain exactly one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, AlkaneId { block: 2, tx: 0 }, "Response should contain diesel");
    
    let diesel_received = response.alkanes.0[0].value;
    assert!(diesel_received > 0, "Diesel amount should be positive");
    assert!(diesel_received < diesel_amount, "Diesel received should be less than diesel provided due to price impact");
    
    // Verify the contract state
    assert_eq!(contract.diesel_reserve(), initial_diesel_reserve + diesel_amount - diesel_received, 
               "Diesel reserve should be updated correctly");
    
    // The alkane supply should be back to the initial value
    let initial_alkane_supply = initial_diesel_reserve;
    assert_eq!(contract.alkane_supply(), initial_alkane_supply, 
               "Alkane supply should be back to initial value");
}

#[test]
fn test_current_price() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract with linear bonding curve
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
    let n_exponent = 1; // Linear bonding curve
    
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
    
    // Get the initial price
    let initial_price_response = contract.current_price().unwrap();
    let initial_price = u128::from_le_bytes(initial_price_response.data.try_into().unwrap());
    assert_eq!(initial_price, k_factor * initial_diesel_reserve, 
               "Initial price should be k_factor * diesel_reserve");
    
    // Buy alkane with diesel
    let diesel_amount = 10_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let response = contract.buy_alkane(0).unwrap();
    let alkane_amount = response.alkanes.0[0].value;
    
    // Get the price after buy
    let price_after_buy_response = contract.current_price().unwrap();
    let price_after_buy = u128::from_le_bytes(price_after_buy_response.data.try_into().unwrap());
    assert!(price_after_buy > initial_price, "Price should increase after buying alkane");
    
    // Sell alkane for diesel
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: alkane_amount,
        }
    ]);
    set_mock_context(context);
    
    contract.sell_alkane(alkane_amount).unwrap();
    
    // Get the price after sell
    let price_after_sell_response = contract.current_price().unwrap();
    let price_after_sell = u128::from_le_bytes(price_after_sell_response.data.try_into().unwrap());
    assert!(price_after_sell < price_after_buy, "Price should decrease after selling alkane");
}

#[test]
fn test_get_buy_amount() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract with linear bonding curve
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
    let n_exponent = 1; // Linear bonding curve
    
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
    
    // Test with small amount
    let small_diesel = 1_000;
    let small_alkane_response = contract.get_buy_amount(small_diesel).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(small_alkane_response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&small_alkane_response.data);
    let small_alkane = u128::from_le_bytes(bytes);
    
    assert!(small_alkane > 0, "Small alkane amount should be positive");
    
    // Test with large amount
    let large_diesel = 100_000;
    let large_alkane_response = contract.get_buy_amount(large_diesel).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(large_alkane_response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&large_alkane_response.data);
    let large_alkane = u128::from_le_bytes(bytes);
    
    assert!(large_alkane > 0, "Large alkane amount should be positive");
    
    // Verify that larger input gives larger output
    assert!(large_alkane > small_alkane, "Larger diesel amount should give larger alkane amount");
    
    // Calculate expected alkane amounts using our formula
    let small_avg_price = initial_diesel_reserve + small_diesel / 2;
    let small_expected = k_factor * small_diesel * small_avg_price / 1_000_000;
    
    let large_avg_price = initial_diesel_reserve + large_diesel / 2;
    let large_expected = k_factor * large_diesel * large_avg_price / 1_000_000;
    
    // Verify alkane amounts match expected
    assert_eq!(small_alkane, small_expected, "Small alkane amount should match expected");
    assert_eq!(large_alkane, large_expected, "Large alkane amount should match expected");
    
    // Verify that the ratio is not linear (due to price impact)
    let ratio_small = small_alkane as f64 / small_diesel as f64;
    let ratio_large = large_alkane as f64 / large_diesel as f64;
    
    // Debug output
    println!("small_diesel: {}, small_alkane: {}, ratio_small: {}", small_diesel, small_alkane, ratio_small);
    println!("large_diesel: {}, large_alkane: {}, ratio_large: {}", large_diesel, large_alkane, ratio_large);
    
    assert!(ratio_large < ratio_small, "Larger buys should have less favorable price impact");
}

#[test]
fn test_get_sell_amount() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract with linear bonding curve
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
    let n_exponent = 1; // Linear bonding curve
    
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
    
    // First, buy some alkane to increase the supply
    let diesel_amount = 100_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let response = contract.buy_alkane(diesel_amount).unwrap();
    let alkane_bought = response.alkanes.0[0].value;
    
    // Reset the context
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Test with small amount
    let small_alkane = alkane_bought / 10;
    let small_diesel_response = contract.get_sell_amount(small_alkane).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(small_diesel_response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&small_diesel_response.data);
    let small_diesel = u128::from_le_bytes(bytes);
    
    assert!(small_diesel > 0, "Small diesel amount should be positive");
    
    // Test with large amount
    let large_alkane = alkane_bought / 2;
    let large_diesel_response = contract.get_sell_amount(large_alkane).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(large_diesel_response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&large_diesel_response.data);
    let large_diesel = u128::from_le_bytes(bytes);
    
    assert!(large_diesel > 0, "Large diesel amount should be positive");
    
    // Verify that larger input gives larger output
    assert!(large_diesel > small_diesel, "Larger alkane amount should give larger diesel amount");
    
    // Verify that the ratio is not linear (due to price impact)
    let ratio_small = small_diesel as f64 / small_alkane as f64;
    let ratio_large = large_diesel as f64 / large_alkane as f64;
    assert!(ratio_large < ratio_small, "Larger sells should have less favorable price impact");
}

#[test]
fn test_quadratic_bonding_curve() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract with quadratic bonding curve
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
    let n_exponent = 2; // Quadratic bonding curve
    
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
    
    // Get the initial price
    let initial_price_response = contract.current_price().unwrap();
    let initial_price = u128::from_le_bytes(initial_price_response.data.try_into().unwrap());
    assert_eq!(initial_price, k_factor * initial_diesel_reserve * initial_diesel_reserve, 
               "Initial price should be k_factor * diesel_reserve^2");
    
    // Buy alkane with diesel
    let diesel_amount = 10_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let response = contract.buy_alkane(0).unwrap();
    let alkane_amount = response.alkanes.0[0].value;
    
    // Get the price after buy
    let price_after_buy_response = contract.current_price().unwrap();
    let price_after_buy = u128::from_le_bytes(price_after_buy_response.data.try_into().unwrap());
    assert!(price_after_buy > initial_price, "Price should increase after buying alkane");
    
    // The price increase should be more dramatic with a quadratic curve
    let new_diesel_reserve = initial_diesel_reserve + diesel_amount;
    let expected_price = k_factor * new_diesel_reserve * new_diesel_reserve;
    assert_eq!(price_after_buy, expected_price, "Price should follow quadratic curve");
}

#[test]
fn test_get_name() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
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
    
    // Get the name
    let response = contract.get_name().unwrap();
    let name_str = String::from_utf8(response.data).unwrap();
    assert_eq!(name_str, "TestToken", "Name should be TestToken");
}

#[test]
fn test_get_symbol() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
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
    
    // Get the symbol
    let response = contract.get_symbol().unwrap();
    let symbol_str = String::from_utf8(response.data).unwrap();
    assert_eq!(symbol_str, "TEST", "Symbol should be TEST");
}

#[test]
fn test_get_diesel_reserve() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
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
    
    // Get the diesel reserve
    let response = contract.get_diesel_reserve().unwrap();
    let diesel_reserve = u128::from_le_bytes(response.data.try_into().unwrap());
    assert_eq!(diesel_reserve, initial_diesel_reserve, "Diesel reserve should be initial_diesel_reserve");
}

#[test]
fn test_get_alkane_supply() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a new contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract with standard values
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
    let mut name_bytes = [0u8; 16];
    let name_str = b"TestToken";
    name_bytes[..name_str.len()].copy_from_slice(name_str);
    let name = u128::from_le_bytes(name_bytes);
    
    let mut symbol_bytes = [0u8; 16];
    let symbol_str = b"TEST";
    symbol_bytes[..symbol_str.len()].copy_from_slice(symbol_str);
    let symbol = u128::from_le_bytes(symbol_bytes);
    
    contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve).unwrap();
    
    // Get the alkane supply
    let response = contract.get_alkane_supply().unwrap();
    let alkane_supply = u128::from_le_bytes(response.data.try_into().unwrap());
    assert_eq!(alkane_supply, initial_diesel_reserve, "Alkane supply should be initial_diesel_reserve");
}
