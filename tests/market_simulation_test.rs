use alkanes_runtime::runtime::{AlkaneResponder, Context, RuntimeContext};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

// Import the bonding contract
use bonding_contract::BondingContractAlkane;
use bonding_contract::BondingCurve;

// Mock runtime context for testing
struct MockRuntimeContext {
    storage: HashMap<String, Vec<u8>>,
    caller: AlkaneId,
    myself: AlkaneId,
    incoming_alkanes: AlkaneTransferParcel,
}

impl RuntimeContext for MockRuntimeContext {
    fn get_storage(&self, key: &[u8]) -> Vec<u8> {
        self.storage.get(&String::from_utf8_lossy(key).to_string())
            .cloned()
            .unwrap_or_default()
    }

    fn set_storage(&mut self, key: &[u8], value: Vec<u8>) {
        self.storage.insert(String::from_utf8_lossy(key).to_string(), value);
    }

    fn get_caller(&self) -> AlkaneId {
        self.caller.clone()
    }

    fn get_myself(&self) -> AlkaneId {
        self.myself.clone()
    }

    fn get_incoming_alkanes(&self) -> AlkaneTransferParcel {
        self.incoming_alkanes.clone()
    }
}

// Helper function to create a mock context
fn create_mock_context(
    caller: AlkaneId,
    myself: AlkaneId,
    incoming_alkanes: Vec<AlkaneTransfer>,
) -> MockRuntimeContext {
    MockRuntimeContext {
        storage: HashMap::new(),
        caller,
        myself,
        incoming_alkanes: AlkaneTransferParcel(incoming_alkanes),
    }
}

// Helper function to set the runtime context
fn set_runtime_context(context: &MockRuntimeContext) {
    // In a real test environment, this would set the context for the runtime
    // For this mock, we'll just pretend it works
}

// Struct to represent a market participant
struct MarketParticipant {
    id: AlkaneId,
    diesel_balance: u128,
    token_balance: u128,
}

impl MarketParticipant {
    fn new(id: AlkaneId, diesel_balance: u128) -> Self {
        Self {
            id,
            diesel_balance,
            token_balance: 0,
        }
    }
    
    fn buy_tokens(&mut self, contract: &BondingContractAlkane, contract_id: &AlkaneId, diesel_amount: u128) -> Result<u128> {
        // Check if the participant has enough diesel
        if diesel_amount > self.diesel_balance {
            return Err(anyhow!("Insufficient diesel balance"));
        }
        
        // Set up the context for buying
        let buy_context = create_mock_context(
            self.id.clone(),
            contract_id.clone(),
            vec![
                AlkaneTransfer {
                    id: AlkaneId { block: 2, tx: 0 }, // Diesel is [2, 0]
                    value: diesel_amount,
                },
            ],
        );
        set_runtime_context(&buy_context);
        
        // Call the buy function
        let buy_response = contract.buy(diesel_amount)?;
        
        // Update balances
        self.diesel_balance -= diesel_amount;
        let token_amount = buy_response.alkanes.0[0].value;
        self.token_balance += token_amount;
        
        Ok(token_amount)
    }
    
    fn sell_tokens(&mut self, contract: &BondingContractAlkane, contract_id: &AlkaneId, token_amount: u128) -> Result<u128> {
        // Check if the participant has enough tokens
        if token_amount > self.token_balance {
            return Err(anyhow!("Insufficient token balance"));
        }
        
        // Set up the context for selling
        let sell_context = create_mock_context(
            self.id.clone(),
            contract_id.clone(),
            vec![
                AlkaneTransfer {
                    id: contract_id.clone(),
                    value: token_amount,
                },
            ],
        );
        set_runtime_context(&sell_context);
        
        // Call the sell function
        let sell_response = contract.sell(token_amount)?;
        
        // Update balances
        self.token_balance -= token_amount;
        let diesel_amount = sell_response.alkanes.0[0].value;
        self.diesel_balance += diesel_amount;
        
        Ok(diesel_amount)
    }
    
    fn get_current_price(&self, contract: &BondingContractAlkane, contract_id: &AlkaneId) -> Result<u128> {
        // Set up the context for getting the price
        let price_context = create_mock_context(
            self.id.clone(),
            contract_id.clone(),
            vec![],
        );
        set_runtime_context(&price_context);
        
        // Call the get_current_price function
        let price_response = contract.get_current_price()?;
        
        // Parse the price
        let price_bytes = price_response.data;
        let price = u128::from_le_bytes(price_bytes.try_into().unwrap_or([0; 16]));
        
        Ok(price)
    }
}

