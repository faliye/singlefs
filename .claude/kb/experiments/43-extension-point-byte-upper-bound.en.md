## E43 Extension Point Byte Upper Bound — Run (2026-08-30, N=5 Rounds Byte-Identical; Record Header 311 → 487 Rerun 2026-10-04): Failed to Bracket a Binding Upper Bound; Upper Bound A Aborted, Upper Bound B Is a Soft Guideline

[中文](43-扩展点字节上限.md) · **English** · [日本語](43-拡張ポイントのバイト上限.ja.md)

**The question asked**: What fixed byte upper bound to choose for D21 (Boundary Between Authoritative and Derived States) settled item 2 "custom space fixed byte upper bound".

<!-- doc-lint:registry name-col=2 -->

| Number | Short Name |
|---|---|
| Q43.1 | Header width did not leak into other cells |
| Q43.5 | Mount evaluation grid |

⚠️ **The 128 does not count as evidence**: D21 (Boundary Between Authoritative and Derived States) text conceded "example in body is 128 bytes, **unsettled**"—
it was casually offered when rejecting "setting bound proportionally", **with no boundary calculated on either side**.
Same shape as E40 (Ciphertext Checksum Bit Width): single-legged derivation with arbitrary input.

### Verified Premises (2026-08-29 Audited Against kb Body, Verbatim)

| Source | Verbatim |
|---|---|
| D21 (Boundary Between Authoritative and Derived States) settled | Extension point size **declared in system configuration per track** (defined by line opener, can be 0); **upper bound takes fixed bytes, not proportional to unit size** |
| D21 (Boundary Between Authoritative and Derived States) settled | Extension point holds **pointers**, and **may only point to space within that extension point's own quota**, forbidden to point to core-layer objects |
| D21 (Boundary Between Authoritative and Derived States) comparison table | This project 158.0 B: 4 KiB units 3.857%, 128 KiB units 0.121%; ZFS 128.0 B; btrfs 82.5 B / 207.3 B; 4+2 stripe tier 202.0 B (calculated since 2026-09-06 on 105-byte data unit header; 91-byte version was 144.0 / 188.0, 108.0 / 152.0 at initiation) |
| Destination of D21 (Boundary Between Authoritative and Derived States) 158 bytes | Unit header 105 (`format-const: DATA_UNIT_HEADER_BYTES`, D18 (Information Carried in a Block) settled item 7) + pointer header 31 (MAC 16 + nonce 12 + algorithm 1 + extent offset 2) + position entry ×2 total 22 (each: dev 1 + physical offset 6 + ciphertext checksum 4)|
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) Corollary 2 | "Unit" **does not distinguish metadata and data** |
| E29 (Blast Radius of a Single Failed Node) measured | 16M leaves, 40-byte pointers: 2 KiB nodes fanout 49 tree height 5; 4 KiB fanout 100 height 4; 16 KiB fanout 408 height 3; 64 KiB fanout 1636 height 3 |

⚠️ **One distinction absent in D21 (Boundary Between Authoritative and Derived States) that E43 (Extension Point Byte Upper Bound) must run in both tiers**:
text only states "reserve a segment in units", **never specifying whether units like index nodes carry extension points**.
Literal reading of D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) "unit does not distinguish metadata and data" implies **all carry them**.
⇒ Model runs two tiers: **all units carry** and **data units only carry**. If upper bounds differ,
that fork itself must be settled first, with conclusions stated as "depends on which tier", forbidden to report just one.

### What to Measure, Criteria, Failure Clauses

- **What to measure** (three segments, purely arithmetic and counting model without file system):
  1. **Upper bound 1, space comparison**: Scan `N ∈ {0, 16, 32, 64, 128, 256, 512}` × units ∈ {4, 16, 64, 128} KiB,
     computing metadata bytes per unit and ratio, compared on same basis against ext2 / ext4 / btrfs / ZFS.
     ⚠️ **Both accountings must be reported**: **including** extension point in "overhead paid to manage a unit", and **excluding** it (treated as line-opener payload).
  2. **Upper bound 2, fanout and tree height**: Extension point eats available node bytes ⇒ fanout drops.
     Calculate fanout and tree height under each `N` using E29 (Blast Radius of a Single Failed Node) measurement basis (16M leaves, 40-byte pointers).
  3. **Lower bound**: Extension point must at least accommodate **one legal pointer + its quota boundary**,
     position entry width derived independently from fields settled in D19 (Block Pointer Structure & Width Budget) (dev 1 + physical offset 6, plus length and boundary descriptor).
     `N` below this value ⇒ extension point structurally cannot hold any pointer ⇒ **feature effectively non-existent**, whereas D21 (Boundary Between Authoritative and Derived States) settled its form as "holding a pointer".
- **Criteria**: Each segment gives a boundary; take their **intersection**.
  - **Comparison crossover**: `N` causes metadata bytes on 128 KiB units to exceed ZFS 128.0 B or btrfs 207.3 B ⇒
    D21 (Boundary Between Authoritative and Derived States) text "this project saves vs ZFS on 128 KiB units, saves half vs btrfs" **no longer holds**.
    ⇒ Either narrow `N` inside crossover, or rewrite that statement clarifying which accounting was changed.
  - **Tree height**: `N` must not cause tree height in any unit tier to be **one level higher** than at `N = 0`.
  - **Lower bound**: `N ≥` minimum usable value computed in segment 3.
  - When upper bounds across segments conflict, **take the tightest**, specifying which segment constrains it; **forbidden to average or cherry-pick the looser one**.
