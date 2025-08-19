use alloy_primitives::{Address, U256, FixedBytes};
use alloy_sol_types::sol;

sol! {
    // Platform Events
    #[derive(Debug)]
    event CreatorRegistered(
        address indexed creator,
        string ens_name,
        string cultural_background,
        uint256 timestamp
    );

    #[derive(Debug)]
    event ProjectCreated(
        uint256 indexed project_id,
        address indexed creator,
        string title,
        string cultural_category,
        uint256 funding_target,
        uint256 deadline
    );

    #[derive(Debug)]
    event ProjectFunded(
        uint256 indexed project_id,
        address indexed backer,
        uint256 amount,
        uint256 total_raised
    );

    #[derive(Debug)]
    event ProjectValidated(
        uint256 indexed project_id,
        address indexed validator,
        uint256 score,
        bool approved
    );

    #[derive(Debug)]
    event ValidationCompleted(
        uint256 indexed project_id,
        uint256 final_score,
        bool approved,
        uint256 timestamp
    );

    // Revenue Events
    #[derive(Debug)]
    event RevenueAdded(
        uint256 indexed project_id,
        string source,
        uint256 amount,
        uint256 timestamp
    );

    #[derive(Debug)]
    event RevenueDistributed(
        uint256 indexed project_id,
        uint256 total_amount,
        uint256 creator_share,
        uint256 community_share,
        uint256 platform_fee
    );

    #[derive(Debug)]
    event RevenueClaimed(
        uint256 indexed token_id,
        address indexed holder,
        uint256 amount
    );

    // NFT Events
    #[derive(Debug)]
    event RevenueNFTMinted(
        uint256 indexed token_id,
        uint256 indexed project_id,
        address indexed recipient,
        uint256 funding_amount,
        uint256 revenue_share_bps
    );

    #[derive(Debug)]
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 indexed token_id
    );

    #[derive(Debug)]
    event Approval(
        address indexed owner,
        address indexed approved,
        uint256 indexed token_id
    );

    #[derive(Debug)]
    event ApprovalForAll(
        address indexed owner,
        address indexed operator,
        bool approved
    );

    // ENS Events
    #[derive(Debug)]
    event ENSSubdomainRegistered(
        bytes32 indexed node,
        string subdomain,
        address indexed owner,
        uint256 timestamp
    );

    #[derive(Debug)]
    event CulturalMetadataUpdated(
        bytes32 indexed node,
        string key,
        string value,
        uint256 timestamp
    );

    #[derive(Debug)]
    event ReputationUpdated(
        bytes32 indexed node,
        uint256 old_score,
        uint256 new_score
    );

    // Validator Events
    #[derive(Debug)]
    event ValidatorRegistered(
        address indexed validator,
        string ens_name,
        string[] expertise_regions,
        uint256 stake_amount
    );

    #[derive(Debug)]
    event ValidatorSlashed(
        address indexed validator,
        uint256 amount,
        string reason
    );

    // Governance Events
    #[derive(Debug)]
    event ProposalCreated(
        uint256 indexed proposal_id,
        address indexed proposer,
        string title,
        uint256 start_time,
        uint256 end_time
    );

    #[derive(Debug)]
    event VoteCast(
        uint256 indexed proposal_id,
        address indexed voter,
        bool support,
        uint256 voting_power
    );

    #[derive(Debug)]
    event ProposalExecuted(
        uint256 indexed proposal_id,
        bool success
    );

    // Platform Management Events
    #[derive(Debug)]
    event PlatformPaused(uint256 timestamp);

    #[derive(Debug)]
    event PlatformUnpaused(uint256 timestamp);

    #[derive(Debug)]
    event PlatformFeeUpdated(uint256 old_fee_bps, uint256 new_fee_bps);

    #[derive(Debug)]
    event EmergencyWithdrawal(
        address indexed token,
        address indexed recipient,
        uint256 amount
    );

    // Milestone Events
    #[derive(Debug)]
    event MilestoneCompleted(
        uint256 indexed project_id,
        uint256 indexed milestone_id,
        uint256 amount_released
    );

    #[derive(Debug)]
    event MilestoneDisputed(
        uint256 indexed project_id,
        uint256 indexed milestone_id,
        address indexed challenger
    );
}