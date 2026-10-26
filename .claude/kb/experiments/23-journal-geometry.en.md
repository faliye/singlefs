## E23 Journal Geometry — Run (2026-08-29)

[中文](23-journal几何.md) · **English** · [日本語](23-journal-ジオメトリ.ja.md)

**What is measured**: Fixed-length ring vs chained log, and where tail is stored.

**Evaluates [decisions.md](../decisions.md) D23 (Journal Role & Format) Settled Item 2 (fixed ring vs chained log) and Settled Item 3 (tail location).**
These two items **are not orthogonal and must be measured together**: the authoritative location of tail determines when space can be reclaimed,
and "when space can be reclaimed" is precisely the watershed between ring and chain. Measuring them separately yields a neat answer for each in isolation.

### Conclusions

1. **Settled Item 3: Cost difference between the two arms is 21 : 500, a one-sided blowout.**

   | Tail Variant | Steady-State Extra Writes | Replay Needed After Idle Crash |
   |---|---|---|
   | `tail_system_configuration` (jbd2: resides in journal system_configuration, in-place overwrite, FUA) | **21 blocks** (= checkpoint count) | **0** |
   | `tail_inline` (XFS: inlined in `tail_lsn` of each record header) | **0** | **500 blocks** (pbs=4096) / 63 blocks (pbs=512) |

   20000 fsyncs, one checkpoint every 1000. Steady-state overhead of `tail_system_configuration` is **0.105%**
   (21 / 20000), traded for zero replay volume after idle crash.
   **Mechanism**: `tail_inline` can only hitchhike on records—without writing records, tail cannot advance;
   `tail_system_configuration` can write once independently upon checkpoint completion. Idle crashes thus diverge.
   ⚠️ Changing measurement basis to barriers preserves the conclusion: 21 extra FUAs against 40000 persistence points of 20000 fsyncs is 0.05%.

2. **Settled Item 2: Ring and chain are identical on every metric except one, which is allocation count.**

   | | Fixed Ring | Chained |
   |---|---|---|
   | Steady-state written blocks | Identical | Identical |
   | **Requests blocks from allocator** | **0 times** | **Once per journal block** (20000 fsyncs ⇒ 20000 times) |
   | Deadlock 3 (single operation larger than ring) | Exists, requires geometric invariant | Does not exist |

   ⇒ **Chained log turns the journal write path into an allocation path**, entering the cycle of D23 (Journal Role & Format) Deadlock 2
   (checkpoint → COW writes new node → requires allocation → disk full → cannot issue root).
   The 0 allocations of a ring is precisely how "journal cannot be covered by COW" manifests in the model.
   **Chained log trades "eliminating Deadlock 3" for "aggravating Deadlock 2", and Deadlock 2 is unique to COW and already the hardest one.**

3. **Lower bound of ring size = checkpoint interval × record blocks per fsync, amplified 8× with atomic width.**

   | pbs | Ring ≥ how many blocks to avoid forced checkpoint | Peak Occupancy |
   |---|---|---|
   | 512 | **256** | 125 blocks |
   | 4096 | **1024** | 1000 blocks |

   When ring is too small, penalty is cliff-like (pbs=4096, 1 named item per fsync):

   | Ring Size | Forced Checkpoint Count | Tail Writes of `tail_system_configuration` |
   |---|---|---|
   | 4 blocks | **4980** (once every 4 operations) | 5000 |
   | 16 blocks | 1240 | 1260 |
   | 64 blocks | 300 | 320 |
   | 256 blocks | 60 | 80 |
   | 1024 blocks | **0** | 20 |

   ⇒ **Settled Items 2, 3, and 4 are tightly coupled**: atomic width (Settled Item 4) determines how many blocks records occupy,
   record block count determines how large ring must be (Settled Item 2), and an undersized ring amplifies `tail_system_configuration` writes by 250× (Settled Item 3).
   **Deciding any single item in isolation leads to the wrong decision.**

### Arithmetic of Settled Item 4: Header 84 Bytes, Fits

⚠️ **The field table under "Arithmetic of Settled Item 4: Header 84 Bytes, Fits" is the measured arithmetic from the version where `jsn` occupied 8 bytes**,
which was the genuine run on 2026-08-29, preserved verbatim.
D23 (Journal Role & Format) Settled Item 9 (2026-08-30) settled `jsn` as 10 bytes (instance generation 32 bits + counter 48 bits);
on 2026-08-31 the 8 bytes of `tail_lsn` were removed (Settled Item 3 verbatim settled tail not inlined in record header,
and "inlined `tail_lsn`" is precisely the definition of the rejected XFS pattern)
⇒ **those 10 fields total 78 bytes**; D23 (Journal Role & Format) Settled Item 4 plus Settled Items 7 / 8 / 13 three increments totaling 17 **yields 95**, plus in-publish sequence number 4, Settled Item 15 new root segment 373, fsid 8, and MAC 16, **current header is 496 bytes, leaving 16 on 512** (0 named items), leaving 3600 on 4096 (64 named items).
**When citing the field table, use the 10 fields excluding `tail_lsn` from the 11 rows of E23 (Journal Geometry) "Header 84 Bytes, Fits"; when citing header byte count, use 496 (D23 (Journal Role & Format) Settled Item 4).**

