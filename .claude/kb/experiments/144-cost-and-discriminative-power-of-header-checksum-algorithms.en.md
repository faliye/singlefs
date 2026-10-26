## E144 Cost and Discriminative Power of Header Checksum Algorithms — Run (2026-09-13, Timing + Deterministic Discriminative Power, 8 Unit Tests / 10 Mutations All Caught)

[中文](144-头校验和算法的代价与判别力.md) · **English** · [日本語](144-ヘッダチェックサムアルゴリズムのコストと判別力.ja.md)

**It asks about C309 (Algorithm of the 32-Byte Checksum Not Settled)**: the unit header checksum, the root record self-certifying checksum, the journal record header checksum, and the whole-slot checksum of the system configuration have only had their 32-byte width and coverage settled; the digest function is unsettled.
The user directed: "if it has nothing to do with encryption, it can be cheaper" — these places guard against corruption, not attack, so the candidates need not be cryptographic digests.
**All seven criteria pass**: the six formal arms (SHA-256, BLAKE2s, xxHash64 × 4 seeds, xxHash64, CRC-64/XZ, CRC32C) miss nothing on single-bit flips, exhaustive double-bit flips, and random byte changes across five real header widths;
the positive control (8-bit additive sum) misses 11 892 in the exhaustive double-bit run, so the discriminative-power test can tell the difference; on cost, CRC32C takes 26 ns for a 105-byte header and SHA-256 takes 676 ns, a **26×** gap;
14 headers folded into one 359 µs fsync make SHA-256 2.6% and CRC32C 0.1%.

**Replay command** (`timing` mode: the discriminative-power half is compared byte by byte; the nanosecond figures are pinned by the interval assertions of `replay.sh` — crc32c 19 to 37, sha256 473 to 879):

```bash
bash scripts/experiment-apparatus/replay.sh E144
# Run the rig directly:
cd scripts/experiment-apparatus && cargo run --release --bin e144-header-checksum-cost
```

Code `scripts/experiment-apparatus/e144_header_checksum_cost.rs` (the six functions are each pinned by public test vectors: `abc` for SHA-256 and BLAKE2s, the empty string and `abc` for xxHash64, `123456789` for the two CRCs),
raw output `e144-header-checksum-cost-2026-09-13.out` (129 lines, last line `emitted=129`), pre-run registration `e144-preregistration.md`, mutation table `scripts/mutation/tables/e144_header_checksum_cost.tsv`.

### Measurement Basis

- The discriminative-power half is deterministic (fixed seeds): single-bit flips go through every bit (including the bits of the checksum field itself); double-bit flips exhaust all 170 236 pairwise combinations of the 584 bits outside the field on 105 bytes; random changes alter 1 to 8 bytes outside the field 200 000 times each (redrawn if the result equals the original);
  the probe is 1 000 000 random 42-byte prefixes, reporting three numbers: accepted by magic + version + the whole 32-byte field, accepted by the whole field only, accepted by the used bytes only.
- Timing is a varying observation: 7 rounds per arm per width, 200 000 iterations per round, the minimum taken, with the per-round spread (in basis points) reported separately; this machine's software implementation, with neither SHA-256 nor CRC32C using hardware instructions.
- All 10 mutations caught: one constant changed in each of five functions (vector tests red), the control swapped for a real hash (control test red), unused bytes not zero-padded, the number of headers per fsync changed to 12, the probe that compares only used bytes changed to compare the whole field, single-bit flips skipping the bits of the checksum field.

### Verdict (Judged by the Criteria Fixed Before the Run, Lines Copied Verbatim from the Artifact)

```text
E7RESULT name=single_bit arm=crc32c width=105 positions=840 missed=0
E7RESULT name=double_bit arm=crc32c width=105 pairs=170236 missed=0
E7RESULT name=double_bit arm=sha256 width=105 pairs=170236 missed=0
E7RESULT name=double_bit arm=sum8 width=105 pairs=170236 missed=11892
E7RESULT name=probe arm=crc32c trials=1000000 bytes_used=4 accepted_with_magic=0 accepted_full_field=0 accepted_used_bytes_only=0
E7RESULT name=probe arm=sum8 trials=1000000 bytes_used=1 accepted_with_magic=0 accepted_full_field=0 accepted_used_bytes_only=3919
E7RESULT name=cost arm=sha256 width=105 ns_per_op=676 spread_bp=0
E7RESULT name=cost arm=blake2s width=105 ns_per_op=167 spread_bp=59
E7RESULT name=cost arm=xxh64x4 width=105 ns_per_op=34 spread_bp=294
E7RESULT name=cost arm=xxh64 width=105 ns_per_op=10 spread_bp=0
E7RESULT name=cost arm=crc64 width=105 ns_per_op=131 spread_bp=0
E7RESULT name=cost arm=crc32c width=105 ns_per_op=26 spread_bp=0
E7RESULT name=fsync_share arm=sha256 headers_per_fsync=14 ns_per_op=676 fsync_ns=359000 fsync_ratio_bp=263
E7RESULT name=fsync_share arm=crc32c headers_per_fsync=14 ns_per_op=26 fsync_ns=359000 fsync_ratio_bp=10
E7RESULT name=verdict formal_single_bit_missed=0 formal_double_bit_missed=0 formal_random_missed=0 control_double_bit_missed=11892 control_has_teeth=true sha256_over_crc32c_ratio_bp=260000
```

