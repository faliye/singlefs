## E21 Net Benefit of GPU Offloading —— All Three Segments Measured (2026-08-28); Pure Scan Cell Judged Negative Benefit, Three Candidate Cells Suspended and Unscheduled

[中文](21-GPU卸载的净收益.md) · **English** · [日本語](21-GPUオフロードの純収支.ja.md)

### Two Completed Segments, and What They Alone Can Eliminate

**Criterion 1 is `CPU 16-thread elapsed time ÷ (transfer + compute + transfer back) elapsed time`, eliminating that cell if less than 1.**
⇒ **Even if kernel execution time were zero**, the GPU path must still pay for transfer. That segment constitutes the **floor** on the GPU side,
while CPU memory bandwidth forms the **ceiling** on the CPU side. Comparing the two allows eliminating parts without waiting for kernels to be written.

| Path | Throughput | Nature |
|---|---|---|
| CPU scan (XOR fold, 2 GiB, best of 5 rounds) | **64.91 GB/s** | Memory bandwidth wall, CPU-side **ceiling** |
| GPU one-way (pinned H2D, ≥64 MiB) | 56.69 GB/s | Transfer **floor** |
| GPU round-trip (H2D + D2H serial) | **28.5 GB/s** | Transfer **floor** |

**⇒ Criterion: GPU can only win when "actual throughput on the CPU side is below GPU transfer floor".**

| Scenario | What is Returned | GPU Floor | How Slow CPU Must Be for GPU to Have a Chance |
|---|---|---|---|
| Full-disk scrub | Returns only a single conclusion | 56.7 GB/s | < 56.7 GB/s |
| Batch compression | Compressed data (smaller than input) | 40.6 GB/s (compression ratio 0.5) / 47.6 (0.25) | < that number |
| **EC reconstruction** | **All reconstructed data** | **28.5 GB/s** | **< 28.5 GB/s** |

**Substituting multi-core saturation measurements from [experiments.md](../experiments.md) E6 (Encryption Algorithm Selection)**: with AES-NI,
**8 threads saturate at 17.4 GB/s** (16 / 32 threads at 16.3 / 16.5, E6 (Encryption Algorithm Selection) 2026-08-28 multi-core tier)
⇒ **Lower than both GPU floors (round-trip 28.5, one-way 50.3), GPU wins**;
without AES-NI, multi-core was unmeasured, single-core is 0.257 GB/s (244.74 MiB/s, remeasured 2026-08-31), saturation value will only be lower
⇒ **Far below 28.5 GB/s, GPU wins by a wide margin**.

⚠️ **What is substituted must be empirical multi-core saturation measurements, never "single-core × core count".** For the same quantity, linear extrapolation predicts ≈61 GB/s
(3.83 GB/s × 16), whereas actual saturation is **17.4 GB/s**—a nearly 4× discrepancy **in the opposite direction**: extrapolation suggests "with AES-NI ⇒ GPU loses",
whereas measurement shows GPU wins.

⇒ **Criterion is multi-core saturation throughput on the CPU side**, depending neither on GPU speed nor on whether the CPU possesses matching hardware instructions:
AES-GCM with AES-NI still saturates at 17.4 GB/s; **possessing hardware instructions ≠ fast enough that GPU cannot catch up**.
The only workload on this machine measured higher than the GPU floor is **pure scan** (XOR fold 64.91 GB/s, hitting memory bandwidth wall).

⚠️ **"Two Completed Segments, and What They Alone Can Eliminate" is only a necessary condition.** For compute segment and end-to-end, see "Compute Segment" and "Answer for Criterion 1" sections.

### Compute Segment: Upper Bound Measurable Without nvcc, and Virtually Free

Inner loops of scrub / EC / compression are all **bandwidth-bound**; the ceiling of bandwidth-bound kernels is VRAM bandwidth—
and that can be measured using existing bitwise operators. What is measured is the **upper bound of the kernel** (a real kernel will only be slower).

| Segment | Throughput | Share of End-to-End |
|---|---|---|
| Compute (bitwise XOR fold in VRAM, read+write one pass each) | **1041.3 GB/s** | 9.7% |
| **End-to-end (pinned host memory → VRAM → compute, no writeback)** | **50.3 GB/s** | — |
| Of which transfer accounts for | — | **90.3%** |

Positive control: including transfer must be significantly slower than pure compute (required >3×), measured **10.3×**, passed.
⚠️ Failure to measure this difference is almost certainly due to **lack of synchronization**—if `torch.cuda.synchronize()` is omitted,
what is measured is asynchronous launch, yielding numbers an order of magnitude overly optimistic.

