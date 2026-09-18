# Local defense round: six implementation choices in the second-transaction step-0/step-3 code (m2-step3-code-r1)

You are the defense side. For each of the six implementation choices below, first write the
strongest objection a hostile reviewer could raise against it, using the concrete facts given.
Then answer whether the choice still holds up given those facts, or whether it should be
recorded as an open decision point instead of accepted silently. Say plainly what observation
would prove your defense wrong. Do not use any markdown emphasis (no bold, no italics). Answer
by item number, in plain sentences.

None of these six choices are dictated word-for-word by any existing decision document. They
were made while writing the code for milestone "second transaction" step 0 and step 3
(reopening a pool, taking a new instance number, writing an instance-table row, warming up,
then publishing a third file version). Your job is to judge whether each one is a reasonable,
defensible reading of the decisions that do exist, or whether it silently goes beyond what any
decision says, in a way that could be wrong.

## Background facts you can rely on (quoted or paraphrased from the source files; file paths
## and line numbers are given so you do not need to guess)

Fact A. The commit protocol for one publish is: write every unit that this publish rewrites to
every device, one barrier, write the journal record to every device, one barrier, write the
root record with force-unit-access, rotate the superblock slot. There is only one barrier
after the whole batch of rewritten units; there is no barrier between individual unit writes
within that batch. If the process crashes in the middle of writing that batch, none of the
units in it become the published version, because the journal record and root that would make
them visible are only written after the barrier. Source: crates/singlefs-core/src/transaction.rs,
around line 1601 to 1614 (the loop over written_units, one Barrier call after it, then the
journal record, another Barrier, then the root, then the superblock rotation).

Fact B. The TransactionUnit type has a fixed array named IN_BUMP_ORDER of exactly eight roles: Data,
ExtentRoot, InodeLeaf, InodeRoot, AllocationTree, AccountingTree, MappingTree, TreeTable.
InstanceTable is a ninth variant of the TransactionUnit enum, and it is not a member of this
eight-item array. Source: crates/singlefs-core/src/transaction.rs, lines 387 to 411.

Fact C. When the code builds its internal lookup vector called "units" (used only to look up a
unit by its role afterwards, not written anywhere), it first loops over the eight IN_BUMP_ORDER
roles and pushes them, and only after that loop finishes does it push the InstanceTable unit
(if this publish rewrites it) at the end. Source: crates/singlefs-core/src/transaction.rs,
lines 1490 to 1533. This "units" vector's internal order has no effect on anything written to
disk; it is only used later with a linear search by role identity.

Fact D. The actual order in which units are written to disk for one publish is controlled by a
different list, called "rewritten", which comes from a method called rewritten_roles(). Its
logic, in the order written in the source, is: if this publish carries a new file, push Data
first; then, if this publish rewrites the instance table, push InstanceTable; then, if this
publish carries a new file, push ExtentRoot, InodeLeaf, InodeRoot; then always push
AllocationTree, AccountingTree, MappingTree, TreeTable, in that order. Source:
crates/singlefs-core/src/transaction.rs, lines 785 to 811. The actual per-device writes
(the WriteUnitToEveryDevice case of CommitStep) are issued by iterating this same "rewritten" order,
at crates/singlefs-core/src/transaction.rs lines 1555 to 1607.

