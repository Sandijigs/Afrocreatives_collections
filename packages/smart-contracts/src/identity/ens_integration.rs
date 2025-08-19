use alloy_primitives::{Address, FixedBytes, U256};
use stylus_sdk::{
    block, call, contract, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageString, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
    interfaces::{ENSRegistry, ENSResolver},
    AFROCREATE_ENS_NODE,
};

#[storage]
#[entrypoint]
pub struct ENSIntegration {
    // ENS contracts
    ens_registry: StorageAddress,
    default_resolver: StorageAddress,
    
    // Node management  
    afrocreate_node: StorageString, // Store as string for simplicity
    creator_nodes: StorageMap<Address, FixedBytes<32>>,
    project_nodes: StorageMap<U256, FixedBytes<32>>,
    node_owners: StorageMap<FixedBytes<32>, Address>,
    
    // Text record standardized keys
    cultural_keys: StorageVec<String>,
    platform_keys: StorageVec<String>,
    
    // Metadata storage (on-chain cache)
    node_cultural_data: StorageMap<FixedBytes<32>, StorageMap<String, String>>,
    node_platform_data: StorageMap<FixedBytes<32>, StorageMap<String, String>>,
    
    // Subdomain registry
    subdomain_to_node: StorageMap<String, FixedBytes<32>>,
    node_to_subdomain: StorageMap<FixedBytes<32>, String>,
    
    // Access control
    owner: StorageAddress,
    authorized_updaters: StorageMap<Address, bool>,
    platform_contract: StorageAddress,
}

#[public]
impl ENSIntegration {
    pub fn initialize(
        &mut self,
        ens_registry: Address,
        default_resolver: Address,
        platform_contract: Address,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.ens_registry.set(ens_registry);
        self.default_resolver.set(default_resolver);
        self.platform_contract.set(platform_contract);
        self.afrocreate_node.set("afrocreate.eth".to_string());
        
        // Initialize standardized text record keys
        self.initialize_cultural_keys();
        self.initialize_platform_keys();
        
        Ok(())
    }

