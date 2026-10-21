## E143 Journal Accounting Under One Transaction One Unit — Run (2026-09-13, Pure Arithmetic, 6 Unit Tests / 10 Mutations All Caught)

[中文](143-一事务一单元下的journal账.md) · **English** · [日本語](143-一事务一单元下的journal账.ja.md)

What is being asked is how much must be paid under user settlement (splitting discipline is one transaction per unit, last paragraph of D16 (Publish semantics) settled item 5 as before) for the ledger of C310 (Conflict between transaction splitting discipline and record count measurement basis):
across the six workload points of D25 (Target workload priority), how many records per fsync, what percentage of user data journal occupies, and how large a ring is needed for a full T_dirty window.
**All six criteria passed, cross-apparatus gate established**: Arm Bing calculated 1847 records / 7388 KiB per measurement basis of E91 (Ring and Acceptance Accounting Under Settled Format), verbatim identical.
**Primary numbers**: Arm Jia (of clause) on seq batch 1 incurs 8 records per fsync, journal taking 12.5% of user data; full T_dirty window takes 65 536 records, 256 MiB, 4 times the anticipated 64 MiB ring;
Arm Yi (one record packing multiple transactions, control) takes 1 record, 1.56%, full window 4232 KiB. Reports numbers only, without judging whether clause should change.

**Re-run command** (`exact` mode, byte-for-byte comparison against committed artifact):

```bash
bash scripts/experiments/replay.sh E143
# Run apparatus directly:
cd scripts/experiments && cargo run --release --bin e143-one-unit-per-txn-journal
```

Code `scripts/experiments/e143_one_unit_per_transaction_journal.rs`, raw output `e143-one-unit-per-txn-journal-2026-09-13.out` (34 lines, final line `emitted=34`),
pre-run registration `e143-preregistration.md`, mutation table `scripts/mutation/tables/e143_one_unit_per_transaction_journal.tsv`.

### This Is Pure Arithmetic, Not Measurement

No I/O, no random source ⇒ running N times is byte-for-byte identical. Evidence strength derives from absolute values pinned across 6 unit tests + 10 mutations all caught:
record header changed back to 78, Yi item width changed to 56, Jia opening separate record for ancestors, seq batch 1 changed to 12 leaves, disk count changed to 1, Bing full window calculated by unit count, F changed to 2, ring changed to 128 MiB, Bing one transaction per unit, T_dirty changed to 1 GiB; each has a test turning red.

### Tested Object and Arms

| Arm | How transactions are split | What one record holds | Items packed per record |
|---|---|---|---|
| Jia: Today's clause | One transaction per data unit (last paragraph of D16 (Publish semantics) settled item 5) | One transaction (D23 (Role and format of journal) settled item 7: one transaction number per record); ancestors merged into last record of this fsync | (4096 − 95) ÷ 56 = 71 |
| Yi: Control | One transaction per data unit | Multiple transactions: named items each carry transaction number 8, record header drops transaction number and commit mark, adds first and last transaction numbers 16 | (4096 − 102) ÷ 64 = 62 |
| Bing: Control (rejected by user) | One transaction per fsync (old reading of E75 (Record Dimensions and Ring Geometry)) | One transaction | 71 |

Workload points follow D25 (Target workload priority): rand batch 1 (1 leaf + 4 ancestors), multistream batch 1 (1 + 4), seq batch 1 (8 + 4), metaheavy batch 1 (2 + 5.937, ancestors rounded to 6 by item count), seq batch 10 (80 + 4), multistream batch 10 (10 + 31);
"leaves" treated as data units per reading of E75 (Record Dimensions and Ring Geometry).

### Verdict (Per Pre-Run Hardcoded Criteria, Transcribed Verbatim from Artifact)

```text
E7RESULT name=capacity arm=jia items_per_record=71
E7RESULT name=capacity arm=yi items_per_record=62
E7RESULT name=fsync load=seq batch=1 leaves=8 ancestors=4 arm=jia transactions=8 records=8 journal_bytes_two_devices=65536 data_bytes_two_devices=524288 journal_over_data_bp=1250 records_per_second=22280 journal_kib_per_second=178240
E7RESULT name=fsync load=seq batch=1 leaves=8 ancestors=4 arm=yi transactions=8 records=1 journal_bytes_two_devices=8192 data_bytes_two_devices=524288 journal_over_data_bp=156 records_per_second=2785 journal_kib_per_second=22280
E7RESULT name=fsync load=seq batch=10 leaves=80 ancestors=4 arm=jia transactions=80 records=80 journal_bytes_two_devices=655360 data_bytes_two_devices=5242880 journal_over_data_bp=1250 records_per_second=222800 journal_kib_per_second=1782400
E7RESULT name=fsync load=multistream batch=10 leaves=10 ancestors=31 arm=jia transactions=10 records=10 journal_bytes_two_devices=81920 data_bytes_two_devices=655360 journal_over_data_bp=1250 records_per_second=27850 journal_kib_per_second=222800
E7RESULT name=window arm=jia records=65536 bytes_kib=262144 ring_kib=65536 window_over_ring_milli=4000
E7RESULT name=window arm=yi records=1058 bytes_kib=4232 ring_kib=65536 window_over_ring_milli=64
E7RESULT name=window arm=bing records=1847 bytes_kib=7388 ring_kib=65536 window_over_ring_milli=112
E7RESULT name=ring arm=jia worst_transaction_bytes=4096 lower_bound_any_transaction_kib=12 lower_bound_dirty_window_kib=786432
E7RESULT name=ring_cap arm=jia ring_kib=65536 safety_factor=3 dirty_bytes_allowed=178946048 dirty_kib_allowed=174752
E7RESULT name=keys load=seq batch=1 arm=jia data_units=8 distinct_write_orders=8
E7RESULT name=verdict jia_records_seq_batch1=8 jia_journal_over_data_bp_seq_batch1=1250 bing_window_records=1847 bing_window_kib=7388 e91_anchor_holds=true jia_window_over_ring_milli=4000
```

