use alloy_primitives::{Address, U256, FixedBytes};
use afrocreate_contracts::{AfroCreatePlatform, types::*};
use std::collections::HashMap;

pub struct TestContext {
    pub platform: AfroCreatePlatform,
    pub test_accounts: Vec<Address>,
    pub ens_registry: Address,
    pub current_timestamp: u64,
}

impl TestContext {
    pub fn new() -> Self {
        let mut platform = AfroCreatePlatform::default();
        let test_accounts = generate_test_accounts(10);
        let ens_registry = test_accounts[0];
        
        // Initialize platform
        platform.initialize(
            ens_registry,
            U256::from(1000), // min funding 1000 wei
            U256::from(90), // max duration 90 days
        ).expect("Platform initialization failed");
        
        Self {
            platform,
            test_accounts,
            ens_registry,
            current_timestamp: 1625097600, // July 1, 2021
        }
    }
    
    pub fn creator(&self) -> Address {
        self.test_accounts[1]
    }
    
    pub fn backer(&self) -> Address {
        self.test_accounts[2]
    }
    
    pub fn validator(&self) -> Address {
        self.test_accounts[3]
    }
    
    pub fn admin(&self) -> Address {
        self.test_accounts[4]
    }
    
    pub fn advance_time(&mut self, seconds: u64) {
        self.current_timestamp += seconds;
    }
    
    pub fn register_test_creator(&mut self) -> Result<U256, String> {
        self.platform.register_creator(
            "testcreator".to_string(),
            "Nigerian".to_string()
        ).map_err(|e| format!("Creator registration failed: {:?}", e))
    }
    
    pub fn create_test_project(&mut self) -> Result<U256, String> {
        self.platform.create_project(
            "Test Music Album".to_string(),
            "A traditional Nigerian music album".to_string(),
            "Music".to_string(),
            U256::from(10000), // 10,000 wei target
            U256::from(30), // 30 days duration
            "QmTestHash123".to_string()
        ).map_err(|e| format!("Project creation failed: {:?}", e))
    }
}

pub fn generate_test_accounts(count: usize) -> Vec<Address> {
    (0..count)
        .map(|i| {
            let mut bytes = [0u8; 20];
            bytes[19] = i as u8;
            Address::from(bytes)
        })
        .collect()
}

pub fn assert_event_emitted<T: std::fmt::Debug>(expected_event: T, actual_events: &[T]) 
where 
    T: PartialEq 
{
    assert!(
        actual_events.contains(&expected_event),
        "Expected event {:?} not found in {:?}",
        expected_event,
        actual_events
    );
}

pub struct GasMeter {
    measurements: HashMap<String, u64>,
}

impl GasMeter {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
        }
    }
    
    pub fn measure<F>(&mut self, operation: &str, f: F) -> u64 
    where 
        F: FnOnce()
    {
        let start_gas = get_gas_used();
        f();
        let end_gas = get_gas_used();
        let gas_consumed = end_gas - start_gas;
        
        self.measurements.insert(operation.to_string(), gas_consumed);
        gas_consumed
    }
    
    pub fn get_measurement(&self, operation: &str) -> Option<u64> {
        self.measurements.get(operation).copied()
    }
    
    pub fn assert_gas_limit(&self, operation: &str, max_gas: u64) {
        if let Some(gas_used) = self.get_measurement(operation) {
            assert!(
                gas_used <= max_gas,
                "Operation '{}' used {} gas, exceeding limit of {}",
                operation, gas_used, max_gas
            );
        } else {
            panic!("No gas measurement found for operation '{}'", operation);
        }
    }
    
    pub fn print_report(&self) {
        println!("\n=== Gas Usage Report ===");
        let mut operations: Vec<_> = self.measurements.iter().collect();
        operations.sort_by_key(|(_, gas)| *gas);
        operations.reverse();
        
        for (operation, gas) in operations {
            println!("{}: {} gas", operation, gas);
        }
        println!("========================\n");
    }
}

// Mock gas measurement - in real tests, this would use actual gas measurement
fn get_gas_used() -> u64 {
    // This is a placeholder - in real Stylus tests, you'd measure actual gas
    42000
}

pub fn expect_error<T, E>(result: Result<T, E>, expected_error: &str) 
where 
    E: std::fmt::Debug
{
    match result {
        Ok(_) => panic!("Expected error '{}' but operation succeeded", expected_error),
        Err(e) => {
            let error_string = format!("{:?}", e);
            assert!(
                error_string.contains(expected_error),
                "Expected error containing '{}' but got '{}'",
                expected_error,
                error_string
            );
        }
    }
}

pub fn assert_within_range(actual: U256, expected: U256, tolerance_percent: u8) {
    let tolerance = expected * U256::from(tolerance_percent) / U256::from(100);
    let min_val = expected - tolerance;
    let max_val = expected + tolerance;
    
    assert!(
        actual >= min_val && actual <= max_val,
        "Value {} is not within {}% of expected value {} (range: {}-{})",
        actual, tolerance_percent, expected, min_val, max_val
    );
}

#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $tolerance:expr) => {
        assert_within_range($left, $right, $tolerance);
    };
}

pub fn create_mock_creator_profile(address: Address) -> CreatorProfile {
    CreatorProfile {
        creator_address: address,
        ens_name: "mocktest.afrocreate.eth".to_string(),
        cultural_background: "Ghanaian".to_string(),
        reputation_score: U256::from(100),
        projects_created: U256::from(0),
        total_funding_raised: U256::from(0),
        is_verified: false,
        registration_timestamp: U256::from(1625097600),
    }
}

pub fn create_mock_project_info(project_id: U256, creator: Address) -> ProjectInfo {
    ProjectInfo {
        project_id,
        creator,
        title: "Mock Project".to_string(),
        description: "A test project for cultural creators".to_string(),
        cultural_category: "Music".to_string(),
        funding_target: U256::from(5000),
        funding_raised: U256::from(0),
        deadline: U256::from(1625097600 + 30 * 86400), // 30 days from timestamp
        status: 0, // Active
        validation_status: 0, // Pending
        validation_score: U256::from(0),
        metadata_uri: "QmMockHash".to_string(),
    }
}