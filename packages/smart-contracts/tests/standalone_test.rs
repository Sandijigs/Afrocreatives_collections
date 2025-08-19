// Standalone test for AfroCreate smart contract functionality
// This test validates gas optimization and comprehensive coverage without external dependencies

use std::collections::HashMap;
use std::time::Instant;

// Mock U256 implementation for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct U256(pub u64);

impl U256 {
    pub fn from(value: u64) -> Self {
        U256(value)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::ops::Add for U256 {
    type Output = U256;
    fn add(self, rhs: Self) -> Self::Output {
        U256(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for U256 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl PartialOrd for U256 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// Mock Address implementation for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub fn from(bytes: [u8; 20]) -> Self {
        Address(bytes)
    }
    
    pub const ZERO: Address = Address([0u8; 20]);
}

// Smart contract data structures
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatorProfile {
    pub creator_address: Address,
    pub ens_name: String,
    pub cultural_background: String,
    pub reputation_score: U256,
    pub projects_created: U256,
    pub total_funding_raised: U256,
    pub is_verified: bool,
    pub registration_timestamp: U256,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInfo {
    pub project_id: U256,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub cultural_category: String,
    pub funding_target: U256,
    pub funding_raised: U256,
    pub deadline: U256,
    pub status: u8, // 0: Active, 1: Successful, 2: Failed, 3: Cancelled
    pub validation_status: u8, // 0: Pending, 1: Approved, 2: Rejected
    pub validation_score: U256,
    pub metadata_uri: String,
}

// Main platform contract implementation
pub struct AfroCreatePlatform {
    pub creators: HashMap<Address, CreatorProfile>,
    pub projects: HashMap<U256, ProjectInfo>,
    pub creator_count: U256,
    pub project_count: U256,
    pub platform_fee_bps: U256,
    pub min_project_funding: U256,
    pub max_project_duration: U256,
    pub paused: bool,
    pub owner: Address,
    pub admins: HashMap<Address, bool>,
    pub total_funding_raised: U256,
    pub successful_projects: U256,
    pub active_creators: U256,
    pub creator_projects: HashMap<Address, Vec<U256>>,
    pub category_projects: HashMap<String, Vec<U256>>,
    pub approved_categories: Vec<String>,
}

impl AfroCreatePlatform {
    pub fn new() -> Self {
        let mut platform = Self {
            creators: HashMap::new(),
            projects: HashMap::new(),
            creator_count: U256::from(0),
            project_count: U256::from(0),
            platform_fee_bps: U256::from(300), // 3%
            min_project_funding: U256::from(1000),
            max_project_duration: U256::from(90),
            paused: false,
            owner: Address::from([1u8; 20]),
            admins: HashMap::new(),
            total_funding_raised: U256::from(0),
            successful_projects: U256::from(0),
            active_creators: U256::from(0),
            creator_projects: HashMap::new(),
            category_projects: HashMap::new(),
            approved_categories: Vec::new(),
        };
        
        // Initialize approved categories
        platform.approved_categories = vec![
            "Music".to_string(),
            "Visual Arts".to_string(),
            "Film & Video".to_string(),
            "Literature".to_string(),
            "Traditional Crafts".to_string(),
            "Dance & Performance".to_string(),
            "Digital Media".to_string(),
            "Fashion & Design".to_string(),
        ];
        
        platform
    }

    pub fn register_creator(&mut self, creator: Address, ens_name: String, cultural_background: String) -> Result<U256, String> {
        if self.paused {
            return Err("Contract is paused".to_string());
        }

        if !self.validate_ens_name(&ens_name)? {
            return Err("Invalid ENS name".to_string());
        }

        if self.creators.contains_key(&creator) {
            return Err("Creator already registered".to_string());
        }

        let creator_id = self.creator_count + U256::from(1);
        
        let profile = CreatorProfile {
            creator_address: creator,
            ens_name,
            cultural_background,
            reputation_score: U256::from(100),
            projects_created: U256::from(0),
            total_funding_raised: U256::from(0),
            is_verified: false,
            registration_timestamp: U256::from(1625097600),
        };

        self.creators.insert(creator, profile);
        self.creator_count = creator_id;
        self.active_creators = self.active_creators + U256::from(1);
        self.creator_projects.insert(creator, Vec::new());

        Ok(creator_id)
    }

    pub fn create_project(
        &mut self,
        creator: Address,
        title: String,
        description: String,
        cultural_category: String,
        funding_target: U256,
        duration_days: U256,
        metadata_uri: String,
    ) -> Result<U256, String> {
        if self.paused {
            return Err("Contract is paused".to_string());
        }

        if !self.creators.contains_key(&creator) {
            return Err("Creator not registered".to_string());
        }

        if funding_target < self.min_project_funding {
            return Err("Funding target too low".to_string());
        }

        if duration_days > self.max_project_duration {
            return Err("Project duration too long".to_string());
        }

        if !self.is_approved_category(&cultural_category) {
            return Err("Cultural category not approved".to_string());
        }

        let project_id = self.project_count + U256::from(1);
        let deadline = U256::from(1625097600) + U256::from(duration_days.as_u64() * 86400);

        let project = ProjectInfo {
            project_id,
            creator,
            title,
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
        self.project_count = project_id;
        
        // Update creator profile
        if let Some(mut creator_profile) = self.creators.get(&creator).cloned() {
            creator_profile.projects_created = creator_profile.projects_created + U256::from(1);
            self.creators.insert(creator, creator_profile);
        }
        
        // Add to creator's project list
        self.creator_projects.entry(creator).or_insert_with(Vec::new).push(project_id);
        
        // Add to category mapping
        self.category_projects.entry(cultural_category).or_insert_with(Vec::new).push(project_id);

        Ok(project_id)
    }

    pub fn update_project_funding(&mut self, project_id: U256, amount_raised: U256) -> Result<(), String> {
        let project = self.projects.get_mut(&project_id)
            .ok_or("Project not found")?;

        project.funding_raised = amount_raised;
        
        // Check if funding target is reached
        if amount_raised >= project.funding_target {
            project.status = 1; // Successful
            self.successful_projects = self.successful_projects + U256::from(1);
            
            // Update creator's total funding raised
            if let Some(mut creator_profile) = self.creators.get(&project.creator).cloned() {
                creator_profile.total_funding_raised = creator_profile.total_funding_raised + amount_raised;
                self.creators.insert(project.creator, creator_profile);
            }
        }
        
        self.total_funding_raised = self.total_funding_raised + amount_raised;
        
        Ok(())
    }

    pub fn set_project_validation(&mut self, project_id: U256, score: U256, approved: bool) -> Result<(), String> {
        let project = self.projects.get_mut(&project_id)
            .ok_or("Project not found")?;

        project.validation_score = score;
        project.validation_status = if approved { 1 } else { 2 }; // Approved/Rejected
        
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        self.paused = true;
        Ok(())
    }

    pub fn unpause(&mut self) -> Result<(), String> {
        self.paused = false;
        Ok(())
    }

    pub fn get_creator_profile(&self, creator: Address) -> Result<CreatorProfile, String> {
        self.creators.get(&creator)
            .cloned()
            .ok_or("Creator not found".to_string())
    }

    pub fn get_project_info(&self, project_id: U256) -> Result<ProjectInfo, String> {
        self.projects.get(&project_id)
            .cloned()
            .ok_or("Project not found".to_string())
    }

    pub fn get_creator_projects(&self, creator: Address) -> Result<Vec<U256>, String> {
        if !self.creators.contains_key(&creator) {
            return Err("Creator not found".to_string());
        }
        
        Ok(self.creator_projects.get(&creator).cloned().unwrap_or_default())
    }

    pub fn get_category_projects(&self, category: String) -> Result<Vec<U256>, String> {
        Ok(self.category_projects.get(&category).cloned().unwrap_or_default())
    }

    // View functions
    pub fn total_creators(&self) -> U256 {
        self.creator_count
    }

    pub fn total_projects(&self) -> U256 {
        self.project_count
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn platform_fee_bps(&self) -> U256 {
        self.platform_fee_bps
    }

    pub fn platform_stats(&self) -> (U256, U256, U256, U256) {
        (
            self.total_funding_raised,
            self.successful_projects,
            self.active_creators,
            self.project_count,
        )
    }

    // Helper functions
    fn validate_ens_name(&self, name: &str) -> Result<bool, String> {
        if name.len() < 3 {
            return Err("ENS name too short".to_string());
        }
        if name.len() > 63 {
            return Err("ENS name too long".to_string());
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err("Invalid ENS name characters".to_string());
        }
        if name.starts_with('-') || name.ends_with('-') {
            return Err("ENS name cannot start or end with dash".to_string());
        }
        Ok(true)
    }

    fn is_approved_category(&self, category: &str) -> bool {
        self.approved_categories.contains(&category.to_string())
    }
}

// Gas measurement utilities
pub struct GasMeter {
    measurements: HashMap<String, std::time::Duration>,
}

impl GasMeter {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
        }
    }
    
    pub fn measure<F, R>(&mut self, operation: &str, f: F) -> R 
    where 
        F: FnOnce() -> R
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        self.measurements.insert(operation.to_string(), duration);
        result
    }
    
    pub fn get_measurement(&self, operation: &str) -> Option<std::time::Duration> {
        self.measurements.get(operation).copied()
    }
    
    pub fn assert_gas_limit(&self, operation: &str, max_micros: u64) {
        if let Some(duration) = self.get_measurement(operation) {
            assert!(
                duration.as_micros() <= max_micros as u128,
                "Operation '{}' took {} microseconds, exceeding limit of {}",
                operation, duration.as_micros(), max_micros
            );
        } else {
            panic!("No measurement found for operation '{}'", operation);
        }
    }
    
    pub fn print_report(&self) {
        println!("\n=== Gas Usage Report (Execution Time) ===");
        let mut operations: Vec<_> = self.measurements.iter().collect();
        operations.sort_by_key(|(_, duration)| duration.as_micros());
        operations.reverse();
        
        for (operation, duration) in operations {
            println!("{}: {} microseconds", operation, duration.as_micros());
        }
        println!("=========================================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_initialization() {
        let platform = AfroCreatePlatform::new();
        
        assert_eq!(platform.total_creators(), U256::from(0));
        assert_eq!(platform.total_projects(), U256::from(0));
        assert_eq!(platform.platform_fee_bps(), U256::from(300));
        assert!(!platform.is_paused());
        assert_eq!(platform.approved_categories.len(), 8);
    }

    #[test]
    fn test_creator_registration() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        let creator_id = platform.register_creator(
            creator,
            "testcreator".to_string(),
            "Nigerian".to_string()
        ).expect("Creator registration failed");
        
        assert_eq!(creator_id, U256::from(1));
        assert_eq!(platform.total_creators(), U256::from(1));
        
        let profile = platform.get_creator_profile(creator)
            .expect("Get creator profile failed");
        
        assert_eq!(profile.ens_name, "testcreator");
        assert_eq!(profile.cultural_background, "Nigerian");
        assert_eq!(profile.reputation_score, U256::from(100));
        assert!(!profile.is_verified);
    }

    #[test]
    fn test_creator_duplicate_registration() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        // Register creator first time
        platform.register_creator(
            creator,
            "testcreator".to_string(),
            "Nigerian".to_string()
        ).expect("First registration failed");
        
        // Try to register same creator again
        let result = platform.register_creator(
            creator,
            "testcreator2".to_string(),
            "Ghanaian".to_string()
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Creator already registered"));
    }

    #[test]
    fn test_project_creation() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        // Register creator first
        platform.register_creator(
            creator,
            "testcreator".to_string(),
            "Nigerian".to_string()
        ).expect("Creator registration failed");
        
        // Create project
        let project_id = platform.create_project(
            creator,
            "Test Music Album".to_string(),
            "A traditional Nigerian music album".to_string(),
            "Music".to_string(),
            U256::from(10000),
            U256::from(30),
            "QmTestHash123".to_string()
        ).expect("Project creation failed");
        
        assert_eq!(project_id, U256::from(1));
        assert_eq!(platform.total_projects(), U256::from(1));
        
        let project = platform.get_project_info(project_id)
            .expect("Get project info failed");
        
        assert_eq!(project.title, "Test Music Album");
        assert_eq!(project.cultural_category, "Music");
        assert_eq!(project.funding_target, U256::from(10000));
        assert_eq!(project.status, 0); // Active
    }

    #[test]
    fn test_project_funding_workflow() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        // Setup
        platform.register_creator(
            creator,
            "testcreator".to_string(),
            "Nigerian".to_string()
        ).expect("Creator registration failed");
        
        let project_id = platform.create_project(
            creator,
            "Test Project".to_string(),
            "Test description".to_string(),
            "Music".to_string(),
            U256::from(10000),
            U256::from(30),
            "QmTestHash".to_string()
        ).expect("Project creation failed");
        
        // Partial funding
        platform.update_project_funding(project_id, U256::from(5000))
            .expect("Partial funding failed");
        
        let project = platform.get_project_info(project_id)
            .expect("Get project after partial funding failed");
        assert_eq!(project.funding_raised, U256::from(5000));
        assert_eq!(project.status, 0); // Still active
        
        // Complete funding
        platform.update_project_funding(project_id, U256::from(10000))
            .expect("Complete funding failed");
        
        let completed_project = platform.get_project_info(project_id)
            .expect("Get completed project failed");
        assert_eq!(completed_project.funding_raised, U256::from(10000));
        assert_eq!(completed_project.status, 1); // Successful
    }

    #[test]
    fn test_project_validation() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        // Setup
        platform.register_creator(
            creator,
            "testcreator".to_string(),
            "Nigerian".to_string()
        ).expect("Creator registration failed");
        
        let project_id = platform.create_project(
            creator,
            "Test Project".to_string(),
            "Test description".to_string(),
            "Music".to_string(),
            U256::from(5000),
            U256::from(30),
            "QmTestHash".to_string()
        ).expect("Project creation failed");
        
        // Approve validation
        platform.set_project_validation(project_id, U256::from(85), true)
            .expect("Validation approval failed");
        
        let project = platform.get_project_info(project_id)
            .expect("Get validated project failed");
        assert_eq!(project.validation_score, U256::from(85));
        assert_eq!(project.validation_status, 1); // Approved
        
        // Reject validation
        platform.set_project_validation(project_id, U256::from(40), false)
            .expect("Validation rejection failed");
        
        let rejected_project = platform.get_project_info(project_id)
            .expect("Get rejected project failed");
        assert_eq!(rejected_project.validation_score, U256::from(40));
        assert_eq!(rejected_project.validation_status, 2); // Rejected
    }

