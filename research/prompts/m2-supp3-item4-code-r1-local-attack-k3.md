SINGLEFS FAULT INJECTION SCOPE - LOCAL ATTACK LEG - ROUND m2-supp3-item4-code-r1 - DOCUMENT K3

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
fact 10. Do not write any source file name together with a line number,
and do not write any line number at all anywhere in your answer, because
the underlying files may have moved to different line numbers by the time
anyone reads your answer; if you need to point at a piece of code, name
the function by its name instead.

This is a separate, independent document from another document in this
same review round that asks about a different part of the same code,
called document K6. You are not expected to see that other document and
should not assume anything about its content.

Background. Singlefs is being extended with a general purpose fault
injecting device wrapper, meant to sit between the code under test and a
simulated block device, and to inject faults driven by a random seed
across many generated multi step histories, so that failure handling can
be checked automatically rather than only in a few hand written test
scenarios.

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

Fact 2, the six kinds of fault the wrapper actually implements today,
each with its own documentation comment, from direct observation of the
source. A kind called WriteFails is documented as: this write reports a
block device error, not one byte reaches the device. A kind called
WriteIsSwallowed is documented as: this write reports success, but in
fact not one byte reaches the device; this models the case where a write
went into the device's cache and then power was lost. A kind called
ReadFails is documented as: this read reports a block device error, the
caller's buffer is left untouched. A kind called
ReadReturnsCorruptedBytes is documented as: this read reports success,
but one bit inside the bytes handed back to the caller has been flipped.
A kind called BarrierFails is documented as: this flush reports a block
device error. A kind called BarrierIsSwallowed is documented as: this
flush reports success, but in fact it was never sent to the device; this
models a single missing flush.

Fact 3, what each of the six kinds actually does to the underlying
simulated device and to the caller, from direct observation of the
source code inside the wrapper's read, write, and flush functions. When
WriteFails is chosen for a given write call, the wrapper returns the
block device error immediately and never calls the wrapped device's own
write function for that call, so those bytes never reach the wrapped
device at all. When BarrierFails is chosen for a given flush call, the
wrapper returns the block device error immediately and never calls the
wrapped device's own flush function for that call. When
ReadReturnsCorruptedBytes is chosen for a given read call, the wrapper
first calls the wrapped device's own real read function so the caller's
buffer is filled with the true bytes, and only afterward flips one bit
inside that buffer before returning success; it never writes anything
back to the wrapped device, so the bytes actually stored on the wrapped
device are never altered by this fault. When WriteIsSwallowed is chosen,
the wrapper returns success immediately and never calls the wrapped
device's own write function for that call, so the bytes never reach the
wrapped device either, exactly as with WriteFails, but unlike
WriteFails the caller is told the write succeeded. When
BarrierIsSwallowed is chosen, the wrapper returns success immediately
and never calls the wrapped device's own flush function for that call,
but unlike BarrierFails the caller is told the flush succeeded.

Fact 4, which calls count as truly reaching the device, from a
documentation comment inside the module that runs generated multi step
histories. A write is included in the recorded stream of operations used
to rebuild a device image after a run only when the wrapper actually
calls the wrapped device's own write function for that specific call.
The comment states explicitly that an errored write, a swallowed write,
and a swallowed flush are all excluded from this recorded stream, and
that because of this exclusion the recorded stream is always equal to
the sequence of operations that truly reached the device.

Fact 5, which of the six kinds the random sampling engine actually
draws, from direct observation of the source and its own documentation
comment. A function that draws one fault kind for a given point in a
generated history picks uniformly among exactly four of the six kinds
listed in fact 2: WriteFails, ReadFails, ReadReturnsCorruptedBytes, and
BarrierFails. WriteIsSwallowed and BarrierIsSwallowed are never returned
by this function; there is no code path anywhere in the random sampling
engine by which either of these two kinds could be chosen for a
randomly generated history. The function's own documentation comment
gives a reason for this exclusion: it calls these two kinds the device
lying, meaning the device reports success while doing nothing, which the
comment treats as different in kind from returning an error. The same
comment states that a silently swallowed write drops content on disk
with no filesystem able to detect it, because the call site that issued
the write receives a success result and the checksum was computed by the
implementation itself, so only a later read would reveal the loss; the
comment states that using this kind to judge whether the implementation
returns an error rather than panicking would be judging something the
implementation cannot achieve. The same comment states that both
excluded kinds are kept inside the wrapper for other stated purposes: a
swallowed flush is used elsewhere to measure what extra crash states
appear from one missing flush on a real device, and a swallowed write is
stated as intended to be paired with a separate
crash injection mechanism.

