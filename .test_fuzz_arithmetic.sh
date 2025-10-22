#!/usr/bin/env bash
set -e

cd /mnt/c/Users/ivan/zk-fuzz-lab

# Set PATH for cargo
export PATH=$HOME/.cargo/bin:$HOME/.sp1/bin:$PATH

echo "🧪 Testing Phase 5 fuzzing with arithmetic..."
echo ""

# Run fuzzing on arithmetic (will build guest)
make fuzz CORE=arithmetic

echo ""
echo "✅ Arithmetic fuzzing test complete!"