- **Positive control** (**run once across each of four unit tiers, not just tier 1**): In each tier, raise `N` to half that tier's unit size;
  ratio and fanout must change significantly and match independently calculated values—otherwise model is not factoring extension point in, **entire round aborted**.
- **Failure clauses**:
  - Positive control detects no change ⇒ **implementation flawed, entire round aborted**.
  - Intersection of three segments is empty ⇒ **legitimate outcome, faithfully recorded**: conclusion is "unsettlable under current basis",
    what must change is measurement basis or D21 (Boundary Between Authoritative and Derived States) comparison statement, **forbidden to change the accounting scheme to legitimize 128**.
  - Divergent conclusions between "all units carry" and "data units only carry" ⇒ faithfully record both, returning fork to D21 (Boundary Between Authoritative and Derived States).
- ⚠️ **Must include absolute value assertions** (segments are inter-arm comparisons; relative checks cannot catch all arms wrong together):
  - `N = 0` tier must reproduce D21 (Boundary Between Authoritative and Derived States) table verbatim: 158.0 B, 4 KiB 3.857%, 128 KiB 0.121%, 4+2 stripe 202.0 B (tracks active comparison table figures).
  - Fanout at `N = 0` must equal 49 / 100 / 408 / 1636 of E29 (Blast Radius of a Single Failed Node), tree height must equal 5 / 4 / 3 / 3.
  - Ratio for each `N` computed independently via `(158 + N) / unit_bytes`, **forbidden to read back from tested code**.

### ⚠️ The Half It Cannot Answer

**"How many bytes line openers actually need" has no host origin**—this project lacks any line opener line.
E43 (Extension Point Byte Upper Bound) answers only two things: given `N`, what is the cost; how large `N` must at least be to avoid being purely decorative.
It **does not answer** "whether `N` is sufficient". ⇒ Conclusion must be written as intervals and conditional expressions,
with those conditions cited together when settling D21 (Boundary Between Authoritative and Derived States) settled item 2.
Same shape as E40 (Ciphertext Checksum Bit Width): missing corruption rate there, missing requirements here.

⚠️ **Candidate slot width of root slot tier mismatched active root record width in `crates/`** (discovered during 2026-09-24 rerun at header 311, `e43-r1-prereg.md` section 3.1):
harness used `ROOT_SLOT_CANDIDATE = 256` (D22 (Synthesizing Unit Atomicity) settled item 2 candidate width), whereas active `ROOT_RECORD_BYTES = 466` in `crates/singlefs-format/src/lib.rs`.
This delta is unrelated to header width, unadjusted this round—self-witness unit criterion in E43 (Extension Point Byte Upper Bound) clamps the journal record header side (25 < 255); root slot tier was already aborted ("255 computed for root slot tier is aborted"),
but candidate 257 itself is not identical to active root record size in implementation; no answer exists for which follows which, left to main agent.

### ⚠️ What It Does Not Cover: Five Hard Constraints

Five hard constraints of D21 (Boundary Between Authoritative and Derived States) (checksum coverage, checker ignores content, size declared in format,
pointers respect project quota, no pointers to core-layer objects) **are entirely outside E43 (Extension Point Byte Upper Bound) scope**—
they are testable properties on images, landing on [checks-owed.md](../checks-owed.md) C21 (Extension point pointer escaping accounting);
prerequisites are transaction layer, accounting implementation, extension point format finalization, and checker. **E43 (Extension Point Byte Upper Bound) answers only the byte count question.**

### Prerequisites

None. Pure arithmetic and counting model, independent of file system and QEMU. Fanout segment reuses model of E29 (Blast Radius of a Single Failed Node).

### Actual Measurement (2026-08-30)

**Criterion was swapped once.** At initiation, E43 (Extension Point Byte Upper Bound) took "saving vs ZFS / btrfs" as source of upper bound,
whereas **"saving vs competitors" is not a criterion of this project** (user settlement;
`.claude/rules/fs-design.md` "do not sacrifice self-containment to save space" agrees, default answer being not saving).
⇒ Upper bound derived from that comparison axis (19 bytes) **was deleted along with its rationale**—
not merely "aborted due to circular reasoning", **the cited criterion is fundamentally rejected by this project**.

**Remaining boundaries are entirely unrelated to saving space**:

| Boundary | Value | What It Converts into "Cannot Mount" |
|---|---|---|
| **Structural upper bound**: Payload squeezed to 0 | 4 KiB units **3990**, 128 KiB units **130966** (on 105-byte unit header); index nodes must also fit one pointer ⇒ 2 KiB nodes **1944**, 64 KiB nodes **65432** | "Cannot fit", not "waste" |
| **Self-witness unit upper bound**: Root slots and journal record headers lack parent pointers with checksums; atomic width = runtime-probed `physical_block_size` (D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) Corollary 3)| Journal record header 487 bytes in 512 sector leaves **25**, in 4096 leaves **3609** (matches D23 (Journal Role & Format) settled item 4 verbatim: "leaving 25 bytes thereafter (cannot fit one named item of 56 bytes); leaves 3609 on 4096"; artifacts `hdr=487 atomic=512 room=25`, `hdr=487 atomic=4096 room=3609`); root slot tier **cannot provide machine-independent constant** (255 from candidate slot width 256 aborted, see E43 (Extension Point Byte Upper Bound) limitations) | Header cannot span atomic width ⇒ tear undetectable |
| **Tear isolation**: Only one slot permitted per atomic unit | Candidate slot width 256 + atomic width 512 ⇒ fits **2**, on 4096 fits **16**; positive control raising slot width to 512 ⇒ drops to **1** | Writing one slot may tear adjacent slot (E34 (Root Ring Slot Geometry) Claim 1)|
| **Lower bound** | Interval **[7, 13]**: placement 7 / with length 9 / with ciphertext checksum 11 / all needed 13 | Cannot fit one legal pointer ⇒ feature non-existent |
| Tree height (tier where every unit carries extension point)| 864 | **Converts to nothing**—tree height is derived performance property, remaining a soft guideline |

