SINGLEFS CHECKPOINT TRIGGER DESIGN REVIEW - LOCAL ATTACK LEG - ROUND checkpoint-trigger-r1

You are one of three independent reviewers examining a proposed change to
the checkpoint trigger rule of singlefs, a copy on write filesystem being
designed from scratch. You are given a self contained set of facts below.
Do not assume anything not stated here. Read every fact carefully, every
qualifier is load bearing, a definition with one word removed can flip an
answer. Do not use any markdown emphasis anywhere in your answer, no bold
text, no italic text, no backtick code formatting, no asterisk bullets, no
pipe tables. Number your answers to match the four judgments given at the
end. For each judgment give three separate labeled parts: a verdict stated
as an explicit phrase, not a bare yes or no and not merged with another
judgment's verdict; two to four sentences of justification that name the
specific fact numbers your verdict rests on; and one sentence starting
with "This would be refuted by:" describing the specific observation that
would prove that verdict wrong. When your answer needs to point at a
source, point at it only by the fact number given below, fact 1 through
fact 10. Do not write any source file name together with a line number,
and do not write any line number at all anywhere in your answer, because
the underlying files may have moved to different line numbers by the time
anyone reads your answer; if you need to name a piece of code, name the
function by its own name instead.

This is a separate, independent document from other documents in this same
review round that ask about other parts of the same proposed change,
covering whether the new quantity introduced by this change can be read
cheaply enough to belong in a runtime decision path rather than requiring
a full scan, and covering two other questions about whether the proposed
change creates a new deadlock and about whether the currently shipped rule
already has a real gap. You are not expected to see those other documents
and should not assume anything about their content, and you are not being
asked here whether this new quantity belongs in a runtime decision path or
in some other category of accounting; you are only being asked, in
judgment four below, whether the underlying source code already computes
this quantity today, as a plain fact about what exists, not a design
recommendation.

Background. Today, singlefs decides when to run a checkpoint using a rule
with two branches joined by or: either enough time has passed since the
last checkpoint, or enough bytes have become dirty. A proposal on the
table adds a third branch to that same rule, based on how much of the
journal's fixed size ring buffer has already been used up since the start
of the current checkpoint interval. The facts below describe the exact
wording of the current rule, the exact wording of the proposed third
branch, one earlier finding about a weakness in the current two branch
rule, the current implementation status of the two thresholds involved,
and a set of direct source code searches carried out for this review.

Fact 1, the current trigger rule's exact definition, from the project's
decision record. The definition states: a checkpoint trigger holds if and
only if either the elapsed time since the last checkpoint is greater than
or equal to a threshold called T_time, or the current dirty byte count is
greater than or equal to a threshold called T_dirty. Both thresholds are
values declared in the on disk system configuration, and both are
evaluated once at mount time, the same pattern the project uses elsewhere
for certain geometry values that are also fixed once at mount time.

Fact 2, a hinge sentence from the same decision record's entry that fixes
the numeric values of the two thresholds. It states: T_dirty and T_time
follow two different rules. T_dirty follows a clamping rule: its effective
value equals the minimum of the value stored in its configuration field
and the ring length divided by a safety factor called F, and the stored
field itself is never changed by this clamping. T_time follows a warning
rule: a configured value outside its allowed range is not rejected, it
only produces a warning. The same sentence adds explicitly: within this
one passage the two fields follow two different rules, and they must not
be read as if they followed the same rule as each other.

Fact 3, the concrete numeric definition of T_time, from the same decision
record entry as fact 2. T_time's default value is 5 seconds, chosen as the
upper end of a feasible range running from 200 milliseconds to 5 seconds;
the stated reasoning is that the default data loss window after a crash is
exactly that 5 seconds, and choosing a smaller default would only reduce
how much work gets amortized per checkpoint without shrinking that loss
window any further, since the loss window itself is bounded by the full
5 seconds regardless. T_time is user settable without restriction: a value
supplied at mount time overwrites the on disk field and is written back to
disk; a supplied value outside the 200 millisecond to 5 second range is
not rejected, it only produces a warning, per a user decision recorded as
made on 2026 09 20.

Fact 4, the concrete numeric definition of T_dirty, from the same decision
record entry as facts 2 and 3. T_dirty equals 2 gibibytes, described as an
initial value meant to be tuned later once the code implementing it is
written. It is explicitly stated not to be determined by the data loss
window, because under the or rule from fact 1 the time branch already
guarantees that window on its own; instead T_dirty governs a different
quantity, described as the dirty page memory budget itself. T_dirty is
described as a tunable value rather than a fixed format constant: the slot
that holds it inside the system configuration is fixed, but the number
placed inside that slot is not. At mount time T_dirty is clamped once
more, per the clamping rule from fact 2: its effective value equals the
minimum of the field value stored in the system configuration and the
ring length divided by the safety factor F. The ring length is itself an
mkfs time parameter, defaulting to 768 mebibytes and constrained to never
exceed one quarter of the underlying device's capacity. Under this default
ring length, the effective T_dirty value works out to the minimum of
2 gibibytes and 256 mebibytes, which equals 256 mebibytes; the field
stored on disk still holds 2 gibibytes and is never itself changed by this
computation.

