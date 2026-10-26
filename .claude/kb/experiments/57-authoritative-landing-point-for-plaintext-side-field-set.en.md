## E57 Authoritative Placement for Plaintext-Side Field Set — Run (2026-08-31)

[中文](57-明文侧字段集的权威落点.md) · **English** · [日本語](57-平文側フィールドセットの権威着地点.ja.md)

**It answers a "who has final say" question rather than a performance question.**
D9 (Encryption) settled item 9 and D18 (Information Carried in a Block) settled item 7 (settled 2026-09-02, unsettled during run) were
**two halves of the same gap** (minimal field set of the plaintext side); the two cross-referenced each other ⇒ gate stage
"whether an unsettled item has been settled elsewhere" perpetually pointed at whichever was modified later. To converge, one must first settle: **which decision hosts the authoritative record.**

### Criteria Based on Information Flow, Not Taste

> **If field set of A uniquely determines that of B while the reverse does not hold, then A is authoritative and B is a projection.**

This was not invented by E57 (Authoritative Placement for Plaintext-Side Field Set): settlement of D9 (Encryption) settled item 7 had already adopted the same reading
("plaintext mapping layer is derived state; source of reconstruction is unit self-description"), but **that statement had never been measured**.

### What to Measure

Build a model, providing field sets on both sides per settled clauses:

| Side | Fields | Source |
|---|---|---|
| **Block header** (D18 (Information Carried in a Block)) | Unit type tag, tree ID, object ID, object birth generation, anchor offset | D18 (Information Carried in a Block) settled item 3 takes the five AAD fields |
| **Plaintext mapping layer** (D9 (Encryption)) | Logical identity (tree ID, object ID, birth gen, anchor offset) → physical address (dev + offset), ciphertext checksum | D9 (Encryption) settled item 5 + D19 (Block Pointer Structure & Width Budget) settled items 1 / 2 |

Run **reconstruction** once in each direction, counting missing fields cell by cell:

- **Direction A**: Given only block header (plus physical placement and ciphertext naturally known during scan), reconstruct mapping layer entries.
- **Direction B**: Given only mapping layer entries, reconstruct block header fields for each unit.

### Criteria (Hardcoded Before Run)

| # | Criterion |
|---|---|
| 1 | Direction A gap is **0** ⇒ mapping layer reconstructible from block header |
| 2 | Direction B gap is **> 0** ⇒ block header not reconstructible from mapping layer |
| 3 | 1 and 2 hold simultaneously ⇒ **authority lands on D18 (Information Carried in a Block)**, D9 (Encryption) item changes to citation |
| 4 | Both directions are 0 ⇒ **equivalent, authority freely chosen**, faithfully recorded; both > 0 ⇒ **authority split by field**, faithfully recorded |

### What Constitutes Failure (Hardcoded Before Run, Forbidden to Change After Run)

| Clause | Judgment When Triggered |
|---|---|
| **Positive control**: Stuff block header "unit type tag" into mapping layer ⇒ Direction B gap must **zero out** | Does not zero out ⇒ gap not caused by that field, model has other bugs, **entire round aborted** |
| **Negative control**: Strip all block header fields ⇒ Direction A gap must equal **total units** | Not equal ⇒ reconstruction path not reading block header, **entire round aborted** |
| Conclusion points to D9 (Encryption) instead of D18 (Information Carried in a Block) | **Faithfully record**. E57 (Authoritative Placement for Plaintext-Side Field Set) does not presuppose answers—reading adopted in D9 (Encryption) settled item 7 **is precisely what is being tested** |

### Results (2026-08-31, 5 Rounds Byte-Identical): Criterion 4 — Authority Must Be Split by Field

Six corpora across 2 / 32 / 256 / 4000 / 16384 units, six arms:

| Arm | Direction A Deficient Units | Direction B Deficient Units | Which Fields Are Missing |
|---|---|---|---|
| Plan A: Physical placement as free input (**old reading, aborted**) | 0 | N (all) | B lacks unit type tag |
| Plan B: Positive control (mapping layer supplements type tag) | 0 | **0** | —— |
| Plan C: Header only on metadata | Data block count | Metadata count | —— |
| Plan D: Negative control (all block headers stripped) | **N (all)** | 0 (**empty set, not "reconstructible"**) | —— |
| Plan E: Symmetric arm (Direction B also receives free input) | 0 | N | Identical to Plan A |
| **Plan F: Block header forbids physical placement (verdict runs on this)** | **N (all)** | **N (all)** | A lacks Dev + Off, B lacks Kind |

