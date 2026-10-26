## E97 Entry Encoding of Accounting and Allocation Records —— Executed (2026-09-03, pure arithmetic, 22 unit tests / 17 mutations caught, rerun three times after three rounds of three-way argument; fifth rerun on 2026-10-20 recomputed criterion 10 with each root holding only one generation, 68 unit tests / 16 mutations all caught): **It eliminated candidates and narrowed two unsettled items, but neither was settled** — segment width does not affect tree height; finite modulo generation cycles in 91 ms; placement coordinate system must be 16 KiB slot numbers; "can represent all legal starting points" and "half a unit cannot be represented" are mutually exclusive; segment manifest still lacks settled field `seq`; under the single-generation measurement basis the wide-read cells 1024x64 and 8192x8 stay within the bound, and the tightest narrow-read cell is at 2%

[中文](97-记账与分配记录的条目编码.md) · **English** · [日本語](97-記帳および割り当て記録のエントリ符号化.ja.md)

**What it aims to answer are two unsettled items blocking before the first line of code**: D5 (Snapshot / Space Accounting Mechanism) settled item 5
(widths of accounting key segments) and D3 (Space Allocation) settled item 7 (on-disk encoding and width of allocation record entries).
Both were judged "modifies new-pool file creation bytes: **yes**".

**Why these two items can be synthesized into one experiment**: both trees share the same btree mechanism (D3 (Space Allocation) settled item 3
verbatim: "format follows the accounting tree in D5 (Snapshot / Space Accounting Mechanism) settled item 3 — identical btree mechanism, no second structure created"),
share the same settled node size (D8 (Core Index Structure) settled item 2, 16 KiB),
and allocation record keys and position entries (position entry) share the **same coordinate system** (D3 (Space Allocation) settled item 3 basis item 2 verbatim).
⇒ The segment width dimension is the same geometric problem across both trees.

### Cited Clauses (Transcribed Verbatim into Source Comments)

| Clause | Verbatim excerpt used by E97 (Entry Encoding of Accounting and Allocation Records) |
|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 1 | key is written as `(metric, tree ID, dev, gen)` |
| D5 (Snapshot / Space Accounting Mechanism) settled item 2 | generation takes checkpoint number; retains only recent K generations, `K = root ring total slots + 1` |
| D5 (Snapshot / Space Accounting Mechanism) settled item 3 | btree with independent keyspace; entry `(metric, dimension tuple, gen) -> full value`; traverses write buffer front-end, entry carries seq |
| D5 (Snapshot / Space Accounting Mechanism) settled item 4 | eleven metrics; row-by-row whether device dimension is included |
| D3 (Space Allocation) settled item 3 | key = placement (device ID + intra-device offset), value = allocation generation; `(dev, offset)` shares same coordinate system with position entries |
| D4 (Checksum Location) settled item 5 | unit disk footprint strictly 32768 bytes (including header) |
| D8 (Core Index Structure) settled item 2 | node 16 KiB, pinned as constant |
| D18 (Information Carried in a Block) settled item 2 | **accounting tree** nodes carry key intervals; dirent tree deferred. **Whether allocation record tree carries them is not stated** |
| D18 (Information Carried in a Block) settled item 7 | index node header field set => base header >= 58 bytes (E73 (Fanout Cost of Node Key Interval) recomputed over 58 / 67 / 76 tiers) |
| D22 (Synthesizing Unit Atomicity) settled item 2 | root ring R = 3 regions, slots per region S lives in system configuration, 1..16 => total slots 3S, `K = 3S + 1 <= 49` |
| D22 (Synthesizing Unit Atomicity) settled item 7 | `checkpoint_txg` in root record is 8 bytes; tree table unit pointer actual width (53 during run) |
| D18 (Information Carried in a Block) settled item 9 | "Unit starting point aligns by class — **data units align to 32768, index nodes align to 16384**, otherwise scanner stride will misalign". ⚠️ **Omitted in first version**, supplemented into manifest after first adversarial round — alone refutes "encoding placement by 32 KiB unit numbers" |
| D16 (Publish Semantics) settled item 5 | `T_dirty` = 2 GiB; `T_time` = 5 s (gives wraparound period for criterion 8) |
| `.claude/rules/fs-design.md` | "Do not sacrifice self-containment for space savings"; and "**comparative savings constitute no criterion**" |

### Criteria (Hardcoded Before Run, Immutable After Run)

1. **Absolute Value Assertion**: Fanout under each width combination tier must exactly equal
   `(16384 − node header − interval fields) / entry width`, where interval fields are `2 × key width`
   on trees carrying key intervals, and 0 on trees without. **Cross-arm relative comparison alone is prohibited** — three arms sharing a wrong formula will be mutually identical.
2. **Domain Coverage**: Each segment's width `w` must satisfy `2^(8w) > segment domain upper bound`,
   **and that upper bound must point to a settled clause**; for segments pointing to none, E97 (Entry Encoding of Accounting and Allocation Records) **reports "upper bound unavailable", without settling it arbitrarily**.
3. **Key Order Monotonicity**: Key must be monotonic over its domain. When encoding "generation" as a ring counter modulo M,
   the model must **count inverted pairs**; arms with non-zero inverted pairs are judged "unusable".
4. **Budget**: Accounting write bytes per checkpoint <= 1% of `T_dirty` (same line as E71 (Entry Count and Width of Accounting Keys) criterion 2).
5. **Monotonicity**: Adding 1 byte to any segment => entry width increases by 1, fanout monotonically non-increasing, tree height monotonically non-decreasing.
6. **Representability of Illegal States**: For the same placement coordinate, how many **non-unit-aligned** placements can be expressed
   when encoded by byte offset versus unit number. This is the field encoding manifestation of `.claude/rules/fs-design.md`
   criterion "can misuses turn into failure to mount", requiring countable absolute values rather than merely saying "safer".
   ⚠️ **Has an unsettled prerequisite**: "placement" granularity is nowhere settled across the repository
   (E74 (Allocation Record Entry Count and Compaction Cost) verbatim: "32 KiB settled in D4 (Checksum Location) is **unit**, not definition of 'placement'")
   => This criterion's conclusion is **conditional**, conditional on "placement granularity = unit". If condition does not hold, criterion 6 has no object.
   ⚠️ **Both quantities in criterion 6 are tautologies**: "non-aligned placement is 0 when encoded by slot number" returns 0 on function's first line,
   and "N times addressing under same width" has N identically equal to granularity itself. **They are definitions of encoding conversion, not discoveries** —
   they can only support the analytical argument "slot number encoding is strictly stronger under same width"
   (`.claude/singlefs-ai-sop/rules/machine-first.md` dogma 2), **forbidden from being recorded as measured facts**.
   What genuinely holds discriminative power is criterion 7.
7. **Alignment Representability**: For a placement field encoding slot numbers at granularity g, which classes among the **two classes**
   required by D18 (Information Carried in a Block) settled item 9 (data unit 32768, index node 16384) can it represent? Those unrepresentable must **have absolute counts measured**.
