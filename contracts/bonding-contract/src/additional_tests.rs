//! Additional tests to achieve full coverage for the bonding contract

use super::*;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};

// Import the mock context functions
use crate::mock_context::{set_mock_context};
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

// Helper function to initialize a contract for testing
fn init_test_contract() -> BondingContractAlkane {
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
    contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve).unwrap();
    
    contract
}

#[test]
fn test_double_initialization() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let contract = init_test_contract();
    
    // Try to initialize it again with the same parameters
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
    
    // Try to initialize again - should fail
    let result = contract.init_contract(name, symbol, k_factor, n_exponent, initial_diesel_reserve);
    assert!(result.is_err(), "Second initialization should fail");
    assert_eq!(result.unwrap_err().to_string(), "already initialized", 
               "Error message should indicate already initialized");
}

#[test]
fn test_buy_alkane_no_diesel() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let contract = init_test_contract();
    
    // Set up a context with no diesel
    let context = create_context_with_alkanes(vec![]);
    set_mock_context(context);
    
    // Try to buy alkane with diesel amount > 0 but no diesel in context
    let result = contract.buy_alkane(1000);
    assert!(result.is_err(), "Buying alkane with no diesel should fail");
    assert_eq!(result.unwrap_err().to_string(), "no diesel received", 
               "Error message should indicate no diesel received");
}

#[test]
fn test_sell_alkane_no_alkane() {
    // Reset the mock environment
    reset_mock_environment();

    // Create and initialize a contract
    let contract = init_test_contract();

    // Set up a context with no alkane
    let context = create_context_with_alkanes(vec![]);
    set_mock_context(context);

    // Try to sell alkane with alkane amount > 0 but no alkane in context
    let result = contract.sell_alkane(1000);
    assert!(result.is_err(), "Selling alkane with no alkane should fail");
    
    // Get the error message
    let err_msg = result.unwrap_err().to_string();
    assert_eq!(err_msg, "no alkane received",
               "Error message should indicate no alkane received");
}

#[test]
fn test_sell_alkane_insufficient_reserve() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract with a small diesel reserve
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
    
    let initial_diesel_reserve = 100; // Small reserve
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
    
    // First, buy a lot of alkane to increase the supply
    let diesel_amount = 10_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let response = contract.buy_alkane(diesel_amount).unwrap();
    let alkane_amount = response.alkanes.0[0].value;
    
    // Now, try to sell more alkane than the reserve can handle
    // We'll manipulate the diesel reserve to simulate this scenario
    contract.set_diesel_reserve(10); // Set a very small diesel reserve
    
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: alkane_amount,
        }
    ]);
    set_mock_context(context);
    
    // Try to sell all alkane - should fail due to insufficient diesel reserve
    let result = contract.sell_alkane(alkane_amount);
    assert!(result.is_err(), "Selling alkane with insufficient diesel reserve should fail");
    assert_eq!(result.unwrap_err().to_string(), "insufficient diesel reserve", 
               "Error message should indicate insufficient diesel reserve");
}

#[test]
fn test_get_buy_amount_response() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let contract = init_test_contract();
    
    // Set up a default context
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Test get_buy_amount_response
    let diesel_amount = 10_000;
    
    // Get the bonding curve directly to calculate the expected amount
    let curve = contract.get_bonding_curve();
    let expected_alkane = curve.get_buy_amount(diesel_amount);
    
    // Get the response
    let response = contract.get_buy_amount_response(diesel_amount).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&response.data);
    let alkane_amount = u128::from_le_bytes(bytes);
    
    // Verify the response contains the correct alkane amount
    assert_eq!(alkane_amount, expected_alkane,
               "get_buy_amount_response should return the correct alkane amount");
}

#[test]
fn test_get_sell_amount_response() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let contract = init_test_contract();
    
    // Set up a default context
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // First, buy some alkane to ensure the contract has enough reserve
    let diesel_amount = 100_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    contract.buy_alkane(diesel_amount).unwrap();
    
    // Reset context for the test
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Test get_sell_amount_response
    let alkane_amount = 10_000;
    
    // Get the bonding curve directly to calculate the expected amount
    let curve = contract.get_bonding_curve();
    let expected_diesel = curve.get_sell_amount(alkane_amount);
    
    // Get the response
    let response = contract.get_sell_amount_response(alkane_amount).unwrap();
    
    // Make sure the response data is exactly 16 bytes (size of u128)
    assert_eq!(response.data.len(), 16, "Response data should be exactly 16 bytes");
    
    // Convert the response data to u128 safely
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&response.data);
    let diesel_amount = u128::from_le_bytes(bytes);
    
    // Verify the response contains the correct diesel amount
    assert_eq!(diesel_amount, expected_diesel, 
               "get_sell_amount_response should return the correct diesel amount");
}

#[test]
fn test_balance_of_and_set_balance() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let contract = init_test_contract();
    
    // Set up a default context
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Test address
    let address = 12345u128;
    
    // Initial balance should be 0
    let initial_balance = contract.balance_of(address);
    assert_eq!(initial_balance, 0, "Initial balance should be 0");
    
    // Set a balance
    let new_balance = 50_000;
    contract.set_balance(address, new_balance);
    
    // Check the updated balance
    let updated_balance = contract.balance_of(address);
    assert_eq!(updated_balance, new_balance, "Balance should be updated correctly");
    
    // Set a different balance
    let another_balance = 75_000;
    contract.set_balance(address, another_balance);
    
    // Check the updated balance again
    let final_balance = contract.balance_of(address);
    assert_eq!(final_balance, another_balance, "Balance should be updated correctly again");
}

