## E39 Does Reverse Chain Block Remnant Records —— Run (2026-08-29, N=5 rounds byte-identical)

[中文](39-反向链挡不挡得住残留记录.md) · **English** · [日本語](39-逆方向チェーンは残留レコードを阻止できるか.ja.md)

**What it asks**: Can the directional settlement in D23 (Journal Role & Format) settled item 8, "add a reverse chain to record header (hash of previous record)", **truly block remnant records of the previous timeline outside the legal prefix**?

⚠️ **Direction settled does not mean effectiveness is proven.** E32 (Remnants of the Previous Timeline) tested that the vulnerability holds (N=5 rounds byte-identical): remnant record checksum is intact, `jsn` happens to equal expected, and none of the three gates—I-8.3 (Strictly Continuous Replay Prefix), in-flight record count upper bound, and `checkpoint_txg`—trigger upon verification. **E39 (Does Reverse Chain Block Remnant Records) tests the patch, not the vulnerability.**

### What to Test, Criteria, Failure Clauses

- **What to test**: Add a reverse chain field onto the model from E32 (Remnants of the Previous Timeline), re-run the same "hole → recovery → small resume append → crash again" sequence, and observe whether records belonging to discarded timelines still appear in the replay prefix.
- **Criteria**:
  - **Effectiveness**: After adding reverse chain, the number of times records belonging to old timelines are accepted into the prefix must be **0**.
  - **No false kills**: Legal prefixes within the same timeline must be accepted **without missing a single record** (otherwise it achieves 0 simply by "accepting nothing").
  - **Width**: Sweep hash truncation width (32 / 64 / 128 bits), find the minimum width where "false acceptance rate remains 0", and provide collision probability arithmetic under that width.
- **Positive control**: Turn off the reverse chain field entirely; false acceptance count must return to the non-zero value in E32 (Remnants of the Previous Timeline)—otherwise "0" cannot distinguish whether the reverse chain is effective or the model simply failed to generate remnants.
- **Failure clauses**: If under any parameter combination false acceptance still occurs with reverse chain added, the directional settlement does not hold, **faithfully record and return to D23 (Journal Role & Format) settled item 8**.
- ⚠️ **Must include absolute value assertions**: The number of remnant records generated must be computed directly by construction, not read back from tested code—otherwise "0 false acceptances" might merely be due to zero remnants being generated.

### Measurement Basis Requirements

⚠️ **Must cover the two combinations not covered by E32 (Remnants of the Previous Timeline)**: That round only covered 2 / 4 combinations; E39 (Does Reverse Chain Block Remnant Records) must run all four, otherwise patch scope does not align with vulnerability scope.

### Host Testing (2026-08-29, N=5 rounds byte-identical; **10 mutation items all caught**)

#### Function: Open Chain 16 Cells All 0 Remnants, and No False Kills

2 slot mapping modes × 2 resume strategies × chain enabled/disabled × 4 width tiers:

| | Result |
|---|---|
| **Effectiveness** | Open chain **all 16 cells** `replayed_stale = 0` |
| **No false kills** | Same 16 cells all `lost_own = 0`, prefix is exactly `20 settled + 3 newly written = 23` —— **not achieving 0 by accepting nothing** |
| **Positive control** | Closed chain resuming from prefix end has all 8 cells `replayed_stale = 6` —— remnants indeed created |

#### Width: 8-bit Insufficient, 16-bit Marginal, Clean Starting from 32-bit

**Collision is a probabilistic event, one round cannot measure probability** ⇒ Each tier runs **2 million rounds of independent trials** (seed changed per round, judging "whether remnant duped reverse chain"):

| Width | False Acceptance / 2M Rounds | Equivalent ppm | Theoretical 2⁻ⁿ | Record Header 84 → (Run Measurement Basis) | Header Growth |
|---|---|---|---|---|---|
| **8-bit** | **7 922** | 3 961 | 3 906 | 85 bytes | +1.19% |
| **16-bit** | **33** | 16 | 15.3 | 86 bytes | +2.38% |
| **32-bit** | **0** | 0 | 0.0005 | 88 bytes | +4.76% |
| 64-bit | 0 | 0 | ~0 | 92 bytes | +9.52% |

**Actual measurements closely track theoretical value 2⁻ⁿ tier by tier** (3961 vs 3906, 16 vs 15.3) ⇒ this curve is extrapolatable, not a coincidence.

