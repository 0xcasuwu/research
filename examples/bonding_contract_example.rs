use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};

// Import the bonding contract
use bonding_contract::BondingContractAlkane;
use bonding_contract::BondingCurve;

fn main() -> Result<()> {
    println!("Bonding Contract Example");
    
    // Create a new bonding contract
    let contract = BondingContractAlkane::default();
    
    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;
    
    println!("Initializing contract with:");
    println!("  Name: BOND");
    println!("  Symbol: BND");
    println!("  Initial Supply: {}", initial_supply);
    println!("  Initial Reserve: {}", initial_reserve);
    
    let response = contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    println!("Contract initialized successfully");
    
    // Get the current price
    let current_price = contract.current_price()?;
    println!("Current price: {}", current_price);
    
    // Buy tokens with diesel
    let diesel_amount = 10000;
    println!("\nBuying tokens with {} diesel", diesel_amount);
    
    // Calculate the expected token amount
    let expected_token_amount = contract.calculate_buy_amount(diesel_amount)?;
    println!("Expected token amount: {}", expected_token_amount);
    
    // Call the buy function
    let response = contract.buy(diesel_amount)?;
    println!("Buy successful");
    
    // Get the new price
    let new_price = contract.current_price()?;
    println!("New price: {}", new_price);
    println!("Price increased by: {}%", (new_price as f64 / current_price as f64 - 1.0) * 100.0);
    
    // Sell tokens
    let token_amount = expected_token_amount / 2;
    println!("\nSelling {} tokens", token_amount);
    
    // Calculate the expected diesel amount
    let expected_diesel_amount = contract.calculate_sell_price(token_amount)?;
    println!("Expected diesel amount: {}", expected_diesel_amount);
    
    // Call the sell function
    let response = contract.sell(token_amount)?;
    println!("Sell successful");
    
    // Get the final price
    let final_price = contract.current_price()?;
    println!("Final price: {}", final_price);
    
    // Calculate slippage
    let buy_price_per_token = diesel_amount as f64 / expected_token_amount as f64;
    let sell_price_per_token = expected_diesel_amount as f64 / token_amount as f64;
    let slippage = (1.0 - sell_price_per_token / buy_price_per_token) * 100.0;
    
    println!("\nSlippage Analysis:");
    println!("  Buy price per token: {:.6}", buy_price_per_token);
    println!("  Sell price per token: {:.6}", sell_price_per_token);
    println!("  Slippage: {:.2}%", slippage);
    
    // Show final contract state
    println!("\nFinal Contract State:");
    println!("  Total Supply: {}", contract.total_supply());
    println!("  Reserve: {}", contract.reserve());
    println!("  Price: {}", final_price);
    
    Ok(())
}
