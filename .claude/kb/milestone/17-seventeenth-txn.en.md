# Milestone "Format Freeze": Tasks, Acceptance, and Ordering

[中文](17-seventeenth-txn.md) · **English** · [日本語](17-seventeenth-txn.ja.md)

**Measurement basis**: Planning document, pre-filed on 2026-10-12, serving as a draft. The user settled Milestones 7 through 10, asking "what might you have missed, how do you think milestones should be ordered", and the main agent suggested placing 11 through 15 sequentially, based on the principle that changes to on-disk formats are scheduled before format freeze, and freeze before external availability (`records/2026-10-11-里程碑五打包与决策进度列.md` Section 7); on the same day the user requested "write it into /home/fy5090/code/singlefs/.claude/kb/milestone, pre-fill it", settling this order: "this is acceptable, adjustments needed during execution". **It does not require correctness**: revise before starting each item. FUSE mount ([12-twelfth-txn.md](12-twelfth-txn.md)) is handed over to external users only after freeze. The prerequisite is that all exit criteria for Milestones 3 through 16 are fully met; if any item is missing when previous milestones close out, reschedule this document according to what is missing.

**How to read**: Three fields per item—"Existing in repo" lists which clauses, owed checks, and code this item bears down on in the repository (inspected on 2026-10-12, with sources); "Tasks" is a table: number, what to do, clauses and owed checks borne down upon, acceptance (prove each goes red first), dependencies; "To be settled before starting" lists open questions that must undergo three-way arbitration or ask the user before work begins. Anything labeled "inferred" is a judgment by the main agent without measurement. Task numbering is written as "Item·Point" (I·1, II·3), used solely within this document. Owed check numbers were all open on the day of filing, to be checked individually before starting work.

**Rules change after freeze**: `.claude/rules/format-evolution.md` states "once external users exist, this document is superseded, replaced by strict compatibility—at that time modify this document directly and record the overturn rationale in decision changelog". After freezing a component, modifying its bytes requires an incompat bit (table in D15 (format freeze policy) settled item 4). **No new transaction types added**.

## Exit Criteria (Unsettled)

Exit criteria and which components to freeze are left to user decision in Phase 0. Main agent's inferred candidates (inferred, unmeasured):

| # | Candidate Exit Criteria | Belongs to Which Item |
|---|---|---|
| 1 | Extension points land: additions after freeze require incompat, complete prior to freeze | I |
| 2 | `format-spec/<component>.toml` generated from kb field tables, `spec_hash` strictly evaluated over it; freezing a component means committing `.claude/kb/format-frozen/<component>.toml` | II |
| 3 | Two checks that can turn red: format freeze contract (`spec_hash` comparison) and backward compatibility regression (golden image) | II |
| 4 | `.claude/rules/format-evolution.md` updated to strict compatibility, changelog records overturn rationale | III |

## Grouping and Ordering

| Phase | What to Do | Why Sequenced Here | Status |
|---|---|---|---|
| 0 Handover | Inventory formats modified in 3 through 16, and still-open "format-modifying" unsettled items (D2 (RAID Stripe Policy) unsettled item 21, D26 (background compaction and placement reclamation) unsettled item 10); settle exit criteria | Cannot freeze before format-modifying items are settled | Unstarted |
| 1 Extension Points | I·1 | Final addition of formats before freeze | Unstarted |
| 2 Freeze Apparatus | II·1 → II·2 | Spec precedes comparison | Unstarted |
| 3 Freeze | III·1 | Freeze only after all turn green | Unstarted |

**Estimation**: Not estimated.

## I. Extension Points

- **Existing in repo**: D21 (Boundary Between Authoritative and Derived States) settled item 14 "a reserved region plus an interface, containing a pointer rather than opaque bytes"; settled item 17 five hard constraints (falls within checksum / authentication coverage, checker does not judge its content, size declared by format, targeted object follows allocator and accounting of this project, pointing to core layer objects forbidden; lifecycle tied to host unit); settled item 2 "no a priori constants, upper bound evaluated by mount-time verdicts"; settled item 3 "a pointer is 7 bytes with byte unit (intra-quota offset); field stores 'offset + 1', all zero indicates empty"; settled item 4 "data unit carries; index node conditionally carries; self-describing unit does not carry"; settled item 7 "must be a unit of this project". All six items are unimplemented. Owed checks: C102 (index node header has no reserved area, adding fields is not compat), C115 (inode attribute overflow path has no placement) (Milestone 11 I·2 settles placement first).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| I·1 | Extension points: reserved space, pointer field, quota, interface | D21 (Boundary Between Authoritative and Derived States) settled items 2, 3, 4, 7, 14, 17; C102 (index node header has no reserved area, adding fields is not compat) | Extensions pointing to core layer objects rejected; mutation where extension point falls outside MAC coverage turns red | Phase 0 |

