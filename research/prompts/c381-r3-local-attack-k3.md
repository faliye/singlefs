SINGLEFS PUBLISH FAILURE LOCATION SAFETY - LOCAL ATTACK LEG - ROUND c381-r3 - DOCUMENT K3

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
fact number given below (fact 1 through fact 11), or by the failure
point label given below (P1 through P8). Do not write any source file
name together with a line number, and do not write any line number at
all anywhere in your answer, because the underlying files may have moved
to different line numbers by the time anyone reads your answer.

This is a separate, independent document from another document in this
same review round that asks about a different candidate policy and a
different set of stages. You are not expected to see that other document
and should not assume anything about its content.

Background. Singlefs groups writes into checkpoints. Each checkpoint is
identified by a strictly increasing counter called txg. Publishing a
checkpoint writes, in a fixed order given in fact 1, ending with a root
record and then a rotation of a small on disk structure called the
system config slot. When a publish attempt fails partway through, the
current decision item's failure table says the response should first
perform a probe write, described in fact 5, to decide whether the fault
is transient or persistent. Today's running code does not implement this
probe write at all, stated as fact 7. This document asks about a
candidate change to where that probe write is targeted, called candidate
B below and given in fact 4.

Fact 1, persistence order, from a decision item about publish semantics.
The persistent write order of one publish is always: copy on write units
or nodes, then a barrier, then the journal record, then a barrier, then
the root slot written with force unit access, then the system config
slot. The system config slot's update happens only once per checkpoint,
strictly after the root slot has persisted, and it is the last step of
the sequence.

Fact 2, how root slot force unit access actually behaves today, and a
separate fact about what a barrier actually is, from a decision item
about verification and from an implementation note. In the current
implementation, the write mode used for the root slot, called force unit
access mode, first performs a plain write of the root record's bytes to
its target device offset using a function called write_all_at, which
issues a plain pwrite and does not itself request a flush, and then
performs one whole device flush. A separate function called barrier
performs only that same whole device flush; it takes no target device
offset and writes no bytes of its own. Both the force unit access mode's
own flush step and the standalone barrier function call the exact same
underlying primitive, a whole device flush, but only force unit access
mode is preceded by an actual write of specific bytes to a specific
offset; the standalone barrier function is only ever the flush, nothing
else. This means: wherever this document below refers to the first
barrier or the second barrier, those two steps have no target device
offset and write no bytes of their own; they are pure synchronization
points. Ordinary writes, meaning the unit writes, the journal record
write, and the system config slot write, each go to one specific device
offset using write_all_at and do not by themselves force a flush.

Fact 3, root slot placement and root ring geometry, from the physical
layout of the first checkpoint plus a decision item about publish
semantics. The root ring has three regions, numbered 0, 1, and 2; region
0 and region 2 are both placed on device 0, and region 1 is placed on
device 1. Each region holds a fixed set of 8 root slots. A checkpoint
with counter txg writes its root record to region number (txg modulo 3),
and to slot number ((txg divided by 3) modulo 8) within that region.
Except for the very first root record written at format time, which is
written once into all three regions at once, every later checkpoint's
root record is written to exactly one region, hence to exactly one of
the two devices; the root slot is not mirrored across both devices for
that checkpoint. A stated known consequence of this, from the same
decision item, is that after an ordinary fsync return, that checkpoint's
generation is covered by only that one root slot, not by both devices.

Fact 4, candidate B, the change under review in this document. Instead
of the existing fixed location probe write given in fact 5, candidate B
changes the probe write so that, when a publish attempt fails, the probe
write goes to the exact device offset that this specific failure's own
write was itself targeting, rather than to a separate fixed location on
the target device. The stated reason for this change is to close a gap
where a fixed location probe write can succeed even when the failed
write's own specific location is the part of the device that is bad,
which would otherwise wrongly classify a location level fault as
transient and trigger three wasted instance switch attempts before
finally going read only anyway. The stated cost
is that only the words naming a fixed location, inside the existing
decision item's wording, would be replaced by wording naming the failed
write's own location; nothing about which write functions exist would
change.

Fact 5, the existing fixed location probe write, from a decision item
about replay, rollback, and failure handling, for contrast with
candidate B. When a publish attempt fails, the current decision item
says to first perform one probe write to a fixed location on the target
device, before deciding whether to treat the failure as transient or
persistent.