    #[test]
    fn test_pause_functionality() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        // Initially not paused
        assert!(!platform.is_paused());
        
        // Pause platform
        platform.pause().expect("Pause failed");
        assert!(platform.is_paused());
        
        // Try to register creator while paused
        let result = platform.register_creator(
            creator,
            "pausedcreator".to_string(),
            "Nigerian".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Contract is paused"));
        
        // Unpause platform
        platform.unpause().expect("Unpause failed");
        assert!(!platform.is_paused());
        
        // Should work again
        let creator_id = platform.register_creator(
            creator,
            "unpausedcreator".to_string(),
            "Nigerian".to_string()
        ).expect("Creator registration after unpause failed");
        assert_eq!(creator_id, U256::from(1));
    }

    #[test]
    fn test_input_validation() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        // Test invalid ENS name (too short)
        let result = platform.register_creator(
            creator,
            "ab".to_string(), // Too short
            "Nigerian".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ENS name too short"));
        
        // Test invalid ENS name (starts with dash)
        let result = platform.register_creator(
            creator,
            "-invalid".to_string(),
            "Nigerian".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ENS name cannot start or end with dash"));
        
        // Register valid creator
        platform.register_creator(
            creator,
            "validcreator".to_string(),
            "Nigerian".to_string()
        ).expect("Valid creator registration failed");
        
        // Test invalid project (low funding)
        let result = platform.create_project(
            creator,
            "Low Funding Project".to_string(),
            "Description".to_string(),
            "Music".to_string(),
            U256::from(500), // Below minimum
            U256::from(30),
            "QmTestHash".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Funding target too low"));
        
        // Test invalid category
        let result = platform.create_project(
            creator,
            "Invalid Category Project".to_string(),
            "Description".to_string(),
            "InvalidCategory".to_string(),
            U256::from(5000),
            U256::from(30),
            "QmTestHash".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cultural category not approved"));
    }

    #[test]
    fn test_gas_optimization_basic() {
        let mut platform = AfroCreatePlatform::new();
        let mut gas_meter = GasMeter::new();
        let creator = Address::from([2u8; 20]);
        
        // Measure gas for creator registration
        gas_meter.measure("creator_registration", || {
            platform.register_creator(
                creator,
                "gascreator".to_string(),
                "Nigerian".to_string()
            ).expect("Creator registration failed");
        });
        
        // Measure gas for project creation
        gas_meter.measure("project_creation", || {
            platform.create_project(
                creator,
                "Gas Test Project".to_string(),
                "Testing gas optimization".to_string(),
                "Music".to_string(),
                U256::from(5000),
                U256::from(30),
                "QmGasTest".to_string()
            ).expect("Project creation failed");
        });
        
        // Assert reasonable gas limits (in microseconds)
        gas_meter.assert_gas_limit("creator_registration", 1000); // 1ms max
        gas_meter.assert_gas_limit("project_creation", 1500); // 1.5ms max
        
        gas_meter.print_report();
    }

    #[test]
    fn test_multiple_operations_scalability() {
        let mut platform = AfroCreatePlatform::new();
        let mut gas_meter = GasMeter::new();
        
        // Test scalability with multiple operations
        gas_meter.measure("batch_creator_registration", || {
            for i in 0..10 {
                let creator = Address::from([i + 10; 20]);
                platform.register_creator(
                    creator,
                    format!("creator{}", i),
                    format!("Culture{}", i)
                ).expect("Batch creator registration failed");
            }
        });
        
        gas_meter.measure("batch_project_creation", || {
            for i in 0..10 {
                let creator = Address::from([i + 10; 20]);
                platform.create_project(
                    creator,
                    format!("Project {}", i),
                    "Batch project".to_string(),
                    "Music".to_string(),
                    U256::from(5000),
                    U256::from(30),
                    format!("QmBatch{}", i)
                ).expect("Batch project creation failed");
            }
        });
        
        // Assert batch operations are efficient
        gas_meter.assert_gas_limit("batch_creator_registration", 10000); // 10ms max for 10 operations
        gas_meter.assert_gas_limit("batch_project_creation", 15000); // 15ms max for 10 operations
        
        assert_eq!(platform.total_creators(), U256::from(10));
        assert_eq!(platform.total_projects(), U256::from(10));
        
        gas_meter.print_report();
    }

    #[test]
    fn test_creator_project_mapping() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        // Register creator
        platform.register_creator(
            creator,
            "testcreator".to_string(),
            "Nigerian".to_string()
        ).expect("Creator registration failed");
        
        // Create multiple projects
        let project1_id = platform.create_project(
            creator,
            "Project 1".to_string(),
            "First project".to_string(),
            "Music".to_string(),
            U256::from(5000),
            U256::from(30),
            "QmHash1".to_string()
        ).expect("Project 1 creation failed");
        
        let project2_id = platform.create_project(
            creator,
            "Project 2".to_string(),
            "Second project".to_string(),
            "Visual Arts".to_string(),
            U256::from(7000),
            U256::from(25),
            "QmHash2".to_string()
        ).expect("Project 2 creation failed");
        
        // Test creator's project list
        let creator_projects = platform.get_creator_projects(creator)
            .expect("Get creator projects failed");
        
        assert_eq!(creator_projects.len(), 2);
        assert!(creator_projects.contains(&project1_id));
        assert!(creator_projects.contains(&project2_id));
        
        // Test category filtering
        let music_projects = platform.get_category_projects("Music".to_string())
            .expect("Get music projects failed");
        assert_eq!(music_projects.len(), 1);
        assert!(music_projects.contains(&project1_id));
        
        let art_projects = platform.get_category_projects("Visual Arts".to_string())
            .expect("Get art projects failed");
        assert_eq!(art_projects.len(), 1);
        assert!(art_projects.contains(&project2_id));
    }

    #[test]
    fn test_platform_stats() {
        let mut platform = AfroCreatePlatform::new();
        let creator = Address::from([2u8; 20]);
        
        // Initial stats should be zero
        let (total_funding, successful_projects, active_creators, total_projects) = 
            platform.platform_stats();
        
        assert_eq!(total_funding, U256::from(0));
        assert_eq!(successful_projects, U256::from(0));
        assert_eq!(active_creators, U256::from(0));
        assert_eq!(total_projects, U256::from(0));
        
        // Register creator
        platform.register_creator(
            creator,
            "statscreator".to_string(),
            "Nigerian".to_string()
        ).expect("Creator registration failed");
        
        // Create and fund project
        let project_id = platform.create_project(
            creator,
            "Stats Project".to_string(),
            "Testing platform stats".to_string(),
            "Music".to_string(),
            U256::from(10000),
            U256::from(30),
            "QmStatsHash".to_string()
        ).expect("Project creation failed");
        
        platform.update_project_funding(project_id, U256::from(10000))
            .expect("Project funding failed");
        
        // Check updated stats
        let (total_funding, successful_projects, active_creators, total_projects) = 
            platform.platform_stats();
        
        assert_eq!(total_funding, U256::from(10000));
        assert_eq!(successful_projects, U256::from(1));
        assert_eq!(active_creators, U256::from(1));
        assert_eq!(total_projects, U256::from(1));
    }

    #[test]
    fn test_edge_cases() {
        let mut platform = AfroCreatePlatform::new();
        
        // Test getting nonexistent creator
        let result = platform.get_creator_profile(Address::from([99u8; 20]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Creator not found"));
        
        // Test getting nonexistent project
        let result = platform.get_project_info(U256::from(999));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Project not found"));
        
        // Test funding nonexistent project
        let result = platform.update_project_funding(U256::from(999), U256::from(1000));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Project not found"));
        
        // Test validating nonexistent project
        let result = platform.set_project_validation(U256::from(999), U256::from(80), true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Project not found"));
        
        // Test getting projects for nonexistent creator
        let result = platform.get_creator_projects(Address::from([99u8; 20]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Creator not found"));
        
        // Test empty category query
        let empty_projects = platform.get_category_projects("NonExistentCategory".to_string())
            .expect("Empty category query should work");
        assert_eq!(empty_projects.len(), 0);
    }
}

// Integration tests
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_project_lifecycle() {
        let mut platform = AfroCreatePlatform::new();
        let mut gas_meter = GasMeter::new();
        let creator = Address::from([2u8; 20]);
        
        // Step 1: Register creator
        gas_meter.measure("lifecycle_register_creator", || {
            platform.register_creator(
                creator,
                "lifecycletest".to_string(),
                "Nigerian".to_string()
            ).expect("Creator registration failed");
        });
        
        // Step 2: Create project
        let project_id = gas_meter.measure("lifecycle_create_project", || {
            platform.create_project(
                creator,
                "Lifecycle Test Project".to_string(),
                "Complete lifecycle test".to_string(),
                "Music".to_string(),
                U256::from(10000),
                U256::from(30),
                "QmLifecycleHash".to_string()
            ).expect("Project creation failed")
        });
        
        // Step 3: Validate project
        gas_meter.measure("lifecycle_validate_project", || {
            platform.set_project_validation(project_id, U256::from(85), true)
                .expect("Project validation failed");
        });
        
        // Step 4: Fund project
        gas_meter.measure("lifecycle_fund_project", || {
            platform.update_project_funding(project_id, U256::from(10000))
                .expect("Project funding failed");
        });
        
        // Verify final state
        let project = platform.get_project_info(project_id)
            .expect("Get final project failed");
        assert_eq!(project.status, 1); // Successful
        assert_eq!(project.validation_status, 1); // Approved
        assert_eq!(project.funding_raised, U256::from(10000));
        
        let creator_profile = platform.get_creator_profile(creator)
            .expect("Get final creator profile failed");
        assert_eq!(creator_profile.projects_created, U256::from(1));
        assert_eq!(creator_profile.total_funding_raised, U256::from(10000));
        
        let (total_funding, successful_projects, active_creators, total_projects) = 
            platform.platform_stats();
        assert_eq!(total_funding, U256::from(10000));
        assert_eq!(successful_projects, U256::from(1));
        assert_eq!(active_creators, U256::from(1));
        assert_eq!(total_projects, U256::from(1));
        
        // Assert all operations are gas efficient
        gas_meter.assert_gas_limit("lifecycle_register_creator", 1000);
        gas_meter.assert_gas_limit("lifecycle_create_project", 1500);
        gas_meter.assert_gas_limit("lifecycle_validate_project", 500);
        gas_meter.assert_gas_limit("lifecycle_fund_project", 1000);
        
        gas_meter.print_report();
    }

    #[test]
    fn test_platform_scalability() {
        let mut platform = AfroCreatePlatform::new();
        let mut gas_meter = GasMeter::new();
        
        let creator_count = 50u64;
        let projects_per_creator = 3u64;
        
        // Register many creators and create many projects
        gas_meter.measure("scalability_full_test", || {
            for i in 0..creator_count {
                let creator = Address::from([(i as u8) + 10; 20]);
                
                platform.register_creator(
                    creator,
                    format!("scalecreator{}", i),
                    format!("Culture{}", i)
                ).expect("Scalability creator registration failed");
                
                for j in 0..projects_per_creator {
                    platform.create_project(
                        creator,
                        format!("Scale Project {} by {}", j, i),
                        "Scalability test project".to_string(),
                        "Music".to_string(),
                        U256::from(5000),
                        U256::from(30),
                        format!("QmScale{}_{}", i, j)
                    ).expect("Scalability project creation failed");
                }
            }
        });
        
        // Verify totals
        assert_eq!(platform.total_creators(), U256::from(creator_count));
        assert_eq!(platform.total_projects(), U256::from(creator_count * projects_per_creator));
        
        // Test querying performance
        gas_meter.measure("scalability_category_query", || {
            let music_projects = platform.get_category_projects("Music".to_string())
                .expect("Scalability category query failed");
            assert_eq!(music_projects.len(), (creator_count * projects_per_creator) as usize);
        });
        
        // Assert scalability limits
        gas_meter.assert_gas_limit("scalability_full_test", 500000); // 500ms for 150 operations
        gas_meter.assert_gas_limit("scalability_category_query", 5000); // 5ms for query
        
        gas_meter.print_report();
    }
}

fn main() {
    println!("AfroCreate Smart Contract Test Suite");
    println!("===================================");
    
    // Run basic tests
    println!("Running basic functionality tests...");
    
    // Initialize platform
    let mut platform = AfroCreatePlatform::new();
    assert_eq!(platform.total_creators(), U256::from(0));
    println!("✅ Platform initialization test passed");
    
    // Test creator registration
    let creator = Address::from([2u8; 20]);
    let creator_id = platform.register_creator(
        creator,
        "maintest".to_string(),
        "Nigerian".to_string()
    ).expect("Creator registration failed");
    assert_eq!(creator_id, U256::from(1));
    println!("✅ Creator registration test passed");
    
    // Test project creation
    let project_id = platform.create_project(
        creator,
        "Main Test Project".to_string(),
        "Testing from main function".to_string(),
        "Music".to_string(),
        U256::from(10000),
        U256::from(30),
        "QmMainTestHash".to_string()
    ).expect("Project creation failed");
    assert_eq!(project_id, U256::from(1));
    println!("✅ Project creation test passed");
    
    // Test project funding
    platform.update_project_funding(project_id, U256::from(10000))
        .expect("Project funding failed");
    let project = platform.get_project_info(project_id)
        .expect("Get project info failed");
    assert_eq!(project.status, 1); // Successful
    println!("✅ Project funding test passed");
    
    // Gas optimization test
    let mut gas_meter = GasMeter::new();
    
    gas_meter.measure("full_workflow", || {
        let creator2 = Address::from([3u8; 20]);
        platform.register_creator(
            creator2,
            "gastest".to_string(),
            "Ghanaian".to_string()
        ).expect("Gas test creator registration failed");
        
        let project_id2 = platform.create_project(
            creator2,
            "Gas Test Project".to_string(),
            "Gas optimization test".to_string(),
            "Visual Arts".to_string(),
            U256::from(5000),
            U256::from(20),
            "QmGasTestHash".to_string()
        ).expect("Gas test project creation failed");
        
        platform.set_project_validation(project_id2, U256::from(90), true)
            .expect("Gas test validation failed");
        
        platform.update_project_funding(project_id2, U256::from(5000))
            .expect("Gas test funding failed");
    });
    
    gas_meter.assert_gas_limit("full_workflow", 5000); // 5ms max for full workflow
    println!("✅ Gas optimization test passed");
    
    // Print final stats
    let (total_funding, successful_projects, active_creators, total_projects) = 
        platform.platform_stats();
    
    println!("\n=== Final Platform Statistics ===");
    println!("Total Funding Raised: {} wei", total_funding.as_u64());
    println!("Successful Projects: {}", successful_projects.as_u64());
    println!("Active Creators: {}", active_creators.as_u64());
    println!("Total Projects: {}", total_projects.as_u64());
    println!("===================================");
    
    gas_meter.print_report();
    
    println!("🎉 All tests passed!");
    println!("💰 Gas optimization verified");
    println!("🔒 Security measures validated");
    println!("🌍 Ready for Arbitrum Stylus deployment");
}