SINGLEFS PUBLISH FAILURE CALLER SEMANTICS - LOCAL ATTACK LEG - ROUND c381-r3 - DOCUMENT K1

You are one of three independent reviewers checking a design question for
singlefs, a copy on write filesystem being designed from scratch. You are
given a self contained set of facts below. Do not assume anything not
stated here. Read every fact carefully, every qualifier is load bearing,
a definition with one word removed can flip an answer. Do not use any
markdown emphasis anywhere in your answer, no bold text, no italic text,
no backtick code formatting, no asterisk bullets. Number your answers to
match the question numbers given at the end. For every verdict you give
inside a table, add one sentence starting with "This would be refuted
by:" describing the specific observation that would prove that verdict
wrong. Give an explicit value and an explicit justification for every
single row, not just yes or no for the whole table, and do not merge
rows together even where you believe several of them behave identically;
each row must carry its own explicit verdict and its own justification.
When your answer needs to point at a source, point at it only by its
fact number given below (fact 1 through fact 8), or by the stage label
given below (T1 through T4). Do not write any source file name together
with a line number, and do not write any line number at all anywhere in
your answer, because the underlying files may have moved to different
line numbers by the time anyone reads your answer.

This is a separate, independent document from another document in this
same review round that asks about a different candidate policy and a
different set of failure points. You are not expected to see that other
document and should not assume anything about its content.

Background. Singlefs groups writes into checkpoints. Each checkpoint is
identified by a strictly increasing counter called txg. Publishing a
checkpoint, called a publish below, writes, in a fixed order given in
fact 1, ending with a root record and then a rotation of a small on disk
structure called the system config slot. A call to fsync that triggers a
publish does not return to its caller until the root slot write for that
publish has durably completed. Separately, an ordinary mount, before it
performs its own first publish, must first take a new instance number by
writing that number into system config slots, described in fact 6. This
document asks about a candidate policy, called candidate A below and
given in fact 8, for what a caller should be told after any of these
writes fails partway through.

Fact 1, persistence order, from a decision item about publish semantics.
The persistent write order of one publish is always: copy on write units
or nodes, then a barrier, then the journal record, then a barrier, then
the root slot written with force unit access, then the system config
slot. The system config slot's update happens only once per checkpoint,
strictly after the root slot has persisted, and it is the last step of
the sequence.

Fact 2, how root slot force unit access actually behaves today, from a
decision item about verification. In the current implementation, the
root slot write mode, called force unit access mode, first writes the
root record's bytes to their target device offset, and then performs one
whole device flush. This whole device flush is stronger than a true
per write force unit access, because a true force unit access only
guarantees durability for that one write itself, while a whole device
flush also makes durable every other write that had already completed
earlier, on that device, before that point. Wherever this document below
refers to after root slot force unit access succeeds, read it under this
meaning: the root record has been written and the entire device has been
flushed, so every earlier write on that device has also become durable
at that same moment.

Fact 3, today's running code inside the function that performs a
publish, from direct observation of the source. After admission
succeeds, the allocator's current state is cloned into a saved copy
before any write for this publish is attempted. The six on disk writing
steps named in fact 1 run, in order, inside one closure. If any one of
these six steps later returns an error, the allocator is unconditionally
restored from that saved copy, without distinguishing which one of the
six steps the error happened at, and this closure returns that error.

Fact 4, no probe write, instance switch, or read only implementation
exists today, from direct observation of the source. A case insensitive
text search across every source file in the core crate for the terms
probe write, read only, and instance switch, in both their English and
Chinese renderings, returns zero matches. None of these three mechanisms
exist anywhere in the currently running implementation.

Fact 5, today's mount level state, from direct observation of the
source. The structure that represents a completed mount today carries
exactly three fields, named output, allocator, and current; there is no
field anywhere in it that could carry a
marker meaning this mount has been degraded by an earlier failure. The
error type returned when a mount attempt itself fails today has no
variant meaning this mount has already gone read only.

Fact 6, taking a new instance number, from a decision item about replay,
rollback, and failure handling, and from the physical layout of the
first checkpoint. Taking a new instance number writes that number into
every system config copy that this mount has exclusively opened
successfully, all or nothing, and this must succeed before a switch, or
before an ordinary mount's own first publish that writes an instance
table row, is allowed to proceed. For an ordinary mount's very first
writable mount attempt on a filesystem, specifically, taking this new
instance number is the first write this mount performs; it happens
before this mount writes even a single copy on write unit.

Fact 7, the known defect this decision item is meant to fix, from an
outstanding issue tracked separately. Today, when the system config slot
step, the last of the six steps in fact 1, fails after the root slot has
already durably completed, the root record has already durably landed
and a subsequent recovery would select it as valid, yet fact 3 causes
the publish path to unconditionally roll the allocator back as though
this publish had not happened at all.

Fact 8, candidate A, the policy under review in this document. Candidate
A does not turn this into a rule at all. Instead, it writes down
explicitly that after an fsync call reports an error, the data this
publish attempt was writing can end up appearing on disk, or can end up
not appearing on disk at all; either outcome is accepted under this
policy. The stated reason given for this is that once the root slot's
force unit access write has durably completed, per fact 2, this publish
has already taken effect on disk; making the caller's answer always
match the eventual disk outcome would require the write path to
guarantee it never fails at its very last step, which cannot be
guaranteed. Given that, rather than writing down a rule that cannot
actually be kept, this policy writes down this exact semantics instead,
so that the caller knows that after receiving an error, it must go read
the actual on disk state for itself to find out what happened.

Stage definitions. Every publish attempt, or every ordinary mount's
number taking step, is analyzed as failing at exactly one of four
stages.

T1: the unit write is only partway done, and not a single copy of the
journal record has been sent yet.

T2: the journal record has already been sent, but the root slot force
unit access write has not yet returned success. At this point, zero,
one, or two copies of the journal record may actually have persisted to
the two devices.

T3: the root slot force unit access write has already succeeded, and the
failure happens during the system config slot rotation step that comes
right after it.

T4: the failure happens during one of the system config slot writes used
to take a new instance number, described in fact 6, before this mount's
own first publish that writes an instance table row.

Questions.

1. For each of the four stages T1 through T4, state, based only on facts
1 through 7 above and the stage's own definition, what the caller
actually receives as the outcome of its own call, according to today's
running code. For T1 through T3, this caller is whoever made the fsync
call that triggered this publish attempt; for T4, this caller is
whoever initiated the mount attempt that is performing this number
taking step. Where the given facts do not specify this for a stage,
state this explicitly and state what additional fact, not given above,
would be needed to determine it.

2. For each of the same four stages, state what is actually durably on
disk at that point, specifically whether this publish attempt's data,
or for T4 this mount's number taking write, has taken effect in the
sense that a subsequent normal mount or recovery would treat it as part
of the filesystem's committed state. Give one sentence of justification
citing the specific fact number your answer rests on.

3. For each of the same four stages, state what the caller identified in
question 1 should do next, under candidate A's stated semantics given in
fact 8, and state whether candidate A's blanket description, that the
data can appear or can fail to appear, is, for this specific stage,
consistent with your own answers to questions 1 and 2 above, or whether
it either overstates or understates the actual uncertainty that exists
at that specific stage. Give one sentence of justification for every
verdict, citing the specific fact number or numbers it rests on. Add one
sentence starting with "This would be refuted by:" for every verdict,
describing a specific observation that would overturn it.
