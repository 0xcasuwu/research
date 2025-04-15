use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, message::MessageDispatch};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::response::CallResponse;
use alkanes_support::context::Context;
use alkanes_support::id::AlkaneId;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;
use anyhow::{anyhow, Result};
use std::sync::Arc;

/// Helper function to trim a u128 to a string
/// 
/// This function converts a u128 to a string and trims any null bytes.
fn trim(value: u128) -> String {
    let bytes = value.to_le_bytes();
    let mut end = 0;
    
    // Find the first null byte or end of array
    for (i, &b) in bytes.iter().enumerate() {
        if b == 0 {
            end = i;
            break;
        }
        if i == bytes.len() - 1 {
            end = bytes.len();
        }
    }
    
    // Convert to string
    String::from_utf8_lossy(&bytes[0..end]).to_string()
}

// Import our bonding curve implementation
mod bonding_curve;
// Re-export the BondingCurve struct to make it public
pub use bonding_curve::BondingCurve;

// Import our bond curve implementation
mod bond_curve;
// Re-export the BondCurve struct and related types to make them public
pub use bond_curve::{BondCurve, Bond, Pricing};

// Import mock modules for testing
pub mod mock_runtime;
pub mod mock_context;
pub mod mock_storage;
pub mod reset_mock_environment;
pub mod isolated_tests;

/// BondingContract trait defines the interface for bonding curve functionality
pub trait BondingContract: AlkaneResponder {
    /// Buy alkane with diesel
    fn buy_alkane(&mut self, diesel_amount: u128) -> Result<CallResponse>;
    
    /// Sell alkane for diesel (internal use only)
    fn sell_alkane(&mut self, alkane_amount: u128) -> Result<CallResponse>;
    
    /// Get the current reserve of diesel
    fn diesel_reserve(&self) -> u128;
    
    /// Get the current supply of alkane
    fn alkane_supply(&self) -> u128;
    
    /// Get the current price of alkane in terms of diesel
    fn current_price(&self) -> Result<CallResponse>;
}

/// BondContract trait defines the interface for bond-based functionality
pub trait BondContract: AlkaneResponder {
    /// Purchase a bond with diesel
    fn purchase_bond(&mut self, to: u128, min_output: u128) -> Result<CallResponse>;
    
    /// Redeem a bond
    fn redeem_bond(&mut self, bond_id: u128) -> Result<CallResponse>;
    
    /// Redeem multiple bonds
    fn redeem_bond_batch(&mut self, bond_ids: Vec<u128>) -> Result<CallResponse>;
    
    /// Transfer a bond to another address
    fn transfer_bond(&mut self, to: u128, bond_id: u128) -> Result<CallResponse>;
    
    /// Get the number of bonds owned by an address
    fn position_count_of(&self, address: u128) -> u128;
    
    /// Get the available debt (alkane available for redemption)
    fn available_debt(&self) -> u128;
}

/// BondingContractAlkane implements a bonding curve contract with bond functionality
#[derive(Default)]
pub struct BondingContractAlkane {
    /// The bonding curve implementation
    bonding_curve: Option<BondingCurve>,
    /// The bond curve implementation
    bond_curve: Option<BondCurve>,
    /// Whether the contract is paused
    paused: bool,
}

/// Message enum for opcode-based dispatch
// Temporarily remove MessageDispatch derive to fix compilation
// #[derive(MessageDispatch)]
enum BondingContractAlkaneMessage {
    /// Initialize the contract
    // #[opcode(0)]
    InitContract {
        name: u128,
        symbol: u128,
        k_factor: u128,
        n_exponent: u128,
        initial_diesel_reserve: u128,
    },
    
    /// Initialize the contract with bond functionality
    // #[opcode(10)]
    InitBondContract {
        name: u128,
        symbol: u128,
        virtual_input_reserves: u128,
        virtual_output_reserves: u128,
        half_life: u64,
        level_bips: u64,
        term: u64,
    },
    
    /// Buy alkane with diesel (legacy)
    // #[opcode(1)]
    BuyAlkane {
        diesel_amount: u128,
    },
    
    /// Sell alkane for diesel (legacy, internal use only)
    // #[opcode(2)]
    SellAlkane {
        alkane_amount: u128,
    },
    
    /// Purchase a bond with diesel
    // #[opcode(11)]
    PurchaseBond {
        to: u128,
        min_output: u128,
    },
    
    /// Redeem a bond
    // #[opcode(12)]
    RedeemBond {
        bond_id: u128,
    },
    
