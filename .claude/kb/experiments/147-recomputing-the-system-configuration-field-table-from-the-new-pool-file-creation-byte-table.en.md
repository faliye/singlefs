## E147 Recomputing the System Configuration Field Table from the New-Pool File Creation Byte Table — Run (2026-09-13, counting model, 5 unit tests / all 9 mutations caught)

[中文](147-系统配置字段表按新池新建文件字节表重算.md) · **English** · [日本語](147-システム構成フィールド表を新規プールでのファイル作成のバイト表から再計算.ja.md)

The question concerns the two sentences in blocking clause ③ of D22 (Synthesizing Unit Atomicity) settled item 9 (system configuration field table): “The numbers in the four cells cannot be cited today—first fix C231 (root record fields mixed into system configuration required items), then rerun E115 (Completeness and Byte Budget of System Configuration Field Table) and E124 (System Configuration Field Table Recomputed per D18 Settled Item 14).”
Following this repository's precedent of leaving experiments with fixed criteria and committed artifacts unchanged and rerunning under a new number, this experiment copies the proposed system configuration field table from section 1 of [layout/01-first-txn.md](../layout/01-first-txn.md) line by line into the apparatus. That table has removed the tree table unit pointer rejected by D22 (Synthesizing Unit Atomicity) settled item 12, moved the device table and master-key slot into an indirect directory unit under the combined arm, and set the nonce watermark to 12. We recalculate the 512-byte slot budget and torn states, then calculate one variant for each of the three cells still unsettled in blocking clause ②. We report numbers without choosing a winner.

**Result in one sentence**: The table totals 413, leaving 99 in a 512-byte slot; it spans one sector and has 0 torn states. Inline master-key slot: 493; 4-byte KDF identifier: 415; omit device count: 409; reserve no indirect-directory pointer in version one: 354; all three at their widest: 495—**all six variants fit in one sector**. The four numbers behind the “three cells overflow” result in E115 (Completeness and Byte Budget of System Configuration Field Table) / E124 (System Configuration Field Table Recomputed per D18 Settled Item 14) counted both the 59-byte tree-table pointer and a separate mapping-root pointer in the slot; they are no longer cited.

**Rerun command** (`exact` mode, byte-for-byte comparison with the retained artifact):

```bash
bash scripts/experiment-apparatus/replay.sh E147
# 直接跑装置：
cd scripts/experiment-apparatus && cargo run --release --bin e147-system-configuration-recompute-from-layout
```

Code: `scripts/experiment-apparatus/e147_system_configuration_recompute_from_layout.rs`; raw output: `research/results/e147-system-configuration-recompute-from-layout-2026-09-13.out` (14 lines, last line `emitted=14`);
pre-run registration: `e147-preregistration.md`; mutation table: `scripts/mutation/tables/e147_system_configuration_recompute_from_layout.tsv` (all 9 caught,
`bash scripts/mutation/mutate.sh e147-system-configuration-recompute-from-layout scripts/experiment-apparatus/e147_system_configuration_recompute_from_layout.rs scripts/mutation/tables/e147_system_configuration_recompute_from_layout.tsv`).

### This is a counting model, not an implementation

There is no I/O or random source ⇒ N runs are necessarily byte-identical. The evidence comes from 5 unit tests and all 9 mutations caught: absolute-value assertions pin the five segment totals (166 / 32 / 167 / 36 / 12), total 413, six variants, torn-state formula (2 for n = 2, 6 for n = 3), and widths of the three rows used by the variants. A unit test pins the table at 39 rows (pre-run registration clause b).

### Measurement basis

- The field table matches the “proposed system configuration field table” in [layout/01-first-txn.md](../layout/01-first-txn.md) row by row (39 rows, including the zero-byte journal-reservation row). Segments follow that table: bootstrap header, encryption reserve, geometry, tunables, runtime.
- Slot width 512: the 2026-09-10 user settlement in D22 (Synthesizing Unit Atomicity) was “slot width = detected physical_block_size”; the local machine has 512. Torn states follow E100 (Three-Segment Geometry of System Configuration): spanning n sectors gives 2ⁿ − 2 states.
- Variants: `key_slot_inline` uses 80, the minimum usable assumption for the master-key slot in E115 (Completeness and Byte Budget of System Configuration Field Table); `kdf_identifier_4` raises 2 to 4; `device_count_dropped` saves 4 when code 3 is chosen under C218 (Device table unit code assignment unsettled), because the container carries its own record count; `directory_pointer_dropped` is the reading in which version one reserves no space for it.
- All 9 mutations were caught: copy feature bits as 64; subtract 1 from the torn-state formula; round sector span down; estimate the master-key slot as 60; subtract too little when dropping devs; forget to subtract the original width in the KDF variant; change slot width to 4096; copy nonce watermark as 8; assign a row to the wrong segment (move mac_length to geometry).

