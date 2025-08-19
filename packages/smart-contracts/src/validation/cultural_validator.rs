use alloy_primitives::{Address, U256};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageString, StorageU256, StorageVec},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
    ValidatorProfile, ValidationSubmission, ValidationStatus,
    VALIDATION_THRESHOLD, MIN_VALIDATORS_REQUIRED,
};

#[derive(SolidityType, Clone, Debug)]
pub struct ValidationResult {
    pub project_id: U256,
    pub final_score: U256,
    pub status: u8, // ValidationStatus
    pub validator_count: U256,
    pub completed_timestamp: U256,
    pub can_appeal: bool,
}

#[derive(SolidityType, Clone, Debug)]
pub struct Appeal {
    pub appeal_id: U256,
    pub project_id: U256,
    pub challenger: Address,
    pub reason: String,
    pub evidence_uri: String,
    pub status: u8, // 0: Pending, 1: Upheld, 2: Rejected
    pub created_timestamp: U256,
    pub resolution_timestamp: U256,
    pub resolution_notes: String,
}

#[storage]
#[entrypoint]
pub struct CulturalValidator {
    // Validator management
    validators: StorageMap<Address, ValidatorProfile>,
    validator_regions: StorageMap<Address, StorageVec<String>>,
    validator_stakes: StorageMap<Address, U256>,
    validator_count: StorageU256,
    
    // Regional authorities (validators with special permissions for specific regions)
    regional_authorities: StorageMap<String, StorageVec<Address>>,
    regional_authority_count: StorageMap<String, U256>,
    
    // Project validations
    project_validations: StorageMap<U256, ValidationResult>,
    project_submissions: StorageMap<U256, StorageVec<ValidationSubmission>>,
    validator_project_submissions: StorageMap<U256, StorageMap<Address, ValidationSubmission>>,
    
    // Validator performance tracking
    validator_reputation: StorageMap<Address, U256>,
    validator_accuracy_history: StorageMap<Address, StorageVec<U256>>, // Success rates over time
    validation_history: StorageMap<Address, StorageVec<U256>>, // Projects validated
    
    // Appeals system
    appeals: StorageMap<U256, Appeal>,
    project_appeals: StorageMap<U256, StorageVec<U256>>, // project -> appeal_ids
    next_appeal_id: StorageU256,
    
    // Platform integration
    platform_contract: StorageAddress,
    
    // Validation settings
    min_validators_required: StorageU256,
    validation_threshold_score: StorageU256,
    validator_reward_amount: StorageU256,
    stake_requirement: StorageU256,
    appeal_period: StorageU256, // Time window for appeals
    dispute_resolution_period: StorageU256,
    
    // Access control
    owner: StorageAddress,
    admins: StorageMap<Address, bool>,
    
    // Cultural expertise database
    cultural_elements_db: StorageMap<String, StorageVec<String>>, // region -> elements
    traditional_practices: StorageMap<String, StorageVec<String>>, // region -> practices
    language_families: StorageMap<String, StorageVec<String>>, // region -> languages
    
    // Validation metrics
    total_validations_completed: StorageU256,
    total_projects_approved: StorageU256,
    total_projects_rejected: StorageU256,
    average_validation_score: StorageU256,
    
    // Slashing and penalties
    slashing_penalties: StorageMap<Address, U256>, // validator -> penalty amount
    validator_suspension_status: StorageMap<Address, bool>,
    suspension_end_times: StorageMap<Address, U256>,
}

#[public]
impl CulturalValidator {
    pub fn initialize(&mut self, platform_contract: Address) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.platform_contract.set(platform_contract);
        
