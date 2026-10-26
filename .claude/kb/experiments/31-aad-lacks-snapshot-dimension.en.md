## E31 AAD Lacks Snapshot Dimension — Run (2026-08-29, Main Agent Rewrote Implementation, N=5 Rounds Byte-identical)

[中文](31-AAD缺快照维.md) · **English** · [日本語](31-AADにスナップショット次元が欠落.ja.md)

**What it asks**: If AAD lacks the "snapshot" dimension, whether snapshot key mismatches pass silently.

**⚠️ Origin of E31 (AAD Lacks Snapshot Dimension) must be clearly stated**: During the adversarial verification round, an attacker leg explicitly instructed "not to modify any files"
**wrote an unauthorized implementation** (`scripts/experiment-apparatus/e31_aad_snapshot.rs`, 7 KB).
The main agent **did not adopt that code**, for two reasons: (1) it had **zero absolute assertions**—
the absolute-assertions check in gate stage `.claude/gate.d/code-source-discipline.sh` failed red on the spot;
(2) it was never reviewed by anyone, nor was it proven to fail red with mutation testing.
⇒ **The code was deleted, the design retained.** Running it required reimplementation following the measurement basis in "What is measured", "Criteria", "Positive control", and "What counts as failure", proving every assertion fails red.

### Which Sentence It Attacks (Verified Line-by-line by Main Agent)

[decisions.md](../decisions.md) D9 (Encryption) Settled Item 6 table "Definitely Must Not Enter AAD" contains this row:

> "**Reader's current snapshot ID** | B1 / COW sharing, **and D6 (Snapshot Implementation Model) unsettled** | A block reachable from N snapshots,
> binding reader-side snapshot ⇒ block verifies in only one snapshot"

**Half the premise for that exclusion disappeared the very next day**:

| Fact | Current Audit Anchor |
|---|---|
| D9 (Encryption) Settled Item 6 settled on **2026-08-28** | D9 (Encryption) section of decisions-history.md (same entry transcribed in D6 (Snapshot Implementation Model) section) 2026-08-29 entry "D9 (Encryption) Settled Item 6 exclusion rationale premise vanished next day": "This unsettled item settled on 2026-08-28" |
| D6 (Snapshot Implementation Model) settled on **2026-08-29**, choosing multi-writable heads | `decisions/06-快照实现模型.md` "Settled (2026-08-29, user settlement): Adopt multi-writable heads" |
| D9 (Encryption) avows AAD's value lies almost entirely in Chain B, where the third category of Chain B threats verbatim is "**snapshot key rewrite**" | Chain A / Chain B table in `decisions/09-加密.md` |

⇒ **That exclusion covers only "reader-side snapshot", failing to cover "different versions of the same logical key under different writable heads"**,
which is precisely the form that came into existence only after D6 (Snapshot Implementation Model) adopted multi-writable heads. **This exclusion was never re-audited.**

⚠️ **Gate stage "whether unsettled items were settled elsewhere" cannot catch it**: that stage only scans **unsettled items**,
whereas D9 (Encryption) Settled Item 6 is already marked "Settled"—**premises serving as decision basis in settled subsections can similarly grow stale without anything monitoring them**.
Placement: [checks-owed.md](../checks-owed.md).

### What Is Measured

Construct a **snapshot key rewrite bug** (attaching the pointer of writable head X's version under writable head Y's key),
asking each of the four arms: **whether reader silently receives data of another writable head.**

| Arm | AAD | nonce |
|---|---|---|
| `current` | D9 (Encryption) settled 5 items (no snapshot dimension) | Stored in pointer (D19 (Block Pointer Structure & Width Budget) settled "nonce belongs to pointer head") |
| `reader_snapshot` | 5 items + **reader's current snapshot** (explicitly excluded by D9 (Encryption)) | Stored in pointer |
| `birth_snapshot` | 5 items + **snapshot carried by key itself (birth snapshot)** | Stored in pointer |
| `derived_nonce` | 5 items (no snapshot dimension) | **Computed on-the-fly from full logical identity** (other half of D9 (Encryption) Settled Item 4) |

⚠️ **`reader_snapshot` is not a strawman**: it was the **sole form considered** when D9 (Encryption) excluded snapshot dimension,
and it inevitably fails on "shared blocks read normally", which is the executable form of that exclusion rationale.

### Criteria (Both Must Be Satisfied Simultaneously; Arms Satisfying Only One Are Exactly What E31 (AAD Lacks Snapshot Dimension) Distinguishes)

1. **Mismatches must be caught**: both mismatches detected, silent reads of erroneous data equals **0**.
2. **Shared blocks must read normally**: when a block is reachable from 3 snapshots, all three reads succeed.

