## E202 GPU vs CPU GC Comparison Groups — Pre-run Registration Written, Apparatus Not Run

[中文](202-GPU与CPU两种模式GC对照组.md) · **English** · [日本語](202-GPUとCPUの2方式GC対照群.ja.md)

Pre-run registration `research/prompts/e202-preregistration.md`. Answers question list `research/prompts/m4-gpugc-compare-questions.md`: compares GC wall-clock, host CPU time used and saved, foreground fsync latency distribution, and saved CPU time scaling with disk capacity between `background_compute=cpu` and `background_compute=gpu`.

In-tree apparatus `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` (lives in the background-gpu-tests crate, the test crate of the product GPU crate). **This page records that the pre-run registration and apparatus skeleton are written; no artifacts have been run.**

### Current Status

- Pre-run registration written (2026-11-03), apparatus skeleton created, no artifacts run yet.
- The geometry check of segment one was started once, did not finish and left no artifact: building the starting pools is too slow to fit under the four-hour cap of the task service at the measured speed. Measured: on the 2288MiB tier the sequential half took about 34 minutes and the interleaved half did 95 steps in about 60 seconds out of 18304; on the 8GiB tier the sequential half reached 18292 steps out of 65536 at 2100 seconds; with three tiers built at once the resident memory was 9.4 GiB at 2100 seconds. The user settled not to run it this time and to run it after the pool-building speed-up is done.
- Capacity reading uses fixed-occupancy reading (50% live data in unit area).
- Apparatus constraints: GPU mode per-card cap 1 GiB, filling cards in order; runs with single card and multiple cards; a data set counts only when judging clauses match CPU reference item by item.
- How many cards the multi-card arm uses is set by `lockstep --cards <count>` (0 or 1, or more than the task server handed over, exits 2; without it the arm uses every card the task server handed over). The product has two kinds of card-recording lines: `name=every_card_arm_cards` records `cards_named`, how it was set (`named_by=cards_option` or `every_handed_over_card`), and the PCI address and model of each card; `name=one_card_arm_card` records the PCI address and model of the one card of the single-named-card arm, and is printed by both the lockstep run and the concurrent run. When resuming, if any of these differs from the line of the same kind already in the product being resumed, or the product has result lines produced by this arm (`name=run`, `name=concurrent_run`) but no line of this kind, it prints `name=stopped clause=cards_differ_from_the_product_being_resumed` and exits 2; a run that produced no result line at all does not block later resumes.

### Repro

```bash
# In-repo apparatus compilation and execution belongs to the main agent; scheduled via task-server
task-server/task submit -- cargo run --release -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --bin e202_gpu_cpu_gc_comparison
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
| D26 (Background Compaction and Placement Reclamation) settled item 13 | Preparation | 2026-11-18 Unaffected: this change only rewrites the multi-card item to follow amendment 12 of the preregistration (the two kinds of card-recording lines, the name of the stop clause that checks cards on resume); the criteria, arms and conditions are unchanged and the product has still never been run. The change of 2026-11-14 moved the apparatus into its test package, changed how it is started, and changed how the number of cards for the multi-card arm is set; the criteria, arms and scope are unchanged and no artifact has been run, so it is still pre-run registration written, apparatus not run |

## Historical Versions

### 2026-11-13

- The geometry check of segment one was started once (task 16507); building the starting pools was too slow and the memory cap was hit three times with a restart from scratch each time, so the main agent cancelled it; the user settled not to run it this time and to wait for the pool-building speed-up. No artifact.
- The apparatus moved whole from the product GPU crate into its test crate `crates/singlefs-background-gpu-tests/src/bin/`, and is now started with `-p singlefs-background-gpu-tests --features tests-on-a-real-gpu`; the card count of the multi-card arm is now set by `lockstep --cards <count>`, and the product gains a `name=every_card_arm_cards` line (the user settlement, verbatim, is in `records/2026-11-11-GPU两段与按需领卡核查.md`). Criteria, arms and measurement definitions are unchanged; the product has never been run.

### 2026-11-03

- File created, pre-run registration and apparatus skeleton recorded for E202 (GPU vs CPU GC Comparison Groups).
