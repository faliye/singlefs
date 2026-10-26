## E17 Write Buffer Merge Limit — Run (2026-08-28)

[中文](17-writebuffer合并上限.md) · **English** · [日本語](17-ライトバッファマージ上限.ja.md)

**What it tests**: Single-thread limit of merge, and lock-free parallelism.

**Conclusions**:

1. **Merge itself holds up; it is not a bottleneck on the critical path.** Single-thread sort-merge-sweep achieves
   **29.9–31.8M entries/s (685–727 MiB/s, eight independent runs)**.
   ⚠️ Round 3 is the audit rerun on 2026-08-29 (`e17-merge-2026-08-29-repro.out`):
   single-thread 31.78M entries/s / 727.44 MiB/s, 16-thread speedup 6.26×, 32-thread 6.96×,
   positive control 16-thread 12.04×, 32-thread 15.93×. **Conclusions consistent with first two rounds; range upper bound thus raised from 31.6M to 31.8M.**
   ⚠️ Round 4 was run via `replay.sh` full run profile on 2026-08-31 (`e17-merge-2026-08-31-repro.out`):
   single-thread **31.61M entries/s / 723.49 MiB/s**, 16-thread 6.26×, 32-thread 7.08×,
   positive control 16-thread 12.26×, 32-thread 15.67×. **First four rounds all fell in the 30.9–31.8M interval.**
   ⚠️ Rounds 5 through 8 were run consecutively on the same day, 2026-09-16 (other sessions on the machine were running builds and experiments, `uptime` reading 2.1–2.5):
   single-thread **29.94 / 30.10 / 30.47 / 30.71M entries/s (685.18 / 688.92 / 697.45 / 702.93 MiB/s)**,
   three out of four rounds fell outside original lower bound ⇒ **interval modified to union of eight rounds: 29.9–31.8M**, assertion lower bound in `replay.sh` relaxed simultaneously from 30591000 to 29636000.
   ⚠️ **Parallel speedups for these four rounds along with positive controls were all depressed** (16 threads 5.49–6.12×, 32 threads 5.13–6.11×,
   positive control 16 threads 9.87–10.58×, 32 threads 11.51–13.42×)—joint depression points to machine contention from other workloads,
   while **apparatus does not record what is on the machine during runs**, so "drift vs contention" cannot be distinguished for the four rounds on 2026-09-16,
   placement [checks-owed.md](../checks-owed.md) C350 (timing experiments do not record machine load at run time).
   ⚠️ One fix on the same day: in `arm_control`, `t as u64 * 0x9E3779B97F4A7C15` overflow-panicked in release with `overflow-checks` enabled,
   causing E17 (Write Buffer Merge Limit) to **fail to run at all under the gate full run profile** (default profile does not run timing experiments, so had gone unnoticed);
   after changing to `wrapping_mul`, structure matches retained artifacts.
   ⇒ **Eight-round range for parallel speedup is 16 threads 5.49–6.38×, 32 threads 5.13–7.08×**,
   positive control 16 threads 9.87–12.26×. When citing, cite the range rather than single-round numbers;
   ⚠️ The gap between the first four rounds and last four rounds has no explanation outside workload contention—readings and rationale documented in Conclusion 1 paragraph for rounds 5 through 8.
   ⚠️ **However, deriving "encryption is the bottleneck" from this does not hold**, see comparison set correction at the end of E17 (Write Buffer Merge Limit). Converted to user write bandwidth: calculating 3 buffered entries produced per 4 KiB user write,
   using the eight-round lower bound 29.9M, a single thread sustains **40.8 GB/s**; even at 10 entries per 4 KiB, it sustains 12.2 GB/s.
   Meanwhile, [experiments.md](../experiments.md) E6 (Encryption Algorithm Selection) measured single-core encryption at 3.83 GB/s (with AES-NI) /
   0.257 GB/s (without AES-NI) (4 KiB unit, retested 2026-08-31).
   ⇒ **[decisions.md](../decisions.md) D8 (Core Index Structure) "sort-merge-sweep is essentially single-threaded" statement does not hold in its thread sense**—single-threaded merge is an order of magnitude faster than single-core encryption, and is parallelizable.
   ⚠️ **The I/O meaning of original sentence in D8 (Core Index Structure) still holds**, see end of E17 (Write Buffer Merge Limit).
