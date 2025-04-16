use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::context::Context;
use anyhow::Result;
use metashrew_support::index_pointer::KeyValuePointer;

use bonding_contract::{BondingContractAlkane, BondContract, AlkaneIdExt};
use bonding_contract::mock_context;
use bonding_contract::mock_storage;
use bonding_contract::reset_mock_environment;

#[test]
fn test_bond_curve_initialization() {
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
    
    mock_context::set_mock_context(context);
    
    // Initialize the contract with bond functionality
    let name = u128::from_le_bytes(*b"BondToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"BOND\0\0\0\0\0\0\0\0\0\0\0\0");
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 500_000;
    let half_life = 3600; // 1 hour
    let level_bips = 5000; // 50%
    let term = 86400; // 24 hours
    
    let result = contract.init_bond_contract(
        name,
        symbol,
        virtual_input_reserves,
        virtual_output_reserves,
        half_life,
        level_bips,
        term
    );
    
    assert!(result.is_ok(), "Contract initialization should succeed");
    
    // Verify the contract state
    assert_eq!(contract.name(), "BondToken", "Name should be set correctly");
    assert_eq!(contract.symbol(), "BOND", "Symbol should be set correctly");
    assert_eq!(contract.virtual_input_reserves(), virtual_input_reserves, "Virtual input reserves should be set correctly");
    assert_eq!(contract.virtual_output_reserves(), virtual_output_reserves, "Virtual output reserves should be set correctly");
    assert_eq!(contract.half_life(), half_life, "Half life should be set correctly");
    assert_eq!(contract.level_bips(), level_bips, "Level bips should be set correctly");
    assert_eq!(contract.term(), term, "Term should be set correctly");
    assert_eq!(contract.total_debt(), 0, "Total debt should be initialized to 0");
    assert_eq!(contract.owner(), caller.into_u128(), "Owner should be set to the caller");
    assert!(contract.is_paused(), "Contract should be paused initially");
}

#[test]
fn test_bond_purchase_and_redemption() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 1 };
    let myself = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_context::set_mock_context(context);
    
    // Initialize the contract with bond functionality
    let name = u128::from_le_bytes(*b"BondToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"BOND\0\0\0\0\0\0\0\0\0\0\0\0");
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 500_000;
    let half_life = 3600; // 1 hour
    let level_bips = 5000; // 50%
    let term = 86400; // 24 hours
    
    let result = contract.init_bond_contract(
        name,
        symbol,
        virtual_input_reserves,
        virtual_output_reserves,
        half_life,
        level_bips,
        term
    );
    
    assert!(result.is_ok(), "Contract initialization should succeed");
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // Unpause the contract
    contract.set_paused(false);
    
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
    
    mock_context::set_mock_context(context);
    
    // Purchase a bond
    let min_output = 1; // Minimum output
    let result = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(result.is_ok(), "Bond purchase should succeed");
    
    // Verify the bond was created
    let position_count = contract.position_count_of(caller.into_u128());
    assert_eq!(position_count, 1, "Position count should be 1");
    
    // Get the bond
    let bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    
    // Verify the bond properties
    assert!(bond.owed > 0, "Bond owed should be positive");
    assert_eq!(bond.redeemed, 0, "Bond redeemed should be 0");
    
    // Verify the total debt was updated
    assert_eq!(contract.total_debt(), bond.owed, "Total debt should match bond owed");
    
    // Verify the virtual input reserves were updated
    assert_eq!(contract.virtual_input_reserves(), virtual_input_reserves + diesel_amount, "Virtual input reserves should be updated");
    
    // Set up the mock context for redeeming the bond
    // We need to simulate time passing for the bond to mature
    // For simplicity, we'll modify the bond's creation timestamp directly
    let bond_pointer = contract.bonds_pointer(caller.into_u128()).select(&0u128.to_le_bytes().to_vec());
    bond_pointer.select(&b"creation".to_vec()).set_value::<u64>(0); // Set creation time to 0 (fully matured)
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_context::set_mock_context(context);
    
    // Redeem the bond
    let result = contract.redeem_bond(0);
    
    assert!(result.is_ok(), "Bond redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    assert_eq!(response.alkanes.0[0].value, bond.owed, "Transfer value should match bond owed");
    
    // Verify the bond was updated
    let updated_bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    assert_eq!(updated_bond.redeemed, bond.owed, "Bond redeemed should match owed");
    
    // Verify the total debt was updated
    assert_eq!(contract.total_debt(), 0, "Total debt should be 0");
}

#[test]
fn test_bond_transfer() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 1 };
    let recipient = AlkaneId { block: 1, tx: 3 };
    let myself = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_context::set_mock_context(context);
    
    // Initialize the contract with bond functionality
    let name = u128::from_le_bytes(*b"BondToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"BOND\0\0\0\0\0\0\0\0\0\0\0\0");
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 500_000;
    let half_life = 3600; // 1 hour
    let level_bips = 5000; // 50%
    let term = 86400; // 24 hours
    
    let result = contract.init_bond_contract(
        name,
        symbol,
        virtual_input_reserves,
        virtual_output_reserves,
        half_life,
        level_bips,
        term
    );
    
    assert!(result.is_ok(), "Contract initialization should succeed");
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // Unpause the contract
    contract.set_paused(false);
    
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
    
    mock_context::set_mock_context(context);
    
    // Purchase a bond
    let min_output = 1; // Minimum output
    let result = contract.purchase_bond(caller.into_u128(), min_output);
    
    assert!(result.is_ok(), "Bond purchase should succeed");
    
    // Verify the bond was created
    let position_count = contract.position_count_of(caller.into_u128());
    assert_eq!(position_count, 1, "Position count should be 1");
    
    // Get the bond
    let bond = contract.get_bond(caller.into_u128(), 0).unwrap();
    
    // Set up the mock context for transferring the bond
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_context::set_mock_context(context);
    
    // Transfer the bond
    let result = contract.transfer_bond(recipient.into_u128(), 0);
    
    assert!(result.is_ok(), "Bond transfer should succeed");
    
    // Verify the bond was transferred
    let caller_position_count = contract.position_count_of(caller.into_u128());
    assert_eq!(caller_position_count, 0, "Caller position count should be 0");
    
    let recipient_position_count = contract.position_count_of(recipient.into_u128());
    assert_eq!(recipient_position_count, 1, "Recipient position count should be 1");
    
    // Get the transferred bond
    let transferred_bond = contract.get_bond(recipient.into_u128(), 0).unwrap();
    
    // Verify the bond properties
    assert_eq!(transferred_bond.owed, bond.owed, "Bond owed should match");
    assert_eq!(transferred_bond.redeemed, bond.redeemed, "Bond redeemed should match");
    assert_eq!(transferred_bond.creation, bond.creation, "Bond creation should match");
}

