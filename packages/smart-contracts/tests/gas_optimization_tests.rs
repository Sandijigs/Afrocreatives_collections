use alloy_primitives::{Address, U256};
use afrocreate_contracts::{AfroCreatePlatform, types::*};
use crate::test_utils::*;

#[cfg(test)]
mod gas_optimization_tests {
    use super::*;

    // Gas limits for different operations (in gas units)
    const MAX_GAS_INITIALIZE: u64 = 200_000;
    const MAX_GAS_REGISTER_CREATOR: u64 = 150_000;
    const MAX_GAS_CREATE_PROJECT: u64 = 180_000;
    const MAX_GAS_UPDATE_PROJECT: u64 = 80_000;
    const MAX_GAS_GET_CREATOR_PROFILE: u64 = 30_000;
    const MAX_GAS_GET_PROJECT_INFO: u64 = 30_000;
    const MAX_GAS_PLATFORM_STATS: u64 = 25_000;
    const MAX_GAS_PAUSE_UNPAUSE: u64 = 40_000;
    const MAX_GAS_SET_PLATFORM_FEE: u64 = 35_000;

    #[test]
    fn test_initialization_gas_usage() {
        let mut gas_meter = GasMeter::new();
        
        gas_meter.measure("platform_initialization", || {
            let mut platform = AfroCreatePlatform::default();
            let ens_registry = Address::from([1u8; 20]);
            
            platform.initialize(
                ens_registry,
                U256::from(1000),
                U256::from(90),
            ).expect("Initialization failed");
        });
        
        gas_meter.assert_gas_limit("platform_initialization", MAX_GAS_INITIALIZE);
        gas_meter.print_report();
    }

    #[test]
    fn test_creator_registration_gas_usage() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        gas_meter.measure("creator_registration", || {
            context.register_test_creator().expect("Creator registration failed");
        });
        
