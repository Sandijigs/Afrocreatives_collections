use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageString, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
};

#[derive(SolidityType, Clone, Debug)]
pub struct CulturalBadge {
    pub badge_id: U256,
    pub badge_type: String, // "Validator", "Creator", "Cultural Expert", etc.
    pub cultural_region: String,
    pub expertise_level: u8, // 1-5 scale
    pub issued_by: Address,
    pub issued_timestamp: U256,
    pub metadata_uri: String,
    pub is_transferable: bool,
}

#[storage]
#[entrypoint]
pub struct CulturalBadgeNFT {
    // ERC721 basics
    name: StorageString,
    symbol: StorageString,
    owners: StorageMap<U256, Address>,
    balances: StorageMap<Address, U256>,
    token_approvals: StorageMap<U256, Address>,
    operator_approvals: StorageMap<Address, StorageMap<Address, bool>>,
    
    // Badge-specific data
    badges: StorageMap<U256, CulturalBadge>,
    user_badges: StorageMap<Address, StorageVec<U256>>,
    badge_types: StorageVec<String>,
    
    // Badge issuance tracking
    next_badge_id: StorageU256,
    badge_issuers: StorageMap<Address, bool>,
    
    // Platform integration
    cultural_validator: StorageAddress,
    platform_contract: StorageAddress,
    
    // Access control
    owner: StorageAddress,
    
    // Base URI for metadata
    base_uri: StorageString,
}

#[public]
impl CulturalBadgeNFT {
    pub fn initialize(
        &mut self,
        name: String,
        symbol: String,
        base_uri: String,
        cultural_validator: Address,
        platform_contract: Address,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.name.set(name);
        self.symbol.set(symbol);
        self.base_uri.set(base_uri);
        self.cultural_validator.set(cultural_validator);
        self.platform_contract.set(platform_contract);
        
        self.next_badge_id.set(U256::from(1));
        
        // Initialize badge types
        self.initialize_badge_types();
        
        // Add owner as initial badge issuer
        self.badge_issuers.insert(caller, true);
        
        Ok(())
    }

    pub fn mint_cultural_badge(
        &mut self,
        to: Address,
        badge_type: String,
        cultural_region: String,
        expertise_level: u8,
        metadata_uri: String,
        is_transferable: bool,
    ) -> Result<U256> {
        self.require_badge_issuer()?;
        
        require_valid_input(!to.is_zero(), "Cannot mint to zero address")?;
        require_valid_input(expertise_level >= 1 && expertise_level <= 5, "Invalid expertise level")?;
        require_valid_input(self.is_valid_badge_type(&badge_type), "Invalid badge type")?;
        
        let badge_id = self.next_badge_id.get();
        let issuer = msg::sender();
        
        let badge = CulturalBadge {
            badge_id,
            badge_type,
            cultural_region,
            expertise_level,
            issued_by: issuer,
            issued_timestamp: U256::from(block::timestamp()),
            metadata_uri,
            is_transferable,
        };
        
        // Mint the NFT
        self.owners.insert(badge_id, to);
        let balance = self.balances.get(to);
        self.balances.insert(to, balance + U256::from(1));
        
        // Store badge data
        self.badges.insert(badge_id, badge);
        self.user_badges.get_mut(to).push(badge_id);
        
        self.next_badge_id.set(badge_id + U256::from(1));

        evm::log(Transfer {
            from: Address::ZERO,
            to,
            token_id: badge_id,
        });

        Ok(badge_id)
    }

    // ERC721 standard functions
    pub fn balance_of(&self, owner: Address) -> Result<U256> {
        require_valid_input(!owner.is_zero(), "Zero address query")?;
        Ok(self.balances.get(owner))
    }

    pub fn owner_of(&self, token_id: U256) -> Result<Address> {
        let owner = self.owners.get(token_id);
        require_valid_input(!owner.is_zero(), "Token does not exist")?;
        Ok(owner)
    }

