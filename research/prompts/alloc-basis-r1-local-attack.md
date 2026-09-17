SINGLEFS DESIGN QUESTION - LOCAL ATTACK LEG - ROUND alloc-basis-r1

You are one of four independent reviewers checking a design decision for
singlefs, a copy-on-write filesystem being designed from scratch. You are
given a self-contained set of facts below. Do not assume anything not stated
here. Read carefully: every qualifier in this text is load-bearing (a
definition with one word removed can flip the answer). Do not use any
markdown emphasis (no bold, no italics, no asterisk bullets) anywhere in your
answer. Number your answers to match the question numbers given at the end.
For every numbered answer you give, add one sentence starting with "This
would be refuted by:" describing the specific observation that would prove
that answer wrong.

Background. singlefs journals writes and periodically "publishes" a new
root that becomes durable in a root ring (a fixed-size ring buffer of
root-record slots on each device). Users see "df" (free space reported) and
issue writes; the filesystem must decide whether to admit a write (an
"admission" check) before it allocates physical space for that write.

Section 1: statistics maintained.

The filesystem maintains, among other things, these accounting statistics,
each updated incrementally as part of each publish (never recomputed by
scanning):

- item 1, "allocated bytes", per device.
- item 2, "free bytes", per device. This item must be maintained completely
  independently of item 1; it must never be computed on the fly as
  (capacity minus allocated). If it were computed that way, any invariant
  checking "free plus allocated equals capacity" would be a tautology with
  zero power to catch a bug.
- item 5, "defer queue pending-release bytes", per device, tracked by
  release-generation.

"Capacity" here means the size of the unit area (from the superblock's
unit-area-start-slot to the end of the disk); fixed structures (superblock
slots, root ring, journal ring) are excluded from both "capacity" and
"allocated".

"Release" is defined as the moment the allocator puts a placement (a
physical location) into the defer queue -- that is, the moment no root
(including snapshot roots) references it any more.

Section 2: root ring and "oldest valid root in ring".

The root ring has R = 3 regions. The region a given publish with checkpoint
generation number txg = t lands in is (t mod R). Within a region, the slot
used is (t div R) mod S, where S is at least 4 slots. Each publish writes
exactly one new root record to one slot.

Define "oldest valid root in ring" as: the minimum checkpoint_txg among all
root-ring slots on disk whose current content (a) self-verifies (checksum
and any other self-consistency checks pass) and (b) is judged still valid by
the instance table. A slot whose most recent write attempt failed is read,
for this definition, as still holding its old (previous) content. A publish
that is in flight and not yet durable does not count at all for this
definition.

Define F (the rollback floor) as a value carried in each root record,
normally unmoved (so the reclaim window spans the whole ring), and only
raised when the pool is tight, through an explicit filesystem-internal
mechanism ("raising F", done by forcing extra publishes). F becomes
"effective" (F_effective) only once every surviving device carries a
persisted root with the new F value; until then the previous F value remains
effective.

Define "reclaimable" (eligible to be handed out again as a new allocation)
as: a placement is reclaimable if and only if it has been released AND its
release-generation is less than or equal to max(F_effective, oldest-valid-
root-in-ring).

Section 3: root slot write failure and crash-before-persist.

Failure mode 1. When a publish's write to a root-ring slot fails, the
filesystem retries the publish; on that retry, checkpoint_txg is advanced by
exactly one before writing the next attempt (it does not retry at the same
checkpoint_txg, and it does not advance by more than one). This is required
because the region a txg lands in is (txg mod R); retrying at the same txg
would repeatedly target the same region and slot, collapsing the R
independent failure domains into one.

Failure mode 2. The publish crashes before any root-ring slot durably holds
the new root record (the write to the root-ring slot for that publish either
never started or never completed). On the next mount, the newest root that
self-verifies and is judged valid is whatever root was durable before this
publish attempt began; the attempted publish leaves nothing that counts as
"durable".

Section 4: the two admission formulas as currently written.

The design documents state, as a hard requirement, that a write request
composes two gates in series: it must first pass gate D28, then pass gate
D16's logic.

Gate D28 ("available"): available = sum over devices of (capacity minus
allocated minus unrecoverable minus defer_pending_release minus
mount_time_commitment minus abandoned_root_exclusive) minus pending_delete
minus committed_reserve minus checkpoint_reserve_pool.

One design document states plainly: "available already has [the mount-time
commitment item] deducted, df reports available" -- meaning the number shown
to the user as free space (df) is exactly this "available" quantity computed
by gate D28's formula above.

