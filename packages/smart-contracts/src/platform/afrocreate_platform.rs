use alloy_primitives::{Address, U256, FixedBytes};
use stylus_sdk::{
    block, call, contract, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
    interfaces::{ENSRegistry, IProjectFunding, ICulturalValidator},
    CreatorProfile, ProjectInfo, PLATFORM_FEE_BPS, AFROCREATE_ENS_NODE,
};

#[storage]
#[entrypoint]
pub struct AfroCreatePlatform {
    // Core registries
    creators: StorageMap<Address, CreatorProfile>,
    projects: StorageMap<U256, ProjectInfo>,
    creator_count: StorageU256,
    project_count: StorageU256,
    
    // ENS integration
    ens_registry: StorageAddress,
    afrocreate_node: StorageString, // Store as string for now
    subdomain_registry: StorageMap<String, Address>,
    creator_ens_names: StorageMap<Address, String>,
    
    // Platform settings
    platform_fee_bps: StorageU256,
    min_project_funding: StorageU256,
    max_project_duration: StorageU256,
    
    // Contract addresses
    project_funding: StorageAddress,
    revenue_distributor: StorageAddress,
    cultural_validator: StorageAddress,
    governance: StorageAddress,
    
    // Security and access control
    paused: StorageBool,
    owner: StorageAddress,
    admins: StorageMap<Address, bool>,
    
    // Metrics
    total_funding_raised: StorageU256,
    successful_projects: StorageU256,
    active_creators: StorageU256,
    
    // Creator to project mapping
    creator_projects: StorageMap<Address, StorageVec<U256>>,
    
    // Cultural categories
    approved_categories: StorageVec<String>,
    category_projects: StorageMap<String, StorageVec<U256>>,
}

