## E96 Consistency of Hybrid Architecture — Run (2026-09-03, 10 Unit Tests / 12 Mutations All Caught): When Unit Header Has Only 5-Tuple, Snapshot Read Silently Reads New Version, While Header with Block Birth Has Constant 0; Release Must Traverse Map Not Hint; Migration 3-Step Truncation Does Not Add Misreads

[中文](96-混合架构的一致性.md) · **English** · [日本語](96-ハイブリッドアーキテクチャの一致性.ja.md)

**Problem**: The position authority form recommended by argumentation materials of the six unsettled items in D26 (Background Compaction and Placement Reclamation) at that time was a **hybrid architecture** — position entries in tree pointers (D19 (Block Pointer Structure & Width Budget) settled item 1) demoted to **hints**, while central map of "logical identity → placement" (form settled on encryption side in D9 (Encryption) settled item 5) unconditionally exists and serves as **authoritative**; reads take hints, verify unit header, fall back to map on mismatch; compaction modifies only map, leaving snapshot trees and deadlist untouched. The user asked: does it have any consistency problems? E94 (Touch Surface of Moving Shared Placement Points) only measured costs, not consistency.

Four dangers conceived before running were formed into four criteria: hints landing on **reused slots** after expiration (unit header 5-tuple — D18 (Information Carried in a Block) settled item 3 — lacks block birth, so new and old versions of same (inode, offset) share identical 5-tuple); release path mistakenly treating hints as authoritative; mid-flight crash during migration transactions; and conservation between map and occupancy table.

### Model

Inheriting persistent COW tree (L=4096, F=16), snapshot rotation, and destruction cascade of E94 (Touch Surface of Moving Shared Placement Points), adding:

| Component | Model |
|---|---|
| Leaf entry | (version, placement hint); hint refreshed only when entry is rewritten, compaction does not modify it |
| Central map | version → placement, authoritative; compaction modifies only here |
| Unit header | Two arms: `hdr_key` (only 5-tuple ⇒ modeled as key) / `hdr_birth` (5-tuple + block birth ⇒ modeled as (key, birth)) |
| Read path | Hint-first: returns if occupant unit header of hinted slot matches version to read; otherwise falls back to map with extra hop |
| Release | When destroying snapshot, deadlist entries resolve placement via map (correct resolution); additionally calculating how many times "resolving placement via hint" erroneously releases (negative arm, counting only, not applied) |
| Allocation | Two arms: `alloc_lowest` (lowest empty slot) / `alloc_lifo` (most recently vacated reused first) — neither targeted at key |
| Compaction | From scan cursor, take N=64 occupied slots and move to new slots, looping back after scanning a full circle |
| defer | Slots released in a round are reusable only in next round (D16 (Publish Semantics) settled item 15) |

Workload: 64 updates + 1 batch compaction + read all keys of all roots at round end per round, 200 rounds, snapshot established every 8 rounds, retaining 4; two sets: `uniform` (uniform random keys) and `adversarial` (half updates hit keys just moved — making "same key new version reusing old slot" reachable, form of control akin to E31 (AAD Lacks Snapshot Dimension)).

### Criteria (Hardcoded Before Run 2026-09-03) and Verdicts

| # | Criterion | Verdict |
|---|---|---|
| 1 | Manual calculation anchor: Snapshot → move key 3 (slot 3 → 8) → update key 3 reusing slot 3 → read snapshot key 3: `hdr_key` misread 1, `hdr_birth` misread 0 extra hop 1 | **Passed** (unit test) |
| 2 | Conservation: For each version referenced, `occupant(map(v)) == v`, map and occupancy are inverse functions of each other | **Passed** (40 configurations × 200 rounds audited round by round) |
| 3 | Positive control: Corrupt map ⇒ audit turns red; swap occupancy ⇒ misread count sounds | **Passed** (unit test, including construction catchable only by reverse verification) |
| 4 | `hdr_key` misread count exactly equals count independently scanned of "hinted slot occupied by another version of same key"; `hdr_birth` constant 0 | **Passed** (equal round by round; `hdr_birth` constant 0 across 40 × 200 rounds) |
| 5 | Erroneous release count when resolving placement via hint > 0; constant 0 via map | **Passed** |
| 6 | Truncate once at each of three migration steps: **does not add** misreads, conservation constantly holds | **Passed** (8 configurations each with three steps, difference constantly 0; measurement basis clarification see revision record) |
| 7 | Extra-hop rate over rounds (reporting numbers only) | See measurement |

### Actual Measurement (5 Seeds)

#### 1. Unit header has only 5-tuple: Snapshot read silently reads new version (criterion 4)

| Unit Header | Allocation | Workload | 200-Round Total Misreads (5 seeds) | Round 200 Misreads |
|---|---|---|---|---|
| `hdr_key` | Lowest empty slot | Uniform | **65 / 0 / 72 / 0 / 69** | 0 |
| `hdr_key` | LIFO | Uniform | 66 / 48 / 40 / 128 / 57 | 0 |
| `hdr_key` | Lowest empty slot | Adversarial | 4052 / 3925 / 3992 / 4503 / 4326 | 22–31 |
| `hdr_key` | LIFO | Adversarial | 4581–5259 | 17–23 |
| `hdr_birth` | Any | Any | **0 (all 20 runs)** | 0 |

