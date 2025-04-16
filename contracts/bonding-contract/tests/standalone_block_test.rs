//! Standalone block-based test for the bonding contract
//!
//! This file contains a simple test that doesn't rely on the alkanes crate.

#[cfg(test)]
mod tests {
    use protorune::test_helpers::{
        clear,
        create_block_with_txs,
        create_coinbase_transaction,
    };
    use anyhow::Result;

    #[test]
    fn test_simple_block_creation() -> Result<()> {
        // Clear environment
        clear();
        
        // Create a simple block with a coinbase transaction
        let coinbase_tx = create_coinbase_transaction(1);
        let block = create_block_with_txs(vec![coinbase_tx]);
        
        // Verify the block was created successfully
        assert_eq!(block.txdata.len(), 1, "Block should have 1 transaction");
        
        println!("Simple block creation test passed!");
        Ok(())
    }
}
