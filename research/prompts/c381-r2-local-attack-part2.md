SINGLEFS PUBLISH FAILURE PLACEMENT - LOCAL ATTACK LEG - ROUND c381-r2

You are one of three independent reviewers checking a design question for
singlefs, a copy on write filesystem being designed from scratch. You are
given a self contained set of facts below. Do not assume anything not
stated here. Read carefully, every qualifier in this text is load bearing,
a definition with one word removed can flip an answer. Do not use any
markdown emphasis anywhere in your answer, no bold text, no italic text,
no backtick code formatting, no asterisk bullets. Number your answers to
match the question numbers given at the end. For every numbered answer you
give, add one sentence starting with "This would be refuted by:"
describing the specific observation that would prove that answer wrong.
Where a question asks you to fill in a table, give the table with an
explicit value and an explicit one sentence justification for every single
cell, not just "yes" or "no" for the whole table, and do not merge cells
together even where you believe several rows or columns behave
identically, each cell must carry its own explicit verdict and its own
quoted or explained justification. When your answer needs to point at a
source, point at it by its fact number given below, by the point or stage
label given below (P1 through P8, T1 through T4), by the arm name (arm
STOP or arm TABLE), or by naming the row and column of the table you are
answering. Do not write any source file name together with a line number,
and do not write any line number at all, anywhere in your answer, because
the underlying files may have moved to different line numbers by the time
anyone reads your answer.

This document is part 2 of a 3 part series that all share the exact same
background facts below, given verbatim in every part. The other two
documents ask about different tables and are independent of this one;
you are not expected to see them and should not assume anything about
their content. This document only asks about the second half of table 1,
covering failure points P5 through P8.

Background.

Singlefs groups writes into checkpoints. Each checkpoint is identified by
a strictly increasing counter called txg. Publishing a checkpoint, called
a publish below, writes, in a fixed order described in fact 1 below,
ending with a root record into a fixed size ring of root slots kept on
every device, and finally a rotation of a separate small on disk structure
called the system config slot. A call to fsync that triggers a publish
does not return to its caller until the root slot write for that publish
has durably completed, subject to one more condition given in fact 2.
After a crash, a recovery procedure reads the root ring, picks one root
record to trust, called the selected root, and then decides which journal
records, if any, on top of that selected root should be treated as having
taken effect, a step called replay. Separately, a component called the
allocator tracks which storage blocks are free to hand out. A publish
attempt can fail partway through, and one open design question is what
state the allocator and the write path should be left in afterward, and
whether the code should perform a step called an instance switch, defined
in fact 10, when that happens. A separate decision item states a rule,
given in facts 4 through 9 below and called the failure table below, for
when a switch should happen instead of simply going read only. Today's
running code does not implement a switch, does not implement the probe
write named in the failure table, and does not implement going read only
as a response to a publish failure; this is stated as fact 14.

Fact 1, persistence order and fsync return, from a decision item about
publish semantics. The persistent write order of one publish is always:
copy on write units or nodes, then a barrier, then the journal record,
then a barrier, then the root slot written with force unit access, then
the system config slot. The fsync call that triggers this publish does
not return until the root slot write is persisted. During recovery
replay, before applying any record, every unit that record names must
have its checksum individually verified. The system config slot's update
happens only after the root slot is persisted, once per checkpoint; it is
the last step of the publish sequence, and because of this the sequence
of segments within one publish is unique.

Fact 2, an added fsync return condition, from the same decision item as
fact 1. The fsync return condition has one more clause: until a root
written under a new instance covers two devices, fsync does not return,
and does not confirm a rollback to the administrator. How that coverage
is achieved, and how many publishes it takes, is defined elsewhere in
this decision item and is not needed for this task.

Fact 3, integral application, from a decision item about publish
semantics. For the question of what applying one journal record does at
the pointer layer, the choice taken between two options is that one
publish is applied as a whole, or not applied at all; there is no partial
application of a single publish.

Fact 4, the core rule of the failure table, from a decision item about
replay, rollback, and failure handling. The handling of a failure splits
into two branches according to the nature of the failure. The
discriminant is a quantity that can be determined right at that moment,
not a count of occurrences: when a failure occurs, first perform one
probe write to a fixed location on the target device. If that probe write
succeeds, the failure is classified as transient, and the response is an
instance switch. If that probe write fails, the failure is classified as
persistent, and the response is going read only until the next mount.

Fact 5, additional read only triggers, from the same decision item as
fact 4. If the count of writable devices falls below a floor called w, or
if the switch's own reservation cannot be obtained, the response also
goes straight to the read only branch. If the count of consecutive
switches exceeds a named tunable parameter called N_switch, set to 3, the
response also goes read only; this rule is described as a backstop of the
backstop, and the real discriminant remains the probe write.