| # | Criterion | Result |
|---|---|---|
| 1 | Records per fsync | Jia 1 / 1 / 8 / 2 / 80 / 10, Yi 1 / 1 / 1 / 1 / 2 / 1, Bing 1 / 1 / 1 / 1 / 2 / 1, matching registry cell-by-cell |
| 2 | Journal share of user data | Jia 12.5% (identical across six workload points, strictly one record per data unit), Yi seq batch 1 is 1.56% |
| 3 | Cross-apparatus gate | Bing matches E91 (Ring and Acceptance Accounting Under Settled Format) measurement basis at 1847 records, 7388 KiB, verbatim identical |
| 4 | Full T_dirty window | Jia 65 536 records, 262 144 KiB, 4.0 times 64 MiB ring; Yi 1058 records, 4232 KiB |
| 5 | Two readings of I-8.1 (Ring geometry sufficiently large) | Reading by "any transaction", Jia worst-case single transaction is 1 record, F = 3 lower bound 12 KiB; reading by "full window fitting in ring", Jia requires 768 MiB, or T_dirty squeezed to 174 752 KiB (~170.7 MiB) |
| 6 | Mapping key uniqueness | Jia, Yi next fsync 8 data units have 8 distinct write orders; Bing has only 1 (precisely the cell where D19 (Block pointer structure and width budget) settled item 6 staked on splitting discipline) |

### What These Numbers Indicate

1. **The sentence in D23 (Role and format of journal) settled item 12 "target workload 12-item transaction takes exactly 1 record ⇒ 5.9x margin" follows Bing's reading**. Under user-settled splitting discipline, seq batch 1 is 8 records, seq batch 10 is 80 records; margin cell no longer exists,
   each record packing only 1 to 5 named items (using 151 to 375 bytes within 4096 bytes).
2. **Ring geometry constraint moves from "any transaction" to "full window"**: verbatim I-8.1 (Ring geometry sufficiently large) governs worst-case occupancy of any transaction, which under Jia is 1 record, lower bound 12 KiB, making constraint essentially absent;
   what truly constrains is full T_dirty window 256 MiB vs 64 MiB ring — either ring ≥ 768 MiB (F = 3), or T_dirty ≤ 170.7 MiB, or records pack multiple transactions (Yi).
3. **Yi is not a candidate clause**: it requires changing record header of D23 (Role and format of journal) settled item 7 (dropping transaction number and commit mark, named items carrying transaction number), which is independently frozen component settled by D16 (Publish semantics) settled item 3; it merely serves as control for "12.5% and 4x".
4. Journal write bandwidth at 2785 fsyncs per second on seq batch 10 is 1.7 GiB/s (two disks combined), whose ratio to data write bandwidth 5 GiB/s remains 12.5%.

### What It Cannot Answer

1. Pure arithmetic: does not simulate ring wrap-around, does not measure time; which record names ancestor nodes is unwritten in clause, Jia adopts "merging into last record".
2. Record header modification in Yi was constructed by apparatus as control, not a candidate.
3. Whether T_dirty "dirty bytes" counts data units or nodes is unwritten in D16 (Publish semantics) settled item 5; Jia, Yi count data units, Bing counts by node measurement basis of E91 (Ring and Acceptance Accounting Under Settled Format), three not compared on identical measurement basis.
4. Determinism: running N times identical.

### Affected Decisions

| Decision item | Relationship | Review |
|---|---|---|
| D23 (Role and format of journal) settled item 19 | Supports | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D23 (Role and format of journal) settled item 12 | Material | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D23 (Role and format of journal) settled item 7 | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D16 (Publish semantics) settled item 5 | Material | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D19 (Block pointer structure and width budget) | Material | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |
| D25 (Target workload priority) | Does not affect | 2026-10-11 unaffected: wording only (temporal terms, positional references, self-references); conclusions, numbers, and artifacts unchanged |

## Historical Versions

### 2026-09-13
- Created. Pre-run registration written prior to apparatus; single run, criteria and expected values never modified.
