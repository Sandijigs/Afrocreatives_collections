use alloy_primitives::{Address, U256, FixedBytes};
use stylus_sdk::{
    block, evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageString, StorageU256, StorageVec, StorageBool},
};

use crate::types::{
    errors::{AfroCreateError, Result, require_authorized, require_valid_input},
    events::*,
};

#[derive(SolidityType, Clone, Debug)]
pub struct CulturalProfile {
    pub regions: Vec<String>,
    pub languages: Vec<String>,
    pub traditions: Vec<String>,
    pub expertise_areas: Vec<String>,
    pub verification_status: u8,
    pub verification_date: U256,
    pub verified_by: Address,
}

#[derive(SolidityType, Clone, Debug)]
pub struct CulturalCredential {
    pub credential_type: String,
    pub issuer: Address,
    pub metadata_uri: String,
    pub issued_date: U256,
    pub expiry_date: U256,
    pub is_active: bool,
}

#[storage]
#[entrypoint]
pub struct CulturalIdentity {
    // Cultural profiles
    cultural_profiles: StorageMap<Address, CulturalProfile>,
    
    // Credentials
    user_credentials: StorageMap<Address, StorageVec<CulturalCredential>>,
    
    // Verification system
    cultural_verifiers: StorageMap<Address, bool>,
    regional_authorities: StorageMap<String, StorageVec<Address>>, // region -> verifiers
    
    // Supported regions and cultures
    supported_regions: StorageVec<String>,
    region_languages: StorageMap<String, StorageVec<String>>,
    region_traditions: StorageMap<String, StorageVec<String>>,
    
    // Identity verification requirements
    min_verifiers_required: StorageU256,
    verification_period: StorageU256, // Duration credentials remain valid
    
    // Access control
    owner: StorageAddress,
    platform_contract: StorageAddress,
    
    // Metrics
    total_verified_users: StorageU256,
    verifications_completed: StorageU256,
}

#[public]
impl CulturalIdentity {
    pub fn initialize(&mut self, platform_contract: Address) -> Result<()> {
        require_valid_input(self.owner.get().is_zero(), "Already initialized")?;
        
        let caller = msg::sender();
        self.owner.set(caller);
        self.platform_contract.set(platform_contract);
        self.min_verifiers_required.set(U256::from(2));
        self.verification_period.set(U256::from(365 * 24 * 3600)); // 1 year in seconds
        
        // Initialize supported regions
        self.initialize_supported_regions();
        
        Ok(())
    }

    pub fn create_cultural_profile(
        &mut self,
        regions: Vec<String>,
        languages: Vec<String>,
        traditions: Vec<String>,
        expertise_areas: Vec<String>,
    ) -> Result<()> {
        let user = msg::sender();
        
        // Validate that user doesn't already have a profile
        require_valid_input(
            self.cultural_profiles.get(user).regions.is_empty(),
            "Cultural profile already exists"
        )?;
        
        // Validate regions are supported
        for region in &regions {
            require_valid_input(
                self.is_supported_region(region),
                "Unsupported region"
            )?;
        }
        
        let profile = CulturalProfile {
            regions,
            languages,
            traditions,
            expertise_areas,
            verification_status: 0, // Pending
            verification_date: U256::from(0),
            verified_by: Address::ZERO,
        };
        
        self.cultural_profiles.insert(user, profile);
        
        Ok(())
    }

    pub fn submit_cultural_verification(
        &mut self,
        user: Address,
        credential_type: String,
        metadata_uri: String,
        expiry_months: U256,
    ) -> Result<()> {
        self.require_verifier()?;
        
        let verifier = msg::sender();
        let current_time = U256::from(block::timestamp());
        let expiry_date = current_time + (expiry_months * U256::from(30 * 24 * 3600)); // Approximate months to seconds
        
        let credential = CulturalCredential {
            credential_type,
            issuer: verifier,
            metadata_uri,
            issued_date: current_time,
            expiry_date,
            is_active: true,
        };
        
        self.user_credentials.get_mut(user).push(credential);
        
        // Check if user now meets verification requirements
        self.check_and_update_verification_status(user)?;
        
        Ok(())
    }