### Positive Control

**Directly flip one byte of ciphertext; all four arms must catch it.**
Without it, "`current` catching 0" cannot distinguish between AAD lacking dimension or **MAC verification not running at all**.

### What Counts as Failure

- If all four arms yield identical results on criterion "all mismatches caught" ⇒ first inspect whether mismatch was genuinely constructed (count if mismatch count equals 2);
  if construction holds and still no difference, concern that "AAD lacks snapshot dimension" is invalid, **record truthfully as valid result**.
- If any arm fails to catch flipped byte in positive control ⇒ **entire round voided**, that arm was not validating MAC at all.

⚠️ **Reimplementation must carry absolute assertions** (blocked by absolute-assertions in `.claude/gate.d/code-source-discipline.sh`):
"2 mismatches" and "3 shared reads" must be directly provided by construction, not read back from tested code;
and a mutation table must be maintained per `scripts/mutation/tables/`.

### Measured Results (2026-08-29, N=5 Rounds Byte-for-byte Identical)

| Arm | Mismatch Caught | Silently Read Others' Data | Shared Read Succeeded | Criterion 1 | Criterion 2 |
|---|---|---|---|---|---|
| `current` (D9 (Encryption) settled 5 items) | **0 / 2** | **2** | 3 / 3 | ✗ | ✓ |
| `reader_snapshot` (Excluded by D9 (Encryption)) | 2 / 2 | 0 | **1 / 3** | ✓ | **✗** |
| `birth_snapshot` | 2 / 2 | 0 | 3 / 3 | ✓ | ✓ |
| `derived_nonce` | 2 / 2 | 0 | 3 / 3 | ✓ | ✓ |

**Positive control passed**: All four arms caught flipped ciphertext byte (2 / 2 each)
⇒ "`current` catching 0" is not because it was not verifying MAC.

**Three Conclusions**:

1. **The settled AAD of D9 (Encryption) is blind to snapshot key mismatches, and silently so**:
   both mismatches read **data belonging to another writable head**, with MAC completely green.
2. **The rationale in D9 (Encryption) for excluding "reader-side snapshot" itself holds**: binding reader-side snapshot indeed slashes shared reads to 1 / 3.
   ⇒ **It is not a strawman; that exclusion is correct for its specific variant.**
3. **There are two solutions, each satisfying both criteria simultaneously**: place **birth snapshot** (the dimension carried by key itself) into AAD,
   or **fold snapshot dimension into on-the-fly computed nonce**.
   ⚠️ **Folding snapshot dimension into on-the-fly computed nonce is the exact same choice as D9 (Encryption) Settled Item 4 (computed vs stored nonce)**—
   and Settled Item 4 is already in the "blocking first line of transaction layer code" table.
   ⇒ **Choosing "computed on-the-fly containing full logical identity" plugs this hole in passing, without adding AAD fields.**

### Measurement Basis and Known Limitations

- Pure logic, no device, no VM. Uses `chacha20poly1305` 0.11.0 `encrypt/decrypt` with `Payload{msg,aad}`.
- **Models only two writable heads and a single mismatch pattern** (attaching X's pointer under Y's key).
  Realistic btree split/merge and write buffer deduplication can produce more mismatch patterns that E31 (AAD Lacks Snapshot Dimension) cannot reach.
- **Does not model nonce reuse**: `derived_nonce` arm folds logical identity into nonce;
  in real implementation that must be verified alongside I-6.1 (nonce not reused), on which E31 (AAD Lacks Snapshot Dimension) says not a word.
- Code `scripts/experiment-apparatus/e31_aad_snapshot.rs` (`cargo run --release --bin e31-aad-snapshot`),
  raw output `e31-aad-snapshot-2026-08-29.out`.
  5 unit tests, **all 4 mutations caught** (table in `scripts/mutation/tables/e31_aad_snapshot.tsv`).

⚠️ **A note regarding the origin of E31 (AAD Lacks Snapshot Dimension)**: The attacker leg that wrote unauthorized code also produced a result,
**cell-by-cell identical across four arms with the version rewritten by main agent**.
Two independent implementations giving identical results served as an unscheduled differential;
**however, what is credited is the rewritten version**—the other version lacked absolute assertions and mutation proofs, failing gates.

**Prerequisites**: None. Pure logic + existing AEAD library, requiring no device, VM, or file system.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D9 (Encryption) Settled Item 6 | Support | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D9 (Encryption) Settled Item 4 | Material | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D6 (Snapshot Implementation Model) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) | No impact | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

Historical entries for E31 (AAD Lacks Snapshot Dimension) are gathered in [experiments-history.md](../experiments-history.md).