#[test]
fn test_bond_batch_redemption() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Create a new bonding contract
    let mut contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 1 };
    let myself = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_context::set_mock_context(context);
    
    // Initialize the contract with bond functionality
    let name = u128::from_le_bytes(*b"BondToken\0\0\0\0\0\0\0");
    let symbol = u128::from_le_bytes(*b"BOND\0\0\0\0\0\0\0\0\0\0\0\0");
    let virtual_input_reserves = 1_000_000;
    let virtual_output_reserves = 500_000;
    let half_life = 3600; // 1 hour
    let level_bips = 5000; // 50%
    let term = 86400; // 24 hours
    
    let result = contract.init_bond_contract(
        name,
        symbol,
        virtual_input_reserves,
        virtual_output_reserves,
        half_life,
        level_bips,
        term
    );
    
    assert!(result.is_ok(), "Contract initialization should succeed");
    
    // Set initial alkane supply
    contract.set_alkane_supply(1_000_000);
    
    // Unpause the contract
    contract.set_paused(false);
    
    // Purchase multiple bonds
    let diesel_amounts = [100_000, 200_000, 300_000];
    let mut total_owed = 0;
    
    for &diesel_amount in &diesel_amounts {
        // Set up the mock context for purchasing a bond
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
        
        mock_context::set_mock_context(context);
        
        // Purchase a bond
        let min_output = 1; // Minimum output
        let result = contract.purchase_bond(caller.into_u128(), min_output);
        
        assert!(result.is_ok(), "Bond purchase should succeed");
        
        // Get the bond
        let bond_id = contract.position_count_of(caller.into_u128()) - 1;
        let bond = contract.get_bond(caller.into_u128(), bond_id).unwrap();
        
        total_owed += bond.owed;
    }
    
    // Verify the position count
    let position_count = contract.position_count_of(caller.into_u128());
    assert_eq!(position_count, diesel_amounts.len() as u128, "Position count should match number of bonds");
    
    // Verify the total debt
    assert_eq!(contract.total_debt(), total_owed, "Total debt should match sum of all bonds");
    
    // Make all bonds fully mature
    for i in 0..diesel_amounts.len() {
        let bond_pointer = contract.bonds_pointer(caller.into_u128()).select(&(i as u128).to_le_bytes().to_vec());
        bond_pointer.select(&b"creation".to_vec()).set_value::<u64>(0); // Set creation time to 0 (fully matured)
    }
    
    // Set up the mock context for redeeming the bonds
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![]),
        vout: 0,
        inputs: vec![],
    };
    
    mock_context::set_mock_context(context);
    
    // Redeem all bonds in batch
    let bond_ids = (0..diesel_amounts.len() as u128).collect();
    let result = contract.redeem_bond_batch(bond_ids);
    
    assert!(result.is_ok(), "Bond batch redemption should succeed");
    
    // Verify the response contains the alkane transfer
    let response = result.unwrap();
    assert_eq!(response.alkanes.0.len(), 1, "Response should contain one alkane transfer");
    assert_eq!(response.alkanes.0[0].id, myself, "Transfer ID should be the contract");
    assert_eq!(response.alkanes.0[0].value, total_owed, "Transfer value should match total owed");
    
    // Verify all bonds were updated
    for i in 0..diesel_amounts.len() {
        let bond = contract.get_bond(caller.into_u128(), i as u128).unwrap();
        assert_eq!(bond.redeemed, bond.owed, "Bond redeemed should match owed");
    }
    
    // Verify the total debt was updated
    assert_eq!(contract.total_debt(), 0, "Total debt should be 0");
}