⇒ **Gaps exist in both directions ⇒ Criterion 4: Authority must be split by field.**
Authority for physical address lies in mapping layer; authority for logical identity and unit type lies in block header.

Both controls executed per pre-run hardcoded clauses: positive control zeroed out, negative control equaled total units.
⚠️ **Zero in Direction B of Plan D stems from empty set**—when block headers are all stripped, nothing remains to reconstruct, which does not mean "reconstructible".

### Why Plan A Was Aborted: It Borrowed Authority from the Opposing Side

Plan A gave device number and intra-device offset to Direction A for free, citing "naturally known during scan".
**This step erased a settled clause**:

- `.claude/kb/decisions/09-加密.md` settled item 5 (2026-08-26 user settlement) verbatim:
  "move **authoritative recording point of physical address of a block** out of encryption structures" ⇒ authoritative recording point is in plaintext mapping layer.
- `.claude/kb/decisions/18-块里携带什么信息.md` settled item 3 forbidden list verbatim:
  block header **permanently forbids** physical placement / device ID.

⇒ Placement found via scan is an **observation**, not authoritative record. Plan F fed this clause into model; Direction A collapsed immediately.
⚠️ This was precisely the **falsification observation written by forward leg itself** ("forbidden to rerun Direction A feeding scan placement")—it fired.

### Three-Way Argumentation: Two Legs Struck True, Criterion Itself Struck

| Leg | Stance | Result |
|---|---|---|
| Local (Qwen3-Next-80B) | Counterexamples | Struck form: A gets free input while B does not. **Attack invalid**—Dev/Off/Crc were already within mapping layer's seven fields, held by Direction B from start (Plan E strictly equals Plan A, asserted)|
| Cloud Sonnet | Forward | Judged "consistent", providing falsification observation "rerun Direction A without feeding scan placement". **That observation fired** |
| Cloud Opus | Reverse | **Struck core**: criterion of E57 (Authoritative Placement for Plaintext-Side Field Set) was **self-invented**, whereas boundary criterion in `.claude/kb/decisions/21-权威态与派生态的分界.md` was settled as "loss loses data vs loss only slows down" + format freeze participation, not direction of derivability |

**Verdict rendered by main agent, not voting** (`.claude/rules/three-way-inference.md`):
Three source passages verified verbatim (`decisions/09-加密.md:173`, `decisions/18-块里携带什么信息.md:205` and `:218`,
`decisions/21-权威态与派生态的分界.md:6-8`); backward leg held, forward leg misread forbidden list.

⚠️ **Another claim E57 (Authoritative Placement for Plaintext-Side Field Set) is forbidden to make**: `decisions/18-块里携带什么信息.md` verbatim at the time
"prior to D18 (Information Carried in a Block) settled item 5 being answered, **forbidden** to read 'data block has self-describing header' as
'reconstructed mapping is trustworthy'"—Plan A's verdict line stated exactly that, **the second reason for its invalidation**.

### Mutation Testing: 14 Injections, 14 Caught, Zero Blind Spots

Including "reversing two directions", "difference set calculated as intersection", "Plan F secretly reclaiming physical placement", "swapping cell 1 and cell 2 in verdict table".

⚠️ **First version of truth table in verdict line was wrong**: in second cell of `(a.units==0, b.units>0)`,
`b.units>0` meant "B **failed**", while label read "B succeeded"—Plan F reported inverted verdict upon initial run.
Split into independent `verdict_of`, asserting across four cells individually.

⚠️ **Running E57 (Authoritative Placement for Plaintext-Side Field Set) on 2026-08-31 first caught a bug in tool itself**: `scripts/mutation/mutate.sh` identified "which test turned red"
using regex `[A-Za-z0-9_]*`, **failing to match Chinese test names**, thus reporting 9 fully red mutations as
"not a single test turned red". This was a **false blind spot**, misleading humans to write existing checks.
Blast radius audited to 1 file (only E57 (Authoritative Placement for Plaintext-Side Field Set) used Chinese test names); 28 mutation proofs prior to 2026-08-31 unaffected.
Changed to "everything before `...` counts as name", recording "forbidden to enumerate character classes" into script—**same mistake committed twice**
(2026-08-29 was `[a-z_]*` omitting names with numbers).

