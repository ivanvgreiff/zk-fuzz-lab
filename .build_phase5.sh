#!/usr/bin/env bash
set -e

cd /mnt/c/Users/ivan/zk-fuzz-lab

# Set PATH for cargo
export PATH=$HOME/.cargo/bin:$HOME/.sp1/bin:$PATH

echo "🔨 Building Phase 5 components..."
echo ""

# Build source mutator
echo "📦 Building source-mutator..."
cargo build --release -p source-mutator

# Build harness
echo "📦 Building harness..."
cargo build --release -p harness

echo ""
echo "✅ Phase 5 build complete!"

