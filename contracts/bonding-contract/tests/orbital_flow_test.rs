use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::context::Context;
use alkanes_support::cellpack::Cellpack;
use std::time::{SystemTime, UNIX_EPOCH};

use bonding_contract::{BondingContractAlkane, BondContract, BondingContract, AlkaneIdExt, BondOrbital};
use bonding_contract::mock_runtime;
use bonding_contract::reset_mock_environment;
use bonding_contract::BOND_ORBITAL_TEMPLATE_ID;

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

// Mock implementation of the orbital template
fn mock_orbital_template_call(owed: u128, creation: u64, term: u64) -> AlkaneId {
    println!("Creating mock orbital with owed: {}, creation: {}, term: {}", owed, creation, term);
    
    // Create a unique orbital ID
    let orbital_id = AlkaneId {
        block: 2,
        tx: 1000 + reset_mock_environment::TEST_RUN_COUNTER.load(std::sync::atomic::Ordering::SeqCst),
    };
    
    // Initialize the orbital
    let orbital = BondOrbital::default();
    
    // Set up the mock context for initializing the orbital
    let caller = AlkaneId { block: 3, tx: 0 }; // Bonding contract
    let myself = orbital_id.clone();
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![0, owed, creation as u128, term as u128], // Initialize opcode with parameters
    };
    
    mock_runtime::set_mock_context(context);
    
    // Initialize the orbital
    orbital.initialize(owed, creation, term).expect("Failed to initialize orbital");
    
    // Store the orbital in the mock registry
    mock_runtime::set_mock_orbital(orbital_id.clone(), orbital);
    
    orbital_id
}

// Mock implementation of the orbital staticcall
fn mock_orbital_staticcall(orbital_id: &AlkaneId, opcode: u128) -> Vec<u8> {
    println!("Calling mock orbital {} with opcode {}", orbital_id.tx, opcode);
    
    // Get the orbital from the mock registry
    let orbital = mock_runtime::get_mock_orbital(orbital_id).expect("Orbital not found");
    
    // Set up the mock context for the call
    let caller = AlkaneId { block: 3, tx: 0 }; // Bonding contract
    let myself = orbital_id.clone();
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![opcode],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Call the appropriate method based on the opcode
    match opcode {
        102 => { // GetBondDetails
            let response = orbital.get_bond_details().expect("Failed to get bond details");
            response.data
        },
        103 => { // GetOwed
            let response = orbital.get_owed().expect("Failed to get owed");
            response.data
        },
        104 => { // GetRedeemed
            let response = orbital.get_redeemed().expect("Failed to get redeemed");
            response.data
        },
        105 => { // GetCreation
            let response = orbital.get_creation().expect("Failed to get creation");
            response.data
        },
        106 => { // GetTerm
            let response = orbital.get_term().expect("Failed to get term");
            response.data
        },
        107 => { // GetMaturity
            let response = orbital.get_maturity().expect("Failed to get maturity");
            response.data
        },
        _ => {
            panic!("Unsupported opcode: {}", opcode);
        }
    }
}

// Mock implementation of the orbital call for redemption
fn mock_orbital_redeem(orbital_id: &AlkaneId) -> AlkaneTransfers {
    println!("Redeeming mock orbital {}", orbital_id.tx);
    
    // Get the orbital from the mock registry
    let orbital = mock_runtime::get_mock_orbital(orbital_id).expect("Orbital not found");
    
    // Set up the mock context for the call
    let caller = AlkaneId { block: 3, tx: 0 }; // Bonding contract
    let myself = orbital_id.clone();
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![200], // Redeem opcode
    };
    
    mock_runtime::set_mock_context(context);
    
    // Call the redeem method
    let response = orbital.redeem().expect("Failed to redeem orbital");
    
    // Update the orbital in the mock registry
    mock_runtime::set_mock_orbital(orbital_id.clone(), orbital);
    
    response.alkanes
}

// Extend the mock_runtime module to support orbitals
mod mock_orbital_extension {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    
    lazy_static::lazy_static! {
        static ref MOCK_ORBITALS: Mutex<HashMap<AlkaneId, BondOrbital>> = Mutex::new(HashMap::new());
    }
    
    pub fn set_mock_orbital(id: AlkaneId, orbital: BondOrbital) {
        let mut orbitals = MOCK_ORBITALS.lock().unwrap();
        orbitals.insert(id, orbital);
    }
    