    pub fn verify_cultural_authenticity(
        &mut self,
        user: Address,
        region: String,
    ) -> Result<()> {
        self.require_regional_authority(&region)?;
        
        let mut profile = self.cultural_profiles.get(user);
        require_valid_input(!profile.regions.is_empty(), "User has no cultural profile")?;
        
        // Verify the user has claimed expertise in this region
        require_valid_input(
            profile.regions.contains(&region),
            "User has not claimed expertise in this region"
        )?;
        
        let verifier = msg::sender();
        profile.verification_status = 1; // Verified
        profile.verification_date = U256::from(block::timestamp());
        profile.verified_by = verifier;
        
        self.cultural_profiles.insert(user, profile);
        self.total_verified_users.set(self.total_verified_users.get() + U256::from(1));
        self.verifications_completed.set(self.verifications_completed.get() + U256::from(1));
        
        Ok(())
    }

    pub fn add_cultural_verifier(&mut self, verifier: Address, regions: Vec<String>) -> Result<()> {
        self.require_owner()?;
        
        self.cultural_verifiers.insert(verifier, true);
        
        // Add verifier to regional authorities
        for region in regions {
            if self.is_supported_region(&region) {
                self.regional_authorities.get_mut(region).push(verifier);
            }
        }
        
        Ok(())
    }

    pub fn revoke_credential(&mut self, user: Address, credential_index: U256) -> Result<()> {
        let caller = msg::sender();
        let credentials = self.user_credentials.get_mut(user);
        
        require_valid_input(
            credential_index.as_usize() < credentials.len(),
            "Invalid credential index"
        )?;
        
        if let Some(mut credential) = credentials.get(credential_index.as_usize()) {
            // Only the issuer or owner can revoke
            require_authorized(
                caller == credential.issuer || caller == self.owner.get(),
                "Not authorized to revoke credential"
            )?;
            
            credential.is_active = false;
            credentials.set(credential_index.as_usize(), credential);
            
            // Re-check verification status
            self.check_and_update_verification_status(user)?;
        }
        
        Ok(())
    }

    // View functions
    pub fn get_cultural_profile(&self, user: Address) -> Result<CulturalProfile> {
        let profile = self.cultural_profiles.get(user);
        require_valid_input(!profile.regions.is_empty(), "User has no cultural profile")?;
        Ok(profile)
    }

    pub fn get_user_credentials(&self, user: Address) -> Result<Vec<CulturalCredential>> {
        let credentials = self.user_credentials.get(user);
        let mut result = Vec::new();
        
        for i in 0..credentials.len() {
            if let Some(credential) = credentials.get(i) {
                result.push(credential);
            }
        }
        
        Ok(result)
    }

    pub fn is_culturally_verified(&self, user: Address) -> bool {
        let profile = self.cultural_profiles.get(user);
        profile.verification_status == 1
    }

    pub fn get_regional_expertise(&self, region: String) -> Result<Vec<Address>> {
        require_valid_input(self.is_supported_region(&region), "Unsupported region")?;
        
        let mut experts = Vec::new();
        // This would require iteration over all profiles in a real implementation
        // For now, returning empty vector
        Ok(experts)
    }

    pub fn get_supported_regions(&self) -> Vec<String> {
        let mut regions = Vec::new();
        for i in 0..self.supported_regions.len() {
            if let Some(region) = self.supported_regions.get(i) {
                regions.push(region);
            }
        }
        regions
    }

    pub fn get_region_languages(&self, region: String) -> Vec<String> {
        let languages = self.region_languages.get(region);
        let mut result = Vec::new();
        for i in 0..languages.len() {
            if let Some(language) = languages.get(i) {
                result.push(language);
            }
        }
        result
    }

    pub fn get_region_traditions(&self, region: String) -> Vec<String> {
        let traditions = self.region_traditions.get(region);
        let mut result = Vec::new();
        for i in 0..traditions.len() {
            if let Some(tradition) = traditions.get(i) {
                result.push(tradition);
            }
        }
        result
    }

    pub fn verification_stats(&self) -> (U256, U256) {
        (self.total_verified_users.get(), self.verifications_completed.get())
    }
}

// Internal helper functions
impl CulturalIdentity {
    fn require_owner(&self) -> Result<()> {
        require_authorized(msg::sender() == self.owner.get(), "Only owner")
    }

    fn require_verifier(&self) -> Result<()> {
        let caller = msg::sender();
        require_authorized(
            self.cultural_verifiers.get(caller) || caller == self.owner.get(),
            "Not authorized verifier"
        )
    }

