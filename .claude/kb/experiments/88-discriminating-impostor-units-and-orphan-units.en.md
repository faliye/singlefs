## E88 Discrimination of Impostor Units vs Orphan Units — Run (2026-09-02, Arithmetic Exhaustion, 10 Unit Tests / 11 Mutations All Caught): Overturn Clause Not Triggered——47-Byte Header Misclassifies 12 (Claim On) / 21 (Claim Off), Two Riders fsid and Birth Generation Stand; Claim Rule Proven Sole Defense for Self-Mirror Cell

[中文](88-冒名单元与孤儿单元的判别.md) · **English** · [日本語](88-なりすましユニットと孤児ユニットの判別.ja.md)

**It is the falsification check issued by the backward leg itself in the adversarial round of C68 (Invariant Allowlist Conflicts with Settled Clauses)**
(C89 (Leftover Recomputation from Closing the Unit Header) item 3): The backward leg persuaded the verdict to return fsid and birth generation to the unit header
(D18 (Information Carried in a Block) settled item 7), explicitly noting "if the 47-byte header also classifies all these fixtures correctly,
the two riders fsid and birth generation shall be overturned back". E88 (Discrimination of Impostor Units vs Orphan Units) turns that statement into a runnable model.

### What to Test

Counting model (pure arithmetic, zero I/O): a volume of disk placements + allocation record set + root checkpoint_txg;
the scanner walks all probe points per the 16384 stepping of E86 (Scan Stride and Two Unit Sizes), classifying point by point.
Classification basis varies by arm:

| Arm | Header Fields | Claim Rule |
|---|---|---|
| `hdr91_claim` | 91-byte initial value: includes fsid + birth generation (settled form of D18 (Information Carried in a Block) settled item 7) | On (skips payload internal probes across whole unit once claimed) |
| `hdr91_noclaim` | 91-byte initial value: includes fsid + birth generation | Off |
| `hdr47_claim` | 47 bytes: no fsid, no birth generation (original proposed form prior to adversarial round) | On |
| `hdr47_noclaim` | 47 bytes: no fsid, no birth generation | Off |

**Fixtures (quantity and positions of each class hardcoded in source constants before run)**:

| Fixture | Construction | Correct Classification |
|---|---|---|
| Live units x37 | Checksum correct, birth generation <= root txg, allocation record present | Admitted Level 2 (double proof of D18 (Information Carried in a Block) settled item 5) |
| Lost-account live units x6 | Checksum correct, birth generation <= root txg, but allocation record lost | Admitted Level 1 (read-only), **clearing forbidden** |
| Torn orphans x9 | Birth generation > root txg, no allocation record (written by uncommitted publish during power loss) | **Cleared** (positive classification: no published state can reference future generation) |
| Misdirected impostors x3 | Legitimate unit from external volume landing entirely on **placement with allocation record** (different fsid, genuine magic / checksum) | Rejected (impostor) |
| Payload-embedded foreign images x5 | External volume image stored as file in this volume, with its internal unit headers landing precisely on **probe points inside live unit payloads** (worst-case alignment) | Not admitted (skipped by claim, or rejected by fsid) |
| Payload-embedded self-images x4 | Backup image of this volume stored as file, **same** fsid, header landing on probe point inside payload | Not admitted (**defended solely by claim rule**——fsid cannot discriminate self) |
| Corrupted checksum x2 | Header bytes damaged | Rejected (all arms must reject) |

### Criteria (Written Hard Before Run)

1. **`hdr91_claim` arm must be completely correct** (nailing absolute values): Level 2 exactly 37, Level 1 exactly 6, cleared exactly 9,
   impostor rejected exactly 3, nested admitted exactly 0, corrupted checksum rejected exactly 2. Any cell missed ⇒ settled form itself contains holes, faithfully record.
2. **Overturn clause**: If misclassifications on `hdr47_claim` arm are also 0 ⇒
   **the two riders fsid / birth generation overturn back, reopening those two fields in D18 (Information Carried in a Block) settled item 7**.
   Misclassifications = impostor admitted Level 2, torn orphan not cleared (mixed into Level 1 with lost-account), nested admitted >=1, sum of three classes.
3. **Discriminative power of claim rule** (nailing absolute values): On `hdr91_noclaim` arm, self-images must **admit exactly 4**
   (identical fsid cannot block own images)——proving claim rule is sole defense for self-image cell, not decorative;
   on `hdr91_claim` arm exactly 0.
4. **Do not conflate two failure classes**: Torn orphans cleared and lost-account live units admitted Level 1 must be **reported separately**;
   if intra-arm sum fails to match seeded count ⇒ scanner did not complete walk, entire round invalidated.
5. **Positive controls (run across every arm)**: Corrupted checksums reject exactly 2; live units Level 2 exactly 37 (for both `hdr47` arms 37 + 3 impostors = 40,
   faithfully reported——inability to distinguish impostors is precisely what is tested); total probe point visits exactly match value given by independent arithmetic,
   mismatch ⇒ code failed to execute, entire round invalidated.

### Failure Clauses (Written Hard Before Run)

- Criterion 1 any cell missed: **faithfully record**, that is a hole in settled form, not a modeling error——never alter fixtures to fit numbers.
- Criterion 5 positive controls missed: **implementation flawed, entire round invalidated.**
- Deterministic model: 5 rounds of byte-identical md5 only proves absence of hidden state, not masquerading as statistical stability
  (`.claude/singlefs-ai-sop/rules/test-discipline.md`).

### Measurement Basis and Known Limitations