Fact E. The specific publish that writes the instance-table row after reopening a pool
(mount.rs's mount_writable, "write the row" step) builds its PublishPlan with file set to
None and instance_table set to Rewrite. Given the logic in Fact D, for this specific publish,
rewritten_roles() returns exactly: InstanceTable, AllocationTree, AccountingTree, MappingTree,
TreeTable, in that order, because the file-is-Some branches are skipped (file is None) and the
instance-table branch fires. This means InstanceTable is the first unit written to disk in
this publish's batch, not the last, and not the ninth out of nine. Source:
crates/singlefs-core/src/mount.rs, lines 267 to 283, combined with Fact D.

Fact F. Sequence numbers (called birth_sequence) used inside node pointers are assigned by a
counter keyed by the triple (tree identifier, txg, instance), starting at 0 for each distinct
key, independently per key. Source: crates/singlefs-core/src/transaction.rs, lines 701 to 719
(struct BirthSequenceAllocator, using a BTreeMap keyed by that triple). Both TreeTable and
InstanceTable map to the same tree identifier, called TREE_IDENTIFIER_NONE (source:
crates/singlefs-core/src/transaction.rs, lines 453 to 470, the tree() method). This means that
for a given txg and instance, TreeTable and InstanceTable draw their birth_sequence numbers
from the very same counter. In the source code, the statement that assigns InstanceTable's
sequence number (line 1156, "sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg,
instance)") executes before the statement that assigns TreeTable's sequence number (later in
the same function, after allocation, accounting and mapping sequences are assigned). Because
the counter for that shared key starts at 0 and increments by 1 on every call, InstanceTable
receives the lower number (0) and TreeTable receives the higher number (1) whenever both are
rewritten together in the same publish. A comment directly above this code, in the source
file, reads (translated from Chinese): "the instance table unit ... belongs to tree 0, and is
assigned its sequence number before the tree table (mkfs also does instance table before tree
table)." This comment is about the order in which sequence numbers are handed out, not about
the order in which bytes are written to disk.

Fact G. Separately from all of the above, one paragraph in the project's milestone notes
(file .claude/kb/milestone/02-second-txn.md, matching text also appears in the round's
appendix file, around its line 1499) describes this same code by writing, in Chinese, that
among the choices made while implementing this step but not dictated by any decision, one is:
"instance table sequence is placed at the end of the bump sequence, as a ninth role"
(literal translation: "the instance table, at the end of the bump sequence, counts as the
ninth role"). The round's body document restates this as "instance table counts as the ninth
role, written after the eight bump roles."

Given facts B through G together: the claim "instance table is written after the eight bump
roles" is true only in the narrow, causally inert sense of Fact C (an internal lookup vector's
construction order, which is never serialized or observed by anything). It is false as a
description of the actual disk write order for the row-writing publish (Fact E: instance table
is written first among the five roles this publish rewrites), and it is also false as a
description of the birth_sequence numbering order for the shared TREE_IDENTIFIER_NONE counter
(Fact F: instance table gets the lower number, before tree table). Given Fact A, the internal
order among units written before the single barrier has no crash-consistency consequence
either way, because none of them become visible until the journal record and root are written
afterward; so this is a question of whether the written characterization is accurate, not of
whether the system is broken.

## Question 1 (item numbered 1)

Given facts A through G, does the claim "instance table counts as a ninth role, written after
the eight bump roles" hold up as an accurate description of what the code does? Write the
strongest objection to it first (you may use the three different orderings identified above:
internal lookup-vector order, disk write order, and sequence-number order, and point out they
disagree), then say which of the three orderings (if any) the claim is actually describing
correctly, and whether this mismatch matters for correctness given Fact A. State what
observation would prove your answer wrong.

## Item 2: warm-up publish count computed from device coverage, capped at 3, instead of the
## registered format constant 2

Fact H. Decision D16 (publish semantics), settled item 8, reads (translated): "before the new
instance's root, once durably written with force-unit-access, covers two devices, do not let
any fsync return and do not confirm a rollback to the administrator; do this by issuing empty
publishes back to back, at most R = 3 times in the first version's geometry." A warning
attached to the same settled item reads (translated): "the first version actually pays this
cost 2 times, and this is a format constant, not a runtime predicate (user decision on
2026-09-14): the root ring's three regions have their device assignment hard-coded as 0 / 1 / 0
in the first version ... so txg 1 lands in region 1 (device 1) and txg 2 lands in region 2
(device 0), and these two happen to cover both devices; so the count is uniquely determined by
this assignment. This has been registered as a format constant, WARM_UP_EMPTY_PUBLISHES = 2 ...
warning: if the assignment changes (for example when a device is added or striping is turned
on), these two constants must be recomputed; the phrase 'at most R = 3 times' remains a
geometric upper bound, not the count the first version actually reaches. This was not put
through three-way argument, and can be overturned. The lead reviewing agent's own
recommendation is also this constant-2 form."