⚠️ **Header widths in the last two columns are the 84 bytes at run time on 2026-08-29**; current header without reverse chain is 492 (D23 (Journal Role & Format) settled item 4: header 496 contains reverse chain 4) ⇒ four tiers are 493 / 494 / 496 / 500 bytes, +0.20% / +0.41% / +0.81% / +1.63%. **Width selection unchanged**—chosen basis is the false acceptance columns, independent of header width.

⇒ **Interpretation**: 8-bit admits a record from another timeline once every 256 "hole → recovery → resume → crash again" sequences, **too few**; 16-bit is once every 65,000 times; 32-bit had 0 occurrences across 2 million rounds, theoretically once every 4.3 billion times.

#### ⚠️ "Header Growth Percentage" Is Not on-Disk Cost; Those Bytes Are Free

D23 (Journal Role & Format) settled item 4 settled "**record header falls completely within one atomic unit**" ⇒ on-disk records round up to atomic units. Factoring in the 9 bytes added in D23 (Journal Role & Format) settled item 7 settlement (txg 8 + commit flag 1):

| Chain Width | Header Total | Header Growth | **On-Disk Occupancy (512 Unit, 1 Item Named)** | **12 Items Named** | **Items Packed in One 512 Unit** |
|---|---|---|---|---|---|
| 2 bytes (16-bit) | 95 B | +2.38% | **512 B** | **1024 B** | **7** |
| 4 bytes (32-bit) | 97 B | +4.76% | **512 B** | **1024 B** | **7** |
| 8 bytes (64-bit) | 101 B | +9.52% | **512 B** | **1024 B** | **7** |

⚠️ **The claim "all three tiers are identical cell by cell" has been overturned (2026-08-30 struck by the backward leg of the three-way argument, main agent independent recomputation confirmed).**

E39 (Does Reverse Chain Block Remnant Records) only sampled two points: naming 1 item and 12 items, and **neither point happens to be a bifurcation point**. Sweeping 0..1200 items using E39 (Does Reverse Chain Block Remnant Records)'s own constants (`RECORD_HEADER_BYTES=84` + `TRANSACTION_FIELD_BYTES=9` = 93, `ITEM_BYTES=56`):

| Unit | 2 Bytes vs 4 Bytes | 4 Bytes vs 8 Bytes |
|---|---|---|
| 512 | **Differs on 19 item counts**, minimum is **items=44** (2560 B vs 3072 B, **difference 512 B / 20%**) | 0 places |
| 4096 | **Differs on 2 item counts** (items=364, 876) | 0 places |

Mechanism: `gcd(56, 512) = 8` ⇒ "identical cell by cell" depends solely on `total record header bytes mod 8`, **an artifact of header byte count in this version, not a structural property**.
⇒ **"Difference between 2 and 4 does not exist on disk" is literally false**; the correct formulation is
**"identical when items ∉ {44, 108, 172, …}"**.

⚠️ **Whereas D23 (Journal Role & Format) settled item 9 settlement (2026-08-30: `jsn` 8 → 10 bytes, header 84 → 86) happens to turn it true again**: after base becomes 95, 2 vs 4 and 4 vs 8 under both 512 and 4096 units have **bifurcation counts drop to zero** (recomputed via same arithmetic).
⇒ **This evidence holds under 2026-08-30 header width, but it holds because "header byte count happens to fall into that congruence class", while header byte count is still shifting** (the repo simultaneously contains multiple measurement bases including 84 / 86 / 93 / 95).
**It is forbidden to cite this again as "structurally free".**

⇒ **Between 16 and 32, the correct cost-side statement is**:
**Empty under 2026-08-30 header width, yet sensitive to header width**—shifting header by 2 bytes may reopen bifurcation.

#### Independently Reproduced Conclusion of E36 (Slot Mapping Dimension)

The model for E39 (Does Reverse Chain Block Remnant Records) was independently rewritten, **without using code from E32 (Remnants of the Previous Timeline) / E36 (Slot Mapping Dimension)**.
Decoupling slot and sequence number, and rewinding write head to breakpoint during recovery per jbd2 practice,
the decoupled arm's `replayed_stale` and modulo `jsn` arm are **equal cell by cell (both 6)**
⇒ **Decoupling alone does not confer immunity**, consistent with conclusion of E36 (Slot Mapping Dimension). This serves as a free cross-check.

#### Three Places Caught by Assertions and Mutations, Not Reviewed by Eye

1. **Absolute value assertion went red first**: Written assuming "generating 8 remnants should replay 8", measured **6**.
   The 3 records written by new timeline overwrote the hole slot plus 2 following remnants
   ⇒ **Surviving remnants = `stale − (new_after − 1)`**. **Model was not wrong; assertion undercounted overwrites.**
