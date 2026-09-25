Translation audit: m2-final-code-r4-local-attack (2026-09-25 UTC)

Line-by-line audit of every Fact and Item in
research/prompts/m2-final-code-r4-local-attack.md against the source it is translated or
paraphrased from. Source line numbers are current lines in the frozen copies named in the
dispatch: kb decisions in
/tmp/claude-1000/m2-final-code-r4/kb-snapshot/.claude/kb/decisions/, source code in
/tmp/claude-1000/m2-final-code-r4/tree/crates/. Every line number below was read with the Read
tool against these frozen copies in this session (not counted from the background material).
Numbers explicitly withheld from the prompt (the report's answers 5, 6, minus 3 and "the 11th
overwrite") are marked "withheld" below wherever the source line contains one of them.

Fact 1 (admission formula and per-device conjunction).
Source: decisions/28-挂载期承诺量.md:18 (the formula itself, verbatim except substituting
English term names for the nine Chinese term names, in the same left-to-right order) and :24
("逐设备算，每块盘各自满足...一块盘不够就拒，不拿别的盘的富余补" -> "admitted only if
available(d) is at least demand(d) on every device d... a surplus on one device never offsets a
shortfall on another device") and :24 again for the six-per-device/three-pool-wide split ("前
六项...取这块盘自己的值...后三项...是全池的承诺量...按全部副本之和记字节...摊到每块盘各扣
副本之和除以副本数").
First draft omitted: the parenthetical "逐设备合取" (per-device conjunction) name for the rule;
added back as "(a per-device conjunction)".
Final wording adds nothing beyond the source; "In every case discussed in this task, that
division has no remainder" is listed separately below as an addition (see Fact 5).

Fact 2 (mount_time_commitment is instance_switch_reserve only).
Source: singlefs-core/src/admission.rs:231-232 (field doc: "挂载期承诺量按设备摊的成员：实例
切换的预留...它的池级成员 checkpoint 保留池在式子里单列，这里不重复扣") and
decisions/28-挂载期承诺量.md:25 (naming the two members and that checkpoint reserve pool is
listed separately as the pool-formula's eighth item).
First draft conflated the two members into one term; corrected after reading admission.rs:231-232
directly, which states explicitly that checkpoint_reserve_pool is not double-counted here.

Fact 3 (zero-valued terms).
Source: admission.rs:138 (UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED, doc at :136-137: "第一版不
跑 zoned ⇒ 恒 0") and :140-148 (PENDING_DELETE_OF_THE_FIRST_VERSION,
COMMITTED_RESERVATION_OF_THE_FIRST_VERSION, docs at :140-141 and :145-146: "第一版没有删除路径
...⇒ 恒 0" / "第一版没有删除路径，墓碑的预付...都不发生 ⇒ 恒 0").
No omission: the three constants and their stated preconditions are carried over unabridged.

Fact 4 (abandoned_root_exclusive and no-rollback implies zero).
Source: decisions/28-挂载期承诺量.md:27 (definition: "只被被抛弃根引用的槽数 = 被抛弃根引用的
槽 − 回退候选集里的根引用的槽（下界 0）") combined with an independent reading of
singlefs-harness/tests/second_transaction_supplement_two_admission_formula.rs:394-400 (the
HistoryOperation list used to build the two small-disk images, which contains only
CloseAndMountWritable and PublishOverwrite, no rollback operation).
Added beyond the literal kb clause: the sentence "In a pool history that never performs an
administrator rollback, no root is ever abandoned, so this term is 0" is my own inference from
combining the definition with the test's operation list, not a sentence written verbatim
anywhere in either source; it is flagged in the "additions" section below.

Fact 5 (checkpoint_reserve_pool per-device share equals ckpt_cost slots exactly).
Source: admission.rs:160-172 (checkpoint_reserve_pool function and its doc: "ckpt_cost 个元数据
块...按全部副本之和记 ckpt_cost × 16384 × 副本数") and :86-91
(BytesSummedOverAllReplicas::share_of_one_device: "副本之和 ÷ 副本数...总除得尽").
Added beyond the literal source: the explicit algebraic conclusion "each device's share... is
exactly ckpt_cost times 16384 bytes, regardless of how many replicas" is my own arithmetic
carried out on the two cited functions (multiply by replicas, then divide by the same replicas);
neither function's own doc comment states this cancellation as a single sentence. Flagged below.

Fact 6 (ckpt_cost Sigma list).
Source: decisions/28-挂载期承诺量.md:99-104 (already-adopted clause 4: the Sigma-list wording,
withheld nothing since this clause states the formula, not a scenario's number) and
admission.rs:476-508 (checkpoint_cost_of_the_version_to_build_on function and its doc).
No numeric answer lives in this clause; nothing withheld.

Fact 7 (instance_switch_reserve formula).
Source: admission.rs:150-158 (INSTANCE_SWITCHES_ALLOWED_PER_MOUNT=3,
WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH=3,
INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE=INSTANCE_TABLE_PAGE_RECORDS-1) and :174-213
(instance_switch_reserve_on_one_device function body, translated step for step: pages_of_chain,
chain_rewrite, warm_up, one_switch, final multiply by N_switch+1) and
singlefs-format/src/lib.rs:117 (INSTANCE_TABLE_PAGE_RECORDS=370).
First draft used the constant name INSTANCE_TABLE_PAGE_RECORDS instead of its value 370 with the
subtraction spelled out; changed to spell out "369 = 370 - 1" plus the one-sentence reason from
admission.rs:156-158's own comment ("一片装 370 条，最后一条恒为链指针记录") so the model does
not need to intuit why 1 is subtracted.

Fact 8 (span and category of every unit).
Source: singlefs-core/src/transaction.rs:1838-1846 (span_slots function: UserData=2,
CommitGenerated TwoSlotsAligned=2, CommitGenerated OneSlot=1) and :1973-1993 (placement function,
naming exactly which TransactionUnit variants fall into which PlacementRule) and admission.rs:
510-546 (SpaceBudgetOfARole enum and space_budget_of_role function, naming exactly which
variants are Demand, CheckpointReservePool or InstanceSwitchReserve).
First draft listed the roles in the source code's declaration order; reordered into the three
category paragraphs used in the prompt for readability, with no role added or dropped (checked by
counting: transaction.rs's TransactionUnit has the same set of roles space_budget_of_role
matches on, all accounted for across the three paragraphs).

Fact 9 (SLOT_BYTES).
Source: singlefs-format/src/lib.rs:12 (SLOT_BYTES=16384) and :15
(NODE_BYTES=16384, same value, not separately named in the prompt since the two constants are
not both needed here). No omission of a qualifier that matters to this task.

Fact 10 (allocation-record tree geometry).
Source: singlefs-core/src/allocation_record_tree.rs:86 (doc: "从每块盘的绝对槽数（设备字节数 ÷
16384）算") and :91-113 (of_devices function body: the find-loop over candidate_root_level,
child_span, cells<=169) and :159-163 (height=root_level+1) and singlefs-format/src/lib.rs:138
(ALLOCATION_RECORD_TREE_LEAF_SLOTS=812), :144 (ALLOCATION_RECORD_TREE_INTERNAL_FANOUT=169), :285
(UNIT_AREA_START_SLOT=50176).
First draft omitted that of_allocator (allocation_record_tree.rs:117-129) feeds
UNIT_AREA_START_SLOT + unit_area_slots() into of_devices, i.e. that the "absolute slots" figure
used by this geometry is the whole device's slot count, not just the unit-area slot count; added
back explicitly ("a device configured with N unit-area slots therefore has 50176 + N absolute
slots in total") because omitting it would let the model wrongly compute the small-disk tree
heights from 256 or 384 alone instead of from 50176+256 or 50176+384.

Fact 11 (mapping/accounting tree height reading rule).
Source: transaction.rs:2259-2273 (height_read_from_the_root_node_header: "根节点头里的层级 + 1")
and admission.rs:14-16 (module doc: "记账树按每发布的节点数" is a node count, not this height
rule). No number withheld; this is a general rule.

Fact 12 (mapping/accounting leaf capacities).
Source: singlefs-format/src/lib.rs:72 (CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES=169), :93
(MAPPING_ENTRY_BYTES=55), :69 (ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES=159), :128
(ACCOUNTING_ENTRY_BYTES=34).
Added beyond the literal constants: the two floor-division results 294 and 477 are my own
arithmetic on the four cited constants (not written as a single number anywhere in the source);
flagged below.

Fact 13 (first-transaction tree shapes on a 4 GiB two-device pool).
Source: singlefs-harness/tests/first_transaction_step_five_publish.rs:602-604 (comment: "4 GiB
两块盘上分配记录树根在第 2 层...extent 树根兼叶在第 0 层...只有 inode 树根是层级 1") and :1009
((allocation.level, allocation.entries.len(), allocation.entry_width, allocation.key_width) ==
(2, 2, 96, 10), giving level=2) and :899 ("映射根兼叶") and :908 ((mapping.tree_identifier,
mapping.entries.len(), mapping.key_width, mapping.entry_width) == (15, 10, 27, 55), giving
entries.len()=10) and :643-650 (match arm: AccountingTree level=0, and the unreachable! arm
confirming "第一个事务的记账树与中央映射树都是根兼叶") and :1085
(comment: "记账树 15 行") and singlefs-format/src/lib.rs:164
(FIRST_TRANSACTION_ACCOUNTING_ROWS=15).
No number withheld: level=2, level=0 (twice), entries.len()=10 and rows=15 are none of the four
withheld numbers (5, 6, minus 3, 11); they describe a different pool (4 GiB, not the small disks)
and a different quantity (tree shape at the first transaction, not an admission verdict).

Fact 14 (three small-disk unit-area sizes).
Source: singlefs-harness/src/history.rs:90 (SMALL_DEVICE_UNIT_AREA_SLOTS=384), :93
(SMALLER_DEVICE_UNIT_AREA_SLOTS=256), :96 (NARROWEST_DEVICE_UNIT_AREA_SLOTS=240), with the three
HistoryDeviceWidth variant names at :78, :82, :86.
No omission.

Fact 15 (384-slot growth comment, approximate).
Source: singlefs-harness/src/history.rs:74 ("第一个文件之后每盘占 13 槽、一次覆盖写再占 10 槽，
回收之前几十次发布就写满"). Translated close to verbatim ("about 13 slots" / "roughly 10 more
slots" / "for dozens of publishes before reclamation catches up"); the word "约" is not present
in this exact clause of the source comment (it appears in the neighbouring Fact 16 comment,
:80-81), so "marked as approximate" in the prompt is my own added qualifier here, carried over
from the neighbouring sentence's explicit "约" for consistency and because 13 and 10 are
themselves not exact per-overwrite constants tested anywhere (they are the comment author's own
estimate, stated without a supporting assertion in this file). Flagged below as an addition.
Withheld: the rest of this same source line ("回收之前几十次发布就写满，分配器的落点拒绝...在一
段历史里走得到") and the whole of the neighbouring second sentence about a raw allocator wall are
carried into Fact 16 instead, and the two clauses further along in history.rs describing the
384-slot mount test's own docstring (second_transaction_supplement_two_admission_formula.rs:
457-462, "覆盖写 8 次的盘面上式子放行...10 次是第一个被拒的点") are deliberately not quoted or
paraphrased anywhere in this prompt, because that sentence states the exact overwrite count at
which the admission formula first refuses a mount on this configuration, which is the same fact
as the report's withheld "11th overwrite" conclusion one arithmetic step removed (10 succeed, so
the next, the 11th, is refused). Withheld in full.

Fact 16 (256/384-slot steady-state comment, approximate).
Source: singlefs-harness/src/history.rs:79-81 ("稳态占用约是根环里那 24 版的账（每版每盘约 10
槽），384 槽那一档一次落点拒绝都走不到了；256 槽走得到用户数据那一处的拒绝").
Translated close to verbatim, keeping "约" (approximately) on both the generation count and the
per-generation figure, and keeping the qualitative claim about which of the two configurations
reaches the raw allocator's wall.

Fact 17 (replicas=2, history contains only mount/overwrite steps, no rollback).
Source: singlefs-harness/tests/second_transaction_supplement_two_admission_formula.rs:394-400
(GeneratedHistory operations list: one CloseAndMountWritable followed by repeat_n(overwrite,
overwrites), nothing else) combined with admission.rs:94-116 (ReplicaCount::of_every_device_in_
the_pool: "分配器里一块盘都没有：池至少有一块盘...池里每一块盘"—replicas equals device count)
and the two devices explicitly constructed for these tests (HistoryDeviceWidth is a two-device
model throughout this harness; DISKS arrays of length 2 appear in the neighbouring
second_transaction_supplement_two_row_publish_admission.rs:31 as the same pattern).
No number withheld.

Fact 18 (256-slot setup).
Source: singlefs-harness/tests/second_transaction_supplement_two_admission_formula.rs:374-421
(small_pool_image_after_overwrites function: HistoryStartingPoint::AfterFirstFile,
CloseAndMountWritable, repeat_n(overwrite, overwrites), SpaceAdmission::SkippedByTheTestOnlySwitch)
and :513-538 (an_overwrite_whose_ordinary_allocations_exceed_the_available_bytes_is_refused_by_
the_space_admission_before_any_write: device_width=UnitAreaOf256Slots, overwrites=4, a fresh
mount_writable_with_space_admission with SpaceAdmission::JudgedByTheFormula, then one more
publish_overwrite consulted by the formula).
Withheld: this test's own assertion values (available=5 slots, demand=6 slots, both per the
dispatch's withheld list) at lines 561-583 of the same file are not quoted, paraphrased, or
referenced by number anywhere in Fact 18 or in item 1; only the setup steps that precede that
assertion are carried over.

Fact 19 (384-slot setup).
Source: same file, :457-467 as far as the two lines that construct the scenario
(device_width=UnitAreaOf384Slots, small_pool_image_after_overwrites(device_width, 10)) and
:472-511 (mount_writable_with_space_admission called with SpaceAdmission::JudgedByTheFormula,
demand computed as 0 for a plain mount, per instance_to_acquire and the surrounding assertions'
structure, not their content).
Withheld: this test's own docstring at :457-462 ("覆盖写 8 次的盘面上式子放行...10 次是第一个被
拒的点") and its assertion value (available = minus 3 slots, per the dispatch's withheld list) at
:485-497 are not quoted, paraphrased, or referenced anywhere in Fact 19 or in item 2; only the
literal overwrite count (10) and the mount step that follows are carried over, framed neutrally
("this new mount is admitted by the formula" for Fact 18, but deliberately left unresolved --
neither admitted nor refused -- for Fact 19, since revealing either verdict for the 384-slot case
would give away the same fact the report's minus-3 conclusion states).

Items 1-4 (worksheets and questions).
Source: the dispatch message's own four bullets (which item asks about which scenario) plus
Facts 1-19 above for every number and rule used inside each worksheet row. No withheld number
from the dispatch's list (5, 6, minus 3, 11) appears in any row label of any of the four
worksheets; each row asks for a term defined in Facts 1-17 or a setup fact from Fact 18/19, never
for the report's own conclusion.
Added beyond a literal restatement of the four dispatch bullets: the row-by-row worksheet
structure itself (13 rows for items 1 and 2, 6 rows for item 3, a function-of-N table for item 4)
is my own construction, built so that the model cannot answer with a single number; the
instruction that item 4's answer may be "off by a few" and must say so explicitly is also my own
addition, because Fact 15/16's growth figures are stated as approximate in their own source
comments, and asking the model for an exact integer without that caveat would misrepresent how
precise the underlying facts actually are.

Consolidated list of everything the English prompt states beyond a literal translation of its
cited source (each already flagged inline above; collected here in one place per the rule that
additions must be listed and justified, not just omissions):
1. Fact 4's closing sentence (abandoned_root_exclusive=0 given no rollback in these two
   histories) is an inference from combining a kb clause with a test's operation list.
2. Fact 5's closing sentence (per-device share equals ckpt_cost slots exactly, replica count
   cancels) is my own arithmetic on two cited functions.
3. Fact 7's spelled-out "369 = 370 - 1" plus its one-sentence reason.
4. Fact 12's two floor-division results, 294 and 477.
5. Fact 15's "marked as approximate" qualifier, carried over from Fact 16's explicit "约" for
   consistency, since 13 and 10 in this exact source line are not otherwise flagged as estimates
   in that line by itself.
6. Item 1-4's row-by-row worksheet structure and item 4's "off by a few" caveat.
None of these six additions introduces, restates, or narrows in on any of the four withheld
report numbers (5, 6, minus 3, 11); each is either a generic algebraic step on already-disclosed
constants, or an explicit uncertainty caveat that makes the model's task harder, not easier.

Cross-check against the dispatch's withheld list (5, 6, minus 3, 11): a literal-value grep of the
finished prompt file (excluding fact numbers, row numbers, and the unrelated constants 55, 169,
369, 384, 812 and so on that merely contain these digits as part of a longer number) was run
after the prompt was written; every isolated occurrence of 5, 6 or 11 found this way is a "Fact
N" or "Row N" cross-reference or a constant unrelated to the four withheld report answers (see
command and output below). No isolated "-3" or "minus 3" occurs anywhere in the prompt.

Command: grep -noE '(^|[^0-9])-?[0-9]+([^0-9]|$)' research/prompts/m2-final-code-r4-local-attack.md
Reviewed by hand line by line (see the exploration transcript of this session); every isolated 5,
6 and 11 traced back to a Fact/Row label, never to a computed admission-formula outcome.
