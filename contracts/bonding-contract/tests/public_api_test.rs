use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::context::Context;
use std::time::{SystemTime, UNIX_EPOCH};

use bonding_contract::{BondingContractAlkane, BondContract, AlkaneIdExt, BondingContract};
use bonding_contract::mock_runtime;
use bonding_contract::reset_mock_environment;

// Mock time function to control time in tests
fn set_mock_time(time: u64) {
    // This is just a placeholder - in a real implementation, we would need to
    // modify the contract to use this time instead of the system time
    println!("Setting mock time to: {}", time);
}

// Get the current mock time
fn get_mock_time() -> u64 {
    // For now, just return the current system time
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[test]
fn test_contract_initialization() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Set initial time (timestamp in seconds)
    let initial_time = 1000000;
    set_mock_time(initial_time);
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 1 };
    let myself = AlkaneId { block: 1, tx: 2 };
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Initialize the contract with bond functionality
    let name = u128::from_le_bytes(*b"BondToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"BOND\0\0\0\0\0\0\0\0\0\0\0\0");
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 500_000;
    let half_life = 3600; // 1 hour
    let level_bips = 5000; // 50%
    let term = 86400; // 24 hours (1 day)
    
    // Manually set up the contract state using public methods
    contract.set_name(name);
    contract.set_symbol(symbol);
    contract.set_virtual_input_reserves_internal(virtual_input_reserves);
    contract.set_virtual_output_reserves_internal(virtual_output_reserves);
    contract.set_half_life_internal(half_life);
    contract.set_level_bips_internal(level_bips);
    contract.set_term(term);
    contract.set_last_update_internal(contract.get_current_timestamp());
    contract.set_total_debt(0);
    contract.set_owner(caller.into_u128());
    contract.set_paused(false);
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // Verify initial contract state
    assert_eq!(contract.name(), "BondToken", "Name should be set correctly");
    assert_eq!(contract.symbol(), "BOND", "Symbol should be set correctly");
    assert_eq!(contract.virtual_input_reserves(), virtual_input_reserves, "Virtual input reserves should be set correctly");
    assert_eq!(contract.virtual_output_reserves(), virtual_output_reserves, "Virtual output reserves should be set correctly");
    assert_eq!(contract.half_life(), half_life, "Half life should be set correctly");
    assert_eq!(contract.level_bips(), level_bips, "Level bips should be set correctly");
    assert_eq!(contract.term(), term, "Term should be set correctly");
    assert_eq!(contract.total_debt(), 0, "Total debt should be initialized to 0");
    assert_eq!(contract.owner(), caller.into_u128(), "Owner should be set to the caller");
    assert!(!contract.is_paused(), "Contract should be unpaused");
    
    println!("\nContract initialization test completed successfully!");
}

#[test]
fn test_contract_pricing() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 1 };
    let myself = AlkaneId { block: 1, tx: 2 };
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Initialize the contract with bond functionality
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 500_000;
    
    // Set up the contract state
    contract.set_virtual_input_reserves_internal(virtual_input_reserves);
    contract.set_virtual_output_reserves_internal(virtual_output_reserves);
    contract.set_alkane_supply(1_000_000);
    
    // Test the current price using the trait method
    let current_price_result = <BondingContractAlkane as BondingContract>::current_price(&contract);
    println!("Current price result: {:?}", current_price_result);
    assert!(current_price_result.is_ok(), "Current price should return Ok");
    
    // Test the bond curve
    let bond_curve = contract.get_bond_curve();
    println!("Bond curve pricing: {:?}", bond_curve.pricing);
    println!("Bond curve total_debt: {}", bond_curve.total_debt);
    println!("Bond curve term: {}", bond_curve.term);
    assert!(bond_curve.term > 0, "Bond curve term should be positive");
    
    println!("\nContract pricing test completed successfully!");
}
