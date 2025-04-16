use bonding_contract::{BondingContractAlkane, BondingContract, BondContract, Bond};
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransferParcel as AlkaneTransfers;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::context::Context;
use alkanes_runtime::storage::StoragePointer;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn test_simple() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_sell_alkane() {
    // This is just a placeholder test to verify that the test framework is working
    assert!(true);
}