8. **Generation Field Wraparound Period**: Generation field modulo M wraps around every `M / 2785` seconds based on **publish rate**
   (E44 (Host Testing and Cost of Sequence Number Bit Width) measured 2785/sec, used by rider clause of D5 (Snapshot / Space Accounting Mechanism) settled item 2);
   criterion 3 inverted pairs **must be tested on grids where generation count > M**; testing only 32 / 64 generations grants exemption to modulo 256.
9. **Segment Manifest Completeness**: Every settled clause requiring "a certain field to exist" must find a placement in the segment manifest.
   **Criterion 2 checks only listed segments; it says nothing when an entire segment is omitted.**
10. **Budget Counted by Dirty Nodes, Not Entry Bytes**: Accounting tree hangs under root via COW
   (D5 (Snapshot / Space Accounting Mechanism) settled item 3 clause 4) => an insertion of several tens of bytes dirties a full 16 KiB node.

### Failure Clauses (Hardcoded Before Run)

| Clause | Judgment when triggered |
|---|---|
| **Positive control, run on every arm**: Truncate any segment width to 0, ① entry width must decrease by exactly that much, ② coverage judgment of criterion 2 must flip Red | If either fails => that segment never entered model, **entire run void** |
| **Positive control two**: Assign modulo M smaller than generation count to "generation" => criterion 3 inverted pairs must be > 0; when M is larger than generation count it must be exactly 0 | If fails => criterion 3 check is cosmetic, **entire run void** |
| Base bytes of node header remain unsettled in repository => compute once for each of E73 (Fanout Cost of Node Key Interval) three lower bound tiers 58 / 67 / 76 | If three tiers differ in direction => record conclusion as "dependent on node header", forbidden from picking one tier |
| **Reverse acceptance clause (hardcoded before run)**: If most economical tier turns Red on criterion 2 or 3, while widest tier exceeds boundary on criterion 4 | Conclusion is "no feasible point exists, 16 KiB of D8 (Core Index Structure) settled item 2 or K of D5 (Snapshot / Space Accounting Mechanism) settled item 2 must be reopened", **record truthfully, forbidden from forcing a middle tier** |
| "Comparative savings" constitutes no criterion (`.claude/rules/fs-design.md`) | Capacity percentage reported solely as **cost reading**, **forbidden from selecting a tier based on it alone** |

### Measurements (2026-09-03, pure arithmetic, N=5 rounds byte-identical, 15 unit tests / 10 mutations all caught)

**Measurement basis and rerun**:

- Code `scripts/experiment-apparatus/e97_entry_encoding.rs`
  (`cargo run --release --bin e97-entry-encoding`),
  raw output `e97-entry-encoding-2026-09-07.out` (rerun 2026-09-07 under established header width of that day; version before accounting fix is `e97-entry-encoding-2026-09-03.out`, retained untouched in `results/`) (240 lines, concluding `emitted=240`).
- **N=5 rounds byte-identical**. **22 unit tests, 17 mutations hit individually, 0 judged equivalent**, mutation table
  `scripts/mutation/tables/e97_entry_encoding.tsv`
  (`bash scripts/mutation/mutate.sh e97-entry-encoding scripts/experiment-apparatus/e97_entry_encoding.rs scripts/mutation/tables/e97_entry_encoding.tsv`).
- Integrated into `scripts/experiment-apparatus/replay.sh` (exact tier), rerun via
  `bash scripts/experiment-apparatus/replay.sh` with experiment number.
⚠️ **Deterministic model, N=5 demonstrates absence of hidden state, not statistical stability.**

#### Accounting Tree: Width Does Not Shift Tree Height by a Single Cell

| Arm (metric/tree/device/gen) | key width | leaf entry | leaf fanout (header 58) | internal fanout (child ptr 53) | narrow read 4300 entries height | wide read 73600 entries height |
|---|---|---|---|---|---|---|
| 2/8/4/8 (E73 (Fanout Cost of Node Key Interval) assumed 22) | 22 | 30 | 542 | 217 | **2** | **2** |
| 2/8/1/8 | 19 | 27 | 603 | 226 | **2** | **2** |
| 1/4/1/8 | 14 | 22 | 740 | 243 | **2** | **2** |
| 1/4/1/1 (window generation) | 7 | 15 | 1087 | 271 | **2** | **2** |

**Criterion 1 self-consistent with two existing models**: 22-byte key + header 58 + with interval + child pointer 32 => internal fanout exactly **301**,
cell-for-cell identical to E73 (Fanout Cost of Node Key Interval) table; without interval is 302, also identical.
E74 (Allocation Record Entry Count and Compaction Cost) 20-byte entry + header 64 => fanout exactly **816**, likewise identical.
⚠️ **This is merely self-consistency verification, not independent validation** (`.claude/singlefs-ai-sop/rules/evidence-discipline.md`
requires validation paths to "share no identical code segment, no identical sampling, no identical tool"): each of the three experiments wrote its own
`fanout()`, and their formulas were verbatim identical — `(node − header) / entry width`.
**It catches transcribed parameter errors, but cannot catch "fanout concept itself modeled incorrectly", where all three would err together.**

**Criterion 2 (Domain Coverage)**: Upper bounds for metric tag and generation point to settled clauses;
**upper bounds for tree ID and device ID point to no settled clause**, model reports `upper_unknown`, **without settling them arbitrarily**.

**Criterion 3 + 8 (Key Order Monotonicity and Wraparound Period)**: Encoding generation as a ring counter modulo M disrupts key order,
**and inverted pairs appear only after generation count exceeds M** —

**Wraparound period calculated by publish rate, not by `T_time`** — rider clause of D5 (Snapshot / Space Accounting Mechanism) settled item 2 verbatim:
"**Generation advances with publish, not with 5-second window**... generation segment width **>= 48 bits** (wraps around in 17.8 days under 2785 publishes/sec with 32 bits)",
publish rate 2785/sec originates from E44 (Host Testing and Cost of Sequence Number Bit Width) host measurements.

| Modulo M | Wraparound period | 32 gen | 64 gen | 257 gen | 300 gen | 512 gen | 70000 gen |
|---|---|---|---|---|---|---|---|
| No modulo | Never | 0 | 0 | 0 | 0 | 0 | **0** |
| 49 (upper bound of K) | **17 ms** | 0 | **615** | 14 310 | 19 278 | 61 170 | 1 199 579 976 |
| 256 (1 byte) | **91 ms** | 0 | 0 | **255** | 10 274 | 32 640 | 1 217 957 832 |
| 1024 | 367 ms | 0 | 0 | 0 | 0 | 0 | 1 214 169 376 |
| 65536 (2 bytes) | 23.5 s | 0 | 0 | 0 | 0 | 0 | **282 586 824** |

