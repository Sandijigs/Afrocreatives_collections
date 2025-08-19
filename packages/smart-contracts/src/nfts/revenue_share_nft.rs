use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
    interfaces::{IERC721, IERC721Metadata},
};

#[derive(SolidityType, Clone, Debug)]
pub struct TokenRevenue {
    pub token_id: U256,
    pub project_id: U256,
    pub holder: Address,
    pub revenue_share_bps: U256, // Basis points (10000 = 100%)
    pub total_claimable: U256,
    pub total_claimed: U256,
    pub last_claim_timestamp: U256,
}

#[derive(SolidityType, Clone, Debug)]
pub struct RevenueStats {
    pub total_revenue_generated: U256,
    pub total_revenue_distributed: U256,
    pub holders_count: U256,
    pub last_distribution_timestamp: U256,
}

#[storage]
#[entrypoint]
pub struct RevenueShareNFT {
    // ERC721 standard storage
    name: StorageString,
    symbol: StorageString,
    owners: StorageMap<U256, Address>,
    balances: StorageMap<Address, U256>,
    token_approvals: StorageMap<U256, Address>,
    operator_approvals: StorageMap<Address, StorageMap<Address, bool>>,
    
    // Revenue sharing specific
    token_revenue_share: StorageMap<U256, U256>, // tokenId -> share in basis points
    token_project: StorageMap<U256, U256>,       // tokenId -> projectId
    token_funding_amount: StorageMap<U256, U256>, // tokenId -> original funding amount
    
    // Revenue tracking
    project_total_revenue: StorageMap<U256, U256>, // projectId -> total revenue received
    token_claimed_revenue: StorageMap<U256, U256>, // tokenId -> total claimed by holder
    token_claimable_revenue: StorageMap<U256, U256>, // tokenId -> currently claimable
    
    // ENS and metadata
    token_ens_metadata: StorageMap<U256, String>, // tokenId -> ENS metadata JSON
    token_uri_base: StorageString,
    
    // Project holders tracking
    project_holders: StorageMap<U256, StorageVec<U256>>, // projectId -> tokenIds[]
    project_holder_count: StorageMap<U256, U256>,
    
    // Revenue distribution tracking
    project_revenue_stats: StorageMap<U256, RevenueStats>,
    last_distribution_block: StorageMap<U256, U256>,
    
    // Contract management
    next_token_id: StorageU256,
    platform_contract: StorageAddress,
    funding_contract: StorageAddress,
    revenue_distributor: StorageAddress,
    
    // Access control
    owner: StorageAddress,
    minters: StorageMap<Address, bool>,
    
    // Transfer restrictions
    transfer_restrictions: StorageMap<U256, bool>, // tokenId -> restricted
    restriction_period: StorageU256, // Period during which transfers are restricted
    
    // Revenue settings
    min_claim_amount: StorageU256,
    claim_fee_bps: StorageU256, // Fee for claiming revenue (basis points)
    
    // Reentrancy guard
    locked: StorageBool,
}

#[public]
impl RevenueShareNFT {
    pub fn initialize(
        &mut self,
        name: String,
        symbol: String,
        platform_contract: Address,
        funding_contract: Address,
        base_uri: String,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.name.set(name);
        self.symbol.set(symbol);
        self.platform_contract.set(platform_contract);
        self.funding_contract.set(funding_contract);
        self.token_uri_base.set(base_uri);
        
        self.next_token_id.set(U256::from(1));
        self.restriction_period.set(U256::from(30 * 24 * 3600)); // 30 days
        self.min_claim_amount.set(U256::from(1000000000000000u64)); // 0.001 ETH
        self.claim_fee_bps.set(U256::from(100)); // 1%
        
        // Add authorized minters
        self.minters.insert(funding_contract, true);
        self.minters.insert(caller, true);
        
        Ok(())
    }

    pub fn mint_revenue_nft(
        &mut self,
        to: Address,
        project_id: U256,
        funding_amount: U256,
        revenue_share_bps: U256,
        ens_data: String,
    ) -> Result<U256> {
        self.require_minter()?;
        require_valid_input(!to.is_zero(), "Cannot mint to zero address")?;
        require_valid_input(funding_amount > U256::from(0), "Funding amount must be positive")?;
        require_valid_input(revenue_share_bps <= U256::from(10000), "Invalid revenue share")?;

        let token_id = self.next_token_id.get();
        
        // Mint the NFT
        self.owners.insert(token_id, to);
        let balance = self.balances.get(to);
        self.balances.insert(to, balance + U256::from(1));
        
        // Set revenue sharing data
        self.token_project.insert(token_id, project_id);
        self.token_funding_amount.insert(token_id, funding_amount);
        self.token_revenue_share.insert(token_id, revenue_share_bps);
        self.token_ens_metadata.insert(token_id, ens_data);
        
        // Add to project holders
        self.project_holders.get_mut(project_id).push(token_id);
        let holder_count = self.project_holder_count.get(project_id);
        self.project_holder_count.insert(project_id, holder_count + U256::from(1));
        
        // Set transfer restriction for initial period
        self.transfer_restrictions.insert(token_id, true);
        
        self.next_token_id.set(token_id + U256::from(1));

        evm::log(Transfer {
            from: Address::ZERO,
            to,
            token_id,
        });

        evm::log(RevenueNFTMinted {
            token_id,
            project_id,
            recipient: to,
            funding_amount,
            revenue_share_bps,
        });

        Ok(token_id)
    }

