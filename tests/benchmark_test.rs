use alkanes_runtime::runtime::{AlkaneResponder, Context, RuntimeContext};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::cell::RefCell;
use std::time::{Duration, Instant};

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

// Thread-local storage for the runtime context
thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<MockRuntimeContext>> = RefCell::new(None);
}

// Helper function to set the runtime context
fn set_runtime_context(context: &MockRuntimeContext) {
    // Clone the context and store it in thread-local storage
    let context_clone = MockRuntimeContext {
        storage: context.storage.clone(),
        caller: context.caller.clone(),
        myself: context.myself.clone(),
        incoming_alkanes: context.incoming_alkanes.clone(),
    };
    
    CURRENT_CONTEXT.with(|cell| {
        *cell.borrow_mut() = Some(context_clone);
    });
}

// Helper function to get the current runtime context
fn get_runtime_context() -> Option<MockRuntimeContext> {
    CURRENT_CONTEXT.with(|cell| {
        cell.borrow().clone()
    })
}

// Benchmark struct to track performance metrics
struct Benchmark {
    name: String,
    operation_count: usize,
    total_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
}

impl Benchmark {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            operation_count: 0,
            total_duration: Duration::new(0, 0),
            min_duration: Duration::new(u64::MAX, 0),
            max_duration: Duration::new(0, 0),
        }
    }
    
    fn record(&mut self, duration: Duration) {
        self.operation_count += 1;
        self.total_duration += duration;
        
        if duration < self.min_duration {
            self.min_duration = duration;
        }
        
        if duration > self.max_duration {
            self.max_duration = duration;
        }
    }
    
    fn average_duration(&self) -> Duration {
        if self.operation_count == 0 {
            return Duration::new(0, 0);
        }
        
        Duration::from_nanos((self.total_duration.as_nanos() / self.operation_count as u128) as u64)
    }
    
    fn print_results(&self) {
        println!("Benchmark: {}", self.name);
        println!("  Operations: {}", self.operation_count);
        println!("  Total Duration: {:?}", self.total_duration);
        println!("  Average Duration: {:?}", self.average_duration());
        println!("  Min Duration: {:?}", self.min_duration);
        println!("  Max Duration: {:?}", self.max_duration);
        println!("  Operations per second: {:.2}", self.operations_per_second());
    }
    
    fn operations_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() == 0.0 {
            return 0.0;
        }
        
        self.operation_count as f64 / self.total_duration.as_secs_f64()
    }
}

#[test]
fn benchmark_buy_operations() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    let user = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 }; // Diesel is [2, 0]

    // Create a new bonding contract
    let contract = BondingContractAlkane::default();

    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;

    // Set up the context for initialization
    let init_context = create_mock_context(
        deployer.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&init_context);

    // Call the initialize function
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Benchmark small buy operations
    let mut small_buy_benchmark = Benchmark::new("Small Buy Operations (100 diesel)");
    let small_amount = 100;
    
    for _ in 0..100 {
        // Set up the context for buying
        let buy_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![
                AlkaneTransfer {
                    id: diesel_id.clone(),
                    value: small_amount,
                },
            ],
        );
        set_runtime_context(&buy_context);
        
        // Measure the time it takes to execute the buy operation
        let start = Instant::now();
        contract.buy(small_amount)?;
        let duration = start.elapsed();
        
        // Record the benchmark
        small_buy_benchmark.record(duration);
    }
    
    // Benchmark medium buy operations
    let mut medium_buy_benchmark = Benchmark::new("Medium Buy Operations (1,000 diesel)");
    let medium_amount = 1000;
    
    for _ in 0..100 {
        // Set up the context for buying
        let buy_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![
                AlkaneTransfer {
                    id: diesel_id.clone(),
                    value: medium_amount,
                },
            ],
        );
        set_runtime_context(&buy_context);
        
        // Measure the time it takes to execute the buy operation
        let start = Instant::now();
        contract.buy(medium_amount)?;
        let duration = start.elapsed();
        
        // Record the benchmark
        medium_buy_benchmark.record(duration);
    }
    
    // Benchmark large buy operations
    let mut large_buy_benchmark = Benchmark::new("Large Buy Operations (10,000 diesel)");
    let large_amount = 10000;
    
    for _ in 0..100 {
        // Set up the context for buying
        let buy_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![
                AlkaneTransfer {
                    id: diesel_id.clone(),
                    value: large_amount,
                },
            ],
        );
        set_runtime_context(&buy_context);
        
        // Measure the time it takes to execute the buy operation
        let start = Instant::now();
        contract.buy(large_amount)?;
        let duration = start.elapsed();
        
        // Record the benchmark
        large_buy_benchmark.record(duration);
    }
    
    // Print benchmark results
    println!("\nBuy Operation Benchmarks:");
    small_buy_benchmark.print_results();
    medium_buy_benchmark.print_results();
    large_buy_benchmark.print_results();
    
    Ok(())
}