⇒ **If self-witness units also carry extension points, tightest bound lands on journal record header, not index node 864**—
34.6× tighter (864 ÷ 25), with rationale unrelated to saving space.
⚠️ **Upper bound of 25 in this tier is smaller than 128 example in D21 (Boundary Between Authoritative and Derived States)** (artifact `n_max_if_self_witness_carries=25`): if self-witness units carry extension points, 128 cannot fit; but D21 (Boundary Between Authoritative and Derived States) settled item 4 settled that self-witness units do not carry them, and settled item 2 removed this check, placing this cell on the removed branch.
⚠️ **E43 (Extension Point Byte Upper Bound) had zero coverage of this tier in round 1**: that tier modeled index nodes, not root slots or journal headers.

### Evaluated Once at Mount

**Settled (2026-08-30, user settlement): Unit geometry computed dynamically, but computed once at mount and looked up at runtime.**

E43 (Extension Point Byte Upper Bound) packages structural bound, self-witness bound, and tear isolation into a mount evaluation function returning **"mounts / cannot mount", not "runs slower"**—
this is `.claude/rules/fs-design.md` "convert misuse into mount failure" manifested on this item.
Inputs are all **quantities constant during operation** (declared N in configuration, unit and node sizes in format,
probed `physical_block_size`), satisfying rule "branch variables must not change mid-operation".
Empirically, all four rejection reasons **are reachable**: payload is 0 / node cannot fit one pointer /
more than one slot in an atomic unit / self-witness header cannot span an atomic width.

⚠️ **Its cost must be recorded**: identical declaration (N = 0, slot width 512) returns `ok` on 512-byte atomic width devices,
returning `reject_slot_shares_atomic` on 4Kn devices (unit test pins N = 0).
After header 487, N = 128 on 512 evaluates first to `reject_self_witness_overflow` (headroom only 25), no longer the pair of "`ok` on 512, rejected on 4Kn":
two rows of N = 128 in 18 rows of `mount_eval` with `atomic=512` and slot width 512 or 4096 flipped from `ok` to `reject_self_witness_overflow`
(`research/results/e43-ext-budget-2026-10-04-h487.out`).
⇒ **Mountability of an image depends on which machine it is plugged into.** This is not a defect—rejecting mount is precisely the safe failure mode—
but must be documented, as "refusing to mount when moved to another machine" appears like a malfunction in the field.

### Three-Way Argumentation Leg Status (Round 2, After Premises Revised)

| Leg | Stance | Unique Output | Verdict |
|---|---|---|---|
| Cloud A | Forward | Provided **structural upper bound** and "convert misuse into mount failure" form; noted four core-layer costs (checksum coverage, generic tool skip, ENOSPC reservation, post-mortem reclamation) **none turn into failures with N**; noted D21 (Boundary Between Authoritative and Derived States) settled item 4 must precede item 2 | Accepted, rewritten question |
| Cloud B | Reverse | **Struck "costs scale linearly and small with N"**: self-witness atomic width is space-independent upper bound already in kb; cost shape is **cliff**, not linear; noted E43 (Extension Point Byte Upper Bound) had **zero coverage** of self-witness units | Narrowed |
| Local | Counterexamples | Insisted upper bound must exist with step at **atomic boundary** | Rejected |

**Main agent verdict: Adopted backward leg narrowing + forward leg form + user settled "compute once at mount".**

- **Corrected forward leg formula**: it wrote "108 + N ≥ unit size ⇒ payload is 0" (108 was metadata total per unit at the time). **Wrong**—
  D4 (Checksum Location) settled "parent stores child checksum" ⇒ in that total, only **unit header** resides in unit,
  pointer's 53 bytes (header 31 + position entry 22) reside in **parent node**. Container constraint deducts unit header segment,
  currently **105** (`format-const: DATA_UNIT_HEADER_BYTES`), total **158**; numbers at the time were 55 and 108.
- **Local leg specific counterexample did not hold**: argued extension points push unit over sector boundaries—extension points reside **inside** units,
  not altering unit size; its other point (tree height deepening) is a performance argument, while index is derived state.
  **However, its direction was correct**: step is indeed at atomic boundary, located on self-witness unit tier. Retested.
- Both cloud legs **independently pointed out same fact**: D21 (Boundary Between Authoritative and Derived States) settled item 4 must be settled first, dictating whether tightest bound falls on self-witness units or index nodes.

### Measurement Basis and Known Limitations

