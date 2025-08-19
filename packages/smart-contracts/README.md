# AfroCreate Collective Smart Contracts

A comprehensive set of Rust-based smart contracts for Arbitrum Stylus that power AfroCreate Collective - a Web3 platform enabling African creators to crowdfund culturally-rich content projects through tokenized revenue sharing with ENS-powered identity management.

## 🌍 Vision

AfroCreate Collective empowers African creators to showcase their cultural heritage while building sustainable revenue streams through:

- **Cultural Authenticity**: Community-driven validation ensures genuine representation
- **ENS Identity**: Subdomains under afrocreate.eth for professional creator identities  
- **Revenue Sharing**: NFT-based revenue distribution to project backers
- **Multiple Funding Models**: All-or-nothing, flexible, and milestone-based crowdfunding
- **Governance**: Community-driven platform decisions and cultural fund allocation

## 📋 Contract Architecture

### Core Contracts

```
AfroCreateEcosystem/
├── platform/
│   ├── AfroCreatePlatform.rs     ✅ Main platform orchestrator  
│   └── PlatformGovernance.rs     🚧 DAO voting and governance
├── identity/
│   ├── ENSIntegration.rs         ✅ ENS resolver and management
│   └── CulturalIdentity.rs       ✅ Cultural background verification
├── projects/
│   ├── ProjectFactory.rs         ✅ Project creation and management
│   ├── ProjectFunding.rs         ✅ Crowdfunding mechanics
│   └── MilestoneManager.rs       🚧 Project milestone tracking
├── nfts/
│   ├── RevenueShareNFT.rs        🚧 Revenue-sharing NFT implementation
│   └── CulturalBadgeNFT.rs       🚧 Cultural authenticity badges
├── revenue/
│   ├── RevenueDistributor.rs     🚧 Revenue splitting and distribution
│   └── OracleManager.rs          🚧 Revenue data oracle integration
├── validation/
│   ├── CulturalValidator.rs      🚧 Cultural authenticity validation
│   └── CommunityValidator.rs     🚧 Community-driven validation
└── treasury/
    ├── PlatformTreasury.rs       🚧 Platform fee management
    └── ProjectEscrow.rs          🚧 Project fund escrow
```

**Legend**: ✅ Implemented | 🚧 Planned | ❌ Not Started

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- `cargo-stylus` CLI tool
- Node.js 18+ (for frontend integration)

```bash
# Install cargo-stylus
cargo install cargo-stylus

# Add WebAssembly target
rustup target add wasm32-unknown-unknown
```

### Building

```bash
# Build all contracts
cargo build --release --target wasm32-unknown-unknown

# Run contract checks
cargo run --bin check

# Export ABIs
cargo build --features export-abi
```

### Deployment

```bash
# Set your private key
export PRIVATE_KEY="your_private_key_here"

# Deploy to Arbitrum Sepolia (testnet)
./scripts/deploy.sh arbitrum-sepolia

# Initialize platform
./scripts/init-platform.sh <PLATFORM_ADDRESS>
```

## 🏗️ Core Features

### 1. Creator Registration & ENS Integration

Creators register with ENS subdomains under `afrocreate.eth`:

```rust
// Register creator with cultural background
platform.register_creator(
    "artist123".to_string(),        // ENS subdomain  
    "West African, Nigerian".to_string()  // Cultural background
)?;

// Results in: artist123.afrocreate.eth
```

**ENS Text Records**:
- `cultural.background`: Regional/ethnic background
- `cultural.languages`: Spoken languages  
- `cultural.traditions`: Cultural practices
- `platform.reputation`: Creator reputation score
- `platform.projects`: Number of projects created

### 2. Cultural Validation System

Community validators ensure authentic cultural representation:

```rust
// Submit project for cultural validation
validator.submit_validation(
    project_id,
    85,  // Score out of 100
    "Authentic Yoruba storytelling elements".to_string(),
    vec!["Yoruba folklore", "Traditional music"]
)?;
```

**Validation Criteria**:
- Cultural accuracy and authenticity
- Appropriate representation of traditions
- Language usage and context
- Community benefit and impact

### 3. Multi-Model Crowdfunding

Support for different funding approaches:

#### All-or-Nothing (Kickstarter-style)
```rust
funding.setup_project_funding(
    project_id,
    target_amount,
    deadline, 
    creator,
    FundingModel::AllOrNothing,
    vec![] // No milestones needed
)?;
```

#### Milestone-Based Funding
```rust
let milestones = vec![
    Milestone {
        title: "Pre-production".to_string(),
        funding_amount: U256::from(30_000), // 30% of target
        deadline: start + 30.days(),
    },
    // ... more milestones
];

funding.setup_project_funding(
    project_id,
    target_amount,
    deadline,
    creator, 
    FundingModel::MilestoneBased,
    milestones
)?;
```

### 4. Revenue-Sharing NFTs

Backers receive NFTs representing their revenue share:

```rust
// Automatically minted when funding projects
let nft_id = funding.fund_project(
    project_id,
    "backer123.afrocreate.eth".to_string()
)?;

// NFT represents proportional revenue share
let share_percentage = (contribution * 10000) / total_raised; // In basis points
```

**Revenue Sources**:
- Streaming royalties (Spotify, Apple Music)
- Licensing deals 
- Merchandise sales
- Live performance revenue
- Secondary content sales

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test platform::tests

