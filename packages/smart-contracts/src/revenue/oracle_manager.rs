use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
};

#[derive(SolidityType, Clone, Debug)]
pub struct OracleConfig {
    pub oracle_address: Address,
    pub is_active: bool,
    pub reliability_score: U256, // 0-100
    pub last_update: U256,
    pub data_source: String, // "spotify", "youtube", etc.
    pub update_frequency: U256, // seconds between updates
}

#[derive(SolidityType, Clone, Debug)]
pub struct RevenueData {
    pub project_id: U256,
    pub source: String,
    pub amount: U256,
    pub timestamp: U256,
    pub oracle_address: Address,
    pub verification_score: U256, // Consensus score from multiple oracles
    pub is_disputed: bool,
}

#[storage]
#[entrypoint]
pub struct OracleManager {
    // Oracle registry
    oracles: StorageMap<Address, OracleConfig>,
    source_oracles: StorageMap<String, StorageVec<Address>>, // source -> oracles
    oracle_count: StorageU256,
    
    // Revenue data tracking
    revenue_reports: StorageMap<U256, StorageMap<String, RevenueData>>, // project -> source -> data
    oracle_submissions: StorageMap<U256, StorageMap<String, StorageMap<Address, U256>>>, // project -> source -> oracle -> amount
    
    // Consensus mechanism
    min_oracles_required: StorageU256,
    consensus_threshold: StorageU256, // Percentage agreement required
    
    // Platform integration
    revenue_distributor: StorageAddress,
    platform_contract: StorageAddress,
    
    // Access control
    owner: StorageAddress,
    authorized_operators: StorageMap<Address, bool>,
    
    // Oracle performance tracking
    oracle_accuracy: StorageMap<Address, U256>, // Running accuracy score
    oracle_response_times: StorageMap<Address, U256>, // Average response time
    
    // Dispute handling
    disputed_reports: StorageMap<U256, bool>, // reportId -> disputed
    dispute_resolution_period: StorageU256,
    
    // Emergency controls
    paused: StorageBool,
}

#[public]
impl OracleManager {
    pub fn initialize(
        &mut self,
        revenue_distributor: Address,
        platform_contract: Address,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.revenue_distributor.set(revenue_distributor);
        self.platform_contract.set(platform_contract);
        
        // Set default parameters
        self.min_oracles_required.set(U256::from(3));
        self.consensus_threshold.set(U256::from(70)); // 70% agreement
        self.dispute_resolution_period.set(U256::from(48 * 3600)); // 48 hours
        
        Ok(())
    }

    pub fn register_oracle(
        &mut self,
        oracle_address: Address,
        data_source: String,
        update_frequency: U256,
    ) -> Result<()> {
        self.require_owner()?;
        
        require_valid_input(!oracle_address.is_zero(), "Invalid oracle address")?;
        require_valid_input(!data_source.is_empty(), "Data source required")?;
        
        let config = OracleConfig {
            oracle_address,
            is_active: true,
            reliability_score: U256::from(100), // Start with perfect score
            last_update: U256::from(0),
            data_source: data_source.clone(),
            update_frequency,
        };
        
        self.oracles.insert(oracle_address, config);
        self.source_oracles.get_mut(data_source).push(oracle_address);
        self.oracle_count.set(self.oracle_count.get() + U256::from(1));
        
        Ok(())
    }

    pub fn submit_revenue_data(
        &mut self,
        project_id: U256,
        source: String,
        amount: U256,
        timestamp: U256,
    ) -> Result<()> {
        self.require_not_paused()?;
        
        let oracle = msg::sender();
        let oracle_config = self.oracles.get(oracle);
        
        require_valid_input(!oracle_config.oracle_address.is_zero(), "Oracle not registered")?;
        require_valid_input(oracle_config.is_active, "Oracle not active")?;
        require_valid_input(oracle_config.data_source == source, "Invalid data source for oracle")?;
        
        // Store oracle submission
        self.oracle_submissions
            .get_mut(project_id)
            .get_mut(source.clone())
            .insert(oracle, amount);
        
        // Check if we have enough submissions for consensus
        let submissions = self.oracle_submissions.get(project_id).get(source.clone());
        let submission_count = self.count_submissions(&submissions);
        
        if submission_count >= self.min_oracles_required.get().as_usize() {
            self.process_consensus(project_id, source.clone())?;
        }
        
        Ok(())
    }

    pub fn process_consensus(&mut self, project_id: U256, source: String) -> Result<()> {
        let submissions = self.oracle_submissions.get(project_id).get(source.clone());
        let mut amounts = Vec::new();
        let mut oracles = Vec::new();
        
        // Collect all submissions
        for oracle_addr in self.get_source_oracles(&source) {
            let amount = submissions.get(oracle_addr);
            if amount > U256::from(0) {
                amounts.push(amount.as_u64());
                oracles.push(oracle_addr);
            }
        }
        
        if amounts.len() < self.min_oracles_required.get().as_usize() {
            return Ok(()); // Not enough submissions yet
        }
        
        // Calculate consensus
        let (consensus_amount, agreement_percentage) = self.calculate_consensus(&amounts);
        
        if agreement_percentage >= self.consensus_threshold.get().as_u64() {
            // Consensus reached
            let revenue_data = RevenueData {
                project_id,
                source: source.clone(),
                amount: U256::from(consensus_amount),
                timestamp: U256::from(block::timestamp()),
                oracle_address: Address::ZERO, // Consensus from multiple oracles
                verification_score: U256::from(agreement_percentage),
                is_disputed: false,
            };
            
            self.revenue_reports
                .get_mut(project_id)
                .insert(source.clone(), revenue_data);
            
            // Update oracle accuracy scores
            self.update_oracle_accuracy(&oracles, &amounts, consensus_amount);
            
            // Notify revenue distributor
            // In production, would call revenue distributor contract
            
        } else {
            // No consensus - flag for manual review
            // Could implement escalation mechanism here
        }
        
        Ok(())
    }

