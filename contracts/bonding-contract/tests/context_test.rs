use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::context::Context;
use bonding_contract::mock_runtime;
use bonding_contract::reset_mock_environment;
use bonding_contract::BondingContractAlkane;
use alkanes_runtime::runtime::AlkaneResponder;

#[test]
fn test_context_availability() {
    // Reset the mock environment
    reset_mock_environment::reset();
    
    // Create a new bonding contract
    let contract = BondingContractAlkane::default();
    
    // Set up the mock context
    let caller = AlkaneId { block: 1, tx: 1 };
    let myself = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 };
    
    let context = Context {
        caller: caller.clone(),
        myself: myself.clone(),
        incoming_alkanes: AlkaneTransfers(vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: 100_000,
            },
        ]),
        vout: 0,
        inputs: vec![],
    };
    
    // Set the context
    mock_runtime::set_mock_context(context.clone());
    
    // Try to get the context
    let result = contract.context();
    
    // Print the result
    println!("Context result: {:?}", result);
    
    // Assert that the context is available
    assert!(result.is_ok(), "Context should be available");
    
    // Verify the context values
    let retrieved_context = result.unwrap();
    assert_eq!(retrieved_context.caller.block, caller.block);
    assert_eq!(retrieved_context.caller.tx, caller.tx);
    assert_eq!(retrieved_context.myself.block, myself.block);
    assert_eq!(retrieved_context.myself.tx, myself.tx);
    assert_eq!(retrieved_context.incoming_alkanes.0.len(), 1);
    assert_eq!(retrieved_context.incoming_alkanes.0[0].id.block, diesel_id.block);
    assert_eq!(retrieved_context.incoming_alkanes.0[0].id.tx, diesel_id.tx);
    assert_eq!(retrieved_context.incoming_alkanes.0[0].value, 100_000);
}