⇒ **Any finite modulo merely postpones order disruption to the moment of wraparound**, and a 1-byte generation completes a cycle in **91 milliseconds**.
⚠️ **"Generation segment width >= 48 bits" is a rider clause of D5 (Snapshot / Space Accounting Mechanism) settled item 2, not a discovery here.**
E97 (Entry Encoding of Accounting and Allocation Records) provided it with the **first executable assertion**: wraparound period modulo 2^32 is calculated as **17 days**
(rider clause states 17.8 days), modulo 2^48 is **3204 years** (E44 (Host Testing and Cost of Sequence Number Bit Width) states 48 bits lasts 3202 years).
⚠️ **This is "first automated check", not "independent verification"** — source code of E44 (Host Testing and Cost of Sequence Number Bit Width)
never computed these two figures; prior to E97 (Entry Encoding of Accounting and Allocation Records), those two statements existed only as manual text;
E97 (Entry Encoding of Accounting and Allocation Records) merely made them reproducible, **with no second path to compare against**.
⚠️ **Criterion 2 has no discriminative power over this segment**: model sets domain upper bound for generation segment to `u64::MAX`,
on grounds that "`checkpoint_txg` is 8 bytes" — **which equates to first assuming key stores full txg, then proving it requires 8 bytes**.
What genuinely eliminated window generation was the inverted pair table of criterion 3 and criterion 8 ("modulo M × generation count"), not that `OVERFLOWS`.

**Criterion 4 (Budget = 1% of `T_dirty` = 21 474 836 bytes)**: **Upper bound depends heavily on unsettled dimension interpretation,
and wide read has no feasible point on largest cell** —

**⚠️ Measurement basis revised first: Budget must count dirty nodes, not entry bytes (Criterion 10).**
Accounting tree hangs under root via COW (D5 (Snapshot / Space Accounting Mechanism) settled item 3 clause 4)
=> a 27-byte insertion dirties an entire 16 KiB node. First and second versions evaluated by entry bytes, **systematically underestimating**.

Cell-by-cell comparison under 19-byte key (leaf fanout 603, each leaf holds `603 / 25 = 24` groups) between both measurement bases:

| Cell | Reading | Group count | Entry byte basis | Dirty node basis | Exceeded (by node) |
|---|---|---|---|---|---|
| t=64 d=8 | Narrow | 172 | 9288 | 131 072 | No |
| t=64 d=8 | Wide | 2944 | 158 976 | 2 015 232 | No |
| t=1024 d=64 | Narrow | 2372 | 128 088 | 1 622 016 | No |
| t=1024 d=64 | Wide | 333 824 | 18 026 496 | **227 901 440** | **Yes** (judged No under entry basis) |
| t=8192 d=8 | Narrow | 16 428 | 887 112 | 11 223 040 | No |
| t=8192 d=8 | Wide | 376 832 | 20 348 928 | **257 261 568** | **Yes** (judged No under entry basis) |
| t=8192 d=64 | Narrow | 16 708 | 902 232 | 11 419 648 (accounts for **53%** budget) | No |
| t=8192 d=64 | Wide | 2 670 592 | 144 211 968 | 1 823 129 600 | Yes |

⇒ **After switching to node measurement basis, two more cells in wide read turn Red**; narrow read passes all four cells, with tightest cell occupying 53% of budget.
⚠️ **Consequently, the "key width at most 20 bytes" derived in first and second versions is no longer a valid instrument**:
under node basis, narrowing key only reduces fanout linearly and mildly; **there is no path of "narrowing key to rescue wide read"**.
⚠️ Reverse acceptance clause still triggers on `t=8192 / d=64` wide read cell (budget / 5 341 184 = 4 bytes per entry under entry basis,
unable to hold even 8-byte value). **What triggers is not "16 KiB or K must be reopened", but that the wide read interpretation itself cannot hold**
=> This table conversely serves as basis that "which metrics carry tree dimension" should take narrow read.
⚠️ Upper bound is also sensitive to "whether item 10 (fragmentation runs) carries device dimension": D5 (Snapshot / Space Accounting Mechanism) settled item 4
notes that this cell is an inference needing re-verification; if it carries device dimension, upper bound for t=8192 / d=8 wide read drops from 20 to 16.

#### Allocation Record Tree: Under Same Width, Changing Encoding Gains 32768x Addressing, Illegal Placements Zeroed

| Arm | Entry | Addressable per dev | Representable non-aligned placements | **Half data unit representable** | 16 TiB entry count | Height | Capacity ratio |
|---|---|---|---|---|---|---|---|
| dev1 + 6-byte offset + gen8 | 15 | 256 TiB | **281 466 386 776 064** | No | 483 183 820 | 4 | 415 ppm |
| dev4 + 8-byte offset + gen8 (E74 (Allocation Record Entry Count and Compaction Cost) assumed 20) | 20 | 16 EiB | 18 446 181 123 756 130 304 | No | 483 183 820 | 4 | 553 ppm |
| dev1 + 5-byte 32K unit no. + gen8 | 14 | 32 PiB | 0 | No | 483 183 820 | 4 | 387 ppm |
| dev1 + 6-byte 32K unit no. + gen8 | 15 | 8 EiB | 0 | No | 483 183 820 | 4 | 415 ppm |
| dev1 + 5-byte 32K unit no. + gen1 (window gen) | 7 | 32 PiB | 0 | No | 483 183 820 | 4 | 193 ppm |
| **dev1 + 6-byte 16K slot no. + gen8 (one entry per slot)** | 15 | **4 EiB** | **0** | **Yes** | 966 367 641 | 4 | **830 ppm** |
| **dev1 + 6-byte 16K slot no. + span1 + gen8 (one entry per unit)** | 16 | **4 EiB** | **0** | **Yes** | 483 183 820 | 4 | **442 ppm** |

**First five arms completely eliminated**: those encoded by 32 KiB unit number fail criterion 7 because they cannot represent placements for half of index nodes;
those using byte offset allow non-16 KiB-aligned starting points to be represented. Remaining two both use 6-byte 16 KiB slot numbers
(4 EiB / device, non-aligned starting points unrepresentable).

⚠️ **Two requirements are mutually exclusive, representing the hardest conclusion of this experiment**:
① Representing **all** legal starting points of D18 (Information Carried in a Block) settled item 9 requires 16 KiB resolution;
② Rendering "allocating only half of a 32 KiB data unit" unrepresentable requires coarsening resolution to 32 KiB.
**No granularity satisfies both**, and adding span segments changes nothing — because "span = 1" must remain permanently legal
(an index node is exactly one 16 KiB slot), so the record `(even slot, span 1)` can always be written.
⇒ **The dimension of "whether half a unit is representable" cannot distinguish between remaining two candidates (both can represent it),
and must not serve as grounds for choosing sides.**

⚠️ **The sole remaining difference is capacity footprint (830 vs 442 ppm, 1.88x),
while `.claude/rules/fs-design.md` explicitly commands that "comparative savings constitute no criterion"**
⇒ **E97 (Entry Encoding of Accounting and Allocation Records) is insufficient to settle between these two.**
What it achieved was **elimination** (five arms out) and **narrowing** (coordinate system must be 16 KiB slot number), not settling.