#[test]
fn test_trim_function() {
    // Test the trim function with various inputs
    
    // Test with a simple string
    let test_str = b"Hello";
    let mut bytes = [0u8; 16];
    bytes[..test_str.len()].copy_from_slice(test_str);
    let value = u128::from_le_bytes(bytes);
    
    let result = trim(value);
    assert_eq!(result, "Hello", "trim should correctly handle simple strings");
    
    // Test with a string containing null bytes
    let test_str = b"World\0\0\0";
    let mut bytes = [0u8; 16];
    bytes[..test_str.len()].copy_from_slice(test_str);
    let value = u128::from_le_bytes(bytes);
    
    let result = trim(value);
    assert_eq!(result, "World", "trim should remove trailing null bytes");
    
    // Test with an empty string
    let value = 0u128;
    let result = trim(value);
    assert_eq!(result, "", "trim should handle empty strings");
    
    // Test with a full 16-byte string
    let test_str = b"0123456789ABCDEF";
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(test_str);
    let value = u128::from_le_bytes(bytes);
    
    let result = trim(value);
    assert_eq!(result, "0123456789ABCDEF", "trim should handle full 16-byte strings");
}

#[test]
fn test_buy_alkane_zero_output() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a contract with special parameters that would result in zero output
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
    let k_factor = 1000000; // Very high k factor to make small purchases result in zero output
    let n_exponent = 2; // Quadratic curve to amplify the effect
    
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
    
    // Set up a context with a tiny amount of diesel
    let tiny_diesel_amount = 1;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: tiny_diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    // Try to buy alkane with a tiny amount of diesel
    // With the high k factor and quadratic curve, this should result in zero output
    let result = contract.buy_alkane(tiny_diesel_amount);
    assert!(result.is_err(), "Buying alkane with insufficient output should fail");
    assert_eq!(result.unwrap_err().to_string(), "insufficient output amount", 
               "Error message should indicate insufficient output amount");
}

#[test]
fn test_sell_alkane_zero_output() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a contract with special parameters that would result in zero output
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
    let k_factor = 1000000; // Very high k factor to make small sales result in zero output
    let n_exponent = 2; // Quadratic curve to amplify the effect
    
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
    
    // First, buy some alkane to have some to sell
    let diesel_amount = 100_000;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        }
    ]);
    set_mock_context(context);
    
    let response = contract.buy_alkane(diesel_amount).unwrap();
    let alkane_amount = response.alkanes.0[0].value;
    
    // Set up a context with a tiny amount of alkane
    let tiny_alkane_amount = 1;
    let context = create_context_with_alkanes(vec![
        AlkaneTransfer {
            id: AlkaneId { block: 3, tx: 3 }, // Contract alkane
            value: tiny_alkane_amount,
        }
    ]);
    set_mock_context(context);
    
    // Try to sell a tiny amount of alkane
    // With the high k factor and quadratic curve, this should result in zero output
    let result = contract.sell_alkane(tiny_alkane_amount);
    assert!(result.is_err(), "Selling alkane with insufficient output should fail");
    assert_eq!(result.unwrap_err().to_string(), "insufficient output amount", 
               "Error message should indicate insufficient output amount");
}

#[test]
fn test_trait_implementation() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create and initialize a contract
    let mut contract = init_test_contract();
    
    // Test the BondingContract trait implementation
    
    // Test diesel_reserve
    let diesel_reserve = contract.diesel_reserve();
    assert_eq!(diesel_reserve, 1_000_000, "diesel_reserve should return the correct value");
    
    // Test alkane_supply
    let alkane_supply = contract.alkane_supply();
    assert_eq!(alkane_supply, 1_000_000, "alkane_supply should return the correct value");
    
    // Test current_price
    let price_response = contract.current_price().unwrap();
    let price = u128::from_le_bytes(price_response.data.try_into().unwrap());
    assert_eq!(price, 1_000_000, "current_price should return the correct value");
    
    // Test buy_alkane through the trait
    let diesel_amount = 10_000;
    let response = contract.buy_alkane(diesel_amount).unwrap();
    
    // Verify the response
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain exactly one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, AlkaneId { block: 3, tx: 3 }, "Response should contain contract alkane");
    
    let alkane_amount = response.alkanes.0[0].value;
    assert!(alkane_amount > 0, "Alkane amount should be positive");
    
    // Test sell_alkane through the trait
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
}

#[test]
fn test_execute_direct_call() {
    // Reset the mock environment
    reset_mock_environment();
    
    // Create a contract
    let contract = BondingContractAlkane::default();
    
    // Set up a default context
    let context = Context {
        caller: AlkaneId { block: 1, tx: 1 },
        myself: AlkaneId { block: 3, tx: 3 },
        incoming_alkanes: Default::default(),
        vout: 0,
        inputs: vec![],
    };
    set_mock_context(context);
    
    // Try to call execute directly - should fail
    let result = contract.execute();
    assert!(result.is_err(), "Direct execute call should fail");
    assert_eq!(result.unwrap_err().to_string(), "Use the declare_alkane macro instead", 
               "Error message should indicate to use the declare_alkane macro");
}