- Code `scripts/experiment-apparatus/e43_extension_point_budget.rs`
  (`cargo run --release --bin e43_extension_point_budget`), **active artifact
  `e43-ext-budget-2026-10-04-h487.out`** (145 lines, closing line `emitted=145`,
  registered in `scripts/experiment-apparatus/replay.sh`); earlier reruns noted in dated ⚠️ notes
  (first being 2026-08-30); header 311 artifact `e43-ext-budget-2026-09-24-h311.out` and earlier copies archived out of working directory per "each commit deletes previous experiment log" (deleted in commit 31128273, auditable via `git log -- research/results/e43-ext-budget-2026-09-24-h311.out`).
  ⚠️ **2026-08-31 rerun applied two changes simultaneously** (artifact `e43-ext-budget-2026-08-31.out`, 128 lines, `emitted=128`): ① source constant (then `JOURNAL_HDR`, now `JOURNAL_HEADER_BYTES`) had stayed at 84 while D23 (Journal Role & Format) settled item 9 raised `jsn` to 10 bytes on 2026-08-30;
  ② `tail_lsn` 8 bytes removed same day with D23 (Journal Role & Format) settled item 3 ⇒ **record header 10 fields became 78 bytes** (D23 (Journal Role & Format) settled item 4).
  Together changed 3 reported lines (`room` 428→434, 4012→4018, `n_max_journal_512` 428→434),
  **`mount_verdict` lines completely unchanged**—desynchronization flipped zero mount verdicts.
  2026-08-30 artifact `e43-ext-budget-2026-08-30.out` preserved verbatim for evidence chain.
  ⚠️ **2026-09-02 third rerun** (artifact `e43-ext-budget-2026-09-02.out`,
  128 lines, `emitted=128`, earlier copies preserved): unit header changed from 55-byte assumption to
  **91-byte initial value** of D18 (Information Carried in a Block) settled item 7 (flagged by C89 (Leftover Recomputation from Closing the Unit Header)).
  Structural upper bound 4040→**4004**, 131016→**130980**; baseline 108.0→**144.0**, stripe tier 152.0→**188.0**;
  **`mount_verdict` lines completely unchanged**. Faithfully recorded two items: "saving vs ZFS" count dropped from 19 to **0**
  (144 > 128, both tiers more expensive than ZFS at N=0—comparison was not a criterion anyway, D21 (Boundary Between Authoritative and Derived States)
  2026-08-30 user settlement); original mutation M5 (`<` to `<=`) became **equivalent** under new constant (both comparisons yield 0),
  replaced by M5_payload_underdeducts_one_byte per `.claude/singlefs-ai-sop/rules/test-discipline.md` "record equivalent mutations separately",
  equivalence pinned by assertion `counted_upper_bound_from_the_zfs_clause_is_zero_now`.
  ⚠️ **2026-09-06 fourth rerun** (artifact `e43-ext-budget-2026-09-06.out`,
  128 lines, `emitted=128`, earlier three preserved): unit header calculated as **105** bytes settled in C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild)
  (`format-const: DATA_UNIT_HEADER_BYTES` = 91 + write sequence 10 + payload CRC 4).
  Desynchronization mechanism: source constant was `IN_UNIT_HEADER_BYTES`, differing from registered name ⇒ `.claude/gate.d/code-source-discipline.sh` format-constants check failed to recognize it, 91 lingering from 2026-09-05 to 2026-09-06 (described in C56 (Unregistered Format Constants Still Drift), observed 2026-09-06).
  Structural bound 4004→**3990**, 130980→**130966**; baseline 144.0→**158.0**, stripe tier 188.0→**202.0**;
  68 of 128 lines changed, **18 `mount_eval` verdicts verbatim unchanged**.
  ⚠️ **Order inversion appeared this round**: on `unit == node == 4096`, payload bound 3990 became 2 bytes tighter than pointer bound 3992
  ⇒ cell could no longer reach `reject_no_pointer`, `every_rejection_reason_is_reachable` moved sample to "128 KiB unit + 4 KiB node",
  pinning ordering with assertion (`maximum_extension_point_bytes_keeping_payload_positive(4096) + 2 == maximum_extension_point_bytes_keeping_one_pointer_in_node(4096)`).
  **21 unit tests all green; 19 mutations caught** (three with constants in mutation table—baseline metadata, stripe baseline, unit header width—updated synchronously,
  last changing to "reset header width to 91").
  ⚠️ **2026-09-13 fifth rerun** (artifact `e43-ext-budget-2026-09-13.out`,
  128 lines, `emitted=128`, earlier four preserved): journal record header calculated as **95** (ten fields 78 + three settled increments 17 of D23 (Journal Role & Format) settled items 7 / 8 / 13,
  D23 (Journal Role & Format) settled item 4 registered value is 95, C94 (Registered Format Constants Disagree with Later Rulings)). 3 lines changed (`room` 434→417, 4018→4001, `n_max_journal_512` 434→417),
  **`mount_verdict` verdicts verbatim unchanged**; 21 unit tests all green, 19 mutations caught (`e43-mutate-2026-09-13.log`).
  ⚠️ **2026-09-13 sixth rerun** (artifact `e43-ext-budget-2026-09-13-hdr277.out`,
  128 lines, `emitted=128`, earlier five preserved): journal record header calculated as **277**—
  ten fields 78 + three settled increments 17 of D23 (Journal Role & Format) settled items 7 / 8 / 13
  + new root segment **182** of settled item 15 (tree table unit pointer 83 + central mapping tree root pointer 83 + tree ID water level 8 + rollback floor F 8,
  2026-09-13 user settlement). 3 reported lines changed, copied from `e43-ext-budget-2026-09-13-hdr277.out`:
  `E7RESULT name=self_witness kind=journal_record hdr=277 atomic=512 room=235 slots_per_atomic=NA`,
  `E7RESULT name=self_witness kind=journal_record hdr=277 atomic=4096 room=3819 slots_per_atomic=NA`,
  `E7RESULT name=self_witness_bound n_max_journal_512=235 n_max_root_slot_256=255 n_max_if_self_witness_carries=235`
  (`room` 417→235, 4001→3819, `n_max_journal_512` 417→235);
  **18 `mount_eval` verdicts verbatim unchanged** (zero diff on `name=mount_eval`).
  ⚠️ **Clamping side of self-witness unit tier shifted**: `n_max_if_self_witness_carries` changed from **255** to **235**—
  at header 95, `min(417, 255)` clamped by root slot 255; at header 277, `min(235, 255)` clamped by journal record header.
  Unit test renamed `the_self_witness_bound_is_235_not_864`, arithmetic rewritten as `min(512 − 277, 256 − 1)`.
  **21 unit tests all green; 19 mutations caught** (`e43-mutate-2026-09-13-hdr277.log`);
  **mutation table unchanged**—none of 19 rules referenced 95 / 417 / 4001 / 255;
  rule targeting self-witness headroom (M16) targeted formula `atomic - header_bytes`.
  ⚠️ **2026-09-14 seventh rerun** (artifact `e43-ext-budget-2026-09-14-round2.out`,
  128 lines, `emitted=128`, earlier six preserved): journal record header calculated as **307**—
  ten fields 78 + three settled increments 17 of D23 (Journal Role & Format) settled items 7 / 8 / 13
  + new root segment **188** of settled item 15 (tree table unit pointer 86 + central mapping tree root pointer 86 + tree ID water level 8 + rollback floor F 8,
  both pointers +3 on 2026-09-14) + **fsid 8** + **MAC 16** (both 2026-09-14 user settlement).
  3 lines changed, copied from `e43-ext-budget-2026-09-14-round2.out`:
  `E7RESULT name=self_witness kind=journal_record hdr=307 atomic=512 room=205 slots_per_atomic=NA`,
  `E7RESULT name=self_witness kind=journal_record hdr=307 atomic=4096 room=3789 slots_per_atomic=NA`,
  `E7RESULT name=self_witness_bound n_max_journal_512=205 n_max_root_slot_256=255 n_max_if_self_witness_carries=205`
  (`room` 235→205, 3819→3789, `n_max_journal_512` 235→205);
  **18 `mount_eval` verdicts verbatim unchanged** (zero diff on `name=mount_eval`—
  declared N takes only 0 / 128 / 512, 128 not exceeding bound on both 205 and 235, 512 exceeding on both).
  Clamping side remains journal record header (`min(205, 255) = 205`).
  Unit test renamed `the_self_witness_bound_is_205_not_864`, arithmetic rewritten as `min(512 − 307, 256 − 1)`.
  **21 unit tests all green; 19 mutations caught** (`e43-mutate-2026-09-14-round2.log`);
  **mutation table unchanged**, same reason as sixth rerun.
  Reproduction passes E43 (Extension Point Byte Upper Bound) number to `scripts/experiment-apparatus/replay.sh`.
  ⚠️ **2026-09-24 eighth rerun (header 307 → 311, issue list line 1, `e43-r1-prereg.md`)**: journal record header calculated as **311**—
  D23 (Journal Role & Format) settled item 4 added "intra-publish sequence number 4 bytes" on 2026-09-24; ten fields 78 + three settled increments 17 + intra-publish sequence 4 + new root segment 188 + fsid 8 + MAC 16 = 311.
  Artifact `e43-ext-budget-2026-09-24-h311.out` (145 lines, `emitted=145`; previous `e43-ext-budget-2026-09-14-round2.out` archived per commit discipline;
  prerun baseline compared byte-by-byte rather than against old artifact):
  `E7RESULT name=self_witness kind=journal_record hdr=311 atomic=512 room=201 slots_per_atomic=NA`,
  `E7RESULT name=self_witness kind=journal_record hdr=311 atomic=4096 room=3785 slots_per_atomic=NA`,
  `E7RESULT name=self_witness_bound n_max_journal_512=201 n_max_root_slot_256=255 n_max_if_self_witness_carries=201`
  (`room` 205→201, 3789→3785, `n_max_journal_512` 205→201);
  **18 `mount_eval` verdicts verbatim unchanged** (zero diff on `name=mount_eval`—
  N takes 0 / 128 / 512, 128 inside bounds on both 201 and 205, 512 exceeding on both).
  Clamping side remains journal record header (`min(201, 255) = 201`).
  Unit test renamed `the_self_witness_bound_is_201_not_864`, arithmetic rewritten as `min(512 − 311, 256 − 1)`.
  **Added Section 8 geometric sensitivity** (`.claude/rules/mutation-sampling.md` category 6): `H` sampled across `{256,257,258,307,311,384,385,499,500,503,504}` (512 tier) and `{307,311,3584,3585,3968,3969}` (4096 tier), adding 17 lines `name=header_sensitivity_512` / `name=header_sensitivity_4096`; 307 and 311 fell on same side of flip point in every cell, validating discriminative power.
  **25 unit tests all green (4 new: A3 headroom items, A4 header breakdown sum, B4 sensitivity pairwise check, discriminative power self-test); mutation table 19 → 22 rules (added M20 header to 307, M21 header to root slot line, M22 sensitivity 128 strictly less); ran before/after constant change, all 22 caught, 0 invalid, 0 un-red** (`e43-mutate-before.log` 19/19, `e43-mutate-after.log` 22/22).
  Q43.1 (Header width did not leak into other cells): diff of two artifacts restricted to `name=self_witness` (2 lines), `name=self_witness_bound`, 17 lines of `name=header_sensitivity_*`, `name=done` `emitted=`, inside permitted set.
  Q43.2–Q43.5 (Mount evaluation grid): values identical across all five cells ⇒ **issue list line 1 recorded "unchanged"**—self-witness upper bound still clamped by journal header, still fitting all four lower bounds and D21 (Boundary Between Authoritative and Derived States) example 128; 18 mount verdicts verbatim unchanged.