⇒ **The cost of the GPU path is transfer; the compute segment is virtually free.**

### Answer for Criterion 1: Pure Scan Cell Yields Negative Benefit, Directly Eliminated

E21 (Net Benefit of GPU Offloading) Criterion 1 is `CPU 16-thread elapsed time ÷ (transfer + compute + transfer back) elapsed time`, **eliminating that cell if less than 1**.

| | Throughput | Net Benefit |
|---|---|---|
| CPU (memory bandwidth wall) | 64.91 GB/s | — |
| GPU end-to-end (one-way, measured) | **50.3 GB/s** | **0.78×** |

⇒ **For the cell where CPU hits the memory bandwidth wall (pure scan, 64.91 GB/s), GPU offload yields negative benefit, eliminated.**
This is no longer deduction—both end-to-end numbers are measured.

⚠️ **Pure scan is not any of the three candidate scenarios in [decisions.md](../decisions.md) D24 (Offloading Heavy Background Work to GPU)**:
scrub checksum / EC reconstruction / batch compression on CPU all compute on top of scanning, whereas empirical encryption cell is only 17.4 GB/s
⇒ below both floors. **The 0.78× negative benefit eliminates a scenario not on the candidate list.**

**⇒ The threshold is sharpened accordingly: GPU begins to win only when CPU throughput drops below 50.3 GB/s (one-way scenario).**
Substituting empirical multi-core saturation from [experiments.md](../experiments.md) E6 (Encryption Algorithm Selection): with AES-NI at **17.4 GB/s** ⇒ wins;
without AES-NI, multi-core unmeasured, single-core 0.257 GB/s and saturation will only be lower ⇒ wins, by an order of magnitude.

### GPU Is Indeed Working: Three Independent Lines of Evidence (Supplementary Evidence 2026-08-28)

**Origin was a reasonable suspicion**: someone looking at `nvidia-smi` observed 5090 at **11 W**, concluding "the card did not run at all".
The suspicion is grounded but the conclusion is false—both facts are true:

| Moment | Power | Utilization | SM Clock |
|---|---|---|---|
| Gap between benchmarks | **11.67 W** | 0% | 195 MHz |
| **Sustained run for 10 s** (35 samples) | **352 W** (peak 353) | **100%** | **2857 MHz** |
| Just completed | 74.5 W | 0% | 570 MHz |

Three independent lines of evidence, any one of which suffices alone:

1. **Utilization control**: idle for 2 seconds median **0%**, full load for 6 seconds median **100%**.
2. **Power draw**: 11.67 W → **352 W**.
3. **Physical impossibility**: identical `bitwise_xor` runs at **35.6 GB/s** on CPU, **1037.3 GB/s** on GPU,
   **29.1×**. Local host memory bandwidth caps at **64.91 GB/s**—CPU cannot produce 1037.

⚠️ **This lesson is more important than numbers: a benchmark running only milliseconds cannot be observed from outside to confirm whether it ran.**
Previous measurements in E21 (Net Benefit of GPU Offloading) each kept the GPU busy for only 2.06 ms (compute segment) to 21.3 ms (including transfer), 5 rounds each,
totaling under 0.2 seconds; whereas `nvidia-smi` samples per second and power draw exhibits latency
⇒ **glancing at any moment almost inevitably reveals idle**.
This collides directly with `.claude/singlefs-ai-sop/rules/test-discipline.md`
"negative results must be distinguishable from 'code was never reached'"—**to claim "this path was run", one must have readings proving it actually executed**,
which millisecond-level benchmarks cannot provide on their own.
⇒ **Approach**: GPU benchmarks must either sample utilization/power concurrently, or run a sufficiently long sustained workload as proof of existence.
Script `scripts/experiment-apparatus/e21-prove-gpu.py`.

### Another Layer Overhead: All Three Scenarios Are Disk-Bound

All three candidate scenarios (scrub / EC reconstruction / batch compression) are **full-disk I/O-intensive background tasks**.
Data must first be read from disk, and disk bandwidth is far lower than both 64.91 (CPU memory bandwidth) and 28.5 (GPU round-trip).
⇒ **If the entire pipeline is disk-bound, CPU is already idling; offloading compute to GPU saves nothing**—
unless CPU is so slow as to be slower than disk, which cycles back to "whether hardware instructions exist".

