SINGLEFS DESIGN QUESTION - LOCAL ATTACK LEG - ROUND alloc-basis-r2

You are one of four independent reviewers checking a design decision for
singlefs, a copy-on-write filesystem being designed from scratch. You are
given a self-contained set of facts below about one specific, concrete
history of publishes (a fixed test script that has actually been run
against the real implementation). Do not assume anything not stated here.
Read carefully: every qualifier in this text is load-bearing (a definition
with one word removed can flip the answer). Do not use any markdown
emphasis (no bold, no italics, no asterisk bullets) anywhere in your answer.
Number your answers to match the question numbers given at the end. For
every numbered answer you give, add one sentence starting with "This would
be refuted by:" describing the specific observation that would prove that
answer wrong. Where a question asks you to fill in a table, give the table
with explicit slot numbers and explicit counts, not just "yes" or "no".

Background. singlefs journals writes and periodically "publishes" a new
root that becomes durable in a root ring (a fixed-size ring buffer of
root-record slots on each device, mirrored across 2 devices, device0 and
device1). Storage is divided into fixed-size 16 KiB slots; a "unit" is one
or more contiguous slots. Each root record points to several units,
including an "instance table unit" and a "tree table unit" and, indirectly,
the current file's "data unit".

An administrator can roll back to an older root R_old. After a rollback,
some earlier roots become "abandoned" (their timeline is no longer the live
one); the live accounting statistics (allocated bytes, free bytes, the
defer/release queue) are reloaded from R_old's own account, not from
whatever was live just before the rollback. A hard rule (call it the main
clause) states: before an abandoned-timeline root leaves the root ring
(gets physically overwritten by ring rotation), any unit it references must
not be reallocated to something else, and must not have its header erased.
The mechanism that enforces this is called the shadow ledger: it separately
tracks, for each abandoned root, which units that root's own account still
shows as allocated (not released, as of that abandoned root's own frozen
snapshot), and blocks the allocator from handing those units out, on top of
and in addition to the normal release/reclaim accounting described below.

Root ring geometry. There are R = 3 regions, numbered 0, 1, 2. Region 0 and
region 2 are backed by device0; region 1 is backed by device1. For a
publish with checkpoint generation number txg = t, the region it lands in
is (t mod 3); the slot within that region is (t div 3) mod 8, where "div"
is integer division (S = 8 slots per region in this implementation).

Normal release and reclaim accounting (this applies regardless of which
shadow-ledger reading is chosen; it is a separate, independent mechanism).
Every publish that supersedes an old version of some structure (instance
table, tree table, file data) releases the old unit it replaces, recording
a release-generation equal to the txg of the publish that did the
releasing. A released unit becomes reclaimable (eligible to be handed out
again to a new allocation) once its release-generation is less than or
equal to a reclaim-floor. The reclaim-floor equals the maximum of
F_effective and the txg of the oldest valid root physically present in the
root ring; in this particular script the oldest valid root in the ring
stays at txg 0 throughout (the genesis root written by mkfs is never
physically overwritten and always counts as valid), so the reclaim-floor
equals F_effective throughout this script. F is a rollback-floor value
carried in every root record; it is raised only by an explicit forced
mechanism (never automatically in this script) and only becomes
"F_effective" once every surviving device carries a persisted root record
with the new, raised F value; until that happens the previous F value
remains effective.

Section 1: the concrete publish history (this is the actual fixed test
script, from the first file write through the write that lands right after
a rollback-floor raise).

txg 3, instance 1, called "A": first file write. Region = 3 mod 3 = 0,
device0, F carried = 0. A is the root that the administrator later rolls
back to (A = R_old).

txg 4, instance 1, called "B": overwrite of the same file. Region = 4 mod
3 = 1, device1, F carried = 0.