## II. Freeze Apparatus

- **Existing in repo**: D15 (format freeze policy) settled item 3 "evaluated strictly over `format-spec/<component>.toml`, that toml being a full projection of kb field tables, including row order and non-scalar on-disk semantic rules"; settled item 5 "freeze = committing `.claude/kb/format-frozen/<component>.toml`, humans decide only whether to initiate, machines decide whether the request is valid"; settled item 11 "format freeze contract (`spec_hash` comparison) and backward compatibility regression (golden image), image artifacts omitted from git, checklist strictly append-only"; D16 (publish semantics) settled item 3 journal is third independent freeze component outside four layers. `grep -rli spec_hash crates/*/src` hits zero. Owed checks: C119 (frozen components lack spec files), C226 (spec is transcribed manually, not generated from field tables), C225 (component freeze moment bypasses gate triggering), C9 (covert format changes after freeze), C227 (golden image baseline written by its own code), C10 (historical unit formats cannot be read), C97 (altering field semantics without altering width turns red nowhere across repo), C406 (read side does not judge algorithm type and those three compression bytes).
- **Tasks**:

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| II·1 | Generate `format-spec/<component>.toml` from kb field tables, calculate `spec_hash` | D15 (format freeze policy) settled item 3; D16 (publish semantics) settled item 3; C119 (frozen components lack spec files), C226 (spec is transcribed manually, not generated from field tables), C97 (altering field semantics without altering width turns red nowhere across repo) | Modifying a row in field table without regenerating turns red | I |
| II·2 | Freeze contract check and golden image regression | D15 (format freeze policy) settled items 5, 11; C9 (covert format changes after freeze), C225 (component freeze moment bypasses gate triggering), C227 (golden image baseline written by its own code), C10 (historical unit formats cannot be read), C406 (read side does not judge algorithm type and those three compression bytes) | Modifying one byte after freeze without incompat bit turns red; golden image written by separate implementation (C227 (golden image baseline written by its own code)) | II·1 |
| II·3 | Two gates for layout lines: gate evaluates affected layout set from diff and prints (settle directory conventions for layout-specific vs shared paths first; touching shared code mandates full run); during the same round where each layout is executed, run its own checker against its known bad images, archiving known bad images alongside format version and golden image; when `.claude/gate.d/layouts.tsv` contains only one layout, self-prove using an extra sample layout row in gate fixtures (inferred) | D12 (Target Media) settled items 1, 7; C8 (gate scope cannot be determined), C13 (checker verdict becomes ineffective) | Diff touching only shared enum judged as full run, diff touching only single layout specific path judged as running only that layout, failing to obtain set or empty set with non-empty diff turns red; known bad image failing to report error turns red | II·2 |

## III. Freeze

| # | What to Do | Borne Clauses and Owed Checks | Acceptance | Dependencies |
|---|---|---|---|---|
| III·1 | Commit `format-frozen/<component>.toml` per component; modify `.claude/rules/format-evolution.md` to strict compatibility | D15 (format freeze policy) settled items 5, 6, 7 | Differential against model and crash-point replay each run at least one round over format fingerprint about to be frozen (D15 (format freeze policy) settled item 6 checklist line 5); remaining checklist lines passed item-by-item | II |

- **To be settled before starting**: Which components to freeze, and which to leave unfrozen initially (settled by user).

## Unschedulable Items

Not in any milestone from 3 through 17; scheduled when respective conditions are met:

| What | Current Status |
|---|---|
| GPU offload | D24 (Offloading Heavy Background Work to GPU) settled item 1 deferred, unscheduled; entry criteria in [04-fourth-txn.md](04-fourth-txn.md) "Optimization items: not in exit criteria, entry criteria registered" |
| Third-party pipeline | D17 (Implementation Layering & Third-Party Pipeline) settled items 1, 3, 7 unimplemented; owed check C16 (pipeline declares failure model exception) moved from Milestone 3 on 2026-10-18 belongs to this row |
| zoned / FDP, reclamation delegated to disk | D26 (background compaction and placement reclamation) unsettled items 8, 9, 10, user settled no deliberation in near term |
| Compression | No decision governs compression algorithm and switches; D9 (encryption) settled item 14 only registers algorithm code (0 = no compression), D9 (encryption) settled item 13 settles compression → encryption → MAC order. Formulate decision first if needed; if modifying format, schedule before this document |
| Real-disk power cut apparatus | Milestone 9 Item IV, parallel track, awaiting hardware |
