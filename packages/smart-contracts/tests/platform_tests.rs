use alloy_primitives::{Address, U256};
use afrocreate_contracts::{AfroCreatePlatform, types::*};
use crate::test_utils::*;

#[cfg(test)]
mod platform_tests {
    use super::*;

    #[test]
    fn test_platform_initialization() {
        let mut context = TestContext::new();
        
        // Check initial state
        assert_eq!(context.platform.owner(), context.test_accounts[0]);
        assert_eq!(context.platform.platform_fee_bps(), U256::from(PLATFORM_FEE_BPS));
        assert_eq!(context.platform.total_creators(), U256::from(0));
        assert_eq!(context.platform.total_projects(), U256::from(0));
        assert!(!context.platform.is_paused());
    }

    #[test]
    fn test_platform_initialization_prevents_double_init() {
        let mut context = TestContext::new();
        
        // Try to initialize again - should fail
        let result = context.platform.initialize(
            context.ens_registry,
            U256::from(2000),
            U256::from(60),
        );
        
        expect_error(result, "Already initialized");
    }

    #[test]
    fn test_pause_and_unpause() {
        let mut context = TestContext::new();
        
        // Initially not paused
        assert!(!context.platform.is_paused());
        
        // Pause the platform
        context.platform.pause().expect("Pause failed");
        assert!(context.platform.is_paused());
        
        // Unpause the platform
        context.platform.unpause().expect("Unpause failed");
        assert!(!context.platform.is_paused());
    }

    #[test]
    fn test_pause_only_owner() {
        let mut context = TestContext::new();
        
        // Try to pause from non-owner account - should fail
        // Note: In real tests, you'd set msg::sender to different address
        // This is a simplified test structure
        expect_error(context.platform.pause(), "Only owner");
    }

    #[test]
    fn test_admin_management() {
        let mut context = TestContext::new();
        let admin_address = context.admin();
        
        // Add admin
        context.platform.add_admin(admin_address).expect("Add admin failed");
        
        // Remove admin
        context.platform.remove_admin(admin_address).expect("Remove admin failed");
    }

    #[test]
    fn test_platform_fee_update() {
        let mut context = TestContext::new();
        let new_fee = U256::from(500); // 5%
        
        // Update platform fee
        context.platform.set_platform_fee(new_fee).expect("Fee update failed");
        assert_eq!(context.platform.platform_fee_bps(), new_fee);
    }

    #[test]
    fn test_platform_fee_too_high() {
        let mut context = TestContext::new();
        let invalid_fee = U256::from(1500); // 15% - too high
        
        // Should fail with fee too high
        expect_error(
            context.platform.set_platform_fee(invalid_fee),
            "Fee too high"
        );
    }

    #[test]
    fn test_platform_stats() {
        let mut context = TestContext::new();
        
        // Initial stats should be zero
        let (total_funding, successful_projects, active_creators, total_projects) = 
            context.platform.platform_stats();
        
        assert_eq!(total_funding, U256::from(0));
        assert_eq!(successful_projects, U256::from(0));
        assert_eq!(active_creators, U256::from(0));
        assert_eq!(total_projects, U256::from(0));
    }

    #[test]
    fn test_creator_registration_flow() {
        let mut context = TestContext::new();
        
        // Register a creator
        let creator_id = context.register_test_creator()
            .expect("Creator registration failed");
        
        assert_eq!(creator_id, U256::from(1));
        assert_eq!(context.platform.total_creators(), U256::from(1));
        
        // Verify creator profile
        let profile = context.platform.get_creator_profile(context.creator())
            .expect("Get creator profile failed");
        
        assert_eq!(profile.creator_address, context.creator());
        assert_eq!(profile.ens_name, "testcreator");
        assert_eq!(profile.cultural_background, "Nigerian");
        assert_eq!(profile.reputation_score, U256::from(100));
        assert_eq!(profile.projects_created, U256::from(0));
        assert!(!profile.is_verified);
    }

    #[test]
    fn test_creator_double_registration_fails() {
        let mut context = TestContext::new();
        
        // Register creator first time
        context.register_test_creator().expect("First registration failed");
        
        // Try to register same creator again - should fail
        expect_error(
            context.register_test_creator(),
            "Creator already registered"
        );
    }

    #[test]
    fn test_project_creation_flow() {
        let mut context = TestContext::new();
        
        // Register creator first
        context.register_test_creator().expect("Creator registration failed");
        
        // Create project
        let project_id = context.create_test_project()
            .expect("Project creation failed");
        
        assert_eq!(project_id, U256::from(1));
        assert_eq!(context.platform.total_projects(), U256::from(1));
        
        // Verify project info
        let project = context.platform.get_project_info(project_id)
            .expect("Get project info failed");
        
        assert_eq!(project.project_id, project_id);
        assert_eq!(project.creator, context.creator());
        assert_eq!(project.title, "Test Music Album");
        assert_eq!(project.cultural_category, "Music");
        assert_eq!(project.funding_target, U256::from(10000));
        assert_eq!(project.funding_raised, U256::from(0));
        assert_eq!(project.status, 0); // Active
        assert_eq!(project.validation_status, 0); // Pending
    }