    pub fn dispute_revenue_report(
        &mut self,
        project_id: U256,
        source: String,
        evidence_uri: String,
    ) -> Result<()> {
        let disputer = msg::sender();
        
        // Check if report exists
        let mut revenue_data = self.revenue_reports.get(project_id).get(source.clone());
        require_valid_input(revenue_data.project_id != U256::from(0), "Revenue report not found")?;
        
        // Mark as disputed
        revenue_data.is_disputed = true;
        self.revenue_reports
            .get_mut(project_id)
            .insert(source, revenue_data);
        
        // Create dispute record (simplified)
        let report_id = project_id * U256::from(1000) + U256::from(source.len() as u64);
        self.disputed_reports.insert(report_id, true);
        
        Ok(())
    }

    // View functions
    pub fn get_revenue_data(&self, project_id: U256, source: String) -> Result<RevenueData> {
        let data = self.revenue_reports.get(project_id).get(source);
        require_valid_input(data.project_id != U256::from(0), "Revenue data not found")?;
        Ok(data)
    }

    pub fn get_oracle_config(&self, oracle: Address) -> Result<OracleConfig> {
        let config = self.oracles.get(oracle);
        require_valid_input(!config.oracle_address.is_zero(), "Oracle not found")?;
        Ok(config)
    }

    pub fn is_authorized_reporter(&self, reporter: Address) -> bool {
        let config = self.oracles.get(reporter);
        config.is_active
    }

    pub fn validate_revenue_claim(
        &self,
        project_id: U256,
        source: String,
        amount: U256,
        _proof: Vec<u8>,
    ) -> bool {
        let revenue_data = self.revenue_reports.get(project_id).get(source);
        if revenue_data.project_id == U256::from(0) {
            return false;
        }
        
        // Check if amount is within reasonable bounds of reported data
        let reported = revenue_data.amount.as_u64();
        let claimed = amount.as_u64();
        let tolerance = reported / 10; // 10% tolerance
        
        claimed >= reported.saturating_sub(tolerance) && claimed <= reported + tolerance
    }

    // Admin functions
    pub fn deactivate_oracle(&mut self, oracle: Address) -> Result<()> {
        self.require_owner()?;
        
        let mut config = self.oracles.get(oracle);
        require_valid_input(!config.oracle_address.is_zero(), "Oracle not found")?;
        
        config.is_active = false;
        self.oracles.insert(oracle, config);
        
        Ok(())
    }

    pub fn update_consensus_parameters(
        &mut self,
        min_oracles: U256,
        threshold: U256,
    ) -> Result<()> {
        self.require_owner()?;
        
        require_valid_input(min_oracles >= U256::from(2), "Need at least 2 oracles")?;
        require_valid_input(threshold >= U256::from(51) && threshold <= U256::from(100), "Invalid threshold")?;
        
        self.min_oracles_required.set(min_oracles);
        self.consensus_threshold.set(threshold);
        
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        self.require_owner()?;
        self.paused.set(true);
        Ok(())
    }
}

// Internal helper functions
impl OracleManager {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_not_paused(&self) -> Result<()> {
        require_valid_input(!self.paused.get(), "Contract is paused")
    }

    fn count_submissions(&self, submissions: &StorageMap<Address, U256>) -> usize {
        // In a real implementation, would need to iterate through submissions
        // For now, returning a placeholder
        3 // Assumes we have 3 submissions
    }

    fn get_source_oracles(&self, source: &str) -> Vec<Address> {
        let oracles = self.source_oracles.get(source.to_string());
        let mut result = Vec::new();
        
        for i in 0..oracles.len() {
            if let Some(oracle) = oracles.get(i) {
                result.push(oracle);
            }
        }
        
        result
    }

    fn calculate_consensus(&self, amounts: &[u64]) -> (u64, u64) {
        if amounts.is_empty() {
            return (0, 0);
        }
        
        // Simple median consensus
        let mut sorted = amounts.to_vec();
        sorted.sort();
        
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2
        } else {
            sorted[sorted.len() / 2]
        };
        
        // Calculate agreement percentage (simplified)
        let tolerance = median / 10; // 10% tolerance
        let agreeing = amounts
            .iter()
            .filter(|&&amount| {
                amount >= median.saturating_sub(tolerance) && amount <= median + tolerance
            })
            .count();
        
        let agreement_percentage = (agreeing * 100) / amounts.len();
        
        (median, agreement_percentage as u64)
    }

    fn update_oracle_accuracy(&mut self, oracles: &[Address], amounts: &[u64], consensus: u64) {
        let tolerance = consensus / 20; // 5% tolerance for accuracy
        
        for (i, &oracle) in oracles.iter().enumerate() {
            if i < amounts.len() {
                let amount = amounts[i];
                let is_accurate = amount >= consensus.saturating_sub(tolerance) 
                    && amount <= consensus + tolerance;
                
                let current_accuracy = self.oracle_accuracy.get(oracle);
                let new_accuracy = if is_accurate {
                    core::cmp::min(current_accuracy.as_u64() + 1, 100)
                } else {
                    current_accuracy.as_u64().saturating_sub(2)
                };
                
                self.oracle_accuracy.insert(oracle, U256::from(new_accuracy));
            }
        }
    }
}