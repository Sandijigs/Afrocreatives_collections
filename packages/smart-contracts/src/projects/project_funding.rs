use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, call, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256, StorageVec, StorageBool},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input, require_sufficient_funds},
    events::*,
    interfaces::{IAfroCreatePlatform, IRevenueShareNFT},
    FundingInfo, FundingModel, Milestone,
};

#[storage]
#[entrypoint]
pub struct ProjectFunding {
    // Core funding data
    project_funding: StorageMap<U256, FundingInfo>,
    backer_contributions: StorageMap<U256, StorageMap<Address, U256>>, // projectId -> (backer -> amount)
    project_backers: StorageMap<U256, StorageVec<Address>>, // projectId -> backers list
    
    // NFT contract for revenue shares
    revenue_nft_contract: StorageAddress,
    
    // Funding models
    funding_models: StorageMap<U256, U256>, // projectId -> FundingModel (as u8)
    
    // Milestones
    project_milestones: StorageMap<U256, StorageVec<Milestone>>,
    milestone_releases: StorageMap<U256, StorageMap<U256, bool>>, // projectId -> (milestoneId -> released)
    milestone_completion: StorageMap<U256, StorageMap<U256, bool>>, // projectId -> (milestoneId -> completed)
    
    // Platform integration
    platform_contract: StorageAddress,
    
    // Settings
    platform_fee_bps: StorageU256,
    min_contribution: StorageU256,
    refund_period: StorageU256, // Period after deadline for refunds
    
    // Escrow and treasury
    project_escrow: StorageMap<U256, U256>, // projectId -> escrowed amount
    platform_treasury: StorageU256,
    
    // Access control
    owner: StorageAddress,
    authorized_callers: StorageMap<Address, bool>,
    
    // Metrics
    total_projects_funded: StorageU256,
    total_amount_raised: StorageU256,
    total_backers: StorageU256,
    
    // Reentrancy guard
    locked: StorageBool,
}

#[public]
impl ProjectFunding {
    pub fn initialize(
        &mut self,
        platform_contract: Address,
        revenue_nft_contract: Address,
        platform_fee_bps: U256,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.platform_contract.set(platform_contract);
        self.revenue_nft_contract.set(revenue_nft_contract);
        self.platform_fee_bps.set(platform_fee_bps);
        self.min_contribution.set(U256::from(1000000000000000u64)); // 0.001 ETH minimum
        self.refund_period.set(U256::from(30 * 24 * 3600)); // 30 days
        
        Ok(())
    }

    #[payable]
    pub fn fund_project(&mut self, project_id: U256, backer_ens_name: String) -> Result<U256> {
        self.nonreentrant_guard()?;
        
        let backer = msg::sender();
        let contribution = msg::value();
        
        require_sufficient_funds(
            contribution >= self.min_contribution.get(),
            "Contribution too small"
        )?;
        
        // Get project info from platform contract
        let funding_info = self.project_funding.get(project_id);
        require_valid_input(funding_info.target > U256::from(0), "Project not found")?;
        require_valid_input(funding_info.status == 0, "Project not active")?;
        require_valid_input(
            U256::from(block::timestamp()) <= funding_info.deadline,
            "Funding deadline passed"
        )?;
        
        // Update funding info
        let mut updated_funding = funding_info;
        updated_funding.raised += contribution;
        
        // Track backer contribution
        let previous_contribution = self.backer_contributions.get(project_id).get(backer);
        self.backer_contributions.get_mut(project_id).insert(backer, previous_contribution + contribution);
        
        // Add to backers list if first contribution
        if previous_contribution == U256::from(0) {
            self.project_backers.get_mut(project_id).push(backer);
            updated_funding.backer_count += U256::from(1);
        }
        
        // Update escrow
        let current_escrow = self.project_escrow.get(project_id);
        self.project_escrow.insert(project_id, current_escrow + contribution);
        
        // Check if funding target reached
        let funding_model = self.get_funding_model(project_id);
        if updated_funding.raised >= updated_funding.target {
            updated_funding.status = 1; // Successful
            self.total_projects_funded.set(self.total_projects_funded.get() + U256::from(1));
        }
        
        self.project_funding.insert(project_id, updated_funding);
        self.total_amount_raised.set(self.total_amount_raised.get() + contribution);
        
        // Mint revenue-sharing NFT to backer
        let nft_token_id = self.mint_revenue_nft(project_id, backer, contribution, backer_ens_name)?;
        
        // Update platform contract
        self.update_platform_funding(project_id, updated_funding.raised)?;

        evm::log(ProjectFunded {
            project_id,
            backer,
            amount: contribution,
            total_raised: updated_funding.raised,
        });

        self.unlock_guard();
        Ok(nft_token_id)
    }

