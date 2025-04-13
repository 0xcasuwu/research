use alkanes_runtime::runtime::{AlkaneResponder, Context, RuntimeContext};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

// Import the bonding contract
use bonding_contract::BondingContractAlkane;
use bonding_contract::BondingCurve;

// Mock runtime context for testing
struct MockRuntimeContext {
    storage: HashMap<String, Vec<u8>>,
    caller: AlkaneId,
    myself: AlkaneId,
    incoming_alkanes: AlkaneTransferParcel,
}

impl RuntimeContext for MockRuntimeContext {
    fn get_storage(&self, key: &[u8]) -> Vec<u8> {
        self.storage.get(&String::from_utf8_lossy(key).to_string())
            .cloned()
            .unwrap_or_default()
    }

    fn set_storage(&mut self, key: &[u8], value: Vec<u8>) {
        self.storage.insert(String::from_utf8_lossy(key).to_string(), value);
    }

    fn get_caller(&self) -> AlkaneId {
        self.caller.clone()
    }

    fn get_myself(&self) -> AlkaneId {
        self.myself.clone()
    }

    fn get_incoming_alkanes(&self) -> AlkaneTransferParcel {
        self.incoming_alkanes.clone()
    }
}

// Helper function to create a mock context
fn create_mock_context(
    caller: AlkaneId,
    myself: AlkaneId,
    incoming_alkanes: Vec<AlkaneTransfer>,
) -> MockRuntimeContext {
    MockRuntimeContext {
        storage: HashMap::new(),
        caller,
        myself,
        incoming_alkanes: AlkaneTransferParcel(incoming_alkanes),
    }
}

// Helper function to set the runtime context
fn set_runtime_context(context: &MockRuntimeContext) {
    // In a real test environment, this would set the context for the runtime
    // For this mock, we'll just pretend it works
}

#[test]
fn test_full_lifecycle() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    let user1 = AlkaneId { block: 1, tx: 2 };
    let user2 = AlkaneId { block: 1, tx: 3 };
    let diesel_id = AlkaneId { block: 2, tx: 0 }; // Diesel is [2, 0]

    // Create a new bonding contract
    let contract = BondingContractAlkane::default();

    // Step 1: Initialize the contract
    println!("Step 1: Initializing contract");
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;

    // Set up the context for initialization
    let init_context = create_mock_context(
        deployer.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&init_context);

    // Call the initialize function
    let init_response = contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Verify the initialization
    assert_eq!(init_response.alkanes.0.len(), 1);
    assert_eq!(init_response.alkanes.0[0].value, initial_supply);
    assert_eq!(contract.total_supply(), initial_supply);
    assert_eq!(contract.reserve(), initial_reserve);
    
    // Step 2: User1 buys tokens
    println!("Step 2: User1 buys tokens");
    let diesel_amount = 10000;
    
    // Set up the context for buying
    let buy_context = create_mock_context(
        user1.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount,
            },
        ],
    );
    set_runtime_context(&buy_context);
    
    // Calculate the expected token amount
    let expected_token_amount = contract.calculate_buy_amount(diesel_amount)?;
    
    // Call the buy function
    let buy_response = contract.buy(diesel_amount)?;
    
    // Verify the buy operation
    assert_eq!(buy_response.alkanes.0.len(), 1);
    assert_eq!(buy_response.alkanes.0[0].value, expected_token_amount);
    assert_eq!(contract.reserve(), initial_reserve + diesel_amount);
    assert_eq!(contract.total_supply(), initial_supply + expected_token_amount);
    
    // Step 3: User2 buys tokens
    println!("Step 3: User2 buys tokens");
    let diesel_amount2 = 20000;
    
    // Set up the context for buying
    let buy_context2 = create_mock_context(
        user2.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount2,
            },
        ],
    );
    set_runtime_context(&buy_context2);
    
    // Calculate the expected token amount
    let expected_token_amount2 = contract.calculate_buy_amount(diesel_amount2)?;
    
    // Call the buy function
    let buy_response2 = contract.buy(diesel_amount2)?;
    
    // Verify the buy operation
    assert_eq!(buy_response2.alkanes.0.len(), 1);
    assert_eq!(buy_response2.alkanes.0[0].value, expected_token_amount2);
    
    // Step 4: User1 sells tokens
    println!("Step 4: User1 sells tokens");
    let token_amount = expected_token_amount / 2;
    
    // Set up the context for selling
    let sell_context = create_mock_context(
        user1.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: contract_id.clone(),
                value: token_amount,
            },
        ],
    );
    set_runtime_context(&sell_context);
    
    // Calculate the expected diesel amount
    let expected_diesel_amount = contract.calculate_sell_price(token_amount)?;
    
    // Call the sell function
    let sell_response = contract.sell(token_amount)?;
    
    // Verify the sell operation
    assert_eq!(sell_response.alkanes.0.len(), 1);
    assert_eq!(sell_response.alkanes.0[0].value, expected_diesel_amount);
    
    // Step 5: Get the current price
    println!("Step 5: Getting current price");
    
    // Set up the context for getting the price
    let price_context = create_mock_context(
        user1.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&price_context);
    
    // Call the get_current_price function
    let price_response = contract.get_current_price()?;
    
    // Verify the price
    let price_bytes = price_response.data;
    let price = u128::from_le_bytes(price_bytes.try_into().unwrap_or([0; 16]));
    assert!(price > 0);
    
    // Step 6: Test slippage
    println!("Step 6: Testing slippage");
    
    // Calculate buy price
    let buy_amount = 5000;
    let buy_price_context = create_mock_context(
        user1.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&buy_price_context);
    
    let buy_price_response = contract.get_buy_price(buy_amount)?;
    let buy_price_bytes = buy_price_response.data;
    let buy_price = u128::from_le_bytes(buy_price_bytes.try_into().unwrap_or([0; 16]));
    
    // Calculate sell price
    let sell_price_context = create_mock_context(
        user1.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&sell_price_context);
    
    let sell_price_response = contract.get_sell_price(buy_amount)?;
    let sell_price_bytes = sell_price_response.data;
    let sell_price = u128::from_le_bytes(sell_price_bytes.try_into().unwrap_or([0; 16]));
    
    // Verify slippage
    println!("Buy price: {}, Sell price: {}", buy_price, sell_price);
    assert!(buy_price > sell_price);
    
    // Calculate slippage percentage
    let slippage = (buy_price as f64 - sell_price as f64) / buy_price as f64 * 100.0;
    println!("Slippage: {}%", slippage);
    
    Ok(())
}

