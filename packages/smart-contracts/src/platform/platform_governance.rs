use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
    Proposal, Vote, ProposalStatus, Role,
};

#[derive(SolidityType, Clone, Debug)]
pub struct VotingPowerBreakdown {
    pub creator_power: U256,
    pub backer_power: U256,
    pub validator_power: U256,
    pub reputation_multiplier: U256,
    pub total_power: U256,
}

#[derive(SolidityType, Clone, Debug)]
pub struct GovernanceStats {
    pub total_proposals: U256,
    pub executed_proposals: U256,
    pub active_voters: U256,
    pub total_voting_power: U256,
    pub treasury_balance: U256,
    pub cultural_fund_balance: U256,
}

#[derive(SolidityType, Clone, Debug)]
pub struct CulturalFundAllocation {
    pub allocation_id: U256,
    pub recipient: Address,
    pub amount: U256,
    pub purpose: String,
    pub region: String,
    pub approved_timestamp: U256,
    pub disbursed: bool,
}

#[storage]
#[entrypoint]
pub struct PlatformGovernance {
    // Voting power tracking
    creator_voting_power: StorageMap<Address, U256>,
    backer_voting_power: StorageMap<Address, U256>,
    validator_voting_power: StorageMap<Address, U256>,
    reputation_scores: StorageMap<Address, U256>,
    
    // Proposals
    proposals: StorageMap<U256, Proposal>,
    proposal_votes: StorageMap<U256, StorageMap<Address, Vote>>,
    proposal_vote_counts: StorageMap<U256, (U256, U256)>, // (for_votes, against_votes)
    next_proposal_id: StorageU256,
    
    // Treasury management
    treasury_balance: StorageU256,
    cultural_fund_balance: StorageU256,
    cultural_fund_allocations: StorageMap<U256, CulturalFundAllocation>,
    next_allocation_id: StorageU256,
    
    // Platform integration
    platform_contract: StorageAddress,
    validator_contract: StorageAddress,
    funding_contract: StorageAddress,
    revenue_distributor: StorageAddress,
    
    // Governance parameters
    proposal_threshold: StorageU256,
    voting_period: StorageU256,
    execution_delay: StorageU256,
    quorum_threshold: StorageU256, // Minimum participation required
    
    // Voting power weights
    creator_weight: StorageU256,
    backer_weight: StorageU256,
    validator_weight: StorageU256,
    reputation_multiplier: StorageU256,
    
    // Cultural fund management
    cultural_regions: StorageVec<String>,
    regional_fund_allocation: StorageMap<String, U256>, // region -> allocated amount
    regional_coordinators: StorageMap<String, Address>,
    
    // Access control
    owner: StorageAddress,
    governance_admins: StorageMap<Address, bool>,
    
    // Emergency controls
    emergency_pause: StorageBool,
    emergency_council: StorageVec<Address>,
    
    // Metrics and history
    total_proposals_created: StorageU256,
    total_proposals_executed: StorageU256,
    active_voter_count: StorageU256,
    total_cultural_fund_distributed: StorageU256,
    
    // Delegation system
    voting_delegates: StorageMap<Address, Address>, // delegator -> delegate
    delegate_power: StorageMap<Address, U256>, // delegate -> total delegated power
}

#[public]
impl PlatformGovernance {
    pub fn initialize(
        &mut self,
        platform_contract: Address,
        validator_contract: Address,
        funding_contract: Address,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.platform_contract.set(platform_contract);
        self.validator_contract.set(validator_contract);
        self.funding_contract.set(funding_contract);
        
        // Set default governance parameters
        self.proposal_threshold.set(U256::from(1000000000000000000u64)); // 1 unit of voting power
        self.voting_period.set(U256::from(7 * 24 * 3600)); // 7 days
        self.execution_delay.set(U256::from(24 * 3600)); // 1 day
        self.quorum_threshold.set(U256::from(1000)); // 10% in basis points
        
        // Set voting power weights
        self.creator_weight.set(U256::from(100)); // Base weight for creators
        self.backer_weight.set(U256::from(50)); // Base weight for backers
        self.validator_weight.set(U256::from(150)); // Higher weight for validators
        self.reputation_multiplier.set(U256::from(150)); // 1.5x multiplier for high reputation
        
        self.next_proposal_id.set(U256::from(1));
        self.next_allocation_id.set(U256::from(1));
        
        // Initialize cultural regions
        self.initialize_cultural_regions();
        
        Ok(())
    }

