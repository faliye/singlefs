## E140 Cross-Unit Pages and Read-Side Copying Forced by Unit Header — Run (2026-09-13 Round 3, Timed, N=5 Seeds, 10 Unit Tests / 12 Mutations All Caught)

[中文](140-单元含头逼出的跨单元页与读侧拷贝.md) · **English** · [日本語](140-ユニットヘッダに起因するユニット境界跨ぎページと読み取り側コピー.ja.md)

What is being asked is the two layers of costs never recorded on the **read path** for verbatim wording of D4 (Checksum Location) settled item 5 "including header; units are strictly 32768 bytes, with self-describing header, declared length, payload, and padding all within these 32768 bytes".
The cost noted by D4 (Checksum Location) settled item 5 itself is that payload is not a power of 2, requiring division to map file offset to unit; D18 (Information Carried in a Block) settled item 7 / settled item 14 settled the code 1 header as 105 bytes (133 bytes including nonce / MAC reservation; after D18 (Information Carried in a Block) settled item 16 counted algorithm type 1 byte into reservation, code 1 header with reservation is 134 bytes, and E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) ran the 133 tier).
Hence payload P = 32768 − h is not an integer multiple of 4096, and payload starts from offset h:

1. A proportion of (4096 − gcd(4096, P)) / P of file 4 KiB pages straddles across two units (4095 / 32635 = 12.55% when P = 32635), requiring reading two units and verifying twice to read one page.
2. Page cache pages / O_DIRECT user buffers cannot achieve zero-copy, incurring one additional memcpy per page read; sequential reads must copy payload of each unit into a contiguous buffer.

The only in-format alternative is padding the payload to 28672 (7 whole pages, header and padding taking one page), entirely eliminating both consequences, at the cost of 32768 / 28672 − 1 = 14.3% extra device bytes per user byte.

**All six failure clauses in Round 3 untriggered, criteria 1 / 2 across 5 seeds in same direction**: random 4 KiB page read with header (header 133) is 7.7%–9.1% slower than padded form (QD=1), 7.1%–8.0% slower (QD=16);
sequential read user bandwidth of padded form is 7.6%–9.2% less than with header. The two cells move in opposite directions; per pre-run hardcoded criterion 4 reporting only the crossover point: when random page reads account for 1.05%–1.47% of user bytes, the mixed costs of both forms are equal.
Round 1 and Round 2 were each invalidated per their own clause b; their progression is documented in section "Progression of Two Invalidation Rounds".

**Re-run command** (`timing` mode: single-round artifact for seed 1, ops 4096 supplied for `replay.sh` byte-for-byte comparison after stripping timing; nanoseconds and ratios pinned by range assertions):

```bash
bash scripts/experiment-apparatus/replay.sh E140          # Test partition taken from REPLAY_DEV140, defaulting to temp dir, binary creates and fills 8 GiB itself
# Run apparatus directly (five seeds of Round 3; test file path forbidden from repository):
cd scripts/experiment-apparatus && cargo build --release --bin e140-header-alignment
for s in 1 2 3 4 5; do ./target/release/e140-header-alignment "${E140_IMG:-${TMPDIR:-/tmp}/e140.img}" $s none 16384 8192; done
./target/release/e140-header-alignment "${E140_IMG:-${TMPDIR:-/tmp}/e140.img}" 1 nodirect 16384 8192   # Negative control, run twice
```

Code `scripts/experiment-apparatus/e140_header_alignment.rs`, pre-run registration `e140-preregistration.md` (all three rounds of registration contained within, criteria and thresholds unedited by a single word),
mutation table `scripts/mutation/tables/e140_header_alignment.tsv` (12 caught completely, `bash scripts/mutation/mutate.sh e140-header-alignment scripts/experiment-apparatus/e140_header_alignment.rs scripts/mutation/tables/e140_header_alignment.tsv`).
Eight raw output artifacts exist, with only the first four entering conclusions:

| Artifact | What it is | Enters conclusion |
|---|---|---|
| `e140-header-alignment-r3-2026-09-13.out` | Round 3, 5 seeds, ops 16384, 115 lines | **Enters**, verdict table transcribed entirely from it |
| `e140-header-alignment-nodirect-2026-09-13.out` | Negative control, stripping O_DIRECT run twice, 46 lines | Enters, answers clause d only |
| `e140-header-alignment-repro-2026-09-13.out` | Seed 1, ops 4096 single round, 23 lines | For `replay.sh`, does not enter verdict table |
| `e140-header-alignment-r2-2026-09-13.out` | Round 2, 5 seeds, ops 16384, 110 lines | **Invalidated** (clause b, first arm slow after disk idle), retained as reference only |
| `e140-header-alignment-2026-09-12.out` | Round 1, 5 seeds, ops 8192, 110 lines | **Invalidated** (clause b threshold misestimated pre-run), retained as reference only |
| `e140-header-alignment-r2-2026-09-12.out` | Round 2 started 2026-09-12, seed 1 complete, seed 2 interrupted, 35 lines | Incomplete run, does not enter conclusion |
| `e140-header-alignment-nodirect-2026-09-12.out` | Negative control of old binary, 44 lines | Replaced by that version |
| `e140-header-alignment-repro-2026-09-12.out` | Single round artifact of old binary, 22 lines | Replaced by that version (page order differs after seed scrambling, bytes do not match) |

### Arms and Workloads

| Arm | Payload | Page to unit | Copy |
|---|---|---|---|
| `padded` | 28672 | Page k in unit ⌊k/7⌋, never straddles | None, consumed in place |
| `h133` | 32635 | Page k from file offset 4096k, unit ⌊4096k/32635⌋, straddles read two units | Random arm 4 KiB, sequential arm 32635 bytes per unit moved into aligned buffer |
| `h133_nocopy` | 32635 | Same as `h133` | None (diagnostic arm, separating cross-unit from copy) |
| `h105` / `h105_nocopy` | 32663 | Same as above | Same as above (inspecting header width sensitivity) |
| `ctl_copy1m` | 28672 | Same as `padded` | Moves extra 1 MiB each time (copy discriminative control) |

Workloads: `rand` random 4 KiB page read QD=1, `randq` QD=16 (16 threads), `seq` sequential read of 512 MiB user bytes, I/O fixed at 1 MiB. Each arm and workload verifies whole unit (AES-256-GCM, adopting the tested object of E58 (End-to-End Cost of Checksum Granularity)).

### Measurement Basis

- Apparatus: 8 GiB O_DIRECT file on local ext4 (`/tmp/e140.img`, same DRAM-less NVMe as E58 (End-to-End Cost of Checksum Granularity)), unit 32768, page 4096; random arm 16384 page reads per seed; timing is variable observation, differing across N runs.
- Warm-up before each process: 16384 random whole-unit reads before first arm (line `name=warmup` in artifact, not counted toward any arm). Added in Round 3, reason in "Progression of Two Invalidation Rounds".
- Seeds: command line seeds passed through splitmix scrambling before bit setting (fix from C59 (Seeds Folding into the Same State)). Rounds 1 and 2 used `seed | 1`, folding seeds 2 and 3, 4 and 5 into identical page sequence, with `dev_bytes` and `straddled` matching cell-by-cell in artifact; five seeds in Round 3 are five distinct access patterns (`straddled` 2081 / 2076 / 2039 / 2160 / 2064).
- Criterion 1 `r_rand` = `ns_per_op(h133) ÷ ns_per_op(padded)`, one cell each for QD=1 and QD=16; Criterion 2 `r_seq` = `user_mib_per_s(padded) ÷ user_mib_per_s(h133)`; crossover point calculated by nanoseconds per user byte (random using `ns_per_op / 4096`, sequential using `1e9 / bytes_per_sec`), `p·random_gap = (1−p)·sequential_gap`.
- Verification path: kernel `/proc/self/io` `read_bytes` delta ÷ program accounting `dev_bytes` (clause a), sharing no code with program accounting; proven to turn red by negative control.
- 12 mutations all caught: padding arm short by one byte, header with reservation miswritten, closed-form omitting gcd subtraction, gcd constant 1, page to unit dividing by unit size, device byte ratio inverted, expected unit count subtracting, straddled criterion relaxed by one cell, crossover denominator omitting one term, unit size changed to 64 KiB, one GiB unit count floored, seed unscrambled collapsing back.

### Three Controls and Invalidation Clause Verdicts (Round 3, Five Seeds)