⚠️ **This layer is deductive reasoning, not measured**: this project lacks disk arrays; disk bandwidth has not been measured.
⚠️ **Another escape route remains untested**: GPUDirect Storage allowing NVMe direct DMA into VRAM,
bypassing host memory and PCIe hops; **it would overturn the transfer floor of "GPU consumes data at 56.69 GB/s"**. Unmeasured, forbidden to treat as non-existent.

### Real Kernel Still Unwritten, but No Longer Affects Verdict for This Cell

**Host has two cards (RTX 5090 32 GiB / RTX 5060 Ti 16 GiB, driver 595.84), but lacks `nvcc`.**
⇒ Real kernels cannot be compiled. But the **upper bound** provided by bandwidth-bound operators shows compute accounts for only 9.7%,
⇒ even a slower real kernel would only worsen the GPU side, **never flipping the verdict**.
⚠️ This deduction holds only for **bandwidth-bound** kernels. **Compute-bound** kernels like compression do not fall into this category;
their compute segment may far exceed 9.7%, **and that cell still awaits toolchain support**.
⚠️ **The "no prerequisites" recorded prior to 2026-08-28 in E21 (Net Benefit of GPU Offloading) was incorrect**; the compression cell's prerequisite is this toolchain.

⚠️ 5060 Ti had only ~1 GiB free at the time (held by local inference service); **08-28 round used only 5090**,
and scripts directly refuse to run when free VRAM < 3 GiB—disrupting other services is unacceptable.

### Two Positive Controls, and How the Second Was Corrected Twice Before Succeeding

| Control | Assertion | Result |
|---|---|---|
| GPU transfer | Pinned memory must be significantly faster than pageable (pageable requires extra staging copy) | **2.05×** (56.69 vs 27.63), passed |
| CPU scan | **Set up separate compute-bound arm**, 16 threads must scale near-linearly | **15.30×**, passed |

⚠️ **The CPU control was written incorrectly twice; both times the control itself was flawed, not the machine**:

1. Initial version took **the measured arm itself** as control, demanding "16 threads 2× faster". But XOR fold is **bandwidth-bound**;
   single thread already reached 54.12 GB/s, hitting the wall at 2 threads; **it was never supposed to scale linearly**.
   Measured 1.16× ⇒ a valid result was misjudged as harness failure.
   **`.claude/singlefs-ai-sop/rules/test-discipline.md` explicitly mandates positive controls use "the baseline where effect conclusively exists"**,
   rather than the measured arm itself.
2. Second version established a separate compute-bound arm, but had **each** thread execute identical iteration counts ⇒ total work scaled with thread count,
   representing weak scaling where wall-clock is flat and ratio ≈ 1 under perfect parallelism; measured 0.96 was misjudged as non-parallel.
   Switching to **fixed total workload divided equally among threads** yielded 15.30×.

⇒ **Lesson: Upon positive control failure, suspect control specifications first, machine second.** Both failures were specification errors.

### ⚠️ Units for Cross-Language Comparison Previously Had Nothing Pinning Them (2026-08-29 Adversarial Verification Supplement)

The verdict in E21 (Net Benefit of GPU Offloading) is a **cross-language ratio**: CPU side 64.91 GB/s comes from Rust (`e21_cpu.rs`),
GPU side 50.3 / 56.69 / 28.5 GB/s comes from Python (`e21-transfer.py`).
**Both sides use decimal 1e9** (Rust `bytes / ns`, Python `gb = n / 1e9`)—
verification confirms they match, **but previously no assertion pinned this down**.
⇒ Changing either side to GiB (2³⁰) skews the ratio by **7.4%**, and
the verdict in [decisions.md](../decisions.md) D24 (Offloading Heavy Background Work to GPU) falls precisely into this order of magnitude
(the 0.78× net benefit is only 22% away from 1.0).

⚠️ **This is a cross-experiment form where "comparing only against each other cannot detect joint error"**: changing units on both sides leaves the ratio seemingly "valid".
The two unit tests in E21 (Net Benefit of GPU Offloading) pin "folding is order-independent" and "chunking forms a partition", **both being structural properties**.

Added absolute value assertion `throughput_unit_is_decimal_gigabytes_per_second`:
`decimal_gigabytes_per_second(10⁹ bytes, 10⁹ nanoseconds) == 1.0`, and `2 GiB / 1 s` must calculate as **2.147…** rather than 2.0.
Mutation table lives at `scripts/mutation/tables/e21_cpu.tsv`; three mutations (changing to GiB measurement basis, floor chunking, un-truncated final chunk) all killed.

### Measurement Basis and Code