**Same-width comparison under Criterion 6**: At identical 15 bytes, encoding by slot number yields **32768 times** addressing range compared to byte offset
(8 EiB vs 256 TiB), with non-unit-aligned placements dropping from 281 466 386 776 064 to **0**.
⚠️ **Both quantities are tautologies**: 32768 is the unit size itself, and "non-aligned placement is 0" returns 0 on function's first line.
**They must not be cited as empirical findings**; they support only an analytical argument — slot number encoding is strictly stronger under same width.

#### Criterion 9: Manifest Missing a Segment — D8 (Core Index Structure) Write Buffer Hard Requirement 3 `seq`

D8 (Core Index Structure) write buffer hard requirement 3 verbatim: "**Entries must carry sequence numbers (seq), this is a format requirement**",
inherited by D5 (Snapshot / Space Accounting Mechanism) settled item 3 basis item 3.
Yet accounting entry segment manifest (metric / tree ID / device / generation / full value) **lacks seq**.

| Requiring clause | Field | Present in segment manifest |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 1 | metric tag / tree ID / device ID | Yes |
| D5 (Snapshot / Space Accounting Mechanism) settled item 2 | generation | Yes |
| D5 (Snapshot / Space Accounting Mechanism) settled item 3 | full value | Yes |
| **D8 (Core Index Structure) write buffer hard requirement 3** | **`seq`** | **No** |

⇒ **This is a format-level omission unaddressed before E97 (Entry Encoding of Accounting and Allocation Records)**, to which criterion 2 is naturally blind —
criterion 2 works only on listed manifest elements; when a segment is completely missing, it cannot even report "upper bound unavailable".
⚠️ Three questions unanswered: whether `seq` lives in key or value, width, and **monotonicity across mounts**
(if zeroed upon mount, older replayed entries could override newer entries with larger seq, causing stale values to win).

#### Criterion 7: Encoding by 32 KiB Unit Numbers Cannot Represent Placements for Half of Index Nodes

D18 (Information Carried in a Block) settled item 9 verbatim adds a hard requirement to allocator:
**data units align to 32768, index nodes align to 16384**. Index nodes are 16 KiB (D8 (Core Index Structure) settled item 2)
=> **legal placement starting points are all 16 KiB slot boundaries**.

| Region (16 KiB slot count) | Encoding granularity | Representable | **Unrepresentable** |
|---|---|---|---|
| 4096 | 16384 | 4096 | **0** |
| 4096 | 32768 | 2048 | **2048** |
| 998 | 16384 | 998 | **0** |
| 998 | 32768 | 499 | **499** |

⇒ **Encoding placement by 32 KiB unit number makes half of legal starting points unwriteable.**
Corroborating evidence: E86 (Scan Stride and Two Unit Sizes) measured 998 nodes in 64 MiB region, with **496 landing on odd 16 KiB slots**
(measurement bases differ — that was a count in a specific layout, while here it is a representability count in contiguous slot ranges).

#### Placement Granularity Dimension: Halving Granularity Doubles Entries and Capacity, Tree Height Unchanged

E74 (Allocation Record Entry Count and Compaction Cost) swept three granularity tiers, whereas first version of E97 (Entry Encoding of Accounting and Allocation Records)
hardcoded it to one tier (32768) — first three-way argument round noted this as "model mistaking unsettled variable for settled constant", swept in second round.

| Granularity | 16 TiB entry count | 256 TiB entry count | Tree height | Capacity ratio |
|---|---|---|---|---|
| 16384 | 966 367 641 | 15 461 882 265 | 4 | **830 ppm** |
| 32768 | 483 183 820 | 7 730 941 132 | 4 | **415 ppm** |

(15-byte entry, header 58, without key interval; leaf fanout 1088 and internal fanout 272 are identical across both tiers.)

⚠️ **Scope note (2026-09-03, following same-day settling): child pointer in internal fanout column used 53, whereas post-settling value is 59.**
D19 (Block Pointer Structure & Width Budget) settled item 4 settled position entries at 14 bytes
=> tree table unit pointer 31 + 14 × 2 = **59**. Recomputing with 59 is arithmetic, not a rerun:
accounting tree 22-byte key internal fanout changes from 217 to **201** (`(16384−102)/81`),
allocation record 15-byte entry internal fanout changes from 272 to **247** (`(16384−58)/66`).
**Tree heights in both cases remain unchanged** (accounting remains 2, allocation record remains 4) => conclusions untouched,
with reason documented here: tree height is the sole metric used by this experiment to judge feasibility, and it is insensitive to minor fanout shifts.

**Tree height is 4 across all five arms** => width likewise does not move tree height by a single cell.

⚠️ **"COW amplification factor" column is a ratio whose denominator varies with entry width; do not use as criterion.**
Empirical absolute quantities: node writes when relocating 4096 scattered entries are **independent** of entry width (all 4096);
when relocating contiguously, narrower entries touch **fewer** leaves (15 bytes touches 4, 7 bytes touches 2).
⇒ **"Narrow entries make compaction more expensive" is a false reading**, clarified here lest it be used against compaction.
This is pinned by a dedicated unit test (`copy_on_write_amplification_is_a_ratio_not_an_absolute_cost`).

#### Execution of Failure Clauses

| Clause | Result |
|---|---|
| Positive control 1 (each segment cut to 0) | **Passed**: four segments cut once each, key width decreases by exactly that much; coverage flips Red for two bounded segments, remains `upper_unknown` for two unbounded segments |
| Positive control 2 (modulo M smaller than gen count) | **Passed**: modulo 16 / 49 inverted pairs are 720 / 615 respectively, modulo 256 and no-modulo constantly 0 |
| Three node header tiers 58 / 67 / 76 | **Co-directional**: tree height unchanged across all three tiers, conclusion independent of node header |
| Reverse acceptance clause | **Triggered on one cell and recorded truthfully**: on `t=8192 / d=64` **wide read** cell, any key width exceeds boundary (budget / 5 341 184 = 4 bytes per entry, cannot hold 8-byte value). Remaining seven cells have non-empty feasible domains. => Triggered is not "16 KiB or K must reopen", but that **wide read reading does not hold at that scale** |

### What It Cannot Answer

Pure arithmetic geometric model: no btree implementation, no write buffer, no allocator, no I/O, zero file operations.
**Does not answer wall clock.** Does not answer "which metrics carry tree dimension" — that dimension is not listed in table of D5 (Snapshot / Space Accounting Mechanism) settled item 4
(E71 (Entry Count and Width of Accounting Keys) measured 14.5x disparity between both readings on same cell),
E97 (Entry Encoding of Accounting and Allocation Records) computed **once each for narrow and wide readings without picking one**.
**Deterministic model**: running identical binary N times produces byte-identical output => N runs prove absence of hidden state,
not statistical stability (`.claude/singlefs-ai-sop/rules/test-discipline.md`). Evidence strength derives from
criterion 1 absolute assertions and mutation testing.