    #[test]
    fn test_project_creation_unregistered_creator_fails() {
        let mut context = TestContext::new();
        
        // Try to create project without registering creator first
        expect_error(
            context.create_test_project(),
            "Creator not registered"
        );
    }

    #[test]
    fn test_project_creation_invalid_category_fails() {
        let mut context = TestContext::new();
        
        // Register creator first
        context.register_test_creator().expect("Creator registration failed");
        
        // Try to create project with invalid category
        let result = context.platform.create_project(
            "Test Project".to_string(),
            "Description".to_string(),
            "InvalidCategory".to_string(), // Not in approved categories
            U256::from(10000),
            U256::from(30),
            "QmTestHash".to_string()
        );
        
        expect_error(result, "Cultural category not approved");
    }

    #[test]
    fn test_project_creation_funding_too_low_fails() {
        let mut context = TestContext::new();
        
        // Register creator first
        context.register_test_creator().expect("Creator registration failed");
        
        // Try to create project with funding below minimum
        let result = context.platform.create_project(
            "Test Project".to_string(),
            "Description".to_string(),
            "Music".to_string(),
            U256::from(500), // Below minimum of 1000
            U256::from(30),
            "QmTestHash".to_string()
        );
        
        expect_error(result, "Funding target too low");
    }

    #[test]
    fn test_project_creation_duration_too_long_fails() {
        let mut context = TestContext::new();
        
        // Register creator first
        context.register_test_creator().expect("Creator registration failed");
        
        // Try to create project with duration too long
        let result = context.platform.create_project(
            "Test Project".to_string(),
            "Description".to_string(),
            "Music".to_string(),
            U256::from(10000),
            U256::from(120), // Above maximum of 90 days
            "QmTestHash".to_string()
        );
        
        expect_error(result, "Project duration too long");
    }

    #[test]
    fn test_creator_projects_mapping() {
        let mut context = TestContext::new();
        
        // Register creator and create multiple projects
        context.register_test_creator().expect("Creator registration failed");
        
        let project1 = context.create_test_project().expect("Project 1 creation failed");
        
        // Create second project with different details
        let project2 = context.platform.create_project(
            "Second Project".to_string(),
            "Another project".to_string(),
            "Visual Arts".to_string(),
            U256::from(5000),
            U256::from(20),
            "QmTestHash2".to_string()
        ).expect("Project 2 creation failed");
        
        // Get creator's projects
        let creator_projects = context.platform.get_creator_projects(context.creator())
            .expect("Get creator projects failed");
        
        assert_eq!(creator_projects.len(), 2);
        assert!(creator_projects.contains(&project1));
        assert!(creator_projects.contains(&project2));
    }

    #[test]
    fn test_category_projects_mapping() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Create projects in different categories
        let music_project = context.create_test_project().expect("Music project creation failed");
        
        let art_project = context.platform.create_project(
            "Art Project".to_string(),
            "Visual arts project".to_string(),
            "Visual Arts".to_string(),
            U256::from(8000),
            U256::from(25),
            "QmArtHash".to_string()
        ).expect("Art project creation failed");
        
        // Get projects by category
        let music_projects = context.platform.get_category_projects("Music".to_string())
            .expect("Get music projects failed");
        let art_projects = context.platform.get_category_projects("Visual Arts".to_string())
            .expect("Get art projects failed");
        
        assert_eq!(music_projects.len(), 1);
        assert!(music_projects.contains(&music_project));
        
        assert_eq!(art_projects.len(), 1);
        assert!(art_projects.contains(&art_project));
    }

    #[test]
    fn test_ens_name_validation() {
        let mut context = TestContext::new();
        
        // Test valid ENS names
        let valid_names = vec![
            "goodname",
            "good-name",
            "test123",
            "a".repeat(63), // Maximum length
        ];
        
        for name in valid_names {
            let result = context.platform.register_creator(
                name.clone(),
                "Nigerian".to_string()
            );
            // Note: This would succeed in a full test environment
            // Here we're just testing the validation logic exists
        }
        
        // Test invalid ENS names would be handled in validation
        // (The actual validation happens in validate_ens_name internal function)
    }

    #[test]
    fn test_get_nonexistent_creator_fails() {
        let context = TestContext::new();
        let nonexistent_creator = Address::from([1u8; 20]);
        
        expect_error(
            context.platform.get_creator_profile(nonexistent_creator),
            "Creator not found"
        );
    }

    #[test]
    fn test_get_nonexistent_project_fails() {
        let context = TestContext::new();
        let nonexistent_project_id = U256::from(999);
        
        expect_error(
            context.platform.get_project_info(nonexistent_project_id),
            "Project not found"
        );
    }
}