Fact 5, the project's own invariant covering the journal ring's geometry,
from its invariant registry, together with its current implementation
status and a calibration note attached to it. The invariant states: the
ring's total size must be at least F times the worst case journal
occupancy of any single transaction, where F is a safety factor of at
least 2; both this worst case occupancy figure and F are written into the
system configuration, computed once at the time the filesystem is
created, and recomputed and compared again every time it is mounted. Its
checker's implementation status is recorded as not implemented. A
calibration note dated 2026 09 13 attached to this same invariant states:
a related cost figure, called ckpt_cost, is instead computed on the fly
from the current height of each of several record trees, and this figure
can grow over the course of a single mount session; whether the worst
case occupancy value declared at filesystem creation time should already
be an upper bound that accounts for that later growth, or should instead
also be recomputed and compared again during the mount session, has not
been decided. A separate warning elsewhere in the same registry states:
the entire argument for choosing a fixed length ring rather than a
growable one for the journal rests on this one invariant holding, and
this invariant's checker is not implemented; being written down in the
checklist does not by itself mean anything is actually stopping a
violation of it from happening at runtime.

Fact 6, a tracked gap in the project's own registry of owed checks,
quoted here across all of its recorded columns. Its short description is:
no clause exists constraining the journal occupancy of one checkpoint
interval. The phenomenon it records, in full: the two clauses meant to
keep the journal ring from overflowing both take as their grammatical
subject any operation or any transaction, namely one item inside the
journal's own decision record and the invariant from fact 5; under the
journal form actually chosen for this project, described in fact 8 below,
one checkpoint interval and one transaction happen to have exactly the
same extension, so this gap has never been exposed in practice; under a
different, write ahead log style form, one checkpoint interval would
instead span many transactions, so how much journal one interval occupies
would become, for the first time, an independent quantity, and no clause
anywhere in the project constrains that quantity; one empirically observed
point where this actually gets crossed, found during an earlier round of
independent review, is that T_time crosses ring length divided by F at
T_time's own default value of 5 seconds. The suggested way to catch this
gap, in full: establish a clause stating that the journal occupancy of one
checkpoint interval must not exceed some declared fraction of the ring,
and turn it into a check that can fail, computed from the fsync rate,
T_time, and the number of transactions inside one interval, using T_time's
default value as the sample expected to already fail today. The recorded
reason this has not yet been fixed, in full: it first requires deciding
whether the write ahead log style form is adopted at all; two prior rounds
of independent review concluded that this decision currently stays with
the actually chosen form from fact 8, unchanged, so this gap does not
manifest today; this clause must be answered first whenever the write
ahead log style form is ever reopened for reconsideration in the future.

Fact 7, the proposed third branch under review, from the project's own
working materials for this round. It adds one new branch to the trigger
condition from fact 1, so that the full three way condition reads: a
checkpoint trigger holds if and only if either the time branch from fact
1, or the dirty byte branch from fact 1, or a new third branch defined as
the number of bytes of the journal ring already occupied within the
current checkpoint interval being greater than or equal to the ring
length divided by the safety factor F. The stated rationale for this new
branch is that it depends on neither the fsync rate, nor bandwidth, nor
how fast the underlying machine is: on a fast machine, occupancy rises
quickly and this new branch fires first; on a slow machine, the time
branch fires first instead. Because of this, the proposal states that
T_time can keep its existing behavior exactly as described in fact 3,
namely user settable, without restriction, producing only a warning when
out of range, with no change needed to that behavior.

Fact 8, the journal form actually chosen for this project today, from its
decision record. It is defined as follows: every single fsync call writes
every currently dirty leaf, all of those leaves' ancestor nodes, the root
slot, and one journal record; ancestor nodes are never deferred to a
later point in time. Dirty leaves here means all dirty leaves anywhere in
the whole filesystem at that moment, not only the leaves belonging to the
one file that was passed to this particular fsync call: in effect, one
fsync call equals one full publish of the entire filesystem's current
state.

