use alkanes_runtime::runtime::{AlkaneResponder, Context};
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use anyhow::Result;

// Import the bonding contract
use bonding_contract::BondingContractAlkane;
use bonding_contract::BondingCurve;

#[test]
fn test_initialize() -> Result<()> {
    // Create a new bonding contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;
    
    // Mock the context
    // In a real test, you would use a test context provider
    
    // Call the initialize function
    let response = contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Check the response
    assert_eq!(response.alkanes.0.len(), 1);
    assert_eq!(response.alkanes.0[0].value, initial_supply);
    
    // Check the contract state
    assert_eq!(contract.total_supply(), initial_supply);
    assert_eq!(contract.reserve(), initial_reserve);
    
    Ok(())
}

#[test]
fn test_buy() -> Result<()> {
    // Create a new bonding contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Buy tokens with diesel
    let diesel_amount = 1000;
    
    // Mock the context with incoming diesel
    // In a real test, you would use a test context provider
    
    // Calculate the expected token amount
    let expected_token_amount = contract.calculate_buy_amount(diesel_amount)?;
    
    // Call the buy function
    let response = contract.buy(diesel_amount)?;
    
    // Check the response
    assert_eq!(response.alkanes.0.len(), 1);
    assert_eq!(response.alkanes.0[0].value, expected_token_amount);
    
    // Check the contract state
    assert_eq!(contract.reserve(), initial_reserve + diesel_amount);
    assert_eq!(contract.total_supply(), initial_supply + expected_token_amount);
    
    Ok(())
}

#[test]
fn test_sell() -> Result<()> {
    // Create a new bonding contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Sell tokens
    let token_amount = 1000;
    
    // Mock the context with incoming tokens
    // In a real test, you would use a test context provider
    
    // Calculate the expected diesel amount
    let expected_diesel_amount = contract.calculate_sell_price(token_amount)?;
    
    // Call the sell function
    let response = contract.sell(token_amount)?;
    
    // Check the response
    assert_eq!(response.alkanes.0.len(), 1);
    assert_eq!(response.alkanes.0[0].value, expected_diesel_amount);
    
    // Check the contract state
    assert_eq!(contract.reserve(), initial_reserve - expected_diesel_amount);
    assert_eq!(contract.total_supply(), initial_supply - token_amount);
    
    Ok(())
}

#[test]
fn test_bonding_curve() -> Result<()> {
    // Create a new bonding contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Test the bonding curve
    // As more tokens are minted, the price should increase
    let amount1 = 1000;
    let amount2 = 2000;
    
    let price1 = contract.calculate_buy_price(amount1)?;
    let price2 = contract.calculate_buy_price(amount2)?;
    
    // Price should be higher for a larger amount
    assert!(price2 > price1);
    
    // Test buying and selling
    // Buy some tokens
    let diesel_amount = 1000;
    let token_amount = contract.calculate_buy_amount(diesel_amount)?;
    
    // Sell the tokens
    let diesel_received = contract.calculate_sell_price(token_amount)?;
    
    // There should be some slippage (diesel_received < diesel_amount)
    assert!(diesel_received < diesel_amount);
    
    Ok(())
}
