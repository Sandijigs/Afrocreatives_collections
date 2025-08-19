use alloy_primitives::{Address, U256, FixedBytes};
use alloy_sol_types::{sol, SolType};
use stylus_sdk::prelude::*;

pub mod events;
pub mod errors;
pub mod interfaces;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    struct CreatorProfile {
        address creator_address;
        string ens_name;
        string cultural_background;
        uint256 reputation_score;
        uint256 projects_created;
        uint256 total_funding_raised;
        bool is_verified;
        uint256 registration_timestamp;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct ProjectInfo {
        uint256 project_id;
        address creator;
        string title;
        string description;
        string cultural_category;
        uint256 funding_target;
        uint256 funding_raised;
        uint256 deadline;
        uint8 status; // 0: Active, 1: Successful, 2: Failed, 3: Cancelled
        uint8 validation_status; // 0: Pending, 1: Approved, 2: Rejected
        uint256 validation_score;
        string metadata_uri; // IPFS hash
    }

    #[derive(Debug, PartialEq, Eq)]
    struct ValidatorProfile {
        address validator_address;
        string ens_name;
        string[] expertise_regions;
        string credentials_uri;
        uint256 reputation_score;
        uint256 validations_completed;
        bool is_active;
        uint256 stake_amount;
        uint256 registration_timestamp;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct ValidationSubmission {
        address validator;
        uint256 score; // 0-100
        string feedback_uri;
        string[] cultural_elements;
        uint256 timestamp;
        bool is_final;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct FundingInfo {
        uint256 target;
        uint256 raised;
        uint256 deadline;
        uint8 status;
        address creator;
        uint256 backer_count;
        uint8 funding_model; // 0: AllOrNothing, 1: FlexibleFunding, 2: MilestoneBased
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Milestone {
        uint256 id;
        string title;
        string description;
        uint256 funding_amount;
        uint256 deadline;
        bool is_completed;
        bool funds_released;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct RevenueInfo {
        uint256 total_revenue;
        uint256 last_distribution_timestamp;
        string[] revenue_sources;
        bool oracle_verified;
        uint256 creator_share_bps;
        uint256 community_share_bps;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct DistributionEvent {
        uint256 timestamp;
        uint256 amount;
        uint256 recipients_count;
        string source;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Proposal {
        uint256 id;
        string title;
        string description;
        address proposer;
        uint256 start_time;
        uint256 end_time;
        uint256 for_votes;
        uint256 against_votes;
        uint8 status; // 0: Active, 1: Succeeded, 2: Failed, 3: Executed
        bytes execution_data;
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Vote {
        bool support;
        uint256 voting_power;
        uint256 timestamp;
    }
}

#[derive(SolidityType, Debug, Clone, PartialEq, Eq)]
pub enum ProjectStatus {
    Active,
    Successful,
    Failed,
    Cancelled,
}

#[derive(SolidityType, Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    Pending,
    Approved, 
    Rejected,
}

#[derive(SolidityType, Debug, Clone, PartialEq, Eq)]
pub enum FundingModel {
    AllOrNothing,
    FlexibleFunding,
    MilestoneBased,
}

#[derive(SolidityType, Debug, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Succeeded,
    Failed,
    Executed,
}

#[derive(SolidityType, Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Owner,
    Admin,
    Validator,
    Creator,
}

pub struct PackedProjectInfo {
    pub creator: Address,
    pub status: u8,
    pub category: u8,
    pub validation_score: u8,
}

impl PackedProjectInfo {
    pub fn pack(creator: Address, status: ProjectStatus, category: u8, validation_score: u8) -> Self {
        Self {
            creator,
            status: status as u8,
            category,
            validation_score,
        }
    }
    
    pub fn unpack_status(&self) -> ProjectStatus {
        match self.status {
            0 => ProjectStatus::Active,
            1 => ProjectStatus::Successful,
            2 => ProjectStatus::Failed,
            3 => ProjectStatus::Cancelled,
            _ => ProjectStatus::Active,
        }
    }
}

pub const PLATFORM_FEE_BPS: u16 = 300; // 3%
pub const MAX_VALIDATION_SCORE: u8 = 100;
pub const MIN_VALIDATION_SCORE: u8 = 0;
pub const VALIDATION_THRESHOLD: u8 = 70;
pub const MIN_VALIDATORS_REQUIRED: u8 = 3;

pub const AFROCREATE_ENS_NODE: FixedBytes<32> = FixedBytes([
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
]);