2. **Decoupled arm must rewind write head to breakpoint**: Without rewind, new records write behind remnants, and sequential scan cannot reach them
   ⇒ Measurement would report "new records all lost" (`lost_own = 3`) instead of "remnants replayed".
   ⚠️ **A model flaw can masquerade as a conclusion.**
3. **When removing "sequence number must equal expected", none of nine tests went red**—records read by other arms already had sequence numbers matching expected; that gate was redundant for them. Caught after adding a test creating "checksum intact but sequence number mismatched" via "decouple + skip breakpoint resume".

⚠️ **Hash itself must be proven to vary with seed.** Seed must be placed **at front** with finalizer:
When placed at end and truncating lower bits directly, FNV-1a lower-byte evolution is a **bijection** for each input byte
⇒ Appending same seed bytes to both ends makes lower byte equality **independent of seed**
⇒ 8-bit tier ran **0 false acceptances** (theoretical 7 813).
**That is not "8-bit suffices", that is failing to measure anything.**
Guarded by assertion `the_measured_false_accept_rate_tracks_two_to_the_minus_chain_bits` (measurement must closely track 2⁻ⁿ) and `M8_退回第一版哈希` in E39 (Does Reverse Chain Block Remnant Records) mutation table.
⚠️ **Second pitfall of same class**: Making seed global static caused concurrent unit tests to trample each other; "8-bit should yield hundreds" measured 3 times. **Seed is parameter, not global.**

**Measurement basis**: 64-slot ring, 20 settled, 21st is hole, followed by 8 remnants, new timeline appends 3; width section runs 2M independent trials per tier. Record header 84 bytes taken verbatim from 11 fields in E23 (Journal Geometry) (current 78: `tail_lsn` removed per D23 (Journal Role & Format) settled item 3, `jsn` widened by 2 per settled item 9; source code and artifacts retained under run-time measurement basis).
Code `scripts/experiment-apparatus/e39_back_chain.rs`
(`cargo run --release --bin e39_back_chain`),
raw output `e39-back-chain-2026-08-29.out`,
mutation table `scripts/mutation/tables/e39_back_chain.tsv`.
**13 unit tests, 10 mutation items in `scripts/mutation/tables/e39_back_chain.tsv` caught one by one.**
⚠️ **"0 blind spots" retracted (2026-08-30)**: Mutations only prove **functions covered by written assertions** go red; they cannot prove "assertion quantifiers are written correctly". A concrete counterexample exists within E39 (Does Reverse Chain Block Remnant Records) itself—
`two_four_and_eight_byte_chains_cost_the_same_on_disk` only took items ∈ {1,7,8,12,16}, while the bifurcation point is at items=44; **not one of the 10 mutation items touches that quantifier**.

### ⚠️ What It Cannot Answer

- **False acceptance rate is probability of "one chance per incident", not frequency "per second".** Denominator is number of occurrences of "hole → recovery → small resume append → crash again" sequence; E39 (Does Reverse Chain Block Remnant Records) does not answer how often it occurs.
  ⇒ Inferring "how often failures occur" from ppm requires providing that frequency separately.
- **It only tested one collision form**: Hash of last record in new timeline colliding with `previous_hash` stored in first living remnant record. Real implementation contains other comparison points (e.g. checker reverse traversal), not covered by E39 (Does Reverse Chain Block Remnant Records).
- **It does not answer which hash algorithm to choose.** Model uses FNV-1a + splitmix64 finalizer; empirical tracking of 2⁻ⁿ shows it is sufficiently random at this scale, **but that does not equate to suitability as that on-disk format field**.

### Prerequisites

None. Model from E32 (Remnants of the Previous Timeline) already in repo; adding one field suffices.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) settled item 8 | Supports | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-29
- Documented and ran. Source: D23 (Journal Role & Format) settled item 8 directional settlement (user) explicitly requested "test it".
- Conclusion: All three criteria passed, directional settlement holds. Width sweep yielded empirical curve: 8-bit 3961 ppm, 16-bit 16 ppm, 32-bit 0 (tracking 2⁻ⁿ tier by tier).
- ⚠️ First-version hash placed seed at end and truncated low bits directly; width section **measured nothing** (8-bit 0 false acceptances, masquerading as conclusion); changed to seed at front + finalizer, added assertion "measurement must track 2⁻ⁿ" and corresponding mutation.
- Concurrently independently reproduced E36 (Slot Mapping Dimension) "decoupling alone does not confer immunity".
