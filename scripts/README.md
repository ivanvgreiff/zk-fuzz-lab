# Build Scripts

Reusable build scripts to avoid PowerShell → WSL PATH issues.

## Problem

Running SP1 builds via inline WSL commands fails:
```bash
# ❌ FAILS:
wsl bash -c 'export PATH=$HOME/.sp1/bin:$PATH && cd ... && cargo prove build'
```

**Why**: PowerShell expands `$PATH` before WSL sees it, and Windows PATH contains spaces (`Program Files`) which breaks bash parsing.

## Solution

Use bash script files in the project directory:
```bash
# ✅ WORKS:
wsl bash /mnt/c/Users/ivan/zk-fuzz-lab/scripts/build_sp1_guests.sh all
```

## Available Scripts

### `build_sp1_guests.sh`

Build one or more SP1 guest programs.

**Usage**:
```bash
# Build all guests
wsl bash scripts/build_sp1_guests.sh all

# Build specific guests
wsl bash scripts/build_sp1_guests.sh fib_guest io_echo_guest

# Build with shortened names (automatically adds _guest suffix)
wsl bash scripts/build_sp1_guests.sh fib io_echo arithmetic

# List available guests
wsl bash scripts/build_sp1_guests.sh
```

**Features**:
- ✅ Automatically sets PATH for cargo-prove
- ✅ Color-coded output (green = success, red = failure)
- ✅ Builds multiple guests in sequence
- ✅ Reports summary at the end
- ✅ Exits with error code if any build fails

**Example Output**:
```
📦 Building fib_guest...
[sp1]      Finished `release` profile [optimized] target(s) in 4.52s
✅ fib_guest built successfully

📦 Building io_echo_guest...
[sp1]      Finished `release` profile [optimized] target(s) in 1m 11s
✅ io_echo_guest built successfully

=========================================
✅ All guests built successfully! (2 total)
```

## Why This Works

1. **Script file** = pure bash (no PowerShell interference)
2. **PATH set inside bash** = no expansion issues  
3. **No escaping needed** = clean, readable
4. **Reusable** = works for all future phases

## Future Enhancements

Potential additions:
- `run_batch_tests.sh` - Run `make batch` with proper PATH
- `clean_artifacts.sh` - Clean build artifacts
- `rebuild_all.sh` - Clean + rebuild everything

