SINGLEFS FAULT INJECTION SCOPE - LOCAL ATTACK LEG - ROUND m2-supp3-item4-code-r1 - DOCUMENT K6

You are one of three independent reviewers checking a piece of newly written
code for singlefs, a copy on write filesystem being designed from scratch.
You are given a self contained set of facts below. Do not assume anything
not stated here. Read every fact carefully, every qualifier is load
bearing, a definition with one word removed can flip an answer. Do not use
any markdown emphasis anywhere in your answer, no bold text, no italic
text, no backtick code formatting, no asterisk bullets, no pipe tables.
Number your answers to match the three judgments given at the end. For
each judgment give three separate labeled parts: a verdict stated as an
explicit phrase, not a bare yes or no and not merged with another
judgment's verdict; one or two sentences of justification that name the
specific fact numbers your verdict rests on; and one sentence starting
with "This would be refuted by:" describing the specific observation that
would prove that verdict wrong. When your answer needs to point at a
source, point at it only by the fact number given below, fact 1 through
fact 9. Do not write any source file name together with a line number,
and do not write any line number at all anywhere in your answer, because
the underlying files may have moved to different line numbers by the time
anyone reads your answer; if you need to point at a piece of code, name
the function by its name instead.

This is a separate, independent document from another document in this
same review round that asks about a different part of the same code,
called document K3. You are not expected to see that other document and
should not assume anything about its content.

Background. Singlefs is being extended with a general purpose fault
injecting device wrapper, meant to sit between the code under test and a
simulated block device, and to inject faults driven by a random seed
across many generated multi step histories. After each injected fault,
the harness closes the devices and reopens them cold, to check that
recovery lands on a version an in memory reference model would allow.

Fact 1, the requirement this new code is meant to satisfy, from the
project's milestone document. The requirement is written as: build a
general purpose device wrapper; driven by a seed, it must make any one
write attempt, or any one read attempt, or any one flush attempt, return
a value called BlockDeviceError, or it must make one read attempt return
corrupted bytes; the implementation must return an error rather than
panicking; after an error occurs, a fresh cold reopen must recover to a
version the model allows; the hand written wrappers that exist today
inside individual test files must be folded into this one general
purpose wrapper.

Fact 2, the three labels used to classify what a fresh cold reopen lands
on after one injected fault, each with its own documentation comment,
from direct observation of the source. A label called
CommittedByTheModel is documented as: the reopen landed on a version the
model itself recorded as committed, using the set of versions the in
memory model's own root ring held at the moment this particular faulted
run of the history stopped. A label called
TheVersionTheFaultedStepWasWriting is documented as: the model did not
record this version as committed, but it is exactly the version that the
one step which failed would have written had the fault not been
injected; the same comment states this is the exact cell a separately
tracked known defect is arguing over, and states plainly, in these
words, that here it is only tallied, not judged. A label called
OutsideEverythingTheModelAllows is documented as: neither of the above;
a function that compares the model's records against what was actually
read back found a disagreement.

Fact 3, the counters kept for each attempt, from direct observation of
the source. Separate running counts exist for: the total number of cold
reopens performed; reopens that read back a file; reopens that read back
no file; reopens where the read back itself failed; reopens landing on
the CommittedByTheModel label from fact 2; reopens landing on the
TheVersionTheFaultedStepWasWriting label from fact 2, whose own comment
states plainly, in these words, this cell is only tallied, not judged;
and reopens landing on the OutsideEverythingTheModelAllows label from
fact 2. A separate test asserts, as a hard requirement that must hold
today, that the count for OutsideEverythingTheModelAllows must equal
zero. No assertion anywhere in that same test file places any bound at
all on the count for TheVersionTheFaultedStepWasWriting; that count is
printed in a report but nothing today requires it to be any particular
value, small or large, zero or nonzero.

Fact 4, exactly how one single injected fault gets scored, from direct
observation of the source, inside the function that scores one injected
fault. After the fault is injected and the history is replayed once with
that one fault armed, the code builds a permissive set of versions,
built as the union of two sources: first, every version the model
itself recorded as committed during this particular faulted run of the
history, including anything it recorded right at the very end; second,
every version the model would have committed by the exact step where
the fault was injected, taken from a separate, earlier, fault free
measurement run of the very same history, and this second source
specifically includes the version that the one step which failed was in
the middle of writing during that earlier fault free run. The function
that compares the model's records against what was actually read back,
described fully in fact 5, is then called exactly once against this
permissive set, and its result is the only thing that decides whether
this attempt is added to the list of failures needing further report.
That same function is also called a second time, separately, against a
narrower set containing only the first source above, meaning only
versions actually recorded as committed during this faulted run, purely
in order to choose which of the three labels from fact 2 gets recorded
in the tally from fact 3; this second call's result never by itself adds
or removes anything from the list of failures. Separately from both of
these two calls, the on disk image left behind by this same attempt is
independently walked by the pool level checker, and any invariant the
checker finds violated is added to the same list of failures regardless
of what either call to the comparison function decided.