Fact I. The milestone document (.claude/kb/milestone/02-second-txn.md, matching text in the
appendix around its line 1520) explicitly lists as a still-open decision point (translated):
"the warm-up count for later writable mounts and for mounts after a rollback (decision D16
settled item 8 only defines the first writable mount; flagged 2026-09-16 by the forward-
reasoning leg of the first three-way round)."

Fact J. The implemented code (crates/singlefs-core/src/mount.rs, lines 285 to 323) does not
use the constant 2 at all. Instead, it computes, for the row-writing publish's own root, which
device it lands on (using target_for_publish and the region-to-device assignment table), adds
that device to a "covered" list, and then loops issuing more empty publishes, each time
checking which device the new root lands on and adding it to "covered", stopping either when
"covered" includes every device in the pool or when the loop has run ROOT_RING_REGIONS (3)
times, whichever comes first. It never reads the constant 2 anywhere.

Given facts H, I and J: is computing warm-up coverage dynamically, capped at the geometric
upper bound of 3, and ignoring the registered format constant 2, a defensible choice, given
that the registered constant 2 is explicitly a first-mount-only, assignment-dependent number
that decision D16 itself warns is not safe to reuse elsewhere without recomputation, and given
that the milestone document explicitly says the later-mount warm-up count was left open by the
decision? Write the strongest objection first: consider whether "compute it dynamically" could
itself be wrong in some way the fixed constant would have avoided, for example whether the
loop could stop too early, or never terminate, or produce different behavior on different runs
in a way that breaks the crash-consistency guarantee the warm-up mechanism exists to provide.
Then answer whether the choice holds up. State what observation would prove your answer wrong.

## Item 3: after reopening, the allocator's open commit-generated segment is not carried over;
## the first allocation after reopening starts a fresh search on the free bitmap