- **27 unit tests all green** (`e43-unit-test-2026-10-04-h487.out`); mutation table `scripts/mutation/tables/e43_extension_point_budget.tsv` 24 rules (M23, M24 added for header 487), 22 rules caught at header 311 (`e43-mutate-after.log`, 2026-09-24, archived); re-running 24 rules after header 487: **caught 24, invalid 0, un-red 0** (log `e43-mutate-2026-10-04-h487.log`, see section "2026-10-04: Record Header 311 → 487 Rerun").
  ⚠️ **Do not write "zero blind spots"** (noted in 2026-08-30 round 2 attack): mutations cover only **functions with written assertions**,
  proving "every asserted code location was mutation tested", not "all code covering conclusions was tested".
- **Positive control**: Four unit tiers and four node tiers each raised to half; all ratios rose, all fanouts dropped,
  fanout motionless only in data-units-only tier; slot width raised to atomic width, slots per atomic unit dropping to 1.
- **All integer arithmetic**, ratio in 0.001% units. Verification path recomputed same cells with f64, matching cell by cell,
  with verification path proven to turn red.
- ⚠️ **Two craftsmanship lessons**: ① Constant assertions written as **subtraction** overflowed at compile time under mutation,
  counted as "invalid mutation" rather than "caught"—both turned red after rewriting as addition;
  ② Constants used only in tests left `never used` warnings, whereas
  `.claude/singlefs-ai-sop/rules/command-safety.md` notes warnings are cheapest failure signals, forbidden to ignore.
