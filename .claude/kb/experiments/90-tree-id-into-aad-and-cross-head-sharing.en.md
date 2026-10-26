## E90 Tree ID into AAD and Cross-Head Sharing — Run (2026-09-03, Real AEAD Pure Logic, 9 Unit Tests / All 9 Mutations Caught): **Chain 4 Deduction Only Half True — Reader Tree Dimension Catches All Swaps but Rejects 4 of 6 Legitimate Shared Reads; Feasible Form Is Birth (Tree, txg) into AAD + Ancestry Table Acceptance, Passing Both Criteria**

[中文](90-树ID进AAD与跨头共享.md) · **English** · [日本語](90-ツリーIDのAAD格納とヘッド間共有.ja.md)

**It verifies the hanging deduction at chain link 4 of D6 (Snapshot Implementation Model) settlement**:
"AAD includes tree ID (settled in D9 (Encryption) settled item 6), and under one tree per head the AAD of two heads is naturally distinct
⇒ the two silent mismatches in E31 (AAD Lacks Snapshot Dimension) should be caught by MAC. This is a deduction without a re-run — to treat it as settled, supplement the run first."

**The deduction has an unasked flip side**: D9 (Encryption) settled item 6 settled that expected value of tree ID originates from **lookup path**,
while D6 (Snapshot Implementation Model) Criterion 4 settled that cloning **does not touch any bytes of already-published blocks** —
⇒ for blocks born prior to clone point and legitimately shared into a new head, the AAD tree in ciphertext remains the birth tree.
When a reader takes expected value per its own tree, **will legitimate shared reads be rejected alongside mismatches** (
the exact failure form of 1/3 shared reads in reader_snapshot arm of E31 (AAD Lacks Snapshot Dimension))?
Chain 4 only examined the mismatch half.

### The World (Hardcoded Before Run, Figures Derived Directly from Construction)

Cloning lineage: Head 1 as root; Head 2 cloned from Head 1 at txg 10; Head 3 cloned from Head 2 at txg 20;
Head 4 cloned from Head 1 at txg 10 (collateral branch). Each extent carries (birth_tree, birth_txg):

| extent | Birth | Legitimately Reachable Heads |
|---|---|---|
| S | (1, 5) | 1, 2, 3, 4 (prior to clone points) |
| S2 | (2, 15) | 2, 3 |
| A | (2, 25) | Only 2 (after Head 3 fork) |
| B | (3, 25) | Only 3 |
| C | (4, 30) | Only 4 |

- **Exactly 3 mismatch injections**: A grafted under Head 3's key, B grafted under Head 2's key, C grafted under Head 3's key
  (third injection covers "collateral head not on reader's ancestral chain").
- **Exactly 1 forged declaration injection**: A grafted under Head 3's key with pointer birth declaration altered to
  (2, 15) which passes ancestry acceptance — acceptance cannot stop it, requiring the "declaration entered AAD" layer to catch it.
  This cell is the actionable reason why "birth (tree, txg) must enter AAD and acceptance cannot inspect pointers alone".
- **Exactly 6 legitimate shared reads**: S read once each from 1, 2, 3, 4 (4 reads), S2 read once each from 2, 3 (2 reads).
- Positive control: Flipping one ciphertext byte per sealed unit, every arm must catch all.

### Three Arms (AAD Five Items per D9 (Encryption) Settled Item 6, Tree Dimension Taken per Arm; Nonce Invariably Stored in Pointer, D9 (Encryption) Settled Item 4)

| Arm | How Tree Enters AAD | Expected Value Source |
|---|---|---|
| `reader_tree` | Reader's current tree | Literal reading of lookup path — form of Chain 4 deduction |
| `pointer_tree` | Birth tree stored in pointer | **Read back from pointer** (degraded form strictly prohibited by D9 (Encryption) expected value source criterion, serving as negative control) |
| `ancestry_birth` | (birth_tree, birth_txg) into AAD; pointer carries both as declaration | Reader validates declaration against **clone ancestry table** first (declared tree on ancestral chain and birth txg ≤ clone point, or matches current tree); only on passing does reader construct AAD into MAC — mapping of E31 (AAD Lacks Snapshot Dimension) Outlet 1 (birth snapshot into AAD) to one tree per head |

### Criteria (Hardcoded Before Run, Same Pair as E31 (AAD Lacks Snapshot Dimension))

1. **Catch all mismatches**: swap_detected == 3 and swap_silent_wrong == 0.
2. **Pass all legitimate shared reads**: shared_ok == 6.
3. **Positive control** (every arm, every unit): Flipping ciphertext bytes must all be caught; bug-free readback of own data must all pass.
   Failing ⇒ arm not verifying MAC, **entire round aborted**.

### Verdict Clauses (Hardcoded Before Run)

- `reader_tree` **passing both criteria** ⇒ Chain 4 deduction holds completely, sharing concerns unfounded, faithfully recorded.
- `reader_tree` failing Criterion 2 ⇒ Chain 4 phrase "incidentally remedied" must be rewritten in body of D6 (Snapshot Implementation Model)
  (cost of tree dimension catching swaps is rejecting all cross-head inherited reads), recorded in decisions-history.
- `pointer_tree` surprisingly passing Criterion 1 ⇒ mechanism rationale in D9 (Encryption) of "forbidden to read back from pointer" is flawed, faithfully recorded.
- `ancestry_birth` passing both ⇒ a viable form exists for Outlet 1 of E31 (AAD Lacks Snapshot Dimension) under one tree per head;
  its format implications (pointer carries birth (tree, txg), reader maintains clone ancestry table) recorded in body, **without deciding on the spot**.
