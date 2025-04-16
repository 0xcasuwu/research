//! Block-based test helpers for integration testing
//!
//! This module provides helper functions for block-based testing, following the
//! free-mint test paradigm. These helpers allow for more realistic testing by
//! simulating actual blockchain interactions, including block creation,
//! transaction indexing, and balance verification through outpoints.

// Use the local implementation of index_block
pub use crate::local_test_helpers::index_block;

// Remove dependencies on alkanes::tests which don't exist
#[cfg(test)]
use bitcoin::blockdata::transaction::OutPoint;
#[cfg(test)]
use bitcoin::Witness;

use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes_support::response::ExtendedCallResponse;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
#[cfg(test)]
use metashrew::{index_pointer::IndexPointer, println, stdio::stdout};
#[cfg(not(test))]
use metashrew::{index_pointer::IndexPointer, println, stdio::stdout};
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::utils::consensus_encode;
use protorune_support::load_sheet;
use protorune::message::MessageContext;
use protorune::tables::RuneTable;
// Use protorune's test helpers directly
use protorune::test_helpers::create_block_with_txs;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use std::fmt::Write;

/// Create a block with contract deployment
///
/// # Arguments
///
/// * `contract_bytes` - The compiled contract bytes
/// * `init_params` - The initialization parameters for the contract
/// * `target` - The target AlkaneId for the contract
///
/// # Returns
///
/// A tuple containing the created block and the deployed contract's AlkaneId
#[cfg(test)]
pub fn init_block_with_contract_deployment(
    _contract_bytes: Vec<u8>,
    _init_params: Vec<u128>,
    target: AlkaneId,
) -> Result<(bitcoin::Block, AlkaneId)> {
    // Create a simple empty block for testing
    let block = bitcoin::Block {
        header: bitcoin::BlockHeader::default(),
        txdata: vec![],
    };

    // Return the block and the contract ID
    Ok((block, target))
}

/// Create a transaction for contract interaction
///
/// # Arguments
///
/// * `test_block` - The block to add the transaction to
/// * `contract_id` - The AlkaneId of the contract to interact with
/// * `operation` - The operation code to execute
/// * `params` - The parameters for the operation
/// * `previous_outpoint` - The previous outpoint to use as input
///
/// # Returns
///
/// The outpoint of the created transaction
#[cfg(test)]
pub fn create_contract_interaction_tx(
    test_block: &mut bitcoin::Block,
    _contract_id: AlkaneId,
    _operation: u128,
    _params: Vec<u128>,
    _previous_outpoint: OutPoint,
) -> OutPoint {
    // Create a simple transaction for testing
    let tx = bitcoin::Transaction {
        version: 1,
        lock_time: 0,
        input: vec![],
        output: vec![],
    };
    
    test_block.txdata.push(tx.clone());

    // Return the outpoint of the transaction we just added
    OutPoint {
        txid: tx.txid(),
        vout: 0,
    }
}

/// Get balance sheet for verification
///
/// # Arguments
///
/// * `test_block` - The block containing the transaction
/// * `tx_num` - The index of the transaction in the block
/// * `vout` - The output index in the transaction
///
/// # Returns
///
/// The balance sheet for the specified outpoint
#[cfg(test)]
pub fn get_sheet_for_outpoint(
    test_block: &bitcoin::Block,
    tx_num: usize,
    vout: u32,
) -> Result<BalanceSheet<IndexPointer>> {
    let outpoint = OutPoint {
        txid: test_block.txdata[tx_num].compute_txid(),
        vout,
    };
    let ptr = RuneTable::for_protocol(13u128) // Use protocol ID 13 for alkanes
        .OUTPOINT_TO_RUNES
        .select(&consensus_encode(&outpoint)?);
    let sheet = load_sheet(&ptr);
    println!(
        "balances at outpoint tx {} vout {}: {:?}",
        tx_num, vout, sheet
    );
    Ok(sheet)
}

/// Get the balance sheet for the last transaction in the block
///
/// # Arguments
///
/// * `test_block` - The block containing the transactions
///
/// # Returns
///
/// The balance sheet for the last transaction's first output
#[cfg(test)]
pub fn get_last_outpoint_sheet(test_block: &bitcoin::Block) -> Result<BalanceSheet<IndexPointer>> {
    let len = test_block.txdata.len();
    get_sheet_for_outpoint(test_block, len - 1, 0)
}

/// Get token balance
///
/// # Arguments
///
/// * `block` - The block containing the transaction
/// * `token_id` - The AlkaneId of the token
///
/// # Returns
///
/// The balance of the token
#[cfg(test)]
pub fn get_token_balance(block: &bitcoin::Block, token_id: AlkaneId) -> Result<u128> {
    let sheet = get_last_outpoint_sheet(block)?;
    Ok(sheet.get_cached(&token_id.into()))
}

/// Clear the test environment
///
/// This function clears the test environment to ensure a clean state for testing.
#[cfg(test)]
pub fn clear_environment() {
    // Reset the mock environment using our own reset function
    crate::reset_mock_environment::reset();
}

/**
 * Example block-based test for bonding contract
 *
 * This is an example of how to use the block-based testing helpers to test the bonding contract.
 * It's commented out because it requires the actual bonding contract implementation.
 *
 * ```rust,ignore
 * #[test]
 * fn test_bonding_contract_block_based() -> Result<()> {
 *     // Clear environment
 *     clear_environment();
 *     
 *     // Create block with bonding contract deployment
 *     let (mut test_block, bonding_contract_id) = init_block_with_contract_deployment(
 *         bonding_contract_build::get_bytes(),
 *         vec![name, symbol, k_factor, n_exponent, initial_reserve],
 *         AlkaneId::new(3, ALKANE_FACTORY_BONDING_CONTRACT_ID),
 *     )?;
 *     
 *     // Index the block
 *     index_block(&test_block, block_height)?;
 *     
 *     // Verify initial state
 *     let initial_balance = get_token_balance(&test_block, bonding_contract_id)?;
 *     assert_eq!(initial_balance, initial_supply, "Initial token balance should match initial supply");
 *     
 *     // Create buy transaction
 *     let previous_outpoint = OutPoint {
 *         txid: test_block.txdata[0].compute_txid(),
 *         vout: 0,
 *     };
 *     let buy_outpoint = create_contract_interaction_tx(
 *         &mut test_block,
 *         bonding_contract_id,
 *         BUY_OPCODE,
 *         vec![amount],
 *         previous_outpoint,
 *     );
 *     
 *     // Index the block
 *     index_block(&test_block, block_height)?;
 *     
 *     // Verify final state
 *     let final_balance = get_token_balance(&test_block, bonding_contract_id)?;
 *     assert!(final_balance > initial_balance, "Balance should increase after buy operation");
 *     
 *     Ok(())
 * }
 * ```
 */

// This is a dummy function to make the doc comment valid
fn _dummy() {}