    fn require_regional_authority(&self, region: &str) -> Result<()> {
        let caller = msg::sender();
        if caller == self.owner.get() {
            return Ok(());
        }
        
        let authorities = self.regional_authorities.get(region.to_string());
        for i in 0..authorities.len() {
            if let Some(authority) = authorities.get(i) {
                if authority == caller {
                    return Ok(());
                }
            }
        }
        
        Err(AfroCreateError::Unauthorized("Not regional authority".to_string()))
    }

    fn is_supported_region(&self, region: &str) -> bool {
        for i in 0..self.supported_regions.len() {
            if let Some(supported_region) = self.supported_regions.get(i) {
                if supported_region == region {
                    return true;
                }
            }
        }
        false
    }

    fn check_and_update_verification_status(&mut self, user: Address) -> Result<()> {
        let credentials = self.user_credentials.get(user);
        let mut active_credentials = 0;
        let current_time = U256::from(block::timestamp());
        
        for i in 0..credentials.len() {
            if let Some(credential) = credentials.get(i) {
                if credential.is_active && credential.expiry_date > current_time {
                    active_credentials += 1;
                }
            }
        }
        
        let mut profile = self.cultural_profiles.get(user);
        if active_credentials >= self.min_verifiers_required.get().as_usize() {
            if profile.verification_status == 0 {
                profile.verification_status = 1; // Verified
                profile.verification_date = current_time;
                self.total_verified_users.set(self.total_verified_users.get() + U256::from(1));
            }
        } else {
            if profile.verification_status == 1 {
                profile.verification_status = 2; // Verification expired/revoked
                if self.total_verified_users.get() > U256::from(0) {
                    self.total_verified_users.set(self.total_verified_users.get() - U256::from(1));
                }
            }
        }
        
        self.cultural_profiles.insert(user, profile);
        Ok(())
    }

    fn initialize_supported_regions(&mut self) {
        let regions = vec![
            "West Africa", "East Africa", "Central Africa", "Southern Africa", "North Africa",
            "Nigeria", "Ghana", "Kenya", "South Africa", "Ethiopia", "Egypt", "Morocco",
            "Senegal", "Uganda", "Tanzania", "Zimbabwe", "Cameroon", "Ivory Coast"
        ];
        
        for region in regions {
            self.supported_regions.push(region.to_string());
        }
        
        // Initialize languages for major regions
        self.initialize_region_languages();
        self.initialize_region_traditions();
    }

    fn initialize_region_languages(&mut self) {
        // West Africa
        let mut west_africa_languages = self.region_languages.get_mut("West Africa".to_string());
        west_africa_languages.push("Yoruba".to_string());
        west_africa_languages.push("Igbo".to_string());
        west_africa_languages.push("Hausa".to_string());
        west_africa_languages.push("Wolof".to_string());
        west_africa_languages.push("Fula".to_string());
        west_africa_languages.push("Akan/Twi".to_string());
        
        // East Africa
        let mut east_africa_languages = self.region_languages.get_mut("East Africa".to_string());
        east_africa_languages.push("Swahili".to_string());
        east_africa_languages.push("Amharic".to_string());
        east_africa_languages.push("Oromo".to_string());
        east_africa_languages.push("Kikuyu".to_string());
        east_africa_languages.push("Luo".to_string());
        
        // Southern Africa
        let mut southern_africa_languages = self.region_languages.get_mut("Southern Africa".to_string());
        southern_africa_languages.push("Zulu".to_string());
        southern_africa_languages.push("Xhosa".to_string());
        southern_africa_languages.push("Afrikaans".to_string());
        southern_africa_languages.push("Shona".to_string());
        southern_africa_languages.push("Sesotho".to_string());
    }

    fn initialize_region_traditions(&mut self) {
        // West Africa
        let mut west_africa_traditions = self.region_traditions.get_mut("West Africa".to_string());
        west_africa_traditions.push("Griot Storytelling".to_string());
        west_africa_traditions.push("Kente Weaving".to_string());
        west_africa_traditions.push("Djembe Music".to_string());
        west_africa_traditions.push("Masquerade Dancing".to_string());
        
        // East Africa
        let mut east_africa_traditions = self.region_traditions.get_mut("East Africa".to_string());
        east_africa_traditions.push("Maasai Beadwork".to_string());
        east_africa_traditions.push("Ethiopian Coffee Ceremony".to_string());
        east_africa_traditions.push("Swahili Poetry".to_string());
        east_africa_traditions.push("Tribal Dancing".to_string());
    }
}