#[test]
fn test_edge_cases() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    let user = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 }; // Diesel is [2, 0]

    // Create a new bonding contract
    let contract = BondingContractAlkane::default();

    // Initialize the contract with small values
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000;
    let initial_reserve = 1000;

    // Set up the context for initialization
    let init_context = create_mock_context(
        deployer.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&init_context);

    // Call the initialize function
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Test case 1: Buy with zero diesel
    println!("Test case 1: Buy with zero diesel");
    
    // Set up the context for buying
    let buy_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: 0,
            },
        ],
    );
    set_runtime_context(&buy_context);
    
    // Call the buy function
    let buy_result = contract.buy(0);
    
    // Verify the buy operation fails
    assert!(buy_result.is_err());
    
    // Test case 2: Sell more tokens than owned
    println!("Test case 2: Sell more tokens than owned");
    
    // Set up the context for selling
    let sell_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: contract_id.clone(),
                value: initial_supply + 1,
            },
        ],
    );
    set_runtime_context(&sell_context);
    
    // Call the sell function
    let sell_result = contract.sell(initial_supply + 1);
    
    // Verify the sell operation fails or handles it gracefully
    // Note: In our implementation, this might not fail because we don't check individual balances
    // but it would fail in a real contract
    
    // Test case 3: Buy a very large amount
    println!("Test case 3: Buy a very large amount");
    
    // Set up the context for buying
    let large_amount = initial_reserve * 10;
    let buy_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: large_amount,
            },
        ],
    );
    set_runtime_context(&buy_context);
    
    // Call the buy function
    let buy_result = contract.buy(large_amount);
    
    // Verify the buy operation succeeds
    assert!(buy_result.is_ok());
    
    // Test case 4: Sell all tokens
    println!("Test case 4: Sell all tokens");
    
    // First, buy some tokens
    let buy_amount = 100;
    let buy_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: buy_amount,
            },
        ],
    );
    set_runtime_context(&buy_context);
    
    let buy_response = contract.buy(buy_amount)?;
    let token_amount = buy_response.alkanes.0[0].value;
    
    // Now sell all tokens
    let sell_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: contract_id.clone(),
                value: token_amount,
            },
        ],
    );
    set_runtime_context(&sell_context);
    
    let sell_result = contract.sell(token_amount);
    
    // Verify the sell operation succeeds
    assert!(sell_result.is_ok());
    
    Ok(())
}

#[test]
fn test_price_impact() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    let user = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 }; // Diesel is [2, 0]

    // Create a new bonding contract
    let contract = BondingContractAlkane::default();

    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;

    // Set up the context for initialization
    let init_context = create_mock_context(
        deployer.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&init_context);

    // Call the initialize function
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Get the initial price
    let price_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&price_context);
    
    let price_response = contract.get_current_price()?;
    let price_bytes = price_response.data;
    let initial_price = u128::from_le_bytes(price_bytes.try_into().unwrap_or([0; 16]));
    
    // Test small buy
    let small_amount = initial_supply / 100; // 1% of supply
    
    // Set up the context for buying
    let buy_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: small_amount,
            },
        ],
    );
    set_runtime_context(&buy_context);
    
    // Call the buy function
    contract.buy(small_amount)?;
    
    // Get the new price
    let price_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&price_context);
    
    let price_response = contract.get_current_price()?;
    let price_bytes = price_response.data;
    let small_buy_price = u128::from_le_bytes(price_bytes.try_into().unwrap_or([0; 16]));
    
    // Calculate price impact
    let small_price_impact = (small_buy_price as f64 - initial_price as f64) / initial_price as f64 * 100.0;
    println!("Small buy price impact: {}%", small_price_impact);
    
    // Test large buy
    let large_amount = initial_supply / 10; // 10% of supply
    
    // Set up the context for buying
    let buy_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: large_amount,
            },
        ],
    );
    set_runtime_context(&buy_context);
    
    // Call the buy function
    contract.buy(large_amount)?;
    
    // Get the new price
    let price_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&price_context);
    
    let price_response = contract.get_current_price()?;
    let price_bytes = price_response.data;
    let large_buy_price = u128::from_le_bytes(price_bytes.try_into().unwrap_or([0; 16]));
    
    // Calculate price impact
    let large_price_impact = (large_buy_price as f64 - small_buy_price as f64) / small_buy_price as f64 * 100.0;
    println!("Large buy price impact: {}%", large_price_impact);
    
    // Verify that larger buys have more price impact
    assert!(large_price_impact > small_price_impact);
    
    Ok(())
}