- **Worst-case alignment is part of the measurement basis**: In real backup files, nested headers are shifted 91 bytes by container headers,
  ordinarily not landing on probe points——E88 (Discrimination of Impostor Units vs Orphan Units) **deliberately** places them on probe points,
  because scan rebuild is a correctness path whose defenses must not rely on "usually won't align".
- Model constructs no crash points, no encryption, no multi-disk; "Level 1 / Level 2 / clear" are merely counting labels,
  and promotion protocol (D18 (Information Carried in a Block) settled item 5) is outside scope.
- Allocation record predicate modeled per-placement form per D3 (Space Allocation) settled item 1 (probe address not a placement ⇒ predicate false).
- Prerequisites: None. Pure arithmetic.

### Empirical Measurements (2026-09-02)

| Arm | Level 2 | Level 1 | Clear | Impostor Rejected | Nested Admitted | Misclassifications |
|---|---|---|---|---|---|---|
| hdr91_claim (settled form) | **37** | **6** | **9** | **3** | **0** | **0** |
| hdr91_noclaim | 37 | 10 | 9 | 8 | **4** | 4 |
| hdr47_claim | **40** | 15 | **0** | 0 | 0 | **12** |
| hdr47_noclaim | 40 | 24 | 0 | 0 | 9 | 21 |

- **Criterion 1 fully hit.** Settled form matches seeded counts exactly across every cell, with zero misclassifications.
- **Criterion 2 (overturn clause) not triggered.** The 47-byte header misclassifies 12 items even with claim rule enabled.
  First, 3 impostors slip into Level 2——their magic and checksums are entirely genuine, and the placements possess allocation records,
  forging the double proof, which only fsid can expose.
  Second, none of the 9 torn orphans are cleared: without birth generation, no unit can be **positively** judged as garbage,
  leaving them relegated to Level 1 alongside lost-account live units, permanently locking the volume into read-only mode without path to promotion.
  Therefore, **the two riders fsid and birth generation are maintained**.
- **Criterion 3 hit.** Disabling claim rule leaks exactly 4 self-images (identical fsid cannot block self),
  while enabling it yields exactly 0. This demonstrates claim rule is no decoration, but the sole defense for the self-image cell.
  Implementing it in checker already done: C89 (Leftover Recomputation from Closing the Unit Header) item 2, pool-level checker I-7.8 (root record tree ID watermark not lower than the pool-wide maximum tree ID) scanning direction whole-unit claim, pinned by `crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs`.
- **Criteria 4 and 5 both hit.** All four arms conserve exactly 114 probes; corrupted checksums reject exactly 2 on every arm;
  nested headers reach Level 2 on none of the arms, because probe addresses themselves are not placements and allocation record predicate is constantly false.
- Scan results match independent audit cell by cell. The audit employs fixture constants in pure arithmetic, never touching scanner counters.

### Measurement Basis and Replay

- Code `scripts/experiment-apparatus/e88_impostor_orphan.rs`
  (`cargo run --release --bin e88-impostor-orphan`), artifact
  `e88-impostor-orphan-2026-09-02.out` (6 lines, closing `emitted=6`).
  Replay passes E88 (Discrimination of Impostor Units vs Orphan Units) identifier to `scripts/experiment-apparatus/replay.sh`, byte-for-byte comparison.
- **All 10 unit tests green; all 11 mutations caught** (`scripts/mutation/tables/e88_impostor_orphan.tsv`),
  including both "torn boundary > changed to >=" (caught via boundary fixture where birth == root txg) and "audit echo".
- Deterministic model: 5 rounds of byte-identical md5 (proving absence of hidden state, not claiming statistical stability).

### Affected Decisions

| Decision Branch | Relationship | Review |
|---|---|---|
| D18 (Information Carried in a Block) settled item 7 | Supports | 2026-10-17 Unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-19 unaffected: settled form zero misclassifications, 47-byte header misclassifies even with claim rule on——two riders fsid and birth generation stand, self-image cell defended solely by claim rule; 2026-10-11 unaffected: knowledge decay governance only updated outdated status sentences, conclusions and artifacts unchanged |
| D18 (Information Carried in a Block) settled item 13 | Supports | 2026-10-17 Unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-19 unaffected: declared length occurs 0 times in source, whole unit skipped once claimed——declared length entering ciphertext does not alter claim, no rerun needed; 2026-10-11 unaffected: knowledge decay governance only updated outdated status sentences, conclusions and artifacts unchanged |
| D18 (Information Carried in a Block) settled item 20 | Supports | 2026-10-17 Unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-19 unaffected: without fsid and birth generation in header, impostors enter Level 2 and torn orphans cannot be cleared——fsid and birth generation mandatory; 2026-10-11 unaffected: knowledge decay governance only updated outdated status sentences, conclusions and artifacts unchanged |
| D3 (Space Allocation) settled item 1 | Does not affect | 2026-10-17 Unaffected: this round only synchronized wording, artifact pointers, paths, and numbers, conclusions, numbers, and artifacts unchanged; previously 2026-09-19 unaffected: only takes per-placement allocation record predicate form as input; 2026-10-11 unaffected: knowledge decay governance only updated outdated status sentences, conclusions and artifacts unchanged |

## Historical Versions

### 2026-09-02
- Established. Source: C89 (Leftover Recomputation from Closing the Unit Header) item 3,
  original text of falsification clause from backward leg in adversarial round of C68 (Invariant Allowlist Conflicts with Settled Clauses)
  archived in closing section of leg report.
