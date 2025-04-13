use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, message::MessageDispatch};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::response::CallResponse;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::context::Context;
use metashrew_support::index_pointer::KeyValuePointer;
use metashrew_support::compat::to_arraybuffer_layout;
use anyhow::{anyhow, Result};
use std::sync::Arc;

// Import our bonding curve implementation
mod bonding_curve;
// Re-export the BondingCurve struct to make it public
pub use bonding_curve::BondingCurve;

/// BondingContract trait defines the interface for bonding curve functionality
pub trait BondingContract: AlkaneResponder {
    /// Buy alkane with diesel
    fn buy_alkane(&mut self, diesel_amount: u128) -> Result<CallResponse>;
    
    /// Sell alkane for diesel
    fn sell_alkane(&mut self, alkane_amount: u128) -> Result<CallResponse>;
    
    /// Get the current reserve of diesel
    fn diesel_reserve(&self) -> u128;
    
    /// Get the current supply of alkane
    fn alkane_supply(&self) -> u128;
    
    /// Get the current price of alkane in terms of diesel
    fn current_price(&self) -> Result<CallResponse>;
}

/// BondingContractAlkane implements a bonding curve contract
#[derive(Default)]
pub struct BondingContractAlkane {
    /// The bonding curve implementation
    bonding_curve: Option<BondingCurve>,
}

/// Message enum for opcode-based dispatch
#[derive(MessageDispatch)]
enum BondingContractAlkaneMessage {
    /// Initialize the contract
    #[opcode(0)]
    InitContract {
        name: u128,
        symbol: u128,
        k_factor: u128,
        n_exponent: u128,
        initial_diesel_reserve: u128,
    },
    
    /// Buy alkane with diesel
    #[opcode(1)]
    BuyAlkane {
        diesel_amount: u128,
    },
    
    /// Sell alkane for diesel
    #[opcode(2)]
    SellAlkane {
        alkane_amount: u128,
    },
    
    /// Get the current price of alkane in terms of diesel
    #[opcode(3)]
    #[returns(CallResponse)]
    GetCurrentPrice,
    
    /// Get the amount of alkane that can be received for a specific amount of diesel
    #[opcode(4)]
    #[returns(CallResponse)]
    GetBuyAmount {
        diesel_amount: u128,
    },
    
    /// Get the amount of diesel that can be received for a specific amount of alkane
    #[opcode(5)]
    #[returns(CallResponse)]
    GetSellAmount {
        alkane_amount: u128,
    },
    
    /// Get the name of the token
    #[opcode(99)]
    #[returns(CallResponse)]
    GetName,
    
    /// Get the symbol of the token
    #[opcode(100)]
    #[returns(CallResponse)]
    GetSymbol,
    
    /// Get the reserve of diesel
    #[opcode(101)]
    #[returns(CallResponse)]
    GetDieselReserve,
    
    /// Get the supply of alkane
    #[opcode(102)]
    #[returns(CallResponse)]
    GetAlkaneSupply,
    
    /// Get the k factor
    #[opcode(103)]
    #[returns(CallResponse)]
    GetKFactor,
    
    /// Get the n exponent
    #[opcode(104)]
    #[returns(CallResponse)]
    GetNExponent,
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
    pub fn current_price(&self) -> Result<CallResponse> {
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
        
        Ok(response)
    }
    