# Test with coverage
cargo test --features coverage
```

### Test Categories

- **Unit Tests**: Individual contract functionality
- **Integration Tests**: Cross-contract interactions  
- **Gas Tests**: Optimization validation
- **Security Tests**: Vulnerability assessments

## 🔒 Security Features

### Access Control
```rust
// Role-based permissions
enum Role {
    Owner,       // Full platform control
    Admin,       // Administrative functions
    Validator,   // Cultural validation rights
    Creator,     // Project creation rights
}
```

### Reentrancy Protection
```rust
#[storage]
pub struct ReentrancyGuard {
    locked: StorageBool,
}

// Usage in sensitive functions
self.nonreentrant_guard()?;
// ... critical operations
self.unlock_guard();
```

### Input Validation
```rust
pub fn validate_ens_name(name: &str) -> Result<(), Error> {
    require_valid_input(name.len() >= 3, "Name too short")?;
    require_valid_input(name.len() <= 63, "Name too long")?;
    require_valid_input(
        name.chars().all(|c| c.is_alphanumeric() || c == '-'),
        "Invalid characters"
    )?;
    Ok(())
}
```

## ⛽ Gas Optimization

### Packed Storage
```rust
pub struct PackedProjectInfo {
    pub creator: Address,     // 20 bytes
    pub status: u8,          // 1 byte  
    pub category: u8,        // 1 byte
    pub validation_score: u8, // 1 byte (0-100)
    // Total: 23 bytes, fits in one storage slot
}
```

### Batch Operations
```rust
pub fn batch_mint_nfts(
    recipients: Vec<Address>,
    project_ids: Vec<U256>
) -> Result<Vec<U256>, Error> {
    // Process multiple operations in single transaction
}
```

### Event-Driven Architecture
```rust
// Emit events for off-chain indexing instead of expensive storage
evm::log(ProjectCreated {
    project_id,
    creator,
    cultural_category,
    funding_target,
});
```

## 🌐 Network Deployment

### Arbitrum Stylus Advantages

- **Native Rust**: Write contracts in Rust with full safety guarantees
- **EVM Compatibility**: Seamless interaction with existing Ethereum tools
- **Gas Efficiency**: Lower costs than Ethereum mainnet
- **High Performance**: Fast transaction processing

### Supported Networks

| Network | RPC URL | Chain ID |
|---------|---------|----------|
| Arbitrum One | https://arb1.arbitrum.io/rpc | 42161 |
| Arbitrum Sepolia | https://sepolia-rollup.arbitrum.io/rpc | 421614 |

## 🗳️ Governance System

Community-driven platform decisions through:

### Proposal Types
- Platform fee adjustments
- New cultural category additions
- Validator appointment/removal
- Cultural fund allocations
- Platform upgrades

### Voting Power
- **Creators**: Based on successful projects and reputation
- **Backers**: Based on total funding contributed  
- **Validators**: Based on validation accuracy and community trust

```rust
// Create governance proposal
governance.create_proposal(
    "Add 'Digital Art' Category".to_string(),
    "Proposal to add digital art as supported category...".to_string(),
    execution_data // Encoded function call
)?;
```

## 📊 Platform Metrics

Real-time tracking of platform health:

```rust
pub fn platform_stats(&self) -> (U256, U256, U256, U256) {
    (
        self.total_funding_raised.get(),
        self.successful_projects.get(), 
        self.active_creators.get(),
        self.project_count.get(),
    )
}
```

**Key Metrics**:
- Total funding raised across all projects
- Number of successful projects
- Active creator count
- Cultural validation accuracy
- Revenue distribution efficiency

## 🎯 Development Roadmap

### Phase 1: Core Platform (Completed ✅)
- [x] Platform registry and creator management
- [x] ENS integration with subdomain support
- [x] Project creation and funding mechanisms
- [x] Cultural identity verification
- [x] Basic deployment infrastructure

### Phase 2: Advanced Features (In Progress 🚧)
- [ ] Revenue-sharing NFT implementation
- [ ] Cultural validation system with scoring
- [ ] Multiple revenue source integration
- [ ] Platform governance and voting
- [ ] Advanced milestone management

### Phase 3: Ecosystem Growth (Planned 📋)
- [ ] Oracle integration for revenue verification  
- [ ] Superfluid streaming for real-time revenue distribution
- [ ] Mobile app integration
- [ ] Creator analytics dashboard
- [ ] Community features and social elements

### Phase 4: Scale & Sustainability (Future 🔮)
- [ ] Multi-chain deployment
- [ ] AI-powered cultural validation assistance
- [ ] Creator education and resources platform
- [ ] Partnership integrations (streaming platforms, galleries)
- [ ] Decentralized governance transition

## 🤝 Contributing

We welcome contributions from developers passionate about supporting African creators:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Test** your changes thoroughly
4. **Commit** with clear messages (`git commit -m 'Add amazing feature'`)
5. **Push** to the branch (`git push origin feature/amazing-feature`)
6. **Open** a Pull Request

### Contribution Guidelines

- Follow Rust best practices and idioms
- Add comprehensive tests for new features  
- Update documentation for public APIs
- Ensure gas optimization for new functions
- Respect cultural sensitivity in all implementations

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Arbitrum Stylus Team** for the innovative Rust-on-chain platform
- **ENS Team** for the decentralized naming infrastructure
- **African Creator Community** for inspiration and cultural guidance
- **Web3 Builders** contributing to decentralized creator economies

---

**Built with ❤️ for African creators by the AfroCreate Collective**

*Empowering cultural creativity through Web3 technology*