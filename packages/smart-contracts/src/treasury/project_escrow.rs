use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU256},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
};

#[derive(SolidityType, Clone, Debug)]
pub struct EscrowAccount {
    pub project_id: U256,
    pub creator: Address,
    pub total_funded: U256,
    pub released_amount: U256,
    pub is_active: bool,
    pub creation_timestamp: U256,
}

#[storage]
#[entrypoint]
pub struct ProjectEscrow {
    // Escrow accounts
    project_escrows: StorageMap<U256, EscrowAccount>,
    creator_escrows: StorageMap<Address, U256>, // creator -> total escrowed
    
    // Platform integration
    platform_contract: StorageAddress,
    funding_contract: StorageAddress,
    
    // Access control
    owner: StorageAddress,
    authorized_releasers: StorageMap<Address, bool>,
    
    // Emergency controls
    emergency_pause: StorageBool,
    
    // Metrics
    total_escrowed: StorageU256,
    total_released: StorageU256,
    active_escrow_count: StorageU256,
}

#[public]
impl ProjectEscrow {
    pub fn initialize(
        &mut self,
        platform_contract: Address,
        funding_contract: Address,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.platform_contract.set(platform_contract);
        self.funding_contract.set(funding_contract);
        
        // Add funding contract as authorized releaser
        self.authorized_releasers.insert(funding_contract, true);
        
        Ok(())
    }

    #[payable]
    pub fn create_escrow(&mut self, project_id: U256, creator: Address) -> Result<()> {
        self.require_authorized_contract()?;
        self.require_not_paused()?;
        
        let amount = msg::value();
        require_valid_input(amount > U256::from(0), "No funds provided")?;
        require_valid_input(!creator.is_zero(), "Invalid creator address")?;
        
        let mut escrow = self.project_escrows.get(project_id);
        
        if escrow.project_id == U256::from(0) {
            // Create new escrow
            escrow = EscrowAccount {
                project_id,
                creator,
                total_funded: amount,
                released_amount: U256::from(0),
                is_active: true,
                creation_timestamp: U256::from(block::timestamp()),
            };
            
            self.active_escrow_count.set(self.active_escrow_count.get() + U256::from(1));
        } else {
            // Add to existing escrow
            escrow.total_funded += amount;
        }
        
        self.project_escrows.insert(project_id, escrow);
        
        // Update creator total
        let creator_total = self.creator_escrows.get(creator);
        self.creator_escrows.insert(creator, creator_total + amount);
        
        // Update global metrics
        self.total_escrowed.set(self.total_escrowed.get() + amount);
        
        Ok(())
    }

    pub fn release_funds(&mut self, project_id: U256, amount: U256) -> Result<()> {
        self.require_authorized_releaser()?;
        self.require_not_paused()?;
        
        let mut escrow = self.project_escrows.get(project_id);
        require_valid_input(escrow.project_id != U256::from(0), "Escrow not found")?;
        require_valid_input(escrow.is_active, "Escrow not active")?;
        
        let available = escrow.total_funded - escrow.released_amount;
        require_valid_input(amount <= available, "Insufficient escrowed funds")?;
        
        // Transfer funds to creator
        stylus_sdk::call::transfer_eth(escrow.creator, amount)?;
        
        // Update escrow
        escrow.released_amount += amount;
        self.project_escrows.insert(project_id, escrow);
        
        // Update global metrics
        self.total_released.set(self.total_released.get() + amount);
        
        Ok(())
    }

    pub fn process_refund(&mut self, project_id: U256, backer: Address, amount: U256) -> Result<()> {
        self.require_authorized_releaser()?;
        self.require_not_paused()?;
        
        let escrow = self.project_escrows.get(project_id);
        require_valid_input(escrow.project_id != U256::from(0), "Escrow not found")?;
        
        let available = escrow.total_funded - escrow.released_amount;
        require_valid_input(amount <= available, "Insufficient funds for refund")?;
        
        // Transfer refund to backer
        stylus_sdk::call::transfer_eth(backer, amount)?;
        
        // Update escrow (treat as release for accounting)
        let mut updated_escrow = escrow;
        updated_escrow.released_amount += amount;
        self.project_escrows.insert(project_id, updated_escrow);
        
        self.total_released.set(self.total_released.get() + amount);
        
        Ok(())
    }

