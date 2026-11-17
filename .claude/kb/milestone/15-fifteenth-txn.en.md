# Milestone "Compression and Encryption": Tasks, Acceptance, and Ordering

[中文](15-fifteenth-txn.md) · **English** · [日本語](15-fifteenth-txn.ja.md)

**Measurement basis**: Planning document, pre-filed on 2026-10-12, serving as a draft. The user settled Milestones 7 through 10, asking "what might you have missed, how do you think milestones should be ordered", and the main agent suggested placing 11 through 15 sequentially, based on the principle that changes to on-disk formats are scheduled before format freeze, and freeze before external availability (`records/2026-10-11-里程碑五打包与决策进度列.md` Section 7); on the same day the user requested "write it into /home/fy5090/code/singlefs/.claude/kb/milestone, pre-fill it", settling this order: "this is acceptable, adjustments needed during execution". **It does not require correctness**: revise before starting each item. Scheduled after Milestone 11 so AAD covers directory entries in one pass, without opening an extra round for directories (inferred). The prerequisite is that all exit criteria for Milestones 3 through 14 are fully met; if any item is missing when previous milestones close out, reschedule this document according to what is missing.

**How to read**: Three fields per item—"Existing in repo" lists which clauses, owed checks, and code this item bears down on in the repository (inspected on 2026-10-12, with sources); "Tasks" is a table: number, what to do, clauses and owed checks borne down upon, acceptance (prove each goes red first), dependencies; "To be settled before starting" lists open questions that must undergo three-way arbitration or ask the user before work begins. Anything labeled "inferred" is a judgment by the main agent without measurement. Task numbering is written as "Item·Point" (I·1, II·3), used solely within this document. Owed check numbers were all open on the day of filing, to be checked individually before starting work.

**Encryption is an optional computation, not an optional format** (`.claude/rules/fs-design.md` section of identical name): reserved fields for MAC, nonce, and algorithm type are preserved across all volumes; this milestone merely attaches that computation to volumes with encryption enabled, without changing bytes on unencrypted volumes. **No new transaction types added**: encryption operates at both unit writeout and readin endpoints, following the same transaction layer; fast paths bypassing COW must not bypass encryption (D9 (Encryption) settled item 17).

## Exit Criteria (One Settled, Remaining Unsettled)

The user settled one exit criterion on 2026-10-22:

| # | Exit Criteria | Belongs to Which Item |
|---|---|---|
| 1 | Unscoped run of crash-point replay and crash amplification: finish judging what is new and keep judging what has never been judged, at least 10 billion states per milestone (settled by user on 2026-10-22). Run in two stages (recording stage on CPU, judging stage on GPU, all observers attached), without narrowing the enumeration domain, all 0 red. The gate selects the blocks to judge by unit key itself: ① the crash streams new in this milestone and the blocks whose unit key changed; ② blocks never judged under the current key, at least 5 billion states. When the two together exceed 10 billion states, this milestone judges at least 10 billion states; when together they come to less than 10 billion, judge all of it. The state count is the states actually judged by the unscoped runs in this milestone. On red, pin it into a case per the "A red from the gate is pinned into a case that can be rejudged on its own" section of `.claude/rules/test-script-discipline.md`, fix it, then rerun. | Crash-point replay and crash amplification (cross-milestone) |

Exit criteria and whether post-encryption key rotation is enabled are left to user decision in Phase 0. Main agent's inferred candidates (inferred, unmeasured):

| # | Candidate Exit Criteria | Belongs to Which Item |
|---|---|---|
| 1 | Whole-volume AEAD: two algorithms, 96-bit nonce stored per extent, full 128-bit MAC, ciphertext checksum, all units on encryption-enabled volumes (except system configuration) are ciphertext | I |
| 2 | KDF and master key slot, system configuration authentication | I |
| 3 | Expected value of each AAD field originates solely from lookup path or already authenticated ciphertext-side fields, including directory entries | II |
| 4 | Keyless side is strictly read-only check-only; crash streams of encrypted volumes whole-domain 0 red | III |
| 5 | Interaction with features landed earlier each has one test case and one must-red world: snapshots, GC, truncation, packing, parallel commit, multiple files, and the transactions added by each milestone ordered before this file (settled by the user on 2026-11-16; the shape follows Milestone 5 Exit 5) | III |

## Grouping and Ordering