    pub fn calculate_claimable_revenue(&self, token_id: U256) -> Result<U256> {
        require_valid_input(self.owners.get(token_id) != Address::ZERO, "Token does not exist")?;
        
        let project_id = self.token_project.get(token_id);
        let revenue_share = self.token_revenue_share.get(token_id);
        let total_project_revenue = self.project_total_revenue.get(project_id);
        let already_claimed = self.token_claimed_revenue.get(token_id);
        
        // Calculate total entitled revenue
        let total_entitled = (total_project_revenue * revenue_share) / U256::from(10000);
        
        // Calculate claimable amount (total entitled - already claimed)
        if total_entitled > already_claimed {
            Ok(total_entitled - already_claimed)
        } else {
            Ok(U256::from(0))
        }
    }

    pub fn claim_revenue(&mut self, token_id: U256) -> Result<U256> {
        self.nonreentrant_guard()?;
        
        let holder = self.owners.get(token_id);
        require_authorized(msg::sender() == holder, "Not token owner")?;
        
        let claimable = self.calculate_claimable_revenue(token_id)?;
        require_valid_input(claimable >= self.min_claim_amount.get(), "Below minimum claim amount")?;
        
        // Calculate claim fee
        let fee = (claimable * self.claim_fee_bps.get()) / U256::from(10000);
        let net_amount = claimable - fee;
        
        // Update claimed amount
        let already_claimed = self.token_claimed_revenue.get(token_id);
        self.token_claimed_revenue.insert(token_id, already_claimed + claimable);
        
        // Transfer revenue to holder
        if net_amount > U256::from(0) {
            stylus_sdk::call::transfer_eth(holder, net_amount)?;
        }
        
        // Update claimable cache
        self.token_claimable_revenue.insert(token_id, U256::from(0));

        evm::log(RevenueClaimed {
            token_id,
            holder,
            amount: net_amount,
        });

        self.unlock_guard();
        Ok(net_amount)
    }

    pub fn batch_distribute_revenue(&mut self, project_id: U256, total_amount: U256) -> Result<()> {
        self.require_revenue_distributor()?;
        require_valid_input(total_amount > U256::from(0), "Amount must be positive")?;
        
        // Update project total revenue
        let current_revenue = self.project_total_revenue.get(project_id);
        self.project_total_revenue.insert(project_id, current_revenue + total_amount);
        
        // Update revenue statistics
        let mut stats = self.project_revenue_stats.get(project_id);
        stats.total_revenue_generated += total_amount;
        stats.last_distribution_timestamp = U256::from(block::timestamp());
        self.project_revenue_stats.insert(project_id, stats);
        
        // Update claimable amounts for all token holders
        let holders = self.project_holders.get(project_id);
        for i in 0..holders.len() {
            if let Some(token_id) = holders.get(i) {
                let claimable = self.calculate_claimable_revenue(token_id)?;
                self.token_claimable_revenue.insert(token_id, claimable);
            }
        }
        
        self.last_distribution_block.insert(project_id, U256::from(block::number()));

        evm::log(RevenueDistributed {
            project_id,
            total_amount,
            creator_share: U256::from(0), // Would be calculated based on project settings
            community_share: total_amount, // Simplified for this example
            platform_fee: U256::from(0),
        });

        Ok(())
    }

    pub fn remove_transfer_restriction(&mut self, token_id: U256) -> Result<()> {
        require_valid_input(self.owners.get(token_id) != Address::ZERO, "Token does not exist")?;
        
        let caller = msg::sender();
        let token_owner = self.owners.get(token_id);
        
        require_authorized(
            caller == token_owner || 
            caller == self.owner.get() ||
            self.minters.get(caller),
            "Not authorized"
        )?;
        
        self.transfer_restrictions.insert(token_id, false);
        Ok(())
    }

    // ERC721 Implementation
    pub fn balance_of(&self, owner: Address) -> Result<U256> {
        require_valid_input(!owner.is_zero(), "Zero address query")?;
        Ok(self.balances.get(owner))
    }

    pub fn owner_of(&self, token_id: U256) -> Result<Address> {
        let owner = self.owners.get(token_id);
        require_valid_input(!owner.is_zero(), "Token does not exist")?;
        Ok(owner)
    }

