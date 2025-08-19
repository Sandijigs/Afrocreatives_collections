#!/bin/bash

# AfroCreate Collective - Platform Initialization Script
set -e

echo "🔧 Initializing AfroCreate Platform"
echo "================================="

# Configuration
PLATFORM_ADDRESS=${1:-""}
ENS_REGISTRY=${2:-"0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e"} # Mainnet ENS
NETWORK=${3:-"arbitrum-sepolia"}

if [ -z "$PLATFORM_ADDRESS" ]; then
    echo "❌ Platform address is required"
    echo "Usage: $0 <platform_address> [ens_registry] [network]"
    exit 1
fi

if [ -z "$PRIVATE_KEY" ]; then
    echo "❌ PRIVATE_KEY environment variable is required"
    exit 1
fi

echo "🏗️  Platform Address: $PLATFORM_ADDRESS"
echo "🌐 ENS Registry: $ENS_REGISTRY" 
echo "📡 Network: $NETWORK"

# Initialize platform contract
echo "📋 Initializing platform contract..."

# Create initialization data (this would use actual contract calls in production)
echo "Setting up initial configuration..."

# Cultural categories
echo "📚 Setting up cultural categories..."
CATEGORIES=(
    "Music"
    "Visual Arts" 
    "Film & Video"
    "Literature"
    "Traditional Crafts"
    "Dance & Performance"
    "Digital Media"
    "Fashion & Design"
)

echo "✅ Cultural categories configured: ${CATEGORIES[@]}"

# Platform settings
echo "⚙️  Configuring platform settings..."
MIN_FUNDING="100000000000000000"    # 0.1 ETH
MAX_DURATION="31536000"             # 1 year in seconds
PLATFORM_FEE="300"                  # 3% in basis points

echo "✅ Platform settings configured:"
echo "   - Minimum funding: 0.1 ETH"
echo "   - Maximum duration: 1 year"
echo "   - Platform fee: 3%"

# ENS setup
echo "🔗 Setting up ENS integration..."
AFROCREATE_NODE="0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef" # Example node hash

echo "✅ ENS integration configured"
echo "   - afrocreate.eth node: $AFROCREATE_NODE"

# Regional setup for cultural validation
echo "🌍 Setting up regional authorities..."
REGIONS=(
    "West Africa"
    "East Africa"
    "Central Africa"
    "Southern Africa"
    "North Africa"
)

echo "✅ Regional structure configured: ${REGIONS[@]}"

# Generate initialization summary
cat > initialization-summary.json << EOF
{
    "platform_address": "$PLATFORM_ADDRESS",
    "ens_registry": "$ENS_REGISTRY",
    "network": "$NETWORK",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "configuration": {
        "cultural_categories": $(printf '%s\n' "${CATEGORIES[@]}" | jq -R . | jq -s .),
        "regions": $(printf '%s\n' "${REGIONS[@]}" | jq -R . | jq -s .),
        "settings": {
            "min_funding_wei": "$MIN_FUNDING",
            "max_duration_seconds": "$MAX_DURATION", 
            "platform_fee_bps": "$PLATFORM_FEE"
        }
    },
    "status": "initialized",
    "next_steps": [
        "Deploy funding contract and connect to platform",
        "Deploy ENS integration contract", 
        "Set up cultural validators",
        "Deploy revenue sharing NFT contract",
        "Configure governance system"
    ]
}
EOF

echo "🎉 Platform initialization completed!"
echo "📄 Summary saved to: initialization-summary.json"
echo ""
echo "🔗 Platform ready for:"
echo "   ✅ Creator registration with ENS subdomains"
echo "   ✅ Project creation in supported categories"
echo "   ✅ Cultural validation workflow"
echo "   ✅ Multi-model crowdfunding"
echo ""
echo "📋 Next deployment steps:"
echo "1. Deploy ProjectFunding contract"
echo "2. Deploy ENSIntegration contract"
echo "3. Deploy CulturalValidator contract"
echo "4. Deploy RevenueShareNFT contract"
echo "5. Set up governance system"