⚠️ **Rerun 2026-09-07 under established header width of that day** (step requested by user prior to D18 (Information Carried in a Block) settled item 14).
Prior to 2026-09-07 rerun, three base node header tiers in source were **58 / 67 / 76**, representing lower bounds on the day E73 (Fanout Cost of Node Key Interval) ran;
two subsequent widenings were never reflected in source, supplemented together here:
write sequence 10 (D18 (Information Carried in a Block) settled item 7 supplementary note) + reserved bits 28
(D18 (Information Carried in a Block) settled item 7 verbatim "v1 reserves field bits for nonce tag and MAC",
with settled item 14 Arm A nonce tag 12 and D9 (Encryption) settled item 2 MAC 16) => **96 / 105 / 114**.
Both header sets remain in source code evaluated cell by cell; older set reflects ground where conclusion stood prior to 2026-09-07 rerun.
**Conclusion did not overturn**, only intermediate quantities shifted. Artifact replaced with rerun of same date.

**Intermediate quantities**: accounting tree leaf fanout 542 -> **540**, internal fanout (entry 54) 301 -> **300** (entry 75 tier 216 unchanged).

### 2026-10-20: Criterion 10 Recomputed Under "Each Root Holds Only One Generation, Write Only, No Delete" (Fifth Rerun, the E97 Half of Q3 in Question List `c693-forks.md`, Rerun Registration `e97-r5-prereg.md`)

**Why rerun**: D5 (Snapshot / Space Accounting Mechanism) settled item 2 now reads "the accounting tree of each root holds only the rows of that root's generation, not several generations", with the range "each publish writes all rows of this generation once". Criterion 10 was computed with K = 25 generations and `leaf_fanout / 25` groups per leaf, and criterion 4 with one write plus one delete × 2; those premises no longer hold. **Only the row-count measurement basis changes** (K 25 → 1, writes per group per publish 2 → 1); the eleven-item statistic table (the one of 2026-09-03), the four key arms, node header 58, the four (t, d) cells × narrow read / wide read, and the threshold 21 474 836 (1% of `T_dirty`, exceeding = strictly greater) stay as originally registered. The judged arm is the 19-byte key arm (`acct_wide_19B`, leaf fanout 603; the "wide" in the arm name is the key width, not the wide read).

**Measurement basis and rerun**:

- A separate new harness `scripts/experiment-apparatus/e97_entry_encoding_single_generation.rs` (`cargo run --release --bin e97-entry-encoding-single-generation`); the original harness `e97_entry_encoding.rs`, its mutation table and its rerun row are unchanged to the letter. The same harness computes the old and new measurement bases side by side, cell by cell.
- Artifact `research/results/e97-entry-encoding-single-generation-2026-10-20.out` (247 lines, closing `emitted=247`); `scripts/experiment-apparatus/replay.sh` gained one exact-tier row (under E97 (Entry Encoding of Accounting and Allocation Records), with driver `e97-entry-encoding-single-generation` and artifact `e97-entry-encoding-single-generation-2026-10-20.out`), the original row unchanged.
- **N=5 rounds byte-identical** (the other four rounds show no difference from the committed one under `cmp`). **All 68 unit tests green** (20 hand-written + 48 anchor cases). **16 mutations: 16 caught, 0 invalid, 0 not red** (`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh e97-entry-encoding-single-generation e97_entry_encoding_single_generation.rs ../mutation/tables/e97_entry_encoding_single_generation.tsv`, closing "restored, baseline still all green"; memory cap hit, timeouts, not scheduled, squeezed out by the total cap, GPU memory short and anchors broken all 0).
- Section 12 of the registration records five amendments made after the unit tests and before the artifact, all of them asking for more: incidental 2 (criterion 4 by × 1 and its inverse) and incidental 3 (tree height with K = 1) run into the artifact; mutations FV11–FV16 added; anchor comparison extended to 48 rows; the old measurement basis additionally compared on 8 inverse rows and 48 tree-height rows; the per-cell lines additionally report the old-basis percent and permille. Incidental 1 (the tightest wide-read cell within the bound in the literal reading of D5:146) and incidental 4 (the implementation row set 3 + 6d) **were not run once the question was sufficient to judge**.
⚠️ **Deterministic model: N=5 shows there is no hidden state, not statistical stability**; evidence strength comes from the assertions pinning absolute values (48 anchors computed only from the formulas written down in the registration by the independent script of class B in section 7 of the registration) and from mutations.

**Verdict lines** (copied whole from lines 226–230 of `e97-entry-encoding-single-generation-2026-10-20.out`):

```
E7RESULT name=judged_cell question=Q-E97-1 arm=acct_wide_19B trees=1024 devs=64 reading=wide old_node_bytes=227901440 old_over=1 new_node_bytes=9076736 new_over=0 new_percent=42 new_permille=422
E7RESULT name=judged_cell question=Q-E97-2 arm=acct_wide_19B trees=8192 devs=8 reading=wide old_node_bytes=257261568 old_over=1 new_node_bytes=10240000 new_over=0 new_percent=47 new_permille=476
E7RESULT name=wide_over_cells question=Q-E97-3 arm=acct_wide_19B old_over_cells=3 old_over_grids=1024x64,8192x8,8192x64 new_over_cells=1 new_over_grids=8192x64
E7RESULT name=narrow_tightest question=Q-E97-4 arm=acct_wide_19B old_node_bytes=11419648 old_grids=8192x64 old_percent=53 old_permille=531 old_over=0 new_node_bytes=458752 new_grids=8192x8,8192x64 new_percent=2 new_permille=21 new_over=0
E7RESULT name=narrow_over_cells question=Q-E97-5 arm=acct_wide_19B old_over_cells=0 old_over_grids=none new_over_cells=0 new_over_grids=none
```

| Quantity | Old measurement basis (K = 25, one write plus one delete) | New measurement basis (one generation, write only, no delete) |
|---|---|---|
| Q-E97-1 wide read t=1024 / d=64 | 227 901 440, exceeds the bound | **9 076 736, within the bound** (42%, 422‰) |
| Q-E97-2 wide read t=8192 / d=8 | 257 261 568, exceeds the bound | **10 240 000, within the bound** (47%, 476‰) |
| Q-E97-3 wide-read cells exceeding the bound, out of four | 3 (1024x64, 8192x8, 8192x64) | **1 (8192x64, 72 564 736, 337%)** |
| Q-E97-4 tightest narrow-read cell | 11 419 648 (8192x64), 53% (531‰) | **458 752 (8192x8 and 8192x64 tied), 2% (21‰)** |
| Q-E97-5 narrow-read cells exceeding the bound | 0 | 0 |

**Failure clauses (section 10 of the registration)**: FC1 **triggered** — D5:145 "at `1024 × 64` the wide-read arm exceeds 1% of `T_dirty`" does not hold on the E97 side under the new measurement basis; FC2 not triggered (one wide-read cell still exceeds the bound: 8192x64); FC3 **triggered** — "the tightest cell takes 53% of the budget" is 2% under the new measurement basis; FC4 not triggered (all four narrow-read cells within the bound under the new measurement basis); FC5 not triggered (geometry sensitivity below); FC6 not triggered (both judged cells judged "within the bound" alike). These are all legitimate results and void nothing; whether the clauses change is decided by the main agent.

