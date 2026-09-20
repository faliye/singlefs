Task: fill in the tables below using only the quoted rule text and the fixed facts given. Do not use any code, do not run anything, just compute and map by hand from what is given.

Format rules for your answer:
1. Number your answers 1 through 16, matching the question numbers in section 6.
2. Do not use any markdown emphasis at all: no bold, no italics, no heading markers, no asterisks, no pound signs used as headings. Plain sentences and plain pipe-delimited tables only.
3. After every answer, add one line starting with "Would be overturned by:" that states what observation would prove that answer wrong.
4. Do not cite any code line number or file line number anywhere in your answer. If you need to point at something, name the function, the enum member, or refer to a table row letter or number given below.
5. Do not use the Rust path separator, meaning two colons together, anywhere in your answer.
6. Do not use any Chinese characters or Chinese-derived variable names anywhere in your answer.
7. If you find yourself about to write a number that looks like a source-code line number or a file line number, stop and replace it with a function name, an enum member name, or a table row letter or number instead, per rule 4.
8. Fill every cell asked for. Do not answer a whole row with only yes or no; show the arithmetic or the specific words that led to your answer.

Background: this is about a filesystem design. All quoted text below is a faithful English translation of source code comments or settled design-decision documents; each quote is tagged with its source file and, where useful, the name of the function or enum it comes from. These citations were checked against the source file before this prompt was written, so you may treat them as accurate. There are two separate topics: (topic one, section 3) arithmetic about how many allocation records a publish operation adds and whether that goes over a fixed capacity of 812 records; (topic two, sections 5) mapping eight plain-language definitions of refusal reasons onto a short list of named candidate reasons used elsewhere in the same system. The two topics do not depend on each other; answer them independently.

