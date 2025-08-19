use alloy_primitives::{Address, FixedBytes, U256};
use stylus_sdk::sol_interface;

#[sol_interface]
pub trait ENSRegistry {
    fn set_subnode_owner(node: FixedBytes<32>, label: FixedBytes<32>, owner: Address) -> FixedBytes<32>;
    fn set_resolver(node: FixedBytes<32>, resolver: Address);
    fn owner(node: FixedBytes<32>) -> Address;
    fn resolver(node: FixedBytes<32>) -> Address;
    fn ttl(node: FixedBytes<32>) -> u64;
    fn record_exists(node: FixedBytes<32>) -> bool;
}

#[sol_interface]
pub trait ENSResolver {
    fn set_text(node: FixedBytes<32>, key: String, value: String);
    fn text(node: FixedBytes<32>, key: String) -> String;
    fn set_addr(node: FixedBytes<32>, addr: Address);
    fn addr(node: FixedBytes<32>) -> Address;
    fn set_content_hash(node: FixedBytes<32>, hash: Vec<u8>);
    fn content_hash(node: FixedBytes<32>) -> Vec<u8>;
}

#[sol_interface]
pub trait ISuperfluid {
    fn create_flow(token: Address, receiver: Address, flow_rate: i128);
    fn update_flow(token: Address, receiver: Address, flow_rate: i128);
    fn delete_flow(token: Address, sender: Address, receiver: Address);
    fn get_flow(token: Address, sender: Address, receiver: Address) -> (u256, i128, u256, u256);
}

#[sol_interface]
pub trait IRevenueOracle {
    fn get_revenue_data(project_id: U256, source: String) -> (U256, bool, u256);
    fn validate_revenue_claim(project_id: U256, source: String, amount: U256, proof: Vec<u8>) -> bool;
    fn is_authorized_reporter(reporter: Address) -> bool;
    fn update_revenue_data(project_id: U256, source: String, amount: U256, timestamp: U256);
}

#[sol_interface]
pub trait IERC721 {
    fn balance_of(owner: Address) -> U256;
    fn owner_of(token_id: U256) -> Address;
    fn safe_transfer_from(from: Address, to: Address, token_id: U256);
    fn safe_transfer_from_with_data(from: Address, to: Address, token_id: U256, data: Vec<u8>);
    fn transfer_from(from: Address, to: Address, token_id: U256);
    fn approve(to: Address, token_id: U256);
    fn set_approval_for_all(operator: Address, approved: bool);
    fn get_approved(token_id: U256) -> Address;
    fn is_approved_for_all(owner: Address, operator: Address) -> bool;
}

#[sol_interface]
pub trait IERC721Metadata {
    fn name() -> String;
    fn symbol() -> String;
    fn token_uri(token_id: U256) -> String;
}

#[sol_interface]
pub trait IAfroCreatePlatform {
    fn register_creator(ens_subdomain: String, cultural_data: String) -> U256;
    fn create_project(project_data: Vec<u8>) -> U256;
    fn validate_ens_ownership(subdomain: String, claimer: Address) -> bool;
    fn get_creator_profile(creator: Address) -> Vec<u8>;
    fn get_project_info(project_id: U256) -> Vec<u8>;
    fn is_paused() -> bool;
}

#[sol_interface]
pub trait IProjectFunding {
    fn fund_project(project_id: U256, backer_ens_name: String) -> U256;
    fn release_milestone_funds(project_id: U256, milestone_id: U256);
    fn process_refunds(project_id: U256);
    fn get_funding_stats(project_id: U256) -> Vec<u8>;
    fn get_backer_contributions(project_id: U256, backer: Address) -> U256;
}

#[sol_interface] 
pub trait IRevenueShareNFT {
    fn mint_revenue_nft(to: Address, project_id: U256, funding_amount: U256, ens_data: String) -> U256;
    fn calculate_claimable_revenue(token_id: U256) -> U256;
    fn claim_revenue(token_id: U256) -> U256;
    fn get_revenue_stats(token_id: U256) -> Vec<u8>;
    fn get_project_holders(project_id: U256) -> Vec<U256>;
}

#[sol_interface]
pub trait ICulturalValidator {
    fn register_validator(ens_name: String, regions: Vec<String>, credentials: String, stake: U256) -> bool;
    fn submit_validation(project_id: U256, score: U256, feedback: String, elements: Vec<String>);
    fn finalize_validation(project_id: U256) -> U256;
    fn challenge_validation(project_id: U256, reason: String) -> U256;
    fn get_validation_status(project_id: U256) -> Vec<u8>;
    fn get_qualified_validators(cultural_region: String) -> Vec<Address>;
}