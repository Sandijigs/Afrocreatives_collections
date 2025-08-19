use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
};

#[derive(SolidityType, Clone, Debug)]
pub struct CommunityVote {
    pub voter: Address,
    pub project_id: U256,
    pub vote_type: u8, // 0: Approve, 1: Reject, 2: Request Changes
    pub score: U256, // 0-100
    pub feedback: String,
    pub voting_power: U256,
    pub timestamp: U256,
}

#[derive(SolidityType, Clone, Debug)]
pub struct CommunityValidationResult {
    pub project_id: U256,
    pub total_votes: U256,
    pub approval_percentage: U256,
    pub average_score: U256,
    pub community_consensus: bool,
    pub completion_timestamp: U256,
}

#[storage]
#[entrypoint]
pub struct CommunityValidator {
    // Community voting
    project_votes: StorageMap<U256, StorageVec<CommunityVote>>,
    user_project_votes: StorageMap<Address, StorageMap<U256, bool>>, // user -> project -> voted
    
    // Validation results
    community_results: StorageMap<U256, CommunityValidationResult>,
    
    // Voting power calculation
    user_reputation: StorageMap<Address, U256>,
    cultural_expertise: StorageMap<Address, StorageMap<String, U256>>, // user -> region -> expertise
    
    // Platform integration
    platform_contract: StorageAddress,
    cultural_validator: StorageAddress,
    
    // Community validation parameters
    min_votes_required: StorageU256,
    voting_period: StorageU256,
    consensus_threshold: StorageU256, // Percentage needed for consensus
    reputation_weight: StorageU256, // How much reputation affects voting power
    
    // Access control
    owner: StorageAddress,
    
    // Community member tracking
    verified_community_members: StorageMap<Address, bool>,
    community_member_count: StorageU256,
    
    // Incentives
    voting_rewards: StorageMap<Address, U256>, // Accumulated rewards for participation
    reward_per_vote: StorageU256,
    
    // Anti-gaming measures
    vote_cooldown: StorageMap<Address, U256>, // Last vote timestamp
    cooldown_period: StorageU256,
    max_votes_per_period: StorageU256,
    user_vote_count: StorageMap<Address, StorageMap<U256, U256>>, // user -> period -> vote_count
}

#[public]
impl CommunityValidator {
    pub fn initialize(&mut self, platform_contract: Address, cultural_validator: Address) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.platform_contract.set(platform_contract);
        self.cultural_validator.set(cultural_validator);
        
        // Set default parameters
        self.min_votes_required.set(U256::from(10));
        self.voting_period.set(U256::from(5 * 24 * 3600)); // 5 days
        self.consensus_threshold.set(U256::from(60)); // 60% approval needed
        self.reputation_weight.set(U256::from(50)); // 50% weight to reputation
        self.reward_per_vote.set(U256::from(1000000000000000u64)); // 0.001 ETH
        self.cooldown_period.set(U256::from(1 * 3600)); // 1 hour between votes
        self.max_votes_per_period.set(U256::from(5)); // Max 5 votes per day
        
