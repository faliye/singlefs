## E163 Multi-GPU Unit Checksum Computation — Run (2026-09-27)

[中文](163-GPU多卡算单元校验和.md) · **English** · [日本語](163-GPUマルチカードによるユニットチェックサム計算.ja.md)

Pre-run registration `e163-preregistration.md`; issue sheet `m2-gpu-multicard-r1-forks.md` (M1, M2, M3). The apparatus is an independent hand-written model that shares no code with `crates/`: `scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs`, bin `e163-gpu-multicard-crc32c`, hung on the feature `e163-gpu` that is not enabled by default (`wgpu = "=30.0.1"` with only `vulkan`/`wgsl`/`std` enabled, `pollster = "=1.0.1"`); the research-unit-tests cell of the gate checker-tier-research-build-and-replay does not compile it. All 10 unit tests pass (`cargo test --release --features e163-gpu --bin e163-gpu-multicard-crc32c`).

### What this section did

- **Arm C**: a bitwise, reflected CRC-32C reference implementation (polynomial 0x82f63b78, initial value 0xFFFFFFFF, output inverted), source `crc32c_reference` in `e163_gpu_multicard_crc32c.rs`.
- **Arm G**: a WGSL compute shader, one invocation per unit, computing the same CRC-32C bitwise; bytes are fetched from `array<u32>` as `(word >> 8*(k%4)) & 0xFF`; source is the embedded string `COMPUTE_SHADER_SOURCE` in `e163_gpu_multicard_crc32c.rs`.
- Batch: N = 2560 units × 32768 bytes, seed `0xE163092700000001`, unit content = filled with splitmix64.
- R1 (M1): the two NVIDIA discrete GPU cards on this machine (RTX 5090, RTX 5060 Ti) each compute 1280 units.
- R2 (M2): five shards of 512 units each; shards 0–1 on the two cards of this machine, shards 2–4 on another machine with three cards (RTX 5080, RTX 5060 Ti × 2); the sha256 of the same binary is identical on both ends, and the sha256 of the round-trip files is identical on both ends.
- F1, F2 (M3): on the two-shard configuration of R1, inject a one-bit flip (at two places) and a dropped result (at one place) respectively.
- PC-A (at the start of every run, computed by every card and by Arm C): the eight class-B anchors of section 7 of the pre-run registration (ASCII "123456789", empty string, 32×00/32×FF, ascending/descending, 32768×00/32768×FF), compared one by one against the expected values verified by command.

### Results copied verbatim from the artifacts

Artifacts: `e163-gpu-multicard-crc32c-2026-09-27-r1.out` (M1), `e163-gpu-multicard-crc32c-2026-09-27-r2.out` (M2), `e163-gpu-multicard-crc32c-2026-09-27-f1.out`, `e163-gpu-multicard-crc32c-2026-09-27-f2.out` (M3).

M1 (R1, the two cards on this machine):
```
MERGE_INPUT_HEADER seed=0xE163092700000001 unit_count=2560
MERGE total_units=2560 shards=2
MERGE matched_count=2560
MERGE mismatched_count=0 mismatched_global_unit_identifiers=[]
MERGE missing_count=0 missing_global_unit_identifiers=[]
MERGE duplicated_count=0 duplicated_global_unit_identifiers=[]
MERGE all_reference_values_distinct=true
```
Both cards report `anchors_cpu_match=true anchors_gpu_match=true sentinel_and_gid_ok=true record_count=1280` (once each). **M1 verdict: "can"**: Q1 both cards on this machine start (`nvidia-smi -L` gives 2 lines, `probe` also reports 2 cards), Q2 the mismatch count per shard is 0, Q3 the merged coverage count is 2560.