        gas_meter.assert_gas_limit("creator_registration", MAX_GAS_REGISTER_CREATOR);
        gas_meter.print_report();
    }

    #[test]
    fn test_project_creation_gas_usage() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Register creator first
        context.register_test_creator().expect("Creator registration failed");
        
        gas_meter.measure("project_creation", || {
            context.create_test_project().expect("Project creation failed");
        });
        
        gas_meter.assert_gas_limit("project_creation", MAX_GAS_CREATE_PROJECT);
        gas_meter.print_report();
    }

    #[test]
    fn test_multiple_operations_gas_usage() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Test batch operations to ensure gas usage scales appropriately
        context.register_test_creator().expect("Creator registration failed");
        
        // Create multiple projects and measure gas usage
        for i in 0..5 {
            gas_meter.measure(&format!("project_creation_{}", i), || {
                let result = context.platform.create_project(
                    format!("Project {}", i),
                    format!("Description for project {}", i),
                    "Music".to_string(),
                    U256::from(10000),
                    U256::from(30),
                    format!("QmHash{}", i)
                );
                result.expect("Project creation failed");
            });
        }
        
        // Ensure gas usage doesn't increase significantly with project count
        for i in 0..5 {
            gas_meter.assert_gas_limit(&format!("project_creation_{}", i), MAX_GAS_CREATE_PROJECT);
        }
        
        gas_meter.print_report();
    }

    #[test]
    fn test_read_operations_gas_usage() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Setup data
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        // Test read operations
        gas_meter.measure("get_creator_profile", || {
            context.platform.get_creator_profile(context.creator())
                .expect("Get creator profile failed");
        });
        
        gas_meter.measure("get_project_info", || {
            context.platform.get_project_info(project_id)
                .expect("Get project info failed");
        });
        
        gas_meter.measure("platform_stats", || {
            context.platform.platform_stats();
        });
        
        gas_meter.measure("total_creators", || {
            context.platform.total_creators();
        });
        
        gas_meter.measure("total_projects", || {
            context.platform.total_projects();
        });
        
        // Assert gas limits for read operations
        gas_meter.assert_gas_limit("get_creator_profile", MAX_GAS_GET_CREATOR_PROFILE);
        gas_meter.assert_gas_limit("get_project_info", MAX_GAS_GET_PROJECT_INFO);
        gas_meter.assert_gas_limit("platform_stats", MAX_GAS_PLATFORM_STATS);
        
        gas_meter.print_report();
    }

    #[test]
    fn test_administrative_operations_gas_usage() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        gas_meter.measure("pause_platform", || {
            context.platform.pause().expect("Pause failed");
        });
        
        gas_meter.measure("unpause_platform", || {
            context.platform.unpause().expect("Unpause failed");
        });
        
        gas_meter.measure("set_platform_fee", || {
            context.platform.set_platform_fee(U256::from(400))
                .expect("Set platform fee failed");
        });
        
        gas_meter.measure("add_admin", || {
            context.platform.add_admin(context.admin())
                .expect("Add admin failed");
        });
        
        gas_meter.measure("remove_admin", || {
            context.platform.remove_admin(context.admin())
                .expect("Remove admin failed");
        });
        
        // Assert gas limits
        gas_meter.assert_gas_limit("pause_platform", MAX_GAS_PAUSE_UNPAUSE);
        gas_meter.assert_gas_limit("unpause_platform", MAX_GAS_PAUSE_UNPAUSE);
        gas_meter.assert_gas_limit("set_platform_fee", MAX_GAS_SET_PLATFORM_FEE);
        
        gas_meter.print_report();
    }

    #[test]
    fn test_project_funding_update_gas_usage() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        gas_meter.measure("update_project_funding", || {
            context.platform.update_project_funding(project_id, U256::from(5000))
                .expect("Update funding failed");
        });
        
        gas_meter.assert_gas_limit("update_project_funding", MAX_GAS_UPDATE_PROJECT);
        gas_meter.print_report();
    }

    #[test]
    fn test_storage_optimization_patterns() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Test that reading from packed storage is efficient
        context.register_test_creator().expect("Creator registration failed");
        
        // Create multiple projects to test storage patterns
        let mut project_ids = Vec::new();
        for i in 0..10 {
            let project_id = context.platform.create_project(
                format!("Project {}", i),
                format!("Description {}", i),
                "Music".to_string(),
                U256::from(5000 + i as u64 * 1000),
                U256::from(30),
                format!("QmHash{}", i)
            ).expect("Project creation failed");
            project_ids.push(project_id);
        }
        
        // Test sequential reads are efficient
        gas_meter.measure("sequential_project_reads", || {
            for project_id in &project_ids[0..5] {
                context.platform.get_project_info(*project_id)
                    .expect("Get project info failed");
            }
        });
        
        // Test random access patterns
        gas_meter.measure("random_project_reads", || {
            for &project_id in &[project_ids[2], project_ids[7], project_ids[1], project_ids[9]] {
                context.platform.get_project_info(project_id)
                    .expect("Get project info failed");
            }
        });
        
        gas_meter.print_report();
    }

    #[test]
    fn test_creator_projects_access_gas() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Setup multiple creators with projects
        context.register_test_creator().expect("Creator registration failed");
        
        // Create multiple projects for the creator
        for i in 0..5 {
            context.platform.create_project(
                format!("Project {}", i),
                format!("Description {}", i),
                match i % 3 {
                    0 => "Music",
                    1 => "Visual Arts", 
                    _ => "Film & Video",
                }.to_string(),
                U256::from(5000),
                U256::from(30),
                format!("QmHash{}", i)
            ).expect("Project creation failed");
        }
        
        // Test getting creator's projects
        gas_meter.measure("get_creator_projects", || {
            context.platform.get_creator_projects(context.creator())
                .expect("Get creator projects failed");
        });
        
        // Test getting projects by category
        gas_meter.measure("get_category_projects_music", || {
            context.platform.get_category_projects("Music".to_string())
                .expect("Get category projects failed");
        });
        
        gas_meter.measure("get_category_projects_visual_arts", || {
            context.platform.get_category_projects("Visual Arts".to_string())
                .expect("Get category projects failed");
        });
        
        gas_meter.print_report();
    }

    #[test]
    fn test_validation_operations_gas() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Setup
        context.register_test_creator().expect("Creator registration failed");
        let project_id = context.create_test_project().expect("Project creation failed");
        
        gas_meter.measure("set_project_validation", || {
            context.platform.set_project_validation(
                project_id, 
                U256::from(85), 
                true
            ).expect("Set validation failed");
        });
        
        gas_meter.print_report();
    }

    #[test]
    fn test_memory_usage_optimization() {
        let mut context = TestContext::new();
        
        // Test large string handling
        let long_description = "A".repeat(1000);
        let long_title = "B".repeat(100);
        
        context.register_test_creator().expect("Creator registration failed");
        
        // Should handle large strings efficiently
        let result = context.platform.create_project(
            long_title,
            long_description,
            "Music".to_string(),
            U256::from(10000),
            U256::from(30),
            "QmLongContentHash".to_string()
        );
        
        assert!(result.is_ok(), "Large content handling failed");
    }

    #[test]
    fn test_batch_operations_gas_efficiency() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Register creator
        context.register_test_creator().expect("Creator registration failed");
        
        // Measure gas for creating many projects
        let project_count = 20;
        let mut total_gas = 0u64;
        
        for i in 0..project_count {
            let gas_used = gas_meter.measure(&format!("project_{}", i), || {
                context.platform.create_project(
                    format!("Batch Project {}", i),
                    format!("Description {}", i),
                    "Music".to_string(),
                    U256::from(5000),
                    U256::from(30),
                    format!("QmBatch{}", i)
                ).expect("Batch project creation failed");
            });
            total_gas += gas_used;
        }
        
        let avg_gas = total_gas / project_count;
        println!("Average gas per project creation: {}", avg_gas);
        
        // Ensure average gas usage is reasonable
        assert!(avg_gas <= MAX_GAS_CREATE_PROJECT, 
                "Average gas {} exceeds limit {}", avg_gas, MAX_GAS_CREATE_PROJECT);
        
        gas_meter.print_report();
    }

    #[test]
    fn test_string_storage_optimization() {
        let mut context = TestContext::new();
        let mut gas_meter = GasMeter::new();
        
        // Test different string lengths
        let test_cases = vec![
            ("short", "A"),
            ("medium", &"B".repeat(32)),
            ("long", &"C".repeat(100)),
            ("very_long", &"D".repeat(500)),
        ];
        
        for (test_name, content) in test_cases {
            context.register_test_creator().expect("Creator registration failed");
            
            gas_meter.measure(&format!("string_storage_{}", test_name), || {
                context.platform.create_project(
                    content.to_string(),
                    content.to_string(),
                    "Music".to_string(),
                    U256::from(5000),
                    U256::from(30),
                    format!("Qm{}", test_name)
                ).expect("String storage test failed");
            });
        }
        
        gas_meter.print_report();
    }
}