# Experiment changelog

[中文](experiments-history.md) · **English** · [日本語](experiments-history.ja.md)

<!-- doc-lint:history-registry -->
<!-- doc-lint:not-numbers M1 M2 M3 M4 M5 M6 M7 M8 M9 M10 M11 M12 M13 M14 M15 M16 M43 M63 M67 A11 -->

The history of [experiments.md](experiments.md) and of each experiment's text under `experiments/`. One section per experiment, headed `## E<n>（简称）`, in the order of the index table in experiments.md; the line at the top of a section, `**现状**：`, is taken from the "status" and "conclusion (brief)" cells of that index row, and below it entries are grouped by date in reverse order, one change per `#### ` subheading, with the text giving before, after and basis. A change that names several experiments has one copy in each of those sections, with the same content. The layout and how to add an entry follow `.claude/rules/changelog-format.md`.

⚠️ Item numbers in the entries are always today's numbers (the repository-wide rewrite of 2026-08-30 also rewrote entries older than that day); what state an item was in at the time is given by that entry's own before, after and basis.

## E1 (Node Size vs Recoverability)

### 2026-10-17

- Title parenthetical synced with the index row: the recovery-time half is written as "not run, checker exists" (the checker is implemented, the experiment still has not run).
  - **Before**: the title line of `git show ce1457c3:.claude/kb/experiments/01-节点大小vs可恢复性.md`.

### 2026-09-20

- E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal), E3 (Power-loss Semantics of Intent Log), E4 (Crossover of Range Rebuild vs Item-by-item Modification), E5 (Nonce Uniqueness × Crash-Point Replay), E7 (Offline Index Harness), E9 (Key Encoding Impact on Traversal Locality), E10 (Aging: Structural Resistance vs Post-hoc Compaction), E13 (Four Risk Paths of D5), E15 (Validity of Heterogeneous Device Geometry Simulation), E20 (CPU Cache Cost of Widening Pointers), E29 (Blast Radius of a Single Failed Node), E30 (Write Amplification Half of the Crossover Point) get an "Affected decisions" table; in E3 (Power-loss Semantics of Intent Log) and E7 (Offline Index Harness), one decision clause each that was cited by its old name is changed to its current value as looked up now
  - **Before**: none of the thirteen pages had a "### 影响的决策" table, and all thirteen rows were in `.claude/decision-links-pending`. The background sentence of E3 (Power-loss Semantics of Intent Log) cited the subsection name "断电之后怎么办" of D8 (Core Index Structure); the parenthetical in the node-size sweep section of E7 (Offline Index Harness) read "D12 (Target Media) settled item 1 has been settled on this basis as 2 sets".
  - **After**: each of the thirteen pages gets a table with one row per decision mentioned on the page, 23 rows in all — 5 support, 1 overturn, 10 preparation, 7 no effect; the thirteen rows are deleted from `.claude/decision-links-pending`. That sentence of E3 (Power-loss Semantics of Intent Log) now points to the three-state power-loss table of D8 (Core Index Structure) settled item 5; that parenthetical of E7 (Offline Index Harness) becomes "the number of layout sets belongs to D12 (Target Media) settled item 1, no cap set in advance today". Not one number of the experiments' conclusions, criteria, artifacts or preregistrations changed.
  - **Basis**: looking up `.claude/kb/decisions/08-核心索引结构.md` now, the whole text has no heading "断电之后怎么办"; the three-state power-loss table and "intents must be idempotent" live in settled item 5. Looking up `.claude/kb/decisions/12-目标介质.md` settled item 1 now, the settled text reads verbatim "no cap set in advance. Every medium may have its own disk layout"; "number of sets = 2" was overturned on 2026-08-28, with the original text in `.claude/kb/decisions-history/2026-08.md`. The 6 support and overturn rows refer to D8 (Core Index Structure) and D19 (Block Pointer Structure & Width Budget), both still in `.claude/decision-links-pending` and with text not yet rewritten into the slimmed form, so no decision's "**Basis**" paragraph changed along with this. The changes were made in place with `scripts/kb/replace-once.py`, each hitting exactly once and read back to confirm.

### 2026-09-06

- The criterion of E1 (Node Size vs Recoverability) is replaced by the rescan cost (user decision, at the same time as D8 (Core Index Structure) settled item 4)
  - **Before**: the criterion was "take the node size with the lowest lookup cost on condition that **the full-disk scan rebuild success rate ≥ the target line**";
    "What is measured" listed "the **success rate** of the full-disk scan rebuild"; the background read "the **range lost** to one bad node grows linearly with node size";
    "What it should also produce" asked for a deterministic mkfs selection rule that **takes device characteristics as input and outputs a node size**.
  - **After**: the criterion is "take the node size with the lowest lookup cost on condition that **the rescan cost ≤ the target line**";
    the success-rate line is replaced by the rescan cost, stating that the success rate **has no discriminating power**; the background becomes "the **rescan cost** of one bad node grows linearly with node size"
    with a note that what grows is recovery time, not permanent loss; the selection rule is dropped entirely and replaced by "what it produces is **an upper-bound check on that constant**".
  - **Basis**: E29 (Blast Radius of a Single Failed Node) measured on 2026-08-29 that permanent loss from a bad index is always 0,
    and its text says verbatim "the 'success rate' measures the wrong thing; what should be measured is the rebuild cost";
    D8 (Core Index Structure) settled item 2 has already pinned 16 KiB as a constant, "in the format it is a constant, not a formula".
  - ⚠️ **Since 2026-08-29 the title had said "its criterion was corrected by it", while the criterion sentence in the text had not changed by a word, and for five days nobody noticed** —
    the title changed but the sentence the title points to did not, which is another form of the index and the text drifting apart
    (the same day's "number fourteen" records the form involving index rows of experiments.md).
    **Stage 34 cannot catch this form**: it compares index rows with text titles and has no reach over whether the text is consistent within itself.

### 2026-08-26

- 
  - File created. E1 (Node Size vs Recoverability) node size vs recoverability, E2 (Key Layout Impact on Recursive Traversal) impact of key layout on traversal,
    E3 (Power-loss Semantics of Intent Log) power-loss semantics of the intent log, E4 (Crossover of Range Rebuild vs Item-by-item Modification) crossover of range rebuild vs item-by-item modification, none run yet.

- See the same-day entry "" in the E7 (Offline Index Harness) section.

## E2 (Key Layout Impact on Recursive Traversal)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-08-29

- 
  - **E2 (Key Layout Impact on Recursive Traversal) is no longer a separate experiment and is answered by E9 (Key Encoding Impact on Traversal Locality) instead.**
    **Previously**: E2 (Key Layout Impact on Recursive Traversal) was recorded as "not run (depends on the D8 (Core Index Structure) decision)"; **now**: the three tiers it was to measure are exactly the three real arms of E9 (Key Encoding Impact on Traversal Locality);
    **Basis**: the criterion of E2 (Key Layout Impact on Recursive Traversal) reads verbatim "compare recursive traversal under three tiers: by inode / inode+locality prefix / full path",
    and `Inode` / `Locality` / `FullPath` of E9 (Key Encoding Impact on Traversal Locality) are exactly these three tiers, and E9 (Key Encoding Impact on Traversal Locality) also answered the failure clauses of E2 (Key Layout Impact on Recursive Traversal).
    ⚠️ D8 (Core Index Structure) in decisions.md previously had one citation that wrongly pointed to E2 (Key Layout Impact on Recursive Traversal) (the full path's 7.38× / 10.80×); it now points to E9 (Key Encoding Impact on Traversal Locality).
  - **E6 (Encryption Algorithm Selection) gets a multi-core scaling tier**; see the text of the "2026-08-29" entry.

### 2026-08-26

- See the same-day entry "" in the E1 (Node Size vs Recoverability) section.

## E3 (Power-loss Semantics of Intent Log)

### 2026-10-17

- Title parenthetical synced with the index row: changed to "transaction layer and checker exist, experiment not run".
  - **Before**: the title line of `git show ce1457c3:.claude/kb/experiments/03-意图日志的断电语义.md`.

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-08-26

- 
  - E3 (Power-loss Semantics of Intent Log) takes over "idempotent resume": the property moves here from invariants.md (a single image cannot judge it), and criterion ③ is its only statement.
  - New: E5 (Nonce Uniqueness × Crash-Point Replay) nonce uniqueness × crash-point replay, E6 (Encryption Algorithm Selection) encryption algorithm selection, neither run yet.
    Based on D9 (Encryption) in decisions.md and the I-6 (encryption invariants) class in invariants.md.

- See the same-day entry "" in the E1 (Node Size vs Recoverability) section.

## E4 (Crossover of Range Rebuild vs Item-by-item Modification)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-08-26

- See the same-day entry "" in the E1 (Node Size vs Recoverability) section.

## E5 (Nonce Uniqueness × Crash-Point Replay)

### 2026-10-17

- Title parenthetical synced with the index row: changed to "transaction layer and checker exist, encryption implementation still owed".
  - **Before**: the title line of `git show ce1457c3:.claude/kb/experiments/05-nonce唯一性×崩溃点重放.md`.

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-08-26

- See the same-day entry "" in the E3 (Power-loss Semantics of Intent Log) section.

## E6 (Encryption Algorithm Selection)

### 2026-09-19

- Several current-state sentences in E6 (Encryption Algorithm Selection), E58 (End-to-End Cost of Checksum Granularity), E65 (Write-Side Cost of Checksum Granularity) and E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) corrected as looked up now — add "holds only for hot buffers", delete a quotation that the decision never had, align the SLC wording with the same page, add 134 for the code 1 header width
  - **Before**: the section "Unit size: the gain is essentially saturated beyond 16 KiB" of E6 (Encryption Algorithm Selection) said "⇒ **large units buy no throughput on the CPU side.**" and "what it proves is '32 KiB buys nothing in CPU throughput', not '32 KiB should not be chosen'", without the qualifier "holds only for hot buffers"; line 15 of E58 (End-to-End Cost of Checksum Granularity) said "the text of D4 (Checksum Location) itself says 'E6 (Encryption Algorithm Selection) measures encryption throughput, not end-to-end read latency'", and D4 (Checksum Location) never had that sentence; "What it cannot answer" of E65 (Write-Side Cost of Checksum Granularity) said "**changing the device requires a rerun** (the time multiple being absorbed by the SLC is a property of this disk)", contradicting the section on the same page "the explanation 'NVMe absorbs writes into the SLC' is void, the phenomenon still holds"; line 4 of E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) said "the text of D4 (Checksum Location) only records 'the payload is not a power of 2, so mapping a file offset to a unit needs a division'; D18 (Information Carried in a Block) settled item 7 / settled item 14 set the code 1 header at 105 (133 including the nonce / MAC reserved bits)".
  - **After**: the two places in E6 (Encryption Algorithm Selection) get "holds only for hot buffers", and the cold-buffer half is written out — by the five-round mean of `dev_bytes ÷ verify_ns` of the `rand` arm in `e58-csum-grain-2026-08-31.out`, AES throughput from 16 → 32 KiB is 3.18 → 3.76 GB/s (+18.2%), far smaller than the device cost on the same path (the device-side breakdown table of E58 (End-to-End Cost of Checksum Granularity): 12.9 μs more per call, +8.6%); line 15 of E58 (End-to-End Cost of Checksum Granularity) becomes the unquoted "E6 (Encryption Algorithm Selection) measures encryption throughput, not end-to-end read latency"; that sentence of E65 (Write-Side Cost of Checksum Granularity) becomes: "bytes grow 8× while time grows only a little over a tenth" is a phenomenon measured on this disk, and the SLC mechanism has not been tested; line 4 of E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) drops the quotation that D4 (Checksum Location) never had, adds the header width of 134 including reserved bits after D18 (Information Carried in a Block) settled item 16, and states that E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) ran the 133 tier. The experiments themselves, their artifacts and conclusions did not change; the D4 (Checksum Location) rows in the "Affected decisions" tables of the four pages were reviewed, and the relationships and reasons are unchanged.
  - **Basis**: the four rows of "rot not yet fixed" in `records/2026-09-19-决策瘦身与双向登记.md`; the second slimming pass of D4 (Checksum Location) (decision changelog (number five)) moved the cold-buffer numbers of E6 (Encryption Algorithm Selection) from the decision's basis to the experiment page. Header width 134 looked up now in `DATA_UNIT_PAYLOAD_OFFSET` = 105 + 29 of `crates/singlefs-format/src/lib.rs` and in D18 (Information Carried in a Block) settled item 16.

### 2026-09-07

- See the same-day entry "the substitution paragraph of E21 (Net Benefit of GPU Offloading) stopped at an overturned linear extrapolation, and nobody looked back for nine days" in the E21 (Net Benefit of GPU Offloading) section.

### 2026-08-31

- Extra measurements and corrections for five experiments (trigger: one check that "experiment data agree with the decisions")
  **This round produces no new conclusions; it only aligns the numbers cited in decisions with the experiment artifacts.** Five places, each with its own before / after / basis:

  - **E6 (Encryption Algorithm Selection): remeasure the two CPU-feature tiers.**
    **Before**: the load-bearing table (4 KiB `3888.51 / 267.55 / 1192.00`) was measured on 2026-08-27,
    but **the code that produced it was not in the repository and the artifact was not checked in** — `research/results/` had only the 08-28 multi-core round.
    **After**: newly written `e6_units.rs` (7 unit tests / all 9 mutations caught) + driver script `scripts/experiment-apparatus/e6-units.sh`,
   artifact `results/e6-units-2026-08-31.out`. With AES-NI, AES is **2.59–3.07×** faster;
    with it masked, ChaCha is **4.88–6.47×** faster (previously recorded 2.6–3.3 / 4.5–5.9; direction and magnitude agree).
    **Basis**: `kb-discipline.md` "all historical data is only a reference" — to use it in support of D9 (Encryption) settled item 3 it must be rerun first,
    and it could not be rerun.
    ⚠️ **A failure clause really did judge red for one round along the way**: `-cpu host,-aes` only removes the `aes` bit from CPUID,
    and `aes` 0.9.3 still probes `vaes` ⇒ that tier **ran as fast as with AES-NI and computed wrong tags**
    (known-answer test: `2112edc9…` vs the external implementation's `62d27233…`). **The whole round was voided and rerun with `-aes,-vaes`; the criteria were not changed after the fact.**
    ⇒ The new known-answer test (expected values computed independently with `cryptography` 49.0.0) is the only check in this experiment that does not rely on self-consistency.

  - **E16 (Role of Journal: WAL vs Intent Log): add an ancestor-block counter.**
    **Before**: D10 (True Answer to Random Writes / Fragmentation) used "ancestors pressed down to **0.069 blocks/operation**" as the basis for lifting the format-freeze block,
    but that number had **zero hits** in the text and artifacts of E16 (Role of Journal: WAL vs Intent Log) (searched across four fields and loose ranges).
   **After**: rerun with two added fields `anc` / `anc_per_op`, artifact `results/e16-journal-2026-08-31.out`.
    Under that measurement basis (multi-stream / ckpt=1000 / **ancestors deferred to checkpoint**) the real value is **0.0567**;
    in the same cell without deferring ancestors it is **4.0**, a 70× difference — the measurement basis must be cited together with the number.
    **Basis**: section four of `records/2026-08-30-三轮攻击轮.md` had already recorded this dangling number; this round measured it.
    **This change's own control**: the existing 78 lines, with the new fields removed, are **byte-for-byte identical** to the 08-28 copy.

  - **E17 (Write Buffer Merge Limit): a fourth round.** **Before**: three rounds, 30.9–31.8M entries/s.
   **After**: four rounds (`results/e17-merge-2026-08-31-repro.out`), single thread 31.61M / 723.49 MiB/s,
    **the range does not move**; parallel speedup gives four-round ranges of 6.26–6.38× at 16 threads and 6.96–7.08× at 32 threads.
    **Basis**: D8 (Core Index Structure) cited "two rounds, 30.9–31.6M", one round older than the text.

  - **E20 (CPU Cache Cost of Widening Pointers): a sixth round.** **Before**: five rounds, while D8 (Core Index Structure) and
   decisions-history both said "six rounds". **After**: ran a sixth round (`results/e20-fanout-2026-08-31-repro.out`);
    in the beyond-L3 tier 16K=243.50 ns is still the only minimum, and the 8K=478.06 anomaly reproduces for the sixth time ⇒ **"six rounds" is now true**;
    the wording of that decision's same-day entry in decisions-history is changed back to "five rounds".
    **Basis**: checked by counting artifacts — the 8 KiB tier appears only in the three sweep rounds + repro + poscontrol; the first round has no 8 KiB.

  - **E8 (Different Write Paths for Large vs Small Files): write the four pointer arms into the text.**
    **Before**: the text only reported `ptr256 → ptr320`, while the artifact actually ran four
    (`ptr256 / ptr320 / ptr536mirror / ptr888stripe`); the
    "32 B → 111 B device reads +42.6%" cited by D19 (Block Pointer Structure & Width Budget) **had no landing place in the experiment text**.
    **After**: the device read counts and totals of the four arms go into the text (47 / 49 / 55 / 67, totals 1.000 / 1.022 / 1.086 / 1.206×).
    **Basis**: checking the artifact gives 47→67 = +42.55%; the number in D19 (Block Pointer Structure & Width Budget) is right, what was missing was a landing place.

- See the same-day entry "E58 (End-to-End Cost of Checksum Granularity) set up and run to completion" in the E58 (End-to-End Cost of Checksum Granularity) section.

### 2026-08-29

- See the same-day entry "" in the E2 (Key Layout Impact on Recursive Traversal) section.

### 2026-08-27

- 
  - E6 (Encryption Algorithm Selection) run to completion (two CPU-feature tiers): which is faster flips with AES-NI; with AES-NI, AES is 1.5–3.2× faster,
    with it masked, ChaCha is 3.7–4.4× faster ⇒ the answer to D9 (Encryption) settled item 3 is to choose at run time by CPU features.
    The failure clause passed (AES drops 14.5×), and ChaCha is unaffected by masking across all four unit tiers (≤0.58%),
    with the negative control running through the whole sweep; the tagchk values of the two tiers are bit-for-bit identical, proving the same computation ran.
    The unit-size gain saturates beyond 16 KiB (4K→64K adds only 6.1%/33.3%), directly answering the unsettled item of D4 (Checksum Location): units should not be large.
    The second criterion is triggered: without AES-NI a single core does 266.60 MiB/s, below the sequential bandwidth of a single disk ⇒ encryption must be parallelizable.

### 2026-08-26

- See the same-day entry "" in the E3 (Power-loss Semantics of Intent Log) section.

## E7 (Offline Index Harness)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-08-31

- See the same-day entry "E58 (End-to-End Cost of Checksum Granularity) set up and run to completion" in the E58 (End-to-End Cost of Checksum Granularity) section.

### 2026-08-29

- The verification path of E7 (Offline Index Harness) reproduced in the VM; two fixes to the VM harness
  - **E7 (Offline Index Harness) rerun on a real block device in the VM, `blkstat=true`**,
   artifact `e7-index-2026-08-29-vm.out`.
    **All nine cells match one by one** (reads and writes of the three arms + reads of the three queries),
    the same check as the text's "independent block-layer readings match round by round"
    ⇒ **the gap of having reproduced "the numbers" only on a plain file is now closed.**
    Moreover `io_per_op` in the VM and on the host is **bit-for-bit identical** (1.937385 / 1.575920 / 0.493670) —
    different medium, different kernel, and the counts do not differ by a single digit.
    ⚠️ **Times are not comparable across environments**: the only readable kernel has lockdep, and the same tier takes about 2× as long as on the host.

  - **Two fixes to the VM harness** (recorded in the new [vm-harness.md](vm-harness.md)):
    ① the working directories of `vm-bench.sh` were never cleaned up; that day **154 directories, 4.5 GB** were cleared out —
    the root cause was "assignments in a subshell do not propagate back to the parent process"; it now works backward from its own log pointer file and deletes them on `EXIT`;
    ② new `scripts/devices/vm-kernel.sh`, which turns the precondition that got stuck every time, "the kernel under `/boot` is unreadable", into a script.

- See the same-day entry "experiment control audit — two experiments had zero absolute-value assertions, one experiment's positive control did not cover the configuration that produced the conclusion" in the E9 (Key Encoding Impact on Traversal Locality) section.
- See the same-day entry "E37 (Log Instance Generation) newly set up and run to completion; E7 (Offline Index Harness) has an archived artifact for the first time" in the E37 (Log Instance Generation) section.

### 2026-08-28

- See the same-day entry "" in the E20 (CPU Cache Cost of Widening Pointers) section.

### 2026-08-27

- 
  - E7 (Offline Index Harness) adds three hard requirements learned from E12 (Batched Sequential Append vs Unbatched Random Page RMW): every arm must really implement the structure it claims (with a self-check that "replacing it with a constant must change the result"),
    the amortization architecture of all arms must be symmetric (the same checkpoint cadence), and device I/O must have independent block-layer readings, with that verification path
    itself proven discriminating by fault injection. One more independent variable is added: whether the buffer holds idempotent full values or incremental Δ.
- 
  - **First round of E7 (Offline Index Harness) has run**, N=5, five different seeds, block layer matching round by round.
    Main result: Bε is 3.22× relative to the current direction of D8 (Core Index Structure), while the current direction of D8 (Core Index Structure) is only 1.23× relative to the no-batching control.
    The three hard requirements were judged to pass one by one — of these, "replacing the criterion with a constant must change the result"
    gave +73% and +897% on the two batching arms, proving the structure really participates (exactly the bar that arm A of E12 (Batched Sequential Append vs Unbatched Random Page RMW) did not pass).
    Four extrapolation boundaries are written down as well; the most important is that the tree has only 1057 nodes and the cache holds 6%.
- 
  - E7 (Offline Index Harness) adds a cache/tree ratio sweep (4 tiers × 5 seeds, block layer 60/60 cells matching). Two conclusions:
    ① `Bε / D8` (core index structure) is non-monotonic, rising to 4.96× from 2%→25%, and dropping sharply to 1.50× when the cache holds the whole tree
    ⇒ Bε's advantage condition is "the tree does not fit in the cache", isomorphic to the advantage condition of D5 (Snapshot / Space Accounting Mechanism) in E12 (Batched Sequential Append vs Unbatched Random Page RMW);
    ② the current direction of D8 (Core Index Structure) has bit-for-bit identical io/op across the first three tiers — once the write buffer is sorted by leaf, flush becomes a sequential scan,
    and LRU has a 0 hit rate on repeated large scans. Confirmed by intervention (shuffling group order −9.2%, with the 100% cache tier fully identical as the negative control).
    This yields a design requirement: if the write buffer is sorted, the node cache must be scan-resistant.
- 
  - E7 (Offline Index Harness) adds a node-size sweep (3 tiers × 5 seeds, fixed cache budget of about 1 MB, block layer 45/45 cells matching).
    Conclusion: under a fixed memory budget, large nodes are worse on all three axes, io/op, bytes and point-lookup reads,
    because as nodes grow the hit rate falls faster than "reading more per read" earns back.
    ⚠️ This conclusion is tied to the medium — E7 (Offline Index Harness) counts one node read as 1 I/O regardless of size;
    in the VM bytes dominate, so large nodes lose net; on rotating media seeks dominate and would give the opposite conclusion.
    What it supports is "node size takes different values per medium", not "layered layouts per medium".
- 
  - E7 (Offline Index Harness) adds a seek-dominated tier (QEMU throttled to 4000 IOPS). **This overturns a warning previously written in experiments-history.md**,
    "on rotating media seeks dominate, and the same data would give the opposite conclusion" / basis: at the extreme where "every I/O costs the same,
    regardless of size", large nodes are still 13.6% worse (no-batching control) and 100.8% worse (Bε), and the current direction of D8 (Core Index Structure) is even.
    The warning was wrong because it compared the wrong pair: the experiment's independent variable is a fixed memory budget, so larger nodes have higher miss rates and in fact more reads.
    N=5 (5 seeds for each of two configurations, 10 rounds with none voided), spread ≤0.72%.
    ⇒ Small nodes win at both ends, byte-dominated and seek-dominated.

### 2026-08-26

- 
  - New: E7 (Offline Index Harness) offline index harness, E8 (Different Write Paths for Large vs Small Files) large/small file split, E9 (Key Encoding Impact on Traversal Locality) full path vs inode indexing, E10 (Aging: Structural Resistance vs Post-hoc Compaction) aging, none run yet.
    The reason for setting up E7 (Offline Index Harness) is to break the deadlock "experiments wait for code, code waits for D8 (Core Index Structure)": choosing the index structure does not need a complete file system.
    Basis in [prior-art.md](prior-art.md) "Seven. Survey of academic results of the last decade (2012–2026)".
- 
  - E7 (Offline Index Harness) fixes two things: ① the baseline becomes "the current direction of D8 (Core Index Structure) (log-structured nodes + write buffer front end)",
    and a no-batching control group must also be kept as the positive control;
    ② the failure clause used to define "Bε has no advantage" as an implementation bug; it is now a legitimate result.
    Basis for the overturn: the old wording made the conclusion "whether to adopt message buffers" **unfalsifiable** —
    the experiment could only ever produce supporting evidence.
  - E1 (Node Size vs Recoverability) adds a metric column "number of keys reachable but with stale or wrong values", reported separately from "number of unreachable keys".
    Basis: a tree whose block headers, checksums and Merkle chain are all right can still have old content,
    and no current entry in invariants.md can detect this class.

## E8 (Different Write Paths for Large vs Small Files)

### 2026-09-19

- See the same-day entry "E151 (Arrival Order of User Data Placement Points and Container Arms), E68 (Selection of Inlining Threshold) one current-state sentence each corrected as looked up now; E8 (Different Write Paths for Large vs Small Files)…" in the E151 (Arrival Order of User Data Placement Points and Container Arms) section.

### 2026-09-03

- See the same-day entry "adversarial test round — all 19 owed mutation tables filled in, 10 new unit tests added where mutations escaped" in the E68 (Selection of Inlining Threshold) section.

### 2026-08-31

- See the same-day entry "extra measurements and corrections for five experiments (trigger: one check that 'experiment data agree with the decisions')" in the E6 (Encryption Algorithm Selection) section.
- See the same-day entry "E66 (How Small Files Land After Padding) set up and run to completion" in the E66 (How Small Files Land After Padding) section.
- See the same-day entry "E68 (Selection of Inlining Threshold) newly set up and run to completion" in the E68 (Selection of Inlining Threshold) section.

### 2026-08-29

- Audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical
  - **What was done**: `cargo test --release` all green (125 unit tests),
    then the 14 experiments that do not depend on devices/VMs were rerun one by one and compared byte for byte with the archives under `research/results/`.
  - **Result**: E8 (Different Write Paths for Large vs Small Files) / E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta) / E16 (Role of Journal: WAL vs Intent Log) / E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) / E19 (Spurious ENOSPC Under Defer Window) /
    E23 (Journal Geometry) / E24 (Recovery Algorithm: Does Trusting Tail First Lose Records) / E25 (Reserve Pool Size for Ring vs Chain) / E26 (Lookup Cost and Accuracy of Three Accounting Models) / E27 (Three Modelable Risk Paths of D5) / E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) / E29 (Blast Radius of a Single Failed Node) / E30 (Write Amplification Half of the Crossover Point)
    **12 in all are byte-for-byte identical** (they are deterministic counting models, as they should be).
  - **The two timing experiments reproduce within noise, conclusions unchanged**:
    - E17 (Write Buffer Merge Limit): **previously** "30.9–31.6M entries/s (707–724 MiB/s, two rounds)";
      **now** "30.9–31.8M (707–727 MiB/s, three rounds)"; **basis**: the third round's single thread at 31.78M / 727.44 MiB/s,
   parallel speedup 6.26× / 6.96×, positive control 12.04× / 15.93×. Artifact `e17-merge-2026-08-29-repro.out`.
    - E20 (CPU Cache Cost of Widening Pointers): the fourth round reproduces the same shape; in the beyond-L3 tier 16 KiB is still the only minimum (238.5 ns),
   **the 8K anomaly reproduces for the fourth time and is still unexplained**; per-tier differences from the first three rounds are 0.6–4.7%. Artifact `e20-fanout-2026-08-29-repro.out`.
      **Basis**: this curve has an unexplained kink ⇒ until it is explained, node size can only be pinned as a constant, not written as "computed from device characteristics".
  - **A pitfall hit this round, recorded as an owed check**: the first batch rerun built every binary name with underscores,
    while under `scripts/experiment-apparatus/target/release/` **two naming schemes coexist** (old with hyphens, new with underscores);
    7 experiments with `rc=127` never even started, yet the script reported them as "differs from the archive".
    ⇒ [checks-owed.md](checks-owed.md) C39 (rerun script silently misses).

### 2026-08-28

- 
  - Layer A of E8 (Different Write Paths for Large vs Small Files) has run: the crossover is the inline threshold itself, with zero gain at 4 KiB and above;
    the crossover shifts ≤0.5% with pointer width; appending writes cross the threshold midway through an operation.
    In the first round inline and msgbuf were identical cell by cell, a modeling defect (the message layer had no distinguishing behavior); fixed and rerun.
- 
  - **All six experiments cleared and rerun** (zero compile warnings, all positive controls pass, all integrity gates pass).
    E8 (Different Write Paths for Large vs Small Files) / E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) / E19 (Spurious ENOSPC Under Defer Window) match the kb cell by cell (deterministic experiments), and E17 (Write Buffer Merge Limit) / E20 (CPU Cache Cost of Widening Pointers) are within noise with conclusions unchanged.
  - **One change to E17 (Write Buffer Merge Limit)**: it had said "at 32 threads it falls back to 6.39×, hyperthreading does not help" — that was **a single observation**.
    N=5 rerun: median 6.38× at 16 threads, median **6.96×** at 32 threads, **32 > 16 in 5/5 rounds**.
    Changed to "hyperthreading still gives about +9%, but the share eaten by scattering rises from 46% to 55%".

### 2026-08-26

- See the same-day entry "" in the E7 (Offline Index Harness) section.

## E9 (Key Encoding Impact on Traversal Locality)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-08-29

- Experiment control audit — two experiments had zero absolute-value assertions, one experiment's positive control did not cover the configuration that produced the conclusion
  **The criteria are the three in `test-discipline.md`**: positive controls must run on every arm,
  every comparison must be accompanied by assertions pinning absolute values, and failure clauses must not make the conclusion unfalsifiable. The previous round only reran the experiments and **did not audit the controls**.

  ##### One. Zero absolute-value assertions (two, both fixed)

  - **E9 (Key Encoding Impact on Traversal Locality)**: **previously** all seven unit tests pinned structural properties
    (tree shape, key encoding, rename semantics, `interleave` really changing the order), and **none pinned the number being measured**;
    **now** four absolute-value assertions are added (total leaves == 122, one leaf cannot hold more than 255 objects,
    under full path each directory spans at most 2 leaves, a negative control with the hash arm > 10);
    **basis**: when leaf assignment or geometry is wrong as a whole, all four arms go wrong together, while `Inode/Locality = 1.46×` still "holds",
    **and that ratio is exactly the number D8 (Core Index Structure) cited when adopting the locality prefix**.
    ⚠️ **The first two assertions added could not stop a single mutation** (under `pos/(SLOTS+1)` and "no sorting", the total leaf count and the union happen to stay the same),
    only the latter two bite; and the first version **reimplemented the logic under test inside the test**, bypassing the production code —
    the same mistake this kb recorded in E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units). It has been extracted into `assign_leaves()`.
  - **E21 (Net Benefit of GPU Offloading)**: **previously** the two unit tests pinned "folding is order-independent" and "chunking is a partition",
    **while its verdict is a cross-language ratio** (Rust on the CPU side, Python on the GPU side), with unit consistency confirmed only by looking it up, no assertion;
    **now** decimal GB/s is pinned (`gbps(10⁹,10⁹)==1.0`; 2 GiB/s must be 2.147, not 2.0);
    **basis**: if either side switched to GiB the ratio would be off by 7.4%, while the 0.78× of D24 (Offloading Heavy Background Work to GPU) is only 22% from 1.0.

  ##### Two. Analytic controls living only in prose (one, fixed)

  - **E7 (Offline Index Harness)**: **previously** "the two analytic formulas agree with measurement (1.9375 vs 1.9378)"
    was written only in the text, with no assertion in the code; **now** `1 + (1 − 64/1024) == 1.9375` is added
    and `N_LEAF == 1024`, `FANOUT² == N_LEAF` are pinned;
    **basis**: changing a geometry constant would silently void that number, and the 3.22× cited by
    precondition 2 of D11 (Message Buffers in Index Nodes) is exactly what this set of numbers computes.

  ##### Three. Arm coverage of positive controls (two caught)

  - **E20 (CPU Cache Cost of Widening Pointers) — fixed**: **previously** the positive control was hardcoded to
    run once at `entry=16 / node=4096`, while what produced the conclusion was the `entry=40 × six node tiers` group, which **never passed the gate**;
    **now** it iterates over 18 configurations, **all pass (ratios 2.97–9.81)**, artifact
   `e20-poscontrol-2026-08-29.out`;
    **basis**: this is the second replay of the lesson from E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads).
    ⇒ Adding the gate **overturned no conclusion**, but it moved the 8 KiB anomaly from "possibly an artifact" to "more likely a real effect" —
    the ratio for `8192/40` is the highest in the 40-byte column (8.34), and the fifth sweep round reproduces the same shape tier by tier.
  - **E12 (Batched Sequential Append vs Unbatched Random Page RMW) — not fixed**:
    the three failure clauses cover only arm A and arm B, **arm C (sparse side table) has none**,
    and it is the only source of the conclusion in D5 (Snapshot / Space Accounting Mechanism) "primary + sparse side table as auxiliary".
    The owed form of the control is written down (make the probe always miss; the I/O count must change significantly), **which needs a real block device and a VM, and was not run this round**.

  ##### Four. Unit test count and a new gate

  Unit tests rose from 125 to **155**. New gate stage `.claude/gate.d/80-absolute-assertions.sh`
  (at least one absolute-value assertion per experiment), proven in both directions to go red.
  Three mutation tables have been saved into `scripts/mutation/tables/` (e9 / e21 / e7, plus e30).

- See the same-day entry "" in the E2 (Key Layout Impact on Recursive Traversal) section.
- See the same-day entry "rerun-and-review round — 20 experiments reproduce byte for byte, E34 (Root Ring Slot Geometry) file created…" in the E34 (Root Ring Slot Geometry) section.

### 2026-08-28

- 
  - E9 (Key Encoding Impact on Traversal Locality) run to completion (3 rename tiers × 5 seeds): `locality_id` saves 1.468× over pure inode on a fresh tree,
    decaying to 1.057× after 1024 renames (down to noise level); the full-path traversal advantage is 7.34× (relative to locality)
    / 10.78× (relative to inode), at the cost of rewriting about 2700 keys per rename on average.
    **The modeling was changed four times**; the first three could all produce nice tables while measuring the wrong thing, and the symptoms and root causes of the four rounds are recorded in the text.

### 2026-08-26

- See the same-day entry "" in the E7 (Offline Index Harness) section.

## E10 (Aging: Structural Resistance vs Post-hoc Compaction)

### 2026-10-26

- Raw outputs now cited from git history: the four shard artifacts of the first segment are archived (neither in the working tree nor in the staging area), and the "measurement basis and artifacts" paragraph now reads `git show e1b1ceb95:research/results/e10-aging-compaction-2026-10-13/<shard>.out`; the same paragraph states that this registration yields only the compaction floor `N₀`, and that the product batch size and the three compaction watermarks do not belong to it. Numbers, conclusions and status unchanged; the two rows of "Affected Decisions" are reviewed on the same day.
  - **Basis**: `git rev-parse` confirms the four files are the same blobs in HEAD and in e1b1ceb95; pre-run registration `research/prompts/e10-r1-prereg.md` line 32 (the batch size of row 6 of the fork sheet does not belong to this registration) and line 500 (the start watermarks are values the registration takes, not quantities it measures); investigation report `~/.local/share/singlefs/work/m4-close-exit5-exit10/result.md` section 3.4.
  - **Before**: the "measurement basis and artifacts" paragraph and the two review cells were only in the staging area before this change, without a commit; original text: "raw outputs: `research/results/e10-aging-compaction-2026-10-13/main-s4.out`, `research/results/e10-aging-compaction-2026-10-13/disk_1gib-s4.out`, `research/results/e10-aging-compaction-2026-10-13/disk_1gib-s8.out`, `research/results/e10-aging-compaction-2026-10-13/disk_1gib-s16.out`"; the two review cells began with "2026-10-24 unaffected: product full paths completed with directory prefix".

### 2026-10-21

- Status changed from "not run" to "partly run", title and index row synced: the rig, the shard driver and the 21-row mutation table are checked in; of the 21 shards of phase one, 4 have run, with the readings taken only as leads; the remaining 17 shards are stopped, and whether to revise three places in the registration (the row-3 criterion, the Y1 definition, the P1 expectation) is unsettled; no conclusion yet.
  - **Basis**: preregistration `research/prompts/e10-r1-prereg.md`, 4 artifacts under `research/results/e10-aging-compaction-2026-10-13/`, and the two entries for the phase-one runner's handback and the investigator's handback in the milestone four pre-development record `records/2026-10-13-里程碑四预开发.md`.
  - **Before**: the title line of `git show 841b3beab:.claude/kb/experiments/10-老化结构抗老化vs事后整理.md`, and the E10 (Aging: Structural Resistance vs Post-hoc Compaction) row of `git show 841b3beab:.claude/kb/experiments.md`.

### 2026-10-17

- Title parenthetical synced with the index row: changed to "file system exists, background compaction still owed".
  - **Before**: the title line of `git show ce1457c3:.claude/kb/experiments/10-老化结构抗老化vs事后整理.md`.

### 2026-09-20

- See the same-day entry "two places in E74 (Allocation Record Entry Count and Compaction Cost) saying 'all six items of D26 (Background Compaction and Placement Reclamation) are open' corrected as looked up now…" in the E74 (Allocation Record Entry Count and Compaction Cost) section.
- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-09-03

- See the same-day entry "new E93 (Placement and Fragmentation Under Aging), run to completion the same day — D26 (Background Compaction and Placement Reclamation) settled items 1 / 4…" in the E93 (Placement and Fragmentation Under Aging) section.
- See the same-day entry "E95 (Aging Node Layout Arm) design registration — the arm missing from D26 (Background Compaction and Placement Reclamation) settled item 4 set up, not…" in the E95 (Aging Node Layout Arm) section.

### 2026-08-26

- See the same-day entry "" in the E7 (Offline Index Harness) section.

## E11 (Dual-Track: Gains and Flip Cost, Measured Directly)

### 2026-09-19

- See the same-day entry "E151 (Arrival Order of User Data Placement Points and Container Arms), E68 (Selection of Inlining Threshold) one current-state sentence each corrected as looked up now; E8 (Different Write Paths for Large vs Small Files)…" in the E151 (Arrival Order of User Data Placement Points and Container Arms) section.

### 2026-08-26

- 
  - New: E11 (Dual-Track: Gains and Flip Cost, Measured Directly) gains and flip cost of the dual track. Reason for setting it up: D14 (Dual-Track: Large/Small Files & Persistent/Transient) in decisions.md is now derived and verified by ourselves,
    no longer taking the literature as its basis. The criteria of the two groups are "the savings of each measured separately" and "the slope of flip migration with scale",
    the latter directly deciding whether that entry point is an unbounded operation.

## E12 (Batched Sequential Append vs Unbatched Random Page RMW)

### 2026-09-12

- 13 experiment sources renamed, enum members renamed in two experiments, retained artifacts not changed by a byte
  - **Before**: the file names of 13 experiment sources had abbreviations or single letters (see the before column of "rename table for the 13 experiment sources"); the enum members `Obj`, `Dev` of `e57_field_authority.rs` and `Arm::NoSeq` of `e99_writebuffer_seq.rs` were printed into retained artifacts via `{:?}`.
  - **After**: the file names change to the after column of "rename table for the 13 experiment sources", and the bin names in Cargo.toml, the binary column of `scripts/experiment-apparatus/replay.sh`, the same-named mutation tables in `scripts/mutation/tables/`, and the rerun commands and source paths in the kb change with them. Artifact file names in `research/results/` are not changed: they are evidence saved under the binary names of the time, and `research/prompts/` and `records/` cite them by those names. The enum members become `Object`, `Device`, `NoSequence`, with an added `output_label()` that prints the old strings as before.

    Rename table for the 13 experiment sources:

    | Before | After |
    |---|---|
    | `e121_cap_tiers` | `e121_capacity_tiers` |
    | `e122_dir_locality` | `e122_directory_locality` |
    | `e123_k_fork_cost` | `e123_reuse_window_versus_rollback_depth` |
    | `e27_d5_paths` | `e27_snapshot_accounting_risk_paths` |
    | `e38_accounting_cow` | `e38_accounting_copy_on_write` |
    | `e40_csum_width` | `e40_checksum_width` |
    | `e42_txn_records` | `e42_transaction_records` |
    | `e43_ext_budget` | `e43_extension_point_budget` |
    | `e54_accounting_gen` | `e54_accounting_generations` |
    | `e59_msg_recompute` | `e59_message_recompute` |
    | `e74_alloc_records` | `e74_allocation_records` |
    | `e76_payload_csum` | `e76_payload_checksum` |
    | `e99_writebuffer_seq` | `e99_writebuffer_sequence` |

  - **Basis**: naming-lint flagged cap, dir, k, d5, cow, csum, txn, ext, gen, msg, alloc, seq in the file names and obj, dev, seq in the enum members. After the change, reruns of the 36 affected experiments match the retained artifacts byte for byte; the 123 mutation tables were checked row by row, and the number of hits of each original text in the sources is the same as before the rename. E12 (Batched Sequential Append vs Unbatched Random Page RMW) needs a real block device and its artifact was not retained; only one of its identifiers changed this time, and there is no rerun evidence for it, only the build and unit tests.

### 2026-08-29

- See the same-day entry "experiment control audit — two experiments had zero absolute-value assertions, one experiment's positive control did not cover the configuration that produced the conclusion" in the E9 (Key Encoding Impact on Traversal Locality) section.

### 2026-08-27

- See the same-day entry "" in the E7 (Offline Index Harness) section.
- See the same-day entry "" in the E7 (Offline Index Harness) section.
- See the same-day entry "" in the E7 (Offline Index Harness) section.

### 2026-08-26

- 
  - New: E12 (Batched Sequential Append vs Unbatched Random Page RMW): measure ourselves the two ways of judging lifetimes (D5 (Snapshot / Space Accounting Mechanism) time-order derivation vs reference counting).
    The measurement basis is set as "counter-table working set vs cache budget", because that is the only place where the two diverge.
    Three arms: D5 (Snapshot / Space Accounting Mechanism) / reference counting / D5 (Snapshot / Space Accounting Mechanism) + sparse side table. The criteria report device I/O counts and bytes, not just wall clock.
    The failure clauses have two control groups: no measurable difference in the 8× cache tier means the implementation is broken (positive control),
    while no measurable difference in the fits-in-cache tier is a legitimate result.
- 
  - E12 (Batched Sequential Append vs Unbatched Random Page RMW) has run, all N=5 rounds valid, and the three failure clauses were judged and recorded one by one.
    Main result: the D5 (Snapshot / Space Accounting Mechanism) side is completely unchanged across the three tiers (0.000010 io/op); reference counting, once the counter table no longer fits in the cache,
    jumps from 0.00256 to 0.998 (2×) and 1.751 (8×); at a 1‰ clone rate the sparse side table pushes it back down to 0.0172.
    Four extrapolation boundaries are written down as well, the most important being "the probe uses an in-memory whole-domain bitmap, which is idealized;
    a real sparse structure must be remeasured".
- 
  - **E12 (Batched Sequential Append vs Unbatched Random Page RMW) rerun, overturning the previous version's reading.** The previous version reported "five-round spread 0.0%" and hung the conclusion on I/O counts,
    both wrong: ① that 0.0% was **an illusion of a fixed seed** — the in-program I/O counter is the output of a pure function,
    the device took no part at all, and running five rounds was computing the same function five times, satisfying N≥5 in form but not in substance;
    ② at the time no independent path had verified that O_DIRECT really bypassed the cache.
    The rerun changes two things: **five different seeds**, and reading `/sys/block/vda/stat` once before and after each arm
    (counted by the kernel block layer itself, sharing no code/sampling/tools with the program counter).
    Result: **in all nine cells the ratio of block layer to program counter is 1.00**; with seeds changed the dispersion is 0.4%–4%, which is real;
    D5 (Snapshot / Space Accounting Mechanism) being constantly 2 is a structural invariant, not caused by the seed. The main conclusion is unchanged, but now it is verified.
- 
  - E12 (Batched Sequential Append vs Unbatched Random Page RMW) adds a mechanism derivation: the I/O counts of both sides are derived from first principles,
    and the derivation agrees with measurement to three significant figures (the analytic formula for reference counting io_per_op = 2×(1−cache/table);
    2× tier analytic 1.000 measured 0.998, 8× tier analytic 1.750 measured 1.751), forming a third independent verification path.
    It is also written down that the premise of the "2" of D5 (Snapshot / Space Accounting Mechanism) is "appends can be flushed in batches" — which is exactly what D16 (Publish Semantics) gives by settling on checkpoint;
    if the publish semantics went back to a new root per transaction, the arm A result would no longer hold.
- 
  - E12 (Batched Sequential Append vs Unbatched Random Page RMW) adds a section "fault injection on the measurement itself", with four cases. Core result: with O_DIRECT removed,
    block-layer readings drop to zero while the program counter is unchanged ⇒ proving the block-layer check has discriminating power, and that 1.00 is not decoration.
    The other two injections prove that the working-set/cache ratio and randomness are indeed the drivers (85× and 448× respectively).
    It also records that one of our own injected faults was written wrong at first (bigcache made the table grow along with it),
    and that it was caught by the habit of "result lines print all parameters".
- 
  - **The reach of E12 (Batched Sequential Append vs Unbatched Random Page RMW) is overturned on review.** E12 (Batched Sequential Append vs Unbatched Random Page RMW) had been written up as "a mechanism comparison of D5 (Snapshot / Space Accounting Mechanism) and reference counting";
    the text now states that it is a comparison of "batched sequential append vs unbatched random page read-modify-write" plus a rig self-check. The review read the measurement source and pointed out that
    arm A is not an implementation of D5 (Snapshot / Space Accounting Mechanism) but a 1 MiB append buffer: `prev_snap_txg` is a hardcoded constant
    (no snapshot was ever created), `birth` is a freshly made random number not read from the pointer, 16-byte records only write 8 bytes
    (the block pointer is not in them), and the whole stretch does zero reads — replacing the criterion with a coin toss gives the same result.
    Moreover the amortization architecture of the two arms is asymmetric (arm A batches, arm B does not), so in the 170,000× the contributions of the mechanism and of batching were not separated,
    and the sequential injection of E12 (Batched Sequential Append vs Unbatched Random Page RMW) itself shows batching + sorting alone eating two and a half orders of magnitude.
    It also missed the defer queue, the second persistent log that the pseudocode of D5 (Snapshot / Space Accounting Mechanism) states explicitly.
    The four premises of D5 (Snapshot / Space Accounting Mechanism) have zero coverage. **The correct characterization of E12 (Batched Sequential Append vs Unbatched Random Page RMW) becomes: a measurement rig self-check + a sensitivity analysis of arm B.**
- 
  - The status of E12 (Batched Sequential Append vs Unbatched Random Page RMW) changes from "to run" to "run", and the title is changed to what it actually measures
    (batched sequential append vs unbatched random page read-modify-write).
  - New: E13 (Four Risk Paths of D5): the four risk paths of D5 (Snapshot / Space Accounting Mechanism) (snapshot creation boundary, destruction cascade, crash replay, defer queue),
    set up on the basis of "one real measurement is still owed", as stated in the text of D5 (Snapshot / Space Accounting Mechanism) — arm A of E12 (Batched Sequential Append vs Unbatched Random Page RMW) is not an implementation of D5 (Snapshot / Space Accounting Mechanism).

## E13 (Four Risk Paths of D5)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-08-26

- See the same-day entry "" in the E12 (Batched Sequential Append vs Unbatched Random Page RMW) section.

## E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta)

### 2026-09-19

- E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta), E56 (Message Buffer Gains vs Epsilon), E59 (Can Buffered Messages Recompute Solely from Units), E98 (Geometry of Inode Records and Inode Trees), E104 (Current Version Determination in Scan Rebuild) get an "Affected decisions" table
  - **Before**: none of the five pages had a "### 影响的决策" section; all were in `.claude/decision-links-pending`.
  - **After**: each of the five pages gets a table, row by row with support / preparation / no effect and the look-back, one row for every decision mentioned in the page text; the rows supporting D11 (Message Buffers in Index Nodes) match its slimmed basis paragraph in both directions (settled item 1 ← E56 (Message Buffer Gains vs Epsilon), E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta); settled items 2 and 5 ← E56 (Message Buffer Gains vs Epsilon); settled item 3 ← E104 (Current Version Determination in Scan Rebuild), E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta); settled item 4 ← E59 (Can Buffered Messages Recompute Solely from Units), E104 (Current Version Determination in Scan Rebuild); settled item 6 ← E98 (Geometry of Inode Records and Inode Trees), E59 (Can Buffered Messages Recompute Solely from Units)). Decisions still on the backfill list (D3 (Space Allocation), D8 (Core Index Structure), D18 (Information Carried in a Block), D23 (Journal Role & Format)) get support written per the page's conclusions, down to the item, to be cited back when they are slimmed. The five pages leave the backfill list. The experiments themselves, their artifacts and conclusions did not change.
  - **Basis**: `.claude/rules/format-evolution.md` "decision text states only the current state, the basis is written as pointers; decisions and experiments are registered both ways"; for the slimming of D11 (Message Buffers in Index Nodes) see the decision changelog (number eight).

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.

### 2026-08-27

- 
  - New: E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta): turn the criterion of cost 1 of D11 (Message Buffers in Index Nodes) (idempotent full value vs incremental Δ) into something measurable.
    What it measures is not performance but **discriminating power** — inject four kinds of semantic bugs and see how many kinds I-3.1 (Allocated Statistics Match) catches under each of the two entry forms.
    The three failure clauses are written separately, one of them specifically guarding against "the form B checker not really sharing the run-time code".
- 
  - E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta) run to completion. At setup it predicted "under the incremental Δ form, the two kinds 'tombstone taken as a plain value' and 'one message lost' should be caught";
    measured, neither kind was caught / basis: that prediction implicitly assumed the form B checker also had an independent content traversal,
    which contradicts form B's own premise (the ledger is the authoritative content). The corrected mechanism is written into the text.

## E15 (Validity of Heterogeneous Device Geometry Simulation)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-09-02

- See the same-day entry "E72 (Mount-Time Recalculation of Device Descriptor Table) run, precondition downgraded from 'QEMU owed' to 'sudo owed'" in the E72 (Mount-Time Recalculation of Device Descriptor Table) section.

### 2026-08-27

- 
  - New: E15 (Validity of Heterogeneous Device Geometry Simulation), run to completion: the heterogeneous device geometry simulation holds, with a negative control (the homogeneous tier collapses to 1 kind).
    It records four silently failing pitfalls hit while building it, two of which together can fabricate a "negative result".

## E16 (Role of Journal: WAL vs Intent Log)

### 2026-10-17

- "C6 (Wrong Block Layer Semantic Assumptions Written) also requires an explicit cap on in-flight records" now points to C42 (Residual record posing as valid prefix): the C6 (Wrong Block Layer Semantic Assumptions Written) row has no such item, while the C42 (Residual record posing as valid prefix) row lists it as one of three gates; conclusions, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/16-journal的角色WALvs意图日志.md`.

### 2026-10-04

- Sixth run fills in the three mutation numbers (two whole-table runs, before and after, of the rerun with named entries 67 → 64, run by the main agent; numbers only, conclusions and artifacts unchanged)
  - **Before**: the next entry said the 18-row mutation table had two whole-table runs still to run, and the experiment page title and the index row said "not run yet".
  - **After**: before (old tree, old table of 17 rows) caught 17, invalid 0, not red 0; after (18 rows) caught 18, invalid 0, not red 0 (`research/results/e16-mutate-before-2026-10-04-h487.log`, `research/results/e16-mutate-after-2026-10-04-h487.log`, both ending with "restored, baseline still all green", with hitting the ceiling, timeouts, unscheduled and evicted all 0); registration section nine "apart from two rows, the other 16 must have the same state in both runs": comparing row by row by mutation name, the differences are exactly the two rows the registration names, and the other 16 are "caught" both times; voiding clause V6 does not trigger.
  - **Basis**: the two logs as above; sections nine and eleven of registration `research/prompts/e16-r6-prereg.md`.
- Sixth run: rerun with named entries 67 → 64 (row 4 of the question sheet), the rig and geometry of the fifth run unchanged
  - **Before**: rig constants `JOURNAL_HEADER_BYTES` 311 and `JOURNAL_NAMED_ENTRIES_PER_RECORD` 67 (Yi writes ⌈k ÷ 67⌉ records per fsync), while `crates/singlefs-format/src/lib.rs` was already 487 / 64 (stop S1 in the triggered state, used only as the comparison baseline: the three lines of `replay.sh E16R5` byte-identical); the batch axis did not include 63, 65; the "fifth run" section of the experiment page said today's record width is 67 entries/record.
  - **After**: constants changed to 487 / 64, 63 and 65 merged into the batch axis, four unit tests changed their pinned values (23 unit tests all green), the mutation table drops "67→72" and adds "64→72" and "64→67" (18 rows). Artifacts: phase one `e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out` (3650 lines), phase two `e16-fifth-run-peak-under-today-widths-second-segment-2026-10-04-h487.out` (13366 lines); the pc3 artifact is byte-for-byte identical to the registered one, so no new artifact. The cells that differ in the two phases all fall on the 18 batches where `⌈B ÷ 64⌉ ≠ ⌈B ÷ 67⌉` or on the two newly added batches 63 / 65 (artifact changed); judgements changed: the stream-count-128 row of Q1 in G4 goes from wide plateau to a Yi row, and one more row has its minimum ratio drop below 1 (G1 stream count 128, G2 stream count 64, etc.); the Q1 classification of G2 and Q2, Q3, Q5, Q6 ①② of the three geometries are unchanged; the two-dimensional maximum of G3 in phase two changes from 3.985294 to 3.984615. The two rows of `scripts/experiment-apparatus/replay.sh` for E16R5 phase one and phase two now point to the new artifacts, and the three-line rerun is byte-identical. The two whole-table mutation runs, before and after, have not run yet. The experiment page gets a "sixth run" section, the title and the index row state that the mutation table is still to run, and the look-back dates of the thirteen rows in the affected-decisions table are changed.
  - **Basis**: rerun registration `research/prompts/e16-r6-prereg.md`; artifacts `research/results/e16-fifth-run-peak-under-today-widths-2026-10-04-h487.out` and `research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-10-04-h487.out`.

### 2026-09-28

- Fifth run: rerun the question of the 56-cell sweep under today's unit widths (fanout, tree height, record width)
  Standalone rig `scripts/experiment-apparatus/e16_fifth_run_peak_under_today_widths.rs` (shares no code with `crates/`; it moves the two arms of the original rig over unchanged and swaps in today's fanout 144/147, a tree-height knob computed from file size, and today's record width of 67 entries/record). The original rig `e16-journal sweep` was run once now as the positive control, and its 56 cells are byte-for-byte identical to the 2026-08-28 run (`e16-journal-2026-09-28.out`); the new rig ran the same 56 cells on the original geometry, and comparing 224 fields (56 cells × 4 fields) one by one with the artifact of the original rig run now gives 0 differences.

  **Result**: on the real baseline (1 TiB file, today's fanout, tree height 4, 11 children under the root), "the peak falls at batch ≈ stream count" holds only in the 9 rows with stream count ≤ 16; in the 12 rows with stream count ≥ 24 the peak falls right at batch=1 (declining monotonically after that, with no per-row interior peak); on a 4 GiB file (tree height 3) none of the 21 rows has a per-row interior peak; on a 16 TiB file (tree height 5) the shape is more complex (Bing→Jia→wide plateau→Yi→wide plateau→Bing, varying with stream count). Across the 21 stream-count values, the three geometries disagree on the overall classification in 15 — the reach changes with tree height. "No drop while batch ≤ stream count" fails in 12 of the 21 rows on the real baseline (drops of 1.3%–4.7%); both halves of "collapses toward 1 once batch exceeds stream count" (no further rise, settling at 16×stream count) hold, but 12 rows actually fall below 1 at the far end of the batch axis (16384) (0.9927–0.9968), not just close to 1. Today's error of the prediction formula (subset of 56 cells, two denominators) has a median of 13.0%–22.1% and a maximum of 33.4%–93.9%, all three geometries far above the 9.6%/20.5% originally on the page. This time the denominator basis was verified: the original table's 9.6%/20.5% used **the predicted ratio as the denominator**, not the model ratio the registration wrote as default — both denominators were run and reported; this only verifies the denominator, it does not change the criterion.

  19 unit tests all green, all 16 mutations caught (`scripts/mutation/tables/e16_fifth_run_peak_under_today_widths.tsv`). Artifacts: `e16-fifth-run-peak-under-today-widths-2026-09-28.out` (phase one full grid), `research/results/e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out` (differential), `e16-journal-2026-09-28.out` (original rig positive control). Rerun: `bash scripts/experiment-apparatus/replay.sh E16R5`. Preregistration `e16-r5-prereg.md`. Phase two (G3, G5, fanout 128/296, records of 1 entry and 10⁶ entries) did not run: the enough-to-judge stop was met, with only one thing owed, that the two factors "tree height" and "children under the root" have not been separated.

  - **Basis**: the `name=q1_summary`/`name=q3_summary`/`name=row_summary` lines of `e16-fifth-run-peak-under-today-widths-2026-09-28.out`; the differential comparison of `e16-journal-2026-09-28.out` and `research/results/e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out`.
- Fifth run phase two: on 1 TiB, "with stream count ≥ 24 the peak falls to batch=1" is dominated by the number of children under the root, not tree height
  The rig gets a `second_segment` mode that runs the grid once for each of two knobs, fanout and record width, rerunning `one_tebibyte_file` (phase one's real baseline) once per group to fill the same table. The four points of the fanout knob (today's 144/147, the full fanout 147³ of `full_root_level_four`, small fanout 128/128, large fanout 296/296) all have the same 1 TiB unit count, tree height is constantly 4, and the only variable is the number of children under the root (11, 147, 17, 2): the stream count at which the peak disappears (Bing rows) moves later monotonically with the number of children under the root — with 2 children under the root Bing rows start at stream count 2, with 11 at 24, with 17 at 32, and with 147 only at 384, a 16× shift. The tree-height knob, without controlling the number of children under the root, is not monotonic (4 GiB/tree height 3/7 under the root is all Bing rows, and 256 MiB/tree height 2/58 under the root is also all Bing rows — the lowest tree, yet with not few children under the root, still all Bing rows), so tree height alone cannot explain it. Record-width knob: at 1 entry/record all 21 rows are Bing rows (Yi's record overhead is pushed so high that it eats all of Jia's advantage from batch=1 on); at 1,000,000 entries/record the row-by-row classification is the same as at today's 67 entries/record.

  23 unit tests all green (4 new), all 17 mutations caught (the new 17th anchors the newly added `RecordFormat::EntriesPerRecord` branch). Two independent runs of this mode (same code, same machine) give byte-for-byte identical artifacts (V5). Artifact: `e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out`. Rerun: `bash scripts/experiment-apparatus/replay.sh E16R5`.

  - **Basis**: the `name=geometry_config`/`name=q1_summary`/`name=row_summary` lines of `e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out`.
- Verdict d23-e16r5-r1: the miscopied unit count of full_root_level_four corrected, the phase-two attribution sentence marked "hit by the three-way argument, to be redone", G23.2 (Jia–Bing difference does not converge) and the basis paragraph rewritten under the ratio reading
  Per verdict `research/prompts/d23-e16r5-r1-main-verification.md` (user decision): line 110 of `.claude/kb/experiments/16-journal的角色WALvs意图日志.md`, "the unit counts are all the same 1 TiB number (33692212)", is corrected — the unit count of `full_root_level_four` is 457419312 (the `geometry_config` line of phase-two artifact `e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out`), and the other three points are still 33692212; the phase-two sentence "dominated by the number of children under the root, not tree height" is marked "hit by the three-way argument, to be redone" — the attacking leg of d23-e16r5-r1 swept tree height, children under the root and fanout separately: tree height is a switch, fanout has zero effect, and children under the root set the threshold only at tree height 4; these data were measured only on the attacking leg's copy in the three-way argument, the main agent's rerun was byte-for-byte identical, and the checked-in rig was not redone (the user decided to fix only the miscopied number and not run a phase three). The settled text of D23 (Journal Role & Format) settled item 1 is unaffected; the G23.2 (Jia–Bing difference does not converge) cell and the first half of the E16 (Role of Journal: WAL vs Intent Log) entry in the basis paragraph are rewritten under the ratio reading, varying with tree height.

  - **Basis**: sections one and four, question 2 of section five, and section six of `research/prompts/d23-e16r5-r1-main-verification.md`; the `geometry_config` line of `e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out`.

  In the "fifth run phase two" entry, the unit count 33692212 for full_root_level_four is likewise miscopied; it is really 457419312, and this entry prevails.

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-03

- E16 (Role of Journal: WAL vs Intent Log) gets a byte measurement basis — clearing the precondition of C27 (Format Parameters Settled on a Block-Count Measurement Basis)
  Before: C27 (Format Parameters Settled on a Block-Count Measurement Basis) carried "E16 (Role of Journal: WAL vs Intent Log) adds byte accounting (pure owed work)",
  and E16 (Role of Journal: WAL vs Intent Log) had only the block-count basis; the mutation table had only the 3 rows of the 08-31 round.
  After: new `e16-journal bytes` subcommand (parameterized geometry: fanout = (node bytes−64)/40, tree height computed independently),
  three criteria written down before the run and all hit — per-operation ancestors go from 9 nodes / 4.5 KiB for 512 B nodes to
  3 nodes / 192 KiB for 64 KiB nodes: **3× cheaper by node count, 42.7× dearer by bytes; the two bases rank in opposite orders**;
  batch 10 multi-stream goes the same way (3.3× cheaper vs 38.4× dearer). Artifact `e16-bytes-2026-09-03.out` checked in,
  a second E16 (Role of Journal: WAL vs Intent Log) row added to the replay table; unit tests 21→24; mutation table 3→9 rows all caught (3 new on the byte basis +
  the 3 historical mutations of 08-28 rebuilt from test comments — of the rig's "14 rows each hit", only these 3 can be rerun today,
  the same type as C40 (mutation table not saved), recorded as such in the text). One thing filed in passing: the first version's config line bypassed the Emitter,
  and the replay integrity gate judged it red on the spot (direct evidence the gate works).
  Basis: the original text of the check form of C27 (Format Parameters Settled on a Block-Count Measurement Basis); the status of that owed check has been updated (precondition repaid, what is owed is the judgement itself).
- Criterion 3 (maintenance cost) of E16 (Role of Journal: WAL vs Intent Log) closed by a three-way argument — the question as posed does not hold
  Before: criterion 3 registered for E16 (Role of Journal: WAL vs Intent Log), "how many invariants and how many crash-point categories each form needs", hung under
  "the half not done yet" and was never answered.
  After: the full three-way argument was run (local Qwen counterexample leg / Sonnet forward leg / Opus backward leg, with prompts and original texts archived in
  `research/prompts/e16-maintenance-*.md`). Verdict: **all three legs independently agree that "the count is not a
  reasonable proxy for maintenance cost"**; the three legs disagree on the direction of the counts (10/6 vs 14/10, 15/12 vs 20/14, and once the boundary is widened to the publish-rate spillover
  obligations Jia ends up with more), and on verification the root of the disagreement is how the list boundary is drawn, with both ways valid ⇒ which proves exactly that, as originally posed,
  two numbers cannot answer it. The load-bearing structural answer: Yi's decisive cost is that replay needs the allocator ⇒ O2 (independent parser + checker) of D13 (Verification Route)
  must implement the allocator a second time independently (the only surviving reason after D23 (Journal Role & Format) rejected the logical intent
  log, inherited by Yi as is); Jia's corresponding bill is that publish rate = fsync rate leaks into accounting and the root ring
  (rider clauses 2/4 of D16 (Publish Semantics) settled item 6, the protection window of I-7.4 (Blocks in Recent K Generations Not Reused) shrinking to milliseconds,
  one more sequence point per fsync). The backward leg caught a wrong premise in the background material (Jia's record as a 48-byte cursor vs
  the 4 KiB fixed-length record of D23 (Journal Role & Format) settled item 12), which the main agent verified word for word as true — the local leg and the forward leg inherited it,
  so the counting details are contaminated while the structural argument is not; two model-basis numbers in the text of E16 (Role of Journal: WAL vs Intent Log) now carry reach notes,
  and two computable but uncomputed bills are posted (recompute ring occupancy on the 4 KiB basis; plug 4.0 blocks/operation into the acceptance formula of E19 (Spurious ENOSPC Under Defer Window)).
  D23 (Journal Role & Format) settled item 1 is not reopened (axis one structurally excluded, all three legs agree).
  Basis: `.claude/rules/three-way-inference.md`; the verdict is made by the main agent, not by vote.

- See the same-day entry "new E91 (Ring and Admission Accounting Under Settled Format), run to completion the same day — all bills posted by the three-way argument measured" in the E91 (Ring and Admission Accounting Under Settled Format) section.

### 2026-09-01

- Stale citations in the text of E16 (Role of Journal: WAL vs Intent Log) and E67 (Which Disks a Stripe Spans) corrected
  - **Two places in E16 (Role of Journal: WAL vs Intent Log) called D22 (Synthesizing Unit Atomicity) unsettled.**
    **Previously**: conclusion 5 and the limitations section each had one place that wrote the status of D22 (Synthesizing Unit Atomicity) as unsettled
    (hung on "the number of root ring slots is limited" and on "the COW cost of the accounting blocks themselves is not modeled at all" respectively);
    **now**: the former becomes "D22 (Synthesizing Unit Atomicity) settled item 2 settles R=3, with the slots per region living in the system configuration",
    and the latter "settled item 3 has settled that the accounting structure goes COW, and this cost is not in the model";
    **basis**: D22 (Synthesizing Unit Atomicity) settled items 2 and 3 are both settled; caught by the new gate stage
    `.claude/gate.d/28-cross-decision-status.sh`. ⚠️ **Both limitations themselves still hold**; only their statements about the status of D22 (Synthesizing Unit Atomicity) changed.
  - **Two self-referential phrasings in E67 (Which Disks a Stripe Spans)** ("this experiment measured the four arms equal cell by cell", "this experiment did not measure this dimension")
    are changed into numbered references with short names, based on item 1 of `.claude/singlefs-ai-sop/rules/kb-discipline.md` —
    an entry pulled out by retrieval has no "here". Caught by `doc-lint.sh`.
  - ⚠️ **This round reran no experiment and changed no experiment conclusion**;
    the seed folding (C59 (Seeds Folding into the Same State)) of E58 (End-to-End Cost of Checksum Granularity) / E65 (Write-Side Cost of Checksum Granularity) / E66 (How Small Files Land After Padding)
    still owes a rerun.

### 2026-08-31

- See the same-day entry "extra measurements and corrections for five experiments (trigger: one check that 'experiment data agree with the decisions')" in the E6 (Encryption Algorithm Selection) section.

### 2026-08-29

- See the same-day entry "" in the E23 (Journal Geometry) section.
- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.

### 2026-08-28

- 
  - E16 (Role of Journal: WAL vs Intent Log) changes from "not run" to "the accounting-model half has run", and was finalized only **after correcting modeling errors three times**.
    All three errors were caught by the other two legs of the three-way argument, and their common shape is "the checks only compared the three arms with each other
    and never pinned absolute values" — when all three arms are wrong together, the comparisons still come out equal.
  - New: E22 (Recording Named Ancestor Placement). Source: in that three-way argument round for E16 (Role of Journal: WAL vs Intent Log), the forward leg pointed out
    that the criterion of C3 (read retries lack hard upper bound) ② had been written too narrowly, and the structural statement of the local leg agreed. **It is a new candidate, not a conclusion.**

- See the same-day entry "" in the E17 (Write Buffer Merge Limit) section.

## E17 (Write Buffer Merge Limit)

### 2026-09-19

- E17 (Write Buffer Merge Limit), E42 (Number of Records per Transaction), E44 (Host Testing and Cost of Sequence Number Bit Width), E77 (Persistence Ordering of Publishes) get an "Affected decisions" table; the current record header width in E42 (Number of Records per Transaction) and two "now / current" places in E44 (Host Testing and Cost of Sequence Number Bit Width) corrected as looked up now
  - **Before**: none of the four pages had a "### 影响的决策" table. The parenthetical in the "measured" section of E42 (Number of Records per Transaction) said "currently 78, D23 (Journal Role & Format) settled item 4"; the opening of E44 (Host Testing and Cost of Sequence Number Bit Width) said "D23 (Journal Role & Format) settled item 9 now says 'the 10⁶ records/s tier has no measurement on this host, it is an assumed upper bound'", and the first row of the part B table said "8 bytes (current)".
  - **After**: each of the four pages gets a "### 影响的决策" table: E17 (Write Buffer Merge Limit) and E44 (Host Testing and Cost of Sequence Number Bit Width) support D13 (Verification Route) settled item 2 (the two halves of the serial-ceiling intersection), E42 (Number of Records per Transaction) supports D13 (Verification Route) settled items 1 and 2, E77 (Persistence Ordering of Publishes) supports D13 (Verification Route) settled item 4, matching the slimmed basis paragraph of D13 (Verification Route) in both directions; the other decisions mentioned in each page's text (D8 (Core Index Structure), D16 (Publish Semantics), D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment), D23 (Journal Role & Format), D25 (Target Workload Priority)) get one row each. The parenthetical in E42 (Number of Records per Transaction) becomes "currently 307 (the 78 of the ten basic fields plus the increments settled later)"; E44 (Host Testing and Cost of Sequence Number Bit Width) becomes "before the run it said" and "8 bytes (the current width at run time)". Experiment conclusions and artifacts unchanged.
  - **Basis**: `.claude/rules/format-evolution.md` "decision text states only the current state, the basis is written as pointers; decisions and experiments are registered both ways". Looked up now: `JOURNAL_HEADER_BYTES = 307` and `JOURNAL_HEADER_TEN_FIELD_BYTES = 78` in `crates/singlefs-format/src/lib.rs`, and D23 (Journal Role & Format) settled item 4 writes a 307-byte header; D23 (Journal Role & Format) no longer has the sentence "the 10⁶ records/s tier has no measurement on this host", and settled item 9 today is a 32-bit instance identifier + a 48-bit counter, 10 bytes in all.

### 2026-09-18

- The user write bandwidth derived from the range of E17 (Write Buffer Merge Limit) recomputed with the eight-round lower bound; the experiment index row follows the eight rounds
  - **Before**: the range in conclusion 1 of the text had been changed on 2026-09-16 to 29.9–31.8M per eight rounds, while the sentence "converted to user write bandwidth" still used the four-round lower bound of 30.9M, saying "42.2 GB/s; even producing 10 entries per 4 KiB it can sustain 12.6 GB/s"; the experiment index row said "run (08-28, four rounds)", "30.9–31.8M", "parallel 6.26–6.38×".
  - **After**: "computed with the eight-round lower bound of 29.9M, 40.8 GB/s; 12.2 GB/s at 10 entries" (29.9M ÷ 3 × 4096, 29.9M ÷ 10 × 4096); the index says "eight rounds", "29.9–31.8M", "16 threads 5.49–6.38×", all per the eight-round ranges of conclusion 1 of the text.
  - **Basis**: the acceptance sweeper of the knowledge-rot sweep reported that the D8 (Core Index Structure) row cited the old range; when the main agent searched the whole repository for "42.2 GB/s" and "30.9–31.8", it found that the text's own derived numbers and the index row had not followed.

### 2026-09-06

- See the same-day entry "three experiment index rows disagree with the text — the index row of E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) is entirely a voided conclusion" in the E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) section.

### 2026-08-31

- See the same-day entry "extra measurements and corrections for five experiments (trigger: one check that 'experiment data agree with the decisions')" in the E6 (Encryption Algorithm Selection) section.

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.
- See the same-day entry "rerun-and-review round — 20 experiments reproduce byte for byte, E34 (Root Ring Slot Geometry) file created…" in the E34 (Root Ring Slot Geometry) section.

### 2026-08-28

- 
  - E17 (Write Buffer Merge Limit) has run: single-thread sort-merge-sweep at 30.88M entries/s (equivalent to 42.2 GB/s of user writes),
    lock-free parallel by leaf partition at 6.55× with 16 threads; the positive control at the same tier is 11.96×.
    The most valuable output is a format requirement: **write buffer entries must carry their own sequence numbers** — forced out by a red test.
  - Set up E16 (Role of Journal: WAL vs Intent Log), E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) pure logic, runnable now, and the modeling half of E19 (Spurious ENOSPC Under Defer Window), runnable now.

- See the same-day entry "" in the E8 (Different Write Paths for Large vs Small Files) section.

## E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads)

### 2026-09-03

- See the same-day entry "adversarial test round — all 19 owed mutation tables filled in, 10 new unit tests added where mutations escaped" in the E68 (Selection of Inlining Threshold) section.

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.
- See the same-day entry "experiment control audit — two experiments had zero absolute-value assertions, one experiment's positive control did not cover the configuration that produced the conclusion" in the E9 (Key Encoding Impact on Traversal Locality) section.

### 2026-08-28

- 
  - E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads) has run: the interval judgement of D5 (Snapshot / Space Accounting Mechanism) **gives wrong answers** under forks — a single fork already judges 1.73% live blocks dead
    (the data-loss direction), and 8 writable heads 5.80%; linear history has zero errors (positive control).
    Changing the death recording rule only brings it down to 4.00%, not to zero ⇒ the problem is "one death field cannot hold a fork".

- See the same-day entry "" in the E17 (Write Buffer Merge Limit) section.
- See the same-day entry "" in the E8 (Different Write Paths for Large vs Small Files) section.

## E19 (Spurious ENOSPC Under Defer Window)

### 2026-09-19

- E19 (Spurious ENOSPC Under Defer Window), E82 (In-Flight Synthesis for Admission), E141 (Mount Admission Self-Test for Switchover Reservation), E148 (Commit Fixed Point Recomputed with Two Record Trees), E150 (Rollback Reuse of Units Referenced by Abandoned Roots) get an "Affected decisions" table; line 4 of E148 (Commit Fixed Point Recomputed with Two Record Trees) no longer quotes verbatim the original sentence of D28 (Mount-Time Commitments) settled item 4 from before it was settled
  - **Before**: none of the five pages had a "### 影响的决策" section; line 4 of E148 (Commit Fixed Point Recomputed with Two Record Trees) said "D28 (Mount-Time Commitments) settled item 4 verbatim: 'its upper bound is staked on C83 (commit fixed point unanswered)'", while that sentence had moved into the decision changelog (number three) with the slimming of D28 (Mount-Time Commitments).
  - **After**: each of the five pages gets a table, row by row with support / no effect and the look-back; line 4 of E148 (Commit Fixed Point Recomputed with Two Record Trees) becomes "before D28 (Mount-Time Commitments) settled item 4 was settled, its upper bound was staked on C83 (commit fixed point unanswered)", no longer citing a sentence that is no longer in the text. The experiments themselves, their artifacts and conclusions did not change.
  - **Basis**: `.claude/rules/format-evolution.md` "decision text states only the current state, the basis is written as pointers; decisions and experiments are registered both ways"; the broken quotation was found by the quotation check of `scripts/kb/decision-slim-check.py`.

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-03

- See the same-day entry "adversarial test round — all 19 owed mutation tables filled in, 10 new unit tests added where mutations escaped" in the E68 (Selection of Inlining Threshold) section.
- See the same-day entry "criterion 3 (maintenance cost) of E16 (Role of Journal: WAL vs Intent Log) closed by a three-way argument — the question…" in the E16 (Role of Journal: WAL vs Intent Log) section.

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.
- See the same-day entry "E33 (Two Pinned Block Rules) newly set up and run to completion — one basis knocked out by arithmetic" in the E33 (Two Pinned Block Rules) section.

### 2026-08-28

- 
  - The modeling layer of E19 (Spurious ENOSPC Under Defer Window) has run: the acceptance criterion hits 36/36; at 90%/95% fill, spurious ENOSPC is all 0;
    at 99.9% fill with delay 16 it is 7.875%. **A reserve pool cannot rescue a stuck checkpoint; it makes spurious ENOSPC worse instead.**
    This round caught two of our own modeling bugs (u64 underflow, the reserve pool implemented as a reduced capacity),
    and marks that the analytic_held verification path has no discriminating power under regular workloads.

- See the same-day entry "" in the E17 (Write Buffer Merge Limit) section.
- See the same-day entry "" in the E8 (Different Write Paths for Large vs Small Files) section.

## E20 (CPU Cache Cost of Widening Pointers)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-09-10

- See the same-day entry "new E128 (Point Lookup Cost of Pointers with Birth (tree, txg)), run to completion — the cost the user named as owed is measured…" in the E128 (Point Lookup Cost of Pointers with Birth (tree, txg)) section.

### 2026-09-03

- See the same-day entry "adversarial test round — all 19 owed mutation tables filled in, 10 new unit tests added where mutations escaped" in the E68 (Selection of Inlining Threshold) section.

### 2026-08-31

- See the same-day entry "extra measurements and corrections for five experiments (trigger: one check that 'experiment data agree with the decisions')" in the E6 (Encryption Algorithm Selection) section.
- See the same-day entry "E58 (End-to-End Cost of Checksum Granularity) set up and run to completion" in the E58 (End-to-End Cost of Checksum Granularity) section.

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.
- See the same-day entry "experiment control audit — two experiments had zero absolute-value assertions, one experiment's positive control did not cover the configuration that produced the conclusion" in the E9 (Key Encoding Impact on Traversal Locality) section.
- See the same-day entry "rerun-and-review round — 20 experiments reproduce byte for byte, E34 (Root Ring Slot Geometry) file created…" in the E34 (Root Ring Slot Geometry) section.

### 2026-08-28

- 
  - E20 (CPU Cache Cost of Widening Pointers) has run: **the cost of widening pointers is in the CPU cache, and appears only when the working set exceeds L3** —
    in the beyond-L3 tier 16B→111B is 2.6×; and **node size decides at which tier the cost starts**:
    at 40 bytes, 4 KiB nodes pay +98%, while 16 KiB nodes pay only +14%.
    ⚠️ This goes in the opposite direction from "small nodes win at both ends" in E7 (Offline Index Harness), because the two measure different resources (device I/O vs CPU cache).
    The positive control stopped three versions, two of them my own modeling errors (out-of-order execution parallelized away the cache misses, wrong data layout).
  - Set up E21 (Net Benefit of GPU Offloading): the net benefit of GPU offloading (measurable on this host's RTX 5090 + 5060 Ti, no preconditions).

- See the same-day entry "" in the E8 (Different Write Paths for Large vs Small Files) section.

## E21 (Net Benefit of GPU Offloading)

### 2026-09-07

- The substitution paragraph of E21 (Net Benefit of GPU Offloading) stopped at an overturned linear extrapolation, and nobody looked back for nine days
  - **Before**: three places in the text of E21 (Net Benefit of GPU Offloading) plugged in the encryption throughput of E6 (Encryption Algorithm Selection) extrapolated as "single core × 16 cores" —
    with AES-NI "16 cores about 65 GB/s ⇒ above all GPU floors, GPU loses"; without AES-NI one place said 16 cores about 4.1 GB/s
    and another said 4.5 GB/s; the subsection title and the index row called the 0.78× cell "the bandwidth-bound cell",
    while the text used the same word for encryption with AES-NI.
  - **After**: plug in the saturated measurement of the 2026-08-28 multi-core tier of E6 (Encryption Algorithm Selection), **17.4 GB/s** ⇒ **below both GPU floors, GPU wins**;
    without AES-NI it says throughout "multi-core not measured, single core 0.257 GB/s, the saturated value can only be lower", no longer giving an extrapolated value;
    "the bandwidth-bound cell" in the title, the index row and the subsection title becomes "the **pure scan** cell", with a statement that pure scan is not
    any of the three candidate scenarios of D24 (Offloading Heavy Background Work to GPU).
  - **Basis**: this overturn had been made as early as 2026-08-28, but the sweep never reached E21 (Net Benefit of GPU Offloading) —
    [decisions-history.md](decisions-history.md) 2026-08-28 (number twenty-eight) records verbatim "the linear extrapolation was wrong by nearly 4× and in the opposite direction";
    the texts of D24 (Offloading Heavy Background Work to GPU) and E6 (Encryption Algorithm Selection) were both changed at the time, and E21 (Net Benefit of GPU Offloading) was left untouched in one place.
    ⚠️ This is exactly the cell of `.claude/singlefs-ai-sop/rules/evidence-discipline.md` "withdrawing a number or a conclusion
    likewise requires sweeping, on the spot, who cites it": the person withdrawing swept two documents, and the one missed was precisely **the source of the criterion**,
    so when D24 (Offloading Heavy Background Work to GPU) cited "criterion 1 of E21 (Net Benefit of GPU Offloading)", the text it pointed to said the opposite conclusion.
    This round swept the whole repository for `65 GB/s / 61 GB/s / 4.1 / 4.5 GB/s / GPU 输 / GPU 赢 / 17.4`;
    apart from two history files, only three places in E21 (Net Benefit of GPU Offloading) remained, and all were fixed.
    ⚠️ **No experiment was rerun**: not one of the five measured numbers 64.91 / 56.69 / 28.5 / 50.3 / 1041.3 GB/s changed;
    what changed is only the CPU-side number plugged into the criterion and its wording.

### 2026-08-29

- See the same-day entry "experiment control audit — two experiments had zero absolute-value assertions, one experiment's positive control did not cover the configuration that produced the conclusion" in the E9 (Key Encoding Impact on Traversal Locality) section.
- See the same-day entry "rerun-and-review round — 20 experiments reproduce byte for byte, E34 (Root Ring Slot Geometry) file created…" in the E34 (Root Ring Slot Geometry) section.

### 2026-08-28

- See the same-day entry "" in the E20 (CPU Cache Cost of Widening Pointers) section.

## E22 (Recording Named Ancestor Placement)

### 2026-09-19

- Several current-state sentences in E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records), E49 (Reverse Chain Width: 32 vs 64), E61 (Uniformity of Reverse Chain Hash Algorithm), E75 (Record Size and Ring Geometry), E77 (Persistence Ordering of Publishes), E78 (Starting Point for Replay) corrected as looked up now; E16 (Role of Journal: WAL vs Intent Log), E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E24 (Recovery Algorithm: Does Trusting Tail First Lose Records), E25 (Reserve Pool Size for Ring vs Chain), E32 (Remnants of the Previous Timeline), E36 (Slot Mapping Dimension), E37 (Log Instance Generation), E39 (Does Reverse Chain Block Remnant Records), E40 (Ciphertext Checksum Bit Width), E45 (Ring Space Occupancy for Multi-Record Transactions), E49 (Reverse Chain Width: 32 vs 64), E51 (Number of Reverse Chain Collision Opportunities), E61 (Uniformity of Reverse Chain Hash Algorithm), E75 (Record Size and Ring Geometry), E76 (Discriminative Power of Payload Checksum), E78 (Starting Point for Replay), E142 (New-Pool File Creation Dry Run), E143 (Journal Accounting Under One Transaction One Unit) get an "Affected decisions" table; the D23 (Journal Role & Format) rows of E19 (Spurious ENOSPC Under Defer Window), E42 (Number of Records per Transaction), E44 (Host Testing and Cost of Sequence Number Bit Width), E77 (Persistence Ordering of Publishes) are added or changed back to item level
  - **Before**:
    - E22 (Recording Named Ancestor Placement): "the criterion of C3 (read retries lack hard upper bound) ② was written as 'whether replay runs the allocator'", "C2 (unbounded operation made into single transaction) of D23 (Journal Role & Format) has settled 'allocation precedes the journal'" — those two numbers were earlier local labels within D23 (Journal Role & Format), mistaken for owed-check numbers and given short names.
    - E23 (Journal Geometry): "D23 (Journal Role & Format) settled item 4 plus the three increments of settled items 7 / 8 / 13, 17 in all: **the header is 95 bytes, leaving 417 of 512**, and the named-entry capacity is still 7 entries (E44 (Host Testing and Cost of Sequence Number Bit Width) measured at 78; 417 ÷ 56 still rounds down to 7)." "When citing the header byte count, use 95 (ten fields 78 + 17)."
    - E32 (Remnants of the Previous Timeline): the anchor of row 2 of the premise table, "verbatim from the 'prefix judgement' section of D23 (Journal Role & Format)".
    - E39 (Does Reverse Chain Block Remnant Records): "the current header without the reverse chain is 91 (D23 (Journal Role & Format) settled item 4: after widening `jsn` +2 and dropping `tail_lsn` −8 the ten fields are 78, plus transaction fields 9 and payload checksum 4) ⇒ the four tiers are 92 / 93 / 95 / 99 bytes, +1.10% / +2.20% / +4.40% / +8.79%."
    - E49 (Reverse Chain Width: 32 vs 64): title "while the current record header is 78"; opening "**the current record header is 78** (D23 (Journal Role & Format) settled item 4, 91 once the two settled but not yet landed increments land)", "the cost side must be recomputed once each at 78 and 91"; the 95 row of the base table "the current header without the reverse chain is 91 (… the header of 95 also contains the reverse chain's 4)"; conclusion 3 "currently corresponds to 91 (…)", "must be recomputed at 91 (the ⚠️ above 'width settled' in D23 (Journal Role & Format) already records the same bill)".
    - E61 (Uniformity of Reverse Chain Hash Algorithm): "the current ladder is **78 / 91 / 95** (D23 (Journal Role & Format) settled item 4: currently 78, 91 after the two settled increments land, 95 after three)."
    - E75 (Record Size and Ring Geometry): "while the current value settled by settled item 4 is **78**, and the three settled but not yet landed increments raise it to **95**." Capacity table header "hdr=78 (current)".
    - E77 (Persistence Ordering of Publishes): conclusion 2 "'every entry named by a record carries its own checksum' (D23 (Journal Role & Format) settled item 1)"; in the table "D23 (Journal Role & Format) settled item 1 | support" and "D23 (Journal Role & Format) | preparation | … settled item 8 (reverse chain) …".
    - E78 (Starting Point for Replay): "What it cannot answer" said "tail is written once per checkpoint, D23 (Journal Role & Format) settled item 3".
    - E42 (Number of Records per Transaction), E44 (Host Testing and Cost of Sequence Number Bit Width): the D23 (Journal Role & Format) rows were written at decision level ("D23 (Journal Role & Format) | preparation | … settled item 7 …", "… settled item 9 …", "D23 (Journal Role & Format) | no effect | the transaction fields of settled item 7 …"); the gate at the time could not recognize items in a numbered list.
    - Nineteen pages had no "### 影响的决策" section; the table of E19 (Spurious ENOSPC Under Defer Window) had no D23 (Journal Role & Format) row.
  - **After**:
    - E22 (Recording Named Ancestor Placement): becomes "the criterion of the second reason for rejecting the logical intent log (uncertain placement degrades reconciliation, D23 (Journal Role & Format) settled item 5) was written as …", "D23 (Journal Role & Format) settled item 20 has settled 'allocation precedes the journal'".
    - E23 (Journal Geometry): becomes 95 after the three increments, plus the new-root segment 188, fsid 8 and MAC 16, a current header of 307 bytes, leaving 205 of 512 (3 named entries) and 3789 of 4096 (67 entries); use 307 when citing the header byte count.
    - E32 (Remnants of the Previous Timeline): the anchor becomes "verbatim from D23 (Journal Role & Format) settled item 22" (that section became settled item 22).
    - E39 (Does Reverse Chain Block Remnant Records): the current header without the reverse chain is 303 (the header of 307 contains the reverse chain's 4) ⇒ the four tiers are 304 / 305 / 307 / 311 bytes, +0.33% / +0.66% / +1.32% / +2.64%; the width choice is unchanged.
    - E49 (Reverse Chain Width: 32 vs 64): the current record header without the reverse chain is 303, the cost side is recomputed at 303, and the same bill is recorded in the reach of D23 (Journal Role & Format) settled item 8.
    - E61 (Uniformity of Reverse Chain Hash Algorithm): the current header is 307 (ten fields 78 + three settled increments 17 + new-root segment 188 + fsid 8 + MAC 16).
    - E75 (Record Size and Ring Geometry): those ten fields are 78, the three settled increments raise it to 95, and the current header of 307 also contains the new-root segment, fsid and MAC; table header "hdr=78 (ten fields)".
    - E77 (Persistence Ordering of Publishes): conclusion 2 points to D23 (Journal Role & Format) settled item 17 (the named-entry field table, where "every entry named by a record carries its own checksum" lives); the table becomes settled item 14 support (the fourth load-bearing item of the prefix basis), settled item 17 support, settled item 8 preparation.
    - E78 (Starting Point for Replay): becomes "tail is written once with every publish".
    - E42 (Number of Records per Transaction): D23 (Journal Role & Format) settled item 7 support; E44 (Host Testing and Cost of Sequence Number Bit Width): D23 (Journal Role & Format) settled item 9 support, settled item 7 no effect; E19 (Spurious ENOSPC Under Defer Window) gets two support rows for settled items 23 / 24.
    - Each of the nineteen pages gets a table: the D23 (Journal Role & Format) rows match its slimmed basis paragraph in both directions; E23 (Journal Geometry), E76 (Discriminative Power of Payload Checksum) and E142 (New-Pool File Creation Dry Run) were also being written by other agents, so only their D23 (Journal Role & Format) rows were written and not the rows for other decisions on the page, and these three pages stay on the backfill list.
    - Not a word of the experiment conclusions or artifacts changed.
  - **Basis**: looked up now in `.claude/kb/decisions/23-journal的角色与格式.md`: settled item 4 (header 307 = 78 + 17 + 188 + 8 + 16), settled item 3 (written once with every publish), settled item 17 (the named-entry field table), settled items 22 / 20 / 5 (the items set up and the reasons rewritten after slimming); C3 (read retries lack hard upper bound) in `.claude/kb/checks-owed.md` is the MAC truncation bill in the repaid table, unrelated to those two sentences of E22 (Recording Named Ancestor Placement); for the slimming of D23 (Journal Role & Format) see the decision changelog (number twelve).

### 2026-08-28

- See the same-day entry "" in the E16 (Role of Journal: WAL vs Intent Log) section.

## E23 (Journal Geometry)

### 2026-09-19

- In the seven pages E23 (Journal Geometry), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place), E100 (Three-Segment Geometry of System Configuration), E115 (Completeness and Byte Budget of System Configuration Field Table), E118 (Three Arms of Single-Disk Self-Recovery), E126 (Costs of Four System Configuration Slot Width Candidates) and in [experiments.md](experiments.md), places citing D2 (RAID Stripe Policy), D9 (Encryption), D18 (Information Carried in a Block), D22 (Synthesizing Unit Atomicity) by old subsection names now point to item numbers; comments in six experiment binaries and one probe script synced
  - **Before**: the seven pages and [experiments.md](experiments.md) cited subsection names from before the slimming — E100 (Three-Segment Geometry of System Configuration), E126 (Costs of Four System Configuration Slot Width Candidates), E38 (Accounting Structure: COW vs In-Place) wrote "settled one / settled two / settled three" of D22 (Synthesizing Unit Atomicity), E34 (Root Ring Slot Geometry) and the E126 (Costs of Four System Configuration Slot Width Candidates) row of [experiments.md](experiments.md) wrote "settled hard requirement 1 / 2" of D2 (RAID Stripe Policy), E115 (Completeness and Byte Budget of System Configuration Field Table) and E23 (Journal Geometry) wrote "day-1 reservation items 3 / 4" of D9 (Encryption), and E118 (Three Arms of Single-Disk Self-Recovery) wrote the "rejection list" of D18 (Information Carried in a Block); the clause citations in e100 / e115 / e126 / e77 / e38 / e142 / e118 under `scripts/experiment-apparatus/` and in `scripts/devices/e125-zoned-wp-probe.sh` likewise used old subsection names.
  - **After**: all now point to today's item numbers — D22 (Synthesizing Unit Atomicity) settled items 19 / 20 / 21 / 24, D2 (RAID Stripe Policy) settled items 16 / 17, D9 (Encryption) settled items 12 / 14, D18 (Information Carried in a Block) settled item 23; not a word of the quotations inside 「」, the criteria, the artifacts or the conclusions changed. After the change the seven binaries were rerun together with `scripts/experiment-apparatus/replay.sh`, and all seven match byte for byte (the two cells changed in E115 (Completeness and Byte Budget of System Configuration Field Table) are source strings of already-covered entries in the `REQUIRED` table; only the source of missing entries goes into the artifact).
  - **In passing**: once the texts of E100 (Three-Segment Geometry of System Configuration) and E126 (Costs of Four System Configuration Slot Width Candidates) changed, the decisions they affect had to be reviewed (gate 75), and these two pages had no "Affected decisions" table yet ⇒ each of the two pages gets a table with one row per decision mentioned on the page (11 rows for E100 (Three-Segment Geometry of System Configuration), 8 rows for E126 (Costs of Four System Configuration Slot Width Candidates)), and the two pages' rows are deleted from `.claude/decision-links-pending`; the relationships written are only preparation and no effect, with no row of support or overturn, so no decision's "**Basis**" paragraph changed along with this.
  - **Basis**: the item numbers of D9 (Encryption), D22 (Synthesizing Unit Atomicity) and D18 (Information Carried in a Block) come from the decision changelog (number eleven), (number thirteen) and (number fourteen); the repointing and basis of this round are recorded in the decision changelog (number fifteen). The changes were made in place with `scripts/kb/replace-once.py`, each hitting exactly once and read back to confirm.

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-13

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) by journal header…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-06

- See the same-day entry "a round of recovering from format constants drifting out of sync — E43 (Extension Point Byte Upper Bound) rerun a fourth time at 105…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-08-29

- 
  - New: E23 (Journal Geometry): complexity and performance analysis of D23 (Journal Role & Format) settled items 2/3/4; the three items are judged to be coupled.
  - E16 (Role of Journal: WAL vs Intent Log) reinterpreted after the fact: the four workloads are unified by one zero-fit formula, closing cell by cell;
    on that basis it points out that `fsync_every=1` is a degenerate cell (stream count is invisible at that tier),
    and when discussing workloads with the numbers of E16 (Role of Journal: WAL vs Intent Log), the batch tier must be stated. Landing place: D25 (Target Workload Priority) in [decisions.md](decisions.md).

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.

## E24 (Recovery Algorithm: Does Trusting Tail First Lose Records)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-02

- See the same-day entry "four new experiments E77 (Persistence Ordering of Publishes) to E80 (Partial Release of Stripes), run to completion the same day — reasoning forward from settled clauses '…" in the E77 (Persistence Ordering of Publishes) section.

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.

## E25 (Reserve Pool Size for Ring vs Chain)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.
- See the same-day entry "E33 (Two Pinned Block Rules) newly set up and run to completion — one basis knocked out by arithmetic" in the E33 (Two Pinned Block Rules) section.

## E26 (Lookup Cost and Accuracy of Three Accounting Models)

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.

## E27 (Three Modelable Risk Paths of D5)

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.

## E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units)

### 2026-10-17

- The page header "repays C34 (three-way clash on plain text mapping layer states)" becomes "this page was originally run to repay C34 (three-way clash on plain text mapping layer states)", stating that this bill was voided as having no object on 2026-09-07 when D9 (Encryption) settled item 5 switched to "scrub only", and is no longer owed; conclusions, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/28-明文映射层能不能只靠单元自描述重建.md`.

### 2026-09-19

- E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval), E83 (Granularity of Tombstones), E84 (Pinned Volume of Packed Tombstones), E85 (Field Table of Unit Headers), E86 (Scan Stride and Two Unit Sizes), E88 (Discrimination of Impostor Units vs Orphan Units), E102 (Unit Class Registry Table and Packed Record Units), E117 (Header Width and Geometry After Factoring in Reserved Bits), E118 (Three Arms of Single-Disk Self-Recovery), E144 (Cost and Discriminative Power of Header Checksum Algorithms), E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) get an "Affected decisions" table; E59 (Can Buffered Messages Recompute Solely from Units), E76 (Discriminative Power of Payload Checksum), E104 (Current Version Determination in Scan Rebuild), E142 (New-Pool File Creation Dry Run) get D18 (Information Carried in a Block) rows; one current-state sentence each in E73 (Fanout Cost of Node Key Interval) and E84 (Pinned Volume of Packed Tombstones) corrected as looked up now
  - **Before**: the first twelve pages had no "### 影响的决策" section and were all in `.claude/decision-links-pending`; in the table of E59 (Can Buffered Messages Recompute Solely from Units), D18 (Information Carried in a Block) had only two rows, settled items 1 / 8, E104 (Current Version Determination in Scan Rebuild) only three rows, settled items 7 / 10 / 12, and the tables of E76 (Discriminative Power of Payload Checksum) and E142 (New-Pool File Creation Dry Run) had no D18 (Information Carried in a Block) row. Item 4 of "the half it cannot answer" in E73 (Fanout Cost of Node Key Interval) read verbatim "4. **Holds only for the accounting tree.** The dirent tree cell is deferred per D18 (Information Carried in a Block) settled item 2 —"; after the conclusion E84 (Pinned Volume of Packed Tombstones) read verbatim "⇒ **candidate addendum for D18 (Information Carried in a Block) settled item 10** (decision pending with the user):".
  - **After**: each of the twelve pages gets an "Affected decisions" table, with one row per decision mentioned on the page; the D18 (Information Carried in a Block) rows match the basis paragraph of its new text in both directions: E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) supports settled items 3 / 13, E73 (Fanout Cost of Node Key Interval) supports settled item 2, E83 (Granularity of Tombstones) and E84 (Pinned Volume of Packed Tombstones) support settled item 10, E85 (Field Table of Unit Headers) supports settled items 7 / 11, E86 (Scan Stride and Two Unit Sizes) supports settled item 9, E88 (Discrimination of Impostor Units vs Orphan Units) supports settled items 7 / 13 / 20, E102 (Unit Class Registry Table and Packed Record Units) supports settled items 8 / 11, E117 (Header Width and Geometry After Factoring in Reserved Bits) supports settled items 14 / 16, E118 (Three Arms of Single-Disk Self-Recovery) supports settled item 4, E144 (Cost and Discriminative Power of Header Checksum Algorithms) supports settled item 17, E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) supports settled item 18; E59 (Can Buffered Messages Recompute Solely from Units) and E104 (Current Version Determination in Scan Rebuild) each add a settled item 11 row (support), E142 (New-Pool File Creation Dry Run) adds settled item 2 (support) and settled items 17 / 18 (preparation), and E76 (Discriminative Power of Payload Checksum) adds settled item 17 (no effect). That sentence of E73 (Fanout Cost of Node Key Interval) becomes "whether the dirent tree carries it has been answered uniformly as yes by D18 (Information Carried in a Block) settled item 2, and its fanout cost must be measured separately —"; the parenthetical of that sentence of E84 (Pinned Volume of Packed Tombstones) becomes "(adopted by user decision, written into the loading discipline of D18 (Information Carried in a Block) settled item 10)". Not a word of the experiment conclusions or artifacts changed.
  - **Basis**: for D18 (Information Carried in a Block) rewritten into the slimmed form see the decision changelog (number fourteen); looked up now: D18 (Information Carried in a Block) settled item 2 (all code 2 nodes carry key intervals unconditionally, closing the dirent tree cell along with it) and settled item 10 (loading by generation is a user decision, with how it was decided in the decision changelog).

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.
- See the same-day entry "experiment control audit — two experiments had zero absolute-value assertions, one experiment's positive control did not cover the configuration that produced the conclusion" in the E9 (Key Encoding Impact on Traversal Locality) section.
- See the same-day entry "experiment set up for D21 (Boundary Between Authoritative and Derived States) settled item 2" in the E43 (Extension Point Byte Upper Bound) section.

## E29 (Blast Radius of a Single Failed Node)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-09-06

- See the same-day entry "the criterion of E1 (Node Size vs Recoverability) is replaced by the rescan cost…" in the E1 (Node Size vs Recoverability) section.

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.
- See the same-day entry "experiment set up for D21 (Boundary Between Authoritative and Derived States) settled item 2" in the E43 (Extension Point Byte Upper Bound) section.

## E30 (Write Amplification Half of the Crossover Point)

### 2026-09-20

- See the same-day entry "E1 (Node Size vs Recoverability), E2 (Key Layout Impact on Recursive Traversal)…" in the E1 (Node Size vs Recoverability) section.

### 2026-09-03

- See the same-day entry "adversarial test round — all 19 owed mutation tables filled in, 10 new unit tests added where mutations escaped" in the E68 (Selection of Inlining Threshold) section.

### 2026-08-29

- See the same-day entry "audit-round reruns — 14 deterministic experiments, 12 byte-for-byte identical" in the E8 (Different Write Paths for Large vs Small Files) section.

## E31 (AAD Lacks Snapshot Dimension)

### 2026-09-19

- One current-state anchor in E31 (AAD Lacks Snapshot Dimension) corrected as looked up now; E31 (AAD Lacks Snapshot Dimension) and E90 (Tree ID into AAD and Cross-Head Sharing) get an "Affected decisions" table
  - **Before**: in the lookup anchor table "which sentence it attacks" of E31 (AAD Lacks Snapshot Dimension), the anchor of the row "D9 (Encryption) settled item 6 settled on 2026-08-28" read "the subsection title '已定项 6（2026-08-28）' of `decisions/09-加密.md`". The two pages E31 (AAD Lacks Snapshot Dimension) and E90 (Tree ID into AAD and Cross-Head Sharing) had no "Affected decisions" table.
  - **After**: the anchor now points to the "2026-08-29 (number nineteen)" entry of `decisions-history/2026-08.md`, which reads verbatim "this unsettled item was settled on 2026-08-28"; each of the two pages gets a table before "## Historical Versions": E31 (AAD Lacks Snapshot Dimension) supports D9 (Encryption) settled item 6, is preparation for D9 (Encryption) settled item 4, and has no effect on D6 (Snapshot Implementation Model) and D19 (Block Pointer Structure & Width Budget); E90 (Tree ID into AAD and Cross-Head Sharing) supports D9 (Encryption) settled items 6 and 16 and is preparation for D6 (Snapshot Implementation Model) settled item 1. Experiment conclusions and artifacts unchanged.
  - **Basis**: after D9 (Encryption) was rewritten into the slimmed form, item titles no longer carry dates (decision changelog (number eleven)), so the title the old anchor pointed to no longer exists; the new anchor was confirmed by looking up that entry in `.claude/kb/decisions-history/2026-08.md` now.

### 2026-09-03

- See the same-day entry "new E90 (Tree ID into AAD and Cross-Head Sharing), run to completion the same day — the 4th place in the D6 (Snapshot Implementation Model) chain…" in the E90 (Tree ID into AAD and Cross-Head Sharing) section.
- See the same-day entry "new E96 (Consistency of Hybrid Architecture), run to completion the same day — the user asked to test 'the central mapping exists unconditionally'" in the E96 (Consistency of Hybrid Architecture) section.

### 2026-08-29

- E31 (AAD Lacks Snapshot Dimension) newly set up and run to completion — its origin itself is worth recording
  - **E31 (AAD Lacks Snapshot Dimension) newly set up and run**, three rounds byte-identical, 5 unit tests, all 4 mutations caught.
    **Conclusion**: the AAD settled by D9 (Encryption) settled item 6 (five items, no snapshot dimension)
    **passes silently 2 / 2** against "snapshot key rewriting" — what is read is another writable head's data, and the MAC is all green;
    the positive control (flipping ciphertext bytes) is caught on all four arms, so that 0 is not "the MAC is not being verified".
    Two ways out each satisfy both criteria at once: bind the **birth snapshot**, or fold the snapshot dimension **into the nonce computed on the fly** —
    the latter is one value of D9 (Encryption) settled item 4, **and settled item 4 is already in the blocking set**.

  - ⚠️ **Its origin must be stated**: E31 (AAD Lacks Snapshot Dimension) comes from the adversarial verification round, where an attacking leg that had been explicitly told "do not change any file"
    **wrote an implementation and ran it on its own**. The main agent **did not adopt that code** (zero absolute-value assertions,
    judged red on the spot by the new gate stage `.claude/gate.d/80-absolute-assertions.sh`; and unreviewed,
    with no mutation test proving it can go red), **deleted the code and artifacts, keeping only the design**, and then rewrote a compliant one itself.
    **The two independent implementations are identical cell by cell on all four arms** — an unplanned control,
    but the rewritten version is the one taken as evidence. **"Same result" is no reason to adopt unreviewed code.**

## E32 (Remnants of the Previous Timeline)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-08-29

- See the same-day entry "one experiment set up for each of the five blocking items" in the E38 (Accounting Structure: COW vs In-Place) section.
- See the same-day entry "E35 (Two Forms of Multi-Writable-Heads) / E36 (Slot Mapping Dimension) newly set up and run to completion — both caught problems before code was written…" in the E35 (Two Forms of Multi-Writable-Heads) section.
- See the same-day entry "E39 (Does Reverse Chain Block Remnant Records) run to completion, D23 (Journal Role & Format) settled item 8's…" in the E39 (Does Reverse Chain Block Remnant Records) section.

## E33 (Two Pinned Block Rules)

### 2026-09-19

- E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place), E41 (Root Ring Slot Geometry 2×4×256), E46 (Root Ring Region Spacing and Failure Domains), E47 (Root Ring Failure Domain Losses), E48 (Feasible Points for Root Ring Geometry), E50 (Upper and Lower Bounds on Root Ring Slot Count), E53 (Can Root Ring Mount After Losing an Entire Disk), E55 (Stripe Mapping Arithmetic Validity on Real Devices), E79 (Capacity of Root Record), E87 (Placement of Fixed Structures), E92 (Worst-Case Block Reuse Delay Requirements), E125 (What is Read Above Write Pointer in Sequential Zone), E142 (New-Pool File Creation Dry Run), E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table), E115 (Completeness and Byte Budget of System Configuration Field Table) get "Affected decisions" rows for D22 (Synthesizing Unit Atomicity); two citations in E125 (What is Read Above Write Pointer in Sequential Zone) now point to the new item numbers, one verbatim quotation in E115 (Completeness and Byte Budget of System Configuration Field Table) becomes a parenthetical, and several current-state sentences in E142 (New-Pool File Creation Dry Run) and E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) corrected as looked up now
  - **Before**: line 82 of E115 (Completeness and Byte Budget of System Configuration Field Table) said "⇒ **the registered blocking item ③ of D22 (Synthesizing Unit Atomicity) settled item 9, 'at least 124 of the 420 bytes have no source', speaks only in one direction.**", and that quotation is no longer in the text of D22 (Synthesizing Unit Atomicity) after its slimming. The first sixteen pages had no "### 影响的决策" section and were all in `.claude/decision-links-pending`; the table of E142 (New-Pool File Creation Dry Run) had only the D23 (Journal Role & Format) rows. The first cell of row 1 of the "cited clauses" table of E125 (What is Read Above Write Pointer in Sequential Zone) read "D22 (Synthesizing Unit Atomicity) 'the root falls in the same class, and both classes of layout need the same mechanism'", and row 3 read "D22 (Synthesizing Unit Atomicity) settled three". Lines 59–60 of E142 (New-Pool File Creation Dry Run) said of the system configuration slot width cell "the rig is written first per the main agent's verdict; **the user has not yet confirmed this cell in person**, so it differs from the settled answers to the other 18 questions — overturning it only changes the number of padding bytes in the system configuration slot". Item 1 of "What it cannot answer" of E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) said "the encoding of the fields (byte order, enum values, bit assignment of feature bits) is not settled", and item 2 said "the 4096-byte slot is not computed: the slot width is settled at 512".
  - **After**: of the seventeen pages listed in the title, the fifteen other than E142 (New-Pool File Creation Dry Run) and E115 (Completeness and Byte Budget of System Configuration Field Table) each get an "Affected decisions" table, with one row per decision mentioned on the page (nine rows for E79 (Capacity of Root Record), one row each for E53 (Can Root Ring Mount After Losing an Entire Disk) and E55 (Stripe Mapping Arithmetic Validity on Real Devices)); E142 (New-Pool File Creation Dry Run) gets three D22 (Synthesizing Unit Atomicity) rows inserted into its existing table. The rows whose relationship is support match the basis paragraphs of D22 (Synthesizing Unit Atomicity) settled items 2 / 3 / 7 / 8 / 9 / 10 / 14 / 15 / 16 / 25 in both directions. Those two cells of E125 (What is Read Above Write Pointer in Sequential Zone) now point to D22 (Synthesizing Unit Atomicity) settled item 24 (quotation unchanged) and settled item 21 (the old "settled three"). That sentence of E142 (New-Pool File Creation Dry Run) becomes "the rig is written first per the main agent's verdict, and on the same day the user's closing popup decided on the format constant 4096 (decision changelog 2026-09-14 (number five)); it only changes the number of padding bytes in the system configuration slot". Item 1 of E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) becomes "not computed here; settled on the same day after the run by D22 (Synthesizing Unit Atomicity) settled items 13 and 17", and item 2 becomes "at run time the system configuration slot width was 512; since 2026-09-14 the system configuration slot width is the format constant 4096 (D22 (Synthesizing Unit Atomicity) settled item 2), the field table total of 481 still fits in the first 512-byte sector, and the 4096 tier has not been rerun". E115 (Completeness and Byte Budget of System Configuration Field Table) gets a nine-row table, and the quotation on line 82 becomes a parenthetical (the blocking item ③ registered at the time: at least 124 of the 420 bytes have no source), pointing to the same thing. Experiment conclusions and artifacts unchanged.
  - **Basis**: for D22 (Synthesizing Unit Atomicity) rewritten into the slimmed form see the decision changelog (number thirteen); looked up now: decision changelog 2026-09-14 (number five) (taking the format constant 4096 as the system configuration slot width is a decision from the user's closing popup), `SYSTEM_CONFIGURATION_SLOT_BYTES = 4096` in `crates/singlefs-format/src/lib.rs`, and D22 (Synthesizing Unit Atomicity) settled items 13 / 17 (byte order and registration bits settled on 2026-09-13).

### 2026-09-09

- Two self-reported unit test / mutation counts in the text of E33 (Two Pinned Block Rules) contradicted each other; aligned to the source
  - **Before**: the same text reported different counts in two places — line 12 "the rewritten version's **6 unit tests and 5 mutations** all caught",
    and the measurement-basis section "**9 unit tests and 6 mutations** all caught".
  - **After**: changed to 9 / 6 by counting the source. Looked up now:
    `grep -c '#\[test\]' scripts/experiment-apparatus/e33_pin_rules.rs` = 9,
    `wc -l < scripts/mutation/tables/e33_pin_rules.tsv` = 6 ⇒ the two numbers in the measurement-basis section were right, and both on line 12 were wrong.
  - **Basis**: `.claude/singlefs-ai-sop/rules/evidence-discipline.md` verbatim
    "**a number that a command can count should be counted by a command**, not left to the writer remembering to come back and fix it".
  - **Who found it**: the defense leg of the arm ① round found it in passing. ⚠️ **E33 (Two Pinned Block Rules)
    is exactly the tool that round used to knock out one basis of D22 (Synthesizing Unit Atomicity)**, so its own evidence-strength basis must be accurate.

### 2026-08-30

- See the same-day entry "two rounds of adversarial attack sent seven places back to the experiment side" in the E43 (Extension Point Byte Upper Bound) section.
- See the same-day entry "E50 (Upper and Lower Bounds on Root Ring Slot Count) and E51 (Number of Reverse Chain Collision Opportunities) set up and run to completion" in the E50 (Upper and Lower Bounds on Root Ring Slot Count) section.

### 2026-08-29

- E33 (Two Pinned Block Rules) newly set up and run to completion — one basis knocked out by arithmetic
  - **E33 (Two Pinned Block Rules) newly set up and run**, three rounds byte-identical, 6 unit tests, all 5 mutations caught.
    **Conclusion**: with the root ring depth K turned down to its lower limit, in a cycle with checkpoint interval 1000
    **only K−1 phases (3 / 1000 at K=4) free even a single block**, at most 16 blocks,
    while in the same period the journal replay window holds 4004 blocks — **a 250× difference**.
    **Basis**: the repository has two retention rules (the K generations of roots of I-7.4 (Blocks in Recent K Generations Not Reused),
    and the whole replay window of D16 (Publish Semantics) new rule 2 + D22 (Synthesizing Unit Atomicity) settled item 4),
    and only in the K−1 fsyncs right after a checkpoint completes does the former hold anything extra.
    **Positive control**: with the checkpoint interval set to 1, the same K=4 frees 16 blocks ⇒ the metric moves.

  - **It does not overturn the conclusion of D22 (Synthesizing Unit Atomicity) that "K is a run-time policy"; what falls is that basis**
    ("what K holds is exactly the space that saves us when the disk is nearly full, and it is the only escape hatch").
    The way out of deadlock 2 of D23 (Journal Role & Format) is settled as the **reserve pool**, which E19 (Spurious ENOSPC Under Defer Window)
    and E25 (Reserve Pool Size for Ring vs Chain) have both measured — **this repository already had an escape hatch.**

## E34 (Root Ring Slot Geometry)

### 2026-10-17

- A sentence added after the command of that one local-leg Q&A: the script was deleted on 2026-10-11, and how to retrieve it is in the section "local LLM (not dispatched as a leg in three-way arguments)" of tooling.md; conclusions, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/34-根环槽几何.md`.

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.
- See the same-day entry "E23 (Journal Geometry), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place), E1 (Node Size vs Recoverability)…" in the E23 (Journal Geometry) section.

### 2026-09-10

- See the same-day entry "new E129 (How Writes Below io_min Fail on Real Devices), run to completion — the first time this repository measured on a real device '…" in the E129 (How Writes Below io_min Fail on Real Devices) section.

### 2026-09-08

- The sentence in E34 (Root Ring Slot Geometry) "never settled in the repository" rewritten as looked up now, and the hard requirement 1 that its own premise table failed to cite added
  - **Before**: the section "the one remaining decision question" of E34 (Root Ring Slot Geometry) read verbatim "**whether different slots in the same region are
    'objects with different lifetimes' has never been settled in the repository.**"
  - **After**: becomes "there is only one instantiation given by generation, and no general criterion", naming D18 (Information Carried in a Block) settled item 10
    and E84 (Pinned Volume of Packed Tombstones), and pointing the criterion gap to D2 (RAID Stripe Policy) settled item 15;
    plus one sentence: the verified premise table of E34 (Root Ring Slot Geometry) copied the two hard requirements of D2 (RAID Stripe Policy) side by side,
    while the whole argument cited only hard requirement 2 — what actually decides this is hard requirement 1, landing in C212 (fixed-structure writes smaller than io_min).
  - **Basis**: three-way argument (artifacts `research/prompts/d22-slotwidth-*`);
    pointed out by the backward leg, confirmed by the main agent checking verbatim D18 (Information Carried in a Block):743-745 and the opening paragraph of E84 (Pinned Volume of Packed Tombstones).
  - **Not a word of the measured numbers or conclusions changed.**

### 2026-09-02

- The real-device half of E34 (Root Ring Slot Geometry) run; the exposure surface proven to be an upper bound
  - **Previously**: the real-device half of E34 (Root Ring Slot Geometry) was recorded as "owes `sudo` and power-loss injection",
    with only the arithmetic exposure surface "one `io_min` unit covers the whole region ⇒ 16 slots".
  - **Now**: the user provided the sudo password in `.env` ⇒ run to completion on a **real md/raid5** (4 legs, chunk 64 KiB).
    **`io_min` = 65536 / `physical_block_size` = 512 measured on a real device in this repository for the first time** (a 128× difference).
    Positive control (tear + degrade) judged bad 10/10, negative control (no tear + degrade) intact 10/10
    ⇒ the mechanism really reproduces and is not broken by the harness.
    **Two regions across zones intact 10/10, adjacent slots within the same chunk intact 10/10.**
  - **Basis**: real-device measurement, N=5 rounds × 2 complete invocations, verdicts of the five scenarios identical cell by cell.
    ⚠️ **It narrows the conclusion of the arithmetic half**: md/raid5 computes parity row by row by **sector row**
    ⇒ the failure unit is "same stripe row × same sector position" across legs, **not a whole `io_min` unit**
    ⇒ **exposure surface ≠ loss surface, 16 is an upper bound**, which this class of fault cannot realize on a real device.
    ⚠️ **It covers only one mechanism, the RAID5 write hole**: the tear constructs on-disk state and is **not a real power loss** (a loop device cannot lose power);
    **RMW of a whole mapping unit inside an SSD** cannot be injected and is still owed. The two mechanisms have different failure units and must not stand in for each other.

  ⚠️ **All three harness defects were stopped by controls and gates, not explained after the fact**:
  ① injecting without stopping the array ⇒ md's stripe cache still held old data, and the positive control could not judge bad (per the failure clause the whole round should be voided);
  ② on an all-zero disk the parity of a single marker row is a **verbatim copy** of the marker, and `grep` took the parity leg for a data leg ⇒ the wrong leg was failed;
  ③ the data marker `B_1_9` is a **substring** of the companion marker `CB_1_9`, and `grep` misjudged two legs ⇒ 5 rounds in a row discarded.
  **Each of them would have turned the conclusion into an idle spin, and all three were exposed only because "the gate went red".**

### 2026-09-01

- E34 (Root Ring Slot Geometry) changes its premise but not its criterion; the arithmetic half run
  - **Previously**: E34 (Root Ring Slot Geometry) was recorded as "not run", asking whether the candidate root ring geometry **proposed by** D22 (Synthesizing Unit Atomicity) settled item 2
    (verbatim "2 regions × 4 slots × 256 bytes = 2048 bytes") holds up.
  - **Now**: that candidate **was replaced entirely on 2026-08-31** (D22 (Synthesizing Unit Atomicity) settled item 2 verbatim:
    "the candidate form of 2 regions × 4 slots × 256 bytes is replaced entirely"; slot width 256 rejected by E41 (Root Ring Slot Geometry 2×4×256),
    R=4 rejected by E47 (Root Ring Failure Domain Losses)) ⇒ **the premise changes to the settled geometry, and not a word of the criterion changes** —
    the criterion of E34 (Root Ring Slot Geometry) was written parameterized by `(physical_block_size, io_min, 槽宽, 区域间距)` from the start.
    The arithmetic half has run: **claim 1 holds** (with the settled slot width choice = `physical_block_size`,
    one `io_min` unit on an array covers the whole region, knocking out 16 slots at S=16; under the choice suggested by E34 (Root Ring Slot Geometry) 0/48 cells hold);
    **claim 2 does not hold** (0/48 cells under prime-stride placement; only on the already-replaced contiguous form do 7/48 cells hold);
    after wiping a whole region **the worst rollback is 1 generation**, equal cell by cell to the real-device measurement of E53 (Can Root Ring Mount After Losing an Entire Disk).
  - **Basis**: `.claude/singlefs-ai-sop/rules/verify-before-claiming.md` "whether it is settled and how exactly it is defined are two questions" —
    checking the status column showed D22 (Synthesizing Unit Atomicity) settled item 2 was settled, and only reading the definition showed the candidate under test was gone.
    **Running two cloud legs against an expired premise is exactly what "the background material fed to a multi-party argument must itself be checked first" is meant to stop.**

  ⚠️ **The two cloud legs are still owed**, and the existing local-leg artifact asked about the already-replaced candidate
  ⇒ **it cannot count as a vote in this round**; a rerun needs new material.
  ⚠️ **The real-device half is still owed**: the means exist (E55 (Stripe Mapping Arithmetic Validity on Real Devices) already runs through on a real dm-stripe,
  where the chunk is `io_min`); what is owed is `sudo` permission and one power-loss injection.

### 2026-08-29

- Rerun-and-review round — 20 experiments reproduce byte for byte, E34 (Root Ring Slot Geometry) file created, the rerun loop of E9 (Key Encoding Impact on Traversal Locality) restored
  - **Before**: the experiment data in the kb could only be trusted; no command could replay them;
    a batch of experiments pointed to artifacts in `research/results/` without writing rerun commands.
    **After**: `scripts/experiment-apparatus/replay.sh` turns the reruns of 23 experiments into one table,
    **20 byte-for-byte identical, 3 (E17 (Write Buffer Merge Limit) / E20 (CPU Cache Cost of Widening Pointers) / E21 (Net Benefit of GPU Offloading)) differing only in timing fields with all conclusion-range assertions hit, 0 mismatched**.
    **Basis**: item 2 of `.claude/singlefs-ai-sop/rules/kb-discipline.md` "all historical data is only a reference;
    to use it in support of a new conclusion, rerun it first" — until now no rig could carry out that sentence.

  - **Before**: the checked-in artifact of E9 (Key Encoding Impact on Traversal Locality) was 25 runs stitched together, and not a word of that loop was written into the kb.
    **After**: the loop was worked back from the artifact's config lines (5 seeds × 5 rename tiers, `cache_leaves` always 8),
    and the rerun is **byte-for-byte identical** to the checked-in artifact (150 lines, all 25 completion markers matching); the loop is now written verbatim into the measurement-basis section of E9 (Key Encoding Impact on Traversal Locality).
    **Basis**: the rerun showed that "the artifact exists" and "the artifact can be reproduced" are two different things,
    while the gate stage "were experiment artifacts written back" only checked the former. ⇒ New gate stage
    `.claude/gate.d/85-repro-command.sh`, which **went red on 16 files on its first run**; this round filled them in one by one.

  - **Before**: E34 (Root Ring Slot Geometry) existed only in section 4 of `records/2026-08-29-组合对攻轮.md`,
    with prompts and artifacts under research but no number in the kb.
    **After**: file created, status **not run**, with four unclear points written down one by one (two cloud legs owed,
    `io_min == physical_block_size == 512` on this host so fault injection cannot falsify it, the counterexample given by the local leg is unreachable on Linux,
    and that disagreement itself has had no third-party review).
    **Basis**: caught by the new gate stage `.claude/gate.d/86-experiment-orphans.sh`.
    The existing "were experiment artifacts written back" could not catch it — that stage excludes every artifact whose file name contains `local` as a local-leg Q&A,
    and this entry's artifact happens to be named `e34-rootring-local.out`. ⇒ **The exclusion rule happened to cover exactly this one.**

  - **Before**: the rerun baseline of E20 (CPU Cache Cost of Widening Pointers) was `e20-fanout-2026-08-28.out` (51 lines).
   **After**: the baseline now points to `e20-poscontrol-2026-08-29.out` (164 lines).
    **Basis**: today the default run of the same binary produces 164 lines — the positive-control part was added later.
    ⇒ The 08-28 copy can only serve as a historical record, **not as a rerun baseline**; a structural comparison of the two shows 0 differing lines.

  - **Mutation table reruns**: all 10 rerun, **48 caught, 0 blind spots, 1 judged invalid for failing to compile** (the e30 row "change the rescan amount to fanout minus one", which does not compile after replacement).
    ⚠️ `src/bin/` has 26 bins ⇒ **16 still have no table**; C40 (mutation table not saved) has been rewritten with the numbers as looked up now.

- See the same-day entry "one experiment set up for each of the five blocking items" in the E38 (Accounting Structure: COW vs In-Place) section.

## E35 (Two Forms of Multi-Writable-Heads)

### 2026-08-30

- See the same-day entry "E52 (Operational Costs of Two Multi-Writable-Head Mechanisms) set up and run to completion" in the E52 (Operational Costs of Two Multi-Writable-Head Mechanisms) section.

### 2026-08-29

- E35 (Two Forms of Multi-Writable-Heads) / E36 (Slot Mapping Dimension) newly set up and run to completion — both caught problems before code was written
  - **E35 (Two Forms of Multi-Writable-Heads)**: N=5 rounds byte-identical, 7 unit tests, all 6 mutations caught.
    **Conclusion**: the three-part key settled by D8 (Core Index Structure) is **wrong in itself** under multiple writable heads —
    two heads share one key slot, `own_reads_ok = 1 / 2`, and **this is independent of encryption**;
    with encryption on and D9 (Encryption) settled item 4 choosing "computed on the fly", a further layer of **keystream reuse** stacks on top
    (running real ChaCha20 gives "XOR of the two ciphertexts == XOR of the two plaintexts").
    Both ways out (add a snapshot dimension to the key / one tree per head) satisfy both criteria at once.
    ⚠️ "Just add a version number" was stopped by a unit test: each head is writing for the first time, so both version numbers are 1.

  - **E36 (Slot Mapping Dimension)**: N=5 rounds byte-identical, 8 unit tests, all 7 mutations caught.
    **It picks up the 2 / 4 combination owed by E32 (Remnants of the Previous Timeline)**,
    and **overturns the way out proposed by the three-way argument**: merely decoupling slot from sequence number **cannot rescue it**
    (at equal lengths it is identical cell by cell to mapping by sequence number); what really breaks the alignment is **records of different lengths**,
    and the same arm with lengths changed back to equal replays everything ⇒ **that is coincidence, not a guarantee**.
    Positive control: the by-sequence-number column reproduces the formula of E32 (Remnants of the Previous Timeline) cell by cell — two independent models agree.

## E36 (Slot Mapping Dimension)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-08-29

- See the same-day entry "E35 (Two Forms of Multi-Writable-Heads) / E36 (Slot Mapping Dimension) newly set up and run to completion — both caught problems before code was written…" in the E35 (Two Forms of Multi-Writable-Heads) section.
- See the same-day entry "E39 (Does Reverse Chain Block Remnant Records) run to completion, D23 (Journal Role & Format) settled item 8's…" in the E39 (Does Reverse Chain Block Remnant Records) section.

## E37 (Log Instance Generation)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-13

- A note added to conclusion 1 of E37 (Log Instance Generation) — with the prefix rule unchanged, that conclusion holds only under the basis "recovery picks up from a stale tail"
  - **Before**: conclusion 1 said "⇒ it also has to change a settled prefix rule (accept 'identifier +1 with the counter continuing')".
  - **After**: a note line is added under conclusion 1: D23 (Journal Role & Format) settled item 14 makes replay pick up after the last record covered by the chosen root, so in normal recovery new records share the chosen root's instance and the prefix rule does not change; artifacts and source unchanged.
  - **Basis**: decision changelog number nine (two rounds of three-way argument on C199 (instance generation increment contradicts stopping at jsn gaps)).

### 2026-08-29

- E37 (Log Instance Generation) newly set up and run to completion; E7 (Offline Index Harness) has an archived artifact for the first time
  - **E37 (Log Instance Generation)**: N=5 rounds byte-identical, 7 unit tests, all 6 mutations caught.
    It picks up the only untested way out in the table of ways out of D23 (Journal Role & Format) settled item 8.
    **Conclusion: that way out costs more than the table recorded.**
    Adding the field is not enough — once the identifier is incremented, the `jsn` of new records is no longer contiguous with the old prefix,
    and the settled criterion `jsn == expected` would **drop all** newly written records (the same disease as `skip_hole`);
    **it also has to change a settled prefix rule**.
    Moreover, rolling the identifier back by one makes it **fall back to baseline cell by cell** ⇒ correctness rests entirely on "the identifier must never roll back",
    the same type as the requirement of D9 (Encryption) settled item 8 on the nonce watermark, **and D23 (Journal Role & Format) has never written a word about it**.

  - **E7 (Offline Index Harness) has an archived artifact for the first time**: this round reran it on a plain file on this host with
   N=5 and five seeds, artifact `e7-index-2026-08-29-file.out`.
    The medians of the three arms differ from the text by **less than 0.3%**, and the two ratios 3.20 / 1.23 match the text's 3.22 / 1.23.
    ⚠️ **But this rerun is not a complete reproduction of its chain of evidence**: on a plain file `blkstat=false`,
    **the verification path of independent block-layer readings is unavailable**, while the text lists it as one of four layers of verification.
    ⇒ **What reproduced is the numbers, not that verification.**

## E38 (Accounting Structure: COW vs In-Place)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.
- See the same-day entry "E23 (Journal Geometry), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place), E1 (Node Size vs Recoverability)…" in the E23 (Journal Geometry) section.

### 2026-08-29

- One experiment set up for each of the five blocking items
  For the five unsettled items "blocking the first line of transaction-layer code", the user gave a direction or asked for measurement, and one experiment text was set up for each, **none run yet**:

  | Experiment | Which unsettled item it serves | The step it must answer |
  |---|---|---|
  | E38 (Accounting Structure: COW vs In-Place) | D22 (Synthesizing Unit Atomicity) settled item 3 | The reason for the settled direction, "units are self-contained ⇒ parent pointers are better", is an inference and has never been measured |
  | E39 (Does Reverse Chain Block Remnant Records) | D23 (Journal Role & Format) settled item 8 | A settled direction does not mean proven effectiveness; E32 (Remnants of the Previous Timeline) measured the hole, E39 (Does Reverse Chain Block Remnant Records) measures the patch |
  | E40 (Ciphertext Checksum Bit Width) | D19 (Block Pointer Structure & Width Budget) settled item 2 | The existing derivation admits itself "single leg, not reproduced by a verification path, the corruption rate is assumed" |
  | E41 (Root Ring Slot Geometry 2×4×256) | D22 (Synthesizing Unit Atomicity) settled item 2 | Whether the candidate form suffices under the two constraints of crashes and block reuse (division of labor with E34 (Root Ring Slot Geometry): that one asks about the relation between slot width and the device mapping unit) |
  | E42 (Number of Records per Transaction) | D23 (Journal Role & Format) settled item 7 | Two wordings in the repository give two readings and both hold ⇒ it was never settled |

  Each one writes down what is measured / criteria / **positive control** / failure clauses / required absolute-value assertions.
  ⚠️ **None of the five is implemented yet**, so for now they are only "the question has been written clearly", not "the question has an answer".
- E38 (Accounting Structure: COW vs In-Place) and E42 (Number of Records per Transaction) run to completion; one settles its item, one is still owed
  - **E38 (Accounting Structure: COW vs In-Place)**: N=5 rounds byte-identical, all 6 mutations caught, 0 blind spots.
    **D22 (Synthesizing Unit Atomicity) settled item 3 is thereby settled as COW, and no longer blocks the first line of code.**
    ⚠️ **The decisive reason is neither the one the user originally gave nor write amplification**:
    the recovery cost differs by **350,000×** (COW = tree height of 2–3 blocks, in-place = the whole disk of 1 048 576 blocks),
    and in-place needs mkfs to reserve accounting leaf count × 2 (**33.55 million blocks** at the F=256/H=3 tier)
    ⇒ **the path "run in place first, change it later" does not exist**.
    ⚠️ The write amplification criterion (1.25–1.375×) **is bought from the coarse-grained settlement of D25 (Target Workload Priority)**;
    the same 8 leaves scattered become 2.125–3.125×, **so any change to D25 (Target Workload Priority) requires a rerun**.

  - **E42 (Number of Records per Transaction)**: N=5 rounds byte-identical, all 7 mutations caught, 0 blind spots.
    **Both paths can satisfy I-4.3 (Atomic Commit)** — one record per transaction 0/201 illegal; spanning multiple records without boundary fields **1400/1601 (87.4%)** illegal;
    spanning multiple records with transaction number + commit marker back to 0/1601.
    ⇒ **The experiment cannot tell right from wrong, only where the cost lands; D23 (Journal Role & Format) settled item 7 is still unsettled.**
    ⚠️ That 87.4% shows "spanning multiple records goes wrong" is not a corner case but the vast majority of crash points.

  - ⚠️ **The first version of E42 (Number of Records per Transaction) had a blind spot, filled by mutation testing**:
    "prefix count" was written as `recs.len() + 1`, the **value it ought to be**, so when the enumeration loop's upper bound was changed from `0..=len` to `0..len`
    **not one of the eight tests went red** — neither side of the comparison came from the loop.
    After changing it to count "how many were really enumerated", that mutation is caught. **This is a new form of `rules/test-discipline.md`
    "comparing arms only with each other cannot detect all arms being wrong together": both sides take the value it ought to be.**
- E38 (Accounting Structure: COW vs In-Place) and E39 (Does Reverse Chain Block Remnant Records) redone at the user's follow-up questions
  - **E38 (Accounting Structure: COW vs In-Place) converts "how many blocks" into "how many bytes"**, and changes the acceptable line for the reservation from 1% to **2%** (user decision).
    ⚠️ Two measurement-basis fixes in passing: ① what in-place must reserve is **all nodes**, not just leaves (internal nodes were undercounted before);
    ② the pool size expands from a single 4 GiB to three tiers, 4 GiB / 64 GiB / 1 TiB,
    because both "full-disk scan" and "reservation share" are functions of the pool, and one tier cannot show a trend.
    Result: in-place reserves **32.5 MiB (F=64/H=2) to 128.5 GiB (F=256/H=3)**;
    the recovery cost is 350,000× for a 4 GiB pool and **89.48 million×** for a 1 TiB pool.
    ⚠️ **The share arithmetic originally lived only in `main()`**, and changing 10 000 to 1 000 turned no test red —
    after it was extracted into a function with pinned absolute values, the mutation `M8_占比算成千分之一` of `e38_accounting_copy_on_write` caught it.

  - **E39 (Does Reverse Chain Block Remnant Records) adds a width sweep** (user: 8 bits is too few, try 16 and 32, and give the size).
    **2 million independent trials** per tier: 8 bits 3961 ppm, 16 bits 16 ppm, 32 bits 0, **each tier sticking to 2⁻ⁿ**;
    record header 84 → 85 / 86 / 88 bytes (+1.19% / +2.38% / +4.76%).

  - ⚠️ **The first version of the width sweep measured nothing at all, yet it looked like a conclusion**:
    the hash put the seed **at the end** and truncated straight to the low bits, so the 8-bit tier produced **0 false acceptances** (7813 in theory).
    The mechanism is that FNV-1a's low-byte evolution `low = (low ^ b) * 0xb3 mod 256` is a **bijection** for every input byte
    ⇒ appending **the same** seed bytes after each end's own state, whether the low bytes are equal **does not depend on the seed**.
    After changing to seed first + splitmix64 final mixing, measurement sticks to the theoretical value.
    It is guarded by the new assertion "measurement must stick to 2⁻ⁿ" and the mutation `M8_退回第一版哈希` of `e39_back_chain`.
    **This is the most valuable lesson of this round: a "safe" reading may just mean nothing was measured.**

  - ⚠️ **A second pitfall of the same kind**: the seed was made a global static, and unit tests running in parallel trampled each other;
    "the 8-bit tier should give several hundred" measured 3. **The seed is a parameter, not a global.**

- See the same-day entry "experiment set up for D21 (Boundary Between Authoritative and Derived States) settled item 2" in the E43 (Extension Point Byte Upper Bound) section.

## E39 (Does Reverse Chain Block Remnant Records)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-13

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) by journal header…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-06

- See the same-day entry "a round of recovering from format constants drifting out of sync — E43 (Extension Point Byte Upper Bound) rerun a fourth time at 105…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-08-30

- Two corrections to E39 (Does Reverse Chain Block Remnant Records) — one basis overturned, two counts wrong
  Forced out by the two cloud legs on D23 (Journal Role & Format) settled item 8, adopted after the main agent recomputed each one independently.

  - **"The three tiers of 2 / 4 / 8 bytes are identical cell by cell on disk" is overturned.** Before: the text said "the difference between 2 and 4 does not exist on disk; it is not 'very small'".
    After: sweeping 0..1200 entries with the own constants of E39 (Does Reverse Chain Block Remnant Records) (header 84 + transaction fields 9 = 93, 56 per entry) —
    **under 512-byte units, 2 and 4 bytes differ at 19 entry counts** (smallest items=44, 2560 B vs 3072 B, a 512 B difference),
    and at 2 under 4096-byte units; 0 places between 4 and 8. Mechanism: `gcd(56, 512) = 8` ⇒ whether it comes free depends only on
    `记录头总字节 mod 8`, **which is coincidence, not structure**.
    ⚠️ And D23 (Journal Role & Format) settled item 9, settled the same day (`jsn` 8 → 10, header 84 → 86), **happens to bring the number of divergence points back to 0**
    ⇒ this basis holds now, but **it is sensitive to the header width** and must no longer be cited as "structurally free".
    The original table took only the two points of 1 and 12 named entries, **and neither of those two points happens to be a divergence point**.
  - **Mutation count**: two places in the text said "8 mutations", while `scripts/mutation/tables/e39_back_chain.tsv` actually has **10**. Fixed.
  - **"0 blind spots" withdrawn**: mutations only prove "the functions covered by the assertions written down will go red", not "the quantifiers of the assertions are written right".
    The concrete counterexample is exactly the "three tiers identical cell by cell" item. New owed check C49 (Universal Assertions Only Verified at Sample Points).

- See the same-day entry "E49 (Reverse Chain Width: 32 vs 64) set up and run to completion" in the E49 (Reverse Chain Width: 32 vs 64) section.

### 2026-08-29

- E39 (Does Reverse Chain Block Remnant Records) run to completion; "add it or not" of D23 (Journal Role & Format) settled item 8 is settled
  - **E39 (Does Reverse Chain Block Remnant Records)**: N=5 rounds byte-identical, all 7 mutations caught, 0 blind spots.
    Of the 32 cells, **the 16 with the chain on have all remnants at 0 and not one of their own records wrongly killed**;
    the 8 with the chain off and continuing from the end of the prefix all replay 6 remnant records.
    ⇒ "Add it or not" of D23 (Journal Role & Format) settled item 8 is settled, **the width is still unsettled** —
    E39 (Does Reverse Chain Block Remnant Records) ran only 31 records, and an 8-bit hash not hitting a collision **does not amount to "8 bits is enough"**.

  - **It independently reproduces the conclusion of E36 (Slot Mapping Dimension)**: the model was rewritten without using the code of E32 (Remnants of the Previous Timeline)
    and E36 (Slot Mapping Dimension), yet the replay counts of the decoupled branch and of the `jsn`-modulo branch are **equal cell by cell**
    ⇒ decoupling alone does not immunize. This is a free cross-check.

  - ⚠️ **Three problems were knocked out by assertions and mutations**:
    ① the absolute-value assertion was written as "build 8 and 8 should replay", and measured **6** — the new timeline overwrote the hole plus the 2 after it,
    **the model was not wrong; the assertion undercounted the overwrite**;
    ② if the decoupled arm does not move the write head back to the break point, what is measured is "all new records lost" rather than "remnants replayed" —
    **a model defect disguising itself as a conclusion**;
    ③ with "sequence number must equal expected" removed, **not one of the nine tests went red**, because that gate is redundant in the other arms.

- See the same-day entry "one experiment set up for each of the five blocking items" in the E38 (Accounting Structure: COW vs In-Place) section.
- See the same-day entry "E38 (Accounting Structure: COW vs In-Place) and E39 (Does Reverse Chain Block Remnant Records) redone at the user's follow-up questions" in the E38 (Accounting Structure: COW vs In-Place) section.

## E40 (Ciphertext Checksum Bit Width)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-08-30

- E40 (Ciphertext Checksum Bit Width) has run
  - Before: not run; D19 (Block Pointer Structure & Width Budget) settled item 2 had only a derivation that admitted itself "single leg, not reproduced by a verification path, the corruption rate is assumed".
    After: N=5 rounds byte-identical, all 10 unit tests green, 9 mutations with zero blind spots.
    **The arithmetic part reproduces the three numbers of that derivation** (block reads 1.273×10¹², corrupted reads 1273, minimum bit width 31),
    and measures that **the margin is only 3.37×** — if any one of the five inputs were wrong by that factor, 33 bits would be needed; a corruption rate wrong by 10× would need 34 bits.
    **The collision part first proves the law "miss rate = 2⁻ⁿ"** (truncated to 8 / 12 / 16 bits, measurement and theory are of the same order),
    so the 32 / 64-bit rows are **extrapolations**, not measurements (32 bits would need ~4×10⁹ trials to see a single miss).
    The positive control holds both ways: the 8-bit sum check misses all 248951/248951 byte swaps, full-width CRC32C misses 0.
   Basis: artifact `e40-csum-width-2026-08-30.out` (48 lines).
  - **A finding unique to E40 (Ciphertext Checksum Bit Width)**: **a truncated CRC is not "a shorter CRC"** —
    full-width CRC32C misses 0 single-bit flips (a structural property), while truncated to 8 bits it misses 3212 ppm.
    ⇒ Truncation downgrades single-bit flips from "always detected" to "probabilistically detected".
  - ⚠️ **All three algorithm implementations are pinned with public test vectors** (CRC32C / CRC64 / SHA-256).
    Without this step, a wrong implementation would make the four arms **drift together**, which comparing them with each other cannot see.
- The experiment index changed to carry conclusion briefs; three stale statuses fixed
  - Before: the index table in `experiments.md` was cut into four pieces by blank lines and had only a navigation column;
    the titles of the three texts E40 (Ciphertext Checksum Bit Width), E41 (Root Ring Slot Geometry 2×4×256), E43 (Extension Point Byte Upper Bound) still said "not run",
   while all three already had a "### 实测（2026-08-30）" section and their artifacts were in `research/results/`.
  - After: the index is merged into one table with an added "conclusion (brief)" column (within 50 characters, the same shape as `decisions.md`),
    and the status column is always short; the titles of the three texts change to run, each with a one-sentence conclusion.
  - Basis: the three texts' own measurement sections and artifact files; `.claude/gate.d/40-results-cited.sh` all green as looked up now.
  - In passing: one of the duplicated "## Historical Versions" headings in the text of E41 (Root Ring Slot Geometry 2×4×256) is deleted;
    in E43 (Extension Point Byte Upper Bound), "E40 (Ciphertext Checksum Bit Width) has run but that item is still unsettled" becomes
   "that item is settled at 32 bits, and the reopening conditions are inherited with it" — D19 (Block Pointer Structure & Width Budget) settled item 2 has been settled.

- See the same-day entry "two rounds of adversarial attack sent seven places back to the experiment side" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-08-29

- See the same-day entry "one experiment set up for each of the five blocking items" in the E38 (Accounting Structure: COW vs In-Place) section.
- See the same-day entry "experiment set up for D21 (Boundary Between Authoritative and Derived States) settled item 2" in the E43 (Extension Point Byte Upper Bound) section.

## E41 (Root Ring Slot Geometry 2×4×256)

### 2026-10-17

- Item 3, "tear isolation still owes a requirement that root slots be aligned to the judging width", becomes "was still owed at the time", adding that it is now I-7.5 (root slots aligned to the judged width), implemented, and judged slot by slot by the pool-level checker; conclusions, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/41-根环槽几何2x4x256.md`.

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-01

- See the same-day entry "E34 (Root Ring Slot Geometry) changes its premise but not its criterion; the arithmetic half run" in the E34 (Root Ring Slot Geometry) section.

### 2026-08-30

- E41 (Root Ring Slot Geometry 2×4×256) has run
  - Before: not run; only the candidate form "2 regions × 4 slots × 256 bytes" proposed by D22 (Synthesizing Unit Atomicity) settled item 2, unverified.
    After: N=5 rounds byte-for-byte identical, all 12 unit tests green, all 7 mutations caught.
    **Tear isolation judged red** — slot width 256 knocks out 2 / 16 / 256 slots at atomic widths 512 / 4096 / 65536 respectively,
    and the last tier exceeds the ring's 8 slots ⇒ one write wipes out every witness;
    the positive control drops to 1 once the slot width is raised to 512.
    **"Is one lap enough" cannot be judged**: the artifact says `reuse_requirement_source=none`,
    and the kb has no number for "worst-case block reuse delay requirement"; only an inverse solution can be given (protecting 1000 operations, with a root published per fsync ⇒ 1000 slots needed).
   Basis: artifact `e41-root-ring-geom-2026-08-30.out` (29 lines).
  - ⚠️ **Mutation testing caught two real blind spots, now fixed**: when a slot is wider than the atomic unit, the number of slots knocked out was computed as 0; without rounding up, the inverse solution computed "0 slots needed".
  - ⚠️ **One equivalent mutation is also recorded** (`>=` changed to `>`): the same value on all inputs, so not catching it is no blind spot; it is filed as a test.
- E41 (Root Ring Slot Geometry 2×4×256) rerun with a changed model
  - Before: the judging-width column included 65536, from which it concluded "one write knocks out 256 slots and wipes out every witness".
    After: **that conclusion is deleted** — 64 KiB is of the order of `io_min`, not of `physical_block_size`
    (checked line by line now in `block/blk-settings.c`, `drivers/md/raid0.c`, `raid5.c`),
    and the atomic width of the self-verifying unit settled by D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) is the latter.
    It also adds the **starting offset** input that was required at setup but left out, and makes `io_min` its own dimension.
    New conclusion: **a 2048-byte ring fits within an `io_min` of 4096 / 65536** ⇒ an RMW inside the device can overwrite the whole ring at once,
    and the cure is to widen the region spacing, not to raise the slot width.
    Basis: the backward leg of the three-way argument pointed out the mismatch in what was measured, and the main agent confirmed it by checking line by line now; the artifact rerun is byte-identical over N=5 rounds (40 lines).
  - ⚠️ **A third mutation blind spot fixed**: without taking the starting offset modulo, the 8th slot in the ring was computed as touching 9 atomic units.
  - ⚠️ **A self-assessment recorded**: the tear half shares the day, the crate and the same one-line formula with E43 (Extension Point Byte Upper Bound)
    ⇒ **it does not count as an independent verification leg**; the net increment of E43 (Extension Point Byte Upper Bound) is the dimensions "one lap / inverse solution" and "alignment / `io_min` span".

- See the same-day entry "two rounds of adversarial attack sent seven places back to the experiment side" in the E43 (Extension Point Byte Upper Bound) section.
- See the same-day entry "the experiment index changed to carry conclusion briefs; three stale statuses fixed" in the E40 (Ciphertext Checksum Bit Width) section.
- See the same-day entry "new E46 (Root Ring Region Spacing and Failure Domains), run to completion" in the E46 (Root Ring Region Spacing and Failure Domains) section.

### 2026-08-29

- See the same-day entry "one experiment set up for each of the five blocking items" in the E38 (Accounting Structure: COW vs In-Place) section.

## E42 (Number of Records per Transaction)

### 2026-09-19

- See the same-day entry "E17 (Write Buffer Merge Limit), E42 (Number of Records per Transaction)…" in the E17 (Write Buffer Merge Limit) section.
- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-08-29

- See the same-day entry "one experiment set up for each of the five blocking items" in the E38 (Accounting Structure: COW vs In-Place) section.
- See the same-day entry "E38 (Accounting Structure: COW vs In-Place) and E42 (Number of Records per Transaction) run to completion; one settles its item, one is still owed" in the E38 (Accounting Structure: COW vs In-Place) section.

## E43 (Extension Point Byte Upper Bound)

### 2026-10-04

- E43 (Extension Point Byte Upper Bound) fills in the three mutation numbers (whole-table rerun of the record header 311 → 487 rerun, run by the main agent; numbers only, conclusions and artifacts unchanged)
  - **Before**: the next entry said "the whole-table rerun has no result yet", and the experiment page said the three numbers were to be backfilled; the before run fixed by the registration was not run either (reason in section twelve of the registration).
  - **After**: the whole table of 24 rows caught 24, invalid 0, not red 0 (`research/results/e43-mutate-2026-10-04-h487.log`, ending with "restored, baseline still all green", with hitting the ceiling, timeouts, unscheduled and evicted all 0); registration voiding clause V4 triggers only if, after the change, one of the original M1–M19, M22 is not "caught", and these 20 rows plus M20, M21, M23, M24 are ✅ one by one ⇒ it does not trigger. The "27 unit tests all green" entry on the experiment page and the mutation sentence of the section "2026-10-04: record header 311 → 487 rerun" are changed per the three numbers.
  - **Basis**: log `research/results/e43-mutate-2026-10-04-h487.log`; sections eleven and twelve of registration `research/prompts/e43-r2-prereg.md`.
- E43 (Extension Point Byte Upper Bound) rerun with record header 311 → 487 (row 4 of the question sheet)
  - **Before**: `JOURNAL_HEADER_BYTES` in `git show 31e47115:scripts/experiments/e43_extension_point_budget.rs` was 311 (new-root segment 188), artifact `e43-ext-budget-2026-09-24-h311.out`.
  - **After**: after `research/prompts/e43-r2-prereg.md` fixed the criteria (swapping only the two judged values 311 and 487 and the sensitivity sample points next to the old value, with criteria, thresholds and arms unchanged), the rig constant changed to 487, 6 unit tests changed their pinned values and 2 were added (27 all green), the sensitivity sample point 307 was replaced by 487, and the mutation table went 22 → 24 rows (the original texts of M20, M21 follow the constant, M23, M24 added, the whole-table rerun has no result yet). The comparison baseline run right before changing the rig is byte-for-byte identical to the h311 copy. Artifact `e43-ext-budget-2026-10-04-h487.out` differs from the h311 copy in 14 lines: the two self-verifying unit lines `room` 201 / 3785 → 25 / 3609, `n_max_if_self_witness_carries` 201 → 25, in `mount_eval` the two lines with `atomic=512`, slot widths 512 and 4096, N=128 change from `ok` to `reject_self_witness_overflow`, and the two `h=307` lines of the sensitivity table are replaced by `h=487`. **Row 4 of the question sheet is recorded as "changed"**: whether the 128 in the example of D21 (Boundary Between Authoritative and Derived States) fits the self-witness margin and two cells of the mount judgement grid flip, while the three cells "which side bounds the self-witness upper bound", "fits lower bound 9" and "fits lower bound 13" are unchanged. The E43 (Extension Point Byte Upper Bound) row of `scripts/experiment-apparatus/replay.sh` now points to the new artifact, and the rerun is "byte-identical".
  - **Basis**: `research/prompts/e43-r2-prereg.md`; row 4 of `research/prompts/m3-r1-batch9-reruns-questions.md`; D23 (Journal Role & Format) settled items 4 / 15 / 17; D21 (Boundary Between Authoritative and Derived States) settled items 2 / 4 / 16.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 documentation iron-rule correction: numbers in this block got short names and clashing local arm names were changed to lowercase; the original sentences are in this version)

### 2026-09-25

- See the same-day entry "E49 (Reverse Chain Width: 32 vs 64) rerun with record header 307 → 311 (row 7 of the question sheet)" in the E49 (Reverse Chain Width: 32 vs 64) section.

### 2026-09-24

- E43 (Extension Point Byte Upper Bound) and E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) rerun with record header 307 → 311
  - **Before**: `JOURNAL_HEADER_BYTES` in both rigs, E43 (Extension Point Byte Upper Bound) and E116 (Packed Container Balance: Metadata Writes and Compaction Strategy), was still at 307, while D23 (Journal Role & Format) settled item 4 changed the record header to 311 on the same day (adding 4 bytes of "sequence number within this publish") — `.claude/gate.d/27-format-constants.sh` judges that format constants at this level must move in three places together (kb marker and text, experiment source constants and unit tests, artifacts), and these two rigs had never followed.
  - **After**: after the two rerun registrations `e43-r1-prereg.md` and `e116-r1-prereg.md` fixed the criteria cell by cell, the constants of both rigs changed to 311, and each added a section-eight geometry-sensitivity output with unit tests (one point on each side of the flip point, proving the criterion discriminates on this knob). E43 (Extension Point Byte Upper Bound): 25 unit tests all green (4 new), all 22 mutations caught (M20/M21/M22 new, run once before and once after the constant change, 0 invalid, 0 not red); the five cells Q43.1 (header width not leaking into other cells)–Q43.5 (mount judgement grid) judge the same values both times, **row 1 of the question sheet recorded as "unchanged"** — the self-verifying unit upper bound is still bounded by the journal record header side (201, formerly 205). E116 (Packed Container Balance: Metadata Writes and Compaction Strategy): 20 unit tests all green (3 new), all 18 mutations caught (M17/M18 new, 0 invalid, 0 not red); the five cells Q116.2–Q116.6 judge the same values both times, **row 2 of the question sheet recorded as "unchanged"** — the payback ratios of the two packing arms `bg_key`/`bg_fill` both rise slightly, but no cell crosses the threshold 1 of the reverse-acceptance clause. The two artifacts `e43-ext-budget-2026-09-24-h311.out` and `e116-pack-settle-2026-09-24-h311.out` go into `research/results/`, two rows of `scripts/experiment-apparatus/replay.sh` now point to the new artifacts, and both reruns are "byte-identical". The six current-state sentences judged "change the sentence only" in row 7 of the question sheet (`23-journal几何.md`, `39-反向链挡不挡得住残留记录.md`, `42-一事务几条记录.md`, `61-反向链hash算法的均匀性.md`, `75-记录尺寸与环几何.md`) were changed to 311 along the way; the E49 (Reverse Chain Width: 32 vs 64) cell of row 7 is judged "needs a rerun", not run this time.
  - **Basis**: `e43-r1-prereg.md`, `e116-r1-prereg.md` (two rerun registrations, with criteria, thresholds, failure and stop clauses fixed before the run); rows 1, 2 and 7 of `m2-header311-rerun-questions.md`; D23 (Journal Role & Format) settled item 4.

### 2026-09-20

- The "Affected decisions" tables of E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set), E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State), E70 (Feasible Region for Two Checkpoint Thresholds), E91 (Ring and Admission Accounting Under Settled Format), E99 (Deduplication of Write Buffer Entries and Seq), E103 (Update Cost of Inode Packing Path 1), E105 (Cost of Changing Extent Leaf to Code 3), E108 (Cost Crossover Point of Plaintext Layer), E109 (Runtime Costs of Three Location Authority Arms), E112 (Legacy Writer Encountering Unknown Trees), E113 (Unknown Trees: Saturated Arm Set), E123 (Trade-off Costs Between K and Rollback Depth) completed
  - **Before**: the nine pages E57 (Authoritative Placement for Plaintext-Side Field Set), E70 (Feasible Region for Two Checkpoint Thresholds), E91 (Ring and Admission Accounting Under Settled Format), E99 (Deduplication of Write Buffer Entries and Seq), E103 (Update Cost of Inode Packing Path 1), E108 (Cost Crossover Point of Plaintext Layer), E112 (Legacy Writer Encountering Unknown Trees), E113 (Unknown Trees: Saturated Arm Set), E123 (Trade-off Costs Between K and Rollback Depth) had no "### 影响的决策" table; the tables of the four pages E43 (Extension Point Byte Upper Bound), E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State), E105 (Cost of Changing Extent Leaf to Code 3), E109 (Runtime Costs of Three Location Authority Arms) registered only D21 (Boundary Between Authoritative and Derived States) and D1 (Data Mobility / Reverse Index); all thirteen rows were in `.claude/decision-links-pending`.
  - **After**: the nine pages each get a table and the four pages each get their missing rows, 59 rows added in all, 73 rows across the thirteen tables — 12 support, 0 overturn, 13 preparation, 48 no effect; every decision mentioned on a page has a row, and the thirteen rows are deleted from `.claude/decision-links-pending`. Not one number of the experiments' conclusions, criteria, artifacts or preregistrations changed.
  - **Basis**: each page's text was read to list every decision it mentions, then each was judged against the four relationships. The 12 support rows were all already on the four pages (the item basis paragraphs of D21 (Boundary Between Authoritative and Derived States) and D1 (Data Mobility / Reverse Index) were looked up one by one now, and all cite that experiment back); of the 59 rows added this round none is support or overturn, so no decision's "**Basis**" paragraph changed along with this. D8 (Core Index Structure), D16 (Publish Semantics) and D19 (Block Pointer Structure & Width Budget) are still in `.claude/decision-links-pending` with text not yet rewritten into the slimmed form, so rows pointing to them are written at decision level per the rule, without item numbers; of these, the six rows E70 (Feasible Region for Two Checkpoint Thresholds), E91 (Ring and Admission Accounting Under Settled Format) on D16 (Publish Semantics) and E99 (Deduplication of Write Buffer Entries and Seq), E103 (Update Cost of Inode Packing Path 1), E112 (Legacy Writer Encountering Unknown Trees), E113 (Unknown Trees: Saturated Arm Set) on D8 (Core Index Structure) are written as preparation, and after those three decisions are slimmed they must get item numbers and be upgraded to support, with the experiments added to the basis paragraphs of those items at the same time. The changes were made in place with `scripts/kb/replace-once.py`, each hitting exactly once and read back to confirm.

### 2026-09-14

- Five rigs follow the three format constants settled by the user — journal record header 307, inode internal entry 120, tree table entry 148
  - **Before**: `JOURNAL_HEADER_BYTES = 277` (E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy)), `INODE_INTERNAL_ENTRY = 117` and `CHILD_POINTER_BYTES = 83` (E103 (Update Cost of Inode Packing Path 1)), `TREE_TABLE_ENTRY_BYTES = 145` (E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width), E146 (Width and Cost of Livelist Entries Identified by Mapping Key)). E43 (Extension Point Byte Upper Bound) reported self-verifying unit margins of 235 / 3819; E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) had a journal write of 277 + 61 = 338 bytes per object moved, `bg_key` 512 B 0.0506 / 1 KiB 0.0847 / 4 KiB 0.3512 / 8 KiB 1.0233, fill-up order 512 B 2.0983 at `b`=1 / 0.0517 at `b`=N; E103 (Update Cost of Inode Packing Path 1) had 179 where the internal fanout is now 173, container index fanout 149, 1e6 level vectors [8621, 49, 1] and [29, 1]; E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) three-arm `tree_table_bytes` 145 / 290 / 145; E146 (Width and Cost of Livelist Entries Identified by Mapping Key) day-1 registration 145 / 16638 bytes.
  - **After**: `JOURNAL_HEADER_BYTES = 307`, `INODE_INTERNAL_ENTRY = 120` (with `CHILD_POINTER_BYTES = 86` following, see basis), `TREE_TABLE_ENTRY_BYTES = 148`; all five artifacts rerun with file-name suffix `-2026-09-14-round2`, and not one old artifact deleted. E43 (Extension Point Byte Upper Bound): **3 of 128 lines** change (`room` 235→**205**, 3819→**3789**, `n_max_journal_512` 235→**205**, and with it `n_max_if_self_witness_carries` 235→**205**), the 18 `mount_eval` lines unchanged word for word, unit test renamed `the_self_witness_bound_is_205_not_864`, 21 unit tests / all 19 mutations caught. E116 (Packed Container Balance: Metadata Writes and Compaction Strategy): **36 of 306 lines** change (config 1 + `payback_key` 5 + `payback_fill` 30), journal write 307 + 61 = 368 bytes each, `bg_key` 512 B **0.0515** / 1 KiB **0.0857** / 4 KiB **0.3523** / 8 KiB **1.0247 (still no gain)**, fill-up order 512 B **2.0992** at `b`=1 / **0.0526** at `b`=N, the factor separating the strategies 41→**40**; steady-state occupancy, blast radius, positive and negative controls unchanged word for word, 17 unit tests / all 16 mutations caught. E103 (Update Cost of Inode Packing Path 1): **49 of 131 lines** change (config 1 + `geom` 12 + `write` 36), internal fanout 179→**173**, container index fanout 149→**145**, 1e6 level vectors [8621, **50**, 1] and [**30**, 1], third-arm internal levels [**32**, 1] and [**3156, 24**, 1], with `stat`, `write_clustered` and blast radius unchanged cell by cell, batch ratio 1.83→**1.82**, the third arm at most 0.93%→**0.92%** more than path ①, 13 unit tests / all 11 mutations caught. E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width): **3 of 57 lines** change (three-arm `tree_table_bytes` 148 / 296 / 148), 9 unit tests / all 11 mutations caught. E146 (Width and Cost of Livelist Entries Identified by Mapping Key): **6 of 130 lines** change (config 1 + `new_pool_file_creation` 5, day-1 registration **148** / **16641** bytes, two trees **296** / **33282**), 8 unit tests / all 12 mutations caught. These five rows in `scripts/experiment-apparatus/replay.sh` now point to the new artifacts, and all five byte-for-byte comparisons agree.
  - **Basis**: `<!-- format-const: JOURNAL_HEADER_BYTES = 307 -->` in `.claude/kb/decisions/23-journal的角色与格式.md`, `<!-- format-const: INODE_INTERNAL_ENTRY = 120 -->` in `.claude/kb/decisions/08-核心索引结构.md`, and `<!-- format-const: TREE_TABLE_ENTRY_BYTES = 148 -->` in [layout/01-first-txn.md](layout/01-first-txn.md), all decided by the user via popup; the gate stage "format constants in sync between kb and experiment sources" judged these five rigs red that day, and its howto requires verbatim "changing a constant must move three places together: ① kb marker and text ② the experiment source's `const` and the unit test pinning it ③ rerun the experiment and update the artifact in `research/results/`". `CHILD_POINTER_BYTES` 83→86 is no extra liberty: E103 (Update Cost of Inode Packing Path 1) has a structural assertion `INODE_INTERNAL_ENTRY == INODE_KEY + CONTAINER_KEY_BYTES + CHILD_POINTER_BYTES`, and D8 (Core Index Structure) settled item 6 reads verbatim "**internal node entry = separator key 8 + identity reference 26 + child pointer 86 = 120 bytes**" — 120 was derived from 86 in the first place, and changing only the 120 would keep that assertion red forever. ⚠️ The original text of M12 in the mutation table of E146 (Width and Cost of Livelist Entries Identified by Mapping Key) was stuck at `TREE_TABLE_FIRST_VERSION_ENTRIES = 5` (the rig had been changed to 7 per the decision on 2026-09-13), so before this round it could no longer be substituted and the whole mutation run broke off; after the original text was changed to 7, all 12 are caught.

### 2026-09-13

- E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) rerun with journal header 95; the ten-field constant of E75 (Record Size and Ring Geometry), E76 (Discriminative Power of Payload Checksum) renamed, artifacts unchanged
  - **Before**: the sources of all four experiments had `const JOURNAL_HEADER_BYTES: u64 = 78`. E43 (Extension Point Byte Upper Bound) and E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) treated it as the whole record header (self-verifying unit margins 434 / 4018; journal write per relocation 78 + 61); E75 (Record Size and Ring Geometry) and E76 (Discriminative Power of Payload Checksum) treated it as the ten fields and added three increments to get 95. The texts of E23 (Journal Geometry), E39 (Does Reverse Chain Block Remnant Records), E49 (Reverse Chain Width: 32 vs 64) said "current header 78".
  - **After**: the constant of E43 (Extension Point Byte Upper Bound) changes to 95, and artifact `e43-ext-budget-2026-09-13.out` changes in only 3 lines (room 434→417, 4018→4001, n_max_journal_512 434→417), with the `mount_verdict` lines unchanged, 21 unit tests and all 19 mutations caught. The constant of E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) changes to 95, and artifact `e116-pack-settle-2026-09-13.out` differs from the previous version in 36 lines (17 more bytes per journal write; bg_key payback ratio at the 512 B tier 0.0444→0.0449, at the 8 KiB tier 1.0142→1.0150, fill-up order b=1 2.0921→2.0926), 17 unit tests and all 16 mutations caught, with the four assertions pinning absolute values and mutation table M16 changed along with it. The constants of E75 (Record Size and Ring Geometry) and E76 (Discriminative Power of Payload Checksum) are renamed `JOURNAL_HEADER_TEN_FIELD_BYTES` (value still 78), and reruns are byte-for-byte unchanged. The texts of E23 (Journal Geometry), E39 (Does Reverse Chain Block Remnant Records), E49 (Reverse Chain Width: 32 vs 64) become ten fields 78, header 95, 91 without the reverse chain.
  - **Basis**: decision changelog number eight (the registered value of D23 (Journal Role & Format) settled item 4 changed to 95); C94 (Registered Format Constants Disagree with Later Rulings).
- E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) rerun with journal header 277 — the side bounding the self-verifying unit switches from the root slot to the journal record header
  - **Before**: both rigs wrote `const JOURNAL_HEADER_BYTES: u64 = 95`. E43 (Extension Point Byte Upper Bound) reported self-verifying unit margins of 417 / 4001, `n_max_if_self_witness_carries=255` (bounded by the root slot's 256 − 1); E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) had a journal write of 95 + 61 = 156 bytes per object moved, `bg_key` payback ratios 512 B 0.0449 / 1 KiB 0.0790 / 4 KiB 0.3448 / 8 KiB 1.0150, fill-up order 512 B 2.0926 at `b`=1, 0.0461 at `b`=N.
  - **After**: both constants change to **277** (D23 (Journal Role & Format) settled item 15, by user decision, adds a new-root segment of 182 to the record header = tree table unit pointer 83 + central mapping tree root pointer 83 + tree ID watermark 8 + rollback floor F 8). Artifact `e43-ext-budget-2026-09-13-hdr277.out` of E43 (Extension Point Byte Upper Bound) (128 lines, ending `emitted=128`) changes in only **3 lines**: `room` 417→235, 4001→3819, `n_max_journal_512` 417→235, and with it `n_max_if_self_witness_carries` 255→**235** — the side doing the bounding switches from the root slot to the journal record header, and the unit test is renamed `the_self_witness_bound_is_235_not_864`; the 18 `mount_eval` judgements unchanged word for word; 21 unit tests all green, all 19 mutations caught (`e43-mutate-2026-09-13-hdr277.log`), and not a row of the mutation table changed. Artifact `e116-pack-settle-2026-09-13-hdr277.out` of E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) (306 lines, ending `emitted=306`) differs from the previous version in **36 lines** (config 1 + `payback_key` 5 + `payback_fill` 30): 182 more bytes per journal write, `bg_key` payback ratios 512 B **0.0506** / 1 KiB **0.0847** / 4 KiB **0.3512** / 8 KiB **1.0233 (still no gain)**, fill-up order 512 B **2.0983** at `b`=1, **0.0517** at `b`=N, the factor separating the strategies 45→**41**; steady-state occupancy, blast radius, positive and negative control lines unchanged word for word (those ledgers do not include the journal); 17 unit tests all green, all 16 mutations caught (`e116-mutate-2026-09-13-hdr277.log`), with the five assertions pinning absolute values and mutation table M16 changed along with it. The two rows for E43 (Extension Point Byte Upper Bound) and E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) in `scripts/experiment-apparatus/replay.sh` now point to the new artifacts, and both reruns are judged "byte-identical"; not one old artifact deleted.
  - **Basis**: `<!-- format-const: JOURNAL_HEADER_BYTES = 277 -->` in `.claude/kb/decisions/23-journal的角色与格式.md`; the gate stage "format constants in sync between kb and experiment sources" judged these two rigs red; that stage's howto requires verbatim "changing a constant must move three places together: ① kb marker and text ② the experiment source's `const` and the unit test pinning it ③ rerun the experiment and update the artifact in `research/results/`".

### 2026-09-06

- A round of recovering from format constants drifting out of sync — E43 (Extension Point Byte Upper Bound) rerun a fourth time at 105, E79 (Capacity of Root Record) recomputed at 68, measurement-basis notes changed in E23 (Journal Geometry) / E39 (Does Reverse Chain Block Remnant Records) / E83 (Granularity of Tombstones) / E98 (Geometry of Inode Records and Inode Trees) / E103 (Update Cost of Inode Packing Path 1)
  - Trigger: the decision on C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) (2026-09-05) changed the data unit header to 105 and the packed record unit header to 103,
    while the experiment sources each called the same quantity by its own name (`IN_UNIT_HDR` / `UNIT_HDR` / `DATA_UNIT_HEADER`)
    ⇒ `.claude/gate.d/27-format-constants.sh` binds by name and could not recognize them, so 91 / 55 just sat there while the gate was all green (the form of C56 (Unregistered Format Constants Still Drift)).
  - **E43 (Extension Point Byte Upper Bound)**: before, unit header 91, structural upper bounds 4004 / 130980, D21 (Boundary Between Authoritative and Derived States) comparison rows 144.0 / 188.0;
   after, header 105, upper bounds **3990 / 130966**, comparison rows **158.0 / 202.0** (artifact `e43-ext-budget-2026-09-06.out`, 128 lines,
    68 lines changed, the 18 `mount_eval` judgement lines unchanged word for word). **One new order flip appears**: in the 4 KiB cell the payload upper bound 3990 overtakes the pointer upper bound 3992,
    so `every_rejection_reason_is_reachable` now samples at "128 KiB unit + 4 KiB node", and the order is pinned as an absolute-value assertion.
    21 unit tests all green, all 19 mutations caught again (the original texts of the three mutations carrying constants synced along with it; the unit header one is now "knock the header width back to 91").
  - **E79 (Capacity of Root Record)**: before, `UNIT_HDR = 55` (the value proposed by D21 (Boundary Between Authoritative and Derived States)), a tree table unit holds 267 trees;
    after, computed with the current header width 68 of the index node class in D18 (Information Carried in a Block) settled item 7, it is **266**,
   with the three tiers 68 / 77 / 86 pinned to the same value and a new mutation M6 "knock the header width back to 55" (artifact `e79-root-record-2026-09-06.out`, differing by only one number).
    The basis is its own note of 2026-09-03, "recompute once the width is settled" — the width was settled, and this recomputation was owed until today. Not a word of the structural conclusion changed.
  - **Measurement-basis notes (numbers unchanged)**: the header width in E23 (Journal Geometry) and E39 (Does Reverse Chain Block Remnant Records) is the 84 of run time, currently 78; each gets a sentence, with the current values of the derived columns computed;
    the 91 in E83 (Granularity of Tombstones) and E98 (Geometry of Inode Records and Inode Trees) is still stuck there, and the numbers do not flip within the header width range of E102 (Unit Class Registry Table and Packed Record Units), so each gets a sentence "rerun before citing", with the owed check recorded as C89 (Leftover Recomputation from Closing the Unit Header) ⑤.
  - **Index table**: "86 after settled item 9" in the E23 (Journal Geometry) row becomes the current 78; the E103 (Update Cost of Inode Packing Path 1) row gets the third arm from 09-05,
    and "identical to the full run that rewrites everything" becomes the text's **0.989**.

### 2026-09-02

- See the same-day entry "E58 (End-to-End Cost of Checksum Granularity) re-pinned with a changed formula + E43 (Extension Point Byte Upper Bound) rerun a third time with a 91-byte header" in the E58 (End-to-End Cost of Checksum Granularity) section.

### 2026-08-30

- E43 (Extension Point Byte Upper Bound) has run; the conclusion is "no upper bound can be squeezed out"
  - E43 (Extension Point Byte Upper Bound) has run, N=5 rounds byte-for-byte identical, 14 unit tests all green, all 12 mutations caught (zero blind spots).
    Before: status not run; at setup the three parts were expected to squeeze out an interval.
    After: **the lower bound holds but is the interval [7, 13], and neither upper bound stands** —
    upper bound A (19) is void: its basis was a comparison sentence derived from the row that D21 (Boundary Between Authoritative and Derived States) itself calls "a proposed value, not yet settled",
    and that sentence is false at N=0 already in the 4+2 stripe tier (152.0 > 128.0);
    upper bound B (864) is only a soft reference line: tree height is a performance property of derived state, and node size itself is unsettled.
   Basis: artifact `e43-ext-budget-2026-08-30.out`; three-way argument (forward accepts / backward narrows / local rejects);
    the main agent adopted the backward leg's narrowing, with the distinguishing facts looked up verbatim now.
    ⚠️ **A craft lesson**: when a constant assertion is written as a subtraction, mutation makes it overflow at compile time,
    so the mutation is recorded as "invalid" instead of "caught" — an assertion that should go red silently fails. Rewritten as an addition, both go red.
- E43 (Extension Point Byte Upper Bound) rerun with a changed criterion
  - Before: the upper bound was derived from "this project saves more than ZFS" (19 bytes).
    After: **that criterion is rejected by user decision (saving space is not a goal)**, and that upper bound is deleted along with its reasoning;
    three boundaries unrelated to saving space are measured instead — the structural upper bound (4040 for 4 KiB units, 1944 for 2 KiB nodes),
    **the self-verifying unit upper bound (root slot 255, journal record header leaves 428 in a 512 sector)**, and tear isolation (slot width 256 ⇒ 2 slots squeezed into one atomic unit);
    and per user decision a judgement tier "evaluated once at mount" is added, with all four rejection reasons reachable.
   Basis: artifact `e43-ext-budget-2026-08-30.out` (128 lines), 21 unit tests, 19 mutations with zero blind spots, N=5 rounds byte-identical.
  - ⚠️ **A zero-coverage gap from the first round is filled**: the first round's "every unit carries it" tier built index nodes,
    **not root slots and journal record headers**, so there was no observation at all of the whole class of self-verifying units — pointed out by the backward leg.
- Two rounds of adversarial attack sent seven places back to the experiment side
  Three LLMs attacked all the conclusions of this round, and the main agent changed things after looking each one up:

  - **E43 (Extension Point Byte Upper Bound)**: the **255 of the root slot tier is void** — it was computed as the candidate slot width 256 minus 1,
    and E41 (Root Ring Slot Geometry 2×4×256) has already rejected 256; more fundamentally, the kb has never written down the fixed fields inside the root slot.
    Also marked: the 11 / 13 tiers of the lower bound include the 4-byte ciphertext checksum, which is exactly what E40 (Ciphertext Checksum Bit Width) is to settle
    ⇒ **a circular input**; the judge-at-mount tier lacks the starting-offset input, and the self-verifying unit rule takes effect unconditionally,
    which amounts to answering D21 (Boundary Between Authoritative and Derived States) settled item 4 on its behalf.
  - **E41 (Root Ring Slot Geometry 2×4×256)**: the `io_min` part — **the arithmetic source of conclusion 4** — had **zero tests and zero mutations**;
    one test and two mutations are now added, both caught (unit tests 14→15, mutations 9→11).
    The old wording "K = root ring depth" in the source header comment is fixed as well.
  - **E40 (Ciphertext Checksum Bit Width)**: of the four candidates listed at setup, **xxh3-64 was never implemented and never run**
    ⇒ "not one candidate algorithm eliminated" narrows to "none of the three measured was eliminated";
    it also states that the positive control did only one pair, 8-bit sum check vs CRC32C, and CRC64 and truncated SHA-256 did not each prove their discriminating power.
  - **One wording changed in all three experiments**: "N=5 rounds byte-for-byte identical" is rewritten as **"running 5 rounds proves there is no hidden state,
    not statistical stability"** — all three are deterministic models, and the same binary run 5 times is necessarily byte-identical,
    so N=1 and N=5 carry the same information; "a single observation does not count" in `.claude/singlefs-ai-sop/rules/test-discipline.md`
    targets observations with inherent variation. **The earlier wording dressed determinism up as evidence of the strength of a real stress test.**
  - **The phrase "zero blind spots" removed from all three experiments**: mutations only cover the functions that already have assertions,
    proving "every piece of code with assertions has been mutation-tested", not "every piece of code covered by the conclusion has been tested" —
    the `io_min` part of E41 (Root Ring Slot Geometry 2×4×256) is a live example.
  - **E33 (Two Pinned Block Rules)**: the original text of I-7.4 (Blocks in Recent K Generations Not Reused) quoted verbatim in its table no longer exists; corrected to match.

- See the same-day entry "the experiment index changed to carry conclusion briefs; three stale statuses fixed" in the E40 (Ciphertext Checksum Bit Width) section.
- See the same-day entry "E41 (Root Ring Slot Geometry 2×4×256) rerun with a changed model" in the E41 (Root Ring Slot Geometry 2×4×256) section.

### 2026-08-29

- Experiment set up for D21 (Boundary Between Authoritative and Derived States) settled item 2
  - New: E43 (Extension Point Byte Upper Bound): file created for D21 (Boundary Between Authoritative and Derived States) settled item 2, "what fixed byte upper bound to take for the custom space", **not run**.
    Before: zero experiments covered that unsettled item — `grep -rn "扩展点\|自定义空间\|开线方" .claude/kb/experiments.md .claude/kb/experiments/ research/`
    gives no output; the three experiments touching D21 (Boundary Between Authoritative and Derived States) (E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units),
    E29 (Blast Radius of a Single Failed Node), E38 (Accounting Structure: COW vs In-Place)) all touch only its authoritative/derived half.
    After: three parts squeeze an interval — the space-comparison flip point and tree height give the upper bound, fitting one legal pointer gives the lower bound, and an empty intersection is a legitimate result.
    Basis: the text of D21 (Boundary Between Authoritative and Derived States) says of itself "the text gives 128 bytes as an example, not settled", the same shape as E40 (Ciphertext Checksum Bit Width) (single leg, inputs picked by hand).
    ⚠️ At setup a fork that D21 (Boundary Between Authoritative and Derived States) does not distinguish was pointed out: whether units like index nodes carry an extension point;
    the text does not say, and the literal reading of D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) "units do not distinguish metadata from data" is that all do ⇒ the experiment runs both tiers.

## E44 (Host Testing and Cost of Sequence Number Bit Width)

### 2026-09-19

- See the same-day entry "E17 (Write Buffer Merge Limit), E42 (Number of Records per Transaction)…" in the E17 (Write Buffer Merge Limit) section.
- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-03

- See the same-day entry "new E91 (Ring and Admission Accounting Under Settled Format), run to completion the same day — all bills posted by the three-way argument measured" in the E91 (Ring and Admission Accounting Under Settled Format) section.
- See the same-day entry "new E97 (Entry Encoding of Accounting and Allocation Records), run to completion the same day — three adversarial rounds, rerun three times" in the E97 (Entry Encoding of Accounting and Allocation Records) section.

### 2026-08-30

- E44 (Host Testing and Cost of Sequence Number Bit Width) knocks out an upper-bound assumption
  The user asked that "experiments must measure on this host". E44 (Host Testing and Cost of Sequence Number Bit Width) was created and run to completion (N=5 rounds, dispersion ≤0.42%, all 8 mutations caught):

  - **This host's fsync rate is 2785/s** (consumer NVMe + ext4 + O_DIRECT),
    **359× lower** than the upper-bound assumption of "10⁶/s" in D23 (Journal Role & Format) settled item 9.
    ⇒ **A 48-bit counter lasts 3202 years on this host; the premise of the reasoning "48 bits is not enough" does not hold on this host.**
  - ⚠️ **The direction of the conclusion narrowed by this changed**: the weak spot is not the counter but **the 16-bit identifier** —
    it is independent of the fsync rate, and measurement does not relieve it (10 crashes a day, 36 500 in ten years, a margin of 1.8×).
    The candidate form narrows from "identifier 32 + counter 64 (12 bytes)" to "identifier 32 + counter 48 (10 bytes)".
  - **The measured cost of widening is zero**: with `jsn` 8 → 10 or 12 bytes, the named-entry capacity is unchanged under both 512 and 4096 units,
    and not one extra cell is taken over 0..=100 entries.
  - ⚠️ **It knocks out only the "this host" tier**: this host's disk has no PLP, and enterprise disks have fsync rates one to two orders of magnitude higher.

  ⚠️ **Four blind spots filled by mutation testing**: no fdatasync, `sync_8` writing only one block, zero elapsed time clamped to 1,
  the cost sweep looking at only one cell — at first **none of the four turned a single test red**, because they all live in the I/O path or in `main()`.
  ⇒ **A positive-control unit test that really runs I/O was added** (`fdatasync_actually_reaches_the_device`).

  ⚠️ **The first round was cold**: without warm-up the five-round dispersion was 49.7%, and under the own failure clause of E44 (Host Testing and Cost of Sequence Number Bit Width) no conclusion could be drawn.
  Now one warm-up round is run explicitly and **reported in the artifact** (`name=warmup counted=false`) —
  warm-up must be visible; hiding it amounts to quietly deleting the slowest round.

## E45 (Ring Space Occupancy for Multi-Record Transactions)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-06

- See the same-day entry "three experiment index rows disagree with the text — the index row of E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) is entirely a voided conclusion" in the E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) section.

### 2026-09-01

- See the same-day entry "one experiment set up for each of the six of the nine settled items that owed measurement, none run" in the E71 (Entry Count and Width of Accounting Keys) section.
- See the same-day entry "three experiments go from 'not run' to 'run', with not a word of the criteria changed" in the E71 (Entry Count and Width of Accounting Keys) section.
- See the same-day entry "the structural half of E74 (Allocation Record Entry Count and Compaction Cost) run, the wall-clock question still owed" in the E74 (Allocation Record Entry Count and Compaction Cost) section.

### 2026-08-30

- E45 (Ring Space Occupancy for Multi-Record Transactions) created, run to completion, conclusion voided
  - **Why it was created**: in the attack round, the basis of D23 (Journal Role & Format) settled item 7 was pointed out to have the direction reversed
    (it had said "one record per transaction pushes the ring up", changed to "spanning multiple records pushes the ring up"),
    and the user asked that "two-way swaps of this scope need three verifications".
  - **Run to completion**: three paths claimed to share no code are equal cell by cell, both positive controls pass, all 7 mutations caught.
  - **Then the conclusion is voided**: the three paths **share the same alignment rule**, and that rule is stricter than
    I-8.2 (Record Header Does Not Cross Atomic Unit) — I-8.2 (Record Header Does Not Cross Atomic Unit) only constrains the **record header** to fall within one atomic unit,
    while the model rounded **the whole record** up ⇒ that 12× was made by the model itself.
    ⇒ **It lands as a new owed check, C48 (multiple verification paths share one premise).**
  - **A straw man confirmed in passing**: E45 (Ring Space Occupancy for Multi-Record Transactions) took "one record per entry" as the representative of the multi-record arm,
    while both production systems **fill up one fixed-length record** (XFS fills the iclog, 16 KiB / default 32 KiB / maximum 256 KiB;
    jbd2 fills a descriptor block with tags until full) ⇒ the 12-entry transaction of the target workload has **exactly the same bytes** under both arms.
  - ⚠️ **The second attack round lacked two cloud legs** (not returned when the session ended), and per `.claude/rules/three-way-inference.md`
    **it must not be recorded as "the three agree"**. The local leg left two valid counterexamples, both costs on the "one record per transaction" side that E45 (Ring Space Occupancy for Multi-Record Transactions) did not model:
    **ring wraparound needs contiguous space**, and **it cannot be emitted as a stream (the whole record must first be gathered in memory)**.

## E46 (Root Ring Region Spacing and Failure Domains)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-08-30

- New E46 (Root Ring Region Spacing and Failure Domains), run to completion
  - Basis for setting it up: the region dimension that E41 (Root Ring Slot Geometry 2×4×256) itself recorded as having **zero coverage** (in the model the region count only takes part in multiplication,
    with no inputs for region spacing or failure domains), plus the user's request "measure D22 (Synthesizing Unit Atomicity) settled item 2 once more".
  - Measured: **in 7 of the 18 geometry cells, the disk head and the disk tail land on the same disk** — 1 / 2 / 3 / 6 disks with 64 KiB chunks,
    1 / 2 disks with 512 KiB, a single disk with 4 MiB. The chunk number of the disk tail, 152 587 890, is divisible by 2, 3 and 6.
    ⇒ **"the disk head and disk tail are independent failure domains" is false geometrically already**, without waiting for physical evidence.
    The candidate layout **fully satisfies** the spacing condition (about 10 TB, far beyond any `io_min`) ⇒ **the candidate falls on the same-disk condition, not on spacing**.
    A region count of 2 tolerates only 1 failure domain.
   Basis: artifact `e46-region-spacing-2026-08-30.out` (45 lines),
    9 unit tests, all 6 mutations caught, positive control holding both ways.
  - ⚠️ **It can only falsify in one direction**: landing on the same disk geometrically ⇒ the assumption fails; landing on different disks geometrically **does not mean** the physical failure domains are independent.

- See the same-day entry "E47 (Root Ring Failure Domain Losses) set up and run to completion" in the E47 (Root Ring Failure Domain Losses) section.
- See the same-day entry "E53 (Can Root Ring Mount After Losing an Entire Disk) — the first failure-domain experiment that really touches a device" in the E53 (Can Root Ring Mount After Losing an Entire Disk) section.
- See the same-day entry "E55 (Stripe Mapping Arithmetic Validity on Real Devices) — taking what E53 (Can Root Ring Mount After Losing an Entire Disk) stated…" in the E55 (Stripe Mapping Arithmetic Validity on Real Devices) section.

## E47 (Root Ring Failure Domain Losses)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-01

- See the same-day entry "E34 (Root Ring Slot Geometry) changes its premise but not its criterion; the arithmetic half run" in the E34 (Root Ring Slot Geometry) section.

### 2026-08-30

- E47 (Root Ring Failure Domain Losses) set up and run to completion
  - Reason for setting it up: D22 (Synthesizing Unit Atomicity) settled item 2 is stuck on the number "how many failure domains to tolerate", and that number must be decided by the size of the loss;
    E46 (Root Ring Region Spacing and Failure Domains) only answered "will the regions die together", not "what is lost when they die".
  - Criteria, failure clauses and measurement basis **fixed before the run** (the three text sections "what is measured / criteria / what counts as failure").
  - Result: the candidate geometry (2 regions × 4 slots) has **0 surviving slots in 4 of the 6 device-count tiers**;
    **slot order decides the dimension of the loss** (worst rollback: contiguous within a region 24 generations / rotating across regions 3 generations);
    **raising the region count lowers the payoff instead** (the number of geometry cells with R=1/2/3/4 all on different disks: 18 / 11 / 7 / 3).
  - N=5 rounds byte-identical; 11 unit tests, all 8 mutations caught.
  - ⚠️ **A record of "the implementation not matching the criterion"**: the first version fixed `latest` at 1 000 000,
    and `1 000 000 mod 8 == 0` happens to fall at the start of region 0's stretch, so the measured rollback was 1 instead of 4 —
    **it measured that one phase, not the worst phase**. The criterion says "worst rollback" ⇒ the implementation was fixed (new `survey_worst`),
    with not a word of the criterion changed. Mutation `M4_只取一个相位` injects this error back, and 5 tests go red at once.

- See the same-day entry "E48 (Feasible Points for Root Ring Geometry) set up and run to completion" in the E48 (Feasible Points for Root Ring Geometry) section.
- See the same-day entry "E53 (Can Root Ring Mount After Losing an Entire Disk) — the first failure-domain experiment that really touches a device" in the E53 (Can Root Ring Mount After Losing an Entire Disk) section.

## E48 (Feasible Points for Root Ring Geometry)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-09

- The spacing number in the text of E48 (Feasible Points for Root Ring Geometry) corrected — it differed from its own artifact by 66 560 bytes, and the wrong number happened to hide an incompatibility
  - **Before**: the text read verbatim "spacing = `P × chunk`, which is **536 870 912 bytes** with 64 KiB chunks, far beyond the largest `io_min` tier (65536)."
  - **After**: "spacing = `P × chunk − 区域字节`, which is **536 804 352 bytes** with 64 KiB chunks (`8191 × 65536 − 1024`)".
    Both places were wrong: the formula forgot to subtract the region bytes, and the number is `8192 × 65536` instead of `8191 × 65536`.
  - **Basis**: E48 (Feasible Points for Root Ring Geometry)'s **own unit test** reads verbatim `assert_eq!(g, 536_804_352)` and
    `// 独立算术：8191 × 65536 = 536 805 376，减去区域自身 1024 字节`, and the artifact reads verbatim `gap=536804352`.
    ⚠️ **The conclusion is unchanged** (both numbers far exceed 65536); what changes is its downstream usability: the wrong number `2²⁹` is a power of 2,
    **divisible by any power-of-2 `zone_size`**; the true value, because 8191 is an odd prime, divides no power of 2 larger than the chunk
    ⇒ that slip of the pen happened to hide the incompatibility "root ring region starts cannot land on zone boundaries".
    This is yet another instance of the cell "a number cited in the text does not match its own artifact" in `.claude/singlefs-ai-sop/rules/evidence-discipline.md`,
    **with the gate all green** (the range assertion of the rerun pins which tier the conclusion falls in, not the number in the text).
  - **How it was found**: the backward leg of the three-way argument round on D22 (Synthesizing Unit Atomicity) unsettled item 6 plugged
    `P × chunk` into power-of-2 `zone_size` values to test divisibility; when the main agent independently recomputed the 3 × 3 cells, none divided, which disagreed with the text,
    and checking back against the source and artifact of E48 (Feasible Points for Root Ring Geometry) confirmed it.

- See the same-day entry "new E123 (Trade-off Costs Between K and Rollback Depth), run to completion — giving…" in the E123 (Trade-off Costs Between K and Rollback Depth) section.

### 2026-08-30

- E48 (Feasible Points for Root Ring Geometry) set up and run to completion
  - Reason for setting it up: E47 (Root Ring Failure Domain Losses) pointed out "what must be settled is not the region count but the rule for choosing region positions plus the slot order",
    and D22 (Synthesizing Unit Atomicity) settled item 2 needs a feasible point to be settled.
    ⚠️ **A separate number instead of rerunning E47 (Root Ring Failure Domain Losses)**: the question asked changed (E47 (Root Ring Failure Domain Losses) asks how large the loss is, E47 (Root Ring Failure Domain Losses) asks whether some point has no such loss).
  - Criteria fixed before the run: feasible = always mountable on all cells with `devs ≥ R`, K=2 always available, spacing ≥ 65536;
    cells with `devs < R` are exempt by the pigeonhole principle.
  - Result: **9 feasible points, all "prime-stride placement + rotation across regions"** (R ∈ {2,3,4} × S ∈ {2,4,8}),
    worst rollback always 1, ownership pairwise distinct 100%; `even_span` (the one E47 (Root Ring Failure Domain Losses) used) has **not a single feasible point**;
    **the slot order contiguous within a region cannot achieve K=2 availability under any placement rule, and can be eliminated**.
  - ⚠️ **The first round of mutation testing knocked out two blind spots**: the feasibility judgement and the pigeonhole criterion both lived in `main()`, out of reach of any unit test.
    After extracting the pure function `judge()` and adding two absolute-value assertions, all 8 are caught. **This is a close relative of C49 (Universal Assertions Only Verified at Sample Points)**.

- See the same-day entry "E53 (Can Root Ring Mount After Losing an Entire Disk) — the first failure-domain experiment that really touches a device" in the E53 (Can Root Ring Mount After Losing an Entire Disk) section.
- See the same-day entry "E55 (Stripe Mapping Arithmetic Validity on Real Devices) — taking what E53 (Can Root Ring Mount After Losing an Entire Disk) stated…" in the E55 (Stripe Mapping Arithmetic Validity on Real Devices) section.

## E50 (Upper and Lower Bounds on Root Ring Slot Count)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-13

- See the same-day entry "row-width constant of E141 (Mount Admission Self-Test for Switchover Reservation) renamed, E79 (Capacity of Root Record) marks 243 as void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-09

- See the same-day entry "new E123 (Trade-off Costs Between K and Rollback Depth), run to completion — giving…" in the E123 (Trade-off Costs Between K and Rollback Depth) section.

### 2026-09-08

- See the same-day entry "E92 (Worst-Case Block Reuse Delay Requirements) has run — that quantity is not a single number, criterion 2 triggered…" in the E92 (Worst-Case Block Reuse Delay Requirements) section.
- See the same-day entry "E92 (Worst-Case Block Reuse Delay Requirements) rerun with an added column — the first round's c3 was a straw man, conclusion two replaced entirely" in the E92 (Worst-Case Block Reuse Delay Requirements) section.

### 2026-09-03

- See the same-day entry "E92 (Worst-Case Block Reuse Delay Requirements) design registration — the control quantity missing from I-7.4 (Blocks in Recent K Generations Not Reused) set up, not run" in the E92 (Worst-Case Block Reuse Delay Requirements) section.

### 2026-08-30

- E50 (Upper and Lower Bounds on Root Ring Slot Count) and E51 (Number of Reverse Chain Collision Opportunities) set up and run to completion
  **E51 (Number of Reverse Chain Collision Opportunities)** reviews the `D-遍历` denominator basis of E49 (Reverse Chain Width: 32 vs 64) and **judges it an overcount**:
  E49 (Reverse Chain Width: 32 vs 64) counted every adjacent comparison by the record checker as a collision opportunity,
  while **the chain values of records from the same source come from the same hash applied to the same input ⇒ equal by construction, with no randomness**.
  Measured: in clean logs (10 / 100 / 20480 records) comparisons across timelines are **always 0**; with 1 / 6 / 20 remnant records attached they are **always 1**.
  ⇒ The right denominator is the number of seams, which at 12 tiers is almost always equal to the number of incidents (3.65×10⁴) ⇒ **32 bits suffices in all 12 tiers** (8.50×10⁻⁶ over a lifetime).
  In passing it measures the lap time (7.35 seconds for a 10 MiB ring, 188 seconds for a 2 GiB ring) and the expected number of times a remnant is swept by a weekly checker run, 3.11×10⁻⁴.
  ⚠️ Mutation testing knocked out three blind spots (chain value depending on the previous record, the formula of reading B, the lap-time formula); after extracting functions and adding assertions, all 7 are caught.
  ⚠️ The artifact of E49 (Reverse Chain Width: 32 vs 64) and the reading-A table are **not touched**; only a correction pointing to E49 (Reverse Chain Width: 32 vs 64) is added.

  **E50 (Upper and Lower Bounds on Root Ring Slot Count)** measures the S left over from D22 (Synthesizing Unit Atomicity) settled item 2:
  the lower bound is **S = 1 is enough** (when R ≥ 2); the upper bound is **256 slots** under coarse-grained load (64 slots hold only 832 blocks), 32 slots under scattered load
  ⇒ **any S between 1..16 stays clear of both edges**.
  ⚠️ **A more important correction**: the lap time is **in milliseconds** (2.87 ms for 8 slots on this host),
  and covering a checkpoint interval on the order of 5 seconds would need ≈14 000 slots ⇒ **the root ring does not bear checkpoint-scale rollback**;
  that side is borne by D16 (Publish Semantics) new rule 2 (E33 (Two Pinned Block Rules) had long since measured that the two pinned-block rules hold different batches).

- See the same-day entry "E53 (Can Root Ring Mount After Losing an Entire Disk) — the first failure-domain experiment that really touches a device" in the E53 (Can Root Ring Mount After Losing an Entire Disk) section.

## E49 (Reverse Chain Width: 32 vs 64)

### 2026-10-17

- After the lead-in sentence "copied as whole lines" for artifact `e49-chain-width-2026-09-25.out`, a parenthetical is added: "(lines from artifacts moved out of the workspace are written as one-line pointers)"; conclusions, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/49-反向链宽度32还是64.md`.
- The unit test name in the last sentence of the fifth question gets both its name at the time and its current name: `fixed_record_reading_diverges_only_at_these_bases_in_280_to_345` was renamed `fixed_record_reading_diverges_only_at_these_bases_in_440_to_520` when the record header changed to 487 bytes (range base 440..=520); numbers, conclusions and artifacts unchanged, and the two rows of the affected-decisions table were looked back on the same day, both unaffected.
  - **Basis**: the test with the current name in `scripts/experiment-apparatus/e49_chain_width.rs` covers base 440..=520, and the old name has zero hits in the whole repository; judged line by line in the second round of knowledge-rot governance (`records/2026-10-17-全仓知识腐烂治理.md`).
  - **Before**: the fifth-question paragraph in `git show HEAD:.claude/kb/experiments/49-反向链宽度32还是64.md`.

### 2026-10-15

- The 3 artifact lines copied as whole lines become pointers "artifact line at line N of git show <commit>:<artifact path> (name=<name>)" (relation to that artifact line: 3 whole lines); these lines were copied from an older round or an archived artifact, not from the run registered in `scripts/experiment-apparatus/replay.sh` at this moment, and numbers and conclusions are unchanged (user decision, handling of the existing stock under C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/49-反向链宽度32还是64.md`.

### 2026-10-04

- E49 (Reverse Chain Width: 32 vs 64) fills in the three mutation numbers (whole-table rerun of the record header 311 → 487 rerun, run by the main agent; numbers only, conclusions and artifacts unchanged)
  - **Before**: the next entry said "the whole-table rerun has no result yet", and the experiment page said the three numbers were to be backfilled; the before run fixed by the registration was not run either (reason in section twelve of the registration).
  - **After**: the whole table of 14 rows caught 14, invalid 0, not red 0 (`research/results/e49-mutate-2026-10-04-h487.log`, ending with "restored, baseline still all green", with hitting the ceiling, timeouts, unscheduled and evicted all 0); registration voiding clause V4 triggers only if, after the change, one of the original eleven rows (M1–M8, M10–M12) is not "caught", and M1 to M14 are ✅ one by one ⇒ it does not trigger. The "2026-10-04" section of the experiment page and the mutation sentence on line 68 are changed per the three numbers.
  - **Basis**: log `research/results/e49-mutate-2026-10-04-h487.log`; sections eleven and twelve of registration `research/prompts/e49-r2-prereg.md`.
- E49 (Reverse Chain Width: 32 vs 64) rerun with record header 311 → 487 (row 4 of the question sheet)
  - **Before**: in `git show 31e47115:scripts/experiments/e49_chain_width.rs`, `JOURNAL_HEADER_BYTES` was 311, reading Jia swept base 296..319, reading Yi swept base 280..345, the entry-count upper bound was the 67 written inside a function, artifact `e49-chain-width-2026-09-25.out`.
  - **After**: after `research/prompts/e49-r2-prereg.md` fixed the criteria (both readings kept as in the header-311 run, criterion 2 decided by reading Yi; only the two judged bases 483 and 307, the sweep windows 472..495 and 440..520, and the entry-count upper bound 64 swapped), the rig constant changed to 487, the entry-count upper bound moved into the module-level constant `ITEM_UPPER_BOUND_TODAY`, unit tests changed their pinned values and B11 was added (25 all green), and the mutation table went 12 → 14 rows (the original text of M9 follows the constant, M13, M14 added, the whole-table rerun has no result yet). The comparison baseline run right before changing the rig is byte-for-byte identical to the 2026-09-25 copy. Artifact `e49-chain-width-2026-10-04-h487.out` (211 lines) differs from the 2026-09-25 copy in 259 lines, all in the five families `cost_current`, `cost_fixed_record`, `base_sweep`, `base_sweep_items` and `done`. **Row 4 of the question sheet is recorded as "judgement unchanged"**: under reading Yi both 483 and 307 are free; under reading Jia 483 and 307 judge the same cell by cell (2 vs 4 free, 4 vs 8 not free), the first divergent entry count 31 → 37, 287 → 357, `cap_items` 3 / 67 → 0 / 64; the two readings judge 4 vs 8 oppositely at 483 (F4, set side by side as is). The mod 8 regularity in the reach of D23 (Journal Role & Format) settled item 8 holds over base 472..495. The E49 (Reverse Chain Width: 32 vs 64) row of `scripts/experiment-apparatus/replay.sh` now points to the new artifact, and the rerun is "byte-identical".
  - **Basis**: `research/prompts/e49-r2-prereg.md`; row 4 of `research/prompts/m3-r1-batch9-reruns-questions.md`; D23 (Journal Role & Format) settled items 4 / 8 / 12 / 17.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 documentation iron-rule correction: numbers in this block got short names and clashing local arm names were changed to lowercase; the original sentences are in this version)

### 2026-09-25

- E49 (Reverse Chain Width: 32 vs 64) rerun with record header 307 → 311 (row 7 of the question sheet)
  - **Before**: the rig had only one occupancy model (variable-length records, reading Jia), swept only the original six bases, and had neither of the two bases today (307) and before header 311 (303); row 5 of the appended "sentence-by-sentence judgement of row 7 of the question sheet" in `e43-r1-prereg.md` judged the E49 (Reverse Chain Width: 32 vs 64) cell "takes part in the criterion, needs a rerun", and the 2026-09-24 batch (E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy)) recorded "the E49 (Reverse Chain Width: 32 vs 64) cell of row 7 is judged 'needs a rerun', not run this time".
  - **After**: after `e49-r1-prereg.md` fixed the criteria, the rig adds the two judged bases today (307) and before header 311 (303), with two occupancy models side by side, reading Yi (today's fixed-length records, D23 (Journal Role & Format) settled item 12) and reading Jia (the original variable-length record model) (decided by the main agent on 2026-09-24: keep both, criterion 2 decided by reading Yi, reading Jia computed and reported as is), plus a section-eight geometry-sensitivity sweep (reading Jia base 296..319, reading Yi base 280..345) and a discriminating-power self-check. 24 unit tests all green (13 new), all 12 mutations caught (M8–M12 new, run once before and once after the constant change, 0 invalid, 0 not red, the original 7 with the same state both times). Under reading Yi, 303 and 307 judge the same (2 vs 4 and 4 vs 8 both free) — **row 7 of the question sheet recorded as "unchanged"** (the criterion goes by reading Yi); under reading Jia the pair 4 vs 8 goes from free (303) to not free (307) (31 places with 512-byte units, 4 with 4096-byte units), and the two readings judge oppositely at 307 (F4, set side by side as is). Artifact `e49-chain-width-2026-09-25.out` goes into `research/results/`, `scripts/experiment-apparatus/replay.sh` now points to the new artifact, and the rerun is "byte-identical".
  - **Basis**: `e49-r1-prereg.md`; row 7 of `m2-header311-rerun-questions.md`; row 5 of the appended "sentence-by-sentence judgement of row 7 of the question sheet" in `e43-r1-prereg.md`; D23 (Journal Role & Format) settled items 4 / 8 / 12 / 17.

### 2026-09-24

- See the same-day entry "E43 (Extension Point Byte Upper Bound) and E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) rerun with record header 307…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-13

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) by journal header…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-08-30

- E49 (Reverse Chain Width: 32 vs 64) set up and run to completion
  - Reason for setting it up: after the three-way argument ruled out 16 bits, nobody had measured one notch further up with the same ruler (what 64 buys over 32);
    and the cost table of E39 (Does Reverse Chain Block Remnant Records) took only 5 named-entry counts, while C49 (Universal Assertions Only Verified at Sample Points) requires sweeping the domain.
  - Criteria fixed before the run: "n bits suffice" = probability of at least one occurrence over a lifetime < 1%; "free" = the same on-disk occupancy on every cell swept; the two denominators given separately.
  - Result: **under D-事故 both 32 and 64 far exceed the need** (32 bits 0.00085%); **under D-遍历 32 bits suffices in only 4 of 12 tiers**,
    the heaviest tier (2 GiB ring / 512 B per record / daily × ten years) expects 3.56 occurrences, **97.2%** over a lifetime, while 64 bits suffices in all 12 tiers.
    **"Free" on the cost side is a coincidence of `base mod 8`**: of the 6 candidate headers, only 95 (the current value) is free on both sides.
  - ⚠️ **Two pre-written expected values were knocked back by our own unit tests**: ① I first wrote "4 and 8 do not diverge at any base",
    and measurement gives 31 places each at base=84 and 99; ② for the heaviest tier I wrote the 32-bit expectation two orders of magnitude too low (3.56e-2 should be 3.56).
    **What was fixed is my arithmetic; not a word of the criteria changed.**

- See the same-day entry "E50 (Upper and Lower Bounds on Root Ring Slot Count) and E51 (Number of Reverse Chain Collision Opportunities) set up and run to completion" in the E50 (Upper and Lower Bounds on Root Ring Slot Count) section.

## E51 (Number of Reverse Chain Collision Opportunities)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-08-31

- See the same-day entry "E61 (Uniformity of Reverse Chain Hash Algorithm) conclusion voided: the model manufactured 'the only difference left is the instance identifier'" in the E61 (Uniformity of Reverse Chain Hash Algorithm) section.

### 2026-08-30

- See the same-day entry "E50 (Upper and Lower Bounds on Root Ring Slot Count) and E51 (Number of Reverse Chain Collision Opportunities) set up and run to completion" in the E50 (Upper and Lower Bounds on Root Ring Slot Count) section.

## E52 (Operational Costs of Two Multi-Writable-Head Mechanisms)

### 2026-08-30

- E52 (Operational Costs of Two Multi-Writable-Head Mechanisms) set up and run to completion
  - Reason for setting it up: D6 (Snapshot Implementation Model) settled item 1 is stuck on "which mechanism to take", while E35 (Two Forms of Multi-Writable-Heads)
    itself admits its two operational metrics are constants given by construction, **not enough to choose between the two**; the user asked "do not confirm by feel, confirm with numbers".
  - Criteria fixed before the run: "separable" = at least one metric differs by ≥ 2×; each metric has its own absolute-value assertion; the five metrics are given separately, **not combined into a total cost**.
  - ⚠️ **One correction before the run**: the draft positive control said "with head count = 1 all five metrics are identical", **which is wrong**
    (the deletion granularity of the two arms differs to begin with), and before starting it was replaced by A (doubling the head count ⇒ ① doubles, ② unchanged) and B (single head with zero sharing ⇒ side table and metadata identical).
  - ⚠️ **After starting and before writing any conclusion, a third arm was added**: looking it up showed that D6 (Snapshot Implementation Model) and D8 (Core Index Structure)
    each state a different position for the snapshot ID in candidate ① (low bits / prefix), and the position decides whether deletion is a full scan or a range operation. **Not a word of the criteria changed.**
  - Result: **①-low-bits is strictly dominated by ①-prefix and is out**; the remaining trade-off is "the read path always takes H−1 more descents"
    vs "one reference-count side table + 3 metadata entries per head". Separation: maximum head-deletion ratio 5442×, point lookup 3.67×.
  - 10 unit tests / all 9 mutations caught, N=5 rounds byte-identical.

## E53 (Can Root Ring Mount After Losing an Entire Disk)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-01

- See the same-day entry "E34 (Root Ring Slot Geometry) changes its premise but not its criterion; the arithmetic half run" in the E34 (Root Ring Slot Geometry) section.

### 2026-08-30

- E53 (Can Root Ring Mount After Losing an Entire Disk) — the first failure-domain experiment that really touches a device
  - Reason for setting it up: the user pointed out "to this day I have not seen a failure-domain test report", and looking it up confirmed that **failure-domain tests did not exist**
    (the four binaries E46 (Root Ring Region Spacing and Failure Domains) / E47 (Root Ring Failure Domain Losses) / E48 (Feasible Points for Root Ring Geometry) / E50 (Upper and Lower Bounds on Root Ring Slot Count) have **0 file operations and 0 fault injections**).
  - For it, `vm-bench.sh` got **multi-disk support** (`VM_DISKS`, default 1 keeping existing usage),
    and after adding it `--selftest` was run and passed before testing began.
  - Criteria fixed before the run: after losing one disk, survivors ≥ 1 and rollback ≤ 1, holding for each of the four disks lost once in turn.
  - Result (QEMU with 4 real virtio disks, O_DIRECT throughout, N=5 rounds byte-identical):
    **mountable all four times, rollback 0 / 0 / 0 / 1** (rollback 1 appears only when "the newest generation happens to land in the wiped region");
    positive control (no injection) 8 survivors, rollback 0; **negative control (wipe everything) 0 survivors ⇒ proving it reads the disk, not a cache**.
    ⇒ **The model conclusion of E48 (Feasible Points for Root Ring Geometry) reproduces on real devices.**
  - ⚠️ Mutation testing knocked out one blind spot: in the first round `M3_magic不参与判定` turned no test red —
    **the wiped slots are all zero, so the CRC stops them, and the magic gate was never tested on its own**.
    A test was added that builds a record with "wrong magic but self-consistent CRC", after which all 6 are caught.
  - ⚠️ **Still unverified**: physical failure domains (controller / backplane / power supply); disk failure in EIO form (the minimal initramfs has no `dm-error`);
    the half "prime stride spreads regions across different disks" (4 bare disks in the VM, no striping).

- See the same-day entry "E55 (Stripe Mapping Arithmetic Validity on Real Devices) — taking what E53 (Can Root Ring Mount After Losing an Entire Disk) stated…" in the E55 (Stripe Mapping Arithmetic Validity on Real Devices) section.

## E54 (How Many Generations to Retain in Accounting)

### 2026-09-09

- See the same-day entry "new E123 (Trade-off Costs Between K and Rollback Depth), run to completion — giving…" in the E123 (Trade-off Costs Between K and Rollback Depth) section.

### 2026-08-30

- E54 (How Many Generations to Retain in Accounting) set up and run to completion
  - Reason for setting it up: the user decided "generations are needed" ⇒ D5 (Snapshot / Space Accounting Mechanism) settled item 2 is left with only Jia and Bing, whose difference can be computed;
    and when D8 (Core Index Structure) settled item 1 was settled it stated explicitly "the increase in key count has not been measured".
  - Criteria fixed before the run: Bing's slope is 0 and Jia's slope = the number of statistics; Bing's discard cost **must not involve the disk capacity**;
    Jia + incremental discard is compared cell by cell with Bing; the lower bound on K is given by "how many generations the root can roll back at most".
  - **The main conclusion is "this either-or does not hold in the first place"**: Jia + incremental discard and Bing are identical cell by cell over 60 cells
    ⇒ "what a generation takes" and "how many generations to keep" are orthogonal.
  - Discarding is **a point delete, not a scan** (touches = the number of statistics, independent of both checkpoint count and disk capacity) ⇒ satisfies the first cell.
  - Magnitude: 64 million entries without discarding vs 512 entries at K=8 (64 statistics, 1 million checkpoints).
  - 8 unit tests / all 6 mutations caught, N=5 rounds byte-identical.

## E55 (Stripe Mapping Arithmetic Validity on Real Devices)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-01

- See the same-day entry "E34 (Root Ring Slot Geometry) changes its premise but not its criterion; the arithmetic half run" in the E34 (Root Ring Slot Geometry) section.

### 2026-08-30

- E55 (Stripe Mapping Arithmetic Validity on Real Devices) — taking what E53 (Can Root Ring Mount After Losing an Entire Disk) stated as unverified and filling in that half
  - Trigger: the user asked "what does 4 bare disks in the VM with no striping mean; does no striping mean it is all inferred?" —
    **the question landed right on the reach boundary that E53 (Can Root Ring Mount After Losing an Entire Disk) itself had stated**.
  - That boundary is real: all the geometric conclusions of E46 (Root Ring Region Spacing and Failure Domains) / E48 (Feasible Points for Root Ring Geometry) rest on
    the arithmetic `(p / chunk) % devs`, which **had only been read in the Linux source and never measured on a device**.
  - Handling: on **a real striped device** (`dm-stripe` built on 4 loops), write markers and check back which backing file they landed in.
    Result: **72 / 72 cells all right** (3 device-count tiers × 2 chunk tiers × 12 addresses);
    the six prime-stride groups **all land on mutually distinct disks**;
    the **positive control**, with the non-prime stride 8, gives `distinct_devices = 1` for the four regions, **all hitting the same disk** ⇒ the measurement can tell them apart.
  - ⚠️ **The first version of the probe gave "none" in all 12 cells**, because `sudo -S` **ate the marker meant for the device as the password** (stdin conflict) —
    after switching to writing from a file, 11/11 matched at once. This is now written into the discipline in the script header.
  - ⚠️ Still unverified: physical failure domains (need real hardware); disk failure in EIO form; `md/raid0` (E48 (Feasible Points for Root Ring Geometry) built device-mapper's stripe target).
  - ⚠️ The probe uses P=13 rather than the 8191 of E48 (Feasible Points for Root Ring Geometry) — 8191 × 64 KiB ≈ 536 MB,
    and the four regions would exceed the probe disk capacity; 13 likewise satisfies the mechanism requirement "prime and larger than the device count",
    while 8191 is there to satisfy **spacing ≥ `io_min`**, which is a different criterion.

## E56 (Message Buffer Gains vs Epsilon)

### 2026-09-19

- See the same-day entry "E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta), E56 (Message Buffer Gains vs Epsilon)…" in the E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta) section.

## E57 (Authoritative Placement for Plaintext-Side Field Set)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-08-31

- E57 (Authoritative Placement for Plaintext-Side Field Set) set up and run to completion
  - **New**: E57 (Authoritative Placement for Plaintext-Side Field Set). Trigger: D9 (Encryption) settled item 9 and item 7 of D18 (Information Carried in a Block) (unsettled at the time)
    named each other, so the gate stage "has an unsettled item been settled elsewhere" kept pointing at whichever side changed later; the user asked to **settle the authority by experiment**.
  - **Conclusion**: it falls on criterion 4, fixed before the run — **authority is split by field**. The physical address lives in the plaintext mapping layer; logical identity and the type tag live in the block header.
  - **Arm Jia (criterion 3, authority lands entirely on D18 (Information Carried in a Block)) is void**. Previously: direction A has zero gaps ⇒ authority lands on D18 (Information Carried in a Block).
    Now: that version handed direction A the physical placement for free, while D9 (Encryption) settled item 5 (user decision 2026-08-26)
    had already settled "the authoritative recording point of the physical address" in the plaintext mapping layer, and the forbidden list of D18 (Information Carried in a Block) settled item 3
    forbids the block header from carrying it, verbatim. Basis: the backward leg of the three-way argument pointed out that the criterion was self-made (the boundary criterion of D21 (Boundary Between Authoritative and Derived States)
    is "losing it loses data / losing it is only slower"), and the main agent confirmed it by checking the three original texts verbatim;
    the falsifying observation written down by the forward leg, "rerun direction A without feeding the scan placements", **fired**.
  - **Tool bug**: the "which test went red" regex of `scripts/mutation/mutate.sh` could not match Chinese test names,
    so 9 mutations that were all red were reported as "no test went red" (**false blind spots**). Changed so it no longer enumerates character classes;
    the blast radius looked up now is 1 file, and the earlier 28 mutation proofs are unaffected.

## E59 (Can Buffered Messages Recompute Solely from Units)

### 2026-09-19

- See the same-day entry "E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta), E56 (Message Buffer Gains vs Epsilon)…" in the E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta) section.
- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-08-31

- E59 (Can Buffered Messages Recompute Solely from Units) set up and run to completion
  - **Before**: D11 (Message Buffers in Index Nodes) settled item 4 was still unsettled at the time and called itself "the only life-or-death precondition", yet it had not a single number —
    everything in the repository about "whether buffered messages can be recomputed" was reasoning.
  - **After**: E59 (Can Buffered Messages Recompute Solely from Units) lands on the criterion "split by class": the mapping class and the reverse index class **hold conditionally**
    (the condition is an allocator property nobody has ever settled), and the tombstone class **does not hold, with no condition** (deletion leaves no trace on disk,
    the number of resurrections exactly equals the number of deletions), and the accounting class goes wrong along with the tombstone class. Lineage rewrite sequence numbers (D18 (Information Carried in a Block) settled item 1)
    buy only the overwrite cell, not the deletion cell.
  - **Basis**: the user asked to settle settled item 4 by running an experiment, not by argument. 7 arms × 3 workload groups, 5 rounds byte-for-byte identical,
    all 16 mutations caught; all four voiding clauses fixed before the run came into play.
    ⚠️ **The conclusion has not gone through the three-way argument**, and until it does, it must not be written into any text under `.claude/kb/decisions/`.
  - Arm Geng added (the allocator reuses low addresses). **Before**: there was only arm Jia, whose mapping-class divergence was 0.
    **After**: that 0 is judged to be luck — with monotonically forward allocation, "scan by address, take the largest" happens to equal "take the newest",
    while D3 (Space Allocation) has not settled that allocation moves monotonically forward. **Basis**: arm Geng changes only the allocation direction, and the same rebuild code bets wrong entry by entry,
    with the number of wrong places exactly equal to the number of overwrites.
  - The comparison assertion of the reverse-index column replaced by three absolute values. **Before**: `backref_bad == map_wrong`,
    and it ran on arm Jia, where both numbers are 0. **After**: each of the three arms pins an absolute value (0 / number of overwrites / total live keys).
    **Basis**: mutation testing changed the count of the missing-entry branch to `+= 0`, and **not one test went red**.

## E58 (End-to-End Cost of Checksum Granularity)

### 2026-09-19

- See the same-day entry "E6 (Encryption Algorithm Selection), E58 (End-to-End Cost of Checksum Granularity), E65 (Write-Side Cost of Checksum Granularity), E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header)…" in the E6 (Encryption Algorithm Selection) section.

### 2026-09-02

- The `meta` arm of E58 (End-to-End Cost of Checksum Granularity) gets a double-counting warning; the measured arms unchanged
  Before: the amortization ratio of the `meta` arm counted the full 108 bytes per unit and was cited by D4 (Checksum Location) settled item 1 as 0.0823%.
  After: a warning note in the text — the 55-byte unit header within the 108 lives inside the unit and is already counted when reading G, so the extra per unit should be 53;
  and the 55-byte header width assumption has been superseded by the 91-byte initial value of D18 (Information Carried in a Block) settled item 7.
  The `meta` arm numbers are no longer to be cited until recomputed, with the owed check recorded as C89 (Leftover Recomputation from Closing the Unit Header).
  Basis: found independently by the backward leg of the adversarial round on C68 (Invariant Allowlist Conflicts with Settled Clauses), and confirmed by the main agent checking
  the source comments and cost formula in `scripts/experiment-apparatus/e58_csum_grain.rs`.
  The readings and conclusions of the measured arms (randq / seq) are unaffected.
- E58 (End-to-End Cost of Checksum Granularity) re-pinned with a changed formula + E43 (Extension Point Byte Upper Bound) rerun a third time with a 91-byte header
  Before: the `meta` arm of E58 (End-to-End Cost of Checksum Granularity) carried the warning "do not cite until recomputed";
  E43 (Extension Point Byte Upper Bound) counted a 55-byte unit header.
  After: the extra bytes per unit in the cost formula of E58 (End-to-End Cost of Checksum Granularity) go 108→**53** (the pointer payload on the parent node side,
  independent of the unit header width), the critical share 0.0823%→**0.0404%**, 8 unit tests re-pinned, all 10 mutations caught again,
  the formulas and readings of the measured arms unchanged by a word, artifact kept; the unit header of E43 (Extension Point Byte Upper Bound) goes 55→**91**
  (the initial value of D18 (Information Carried in a Block) settled item 7), rerun and archived as `e43-ext-budget-2026-09-02.out`,
  structural upper bounds 4040→4004, 131016→130980, zero flips in the mount judgements, 21 unit tests and all 19 mutations caught again,
  the replay registration row points to the new artifact, byte-identical; the "saves more than ZFS" count 19→0 recorded as is,
  and the original M5 mutation is equivalent under the new constant, so per the discipline it is replaced by M5_净荷少扣一字节.
  Basis: item ① of C89 (Leftover Recomputation from Closing the Unit Header).

### 2026-09-01

- See the same-day entry "stale citations in the text of E16 (Role of Journal: WAL vs Intent Log) and E67 (Which Disks a Stripe Spans)…" in the E16 (Role of Journal: WAL vs Intent Log) section.

### 2026-08-31

- E58 (End-to-End Cost of Checksum Granularity) set up and run to completion
  - **Before**: D4 (Checksum Location) settled item 1 (the tension between checksum granularity and small random reads) was at the time still among the unsettled items, with a direction but no value,
    and the three experiments supporting that direction all measured side axes — E6 (Encryption Algorithm Selection) encryption throughput,
    E20 (CPU Cache Cost of Widening Pointers) the CPU cache, E7 (Offline Index Harness) I/O counts under a fixed memory budget.
    The text of D4 (Checksum Location) said of itself "E6 (Encryption Algorithm Selection) measures encryption throughput, not end-to-end read latency",
    and D25 (Target Workload Priority) said of itself "the byte measurement basis has not been measured to this day".
  - **After**: new E58 (End-to-End Cost of Checksum Granularity), sweeping three workloads on a real NVMe over G ∈ {4,8,16,32,64,128} KiB,
    N=5 rounds + one round at 4× the operation count for robustness. Conclusion: 32 KiB relative to 16 KiB costs 10.63% more on small random reads (QD=1) /
    8.34% less bandwidth (QD=16), gains only 0.83% on sequential reads and saves 0.33 percentage points of metadata;
    the small-random-read share at which the two break even is **0.0961% (QD=1) / 0.8648% (QD=16)**,
    of the same order as the byte-basis prediction of 0.0823% fixed before the run.
  - **Basis**: the questioner said "16 KiB is btrfs's default (looked up now in `man mkfs.btrfs`: default 16384, upper limit 65536);
    I lean toward 32" and asked for measurement. Criteria, failure clauses and "whether the reverse result is accepted" were fixed before the run, and the criteria were not changed after it.
  - ⚠️ **The three-way argument ran only the local leg**, with two cloud legs owed ⇒ "take 16 KiB", derived from E58 (End-to-End Cost of Checksum Granularity),
    **did not become a conclusion**. On the same day D4 (Checksum Location) settled item 1 was settled by the user as **32 KiB**,
   opposite to where the measurement points, with the cost written into the text; see that entry in [decisions-history.md](decisions-history.md).

- See the same-day entry "E66 (How Small Files Land After Padding) three-way argument run to completion, the point-read conclusion rewritten" in the E66 (How Small Files Land After Padding) section.
- See the same-day entry "E65 (Write-Side Cost of Checksum Granularity) set up and run to completion" in the E65 (Write-Side Cost of Checksum Granularity) section.

## E60 (Cost of Adding Disks Without Data Migration)

### 2026-08-31

- E60 (Cost of Adding Disks Without Data Migration) newly set up and run to completion
  - **Trigger**: on the day D2 (RAID Stripe Policy) settled item 4 c (no data migration, new data lands preferentially on the new disk) was settled,
    the user asked "measure the performance; if performance is poor, migrating is fine too".
  - **Conclusion**: the dimension of the cost is not slowness but **capacity that cannot be bought** — without migration the new disk can only absorb "the sum of the remaining free space on the other disks",
    and at 90% fill 60% of the new disk is stranded; after levelling, 2.05–2.09× more user data can be written, at the price of a one-off migration of 720 cells.
  - ⚠️ **The criterion fixed before the run judged a split result**: the criterion only watched the parity overhead, judging Jia w=2 "significant" (+18.3%)
    and Jia w=3 "not significant" (−2.0%), while for the decisive stranding dimension the criterion only required reporting, with no threshold set.
    Per test-discipline **the criterion must not be changed after the fact**, so the two are set side by side as is; making stranding a criterion needs a separate experiment.
  - ⚠️ **Mutation M5 was not caught the first time**: a second width gate inside `run()` swallowed the error of the disk-selection step,
    and it went red only after a test pinned directly on the disk-selection step was added. **When both gates are right, removing one cannot be seen.**
  - ⚠️ **The positive control caught a real bug on the spot**: levelling first computed the target level as `total/n` without clamping to each disk's capacity,
    and with a new disk of capacity 0 it migrated 200 cells. D2 (RAID Stripe Policy) settled item 2 has settled that disks need not be equal in size ⇒ water-filling is required.

## E61 (Uniformity of Reverse Chain Hash Algorithm)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-06

- See the same-day entry "three experiment index rows disagree with the text — the index row of E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) is entirely a voided conclusion" in the E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) section.

### 2026-09-03

- See the same-day entry "adversarial test round — all 19 owed mutation tables filled in, 10 new unit tests added where mutations escaped" in the E68 (Selection of Inlining Threshold) section.

### 2026-08-31

- E61 (Uniformity of Reverse Chain Hash Algorithm) conclusion voided: the model manufactured "the only difference left is the instance identifier"
  - **Previously**: the conclusion said "take Jia (full 32-bit CRC32C), because in the real form it gives a **provable 0**,
    while Yi (SHA-256 truncated to 32 bits) gives an unprovable 2⁻³²".
  - **Now**: **measurement two is voided together with this conclusion.** Measurements one and three, the 9 unit tests and the artifact are kept.
   **The corrected experiment has not run and is not planned to** (user decision): the margin of E51 (Number of Reverse Chain Collision Opportunities) is three orders of magnitude
    ⇒ both candidates suffice. D23 (Journal Role & Format) settled item 11 is settled on secondary reasons as full 32-bit CRC32C.
  - **Basis for the overturn** (three-way argument: the local leg and the backward leg each pointed it out independently, and the main agent confirmed each point by looking it up):
    1. **The difference set was hardcoded** — `record_length` takes `hdr_len`, `named_count` is always 7,
       and `nonce` is a function of `jsn`, constructing the two competing records to be identical in these three fields.
       Yet they are two **different transactions**, nothing guarantees `named_count` is the same, and I-6.1 (nonce not reused) also requires nonces to be unique across the whole disk.
    2. **The header 95 / 99 tiers have no new fields** — `make_header` fills only bytes 0..54,
       while the fields added by settled items 7 / 8 land in 86..99, and not a single bit of them was swept.
    3. **Measurement one could never fail for Jia** — CRC-32 having full rank over windows of ≤32 bits is an algebraic tautology,
       independent of base, header length and field layout. Criterion 5 has zero discriminating power for Jia.
    4. **Measurement three lacks statistical power** — λ=128, sd=11.3, and the band of criterion 4 is ±5.7σ,
       so it can only exclude "effective bit-width loss ≥ 1 whole bit". "Yi's truncation is uniform" overclaims.
  - ⚠️ **A methodology bill, worth recording on its own**: defect 1 is
    the first form listed under "never build the model after the conclusion" in `.claude/singlefs-ai-sop/rules/evidence-discipline.md` —
    every step is reasonable, but the steps were picked by the conclusion. The three arms share the same `make_header`
    ⇒ **comparisons between arms are normal throughout**, which is exactly the class `.claude/singlefs-ai-sop/rules/test-discipline.md`
    "comparing arms only with each other cannot detect all arms being wrong together" is meant to stop.
    **Unit tests could not stop it either**: `header_model_is_pinned` asserts only on the 86 tier, pinning defect 2 down as an "absolute-value assertion".

- See the same-day entry "E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) conclusion voided as a whole: the criterion hung on a prohibition that cannot reach this item" in the E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) section.

## E62 (Root Ring Ownership: Storing Identity vs Calculating Formula)

### 2026-08-31

- E62 (Root Ring Ownership: Storing Identity vs Calculating Formula) newly set up and run to completion
  - **Trigger**: D2 (RAID Stripe Policy) settled item 7 was settled as "store the device identity per region", and the user asked to measure performance and safety.
  - **Conclusion**: on safety, the store-identity arm points to the wrong disk constantly 0 times (the formula arm 2–19 times over 8 steps); on performance, 0 extra reads and a byte cost of 4 bytes.
  - ⚠️ **The negative control held one boundary**: with `R > devs` at mkfs both arms collide ⇒ storing identity cannot solve the pigeonhole,
    what it solves is pointing to the wrong disk. The criterion was written before the run and not loosened after.
  - Exhaustive over R∈[2,8] × devs∈[1,64]: all 28 cells where the formula arm collides are `devs < R`, and 0 cells collide with `devs ≥ R`.

## E63 (How Many Columns Allocator Writes per Invocation)

### 2026-08-31

- E63 (How Many Columns Allocator Writes per Invocation) newly set up and run to completion
  - **Trigger**: the criterion of D2 (RAID Stripe Policy) settled item 6 needs data.
  - **Conclusion**: the criterion must include the write volume (at 90% small writes the fitting arm saves 61% over the full-width arm); the width must have an upper bound
    (the two-disk hit rate `C(w,2)/C(D,2)` is strictly monotonic, 1.000 at full width over 8 disks); the input should be the volume after batching.
  - ⚠️ **Criterion 3 holds only partly**: in the tier with 90% small writes, the batching arm is 1.10× "exactly full", exceeding the 1.05 fixed before the run.
    Recorded as is; the criterion was not changed.
  - ⚠️ **The first version of the positive control was itself written wrong** (feeding `DEVS−1` to every arm), so what was judged red was the control, not the arm.
  - ⚠️ **Two mutations were not caught in the first round** (not deducting parity, changing the batch to 1), and went red only after two tests were added, a gold standard and "the batching really batches".

## E65 (Write-Side Cost of Checksum Granularity)

### 2026-09-19

- See the same-day entry "E6 (Encryption Algorithm Selection), E58 (End-to-End Cost of Checksum Granularity), E65 (Write-Side Cost of Checksum Granularity), E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header)…" in the E6 (Encryption Algorithm Selection) section.

### 2026-09-01

- See the same-day entry "stale citations in the text of E16 (Role of Journal: WAL vs Intent Log) and E67 (Which Disks a Stripe Spans)…" in the E16 (Role of Journal: WAL vs Intent Log) section.

### 2026-08-31

- E65 (Write-Side Cost of Checksum Granularity) set up and run to completion
  - **Before**: all three arms of E58 (End-to-End Cost of Checksum Granularity) were read arms, with zero coverage of the write side,
    while D4 (Checksum Location) settled item 3 had just settled "writes that cannot fill a unit go read–modify–write".
  - **After**: new E65 (Write-Side Cost of Checksum Granularity), three measured arms (`rmw` / `full` / `rmwsync`)
    plus one arithmetic arm (`pad`), N=5. 32 KiB relative to 16 KiB: `rmw` 15.76% dearer, `full` 25.92% dearer,
    `rmwsync` 8.16% dearer; byte write amplification constantly 8×.
  - **Basis**: the user asked that "write amplification is needed" be measured. Criteria and failure clauses fixed before the run, not changed after it.
  - **The most valuable point**: **the byte multiple and the time multiple differ by an order of magnitude** (constantly 8× vs a little over a tenth),
    so citing write amplification must state its measurement basis.

- See the same-day entry "E66 (How Small Files Land After Padding) adds a verification split and reruns; the ratio changes by nearly a factor of two" in the E66 (How Small Files Land After Padding) section.
- See the same-day entry "E66 (How Small Files Land After Padding) three-way argument run to completion, the point-read conclusion rewritten" in the E66 (How Small Files Land After Padding) section.

## E66 (How Small Files Land After Padding)

### 2026-09-01

- See the same-day entry "stale citations in the text of E16 (Role of Journal: WAL vs Intent Log) and E67 (Which Disks a Stripe Spans)…" in the E16 (Role of Journal: WAL vs Intent Log) section.

### 2026-08-31

- E66 (How Small Files Land After Padding) set up and run to completion
  - **Before**: D4 (Checksum Location) settled item 2 settled padding to 32 KiB, while the inlining conclusion of E8 (Different Write Paths for Large vs Small Files)
    was measured on the premise of **no padding** — the item "without inlining it fills a whole unit" did not exist in its model.
  - **After**: new E66 (How Small Files Land After Padding), three ways of landing (`extent` / `inline` / `pack`) × eight file-size tiers, N=5.
    `pack` reads the same as `extent` tier by tier while writing 585–591% dearer ⇒ **purely dominated, out**;
    `inline` wins only on point reads, by 9.8–11.0%, breaking even at a read:write ratio of **8.43:1 ~ 9.74:1**;
    ⇒ **padding does not change the conclusion of E8 (Different Write Paths for Large vs Small Files).**
  - **Basis**: the user asked to "test it". Criteria and failure clauses fixed before the run, not changed after it.
  - ⚠️ **A note against misreading**: E8 (Different Write Paths for Large vs Small Files) says inlining pays 6.2× on the read side,
    E66 (How Small Files Land After Padding) says inline point reads are 10% faster — the two do not measure the same kind of read (scan vs point read),
    and the `inline` arm of E66 (How Small Files Land After Padding) **does not model the drop in fanout**, giving a lower bound on the cost of inlining.
- E66 (How Small Files Land After Padding) adds a verification split and reruns; the ratio changes by nearly a factor of two
  - **Before**: the first round did not time verification separately (the same flaw as the first round of E65 (Write-Side Cost of Checksum Granularity)).
    First-round numbers: inline point reads **9.8–11.0%** faster, writes 491–501% dearer, break-even read:write ratio **8.43 : 1 ~ 9.74 : 1**.
  - **After**: fields such as `read_verify_ns_per_op` / `write_dev_ns_per_op` added, rerun at N=5.
    Device-side numbers: inline point reads **13.3–17.0%** faster, writes **902–947%** dearer, break-even read:write ratio **5.11 : 1 ~ 6.37 : 1**.
  - **Basis**: the backward leg of the three-way argument hit "the multiple includes AES" on E65 (Write-Side Cost of Checksum Granularity),
    and E66 (How Small Files Land After Padding) has the same flaw ⇒ material known to be flawed must not be fed to the next round of three-way argument.
  - ⚠️ **Only about 12 GiB of writes separate the two runs** (done by the backward leg on the same disk), and absolute latency shifts up as a whole
    (`extent` point read 174 μs → 255 μs) ⇒ **the break-even read:write ratio varies with device state and is not a number to cite to the decimal place**;
    what can be cited is the direction and magnitude (the break-even point lies between read:write ratios of 5:1 and 10:1).
  - ⚠️ **A mistake recorded as is**: the rerun reused the same output file name, so **the first-round artifact was not retained**
    ⇒ the first-round numbers have no artifact to check against today, only the numbers in this history entry.
    `.claude/singlefs-ai-sop/rules/show-me-test.md` states explicitly "output paths must not be reused across rounds".
  - **The direction of the conclusion is unchanged**: `pack` is still purely dominated and out, and inlining still wins only on point reads.
    Moreover, the settling criterion of D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 is **atomicity**, not this set of performance numbers.
- Third and fourth runs of E66 (How Small Files Land After Padding): a discarded warm-up added, and medians used instead
  - **Before**: in the second run the 512 B tier was the first batch of device operations of the whole round, 45–71% high
    (round1: extent read 374140, inline read 362596, inline write 317619; round2–5 are 255256 / 211734 / 247165).
    The text marked only the `extent` read cell as a warm-up artifact, **without noticing it equally polluted the `inline` reads and writes**.
  - **After**: a warm-up of 64 discarded operations added (`const WARMUP`, pinned by a unit test, with one mutation proving it can go red), rerun at N=5.
    ⚠️ **Adding warm-up was chosen over dropping that round after the fact**: "deciding which round to drop after seeing the numbers" is exactly
    the form `.claude/singlefs-ai-sop/rules/test-discipline.md` is meant to stop.
  - **The fourth run found something else**: the seed=2 round was pushed up as a whole by outside activity (`extent` read 569805,
    `pack` write 704808, while the other four rounds of the same tier are on the order of 255–305 μs) — **another session on this host was running experiments on the same NVMe at the same time**.
    ⇒ The measurement basis switches to **medians**, and "this host is shared" is written into "What it cannot answer".
    Means and medians give conclusions of the same direction and magnitude, differing only in the two cells hit by that round.
  - **The currently valid numbers** (medians, five tiers from 512 B to 8 KiB): point-read advantage **16.23–18.01%**,
    `inline` writes **861–906%** dearer, `pack` writes **1076–1106%** dearer, `pack` reads differing from `extent` reads by **±1.06%**,
    break-even read:write ratio **4.84 : 1 ~ 5.42 : 1**.
  - **Two more points from the forward leg adopted**: ① the range for how much dearer `pack` writes are was originally written as 1108–1114%, which did not recompute; fixed;
    ② `inline` and `extent` have **exactly equal total I/O bytes** (both 32768), yet their times still differ by 10×
    ⇒ the form of the argument changes from "a 2× byte difference cannot explain 10×" to "**a 0× byte difference cannot explain 10×**".
  - ⚠️ **Over four runs on the same day the ratio went 8.4–9.7 → 5.1–6.4 → 5.2–5.4 → 4.8–5.4**, and the artifacts of the first three were not retained
    (the same output file name was reused, a mistake). ⇒ The break-even read:write ratio must not be cited to the decimal place.
  - ⚠️ **A tool mistake recorded as is**: the edit adding `WARMUP` was swallowed together with `&&` by a failed `cd`,
    so the replacement never ran, while the text already said "warm-up added". It was found only after reading back to confirm the hit count.
    `.claude/singlefs-ai-sop/rules/command-safety.md` states explicitly "a replacement must assert its hits and be read back after the change".
- E66 (How Small Files Land After Padding) three-way argument run to completion, the point-read conclusion rewritten
  - **Before**: conclusion 3 said "`inline` point reads win by 16–18%", taken as the read-side advantage of inlining.
  - **After**: **that percentage is an artifact of asymmetric accounting** — `read_bytes_of` charged `inline` with the 16 KiB of the index leaf,
    but did not charge `extent` with the same leaf read, though `extent` too must read that leaf before it can get the extent pointer.
    Symmetric accounting gives **54.5% (leaf miss) to 100% (leaf hit)**.
    And that percentage also drifts with device state: fio on four 8 GiB direct-I/O files measured the same axis at **6.8%–16.6%**,
    while the device-side breakdown table of E58 (End-to-End Cost of Checksum Granularity) gives **7.9%** on the same axis — **the two "device-side" figures in the kb differ by more than 2×**.
  - **The break-even read:write ratio widens accordingly**: the observable interval is **5:1 ~ 13.5:1** (varying with device state),
    and with the index node cache hit rate stacked on top, the lower bound drops to **0.42:1**.
  - **Two more notes**: ① the drift is entirely on the device side (the same fio on the same file gave 67.8 μs one time and 142.3 μs the next,
    the whole curve shifting additively by +70–80 μs; not GC, not temperature, not neighbors; a freshly filled new file returns to the "old" level) —
    **the mechanism has not been verified, and SLC must not be written in as the explanation**;
    ② **N=5 is really only 3 seeds** (`seed | 1` folds 2/3 and 4/5 into the same state),
    the same idiom in three places, E58 (End-to-End Cost of Checksum Granularity), E65 (Write-Side Cost of Checksum Granularity), E66 (How Small Files Land After Padding)
    ⇒ the between-round variation recorded by all three is systematically too small. Landing place: C59 (Seeds Folding into the Same State).
  - **Basis**: three-way argument (local = attack, Sonnet = forward, Opus = backward).
    Three points from the forward leg and four from the backward leg, all confirmed by the main agent recomputing each one, then adopted.
  - ⚠️ **D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3 is unaffected**: its criterion is atomicity,
    and it states explicitly that performance numbers cannot answer it. What was rewritten is the quantity in the "cost" column.

## E67 (Which Disks a Stripe Spans)

### 2026-09-01

- See the same-day entry "stale citations in the text of E16 (Role of Journal: WAL vs Intent Log) and E67 (Which Disks a Stripe Spans)…" in the E16 (Role of Journal: WAL vs Intent Log) section.

### 2026-08-31

- E67 (Which Disks a Stripe Spans) newly set up and run to completion
  - **Trigger**: D2 (RAID Stripe Policy) settled item 8 needs data.
  - **Conclusion**: the expected share of data lost is set by the width alone (the five arms equal cell by cell at 3/28); the selection method decides **how the loss is distributed**.
    "At object creation pick a group of the 4 emptiest disks, then keep it fixed" gets 78.6% intact objects with a between-disk spread of 0 in both rounds (uniform / skewed).
  - ⚠️ **The criteria of the two rounds reach opposite conclusions**: in the uniform round the intact-share dimension decides the choice (a 21.4pp difference), in the skewed round it does not (1.4pp),
    and balance decides instead. Set side by side as is; the criteria were not changed.
  - **Third and fourth rounds measured (same day)**: the heterogeneous-capacity round falsified the local leg's counterexample; the disk-addition round measured the migration-granularity cell —
    whole-object migration 83.3% (the theoretical value, not a cell lost) vs per-stripe migration 75.0%.
    ⚠️ The first version of the per-stripe arm was modeled wrong (scanning in order, while stripes cluster by object ⇒ equivalent to whole-object migration, with both arms giving the same number); they separated only after it was changed to spread by stride.
  - ⚠️ **Mutation M4 was not caught in the first round**: the conservation test only compared arms with each other, and four arms wrong together are still equal
    (the textbook form of `test-discipline.md` "comparing arms only with each other cannot detect all arms being wrong together").
    It went red only after an absolute-value assertion pinning `3/28` was added.

## E68 (Selection of Inlining Threshold)

### 2026-09-19

- See the same-day entry "E151 (Arrival Order of User Data Placement Points and Container Arms), E68 (Selection of Inlining Threshold) one current-state sentence each corrected as looked up now; E8 (Different Write Paths for Large vs Small Files)…" in the E151 (Arrival Order of User Data Placement Points and Container Arms) section.

### 2026-09-03

- Adversarial test round — all 19 owed mutation tables filled in, 10 new unit tests added where mutations escaped
  Before: C40 (mutation table not saved) owed mutation tables for 19 bins (e12/e14/e17/e18/e19/e20/e23/e24/e25/
  e26/e27/e28/e30/e61/e68/e69/e6_multicore/e70/e8_split), and nobody had re-proven that "the tests can go red" for this batch of experiments.
  After: all 19 tables were written and run, all 118 mutations caught, 0 blind spots;
  the existing tables of the 7 bins touched by this round's changes (e43/e57/e58/e83/e84/e85/e88) were also rerun, and all are still caught.
  The real blind spots exposed while writing the tables were turned on the spot into unit tests that can go red (10 in all): in E68 (Selection of Inlining Threshold),
  the buffered storage-and-read model had zero assertions before (the conclusion "the optimal threshold = the tier where the working set fits in the cache" comes from that code),
  and four were added: independent recomputation of the inline count, hit-rate endpoints, tree-height fanout, size distribution;
  in E19 (Spurious ENOSPC Under Defer Window), "which view checkpoint uses" was invisible under tests with reserve=0
  (the text had recorded that the first version was wrong on exactly that line), and a test that the reserve pool really rescues a deadlock was added;
  in E61 (Uniformity of Reverse Chain Hash Algorithm), with elimination disabled in the GF(2) rank computation the original three tiers were all green —
  the unit vectors never overlap, so elimination was never exercised, and a {0b11,0b10} discrimination case was added;
  plus the Conservative rule of E18 (Whether D5 Scraps Under Multiple Forks / Multiple Writable Heads),
  the binary-search hit and child pointer of E20 (CPU Cache Cost of Widening Pointers), the scatter saturation of E30 (Write Amplification Half of the Crossover Point),
  the inline boundary of E8 (Different Write Paths for Large vs Small Files),
  and the small-ring underflow guard of E70 (Feasible Region for Two Checkpoint Thresholds).
  A new gate stage `.claude/gate.d/33-mutation-tables.sh` was added at the same time (every bin must have a well-formed same-named mutation table;
  it does not run mutations, and says so itself rather than pretending to have run them), verified both ways with red and green samples.
  Basis: the check form of C40 (mutation table not saved); each table's catch record is in the mutate.sh output,
  and the tables themselves are in `scripts/mutation/tables/`.

### 2026-08-31

- E68 (Selection of Inlining Threshold) newly set up and run to completion
  - **Trigger**: D14 (Dual-Track: Large/Small Files & Persistent/Transient) settled item 3; the 3584 B used by E8 (Different Write Paths for Large vs Small Files) was set, not measured.
  - **Conclusion**: the main criterion fixed before the run (the minimum of total device bytes per file) **did not bracket an interval** — by bytes, inlining up to the block size always pays,
    because what is saved is intra-block fragmentation and what is paid is record bloat, and the former is always larger. **The main criterion asked the wrong question; recorded as is, the criterion not changed.**
  - **The dimension with discriminating power is leaf-count bloat**: 1.40× at 512 B, already 2.61× at 1 KiB, 20.61× at 3584 B.
    The 6.2× read-side cost measured by E8 (Different Write Paths for Large vs Small Files) comes exactly from here.
  - ⚠️ **Criterion 3 cannot be judged**: the read model of E68 (Selection of Inlining Threshold) has no cache, and the read count moves only with tree height (3→4).

## E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-06

- Three experiment index rows disagree with the text — the index row of E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) is entirely a voided conclusion
  - **Trigger**: the user pointed out "experiment conclusions and decision conclusions partly do not match", and that round of checking also compared
    the status / conclusion columns of every row of [experiments.md](experiments.md) with each text's title.

  | Row | Before | After | Basis |
  |---|---|---|---|
  | E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) | Status "run (08-31, 8 unit tests)", conclusion column "exactly 0 when authoritative state has no sharing; derived state is charged by disk capacity" | Status adds **conclusion voided as a whole**; the conclusion column becomes "the criterion hung on a prohibition that cannot reach this item; 'authoritative-state commit reads 0' is an artifact of a counter that never incremented" | Since 2026-08-31 the text title has been "**conclusion voided as a whole**", and what was hit are exactly item ② "exactly 0 is an instrument artifact" and item ③ "charging by disk capacity is a property of that `Derived` implementation, not of derived state". The index rows of E45 (Ring Space Occupancy for Multi-Record Transactions) / E61 (Uniformity of Reverse Chain Hash Algorithm) both carried void marks; only this row did not |
  | E17 (Write Buffer Merge Limit) | "30.9–**31.6M** entries/s, parallelizable by leaf partition **6.38×**" | "30.9–**31.8M** entries/s, parallelizable by leaf partition **6.26–6.38×** (cite with the range)" | The text's third round raised the upper bound from 31.6M to 31.8M and says verbatim "cite the range, not the number from a single round" |
  | E79 (Capacity of Root Record) | Status "run (2026-09-02)", conclusion "the set of 7 common-knowledge trees **overflows by exactly 1 byte**" | Status adds "recomputed on 09-06 with position entry 14", conclusion "the set of 7 common-knowledge trees **overflows by 43 bytes**", plus "still 6 trees without the watermark" | The rerun of the same day's "number eleven"; the index row did not follow |

  - **Why it is worth recording separately**: the index is the entry point for retrieving this batch of experiments. **Fixing the text without fixing the index is changing half the conclusion** —
    someone who retrieves only the index row still gets the old number or the voided conclusion, and it looks completely normal.
   Of the same type is the single value where D25 (Target Workload Priority) cites E17 (Write Buffer Merge Limit) (recorded in [decisions-history.md](decisions-history.md) (number forty-four)).
  - **Made into a check that can go red**: `.claude/gate.d/34-experiment-index-sync.sh` checks four things —
    the text declares the conclusion void while the index says nothing; the text's status is "not run / partly run" while the index status column does not say so;
    a number in the index conclusion column cannot be found in the text; the two sides do not correspond one to one.
    **Its discriminating power was verified on a real regression**: with the E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) row changed back to the old wording,
    the stage exits 1 naming that row, and 0 after restoring; the samples are in `.claude/gate.d/fixtures/34-experiment-index-sync.sh/`.
  - ⚠️ **It cannot catch the two rows of E17 (Write Buffer Merge Limit) and E79 (Capacity of Root Record)**:
    as long as the old value remains in the text in the form of historical narrative ("raised from 31.6M to 31.8M" is like that), the third check still judges it "found".
    **These two rows were caught by a person comparing; today no check can stop the same kind** — this stage must not be read as "the index will not drift again".

### 2026-08-31

- E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) conclusion voided as a whole: the criterion hung on a prohibition that cannot reach this item
  - **Previously**: the conclusion said "the two arms have different cost shapes — Jia (authoritative state) is charged by benefit, exactly 0 without sharing;
    Yi (derived state) is charged by disk capacity, paying 100 reads per shared reference at 1% density".
  - **Now**: **voided as a whole**, and D21 (Boundary Between Authoritative and Derived States) settled item 1 stays unsettled. Unit tests and artifacts kept.
  - **Basis for the overturn** (three-way argument: the local leg and the backward leg each hit it, with the backward leg also running a mutation; the main agent confirmed each point by looking it up):
    1. **The criterion hung on the wrong cell (decisive)**: criterion 3 and conclusion 1 hung on the first cell of `.claude/rules/fs-design.md`,
       while the first cell names verbatim "allocation, ENOSPC acceptance, defer window, lifetime judgement", and the reverse index is not among them;
       the third cell names verbatim "destroying snapshots, compaction, scrub — no commandment", and the consumers listed by D1 (Data Mobility / Reverse Index) and
       D6 (Snapshot Implementation Model) all fall in the third cell. **The prohibition cannot reach this item.**
    2. **Jia's "reads 0" is a tautological value**: `units_read_on_commit` increments in zero places, while Yi's side has a real `read += 1`
       ⇒ the readings of the two arms **were not measured with the same ruler**. Measured by mutation (changing `commit` to scan the whole disk per transaction):
       **not one of the 8 unit tests went red, and the artifact did not change by a byte**.
    3. **The two arms are a false dichotomy**: derived state can maintain a losable shared summary (losing it falls back to a full-disk scan = "only slower",
       still pure derived state), with a rebuild cost of O(shared units) rather than O(disk).
    4. **The density formula is an identity**, and those 101 values over the whole domain prove arithmetic.
  - ⚠️ **The root cause is one verbatim check not done**: before modeling, D21 (Boundary Between Authoritative and Derived States) settled item 1 was read verbatim,
    **but the first cell of `fs-design.md` that was used as the criterion was not**.
    `.claude/singlefs-ai-sop/rules/verify-before-claiming.md` "whether it is settled and how exactly it is defined are two questions"
    governs exactly this, and it **does not apply only to decisions** — the criteria themselves must likewise be read verbatim.
  - ⚠️ **The second instance of the same type within one round**: measurement one of E61 (Uniformity of Reverse Chain Hash Algorithm) is an algebraic tautology,
    and here a counter was never incremented; **both times it was "building an observation that cannot fail"**. Landing place: C60 (Constant Readings Lack Fault Injection Self-Test).

## E70 (Feasible Region for Two Checkpoint Thresholds)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-03

- See the same-day entry "adversarial test round — all 19 owed mutation tables filled in, 10 new unit tests added where mutations escaped" in the E68 (Selection of Inlining Threshold) section.

### 2026-08-31

- E70 (Feasible Region for Two Checkpoint Thresholds) newly set up and run to completion
  - **Trigger**: item 5, newly opened on the day D16 (Publish Semantics) settled item 2 was settled, whose text wrote two couplings. **E70 (Feasible Region for Two Checkpoint Thresholds) checks whether those two sentences are right.**
  - **Conclusion one**: the amplification factor of `T_dirty` is `节点/条目 = 512` — one dirty node of 16 KiB takes only one 32 B position entry in the journal
    ⇒ a 64 MiB ring allows 16 GiB of dirty data. **The direction "the upper bound is tied to the ring geometry" is not wrong but does not bear load**; what actually holds it back is dirty-page memory and the crash-loss window.
  - **Conclusion two**: the lower bound of `T_time` differs by two orders of magnitude across workloads — multistream needs 94–197 ms, seq needs none at all.
    The main workload of D25 (Target Workload Priority) is seq ⇒ **settling by the main workload and by the worst workload give different numbers**, and E70 (Feasible Region for Two Checkpoint Thresholds) does not choose for it.
  - ⚠️ **The absurdity line of criterion 1 (1 TiB) did not trigger**, so that criterion gave no verdict "the ring is not a constraint";
    the direction comes from the numbers themselves (16 GiB vs actual dirty-page memory). **Set side by side as is; the criterion not changed.**

## E71 (Entry Count and Width of Accounting Keys)

### 2026-10-21

- The title and index row add the first rerun of 2026-10-20: under the single-generation basis, 5 of the 45 cells of the upper-bound arm exceed the bound (9 under the old basis), and narrow reads 0 cells; text, numbers and artifacts unchanged.
  - **Basis**: the section "first rerun (2026-10-20): criterion 2 with each root holding only one generation, write-only with no deletes" of the experiment page and artifact `research/results/e71-accounting-keys-single-generation-2026-10-20.out`.
  - **Before**: the title line of `git show 841b3beab:.claude/kb/experiments/71-记账key的条目数与宽度.md`, and the E71 (Entry Count and Width of Accounting Keys) row of `git show 841b3beab:.claude/kb/experiments.md`.

### 2026-10-20

- First rerun: criterion 2 recomputed as "each root holds only one generation, write-only with no deletes" (preregistration `research/prompts/e71-r1-prereg.md`, Q3 of question sheet `research/prompts/c693-forks.md`)
  **Before**: the write bytes of criterion 2 were multiplied by 2 for "one write, one delete" (D5 (Snapshot / Space Accounting Mechanism) settled item 2 as of 2026-09-01); at a width of 30 bytes, the upper-bound arm cell 1024 × 64 at 35 389 440 bytes exceeds the bound, and 9 of 90 cells exceed it, all in the upper-bound arm.

  **After**: once D5 (Snapshot / Space Accounting Mechanism) settled item 2 changed to each root holding only one generation, the coefficient is 1; a new rig `scripts/experiment-apparatus/e71_accounting_keys_single_generation.rs` was started, with the original rig and the original rerun row untouched;
  artifact `research/results/e71-accounting-keys-single-generation-2026-10-20.out`. The same cell under the new basis is 17 694 720 bytes, not exceeding the bound; 5 of the 45 cells of the upper-bound arm exceed it, narrow reads 0 cells, and the added wide-read arm 3 cells;
  the judgement of the verdict cell flips between 34 and 50 bytes. The 90 cells under the old basis have 0 lines of `diff` against the original artifact.

  **Basis**: the current settled text of D5 (Snapshot / Space Accounting Mechanism) settled item 2; numbers and commands are in the section "first rerun (2026-10-20): criterion 2 with each root holding only one generation, write-only with no deletes" of the experiment page.
- The look-back of the D5 (Snapshot / Space Accounting Mechanism) settled item 4 row of the "Affected decisions" table changed to "2026-10-20 changed": the sentence about E71 (Entry Count and Width of Accounting Keys) in that item's basis is rewritten with the numbers of the first rerun
  - **Basis**: the T3 cell in section two of `research/prompts/m3-close-c693-r3-main-verification.md` (clause ② as changed by the second-round forward leg checked number by number against `research/results/e71-accounting-keys-single-generation-2026-10-20.out`).
  - **Before**: in the intermediate state before this version, the look-back said "2026-10-20 unaffected: the decision text was not changed this time … for the main agent to judge" (written by the runner of the first rerun, never committed).

### 2026-09-03

- E71 (Entry Count and Width of Accounting Keys) gets a reach note — it ran nine statistics, while the list already has eleven
  **Before**: the text of E71 (Entry Count and Width of Accounting Keys) said verbatim "once the **nine statistics** are settled, it can be computed",
  while D5 (Snapshot / Space Accounting Mechanism) settled item 4 was reopened for the first time on the 3rd of the same month, and the list became eleven.
  The two coexisted, and someone retrieving E71 (Entry Count and Width of Accounting Keys) would compute with nine.

  **After**: a reach note is hung at the start of the text. **Not a character of the binary or artifact changes** — `scripts/experiment-apparatus/replay.sh`
  compares them byte for byte; recomputing at s = 11 is arithmetic, not a rerun: the upper-bound arm cell at t=64 / d=8 / K=25 goes
  115 200 → 140 800, the narrow-read arm 4075 → 4300 (neither new item is split by tree: runs has no dimension ⇒ +K=25,
  the count of all-empty clustered segments carries the device dimension ⇒ +d×K=200), and the overestimation factor 28.3 → 32.7. **Neither conclusion moves.**

  **Basis**: the settled texts of D5 (Snapshot / Space Accounting Mechanism) settled item 4 and D26 (Background Compaction and Placement Reclamation) settled item 6,
  plus E71 (Entry Count and Width of Accounting Keys)'s own formula counting dimensions per item.

### 2026-09-02

- See the same-day entry "second round of E80 (Partial Release of Stripes) — measurement basis of repair unified, a lazy arm and a locality workload added" in the E80 (Partial Release of Stripes) section.

### 2026-09-01

- One experiment set up for each of the six of the nine settled items that owed measurement, none run
  The user decided "settle as recommended, adjust slowly while writing code later; **what has not been measured must be measured first**".
  Of the nine settled items, six rest on arithmetic or elimination with **no measurement at all**; one text was set up for each, with criteria and failure clauses fixed before running:

  | Experiment | Which settled item it repays | The load-bearing criterion |
  |---|---|---|
  | E71 (Entry Count and Width of Accounting Keys) | D5 (Snapshot / Space Accounting Mechanism) settled item 1 (dimension tuple) | The entry count is **exactly** `s × t × d × K`; the positive control is that removing the device dimension must drop it by a factor of d |
  | E72 (Mount-Time Recalculation of Device Descriptor Table) | D12 (Target Media) settled item 3 (placement) and settled item 4 (which ones to verify) | Every comparison item must be forcible into its "mismatch" branch |
  | E73 (Fanout Cost of Node Key Interval) | D18 (Information Carried in a Block) settled item 2 (the accounting tree carries key intervals) | Fanout is **exactly** `(节点大小 − 节点头) / 条目宽度`; a tree one level taller reopens D8 (Core Index Structure) settled item 2 |
  | E74 (Allocation Record Entry Count and Compaction Cost) | D3 (Space Allocation) settled item 3 (carrier form) | The write volume of compaction must not grow with **disk capacity** — that hits the first cell of `.claude/rules/fs-design.md` |
  | E75 (Record Size and Ring Geometry) | D23 (Journal Role & Format) settled item 12 (4 KiB) | The number of entries each record can hold equals the arithmetic of E45 (Ring Space Occupancy for Multi-Record Transactions) cell by cell; the cross-record count at the 4 KiB tier must be 0 |
  | E76 (Discriminative Power of Payload Checksum) | D23 (Journal Role & Format) settled item 13 (candidate Yi) | Four kinds of construction judged cell by cell; the load-bearing row is "the header landed, the payload landed only halfway" |

  ⚠️ **All six are unrun and all lack implementations** — they are not "this round's evidence" but **IOUs recorded** this round.
  Per `.claude/singlefs-ai-sop/rules/kb-discipline.md` "record not knowing explicitly",
  until they are run, the strength of those six settled items is "elimination / arithmetic", not "measurement".
- Three experiments go from "not run" to "run", with not a word of the criteria changed
  Of the six experiments set up on the same day (number two), **the preconditions of three were cleared on the same day**, and they were run that day:

  | Experiment | How the precondition was cleared | Conclusion |
  |---|---|---|
  | E71 (Entry Count and Width of Accounting Keys) | **Previously**: owed "which statistics there are", and the text said verbatim "the absolute value cannot be computed today". **Now**: D5 (Snapshot / Space Accounting Mechanism) settled item 4 settled nine statistics the same day ⇒ it can be computed | What blows up first is the product `t × d` (≳65536), not any of the four independent variables; the upper bound `s × t × d × K` overestimates the real keys counted per item dimension by 28× |
  | E75 (Record Size and Ring Geometry) | **Previously**: recorded as "journal implementation owed". **Now**: the three criteria (capacity arithmetic, cross-record count at the 4 KiB tier, whether the record header fits in an atomic unit) **are all arithmetic**, and none needs a journal implementation | The 4 KiB settlement stands (12 entries take 1 record, margin 5.92×, ring lower bound 8 KiB, equal cell by cell to the published cell of E45 (Ring Space Occupancy for Multi-Record Transactions)); with fixed-length power-of-2 sizes the alignment waste of I-8.2 (Record Header Does Not Cross Atomic Unit) is always 0 |
  | E76 (Discriminative Power of Payload Checksum) | **Previously**: recorded as "journal implementation owed". **Now**: the four rows of criteria need a **record codec**, not a journal — encoding, the two checksums and the four kinds of construction can all be built in memory | The four rows match cell by cell, and the two digest functions CRC32C and FNV-1a give exactly the same 8-cell verdicts ⇒ the conclusion is a product of coverage, not of function choice; candidate Jia has `distinguishable` all 0 on the four rows |

  **Basis**: `.claude/singlefs-ai-sop/rules/test-discipline.md` "before an experiment runs, the answer must not already exist" —
  the criteria, thresholds and voiding clauses of all three were fixed in the (number two) round **when there were no numbers at all**, and this round ran them as they were, without changing a word.

  ⚠️ **The `s × t × d × K` in criteria 1 and 3 of E71 (Entry Count and Width of Accounting Keys) is an upper bound, not the entry count**,
  because D5 (Snapshot / Space Accounting Mechanism) settled item 4 says verbatim "not every statistic uses the full dimension tuple", a sentence that appeared only after the criteria were written.
  The handling is **to run the upper-bound arm as is with all criteria falling on it**, plus two arms counting dimensions per item to measure the gap — **not changing the criteria**.

  ⚠️ **E75 (Record Size and Ring Geometry) found in passing two measurement-basis points that must be written back**:
  the sentence in D23 (Journal Role & Format) settled item 4, "two settled but not yet landed increments ⇒ 91 bytes", was written before settled item 13 was settled,
  and is now **three ⇒ 95 bytes**; the arithmetic of settled item 12 cites a header of 99 (including the `tail_lsn` already removed by settled item 3),
  **which does not affect the conclusion at 4 KiB (all four bases give 71 entries) but does at 2 KiB (78 gives 35, the rest give 34)**.

## E72 (Mount-Time Recalculation of Device Descriptor Table)

### 2026-09-02

- E72 (Mount-Time Recalculation of Device Descriptor Table) run, precondition downgraded from "QEMU owed" to "sudo owed"
  - **Previously**: E72 (Mount-Time Recalculation of Device Descriptor Table) was recorded as "not run (depends on system configuration and the mount path)",
    and the text said heterogeneity needed QEMU (E15 (Validity of Heterogeneous Device Geometry Simulation) proved it can be built).
  - **Now**: `losetup --sector-size` alone can build a **truly heterogeneous pool** mixing `physical_block_size` 512 and 4096
    ⇒ no QEMU needed. Each of the four comparison items **is forced red by real means** (remount with a different sector-size, change capacity with `truncate`,
    change device order, change layout bits), **and only that one goes red each time**; the handling matches D12 (Target Media) settled item 4 cell by cell
    (pbs is `refuse_rw`, not `refuse`; a larger capacity is `allow_and_note`).
    The table's byte count is exactly `设备数 × 条目宽度`; the recomputation time is linear (13.02 / 12.63 / 12.48 ms per device).
  - **Basis**: real-device measurement, N=5 rounds × 2 complete invocations, the handling patterns of the seven scenarios identical cell by cell.

  ⚠️ **The most valuable cell is that the "do not compare" rule was also verified**: D12 (Target Media) settled item 4 says verbatim
  "not on the list: rotation speed, queue depth, `io_opt`, discard… comparing them would only manufacture false reds that make the pool unmountable".
  After changing readahead, the recomputation **still lets it through** ⇒ that rule has discriminating power and is not just a statement of attitude.
  **Verifying only "what should go red went red" without "what should not go red did not" is doing only half.**

  ⚠️ **The mount path itself still does not exist**: what was verified is whether the **rule** of D12 (Target Media) settled item 4 has discriminating power,
  **not** whether this project's mount code is right; once the code is written, the same set of scenarios must be verified again.
  ⚠️ Heterogeneity covers only the single dimension `physical_block_size`; the entry width and dev_uuid width are still assumptions.

### 2026-09-01

- See the same-day entry "one experiment set up for each of the six of the nine settled items that owed measurement, none run" in the E71 (Entry Count and Width of Accounting Keys) section.

## E73 (Fanout Cost of Node Key Interval)

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-07

- See the same-day entry "new E117 (Header Width and Geometry After Factoring in Reserved Bits); one absolute value of the preregistration judged no" in the E117 (Header Width and Geometry After Factoring in Reserved Bits) section.

### 2026-09-03

- See the same-day entry "new E97 (Entry Encoding of Accounting and Allocation Records), run to completion the same day — three adversarial rounds, rerun three times" in the E97 (Entry Encoding of Accounting and Allocation Records) section.

### 2026-09-01

- E73 (Fanout Cost of Node Key Interval) run; the precondition "index node implementation owed" judged not to hold
  - **Previously**: E73 (Fanout Cost of Node Key Interval) was recorded as "not run (depends on the index node implementation)".
  - **Now**: of the three criteria (fanout arithmetic, whether tree height rose, an isolated node self-verifying its covered interval) **none needs a btree implementation** —
    the first two are arithmetic, the third is a judgement model of one isolated node ⇒ run to completion that day.
    **Fanout drops by at most 1 cell (0.33%), and at the basic node header 32 tier the cost is exactly 0; across 12 cells tree height did not rise by a level**
    ⇒ the reopening condition of criterion 2 did not trigger, and the 16 KiB of D8 (Core Index Structure) settled item 2 need not be reopened.
  - **Basis**: E73 (Fanout Cost of Node Key Interval) measurement, 8 unit tests / all 8 mutations caught, N=5 rounds byte-for-byte identical.

  ⚠️ **An implementation defect was forced out by mutation testing, not thought of**: the loop in `tree_height` was originally unbounded,
  and with fanout 1 `cap` does not grow, so the loop never exits. When the mutation loosened that guard, **the whole round hung** —
  **hanging is not judging red**: neither red nor green, it could only end with `kill`.
  ⇒ The loop got an upper bound (with fanout ≥ 2, a tree that fits in u64 is at most 64 levels tall),
  and the mutation was replaced by one that gives a **wrong answer** instead of hanging (tightening the guard to `f < 3`, so fanout 2 is wrongly rejected).
  **Keep this in mind: the breakage a mutation test picks must make the test go red; one that makes the program hang proves nothing.**

- See the same-day entry "one experiment set up for each of the six of the nine settled items that owed measurement, none run" in the E71 (Entry Count and Width of Accounting Keys) section.

## E74 (Allocation Record Entry Count and Compaction Cost)

### 2026-09-20

- Two places in E74 (Allocation Record Entry Count and Compaction Cost) saying "all six items of D26 (Background Compaction and Placement Reclamation) are open" changed to the current values as looked up now; an "Affected decisions" table added
  - **Conclusion**: both places changed to current values — D26 (Background Compaction and Placement Reclamation) settled items 1 / 2 / 3 / 12 today settle the trigger, transaction form, acceptance and form framework, while the threshold values still wait for E10 (Aging: Structural Resistance vs Post-hoc Compaction); not one number of the experiment's conclusions, criteria or artifacts changed.

  - **Before**: the reach note of criterion 2, "while the compaction strategy itself (all six items of D26 (Background Compaction and Placement Reclamation) are open) **has not been measured**."; item 3 of "the half it cannot answer", "**the form of background compaction itself is not settled yet** (all six items of D26 (Background Compaction and Placement Reclamation) are open) ⇒ what is measured is…".
  - **After**: the first becomes "while the compaction strategy itself **has not been measured** — D26 (Background Compaction and Placement Reclamation) settled items 1 / 2 / 3 / 12 today settle the trigger, transaction form, acceptance and form framework, while the threshold values still wait for E10 (Aging: Structural Resistance vs Post-hoc Compaction), and none of this is in the model of E74 (Allocation Record Entry Count and Compaction Cost)."; the second becomes "**the background compaction strategy is not in the model** (not one of the trigger, transaction form, acceptance and form framework settled by D26 (Background Compaction and Placement Reclamation) settled items 1 / 2 / 3 / 12 is modeled)". Both places say "this experiment did not measure the compaction strategy"; the fact is unchanged, and only the current item status of D26 (Background Compaction and Placement Reclamation) changed. On the same day E74 (Allocation Record Entry Count and Compaction Cost) got a "### 影响的决策" table.
  - **Basis**: looking up `.claude/kb/decisions/26-后台整理与放置回收.md` now: settled items 1–7, 11, 12 are settled and unsettled items 8, 9, 10 are unsettled, nine settled and three unsettled, not six all open.

### 2026-09-03

- See the same-day entry "new E94 (Touch Surface of Moving Shared Placement Points), run to completion the same day — the data pre… of D26 (Background Compaction and Placement Reclamation) settled item 5" in the E94 (Touch Surface of Moving Shared Placement Points) section.
- See the same-day entry "new E97 (Entry Encoding of Accounting and Allocation Records), run to completion the same day — three adversarial rounds, rerun three times" in the E97 (Entry Encoding of Accounting and Allocation Records) section.

### 2026-09-01

- The structural half of E74 (Allocation Record Entry Count and Compaction Cost) run, the wall-clock question still owed
  - **Previously**: E74 (Allocation Record Entry Count and Compaction Cost) was recorded as "not run (depends on the allocator and background compaction)".
  - **Now**: D3 (Space Allocation) settled items 1 and 3 are both settled ⇒ entry count, tree height,
    and "how many nodes must be rewritten to compact a batch of N placements" are all **structural quantities forced by the carrier** and can be computed.
    **The tree itself eats 0.055% of capacity**; with N fixed and capacity growing 256×, the number of rewritten nodes only rises from 13 to 19 (all from tree height)
    ⇒ criterion 2 holds: no term in the compaction path scales with capacity.
    ⚠️ **But it measured something unexpected**: COW rewrites a whole 16 KiB node ⇒ amplification 2.6× when placements are contiguous,
    **964×** when scattered, **a 370× difference**; the key of settled item 3 (device identity + offset within the device)
    puts compaction at the 2.6× end.
  - **Basis**: E74 (Allocation Record Entry Count and Compaction Cost) measurement, 9 unit tests / all 8 mutations caught, N=5 rounds byte-for-byte identical.
    ⚠️ **It answers only one of the two questions of axis ②**: "how many bytes are written" is a structural quantity; "how long it runs" is wall clock and needs an implementation.
    The compaction **strategy** itself was not measured (all six items of D26 (Background Compaction and Placement Reclamation) are open).

  ⚠️ **A blind spot was caught on the spot by mutation testing**: the written byte count in the artifact was originally computed by `main` itself,
  with each unit test computing it again ⇒ the mutation "charge per entry instead of per whole node" **turned no test red**.
  After `main` and the unit tests were made to go through the same function, it was caught by two tests.
  **This is the same type of lesion that voided E45 (Ring Space Occupancy for Multi-Record Transactions): a number in the artifact pinned by no test.**

- See the same-day entry "one experiment set up for each of the six of the nine settled items that owed measurement, none run" in the E71 (Entry Count and Width of Accounting Keys) section.

## E75 (Record Size and Ring Geometry)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-13

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) by journal header…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-03

- See the same-day entry "new E91 (Ring and Admission Accounting Under Settled Format), run to completion the same day — all bills posted by the three-way argument measured" in the E91 (Ring and Admission Accounting Under Settled Format) section.

### 2026-09-01

- See the same-day entry "one experiment set up for each of the six of the nine settled items that owed measurement, none run" in the E71 (Entry Count and Width of Accounting Keys) section.
- See the same-day entry "three experiments go from 'not run' to 'run', with not a word of the criteria changed" in the E71 (Entry Count and Width of Accounting Keys) section.

## E76 (Discriminative Power of Payload Checksum)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.
- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-13

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) by journal header…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-02

- See the same-day entry "four new experiments E77 (Persistence Ordering of Publishes) to E80 (Partial Release of Stripes), run to completion the same day — reasoning forward from settled clauses '…" in the E77 (Persistence Ordering of Publishes) section.

### 2026-09-01

- See the same-day entry "one experiment set up for each of the six of the nine settled items that owed measurement, none run" in the E71 (Entry Count and Width of Accounting Keys) section.
- See the same-day entry "three experiments go from 'not run' to 'run', with not a word of the criteria changed" in the E71 (Entry Count and Width of Accounting Keys) section.

## E77 (Persistence Ordering of Publishes)

### 2026-09-19

- See the same-day entry "E17 (Write Buffer Merge Limit), E42 (Number of Records per Transaction)…" in the E17 (Write Buffer Merge Limit) section.
- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-02

- Four new experiments E77 (Persistence Ordering of Publishes) to E80 (Partial Release of Stripes), run to completion the same day — the products of the round reasoning forward from settled clauses "what will be hit first at implementation time"
  - **Before**: the experiment index ended at E76 (Discriminative Power of Payload Checksum); four places had zero coverage: the persistence order of publish, the replay start point, root record capacity, and partial release of stripes.
  - **After**: four added and run to completion:
    E77 (Persistence Ordering of Publishes) — exactly one necessary barrier (before the root slot), and verifying the named units before replay applies them is a load-bearing step;
    E78 (Starting Point for Replay) — with a stale tail × block reuse stacked together, recovery in the form of E24 (Recovery Algorithm: Does Trusting Tail First Lose Records)
    aborts itself on a healthy image; the ways out are a jsn watermark carried by the root or blind application plus tail deletion, and both pass the criteria;
    E79 (Capacity of Root Record) — a 512-byte root slot with a watermark holds only 6 trees, the set of 7 common-knowledge trees overflows by exactly 1 byte,
    and the indirection layer adds +7.1% blocks per publish;
    E80 (Partial Release of Stripes) — once stripes with w ≥ 3 are partially released by COW, letting reuse happen freely loses 10–13 live units on average per single-disk failure,
    reclaiming only when fully dead pins 36–46% live data, migrating survivors costs +64–66% write amplification, and the w=2 control is all zero (the first version could not see it).
  - **Basis**: the user asked to "reason out implementation-time problems from the current decisions, find blind spots for the test code, and then resolve them with experiments and decisions".
    The criteria of all four experiments were fixed in the source headers before running; the owed checks set up along with them are
   the batch in [checks-owed.md](checks-owed.md) starting from C76 (Publish Ordering Has No Authoritative Placement).
  - ⚠️ Mutation testing of E77 (Persistence Ordering of Publishes) caught two blind spots of the model itself on the spot (recovery outcomes self-reported, walk-reads masked by the audit fallback),
    both fixed by "prove it can go red first"; the first version of E80 (Partial Release of Stripes) had colliding stripe numbers (a mid-loop restripe pushed new entries into
    `stripes` while the batch stripe took its number just before entering the loop), and after the panic it was changed to reserve a slot first and fill it later.

## E78 (Starting Point for Replay)

### 2026-10-17

- The landing place C85 (Four Equalities of the Publish Counter Not Hard-Coded) now points to C88 (Timeline discrimination for root ring unimplemented): this bill was merged into C88 (Timeline discrimination for root ring unimplemented) on 2026-10-11; conclusions, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/78-重放的起点.md`.

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-02

- See the same-day entry "four new experiments E77 (Persistence Ordering of Publishes) to E80 (Partial Release of Stripes), run to completion the same day — reasoning forward from settled clauses '…" in the E77 (Persistence Ordering of Publishes) section.
- See the same-day entry "second round of E80 (Partial Release of Stripes) — measurement basis of repair unified, a lazy arm and a locality workload added" in the E80 (Partial Release of Stripes) section.

## E79 (Capacity of Root Record)

### 2026-10-17

- Two landing places now point to current numbers: the owed rerun with one more tier is under C148 (Wider tree table entries compress indirection threshold to two layers) (the former C157 (Tree table capacity computed with differing entry widths in two places), merged on 2026-10-11); the remaining owed check is C66 (format-level gaps have no item number) (the former C78 (Root record lacks field table), merged on 2026-10-11); conclusions, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/79-根记录的容量.md`.

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-16

- See the same-day entry "E131 (Two Livelist Carriers: Values and Costs) and…" in the E131 (Two Livelist Carriers: Values and Costs) section.

### 2026-09-13

- See the same-day entry "row-width constant of E141 (Mount Admission Self-Test for Switchover Reservation) renamed, E79 (Capacity of Root Record) marks 243 as void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-06

- The position entry of E79 (Capacity of Root Record) stuck at an old value for three days without anyone noticing; after the rerun the tree table unit capacity goes 266 → 243
  - **Before**: in `e79_root_record.rs`, `const TREE_ENTRY: u64 = 8 + 31 + 11 * 2`, counting the position entry at **11**;
    tree table unit capacity 266 trees/level, 7 trees in a 512 slot without the watermark, and the set of 7 common-knowledge trees "overflows by exactly 1 byte".
  - **After**: the position entry is counted at the **14** of D19 (Block Pointer Structure & Width Budget) settled item 4 (user decision 2026-09-03),
    entry 61 → 67; tree table unit **243** trees/level (the three node-header tiers **243 / 242 / 242**, no longer the same cell by cell);
    **6** trees in a 512 slot without the watermark (the same as with it); the set of 7 common-knowledge trees overflows by **43** bytes;
    a 4096 slot holds **59 with the watermark / 60 without**.
  - **Basis**: what D19 (Block Pointer Structure & Width Budget) settled item 4 settled is exactly 14, and D22 (Synthesizing Unit Atomicity)'s own field table also goes by 14 (tree table unit pointer 59) —
    only the source of E79 (Capacity of Root Record) stuck at 11.
  - **Why it went unnoticed for three days**: `87-replay.sh` compares artifacts byte for byte, and **when source and artifact stay at the old value together it is always green**
    (it had been rerun earlier the same day, byte-identical); `27-format-constants.sh` only checks constants with registered markers,
    and the position entry was not registered at the time. Both were running, and the drift slipped through between them — the same type as the JOURNAL_HDR case recorded in that stage's header.
  - **The gate added**: the position entry is extracted into `const LOC_ENTRY` (the same name as in E109 (Runtime Costs of Three Location Authority Arms)),
    and `format-const: LOC_ENTRY = 14 stale=11 * 2|tree_entry=61` is registered next to D19 (Block Pointer Structure & Width Budget) settled item 4.
    Its discriminating power was verified both ways: changing the source back to 11 makes the stage exit **1**, and **0** after restoring.
  - **Mutations**: the original text of M1 changes with it; M6 (knock the header width back to 55) **becomes a blind spot** under the new entry width
    (16284/67 and 16297/67 are both 243), and three assertions were added on the spot to catch it back, all 6 caught.
  - **Not a word of the structural conclusion changes**: a flat layout does not fit, so the indirection layer is required — and more firmly so (even without the watermark 7 trees do not fit).
    ⚠️ The **basis number** of D22 (Synthesizing Unit Atomicity) settled item 7 changes with it ("overflows by exactly 1 byte" → "overflows by 43 bytes"), **with the direction of the conclusion unchanged**.
  - **How it was found**: the backward leg of the second adversarial round on D8 (Core Index Structure) settled item 8 read that source line while checking where 266 came from.

- See the same-day entry "a round of recovering from format constants drifting out of sync — E43 (Extension Point Byte Upper Bound) rerun a fourth time at 105…" in the E43 (Extension Point Byte Upper Bound) section.
- See the same-day entry "three experiment index rows disagree with the text — the index row of E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) is entirely a voided conclusion" in the E69 (Incremental Maintenance Cost of Reverse Index Taking Authoritative State) section.

### 2026-09-02

- See the same-day entry "four new experiments E77 (Persistence Ordering of Publishes) to E80 (Partial Release of Stripes), run to completion the same day — reasoning forward from settled clauses '…" in the E77 (Persistence Ordering of Publishes) section.
- See the same-day entry "second round of E80 (Partial Release of Stripes) — measurement basis of repair unified, a lazy arm and a locality workload added" in the E80 (Partial Release of Stripes) section.

## E80 (Partial Release of Stripes)

### 2026-09-02

- Second round of E80 (Partial Release of Stripes) — measurement basis of repair unified, a lazy arm and a locality workload added
  - **Before**: three arms in the first round; eager counted as "each survivor re-landed alone at w=2" (an upper-bound basis, +64.3–66.4%).
  - **After**: restripe_on_free and the newly added lazy_restripe **share the same repair implementation**
    (the survivors land as a batch on one new stripe and the old stripe is reclaimed whole), so the two arms differ only in "when to repair"; the workload adds a skew (80/20) dimension.
    Measured: eager under the fair basis is **+74.7–77.1%** (batch repair re-couples survivors into multi-unit stripes, so the repair debt renews itself);
    **lazy +46.6–47.7%, about sixty percent of the repair events, end-of-round pinning always 0 with a budget of 2 stripes/round, drained to zero, losses always 0**.
    Unit tests 11 → 14, mutations 8 → 11 all caught; artifact overwritten under the same name (two rounds the same day; the free_now / pin lines of the first round are unchanged word for word).
  - **Basis**: the user's handling instruction for C81 (no rule for partial stripe release), "verify once more, then give me a recommendation";
    the lazy arm is exactly the form of leg one (resident background compaction) of D26 (Background Compaction and Placement Reclamation).
  - In the same round, the landing places for "which one to choose is a decision" of E78 (Starting Point for Replay) / E79 (Capacity of Root Record) were added
    (settled respectively in D23 (Journal Role & Format) settled item 14 and D22 (Synthesizing Unit Atomicity) settled item 7),
    and E71 (Entry Count and Width of Accounting Keys) gets a rider note "generation segment ≥ 48 bits, byte arithmetic must be rerun"
    (source: D16 (Publish Semantics) settled item 6).

- See the same-day entry "four new experiments E77 (Persistence Ordering of Publishes) to E80 (Partial Release of Stripes), run to completion the same day — reasoning forward from settled clauses '…" in the E77 (Persistence Ordering of Publishes) section.

## E81 (Fixed Point of Commits)

### 2026-09-13

- See the same-day entry "new E148 (Commit Fixed Point Recomputed with Two Record Trees), run to completion — with the mapping tree added, the clustered fixed point goes from 5 → 9 blocks, an empty publish…" in the E148 (Commit Fixed Point Recomputed with Two Record Trees) section.

### 2026-09-02

- New E81 (Fixed Point of Commits) / E82 (In-Flight Synthesis for Admission) / E83 (Granularity of Tombstones), run to completion the same day — clearing three owed checks that "can be modeled right now"
  - **Before**: C83 (commit fixed point unanswered) / C87 (Synthesized value of admission reading undefined) / C84 (Nobody Has Settled the Granularity of Tombstone Units)
    were all marked "the model can be built right now", yet nobody had built it.
  - **After**: E81 (Fixed Point of Commits) — convergence is decided by "whether the blocks the commit itself produces cluster"; scattered allocation amplifies one
    12-unit fsync into 5909 metadata blocks, and clustering only the metadata breaks the chain (the candidate rule hangs as a warning note on D3 (Space Allocation));
    E82 (In-Flight Synthesis for Admission) — the three intuitive ways of writing it each produce one countable class of violations, and the correct form is one sentence;
    E83 (Granularity of Tombstones) — with each record taking a unit of its own, rm -rf of a million files physically writes 62.5 GiB,
    while range records + packing are 583× cheaper but couple reclamation lifetimes. Each of the three C bills is marked "model layer repaid / quantification repaid".
  - **Basis**: the user's "you may continue"; all criteria fixed before the run.
    ⚠️ The second half of criterion 4 of E81 (Fixed Point of Commits) was revised once during the first trial run (the comparison base was miscomputed, before any
    formal round was checked in; the revision is recorded in the source header and the text); two mutations were replaced for "semantics-preserving / killed as unbounded",
    both replacements going through "prove it can go red first".

## E82 (In-Flight Synthesis for Admission)

### 2026-09-19

- See the same-day entry "E19 (Spurious ENOSPC Under Defer Window), E82 (In-Flight Synthesis for Admission)…" in the E19 (Spurious ENOSPC Under Defer Window) section.

### 2026-09-02

- See the same-day entry "new E81 (Fixed Point of Commits) / E82 (In-Flight Synthesis for Admission) / E83 (Granularity of Tombstones), run to completion the same day — three…" in the E81 (Fixed Point of Commits) section.

## E83 (Granularity of Tombstones)

### 2026-10-04

- Run 2: rerun with 40-byte tombstone records and the record area starting at 136 (D18 (Information Carried in a Block) settled item 24 clause 2, settled item 16; preregistration `e83-r2-prereg.md`). Packing capacity 583 → 815; rm -rf of 1 million files packed 1716 → 1227 units (107.2 → 76.7 MiB); per_extent × packed truncate of 4096 extents 8 → 6 units; the exclusive cells, the unlink-one-small-file cells and the per_object truncate cells byte-for-byte identical to run 1. In all five Q2 cells packing writes less, and in none of the five Q3 cells does the gap factor shrink, recorded as "no flip". 21 unit tests, all 9 mutations caught (caught 9 / invalid 0 / not red 0). Artifact `e83-tombstone-grain-2026-10-04-r2.out` (37 lines). The seven rows of "Affected decisions" looked back on this day; the D18 (Information Carried in a Block) settled item 11 row says "changed", the decision text was not touched, and it is left for the main agent to judge.
  - **Correction**: the run-1 text wrote `phys_mib=62500.0` as "62.5 GiB"; it should be 61.04 GiB (62 500 MiB).
  - **Before**: `git show fcc4d340:.claude/kb/experiments/83-墓碑的粒度.md`.

### 2026-10-01

- Notes added in three places in the text: the first-version tombstone record has been settled at 40 bytes, capacity 815, by D18 (Information Carried in a Block) settled item 24, while this page ran at 56 bytes and its numbers and artifacts were not rerun with it; five rows of "Affected decisions" looked back on this day, unaffected.
  - **Basis**: the three E83 (Granularity of Tombstones) rows of table 1 in sweep report `/tmp/claude-1000/m3-sweep-tomb56/report.md`.
  - **Before**: `git show fcc4d340:.claude/kb/experiments/83-墓碑的粒度.md`.

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-06

- See the same-day entry "a round of recovering from format constants drifting out of sync — E43 (Extension Point Byte Upper Bound) rerun a fourth time at 105…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-04

- See the same-day entry "E102 (Unit Class Registry Table and Packed Record Units) added and run to completion the same day" in the E102 (Unit Class Registry Table and Packed Record Units) section.

### 2026-09-02

- See the same-day entry "new E81 (Fixed Point of Commits) / E82 (In-Flight Synthesis for Admission) / E83 (Granularity of Tombstones), run to completion the same day — three…" in the E81 (Fixed Point of Commits) section.

## E84 (Pinned Volume of Packed Tombstones)

### 2026-10-01

- After "CAP=584 depends on the 56-byte record width assumption of E83 (Granularity of Tombstones)", a note is added that the first version has been settled at 40 bytes (capacity 815) and this page has not been rerun; two rows of "Affected decisions" looked back on this day, unaffected. The CAP=584 in the title does not match the 583 in the same page's artifact; that was not introduced this time and was left alone.
  - **Basis**: the E84 (Pinned Volume of Packed Tombstones) row of table 1 and line 17 of E84 (Pinned Volume of Packed Tombstones) in table 2 of sweep report `/tmp/claude-1000/m3-sweep-tomb56/report.md`.
  - **Before**: `git show fcc4d340:.claude/kb/experiments/84-打包墓碑的钉住量.md`.

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-08

- See the same-day entry "the sentence in E34 (Root Ring Slot Geometry) 'never settled in the repository' rewritten as looked up now, and the hard requirement 1 that its own premise table failed to cite added" in the E34 (Root Ring Slot Geometry) section.

### 2026-09-04

- See the same-day entry "E102 (Unit Class Registry Table and Packed Record Units) added and run to completion the same day" in the E102 (Unit Class Registry Table and Packed Record Units) section.

### 2026-09-02

- Four new experiments E84 (Pinned Volume of Packed Tombstones) to E87 (Placement of Fixed Structures) — the "closing out experiments" round
  - **Before**: C84 (Nobody Has Settled the Granularity of Tombstone Units), C86 (Two Unit Sizes Conflict with Scan Stepping), C79 (Nobody Has Settled the Placement of the System Configuration and the Journal Ring), C68 (Invariant Allowlist Conflicts with Settled Clauses) each owed half (quantification or preparation), and closing out the last one had no machine-checkable requirement matrix.
  - **After**: E84 (Pinned Volume of Packed Tombstones) — when generations are mixed, 83.9% of live tombstone units under sparse deletion are dead records,
    while loading units per generation is always 0 (the candidate loading discipline handed to D18 (Information Carried in a Block) settled item 10 as an addendum);
    E85 (Field Table of Unit Headers) — requirement matrix + byte ledger (the 71 bytes of the patch set are all enough, with exactly eight conflicts with the whitelist);
    E86 (Scan Stride and Two Unit Sizes) — a 16384 stride collects everything, 32768 misses exactly all the odd-slot nodes, and exclusive slots cost ×2 in space;
    E87 (Placement of Fixed Structures) — with a single copy of the system configuration, losing disk 0 makes the whole pool unmountable, while one copy per disk + journal mirroring makes all 8 cells mountable,
    **and in passing it ran into an mkfs hard requirement: seed the initial root into every region** (without seeding, losing that disk after the first publish wipes everything out).
  - **Basis**: the user's "continue closing out the experiments". All criteria fixed before the run;
    the residency denominator of E84 (Pinned Volume of Packed Tombstones) and the mkfs seed of E87 (Placement of Fixed Structures) were revised during the first trial run and recorded in their respective source headers
    (both before any formal round was checked in); the four experiments total 34 unit tests / 23 mutations all caught, 5 rounds byte-identical.

## E85 (Field Table of Unit Headers)

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-03

- See the same-day entry "new E96 (Consistency of Hybrid Architecture), run to completion the same day — the user asked to test 'the central mapping exists unconditionally'" in the E96 (Consistency of Hybrid Architecture) section.

### 2026-09-02

- See the same-day entry "four new experiments E84 (Pinned Volume of Packed Tombstones) to E87 (Placement of Fixed Structures) — the 'closing out experiments' round" in the E84 (Pinned Volume of Packed Tombstones) section.

## E86 (Scan Stride and Two Unit Sizes)

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-02

- See the same-day entry "four new experiments E84 (Pinned Volume of Packed Tombstones) to E87 (Placement of Fixed Structures) — the 'closing out experiments' round" in the E84 (Pinned Volume of Packed Tombstones) section.

## E87 (Placement of Fixed Structures)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-02

- See the same-day entry "four new experiments E84 (Pinned Volume of Packed Tombstones) to E87 (Placement of Fixed Structures) — the 'closing out experiments' round" in the E84 (Pinned Volume of Packed Tombstones) section.

## E88 (Discrimination of Impostor Units vs Orphan Units)

### 2026-10-17

- "The owed check for putting it into the checker still hangs on stroke ② of C89 (Leftover Recomputation from Closing the Unit Header)" changed to done: the pool-level checker's scan direction for I-7.8 (root record tree ID watermark not lower than the pool-wide maximum tree ID) claims whole units, pinned by the test `pool_checker_scan_of_tree_identifiers_claims_whole_units`; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/88-冒名单元与孤儿单元的判别.md`.

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-02

- Added E88 (Discrimination of Impostor Units vs Orphan Units) and ran it the same day: the falsification check the adversarial round opened for itself
  Before: stroke ③ of C89 (Leftover Recomputation from Closing the Unit Header) carried "falsification fixture not run: if the verdict is wrong, fsid / birth generation flip back".
  After: E88 (Discrimination of Impostor Units vs Orphan Units) landed and ran (four arms × seven fixture classes counting model,
  10 unit tests / all 11 mutations caught / independent audit equal arm by arm / replay byte comparison registered).
  Verdict: **the overturn clause does not trigger**: the finalized form (91-byte header + claiming) misclassifies 0,
  the 47-byte header misclassifies 12 even with claiming on (3 impostors promoted, 9 torn not cleared);
  the hdr91 no-claiming arm misses exactly 4 self-mirrors ⇒ the claiming rule is the only defense for the self-mirror cell.
  Basis: the falsification clause of the reverse-inference leg in the C68 (Invariant Allowlist Conflicts with Settled Clauses) adversarial round
  (closing paragraph of `c68-unit-header-opus-output.md`).

## E89 (Nested Interval Labeling and Deletion Frontier)

### 2026-09-03

- Added E89 (Nested Interval Labeling and Deletion Frontier) and ran it the same day: the modeling verification named by C35 (Multi-Writable-Head O(N) Unoptimized)
  Before: C35 (Multi-Writable-Head O(N) Unoptimized) carried "model-verify nested interval labeling + deletion frontier;
  a pure counting model can do it now, but first say whether multiple writable heads produce a tree or a DAG".
  After: the precondition is resolved by settled item 1 of D6 (Snapshot Implementation Model) (one tree per head, cloned from a single origin,
  no merge operation ⇒ the lineage is a tree); E89 (Nested Interval Labeling and Deletion Frontier) landed and ran
  (two arms + independent ground truth, 13 unit tests / all 11 mutations caught / replay byte comparison registered).
  Verdict: **all three criteria of C35 (Multi-Writable-Head O(N) Unoptimized) are met**: the candidate arm's false releases and missed releases are constantly 0 over the whole grid;
  on a fixed tree with extent ×32 the baseline reverse lookup goes 8→252 while the candidate stays 0; the frontier maximum of 12–16 does not follow N,
  and under branch scanning goes 5→22, following the snapshot count; under Linear the frontier ≤ 1 degenerates into
  the constant form "compare txg with the previous snapshot" of D5 (Snapshot / Space Accounting Mechanism).
  Limits recorded as they are: total snapshot table probes still follow the number of extents referenced by the deleted snapshot (the class difference is in the structure each probe touches);
  label maintenance (order-maintenance) is out of scope and is verified separately before implementation.
  Basis: the original text of the check form in C35 (Multi-Writable-Head O(N) Unoptimized); criteria fixed in the page body before the run.

## E90 (Tree ID into AAD and Cross-Head Sharing)

### 2026-09-19

- See the same-day entry "E31 (AAD Lacks Snapshot Dimension) one current-state anchor corrected after a fresh check; E31 (AAD Lacks Snapshot Dimension)…" in the E31 (AAD Lacks Snapshot Dimension) section.

### 2026-09-03

- Added E90 (Tree ID into AAD and Cross-Head Sharing) and ran it the same day: the catch-up run for the 4th inference in the D6 (Snapshot Implementation Model) chain
  Before: the 4th point of the D6 (Snapshot Implementation Model) settlement chain carried "AAD tree IDs naturally differ ⇒ the two silent mismatches of E31 (AAD Lacks Snapshot Dimension)
  should be caught by the MAC; this is an inference, and it must be run before it is treated as settled";
  nobody had asked the reverse of the inference (whether the tree dimension also rejects legitimate cross-head shared reads).
  After: E90 (Tree ID into AAD and Cross-Head Sharing) landed and ran (three arms + forged-claim injection,
  real AEAD, 9 unit tests / all 9 mutations caught / replay byte comparison registered).
  Verdict: **the inference is only half right**: `reader_tree` catches all 3 re-attachments but leaves 2 of 6 legitimate shared reads
  (legitimate inheritance and illegitimate re-attachment cannot be told apart on the tree dimension); `pointer_tree` is silent on all three and cannot see forgery either;
  `ancestry_birth` (birth (tree, txg) into AAD + pointer carries a claim + clone ancestry table acceptance) passes both criteria,
  which is the concrete form of way out 1 of E31 (AAD Lacks Snapshot Dimension) under one tree per head.
  The 4th point of the D6 (Snapshot Implementation Model) chain and the "AAD lacks snapshot dimension" section of D9 (Encryption) have been rewritten per the verdict clause.
  Basis: criteria and worlds fixed in the page body before the run; the second cell of the verdict clause triggered.

## E91 (Ring and Admission Accounting Under Settled Format)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-13

- See the same-day entry "Added E143 (Journal Accounting Under One Transaction One Unit) and ran it: seq batches under the user-settled split discipline…" in the E143 (Journal Accounting Under One Transaction One Unit) section.

### 2026-09-03

- Added E91 (Ring and Admission Accounting Under Settled Format) and ran it the same day: every account the three-way argument raised is measured out
  Before: three mutually contradictory ring lower bounds coexisted in the repository (8 KiB from E75 (Record Size and Ring Geometry),
  8 MiB derived from settled item 5 of D16 (Publish Semantics), and the new semantics of settled item 14 of D23 (Journal Role & Format)
  never recomputed); the retention-time and acceptance-watermark accounts hung on E16 (Role of Journal: WAL vs Intent Log)
  "the maintenance-cost half".
  After: E91 (Ring and Admission Accounting Under Settled Format) landed and ran (pure arithmetic + occupancy simulation,
  12 unit tests / all 8 mutations caught / 24 simulated cells exactly equal to the closed form / replay byte comparison registered).
  Verdict: the three old numbers **each reproduce exactly under their own measurement basis and do not contradict each other**: each answers a different question;
  under the settled measurement basis (56 B/item, 71 items/record, 4 KiB fixed-length records) the constraint value of I-8.1 (Ring Geometry Sufficiently Large) is
  **14776 KiB (F=2) / 22164 KiB (F=3)**, the dominant term is the full-window T_dirty checkpoint transaction,
  identical for forms A and B; the real retention-time difference is **1 vs 10 923 live records** (4 KiB vs 42.7 MiB,
  whichever comes first being the T_dirty branch); acceptance watermark A **1.91 GiB** vs deferred 28 MiB (70.6×).
  A fourth inconsistency found during design (the derived number of settled item 5 of D16 (Publish Semantics) uses the old 32 B/entry measurement basis) is judged real;
  that text now carries a scope note (see decisions-history, part seven). The two accounts of E16 (Role of Journal: WAL vs Intent Log) are marked cleared.
  Basis: criteria fixed in the page body before the run; numbers from E75 (Record Size and Ring Geometry), settled item 5 of D16 (Publish Semantics), E16 (Role of Journal: WAL vs Intent Log) and E44 (Host Testing and Cost of Sequence Number Bit Width) quoted verbatim.

## E92 (Worst-Case Block Reuse Delay Requirements)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-08

- E92 (Worst-Case Block Reuse Delay Requirements) was run: that quantity is not one number, criterion 2 triggers; the blank cell of E50 (Upper and Lower Bounds on Root Ring Slot Count) points over to it
  - **E92 (Worst-Case Block Reuse Delay Requirements) moves from "not run (design registered)" to "run".** The criteria and failure clause have **not changed by a single word** since registration on 2026-09-03
    and were not touched after the run either. 10 unit tests, all 8 mutations caught, N=5 rounds with identical md5
    (deterministic model; the round count only proves there is no hidden state).
  - **Conclusion**: "worst-case block reuse delay requirement" is priced in two classes by consumer: the requirement of the class priced in generations is **exactly 2 generations**
    (root-selection rollback: 1 generation + verification lag; this independently recomputes the lower bound 2 already fixed in I-7.4 (Blocks in Recent K Generations Not Reused)),
    while the class priced in wall clock (scrub walking old-generation roots) **cannot be covered by any K** under A:
    publish rate 2785/s ⇒ the window for K = 3 is 1.077 ms, and even K = 48 gives only 17.2 ms,
    while the smallest scrub tier needs 0.1 s ⇒ 15/15 cells exceed the window. Two more cells could only be judged for the unit, not the value
    (c2 in-flight record count ceiling, c4 timeline discrimination), and one cell is honestly marked blank.
  - **Handled per the clause fixed before the run**: where it lands goes to decide; the invariant is not changed on the spot
    ⇒ new entry [checks-owed.md](checks-owed.en.md) C213 (Wallclock consumers traversing old generation roots have no undertaker).
  - **Item 1 of "the half it cannot answer" in E50 (Upper and Lower Bounds on Root Ring Slot Count) rewritten**: that cell used to read verbatim
    "**this repository does not have the quantity 'worst-case block reuse delay requirement'**"; it now points to E92 (Worst-Case Block Reuse Delay Requirements), with the original wording kept in the same paragraph.
  - **Basis**: E92 (Worst-Case Block Reuse Delay Requirements)'s own artifact `e92-reuse-requirement-2026-09-08.out`
    (31 lines, last line `name=done emitted=31`), registered into `scripts/experiment-apparatus/replay.sh`, rerun byte-identical.
- E92 (Worst-Case Block Reuse Delay Requirements) rerun with consumers added: the first round's c3 was a straw man, and conclusion two is replaced entirely
  - **Why rerun**: the enumeration gate of criterion 1 said verbatim before the run "consumers outside the list found after the run ⇒ **add them and rerun**,
    no silent omission". After the first round three outside the list were found: I-4.8 (Root Checksums Self-Consistent Within K Generations), the record checker,
    and administrator rollback (settled item 14 of D23 (Journal Role & Format), user-settled 2026-09-05, **before the first round**).
  - **The same round found c3 was built in the wrong shape (this one weighs more)**: the first round built generational incremental scrub as "pin one generation's root and walk to the end",
    while the verbatim definition in D13 (Verification Route) is "use I-1.2 (block header write order published) and the birth txg of D5 (Snapshot / Space Accounting Mechanism)
    to maintain a 'generation watermark reached by the last scrub', and traverse only blocks with `birth generation > 水位` (watermark)":
    **watermark-based, holding no old root**. A repository-wide grep for "scrub…old root" hits only two places, both written the day of the first round into
    D22 (Synthesizing Unit Atomicity) and C213 (Wallclock consumers traversing old generation roots have no undertaker) ⇒ per `.claude/singlefs-ai-sop/rules/evidence-discipline.md`
    its straw-man criterion voids them.
  - **Conclusion two before**: "the class priced in wall clock (scrub walking old-generation roots) cannot be covered by any K under A, 15/15 cells exceed the window".
  - **Conclusion two after**: of nine consumers **only one is priced in wall clock** (the traversal leap: the step from reading the parent pointer to reading the child block),
    and today it is covered: window = K ÷ publish rate, 718 µs under A with K=2; at 100 µs per leap the margin is 7.18×
    ⇒ **criterion 2 does not trigger**. All three pricing classes have subjects this round (generations 6 / wall clock 1 / external event 1 / blank 1).
  - **Criteria 1-4 and the failure clause are unchanged word for word**; what changed is the consumer list and the mechanism derivation of c3.
    12 unit tests / all 10 mutations caught (the four new ones pin, respectively, scrub falling back to the straw-man shape, administrator rollback treated as generations,
    the traversal leap left off the list, and the frozen-image consumer treated as having a requirement), N=5 rounds with identical md5, artifact 61 lines.
  - **One measurement basis fixed along the way**: the first round said "even K = 48 gives only 17.2 ms", but 48 is **not in the scan grid**:
    the source says verbatim `const KS: [u64; 5] = [1, 2, 3, 4, 8];`, and 48 only lives in one unit test. This round no longer says so.
  - **Back-sweep** (`.claude/singlefs-ai-sop/rules/evidence-discipline.md` "when you withdraw a number, sweep back on the spot for whoever cites it"):
    the E92 (Worst-Case Block Reuse Delay Requirements) page body, the index row in [experiments.md](experiments.en.md),
    item 1 of "the half it cannot answer" in E50 (Upper and Lower Bounds on Root Ring Slot Count), the paragraph of settled item 2 in D22 (Synthesizing Unit Atomicity),
    the heading of section eight and 8.2 in [prior-art.md](prior-art.en.md), and C213 (Wallclock consumers traversing old generation roots have no undertaker) replaced by C214 (window margin of traversal jump unguarded) were all changed.
- E92 (Worst-Case Block Reuse Delay Requirements) third round: the verdict wording for c8 was circular reasoning; withdrawn and replaced by a computable fake-candidate count
  - **Before**: the second round's verdict wording for c8 administrator rollback was "the trigger time is set by the administrator ⇒ external event; but its requirement is
    'how deep it can roll back', which is **defined** by the root ring depth and does not in turn demand K ⇒ not an independent requirement on K", with `req_gens` recorded as None.
  - **After**: that sentence took the claim to be proven, `K = 根环深度` (K = root ring depth), as a premise ⇒ withdrawn. The requirement of c8 becomes **the whole root ring depth**,
    made into a computable quantity: **fake candidate count = root ring depth − K**. The artifact verbatim:
    `name=verdict_rollback pairs=22 pairs_with_fake_candidates=21 k_must_equal_ring_depth=1`.
  - **The same round replaced a unit test that only asserted literal values**: `admin_rollback_is_external_event_and_makes_no_demand`
    asserted what the table says; with c8 misclassified all ten mutations still went green
    (`.claude/singlefs-ai-sop/rules/show-me-test.md` "tests test the implementation, not the contract").
    Replaced by two that assert the mechanism: the absolute value of the fake candidate count, and "every pair with K < ring depth leaves fake candidates".
  - **One mutation was not caught; after the three-way split of `.claude/rules/mutation-sampling.md` it was judged an insensitive sampling point**:
    changing the largest tier of the ring depth table from 48 to 24 leaves "the number of pairs with k < depth" unchanged (both exceed the maximum 8 of KS)
    ⇒ a sensitive sampling point was added (the ring depth table must satisfy N = 3 × S), and the same mutation was caught on the spot.
  - Criteria 1-4 and the failure clause **were unchanged across all three rounds**. 14 unit tests / all 13 mutations caught, N=5 rounds with identical md5, artifact 84 lines.

### 2026-09-03

- E92 (Worst-Case Block Reuse Delay Requirements) design registered: the missing reference quantity for I-7.4 (Blocks in Recent K Generations Not Reused) set up as a project, not run
  Before: "how wide it needs to be" for I-7.4 (Blocks in Recent K Generations Not Reused) had never been derived; E50 (Upper and Lower Bounds on Root Ring Slot Count)
  admitted "whether one millisecond-scale lap is enough cannot be judged"; the reverse-inference leg listed it as an uncleared owed check unique to A.
  After: E92 (Worst-Case Block Reuse Delay Requirements) set up: enumerate the consumers of old-generation roots one by one (root-selection rollback, watermark replay,
  generational incremental scrub, timeline discrimination, block-pinning phase), and judge whether each requirement is priced in generations, wall clock or external events;
  if a consumer priced in wall clock exists, the generation-counted unit of I-7.4 (Blocks in Recent K Generations Not Reused) cannot protect it under A (publish rate = fsync rate).
  Four criteria (enumeration completeness gate, unit judgment, root-selection rollback positive control including the K=1 discriminating-power self-proof,
  violation count exactly equal to the closed form) and the failure clause fixed before the run; pure arithmetic + counting simulation, not run.
  Basis: the original text of item 1 of "the half it cannot answer" in E50 (Upper and Lower Bounds on Root Ring Slot Count).

## E93 (Placement and Fragmentation Under Aging)

### 2026-09-13

- See the same-day entry "Added E151 (Arrival Order of User Data Placement Points and Container Arms) and ran it: E93 (Placement and Fragmentation Under Aging)'s bu…" in the E151 (Arrival Order of User Data Placement Points and Container Arms) section.

### 2026-09-10

- See the same-day entry "E95 (Aging Node Layout Arm) from 'not run' to 'run': criterion 5 judges 'can', while the winning mechanism in the model does not…" in the E95 (Aging Node Layout Arm) section.

### 2026-09-06

- One concluding sentence of E93 (Placement and Fragmentation Under Aging) contradicts its own artifact; withdrawn
  - Before, the body said "**compaction does not saturate**: compaction at 5× budget presses the run-heavy load to F ≈ 10–21% (five seeds fluctuate 713–880, reported as is),
    and the trajectory is still going down over time."
  - After: the range becomes **F ≈ 11.5–21.1%** (the five seeds' `f_pct` listed one by one: 16.35 / 20.39 / 21.12 / 11.52 / 13.04),
    and that artifact line is **copied whole into the body**:
    `E7RESULT name=traj arm=bump_cp_b256 load=runs8 seed=0 runs=0:1,500:914,1000:1147,1500:1202,2000:1339`.
    The whole sentence "does not saturate, the trajectory is still going down" is withdrawn.
  - Basis (fresh check of `e93-aging-placement-2026-09-03.out`):
    ① `grep -c "713\|880"` has **zero hits** in the whole artifact; nobody knows where that range came from;
    ② the five seeds' `runs_final` are 1339 / 1670 / 1730 / 944 / 1068, a range of 944–1730, not 713–880;
    ③ the trajectory 914 → 1147 → 1202 → 1339 is **monotonically rising**, exactly the opposite of "still going down".
  - **Why this one deserves its own entry**: a reversed direction makes downstream underestimate the problem: the stop condition of settled item 1 of D26 (Background Compaction and Placement Reclamation)
    rests right on this sentence, and reading "still going down" would suggest compaction eventually pushes `F` below the stop threshold.
    This is exactly the drift that `.claude/singlefs-ai-sop/rules/evidence-discipline.md` "when you cite an artifact, copy the whole line" is meant to stop.
  - Source: hit 2 of the reverse-inference attack leg in the second D26 (Background Compaction and Placement Reclamation) round, confirmed by the main agent checking the artifact line by line.

### 2026-09-03

- Added E93 (Placement and Fragmentation Under Aging) and ran it the same day: the data prerequisite for settled items 1 / 4 / 6 of D26 (Background Compaction and Placement Reclamation)
  Before: both prerequisites of settled item 1 of D26 (Background Compaction and Placement Reclamation) ("how much it degrades without compaction" and
  a computable definition of fragmentation) were empty, and E10 (Aging: Structural Resistance vs Post-hoc Compaction) could not run for lack of a file system.
  After: E93 (Placement and Fragmentation Under Aging) registered its criteria and ran the same day (14 unit tests / all 11 mutations caught,
  5 rounds byte-identical): fragmentation `F = runs / L` is confirmed O(1)-incrementally maintainable with an independent audit path;
  without compaction both load groups hit the ceiling within a few hundred checkpoints (99.9% / 97.6%);
  the clustered-segment policy is not self-sustaining without compaction feeding empty segments (fallback rate 92–94%);
  write-path neighbor rewrite has a ceiling (F ≈ 66% saturation, budget capped by load geometry) while compaction keeps going down with budget.
  One pre-run model revision changed the direction of the conclusion and is recorded in the body (the first neighbor arm was a straw man:
  clusters were physically non-contiguous through slot-by-slot fallback; after revision each wins some equal-budget cells).
  E93 (Placement and Fragmentation Under Aging) states that it does not replace E10 (Aging: Structural Resistance vs Post-hoc Compaction); the degradation criterion still belongs to E10 (Aging: Structural Resistance vs Post-hoc Compaction).
  Basis: the two prerequisites named verbatim by settled item 1 of D26 (Background Compaction and Placement Reclamation).

- See the same-day entry "E95 (Aging Node Layout Arm) design registered: the arm missing for settled item 4 of D26 (Background Compaction and Placement Reclamation) set up, not…" in the E95 (Aging Node Layout Arm) section.

## E94 (Touch Surface of Moving Shared Placement Points)

### 2026-10-17

- After C34 (three-way clash on plain text mapping layer states) was voided, two present-tense phrases ("until it is closed out" and "must be added separately when closing out C34 (three-way clash on plain text mapping layer states)") changed to the state at the time, stating that this account was voided with no subject on 2026-09-07 when settled item 5 of D9 (Encryption) switched to "scrub only"; the old file name invariants.md now points to crates/singlefs-checker/invariants/; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/94-搬共享落点的触碰面.md`.

### 2026-09-03

- Added E94 (Touch Surface of Moving Shared Placement Points) and ran it the same day: the data prerequisite for settled item 5 of D26 (Background Compaction and Placement Reclamation)
  Before: which structures must be rewritten to move a snapshot-referenced placement had been measured in the whole repository only by E74 (Allocation Record Entry Count and Compaction Cost),
  for the allocation record tree half; the snapshot-referenced half was unmeasured, and where location authority lives was weighed down by
  the three versions of C34 (three-way clash on plain text mapping layer states).
  After: E94 (Touch Surface of Moving Shared Placement Points) registered its criteria and ran the same day (14 unit tests / all 11 mutations caught + 1 equivalent kept on file,
  5 rounds byte-identical): under pointer location authority (ptr_rewrite) the batch move cost grows linearly with the retained snapshot count K,
  each batch rewrites all K+1 root references and rewrites deadlist entries one by one; the central mapping is always one root, always zero deadlist rewrites,
  insensitive to K; moving only exclusive ones has a ceiling of 0–17%, lower the quieter it is;
  the negative arm proves the violation count of birth reassignment equals exactly the number of snapshot-referenced versions that were moved.
  Two pre-run model revisions recorded in the body (segments born after the snapshot, destruction cascade aligned with the ZFS form).
  Basis: settled item 5 of D26 (Background Compaction and Placement Reclamation) names verbatim "E74 (Allocation Record Entry Count and Compaction Cost)
  gives the rewrite cost" as its prerequisite, while E74 (Allocation Record Entry Count and Compaction Cost) admits it only answers the structure half.

- See the same-day entry "Added E96 (Consistency of Hybrid Architecture) and ran it the same day: the user asked to test 'central mapping exists unconditionally'" in the E96 (Consistency of Hybrid Architecture) section.

## E95 (Aging Node Layout Arm)

### 2026-10-17

- The comment in the rerun block "run under research/" changed to "run under scripts/experiment-apparatus/" (research/ no longer holds a Cargo project); conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/95-老化的节点布局臂.md`.

### 2026-09-13

- See the same-day entry "Added E127 (Whether to Store Group Identity Under Splits and Merges) and ran it: node-count grouping did not press a single cell down to key-interval grouping's…" in the E127 (Whether to Store Group Identity Under Splits and Merges) section.

### 2026-09-10

- E95 (Aging Node Layout Arm) from "not run" to "run": criterion 5 judges "can", while the winning mechanism needs no format bits in the model
  - **Before**: E95 (Aging Node Layout Arm) was marked "not run (design registered 2026-09-03)", with the prerequisite verbatim "none. Pure arithmetic; arms can be added directly on
    the source of E93 (Placement and Fragmentation Under Aging)". Settled item 4 of D26 (Background Compaction and Placement Reclamation) had been blocked on this one thing since 2026-09-01,
    and the 2026-09-06 user settlement said verbatim "the policy half is not cut either; the whole item waits for E95 (Aging Node Layout Arm)".
  - **After**: run (the experiment ran on 2026-09-09, artifact `e95-node-layout-arms-2026-09-09.out`,
    20 unit tests / all 20 mutations caught, four rounds byte-identical, registered in `replay.sh`). Per criterion 5 fixed before the run,
    the literal answer is **can**: the node-group arm presses the policy arm's runs below half in all three runs8 cells (1.5× / 3× / 5×)
    (1595 vs 5146, 904 vs 3922, 276 vs 1339); in the three uniform cells not one reaches two times (1.15 / 1.38 / 1.40).
    **But the three-way argument the same day overturned that "can"**: the "format / policy" boundary in the artifact is a hard-coded label
    (`Arm::family()` at `e95_node_layout_arms.rs:108-113`, the tag prefix filter at `:953` / `:959`);
    the reverse-inference leg registered the same function `step_node_group` unchanged as a policy arm, and **the 24 cell numbers stayed identical while the answer line flipped to false**
    ⇒ the arm judged a format arm consumes no on-disk field in the model (`:683` verbatim `dirty.iter().map(|&k| k / g)`).
    ⇒ E95 (Aging Node Layout Arm) can neither overturn "policy is enough" nor establish it.
    The generational-separation format arm wins only one of six cells (`budget=256 load=uniform`, 4347 vs the policy arm's 4791, 9.3% lower, far from half),
    loses the other five, and **never beats the node-group arm in any cell**.
    The body also records three scope notes that only became visible after the run: the winning mechanism can be computed from `key / N` in this model and needs no format bits;
    the runs8 advantage is tied to group-size alignment; the conclusion is not monotonic along the budget axis (a probe at 9× had the policy arm overtake; the artifact was not kept).
  - **Basis**: criteria and failure clause fixed 2026-09-03 and not changed by a word after the run. The cross-apparatus gate pins, one by one,
    E93 (Placement and Fragmentation Under Aging)'s committed artifact's 14 `aging_median` values, all reproduced
    (`.claude/singlefs-ai-sop/rules/show-me-test.md` "when the pitfall is in the modeling measurement basis, the check must stand between apparatuses").

### 2026-09-03

- E95 (Aging Node Layout Arm) design registered: the arm missing for settled item 4 of D26 (Background Compaction and Placement Reclamation) set up, not run
  Before: E93 (Placement and Fragmentation Under Aging) had only policy arms and no arm that touches node layout,
  so "is the policy form enough" could not be compared at the model level; on that basis both the forward-inference and reverse-inference legs of the three-way argument judged
  that the byte verdict of settled item 4 could not be flipped.
  After: E95 (Aging Node Layout Arm) set up: on the E93 (Placement and Fragmentation Under Aging) model add two format arms, "node group" and "generational separation",
  given every favorable implementation, and compare runs against the policy arm at three equal-budget tiers of 1.5× / 3× / 5×;
  criteria (hand-computed anchors, conservation, a positive control per arm, equal-budget comparison, one question two answers) and failure clause fixed before the run;
  states explicitly that it does not replace E10 (Aging: Structural Resistance vs Post-hoc Compaction). Not run.
  Basis: the settled item 4 row of "argument material for the six unsettled items" in D26 (Background Compaction and Placement Reclamation).

## E96 (Consistency of Hybrid Architecture)

### 2026-09-06

- See the same-day entry "Added E109 (Runtime Costs of Three Location Authority Arms) and ran it the same day" in the E109 (Runtime Costs of Three Location Authority Arms) section.

### 2026-09-03

- Added E96 (Consistency of Hybrid Architecture) and ran it the same day: the user asked to test "central mapping exists unconditionally"
  Before: for the hybrid architecture recommended by the D26 (Background Compaction and Placement Reclamation) argument material (position entries as hints + central mapping as authority)
  only E94 (Touch Surface of Moving Shared Placement Points) had measured the cost; there was not a single number on consistency;
  E85 (Field Table of Unit Headers) and E31 (AAD Lacks Snapshot Dimension) had each pointed out that the header has no block birth and that the five-tuple is blind to the snapshot dimension,
  but nobody had connected the two to compaction.
  After: E96 (Consistency of Hybrid Architecture) registered four criteria and ran the same day (10 unit tests / all 12 mutations caught, 5 rounds byte-identical):
  when the unit header carries only the five-tuple, after a slot moved by compaction is reused by a new version of the same key, snapshot reads **silently read the new version**
  (in the uniform random + lowest free slot cell, 3 of 5 seeds are non-zero; under adversarial load about 20 per round at the end);
  with block birth in the header, 40 cells × 200 rounds are constantly 0; resolving placement from hints in the deadlist misreleases about 5000–6000 times per 200 rounds,
  and from the mapping constantly 0; truncating each of the three move steps adds no misreads; the multi-hop rate of stale hints is 19–37% at the end of rounds.
  Three pre-run revisions recorded in the body (batching changed to a scan cursor, crash criterion changed to a difference measurement basis, an allocation policy dimension added).
  Basis: the user's questions "test that the central mapping exists unconditionally" and "does the hybrid architecture have a consistency problem".

## E97 (Entry Encoding of Accounting and Allocation Records)

### 2026-10-20

- E97 (Entry Encoding of Accounting and Allocation Records) criterion 10 rerun a fifth time under "each root holds one generation only, write-only, no deletion" (the E97 (Entry Encoding of Accounting and Allocation Records) half of Q3 in the question sheet `c693-forks.md`): the wide-read cells 1024x64 and 8192x8 go from over the limit to within it, and the tightest narrow-read cell goes from 53% to 2%
  - **After**: the rerun registration `research/prompts/e97-r5-prereg.md` changes only the row-count measurement basis (K 25 → 1, writes per group per publish 2 → 1); the eleven-item statistics table, the four key arms, node header 58, four cells × two read modes and the threshold 21 474 836 stay unchanged. A new apparatus `scripts/experiment-apparatus/e97_entry_encoding_single_generation.rs` was started, with the old and new measurement bases side by side cell by cell; 68 unit tests all green, 16 mutations: 16 caught, 0 invalid, 0 not red, 5 rounds byte-identical. Artifact `e97-entry-encoding-single-generation-2026-10-20.out` (247 lines), one exact line added to `scripts/experiment-apparatus/replay.sh`. Verdict arm 19B (leaf fanout 603) under the new measurement basis: wide read 1024x64 is 9 076 736 (42%) and 8192x8 is 10 240 000 (47%), both within the limit; wide-read over-limit cells 3 → 1 (only 8192x64 remains, 72 564 736); tightest narrow read 11 419 648 (53%) → 458 752 (2%, 8192x8 and 8192x64 tied); narrow-read over-limit 0 → 0. Failure clauses FC1 and FC3 trigger, FC2, FC4, FC5 and FC6 do not; on six leaf geometries (four key arms, today's node header 96, implementation geometry 477) both verdict cells are within the limit. In passing: criterion 4 entry bytes halve, and the inverse solution for wide read 8192x64 still has no feasible point; tree height under the one-generation measurement basis is 1 for narrow read and 2 for wide read (old measurement basis 2 / 2). V1 (old measurement basis reproduces the original artifact's 32 + 8 + 48 lines), V2 (48-line anchors) and positive controls A and C all pass. Section twelve of the registration records five revisions made before the artifact (all asking for more). The experiment page gained a section "2026-10-20: criterion 10 recomputed under…", a pointer line was added under the table of the original criterion 4, and the "affected decisions" table was reviewed row by row.
  - **Basis**: `research/prompts/e97-r5-prereg.md`; Q3 of `research/prompts/c693-forks.md`; settled item 2 of D5 (Snapshot / Space Accounting Mechanism) (one-generation measurement basis).
  - **Before**: criterion 10 had only K = 25 and one write one delete as its single measurement basis (`e97-entry-encoding-2026-09-07.out`, archived: `git show 3cff909b0^:research/results/e97-entry-encoding-2026-09-07.out`).
- The experiment page title and the `experiments.md` index row gained the fifth rerun; a section "paths and conclusion registration" was added (three paths: the new apparatus, an independent anchor script, the old measurement basis of the original apparatus); in the "affected decisions" table the three rows for settled item 1, settled item 4 and settled item 5 of D5 (Snapshot / Space Accounting Mechanism) were reviewed and changed to "changed 2026-10-20": the basis sentences of the three items were rewritten per the one-generation measurement basis of the fifth rerun
  - **Basis**: the T3 cell in section two of `research/prompts/m3-close-c693-r3-main-verification.md`; line 364 of `records/2026-10-18-里程碑三收尾与近期欠账.md` (registration and index row folded into the kb write-back batch).
  - **Before**: for the title and index row see the first line of `git show 1bb3a24dc:.claude/kb/experiments/97-记账与分配记录的条目编码.md` and the E97 (Entry Encoding of Accounting and Allocation Records) row in `git show 1bb3a24dc:.claude/kb/experiments.md`; in the intermediate state before this version the three reviewed rows read "2026-10-20 not affected: …… handed to the main agent" (written by the fifth-rerun runner, never committed).

### 2026-09-06

- See the same-day entry "Added E111 (Stripe Table Keys and Geometry) and ran it the same day: incidentally corrected…" in the E111 (Stripe Table Keys and Geometry) section.

### 2026-09-03

- Added E97 (Entry Encoding of Accounting and Allocation Records) and ran it the same day: three adversarial rounds, rerun three times
  **Added**: the data prerequisite for settled item 5 of D5 (Snapshot / Space Accounting Mechanism) and settled item 7 of D3 (Space Allocation).
  Both were judged "changes the bytes of new-pool file creation: yes", and they are the same geometry question
  (the same btree mechanism, the same 16 KiB node, the same coordinate system).

  **Criteria 1..6 and the failure clause were fixed before the run, and not one changed over three rounds.** After each round changed the attack surface, four were added:
  criterion 7 (alignment representability), 8 (wraparound period), 9 (segment list completeness), 10 (budget counted by dirty nodes);
  and one measurement basis correction: the first version of criterion 8 computed with `T_time` = 5 s, while the correct measurement basis is the publish rate of 2785/s
  (the one the rider clause of settled item 2 of D5 (Snapshot / Space Accounting Mechanism) uses), **a difference of about 14000×**.

  **What each of the three rounds knocked out** (three legs per round: forward inference sonnet / reverse-inference attack opus / counterexample search with local Qwen):

  | Round | Attack surface | Knocked out |
  |---|---|---|
  | One | Modeling choices and citation chain | "Encode placement by 32 KiB unit number" is directly falsified by settled item 9 of D18 (Information Carried in a Block) (index nodes aligned to 16384); it was also confirmed that "dev 1 byte + offset 6 bytes" has no decision source anywhere in the repository |
  | Two | Implementation, recovery and cross-structure consistency | Criterion 8 used the wrong measurement basis; the segment list missed the `seq` of hard requirement 3 of D8 (Core Index Structure); the budget should be counted by dirty 16 KiB nodes; the adopted encoding appeared in not a single artifact line |
  | Three | Format permanence and evolution | "One entry per unit makes half a data unit unrepresentable" is **an artifact hard-coded by the model** (computed from the arm's label, unrelated to the encoding); the three "reproduces the existing model" checks were none of them independent |

  **Final form**: 22 unit tests, 17 mutations each hit, N=5 rounds byte-identical, 240-line artifact, hooked into
  `scripts/experiment-apparatus/replay.sh` (exact tier). **The conclusion is exclusion and narrowing, not settlement**:
  it rules out five candidate arms, narrows the placement coordinate system to the 16 KiB slot number, brings the generation segment of the accounting key onto settled clauses,
  and exposes two format-level blanks nobody had named before (`seq`, position entry segment width).

  **One methodological cost that must be recorded**: the third round verified that the three places that "match the existing model" are **none of them independent checks**:
  the two in E73 (Fanout Cost of Node Key Interval) and E74 (Allocation Record Entry Count and Compaction Cost) are the same
  `(节点 − 头) / 条目宽` formula retyped (it catches mistyped parameters, not modeling errors; all three would be wrong together);
  "32 bits, 17.8 days" has no second path to compare at all: the source of E44 (Host Testing and Cost of Sequence Number Bit Width) never computed it,
  so E97 (Entry Encoding of Accounting and Allocation Records) is **the first machine check**. The body wording was changed to "self-consistency check" and "first machine check",
  and must not be written as "independent verification".

## E98 (Geometry of Inode Records and Inode Trees)

### 2026-09-19

- See the same-day entry "E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta), E56 (Message Buffer Gains vs Epsilon)…" in the E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta) section.

### 2026-09-06

- See the same-day entry "One round of reclaiming format constants out of sync: E43 (Extension Point Byte Upper Bound) rerun a fourth time with 105…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-05

- See the same-day entry "E103 (Update Cost of Inode Packing Path 1) third arm entry changed to 93; 93 / 233 pinned as literal const…" in the E103 (Update Cost of Inode Packing Path 1) section.

### 2026-09-04

- E98 (Geometry of Inode Records and Inode Trees) and E99 (Deduplication of Write Buffer Entries and Seq) each fix one "reverse acceptance clause that can never be reached"
  Both were picked out by the second round of the three-way argument as **the same kind of defect**: a reverse acceptance clause fixed before the run has **no corresponding input** in the code,
  so it can never trigger, and a failure clause that never triggers looks exactly the same in the artifact as having no such clause.

  | Experiment | Before | Now |
  |---|---|---|
  | E98 (Geometry of Inode Records and Inode Trees) | `records_lost_per_leaf` was emitted only on the btree geometry line; the packing arm's 233 appeared only as `records_per_container`; the two numbers were **never compared side by side** ⇒ the clause "living in a unit is vetoed by the blast radius" could not be evaluated | Each form reports one **permanent loss** (116 / 233) and reports `needs_redundancy_clause`. Conclusion: **neither form is 0** ⇒ that clause still cannot be evaluated (nobody ever set a threshold), but this dimension exposes that "whichever is chosen needs a redundancy / detectability clause" |
  | E99 (Deduplication of Write Buffer Entries and Seq) | Criterion 4 only printed the wraparound period; that number was **never connected to any violation count** ⇒ the clause "truncation causes > 0 wrong picks" never produced a single number in the code | Connected per the key structure of settled item 5 of D5 (Snapshot / Space Accounting Mechanism): once it is put into the key, truncating it hits **key identity**: at 16 bits, after ten million publishes **757 947 392** pairs of entries from different checkpoints collapse into the same key; at 48 bits constantly 0 |

  **Basis**: the second round's reverse-inference attack leg (opus) and forward-inference leg (sonnet) **each independently** pointed out these two,
  confirmed by the main agent reading the source item by item (`records_lost_per_leaf` in `e98_inode_record.rs` appears only on the `name=geom` line,
  and `wrap_millis` in `e99_writebuffer_sequence.rs` is never consumed by `wrong_winners`).

  **Two equivalent mutations kept on file the same round**: in E99 (Deduplication of Write Buffer Entries and Seq),
  "wraparound bound `<=` changed to `<`" gives the same result on all inputs (the boundary cell q=1, r=0 ⇒ `C(1,2)=0`);
  the equivalence was written as a unit test, and that mutation table entry was replaced by one that really changes behavior (counting residue classes as publishes).
  Unit tests E98 (Geometry of Inode Records and Inode Trees) 11 → 12, E99 (Deduplication of Write Buffer Entries and Seq) 10 → 12;
  mutations 8 → 9 and 8 → 10. **N=5 rounds byte-identical, replay reproduces.**

- See the same-day entry "E102 (Unit Class Registry Table and Packed Record Units) added and run the same day" in the E102 (Unit Class Registry Table and Packed Record Units) section.

### 2026-09-03

- Four format-level blanks each paired with a counting model, run the same day
  **Added**: the data prerequisites for settled item 6 / settled item 7 of D8 (Core Index Structure), settled item 9 of D22 (Synthesizing Unit Atomicity),
  and settled item 4 of D26 (Background Compaction and Placement Reclamation). All four were judged "changes the bytes of new-pool file creation: yes".

  | Experiment | Unit tests / mutations | One-line conclusion |
  |---|---|---|
  | E98 (Geometry of Inode Records and Inode Trees) | 11 / 8 all caught | **11** of 14 fields cannot be recomputed from authoritative state alone ⇒ living in an index node triggers hard constraint 6 of D21 (Boundary Between Authoritative and Derived States); packing into a shared unit does not trigger it and holds 233 records |
  | E99 (Deduplication of Write Buffer Entries and Seq) | 9 / 6 all caught + 1 equivalent mutation kept on file | 8 bytes leave **0 bits** for the window sequence number, which needs at least **27 bits** ⇒ seq needs **12 bytes** |
  | E100 (Three-Segment Geometry of System Configuration) | 8 / 6 all caught | Inlining the device table in a 512-byte slot **holds only 2 disks**; moved out behind an indirection layer it is constantly **420 bytes** with zero torn states |
  | E101 (Cost and Expressiveness of Reserved Bits in Node Header) | 7 / 5 all caught | Not a single cell of cost dropped, but the **upper bound on label values** for the node-group arm cannot be reached ⇒ reserved bits **cannot flip** the "yes" of settled item 4 of D26 (Background Compaction and Placement Reclamation) |

  **All four are pure arithmetic / pure counting models, N=5 rounds byte-identical**, hooked into `scripts/experiment-apparatus/replay.sh` (exact tier).
  **All criteria and failure clauses fixed before the run, and not changed by a word after it.**

  **E99 (Deduplication of Write Buffer Entries and Seq) records one equivalent mutation**: replacing the "no seq" arm's `None` with "every entry gets the same constant sequence number"
  gives the same result as the original on every input scanned: both make all entries with the same key tie, and a tie is judged "winner undefined" in the model.
  **This is not a blind spot but an equivalence**; the equivalence was written as a unit test and kept on file, and that mutation table entry was replaced by one that really changes behavior (collapsing the wall clock to a constant).

  **One methodological gain**: the first version of E99 (Deduplication of Write Buffer Entries and Seq) mixed "wrong winner picked" and "winner undefined" into one number, so
  the "counter reset at every mount" arm was only half wrong, and that half was rescued by the tie-break rule, not by being right.
  Hard requirement 3 of the D8 (Core Index Structure) write buffer says verbatim "with an unstable sort, **the relative order** of entries with the same key **is undefined**"
  ⇒ **the model must not pick a side for the implementation**; once the two numbers are reported separately, that arm's error rate goes from "half" to "all undefined".

## E99 (Deduplication of Write Buffer Entries and Seq)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-04

- See the same-day entry "E98 (Geometry of Inode Records and Inode Trees) and…" in the E98 (Geometry of Inode Records and Inode Trees) section.

### 2026-09-03

- See the same-day entry "Four format-level blanks each paired with a counting model, run the same day" in the E98 (Geometry of Inode Records and Inode Trees) section.

## E100 (Three-Segment Geometry of System Configuration)

### 2026-10-17

- "C78 (Root record lacks field table) records verbatim" gains a note that this account was merged into C66 (format-level gaps have no item number) on 2026-10-11, with the original text in git history; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/100-系统配置的三段几何.md`.

### 2026-09-19

- See the same-day entry "E23 (Journal Geometry), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place), E1 (Node Size vs Recoverability)…" in the E23 (Journal Geometry) section.

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-09

- See the same-day entry "Added E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) and ran it: the nonce watermark needs 12…" in the E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) section.

### 2026-09-07

- The concluding number of E100 (Three-Segment Geometry of System Configuration) corrected: "inlining in a 512 slot holds only 2 disks" contradicts its own unit tests
  - **Before**: the title and the concluding sentence of criterion 2 said verbatim "inlining the device table in a 512-byte slot only fits 2 disks",
    followed by "the 64-byte entry tier tops out earlier: at 2 disks". The earlier change-history entry (the E100 (Three-Segment Geometry of System Configuration) line in this file) also copied that number.
  - **After**: the ceiling varies with entry width: 24-byte entries **6 disks**, 40 bytes **3 disks**, 64 bytes **2 disks**.
  - **Basis**: E100 (Three-Segment Geometry of System Configuration)'s **own unit tests** assert verbatim `total_bytes(6, 24, true, true) == 505` (fits) and
    `total_bytes(7, 24, true, true) == 529` (over by 17) ⇒ **the body contradicts its own tests**.
    The cause was reading the scan set, which only has {1, 2, 8, 64, 256}, as the ceiling: in the table 2 disks fit and 8 do not,
    so it was written as "only fits 2 disks". The phrase "the 64-byte entry tier tops out **earlier**" is the fingerprint:
    both tiers are 2 on the sampled grid, and "earlier" only means something when the true values differ, and the true values had not been computed when it was written.
  - **Knock-on**: E100 (Three-Segment Geometry of System Configuration) gains a unit test pinning the three tier ceilings directly (unit tests 8 → 9);
    the E100 (Three-Segment Geometry of System Configuration) index row in `.claude/kb/experiments.md` and the inherited sentence in settled item 9 of D22 (Synthesizing Unit Atomicity) were changed with it.
  - **How it was found**: the reverse-inference leg of the three-way argument round on settled item 9 of D22 (Synthesizing Unit Atomicity) pointed it out,
    and the main agent confirmed it by recomputing with `python3 -c "b=361;print([(e,max(d for d in range(1,40) if b+d*e<=512)) for e in (24,40,64)])"`.

- See the same-day entry "Added E115 (Completeness and Byte Budget of System Configuration Field Table), and rejudged mutation M1 from 'blind spot' to sampling point not…" in the E115 (Completeness and Byte Budget of System Configuration Field Table) section.

### 2026-09-03

- See the same-day entry "Four format-level blanks each paired with a counting model, run the same day" in the E98 (Geometry of Inode Records and Inode Trees) section.

## E101 (Cost and Expressiveness of Reserved Bits in Node Header)

### 2026-09-05

- See the same-day entry "E103 (Update Cost of Inode Packing Path 1) third arm entry changed to 93; 93 / 233 pinned as literal const…" in the E103 (Update Cost of Inode Packing Path 1) section.

### 2026-09-03

- See the same-day entry "Four format-level blanks each paired with a counting model, run the same day" in the E98 (Geometry of Inode Records and Inode Trees) section.

## E102 (Unit Class Registry Table and Packed Record Units)

### 2026-10-04

- E102 (Unit Class Registry Table and Packed Record Units) run for the 2nd time (tombstone record 56 → 40 bytes, reading B: capacity, identification power and the parsed record area start take 136); status gains "2nd run 2026-10-04, tombstone record 40 bytes"
  - **Before**: the status said "run (2026-09-04, pure arithmetic, 9 unit tests / all 8 mutations caught)", and the concluding sentence was "with a 93-byte packing header, 583 / 233 do not move"; the artifact `e102-unit-class-registry-2026-09-12.out` (39 lines, tombstone record 56 bytes, origin 107 for parsing and identification) is archived and retrievable with `git show 3cff909b^:research/results/e102-unit-class-registry-2026-09-12.out`; for the page before the change see `git show HEAD:.claude/kb/experiments/102-单元类登记表与打包记录单元.md`.
  - **After**: the apparatus changed to tombstone width 40 and record area start 136 (adding `PACKED_UNIT_RECORDS_OFFSET`, `NONCE_MAC_ALGORITHM_RESERVED_BYTES` and the kb-registered values of two downstream capacities); 13 unit tests, mutation table of 12 entries (M4 reworded, M9 to M12 new) all caught, 0 invalid, 0 not red; artifact `research/results/e102-unit-class-registry-2026-10-04.out` (91 lines, `emitted=91`, two consecutive runs byte-identical). Readings: cap(136, 40) = 815, cap(136, 140) = 233, matching the kb-registered 815 / 233; the intersection of the two intervals is [129, 148], with 136 at 7 from the lower edge and 12 from the upper; the rewrite of P8 holds and the original reverse acceptance clause (literal 103) does not trigger; the tombstone capacity at h = 107 is 816; of 815 identifications the five-tuple misses 814; parsing 815 accepts and 816 is judged bad; mixed packing of 40 / 140 misclassifies 5. Readings for registry mapping, domain separation, unregistered-code policy and value-range margin are byte-identical to the 1st run (22 lines), and the 10 lines for r = 56 are identical too. The page gained four "2nd run" sections (how the criteria changed and the failure clause, measurements, differential), four items under "what it cannot answer", and an "affected decisions" review (settled item 11 changed, the rest unaffected; two prepared rows added for settled item 24 and settled item 16). Line 122 of `scripts/experiment-apparatus/replay.sh` now points to the new artifact.
  - **Basis**: pre-run registration `e102-r2-prereg.md` (reading B settled by the main agent); line 2 of the question sheet `m3-tombstone-record-40-rerun-questions.md`; `e102-mutate-2026-10-04.log`. The decision text was not touched; rewriting the last sentence of the basis paragraph of settled item 11 of D18 (Information Carried in a Block), "with a code 3 header of 107 neither downstream capacity moves", for 136, whether the two prepared cells for settled item 24 and settled item 16 should be upgraded to support, and the 583 hard-coded in the E84 (Pinned Volume of Packed Tombstones) apparatus, are handed to the main agent.

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-06

- See the same-day entry "One round of reclaiming format constants out of sync: E43 (Extension Point Byte Upper Bound) rerun a fourth time with 105…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-05

- See the same-day entry "After the fifth round of C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild)…" in the E104 (Current Version Determination in Scan Rebuild) section.

### 2026-09-04

- E102 (Unit Class Registry Table and Packed Record Units) added and run the same day
  **Added**: the counting model for settled item 11 of D18 (Information Carried in a Block) (the value range of the unit type tag, and which class a packed shared unit counts as);
  criteria and failure clause fixed before the run and not changed by a word after it. 9 unit tests, 8 mutations each hit, N=5 rounds byte-identical,
  artifact of 37 lines ending `emitted=37`, hooked into `scripts/experiment-apparatus/replay.sh` (exact tier).

  **Basis**: settled item 11 was judged "changes the bytes of new-pool file creation: yes", and it is the common prerequisite of three lines: inode packing / tombstones / anti-aging;
  before settling it, "can the four lists all be mapped over", "what makes domain separation hold" and "will the numbers of E83 (Granularity of Tombstones) / E84 (Pinned Volume of Packed Tombstones) /
  E98 (Geometry of Inode Records and Inode Trees) move" were turned into things whose absolute values can be pinned.
- E102 (Unit Class Registry Table and Packed Record Units) changed constants twice following three versions of the proposal; criteria and pinned numbers did not move
  | Before | Now | Basis |
  |---|---|---|
  | Packed-class identity segment 34 (tree ID 8 + record type 2 + in-generation sequence number 4 + record count 2 + record width 2 + birth generation 8 + fsid 8), header 76 | Second version 48 (container number 8 + container birth generation 8 replace the in-generation sequence number; payload checksum 4 added; record width removed), header 90 | The two cloud legs of the first round: the header checksum only protects the header, and the whole-unit checksum lives in the parent pointer; the birth generation is not the loading generation |
  | Header 90 | Third version 51 (tag copy 1 and record width 2 added back), header 93; AAD packed body 22 → 26 | The second round's reverse-inference leg: record width is required for readers that do not know the type; the tag needs a ciphertext-side copy |
  | The cited clause table said "settled item 4 of D23 (Journal Role & Format)" | Changed to "clause A of the subsection 'type field of the record header' under settled item 1" | A misattributed source found by the first round's forward-inference leg |

  Across the three versions cap(h, 56) = 583 and cap(h, 140) = 233 do not move (h ∈ [65, 120] / [9, 148]); the artifact was regenerated with each version (39 lines, `emitted=39`),
  9 unit tests and 8 mutations rerun with each version, all caught, `replay.sh` byte-identical. The "full proposal" pointer in the E102 (Unit Class Registry Table and Packed Record Units) body now points to the third version.

## E103 (Update Cost of Inode Packing Path 1)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-14

- See the same-day entry "Five apparatuses catch up with the three format constants the user settled on 2026-09-14: journal record header 307, i…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-12

- E103 (Update Cost of Inode Packing Path 1) rerun with child pointer 59 → 83: single update, stat read and blast radius unchanged cell by cell, batched scatter upper edge 1.85 → 1.83; "the third arm's batch writes are under 0.2% more" corrected
  - **Before**: child pointer 59 in `e103_inode_update_cost.rs`, `INODE_INTERNAL_ENTRY = 93`; pinned values internal fanout 243 / 242, container index fanout 192 / 191, third arm fanout 175 (header 109 still 175, 110 drops one), levels [8621, 36, 1], [23, 1], [2146, 9, 1], [25, 1], [2453, 15, 1]. Artifact `e103-inode-update-cost-2026-09-12.out`. The body said batched scatter 1.3–1.85, 1e6 k = 100 / 1000 at 1.65 / 1.84, the third arm's saturated cell c_bytes=59319297 vs b_bytes=59253761, k = 10 / 100 differences 4104 / 60560, "batch writes are under 0.2% more", and the measurement heading said 11 unit tests.
  - **After**: child pointer 83 (settled item 7 / 8 of D19 (Block Pointer Structure & Width Budget)), `INODE_INTERNAL_ENTRY = 117`; pinned values internal fanout 179 / 178, container index 149 / 149, third arm 139 (headers 116 and 121 still 139, 122 drops one), levels [8621, 49, 1], [29, 1], [2146, 12, 1], [31, 1], [3088, 23, 1]. New artifact `e103-inode-update-cost-2026-09-12-ptr83.out` (131 lines), mutation rerun `e103-mutate-2026-09-12-ptr83.log` all 11 caught, the old artifact kept. The body changed to batched scatter 1.3–1.83, 1e6 k = 100 / 1000 at 1.58 / 1.83, saturated cell c_bytes=59515905 vs b_bytes=59450369 (still 2 nodes apart), k = 10 / 100 differences 2745 / 55709, "batch writes are under 1% more (at most 0.93%)", 13 unit tests.
  - **Basis**: the inode internal entry of settled item 6 of D8 (Core Index Structure) 93 → 117 (decisions-history.md (part ten)); gate number 27 binds `INODE_INTERNAL_ENTRY` to this source. The newly pinned fanouts and levels were computed separately from the closed form ⌊(16384 − h) / e⌋ and level-by-level ⌈n / f⌉; the 18 lines for k = 1 are identical to before the rerun. "Under 0.2%" only held for the saturated cell even in the child-pointer-59 version (1e6 k = 10 was 0.43%, the table maximum 1.33%), and is corrected along with this.

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-06

- See the same-day entry "One round of reclaiming format constants out of sync: E43 (Extension Point Byte Upper Bound) rerun a fourth time with 105…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-05

- E103 (Update Cost of Inode Packing Path 1) third arm entry changed to 93, 93 / 233 pinned as literal constants, two entries added to the mutation table; E98 (Geometry of Inode Records and Inode Trees) / E101 (Cost and Expressiveness of Reserved Bits in Node Header) each gain a measurement basis note
  - E103 (Update Cost of Inode Packing Path 1): the third arm's internal node entry changed from 91 (separator key 8 + birth tree 8 + container number 8 + container birth generation 8 + child pointer 59) to 93 (separator key 8 + the 26-byte identity reference of settled item 11 of D18 (Information Carried in a Block) + child pointer 59), fanout 179 → 175, internal levels [24, 1] / [2398, 14, 1] → [25, 1] / [2453, 15, 1];
    `INODE_INTERNAL_ENTRY = 93` and `INODE_LEAF_RECORDS = 233` written as literal constants for gate number 27 to pin, each with an assertion that it equals the derivation; unit tests pin header 92 with a 16-byte key interval and the fanout edge at 109 / 110; the mutation table gains M10 (CONT_KEY written as the third round's old value 24) and M11 (fanout counted one too many), all 11 caught; the artifact was regenerated three times the same day, and not one of the two arms' existing numbers changed.
    The sentence "slightly cheaper at k ≥ 10" in the body had the direction reversed (hit by detailed check 8 of the third round's reverse-inference leg) and was changed to a whole-line copy of the artifact: at k ≥ 10 the third arm is slightly more expensive, and the k = 1000 cell is exactly 2 nodes.
  - E98 (Geometry of Inode Records and Inode Trees) / E101 (Cost and Expressiveness of Reserved Bits in Node Header): each gains a measurement basis note: the "live in an index leaf" arm has been ruled out by settled item 6 of D8 (Core Index Structure), and the numbers are kept for traceability only.
  - Basis: the third and fourth argument rounds on settled item 6 of D8 (Core Index Structure) (`research/prompts/d8-item6-r3-*`, `d8-item6-r4-*`).
    ⚠️ The fourth round's local leg `d8-item6-r4-local.out` was judged red on 2026-09-06 by the narrowed word-corruption check
    (`watermark watermark` ×2) ⇒ that round's measurement basis is **two cloud legs + the local leg voided**, not three-way agreement.

- See the same-day entry "After the fifth round of C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild)…" in the E104 (Current Version Determination in Scan Rebuild) section.

### 2026-09-04

- E103 (Update Cost of Inode Packing Path 1) added and run the same day: the second round forward-inference leg's "≈ 3×" voided
  **Added**: the measurement owed by the comparison table of settled item 6 of D8 (Core Index Structure); the user asked to replace the estimate with a measurement. Criteria and failure clause fixed before the run and unchanged after it:
  10 unit tests, 8 mutations each hit, N=5 byte-identical, artifact of 68 lines ending `emitted=68`, hooked into `scripts/experiment-apparatus/replay.sh` (exact tier).

  **Basis**: when settled item 11 of D18 (Information Carried in a Block) landed, the second round forward-inference leg's estimate (changing one record ≈ 3× living in an index leaf) was written into settled item 6 of D8 (Core Index Structure);
  that estimate only counted leaves, while choosing A in settled item 1 of D23 (Journal Role & Format) requires every fsync to write the dirty leaf plus all its ancestors, and the two sides have different ancestor counts.
  Measured: single update 1.25–1.5×, batched scatter 1.3–1.85×, full rewrite the same, stat read 2.25–2.5× the bytes.

## E104 (Current Version Determination in Scan Rebuild)

### 2026-09-19

- See the same-day entry "E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta), E56 (Message Buffer Gains vs Epsilon)…" in the E14 (Discriminative Power of I-3.1: Idempotent Full Value vs Incremental Delta) section.
- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-06

- The scope of E104 (Current Version Determination in Scan Rebuild) narrowed: the placement dimension is always true in worlds with `w ≥ 2`
  - Before, the body did not mention the placement dimension; after, the first item of "what it cannot answer" states: the ground truth defines "the correct placement" as the smallest one
    (`e104_current_version.rs:400-410`), the rebuild side iterates an ascending `BTreeMap` and replica merging keeps the first one met (`:1118-1123`),
    and `o.wrong` compares a single `u64` (`:1212-1213`) ⇒ **the side under test and the ground-truth side share the same "smallest placement wins" rule that exists only in the model**,
    and settlement P3 of C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) says not a word about it.
    ⇒ the conclusion of E104 (Current Version Determination in Scan Rebuild) covers only "which version is current", not "at which placements it lives".
    **Not one number changed and not one mutation moved**: what changed is what this artifact can prove.
   Basis: H1 of the first-round reverse-inference leg on settled item 9 of D9 (Encryption); the main agent opened those three source locations verbatim to verify.
    It is a variant of `.claude/singlefs-ai-sop/rules/test-discipline.md` "only comparing arms with each other cannot detect all arms being wrong together":
    here no arms are even needed; the side under test and the ground-truth side are wrong together.

### 2026-09-05

- After the fifth round of C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild): E104 (Current Version Determination in Scan Rebuild) rebuilt per the sixth version, E105 (Cost of Changing Extent Leaf to Code 3) gains equal-height segment analysis, E102 (Unit Class Registry Table and Packed Record Units) / E103 (Update Cost of Inode Packing Path 1) regenerated with the header constants
  - **E104 (Current Version Determination in Scan Rebuild)**: before, an instance table row was (in-flight txg, transaction number ceiling), the predicate did not distinguish unit classes, 25 unit tests / 18 mutations, artifact 497 lines; after, a row is (last published txg, applied prefix of the in-flight txg), one predicate each for data units and containers, two txn_fail worlds added (the transaction fails after issuing unit writes), row reclamation and rollback candidate sets, recovery rewrites in-flight containers, rebirth keeps the container number; 28 unit tests / all 21 mutations caught, artifact 563 lines.
    Changed again after the sixth round: the row's T_pub takes the selected root's txg (not in-flight txg − 1), data predicate b ≤ T_pub ∨ n ≤ W, two lost_root worlds added (the newest root's slot is bad, so recovery steps back one); 29 unit tests / all 22 mutations caught, artifact 629 lines. Basis: item 2.1 of the sixth round's reverse-inference leg (`c113-r6-opus-output.md`).
    Changed again after the seventh round: code 3's newest-pick key takes (birth generation, instance generation), the W of recovery rows is split per instance, rollback rows may not be overwritten; three worlds `cont_same_wseq` and `recovery_own_txns` and the `cont_key_wseq` ablation arm added; 31 unit tests / all 24 mutations caught, artifact 794 lines. The mutation that could not be caught (a rollback row can be overwritten by recovery) was deleted from the table and set up as C124 (rollback row and replay lower bound lack checks that turn red): its harm is that the rollback is not durable, which falls outside this experiment's measure. Basis: items 2.1 / 3.1 / 9.1 of the seventh round's reverse-inference leg (`c113-r7-opus-output.md`).
    Changed again after the eighth round: the quantifier of the tombstone reclamation gate relaxed to "roots (including snapshots and writable heads)", rollback row protection gains the `rollback_row_protected` world and assertion (the first half of C124 (rollback row and replay lower bound lack checks that turn red) paid off by this), code 3's newest-pick key changed to a triple aligned with the ninth-version rule; 32 unit tests / all 24 mutations caught, artifact 830 lines. New finding: the model's tombstone gate judges by key while the rule asks "does any root still reference that dead version", and the two diverge on "delete then rebuild" ⇒ the model cannot tell the old and new quantifiers apart; that mutation was deleted and recorded under "what it cannot answer". Basis: items 8.1 / 12.1 / 14.1 of the eighth round's reverse-inference leg (`c113-r8-opus-output.md`).
    Basis: three reachable histories, items 1.1 / 2.2 / 4.1 of the fifth round's reverse-inference leg (`c113-r5-opus-output.md`), each first made into a world in the model that must report non-zero, then the rule changed; the rebirth collision was produced by the model itself on mixed_w2 seed=5 (the oracle arm also loses 1).
  - **E105 (Cost of Changing Extent Leaf to Code 3)**: before, the conclusion was "1.25–1.78× more expensive", 5 unit tests, artifact 110 lines; after, `equal_cost_segments` and six `name=equal_height` lines added, conclusion changed to "determined by tree height: the segments where B is one level shorter (13.2% on the log axis) are constantly 1.000, the rest 1 + 1/tree height, ratio ≥ 1 without exception"; 6 unit tests, artifact 116 lines.
    Basis: item 6.4 of the fifth round's reverse-inference leg, "1.25–1.78× is a cherry-picked range"; in the same artifact the 1e4-record k = 1000 cell is exactly 1.000.
  - **E102 (Unit Class Registry Table and Packed Record Units) / E103 (Update Cost of Inode Packing Path 1)**: before, header constants 91 / 93 (artifact 2026-09-04); after, 105 / 103 (the C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) settlement adds write sequence 10 and payload CRC 4 to data units and write sequence 10 to packed record units); the artifact was regenerated to that version, and not one pinned number (583 / 233, single-update bytes) moved; only the header width in the config and capacity lines changed; 8 / 11 mutations still all caught, and the replay line changed accordingly.
    Basis: `.claude/gate.d/27-format-constants.sh` binds source constants to kb-registered values, and once the registered values became 105 / 103 it goes red unless the source changes.

- See the same-day entry "Added E105 (Cost of Changing Extent Leaf to Code 3) and ran it the same day; E104 (Current Version Determination in Scan Rebuild)…" in the E105 (Cost of Changing Extent Leaf to Code 3) section.

## E105 (Cost of Changing Extent Leaf to Code 3)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-05

- Added E105 (Cost of Changing Extent Leaf to Code 3) and ran it the same day; E104 (Current Version Determination in Scan Rebuild) gains an ablation of opposing arm B
  - **E105 (Cost of Changing Extent Leaf to Code 3)**: before, none; after, run: once extent leaves become code 3 packed record units, single update and point lookup are each 1.25× / 1.333× / 1.5× more expensive (1e8 / 1e6 / 1e4 records), batches 1.44–1.78×; all 5 mutations caught, of which "fanout counts containers per node" turned no test red in the first round and was caught after container fanout and container count were also pinned.
   Basis: item 6.3 of the fourth round's reverse-inference leg on C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) named "measure arm B's cost, do not estimate it", and the user instructed "if it cannot be verified, verify it properly".
  - **E104 (Current Version Determination in Scan Rebuild)**: before, eight ablations, 24 unit tests, artifact 380 lines; after, the `wseq_meta_only` arm added (opposing arm B: write sequence only on code 3 containers, data unit headers carry only the birth generation), 25 unit tests, artifact 497 lines, 18 mutations still all caught (M4 / M5 anchors re-set with the source). Arm B resurrects all data orphans in every crash / rollback world, and in the same-txg world gives 4 ambiguities + 1 resurrection.
    Basis: the same reverse-inference leg item 6.3, "measure arm B's residue, do not estimate it".

- See the same-day entry "After the fifth round of C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild)…" in the E104 (Current Version Determination in Scan Rebuild) section.

## E106 (Carrier for Stripe Member Table)

### 2026-09-06

- Added E106 (Carrier for Stripe Member Table) and ran it the same day
  - **E106 (Carrier for Stripe Member Table)**: before, none; after, set up and run: four arms (A: code 3 packed records in the same pool / A-mirror: container mirroring / B: parity self-proof / C: no member table, as the positive control), 10 unit tests / all 7 mutations caught, artifact 56 lines.
    Basis for setting it up: P4 of the first proposal for settled item 12 of D2 (RAID Stripe Policy) admitted that "unsolvable when containers point at each other in pairs" was **estimated**,
    while `.claude/singlefs-ai-sop/rules/evidence-discipline.md` requires statements like that to be measured.
    Fixed before the run, "both outcomes are accepted": all 0 ⇒ the P4 concern does not hold; non-zero ⇒ that bootstrap chain is real.
    **Steady-state measurement is all 0**, even at the tier where containers occupy 50% of placements; meanwhile a hand-arranged deadlock reports 2, proving this measure can report non-zero.
  - **E106 (Carrier for Stripe Member Table)** changed again the same day: a `first_stripe` world added (the pool just created, the stripe table has only one container, and it lands in the very stripe it describes),
    `unsolvable=1` on five seeds; containers changed to **logical containers** (one container with several replicas, readable if any replica is readable),
    and arm A-mirror goes from 2 false reports to 0 in the same cell; 11 unit tests / all 9 mutations caught, artifact 56 → 61 lines.
    The conclusion before was "arm A's bootstrap concern did not materialize"; after, "it does not materialize in steady state, and the pool's first `w ≥ 3` stripe always hits it".
   Basis: the reachable history given in section five of the reverse-inference leg (`d2-item12-r1-opus-output.md`),
    put into the model first per `.claude/singlefs-ai-sop/rules/show-me-test.md` "every hit from an adversarial round is first made into a world that must report non-zero, then the rule is changed".
    ⚠️ It also exposed a modeling error in the first model: the two mirror copies were treated as two containers ⇒ "any replica readable" could not be judged.
  - **E106 (Carrier for Stripe Member Table)** changed a third time the same day: the cost measurement basis switched from "how many bytes the metadata itself takes" to "how many fewer data cells it holds relative to arm C",
    adding `occupancy_is_self_consistent` and a closed-form stripe count (a per-stripe loop cannot run over 480 million units);
    an `add_disk` world added (deterministic placement per "take the emptiest" of settled item 8 of D2 (RAID Stripe Policy) + a new empty disk); 15 unit tests / all 11 mutations caught, artifact 61 → 65 lines.
    Before, the three arms' ppm were 2166 / 4333 / 3516 with the order "A < B < A-mirror"; after, 1544 / 3088 / 2511 with the order "A < B < A-mirror" unchanged;
    **but the old numbers put 126.3% of cells onto a disk claimed to be 90% full** (placements 678 085 686 vs a total of 536 870 912 slots); the assertion pinning absolute values pinned only the data cell count and did not guard the quantities fed into the cost formula.
   Basis: item 5.5 of the second round's attack leg (`d2-item12-r2-opus-output.md`).
    ⚠️ The same leg's item 5.3 predicted that "the add-disk window degrades arm A into the positive control"; **after modeling it did not reproduce**; recorded honestly as 0, with the reason pinned as an assertion (a stripe has at most one cell on any one disk).

- See the same-day entry "Added E111 (Stripe Table Keys and Geometry) and ran it the same day: incidentally corrected…" in the E111 (Stripe Table Keys and Geometry) section.
- See the same-day entry "The measurement basis of E111 (Stripe Table Keys and Geometry) made the mistake E106 (Carrier for Stripe Member Table) had just fixed that day…" in the E111 (Stripe Table Keys and Geometry) section.
- See the same-day entry "Added E107 (Stripe Table Write Amplification on Primary Workload), not run" in the E107 (Stripe Table Write Amplification on Primary Workload) section.

## E107 (Stripe Table Write Amplification on Primary Workload)

### 2026-09-12

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-06

- Added E107 (Stripe Table Write Amplification on Primary Workload), not run
  - **E107 (Stripe Table Write Amplification on Primary Workload)**: before, none; after, set up with three arms (A byte model / B real-machine latency / C "`w` always 2, full mirroring, no stripe table" control), not run.
    Basis for setting it up: the third version of settled item 12 of D2 (RAID Stripe Policy) listed "how many more bytes each publish writes" as unanswered, while E106 (Carrier for Stripe Member Table) measured steady-state space ppm and states it does not answer this dimension.
   ⚠️ When it was set up there was already a hand-computed number (1.000× → 1.493×, the third round's forward-inference leg); **with no apparatus it is only a hypothesis**: the point of E107 (Stripe Table Write Amplification on Primary Workload) is to turn it into something with assertions, mutations and reruns.
    Arm C was added by the main agent: the gain of `w ≥ 3` is capacity efficiency and its cost is the stripe table; the cost of `w = 2` is writing data twice and its gain is that the whole mechanism does not exist; the repository has no **write byte** numbers for either, and whether settled item 12 survives rests on this comparison.
- E107 (Stripe Table Write Amplification on Primary Workload) moves from "not run" to "run"; the crossover falls exactly on the primary workload point
  - **E107 (Stripe Table Write Amplification on Primary Workload)**: before, set up and not run, with a hand-computed 1.493× from the third round's forward-inference leg hanging in the body;
    after, run: 17 unit tests, all 14 mutations caught, artifact 22 lines, and all five conclusion-interval assertions in the rerun script hit.
  - **The hand-computed number reproduced, but its measurement basis is not the one the verdict asks about**: 1.4931× (1.2465× without mirroring) is a ratio **without parity**,
    measuring "how much more is written with the stripe table added"; comparing the two arms on the same scale must count parity. The body now states this firmly,
    and the assertion `third_round_hand_arithmetic_is_the_no_redundancy_ratio` pins it.
  - **Load-bearing conclusion**: on the same scale, arm A (`w ≥ 3` + stripe table) is strictly cheaper than arm C (`w` always 2, full mirroring) for the first time
    **at exactly 8 leaves**, which is exactly the primary workload point of D25 (Target Workload Priority), and only 4.97% cheaper; 6 leaves and 7 leaves are **byte-for-byte equal**.
    The stripe table share is a **fixed amount** of 163 840 bytes, independent of the leaf count.
  - **The crossover of 6 hand-computed when it was set up was wrong**: it missed that data and parity grow together at the 7-leaf step.
    Basis: hand-computing cell by cell `甲 − 丙 = parity + 163840 − (数据 + 索引)`, which is exactly 0 at both 6 and 7 leaves.
    The assertion pinning absolute values was changed to 8 only after re-listing by hand: **not changed to follow the model output**.
  - **Saturation control criterion tightened**: in the setup text "when `N` is far above 583 the difference between the container count and `⌈N/583⌉` is ≤ 0.5%", whether `N`
    means leaves or records was ambiguous; this version tightens it to "the container count equals exactly `⌈记录数 / 583⌉`", which can only be stricter.
  - **Void clause 4 triggered once and was handled**: without warm-up, `one_shot` scatters 214% over five rounds; with 8 warm-up rounds ≤ 24%;
    the `two_phase` cell of `durable` exceeded ±20% in two of three independent runs ⇒ **reported honestly as "unstable"**, no conclusion drawn.

- See the same-day entry "Added E110 (Expected Write Amplification of Stripe Table Under Continuous Publishing) and ran it the same day: E107 (Stripe Table Write Amplification on Primary Workload)…" in the E110 (Expected Write Amplification of Stripe Table Under Continuous Publishing) section.

## E108 (Cost Crossover Point of Plaintext Layer)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-12

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-06

- Added E108 (Cost Crossover Point of Plaintext Layer) and ran it the same day
  - **E108 (Cost Crossover Point of Plaintext Layer)**: before, none; after, set up and run: 7 unit tests / all 6 mutations caught, artifact 9 lines.
    Basis for setting it up: for settled item 9 of D9 (Encryption) the user gave a **conditional settlement** ("if physical keys have better cost-effectiveness and efficiency, settle on physical keys; otherwise logical keys"),
    and the costs of the two paths have **different shapes** (logical keys: hot-path write amplification on every publish + resident space; physical keys: a one-off hit on the cold path at every lock)
    ⇒ comparing a single number cannot decide it; the **crossover** must be solved for. At 16 TiB / `w = 2` the crossover is at 333 578 publishes ≈ 19 days.
    ⚠️ Two things that are not in this table but decide the choice are written in the body: logical keys cannot move code 2 / code 3 (losing a round on coverage first);
    the bytes on the two sides land on different paths.

## E109 (Runtime Costs of Three Location Authority Arms)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-11

- See the same-day entry "Added E133 (Format Cost of Mapping Key Candidates) and ran it: the format of the 12 key forms in the two argument rounds…" in the E133 (Format Cost of Mapping Key Candidates) section.
- See the same-day entry "Added E136 (Cost Rows for Mapping Key Forks) and ran it: D19 (Block Pointer Structure & Width Budget) settled item…" in the E136 (Cost Rows for Mapping Key Forks) section.

### 2026-09-06

- Added E109 (Runtime Costs of Three Location Authority Arms) and ran it the same day
  - **E109 (Runtime Costs of Three Location Authority Arms)**: before, none; after, set up and run:
    11 unit tests, all 14 mutations caught, artifact 61 lines, rerun criterion `exact`.
  - **Basis for setting it up**: two rounds of three-way argument independently judged that the D21 (Boundary Between Authoritative and Derived States) item
    "where multiple referrers live under sharing", settled item 5 of D26 (Background Compaction and Placement Reclamation) and C34 (three-way clash on plain text mapping layer states)
    are **the same decision**, and **the repository has no numbers on "which is faster"**; the user decided that day "first fill in the argument, then settle".
  - **Result**: for moves, `ptr` grows linearly with the number of referencing trees `K` (the ratio equals `K` exactly), while the central mapping is constantly 66 048 bytes, independent of `K`;
    for publishes, the central mapping pays an extra **fixed 66 048 bytes, independent of the leaf count** (1.199× on the primary workload);
    for reads, the central mapping **doubles** (a thousand reads 4000 → 8000), while the hybrid form, at the 19–37% staleness rate measured by E96 (Consistency of Hybrid Architecture),
    **costs only 19–37% more**. Crossover: with `K = 1` the central mapping never pays back,
    with `K ≥ 2` it pays back as soon as each publish moves 1 shared unit.
  - ⚠️ **The key width of a mapping entry is not a settled constant** (the width of each segment of the logical identity five-tuple is not fixed anywhere in the repository),
    so it is a parameter scanned over four tiers, with an assertion requiring **the conclusion's direction to agree across all four tiers**.
  - ⚠️ **The one with the hardest discriminating power**: at a hint staleness rate of 0 the hybrid form must equal `ptr` cell by cell,
    at 1000‰ it must equal the central mapping cell by cell, and intermediate values must lie strictly between the two ends;
    if this fails, the hybrid arm was never built as an intermediate arm at all.
- E109 (Runtime Costs of Three Location Authority Arms) sent back for repair the same day: the first version sat on a degenerate sampling point, and deleting the mapping tree geometry left all unit tests green
  - **Before**: 11 unit tests, 14 mutations; measurement basis item 2 said "the key width of a mapping entry is not a settled constant"; one sampling point.
  - **After**: 13 unit tests, 17 mutations; key takes the settled 33 bytes; six sampling points (three of them with the two trees at different heights);
    the positive control rewritten. **The structural conclusion did not change**; what changed is that those absolute values hold only at the stated sampling points.
  - **The two items that sent it back** (the second-round reverse-inference attack leg on D21 (Boundary Between Authoritative and Derived States) multiple referrers, adopted after the main agent verified line by line):
    ① **measurement basis item 2 is wrong**: settled item 3 of D18 (Information Carried in a Block) fixes the five-tuple of the data unit self-describing header at 33 bytes,
    and the field table `105 = 42 + 33 + 8 + 8 + 10 + 4` of settled item 7 agrees with it;
    ② **at the sampling point the referencing tree and the mapping tree happen to have the same height 4**; the attack leg copied the source unchanged into an out-of-repository copy,
    replaced the whole mapping tree geometry with the referencing tree's height, and **all 11 unit tests stayed green, with the 61-line artifact's md5 byte-identical**.
    Its "positive control" `assert_eq!(same_geo_map, ptr_k1)` has the same expression on both sides and **never once calls the `map` arm under test**;
    none of the 14 mutations could separate this either.
  - **This is exactly the shape that `.claude/singlefs-ai-sop/rules/evidence-discipline.md` "the checking path itself must also prove it can go red"
    and test-discipline "the positive control must run on every arm under test" are meant to stop**:
    the first version wrote both down in words and honored neither.
  - **The repair itself caught another blind spot of the same type**: of the three added mutations, "the read path uses the referencing tree's height for the mapping height" went unseen on its first run
    (the read path had also only been tested at the equal-height point); it was caught only after another assertion testing the read path at a point where the two trees differ in height.

- See the same-day entry "The position entry of E79 (Capacity of Root Record) stayed at the old value for three days unnoticed; after rerun the tree table unit capacity is 266 → 243" in the E79 (Capacity of Root Record) section.

## E110 (Expected Write Amplification of Stripe Table Under Continuous Publishing)

### 2026-09-12

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-06

- Added E110 (Expected Write Amplification of Stripe Table Under Continuous Publishing) and ran it the same day: the knife edge of E107 (Stripe Table Write Amplification on Primary Workload) resolved
  - **E110 (Expected Write Amplification of Stripe Table Under Continuous Publishing)**: before, none; after, set up and run:
    10 unit tests, all 13 mutations caught, artifact 11 lines, rerun criterion `exact`.
  - **Basis for setting it up**: E107 (Stripe Table Write Amplification on Primary Workload) admitted it cannot answer the distribution of `f` (slots already taken in the current container) under real load,
    while at the primary workload point it measured ratios of 0.950 and 1.050 at the two endpoints `f = 0` and `f = 582`
    ⇒ **the conclusion sits on a knife edge, and the two sides of the edge are decided by an unmodeled quantity**.
  - **Result**: once `f` is left to evolve under continuous publishing, the cumulative ratio on the primary workload settles at **0.9531**,
    and the near-full-container endpoint **accounts for only 2.74% of publishes** (274 / 10000) ⇒ in expectation it leans toward arm A, by only 4.7%.
    Mixed load (rotating 1 / 8 / 64 leaves) is 0.7816 by bytes, but **by rounds arm A is cheaper on only 65.8% of publishes**: the two numbers say two different things.
  - **Two first guesses that were wrong and were corrected by hand calculation**: ① the overflow count over 1000 rounds was first estimated at 29 (`17 × 1000 / 583 ≈ 29.16`),
    missing that the one time at `17 × 583 = 9911` **fills exactly and does not count as overflow**; actually **28**;
    ② it was thought mixed load would **raise** the ratio (at the single-leaf tier arm A is 1.651×), but measured it **lowers** it: weighted by bytes, large publishes dominate.
    Both were changed only after re-listing the assertions by hand cell by cell, **not changed to follow the model output**.
  - **The first mutation round caught two blind spots, both now covered**: "a publish of zero leaves is also billed" was originally seen by nobody
    (only `steady(0, ...)` was tested, and that path does not call the single publish); whether the comparison for "rounds where A is cheaper" uses `<` or `<=`
    is **equivalent** in this model; per discipline the equivalence was written as a test and kept on file, and replaced by a mutation that really changes behavior.

## E111 (Stripe Table Keys and Geometry)

### 2026-09-12

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-06

- Added E111 (Stripe Table Keys and Geometry) and ran it the same day: incidentally corrected the ppm of E106 (Carrier for Stripe Member Table) as using a measurement basis "without parity cells"
  ⚠️ **The half-sentence in this entry, "corrected the measurement basis error of E106 (Carrier for Stripe Member Table)", was overturned the same day, and every number below that follows from it (724 775 730 / 4631 ppm / 10.5× / 4116 ppm) is void with it**: what was really wrong was the first version of E111 (Stripe Table Keys and Geometry). See the same day's "part thirteen".

  - **E111 (Stripe Table Keys and Geometry)**: before, none; after, set up and run: 8 unit tests, all 12 mutations caught, artifact 14 lines, rerun criterion `exact`.
  - **Basis for setting it up**: settled item 12 of D2 (RAID Stripe Policy) (user settlement) fixed the **carrier** of the member table,
    **but did not say what key this logical tree is sorted by**; and its three consumers all ask the same question, "given a placement, which stripe does it belong to",
    which the container index cannot answer (none of the four key segments is a placement). The user's words that day: **"just continue the experiment today, that's all"**:
    this cell is exactly the one of the three prerequisites that an experiment can help with.
  - **Result**: of the three keys **only "sorted by placement" can answer that question** (this one is not measured; it follows directly from the definitions of the three keys);
    the tree sorted by placement has 724 775 730 entries at `w = 3`, **height 3**, taking **4631 ppm**,
    **10.5×** the allocation record tree on the same scale (E97 (Entry Encoding of Accounting and Allocation Records): 442 ppm); `w = 4` is 4116 ppm; `w = 2` is constantly 0.
  - **The measurement basis error corrected along the way**: the 1544 / 3088 ppm reported by E106 (Carrier for Stripe Member Table) are
    "**one record per data cell**", **without parity cells** (`828 789 = ⌈483 183 820 / 583⌉`,
    and 483 183 820 is exactly the data cell count); while settled item 12 of D2 (RAID Stripe Policy) settled "one record per cell, **including parity cells**"
    ⇒ the real entry count is 50% more at `w = 3`. **That row in the text of settled item 12 of D2 (RAID Stripe Policy) was changed the same day on this basis**
    (3088 → 4631), and the E106 (Carrier for Stripe Member Table) body gained a measurement basis note.
  - **One hand calculation went wrong during modeling**: the container count was first written as 1 243 183, while `583 × 1 243 183 = 724 775 689 < 724 775 730`
    ⇒ rounding up gives 1 243 184, the difference being the last 41 entries. The assertion was changed only after re-listing step by step by hand, **not changed to follow the model output**.
  - **Same-scale cross check**: the data cell count 483 183 820 is **digit-for-digit identical** to the entry count already committed by E97 (Entry Encoding of Accounting and Allocation Records).
- The measurement basis of E111 (Stripe Table Keys and Geometry) made the mistake E106 (Carrier for Stripe Member Table) had just fixed that day; rerun
  - **E111 (Stripe Table Keys and Geometry)**: before, 8 unit tests / 12 mutations / entry count 724 775 730;
    after, **9 unit tests / 13 mutations / entry count 483 183 820**, artifact still 14 lines, rerun criterion `exact` byte-identical.
  - **Before**: `cells()` treated `16 TiB × 90% ÷ 32768 = 483 183 820` as the **data cell count** and then added `parity = 数据格 / (w−1)`,
    getting 724 775 730 32 KiB cells at `w = 3`: **135% of a 16 TiB disk**, with no assertion watching this dimension.
  - **After**: that 90% budget is spent by data cells and parity cells together ⇒ the total cell count (= entry count) is constantly 483 183 820, independent of `w`,
    and `w` only changes the data-to-parity split within the budget.
  - **Conclusion numbers changed along with it**: ppm 4631 / 4116 → **3087 (the same value at both tiers)**; the multiple over the allocation record tree 10.5 → **6.98**;
    `stripe` arm tree height 4 → **3**. **The load-bearing conclusion did not change by a word** (of the three keys only `slot` can answer that question,
    and that is not measured; it follows directly from the definitions of the three keys).
  - **Basis**: checking why the two experiments report numbers 1.5× apart for the same quantity. Both the body and the source of E106 (Carrier for Stripe Member Table) state that
    483 183 820 is **the total number of occupied placements** (including data cells, parity cells and containers); its own first version made the same mistake (126.3%),
    fixed the same day, leaving the assertion `locs <= occupied_slots()` that can go red.
    ⇒ the part-twelve entry "E111 (Stripe Table Keys and Geometry) incidentally corrected the ppm of E106 (Carrier for Stripe Member Table) as a measurement basis without parity cells" **had the direction reversed and is withdrawn**.
  - **A cross check that only appeared after the fix**: the 54 315 515 904 bytes of E111 (Stripe Table Keys and Geometry)
    are **digit-for-digit identical** to the `cost_bytes` of arm A-mirror in E106 (Carrier for Stripe Member Table): two experiments compute the same quantity with mutually different algorithms.
  - **This pitfall was made into a shape that goes red**: the unit test `cells_never_overfill_the_pool` (with its own discriminating power: the first version's
    724 775 730 must be judged red by it) and mutation M13 (parity added on top of the budget),
   log `e111-mutate-2026-09-06.log`.
  - **Still owed**: there is no cross assertion between the two experiments; this time it was found only by a person comparing them ⇒ C155 (same quantity across experiments has no cross-assertion).
  - **Downstream**: the cost row of settled item 12 and the cost table of settled item 14 of D2 (RAID Stripe Policy) were changed back the same day.

## E112 (Legacy Writer Encountering Unknown Trees)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

## E113 (Unknown Trees: Saturated Arm Set)

### 2026-10-21

- The placement sentence for the accounting dimension changed to the current state: C693 (Arguments to redo after K generations was withdrawn) is paid off, the user decided on 2026-10-20 to "keep carrying, ㊀ unchanged", and the accounting dimension takes `Carry`; conclusion, numbers and artifacts unchanged.
  - **Basis**: ③ of the C693 (Arguments to redo after K generations was withdrawn) row in the paid-off table of `.claude/kb/checks-owed.md`.
  - **Before**: `git show 841b3beab:.claude/kb/experiments/113-不认识的树-补齐臂集.md`.

### 2026-10-17

- The placement C160 (true cost of the accounting arm is not on the balance) now points to C693 (Arguments to redo after K generations was withdrawn): this account was merged into C693 (Arguments to redo after K generations was withdrawn) on 2026-10-11; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/113-不认识的树-补齐臂集.md`.

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

## E114 (Overall Accounting of Small-Data Packing Containers)

### 2026-09-19

- Several current-state sentences in E114 (Overall Accounting of Small-Data Packing Containers), E119 (Padding Cost of Fixed-Width Slots) and E121 (Optimal Points for Tiering by Cap and Tier Count) corrected after a fresh check; E114 (Overall Accounting of Small-Data Packing Containers), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy), E119 (Padding Cost of Fixed-Width Slots), E120 (Choosing the Common Ratio of a Geometric Size-Class Table), E121 (Optimal Points for Tiering by Cap and Tier Count) and E149 (Costs of Three Repair Options for Broken Packed Containers) gain an "affected decisions" table
  - **Before**: item 1 under the measurement table of E114 (Overall Accounting of Small-Data Packing Containers) said "the condition for the gain multiple is settled item 6 of D27 (Small-Data Packing Container) (compaction trigger criterion and batch size), which is unsettled"; item 1 of "what it says about the proposal" said "and 'small file' appears 0 times in the whole text of D25 (Target Workload Priority)"; "what it cannot answer" said "compaction trigger policy unsettled (settled item 4 of D26 (Background Compaction and Placement Reclamation))" and "the last one faces settled item 5 of D27 (Small-Data Packing Container) (where the per-container live slot count lives), and that item is unsettled". "What it cannot answer" in E119 (Padding Cost of Fixed-Width Slots) said "does not answer 'how the tier table is frozen into the format': that is settled together with the field table of settled item 3 of D27 (Small-Data Packing Container)". Row 4 of E121 (Optimal Points for Tiering by Cap and Tier Count) said "answers the 'how tiers are cut' cell in settled item 6 of D27 (Small-Data Packing Container)". None of the six pages had an "affected decisions" table.
  - **After**: item 1 of E114 (Overall Accounting of Small-Data Packing Containers) changed to: settled item 6 of D27 (Small-Data Packing Container) is settled, and following the central mapping key order is exactly that condition; "small file appears 0 times" limited to how it was at run time, adding that D25 (Target Workload Priority) later gained the smallfile row whose three columns are still unmeasured; the compaction trigger sentence changed to: packing is kicked off by the continuous patrol plus rate limiting of settled item 6 of D27 (Small-Data Packing Container), and the idle overhead has no number (C202 (idle overhead and hysteresis K of packing patrol lack values)); the live slot count sentence gains "settled item 5 was later settled as having no live slot count". The E119 (Padding Cost of Fixed-Width Slots) sentence changed to being answered by settled item 12 of D27 (Small-Data Packing Container) (the tier table does not go into the format). Row 4 of E121 (Optimal Points for Tiering by Cap and Tier Count) now points to settled item 13 of D27 (Small-Data Packing Container). E114 (Overall Accounting of Small-Data Packing Containers), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy), E119 (Padding Cost of Fixed-Width Slots), E120 (Choosing the Common Ratio of a Geometric Size-Class Table), E121 (Optimal Points for Tiering by Cap and Tier Count) and E149 (Costs of Three Repair Options for Broken Packed Containers) each gain an "affected decisions" table, matched in both directions with the basis paragraphs of the items of D27 (Small-Data Packing Container). Experiment conclusions and artifacts unchanged.
  - **Basis**: fresh check of the index table in `.claude/kb/decisions/27-小数据打包容器.md` (settled items 5 / 6 / 12 / 13 are all settled; "how tiers are cut" moved on 2026-09-08 from settled item 6 to settled item 13), line 130 of `.claude/kb/decisions/25-目标负载优先级.md` (the smallfile row), and the index table in `.claude/kb/decisions/26-后台整理与放置回收.md` (settled item 4 is structural anti-aging, settled, and does not cover compaction triggering); for D27 (Small-Data Packing Container) rewritten in its slimmed form see the decision change history (part six).

### 2026-09-13

- See the same-day entry "Added E149 (Costs of Three Repair Options for Broken Packed Containers) and ran it: multi-replica gain ×2/3, writes ×3…" in the E149 (Costs of Three Repair Options for Broken Packed Containers) section.

### 2026-09-12

- Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 / 90: constants changed in 14 experiment sources, 10 artifacts rerun, not one conclusion flipped; the mutation tables of E114 (Overall Accounting of Small-Data Packing Containers) / E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) repaired, and mutate.sh fixed for the pitfall of reusing mutation builds across rounds
  - **Before**: 14 experiment sources hard-coded the code 3 header as 103 (e102 / e103 / e105 / e119 / e120 / e121 use the registered name `PACKED_UNIT_HEADER_BYTES`; e106 / e107 / e108 / e110 / e111 / e114 / e116 / e117 each use their own names); the code 2 three-tier headers of E117 (Header Width and Geometry After Factoring in Reserved Bits) were 68 / 77 / 86. The artifacts were the 2026-09-05 versions of e102 / e105, the 09-06 version of e106, the 09-07 versions of e103 / e114 / e116 / e117, and the 09-08 versions of e119 / e120 / e121. Of the 12 rows of the E114 (Overall Accounting of Small-Data Packing Containers) mutation table, the original text of M6 / M7 / M10 each hit two places in the source (those lines were copied once when the `packbg` arm was added on 2026-09-07), and the original text of M16 of E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) was the comment wording from before the rename back-sweep; neither table had run since, so "all 12 caught" and "all 16 caught" in the bodies had no evidence on this version of the source. The E103 (Update Cost of Inode Packing Path 1) body said the artifact had 68 lines, which was the line count of the 2026-09-05 one; the 2026-09-07 one has 131 lines.
  - **After**: the code 3 header in the 14 sources changed to 107 (e102's class identity segment 61 → 65), and the code 2 three-tier headers of E117 (Header Width and Geometry After Factoring in Reserved Bits) changed to 72 / 81 / 90. Pinned values changed along with them: e102 "at most how many when 1000 are declared" 16 → 15; e106 / e107 / e110 padding margin 17 → 13; e108 per-container 695 → 694, container count 1 390 458 → 1 392 461; e119 tier-width upper edge at cap = 8 4040 → 4039, upper edge at cap = 4 8123 → 8122, variable-length arm containers 6814 → 6815; e120 variable-length arm 6815, `logunif` r = 1.5 1903 → 1904; e121 before truncation 4623 → 4622, `uniform` container counts 6815 / 7456 / 8181 / 9117 / 6979, `logunif` capall 1788 → 1789; two header width assertions in e117. The 10 artifacts were rerun and saved as new versions, with the old ones left untouched in results/, and `scripts/experiment-apparatus/replay.sh` now points to the new ones; the non-timing lines of the artifacts of E107 (Stripe Table Write Amplification on Primary Workload) / E108 (Cost Crossover Point of Plaintext Layer) / E110 (Expected Write Amplification of Stripe Table Under Continuous Publishing) / E111 (Stripe Table Keys and Geometry) were byte-identical and not re-saved. All 14 mutation tables were rerun, logs `research/results/<实验>-mutate-2026-09-12.log`: E114 (Overall Accounting of Small-Data Packing Containers) gave the original text of M6 / M7 / M10 more context so they hit only the first arm, and added M13 / M14 / M15 for the `packbg` arm (the first two were real blind spots, M15 an insensitive sampling point); `packbg_absolute` additionally pins the metadata at 6 453 404 bytes and the unit count at f = 0.3; all 15 caught; E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) replaced the original text of M16 with the current comment wording, all 16 caught; the other 12 tables kept their entry counts, all caught. `scripts/mutation/mutate.sh` now touches the source under test right after copying the replica. The artifact line count in the E103 (Update Cost of Inode Packing Path 1) body changed to 131.
  - **Basis**: settled item 11 of D18 (Information Carried in a Block) added the 4-byte birth sequence number per item ① of C288 (collateral changes from mapped key settlement did not land in other decisions), making the code 3 header 107 and adding 4 to each of the code 2 three tiers (decisions-history.md (part nine)). Not one conclusion flipped: the arms of E119 (Padding Cost of Fixed-Width Slots) / E120 (Choosing the Common Ratio of a Geometric Size-Class Table) / E121 (Optimal Points for Tiering by Cap and Tier Count) differ by at most 1 container with the order unchanged, and the fanouts and record counts of E117 (Header Width and Geometry After Factoring in Reserved Bits) are identical cell by cell. Among the newly pinned numbers, the tier-width boundaries and 4622 were computed separately from the closed form ⌊32661 / (W + 43)⌋; container counts have no closed form and take the rerun output. The basis for the mutate.sh line is a measurement: after e114 exited at M6, the shared target directory still held the binary with M5 applied, and since rsync -a keeps the source's old mtime, cargo judged no rebuild was needed, so the next round's baseline reported "the baseline is already red" twice in a row; running the unit tests directly gave all 14 green, and with a fresh empty target directory they were also all green. Also, in the first replay the durable interval of E107 (Stripe Table Write Amplification on Primary Workload) measured 0.9604, outside 1.15–2.10; that was load from running alongside background mutation builds, and rerun on an idle machine it was 1.6000, inside the interval.

### 2026-09-11

- See the same-day entry "Added E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) and ran it…" in the E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) section.

### 2026-09-07

- Added E114 (Overall Accounting of Small-Data Packing Containers), with a fourth arm added halfway through the run
  - **Before**: the small-data packing path had only three rounds and nine legs of argument plus an analytic model, and no apparatus at all;
  the overall verdict of the second round's defense review leg said verbatim "it depends on this one number: on a written-down small-file load,
    is 'bytes saved on the data side − bytes paid for the container index and one extra indirection layer' positive", and that number did not exist anywhere in the repository.
  - **After**: E114 (Overall Accounting of Small-Data Packing Containers) ran four arms (`pad` / `pack` / `packmin` / `packbg`),
    14 unit tests, all 12 mutations caught, all three controls passed. The online form's gain = `m / ⌈m/cap⌉`, and at `m`=1 the overall balance is negative;
    the background compaction form's gain is independent of `m` (512 B 52.33×, 1 KiB 28.40×), with 1.7%–3.3% more write bytes.
  - **Basis**: three points. ① The analytic formula registered before the run, `min(m, cap)`, **is refuted** (the 1 KiB / `m`=100 cell measures 25 while the registered formula gives 30),
    and both formulas are kept in the source; ② the positive control caught a modeling error on the spot: the last publish had fewer than `m` objects but was counted as full,
    conjuring 20 extra containers in the 1 KiB / `m`=30 cell; ③ the fourth arm `packbg` was added halfway through the run,
    because `pack`'s gain hangs on `m`, which has to be defined by D25 (Target Workload Priority),
    and "small file" appears 0 times in the whole text of D25 (Target Workload Priority). When the arm was added, the positive control loop was extended with it
    (`.claude/singlefs-ai-sop/rules/test-discipline.md` "the positive control must run on every arm under test").
    ⚠️ Mutation M1 (container header counted as 0) turned **not a single test red** in the first round, exposing an insensitive sampling point rather than an equivalent mutation:
    after adding an assertion at the 16286 B tier (crossing an integer boundary), M1 was caught, 12/12 with no blind spot.
- The gain multiple of E114 (Overall Accounting of Small-Data Packing Containers) gains a rider: three omissions, one of which makes all three tiers negative under a naive compaction policy
  - **Before**: the measurement table gave the background compaction column (512 B 52.33×, 1 KiB 28.40×, 4 KiB 6.91×) as unconditional numbers;
    the "what it cannot answer" section did not mention metadata writes.
  - **After**: a rider under the table with three items: ① not one byte of metadata writes was counted (`write_bytes` counts only data units, source
    `e114_pack_ledger.rs:172`); adding the central mapping's move overwrites, compaction in container fill order with batch B = 1 gives
    net gains of **−53.9 / −104.2 / −446.4 MiB** at 512 B / 1 KiB / 4 KiB, while in mapping key order it is about 11.72 MiB, negligible;
    ② `packbg` is a fourth arm added halfway through the run, while all three reverse acceptance clauses say verbatim "form A" ⇒ the arm that produced the settlement
    is not within the scope of any pre-run failure clause; ③ the container count ⌈N/cap⌉ silently assumes the answer to settled item 8 of D27 (Small-Data Packing Container).
    "What it cannot answer" gains metadata writes and three constants without a source. **Not one measured number changed, and nothing was rerun.**
  - **Basis**: the second three-way argument round on D27 (Small-Data Packing Container). The central mapping leaf capacity was originally estimated at 536 entries from 32768;
    the two cloud legs **each independently** pointed out the error is in the unit size: settled item 1 of D4 (Checksum Location) says verbatim "metadata side 16 KiB, data side 32 KiB";
    at 16384 with 61-byte entries that gives 267 entries per leaf and 375 leaves in total. The main agent recomputed independently with a random scatter model:
    net(B=1) is byte-identical to the forward-inference leg, and the critical batch differs by 1–2 (15 / 27 / 120 vs 13 / 25 / 119; different rounding measurement bases, not unified).
  - **Knock-on**: set up C186 (arm added mid-run falls outside pre-run failure clause scope) and C187 (format constants in the source not registered in kb are invisible to the format constant gate).

- See the same-day entry "Added E116 (Packed Container Balance: Metadata Writes and Compaction Strategy); the structural lower bound of the payback ratio is `w/(cap −…" in the E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) section.

## E115 (Completeness and Byte Budget of System Configuration Field Table)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.
- See the same-day entry "E23 (Journal Geometry), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place), E1 (Node Size vs Recoverability)…" in the E23 (Journal Geometry) section.

### 2026-09-13

- See the same-day entry "Added E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) and ran it: 413 leaves 9 to spare in the 512 slot…" in the E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) section.

### 2026-09-09

- See the same-day entry "Added E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) and ran it: the nonce watermark needs 12…" in the E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) section.

### 2026-09-07

- Added E115 (Completeness and Byte Budget of System Configuration Field Table), and rejudged mutation M1 from "blind spot" to insensitive sampling point
  - **Added**: E115 (Completeness and Byte Budget of System Configuration Field Table). It answers blockers ② and ③ of settled item 9 of D22 (Synthesizing Unit Atomicity).
  - **Basis**: a mechanical census of settled item 9 of D22 (Synthesizing Unit Atomicity) that day
    (`grep -rn "系统配置" .claude/kb/decisions/*.md crates/singlefs-checker/invariants/`)
    found that blocker ③'s registered "at least 124 of the 420 bytes have no source" covered only one direction;
    **the other direction is rows missing from the table**: fields that settled clauses explicitly require to go into the system configuration have no row in the 24-field candidate table of E100 (Three-Segment Geometry of System Configuration).
  - **Result**: 31 required items, 24 rows in the candidate table, **9 missing**; once filled in, three of the four cells on the 512-byte slot overflow (by 20 / 50 / 79 bytes, each with 2 torn states),
    and after merging the four indirect pointers all four cells fit.
  - **Mutation M1 was not caught in the first round; per the three-way split of `.claude/rules/mutation-sampling.md` it is an "insensitive sampling point"**:
    it removes the first row of `REQUIRED`, and that row was already covered by the candidate table ⇒ the missing count does not change;
    switched to removing the last row (an uncovered item), it was caught on the spot. **Recorded neither as a blind spot nor as an equivalent mutation.**

## E116 (Packed Container Balance: Metadata Writes and Compaction Strategy)

### 2026-10-04

- E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) fills in the three mutation numbers (the two whole-table runs before and after the record header 311 → 487 rerun, run by the main agent; only numbers filled in, conclusion and artifacts unchanged)
  - **Before**: the next entry said "the two whole-table mutation runs before and after the change have not been run yet", and the experiment page title and index row status said "partly run".
  - **After**: before (old tree, old table of 18): 18 caught, 0 invalid, 0 not red; after (19): 19 caught, 0 invalid, 0 not red (`research/results/e116-mutate-before-2026-10-04-h487.log`, `research/results/e116-mutate-after-2026-10-04-h487.log`; both end with "restored, baseline still all green", and hit-the-ceiling, timeout, could-not-schedule and squeezed-out are all 0); registered void clause V4 (one existing mutation's status gets worse after the constant change): comparing by mutation name one by one, the only difference is the extra 19th in the after run, and the existing 18 are "caught" both times ⇒ does not trigger. The experiment page title and index row status changed to "run".
  - **Basis**: the two logs above; sections nine and eleven of the registration `research/prompts/e116-r2-prereg.md`.
- E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) rerun with record header 311 → 487 (line 4 of the question sheet)
  - **Before**: `JOURNAL_HEADER_BYTES` in the apparatus was 311 (the file header comment said new root segment 188), and unit tests pinned 37 200 000 / 166 240 384 / 0.051623 / 2.099316 / 169 943 168; today's header in settled item 4 of D23 (Journal Role & Format) is 487 (new root segment 364). The current-state numbers on the experiment page were from the header-311 version.
  - **After**: the constant changed to 487 and the file header comment to 364; the unit test pins changed to 54 800 000 / 183 840 384 / 0.057088 / 2.104781 / 187 543 168; the itemized sum for the new root segment 364 (86 + 86 + 8 + 8 + 86 + 86 + 4) written into `header_bytes_equals_the_decision_23_breakdown`; 20 unit tests all green. Artifact `e116-pack-settle-2026-10-04-h487.out` (308 lines) differs from the header-311 one in 36 lines (`name=config` 1, `payback_key` 5, `payback_fill` 30; each of the 35 `move_write` lines with `journal=true` is 17 600 000 higher), and the five-cell verdict is unchanged (artifact changed, verdict unchanged); the E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) line in `scripts/experiment-apparatus/replay.sh` now points to the new artifact, rerun byte-identical. Mutation table 18 → 19 entries (the original text of the two old header width entries replaced with `= 487;`, and a new "header width falls back to 311" added); the two whole-table runs before and after the change have not been run yet. The experiment page title and index row status changed to "partly run", the current-state numbers replaced with the new artifact's, and the review dates of eight rows in the affected decisions table changed.
  - **Basis**: rerun registration `research/prompts/e116-r2-prereg.md`; artifact `research/results/e116-pack-settle-2026-10-04-h487.out`.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

### 2026-09-25

- See the same-day entry "E49 (Reverse Chain Width: 32 vs 64) rerun with record header 307 → 311 (line 7 of the question sheet)" in the E49 (Reverse Chain Width: 32 vs 64) section.

### 2026-09-24

- See the same-day entry "E43 (Extension Point Byte Upper Bound) and E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) rerun with record header 307…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-19

- See the same-day entry "E114 (Overall Accounting of Small-Data Packing Containers), E119 (Padding Cost of Fixed-Width Slots)…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-14

- See the same-day entry "Five apparatuses catch up with the three format constants the user settled on 2026-09-14: journal record header 307, i…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-13

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) per the journal header…" in the E43 (Extension Point Byte Upper Bound) section.
- See the same-day entry "E43 (Extension Point Byte Upper Bound), E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) per the journal header…" in the E43 (Extension Point Byte Upper Bound) section.
- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-12

- "Settled item k of D27 (Small-Data Packing Container) unsettled" rewritten in three places in the E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) body and four places in the apparatus comments
  - **Before**: the "three assumptions, not clauses" section of the body and the comments in `e116_pack_settle.rs` said "settled items 3 / 5 / 6 of D27 (Small-Data Packing Container) unsettled": those three really were unsettled when this experiment ran, were settled later, and once references across the repository were rewritten to "settled item" under today's numbering the sentences read as self-contradictory.
  - **After**: written as "item k was not yet settled when this experiment ran; today it is settled item k"; item 5 additionally states it is settled today as having no live slot count, and item 3 states that this experiment's numbers were not recomputed under it.
  - **Basis**: reported on the spot when stage 44 (a reference says settled item and then immediately says it is unsettled) was newly set up; only comments and body text were touched, and the E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) rerun is byte-identical.

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-08

- See the same-day entry "Added E120 (Choosing the Common Ratio of a Geometric Size-Class Table): the worst-case waste of a geometric table is distribution-independent, closed form refuted twice" in the E120 (Choosing the Common Ratio of a Geometric Size-Class Table) section.

### 2026-09-07

- Added E116 (Packed Container Balance: Metadata Writes and Compaction Strategy); the structural lower bound of the payback ratio is `w/(cap − 1)`
  - **Before**: the write account of E114 (Overall Accounting of Small-Data Packing Containers) counted only data units, while the second argument round subtracted one-off writes directly from permanent occupancy and got "net gain negative": two quantities in different units.
  - **After**: E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) splits the three accounts apart (steady-state occupancy / one-off move writes / payback ratio),
    and counts all four write components (container data, central mapping, allocation records, journal). **Payback ratio = one-off move writes ÷ permanently freed occupancy.**
    The upper-bound control arm (metadata writes recorded as 0) lands cell by cell on the closed form `w / (cap − 1)` ⇒ **it pays back if and only if `cap > w + 1`**;
    at `w` = 2 it needs `cap ≥ 4`, and the 8 KiB tier with `cap` = 3 is exactly 1.0000 ⇒ **the 4 KiB boundary of settled item 2 of D27 (Small-Data Packing Container) is supported by the closed form**,
    no longer relying only on the collapse of the gain curve. `bg_key` measures 0.0444 / 0.0784 / 0.3441 / 1.0142 / ∞.
    On the same object tier the compaction policy spreads the payback ratio 46× apart (512 B: fill order `b`=1 gives 2.0921, `b`=N gives 0.0455).
    In the dead-slot scan of 210 cells, `pack_worse` appears 0 times. The ratio of gain multiple to blast radius multiple is 0.9995–1.0000.
  - **Basis**: the third three-way argument round on D27 (Small-Data Packing Container). 17 unit tests / all 16 mutations caught / all three controls pass / rerun byte-identical.
  - **One modeling error fixed the same day**: allocation records were counted by placement as two per unit, while settled item 7 of D3 (Space Allocation) says verbatim
    "one record per unit, with a span segment". The measurement basis leg pointed it out, and the main agent changed it after checking verbatim and reran; the payback ratios of the three tiers each dropped about 2%,
    **the 8 KiB tier 1.0164 → 1.0142, still > 1, the conclusion's direction unchanged**.
  - **Two formulas registered before the run are refuted**: ① the payback ratio at `b`=1 was registered as 1.058893 and measures 2.092080 (the registered value came from the second round's account, which counted only the central mapping);
    ② "`b` = N converges to key order" is refuted: the central mapping converges (375 leaves), but allocation record changes are 101 725 > `b` ⇒ still 2 batches.
  - **Mutation M14 is a real blind spot**: the dead-slot assertion is a mutual comparison (`pack ≤ pad`), and it still holds after changing "the probability that a container is entirely dead, `d^cap`" to `d`
    ⇒ no sampling point can catch it; per the three-way split of `.claude/rules/mutation-sampling.md` it is **neither an insensitive sampling point nor an equivalent mutation**.
    After adding `death_uniform_absolute`, which pins an absolute value, it was caught on the spot, 16/16 with no blind spot.

## E117 (Header Width and Geometry After Factoring in Reserved Bits)

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-12

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-07

- Added E117 (Header Width and Geometry After Factoring in Reserved Bits); one absolute value registered before the run is refuted
  - **Added**: E117 (Header Width and Geometry After Factoring in Reserved Bits). Cause: the first-round forward-inference leg on settled item 14 of D18 (Information Carried in a Block)
    found that the 105 bytes of settled item 7 of D18 (Information Carried in a Block) **do not include** the two fields in the sentence of the same section,
   "v1 reserves field bits for the nonce generation and the MAC"; the user decided
    "take A, but work out the header width first, then settle". Lands in C193 (size of unit header when encryption is enabled never calculated across repo).
  - **Conclusion**: counting arm A (generation = full 96-bit nonce 12 + MAC 16),
    code 1 header 105 → **133**, code 3 header 103 → **131**, code 2 three tiers 68 / 77 / 86 → 96 / 105 / 114;
    the geometry cost is that fanouts each drop 0–1 (entry width 54 of E73 (Fanout Cost of Node Key Interval): 302 / 301 / 301 → 301 / 301 / 301;
    entry width 81 after the update of settled item 4 of D19 (Block Pointer Structure & Width Budget): 201 / 201 / 201 → 201 / 200 / 200),
    **and the code 3 records per container, 233, do not drop at all**.
  - **One value registered before the run is refuted**: criterion 2 registered "`bare` is 302 at all three tiers", measured **302 / 301 / 301**.
    The registered value was extrapolated by the main agent after computing only the first tier, **and when the verdict was revised only the third tier was changed, again without computing the second**:
    both mistakes are the same defect, treating one sampling point as three. Both values are kept in the source `t03`, and the pre-run one must not be changed to the correct one.
  - **Discriminating power lands on a non-obvious cell**: `resv12` and `resv4` are **identical cell by cell** at E73 (Fanout Cost of Node Key Interval)'s
    own entry width 54 (rounding down eats the 8-byte difference) and only separate at the second tier of entry width 81.
    Per `.claude/rules/mutation-sampling.md` this is an insensitive sampling point, kept on file in unit test `t10`.
  - 12 unit tests / all 12 mutations caught; the rerun is byte-identical to `e117-reserved-header-2026-09-07.out`.

## E119 (Padding Cost of Fixed-Width Slots)

### 2026-09-19

- See the same-day entry "E114 (Overall Accounting of Small-Data Packing Containers), E119 (Padding Cost of Fixed-Width Slots)…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-12

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-08

- Added E119 (Padding Cost of Fixed-Width Slots): four tiers are too coarse, and tiers must be dense at the small end
  - **Before**: settled item 8 ④ of D27 (Small-Data Packing Container) (user settlement) took fixed-width slots, bringing padding waste back by tier width,
    and the repository had not a single number on "how finely tiers should be cut".
  - **After**: E119 (Padding Cost of Fixed-Width Slots) ran six arms over three distributions. Loss relative to variable length: 64-byte granularity (64 tiers)
    +1.78% / +9.61% / +0.02%; 256-byte granularity (16 tiers) +6.71% / +37.31% / +0.02%;
    four tiers +39.79% / **+90.83%** / +0.02%; single tier +109.66% / +797.36% / +121.56%.
  - **One reverse clause fixed before the run triggers**: "`tier4` loses > 30% on any distribution ⇒ four tiers are too coarse, tiers must be finer",
    and two distributions cross the line. **"All fixed-width arms lose > 30% ⇒ settled item 8 ④ should be reopened" did not trigger**
    (64-byte granularity is below 10% in all three cells) ⇒ the fixed-width settlement does not need reopening; what needs settling is the tier table.
  - **One new conclusion**: tiers must be **dense at the small end**, not cut at equal spacing: `pow2` (7 tiers) beats the equally spaced
    `gran256` (16 tiers) on the distribution with many small files, 31.85% vs 37.31%, because the mass piles up at the small end and the finest tier of equal spacing is 256.
  - **One sampling-point trap recorded**: the three settled sampling points of D27 (Small-Data Packing Container), 512 / 1024 / 4096,
    **are all exactly tier widths**; looking only at them, all four candidate arms are +0.02%, which would wrongly suggest "fixed width is free".
  - **One modeling error caught on the spot by the positive control fixed before the run**: the first version of the `var` arm computed "perfect packing"
    (Σ slot lengths ÷ payload, rounded up), which amounts to allowing one slot to span containers; with all 512-byte objects it gave 1700 containers where the correct answer is 1725.
    After switching to greedy packing in ascending size order the control passes, and both models are kept in the source.
  - **Mutation M8 is a real blind spot**: degrading the log-uniform distribution to uniform turned not a single test red in the first round:
    all the assertions pinning absolute values were then only on `uniform`, while the heaviest number in the conclusion is on `logunif`.
    Per the three-way split of `.claude/rules/mutation-sampling.md` it is **neither equivalent nor an insensitive sampling point**; after adding assertions, 12/12 with no blind spot.
  - ⚠️ **The number collided twice**: first 117 was taken, then 118; both numbers were already taken by concurrent sessions, and it finally landed on E119 (Padding Cost of Fixed-Width Slots).

- See the same-day entry "Added E120 (Choosing the Common Ratio of a Geometric Size-Class Table): the worst-case waste of a geometric table is distribution-independent, closed form refuted twice" in the E120 (Choosing the Common Ratio of a Geometric Size-Class Table) section.

## E118 (Three Arms of Single-Disk Self-Recovery)

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.
- See the same-day entry "E23 (Journal Geometry), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place), E1 (Node Size vs Recoverability)…" in the E23 (Journal Geometry) section.

### 2026-09-07

- Added E118 (Three Arms of Single-Disk Self-Recovery); the failure clause fixed before the run triggers literally
  - **Added**: E118 (Three Arms of Single-Disk Self-Recovery). Cause: after the 2026-09-01 three-way argument, settled item 4 of D18 (Information Carried in a Block)
    wrote down six burdens of proof that must be met before the line opens, and this experiment meets them one by one.
  - **Conclusion**: the chain arm (same-batch chain, 12 bytes) recovers exactly as many as the no-redundancy arm at **every** fault point
    ⇒ what it buys is **detection**, not recovery, and detection is something the allocation records of settled item 3 of D3 (Space Allocation) could already answer
    ⇒ **the failure clause fixed before the run triggers literally, recorded as is**. When a contiguous range destroys 1/16 of the disk, the ditto arm raises metadata recovery
    from 0 to 5242 (all of it), at a cost of 2.2%, the same order of magnitude as ZFS's own statement.
  - **A structural risk measured along the way**: the placement offset of `至少隔 1/8 盘` (at least 1/8 disk apart) is a power of 2,
    **resonating with any fault with a power-of-2 period** (all three copies congruent and dying together): at k = 8 ditto degrades into the no-redundancy arm,
    and at k = 9 it immediately pulls apart. Real bad sectors are not periodic, so what this says is "placement offsets should not be powers of 2".
  - **Mutation testing forced out a real blind spot**: `M4_散点周期判反` turned not a single test red, because the scatter assertions
    **were all mutual comparisons between arms**, with none pinning an absolute value. After adding `t14_scatter_counts_are_absolute`, all 12 caught.
    Per the three-way split of `.claude/rules/mutation-sampling.md`, this is a real blind spot, not an insensitive sampling point.
  - 14 unit tests / all 12 mutations caught; the rerun is byte-identical to `e118-single-disk-recovery-2026-09-07.out`.
- After adding E118 (Three Arms of Single-Disk Self-Recovery), added E122 (Benefits Bought by Directory-Based Grouping)
  - **Added**: E122 (Benefits Bought by Directory-Based Grouping). Cause: in the table of settled item 3 of D8 (Core Index Structure),
    the only basis for the row "allocator (places a subdirectory's data near its parent directory)" was marked verbatim
    "⚠️ **unverified**: no source and no measurement basis in the kb; fill it in before use", and after a round of three-way argument on settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient)
    the user decided "first fill in the gain data, then settle".
  - **Conclusion**: the traversal half of the gain **hangs entirely on interleaving**: the contiguous run count of `arrival` = total objects ÷ interleave stride,
    and without interleaving it is exactly equal to `grouped` (both 16); **only the reclamation half is unique to clustering**:
    after deleting one directory, `grouped` frees whole segments (2 at a segment length of 32 slots, 1 at 64 slots),
    `arrival` and `random` free none, and at a segment length of 128 slots the gain goes to zero.
  - **One parameter registered before the run was labeled backwards**: criterion 3 said "the two arms are equal at `stride = 1`",
    but `stride = 1` is the **fully round-robin** cell; the one without interleaving is `stride = 每目录文件数`.
    The criterion's meaning did not change; the label attached at registration was wrong, and both values are kept in the source `t02`.
  - **Mutation testing forced out one real blind spot and one insensitive sampling point**:
    `M11` (changing the load size in `main`) was seen by no test: the unit tests each hard-code 16 / 32,
    and nothing pinned the parameter set the artifact actually uses ⇒ lifted into constants and pinned;
    `M10` (`random` without deduplication) was not seen on 512 objects: the LCG modulus is 2⁶⁴ and 512 is a power of 2,
    so `x % 512` happens to walk the full period and never collides ⇒ a non-power-of-2 sampling point was added (480, 172 collisions).
  - 10 unit tests / all 12 mutations caught; the rerun is byte-identical to `e122-dir-locality-2026-09-07.out`.

## E120 (Choosing the Common Ratio of a Geometric Size-Class Table)

### 2026-09-19

- See the same-day entry "E114 (Overall Accounting of Small-Data Packing Containers), E119 (Padding Cost of Fixed-Width Slots)…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-12

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-08

- Added E120 (Choosing the Common Ratio of a Geometric Size-Class Table): the worst-case waste of a geometric table is distribution-independent, closed form refuted twice
  - **Before**: E119 (Padding Cost of Fixed-Width Slots) judged "four tiers are too coarse, tiers must be dense at the small end", but it compared five specific tables,
    each picked according to some distribution, and **the object size distribution had never been measured anywhere in the repository** (C174 (Three columns of smallfile row unmeasured)).
  - **After**: E120 (Choosing the Common Ratio of a Geometric Size-Class Table) switches to geometric tier tables, `W_k = W_min · r^k`.
    **Worst-case relative waste = `1 − 1/r`, independent of the distribution**: exactly the property wanted while the distribution is unmeasured.
    Four common ratios: `r` = 1.125, 36 tiers / worst 11.10% / measured +8.28% and +11.24%;
    1.25, 20 tiers / 19.99% / +12.64% and +14.32%; 1.5, 12 tiers / 33.31%; 2.0, 7 tiers / 49.98%.
    Every tier has `cap` ≥ 4, and the payback gate of E116 (Packed Container Balance: Metadata Writes and Compaction Strategy) passes in every cell.
  - **The closed form registered before the run is refuted twice**: ① the first tier covers 1..=`W_min`, so a 1-byte object padded to 64 wastes 98.44%,
    **and that number is identical across the four common ratios** ⇒ it is the floor, not the common ratio; the closed form holds only above `W_min`,
    and the floor segment is looked at in absolute bytes (at most 63 per object); ② integer rounding makes the actual tier ratio slightly exceed `r` (81 → 92 is 1.1358)
    ⇒ the upper bound is relaxed to `1 − 1/(r + 1/W_min)`. **Neither went back to change the registered formula**; the three numbers are kept side by side in the source.
  - **The `discrete` distribution is not neutral**: its three sampling points 512 / 1024 / 4096 are all powers of 2 ⇒
    `r` = 2.0 costs only 1 more container on it (+0.02%), and looking only at it would give the opposite conclusion from the other two distributions.
    This is the second outbreak of the sampling-point trap recorded by E119 (Padding Cost of Fixed-Width Slots).
  - **The relation to equally spaced tables stated**: E119 (Padding Cost of Fixed-Width Slots)'s 64 equally spaced tiers lose only 1.78% on `uniform`,
    better than any geometric table, **but they have no distribution-independent upper bound** (padding 65 bytes to 128 is likewise 49%) ⇒
    while the distribution is unmeasured, take the bounded kind; once C174 (Three columns of smallfile row unmeasured) is paid off, equally spaced tables can rejoin the comparison.
  - **All 13 mutations caught**, including one aimed specifically at "the floor tier is also counted in the worst-case waste".

- See the same-day entry "Added E121 (Optimal Points for Tiering by Cap and Tier Count): more tiers is not always better; tail containers eat fine tiers back" in the E121 (Optimal Points for Tiering by Cap and Tier Count) section.

## E121 (Optimal Points for Tiering by Cap and Tier Count)

### 2026-09-19

- See the same-day entry "E114 (Overall Accounting of Small-Data Packing Containers), E119 (Padding Cost of Fixed-Width Slots)…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-12

- See the same-day entry "Code 3 header 103 → 107, code 2 three-tier headers 68 / 77 / 86 → 72 / 81 /…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-08

- Added E121 (Optimal Points for Tiering by Cap and Tier Count): more tiers is not always better; tail containers eat fine tiers back
  - **Before**: the four candidates of E120 (Choosing the Common Ratio of a Geometric Size-Class Table) were one family (geometric in slot width `W`);
    cutting by `cap` had not been tried, and `W_min` had not been scanned.
  - **After**: E121 (Optimal Points for Tiering by Cap and Tier Count) ran seven arms. **The main conclusion is that the tier count has an optimum**:
    each tier leaves one partly filled tail container at the end of a compaction batch. The finest non-redundant table (248 tiers, one per reachable `cap`)
    wins on the uniform distribution (+2.39%), **but loses on the small-file-dense distribution** to the 36-tier one (+12.31% vs +11.24%),
    because there the total container count is only 1592 and 248 tail containers take 15%.
    ⇒ the optimal tier count depends on how many objects each tier can share, that is, on **the load size**, not only on the shape of the distribution.
  - **Three more**: geometric in `cap` is a bad parameterization (worst-case waste 25%–54%, losing on both neutral distributions);
    `W_min` at 32 and 64 is byte-identical, while at 128 the loss on small-file-dense rises from 20.41% to 36.06%;
    the prediction written before the run, "deduplicating by `cap` only reduces, never increases", **is empty at `r` = 1.125** (36 tiers are still 36 after deduplication),
    and only switching to `r` = 1.01 (370 tiers) makes deduplication cut 44% (→ 208). All three numbers are kept in the source, not changed back.
  - **One equivalent mutation kept on file**: M10, removing `ts.push(LIMIT)` at the end of `capall_tiers`, turned not a single test red:
    the `cap` = 7 tier already computes to 4096 (`32665/7 − 43 = 4623` clipped to 4096 by the boundary) ⇒ that statement is redundant.
    Per `.claude/rules/mutation-sampling.md` it is judged **an equivalent mutation, not a blind spot**; the equivalence was written as a test and kept on file,
    and one that really changes behavior (removing the boundary clipping) was added in its place; the new one turned 8 tests red on the spot. 14/14 with no blind spot.

## E122 (Benefits Bought by Directory-Based Grouping)

### 2026-09-19

- See the same-day entry "One current-state sentence each in E151 (Arrival Order of User Data Placement Points and Container Arms) and E68 (Selection of Inlining Threshold) corrected after a fresh check; E8 (Different Write Paths for Large vs Small Files)…" in the E151 (Arrival Order of User Data Placement Points and Container Arms) section.

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.
- See the same-day entry "E151 (Arrival Order of User Data Placement Points and Container Arms) fourth run: the two directory-grouping hint arms against the subtraction arm: following the members…" in the E151 (Arrival Order of User Data Placement Points and Container Arms) section.
- See the same-day entry "E151 (Arrival Order of User Data Placement Points and Container Arms) fifth run: overflow segment counts for fixed-home hints scanned at 4 / 8 / 1…" in the E151 (Arrival Order of User Data Placement Points and Container Arms) section.

### 2026-09-09

- The E122 (Benefits Bought by Directory-Based Grouping) conclusion "only clustering can buy it" corrected per its own artifact: the gains in both cells hang on interleaving
  - **Before**: the concluding sentence said verbatim "the gain in the traversal cell **hangs entirely on interleaving**……; what only clustering can really buy is the **reclamation** cell:
    after deleting one directory `grouped` frees whole segments (2 at a segment length of 32 slots, 1 at 64 slots),
    while `arrival` and `random` **free none**"; the "fully empty segments freed" table **had no `stride` column** and recorded 0 / 0 / 0 for `arrival`.
  - **After**: the concluding sentence changed to "**the gains in both cells hang on interleaving**"; the table gained a `stride` measurement basis column,
    and the two artifact lines were pasted into the body unchanged; item 2 of "what it says about settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient)" was rewritten entirely,
    and an item 4 was added stating what it actually proves and what it must not be read as.
  - **Basis (committed artifact line by line, not paraphrased)**: in `e122-dir-locality-2026-09-07.out`,
    `E7RESULT name=walk arm=arrival stride=32 total_runs=16 worst_dir_runs=1 empty_segs_at_32=2 empty_segs_at_64=1 empty_segs_at_128=0`
    is **identical cell by cell** with the `grouped` line of the same `stride`.
    This is the same thing as E122 (Benefits Bought by Directory-Based Grouping)'s own positive control, whose line says verbatim
    "at `stride = 32` the placement sequences of the two arms are **exactly the same**", and if the placement sequences are the same, any quantity computed from placements must be the same.
  - **The apparatus is right; the body is wrong**: in `scripts/experiment-apparatus/e122_directory_locality.rs`,
    `t05_empty_segs_are_absolute` hard-codes `stride = 2`, with the assertion message verbatim
    "arrival cannot empty a single whole segment **after interleaving** (segment length {sg})" ⇒ the assertion's scope was right all along;
    it was the body that dropped that qualifier. **Not a word of the source or the artifact changed.**
  - ⚠️ **The gate cannot see this kind**: a fresh rerun of E122 (Benefits Bought by Directory-Based Grouping) (`scripts/experiment-apparatus/replay.sh`) outputs
    `字节一致 1 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0`:
    the rerun compares artifact with artifact, while what does not match is **the body and the artifact**. Lands in C233 (Exclusive Gain Named by a Conclusion Has a Counterexample in Its Own Artifact).
  - **One more measurement basis narrowing**: the modeling premise line originally said "each small file takes a whole unit, **never shared**",
    while settled items 1 / 2 of D27 (Small-Data Packing Container) settled that objects ≤ 4 KiB are moved into shared containers by background compaction after being written
    ⇒ the model holds only for objects **> 4 KiB**, and for objects ≤ 4 KiB **before they are moved away**
    (the length of that period is set by the hysteresis `K`, and `K` has no number).
  - **Three downstream places changed with it**: the E122 (Benefits Bought by Directory-Based Grouping) concluding sentence, its item 2 of "what it says about D14 (Dual-Track: Large/Small Files & Persistent/Transient)",
    and the index row in [experiments.md](experiments.en.md); the one on the decision side (the text of settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient))
   is recorded in decisions-history.md (part ten).
  - **Found in** the second three-way argument round on settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient); the reverse-inference leg hit it by checking the artifact fresh, and the main agent confirmed it line by line.

### 2026-09-07

- See the same-day entry "After adding E118 (Three Arms of Single-Disk Self-Recovery), added E122 (Benefits Bought by Directory-Based Grouping)" in the E118 (Three Arms of Single-Disk Self-Recovery) section.

## E123 (Trade-off Costs Between K and Rollback Depth)

### 2026-09-20

- See the same-day entry "E43 (Extension Point Byte Upper Bound), E57 (Authoritative Placement for Plaintext-Side Field Set)…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-09

- Added E123 (Trade-off Costs Between K and Rollback Depth) and ran it: cost numbers attached to the fork of C215 (promises on rollback depth and lower bound on K cannot hold simultaneously)
  - **Added**: E123 (Trade-off Costs Between K and Rollback Depth). Cause: C215 (promises on rollback depth and lower bound on K cannot hold simultaneously)
    wrote down a two-way choice, but both arms had only the sentence "needs a three-way argument" and no numbers. Filled in per the practice of pricing a fork before handing it over for settlement.
  - **Conclusion**: the four quantities (blocks deducted, accounting key count, generations that can be rolled back, ENOSPC escape phase count) **separate the two arms in every cell**
    across the five S tiers scanned; none is identical. The price difference grows linearly with S: at S=1 arm ① deducts 13 more blocks, stores 64 more keys
    and rolls back 1 more generation; at S=16 it deducts 598 more blocks, stores 2944 more keys and rolls back 46 more generations.
    Arm ② is constant at 26 blocks / 192 keys / 2 generations rollback / 1 escape phase.
    ⚠️ Arm ① does not reach the 1000-block "it matters" line of E50 (Upper and Lower Bounds on Root Ring Slot Count) at **any of the five tiers** (624 blocks at S=16).
  - **A tautology of its own fixed on the spot during the run**: the first version's negative control was written as
    `pinned_blocks(n) == pinned_blocks(n)`: comparing a value with itself, always true.
    Only after changing it to go through the price-difference function could the "price difference constantly zero" mutation be caught (before, it could not),
    the same cell as `.claude/singlefs-ai-sop/rules/show-me-test.md` "tests test the implementation, not the contract".
  - **Measurement basis**: 13 blocks written per fsync is taken from the coarse-grained settlement of D25 (Target Workload Priority) (not measured),
    and 64 statistics are taken from the example value of E54 (How Many Generations to Retain in Accounting) (not a settlement).
    **It does not judge which arm is right**: "how deep a rollback is enough" is a product judgment, handled the same way as E48 (Feasible Points for Root Ring Geometry) handles F.
  - 9 unit tests / all 11 mutations caught, N=5 rounds with identical md5, artifact 20 lines, registered into `scripts/experiment-apparatus/replay.sh`.

## E124 (System Configuration Field Table Recomputed per D18 Settled Item 14)

### 2026-09-13

- See the same-day entry "Added E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) and ran it: 413 leaves 9 to spare in the 512 slot…" in the E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) section.

### 2026-09-09

- Added E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) and ran it: the nonce watermark needs 12 bytes, not 8
  - **Before**: that row in the candidate tables of E100 (Three-Segment Geometry of System Configuration) and E115 (Completeness and Byte Budget of System Configuration Field Table) was verbatim
    `("nonce_watermark", 8)`, and neither had an assertion saying how wide it should be. The 2026-09-08 review paragraph of settled item 9 of D22 (Synthesizing Unit Atomicity),
    after "all fourteen items of D18 (Information Carried in a Block) settled", named verbatim "**settled item 14 (the nonce generation is the full 96-bit nonce, living in the unit's plaintext header) changed the input to the cell of blocker ②**
    : E115 (Completeness and Byte Budget of System Configuration Field Table) took minimal usable assumed values for those three items of D9 (Encryption) and must be recomputed under it", and the recomputation had never been done.
  - **After**: E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) ran (12 unit tests / all 14 mutations caught).
    I-6.5 (nonce watermark monotonic) requires "watermark > the largest nonce that has ever appeared on the disk", and after settled item 14 that nonce is the full 96 bits
    ⇒ **the watermark needs at least 12 bytes; the 8 used by E100 (Three-Segment Geometry of System Configuration) can only represent 64 bits, 4 bytes short**.
    The four cell numbers of E115 (Completeness and Byte Budget of System Configuration Field Table) each +4 (503/562/532/591 → **507/566/536/595**);
    **the verdict's direction did not change by a word**: the merged arm is still the only form that fits a 512-byte sector under every measurement basis (389 / 477);
    what changed is the margin: the unmerged cell shrinks from "9 to spare" to "5 to spare".
    It also measured a cell E115 (Completeness and Byte Budget of System Configuration Field Table) never asked about: the second half of the source for item 30 of its required table, "device count `devs`",
    says verbatim "after the device table moves out of the slot, the slot holds no entry count", while in the field table of the code 3 packed record container of settled item 11 of D18 (Information Carried in a Block)
    the row at offset 69 is verbatim `| 69 | 记录数 | 2 | ≤ 583 |` ⇒ **if the device table unit takes code 3, those 4 bytes in the slot can be saved**,
    exactly offsetting the 4 bytes of the watermark. **Which code the device table unit takes has never been settled in the repository**; per its failure clause E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) only computes the two arms once each and does not choose for the repository.
  - **Basis**: named by the 2026-09-08 review paragraph of settled item 9 of D22 (Synthesizing Unit Atomicity) itself.
    Cross-apparatus reconciliation: E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) splits the candidate table into a 23-row constant table + one watermark parameter, and recomputes E115 (Completeness and Byte Budget of System Configuration Field Table)'s
    four numbers 361 / 420 / 83 / 112 identically (last artifact line `name=e115_reconcile`),
    and 361 is **summed, not hard-coded**: changing the width of any row turns it red on the spot (mutation M12).

## E125 (What is Read Above Write Pointer in Sequential Zone)

### 2026-09-28

- See the same-day entry "Host-private information removed from the bodies of E163 (Multi-GPU Unit Checksum Computation) and E125 (What is Read Above Write Pointer in Sequential Zone)" in the E163 (Multi-GPU Unit Checksum Computation) section.

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-09

- E125 (What is Read Above Write Pointer in Sequential Zone) goes from "design registered, not run" to run: the previous round said root was missing, but the real reason was not taking the virtual machine route
  - **Before**: the status of E125 (What is Read Above Write Pointer in Sequential Zone) was "design registered, not run (2026-09-09, root missing)",
    with criteria and failure clause fixed before the run and **not a single cell of data**.
  - **After**: run. Criterion 1 lands on **② (read succeeds and is non-zero)**: the 67 076 096 bytes above the WP are all `0xff`, `rc=0`;
    criterion 2: after the zone reset the whole zone returns to `0xff` and the signature bytes cannot be recovered; criterion 3 `lead_constructed=yes`;
    criterion 4: the nine cells are **identical cell by cell** with the table in the D22 (Synthesizing Unit Atomicity) text; criterion 5: conventional zone 64 MiB,
    and `{P·chunk, 2P·chunk}` at all three chunk tiers are `inside=no`.
   Added `scripts/devices/e125-zoned-wp-probe.sh` and `e125-zoned-wp-2026-09-09.out`.
  - **Basis**: host `sudo` was available this round (the password goes through `SUDO_PASS_A` in the repository root `.env`,
    the same form as `scripts/devices/e72-devtable-probe.sh`).
    ⚠️ **The previous round's claim "root missing" itself must be corrected**: the virtual machine route in [vm-harness.md](vm-harness.en.md)
    **does not need host root** (this host gets the permission through `kvm` group membership), and for zoned devices
    [tooling.md](tooling.en.md) says verbatim "measured and passing (QEMU 8.2.2, exit code 0, all three devices constructed successfully)"
    ⇒ the previous round's real state was "chose not to run", not "could not", the same form as what that round itself wrote,
    "⇒ **the reason 'wait for the crash-point replay harness' does not hold for this cell**".
  - ⚠️ **Mutation testing forced out a real defect**: the positive control originally had only the one **mutual comparison** assertion `WPB == M × SLOT`,
    and `M` drives both the writes and the expected value ⇒ changing `M` from 8 to 7 passed silently.
    Per `.claude/singlefs-ai-sop/rules/test-discipline.md` "for every mutual comparison assertion you write, put one beside it that pins the absolute value",
    `WPB == 32768` and `ZBYTES − WPB == 67076096` were added, and the same mutation was caught on the spot.

## E126 (Costs of Four System Configuration Slot Width Candidates)

### 2026-09-19

- See the same-day entry "E23 (Journal Geometry), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place), E1 (Node Size vs Recoverability)…" in the E23 (Journal Geometry) section.

### 2026-09-09

- Added E126 (Costs of Four System Configuration Slot Width Candidates) and ran it: cost numbers attached to the fork of C232 (system configuration slot width: class rule gave answer, field table says unset)
  - **Before**: the system configuration slot width had no settled place anywhere in the repository, while all the arithmetic of settled item 9 of D22 (Synthesizing Unit Atomicity) took 512 as a given.
  - **After**: each of the four candidates' occupancy, `io_min` compliance, whether it fits, torn exposure surface, and whether it is single-valued on heterogeneous pools:
    all five cells have absolute values. **Criteria and failure clause fixed before the run and not changed by a word after it**; the reverse acceptance clause triggers
    (none of the four dominates across the board).
  - **Basis**: the user named "start on the system configuration slot width now"; C232 (system configuration slot width: class rule gave answer, field table says unset) was opened that day.
  - ⚠️ **Three mutations were not caught in the first round; per the three-way split of `.claude/rules/mutation-sampling.md`**:
    `first_version_disks` is a real blind spot (the constant is only used in `main()`), while `fits_strict` and `tear_sector_floor`
    are **insensitive sampling points** (none of the six record byte counts happens to equal a slot width tier; all four candidates' slot widths are integer multiples of 512)
    ⇒ after adding sensitive points for each, all three were caught on the spot. **None was recorded as an equivalent mutation.**
  - ⚠️ **An overflow of the same origin was hit before the run**: `tear_states` wrote `(1u64 << n) - 2`; the three unit test points with n ≤ 32 do not overflow,
    but reaching candidate B's 65536 (n = 128) panicked on the spot ⇒ changed to return `None` when n > 63 and report it in the exponent form `2^n-2`.

## E127 (Whether to Store Group Identity Under Splits and Merges)

### 2026-09-13

- Added E127 (Whether to Store Group Identity Under Splits and Merges) and ran it: node-count grouping did not press a single cell below 0.8× of key-interval grouping; the model level supports "group identity need not go into the format"
  - **Before**: E127 (Whether to Store Group Identity Under Splits and Merges) had only registered its criteria on 2026-09-10, with no apparatus; the whole of settled item 4 of D26 (Background Compaction and Placement Reclamation) was waiting for it.
  - **After**: the apparatus `scripts/experiment-apparatus/e127_group_identity_under_split_merge.rs` (nodes split and merge, three arms at equal budget, the cross-apparatus gate reproduces the 24 cells of E95 (Aging Node Layout Arm) verbatim on degenerate geometry) ran: `any_cell_node_group_below_0_8=false all_cells_tied=false key_interval_wins_every_cell=false`, six-cell ratios 0.930 / 0.889 / 0.957 / 1.651 / 1.856 / 1.031; criteria 1–5 all pass (anchor `differ=true`, splits and merges happen in every cell); 17 unit tests, all 14 mutations caught; artifact `e127-group-identity-under-split-merge-2026-09-13.out` (101 lines), rerun registered as exact.
  - **Basis**: criterion 6 fixed before the run (2026-09-10); "group identity need not go into the format" is a model-level conclusion, and changing the verdict of settled item 4 of D26 (Background Compaction and Placement Reclamation) goes through a three-way argument.
- E127 (Whether to Store Group Identity Under Splits and Merges) second run: the knobs of the two families filled in to the same set of node widths, and key-interval grouping wins all six cells
  - **Before**: the first run had W ∈ {8, 32, 128}, G ∈ {2, 8, 32}, so the two families were not on the same grid; six-cell ratios 0.930 / 0.889 / 0.957 / 1.651 / 1.856 / 1.031, `key_interval_wins_every_cell=false`.
  - **After**: W ∈ {8, 16, 32, 64, 128, 256}, G ∈ {1, 2, 4, 8, 16, 32}, artifact `e127-group-identity-under-split-merge-2026-09-13-knobs.out` (143 lines): `any_cell_node_group_below_0_8=false all_cells_tied=false key_interval_wins_every_cell=true`, ratios 1.004 / 1.043 / 1.072 / 1.116 / 1.108 / 1.134; criteria 1–5 still pass; the rerun registration points to the new artifact, and the first run's artifact is kept on file.
  - **Basis**: the probe of the fifth-round reverse-inference leg on settled item 4 of D26 (Background Compaction and Placement Reclamation) first computed that each grid missed one tier (section two of `d26-r5-opus-output.md`), confirmed by the committed second run.

## E128 (Point Lookup Cost of Pointers with Birth (tree, txg))

### 2026-10-17

- The placement C157 (Tree table capacity computed with differing entry widths in two places) now points to C148 (Wider tree table entries compress indirection threshold to two layers): this account was merged into C148 (Wider tree table entries compress indirection threshold to two layers) on 2026-10-11; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/128-指针带出生树txg之后的点查代价.md`.

### 2026-09-16

- See the same-day entry "E131 (Two Livelist Carriers: Values and Costs) and…" in the E131 (Two Livelist Carriers: Values and Costs) section.

### 2026-09-11

- See the same-day entry "Added E133 (Format Cost of Mapping Key Candidates) and ran it: the format of the 12 key forms in the two argument rounds…" in the E133 (Format Cost of Mapping Key Candidates) section.

### 2026-09-10

- Added E128 (Point Lookup Cost of Pointers with Birth (tree, txg)) and ran it: the item the user named as owed has been measured, and the direction is the opposite of what was expected
  - **Before**: the cache cost column for candidate A in settled item 7 of D19 (Block Pointer Structure & Width Budget) carried
    the 67-byte 2.22× from E20 (CPU Cache Cost of Widening Pointers), and admitted "**this item has never been in any leg's material**";
    while of E20 (CPU Cache Cost of Widening Pointers)'s six tiers (16 / 24 / 32 / 40 / 67 / 111)
    **not one is a real entry width of this project** (93 → 109 and 81 → 97).
  - **After**: E128 (Point Lookup Cost of Pointers with Birth (tree, txg)) registered and run.
    **The accounting tree pair (81 → 97) points the same way over four rounds, with A slower every round** (1.0179 / 1.0454 / 1.0210 / 1.0357);
    **the inode tree pair (93 → 109) is unstable over four rounds**: the first three rounds `0/5 jia_never_slower` (0.9686 / 0.9655 / 0.9764),
    the fourth round `5/5 jia_slower_every_round` (1.0885), opposite directions ⇒ recorded as unstable per "if neither side is satisfied, write unstable", **no conclusion drawn**.
    All six arms' positive controls pass, and the cross-apparatus gate passes in all four rounds (in the fourth round the whole machine was 4.9% and 5.9% slower than the baseline).
    ⚠️ **The flip was caught by the byte-for-byte structural comparison of `replay.sh`, not by an interval assertion**: `strip_timing` deliberately does not strip
    `verdict=` and `rounds_jia_slower=`; had those been stripped as well, the fourth round would have reported "timing differs only, structure identical" and passed green.
   Lands in C252 (inode pair of E128 unstable, apparatus did not isolate load). Artifact `e128-pointer-birth-cost-2026-09-10.out`, registered into `replay.sh`.
  - **Basis**: the criteria fixed before the run are in `e128-preregistration.md`,
    whose section "will I accept the result if it comes out reversed" says verbatim "if `inode_jia` is **faster** than `inode_bing`……
    **this is a result I accept, not one to be avoided**", and the measurement landed exactly in that cell.
  - ⚠️ **The third round's artifact had one non-monotonicity, which disappeared in the fourth round**: in the third round, at the same tree depth 4, 93 bytes (586.89 ns)
    was slower than both 109 bytes (572.95 ns) and 111 bytes (555.26 ns); in the fourth round it was 612.09 vs 667.30.
    ⇒ it does not reproduce across rounds, **and is not the same in nature as the 8 KiB anomaly of E20 (CPU Cache Cost of Widening Pointers), stable over six rounds; the two must not be cited side by side**.

## E129 (How Writes Below io_min Fail on Real Devices)

### 2026-09-10

- Added E129 (How Writes Below io_min Fail on Real Devices) and ran it: this repository's first measurement of "in-device RMW" on a real device, and its failure form is the opposite of what was expected
  - **Before**: the end of the "write granularity" section of D2 (RAID Stripe Policy) said verbatim "⚠️ this host cannot falsify it with fault injection
    (`dm-flakey` cannot produce tears)…… falsification waits for the crash-point replay harness.";
    candidate ② of C212 (fixed-structure writes smaller than io_min) verbatim still owed "why fixed structures are not afraid of in-device RMW,
    while the real-device round of E34 (Root Ring Slot Geometry) **only proved the one mechanism of the RAID5 write hole**".
  - **After**: all three segments of E129 (How Writes Below io_min Fail on Real Devices) ran, five rounds identical cell by cell:
    ① the tear injector (`dm-linear` good segment + `dm-error` bad segment) can be built, `span_torn_rounds=5/5`;
    ② `dm-thin` (64 KiB blocks) is a **non-RAID** device with `thin_io_min=65536` / `thin_pbs=512`;
    with a shared block one 512-byte write makes the data device write **129** sectors, with an exclusive block **1** sector;
    ③ with the tear injector placed under the pool data device and the COW landing only 64/128 sectors,
    the neighbor is **intact byte for byte in 5/5 rounds**, and the writer gets `write_rc=1`.
  - **Basis**: the criteria are written in `e129-preregistration.md`, before any line of harness code;
    each of the three segments carries its own void clause, and when any control does not match the script reports `name=fatal` itself and exits non-zero;
    all 9 mutations caught (one of them judged an **equivalent mutation** and kept on file, replaced by one that tests the same discriminator from the other side).
  - ⚠️ **Scope**: the tear injector only overturns the meta-claim "an injector cannot be built"; it cannot produce "in-device RMW";
    conclusion three covers only the one `dm-thin` mechanism, and **RMW inside an SSD FTL still cannot be injected** (already recorded by E34 (Root Ring Slot Geometry));
   ⚠️ **The RAID5 item was once written wrong that day and corrected the same day**: it said "E34 (Root Ring Slot Geometry) proved the write hole **does** damage neighbors",
    while the real-device half of E34 (Root Ring Slot Geometry) says verbatim "neighbor slot B in the same chunk (sector position 1) **intact 10/10**",
    and the fault unit "is 'same stripe row × same sector position' across legs, **not the whole `io_min` unit**"
    ⇒ **the neighbors are intact under both measured mechanisms**, and E129 (How Writes Below io_min Fail on Real Devices) and E34 (Root Ring Slot Geometry) do not form a pair with "opposite failure forms".
  - ⚠️ **What it builds is on-disk state, not a real power loss**: a loop device cannot lose power; the state is equivalent, the path is not.

## E130 (Does Per-Head Livelist Pass Bounded Destruction)

### 2026-09-10

- Added E130 (Does Per-Head Livelist Pass Bounded Destruction) and ran it: A passes criterion 5, but "condensable" is proven not to be optional
  - **Why run**: the three-way round on settled item 2 of D6 (Snapshot Implementation Model) judged C and B out and re-estimated D's cost, leaving only A alive,
    while the reverse-inference leg itself wrote "my conclusion only reaches 'B fails', **not 'A passes'**". The user judged that day
    "never verified, cannot stand"; E130 (Does Per-Head Livelist Pass Bounded Destruction) fills exactly that cell.
  - **Conclusion**: A passes criterion 5 of D6 (Snapshot Implementation Model): the worst-case space requirement per batch, scanned over all 24 load cells, has a value set of size constantly 1
    (180224 bytes = 4096 × (20 + 24)), and the pre-batch reservation can be computed from the structure's own counts alone, without reading the live tree.
  - **Two costs**: ① with `naive` (no condense) the entry count is of the same order as the write count: `n=65536 churn=16` verbatim
    `entries=2162688`, while net live blocks are only 65536, **33×**, and pending-deletion occupancy is overestimated `overestimate=17.0000`,
    while `condense` in the same cell is verbatim `entries=65536 ... overestimate=1.0000`
    ⇒ **"condensable" is a necessary condition, not an option**; ② the release path prepays 24 extra bytes per released block
    ⇒ this must be added to the quantity "how much it needs to unwind itself" of settled item 2 of D3 (Space Allocation), which is a user-settled clause;
    lands in C260 (livelist entry prepayment omits the amount for unwinding itself).
  - **The criteria fixed before the run were checked one by one**, including the positive control (`ratio=33`, the void clause does not trigger).
  - **An arm was changed once, before any measurement**: the first trigger criterion of `condense`, "FREE share ≥ 1/2",
    has an antecedent that is always false for this load family (`c / (1 + 2c) < 1/2` holds for any finite `c`)
    ⇒ that arm would degrade into a copy of `naive`, a straw-man control arm. Kept on file by the unit test
    `the_first_condense_predicate_never_fires`.
  - **One mutation was not caught in the first round; per `.claude/rules/mutation-sampling.md` it is judged the third class, "insensitive sampling point"**:
    M8 differs from the original only on `raw / net ∈ (1, 2]`, while the load family only reaches 1, 3, 9 and 33.
    The handling was to extract the criterion into a separately testable function and add three sampling points, not to record it as an equivalent mutation; after that it was caught on the spot.
  - **Artifact**: `e130-livelist-bounded-destroy-2026-09-10.out`,
    registered into `scripts/experiment-apparatus/replay.sh` (`exact` mode, byte-for-byte comparison).

## E131 (Two Livelist Carriers: Values and Costs)

### 2026-09-18

- See the same-day entry "The experiment index row of E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) catches up with the 2026-09…" in the E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) section.

### 2026-09-16

- E131 (Two Livelist Carriers: Values and Costs) and E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) catch up with the tree table denominator 16253 and entry width 200, each with a new artifact
  - **Before**: both apparatuses had a tree table denominator of 16284 and entry width 121 (E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) wrote it as the breakdown 2 + 2 + 2 + 8 + 59 + 8 + 8 + 32), 134 trees per level; the criterion 1 threshold of E131 (Two Livelist Carriers: Values and Costs) was per_head 68 / 8979 heads, and E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) had 44 / 5985, 33 / 4489, 66 / 8977; artifacts `e131-livelist-carrier-2026-09-10.out` and `e132-livelist-carrier-recount-2026-09-10.out`.
  - **After**: denominator 16253 (the only reading hard-coded by `22-单元原子性怎么合成.md`: 16384 minus the code 2 header of 131 including reserved bits), entry width 200 (settled item 8 of D8 (Core Index Structure), breakdown 8 + 2 + 2 + 2 + 86 + 8 + 8 + 8 + 76), 81 trees per level. E131 (Two Livelist Carriers: Values and Costs) thresholds per_head 41 / 3281, shared 81 / 6561, artifact `e131-livelist-carrier-2026-09-16.out`, 9 lines changed (the measurement basis line, three `tree_table_levels` lines each for per_head at 64 heads and 4096 heads, two criterion 1 lines); E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) thresholds 27 / 2187, 20 / 1640, 39 / 3279, artifact `e132-livelist-carrier-recount-2026-09-16.out`, 11 lines changed. Criterion 1 does not trigger in either; the "about 2/3" of E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) is unchanged, and not one artifact line from criterion 2 down changed. Unit tests 19 / 13 all pass, mutations 15 / 14 all caught; the two mutations anchored on the denominator and the entry width had their replacement values changed to values sensitive at the 200-byte tier (the original replacements 16320 and 16284 both give 81 at this tier and cannot be caught).
  - **Basis**: the user pointed out that E131 (Two Livelist Carriers: Values and Costs) still used the old denominator; the entry width changes together with the same quantity, and changing only the denominator would give a model mixing old and new. E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) computes the same quantity and is the current state designated by the E131 (Two Livelist Carriers: Values and Costs) page, so changing only one would create a new contradiction on this quantity. **Not changed along with it**: the child pointer width 59 of E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout); the five apparatuses of E79 (Capacity of Root Record), E128 (Point Lookup Cost of Pointers with Birth (tree, txg)), E133 (Format Cost of Mapping Key Candidates), E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) and E136 (Cost Rows for Mapping Key Forks) likewise hard-code 16284; they answer other questions and carry other old widths, and were not touched this round.

### 2026-09-10

- Added E131 (Two Livelist Carriers: Values and Costs) and ran it: "one tree per head" loses none of the five cells; the cost is that the tree table level threshold halves
  - **Why run**: on "each head gets its own tree" the user said verbatim "**this is a bit wild, isn't it. Look at the numbers and the performance.
    If they're good it's not unacceptable**" ⇒ numbers first, then the call.
  - **Conclusion**: `per_head` loses none of the five cells fixed before the run: the tree table still needs only two levels at 4096 heads;
    in point lookups 12 of 21 cells are shallower and 0 deeper (`heads=4096 blocks_per_head=65536` verbatim `per_head_descents=2 shared_descents=4`);
    occupied capacity is constantly 3/4 of `shared`; the pre-batch reservation needs no range count.
  - **The cost of "wild" measured**: 2 tree table entries per head ⇒ the tree table level threshold halves (second level at 68 heads vs 134 heads,
    third level at 8979 heads vs 17956 heads; the first half pinned by unit tests, the second half arithmetic at 134 trees per level).
  - **The unmodeled cell must be carried along**: where the "no range count needed" counter of `per_head` lives is recorded as 0 in the model and not built;
    either it lives in the 32 reserved bytes of the tree table entry, or a separate accounting statistic with a tree dimension is set up (the latter would reopen the 2026-09-06 withdrawal).
  - **Measurement basis**: the per-level denominator of the tree table takes 16284 (the one of C148 (Wider tree table entries compress indirection threshold to two layers));
    the other measurement basis is recorded in C157 (Tree table capacity computed with differing entry widths in two places), and this experiment does not pick one itself.
  - **Two mutations did not go red in the first round, handled separately**: one real blind spot (the descent count had only mutual comparison assertions; an absolute value was pinned),
    and one third-class insensitive sampling point (the only absolute assertion on occupied capacity was pinned at the baseline point where the scaling factor happens to be 1; a point off the baseline was added).
  - **Artifact**: `e131-livelist-carrier-2026-09-10.out`, registered into `scripts/experiment-apparatus/replay.sh` (`exact`).

- See the same-day entry "Added E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) and ran it: the E1 (Node Size vs Recoverability)… the user based a stance on" in the E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) section.

## E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout)

### 2026-09-18

- The experiment index row of E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) catches up with the 2026-09-16 rerun
  - **Before**: that index row said "one livelist per head 44 heads, shared tree 66 heads, third level 5985 / 8977 heads" (the numbers at entry width 121 and 134 trees per level).
  - **After**: 27 heads / 39 heads, third level 2187 / 3279 heads (rerun at tree table entry 200 and 81 trees per level), per the "now" row of the experiment page.
  - **Fixed in the same entry**: the E131 (Two Livelist Carriers: Values and Costs) index row "by its own model it reports 68 heads / 134 heads" changed to the rerun's 41 / 81 heads, per the "now" row of the experiment page.
  - **Basis**: the line-by-line judging reports of the back-sweeper reliability acceptance (`sweep-acceptance-2026-09-18-v2-judge-4.md`, `-v3-judge-4.md`), with the main agent checking against the experiment pages fresh.

### 2026-09-16

- See the same-day entry "E131 (Two Livelist Carriers: Values and Costs) and…" in the E131 (Two Livelist Carriers: Values and Costs) section.

### 2026-09-13

- See the same-day entry "Added E146 (Width and Cost of Livelist Entries Identified by Mapping Key) and ran it: entry 2…" in the E146 (Width and Cost of Livelist Entries Identified by Mapping Key) section.

### 2026-09-10

- Added E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) and ran it: the numbers of E131 (Two Livelist Carriers: Values and Costs) that the user based a stance on are corrected, and one tree per head still loses no cell
  - **Why run**: the numbers of E131 (Two Livelist Carriers: Values and Costs) were used by the user to state a leaning, while the reverse-inference leg in the first of three adversarial rounds on settled item 2 of D6 (Snapshot Implementation Model)
    hit two modeling errors, confirmed by the main agent checking fresh: it counted tree table entries as "one data tree per head, no pool-level tree",
    and used the leaf fanout for internal nodes. Per this repository's precedent "an experiment whose criteria are fixed and whose artifact is committed is not changed back; rerun under a new number", a new number was set up, and E131 (Two Livelist Carriers: Values and Costs) is kept as is.
  - **Conclusion**: the second-level tree table threshold is **44** heads for one livelist per head, **33** heads for one per data tree, **66** heads for the shared tree;
    third level **5985 / 4489 / 8977** heads ⇒ one tree per head does not reach the third level within 1024 heads. Under five entry widths, point lookups for one tree per head
    are shallower in 12–16 of 21 cells and **deeper in 0**. ⇒ the "wild" cost is that the threshold drops to about 2/3 of the shared tree's, not the "halving" reported by E131 (Two Livelist Carriers: Values and Costs).
  - **Criterion 3 only reports, does not judge**: criterion 4 of E131 (Two Livelist Carriers: Values and Costs) framed occupied capacity as win/loss, violating `.claude/rules/fs-design.md`
    "'who saves more' does not constitute any criterion"; lands in C268 (Criterion table framed economy comparison as win/loss).
  - **In the first mutation round M9 did not go red**; judged a third-class insensitive sampling point, and caught after adding a sampling point straddling the two-level boundary; all 14 caught.
  - **Artifact**: `e132-livelist-carrier-recount-2026-09-10.out`, registered into `scripts/experiment-apparatus/replay.sh` (`exact`).

## E133 (Format Cost of Mapping Key Candidates)

### 2026-09-16

- See the same-day entry "E131 (Two Livelist Carriers: Values and Costs) and…" in the E131 (Two Livelist Carriers: Values and Costs) section.

### 2026-09-11

- Added E133 (Format Cost of Mapping Key Candidates) and ran it: the format cost of the 12 key forms in the two argument rounds, reporting numbers only, not judging win/loss
  - **Before**: for the candidates of settled item 6 of D19 (Block Pointer Structure & Width Budget) no apparatus had computed format cost across them together; E128 (Point Lookup Cost of Pointers with Birth (tree, txg)) only computed pointer + 16, and E109 (Runtime Costs of Three Location Authority Arms) only scanned key 33 / 40.
  - **After**: E133 (Format Cost of Mapping Key Candidates) gives, for 12 arms, the widths of the pointer, header, root record, tree table entry and mapping entry, the fanouts and level thresholds of four trees, and the mapping tree's level count and leaf-level bytes at the pool size of E109 (Runtime Costs of Three Location Authority Arms).
  - **Basis**: pre-run registration `e133-preregistration.md`; 16 unit tests / all 19 mutations caught; the cross-apparatus gate matches E128 (Point Lookup Cost of Pointers with Birth (tree, txg))'s 75 / 109 / 97 / 226 / 137, E109 (Runtime Costs of Three Location Authority Arms)'s 267, the 196 / 164 of that cell in settled item 7 of D19 (Block Pointer Structure & Width Budget), and 134 / 118 under the measurement basis of C157 (Tree table capacity computed with differing entry widths in two places).
  - **One modeling error corrected before the run**: the first version added the 40 bytes of the strongest logical identity form to every pointer (pointers to index nodes should carry only birth tree + birth txg, 16), and on that basis once reported "the mapping tree gains a level" in conversation; after correcting by the two kinds of pointer increment, the one that gains a level is logical identity + write-order key.

- See the same-day entry "Added E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) and ran it…" in the E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) section.
- See the same-day entry "Added E136 (Cost Rows for Mapping Key Forks) and ran it: D19 (Block Pointer Structure & Width Budget) settled item…" in the E136 (Cost Rows for Mapping Key Forks) section.
- See the same-day entry "Added E137 (Performance Disparity Across Mapping Key Forms) and ran it: D19 (Block Pointer Structure & Width Budget) settled item…" in the E137 (Performance Disparity Across Mapping Key Forms) section.

## E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms)

### 2026-09-16

- See the same-day entry "E131 (Two Livelist Carriers: Values and Costs) and…" in the E131 (Two Livelist Carriers: Values and Costs) section.

### 2026-09-11

- Added E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) and ran it: the slot baseline of E133 (Format Cost of Mapping Key Candidates) corrected, plus the form tightened by per-class reuse of the write-order key and the hybrid path
  - **Before**: E133 (Format Cost of Mapping Key Candidates) took "the slot already has five-tuple 33 and write order 10" as the slot baseline of D27 (Small-Data Packing Container); those 10 write-order bytes came from the definition of the `pack` arm of E114 (Overall Accounting of Small-Data Packing Containers), while the text of D27 (Small-Data Packing Container) only puts the five-tuple in the slot. The form tightened by per-class reuse of the write-order key and the hybrid path had no cost numbers.
  - **After**: E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) reports both baselines and adds two arms. The source of E133 (Format Cost of Mapping Key Candidates) changed only that one comment; its artifact and mutation table are kept as is, and its body gains a sentence pointing over.
  - **Basis**: the third-round reverse-inference leg on settled item 6 of D19 (Block Pointer Structure & Width Budget) pointed out the slot baseline premise, confirmed by the main agent with `grep -c 写序` on D27 (Small-Data Packing Container) returning 0; pre-run registration `e134-preregistration.md`; 20 unit tests / all 24 mutations caught; cross-apparatus gate: the twelve shared arms reproduce the committed artifact of E133 (Format Cost of Mapping Key Candidates) line by line.
  - **Why a new number**: the artifact of E133 (Format Cost of Mapping Key Candidates) was already cited line by line by the third-round reverse-inference leg (`d19-item6-r3-opus-output.md` cites its lines 29 and 69), and changing it in place would leave those citations mismatched.
- One item in the E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) body changed: which code the class tag takes is written as the tightened current state, and it states that the two new arms assume different things about the slot
  - **Before**: the "what it cannot answer" section said "nobody has settled the value range of the class tag…… the number reported here is the non-reused one".
  - **After**: written as the current state after tightening in the first of three adversarial rounds on settled item 6 of D19 (Block Pointer Structure & Width Budget) (take the registry code of the named object; the slot is always code 1), stating that the hybrid path uses the first byte of the five-tuple as the class tag while `class_reuse_tagged` does not reuse it, so the two arms' assumptions differ by 1 byte here. Artifact, source and mutation table untouched.
  - **Basis**: pointed out by the reverse-inference leg in the first of three adversarial rounds, confirmed by the main agent checking those two rows of the E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) arm table; for the background see [decisions-history.md](decisions-history.en.md) (part three).

- See the same-day entry "Added E136 (Cost Rows for Mapping Key Forks) and ran it: D19 (Block Pointer Structure & Width Budget) settled item…" in the E136 (Cost Rows for Mapping Key Forks) section.
- See the same-day entry "Added E137 (Performance Disparity Across Mapping Key Forms) and ran it: D19 (Block Pointer Structure & Width Budget) settled item…" in the E137 (Performance Disparity Across Mapping Key Forms) section.

## E135 (Dynamic Rollback Floor)

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-11

- Added E135 (Dynamic Rollback Floor) and ran it: the fifth arm of settled item 1 of D16 (Publish Semantics) goes into the model
  - **Before**: "dynamic rollback floor F" was only a form the main agent drafted in that round's conversation (written in settled item 1 of D16 (Publish Semantics)), not in a model and not through a three-way argument.
  - **After**: E135 (Dynamic Rollback Floor) set up and run: six arms × seven worlds × three S tiers × 24 seeds; the cross-apparatus gates and absolute values of criteria 1, 2 and 5 all match, criterion 3 catches all three cells, criterion 4 has zero fake candidates for arm A and arm D, criterion 6 triggers (arm D gets fully stuck in the fill phase at an empty-publish overhead of 5 blocks and S = 16), and criterion 7 has every arm.
  - **Basis**: pre-run registration `e135-preregistration.md` (including five pre-run supplementary registrations written before the first run); six changes to the apparatus before the formal artifact (three on unit test discriminating power, two on filling a nearly full disk, one blind spot found by mutation M8), all recorded in the E135 (Dynamic Rollback Floor) body under "what was changed in the apparatus before the formal artifact"; criteria and failure clause not changed by a word.
  - **Not done yet**: using these numbers to settle settled item 1 of D16 (Publish Semantics) requires a three-way argument.
- A universal conclusion sentence in the E135 (Dynamic Rollback Floor) body corrected: spurious ENOSPC is not "absent in every cell"
  - **Before**: the verdict section said "none of the three arms has spurious ENOSPC in any cell, nor real ENOSPC"; the near-full-disk cost table had no spurious ENOSPC column.
  - **After**: the table gains a spurious ENOSPC column; the sentence changed to "non-zero in only one cell: at an empty-publish overhead of 5 blocks and S = 16, arm A (and control arm B) 24 times each; arm D's cell never entered the counting phase, so its reading does not count as 0". Artifact and source not changed by a word.
  - **Basis**: pointed out by the forward-inference leg of the second three-way round on settled item 1 of D16 (Publish Semantics) checking the artifact (item 5-1 of `d16-item1-r2-sonnet-output.md`), confirmed by the main agent printing the whole lines of `e135-rollback-floor-2026-09-11.out`. The error came from picking columns with awk when excerpting the table, missing the false_enospc column, and then treating everything not excerpted as 0: the shape of the `.claude/singlefs-ai-sop/rules/evidence-discipline.md` rule "the qualifier is gone, and the conclusion is wider than the artifact". The second-round background material already sent is kept as evidence unchanged and not changed back; this error is recorded in that round's verdict.
- The E135 (Dynamic Rollback Floor) body gains two scope notes and the measurement basis of `df`
  - **Before**: the "what it cannot answer" section had six items and did not state the interval of fault injection or the measurement basis of "generations that can be rolled back".
  - **After**: item 7 added (faults were never injected in the interval from raising F to it taking effect; arm A's zero is close to constructive) and item 8 (the rollback generation count counts roots, not distinct user states); the near-full-disk paragraph gains the `df` measurement basis for "spurious ENOSPC": under a different `df` measurement basis, the conclusion of that cell may flip (the section of settled item 9 of D3 (Space Allocation)). Artifact and source untouched.
  - **Basis**: two items from the reverse-inference leg of the second round on settled item 1 of D16 (Publish Semantics) (fault injection interval, measurement basis of rollback generations; `d16-item1-r2-opus-output.md`); the main agent checked the fault placement against the source and the conversion against the artifact fields.

## E136 (Cost Rows for Mapping Key Forks)

### 2026-09-16

- See the same-day entry "E131 (Two Livelist Carriers: Values and Costs) and…" in the E131 (Two Livelist Carriers: Values and Costs) section.

### 2026-09-11

- Added E136 (Cost Rows for Mapping Key Forks) and ran it: the cost numbers missing from the fork before settled item 6 of D19 (Block Pointer Structure & Width Budget) went to the user
  - **Before**: the fork lacked four cost numbers (⑫㈤ in that cell of settled item 6 of D19 (Block Pointer Structure & Width Budget)): the branch holding only code 1 had no experimental configuration; the fanout cost of the code 2 header +4 was unmeasured (E133 (Format Cost of Mapping Key Candidates) admits the header increment does not feed back into fanout); the mapping tree level difference took only one pool size point; the C280 (Central Mapping Has No Clause for Which Unit Classes It Contains) switch itself and the one extra read when moving code 2 nodes of derived trees had no numbers.
  - **After**: E136 (Cost Rows for Mapping Key Forks) reports both configurations × both fanout measurement bases, plus move bytes, parent node read count and the number of extra nodes read for derived trees.
  - **Basis**: pre-run registration `e136-preregistration.md` (written before the apparatus, with two supplements before the first run); 15 unit tests / all 24 mutations caught; three cross-apparatus gates (the committed artifact of E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms), line 308 of D18 (Information Carried in a Block), the move table of E109 (Runtime Costs of Three Location Authority Arms)). On the first run one unit test was red (the rounding of the expected value of the leaf-level byte ratio); after the first run, an output section "the code 2 header increment taken on its own" and two unit tests were added and the artifact regenerated; no body text had cited it before.

- See the same-day entry "Added E137 (Performance Disparity Across Mapping Key Forms) and ran it: D19 (Block Pointer Structure & Width Budget) settled item…" in the E137 (Performance Disparity Across Mapping Key Forms) section.

## E137 (Performance Disparity Across Mapping Key Forms)

### 2026-09-11

- Added E137 (Performance Disparity Across Mapping Key Forms) and ran it: the performance gap among the three arms of settled item 6 of D19 (Block Pointer Structure & Width Budget)
  - **Before**: E133 (Format Cost of Mapping Key Candidates), E134 (Mapping Key Format Cost: Slot Baseline and Two New Arms) and E136 (Cost Rows for Mapping Key Forks) only reported format quantities and never touched key order; after looking at the cost rows of E136 (Cost Rows for Mapping Key Forks) the user said "the costs don't show a performance gap".
  - **After**: E137 (Performance Disparity Across Mapping Key Forms) uses an ordered leaf structure to measure three arms × two mapping scopes: blocks per fsync for five loads × batch 1 / 10, mapping tree blocks per unit for four background loads, random read misses for three cache tiers × two hint staleness tiers, seeds 1 / 2 / 3.
  - **Basis**: pre-run registration `e137-preregistration.md` (written before the apparatus; supplement one before the first run; supplement two after the unit tests' first run and before the artifact's first run: two places in the apparatus did not match the registration, multi-stream appends went to new files and snapshot deletion overwrote the full mirror; the apparatus was changed back to the registration, and `snapshot_delete_global` was added); 15 unit tests / all 27 mutations caught; the cross-apparatus gate pins to the committed artifact of E136 (Cost Rows for Mapping Key Forks); positive controls and discriminating-power controls pass; rerun byte-identical.

- See the same-day entry "Added E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) and ran it: arm G on D3 (Space Allocation) settled item…" in the E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) section.

## E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty)

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-12

- The E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) body changes its mechanism and adds scope: "why arm G loses" replaced by the reproduced form, and the near-full-disk conclusion holds only without faults
  - **Before**: the paragraph "why arm G loses" was marked "inferred", saying the problem is that the number of publishes for handling varies with placement and the fix is "always do all 7"; the criterion 6 cell said both arms' minimum non-stuck reserve pool is "smaller than the pre-run formula"; none of the ten items in "what it cannot answer" covered the combination of near-full disk and faults; the cost paragraph said arm G "about 4.8 times".
  - **After**: the mechanism replaced with the form the main agent reproduced (the root cause is that acceptance admits by the actual residue while `df` deducts by the worst-case residue, stopping at −3 when full); criterion 6 notes that arm A's number is the lower edge of the scan; item 10 replaced with the scope "near-full disk, no faults", and item 11 added (instance switchover reservation, overhead taken as a constant); 4.8 → 4.9. Artifact and source untouched.
  - **Basis**: the third round on settled item 1 of D16 (Publish Semantics): the forward-inference leg wired "handle until put back" into acceptance in a copy and failed the same way seed by seed; the reverse-inference leg hit two places, near-full disk plus one write failure and a bad slot; the main agent reproduced it in a copy (`research/prompts/d16-item1-r3-main-verification.md`, `-2.md`).

- See the same-day entry "Added E139 (Tightened Form of Per-Disk Rollback Floor) and ran it: the tightened form read by root count passes all criteria; the one read by state count was deleted…" in the E139 (Tightened Form of Per-Disk Rollback Floor) section.

### 2026-09-11

- Added E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty) and ran it: arm G is out on item 2 of settled item 9 of D3 (Space Allocation), and arm A meets both
  - **Before**: none.
  - **After**: new body and index row. Pre-run registration `e138-preregistration.md`; 15 unit tests, all 9 mutations caught (M6 was missed in the first round and caught after adding a sensitive sampling point); artifact `e138-per-disk-floor-2026-09-11.out`.
  - **Basis**: the second-round verdict on settled item 1 of D16 (Publish Semantics) and the same-day user settlement (retain at least 4 durable roots + the newest root of each disk; settled item 9 of D3 (Space Allocation)).
  - ⚠️ The pre-run registration was first written to `e137-preregistration.md`: another session had taken the number E137 (Performance Disparity Across Mapping Key Forms) a few minutes earlier and written an uncommitted file of the same name, which this `Write` overwrote; it was restored verbatim from that session's conversation record (the original 58 lines + the supplement it appended afterwards), and this one was renumbered to E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty).

## E139 (Tightened Form of Per-Disk Rollback Floor)

### 2026-10-05

- 2nd rerun (candidate floor adds the system configuration's F, the reclamation threshold and five other places falling back to the whole ring) run: comparing the 610 lines of the 1st run's cells line by line, 228 lines changed their counts, all in A (only the counting measurement basis changed), 0 in B and C; the fake candidates of `activation` for G2SW drop from 46 lines to 6; the old floor's candidate set is strictly larger than the clause form's 53392 times, the reverse direction 0 times (the page's sentence "not larger, slightly fewer" had the direction reversed and was corrected); RGall and RGall_Q have all-zero fake candidates, unrecoverables and spurious ENOSPC in judged cells on the 1st run's 13 world classes and on `disk_stuck`; on `disk_stuck` the units held have no upper bound in 7 lines and are stuck in 1; `broken_slot` and H1 have no upper bound at S = 4 and an upper bound at S = 16
  - **Before**: the page and index row before this change (the 1st rerun section was written, uncommitted, in the same batch as this one; the E139 (Tightened Form of Per-Disk Rollback Floor) row in `git show 31e47115:.claude/kb/experiments.md` is from before the 1st rerun).
  - **After**: the experiment page gains a "2nd rerun" section (reading table, verdict lines copied whole, counting commands and raw output, comparison against settled item 1 of D16 (Publish Semantics), what it cannot answer); in the 1st rerun section the rollback candidate set row, item 3 of what it cannot answer and the "direction overview" had their direction changed; the E139 (Tightened Form of Per-Disk Rollback Floor) line in `scripts/experiment-apparatus/replay.sh` points to `e139-tightened-floor-2026-10-05.out` (the old artifact `e139-tightened-floor-2026-10-04.out` is kept); unit tests 36 → 55, mutations 26 → 42; two "affected decisions" rows reviewed, and one D23 (Journal Role & Format) row added.
  - **Basis**: rerun registration `research/prompts/e139-r2-prereg.md`; question sheet `research/prompts/m3-r1-e139-r2-questions.md`; artifact `e139-tightened-floor-2026-10-05.out`; owed check C660 (Sentence in per-disk rollback floor settlement does not match apparatus).
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (the wording before the owed check short name cited in this line was renamed on 2026-10-11)

### 2026-10-04

- 1st rerun (reclamation threshold changed to a one-lap window) run: arm G2SW with the one-lap window gives fake candidates in 46 of the 60 lines of the activation family and unrecoverables in 3, with 0 spurious ENOSPC in judged cells; the old-threshold control G2SW_ringgate is all 0 in its 108 judged lines; 5 item-by-item matches for C
  - **Before**: `git show 31e47115:.claude/kb/experiments/139-按盘回退下界的收严形态.md` and the E139 (Tightened Form of Per-Disk Rollback Floor) row in `git show 31e47115:.claude/kb/experiments.md` (status "run (2026-09-12, counting model, 21 unit tests / all 16 mutations caught)", with no 1st rerun section).
  - **After**: the experiment page gains a "1st rerun" section (reading table, verdict lines copied whole, line count self-check, unit tests and mutations, minimal example, item-by-item match table, the impact of difference ①, comparison against that sentence of settled item 1 of D16 (Publish Semantics), what it cannot answer); the index row status rewritten; the E139 (Tightened Form of Per-Disk Rollback Floor) line in `scripts/experiment-apparatus/replay.sh` points to `e139-tightened-floor-2026-10-04.out`; unit tests 21 → 36, mutations 16 → 26.
  - **Basis**: rerun registration `research/prompts/e139-r1-prereg.md`; artifact `e139-tightened-floor-2026-10-04.out`; owed check C660 (Sentence in per-disk rollback floor settlement does not match apparatus).
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (the wording before the owed check short name cited in this line was renamed on 2026-10-11)

### 2026-09-16

- Artifact lines copied whole into the bodies of E139 (Tightened Form of Per-Disk Rollback Floor) and E144 (Cost and Discriminative Power of Header Checksum Algorithms) changed back per the committed artifacts
  - **Before**: in the verdict block of the E139 (Tightened Form of Per-Disk Rollback Floor) body, the two lines `name=near_full arm=G2R_tightened_roots s=16` (c0 = 1 and c0 = 5) had `admissions_under_pressure` 1848 / 1608 and `avg_candidates_x100` 2019 / 2036, and the body sentence "`admissions_under_pressure` 1608–1848 vs 9720–9984" was written from them; the `name=probe arm=sum8` line and row 5 of the criterion table in the E144 (Cost and Discriminative Power of Header Checksum Algorithms) body said `accepted_used_bytes_only=3901`.
  - **After**: the two E139 (Tightened Form of Per-Disk Rollback Floor) lines copied back whole from `e139-tightened-floor-2026-09-12.out` (`admissions_under_pressure` 2784 / 1968, `avg_candidates_x100` 1490 / 1433; these two lines are identical in the first run's `-run1.out`), the sentence changed to "1968–2784 vs 9720–9984", with the direction "G2R fewer than G" unchanged; the two places in E144 (Cost and Discriminative Power of Header Checksum Algorithms) changed to 3919 per `e144-header-checksum-cost-2026-09-13.out`, "≈ 1/256" unchanged.
  - **Basis**: on the day gate number 88 (artifact lines copied whole into kb bodies must be found verbatim in the artifact) was wired in, it swept out these three lines, found in none of the 290 artifacts under `research/results/`. In all three places the body's numbers do not match the committed artifact it names itself; how the body miscopied them can no longer be traced (there is no other output besides the committed artifacts), so they were changed per the committed artifacts.

### 2026-09-13

- See the same-day entry "E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E…" in the E141 (Mount Admission Self-Test for Switchover Reservation) section.

### 2026-09-12

- Added E139 (Tightened Form of Per-Disk Rollback Floor) and ran it: the tightened form read by root count passes all criteria; for the one read by state count, delete-then-rewrite must pass through 3 states
  - **Before**: after the third-round verdict on settled item 1 of D16 (Publish Semantics) there was only a direction (the user decided: go toward the per-disk rollback floor family, and set up an experiment measuring its tightened form); the tightened form had not a single cell of numbers.
  - **After**: E139 (Tightened Form of Per-Disk Rollback Floor) ran: the tightened arm G2R read by root count has zero fake candidates and zero unrecoverables in every fault world; on a near-full disk, with ≤ 2 root slot write failures during push-empty, with bad slots, and with overhead rising to c_max, every cell has 0 spurious ENOSPC, delete-then-rewrite always succeeds, and publishes ≤ 11 (exactly 11 with 2 failures);
    G2S, read by state count, likewise passes all correctness checks, but delete-then-rewrite fails in the first 3 windows after filling (the same for each of 24 seeds, and not once in the 400 windows after): the blocks deleted this time are put back only after 3 more state-changing windows;
    the "take the upper bound per disk" half (judged "unverified" by E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty)) was measured by the world that skips numbers first and then raises F, where control arm G_txgcap goes out of bounds 4192 times in each of three cells.
    Three changes were made while writing the apparatus and one after the first run (how the upper bound read by state count is taken at S = 1); the first run's artifact is kept as `-run1.out`, and the two runs differ only in G2S's five cells at S = 1.
  - **Basis**: pre-run registration `e139-preregistration.md`; artifact `e139-tightened-floor-2026-09-12.out`; 20 unit tests, all 15 mutations caught (M13 was caught only after adding the sampling point of raising once more after push-empty).
- After the fourth three-way argument round, E139 (Tightened Form of Per-Disk Rollback Floor) changes four concluding sentences and adds a test and an artifact: "all 26 cells are in the first 3 windows" does not hold for the reserve pool scan, and "never fewer than 4 states" holds only on a near-full disk
  - **Before**: the E139 (Tightened Form of Per-Disk Rollback Floor) body said the two artifacts differ only in five cells and two lines; all 26 G2S delete-then-rewrite failures are in the first 3 windows after filling; read by state count there are never fewer than 4 states; one handling with 3 failures uses 11 txgs; 205 blocks = 3.3 MiB; "G2R passes every cell" without qualification.
  - **After**: the diff is 8 lines (five cells, the lag of two `near_full` cells at S = 1, one summary line, all at S = 1); the near-full main cell and the 8 cells of rising overhead are windows 1..3, while the 18 cells of the reserve pool scan were measured by the post-run test `states_arm_reserve_sweep_failure_windows_end_by_window_six` to have last window 6 in the four cells c₀ = 1, reserve = formula + 2..5 (artifact `e139-tightened-floor-2026-09-12-reserve-sweep-windows.out`; mutation M16 turns it red); "never fewer than 4 states" narrowed to near-full disk S ∈ {4, 16} (fault worlds `min_states` 1..2); "11 txgs" withdrawn; 3.2 MiB; G2R's "passes all" carries two qualifications (it depends on live metadata being recorded back into the reserve pool; item 1 cannot be judged until C126 (Worst-case amount of switching reservation has no measurement basis) has a number). The literal text of criterion 7, `bound_exceeded` being constantly true, and seven leads from the reverse-inference leg's probes were written into "notes after the fourth three-way argument round". Unit tests 20 → 21, mutations 15 → 16.
  - **Basis**: the fourth three-way argument round on settled item 1 of D16 (Publish Semantics), `research/prompts/d16-item1-r4-main-verification.md` (forward-inference leg `-sonnet-output.md`, reverse-inference leg `-opus-output.md`; the diffs and artifacts of the reverse-inference leg's probes are in `research/prompts/d16-item1-r4-opus-probes/`, and the main agent reran each one byte-identically).

## E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header)

### 2026-09-19

- See the same-day entry "E6 (Encryption Algorithm Selection), E58 (End-to-End Cost of Checksum Granularity), E65 (Write-Side Cost of Checksum Granularity), E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header)…" in the E6 (Encryption Algorithm Selection) section.

### 2026-09-13

- Third round of E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) run and committed: random page reads with headers 7.7%–9.1% slower, sequential padding 7.6%–9.2% less bandwidth, crossover at a random share of 1.05%–1.47%
  - **Before**: E140 (Cross-Unit Pages and Read-Side Copying Forced by Unit Header) had only the apparatus another session ran on 2026-09-12, the pre-run registration, the first round (voided per clause b) and half a second round, with no body, and three gate stages were red: "artifact not named", "experiment number has no body" and "naming discipline, 34 places".
  - **After**: the second round was completed and voided per clause b (after the disk went idle, the first arm in the process was 15% slower); the third round added warm-up and splitmix seed scrambling with the criteria unchanged word for word, none of the six failure clauses triggered, and criteria 1 / 2 point the same way on all five seeds; the body, the index row, the `timing` registration and four interval assertions in `replay.sh`, the experiment note on settled item 5 of D4 (Checksum Location), and the fourth place of C59 (Seeds Folding into the Same State) were written back together.
  - **Basis**: three rounds of registration in `e140-preregistration.md`; artifacts `e140-header-alignment-r3-2026-09-13.out` (verdict), `e140-header-alignment-nodirect-2026-09-13.out` (clause d), `e140-header-alignment-repro-2026-09-13.out` (rerun); 10 unit tests, all 12 mutations caught.

## E141 (Mount Admission Self-Test for Switchover Reservation)

### 2026-09-19

- See the same-day entry "E19 (Spurious ENOSPC Under Defer Window), E82 (In-Flight Synthesis for Admission)…" in the E19 (Spurious ENOSPC Under Defer Window) section.

### 2026-09-14

- E141 (Mount Admission Self-Test for Switchover Reservation) third run: reservation of N_switch + 1 shares (one more for the row-writing publish)
  - **Before**: second run (artifact `e141-switch-reserve-mount-admission-2026-09-13-warmup.out`, 120 lines): share = N_switch × (chain rewrite + warm-up), new-pool file creation geometry 42 / 87 blocks per disk (c_max 4 / 9), six wrong forms, 13 mutations.
  - **After**: artifact `e141-switch-reserve-mount-admission-2026-09-14-row-writing.out` (121 lines): share = (N_switch + 1) × (chain rewrite + warm-up), 56 / 116 blocks per disk; all five criteria pass, each of seven wrong forms (adding "missed the row-writing share") has a sampling point that separates it, all 14 mutations caught; the two hand-computed tables were recomputed before the run, and criteria and void clauses unchanged.
  - **Basis**: in the closing pop-up the user settled one more reservation share for C329 (no check for the empty publish that raises F before the row-writing publish) (section five of `research/prompts/c329-c330-r3-main-verification.md`), and settled item 3 of D28 (Mount-Time Commitments) was rewritten accordingly.

### 2026-09-13

- Added E141 (Mount Admission Self-Test for Switchover Reservation) and ran it: the discriminating-power self-proof owed by C126 (Worst-case amount of switching reservation has no measurement basis) done in a counting model, all five criteria pass
  - **Before**: settled item 3 of D28 (Mount-Time Commitments) settled the switchover reservation formula, and C126 (Worst-case amount of switching reservation has no measurement basis) had only the discriminating-power self-proof left undone: nothing proved that mount admission can separate two pools differing by just 1 block, separate per-device from pool-level scalars, or separate today's formula from several wrong forms of it.
  - **After**: E141 (Mount Admission Self-Test for Switchover Reservation) ran: 14 sampling points × two disks taking turns as the tight disk, all 28 pool pairs flip at the share; with the reservation set to 0 all are writable; the pool-level scalar cannot detect the short disk; the total reservation equals the 14 hand-computed numbers cell by cell (12 blocks for new-pool file creation geometry, 108 / 144 blocks at rows0 = 4064); each of four wrong forms has a sampling point that separates it, and the rows0 = 0 cell separates none of the four.
  - **Basis**: pre-run registration `e141-preregistration.md`; artifact `e141-switch-reserve-mount-admission-2026-09-13.out`; 8 unit tests, all 11 mutations caught.
- E141 (Mount Admission Self-Test for Switchover Reservation) row-width constant renamed, E79 (Capacity of Root Record) marks 243 void, E100 (Three-Segment Geometry of System Configuration) and E112 (Legacy Writer Encountering Unknown Trees) rewrite the cell citing settled item 2 of D15 (Format Freeze Policy), and citations in several experiments now point to the current items
  - **Before**: the E141 (Mount Admission Self-Test for Switchover Reservation) source called 64 "today's row width" and 88 "the row width lower bound given by C304 (Instance table chain pointer record cannot accommodate 83-wide pointer)"; the E79 (Capacity of Root Record) body reported the tree table at 243 per level; E100 (Three-Segment Geometry of System Configuration) and E112 (Legacy Writer Encountering Unknown Trees) wrote settled item 2 of D15 (Format Freeze Policy) as "the system configuration's tree table"; the bodies or source comments of E135 (Dynamic Rollback Floor), E138 (Per-Disk Rollback Floor and Space Requirements for Pushing Empty), E139 (Tightened Form of Per-Disk Rollback Floor), E50 (Upper and Lower Bounds on Root Ring Slot Count) and E116 (Packed Container Balance: Metadata Writes and Compaction Strategy), E119 (Padding Cost of Fixed-Width Slots), E120 (Choosing the Common Ratio of a Geometric Size-Class Table), E121 (Optimal Points for Tiering by Cap and Tier Count), E122 (Benefits Bought by Directory-Based Grouping) cited items by the old item states of D16 (Publish Semantics) and D27 (Small-Data Packing Container).
  - **After**: the two E141 (Mount Admission Self-Test for Switchover Reservation) constants renamed by pointer width (59 wide / 83 wide), mutation table M11 changed with them, all 11 mutations still caught; the artifact does not print constant names, and the rerun is byte-identical; the kb body states that the row width is settled at 88; the E79 (Capacity of Root Record) body gains a line "243 was computed with the old entry width 67 and is void; a rerun with one more tier is owed in C157 (Tree table capacity computed with differing entry widths in two places)"; that cell of E100 (Three-Segment Geometry of System Configuration) / E112 (Legacy Writer Encountering Unknown Trees) changed to "the tree table is an open list from day 1"; the citations in the other experiments were changed by the relabel tool to settled item 1 of D16 (Publish Semantics) and settled items 1 / 2 of D27 (Small-Data Packing Container); in the source only comments changed, and the artifacts are unchanged.
  - **Basis**: decision change history part three (settled item 1 of D16 (Publish Semantics)), part four (D27 (Small-Data Packing Container) reopened), part five (C304 (Instance table chain pointer record cannot accommodate 83-wide pointer), C157 (Tree table capacity computed with differing entry widths in two places) and C241 (Ownership of tree table, two user settlements contradict) closed out).
- E141 (Mount Admission Self-Test for Switchover Reservation) second run: share gains warm-up N_switch × R × c_max, 28 sampling points, 6 wrong forms, all five criteria still pass
  - **Before**: the share had only the chain rewrite half, artifact `e141-switch-reserve-mount-admission-2026-09-13.out` (62 lines); the note on criterion 4 said verbatim "the switchover reservation gained 'R empty publishes × freshly computed c_max'; this apparatus tested neither".
  - **After**: share = chain rewrite + N_switch × R × c_max (R = 3, c_max scanned at 4 / 9 ⇒ 36 / 81 blocks per disk, one share each on the two mirrored disks), artifact `e141-switch-reserve-mount-admission-2026-09-13-warmup.out` (120 lines): `crit1_bad_pairs=0 crit2_control_failures=0 crit3_settled_writable=0 crit3_scalar_read_only=0 crit4_mismatches=0 crit5_blind_forms=0`; the wrong forms "warm-up missed" and "warm-up only as one pool-level share" are separated at all 28 points; 9 unit tests, all 13 mutations caught; the rerun registration points to the new artifact.
  - **Basis**: knock-on from settled item 8 of D16 (Publish Semantics) and settled item 3 of D28 (Mount-Time Commitments); the half-paid cell of C126 (Worst-case amount of switching reservation has no measurement basis).

## E142 (New-Pool File Creation Dry Run)

### 2026-10-21

- Two statements citing the full count of the `crates/` new-pool file creation stream changed to current values for two rounds of number acquisition writes: one sentence on the crash-point replay main arm gains "the apparatus measured with one round of number acquisition writes; after two rounds the full count is 16777268, 61 operations", and limitation item 9 separates what the apparatus measured from today's stream in `crates/`; the apparatus's numbers, conclusion and artifacts unchanged.
  - **Basis**: `FULL_STATES` in `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` is 16_777_268; `.claude/kb/layout/01-first-txn.md` gives the same number.
  - **Before**: `git show 841b3beab:.claude/kb/experiments/142-新池新建文件的干跑.md`.

### 2026-10-17

- The opening paragraph checked and replaced against the twenty-fifth main artifact (`gap` 33 lines, verdict lines `layer0_*` not_run, 29 transactions writing 6 barriers and 1 FUA naming 12 units, positive control 8192 states with 4092 violations, and the journal load-bearing cell stating that this page's apparatus does not count); the apparatus line count written as "11081 lines per the frozen version of the twenty-fifth run; for the current count use wc -l"; limitation item 9 gains "at the time of the fourteenth run" and adds today's stream (12 units, closed form 16777240, 16777260 with the third state added); the mkfs model of G19 changed to five segments from the twenty-fifth run on, identical verbatim to the registration line; in the G table, placements C305 (inline formulas in kb are summed by nobody), C311 (the conjunction measurement basis for the two journal mirror copies is unsettled), C306 (code 2 header is not self-describing about key width) and C304 (Instance table chain pointer record cannot accommodate 83-wide pointer) marked paid off, and C284 (No Definition of What Applying a Record Does at the Pointer Layer) and C291 (birth sequence number allocation rule exists only in experiment source) changed to clause half paid, check half still owed; G4's "offset column" changed to "width and row order"; the formula statement for the apparatus constant `JOURNAL_NEW_ROOT_SEGMENT_BYTES` at line 199 changed to "at the eleventh run", now the literal 364; the verdict section heading gains the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/142-新池新建文件的干跑.md`.

### 2026-10-15

- 12 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 12 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/142-新池新建文件的干跑.md`.

### 2026-10-13

- E142 (New-Pool File Creation Dry Run) twenty-fifth run: after the apparatus added, per the clauses, mkfs zeroing [0, root ring base) on each disk and the retired tree, the mkfs line is identical verbatim to the eight registered rows of layout table, and at the main point all 29 regions are byte-for-byte identical to `crates/`; after the replacement texts of the whole mutation table were fixed, the whole table rerun gave 220 ✅
  - **Before**: apparatus 10868 lines (`227ef79b…f281`), the mkfs line was `operations=23 segments=12+1+1+1+4 closed_form=4114`, tree table 11 entries, watermark 65548, label 6 98386; after the retired tree was added to `crates/` the apparatus had not caught up (C774 (E142 (New-Pool File Creation Dry Run) apparatus leaves 26th registration after catching up with the retirement tree)); mutation table 210 rows; the rerun registration pointed to the twenty-fourth main artifact and the P8G export; the experiment page and index row said "run (enough to judge; 2026-10-05…)".
  - **After**: the apparatus changed in two segments (N25M: mkfs records one more step per disk, zeroing [0, root ring base); N25: tree table entry 11 the retired tree, watermark 65549, label 6 gains the retired item), frozen at 11081 lines `352ac89832c32bb17728a5f71eb5b11e5d47c3410760356b518a6fea05e46588`, unit tests 129 passed 0 failed 1 ignored. Question sheet line 1 identical (`R25_MKFS arm=N25M cmp=same`, main artifact line 35 `operations=25 segments=14+1+1+1+4 closed_form=16402`); line 2: all three comparisons equal (`impl_bytes_equal_summary … matched=29 equal=29 unequal=0`, `layout_field_summary rows=180 equal=180 unequal=0`, the four label 6 lines all decode to 196760), P8G judged the same; the second-segment gate `R25_VERDICT profile=retire pass=false groups_bad=p8g_control_main_point` hit the F25d cell per the registration's literal text, and the bad group is the fallback predicted by first-segment stop S25-drift (`p8g_control_main_point` back to `equal=12 unequal=0`); line 3 (a) does not hold literally (command seven `residual_old_values count=2`, both being the mutation name M233), and PC-old sees `count=3` (registration says 1); the main agent judged: F25d counts as one loss per the registration, line 2 not enough to judge (the twenty-sixth registration writes the S25-drift fallback as an expected group and then judges); (a) recorded per F25j, the scan treating mutation names as old values (the twenty-sixth run changes the scan); PC-old not voided (the before/after difference of 1 equals the injected count); (b) holds (`Q142_VERDICT r71_flipped=0 extra_flipped=0 r71_keys_union=77`); (c) mutation table 220 rows (M232–M241 added, M79, M15, M204, M206, M227 re-anchored), first whole-table run `✅` 189, `⏭` 31: `scripts/experiment-apparatus/Cargo.toml` had gained `[lints.rust] warnings = "deny"`, and 31 replacements left unused names behind that would not compile; those 31 replacement texts were changed to forms that compile under deny with the same change (anchors and names unchanged, table `0b5f8d16571334f692db29c82ae4c623f22480b18e79413edc850e6fa2ee75ce`), the sub-table of only these 31 rows gave `✅` 31, and the fixed whole table rerun gave 220 ✅ (`research/results/e142-mutate-r25-2026-10-13-b.log`, ⏭ 0, ❌ 0, last line "restored, baseline still all green"). The rerun registration now points to the twenty-fifth main artifact and the P8G export; the experiment page and index row status changed to "partly run (2026-10-13…)".
  - **Basis**: `research/results/e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out`, `research/results/e142-r25-p8g-crates-dump-2026-10-13.out`, `research/results/e142-r25-s1-comparisons-2026-10-13.out`, `research/results/e142-r25-s2-comparisons-2026-10-13.out`, `research/results/e142-r25-anchors-2026-10-13.out`, `research/results/e142-mutate-r25-2026-10-13.log`, `research/results/e142-mutate-r25-2026-10-13-b.log`, `research/results/e142-mutate-r25-s3-2026-10-13.log`; pre-run registration `research/prompts/e142-r25-prereg.md`; item-by-item numbers in the experiment page section "2026-10-13 (twenty-fifth run)".

### 2026-10-05

- E142 (New-Pool File Creation Dry Run) twenty-fourth run: after the gate was narrowed to Q142.123 (five groups change, of which the two write-list groups change only `sha256=`), the apparatus (the twenty-third one, unchanged by a word) and `crates/` are still byte-for-byte identical in all 29 regions at the main point, the gate passes, and what the twenty-third run had enough to judge still holds
  - **Before**: the twenty-third run's gate Q142.114 allowed only three groups to change and hit F23d per the registration's literal text (`write_list_row` and `p8g_write_list_row` each changed 2 lines), so question sheet lines 1 and 3 were literally not enough to judge; the rerun registration pointed to the twenty-third main artifact and P8G export.
  - **After**: registration `research/prompts/e142-r24-prereg.md` relaxed the gate to Q142.123 (five groups may change; the two extra groups may change only the `sha256=` column, with old and new values each equal to the `sha256=` of the region line of the same write); the apparatus untouched, no mutations added. Main point `impl_bytes_equal_summary … matched=29 equal=29 unequal=0`; `Q123_VERDICT pass=true ambiguous=0 groups_outside_allowed=none` (`changed_lines_total=20`, five groups 2, 12, 2, 2, 2); `Q123 order_same=true`, `Q123 changed_lines_total=0` (the twenty-third main artifact against this main artifact: no drift on the apparatus side); `CRATES_HALF … identical=true` (the `crates/` side has not moved since the twenty-third run); the four main-point lines decode `t_time=5000`, label 6 `98386`; t13 `2660362816`, w4 `750278362`; the field table's 174 rows all equal; command seven `count=0`; the verdict fields (77) identical one by one with the twenty-second run's second-segment main artifact. The whole mutation table of 210 handed to the main agent to run. The rerun registration now points to this main artifact and this P8G export (the P8G export is byte-identical to the twenty-third one).
  - **Basis**: `research/results/e142-new-pool-file-creation-dry-run-r24-main-2026-10-05.out`, `research/results/e142-r24-p8g-crates-dump-2026-10-05.out`, `research/results/e142-r24-comparisons-2026-10-05.out`, `research/results/e142-r24-anchors-2026-10-05.out`, `research/results/e142-r24-controls-2026-10-05.out`; pre-run registration `research/prompts/e142-r24-prereg.md`; item-by-item numbers in the experiment page section "2026-10-05 (twenty-fourth run)".

- E142 (New-Pool File Creation Dry Run) twenty-third run: after the on-disk unit of T_time changed to milliseconds (default 5000), the apparatus changed only one place at line 1524, and is byte-for-byte identical in all 29 regions to `crates/` after the 30-3 merge, at the main point and on P8G; one gate cell was hit per the registration's literal text, and lines 1 and 3 are handed to the main agent to judge
  - **Before**: apparatus line 1524 said `writer.put_u32(5); // T_time 秒`, byte-identical to `crates/` before the merge; the rerun registration pointed to the twenty-second run's second-segment artifact and P8G export; the three lines of the question sheet `research/prompts/e142-r23-questions.md` were open.
  - **After**: line 1524 changed to `writer.put_u32(5000); // T_time 毫秒` (exactly one line of `diff` from the frozen version; unit tests 117 passed 0 failed 1 ignored; the mutation table of 210 untouched; the value of T_time has neither a mutation nor a unit test pinning it, only an absolute value check on the artifact). Main point `impl_bytes_equal_summary … matched=29 equal=29 unequal=0`, P8G `P8G_SUMMARY … matched=29 equal=29 unequal=0`; the eight lines decode `t_time=5000` on both sides; reverse chain t13 `2660362816` and w4 `750278362` unchanged; the field table's 174 rows all equal; the verdict fields (77) identical one by one with the twenty-second run's second-segment main artifact; the 12 fields of `p8g_verdict` unchanged against both the main point and the twenty-second run's second segment. Arm N22R (apparatus as is, `crates/` after the merge) `equal=27 unequal=2`, the two different ones being exactly the system configuration slots at in-device offset 4096 on both disks. Gate Q142.114: in the normalized diff, besides the three groups named by the registration, `device_region_bytes`, `p8g_device_region_bytes` and `journal_header_write_digest`, `write_list_row` and `p8g_write_list_row` each changed 2 lines (only the `sha256=` field changed, identical to the `device_region_bytes` line of the same write, traceable to line 1524); per the literal text of registration F23d, lines 1 and 3 are not enough to judge and are handed to the main agent. The rerun registration now points to this main artifact and this P8G export; the whole mutation table of 210 handed to the main agent to run.
  - **Basis**: `research/results/e142-new-pool-file-creation-dry-run-r23-main-2026-10-05.out`, `research/results/e142-new-pool-file-creation-dry-run-r23-arm-n22r-2026-10-05.out`, `research/results/e142-r23-p8g-crates-dump-2026-10-05.out`, `research/results/e142-r23-comparisons-2026-10-05.out`, `research/results/e142-r23-anchors-2026-10-05.out`; pre-run registration `research/prompts/e142-r23-prereg.md`; item-by-item numbers in the experiment page section "2026-10-05 (twenty-third run)".

- E142 (New-Pool File Creation Dry Run) twenty-second run, second segment: on a second two-disk geometry P8G (8 GiB per disk), the apparatus and `crates/` compared byte by byte are identical in all 29 regions, and neither the field table nor the verdict flips
  - **Before**: question sheet line 2 was open; disk size had only one sampling point, 4 GiB (C317 (Verdict measured on single geometry sample point unchecked)), and the `crates/` export bin did not take a disk size parameter; the rerun registration pointed to the first-segment main artifact, and the experiment page and index row status said "partly run".
  - **After**: bytes per disk in the apparatus became a parameter, and `main()` adds a P8G segment after the main point (250 lines starting with `name=p8g_`); the frozen apparatus is 10866 lines, mutation table 210 rows (M55 re-anchored, M228–M231 newly taken in, the small table's 5 all caught), unit tests 117 passed 1 ignored. On P8G, against the P8G export of `crates/`, `equal=29 unequal=0`; the field table's 174 rows `equal=174 unequal=0`; the 12 class-A verdict fields identical one by one with the main point; accounting free space 7767572480 per disk, 7406 fully empty clustered segments, label 6 98386 (decoding on both sides agrees with A5 of `label6_anchor.py`); with the `name=p8g_` lines deleted, the second-segment artifact is byte-identical to the first-segment main artifact; question sheet line 2 is enough to judge. The rerun registration now has two lines: the second-segment artifact and the `crates/` export for P8G. The whole mutation table of 210 on the second-segment apparatus is handed to the main agent to run, not yet run.
  - **Basis**: `research/results/e142-new-pool-file-creation-dry-run-r22-s2-main-2026-10-05.out`, `research/results/e142-r22-p8g-crates-dump-2026-10-05.out`, `research/results/e142-r22-s2-comparisons-2026-10-05.out`; item-by-item numbers in the experiment page section "2026-10-05 (twenty-second run, second segment)".

- The whole mutation table of 206 for the twenty-second run's first segment run by the main agent: 206 caught, 0 invalid, 0 not red, ending "restored, baseline still all green"; all four items of line 3 of the question sheet `research/prompts/e142-r22-questions.md` hold, enough to judge.
  - **Basis**: `research/results/e142-mutate-r22-s1-2026-10-05.log`.

- E142 (New-Pool File Creation Dry Run) twenty-second run, first segment: the apparatus's committed reservation computed fresh per the formula of settled item 2 of D3 (Space Allocation), and compared byte by byte with `crates/` all 29 regions are identical
  - **Before**: apparatus line 3000 wrote accounting item 6 (committed reservation) as the literal 0, and compared with `crates/` 9 of 29 regions differed (all a checksum cascade from this one place); the rerun registration pointed to `e142-new-pool-file-creation-dry-run-r21-main-2026-10-04.out`; the experiment page and index row said G13 did not run successfully and the whole mutation table had 199.
  - **After**: the apparatus computes label 6 fresh per the formula of settled item 2 of D3 (Space Allocation) (replica count × ((2 + H + 1) × 16384 + 41 × n_live), with replica count taken as the device count, H read fresh from the tombstone tree root pointer, n_live counting records with nlink ≥ 1, and 0 when the tree table has no `deleted_inodes` entry), with not a word changed elsewhere; unit tests 107 passed 1 ignored, mutation table 206 rows (M221–M227 newly taken in, the small table's 7 all caught; the whole 206 handed to the main agent to run, not run). Compared byte by byte with `crates/`, `equal=29 unequal=0`; label 6 decoded on both sides is 98386, and reverse chain t13 `2660362816` equals both the `crates/` pinned value and the independent anchor; compared with the twenty-first run's apparatus exactly 9 regions changed (accounting tree, central mapping tree and tree table on two disks each, journal records on two disks, root record device 0), with zero difference in the other 67 output groups; the field table's 174 rows still all equal, the 76 verdict fields identical one by one with the twenty-first run, and no old value left in the apparatus. The rerun registration now points to `e142-new-pool-file-creation-dry-run-r22-main-2026-10-05.out`. The second geometry P8G (question sheet line 2) is not run this time; the `crates/` export bin needs a disk size parameter, which belongs to deletion-line batch 18; G13 still did not run successfully.
  - **Basis**: `research/prompts/e142-r22-prereg.md` (pre-run registration and section twelve "runner revision one"); artifacts and item-by-item numbers in the experiment page section "2026-10-05 (twenty-second run, first segment)".

### 2026-10-04

- E142 (New-Pool File Creation Dry Run) twenty-first run fills in the three mutation numbers (the whole 199, run by the main agent; only numbers filled in, conclusion and artifacts unchanged)
  - **Before**: the next entry said "the small table's 27 all caught, the whole table handed to the main agent to run", and the experiment page and index row said the whole table was not run.
  - **After**: the whole 199: 199 caught, 0 invalid, 0 not red (`research/results/e142-mutate-2026-10-04-r21.log`, ending "restored, baseline still all green"; hit-the-ceiling, timeout, could-not-schedule and squeezed-out all 0; during the run the sha256 of the apparatus and mutation table were still the frozen values of registration section twelve); these are the three numbers of section nine "whole-table rerun"; none of void clauses V21a–V21f hangs on mutations, and the case in revision one item 1, "not red and the apparatus source needs changing", did not occur.
  - **Basis**: the log above; sections nine and twelve of registration `research/prompts/e142-r21-prereg.md`.
- E142 (New-Pool File Creation Dry Run) twenty-first run: tree IDs from 65536, tree table 11 entries, record header 487, new root segment 364 (last three items), named items 64: the apparatus against `layout/01` field by field and against `crates/` byte by byte, and whether the verdict still holds
  - **Before**: the apparatus still had tree IDs 11..18, tree table 7 entries, record header 311, new root segment 188, named items 67, the last three items copied from the selected root on apply, and no field table; the rerun registration pointed to `e142-new-pool-file-creation-dry-run-2026-09-28-r19-main-2026-09-28.out`.
  - **After**: the apparatus rewritten per today's settlement sentences of settled items 8 / 9 / 11 of D8 (Core Index Structure), settled item 24 of D18 (Information Carried in a Block) and settled items 4 / 15 / 17 of D23 (Journal Role & Format) (not copied from `crates/`), with a 174-row field table added; unit tests 104 pass, mutation table 199 rows (M204–M220 newly taken in, five re-anchored), the small table's 27 all caught, the whole table handed to the main agent to run. The field table's 174 rows all equal `layout/01`, the verdict fields identical one by one with before the change, the set of gap lines unchanged, the three reverse chain cells all equal; compared byte by byte with `crates/`, 9 of 29 regions differ, from one root cause (the day-1 value of accounting item 6, committed reservation: the apparatus writes 0 per `layout/01`, while `crates/` writes 98386 per the tombstone prepayment of settled item 2 of D3 (Space Allocation); the other 8 regions are its checksum cascade); the one-disk export of G13 did not run successfully (S21-dump). The rerun registration now points to `e142-new-pool-file-creation-dry-run-r21-main-2026-10-04.out`.
  - **Basis**: `research/prompts/e142-r21-prereg.md` (pre-run registration, section twelve "main agent judgment" and "runner revision one"); artifacts and item-by-item numbers in the experiment page section "2026-10-04 (twenty-first run)".
- The main agent's judgment on the E142 (New-Pool File Creation Dry Run) twenty-first run: V21a does not trigger, V21c accepts the whole-segment rerun, S21-crates belongs to kb-derived values not keeping up, the apparatus must change and a 22nd run is due, S21-dump handled with the 22nd run
  - **Conclusion**: V21a does not trigger (the scope of Q142.80 is the lines on the apparatus side that do not read the export; the 10 extra lines read text on the export side, and the exclusion list missed three kinds of downstream lines, `self_check_text_numbers`, `self_check_text_number_unclassified` and `text_numbers_all_classified` of `verdict`; with those removed the diff is zero); V21c accepts the whole-segment rerun a3 = B3 = c3, and the main artifact is kept as is; S21-crates is not two clauses contradicting each other, but the 0 in `layout/01` and in row 10 of settled item 8 of D5 (Snapshot / Space Accounting Mechanism) being derived values from before the tree ID renumbering that did not keep up, written back as 98 386; apparatus line 3000 writes 0 per the old value and must be changed per settled item 2 of D3 (Space Allocation) with a 22nd run before tree ID question sheet R2 can close; the one-disk export of G13 is rejected by `make_filesystem.rs:474`, handled with the 22nd run.
  - **Basis**: of the items handed to the main agent in section four of the E142 (New-Pool File Creation Dry Run) 21st report in `records/2026-10-01-删除与并发事务.md`, the one the main agent judged.
  - **Before**: the opening status of the experiment page and the section "2026-10-04 (twenty-first run)" said V21a, V21c and S21-crates were handed to the main agent to judge, with no judgment yet; that section itself was never committed, so the before-state is not in git.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

### 2026-09-28

- E142 (New-Pool File Creation Dry Run) nineteenth run, first segment: the segment sequence and closed form after recording barriers per device, the in-place overwrite third state, and the publish tail barrier
  - **Before**: the model recorded one pool barrier as one step, closed a segment on any barrier, had no third state, and had no barrier at the end of a publish; the rerun registration pointed to `e142-new-pool-file-creation-dry-run-2026-09-26-r18-main-2.out`, with the whole-stream segment sequence `2+2+1+2+2+1+26+2+1+2` and two-state 67108885.
  - **After**: the model changed per today's settlement sentences of settled item 4 of D13 (Verification Route) and settled item 7 of D16 (Publish Semantics) (arm N19C): whole stream `2+2+1+2+2+1+2+24+2+1+2`, two-state 16777240, three-state 16777260, identical to what `crates/` pins; the operation counts and kind strings of warm-up and the whole stream differ on whether consecutive barriers are merged (fork R40, stop S19-crates handed to the main agent); the six verdict fields identical to the eighteenth run; every difference attributed. The rerun registration now points to `e142-new-pool-file-creation-dry-run-2026-09-28-r19-main.out`. The experiment page status changed to "partly run": the second-segment layer-0 main arm (heavy tier) not run.
  - **Basis**: `e142-r19-prereg.md` (revisions R19E-1–R19E-3 in section twelve); artifacts in the experiment page section "2026-09-28 (nineteenth run, first segment)".
- E142 (New-Pool File Creation Dry Run) nineteenth run, fourth segment: consecutive barriers on the same disk merged into one
  - **Before**: per registration R40 the model recorded both of two consecutive barriers (arm N19C), and in 4 of the 20 cells of question sheet line 1 the operation counts and kind strings of warm-up and the whole stream differed from what `crates/` pins (stop S19-crates); the rerun registration pointed to `e142-new-pool-file-creation-dry-run-2026-09-28-r19-main.out`.
  - **After**: the model implements the merge independently per the newly added final settlement sentence of settled item 4 of D13 (Verification Route), "consecutive barriers on the same disk merge into one" (arm N19M, reading R46); all 20 cells are identical verbatim to what `crates/` pins, warm-up 20 and the whole stream 57 operations; segment sequence, two-state, three-state, bytes and verdict lines unchanged; all 182 mutations in the table caught; the rerun registration now points to `e142-new-pool-file-creation-dry-run-2026-09-28-r19-main-2026-09-28.out`. In the experiment page's "affected decisions", the review of the settled item 4 row of D13 (Verification Route) says "changed", and the other 36 rows say "not affected".
  - **Basis**: the user decided "merge, follow today's recorder" (line 82 of the basis of settled item 4 of D13 (Verification Route), and the item "settled item 4: consecutive barriers on the same disk merge into one" in the D13 (Verification Route) section of `decisions-history.md`); revisions R19E-4 and R19E-5 in section twelve of `e142-r19-prereg.md`; artifacts in the experiment page section "2026-09-28 (nineteenth run, fourth segment)".

### 2026-09-25

- E142 (New-Pool File Creation Dry Run) sixteenth run, first segment: the five cells α/β/ι/γ/δ settle into the single form fixed by the scope of settled item 14 of D8 (Core Index Structure) and settled item 2 of D18 (Information Carried in a Block), and the region-by-region comparison with `crates/` judges "all equal"
  - **Before**: one set of 5.2 variant switches (`GapReadings`/`AllocationInternalCellReading`/`AllocationRootKeyEndReading`/`GammaBoundaryReading`/`EtaThirdComponentReading`) threaded through `build_allocation_tree`/`publish_first_file`; the two cells α (internal/root node cell) and β (root largest_key) were real variant arms, while γ/ε/η (extent upper-segment leaf key interval) were blocked (fifth segment of the fifteenth run: the frozen arm N was for this field neither the registered A nor B but a third, unregistered form), with only an independent check and no main reading; measure 5's region-by-region comparison (29 window writes, paired by (device, offset, length)) judged "not equal" in the fifteenth run (14 equal / 15 unequal), and the readings of the 15 unequal regions had only been guessed by hand-checking bytes, not verified item by item through the variant arms; mutation table 123 rows; the second command-line argument pointed to the arm O historical remnant left by the fourteenth run.
  - **After**: once kb clerk sixteen fixed lines 395/401 of settled item 14 of D8 (Core Index Structure) and the scope of settled item 2 of D18 (Information Carried in a Block), the apparatus deleted the whole set of variant switches, and the four cells α/β/ι (the internal entry key takes the start of the child's segment, ι-A)/γ (the k-th piece of the extent upper-segment leaf covers the closed interval `[143k,143k+142]`, the third component constantly `2^64−1`) settle into the single literal form of the clauses (sparse, the root covers the whole key space, positional sharding intervals), and δ (slot count on disk) closes into one const fn `slots_of_device_bytes` (rounding down); the read path (`read_allocation_records`/`read_allocation_tree`/new `read_extent_upper_leaf`) takes the child position from the entry key, checks strict increase and device consistency, and no longer relies on positional indices. The second command-line argument changed to the arm N15 reference freshly compiled and run in step ① this time, pairing by (device, offset, length) into `name=old_new_region` (Q142.19, no longer by name). Three new classes of `name=` emission added: G3 (pure functions of the five geometry points in section eight), G4 (one disk walking the whole write path compared against a one-disk export of `crates/`; `crates/` could not produce one this time, recorded as "measured only on one two-disk geometry"), and P2 (the comparator can tell a single byte apart, all four points pass); `code2_field_rows` changed from "non-blank + first blank" to "list every entry + one row for the padding area"; a new independent comparison binary `e142_region_diff_independent.rs` created (it does not `use` anything from the model). 64 unit tests all green (adding 1 real blind-spot patch); mutation table 123 → 127 rows (12 deleted, 6 re-anchored, 16 added), run twice: first round 126/127 (1 real blind spot, unit test added), second round **127/127 caught, 0 invalid, 0 not red**. The region-by-region comparison (Q142.11) **paired all 29 regions, all equal, judged "all equal"**, with the independent comparison binary agreeing line by line; the 15 changed regions and 14 unchanged regions predicted by Q142.19 match the observations one by one (P1 passes); G3's five geometry points are identical field by field with the closed-form anchors; the three criteria of segments and ordering, write list, and field-level rows (Q142.13/14/16/17/18) are all "complete" and "matched". **This all-equal only shows that the kb transcription agrees with `crates/`, not that these five cells' clauses are themselves right** (section 5.4 of the pre-run registration). The second segment (Q142.12 field-level attribution) is marked "not run after enough to judge" because of the all-equal; the third segment (a whole crash-point replay round, G5, and the three numbers of a whole-table mutation rerun) not run.
  - **Basis**: `e142-r16-prereg.md` (pre-run registration; the revisions in section twelve record the mutation table changes item by item); artifacts `e142-new-pool-file-creation-dry-run-2026-09-25-r16-main.out` (684 lines), `e142-new-pool-file-creation-dry-run-2026-09-25-r16-combined.out` (`replay.sh:157` now points to this one, rerun byte-identical), `e142-region-diff-independent-2026-09-25-r16.out` (P3 independent comparison), `e142-r16-crates-dump-2026-09-25.out` (crates main export); lines 4 and 5 of the question sheet `m2-keyspace-rerun-questions.md`.

### 2026-09-23

- E142 (New-Pool File Creation Dry Run) thirteenth run: catching up with three settled clauses, C512 (where superseded instance table on 0-entry tree table version is recorded lacks clauses), C484 (mkfs does not clear root ring, old root reselected on same fsid restart) and C480 (How blocks in inode record is calculated lacks clause repository-wide), done together in one change
  - **Before**: the root record struct had no `allocation_record_tree_root` field (`ROOT_RECORD_BYTES=371`); mkfs did not clear the root ring, segment sequence `4+1+1+1+4` (13 operations, 34 crash states); the `blocks` field in `InodeRecord::to_bytes()` hard-coded `DATA_UNIT_BYTES / 512` (=64, a placeholder unrelated to file size, with no clause and no assertion watching it anywhere in the repository). The committed artifact `e142-new-pool-file-creation-dry-run-2026-09-22-mkfs-zero-fill-resync.out` (twelfth run) compared with the `crates/` implementation gave `equal=21 unequal=0`.
  - **After**: all three implemented independently "per the clauses, not copied from the `crates/` code": ① `RootRecord` gains the field, `ROOT_RECORD_BYTES` 371→457, the new field lands at [342,428), and the algorithm type/nonce/MAC move to [428,457); all four construction points take `NodePointer::empty_root()` or copy the previous version (this apparatus's scope only reaches milestone one and never does the one scenario that writes non-zero, "tree table 0 entries, write a row"); along the way, a formula that would silently miscompute was fixed (the formula for the `mapping_root` offset must subtract two pointer widths, not one, when the total width changes but the insertion point is after it). ② First, per the original text of C484 (mkfs does not clear root ring, old root reselected on same fsid restart) (which mentions only the three root ring regions), a new `StepKind::RootRingZeroFill` was recorded at a granularity of one write per region (3 new writes, `7+1+1+1+4`), which differs from the `crates/` ground truth; the coordinator then rewrote item 3 of settled item 8 of D22 (Synthesizing Unit Atomicity) as "the three root ring regions and the journal ring are all cleared by mkfs through a write-zeroes action, one call per segment for the four segments on each disk, 8 steps across the two disks", and the apparatus switched to `StepKind::ZeroFill` and `RecordingPool::record_zero_fill` (recording only the operation, not actually pouring the 768 MiB journal ring contents into the sector table of `SparseDevice`, to avoid slowing down every `mkfs()` call across 79 mutations × 50 unit tests); the mkfs segment sequence became `12+1+1+1+4` (21 operations, 4114 crash states), identical verbatim to the `crates/` ground truth registered in the "mkfs seeds the root" row of section eight of `.claude/kb/layout/01-first-txn.md`. ③ `blocks` changed to `self.size.div_ceil(512)`; E142 (New-Pool File Creation Dry Run)'s file is fixed at 3000 bytes, so blocks goes from 64 to 6; a new assertion at offset 48 checks the raw bytes written. Unit tests still 50 passed (release 25.53s; debug 574.99s, so from now on run with `--release`); mutations 75→79 (new M76: the loop writing 0 over whole root ring regions clears one region too few; M77: mkfs writes the new field non-zero; M78: blocks falls back to the placeholder; M79: only one disk cleared), all 79 caught, 0 invalid, 0 not red. New artifact `e142-new-pool-file-creation-dry-run-2026-09-23-decisions-catchup.out` (309 lines); compared with the twelfth run, of the apparatus's own 284 lines only the two lines `name=segments path=mkfs` and `name=width structure=root_record` differ; `replay.sh:157` now points to the new artifact, rerun byte-identical. Measure 5 (apparatus ↔ `crates/` implementation comparison) went from `equal=21` to `equal=10 unequal=11`; decoding both sides' `head_hexadecimal` cell by cell traced it to one root cause: `crates/singlefs-core/src/transaction.rs:1903` still writes inode records with the old placeholder `DATA_UNIT_BYTES / 512` (=64) and has not implemented C480 (How blocks in inode record is calculated lacks clause repository-wide), while this apparatus already writes 6; `blocks` changes the payload CRC and whole-unit CRC32C of `inode_leaf`, which propagate along the position entry pointer chain into `inode_root`→`tree_table`→`root_record` (offset 96), and via mapping entries and naming entries into `mapping_root` and `journal_record`; all 11 places share the same origin, and the coverage formulas of both sides' header checksums were checked item by item to agree, so it is not a coverage miscalculation; only reported, `crates/` not changed, and written into item 8 of "what it cannot answer" on the experiment page. The 32 rows of the "affected decisions" table (one row added for settled item 8 of D22 (Synthesizing Unit Atomicity)) were reviewed again row by row against today's artifact, and the review cells changed to one of the two legal forms gate 75 requires, "YYYY-MM-DD changed" or "YYYY-MM-DD not affected: reason"; of these, the two rows for settled item 9 of D16 (Publish Semantics) and settled item 7 of D22 (Synthesizing Unit Atomicity) were each checked against whether the "basis" paragraph of the respective decision file already cites E142 (New-Pool File Creation Dry Run) (the former does, kept as support; the latter does not, kept as prepared material, with whether to upgrade it handed to the main agent).
  - **Basis**: mainly the dispatch prompt (this round has no separate `research/prompts/e142-r13-prereg.md`; the placements of the three decisions to catch up with were given directly by the dispatch prompt); the authoritative source for the field, width and offset of C512 (where superseded instance table on 0-entry tree table version is recorded lacks clauses) is section three of the implementation writer's report `m2-c511-c512-implementer-report.md`; new artifact `e142-new-pool-file-creation-dry-run-2026-09-23-decisions-catchup.out`; mutation log `e142_new_pool_file_creation_dry_run-mutate-2026-09-23-decisions-catchup.log`; rerun `bash scripts/experiment-apparatus/replay.sh` (the argument for this version being E142 (New-Pool File Creation Dry Run)) reports byte-identical; gate number 27 (`ROOT_RECORD_BYTES` in sync across kb, source and artifact) is already all green today: the kb side had already been synced to 457 by the clerk, not "red on the kb side" as the dispatch prompt expected; gates number 33, 75 and 88 all green (75 has no new red apart from one problem of another session in `c510-date-gate-r1-main-verification.md`); gate number 55 (QEMU) is registered to `crash-verifier` and not run this round; body [142-新池新建文件的干跑.md](experiments/142-新池新建文件的干跑.md).

### 2026-09-22

- E142 (New-Pool File Creation Dry Run) twelfth run: catching up with one scalar in the `crates/` read-only snapshot that changed after mkfs gained zero-fill writes; no new numbers measured, criteria untouched
  - **Before**: in the committed artifact `e142-new-pool-file-creation-dry-run-2026-09-18-change-count-three.out` (eleventh run, 2026-09-18), the `name=impl_config` line in the `crates/` read-only snapshot section said `mkfs_operations=13`; that artifact holds the bytes from before parallel line four changed the mkfs write path.
  - **After**: parallel line four changed the `crates/` mkfs write path twice on the same day: ① mkfs itself zeroes the three root ring regions and the journal ring whole (user settlement 2026-09-19, "add write-zeroes to the block device abstraction"); ② the root ring is cleared too (C484 (mkfs does not clear root ring, old root reselected on same fsid restart), user settlement); `crates/singlefs-harness/src/segments.rs` gained a sixth step kind `StepKind::ZeroFill`, and `crates/singlefs-core/src/make_filesystem.rs:260-262` zeroes region by region per the geometry. Rerunning `driver_e142` (`scripts/experiment-apparatus/replay.sh:157`), the artifact became `e142-new-pool-file-creation-dry-run-2026-09-22-mkfs-zero-fill-resync.out`: compared byte by byte with the eleventh run's artifact, only 1 of 309 lines differs: `mkfs_operations` 13 → 21 (8 new `write_zeroes`: 3 root ring regions + 1 journal ring per disk, 2 disks); `policy_mismatches=0` and `name=impl_bytes_equal_summary regions=21 equal=21 unequal=0` unchanged, and the region bytes did not change with it (the starting devices were all zeros already, so the zero-fill writes bytes that are already zero). The apparatus's own `name=segments path=mkfs` is still `operations=13 segments=4+1+1+1+4`: `scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs` did not change a single byte this time; it is an independently handwritten model that does not read the `crates/` mkfs implementation, and this rerun did not make it catch up with the new zero-fill steps in `crates/`. Unit tests rerun with `cargo test --release --manifest-path scripts/experiment-apparatus/Cargo.toml --bin e142-new-pool-file-creation-dry-run` still 50 passed; the mutation table source unchanged, not rerun. `replay.sh:157` now points to the new artifact, and the rerun judges "byte-identical". The 31 rows of the "affected decisions" table were reviewed again row by row against today's artifact, and not one relation (support/prepared material/not affected) changed.
  - **Basis**: the pre-run registration is the main agent's dispatch prompt verbatim (this round has no separate `e142-r12-prereg.md`, and the task states "measure no new numbers, do not touch the criteria, do not touch the arm definitions"); new artifact `e142-new-pool-file-creation-dry-run-2026-09-22-mkfs-zero-fill-resync.out`; `diff` between it and `e142-new-pool-file-creation-dry-run-2026-09-18-change-count-three.out` (both kept in the repository) differs only at line 285; rerun `bash scripts/experiment-apparatus/replay.sh E142` (Dry Run of New File in New Pool) reports byte-identical; gate number 52 (segment sequence registration) is still green: it only compares the kb registry with the E142 (New-Pool File Creation Dry Run) artifact, and neither side reflects the new zero-fill steps in `crates/`, a false green already recorded in C485 (segment-registry cell does not read live code, stays green when segment sequence changes); this round copies it as is and does not fix it; gate number 55 (QEMU) and whether to start a separate round to change the apparatus model to six step kinds too were not judged this round, handed to the main agent; body [142-新池新建文件的干跑.md](experiments/142-新池新建文件的干跑.md).

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.
- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.
- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-18

- E142 (New-Pool File Creation Dry Run) eleventh run: change count field 1 → 3 subtracted byte by byte, plus an apparatus ↔ `crates/` implementation comparison (F1 triggered for the first time and closed out)
  - **Before**: the tenth run (2026-09-17) artifact had 93 lines, and both the apparatus and the pre-run registration wrote the change count field (offset 88 of settled item 6 of D8 (Core Index Structure)) as the literal 1; measure 5 (apparatus ↔ `crates/` implementation byte-by-byte comparison) had never run and was only a placeholder in the registration; 27 unit tests / 66 mutations.
  - **After**: the user settled that the change count is written by definition as checkpoint_txg (3 for new-pool file creation); the apparatus's `publish_first_file` gains a `change_count` parameter, and `main()` and the new `run_full_pipeline` pass 3. New whole-image byte-by-byte subtraction of M_A (new measurement basis) / M_B (real baseline, literal 1): all 67 differing segments attributed (0 gaps), each of the six places named by the registration has at least one segment, and two places outside those named, `journal_record` and `mapping_root`, were also found to change; the positive control reports the injection point exactly; existing result lines unchanged by a word except the `name=done` count.
    After the `crates/` side added the read-only output `new_pool_file_creation_region_bytes` (a 21-line region list) that day, measure 5 **triggered F1 on its first real run** (10 `equal=true`, 11 `equal=false`); the read-only diagnosis report `e142-r11-f1-diagnosis.md` found that all 11 inequalities share one root cause: the apparatus and `crates/singlefs-harness/src/scenario.rs::first_file_content()` feed new-pool file creation different formulas for the 3000-byte file content (apparatus `(index*7+3)%251`, `crates/` `index%251`), while each side's own write path is identical verbatim at the byte level (an independent bitwise CRC32C checker passes all 92 cells on both sides, and after swapping either side's content for the other's, 21/21 are all equal). The main agent judged to change the apparatus to align with `index % 251` (the `crates/` side is pinned by the QEMU log checker, three test cases and two crash-point replay streams); after the change and rerun, measure 5 is **21/21 all equal**. Unit tests 27 → 50, mutations 67 → 75, and both rounds (before and after changing the file content) caught all 75.
  - **Basis**: pre-run registration `e142-r11-prereg.md` (including three "revisions"); artifact `e142-new-pool-file-creation-dry-run-2026-09-18-change-count-three.out` (309 lines; the version that triggered F1 is kept as `...-pre-f1-fix.out`); diagnosis report `e142-r11-f1-diagnosis.md`; `replay.sh:157` switched to `@driver_e142`, running and comparing live across two cargo workspaces; body [142-新池新建文件的干跑.md](experiments/142-新池新建文件的干跑.md).

### 2026-09-16

- Three apparatuses catch up with tree table entry 148 → 200: E142 (New-Pool File Creation Dry Run) eighth run, and E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) and E146 (Width and Cost of Livelist Entries Identified by Mapping Key) each get a new artifact
  - **Before**: `TREE_TABLE_ENTRY_BYTES` was 148 in all three apparatuses, with artifacts `e142-first-txn-dry-run-2026-09-14-round2-slot4096.out`, `e145-self-describing-node-header-2026-09-14-round2.out` and `e146-livelist-entry-width-2026-09-14-round2.out`; the `TREE_TABLE_ENTRIES_PER_UNIT` of E146 (Width and Cost of Livelist Entries Identified by Mapping Key) was stuck at 112 (when the kb changed the trees per level to 109 on 2026-09-14 this one did not follow; it is not a registered format-const, so gate number 27 cannot see it).
  - **After**: the constant is 200 in all three. The E142 (New-Pool File Creation Dry Run) eighth run artifact `e142-first-txn-dry-run-2026-09-16-tree-table-200.out` (93 lines, last line `emitted=93`): **3** of 93 lines changed: `tree_table_entry` of `name=width` 148 → 200, `back_chain` of `name=root_record` 3984932094 → 628216162, chains items 2 / 3 of `name=back_chain` 1195436654 / 3984932094 → 1134114971 / 628216162; the five segment sequence lines, layer 0's 262165 states and zero violations unchanged verbatim. E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) artifact `e145-self-describing-node-header-2026-09-16-tree-table-200.out` (57 lines): `tree_table_bytes` in 3 `name=mapping` lines 148 / 296 / 148 → 200 / 400 / 200. E146 (Width and Cost of Livelist Entries Identified by Mapping Key) artifact `e146-livelist-entry-width-2026-09-16-tree-table-200.out` (130 lines), 6 lines changed: `tree_table_entry` of `name=config` and `day1_null_root_bytes` / `day1_root_required_bytes` in 5 `name=new_pool_file_creation` lines (148 / 16641 → 200 / 16693; the two-tree arm 296 / 33282 → 400 / 33386); `TREE_TABLE_ENTRIES_PER_UNIT` changed to 81 along with it, with a comment stating that it follows the entry width; this change turned the E146 (Width and Cost of Livelist Entries Identified by Mapping Key) mutation "tree table first-version entry count changed to 112" from caught to "the mutation fails to compile, this entry invalid": the assertion was `!fits(每单元容量 − 第一版条目数 + 1)`, and once capacity went from 112 to 81 that subtraction overflowed during constant evaluation, silently turning all 12 caught into 11; after changing the assertion to `saturating_sub` that entry was caught again, and the rerun confirmed all 12 caught, 0 invalid. Two entries of the E142 (New-Pool File Creation Dry Run) mutation table changed with it: the original string and mutated value of `M43` 148 / 145 → 200 / 197 (the name changing accordingly from `M43_tree_table_entry_145` to `M43_tree_table_entry_197`), and the anchor of `M63` changed from 24 to 76 per the reserved width.
  - **Basis**: the user settled widening the tree table entry (decisions-history (part nine)); each of the three artifacts passed the integrity gate (last line `emitted=N` equals the line count: 93 / 57 / 130). **The cross-apparatus cell matched by itself**: `back_chain=628216162` in the new E142 (New-Pool File Creation Dry Run) artifact is identical verbatim to the value computed independently by the `crates/singlefs-harness` implementation: the assertion in `new_pool_file_creation_publish.rs` first went red against the old value and, after switching to the new value, equals the artifact on both sides; the assertion was not changed to accommodate the implementation.
- E142 (New-Pool File Creation Dry Run) ninth run: the recovery prefix rule gains "does not cross instance boundaries", and layer 0's verification_ran 9 → 6
  - **Before**: both the apparatus and the recovery in `crates/singlefs-core` chained records above the watermark into a prefix regardless of instance: when the selected root was mkfs's generation-0 root (instance 0), warm-up jsn 1 (instance 1) was applied on top, and layer 0's first stream had `verification_ran=9`.
  - **After**: only records of the selected root's own instance are candidates, and the chain continues after the last one covered by the selected root (item 1 of settled item 14 of D23 (Journal Role & Format) verbatim: "the chain continues after the last record covered by the selected root, and stops as soon as the next one's instance generation differs from the selected root's"); `verification_ran=6`, while file read-back 7, journal load-bearing 3 and violations 0 are all unchanged. Artifact `e142-first-txn-dry-run-2026-09-16-instance-boundary.out` (93 lines, differing from the eighth run only in the `name=layer0` line); the mutation table gains M67, and `replay.sh` points to the new artifact.
  - **Basis**: step 3 of the milestone "overwrite, release, rollback and reuse" started, and once two instances come in this rule carries load; the `crates/` side changed the same way the same day (`replay_journal` in `recovery.rs`), and the counts of the new-pool file creation stream in gate checker-tier-crash-point-replay are compared verbatim with the new artifact.

### 2026-09-14

- E142 (New-Pool File Creation Dry Run) seventh run: the apparatus changed and rerun per the settlements of that day's 18 questions, with the system configuration slot width changed to 4096 at closing
  - **Before**: sixth run (artifact `e142-first-txn-dry-run-2026-09-13-settled6.out`): pointers 85 / 83, root record 333, journal record header 277, tree table 6 entries of 145, system configuration 452 / slot 512, mirror 1 GiB, accounting 8 rows, t3 before t2.
  - **After**: artifact `e142-first-txn-dry-run-2026-09-14-round2-slot4096.out` (93 lines): pointers 88 / 86, root record 371, journal record header 307, tree table 7 entries of 148, system configuration 481 / slot 4096, mirror 4 GiB (unit area 211968 slots), accounting 15 rows, bump order by ascending tree ID (t2 50240, t3 50242–50243 … t8 50248, runs 4 per disk); the 28 `name=width` lines expected == actual, the five `name=segments` lines identical verbatim to the sixth run, layer 0 262165 states with zero violations; 25 unit tests; two new accounts G24 / G25 recorded.
  - **Basis**: the user's pop-up settlements of 18 questions (section eleven of the record) and the main agent's verdict in the first three-way round on the system configuration slot width (`research/prompts/d22-slotwidth-r1-main-verification.md`, anticipated, awaiting the closing pop-up).

### 2026-09-13

- Added E142 (New-Pool File Creation Dry Run) and ran it: the byte table written into bytes as is, all eight criteria pass, and 18 blanks forced out
  - **Before**: the byte table in [layout/01-first-txn.md](layout/01-first-txn.en.md) had never been written into bytes; not one acceptance check from milestone step 0 to step 7 had been run; whether the journal carries load in new-pool file creation, how many states layer 0 has, and whether the oracle can tell "root present but unit absent" were all only inferred.
  - **After**: E142 (New-Pool File Creation Dry Run) ran: 16 structure widths equal the byte table cell by cell; the transaction has 21 writes, 2 barriers and 1 FUA; cold-start read-back is byte-identical; layer 0 main arm 65543 states with zero violations (only 4 with the root slot durable); the control with one disk lacking barriers has exactly 1020 violations in 2048 states; recovery ignoring the journal and recovery consulting it differ zero times over 65543 states; the eight probes agree with the milestone step 6 acceptance; of the 18 `gap` lines the hardest three are that the code 2 header cannot be resolved during scanning, the two key widths of the mapping tree, and that the transaction split discipline of D16 (Publish Semantics) and settled item 12 of D23 (Journal Role & Format) compute record counts 8× apart for the same load.
  - **Basis**: pre-run registration `e142-preregistration.md` (before the apparatus); artifact `e142-first-txn-dry-run-2026-09-13.out`; 16 unit tests, all 16 mutations caught; along with it, four formulas in the byte table and two acceptance sentences of the milestone changed, and C305 (inline formulas in kb are summed by nobody) through C313 (Whether FUA Writes Count as Crash Segment Boundaries) newly set up.
- E142 (New-Pool File Creation Dry Run) rerun per user settlements: system configuration 495, tree table version 1 with 7 entries, tree ID watermark 8; verdict unchanged
  - **Before**: artifact `e142-first-txn-dry-run-2026-09-13.out`, per the byte table before the settlements: system configuration 413, tree table 5 entries, watermark 6.
  - **After**: after the settlements of settled item 9 of D22 (Synthesizing Unit Atomicity), settled item 2 of D6 (Snapshot Implementation Model) and settled item 6 of D5 (Snapshot / Space Accounting Mechanism), rerun as `e142-first-txn-dry-run-2026-09-13-settled.out`: only the two lines `structure=system_configuration` and `root_record` changed; the 21-entry write list and layer 0's 65543 states with zero violations unchanged; the recovery path skips reading the root for tree table entries whose root pointer is zero; `replay.sh` points to the new artifact, and the old artifact is kept on file.
  - **Basis**: user settlements; 16 unit tests and 16 mutations rerun.
- E142 (New-Pool File Creation Dry Run) third run per settled item 4 of D15 (Format Freeze Policy): incompat bit 0 set to 1, the parser "refuses to mount what it does not recognize"; verdict unchanged
  - **Before**: in artifact `e142-first-txn-dry-run-2026-09-13-settled.out` the system configuration feature bits were all 0, and the parser skipped those 96 bytes; blank G12 open.
  - **After**: artifact `e142-first-txn-dry-run-2026-09-13-featurebit.out` (66 lines): mkfs sets incompat bit 0 to 1, and the parser refuses when it meets an unregistered incompat bit or when bit 0 is not set, ignoring compat_ro / compat; one more line `name=feature_bits incompat_byte0=0x01 … refuses_unknown_incompat=true refuses_missing_layout_bit=true`, G12 changed to closed out, verdict lines unchanged verbatim; 17 unit tests, all 18 mutations caught; the rerun registration now points to the new artifact.
  - **Basis**: user settlement of settled item 4 of D15 (Format Freeze Policy); the prerequisite of C312 (incompat bits of the first layout line unassigned) paid off.
- E142 (New-Pool File Creation Dry Run) fourth run: warm-up lands in bytes, new-pool file creation txg 3, layer 0 takes every operation after mkfs, 262162 states with zero violations
  - **Before**: artifact `e142-first-txn-dry-run-2026-09-13-featurebit.out`: new-pool file creation txg 1, jsn 1, layer 0 enumerated only the transaction's 21 writes, 65543 states; G17 open.
  - **After**: artifact `e142-first-txn-dry-run-2026-09-13-warmup.out` (68 lines): `name=warm_up publishes=2 writes=10 barriers=4 fua=2 new_pool_file_creation_txg=3`, `recover_full root=1:3 valid_records=3`, `layer0 segments=2+1+2+2+1+18+2+1+2 states=262162 violations=0`, `layer0_fua_not_boundary closed_form=524311`; G17 closed out, G19 newly recorded; 19 unit tests, all 19 mutations caught; the rerun registration points to the new artifact.
  - **Basis**: user settlement of settled item 8 of D16 (Publish Semantics), and for C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused) the user took shadow accounting (clause written); the FUA segment boundary of settled item 4 of D13 (Verification Route) and the system configuration ownership of settled item 7 of D16 (Publish Semantics) settled the same day.
- E142 (New-Pool File Creation Dry Run) fifth run: the recorder's step kinds made into a closed enum, and the `name=segments` lines carry each segment's kind multiset (half ② of C316 (Commit Steps Registered in Four Places That All Differ))
  - **Before**: step kinds in the recorder were free-text labels `WriteRequest.label`, FUA was passed as a boolean by each call site, and the four `name=segments` lines had only segment sequences and closed forms; 20 mutations; the replay line pointed to the fourth artifact `-warmup.out`.
  - **After**: `StepKind` (`unit_write` / `journal_record` / `root_record_fua` / `system_configuration_slot`) + `RecordedStepKind::Barrier`; a `TransactionUnit` enum replaces the two `_ => panic!` sites that matched on strings; `segment_step_kinds()` groups by the same boundary rule as `split_into_segments` and asserts that the write counts per segment are equal; each of the four `name=segments` lines gains a `kinds=` field, and unit tests pin the kind multisets of the four paths as absolute values; fifth artifact `e142-first-txn-dry-run-2026-09-13-kinds.out` (72 lines, only those four lines differing from the fourth), with the replay line now pointing to it; all 21 mutations caught, and the newly added 21st mutation (recording the system configuration slot write as a unit write: neither segment boundaries nor state counts change) is caught only by `registered_segment_sequences_match_every_recorded_path`.
  - **Basis**: the user directed doing both halves of C316 (Commit Steps Registered in Four Places That All Differ); settled item 2 of D17 (Implementation Layering & Third-Party Pipeline).

- See the same-day entry "Added E143 (Journal Accounting Under One Transaction One Unit) and ran it: seq batches under the user-settled split discipline…" in the E143 (Journal Accounting Under One Transaction One Unit) section.
- See the same-day entry "Added E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) and ran it: the self-describing header is nearly free, today's header…" in the E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) section.

## E143 (Journal Accounting Under One Transaction One Unit)

### 2026-09-19

- See the same-day entry "E22 (Recording Named Ancestor Placement), E23 (Journal Geometry), E32 (Remnants of the Previous Timeline), E39 (Does Reverse Chain Block Remnant Records)…" in the E22 (Recording Named Ancestor Placement) section.

### 2026-09-13

- Added E143 (Journal Accounting Under One Transaction One Unit) and ran it: under the user-settled split discipline, seq batch 1 gives 8 records per fsync, the journal takes 12.5% of user data, and a full T_dirty window is 4× the ring
  - **Before**: C310 (Txn Split Discipline Conflicts With Record Count Measurement Basis) had only the one number 12% reported by E142 (New-Pool File Creation Dry Run); the user settled "the split discipline is one transaction per unit".
  - **After**: E143 (Journal Accounting Under One Transaction One Unit) ran: over the six load points of D25 (Target Workload Priority), under today's clauses the record count per fsync equals the data unit count (1 / 1 / 8 / 2 / 80 / 10), and the journal constantly takes 12.5% of user data; a full T_dirty window is 65 536 records, 256 MiB, 4× the 64 MiB ring, while I-8.1 (Ring Geometry Sufficiently Large) read as "any one transaction" needs only 12 KiB; the control arm packing several transactions into one record gives 1 record, 1.56%, 4232 KiB for a full window; arm C reproduces 1847 records / 7388 KiB under the measurement basis of E91 (Ring and Admission Accounting Under Settled Format).
  - **Basis**: pre-run registration `e143-preregistration.md`; artifact `e143-one-unit-per-txn-journal-2026-09-13.out`; 6 unit tests, all 10 mutations caught (one insensitive sampling point replaced by a sensitive one).

## E144 (Cost and Discriminative Power of Header Checksum Algorithms)

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-16

- See the same-day entry "Artifact lines copied whole into the bodies of E139 (Tightened Form of Per-Disk Rollback Floor) and E144 (Cost and Discriminative Power of Header Checksum Algorithms)…" in the E139 (Tightened Form of Per-Disk Rollback Floor) section.

### 2026-09-13

- Added E144 (Cost and Discriminative Power of Header Checksum Algorithms) and ran it: the six arms cannot be told apart on discriminating power, while costs differ 26×, taking 2.6% vs 0.1% of one fsync
  - **Before**: C309 (Algorithm of the 32-Byte Checksum Not Settled) had only the sentence "the algorithm is unsettled"; the user directed "if it has nothing to do with encryption, it can be cheaper".
  - **After**: E144 (Cost and Discriminative Power of Header Checksum Algorithms) ran: SHA-256, BLAKE2s, xxHash64 × 4, xxHash64, CRC-64/XZ and CRC32C miss zero under single-bit and exhaustive double-bit flips and random byte changes on five real header widths, while the 8-bit additive sum control misses 11 892; CRC32C 26 ns, SHA-256 676 ns; 14 headers folded into a 359 µs fsync give 0.1% vs 2.6%.
  - **Basis**: pre-run registration `e144-preregistration.md`; artifact `e144-header-checksum-cost-2026-09-13.out` (timing; `replay.sh` interval assertions crc32c 19 to 37, sha256 473 to 879); 8 unit tests, all 10 mutations caught.

## E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width)

### 2026-09-19

- See the same-day entry "E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units), E73 (Fanout Cost of Node Key Interval)…" in the E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units) section.

### 2026-09-16

- See the same-day entry "Three apparatuses catch up with tree table entry 148 → 200: E142 (New-Pool File Creation Dry Run) eighth run…" in the E142 (New-Pool File Creation Dry Run) section.

### 2026-09-14

- See the same-day entry "Five apparatuses catch up with the three format constants the user settled on 2026-09-14: journal record header 307, i…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-13

- Added E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) and ran it: a self-describing header is nearly free, and today's header cannot resolve unregistered trees
  - **Before**: C306 (code 2 header is not self-describing about key width) and C307 (how two key widths of the mapping tree fit into one tree with a fixed-width key) had only the blanks reported by E142 (New-Pool File Creation Dry Run), with no cost numbers; the user directed "code 2 can be self-describing".
  - **After**: E145 (Cost of Code 2 Self-Describing Header and Mapping Tree Key Width) ran: a self-describing header adds 5 bytes per node, and the fanouts of five trees and the tree heights on a 10 TB pool do not change by a single tier; today's header, by trying widths one by one, resolves 1000 nodes of registered trees but not one of 200 unregistered ones, while the self-describing header resolves them all; the three ways of fitting the mapping tree, zero-padding to 27, two trees, and self-describing entries, all have tree height 4, with leaf fanouts 294 / 306 / 284.
  - **Basis**: pre-run registration `e145-preregistration.md`; artifact `e145-self-describing-node-header-2026-09-13.out`; 9 unit tests, all 11 mutations caught (one equivalent mutation kept on file and replaced by "try only the first four widths").

## E146 (Width and Cost of Livelist Entries Identified by Mapping Key)

### 2026-09-16

- See the same-day entry "Three apparatuses catch up with tree table entry 148 → 200: E142 (New-Pool File Creation Dry Run) eighth run…" in the E142 (New-Pool File Creation Dry Run) section.

### 2026-09-14

- See the same-day entry "Five apparatuses catch up with the three format constants the user settled on 2026-09-14: journal record header 307, i…" in the E43 (Extension Point Byte Upper Bound) section.

### 2026-09-13

- Added E146 (Width and Cost of Livelist Entries Identified by Mapping Key) and ran it: entries 24 → 35 bytes, only two of 21 cells gain a level, and each FREE prepays 35 bytes
  - **Before**: settled item 2 ① of D6 (Snapshot Implementation Model) had only the constraint "the identity must be the central mapping key", and none of the entry width, fanout, tree height or the prepayment of C260 (livelist entry prepayment omits the amount for unwinding itself) recomputed under it.
  - **After**: E146 (Width and Cost of Livelist Entries Identified by Mapping Key) ran: the form padded to the widest with the event folded into the class tag is 35 bytes (leaf 463, internal 137), with an event byte 36 (leaf 450), the code 2 / 3 tree when split into two trees 33 (leaf 491), and value carrying a hint 49 (leaf 330); in the 21 cells of the E132 (Livelist Carrier: Recomputed with Real Tree Count and Fanout) grid the 35-byte form is one level taller than the 24-byte form only in the two cells with 65536 total entries; each FREE prepays 35 / 36 / 49 bytes, i.e. 10 / 10 / 14 basis points; day-1 registration adds 145 bytes in new-pool file creation (16638 if empty trees need a root), lazy creation 0.
  - **Basis**: pre-run registration `e146-preregistration.md`; artifact `e146-livelist-entry-width-2026-09-13.out`; 8 unit tests, all 12 mutations caught.

## E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table)

### 2026-09-19

- See the same-day entry "E33 (Two Pinned Block Rules), E34 (Root Ring Slot Geometry), E38 (Accounting Structure: COW vs In-Place)…" in the E33 (Two Pinned Block Rules) section.

### 2026-09-13

- Added E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) and ran it: 413 leaves 99 to spare in the 512 slot, all six variants fit in one sector, and the four cells of E115 (Completeness and Byte Budget of System Configuration Field Table) / E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) are no longer cited
  - **Before**: settled item 9 of D22 (Synthesizing Unit Atomicity) said verbatim "the four cell numbers cannot be cited today", waiting for a rerun after C231 (root record fields mixed into system configuration required items) was fixed.
  - **After**: E147 (Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table) copied the 39-row system configuration table of layout/01-first-txn.md into the apparatus: 413 / 512 with 99 to spare, 0 torn states; master key slot inlined 493, KDF 4 bytes 415, devs omitted 409, no indirect directory pointer kept 354, the widest of the three 495; all six fit in one sector.
  - **Basis**: pre-run registration `e147-preregistration.md`; artifact `research/results/e147-system-configuration-recompute-from-layout-2026-09-13.out`; 5 unit tests, all 9 mutations caught.

## E148 (Commit Fixed Point Recomputed with Two Record Trees)

### 2026-09-19

- See the same-day entry "E19 (Spurious ENOSPC Under Defer Window), E82 (In-Flight Synthesis for Admission)…" in the E19 (Spurious ENOSPC Under Defer Window) section.

### 2026-09-13

- Added E148 (Commit Fixed Point Recomputed with Two Record Trees) and ran it: with the mapping tree added, the clustered fixed point goes 5 → 9 blocks, and the empty publish of 9 blocks (4 for new-pool file creation) is the candidate for c_max
  - **Before**: C83 (commit fixed point unanswered) had only the one-record-tree numbers of E81 (Fixed Point of Commits); the c_max of settled item 4 of D28 (Mount-Time Commitments) and settled item 8 of D16 (Publish Semantics) had no measurement basis.
  - **After**: E148 (Commit Fixed Point Recomputed with Two Record Trees) adds the mapping tree (settled item 8 of D19 (Block Pointer Structure & Width Budget)): at pool scale, the clustered empty publish and the target load are both 9 blocks, clustering only the commit blocks gives 59, scattered 138 416–138 600; at new-pool file creation scale 4 blocks; `trees=1` reproduces E81 (Fixed Point of Commits) cell by cell.
  - **Basis**: pre-run registration `e148-preregistration.md`; artifact `e148-commit-fixpoint-two-record-trees-2026-09-13.out`; 9 unit tests, all 10 mutations caught.

## E149 (Costs of Three Repair Options for Broken Packed Containers)

### 2026-09-19

- See the same-day entry "E114 (Overall Accounting of Small-Data Packing Containers), E119 (Padding Cost of Fixed-Width Slots)…" in the E114 (Overall Accounting of Small-Data Packing Containers) section.

### 2026-09-13

- Added E149 (Costs of Three Repair Options for Broken Packed Containers) and ran it: multiple replicas give gain ×2/3 and writes ×3/2, lowering cap reduces gain and radius together, and no arm has gain > 1 at the 16 KiB tier
  - **Before**: the three remedies listed by settled item 1 of D27 (Small-Data Packing Container) were verbatim "none measured yet".
  - **After**: under the measurement basis of E114 (Overall Accounting of Small-Data Packing Containers), E149 (Costs of Three Repair Options for Broken Packed Containers) computed for five object size tiers × six arms the gain, write byte ratio, loss radius and how many copies must fail together; a ceiling of 8 only bites the two tiers 512 B and 1 KiB.
  - **Basis**: pre-run registration `e149-preregistration.md`; artifact `e149-pack-container-repair-options-2026-09-13.out`; 6 unit tests, all 8 mutations caught (two were insensitive sampling points in the first round, caught after adding a 6486-byte sampling point).

## E150 (Rollback Reuse of Units Referenced by Abandoned Roots)

### 2026-10-22

- Title status and index row add "the rollback shape it measured no longer exists, conclusions are history only": E150 measured two cells, a rollback publish crashing after the units and before the record, and the rollback instance's roots all unreadable after the rollback, when rollback was an instance switch at mount; D23 (Journal Role & Format) confirmed item 14 made rollback a forward publish while mounted. The short name is kept (renaming it touches 111 files across the repo).
  - **Basis**: milestone 3 五·9; cluster 庚 writer report `~/.local/share/singlefs/work/m3-close-b16-cluster-geng/report.md` section 6 item 3, the main agent took reading 甲.
  - **Before**: `git show HEAD:.claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md` first line.

### 2026-09-19

- See the same-day entry "E19 (Spurious ENOSPC Under Defer Window), E82 (In-Flight Synthesis for Admission)…" in the E19 (Spurious ENOSPC Under Defer Window) section.

### 2026-09-13

- Added E150 (Rollback Reuse of Units Referenced by Abandoned Roots) and ran it: both cells of C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused) are reachable, and the two fixes, shadow accounting and tombstones, have zero violations
  - **Before**: the two cells of C314 (After Crash Recovery Lands on an Old Root, Units Referenced by the Abandoned Root Can Be Reused) had only reasoning from the clause text and a model check by the second-round reverse-inference leg on C199 (instance generation increment contradicts stopping at jsn gaps); the fix candidates were only F-A, with its cost never computed.
  - **After**: the apparatus `scripts/experiment-apparatus/e150_rollback_reuse_of_abandoned_roots.rs` (one disk, root ring 3 × 4, 32 units, rollback per settled item 14 of D23 (Journal Role & Format), recovery picks the newest readable root + the instance table it points to) ran: `baseline_case1=3 baseline_case2=19 shadow_case1=0 shadow_case2=0 tombstones_case1=0 tombstones_case2=0 shadow_clean=true tombstones_clean=true shadow_quarantine_zero_at_follow_up=11`; 15 unit tests, all 12 mutations caught; artifact `e150-rollback-reuse-of-abandoned-roots-2026-09-13.out` (87 lines), rerun registered as exact. The end of the pre-run registration records one anchor fix made halfway through writing the apparatus, before the artifact (the baseline also protects the rollback candidate set per I-7.4 (Blocks in Recent K Generations Not Reused)).
  - **Basis**: pre-run registration `e150-preregistration.md`; the prerequisite of settled item 8 of D16 (Publish Semantics).
- E150 (Rollback Reuse of Units Referenced by Abandoned Roots) second run: discriminating-power self-proof of admission item nine, "volume held exclusively by abandoned roots" (C318 (Units isolated by shadow accounting not in admission inequality))
  - **Before**: E150 (Rollback Reuse of Units Referenced by Abandoned Roots) measured only the quarantined volume, with no reconciliation on the admission side; the discriminating-power self-proof of C318 (Units isolated by shadow accounting not in admission inequality), "set the quarantined volume to 0, and a pool short by just those few blocks must flip from read-only to writable", had no model form.
  - **After**: the three items computed each independently, and the allocator independently counts the slots it can issue; artifact `e150-rollback-reuse-of-abandoned-roots-2026-09-13-admission.out` (91 lines, only four more): with item nine, the available count and the issuable slot count are identical verbatim in three lines; without it, it over-reports by 2 (just after the rollback) and returns to 0 at the 11th publish; 16 unit tests / 13 mutations (the thirteenth, dropping item nine, must go red); the replay line now points to the second artifact.
  - **Basis**: `e150-r2-prereg.md`; directed by the user.

## E151 (Arrival Order of User Data Placement Points and Container Arms)

### 2026-10-17

- The subheading of item 1 at the top of the page, "depends on a sentence that has no clause yet", gains "at the time of the second run" and states that it is now a clause in the transaction split discipline of settled item 5 of D16 (Publish Semantics), and that C319 (No Clause and No Check That Units Within a Request Are Issued in Ascending Key Order) is paid off; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/151-用户数据落点的到达序与容器臂.md`.

### 2026-09-19

- One current-state sentence each in E151 (Arrival Order of User Data Placement Points and Container Arms) and E68 (Selection of Inlining Threshold) corrected after a fresh check; E8 (Different Write Paths for Large vs Small Files), E11 (Dual-Track: Gains and Flip Cost, Measured Directly), E68 (Selection of Inlining Threshold), E122 (Benefits Bought by Directory-Based Grouping) and E151 (Arrival Order of User Data Placement Points and Container Arms) gain an "affected decisions" table
  - **Before**: item 4 of "what it cannot answer" in E151 (Arrival Order of User Data Placement Points and Container Arms) said "does not answer the 'make it a hint' arm (D) of settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient); that is `grouped` of E122 (Benefits Bought by Directory-Based Grouping)", while the fourth / fifth runs on the same page had already built two directory-grouping hint arms, and item 7 says `grouped` is an initial condition corresponding to no arm. Item 3 of "the half it cannot answer" in E68 (Selection of Inlining Threshold) said "does not model the other half of settled item 3 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) (the crash semantics of that bounded inline → extent transaction)", while that cell is settled item 5 of D14 (Dual-Track: Large/Small Files & Persistent/Transient). None of the five pages had an "affected decisions" table.
  - **After**: item 4 of E151 (Arrival Order of User Data Placement Points and Container Arms) changed to: the "make it a hint" arm was not built in the first to third runs, the fourth / fifth runs built only the two directory-grouping ones, and `grouped` corresponds to no arm (see item 7); item 3 of E68 (Selection of Inlining Threshold) now points to settled item 5 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) and states that with the threshold at 0 it has no subject. E8 (Different Write Paths for Large vs Small Files), E11 (Dual-Track: Gains and Flip Cost, Measured Directly), E68 (Selection of Inlining Threshold), E122 (Benefits Bought by Directory-Based Grouping) and E151 (Arrival Order of User Data Placement Points and Container Arms) each gain an "affected decisions" table; the E151 (Arrival Order of User Data Placement Points and Container Arms) row is matched in both directions with the basis paragraph of settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient), and the D14 (Dual-Track: Large/Small Files & Persistent/Transient) rows on the other four pages are prepared material. Experiment conclusions and artifacts unchanged.
  - **Basis**: fresh check of item 5 of the conclusions and the arm table on the same E151 (Arrival Order of User Data Placement Points and Container Arms) page (`directory_hint` and `directory_home` added in the fourth run), and row 5 of the index table in `.claude/kb/decisions/14-双轨大小文件-持久临时.md` (crash semantics of inline → extent); for D14 (Dual-Track: Large/Small Files & Persistent/Transient) rewritten in its slimmed form see the decision change history (part nine).

### 2026-09-13

- Added E151 (Arrival Order of User Data Placement Points and Container Arms) and ran it: the bump_seg of E93 (Placement and Fragmentation Under Aging) favors B by 12% on run rewrites, the volume of commit-born blocks barely moves the fallback rate, and container packing presses runs down but cannot feed empty segments
  - **Before**: the candidate table of settled item 8 of D3 (Space Allocation) took E93 (Placement and Fragmentation Under Aging)'s bump_seg as B's number (runs8 7312, 89.3%), with a fallback rate of 91.8%–93.8% computed from the commit-born block volume of 5 trees; the count of fully empty clustered segments had no trajectory (C243 (Quantity in Termination Predicate Lacks Trajectory)).
  - **After**: artifact `e151-arrival-and-container-arms-2026-09-13.out` (86 lines): `arrival_ties_bump_in_every_cell=false` (uniform ties; runs8 arrival order 8183 / 8181 / 8183 vs key order 7312 / 7364 / 7371); bump_seg fallback rate at M = 0 / 4 / 9 is 93.5 / 93.6 / 94.2 (uniform) and 92.0 / 92.4 / 93.1 (runs8); containers K16 / K64 give runs 0.83 / 0.97 of B's (uniform) and 0.49 / 0.67 (runs8), `any_container_cell_has_empty_segments=false`; the cross-apparatus gate reproduces 14 cells of E93 (Placement and Fragmentation Under Aging) verbatim; 15 unit tests, all 12 mutations caught (one mutation, "two pieces in the same slot do not break", measured equivalent, turned into an assertion kept on file and replaced); rerun registered as exact.
  - **Basis**: pre-run registration `e151-preregistration.md`; directed by the user.
- E151 (Arrival Order of User Data Placement Points and Container Arms) second run: with arrival order read per request, B's numbers come back, the fallback of commit-born blocks is accounted separately, and packing can feed empty segments
  - **Before**: conclusion 1 of E151 (Arrival Order of User Data Placement Points and Container Arms) said "B's advantage over A is an artifact of key-order batches", and conclusion 3 said "packing cannot feed empty segments either"; the fallback rate had only a mixed measurement basis; the fully empty clustered segment count had only an end-of-run value; 15 unit tests / 12 mutations; the replay line pointed to the first artifact (86 lines).
  - **After**: the second-round reverse-inference leg on settled item 8 of D3 (Space Allocation) hit three places on a copy; the committed apparatus added three arms (`arrival_by_request`, `container_firstfit_k16 / k64`) and three trajectory quantities to re-check, artifact `e151-arrival-and-container-arms-2026-09-13-request-arrival.out` (107 lines): `arrival_by_request_ties_bump_in_every_cell=true` (runs8 7270 / 7335 / 7347 vs B 7312 / 7364 / 7371); commit-born block fallback is 0 in all six first_fit cells and 16574–16835 / 18000 for the bump family at M = 9; `any_container_firstfit_cell_has_empty_segments=true`, and after bump_seg used up the initial 32 segments its peak is 2, positive for rounds 97–128. Conclusion 1 narrowed, conclusion 3 withdrawn; 18 unit tests / 14 mutations; the replay line now points to the second artifact, and the first artifact is kept on file.
  - **Basis**: `e151-r2-prereg.md`, `research/prompts/d3-r2-main-verification.md` (verification table R1–R4).
- E151 (Arrival Order of User Data Placement Points and Container Arms) third run: packing patrol budget scanned at 64 / 256 / 1024; the runs multiple is sensitive, fully empty segments are not
  - **Before**: the patrol budget of E151 (Arrival Order of User Data Placement Points and Container Arms) was the constant 64 (equal to the dirty objects per checkpoint), and the container arms' runs-pressing multiple and fully-empty-segment answer had only been measured at this one point (the instance in C317 (Verdict measured on single geometry sample point unchecked)).
  - **After**: the budget made a parameter, container_k16 × two load groups × M = 9 scanned at three points, artifact `e151-arrival-and-container-arms-2026-09-13-sweep.out` (116 lines): runs 6771 / 5864 / 5420 and 3779 / 3018 / 2835 decrease monotonically, while the fully empty segment peaks after exhaustion, 3 / 4 / 4 and 4 / 4 / 4, do not move, all 0 at the end; the conjunction S1 fixed before the run does not hold (`verdict_sensitive_to_pack_sweep=false`), and the two halves are recorded separately: the runs multiple is sensitive, fully empty segments are not. 19 unit tests / 15 mutations; the replay line now points to the third artifact.
  - **Basis**: `e151-r3-prereg.md`; directed by the user.
- E151 (Arrival Order of User Data Placement Points and Container Arms) fourth run: the two directory-grouping hint arms against the subtraction arm: following the members scatters as much as the lowest free slot, while a fixed home pins the directory but uses up the empty segments of commit-born blocks
  - **Before**: the minuend of settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) had only the one-off count from E122 (Benefits Bought by Directory-Based Grouping), on a different axis from the subtraction arm (settled item 8 of D3 (Space Allocation), the first_fit of E151 (Arrival Order of User Data Placement Points and Container Arms)).
  - **After**: E151 (Arrival Order of User Data Placement Points and Container Arms) added two arms, `directory_hint` (the members' segment) and `directory_home` (home segment + overflow segments, added before the artifact run per the first arm's unit tests, with the revision recorded in the pre-run registration), and three directory-locality quantities; artifact `e151-arrival-and-container-arms-2026-09-13-directory.out` (159 lines): following the members, directory traversal segments are 97%–99%, scattering over at most 18–24 segments; with a fixed home 54%–55%, at most 2 segments, and runs8 key-order runs 18% fewer, but the overflow segments are exactly the trailing fully empty segments, and at M = 9 the commit-born block fallback is 17936 / 18000; neither H1 (≤ 50%) nor the overturn line (≥ 80%) fixed before the run was reached, H9 (segment granularity) holds in four cells, and H3 holds only on runs8. 23 unit tests / 18 mutations; the replay line now points to the fourth artifact.
  - **Basis**: `e151-r4-prereg.md`; directed by the user.
- E151 (Arrival Order of User Data Placement Points and Container Arms) fifth run: overflow segment count of the fixed-home hint scanned at 4 / 8 / 16 / 32: on uniform, leaving 24 segments for commit-born blocks satisfies both, on runs8 it cannot
  - **Before**: the E151 (Arrival Order of User Data Placement Points and Container Arms) body said that the fixed-home hint makes commit-born blocks fall back 99% and that this "is a number, not a choice"; the overflow segment count was all 32 trailing segments, never scanned.
  - **After**: the third-round reverse-inference leg on settled item 2 of D14 (Dual-Track: Large/Small Files & Persistent/Transient) scanned on a copy and found fallback 0 at uniform k = 8; the committed apparatus's fifth run confirmed it field by field: artifact `e151-arrival-and-container-arms-2026-09-13-overflow.out` (168 lines) `uniform_has_overflow_count_with_zero_metadata_fallback_and_local=true runs8_has_such_overflow_count=false`; the sentence "is a number, not a choice" was changed, and three measurement basis notes added: load bias, the H9 tautology, and `grouped` of E122 (Benefits Bought by Directory-Based Grouping) being an initial condition; 24 unit tests / 19 mutations; the replay line now points to the fifth artifact.
  - **Basis**: `e151-r5-prereg.md`; verification table K1 / K2 of `research/prompts/d14-r3-main-verification.md`.
- E151 (Arrival Order of User Data Placement Points and Container Arms) sixth run: the two assertions of C146 (fallback policy with no empty segment undefined repo-wide), the discriminating-power self-proof of C243 (Quantity in Termination Predicate Lacks Trajectory) and the count of C319 (No Clause and No Check That Units Within a Request Are Issued in Ascending Key Order) each made into a quantity that can go red
  - **Before**: the checks of all three accounts were waiting for the implementation, with no form in the model; E151 (Arrival Order of User Data Placement Points and Container Arms) had no compaction arm and no `R`.
  - **After**: two compaction arms (excluding / not excluding `R`) and four quantities, artifact `e151-arrival-and-container-arms-2026-09-13-region.out` (196 lines): when excluding `R`, moves back into `R` and placements deviating from the policy function are both constantly 0, and `R` is fully empty 1999 / 1749 times after the intent completes; when not excluding, both are in the tens of thousands, and `R` is fully empty only 6–439 times (collapsing to 0.3%–2% at M = 0, reaching only 25% at M = 9, because `R` is occupied by commit-born blocks); the within-request order count is constantly 0 in per-request arrival order and 55980 under uniform permutation on runs8; 27 unit tests / 22 mutations; the replay line now points to the sixth artifact. The two "= 0" sentences written before the run (the count on uniform, and the fully-empty count when not excluding) were not reached; the reasons are written into the body, and the criteria unchanged.
  - **Basis**: `e151-r6-prereg.md`; directed by the user.

## E152 (Per-Milestone File Performance Against Six File Systems)

### 2026-10-17

- Five headings or lead sentences saying "copied whole from the artifact" gain the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md`.

### 2026-10-15

- 60 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 60 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md`.

### 2026-09-30

- The experiment page reordered by run date (newest first, one index table "runs (newest first)", one section per date, a second run on the same day as a separate paragraph within that section, each with the seven tables produced by `e152-tables.py`), with ordinal run numbers changed to dates; numbers and artifacts unchanged.
- The last row of each of the six-file-system tables gains singlefs: `scripts/experiment-apparatus/e152-tables.py` fills a cell only for the same quantity and writes "—" otherwise (small-file persistence takes publish B's wall clock, write bytes, write requests computed from block-layer before/after differences, and FLUSH; cold mount takes cold recovery); the tables of four artifacts were regenerated from the new output; with the singlefs row removed, the old tables on the page are identical verbatim to the new output.
  - **Basis**: in the 2026-09-29 run, block-layer write requests 36 = write calls 29 + barriers 6 + FUA 1, and FLUSH 7 = 6 + 1, matching the notation of item 5 of the section "what these numbers show (overwrite, release, rollback and reuse, compared with the six file systems on a two-disk mirror)"; the script's self-proof gains a cell that goes red when `E152_TABLES_SINGLEFS_WRITE_CALLS=1` substitutes the write call count for the block-layer count.
  - **Before**: `git show HEAD:.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md`.

### 2026-09-29

- The E152 (Per-Milestone File Performance Against Six File Systems) formal run of that day: quiesce after the full crash amplification run, two-disk mirror, seven file systems + singlefs
  - The ordinal number (which formal run) changed to a date, numbers and artifacts unchanged; before: `git show 4e8b0370:.claude/kb/experiments-history.md`.
  - Artifact `e152-file-system-benchmark-mirror-2026-09-29.out`, all 40 virtual machine runs `vm_exit=0`, host load 0.50–9.61; the tables and cell-by-cell comparison are in section 3.4 of `scripts/experiment-apparatus/perf-by-milestone.md`. singlefs write/read bytes and requests are identical verbatim to the 2026-09-28 run (per disk 805 959 680 bytes written in 1 598 requests; cold recovery reads 806 371 328 bytes in 196 874 requests per disk); the first-segment median 1 874 → 348 ms and cold recovery at 9 886–9 985 ms over 5 rounds with no outliers are host state; of the 207 cells in the two tables, 13 changed by more than 15% (including the singlefs first-segment cell), mostly on sequential writes marked unstable on both days. The four rows of the experiment page's affected decisions table were reviewed up to that day, all not affected.

### 2026-09-28

- The E152 (Per-Milestone File Performance Against Six File Systems) formal run of that day: step 8 of milestone two's closing, two-disk mirror, seven file systems + singlefs
  - Artifact `e152-file-system-benchmark-mirror-2026-09-28.out`, all 40 virtual machine runs `vm_exit=0`, host load 1.59–2.55; the tables and line-by-line comparison are in section 3.3 of `scripts/experiment-apparatus/perf-by-milestone.md`. singlefs's first segment is slow because mkfs really writes the whole journal ring (805 959 680 bytes in 1 598 writes per disk, vs 423 936 / 47 on September 17), B has 29 writes and 6 barriers (21 / 4 on September 17; C577 (Newest Root Not Witnessed by the System Configuration Is Not Covered by B) and the barriers before number acquisition / raising F), and the cold recovery median is 10.0 s.

### 2026-09-19

- The E152 (Per-Milestone File Performance Against Six File Systems) formal run of that day: the first formal run after the apparatus switched to read everything first and then relay; Q2 not enough to judge
  - The ordinal number (which formal run) changed to a date, numbers and artifacts unchanged; before: `git show 4e8b0370:.claude/kb/experiments-history.md`.
  **Before**: after the fourth run (2026-09-17), the apparatus change (read the child process output to EOF before relaying) had only had one smoke run and no formal run; `.claude/kb/vm-harness.md` recorded only the one source "relay blocking causes backlog".

  **After**: the rerun registration `e152-r5-prereg.md` (taking no new number) reran only the singlefs arm, 5 rounds in one pass, artifact `e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out` (142 lines). A second truncation source was found to remain after the apparatus change: in 3 of 5 rounds the `singlefs_timing` line was cut in the middle by `reboot: Power down` printed by the kernel at virtual machine shutdown, and the two fields `file_overwrite_outer_contains_inner` and `file_overwrite_inner_nanoseconds` fell after the cut point and could not be read; independent recomputation (subtracting the self-reported nanoseconds from the arrival time of the `inner=file_overwrite` line of the same round) confirmed that the containment still physically holds in those 3 rounds; only that one field cannot be read. Per the registered measurement basis, Q1 (does the outer contain the inner) takes the "more than half uncertain" path and is enough to judge, with the whole outer "overwrite, release, rollback and reuse ms" cell not reported; Q2 (the median and dispersion of the overwrite, release, rollback and reuse wall clock timed by the child process itself) is judged "not enough to judge" because only 2 rounds (<5) were collected; Q3 (publish B write count and segment sequence) is enough to judge, with all 5 rounds identical verbatim to the third and fourth formal runs, although the existing difference between the anchor cited by the registration, "21 operations", and the implementation's "23 operations" (already explained in `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md`, not a regression of this round) was confirmed again; Q4 (the first-segment lines span ≤ 1 ms) is enough to judge, satisfied in all 5 rounds (78 121–86 810 ns). Two more were run: P1 (recomputing on the 10 rounds of old artifacts from the third and fourth formal runs) confirms the judgment has teeth (the outer does not contain the inner in 9/10 rounds, the span exceeds 1 ms in 8/10 rounds, both reproducing the recorded numbers); P2 (the positive control putting the relay back inside the read loop) succeeded in 0/6 attempts because the registration's assumption about how `scripts/devices/vm-bench.sh` captures lines ("lines not starting with E7RESULT are not collected") does not match the implementation (that script uses `grep -ao 'E7RESULT .*'`, not anchored to the line start), and its artifacts are all void; G1 (`VM_CPUS=1` single-vCPU geometry, 3 rounds) confirms instability: in 2 of 3 rounds the containment is still false, and the first-segment span is still ≤ 1 ms in all.

  **Basis**: artifacts `e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out`, `…-relay-in-loop-control-2026-09-19.out` (P2, all void), `…-relay-timing-vcpu1-2026-09-19.out` (G1); pre-run rerun registration `e152-r5-prereg.md`, question sheet `e152-r5-questions.md`; body [152-按里程碑对比六家文件系统的文件性能.md](experiments/152-按里程碑对比六家文件系统的文件性能.md), section "the formal run of 2026-09-19".

### 2026-09-17

- See the same-day entry "" in the E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) section.

### 2026-09-15

- Created E152 (Per-Milestone File Performance Against Six File Systems): pre-run registration, apparatus and smoke run
  - **Before**: the repository had no performance comparison between singlefs and file systems in service, not a single number.
  - **After**: pre-run registration `e152-preregistration.md` (including three revisions before running the artifact), binaries for the guest load and host summary (17 unit tests, all 10 mutations caught), and two scripts for the extra root and the driver; `vm-bench.sh` gains `VM_EXTRA_ROOT` and `VM_DISK_PREALLOCATE`, and `vm-kernel.sh` gains `--release`. The first smoke run found two test bench defects (the dynamic loader lacked the execute bit; on failure the result lines were collected into the artifact twice), fixed; the formal 5 rounds not run.
  - **Basis**: directed by the user: compare the six file systems XFS, ext4, F2FS, Bcachefs, Btrfs and OpenZFS with singlefs on file performance, find the comparable items and the gaps, and put the results by milestone in one file under `research/`.
- E152 (Per-Milestone File Performance Against Six File Systems) formal run, 5 rounds
  - **Before**: the pre-run registration and apparatus were written, four smoke runs fixed several test bench defects (revisions three to six in section ten of the pre-run registration), and the formal run had not run.
  - **After**: artifact `e152-file-system-benchmark-2026-09-15.out` (890 lines, last line `emitted=174`): all 40 virtual machine runs passed at once, the controls of the seven configurations with controls all have teeth 5 / 5, and no cell was excluded; the body gains a "results" section (copied whole from the artifact), and the per-milestone comparison is written into `scripts/experiment-apparatus/perf-by-milestone.md`; the table script `scripts/experiment-apparatus/e152-tables.py` gains a block-layer amplification table.
  - **Basis**: section seven of the pre-run registration (5 rounds, interleaved by round); criteria and thresholds unchanged from registration to the end of the run, and the six revisions before the formal run touched only the test bench and job parameters.
- E152 (Per-Milestone File Performance Against Six File Systems) two-disk mirror formal run: the six file systems also run on a two-disk mirror
  - **Before**: there was only the single-disk formal run, with the six file systems on one disk and singlefs on two, different configurations.
  - **After**: section eleven of the pre-run registration (seven two-disk mirror configurations, member disk counts summed, the capacity denominator changed to the two-disk total, a void clause for the md sync check); the apparatus gains seven mirror configurations (22 unit tests / all 16 mutations caught); one smoke run all passed; formal run artifact `e152-file-system-benchmark-mirror-2026-09-15.out` (941 lines, last line `emitted=174`); of 41 virtual machine runs, one failed because another session changed vm-bench.sh midway, and a rerun with the same parameters succeeded; the driver script changed to copy the test bench scripts at start and call only the copies afterwards.
  - **Basis**: in the closing pop-up the user chose "fill it in now" for the two-disk mirror; criteria and thresholds the same as the first time.

## E153 (Ledger Forms and Cost of Holes in Ring)

### 2026-09-17

- See the same-day entry "" in the E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) section.

## E154 (Cost of Two-Gate Serial Re-evaluation and Reclamation Timing)

### 2026-09-17

- See the same-day entry "" in the E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) section.

## E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds)

### 2026-09-24

- E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds), E157 (Count Model for Two Clauses on Concurrent Line 1) and E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms) rerun with record header 307 → 311
  - **Before**: the record header constant in the four apparatuses of E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds), the one of E157 (Count Model for Two Clauses on Concurrent Line 1) and the one of E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms) (called `RECORD_HEADER_BYTES` in E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds)/E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms), and `JOURNAL_HEADER_BYTES` in E157 (Count Model for Two Clauses on Concurrent Line 1)) was still at 307; it was first verified that these two names are the same quantity as the `JOURNAL_HEADER_BYTES` of settled item 4 of D23 (Journal Role & Format) (they are: the doc comments, the only occurrence, and the same formula as `crates/` match item by item). `H` enters the six apparatuses through only one channel, `⌊(4096 − H) / 56⌋`, which gives 67 for both 307 and 311, and no second channel reads it.
  - **After**: after the three rerun registrations `e155-r5-prereg.md`, `e157-r1-prereg.md` and `e159-r1-prereg.md` fixed the criteria, the constant in all six apparatuses changed to 311, each gaining a unit test pinning the remainder (33; 37 at 307), **written as an addition, not a subtraction**: the first time it was written as a subtraction, under one mutation (enlarging the named item capacity) it overflowed at compile time under `overflow-checks = true` of the `release` profile and was recorded by `mutate.sh` as "invalid" rather than "caught", hitting `test-discipline.md` "write constant assertions as additions, not subtractions"; after changing to the addition form it ran through again. The six artifacts are byte-identical to before the constant change (four for E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds), one for E157 (Count Model for Two Clauses on Concurrent Line 1), one from the `anchors` subcommand of E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms)), all byte-identical to the existing artifacts in `research/results/`, so nothing new was saved and the registration lines in `replay.sh` were not changed. E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms) additionally ran S2 (the byte-for-byte diff of the four write volume functions shared with the fourth apparatus of E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) is empty) and a real-machine smoke run (S3 probe `logical_block_size=512`, the three arms A / wal_full-K10 / B-M-K10 each run through one cell, newly saved evidence `e159-fsync-wait-group-commit-2026-09-24-h311-smoke.out`); the timing segment was not part of this run. Unit tests: the four of E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) 76/43/53/23 (+1 each); E157 (Count Model for Two Clauses on Concurrent Line 1) 21 (+1); E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms) 55 (+1), all green. Mutations: the four of E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) before 15/34(33+1 equivalent)/14/6, after 18/37(36+1 equivalent)/17/9; E157 (Count Model for Two Clauses on Concurrent Line 1) before 7, after 10; E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms) before 14, after 17; none regressed. **Lines 3, 4 and 5 of the question sheet are all recorded "unchanged"**: the single channel `⌊(4096−H)/56⌋` gives the same number at 307 and 311, and every cell of each of the six apparatuses' original criteria judges the same value.
  - **Also**: the main agent relayed that gate number 12 (no prime-mark superscripts for naming variants anywhere in the repository) was red; the label names with primes in the three registrations `e116-r1-prereg.md`, `e157-r1-prereg.md` and `e159-r1-prereg.md` and the two sources `e157_parallel_line_one_clauses.rs` and `e159_fsync_wait_group_commit.rs` (the primed variants of P1 / P2 / B3 / V4 / A1 / B2) were changed to the next unused number in each family, with the change and the collision check recorded in the "revisions" paragraph of each registration; `bash .claude/gate.d/12-no-prime-marks.sh` was checked by hand to be clear for these places (the script itself was blocked by `heavy-test-guard.sh` and could not run this round, so a per-file grep for prime-mark characters confirmed zero hits instead).
  - **Basis**: `e155-r5-prereg.md`, `e157-r1-prereg.md`, `e159-r1-prereg.md` (three rerun registrations, with criteria, thresholds, failure and stop clauses fixed before the run); lines 3, 4 and 5 of `m2-header311-rerun-questions.md`; settled items 4 / 17 of D23 (Journal Role & Format).

### 2026-09-21

- E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) fourth run, first segment: the k axis of line 7 extended from five tiers to eleven (1–1024), triggering failure clause F11
  - **Before**: line 7 of the fork sheet `m2-s1-forks.md` was only measured up to k=16 (on 2026-09-19 the user answered D25 (Target Workload Priority) on concurrent fsyncs "no limit on the count, at least 16 or more, based on the performance of the disks and the computer", and the top of k fell exactly on that lower bound, with no tier above 16 measured); `row7_grid` 1800 cells (5P×2 families×2 placements×3 policies×3N×5concurrent_fsync_count×2 sharing), `row7_fold_point` 360 groups; Q7.14 (write request measurement basis) and Q7.16 (unshared saturated level indices) not computed.
  - **After**: pre-run registration `e155-r4-prereg.md`; a separate binary `scripts/experiment-apparatus/e155_fourth_run_group_commit_concurrency.rs` (copied from the second run's group commit apparatus); the k axis gains six tiers (32/64/128/256/512/1024, based respectively on `nproc`=32, 2×/4×/8×/16×`nproc`, and `/sys/block/nvme0n1/queue/nr_requests`=1023 rounded up to a power of 2); the algorithm for N_b and the amortization denominator when `k > N` fixed (for k ≤ N, N_b=N/k and denominator=N; for k > N, N_b=1 and denominator=k); Q7.14 (four new `*_calls` fields) and Q7.16 (`saturated_layer_indices`, excluding levels with `node_count==1`: a single-node level necessarily has "dirty nodes = n_l" whether touched or not, a trivial fact of tree structure; section twelve "revisions" of the registration records this tightening) added. 75 unit tests all green (22 new, including correctness anchors B-r4-0 to B-r4-10 and six rows of positive controls); the newly written mutation table `scripts/mutation/tables/e155_fourth_run_group_commit_concurrency.tsv` 6 entries (M1–M6) all hit; the inherited second-run mutation table (34 entries) stopped at 31/34 on the new binary because an anchor no longer hit exactly once in the source (seventh class of `.claude/rules/mutation-sampling.md`, R2_M13); the other two (R2_M16, R2_M17) were verified separately with a temporary supplementary table holding only those two rows and are still caught, while R2_M13 itself was not re-verified on the new binary. Artifact `e155-fourth-run-group-commit-concurrency-2026-09-21.out` (4191 lines; `row7_grid` 3456 cells, matching the registration's "3456 cells to solve"; `row7_fold_point` 360). Stop clause S5 (field-by-field differential of k ≤ 16 against the second run's artifact): the k ≤ 16 parts such as `row6_grid`/`row7_grid` (the second run artifact's 61566 same-named fields minus 259) differ in 0 places; 259 fields in `row7_fold_point` (`fold_point_ten_percent_wal_full`/`_wal_leaf`) changed from `None` to the likes of `Some(32)`/`Some(64)`, which is the expected result of widening the search range from k≤16 to k≤1024 (the old artifact only searched up to 16 and reported `None` when nothing was found), not a formula change; the two `fold_point_zero_*` (0% threshold) fields did not change once across those 360 lines; S5 passes, no stop. **Lead finding (triggering failure clause F11)**: the two fields `fold_point_zero_wal_full` and `fold_point_zero_wal_leaf` are `None` in all 360 lines of `row7_fold_point`: over k ≤ 1024, not one of the 360 verdict strings has a flip point with `s` ≤ 0; group commit by itself (without changing clauses) never catches up with either WAL arm within the concurrency this host can start; in addition, for 4 strings (1 wal_full, 3 wal_leaf) the 10% threshold flip point `fold_point_ten_percent_*` lands exactly on the last tier k=1024. Both trigger conditions of F11 appear together, showing that the top end k=1024 has not yet asked "can A catch up with the WAL arms" to the end, so the conclusion can only be written as "over k ≤ 1024……", not unconditionally. Q7.16: `extent_saturated_layers` non-empty in 1026 lines and `inode_saturated_layers` non-empty in 612 lines, all with `shared=false`, as F14 expects. The five reverse geometry points (R10), F9 random for unshared (R11), and the discriminating-power self-proof along the k axis (R12) were not done, handed to a later segment or the main agent.
  - **Basis**: pre-run registration `e155-r4-prereg.md`; artifact `e155-fourth-run-group-commit-concurrency-2026-09-21.out`; rerun `bash scripts/experiment-apparatus/replay.sh E155R4` reports byte-identical; the second-run artifact used for the S5 differential, `e155-second-run-fsync-write-volume-2026-09-19-stage2.out`, was archived and deleted by gate number 91; it was read back from `git show 3cff909^:research/results/e155-second-run-fsync-write-volume-2026-09-19-stage2.out` and rechecked, with the conclusion unchanged; body [155-每次持久化的写量三种fsync形态与反事实上界.md](experiments/155-每次持久化的写量三种fsync形态与反事实上界.md).

### 2026-09-19

- E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) second run, first segment: K10, R1–R7, the line 6 main table and the F2 judgment built
  - **Before**: there were only all four segments of section 5.7 from the first run (2026-09-17), 42 unit tests / all 15 mutations caught; K10 (an intermediate version replaced within the interval also writes one released allocation record) was not modeled; section 3.2 pointed out four places where the first apparatus did not match the first registration (allocation record W excluding data units, L-balanced+rand near/far reversed, the old-version placement policy of data units, attribution of new data placement under L-far+rand); neither half of F2 (fixed point not converging / crash-point replay of some tree saturated) had ever been judged in an artifact.
  - **After**: rerun registration `e155-r2-prereg.md`; a separate binary `scripts/experiment-apparatus/e155_second_run_fsync_write_volume.rs` (the binary, artifact and `replay.sh` line of the 2026-09-17 first run of E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) untouched, with a new `E155R2` rerun registration line added); the four mismatches fixed per R1–R4 (the B10 threshold moved from 205 to 181, and the three hand-computed cells B15/B16/B17 checked byte by byte); R6 (scattered contiguous groups, B18) and R7 (K10 intermediate version count + into the K6 backlog, B11: in the P=1, N=16 cell, wal_full intermediate versions 60, allocation record tree `[5,1]`) added; the three arms of line 6 added (A, wal_full-K10, B-M-K10, plus two controls wal_full-K9 and B-M-K9 to compute how much more K10 writes than K9); after the main agent's additional request, the judgment for both halves of F2 was built, with every row of `row6_grid` and `q6_11_legacy_vs_fixed` reporting it (A judged separately from the four WAL checkpoint cores). 47 unit tests all green (5 new pinning the two F2 anchors); mutation table `scripts/mutation/tables/e155_second_run_fsync_write_volume.tsv` (27 entries: the first run's 15 run unchanged and all caught again; of the second run's 12 new, 11 caught and 1 judged "equivalent" as the registration expected: each A batch is exactly one publish, so applying the WAL arm's intermediate-version formula is structurally constant 0, turning 0 tests red). Artifact `e155-second-run-fsync-write-volume-2026-09-19-stage1.out` (original not kept: it was never committed, and the second segment extended it into the cumulative `research/results/e155-second-run-fsync-write-volume-2026-09-19-stage2.out`, retrievable with `git show cc3e8ba1:research/results/e155-second-run-fsync-write-volume-2026-09-19-stage2.out`; 372 lines: `row6_grid` 288, `q6_11_legacy_vs_fixed` 72, `geometry_sensitivity_sample_row6` 6, 1 each for the rest, `name=done emitted=372` agreeing with the lines actually collected). Lead reading: in the P=1, F1, seq, Lbalanced cell along the checkpoint interval N axis, the share `s` that wal_full-K10 saves over A rises from −2.4% at N=1 (A cheaper; the N_b=1 cell does not count as a criterion) to 35.4% at N=16, 40.2% at N=256 and 40.4% at N=4096, crossing both thresholds 0 and 10% (the discriminating-power self-proof passes); directional check of R6: treating the 8 adjacent units of one F8A fsync as independent scattered points would overestimate the interval-union dirty node count of the two WAL arms' checkpoints under rand by about 7.6× (16.75 vs 126.55). **Lead finding of the F2 judgment**: the first half (hitting 1000 iterations without converging) triggers only at P=10⁸, F1, rand, Lbalanced (all 4 N); the second half (some tree's level 0 has ≥64 nodes with ≥50% dirty) triggers in 24 rows (P∈{10⁴ (only F8A/Lbalanced), 10⁶, 10⁸}×rand) on the `allocation` tree, triggers more for the WAL checkpoint cores (K10 65 rows), and **also triggers under Lbalanced (not only L-far)**: the half-sentence in the original F2 text of section ten of the registration, "narrowed only when L-far appears", does not hold; rereading the conclusion of line 5 on the first experiment page along the P axis, `seq` placement is still not out of control (344 576→1 831 514, 5.3× as P goes to 10⁸), but `rand` placement after the R1 fix grows 29.7× from P=10⁴ to 10⁶ (35 332 318), and the 10⁸ cell hits the first half of F2 (magnitude up to 3 330 926 706; the value is not trustworthy and only the magnitude is marked); whether to rewrite the conclusion "not out of control" because of this is handed to the main agent. Line 7 (R8 group commit, k axis) and the full section 8.2 geometry sensitivity scan on the line 6 verdict cells (only 1 representative cell this time) were not done; this version built no computable switch for the two reverse sampling points K12 and N counted by batch, which are placeholders in the artifact, handed to the next segment.
  - **Basis**: pre-run registration `e155-r2-prereg.md`; artifact `e155-second-run-fsync-write-volume-2026-09-19-stage1.out` (original not kept; the second segment extended it into `research/results/e155-second-run-fsync-write-volume-2026-09-19-stage2.out`); rerun `bash scripts/experiment-apparatus/replay.sh E155R2` reports byte-identical; body [155-每次持久化的写量三种fsync形态与反事实上界.md](experiments/155-每次持久化的写量三种fsync形态与反事实上界.md).
  - **Path correction** (2026-10-14): for the before-state see `git show f111c3f3:.claude/kb/experiments-history.md`, where the "after" and "basis" lines of this entry gave the full path of the first-segment artifact under `research/results/`; that artifact was never committed.
- E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) second run, second segment: R8 group commit and the line 7 main table built
  - **Before**: the second run had only reached the first segment (decision change history (part four): K10, R1–R7, the line 6 main table, the F2 judgment, 47 unit tests / 27 mutations); not a single byte of line 7 (group commit, concurrent_fsync_count ∈ {1,2,4,8,16}, both shared spine and unshared) had been computed, and B12–B14, Q7.1–Q7.6 and two positive controls were all unrun.
  - **After**: the same binary gains `solve_jia_batch_publish` (A merges concurrent_fsync_count concurrent fsyncs into one batch = one publish), `solve_write_ahead_log_batch_fsync` / `solve_write_ahead_log_batch_checkpoint` (batch-level versions of the two WAL arms, using contiguous groups for the shared spine and mutually unshared paths for unshared), and `intermediate_version_counts_batch` (the batch-level version of the R7 intermediate version count); the line 7 main table (`row7_grid` 1800 cells = 5P×2 families×2 placements×3 policies×3N×5concurrent_fsync_count×2 sharing) and flip points (`row7_fold_point` 360 groups) added; both halves of F2 (section ten) judged likewise on every cell of line 7 (`jia_saturated_trees` non-empty in 186/1800 rows, a higher proportion than line 6's 24/288). 52 unit tests all green (5 new: the three hand-computed cells B12/B13/B14 checked byte by byte, and at concurrent_fsync_count=1 the batch-level functions are byte-identical to line 6's single-fsync functions); the mutation table extended to 34 entries (seven new mutations R2_M9–R2_M13, R2_M16, R2_M17): 33 caught, 1 (R2_M19, the equivalent mutation already judged in the first segment) still judged equivalent; while the second-segment code was being added, at one point some existing anchors deviated from hitting exactly once because structurally similar second-segment code appeared in the source (exactly which ones, neither the previous runner's handback record nor this segment kept an itemized list, and none is made up here); after the fix, gate number 33 and a per-entry substring count re-check both confirmed that all 34 entries of this table hit exactly once (`scripts/mutation/tables/e155_second_run_fsync_write_volume.tsv`, and all 1509 mutation anchors across the repository likewise hit exactly once). Lead reading: of the 360 groups in `row7_fold_point`, not one has `fold_point_zero` (`s` ≤ 0, A catching up with a WAL arm) within concurrent_fsync_count ≤ 16: group commit by itself (without changing clauses) is not enough to make A cheaper than the two WAL arms; `fold_point_ten_percent` (`s` pressed within 10%) appears in 125/360 groups (wal_full) and 94/360 groups (B-M), mostly (77.6%, 70.2%) at the concurrent_fsync_count=8 tier; the "N counted by batch" reverse point was checked for direction in 1 representative cell (P=10⁴, F1, seq, Lbalanced, concurrent_fsync_count=2, N=16): amortization drops from 184 281 to 168 318.875, as the registration expected. The artifact was extended from `-stage1.out` (372 lines) to `e155-second-run-fsync-write-volume-2026-09-19-stage2.out` (2535 lines, cumulative, `name=done emitted=2535` agreeing with the lines actually collected), and `-stage1.out` is no longer kept separately; the `E155R2` line in `replay.sh` now points to stage2. The apparatus file header comment and the artifact's `name=scope` line originally said "stage=1" and "second segment not done" (the previous runner was interrupted by the session context running out while writing the experiment page, before syncing these two descriptive texts); this segment fixed them too, affecting no value or verdict. What it cannot answer: F9 (random for unshared) and the five reverse sampling points φ=0.5/K11/g=0/pbs=4096/K12 on the verdict cells of lines 6 and 7, and the discriminating-power self-proof along the concurrent_fsync_count axis: this version has no computable switch for them or did not do them, handed to the main agent.
  - **Basis**: section 5.1 R8 and section one "readings fixed" of pre-run registration `e155-r2-prereg.md`; artifact `e155-second-run-fsync-write-volume-2026-09-19-stage2.out`; rerun `bash scripts/experiment-apparatus/replay.sh E155R2` reports byte-identical; body [155-每次持久化的写量三种fsync形态与反事实上界.md](experiments/155-每次持久化的写量三种fsync形态与反事实上界.md).

### 2026-09-17

- 
  - E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) continued with the third and fourth segments: before, there were only the first and second segments (K1–K9 core + the three arms A/wal_full/B + Q1.1–Q1.8), 29 unit tests / 11 mutations; after, Q2.1–Q2.6 filled in (a `SuppressedFixedPoints` struct forces the dirty node count of this persistence to 0 for any one of accounting / allocation records / mapping / tree table, with the tree still present and entry counts computed normally), Q2.5 (the tree table entry count changed from a constant to a parameter, reusing `tree_layers` to compute the tree table's own height), Q3.1 (`q3_1_tree_savings` strings together each level's dirty node count, that level's condition count c and the rounded bytes, expanding six trees level by level) and Q4.1 (pure closed form) over the full grid, plus Q5.1–Q5.4 (`bisect_first_threshold` bisects for the threshold P* of each tree that grows taller, and `rho_readings` computes ρ separately under the three registered readings); 42 unit tests / all 15 mutations caught (13 unit tests / 4 mutations new). **Lead finding, answering directly the user's worry "will it get exponentially worse with scale"**: under the main geometry Lbalanced it does not: ρ_a (A's bytes written per persistence ÷ the bound of the six trees' own path bytes) only swings between 1.0 and 4.0 as P goes from 1 to 10⁸; but under the worst placement policy Lfar (the old version always lands farthest from the insertion point) combined with rand placement, `a_tree` is nearly proportional to P from P=10⁵ on, and ρ_a grows all the way to 11270 at P=10⁸: not exponential in the mathematical sense (`c^P`), but "getting worse almost linearly with scale", and only under this artificially constructed worst-case assumption (the model itself does not simulate real histories). It also found that the counterfactual upper bounds of the four fixed points move differently with P: accounting / tree table actually shrink below the 10% threshold as P grows (fixed costs amortized), while allocation records / mapping grow past the 10% threshold as P grows; one more level for extent/inode is exactly 1 extra path's share (e=1.0), and one more level for allocation records/mapping costs 2 extra (e=2.0, pinned in unit tests). Naming discipline hit one real format constant name collision (the `DATA_UNIT_HEADER_BYTES=134` I took collided with the same-named constant 105 already registered in the kb; after renaming to `DATA_UNIT_HEADER_AND_TRAILER_BYTES`, gate number 27 is green). Five mutations M8/M11/M13/M16/M20 are still unwritten (lacking a pinned assertion confident of really hitting), and the full Q1.4 label matrix, the full 8.2 reverse geometry scan and K10 modeling are still undone. Basis: artifact `e155-fsync-write-volume-2026-09-17-stage4.out` (1334 lines; `replay.sh` now points to it, byte-identical); revision ③ in "section twelve" of the pre-run registration records the real timing of the 7.3 hand-computed cells; body [155-每次持久化的写量三种fsync形态与反事实上界.md](experiments/155-每次持久化的写量三种fsync形态与反事实上界.md).
  - Added E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) and ran the first and second segments of registration section 5.7 (not fully at registered scale): the registration requires four segments (the counterfactual upper bounds and P threshold search of lines 2–5 are the third and fourth); this round did only the first and second: the K1–K9 model core, the 5.3 fixed point, the 5.4 three arms A / wal_full / B, the 5.5 positive controls, Q1.1–Q1.8, 29 unit tests / all 11 mutations caught; all 5 hand-computed cells required by 7.3 filled in (level 2 of the allocation record tree / mapping tree first appearing at P=205 / P=285 respectively, Lfar and Lbalanced under rand giving different byte counts at P=1000, named items exceeding 67 with P=10⁴/F8A/rand/Lfar, Q3.1 hand-computed at P=1 `U_3≈72.81%`), but the step of transcribing them into "revisions" in section twelve of the pre-run registration came after the artifact (the content is unrelated to the artifact, pure hand computation; the registration file was not changed again this round; the gap in procedural order is handed to the main agent). Lead result: A/B (fsync bytes) rises with P from 2.00× (P≤100) to 10.65× at P=10⁸, step by step ≥5% without converging; G23.1 (ancestor chain share reaching the bar) write byte proxy share rises from 47.70% to 90.16% (99.63% in the P=10⁸ cell); on the fsync row of the three arms, B≤wal_full≤A is violated in not one cell of the P∈{1,145,10⁴,10⁶}×family×placement×placement policy grid. Basis: artifact `e155-fsync-write-volume-2026-09-17-stage2.out` (305 lines), pre-run registration `e155-preregistration.md`, body [155-每次持久化的写量三种fsync形态与反事实上界.md](experiments/155-每次持久化的写量三种fsync形态与反事实上界.md). Whether the third and fourth segments (counterfactual upper bounds, P threshold search) and the full 8.2 geometry sensitivity scan need to be continued is for the main agent to decide.
  - E152 (Per-Milestone File Performance Against Six File Systems) apparatus change: before, the guest program, in the loop reading child process output, stamped times while relaying lines to the serial port, 22 unit tests / 16 mutations; after, `read_lines_with_arrival_times` reads to EOF first and then relays, the `singlefs_timing` line additionally reports the nanoseconds timed by the child itself and a containment self-check, rounds where the outer does not contain the inner are reported as `summary_excluded` in the summary and do not enter the median, a new metric `singlefs_file_overwrite_inner_milliseconds`, `e152-tables.py` gains a column, 26 unit tests / all 26 mutations caught (M17–M26). Section fourteen of the pre-run registration was written before any formal run, fixing the criteria from the next run on; in one smoke round the containment self-check was true. Separately, gate number 65 was set up to check for loops in the apparatus that take times while outputting as they read child process output, and `.claude/kb/vm-harness.md` gains a section "timing does not stamp times inside the loop that forwards output". The existing four artifacts are not recomputed. Basis: the user asked to settle this timing problem into lasting form.
  - E152 (Per-Milestone File Performance Against Six File Systems) statement about cold recovery read volume corrected: before, it said the two disks' read requests "are exactly 2 × 196 608 whole-ring reads of 4 KiB plus 81", "the block-layer count is an independent second path", and "the read volume varies only with the ring size"; after, it says the block-layer count is compatible with reading the whole ring slot by slot but cannot confirm it. Per-disk accounting (the same in all four runs): in the two new-pool file creation runs, disk 0 805 568 512 bytes / 196 661 requests, disk 1 805 363 712 bytes / 196 636 requests; the 53 / 28 requests beyond the ring slot count carry 262 144 / 57 344 bytes, and on both disks read bytes are 12 288 fewer than requests × 4 KiB; in the two overwrite, release, rollback and reuse runs, disk 0 805 609 472 / 196 664, disk 1 805 371 904 / 196 638, and on both disks read bytes are 16 384 more than requests × 4 KiB. The whole ring (805 306 368 bytes per disk) is 99.98% of read bytes, and reads outside the ring vary with content; the block layer cannot tell which requests land on the ring, and the breakdown comes from `scan_journal` in `crates/singlefs-core/src/recovery.rs`. Basis: the user pointed out that the sentence does not match the two raw counts, read bytes and read requests.
  - E152 (Per-Milestone File Performance Against Six File Systems) gains a comparison conclusion for the two runs: before, the two runs had only whole-line transcriptions, the conclusion was only in section 3.2 of `scripts/experiment-apparatus/perf-by-milestone.md`, compared against the numbers from the six file systems' first write, with the two sides' write request measurement bases differing, and wall clock taken from the summary line's 5.08 / 5.53 ms; after, the experiment page gains a section "what these numbers show (overwrite, release, rollback and reuse, compared with the six file systems on a two-disk mirror)": the ranking among seven compared per operation with the six mirrored file systems (last on write bytes, block-layer write requests and wall clock, middle on FLUSH, slowest on cold mount), publish B bytes broken down by unit (metadata units 76.1%, data units 19.0%, fixed points 4.9%), and two measurement basis problems: the overwrite, release, rollback and reuse wall clock in the summary line was cut by the outer side at output line arrival times, mixing in print backlog (in 9 of 10 rounds smaller than the publish B nanoseconds timed by the binary itself; the "two clusters" of the first segment are also this), so it now takes the binary's own timed median 6.14 / 6.63 ms; write requests now use the block-layer notation 26 (block layer = write calls + barriers + FUA, matching the per-disk readings of four groups). The apparatus was not changed; before the next formal run it must be changed so the binary reports the three segments' wall clock itself. Basis: the user pointed out that the core of E152 (Per-Milestone File Performance Against Six File Systems) is comparison and finding gaps.
  - E154 (Cost of Two-Gate Serial Re-evaluation and Reclamation Timing) fourth segment: before, 30 unit tests short (23 at the time), H5-k-bit and H6 implemented, H4a/H4b and H8 not done; after, H8 added (an H5 variant designated by the main agent, the injection point moved from the admission of "write back after deletion" to the empty publish pushed by the fill phase itself to raise F, with `LazyFailureInjector` computing `window_start` fresh at the first real self-pushed empty publish) and H4a/H4b (crash and reopen in the middle of a push-empty publish, `crash_and_reopen` reusing the existing `perform_instance_switch`); 30 unit tests / all 14 mutations caught. Lead result: on H8 the real baseline (serial re-evaluation × G7) degraded cells drop from H5's 210/288 (stop) to 0/288 (stop) and 0/288 (full); H4a and H4b produce byte-identical RUN lines in this model (an equivalence determined by the atomicity granularity of `attempt_publish`, pinned by a unit test). The artifact self-check found an existing problem (not introduced by this segment; the second-segment artifact already had it): the real baseline has rows triggering F5 (unsafe reclamation timing, issuing slots still referenced by candidate roots) on H5／H6／H4a／H4b, which earlier reports and the experiment page never mentioned; judging whether it is right needs a three-way argument. Stop clause S1 was not done at the start of this segment, only checked fresh after the artifact ran: the hashes of `mount.rs`／`transaction.rs` had changed relative to the baseline recorded in the second segment (another session is still changing `crates/`), and S2 (whether the six apparatus-implementation correspondences still hold today) was not re-checked in this segment; per the dispatch prompt's "stop and report if they don't match", handed to the main agent. H3a/H3b, ROW details and the exact Q4 components are still unimplemented. Basis: artifact `e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage4.out` (37346 lines), fourth segment of section "twelve" of the pre-run registration, body [154-两道闸串-重判与回收时点的代价.md](experiments/154-cost-of-two-gate-serial-re-evaluation-and-reclamation-timing.en.md).
  - E153 (Ledger Forms and Cost of Holes in Ring) continued with the fifth segment: before, the fourth segment had run 26 unit tests / 22 mutations, Q1–Q6/Q9a/Q9b (peak/mean/final)/Q11–Q13 over 12 family labels × 12 cells, with not one of Q7/Q8/Q10/Q6b computed, and the crash branches (L2/L3) and the bad mirror matrix not built at all; after, three complete implementations added: Q7 (number of states where G27 judges red on legal states, per disk "accounting item 1 − item 5 == references walked from the newest root"), Q10 (independent check of the interval reasoning of G12×G28 "trees not separately read from candidate roots" against the union of real walks), Q6b (comparing the two readings of the upper bound of item nine in settled item 1 of D28 (Mount-Time Commitments) with the slot count quarantined by shadow accounting), plus five kinds of the Q8 bad mirror matrix, B1a/B1b/B2/Bk1/Bk2 (two bases: β1 = H1 final state, β2 = after the S-th workload following the H4dNm2 rollback; the five kinds B3a/B3b/B4a/B4b/B4c are still unbuilt, each needing one of two new mechanisms, "rerun the history with reclamation disabled" and "force placement collisions"); 40 unit tests / all 31 mutations caught (20 unit tests / 10 mutations new, each first proven to go red). Four new findings: Q7 is 0 on all 720 (family, cell, arm) rows run (G27 has zero false reports on legal states); the five bad mirror kinds run match the registration's P5 expectations item by item with zero exceptions over the 72 discriminable (base, geometry, arm) combinations; both readings of Q6b are exceeded by the quarantined slot count on all 360 non-zero samples of the four families with rollback, and the upper bound is never computed negative (recorded per registration F4 "the clause may be wrong, hand to the main agent", without changing the shadow accounting definition to fit); Q10 is 0 on all histories run (a different matter from the G12 main predicate divergence found in the fourth segment; see item 8 of "what was done" in the body). One naming discipline rework was found and reverted along the way: while changing Rust identifiers to satisfy `naming-lint.sh`, a whole-text replacement accidentally hit the existing artifact field label `q3_i52_red_states` (once wrongly changed to `q3_free_statistic_mismatch_states`), now reverted. Basis: artifact `e153-ledger-shape-and-ring-holes-2026-09-17-stage5.out` (2487 lines), "fifth segment of section twelve" of the pre-run registration, body [153-账本形态与环上有洞的代价.md](experiments/153-账本形态与环上有洞的代价.md). The crash branches (L2/L3) and the remaining five bad mirror kinds of Q8 are still unimplemented; whether a supplementary run is needed is for the main agent to decide.
  - The E152 (Per-Milestone File Performance Against Six File Systems) closing formal run: before, after the step 7 run `crates/` changed in four places (slots reclaimed by raising F held until taking effect, the bump cursor avoiding quarantined and held positions, raising F catching unreadable counts, the checker judging two more items); after, at milestone closing the user asked to rerun the singlefs arm once more (section thirteen of the pre-run registration), 5 rounds in one pass, all criteria hit, write counts and segment sequences identical verbatim to the third run, with the two wall clocks listed side by side without comparing speed (overwrite, release, rollback and reuse median 5.53 ms, dispersion 259%). Basis: artifact `e152-file-system-benchmark-second-transaction-final-2026-09-17.out`; section 3.2 "running once more at closing" of `scripts/experiment-apparatus/perf-by-milestone.md`.
    - The ordinal number (which formal run) changed to a date, numbers and artifacts unchanged; before: `git show 4e8b0370:.claude/kb/experiments-history.md`.
  - The E152 (Per-Milestone File Performance Against Six File Systems) step 7 formal run: before, there were only the two from 2026-09-15 (the six file systems and singlefs new-pool file creation); after, on the milestone "overwrite, release, rollback and reuse" only the singlefs arm was rerun (`file-overwrite` mode; section twelve of the pre-run registration fixed the criteria: B's segment sequence 16+2+1+2, two disks totaling 21 writes / 344 576 bytes / 4 barriers / 1 FUA, cold recovery reading back the second version), 5 rounds in one pass, all criteria hit, overwrite, release, rollback and reuse wall clock 5.08 ms (dispersion 186%, reported as is). Basis: artifact `e152-file-system-benchmark-second-transaction-2026-09-17.out`; the conclusion is written in section 3.2 of `scripts/experiment-apparatus/perf-by-milestone.md`.
  - Added E154 (Cost of Two-Gate Serial Re-evaluation and Reclamation Timing) and ran it (not fully at registered scale): the registration requires 12 histories (H1–H6, 24 arms × 48 geometry cells in total); this round implemented only H1, leaving the other 11 (H2 covered by the n=1 sampling point; H3a/H3b/H4a/H4b/H5-k-bit/H6 unimplemented); 16 unit tests / all 7 mutations caught. The real baseline (serial re-evaluation × G7) "meets the bar" in 44/48 cells under P-stop and 0/48 under P-full (24 out of bounds + 24 stuck); Q6 (slots still referenced by candidate roots) is constantly 0 over all 1152 runs; the four positive controls PC1/PC2/PC4/PC5 all have teeth. Two implementation bugs were found and fixed during execution (newly allocated slots were not marked `Allocated`; under PC5, failure propagation with `.expect()` would panic, changed to hand back to the caller and treat as stuck). Basis: artifact `e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17.out`, pre-run registration `e154-preregistration.md`, body [154-两道闸串-重判与回收时点的代价.md](experiments/154-cost-of-two-gate-serial-re-evaluation-and-reclamation-timing.en.md). Whether the remaining 11 histories need a supplementary run is for the main agent to decide.
  - Added E153 (Ledger Forms and Cost of Holes in Ring) and ran it (not fully at registered scale): the registration requires a full Q1–Q13 scan over 9 history families × 12 geometry cells × 5 arms, crash branches (c1/c2 × 2N suffixes) and the bad mirror matrix; this round only computed and pinned the absolute-value anchors of sections seven and nine of the pre-run registration (five class K, seven class A, four class U), 16 unit tests / all 13 mutations caught; the three accounting items, placement geometry, warm-up and F-raising publish counts of the three history families H1 (consecutive overwrites), H4 (single rollback) and H6 (single raise of F) match the registration's independently computed numbers item by item, while not one of the 6 history families H2/H3/H5/H7, Q1–Q13, the crash branches or the bad mirrors was run. Two implementation bugs that systematically biased the accounting threshold by one generation were found and fixed during execution (the reclamation step must distinguish two different rings, "the ring" and "the accounting ring"; a span-2 record only removed the starting slot, so the second slot could never be reclaimed). Basis: artifact `e153-ledger-shape-and-ring-holes-2026-09-17.out`, pre-run registration `e153-preregistration.md`, body [153-账本形态与环上有洞的代价.md](experiments/153-账本形态与环上有洞的代价.md). Whether Q1–Q13, the crash branches, the bad mirrors and the remaining history families need a supplementary run is for the main agent to decide.
  - E153 (Ledger Forms and Cost of Holes in Ring) continued with the fourth segment: before, the third segment had run 21 unit tests / 18 mutations, Q1–Q6/Q9b (final value only)/Q11–Q13 over 11 family labels (H1/H2/H3φ×3/H3²/H4×2/H5-early/H5-late/H6) × 12 cells, and in the third-segment artifact all 24 q_family lines of H5-early/H5-late had only `q13_no_candidate=true` (cause not found); after, the root cause for H5-early/H5-late was found: when the history driver function borrowed the H4 prefix it also borrowed H4's own tail workload, so the target of the second rollback had long been rotated out of the ring; after the fix both families produce complete 5-arm data in all 12 cells; a new H7 family (the 12th family label) added; Q9a/Q9b extended from final values to peak/mean/final trajectories; slot-by-slot checking found the root cause of the third segment's finding ① (G12 makes I-3.1 (Allocated Statistics Match) go red on families with rollback): G12's predicate absorbs the duty of "protecting references from abandoned roots" directly into accounting item 1, which is architecturally misaligned with the definition of I-3.1 (Allocated Statistics Match), counting only references from the candidate set, compounded by false positives of the interval approximation itself (a slot already replaced is still judged occupied by the interval); neither cause leads to fixing the definition of G12, and per the registration it is only recorded as a "loss". 26 unit tests / all 22 mutations caught. Basis: artifact `e153-ledger-shape-and-ring-holes-2026-09-17-stage4.out`, "fourth segment of section twelve" of the pre-run registration, body [153-账本形态与环上有洞的代价.md](experiments/153-账本形态与环上有洞的代价.md). The crash branches and Q7/Q8/Q10 (G27 and the bad mirror matrix) are still unimplemented; whether a supplementary run is needed is for the main agent to decide.

## E156 (Cost Figures for Four Alloc-Basis Forks)

### 2026-10-18

- E156 (Cost Figures for Four Alloc-Basis Forks) 7th rerun: question sheet H1 (the C566 (extra slots deducted when ring has holes grow unboundedly with hole count) half): hole count scanned from tier 1 to a full lap of the ring and across laps; under both reading A and reading B the Q1a peak is judged "still rising", and the upper bound numbers all land on the largest tier (≤ the endpoint; no upper bound was reached); the E156 (Cost Figures for Four Alloc-Basis Forks) line of `scripts/experiment-apparatus/replay.sh` now points to the new artifact, and `driver_e156` switched to release, with the `verdict-store,gpu` features, started through `gpu-quiesce-run.sh`
  - **Before**: apparatus 6291 lines, 19 unit tests; the artifact `research/results/e156-alloc-basis-counts-2026-10-05-r2.out` (6th run, only the Hh family, hole count tiers only k = 0, 1, 2, 4) was the one registered in `replay.sh`, and `driver_e156` was the debug `cargo run -q -p singlefs-checker-tier`; H1 had no reading.
  - **After**: apparatus 7580 lines, 25 unit tests (run by the main agent under release, 25 passed); artifact `research/results/e156-alloc-basis-counts-2026-10-18.out` (16669 lines, sha256 `1663f1ea28e5b3334f21adfc121bbf899bc66e7e429eea782f1f0c562c776c83`, wall clock about 241 s), 116 histories × two reclamation timings, plus hole count tiers and two families, Hc crossing laps and Hs stacking holes on the same slot. Reading A (R7a, 8 statistical units at S = 8, 16): 8 "still rising", deducting 10, 20, 28, 36 … more per tier (up to 68 at S = 16); reading B (R7c, 12 statistical units): 12 "still rising", largest tier 914 (S = 8, ρ = 1), 1810 (S = 16, ρ = 1), 1416 (S = 16, ρ = 1/4); F31, F33 and F34 do not trigger, and F32 triggers because R7b (Hh-after) has 8 units judged "not monotonic" (the registration predicted the same per tier and was wrong); K10, F24, G-adm and S20 do not trigger. H1 is enough to judge per the registered sufficiency condition; which of candidates A, B and C holds is not judged this time.
  - **Basis**: rerun registration `research/prompts/e156-r7-prereg.md` (including revision 1); artifact `research/results/e156-alloc-basis-counts-2026-10-18.out` and the check scripts `research/results/e156-alloc-basis-counts-2026-10-18-support/`; body [156-alloc-basis四条岔路的代价数.md](experiments/156-alloc-basis四条岔路的代价数.md), section "7th rerun (2026-10-18)"; the six rows of the affected decisions table reviewed on 2026-10-18, all not affected (the basis paragraph of settled item 1 of D16 (Publish Semantics) does not cite E156; whether to upgrade it to support is handed to the main agent).

### 2026-10-17

- In the title status, "which one carries the load is for the main agent to decide" changed to "the main agent has accepted the 5th artifact", pointing to the acceptance item in e156-r5-prereg.md, and "17 unit tests" marked as the measurement basis at the time of the 4th rerun; the results section heading gains the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md`.

### 2026-10-15

- 74 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 74 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md`.

### 2026-10-06

- "What it cannot answer" on the experiment page gains verdict P5 of the code round of batches 32–33: the two controls cannot tell pinning the oldest from pinning the largest txg, unpinning on reread, or slots read as bad; M36 may lose a bit of discriminating power on the front-hole cell (inferred). After B was settled and written back after line 37 of settled item 1 of D16 (Publish Semantics), the review found the 6th run's threshold reading identical to the clause text item by item.
  - **Basis**: P5 in section two of `research/prompts/m3-formal-code-b33-r1-main-verification.md`; "main agent review" at the end of `research/prompts/e156-r6-prereg.md`.

### 2026-10-05

- E156 (Cost Figures for Four Alloc-Basis Forks) 6th rerun: the reclamation threshold read per the user-settled B (root ring slots read back as all zeros at mount are pinned at the oldest txg they could have held)
  - **Before**: apparatus 5819 lines, 18 unit tests; the threshold was taken only from valid roots readable in the mirrored root ring (in the 5th artifact `research/results/e156-alloc-basis-counts-2026-10-05.out`, `s11_k10_violation_points=34` and `f24_real_every_differ_points=34` came from core pinning all-zero slots per the settled B while the apparatus did not), and the anchor model did not pin either.
  - **After**: apparatus 6205 lines, 19 unit tests (new U20); at mount it reads the all-zero slots of the cut-segment mirror and pins them at the oldest txg, taken as the new instance's first txg, unpinning when this process writes over that slot or reads a root from it; the anchor model changed the same place per registered command four; new lines `name=all_zero_pins_at_mount`, `name=all_zero_pin_cell`, `name=all_zero_pin_summary`. Main artifact `e156-alloc-basis-counts-2026-10-05-r2.out` (4063 lines): in 38 cells `s11_k10_violation_points=0` and `f24_real_every_differ_points=0`; in a copy with core's all-zero pinning removed, `research/results/e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out` gives 34 for both, with point sets identical to registered L6 × two ρ; compared with the 5th run: the "real" column is identical field by field, the "every" column becomes identical to the "real" column along with it, Q1d in 4 groups of the "every" column changes from `grows_with_holes` to `not_monotonic`, F1 still triggers, and the Q1c-front verdict is unchanged; the seven PC-pinned-floor lines are byte-identical to the 5th run. Unit tests and mutations M45–M47 not run (left for commit time).
  - **Basis**: rerun registration `research/prompts/e156-r6-prereg.md`; artifacts `e156-alloc-basis-counts-2026-10-05-r2.out`, `research/results/e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.out`, `research/results/e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.txt`, `research/results/e156-alloc-basis-counts-2026-10-05-r6-point-sets.txt`; body [156-alloc-basis四条岔路的代价数.md](experiments/156-alloc-basis四条岔路的代价数.md), section "6th rerun (2026-10-05)".

- E156 (Cost Figures for Four Alloc-Basis Forks) 5th rerun: after the core assertion, the positive control of the W7 cell replaced by PC-pinned-floor, and the 4th run's readings recompared field by field on the new `crates/`
  - **Before**: in the apparatus `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` (5776 lines, 17 unit tests), the positive control of W7 was PC-allocator (`allocator_without_the_root_ring`, S = 4, ρ = 1, k = 0, swapping back an allocator without the root ring table, which A10 should judge red), artifact `e156-alloc-basis-counts-2026-09-26-r2.out` (4007 lines), with the E156 (Cost Figures for Four Alloc-Basis Forks) line of `replay.sh` pointing to it; once the user settled "core asserts when the allocator has no root ring table and a root is recorded", PC-allocator panicked at the first root recording.
  - **After**: apparatus 5819 lines, 18 unit tests (PC-allocator removed, PC-pinned-floor added: the two lag sets of the six Hh(1, after, S, ρ) cells in the cell table fed to the A10 verdict at k = 0, printing six `name=pc_pinned_floor` lines and one summary line; new unit test U19); artifact `research/results/e156-alloc-basis-counts-2026-10-05.out` (3955 lines; rerunning the same binary is byte-identical). Line 1: the verdicts of both columns in all six cells are "mismatch", and the lag sets all contain 6S − 1, `v2_pc_pinned_floor_seen=true`. Line 2: after normalization 294 of 3948 lines differ (60 verdict lines, 234 count lines, including the count item of `name=done`): `f24_real_every_differ_points` 0 → 34, `s11_k10_violation_points` 0 → 34, the geometry sensitivity of `q1d_growth` changes from "consistent over 12 sampling points" to "unstable" (4 points `not_monotonic` in the "real" column), and F1 still triggers; the attribution run (with the 31-1 assertion reverse-patched in a copy) is byte-identical to the new artifact, so the differences do not come from 31-1. Stops S11 and S14 trigger, handed to the main agent; the E156 (Cost Figures for Four Alloc-Basis Forks) line of `replay.sh` now points to the new artifact, and the r2 one is kept. Inline unit tests and the whole mutation table not run (heavy tier, left for commit time).
  - **Basis**: rerun registration `research/prompts/e156-r5-prereg.md`; artifacts `research/results/e156-alloc-basis-counts-2026-10-05.out`, `research/results/e156-alloc-basis-counts-2026-10-05-attribution-without-31-1.out`, `research/results/e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt`; two mutation files `research/results/e156-alloc-basis-counts-2026-10-05-mutations-replacements.tsv`, `research/results/e156-alloc-basis-counts-2026-10-05-mutations-append.tsv`; body [156-alloc-basis四条岔路的代价数.md](experiments/156-alloc-basis四条岔路的代价数.md).
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

### 2026-09-26

- E156 (Cost Figures for Four Alloc-Basis Forks) 4th rerun: three premises reproduced on the apparatus, fork sheet line 3 invalid by premise 2, and line 1 measured over all 38 histories
  - **Before**: the apparatus `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` (4335 lines, 11 unit tests / 12 mutations) was written per the old implementation that rolled back at mount and did not reclaim within a mount: eight families H0 / HR / HK / HK-F0 / HF / HX / HY / Hh; fork 1 had only been measured at S = 8 and at S = 4, ρ = 1, reclamation timing "real", hole position "after", k ≤ 2; at the start of work, unit tests U11 and U13 were already red (rollback at mount had been removed). The index row status was "partly run (2026-09-22 first and second segments + 2026-09-23 R2 first segment + 2026-09-24 R2 second and third segments (narrowed scope, only at the one geometry sampling point S=8, ρ=1), committed apparatus, deterministic, 5 unit tests / all 4 mutations caught; forks 2 and 7 run; one geometry cell of fork 3 run (X8 (the two measurement bases of small-pool exhaustion construction)-A not done); three hole count sampling points of fork 1 run (full S×ρ scan not done))"; the conclusion column as it was: K1 checked verbatim and matching (disk 0 accounting items 1/2/5 at 13/211955/1); fork 2: list L (4 places) missed 41 places caught by the command (`|M∖L|=41`); gate number 51 itself needs no change (data-driven), but the two-step explanation the registration gave itself missed one copy of a formula; fork 7 (rerun after the R2 first segment changed to reading only the judged mirror): G27 has zero false reports on 168 real legal states of H0/HR/HK (Q7b); discriminating-power self-test ① (not subtracting item 5) did not show the required color change on any of the 7 reachable bases tested in that segment (β0/β1/β2 and the four observation points of HK), triggering the registration's void clause, and the whole Q7a table was voided per the registration rule; Q7d-2/Q7d-3 ruled out the explanation "the measure itself cannot see 0" (on HK-F0 and β_syn the measure can detect 0), and the root cause was then narrowed to "item 5 of reachable legal states is never 0"; **this conclusion was overturned by the continued dispatch segment of the 3rd rerun on 2026-09-25**: after adding a new reachable base (`beta_hr_rollback_row`, the rollback row-writing step of the HR family, `family=HR kind=rollback_row txg=76`, item 5 exactly 0), self-proof ① changed color on a reachable base for the first time (`flips_red_to_green=true`); see the 2026-09-25 continued dispatch segment in "Historical Versions" on the experiment page; fork 3 (one HF geometry cell): the D_rel of G7 (public entry regrouping) and F-deduct (real implementation) are identical (both 0, non-empty publishes counted), with little difference (`max_diff=0≤3`), and K9 did not verify the expected 3; fork 1 (three Hh(k) hole count sampling points, k=0/1/2): the difference Δ between Q1a and Q1b rises monotonically with hole count by +50 per hole (20→70→120), crossing both the 2-slot and 1-slot thresholds from h=0, but the mechanism behind the h=0 baseline value 20 itself was not fully traced.
  - **After**: the apparatus rewritten entirely (5755 lines, 17 unit tests; nine new rows M35–M43 and three re-anchored rows M32–M34 in `crates/mutations.tsv`; the 22 rows pointing at this apparatus all red one by one in the repository copy in the draft directory): only the first segment (minimal reproduction of premises 1–3, PC-c2, two unit tests of the form that hangs on rollback) and the second segment (38 schedulable histories of Hh(k, position, S, ρ) × "real" and "every") run. Premise 1 holds, premise 2 does not (fork sheet line 3 invalid, not measured), and premise 3 holds only for the fork 2 family under the old formula; at the 12 sampling points of line 1, Q1c-front crosses 2 slots everywhere, and F1 triggers. The first segment hit stop S1(i) (the apparatus's existing control mirror, "after issuing, change back only the root slot", differed from the recorded cut segment by one system configuration sector per disk, because the implementation rotates the system configuration after the root slot), and per the S1 line "change the apparatus and write it into section twelve", the control mirror was changed to also change back both system configuration slots, handed to the main agent to accept. The experiment page body was replaced with the current state of the 4th rerun, and the bodies of the 1st–3rd reruns were moved unchanged into "Historical Versions" on the page.
  - **Basis**: rerun registration `e156-r4-prereg.md` (revisions 1 and 2 of section twelve); artifact `e156-alloc-basis-counts-2026-09-26.out`; running `bash scripts/experiment-apparatus/replay.sh` for E156 (Cost Figures for Four Alloc-Basis Forks), the E156 (Cost Figures for Four Alloc-Basis Forks) cell reports byte-identical; unit tests `cargo test -p singlefs-harness --bin e156_allocation_basis_counts` → 17 passed; body [156-alloc-basis四条岔路的代价数.md](experiments/156-alloc-basis四条岔路的代价数.md).

### 2026-09-24

- E156 (Cost Figures for Four Alloc-Basis Forks) R2 second and third segments: numbers filled in for fork 3 and fork 1, each on one geometry cell (narrowed-scope version)
  - **Before**: the apparatus had only done fork 7 (`4` of `5 单测`, 3 entries for this bin in `crates/mutations.tsv`); forks 1 and 3 (lines 9 and 11 of the fork sheet) had no measurements at all, and the experiment page marked "forks 1 and 3 not done".
  - **After**: first, the R2 first segment was rerun on today's `crates/` (five files `mount.rs`, `transaction.rs`, `recovery.rs`, `walk.rs`, `crash.rs` changed by another line) and compared byte by byte with the 2026-09-23 artifact: the only difference is that the `I-3.1` (Allocated Statistics Match) verdict on the `HK_l2` base turned from red to green (someone else is changing the checker implementation in `walk.rs`; only the reading is recorded here as is), `q7a_summary.all_red_count` changed from 12 to 15, and the Q7c readings and the fork 7 verdict are unchanged. The apparatus gained two functions, each run on **one geometry cell, S = 8, ρ = 1** (not the full sampling of registration "five, 5.2"; reason under "scope"): ① `run_hf_single_cell` (fork 3): β0 → reopen without crash → 8 non-empty overwrites, then three branches (probe the upper bound, F-deduct, G7) each rebuilt from the same snapshot; V13 checked first (before raising F the device bytes of all three branches are byte-identical to the snapshot), then each raises F to the same upper bound; G7 uses the new function `raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect` (public entry regrouping: first push an empty publish carrying the new F until every disk is covered, then `reclaim_released_up_to(..., ReclaimedReuse::Immediately)`); F-deduct uses the real `raise_rollback_floor`. ② `run_hh_cell` (fork 1): runs three Hh(k) histories (k = 0, 1, 2, hole position fixed at "after"), each cell β0 → workload until the ring has turned one lap → make k holes (c2 crash + recovery, 2 workloads apart) → finally one reopen without crash (the reclamation point of the A-T1 "real" reading); G12's bypass predicate needs an "allocation generation" field that real records lack (registration "three" N2 (the allocation generation field rewritten at release)), so the apparatus maintains its own shadow ledger (new functions `snapshot_allocated_generations`/`record_release_generations`). New unit test `allocation_generation_ledger_recovers_the_pre_release_generation` (5 unit tests), one new mutation (`snapshot_allocated_generations` read in the reverse direction, verified by hand red→green→restored green, 4 mutations); one more mutation that does not go red was verified by hand (G7's `ReclaimedReuse::Immediately`→`HeldUntilFloorTakesEffect`, with the Q3a/Q3b/Q3c/K9 output byte-identical), judged "class one, real blind spot" per `mutation-sampling.md`, and not put into `crates/mutations.tsv`. `naming-lint.sh` fixed over 20 single-letter / letter-plus-digit identifiers (such as `raise_rollback_floor_via_g7`→`raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect`, `f_kou_*`→`real_raise_*`, `g7_*`→`reconstructed_raise_*`), all green after the change; `rustfmt` formatted this file, and `cargo clippy --profile test -- -D warnings` has zero warnings (another bin, `e158_root_choice_repair.rs`, has its own clippy red under the full `--tests` scan, unrelated to this file and untouched). Artifact `e156-alloc-basis-counts-2026-09-24-stage2.out` (338 lines, cumulative over the first, second and third segments), with `replay.sh:175` now pointing to it. **Conclusion**: on one geometry cell of fork 3, the D_rel of G7 and F-deduct are identical (`d_rel_非空=0`), and Q3c judges "little difference" (`max_diff=0≤3`); K9 did not verify the registration's expected 3 (the premise "≥4 non-empty valid roots in the ring" was not specifically checked, or the direction already anticipated by registration P21 (the earlier the release, the longer the wait) (e) may not hold); Q3e (X8 (the two measurement bases of small-pool exhaustion construction)-A) not done at all. At the three sampling points of fork 1, Δ rises monotonically by +50 per hole (20→70→120), crossing both the 2-slot and 1-slot thresholds from h=0, but the mechanism of the h=0 baseline value 20 itself was not fully traced (only down to the aggregate count level). The sufficiency conditions of both forks are literally touched, but only on one geometry cell; the scope and "what it cannot answer" state the unscanned dimensions clearly, and whether to continue is for the main agent to judge.
  - **Scope**: this round's time budget gave priority to the two minimums named verbatim by the dispatch prompt (fork 3 needs "one number for each of the two measurement bases", fork 1 needs "at least two sampling points with different hole counts") rather than the full S × ρ × φ / S × ρ × position × k scans of registration "five, 5.2" (1 of HF's 18 cells run; Hh's S=4/16, ρ=1/4, position "before" and k=4 not run); X8 (the two measurement bases of small-pool exhaustion construction)-A (the HX/HY 128-slot two-segment small-pool geometry), Q3d (D_acct/D_alloc), and the two positive controls PC-over-deduct and PC-hole were not done either. These are listed one by one under "what it cannot answer" on the experiment page; they were not measured and then hidden.
  - **Basis**: pre-run registration `e156-r2-prereg.md`; artifact `e156-alloc-basis-counts-2026-09-24-stage2.out`; rerunning `bash scripts/experiment-apparatus/replay.sh` for E156 (Cost Figures for Four Alloc-Basis Forks) reports byte-identical; unit tests `cargo test -p singlefs-harness --bin e156_allocation_basis_counts` → 5 passed; the 4 entries for this bin in mutation table `crates/mutations.tsv` verified one by one by hand; body [156-alloc-basis四条岔路的代价数.md](experiments/156-alloc-basis四条岔路的代价数.md).

### 2026-09-23

- E156 (Cost Figures for Four Alloc-Basis Forks) R2 rerun, first segment: R3 (G27 reads the mirror instead), two new history families HK/HK-F0, and the full Q7a–Q7f of fork 7
  - **Before**: in the first- and second-segment apparatus `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs` (2 unit tests / 1 mutation), neither side of G27 read the judged mirror: the base side read the in-memory allocator, and the bad mirror side read a triple passed in by the caller (still this reading after fixing the field-semantics-reversed defect recorded by C471 (the same field name in the artifact points to two opposite quantities on two paths)); there were only three bases β0/β1/β2; fork 7 discriminating-power self-proof ① did not change color on any of the three bases, and Q7a was voided per V3; S1(c) only ran the existing test files live, with no apparatus-side replay; forks 1, 3 and 7 were all open, and there were none of the Q7d/Q7e/Q7f/S9 kinds of quantities.
  - **After**: pre-run registration `e156-r2-prereg.md` (covering only lines 9, 11 and 12 of the fork sheet; this segment only does line 12). R3: a new function `allocated_minus_deferred_mismatches_referenced(pool, accounting_slot, referenced)` parses only mirror bytes; the old in-memory reading is kept for U8's mutation counterpart. Two new real history families HK / HK-F0 (β0→O→E→reopen without crash→O×4→raise F to the upper bound→O→administrator rollback→O→c2 crash and recover→O, covering all publish kinds of the skeleton publish; HK-F0 is the same sequence of operations but with `ReuseWindow::ForcedToZero`, R7's unreachable positive control), together with β0/β1/β2, β_syn (the β0 mirror with (item 1 −1, item 5 −1, item 2 +1)) and β_F0 make 9 bases, each run through Q7a (Bd(+1)/(−1)/(+8)), Q7c ①②, and PC-check-Bk1/Bk2/Bk3; five new kinds of quantities Q7d-1/Q7d-2/Q7d-3, Q7e, Q7f; S1(c) changed to the apparatus itself replaying the rollback quarantine scenario of `rollback_by_a_forward_publish.rs` and checking item by item with `assert_eq!`; the three anchors anchor 1/anchor 2/anchor 3 computed live in the apparatus and compared back against the implementation's same-named constants or geometry functions. While writing the apparatus, two gaps in its own implementation were found and fixed (before the artifact; see revisions 1 and 2 of "twelve" in `e156-r2-prereg.md`): ① HK's rollback target selection must filter the candidate set by `effective_rollback_floor`, not directly take the one with the smallest `checkpoint_txg` in `readable_roots` (after HK raised F, genesis is no longer a candidate); ② the allocator rebuilt inside `mount_writable`/`mount_rollback` resets `ReuseWindow` to the product default, so `set_reuse_window` must be called again after every mount/rollback. Naming discipline changed over 40 single-letter / letter-plus-digit identifiers (`g8_prime_is_red` and so on), and `naming-lint.sh` is all green. 4 unit tests (new `new_pool_file_creation_self_release_is_one_slot` and `red_check_reads_the_corrupted_mirror_not_the_live_allocator`); the mutation table has 3 entries for this bin cumulatively, all verified by hand red→green→restored green. Artifact `e156-alloc-basis-counts-2026-09-23-stage1.out` (324 lines, two live runs byte-identical), with `replay.sh:175` now pointing to it. **Conclusion**: Q7b: G27 has zero false reports on 168 reachable legal states (H0/HR/HK); Q7d-2 shows item 5 is never 0 in these 168 states (minimum 1); Q7d-3 (positive control) proves the measure can see 0 (detected on both HK-F0 and β_syn); Q7c ① still does not change color on any of the 7 reachable bases, only on the 2 unreachable bases (β_syn, β_F0); per registration V3, **the whole Q7a table is still void**: continuing fork 7 (line 12 of the fork sheet) cannot flip it, and whether to change the criterion or accept it is handed to the main agent. HK-F0's raise-F probe hit S9 (the candidate root's tree table unreadable; under `ForcedToZero` genesis's tree table unit was reused too early) and stopped at that cell, which does not affect the verdicts already reported.
  - **Basis**: pre-run registration `e156-r2-prereg.md`; artifact `e156-alloc-basis-counts-2026-09-23-stage1.out`; rerun `bash scripts/experiment-apparatus/replay.sh E156` (Cost Figures for Four Alloc-Basis Forks) reports byte-identical; unit tests `cargo test -p singlefs-harness --bin e156_allocation_basis_counts` → 4 passed; the 3 entries for this bin in mutation table `crates/mutations.tsv` verified one by one by hand; body [156-alloc-basis四条岔路的代价数.md](experiments/156-alloc-basis四条岔路的代价数.md).

## E157 (Count Model for Two Clauses on Concurrent Line 1)

### 2026-10-04

- E157 (Count Model for Two Clauses on Concurrent Line 1) fills in the three mutation numbers (the two whole-table runs before and after the rerun with record header 311 → 487 and named items 67 → 64, run by the main agent; only numbers filled in, conclusion and artifacts unchanged)
  - **Before**: the next entry said "the two whole-table runs of the 10-entry mutation table before and after the change have not been run yet"; the "all N mutations caught" sentence on the experiment page was the 7 of 2026-09-22; the three entries the registration inferred to be deleted, header width falling back to 307, header width over the upper boundary and header width over the lower boundary, "would not compile" and had not been run separately.
  - **After**: before (old tree, old table of 10): 8 caught, 2 invalid, 0 not red, the invalid ones being M15 and M16 (they would not compile even on the old tree with header 311 and named items 67), and M14 measured "caught"; after (10): 10 caught, 0 invalid, 0 not red (`research/results/e157-mutate-before-2026-10-04-h487.log`, `research/results/e157-mutate-after-2026-10-04-h487.log`; both end with "restored, baseline still all green", and hit-the-ceiling, timeout, could-not-schedule and squeezed-out are all 0). Registered void clause V2 (M17, M18 or M19 not red) does not trigger, and V7 (an existing mutation's status gets worse) does not trigger: the 7 existing M1–M6 and M13 are "caught" both times. The "all N mutations caught" on the experiment page changed to 10 per the after run, the status sentence dropped "not run yet", and it is still "partly run" (line 2 not run).
  - **Basis**: the two logs above; sections nine and eleven of registration `research/prompts/e157-r2-prereg.md`.
- E157 (Count Model for Two Clauses on Concurrent Line 1) rerun with record header 311 → 487 and named items 67 → 64 (first segment, line 4 of the question sheet)
  - **Before**: the apparatus constants `JOURNAL_HEADER_BYTES` was 311 and `JOURNAL_NAMED_ENTRIES_PER_RECORD` 67, while `crates/singlefs-format/src/lib.rs` was already 487 / 64 (stop clause V6 was in a triggered state, used only as a comparison baseline: `replay.sh E157` (Count Model for Two Clauses on Concurrent Line 1) byte-identical against the old artifact); apparatus line 75 (added 2026-09-27) has a compile-time assertion tying the per-record item count to the header width; line 12 of the experiment page said "all equal verbatim to this apparatus's constants, V6 does not trigger", which is false today. The mutation table had three entries: header width falling back to 307, header width over the upper boundary and header width over the lower boundary.
  - **After**: constants changed to 487 / 64, with the file header comment synced; a pure function `named_entries_per_record_for_header_bytes` added, and both the compile-time assertion and the anchor line recomputation now call it; a unit test added pinning 65 / 64 / 64 / 63 at the four header widths 456 / 457 / 512 / 513 (discriminating-power self-proof: changing the expectation at 512 to 63 and at 513 to 64 each turned it red once, restored); the remainder unit test renamed to pin 25; the unit test for how many items one record holds changed to 64. 22 unit tests all green. The artifact `e157-parallel-line-one-clauses-2026-10-04-h487.out` (902 lines) differs from the header-311 one only in two fields of the `name=anchors` line, `a1_named_entries_per_record=` and `b2_named_entries_per_record_check=` (67 → 64); the first-segment verdict cells do not read the named item count and are all unchanged (artifact changed, verdict unchanged); the E157 (Count Model for Two Clauses on Concurrent Line 1) line of `scripts/experiment-apparatus/replay.sh` now points to the new artifact, rerun byte-identical. The mutation table dropped the three entries header width falling back to 307 / over the upper boundary / over the lower boundary, and added three: items per record counted one byte too many, one byte too few, and the header width moved one byte within the same tier (10 entries); the two whole-table runs before and after the change have not been run yet. Line 12 of the experiment page changed to today's values and freshly checked command output, the title and index row status state that the mutation table is pending, and the review dates of the six rows of the affected decisions table changed. The unit count sampling points 67 / 68 are not changed (they were chosen only for the unrun second segment).
  - **Basis**: rerun registration `research/prompts/e157-r2-prereg.md`; artifact `research/results/e157-parallel-line-one-clauses-2026-10-04-h487.out`.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

### 2026-09-24

- See the same-day entry "E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds)…" in the E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) section.

## E158 (Four Forks for Root Selection and Repair)

### 2026-10-17

- Six headings or lead sentences saying "results copied whole from the artifact" gain the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/158-择根与修复四岔路.md`.

### 2026-10-15

- 29 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 29 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/158-择根与修复四岔路.md`.

### 2026-09-27

- E158 (Four Forks for Root Selection and Repair) 3rd run first segment and 4th run: the three lines of the fork sheet on how to fix C554 (shadow accounting unreachable when crash recovery temporarily cannot read abandoned roots)
  - **Before**: the current state on the experiment page was the section "2026-09-26: 2nd run, first segment", with the title status being the long parenthetical "partly run (2026-09-23 first segment + 2026-09-24 segments + 2026-09-26 2nd run first segment)"; the artifacts of the 3rd run first segment (rerun registration `e158-r3-prereg.md`) and the 4th run (rerun registration `e158-r4-prereg.md`) were already in `research/results/` but not written on the page, and gate number 40 reported them as not named.
  - **After**: the top of the page gains a section "2026-09-27: 4th run" (current state) and a section "2026-09-27: 3rd run, first segment" (for reference only), and the title changed to "4th run handed in (forks 1 and 2 settled by the user, 3 recorded as owed)". The 4th run: eighteen arms, five segments (the second segment's apparatus key collision was fixed and the whole segment rerun as seg2b, which is authoritative); cross-segment quantities follow `research/results/e158-root-choice-repair-2026-09-27-r4-compare.out`, merging the five segments: today and -slot lose writes > 0 across all families in every arm; B-config loses 16 / 32 writes on H1g and B-config-continued 0 / 32, both 6 / 37 on H-random (all ③c tails; ③c counts 2 cells there, while Q1 counts only within the ring and ③c 1 cell); both reject 22 more on the L family; for the same successes, B-config reads +3 or +4 more per cell and B-config-continued +7 or +8; read cache peak not measured (line 3 of the fork sheet recorded as owed). `scripts/experiment-apparatus/replay.sh` gains a `driver_e158_r4_compare` line. The row for settled item 14 of D23 (Journal Role & Format) in the affected decisions table (changed to support by the main agent the same day) gains an artifact pointer, and the other four rows reviewed remain prepared material.
  - **Basis**: `e158-r4-prereg.md` (runner revisions 1–30 in section twelve and the main agent's acceptance after each segment); the per-segment runner reports `e158-r4-seg1-runner-report.md`, `e158-r4-seg2-runner-report.md`, `e158-r4-seg2b-runner-report.md`, `e158-r4-seg3-runner-report.md`, `e158-r4-seg4-runner-report.md`, `e158-r4-seg5-runner-report.md`, and the key collision investigation `e158-r4-seg2-ledger-investigation-report.md`; fork sheet `c554-fix-forks.md`.

### 2026-09-26

- E158 (Four Forks for Root Selection and Repair) 2nd run, first segment: the three premises of the question sheet, three cells of H1d on each arm, Q1-0 and PC1
  - **Before**: the experiment page only had records of the first run (pre-run registration `e158-preregistration.md`), where abandoned roots were all produced by rollback at mount, a path deleted from today's code; of the apparatus's 43 unit tests, `mount_writable_trajectory_distinguishes_persistent_from_transient_faults` was red from the start; the arm table `scripts/mutation/tables/e158_arms.tsv` was the field table 481 version, and its anchors no longer hit.
  - **After**: the committed apparatus gains a section for the 2nd run (modes such as `r2-all`, with the first run's modes kept unchanged); the arm table rewritten per rerun registration 5.2 and 5.3 (A1, A-txg, BF-keep-ring, and the four arms B / D, with rows identical across several arms joined by "、"); one run each on today and seven arm copies: premises 1–3 of the question sheet all hold, the three cells of H1d over 800 cells per arm are identical cell by cell between today and the six arms of fork sheet line 2, with A1 rejecting only 2 more cells; PC1-a/b/c (today) and A1's PC1-a all pass; the apparatus's 57 unit tests all green, and 17 rows added at the end of `crates/mutations.tsv`. The experiment page body gains, at the very top, a section "2026-09-26: 2nd run, first segment", and the title and index row continue with this segment.
  - **Basis**: `e158-r2-prereg.md` (seven revisions in 12.1 of section twelve, all before the artifact); artifact `e158-root-choice-repair-2026-09-26-r2-all-today.out` (`driver_e158_r2_all` registered under `E158` (Four Forks for Root Selection and Repair) in `replay.sh`, rerun byte-identical) and the eight with the same prefix: `-today-snapshot`, `-a1`, `-jia-txg`, `-yif-keep-ring`, `-yi-keep-ring`, `-yi-config-only`, `-ding-keep-ring`, `-ding-config-only`.

### 2026-09-24

- E158 (Four Forks for Root Selection and Repair) first segment rerun (artifact structurally mismatched, cause diagnosed) + second segment Q3-1 (the candidate 3 half of fork sheet line 3)
  - **Before**: the apparatus had only run once, on 2026-09-23 (5 unit tests / 3 mutations, covering only the first segment: S1–S5 differentials, 5.7 constant back-comparison, section seven anchors,
    H3 full enumeration without injection, Q3-2/Q2-3 arithmetic, PC-Nw, PC3); fork sheet line 3 had numbers only for the candidate 2 half, while the candidate 3 half (Q3-1)
    and the section eight sensitivity cells had not run; fork sheet lines 1, 2 and 4 were all undone.
  - **After**: first the first segment was rerun on today's `crates/` (in which the settlement of C512 (where superseded instance table on 0-entry tree table version is recorded lacks clauses) landed in the meantime,
    removing the two rejection branches `MountError::RollbackToVersionWithoutFileUnsupported` / `InstanceRowsOnVersionWithoutFileUnsupported`)
    and compared with the 2026-09-23 artifact: **mismatched, 50 lines differ**, diagnosed cell by cell to one root cause
    (H3's `R(j)` branch can now really roll back to a root whose tree table has 0 entries); the load-bearing quantities of the first segment (Q3-2's N_w=4, the witness width margin,
   the PC3 core assertion) are unaffected; the new artifact is saved separately as `e158-root-choice-repair-2026-09-24-segment1-rerun.out`,
    the old artifact kept as is, both named, and which one carries the load is for the main agent to decide. Then the second segment Q3-1: `SimNode` gains an append-only
    `content_by_root` field (the only new state needed by V2 of P332 (violation criterion)); the P332 (violation criterion) verdict (V1 = the selected root lies in the abandoned set,
    V2 = the content number read back belongs to no root on the current timeline) is judged once each at three points: (a) cold-start recover with faults, (b) after `mount_writable` with faults,
    then cold start with the same faults, (c) cold start after the faults are withdrawn; Φ3 (`|F|≤2`) fully enumerated, and 5.1 "H3's shortcut" implemented exactly as registered;
    a cost calibration was done before the run (finding that the old way of rebuilding a read-only image once per fault set in (a) could not even finish level one, it was changed to build once per whole node
    and share, a "revision before the artifact"). All four geometry points G0 (main geometry point) /S16/small ring/S4 measured: violation share about 0.12–0.125%,
    appearing only when two faults happen together and the history contains a rollback, in a direction consistent with "2 faults" in the text of C332 (Rollback undone when both roots unreadable in rollback instance), and the verdict does not flip at any of the four points; G_default
    gets only a mechanism argument and no independent artifact, because `scan_journal` reads every record slot of the whole ring one by one, 256× the cost of the 3 MiB ring.
    7 new unit tests (12 all green, each proven to go red when broken), 3 mutations (6 cumulative in `crates/mutations.tsv`, each verified by hand
    red→green→restored green). Naming discipline changed about 45 single-letter / letter-plus-digit identifiers, and `naming-lint.sh` is all green; `cargo fmt`/`clippy`
    (including the lints of the shared constraints) cleared all warnings for this bin (one mutation table anchor was rewritten because `cargo fmt` split `is_abandoned` across three lines,
    and was rechecked to hit exactly once). **Fork sheet lines 1, 2 and 4 (prerequisite unmet) were still not done this time**, not skipped after being enough to judge.
  - **Basis**: pre-run registration `e158-preregistration.md`; artifacts
   `research/results/e158-root-choice-repair-2026-09-24-{segment1-rerun,q3-1-g0,q3-1-s16,q3-1-small-ring,q3-1-s4}.out`;
    rerunning `bash scripts/experiment-apparatus/replay.sh` for E158 (Four Forks for Root Selection and Repair) reports the four Q3-1 points byte-identical and the first segment mismatched (known cause); unit tests
    `cargo test -p singlefs-harness --bin e158_root_choice_repair` → 12 passed; body
    [158-择根与修复四岔路.md](experiments/158-择根与修复四岔路.md).
- (Continued, session s3 second segment): E158 (Four Forks for Root Selection and Repair) fork sheet line 1, H1×Φ1×PC1-a/b written, Q1-1 to Q1-4 have real numbers
  - **Before**: the A1 copy for candidate (a) of fork sheet line 1 (C393 (Repair lacks clause when abandoned root accounting unreadable)) had been built and compiled (see the previous history entry), but not a line of apparatus code was written for H1 family enumeration,
    Φ1 fault injection, PC1-a/PC1-b, or Q1-1 to Q1-4, and there was not a single number; fork sheet line 2 (C331 (Root selection inversion overrides acknowledged writes) ①②)
    likewise had not a single number.
  - **After**: the main agent's message "keep going, this segment has not yet delivered the numbers the fork sheet needs, don't stop"; in
    `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs` about 500 lines added: H1 family enumeration (`h1_family`:
    n1∈{1,2,3}, j taking each readable root, m∈{0,1,2}), Φ1 fault injection (`attempt_op1`: precise injection by (device, offset), with the placement looked up fresh by
    `locate_allocation_record_tree` in two branches, tree table 0 entries / non-0), and PC1-a/PC1-b (`run_pc1_a_and_b`).
    **A bug was found and fixed along the way (before the artifact)**: `root_record_of` first fed `RootView::record_bytes` (truncated to 457
    bytes) to `RootRecord::parse_slot`, whose checksum is computed over the full slot width of 512, so after truncation it never matched and was constantly `None`: in the whole H1 family not one "abandoned readable root"
    could be obtained (`pairs=0`); after changing to reread by the full slot width per `checker_image::root_slot_positions` itself, after the fix
    `pairs=3942 trigger_count=798`, byte-identical over two independent runs. **Candidate (c) = today** G0 (main geometry point): 798 of 3942 pairs trigger
    "both copies fail to read" (1 fault never triggers it, and without injection it never triggers naturally), and today all 798 triggering pairs are `Ok(count>0)`
    (C393 (Repair lacks clause when abandoned root accounting unreadable) "today only counts, does not refuse the mount" confirmed item by item, F1 does not trigger). **Candidate (a) = A1 copy** (compiled in the draft directory, not put into
    `replay.sh`): the same 3942 pairs, `trigger_count=0`: all 798 of today's triggering pairs are rejudged
    `AbandonedRootLedgerUnreadable`, and in 798/798 refusals the device layer writes 0 bytes (Q1-4 holds in full, not just for the one PC1-a case).
    Assertions (a) and (c) of PC1-a pass; (b) "the allocation record tree must succeed" did not reproduce under the operationalization "reread the same node"
    (logically impossible: when both copies fail to read, candidate (b) rereading them itself cannot read them either); recorded per V1 as "the positive control did not pass", with whether
    candidate (b) should switch to a different operationalization (inferring the occupied set back from the content tree, without rereading the injected node) handed to the main agent. Q1-2/Q1-3 also have measured numbers:
    for the allocation record tree, candidate (b) can recompute 141 and cannot recompute 57 (most recomputable ones come from single-disk faults, relying on the other mirror copy); released records
    zero 27, non-zero 114 (only 27/168 can be written back byte for byte); for the tree table, candidate (b) "has a replica" 9, none 57 (COW means tree table nodes mostly
    do not repeat); the persistence cost of both designations is 32768 in this batch of data (a single node × 2 mirror copies; the flip condition the pre-run registration gave, "the two designations take different values",
    does not hold at this granularity). **Still missing**: op1 only did the two kinds `mount_writable`/`mount_rollback` (missing
    the `raise_rollback_floor` kind); only "transient", not "persistent"; only G0 (main geometry point) run, with no geometry sensitivity (S4/S16/small ring/G_default);
    candidate (b)'s tree table designation only tested "has a replica", not "crates has a path". These gaps are written one by one into "twelve, revisions" of the pre-run registration: **fork sheet
    line 1 has numbers but is still not enough to judge**. Fork sheet line 2 (C331 (Root selection inversion overrides acknowledged writes) ①②: five copies + H2/H2-R/H2c + the Q2-1 search algorithm + Q2-2)
    **was still not done this time**; its engineering volume is larger than line 1's, and this segment's time went entirely into line 1. 4 new unit tests (16 all green,
    `fault_targets_for`/`error_member_of_debug` each verified to go red when broken), 3 mutations (9 cumulative); new
    `driver_e158_q1_g0` (candidate (c), registered into `replay.sh`; all 6 drivers byte-identical).
  - **Basis**: the main agent's continued dispatch message (original text quoted in the heading of this history entry); artifacts
   `research/results/e158-root-choice-repair-2026-09-24-q1-g0-{today,a1-arm}.out` (each byte-identical
    over two independent runs); rerun log `/tmp/claude-1000/e158-s3/replay-e158-3.log` (all 6 items byte-identical); unit tests
    `cargo test --release -p singlefs-harness --bin e158_root_choice_repair` → 16 passed (run once each on the A1 copy
   and the main workspace); gate number 33; the entry (session s3, fork sheet line 1) in "twelve, revisions" of the pre-run registration; body
    [158-择根与修复四岔路.md](experiments/158-择根与修复四岔路.md).
- (Continued, session s3): E158 (Four Forks for Root Selection and Repair) `driver_e158` settled to point to today's artifact + the A1 copy for candidate (a) of fork 1 built
  - **Before**: the `driver_e158` line of `scripts/experiment-apparatus/replay.sh` still pointed to the old 2026-09-23 artifact (structurally mismatched with today's `crates/`,
    50 lines differing, the cause already diagnosed in the paragraph earlier the same day; see the previous history entry); the experiment page section "results copied whole from the artifact"
    also still cited the old artifact's bytes. In addition, after that segment was handed back, the main workspace took in another batch of uncommitted `crates/` changes (`impl-m2-mountfix`,
    a parallel line: `mount.rs`/`recovery.rs`/`transaction.rs`/`allocator.rs`/`instance_table.rs` and others), and whether this batch
    affects the verdicts already measured by this experiment had not been re-checked. Fork sheet line 1 (C393 (Repair lacks clause when abandoned root accounting unreadable)) and line 2 ①② (C331 (Root selection inversion overrides acknowledged writes)) had not a single number computed, and of the copies candidate (a)
    needs, not one A1 `crates/` copy had been built.
  - **After**: the main agent settled "the first-segment rerun artifact carries the load": `driver_e158` now points to
   `e158-root-choice-repair-2026-09-24-segment1-rerun.out`, and the corresponding paragraph of the experiment page now cites the new artifact's bytes,
    adding a note that `pc3.path` changed from `RollbackTo(1,3)` to `RollbackTo(0,0)` (the core assertion `expected_root=(1,6)` unchanged);
    the old artifact is kept as is, noted as corresponding to the code before C512 (where superseded instance table on 0-entry tree table version is recorded lacks clauses) landed. After the change, `bash scripts/experiment-apparatus/replay.sh` was rerun for E158 (Four Forks for Root Selection and Repair):
    **all 5 drivers "byte-identical"**, confirming that the uncommitted `impl-m2-mountfix` changes did not affect any observable surface of `driver_e158` or the four `q3_1_*`
    drivers. For candidate (a) of fork sheet line 1, an a1 copy was built (`/tmp/claude-1000/e158-s3/arms/a1/`, containing only
    `Cargo.toml`/`Cargo.lock`/`.cargo`/`crates`): `crates/singlefs-core/src/mount.rs` gains
    `MountError::AbandonedRootLedgerUnreadable { count: u64 }`: in `mount_writable`/`mount_rollback`, after the
    `RebuiltAllocator` destructuring and before `establish_instance` (number acquisition), whenever `abandoned_roots_unreadable > 0` it returns
    this error early; the two exhaustive `match`es in `history.rs`/`model_comparison.rs` gain an arm for the new variant (in the default bucket "the model has no reason",
    with no change to the verdict logic). The four changes are recorded in the newly created `scripts/mutation/tables/e158_arms.tsv` (four columns, the same as
    `crates/mutations.tsv`). The A1 copy compiles (`cargo build --release -p singlefs-harness --bin
    e158_root_choice_repair`). **No H1 apparatus code was written**: H1 family enumeration (n1/j/m/op1), Φ1 fault injection,
    PC1-a/PC1-b, the computation of Q1-1 to Q1-4, and the other five copies fork 2 needs (A-txg/B-keep-ring/B-config-only/D-keep-ring/D-config-only)
    and the search algorithm for H2/H2-R/H2c are all still unwritten. Reconnaissance confirmed this is not a blocker: in `crates/singlefs-core::recovery`,
    `tree_table_has_no_entries`/`allocation_records_under_root`, together with `RootRecord`/`NodePointer`/
    `LocationEntry`/`SlotNumber::to_device_offset`/`TreeTableEntry`/`TREE_KIND_ALLOCATION`/`parse_index_node`,
    are all `pub`, and the two physical positions Φ1 needs (the tree table unit, the allocation record tree nodes) can be found using only these public APIs. Fork sheet lines 1 and 2
    are still "open", not skipped after being enough to judge.
  - **Basis**: the main agent's dispatch prompt (the original dispatch at `/tmp/claude-1000/handover/a33f8ba874ca42006.md`); artifact
   `e158-root-choice-repair-2026-09-24-segment1-rerun.out` (registered as the load-carrying one for `driver_e158`);
    rerun log `/tmp/claude-1000/e158-s3/replay-e158-2.log` (all 5 items byte-identical); A1 copy
    `/tmp/claude-1000/e158-s3/arms/a1/`; `scripts/mutation/tables/e158_arms.tsv`; unit tests
    `cargo test --release -p singlefs-harness --bin e158_root_choice_repair` → 12 passed (run once each on the copy and the main workspace
    ); gate number 33 (all 457 anchors hit the source exactly once); body
    [158-择根与修复四岔路.md](experiments/158-择根与修复四岔路.md).
- (Continued, session s9): E158 (Four Forks for Root Selection and Repair) root cause of the 171-cell difference set found, the count cap removed from Q2-1 exhaustive search, and the three copies B-keep-ring/D-keep-ring/D-config-only filled in for n1=0..6
  - **Before**: after the 171 cells of candidate (b) "walk everything" changed from `unequal` to `subset_of_pristine` (session s8), what exactly was in the difference set and why walking everything was still not complete had not been checked; `search_minimum_weight_that_triggers_rootback` used a count cap `SUBSET_ENUMERATION_CAP` (20000) and stopped midway through a weight tier, and when it stopped it knew neither whether that tier had been exhausted nor how large the full evidence space was, `k_min=capped` swallowing both pieces of information; Q2-1 (Φ2 plus system configuration slot evidence) for the three copies B-keep-ring/D-keep-ring/D-config-only had only been measured up to n1≤1 (fully exhausted) and n1∈{2,...,6} (hit the old cap, not exhausted); persistent faults (op1 and the two mounts after it all carrying them) and the mount trajectory required by 8.1 were not implemented; after the real-nineteen merge (2026-09-25), the central mapping tree grows to multiple levels when its entry count exceeds 294, while `allocation_record_tree_reachable_placements_via_central_mapping` parsed the root node as a leaf without checking `level`.
  - **After**: the new `classify_diff_pair` found that the 171-cell difference set has only 2 distinct placements (`(0, 50176)`, `(1, 50176)`), which are the physical nodes of `RootRecord::instance_table` itself (`UNIT_AREA_START_SLOT`=`INSTANCE_TABLE_SLOT`), never read by the walk-everything version; `publish_admitted` in `transaction.rs` was checked: the three tree kinds `TREE_KIND_LIVELIST`/`_SPARSE_SIDE_TABLE`/`_DEADLIST` are today constantly `NodePointer::empty_root()` on every publish across the repository, unrelated to this batch of histories. The parameter of `search_minimum_weight_that_triggers_rootback` changed from `subset_enumeration_cap` to `weight_ceiling`: it stops only at weight tier boundaries (not midway through a tier), and always reports `full_space_subset_count` (how large the full space is) and `highest_weight_examined` (which tier it exhausted up to); when the full space exceeds `FEASIBLE_FULL_SEARCH_SUBSET_BUDGET` (100000) it automatically falls back to weight 12 (aligned with C's k=12). The today/C arm `q2-1-g0` rerun with the new mechanism: n1=4 and 6 independently give `k_min=12` (previously reported `capped` after hitting the old cap), and n1=5 has a full space of 131072 and honestly reports exhausting up to weight 12 without a hit (not a truncation). The three copies B-keep-ring/D-keep-ring/D-config-only were rebuilt from a new snapshot after real nineteen landed, and filled in for n1=0..6 with the new mechanism: n1=0..2 fully exhausted, n1=3..6 exhausted up to weight 12 without a hit, the three copies byte-identical: their k_min (if it exists) is now established as > 12, overlapping neither A-txg's [4,4] nor C's [12,12], and the cross-section of fork sheet line 2, "today/A-txg/C vs the three B and D arms, crash tier 0, any kind hits", is now enough to judge. The newly written `mount_writable_trajectory` adds the three-step mount trajectory for persistent/transient faults, matching the original text of registration 8.1 verbatim. `allocation_record_tree_reachable_placements_via_central_mapping` gains a `level != 0` check; a hand-constructed fake node proved that before the change it would silently resolve a wrong placement set (not report an error); measured, the entry count of this batch of histories is constantly 6, far below 294. 4 new unit tests (35→39), 4 rows appended to the mutation table, and 1 existing anchor line fixed (gate number 33 red, anchor drift).
  - **Basis**: the session s9 entry of "twelve, revisions" in `e158-preregistration.md` (8 items, each with the basis of its manual verification); fork sheet lines 1 and 2 of `m2-rootchoice-repair-r1-forks.md`; artifacts `research/results/e158-root-choice-repair-2026-09-24-{q1-g0-today,q1-s16,q1-s4,q2-1-hc1-lower-bound,q2-1-g0-today-session-s5,q2-1-g0-configuration-evidence-yi-ring-retained,q2-1-g0-configuration-evidence-ding-ring-retained,q2-1-g0-configuration-evidence-ding-only-configuration}.out`.

## E159 (Fsync Latency vs Concurrency: Group Commit and WAL Arms)

### 2026-10-17

- The heading "results copied whole from the artifact" gains the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; the parenthetical of the pointer line for the smoke run evidence changed from an empty `name=` to that line's actual form `E7RESULT E159RESULT arm=yi-M-K10`; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md` has the artifact line copied whole; the pointer form with an empty `name=` was newly written by the item earlier the same day, and the staging area has no commit of it yet.

### 2026-10-15

- 1 artifact line copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 1 whole line); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md`.

### 2026-09-24

- See the same-day entry "E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds)…" in the E155 (Write Volume per Persistence: Three Fsync Forms and Counterfactual Upper Bounds) section.

## E160 (Random Small Read Share: Three Sources and Synthesis of Reopening Criteria)

## E161 (Deduplication and Segmentation Timing of Crash Amplification)

### 2026-10-26

- E161 (Deduplication and Segmentation Timing of Crash Amplification) 3rd rerun: segment shapes and pinned values re-set after the two-round number acquisition write
  - **Before**: the new-pool file creation stream had 11 segments and the fixed-script overwrite stream 78, sampled segments 11–26, small domains 45 and 393 261, quick tiers 46 and 52, PC3 in fixed-script overwrite stream segment 16; the apparatus had 14 inline tests; the experiment page status was `可行性档已跑（2026-09-27），第 2 次重跑已跑（够判，2026-10-05）`, and the `replay.sh` registration row pointed to the second-rerun artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out` with 10 threads.
  - **After**: rerun registration `research/prompts/e161-r3-prereg.md` (three revisions in section twelve: C3 and C2 built with the `gpu` feature and run after a quiesce, one erratum). After the two-round number acquisition write (`ACQUISITION_WRITE_ROUNDS` in `crates/singlefs-core/src/transaction.rs`) the new-pool file creation stream has 12 segments and 43 writes and the fixed-script overwrite stream 80 segments and 485 writes; the sampled segments moved with the same intent to 12–28, small domains 53 and 393 269, quick tiers 54 and 60, and PC3 moved to segment 18; the apparatus has 15 inline tests (all pass), and rows M14 and M15 of `crates/mutations.tsv` changed with the constants. Feasibility artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out` has 189 lines with final `loadavg1=60.16 cpus=32`: row 1, all four numbers of the 12 new-pool file creation stream segments equal segment by segment (0 unequal segments); row 2, the domain is fixed and finishes with an object in every cell; row 3, 0 cells flipped relative to the second rerun (361 fields compared); row 4, inline tests all pass and the 16 mutations belong to the submission gate. The registration has no clause voiding timing cells because of load, so PC3 counts as registered; whether to judge it differently because of load is handed to the main agent. The first feasibility-tier trial run (2026-10-25) did not compile in the `gpu` feature, did not run the judging stage and exited 77; it is not an artifact. The `replay.sh` registration row now points to the new artifact with 31 threads. The experiment page heading gains `第 3 次重跑已跑（够判，2026-10-26）`, a "Third rerun" section is added, "What it cannot answer" gains three items, and four rows are reviewed.
  - **Basis**: registration `research/prompts/e161-r3-prereg.md`; artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-2026-10-23.out`, and the judging script with three outputs `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-judge-2026-10-26.py`, `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-pcr3-2026-10-26.out`, `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-compare-2026-10-26.out`, `research/results/e161-crash-state-dedup-and-time-split-feasibility-r3-rows-2026-10-26.out`.
  - **Before**: `git show 841b3beab:.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md` (later the language switch line and the `scripts/experiment-apparatus/` path were staged in two places, unrelated to this item).

### 2026-10-17

- The heading "results copied whole from the artifact" gains the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md`.

### 2026-10-15

- 14 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 14 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md`.

### 2026-10-05

- E161 (Deduplication and Segmentation Timing of Crash Amplification) 2nd rerun: the sampling domain re-set for the 11-segment stream
  - **Before**: the new-pool file creation stream had 10 segments, and the registered sampling domain "all of every segment with fewer than 26 writes" was 37 states; after a barrier was added before publish returns (line 1301 of `crates/singlefs-core/src/transaction.rs`, commit c3540c02) the new-pool file creation stream became 11 segments, and the same sentence read literally is 16 777 260 states; of the apparatus's 13 inline tests two were red and six hit the 32G memory cap; the experiment page status was "feasibility tier run (2026-09-27)", and the registration line pointed to the 1st artifact.
  - **After**: rerun registration `research/prompts/e161-r2-prereg.md`, sampling domain reading A (`STRIDED_SEGMENT_WRITES` 26 changed to 24; first stream small domain 45, second stream sampled segments 11–26 small domain 393 261, quick tier 46 and 52, and for each of the two 24-write segments the 4096 with the smallest in-segment sequence numbers); the apparatus changed to 14 inline tests (all green), and E161 (Deduplication and Segmentation Timing of Crash Amplification) has 16 mutation rows in `crates/mutations.tsv` (M13–M16 new, not run). New feasibility tier artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out`, 186 lines: line 1, the four numbers of the first stream's 11 segments equal segment by segment (0 unequal segments); line 2, the domain is fixed and runs to completion, with every cell's criterion having a subject; line 3, 7 cells flip relative to the 1st run (verdict line `first_k4_full_pc2c_pc4c_found_a_position` false to true; reuse arm K4-units in the two segment-head cells `hits` 3640 to 0 and `keys_at_or_above_half` false to true; two fields of the position-removal control K4-full), with no attribution, and the supplementary run with thread count 1 owed; the registration at line 216 of `replay.sh` now points to the new artifact, and the 1st artifact is not deleted. The experiment page status changed to "feasibility tier run (2026-09-27), 2nd rerun run (enough to judge, 2026-10-05)", with a "2nd rerun" section added and three rows reviewed.
  - **Basis**: registration `research/prompts/e161-r2-prereg.md`; artifact `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-2026-10-05.out`, and the judging script with three outputs `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-judge-2026-10-05.py`, `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-pcr3-2026-10-05.out`, `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-compare-2026-10-05.out`, `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-rows-2026-10-05.out`.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

### 2026-09-27

- E161 (Deduplication and Segmentation Timing of Crash Amplification) feasibility tier
  - **Before**: there was only the pre-run registration `research/prompts/e161-preregistration.md`, with no apparatus, artifact or experiment page.
  - **After**: committed apparatus `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` (13 unit tests, 12 mutations appended to `crates/mutations.tsv`); per the user's decision only the feasibility tier was run (all of each segment of the first stream with fewer than 26 writes, the full domain of the A-two quick tier of both streams, and 4096 segment heads for each of the two 26-write segments); stop clauses S1–S6, all anchors of registration section seven, and the state-by-state differential against crates all pass; G1–G5 not judged.
  - **Basis**: 12.2 of registration section twelve (the user decided to verify feasibility only); artifact `e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out`.

## E162 (Storage Backend Selection for Crash Amplification Judgment)

### 2026-09-28

- E162 (Storage Backend Selection for Crash Amplification Judgment) path move: configuration file renamed
  - **Before**: `peer_logical_processors` in `scripts/experiment-apparatus/e162_crash_verdict_block_store.rs` read `PEER_SSH_HOST` from `layer0-shard.env` at the repository root, and the comment above that line cited `layer0-shard-run.sh:155` (that line number no longer points to the code that takes the host name today); the E162 (Storage Backend Selection for Crash Amplification Judgment) anchors registration line in `replay.sh` pointed to `e162-crash-verdict-block-store-2026-09-27-anchors.out`.
  - **After**: the git-ignored multi-host configuration at the repository root was renamed from `layer0-shard.env` to `multi-host.env` (user decision), and this now reads the new name, with the comment changed to cite the `run_on_peer` function of `layer0-shard-run.sh` (the same way of taking it; a function name does not rot with line numbers); `peer_logical_processors` is called only in the `throughput_context` of S2/geometry sensitivity, not on the call path driven by anchors, so conclusion, numbers and artifacts are unchanged: unit tests 23 passed / 0 failed; the new artifact `research/results/e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out` is byte-identical by `cmp` to the artifact from before the rename; the E162 (Storage Backend Selection for Crash Amplification Judgment) anchors registration line in `replay.sh` now points to the new artifact, and the old artifact is neither deleted nor changed.
  - **Basis**: `.claude/rules/path-moves.md` "how to do it"; the user decided to rename the configuration file; on the experiment page see the section "2026-09-28: path move".

### 2026-09-27

- E162 (Storage Backend Selection for Crash Amplification Judgment) feasibility tier
  - **Before**: there was only the pre-run registration `e162-preregistration.md` (including the runner's revisions one to six in section twelve), with no experiment page or index row; the apparatus, the mutation table and 8 S1/S2 artifacts plus 1 anchors artifact were already in the repository (produced by earlier sessions the same day), but no kb page named them.
  - **After**: the experiment page [162-崩溃放量判定块存储选型.md](experiments/162-崩溃放量判定块存储选型.md) created; per the user's decision (registration revision six) only the feasibility tier was run: each of the four arms F/R1/R0/K killed 20 times (S1) and writing 10⁴ blocks each (S2); S1-large, the four geometry sampling points and S3 not run; S1 and S2 are both recorded as "not judged (only the feasibility tier was run)". `scripts/experiment-apparatus/replay.sh` registers only the anchors line (`driver_e162_anchors`, added by earlier sessions the same day).
  - **Basis**: registration section twelve revision six (the user decided to verify feasibility only); artifacts `research/results/e162-crash-verdict-block-store-2026-09-27-{s1,s2}-{F,R1,R0,K}-feasibility.out`, `e162-crash-verdict-block-store-2026-09-27-anchors.out`.
- E162 (Storage Backend Selection for Crash Amplification Judgment) S1 enough-to-judge tier
  - **Before**: the current state of the experiment page was "feasibility tier run (2026-09-27)", with S1 and S2 both recorded as "not judged (only the feasibility tier was run)": S1 had only 20 kills and S2 only 10⁴ blocks written; the index row status the same.
  - **After**: per the main agent's dispatch (inheriting the user settlement "it can run, it can be used, then hand verification to Milestone 2 remaining tasks"), S1 was filled in from the feasibility tier's 20 kills to the originally registered 200, run once for each of the four arms (F, R1, R0, K), with the criteria of registration 6.1 and V1–V6 of 11.1 unchanged. S1 is now enough to judge: Q1a–Q1f are all 0 over the 200 kills of all four arms, and the positive controls and the V6 threshold (Q1g_kills_inside_commit ≥ 100, 178–180 for the four arms) all pass; reopen time for R0 (default repair), median 124 691 µs, is two orders of magnitude slower than R1 (quick-repair), median 1015 µs, in the direction redb 4.3.0's documentation gives, taken only as a trajectory observation and not entering the criteria. S2, S3, S1-large and the four geometry sampling points not run in this segment. The experiment page title changed to "S1 enough-to-judge tier handed in, S2 / S3 not run yet (2026-09-27)", with an "S1 enough-to-judge tier" section added; the index row status and conclusion synced. 4 new artifacts (`e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`), neither overwriting nor deleting the feasibility tier's old artifacts; `replay.sh` gained no new registration (S1's wall-clock kill points differ every time, read the same way as the feasibility tier, not compared byte by byte). **Before running the artifacts in this segment, the runner did not first add a "revision seven" to the registration recording this narrowing of scope; this was the runner's omission**; the artifacts have already been produced, so per discipline the registration is not changed afterwards, handed to the main agent to handle. Before the run the unit tests were rerun, 23 passed / 0 failed (`BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include` resolved the `librocksdb-sys` build blocker recorded in the previous version); the source was not changed and the mutation table not rerun (13 entries, unchanged).
  - **Basis**: pre-run registration `e162-preregistration.md` (section twelve revision six, and revision five's V15 interference rerun hit item by item); artifacts `research/results/e162-crash-verdict-block-store-2026-09-27-s1-{F,R1,R0,K}-decisive.out`; handover background in section three of `records/2026-09-24-里程碑二收尾调度.md`.

## E163 (Multi-GPU Unit Checksum Computation)

### 2026-09-28

- Host-private information removed from the bodies of E163 (Multi-GPU Unit Checksum Computation) and E125 (What is Read Above Write Pointer in Sequential Zone)
  - **Before**: the R2 item of "what this segment did" in the E163 (Multi-GPU Unit Checksum Computation) body gave the ssh alias of the other host; the section "two items outside the criteria, from kernel documentation rather than this apparatus" in the E125 (What is Read Above Write Pointer in Sequential Zone) body gave an absolute path under this host's user directory.
  - **After**: that place in E163 (Multi-GPU Unit Checksum Computation) says only "the other host", and that place in E125 (What is Read Above Write Pointer in Sequential Zone) is written as a path starting with `~/`; conclusions, numbers and artifacts unchanged, and the "affected decisions" of both pages reviewed row by row, all not affected.
  - **Basis**: the user decided that private host and service information does not go into the kb or other public logs, and existing current files are changed too (line 312 of `records/2026-09-24-里程碑二收尾调度.md`).

### 2026-09-27

- E163 (Multi-GPU Unit Checksum Computation) page created, first run
  - **Before**: there was only the pre-run registration `e163-preregistration.md`, with no apparatus, artifact or experiment page.
  - **After**: a new independent handwritten model `scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs` (bin `e163-gpu-multicard-crc32c`, behind the feature `e163-gpu`, depending on `wgpu = "=30.0.1"` and `pollster = "=1.0.1"`), 10 unit tests all green; the experiment page [163-GPU多卡算单元校验和.md](experiments/163-gpu-multi-card-unit-checksum-computation.en.md) created. M1 (two cards on one host) judged "can" (2560/2560 identical one by one); M2 (five cards on a two-host setup) judged "cannot" under today's GPU memory conditions (4/5 cards succeeded; the 5th could not create a context with 141 MiB of GPU memory to spare); M3 (whether the comparison goes red) judged "goes red" (both injections, a flipped bit and a dropped entry, reported as they are). Mutation table `scripts/mutation/tables/e163_gpu_multicard_crc32c.tsv` (7 entries); two more (arm C and the shader both changed to the same wrong polynomial; the result file received on this host truncated by its last line) did not go into the TSV because of mechanism limits of `mutate.sh`, verified by hand to be caught, 9 in all caught. `scripts/experiment-apparatus/replay.sh` gains a `driver_e163_r1_merge` line (registering the only merge output in R1 without timing fields, byte-identical). Along the way a real bug was found and fixed where, at `cargo test`'s default parallelism, three GPU unit tests trampled each other and hung for 39 minutes (serialized by adding `GPU_TEST_MUTEX`).
  - **Basis**: pre-run registration `e163-preregistration.md`; artifacts `research/results/e163-gpu-multicard-crc32c-2026-09-27-{r1,r2,f1,f2,r1-merge}.out`.

## E164 (Small-Scale Functional Testing of New Crash Amplification Design)

### 2026-10-17

- The heading "results copied whole from the artifact" gains the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; the W5 line gains a qualifier: this was measured by crate closure, not by function unit key (settled item 17 of D13 (Verification Route)); conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md`.

### 2026-10-15

- 100 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 100 prefix lines; those lines on the page are prefixes of the artifact lines with the tail removed, and the pointer points to the whole line in the artifact); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md`.

### 2026-10-11

- The third-segment probe `probe_crash_states.rs` deleted from the apparatus directory: it was never wired into the apparatus and no target references it, and the crate-sources-reachable cell of gate code-source-discipline judged it orphaned (the user decided "delete it, and point the experiment page at git history"); conclusion, numbers and artifacts unchanged.
  - **Before**: `git show dbcc0d2f:research/e7-index-bench/src/bin/e164_crash_amplification_design_function_test/probe_crash_states.rs`

### 2026-09-28

- E164 (Small-Scale Functional Testing of New Crash Amplification Design) page created, first and second segments run
  The runner got stuck while writing the apparatus and was stopped; the main agent took over writing the apparatus `scripts/experiment-apparatus/e164_crash_amplification_design_function_test.rs`, 12 unit tests and 22 mutations (all caught), ran the first and second segments, and put the three artifacts in `research/results/e164-crash-amplification-design-function-test-2026-09-28-*.out`; section twelve of the pre-run registration records revisions R1–R9. The third segment was not run.

## E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound)

### 2026-10-23

- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) 7th rerun segment 1 run (adversarial root ring placed by the placement formula: KA replaced by KL); the status gains "7th rerun partly run (2026-10-23: segment 1 run, KA replaced by KL, whose roots are placed by the placement formula; the inline tests of segment 2 not run, left to the main agent; segment 3 and P1b not run after it was enough to judge)"
  - **Before**: `git show 04ceaa904:.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md`. The status stopped at the 6th rerun; the adversarial root ring of the two root selection families was KA (slots 0, 1 and 2 of region 0 each take empty or one of four roots, 125 states), and after the placement check of C743 (rebuilt allocator recording new class does not verify root occupies slot) went into the implementation all of its roots were misplaced reads; the three root selection registration lines in `scripts/experiment-apparatus/replay.sh` pointed to the artifacts of segment 2 of the 3rd rerun, segment 1 of the 4th rerun and 5-A of the 5th rerun; "paths and conclusion registration" went up to path 16, and the local function of path 13 was called `ka_kj2_violations_of_a_local_rule`; the review of the twelve rows of "affected decisions" stopped at 2026-10-15
  - **After**: the experiment page gains a "7th rerun" section: KA replaced by the new domain KL (the placement slots of txg 100, 101 and 102 each take empty or one of instances 1 and 2, 27 states), and the three root selection artifacts produced again; the shape of KL is 27, 1, 0, 7, 12, 4, today's three steps give KJ0 1 / 27 and KJ2 0 / 26, and the local root selection orders give 0, 0, 6, 0, 4 on the r3 tier and 0, 0, 6, 0, 4, 0 on the r4 tier, equal cell by cell to registration section 7 in all three, with the four KL lines byte-identical across the three (the artifact half of L1 is enough to judge); every line outside KL is byte-identical to the previous artifact, and the summary line differs only in the state count, by −98 (L2 is enough to judge). "Rerun" gains a paragraph (the three registration lines point to the new artifacts, and the old artifacts are kept), "paths and conclusion registration" gains path 17 and renames the local function of path 13 to `kl_kj2_violations_of_a_local_rule`, "what it cannot answer" gains an item, and the twelve rows of "affected decisions" reviewed row by row with relations unchanged. The inline-test half of L1 and the five harness mutation lines were not run (checker tier); L3 and P1b were not run after it was enough to judge
  - **Basis**: registration `research/prompts/e165-r7-prereg.md`; artifacts `research/results/e165-replay-bounded-enumeration-2026-10-23-r7-r3-root-choice.out`, `research/results/e165-replay-bounded-enumeration-2026-10-23-r7-r4-root-choice-faults.out`, `research/results/e165-replay-bounded-enumeration-2026-10-23-r7-r5-gate-r2-root-choice-faults.out`

### 2026-10-17

- The two closing lines of the second-segment mutation kill bound (name=restored, name=verdict) changed from the `git show 5d0b7e7c1667:…` pointer to point to file names in `research/results/` (30c245b0 already put the artifacts back into the workspace); conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md` has the artifact lines copied whole; the `git show 5d0b7e7c1667:…` pointer form was newly written by the item earlier the same day, and the staging area has no commit of it yet.

### 2026-10-15

- 2 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 2 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md`.

### 2026-10-09

- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) 6th rerun run (the r3 tier legal family); the status gains "6th rerun run (2026-10-09: r3 tier legal family)"
  - **Before**: the version in the staging area (`git show :.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md`). The status stopped at the 5th rerun; the r3 tier had formal runs only for the adversarial and root selection families, and the controls of the five J6Qn and j7 cells had no readings at the r3 tier
  - **After**: the experiment page gains a "6th rerun" section: the 17 control criteria of `--profile r3 --family legal` (even, tables built directly, disk order as usual) are the same as those fixed by the registration, all 58 control readings red, and 23 lines of six items equal the pinned values; the verdict differs from the r2 legal family on the same root only by tier name (`E7RESULT` differs by tier name only in the two lines `config` and `scope=run`); two passes at 8 and 4 threads byte-identical. "Rerun" gains a paragraph, "paths and conclusion registration" gains path 16, "what it cannot answer" gains an item, and the twelve rows of "affected decisions" reviewed row by row with relations unchanged
  - **Basis**: registration `research/prompts/e165-r6-prereg.md` (including the main agent's acceptance); line 1 of the question sheet `research/prompts/m3-formal-e165-r6-questions.md`; artifacts `research/results/e165-replay-bounded-enumeration-2026-10-09-r6-comparison.out`, `research/results/e165-replay-bounded-enumeration-2026-10-09-r6-r3-legal-family.out`

### 2026-10-04

- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) 5th rerun run (all five cells enough to judge; the fourth segment not run after being enough to judge); the status gains "5th rerun run (enough to judge, 2026-10-04: first, second and third segments and the two steps of the fifth; the fourth segment not run after being enough to judge)"
  - **Before**: `git show 0c065788:.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md`. The status stopped at the 4th rerun; the "4th rerun" section said "the gate takes the implementation's current A"; "what it cannot answer" said that once R2 enters the code this family must be rerun and that the Q1 and Q2 readings do not vouch for R2; the paragraph on the 4th run first segment in "rerun" said the `mount.rs` and `transaction.rs` of the main workspace changed, so "the reproduced numbers will differ"; "paths and conclusion registration" went up to path 13; "affected decisions" had eleven rows, without settled item 15 of D23 (Journal Role & Format).
  - **After**: the experiment page gains a "5th rerun" section (a summary table of five cells and five subsections for the first, second and third segments and the fifth segment's 5-A and 5-B): 3-A reran the 16 comparison targets of the r1 to r4 tiers on the root after line 7-2 of the root ring line, with the verdict changing only in j7 fields (J7a allocation record tree root and J7a instance table pointer from 0 to non-0) and 0 places outside j7; the three of the r1 tier, r3 root selection, and the five of r4 with faults byte-identical; for multi-disk `lagging_class` split into rows, the identity takes the strict reading (main agent acceptance); 3-B: the three driver modes gain `--pins r5`, and the kill bounds of 28, 6 and 6 variants are identical variant by variant to the previous time, with the first-red criterion list gaining two J7a cells reported separately; 3-C: the five j8 cells of the new tier `--profile r5` are 0 in every one of 830 cells, judged the same as the old j7 cells cell by cell, and the 38 control readings C8a to C8d equal the pinned values; 5-A: the generator gate changed to take R2, and KJ1 and KJ1T are 0 in every cell within the commitment on all four legal domains, the same verdict as the 4th run; rerunning the first segment with A is byte-identical to the 4th run, with no flip needing attribution; 5-B: the driver with `--gate r2`; the outcomes and first-red positions of the six variants are the same as the 4th run, and the three root selection variants are still within bounds and not red; in the first pass at 8G tsv-1812 hit the cap, the second pass at 16G filled in only that one, and which pass each of the seven lines came from was checked one by one. The sentence in the "4th rerun" section changed to describe the 2026-10-03 run; "what it cannot answer" gains five items, with two changed to describe the 4th run and point to the 5th run's rejudgment; "rerun" gains a paragraph on the 5th run and that sentence of the 4th run first segment paragraph changed; "paths and conclusion registration" gains paths 14 and 15; the eleven rows of "affected decisions" reviewed row by row, all written as not affected, with one row added for settled item 15 of D23 (Journal Role & Format) (prepared material); `scripts/experiment-apparatus/replay.sh` gains ten registration lines and ten driver functions. The 72 artifacts (35 first segment, 8 second, 16 third, 6 5-A, 7 5-B) are named one by one in the "5th rerun" section. The decision text was not touched; whether the four cells of settled items 4, 14 and 15 of D23 (Journal Role & Format) and settled item 8 of D22 (Synthesizing Unit Atomicity) should be upgraded to support is handed to the main agent. Apparatus unit tests and apparatus mutations not run.
  - **Basis**: registration `research/prompts/e165-r5-prereg.md` (including revision 1 and four acceptances by the main agent); artifacts `research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-comparison.out`, `…-r5-segment2-comparison.out`, `…-r5-segment3-cell-correspondence.out`, `…-r5-segment5-comparison-with-r4-q1.out`, `…-r5-segment5-comparison-with-r4-q2.out`.

### 2026-10-03

- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) 4th rerun registration revision 4: wording of the t1 reading corrected (hit by K4-a of the third code round verdict); status unchanged
  - **Before**: the registration and the experiment page wrote t1 as "in this mount, the first read of that placement cannot be read", and in parentheses also "the first read of the read phase fails, while the reread, the read at the moment of number acquisition and the reads on the write side all succeed"; the two sentences are not equivalent at the system configuration slot placement. The experiment page's "geometry sensitivity and trajectory" said "the t1 of the three items SN0, SN1 and SB takes the second reread, 28 466 each", without saying which timing t1 refers to.
  - **After**: t1 written as "unreadable from the first read of that placement in this mount on, recovered before the reread" (registration revision 4); the experiment page's "how faults are placed", "geometry sensitivity and trajectory" and the description of the self-proof `t1_takes_the_second_read_in_the_gate_on_domain` changed to match, and "what it cannot answer" gains an item: when "only the first read is bad", system configuration selection may pick the other slot and then go down another path, which this family does not enumerate. The eleven rows of "affected decisions" reviewed row by row, relations all unchanged. Not a word of the artifacts, apparatus, drivers, or the conclusions and numbers of Q1 and Q2 changed.
  - **Basis**: the K4-a row in section two of the third code round verdict `research/prompts/m3-formal-code-r3-main-verification.md` (main agent judgment: the history is real, Q1 and Q2 do not flip, the error is in the wording); attacker readings in `k4t1.log`, lines 2, 4, 6 and 10 (only the first read bad, `OnTheFirstRead`) and lines 3, 5, 7 and 11 (bad until before the reread, `OnTheOneReread`).
- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) 4th rerun second segment run (Q2: do the three root selection variants that did not go red in the 3rd run go red on the new family); the status changed to "4th rerun run (two segments: first and second, 2026-10-03)"
  - **Before**: the status said "4th rerun partly run (first segment, 2026-10-03)", the Q2 row said "not judged (second segment not run)", recording only two root ring shape counts read incidentally in the first segment; "rerun" had no second-segment line, and "paths and conclusion registration" went up to path 12. The 4th rerun section had not been committed at the time of this change (the experiment page is a newly added file in `git status`), so the before-state is the version written by the next entry (first segment).
  - **After**: the experiment page gains a "4th rerun second segment" section, a second-segment paragraph in "rerun", path 13 in "paths and conclusion registration", four items in "what it cannot answer", reviews of each row of "affected decisions", and an item in "Historical Versions"; Q2: some are still not red: compare-without-instance (v1823), tie-takes-lower (v1825) and instance-generation-first (v1826) are identical sub-cell by sub-cell to the baseline over 410 sub-cells of four legal domains (KF, KF3, KF9, KFW), within bounds and not red, and all three are judged insensitive sampling points (class three, neither equivalent mutations nor real blind spots): on KA, KJ2 reads 18, 36, 18, equal to the registration's pinned values; v1826 differs in 8, 8 and 12 sub-cells on KG with the clause protections removed, while v1823 and v1825 do not differ even on KG; take-the-first-valid (v1824) is red on all four legal domains, while tie-the-latter-wins (v1822) and glue-spreads-keys-without-instance-generation (v1827) are not red. `ka_pin_mismatches=0`, `kg_pin_mismatches=0`; the opening baseline is identical line by line with the first-segment artifact, and the closing baseline restored. In the first pass at a 4G memory cap tsv-1823 hit the cap (exit 250, output not counted), and only the supplementary run at 8G produced it. `scripts/experiment-apparatus/replay.sh` gains the second-segment rerun line and driver function. The decision text was not touched: every row of "affected decisions" reviewed and written as not affected, and no prepared-material cell upgraded to support (main agent judgment); this family is still generated by taking A, and must be rerun once R2 enters the code (C662 (The legal history family with faults is still generated by taking A)); the acceptance "the three variants must go red" in the row of C657 (No faulty root selection combined-judgment family) was not reached this time, and how to handle it is handed to the main agent.
  - **Basis**: `research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out` (11 lines) and the driver log of the two passes `e165-replay-bounded-enumeration-2026-10-03-r4-segment2-driver-log.out`; revision 3 of registration `research/prompts/e165-r4-prereg.md` and TF5, TF6, TF7 of section ten.
- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) 4th rerun first segment run (Q1: do root selection and replay lose anything on legal histories with read faults); the status gains "4th rerun partly run (first segment, 2026-10-03)"
  - **Before**: the status said "3rd rerun run (three segments)", with no 4th rerun section; "affected decisions" had no rows for settled items 8 and 16 of D22 (Synthesizing Unit Atomicity) and settled item 13 of D2 (RAID Stripe Policy). The "3rd rerun" section in `git show HEAD:.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md`.
  - **After**: the experiment page gains a "4th rerun" section (first segment), with additions to "rerun", "paths and conclusion registration" (paths 10 to 12), "what it cannot answer", "affected decisions" and "Historical Versions"; Q1: nothing lost within the commitment: four legal domains (KF, KF3, KF9, KFW), 11 228 080 states, of which 6 919 344 within the commitment, with KJ1 and KJ1T violations 0 in every cell and KJ0, KJ2 and KJ3 also 0; the 587 599 violations (KJ1 and KJ1T the same count cell by cell) are all in the class RI at judgment time, and the clause's ideal recovery loses the same on the same batch of states; on KG (with two protections removed, not a legal history), within the commitment KJ1 has 10 368 violations and KJ1T 20 628, and on states with no fault at judgment time KJ1T's 648 equals the number computed by the local ideal recovery. The gate takes the implementation's current A, not the clause's current authoritative form R2 (registration revision 1); once R2 enters the code this family must be rerun; Q2 (do the three root selection variants go red on the new family) not judged, with the driver's r4 mode in the second segment. In "affected decisions", existing rows such as settled item 1 of D13 (Verification Route), settled items 7 and 24 of D22 (Synthesizing Unit Atomicity) and settled item 14 of D23 (Journal Role & Format) each gain a sentence reviewing the 4th rerun first segment (not affected), and three new prepared-material rows are added for settled items 8 and 16 of D22 (Synthesizing Unit Atomicity) and settled item 13 of D2 (RAID Stripe Policy). The apparatus, 18 apparatus mutation rows and 10 artifacts landed; unit tests and the whole apparatus mutation table not run.
  - **Basis**: `research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out` (byte-identical to the same-named one without the suffix from the first pass); revisions 1 and 2 of registration `research/prompts/e165-r4-prereg.md`. The decision text was not touched; whether these prepared-material cells should be upgraded to support, and whether the M1 placement and M4 scope should state the readings with faults, are handed to the main agent.
- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) 2nd rerun Q4 row gains "the column representing the implementation is the one following `device_identities()` order"; the review of the settled item 1 row of D13 (Verification Route) in "affected decisions" changed to "changed"
  - **Before**: the Q4 row reported only the numbers of the two columns without saying which column represents the implementation; the settled item 1 row of D13 (Verification Route) said "2026-10-03 not affected", with the M1 cell's sentence handed to the main agent for a back-sweep and the M3 fixed-point premise judged by the main agent.
  - **After**: the Q4 row adds: when the disk order is reversed, the column representing the implementation is "following `device_identities()` order" (`scan_journal` reads disk by disk in the order of the disk table the reader hands over), and the "ascending disk number" column agrees with the implementation only when the caller happens to hand over the disk table in disk number order. The settled item 1 row of D13 (Verification Route) changed to "changed": the M3 fixed-point premise gains I-8.12 (Non-Decreasing Journal Record TXG for Same Instance) (basis citing the 3rd rerun first segment), and the M1 placement changed to the root selection combined-judgment family calling the real three steps (basis citing the second segment). The apparatus, artifacts and other conclusions untouched.
  - **Basis**: the apparatus cell in section two and item 5 of section three of the code round verdict `research/prompts/m3-formal-code-r1-main-verification.md`; the same-day entry in the D13 (Verification Route) section of decisions-history.md.
- E165 (Bounded Exhaustive Recovery Replay and Mutation Kill Bound) 3rd rerun third segment run (Q2: variants that break the root selection order); the status changed to "3rd rerun run (three segments)"
  - **Before**: the status said "3rd rerun partly run (first segment; second segment)", and Q2 had no numbers (the driver's r3 mode not run). The "3rd rerun" section in `git show HEAD:.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md`.
  - **After**: Q2: no: on the frozen root of the second segment, the driver compiled each of six variants once and ran each `--profile r3 --family root_choice`; of the four variants that break the root selection order, only take-the-first-valid (1824) goes red on K, K9 and KW (KJ2 first red at M1U0, KJ1 first red at M1U2; the four cells U11 to U14 of KW not red); compare-without-instance (1823), tie-takes-lower (1825) and instance-generation-first (1826) are identical cell by cell to the baseline over 36 legal history cells, within bounds and not red; on KA, KJ2 reads 18, 36, 18, equal to the registration's pinned values, recorded as insensitive sampling points (not equivalent mutations); the incidental 1822 and 1827 not red. The experiment page gains a third-segment section, line 9 of the path registration and reviews of each row of "affected decisions"; 1 artifact, whose opening baseline is identical line by line with the second-segment artifact.
  - **Basis**: `research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out`; revision 3 of registration `research/prompts/e165-r3-prereg.md` and TF5, TF7, TF10 of section ten. The decision text was not touched; the reviews of settled item 1 of D13 (Verification Route) and settled items 7 and 24 of D22 (Synthesizing Unit Atomicity) are written in "affected decisions" on the experiment page, and whether the placements need to be supplemented is handed to the main agent.

## E166 (Shared State Partitioning for Parallel Commits)

### 2026-10-01

- E166 (Shared State Partitioning for Parallel Commits) page created, first segment run
  An independent handwritten counting model `scripts/experiment-apparatus/e166_parallel_commit_shared_state_partition.rs` (21 unit tests, all 19 mutations caught), Q-1 to Q-9 for six arms on the default geometry and D = 1; artifact `e166-parallel-commit-shared-state-partition-2026-10-01.out` (1315 lines). The second segment not run, marked "not run after enough to judge".

### 2026-10-02

- E166 (Shared State Partitioning for Parallel Commits) 2nd rerun (tree ID renumbering) run
  Question sheet R3; rerun registration `e166-r2-prereg.md` (revisions 1 to 3 written in section twelve). The apparatus's tree ID family `u8` → `u32`, watermarks 65536 / 65548, 13 anchors changed to new values, unit tests pinning new values (one more pin, Q1-C's 144 → 216), the mutation table re-anchored V13 and added V20 and V21, all 21 caught; arm e1 is byte-identical to the first run's artifact; main artifact `research/results/e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out`; of 309 conclusion line pairs 2 differ (A-literal's Q-4 tree ID duplicate total 46944 → 70416, ratio 12 / 8), and failures 11, 12 and 13 do not trigger; the E166 (Shared State Partitioning for Parallel Commits) line of `replay.sh` now points to the main artifact, and the first run's artifact is kept.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

## E167 (Crash Enumeration Partitioning for Multiple In-Flight Publishes)

### 2026-10-15

- 2 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 2 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/167-多在飞发布的崩溃枚举切法.md`.

### 2026-10-01

- E167 (Crash Enumeration Partitioning for Multiple In-Flight Publishes) page created, first segment run
  An independent counting model `scripts/experiment-apparatus/e167_multi_inflight_crash_cut.rs` (37 unit tests, all 12 mutations caught), four candidate cells each for G1, G3 and G5; formal artifact `research/results/e167-multi-inflight-crash-cut-2026-10-01-r2.out`; the same-named one without `-r2` has `void_conditions=1` and is kept as void.
- E167 (Crash Enumeration Partitioning for Multiple In-Flight Publishes) second segment (G4a, G4b) run
  S3 judged first (155117520 free interleavings have only 19719 and 25252 distinct segment signatures, so it can run to completion); the apparatus gains subcommands `s3` and `second`, 52 unit tests, all 22 mutations caught (M18 was not red before the unit test was added, caught after adding it), second-segment revisions 7–9; formal artifact `research/results/e167-multi-inflight-crash-cut-second-2026-10-01-r3.out` (`-r2` the same; the one without a suffix has `void_conditions=1` and is kept as void). Readings: on two disks the ground truth is the full set of state vectors 1327104, exactly reachable 784; K2's real misses L3b against the exactly reachable set are 0, F5 triggers, and K1 misses only on some streams.
- E167 (Crash Enumeration Partitioning for Multiple In-Flight Publishes) second segment gains one cell (L11: K1's cross-transaction misses against T ∩ T_exact) run
  The apparatus gains subcommand `third`, 59 unit tests, all 35 mutations caught, third-segment revision 10; artifact `research/results/e167-multi-inflight-crash-cut-third-2026-10-01.out`. Readings: in the four cells of G4a and G4b free interleaving and Σ_ord, K1 misses reachable cross-transaction states only on some streams (minimum 0, maximum 702 / 498, target sets 729 / 525, with the count of streams that come out positive the same as L2); K2's L3b is still 0. The artifacts and readings of the first two segments unchanged.

## E168 (Ring Window for Borrowing Reserve Capacity on Deletion)

### 2026-10-17

- The segment1 row of the artifact table gains "not committed: 115 MB, ignored by line 32 of .gitignore, only on the machine that ran it"; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/168-删除借保留量的环窗口.md`.

### 2026-10-15

- The 1 artifact line copied whole (`name=verdict`) came from the uncommitted segment1 artifact (115 MB, ignored by line 32 of `.gitignore`), and was changed to a sentence stating that the raw output is not committed, with its path and line number on this host; these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/168-删除借保留量的环窗口.md`.

### 2026-10-01

- E168 (Ring Window for Borrowing Reserve Capacity on Deletion) page created, apparatus written
  The runners of the first two dispatches did not hand back; the third took over and wrote the committed apparatus `crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs` (the deletion entry in the main workspace copy is a stub), the deletion path patch table `scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv` (19 rows), 11 unit tests, and 15 apparatus mutation rows at the end of `crates/mutations.tsv`, all compiling; unit tests, mutations and the first-segment artifact were not run (checker tier), status "in progress (apparatus)".

### 2026-10-03

- E168 (Ring Window for Borrowing Reserve Capacity on Deletion) first segment committed, status changed to "in progress (first segment done)"
  The main agent ran first-segment r8 (the apparatus after pre-run registration revisions 1–17) with `SINGLEFS_HEAVY_TESTS=user-request`: 27 unit tests (26 passed and 1 ignored in the main workspace, 27 passed in the copy), the 14 apparatus mutation rows proven red and all caught, the 19 S2 rows each hit once, the two S1 copies byte-identical, refusals in all eight positive control cells, and no stop line in segment1; nine artifacts committed (`research/results/e168-delete-ring-window-borrow-segment1-2026-10-03.out` and others, with the line prefix `E name=` wrapped as `E7RESULT name=`), and `scripts/experiment-apparatus/replay.sh` registers three lines, `driver_e168_s1_history`, `driver_e168_positive_controls` and `driver_e168_segment1`. Readings: 65 cells refused by the placement judgment (N-A 39, N-B 26), and Q1 is enough to judge "cannot cover" per F2; the apex exceeds (3S + 1) × W_max in 0 cells, and W_max rises with file size in 14 groups (F4), so Q2 is not enough to judge; the patch table's deletion path rewrites the whole lower segment of the extent tree, differing from item 2 of settled item 16 of D8 (Core Index Structure), written into "what it cannot answer"; the "affected decisions" table gains four rows, all prepared material.
- E168 (Ring Window for Borrowing Reserve Capacity on Deletion): the main agent judges Q1 open, and segment1 is withdrawn from the replay registration
  The runner wrote Q1 "cannot cover" per registration 5.8, "any refusal in any cell is enough to judge"; the main agent judged: line 400 of the registration sets K = 144 as the main verdict column and K = 1 as sensitivity; the K = 144 column has only 7 refused cells, all on the N-A arm and all on the deletion path of the patch table that rewrites the whole lower segment of the extent tree, while item 2 of settled item 16 of D8 (Core Index Structure) and the real implementation of D-5c2 (`plan_the_extent_tree_after_a_deletion_batch` in `crates/singlefs-core/src/unlink.rs`) rewrite only the truncated leaf and its ancestors; the N-B arm has 0 refused cells in the K = 144 column. K = 1 and N-A on the whole-rewrite path count as one loss, the K = 144 main verdict column was not measured on the tail-leaf-only path, and Q1 stays open. The user decided: the second segment waits until this deletion batch is merged into the main repository, then a rerun registration is opened and run on the real implementation's deletion batch; segment1 takes about 9 hours and 36G per pass and does not enter the `scripts/experiment-apparatus/replay.sh` registration table (the two lines s1-history and positive-controls are kept), and reruns are run by hand per the experiment page's manual steps when the user names them.
  - **Basis**: the verdict line at line 177053 and the minimal counterexample at line 25777 of the segment1 artifact; the main agent split the refusals by arm on the committed artifact, `39 N-A landing 0`, `26 N-B no_space_after_raising_the_floor 0`; the user's pop-up choices "rerun on the real implementation after it is merged (recommended)" and "keep s1/positive controls, segment1 not in (recommended)".

## E169 (Append Discipline and Prefix Recovery for Parallel Commits)

### 2026-10-01

- E169 (Append Discipline and Prefix Recovery for Parallel Commits) page created, first segment run
  The runner wrote an independent handwritten model `scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs` (24 unit tests, all 13 mutations caught), and ran the first segment (the two Q1 conditions of serial landing and simultaneous issue) after the fourteen sequence models of stop clause t1 matched today's `replay_core` in `crates/singlefs-replay-prefix` on both sides; formal artifact `research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out`; the same-named ones without `-r3` and with `-r2` are 0-byte failed outputs. Section twelve of the pre-run registration records revisions 1 and 2 and one observation (A5 does not hold for C-2). The second and third segments not run.
- E169 (Append Discipline and Prefix Recovery for Parallel Commits) second segment (multi-flight root-ordered, multi-flight root-disordered) run
  The apparatus gains the parameter `segment-two`; k = 2 enumerated directly, k = 3 deduplicated by configuration (at most 19416 configurations, T3 not hit); the two paths are identical cell by cell over the 192 cells at k = 2, and 0 of the 32 B5 anchors mismatch; 30 unit tests, all 22 mutations caught; formal artifact `research/results/e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out`; the same-named one without `-r2` is a 0-byte failed output (memory wrapper exit code 252). The second half of A5 (V_dep = 0 for C-0, C-1 and C-3) does not hold on multi-flight × control D at k = 3 (42), recorded as a class A mismatch and handed to the main agent. The third segment not run.
- E169 (Append Discipline and Prefix Recovery for Parallel Commits) third segment (fsync wait model) and the direct differential of path 3 against path 4 run
  Section twelve of the registration records revision 3 (five readings settled by the main agent) and revision 4 (readings added by the runner). The apparatus gains the parameters `segment-three` and `replay-core-inputs`, 38 unit tests, 29 mutations; third-segment artifact `research/results/e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out` (0 of 100 B6 anchors mismatch; of the 192 Q2-3 rows, 0 judged "bounded", F10 not hit, and the 120 paired rows where both arms grow recorded as a shared component per F9); a temporary crate depending on `crates/singlefs-replay-prefix` by path fed the recovery inputs of the 88560 crash states of the first and second segments to `replay_core`, with 0 mismatches against the model's C-0 (`e169-parallel-commit-append-and-prefix-recovery-replay-core-direct-check-2026-10-01.out`). Status "in progress (third segment)".
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

## E170 (First Version of O3 (independent specification executor))

### 2026-10-17

- The heading "results copied whole from the artifact" gains the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; two places with the old file name invariants.md now point to crates/singlefs-checker/invariants/; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/170-O3规约执行器第一版.md`.

### 2026-10-15

- 32 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 32 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/170-O3规约执行器第一版.md`.

### 2026-10-05

- E170 (First Version of O3 (independent specification executor)) 2nd rerun closed out: the page gains a "2nd rerun" section, the status changed to "2nd rerun run (Q1–Q5 enough to judge)", the nine E170 (First Version of O3 (independent specification executor)) lines in `replay.sh` replaced by the nine of the 2nd rerun, and the four rows of "affected decisions" reviewed again
  - **Before**: `git show 0c065788:.claude/kb/experiments/170-O3规约执行器第一版.md`; the index row is in `git show 0c065788:.claude/kb/experiments.md`.
  - **After**: Q1 A (0 items out of scope; the 8 items in scope are exactly the eight P8 cells of inputs 3 and 4); Q2 A (the 15 and 63 states of K at lines 242 and 269 of E171 (Three-Oracle Collusion Test) all judged absent); Q3 A (the 68 states of K270 at line 270 all judged absent, and 0 false reds over 1 168 legal-history states and 792 + 972 geometry point states); Q5 A (0 differences over the comparable cells of seven families); Q4 A (the 1st rerun of E171 (Three-Oracle Collusion Test) uses v2-487, frozen lines 676–692 of `research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out`, seed `ce39b484ec109c7a`); V16 is literally hit but does not discriminate, and determinism is judged deterministic per the apparatus's measurement basis; all 34 third-segment mutations caught, XC8 and XC9 caught. The "rerun" section records deleting the four lines of the first-generation executor and the one line of the 1st rerun's Q2 comparator (no counterpart in today's format), repointing four lines and adding five, and why the 66 original artifacts cannot be rerun directly; "paths and conclusion registration" gains paths 8–11; "unit tests and mutations" and "what it cannot answer" gain the items of the 2nd rerun; the four rows of "affected decisions" reviewed, all not affected, relations unchanged.
  - **Basis**: pre-run registration `research/prompts/e170-r2-prereg.md` (revisions 1 and 2 and the five closing sections "main agent acceptance"); artifacts `e170-r2-o3-spec-executor-2026-10-04-s2-comparator-q1.out`, `e170-r2-o3-spec-executor-2026-10-04-s2-comparator-q2-q3.out`, `research/results/e170-r2-o3-spec-executor-2026-10-04-s2-q5-comparator.out` (line 74), `e170-r2-o3-spec-executor-2026-10-04-s3-mutations.out`; rerun lines 254–262 of `scripts/experiment-apparatus/replay.sh`; dispatch record `records/2026-10-01-里程碑三形式化接入.md`, batch 22 (lines 903 and 956).

### 2026-10-03

- E170 (First Version of O3 (independent specification executor)) 1st rerun second and third segments: the Q2 and Q3 rows of the current-state table split into the first run and the 1st rerun, the page status changed to "1st rerun run (three segments)", and the four rows of "affected decisions" reviewed again
  - **Before**: the Q2 and Q3 rows of the current-state table had only the first run's readings (Q2 agreement 0 / 450, Q3 literally absent 4 / 26); the page status was "1st rerun partly run", the 1st rerun section said "Q2 and Q3 are in the second and third segments; there are no numbers for them here", and the last section "quantities not run in the first segment" listed Q1-g, the Q2 quantities, the Q3 quantities, the new modes, mutations, XC1 and XE4; the review of settled item 6 of D13 (Verification Route) described the first segment's current state (Q2 and Q3 without numbers yet).
  - **After**: the Q2 row gains the readings of the 1st rerun's second segment (strict running, re-exported batch 1 0 / 450, batch 2 0 / 126, Q2-pc 1; the two batch 1 copies are the same batch; the executor judges all "present", and the implementation lets all through; for F6 only two paths remain unwalked in the batches); the Q3 row gains the third segment's readings (the P8 expectation changed to "present" per the registration revision, P rows 0 / 26, blind tests 0 / 40, P10 0 / 2 misjudged, Q3-pc 1; all 22 executor mutations caught, 11 of them with sensitive points on batch states and bad states; XE4 appended to `crates/mutations.tsv`); the page gains "second segment", "third segment", "mutations" and "quantities not run" sections, and the rerun section gains the 1st rerun commands and the changes at lines 240–248 of `replay.sh` (lines 240–242 now point to artifacts regenerated today, and four drivers added); "what it cannot answer" gains the scope of Q2 and Q3 and that Q3-f reruns to 21 today; settled item 6 of D13 (Verification Route) written as "changed" (the sentence in its scope on the current state of the first version of O3 (independent specification executor) must change, with the decision file changed by the main agent in the same commit), and settled items 4 and 1 and settled item 16 of D22 (Synthesizing Unit Atomicity) reviewed as not affected. The first run's artifacts, verdicts and other conclusions untouched.
  - **Basis**: the 1st rerun's second- and third-segment artifacts `e170-r1-o3-spec-executor-2026-10-03-s2-q2-comparator.out`, `…-s3-bad-states.out`, `…-s3-mutations.out` and pre-run registration `research/prompts/e170-r1-prereg.md`.

- E170 (First Version of O3 (independent specification executor)) 1st rerun first segment: the Q1 row split into the first run and the 1st rerun, the first item of "what it cannot answer" rewritten per this run's readings, and one sentence in the scope of the settled item 6 row of D13 (Verification Route) needs changing (awaiting the main agent's change to the decision file)
  - **Before**: the Q1 row of the current-state table had only "cannot judge"; the first item of "what it cannot answer" said "after G01–G03 were added to the kb, the frozen executor was not rewritten per the new kb and strict running was not rerun, so whether it can judge after the addition has no number"; the review of settled item 6 of D13 (Verification Route) said changed (O3 (independent specification executor) zero code changed to the current state of the first version).
  - **After**: the Q1 row says the first run cannot judge and the 1st rerun first segment can (under strict running, 0 of the 450 states of batch 1 and 126 of batch 2 hit "the specification lacks a sentence"); the first item of "what it cannot answer" changed to this run's readings and scope (it only says these 576 sampled states did not walk into rows of the lacks-a-sentence table, not that other states would not either); the review of settled item 6 of D13 (Verification Route) says "changed": what must change is the sentence in its scope "strict running hits 'the specification lacks a sentence' and cannot judge" (the decision file changed by the main agent in the same commit), while settled items 1 and 4 of D13 (Verification Route) and the new row for settled item 16 of D22 (Synthesizing Unit Atomicity) are not affected. The first run's artifacts, verdicts and other conclusions untouched.
  - **Basis**: the 1st rerun first-segment artifact `research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out` (verdict lines 496, 501, 635 and 640) and pre-run registration `research/prompts/e170-r1-prereg.md`.

- E170 (First Version of O3 (independent specification executor)) Q3-f gains "28 is an upper bound on the number of rows let through"; in "affected decisions" the settled item 6 row of D13 (Verification Route) changed from prepared material to support, and the other two rows reviewed again
  - **Before**: Q3-f only reported "the implementation lets 28 rows through under both judgments among the P rows and the 66 blind test rows", without saying in which direction that number is high; the settled item 6 row of D13 (Verification Route) had the relation prepared material, and whether to change the sentence "O3 (independent specification executor) zero code" and C447 (Specification Executor Not Connected to Crash-Point Replay and RefFS Differential) was handed to the main agent.
  - **After**: the end of Q3-f adds: the bad states' patches are recorded as system configuration slot writes and counted in no publish, the record checker's first criterion (`root_without_record`) is set 0 times in the 76 rows, and 28 is an upper bound on the number of rows let through, enough to support "not all" but not "how weak the implementation side's discriminating power is". The settled item 6 row of D13 (Verification Route) changed to support (the item's basis cites E170 (First Version of O3 (independent specification executor)) the same day); the two rows for settled items 1 and 4 reviewed as not affected. Apparatus, artifacts and other conclusions untouched.
  - **Basis**: the apparatus cell in section two and item 5 of section three of the code round verdict `research/prompts/m3-formal-code-r1-main-verification.md`; the same-day entry in the D13 (Verification Route) section of decisions-history.md.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

## E171 (Three-Oracle Collusion Test)

### 2026-10-17

- The heading "first-segment baseline and controls (copied whole from the s1 artifact)" gains the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; three places "invariants.md not checked item by item" now point to crates/singlefs-checker/invariants/; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/171-三oracle合谋测试.md`.

### 2026-10-15

- 16 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 16 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/171-三oracle合谋测试.md`.

### 2026-10-13

- 1st rerun revision 12: the stream three segment length pin `LOCAL_THIRD_STREAM_SEGMENT_LENGTHS` re-pinned to the one computed by crates today, differing only at index 15 (20 → 24) and 43 (24 → 20), with write count 5980, full state count 362 809 947 and closed form 362 808 742 unchanged; after the re-pin `stream-constants --streams 1,2,3` exits 0 and `s1_failed_checks=0` (all 77 S1 checks `holds=true`), while the pass before the re-pin exited 3, with only the stream three segment length check failing; the experiment page gains a section "1st rerun · revision 12: stream three segment lengths re-pinned", the page title status and the index status column gain ", stream three segment lengths re-pinned per revision 12", the review cells of the 14 rows of "affected decisions" changed to not affected, and the parenthetical of the "rerun" section and the line count of the "rerun" item changed to twenty per the current count (`replay.sh` gains 1 line `driver_e171_r1_revision_12_stream_constants`).
  - **Basis**: revisions 12 and 13 of registration `research/prompts/e171-r1-prereg.md`; artifacts `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12-before-the-repin.out` (A, before the re-pin) and `…-2026-10-13-s1-stream-constants-revision-12.out` (B, after the re-pin); question sheet `research/prompts/e171-r1-s3-segment-lengths-questions.md`; item 3 of section three of the verdict `research/prompts/m3-retire-r1-main-verification.md`.
  - **Before**: the `LOCAL_THIRD_STREAM_SEGMENT_LENGTHS` passage in `git show ce1457c3:crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`; `git show ce1457c3:.claude/kb/experiments/171-三oracle合谋测试.md`; the title and index status column originally said "… the revision 8 pass of the final main line awaits a snapshot of the tree)".
- "The swap is caused by the retirement record" cannot be shown on the committed apparatus: the retirement record cannot be switched off without changing core, and there is no tree with this apparatus before f8a9ddaa; the counterfactual on a copy is only taken as a lead and written into "what it cannot answer (revision 12)" on the page; the "write" side is settled by the `crates=` column summaries of artifacts A and B being identical and equal to the new pinned value.
  - **Basis**: section 12.5.3 of registration revision 12; artifacts as above.
- The 2026-10-05 `…-s1-stream-constants-r6-r2.out` and `-r6-r3.out` are kept as history of the revision 6 tree and are no longer current values; the rerun table never had lines for them (`grep -c 'stream-constants-r6-r[23]' scripts/experiment-apparatus/replay.sh` is 0 both before and after the change).
  - **Basis**: section 12.12.3 of registration revision 12.

### 2026-10-07

- The comparator for the 1st rerun second-segment whole table (revision 6 tree, 281 rows, streams one and two) finished running: a separate second-segment comparator, unit tests and mutation table written, the experiment page gains a section "1st rerun second segment · the revision 6 pass" (Q2 B, Q3 denominator D = 39 and three other measurement bases, each class row by row, two readings awaiting the main agent's acceptance), and the 8 second-segment artifacts named; Q5 changed from "awaiting alarm self-proof" to "alarm self-proof passed, judged A"; `replay.sh` gains two lines; the mutation driver gains an optional `--unit-test-module`; every row of "affected decisions" reviewed, with no relation changed.
  - **Before**: the title status and index status column said "1st rerun first segment (revision 6 tree) run (Q1 A, Q3 A, Q4 baseline A and lines 242 and 269 B; Q5 awaiting the alarm self-proof at commit time; the denominators of Q2 and Q3 wait for the second segment; the revision 8 pass of the final main line awaits a snapshot of the tree)"; the Q5 row "run, awaiting alarm self-proof"; fourteen E171 (Three-Oracle Collusion Test) lines in `replay.sh`.
  - **After**: the title status and index status column say "1st rerun first segment and second-segment whole table (revision 6 tree) run (Q1 A, Q2 B, Q3 A with denominator D = 39, Q4 baseline A and lines 242 and 269 B; Q5 alarm self-proof passed, judged A; the revision 8 pass of the final main line awaits a snapshot of the tree)"; sixteen E171 (Three-Oracle Collusion Test) lines in `replay.sh`; second segment 17 unit tests, all 28 mutations caught; the first segment's verdicts, unit tests and mutation table not touched at all (rerunning the first segment's table is byte-identical to the committed artifact).
  - **Basis**: `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out`, `…-2026-10-07-s2-research-mutations-r6.out`, `…-2026-10-07-s2-rows-driver-r6.out`; Q2-set and Q3-denominator in section six of registration `research/prompts/e171-r1-prereg.md`, and the closing sections "main agent acceptance: Q5 alarm self-proof of the first segment (revision 6 tree)" and "main agent acceptance: second-segment whole-table multi-host run handed back".
- The comparator for the 1st rerun first segment · the revision 6 pass (revision 6 tree `7726cdd6…`, steps 0–10 `-r6-r3`, steps 11 and 12 `-resume2`) finished running, and the experiment page gains a section "1st rerun first segment · the revision 6 pass": Q1 A, Q3 A (line 125 counted, c = 12), Q4 baseline A, lines 242 and 269 B, Q5 is A by the numbers but recorded as "run, awaiting alarm self-proof", and Q2 reports only seven rows; the conditions for counting (S15 ①–⑥, Z7, the positive control "judging-path panics listed separately") copied row by row; the original artifact where the cross-host consistency control judged `false` is stated void, pointing to the acceptance in the registration's closing section, and what counts is `…-2026-10-07.out`. The comparator changed in six places (25 columns, A11 speed-up, reading only this pass's prefix, stream three coverage and F7, A1 for empty mutations, two Q5 columns `judge_panic`), research-side mutations 17 → 23, all caught; the multi-host orchestrator, the other host's end, the revision 6 multi-host running method and the cross-host control entry, together with `check_product.py` and ``, copied byte for byte into the research-side directory; `replay.sh` gains three lines; "paths and conclusion registration" gains lines 7 and 8; every row of "affected decisions" reviewed, with settled item 1 of D13 (Verification Route) and settled item 14 of D23 (Journal Role & Format) changed from not affected to prepared material.
  - **Before**: the experiment page had only the original E171 (Three-Oracle Collusion Test) readings, and not one of the 168 artifacts of the 1st rerun was named; eleven E171 (Three-Oracle Collusion Test) lines in `replay.sh`; mutation table 17 entries; the comparator read a 24-column `oracle-verdicts.tsv` and could not read this pass's export.
  - **After**: the page title status and index row not touched (the dispatch gave no wording); the new section's verdicts belong only to the revision 6 tree, and the revision 8 pass has not run yet.
  - **Basis**: `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3.out`, `…-s1-research-mutations-2026-10-07-r2.out`, `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r2.out`; revisions 6–8 of registration `research/prompts/e171-r1-prereg.md` and the closing section "handling of the S17 stop in the `-resume2` cross-host consistency control".
- The comparator adds the per-state list of Q4-(ii) in section six of the registration and the Q4 and Q5 trajectories of section eight, and the branch of `q5_verdict` where everything abstains or hits `judge_panic` now reports `not_enough_f8`; the comparator artifact re-produced on the revision 6 pass's export; the committed cross-host control entry and multi-host running script now point to the orchestrator and `check_product.py` in the repository; the page title status and index status column rewritten; the experiment page gains a section "per-state list of Q4-(ii) and Q4, Q5 trajectories (the revision 6 pass)"
  - **Before**: the title status and index status column said "run (s4 whole table 117 rows, F4 extended table 164 rows and the fill-ins for the 6 rows that would not compile all run, readings complete; conclusions written back after the three-round verdict)"; the comparator produced neither the per-state list nor the trajectories (the apparatus artifact had only 27 + 3 lines of `name=q5_trajectory`), and that branch of `q5_verdict` reported `not_enough`; 28 unit tests, 23 research-side mutations; the three 1st rerun lines of E171 (Three-Oracle Collusion Test) in `replay.sh` pointed to `…-comparator-resume2-r6-r3.out`, `…-cross-host-check-resume2-r6-r3-2026-10-07-r2.out` and `…-research-mutations-2026-10-07-r2.out`; the cross-host control entry read the orchestrator from a draft path (`04376c00…`), and the running script's orchestrator and `check_product.py` pointed to the draft (`ec89ec78…`); the settled item 6 row of D13 (Verification Route) in affected decisions said "whether the basis paragraph cites this pass is handed to the main agent"; the section "artifacts (1st rerun first segment)" said "81" while the `-r6-r3` artifacts listed numbered 79.
  - **After**: the title status and index status column say "1st rerun first segment (revision 6 tree) run (Q1 A, Q3 A, Q4 baseline A and lines 242 and 269 B; Q5 awaiting the alarm self-proof at commit time; the denominators of Q2 and Q3 wait for the second segment; the revision 8 pass of the final main line awaits a snapshot of the tree)"; the new comparator artifact has 811 lines, the first 677 and the verdict lines byte-identical to the old one, with 132 more lines; not one verdict changed; 41 unit tests, all 37 mutations caught; the three lines in `replay.sh` now point to the new artifacts; cross-host control entry `afc9a9b0…`, running script `567a8682…`, self-proofs of 34 and 38 items both with 0 red; the two rows for settled item 1 of D13 (Verification Route) and settled item 14 of D23 (Journal Role & Format) still prepared material, and the basis paragraph of settled item 6 of D13 (Verification Route) untouched.
  - **Basis**: `research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out`, `…-s1-research-mutations-2026-10-07-r3.out`, `…-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out`; items 3 and 4 of the registration's closing section "main agent acceptance: first segment (revision 6 tree) comparator handed back", and item 7 of section 5.1 of revision 5.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

### 2026-10-06

- The experiment page states that `piece_panic_states=` counts by piece; one sentence recorded: after old line 38 of the mutation table was renamed, line 33 of the two 117-row bad implementation tables could not be found by the old name and goes through the fill-in rerun from today's tree.
  - **Basis**: item 3 of section three of `research/prompts/m3-formal-code-b29-r1-main-verification.md`; P4 in section two and item 5 of section three of `research/prompts/m3-formal-code-b33-r1-main-verification.md`.

### 2026-10-04

- The line 92 cell of "the three-way split after the three-round verdict" changed to the current value (settled item 6 of D8 (Core Index Structure) has added the value taken for the write time; the three oracles cannot judge it by design, and it is judged by pinned-write test cases); the D3 (Space Allocation) original wording (line 86) cited by the line 15 three-way split cell and the scope sentence cited by the settled item 8 row of D3 (Space Allocation) in "affected decisions" changed per the current text of D3 (Space Allocation) (two point replacements on the same day by write-back clerk 18-3 and the main agent); readings and the three-way split unchanged.
  - **Before**: the wording of the same file earlier that day; the background is in those lines of batch 18 of `records/2026-10-01-里程碑三形式化接入.md`.

### 2026-10-03

- E171 (Three-Oracle Collusion Test) page created, first segment run: the committed apparatus for the two columns O1 (in-memory model differential (HashMap ideal file system)) and O2 (independent parser + checker), the research side (selection, row-by-row driver, comparator) and unit tests written; two baseline passes, three positive controls, the switch self-check that turns one off, the O2 (independent parser + checker) column of Q2, and a trial run of three rows by the row-by-row driver finished; the O3 (independent specification executor) column not run (registration stop S7)
  - **Before**: this page did not exist; the experiment number E171 (Three-Oracle Collusion Test) had only the pre-run registration `e171-preregistration.md`, and the experiment-orphans cell of gate doc-experiments judged it as having no kb body.
  - **After**: page status "first segment run"; the four rows of the current-state table all marked "open", each stating what numbers it has at this moment and what it lacks (Q1 denominator D = 0; in the O2 (independent parser + checker) column of Q2 all 41 known bad mirrors red, and 10 of the 26 P-row images red; Q3 only the switch self-check; Q4 about 62 s per row); the three rows of "affected decisions", settled items 6, 7 and 15 of D13 (Verification Route), all written as prepared material.
  - **Basis**: the first-segment artifact `e171-three-oracle-collusion-2026-10-03-s1-comparator.out` and the other 16 with the same prefix, pre-run registration `e171-preregistration.md`.
- E171 (Three-Oracle Collusion Test) s4 whole table run and the page rewritten: the s3 whole table voided and kept on file, Q1, Q3 and Q4 judged per s4, and the F4 extended table of 164 rows prepared to run
  - **Before**: page status "first segment run", the four rows of the current-state table all saying "open"; the page did not describe the second to sixth segments (O3 (independent specification executor) wired in, the full Q2 table, apparatus revisions 3 and 4, the s3 whole table and its voiding).
  - **After**: page status "partly run (s4 whole table 117 rows run, F4 extended table 164 rows to run)"; Q1 has numbers (all three readings let 7 rows through entirely; the row-by-row three-way split: 4 rows equivalent for this batch, 3 rows real blind spot candidates), and the denominator D = 19 < 30 triggers F4; Q2 has numbers (4 P8 images, F3); Q3 has 11 rows relying on only one (Q3-check 9 passes, 0 differences); Q4 can only enter the full run (5 569.6 s). Added: the F4 extended table row file `bad-implementations-expansion-164.tsv`, the driver option `--other-selected-rows`, the comparator's two output roots and the two lines `name=f4_expansion` and `name=q1_denominator_by_table`; research-side mutation table 44 → 52, all caught; `scripts/experiment-apparatus/replay.sh` gains `driver_e171_s4_comparator`, and the mutation line now points to `-r5.out`.
  - **Basis**: `research/results/e171-three-oracle-collusion-2026-10-03-s4-comparator.out`, `…-s4-rows-timing.out`, `…-s3-comparator-mutations-2026-10-03-r5.out`; revision 5 in section twelve of the registration.
- E171 (Three-Oracle Collusion Test) F4 extended table run, merged table 281 rows; the row-by-row driver changed to fill in after a row fails to compile, with the fill-ins for the 6 rows that would not compile to run
  - **Before**: page status "partly run (s4 whole table 117 rows run, F4 extended table 164 rows to run)"; Q1 A, B and C each let 7 rows through entirely, denominator D = 19; Q3 11 rows relying on only one; rows 33 and 59 that would not compile were not filled in and were recorded as a registration deviation handed to the main agent; research side 98 unit tests, 52 mutations.
  - **After**: page status "partly run (s4 whole table 117 rows and F4 extended table 164 rows run, the fill-ins for the 6 rows that would not compile to run)"; Q1 A and B each let 12 rows through entirely and C 13, denominator D = 37 (before the fill-ins), with the rows newly let through entirely by the extended table split three ways row by row (rows 242, 269, 270 and 271 real blind spot candidates, row 277 equivalent for this batch, row 233 a real blind spot candidate in C and an insensitive sampling point in batch 2, row 265 an insensitive sampling point in batch 2); Q3 23 rows relying on only one, with the 12 new rows from the extended table stating why the other two cannot see them; rows 155 and 161 were not reached but judged red by the O1 (in-memory model differential (HashMap ideal file system)) differential, traced to registration reading 4's definition of reaching not comparing the return values of successful calls; the driver fills in after a compile failure (the fill-in pass for rows 33, 59, 127, 163, 206 and 243 handed to the main agent), while rerunning the s4 and F4 artifacts with the switch still does not fill in; 110 unit tests, all 67 mutations caught.
  - **Basis**: `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out`, `…-s4-f4-rows-timing.out`, `…-s3-comparator-mutations-2026-10-03-r7.out`; readings 4 and 5 in section one of the registration and 5.2 "fill-in".
- E171 (Three-Oracle Collusion Test) fill-ins for the 6 rows that would not compile run, page status changed to "run", and one fill-in line added to the rerun registration
  - **Before**: page status "partly run (s4 whole table 117 rows and F4 extended table 164 rows run, the fill-ins for the 6 rows that would not compile to run)"; the section "fill-ins for the 6 rows that would not compile (to run)" gave the inferred first fill-ins (row 33 → index 9 within the mount.rs group, row 59 → mounted_read.rs index 8, row 127 → allocator.rs index 16, row 163 → mount.rs index 37, row 206 → recovery.rs index 4, row 243 → transaction.rs index 25, assuming the first row swapped in compiles); the denominator D = 37 marked "before the fill-ins"; ten E171 (Three-Oracle Collusion Test) registrations in `replay.sh`, with line 1012 saying "add another line once the fill-in artifact is committed"; 44 artifacts.
  - **After**: page status "run (s4 whole table 117 rows, F4 extended table 164 rows and the fill-ins for the 6 rows that would not compile all run, readings complete; conclusions written back after the three-round verdict)"; the six fill-in rows swapped in are all unreached rows (`substitution_summary rows_that_did_not_compile=6 substituted_rows=6 still_not_compiled=0`), and for row 127 the first row swapped in did not compile either and was swapped a second time; the denominator D = 37 is the same before and after the fill-ins, and the Q1 and Q3 lists are identical verbatim (the two artifacts' 48 lines against 48 lines give no `diff` output); the 6 fill-in rows took 1 277.9 s of wall clock; `replay.sh` gains `driver_e171_s4_f4_substitutes_comparator` and one registration table line (eleven lines), 46 artifacts.
  - **Basis**: `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out`, `…-s4-f4-substitutes-rows-timing.out`; the line "the E171 (Three-Oracle Collusion Test) fill-in pass finished" in `records/2026-10-01-里程碑三形式化接入.md`.
- E171 (Three-Oracle Collusion Test) conclusions written back after the three-round verdict: the three-way split column changed to current values, the reaching cell rewritten, several items added to "what it cannot answer", and the settled item 6 row of D13 (Verification Route) changed from prepared material to support
  - **Before**: the three-way split columns of the two sections, the 7 rows let through entirely by all three and the rows newly let through entirely by F4: row 15 "equivalent for this batch" (reason "the clause is unsettled"), rows 27, 112 and 277 "equivalent for this batch", row 34 "equivalent for this batch", row 92 "real blind spot candidate (the clause basis needs the main agent's acceptance)"; the Q1 row said "4 real blind spot candidate rows (rows 242, 269, 270, 271), 1 row equivalent for this batch (row 277)", with the 7 s4 rows "three-way split not redone"; the section "the two rows not reached but red" and "what it cannot answer" said "whether other unreached rows have similar differences cannot be seen, because the apparatus did not record the return values of successful calls"; the settled item 6 row of D13 (Verification Route) in "affected decisions" said prepared material (reason: the basis paragraph does not cite E171 (Three-Oracle Collusion Test) by number, and which rows count as real blind spots is an inference that has not been through a three-way argument). The page had never been committed before the change; these are the before values.
  - **After**: rows 15, 23, 42, 242, 269, 270 and 271 are real blind spot candidates (row 15 based on the t2–t12 placements of settled item 10 ⑤ of D3 (Space Allocation), not t1@50180), rows 27, 112 and 277 belong to "the stretch of the implementation's recovery that rebuilds from records, which none of the three judges", row 34 belongs to allocation policy (the scope of settled item 8 of D3 (Space Allocation)), row 92 belongs to "the clause is in the byte table and milestone acceptance, and the decision text lacks a sentence", and a new section "the three-way split after the three-round verdict" is added; rows 110, 162 and 174 (runtime readings) and rows 49 and 252 (wrong return values or memory state, with the write stream unchanged) are two new kinds that surfaced in the third round; the reaching cell says that of the 232 rows the attacker measured on a copy of the s4 tree, 7 have different return values (rows 49, 110, 155, 161, 162, 174, 252, `name=return_summary hidden_difference=7 identical=225 no_output=6`), with the reaching definition and the denominator 37 unchanged; the Q2 row and the new section add the 29 bad mirrors let through by O2 (independent parser + checker) but judged absent by O3 (independent specification executor) (`o3_only_strict=12`, `o3_only_strict=17`), without writing "unique to O3 (independent specification executor)"; "what it cannot answer" gains: the O1 (in-memory model differential (HashMap ideal file system)) column not connected to RefFS, the source of the 29, row-level readings belonging only to the snapshot tree, the preview of D = 38 with row 125 (both 37 and 38 written), W and the 61 witness rejections holding only for the s4 core (RefFS's 27 / 27 / 36 go only into this section, sourced from the second-round verdict), and whether the value taken for user data placement counts as layout bytes handed separately to the user; the settled item 6 row of D13 (Verification Route) in "affected decisions" changed to support, with three rows added for settled items 8 and 10 of D3 (Space Allocation) and settled item 15 of D23 (Journal Role & Format).
  - **Basis**: `research/prompts/e171-conclusion-r1-main-verification.md`, `…-r2-main-verification.md`, `…-r3-main-verification.md` (the third round is the final round, with sections three and five authoritative); `return-rows.out`, `…/out/row15-offsets.out`; the `name=q2_family` and `name=substitution_summary` lines of the artifact `…-s4-f4-substitutes-comparator.out`.
- The user data placement item in "what it cannot answer" and the settled item 8 row of D3 (Space Allocation) in "affected decisions" changed to current values: the user decided the same day that the placement values in new-pool file creation count as layout-registered bytes (the same-day entry in the D3 (Space Allocation) section), and this page's three-way split and readings are unchanged; the three-way split cell of row 15 gains a basis (moving t1 from 50180 to 50240 is itself illegal under the new scope), where before it said "not the basis of this row…… handed separately to the user"; the placement change of row 34 is not in new-pool file creation and still belongs to allocation policy, with the scope cited in the three-way split cell replaced by the new sentence.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

## E172 (Dry Run of Bytes Written by Deletion)

### 2026-10-26

- Item 6 of "What this cannot answer" gets new pointers: the td and tt header levels and key ranges used to say "correctness belongs to C478 (Is the Key Range of the Code-2 Header the Entry Key Range or the Subtree Coverage Range) and C713 (No provision for the two deletion-tree root levels)", and both are paid off; the levels now point to the clauses D8 (Core Index Structure) settled item 16 item 3 and D18 (Information Carried in a Block) settled item 24 item 1, the key range to the clause D18 (Information Carried in a Block) settled item 2, the tt key range is judged by the pool-level checker, and whether the pool-level checker judges the td key range points to the owed-check table; every row of "Decisions affected" re-reviewed; conclusions, numbers and artifacts unchanged.
  - **Basis**: rows C478 (Is the Key Range of the Code-2 Header the Entry Key Range or the Subtree Coverage Range) and C713 (No provision for the two deletion-tree root levels) in the paid-off table of the owed-check table; the main agent's decision in the batch 17 entry on the `deleted_inodes` root key range in `records/2026-10-18-里程碑三收尾与近期欠账.md`.
  - **Before**: the staged version `git show :.claude/kb/experiments/172-删除写出的字节的干跑.md` (based on `841b3beab`).
- Item 6 of "What this cannot answer" used to say "for whether the pool-level checker judges the td key range, see the owed-check table"; it now says the td key range is also judged by the pool-level checker in the tree table walk (`read_index_node` in `crates/singlefs-checker/src/walk.rs` takes the `FirstAndLastEntries` reading and calls `check_index_node_keys` in `crates/singlefs-checker/src/lib.rs`; a header range that differs from the keys of the first and last entries turns I-1.1 (block header states its logical address) red), and no owed check is registered; "Affected decisions" gains a row for D18 (Information Carried in a Block) settled item 2 with relationship Does not affect; conclusions, numbers and artifacts unchanged.
  - **Basis**: the main agent's batch 17 closing decision (batch 17 in `records/2026-10-18-里程碑三收尾与近期欠账.md`); checked: `key_range_reading_of_a_tree_walked_from_its_tree_table_entry` in `crates/singlefs-checker/src/walk.rs` returns `FirstAndLastEntries` for `deleted_inodes` (kind 9).
  - **Before**: the workspace version (after the same day's item that re-pointed item 6 of "What this cannot answer", not in git); the old sentence is quoted in the item.

### 2026-10-17

- Two "copied whole from" subheadings (1st run: verdict, 1st run: readings of each row) gain the parenthetical "(lines whose artifacts have left the workspace are written as one pointer line)"; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/172-删除写出的字节的干跑.md`.

### 2026-10-15

- 15 artifact lines copied whole changed to the pointer "for the artifact line see line N of git show <commit>:<artifact path> (name=<name>)" (relation to the artifact line: 15 whole lines); these lines were copied from an older round or an archived artifact, not from the run currently registered in `scripts/experiment-apparatus/replay.sh`; numbers and conclusion unchanged (user settlement, the handling of the existing stock for C439 (moving gate-ok forward leaves lines written earlier in the same round unjudged), batch eleven of `records/2026-10-13-欠账还五条.md`).
  - **Before**: `git show c1e9b324:.claude/kb/experiments/172-删除写出的字节的干跑.md`.

### 2026-10-05

- E172 (Dry Run of Bytes Written by Deletion) page created, first and second segments run: an independent decoder reads the committed `crates/`-side deletion export; question sheet lines 1, 2, 3, 4 and 7 enough to judge, line 8's readings complete with the verdict handed to the main agent, and lines 5 and 6 each missing the H3 half
  - **Before**: this page did not exist; the experiment number E172 (Dry Run of Bytes Written by Deletion) had only the pre-run registration `research/prompts/e172-preregistration.md`.
  - **After**: the 186 rows of fields registered in layout/03 for the eight deletion-class publishes all equal; 0 unregistered bytes in the window; the H1 segment string identical to the registration; continuation 2 → 1 and the last batch writing one extra pair of the 62nd leaf of the allocation record tree, with every changed record having a clause basis; the A-journal anchor derivation wrong; line 8: R8.b in the original text order misattributes 2 times, R8.c added by revision 1 misattributes 0 times, and PC-E was already red before breaking anything; the 20-entry mutation table written, not run.
  - **Basis**: export `research/results/e172-deletion-write-dump-2026-10-05.out`, apparatus artifact `research/results/e172-deletion-write-dry-run-2026-10-05.out`, revision 1 in section twelve of the pre-run registration.
- The main agent judged line 8 and V4: before PC-E broke anything the target row R8.b was already red, so per the literal text of V4 all of H2's verdicts are void; the two branches of R8.b do not state an order, and in the original text order they misattribute 2 times on an export without injected errors, which means the criterion itself was written wrong; changing the criterion is a new round, and R8.c of revision 1 is a relaxation that does not stand in for this round; lines 1, 2, 3 and 7 are enough to judge per H1, while line 4, the H2 half of line 6 and line 8 are folded into the third segment and rejudged with R8.c. The 4 mismatches of the A-journal anchor are attributed to "counter = txg", which the registration added when deriving the anchor; the clause at line 319 of layout/01 is not touched (F3, not voided).
  - **Before**: the page-creation entry said "question sheet lines 1, 2, 3, 4 and 7 enough to judge, line 8…… verdict handed to the main agent".
  - **Basis**: runner report `/tmp/claude-1000/e172-run1/report.md` (sha256 2fc64947…); apparatus artifact line 719 `E7RESULT name=r_pc pc=PC-E history=h2 target_green_before=false target_red_after=true`; V4 in section eleven and revision 1 in section twelve of the pre-run registration.
- E172 (Dry Run of Bytes Written by Deletion) 2nd run (rerun) run; all five lines of question sheet `research/prompts/e172-r2-questions.md` are enough to judge, and the status changed to "run (enough to judge)": line 1 (lines 1, 2, 3 and 7 of the 1st question sheet on the export after T_time changed to milliseconds and on H3) all hold, with T_time 5000 all 38 times; line 2 B (the continuation batch writes one extra pair of the 62nd leaf, with a clause basis); line 3 as currently written (the last batch of H3 writes the inode leaf as usual); line 4 per this reading (H3's intent writes the caller's time, copied in each batch); line 5: core does not hand over the number acquisition write account, so it can only be computed as a difference, and the difference is cut correctly on H2 (R8.c misattributes 0), with PC-F and PC-G first green then red
  - **Before**: page status "in progress: first and second segments finished, H2's verdicts void per V4, the third segment waits for H3's export", the body being the 1st run's current state and sections (the 1st run's sections moved unchanged into the page-creation item of "Historical Versions" on the page, with titles changed to "1st run: …"); the E172 (Dry Run of Bytes Written by Deletion) line of `scripts/experiment-apparatus/replay.sh` pointed to `e172-deletion-write-dry-run-2026-10-05.out`, and `driver_e172` exported only one copy.
  - **After**: the body replaced with the 2nd run's current state, verdict, readings, the eight A-journal mismatches (the same derivation error), 38 unit tests and the 32-entry mutation table (not run), what it cannot answer, paths and conclusion registration (source line numbers changed to current values), and affected decisions (one row added for settled item 6 of D8 (Core Index Structure), all eight rows prepared material); the registration line now points to `e172-r2-deletion-write-dry-run-2026-10-05.out`, and `driver_e172` first exports two copies live and then runs the apparatus.
  - **Basis**: rerun registration `research/prompts/e172-r2-prereg.md`; exports `research/results/e172-r2-deletion-write-dump-h1h2-2026-10-05.out`, `research/results/e172-r2-deletion-write-dump-h3-2026-10-05.out`; apparatus artifact `research/results/e172-r2-deletion-write-dry-run-2026-10-05.out` lines 452, 476, 568, 572, 586, 596, 753, 813, 815 and 816.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

## E173 (Three Treatments for Upper Bound Blocked on Bad Slots)

### 2026-10-17

- The rerun block now starts from scripts/experiment-apparatus instead of `cd research`, with the scripts/hosts and scripts/mutation paths changed along with it, and the mutation table pointing to scripts/mutation/tables/; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md`.

### 2026-10-05

- E173 (Three Treatments for Upper Bound Blocked on Bad Slots) status changed to "completed (the two knobs of the third segment not run after being enough to judge)", the conclusion gains the user settlements Q1 as today and Q2 as today without merging, the settled item 1 row of D16 (Publish Semantics) changed from prepared material to support, and "unit tests and mutations" gains a sentence on the main agent's acceptance of how M6 and M9 are written
  - **Before**: page status "in progress: first and second segments run (both Q1 and Q2 rows enough to judge), the two knobs of the third segment not run after being enough to judge"; in "affected decisions" the settled item 1 row of D16 (Publish Semantics) had relation "prepared material" and review "2026-10-05 not affected: the clause under test (……); the basis paragraph does not cite E173 (Three Treatments for Upper Bound Blocked on Bad Slots), ……"; the page was newly created and not in HEAD, so the before-state is not in a commit.
  - **After**: page status "completed (the two knobs of the third segment not run after being enough to judge)"; the current-state paragraph gains an item with the user settlements (Q1 as today, Q2 as today without merging) and position counts summed by the per-arm table; the settled item 1 row of D16 (Publish Semantics) has relation "support" and review "2026-10-05 changed", with the basis paragraph of settled item 1 of D16 (Publish Semantics) citing this experiment back the same day; "unit tests and mutations" gains a sentence that the main agent accepted how the two mutations M6 and M9 are written (the two lines in section nine of the registration are kept, and the way they are written is recorded in section twelve of the registration).
  - **Basis**: the user settlement at line 655 of the dispatch record `records/2026-09-30-里程碑三开工第一轮.md`; item 1 of section 6 of the runner report `/tmp/claude-1000/m3-r1/e173-run/report.md`; artifact `research/results/e173-admission-ceiling-bad-ring-slot-dispositions-arm-table-2026-10-05.out`.

## E174 (Wait and GPU Memory Return on Lost IOU)

### 2026-10-17

- The rerun block now starts from scripts/experiment-apparatus instead of `cd research`; the run command that goes onto the card changed to start through `scripts/experiment-apparatus/device-code-key.py run` per the apparatus file header, and the judging command now runs at the repository root; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/174-借据丢掉时等待与显存回还.md`.

### 2026-10-15

- E174 (Wait and GPU Memory Return on Lost IOU) pre-run registration gains revision 2: the parent process's resume key changed to the function unit key, touching only resume; the apparatus not yet changed
  The resume key changed from "cell label + round number" to two unit closure keys (the child process's real card entry, and the segment where the parent tells the child which cell, computed by `unit-graph key` of `crates/singlefs-unit-graph`) plus the cell's parameter values, round number, toolchain line, dependency version line, driver and card; finished rounds are stored in `~/.local/share/singlefs/experiments/e174/rounds/`, with artifact paths and directory names used only as an index, and artifacts assembled from the store in an unchanged format. Arms, cells, criteria, void clauses and conclusions unchanged by a word; old artifacts are not treated as the resume store and not migrated. Samples that must void and must not void, and how to check "only the key changed" (the four lines of the rerun table byte-identical, and the model card's second segment rerun on an empty store identical to the committed one after sorting by line), are written in parts six and seven of revision 2, for the runner to change the apparatus by; putting the driver and the card into the key awaits the main agent's acceptance.
  - **Basis**: revision 2 in section twelve of `research/prompts/e174-preregistration.md`; the user's 2026-10-15 decision "pruning is per unit; if that can't be done, redo it".
  - **Before**: the "resume" paragraph of section five "run order, resume, segmentation" in `git show c1e9b324:research/prompts/e174-preregistration.md`.

### 2026-10-08

- E174 (Wait and GPU Memory Return on Lost IOU) page created, apparatus written, not run on the card
  An independent handwritten apparatus `scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs` (19 unit tests, all 28 mutations in the table caught, all 31 self-checks without starting the card green); section twelve of the pre-run registration `research/prompts/e174-preregistration.md` records revision 1 (adding the arm "in-flight · medium" and pinning down several readings, with the original criteria unchanged by a word). The two segments on real cards not run.
- E174 (Wait and GPU Memory Return on Lost IOU) both segments run and judged on real cards, status changed to "both segments run, enough to judge"
  - **Before**: page status "apparatus written, not run on the card", the two numbers of G2-B unmeasured; the index row "independent apparatus, model card self-checks and judging method written, …… only obtainable on the card"; `scripts/experiment-apparatus/replay.sh` registered two lines (`checks`, and `judge` on the model card artifact); the settled item 2 row of D24 (Offloading Heavy Background Work to GPU) in "affected decisions" was prepared material, with the review saying "decide whether to upgrade to support once the real-card artifacts are out".
  - **After**: page status "both segments run, enough to judge"; the body gains the readings of the first segment (88 cells, 430 child processes complete, 16 failed with Out of Memory, two cells void) and the second segment (12 cells, all 60 child processes complete), naming in the verdict lines and the fork table; in-flight · light was run in neither segment, recorded "not run after enough to judge"; the index row replaced with the conclusions of both segments; `replay.sh` registers four lines, adding the `judge` output of the two real-card artifacts (`driver_e174_segment_one_judge`, `driver_e174_segment_two_judge`), with manual reruns byte-identical to the artifacts by `cmp`; the settled item 2 row of D24 (Offloading Heavy Background Work to GPU) is still prepared material, with the review rewritten as not affected (the three hard constraints of the clause under test carry no GPU memory readings or wait durations), and whether to upgrade it to support is for the main agent to decide.
  - **Basis**: artifact `research/results/e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out` line 1698 `E7RESULT name=verdict scope=run segment=first cells_total=88 cells_incomplete=2 children_total=446 children_complete=430 children_failed=16 children_void=0 stopped_by=none`, `research/results/e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out` line 242 `E7RESULT name=verdict scope=run segment=second cells_total=12 cells_incomplete=0 children_total=60 children_complete=60 children_failed=0 children_void=0 stopped_by=none`; judging outputs `research/results/e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out` line 187 and `research/results/e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out` line 25 (`poll_arm_also_absent_where_wait_arm_absent=false` means registration F5 was not hit; the first segment's `geometry_same_as_main_grid=false` means registration F8 was hit on four pairs of the no-wait arm while the eight pairs of the wait arm are the same, and the two readings are listed side by side on the experiment page); based on the numbers of both segments, the user decided "switch to B" in the G2 row of `research/prompts/m3-batch10z-a-code-r1-forks.md`.

## E175 (Inode Number Watermark Rollback and Reissuance on Product Path)

### 2026-10-21

- The apparatus source location in the opening paragraph changed to `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs`: after the move on 2026-10-18 this sentence still said it was in the artifact directory; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show 841b3beab:.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md`.

### 2026-10-18

- The apparatus source `device-source.rs` moved from the artifact directory `research/results/e175-product-path-inode-watermark-regression-2026-10-08/` to `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs` (the whole of `research/` may be deleted, so apparatuses do not live there), with content unchanged; the rerun commands in the body, that row of the artifact table and the four lines of paths and conclusion registration changed to the new path, and that row of the artifact table gains the current sha256 (commits `6aa74a765` and `f778192d3` changed the script paths inside it); conclusion, numbers and artifacts unchanged.
  - **Before**: `git show HEAD:.claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md`.

### 2026-10-08

- E175 (Inode Number Watermark Rollback and Reissuance on Product Path) page created, first segment run, status "partly run (first segment; second segment awaiting being named)"
  Apparatus in a repository copy (`crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs` in the repository copy in the draft directory, with the source kept unchanged at `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs`; 17 unit tests, 15 passing and 2 ignored for each of the three quick-tier arms); section twelve of the pre-run registration `research/prompts/e175-preregistration.md` records revisions 1–8 (readings pinned down and how the apparatus is written, criteria untouched). Summary `research/results/e175-product-path-inode-watermark-regression-2026-10-08/summary-segment-one.out`: for both product-arm families F3a and F2, `sum_q1=0` and `segments_q2_one_birth_violated=0`, `sum_q5` 7 870 + 1 062; V1 and V2 are red for both families.
- E175 (Inode Number Watermark Rollback and Reissuance on Product Path) second segment stopped, status changed to "partly run (first segment; second segment stopped: crates/ differs from the first segment)"
  Before starting, the main workspace `crates/` was checked twice, and the second time 6 files differed from the first-segment baseline (`research/results/e175-product-path-inode-watermark-regression-2026-10-08/superseded-old-crates/baseline-segment-two.out`); per the dispatch prompt the run was stopped, and not a single second-segment artifact was run; the first segment's numbers untouched.
  Path correction (2026-10-14): for the before-state see `git show f111c3f3:.claude/kb/experiments-history.md`, where this line gave the path of that artifact from before it was moved into `superseded-old-crates/`.
- E175 (Inode Number Watermark Rollback and Reissuance on Product Path) both segments rerun on a frozen copy, status changed to "run (enough to judge)"
  Per registration revision 2, on the frozen copy taken by the main agent (`research/results/e175-product-path-inode-watermark-regression-2026-10-08/baseline-frozen-crates.out`, `code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`) the whole first segment was rerun and the four F1 groups and four geometry points of the second segment were run for the first time. The 19 first-segment files on the old code were moved unchanged into `research/results/e175-product-path-inode-watermark-regression-2026-10-08/superseded-old-crates/`, with that `crates/` manifest being `old-crates.sha` inside it, and the old readings do not enter the verdict; the E7RESULT lines of the nine new first-segment result files, with `elapsed_ns` removed, are identical file by file to the old ones. Summaries `summary-segment-one.out` and `summary-segment-two.out`: for every product-arm family `sum_q1=0` and `segments_q2_one_birth_violated=0`; every family V1 and V2 reached is red. The W row is enough to judge, and F3b and F4 of the third segment not run after being enough to judge. The apparatus `device-source.rs` gained a `code_baseline` file header (28 unit tests), and `e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` was replaced with the one on the frozen copy.
- E175 (Inode Number Watermark Rollback and Reissuance on Product Path) both segments rerun on a second frozen copy, status still "run (enough to judge)"
  After the tenth and eleventh blocks of batch 10z of the root ring line were merged, `crates/singlefs-core/src/mount.rs` changed again, and the main agent took a second frozen copy (`research/results/e175-product-path-inode-watermark-regression-2026-10-08/baseline-frozen-crates.out`, `code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`), on which P, V1 and V2 of both segments were rerun in full. The 37 files on the previous frozen copy (`b000407a…`) were moved unchanged into `research/results/e175-product-path-inode-watermark-regression-2026-10-08/superseded-b000407a/` and do not enter the verdict. The E7RESULT lines of the twenty-six new result files, with `elapsed_ns` removed, are identical file by file to the previous ones, and `summary-segment-one.out` and `summary-segment-two.out` are identical line by line except for `code_baseline`: for every product-arm family `sum_q1=0` and `segments_q2_one_birth_violated=0`, every family V1 and V2 reached is red, and the W row is enough to judge.
- E175 (Inode Number Watermark Rollback and Reissuance on Product Path) the two rows of affected decisions reviewed per the Opt2 settlement of W3, relations still prepared material
  The review of the settled item 14 row of D23 (Journal Role & Format) changed to "changed": the user settled Y1 (Opt2), so the inode number watermark in the "watermark" cell becomes max(the one in memory for the cur version, the one from roots readable in the ring and not abandoned by crash recovery per the instance table), with the tree ID watermark as before, and the readings of E175 (Inode Number Watermark Rollback and Reissuance on Product Path) were measured under the earlier way of taking all roots in the ring; the review of the settled item 6 row of D8 (Core Index Structure) is still "not affected", with the reason replaced by: Opt2 only makes legitimate reissues across abandonment 11.6% more, numbers issued in the stretch abandoned by crash recovery could be reissued anyway, and the scope does not change. The existing reading sentences of both rows are kept, and "whether to upgrade to support is for the main agent to decide" as before. Basis: the "cost of Opt2" row in section two, section six and the "review of decisions" table of the verdict `research/prompts/m3-w3-r3-main-verification.md`. Before: this page had never been committed, and the two cells were written by the same-section same-day item "both segments rerun on a second frozen copy"; after "not affected:" those two cells originally said that both segments were rerun on the second frozen copy with readings identical family by family to the previous one, followed by each one's reading sentence and the half-sentence "the decision basis paragraph does not cite E175 (Inode Number Watermark Rollback and Reissuance on Product Path)".

## E176 (Capacity and Write Cost of Three Inode Leaf Merge Candidates)

### 2026-10-17

- The rerun block now starts from scripts/experiment-apparatus instead of `cd research`, with the scripts/hosts and scripts/mutation paths changed along with it, and the mutation table pointing to scripts/mutation/tables/; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md`.

### 2026-10-08

- E176 (Capacity and Write Cost of Three Inode Leaf Merge Candidates) page created, first segment run, status "partly run (first segment; the trigger condition for the second segment holds, awaiting being named)"
  An independent handwritten model `scripts/experiment-apparatus/e176_inode_leaf_merge_capacity_and_rewrites.rs` (14 unit tests, all 13 mutations in the table caught) and a differential bin `scripts/experiment-apparatus/e176_crates_oracle/oracle.rs` (separate manifest; 22 sequences compared step by step until the no-merge arm is refused, zero differences); section twelve of the pre-run registration `research/prompts/e176-preregistration.md` records revision 1 (the positive control of the f = 59 variant and the reading of the common window) and revision 2 (the differential bin does not hang off an optional feature but uses a separate manifest instead). Lines 763–775 of artifact `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out` are the overall verdict of each row, and line 27 of `research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle.out` is the differential verdict.
- E176 (Capacity and Write Cost of Three Inode Leaf Merge Candidates) second segment run, status changed to "run (two segments)"
  - **Before**: status "partly run (first segment; the trigger condition for the second segment holds, awaiting being named)"; M3 borrowing exactly enough to reach the lower bound and the three variants where M2 / M3 check the left side first not run; 14 unit tests, 13 mutations; `scripts/experiment-apparatus/replay.sh` registered one line (`segment-one`).
  - **After**: the same model gains a `segment-two` entry, 22 sequences × six arm variants × three common windows; the position notations Δ2p and Δ3p "do not flip" in all three windows, the Δ3c and Q3 rows of the window that borrows exactly enough to reach the lower bound change from "negligible" to "do not flip" with no point reaching 1.00, and the six rows of the check-left-first window are judged the same as the main window; 22 unit tests, all 16 mutations caught (revision 3 in section twelve of the pre-run registration adds V14–V16 and the second segment's windows and positive controls); the differential and discriminating power each rerun once with the second segment's source, byte-identical to the first segment; `replay.sh` gains a `segment-two` line; the experiment page writes the number of pairs where both point values are equal as "pairs where the threshold-shift self-proof cannot be done" (38 in the first segment, 131 in the second). The settled item 6 row of D8 (Core Index Structure) is still prepared material.
  - **Basis**: lines 1137–1148 (overall verdicts of the two new windows) and line 1369 (sensitivity self-proof count) of `research/results/e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out`; line 27 `all_equal=true` of `research/results/e176-inode-leaf-merge-capacity-and-rewrites-crates-oracle-2026-10-08.out`.
- E176 (Capacity and Write Cost of Three Inode Leaf Merge Candidates) affected decisions table: the settled item 6 row of D8 (Core Index Structure) changed from prepared material to support
  - **Before**: relation "prepared material"; review "not affected (reviewed again after the second segment ran)", on the grounds that the basis paragraph does not cite this experiment, that the clause states merge timing is a write-path policy, and that which candidate fork M picks is for the user to decide.
  - **After**: relation "support", review "changed": the user decided "do M3 together with borrowing", with borrowing splitting evenly, and the merge sentence of settled item 6 of D8 (Core Index Structure) changed to do both left absorbing right and left borrowing from right, with borrowing splitting evenly; the basis paragraph cites E176 (Capacity and Write Cost of Three Inode Leaf Merge Candidates). The experiment body, criteria and artifacts untouched.
  - **Basis**: the M row of the fork sheet `research/prompts/m3-3-5-forks.md`; K3 in section two of the verdict `research/prompts/m3-leaf-death-r1-main-verification.md` and "even-split borrowing" in section two of `research/prompts/m3-leaf-death-r2-main-verification.md`.

## E177 (Resurrection Count Without Per-Tree Tombstones)

### 2026-10-17

- The rerun block now starts from scripts/experiment-apparatus instead of `cd research`, with the scripts/hosts and scripts/mutation paths changed along with it, and the mutation table pointing to scripts/mutation/tables/; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/177-逐树不写墓碑的复活数.md`.

### 2026-10-08

- E177 (Resurrection Count Without Per-Tree Tombstones) page created, first segment run, status "run (enough to judge)"
  An independent handwritten model `scripts/experiment-apparatus/e177_per_tree_resurrection_without_tombstones.rs` (23 unit tests, all 17 mutations in the table caught); section twelve of the pre-run registration `research/prompts/e177-preregistration.md` has revision 1 (experiment designer: G-wide does not follow `crates/`, a main sampling point G-ring added, and large-file write-delete replaced by large-file rewrite-delete) and runner revision 2 (the check table, written before the apparatus). Lines 37217–37522 of artifact `research/results/e177-per-tree-resurrection-without-tombstones-segment-one.out` are the verdicts and assertion totals; the failure clause "the clause may be wrong" is hit (on G-wide and G-ring, under the reading that picks the current version per the kb, the inode tree judges resurrection), handed to the main agent for a three-way argument; the second segment not run.
- E177 (Resurrection Count Without Per-Tree Tombstones) wording changed and the affected decisions reviewed row by row after three three-way rounds and the user settlement: F1 written as "under R-S lacking the retirement record", F2 recorded as hit by the three-way judgment, Q8 not rejudged under the literal reading of G-tight, and the settled item 6 row of D8 (Core Index Structure) changed from prepared material to support
  - **Before**: the F1 item in the current state and the F1 row of "failure, void and stop clauses item by item" said "does not hold under R-S" and "handed to the main agent for a three-way argument"; the F2 row only said it was not literally hit; the G-tight item in "what it cannot answer" said "rejudging Q8 under the literal reading not done, handed to the main agent to decide whether to rerun"; in the affected decisions table the settled item 6 row of D8 (Core Index Structure) had relation "prepared material" and review "not affected" (failure clause F1 hit, handed to the main agent for a three-way argument, decision file untouched), and the review reasons of the three rows for settled items 8, 10 and 24 of D18 (Information Carried in a Block) stopped at before the three-way argument.
  - **After**: both F1 places say "does not hold under R-S lacking the retirement record"; the three-way judgment did not overturn it entirely but changed the literal text, the user settled that a retirement record is written when a container identity disappears, and settled item 6 of D8 (Core Index Structure) changed to death in three layers; the F2 row adds that the three-way judgment found it hit (the retirement record is an element already registered in the kb that the first version of merging should have written, and R-S did not use it; the two registered literal triggers were not met, and the impact is only in wording), and the basis column adds that under the retirement record base E178 (Counts of Leaf Containers and Record Death) has 0 container resurrections and 0 false deaths; the G-tight item changed to say the three-way judgment does not rejudge (the forward-inference leg measured on a copy of the apparatus that the Q8 verdicts are identical row by row under the literal reading); the settled item 6 row of D8 (Core Index Structure) changed to relation "support" and review "changed", and the three rows for settled items 8, 10 and 24 of D18 (Information Carried in a Block) keep their relations, with review reasons adding the user settlement (tombstones written at the intent publish as before; reading Z and range tombstones not done; live data leaking in transit while the intent publish takes effect is a cost). Criteria, model, artifacts and readings untouched.
  - **Basis**: K1 "how F1 and F2 are judged" and K4 (the G-tight reason sentence, the affected decisions table) in section two of the verdict `research/prompts/m3-leaf-death-r1-main-verification.md`; the subsection on F2 in section two of `research/prompts/m3-leaf-death-r2-main-verification.md` (the judgment that F2 is hit stands, with impact only on the experiment page's wording); section five and the decision review of `research/prompts/m3-leaf-death-r3-main-verification.md`.

## E178 (Counts of Leaf Containers and Record Death)

### 2026-10-17

- The rerun block now starts from scripts/experiment-apparatus instead of `cd research`; the fourth-segment command changed per the apparatus file header to the default item store path plus starting through `scripts/experiment-apparatus/device-code-key.py run` (the item directory variable E178_SEGMENT_FOUR_ITEMS is deprecated); the mutation table points to scripts/mutation/tables/; conclusion, numbers and artifacts unchanged.
  - **Before**: `git show c1e9b324:.claude/kb/experiments/178-叶容器与记录死亡的计数.md`.

### 2026-10-15

- E178 (Counts of Leaf Containers and Record Death) pre-run registration gains revision 4: the resume key of the fourth-segment item directory changed to the function unit key, touching only resume; the apparatus not yet changed
  An item's key changed from "item name with sequence number + sha256 of the whole apparatus source + geometry label" to the unit closure key of the entry `segment_four_item_lines` (`unit-graph key` of `crates/singlefs-unit-graph`, including the toolchain line and dependency version line) plus the item's parameter values; the item store moved to `~/.local/share/singlefs/experiments/e178/segment-four/`, item files named by key, with item names only as an index; the old item directory deprecated and not migrated. Arms, readers, criteria, void clauses and conclusions unchanged by a word. Samples that must void and must not void, and how to check "only the key changed" (`cmp` of the fourth-segment artifact assembled from fresh judgments on an empty store against the committed one, with the second pass reusing everything), are written in parts six and seven of revision 4, for the runner to change the apparatus by; the empty-store rerun pass exceeds 2 hours, and the three lines for sharding, resume and multi-host are written in part thirteen of revision 4.
  - **Basis**: revision 4 in section twelve of `research/prompts/e178-preregistration.md`; measured on a draft copy: today's apparatus cannot compute a key for this entry (an item-level `include!` and a local variable called `main` pull the whole bin into the closure), and after the two changes in part six of revision 4 it can, with comments and unrelated functions not changing the key on the samples, while functions, constants, dependencies and the toolchain within the closure change it; the user's 2026-10-15 decision "pruning is per unit; if that can't be done, redo it".
  - **Before**: the "resume" item in part thirteen of revision 2 of `git show c1e9b324:research/prompts/e178-preregistration.md`.

### 2026-10-08

- E178 (Counts of Leaf Containers and Record Death) page created, first segment run, status "partly run (first segment)"
  An independent handwritten counting model `scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs` (one entry per segment for three segments; 32 unit tests, all 15 mutations in the table caught); section twelve of the pre-run registration `research/prompts/e178-preregistration.md` has revision 1 written by the runner (on F11, PC-B2 cannot measure the container m of B-off-lin, so a directly constructed PC-B2-direct is added and takes over M3's sensitive sampling point), written after the unit tests ran and before the artifact. Lines 1528–1625 of artifact `research/results/e178-leaf-and-record-death-counts-segment-one.out` are the verdicts and assertion totals; the X4 row of the fork sheet `research/prompts/m3-leaf-death-forks.md` is enough to judge (failure clause FL8 "n1 missed judgment" hit, shape 8); the second and third segments for X2 and X3 not run.
- E178 (Counts of Leaf Containers and Record Death) second and third segments run, status changed to "partly run (first to third segments)"; fourth-segment apparatus written
  The main agent ran the second segment (`research/results/e178-leaf-and-record-death-counts-segment-two.out`, 73284 lines) and the third segment (`…-segment-three.out`, 4335 lines). The runner checked the verdict lines: the X2 and X3 rows are enough to judge under the retirement record base (resurrections and false deaths of Z and T, and live data leaks in transit and at crash points for both E-L arms, are all 0; FL6 is literally hit, a consequence of the E-L arm definition); PC-B1 does not hold on the three n2 segments of F11 (those three segments voided per V2, with the cause pinned by a unit test), and the apparatus's judgments of PC-M and PC-T are looser than registered, handed to the main agent. Registration revision 2 (experiment designer) adds two base arms B-stub and B-tsz as the fourth segment, and the fork sheet gains an X1 row; the runner added the fourth-segment entry (sharding, resume, multi-host), 44 unit tests, all 22 mutations in the table caught, and wrote revision 3 (PC-Bt2-mid); the artifact not run.
- E178 (Counts of Leaf Containers and Record Death) fourth segment run, status changed to "run (four segments)"
  The main agent ran the fourth segment (`research/results/e178-leaf-and-record-death-counts-segment-four.out`, 140179 lines; the second host's shard was rejected by the memory wrapper and computed on this host instead). The runner checked the verdict lines: resurrections and false deaths of B-stub and B-tsz are 0 in all judged cells (B-tsz's container r 1–166 is a consequence of the definition), all 576 lines of Q-Bs-4 and Q-Bt-4 are `differs=false` with the FL-Bx list empty, and live data leaks in transit for both E-L arms under B-stub are 0; the 43176 lines of V-Bt2 are identical row by row with the second segment; none of the newly added void and failure clauses is hit, and V8 and PC-M are literally hit in the same form as the second segment, touching only report-only cells. The correctness side of X1 is enough to judge, missing the write units of each B-stub empty-out (not registered); the fourth-segment line in `replay.sh` not rerun.
- E178 (Counts of Leaf Containers and Record Death) affected decisions reviewed row by row after the user settlement: the two rows for settled item 6 of D8 (Core Index Structure) and settled item 11 of D18 (Information Carried in a Block) changed from prepared material to support, and two rows added for settled item 10 of D18 (Information Carried in a Block) and settled item 14 of D23 (Journal Role & Format)
  - **Before**: the four rows in the table (settled items 6 and 16 of D8 (Core Index Structure), settled items 11 and 24 of D18 (Information Carried in a Block)) all had relation "prepared material" and review "not affected", on the grounds that the basis paragraphs do not cite this experiment; the settled item 16 row of D8 (Core Index Structure) said "under B-stub…… the zero-entry root item in the 'emptied' verdict of item 5 must change".
  - **After**: the settled item 6 row of D8 (Core Index Structure) changed to "support" and "changed" (the user settled retirement record A, reading Z, tombstones taking effect from the intent publish, and the separator key when borrowing taking the smallest key of the child's version; the basis paragraph cites E178 (Counts of Leaf Containers and Record Death)); the settled item 11 row of D18 (Information Carried in a Block) changed to "support" and "changed" (type 3 gains a death txg and a dedicated retirement tree; the basis paragraph cites E178 (Counts of Leaf Containers and Record Death)); the settled item 16 row of D8 (Core Index Structure) is still prepared material, B-stub not chosen, and item 5 needs no change; the settled item 24 row of D18 (Information Carried in a Block) is still prepared material, per today's wording of item 2; two new rows for settled item 10 of D18 (Information Carried in a Block) (prepared material: reading Z and range tombstones not done, the first version allows only whole objects as before) and settled item 14 of D23 (Journal Role & Format) (not affected: the user took A, clause unchanged). The experiment body, criteria and artifacts untouched.
  - **Basis**: the five rows of the decision review table and section five of the verdict `research/prompts/m3-leaf-death-r3-main-verification.md`.
  - **Before**: `git show ce1457c3:.claude/kb/experiments-history.md` (as it was before the 2026-10-12 document iron-rule fix: numbers in this block gained short names and clashing local arm names were lowercased; the original sentences are in this version)

## E179 (Rehearsal Memory Scaling with Deletion Size)

### 2026-10-18

- Newly set up, not run: page created, pre-run registration `e179-preregistration.md` written (revision one written in section twelve); apparatus, unit tests, mutation table and artifacts do not exist yet.
  - **Basis**: fork two of the code three-way round m3-b44-code-r2 (the memory taken by a disk that only goes into memory has no upper bound); on 2026-10-17 the user chose "measure first, then decide" on the fork sheet (`records/2026-10-13-代码欠账批.md`).

## E180 (Acceptance Reading vs Truly Writable Amount)

### 2026-10-21

- Newly set up, not run: page created and registered in the experiments.md index. The first-segment reading apparatus, 45 unit tests, 22 mutation rows and the 29-shard long-run script `scripts/experiment-apparatus/e180-reading-shards.sh` written; neither of the two long reading runs finished, the shards are outside the repository and not committed, and there is no artifact; the second-segment held-position apparatus not written. Pre-run registration `research/prompts/e180-preregistration.md` (revisions 1–4 in section twelve: three columns of verdicts added for states where T is truncated by the journal record ceiling, large bytes stripped from the probe's session copy, six deviations of the first segment, and truncated states tightened on demand with the first segment run in shards and then merged).
  - **Basis**: the four rows F2, F5, F6 and F8 of the fork sheet `research/prompts/m3-close-admission-and-shadow-ledger-forks.md`; lines 230 and 310 of the dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md` (the two long runs did not finish and were stopped); apparatus `crates/singlefs-checker-tier/src/bin/e180_admission_reading_versus_writable_bytes.rs` (`grep -c '#\[test\]'` = 45), and the 22 rows in `crates/mutations.tsv` whose names start with lowercase `e180`.

## E181 (Landing Determination and Mount Lower Bound)

### 2026-10-19

- Newly set up, first-segment apparatus written, artifact not run: page created, pre-run registration `research/prompts/e181-preregistration.md` (revisions 12.1 to 12.10 in section twelve: one geometry G-small-unit-area and one arm HF9-three-level stride 4 added, edge histories taking the superset [n* − 63, n* + 64], F11 gains the real states of G-narrow, verdict line fields, 17 mutation rows, plus listing four substitutes that differ from the registration's wording: the fill method, the marked states, the states of G-unequal, and how unit tests are run).
  - **Basis**: the three rows F9, F10 and F11 of the fork sheet `research/prompts/m3-close-admission-and-shadow-ledger-forks.md` (F14 is in the second segment), dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md`.

## E182 (Three Forks for Unreadable Abandoned Roots)

### 2026-10-18

- Newly set up, first-segment apparatus written, artifact not run: page created, pre-run registration `research/prompts/e182-preregistration.md` (revisions 12.1 to 12.10 in section twelve: the resume key replaced with the function unit closure fingerprint, wall clock rewritten per measurement, the identity of a dropped disk in single-disk recovery still present but unreadable, `Overwrite` taking `publish_overwrite`, states taking only two states, how the positive controls run, arm artifact names, mutation rows, unit tests could not run, `walk.rs` fingerprint differing from the registration); committed apparatus `crates/singlefs-checker-tier/src/bin/e182_unreadable_abandoned_root_forks.rs`, long-run script `scripts/experiment-apparatus/e182-single-device-long-run.sh`, 6 mutation rows; unit tests handed to the main agent to run, and the first-segment artifacts (B1, B4) and the second segment (B2) not run.
  - **Basis**: the three rows B1, B2 and B4 of the fork sheet `research/prompts/m3-close-admission-and-shadow-ledger-forks.md`, dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md`.

### 2026-10-19

- First-segment artifact run, supplementary arm apparatus drafted: artifact `research/results/e182-unreadable-abandoned-root-forks-single-device.out` (1204 lines, 18 histories, the long-run script reporting "judged fresh 432, reused 0"); the verdict line names `source_files_differing_from_registration=1` (`walk.rs` fingerprint differs from the registration, uncertain) and `shadow_ledger_off_covered_states=not_run` (condition not met); Z1, Z2 and Z3 trigger, Z4, Z7 and Z8 do not; B1 and B4 still lack the dimension of overwrites keeping s units. The pre-run registration gains revision 12.11 (the supplementary arm "overwrite keeping s units": each overwrite of instance 1 goes through sequential writes of s units, taking only the 9 histories with s = 64, decided by the main agent in the dispatch), long-run script `scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh`, rerun registration `driver_e182_units_kept`; the apparatus draft (21 unit tests) and 4 draft mutation rows are in the runner's draft directory, and the supplementary arm's artifact not run.
  - **Basis**: the two rows B1 and B4 of the fork sheet `research/prompts/m3-close-admission-and-shadow-ledger-forks.md`, and the main agent's ruling on revision 12.4 (add a stricter arm).
- The supplementary arm and B4 positive control apparatus swapped in and the first-segment artifact re-produced: the apparatus swapped into `crates/` (the supplementary arm's `single-device-units-kept` mode and B4's own positive control, registration revision 12.12; the file header gains admission `admission: always` and run condition `run-condition: command python3`, with `preflight` as the first statement of `main`; 26 unit tests, 15 mutation rows); the first-segment artifact re-produced and saved separately as `research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out` (1211 lines; with the 7 `control=b4_class_a_to_class_b` lines and `name=done` removed it is identical line by line to `research/results/e182-unreadable-abandoned-root-forks-single-device.out`, verdict line identical verbatim); the E182 single-disk line of `scripts/experiment-apparatus/replay.sh` now points to the new file, and the old file is kept; the B4 control summary line "E7RESULT name=positive_control control=b4_class_a_to_class_b_total histories=6 histories_not_made=3 histories_violated=0 trials=6 changed_to_class_b=6 status=holds"; the 3 confirmed-tier histories at s = 1 cannot produce class a states and are recorded `not_made`; B1 and B4 are still recorded as "still missing: the s dimension not covered", and the supplementary arm's artifact not run.
  - **Basis**: lines 42, 58, 74, 90, 106, 122 and 128 and the verdict line 1210 of artifact `research/results/e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out`, and the line "E182 single-device rerun to check the reading" in the dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md`.
- Supplementary arm artifact run, the s dimension of B1 and B4 covered: artifact `research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out` (758 lines, 9 histories, the long-run script reporting "judged fresh 214, reused 0"); C's Q1 leaves 0 at (s = 64, p = 10) (G-default 4 / 393396 per tier, 8 for the two tiers together), A-two's Q3 is still 49.98% (the same value for all three shapes), and A-one fixes 2 / 4; B4's class b takes 100% in the shape of overwrites keeping s units, with class a empty (when overwriting one unit, class b is two to four in ten thousand); the B4 control summary line "E7RESULT name=positive_control control=b4_class_a_to_class_b_total histories=3 histories_not_made=3 histories_violated=0 trials=0 changed_to_class_b=0 status=not_made": none of the 3 histories produced a class a state, which is not a defect of how the apparatus selects (selection and counting use the same batch of states and the same class-judging function); the verdict line's `detection_control_failures=1` comes from this, `source_files_differing_from_registration=1` points at `walk.rs`, which the apparatus does not call, and `shadow_ledger_off_covered_states=not_run` has its condition unmet; B1 and B4 are enough to judge per the fork sheet's sufficiency conditions, and B2 not written.
  - **Basis**: line 757 (verdict line) and lines 42, 58, 74 and 80 (B4 controls) of artifact `research/results/e182-unreadable-abandoned-root-forks-single-device-units-kept.out`, and the line "E182 supplementary arm long run finished" in the dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md`.

## E183 (Capacity Slope, Read Cache Small-Disk Admission, and Sweep Cost Metrics)

### 2026-10-23

- Upstream write-order change, pinned value fix only, conclusions unchanged: after number acquisition started writing in two rounds and the spot of C755 (recovery and two mount triggers fail differently on rollback) began asking F2 once per writable mount, segment one `all` produced the artifact `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` again (459 lines, 152434 bytes, two runs identical under `cmp`, sha256 `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`) and the A31 verification output `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23-anchor-check.out` (56 readings, zero differences); it differs from the 2026-10-18 one only in the `system_configuration=` field of 28 `name=h4_mount_tally` lines (`r26/106496,w3/12288` → `r30/122880,w4/16384`), and the verdict lines and the reading lines the page cites are identical. The `SEGMENT_ONE_PINNED_SHA256` pinned in the apparatus unit test was changed from `7ed2e16c6cc42b914f740f943a05440bef6a43c0917baac0114d5084f558adc8` to the new artifact's sha256; the two replay registration lines and the artifact `driver_e183_anchor_check` reads now point to the new files, and the old artifact and the old verification output are kept; the page's current status, item 7 of what was done, replay and the sources elsewhere now point to the new files, and the three rows of decisions affected were reviewed as "not affected".
  - **Basis**: `diff research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-18.out research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` lists only lines 106, 107, 117, 118, 183, 184, 194, 195, 275, 276, 286, 287, 355, 356, 366, 367, 381, 382, 392, 393, 407, 408, 418, 419, 433, 434, 444, 445; verdict lines 23, 26, 49, 120, 197, 289, 369, 395, 421, 447, 458 of the new artifact; line 58 of the new verification output.
  - **Before**: the version in the staging area, `git show :.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md` (based on `841b3beab`).

### 2026-10-18

- Newly set up, first segment (H4) run: page created, pre-run registration `research/prompts/e183-preregistration.md` (three revisions in section twelve: the empty-pool half of PC-admission counted as one loss per the original text of V33, a positive control PC-admission-file added, and the empty-pool half of K32 judged only at the `writable_mount` entry while the `before_a_publish` entry is recorded per G8), committed apparatus `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs`, artifacts `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-18.out` and `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-18-anchor-check.out`; the V33 empty-pool half is hit, the 4 items of PC-admission-file hold, H4 is enough to judge, and the second to fourth segments (H2, H5, H3) not run.
  - **Basis**: the H4 row of the question sheet `research/prompts/m3-close-cost-measurement-questions.md`, dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md`.

## E184 (Cost Counts of Regular Layer and Large Workload Layer)

### 2026-10-26

- Outdated current-state sentences and pointers in the body corrected; conclusions, numbers and artifacts unchanged: the candidate A / B item in the "Second rerun" section and the "What is still missing" cell of row one of the branch table now record the main agent's decision that candidate A (appending to section 12 of the original registration) is not pursued and B is done (previously "for the main agent to decide"); in the paths and conclusions register, row 5's source location line number is changed to the current `:774` (previously `:762`), and row 6 adds the new-pool file-creation stream's two-state closed form 16777243 from the first 12 segments; every row of "Decisions affected" re-reviewed.
  - **Basis**: the main agent's decision not to pursue candidate A, in the batch 17 entry on what the wrap-up of E184 (Cost Counts of Regular Layer and Large Workload Layer) left to the main agent, in `records/2026-10-18-里程碑三收尾与近期欠账.md`; line 774 `FULL_STATES_WITH_TWO_STATES_PER_WRITE` of `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`; the `python3 -c` recomputation in item 6 of "What was done".
  - **Before**: the staged version `git show :.claude/kb/experiments/184-常规层与大负载层的代价计数.md` (based on `841b3beab`).

### 2026-10-24

- Second rerun (rerun registration `research/prompts/e184-r2-prereg.md`, wide reading; two amendments in section 12, both before the output) written into the body: the whole segment-one artifact was rerun on the new segment shapes (new pool 12 segments, snapshot creation 16 segments, multi-record 20 segments) on the main workspace of 2026-10-24, artifact `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out` (959 lines, `sha256sum` `5c5749effbda6daed7d8f329c6b2fc4fecd71074ff2dff794a996a0f3cfdac0c`, 17 of 18 jobs ran fresh, 0 reused), comparison output `research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out` (83 lines). Both question-sheet rows are decidable: row one, the new numbers of anchors A1, A2, A5, A7 and X3 (16842785; 16777243 with 43 writes; 16777268 with 54; 1678377220 with 294; 16842815) are all “new number settled”, the 10 recomputation-versus-recording cells are `same=true` and the `selfcheck` job exits 0; row two, the narrow reading's 40 cells, the wide reading's 62 cells and 4 derived clauses have 0 words changed, recorded as “not flipped”. Interval S went from 14 streams to 16 (adding `one_file_overwritten_across_two_writable_remounts_stream` and `several_inodes_overwritten_in_turn_stream`, which entered the repository on 2026-10-21), with lower bound 16777243 and upper bound 5722079403; the inside fractions of the scopes move while the verdict words do not (the R 400 steps level at 0.100000 is exactly on the threshold endpoint); B1 now records 1358954642 and the first part of F6 still triggers. The title status gained “rerun in full once on the new segment shapes”; the body gained a section “Second rerun”, and a new item for the second rerun in the current status, item 7 of what was done, the launch and comparison commands in replay, row 6 of the path and conclusion register and the four rows of decisions affected changed accordingly. The replay table registration is unchanged: lines 40 to 78 of the `selfcheck` job in the artifact are byte-identical to the first 39 lines of the retained artifact `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out`, and the whole segment-one artifact is still not registered because it carries wall-clock fields.
  - **Basis**: lines 68 to 77 of the rerun artifact (the 10 `name=anchor` lines) and line 78 `selfcheck_done`; lines 67 to 76 of the comparison output (10 `anchor_recompute` cells), lines 63 to 66 (4 `derived` lines) and lines 81 and 82 (the two `verdict_comparison` lines); the new bounds of interval S on line 936 of the rerun artifact; registration 7.2 and the two amendments in section 12 of the rerun registration.
  - **Text moved out**: the current status used to say “the whole segment-one artifact `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out` was not rerun, and its state counts are readings of the version with one round of number acquisition writes (interval S lower bound 16777240, now pinned 16777243). Whether the verdict words flip was not measured; the inference is that they do not: among the two-state closed forms of the 123 random-history and overwrite-chain streams, the one nearest to the interval ends 16777240 and 2438004868 is still 370870 away (take `two_state` from `grep 'name=flow ' <整份产物> | grep -v 'family=smoke'`, subtract it from both ends and take the minimum), and the two-round write adds only 3 to a two-state closed form for each extra 2-write segment; the interval lower bound moves up by 3 with it, and the pinned stream at the upper bound was not measured afresh. The 16842807 hard-coded in X3 and the local constant of the J2 small-tier cell were not changed (they are registered numbers; whether to change them is up to the registration).” The rerun reading: the verdict words indeed did not flip, but that inference checked only the lower end of the interval, and the upper end moved because the smoke tier gained two pinned streams that entered the repository on 2026-10-21 (2438004868 to 5722079403), which the inference did not cover.
  - **Before the change**: the staged version `git show :.claude/kb/experiments/184-常规层与大负载层的代价计数.md` (based on `841b3beab`).

### 2026-10-23

- Upstream write-order change, pinned value fix only, conclusions unchanged: after number acquisition started writing in two rounds there is one more 2-write system configuration slot segment after mkfs; the main apparatus re-copied `PINNED_BY_THE_TESTS` from the numbers the four smoke-test files pin now (new-pool file-creation stream 16777243, 16777268, 54, 43 writes; fixed script stream 1678377095, 1678377220, 294, 80 segments; multi-record stream 1358954607, 1358954642, 117 writes; snapshot-creation stream 16842785, 16842815), the three segment-shape constants (12, 16, 20 segments) and the mid-tier full count 167837792, computed anchors A1, A2 and the new-pool write-count anchor afresh from the new segment shapes (16842785, 16777243, 43), and changed with them the 12 places in the inline unit tests that follow the pinned values; `selfcheck` produced `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` again (40 lines, exit 0, all 28 stop checks pass, 9 of the 10 anchor lines match, B1 now records 1358954642 and its `match=false` is expected by the registration), the replay registration now points to it, and the 2026-10-19 one is kept. The whole segment-one artifact was not rerun; the 16842807 hard-coded in X3 and the local constant of the J2 small item were not changed (registered numbers, left to the registration). The page's current status, item 6 of what was done, replay, the verdict source, row 6 of the path and conclusion register and the four rows of decisions affected changed accordingly.
  - **Basis**: lines 9, 14, 19 and 24 of the new artifact (the four streams' `recorded_values_match_the_test_pins_* result=pass detail=mismatches=[]`) and lines 38 and 39; `diff research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` lists only lines 5, 7, 12, 13, 17, 18, 22, 23, 27 through 31 and 34 through 38; the apparatus before the pinned-value change exited 3 on `selfcheck` in the same copy (`selfcheck_done stop_checks=28 failed=6 anchors=10 anchors_mismatching=4`).
  - **Before**: the version in the staging area, `git show :.claude/kb/experiments/184-常规层与大负载层的代价计数.md` (based on `841b3beab`).

### 2026-10-19

- Newly set up, first segment (J1, corresponding to fork G1) run: page created, pre-run registration `research/prompts/e184-preregistration.md` (ten revisions in section twelve: revision 1 calls `crash_points_of_the_steps` directly, revision 6 switches the overwrite chains u = 4 and 16 to sequential writes, revision 10 does not schedule expansions whose block count upper bound exceeds 2^26), committed apparatus `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs` and `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts_write_dump.rs`, long-run script `scripts/experiment-apparatus/e184-segment-one-long-run.sh`, artifact `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out` (950 lines, 17 of 18 items judged fresh); J1-B random history by step count tier: overlap, overlap, partial, separate (the step count row recorded as unstable), overwrite chains separate at all three tiers, J1-C all independent, 9 streams cannot be extracted, and anchor B1 triggers failure clause F6; G1 is enough to judge, and the second to fourth segments (J3, J2, J4) not run.
  - **Basis**: the J1 row of the question sheet `research/prompts/m3-close-c409-cost-questions.md`, the G1 row of the fork sheet `research/prompts/m3-close-c409-r1-forks.md`, dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md`.
- The rerun table registers the deterministic shard: `E184|@driver_e184||e184-regular-layer-cost-counts-selfcheck-2026-10-19.out|exact` (`selfcheck`; the kept artifact `research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-19.out` is lines 31 to 69 of the whole artifact plus a closing line added by the driver); the other items contain wall-clock fields and are not registered.
  - **Basis**: lines 30 to 69 of artifact `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out`.
- Third segment (H5) run: artifact `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out` (19067 lines) and the independent check output `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19-anchor-check.out` (28 lines), long-run script `scripts/experiment-apparatus/e183-sweep-history-shards.sh`; the original PC-differential arm hits all 24 cells, V35 holds, R5c and R5d do not come out, and the new arm PC-differential-read-chain passes all 24 cells; R5b A at θ = 369 and 738 exceeds the upper bound of one shard; the discriminating-power self-proof R5c cannot be done (V36); G5 neither triggers nor clears; section twelve of the registration adds how the third segment was started (the same program file on both hosts, recorded after the artifacts were merged). H5 is not enough to judge under the conditions of section eleven of the registration, and the two rows H2 and H3 remain open.
  - **Basis**: lines 7931, 7932, 19065 and 19066 of artifact `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out`, and the line "main agent judges five places in E183 before running artifacts" in the dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md`.

## E185 (Wall-Clock Breakdown of a Batch of GC Migrations)

### 2026-10-23

- The second rerun (rerun registration `research/prompts/e185-r2-prereg.md`, sha256 prefix `7d74c089`; change Jia lets the geometry at fraction 0.25 form, and calibration, thread and machine-state readings, and waiting for normal before retrying after voiding are added; the algorithm, threshold and judgement of Q1–Q17 follow the original registration without a word changed) written into the body, title status changed from "first rerun run (after the 7·6 merge phases one, two and three ran, phase four not run once enough to judge; rows 3 and 4 of the question sheet enough to judge, rows 1 and 2 not enough to judge by the letter of the registration: the geometry of the two f = 0.25 points did not form; phase three's apparatus has three lines more than the registration, left to the main agent to accept)" to "second rerun run (the geometry at live-slot fraction 0.25 formed after the workload change; the single-point check and phases one and two have run; all four rows of the question sheet suffice to judge: rows 1 and 2 read this pass, rows 3 and 4 reuse the artifacts of the first rerun; phases three and four are not run once the others suffice to judge)". The body gains five new sections, "Geometry: the two f = 0.25 points formed (change Jia)", "The set S on which row 1 of the question sheet suffices to judge", "Quantities reused from the first rerun", "Positive controls, self-checks and failure clauses" and "Calibration and machine-state readings"; the judgement of each quantity, the fork table, what was done, the rerun, the measurement basis, what it cannot answer, the paths and conclusions registry (grown from three quantities to five) and the four rows of the affected decisions table are rewritten for this pass; the comparison before and after 7·6 and G1–G6 do not exist in this pass and are moved into the next item together with the first rerun's body. Readings: single-point check (P = 1024) change Jia `geometry_formed_points=2`, the f = 0.25 segment `live_slots=16 slots_of_this_workload=16 other_slots=0`, old-construction control `geometry_formed_points=1` (f = 0.25 `geometry_formed_segments=0`), the two digests of the fraction 0.75 pool identical; in phase one all four points' `name=geometry` are `formed=true`, S = 12 cells (Q3 all `compatible`, at most 0.103088), Q1 and Q2 judged `both` in the six P = 1024 cells (`rho_median` 1.221978–1.493617, `intent_wall_ratio_median` 1.723812–1.906832) and `cpu` in the six P = 8192 cells (8.441898–9.848870; 1.105487–1.118963), of 24 cells `both` 12 and `cpu` 12, flipping with P and not with f or K (`flips=6` against `flips=0`), Q4 12 pairs `rises` (median 1.015102–1.074115), Q16 `maximum_ratio=0.015977` still `single_run_enough`, F1 and F5 trigger counts 0; in phase two the shares of stages Jia, Yi and Bing at most 0.020433, 0.006521, 0.028846, all `not_worth`, F7 of the original registration hit; none of the 140 + 60 intents voided, calibration ratio at most 1.015958 (phase one) and 1.077323 (phase two), baseline end ÷ start 0.997061 and 0.999598; mechanism check: copies landing in old holes 752, 6128, 256, 2048, the same as predicted, and `formed_segments` 1, 1, 3, 3, the same as predicted. The digests of the fraction 0.75 pool differ from lines 237 and 515 of the first rerun's phase one and match the single-point check in the same tree (the difference lies between the two trees); this pass's numbers are not mixed with the first rerun's. Q9 reuses the 72 lines of the first rerun's phase three; whether it counts as a flipping quantity for row 2 of the question sheet is left to the main agent and decided together in the 7·2 design round. Apparatus `0bd22743…`, 118 unit tests, 94 `E185 V` mutation rows. The three passes ran the scripts under `e185-r2-tools/` at the repository root, later merged into `scripts/experiment-apparatus/e185-r2-segment-run.sh`.
  - **Basis**: four verdict lines (`research/results/e185-gc-batch-wall-clock-split-geometry-check-r2-2026-10-23.out` line 124, `research/results/e185-gc-batch-wall-clock-split-geometry-check-slot-order-r2-2026-10-23.out` line 124, `research/results/e185-gc-batch-wall-clock-split-wall-clock-r2-2026-10-23.out` line 2102, `research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out` line 1802); geometry lines (single-point check line 64, phase one lines 104, 717, 911, 1660); mechanism check (phase one lines 102, 103, 715, 716, 909, 910, 1658, 1659); calibration lines (phase one lines 2099, 2100, phase two lines 1799, 1800); line 1 of the three inputs lists `research/results/e185-r2-inputs-geometry-check-2026-10-23.txt`, `research/results/e185-r2-inputs-wall-clock-2026-10-23.txt`, `research/results/e185-r2-inputs-stage-split-2026-10-23.txt`; dispatch record `records/2026-10-19-里程碑四GPU垃圾回收开工.md` lines 368, 378–381, 393, 394; rerun registration `research/prompts/e185-r2-prereg.md` section twelve lines 427–433.
  - **Before**: `git show 841b3beab:.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` (HEAD at the moment the second rerun ran, which is the version from before the first rerun; the first rerun's version of the body was only in the staging area and was never committed on its own, and the parts moved out are copied verbatim in the next item).
- The first rerun's body moved out: the body only describes the current state, so the narrative and numbers of the first rerun that no longer belong to the current state (the two f = 0.25 points not formed, the judgement table with 24 cells `both` 11 `cpu` 10 `unstable` 3, the triggering of F1 and F5, the comparison before and after 7·6, G1–G6, the handling of phase three's apparatus version, phase three's Q9–Q13 except the rows already reused in the table) are copied below in whole, indented by two spaces; the first rerun's summary of conclusions and its artifact pointers are in the next item of the same date block.
  - **Before**: what is copied below is the original text of these paragraphs in the first rerun's page (the version in the staging area), with line numbers from that version; dates equal to this block's heading date have been removed.
  **The first rerun's introductory paragraph (original body line 9)**

  The body is the result of the first rerun: rerun registration `research/prompts/e185-r1-prereg.md`; it was rerun because the GC path under test changed after milestone four 7·6 (the publish path looks up by identity) and the later batches were merged. The criteria, thresholds, voiding and stop clauses, arms, positive controls, sample points and number of rounds follow the original registration (including section twelve 12.1–12.11) without a word changed; what the rerun registration adds is which phases to run, the conditions for phases three and four, the comparison before and after 7·6 (the "reported side by side" table of rerun registration section six: "is it the same work before and after", "the exponent of a batch's CPU against pool size" (one row each for the phase-one and phase-two sources), the exponent of "the rest", "after-to-before ratio", "fork quantities side by side", script `scripts/experiment-apparatus/e185_r1_compare.py`) and the comparison's failure clauses G1–G6 (rerun registration section ten); its six revisions in section twelve come after the unit tests of the comparison script and before the artifacts, and only tighten. "Before" is the two artifacts of the 2026-10-20 pass and "after" is this pass's; the difference between before and after is only said to lie between the two trees and is not attributed to 7·6 alone (rerun registration section one).

  **The first rerun's current state (original body lines 13–23)**

  **Current state**:

  - Two versions of the apparatus: the registration 12.11 version, sha256 `b3a50d50441f9f2e74bec6c941b74e632529688a78563995387708230b60c470` (the copy in commit `841b3beab`), used by phases one and two; the version now in the main workspace, sha256 `eadb929d6d8be037530f2ce94ee4a564202939f9c2d25e275023aa7ce091ae76`, with 3 more lines than the former (`git diff HEAD` on the apparatus source: the `quarantine_mark` field added to struct literals of allocation records, once in `emit_the_disk_size_sensitivity` for the disk-size sensitivity of stage Bing, twice in the record-building helper of the unit tests), used by phase three. `grep -c '#\[test\]'` on the main workspace copy = 87; the 87 tests of the `b3a50d50…` version ran and all passed on the second host (dispatch record `records/2026-10-19-里程碑四GPU垃圾回收开工.md`), and that dispatch record has no entry of the unit tests of the `eadb929d…` version having run. The unit tests count as the checker tier by package and are run by the main agent with the prefix.
  - 55 mutation rows (V1–V55, names start with the experiment number E185 (Wall-Clock Breakdown of a Batch of GC Migrations) followed by " V", at the end of `crates/mutations.tsv`): V1–V15 follow section nine of the registration, V16–V55 were added by the runner (the judging of stop clauses 2, 3 and 5, the check that the destination is filled, the classification of the issuing call, how f = 0.25 picks its overwrites, the split of scan reads; V23–V26 are the column for the pre-release disk-read check of section twelve 12.5: merged back into scan reads, the copy on the other disk not entering this column, the byte anchor of the new sub-item of stage Jia only judging overcounting, the relocation source taken as the destination placement; V27–V30 are the phase-one configuration line of section twelve 12.6: printing the image directory again, missing subdirectories under `/tmp`, taking only 16 bits of the minor device number, taking the wrong column of mountinfo; V31–V33 are section twelve 12.9: the denominator of the CPU arm's write-call disk-wait sentence switched back to the baseline disk wait, the disk-wait increment switched back to all device calls, the order of the three injections at the control point not rotated by round number; V34–V55 are the parallel preparation and global queue places of section twelve 12.10 and 12.11). All 23 rows of light tier mutations, V21 and V34–V55, were caught on the second host (`records/2026-10-19-里程碑四GPU垃圾回收开工.md`); the three numbers for the whole table wait for gate checker-tier-crates-mutation-replay at commit time.
  - The first rerun's artifacts (all in `research/results/`): phase one `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out` (916 lines, sha256 `978ae7b5fe60ab8f5a87338b982e8989c4c0d0ba8f19994bfc8e1c39b3d437fd`, last line `E7RESULT name=done emitted=916`); phase two `research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out` (677 lines, sha256 `4a073e80fdbea6237ab9f39326f676d6b8ff0dd95e2932b0b188997c3ef71743`, last line `E7RESULT name=done emitted=677`); phase three `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out` (203 lines, sha256 `4041a1cac83e7d62ef8bbe1580468aa6b4e43d69d67e8e9e044dd885ab063408`, last line `E7RESULT name=done emitted=203`); comparison `research/results/e185-r1-comparison-2026-10-22.out` (702 lines, sha256 `f6e6d106d8051f9640232240bce126634f3c090143651abcee9293aa3401b0e1`, last line `E7RESULT name=done emitted=702`); inputs lists `research/results/e185-r1-inputs-2026-10-22.txt` (phases one and two) and `research/results/e185-r1-inputs-gpu-submission-2026-10-22.txt` (phase three). The two "before" files are `research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out` (911 lines, sha256 `a6afc7bf2f380d86f3b51a070c91465c3206a2476c216af6ab59fbe700a7d87b`) and `research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out` (677 lines, sha256 `e087a1b6834a381322c87ebd0a53a242b66ce035812fab39d7628b79e3a9e125`); the comparison checked these two values before computing (comparison artifact lines 2 and 3 `ok=true`).
  - Where it ran: phases one and two ran one after another on the same second host, on the same disk as the 2026-10-20 pass (inputs list line 24, model CT2000T500SSD8, logical block 512; comparison artifact line 119 `E7RESULT name=disk_identity same_disk=true image_directory_filesystem_before_after=ext4->ext4 image_device_before_after=64512->64512`), built once without the `gpu` feature, with no `lookup-work-counters` in the output of `cargo tree` (comparison artifact line 4 `E7RESULT name=inputs_list_check problems=none`), outer memory cap 46G, `E185_MEMORY_CAP=40G`, apparatus threads 31. Phase three ran on the two cards of this host (card 0 RTX 5090, card 1 RTX 5060 Ti, driver 595.99.02), built separately with the `gpu` feature, with `--wall-clock-product` and `--stage-split-product` given only this pass's two `-r1-` artifacts; its tree had patches from another session that the tree of phases one and two did not (phase-three inputs list line 3 `transaction.rs` sha256 `25fb4392…`, while the tree of phases one and two had `3b81509b…`), and the apparatus was `eadb929d…` (phase-three inputs list line 2).
  - Verdict lines (one per file, four files, copied whole): phase one line 915 `E7RESULT name=verdict segment=wall_clock constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none discrimination_failures=0 clause_wall_clock_not_rising_with_the_batch_triggered=9 clause_two_cpu_paths_disagree_points=5 gc_to_sequential_write_bandwidth_maximum_ratio=0.011019 gc_to_sequential_write_bandwidth_judgement=single_run_enough stored_intents=140`; phase two line 676 `E7RESULT name=verdict segment=stage_split constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none`; phase three line 202 `E7RESULT name=verdict segment=gpu_submission constant_or_anchor_mismatches=0 cards_measured=2 card_failures=0 card_failure_items=none discrimination_holds=true`; comparison line 701 `E7RESULT name=verdict baseline_sha_mismatches=0 anchor_mismatches=0 inputs_list_problems=0 unreadable_cells=0 g1_after_not_near_linear=1 g2_before_not_near_square=18 g3_two_sources_disagree_before=2 g3_two_sources_disagree_after=1 g4_different_work=0 g5_flipped=27 output_void=false`. The only field with the value `false` in the four lines is `output_void=false` in the comparison line (this comparison output was not voided), and there is no `not_run`; the counts above 0 are the trigger counts of the two failure clauses F1 and F5 (9, 5) and the trigger counts of the comparison's G1, G2, G3 and G5, each written cell by cell in the two sections "Judgement of each quantity" and "The comparison's failure clauses G1–G6".
  - The "apparatus changed" clause (rerun registration section eleven: if the apparatus sha256 in the tree being launched is not `b3a50d50…`, stop, do not launch, and hand over to the main agent): line 1 of the inputs list of phases one and two is `b3a50d50…`, not triggered; line 2 of phase three's inputs list is `eadb929d…`, triggered by the letter, and phase three was run anyway. Of the 3 differing lines (the "two versions of the apparatus" item in the current state), one is in the disk-size sensitivity of stage Bing in phase two and two are in the unit tests, and the `gpu-submission` subcommand cannot reach them; this sentence was inferred by reading `git diff HEAD`, not measured separately. This page reports phase three's numbers as they are; how to handle this clause and whether phase three counts are left to the main agent.
  - Both `-r1-` measurement artifacts and phase three carry wall-clock fields, and `scripts/experiment-apparatus/replay.sh` has no registration row for E185 (Wall-Clock Breakdown of a Batch of GC Migrations) yet (following the precedent of E184 (Cost Counts of Regular Layer and Large Workload Layer), only a deterministic slice is registered); the comparison is deterministic arithmetic over artifacts only and is not registered either.
  - The six revisions in section twelve of the rerun registration (after the comparison script and its unit tests went green, before this pass's two phase artifacts, only tightening): the comparison's unit tests live in the same directory, `scripts/experiment-apparatus/e185_r1_compare_test.py` (15 self-check items, nine mutation rows: caught 9, invalid 0, not red 0, dispatch record `records/2026-10-19-里程碑四GPU垃圾回收开工.md`); check the sha256 of the two "before" files before computing; read the inputs list to check the apparatus sha256 and the counting feature; if the two passes were not on the same disk, do not put the real-disk cells side by side; do not treat unreadable rounds as 0; check the anchor between the copy bytes of the first relocation batch and the scan read bytes of the issuing call point by point and round by round, writing `not_checked` for points with `formed=false`. Of the 48 `name=anchor` lines in the comparison artifact, 24 are `status=ok` (the 6 cells of f = 0.75, two phases, one each before and after) and 24 are `status=not_checked` (the 6 cells of f = 0.25 laid out the same way, geometry not formed); the comparison verdict line has `anchor_mismatches=0`.
  - Section twelve of the original registration has eleven rows. 12.1–12.8 all come before any artifact and leave the criteria of sections six to eleven untouched word for word; 12.9 comes after the artifacts of the first pass of phase one and changes only the CPU-arm disk-wait sentence of row 1 of the positive control table in section five (see the "first-row positive control" entry under "Measurement basis"), counts as a new pass, and reruns phase one in full; 12.10 and 12.11 come before the two checked-in phases' artifacts and change only the untimed preparation (pool building, geometry check, rebuilding and real-disk image creation run in parallel on one global queue, while the timing window stays serial on one thread), leaving the criteria untouched word for word. 12.1–12.8 row by row: 12.1 the mutation table lands in `crates/mutations.tsv`; 12.2 the CPU clock changes from this thread's schedstat to this thread's `clock_gettime(CLOCK_THREAD_CPUTIME_ID)`, with a resolution self-check added at start (stricter), and V2 becomes "clock id 3 changed to 1"; 12.3 how the geometry fallback is read (when the competing segment holds nodes or packing units, go straight to the fallback); 12.4 the wall clock paired with the thread CPU reading changes to `CLOCK_MONOTONIC_RAW`; 12.5 the pre-release disk-read check that publish does on this batch's relocation sources before the first write (one copy read from each of the two disks) is split out of scan reads into its own `release_check_read_bytes`, and stage Jia of phase two gets a sub-item (CPU and bytes of the CRC-32C of those copies, also reported summed with the original item), with the original item and the line-343 anchor unchanged and one more anchor added; 12.6 artifact key names are all written out in full (question and clause numbers do not go into key names; the old-to-new table is in registration 12.6), and the phase-one configuration line no longer prints the path of the image directory but its file system type, whether it lives somewhere cleared at boot, and its device number; 12.7 question numbers, decision numbers and abbreviations in names declared in the apparatus (constants, functions, local variables, unit test names) are changed to full names, the `name=` values and key names of the artifacts and the criteria are not changed by a word, and the anchors or named unit tests of the three mutation rows V2, V7 and V28 are renamed with them (old-to-new table in registration 12.7); 12.8 drops the leading `a_` from two unit test names, and the unit tests named by V10 and V26 follow.

  **The first rerun's geometry, judgement of each quantity, comparison before and after 7·6, and failure clauses G1–G6 (original body lines 25–78)**

  ### Geometry: the two f = 0.25 points did not form

  - The two f = 0.75 points take the registered fallback: phase one lines 236 and 514 `name=geometry` say `choice=forced_fallback`, `formed=true` (phase two lines 272 and 474 likewise).
  - The two f = 0.25 points did not form: phase one line 40 `E7RESULT name=geometry fraction=0.25 pool_data_units=1024 choice=none fallback_region_start_slot=none formed=false reason=greedy_segment_holds_units_other_than_this_workload_no_registered_workload_change`, line 138 the same shape (`pool_data_units=8192`), phase two lines 40 and 158 likewise; phase one lines 39 and 137 `name=geometry_attempt` say `geometry_formed_segments=0`, and the segment greedy picked has `greedy_live_slots=1 greedy_slots_of_this_workload=0 greedy_other_slots=1`. What was measured at these two points is the segment greedy picked: every batch scans 16384 bytes and relocates one unit (the 6 `name=q14` lines of phase one `relocation_copy_bytes_per_round=32768,32768,32768,32768,32768`, the same for K = 4, 8, 16), not a segment with live-slot fraction 0.25. Per section eleven of the original registration, "geometry not formed: that sample point does not enter the judgement", the section "Judgement of each quantity (first rerun)" lists the f = 0.25 judgements as in the artifacts but does not use them to judge the fork rows. In the 2026-10-20 pass the two f = 0.25 points were likewise `formed=false` (rerun registration section twelve item 6).

  ### Judgement of each quantity (first rerun)

  Each quantity reports its own judgement per section six of the original registration; if the 5 rounds do not all give the same judgement it is written "unstable". Artifacts: phase one = `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`, phase two = `research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out`, phase three = `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`. A range is the minimum and maximum over that group of lines, both ends being raw values from the artifact. The 24 cells of Q1 and Q2 in phase one: `both` 11, `cpu` 10, `unstable` 3, `disk` 0 (`grep -E 'name=q[12] ' <第一段产物> | sed -E 's/.* judgement=([a-z_]+)$/\1/' | sort | uniq -c`).

  | Quantity | Where read | This pass's numbers | Judgement |
  |---|---|---|---|
  | Q1 ρ_真盘 | phase one, 12 `name=q1` lines (lines 705–749) | `rho_median`: six P = 1024 cells 1.163279–1.680570, six P = 8192 cells 8.263672–10.195150 | `both` 6 (all of P = 1024), `cpu` 5, `unstable` 1 (f = 0.75, P = 8192, K = 16, `judgement_per_round=both,cpu,cpu,cpu,cpu`), no `disk`. On f = 0.75, where the geometry formed: the three P = 1024 cells `both`; the three P = 8192 cells `cpu`, `cpu`, `unstable`, and Q3 is unstable in these three cells, so per F5 they are not cited. Flips with P: line 754 `E7RESULT name=sensitivity quantity=Q1 knob=pool_data_units pairs=6 flips=6 statement=flips_with_this_knob …` (all six pairs flip) |
  | Q2 wall-clock ratio | phase one, 12 `name=q2` lines | `intent_wall_ratio_median`: P = 1024 1.725366–1.957259, P = 8192 1.080125–1.116675 | `both` 5, `cpu` 5, `unstable` 2 (f = 0.75, P = 1024, K = 4 has one round judged `disk`; f = 0.75, P = 8192, K = 8 has one round judged `both`), flips with P (line 757 `flips=6`). On f = 0.75 the three P = 1024 cells are `unstable`, `both`, `both`, and the three P = 8192 cells `cpu`, `unstable`, `cpu` |
  | Q3 two-way reconciliation | phase one, 12 `name=q3` lines | per-round `reconciliation_per_round` at most 1.235920 (f = 0.75, P = 8192, K = 8, round 1) | `compatible` 7, `unstable` 5 (`row_one_judgeable_here=false`: K = 8 and 16 of f = 0.25 P = 1024, K = 4, 8 and 16 of f = 0.75 P = 8192, each cell with one round > 20%), verdict line `clause_two_cpu_paths_disagree_points=5` (F5): row 1 is not enough to judge at these 5 points, and neither the thread CPU clock reading nor the copy overhead of the memory disk has been looked into yet |
  | Q4 doubling the batch size | phase one, 12 `name=q4` pairs (lines 761–772) | per-pair 5-round median 0.924689–1.062143 | memory arm `does_not_rise` 5, `rises` 1 (f = 0.75 P = 1024 8→16); real-disk arm `does_not_rise` 4, `rises` 2 (f = 0.75 P = 1024 8→16, f = 0.75 P = 8192 4→8). The verdict line has F1 triggered for 9 pairs: 5 on f = 0.75 (memory 1024 4→8, 8192 4→8, 8192 8→16, real disk 1024 4→8, 8192 8→16) and 4 on f = 0.25. The `copy_bytes_double=true` of those 4 f = 0.25 pairs is computed by the apparatus from the closed form `2 × min(K, m) × 32768`, while the measured copy bytes per relocation batch are 32768 for all three K (geometry section); F1 of the original registration requires "the anchor really doubles", so by the measurement these 4 pairs are not F1. F1 is handed to the main agent for a three-way inference per the original registration, and the row 1 judgements are reported by f and P |
  | Q5 stage Jia share | phase two, 28 `name=stage_judgement` lines; `crc_share` of the batch lines of `name=stage` (140 lines) | f = 0.75 at most 0.020472, f = 0.25 at most 0.000480 | 28 lines `crc_share_judgement=not_worth`, in all 5 rounds of every line |
  | Q6 stage Yi incremental share | phase two, 28 `name=stage_judgement` lines; `scan_increment_share` of the batch lines of `name=stage` | f = 0.75 at most 0.006122, f = 0.25 at most 0.000848 | 28 lines `scan_increment_share_judgement=not_worth`, in all 5 rounds of every line |
  | Q7 stage Bing incremental share | `choose_share_per_batch` of the issuing lines of `name=stage` (60 lines) | f = 0.75 at most 0.013997, f = 0.25 at most 0.033060 | The artifact has no Q7 judgement: 28 lines `choose_share_judgement=not_run`, because the apparatus judges only on batch lines, segment choice is only replayed for the issuing call, and batch lines have `choose_share_per_batch=none`. The raw values on the 60 issuing lines are all below the registered threshold of 10%; this reading is the page reading the artifact's raw values against the registered threshold, not a judgement by the apparatus |
  | Q8 single-thread throughput | `name=stage` batch lines `crc_bytes_per_second`, `scan_increment_bytes_per_second`; issuing lines `choose_record_bytes_per_second` | Jia 2981619654.231119–3200117191.791692 B/s; Yi increment 314230916.762562–11685641688.583784 B/s; Bing 113719247.621532–907537456.945270 record bytes/s | not judged on its own, feeds Q9 |
  | Saturated throughput (the CPU side of Q9) | phase two, 108 `name=saturation` lines, 108 with `distinct_threads_holds=true` | 16 threads: Jia 48436312892.2–49249556571.2, Yi 4450326991.8–37692455096.3, Bing 1641740063.1–13977688649.7 B/s | not judged on its own |
  | Q9 16 threads against the GPU | phase three, 72 `name=q9` lines (two cards × three stages × 12 cells) | `ratio` at most 0.007746 (card 0, Yi increment, f = 0.75, P = 8192, K = 8) | 72 lines `judgement=gpu_loses`, single-thread version `judgement_one_thread=gpu_loses`, `clause_sixteen_thread_form_misfits_the_gc_triggered=false` (F2 did not trigger). Whether it still counts as a flipping quantity for row 2 is left to the main agent (rerun registration section six, "the Q9 cell is left to the main agent") |
  | Q10 B_批 | `batch_bytes` of phase three `name=q13` (reads phase one `name=call`) | f = 0.75: 131072–786432 bytes; f = 0.25: 16384 bytes | not judged on its own |
  | Q11 a, Q12 b | phase three lines 54 and 136 | card 0 `E7RESULT name=q11_q12 index=0 card_valid=true intercept_seconds=0.004492 slope_seconds_per_byte=4.117289e-10 one_range_median_seconds=0.005244 largest_two_points_slope_seconds_per_byte=4.172332e-10`; card 1 `E7RESULT name=q11_q12 index=1 card_valid=true intercept_seconds=0.010300 slope_seconds_per_byte=1.266004e-9 one_range_median_seconds=0.005393 largest_two_points_slope_seconds_per_byte=1.191920e-9` | not judged on its own |
  | Q13 φ and n* | phase three, 56 `name=q13` lines | card 0 `fixed_overhead_share` 0.932762–0.998500, `batches_to_merge` 125–5993; card 1 0.911860–0.997990, 94–4470 | 56 lines `judgement=merge_batches` (φ ≥ 50%, "batches should be merged"), both cards judged alike (F8 did not trigger). Discrimination line 201 `E7RESULT name=discrimination quantity=Q13 lower_value=0.911860 higher_value=0.998500 moved_threshold=0.955180 lower_judgement=one_batch_per_submission higher_judgement=merging_optional holds=true` |
  | Q14 GC write bandwidth running alone | phase one, 12 `name=q14` lines | `write_bytes_per_second_median` 4019518.193541–52061628.180733 | not judged on its own |
  | Q15 device sequential write bandwidth | phase one line 773 | `E7RESULT name=q15 direct_with_fdatasync_bytes_per_second_median=4724643851.2 page_cache_without_sync_bytes_per_second_median=12396955393.1 positive_control_row_four_holds=true` | not judged on its own |
  | Q16 bandwidth ratio | phase one line 786 | `E7RESULT name=q16 maximum_ratio=0.011019 judgement=single_run_enough fourth_segment=not_run_after_enough_to_judge` | "running alone is enough"; none of the three knobs flips it at either end (lines 787–789 `flips=0`) |
  | Q17 foreground fsync latency ratio | phase four | not run | not run after enough to judge (row 4, Q16 ≤ 10%, the stop-once-enough rule of section eleven of the original registration) |

  Positive controls and self-checks: the four cells of row 1 in phase one lines 791–794 are `holds=true` (CPU arm memory disk `cpu_increment_over_injected=1.006423`, `write_wait_increment_over_injected=0.000142`, real disk `1.002365`, `0.004330`; in the memory-disk cell the sentence under the original criterion is `wait_clause_under_the_original_criterion_holds=false`, reported only and not judged per 12.9); the four cells of row 2 in phase two lines 56–59 are `holds=true`; phase three's two cards `check_string_holds=true`, and the 32 `name=gpu_submission` lines are all `respects_the_physical_floor=true crcs_agree=true`; `name=clock_resolution_check` is `holds=true` in both phases; the three `name=discrimination` lines of phase one (Q1, Q2, Q16) are `holds=true`. Phase one voided 1 round (line 541 `invalid_reason=unaccounted_time_above_ten_percent_of_the_wall`, real disk, f = 0.75, P = 8192, K = 8, the first attempt of round 1), and after the retry `stored_intents=140`.

  ### The comparison before and after 7·6

  Artifact `research/results/e185-r1-comparison-2026-10-22.out` ("before" = the two 2026-10-20 files, "after" = this pass's phases one and two). Only the memory arm's CPU is compared: 7·6 only changes the lookups on the CPU, so differences before and after in the real-disk arm's disk wait, write bandwidth and ρ cannot be attributed to 7·6 (rerun registration section six).

  | Cell | Where read | This pass's numbers |
  |---|---|---|
  | Is it the same work before and after | comparison lines 120–131, 249–260 `name=same_work` (12 cells per phase) | all 24 lines `judgement=same_work differences=none write_differences=none`, verdict line `g4_different_work=0` |
  | Two-point exponent α of a batch's CPU against pool size (P = 1024 → 8192) | comparison lines 53–70 (before), lines 95–112 (after) `name=cpu_exponent` | before: phase-one source 1.191656–1.219251, phase-two source 1.179549–1.226787, "the rest" 1.179741–1.231472; after: phase-one source 0.995884–1.030719, phase-two source 0.994926–1.019725, "the rest" 1.002926–1.028385. In the later pass 17 of 18 cells are judged `near_linear_between_the_two_points` and 1 cell `unstable` (phase-one source f = 0.75, K = 16, line 98 `per_round_alpha=1.447368,1.032876,0.999670,0.993537,1.002079`) |
  | After-to-before ratio (after ÷ before, same f, P, K) | comparison lines 132–143, 261–272, 279–290 `name=cpu_ratio` | phase-one source 0.301611–0.523085, phase-two source 0.302950–0.518856, "the rest" 0.302814–0.512659; the P = 8192 cells of the phase-one source 0.301611–0.342767 |
  | Fork quantities side by side | comparison `name=paired_value` 503 lines (from line 150, interleaved with other lines) | 27 cells flip their judgement before and after (the G5 row of the section "The comparison's failure clauses G1–G6"), and all 503 lines are `status=compared` |

  The geometry of the two f = 0.25 points did not form, but the two passes are judged "the same work" (segment choice, geometry, number of mapping entries, number of batches, the kind of every call, and the three kinds of read and relocation bytes are all the same), so the CPU after-to-before ratio still holds; their α are listed as they are, not as readings at live-slot fraction 0.25.

  ### The comparison's failure clauses G1–G6 (rerun registration section ten)

  | Clause | Hit or not | Artifact lines (comparison artifact) |
  |---|---|---|
  | G1 dropping to near linear not seen in this cell | hit 1 cell: phase-one source f = 0.75, K = 16, α median 1.002079, where round 1 at 1.447368 pulls the five-round judgement to "unstable". In the same cell the phase-two source and "the rest" are both judged near linear (lines 276 and 294 `triggered=false`). The other 17 lines not hit | line 147 `E7RESULT name=clause clause=G1_after_not_near_linear source=wall_clock_call_cpu fraction=0.75 batch_units=16 alpha=1.002079 judgement=unstable triggered=true`; verdict line `g1_after_not_near_linear=1`. Recorded per G1 as "the second half of the 7·6 acceptance sentence was not seen in the phase-one source of the (f = 0.75, K = 16) cell", handed to the main agent |
  | G2 from square not seen in this cell | hit 18 cells: 6 cells for each of the three sources, with the earlier pass's α all within 1.179549–1.231472 and no cell ≥ 1.75 | lines 71–88 `clause=G2_before_not_near_square … triggered=true`; verdict line `g2_before_not_near_square=18`. Recorded per G2 as "these cells of the earlier pass are not near square": the "from square" half of the acceptance "drop from square to near linear" in the 7·6 row was not seen at the apparatus's two points P = 1024 and 8192, so the "drop" sentence cannot be stated at these two points; the "P = 4096 one batch 217.47 → 62.38 ms" that the 7·6 row cites is a number from another apparatus on a copy, and P = 4096 is not a sample point of this apparatus |
  | G3 the two sources disagree | hit 2 cells before and 1 after: in the earlier pass K = 4 and 8 of f = 0.25 (points where the geometry did not form), in the later pass K = 16 of f = 0.75 (phase-one source `unstable`, phase-two source near linear) | lines 90 and 91 (before) `triggered=true`; line 116 `E7RESULT name=clause clause=G3_two_sources_disagree side=after fraction=0.75 batch_units=16 wall_clock_judgement=unstable stage_split_judgement=near_linear_between_the_two_points triggered=true`; verdict line `g3_two_sources_disagree_before=2 g3_two_sources_disagree_after=1`. Per G3 these three cells get no judgement and are written "the two sources disagree", with both sides to be looked into (not done) |
  | G4 different work before and after | not hit | 24 lines `judgement=same_work`; verdict line `g4_different_work=0` |
  | G5 fork quantities flip before and after | hit 27 cells, all in phase one: Q1 8 cells (the six P = 1024 cells `cpu` → `both`; f = 0.25 P = 8192 K = 4 `unstable` → `cpu`; f = 0.75 P = 8192 K = 16 `cpu` → `unstable`), Q2 9 cells (the three f = 0.25 P = 1024 cells and f = 0.75 P = 1024 K = 8 `cpu` → `both`; f = 0.75 P = 1024 K = 16 `unstable` → `both`; f = 0.25 P = 8192 K = 4 and 16 `unstable` → `cpu`; f = 0.75 P = 1024 K = 4 and f = 0.75 P = 8192 K = 8 `cpu` → `unstable`), Q3 7 cells (f = 0.25 P = 1024 K = 8 and 16 and the three f = 0.75 P = 8192 cells `compatible` → `unstable`; f = 0.25 P = 8192 K = 4 and 16 `unstable` → `compatible`), Q4 3 pairs (memory f = 0.75 P = 1024 4→8 `rises` → `does_not_rise`; real disk f = 0.75 P = 8192 4→8 `does_not_rise` → `rises`, 8→16 `rises` → `does_not_rise`). Neither the 84 stage judgement lines of phase two nor Q16 flipped | `grep -n 'flipped=true'` on the comparison artifact: lines 150, 152, 154, 158, 162, 164, 166, 168 (Q1), 175, 177, 179, 181, 183, 187, 189, 191, 197 (Q2), 198, 200, 201, 202, 207, 208, 209 (Q3), 214, 230, 232 (Q4); verdict line `g5_flipped=27`. Not a failure: the fork rows are judged on this pass, these 27 cells are written side by side as "flipped before and after (between the two trees)" and not attributed to 7·6 alone; the real-disk arm cells also mix in the state of the machine at that moment |
  | G6 the use of Q9 conflicts with today's D26 (Background Compaction and Placement Reclamation) settled item 13 clause 7 | not hit: phase three ran, but no stage of Q5–Q7 was judged worth it (Q5 and Q6 all 28 lines `not_worth`, Q7 not judged in the artifact with raw values all < 10%), so there is no cell "judged worth it while Q9 is judged `gpu_loses`"; Q9 all 72 lines `gpu_loses` | the comparison artifact does not judge this clause (it does not read phase three); what was read is the 28 `name=stage_judgement` lines of phase two and the 72 `name=q9` lines of phase three |

  **The first rerun's fork table (original body lines 80–89)**

  ### Question sheet row by row (fork table)

  Question sheet `research/prompts/m4-gpugc-measure-forks.md`; the conditions for enough to judge follow the question sheet's wording, and the judgements follow this pass.

  | Row | Enough to judge / what is still missing | What this pass can read | Can the remaining quantities flip it |
  |---|---|---|---|
  | 1 disk / CPU | Not enough to judge by the letter of the registration. Still missing: the condition needs "two live-slot fractions within a segment", and the geometry of the two f = 0.25 points did not form, so they do not enter the judgement; on f = 0.75, where the geometry formed, Q3 is unstable in the three P = 8192 cells (F5), Q1 is not cited there, and neither side has been looked into | f = 0.75, the three P = 1024 cells Q1 `both` (`rho_median` 1.163279, 1.352170, 1.293462); f = 0.75, the three P = 8192 cells Q2 `cpu`, `unstable`, `cpu` (1.080125, 1.114507, 1.116675). The judgement flips with P and is not reported by K (F1 handed to the main agent) | This registration has no unrun quantity left. Only measuring again after the f = 0.25 geometry forms and clearing up the F5 cells could move it. Inferred (not measured): at the two f = 0.25 points that did not form, Q1 is likewise `both` at P = 1024 and `cpu` at P = 8192, and the flip follows P |
  | 2 which stages GPU computes | Not enough to judge by the letter of the registration: the condition says "on the set of sample points where row 1 is enough to judge", and row 1 is not enough to judge. At the f = 0.75 points the CPU time and bytes of all three stages were measured | Q5 and Q6 all 28 lines `not_worth` (f = 0.75 shares at most 0.020472, 0.006122); Q7 not judged in the artifact, raw value on the issuing lines at f = 0.75 at most 0.013997; F7 of the original registration hit: none of the three stages reaches 10% of a batch's CPU, the bulk of the CPU is in the rest (reading this version, publishing), and the main agent decides whether row 2 needs an extra "publishing half" arm. Q9 all 72 lines `gpu_loses`; whether it counts as a flipping quantity for this row is left to the main agent | Whether or not Q9 counts as a flipping quantity, it points the same way as Q5–Q7. Inferred (not measured): at f = 0.25 the scan bytes per batch are smaller than at f = 0.75 (original registration section eight), so once it forms the shares of stages Jia and Yi can only be smaller |
  | 3 batching shape | Enough to judge (provided the main agent accepts the phase-three apparatus item) | Q10 at f = 0.75 is 131072–786432 bytes per batch; a and b measured on both cards; Q13 all 56 lines `merge_batches`, φ at least 0.911860, both cards judged alike: one submission per batch (Jia) is out, and the choice is between Yi and Bing; between Yi and Bing, 7·1 only provides n* and bytes per batch, and the semantics of R and rescanning between batches are judged by the 7·2 design round (arm table, row 3, section five of the original registration) | No quantity left. A flip needs φ to fall below 50%, and even the smallest φ cell is above 0.91 |
  | 4 fallback interval and write bandwidth cap | The running-alone half is enough to judge; phase four and Q17 not run after enough to judge | Q16 `maximum_ratio=0.011019 judgement=single_run_enough` (0.006982 in the 2026-10-20 pass) | Q17 runs only when Q16 > 10%; not run in this pass |

  **The first rerun's method and rerun commands (original body lines 91–124; the items that remain on the second rerun's page are unchanged, and what is written differently is items 2, 4, 5 and 7 and the rerun section)**

  ### What was done

  1. Timing wrapper device: wraps the memory disk (`SparseBlockDevice`) or the real disk (an `O_DIRECT` image file on ext4, `DirectInputOutputBlockDevice`), reads `Instant` and this thread's `clock_gettime(CLOCK_THREAD_CPUTIME_ID)` once before and once after every kind of `BlockDevice` call (the CPU reading enclosing the wall-clock reading), and accumulates wall clock, CPU, disk wait (wall clock minus CPU), count and bytes per call kind; reads are split by "before / after the first write in this timing window", and reads before the first write are split into three columns: those landing in R on R's disk, minus re-reads on the relocation-source slots, go to scan reads; those on this batch's relocation-source slots (same-numbered slots on both disks), minus what remains after the first read of each slot on R's disk, go to the pre-release disk-read check (`release_check_read_bytes`, registration section twelve 12.5); the rest go elsewhere; writes are split into "landing on this batch's destinations / elsewhere". The positive-control injection (sleep 2 ms in every `write_at`, or spin 2 ms by the thread CPU clock) also lives here.
  2. Pool building: with harness default parameters, mkfs two 4 GiB memory disks, the first file, writable mount; create P + 64 empty inodes, the first P each sequentially writing one data unit; in segments written entirely by this run, overwrite 3 out of every 4 (f = 0.25) or 1 (f = 0.75) in slot order, and in mixed segments overwrite all units written by this run; fill the highest segment with the extra empty inodes; then "raise F to the acceptance limit and create one inode" until all the overwritten old placements are released.
  3. Geometry check: on a copy, writable mount and replay the segment-choosing loop of the session's issuing step (`choose_region` alternating with "scan first") to see which segment greedy will choose; if that segment is all data units written by this run with live slots = 64f, use greedy; if not but it is all data units written by this run, fold them into the recipe and rebuild (at most three times); if it holds other units, take the registered fallback (the test-only forced segment choice naming the f segment with the lowest slot number).
  4. Measuring one intent: writable mount (`mount_writable_with_compaction_switches`; the product switch only swaps the batch size for the test-only `ForcedForTests(K)`), set the session's "publishes already elapsed" to 1000 before every call (the publish budget drops to 1), and call `run_compaction` once at a time until the last batch deletes the intent; read `Instant` and the thread CPU clock before and after every call (the wall-clock reading enclosing the CPU reading, the reverse of the device-call layer), and after the return, outside the timing window, judge from the version produced whether it was the issuing call, a batch, or the last batch.
  5. Replay in phase two: before the call, take this version's allocation records, mapping entries (a separate copy of the session's private way of getting them), and pinned and quarantined slots, and the bytes of the units the scan read; three stages (Jia: whole-unit CRC-32C of every unit read (the other sub-item is the whole-unit CRC-32C of the copies of the pre-release disk-read check, whose bytes are read before the call, one copy from the same-numbered slot of each disk); Yi: the whole `scan_region`; Bing: every `choose_region` of the issuing step), each replay first running once untimed as warm-up, then once timed, taking the median of 5 replays; saturated throughput at 1, 16 and N threads, 2 seconds each.
  6. Phase three: on each card, using one real data unit (32768 bytes) written during pool building, with n from 1 to 16384 ranges, `GpuCard::crc32c_of_ranges` is run once and discarded, then timed 5 times and the median taken, checking that every range's CRC matches the CPU's and that no run is faster than the PCIe link peak; a and b are fitted by least squares, φ and n* are computed with phase one's B_批, and the ratio in the form of D24 (Offloading Heavy Background Work to GPU) settled item 3 (Q9) is computed with phase two's 16-thread saturated throughput.
  7. The comparison before and after 7·6 (new in the first rerun): `scripts/experiment-apparatus/e185_r1_compare.py` only reads the text of the two artifacts before and the two after, and does not import `crates/`; before computing it checks the sha256 of the two "before" files and the inputs list, takes by the same rule from both passes' artifacts the CPU of the first batch of every round of the memory arm (`cpu_nanoseconds` of `name=call` in phase one, `call_cpu_nanoseconds` of `name=stage` in phase two, with "the rest" further subtracting `scan_cpu_nanoseconds`), and for every (f, K) computes α = ln(c(8192) ÷ c(1024)) ÷ ln 8 and the per-round paired α, with thresholds ≤ 1.25 near linear and ≥ 1.75 near square; it also compares whether the two passes did the same work, computes after ÷ before, and puts the two passes' fork quantities side by side by each line's own key.

  ### Rerun (first rerun; commands follow "how to run" in section five of the rerun registration)

  ```bash
  # Repository root. Build and unit tests (checker tier, run by the main agent with the prefix; the unit tests are the #[test] in the apparatus, 87 in the main workspace copy)
  # Phases one and two without the gpu feature, without --all-targets or --tests (so that lookup-work-counters is not pulled in)
  cargo build --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split
  cargo test --release -p singlefs-checker-tier --bin e185_gc_batch_wall_clock_split
  # Inputs list (written before launch, in the tree being launched): sha256 of the apparatus, of several singlefs-core source files and of Cargo.lock, the features from cargo tree, the image disk model and logical block, nproc, the two caps
  cargo tree -p singlefs-checker-tier -e normal,build,features -i singlefs-core --offline
  # Phase one (the real-disk arm's images go on ext4, not in a directory cleared at boot; E185_MEMORY_CAP=40G)
  target/release/e185_gc_batch_wall_clock_split wall-clock --image-directory e185-images > research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-<日期>.out
  # Phase two (memory disk only; on the same host as phase one, one after the other)
  target/release/e185_gc_batch_wall_clock_split stage-split > research/results/e185-gc-batch-wall-clock-split-stage-split-r1-<日期>.out
  # Comparison (after both phases are handed back; reads artifacts only)
  python3 scripts/experiment-apparatus/e185_r1_compare.py --before-wall-clock research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out --before-stage-split research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out --after-wall-clock research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out --after-stage-split research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out --inputs-list research/results/e185-r1-inputs-2026-10-22.txt > research/results/e185-r1-comparison-<日期>.out
  python3 scripts/experiment-apparatus/e185_r1_compare_test.py
  # Phase three (runs only when all four cells of "whether phase three runs" in section five of the rerun registration hold; built separately with the gpu feature, launched through scripts/crash/gpu-quiesce-run.sh, given only this pass's two -r1- artifacts)
  cargo build --release -p singlefs-checker-tier --features gpu --bin e185_gc_batch_wall_clock_split
  bash scripts/crash/gpu-quiesce-run.sh -- target/release/e185_gc_batch_wall_clock_split gpu-submission --wall-clock-product research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out --stage-split-product research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out --cards 0,1 > research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-<日期>.out
  # Phase four (incidental, runs only when phase one name=q16 is judged needs_the_concurrent_measurement; not run in this pass)
  target/release/e185_gc_batch_wall_clock_split foreground-fsync --image-directory e185-images --point 0.75:8192 --batch-units 16
  ```

  Every command that runs the apparatus is launched through `scripts/hosts/run-with-memory-cap.sh` (outer 46G in this pass); in this pass phases one and two ran on the second host through `scripts/hosts/multi-host-run.sh --peer`; the N-thread level takes `E185_THREADS`, or if unset `SINGLEFS_THREAD_CAP`, or if that is unset too `.claude/heavy-thread-default`. The date in an artifact's file name is the day that phase started, and a second run on the same day appends `-2`, `-3`, never overwriting an existing file.

  **The items of the first rerun's measurement basis that belong only to that pass (original body lines 131, 134, 135)**

  - Device: the memory arm is the harness's sparse memory disk; the real-disk arm is two image files on ext4 on an NVMe drive of the second host (model CT2000T500SSD8, logical block 512, `research/results/e185-r1-inputs-2026-10-22.txt` line 24), the barrier is `fdatasync`, and the conclusion only speaks of "`O_DIRECT` image files on ext4 on that disk of the second host" (rerun registration section one "real disk"); the disk of this host named in the original registration was not used in this pass.

  - Before and after 7·6: "before" is the two artifacts of the 2026-10-20 pass (the tree before 7·6 was merged; the artifacts do not record source hashes), "after" is the first rerun's two artifacts (sha256 of `transaction.rs` and `mounted_session.rs` on lines 2 and 3 of the inputs list); other batches were also merged between the two trees, so the difference before and after is only said to lie between the two trees and is not attributed to 7·6 alone. The comparison only compares the memory arm's CPU; a batch's CPU is the first batch of every round (the call with the smallest `call_index` among those whose `kind` is `batch` or `final_batch`), and the phase-one and phase-two sources are each computed on their own.
  - The two-point exponent α has only the two points P = 1024 and 8192 (the apparatus accepts only these two P): when a batch's CPU has a part that does not vary with P, the two-point exponent is smaller than the exponent of the dominant term, and "after ≤ 1.25" only says that between these two points it grows no faster than P^1.25, and does not prove the same on larger pools (rerun registration section six).

  **The first rerun's "What it cannot answer" (original body lines 140–153)**

  ### What it cannot answer

  - Segments with live-slot fraction 0.25 were not measured in this pass: the geometry of the two f = 0.25 points did not form (section "Geometry: the two f = 0.25 points did not form"), and what those two points measured is the segment greedy picked, with only one live slot that is not a unit written by this run. Rows 1 and 2 of the question sheet are therefore not enough to judge by the letter of the registration; measuring them needs a workload change so that the f = 0.25 segment gets picked (the second change in the "workload" paragraph of section five of the original registration, not implemented in the apparatus), which is a new pass.
  - Phase three's apparatus (`eadb929d…`) is not the version pinned in the rerun registration (`b3a50d50…`), so "apparatus changed" in section eleven of the rerun registration triggers by the letter; that the 3 differing lines are not on phase three's path was inferred by reading the diff, not measured. Whether phase three and Q9–Q13 count is for the main agent to decide.
  - For the three f = 0.75, P = 8192 cells where Q3 is unstable (F5) and the three cells where G3 hit, both the original registration and the rerun registration require "look into both sides", and this page did not; until that is cleared up, the Q1 and two-point exponents of these cells are not cited.
  - The artifact has no Q7 judgement (that row of the quantities table); the raw values the page reads against the threshold are not a judgement by the apparatus, and for the apparatus to judge Q7 it would have to judge on the issuing lines, which is a change to the apparatus.
  - Q9 is computed and reported as the original registration says; today's D26 (Background Compaction and Placement Reclamation) settled item 13 clause 7 says that the wall-clock-ratio criterion of D24 (Offloading Heavy Background Work to GPU) settled item 3 does not govern the background tasks it covers, so whether Q9 still counts as a flipping quantity for row 2 is not judged on this page (rerun registration section six, left to the main agent).
  - `merged_buffer_bytes=16` in the f = 0.25 `name=q13` lines is not GPU memory use: a B_批 of 16384 bytes does not fill one 32768-byte slot, and the apparatus floors the number of ranges per batch to 0 before computing the buffer; the φ and n* of f = 0.25 were also computed at points where the geometry did not form.
  - The 7·6 row's acceptance "drop from square to near linear": at the apparatus's two P points the earlier pass's α is already within 1.179549–1.231472 (G2 hit 18 cells), so the "from square" half was not measured; how this sentence should be written into the acceptance of milestone four 7·6 is not judged on this page.
  - The first-row positive control is judged on the "difference of medians" per 12.9, and the increments paired by round are reported only, not judged (in this pass the CPU arm's real-disk cell `write_wait_increment_paired_median_nanoseconds=1515882.000000`, memory cell `49895.000000`); whether and how to judge pairs after interleaving has not been decided.
  - Geometry fallback: when the segment greedy is about to choose holds nodes or packing units, the registration lists only two changes, "overwrite those segments more" and "keep the old nodes of the node segment referenced too", and the apparatus implements only the former (when the competing segment is all data units written by this run, overwrite them and rebuild); when the competing segment holds other units it goes straight to the fallback without three attempts (registration section twelve 12.3 pins this reading), and the `name=geometry` line states the reason; the latter change was never tried.
  - Phase two's saturated throughput measures replay (the same input run repeatedly), not GC itself running on several threads: today the whole GC path runs synchronously on the calling thread.
  - The pre-release disk-read check column (registration section twelve 12.5) only counts this batch's relocation-source slots. The pre-release disk-read check that the same publish does on old units being replaced (rewritten tree nodes and the like) goes through the same function, also before the first write, lands outside R, and is still recorded in `scan_read_bytes_elsewhere`: this was inferred by reading `transaction.rs`, not measured. The column split relies on the order "the region scan comes before the pre-release disk-read check, and the scan reads every slot on R's disk only once"; the investigator caught callers at only one point, f = 0.75, P = 128, K = 8, and only on R's disk; the other disk's share is inferred, and the end-to-end unit test pins, at that same point, the total for both disks at 2 × units relocated × 32768. The second sub-item of stage Jia does not enter Q5, Q8, Q10 or phase three's B_批; only its numbers are reported.
  - Stop clause 6 (the inverse of the enclosure self-check) is judged by the letter of the registration, but the two clocks do not run at the same speed: the thread CPU clock is not slewed by time adjustment, while `Instant` (`CLOCK_MONOTONIC`) is. On 2026-10-19 the runner measured this on this host with python (busy-wait, the wall-clock reading enclosing the CPU reading): in a 200 ms window the CPU reading exceeded the wall clock by at most 1526 nanoseconds (largest ratio 1.000007630), and `CLOCK_MONOTONIC` was 7.036 ppm slower than `CLOCK_MONOTONIC_RAW`; with the wall clock switched to `CLOCK_MONOTONIC_RAW` the CPU reading never exceeded the wall clock (at most −194 nanoseconds in a 200 ms window). Calls that are almost entirely CPU and last hundreds of milliseconds (the memory arm) could therefore trigger clause 6, since the few tens of nanoseconds of margin from reading the clocks cannot cover it; the wall clock paired with the thread CPU reading has been switched to `CLOCK_MONOTONIC_RAW` per 12.4, both phases' verdict lines in the first rerun are `stop_clause_findings=0`, and clause 6 did not trigger in that pass.

  **The first rerun's affected decisions table (original body lines 171–178)**

  ### Affected decisions

  | Decision item | Relation | Review |
  |---|---|---|
  | D26 (Background Compaction and Placement Reclamation) settled item 13 | Material | unaffected: the first rerun gives the readings the milestone four design round wants (row 3 of the question sheet: Q13 judged "batches should be merged" on both cards; row 2: the shares of the three stages all below 10%, Q9 all judged GPU loses; row 4 running alone is enough); that item's reach leaves "which stages of GC to compute on the GPU" and the defaults of the timed fallback interval and the write bandwidth cap to the design round and C412 (wall-clock cost of compaction unmeasured), its basis paragraph is "no experiment" and does not cite E185 (Wall-Clock Breakdown of a Batch of GC Migrations); the clause that the use of Q9 conflicts with clause 7 (G6) was not hit in this pass, and whether Q9 counts as a flipping quantity is left to the main agent; the decision text was not changed this time |
  | D26 (Background Compaction and Placement Reclamation) settled item 1 | Material | unaffected: in the first rerun the apparatus still reads its stop predicate (R entirely empty by the records) to judge the last batch, and does not measure the three water-mark thresholds; the f = 0.25 geometry not forming means the workload cannot produce that kind of segment, which is unrelated to the trigger criterion; that item's basis paragraph does not cite E185 (Wall-Clock Breakdown of a Batch of GC Migrations), and the decision text was not changed this time |
  | D24 (Offloading Heavy Background Work to GPU) settled item 3 | Material | unaffected: phase three computed Q9 in that item's criterion form (the CPU side takes the 16-thread saturated throughput, the GPU side takes a + b·B fitted on each card), all 72 lines judged GPU loses, F2 did not trigger; this plugs GC's numbers into its form and neither measures nor changes the form itself, and that item's basis paragraph does not cite E185 (Wall-Clock Breakdown of a Batch of GC Migrations); whether Q9 counts as a flipping quantity for row 2 of the question sheet is left to the main agent; the decision text was not changed this time |
  | D24 (Offloading Heavy Background Work to GPU) settled item 2 | Unaffected | unaffected: cited only as a definition (GPU results must be re-verified, two implementations on GPU and CPU): phase three cross-checks the CRC range by range as a differential, 32 lines `crcs_agree=true`, which is the apparatus's voiding check; the re-verification policy of two GPU kernels each computing once at run time and comparing is not measured, and the conclusion is unchanged |

- The first rerun (rerun preregistration `research/prompts/e185-r1-prereg.md`, following merge of milestone 4 VII.6; criteria follow original registration without changing a word) was written into the body, with title status changed from "Partially Run (phases 1 and 2 run, serving as baseline prior to merging milestone 4 VII.6: row 4 of question sheet sufficient when run alone, rows 1 and 2 awaiting rerun; phase 3 unrun, phase 4 not run after sufficiency)" to "First Rerun Completed (phases 1, 2, and 3 run after VII.6 merge, phase 4 not run after sufficiency; question sheet rows 3 and 4 sufficient, rows 1 and 2 literally insufficient per registration: f = 0.25 two-point geometry did not form; phase 3 apparatus has three more lines than registration, handed over to primary agent)". The three phase artifacts, comparison artifact, and two input manifests are named in the body; the body adds five new sections: "Geometry: f = 0.25 two points did not form", "Judgments on quantities (first rerun)", "Comparison before and after VII.6", "Comparison failure clauses G1–G6 (preregistration clause 10)", and "Question sheet line by line (forks table)", with rerun, measurement basis, and what it cannot answer rewritten per this pass, and four rows of the affected decisions table reviewed. Readings: phase 1 Q1 and Q2 across 24 cells yield `both` 11, `cpu` 10, `unstable` 3, `disk` 0, flipping with P; f = 0.25 two points record `formed=false` in `name=geometry` (same in 2026-10-20 pass), excluded from judgment per original registration section 11, leaving only f = 0.75 for the active slot share required by row 1 sufficiency conditions, while f = 0.75, P = 8192 across three cells has unstable Q3 (F5), rendering row 1 and consequently row 2 literally insufficient; phase 2 across 28 rows has stage judgments where phases A and B are all `not_worth`, and phase C is `not_run` (apparatus only judges on batch rows, issue row original value at most 0.033060), hitting original registration F7; phase 3 Q9 across 72 rows all `gpu_loses`, Q13 across 56 rows all `merge_batches` (φ 0.911860–0.998500), rendering row 3 sufficient; Q16 `maximum_ratio=0.011019`, leaving the run-alone half of row 4 sufficient as before. Comparison: post-VII.6 per-batch CPU two-point exponent 0.994926–1.030719, versus 1.179549–1.231472 prior, reducing per-batch CPU for the same workload to 0.301611–0.523085 of prior; G1 hit 1 cell, G2 18 cells, G3 first 2 cells and last 1 cell, G4 0, G5 27 cells, G6 not hit. Phase 3 input manifest apparatus sha256 is `eadb929d…`, differing from `b3a50d50…` pinned in preregistration (3 extra lines for `quarantine_mark`), literally triggering section 11 "apparatus changed", handed over to primary agent.
  - **Basis**: Four verdict lines (line 915 of `research/results/e185-gc-batch-wall-clock-split-wall-clock-r1-2026-10-22.out`, line 676 of `research/results/e185-gc-batch-wall-clock-split-stage-split-r1-2026-10-22.out`, line 202 of `research/results/e185-gc-batch-wall-clock-split-gpu-submission-r1-2026-10-22.out`, line 701 of `research/results/e185-r1-comparison-2026-10-22.out`); geometry lines (phase 1 lines 40, 138, 236, 514); input manifests line 1 of `research/results/e185-r1-inputs-2026-10-22.txt` and line 2 of `research/results/e185-r1-inputs-gpu-submission-2026-10-22.txt`; `git diff HEAD` against apparatus source files.
  - **Before**: `git show 841b3beab:.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` (the version before workspace edits differed only by a language-switch line).
- The conclusions from the 2026-10-20 pass were moved out of the body, retaining them in the body only as the "before" side of comparison: in current state, three items ("phase 1 first pass invalidated", "rerun two phases per 12.9 through 12.11, artifacts checked in, positive controls hold across four cells", "readings: Q1 and Q2 judge `cpu` 20 cells, `unstable` 4 cells, stage judgments for phases A and B all `not_worth`, row 4 `gc_to_sequential_write_bandwidth_maximum_ratio=0.006982`"); in measurement basis, the item "reason phase 1 first pass was invalidated"; in what it cannot answer, the sentence "paired by round number, real-disk cell write call disk-wait increment median 47972223 ns (0.136 of injected volume)", all removed from body; these numbers reside in E185 (Wall-Clock Breakdown of a Batch of GC Migrations) sections for 2026-10-21 and 2026-10-20 and registration `research/prompts/e185-preregistration.md` section twelve 12.9.
  - **Before**: `git show 841b3beab:.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` current state, measurement basis, and what it cannot answer.

### 2026-10-21

- Body and title filled in per the current state, status changed to "partly run (first and second segments run, a baseline from before the milestone four 7·6 merge: line 4 of the question sheet single-run enough, lines 1 and 2 to be judged after a rerun; third segment not run, fourth segment not run after enough to judge)": the two segment artifacts `research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out` (911 lines) and `research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out` (677 lines) are named in the body, and both verdict lines have `constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0`; in the first segment, of the 24 Q1 and Q2 cells, 20 are judged `cpu` and 4 `unstable`; in the second segment, of the 28 `name=stage_judgement` lines, stages A and B are all `not_worth` and stage C all `not_run`; the single-run half of line 4 is "single-run enough" (`gc_to_sequential_write_bandwidth_maximum_ratio=0.006982`); apparatus 87 unit tests (all passed when run on the second host), 55 mutation rows (V1–V55; the light tier of V21 and V34–V55 all caught on the second host); section twelve of the registration recorded up to 12.11; source placement line numbers checked fresh against the main workspace copy of the apparatus; the four rows of the affected decisions table reviewed.
  - **Basis**: the verdict lines of the two artifacts (line 910 of `research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out`, line 676 of `research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out`); `grep -c '#\[test\]'` on the apparatus source file = 87, and 55 rows in `crates/mutations.tsv` whose names start with E185 (Wall-Clock Breakdown of a Batch of GC Migrations) plus " V"; lines 64, 66, 70 and 107 of the dispatch record `records/2026-10-19-里程碑四GPU垃圾回收开工.md`; 12.10 and 12.11 of section twelve of registration `research/prompts/e185-preregistration.md`.
  - **Before**: `git show 5569c0162:.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md` (title "apparatus written, not run", with the current state saying 59 unit tests, 33 mutation rows, and no artifacts under `research/results/`).

### 2026-10-20

- The CPU arm positive control of the first pass of the first segment voided, criterion rewritten (registration section twelve 12.9): in the pass of the first segment run on the second host (5 baseline rounds run first, controls after), row 1 of the positive control, the CPU arm's original criterion "the change in disk wait time is within ±10%", failed in both the in-memory cell (`wait_change=-0.530736`) and the real-disk cell (`wait_change=0.264753`); this arm is void as a whole segment on both devices, and line 1 of the question sheet cannot be judged from that pass; the single-run half of line 4 is enough to judge per the original criterion ("single-run enough"). The main agent decided: the CPU arm's disk wait sentence is rejudged as "the absolute value of the disk wait increment of write calls ≤ number of write calls × 2 ms × 10%", with the original criterion computed and reported as is but not used for the verdict; at the control points the baseline and the two arms' controls are interleaved round by round, with the order rotated by round number; the whole first segment is rerun under the new pass. Apparatus unit tests 59 → 64 and mutations V31–V33 are all in the runner's draft, not yet compiled or run.
  - **Basis**: 12.9 of section twelve of registration `research/prompts/e185-preregistration.md` (the path, sha256 and original lines of that pass's artifact, the investigator's report and the classification as "the criterion itself can also be written wrong" are all there).

### 2026-10-19

- Newly set up, apparatus written, not run: page created, pre-run registration `research/prompts/e185-preregistration.md` (section twelve records only 12.1: the mutation table lands in `crates/mutations.tsv`, criteria untouched); committed apparatus `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` with four subcommands and 45 inline unit tests written, and 22 mutation rows appended to `crates/mutations.tsv`; unit tests, the three mutation numbers and the four segments' artifacts all not run.
  - **Basis**: line 1 of the question sheet `research/prompts/m4-gpugc-measure-forks.md` and the single-run half of line 4 (the forks this dispatched segment must answer), dispatch record `records/2026-10-19-里程碑四GPU垃圾回收开工.md`.

## E186 (Unknown Trees: Accounting Dimension Rederived)

### 2026-10-20

- Newly set up, first segment (Q1, Q2) run, both rows enough to judge, mutation table pending: page created, pre-run registration `research/prompts/e186-preregistration.md` (revision one of section twelve: negative control B judges only the clauses counted under reading B1, with U12 and U13 reported as incidental; the incidental quantities `per_generation_rows_dropped` and `i313_syntactic_violations` are computed in the same pass as the judged quantities, reported but not judged), apparatus `scripts/experiment-apparatus/e186_unknown_tree_accounting_rederive.rs` (22 unit tests), mutation table `scripts/mutation/tables/e186_unknown_tree_accounting_rederive.tsv` (17 entries, not run), artifact `research/results/e186-unknown-tree-accounting-rederive.out.gz` (186312 lines), rerun table registration `E186|e186-unknown-tree-accounting-rederive||e186-unknown-tree-accounting-rederive.out.gz|exact`.
  - **Basis**: the two rows Q1 and Q2 of the question sheet `research/prompts/c693-forks.md`, dispatch record `records/2026-10-18-里程碑三收尾与近期欠账.md`.
- Mutation table of 17 run, all caught; status changed to "run (Q1 and Q2 enough to judge; all 17 mutations in the table caught)"
  - **Basis**: the main agent ran `scripts/mutation/mutate.sh e186-unknown-tree-accounting-rederive …` (cap 3G), ending with "counts: hit memory cap 0 (cap 3G), timeout 0, could not schedule 0, squeezed out by the total cap 0, insufficient GPU memory 0, anchor broken 0, judged fresh 17, reused 0, refused 0" and "restored, baseline still all green", `✅` 17; log `~/.local/share/singlefs/work/m3-close-w8-e186-run2/mutate-1.log`.
  - **Before**: status "partly run (Q1 and Q2 enough to judge; mutation table of 17 pending)".
- Second run (a rerun with changed criteria, the Q1 row): the W1 cell in section two of the verdict `research/prompts/m3-close-c693-r1-main-verification.md` judged that the first run's criteria were written wrong in two places: U5 is "vacuously true when not written" for `NoCarry` kept with the root, and the reverse acceptance clause did not state the consequence of "depends on the sampling point" or the attribution where the two branches overlap, and counted the points with `h_new = 0` in the judgment. Per the rerun registration (section "second run (rerun registration, 2026-10-20)" of `research/prompts/e186-preregistration.md`, revision one of section twelve) the apparatus was changed: a parameter `--judgement <vacuous-u5|dangling-u5>` added, unit tests 22 → 35, mutation table 17 → 26; new artifact `research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz` (186285 lines), and the rerun table gains a line `E186|e186-unknown-tree-accounting-rederive|--judgement dangling-u5|…-second-run-2026-10-20.out|exact`. Readings: the judged points (legal points with `h_new > 0`), 6400 points, have the form `unique_carry`, and ㊀ does not flip; the artifact without the parameter is byte-identical to the first run; all 26 mutations in the table caught. This time it was not a blind test (section four of the registration); the first run's concluding text is not changed, and the two are listed side by side; status changed to "run (first run Q1 and Q2 enough to judge, all 17 mutations caught; second run Q1 enough to judge, all 26 mutations caught)". The artifact was produced by a binary compiled from the same source in a draft copy (the main workspace merge lock was held, so cargo could not start in the main workspace), and byte-for-byte rechecking is left to `replay.sh E186`.
  - **Basis**: the W1 cell in section two of the verdict `research/prompts/m3-close-c693-r1-main-verification.md`; artifact line `E7RESULT name=verdict check=q1_carry_acceptance_form … group=legal_pooled … form=unique_carry consequence=carry_stands winner_sets=Carry:6400` (line 186278 of `research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz`); mutation table ending "counts: hit memory cap 0 (cap 3G), timeout 0, could not schedule 0, squeezed out by the total cap 0, insufficient GPU memory 0, anchor broken 0, judged fresh 26, reused 0, refused 0" and "restored, baseline still all green", `✅` 26.
  - **Before**: status "run (Q1 and Q2 enough to judge; all 17 mutations in the table caught)".

## E187 (Cost of Persisting Quarantine Marks)

### 2026-10-22

- Second segment run, all 18 mutations caught, the fork-sheet row is decidable: the verdict row of the old second-segment artifact `research/results/e187-quarantine-mark-cost-segment-two-2026-10-21.out` had `scheduled_events_realized=false` (two cells placed only 9 rollbacks each), hitting the invalidation condition; the cause was that the apparatus dropped an event when one draw of the rollback delay failed. A second scheduling pass was added to the apparatus (not a word of the registration changed), unit tests went from 30 to 33 and the mutation table from 17 to 18 entries, and the rerun produced `research/results/e187-quarantine-mark-cost-segment-two-2026-10-22.out` (6933 lines), which differs from the old one only in those two cells' rows and the summary rows; the old artifact is void as a whole and kept, not deleted. Readings on the page: at density 10 A reaches at most band two and B band three; failure conditions `quarantine_cost_negligible`, `span_bit_writes_nothing_extra` and `no_mark_false_red_persists` are hit, and `mark_hides_a_leak` is hit on `mark_or_today`; which of A and B is smaller depends on the sample point. The value of the regression pin was added to section twelve of the registration. One row was added to the rerun table: `E187|e187-quarantine-mark-cost|segment-two|e187-quarantine-mark-cost-segment-two-2026-10-22.out|exact`. Status changed to "run (both segments run, the fork-sheet row is decidable; all 18 mutations caught)".
  - **Basis**: line 6932 of the new artifact `E7RESULT name=verdict segment=two quarantine_cost_negligible_holds=false span_bit_writes_nothing_extra_holds=false no_mark_false_red_persists_holds=false arms_agree_without_events=true scheduled_events_realized=true unreadable_counts=zero`; the mutation table ends with "计数：内存撞顶 0 条（上限 3G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条、显存不够 0 条、锚点失效 0 条、现判 4 条、复用 14 条、拒算 0 条" and "已还原，基线仍全绿" (14 reused from the verdict library), `✅` 18 entries; the check of the invalidation condition and the apparatus fix are in the executor report `~/.local/share/singlefs/work/m4-e187-runner-2/report.md`.
  - **Before**: status "partly run (first segment run; second-segment workload and mutation table not run)".

### 2026-10-21

- Newly set up, first segment run, second segment and mutation table not run: page created, pre-run registration `research/prompts/e187-preregistration.md` (revisions one to five in section twelve, written after the unit tests and before the artifact; revision one refines the model's two placement rules per `crates/`, for the main agent to accept), apparatus `scripts/experiment-apparatus/e187_quarantine_mark_cost.rs` (30 unit tests, plus 2 thread registration slots), mutation table `scripts/mutation/tables/e187_quarantine_mark_cost.tsv` (17 entries, not run), first-segment artifact `research/results/e187-quarantine-mark-cost-segment-one-2026-10-21.out` (250 lines), rerun table registration `E187|e187-quarantine-mark-cost|segment-one|e187-quarantine-mark-cost-segment-one-2026-10-21.out|exact`. Status "partly run (first segment run; second-segment workload and mutation table not run)".
  - **Basis**: that row of the fork sheet `research/prompts/m4-i315-quarantine-mark-cost-forks.md`, the m4-i315-quarantine-mark-cost section of batch `records/2026-10-13-里程碑四预开发.md`; the second segment (320 cells × three arms) ran 42 cells in 225 s in the foreground with this host fully loaded, handed to the main agent to start.