    pub fn approve(&mut self, to: Address, token_id: U256) -> Result<()> {
        let owner = self.owners.get(token_id);
        require_valid_input(!owner.is_zero(), "Token does not exist")?;
        require_valid_input(to != owner, "Approval to current owner")?;
        
        let caller = msg::sender();
        require_authorized(
            caller == owner || self.is_approved_for_all(owner, caller),
            "Not owner or approved operator"
        )?;
        
        self.token_approvals.insert(token_id, to);
        
        evm::log(Approval {
            owner,
            approved: to,
            token_id,
        });
        
        Ok(())
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) -> Result<()> {
        require_valid_input(self.is_approved_or_owner(msg::sender(), token_id)?, "Not authorized")?;
        require_valid_input(!self.transfer_restrictions.get(token_id), "Transfer restricted")?;
        
        self.transfer(from, to, token_id)
    }

    pub fn safe_transfer_from(&mut self, from: Address, to: Address, token_id: U256) -> Result<()> {
        self.transfer_from(from, to, token_id)
    }

    // View functions
    pub fn get_revenue_stats(&self, token_id: U256) -> Result<TokenRevenue> {
        require_valid_input(self.owners.get(token_id) != Address::ZERO, "Token does not exist")?;
        
        let project_id = self.token_project.get(token_id);
        let holder = self.owners.get(token_id);
        let revenue_share = self.token_revenue_share.get(token_id);
        let claimable = self.calculate_claimable_revenue(token_id)?;
        let claimed = self.token_claimed_revenue.get(token_id);
        
        Ok(TokenRevenue {
            token_id,
            project_id,
            holder,
            revenue_share_bps: revenue_share,
            total_claimable: claimable,
            total_claimed: claimed,
            last_claim_timestamp: U256::from(0), // Would track actual claim times
        })
    }

    pub fn get_project_holders(&self, project_id: U256) -> Vec<U256> {
        let holders = self.project_holders.get(project_id);
        let mut result = Vec::new();
        for i in 0..holders.len() {
            if let Some(token_id) = holders.get(i) {
                result.push(token_id);
            }
        }
        result
    }

    pub fn get_project_revenue_stats(&self, project_id: U256) -> RevenueStats {
        self.project_revenue_stats.get(project_id)
    }

    pub fn token_uri(&self, token_id: U256) -> Result<String> {
        require_valid_input(self.owners.get(token_id) != Address::ZERO, "Token does not exist")?;
        
        let base_uri = self.token_uri_base.get();
        let project_id = self.token_project.get(token_id);
        
        Ok(format!("{}/{}/{}", base_uri, project_id, token_id))
    }

    pub fn name(&self) -> String {
        self.name.get()
    }

    pub fn symbol(&self) -> String {
        self.symbol.get()
    }

    // Admin functions
    pub fn set_revenue_distributor(&mut self, distributor: Address) -> Result<()> {
        self.require_owner()?;
        self.revenue_distributor.set(distributor);
        Ok(())
    }

    pub fn add_minter(&mut self, minter: Address) -> Result<()> {
        self.require_owner()?;
        self.minters.insert(minter, true);
        Ok(())
    }

    pub fn set_base_uri(&mut self, new_base_uri: String) -> Result<()> {
        self.require_owner()?;
        self.token_uri_base.set(new_base_uri);
        Ok(())
    }

    pub fn set_min_claim_amount(&mut self, amount: U256) -> Result<()> {
        self.require_owner()?;
        self.min_claim_amount.set(amount);
        Ok(())
    }
}

// Internal helper functions
impl RevenueShareNFT {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_minter(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            self.minters.get(caller) || caller == self.owner.get(),
            "Not authorized minter"
        )
    }

    fn require_revenue_distributor(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.revenue_distributor.get() || 
            caller == self.owner.get(),
            "Not authorized distributor"
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

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.operator_approvals.get(owner).get(operator)
    }

    fn is_approved_or_owner(&self, spender: Address, token_id: U256) -> Result<bool> {
        let owner = self.owners.get(token_id);
        require_valid_input(!owner.is_zero(), "Token does not exist")?;
        
        Ok(spender == owner || 
           self.get_approved(token_id) == spender || 
           self.is_approved_for_all(owner, spender))
    }

    fn get_approved(&self, token_id: U256) -> Address {
        self.token_approvals.get(token_id)
    }

    fn transfer(&mut self, from: Address, to: Address, token_id: U256) -> Result<()> {
        require_valid_input(!to.is_zero(), "Transfer to zero address")?;
        require_valid_input(self.owners.get(token_id) == from, "Transfer from incorrect owner")?;
        
        // Clear approval
        self.token_approvals.insert(token_id, Address::ZERO);
        
        // Update balances
        let from_balance = self.balances.get(from);
        self.balances.insert(from, from_balance - U256::from(1));
        
        let to_balance = self.balances.get(to);
        self.balances.insert(to, to_balance + U256::from(1));
        
        // Transfer ownership
        self.owners.insert(token_id, to);

        evm::log(Transfer {
            from,
            to,
            token_id,
        });

        Ok(())
    }
}