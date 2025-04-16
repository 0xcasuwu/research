use alkanes_runtime::{runtime::AlkaneResponder, message::MessageDispatch};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::response::CallResponse;
use alkanes_support::context::Context;
use alkanes_support::id::AlkaneId;
use metashrew_support::index_pointer::KeyValuePointer;
use anyhow::{anyhow, Result};
use std::sync::Arc;

/// BondOrbital represents a single bond as an orbital token
#[derive(Default)]
pub struct BondOrbital {
    /// The amount owed to the bond holder
    owed: u128,
    /// The amount already redeemed
    redeemed: u128,
    /// The creation block number
    creation: u64,
    /// The term in blocks
    term: u64,
    /// The bonding contract ID
    bonding_contract_id: AlkaneId,
}

/// Message enum for opcode-based dispatch
// #[derive(MessageDispatch)]
enum BondOrbitalMessage {
    /// Initialize the bond orbital
    // #[opcode(0)]
    Initialize {
        owed: u128,
        creation: u64,
        term: u64,
    },
    
    /// Redeem the bond
    // #[opcode(200)]
    Redeem,
    
    /// Get the bond details
    // #[opcode(102)]
    // #[returns(CallResponse)]
    GetBondDetails,
    
    /// Get the amount owed to the bond holder
    // #[opcode(103)]
    // #[returns(CallResponse)]
    GetOwed,
    
    /// Get the amount already redeemed
    // #[opcode(104)]
    // #[returns(CallResponse)]
    GetRedeemed,
    
    /// Get the creation block number
    // #[opcode(105)]
    // #[returns(CallResponse)]
    GetCreation,
    
    /// Get the term in blocks
    // #[opcode(106)]
    // #[returns(CallResponse)]
    GetTerm,
    
    /// Get the maturity block number
    // #[opcode(107)]
    // #[returns(CallResponse)]
    GetMaturity,
    
    /// Get the bonding contract ID
    // #[opcode(108)]
    // #[returns(CallResponse)]
    GetBondingContractId,
    
    /// Get the name of the token
    // #[opcode(99)]
    // #[returns(CallResponse)]
    GetName,
    
    /// Get the symbol of the token
    // #[opcode(100)]
    // #[returns(CallResponse)]
    GetSymbol,
    
    /// Get the total supply of the token
    // #[opcode(101)]
    // #[returns(CallResponse)]
    GetTotalSupply,
}

// Manually implement MessageDispatch trait for BondOrbitalMessage
impl MessageDispatch<BondOrbitalMessage> for BondOrbitalMessage {
    fn from_opcode(_opcode: u128, _args: Vec<u128>) -> Result<Self, anyhow::Error> {
        // Placeholder implementation
        Err(anyhow!("Not implemented"))
    }
    
    fn export_abi() -> Vec<u8> {
        // Placeholder implementation
        Vec::new()
    }
    
    fn dispatch(&self, _contract: &BondOrbitalMessage) -> Result<CallResponse, anyhow::Error> {
        // Placeholder implementation
        Err(anyhow!("Not implemented"))
    }
}

impl BondOrbital {
    // Storage functions
    
