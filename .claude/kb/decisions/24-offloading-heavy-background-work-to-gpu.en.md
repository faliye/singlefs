## D24 Offloading Heavy Background Work to GPU —— Settled

[中文](24-后台重活能不能卸给GPU.md) · **English** · [日本語](24-バックグラウンド重処理の-GPU-オフロード要否.ja.md)

D24 (Offloading Heavy Background Work to GPU) governs one thing: whether to offload three heavy background tasks (batch compression, full-disk scrub, large-scale erasure code reconstruction) to GPU. Putting background compaction (GC relocation and packing relocation) computation on GPU is not among these three: the user decided to do it, with switches and operating policies in D26 (Background Compaction and Placement Reclamation) Settled Item 13, one set of parameters governing all background tasks, and the three hard constraints of Settled Item 2 apply to it as well. Out of scope: CPU-side algorithm selection is in D9 (Encryption) Settled Item 3; scrub form itself is in D9 (Encryption) Settled Item 5; background compaction and reclamation queues and budgets are in D26 (Background Compaction and Placement Reclamation); the three-tier classification itself is the accounting discipline of `.claude/rules/fs-design.md`, which D24 (Offloading Heavy Background Work to GPU) only references.

### Settled Items

| # | Item | Settlement |
|---|---|---|
| 1 | **Whether to implement** | Deferred, not scheduled; deferral is not a rejection, compute-limited cell remains open, GPU path testing suspended **Status: Settled.** **Progress: Unimplemented.** |
| 2 | **Three hard constraints** | Must never become essential (not a single bit enters format), forbidden on foreground path, offloaded results must be re-verified, GPU and CPU are two implementations **Status: Settled.** **Progress: No code object.** |
| 3 | **Form of criterion** | Compares CPU-side multi-core saturated throughput against two GPU transmission floors, not "single-core × core count", nor "presence of hardware instructions" **Status: Settled.** **Progress: No code object.** |
| 4 | **Reopening criteria** | Four gates G24.1 (lacking hardware instructions) to G24.4 (GPUDirect available), first two must hold simultaneously **Status: Settled.** **Progress: No code object.** |
| 5 | **Source of legitimacy** | All three candidates reside in `.claude/rules/fs-design.md` tier three; moving any scenario to tier one immediately invalidates D24 (Offloading Heavy Background Work to GPU) for it **Status: Settled.** **Progress: No code object.** |

#### Settled Item 1: Deferred, not implemented

**Settlement**: **Deferred, not implemented**—none of the three candidate scenarios (batch compression, full-disk scrub, large-scale erasure code reconstruction) are scheduled. **Deferral does not equal rejection**: the compute-limited cell (batch compression) remains open, merely unscheduled. **Testing on the GPU path is suspended**, and D24 (Offloading Heavy Background Work to GPU) halts in the state of "direction uncertain, unconfirmed on either side"—the GPU side still lacks a genuine AES-GCM kernel function (requires `nvcc`). **Do not use "GPU wins" as a settled conclusion to support other decisions.**

**Scope**: Load-bearing rests solely on "non-essential"—all three candidate scenarios reside in `.claude/rules/fs-design.md` tier three (background, resumable, non-decision path); not doing them impairs no hard commitments, unlike scaling down in D1 (Data Mobility / Reverse Index) or resident compaction in D3 (Space Allocation), which are committed capabilities. **The measured cell is not in the candidate list**: what was judged and eliminated was "pure scanning", whereas the CPU side of all three candidates performs further computation atop scanning. Therefore, the phrase in the title "one cell measured as negative return" must be read as "a cell unrelated to candidates", not "candidates judged negative by benchmark"—numbers and judging method for that cell reside in Settled Item 3.

**Basis**:

- User decided twice (this matter is non-essential; suspend testing on the GPU path), verbatim words in changelog.
- No experiment: This deferral load-bears solely on user settlements, whereas the cell eliminated by benchmarking was pure scanning, not on the candidate list, recorded in Settled Item 3; two of the three candidate cells lack data (erasure code reconstruction and batch compression), having no measurable quantities.

