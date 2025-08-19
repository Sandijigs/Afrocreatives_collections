use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
    interfaces::{IRevenueOracle, ISuperfluid, IRevenueShareNFT},
    RevenueInfo, DistributionEvent,
};

#[derive(SolidityType, Clone, Debug)]
pub struct RevenueSource {
    pub source_name: String,
    pub oracle_address: Address,
    pub is_active: bool,
    pub verification_required: bool,
    pub last_update_timestamp: U256,
    pub total_revenue_reported: U256,
}

#[derive(SolidityType, Clone, Debug)]
pub struct RevenueBreakdown {
    pub project_id: U256,
    pub total_revenue: U256,
    pub creator_share: U256,
    pub community_share: U256,
    pub platform_fee: U256,
    pub last_distribution: U256,
}

#[derive(SolidityType, Clone, Debug)]
pub struct StreamingRevenue {
    pub project_id: U256,
    pub token_address: Address,
    pub flow_rate: i128, // Superfluid flow rate (tokens per second)
    pub total_streamed: U256,
    pub last_update_timestamp: U256,
    pub is_active: bool,
}

#[storage]
#[entrypoint]
pub struct RevenueDistributor {
    // Revenue tracking per project
    project_revenue: StorageMap<U256, RevenueInfo>,
    project_revenue_sources: StorageMap<U256, StorageMap<String, U256>>, // project -> (source -> amount)
    
    // Revenue sources and oracles
    revenue_sources: StorageMap<String, RevenueSource>,
    oracle_addresses: StorageMap<String, Address>,
    oracle_validators: StorageMap<String, Address>,
    supported_sources: StorageVec<String>,
    
    // Superfluid integration for streaming
    superfluid_host: StorageAddress,
    accepted_tokens: StorageMap<Address, bool>,
    streaming_revenues: StorageMap<U256, StorageMap<Address, StreamingRevenue>>,
    
    // Distribution tracking
    total_distributed: StorageMap<U256, U256>,
    distribution_history: StorageMap<U256, StorageVec<DistributionEvent>>,
    creator_claimed_revenue: StorageMap<U256, StorageMap<Address, U256>>, // project -> creator -> amount
    
    // Contract integration
    platform_contract: StorageAddress,
    nft_contract: StorageAddress,
    
    // Distribution settings
    platform_fee_bps: StorageU256,
    min_distribution_amount: StorageU256,
    distribution_frequency: StorageU256, // Minimum time between distributions
    creator_share_default: StorageU256, // Default creator share in BPS
    
    // Revenue verification
    pending_revenue_claims: StorageMap<U256, StorageMap<String, U256>>, // project -> source -> amount
    revenue_disputes: StorageMap<U256, StorageVec<String>>, // project -> disputed sources
    dispute_resolution_period: StorageU256,
    
    // Access control
    owner: StorageAddress,
    authorized_reporters: StorageMap<Address, bool>,
    revenue_managers: StorageMap<Address, bool>,
    
    // Global metrics
    total_revenue_processed: StorageU256,
    total_projects_with_revenue: StorageU256,
    average_project_revenue: StorageU256,
    
    // Emergency controls
    paused: StorageBool,
    emergency_withdrawal_enabled: StorageBool,
    
    // Reentrancy guard
    locked: StorageBool,
}

#[public]
impl RevenueDistributor {
    pub fn initialize(
        &mut self,
        platform_contract: Address,
        nft_contract: Address,
        superfluid_host: Address,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.platform_contract.set(platform_contract);
        self.nft_contract.set(nft_contract);
        self.superfluid_host.set(superfluid_host);
        
        // Set default parameters
        self.platform_fee_bps.set(U256::from(300)); // 3%
        self.min_distribution_amount.set(U256::from(1000000000000000u64)); // 0.001 ETH
        self.distribution_frequency.set(U256::from(24 * 3600)); // 24 hours
        self.creator_share_default.set(U256::from(3000)); // 30%
        self.dispute_resolution_period.set(U256::from(7 * 24 * 3600)); // 7 days
        
        // Initialize revenue sources
        self.initialize_revenue_sources();
        
        Ok(())
    }