| Phase | What to Do | Why Sequenced Here | Status |
|---|---|---|---|
| 0 Handover | Check current state of reserved fields in implementation (`grep -rl nonce crates/singlefs-core/src` hits eleven files (checked 2026-11-16); verify whether all are merely reserved slots before starting); settle exit criteria | Four kinds of reservations in D9 (Encryption) settled item 12 already implemented; inspect their current state first | Unstarted |
| 0 Compression | 0·1 → 0·2 | Encryption works on the extent after compression, so this comes before I·1 | Not started |
| 1 Computation and Keys | I·1 ∥ I·2 | Mutually independent (inferred) | Unstarted |
| 2 AAD | II·1 → II·2 | Depends on I·1 | Unstarted |
| 3 Keyless and Coverage | III·1 ∥ III·2 | Depend on I, II | Unstarted |

**Addendum (settled by the user on 2026-11-16)**: the design round of 0·1 moves up to run alongside Milestone 5 X·1, by three-way argument only and without code; the implementation of 0·2 stays in this file.

**Estimation**: Not estimated. Record `records/2026-10-11-里程碑五打包与决策进度列.md` Section 3 inferred 3–4 weeks (inferred, unmeasured).

## 0. Compression (the user decided on 2026-11-11 to merge it into this file instead of a separate milestone)

- **Already in the repository**: no decision governs the algorithm or the switch of compression; D9 (Encryption) settled item 14 only registers the algorithm code (0 = not compressed), and D9 (Encryption) settled item 13 fixes the order compression → encryption → MAC. Compression changes the on-disk format, so it has to be finished before the freeze of milestone seventeen; the ciphertext checksum and the nonce of encryption are computed over the extent after compression, so it is placed before I·1.
- **Tasks**:

| # | What to do | Clauses and debts it rests on | Acceptance | Depends on |
|---|---|---|---|---|
| 0·1 | Establish the decision (three-way): whether to compress, which algorithms, at which granularity (extent or unit), how to store data that does not compress, where the logical length and the on-disk length are recorded, and the relation to the small-data container (milestone five) and to truncation | D9 (Encryption) settled items 13 and 14; `.claude/rules/fs-design.md`, "the criterion for format branches" | The verdict lands in `research/prompts/` and the settlement is written back to the kb; a decision needs an experiment, at least one counting experiment | stage 0 |
| 0·2 | Implementation: compress on the write path and decompress on the read path, with the algorithm code in the cell the pointer already reserves; the new invariants go into the pool-level checker and the GPU judge | The settlement of 0·1 | One bad image and one mutation row per invariant; differential testing against the model on a compressed volume is identical item by item; the crash streams of a compressed volume are 0 red over the whole domain | 0·1 |

- **Must be settled before starting work**: whether compression enters the first version (if 0·1 rules it out, this section keeps only the decision and the algorithm code stays reserved).

## I. Encryption Computation and Keys

- **Existing in repo**: D9 (Encryption) settled item 10 "whether encryption enters first runnable version: no, only reserve format fields", four kinds of reservations in settled item 12 already implemented. Unimplemented: D9 (Encryption) settled item 11 "whole-volume AEAD, encryption boundary coincides with checksum boundary; encrypt all except system configuration, metadata retains minimal plaintext header"; settled item 1 "pointers contain MAC and ciphertext checksum integrity fields, ciphertext checksum requires no key and serves all read paths"; settled item 2 "take full 128 bits, do not truncate"; settled item 3 "implement both, record algorithm type on disk, select default at runtime based on CPU features; encryption path must support parallelism"; settled item 4 "store 96-bit nonce per extent, do not compute on the fly"; settled item 8 "fsid enters KDF, system configuration must authenticate due to nonce high-water mark, whole-volume rollback undefended"; settled item 13 "nonce must not derive from physical location; fixed order: compression → encryption → MAC"; D18 (Information Carried in a Block) settled item 13 "Code 1 identity segment and declared length become ciphertext in-place, location width unchanged". Owed checks: C74 (No Defense Against Cross-Volume Misdirected Writes in the Period Without Encryption), C185 (unsettled whether payload CRC covers ciphertext or plaintext), C193 (size of unit header when encryption is enabled never calculated across repo), C399 (prerequisite for note on nonce non-reuse never reviewed), C240 (uncomputed cost of re-encryption during relocation).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I·1 | Unit encryption and decryption: two algorithms, nonce generation and storage, MAC, ciphertext checksum, Code 1 identity segment converted to ciphertext | D9 (Encryption) settled items 1, 2, 3, 4, 11, 13; D18 (Information Carried in a Block) settled item 13; C185 (unsettled whether payload CRC covers ciphertext or plaintext), C193 (size of unit header when encryption is enabled never calculated across repo), C399 (prerequisite for note on nonce non-reuse never reviewed); the two places C406 (the read side does not check the algorithm type and the three compression bytes) still owes: the root record and the journal record still skip their own algorithm type (`RootRecord::parse_slot` in `crates/singlefs-core/src/root_record.rs`, `JournalRecord::parse` in `crates/singlefs-core/src/journal.rs`), to be checked before encryption is turned on | Mutation colliding nonces across two writes turns red; corrupting one ciphertext byte turns red on read; round-trip test of both algorithms on identical plaintext | Phase 0 |
| I·2 | KDF, master key slots, system configuration authentication | D9 (Encryption) settled item 8; C74 (No Defense Against Cross-Volume Misdirected Writes in the Period Without Encryption) | Mounting with key from different volume fails; corrupting one byte of system configuration turns red | Phase 0 |

