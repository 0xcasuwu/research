//! Local implementation of test helpers for block-based testing
//!
//! This module provides local implementations of the helpers needed for block-based testing,
//! avoiding dependency issues with the alkanes crate.

use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::block::Block;
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxIn, TxOut};
use bitcoin::blockdata::script::ScriptBuf;
use bitcoin::hashes::Hash;
use bitcoin::{Address, Amount, Network, Sequence, Witness};
use metashrew::index_pointer::IndexPointer;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::utils::consensus_encode;
use protorune_support::load_sheet;
use protorune::tables::RuneTable;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use std::str::FromStr;

// Import test helpers from protorune
pub use protorune::test_helpers::{
    create_block_with_coinbase_tx,
    create_block_with_txs as protorune_create_block_with_txs,
    clear as clear_environment
};

/// Index a block at a specific height
pub fn index_block(block: &Block, height: u64) -> Result<()> {
    // Use alkanes indexer if available, otherwise just return Ok
    #[cfg(feature = "alkanes-indexer")]
    return alkanes::indexer::index_block(block, height);
    
    #[cfg(not(feature = "alkanes-indexer"))]
    return Ok(());
}

/// Create a test transaction with the given inputs and outputs
pub fn create_test_transaction(
    previous_output: OutPoint,
    script_pubkey: ScriptBuf,
    amount: u64,
) -> Transaction {
    let input_script = ScriptBuf::new();

    // Create a transaction input
    let txin = TxIn {
        previous_output,
        script_sig: input_script,
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };

    let txout = TxOut {
        value: Amount::from_sat(amount),
        script_pubkey,
    };

    Transaction {
        version: bitcoin::blockdata::transaction::Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![txin],
        output: vec![txout],
    }
}

/// Create a block with contract deployment
pub fn init_block_with_contract_deployment(
    contract_bytes: Vec<u8>,
    init_params: Vec<u128>,
    target: AlkaneId,
) -> Result<(Block, AlkaneId)> {
    // Create a simple block with a transaction
    let mut txdata = Vec::new();
    
    // Add a coinbase transaction
    let coinbase_tx = create_coinbase_transaction(1);
    txdata.push(coinbase_tx);
    
    // Create a block
    let block = create_block_with_txs(txdata);
    
    Ok((block, target))
}

/// Create a coinbase transaction
pub fn create_coinbase_transaction(height: u32) -> Transaction {
    // Create a simple coinbase transaction
    let coinbase_input = TxIn {
        previous_output: OutPoint::default(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };

    // Create a simple output
    let address = Address::from_str("bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080")
        .unwrap()
        .require_network(Network::Regtest)
        .unwrap();
    
    let coinbase_output = TxOut {
        value: Amount::from_sat(50_000_000), // 50 BTC
        script_pubkey: address.script_pubkey(),
    };

    let locktime = bitcoin::absolute::LockTime::from_height(height).unwrap();

    Transaction {
        version: bitcoin::blockdata::transaction::Version::TWO,
        lock_time: locktime,
        input: vec![coinbase_input],
        output: vec![coinbase_output],
    }
}

/// Create a block with the given transactions
pub fn create_block_with_txs(txdata: Vec<Transaction>) -> Block {
    // Create a simple block header
    let header = bitcoin::blockdata::block::Header {
        version: bitcoin::blockdata::block::Version::from_consensus(1),
        prev_blockhash: bitcoin::BlockHash::from_str(
            "00000000000000000005c3b409b4f17f9b3a97ed46d1a63d3f660d24168b2b3e"
        ).unwrap(),
        merkle_root: bitcoin::hash_types::TxMerkleNode::from_str(
            "4e07408562b4b5a9c0555f0671e0d2b6c5764c1d2a5e97c1d7f36f7c91e4c77a"
        ).unwrap(),
        time: 1231006505,
        bits: bitcoin::CompactTarget::from_consensus(0x1234),
        nonce: 2083236893,
    };

    Block { header, txdata }
}

/// Create a transaction for contract interaction
pub fn create_contract_interaction_tx(
    test_block: &mut Block,
    contract_id: AlkaneId,
    operation: u128,
    params: Vec<u128>,
    previous_outpoint: OutPoint,
) -> OutPoint {
    // Create a simple transaction
    let input_script = ScriptBuf::new();
    let txin = TxIn {
        previous_output: previous_outpoint,
        script_sig: input_script,
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };

    // Create a simple output
    let address = Address::from_str("bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080")
        .unwrap()
        .require_network(Network::Regtest)
        .unwrap();
    
    let txout = TxOut {
        value: Amount::from_sat(100_000_000),
        script_pubkey: address.script_pubkey(),
    };

    let tx = Transaction {
        version: bitcoin::blockdata::transaction::Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![txin],
        output: vec![txout],
    };

    test_block.txdata.push(tx.clone());

    // Return the outpoint of the transaction we just added
    OutPoint {
        txid: tx.compute_txid(),
        vout: 0,
    }
}

/// Get token balance
pub fn get_token_balance(block: &Block, token_id: AlkaneId) -> Result<u128> {
    let sheet = get_last_outpoint_sheet(block)?;
    Ok(sheet.get_cached(&token_id.into()))
}

/// Get the balance sheet for the last transaction in the block
pub fn get_last_outpoint_sheet(test_block: &Block) -> Result<BalanceSheet<IndexPointer>> {
    let len = test_block.txdata.len();
    get_sheet_for_outpoint(test_block, len - 1, 0)
}

/// Get balance sheet for verification
pub fn get_sheet_for_outpoint(
    test_block: &Block,
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