// Struct to represent the market
struct Market {
    contract: BondingContractAlkane,
    contract_id: AlkaneId,
    participants: Vec<MarketParticipant>,
    initial_supply: u128,
    initial_reserve: u128,
}

impl Market {
    fn new(contract_id: AlkaneId, initial_supply: u128, initial_reserve: u128) -> Self {
        Self {
            contract: BondingContractAlkane::default(),
            contract_id,
            participants: Vec::new(),
            initial_supply,
            initial_reserve,
        }
    }
    
    fn initialize(&self, deployer: &AlkaneId) -> Result<()> {
        // Set up the context for initialization
        let init_context = create_mock_context(
            deployer.clone(),
            self.contract_id.clone(),
            vec![],
        );
        set_runtime_context(&init_context);
        
        // Initialize the contract
        let name = 0x424f4e44; // "BOND"
        let symbol = 0x424e44; // "BND"
        self.contract.initialize(name, symbol, self.initial_supply, self.initial_reserve)?;
        
        Ok(())
    }
    
    fn add_participant(&mut self, id: AlkaneId, diesel_balance: u128) {
        self.participants.push(MarketParticipant::new(id, diesel_balance));
    }
    
    fn get_participant(&mut self, index: usize) -> &mut MarketParticipant {
        &mut self.participants[index]
    }
    
    fn get_total_supply(&self) -> u128 {
        self.contract.total_supply()
    }
    
    fn get_reserve(&self) -> u128 {
        self.contract.reserve()
    }
    
    fn get_current_price(&self, participant_index: usize) -> Result<u128> {
        let participant = &self.participants[participant_index];
        participant.get_current_price(&self.contract, &self.contract_id)
    }
    
    fn print_market_state(&self) -> Result<()> {
        println!("Market State:");
        println!("  Total Supply: {}", self.get_total_supply());
        println!("  Reserve: {}", self.get_reserve());
        
        // Get the current price using the first participant
        if !self.participants.is_empty() {
            let price_context = create_mock_context(
                self.participants[0].id.clone(),
                self.contract_id.clone(),
                vec![],
            );
            set_runtime_context(&price_context);
            
            let price_response = self.contract.get_current_price()?;
            let price_bytes = price_response.data;
            let price = u128::from_le_bytes(price_bytes.try_into().unwrap_or([0; 16]));
            
            println!("  Current Price: {}", price);
        }
        
        println!("Participants:");
        for (i, participant) in self.participants.iter().enumerate() {
            println!("  Participant {}: Diesel={}, Tokens={}", i, participant.diesel_balance, participant.token_balance);
        }
        
        Ok(())
    }
}