M2 (R2, two-host, five cards): the `--probe` on the other machine exits with code 0 and reports 3 NVIDIA discrete GPU cards (Q4 passes). The two shards on this machine (0, 1) and the two shards on the other machine (3, 4, RTX 5060 Ti × 2) all succeed, with `anchors_cpu_match=true anchors_gpu_match=true sentinel_and_gid_ok=true`; the round-trip files (the result files of shards 3 and 4) have identical sha256 on both ends one by one, with 512 records each (Q6 passes). **The third card on the other machine (RTX 5080, shard 2, units 1024–1535) failed to create a GPU context both times**: `request_device` failed with `RequestDeviceError { inner: Core(Device(OutOfMemory)) }`; at that time the card's `nvidia-smi` showed only 141 MiB of free memory (fully occupied by a local service; the registration says "do not stop local models"). Merging the 4 shards (0, 1, 3, 4):
```
MERGE total_units=2560 shards=4
MERGE matched_count=2048
MERGE mismatched_count=0 mismatched_global_unit_identifiers=[]
MERGE missing_count=512 missing_global_unit_identifiers=[1024, ..., 1535]
```
Coverage 2048 < 2560 triggers failure clause 2 (the Q5/Q7 threshold fails). **M2 verdict: "cannot" (under the video-memory conditions of 2026-09-27)**: it is not that two-host multi-card cooperation itself does not work (3 of 5 cards all succeeded, and cross-machine transfer, binary copy, sha256 comparison and merging were all correct), but that one of the cards did not have enough video memory at the time to allocate a wgpu device context; the registration fixes "do not stop local models, do not retry with a changed configuration", so under the registered measurement basis this observation is "cannot", and the flip condition is that card freeing up video memory.

M3 (F1, F2, both on the two-shard configuration of R1):
```
# F1
MERGE mismatched_count=2 mismatched_global_unit_identifiers=[100, 1381]
# F2
MERGE missing_count=1 missing_global_unit_identifiers=[1480]
```
The two global units injected by F1 (100, 1381) match the reported mismatch set one by one, with no over-reporting; the unit dropped by F2 (1480) matches the reported missing set one by one. **M3 verdict: "turns red"**: each of the two faults was injected once, and the comparison reported both faithfully.

### Naming of verdict lines (step 4, 4c)

`grep -n 'name=verdict'` hits 0 lines in the four artifacts — this apparatus does not write verdict lines of the `name=verdict` kind; the judgment is given directly by the `MERGE ...` series of fields. The fields meaning "failed" in each artifact are named one by one:

- `-r1.out`: `mismatched_count=0`, `missing_count=0`, `duplicated_count=0` (all violation-class counts with value 0, not named; `all_reference_values_distinct=true` is boolean true, not named). **There is no false / not_run field.**
- `-r2.out`: the failure in the `r2-shard2-failure.out` section is the wgpu **verbatim error** (`RequestDeviceError`), not a verdict field written by this apparatus; the registration allows "a card that cannot start reports the verbatim wgpu error"; in the merged part `missing_count=512` — an integer count whose name means "missing" and which is non-zero, **named**: it is the direct consequence of insufficient video memory leaving shard 2 with no result at all, within the registered expectation (triggers failure clause 2).
- `-f1.out`: `mismatched_count=2` — an integer count whose name means "mismatch" and which is non-zero, **named**: these are the two bit flips deliberately injected by F1, reported faithfully by the comparison, within the registered expectation (Q8 passes).
- `-f2.out`: `missing_count=1` — an integer count whose name means "missing" and which is non-zero, **named**: it is the one record deliberately dropped by F2, within the registered expectation (Q9 passes).

There are 3 named verdict lines in total (`missing_count=512`, `mismatched_count=2`, `missing_count=1`), all deliberate injections or known environment limits within the registered expectation, and there is no `false` / `not_run` field.

### Mutation