| Clause | Criterion | Measured |
|---|---|---|
| a Positive control | Each arm `pr_over_devbytes` falls within 0.98–1.02 | 5 seeds × 16 arms all **1.0000** |
| b Copy discriminative power | `ctl_copy1m` `copy_ns ÷ ops` ≥ 5 μs, and five-round ctl / padded `ns_per_op` ratio all ≥ 1.05 | 13.6–14.5 μs; ratio 1.121 / 1.115 / 1.101 / 1.098 / 1.107 |
| c Cross-unit discriminative power | `h133_nocopy` `dev_bytes ÷ ops` = 32768 × 1.125479, tolerance ±1% | Unit count each time 1.12701 / 1.12671 / 1.12445 / 1.13184 / 1.12598, all within 1.1142–1.1367 |
| d Negative control | `nodirect` second run `read_bytes` collapses below 10% of direct tier, program accounting unchanged | Second run 16 arms `pr_over_devbytes` all **0.0000**, `dev_bytes` verbatim identical to direct tier (`rand_h133` both 605061120) |
| e Unread ≠ read 0 | Timing, count, and `read_bytes` all non-zero | No zeroes |
| f N = 5 same direction | Criteria 1 / 2 in same direction across five rounds | Five rounds `r_rand` all > 1, `r_seq` all < 1 |

**Verdict table transcribed verbatim from artifact** (Round 3 seed 1, 14 lines from lines 1 to 23 of `e140-header-alignment-r3-2026-09-13.out`):

```text
E7RESULT name=config dev=/tmp/e140.img size=8589934592 region=8589934592 units=262144 ops=16384 qd=16 seq_io=1048576 seq_user_bytes=536870912 unit=32768 page=4096 headers=105/133 padded_payload=28672 control_copy=1048576 seed=1 mode=none o_direct=true
E7RESULT name=warmup reads=16384 elapsed_ns=3264031087 sink=114688
E7RESULT name=rand_padded ops=16384 dev_bytes=536870912 user_bytes=67108864 elapsed_ns=2922883818 verify_ns=142569222 copy_ns=0 straddled=0 units_per_op=1.00000 ns_per_op=178398.7 user_mib_per_s=21.896 proc_read_bytes=536870912 pr_over_devbytes=1.0000 sink=3515688
E7RESULT name=rand_h133 ops=16384 dev_bytes=605061120 user_bytes=67108864 elapsed_ns=3174454908 verify_ns=148817114 copy_ns=778305 straddled=2081 units_per_op=1.12701 ns_per_op=193753.4 user_mib_per_s=20.161 proc_read_bytes=605061120 pr_over_devbytes=1.0000 sink=3170868
E7RESULT name=rand_h133_nocopy ops=16384 dev_bytes=605061120 user_bytes=67108864 elapsed_ns=3214200873 verify_ns=151782014 copy_ns=0 straddled=2081 units_per_op=1.12701 ns_per_op=196179.3 user_mib_per_s=19.912 proc_read_bytes=605061120 pr_over_devbytes=1.0000 sink=3516314
E7RESULT name=rand_ctl_copy1m ops=16384 dev_bytes=536870912 user_bytes=67108864 elapsed_ns=3277741218 verify_ns=135842536 copy_ns=237969365 straddled=0 units_per_op=1.00000 ns_per_op=200057.4 user_mib_per_s=19.526 proc_read_bytes=536870912 pr_over_devbytes=1.0000 sink=5592766
E7RESULT name=randq_padded ops=16384 dev_bytes=536870912 user_bytes=67108864 elapsed_ns=370200444 verify_ns=146501677 copy_ns=0 straddled=0 units_per_op=1.00000 ns_per_op=22595.2 user_mib_per_s=172.879 proc_read_bytes=536870912 pr_over_devbytes=1.0000 sink=3490086
E7RESULT name=randq_h133 ops=16384 dev_bytes=604504064 user_bytes=67108864 elapsed_ns=399946698 verify_ns=162771431 copy_ns=785426 straddled=2064 units_per_op=1.12598 ns_per_op=24410.8 user_mib_per_s=160.021 proc_read_bytes=604504064 pr_over_devbytes=1.0000 sink=3164043
E7RESULT name=seq_padded ops=586 dev_bytes=614465536 user_bytes=536883200 elapsed_ns=291480921 verify_ns=144523441 copy_ns=0 straddled=0 units_per_op=32.00000 ns_per_op=497407.7 user_mib_per_s=1756.587 proc_read_bytes=614465536 pr_over_devbytes=1.0000 sink=1110470
E7RESULT name=seq_h133 ops=515 dev_bytes=540016640 user_bytes=536878385 elapsed_ns=266880190 verify_ns=126198606 copy_ns=8575762 straddled=0 units_per_op=32.00000 ns_per_op=518214.0 user_mib_per_s=1918.491 proc_read_bytes=540016640 pr_over_devbytes=1.0000 sink=1041330
E7RESULT name=seq_h133_nocopy ops=515 dev_bytes=540016640 user_bytes=536878385 elapsed_ns=262208499 verify_ns=128829581 copy_ns=0 straddled=0 units_per_op=32.00000 ns_per_op=509142.7 user_mib_per_s=1952.672 proc_read_bytes=540016640 pr_over_devbytes=1.0000 sink=1041330
E7RESULT name=model_h133 payload=32635 straddle_fraction=0.125479 straddle_count_one_period=4095 expected_units_per_page=1.125479 dev_bytes_per_user_byte=1.004075 units_per_gib=32902
E7RESULT name=model_padded payload=28672 straddle_fraction=0.000000 straddle_count_one_period=0 expected_units_per_page=1.000000 dev_bytes_per_user_byte=1.142857 units_per_gib=37450
E7RESULT name=verdict r_rand_qd1=1.0861 r_seq=0.9156 random_ns_per_byte_padded=43.5544 random_ns_per_byte_h133=47.3031 seq_ns_per_byte_padded=0.5429 seq_ns_per_byte_h133=0.4971 crossover_random_share=0.012075
```