**Owed**: None.

#### Settled Item 2: Three hard constraints

**Settlement**: Three constraints, must not implement if unmet—

1. **Must never become essential.** The file system must function completely and with identical semantics in the absence of a GPU. ⇒ **GPU is an optional accelerator; not a single bit enters the format.**
2. **Forbidden on foreground path.** PCIe round trips are on the scale of tens of microseconds; foreground read/write latency cannot afford this. It can only serve batched heavy background tasks.
3. **It constitutes a new failure domain; offloaded results must be re-verified.** GPU computation errors (driver bugs, lack of ECC memory, algorithm implementation divergences) must not become silent data corruption ⇒ Isomorphic to `.claude/singlefs-ai-sop/rules/evidence-discipline.md` "two verification paths must not share the same snippet of code": **GPU and CPU paths must be two distinct implementations**, run against each other as a differential in tests. How results are re-verified at run time is settled by the decision that uses the GPU: background tasks follow D26 (Background Compaction and Placement Reclamation) Settled Item 13 Rule 8, where two independent GPU kernels each compute once and the results are compared, confirmed entries go to the CPU to update the bitmap, and entries that stay uncertain for several consecutive rounds are handed to the CPU implementation to judge.

**Scope**: Constraint 1 governs format, Constraint 2 governs paths, and Constraint 3 governs verification. All three are independent; failure to satisfy any single one prohibits implementation. They do not settle "whether it is worthwhile", which belongs to Settled Item 1 and Settled Item 3. Neither of the two implementations required by Constraint 3 exists (D24 (Offloading Heavy Background Work to GPU) is deferred, unscheduled). The "PCIe round trips are on the scale of tens of microseconds" of Constraint 2 differs from this project's measurement by about 100×: in the fourth rerun of E185 (Wall-Clock Breakdown of a Batch of GC Migrations) the wall clock of one GPU submission is about 5.3 ms (`research/prompts/m4-gpugc-design-r3-main-verification.md` section two, the cell "今天的形态进前台路径" (today's form enters the foreground path)); what was measured is the whole submission including the kernel, not a bare PCIe round trip, and the direction agrees with "forbidden on foreground path" (even less affordable), so the criterion of the clause is unchanged.

**Basis**:

- User settlement (GPU is an optional accelerator; on 2026-10-21 run-time re-verification was changed to two independent GPU kernels each computing once and comparing, with entries that stay uncertain for consecutive rounds handed to the CPU to judge), verbatim words in changelog.
- No experiment: All three are direct corollaries of policy and verification discipline, without measurable quantities. The PCIe round-trip scale cited in Constraint 2 is public common knowledge; this project has not measured GPU round trips on the foreground path.

**Owed**: None.

#### Settled Item 3: Form of criterion

**Settlement**: The criterion is `CPU 16-thread duration ÷ (transfer + compute + transfer back) duration`, rejecting that cell if less than 1. The CPU-side number substituted must be **multi-core saturated throughput**, not "single-core × core count": linear extrapolation differs from measured throughput on AES-GCM by nearly 4× (extrapolated ~61 GB/s, measured saturated 17.4 GB/s). The two floors on the GPU side are **one-way 56.69 GB/s** (pinned H2D, ≥64 MiB) and **round-trip 28.5 GB/s** (H2D + D2H, used by scenarios returning all data); the CPU-side upper bound is the **memory bandwidth wall 64.91 GB/s**. ⇒ **GPU only has an opportunity when "actual CPU-side throughput drops below the transmission floor".**