#[test]
fn benchmark_sell_operations() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    let user = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 }; // Diesel is [2, 0]

    // Create a new bonding contract
    let contract = BondingContractAlkane::default();

    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;

    // Set up the context for initialization
    let init_context = create_mock_context(
        deployer.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&init_context);

    // Call the initialize function
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // First, buy a large amount of tokens to sell
    let buy_amount = 1000000;
    let buy_context = create_mock_context(
        user.clone(),
        contract_id.clone(),
        vec![
            AlkaneTransfer {
                id: diesel_id.clone(),
                value: buy_amount,
            },
        ],
    );
    set_runtime_context(&buy_context);
    
    let buy_response = contract.buy(buy_amount)?;
    let token_amount = buy_response.alkanes.0[0].value;
    
    // Benchmark small sell operations
    let mut small_sell_benchmark = Benchmark::new("Small Sell Operations (0.1% of tokens)");
    let small_amount = token_amount / 1000;
    
    for _ in 0..100 {
        // Set up the context for selling
        let sell_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![
                AlkaneTransfer {
                    id: contract_id.clone(),
                    value: small_amount,
                },
            ],
        );
        set_runtime_context(&sell_context);
        
        // Measure the time it takes to execute the sell operation
        let start = Instant::now();
        contract.sell(small_amount)?;
        let duration = start.elapsed();
        
        // Record the benchmark
        small_sell_benchmark.record(duration);
    }
    
    // Benchmark medium sell operations
    let mut medium_sell_benchmark = Benchmark::new("Medium Sell Operations (1% of tokens)");
    let medium_amount = token_amount / 100;
    
    for _ in 0..100 {
        // Set up the context for selling
        let sell_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![
                AlkaneTransfer {
                    id: contract_id.clone(),
                    value: medium_amount,
                },
            ],
        );
        set_runtime_context(&sell_context);
        
        // Measure the time it takes to execute the sell operation
        let start = Instant::now();
        contract.sell(medium_amount)?;
        let duration = start.elapsed();
        
        // Record the benchmark
        medium_sell_benchmark.record(duration);
    }
    
    // Benchmark large sell operations
    let mut large_sell_benchmark = Benchmark::new("Large Sell Operations (10% of tokens)");
    let large_amount = token_amount / 10;
    
    for _ in 0..10 { // Fewer iterations for large sells to avoid running out of tokens
        // Set up the context for selling
        let sell_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![
                AlkaneTransfer {
                    id: contract_id.clone(),
                    value: large_amount,
                },
            ],
        );
        set_runtime_context(&sell_context);
        
        // Measure the time it takes to execute the sell operation
        let start = Instant::now();
        contract.sell(large_amount)?;
        let duration = start.elapsed();
        
        // Record the benchmark
        large_sell_benchmark.record(duration);
    }
    
    // Print benchmark results
    println!("\nSell Operation Benchmarks:");
    small_sell_benchmark.print_results();
    medium_sell_benchmark.print_results();
    large_sell_benchmark.print_results();
    
    Ok(())
}

