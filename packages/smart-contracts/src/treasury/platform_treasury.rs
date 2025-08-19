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
pub struct TreasuryAllocation {
    pub allocation_id: U256,
    pub purpose: String,
    pub amount: U256,
    pub recipient: Address,
    pub approved_by: Address,
    pub timestamp: U256,
    pub executed: bool,
}

#[storage]
#[entrypoint]
pub struct PlatformTreasury {
    // Treasury balance tracking
    total_balance: StorageU256,
    platform_fees_collected: StorageU256,
    cultural_fund_balance: StorageU256,
    operational_fund_balance: StorageU256,
    
    // Fee allocation percentages (in basis points)
    cultural_fund_allocation_bps: StorageU256,
    operational_fund_allocation_bps: StorageU256,
    reserve_fund_allocation_bps: StorageU256,
    
    // Allocation tracking
    allocations: StorageMap<U256, TreasuryAllocation>,
    next_allocation_id: StorageU256,
    
    // Governance integration
    governance_contract: StorageAddress,
    platform_contract: StorageAddress,
    
    // Access control
    owner: StorageAddress,
    treasury_managers: StorageMap<Address, bool>,
    
    // Spending limits
    daily_spending_limit: StorageU256,
    daily_spent: StorageMap<U256, U256>, // day -> amount spent
    
    // Emergency controls
    emergency_pause: StorageBool,
    emergency_withdrawal_enabled: StorageBool,
}

#[public]
impl PlatformTreasury {
    pub fn initialize(
        &mut self,
        governance_contract: Address,
        platform_contract: Address,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.governance_contract.set(governance_contract);
        self.platform_contract.set(platform_contract);
        
        // Set default allocation percentages
        self.cultural_fund_allocation_bps.set(U256::from(4000)); // 40%
        self.operational_fund_allocation_bps.set(U256::from(4000)); // 40%
        self.reserve_fund_allocation_bps.set(U256::from(2000)); // 20%
        
        // Set default spending limit
        self.daily_spending_limit.set(U256::from(1000000000000000000u64)); // 1 ETH per day
        
        self.next_allocation_id.set(U256::from(1));
        
        Ok(())
    }

    #[payable]
    pub fn receive_platform_fees(&mut self) -> Result<()> {
        let amount = msg::value();
        require_valid_input(amount > U256::from(0), "No funds received")?;
        
        // Update total balance
        self.total_balance.set(self.total_balance.get() + amount);
        self.platform_fees_collected.set(self.platform_fees_collected.get() + amount);
        
        // Allocate to different funds
        let cultural_allocation = (amount * self.cultural_fund_allocation_bps.get()) / U256::from(10000);
        let operational_allocation = (amount * self.operational_fund_allocation_bps.get()) / U256::from(10000);
        let reserve_allocation = amount - cultural_allocation - operational_allocation;
        
        self.cultural_fund_balance.set(self.cultural_fund_balance.get() + cultural_allocation);
        self.operational_fund_balance.set(self.operational_fund_balance.get() + operational_allocation);
        
        Ok(())
    }

    pub fn create_allocation(
        &mut self,
        purpose: String,
        amount: U256,
        recipient: Address,
    ) -> Result<U256> {
        self.require_treasury_manager()?;
        self.require_not_paused()?;
        
        require_valid_input(amount <= self.operational_fund_balance.get(), "Insufficient operational funds")?;
        require_valid_input(!recipient.is_zero(), "Invalid recipient")?;
        
        // Check daily spending limit
        let today = U256::from(block::timestamp()) / U256::from(24 * 3600);
        let today_spent = self.daily_spent.get(today);
        require_valid_input(
            today_spent + amount <= self.daily_spending_limit.get(),
            "Daily spending limit exceeded"
        )?;
        
        let allocation_id = self.next_allocation_id.get();
        let allocation = TreasuryAllocation {
            allocation_id,
            purpose,
            amount,
            recipient,
            approved_by: msg::sender(),
            timestamp: U256::from(block::timestamp()),
            executed: false,
        };
        
        self.allocations.insert(allocation_id, allocation);
        self.next_allocation_id.set(allocation_id + U256::from(1));
        
        // Reserve the funds
        self.operational_fund_balance.set(self.operational_fund_balance.get() - amount);
        
        Ok(allocation_id)
    }