- Any arm differing from expectation is **faithfully juxtaposed**, forbidden to modify criteria retroactively (modification constitutes a new experiment).

### Measurement Basis and Known Limitations

- Pure logic + `chacha20poly1305` real AEAD, sharing skeleton with E31 (AAD Lacks Snapshot Dimension); no devices, no VMs.
- Models extent-level sharing and one mismatch form only (whole pointer re-grafting); btree split/merge and write buffer
  deduplication can synthesize further mismatch forms beyond current reach.
- Residual blind spot of `ancestry_birth` **explicitly unmeasured**: version substitution **within** ancestry acceptance interval
  (swapping two pre-clone versions of same (object, offset)) — requires retaining old versions across snapshots, outside current scope.
- Clone ancestry table modeled as trusted metadata (residing in authenticated structure with small footprint in real implementation);
  tampering of table itself is outside current scope.
- Prerequisites: None.

### Measurement (2026-09-03, Byte-for-Byte Identical Across 5 Runs of md5)

| Arm | Swaps Rejected | Silently Read Foreign Data | Forged Declarations Caught | Legitimate Shared Reads | Criterion 1 | Criterion 2 |
|---|---|---|---|---|---|---|
| `reader_tree` (form of Chain 4 deduction) | **3 / 3** | 0 | 1 / 1 | **2 / 6** | ✓ | **✗** |
| `pointer_tree` (read back from pointer) | **0 / 3** | **3** | **0 / 1** | 6 / 6 | ✗ | ✓ |
| `ancestry_birth` (birth (tree,txg) + ancestry acceptance) | **3 / 3** | 0 | 1 / 1 | **6 / 6** | ✓ | ✓ |

Positive controls all passed: all three arms caught 5 ciphertext byte flips, bug-free readbacks 5/5.

**Four Conclusions**:

1. **Verdict clause cell 2 triggered**: `reader_tree` caught all swaps but rejected 4 of 6 legitimate inherited reads —
   "AAD tree ≠ reader tree" **looks identical** under legitimate inheritance and illegal swaps; tree dimension cannot distinguish them.
   Chain link 4 of D6 (Snapshot Implementation Model) stating "incidentally remedied" rewritten per verdict clauses (see decisions-history).
2. `pointer_tree` let all three swaps pass silently, **failing even to detect forged txg in declaration** (its AAD binds tree field only) —
   blinder than pre-run manual prediction (which predicted catching forgery, empirical 0 / 1, mechanism: forgery did not touch tree field).
   This is actionable proof for D9 (Encryption) expected value source criterion "forbidden to read back from pointer".
3. **`ancestry_birth` passed both criteria**: all 3 swaps rejected by ancestry acceptance (declared birth txg
   exceeds clone point, or declared tree not on ancestral chain), forged declarations caught by MAC (birth (tree,txg) bound into AAD),
   all 6 legitimate shared reads passed. It materializes Outlet 1 of E31 (AAD Lacks Snapshot Dimension) "binding birth snapshot" under one tree per head.
4. Two format implications (recorded, not decided on the spot): pointer must accommodate two fields for birth (tree, txg);
   clone ancestry table must reside in authenticated structure accessible to readers.

### Measurement Basis and Re-run

- Code `scripts/experiment-apparatus/e90_tree_aad.rs`
  (`cargo run --release --bin e90-tree-aad`), artifact
  `e90-tree-aad-2026-09-03.out` (5 lines, ending with `emitted=5`).
  Re-run by passing E90 (Tree ID into AAD and Cross-Head Sharing) ID to `scripts/experiment-apparatus/replay.sh`, byte-for-byte comparison.
- **9 unit tests all green; all 9 mutations caught** (`scripts/mutation/tables/e90_tree_aad.tsv`),
  including opening acceptance boundary, skipping acceptance, ancestral chain omitting min, ancestry arm not binding txg, reader tree arm peeking at pointer,
  moving shared fixture post-clone point, and control flipping without effect.
- Deterministic model: 5 runs produce byte-for-byte identical md5 (demonstrating absence of hidden state, not claiming statistical stability).

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D9 (Encryption) settled item 6 | Support | 2026-09-19 unaffected: reader taking tree dimension per lookup path rejects legitimate inherited reads; birth (tree, txg) into AAD plus clone ancestry acceptance passes both criteria — viable form of binding birth snapshot under one tree per head in settled item 6 scope |
| D9 (Encryption) settled item 16 | Support | 2026-09-19 unaffected: `pointer_tree` arm lets all three swaps pass silently without detecting forged declarations — actionable proof of expected value source criterion "forbidden to read back from pointer" |
| D6 (Snapshot Implementation Model) settled item 1 | Material prep | 2026-09-19 unaffected: verdict clause cell 2 triggered; phrase "incidentally remedied" at chain link 4 of settled item 1 rewritten per verdict clauses; E90 (Tree ID into AAD and Cross-Head Sharing) does not alter "one tree per head" itself |

## Historical Versions

### 2026-09-03
- Established and completed on same day. Source: Chain link 4 of D6 (Snapshot Implementation Model) settlement hung verbatim "to treat it as settled, supplement run first";
  flip-side question (whether tree dimension breaks cross-head shared reads) derived verbatim from expected value source of D9 (Encryption) settled item 6
  and D6 (Snapshot Implementation Model) Criterion 4; criteria and world hardcoded before run.
  Pre-run manual prediction anticipated `pointer_tree` catching forged declaration (1/1), empirical 0/1 —
  forgery touched only txg while arm's AAD bound only tree field; pinned values updated per empirical facts, reflecting manual prediction error with mechanisms faithfully recorded.