Negative control second run (`e140-header-alignment-nodirect-2026-09-13.out` second section):

```text
E7RESULT name=rand_h133 ops=16384 dev_bytes=605061120 user_bytes=67108864 elapsed_ns=157611900 verify_ns=130850912 copy_ns=623176 straddled=2081 units_per_op=1.12701 ns_per_op=9619.9 user_mib_per_s=406.061 proc_read_bytes=0 pr_over_devbytes=0.0000 sink=3170868
E7RESULT name=verdict r_rand_qd1=1.1315 r_seq=0.9203 random_ns_per_byte_padded=2.0757 random_ns_per_byte_h133=2.3486 seq_ns_per_byte_padded=0.2854 seq_ns_per_byte_h133=0.2627 crossover_random_share=0.076969
```

### Verdict (Per Pre-Run Hardcoded Criteria, Five Seeds Cell-by-Cell)

| Seed | `r_rand` QD=1 | `r_rand` QD=16 | `r_seq` | Crossover p* | `h133_nocopy` / `padded` Random | `padded` / `h133_nocopy` Sequential | `h105` Random | `h105` Sequential |
|---|---|---|---|---|---|---|---|---|
| 1 | 1.0861 | 1.0804 | 0.9156 | 0.012075 | 1.0997 | 0.8996 | 1.0896 | 0.9232 |
| 2 | 1.0847 | 1.0769 | 0.9148 | 0.012514 | 1.0838 | 0.8811 | 1.0829 | 0.9252 |
| 3 | 1.0768 | 1.0782 | 0.9081 | 0.014729 | 1.0748 | 0.8750 | 1.0763 | 0.9047 |
| 4 | 1.0910 | 1.0714 | 0.9238 | 0.010477 | 1.0950 | 0.8852 | 1.0858 | 0.9155 |
| 5 | 1.0858 | 1.0754 | 0.9239 | 0.010925 | 1.0852 | 0.8865 | 1.0867 | 0.9153 |

Three tiers of Criterion 4: "both cells lose ≥ 10%" did not occur, "both cells within ±5%" did not occur, falling on "one wins and one loses ⇒ report crossover point p*, picking single cell to claim conclusion forbidden".

### What These Numbers Indicate

1. **Two layers of costs are separated; the larger layer is cross-unit pages, not copying.** Random page reads with header are 7.7%–9.1% slower than padded, of which `h133_nocopy` (straddling units only, no copying) is already 7.5%–10.0% slower; copying layer accounts for 0.040–0.048 μs per page, submerged in round-to-round noise.
   Measured cross-unit overhead (7.5%–10.0%) is smaller than analytical prediction (1 + 12.55% unit count): straddling pages read two **adjacent** units, where the second 32 KiB read does not pay full price of an independent random read.