**Scope**: ⚠️ **"Whether bottleneck is CPU" does not equate to "whether this task has hardware instructions"**—encryption with AES-NI still saturates at 17.4 GB/s, bottleneck remaining on the CPU side; **having hardware instructions ≠ fast enough that GPU cannot catch up**. ⚠️ **The premise "CPU is fast enough so GPU is useless" does not hold in this project and must not support any conclusion**: it originates from linear extrapolation of single-core times core count, whereas multi-core benchmark saturates at 17.4 GB/s, differing by nearly 4× in the opposite direction—17.4 is lower than both GPU floors. What each of the four candidate cells becomes after substitution: scrub (checksum / MAC) with AES-NI has CPU-side multi-core saturation at 17.4 GB/s (8 threads; 16/32 threads 16.3/16.5; single-core 3656.13 MiB/s) ⇒ By the criterion GPU wins this cell, ⚠️ **unconfirmed on either side, must not be used as settled conclusion** (GPU side lacks real AES-GCM kernel function); on machines without AES-NI, single-core is 244.74 MiB/s ⇒ 16 cores ~4.1 GB/s, same scale as high-end NVMe arrays; in this tier GPU may be useful, multi-core unmeasured, saturation value will only be lower; EC reconstruction lacks data (Reed-Solomon over GF(2⁸), reconstruction reads all surviving disks and computes parity, computation scales linearly with disk count × capacity, computation in this cell is indeed heavy); batch compression lacks data (tradeoff between compression ratio and throughput, GPU compression libraries exist). These two numbers (17.4 and 244.74) come from E6 (Encryption Algorithm Selection), not measured by D24 (Offloading Heavy Background Work to GPU) itself. ⚠️ **Must not be read as "GPU is useless"**: inferences in the computation stage hold only for **bandwidth-limited** kernel functions; **compression is compute-limited**, its compute stage may far exceed that 10%, that cell remains open and still awaits `nvcc`. ⚠️ **Another layer presses above**: all three candidate scenarios are disk-bound background tasks, disk bandwidth being far lower than 64.91 and 28.5 ⇒ If the entire task is disk-bound, the CPU is already idling, and offloading to GPU saves nothing—**this layer is deductive reasoning, not empirical measurement** (this project lacks arrays). ⚠️ **An unverified escape hatch**: GPUDirect Storage lets NVMe directly DMA into GPU memory, bypassing host memory and the PCIe hop; **it would overturn the "GPU ingests data at 56.69 GB/s" transfer floor**; unmeasured, must not assume it does not exist. To settle D24 (Offloading Heavy Background Work to GPU), one must first measure CPU upper bounds for EC reconstruction and batch compression, along with net benefit of the "PCIe transfer + kernel function + transfer back" pipeline relative to pure CPU—**data must traverse PCIe once, and that transfer cost may consume all computation gains**. **Measurable on local machine**: NVIDIA RTX 5090 (32 GB) + RTX 5060 Ti (16 GB), driver 595.84; `nvcc` is not installed (`command -v nvcc` zero hits, `/usr/local/cuda*/bin/nvcc` also absent), which is a prerequisite for G24.3 (Compute stage proportion).

**Basis**:

- E21 (Net Benefit of GPU Offloading): Throughput of three paths (CPU data consumption 64.91 GB/s, GPU one-way 56.69 GB/s, GPU round-trip 28.5 GB/s) pins the two criterion thresholds—GPU merely loading data is already 13% slower than CPU reading it, before computation even begins; compute stage uses bandwidth-limited operators to provide upper bound, 1041 GB/s in GPU memory but end-to-end (including transfer) only 50.3 GB/s ⇒ Transfer occupies 90.3%, compute stage almost free, net benefit for pure scanning cell is 0.78×, less than 1 ⇒ GPU loses. **What determines whether GPU wins is whether the CPU side has corresponding hardware instructions, not how fast GPU is.**
- User settlement (correct form of criterion is multi-core saturated throughput), verbatim words in changelog.

**Owed**: None.

#### Settled Item 4: Reopening criteria

**Settlement**: Without specifying what phenomenon prompts reopening, "deferral" is an indefinite shelving. Four gates—