- **To be settled before starting**: Whether C240 (uncomputed cost of re-encryption during relocation) (whether to re-encrypt during GC, packing, rebalance relocation) is calculated here or established separately; whether key rotation is implemented.

## II. AAD

- **Existing in repo**: D9 (Encryption) settled item 16 "permitted solely from reader's lookup path or ciphertext-side fields already authenticated by higher-level MAC; forbidden from plaintext side or reading back from pointers"; D9 (Encryption) settled item 6 binds five logical identity items, explicitly noting "five items blind and silent to snapshot dimension, field table owes one reopening"; D9 (Encryption) settled item 17 parity calculated over ciphertext, replicas share identical ciphertext, birth txg cannot serve as nonce. Owed checks: C63 (Settled AAD Blind to Snapshot Dimension), C107 (no placement for index node AAD identity), C275 (two AAD segments under cross-file reflink lack expected value source), C295 (whether key segment in pointer counts as AAD lookup path), C184 (reconstruction path cannot acquire full nonce).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II·1 | Reopen field table of D9 (Encryption) settled item 6: AAD identities for snapshot dimension, index nodes, directory entries, cross-head sharing (Milestone 14) | D9 (Encryption) settled items 6, 16, 17; C63 (Settled AAD Blind to Snapshot Dimension), C107 (no placement for index node AAD identity), C275 (two AAD segments under cross-file reflink lack expected value source), C295 (whether key segment in pointer counts as AAD lookup path) | Three-way verdicts land in `research/prompts/`, settled text written back to kb | I·1 |
| II·2 | Implement AAD validation | II·1 settled text; C184 (reconstruction path cannot acquire full nonce) | Moving unit to different logical location turns red on read; mutation reading expected value from plaintext side turns red | II·1 |

## III. Keyless Side and Coverage

- **Existing in repo**: D9 (Encryption) settled item 5 "scrub only: when encryption is enabled, keyless side is read-only check-only, without moving, evacuating, or reclaiming"; D3 (Space Allocation) settled item 16 "read-only check-only, neither moving nor reclaiming; resident automatic compaction commitment does not hold when encryption is enabled". Owed checks: C32 (no gate on whether the keyless side can write bytes), C136 (nobody has marked which invariants can be verified without a key), C166 (Whether Format Reservations Retained After Plaintext Side Cancellation).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| III·1 | Keyless mount: read-only check-only, write path writes zero bytes | D9 (Encryption) settled item 5; D3 (Space Allocation) settled item 16; C32 (no gate on whether the keyless side can write bytes), C136 (nobody has marked which invariants can be verified without a key), C166 (Whether Format Reservations Retained After Plaintext Side Cancellation) | Mutation issuing write request from keyless side turns red | I |
| III·2 | Crash streams and differential against model for encrypted volumes; pool-level checker judges with keys | D13 (Verification Route) settled items 9, 15 | Whole-domain 0 red | I, II |

**Not in exit criteria**: Full form of keyless scrub and handling upon detection (Milestone 16 [16-sixteenth-txn.md](16-sixteenth-txn.md)).

## Debts assigned to this file by the re-check of owed checks on 2026-11-11

After the row-by-row re-check against the code (`records/2026-11-11-欠账表按代码重核逐行结论.md`), these debts have their destination written to this file although this file did not name them before; do each as its registered row says under "how to block it", and see each registered row for what is still owed; the assignment was inferred by topic during the re-check and is to be verified before work starts.

- I·1: C68 (Invariant Allowlist Conflicts with Settled Clauses), C111 (class tag lacks replica on ciphertext side), C173 (plaintext header leakage surface left undefended after the plaintext side was cancelled), C400 (whether the MAC in the unit header and the one in the pointer are two copies), C405 (nobody reconciled encryption's need for parallelism with serial-first commit)
- III·1: C303 (the basis of D2 boundary ③ lapsed with the withdrawal of D9 settled item 7)
- III·2: C189 (Whether I-2.3 Judges Plaintext or On-Disk Bytes When Encryption Is On Is Unsettled)
