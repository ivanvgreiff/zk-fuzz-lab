# Continuous Integration

**Phase 12**: CI pipeline for automated testing and fuzzing campaigns.

## Purpose

- Run smoke tests on every PR
- Deep fuzzing campaigns nightly
- Archive artifacts for triage
- Generate summary reports

## CI Structure

### PR Checks (Fast)
```yaml
# .github/workflows/pr.yml
- Build all components
- Run smoke test
- Execute ONE tiny seed (fib) on native + SP1
- Verify artifacts are generated
```

Target: < 5 minutes

### Nightly Runs (Deep)
```yaml
# .github/workflows/nightly.yml
- Rotate through all seeds in guest/cores/
- Run small RustSmith batch (100 programs)
- Archive artifacts/ as build artifacts
- Generate HTML/Markdown summary
```

Target: 1-2 hours

### Long-Run Queue (A2/A3)
```yaml
# .github/workflows/long-run.yml (manual trigger)
- Full A2 campaign (RISC-V fuzzing)
- A3 mutation campaigns (proof tampering)
- Extended RustSmith generation
```

Target: 8+ hours

## Artifacts

### PR
- `smoke-test-results.txt`

### Nightly
- `artifacts.zip` (all runs)
- `summary.html` (top divergences, stats)
- `summary.csv` (for analysis)

### Long-Run
- Full `artifacts/` directory
- Detailed reports in `artifacts/reports/`

## Integration with Validation

Phase 7 known-bug validation runs can be:
- Part of nightly CI
- Or manual trigger before milestone reviews

## Phase Schedule

- **Phase 0-11**: Not implemented (stub directory)
- **Phase 12**: Initial CI (PR smoke + nightly batch)
- **Phase 13+**: Refinement based on false positive rates

## Notifications

- PR failures → block merge
- Nightly divergences → post to Slack channel
- Long-run completion → email with summary link