    pub fn setup_project_funding(
        &mut self,
        project_id: U256,
        target: U256,
        deadline: U256,
        creator: Address,
        funding_model: U256, // FundingModel as u8
        milestones: Vec<Milestone>,
    ) -> Result<()> {
        self.require_authorized_caller()?;
        
        require_valid_input(
            self.project_funding.get(project_id).target == U256::from(0),
            "Project already configured"
        )?;
        
        let funding_info = FundingInfo {
            target,
            raised: U256::from(0),
            deadline,
            status: 0, // Active
            creator,
            backer_count: U256::from(0),
            funding_model: funding_model.as_u8(),
        };
        
        self.project_funding.insert(project_id, funding_info);
        self.funding_models.insert(project_id, funding_model);
        
        // Setup milestones for milestone-based funding
        if funding_model == 2 { // MilestoneBased
            let mut milestone_storage = self.project_milestones.get_mut(project_id);
            for milestone in milestones {
                milestone_storage.push(milestone);
            }
        }
        
        Ok(())
    }

    pub fn release_milestone_funds(&mut self, project_id: U256, milestone_id: U256) -> Result<()> {
        self.require_authorized_caller()?;
        
        let funding_info = self.project_funding.get(project_id);
        require_valid_input(funding_info.target > U256::from(0), "Project not found")?;
        require_valid_input(
            self.get_funding_model(project_id) == FundingModel::MilestoneBased,
            "Not milestone-based project"
        )?;
        
        // Check milestone exists and is completed
        require_valid_input(
            self.milestone_completion.get(project_id).get(milestone_id),
            "Milestone not completed"
        )?;
        require_valid_input(
            !self.milestone_releases.get(project_id).get(milestone_id),
            "Funds already released for this milestone"
        )?;
        
        let milestones = self.project_milestones.get(project_id);
        require_valid_input(
            milestone_id.as_usize() < milestones.len(),
            "Invalid milestone ID"
        )?;
        
        if let Some(milestone) = milestones.get(milestone_id.as_usize()) {
            let release_amount = milestone.funding_amount;
            
            // Transfer funds to creator
            self.transfer_to_creator(funding_info.creator, release_amount)?;
            
            // Mark as released
            self.milestone_releases.get_mut(project_id).insert(milestone_id, true);
            
            evm::log(MilestoneCompleted {
                project_id,
                milestone_id,
                amount_released: release_amount,
            });
        }
        
        Ok(())
    }

    pub fn process_refunds(&mut self, project_id: U256) -> Result<()> {
        self.nonreentrant_guard()?;
        
        let funding_info = self.project_funding.get(project_id);
        require_valid_input(funding_info.target > U256::from(0), "Project not found")?;
        
        let funding_model = self.get_funding_model(project_id);
        let current_time = U256::from(block::timestamp());
        
        // Check if refunds are allowed
        let refund_eligible = match funding_model {
            FundingModel::AllOrNothing => {
                funding_info.status == 2 || // Failed
                (current_time > funding_info.deadline && funding_info.raised < funding_info.target)
            },
            FundingModel::MilestoneBased => {
                funding_info.status == 3 // Cancelled
            },
            _ => false,
        };
        
        require_valid_input(refund_eligible, "Refunds not available")?;
        require_valid_input(
            current_time <= funding_info.deadline + self.refund_period.get(),
            "Refund period expired"
        )?;
        
        // Process refunds for all backers
        let backers = self.project_backers.get(project_id);
        let escrow_amount = self.project_escrow.get(project_id);
        let total_raised = funding_info.raised;
        
        for i in 0..backers.len() {
            if let Some(backer) = backers.get(i) {
                let contribution = self.backer_contributions.get(project_id).get(backer);
                if contribution > U256::from(0) {
                    // Calculate refund amount proportionally
                    let refund_amount = if total_raised > U256::from(0) {
                        (contribution * escrow_amount) / total_raised
                    } else {
                        contribution
                    };
                    
                    // Transfer refund
                    if refund_amount > U256::from(0) {
                        call::transfer_eth(backer, refund_amount)?;
                    }
                    
                    // Clear contribution
                    self.backer_contributions.get_mut(project_id).insert(backer, U256::from(0));
                }
            }
        }
        
        // Clear escrow
        self.project_escrow.insert(project_id, U256::from(0));
        
        // Update project status
        let mut updated_funding = funding_info;
        updated_funding.status = 2; // Failed/Refunded
        self.project_funding.insert(project_id, updated_funding);
        
        self.unlock_guard();
        Ok(())
    }

    pub fn finalize_successful_project(&mut self, project_id: U256) -> Result<()> {
        self.require_authorized_caller()?;
        
        let funding_info = self.project_funding.get(project_id);
        require_valid_input(funding_info.target > U256::from(0), "Project not found")?;
        require_valid_input(funding_info.status == 1, "Project not successful")?;
        
        let funding_model = self.get_funding_model(project_id);
        let escrow_amount = self.project_escrow.get(project_id);
        
        match funding_model {
            FundingModel::AllOrNothing | FundingModel::FlexibleFunding => {
                // Release all funds to creator minus platform fee
                let platform_fee = (escrow_amount * self.platform_fee_bps.get()) / U256::from(10000);
                let creator_amount = escrow_amount - platform_fee;
                
                self.transfer_to_creator(funding_info.creator, creator_amount)?;
                self.platform_treasury.set(self.platform_treasury.get() + platform_fee);
                
                // Clear escrow
                self.project_escrow.insert(project_id, U256::from(0));
            },
            FundingModel::MilestoneBased => {
                // Funds released per milestone, no action needed here
            }
        }
        
        Ok(())
    }