**Q-E97-6, old and new side by side per cell**: one `name=cell_side_by_side` line for each of the 32 cells (artifact lines 74–81 are the eight cells of the judged arm). The four key arms have the same distribution of cells exceeding the bound; counted by arm (command and verbatim output):

```
$ grep ' name=cell_side_by_side ' research/results/e97-entry-encoding-single-generation-2026-10-20.out | sed 's/.* geometry=\([^ ]*\).*reading=\([a-z]*\).* old_over=\([01]\).* new_over=\([01]\).*/\1 \2 old_over=\3 new_over=\4/' | sort | uniq -c
      4 acct_e73_22B narrow old_over=0 new_over=0
      1 acct_e73_22B wide old_over=0 new_over=0
      2 acct_e73_22B wide old_over=1 new_over=0
      1 acct_e73_22B wide old_over=1 new_over=1
      4 acct_tight_14B narrow old_over=0 new_over=0
      1 acct_tight_14B wide old_over=0 new_over=0
      2 acct_tight_14B wide old_over=1 new_over=0
      1 acct_tight_14B wide old_over=1 new_over=1
      4 acct_wide_19B narrow old_over=0 new_over=0
      1 acct_wide_19B wide old_over=0 new_over=0
      2 acct_wide_19B wide old_over=1 new_over=0
      1 acct_wide_19B wide old_over=1 new_over=1
      4 acct_windowgen_7B narrow old_over=0 new_over=0
      1 acct_windowgen_7B wide old_over=0 new_over=0
      2 acct_windowgen_7B wide old_over=1 new_over=0
      1 acct_windowgen_7B wide old_over=1 new_over=1
```

On every arm the cells that flip are the two wide-read cells 1024x64 and 8192x8; wide read 8192x64 exceeds the bound under both bases, and all four narrow-read cells stay within the bound under both (command and verbatim output):

```
$ grep ' name=cell_side_by_side ' research/results/e97-entry-encoding-single-generation-2026-10-20.out | grep 'old_over=1 .*new_over=0' | sed 's/.* geometry=\([^ ]*\).* trees=\([0-9]*\) devs=\([0-9]*\) reading=\([a-z]*\).*/\1 \2x\3 \4/'
acct_e73_22B 1024x64 wide
acct_e73_22B 8192x8 wide
acct_wide_19B 1024x64 wide
acct_wide_19B 8192x8 wide
acct_tight_14B 1024x64 wide
acct_tight_14B 8192x8 wide
acct_windowgen_7B 1024x64 wide
acct_windowgen_7B 8192x8 wide
```

**Geometry sensitivity (section 8 of the registration)**: on all six leaf geometries both judged cells are within the bound under the new measurement basis, so the judgement does not depend on the leaf geometry (`judged_cell_verdict_differing_geometries=0`); the percentage of the tightest narrow-read cell is 1% on the 14B and 7B arms and 2% on the others (a reading, not a verdict). Copied whole from lines 231–237 of the same artifact:

```
E7RESULT name=geometry_sensitivity geometry=acct_e73_22B leaf_fanout=542 wide_1024x64_new_over=0 wide_8192x8_new_over=0 narrow_tightest_new_percent=2 narrow_tightest_new_permille=23
E7RESULT name=geometry_sensitivity geometry=acct_wide_19B leaf_fanout=603 wide_1024x64_new_over=0 wide_8192x8_new_over=0 narrow_tightest_new_percent=2 narrow_tightest_new_permille=21
E7RESULT name=geometry_sensitivity geometry=acct_tight_14B leaf_fanout=740 wide_1024x64_new_over=0 wide_8192x8_new_over=0 narrow_tightest_new_percent=1 narrow_tightest_new_permille=17
E7RESULT name=geometry_sensitivity geometry=acct_windowgen_7B leaf_fanout=1087 wide_1024x64_new_over=0 wide_8192x8_new_over=0 narrow_tightest_new_percent=1 narrow_tightest_new_permille=12
E7RESULT name=geometry_sensitivity geometry=S1_today_header_19B leaf_fanout=601 wide_1024x64_new_over=0 wide_8192x8_new_over=0 narrow_tightest_new_percent=2 narrow_tightest_new_permille=21
E7RESULT name=geometry_sensitivity geometry=S2_implementation_22B_entry34 leaf_fanout=477 wide_1024x64_new_over=0 wide_8192x8_new_over=0 narrow_tightest_new_percent=2 narrow_tightest_new_permille=27
E7RESULT name=geometry_dependence geometries=6 judged_cell_verdict_differing_geometries=0 narrow_tightest_percent_differing_geometries=2
```

S1 is the 19B arm with today's node header 96 (fanout 601), S2 is the implementation geometry of `crates/` (header 115, interval 2 × 22, entry 34, fanout 477). On S2, which has the smallest fanout, the two judged cells are 11 468 800 (53%, 534‰, artifact line 221) and 12 959 744 (60%, 603‰, line 223), the closest to the threshold among the six geometries; the former only shares the number 53% with the narrow-read cell under the old measurement basis, it is not the same quantity. Discrimination: the unit test `budget_judgement_is_strict_and_flips_when_the_threshold_moves_between_two_cells` moves the threshold to either side of 9 076 736 / 9 076 735, and judged cell one flips from within the bound to exceeding it; mutation FV11 (threshold changed to 9 076 735) is caught.

**Positive controls (section 5 of the registration, run on every arm)**: control A "K enters the formula", 8 cells on each of the four arms: K = 25 is strictly greater than K = 1 on all 8 cells, and K = 1 exactly equals ⌈groups / fanout⌉ × 16384 on all 8 cells; control C "the device dimension enters the group count": removing the device dimension from wide read 1024x64 strictly lowers all four arms (19B arm 9 076 736 → 311 296). The verdict line, copied whole (line 246):

```
E7RESULT name=verdict control_generations_in_formula_ok=true control_device_dimension_ok=true
```

**Voiding clauses (section 11 of the registration), none triggered**:

| # | How it is checked | Result |
|---|---|---|
| V1 old measurement basis reproduces the original artifact | The old-basis lines `acct_budget` 32 lines, `acct_key_ceiling` 8 lines and `acct_geom` 48 lines, with `_old_caliber` stripped, are compared line by line with `diff` against the same-named lines of the original artifact `e97-entry-encoding-2026-09-07.out` (archived: `git show 3cff909b0^:research/results/e97-entry-encoding-2026-09-07.out`) | No difference in any of the three groups (32 / 8 / 48 lines) |
| V2 anchors | The 48 `e97` anchor lines output by the independent script of class B in section 7 of the registration are compared cell by cell with the artifact's `cell_side_by_side` / `sensitivity_cell` on leaf fanout, group count, old and new dirty bytes, old and new judgement, and new permille (the comparison script reports 1 difference when fed one corrupted number) | 48 lines, 0 differences |
| V3 positive controls | Verdict line | Both `true` |
| V4 five rounds byte-identical | `cmp` | No difference in the other four rounds |
| V5 does not `use` `crates/` | The source only has `use e7_index_bench::Emitter;` | Not triggered |
| V6 mutations | Closing of `mutate.sh` | All 16 caught, anchors broken 0, invalid 0 |