A short glossary used throughout: a publish is one write operation that produces a new version of the filesystem. A rewritten role is one of the parts of that version which this particular publish writes anew (for example, the data holding a file's bytes, or the root of an index tree); the exact count of rewritten roles for a given publish is given to you directly in each row, you do not need to derive it. A device is one physical disk in the storage pool. A root is a small record that anchors one version of the filesystem; a root ring is a fixed-size list of the most recent roots kept on disk. An instance is a numbered timeline; a new instance begins whenever the filesystem is mounted for writing after a clean or unclean stop, or after an administrator rolls back. A rollback is an administrator action that picks an older root and starts a new instance from it. A placement is the choice of where on disk a piece of data is written.

Section 1: quoted rule text about the allocation-record arithmetic (topic one)

Quote L1 (source: crates/singlefs-core/src/transaction.rs, comment inside the function admission_of_one_publish): "The allocation record tree's first version has only one node: if it does not fit after this publish, return an error here; this must not be allowed to reach the assertion inside the function that builds an index node. Release only overwrites a record, it does not add a new one; this publish adds one record per rewritten role per device."

Quote L2 (source: crates/singlefs-harness/src/model.rs, comment above the function capacity_wall_is_permitted, describing the true-count side of the allocation-record wall, which is checked using the function root_whose_allocation_records_the_wall_counts): "The true-count side: the executor counts, on the mirror, using the checker's own parsing, the admission base, meaning the version named by the function root_whose_allocation_records_the_wall_counts, plus the count of records added per the admission rule for each publish in the plan up to that point, meaning one record per rewritten role per device, kept as the upper-bound admission by user decision on 2026-09-18; only once this exceeds 812 does it count as not fitting. If the true count is less than or equal to 812 and the implementation refused, that is a mismatch; if the base count cannot be determined, a refusal is not permitted either."

Fixed fact F1 (source: crates/singlefs-harness/src/model.rs, an assertion inside the function the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812): the allocation record node capacity is asserted to equal 812.

Section 2: computation rules for topic one (these are plain-language restatements written for this exercise, not verbatim quotes; keep them separate from section 1 in your reasoning)

Rule R1 (restated from Quote L1 and Fixed fact F1): records_added_by_this_publish equals the number of rewritten roles for this publish multiplied by the number of devices in the pool. running_total_after_this_publish equals base_count_on_mirror plus records_added_by_this_publish. If running_total_after_this_publish is greater than 812, the real admission function in the implementation returns an error named AllocationRecordsExceedOneNode and the publish is refused before anything is written. If running_total_after_this_publish is less than or equal to 812, the real admission function lets the publish proceed past this check.

Rule R2 (restated from Quote L2): a refusal that cites the allocation-record capacity wall is accepted as matching the ideal model only when running_total_after_this_publish, computed from the true base count on the mirror, is greater than 812. If the base count cannot be counted on the mirror, a refusal that cites this wall is never accepted as matching, no matter what the real running total would have been.

Section 3: table T, fixed facts (topic one). Each row gives you base_count_on_mirror (the number of allocation records already on the mirror, before this publish, on the version named by the rule in Quote L2), rewritten_roles_this_publish (the count of rewritten roles for this one publish), devices (the number of devices in the pool), and reused_a_recycled_record (whether this publish's new allocation, according to someone describing it informally, reuses a slot that a previous allocation record had already marked released).

Table T
row T1 | base_count_on_mirror: 808 | rewritten_roles_this_publish: 2 | devices: 2 | reused_a_recycled_record: no
row T2 | base_count_on_mirror: 809 | rewritten_roles_this_publish: 2 | devices: 2 | reused_a_recycled_record: no
row T3 | base_count_on_mirror: 807 | rewritten_roles_this_publish: 2 | devices: 2 | reused_a_recycled_record: no
row T4 | base_count_on_mirror: 808 | rewritten_roles_this_publish: 2 | devices: 2 | reused_a_recycled_record: yes
row T5 | base_count_on_mirror: 796 | rewritten_roles_this_publish: 8 | devices: 2 | reused_a_recycled_record: no | note: these two numbers, 796 and 8, are taken directly from an existing test named the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812
row T6 | base_count_on_mirror: 797 | rewritten_roles_this_publish: 8 | devices: 2 | reused_a_recycled_record: no | note: same source test as row T5
row T7 | base_count_on_mirror: 800 | rewritten_roles_this_publish: 4 | devices: 3 | reused_a_recycled_record: yes | note: devices equal to 3 is a hypothetical value used here only to test whether the multiplication rule is applied correctly for a device count other than 2; it is not a claim about how many devices the shipped first version actually supports
row T8 | base_count_on_mirror: cannot be counted on the mirror | rewritten_roles_this_publish: 2 | devices: 2 | reused_a_recycled_record: no

Section 4: questions about table T (topic one). For each row T1 through T8, answer these six things: (a) records_added_by_this_publish, (b) running_total_after_this_publish, or state that it cannot be computed, (c) whether running_total_after_this_publish is greater than 812, equal to 812, or less than 812, or state that this cannot be determined, (d) what the real admission function in the implementation does with this publish: lets it proceed past this check, or returns the error AllocationRecordsExceedOneNode, or state that this cannot be determined, (e) whether the fact that reused_a_recycled_record is yes or no changes your arithmetic in this row, answer yes or no and say why using Rule R1, (f) whether a refusal that cites the allocation-record capacity wall here would be accepted as matching the ideal model, per Rule R2, answer yes, no, or cannot determine.

Section 5: quoted rule text about the two mapping tables (topic two)

Quote L3 (source: crates/singlefs-core/src/mount.rs, enum RollbackCandidateExclusion, member NotInRing): "There is no readable root slot for that (instance, txg) pair in the root ring: the candidate set is the roots that are in the root ring (decision D23, settled item 14)."

Quote L4 (source: crates/singlefs-core/src/mount.rs, enum RollbackCandidateExclusion, member BelowEffectiveFloor): "The txg is below the effective rollback floor F (decision D16, settled item 1, table row rollback candidate set: txg is greater than or equal to F effective)."

Quote L5 (source: crates/singlefs-core/src/mount.rs, enum RollbackCandidateExclusion, member OnAbandonedTimeline): "The instance table pointed to by the newest root has a row (i, Ti, Wi) for that instance, and the target's txg is greater than Ti: a root on an abandoned timeline (decision D23, settled item 14, table row judged still valid according to the instance table)."

Quote L6 (source: crates/singlefs-core/src/mount.rs, enum RollbackCandidateExclusion, member TargetVersionWithoutFileUnsupported): "That target version's tree table has 0 entries, meaning no file version has been published yet: by the first three rules it can be in the candidate set, but the rollback row has to rewrite the instance table, and there is no clause for where its placement is recorded on a version with no file version, since decision D16 settled item 9 only defines that an empty publish on a version with 0 tree table entries writes zero units; the first version does not support this."

Quote L7 (source: crates/singlefs-harness/src/model.rs, enum ModelRefusalReason, member RollbackTargetNotInRing): "The rollback target is not in the root ring (decision D23, settled item 14: the candidate set is the roots in the root ring)."

Quote L8 (source: crates/singlefs-harness/src/model.rs, enum ModelRefusalReason, member RollbackTargetBelowEffectiveFloor): "The rollback target's txg is below F effective (decision D16, settled item 1, table row rollback candidate set)."

Quote L9 (source: crates/singlefs-harness/src/model.rs, enum ModelRefusalReason, member RollbackTargetOnAbandonedTimeline): "The rollback target is on an abandoned timeline (decision D23, settled item 14: there is a row (i, Ti, Wi) with T greater than Ti)."

Quote L10 (source: crates/singlefs-harness/src/model.rs, enum ModelRefusalReason, member RollbackToVersionWithoutFileUnsupported): "Rolling back to a root whose tree table has 0 entries: the rollback row has to rewrite the instance table, there is no clause for where its placement is recorded, the first version does not support this, per today's reading of the code."

Quote L11 (source: crates/singlefs-core/src/allocator.rs, enum PlacementRefusal, member NoFreeSlotOnAnyDevice): "There is no placement that satisfies policy on any device."

Quote L12 (source: crates/singlefs-core/src/allocator.rs, enum PlacementRefusal, member SomeDevicesFullDeviceSetSelectionUndefined): "Some devices have no placement that satisfies policy while other devices still do, meaning when devices are not equal in size the smaller device fills up first: the first version's device set is simply every device in the pool; warning, how to choose the currently writable device set once a smaller device is full is the undefined half of decision D2 settled item 2, there is no clause; the first version does not support this."

Quote L13 (source: crates/singlefs-core/src/allocator.rs, enum PlacementRefusal, member UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported): "The user-data placement each device works out from its own free-space map differs across devices: decision D3 settled item 8 requires each device to take its own placement, decision D2 settled item 10 requires one placement entry per replica, while the placement structure uses the same slot number on both devices and the publish path's placement entry carries only one slot number; the first version does not support this."

Quote L14 (source: crates/singlefs-core/src/allocator.rs, enum PlacementRefusal, member CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined): "After the open segment cannot hold it, the devices are given different destinations, meaning one device opens a new segment while another falls back, or they open different segments: whether the cluster segments on each device need to stay aligned is decision D3 settled item 8's open item number 1, there is no clause; the placement structure uses the same slot number on both devices; the first version does not support this."

Quote L15 (source: crates/singlefs-harness/src/model.rs, enum ModelRefusalReason, member UnitAreaWall): "The unit area cannot hold it (the admission check in decision D28 settled item 1; the model answers with an interval in which a refusal is permitted)."

Note N1 (background, not a quote to be mapped itself): some of the eight definitions in tables U and W below may not correspond to any of the listed candidate reasons at all. If none of the candidates listed for a table match a given row, write no match instead of forcing the closest one.

Section 5 continued: table U (rollback candidate exclusion definitions, source crates/singlefs-core/src/mount.rs, enum RollbackCandidateExclusion) and its candidate list (source crates/singlefs-harness/src/model.rs, enum ModelRefusalReason)

Table U, rows to map
row A | definition: Quote L3
row B | definition: Quote L4
row C | definition: Quote L5
row D | definition: Quote L6

Table U, candidate reasons (the order here is deliberately not the same order as rows A through D above)
candidate 1 | name: RollbackTargetOnAbandonedTimeline | definition: Quote L9
candidate 2 | name: RollbackTargetNotInRing | definition: Quote L7
candidate 3 | name: RollbackToVersionWithoutFileUnsupported | definition: Quote L10
candidate 4 | name: RollbackTargetBelowEffectiveFloor | definition: Quote L8

Table W (placement refusal definitions, source crates/singlefs-core/src/allocator.rs, enum PlacementRefusal) and its candidate (source crates/singlefs-harness/src/model.rs, enum ModelRefusalReason)

Table W, rows to map
row E | definition: Quote L11
row F | definition: Quote L12
row G | definition: Quote L13
row H | definition: Quote L14

Table W, candidate reason
candidate 5 | name: UnitAreaWall | definition: Quote L15

Section 6: questions about tables U and W (topic two). For each row A through D, and separately for each row E through H, answer these two things: (g) which one candidate (by number) this row's definition maps to, or state no match if none of the listed candidates for that table fit; (h) name the specific words or phrases in the row's definition and in the candidate's definition that led you to that answer, or, if you answered no match, name the specific words in the row's definition that have no counterpart in any candidate's definition.

Question 1: answer (a) through (f) for row T1.
Question 2: answer (a) through (f) for row T2.
Question 3: answer (a) through (f) for row T3.
Question 4: answer (a) through (f) for row T4.
Question 5: answer (a) through (f) for row T5.
Question 6: answer (a) through (f) for row T6.
Question 7: answer (a) through (f) for row T7.
Question 8: answer (a) through (f) for row T8.
Question 9: answer (g) and (h) for row A of table U.
Question 10: answer (g) and (h) for row B of table U.
Question 11: answer (g) and (h) for row C of table U.
Question 12: answer (g) and (h) for row D of table U.
Question 13: answer (g) and (h) for row E of table W.
Question 14: answer (g) and (h) for row F of table W.
Question 15: answer (g) and (h) for row G of table W.
Question 16: answer (g) and (h) for row H of table W.

End of prompt.