| Field | Bytes | Basis |
|---|---|---|
| magic | 4 | Recognizes record header |
| version/type | 2 | D23 (Journal Role & Format) Unsettled Item A (type field + unknown type refuses mount) |
| algo_type | 1 | D9 (Encryption) Settled Item 12 Item 4 |
| padding | 1 | |
| record_length | 4 | D23 (Journal Role & Format) Unsettled Item B (self-describing length, bounds check before checksum) |
| named_count | 4 | Count of named items; items themselves reside in payload |
| jsn | 8 | D23 (Journal Role & Format) Settled: globally monotonic per record, prefix determination |
| checkpoint_txg | 8 | D23 (Journal Role & Format) Settled: belongs to which checkpoint |
| tail_lsn | 8 | Needed only if Settled Item 3 chooses XFS variant; omitted for jbd2 variant |
| nonce | 12 | D9 (Encryption) Reserved Item 1 |
| header_csum | 32 | Covers header only—prerequisite for distinguishing "head landed, tail did not" |
| **Total** | **84** | Occupies 16% of 512 sector, leaving 428 B (fits 7 named items) |

**Alignment Cost** (requiring "record header lands entirely within one atomic unit"):

| pbs | Named 1 Item (140 B) | Named 10 Items (644 B) | Named 100 Items (5684 B) |
|---|---|---|---|
| 512 | 512 B (3.7×) | 1024 B (1.6×) | 6144 B (1.1×) |
| 4096 | **4096 B (29.3×)** | 4096 B (6.4×) | 8192 B (1.4×) |

⇒ On 4Kn, an fsync record naming only 1 item wastes 3956 B.
Without alignment, multiple records packing into the same atomic unit wastes no space, **but none except the first can judge tears**.

### Measurement Basis and Known Limitations (Not to Be Ignored)

- **Pure counting model, touches no devices.** Counts **blocks**, not bytes nor barriers—
  the two measurement basis owed checks [checks-owed.md](../checks-owed.md) C27 (Format Parameters Settled on a Block-Count Measurement Basis) / C26 (barrier count never measured) apply equally to E23 (Journal Geometry).
- **Does not model the cost of checkpoint itself.** The true cost of forced checkpoint is writing all dirty ancestors
  (see E16 (Role of Journal: WAL vs Intent Log)), E23 (Journal Geometry) only counts how many times it occurs.
- **Replay volume is calculated by blocks only, without random reads during replay.** Chained replay must hop block-by-block along pointers,
  while ring can do sequential read-ahead—a layer E23 (Journal Geometry) cannot reach.
- Code `scripts/experiment-apparatus/e23_journal_geom.rs` (`cargo run --release --bin e23_journal_geom`),
  raw output `e23-journal-geom-2026-08-29.out`. Three rounds byte-identical.

### Two Pitfalls Trapped in the 08-29 Round, Both Caught by Mutation Testing

1. **The `pbs` dimension was originally dead code.** First version computed by "how many blocks each record occupies",
   and when `pbs ≤ BLOCK`, "aligning to pbs first then rounding up to 4096" was an **identity operation**,
   yielding identical block counts for 512 and 4096. Mutation testing removed alignment completely, and **zero of the 8 tests failed red**.
   ⇒ Modified to accumulate bytes and fold into blocks once at the end, allowing packing to emerge (8 records packing into one block under 512 sectors).
2. **Tests transcribed `84` / `56` as their own literals**, so changing constants turned zero tests red.
   ⇒ Assertions uniformly reference constants.

### Another Pitfall Caught Not by Mutation Testing, but by Numeric Contradiction

First run produced `rec_blocks=2500` but `allocs=20000`—packing took effect in record blocks, but not in allocation count.
Cause was **running an old binary**: `cargo test` only builds test hosts, not rebuilding release binaries.
⇒ After modifying model, must run `cargo build --release --bin` before rerun,
or have result lines carry a tag matching the build.

### Impacted Decisions

| Decision Item | Relationship | Review |
|---|---|---|
| D23 (Journal Role & Format) Settled Item 2 | Support | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-04 unaffected: updated header width, new root segment, and named items to current values; conclusions and artifacts unchanged |
| D23 (Journal Role & Format) Settled Item 3 | Support | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-04 unaffected: updated header width, new root segment, and named items to current values; conclusions and artifacts unchanged |
| D23 (Journal Role & Format) Settled Item 4 | Support | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-04 unaffected: updated header width, new root segment, and named items to current values; conclusions and artifacts unchanged |
| D23 (Journal Role & Format) Settled Item 23 | Support | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-04 unaffected: updated header width, new root segment, and named items to current values; conclusions and artifacts unchanged |
| D20 (Load-Bearing Surface: Unit Atomicity & Self-Containment) Settled Item 5 | Support | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-04 unaffected: updated header width, new root segment, and named items to current values; conclusions and artifacts unchanged |
| D9 (Encryption) Settled Item 12 | No impact | 2026-10-22 unaffected: only the format widths in sentences describing the current state were changed; previously, 2026-10-21 unaffected: only the trilingual language-switch row was added; previously, 2026-10-04 unaffected: updated header width, new root segment, and named items to current values; conclusions and artifacts unchanged |

## Historical Versions

Historical entries for E23 (Journal Geometry) are gathered in [experiments-history.md](../experiments-history.md).