    pub fn add_revenue_source(
        &mut self,
        project_id: U256,
        source: String,
        amount: U256,
        proof_uri: String,
    ) -> Result<bool> {
        self.require_not_paused()?;
        self.require_authorized_reporter()?;
        
        require_valid_input(amount > U256::from(0), "Amount must be positive")?;
        require_valid_input(
            self.is_supported_source(&source),
            "Revenue source not supported"
        )?;
        
        // Get revenue source configuration
        let source_config = self.revenue_sources.get(source.clone());
        require_valid_input(source_config.is_active, "Revenue source inactive")?;
        
        // If verification required, validate with oracle
        if source_config.verification_required {
            let verified = self.validate_revenue_with_oracle(project_id, source.clone(), amount)?;
            require_valid_input(verified, "Oracle verification failed")?;
        }
        
        // Update project revenue info
        let mut revenue_info = self.project_revenue.get(project_id);
        if revenue_info.total_revenue == U256::from(0) {
            // Initialize new project revenue tracking
            revenue_info = RevenueInfo {
                total_revenue: U256::from(0),
                last_distribution_timestamp: U256::from(0),
                revenue_sources: vec![source.clone()],
                oracle_verified: source_config.verification_required,
                creator_share_bps: self.creator_share_default.get(),
                community_share_bps: U256::from(10000) - self.creator_share_default.get() - self.platform_fee_bps.get(),
            };
        }
        
        // Add revenue amount
        revenue_info.total_revenue += amount;
        
        // Update source-specific tracking
        let current_source_amount = self.project_revenue_sources.get(project_id).get(source.clone());
        self.project_revenue_sources.get_mut(project_id).insert(source.clone(), current_source_amount + amount);
        
        // Update source in revenue info if new
        if !revenue_info.revenue_sources.contains(&source) {
            revenue_info.revenue_sources.push(source.clone());
        }
        
        self.project_revenue.insert(project_id, revenue_info);
        
        // Update global metrics
        self.total_revenue_processed.set(self.total_revenue_processed.get() + amount);

        evm::log(RevenueAdded {
            project_id,
            source,
            amount,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(true)
    }

    pub fn validate_revenue_with_oracle(
        &self,
        project_id: U256,
        source: String,
        amount: U256,
    ) -> Result<bool> {
        let oracle_address = self.oracle_addresses.get(source.clone());
        require_valid_input(!oracle_address.is_zero(), "Oracle not configured")?;
        
        // In production, would call actual oracle contract
        // For now, simplified validation
        let source_config = self.revenue_sources.get(source);
        let is_reasonable_amount = amount <= source_config.total_revenue_reported * U256::from(2); // Max 2x historical
        
        Ok(is_reasonable_amount && amount > U256::from(0))
    }

    pub fn distribute_revenue(&mut self, project_id: U256) -> Result<U256> {
        self.nonreentrant_guard()?;
        self.require_not_paused()?;
        
        let revenue_info = self.project_revenue.get(project_id);
        require_valid_input(revenue_info.total_revenue > U256::from(0), "No revenue to distribute")?;
        
        // Check minimum distribution amount and frequency
        let total_distributed = self.total_distributed.get(project_id);
        let available_for_distribution = revenue_info.total_revenue - total_distributed;
        
        require_valid_input(
            available_for_distribution >= self.min_distribution_amount.get(),
            "Below minimum distribution amount"
        )?;
        
        require_valid_input(
            U256::from(block::timestamp()) >= revenue_info.last_distribution_timestamp + self.distribution_frequency.get(),
            "Distribution frequency not met"
        )?;
        
        // Calculate distribution breakdown
        let platform_fee = (available_for_distribution * self.platform_fee_bps.get()) / U256::from(10000);
        let creator_share = (available_for_distribution * revenue_info.creator_share_bps) / U256::from(10000);
        let community_share = available_for_distribution - platform_fee - creator_share;
        
        // Distribute to NFT holders (community share)
        self.distribute_to_nft_holders(project_id, community_share)?;
        
        // Update distribution tracking
        self.total_distributed.insert(project_id, total_distributed + available_for_distribution);
        
        let distribution_event = DistributionEvent {
            timestamp: U256::from(block::timestamp()),
            amount: available_for_distribution,
            recipients_count: self.get_nft_holder_count(project_id),
            source: "batch_distribution".to_string(),
        };
        
        self.distribution_history.get_mut(project_id).push(distribution_event);
        
        // Update revenue info
        let mut updated_revenue_info = revenue_info;
        updated_revenue_info.last_distribution_timestamp = U256::from(block::timestamp());
        self.project_revenue.insert(project_id, updated_revenue_info);

        evm::log(RevenueDistributed {
            project_id,
            total_amount: available_for_distribution,
            creator_share,
            community_share,
            platform_fee,
        });

        self.unlock_guard();
        Ok(available_for_distribution)
    }

    pub fn setup_superfluid_stream(
        &mut self,
        project_id: U256,
        token: Address,
        flow_rate: i128,
    ) -> Result<()> {
        self.require_revenue_manager()?;
        require_valid_input(self.accepted_tokens.get(token), "Token not accepted for streaming")?;
        require_valid_input(flow_rate > 0, "Flow rate must be positive")?;
        
        let streaming_revenue = StreamingRevenue {
            project_id,
            token_address: token,
            flow_rate,
            total_streamed: U256::from(0),
            last_update_timestamp: U256::from(block::timestamp()),
            is_active: true,
        };
        
        self.streaming_revenues.get_mut(project_id).insert(token, streaming_revenue);
        
        // In production, would call Superfluid contract
        // self.create_superfluid_stream(project_id, token, flow_rate)?;
        
        Ok(())
    }

    pub fn claim_creator_revenue(&mut self, project_id: U256) -> Result<U256> {
        self.nonreentrant_guard()?;
        
        let creator = msg::sender();
        // In production, would verify creator ownership through platform contract
        
        let revenue_info = self.project_revenue.get(project_id);
        require_valid_input(revenue_info.total_revenue > U256::from(0), "No revenue available")?;
        
        let total_distributed = self.total_distributed.get(project_id);
        let available_revenue = revenue_info.total_revenue - total_distributed;
        let creator_share = (available_revenue * revenue_info.creator_share_bps) / U256::from(10000);
        
        let already_claimed = self.creator_claimed_revenue.get(project_id).get(creator);
        let claimable = creator_share - already_claimed;
        
        require_valid_input(claimable > U256::from(0), "No claimable revenue")?;
        
        // Transfer revenue to creator
        stylus_sdk::call::transfer_eth(creator, claimable)?;
        
        // Update claimed amount
        self.creator_claimed_revenue.get_mut(project_id).insert(creator, already_claimed + claimable);
        
        self.unlock_guard();
        Ok(claimable)
    }

    pub fn challenge_revenue_report(
        &mut self,
        project_id: U256,
        source: String,
        challenger: Address,
    ) -> Result<U256> {
        self.require_not_paused()?;
        
        require_valid_input(
            self.is_supported_source(&source),
            "Invalid revenue source"
        )?;
        
        // Add to disputed sources
        self.revenue_disputes.get_mut(project_id).push(source.clone());
        
        // In production, would create a formal dispute resolution process
        let challenge_id = project_id + U256::from(block::timestamp());
        
        Ok(challenge_id)
    }

    // View functions
    pub fn get_revenue_breakdown(&self, project_id: U256) -> Result<RevenueBreakdown> {
        let revenue_info = self.project_revenue.get(project_id);
        require_valid_input(revenue_info.total_revenue > U256::from(0), "Project has no revenue")?;
        
        let total_distributed = self.total_distributed.get(project_id);
        let platform_fee = (revenue_info.total_revenue * self.platform_fee_bps.get()) / U256::from(10000);
        let creator_share = (revenue_info.total_revenue * revenue_info.creator_share_bps) / U256::from(10000);
        let community_share = revenue_info.total_revenue - platform_fee - creator_share;
        
        Ok(RevenueBreakdown {
            project_id,
            total_revenue: revenue_info.total_revenue,
            creator_share,
            community_share,
            platform_fee,
            last_distribution: revenue_info.last_distribution_timestamp,
        })
    }

    pub fn get_project_revenue_sources(&self, project_id: U256) -> Vec<(String, U256)> {
        let sources_map = self.project_revenue_sources.get(project_id);
        let mut result = Vec::new();
        
        for i in 0..self.supported_sources.len() {
            if let Some(source) = self.supported_sources.get(i) {
                let amount = sources_map.get(source.clone());
                if amount > U256::from(0) {
                    result.push((source, amount));
                }
            }
        }
        
        result
    }

    pub fn get_streaming_revenue(&self, project_id: U256, token: Address) -> Result<StreamingRevenue> {
        let streaming = self.streaming_revenues.get(project_id).get(token);
        require_valid_input(streaming.is_active, "No active stream for this token")?;
        Ok(streaming)
    }

    pub fn platform_revenue_stats(&self) -> (U256, U256, U256) {
        (
            self.total_revenue_processed.get(),
            self.total_projects_with_revenue.get(),
            self.average_project_revenue.get(),
        )
    }

    // Admin functions
    pub fn add_revenue_source_config(
        &mut self,
        source_name: String,
        oracle_address: Address,
        verification_required: bool,
    ) -> Result<()> {
        self.require_owner()?;
        
        let source_config = RevenueSource {
            source_name: source_name.clone(),
            oracle_address,
            is_active: true,
            verification_required,
            last_update_timestamp: U256::from(block::timestamp()),
            total_revenue_reported: U256::from(0),
        };
        
        self.revenue_sources.insert(source_name.clone(), source_config);
        self.oracle_addresses.insert(source_name.clone(), oracle_address);
        self.supported_sources.push(source_name);
        
        Ok(())
    }

    pub fn add_accepted_token(&mut self, token: Address) -> Result<()> {
        self.require_owner()?;
        self.accepted_tokens.insert(token, true);
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        self.require_owner()?;
        self.paused.set(true);
        Ok(())
    }

    pub fn unpause(&mut self) -> Result<()> {
        self.require_owner()?;
        self.paused.set(false);
        Ok(())
    }

    pub fn set_platform_fee(&mut self, new_fee_bps: U256) -> Result<()> {
        self.require_owner()?;
        require_valid_input(new_fee_bps <= U256::from(1000), "Fee too high")?; // Max 10%
        self.platform_fee_bps.set(new_fee_bps);
        Ok(())
    }
}

// Internal helper functions
impl RevenueDistributor {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_authorized_reporter(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            self.authorized_reporters.get(caller) || 
            caller == self.owner.get(),
            "Not authorized reporter"
        )
    }

    fn require_revenue_manager(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            self.revenue_managers.get(caller) || 
            caller == self.owner.get(),
            "Not revenue manager"
        )
    }