    pub fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) -> Result<()> {
        require_valid_input(self.is_approved_or_owner(msg::sender(), token_id)?, "Not authorized")?;
        
        let badge = self.badges.get(token_id);
        require_valid_input(badge.is_transferable, "Badge is not transferable")?;
        
        self.transfer(from, to, token_id)
    }

    pub fn token_uri(&self, token_id: U256) -> Result<String> {
        require_valid_input(self.owners.get(token_id) != Address::ZERO, "Token does not exist")?;
        
        let badge = self.badges.get(token_id);
        if !badge.metadata_uri.is_empty() {
            Ok(badge.metadata_uri)
        } else {
            Ok(format!("{}/{}", self.base_uri.get(), token_id))
        }
    }

    // Badge-specific view functions
    pub fn get_badge(&self, badge_id: U256) -> Result<CulturalBadge> {
        let badge = self.badges.get(badge_id);
        require_valid_input(badge.badge_id != U256::from(0), "Badge not found")?;
        Ok(badge)
    }

    pub fn get_user_badges(&self, user: Address) -> Vec<U256> {
        let badges = self.user_badges.get(user);
        let mut result = Vec::new();
        for i in 0..badges.len() {
            if let Some(badge_id) = badges.get(i) {
                result.push(badge_id);
            }
        }
        result
    }

    pub fn has_badge_type(&self, user: Address, badge_type: String) -> bool {
        let user_badges = self.user_badges.get(user);
        for i in 0..user_badges.len() {
            if let Some(badge_id) = user_badges.get(i) {
                let badge = self.badges.get(badge_id);
                if badge.badge_type == badge_type {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_expertise_level(&self, user: Address, cultural_region: String) -> u8 {
        let user_badges = self.user_badges.get(user);
        let mut max_level = 0u8;
        
        for i in 0..user_badges.len() {
            if let Some(badge_id) = user_badges.get(i) {
                let badge = self.badges.get(badge_id);
                if badge.cultural_region == cultural_region && badge.expertise_level > max_level {
                    max_level = badge.expertise_level;
                }
            }
        }
        
        max_level
    }

    // Admin functions
    pub fn add_badge_issuer(&mut self, issuer: Address) -> Result<()> {
        self.require_owner()?;
        self.badge_issuers.insert(issuer, true);
        Ok(())
    }

    pub fn revoke_badge(&mut self, badge_id: U256) -> Result<()> {
        self.require_badge_issuer()?;
        
        let badge = self.badges.get(badge_id);
        require_valid_input(badge.badge_id != U256::from(0), "Badge not found")?;
        
        let owner = self.owners.get(badge_id);
        if !owner.is_zero() {
            // Burn the token
            self.owners.insert(badge_id, Address::ZERO);
            let balance = self.balances.get(owner);
            if balance > U256::from(0) {
                self.balances.insert(owner, balance - U256::from(1));
            }
            
            evm::log(Transfer {
                from: owner,
                to: Address::ZERO,
                token_id: badge_id,
            });
        }
        
        Ok(())
    }

    pub fn name(&self) -> String {
        self.name.get()
    }

    pub fn symbol(&self) -> String {
        self.symbol.get()
    }
}

// Internal helper functions
impl CulturalBadgeNFT {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_badge_issuer(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            self.badge_issuers.get(caller) || caller == self.owner.get(),
            "Not authorized badge issuer"
        )
    }

    fn is_valid_badge_type(&self, badge_type: &str) -> bool {
        for i in 0..self.badge_types.len() {
            if let Some(valid_type) = self.badge_types.get(i) {
                if valid_type == badge_type {
                    return true;
                }
            }
        }
        false
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

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.operator_approvals.get(owner).get(operator)
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
        
        // Update user badge tracking
        self.user_badges.get_mut(to).push(token_id);

        evm::log(Transfer {
            from,
            to,
            token_id,
        });

        Ok(())
    }

    fn initialize_badge_types(&mut self) {
        let types = vec![
            "Cultural Validator",
            "Regional Expert", 
            "Traditional Craftsperson",
            "Cultural Educator",
            "Community Leader",
            "Heritage Keeper",
            "Language Specialist",
            "Music Traditionalist",
            "Storyteller",
            "Cultural Ambassador",
        ];
        
        for badge_type in types {
            self.badge_types.push(badge_type.to_string());
        }
    }
}