txg 5, instance 2, called "write-row": the first publish after a process
restart and remount as a new instance (instance 2). This publish rewrites
the instance table (adding a row recording instance 1's history) and
several other fixed structures. Region = 5 mod 3 = 2, device0, F carried =
0.

txg 6, instance 2, called "warmup-1": an empty publish (changes no
user-visible state) forced so instance 2's root becomes durable on both
devices before any write is acknowledged to the user. Region = 6 mod 3 =
0, device0 (this lands on the same device as txg 5, so it is redundant for
coverage purposes but it still happens). F carried = 0.

txg 7, instance 2, called "warmup-2": second empty publish. Region = 7 mod
3 = 1, device1. F carried = 0. After this publish, instance 2's root
covers both devices.

txg 8, instance 2, called "C": overwrite of the same file (replacing B's
content). Region = 8 mod 3 = 2, device0. F carried = 0.

txg 9, instance 3, called "D": the administrator has rolled back to A
(txg 3). This is the first publish of the new instance 3, called
"rollback-write-row". It rewrites the instance table, writing two new
rows: a rollback row for instance 1, and an intermediate row for instance
2 (see Section 2 below). It does not touch the file's data pointer.
Region = 9 mod 3 = 0, device0. F carried = 0.

txg 10, instance 3, called "rollback-warmup": empty publish forced so
instance 3's root covers both devices before the rollback is acknowledged.
Region = 10 mod 3 = 1, device1. F carried = 0. After this publish,
instance 3's root covers both devices; this is the first point at which
the rollback is fully durable and acknowledged.

txg 11, instance 3, called "overwrite-11": first user overwrite of the
file within instance 3 (replacing A's original data with new content).
Region = 11 mod 3 = 2, device0. F carried = 0.

txg 12, instance 3, called "overwrite-12": another user overwrite. Region
= 12 mod 3 = 0, device0. F carried = 0.

txg 13, instance 3, called "overwrite-13": another user overwrite. Region
= 13 mod 3 = 1, device1. F carried = 0.

txg 14, instance 3, called "overwrite-14": another user overwrite. Region
= 14 mod 3 = 2, device0. F carried = 0.

txg 15, instance 3, called "raise-F-1": an empty publish forced internally
to raise the rollback floor. This publish's own root record carries the
new value F = 11. Region = 15 mod 3 = 0, device0.

txg 16, instance 3, called "raise-F-2": a second empty publish, needed
because F only becomes F_effective once every device carries a persisted
root with the new F. This publish's root record also carries F = 11.
Region = 16 mod 3 = 1, device1. Because device0 already got F = 11 at txg
15 and device1 gets it now at txg 16, F_effective becomes 11 exactly at
this point (F_effective was still 0 immediately after txg 15, since
device1 had not yet caught up).

txg 17, instance 3, called "E": the write whose landing slot this round is
asking about. Region = 17 mod 3 = 2, device0. The root immediately before
E (txg 16's root) carries F = 11, so F_effective = 11 at the moment E is
about to allocate.

Section 2: the instance table, as rewritten by D (txg 9), and the
abandoned-root rule.

The instance table now has exactly two rows: row (instance = 1, T = 3,
W = 0, rollback-flag = 1), and row (instance = 2, T = 0, W = 0).

The abandoned-root rule: a root belonging to instance i, with txg strictly
greater than the T value in instance i's row, is abandoned. Applying this
rule: instance 1's row has T = 3, so any instance-1 root with txg > 3 is
abandoned -- this makes B (txg 4) abandoned; A (txg 3) is not abandoned
(3 is not greater than 3). Instance 2's row has T = 0, so every instance-2
root is abandoned regardless of its txg -- this makes write-row (txg 5),
warmup-1 (txg 6), warmup-2 (txg 7), and C (txg 8) all abandoned. No
instance-3 root is abandoned (instance 3 has no row in this table, and the
abandoned-root rule only applies to a root whose own instance has a row).

So the full set of abandoned roots in this script is: B (txg 4), write-row
(txg 5), warmup-1 (txg 6), warmup-2 (txg 7), C (txg 8). Five roots.

Section 3: the four units this round cares about, and exactly when each
one is allocated and released (these are physical facts about the COW
structure, independent of which shadow-ledger reading is chosen).

Unit U1, slots 50176-50177 (2 slots, same slot numbers on both devices,
mirrored): the instance-table unit written by mkfs before txg 3. A's own
root record (txg 3) points to U1 as its instance-table unit, and this
pointer inside A's own frozen root record never changes for as long as A
exists -- A always points to U1. B's root record (txg 4) also points to
U1 (B does not rewrite the instance table). Write-row (txg 5) rewrites the
instance table, allocating a brand new instance-table unit and thereby
releasing U1, with release-generation = 5, within instance 2's own
(abandoned) timeline. Separately, because going forward from the rollback
the live accounting resets to R_old = A's own account (which never
released U1, since A itself does not rewrite the instance table), D (txg
9) is the publish that rewrites the instance table in the live, going
forward timeline, allocating a new instance-table unit and releasing U1
again, with release-generation = 9, in the live master timeline.

Unit U2, slot 50178 (1 slot, span = 1): the genesis tree-table unit
written by mkfs before txg 3. A's own publish (txg 3) rewrites the tree
table (recording the first file's entry), allocating a new tree-table
unit and releasing U2, with release-generation = 3. This release happens
inside A's own publish. No abandoned root is involved in releasing U2 at
all; U2 was already released before B, write-row, or any other abandoned
root ever existed.

Unit U3, slots 50180-50181 (2 slots): A's data unit, allocated by A (txg
3); A's root record points to U3 as the file's content at that time. B's
publish (txg 4) overwrites the file, releasing U3 with release-generation
= 4, within B's own publish. Going forward from the rollback, live
accounting resets to R_old = A's account, in which U3 was never released
(A's own record never releases its own data). Overwrite-11 (txg 11) is the
first publish in the live, new instance-3 timeline to overwrite the file,
and it is the one that releases U3 in the live master timeline, with
release-generation = 11.

Unit U4, some other pair of slots (the exact slot numbers do not matter
for this round's questions): B's own data unit, allocated by B (txg 4).
C's publish (txg 8) overwrites the file again, releasing U4 with
release-generation = 8, within C's own (abandoned) publish.

Slot 50179 (a single slot) has never been allocated by anything in this
script; it is free from the start.

Given fact, not to be re-derived: instance 2's abandoned timeline (write-
row, warmup-1, warmup-2, C) also allocated several other units of its own
(fixed-point structures rewritten at txg 5 and at each warmup, plus U4
above) that no valid root anywhere references. The aggregate size of all
such abandoned-instance-2-exclusive, still-allocated-in-some-abandoned-
root's-own-account units, per device, is 34 slots. This number (34) is
computed as a single subtraction of two whole-pool statistics (the
allocated-bytes statistic reachable from the most recent abandoned root,
C, minus the allocated-bytes statistic reachable from R_old = A), without
enumerating individual slots; you do not need to and should not try to
list all 34 slots individually. This 34 does not include U1 (U1 is also
referenced by A, so it cancels out of this particular subtraction), and
does not include U2 or U3 (neither one is ever "still allocated" in any
abandoned root's own account snapshot, for the reasons given above).

Section 4: given facts about the reading actually implemented in the
running code today (called the "conservative" reading below; its exact
rule is given in Section 5). These numbers have already been measured by
running the real implementation on this exact script; they are given to
you as ground truth, not something to re-derive:

Fact 4a: measured immediately after the rollback and its warmup (txg 10),
the conservative reading isolates 36 slots per device: the 34-slot
aggregate from Section 3, plus U1's 2 slots.

Fact 4b: measured right before E allocates (txg 17), under the
conservative reading, E's data lands at U2's slot pair, 50178-50179 (U2 at
50178, paired with the never-allocated 50179), not at U1 (50176-50177).

Section 5: the checker's candidate-set definition (used by invariant I-2.1,
checksum matches content, and invariant I-3.1, allocated statistics add
up). A root counts as being in the checker's candidate set if and only if
it is not abandoned (per the Section 2 rule) AND its txg is greater than
or equal to the F value carried by the single latest root record (not
F_effective, and not the F carried by the root itself -- the F carried by
whichever root is the newest one that exists at the moment the check
runs). Only units referenced by a root in this candidate set are checked
by I-2.1 and included in I-3.1's sum; a root outside this candidate set is
not checked at all, and none of its content is verified, by either of
these two invariants.

Section 6: three candidate readings of the shadow ledger's isolation rule.
Under all three readings, the set of "abandoned roots" (Section 2) is the
same, and a unit only becomes a candidate for isolation in the first place
if it is "still allocated" (not released) in the own account/snapshot of
at least one abandoned root (as illustrated for U1 in Section 3: U1 is
"still allocated" in B's own account, because B's own snapshot, taken
right after B's own publish, has never released U1). The three readings
differ only in whether such a candidate unit gets an exemption from
isolation because some other, non-abandoned root also references it.

Reading narrow: isolate a candidate unit unless it is also referenced by
at least one root that is currently valid (not abandoned, per Section 2).
Whether that referencing root is inside or outside the rollback candidate
set (valid AND txg >= F_effective) does not matter for this exemption; a
root that has simply fallen below F_effective, without being abandoned,
still counts as "valid" for this exemption.

Reading conservative (Section 4's rule, the one actually implemented
today): isolate every candidate unit. There is no exemption for also
being referenced by a valid root.

Reading G5: isolate a candidate unit unless it is also referenced by at
least one root that is currently valid AND currently inside the rollback
candidate set, meaning valid AND txg >= F_effective. A valid root whose
txg has fallen below F_effective no longer provides this exemption.

Section 7: the main clause, restated precisely (this is the rule the
shadow ledger exists to enforce): before an abandoned-timeline root leaves
the root ring, any unit that root references must not be reallocated to
anything else. A unit "leaves being protected by this clause" only when
every abandoned root that references it has itself left the ring (been
physically overwritten by ring rotation); in this script no root ever
leaves the ring (the ring has 24 slots total, R times S = 3 times 8, and
the highest txg reached is 17), so every abandoned root listed in Section
2 is still physically present in the ring at txg 17.

Section 8: the formula this round is comparing isolated counts against
(already given as computed in Section 3: 34 per device). This is decision
D28's item 9, "abandoned-root-exclusive amount": it equals (the allocated-
bytes statistic reachable from the abandoned root(s)) minus (the
allocated-bytes statistic reachable from R_old). It is meant as an upper
bound on how much space the shadow ledger should need to isolate. This
formula's own value is fixed at 34 per device throughout this script (it
does not depend on which of the three Section 6 readings is chosen; it is
computed the same way regardless).

Now the questions. For all of these, "isolated slots" means slots isolated
by the shadow ledger specifically (on top of, separately from, normal
release/reclaim accounting); do not count a slot as "isolated" just
because it has not yet been released, or just because its release-
generation has not yet reached the reclaim-floor.

Question set Z4A. For each of the three readings (narrow, conservative,
G5) and each of three points in time (right after the rollback and its
warmup, txg 10; right after F becomes effective, txg 16; right before E
allocates, txg 17), state: (a) which of U1, U2, U3 are isolated by the
shadow ledger at that point under that reading (note: U2 and U3 are never
"still allocated" in any abandoned root's own account at all, per Section
3, so state explicitly whether they can ever be isolated under any
reading); (b) the isolated count per device, computed as 34 plus however
many of U1's 2 slots are isolated under that reading at that point (state
this as a number, for example 34 or 36); (c) the difference between that
count and the Section 8 formula's value of 34 (so 0 or +2); (d) whether U1
is currently reclaimable under the normal release/reclaim accounting rule
given in the Background section, independent of whether it is isolated
(state the release-generation you are comparing against the reclaim-floor,
and the reclaim-floor's value at that point); (e) the lowest-numbered,
lowest-slot-number pair of two adjacent slots that could currently be
handed out to a new user-data write, considering only U1, U2 (paired with
slot 50179), and U3 as candidates, and considering both the normal
release/reclaim rule and the shadow-ledger isolation under that reading
together (if nothing is currently available, say so explicitly, do not
guess a slot number).

Give 9 numbered answers for question set Z4A: Z4A.narrow.txg10,
Z4A.narrow.txg16, Z4A.narrow.txg17, Z4A.conservative.txg10,
Z4A.conservative.txg16, Z4A.conservative.txg17, Z4A.G5.txg10,
Z4A.G5.txg16, Z4A.G5.txg17. Each answer must give all five items (a)
through (e) above, with explicit slot numbers.

Check your Z4A.conservative.txg10 and Z4A.conservative.txg17 answers
against the given facts in Section 4 (4a and 4b) before finalizing your
answer; if your computed answer disagrees with the given fact, say so
explicitly and explain the disagreement rather than silently adjusting
your answer to match.

Question set Z4B (equivalence). Are the three readings (narrow,
conservative, G5) equivalent, in this script, on the question of "which
slots can be issued to a new write" -- meaning: at every one of the three
timepoints in Z4A, do all three readings produce the exact same lowest-
issuable pair (item e above)? If they are not equivalent, state the first
timepoint (txg 10, txg 16, or txg 17) at which two of the readings first
give a different lowest-issuable pair, name which two readings differ
there, and give both slot-pair answers.

Give one numbered answer, Z4B.

Question set Z4C (main clause check). Using your own Z4A answers for the
lowest-issuable pair at txg 17 under each of the three readings: for each
reading, is that lowest-issuable slot pair one that is referenced by at
least one abandoned root listed in Section 2 (name which abandoned root,
if so)? If a reading's own lowest-issuable pair at txg 17 is a unit
referenced by an abandoned root, does handing that pair out to E's write
violate the main clause given in Section 7? Answer this separately for
each of the three readings.

Give one numbered answer, Z4C, covering all three readings.

Question set Z4D (false ENOSPC potential). This script's pool has roughly
211968 total unit-area slots. The difference between the conservative/G5
isolated count and the narrow isolated count, at txg 17, is at most 2
slots (per device). Could isolating those 2 extra slots, under
conservative or G5 but not under narrow, ever be the deciding factor
between a write succeeding and the filesystem reporting ENOSPC (out of
space) to the user, given requirement 1 from decision D3's item 9 (as
long as free space reported is >= s, writing s bytes must succeed)? If
your answer is that it could happen, describe concretely what pool
capacity and what pending write size s would make the 2-slot difference
decide the outcome. If your answer is that it could not happen in this
particular script as given, say so and explain why not, referring to the
actual slot numbers and counts from your Z4A answers rather than giving a
general argument.

Give one numbered answer, Z4D.

Question set Z4E (verifiability, checker coverage). Using the checker's
candidate-set definition from Section 5: at txg 17, is root A (txg 3) in
the checker's candidate set, given that the single latest root at that
point (the root written at txg 16) carries F = 11? If reading narrow's
lowest-issuable pair at txg 17 (from your Z4A.narrow.txg17 answer) is
handed out to E, and it turns out to overlap with U1 (the same physical
slots A's own root record still points to as its instance-table unit),
would invariant I-2.1 (checksum matches content) or invariant I-3.1
(allocated statistics add up), as defined in Section 5, detect this as a
problem on the very next checker run? Answer separately for I-2.1 and for
I-3.1, and state explicitly whether A's content is included in either
check's scope at that point. If you conclude that a check would catch this corruption, name which check
catches it; if you conclude neither check would catch it, say so
explicitly.

Give one numbered answer, Z4E.

Do not answer any question outside Z4A, Z4B, Z4C, Z4D, Z4E as defined
above. In particular do not discuss the D28-versus-D16 df formula
disagreement, the two admission gates in series, or any candidate for
item 1's definition beyond what is given here -- those are separate
problems, not assigned to you.