Fact K. Decision D3 (space allocation), settled item 7, states that an allocation record entry
has only two possible states on disk: allocated, or released-with-a-release-generation-number
(translated: "allocation record entries have only two states: allocated, and released plus
release generation"). There is no disk representation of a third state such as "this slot is
inside the currently open commit-generated segment, before this segment's own tail". The
open segment and its bump cursor are, by the allocator's own design, purely in-memory state
that is never written to the allocation-record tree at all.

Fact L. The code comment in crates/singlefs-core/src/allocator.rs, directly above
rebuild_from_records (lines 285 to 288), reads (translated): "the open segment and bump
cursor exist only in memory; nothing on disk records them (the allocation-record tree does not
record them). After reopening, start from the position of 'no open segment': the next
commit-generated-block allocation opens the lowest fully empty segment. Slots left unused in
the previous segment are simply left behind for now (this is a separate open decision point
from the no-empty-segment fallback policy question, to be revisited at milestone step 3)."

Fact M. Because there is no disk field to record which segment was open or where the bump
cursor was, any attempt to "continue" the previous open segment after a restart would have to
either invent a place to store that information (a format change) or guess it by re-scanning
which segment has the most slots already used without being full (a heuristic that could pick
the wrong segment, or could misidentify a segment that is not actually the one this instance
had open, especially after an instance switch where a different instance's writes could have
also touched segments). No code today attempts this; the function rebuild_from_records on PoolAllocator
(crates/singlefs-core/src/allocator.rs, lines 289 to 309) marks every recorded slot allocated
and, for released ones, also moves them into the defer queue, and simply leaves open_segment as
None (this is the same starting state that constructing a fresh device map produces).

Given facts K, L and M: is "do not continue the open segment; restart the search from the free
bitmap" the correct default, or does it hide a real correctness or fragmentation-cost problem
that a decision should have addressed explicitly? Write the strongest objection first,
considering: could this cause the allocator to jump around the disk in a way that breaks any
invariant, not just cost extra fragmentation; and is "no format field exists for this, so we
cannot do otherwise without a format change" a sufficient justification, or is it dodging the
question of whether continuing across a restart was ever necessary for correctness (as opposed
to placement quality). Then answer whether the choice holds up. State what observation would
prove your answer wrong.

## Item 4: the first version has no clean-shutdown marker; every reopen unconditionally goes
## through the crash-recovery path

Fact N. The doc comment at the top of crates/singlefs-core/src/mount.rs (line 4) states,
translated: "the first version has no clean-shutdown marker; reopening always goes through
recovery." The same sentence appears, marked as a "prediction, not yet a decision, pending the
user" (the Chinese source word used there literally means "a prediction"), in the milestone document and in decision D18's settled item 11
discussion (both quoted in the appendix around lines 1507 and in the milestone excerpt around
line 1499).

Fact O. mount_writable (crates/singlefs-core/src/mount.rs, lines 169 to 338) always begins by
calling choose_superblock, choose_root, scan_journal and replay_journal, unconditionally, with
no branch that would skip straight to "the pool was closed cleanly, previous is exactly what
was last published, no need to replay anything." There is no on-disk field anywhere in the
superblock or root record structures read by this function that records "this pool was closed
cleanly."

Given facts N and O: is it defensible for the first version to have no clean-shutdown fast
path at all, always paying the cost of a full recovery scan (superblock choice, root choice,
full journal scan, replay) even for the overwhelmingly common case of a normal unmount? Write
the strongest objection first: consider whether skipping recovery in the marked-clean case
could ever be unsafe even if the marker itself were trustworthy (for example, because the
marker write and the last publish are not the same atomic operation, so the marker could lie),
and whether "we simply have not built the marker yet" is different from "a marker would be
unsafe here so we deliberately did not build one." Then answer whether the current absence of
any clean-shutdown path is defensible for a first version, or whether it is a functional gap
serious enough that it should already be logged as a numbered open decision point rather than
only appearing as a "prediction" note in two documents. State what observation would prove
your answer wrong.

## Item 5: when the chosen root's own journal record cannot be read, the recovery chain resumes
## from the smallest journal sequence number above the water mark for that instance

Fact P. Decision D23 (journal role and format), settled item 14, and invariant I-8.3 (strict
prefix contiguity) both state that the recovery chain resumes "right after the last record
covered by the chosen root" (translated), and that the sequence numbers (jsn) applied must be
exactly contiguous, with any break stopping replay immediately. Neither of these texts contains
any sentence describing what to do specifically when the record belonging to the chosen root
itself (same instance, same checkpoint txg as the root) cannot be read from disk (both mirrored
copies unreadable or fail their checksum).

Fact Q. The actual code (crates/singlefs-core/src/recovery.rs, function replay_journal, lines
599 to 683) works like this. First it builds a candidate list called "above": every scanned
journal record whose (instance, checkpoint_txg) pair is strictly greater than the chosen root's
own (instance, checkpoint_txg) pair, restricted to the same instance as the root, sorted
ascending by (instance, jsn counter). Separately, it searches the full set of scanned records
for the one whose (instance, checkpoint_txg) exactly equals the chosen root's own (instance,
checkpoint_txg); if found, it takes that record's jsn counter value and sets the expected next
jsn to be (root.instance, that counter + 1). If that record is not found (this is the "own
record unreadable" case), the expected-next value is set to None. Then the code loops over the
"above" list in ascending jsn order: on the very first iteration, if expected-next is None, the
"if let Some(expected_key) = expected_next" check is skipped entirely (no comparison happens),
so whichever record happens to be first in the sorted "above" list is accepted unconditionally,
its jsn becomes the new expected-next value, and strict contiguity checking begins from that
point onward.

Fact R. There already exists a test (crates/singlefs-harness/tests/second_transaction_step_
three_second_instance.rs, named one_missing_record_right_after_the_chosen_root_stops_the_
prefix_even_when_later_records_are_intact) which checks a narrower case: the chosen root's own
record IS readable, but the very next jsn is missing; the test asserts that a later jsn must
not be accepted in that case. This test does not cover the case where the chosen root's own
record itself is unreadable.

Given facts P, Q and R: does "accept whichever record has the smallest jsn above the water mark
for that instance, when the root's own record cannot be read" correctly implement "resume right
after the last record covered by the chosen root," or could it silently apply a record that
does not actually follow immediately after the root, or silently skip an intermediate record
that has itself been lost, producing a result that violates strict contiguity (I-8.3) without
any check noticing? Write the strongest objection first: try to construct, at least in outline,
a sequence of events (which records exist, which are readable, which get overwritten by ring
wraparound) under which this fallback would accept a record that should not have been treated
as contiguous with the root. Consider whether the single-writer, serial-commit nature of this
system (only one publish's record and root are ever in flight at a time) rules out the gap you
are trying to construct, or does not rule it out. Then answer whether the choice holds up, or
whether it should be flagged as an open decision point (a candidate observation to require:
that when the chosen root's own record cannot be read, the code should also verify there is no
gap between the smallest counter above water and the counter that would have been produced by
adding one commit to whatever number scheme back-chains claim, rather than trusting whichever
record sorts first). State what observation would prove your answer wrong.

## Item 6: a pool whose tree table is empty (no file version has ever been published) refuses
## a writable mount through this code path, returning NoPublishedVersion

Fact S. The MountError enum in crates/singlefs-core/src/mount.rs (lines 116 to 126) has a
variant NoPublishedVersion, documented (translated) as: "the chosen root does not yet have a
published file version underneath it (tree table is empty): the first version's writable mount
through this path only attaches to a pool that already has a file; a freshly-mkfs'd pool goes
through the first-writable-mount path instead." mount_writable returns this error whenever
rebuild_version reports an InvariantViolated failure with invariant name that literally means "mount" (translated
"mount") for an empty tree table (see the match arm at lines 196 to 209).

Fact T. The project's test harness (crates/singlefs-harness/tests/common/mod.rs, function
build_pool, around lines 154 to 220) does not call mount_writable at all for a freshly-mkfs'd
pool; it calls publish_first_file directly. mount_writable is only ever exercised, in the
current tests, on a pool that already has at least one published file version.

Fact U. No text found anywhere in the decision appendix for this round describes a
NoPublishedVersion-style restriction, nor does any decision explicitly require that the first
writable mount after mkfs and every subsequent writable mount share one code path. Decision
D18 settled item 11 does describe that a mount that finds no instance-table rows yet writes
none for instance 0 (mkfs), which is consistent with there being a genuinely different, earlier
bootstrapping stage, but it does not say in so many words that this bootstrapping stage must
live in a separate function from mount_writable.

Given facts S, T and U: is having two separate entry points, one (publish_first_file, reached
directly by callers, not through mount_writable) for the very first writable mount of a
freshly-mkfs'd pool, and another (mount_writable) for every later writable mount, a defensible
split, or does it risk the kind of silent divergence the project's own design rule warns
against (one transaction mechanism, not two, citing the rule against opening a second logging
or commit mechanism for a special-cased path)? Write the strongest objection first, focusing
specifically on whether the two paths could, over time, be changed independently and drift
apart in ways that produce different on-disk results for what is conceptually the same
operation (a instance acquiring write access and publishing its first thing). Then answer
whether today's split is defensible as an intentional, currently-narrow scope boundary (first
transaction was already built in an earlier milestone and works; step 3 only had to build the
reopening case) rather than an accidental duplication, and say what would have to be true for
it to count as a real duplicated mechanism rather than a staged rollout. State what observation
would prove your answer wrong.