### The Half It Cannot Answer

1. **This is field-level derivability, not "whether this field should exist"**: field sets on both sides take settled clauses;
   E57 (Authoritative Placement for Plaintext-Side Field Set) does not evaluate whether fields were chosen correctly.
2. **Counting model, not implementation** (0 file operations), same limitation as E54 (How Many Generations to Retain in Accounting).
3. **Does not touch encryption**: ciphertext checksum modeled as "recomputable during scan"; whether to actually recompute is implementation choice.
4. **Does not answer "whether mapping layer needs other fields"**—that is the gap itself; E57 (Authoritative Placement for Plaintext-Side Field Set) only settles **who records it**.
5. **Does not touch second half of D9 (Encryption) settled item 9**: "how mapping layer integrity itself is preserved (not in MAC)".
   That half is independent of derivability; E57 (Authoritative Placement for Plaintext-Side Field Set) gives zero numbers for it.
6. **Criterion is self-invented**: repo boundary criterion is D21 (Boundary Between Authoritative and Derived States) "loss loses data vs loss only slows down".
   Derivability criterion in E57 (Authoritative Placement for Plaintext-Side Field Set) only serves to **partition the gap**, forbidden to override D21 (Boundary Between Authoritative and Derived States).
7. **C34 (three-way clash on plain text mapping layer states) remained owed when running this page, obsolete without object on 2026-09-07 as D9 (Encryption) settled item 5 shifted to "scrub only" (no plaintext mapping layer)**:
   mapping layer state had three conflicting statements across repo; E57 (Authoritative Placement for Plaintext-Side Field Set) did not converge it, only providing reading "split by field" non-conflicting with the three.

### Prerequisites

None. Pure counting model.

### Measurement Basis and Reproduction

- Code `scripts/experiment-apparatus/e57_field_authority.rs` (5 arms)
  (`cargo run --release --bin e57_field_authority`),
  raw output `e57-field-authority-2026-08-31.out`.
- Mutation table `scripts/mutation/tables/e57_field_authority.tsv`.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D9 (Encryption) settled item 9 | Prepares | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D18 (Information Carried in a Block) settled item 7 | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D19 (Block Pointer Structure & Width Budget) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |
| D21 (Boundary Between Authoritative and Derived States) | Does not affect | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-11 unaffected: only wording was changed (temporal terms, location references, and self-references); the conclusions, numbers, and artifacts remain unchanged |

## Historical Versions

### 2026-08-31

- Run completed, concluding on **Criterion 4: Authority must be split by field**. 5 rounds byte-identical, 14 mutations all caught.
- **Plan A (Criterion 3 version) aborted**. Past: Criterion 3 held, authority landing on D18 (Information Carried in a Block) overall.
  Now: Plan A allowed block header to borrow physical placement, whereas authoritative recording point of that item was settled in plaintext mapping layer by D9 (Encryption) settled item 5
  (2026-08-26 user settlement), and D18 (Information Carried in a Block) settled item 3 forbidden list verbatim prohibited block header from holding it. Basis for invalidation: backward leg of three-way argument pointed out criterion was self-invented; main agent confirmed via verbatim check across three original passages.
- Added sixth arm "Plan F: Block header forbids physical placement". Basis: falsification observation written by forward leg fired upon execution.
- Added fifth arm "Plan E: Symmetric arm". Basis: local leg claimed Direction A received free input while B did not; measurements confirmed those three items
  already existed within mapping layer entries; **attack invalid**, symmetric arm strictly identical to Plan A, asserted and archived.
- Initiated. Basis: D9 (Encryption) settled item 9 and D18 (Information Carried in a Block) item 7 (unsettled at the time) cross-referenced each other,
  gate "whether unsettled items settled elsewhere" perpetually pointing to whichever was modified later; user demanded **experiments to settle authority**, not intuition.