| # | Criterion | Result |
|---|---|---|
| 1 | Single-bit flip | All six formal arms miss zero at all five widths (the control also misses zero — an additive sum catches single bits) |
| 2 | Exhaustive double-bit | Zero misses on all six formal arms; CRC32C has Hamming distance 4 within 840 bits, so this cell is structural, not probabilistic |
| 3 | Random byte change | All 30 cells of the six formal arms miss zero (the first round once recorded 1 to 6 "misses" per arm, all of which were the same byte changed twice and cancelling to the original; the rig was changed to redraw) |
| 4 | Positive control | sum8 double-bit misses 11 892 / 170 236, the discriminative-power test has teeth |
| 5 | Probe false acceptance | All three numbers are 0 on the six formal arms; sum8 comparing only 1 used byte gives 3919 / 1 000 000 ≈ 1/256, and 0 when the whole field with 31 zero-padding bytes is compared |
| 6 | Cost (105 bytes, ns, minimum of 7 rounds) | sha256 676, blake2s 167, crc64 131, xxh64x4 34, crc32c 26, xxh64 10; maximum spread 2.9%, no cell marked "unstable" |
| 7 | Folded into fsync | 14 headers: sha256 2.63%, blake2s 0.65%, crc64 0.51%, xxh64x4 0.13%, crc32c 0.10%, xxh64 0.03% |

### What These Numbers Say

1. **On input as short as a header, discriminative power cannot tell the six arms apart**: single-bit, double-bit, and random 1-to-8-byte corruption are all missed by no function of 32 bits or more; what remains are only the theoretical random misses (CRC32C and xxh64 at 2⁻³² and 2⁻⁶⁴, 32 bytes at 2⁻²⁵⁶) and cost.
2. **The order of magnitude of "cheaper"**: SHA-256 is 26 times as expensive as CRC32C, but the absolute amount is 2.6% of one fsync versus 0.1% — what "a bit cheaper" buys is 2.5 percentage points of fsync time, not an order of magnitude of throughput.
3. **CRC32C is already a function settled in the repo** (the ciphertext checksum of settled item 2 of D19 (Block Pointer Structure & Width Budget), the reverse chain of settled item 11 of D23 (Journal Role & Format)): taking it as the header checksum introduces no second function; in the 32-byte field it uses 4 bytes and zero-pads the other 28, and probe false acceptance is 0 when the magic is present.
4. These numbers do not stand in for a user settlement: the trade-off between candidates (whether to keep a cryptographic digest for some use after encryption is enabled) is out of range.

### What It Cannot Answer

1. One machine, software implementation, with neither SHA-NI nor the SSE4.2 `crc32` instruction used; the ratio changes with the instructions.
2. Discriminative power covers only flips, random byte changes, and random prefixes; stale whole-sector content and tearing are covered by the whole-unit CRC and the parent-pointer checksum (the reading of E142 (New-Pool File Creation Dry Run)), not here.
3. It does not answer "should the 32-byte field shrink": the width is a settled clause.
4. Timing is a varying observation: N runs differ, relying on the minimum of 7 rounds and the interval assertions of `replay.sh`.

### Affected Decisions

| Decision Item | Relation | Review |
|---|---|---|
| D18 (Information Carried in a Block) settled item 17 | Supports | 2026-09-27 not affected: the terminology rename only touches wording; conclusions, numbers, and artifacts are unchanged (previous review 2026-09-20: not affected: the six candidates all miss zero on the three kinds of header corruption, SHA-256 is far more expensive than CRC32C yet the absolute amount is only a fraction of one fsync, and CRC32C is a function already settled in the repo — the 32-byte field takes CRC32C; the same-day repo-wide rename "system configuration" → "system configuration" only touched wording, conclusions, numbers, and artifacts unchanged) |
| D19 (Block Pointer Structure & Width Budget) settled item 2 | Not affected | 2026-09-27 not affected: the terminology rename only touches wording; conclusions, numbers, and artifacts are unchanged (previous review 2026-09-20: not affected: only takes its ciphertext checksum using CRC32C as a "function already settled in the repo"; the same-day repo-wide rename "system configuration" → "system configuration" only touched wording, conclusions, numbers, and artifacts unchanged) |
| D23 (Journal Role & Format) settled item 11 | Not affected | 2026-09-27 not affected: the terminology rename only touches wording; conclusions, numbers, and artifacts are unchanged (previous review 2026-09-20: not affected: only takes the reverse chain using CRC32C as the same fact; the same-day repo-wide rename "system configuration" → "system configuration" only touched wording, conclusions, numbers, and artifacts unchanged) |

## Historical Versions

### 2026-09-13
- Created. The pre-run registration was written before the rig; the criteria and expected values were never changed. The rig was changed in two places: redrawing when a random byte change cancels to the original (the "misses" wrongly recorded in the first round were all this shape); single-bit flips now report how many positions were flipped (the mutation "skip the bits of the checksum field" previously had no assertion that could see it).
