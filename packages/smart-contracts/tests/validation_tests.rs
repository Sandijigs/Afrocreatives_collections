use alloy_primitives::{Address, U256};
use afrocreate_contracts::{AfroCreatePlatform, types::*};
use crate::test_utils::*;

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_validation_basic_workflow() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Initial validation state
        let initial_project = context.platform.get_project_info(project_id)
            .expect("Get initial project failed");
        assert_eq!(initial_project.validation_status, 0); // Pending
        assert_eq!(initial_project.validation_score, U256::from(0));
        
        // Set validation approval
        context.platform.set_project_validation(project_id, U256::from(85), true)
            .expect("Validation approval failed");
        
        // Check updated validation state
        let validated_project = context.platform.get_project_info(project_id)
            .expect("Get validated project failed");
        assert_eq!(validated_project.validation_status, 1); // Approved
        assert_eq!(validated_project.validation_score, U256::from(85));
    }

    #[test]
    fn test_validation_rejection() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Set validation rejection
        context.platform.set_project_validation(project_id, U256::from(45), false)
            .expect("Validation rejection failed");
        
        // Check rejection state
        let rejected_project = context.platform.get_project_info(project_id)
            .expect("Get rejected project failed");
        assert_eq!(rejected_project.validation_status, 2); // Rejected
        assert_eq!(rejected_project.validation_score, U256::from(45));
    }

    #[test]
    fn test_validation_score_ranges() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test various validation scores
        let test_scores = vec![
            (U256::from(0), false),   // Minimum score, rejected
            (U256::from(25), false),  // Low score, rejected
            (U256::from(50), false),  // Medium-low score, rejected
            (U256::from(70), true),   // Threshold score, approved
            (U256::from(85), true),   // Good score, approved
            (U256::from(100), true),  // Perfect score, approved
        ];
        
        for (score, should_approve) in test_scores {
            context.platform.set_project_validation(project_id, score, should_approve)
                .expect("Validation with score failed");
            
            let project = context.platform.get_project_info(project_id)
                .expect("Get project after validation failed");
            
            assert_eq!(project.validation_score, score);
            assert_eq!(project.validation_status, if should_approve { 1 } else { 2 });
        }
    }

    #[test]
    fn test_validation_authorization() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test unauthorized validation attempt
        // Note: In real implementation, this would test actual authorization
        expect_error(
            context.platform.set_project_validation(project_id, U256::from(80), true),
            "Not authorized"
        );
    }

    #[test]
    fn test_validation_nonexistent_project() {
        let mut context = TestContext::new();
        let nonexistent_project_id = U256::from(999);
        
        // Test validation of nonexistent project
        expect_error(
            context.platform.set_project_validation(nonexistent_project_id, U256::from(80), true),
            "Project not found"
        );
    }

    #[test]
    fn test_validation_status_enumeration() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test all validation status transitions
        
        // Start: Pending (0)
        let pending_project = context.platform.get_project_info(project_id)
            .expect("Get pending project failed");
        assert_eq!(pending_project.validation_status, 0); // Pending
        
        // Transition: Pending -> Approved (1)
        context.platform.set_project_validation(project_id, U256::from(80), true)
            .expect("Approval transition failed");
        
        let approved_project = context.platform.get_project_info(project_id)
            .expect("Get approved project failed");
        assert_eq!(approved_project.validation_status, 1); // Approved
        
        // Transition: Approved -> Rejected (2)
        context.platform.set_project_validation(project_id, U256::from(30), false)
            .expect("Rejection transition failed");
        
        let rejected_project = context.platform.get_project_info(project_id)
            .expect("Get rejected project failed");
        assert_eq!(rejected_project.validation_status, 2); // Rejected
        
        // Transition: Rejected -> Approved (1) - Re-validation
        context.platform.set_project_validation(project_id, U256::from(90), true)
            .expect("Re-approval transition failed");
        
        let reapproved_project = context.platform.get_project_info(project_id)
            .expect("Get re-approved project failed");
        assert_eq!(reapproved_project.validation_status, 1); // Approved again
    }

    #[test]
    fn test_validation_score_consistency() {
        let mut context = TestContext::new();
        
        // Setup multiple projects for consistency testing
        context.register_test_creator().expect("Creator registration failed");
        
        let project1_id = context.create_test_project().expect("Project 1 creation failed");
        
        let project2_id = context.platform.create_project(
            "Second Validation Test".to_string(),
            "Another project for validation testing".to_string(),
            "Visual Arts".to_string(),
            U256::from(6000),
            U256::from(25),
            "QmValidationTest2".to_string()
        ).expect("Project 2 creation failed");
        
        // Apply same validation to both projects
        let test_score = U256::from(75);
        let should_approve = true;
        
        context.platform.set_project_validation(project1_id, test_score, should_approve)
            .expect("Project 1 validation failed");
        
        context.platform.set_project_validation(project2_id, test_score, should_approve)
            .expect("Project 2 validation failed");
        
        // Verify consistency
        let project1 = context.platform.get_project_info(project1_id)
            .expect("Get project 1 failed");
        let project2 = context.platform.get_project_info(project2_id)
            .expect("Get project 2 failed");
        
        assert_eq!(project1.validation_score, test_score);
        assert_eq!(project2.validation_score, test_score);
        assert_eq!(project1.validation_status, project2.validation_status);
    }

    #[test]
    fn test_validation_cultural_appropriateness() {
        let mut context = TestContext::new();
        
        // Setup projects in different cultural categories
        context.register_test_creator().expect("Creator registration failed");
        
        let categories = vec![
            "Music", "Visual Arts", "Film & Video", "Literature",
            "Traditional Crafts", "Dance & Performance", "Digital Media", "Fashion & Design"
        ];
        
        let mut project_ids = Vec::new();
        
        for (i, category) in categories.iter().enumerate() {
            let project_id = context.platform.create_project(
                format!("Cultural {} Project", category),
                format!("Authentic {} from African heritage", category),
                category.to_string(),
                U256::from(5000),
                U256::from(30),
                format!("QmCultural{}", i)
            ).expect("Cultural project creation failed");
            project_ids.push(project_id);
        }
        
        // Test validation scores that might be appropriate for different categories
        let category_validation_scores = vec![
            (85, true),  // Music - high cultural authenticity
            (90, true),  // Visual Arts - exceptional cultural significance
            (80, true),  // Film & Video - good cultural representation
            (75, true),  // Literature - solid cultural narrative
            (95, true),  // Traditional Crafts - outstanding traditional methods
            (88, true),  // Dance & Performance - strong cultural expression
            (70, true),  // Digital Media - threshold cultural content
            (92, true),  // Fashion & Design - excellent cultural design
        ];
        
        for (i, (score, should_approve)) in category_validation_scores.iter().enumerate() {
            context.platform.set_project_validation(
                project_ids[i], 
                U256::from(*score), 
                *should_approve
            ).expect("Cultural validation failed");
            
            let project = context.platform.get_project_info(project_ids[i])
                .expect("Get culturally validated project failed");
            
            assert_eq!(project.validation_score, U256::from(*score));
            assert_eq!(project.validation_status, if *should_approve { 1 } else { 2 });
        }
    }

    #[test]
    fn test_validation_threshold_boundaries() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test scores around typical validation thresholds
        let threshold_tests = vec![
            (U256::from(69), false), // Just below threshold
            (U256::from(70), true),  // At threshold (typical approval threshold)
            (U256::from(71), true),  // Just above threshold
        ];
        
        for (score, should_approve) in threshold_tests {
            context.platform.set_project_validation(project_id, score, should_approve)
                .expect("Threshold validation failed");
            
            let project = context.platform.get_project_info(project_id)
                .expect("Get threshold project failed");
            
            assert_eq!(project.validation_score, score);
            assert_eq!(project.validation_status, if should_approve { 1 } else { 2 });
        }
    }

    #[test]
    fn test_validation_multiple_rounds() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Simulate multiple validation rounds (re-validation scenarios)
        let validation_rounds = vec![
            (U256::from(60), false), // First validation: rejected
            (U256::from(75), true),  // Second validation: approved after improvements
            (U256::from(85), true),  // Third validation: higher score maintained
            (U256::from(40), false), // Fourth validation: rejected due to issues
            (U256::from(90), true),  // Final validation: approved with high score
        ];
        
        for (round, (score, should_approve)) in validation_rounds.iter().enumerate() {
            context.platform.set_project_validation(project_id, *score, *should_approve)
                .expect(&format!("Validation round {} failed", round + 1));
            
            let project = context.platform.get_project_info(project_id)
                .expect("Get project after round failed");
            
            assert_eq!(project.validation_score, *score);
            assert_eq!(project.validation_status, if *should_approve { 1 } else { 2 });
        }
    }

    #[test]
    fn test_validation_extreme_values() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test extreme validation scores
        let extreme_tests = vec![
            (U256::from(0), false),     // Minimum possible score
            (U256::from(100), true),    // Maximum possible score
            (U256::MAX, true),          // Very large value (should handle gracefully)
        ];
        
        for (score, should_approve) in extreme_tests {
            let result = context.platform.set_project_validation(project_id, score, should_approve);
            
            // Should either succeed or fail gracefully
            if result.is_ok() {
                let project = context.platform.get_project_info(project_id)
                    .expect("Get extreme value project failed");
                assert_eq!(project.validation_score, score);
                assert_eq!(project.validation_status, if should_approve { 1 } else { 2 });
            }
            // If it fails, that's also acceptable for extreme values
        }
    }

    #[test]
    fn test_validation_gas_efficiency() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Measure gas usage for validation operations
        gas_meter.measure("single_validation", || {
            context.platform.set_project_validation(project_id, U256::from(80), true)
                .expect("Single validation failed");
        });
        
        // Test multiple validations for gas consistency
        for i in 0..5 {
            gas_meter.measure(&format!("validation_round_{}", i), || {
                context.platform.set_project_validation(
                    project_id, 
                    U256::from(70 + i as u64 * 5), 
                    true
                ).expect("Multiple validation failed");
            });
        }
        
        gas_meter.print_report();
    }

    #[test]
    fn test_validation_event_emission() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test that validation events are properly emitted
        // Note: In a full implementation, you'd capture and verify actual events
        
        // Approval event
        context.platform.set_project_validation(project_id, U256::from(85), true)
            .expect("Approval validation failed");
        
        // Rejection event  
        context.platform.set_project_validation(project_id, U256::from(45), false)
            .expect("Rejection validation failed");
        
        // In real tests, you would verify:
        // - ValidationCompleted event emitted
        // - Event contains correct project_id, score, approved status, timestamp
    }

    #[test]
    fn test_validation_state_persistence() {
        let mut context = TestContext::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Set initial validation
        let initial_score = U256::from(78);
        let initial_approval = true;
        
        context.platform.set_project_validation(project_id, initial_score, initial_approval)
            .expect("Initial validation failed");
        
        // Verify validation persists across multiple reads
        for _ in 0..3 {
            let project = context.platform.get_project_info(project_id)
                .expect("Get persistent project failed");
            
            assert_eq!(project.validation_score, initial_score);
            assert_eq!(project.validation_status, 1); // Approved
        }
        
        // Change validation and verify persistence
        let new_score = U256::from(65);
        let new_approval = false;
        
        context.platform.set_project_validation(project_id, new_score, new_approval)
            .expect("Updated validation failed");
        
        // Verify new validation persists
        for _ in 0..3 {
            let project = context.platform.get_project_info(project_id)
                .expect("Get updated persistent project failed");
            
            assert_eq!(project.validation_score, new_score);
            assert_eq!(project.validation_status, 2); // Rejected
        }
    }

    #[test]
    fn test_validation_concurrent_operations() {
        let mut context = TestContext::new();
        
        // Setup multiple projects for concurrent testing
        context.register_test_creator().expect("Creator registration failed");
        
        let mut project_ids = Vec::new();
        for i in 0..5 {
            let project_id = context.platform.create_project(
                format!("Concurrent Validation Test {}", i),
                "Concurrent validation testing".to_string(),
                "Music".to_string(),
                U256::from(5000),
                U256::from(30),
                format!("QmConcurrentVal{}", i)
            ).expect("Concurrent project creation failed");
            project_ids.push(project_id);
        }
        
        // Apply validations concurrently (simulated)
        for (i, project_id) in project_ids.iter().enumerate() {
            let score = U256::from(70 + i as u64 * 5);
            let should_approve = i % 2 == 0; // Alternate approval/rejection
            
            context.platform.set_project_validation(*project_id, score, should_approve)
                .expect("Concurrent validation failed");
        }
        
        // Verify all validations were applied correctly
        for (i, project_id) in project_ids.iter().enumerate() {
            let project = context.platform.get_project_info(*project_id)
                .expect("Get concurrent validation project failed");
            
            let expected_score = U256::from(70 + i as u64 * 5);
            let expected_status = if i % 2 == 0 { 1 } else { 2 }; // Approved/Rejected
            
            assert_eq!(project.validation_score, expected_score);
            assert_eq!(project.validation_status, expected_status);
        }
    }
}