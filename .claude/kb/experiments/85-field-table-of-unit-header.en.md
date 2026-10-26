## E85 Field Table of Unit Headers — Run (2026-09-02, Requirements Matrix, 9 Unit Tests / 5 Mutations All Caught): Settled Field Set Cannot Judge I-1.2 (block header write order published) / I-1.4 (block header fsid consistent) / Header Self-Proof; Whitelist Literal Set Cannot Even Judge Scan Rebuild; Patch Set 71 Bytes Fully Sufficient (0.43% / 0.22%), Clashing with Whitelist on Exactly Eight Items

[中文](85-单元头的字段表.md) · **English** · [日本語](85-ユニットヘッダのフィールド表.ja.md)

Machine-checked half of C68 (Invariant Allowlist Conflicts with Settled Clauses) / C69 (Settled Invariants Lack Fields to Judge):
spreading "who requires which header field" across a machine-checkable matrix, detailing which judgments each of the three candidate field sets reaches and how many bytes each consumes.
Closeout is a decision; E85 (Field Table of Unit Headers) first prepares the material.

### Criteria (Written Hard Before Run, Unchanged After Run)

1. Matrix pinned down cell by cell; 2. Byte accounting in closed form (widths are marked assumptions; changing widths updates figures, leaving matrix unchanged);
3. Conflict list with I-6.2 (plaintext header field allowlist) explicitly named; 4. Does not judge which set to select.

### Empirical Measurements

| Candidate Set | Bytes | Share of 16 KiB / 32 KiB | Unreachable Judgments |
|---|---|---|---|
| set_a (Settled minimal set: 5-tuple + declared length) | 35 | 0.21% / 0.11% | **I-1.2 (block header write order published), I-1.4 (block header fsid consistent), scrub watermark, header self-proof, scan magic probe**——machine-checked form of C69 (Settled Invariants Lack Fields to Judge) |
| set_b (Patch set: set_a plus magic, version, flags, fsid, block birth, header checksum) | **71** | **0.43% / 0.22%** | Only "self logical address" row remains (what it points to is unclosed; empirical section lists rationale why all three sets lack it) |
| set_c (I-6.2 (plaintext header field allowlist) literal six items) | 30 | 0.18% / 0.09% | **Entire scan rebuild row** (none of the 5-tuple present), tree ID, fsid, block birth... |

- **Conflict list with whitelist contains exactly eight items** (disposed item-by-item during C68 (Invariant Allowlist Conflicts with Settled Clauses) closeout on 2026-09-02):
  unit_type / tree_id / obj_id / obj_birth / anchor_off / fsid / block_birth / hdr_csum.
- **The "self logical address" row is omitted across all three sets intentionally**: I-1.1 (block header states its logical address) requires it,
  yet the 5-tuple uses "anchor offset + object identity" to express location, and the forbidden list prohibits physical placement——
  what it indicates in the data unit header is itself a question that closeout must answer.
- Block birth and object birth are two distinct fields (verbatim from C69 (Settled Invariants Lack Fields to Judge)), pinned by unit tests forbidding confusion.

### Failure Clauses (Written Hard Before Run)

Matrix and byte accounting completely matched. No invalidation triggered.

### What It Cannot Answer

All widths are assumptions (identity fields taken as 8 bytes; header checksum taken as 4 bytes——E23 (Journal Geometry) record header checksum
is 32 bytes, and whether unit headers require the same width is a closeout question); does not answer "what self logical address indicates";
does not answer which fields move to ciphertext side once encryption is enabled.

### Measurement Basis and Replay

- Code `scripts/experiment-apparatus/e85_unit_header.rs`
  (`cargo run --release --bin e85-unit-header`),
  raw output `e85-unit-header-2026-09-02.out` (6 lines, closing line `emitted=6`).
- Pure matrix, identical md5 across 5 rounds. **9 unit tests, all 5 mutations caught one by one, 0 blind spots**,
  mutation table `scripts/mutation/tables/e85_unit_header.tsv`
  (`bash scripts/mutation/mutate.sh e85-unit-header scripts/experiment-apparatus/e85_unit_header.rs scripts/mutation/tables/e85_unit_header.tsv`).

### Affected Decisions

| Decision Branch | Relationship | Review |
|---|---|---|
| D18 (Information Carried in a Block) settled item 7 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-19 Unaffected: requirements matrix and byte accounting: settled minimal set cannot judge I-1.2 (block header write order published), I-1.4 (block header fsid consistent), and header self-proof, patch set fully sufficient, conflicts with whitelist named item-by-item——field table partitioned by unit class, fsid and birth generation enter header |
| D18 (Information Carried in a Block) settled item 11 | Supports | 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-09-19 Unaffected: 5-tuple total of 33 comes from its "identity fields take 8 bytes" width assumption, partition directly settled based on this |

## Historical Versions

Historical entries for E85 (Field Table of Unit Headers) are centralized in [experiments-history.md](../experiments-history.md).