**Incidental 2 (criterion 4 by × 1) and incidental 3 (tree height with K = 1)**: entry bytes of the 19B arm at wide read 1024x64 go 18 026 496 → 9 013 248; in the inverse, the wide-read 8192x64 cell still has no feasible point under the new measurement basis (`touched_entries=2670592 … max_entry_bytes=8 max_key_bytes=0 feasible=0`, artifact line 113), and the wide-read 1024x64 cell allows keys up to 56 bytes (line 109). Tree height: all 48 old-basis lines are narrow read 4300 entries / wide read 73600 entries, tree height 2 / 2; all 48 new-basis lines are narrow read 172 entries / wide read 2944 entries, **tree height 1 / 2** — segment width still does not move tree height, but under the single-generation measurement basis the narrow-read half of "tree height is always 2" is 1 (the command `grep ' name=acct_geom_new_caliber ' … | sed … | sort | uniq -c` outputs one line `48 narrow 172 h1 wide 2944 h2`).

**Source discrepancy (written as found, kb not changed)**: the E97 sentence in the basis of D5 (Snapshot / Space Accounting Mechanism) settled item 4 says "under the wide reading the tightest cell takes 53% of the budget", but that 53% is narrow read 8192x64 (`old_percent=53`, line 80). The sentence "at `1024 × 64` the wide-read arm exceeds 1% of `T_dirty`, all four narrow-read cells pass", written under E71 (Entry Count and Width of Accounting Keys) in the basis of D5 settled item 4, matches the four cells of E97 criterion 10. This time both readings are reported, one line each for Q-E97-1 and Q-E97-4.

**What it cannot answer**: a pure arithmetic geometric model that counts only leaves, not internal nodes; it does not model the step rows of the inode number watermark (kept with the root, at most one or two more dirty leaves); the statistic table is the eleven-item table of 2026-09-03 and it does not answer the numbers under today's twelve-item table (the conclusion can only be stated as "per that eleven-item table, under the new row-count measurement basis"); it does not answer the row set of 3 + 6d rows per publish of today's `crates/` (incidental 4, not run once sufficient to judge); it does not answer criteria 2, 3, 5–9 or the allocation record tree half (unrelated to the row-count measurement basis, not rerun); it does not answer wall clock.

### Paths and Conclusions Registry

| # | Path | How it computes | Source location | Shared items it reads |
|---|---|---|---|---|
| 1 | New harness, arithmetic under two measurement bases | For each cell, counts groups by reading, counts dirty leaves from the retained generation count and the leaf fanout (old measurement basis K = 25, new measurement basis K = 1), multiplies by 16 384 and compares with the threshold; covers all 32 registered cells (four key arms × four (t, d) pairs × two readings) and the 16 cells of the two geometries S1 and S2, not a sample | `scripts/experiment-apparatus/e97_entry_encoding_single_generation.rs:364 evaluate_cell` | The dimension marks of the eleven-item statistic table of 2026-09-03, the key width and entry width of the four key arms, node header 58, node 16 384, threshold 21 474 836, and the two retained generation counts (all copied from readings 1–3 of the rerun registration) |
| 2 | Independent anchor script | Computes 48 cells (six leaf geometries × four (t, d) pairs × two readings) directly from the formulas of readings 1–3 of the rerun registration, without reading the harness or the code in `crates/`; the 48 cells are all cells of the four registered (t, d) pairs on the six geometries, not a sweep over (t, d) | No implementation: the script is not in the repository; it sits in the draft directory of the pre-registration author, `~/.local/share/singlefs/work/m3-close-w8-c693-reruns/anchors.py` (sha256 `1cef0e2b…`), and its output is pinned as constants by the 48 anchor cases of the new harness (generated by the `anchor_test!` macro, for example `anchor_key22_header58_trees_1024_devices_64_wide`) | The same eleven-item table, key widths, node header, threshold and retained generation counts as row 1 (the same registered readings) |
| 3 | Original harness, old measurement basis | The original harness `dirty_bytes_per_publish` (K = 25); the 32 `acct_budget` cell lines produced by the 2026-09-07 run | `scripts/experiment-apparatus/e97_entry_encoding.rs:365 dirty_bytes_per_publish` | Eleven-item table, four key arms, node header 58, threshold; old retained generation count 25 |

The three paths share the same eleven-item table and the same threshold: if the table itself is not the one to be judged (today it has twelve items), all three go wrong together, so the figures in this section are stated only as "per the eleven-item table of 2026-09-03". Path 1 against path 2 is pinned in the 48 anchor cases (`assert_eq!` cell by cell); the 32 lines of path 1 against path 3 are pinned in the unit test `old_caliber_reproduces_the_archived_budget_lines` (the 32 lines of the original artifact copied as constants and compared verbatim).

### Influenced Decisions