Fact 6, a measured claim written inside that same documentation comment
from fact 5. The comment states that when WriteIsSwallowed was drawn on
a fast running campaign of 24 generated history segments, in 2 of those
24 segments the pool level checker reported three specific invariants as
violated, with a root pointing at a unit that was never durably written.
The three invariants are named I-2.1, I-4.8, and I-7.4, defined in fact
7 below.

Fact 7, the three invariants named in fact 6, from the project's
invariant registry. I-2.1 is defined as: for any block that is
referenced, its checksum must match its content. I-4.8 is defined as:
after any crash point replay, walking from any root inside a specific
rollback candidate set, every block's checksum must agree with what its
parent pointer records; the registry states this invariant's intended
discriminating power is to catch the case where a block released during
the current transaction gets reallocated and overwritten, which would
make an older root a fraudulent rollback candidate, and that checking
only the newest generation of root would never catch this. I-7.4 is
defined as: for every root inside that same rollback candidate set,
every block that root references must not have had its physical range
reallocated to any other object, and must not have been erased by a
sweep.

Fact 8, the milestone's own acceptance criterion for this fault
injection work, from the project's milestone document. It states: each
publish step and each mount step must have been injected at least once,
the number of times each injection point was actually hit must be
reported, and any injection point never hit must be individually named;
as a test of discriminating power, a specific known defect, tracked
elsewhere and referenced only by its tracking name, must make this fault
injection campaign report a failure before that defect is fixed, and
must make it report no failure after that defect is fixed.

Fact 9, what exercises WriteIsSwallowed and BarrierIsSwallowed today
outside the random sampling engine, from direct observation of the
source. Inside the wrapper's own module, two small hand written unit
tests each arm the wrapper with exactly one of these two kinds and then
make exactly one call. One test asserts that after a write with
WriteIsSwallowed armed, reading the same location back yields the
original untouched bytes, and that this device's count of real writes
stayed at zero. The other test asserts that after a flush with
BarrierIsSwallowed armed, the device's count of swallowed flushes
increased by one while its count of forwarded flushes did not. Neither
test runs a generated multi step history, a pool level checker, or a
simulated reopen; each test only checks the wrapper's own bookkeeping
for one single isolated call.

Fact 10, a search across the source tree, from direct observation. A
case insensitive search across the module that performs crash point
injection, meaning deciding where inside an already recorded history to
cut off and rebuild an image, and across that module's own test file,
for either of the two names WriteIsSwallowed or BarrierIsSwallowed,
returns zero matches in both files. A separate search across the entire
source tree for these same two names returns matches in exactly two
files: the wrapper's own module described in facts 2, 3, 5, and 9, and
one binary used for measurements against a real device, where only
BarrierIsSwallowed is used, for the single narrow purpose of measuring
what happens when one specific barrier is skipped during one specific
scripted scenario; that binary never uses WriteIsSwallowed at all, and
it never draws faults randomly, its fault is armed at one fixed, hand
chosen point in a fixed scripted sequence.

Judgment 1. Considering fact 1's exact wording together with facts 2
through 6, is a write, read, or flush call that reports success while
the corresponding bytes never durably reach the wrapped device, meaning
WriteIsSwallowed or BarrierIsSwallowed as documented in fact 2 and
confirmed in fact 3, a genuine gap in what this fault injection
requirement is meant to exercise, or is it something the requirement in
fact 1 does not call for at all, or is it already adequately covered by
something else described in the facts above. Give one of these three
explicit verdicts, or state your own distinct explicit verdict if none
of these three fit, but do not answer only yes or no.

Judgment 2. Considering facts 5, 6, 9, and 10 together, should
WriteIsSwallowed and BarrierIsSwallowed be added to the random sampling
engine described in fact 5, so that a randomly generated history could
draw either of them the same way it draws the four kinds it already
draws, or should they be left exactly as they are today, reachable only
through the two narrow unit tests in fact 9 and the one real device
measurement binary in fact 10. Give an explicit verdict, and state
whether your verdict depends on first making some other change described
or implied anywhere in facts 1 through 10, naming that change.

Judgment 3. If WriteIsSwallowed and BarrierIsSwallowed are left exactly
as they are today, not added to the random sampling engine, state what,
if anything, inside facts 1 through 10 actually exercises this class of
fault, meaning a write or flush call that reports success while the
corresponding bytes never durably reach the wrapped device, across a
range of many different randomly generated multi step histories, as
opposed to exercising it only through one isolated hand written call as
in fact 9, or only through one fixed hand scripted scenario as in fact
10. If your answer is that nothing today does this across a range of
randomly generated histories, say so explicitly and name which specific
fact establishes that gap.