Fact 6, why this process converges, from the same decision item as fact
4. This process is guaranteed to converge because the switch's own first
step is itself a write that can fail: a switch must first take a new
instance number, and taking that number requires writing to every system
config copy that this mount exclusively opened successfully, all or
nothing. Because of this, a fault of the kind where a write cannot go
through lets a switch advance at most one further step: the probe write
catches it first, and even if a switch happens to get past the probe
write, the all or nothing number taking write immediately judges it a
failure.

Fact 7, reservation scope, from the same decision item as fact 4. The
reservation guards against the allocation failure path, not the write
failure path. Write failure is guarded against by the probe write and by
the number taking write's all or nothing rule.

Fact 8, the allocation failure split, from the same decision item as fact
4. Allocation failure, as distinct from write failure, splits into three
branches depending on where it happens. A foreground transaction's user
data allocation failure returns an out of space error to the caller, and
the checkpoint still publishes as normal without changing mount state. A
background, resumable, non decision path allocation failure, such as
rebalancing, tombstone reclamation, scrubbing, or defragmentation, pauses
that activity and records it as an intent, without changing mount state.
Only when the switch's own reservation cannot be obtained does the
response go read only.

Fact 9, the root slot resend clause, from the same decision item as fact
4. When a root slot write fails and the publish is resent, the
checkpoint's txg number is advanced by one before resending. The root
ring region that a given txg number maps to is computed as txg modulo the
ring's region count R. Without advancing the txg number before
resending, a resend would repeatedly rewrite the exact same slot,
collapsing R independent failure domains into one.

Fact 10, the definition of an instance switch, from the same decision
item as fact 4. An instance switch is a recovery performed within the
current mount: it takes a new instance number, writes a row, and resends
the in flight checkpoint. Transactions numbered less than or equal to a
value called W are kept as they were, since every transaction already
completed in the open checkpoint has a number greater than W. Transactions
numbered greater than W are either redone under the new write order, or
an error is reported to whichever caller of those transactions has not
yet received a return. Every fixed point unit is rewritten under the new
instance. This response does not go read only and does not wait for the
next mount. Its cost is one row, plus rewriting the entire instance table
chain, plus rewriting the data units of transactions numbered greater
than W, plus one fixed point redo; how often this happens has not been
measured.

Fact 11, the number taking write sequence, from a decision item about
what is carried inside blocks. Taking a new instance number computes the
new number as one more than the maximum instance number found among this
mount's exclusively opened set, then writes that number into every system
config copy in that set, all or nothing. Writing a system config copy
follows the same orientation as writing a data unit: one input output
error must be retried until a retry limit is exhausted before it counts
as a failure. Once that limit is exhausted and the all or nothing rule
judges a failure, the copies already written are first rolled back to the
old instance number; only if that rollback itself cannot succeed does the
mount fall back to read only. The barrier that comes right after number
taking is judged by the same all or nothing rule if it reports an error
on any device; simply resending just that barrier and continuing is not
allowed.

Fact 12, scope note on the number taking function's own rollback. A
comment attached to the function that takes a new instance number and
performs warm up states, in translation: rollback here only covers write
errors from the number taking writes themselves, it does not cover a
publish failure that happens after the number has already been taken.

Fact 13, today's running code inside the function that performs a
publish. After admission succeeds, the allocator's current state is
cloned into a saved copy before any write for this publish is attempted.
If the inner write step later returns any error, the allocator is
unconditionally restored from that saved copy, regardless of which one of
the on disk writing steps the error happened at. The on disk writing
steps run, in order, inside one closure: writing the changed units, a
barrier, writing the journal record, a second barrier, writing the root
slot with force unit access, and rotating the system config slot. If any
one of these steps returns an error, this attempt is recorded into a
failed publish accounting bucket, and the original device error is handed
back to the caller unchanged. The caller does not receive this attempt's
resulting version object.

Fact 14, no switch implementation exists today. A case insensitive text
search across every source file in the core crate for the terms probe
write, read only, and instance switch, in both their English and Chinese
renderings, returns zero matches. None of the three things named in the
failure table, the probe write, the instance switch, and going read only,
exist anywhere in the currently running implementation.

Fact 15, how root slot force unit access actually behaves today. The root
slot force unit access step, in the current implementation, is not a true
per write force unit access. In both existing block device
implementations, the force unit access write mode and the barrier
operation call the exact same whole device flush primitive, meaning
write, then flush the entire device once more, which is stronger than a
true force unit access, since a true force unit access only guarantees
durability for that one write itself. Wherever a candidate arm's
definition below refers to after root slot force unit access succeeds, it
should be read under this meaning: the root slot has been written and the
entire device has been flushed, so every write that completed earlier
than that point has also become durable at that same moment.

Failure point definitions. Eight distinct places where a write inside one
publish attempt, or inside one instance switch, or inside one ordinary
mount's number taking, can fail. These are the rows you will judge in
table 1 and table 2.

P1: the unit write. The write of copy on write data or index units for
this publish fails partway through.

P2: the first barrier. The barrier that comes after the unit writes and
before the journal record write fails.

P3: the journal record write. The write of the journal record describing
this checkpoint, to its two on disk copies, fails; zero, one, or both
copies may actually have persisted.