    pub fn create_proposal(
        &mut self,
        title: String,
        description: String,
        execution_data: Vec<u8>,
    ) -> Result<U256> {
        self.require_not_paused()?;
        
        let proposer = msg::sender();
        let voting_power = self.calculate_voting_power(proposer)?;
        
        require_valid_input(
            voting_power >= self.proposal_threshold.get(),
            "Insufficient voting power to create proposal"
        )?;
        
        let proposal_id = self.next_proposal_id.get();
        let current_time = U256::from(block::timestamp());
        
        let proposal = Proposal {
            id: proposal_id,
            title,
            description,
            proposer,
            start_time: current_time,
            end_time: current_time + self.voting_period.get(),
            for_votes: U256::from(0),
            against_votes: U256::from(0),
            status: 0, // Active
            execution_data,
        };
        
        self.proposals.insert(proposal_id, proposal.clone());
        self.next_proposal_id.set(proposal_id + U256::from(1));
        self.total_proposals_created.set(self.total_proposals_created.get() + U256::from(1));

        evm::log(ProposalCreated {
            proposal_id,
            proposer,
            title: proposal.title,
            start_time: current_time,
            end_time: proposal.end_time,
        });

        Ok(proposal_id)
    }

    pub fn vote(&mut self, proposal_id: U256, support: bool) -> Result<()> {
        self.require_not_paused()?;
        
        let voter = msg::sender();
        let voting_power = self.calculate_voting_power(voter)?;
        
        require_valid_input(voting_power > U256::from(0), "No voting power")?;
        
        let proposal = self.proposals.get(proposal_id);
        require_valid_input(proposal.id != U256::from(0), "Proposal not found")?;
        require_valid_input(proposal.status == 0, "Proposal not active")?;
        
        let current_time = U256::from(block::timestamp());
        require_valid_input(
            current_time >= proposal.start_time && current_time <= proposal.end_time,
            "Voting period not active"
        )?;
        
        // Check if already voted
        let existing_vote = self.proposal_votes.get(proposal_id).get(voter);
        require_valid_input(
            existing_vote.timestamp == U256::from(0),
            "Already voted on this proposal"
        )?;
        
        // Record vote
        let vote = Vote {
            support,
            voting_power,
            timestamp: current_time,
        };
        
        self.proposal_votes.get_mut(proposal_id).insert(voter, vote);
        
        // Update vote counts
        let (mut for_votes, mut against_votes) = self.proposal_vote_counts.get(proposal_id);
        if support {
            for_votes += voting_power;
        } else {
            against_votes += voting_power;
        }
        self.proposal_vote_counts.insert(proposal_id, (for_votes, against_votes));
        
        // Update proposal
        let mut updated_proposal = proposal;
        updated_proposal.for_votes = for_votes;
        updated_proposal.against_votes = against_votes;
        self.proposals.insert(proposal_id, updated_proposal);

        evm::log(VoteCast {
            proposal_id,
            voter,
            support,
            voting_power,
        });

        Ok(())
    }

    pub fn execute_proposal(&mut self, proposal_id: U256) -> Result<bool> {
        self.require_not_paused()?;
        
        let proposal = self.proposals.get(proposal_id);
        require_valid_input(proposal.id != U256::from(0), "Proposal not found")?;
        require_valid_input(proposal.status == 0, "Proposal not active")?;
        
        let current_time = U256::from(block::timestamp());
        require_valid_input(current_time > proposal.end_time, "Voting period not ended")?;
        require_valid_input(
            current_time >= proposal.end_time + self.execution_delay.get(),
            "Execution delay not passed"
        )?;
        
        // Check if proposal passed
        let total_votes = proposal.for_votes + proposal.against_votes;
        let total_voting_power = self.calculate_total_voting_power();
        let quorum_required = (total_voting_power * self.quorum_threshold.get()) / U256::from(10000);
        
        require_valid_input(total_votes >= quorum_required, "Quorum not reached")?;
        require_valid_input(proposal.for_votes > proposal.against_votes, "Proposal rejected")?;
        
        // Execute proposal
        let success = self.execute_proposal_logic(&proposal)?;
        
        // Update proposal status
        let mut updated_proposal = proposal;
        updated_proposal.status = if success { 3 } else { 2 }; // Executed or Failed
        self.proposals.insert(proposal_id, updated_proposal);
        
        if success {
            self.total_proposals_executed.set(self.total_proposals_executed.get() + U256::from(1));
        }

        evm::log(ProposalExecuted {
            proposal_id,
            success,
        });

        Ok(success)
    }