GPU transfer segment: `scripts/experiment-apparatus/e21-transfer.py` (torch 2.11.0+cu130), warm-up round discarded per tier (building CUDA context),
**`torch.cuda.synchronize()` before and after every timing**—unsynchronized measurements capture asynchronous launches rather than transfer, artificially inflating numbers by an order of magnitude.
N=5 taking median.
CPU: `scripts/experiment-apparatus/e21_cpu.rs`, `cargo run --release --bin e21-cpu -- 2048 5`,
2 GiB buffer, best of 5 rounds; 4 unit tests, 3 mutations proven one-by-one to go red
(one mutation exposed that the unit test **re-implemented chunking logic itself without going through production path**, refactored into shared function).
Output `e21-cpu-2026-08-28.out`.
GPU compute segment: `scripts/experiment-apparatus/e21-compute.py` (bitwise XOR fold in VRAM + same operator including transfer + positive control);
the 1041.3 GB/s, 50.3 GB/s, 9.7%, and positive control 10.3× in "Compute Segment" all originate from it;
⚠️ **its raw output was unpreserved**, numbers can only be verified by rerunning this script (local machine requires CUDA and ≥ 4 GiB free VRAM; script self-rejects unqualified machines).

**Background**: [decisions.md](../decisions.md) D24 (Offloading Heavy Background Work to GPU). Three candidate scenarios (batch compression, full-disk scrub, EC reconstruction)
all fall under `.claude/rules/fs-design.md` third quadrant (background, can resume, non-decision path, unconstrained by commandments),
so introducing GPU does not violate settled discipline; however, **not a single figure existed for benefits**.

**Independent Variables**: Workload (checksum / AEAD / Reed-Solomon codec / compression) × batch size (1 MiB … 4 GiB)
× execution location (CPU single-core / CPU 16 threads / GPU) × whether data is already in VRAM.

**What to Measure** (**all three segments must be reported separately; merging into a single "GPU is N× faster" is meaningless**):

| Segment | Measurement Basis |
|---|---|
| **Transfer**: Host → VRAM | GB/s, including fixed overhead |
| **Compute**: Kernel itself | GB/s |
| **Transfer Back**: VRAM → Host | GB/s (scrub returns only a conclusion; EC reconstruction returns all reconstructed data, massive difference) |

**Criteria**:

1. **End-to-end net benefit**: `CPU 16-thread elapsed time ÷ (transfer + compute + transfer back) elapsed time`.
   **Less than 1 is negative benefit**, directly eliminating that cell.
2. **Where the curve of benefit versus batch size crosses 1**—that batch size marks the threshold for GPU enablement,
   which must be explicitly computable (`.claude/rules/fs-design.md` hard requirement 1).
3. **Impact of return volume**: scrub returns only a verdict; EC reconstruction returns whole-batch data.
   Curve shapes are expected to differ completely, **must report separately, forbidden to generalize with a single figure**.

**What Counts as Failure**:
- If GPU compute segment itself is not faster than CPU 16 threads, **no need to examine transfer segment**, workload directly eliminated.
- If net benefits across all batch sizes fall within ±10%, check whether constrained by PCIe bandwidth;
  **that indicates this category of workload is unsuitable for offload, a valid result faithfully documented**.
- ⚠️ **Positive control**: Run a task known to be extremely well-suited for GPU (e.g. large matrix multiplication) on the same pipeline;
  it must show significant speedup. Failure to measure ⇒ environment or harness defect, **invalidating the entire round**—
  forbidden to record "no speedup" as workload conclusion.

**Host** (verified 2026-08-28): NVIDIA RTX 5090 (32 GB) + RTX 5060 Ti (16 GB), driver 595.84.
⚠️ **The two cards differ in model; benchmarking must pin down which card is used and record in measurement basis.**

**Prerequisites**: None. However, requires CUDA toolchain, not yet checked by `.claude/scripts/env.sh`.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D24 (Offloading Heavy Background Work to GPU) settled item 3 | Supports | 2026-10-11 Unaffected: wording adjustments only (temporal words, location references, self-references); conclusions, numbers, and artifacts unchanged |
| D24 (Offloading Heavy Background Work to GPU) settled item 4 | Supports | 2026-10-11 Unaffected: wording adjustments only (temporal words, location references, self-references); conclusions, numbers, and artifacts unchanged |
| D24 (Offloading Heavy Background Work to GPU) settled item 1 | Does not affect | 2026-10-11 Unaffected: wording adjustments only (temporal words, location references, self-references); conclusions, numbers, and artifacts unchanged |

## Historical Versions

E21 (Net Benefit of GPU Offloading) history entries are consolidated in [experiments-history.md](../experiments-history.md).
