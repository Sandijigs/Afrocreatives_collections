use alloy_primitives::{Address, U256};
use afrocreate_contracts::{AfroCreatePlatform, types::*};
use crate::test_utils::*;

#[cfg(test)]
mod project_tests {
    use super::*;

    #[test]
    fn test_project_creation_basic() {
        let mut context = TestContext::new();
        
        // Register creator first
        context.register_test_creator().expect("Creator registration failed");
        
        // Create project
        let project_id = context.create_test_project().expect("Project creation failed");
        
        assert_eq!(project_id, U256::from(1));
        assert_eq!(context.platform.total_projects(), U256::from(1));
    }

    #[test]
    fn test_project_info_completeness() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Verify project info
        let project = context.platform.get_project_info(project_id)
            .expect("Get project info failed");
        
        assert_eq!(project.project_id, project_id);
        assert_eq!(project.creator, context.creator());
        assert_eq!(project.title, "Test Music Album");
        assert_eq!(project.description, "A traditional Nigerian music album");
        assert_eq!(project.cultural_category, "Music");
        assert_eq!(project.funding_target, U256::from(10000));
        assert_eq!(project.funding_raised, U256::from(0));
        assert_eq!(project.status, 0); // Active
        assert_eq!(project.validation_status, 0); // Pending
        assert_eq!(project.validation_score, U256::from(0));
        assert_eq!(project.metadata_uri, "QmTestHash123");
        assert!(project.deadline > U256::from(0));
    }

    #[test]
    fn test_project_cultural_categories() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Test all approved cultural categories
        let approved_categories = vec![
            "Music",
            "Visual Arts", 
            "Film & Video",
            "Literature",
            "Traditional Crafts",
            "Dance & Performance",
            "Digital Media",
            "Fashion & Design",
        ];
        
        for (i, category) in approved_categories.iter().enumerate() {
            let project_id = context.platform.create_project(
                format!("{} Project", category),
                format!("A project showcasing {}", category),
                category.to_string(),
                U256::from(5000),
                U256::from(30),
                format!("QmHash{}", i)
            ).expect(&format!("Project creation failed for category: {}", category));
            
            let project = context.platform.get_project_info(project_id)
                .expect("Get project info failed");
            assert_eq!(project.cultural_category, *category);
        }
    }

    #[test]
    fn test_project_invalid_category_rejection() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Test unapproved categories
        let invalid_categories = vec![
            "InvalidCategory",
            "Technology", 
            "Sports",
            "Finance",
            "Gaming",
            "",
        ];
        
        for category in invalid_categories {
            expect_error(
                context.platform.create_project(
                    "Invalid Category Project".to_string(),
                    "Testing invalid category".to_string(),
                    category.to_string(),
                    U256::from(5000),
                    U256::from(30),
                    "QmTestHash".to_string()
                ),
                "Cultural category not approved"
            );
        }
    }

    #[test]
    fn test_project_funding_limits() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Test minimum funding requirement (platform minimum is 1000)
        expect_error(
            context.platform.create_project(
                "Low Funding Project".to_string(),
                "Testing minimum funding".to_string(),
                "Music".to_string(),
                U256::from(999), // Below minimum
                U256::from(30),
                "QmTestHash".to_string()
            ),
            "Funding target too low"
        );
        
        // Test exact minimum funding (should work)
        let min_project = context.platform.create_project(
            "Minimum Funding Project".to_string(),
            "Testing exact minimum funding".to_string(),
            "Music".to_string(),
            U256::from(1000), // Exact minimum
            U256::from(30),
            "QmTestHash".to_string()
        );
        assert!(min_project.is_ok(), "Exact minimum funding should work");
        
        // Test high funding target (should work)
        let high_project = context.platform.create_project(
            "High Funding Project".to_string(),
            "Testing high funding target".to_string(),
            "Music".to_string(),
            U256::from(1000000), // Very high
            U256::from(30),
            "QmTestHash".to_string()
        );
        assert!(high_project.is_ok(), "High funding target should work");
    }

    #[test]
    fn test_project_duration_limits() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Test maximum duration requirement (platform maximum is 90 days)
        expect_error(
            context.platform.create_project(
                "Long Duration Project".to_string(),
                "Testing maximum duration".to_string(),
                "Music".to_string(),
                U256::from(5000),
                U256::from(91), // Above maximum
                "QmTestHash".to_string()
            ),
            "Project duration too long"
        );
        
        // Test exact maximum duration (should work)
        let max_project = context.platform.create_project(
            "Maximum Duration Project".to_string(),
            "Testing exact maximum duration".to_string(),
            "Music".to_string(),
            U256::from(5000),
            U256::from(90), // Exact maximum
            "QmTestHash".to_string()
        );
        assert!(max_project.is_ok(), "Exact maximum duration should work");
        
        // Test short duration (should work)
        let short_project = context.platform.create_project(
            "Short Duration Project".to_string(),
            "Testing short duration".to_string(),
            "Music".to_string(),
            U256::from(5000),
            U256::from(1), // Very short
            "QmTestHash".to_string()
        );
        assert!(short_project.is_ok(), "Short duration should work");
    }

    #[test]
    fn test_project_deadline_calculation() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Create project with 30 days duration
        let project_id = context.create_test_project().expect("Project creation failed");
        
        let project = context.platform.get_project_info(project_id)
            .expect("Get project info failed");
        
        // Deadline should be approximately 30 days (30 * 86400 seconds) from current time
        let expected_deadline = U256::from(context.current_timestamp + 30 * 86400);
        assert_within_range(project.deadline, expected_deadline, 1); // 1% tolerance
    }

    #[test]
    fn test_project_funding_updates() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Initial funding should be zero
        let initial_project = context.platform.get_project_info(project_id)
            .expect("Get initial project info failed");
        assert_eq!(initial_project.funding_raised, U256::from(0));
        assert_eq!(initial_project.status, 0); // Active
        
        // Update funding partially
        context.platform.update_project_funding(project_id, U256::from(5000))
            .expect("Partial funding update failed");
        
        let partial_project = context.platform.get_project_info(project_id)
            .expect("Get partially funded project info failed");
        assert_eq!(partial_project.funding_raised, U256::from(5000));
        assert_eq!(partial_project.status, 0); // Still active
        
        // Update funding to target (complete funding)
        context.platform.update_project_funding(project_id, U256::from(10000))
            .expect("Complete funding update failed");
        
        let complete_project = context.platform.get_project_info(project_id)
            .expect("Get completed project info failed");
        assert_eq!(complete_project.funding_raised, U256::from(10000));
        assert_eq!(complete_project.status, 1); // Successful
    }

    #[test]
    fn test_project_validation_workflow() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Initial validation status should be pending
        let initial_project = context.platform.get_project_info(project_id)
            .expect("Get initial project info failed");
        assert_eq!(initial_project.validation_status, 0); // Pending
        assert_eq!(initial_project.validation_score, U256::from(0));
        
        // Set validation (approved)
        context.platform.set_project_validation(project_id, U256::from(85), true)
            .expect("Validation approval failed");
        
        let approved_project = context.platform.get_project_info(project_id)
            .expect("Get approved project info failed");
        assert_eq!(approved_project.validation_status, 1); // Approved
        assert_eq!(approved_project.validation_score, U256::from(85));
        
        // Set validation (rejected)
        context.platform.set_project_validation(project_id, U256::from(40), false)
            .expect("Validation rejection failed");
        
        let rejected_project = context.platform.get_project_info(project_id)
            .expect("Get rejected project info failed");
        assert_eq!(rejected_project.validation_status, 2); // Rejected
        assert_eq!(rejected_project.validation_score, U256::from(40));
    }

    #[test]
    fn test_project_status_transitions() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Project starts as Active (0)
        let active_project = context.platform.get_project_info(project_id)
            .expect("Get active project info failed");
        assert_eq!(active_project.status, 0); // Active
        
        // Funding to target should make it Successful (1)
        context.platform.update_project_funding(project_id, U256::from(10000))
            .expect("Funding to target failed");
        
        let successful_project = context.platform.get_project_info(project_id)
            .expect("Get successful project info failed");
        assert_eq!(successful_project.status, 1); // Successful
        
        // Note: Failed (2) and Cancelled (3) statuses would be set by other mechanisms
        // like deadline expiration or explicit cancellation
    }

    #[test]
    fn test_project_creator_association() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Create multiple projects
        let project1_id = context.create_test_project().expect("Project 1 creation failed");
        
        let project2_id = context.platform.create_project(
            "Second Project".to_string(),
            "Another project by same creator".to_string(),
            "Visual Arts".to_string(),
            U256::from(8000),
            U256::from(25),
            "QmSecondHash".to_string()
        ).expect("Project 2 creation failed");
        
        // Verify both projects are associated with creator
        let creator_projects = context.platform.get_creator_projects(context.creator())
            .expect("Get creator projects failed");
        
        assert_eq!(creator_projects.len(), 2);
        assert!(creator_projects.contains(&project1_id));
        assert!(creator_projects.contains(&project2_id));
        
        // Verify projects have correct creator
        let project1 = context.platform.get_project_info(project1_id)
            .expect("Get project 1 info failed");
        assert_eq!(project1.creator, context.creator());
        
        let project2 = context.platform.get_project_info(project2_id)
            .expect("Get project 2 info failed");
        assert_eq!(project2.creator, context.creator());
    }

    #[test]
    fn test_project_category_filtering() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Create projects in different categories
        let music_project = context.create_test_project().expect("Music project creation failed");
        
        let art_project = context.platform.create_project(
            "Art Exhibition".to_string(),
            "Contemporary African art exhibition".to_string(),
            "Visual Arts".to_string(),
            U256::from(7000),
            U256::from(20),
            "QmArtHash".to_string()
        ).expect("Art project creation failed");
        
        let film_project = context.platform.create_project(
            "Documentary Film".to_string(),
            "Documentary about African heritage".to_string(),
            "Film & Video".to_string(),
            U256::from(15000),
            U256::from(45),
            "QmFilmHash".to_string()
        ).expect("Film project creation failed");
        
        // Test category filtering
        let music_projects = context.platform.get_category_projects("Music".to_string())
            .expect("Get music projects failed");
        assert_eq!(music_projects.len(), 1);
        assert!(music_projects.contains(&music_project));
        
        let art_projects = context.platform.get_category_projects("Visual Arts".to_string())
            .expect("Get art projects failed");
        assert_eq!(art_projects.len(), 1);
        assert!(art_projects.contains(&art_project));
        
        let film_projects = context.platform.get_category_projects("Film & Video".to_string())
            .expect("Get film projects failed");
        assert_eq!(film_projects.len(), 1);
        assert!(film_projects.contains(&film_project));
        
        // Test empty category
        let empty_projects = context.platform.get_category_projects("Literature".to_string())
            .expect("Get empty category projects failed");
        assert_eq!(empty_projects.len(), 0);
    }

    #[test]
    fn test_project_metadata_handling() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Test different metadata URI formats
        let metadata_uris = vec![
            "QmValidIPFSHash123",
            "ipfs://QmAnotherHash456",
            "https://example.com/metadata.json",
            "ar://ArweaveHash789",
            "", // Empty URI
        ];
        
        for (i, metadata_uri) in metadata_uris.iter().enumerate() {
            let project_id = context.platform.create_project(
                format!("Metadata Test Project {}", i),
                format!("Testing metadata URI: {}", metadata_uri),
                "Digital Media".to_string(),
                U256::from(5000),
                U256::from(30),
                metadata_uri.to_string()
            ).expect("Project with metadata creation failed");
            
            let project = context.platform.get_project_info(project_id)
                .expect("Get project with metadata failed");
            assert_eq!(project.metadata_uri, *metadata_uri);
        }
    }

    #[test]
    fn test_project_large_content_handling() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Test with large title and description
        let large_title = "A".repeat(500);
        let large_description = "B".repeat(2000);
        let large_metadata = "QmVeryLongMetadataHashThatExceedsNormalLength123456789";
        
        let result = context.platform.create_project(
            large_title.clone(),
            large_description.clone(),
            "Literature".to_string(),
            U256::from(5000),
            U256::from(30),
            large_metadata.to_string()
        );
        
        assert!(result.is_ok(), "Large content should be handled gracefully");
        
        if let Ok(project_id) = result {
            let project = context.platform.get_project_info(project_id)
                .expect("Get large content project failed");
            assert_eq!(project.title, large_title);
            assert_eq!(project.description, large_description);
            assert_eq!(project.metadata_uri, large_metadata);
        }
    }

    #[test]
    fn test_project_unauthorized_operations() {
        let mut context = TestContext::new();
        
        // Register creator and create project
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test unauthorized funding update
        expect_error(
            context.platform.update_project_funding(project_id, U256::from(5000)),
            "Not authorized"
        );
        
        // Test unauthorized validation
        expect_error(
            context.platform.set_project_validation(project_id, U256::from(80), true),
            "Not authorized"
        );
    }

    #[test]
    fn test_project_nonexistent_operations() {
        let mut context = TestContext::new();
        let nonexistent_project_id = U256::from(999);
        
        // Test operations on nonexistent project
        expect_error(
            context.platform.get_project_info(nonexistent_project_id),
            "Project not found"
        );
        
        expect_error(
            context.platform.update_project_funding(nonexistent_project_id, U256::from(5000)),
            "Project not found"
        );
        
        expect_error(
            context.platform.set_project_validation(nonexistent_project_id, U256::from(80), true),
            "Project not found"
        );
    }

    #[test]
    fn test_project_unregistered_creator_rejection() {
        let mut context = TestContext::new();
        
        // Try to create project without registering creator first
        expect_error(
            context.create_test_project(),
            "Creator not registered"
        );
    }

    #[test]
    fn test_project_funding_edge_cases() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test funding with zero amount
        context.platform.update_project_funding(project_id, U256::from(0))
            .expect("Zero funding update should work");
        
        let zero_project = context.platform.get_project_info(project_id)
            .expect("Get zero funded project failed");
        assert_eq!(zero_project.funding_raised, U256::from(0));
        assert_eq!(zero_project.status, 0); // Still active
        
        // Test funding beyond target
        context.platform.update_project_funding(project_id, U256::from(15000))
            .expect("Over-funding should work");
        
        let overfunded_project = context.platform.get_project_info(project_id)
            .expect("Get overfunded project failed");
        assert_eq!(overfunded_project.funding_raised, U256::from(15000));
        assert_eq!(overfunded_project.status, 1); // Successful (target reached)
    }

    #[test]
    fn test_project_validation_edge_cases() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test validation with extreme scores
        context.platform.set_project_validation(project_id, U256::from(0), false)
            .expect("Zero score validation should work");
        
        let zero_score_project = context.platform.get_project_info(project_id)
            .expect("Get zero score project failed");
        assert_eq!(zero_score_project.validation_score, U256::from(0));
        assert_eq!(zero_score_project.validation_status, 2); // Rejected
        
        context.platform.set_project_validation(project_id, U256::from(100), true)
            .expect("Perfect score validation should work");
        
        let perfect_score_project = context.platform.get_project_info(project_id)
            .expect("Get perfect score project failed");
        assert_eq!(perfect_score_project.validation_score, U256::from(100));
        assert_eq!(perfect_score_project.validation_status, 1); // Approved
    }
}