    pub fn allocate_cultural_fund(
        &mut self,
        recipient: Address,
        amount: U256,
        purpose: String,
        region: String,
    ) -> Result<U256> {
        self.require_governance_admin()?;
        
        require_valid_input(amount <= self.cultural_fund_balance.get(), "Insufficient cultural fund")?;
        require_valid_input(self.is_supported_region(&region), "Unsupported region")?;
        
        let allocation_id = self.next_allocation_id.get();
        
        let allocation = CulturalFundAllocation {
            allocation_id,
            recipient,
            amount,
            purpose,
            region: region.clone(),
            approved_timestamp: U256::from(block::timestamp()),
            disbursed: false,
        };
        
        self.cultural_fund_allocations.insert(allocation_id, allocation);
        self.next_allocation_id.set(allocation_id + U256::from(1));
        
        // Update regional allocation tracking
        let current_regional = self.regional_fund_allocation.get(region.clone());
        self.regional_fund_allocation.insert(region, current_regional + amount);
        
        // Reserve funds
        self.cultural_fund_balance.set(self.cultural_fund_balance.get() - amount);
        
        Ok(allocation_id)
    }

    pub fn disburse_cultural_fund(&mut self, allocation_id: U256) -> Result<()> {
        self.require_governance_admin()?;
        
        let mut allocation = self.cultural_fund_allocations.get(allocation_id);
        require_valid_input(allocation.allocation_id != U256::from(0), "Allocation not found")?;
        require_valid_input(!allocation.disbursed, "Already disbursed")?;
        
        // Transfer funds to recipient
        stylus_sdk::call::transfer_eth(allocation.recipient, allocation.amount)?;
        
        // Mark as disbursed
        allocation.disbursed = true;
        self.cultural_fund_allocations.insert(allocation_id, allocation);
        
        self.total_cultural_fund_distributed.set(
            self.total_cultural_fund_distributed.get() + allocation.amount
        );
        
        Ok(())
    }

    pub fn delegate_voting_power(&mut self, delegate: Address) -> Result<()> {
        let delegator = msg::sender();
        require_valid_input(delegator != delegate, "Cannot delegate to self")?;
        
        let voting_power = self.calculate_voting_power(delegator)?;
        require_valid_input(voting_power > U256::from(0), "No voting power to delegate")?;
        
        // Remove previous delegation if exists
        let previous_delegate = self.voting_delegates.get(delegator);
        if !previous_delegate.is_zero() {
            let previous_power = self.delegate_power.get(previous_delegate);
            self.delegate_power.insert(previous_delegate, previous_power - voting_power);
        }
        
        // Set new delegation
        self.voting_delegates.insert(delegator, delegate);
        let current_power = self.delegate_power.get(delegate);
        self.delegate_power.insert(delegate, current_power + voting_power);
        
        Ok(())
    }

    // View functions
    pub fn calculate_voting_power(&self, user: Address) -> Result<U256> {
        let creator_power = self.creator_voting_power.get(user) * self.creator_weight.get() / U256::from(100);
        let backer_power = self.backer_voting_power.get(user) * self.backer_weight.get() / U256::from(100);
        let validator_power = self.validator_voting_power.get(user) * self.validator_weight.get() / U256::from(100);
        
        let base_power = creator_power + backer_power + validator_power;
        
        // Apply reputation multiplier
        let reputation = self.reputation_scores.get(user);
        let multiplier = if reputation >= U256::from(80) {
            self.reputation_multiplier.get()
        } else {
            U256::from(100) // No multiplier
        };
        
        let total_power = (base_power * multiplier) / U256::from(100);
        
        // Add delegated power
        let delegated_power = self.delegate_power.get(user);
        
        Ok(total_power + delegated_power)
    }