    pub fn execute_allocation(&mut self, allocation_id: U256) -> Result<()> {
        self.require_treasury_manager()?;
        self.require_not_paused()?;
        
        let mut allocation = self.allocations.get(allocation_id);
        require_valid_input(allocation.allocation_id != U256::from(0), "Allocation not found")?;
        require_valid_input(!allocation.executed, "Already executed")?;
        
        // Transfer funds to recipient
        stylus_sdk::call::transfer_eth(allocation.recipient, allocation.amount)?;
        
        // Mark as executed
        allocation.executed = true;
        self.allocations.insert(allocation_id, allocation.clone());
        
        // Update daily spending
        let today = U256::from(block::timestamp()) / U256::from(24 * 3600);
        let today_spent = self.daily_spent.get(today);
        self.daily_spent.insert(today, today_spent + allocation.amount);

        evm::log(EmergencyWithdrawal {
            token: Address::ZERO, // ETH
            recipient: allocation.recipient,
            amount: allocation.amount,
        });

        Ok(())
    }

    pub fn allocate_cultural_fund(
        &mut self,
        recipient: Address,
        amount: U256,
        purpose: String,
    ) -> Result<()> {
        self.require_governance_approval()?;
        
        require_valid_input(amount <= self.cultural_fund_balance.get(), "Insufficient cultural fund")?;
        require_valid_input(!recipient.is_zero(), "Invalid recipient")?;
        
        // Transfer funds
        stylus_sdk::call::transfer_eth(recipient, amount)?;
        
        // Update balance
        self.cultural_fund_balance.set(self.cultural_fund_balance.get() - amount);
        
        Ok(())
    }

    // View functions
    pub fn treasury_stats(&self) -> (U256, U256, U256, U256) {
        (
            self.total_balance.get(),
            self.cultural_fund_balance.get(),
            self.operational_fund_balance.get(),
            self.platform_fees_collected.get(),
        )
    }

    pub fn get_allocation(&self, allocation_id: U256) -> Result<TreasuryAllocation> {
        let allocation = self.allocations.get(allocation_id);
        require_valid_input(allocation.allocation_id != U256::from(0), "Allocation not found")?;
        Ok(allocation)
    }

    pub fn get_daily_spending(&self) -> (U256, U256) {
        let today = U256::from(block::timestamp()) / U256::from(24 * 3600);
        let spent_today = self.daily_spent.get(today);
        (spent_today, self.daily_spending_limit.get())
    }

    pub fn available_operational_funds(&self) -> U256 {
        self.operational_fund_balance.get()
    }

    pub fn available_cultural_funds(&self) -> U256 {
        self.cultural_fund_balance.get()
    }

    // Admin functions
    pub fn update_allocation_percentages(
        &mut self,
        cultural_bps: U256,
        operational_bps: U256,
        reserve_bps: U256,
    ) -> Result<()> {
        self.require_owner()?;
        
        require_valid_input(
            cultural_bps + operational_bps + reserve_bps == U256::from(10000),
            "Percentages must sum to 100%"
        )?;
        
        self.cultural_fund_allocation_bps.set(cultural_bps);
        self.operational_fund_allocation_bps.set(operational_bps);
        self.reserve_fund_allocation_bps.set(reserve_bps);
        
        Ok(())
    }

    pub fn add_treasury_manager(&mut self, manager: Address) -> Result<()> {
        self.require_owner()?;
        self.treasury_managers.insert(manager, true);
        Ok(())
    }

    pub fn set_daily_spending_limit(&mut self, limit: U256) -> Result<()> {
        self.require_owner()?;
        self.daily_spending_limit.set(limit);
        Ok(())
    }

    pub fn emergency_pause(&mut self) -> Result<()> {
        self.require_owner()?;
        self.emergency_pause.set(true);
        Ok(())
    }

    pub fn emergency_withdrawal(&mut self, amount: U256) -> Result<()> {
        self.require_owner()?;
        require_valid_input(self.emergency_withdrawal_enabled.get(), "Emergency withdrawal not enabled")?;
        require_valid_input(amount <= self.total_balance.get(), "Insufficient balance")?;
        
        stylus_sdk::call::transfer_eth(self.owner.get(), amount)?;
        self.total_balance.set(self.total_balance.get() - amount);
        
        evm::log(EmergencyWithdrawal {
            token: Address::ZERO,
            recipient: self.owner.get(),
            amount,
        });
        
        Ok(())
    }
}

// Internal helper functions
impl PlatformTreasury {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_treasury_manager(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.owner.get() || self.treasury_managers.get(caller),
            "Only treasury manager"
        )
    }

    fn require_governance_approval(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.governance_contract.get() || caller == self.owner.get(),
            "Requires governance approval"
        )
    }

    fn require_not_paused(&self) -> Result<()> {
        require_valid_input(!self.emergency_pause.get(), "Treasury is paused")
    }
}