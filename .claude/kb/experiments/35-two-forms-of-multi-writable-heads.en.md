## E35 Two Forms of Multi-Writable-Heads — Run (2026-08-29, N=5 Rounds Byte-identical)

[中文](35-多可写头两种形态.md) · **English** · [日本語](35-複数書き込み可能ヘッドの2つの形態.ja.md)

**What it asks**: After D6 (Snapshot Implementation Model) settled on adopting multi-writable heads, which of the two implementation forms carries pitfalls before writing the first line of code.

### What It Attacks: Two Settled Items Brought Together

| Source | Verbatim |
|---|---|
| D8 (Core Index Structure) Settled Item 3 | `key = (locality_id, inode, offset)` —— **Three segments, no snapshot dimension** |
| D9 (Encryption) Settled Item 4 | Whether nonce is **computed on-the-fly** or stored per extent. Under "computed on-the-fly", input is logical identity, which is the key |

Two writable heads each write once to the same `(locality, inode, offset)`: under three-segment keys their logical identities
are **byte-for-byte identical** ⇒ computing on-the-fly yields **the exact same nonce** ⇒ encrypting two different plaintexts with same key and same nonce
⇒ ChaCha20 keystream reuse.

⚠️ **This is not a detectable error like "MAC will mismatch"; confidentiality is simply destroyed**,
and I-6.1 (nonce not reused) admits that such incidents "will not manifest as any read or write errors".

### Three Arms

| Arm | key | One Tree Per Head | nonce Source |
|---|---|---|---|
| `key3_derived` | Three segments (D8 (Core Index Structure) settled) | No | Computed on-the-fly |
| `key4_derived` | Four segments (adds snapshot dimension) | No | Computed on-the-fly |
| `clone_per_head` | Three segments | **Yes** | Computed on-the-fly |

⚠️ **`clone_per_head` was not invented by E35 (Two Forms of Multi-Writable-Heads)**: D5 (Snapshot / Space Accounting Mechanism)
section "Clones / Writable Snapshots Require Three Additions" already describes it—each writable head holds a copy of `previous_snapshot_txg`
and a copy of deadlist. **It is an existing path lying ready in the repository, unnamed in the D6 (Snapshot Implementation Model) ruling.**

### Criteria (Two, Must Be Satisfied Simultaneously)

1. **Keystream reuse must not occur**: `keystream_reuse_pairs == 0`.
2. **Each head reads back its own writes**: `own_reads_ok == 2`.

### Positive Control

Verify identity "XOR of two ciphertexts with same key and same nonce == XOR of two plaintexts":
on the failing arm it **must hold** (proving reuse is genuine); on the other two arms it **must not hold** (proving metric can discriminate).
⚠️ This tally **unconditionally judges across all pairs**, not wrapped inside a "already judged reuse" branch—
wrapping it would force it to equal reuse count, losing its status as independent corroboration (mutation M5 verified: wrapping it to constant true left zero tests red).

### What Counts as Failure

- If `keystream_reuse_pairs` across all three arms is 0 ⇒ first verify whether logical identities of the two heads were genuinely constructed to collide
  (unit test `under_the_three_segment_key_the_two_heads_collide_on_one_slot` judges exactly this);
  if construction holds and still no difference, concern is invalid, **record truthfully**.
- If XOR identity fails to hold on failing arm ⇒ **entire round voided**, metric was not observing keystream.

### Measured Results (2026-08-29, N=5 Rounds Byte-identical; 2 Writable Heads, 64-Byte Plaintext, ChaCha20-Poly1305)

| Arm | Keystream Reuse Pairs | XOR Identity Holds | Each Head Reads Own Writes | Judgment |
|---|---|---|---|---|
| `key3_derived` (D8 (Core Index Structure) settled + computed on-the-fly) | **1** | **1** | **1 / 2** | ✗ |
| `key4_derived` | 0 | 0 | 2 / 2 | ✓ |
| `clone_per_head` | 0 | 0 | 2 / 2 | ✓ |

**Three Conclusions**:

1. **On the failing arm, encryption is merely the second-layer injury**. Its `own_reads_ok = 1`—
   under three-segment keys, two heads **share the same key slot**, later writes overwriting earlier ones.
   ⚠️ **This is independent of encryption: even without encrypting a single byte, this arm is already broken.**
   Construction self-proof: logical identities computed by both heads match field-by-field (unit test directly compares `Ident`).
2. **When encryption is enabled and D9 (Encryption) Settled Item 4 chooses "computed on-the-fly", a layer of keystream reuse is stacked on top**:
   XOR identity empirically holds ⇒ XOR of two ciphertexts **is** XOR of two plaintexts.
   This is not inference; it ran on real AEAD.
3. **Both exits satisfy both criteria simultaneously**: add snapshot dimension to key, or give each head its own tree (the D5 (Snapshot / Space Accounting Mechanism) path).
   Difference between the two lies not in correctness, but in operational overhead—E35 (Two Forms of Multi-Writable-Heads) measured only two coarse metrics
   (how many intervals touched to delete a head, how many key components compared in point lookup), **insufficient to choose between them**.

**One point blocking natural defense**: "Just add a version number" does not work—
both heads write to that location for the **first time**, so their version numbers are both 1 (pinned by unit test).

### Measurement Basis and Known Limitations

- **Models only 2 writable heads, 1 position, 1 write**. More reuse pairs would emerge under realistic workloads; E35 (Two Forms of Multi-Writable-Heads) does not extrapolate.
- **Measures only two operational metrics, both constants given by construction**, not empirically derived:
  "how many intervals touched to delete a head" and "how many key components compared in point lookup".
  ⇒ **E35 (Two Forms of Multi-Writable-Heads) can judge "which arm is wrong", but cannot judge "which of the two correct arms to choose".**
- **Does not model snapshot ancestor filtering**: `key4_derived` arm placed only two parallel heads,
  without shared blocks or ancestor chains. Shared block cell covered by E31 (AAD Lacks Snapshot Dimension).
- Code `scripts/experiment-apparatus/e35_head_forms.rs`
  (`cargo run --release --bin e35-head-forms`),
  raw output `e35-head-forms-2026-08-29.out`.
  7 unit tests, **all 6 mutations caught** (table in `scripts/mutation/tables/e35_head_forms.tsv`).

**Prerequisites**: None. Pure logic + existing AEAD library.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D6 (Snapshot Implementation Model) Settled Item 1 | Support | 2026-09-20 unaffected: falsified "three-segment key + single tree + multi-heads"; under Form 2, one tree per head makes three-segment keys unique in tree, eliminating conflict; admitted measuring only two coarse metrics, detailed breakdown taken over by E52 (Operational Costs of Two Multi-Writable-Head Mechanisms) |
| D8 (Core Index Structure) Settled Item 3 | Material | 2026-09-20 unaffected: three-segment key collision caused by its key layout; resolution recorded in linked item 3 of D6 (Snapshot Implementation Model) Settled Item 1 |
| D9 (Encryption) Settled Item 4 | No impact | 2026-09-20 unaffected: merely cites storing nonce per extent to note obligation of cross-head non-collision remains, judging nothing |
| D5 (Snapshot / Space Accounting Mechanism) | No impact | 2026-09-20 unaffected: merely serves as source of premise for multi-writable heads; E35 (Two Forms of Multi-Writable-Heads) does not measure its accounting |

## Historical Versions

Historical entries for E35 (Two Forms of Multi-Writable-Heads) are gathered in [experiments-history.md](../experiments-history.md).
