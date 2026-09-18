DEFENSE ROUND. You are the defense side in an adversarial code-review exercise for
singlefs, a from-scratch copy-on-write filesystem written in Rust. Five fixes were
just landed in the implementation after an earlier attack round found problems in
an administrator-rollback and space-reclamation code path. Your job is to defend
each of the five fixes.

Do not use any markdown emphasis (no asterisks, no bold, no italics, no headers
with # symbols). Answer in plain numbered prose. Do not use the Rust path
separator two-colons in your answer; refer to functions and types by their plain
name only (for example say "the isolate function", not a qualified path).

For each of the five items below, do exactly three things, in this order, and
label them exactly as shown:

ITEM N OBJECTION:
First write out, in your own words, the strongest possible version of the
attack question given for that item. Elaborate it. Make it as sharp and
compelling as you can. You may add any additional angle you can think of
beyond what is quoted, as long as it is a genuine weakness and not a strawman.

ITEM N DEFENSE:
Then answer: does the fix hold up against this objection? If yes, give the
specific mechanism (which function, which check, which decided rule) that
resolves it, precisely enough that someone could verify your claim by reading
that function. If it does not hold up, concede clearly and explain exactly why
not.

ITEM N REFUTING OBSERVATION:
State one concrete, specific scenario, sequence of events, or state of the
system that would, if actually observed, refute your defense (that is, prove
the objection right). Be specific: name which roots, which slots, which
records, which order of crashes would have to happen.

BACKGROUND FACTS (use these; you cannot read any other file)

Root ring: a fixed-size ring of on-disk slots (24 slots in the first version:
3 regions of 8 slots each), each slot holding a root record with fields
including: instance (an instance generation number), checkpoint_txg (called
txg, a monotonically increasing publish counter), rollback_floor (called F,
the rollback floor), and pointers to an instance-table unit and a tree-table
unit.

Journal ring: a separate fixed-size ring of on-disk slots holding journal
records. Each journal record is identified by a monotonically increasing
counter called jsn (journal sequence number), unique pool-wide across every
instance that has ever existed. The journal ring is finite, so a jsn
eventually wraps around and reuses a ring slot that some earlier jsn once
occupied.

Instance table: an on-disk table of rows, one row per instance that has
ended (by ordinary recovery, an instance switch, or an administrator
rollback). Each row is (instance, T, W, is_rollback): T is the
checkpoint_txg of the root that was selected when that instance ended; W is
the highest transaction number that instance's records were applied up to;
is_rollback marks a row written specifically because of a rollback.

Administrator rollback: an operator picks an old root, called R_old, from a
rollback candidate set, and mounts onto it, abandoning every root published
after R_old whose instance is not still covered by a later valid instance.
The decided design rule (source: .claude/kb/decisions/23-journal的角色与格式.md,
line 1206, and .claude/kb/decisions/16-发布语义.md, line 358) defines the
candidate set as: roots in the ring that are judged still valid by the
instance table pointed to by the currently newest root, where a root with
(instance i, txg T) is selectable if and only if the table has no row for
instance i, or it has a row (i, Ti, Wi) with T less than or equal to Ti; and
in addition the root's txg must be greater than or equal to F_effective.

F (rollback floor) and F_effective: F is a field on every root record; it is
only ever raised, never lowered, and only when the pool needs to reclaim
released space. F_effective is defined as: for each device, take the maximum
F carried by any currently readable root on that device; then take the
minimum of those per-device maxima across all devices. This handles the case
where only some devices have durably received a root carrying the newer,
higher F value yet.

Shadow ledger: an in-memory-only bookkeeping mechanism, not persisted to
disk. On every mount, whether an ordinary writable mount or an
administrator-rollback mount, the code determines which roots currently
readable in the ring are abandoned. For an ordinary mount, a root is
abandoned if the newest root's own instance table has a row for that root's
instance with the root's own checkpoint_txg strictly greater than that row's
T. For a rollback mount, any additional root whose (checkpoint_txg,
instance) pair is greater than the rollback target's (checkpoint_txg,
instance) pair also counts as abandoned. For every abandoned root, the code
reads that root's own allocation-record tree (by following that root's tree
table entry to the allocation-record tree, then decoding every entry stored
directly in that tree's root node) and, for every entry there that is not
marked released in that abandoned root's own ledger, marks the
corresponding on-device slot as isolated. Isolated is a separate in-memory
flag, independent of the ordinary allocated and free flags. Once a slot is
marked isolated, the live allocator will never hand it out, even after that
same slot later becomes released and reclaimed through the ordinary
allocation-record lifecycle in the live ledger. Nothing in the code ever
clears an isolated flag once set (isolation only accumulates in memory for
the life of the mounted process); the decided rule (same source, line 1206)
says isolation should stop applying to a given abandoned root's slots once
that abandoned root itself is overwritten by ring rotation and leaves the
ring, at which point recomputing the shadow ledger on the next mount would
naturally no longer see that root as present to isolate against.