The mutation table `scripts/mutation/tables/e163_gpu_multicard_crc32c.tsv` (7 rows: `M1_合并比对恒判相同`, `M2_合并只遍历收到的记录不按0..N查缺`, `M3_着色器用未反射的多项式`, `M4_着色器漏最后取反`, `M6_着色器按大端取字节`, `M7_结果的gid不从设备回读`, `M8_每次只派一半工作组`; the row names are the table's own internal labels, not repo-wide identifiers). **`scripts/mutation/mutate.sh` cannot run this table**: it fixedly uses `cargo test --release --bin "$BIN"` without `--features`, while this bin hangs on `required-features = ["e163-gpu"]`, so `cargo test` cannot find the bin target and the baseline is not green to begin with (exit code 2, `mutate: 基线就是红的，先修好再来`) — this shares the same tool limitation as the bin of E162 (Storage Backend Selection for Crash Amplification Judgment), which also hangs on `required-features`, and is not a mistake in this apparatus.

Verified manually one by one per `.claude/rules/mutation-sampling.md` (change one line, recompile and rerun with `cargo test --release --features e163-gpu --bin e163-gpu-multicard-crc32c`, see whether an assertion goes red, revert and confirm the baseline is green again):

| Mutation | Test that caught it | Note |
|---|---|---|
| Merge comparison always judges equal | `compare_reference_and_shards_reports_injected_mismatch` | Caught |
| Merge only traverses received records, does not check for gaps over 0..N | `compare_reference_and_shards_reports_missing_unit` | Caught |
| Shader uses the non-reflected polynomial | `gpu_matches_cpu_reference_on_anchor_vectors` | Caught; the hexadecimal value of the non-reflected polynomial is `0x1edc6f41` |
| Shader omits the final inversion | `gpu_matches_cpu_reference_on_anchor_vectors` | Caught |
| Arm C and the shader both changed to the same wrong polynomial | `crc32c_reference_matches_prereg_anchor_vectors`, `gpu_matches_cpu_reference_on_anchor_vectors`; the cross-comparison tests are all green | Caught, verified manually: the constants independently declared in two places, Rust-side `CASTAGNOLI_REFLECTED_POLYNOMIAL` (`:24`) and WGSL-side `CRC32C_REFLECTED_POLYNOMIAL` (`:234`), were both changed to `0x82f63b79`; the two are about 210 lines apart with no continuous anchor in between that could frame both, and `mutate.sh` recognizes only one original-text→replacement anchor per row, so it cannot be written as one TSV row |
| Shader fetches bytes as big-endian | `gpu_matches_cpu_reference_on_anchor_vectors` | Caught |
| The result's gid is not read back from the device, a fixed value is used instead | `gpu_dispatch_covers_every_unit_no_sentinel_residue`, `gpu_matches_cpu_reference_on_anchor_vectors` | Caught |
| Dispatch only half of the workgroups each time | All three GPU tests | Caught |
| The result file received on this machine has its last line truncated | Manually truncate the last line of `r1-shard1.out` and rerun `merge`: `missing_count=1 missing_global_unit_identifiers=[2559]`, sha256 differs from the original file | Caught, verified manually: this damages the artifact, not the source; `mutate.sh` works by text-substituting the source and then compiling, so it cannot frame this class to begin with |

All nine turned red as expected: caught 9 / ineffective 0 / not red 0; of these, "Arm C and the shader both changed to the same wrong polynomial" and "the result file received on this machine has its last line truncated" could not be written into the TSV table, for the reasons in their respective "Note" cells; both were done manually per the criteria of `mutation-sampling.md` and confirmed to turn red, so they are not blind spots, just not counted in the number of rows of the automated mutation table.

After changing one format constant, each of the nine was confirmed "reverted, baseline still all green" after its change (all 10 unit tests pass).

### A real bug that hung cargo test, now fixed (off topic, recorded on this page so a rerun does not step on it again)

The first run of `cargo test --release --features e163-gpu --bin e163-gpu-multicard-crc32c` at the default parallelism hung for 39 minutes with no output (found by the main agent, who directed the investigation): the three unit tests that need a GPU each create a `wgpu::Instance` and all pick the same card (index 0 in enumeration order); the test threads, running in parallel by default, simultaneously create a `Device` on the same physical card, dispatch, and `device.poll(Wait)`, and after colliding they deadlock in some GPU-side blocking call, neither reporting an error nor timing out. Fix: add an in-process `static GPU_TEST_MUTEX: std::sync::Mutex<()>` to these three tests to serialize access to the GPU; after the change, 4 time-limited runs passed all 10 items, in 0.73–1.53 seconds. This fix is already in the final source and is not a to-do.

### Replay

`scripts/experiment-apparatus/replay.sh` registers only the one segment of R1 that contains no timing fields and is byte-comparable — the output of `merge` (each shard result file itself carries timing fields such as `build_device_ms`, which differ between two runs, so no byte-level comparison is registered for them):

```
bash scripts/experiment-apparatus/replay.sh E163
```

Verdict: bytes identical (`research/results/e163-gpu-multicard-crc32c-2026-09-27-r1-merge.out`, driver function `driver_e163_r1_merge`, measured 2026-09-27). The apparatus itself does not use the `E7RESULT` protocol of `e7_index_bench::Emitter`; the driver function wraps the raw output of `merge` in bash, satisfying the `E7RESULT` / `name=done` closing line required by the completeness gate of `replay.sh`. The machine needs at least two NVIDIA discrete GPU cards that can start a context; the replay commands for R2, F1 and F2 are in the actual invocation form in the section "What this section did", and no byte-level comparison is registered (R2 depends on the other machine and its video-memory state at the time).

### Path and conclusion registration

| # | Path | How it is computed | Source placement | Shared items read |
|---|---|---|---|---|
| 1 | CPU reference | Bitwise, reflected CRC-32C, initial value 0xFFFFFFFF, output inverted (function body ends at :43) | `scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs:30` `fn crc32c_reference` | The four parameters of settled item 17 of D18 (Information Carried in a Block) (Rust-side constants, declared independently) |
| 2 | GPU shader | The WGSL compute shader computes the same CRC-32C bitwise per unit, bytes fetched as `(word >> 8*(k%4)) & 0xFF` (shader source ends at :273) | `scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs:216` `const COMPUTE_SHADER_SOURCE` | The four parameters of settled item 17 of D18 (Information Carried in a Block) (WGSL side writes its own copy of the constants, does not reference path 1) |

Cross-path assertions: the unit test `gpu_matches_cpu_reference_on_anchor_vectors` compares the outputs of the two paths on the 8 anchors each against `EXPECTED_ANCHOR_VALUES` (section 7 class B of the registration, not a comparison between the two paths); `compare_reference_and_shards` of the `merge` subcommand compares the GPU results unit by unit against the references freshly computed on the CPU (2560 values, sweeping the whole batch, not sampled points). The conclusion "for all 2560 units of R1, Arm C and Arm G are identical one by one" is universal, and the value range of X is all 2560 units of this batch swept.

⚠️ What the two paths share is the single premise that "the four parameter values of settled item 17 of D18 (Information Carried in a Block) are themselves correct": if those four values are themselves written wrongly in the kb, the two independent implementations will each write one copy by the same wrong value and keep agreeing with each other on healthy input (the kind of false red warned about in settled item 5 of D13 (Verification Route)). The mutation "Arm C and the shader both changed to the same wrong polynomial" demonstrates this: the cross-comparison tests are all green, and only the absolute-value check against `EXPECTED_ANCHOR_VALUES` catches it — this check (`crc32c_reference_matches_prereg_anchor_vectors`, `gpu_matches_cpu_reference_on_anchor_vectors`) is indispensable.

### What it cannot answer

- Whether all five cards on two machines can also succeed when video memory is ample: the only card that failed this time (RTX 5080) had only 141 MiB of free video memory, caused by local service usage, and is not a limit of this apparatus or of the wgpu/Vulkan path itself; it has not been rerun after video memory was freed.
- Speed, throughput, and the benefit of multiple cards relative to the CPU: the registration fixes "ask only whether it can, set no speed threshold, report elapsed time without judging it"; the elapsed-time numbers are in the artifacts but have no criterion.
- Sensitivity to unit length, dispatch granularity and batch size: it has only been run at N=2560, 128 units per dispatch (the shader actually works in groups of 64, see workgroup size), and 32768-byte units; section 8 of the registration states "no geometry sensitivity".

### Decisions affected

| Decision item | Relation | Review |
|---|---|---|
| D13 (Verification Route) settled item 5 | Preparation | 2026-10-11 unaffected: only wording changed (time words, positional references and self-reference), conclusions, numbers and artifacts unchanged |
| D18 (Information Carried in a Block) settled item 17 | Preparation | 2026-10-11 unaffected: only wording changed (time words, positional references and self-reference), conclusions, numbers and artifacts unchanged |
| D24 (Offloading Heavy Background Work to GPU) | Not affected | 2026-10-11 unaffected: only wording changed (time words, positional references and self-reference), conclusions, numbers and artifacts unchanged |

## Historical Versions

### 2026-09-27: page created, first run

M1 (single machine, two cards) verdict "can", M2 (two machines, five cards) verdict "cannot" under today's video-memory conditions (the only failing card had 141 MiB of free video memory; it is not that the cooperation mechanism itself does not work), M3 (whether the comparison turns red) verdict "turns red". All 9 mutations caught, and one real bug fixed in which the three GPU tests trampled on each other under the `cargo test` default parallelism and hung (a mutex added to serialize them).
