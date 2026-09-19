SINGLEFS PUBLISH FAILURE PLACEMENT - LOCAL ATTACK LEG - ROUND c381-r1

You are one of three independent reviewers checking a design question for
singlefs, a copy on write filesystem being designed from scratch. You are
given a self contained set of facts below. Do not assume anything not
stated here. Read carefully, every qualifier in this text is load bearing,
a definition with one word removed can flip an answer. Do not use any
markdown emphasis anywhere in your answer, no bold text, no italic text, no
backtick code formatting, no asterisk bullets. Number your answers to match
the question numbers given at the end. For every numbered answer you give,
add one sentence starting with "This would be refuted by:" describing the
specific observation that would prove that answer wrong. Where a question
asks you to fill in a table, give the table with an explicit value and an
explicit one sentence justification for every single cell, not just "yes"
or "no" for the whole table. When your answer needs to point at a source,
point at it by its fact number given below, or by the function name given
in a fact, or by naming the row and column of the table you are answering
(for example: table 2, arm B, stage S2, clause R). Do not write any source
file name together with a line number, and do not write any line number at
all, anywhere in your answer, because the underlying files may have moved
to different line numbers by the time anyone reads your answer.

Background.

Singlefs groups writes into checkpoints. Each checkpoint is identified by
a strictly increasing counter called txg. Publishing a checkpoint, called
a publish below, writes, in this fixed order: first the copy on write data
and index units changed by this checkpoint, then a barrier, then a journal
record describing this checkpoint, then a second barrier, then a root
record into a fixed size ring of root slots kept on every device, using a
write mode called force unit access which guarantees the write is durable
the instant it returns, and finally, once the root slot write succeeds, a
rotation of a separate small on disk structure called the superblock slot.
A call to fsync that triggers a publish does not return to its caller
until the root slot write for that publish has durably completed. After a
crash, a recovery procedure reads the root ring, picks one root record to
trust, called the selected root, and then decides which journal records,
if any, on top of that selected root should be treated as having taken
effect, a step called replay. Separately, a component called the
allocator tracks which storage blocks are free to hand out; a publish
attempt can fail partway through, and one open design question is what
state the allocator and the write path should be left in afterward.

Fact 1, persistence order and fsync return, from a decision item about
publish semantics. The persistent write order of one publish is always:
copy on write units or nodes, then a barrier, then the journal record,
then a barrier, then the root slot written with force unit access. The
fsync call that triggers this publish does not return until the root slot
write is persisted. During recovery replay, before applying any record,
every unit that record names must have its checksum individually
verified.

Fact 2, scope note attached to fact 1. After fsync returns, that
generation is covered by exactly one root slot, because root slots are
not mirrored across devices. Within the same instance, losing one
generation can still be recovered by journal replay. But for every new
instance, meaning every writable mount, every recovery, every switch, and
every rollback, that instance's first root is a single point of failure
until the next publish happens: if that one slot cannot be read, or the
device holding it is lost, recovery falls back to the previous instance's
root, and replay, following the strict prefix rule, stops at the
instance boundary. Every transaction in this mount for
which fsync had already returned is then lost. One single fault is
enough to cause this.

Fact 3, new rule 2, from the same decision. Every block freed during a
checkpoint, call it checkpoint C, must not enter the allocatable set
before checkpoint C is published. Two reasons are given. First, rollback:
after a crash, recovery may fall back to the root belonging to checkpoint
C minus one, and a block that was born before checkpoint C and died
during checkpoint C is still alive as of that earlier root. Second, and
specific to this design: this filesystem treats any prefix of the journal
applied on top of a root as a legal intermediate state. If a block is
released at journal position p and then reallocated and written with new
content at a later journal position q, then replaying only the prefix up
to some position r strictly between p and q reconstructs a tree that
still points at that block's address, while the bytes actually on disk at
that address are already the new content written at position q, because
a physical write is not constrained by journal position. This second
problem is stated as not a theoretical risk but a guaranteed error.

Fact 4, the k_tol tolerance, from a decision item about the rollback
floor. One admission attempt may drive at most B publishes, where B
equals four plus two times k_tol, and k_tol is fixed at two, giving B
equal to eight; only once all of them are exhausted and space is still
insufficient does the filesystem report an out of space error. Separately
and independently stated in the same item: the guarantee of keeping at
least four rollback candidate states is only honored when the number of
root slot write failures occurring during one such admission episode is
less than or equal to k_tol.

Fact 5, the resend clause, from a decision item about where the replay
lower bound comes from. When a root slot write fails and the publish is
resent, the checkpoint's txg number is advanced by one before resending.
This matters because the root ring region a given txg number maps to is
computed as txg modulo the ring's region count R; without advancing the
txg number before resending, a resend would repeatedly rewrite the exact
same slot, collapsing R independent failure domains into one.