        // Set default parameters
        self.min_validators_required.set(U256::from(MIN_VALIDATORS_REQUIRED));
        self.validation_threshold_score.set(U256::from(VALIDATION_THRESHOLD));
        self.validator_reward_amount.set(U256::from(10000000000000000u64)); // 0.01 ETH
        self.stake_requirement.set(U256::from(100000000000000000u64)); // 0.1 ETH
        self.appeal_period.set(U256::from(7 * 24 * 3600)); // 7 days
        self.dispute_resolution_period.set(U256::from(14 * 24 * 3600)); // 14 days
        self.next_appeal_id.set(U256::from(1));
        
        // Initialize cultural database
        self.initialize_cultural_database();
        
        Ok(())
    }

    #[payable]
    pub fn register_validator(
        &mut self,
        ens_name: String,
        regions: Vec<String>,
        credentials_uri: String,
    ) -> Result<bool> {
        let validator = msg::sender();
        let stake = msg::value();
        
        require_valid_input(
            stake >= self.stake_requirement.get(),
            "Insufficient stake amount"
        )?;
        require_valid_input(
            self.validators.get(validator).validator_address.is_zero(),
            "Validator already registered"
        )?;
        require_valid_input(!regions.is_empty(), "Must specify at least one region")?;
        
        // Validate regions are supported
        for region in &regions {
            require_valid_input(
                self.is_supported_region(region),
                "Unsupported region"
            )?;
        }
        
        let profile = ValidatorProfile {
            validator_address: validator,
            ens_name: ens_name.clone(),
            expertise_regions: regions.clone(),
            credentials_uri,
            reputation_score: U256::from(100), // Starting reputation
            validations_completed: U256::from(0),
            is_active: true,
            stake_amount: stake,
            registration_timestamp: U256::from(block::timestamp()),
        };
        
        self.validators.insert(validator, profile);
        self.validator_stakes.insert(validator, stake);
        self.validator_reputation.insert(validator, U256::from(100));
        
        // Add to regional expertise
        let mut validator_regions_storage = self.validator_regions.get_mut(validator);
        for region in &regions {
            validator_regions_storage.push(region.clone());
            self.regional_authorities.get_mut(region.clone()).push(validator);
            let count = self.regional_authority_count.get(region.clone());
            self.regional_authority_count.insert(region.clone(), count + U256::from(1));
        }
        
        self.validator_count.set(self.validator_count.get() + U256::from(1));

        evm::log(ValidatorRegistered {
            validator,
            ens_name,
            expertise_regions: regions,
            stake_amount: stake,
        });

        Ok(true)
    }

    pub fn submit_validation(
        &mut self,
        project_id: U256,
        score: U256,
        feedback_uri: String,
        cultural_elements: Vec<String>,
    ) -> Result<()> {
        let validator = msg::sender();
        
        // Verify validator is registered and active
        let validator_profile = self.validators.get(validator);
        require_valid_input(
            !validator_profile.validator_address.is_zero(),
            "Validator not registered"
        )?;
        require_valid_input(validator_profile.is_active, "Validator not active")?;
        require_valid_input(
            !self.validator_suspension_status.get(validator),
            "Validator suspended"
        )?;
        
        // Validate score range
        require_valid_input(score <= U256::from(100), "Score must be 0-100")?;
        
        // Check if validator already submitted for this project
        let existing_submission = self.validator_project_submissions.get(project_id).get(validator);
        require_valid_input(
            existing_submission.validator.is_zero(),
            "Validation already submitted"
        )?;
        
        // Verify validator has expertise in project's cultural region
        // (Would check with platform contract in production)
        self.verify_validator_expertise(validator, project_id)?;
        
        let submission = ValidationSubmission {
            validator,
            score,
            feedback_uri,
            cultural_elements,
            timestamp: U256::from(block::timestamp()),
            is_final: false,
        };
        
        // Store submission
        self.validator_project_submissions.get_mut(project_id).insert(validator, submission.clone());
        self.project_submissions.get_mut(project_id).push(submission);
        
        // Add to validator's history
        self.validation_history.get_mut(validator).push(project_id);
        
        // Check if we have enough validations to finalize
        let submissions = self.project_submissions.get(project_id);
        if submissions.len() >= self.min_validators_required.get().as_usize() {
            self.finalize_validation(project_id)?;
        }

        evm::log(ProjectValidated {
            project_id,
            validator,
            score,
            approved: score >= self.validation_threshold_score.get(),
        });

        Ok(())
    }

    pub fn finalize_validation(&mut self, project_id: U256) -> Result<U256> {
        let submissions = self.project_submissions.get(project_id);
        require_valid_input(
            submissions.len() >= self.min_validators_required.get().as_usize(),
            "Insufficient validator submissions"
        )?;
        
        // Calculate weighted average score
        let mut total_score = U256::from(0);
        let mut total_weight = U256::from(0);
        let mut validator_count = 0;
        
        for i in 0..submissions.len() {
            if let Some(submission) = submissions.get(i) {
                let validator_reputation = self.validator_reputation.get(submission.validator);
                let weight = validator_reputation; // Use reputation as weight
                
                total_score += submission.score * weight;
                total_weight += weight;
                validator_count += 1;
            }
        }
        
        let final_score = if total_weight > U256::from(0) {
            total_score / total_weight
        } else {
            U256::from(0)
        };
        
        // Determine validation status
        let approved = final_score >= self.validation_threshold_score.get();
        let status = if approved { 1u8 } else { 2u8 }; // Approved or Rejected
        
        let result = ValidationResult {
            project_id,
            final_score,
            status,
            validator_count: U256::from(validator_count),
            completed_timestamp: U256::from(block::timestamp()),
            can_appeal: true,
        };
        
        self.project_validations.insert(project_id, result);
        
        // Update metrics
        self.total_validations_completed.set(self.total_validations_completed.get() + U256::from(1));
        if approved {
            self.total_projects_approved.set(self.total_projects_approved.get() + U256::from(1));
        } else {
            self.total_projects_rejected.set(self.total_projects_rejected.get() + U256::from(1));
        }
        
        // Update average score
        self.update_average_validation_score(final_score);
        
        // Reward validators
        self.distribute_validator_rewards(project_id)?;
        
        // Update validator reputations based on consensus
        self.update_validator_reputations(project_id, final_score)?;

        evm::log(ValidationCompleted {
            project_id,
            final_score,
            approved,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(final_score)
    }

    pub fn challenge_validation(&mut self, project_id: U256, reason: String, evidence_uri: String) -> Result<U256> {
        let challenger = msg::sender();
        
        let validation_result = self.project_validations.get(project_id);
        require_valid_input(
            validation_result.project_id != U256::from(0),
            "Project not validated"
        )?;
        require_valid_input(validation_result.can_appeal, "Appeals not allowed")?;
        require_valid_input(
            U256::from(block::timestamp()) <= validation_result.completed_timestamp + self.appeal_period.get(),
            "Appeal period expired"
        )?;
        
        let appeal_id = self.next_appeal_id.get();
        
        let appeal = Appeal {
            appeal_id,
            project_id,
            challenger,
            reason,
            evidence_uri,
            status: 0, // Pending
            created_timestamp: U256::from(block::timestamp()),
            resolution_timestamp: U256::from(0),
            resolution_notes: String::new(),
        };
        
        self.appeals.insert(appeal_id, appeal);
        self.project_appeals.get_mut(project_id).push(appeal_id);
        self.next_appeal_id.set(appeal_id + U256::from(1));
        
        Ok(appeal_id)
    }

    pub fn resolve_appeal(&mut self, appeal_id: U256, upheld: bool, resolution_notes: String) -> Result<()> {
        self.require_admin()?;
        
        let mut appeal = self.appeals.get(appeal_id);
        require_valid_input(appeal.appeal_id != U256::from(0), "Appeal not found")?;
        require_valid_input(appeal.status == 0, "Appeal already resolved")?;
        
        appeal.status = if upheld { 1 } else { 2 }; // Upheld or Rejected
        appeal.resolution_timestamp = U256::from(block::timestamp());
        appeal.resolution_notes = resolution_notes;
        
        self.appeals.insert(appeal_id, appeal.clone());
        
        // If upheld, reverse the validation decision
        if upheld {
            let mut validation_result = self.project_validations.get(appeal.project_id);
            validation_result.status = if validation_result.status == 1 { 2 } else { 1 }; // Flip decision
            self.project_validations.insert(appeal.project_id, validation_result);
            
            // Penalize validators who were wrong
            self.penalize_inaccurate_validators(appeal.project_id)?;
        }
        
        Ok(())
    }

    // View functions
    pub fn get_validation_status(&self, project_id: U256) -> Result<ValidationResult> {
        let result = self.project_validations.get(project_id);
        require_valid_input(result.project_id != U256::from(0), "Project not found")?;
        Ok(result)
    }

    pub fn get_qualified_validators(&self, cultural_region: String) -> Vec<Address> {
        let authorities = self.regional_authorities.get(cultural_region);
        let mut result = Vec::new();
        
        for i in 0..authorities.len() {
            if let Some(validator) = authorities.get(i) {
                let profile = self.validators.get(validator);
                if profile.is_active && !self.validator_suspension_status.get(validator) {
                    result.push(validator);
                }
            }
        }
        
        result
    }

    pub fn get_validator_profile(&self, validator: Address) -> Result<ValidatorProfile> {
        let profile = self.validators.get(validator);
        require_valid_input(!profile.validator_address.is_zero(), "Validator not found")?;
        Ok(profile)
    }

    pub fn get_project_submissions(&self, project_id: U256) -> Vec<ValidationSubmission> {
        let submissions = self.project_submissions.get(project_id);
        let mut result = Vec::new();
        
        for i in 0..submissions.len() {
            if let Some(submission) = submissions.get(i) {
                result.push(submission);
            }
        }
        
        result
    }

    pub fn validator_stats(&self) -> (U256, U256, U256, U256) {
        (
            self.validator_count.get(),
            self.total_validations_completed.get(),
            self.total_projects_approved.get(),
            self.average_validation_score.get(),
        )
    }

    // Admin functions
    pub fn add_admin(&mut self, admin: Address) -> Result<()> {
        self.require_owner()?;
        self.admins.insert(admin, true);
        Ok(())
    }

    pub fn suspend_validator(&mut self, validator: Address, duration_days: U256) -> Result<()> {
        self.require_admin()?;
        
        self.validator_suspension_status.insert(validator, true);
        let end_time = U256::from(block::timestamp()) + (duration_days * U256::from(86400));
        self.suspension_end_times.insert(validator, end_time);
        
        Ok(())
    }

    pub fn slash_validator(&mut self, validator: Address, penalty_amount: U256, reason: String) -> Result<()> {
        self.require_admin()?;
        
        let current_stake = self.validator_stakes.get(validator);
        let penalty = core::cmp::min(penalty_amount.as_u64(), current_stake.as_u64());
        
        self.validator_stakes.insert(validator, current_stake - U256::from(penalty));
        self.slashing_penalties.insert(validator, self.slashing_penalties.get(validator) + U256::from(penalty));

        evm::log(ValidatorSlashed {
            validator,
            amount: U256::from(penalty),
            reason,
        });

        Ok(())
    }
}

// Internal helper functions
impl CulturalValidator {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_admin(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            caller == self.owner.get() || self.admins.get(caller),
            "Only admin"
        )
    }

    fn verify_validator_expertise(&self, validator: Address, project_id: U256) -> Result<()> {
        // In production, would check project's cultural category against validator's regions
        // For now, just verify validator exists and is active
        let profile = self.validators.get(validator);
        require_valid_input(!profile.validator_address.is_zero(), "Validator not found")?;
        require_valid_input(profile.is_active, "Validator not active")?;
        Ok(())
    }

    fn is_supported_region(&self, region: &str) -> bool {
        let elements = self.cultural_elements_db.get(region.to_string());
        elements.len() > 0
    }

    fn distribute_validator_rewards(&self, project_id: U256) -> Result<()> {
        let submissions = self.project_submissions.get(project_id);
        let reward_per_validator = self.validator_reward_amount.get();
        
        for i in 0..submissions.len() {
            if let Some(submission) = submissions.get(i) {
                // In production, would transfer rewards to validators
                // For now, just validate the operation
                if reward_per_validator > U256::from(0) {
                    // call::transfer_eth(submission.validator, reward_per_validator)?;
                }
            }
        }
        
        Ok(())
    }

    fn update_validator_reputations(&mut self, project_id: U256, consensus_score: U256) -> Result<()> {
        let submissions = self.project_submissions.get(project_id);
        
        for i in 0..submissions.len() {
            if let Some(submission) = submissions.get(i) {
                let validator = submission.validator;
                let validator_score = submission.score;
                
                // Calculate accuracy based on deviation from consensus
                let deviation = if consensus_score > validator_score {
                    consensus_score - validator_score
                } else {
                    validator_score - consensus_score
                };
                
                // Adjust reputation based on accuracy
                let current_reputation = self.validator_reputation.get(validator);
                let accuracy = U256::from(100) - core::cmp::min(deviation.as_u64(), 100);
                
                // Simple reputation update: move toward accuracy score
                let new_reputation = (current_reputation * U256::from(9) + accuracy) / U256::from(10);
                self.validator_reputation.insert(validator, new_reputation);
                
                // Update accuracy history
                self.validator_accuracy_history.get_mut(validator).push(accuracy);
            }
        }
        
        Ok(())
    }

    fn penalize_inaccurate_validators(&mut self, project_id: U256) -> Result<()> {
        let submissions = self.project_submissions.get(project_id);
        let penalty_amount = self.validator_reward_amount.get();
        
        for i in 0..submissions.len() {
            if let Some(submission) = submissions.get(i) {
                let validator = submission.validator;
                let current_stake = self.validator_stakes.get(validator);
                
                if current_stake >= penalty_amount {
                    self.validator_stakes.insert(validator, current_stake - penalty_amount);
                    self.slashing_penalties.insert(validator, 
                        self.slashing_penalties.get(validator) + penalty_amount);
                }
            }
        }
        
        Ok(())
    }

    fn update_average_validation_score(&mut self, new_score: U256) {
        let total_validations = self.total_validations_completed.get();
        let current_average = self.average_validation_score.get();
        
        if total_validations > U256::from(0) {
            let new_average = (current_average * total_validations + new_score) / (total_validations + U256::from(1));
            self.average_validation_score.set(new_average);
        } else {
            self.average_validation_score.set(new_score);
        }
    }

    fn initialize_cultural_database(&mut self) {
        let regions = vec![
            ("West Africa", vec!["Griot Storytelling", "Kente Weaving", "Djembe Music", "Yoruba Art"]),
            ("East Africa", vec!["Maasai Beadwork", "Ethiopian Coffee Culture", "Swahili Poetry", "Traditional Dance"]),
            ("Southern Africa", vec!["Ubuntu Philosophy", "Zulu Crafts", "Traditional Healing", "Praise Poetry"]),
            ("Central Africa", vec!["Pygmy Music", "Wood Carving", "Oral Traditions", "Ritual Masks"]),
            ("North Africa", vec!["Berber Textiles", "Islamic Calligraphy", "Desert Music", "Traditional Architecture"]),
        ];
        
        for (region, elements) in regions {
            let mut storage = self.cultural_elements_db.get_mut(region.to_string());
            for element in elements {
                storage.push(element.to_string());
            }
        }
    }
}