    /// Get the pointer to the name
    fn name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/name")
    }
    
    /// Get the name
    pub fn name(&self) -> String {
        String::from_utf8_lossy(self.name_pointer().get().as_ref()).to_string()
    }
    
    /// Set the name
    fn set_name(&self, name: &str) {
        self.name_pointer().set(Arc::new(name.as_bytes().to_vec()));
    }
    
    /// Get the pointer to the symbol
    fn symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/symbol")
    }
    
    /// Get the symbol
    pub fn symbol(&self) -> String {
        String::from_utf8_lossy(self.symbol_pointer().get().as_ref()).to_string()
    }
    
    /// Set the symbol
    fn set_symbol(&self, symbol: &str) {
        self.symbol_pointer().set(Arc::new(symbol.as_bytes().to_vec()));
    }
    
    /// Get the pointer to the owed amount
    fn owed_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/owed")
    }
    
    /// Get the owed amount
    pub fn owed(&self) -> u128 {
        self.owed_pointer().get_value::<u128>()
    }
    
    /// Set the owed amount
    fn set_owed(&self, owed: u128) {
        self.owed_pointer().set_value::<u128>(owed);
    }
    
    /// Get the pointer to the redeemed amount
    fn redeemed_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/redeemed")
    }
    
    /// Get the redeemed amount
    pub fn redeemed(&self) -> u128 {
        self.redeemed_pointer().get_value::<u128>()
    }
    
    /// Set the redeemed amount
    fn set_redeemed(&self, redeemed: u128) {
        self.redeemed_pointer().set_value::<u128>(redeemed);
    }
    
    /// Get the pointer to the creation block number
    fn creation_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/creation")
    }
    
    /// Get the creation block number
    pub fn creation(&self) -> u64 {
        self.creation_pointer().get_value::<u64>()
    }
    
    /// Set the creation block number
    fn set_creation(&self, creation: u64) {
        self.creation_pointer().set_value::<u64>(creation);
    }
    
    /// Get the pointer to the term in blocks
    fn term_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/term")
    }
    
    /// Get the term in blocks
    pub fn term(&self) -> u64 {
        self.term_pointer().get_value::<u64>()
    }
    
    /// Set the term in blocks
    fn set_term(&self, term: u64) {
        self.term_pointer().set_value::<u64>(term);
    }
    
    /// Get the pointer to the bonding contract ID
    fn bonding_contract_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/bonding_contract_id")
    }
    
    /// Get the bonding contract ID
    pub fn bonding_contract_id(&self) -> AlkaneId {
        let data = self.bonding_contract_id_pointer().get();
        
        if data.len() == 0 {
            return AlkaneId { block: 0, tx: 0 };
        }
        
        // Deserialize the AlkaneId from storage
        let bytes = data.as_ref();
        AlkaneId {
            block: u128::from_le_bytes(bytes[0..16].try_into().unwrap()),
            tx: u128::from_le_bytes(bytes[16..32].try_into().unwrap()),
        }
    }
    
    /// Set the bonding contract ID
    fn set_bonding_contract_id(&self, id: &AlkaneId) {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&id.block.to_le_bytes());
        bytes.extend_from_slice(&id.tx.to_le_bytes());
        
        self.bonding_contract_id_pointer().set(Arc::new(bytes));
    }
    
    /// Get the current block number
    fn get_current_block_number(&self) -> u64 {
        // Get the current block number from the context
        match self.context() {
            Ok(context) => context.myself.block as u64,
            Err(_) => {
                // Fallback to timestamp-based calculation if context is not available
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                // Convert seconds to blocks (assuming 1 block per 10 seconds)
                (now / 10) as u64
            }
        }
    }
    
    /// Get the maturity block number
    pub fn maturity(&self) -> u64 {
        self.creation() + self.term()
    }
    
    /// Check if the bond is mature
    pub fn is_mature(&self) -> bool {
        let current_block = self.get_current_block_number();
        current_block >= self.maturity()
    }
    
    /// Check if the bond is fully redeemed
    pub fn is_fully_redeemed(&self) -> bool {
        self.redeemed() >= self.owed()
    }
    
    /// Get the remaining amount to redeem
    pub fn remaining(&self) -> u128 {
        self.owed().saturating_sub(self.redeemed())
    }
    
    // Contract operations
    
    /// Initialize the bond orbital
    pub fn initialize(&self, owed: u128, creation: u64, term: u64) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Set bond orbital properties
        self.set_name("Bond Orbital");
        self.set_symbol("BOND");
        self.set_owed(owed);
        self.set_redeemed(0);
        self.set_creation(creation);
        self.set_term(term);
        
        // Set the bonding contract ID to the caller
        self.set_bonding_contract_id(&context.caller);
        
        Ok(response)
    }
    
    /// Redeem the bond
    pub fn redeem(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Check if the caller is the bonding contract
        let bonding_contract_id = self.bonding_contract_id();
        if context.caller != bonding_contract_id {
            return Err(anyhow!("caller is not the bonding contract"));
        }
        
        // Check if the bond is mature
        if !self.is_mature() {
            return Err(anyhow!("bond not yet mature"));
        }
        
        // Check if the bond is fully redeemed
        if self.is_fully_redeemed() {
            return Err(anyhow!("bond already fully redeemed"));
        }
        
        // Calculate the amount to redeem
        let remaining = self.remaining();
        
        // Update the redeemed amount
        self.set_redeemed(self.owed());
        
        // Add the alkane to the response
        response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
            id: bonding_contract_id,
            value: remaining,
        });
        
        Ok(response)
    }
    
    /// Get the bond details
    pub fn get_bond_details(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Serialize the bond details
        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(&self.owed().to_le_bytes());
        data.extend_from_slice(&self.redeemed().to_le_bytes());
        data.extend_from_slice(&self.creation().to_le_bytes());
        data.extend_from_slice(&self.term().to_le_bytes());
        
        response.data = data;
        
        Ok(response)
    }
    
    /// Get the owed amount
    pub fn get_owed(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.owed().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the redeemed amount
    pub fn get_redeemed(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.redeemed().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the creation block number
    pub fn get_creation(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.creation().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the term in blocks
    pub fn get_term(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.term().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the maturity block number
    pub fn get_maturity(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.maturity().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the bonding contract ID
    pub fn get_bonding_contract_id(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let id = self.bonding_contract_id();
        let mut data = Vec::with_capacity(32);
        data.extend_from_slice(&id.block.to_le_bytes());
        data.extend_from_slice(&id.tx.to_le_bytes());
        
        response.data = data;
        
        Ok(response)
    }
    
    /// Get the name of the token
    pub fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.name().as_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the symbol of the token
    pub fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.symbol().as_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the total supply of the token
    pub fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Bond orbitals always have a total supply of 1
        response.data = 1u128.to_le_bytes().to_vec();
        
        Ok(response)
    }
}

// Implement AlkaneResponder for BondOrbital
impl AlkaneResponder for BondOrbital {
    fn context(&self) -> Result<Context> {
        use crate::mock_runtime::get_context;
        get_context().ok_or_else(|| anyhow!("No context available"))
    }
    
    fn execute(&self) -> Result<CallResponse> {
        Err(anyhow!("Use the declare_alkane macro instead"))
    }
}