    pub fn close_escrow(&mut self, project_id: U256) -> Result<()> {
        self.require_authorized_releaser()?;
        
        let mut escrow = self.project_escrows.get(project_id);
        require_valid_input(escrow.project_id != U256::from(0), "Escrow not found")?;
        require_valid_input(escrow.is_active, "Escrow already closed")?;
        
        // Close the escrow
        escrow.is_active = false;
        self.project_escrows.insert(project_id, escrow);
        
        self.active_escrow_count.set(self.active_escrow_count.get() - U256::from(1));
        
        Ok(())
    }

    // View functions
    pub fn get_escrow(&self, project_id: U256) -> Result<EscrowAccount> {
        let escrow = self.project_escrows.get(project_id);
        require_valid_input(escrow.project_id != U256::from(0), "Escrow not found")?;
        Ok(escrow)
    }

    pub fn get_available_funds(&self, project_id: U256) -> Result<U256> {
        let escrow = self.project_escrows.get(project_id);
        require_valid_input(escrow.project_id != U256::from(0), "Escrow not found")?;
        
        Ok(escrow.total_funded - escrow.released_amount)
    }

    pub fn get_creator_total_escrow(&self, creator: Address) -> U256 {
        self.creator_escrows.get(creator)
    }

    pub fn escrow_stats(&self) -> (U256, U256, U256) {
        (
            self.total_escrowed.get(),
            self.total_released.get(),
            self.active_escrow_count.get(),
        )
    }

    // Admin functions
    pub fn add_authorized_releaser(&mut self, releaser: Address) -> Result<()> {
        self.require_owner()?;
        self.authorized_releasers.insert(releaser, true);
        Ok(())
    }

    pub fn remove_authorized_releaser(&mut self, releaser: Address) -> Result<()> {
        self.require_owner()?;
        self.authorized_releasers.insert(releaser, false);
        Ok(())
    }

    pub fn emergency_pause(&mut self) -> Result<()> {
        self.require_owner()?;
        self.emergency_pause.set(true);
        Ok(())
    }

    pub fn emergency_release(&mut self, project_id: U256, recipient: Address) -> Result<()> {
        self.require_owner()?;
        
        let escrow = self.project_escrows.get(project_id);
        require_valid_input(escrow.project_id != U256::from(0), "Escrow not found")?;
        
        let available = escrow.total_funded - escrow.released_amount;
        require_valid_input(available > U256::from(0), "No funds available")?;
        
        // Emergency release all available funds
        stylus_sdk::call::transfer_eth(recipient, available)?;
        
        // Update escrow
        let mut updated_escrow = escrow;
        updated_escrow.released_amount = escrow.total_funded;
        updated_escrow.is_active = false;
        self.project_escrows.insert(project_id, updated_escrow);
        
        self.total_released.set(self.total_released.get() + available);
        
        evm::log(EmergencyWithdrawal {
            token: Address::ZERO,
            recipient,
            amount: available,
        });
        
        Ok(())
    }
}

// Internal helper functions
impl ProjectEscrow {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_authorized_contract(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.funding_contract.get() || 
            caller == self.platform_contract.get() ||
            caller == self.owner.get(),
            "Not authorized contract"
        )
    }

    fn require_authorized_releaser(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            self.authorized_releasers.get(caller) || caller == self.owner.get(),
            "Not authorized releaser"
        )
    }

    fn require_not_paused(&self) -> Result<()> {
        require_valid_input(!self.emergency_pause.get(), "Escrow is paused")
    }
}