#[test]
fn test_market_simulation() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    
    // Create participant addresses
    let participant1 = AlkaneId { block: 1, tx: 2 };
    let participant2 = AlkaneId { block: 1, tx: 3 };
    let participant3 = AlkaneId { block: 1, tx: 4 };
    let participant4 = AlkaneId { block: 1, tx: 5 };
    
    // Create the market
    let initial_supply = 1000000;
    let initial_reserve = 1000000;
    let mut market = Market::new(contract_id.clone(), initial_supply, initial_reserve);
    
    // Initialize the market
    market.initialize(&deployer)?;
    
    // Add participants
    market.add_participant(participant1.clone(), 100000); // 100k diesel
    market.add_participant(participant2.clone(), 200000); // 200k diesel
    market.add_participant(participant3.clone(), 50000);  // 50k diesel
    market.add_participant(participant4.clone(), 300000); // 300k diesel
    
    // Print initial market state
    println!("Initial Market State:");
    market.print_market_state()?;
    
    // Simulation Round 1: Participants buy tokens
    println!("\nRound 1: Participants buy tokens");
    
    // Participant 1 buys tokens
    let p1_diesel_amount = 10000;
    let p1_token_amount = market.get_participant(0).buy_tokens(&market.contract, &market.contract_id, p1_diesel_amount)?;
    println!("Participant 1 bought {} tokens with {} diesel", p1_token_amount, p1_diesel_amount);
    
    // Participant 2 buys tokens
    let p2_diesel_amount = 50000;
    let p2_token_amount = market.get_participant(1).buy_tokens(&market.contract, &market.contract_id, p2_diesel_amount)?;
    println!("Participant 2 bought {} tokens with {} diesel", p2_token_amount, p2_diesel_amount);
    
    // Participant 3 buys tokens
    let p3_diesel_amount = 5000;
    let p3_token_amount = market.get_participant(2).buy_tokens(&market.contract, &market.contract_id, p3_diesel_amount)?;
    println!("Participant 3 bought {} tokens with {} diesel", p3_token_amount, p3_diesel_amount);
    
    // Print market state after round 1
    println!("\nMarket State after Round 1:");
    market.print_market_state()?;
    
    // Simulation Round 2: Some participants sell tokens
    println!("\nRound 2: Some participants sell tokens");
    
    // Participant 1 sells half of their tokens
    let p1_sell_amount = p1_token_amount / 2;
    let p1_diesel_received = market.get_participant(0).sell_tokens(&market.contract, &market.contract_id, p1_sell_amount)?;
    println!("Participant 1 sold {} tokens for {} diesel", p1_sell_amount, p1_diesel_received);
    
    // Participant 3 sells all of their tokens
    let p3_sell_amount = p3_token_amount;
    let p3_diesel_received = market.get_participant(2).sell_tokens(&market.contract, &market.contract_id, p3_sell_amount)?;
    println!("Participant 3 sold {} tokens for {} diesel", p3_sell_amount, p3_diesel_received);
    
    // Print market state after round 2
    println!("\nMarket State after Round 2:");
    market.print_market_state()?;
    
    // Simulation Round 3: Participant 4 enters the market with a large buy
    println!("\nRound 3: Participant 4 enters the market with a large buy");
    
    // Get the price before the large buy
    let price_before = market.get_current_price(3)?;
    
    // Participant 4 buys tokens
    let p4_diesel_amount = 100000;
    let p4_token_amount = market.get_participant(3).buy_tokens(&market.contract, &market.contract_id, p4_diesel_amount)?;
    println!("Participant 4 bought {} tokens with {} diesel", p4_token_amount, p4_diesel_amount);
    
    // Get the price after the large buy
    let price_after = market.get_current_price(3)?;
    
    // Calculate price impact
    let price_impact = (price_after as f64 - price_before as f64) / price_before as f64 * 100.0;
    println!("Price impact: {}%", price_impact);
    
    // Print market state after round 3
    println!("\nMarket State after Round 3:");
    market.print_market_state()?;
    
    // Simulation Round 4: Participants react to the price change
    println!("\nRound 4: Participants react to the price change");
    
    // Participant 2 sells some tokens due to high price
    let p2_sell_amount = p2_token_amount / 4;
    let p2_diesel_received = market.get_participant(1).sell_tokens(&market.contract, &market.contract_id, p2_sell_amount)?;
    println!("Participant 2 sold {} tokens for {} diesel", p2_sell_amount, p2_diesel_received);
    
    // Participant 1 buys more tokens
    let p1_diesel_amount2 = 20000;
    let p1_token_amount2 = market.get_participant(0).buy_tokens(&market.contract, &market.contract_id, p1_diesel_amount2)?;
    println!("Participant 1 bought {} tokens with {} diesel", p1_token_amount2, p1_diesel_amount2);
    
    // Print final market state
    println!("\nFinal Market State:");
    market.print_market_state()?;
    
    // Verify that the market behaved as expected
    
    // 1. Total supply should be consistent with individual balances
    let total_participant_tokens = market.participants.iter().map(|p| p.token_balance).sum::<u128>();
    assert_eq!(market.get_total_supply(), initial_supply + total_participant_tokens);
    
    // 2. Reserve should be consistent with diesel spent and received
    let total_diesel_spent = market.participants.iter().map(|p| 
        if p.diesel_balance > 0 { 
            p.diesel_balance 
        } else { 
            0 
        }
    ).sum::<u128>();
    let expected_reserve = initial_reserve + (650000 - total_diesel_spent); // 650k is the total initial diesel
    assert_eq!(market.get_reserve(), expected_reserve);
    
    // 3. Price should be higher than initial price due to net buying
    let final_price = market.get_current_price(0)?;
    let initial_price = initial_reserve as f64 / (initial_supply as f64 * initial_supply as f64);
    assert!(final_price as f64 > initial_price);
    
    Ok(())
}