Fact 5, the comparison function used twice in fact 4, from its own
documentation comment. Given a set of versions treated as legitimate, a
record of which root is newest and durably persisted, and what was
actually read back after a reopen, the function checks four things: the
read back must not itself have failed; the root chosen on reopen must be
some version inside the given legitimate set, described in the comment
as guarding against a version that was never recorded appearing on disk;
the chosen root must not be older, by generation number and by instance
number together, than the newest root known to be durably persisted;
and the content read back must match, byte for byte, the content the
legitimate set records for that same version. If the given legitimate
set contains a version that was never actually and honestly committed
during the run being judged, the function has no way to know this and
will treat that version as fully legitimate for all four of these
checks.

Fact 6, a fixed, hand written test built specifically around a
separately tracked known defect, from direct observation of the source
and its own documentation comments. The test's own comments state: this
defect happens when a publish fails on its very last step while an
earlier step in the same publish has already durably written a new
root; the allocator is rolled back as though the publish had not
happened at all, and then the exact same caller retries with the same
prior version, reusing the now rolled back allocator state, and this
retry overwrites the units that the already durably written root still
points to. The test drives exactly this sequence and then asserts, as
today's hard requirement, that the pool level checker reports two
specific invariants, named I-7.4 and I-7.2, as violated, and that a
fresh cold reopen fails outright rather than landing on any root. A
comment directly above this test states, in these words, that once this
known defect is fixed under whichever candidate fix is eventually
chosen, this same test is expected to pass in a different way and this
test's own assertions are expected to need updating, together with a
separately kept list of known, already accepted failures, and states
explicitly that the assertions must not simply be loosened.

Fact 7, measured counts from one full run of the fast, always on
campaign, stated directly in the project's own record of this round of
work. Across 95 total injected faults spread over 24 generated history
segments, cold reopen landed on the CommittedByTheModel label from fact
2 a total of 77 times, on the TheVersionTheFaultedStepWasWriting label a
total of 18 times, and on the OutsideEverythingTheModelAllows label a
total of 0 times; across the same run there were 0 panics, and 0
failures outside a separately kept list of already known, already
accepted failures.

Fact 8, the separately tracked known defect referenced in facts 2 and 6,
from the project's own tracked issue record. The persistent write order
of one publish always finishes with a root record written with a strong
durability mode, and only afterward, as a distinct final step, a small
on disk structure recording which system configuration is current gets
rotated. When that final rotation step fails, the root record has, by
that point, already durably landed, and a subsequent recovery would
select it as legitimate; yet today's publish path unconditionally rolls
the allocator all the way back, exactly as though this publish had never
happened. The tracked record states that three separate rounds of
independent review have now finished on this issue, and states plainly
that the actual fix itself is still awaiting a decision from the
project's own human owner.

Fact 9, one candidate fix for the defect in fact 8, together with its
current status, from the project's own record of the third round of
review just finished on this issue. The candidate does not propose
adding any new binding rule; instead, it proposes explicitly documenting
that after this specific kind of failure, whether the data in question
appears after a later reopen may go either way. That same round's record
states this candidate survived that round of review, but only once two
further additions are made to it: first, an explicit prohibition against
assuming the caller may retry by reusing the exact same in memory
allocator state that was just rolled back; second, a stated scope limit,
restricting this candidate's semantics to only the two specific cells of
a separate, not yet built failure table where the system either switches
to a different running instance or ends up retrying and failing again,
and explicitly excluding cells of that same failure table where the
caller receives no error at all. The same record states plainly that
both of these two additions have so far been checked by only one round
of adversarial review each, and that the candidate as a whole, together
with these two additions, is still awaiting a decision from the
project's own human owner before anything gets changed in the running
code.

Judgment 1. Considering facts 1 through 8, name at least one specific,
concrete class of a genuinely wrong outcome, meaning a case where the
actual sequence of events was not something that fact 1's own
requirement, or the model's own committed history, would sanction, that
could pass all the way through the scoring described in fact 4 without
ever being added to the list of failures, purely because it lands inside
the permissive set described in fact 4, while also never by itself
tripping the pool level checker mentioned at the end of fact 4. Describe
this class concretely enough that someone could construct one example
history that falls into it, rather than describing it only in the
abstract.

Judgment 2. Suppose the candidate fix described in fact 9 is the one the
project's human owner eventually adopts, together with both of the two
additions fact 9 states are needed for it to stand. State specifically
what the permissive set described in fact 4, and the three way
classification described in fact 2, would need to become so that the
scoring in fact 4 matches this adopted candidate's own stated scope
limit from fact 9, meaning it does not simply treat every case that
lands inside today's permissive set as acceptable everywhere, only in
the specific cells fact 9's scope limit actually covers.

Judgment 3. If exactly the change your answer to judgment 2 describes
were made today, and every other part of the current implementation
described anywhere in facts 1 through 9 were left exactly as is, would
the fast, always on campaign whose measured counts are given in fact 7
immediately report a nonzero count somewhere it reports zero today, or a
newly failing case, under today's code exactly as facts 1 through 9
describe it, with no other code changed. Give an explicit verdict of
either yes with a specific counter named from fact 3 or fact 6, or no
with a stated reason drawn from facts 1 through 9.
