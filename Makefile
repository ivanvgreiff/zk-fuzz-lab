# zk-fuzz-lab Makefile
#
# Phase 0: Basic smoke test to verify scaffold
# Phase 1: Walking skeleton for differential testing

.PHONY: smoke help clean run build test

# Default target
help:
	@echo "zk-fuzz-lab - ZKVM Differential Fuzzing Framework"
	@echo ""
	@echo "Available targets:"
	@echo "  make smoke          - Verify repository scaffold (Phase 0)"
	@echo "  make build          - Build all workspace members"
	@echo "  make test           - Run all tests"
	@echo "  make run CORE=<core> INPUT=<input> - Run differential test"
	@echo "  make clean          - Remove artifacts and build outputs"
	@echo "  make help           - Show this help message"
	@echo ""
	@echo "Example:"
	@echo "  make run CORE=guest/cores/fib INPUT=inputs/fib_24.json"
	@echo ""
	@echo "Phase 2+ targets (coming soon):"
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

# Phase 1: Build all workspace members
build:
	@echo "🔨 Building workspace..."
	@cargo build --release
	@echo "✅ Build complete"

# Phase 1: Run all tests
test:
	@echo "🧪 Running tests..."
	@cargo test
	@echo "✅ Tests complete"

# Phase 1: Run differential test
# Usage: make run CORE=guest/cores/fib INPUT=inputs/fib_24.json
run:
ifndef CORE
	$(error CORE is not set. Usage: make run CORE=guest/cores/fib INPUT=inputs/fib_24.json)
endif
ifndef INPUT
	$(error INPUT is not set. Usage: make run CORE=guest/cores/fib INPUT=inputs/fib_24.json)
endif
	@cargo run --release --bin harness -- run --core $(CORE) --input $(INPUT)

# Phase 3: Run batch tests on all seed cores
# Usage: make batch
batch:
	@echo "🔁 Running batch tests on all seed cores..."
	@echo ""
	@make run CORE=guest/cores/fib INPUT=inputs/fib_24.json
	@echo ""
	@make run CORE=guest/cores/io_echo INPUT=inputs/io_echo_empty.json
	@echo ""
	@make run CORE=guest/cores/io_echo INPUT=inputs/io_echo_1kb.json
	@echo ""
	@make run CORE=guest/cores/arithmetic INPUT=inputs/arithmetic_add_normal.json
	@echo ""
	@make run CORE=guest/cores/arithmetic INPUT=inputs/arithmetic_add_overflow.json
	@echo ""
	@make run CORE=guest/cores/arithmetic INPUT=inputs/arithmetic_div_by_zero.json
	@echo ""
	@make run CORE=guest/cores/simple_struct INPUT=inputs/simple_struct_normal.json
	@echo ""
	@echo "✅ Batch tests complete!"
	@echo "📊 Summary available in artifacts/summary.csv"

# Clean up generated artifacts
clean:
	@echo "🧹 Cleaning artifacts..."
	@rm -rf artifacts/*
	@rm -rf target/
	@echo "✅ Clean complete"