- **Uniform random workload with lowest empty slot allocation — the cell least favorable for reproduction — also produces misreads** (3 out of 5 seeds non-zero). Every single occurrence was a snapshot read obtaining **new** content of the same (inode, offset) without any error reported: hinted slot reused by new version of same key, 5-tuples identical word for word, read path treats hint as hit.
- Under adversarial workload ~20 such entries alive at round end (`wrong_last`), accumulating four to five thousand over 200 rounds.
- With block birth added to header, under identical workload, identical hints, and identical reuse, **not a single misread occurred** — mismatches all fell back to map (extra hop rate see item 3).
- E85 (Field Table of Unit Headers) long ago measured "settled field set cannot determine block birth", and E31 (AAD Lacks Snapshot Dimension) measured that 5-tuple is blind to snapshot dimension; E96 (Consistency of Hybrid Architecture) connects both to compaction: **position hint + 5-tuple header = silent misreads on snapshot reads after compaction**.

#### 2. Release must traverse map, not hint (criterion 5)

| Workload | Erroneous releases in 200 rounds when resolving placement via hint (releasing slots actively occupied by other versions) | Via Map |
|---|---|---|
| Uniform | 4861–5123 times | **0** |
| Adversarial | 6003–6158 times | **0** |

Hint carried by deadlist entry is the value in leaf at kill instant, expiring once compaction moves it thereafter; if treated as placement when destroying snapshot, what is released is someone else's slot. => **No release path may treat hints as authoritative** — this is a hard rule of hybrid architecture.

#### 3. Read cost of expired hints (criterion 7)

| Workload | Round 100 Extra-Hop Rate Live Root / Snapshot Roots | Round 200 Live Root / Snapshot Roots |
|---|---|---|
| Uniform | ~30% / 37% | 29–31% / 35–37% |
| Adversarial | ~15–16% / 25% | 19–21% / 27–30% |

Live root hint refreshed only when that key is rewritten, snapshot hints never refreshed; over 200 rounds compaction cursor scans through occupied region approximately once => 30% of reads require an extra hop. **This is the shape of read cost in hybrid architecture**: proportional to how much compaction has moved and how old snapshots are, not proportional to snapshot count K (in E94 (Touch Surface of Moving Shared Placement Points), cost under pointer authority scaled with K).

#### 4. Mid-flight crash during migration does not add misreads (criterion 6)

One migration = ① write new replica → ② publish map → ③ defer release old slot. 8 configurations truncated and recovered after each of three steps (uncommitted allocations after ① rolled back; old slot released by defer after ②): **misread difference constantly 0, conservation constantly holds**. Mechanism: prior to ② map still points to old replica and old replica still exists; after ② when hint lands on old slot occupant is still the same version, reading itself — neither side has a window where "map points to non-existent entity".

### Model Revision Records (All occurred prior to formal round admission, criteria 1–5, 7 untouched)

1. **Compaction batch changed from "lowest 64 slot numbers" to scan cursor**: Initial version took lowest empty slot as destination, which spun in place at disk bottom, raising extra-hop rate to 95% over 200 rounds — that was an artifact, not read cost of hybrid architecture.
2. **Criterion 6 measurement basis clarified as difference**: The world of `hdr_key` arm originally contained misreads from hints landing on same-key reused slots, and judging by absolute 0 before and after truncation attributed them to crashes; changed to "truncation does not add misreads" (under `hdr_birth`, difference 0 is absolute 0).
3. **Added allocation policy dimension**: Under lowest empty slot, adversarial workload over 24 rounds could not create same-key reuse (there were always lower empty slots in front of old slots); adding LIFO (most recently vacated reused first, form used by many allocators, untargeted at key) made danger reachable at scale; admission round reported both policies, with uniform workload + lowest empty slot cell still non-zero.

### What It Cannot Answer

1. 32-bit ciphertext checksum in position entry (D19 (Block Pointer Structure & Width Budget) settled item 2) can also probabilistically detect reuse, not built in model — it only turns misreads of `hdr_key` arm from "inevitable" to "2⁻³² probability", differing in dimension, calculated separately.
2. No latency axis: extra-hop rate is frequency proportion, not latency.
3. Tampering surface on encryption side (plaintext mapping layer, MAC not covering position) outside model.
4. Persistence and reconstruction of mapping layer itself (C34 (three-way clash on plain text mapping layer states), E28 (Can Plaintext Mapping Rebuild Solely from Self-Describing Units)) cited per established conclusions, not reverified.
5. Live root hint refresh policy has only "refresh on rewrite"; "opportunistically refresh hint on read mismatch" modifies leaves outside snapshot tree, arm not built.
6. Absolute counts cannot be transferred to trees with fanout 119 plus message buffers (same limitation as E94 (Touch Surface of Moving Shared Placement Points)).

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e96_hybrid_consistency.rs` (`cargo run --release --bin e96-hybrid-consistency`), raw output `e96-hybrid-consistency-2026-09-03.out` (50 lines, ending `emitted=50`).
- **N=5 rounds byte-for-byte consistent** (same md5). **10 unit tests, 12 mutations hit entry by entry, 0 blind spots** (round 1 blind spots supplemented with failing unit tests: updates must refresh own hints, LIFO must reuse most recently vacated first, audit reverse check must fail when lengths equal), mutation table `scripts/mutation/tables/e96_hybrid_consistency.tsv` (`bash scripts/mutation/mutate.sh e96-hybrid-consistency scripts/experiment-apparatus/e96_hybrid_consistency.rs scripts/mutation/tables/e96_hybrid_consistency.tsv`).
- Hooked into `scripts/experiment-apparatus/replay.sh` (exact tier).

### Affected Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D19 (Block Pointer Structure & Width Budget) settled item 5 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D26 (Background Compaction and Placement Reclamation) settled item 5 | Provisioning | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 3 | Provisioning | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) settled item 5 | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D16 (Publish Semantics) | Unaffected | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E96 (Consistency of Hybrid Architecture) are consolidated in [experiments-history.md](../experiments-history.md).