    pub fn get_proposal(&self, proposal_id: U256) -> Result<Proposal> {
        let proposal = self.proposals.get(proposal_id);
        require_valid_input(proposal.id != U256::from(0), "Proposal not found")?;
        Ok(proposal)
    }

    pub fn get_vote(&self, proposal_id: U256, voter: Address) -> Vote {
        self.proposal_votes.get(proposal_id).get(voter)
    }

    pub fn governance_stats(&self) -> GovernanceStats {
        GovernanceStats {
            total_proposals: self.total_proposals_created.get(),
            executed_proposals: self.total_proposals_executed.get(),
            active_voters: self.active_voter_count.get(),
            total_voting_power: self.calculate_total_voting_power(),
            treasury_balance: self.treasury_balance.get(),
            cultural_fund_balance: self.cultural_fund_balance.get(),
        }
    }

    pub fn get_cultural_fund_allocation(&self, allocation_id: U256) -> Result<CulturalFundAllocation> {
        let allocation = self.cultural_fund_allocations.get(allocation_id);
        require_valid_input(allocation.allocation_id != U256::from(0), "Allocation not found")?;
        Ok(allocation)
    }

    // Admin functions
    pub fn update_platform_parameters(&mut self, new_params: Vec<(String, U256)>) -> Result<()> {
        self.require_owner()?;
        
        for (param, value) in new_params {
            match param.as_str() {
                "proposal_threshold" => self.proposal_threshold.set(value),
                "voting_period" => self.voting_period.set(value),
                "execution_delay" => self.execution_delay.set(value),
                "quorum_threshold" => self.quorum_threshold.set(value),
                _ => return Err(AfroCreateError::InvalidInput("Unknown parameter".to_string())),
            }
        }
        
        Ok(())
    }

    pub fn add_governance_admin(&mut self, admin: Address) -> Result<()> {
        self.require_owner()?;
        self.governance_admins.insert(admin, true);
        Ok(())
    }

    pub fn fund_cultural_fund(&mut self) -> Result<()> {
        let amount = msg::value();
        self.cultural_fund_balance.set(self.cultural_fund_balance.get() + amount);
        Ok(())
    }

    pub fn emergency_pause(&mut self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.owner.get() || self.is_emergency_council_member(caller),
            "Not authorized for emergency actions"
        )?;
        
        self.emergency_pause.set(true);
        Ok(())
    }
}

// Internal helper functions
impl PlatformGovernance {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_governance_admin(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.owner.get() || self.governance_admins.get(caller),
            "Only governance admin"
        )
    }

    fn require_not_paused(&self) -> Result<()> {
        require_valid_input(!self.emergency_pause.get(), "Governance paused")
    }

    fn is_emergency_council_member(&self, user: Address) -> bool {
        for i in 0..self.emergency_council.len() {
            if let Some(member) = self.emergency_council.get(i) {
                if member == user {
                    return true;
                }
            }
        }
        false
    }

    fn calculate_total_voting_power(&self) -> U256 {
        // In production, would iterate through all users or maintain a cached total
        U256::from(1000000) // Placeholder
    }

    fn execute_proposal_logic(&self, proposal: &Proposal) -> Result<bool> {
        // In production, would decode and execute the proposal's execution_data
        // This could involve calling other contracts, updating parameters, etc.
        Ok(true) // Simplified for demo
    }

    fn is_supported_region(&self, region: &str) -> bool {
        for i in 0..self.cultural_regions.len() {
            if let Some(supported_region) = self.cultural_regions.get(i) {
                if supported_region == region {
                    return true;
                }
            }
        }
        false
    }

    fn initialize_cultural_regions(&mut self) {
        let regions = vec![
            "West Africa",
            "East Africa", 
            "Central Africa",
            "Southern Africa",
            "North Africa",
            "African Diaspora",
        ];
        
        for region in regions {
            self.cultural_regions.push(region.to_string());
            self.regional_fund_allocation.insert(region.to_string(), U256::from(0));
        }
    }
}