P4: the second barrier. The barrier that comes after the journal record
write and before the root slot write fails.

P5: root slot force unit access. The root slot write itself fails,
meaning it never returns success.

P6: the system config slot. The rotation of the system config slot, which
happens once per checkpoint after the root slot has persisted, fails on
one or both of its on disk copies.

P7: number taking during an ordinary mount. One of the two system config
slot writes used to take a new instance number, performed before that
mount's own row writing publish, fails.

P8: number taking and row writing performed by a switch itself. During an
instance switch as defined in fact 10, the switch's own act of taking a
new instance number, or the switch's own act of writing a row, itself
fails.

Stage definitions. Every publish attempt is analyzed, at a coarser grain
than the eight failure points above, as failing at exactly one of four
stages. These are used only in the two candidate arm definitions below and
in table 3.

T1: the unit write is only partway done, and not a single copy of the
journal record has been sent yet.

T2: the journal record has already been sent, but the root slot force
unit access write has not yet returned success. At this point, zero, one,
or two copies of the journal record may actually have persisted to the
two devices.

T3: the root slot force unit access write has already succeeded, and the
failure happens during the system config slot rotation step that comes
right after it.

T4: the failure happens during one of the system config slot writes used
to take a new instance number, inside the function that takes a new
number and performs warm up, before that mount's own row writing publish.

Arm definitions. Two candidate policies for what happens to the allocator
and the write path after a publish attempt fails, each given as one
prescribed action per stage T1 through T4. A third candidate exists in
this round but is out of scope for this task and is not given here.

Arm STOP, stop on failure, this is today's running code as far as
allocator rollback goes, but today's code does not implement anything
beyond stage T4's own rollback; see fact 13 and fact 14. At stage T1: the
allocator rolls back to its pre attempt state, the write path may keep
publishing as before, and, per fact 13, the call whose own write failed
at this stage receives the original device error. At stage T2: the
publish call that failed reports an error to its caller, the write path
becomes void, no further publish is allowed for the rest of this mount,
the caller must remount, and recovery alone decides which version ends up
being the one on disk. At stage T3: same handling as arm STOP's stage T2.
At stage T4: taking the new instance number rolls back on its own write
errors; this is today's running code, per fact 12.

Arm TABLE, follow the failure table. At stage T1: follow the failure
table, facts 4 through 6, 9, and 10: first perform a probe write to a
fixed location on the target device; if it succeeds, go to an instance
switch as defined in fact 10; if it fails, go read only until the next
mount; if the count of consecutive switches exceeds N_switch, also go
read only. Whether this first stage, T1, is even governed by this table
at all, is itself an open question, addressed separately by table 1's
own per row verdicts, not settled by this arm definition. At stage T2:
same handling as arm TABLE's stage T1. At stage T3: same handling as arm
TABLE's stage T1. At stage T4: same handling as arm STOP's stage T4.

Task, table 1, part 2 of 2. For each of the four failure points P5
through P8, and for each of six clauses named C1 through C6 below,
decide whether, based only on the wording and context of facts 4 through
10 above, that failure point falls inside the scope of that clause,
falls outside the scope of that clause, or the text simply does not
decide the question either way. Clause C1 is fact 4, the core two branch
rule. Clause C2 is fact 5, the additional read only triggers and the
statement that the probe write is the real discriminant. Clause C3 is
fact 6, the convergence explanation. Clause C4 is facts 7 and 8
together, the boundary between allocation failure and write failure.
Clause C5 is fact 9, the root slot resend clause. Clause C6 is fact 10,
the definition of an instance switch. This produces twenty four cells in
total, four failure points times six clauses. For every single cell,
state your verdict as one of exactly these three phrases: inside scope,
outside scope, or text does not decide, and then give one sentence of
justification. If your verdict is inside scope or outside scope, that
sentence must quote, word for word, the exact phrase from the named fact
that your verdict rests on. If your verdict is text does not decide,
that sentence must state in your own words what specific piece of
information the text would need to contain, but does not contain, for
you to decide. Do not skip any of the twenty four cells. Organize this
table by failure point, then by clause within each failure point, so the
output has four blocks, one per failure point, each block containing six
clause verdicts. A separate, independent document covers the other four
failure points, P1 through P4, against these same six clauses; you are
not asked about those points in this document.

Questions.

1. Give the table described above in full, covering failure points P5
through P8 and all six clauses C1 through C6, following the instructions
for that table above.

2. Fact 5 states that exceeding N_switch also goes read only, calling
this rule a backstop of the backstop, and states that the real
discriminant remains the probe write. For failure point P8, the switch's
own number taking and row writing failing again during a switch, does
fact 5's N_switch clause, by itself, already force a definite verdict
for whether P8 falls inside the scope of clause C1, or does your verdict
for the P8 row, C1 column, in the table you gave above, still depend on
something beyond fact 5? Name exactly what that something is, if there
is one. Add the required "This would be refuted by:" sentence.