Fact 6, the six part prefix rule, from the same decision item as fact 5.
The complete rule for deciding which journal records get applied during
replay has six parts, and all six are required, not five. Part one, jsn
numbers, the record sequence numbers, must be strictly consecutive with
no gaps tolerated. Part two, the pair of instance generation and
checkpoint txg carried by a record must be greater than the selected
root's own water mark. Part three, only transactions whose commit marker
is complete are applied. Part four, before applying a record, every unit
that record names must be individually checksum verified. Part five, if
the selected root's instance has a rollback row recorded in the instance
table, that instance's records are only applied up to the transaction
number W recorded in that rollback row; without this fifth part, an
ordinary recovery that landed on the rolled back root would replay the
entire abandoned timeline back in, silently undoing the administrator's
rollback. Part six, the unit of application
is one whole publish: if the legal prefix determined by the first five
parts stops in the middle of one publish, covering only some of that
publish's transactions, that entire publish is not applied at all, and
the prefix is then truncated back to the nearest publish boundary.

Fact 7, function publish_version. This function, when called, clones the
allocator's current state into a saved copy before attempting to write
anything to disk for this publish. If the inner disk writing step later
returns any error, the allocator is unconditionally restored from that
saved copy, regardless of which one of the on disk writing steps the
error happened at. A comment attached to this function states, in
translation: after admission succeeds, if any subsequent step fails, the
allocator rolls back to the shape it had on entry, this publish did not
take effect.

Fact 8, the on disk writing steps inside publish_version's call chain.
These steps run, in order, inside one function: writing the changed
units, a barrier, writing the journal record, a second barrier, writing
the root slot with force unit access, and rotating the superblock slot.
If any one of these steps returns an error, the code records this
attempt into a failed publish accounting bucket, and hands the original
device error back to the caller unchanged. The caller does not receive
this attempt's resulting version object. A separate function used for a
zero unit empty publish has the identical shape of these same six steps,
and this separate function never touches the allocator at all.

Fact 9, function publish_empty_after. This function computes the next
publish's txg as the in memory current version's own txg plus one, and
its record counter as the in memory current version's own record counter
plus one. Because a failed publish is never written into that in memory
current version, the next publish attempted right after a failure uses
the exact same txg number and the exact same record counter that the
failed attempt used.

Fact 10, two other call sites with the same failure behavior. Two other
functions, one used to raise the rollback floor and one used to take a
new instance number and perform warm up, both internally call the same
publish_version or publish_without_units functions from facts 7 and 8,
and therefore inherit the exact same failure semantics described in facts
7, 8, and 9.

Fact 11, function replay_journal. During recovery, every journal record
that is above the selected root's own water mark, belongs to the same
instance as the selected root, chains correctly to the previous record,
has its commit marker fully set, and whose every individually named unit
passes a checksum check, gets applied by this function. So if both on
disk copies of one publish's journal record persisted successfully, but
that same publish's root slot write never persisted at all, recovery
still applies that publish's record.

Fact 12, no resend implementation exists today. A text search across the
entire core crate's source code for the words "advance by one" and
"resend" only matches comments explaining the intended design. There is
no code anywhere in the currently running implementation that actually
performs a resend of a failed root slot write.

Fact 13, scope note on establish_instance's own rollback. A comment
attached to the function that takes a new instance number states, in
translation: rollback there only covers write errors from the number
taking writes themselves, it does not cover a publish failure that
happens after the number has already been taken.

Stage definitions. Every publish attempt is analyzed as failing at
exactly one of four stages.

Stage S1: a unit write fails partway through, before any copy of the
journal record has been sent to any device.

Stage S2: the journal record has already been sent, but the root slot
force unit access write has not yet returned success. At this point,
zero, one, or two copies of the journal record may actually have
persisted to the two devices.

Stage S3: the root slot force unit access write has already succeeded,
and the failure happens during the superblock slot rotation step that
comes right after it.

Stage S4: the failure happens during one of the superblock slot writes
used to take a new instance number, inside the function that takes a new
number and performs warm up, before that mount's row writing publish.

Arm definitions. Four candidate policies for what happens to the
allocator and the write path after a publish attempt fails, each given as
one prescribed action per stage.

Arm A, always roll back, this is today's running code. At stage S1: the
allocator rolls back to its pre attempt state, and the write path may
keep publishing as before. At stage S2: same handling as stage S1. At
stage S3: same handling as stage S1. At stage S4: if the number taking
write itself fails, the superblock slot rotation is rolled back; this is
today's running code, per the comment in fact 13.