    fn require_not_paused(&self) -> Result<()> {
        require_valid_input(!self.paused.get(), "Contract is paused")
    }

    fn nonreentrant_guard(&mut self) -> Result<()> {
        require_valid_input(!self.locked.get(), "Reentrant call")?;
        self.locked.set(true);
        Ok(())
    }

    fn unlock_guard(&mut self) {
        self.locked.set(false);
    }

    fn is_supported_source(&self, source: &str) -> bool {
        for i in 0..self.supported_sources.len() {
            if let Some(supported_source) = self.supported_sources.get(i) {
                if supported_source == source {
                    return true;
                }
            }
        }
        false
    }

    fn distribute_to_nft_holders(&self, project_id: U256, community_share: U256) -> Result<()> {
        // In production, would call NFT contract to distribute revenue
        // This would trigger the NFT contract's batch_distribute_revenue function
        Ok(())
    }

    fn get_nft_holder_count(&self, project_id: U256) -> U256 {
        // In production, would query NFT contract for holder count
        U256::from(10) // Placeholder
    }

    fn initialize_revenue_sources(&mut self) {
        let sources = vec![
            ("spotify", true),
            ("apple_music", true),
            ("youtube", true),
            ("soundcloud", false),
            ("bandcamp", false),
            ("licensing", true),
            ("merchandise", false),
            ("live_performances", false),
            ("nft_sales", true),
            ("streaming_tips", false),
        ];
        
        for (source, verification_required) in sources {
            self.supported_sources.push(source.to_string());
            
            let source_config = RevenueSource {
                source_name: source.to_string(),
                oracle_address: Address::ZERO, // Would be configured later
                is_active: true,
                verification_required,
                last_update_timestamp: U256::from(0),
                total_revenue_reported: U256::from(0),
            };
            
            self.revenue_sources.insert(source.to_string(), source_config);
        }
    }
}