    pub fn register_creator_subdomain(
        &mut self,
        creator: Address,
        subdomain: String,
        cultural_data: Vec<(String, String)>, // key-value pairs
    ) -> Result<FixedBytes<32>> {
        self.require_authorized()?;
        
        require_valid_input(
            !self.creator_nodes.get(creator).is_zero() == false,
            "Creator already has subdomain"
        )?;
        
        // Generate node hash (simplified)
        let node = self.generate_node_hash(&subdomain)?;
        
        // Register subdomain with ENS (would call actual ENS registry in production)
        // For now, just store the mapping
        self.subdomain_to_node.insert(subdomain.clone(), node);
        self.node_to_subdomain.insert(node, subdomain.clone());
        self.node_owners.insert(node, creator);
        self.creator_nodes.insert(creator, node);
        
        // Set cultural metadata
        for (key, value) in cultural_data.iter() {
            self.set_cultural_metadata(node, key.clone(), value.clone())?;
        }
        
        // Initialize reputation score
        self.set_platform_metadata(node, "reputation_score".to_string(), "100".to_string())?;
        self.set_platform_metadata(node, "projects_count".to_string(), "0".to_string())?;
        self.set_platform_metadata(node, "total_funding".to_string(), "0".to_string())?;

        evm::log(ENSSubdomainRegistered {
            node,
            subdomain,
            owner: creator,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(node)
    }

    pub fn register_project_subdomain(
        &mut self,
        project_id: U256,
        creator: Address,
        name: String,
        metadata: Vec<(String, String)>,
    ) -> Result<FixedBytes<32>> {
        self.require_authorized()?;
        
        // Ensure creator exists
        let creator_node = self.creator_nodes.get(creator);
        require_valid_input(!creator_node.is_zero(), "Creator not found")?;
        
        let project_subdomain = format!("project-{}", project_id);
        let node = self.generate_node_hash(&project_subdomain)?;
        
        self.project_nodes.insert(project_id, node);
        self.subdomain_to_node.insert(project_subdomain.clone(), node);
        self.node_to_subdomain.insert(node, project_subdomain.clone());
        self.node_owners.insert(node, creator);
        
        // Set project metadata
        for (key, value) in metadata.iter() {
            self.set_platform_metadata(node, key.clone(), value.clone())?;
        }
        
        // Link to creator
        self.set_platform_metadata(node, "creator".to_string(), format!("{:?}", creator))?;

        evm::log(ENSSubdomainRegistered {
            node,
            subdomain: project_subdomain,
            owner: creator,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(node)
    }

    pub fn update_cultural_metadata(
        &mut self,
        node: FixedBytes<32>,
        key: String,
        value: String,
    ) -> Result<()> {
        self.require_node_owner_or_authorized(node)?;
        self.validate_cultural_key(&key)?;
        
        self.set_cultural_metadata(node, key.clone(), value.clone())?;

        evm::log(CulturalMetadataUpdated {
            node,
            key,
            value,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(())
    }

    pub fn update_reputation_score(&mut self, creator_node: FixedBytes<32>, new_score: U256) -> Result<()> {
        self.require_authorized()?;
        
        let old_score_str = self.get_platform_metadata(creator_node, "reputation_score".to_string())?;
        let old_score = old_score_str.parse::<u64>().unwrap_or(0);
        
        self.set_platform_metadata(creator_node, "reputation_score".to_string(), new_score.to_string())?;

        evm::log(ReputationUpdated {
            node: creator_node,
            old_score: U256::from(old_score),
            new_score,
        });

        Ok(())
    }

    pub fn batch_update_text_records(
        &mut self,
        nodes: Vec<FixedBytes<32>>,
        keys: Vec<String>,
        values: Vec<String>,
    ) -> Result<()> {
        require_valid_input(
            nodes.len() == keys.len() && keys.len() == values.len(),
            "Array lengths mismatch"
        )?;

        for i in 0..nodes.len() {
            let node = nodes[i];
            let key = &keys[i];
            let value = &values[i];
            
            self.require_node_owner_or_authorized(node)?;
            
            if self.is_cultural_key(key) {
                self.set_cultural_metadata(node, key.clone(), value.clone())?;
            } else if self.is_platform_key(key) {
                self.set_platform_metadata(node, key.clone(), value.clone())?;
            } else {
                return Err(AfroCreateError::InvalidInput("Invalid metadata key".to_string()));
            }
        }

        Ok(())
    }

    // View functions
    pub fn resolve_creator_by_ens(&self, ens_name: String) -> Result<Address> {
        let node = self.subdomain_to_node.get(ens_name);
        require_valid_input(!node.is_zero(), "ENS name not found")?;
        
        let owner = self.node_owners.get(node);
        require_valid_input(!owner.is_zero(), "Node has no owner")?;
        
        Ok(owner)
    }

    pub fn get_creator_node(&self, creator: Address) -> Result<FixedBytes<32>> {
        let node = self.creator_nodes.get(creator);
        require_valid_input(!node.is_zero(), "Creator node not found")?;
        Ok(node)
    }

    pub fn get_project_node(&self, project_id: U256) -> Result<FixedBytes<32>> {
        let node = self.project_nodes.get(project_id);
        require_valid_input(!node.is_zero(), "Project node not found")?;
        Ok(node)
    }

    pub fn get_cultural_data(&self, node: FixedBytes<32>) -> Result<Vec<(String, String)>> {
        require_valid_input(!self.node_owners.get(node).is_zero(), "Node not found")?;
        
        let mut result = Vec::new();
        let cultural_data = self.node_cultural_data.get(node);
        
        for i in 0..self.cultural_keys.len() {
            if let Some(key) = self.cultural_keys.get(i) {
                let value = cultural_data.get(key.clone());
                if !value.is_empty() {
                    result.push((key, value));
                }
            }
        }
        
        Ok(result)
    }

    pub fn get_platform_data(&self, node: FixedBytes<32>) -> Result<Vec<(String, String)>> {
        require_valid_input(!self.node_owners.get(node).is_zero(), "Node not found")?;
        
        let mut result = Vec::new();
        let platform_data = self.node_platform_data.get(node);
        
        for i in 0..self.platform_keys.len() {
            if let Some(key) = self.platform_keys.get(i) {
                let value = platform_data.get(key.clone());
                if !value.is_empty() {
                    result.push((key, value));
                }
            }
        }
        
        Ok(result)
    }

    pub fn get_cultural_metadata(&self, node: FixedBytes<32>, key: String) -> Result<String> {
        require_valid_input(!self.node_owners.get(node).is_zero(), "Node not found")?;
        self.validate_cultural_key(&key)?;
        
        Ok(self.node_cultural_data.get(node).get(key))
    }

    pub fn get_platform_metadata(&self, node: FixedBytes<32>, key: String) -> Result<String> {
        require_valid_input(!self.node_owners.get(node).is_zero(), "Node not found")?;
        self.validate_platform_key(&key)?;
        
        Ok(self.node_platform_data.get(node).get(key))
    }

    pub fn node_owner(&self, node: FixedBytes<32>) -> Address {
        self.node_owners.get(node)
    }

    pub fn subdomain_exists(&self, subdomain: String) -> bool {
        !self.subdomain_to_node.get(subdomain).is_zero()
    }

    // Admin functions
    pub fn add_authorized_updater(&mut self, updater: Address) -> Result<()> {
        self.require_owner()?;
        self.authorized_updaters.insert(updater, true);
        Ok(())
    }

    pub fn remove_authorized_updater(&mut self, updater: Address) -> Result<()> {
        self.require_owner()?;
        self.authorized_updaters.insert(updater, false);
        Ok(())
    }
}

// Internal helper functions
impl ENSIntegration {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_authorized(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.owner.get() 
            || self.authorized_updaters.get(caller)
            || caller == self.platform_contract.get(),
            "Not authorized"
        )
    }

    fn require_node_owner_or_authorized(&self, node: FixedBytes<32>) -> Result<()> {
        let caller = msg::sender();
        let node_owner = self.node_owners.get(node);
        require_authorized(
            caller == node_owner 
            || caller == self.owner.get() 
            || self.authorized_updaters.get(caller)
            || caller == self.platform_contract.get(),
            "Not authorized for this node"
        )
    }

    fn generate_node_hash(&self, subdomain: &str) -> Result<FixedBytes<32>> {
        // Simplified node hash generation
        // In production, would use proper ENS node calculation
        let mut bytes = [0u8; 32];
        let subdomain_bytes = subdomain.as_bytes();
        let len = core::cmp::min(32, subdomain_bytes.len());
        bytes[..len].copy_from_slice(&subdomain_bytes[..len]);
        Ok(FixedBytes(bytes))
    }

    fn set_cultural_metadata(&mut self, node: FixedBytes<32>, key: String, value: String) -> Result<()> {
        self.node_cultural_data.get_mut(node).insert(key, value);
        Ok(())
    }

    fn set_platform_metadata(&mut self, node: FixedBytes<32>, key: String, value: String) -> Result<()> {
        self.node_platform_data.get_mut(node).insert(key, value);
        Ok(())
    }

    fn initialize_cultural_keys(&mut self) {
        self.cultural_keys.push("cultural.background".to_string());
        self.cultural_keys.push("cultural.languages".to_string());
        self.cultural_keys.push("cultural.region".to_string());
        self.cultural_keys.push("cultural.traditions".to_string());
        self.cultural_keys.push("cultural.expertise".to_string());
        self.cultural_keys.push("cultural.verified_by".to_string());
        self.cultural_keys.push("cultural.verification_date".to_string());
    }

    fn initialize_platform_keys(&mut self) {
        self.platform_keys.push("platform.reputation".to_string());
        self.platform_keys.push("platform.projects".to_string());
        self.platform_keys.push("platform.total_funding".to_string());
        self.platform_keys.push("platform.successful_projects".to_string());
        self.platform_keys.push("platform.join_date".to_string());
        self.platform_keys.push("platform.verification_status".to_string());
        self.platform_keys.push("platform.bio".to_string());
        self.platform_keys.push("platform.social_links".to_string());
    }

    fn is_cultural_key(&self, key: &str) -> bool {
        for i in 0..self.cultural_keys.len() {
            if let Some(cultural_key) = self.cultural_keys.get(i) {
                if cultural_key == key {
                    return true;
                }
            }
        }
        false
    }

    fn is_platform_key(&self, key: &str) -> bool {
        for i in 0..self.platform_keys.len() {
            if let Some(platform_key) = self.platform_keys.get(i) {
                if platform_key == key {
                    return true;
                }
            }
        }
        false
    }

    fn validate_cultural_key(&self, key: &str) -> Result<()> {
        require_valid_input(self.is_cultural_key(key), "Invalid cultural key")?;
        Ok(())
    }

    fn validate_platform_key(&self, key: &str) -> Result<()> {
        require_valid_input(self.is_platform_key(key), "Invalid platform key")?;
        Ok(())
    }
}