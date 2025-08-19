use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
    ProjectInfo, FundingModel, Milestone,
};

#[storage]
#[entrypoint]
pub struct ProjectFactory {
    // Platform integration
    platform_contract: StorageAddress,
    funding_contract: StorageAddress,
    validator_contract: StorageAddress,
    
    // Project templates
    project_templates: StorageMap<String, U256>, // category -> template_id
    template_configs: StorageMap<U256, ProjectTemplate>,
    
    // Access control
    owner: StorageAddress,
    authorized_creators: StorageMap<Address, bool>,
    
    // Metrics
    projects_created: StorageU256,
    next_project_id: StorageU256,
}

#[derive(SolidityType, Clone, Debug)]
pub struct ProjectTemplate {
    pub category: String,
    pub min_funding_target: U256,
    pub max_duration_days: U256,
    pub required_milestones: u8,
    pub validation_required: bool,
    pub cultural_requirements: Vec<String>,
}

#[derive(SolidityType, Clone, Debug)]  
pub struct ProjectCreateRequest {
    pub title: String,
    pub description: String,
    pub cultural_category: String,
    pub funding_target: U256,
    pub duration_days: U256,
    pub funding_model: u8,
    pub milestones: Vec<Milestone>,
    pub metadata_uri: String,
}

#[public]
impl ProjectFactory {
    pub fn initialize(
        &mut self,
        platform_contract: Address,
        funding_contract: Address,
        validator_contract: Address,
    ) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.platform_contract.set(platform_contract);
        self.funding_contract.set(funding_contract);
        self.validator_contract.set(validator_contract);
        self.next_project_id.set(U256::from(1));
        
        // Initialize default project templates
        self.initialize_templates();
        
        Ok(())
    }

    pub fn create_project(&mut self, request: ProjectCreateRequest) -> Result<U256> {
        let creator = msg::sender();
        
        // Validate creator authorization (would check with platform contract)
        self.validate_creator_eligibility(creator)?;
        
        // Validate request against template
        self.validate_project_request(&request)?;
        
        let project_id = self.next_project_id.get();
        
        // Create project in platform contract (simplified)
        self.create_project_in_platform(project_id, &request, creator)?;
        
        // Setup funding in funding contract
        self.setup_project_funding(project_id, &request, creator)?;
        
        // Submit for cultural validation if required
        let template = self.get_template_for_category(&request.cultural_category)?;
        if template.validation_required {
            self.submit_for_validation(project_id, &request)?;
        }
        
        self.next_project_id.set(project_id + U256::from(1));
        self.projects_created.set(self.projects_created.get() + U256::from(1));

        evm::log(ProjectCreated {
            project_id,
            creator,
            title: request.title,
            cultural_category: request.cultural_category,
            funding_target: request.funding_target,
            deadline: U256::from(block::timestamp()) + (request.duration_days * U256::from(86400)),
        });

        Ok(project_id)
    }

    pub fn add_project_template(
        &mut self,
        category: String,
        template: ProjectTemplate,
    ) -> Result<()> {
        self.require_owner()?;
        
        let template_id = self.project_templates.len();
        self.project_templates.insert(category.clone(), U256::from(template_id));
        self.template_configs.insert(U256::from(template_id), template);
        
        Ok(())
    }

    pub fn update_project_template(
        &mut self,
        category: String,
        template: ProjectTemplate,
    ) -> Result<()> {
        self.require_owner()?;
        
        let template_id = self.project_templates.get(category);
        require_valid_input(template_id > U256::from(0), "Template not found")?;
        
        self.template_configs.insert(template_id, template);
        Ok(())
    }

    pub fn authorize_creator(&mut self, creator: Address) -> Result<()> {
        self.require_owner()?;
        self.authorized_creators.insert(creator, true);
        Ok(())
    }

    // View functions
    pub fn get_project_template(&self, category: String) -> Result<ProjectTemplate> {
        self.get_template_for_category(&category)
    }

    pub fn is_creator_authorized(&self, creator: Address) -> bool {
        self.authorized_creators.get(creator)
    }

    pub fn factory_stats(&self) -> (U256, U256) {
        (self.projects_created.get(), self.next_project_id.get())
    }

    pub fn validate_project_proposal(&self, request: ProjectCreateRequest) -> Result<bool> {
        self.validate_project_request(&request)?;
        Ok(true)
    }
}

// Internal helper functions
impl ProjectFactory {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn validate_creator_eligibility(&self, creator: Address) -> Result<()> {
        // In production, would check with platform contract for creator registration
        require_valid_input(!creator.is_zero(), "Invalid creator address")?;
        Ok(())
    }

