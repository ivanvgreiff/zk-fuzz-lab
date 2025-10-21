#!/bin/bash
# Reusable script to build SP1 guests
# Usage: 
#   ./scripts/build_sp1_guests.sh [guest1] [guest2] ...
#   ./scripts/build_sp1_guests.sh all  # Build all guests

set -e

# Ensure PATH includes cargo and cargo-prove
export PATH=/root/.cargo/bin:/root/.sp1/bin:$PATH

# Project root
PROJECT_ROOT="/mnt/c/Users/ivan/zk-fuzz-lab"
ADAPTERS_DIR="$PROJECT_ROOT/adapters/sp1_guest"

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to build a single guest
build_guest() {
    local guest_name=$1
    local guest_dir="$ADAPTERS_DIR/${guest_name}"
    
    if [ ! -d "$guest_dir" ]; then
        echo -e "${RED}❌ Error: Guest directory not found: $guest_dir${NC}"
        return 1
    fi
    
    echo -e "${BLUE}📦 Building ${guest_name}...${NC}"
    cd "$guest_dir"
    
    if cargo prove build; then
        echo -e "${GREEN}✅ ${guest_name} built successfully${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}❌ Failed to build ${guest_name}${NC}"
        return 1
    fi
}

# Get list of all SP1 guests
get_all_guests() {
    cd "$ADAPTERS_DIR"
    for dir in */; do
        # Remove trailing slash
        guest_name="${dir%/}"
        # Skip if not a directory or if it's a hidden directory
        if [ -d "$guest_name" ] && [[ ! "$guest_name" =~ ^\. ]]; then
            echo "$guest_name"
        fi
    done
}

# Main logic
if [ $# -eq 0 ]; then
    echo "Usage: $0 [guest1] [guest2] ... OR $0 all"
    echo ""
    echo "Available guests:"
    get_all_guests | sed 's/^/  - /'
    exit 1
fi

# Check if building all guests
if [ "$1" = "all" ]; then
    echo -e "${BLUE}🔨 Building all SP1 guests...${NC}"
    echo ""
    
    failed_guests=()
    success_count=0
    
    for guest in $(get_all_guests); do
        if build_guest "$guest"; then
            ((success_count++))
        else
            failed_guests+=("$guest")
        fi
    done
    
    echo ""
    echo "========================================="
    if [ ${#failed_guests[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ All guests built successfully! ($success_count total)${NC}"
    else
        echo -e "${RED}❌ Some guests failed to build:${NC}"
        for guest in "${failed_guests[@]}"; do
            echo -e "${RED}  - $guest${NC}"
        done
        exit 1
    fi
else
    # Build specific guests
    failed_guests=()
    success_count=0
    
    for guest in "$@"; do
        # Remove trailing _guest if present for flexibility
        guest_name="${guest%_guest}_guest"
        # But if the original name already has _guest, don't double it
        if [[ "$guest" == *"_guest" ]]; then
            guest_name="$guest"
        fi
        
        if build_guest "$guest_name"; then
            ((success_count++))
        else
            failed_guests+=("$guest_name")
        fi
    done
    
    echo ""
    echo "========================================="
    if [ ${#failed_guests[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ All specified guests built successfully! ($success_count/$#)${NC}"
    else
        echo -e "${RED}❌ Some guests failed to build:${NC}"
        for guest in "${failed_guests[@]}"; do
            echo -e "${RED}  - $guest${NC}"
        done
        exit 1
    fi
fi