The decided rule's exact words for which slots to isolate (source, same
line): "before roots on an abandoned timeline leave the root ring, the units
they reference must not be reallocated nor have their headers erased; the
allocator additionally checks the ledger of every readable root in the ring,
and isolates only the slots referenced solely by abandoned roots." A
follow-up user decision on 2026-09-16 (same source) picked what is called
the narrow reading of "referenced solely by abandoned roots": read by the
grammatical subject "roots on the abandoned timeline", a slot still
referenced by any currently valid root does not count and should not be
isolated. The implementation actually landed in this round instead isolates
every slot in an abandoned root's own ledger that is not marked released
there, including slots also referenced by R_old (the rollback target) or by
any other currently valid root. This wider behavior is called the
conservative reading in the implementation's own comments, and it is
explicitly flagged there, in the milestone document, as a known literal
mismatch against the narrow-reading wording the user decided on 2026-09-16,
pending a separate decision to confirm or correct it. You should treat this
as an honest, already-disclosed fact, not something you need to discover.

ITEM 1. Choosing P2 for where the new record chain continues after a rollback.

Mechanism: when an administrator rolls back to R_old, the new instance's
first journal record needs a jsn to continue from. Two candidates were
considered. P1: continue from (R_old's own record's jsn + 1). P2: continue
from (the maximum jsn found anywhere in the whole ring, across every
instance, whether abandoned or not, plus 1). The landed fix takes P2 (source:
crates/singlefs-core/src/mount.rs, line 658, comment: "the new instance's
first jsn continues after the largest jsn in the ring"). Under P2, journal
records belonging to publishes made by the since-abandoned instance timeline
(after R_old but before the rollback) are left completely untouched on disk,
in the ring, forever (until the ring eventually wraps around and some later,
unrelated write happens to reuse that same ring slot). Those abandoned
records are kept from ever being replayed only by two other mechanisms: (a)
the rollback candidate set rule above, which never lets you select a root on
an abandoned timeline, and (b) the ordinary prefix-replay rule, which stops
scanning forward the moment a record's instance number differs from the
instance of whichever root is currently mounted.

Why P2 was chosen over P1 (already established in round 1 of this same
review, and not to be re-litigated as a separate concern; you may still use
it as part of your objection but the underlying crash scenario itself is
settled): with P1, if some root B, belonging to the instance being
abandoned, had already had its own journal record committed to the ring, but
its root slot had not yet become durable, and an administrator then rolled
back to R_old whose next jsn under P1 would be R_old's own jsn plus 1 -- that
new jsn could land on the exact same ring slot that B's already-committed
record occupies, silently overwriting it. If the system then crashed before
the rollback's own new root became durable, the very next recovery would no
longer behave "exactly as if rollback had never been initiated" (the decided
rule's literal requirement, source line 1206), because B's record slot in
the ring would already have been clobbered. P2 avoids this specific
collision by construction, since it never reuses any jsn that has ever been
used by anyone in the ring.

There is also an unresolved format-level question here, already tracked as
an open decision item and explicitly not yet ratified by the user: the
decided rule (source line 1206, and also line 1233 of the same file for the
analogous instance-switch case) says the new instance's records should
continue "from the end of the prefix, plus 1", but does not say, for the
specific case of a rollback, whose prefix that phrase refers to: R_old's own
prefix, or the whole ring's prefix. This is tracked as an open item (its
short label is: "which record the chain continues after, following a
rollback, has no definition"). Do not treat this open item as settled; you
may cite it, but do not claim the wording already resolves in favor of P2.

There is also a separately observed inconsistency in the code's own comments
that you may use: the doc comment directly above the mount_rollback function
itself (source: crates/singlefs-core/src/mount.rs, line 597) still literally
says the new instance's first jsn continues after R_old's own record, taking
P1 -- the opposite of what the function's body actually does at line 658,
which takes P2. This is a stale, un-updated top-of-function comment; it does
not change what the code executes, but it is a real, observable
inconsistency in the source as it exists today.

Attack question (verbatim, source: research/prompts/_m2-step45-code-r2-body.md,
line 35): "Abandoned records are left in the ring: when the chosen root
falls back to instance 1's old root because B's root, C's root, and instance
3's root are all unreadable, B's record gets picked up and the rollback is
undone. Is this the same family of problem as a previously tracked issue
about rollback being undone when both of a rollback instance's roots are
unreadable, or is it a new problem? When the chosen root is (1, 4), will the
abandoned instance 2's record be picked up, given that under P2 its own
record is still present in the ring? Once the ring wraps around, who is
accounting for the moment an abandoned record's slot gets overwritten? Does
jsn equal pool-wide maximum plus 1 literally match the decided rule's phrase
continues writing from end of prefix plus 1?"

ITEM 2. Computing the shadow ledger on every mount, using the conservative
reading.

Mechanism: this round's fix moved shadow-ledger computation out of the
rollback-only code path and into the shared rebuilt_allocator function
(source: crates/singlefs-core/src/mount.rs, lines 186 to 252), which now
runs on every mount, ordinary writable mounts included, not only rollback
mounts. On an ordinary mount, the set of abandoned roots is computed purely
from the newest root's own instance table (a root counts as abandoned if
that table has a row for its instance with the root's own txg greater than
that row's T). On a rollback mount, the newly-abandoned roots (those with
(txg, instance) greater than the rollback target's) are added to that same
set. For every root in that set, whether newly abandoned by this specific
rollback or already-abandoned from some earlier event, the function reads
that root's own allocation-record tree and isolates every entry there not
marked released, as described in the background facts above (the
conservative reading, isolating even slots also referenced by R_old or any
other still-valid root). Before this round's fix, the shadow ledger was only
ever computed once, at the moment of the rollback mount itself, and an
ordinary remount afterward would rebuild the allocator from the on-disk
allocation-record tree with no isolation logic running at all, silently
losing every isolation that had been in memory.

Attack question (verbatim, source: research/prompts/_m2-step45-code-r2-body.md,
line 36): "If the newest root's instance table cannot be read, then
newest_table becomes empty, so no root gets isolated by the table-based
check at all, only newly-abandoned roots from a rollback still get isolated.
Does this square with the decided rule's main clause? If an abandoned root's
tree table or allocation-record tree cannot be read, the rebuilt_allocator
function errors out entirely and the mount cannot open at all. Under the
conservative reading, a slot also referenced by R_old gets isolated, and
even after it is released and reclaimed it can never be handed out again,
until the abandoned root leaves the ring; the first version's 24-slot ring
never actually gets that far in this milestone's fixed test script, so how
much does the free count kept in the ledger diverge from what a df-style
report would say, given that the ninth accounting item in the decided design
for mount-time commitments only ever reports the exclusive amount? Is it
correct to define referenced by skipping ledger entries marked
is_released -- that is, a slot that is released in the abandoned root's own
ledger, but that the abandoned root's own predecessor version still
references?"

ITEM 3. Using F_effective, not the newest root's own F field, for the
rollback candidate-set check.

Mechanism: mount_rollback (source: crates/singlefs-core/src/mount.rs, line
623) checks that the rollback target's txg is greater than or equal to
F_effective (computed as described in the background facts) before allowing
that root into the candidate set. Before this round's fix, that same check
used the newest root's own rollback_floor field directly, which round one's
forward-verification leg found to be a narrowing inconsistent with the
decided rule's literal wording (source: .claude/kb/decisions/16-发布语义.md,
line 358, row labeled rollback candidate set: valid per the instance table
and txg greater than or equal to F_effective). The same decided source
separately defines effective (the row labeled effective) as: it only takes
effect once every surviving device carries a durable root with the new F;
the effective value after recovery equals the minimum, over surviving
devices, of the maximum F carried by that device.

Attack question (verbatim, source: research/prompts/_m2-step45-code-r2-body.md,
line 37): "After raising F has taken effect, reclaiming has happened, and
reuse has happened, suppose the root carried on one device becomes corrupt;
F_effective then falls back down; older roots become candidates again, even
though the units they reference have already been legitimately reused. What
happens if you roll back to such a root -- does the rebuild_version function
error out when it reads a unit that has since been overwritten, or does a
walk partway through panic instead? The candidate set uses F_effective,
while the checker's own candidate set uses the newest root's own F field;
during whatever interval the two disagree, what do the I-2.1 and I-3.1
invariants judge?"

ITEM 4. Reclaim threshold equals the maximum of F_effective and the oldest
valid root in the ring.

Mechanism: reclaim_floor (source: crates/singlefs-core/src/mount.rs, line
169, doc comment at line 167) takes the maximum of effective_floor
(F_effective) and an optional oldest_valid_root value, defaulting to
effective_floor alone if there is no oldest valid root available. This
function is called from two different places with two different notions of
"oldest valid root" computed by two different pieces of code. In the
rebuilt_allocator function (used on every mount), oldest_valid_root is
computed by taking every currently readable root, filtering out anything
judged abandoned by the newest root's own instance table (or, on a
rollback, also anything newly abandoned by that rollback), and taking the
minimum checkpoint_txg among what remains (source: same file, lines 221 to
233). In the raise_rollback_floor function (used when deliberately raising
F, normally triggered by low free space, here exposed as a test-only forced
entry point), oldest_valid_root is instead computed using the instance table
belonging to current, meaning the in-memory TransactionOutput of the process
that is actively in the middle of raising F right now, not the on-disk
newest root's table (source: same file, lines 357 to 366, using the
abandoned_by_table function with the table parsed from current's own
instance-table unit at line 333).