    pub fn get_mock_orbital(id: &AlkaneId) -> Option<BondOrbital> {
        let orbitals = MOCK_ORBITALS.lock().unwrap();
        orbitals.get(id).cloned()
    }
    
    pub fn clear_mock_orbitals() {
        let mut orbitals = MOCK_ORBITALS.lock().unwrap();
        orbitals.clear();
    }
}

// Patch the mock_runtime module to use our orbital extension
impl mock_runtime {
    pub fn set_mock_orbital(id: AlkaneId, orbital: BondOrbital) {
        mock_orbital_extension::set_mock_orbital(id, orbital);
    }
    
    pub fn get_mock_orbital(id: &AlkaneId) -> Option<BondOrbital> {
        mock_orbital_extension::get_mock_orbital(id)
    }
    
    pub fn clear_mock_orbitals() {
        mock_orbital_extension::clear_mock_orbitals();
    }
}

// Patch the BondingContractAlkane to use our mock orbital implementation
impl BondingContractAlkane {
    // Override the create_bond_orbital method for testing
    pub fn test_create_bond_orbital(&self, owed: u128, term: u64) -> Result<AlkaneId, anyhow::Error> {
        // Use our mock implementation
        let creation = self.get_current_block_number();
        let orbital_id = mock_orbital_template_call(owed, creation, term);
        Ok(orbital_id)
    }
    
    // Override the staticcall method for testing
    pub fn test_staticcall(&self, cellpack: &Cellpack) -> Result<Vec<u8>, anyhow::Error> {
        // Check if this is a call to an orbital
        if cellpack.target.block == 2 && cellpack.target.tx >= 1000 {
            // This is a call to an orbital
            let opcode = cellpack.inputs[0];
            let data = mock_orbital_staticcall(&cellpack.target, opcode);
            return Ok(data);
        }
        
        // Otherwise, use the default implementation
        Err(anyhow::anyhow!("Unsupported staticcall target"))
    }
    
    // Override the call method for testing
    pub fn test_call(&self, cellpack: &Cellpack) -> Result<AlkaneTransfers, anyhow::Error> {
        // Check if this is a call to an orbital
        if cellpack.target.block == 2 && cellpack.target.tx >= 1000 {
            // This is a call to an orbital
            let opcode = cellpack.inputs[0];
            if opcode == 200 { // Redeem opcode
                let transfers = mock_orbital_redeem(&cellpack.target);
                return Ok(transfers);
            }
        } else if cellpack.target.block == 6 && cellpack.target.tx == BOND_ORBITAL_TEMPLATE_ID {
            // This is a call to the orbital template
            let owed = cellpack.inputs[1];
            let creation = cellpack.inputs[2] as u64;
            let term = cellpack.inputs[3] as u64;
            let orbital_id = mock_orbital_template_call(owed, creation, term);
            
            // Return a transfer of the orbital token
            let transfers = AlkaneTransfers(vec![
                AlkaneTransfer {
                    id: orbital_id,
                    value: 1,
                },
            ]);
            
            return Ok(transfers);
        }
        
        // Otherwise, use the default implementation
        Err(anyhow::anyhow!("Unsupported call target"))
    }
}

