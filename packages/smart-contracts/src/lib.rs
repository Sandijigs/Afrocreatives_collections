extern crate mini_alloc;
use mini_alloc::MiniAlloc;

#[global_allocator]
static ALLOC: MiniAlloc = MiniAlloc::INIT;

pub mod types;
pub mod platform;
pub mod identity;
pub mod projects;
pub mod nfts;
pub mod revenue;
pub mod validation;
pub mod treasury;

// Re-export all main contracts
pub use platform::{AfroCreatePlatform, PlatformGovernance};
pub use identity::{ENSIntegration, CulturalIdentity};
pub use projects::{ProjectFunding, ProjectFactory};
pub use nfts::{RevenueShareNFT, CulturalBadgeNFT};
pub use revenue::{RevenueDistributor, OracleManager};
pub use validation::{CulturalValidator, CommunityValidator};
pub use treasury::{PlatformTreasury, ProjectEscrow};

// Export main platform contract as the default entrypoint
pub use platform::AfroCreatePlatform as Contract;

#[cfg(feature = "export-abi")]
fn main() {
    println!("AfroCreate Collective - Exporting Contract ABIs");
    println!("===============================================");
    
    // Export ABI for all main contracts
    platform::AfroCreatePlatform::abi();
    platform::PlatformGovernance::abi();
    identity::ENSIntegration::abi();
    identity::CulturalIdentity::abi();
    projects::ProjectFunding::abi();
    projects::ProjectFactory::abi();
    nfts::RevenueShareNFT::abi();
    nfts::CulturalBadgeNFT::abi();
    revenue::RevenueDistributor::abi();
    revenue::OracleManager::abi();
    validation::CulturalValidator::abi();
    validation::CommunityValidator::abi();
    treasury::PlatformTreasury::abi();
    treasury::ProjectEscrow::abi();
    
    println!("✅ All contract ABIs exported successfully!");
    println!("🌍 AfroCreate Collective ready for deployment to Arbitrum Stylus");
}