        Ok(())
    }

    pub fn submit_community_vote(
        &mut self,
        project_id: U256,
        vote_type: u8,
        score: U256,
        feedback: String,
    ) -> Result<()> {
        let voter = msg::sender();
        
        // Check if user is verified community member
        require_valid_input(
            self.verified_community_members.get(voter),
            "Not a verified community member"
        )?;
        
        // Check if already voted
        require_valid_input(
            !self.user_project_votes.get(voter).get(project_id),
            "Already voted on this project"
        )?;
        
        // Check cooldown
        let last_vote = self.vote_cooldown.get(voter);
        require_valid_input(
            U256::from(block::timestamp()) >= last_vote + self.cooldown_period.get(),
            "Cooldown period not elapsed"
        )?;
        
        // Check daily vote limit
        let current_period = U256::from(block::timestamp()) / U256::from(24 * 3600);
        let votes_today = self.user_vote_count.get(voter).get(current_period);
        require_valid_input(
            votes_today < self.max_votes_per_period.get(),
            "Daily vote limit exceeded"
        )?;
        
        // Validate inputs
        require_valid_input(vote_type <= 2, "Invalid vote type")?;
        require_valid_input(score <= U256::from(100), "Score must be 0-100")?;
        
        // Calculate voting power
        let voting_power = self.calculate_voting_power(voter, project_id);
        
        let vote = CommunityVote {
            voter,
            project_id,
            vote_type,
            score,
            feedback,
            voting_power,
            timestamp: U256::from(block::timestamp()),
        };
        
        // Store vote
        self.project_votes.get_mut(project_id).push(vote);
        self.user_project_votes.get_mut(voter).insert(project_id, true);
        
        // Update cooldown and vote count
        self.vote_cooldown.insert(voter, U256::from(block::timestamp()));
        self.user_vote_count.get_mut(voter).insert(current_period, votes_today + U256::from(1));
        
        // Check if we have enough votes to finalize
        let votes = self.project_votes.get(project_id);
        if votes.len() >= self.min_votes_required.get().as_usize() {
            self.finalize_community_validation(project_id)?;
        }
        
        // Reward voter
        let current_rewards = self.voting_rewards.get(voter);
        self.voting_rewards.insert(voter, current_rewards + self.reward_per_vote.get());
        
        Ok(())
    }

    pub fn finalize_community_validation(&mut self, project_id: U256) -> Result<()> {
        let votes = self.project_votes.get(project_id);
        require_valid_input(
            votes.len() >= self.min_votes_required.get().as_usize(),
            "Insufficient votes"
        )?;
        
        let mut total_weighted_score = U256::from(0);
        let mut total_voting_power = U256::from(0);
        let mut approval_votes = U256::from(0);
        let mut total_approval_power = U256::from(0);
        
        // Calculate weighted results
        for i in 0..votes.len() {
            if let Some(vote) = votes.get(i) {
                total_weighted_score += vote.score * vote.voting_power;
                total_voting_power += vote.voting_power;
                
                if vote.vote_type == 0 { // Approve
                    approval_votes += U256::from(1);
                    total_approval_power += vote.voting_power;
                }
            }
        }
        
        let average_score = if total_voting_power > U256::from(0) {
            total_weighted_score / total_voting_power
        } else {
            U256::from(0)
        };
        
        let approval_percentage = if total_voting_power > U256::from(0) {
            (total_approval_power * U256::from(100)) / total_voting_power
        } else {
            U256::from(0)
        };
        
        let community_consensus = approval_percentage >= self.consensus_threshold.get();
        
        let result = CommunityValidationResult {
            project_id,
            total_votes: U256::from(votes.len()),
            approval_percentage,
            average_score,
            community_consensus,
            completion_timestamp: U256::from(block::timestamp()),
        };
        
        self.community_results.insert(project_id, result);
        
        Ok(())
    }

    pub fn verify_community_member(&mut self, member: Address, cultural_regions: Vec<String>) -> Result<()> {
        self.require_owner()?;
        
        require_valid_input(!member.is_zero(), "Invalid member address")?;
        
        self.verified_community_members.insert(member, true);
        self.user_reputation.insert(member, U256::from(50)); // Starting reputation
        
        // Set cultural expertise
        for region in cultural_regions {
            self.cultural_expertise.get_mut(member).insert(region, U256::from(1)); // Basic expertise
        }
        
        self.community_member_count.set(self.community_member_count.get() + U256::from(1));
        
        Ok(())
    }

    pub fn claim_voting_rewards(&mut self) -> Result<U256> {
        let claimer = msg::sender();
        let rewards = self.voting_rewards.get(claimer);
        
        require_valid_input(rewards > U256::from(0), "No rewards to claim")?;
        
        // Transfer rewards (simplified - in production would use proper reward mechanism)
        self.voting_rewards.insert(claimer, U256::from(0));
        
        Ok(rewards)
    }

    // View functions
    pub fn get_community_validation(&self, project_id: U256) -> Result<CommunityValidationResult> {
        let result = self.community_results.get(project_id);
        require_valid_input(result.project_id != U256::from(0), "Validation not found")?;
        Ok(result)
    }

    pub fn get_project_votes(&self, project_id: U256) -> Vec<CommunityVote> {
        let votes = self.project_votes.get(project_id);
        let mut result = Vec::new();
        
        for i in 0..votes.len() {
            if let Some(vote) = votes.get(i) {
                result.push(vote);
            }
        }
        
        result
    }

    pub fn calculate_voting_power(&self, user: Address, project_id: U256) -> U256 {
        let base_power = U256::from(100); // Base voting power
        let reputation = self.user_reputation.get(user);
        
        // Get project's cultural region (would query platform contract in production)
        let project_region = "West Africa".to_string(); // Placeholder
        
        let cultural_expertise = self.cultural_expertise.get(user).get(project_region);
        
        // Calculate weighted power
        let reputation_bonus = (reputation * self.reputation_weight.get()) / U256::from(100);
        let expertise_bonus = cultural_expertise * U256::from(20); // 20 points per expertise level
        
        base_power + reputation_bonus + expertise_bonus
    }

    pub fn get_user_reputation(&self, user: Address) -> U256 {
        self.user_reputation.get(user)
    }

    pub fn is_verified_member(&self, user: Address) -> bool {
        self.verified_community_members.get(user)
    }

    pub fn get_pending_rewards(&self, user: Address) -> U256 {
        self.voting_rewards.get(user)
    }

    // Admin functions
    pub fn update_user_reputation(&mut self, user: Address, adjustment: i64) -> Result<()> {
        self.require_owner()?;
        
        let current_reputation = self.user_reputation.get(user).as_i64();
        let new_reputation = if adjustment >= 0 {
            core::cmp::min(current_reputation + adjustment, 100)
        } else {
            core::cmp::max(current_reputation + adjustment, 0)
        };
        
        self.user_reputation.insert(user, U256::from(new_reputation as u64));
        
        Ok(())
    }

    pub fn set_cultural_expertise(
        &mut self,
        user: Address,
        region: String,
        level: U256,
    ) -> Result<()> {
        self.require_owner()?;
        
        require_valid_input(level <= U256::from(5), "Expertise level must be 1-5")?;
        self.cultural_expertise.get_mut(user).insert(region, level);
        
        Ok(())
    }

    pub fn update_validation_parameters(
        &mut self,
        min_votes: U256,
        consensus_threshold: U256,
    ) -> Result<()> {
        self.require_owner()?;
        
        require_valid_input(min_votes >= U256::from(3), "Need at least 3 votes")?;
        require_valid_input(
            consensus_threshold >= U256::from(50) && consensus_threshold <= U256::from(100),
            "Invalid consensus threshold"
        )?;
        
        self.min_votes_required.set(min_votes);
        self.consensus_threshold.set(consensus_threshold);
        
        Ok(())
    }
}

// Internal helper functions
impl CommunityValidator {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }
}