2. **On sequential side, header wins, driven by device bytes.** Padded incurs 14.3% extra device bytes per user byte, with measured padded user bandwidth being 7.6%–9.2% lower; copying in header arm itself (single-threaded synchronous copy taking 16.7–20.3 μs per 1 MiB request) eats 1.8%–4.2% (`h133` vs `h133_nocopy`), which will shrink further in real implementation once copy is overlapped with subsequent I/O.
3. **Crossover point lands at random page reads accounting for 1.05%–1.47% of user bytes**: above it, padded form saves more; below it, header form saves more. D25 (Target Workload Priority) settled seq primary, rand secondary without numerical weights ⇒ this cell cannot yield a verdict, not reopening D4 (Checksum Location) settled item 5; reopening requires numbers under D25 (Target Workload Priority) measurement basis.
4. **Header widths 105 and 133 indistinguishable**: `h105` random 1.0763–1.0896, sequential 0.9047–0.9252, with difference against `h133` falling within round-to-round range ⇒ 28 bytes reservation added by D18 (Information Carried in a Block) settled item 14 is invisible on read path.
5. Verification (AES-256-GCM whole unit) accounts for 4.9%–5.0% of random page read time, consistent with 5.1% recorded by E58 (End-to-End Cost of Checksum Granularity) for 32 KiB tier; apparatus has not drifted.

### Progression of Two Invalidation Rounds

- **Round 1 (2026-09-12, `e140-header-alignment-2026-09-12.out`)**: Clause b wrote pre-run "`ctl_copy1m` is ≥ 30% higher than `padded`", measured across five rounds at 1.10–1.12. Both sides misestimated pre-run: 1 MiB memcpy measured 13.8 μs (source and dest in L3), while single 32 KiB random read was 178 μs. Timer successfully discriminates copying; what was wrong was the threshold; Round 2 registration revised b to "`copy_ns ÷ ops` ≥ 5 μs and ratio ≥ 1.05", raising ops to 16384. Round 1 criterion cells r_rand_qd1 1.072–1.089, r_seq 0.913–0.924, crossover 0.0118–0.0135, matching Round 3 direction, not entering conclusion.
- **Round 2 (started 2026-09-12 incomplete; finished 2026-09-13, `e140-header-alignment-r2-2026-09-13.out`)**: Seed 1 ctl / padded = 0.960, failing clause b second sentence ⇒ entire round invalidated. Not that copy was unmeasurable (13.7 μs), but that first arm `rand_padded` in process was 15% slow (204.5 μs, remaining four rounds 177.5–178.3 μs); 2026-09-12 half-round seed 1 had same form (206.4 μs), while Round 1 five seeds following 8 GiB fill were never slow ⇒ attributed to disk wake-up in first seconds from idle. Round 3 added warm-up and fixed seed folding, leaving criteria and clauses verbatim untouched.
- Recorded loss: by literal wording of clause b, Round 2 seed 1 padded arm indeed failed to be 5% slower than control. Round 3 is a new experiment, not an explanation of Round 2.

### What It Cannot Answer

1. Write path reported by arithmetic only (cross-unit 4 KiB page write requires COW for two units, expectation 1.125), unmeasured; does not touch crash semantics (one straddling page write not atomic within single unit).
2. "Zero-copy" in O_DIRECT apparatus refers to in-place consumption; real system DMAs into page cache pages, and E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) measures the memcpy exchanged for it, single-threaded synchronous, un-overlapped with I/O.
3. Single disk, no RAID, no read-ahead; header width ran only 105 / 133 tiers; payload padding ran only 28672 tier (36 KiB unit not among candidates: placement encoded by 16 KiB slot numbers, D19 (Block Pointer Structure & Width Budget) settled item 4).
4. Crossover point applies to this local disk: once ratio of device random read latency to sequential bandwidth changes, p* changes.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D4 (Checksum Location) settled item 5 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 14 | Material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) settled item 4 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D25 (Target Workload Priority) | Material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-09-13
- Round 2 invalidated per clause b upon completion; Round 3 added warm-up and seed scrambling, leaving criteria untouched, all five seeds passing, committed. Source renamed 34 locations per naming discipline, added `mixed_seed_state` and `warm_up_device`, unit tests 9 → 10, mutations 11 → 12; `replay.sh` registered `timing` row and four range assertions; C59 (Seeds Folding into the Same State) supplemented fourth location; D4 (Checksum Location) settled item 5 added read path cost note.

### 2026-09-12
- Apparatus, pre-run registration, Round 1, negative control, single-round artifact, and half of Round 2 run by another session and committed with b377eb0, at that time without prose body.