Arm B, split by stage. At stage S1: the allocator rolls back, and the
write path may keep publishing as before. At stage S2: the allocator does
not roll back; this publish is treated as having an undetermined outcome;
the write path is only allowed to resend the identical content at txg
plus one, following the resend rule in fact 5, and nothing else may be
published until that resend succeeds. At stage S3: the allocator does not
roll back; this publish is treated as having already taken effect, the
caller receives this publish's resulting version, and the superblock slot
gets rotated at the next publish instead. At stage S4: same handling as
arm A's stage S4.

Arm C, stop on failure. At stage S1: the allocator rolls back, and the
write path may keep publishing as before. At stage S2: the write path
becomes void, no further publish is allowed for the rest of this mount,
the caller must remount, and recovery alone decides which version ends up
being the one on disk. At stage S3: same handling as arm C's stage S2. At
stage S4: same handling as arm A's stage S4.

Arm D, hold back. At stage S1: the allocator rolls back. At stage S2: the
allocator rolls back, but the storage slots this attempt had occupied are
held back, meaning they may not be handed out to anything else, until a
root record with a strictly larger txg is persisted. At stage S3: same
handling as arm D's stage S2. At stage S4: same handling as arm A's stage
S4.

Task, table 1. For each of the four stages S1, S2, S3, and S4, answer
whether recovery, restarting after a power loss that happened at that
exact stage, could possibly select a root and replay records such that
this failed publish ends up treated as having taken effect. Use only
facts 1, 2, 5, 6, 11, and 12 above, plus the stage definitions. For each
stage, give one of exactly these three labels: definitely yes, definitely
no, or depends on how many journal record copies persisted. If you give
the third label, state explicitly which counts among zero, one, and two
persisted copies would make the answer flip between yes and no, and why.
For every stage, name which fact or facts your answer rests on, and add
the required "This would be refuted by:" sentence.

Task, table 2. For each of the four arms A, B, C, and D, for each of the
four stages S1, S2, S3, and S4, and for each of these four clauses, named
clause F, clause N, clause R, and clause K, where clause F is fact 1,
clause N is fact 3, clause R is fact 5, and clause K is fact 4, decide
whether that arm's prescribed handling at that stage, as given in the arm
definitions above, is consistent with that clause's literal wording, is
in conflict with that clause's literal wording, or is a case where the
clause simply has nothing to say about that stage, making it not
applicable. This produces sixty four cells in total, four arms times four
stages times four clauses. For every single cell, state your verdict as
one of exactly these three words: consistent, conflict, or not
applicable, and then give one sentence of justification. If your verdict
is conflict, that sentence must quote, word for word, the exact clause
sentence from the relevant fact that the arm's handling conflicts with.
If your verdict is consistent, that sentence must quote, word for word,
the exact clause sentence from the relevant fact that supports
consistency. If your verdict is not applicable, that sentence must state
in your own words why the clause does not address that stage at all. Do
not skip any of the sixty four cells, and do not merge cells together
even where you believe several stages or several arms behave identically,
each cell must carry its own explicit verdict and its own quoted or
explained justification.

Organize table 2 by arm, then by stage within each arm, then by clause
within each stage, so that the full output has four blocks, one per arm,
each block containing four stages, each stage containing four clause
verdicts.

Questions.

1. Give table 1 in full, covering all four stages, following the
instructions for table 1 above.

2. Give table 2 in full, covering all four arms, all four stages, and all
four clauses, following the instructions for table 2 above.

3. Across the sixty four cells of table 2, is there any single cell where
you marked conflict, such that the same arm, at a different stage, was
marked consistent with the same clause using what looks like the same
underlying reasoning applied inconsistently between the two stages? If
you find such a case, name the two cells precisely by arm, stage, and
clause, and explain the inconsistency. If you find no such case, say so
explicitly and add the required "This would be refuted by:" sentence.

4. Fact 6's sixth part says that the unit of application during replay is
one whole publish, so a publish whose legal prefix stops in the middle of
its own transactions is not applied at all. For stage S2 specifically,
where zero, one, or two journal record copies may have persisted, explain
whether fact 6's sixth part, by itself, already forces the outcome you
gave in table 1 for stage S2, or whether table 1's answer for stage S2
still depends on something beyond fact 6's sixth part. Name exactly what
that something is, if there is one. Add the required "This would be
refuted by:" sentence.

5. Arm B's stage S2 handling says the write path may only resend the
identical content at txg plus one, and nothing else may be published
until that resend succeeds. Fact 5's resend clause says the txg is
advanced by one before resending, but fact 5 does not state anywhere
that the resent content must be byte for byte identical to the original
failed attempt's content. Is arm B's added restriction, resending only
the identical content, itself consistent with fact 5's literal wording,
in conflict with it, or simply an extra restriction that fact 5 neither
requires nor forbids? Add the required "This would be refuted by:"
sentence.