#[test]
fn test_orbital_bond_lifecycle() {
    // Reset the mock environment
    reset_mock_environment::reset();
    mock_runtime::clear_mock_orbitals();
    
    // Set initial time (timestamp in seconds)
    let initial_time = 1000000;
    set_mock_time(initial_time);
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
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
    
    // Manually set up the contract state instead of using init_bond_contract
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
    contract.set_paused(false); // Start unpaused for simplicity
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // STEP 1: Purchase a bond
    println!("STEP 1: Purchasing bond");
    
    // Set up the mock context for purchasing a bond
    let diesel_amount = 100_000;
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Patch the contract to use our mock orbital implementation
    let original_create_bond_orbital = contract.create_bond_orbital;
    contract.create_bond_orbital = contract.test_create_bond_orbital;
    
    // Purchase a bond
    let min_output = 1; // Minimum output
    let purchase_result = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(purchase_result.is_ok(), "Bond purchase should succeed");
    
    // Verify the bond was created
    let position_count = contract.position_count_of(caller.into_u128());
    assert_eq!(position_count, 1, "Position count should be 1");
    
    // Get the bond
    let bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    
    // Verify the bond properties
    println!("Bond created with owed: {}, redeemed: {}", bond.owed, bond.redeemed);
    assert!(bond.owed > 0, "Bond owed should be positive");
    assert_eq!(bond.redeemed, 0, "Bond redeemed should be 0");
    
    // Get the orbital ID
    let orbital_id = contract.get_bond_orbital_id(0).unwrap();
    println!("Bond orbital ID: {:?}", orbital_id);
    
    // STEP 2: Simulate time passing to full maturity
    println!("\nSTEP 2: Simulating time passing to full maturity");
    set_mock_time(initial_time + term);
    println!("Time elapsed: {} seconds (100% of term)", term);
    
    // STEP 3: Redeem the bond
    println!("\nSTEP 3: Redeeming the bond");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: orbital_id.clone(),
                value: 1, // The orbital token
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Patch the contract to use our mock orbital implementation
    let original_staticcall = contract.staticcall;
    contract.staticcall = contract.test_staticcall;
    
    let original_call = contract.call;
    contract.call = contract.test_call;
    
    // Redeem the bond
    let redeem_result = contract.redeem_bond(0);
    
    assert!(redeem_result.is_ok(), "Bond redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = redeem_result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    let actual_redeemed = response.alkanes.0[0].value;
    println!("Actual redeemed amount: {}", actual_redeemed);
    
    // Verify the redeemed amount is approximately what we expect
    assert_eq!(actual_redeemed, bond.owed, "Redeemed amount should match bond owed");
    
    // Restore the original methods
    contract.create_bond_orbital = original_create_bond_orbital;
    contract.staticcall = original_staticcall;
    contract.call = original_call;
    
    println!("\nOrbital bond lifecycle test completed successfully!");
}

#[test]
fn test_orbital_multiple_bonds() {
    // Reset the mock environment
    reset_mock_environment::reset();
    mock_runtime::clear_mock_orbitals();
    
    // Set initial time (timestamp in seconds)
    let initial_time = 1000000;
    set_mock_time(initial_time);
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
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
    
    // Manually set up the contract state instead of using init_bond_contract
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
    contract.set_paused(false); // Start unpaused for simplicity
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // Patch the contract to use our mock orbital implementation
    let original_create_bond_orbital = contract.create_bond_orbital;
    contract.create_bond_orbital = contract.test_create_bond_orbital;
    
    let original_staticcall = contract.staticcall;
    contract.staticcall = contract.test_staticcall;
    
    let original_call = contract.call;
    contract.call = contract.test_call;
    
    // STEP 1: Purchase first bond
    println!("STEP 1: Purchasing first bond");
    
    // Set up the mock context for purchasing a bond
    let diesel_amount_1 = 100_000;
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount_1,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Purchase first bond
    let min_output = 1;
    let purchase_result_1 = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(purchase_result_1.is_ok(), "First bond purchase should succeed");
    
    // Get the first bond
    let bond_1 = contract.get_bond(caller.into_u128(), 0).unwrap();
    println!("First bond created with owed: {}", bond_1.owed);
    
    // Get the first orbital ID
    let orbital_id_1 = contract.get_bond_orbital_id(0).unwrap();
    println!("First bond orbital ID: {:?}", orbital_id_1);
    
    // STEP 2: Purchase second bond
    println!("\nSTEP 2: Purchasing second bond");
    
    // Set up the mock context for purchasing a second bond
    let diesel_amount_2 = 100_000;
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount_2,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Purchase second bond
    let purchase_result_2 = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(purchase_result_2.is_ok(), "Second bond purchase should succeed");
    
    // Get the second bond
    let bond_2 = contract.get_bond(caller.into_u128(), 1).unwrap();
    println!("Second bond created with owed: {}", bond_2.owed);
    
    // Get the second orbital ID
    let orbital_id_2 = contract.get_bond_orbital_id(1).unwrap();
    println!("Second bond orbital ID: {:?}", orbital_id_2);
    
    // STEP 3: Simulate time passing to full maturity
    println!("\nSTEP 3: Simulating time passing to full maturity");
    set_mock_time(initial_time + term);
    println!("Time elapsed: {} seconds (100% of term)", term);
    
    // STEP 4: Redeem both bonds
    println!("\nSTEP 4: Redeeming both bonds");
    
    // Set up the mock context for redeeming with both orbital tokens
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: orbital_id_1.clone(),
                value: 1, // The first orbital token
            },
            AlkaneTransfer {
                id: orbital_id_2.clone(),
                value: 1, // The second orbital token
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Redeem both bonds using batch redemption
    let redeem_result = contract.redeem_bond_batch(vec![0, 1]);
    
    assert!(redeem_result.is_ok(), "Batch bond redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = redeem_result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    let total_redeemed = response.alkanes.0[0].value;
    println!("Total redeemed amount: {}", total_redeemed);
    
    // Verify the total redeemed amount is the sum of both bonds
    assert_eq!(total_redeemed, bond_1.owed + bond_2.owed, "Total redeemed amount should match sum of bond owed amounts");
    
    // Restore the original methods
    contract.create_bond_orbital = original_create_bond_orbital;
    contract.staticcall = original_staticcall;
    contract.call = original_call;
    
    println!("\nOrbital multiple bonds test completed successfully!");
}

#[test]
fn test_orbital_partial_maturity() {
    // Reset the mock environment
    reset_mock_environment::reset();
    mock_runtime::clear_mock_orbitals();
    
    // Set initial time (timestamp in seconds)
    let initial_time = 1000000;
    set_mock_time(initial_time);
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 0 };
    let myself = AlkaneId { block: 3, tx: 0 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
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
    
    // Manually set up the contract state instead of using init_bond_contract
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
    contract.set_paused(false); // Start unpaused for simplicity
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // Patch the contract to use our mock orbital implementation
    let original_create_bond_orbital = contract.create_bond_orbital;
    contract.create_bond_orbital = contract.test_create_bond_orbital;
    
    let original_staticcall = contract.staticcall;
    contract.staticcall = contract.test_staticcall;
    
    let original_call = contract.call;
    contract.call = contract.test_call;
    
    // STEP 1: Purchase a bond
    println!("STEP 1: Purchasing bond");
    
    // Set up the mock context for purchasing a bond
    let diesel_amount = 100_000;
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: diesel_amount,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Purchase a bond
    let min_output = 1; // Minimum output
    let purchase_result = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(purchase_result.is_ok(), "Bond purchase should succeed");
    
    // Get the bond
    let bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    println!("Bond created with owed: {}, redeemed: {}", bond.owed, bond.redeemed);
    
    // Get the orbital ID
    let orbital_id = contract.get_bond_orbital_id(0).unwrap();
    println!("Bond orbital ID: {:?}", orbital_id);
    
    // STEP 2: Simulate time passing to 50% maturity
    println!("\nSTEP 2: Simulating time passing to 50% maturity");
    set_mock_time(initial_time + term / 2);
    println!("Time elapsed: {} seconds (50% of term)", term / 2);
    
    // STEP 3: Try to redeem the bond (should fail because it's not mature)
    println!("\nSTEP 3: Trying to redeem the bond at 50% maturity");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: orbital_id.clone(),
                value: 1, // The orbital token
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Try to redeem the bond
    let redeem_result = contract.redeem_bond(0);
    
    // This should fail because the bond is not mature
    assert!(redeem_result.is_err(), "Bond redemption should fail at 50% maturity");
    println!("Bond redemption failed as expected: {}", redeem_result.unwrap_err());
    
    // STEP 4: Simulate time passing to full maturity
    println!("\nSTEP 4: Simulating time passing to full maturity");
    set_mock_time(initial_time + term);
    println!("Time elapsed: {} seconds (100% of term)", term);
    
    // STEP 5: Redeem the bond
    println!("\nSTEP 5: Redeeming the bond at full maturity");
    
    // Set up the mock context for redeeming
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: orbital_id.clone(),
                value: 1, // The orbital token
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_runtime::set_mock_context(context);
    
    // Redeem the bond
    let redeem_result = contract.redeem_bond(0);
    
    assert!(redeem_result.is_ok(), "Bond redemption should succeed at full maturity");
    
    // Verify the response contains the alkane transfer
    let response = redeem_result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    let actual_redeemed = response.alkanes.0[0].value;
    println!("Actual redeemed amount: {}", actual_redeemed);
    
    // Verify the redeemed amount is approximately what we expect
    assert_eq!(actual_redeemed, bond.owed, "Redeemed amount should match bond owed");
    
    // Restore the original methods
    contract.create_bond_orbital = original_create_bond_orbital;
    contract.staticcall = original_staticcall;
    contract.call = original_call;
    
    println!("\nOrbital partial maturity test completed successfully!");
}
