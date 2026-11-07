## E202 GPU vs CPU GC Comparison Groups — Pre-run Registration Written, Apparatus Not Run

[中文](202-GPU与CPU两种模式GC对照组.md) · **English** · [日本語](202-GPUとCPUの2方式GC対照群.ja.md)

Pre-run registration `research/prompts/e202-preregistration.md`. Answers question list `research/prompts/m4-gpugc-compare-questions.md`: compares GC wall-clock, host CPU time used and saved, foreground fsync latency distribution, and saved CPU time scaling with disk capacity between `background_compute=cpu` and `background_compute=gpu`.

In-tree apparatus `crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs` (lives in background-gpu crate). **This page records that the pre-run registration and apparatus skeleton are written; no artifacts have been run.**

### Current Status

- Pre-run registration written (2026-11-03), apparatus skeleton created, no artifacts run yet.
- Capacity reading uses fixed-occupancy reading (50% live data in unit area).
- Apparatus constraints: GPU mode per-card cap 1 GiB, filling cards in order; runs with single card and multiple cards; a data set counts only when judging clauses match CPU reference item by item.

### Repro

```bash
# In-repo apparatus compilation and execution belongs to the main agent; scheduled via task-server
task-server/task submit -- cargo run --release -p singlefs-background-gpu --features gpu --bin e202_gpu_cpu_gc_comparison
```

### Measurement Basis

- Raw output not preserved: experiment not run yet.
- Single compaction: all compaction work done by the GC thread from the first call to background entry until the final call returns in a single run.
- GC wall-clock: sum of differences reading `std::time::Instant` before and after each background entry call on the GC thread.
- GC host CPU time: difference reading `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` before and after each GC call minus foreground thread increment in the same window.
- Saved CPU time: process-metric reading of CPU arm minus process-metric reading of GPU arm in the same capacity tier and repetition.
- Repetition count: 5 repeats per cell per arm, reporting median, min, max, and median absolute deviation.

### Criteria and Failure Clauses

- Criteria: CPU mode minus GPU mode difference is measured across capacity levels, with positive/negative values and dispersion; positive controls show less saved CPU time than regular controls.
- Failure clause: triggers if GPU mode saved host CPU time is constantly <= 0 across all capacity tiers, or foreground fsync latency degrades beyond limit.

### Decision Look-back

| Decision item | Relationship | Look-back |
|---|---|---|
| D26 (Background Compaction and Placement Reclamation) settled item 13 | Preparation | 2026-11-03 Unaffected: not run, pre-run registration written |

## Historical Versions

### 2026-11-03

- File created, pre-run registration and apparatus skeleton recorded for E202 (GPU vs CPU GC Comparison Groups).