Fact 9, a finding from an earlier, separate round of independent review,
concerning a hypothetical write ahead log style form rather than the form
actually chosen and described in fact 8. Using a measured fsync rate of
2785 fsync calls per second, taken from a real machine, together with a
separate figure of 8 four kibibyte journal records written per fsync call
under one particular target workload, that finding computes an interval
occupancy formula equal to the fsync rate times elapsed time times 8 times
4096 bytes; at T_time's own default value of 5 seconds, this computed
occupancy already crosses ring length divided by F, specifically crossing
it at 2.94 seconds when F equals 3, and at 4.41 seconds when F equals 2.
The same finding states that T_dirty's clamp from fact 2 does not rescue
this situation, because under a write ahead log style form the underlying
data becomes durable at the exact moment of the fsync call itself, rather
than sitting around afterward as a dirty page in memory, so the dirty
byte branch from fact 1 never fires at all in that regime. A correction
later accepted about this same finding states: T_time is a duration and
ring length divided by F is a byte quantity, so converting between the
two requires a rate, namely the fsync rate together with how many journal
records each fsync call writes; that rate varies by machine and by
workload, and no clause anywhere in the project either defines what that
rate should be or requires it to ever be measured on any given machine.

Fact 10, a set of direct searches across the project's Rust source code,
carried out for this same round of review. Searching every Rust source
file under the main source tree for the literal text T_time finds exactly
4 matches, every one of them inside one single source file, the module
defining the system's per mount runtime configuration; those 4 matches
are two documentation comments plus one small accessor function that
simply returns a fixed constant. Searching the same tree for the literal
text T_dirty finds exactly 4 matches, again all inside that same one
file, again two documentation comments plus one small accessor function
that simply returns a fixed constant. Searching the same tree for the
literal text checkpoint_trigger finds 0 matches anywhere. One of the two
documentation comments found in that same file states outright, quoted
here in translation: there is no trigger path either, meaning the publish
trigger logic for T_time and T_dirty has not been implemented. Searching
the same tree for several plausible names for a live, per interval
occupancy counter, including terms meaning bytes occupied, bytes since
the last checkpoint, checkpoint interval, and interval bytes, finds 0
matches anywhere in the entire tree. One related constant does exist,
called the journal safety factor, with a fixed value of 3, defined in the
on disk format crate; it is consumed in exactly one place, a small
function that divides the ring's total byte size by the fixed record size
and then by this same safety factor, producing a single number called the
in flight record limit. That number is computed once from the ring's
fixed, unchanging geometry, and is used only to cap how many records the
recovery path will replay after a crash; it is not a live counter that
tracks how many bytes have actually been written since the start of the
current checkpoint interval, and no such live counter was found anywhere
in the searches described in this fact.

Judgment 1. Considering fact 1's exact wording of today's two branch
trigger condition, fact 7's added third branch, and fact 2's statement
that T_dirty and T_time already follow two different rules for a
different purpose, namely how an out of range configured value is
handled. Is there any point, under any combination of parameter values or
any workload, at which two of the three branches in the full three way
condition from fact 7 become true at the exact same instant, while the
concrete action called for by one of those two branches genuinely differs
from the concrete action called for by the other. Give an explicit
verdict; if you find such a point, name which two branches, name exactly
what differs between the two actions each branch calls for, and cite
which facts establish each of the two differing actions; if you find no
such point, name what in the facts above guarantees that the two actions
stay the same in every case you can construct, and cite those facts.

Judgment 2. Considering facts 4, 7, and 9 together. Fact 9 states that
under a write ahead log style form, the dirty byte branch never fires,
because data becomes durable at the exact moment of the fsync call rather
than sitting afterward as a dirty page. Once fact 7's third branch is
added to the trigger condition, does that same claim, that the dirty
byte branch never fires under a write ahead log style form, still hold
exactly as it did before, or does adding the third branch change whether
or how it holds. Separately, does adding the third branch call into
question whether T_dirty's own definition, given in fact 4 as governing a
dirty page memory budget, should itself be changed as a consequence. Give
one explicit verdict that addresses both of these two questions together,
citing the specific facts each part of your verdict rests on.

Judgment 3. Considering fact 2's statement that T_dirty and T_time follow
two different rules, clamping and warning respectively, together with
fact 7's third branch. Does fact 7's third branch fall within the scope
of that same statement from fact 2 at all; and if it does, should the
third branch follow the clamping rule the way T_dirty does, the warning
rule the way T_time does, or neither of those two rules. Give one
explicit verdict choosing clamping, warning, neither, or state your own
distinct explicit verdict if none of those three fit, and cite the facts
it rests on.

Judgment 4. Considering fact 10 alone, together with fact 7's exact
wording of the third branch, which requires reading a value described as
the number of bytes of the journal ring already occupied within the
current checkpoint interval. Do any of the source code searches reported
in fact 10 show that this specific quantity, a live count of bytes
occupied since the start of the current checkpoint interval under any
name whatsoever, already exists anywhere in the source tree today. Give
an explicit verdict of exists today, does not exist today, or cannot be
determined from fact 10 alone; cite exactly which part of fact 10
supports your verdict. Do not evaluate whether this quantity should
exist, where it ought to be computed, or what category of accounting it
belongs to; answer only whether it already exists today, as a plain
question of current fact.