Fact 6, today's running code inside the function that performs a
publish, from direct observation of the source. After admission
succeeds, the allocator's current state is cloned into a saved copy
before any write for this publish is attempted. The six on disk writing
steps named in fact 1 run, in order, inside one closure. If any one of
these six steps later returns an error, the allocator is unconditionally
restored from that saved copy, without distinguishing which one of the
six steps the error happened at.

Fact 7, no probe write implementation exists today, from direct
observation of the source. A case insensitive text search across every
source file in the core crate for the terms probe write, read only, and
instance switch, in both their English and Chinese renderings, returns
zero matches. None of these three mechanisms exist anywhere in the
currently running implementation. Because of this, the current source
code does not define, anywhere, at what point in time relative to fact
6's allocator restoration a probe write would actually run if it were
implemented.

Fact 8, block placement happens before the write, from a decision item
about space allocation, referenced directly in the physical layout
document. For a block this publish is about to write, the on disk
location that block will occupy is already decided before that block is
actually written out.

Fact 9, journal ring geometry, from the physical layout of the first
checkpoint. The journal ring is mirrored across both devices: the same
4096 byte record content is written identically to a copy on device 0
and a copy on device 1, at the same offset within the ring on each
device. The ring holds 196608 record sized slots total on each device.
At format time, the entire ring is written full of zeros on both
devices. Replay only ever needs to look at the most recent 65536
records, one third of the ring's total slot count.

Fact 10, system config slot geometry, from the physical layout of the
first checkpoint. Each device carries its own independent pair of two
system config slots, occupying fixed offsets on that device. Successive
generations of the system config on one device alternate which of that
device's two slots receives the new write, so that the other slot
continues to hold the immediately preceding generation's content on that
same device. The two devices' rotations are independent of each other.

Fact 11, the known defect this decision item is meant to fix, from an
outstanding issue tracked separately. Today, when the system config slot
step, the last of the six steps in fact 1, fails after the root slot has
already durably completed, the root record has already durably landed
and a subsequent recovery would select it as valid, yet fact 6 causes
the publish path to unconditionally roll the allocator back as though
this publish had not happened at all.

Failure point definitions. Eight distinct places where a write inside
one ordinary publish attempt can fail, corresponding to the six steps in
fact 1, with the two mirrored steps split by device.

P1: the unit write. The write of copy on write data or index units for
this publish fails partway through, before either barrier is issued.

P2: the first barrier. The barrier issued after the unit writes and
before the journal record write fails.

P3: the journal record write, device 0 copy. Writing this checkpoint's
journal record to its copy on device 0 fails.

P4: the journal record write, device 1 copy. Writing this checkpoint's
journal record to its copy on device 1 fails.

P5: the second barrier. The barrier issued after the journal record
write and before the root slot write fails.

P6: the root slot force unit access write. The write of this
checkpoint's root record fails; it never returns success.

P7: the system config slot write, device 0. Within the system config
slot rotation step, the write to device 0's target slot fails.

P8: the system config slot write, device 1. Within the system config
slot rotation step, the write to device 1's target slot fails.

Questions.

1. For each of the eight failure points P1 through P8, state, based only
on facts 1 through 11 above and the failure point's own definition,
exactly what device offset, or set of device offsets, candidate B's
probe write would target for that failure point, if candidate B were
adopted. If a failure point's own definition, given facts 1 through 11,
does not correspond to any write of specific bytes to any specific
device offset, state this explicitly and explain why, citing the fact
number that establishes this, rather than naming a location the facts do
not support.

2. For each of the same eight failure points where question 1 found one
or more specific device offsets, state what that span of bytes on disk
most likely already held immediately before this failed write attempt
began, based only on facts 1 through 11. Where the given facts leave
this genuinely undetermined, state this explicitly and state what
additional fact, not given above, would be needed to determine it. For
any failure point where question 1 found no specific device offset,
answer not applicable for this question and say so.

3. For each of the eight failure points P1 through P8, state whether
performing candidate B's probe write at this failure point, immediately
after this failure, would be safe, meaning it would not destroy any data
that some other still valid part of the system currently depends on, or
unsafe, meaning it could destroy such data, or state that the facts
given do not decide this either way. Give one sentence of justification
for every verdict, citing the specific fact number or numbers your
verdict rests on. Add one sentence starting with "This would be refuted
by:" for every verdict, describing a specific observation that would
overturn it.
