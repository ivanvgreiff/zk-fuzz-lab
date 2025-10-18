# Windows PowerShell version of 'make smoke'
# Verifies repository scaffold for Phase 0

Write-Host "Checking repository structure..." -ForegroundColor Cyan

$directories = @(
    "guest/cores",
    "generators/rustgen",
    "generators/rvgen",
    "mutators/source_mut",
    "mutators/trace_mut",
    "adapters/sp1_guest",
    "runners/native",
    "runners/sp1",
    "oracles/rust_eq",
    "oracles/riscv_eq",
    "oracles/spec_violations",
    "harness",
    "inputs",
    "artifacts",
    "ci"
)

$allPresent = $true

foreach ($dir in $directories) {
    if (-not (Test-Path $dir)) {
        Write-Host "[X] $dir/ missing" -ForegroundColor Red
        $allPresent = $false
    }
}

if ($allPresent) {
    Write-Host "[OK] All directories present" -ForegroundColor Green
    Write-Host ""
    Write-Host "Skeleton OK" -ForegroundColor Green
    exit 0
} else {
    Write-Host ""
    Write-Host "[ERROR] Scaffold incomplete" -ForegroundColor Red
    exit 1
}