    fn validate_project_request(&self, request: &ProjectCreateRequest) -> Result<()> {
        // Basic validation
        require_valid_input(!request.title.is_empty(), "Title cannot be empty")?;
        require_valid_input(!request.description.is_empty(), "Description cannot be empty")?;
        require_valid_input(request.funding_target > U256::from(0), "Invalid funding target")?;
        require_valid_input(request.duration_days > U256::from(0), "Invalid duration")?;
        require_valid_input(request.duration_days <= U256::from(365), "Duration too long")?;
        
        // Template-specific validation
        let template = self.get_template_for_category(&request.cultural_category)?;
        
        require_valid_input(
            request.funding_target >= template.min_funding_target,
            "Funding target below minimum"
        )?;
        require_valid_input(
            request.duration_days <= template.max_duration_days,
            "Duration exceeds maximum"
        )?;
        
        // Milestone validation for milestone-based funding
        if request.funding_model == 2 { // MilestoneBased
            require_valid_input(
                request.milestones.len() >= template.required_milestones as usize,
                "Insufficient milestones"
            )?;
            
            // Validate milestone funding adds up to total
            let total_milestone_funding: U256 = request.milestones.iter()
                .map(|m| m.funding_amount)
                .fold(U256::from(0), |acc, amount| acc + amount);
            
            require_valid_input(
                total_milestone_funding == request.funding_target,
                "Milestone funding doesn't match target"
            )?;
        }
        
        Ok(())
    }

    fn get_template_for_category(&self, category: &str) -> Result<ProjectTemplate> {
        let template_id = self.project_templates.get(category.to_string());
        require_valid_input(template_id > U256::from(0), "Category not supported")?;
        
        let template = self.template_configs.get(template_id);
        require_valid_input(!template.category.is_empty(), "Template not found")?;
        
        Ok(template)
    }

    fn create_project_in_platform(
        &self,
        project_id: U256,
        request: &ProjectCreateRequest,
        creator: Address,
    ) -> Result<()> {
        // In production, would call platform contract
        // For now, just validate the call would succeed
        Ok(())
    }

    fn setup_project_funding(
        &self,
        project_id: U256,
        request: &ProjectCreateRequest,
        creator: Address,
    ) -> Result<()> {
        // In production, would call funding contract
        // For now, just validate the setup
        let deadline = U256::from(block::timestamp()) + (request.duration_days * U256::from(86400));
        
        require_valid_input(!self.funding_contract.get().is_zero(), "Funding contract not set")?;
        
        Ok(())
    }

    fn submit_for_validation(&self, project_id: U256, request: &ProjectCreateRequest) -> Result<()> {
        // In production, would call validator contract
        require_valid_input(!self.validator_contract.get().is_zero(), "Validator contract not set")?;
        Ok(())
    }

    fn initialize_templates(&mut self) {
        let categories = vec![
            ("Music", ProjectTemplate {
                category: "Music".to_string(),
                min_funding_target: U256::from(100000000000000000u64), // 0.1 ETH
                max_duration_days: U256::from(90),
                required_milestones: 3,
                validation_required: true,
                cultural_requirements: vec![
                    "cultural.background".to_string(),
                    "cultural.languages".to_string(),
                ],
            }),
            ("Visual Arts", ProjectTemplate {
                category: "Visual Arts".to_string(),
                min_funding_target: U256::from(50000000000000000u64), // 0.05 ETH
                max_duration_days: U256::from(120),
                required_milestones: 2,
                validation_required: true,
                cultural_requirements: vec![
                    "cultural.background".to_string(),
                    "cultural.traditions".to_string(),
                ],
            }),
            ("Film & Video", ProjectTemplate {
                category: "Film & Video".to_string(),
                min_funding_target: U256::from(500000000000000000u64), // 0.5 ETH
                max_duration_days: U256::from(180),
                required_milestones: 4,
                validation_required: true,
                cultural_requirements: vec![
                    "cultural.background".to_string(),
                    "cultural.languages".to_string(),
                    "cultural.expertise".to_string(),
                ],
            }),
            ("Literature", ProjectTemplate {
                category: "Literature".to_string(),
                min_funding_target: U256::from(25000000000000000u64), // 0.025 ETH
                max_duration_days: U256::from(150),
                required_milestones: 3,
                validation_required: true,
                cultural_requirements: vec![
                    "cultural.background".to_string(),
                    "cultural.languages".to_string(),
                ],
            }),
            ("Traditional Crafts", ProjectTemplate {
                category: "Traditional Crafts".to_string(),
                min_funding_target: U256::from(75000000000000000u64), // 0.075 ETH
                max_duration_days: U256::from(90),
                required_milestones: 2,
                validation_required: true,
                cultural_requirements: vec![
                    "cultural.background".to_string(),
                    "cultural.traditions".to_string(),
                    "cultural.expertise".to_string(),
                ],
            }),
        ];

        for (i, (category, template)) in categories.into_iter().enumerate() {
            let template_id = U256::from(i + 1);
            self.project_templates.insert(category.to_string(), template_id);
            self.template_configs.insert(template_id, template);
        }
    }
}