    /// Buy alkane with diesel
    pub fn buy_alkane(&self, _diesel_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();
        
        // Check if diesel was sent
        let mut diesel_received = 0;
        for transfer in &context.incoming_alkanes.0 {
            if transfer.id == (AlkaneId { block: 2, tx: 0 }) { // Diesel is [2, 0]
                diesel_received += transfer.value;
            }
        }
        
        if diesel_received == 0 {
            return Err(anyhow!("no diesel received"));
        }
        
        // Calculate the amount of alkane to mint
        let buy_response = self.get_buy_amount(diesel_received)?;
        let alkane_amount = u128::from_le_bytes(buy_response.data.try_into().unwrap());
        
        if alkane_amount == 0 {
            return Err(anyhow!("insufficient output amount"));
        }
        
        // Update the reserves
        let diesel_reserve = self.diesel_reserve();
        let alkane_supply = self.alkane_supply();
        
        self.set_diesel_reserve(diesel_reserve + diesel_received);
        self.set_alkane_supply(alkane_supply + alkane_amount);
        
        // Add the alkane to the response
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: alkane_amount,
        });
        
        Ok(response)
    }
    
    /// Sell alkane for diesel
    pub fn sell_alkane(&self, _alkane_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::default();
        
        // Check if alkane was sent
        let mut alkane_received = 0;
        for transfer in &context.incoming_alkanes.0 {
            if transfer.id == context.myself {
                alkane_received += transfer.value;
            }
        }
        
        if alkane_received == 0 {
            return Err(anyhow!("no alkane received"));
        }
        
        // Calculate the amount of diesel to return
        let sell_response = self.get_sell_amount(alkane_received)?;
        let diesel_amount = u128::from_le_bytes(sell_response.data.try_into().unwrap());
        
        if diesel_amount == 0 {
            return Err(anyhow!("insufficient output amount"));
        }
        
        // Update the reserves
        let diesel_reserve = self.diesel_reserve();
        let alkane_supply = self.alkane_supply();
        
        if diesel_amount > diesel_reserve {
            return Err(anyhow!("insufficient diesel reserve"));
        }
        
        self.set_diesel_reserve(diesel_reserve - diesel_amount);
        self.set_alkane_supply(alkane_supply - alkane_received);
        
        // Add the diesel to the response
        response.alkanes.0.push(AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel is [2, 0]
            value: diesel_amount,
        });
        
        Ok(response)
    }
    
    /// Get the name of the token
    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.name().into_bytes();
        
        Ok(response)
    }
    
    /// Get the symbol of the token
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.symbol().into_bytes();
        
        Ok(response)
    }
    
    /// Get the diesel reserve
    fn get_diesel_reserve(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.diesel_reserve().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the alkane supply
    fn get_alkane_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.alkane_supply().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the k factor
    fn get_k_factor(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.k_factor().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the n exponent
    fn get_n_exponent(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        response.data = self.n_exponent().to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the current price of alkane in terms of diesel
    fn get_current_price(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let curve = self.get_bonding_curve();
        let price = curve.get_current_price();
        
        response.data = price.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    /// Get the amount of alkane that can be received for a specific amount of diesel
    pub fn get_buy_amount_response(&self, diesel_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let buy_response = self.get_buy_amount(diesel_amount)?;
        response.data = buy_response.data;
        
        Ok(response)
    }
    
    /// Get the amount of diesel that can be received for a specific amount of alkane
    pub fn get_sell_amount_response(&self, alkane_amount: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let sell_response = self.get_sell_amount(alkane_amount)?;
        response.data = sell_response.data;
        
        Ok(response)
    }
}

// Implementation of the BondingContract trait
impl BondingContract for BondingContractAlkane {
    fn buy_alkane(&mut self, diesel_amount: u128) -> Result<CallResponse> {
        // Call the implementation directly to avoid recursion
        let context = self.context()?;
        let mut response = CallResponse::default();
        
        // Calculate the amount of alkane to mint
        let buy_response = self.get_buy_amount(diesel_amount)?;
        let alkane_amount = u128::from_le_bytes(buy_response.data.try_into().unwrap());
        
        if alkane_amount == 0 {
            return Err(anyhow!("insufficient output amount"));
        }
        
        // Update the reserves
        let diesel_reserve = self.diesel_reserve();
        let alkane_supply = self.alkane_supply();
        
        self.set_diesel_reserve(diesel_reserve + diesel_amount);
        self.set_alkane_supply(alkane_supply + alkane_amount);
        
        // Add the alkane to the response
        response.alkanes.0.push(AlkaneTransfer {
            id: context.myself.clone(),
            value: alkane_amount,
        });
        
        Ok(response)
    }
    
    fn sell_alkane(&mut self, alkane_amount: u128) -> Result<CallResponse> {
        // Call the implementation directly to avoid recursion
        let context = self.context()?;
        let mut response = CallResponse::default();
        
        // Calculate the amount of diesel to return
        let sell_response = self.get_sell_amount(alkane_amount)?;
        let diesel_amount = u128::from_le_bytes(sell_response.data.try_into().unwrap());
        
        if diesel_amount == 0 {
            return Err(anyhow!("insufficient output amount"));
        }
        
        // Update the reserves
        let diesel_reserve = self.diesel_reserve();
        let alkane_supply = self.alkane_supply();
        
        if diesel_amount > diesel_reserve {
            return Err(anyhow!("insufficient diesel reserve"));
        }
        
        self.set_diesel_reserve(diesel_reserve - diesel_amount);
        self.set_alkane_supply(alkane_supply - alkane_amount);
        
        // Add the diesel to the response
        response.alkanes.0.push(AlkaneTransfer {
            id: AlkaneId { block: 2, tx: 0 }, // Diesel is [2, 0]
            value: diesel_amount,
        });
        
        Ok(response)
    }
    
    fn diesel_reserve(&self) -> u128 {
        self.diesel_reserve_pointer().get_value::<u128>()
    }
    
    fn alkane_supply(&self) -> u128 {
        self.alkane_supply_pointer().get_value::<u128>()
    }
    
    fn current_price(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let curve = self.get_bonding_curve();
        let price = curve.get_current_price();
        
        response.data = price.to_le_bytes().to_vec();
        
        Ok(response)
    }
}

// Import the mock context module
pub mod mock_context;

impl AlkaneResponder for BondingContractAlkane {
    fn context(&self) -> Result<Context> {
        // Use the mock context if available
        if let Some(context) = mock_context::get_mock_context() {
            return Ok(context);
        }
        
        // In non-test mode or if no mock context is set
        Ok(Context::default())
    }
    
    fn execute(&self) -> Result<CallResponse> {
        // This method should not be called directly when using MessageDispatch
        Err(anyhow!("Use the declare_alkane macro instead"))
    }
}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for BondingContractAlkane {
        type Message = BondingContractAlkaneMessage;
    }
}

// Helper function to trim a u128 value to a String by removing trailing zeros
fn trim(v: u128) -> String {
    String::from_utf8(
        v.to_le_bytes()
            .into_iter()
            .fold(Vec::<u8>::new(), |mut r, v| {
                if v != 0 {
                    r.push(v)
                }
                r
            }),
    )
    .unwrap_or_default()
}

// Include test modules
#[cfg(test)]
mod tests;

#[cfg(test)]
mod e2e_tests_updated;

#[cfg(test)]
mod mock_runtime;

#[cfg(test)]
mod additional_tests;

#[cfg(test)]
mod coverage_tests;

#[cfg(test)]
mod penetration_tests;

#[cfg(test)]
mod extreme_value_tests;

#[cfg(test)]
mod precision_tests;

// Include mock storage implementation
mod mock_storage;
