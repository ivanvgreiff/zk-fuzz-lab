# zk-fuzz-lab Makefile
#
# Phase 0: Basic smoke test to verify scaffold
# Later phases will add more targets

.PHONY: smoke help clean

# Default target
help:
	@echo "zk-fuzz-lab - ZKVM Differential Fuzzing Framework"
	@echo ""
	@echo "Available targets:"
	@echo "  make smoke    - Verify repository scaffold (Phase 0)"
	@echo "  make help     - Show this help message"
	@echo "  make clean    - Remove artifacts and build outputs"
	@echo ""
	@echo "Phase 1+ targets (coming soon):"
	@echo "  make run A=<guest>  - Run differential test on a guest program"
	@echo "  make batch          - Run all seed programs"
	@echo "  make fuzz           - Start fuzzing campaign"

# Phase 0 deliverable: verify scaffold is set up correctly
smoke:
	@echo "🔍 Checking repository structure..."
	@test -d guest/cores || (echo "❌ guest/cores/ missing" && exit 1)
	@test -d generators/rustgen || (echo "❌ generators/rustgen/ missing" && exit 1)
	@test -d generators/rvgen || (echo "❌ generators/rvgen/ missing" && exit 1)
	@test -d mutators/source_mut || (echo "❌ mutators/source_mut/ missing" && exit 1)
	@test -d mutators/trace_mut || (echo "❌ mutators/trace_mut/ missing" && exit 1)
	@test -d adapters/sp1_guest || (echo "❌ adapters/sp1_guest/ missing" && exit 1)
	@test -d runners/native || (echo "❌ runners/native/ missing" && exit 1)
	@test -d runners/sp1 || (echo "❌ runners/sp1/ missing" && exit 1)
	@test -d oracles/rust_eq || (echo "❌ oracles/rust_eq/ missing" && exit 1)
	@test -d oracles/riscv_eq || (echo "❌ oracles/riscv_eq/ missing" && exit 1)
	@test -d oracles/spec_violations || (echo "❌ oracles/spec_violations/ missing" && exit 1)
	@test -d harness || (echo "❌ harness/ missing" && exit 1)
	@test -d inputs || (echo "❌ inputs/ missing" && exit 1)
	@test -d artifacts || (echo "❌ artifacts/ missing" && exit 1)
	@test -d ci || (echo "❌ ci/ missing" && exit 1)
	@echo "✅ All directories present"
	@echo ""
	@echo "✨ Skeleton OK"

# Clean up generated artifacts
clean:
	@echo "🧹 Cleaning artifacts..."
	@rm -rf artifacts/*
	@rm -rf target/
	@echo "✅ Clean complete"