Attack question (verbatim, source: research/prompts/_m2-step45-code-r2-body.md,
line 38): "The oldest_valid_root value is judged, in the raise-F code path,
against the instance table belonging to current, but in the rebuild code
path, against the newest root's own table -- can the two locations compute
different oldest roots? The reclaim_floor function taking the maximum
degenerates into simply F_effective whenever F_effective is already greater
than the oldest root; once the generation-0 root has been overwritten by
ring rotation, meaning once txg reaches 25 or beyond in the first version's
24-slot ring, which of the two terms in that maximum arrives there first?"

ITEM 5. Recognizing the mkfs-era generation-0 tree table unit by allocation
generation 0, span 1, unreleased.

Mechanism: rebuild_from_records (source: crates/singlefs-core/src/allocator.rs,
line 358, doc comment starting at line 353) rebuilds the in-memory allocator
bitmaps from a previous version's on-disk allocation-record list. After
replaying every record onto the bitmaps, it separately scans that same
record list for one whose generation field equals checkpoint_txg 0, whose
span_slots field equals 1, and whose is_released flag is false; if found, it
remembers that specific placement as format_time_tree_table (source: same
file, line 376 comment, lines 378 to 389 code). This exists specifically to
close a gap: mkfs writes exactly two allocation records at generation 0 (one
for the instance table, spanning 2 slots, one for the tree table, spanning 1
slot); if mkfs and the very first writable mount that eventually supersedes
that tree table happen to run in two separate operating-system processes
(rather than one continuous process, as every currently existing test
exercises), the freshly rebuilt in-memory allocator would otherwise have no
memory at all of that specific generation-0, span-1 unit's role, and would
never release it when the first file version writes a new tree table over
it.

Attack question (verbatim, source: research/prompts/_m2-step45-code-r2-body.md,
line 39): "Recognizing the generation-0 tree table by allocation generation
0, span 1, unreleased: mkfs writes only two units total, the instance table
spanning 2 slots and the tree table spanning 1 slot -- are there any other
generation-0, span-1 records anywhere in the very first version of the file
system's format? After rebuild, does the first file version actually take
the publish_version with previous set to none code path? Today that path is
unreachable, because a pool that has never published any file cannot be
mounted writable at all in the first place."

CLOSING INSTRUCTIONS

Answer items 1 through 5 in order, using the exact three labels given at the
top (ITEM N OBJECTION, ITEM N DEFENSE, ITEM N REFUTING OBSERVATION) for each
item. Do not summarize all five items at the end. Do not add a general
conclusion beyond the five items. If you genuinely cannot construct a
convincing version of the objection for some item, say so plainly instead of
inventing a weak one, but try hard first: these are real, previously found
weaknesses in an earlier round of this same review, not hypothetical
concerns.