## Additional question (item numbered 7): does the free-count independence finding, and the
## release-via-mapping finding, from the previous round still hold on the rebuild-from-disk
## path used by mount_writable

In the previous round (round 2 of the same code review), two specific findings were reached
and recorded, both concerning code in crates/singlefs-core/src/allocator.rs and
crates/singlefs-core/src/transaction.rs, exercised at that time only through same-process,
in-memory publishing (no restart involved):

Finding 1, "free slots counted independently": the field DeviceFreeMap.free_slots is
maintained by a separate increment/decrement path from DeviceFreeMap.allocated_slots, so that
the checker's invariant I-5.2 (free + allocated == capacity) is not a tautology; the previous
round's attacking leg showed that a single mutation deleting the line "self.free_slots -=
span;" inside the function mark_allocated could not be caught by any value-only assertion
(because a paired, compensating change to both free_slots and allocated_slots keeps the sum
identity true), and could only be caught by a mutation-table entry that specifically breaks
that exact line and requires a specific test (cold_start_reads_the_second_content_and_the_
pool_checker_stays_green, in second_transaction_step_one_overwrite.rs) to turn red.

Finding 2, "release goes through the mapping, not through position-entry hints": the previous
round's attacking leg found and the response fixed three missing checks in the release path
(placements_to_release_via_mapping, and the release function on PoolAllocator): looking up a mapping-given slot
that has no allocation record at all, looking up one that is already released, and looking up
one whose recorded span does not match; all three now return typed errors (ReleaseTarget
NotAllocated, ReleaseTargetAlreadyReleased, ReleaseSpanMismatch) before the allocator's records
or bitmap are touched, instead of panicking or corrupting state.