#[test]
fn test_market_volatility() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    
    // Create participant addresses
    let whale = AlkaneId { block: 1, tx: 2 };
    let trader1 = AlkaneId { block: 1, tx: 3 };
    let trader2 = AlkaneId { block: 1, tx: 4 };
    
    // Create the market with smaller initial values to amplify volatility
    let initial_supply = 100000;
    let initial_reserve = 100000;
    let mut market = Market::new(contract_id.clone(), initial_supply, initial_reserve);
    
    // Initialize the market
    market.initialize(&deployer)?;
    
    // Add participants
    market.add_participant(whale.clone(), 1000000);  // 1M diesel (whale)
    market.add_participant(trader1.clone(), 10000);  // 10k diesel
    market.add_participant(trader2.clone(), 10000);  // 10k diesel
    
    // Print initial market state
    println!("Initial Market State:");
    market.print_market_state()?;
    
    // Traders buy tokens at initial price
    println!("\nTraders buy tokens at initial price");
    
    let t1_diesel_amount = 5000;
    let t1_token_amount = market.get_participant(1).buy_tokens(&market.contract, &market.contract_id, t1_diesel_amount)?;
    println!("Trader 1 bought {} tokens with {} diesel", t1_token_amount, t1_diesel_amount);
    
    let t2_diesel_amount = 5000;
    let t2_token_amount = market.get_participant(2).buy_tokens(&market.contract, &market.contract_id, t2_diesel_amount)?;
    println!("Trader 2 bought {} tokens with {} diesel", t2_token_amount, t2_diesel_amount);
    
    // Record the price after traders buy
    let price_after_traders = market.get_current_price(0)?;
    println!("Price after traders buy: {}", price_after_traders);
    
    // Whale enters with a massive buy
    println!("\nWhale enters with a massive buy");
    
    let whale_diesel_amount = 500000; // 500k diesel
    let whale_token_amount = market.get_participant(0).buy_tokens(&market.contract, &market.contract_id, whale_diesel_amount)?;
    println!("Whale bought {} tokens with {} diesel", whale_token_amount, whale_diesel_amount);
    
    // Record the price after whale buys
    let price_after_whale_buy = market.get_current_price(0)?;
    println!("Price after whale buys: {}", price_after_whale_buy);
    
    // Calculate price impact
    let price_impact = (price_after_whale_buy as f64 - price_after_traders as f64) / price_after_traders as f64 * 100.0;
    println!("Price impact from whale buy: {}%", price_impact);
    
    // Traders react to the price spike
    println!("\nTraders react to the price spike");
    
    // Trader 1 sells all tokens
    let t1_sell_amount = t1_token_amount;
    let t1_diesel_received = market.get_participant(1).sell_tokens(&market.contract, &market.contract_id, t1_sell_amount)?;
    println!("Trader 1 sold {} tokens for {} diesel", t1_sell_amount, t1_diesel_received);
    println!("Trader 1 profit: {} diesel", t1_diesel_received as i128 - t1_diesel_amount as i128);
    
    // Trader 2 sells all tokens
    let t2_sell_amount = t2_token_amount;
    let t2_diesel_received = market.get_participant(2).sell_tokens(&market.contract, &market.contract_id, t2_sell_amount)?;
    println!("Trader 2 sold {} tokens for {} diesel", t2_sell_amount, t2_diesel_received);
    println!("Trader 2 profit: {} diesel", t2_diesel_received as i128 - t2_diesel_amount as i128);
    
    // Record the price after traders sell
    let price_after_traders_sell = market.get_current_price(0)?;
    println!("Price after traders sell: {}", price_after_traders_sell);
    
    // Whale exits with a massive sell
    println!("\nWhale exits with a massive sell");
    
    let whale_sell_amount = whale_token_amount;
    let whale_diesel_received = market.get_participant(0).sell_tokens(&market.contract, &market.contract_id, whale_sell_amount)?;
    println!("Whale sold {} tokens for {} diesel", whale_sell_amount, whale_diesel_received);
    println!("Whale loss: {} diesel", whale_diesel_received as i128 - whale_diesel_amount as i128);
    
    // Record the price after whale sells
    let price_after_whale_sell = market.get_current_price(0)?;
    println!("Price after whale sells: {}", price_after_whale_sell);
    
    // Print final market state
    println!("\nFinal Market State:");
    market.print_market_state()?;
    
    // Verify market behavior
    
    // 1. Price should spike when whale buys
    assert!(price_after_whale_buy > price_after_traders);
    
    // 2. Traders should profit from selling at higher price
    assert!(t1_diesel_received > t1_diesel_amount);
    assert!(t2_diesel_received > t2_diesel_amount);
    
    // 3. Whale should lose money due to slippage
    assert!(whale_diesel_received < whale_diesel_amount);
    
    // 4. Final price should be close to initial price
    let initial_price = initial_reserve as f64 / (initial_supply as f64 * initial_supply as f64);
    let price_diff = (price_after_whale_sell as f64 - initial_price).abs() / initial_price;
    assert!(price_diff < 0.1); // Within 10% of initial price
    
    Ok(())
}