    /// Redeem multiple bonds
    // #[opcode(13)]
    RedeemBondBatch {
        bond_ids: Vec<u128>,
    },
    
    /// Transfer a bond to another address
    // #[opcode(14)]
    TransferBond {
        to: u128,
        bond_id: u128,
    },
    
    /// Get the current price of alkane in terms of diesel
    // #[opcode(3)]
    // #[returns(CallResponse)]
    GetCurrentPrice,
    
    /// Get the amount of alkane that can be received for a specific amount of diesel
    // #[opcode(4)]
    // #[returns(CallResponse)]
    GetBuyAmount {
        diesel_amount: u128,
    },
    
    /// Get the amount of diesel that can be received for a specific amount of alkane
    // #[opcode(5)]
    // #[returns(CallResponse)]
    GetSellAmount {
        alkane_amount: u128,
    },
    
    /// Get the bond price (amount of alkane for a specific amount of diesel)
    // #[opcode(15)]
    // #[returns(CallResponse)]
    GetBondAmount {
        diesel_amount: u128,
    },
    
    /// Get the number of bonds owned by an address
    // #[opcode(16)]
    // #[returns(CallResponse)]
    GetPositionCount {
        address: u128,
    },
    
    /// Get the available debt (alkane available for redemption)
    // #[opcode(17)]
    // #[returns(CallResponse)]
    GetAvailableDebt,
    
    /// Get bond details
    // #[opcode(18)]
    // #[returns(CallResponse)]
    GetBond {
        address: u128,
        bond_id: u128,
    },
    
    /// Set virtual input reserves
    // #[opcode(20)]
    SetVirtualInputReserves {
        value: u128,
    },
    
    /// Set virtual output reserves
    // #[opcode(21)]
    SetVirtualOutputReserves {
        value: u128,
    },
    
    /// Set half life
    // #[opcode(22)]
    SetHalfLife {
        value: u64,
    },
    
    /// Set level bips
    // #[opcode(23)]
    SetLevelBips {
        value: u64,
    },
    
    /// Set last update
    // #[opcode(24)]
    SetLastUpdate,
    
    /// Toggle pause
    // #[opcode(25)]
    SetPause,
    
    /// Update pricing
    // #[opcode(26)]
    UpdatePricing {
        new_virtual_input: Option<u128>,
        new_virtual_output: Option<u128>,
        new_half_life: Option<u64>,
        new_level_bips: Option<u64>,
        update_timestamp: bool,
        pause: bool,
    },
    
    /// Get the name of the token
    // #[opcode(99)]
    // #[returns(CallResponse)]
    GetName,
    
    /// Get the symbol of the token
    // #[opcode(100)]
    // #[returns(CallResponse)]
    GetSymbol,
    
    /// Get the reserve of diesel
    // #[opcode(101)]
    // #[returns(CallResponse)]
    GetDieselReserve,
    
    /// Get the supply of alkane
    // #[opcode(102)]
    // #[returns(CallResponse)]
    GetAlkaneSupply,
    
    /// Get the k factor
    // #[opcode(103)]
    // #[returns(CallResponse)]
    GetKFactor,
    
    /// Get the n exponent
    // #[opcode(104)]
    // #[returns(CallResponse)]
    GetNExponent,
    
    /// Get the term
    // #[opcode(105)]
    // #[returns(CallResponse)]
    GetTerm,
    
    /// Get the half life
    // #[opcode(106)]
    // #[returns(CallResponse)]
    GetHalfLife,
    
    /// Get the level bips
    // #[opcode(107)]
    // #[returns(CallResponse)]
    GetLevelBips,
    
    /// Get the virtual input reserves
    // #[opcode(108)]
    // #[returns(CallResponse)]
    GetVirtualInputReserves,
    
    /// Get the virtual output reserves
    // #[opcode(109)]
    // #[returns(CallResponse)]
    GetVirtualOutputReserves,
    
    /// Get the last update timestamp
    // #[opcode(110)]
    // #[returns(CallResponse)]
    GetLastUpdate,
    
    /// Get the total debt
    // #[opcode(111)]
    // #[returns(CallResponse)]
    GetTotalDebt,
    
    /// Get the paused state
    // #[opcode(112)]
    // #[returns(CallResponse)]
    GetPaused,
}