### Verdict rows copied verbatim from the artifact

```text
E7RESULT name=config slot_bytes=512 rows=39 key_slot_inline=80 kdf_wide=4
E7RESULT name=segment segment=bootstrap bytes=166
E7RESULT name=segment segment=encryption_reserve bytes=32
E7RESULT name=segment segment=geometry bytes=167
E7RESULT name=segment segment=tunable bytes=36
E7RESULT name=segment segment=runtime bytes=12
E7RESULT name=variant arm=as_layout total_bytes=413 slot_remainder=99 overflow_bytes=0 sectors=1 torn_states=0
E7RESULT name=variant arm=key_slot_inline total_bytes=493 slot_remainder=19 overflow_bytes=0 sectors=1 torn_states=0
E7RESULT name=variant arm=kdf_identifier_4 total_bytes=415 slot_remainder=97 overflow_bytes=0 sectors=1 torn_states=0
E7RESULT name=variant arm=device_count_dropped total_bytes=409 slot_remainder=103 overflow_bytes=0 sectors=1 torn_states=0
E7RESULT name=variant arm=directory_pointer_dropped total_bytes=354 slot_remainder=158 overflow_bytes=0 sectors=1 torn_states=0
E7RESULT name=variant arm=all_three_widest total_bytes=495 slot_remainder=17 overflow_bytes=0 sectors=1 torn_states=0
E7RESULT name=verdict layout_total=413 variants=6 variants_fitting_512=6 e115_e124_cells_superseded=true
```

### What these numbers mean

1. **The “four cells cannot be cited” restriction ends here**: using the table of 2026-09-13, system configuration occupies 413 bytes in one 512-byte sector, leaving 99. All nine items in blocking clause ③ (worst-case journal occupancy and F, per-region root ring device identity 12, `w_max`, `g`, MAC length declaration, KDF identifier, encryption type, master-key slot, devs) appear in the table and have source clauses; the new-pool file creation values are 8 / 4 / 12 / 1 / 1 / 4.
2. **Blocking clause ② reduces to three user choices**: KDF identifier 2 or 4 bytes (difference 2); inline master-key slot (+80, leaving 19) or indirect directory unit (the table's current form, leaving only a 59-byte pointer in the slot); and whether to declare encryption type in system configuration (the table reserves 1 byte). All three at their widest total 495, still within one sector ⇒ these are encoding trade-offs, not budget trade-offs.
3. **Dropping devs (−4) or reserving no indirect-directory pointer in version one (−59) does not change the conclusion**: reservation changes only the remainder (103 / 158), not torn states.
4. The latter half of the loop in blocking clauses ①/④ (pool-wide single value versus per-device, and the incompat-bit way out) is an argument, not part of this calculation (C181 (in-pool heterogeneity criterion missed bitwise XOR stripe axis), C183 (one incompat bit per layout while on-disk bytes show no difference)).

### What it cannot answer

1. Field encodings (byte order, enum values, feature-bit assignments) are not calculated here; they were settled later the same day by D22 (Synthesizing Unit Atomicity) settled items 13 and 17. The manual-copy omission risk of E115 (Completeness and Byte Budget of System Configuration Field Table)'s required-fields table (C178 (system configuration required fields table relies on manual copying)) applies here too: this table was copied from layout/01-first-txn.md, so anything omitted there is omitted here.
2. A 4096-byte slot is not calculated: at the time of the run the system configuration slot width was 512. From 2026-09-14 the width is the format constant 4096 (D22 (Synthesizing Unit Atomicity) settled item 2); the field table totals 481 and still fits in the first 512-byte sector, but the 4096-byte case was not rerun.

### Affected decisions

| Decision item | Relationship | Review |
|---|---|---|
| D22 (Synthesizing Unit Atomicity) settled item 9 | Supports | 2026-10-11 unaffected: wording only changed (time expressions, positional references and self-reference); conclusion, numbers and artifact unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 15 | Supports | 2026-10-11 unaffected: wording only changed (time expressions, positional references and self-reference); conclusion, numbers and artifact unchanged |
| D22 (Synthesizing Unit Atomicity) settled item 12 | No impact | 2026-10-11 unaffected: wording only changed (time expressions, positional references and self-reference); conclusion, numbers and artifact unchanged |

## Historical Versions

### 2026-09-13
- On the day of the run, the user settlement in D22 (Synthesizing Unit Atomicity) settled item 9 took KDF 4, inline master-key slot and an encryption-type declaration—`all_three_widest` at 495 / 17. The byte table was changed to 495 accordingly; `as_layout` at 413 is the table from before the decision.
- Created. The pre-run registration preceded the apparatus; mutation 9 was originally “omit a copied row,” but could not compile (the array length was fixed), so it was invalid and was replaced by “copy a row into the wrong segment.”
