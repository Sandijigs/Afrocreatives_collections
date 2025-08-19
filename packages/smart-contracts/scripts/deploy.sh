#!/bin/bash

# AfroCreate Collective - Deployment Script for Arbitrum Stylus
set -e

echo "🚀 Deploying AfroCreate Collective to Arbitrum Stylus"
echo "=================================================="

# Check if required tools are installed
command -v cargo >/dev/null 2>&1 || { echo "❌ Cargo is required but not installed. Aborting." >&2; exit 1; }
command -v cargo-stylus >/dev/null 2>&1 || { echo "❌ cargo-stylus is required. Install with: cargo install cargo-stylus" >&2; exit 1; }

# Build the contracts
echo "📦 Building contracts..."
cargo build --release --target wasm32-unknown-unknown

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo "✅ Contracts built successfully"
else
    echo "❌ Build failed"
    exit 1
fi

# Deploy to Arbitrum Stylus (testnet by default)
NETWORK=${1:-"arbitrum-sepolia"}
PRIVATE_KEY=${PRIVATE_KEY:-""}

if [ -z "$PRIVATE_KEY" ]; then
    echo "❌ PRIVATE_KEY environment variable is required"
    echo "   Set it with: export PRIVATE_KEY=your_private_key_here"
    exit 1
fi

echo "🌐 Deploying to $NETWORK..."

# Deploy platform contract
echo "📋 Deploying AfroCreatePlatform..."
PLATFORM_ADDRESS=$(cargo stylus deploy \
    --wasm-file target/wasm32-unknown-unknown/release/afrocreate_contracts.wasm \
    --network $NETWORK \
    --private-key $PRIVATE_KEY \
    --estimate-gas)

if [ $? -eq 0 ]; then
    echo "✅ Platform deployed at: $PLATFORM_ADDRESS"
else
    echo "❌ Platform deployment failed"
    exit 1
fi

# Export deployment info
echo "📄 Generating deployment info..."
cat > deployment-info.json << EOF
{
    "network": "$NETWORK",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "contracts": {
        "AfroCreatePlatform": "$PLATFORM_ADDRESS"
    },
    "deployment_notes": {
        "description": "AfroCreate Collective - Web3 platform for African creators",
        "features": [
            "ENS integration with afrocreate.eth subdomains",
            "Cultural validation system",
            "Revenue-sharing NFTs",
            "Multiple crowdfunding models",
            "Comprehensive governance system"
        ],
        "next_steps": [
            "Initialize platform with ENS registry address",
            "Set up cultural validators",
            "Configure platform fees and settings",
            "Deploy additional contracts (funding, NFT, governance)"
        ]
    }
}
EOF

echo "🎉 Deployment completed successfully!"
echo "📋 Contract Address: $PLATFORM_ADDRESS"
echo "📄 Deployment info saved to: deployment-info.json"
echo ""
echo "🔧 Next steps:"
echo "1. Initialize the platform with ENS registry"
echo "2. Set up cultural validators"
echo "3. Configure platform settings"
echo "4. Deploy supporting contracts"
echo ""
echo "📚 Documentation: https://github.com/afrocreate/contracts"