| Decision item | Relationship | Retrospective |
|---|---|---|
| D5 (Snapshot / Space Accounting Mechanism) settled item 1 | Supports | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 changed: the basis sentence "the tree height of the four key widths is always 2" is replaced with the reading under the single-generation measurement basis: with the step row at level 0, the tree heights of the four widths are equal at each of the four registered grid points (narrow read 1, 2, 2, 2; wide read 2, 3, 3, 3), with no guarantee between grid points, and the cost is that the number of leaves written grows with the entry width (`research/prompts/m3-close-c693-r3-main-verification.md` section 2, cell T3) |
| D5 (Snapshot / Space Accounting Mechanism) settled item 4 | Supports | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 changed: the basis sentence now gives the readings under the single-generation measurement basis: the tightest narrow-read cell is the 19-byte arm at 2.1% (the 53% cell under the old measurement basis was narrow read `8192 × 64`), and in wide read only `8192 × 64` still exceeds the bound; the E97 (Entry Encoding of Accounting and Allocation Records) figures formerly written under E71 (Entry Count and Width of Accounting Keys) are no longer recorded in that row (`research/prompts/m3-close-c693-r3-main-verification.md` section 2, cell T3) |
| D5 (Snapshot / Space Accounting Mechanism) settled item 2 | No impact | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the 2026-10-20 rerun takes its "each root holds only this generation, each publish writes all of it once" as the input measurement basis and does not measure it |
| D3 (Space Allocation) settled item 7 | Supports | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the rerun on 2026-10-20 only recalculated criterion 10 and the associated criterion 4 and tree heights; the allocation record tree half was not rerun |
| D3 (Space Allocation) settled item 3 | Supports | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the rerun on 2026-10-20 only recalculated criterion 10 and the associated criterion 4 and tree heights; the allocation record tree half was not rerun |
| D5 (Snapshot / Space Accounting Mechanism) settled item 5 | Supports | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 changed: the basis sentence "the tree height of the four keys is always 2, and the cost curve is flat" is replaced with: with the step row at level 0, the tree heights are equal at the four registered grid points and the number of leaves written grows with the entry width; the tree-height sentence in the row of statistic type tags is replaced with one stated in terms of the total row count of the accounting tree (`research/prompts/m3-close-c693-r3-main-verification.md` section 2, cell T3) |
| D8 (Core Index Structure) settled item 2 | Preparation | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the 16 KiB node size was only an input; the rerun on 2026-10-20 did not measure it |
| D18 (Information Carried in a Block) settled item 9 | Preparation | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the rerun on 2026-10-20 did not change placement encoding |
| D22 (Synthesizing Unit Atomicity) settled item 7 | No impact | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the rerun on 2026-10-20 did not change child-pointer widths; the tree-height group used 32 / 53 as in the original apparatus |
| D4 (Checksum Location) settled item 5 | No impact | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the rerun on 2026-10-20 did not change checksums |
| D19 (Block Pointer Structure & Width Budget) settled item 4 | Supports | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the rerun on 2026-10-20 did not change the device segment width or the placement coordinate system |
| D16 (Publish Semantics) settled item 5 | No impact | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: `T_dirty` was only the source of the threshold (1% = 21 474 836); the rerun on 2026-10-20 did not measure it |
| D9 (Encryption) settled item 2 | No impact | 2026-10-21 unaffected: the body only gained short names for numbered items and replaced location references and sentences containing old values; numbers, conclusions, and artifacts are unchanged; 2026-10-20 unaffected: the rerun on 2026-10-20 did not change the MAC width |

## Historical Versions

### 2026-09-03

#### (Part 1)
- Design registration, criteria and failure clauses hardcoded prior to run. Basis: D5 (Snapshot / Space Accounting Mechanism) settled item 5 and
  D3 (Space Allocation) settled item 7 both judged "modifies new-pool file creation bytes: yes", addressing same geometric problem.

#### (Part 2): Two Criteria and Two Dimensions Supplemented After First Three-Way Argument Round, Rerun
- **Formerly**: Criteria only 1..6; generation inverted pairs tested only on 32 / 64 generations, placement granularity hardcoded to 32768 (data unit size),
  criterion 4 derivation evaluated only on `t=8192 / d=8` cell.
- **Now**: Added criterion 7 (alignment representability) and criterion 8 (wraparound period); generation count swept to 70 000, modulo swept to 65536;
  placement granularity swept across 16384 / 32768; criterion 4 derivation swept four cells × two readings. Unit tests 15 -> 19, mutations 10 -> 13.
- **Refutation basis**: First round adversarial attack leg (opus) cited D18 (Information Carried in a Block) settled item 9 verbatim:
  "data units align to 32768, **index nodes align to 16384**", directly disproving "encoding placement by 32 KiB unit numbers" —
  placements for half of index nodes could not be written. Main agent verified via lines 501–502 of `decisions/18-块里携带什么信息.md`.
  Same leg noted: modulo 256 on 32 / 64 generations yielded constantly 0 inverted pairs, exempting 1-byte generation;
  and criterion 4 derivation missed `t=8192 / d=64` cell, which **has no feasible point** under wide read.
  All three held upon recomputation, **none bypassed by altering criteria: criteria 1..6 unchanged, only 7 and 8 added**.

#### (Part 3): Two More Criteria Added, One Metric Corrected, Two Arms Added After Second Three-Way Argument Round, Rerun

- **Formerly**: Criterion 8 calculated generation wraparound by `T_time` = 5 s; criterion 4 budgeted by entry bytes;
  placement granularity hardcoded to 32768, with proposed 16 KiB encoding **lacking any arm**;
  `seq` absent from segment manifest.
- **Now**: Criterion 8 revised to E44 (Host Testing and Cost of Sequence Number Bit Width) measured 2785 publishes/sec
  (modulo 256 changed from "21 minutes" to **91 ms**); added criterion 9 (manifest completeness) and criterion 10 (budget by dirty nodes);
  placement granularity parameterized, adding `16 KiB slot number one per slot` and `16 KiB slot number + 1-byte span one per unit`
  arms. Unit tests 19 -> 22, mutations 13 -> 16.
- **Refutation basis (second round adversarial attack leg, opus, verified item-by-item by main agent)**:
  ① Rider clause of D5 (Snapshot / Space Accounting Mechanism) settled item 2 verbatim states "generation advances with publish, **not with 5-second window**"
  and "generation segment width >= 48 bits" — **correct measurement basis was written directly above cited clause, while first version differed by ~14000x**,
  and ">= 48 bits" was already settled, not a discovery of this experiment (verified via
  `decisions/05-快照-空间记账机制.md` lines 218–222);
  ② D8 (Core Index Structure) write buffer hard requirement 3 verbatim states "entries must carry sequence numbers (seq), **this is a format requirement**",
  yet absent from segment manifest, while criterion 2 checks only listed segments (verified via `decisions/08-核心索引结构.md` line 42);
  ③ Accounting tree hangs under root via COW => budget must be computed by dirty 16 KiB nodes, entry byte basis is systematically underestimated;
  ④ Proposed 16 KiB encoding had zero rows in artifact.
  Same leg proposed "one per unit + 1-byte span segment" alternative, which **was strictly better along two axes** upon rerun.
  ⚠️ **Criteria 1..8 unchanged, only 9 and 10 added, and criterion 8 measurement basis corrected.**

#### (Part 4): Third Three-Way Argument Round Removed Artifact — "Half Unit Unrepresentable" is Arm Tag, Not Encoding Property

- **Formerly**: `half_unit_representable` calculated by `!per_unit && grain < unit`,
  where `per_unit` was a hardcoded arm declaration parameter having no relationship to `off_b` / `span_b` / `grain`
  => artifact showing "one per unit => half unit unrepresentable" was a **hardcoded model artifact**.
  Conclusion drawn was "one per unit is strictly better along two axes".
- **Now**: Revised to evaluate coordinate resolution `grain < unit`. Both 16 KiB candidates **are judged** "representable".
  Yields a firmer conclusion: **"can represent all legal starting points" and "half a unit cannot be represented" are mutually exclusive**,
  because span 1 must remain permanently legal (an index node is a 16 KiB slot).
  => Sole remaining disparity is capacity footprint, which per `.claude/rules/fs-design.md` constitutes no criterion
  => **E97 (Entry Encoding of Accounting and Allocation Records) is insufficient to settle D3 (Space Allocation) settled item 7, achieving only elimination and narrowing.**
- **Refutation basis**: Third round adversarial attack leg (opus) highlighted code structure verbatim, verified by main agent in source;
  added mutation `M18_half_unit_criterion_falls_back_to_arm_tag` caught by unit tests, mutations 16 -> 17.