// Manually implement MessageDispatch trait for BondingContractAlkaneMessage
impl MessageDispatch<BondingContractAlkaneMessage> for BondingContractAlkaneMessage {
    fn from_opcode(_opcode: u128, _args: Vec<u128>) -> Result<Self, anyhow::Error> {
        // Placeholder implementation
        Err(anyhow!("Not implemented"))
    }
    
    fn export_abi() -> Vec<u8> {
        // Placeholder implementation
        Vec::new()
    }
    
    fn dispatch(&self, _contract: &BondingContractAlkaneMessage) -> Result<CallResponse, anyhow::Error> {
        // Placeholder implementation
        Err(anyhow!("Not implemented"))
    }
}

impl BondingContractAlkane {
    // Storage functions
    
    /// Get the pointer to the name
    fn name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/name")
    }
    
    /// Get the name
    fn name(&self) -> String {
        String::from_utf8_lossy(self.name_pointer().get().as_ref()).to_string()
    }
    
    /// Set the name
    fn set_name(&self, name: u128) {
        self.name_pointer().set(Arc::new(trim(name).as_bytes().to_vec()));
    }
    
    /// Get the pointer to the symbol
    fn symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/symbol")
    }
    
    /// Get the symbol
    fn symbol(&self) -> String {
        String::from_utf8_lossy(self.symbol_pointer().get().as_ref()).to_string()
    }
    
    /// Set the symbol
    fn set_symbol(&self, symbol: u128) {
        self.symbol_pointer().set(Arc::new(trim(symbol).as_bytes().to_vec()));
    }
    
    /// Get the pointer to the diesel reserve
    fn diesel_reserve_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/diesel_reserve")
    }
    
    /// Get the diesel reserve
    fn diesel_reserve(&self) -> u128 {
        self.diesel_reserve_pointer().get_value::<u128>()
    }
    
    /// Set the diesel reserve
    fn set_diesel_reserve(&self, reserve: u128) {
        self.diesel_reserve_pointer().set_value::<u128>(reserve);
    }
    
    /// Get the pointer to the alkane supply
    fn alkane_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/alkane_supply")
    }
    
    /// Get the alkane supply
    fn alkane_supply(&self) -> u128 {
        self.alkane_supply_pointer().get_value::<u128>()
    }
    
    /// Set the alkane supply
    fn set_alkane_supply(&self, supply: u128) {
        self.alkane_supply_pointer().set_value::<u128>(supply);
    }
    
    /// Get the pointer to the k factor
    fn k_factor_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/k_factor")
    }
    
    /// Get the k factor
    fn k_factor(&self) -> u128 {
        self.k_factor_pointer().get_value::<u128>()
    }
    
    /// Set the k factor
    fn set_k_factor(&self, k_factor: u128) {
        self.k_factor_pointer().set_value::<u128>(k_factor);
    }
    
    /// Get the pointer to the n exponent
    fn n_exponent_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/n_exponent")
    }
    
    /// Get the n exponent
    fn n_exponent(&self) -> u128 {
        self.n_exponent_pointer().get_value::<u128>()
    }
    
    /// Set the n exponent
    fn set_n_exponent(&self, n_exponent: u128) {
        self.n_exponent_pointer().set_value::<u128>(n_exponent);
    }
    
    /// Get the balance of an address
    fn balance_of(&self, address: u128) -> u128 {
        StoragePointer::from_keyword("/balances/")
            .select(&address.to_le_bytes().to_vec())
            .get_value::<u128>()
    }
    
    /// Set the balance of an address
    fn set_balance(&self, address: u128, balance: u128) {
        StoragePointer::from_keyword("/balances/")
            .select(&address.to_le_bytes().to_vec())
            .set_value::<u128>(balance);
    }
    
    /// Observe initialization to prevent multiple initializations
    fn observe_initialization(&self) -> Result<()> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().len() == 0 {
            pointer.set_value::<u8>(0x01);
            Ok(())
        } else {
            Err(anyhow!("already initialized"))
        }
    }
    
    // Bond-related storage functions
    
    /// Get the pointer to the owner
    fn owner_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/owner")
    }
    
    /// Get the owner
    fn owner(&self) -> u128 {
        self.owner_pointer().get_value::<u128>()
    }
    
    /// Set the owner
    fn set_owner(&self, owner: u128) {
        self.owner_pointer().set_value::<u128>(owner);
    }
    
    /// Check if the caller is the owner
    fn ensure_owner(&self) -> Result<()> {
        let context = self.context()?;
        let owner = self.owner();
        
        // Fixed: Use the caller's ID directly instead of to_bytes()
        let caller_u128 = context.caller.block;
        
        if caller_u128 != owner {
            return Err(anyhow!("caller is not the owner"));
        }
        
        Ok(())
    }
    
    /// Get the pointer to the paused state
    fn paused_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/paused")
    }
    
    /// Get the paused state
    fn is_paused(&self) -> bool {
        self.paused_pointer().get_value::<u8>() == 1
    }
    
    /// Set the paused state
    fn set_paused(&self, paused: bool) {
        self.paused_pointer().set_value::<u8>(if paused { 1 } else { 0 });
    }
    
    /// Get the pointer to the term
    fn term_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/term")
    }
    
    /// Get the term
    fn term(&self) -> u64 {
        self.term_pointer().get_value::<u64>()
    }
    
    /// Set the term
    fn set_term(&self, term: u64) {
        self.term_pointer().set_value::<u64>(term);
    }
    
    /// Get the pointer to the total debt
    fn total_debt_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/total_debt")
    }
    
    /// Get the total debt
    fn total_debt(&self) -> u128 {
        self.total_debt_pointer().get_value::<u128>()
    }
    
    /// Set the total debt
    fn set_total_debt(&self, debt: u128) {
        self.total_debt_pointer().set_value::<u128>(debt);
    }
    
    /// Get the pointer to the virtual input reserves
    fn virtual_input_reserves_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/virtual_input_reserves")
    }
    
    /// Get the virtual input reserves
     fn virtual_input_reserves(&self) -> u128 {
        self.virtual_input_reserves_pointer().get_value::<u128>()
    }
    
    /// Set the virtual input reserves (internal storage function)
    fn set_virtual_input_reserves_internal(&self, reserves: u128) {
        self.virtual_input_reserves_pointer().set_value::<u128>(reserves);
    }
    
    /// Get the pointer to the virtual output reserves
    fn virtual_output_reserves_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/virtual_output_reserves")
    }
    
    /// Get the virtual output reserves
    fn virtual_output_reserves(&self) -> u128 {
        self.virtual_output_reserves_pointer().get_value::<u128>()
    }
    
    /// Set the virtual output reserves (internal storage function)
    fn set_virtual_output_reserves_internal(&self, reserves: u128) {
        self.virtual_output_reserves_pointer().set_value::<u128>(reserves);
    }
    
    /// Get the pointer to the half life
    fn half_life_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/half_life")
    }
    
    /// Get the half life
    fn half_life(&self) -> u64 {
        self.half_life_pointer().get_value::<u64>()
    }
    
    /// Set the half life (internal storage function)
    fn set_half_life_internal(&self, half_life: u64) {
        self.half_life_pointer().set_value::<u64>(half_life);
    }
    
    /// Get the pointer to the level bips
    fn level_bips_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/level_bips")
    }
    
    /// Get the level bips
    fn level_bips(&self) -> u64 {
        self.level_bips_pointer().get_value::<u64>()
    }
    
    /// Set the level bips (internal storage function)
    fn set_level_bips_internal(&self, level_bips: u64) {
        self.level_bips_pointer().set_value::<u64>(level_bips);
    }
    
    /// Get the pointer to the last update
    fn last_update_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/last_update")
    }
    
    /// Get the last update
    fn last_update(&self) -> u64 {
        self.last_update_pointer().get_value::<u64>()
    }
    
    /// Set the last update (internal storage function)
    fn set_last_update_internal(&self, last_update: u64) {
        self.last_update_pointer().set_value::<u64>(last_update);
    }
    
    /// Get the current timestamp
    fn get_current_timestamp(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Get the pointer to the bonds of an address
    fn bonds_pointer(&self, address: u128) -> StoragePointer {
        StoragePointer::from_keyword("/bonds/")
            .select(&address.to_le_bytes().to_vec())
    }
    
    /// Get the bonds of an address
    fn get_bonds(&self, address: u128) -> Vec<Bond> {
        let pointer = self.bonds_pointer(address);
        let count = self.position_count_of(address);
        
        let mut bonds = Vec::new();
        for i in 0..count {
            let bond_pointer = pointer.select(&i.to_le_bytes().to_vec());
            
            // Convert byte slices to Vec<u8> for select method
            let owed = bond_pointer.select(&b"owed".to_vec()).get_value::<u128>();
            let redeemed = bond_pointer.select(&b"redeemed".to_vec()).get_value::<u128>();
            let creation = bond_pointer.select(&b"creation".to_vec()).get_value::<u64>();
            
            bonds.push(Bond {
                owed,
                redeemed,
                creation,
            });
        }
        
        bonds
    }
    
    /// Get a specific bond of an address
    fn get_bond(&self, address: u128, bond_id: u128) -> Option<Bond> {
        let count = self.position_count_of(address);
        
        if bond_id >= count {
            return None;
        }
        
        let pointer = self.bonds_pointer(address).select(&bond_id.to_le_bytes().to_vec());
        
        // Convert byte slices to Vec<u8> for select method
        let owed = pointer.select(&b"owed".to_vec()).get_value::<u128>();
        let redeemed = pointer.select(&b"redeemed".to_vec()).get_value::<u128>();
        let creation = pointer.select(&b"creation".to_vec()).get_value::<u64>();
        
        Some(Bond {
            owed,
            redeemed,
            creation,
        })
    }
    
    /// Add a bond to an address
    fn add_bond(&self, address: u128, bond: Bond) {
        let pointer = self.bonds_pointer(address);
        let count = self.position_count_of(address);
        
        let bond_pointer = pointer.select(&count.to_le_bytes().to_vec());
        
        // Convert byte slices to Vec<u8> for select method
        bond_pointer.select(&b"owed".to_vec()).set_value::<u128>(bond.owed);
        bond_pointer.select(&b"redeemed".to_vec()).set_value::<u128>(bond.redeemed);
        bond_pointer.select(&b"creation".to_vec()).set_value::<u64>(bond.creation);
        
        // Update the count
        self.set_position_count(address, count + 1);
    }
    
    /// Update a bond of an address
    fn update_bond(&self, address: u128, bond_id: u128, bond: Bond) {
        let pointer = self.bonds_pointer(address).select(&bond_id.to_le_bytes().to_vec());
        
        // Convert byte slices to Vec<u8> for select method
        pointer.select(&b"owed".to_vec()).set_value::<u128>(bond.owed);
        pointer.select(&b"redeemed".to_vec()).set_value::<u128>(bond.redeemed);
        pointer.select(&b"creation".to_vec()).set_value::<u64>(bond.creation);
    }
    
    /// Delete a bond of an address
    fn delete_bond(&self, address: u128, bond_id: u128) {
        let count = self.position_count_of(address);
        
        if bond_id >= count {
            return;
        }
        
        // If it's the last bond, just decrease the count
        if bond_id == count - 1 {
            self.set_position_count(address, count - 1);
            return;
        }
        
        // Otherwise, move the last bond to the deleted position
        let last_bond = self.get_bond(address, count - 1).unwrap();
        self.update_bond(address, bond_id, last_bond);
        
        // Decrease the count
        self.set_position_count(address, count - 1);
    }
    
    /// Get the pointer to the position count of an address
    fn position_count_pointer(&self, address: u128) -> StoragePointer {
        StoragePointer::from_keyword("/position_counts/")
            .select(&address.to_le_bytes().to_vec())
    }
    
    /// Get the position count of an address (internal method)
    fn position_count_of_internal(&self, address: u128) -> u128 {
        self.position_count_pointer(address).get_value::<u128>()
    }
    
    /// Set the position count of an address
    fn set_position_count(&self, address: u128, count: u128) {
        self.position_count_pointer(address).set_value::<u128>(count);
    }
    
    /// Get the available debt (alkane available for redemption) (internal method)
    fn available_debt_internal(&self) -> u128 {
        self.alkane_supply().saturating_sub(self.total_debt())
    }
    
    // Bonding curve functions
    
    /// Get the bonding curve instance
    pub fn get_bonding_curve(&self) -> BondingCurve {
        if let Some(curve) = &self.bonding_curve {
            curve.clone()
        } else {
            // Create a new bonding curve from storage
            BondingCurve::new(
                self.diesel_reserve(),
                self.alkane_supply(),
                self.k_factor(),
                self.n_exponent(),
            )
        }
    }
    
    /// Get the bond curve instance
    pub fn get_bond_curve(&self) -> BondCurve {
        if let Some(curve) = &self.bond_curve {
            curve.clone()
        } else {
            // Create a new bond curve from storage
            BondCurve::new(
                self.virtual_input_reserves(),
                self.virtual_output_reserves(),
                self.half_life(),
                self.level_bips(),
                self.term(),
            )
        }
    }
    
    /// Calculate the amount of alkane to mint for a given diesel amount
    fn get_buy_amount(&self, diesel_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let curve = self.get_bonding_curve();
        // Call the BondingCurve's get_buy_amount method which returns a u128
        let amount = curve.get_buy_amount(diesel_amount);

        response.data = amount.to_le_bytes().to_vec();

        Ok(response)
    }
    
    /// Calculate the amount of diesel to return for a given alkane amount
    fn get_sell_amount(&self, alkane_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let curve = self.get_bonding_curve();
        // Call the BondingCurve's get_sell_amount method which returns a u128
        let amount = curve.get_sell_amount(alkane_amount);
        
        response.data = amount.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the current price of alkane in terms of diesel
    pub fn current_price_internal(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let curve = self.get_bonding_curve();
        let price = curve.get_current_price();
        
        response.data = price.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Contract operations
    
    /// Initialize the contract
    pub fn init_contract(&self, name: u128, symbol: u128, k_factor: u128, n_exponent: u128, initial_diesel_reserve: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Prevent multiple initializations
        self.observe_initialization()?;
        
        // Set contract properties
        self.set_name(name);
        self.set_symbol(symbol);
        self.set_k_factor(k_factor);
        self.set_n_exponent(n_exponent);
        self.set_diesel_reserve(initial_diesel_reserve);
        
        // Calculate initial alkane supply based on the bonding curve
        // For simplicity, we'll start with a 1:1 ratio
        self.set_alkane_supply(initial_diesel_reserve);
        
        // Set the owner to the caller
        let caller_u128 = context.caller.block;
        self.set_owner(caller_u128);
        
        Ok(response)
    }
    
    /// Initialize the contract with bond functionality
    pub fn init_bond_contract(
        &self,
        name: u128,
        symbol: u128,
        virtual_input_reserves: u128,
        virtual_output_reserves: u128,
        half_life: u64,
        level_bips: u64,
        term: u64
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Prevent multiple initializations
        self.observe_initialization()?;
        
        // Set contract properties
        self.set_name(name);
        self.set_symbol(symbol);
        
        // Set bond curve properties
        self.set_virtual_input_reserves_internal(virtual_input_reserves);
        self.set_virtual_output_reserves_internal(virtual_output_reserves);
        self.set_half_life_internal(half_life);
        self.set_level_bips_internal(level_bips);
        self.set_term(term);
        self.set_last_update_internal(self.get_current_timestamp());
        
        // Initialize total debt to 0
        self.set_total_debt(0);
        
        // Set the owner to the caller
        let caller_u128 = context.caller.block;
        self.set_owner(caller_u128);
        
        // Set paused to true initially
        self.set_paused(true);
        
        Ok(response)
    }
    
    // Implement BondingContract trait methods
    
    /// Buy alkane with diesel
    fn buy_alkane_internal(&mut self, diesel_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Get the diesel from the incoming alkanes
        let mut diesel_amount = 0;
        for alkane in &context.incoming_alkanes.0 {
            if alkane.id.block == 2 && alkane.id.tx == 0 {
                // This is diesel
                diesel_amount += alkane.value;
            }
        }
        
        if diesel_amount == 0 {
            return Err(anyhow!("no diesel provided"));
        }
        
        // Get the bonding curve
        let mut curve = self.get_bonding_curve();
        
        // Calculate the amount of alkane to mint
        let alkane_amount = curve.buy_alkane(diesel_amount);
        
        if alkane_amount == 0 {
            return Err(anyhow!("no alkane minted"));
        }
        
        // Update the contract state
        self.set_diesel_reserve(curve.diesel_reserve);
        self.set_alkane_supply(curve.alkane_supply);
        
        // Update the bonding curve instance
        self.bonding_curve = Some(curve);
        
        // Add the alkane to the response
        response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
            id: context.myself,
            value: alkane_amount,
        });
        
        Ok(response)
    }
    
    /// Sell alkane for diesel
    fn sell_alkane_internal(&mut self, alkane_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Get the alkane from the incoming alkanes
        let mut actual_alkane_amount = 0;
        for alkane in &context.incoming_alkanes.0 {
            if alkane.id == context.myself {
                // This is our contract's alkane
                actual_alkane_amount += alkane.value;
            }
        }
        
        // If no alkane was provided, use the parameter
        if actual_alkane_amount == 0 {
            actual_alkane_amount = alkane_amount;
        }
        
        if actual_alkane_amount == 0 {
            return Err(anyhow!("no alkane provided"));
        }
        
        // Get the bonding curve
        let mut curve = self.get_bonding_curve();
        
        // Calculate the amount of diesel to return
        let diesel_amount = curve.sell_alkane(actual_alkane_amount);
        
        if diesel_amount == 0 {
            return Err(anyhow!("no diesel returned"));
        }
        
        // Update the contract state
        self.set_diesel_reserve(curve.diesel_reserve);
        self.set_alkane_supply(curve.alkane_supply);
        
        // Update the bonding curve instance
        self.bonding_curve = Some(curve);
        
        // Add the diesel to the response
        response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel
            value: diesel_amount,
        });
        
        Ok(response)
    }
    
    // Implement BondContract trait methods
    
    /// Purchase a bond with diesel
    fn purchase_bond_internal(&mut self, to: u128, min_output: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Check if the contract is paused
        if self.is_paused() {
            return Err(anyhow!("contract is paused"));
        }
        
        // Get the diesel from the incoming alkanes
        let mut diesel_amount = 0;
        for alkane in &context.incoming_alkanes.0 {
            if alkane.id.block == 2 && alkane.id.tx == 0 {
                // This is diesel
                diesel_amount += alkane.value;
            }
        }
        
        if diesel_amount == 0 {
            return Err(anyhow!("no diesel provided"));
        }
        
        // Get the bond curve
        let mut curve = self.get_bond_curve();
        
        // Calculate the amount of alkane to mint
        // Convert timestamp from u64 to u128
        let current_time = self.get_current_timestamp();
        let available_debt = self.available_debt();
        let alkane_amount = curve.purchase_bond(diesel_amount, available_debt);
        
        // Create a new bond
        let bond = Bond {
            owed: alkane_amount,
            redeemed: 0,
            creation: current_time,
        };
        
        if alkane_amount < min_output {
            return Err(anyhow!("output less than minimum"));
        }
        
        // Update the total debt
        self.set_total_debt(self.total_debt() + bond.owed);
        
        // Add the bond to the recipient
        self.add_bond(to, bond);
        
        // Update the contract state
        self.set_virtual_input_reserves_internal(curve.pricing.virtual_input_reserves);
        self.set_virtual_output_reserves_internal(curve.pricing.virtual_output_reserves);
        self.set_last_update_internal(curve.pricing.last_update);
        
        // Update the bond curve instance
        self.bond_curve = Some(curve);
        
        Ok(response)
    }
    
    /// Redeem a bond
    fn redeem_bond_internal(&mut self, bond_id: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Check if the contract is paused
        if self.is_paused() {
            return Err(anyhow!("contract is paused"));
        }
        
        // Get the caller
        let caller = context.caller.block;
        
        // Get the bond
        let bond = self.get_bond(caller, bond_id).ok_or_else(|| anyhow!("bond not found"))?;
        
        // Check if the bond is fully redeemed
        if bond.owed <= bond.redeemed {
            return Err(anyhow!("bond already fully redeemed"));
        }
        
        // Check if the bond is mature
        let current_time = self.get_current_timestamp();
        if current_time < bond.creation + self.term() {
            return Err(anyhow!("bond not yet mature"));
        }
        
        // Calculate the amount to redeem
        let available_debt = self.available_debt();
        let remaining = bond.owed - bond.redeemed;
        let to_redeem = std::cmp::min(remaining, available_debt);
        
        if to_redeem == 0 {
            return Err(anyhow!("no debt available for redemption"));
        }
        
        // Update the bond
        let mut updated_bond = bond.clone();
        updated_bond.redeemed += to_redeem;
        self.update_bond(caller, bond_id, updated_bond);
        
        // Add the alkane to the response
        response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
            id: context.myself,
            value: to_redeem,
        });
        
        Ok(response)
    }
    
    /// Redeem multiple bonds
    fn redeem_bond_batch_internal(&mut self, bond_ids: Vec<u128>) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Check if the contract is paused
        if self.is_paused() {
            return Err(anyhow!("contract is paused"));
        }
        
        // Get the caller
        let caller = context.caller.block;
        
        // Calculate the total amount to redeem
        let mut total_to_redeem = 0;
        let current_time = self.get_current_timestamp();
        let available_debt = self.available_debt();
        let mut remaining_debt = available_debt;
        
        // First pass: calculate how much we can redeem
        for &bond_id in &bond_ids {
            // Get the bond
            let bond = match self.get_bond(caller, bond_id) {
                Some(b) => b,
                None => continue, // Skip if bond not found
            };
            
            // Check if the bond is fully redeemed
            if bond.owed <= bond.redeemed {
                continue;
            }
            
            // Check if the bond is mature
            if current_time < bond.creation + self.term() {
                continue;
            }
            
            // Calculate the amount to redeem
            let remaining = bond.owed - bond.redeemed;
            let to_redeem = std::cmp::min(remaining, remaining_debt);
            
            if to_redeem == 0 {
                break; // No more debt available
            }
            
            total_to_redeem += to_redeem;
            remaining_debt -= to_redeem;
        }
        
        if total_to_redeem == 0 {
            return Err(anyhow!("no bonds eligible for redemption"));
        }
        
        // Second pass: actually redeem the bonds
        remaining_debt = available_debt;
        for &bond_id in &bond_ids {
            // Get the bond
            let bond = match self.get_bond(caller, bond_id) {
                Some(b) => b,
                None => continue, // Skip if bond not found
            };
            
            // Check if the bond is fully redeemed
            if bond.owed <= bond.redeemed {
                continue;
            }
            
            // Check if the bond is mature
            if current_time < bond.creation + self.term() {
                continue;
            }
            
            // Calculate the amount to redeem
            let remaining = bond.owed - bond.redeemed;
            let to_redeem = std::cmp::min(remaining, remaining_debt);
            
            if to_redeem == 0 {
                break; // No more debt available
            }
            
            // Update the bond
            let mut updated_bond = bond.clone();
            updated_bond.redeemed += to_redeem;
            self.update_bond(caller, bond_id, updated_bond);
            
            remaining_debt -= to_redeem;
        }
        
        // Add the alkane to the response
        response.alkanes.0.push(alkanes_support::parcel::AlkaneTransfer {
            id: context.myself,
            value: total_to_redeem,
        });
        
        Ok(response)
    }
    
    /// Transfer a bond to another address
    fn transfer_bond_internal(&mut self, to: u128, bond_id: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Get the caller
        let caller = context.caller.block;
        
        // Get the bond
        let bond = self.get_bond(caller, bond_id).ok_or_else(|| anyhow!("bond not found"))?;
        
        // Add the bond to the recipient
        self.add_bond(to, bond);
        
        // Delete the bond from the caller
        self.delete_bond(caller, bond_id);
        
        Ok(response)
    }
}

