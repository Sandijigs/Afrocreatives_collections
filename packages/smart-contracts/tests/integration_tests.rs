use alloy_primitives::{Address, U256};
use afrocreate_contracts::{AfroCreatePlatform, types::*};
use crate::test_utils::*;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_project_lifecycle() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Step 1: Register creator
        gas_meter.measure("lifecycle_register_creator", || {
            context.register_test_creator().expect("Creator registration failed");
        });
        
        // Verify creator state
        let creator_profile = context.platform.get_creator_profile(context.creator())
            .expect("Get creator profile failed");
        assert_eq!(creator_profile.projects_created, U256::from(0));
        assert_eq!(creator_profile.total_funding_raised, U256::from(0));
        
        // Step 2: Create project
        let project_id = gas_meter.measure("lifecycle_create_project", || {
            context.create_test_project().expect("Project creation failed")
        });
        
        // Verify project state
        let project = context.platform.get_project_info(project_id)
            .expect("Get project info failed");
        assert_eq!(project.status, 0); // Active
        assert_eq!(project.validation_status, 0); // Pending
        assert_eq!(project.funding_raised, U256::from(0));
        
        // Step 3: Update project funding (simulate backing)
        gas_meter.measure("lifecycle_fund_project", || {
            context.platform.update_project_funding(project_id, U256::from(5000))
                .expect("Funding update failed");
        });
        
        // Verify funding update
        let updated_project = context.platform.get_project_info(project_id)
            .expect("Get updated project info failed");
        assert_eq!(updated_project.funding_raised, U256::from(5000));
        assert_eq!(updated_project.status, 0); // Still active (not fully funded)
        
        // Step 4: Validate project
        gas_meter.measure("lifecycle_validate_project", || {
            context.platform.set_project_validation(project_id, U256::from(85), true)
                .expect("Project validation failed");
        });
        
        // Verify validation
        let validated_project = context.platform.get_project_info(project_id)
            .expect("Get validated project info failed");
        assert_eq!(validated_project.validation_status, 1); // Approved
        assert_eq!(validated_project.validation_score, U256::from(85));
        
        // Step 5: Complete funding (reach target)
        gas_meter.measure("lifecycle_complete_funding", || {
            context.platform.update_project_funding(project_id, U256::from(10000))
                .expect("Complete funding failed");
        });
        
        // Verify project completion
        let completed_project = context.platform.get_project_info(project_id)
            .expect("Get completed project info failed");
        assert_eq!(completed_project.status, 1); // Successful
        assert_eq!(completed_project.funding_raised, U256::from(10000));
        
        // Verify creator profile updated
        let final_creator_profile = context.platform.get_creator_profile(context.creator())
            .expect("Get final creator profile failed");
        assert_eq!(final_creator_profile.projects_created, U256::from(1));
        assert_eq!(final_creator_profile.total_funding_raised, U256::from(10000));
        
        // Verify platform stats
        let (total_funding, successful_projects, active_creators, total_projects) = 
            context.platform.platform_stats();
        assert_eq!(total_funding, U256::from(10000));
        assert_eq!(successful_projects, U256::from(1));
        assert_eq!(active_creators, U256::from(1));
        assert_eq!(total_projects, U256::from(1));
        
        gas_meter.print_report();
    }

    #[test]
    fn test_multiple_creators_and_projects() {
        let mut context = TestContext::new();
        
        // Create multiple test accounts
        let creators = vec![
            context.test_accounts[1],
            context.test_accounts[2], 
            context.test_accounts[3],
        ];
        
        let mut creator_ids = Vec::new();
        let mut project_ids = Vec::new();
        
        // Register multiple creators
        for (i, creator) in creators.iter().enumerate() {
            // In real implementation, would set msg::sender to creator
            let creator_id = context.platform.register_creator(
                format!("creator{}", i),
                format!("Culture{}", i)
            ).expect("Creator registration failed");
            creator_ids.push(creator_id);
        }
        
        // Each creator creates multiple projects
        for (i, _creator) in creators.iter().enumerate() {
            for j in 0..3 {
                let project_id = context.platform.create_project(
                    format!("Project {} by Creator {}", j, i),
                    format!("Description for project {} by creator {}", j, i),
                    match j % 3 {
                        0 => "Music",
                        1 => "Visual Arts",
                        _ => "Film & Video"
                    }.to_string(),
                    U256::from(5000 + j as u64 * 1000),
                    U256::from(30),
                    format!("QmHash{}_{}", i, j)
                ).expect("Project creation failed");
                project_ids.push(project_id);
            }
        }
        
        // Verify total counts
        assert_eq!(context.platform.total_creators(), U256::from(3));
        assert_eq!(context.platform.total_projects(), U256::from(9));
        
        // Test category filtering
        let music_projects = context.platform.get_category_projects("Music".to_string())
            .expect("Get music projects failed");
        assert_eq!(music_projects.len(), 3); // One per creator
        
        let art_projects = context.platform.get_category_projects("Visual Arts".to_string())
            .expect("Get art projects failed");
        assert_eq!(art_projects.len(), 3); // One per creator
    }

    #[test]
    fn test_platform_governance_integration() {
        let mut context = TestContext::new();
        
        // Setup initial state
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test administrative operations
        let new_admin = context.admin();
        context.platform.add_admin(new_admin).expect("Add admin failed");
        
        // Test platform fee changes
        let old_fee = context.platform.platform_fee_bps();
        let new_fee = U256::from(400); // 4%
        
        context.platform.set_platform_fee(new_fee).expect("Set platform fee failed");
        assert_eq!(context.platform.platform_fee_bps(), new_fee);
        
        // Test pause/unpause affects operations
        context.platform.pause().expect("Pause failed");
        assert!(context.platform.is_paused());
        
        // Verify creator registration fails when paused
        expect_error(
            context.platform.register_creator(
                "pausedcreator".to_string(),
                "TestCulture".to_string()
            ),
            "Contract is paused"
        );
        
        // Unpause and verify operations work again
        context.platform.unpause().expect("Unpause failed");
        assert!(!context.platform.is_paused());
        
        // Should be able to register creator again
        context.platform.register_creator(
            "unpausedcreator".to_string(),
            "TestCulture".to_string()
        ).expect("Creator registration after unpause failed");
    }

    #[test]
    fn test_funding_and_validation_workflow() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test partial funding
        context.platform.update_project_funding(project_id, U256::from(3000))
            .expect("Partial funding failed");
        
        let project = context.platform.get_project_info(project_id)
            .expect("Get project info failed");
        assert_eq!(project.funding_raised, U256::from(3000));
        assert_eq!(project.status, 0); // Still active
        
        // Test validation with different scores
        let test_scores = vec![
            (U256::from(60), false), // Below threshold, rejected
            (U256::from(80), true),  // Above threshold, approved
        ];
        
        for (score, should_approve) in test_scores {
            context.platform.set_project_validation(project_id, score, should_approve)
                .expect("Validation failed");
            
            let validated_project = context.platform.get_project_info(project_id)
                .expect("Get validated project failed");
            assert_eq!(validated_project.validation_score, score);
            assert_eq!(validated_project.validation_status, if should_approve { 1 } else { 2 });
        }
        
        // Test full funding triggers success
        context.platform.update_project_funding(project_id, U256::from(10000))
            .expect("Full funding failed");
        
        let successful_project = context.platform.get_project_info(project_id)
            .expect("Get successful project failed");
        assert_eq!(successful_project.status, 1); // Successful
        assert_eq!(successful_project.funding_raised, U256::from(10000));
        
        // Verify platform stats updated
        let (total_funding, successful_projects, _, _) = context.platform.platform_stats();
        assert_eq!(total_funding, U256::from(10000));
        assert_eq!(successful_projects, U256::from(1));
    }

    #[test]
    fn test_creator_reputation_system() {
        let mut context = TestContext::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        let initial_profile = context.platform.get_creator_profile(context.creator())
            .expect("Get initial profile failed");
        assert_eq!(initial_profile.reputation_score, U256::from(100)); // Starting reputation
        
        // Create successful project
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Validate project positively
        context.platform.set_project_validation(project_id, U256::from(90), true)
            .expect("Validation failed");
        
        // Complete funding
        context.platform.update_project_funding(project_id, U256::from(10000))
            .expect("Funding completion failed");
        
        // Verify creator profile updated
        let updated_profile = context.platform.get_creator_profile(context.creator())
            .expect("Get updated profile failed");
        assert_eq!(updated_profile.projects_created, U256::from(1));
        assert_eq!(updated_profile.total_funding_raised, U256::from(10000));
        
        // Test multiple projects impact
        let second_project_id = context.platform.create_project(
            "Second Project".to_string(),
            "Another cultural project".to_string(),
            "Visual Arts".to_string(),
            U256::from(8000),
            U256::from(25),
            "QmSecondHash".to_string()
        ).expect("Second project creation failed");
        
        context.platform.update_project_funding(second_project_id, U256::from(8000))
            .expect("Second project funding failed");
        
        let final_profile = context.platform.get_creator_profile(context.creator())
            .expect("Get final profile failed");
        assert_eq!(final_profile.projects_created, U256::from(2));
        assert_eq!(final_profile.total_funding_raised, U256::from(18000));
    }

    #[test]
    fn test_ens_integration_workflow() {
        let mut context = TestContext::new();
        
        // Test ENS name validation
        let valid_names = vec![
            "goodname",
            "test-name",
            "creator123",
            "african-artist",
        ];
        
        for (i, name) in valid_names.iter().enumerate() {
            let result = context.platform.register_creator(
                name.clone(),
                format!("Culture{}", i)
            );
            assert!(result.is_ok(), "Valid ENS name {} should work", name);
        }
        
        // Test subdomain registry
        // In a full implementation, this would interact with actual ENS contracts
        let ownership_test = context.platform.validate_ens_ownership(
            "testcreator",
            context.creator()
        );
        assert!(ownership_test.is_ok(), "ENS ownership validation should work");
    }

    #[test]
    fn test_project_categorization_system() {
        let mut context = TestContext::new();
        
        // Register multiple creators
        for i in 0..3 {
            context.platform.register_creator(
                format!("creator{}", i),
                format!("Culture{}", i)
            ).expect("Creator registration failed");
        }
        
        // Create projects in different categories
        let categories = vec![
            "Music", "Visual Arts", "Film & Video", "Literature",
            "Traditional Crafts", "Dance & Performance", "Digital Media", "Fashion & Design"
        ];
        
        let mut category_project_counts = std::collections::HashMap::new();
        
        for (i, category) in categories.iter().enumerate() {
            let project_id = context.platform.create_project(
                format!("Project in {}", category),
                format!("A {} project", category),
                category.to_string(),
                U256::from(5000),
                U256::from(30),
                format!("QmHash{}", i)
            ).expect("Project creation failed");
            
            *category_project_counts.entry(category.clone()).or_insert(0) += 1;
            
            // Verify project is in correct category
            let project = context.platform.get_project_info(project_id)
                .expect("Get project info failed");
            assert_eq!(project.cultural_category, *category);
        }
        
        // Test category filtering
        for category in categories {
            let projects = context.platform.get_category_projects(category.to_string())
                .expect("Get category projects failed");
            let expected_count = category_project_counts.get(&category.to_string()).unwrap_or(&0);
            assert_eq!(projects.len(), *expected_count);
        }
    }

    #[test]
    fn test_error_recovery_and_edge_cases() {
        let mut context = TestContext::new();
        
        // Test operations with invalid IDs
        expect_error(
            context.platform.get_project_info(U256::from(999)),
            "Project not found"
        );
        
        expect_error(
            context.platform.update_project_funding(U256::from(999), U256::from(1000)),
            "Project not found"
        );
        
        expect_error(
            context.platform.set_project_validation(U256::from(999), U256::from(80), true),
            "Project not found"
        );
        
        // Test operations on unregistered creators
        expect_error(
            context.platform.get_creator_profile(Address::from([99u8; 20])),
            "Creator not found"
        );
        
        expect_error(
            context.platform.get_creator_projects(Address::from([99u8; 20])),
            "Creator not found"
        );
        
        // Test empty category queries
        let empty_category_projects = context.platform.get_category_projects("NonExistentCategory".to_string())
            .expect("Empty category query should work");
        assert_eq!(empty_category_projects.len(), 0);
    }

    #[test]
    fn test_platform_scalability() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Test platform performance with many creators and projects
        let creator_count = 10;
        let projects_per_creator = 5;
        
        // Register many creators
        gas_meter.measure("scalability_register_creators", || {
            for i in 0..creator_count {
                context.platform.register_creator(
                    format!("scalecreator{}", i),
                    format!("ScaleCulture{}", i)
                ).expect("Bulk creator registration failed");
            }
        });
        
        // Create many projects
        gas_meter.measure("scalability_create_projects", || {
            for i in 0..creator_count {
                for j in 0..projects_per_creator {
                    context.platform.create_project(
                        format!("Scale Project {} by {}", j, i),
                        format!("Scale test project"),
                        "Music".to_string(),
                        U256::from(5000),
                        U256::from(30),
                        format!("QmScaleHash{}_{}", i, j)
                    ).expect("Bulk project creation failed");
                }
            }
        });
        
        // Verify total counts
        assert_eq!(context.platform.total_creators(), U256::from(creator_count));
        assert_eq!(context.platform.total_projects(), U256::from(creator_count * projects_per_creator));
        
        // Test querying performance
        gas_meter.measure("scalability_category_query", || {
            let music_projects = context.platform.get_category_projects("Music".to_string())
                .expect("Bulk category query failed");
            assert_eq!(music_projects.len(), creator_count * projects_per_creator);
        });
        
        gas_meter.print_report();
    }

    #[test]
    fn test_concurrent_operations() {
        let mut context = TestContext::new();
        
        // Register creators for concurrent testing
        for i in 0..5 {
            context.platform.register_creator(
                format!("concurrent{}", i),
                format!("ConcurrentCulture{}", i)
            ).expect("Concurrent creator registration failed");
        }
        
        // Simulate concurrent project creation
        let mut project_ids = Vec::new();
        for i in 0..5 {
            let project_id = context.platform.create_project(
                format!("Concurrent Project {}", i),
                "Concurrent test project".to_string(),
                "Music".to_string(),
                U256::from(5000),
                U256::from(30),
                format!("QmConcurrent{}", i)
            ).expect("Concurrent project creation failed");
            project_ids.push(project_id);
        }
        
        // Simulate concurrent funding updates
        for (i, project_id) in project_ids.iter().enumerate() {
            context.platform.update_project_funding(*project_id, U256::from((i + 1) as u64 * 1000))
                .expect("Concurrent funding update failed");
        }
        
        // Verify all projects have correct funding
        for (i, project_id) in project_ids.iter().enumerate() {
            let project = context.platform.get_project_info(*project_id)
                .expect("Get concurrent project failed");
            assert_eq!(project.funding_raised, U256::from((i + 1) as u64 * 1000));
        }
    }
}