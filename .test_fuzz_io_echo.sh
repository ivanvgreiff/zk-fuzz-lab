#!/usr/bin/env bash
set -e

cd /mnt/c/Users/ivan/zk-fuzz-lab

# Set PATH for cargo
export PATH=$HOME/.cargo/bin:$HOME/.sp1/bin:$PATH

echo "🧪 Testing Phase 5 fuzzing with io_echo..."
echo ""

# Run fuzzing on just io_echo
make fuzz CORE=io_echo

echo ""
echo "✅ Fuzzing test complete!"
echo ""
echo "📊 Checking CSV for mutation entries..."
tail -5 artifacts/summary.csv | head -3