// Implement BondingContract trait for BondingContractAlkane
impl BondingContract for BondingContractAlkane {
    fn buy_alkane(&mut self, diesel_amount: u128) -> Result<CallResponse> {
        self.buy_alkane_internal(diesel_amount)
    }
    
    fn sell_alkane(&mut self, alkane_amount: u128) -> Result<CallResponse> {
        self.sell_alkane_internal(alkane_amount)
    }
    
    fn diesel_reserve(&self) -> u128 {
        self.diesel_reserve()
    }
    
    fn alkane_supply(&self) -> u128 {
        self.alkane_supply()
    }
    
    fn current_price(&self) -> Result<CallResponse> {
        self.current_price_internal()
    }
}

// Implement BondContract trait for BondingContractAlkane
impl BondContract for BondingContractAlkane {
    fn purchase_bond(&mut self, to: u128, min_output: u128) -> Result<CallResponse> {
        self.purchase_bond_internal(to, min_output)
    }
    
    fn redeem_bond(&mut self, bond_id: u128) -> Result<CallResponse> {
        self.redeem_bond_internal(bond_id)
    }
    
    fn redeem_bond_batch(&mut self, bond_ids: Vec<u128>) -> Result<CallResponse> {
        self.redeem_bond_batch_internal(bond_ids)
    }
    
    fn transfer_bond(&mut self, to: u128, bond_id: u128) -> Result<CallResponse> {
        self.transfer_bond_internal(to, bond_id)
    }
    
    fn position_count_of(&self, address: u128) -> u128 {
        self.position_count_of_internal(address)
    }
    
    fn available_debt(&self) -> u128 {
        self.available_debt_internal()
    }
}

// Implement AlkaneResponder for BondingContractAlkane
impl AlkaneResponder for BondingContractAlkane {
    fn context(&self) -> Result<Context> {
        use crate::mock_runtime::get_context;
        get_context().ok_or_else(|| anyhow!("No context available"))
    }
    
    fn execute(&self) -> Result<CallResponse> {
        Err(anyhow!("Use the declare_alkane macro instead"))
    }
}