#[test]
fn benchmark_query_operations() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    let user = AlkaneId { block: 1, tx: 2 };

    // Create a new bonding contract
    let contract = BondingContractAlkane::default();

    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;

    // Set up the context for initialization
    let init_context = create_mock_context(
        deployer.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&init_context);

    // Call the initialize function
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Benchmark get_current_price operations
    let mut current_price_benchmark = Benchmark::new("Get Current Price");
    
    for _ in 0..1000 {
        // Set up the context for getting the price
        let price_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![],
        );
        set_runtime_context(&price_context);
        
        // Measure the time it takes to execute the get_current_price operation
        let start = Instant::now();
        contract.get_current_price()?;
        let duration = start.elapsed();
        
        // Record the benchmark
        current_price_benchmark.record(duration);
    }
    
    // Benchmark get_buy_price operations
    let mut buy_price_benchmark = Benchmark::new("Get Buy Price");
    
    for _ in 0..1000 {
        // Set up the context for getting the buy price
        let price_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![],
        );
        set_runtime_context(&price_context);
        
        // Measure the time it takes to execute the get_buy_price operation
        let start = Instant::now();
        contract.get_buy_price(1000)?;
        let duration = start.elapsed();
        
        // Record the benchmark
        buy_price_benchmark.record(duration);
    }
    
    // Benchmark get_sell_price operations
    let mut sell_price_benchmark = Benchmark::new("Get Sell Price");
    
    for _ in 0..1000 {
        // Set up the context for getting the sell price
        let price_context = create_mock_context(
            user.clone(),
            contract_id.clone(),
            vec![],
        );
        set_runtime_context(&price_context);
        
        // Measure the time it takes to execute the get_sell_price operation
        let start = Instant::now();
        contract.get_sell_price(1000)?;
        let duration = start.elapsed();
        
        // Record the benchmark
        sell_price_benchmark.record(duration);
    }
    
    // Print benchmark results
    println!("\nQuery Operation Benchmarks:");
    current_price_benchmark.print_results();
    buy_price_benchmark.print_results();
    sell_price_benchmark.print_results();
    
    Ok(())
}

#[test]
fn benchmark_mixed_operations() -> Result<()> {
    // Create contract addresses
    let deployer = AlkaneId { block: 1, tx: 1 };
    let contract_id = AlkaneId { block: 2, tx: 1 };
    let user = AlkaneId { block: 1, tx: 2 };
    let diesel_id = AlkaneId { block: 2, tx: 0 }; // Diesel is [2, 0]

    // Create a new bonding contract
    let contract = BondingContractAlkane::default();

    // Initialize the contract
    let name = 0x424f4e44; // "BOND"
    let symbol = 0x424e44; // "BND"
    let initial_supply = 1000000;
    let initial_reserve = 1000000;

    // Set up the context for initialization
    let init_context = create_mock_context(
        deployer.clone(),
        contract_id.clone(),
        vec![],
    );
    set_runtime_context(&init_context);

    // Call the initialize function
    contract.initialize(name, symbol, initial_supply, initial_reserve)?;
    
    // Benchmark mixed operations (buy, sell, query)
    let mut mixed_benchmark = Benchmark::new("Mixed Operations");
    let mut token_balance = 0;
    
    for i in 0..300 {
        // Every 3rd operation is a buy
        if i % 3 == 0 {
            let buy_amount = 1000;
            
            // Set up the context for buying
            let buy_context = create_mock_context(
                user.clone(),
                contract_id.clone(),
                vec![
                    AlkaneTransfer {
                        id: diesel_id.clone(),
                        value: buy_amount,
                    },
                ],
            );
            set_runtime_context(&buy_context);
            
            // Measure the time it takes to execute the buy operation
            let start = Instant::now();
            let buy_response = contract.buy(buy_amount)?;
            let duration = start.elapsed();
            
            // Update token balance
            token_balance += buy_response.alkanes.0[0].value;
            
            // Record the benchmark
            mixed_benchmark.record(duration);
        }
        // Every 3rd + 1 operation is a sell (if we have tokens)
        else if i % 3 == 1 && token_balance > 0 {
            let sell_amount = token_balance / 10;
            if sell_amount == 0 {
                continue;
            }
            
            // Set up the context for selling
            let sell_context = create_mock_context(
                user.clone(),
                contract_id.clone(),
                vec![
                    AlkaneTransfer {
                        id: contract_id.clone(),
                        value: sell_amount,
                    },
                ],
            );
            set_runtime_context(&sell_context);
            
            // Measure the time it takes to execute the sell operation
            let start = Instant::now();
            contract.sell(sell_amount)?;
            let duration = start.elapsed();
            
            // Update token balance
            token_balance -= sell_amount;
            
            // Record the benchmark
            mixed_benchmark.record(duration);
        }
        // Every 3rd + 2 operation is a query
        else {
            // Set up the context for getting the price
            let price_context = create_mock_context(
                user.clone(),
                contract_id.clone(),
                vec![],
            );
            set_runtime_context(&price_context);
            
            // Measure the time it takes to execute the get_current_price operation
            let start = Instant::now();
            contract.get_current_price()?;
            let duration = start.elapsed();
            
            // Record the benchmark
            mixed_benchmark.record(duration);
        }
    }
    
    // Print benchmark results
    println!("\nMixed Operation Benchmark:");
    mixed_benchmark.print_results();
    
    Ok(())
}