#[public]
impl AfroCreatePlatform {
    pub fn initialize(
        &mut self,
        ens_registry: Address,
        min_funding: U256,
        max_duration: U256,
    ) -> Result<()> {
        require_valid_input(!self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.ens_registry.set(ens_registry);
        self.platform_fee_bps.set(U256::from(PLATFORM_FEE_BPS));
        self.min_project_funding.set(min_funding);
        self.max_project_duration.set(max_duration);
        
        // Initialize approved cultural categories
        self.approved_categories.push("Music".to_string());
        self.approved_categories.push("Visual Arts".to_string());
        self.approved_categories.push("Film & Video".to_string());
        self.approved_categories.push("Literature".to_string());
        self.approved_categories.push("Traditional Crafts".to_string());
        self.approved_categories.push("Dance & Performance".to_string());
        self.approved_categories.push("Digital Media".to_string());
        self.approved_categories.push("Fashion & Design".to_string());
        
        Ok(())
    }

    pub fn register_creator(
        &mut self,
        ens_subdomain: String,
        cultural_background: String,
    ) -> Result<U256> {
        self.require_not_paused()?;
        require_valid_input(self.validate_ens_name(&ens_subdomain)?, "Invalid ENS subdomain")?;
        
        let creator = msg::sender();
        require_valid_input(
            self.creators.get(creator).creator_address.is_zero(),
            "Creator already registered"
        )?;
        
        // Validate ENS ownership (simplified for demo)
        require_valid_input(
            self.validate_ens_ownership(&ens_subdomain, creator)?,
            "ENS ownership validation failed"
        )?;

        let creator_id = self.creator_count.get() + U256::from(1);
        
        let profile = CreatorProfile {
            creator_address: creator,
            ens_name: ens_subdomain.clone(),
            cultural_background: cultural_background.clone(),
            reputation_score: U256::from(100), // Starting reputation
            projects_created: U256::from(0),
            total_funding_raised: U256::from(0),
            is_verified: false,
            registration_timestamp: U256::from(block::timestamp()),
        };

        self.creators.insert(creator, profile);
        self.subdomain_registry.insert(ens_subdomain.clone(), creator);
        self.creator_ens_names.insert(creator, ens_subdomain.clone());
        self.creator_count.set(creator_id);
        self.active_creators.set(self.active_creators.get() + U256::from(1));

        evm::log(CreatorRegistered {
            creator,
            ens_name: ens_subdomain,
            cultural_background,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(creator_id)
    }

    pub fn create_project(
        &mut self,
        title: String,
        description: String,
        cultural_category: String,
        funding_target: U256,
        duration_days: U256,
        metadata_uri: String,
    ) -> Result<U256> {
        self.require_not_paused()?;
        let creator = msg::sender();
        
        // Verify creator is registered
        let mut creator_profile = self.creators.get(creator);
        require_valid_input(
            !creator_profile.creator_address.is_zero(),
            "Creator not registered"
        )?;
        
        // Validate inputs
        require_valid_input(
            funding_target >= self.min_project_funding.get(),
            "Funding target too low"
        )?;
        require_valid_input(
            duration_days <= self.max_project_duration.get(),
            "Project duration too long"
        )?;
        require_valid_input(
            self.is_approved_category(&cultural_category),
            "Cultural category not approved"
        )?;

        let project_id = self.project_count.get() + U256::from(1);
        let deadline = U256::from(block::timestamp()) + (duration_days * U256::from(86400));

        let project = ProjectInfo {
            project_id,
            creator,
            title: title.clone(),
            description,
            cultural_category: cultural_category.clone(),
            funding_target,
            funding_raised: U256::from(0),
            deadline,
            status: 0, // Active
            validation_status: 0, // Pending
            validation_score: U256::from(0),
            metadata_uri,
        };

        self.projects.insert(project_id, project);
        self.project_count.set(project_id);
        
        // Update creator profile
        creator_profile.projects_created += U256::from(1);
        self.creators.insert(creator, creator_profile);
        
        // Add to creator's project list
        self.creator_projects.get_mut(creator).push(project_id);
        
        // Add to category mapping
        self.category_projects.get_mut(cultural_category.clone()).push(project_id);

        evm::log(ProjectCreated {
            project_id,
            creator,
            title,
            cultural_category,
            funding_target,
            deadline,
        });

        Ok(project_id)
    }

    pub fn validate_ens_ownership(&self, subdomain: &str, claimer: Address) -> Result<bool> {
        // Simplified validation - in production, would call ENS registry
        Ok(!subdomain.is_empty() && subdomain.len() >= 3 && !claimer.is_zero())
    }

    pub fn get_creator_profile(&self, creator: Address) -> Result<CreatorProfile> {
        let profile = self.creators.get(creator);
        require_valid_input(
            !profile.creator_address.is_zero(),
            "Creator not found"
        )?;
        Ok(profile)
    }

    pub fn get_project_info(&self, project_id: U256) -> Result<ProjectInfo> {
        let project = self.projects.get(project_id);
        require_valid_input(
            project.project_id != U256::from(0),
            "Project not found"
        )?;
        Ok(project)
    }

    pub fn get_creator_projects(&self, creator: Address) -> Result<Vec<U256>> {
        let projects = self.creator_projects.get(creator);
        let mut result = Vec::new();
        for i in 0..projects.len() {
            if let Some(project_id) = projects.get(i) {
                result.push(project_id);
            }
        }
        Ok(result)
    }

    pub fn get_category_projects(&self, category: String) -> Result<Vec<U256>> {
        let projects = self.category_projects.get(category);
        let mut result = Vec::new();
        for i in 0..projects.len() {
            if let Some(project_id) = projects.get(i) {
                result.push(project_id);
            }
        }
        Ok(result)
    }

    pub fn update_project_funding(&mut self, project_id: U256, amount_raised: U256) -> Result<()> {
        self.require_authorized()?;
        
        let mut project = self.projects.get(project_id);
        require_valid_input(
            project.project_id != U256::from(0),
            "Project not found"
        )?;

        project.funding_raised = amount_raised;
        
        // Check if funding target is reached
        if amount_raised >= project.funding_target {
            project.status = 1; // Successful
            self.successful_projects.set(self.successful_projects.get() + U256::from(1));
            
            // Update creator's total funding raised
            let mut creator_profile = self.creators.get(project.creator);
            creator_profile.total_funding_raised += amount_raised;
            self.creators.insert(project.creator, creator_profile);
        }
        
        self.projects.insert(project_id, project);
        self.total_funding_raised.set(self.total_funding_raised.get() + amount_raised);
        
        Ok(())
    }

    pub fn set_project_validation(&mut self, project_id: U256, score: U256, approved: bool) -> Result<()> {
        self.require_authorized()?;
        
        let mut project = self.projects.get(project_id);
        require_valid_input(
            project.project_id != U256::from(0),
            "Project not found"
        )?;

        project.validation_score = score;
        project.validation_status = if approved { 1 } else { 2 }; // Approved/Rejected
        
        self.projects.insert(project_id, project);

        evm::log(ValidationCompleted {
            project_id,
            final_score: score,
            approved,
            timestamp: U256::from(block::timestamp()),
        });
        
        Ok(())
    }

    // Administrative functions
    pub fn set_platform_fee(&mut self, new_fee_bps: U256) -> Result<()> {
        self.require_owner()?;
        require_valid_input(new_fee_bps <= U256::from(1000), "Fee too high"); // Max 10%
        
        let old_fee = self.platform_fee_bps.get();
        self.platform_fee_bps.set(new_fee_bps);
        
        evm::log(PlatformFeeUpdated {
            old_fee_bps: old_fee,
            new_fee_bps,
        });
        
        Ok(())
    }

    pub fn add_admin(&mut self, admin: Address) -> Result<()> {
        self.require_owner()?;
        self.admins.insert(admin, true);
        Ok(())
    }

    pub fn remove_admin(&mut self, admin: Address) -> Result<()> {
        self.require_owner()?;
        self.admins.insert(admin, false);
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        self.require_owner()?;
        self.paused.set(true);
        
        evm::log(PlatformPaused {
            timestamp: U256::from(block::timestamp()),
        });
        
        Ok(())
    }

    pub fn unpause(&mut self) -> Result<()> {
        self.require_owner()?;
        self.paused.set(false);
        
        evm::log(PlatformUnpaused {
            timestamp: U256::from(block::timestamp()),
        });
        
        Ok(())
    }

    // View functions
    pub fn is_paused(&self) -> bool {
        self.paused.get()
    }

    pub fn owner(&self) -> Address {
        self.owner.get()
    }

    pub fn platform_fee_bps(&self) -> U256 {
        self.platform_fee_bps.get()
    }

    pub fn total_creators(&self) -> U256 {
        self.creator_count.get()
    }

    pub fn total_projects(&self) -> U256 {
        self.project_count.get()
    }

    pub fn platform_stats(&self) -> (U256, U256, U256, U256) {
        (
            self.total_funding_raised.get(),
            self.successful_projects.get(),
            self.active_creators.get(),
            self.project_count.get(),
        )
    }
}

// Internal helper functions
impl AfroCreatePlatform {
    fn require_not_paused(&self) -> Result<()> {
        require_valid_input(!self.paused.get(), "Contract is paused")
    }

    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_authorized(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.owner.get() || self.admins.get(caller),
            "Not authorized"
        )
    }

    fn validate_ens_name(&self, name: &str) -> Result<bool> {
        require_valid_input(name.len() >= 3, "ENS name too short")?;
        require_valid_input(name.len() <= 63, "ENS name too long")?;
        require_valid_input(
            name.chars().all(|c| c.is_alphanumeric() || c == '-'),
            "Invalid ENS name characters"
        )?;
        require_valid_input(!name.starts_with('-'), "ENS name cannot start with dash")?;
        require_valid_input(!name.ends_with('-'), "ENS name cannot end with dash")?;
        Ok(true)
    }

    fn is_approved_category(&self, category: &str) -> bool {
        for i in 0..self.approved_categories.len() {
            if let Some(approved_category) = self.approved_categories.get(i) {
                if approved_category == category {
                    return true;
                }
            }
        }
        false
    }
}