Gate D16 ("allocatable", and its own "df"): allocatable = min(reclaimable
plus live_metadata minus reserve_pool, df). If admission using this
allocatable value is insufficient, the filesystem first tries internally,
within this one admission attempt, to force extra publishes whose purpose is
to raise F (this is called "draining"), up to B = 8 such publishes total;
only if admission is still insufficient after up to B = 8 attempts does it
report ENOSPC (out of space) to the caller. Publishes forced purely to raise
F this way do not change any user-visible state.

Gate D16 separately defines its own df formula: df = unused_bytes plus
released_bytes plus live_metadata_bytes minus reserve_pool minus
worst_case_drain_residual minus lag_amount, where:

- reserve_pool = 10 + 7 times c_max (c_max is the current worst-case cost in
  blocks of one drain publish, recomputed at every publish from the current
  tree heights).
- worst_case_drain_residual = 5 + 6 times c_max.
- lag_amount = the total size of released bytes whose release-generation is
  greater than max(the txg of the 4th-newest non-empty valid root,
  oldest-valid-root-in-ring's txg), and which were NOT released by a drain
  (raise-F) publish itself. ("Non-empty" here means a publish that changed
  user-visible state, as opposed to a drain publish.)

The apparent contradiction this round investigates: gate D28's formula
subtracts "defer_pending_release" (item 5) from available, and one document
says df equals this available value (Section 4 above). But gate D16
separately defines its own df formula that INCLUDES released bytes (via the
released_bytes term), only netting out lag_amount, worst_case_drain_residual
and reserve_pool -- a much smaller deduction than subtracting all of
defer_pending_release outright. These two df values are not obviously the
same number. Because gate D28 is checked first in the series (its own
"available" gate is evaluated before gate D16's logic runs), a write that
only gate D16's logic (with its raise-F draining) could eventually admit
might already be rejected at gate D28, before gate D16's draining logic ever
gets a chance to run.

Two ways to resolve this composition are proposed:

Reading "Serial": keep the two gates literally in series as stated above.
The user-visible df is gate D28's "available" value. A write of s bytes is
admitted only if it first passes D28's available-greater-or-equal-to-demand
check using D28's "available" formula exactly as given (this check does NOT
get the benefit of D16's raise-F draining before it runs), and then
separately passes gate D16's own logic.

Reading "Merged": change gate D28's available computation so that released
bytes that raising F within the bounded B = 8 drain publishes could make
reclaimable are counted as available already at the first gate; under this
reading the first gate's available formula only nets out D16's lag_amount,
worst_case_drain_residual, and reserve_pool (not defer_pending_release
computed the D28 way). Under this reading, the user-visible df becomes
exactly gate D16's df formula given above, and gate D28's sentence "df
reports available" is rewritten to match.

Section 5: three candidate designs for how item 1 ("allocated") is defined,
and which roots' references I-3.1's checker sums to check it.

Candidate A: item 1 ("allocated") = placements still referenced by a
currently-valid root, PLUS released placements that are not currently
reclaimable (this matches what today's Rust implementation actually does).
I-3.1's checker takes the union of references over the "rollback candidate
set" -- defined as: roots judged valid by the instance table AND with
txg greater than or equal to F_effective. This is a narrower set than "all
self-verifying roots in the ring": a root with txg less than F_effective,
even if it still self-verifies and physically sits in the ring, is excluded
from this union.

Candidate B: item 1 ("allocated") = placements still referenced by a
currently-valid root ONLY (does not include anything sitting in the defer
queue, regardless of whether it is currently reclaimable). I-3.1's checker
takes: allocated equals the sum of references reachable from only the
single latest (highest-txg) valid root (not a union over multiple roots).
Separately, Candidate B adds one new, additional invariant: allocated plus
defer_pending_release equals the union of references over the same
"rollback candidate set" used by Candidate A.

Candidate C: this is what today's implementation and today's checker code
actually do. Item 1 ("allocated") = every placement that is either
currently referenced by a valid root, or has been released but has not had
its slot bit cleared (this matches Candidate A's definition of item 1;
released placements are never removed from item 1 merely by being
released). I-3.1's checker takes the union of references over ALL
self-verifying roots currently physically present in the root ring -- this
union is NOT filtered by the instance table and does NOT check txg against
F_effective at all; it is a wider set than Candidate A's "rollback candidate
set", because it includes roots with txg less than F_effective, and it
includes roots that self-verify but might belong to an instance timeline
that has been abandoned.

For all three candidates, item 2 ("free bytes") is defined identically:
independently maintained, never computed as capacity minus allocated, so
that "free plus allocated equals unit_area_capacity" (invariant I-5.2) is a
real, falsifiable check, not a tautology.

Section 6: the two invariants under test.

I-3.1 ("allocated statistics add up"): allocated space statistic equals the
sum actually obtained by traversing all references. The exact set of roots
whose references get unioned to compute that sum is what differs across
Candidates A, B and C above.

I-5.2 ("free statistics add up"): free space statistic equals total space
minus allocated space.

Section 7: D3's decision item 9, the two hard requirements this design must
satisfy.

Requirement 1 (no false ENOSPC): as long as df reports free space greater
than or equal to s, writing s bytes must succeed. The filesystem is allowed
to do bounded internal work first (forcing publishes, raising F, and so on)
before allocating; it may only report ENOSPC if df's reported free space is
genuinely less than s.

Requirement 2 (deleted space becomes available within a bounded number of
steps): after deleting an object of s bytes, a write of the same size s must
succeed within a bounded number of steps. This bound must be a computable
number, not "eventually". The bound is fixed at 3 publishes that change
user-visible state (a "publish" here means a full commit and checkpoint
cycle that produces a new durable root). Publishes the filesystem forces
internally purely to drain and raise F do NOT count toward this count of 3
-- only publishes that change something the user can observe count toward
it.

Now the questions.

Question set Y2 covers six combinations: each of Candidates A, B, C combined
with each of Reading Serial and Reading Merged (Section 4).

For each of the six combinations (A-Serial, A-Merged, B-Serial, B-Merged,
C-Serial, C-Merged), answer two things:

(a) Does Requirement 1 (no false ENOSPC) hold for this combination?
(b) Does Requirement 2 (deleted space available within 3 user-visible
    publishes) hold for this combination?

For any combination where you answer "no" to (a) or (b), construct one
concrete reachable history that demonstrates the failure. Your history must
state: the pool's device count and per-device slot capacity; a sequence of
publishes, each stating what was allocated and what was released (with
release-generation numbers) in that publish; the value of F and the
oldest-valid-root-in-ring's txg at each point that matters; what df reports
at the moment of the failing write; how many bytes that write attempts; and
whether the write succeeds or fails, and why, under the specific
combination's rules. If you answer "yes" to (a) or (b) for a combination,
you do not need to construct a failing history for that sub-answer, but say
so explicitly and explain briefly why no such history can be constructed.

Give 12 numbered answers for question set Y2: number them Y2.A.Serial.1,
Y2.A.Serial.2, Y2.A.Merged.1, Y2.A.Merged.2, Y2.B.Serial.1, Y2.B.Serial.2,
Y2.B.Merged.1, Y2.B.Merged.2, Y2.C.Serial.1, Y2.C.Serial.2, Y2.C.Merged.1,
Y2.C.Merged.2, where ".1" is requirement 1 and ".2" is requirement 2.

Question set Y5 covers root-slot write failure and crash-before-persist,
for three candidates times two failure scenarios times two invariants,
twelve judgments total.

Scenario 1 is the root-slot write failure of Section 3 (checkpoint_txg
advances by exactly one on retry). Scenario 2 is the crash-before-persist of
Section 3 (no root-ring slot durably holds the new root; the previously
durable root remains the newest self-verifying valid root).

For each of the three candidates (A, B, C) and each of the two scenarios
(1, 2), construct one concrete recovered pool state: specify device count;
which placements are allocated, released, or reclaimed at that point; which
root-ring slots hold which txg and content after the failure or crash; and
what accounting statistics item 1 (allocated) and item 2 (free) currently
read on that state. Then state what I-3.1 judges (holds, or violated) and
what I-5.2 judges (holds, or violated) for that candidate on that state. If
a candidate's checker union range (Section 5) makes I-3.1 compare against a
different reference set than what item 1 actually counts, say explicitly
which placements are on one side of the comparison but not the other.

Give 12 numbered answers for question set Y5: number them Y5.A.Scenario1,
Y5.A.Scenario2, Y5.B.Scenario1, Y5.B.Scenario2, Y5.C.Scenario1,
Y5.C.Scenario2, each answer covering both I-3.1 and I-5.2 for that
candidate and scenario.

Do not answer any question outside Y2 and Y5 as defined above. In
particular do not discuss double-counting of the defer term against item 1
(that is a separate problem, not assigned to you), do not discuss the
timing-lag problem about roots being overwritten by ring rotation (also not
assigned to you), and do not discuss which of readings (i), (ii), (iii) is
the officially correct reading of "all valid roots" beyond what Section 2
and Section 5 already give you for Candidates A, B and C specifically.