2. **Leaf-partitioned lock-free parallelism is viable** (N=5 rounds taking median):

   | Threads | Merge Arm Speedup | Positive Control Speedup | Scatter Overhead Consumed |
   |---|---|---|---|
   | 16 | **6.38×** | 11.91× | 46% |
   | 32 | **6.96×** | 15.60× | 55% |

   **32 threads is higher than 16 threads in 5/5 rounds**; hyperthreading still provides gains on this workload (approx +9%),
   but with diminishing marginal returns—proportion consumed by the O(n) scatter pass increases from 46% to 55%.
   ⇒ **Leaf-partitioned scattering does not consume all gains, but represents the ceiling on parallelism.**

**The most valuable output of E17 (Write Buffer Merge Limit) is a format requirement, not numbers in Conclusion 1 and 2**:

> ⚠️ **Entries in write buffer must carry their own sequence number (seq).**

D8 (Core Index Structure) body explicitly states "flush sorts by key **discarding time order**", while deduplication rule is "last writer wins"—
**after sorting, the concept of "later" must be carried by entries themselves**. Under unstable sort, relative order of duplicate keys is undefined,
**rendering deduplication result non-deterministic**. The first version of E17 (Write Buffer Merge Limit) missed this, immediately causing discrepancies between both arms and ground truth.
⚠️ This is a **distinct requirement** from E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta) settled rule "entries must be idempotent full values":
idempotence governs "no error on repeated application", sequence number governs "deterministic overwrite ordering".

**Measurement basis**: Pure CPU/memory experiment, no devices, no VMs. 4 million entries / 4096 target leaves /
key space is 64× leaf count (guaranteeing duplicates to deduplicate) / 24-byte entries (key + seq + value) /
best of 5 rounds per level. Machine: AMD Ryzen 9 9950X, 16 cores 32 threads.
Code `scripts/experiment-apparatus/e17_merge.rs`, `cargo run --release --bin e17-merge`,
raw output `e17-merge-2026-08-28.out`.

**Non-triviality guarantee**: Both arms **share identical input**, and **output must match ground truth bit-for-bit**, any discrepancy invalidating entire round.
Ground truth independently computed via BTreeMap in arrival order, **sharing zero merge code with either arm**.
Three unit tests proven to turn red under targeted sabotage: removing `seq` from sort key immediately turns
`arms_agree_with_truth` and `partition_count_does_not_change_result` red.

⚠️ **E17 (Write Buffer Merge Limit) measures only the merge segment**: excluding tree insertion, excluding I/O, excluding allocation.

⚠️ **The derived claim that "encryption is the bottleneck" does not hold—comparison set omitted an item, and the omitted item is the largest.**
Actual flush tree insertion I/O was measured by [experiments.md](../experiments.md) E7 (Offline Index Harness):
`logstruct_wb` io/op = **1.5755**, with measurement basis "512 random keys landing across ~400 distinct leaves,
one read and one write per leaf" ⇒ **entries/leaf ≈ 1.28**.
Meanwhile, E17 (Write Buffer Merge Limit) packs 4 million entries into 4096 leaves, **entries/leaf = 976.6**—
**diluting tree insertion I/O by ~1000×**.

Converting per 4 KiB user write with 3 entries and E7 (Offline Index Harness) io/op yields three budget items:

| Item | Per 4 KiB User Write | Source |
|---|---|---|
| Merge CPU | **0.13 µs** | E17 (Write Buffer Merge Limit) |
| Encryption (single core, with AES-NI) | **1.07 µs** | 3.83 GB/s from E6 (Encryption Algorithm Selection) |
| **Flush tree insertion I/O** | **≈ 4.65 µs** | io/op 1.5755 from E7 (Offline Index Harness) |

⇒ **True critical path is flush tree insertion I/O**, approx 4.6× encryption, approx 36× merge.
And this cost is precisely incurred by **sorting itself** (E7 (Offline Index Harness) intervention control: shuffled group order −9.2%,
100% cached profile completely identical as negative control).

⚠️ **The sole unverified assumption here**: "entries/leaf ≈ 1" relies on "leaf count ≫ entries in a single flush".
If true buffer is large enough to cover the hot leaf set, E17 (Write Buffer Merge Limit) proportions become accurate, and the flush I/O item collapses.
**There is no data on this point.**

Therefore, E17 (Write Buffer Merge Limit) provides an **upper bound on the merge step**, not total flush cost, much less a verdict on critical path.

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D13 (Verification Route) settled item 2 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 15 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) settled item 7 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D8 (Core Index Structure) | Informs | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E17 (Write Buffer Merge Limit) are gathered in [experiments-history.md](../experiments-history.md).