<!-- doc-lint:registry name-col=2 -->

| Gate | Short Name | What It Judges | Can It Be Measured |
|---|---|---|---|
| G24.1 | Lacking hardware instructions | Target platform CPU **lacks** corresponding hardware instructions, causing CPU-side throughput for this task to fall below **50.3 GB/s** (one-way scenario) or **28.5 GB/s** (round-trip scenario) | Yes, measure throughput of that task on target CPU |
| G24.2 | Not disk-bound | The task is **not disk-bound**—meaning CPU-side throughput falls below disk bandwidth, making CPU a genuine bottleneck | Requires array, not present in this project |
| G24.3 | Compute stage proportion | Compression cell: compute stage proportion of real kernel function | **No**, local machine lacks `nvcc`, prerequisite is CUDA toolchain |
| G24.4 | GPUDirect available | GPUDirect Storage is available—bypassing host memory and PCIe hop, overturning the "GPU ingests data at 56.69 GB/s" transfer floor | Unverified |

⚠️ **G24.1 (Lacking hardware instructions) and G24.2 (Not disk-bound) must hold simultaneously.** If only G24.1 (Lacking hardware instructions) holds (CPU slow due to missing instructions) while G24.2 (Not disk-bound) fails (entire task is disk-bound), slow CPU does not matter—it is already waiting on disk.

**Scope**: The four gates are sufficient conditions for reopening, not criteria for scheduling; G24.3 (Compute stage proportion) and G24.4 (GPUDirect available) cannot be measured, prerequisites being CUDA toolchain and GPUDirect Storage respectively; G24.2 (Not disk-bound) requires an array. Settled Item 4 settles "what phenomenon prompts reopening", not what order to follow once reopened.

**Basis**:

- E21 (Net Benefit of GPU Offloading): The two thresholds for G24.1 (Lacking hardware instructions) (50.3 GB/s one-way, 28.5 GB/s round-trip) and the floor that G24.4 (GPUDirect available) must overturn (56.69 GB/s) were measured by it; form of Criterion 1 also originates here.
- User settlement (without specifying what phenomenon prompts reopening, deferral is indefinite shelving), verbatim words in changelog.

**Owed**: None.

#### Settled Item 5: Source of legitimacy

**Settlement**: All three candidate scenarios fall into **tier three** of `.claude/rules/fs-design.md` accounting discipline (background, resumable, non-decision path, explicitly stated "no commandments") ⇒ Introducing GPU conflicts with no settled discipline. **The three-tier classification is the entire basis for D24 (Offloading Heavy Background Work to GPU) legitimacy—once a scenario moves into tier one (runtime decision path), D24 (Offloading Heavy Background Work to GPU) immediately becomes invalid for it.**

**Scope**: Three chains linking to other decisions: on D2 (RAID Stripe Policy) side, EC reconstruction is the cell where D24 (Offloading Heavy Background Work to GPU) most likely yields benefits; on D3 (Space Allocation) side, resident background compaction belongs to tier three, serving as a legitimate sphere of action for GPU; on D9 (Encryption) side, scrub checksum and MAC computation volume are determined by it; **the tier lacking AES-NI is the sole entry point for GPU supported by clear data**. Settled Item 5 does not judge whether these three cells are worthwhile, which belongs to Settled Item 1 and Settled Item 3.

**Basis**:

- `.claude/rules/fs-design.md` three-tier table "accounting is a byproduct of transactions, not post-hoc traversal": tier three verbatim "background, resumable, non-decision path (snapshot destruction, compaction, scrub)", explicitly stated "no commandments".
- No experiment: Which tier a scenario lands in is judged by the criterion of that discipline itself ("whether there is an answer the moment it is queried"), having no measurable quantities.

**Owed**: None.

## Historical Versions

Historical entries for D24 (Offloading Heavy Background Work to GPU) are centralized in [decisions-history.md](../decisions-history.md).