    // View functions
    pub fn get_funding_stats(&self, project_id: U256) -> Result<FundingInfo> {
        let funding_info = self.project_funding.get(project_id);
        require_valid_input(funding_info.target > U256::from(0), "Project not found")?;
        Ok(funding_info)
    }

    pub fn get_backer_contributions(&self, project_id: U256, backer: Address) -> U256 {
        self.backer_contributions.get(project_id).get(backer)
    }

    pub fn get_project_backers(&self, project_id: U256) -> Vec<Address> {
        let backers = self.project_backers.get(project_id);
        let mut result = Vec::new();
        for i in 0..backers.len() {
            if let Some(backer) = backers.get(i) {
                result.push(backer);
            }
        }
        result
    }

    pub fn get_project_milestones(&self, project_id: U256) -> Vec<Milestone> {
        let milestones = self.project_milestones.get(project_id);
        let mut result = Vec::new();
        for i in 0..milestones.len() {
            if let Some(milestone) = milestones.get(i) {
                result.push(milestone);
            }
        }
        result
    }

    pub fn calculate_revenue_share(&self, project_id: U256, contribution: U256) -> Result<U256> {
        let funding_info = self.project_funding.get(project_id);
        require_valid_input(funding_info.target > U256::from(0), "Project not found")?;
        
        if funding_info.raised == U256::from(0) {
            return Ok(U256::from(0));
        }
        
        // Calculate share as basis points (10000 = 100%)
        let share_bps = (contribution * U256::from(10000)) / funding_info.raised;
        Ok(share_bps)
    }

    pub fn platform_stats(&self) -> (U256, U256, U256, U256) {
        (
            self.total_projects_funded.get(),
            self.total_amount_raised.get(),
            self.total_backers.get(),
            self.platform_treasury.get(),
        )
    }

    // Admin functions
    pub fn add_authorized_caller(&mut self, caller: Address) -> Result<()> {
        self.require_owner()?;
        self.authorized_callers.insert(caller, true);
        Ok(())
    }

    pub fn set_platform_fee(&mut self, new_fee_bps: U256) -> Result<()> {
        self.require_owner()?;
        require_valid_input(new_fee_bps <= U256::from(1000), "Fee too high"); // Max 10%
        self.platform_fee_bps.set(new_fee_bps);
        Ok(())
    }

    pub fn emergency_withdraw(&mut self, project_id: U256) -> Result<()> {
        self.require_owner()?;
        let escrow_amount = self.project_escrow.get(project_id);
        if escrow_amount > U256::from(0) {
            call::transfer_eth(self.owner.get(), escrow_amount)?;
            self.project_escrow.insert(project_id, U256::from(0));
            
            evm::log(EmergencyWithdrawal {
                token: Address::ZERO, // ETH
                recipient: self.owner.get(),
                amount: escrow_amount,
            });
        }
        Ok(())
    }
}

// Internal helper functions
impl ProjectFunding {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_authorized_caller(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.platform_contract.get() || 
            caller == self.owner.get() ||
            self.authorized_callers.get(caller),
            "Not authorized caller"
        )
    }

    fn nonreentrant_guard(&mut self) -> Result<()> {
        require_valid_input(!self.locked.get(), "Reentrant call")?;
        self.locked.set(true);
        Ok(())
    }

    fn unlock_guard(&mut self) {
        self.locked.set(false);
    }

    fn get_funding_model(&self, project_id: U256) -> FundingModel {
        let model_u8 = self.funding_models.get(project_id).as_u8();
        match model_u8 {
            0 => FundingModel::AllOrNothing,
            1 => FundingModel::FlexibleFunding,
            2 => FundingModel::MilestoneBased,
            _ => FundingModel::AllOrNothing,
        }
    }

    fn mint_revenue_nft(
        &self,
        project_id: U256,
        backer: Address,
        funding_amount: U256,
        ens_data: String,
    ) -> Result<U256> {
        // Call revenue NFT contract to mint
        // This is a simplified version - would use actual contract call in production
        let share_bps = self.calculate_revenue_share(project_id, funding_amount)?;
        
        // For now, return a mock token ID
        let token_id = project_id * U256::from(10000) + funding_amount;
        
        evm::log(RevenueNFTMinted {
            token_id,
            project_id,
            recipient: backer,
            funding_amount,
            revenue_share_bps: share_bps,
        });
        
        Ok(token_id)
    }

    fn update_platform_funding(&self, project_id: U256, amount_raised: U256) -> Result<()> {
        // Would call platform contract in production
        // For now, just emit event
        Ok(())
    }

    fn transfer_to_creator(&self, creator: Address, amount: U256) -> Result<()> {
        if amount > U256::from(0) {
            call::transfer_eth(creator, amount)?;
        }
        Ok(())
    }
}