Now, this round's code adds a new path: after reopening a pool, mount_writable rebuilds its
in-memory allocator, not by continuing an existing in-memory allocator, but by calling
the function rebuild_from_records on PoolAllocator (crates/singlefs-core/src/allocator.rs, lines 289 to 309),
which loops over every allocation record read back from disk and, for each one, calls
device_map.mark_allocated(record.slot, span) - literally the same function that Finding 1's
mutation targets - and, if the record says released, also calls device_map.mark_released(...).
The reconstructed allocator's records field (used by the release-path checks from Finding 2,
via the record_for function on PoolAllocator) is populated directly from the same on-disk allocation records.

Separately, there is an already-registered, unresolved open item, quoted (translated) from the
milestone document: "reopening and rebuilding the previous version from disk: today the
release-decision path reads same-process in-memory state (TransactionOutput) and carries no
reader at all; when rebuilding 'previous' from disk, the mapping entries' position fields carry
a unit checksum, and before releasing, this should be checked against the actual on-disk unit
(flagged by the second round's attacking leg, referred to there as finding Y1's scope)."

There is also a test, second_transaction_step_three_second_instance.rs, lines around 254 to
314, that, after a full reopen-and-rebuild, a row-write publish, warm-up publishes, and a
publish of a third file version, asserts the rebuilt allocator's free_slots field against a
hardcoded absolute number (211_968 - 47), asserts the on-disk accounting statistics
(STATISTIC_ALLOCATED_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_FREE_BYTES) against the same
numbers, and then runs the full pool checker and requires invariants I-3.1, I-3.8, I-5.2, I-7.2
and I-7.7 to each have actually been evaluated and to hold.

Given all of this: does Finding 1 (free slots counted independently, guarded by the mutation
that breaks mark_allocated) still hold, unweakened, on this new rebuild-from-disk path, given
that rebuild_from_records calls the very same mark_allocated function rather than a separate
reimplementation? Does Finding 2 (the three release checks) still hold on this path, given that
rebuild_from_records populates the very same records field those checks read? And separately,
is the already-registered open item about checksum verification before release a real gap that
these two findings do not cover, or does it turn out, on closer inspection of the code, to
already be handled some other way? Write the strongest objection to "these findings still hold
unweakened" first: consider whether rebuild_from_records could have a bug specific to itself
(not inside mark_allocated or record_for) that neither finding's guard would catch, for example
in how it iterates records or matches them to devices. Then give your answer for both findings
and for the open item, item by item. State what observation would prove each answer wrong.