- ⚠️ **Lower bound 11 / 13 tiers are circular inputs** (noted in 2026-08-30 attack round):
  contain 4-byte ciphertext checksum settled in D19 (Block Pointer Structure & Width Budget) settled item 2.
  D19 (Block Pointer Structure & Width Budget) settled item 2 was settled as 32-bit on 2026-08-30 (E40 (Ciphertext Checksum Bit Width) measured, 3.37× margin) ⇒ tiers have determined inputs;
  **re-opening conditions inherited**: if field corruption exceeds 3.37e-9, that item re-opens and these tiers move accordingly.
- ⚠️ **255 computed for root slot tier is aborted**: derived from candidate slot width 256 minus 1,
  whereas E41 (Root Ring Slot Geometry 2×4×256) rejected slot width 256. **Retained self-witness bound is journal record header tier alone** (25 / 3609, at header 487).
- ⚠️ **Mount evaluation tier lacks one input**: self-witness rule in `mount_verdict` takes effect **unconditionally**,
  answering for D21 (Boundary Between Authoritative and Derived States) settled item 4 that "self-witness units carry extension points";
  and tear isolation takes no starting offset, returning "mounts" for counterexample "512 wide slot offset 256" measured in E41 (Root Ring Slot Geometry 2×4×256).
- ⚠️ **"Byte-identical across 5 rounds" proves "no hidden state", not statistical stability** (noted in 2026-08-30 attack round 2):
  E43 (Extension Point Byte Upper Bound) is a **deterministic model** (no real I/O, no concurrency, no clock input); identical binary ran 5 times is inherently byte-identical
  ⇒ N=1 and N=5 provide identical information.
  `.claude/singlefs-ai-sop/rules/test-discipline.md` "single observation does not count" targets observations with **intrinsic variance**,
  which does not apply to E43 (Extension Point Byte Upper Bound). **Forbidden to use N=5 to impersonate empirical stress-test evidence strength.**
- **Cannot answer "whether N is sufficient"**: project lacks line opener lines, demand side lacking host source.
- **Fixed fields in root slot unwritten in kb**, so root slot tier provides **upper bound estimate** (slot width − 1), not true headroom.
- **Does not verify "independent disk start and end failure domains"**, nor verifies `physical_block_size` probing itself.
- **Node geometry in tree height tier borrows parameters from E29 (Blast Radius of a Single Failed Node)**, moving under alternative parameters;
  structural and self-witness bounds **do not depend** on those parameters.

### 2026-10-04: Record Header 311 → 487 Rerun (Issue List Line 4, Rerun Registration `e43-r2-prereg.md`)

D23 (Journal Role & Format) settled item 4 changed record header from 311 to 487 (new root segment of settled item 15 from 188 to 364: added instance table pointer 86, allocation record tree root pointer 86, unmount token 4); `JOURNAL_HEADER_BYTES` in harness lingered at 311 (sweep in `sweep-311-188-67.md`). Rerun registration followed header 311 (`e43-r1-prereg.md`) hardcoding criteria, only updating two evaluated values (311 and 487) and sampling points adjacent to old value; baseline ran before modifying harness (harness 311, `crates/` 487, registered exception for baseline only, producing no conclusions), byte-identical with `e43-ext-budget-2026-09-24-h311.out`.

Modifications: constants and comments updated to 487 / new root segment 364; 6 unit tests updated (headroom 25 / 3609, 0 / 64 named items, breakdown sum 487, bound 25, N=0 pinning "same declaration mounts in one place, rejected in another", N=25 / 26 pinning pair adjacent to new headroom); added 2 unit tests; sensitivity sampling point 307 changed to 487; criteria, thresholds, and arms untouched (registration section 12). New artifact `research/results/e43-ext-budget-2026-10-04-h487.out` (145 lines, `emitted=145`, line counts match), E43 (Extension Point Byte Upper Bound) row in `scripts/experiment-apparatus/replay.sh` updated, rerun "byte-identical".

Comparison with header 311 artifact (`cmp` reports differences from line 96; `diff … | grep -c '^[<>]'` counts 14 lines). Different lines transcribed from both artifacts:

```
E7RESULT name=self_witness kind=journal_record hdr=487 atomic=512 room=25 slots_per_atomic=NA
E7RESULT name=self_witness kind=journal_record hdr=487 atomic=4096 room=3609 slots_per_atomic=NA
E7RESULT name=self_witness_bound n_max_journal_512=25 n_max_root_slot_256=255 n_max_if_self_witness_carries=25
E7RESULT name=mount_eval atomic=512 slot=512 n=128 unit=4096 node=4096 verdict=reject_self_witness_overflow
E7RESULT name=mount_eval atomic=512 slot=4096 n=128 unit=4096 node=4096 verdict=reject_self_witness_overflow
E7RESULT name=header_sensitivity_512 h=487 room512=25 bound=25 binding=journal fits_lb9=1 fits_lb13=1 fits_128=0 self_witness_n128=overflow
E7RESULT name=header_sensitivity_4096 h=487 room4096=3609 self_witness_n128=ok self_witness_n512=ok
```

Corresponding lines in header 311 artifact were `hdr=311 … room=201`, `room=3785`, `n_max_if_self_witness_carries=201`, two lines `verdict=ok`, with two sensitivity lines for `h=307` (`room512=205`, `room4096=3789`) vanishing; `name=done emitted=` was 145 in both.

| Cell | Verdict (Header 487) | Header 311 Run | Changes |
|---|---|---|---|
| Q43.1 (Header width did not leak into other cells) | Different lines inside permitted set (self-witness 2 lines, `self_witness_bound`, two `mount_eval` lines, one line each in two sensitivity tables) | — | Passed |
| Clamping side for self-witness upper bound | `n_max_journal_512=25` less than `n_max_root_slot_256=255`: journal side | 201 < 255: journal side | Unchanged |
| Self-witness upper bound fits lower bound 9 | `n_max_if_self_witness_carries=25` ≥ `n_min_quota=9`: fits | Fits | Unchanged |
| Self-witness upper bound fits lower bound 13 | 25 ≥ `n_min_full=13`: fits | Fits | Unchanged |
| **D21 (Boundary Between Authoritative and Derived States) example 128 fits** | 25 < 128: **cannot fit** | 201 ≥ 128: fits | **Changed** |
| **Q43.5 (Mount evaluation grid) 18 lines** | N=128 with `atomic=512` and slot 512 / 4096 are `reject_self_witness_overflow`; remaining 16 lines unchanged | Those two lines were `ok` | **Changed (2 lines)** |

Headroom 3609 on 4096; self-witness bound tighter than segment 2 bound by 864 ÷ 25 = 34.56× (4.2985× at header 311). P1 (header knob read): `room=` delta on `atomic=512` is 201 − 25 = 176. P0 (original positive control): `sha256sum` of `name=poscontrol_*` identical (`5a2e3152…ac92`). Section 8 sensitivity: `h=311` line `room512=201 … fits_128=1 self_witness_n128=ok`, `h=487` line `room512=25 … fits_128=0 self_witness_n128=overflow`, matching values evaluated in main artifacts; sampling point pairs across flip points (256/257/258, 503/504, 499/500, 384/385, 3584/3585, 3968/3969) all yielded differing verdicts.

Category breakdown count (single command, raw output):

```
$ sed -E 's/^E7RESULT name=([a-z_0-9]+).*//' research/results/e43-ext-budget-2026-10-04-h487.out | sort | uniq -c
      1 bounds
      1 config
      1 done
     56 geom
      8 geom_threshold
      6 header_sensitivity_4096
     11 header_sensitivity_512
      1 interval
     18 mount_eval
      4 poscontrol_geom
      1 poscontrol_slot
      4 poscontrol_space
      4 self_witness
      1 self_witness_bound
     28 space
$ grep -n 'name=verdict' research/results/e43-ext-budget-2026-10-04-h487.out
(no output, exit code 1: this artifact has no `name=verdict` lines, verdicts written in `verdict=` field of `mount_eval`)
```

**Issue list line 4 (E43 (Extension Point Byte Upper Bound)): artifact changed (14 lines differing), two verdict cells flipped (D21 (Boundary Between Authoritative and Derived States) example 128 fit, Q43.5 (Mount evaluation grid)), three cells unchanged (self-witness clamping side, fitting lower bound 9, fitting lower bound 13).** Two flipped cells fall on self-witness branch: D21 (Boundary Between Authoritative and Derived States) settled item 4 settles self-witness units carry no extension points, item 2 removed this check; `mount_verdict` continues evaluating day-of-run rules without altering arms.

27 unit tests all green (`e43-unit-test-2026-10-04-h487.out`). The main agent reran all 24 entries in the mutation table (this requires running in the background and waiting; it was not run during the artifact-producing stage), with log `e43-mutate-2026-10-04-h487.log`:

```
$ for s in '✅' '⏭' '❌' '💥' '⚠️' '⏱' '🧱'; do printf '%s=%s ' "$s" "$(grep -c "^$s" e43-mutate-2026-10-04-h487.log)"; done; echo
✅=24 ⏭=0 ❌=0 💥=0 ⚠️=0 ⏱=0 🧱=0
$ grep -E '^(计数：|已还原)' e43-mutate-2026-10-04-h487.log
计数：内存撞顶 0 条（上限 1G）、超时 0 条、排不上 0 条、被总上限挤掉 0 条
已还原，基线仍全绿
```

**Caught 24, invalid 0, un-red 0**, sum 24 matching 24 rows in table (`scripts/mutation/tables/e43_extension_point_budget.tsv`), full table completed. Registration hardcoded "M1–M19, M22 must match across both runs" (V4), un-run before harness update (section 12); V4 triggers only if a rule after change is not "caught"; all 20 rules plus M20, M21, M23, M24 were "caught" (`grep -E '^(✅|⏭|❌)' … | sed -E 's/^(\S+) +\[(M[0-9]+)_.*/ /' | sort -V` yielding one ✅ line each from M1 to M24) ⇒ **V4 not triggered**; header 311 basis was 22/22 in `e43-mutate-after.log` (archived); both header 311 and 487 runs were "all caught, invalid 0, un-red 0". M20, M21 (updated from `= 311;` to `= 487;`) and new M23 (header back to 311), M24 (self-witness overflow criterion as greater-equal) all caught: M20 back to 307, M21 to root slot line, M23 to 311 all turned red on `header_bytes_equals_the_decision_23_breakdown`. Invalid column did not grow (0), zero assertions disabled.

Reproduction:

```
bash scripts/experiment-apparatus/replay.sh E43
```

Byte-identical `e43-ext-budget-2026-10-04-h487.out`.

**What it cannot answer**: Rerun only answers whether artifact and conclusions change after header width takes 487, not whether D21 (Boundary Between Authoritative and Derived States) settled items 2 / 4 / 16 must update; "journal header tier (417 on 512)" in D21 (Boundary Between Authoritative and Derived States) settled item 2 body reflects earlier header generation, which is 25 under header 487, unjudged in E43 (Extension Point Byte Upper Bound).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D1 (Data Mobility / Reverse Index) settled item 3 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 2 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 3 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 4 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 7 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 8 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 14 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 15 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D21 (Boundary Between Authoritative and Derived States) settled item 16 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D4 (Checksum Location) | Does not affect | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D18 (Information Carried in a Block) settled item 7 | Does not affect | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D19 (Block Pointer Structure & Width Budget) | Does not affect | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) settled item 4 | Does not affect | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D23 (Journal Role & Format) settled item 4 | Does not affect | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |
| D22 (Synthesizing Unit Atomicity) | Does not affect | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-18 unaffected: previous artifact archived in repository (accessible via git history), body citation retains filename only; conclusion, numbers unchanged |

## Historical Versions

### 2026-09-14

- Journal record header 277→307 (D23 (Journal Role & Format) settled item 4 added fsid 8 and MAC 16;
  new root segment of settled item 15 grew from 182 to 188 with both pointers +3, 2026-09-14 user settlement) seventh rerun:
  self-witness headroom 235 / 3819 → **205 / 3789**, upper bound for "self-witness units carry extension points" 235 → **205**,
  18 mount verdicts verbatim unchanged; unit test renamed `the_self_witness_bound_is_205_not_864`.
  21 unit tests all green, 19 mutations caught, table unchanged.
  Basis: `format-const: JOURNAL_HEADER_BYTES = 307` in `.claude/kb/decisions/23-journal的角色与格式.md`, gate "format constants sync between kb and experiment source" turned red.
  Past: self-witness headroom 235 / 3819, upper bound 235.
  ⚠️ Concurrently updated source comment claiming "verbatim" whose text was gone: doc comment of `self_witness_room`
  cited D23 (Journal Role & Format) settled item 4 verbatim "takes 54% of 512 sector, leaving 235 bytes thereafter",
  unfound repo-wide; active text is "takes 60% of 512 sector, leaving 205 bytes thereafter (fitting 3 named items of 56 bytes); leaves 3789 on 4096 (67 named items)"; comment updated.

### 2026-09-13

- Journal record header 95→277 (D23 (Journal Role & Format) settled item 15 added 182-byte new root segment,
  2026-09-13 user settlement) sixth rerun: self-witness headroom 417 / 4001 → **235 / 3819**,
  18 mount verdicts verbatim unchanged; clamping side of "self-witness units carry extension points" upper bound switched from root slot (255)
  to journal header (**235**), test renamed `the_self_witness_bound_is_235_not_864`.
  21 unit tests green, 19 mutations caught, table unchanged (none referenced 95 / 417 / 4001 / 255).
  Basis: `format-const: JOURNAL_HEADER_BYTES = 277` in `.claude/kb/decisions/23-journal的角色与格式.md`, gate stage turned red.
  Past: self-witness headroom 417 / 4001, bound 255.

### 2026-09-06

- Unit header 91→105 (C113 (No Input to Current-Version Determination of Multi-Version Units in Scan Rebuild) settlement added write sequence 10 and payload CRC 4 to data units) fourth rerun:
  structural bound 4004 / 130980 → **3990 / 130966**, baseline 144.0 / 188.0 → **158.0 / 202.0**, 18 mount verdicts verbatim unchanged;
  ordering inverted (payload bound at 4 KiB tightened beyond pointer bound by 2 bytes), rejection reason sample cell moved, adding assertion pinning order.
  Basis: `format-const: UNIT_HDR_DATA` updated to 105 on 2026-09-05, while source called it `IN_UNIT_HDR`, unrecognized by gate, lingering for one round.
  Past: structural bound 4004 / 130980, baseline 144.0 / 188.0.

### 2026-09-02
- Unit header 55→91 (D18 (Information Carried in a Block) settled item 7 initial value) third rerun: structural bound and baseline repinned,
  zero mount verdicts flipped; "saving vs ZFS" count 19→0 recorded; M5 equivalent mutation replaced per discipline.
  Past: structural bound 4040 / 131016, baseline 108.0 / 152.0.

### 2026-08-29
- Document created. Source: D21 (Boundary Between Authoritative and Derived States) settled item 2. Initiation basis: unsettled item had zero experiment coverage
  (`grep -rn "扩展点\|自定义空间\|开线方" .claude/kb/experiments.md .claude/kb/experiments/ research/` empty),
  and 128 bytes in text was illustrative, explicitly unsettled.
