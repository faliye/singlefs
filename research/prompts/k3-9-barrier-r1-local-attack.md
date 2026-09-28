SINGLEFS PUBLISH BARRIER REDUNDANCY - LOCAL ATTACK LEG - ROUND k3-9-barrier-r1

You are one of three independent reviewers checking a design question for
singlefs, a copy on write filesystem being designed from scratch. You are
given a self contained set of facts below. Do not assume anything not
stated here. Read carefully, every qualifier in this text is load bearing,
a definition with one word removed can flip an answer. Do not use any
markdown emphasis anywhere in your answer, no bold text, no italic text,
no backtick code formatting, no asterisk bullets. Number your answers to
match the question numbers given at the end. For every numbered answer
you give, add one sentence starting with "This would be refuted by:"
describing the specific observation that would prove that answer wrong.
Where a question asks you to fill in a table, give the table with an
explicit value and an explicit justification for every single cell, not
just "yes" or "no" for the whole table, and do not merge cells together
even where you believe several rows or columns behave identically, each
cell must carry its own explicit verdict and its own explained
justification grounded in the fact numbers given below. When your answer
needs to point at a source, point at it by its fact number, by the row
label (R1 through R5), by the column label (C1 through C4), or by naming
the row and column of the table you are answering. Do not write any
source file name together with a line number, and do not write any line
number at all, anywhere in your answer, because the underlying files may
have moved to different line numbers by the time anyone reads your
answer; if you need to point at a piece of code, name the function it is
in instead.

Background. Singlefs groups writes into checkpoints, called publishes
below. One publish attempt, inside one function, does five things in this
fixed order: first, it writes every changed storage unit to every device,
in a loop, one unit at a time. Second, it issues a barrier, called
barrier A below. Third, it writes every journal record that belongs to
this publish, to every device, in a loop, one record at a time; there is
no barrier between one record write and the next record write inside
this loop. Fourth, it issues a second barrier, called barrier B below;
barrier B is not the subject of this question and is never removed in
any column below. Fifth, a separate function persists the root slot and
then rotates the system configuration slot. The question under study is
what happens if barrier A, and only barrier A, is removed, while barrier
B, the named-unit verification step described in fact 5 below, and every
other step, stay exactly as they are today. Separately, a recovery
procedure, after a crash, picks one root record to trust, called the
chosen root, and then walks the journal records that come after the
chosen root's own position, in order, deciding which of them, if any,
should be treated as taking effect; this walk is called replay below.

Judges. Three independent judges are used to decide whether a given
crash state is a problem. The pool level checker walks the on disk image
that recovery has left behind and reports a fixed list of invariants
about that image's own internal structure, such as reachability and
checksums inside its trees; it does not compare that image against what
any specific publish attempt intended to write, and a unit write that
ended up not referenced by anything is not, by itself, a violation for
this judge. The ideal model keeps its own separate record of every
version it has itself committed while driving the same writes. After a
simulated crash and recovery, this judge requires two things of the
version recovery actually lands on: first, that version's checkpoint
counter must not be older than the newest root slot the model itself
observed as durably persisted on disk; second, that version must be one
the model itself has a committed record of, with matching file content;
either condition failing is reported by this judge, and either failure
is called "model red" below. The record auditor has two independent
criteria, and either one firing is called "record auditor red" below.
Criterion root without record: for a given publish that itself wrote
at least one journal record, if that publish's root slot write is
present in the post crash image but not a single one of that publish's
journal record writes is present, this criterion fires; a publish that
never wrote any journal record at all, such as a zero unit publish,
does not trigger this criterion merely for having no records present.
Criterion claimed state missing unit: for a given publish whose
checkpoint counter is less than or equal to the checkpoint counter
recovery actually landed on, and which recovery's chosen outcome has not
itself treated as abandoned, belonging to a discarded instance, if every
on disk copy of some unit that publish wrote is judged missing from the
post crash image, this criterion fires.

Fact 1, the crash and segment model used by every probe number below. A
crash point falls inside exactly one write segment, where a segment is a
run of writes with no barrier inside it. Every segment strictly earlier
than the crash point's segment persists in its entirety. The segment the
crash point itself falls in persists an arbitrary, genuine subset of its
own writes, chosen independently for each modeled crash state. Every
segment strictly later than the crash point's segment does not persist
at all. This is stated as the lower bound model of what the block layer
actually promises, and is the enumeration domain every probe number
below was generated from.

Fact 2, segment shape with barrier A present, from a probe run on
unmodified code. With barrier A present, one publish's unit writes and
that same publish's record writes fall into two different segments, in
that order, and that publish's root slot write falls into a third
segment, strictly later than the other two. A concrete example from this
probe: unit writes fall in segment 12, record writes fall in segment 13,
the root slot falls in segment 14.

Fact 3, segment shape with barrier A removed, from a probe run on code
with barrier A deleted. With barrier A removed, one publish's unit
writes and that same publish's record writes fall into the same single
segment, and that publish's root slot write still falls into a separate,
strictly later segment, because barrier B, between the record writes and
the root slot write, was not touched. A concrete example from this
probe, same publish as fact 2's example: unit writes and record writes
both fall in segment 12, the root slot falls in segment 13.

Fact 4, no barrier between one publish's own record writes. Inside the
loop that writes one publish's journal records to every device, one
record after another, there is no barrier between any two of those
record writes, regardless of whether barrier A is present or removed.
Consequently, when a publish has more than one record, those records
always share one segment with each other, and fact 1's "arbitrary
subset" rule can make some of them persist and others of the same
publish not persist, independent of barrier A.

Fact 5, replay's per record verification step. Replay is given a boolean
flag, called verify below. Walking one candidate record, if verify is
false, that record is treated as fully verified without reading a
single one of the units it names. If verify is true, that record is
treated as fully verified only when, for every unit the record names,
every on disk copy of that unit can be read and its checksum matches; if
any copy of any named unit cannot be read or its checksum does not
match, this record is not applied, and the walk stops at this record,
meaning every later candidate record in this same replay call is also
not applied, regardless of what those later records themselves say.

Fact 6, two recovery policies map to the verify flag. There are exactly
two recovery policies that consult the journal at all, named Consult and
ConsultWithoutNamedVerification. Both call the same replay procedure
described in fact 5; the verify flag passed to it is true exactly when
the policy is Consult, and false exactly when the policy is
ConsultWithoutNamedVerification.

Fact 7, which policy ordinary and crash injected recovery use. Every
place in the checker tier code that performs recovery to model what
happens after a crash uses policy Consult. A comment attached to the
policy type states, in translation, that ConsultWithoutNamedVerification
exists only to be forced on for testing, specifically to prove that the
verification step in fact 5 is load bearing; it is not used by ordinary
recovery.

Fact 8, verification passed counter is always zero under the second
policy, from a comment attached to the replay report structure, in
translation. A counter named verification passed counts committed
records whose named units were actually read one by one and whose
checksums were actually compared one by one. With
ConsultWithoutNamedVerification, this counter is always zero, because
that policy does not read a single unit; on an otherwise healthy image,
this counter is the only reading that tells the two policies apart.

Fact 9, only the chosen root's own instance is a candidate for replay.
Walking candidate records for replay, only records whose instance number
equals the instance number of the already chosen root are even
candidates at all; a record belonging to an instance newer than the
chosen root's instance is never a candidate, independent of the verify
flag and independent of barrier A.

Fact 10, named unit verification is the fourth of a five part prefix
rule, from a decision item about the journal's role and format, in
translation. Verify every named unit item by item before applying it
(this criterion is proven load bearing by fact 18 below) -- a named unit
is applied only when both of its on disk copies can be read and their
checksums both match; if one copy cannot be read or its checksum does
not match, this record and every record after it are not applied.

Fact 11, applying is whole publish or nothing, from the same decision
item as fact 10, in translation. The unit that gets applied is one whole
publish -- when the legal prefix of readable, verified records stops
between two transactions inside one publish, that whole publish is not
applied; the prefix produced by the other four criteria is then
truncated again at publish boundaries. How a publish boundary is
recognized: the last record of a publish is the one whose dedicated
record flag bit is set to one; if the walk described in fact 5 does not
reach that flagged record among a publish's records, the whole publish
is not applied.

Fact 12, shared blocks are named only in a publish's last record, from
the same decision item as fact 10, in translation. When one publish is
split into several transactions and several records, the blocks
generated by the commit itself that this publish shares across its
records, namely extent tree nodes, inode leaf containers and their
roots, the allocation record tree, the accounting tree, the central
mapping tree, and the tree table unit, are named only in the last
record; the earlier records of
the same publish do not repeat naming them.

Fact 13, a publish's records can themselves span further records, from
the same decision item as fact 10, in translation. When the shared
blocks named in fact 12 are more numerous than one record can hold, the
last record spans across further records: starting from the record of
the last transaction, a few more records are written, each filled
before the next is opened, and only the truly last one of them carries
the record flag bit described in fact 11. Replay recognizes a publish's
boundary by this flag; if the flagged record is not found among a
publish's records, the whole publish is not applied, per fact 11.

Fact 14, what happens when a crash lands between a record persisting and
its root persisting, from the same decision item as fact 10, in
translation. Recovery rebuilds that publish's root from the record
itself: the record header carries a dedicated segment of root fields,
and applying a record means swapping these four fields of the chosen
root for the ones carried in the record: the tree table unit pointer,
the central mapping tree root pointer, the tree id watermark, and the
rollback floor, without needing any tree level
understanding of what those fields point to. The already stated rule
that a record strictly newer than the chosen root gets applied still
holds unchanged.

Fact 15, the publish persist order restated at the top decision level,
from a decision item about publish semantics, in translation. The
persist order of one publish is always: copy on write units or nodes,
then a barrier, then the journal records, then a barrier, then the root
slot written with force unit access, then the system configuration slot.
Recovery replay must verify the checksums of named units item by item
before applying any record.

Fact 16, every mount runs recovery, regardless of a clean or unclean
prior shutdown, from a decision item about the journal's role and
format, in translation. There is no clean shutdown marker anywhere on
disk; every mount always runs recovery in full, meaning a full scan of
the journal ring, item by item verification, and root selection, without
branching on whether the previous shutdown was clean; admission control,
warm up, the rollback candidate set, and any parameter taking effect all
read the same freshly computed recovery result, never a shortcut flag.

Fact 17, experiment E77's own violation counts across four barrier
layouts, from a crash subset enumeration experiment whose model matched
fact 1's segment rule, using six units, three same transaction records
with the commit flag on the last one, and one root slot. Layout
"all three barriers" (a barrier between units and records, and a barrier
between records and the root slot, in addition to the one after the root
slot): 72 reachable states, 0 violations with verification on, 0
violations with verification off. Layout "no barrier between units and
records, one barrier between records and the root slot": 79 reachable
states, 0 violations with verification on, 0 violations with
verification off, but 7 states where the root slot is present while no
record of that publish is present. Layout "one barrier merging units and
records together, one barrier before the root slot" (this is barrier A
removed, matching fact 3 above): 513 reachable states, 0 violations with
verification on, 63 violations with verification off. Layout "no barrier
at all before the root slot": 1024 reachable states, 504 violations with
verification on.

Fact 18, experiment E77's own conclusion about barrier A, from the same
experiment as fact 17, in translation. No barrier is needed between
units and records, provided that replay verifies named units item by
item before applying -- remove that verification, and the layout in
fact 17 matching barrier A removed immediately shows 63 silent grafts,
meaning a record gets applied while a unit it names does not actually
match on disk. So: each item a record names carrying its own checksum is
not a nice to have, it is the trade off condition that lets barrier A be
dropped; the recovery path must actually go and verify it, which is
exactly fact 5's verify flag.

Fact 19, evidence about a new instance's first publish, from the
mechanism report behind this round, in translation. For a new instance's
first publish, its records are never even candidates for replay, per
fact 9, because the chosen root at that point still belongs to the
previous instance; this holds regardless of the verify flag and
regardless of barrier A.

Fact 20, probe evidence, barrier A removed, policy Consult, from the
mechanism report behind this round. Across 1433 modeled crash states:
record auditor's root without record criterion fired 0 times, its
claimed state missing unit criterion fired 0 times, the pool level
checker fired 0 times, the ideal model fired 0 times. The replay
procedure's own count of records that failed named unit verification
was greater than zero in 903 of these 1433 recoveries.

Fact 21, probe evidence, barrier A removed, policy
ConsultWithoutNamedVerification, from the same report as fact 20, same
1433 modeled crash states as fact 20, same underlying code and same
underlying crash points, only the policy changed. Record auditor's
claimed state missing unit criterion fired 752 times, the ideal model
fired 752 times, root without record fired 0 times, the pool level
checker fired 0 times. The replay procedure's own count of records that
failed named unit verification was zero in all 1433 of these recoveries,
because verification was turned off. The replay procedure's own count of
records that got applied as part of some publish's prefix was greater
than zero in 911 of these 1433 recoveries.

Fact 22, probe evidence, unmodified code, both policies, from the same
report as fact 20. With barrier A present, under 62 modeled crash
states, all four of the pool level checker, the ideal model, the record
auditor's root without record criterion, and the record auditor's
claimed state missing unit criterion fired 0 times; this held both under
policy Consult and, separately, under policy
ConsultWithoutNamedVerification, on the same 62 states.

Fact 23, what this round's probe did not cover, from the same report as
fact 20, in translation. The probe covered only two stages: recovery
right after one crash, and a second writable mount, followed by one more
publish, performed after that recovery. It did not cover a second crash
happening during that second writable mount's own publish, and it did
not run the actual long running test in the checker tier crate that this
whole question originates from.

Row definitions. Five categories of crash state are judged in the table
below, each held fixed across all four columns.

R1, single record publish, units entirely missing. The publish has
exactly one record. Every on disk copy of every unit that record names
is either unreadable or fails its checksum. The record itself is
present, and would pass every one of the five prefix criteria except
possibly fact 10's named unit verification criterion. That publish's
root slot is not present.

R2, single record publish, units partly missing. The publish has exactly
one record. At least one, but not all, of the units that record names
has every on disk copy either unreadable or checksum failing, while at
least one other unit the same record names has at least one readable,
checksum matching copy. The record itself is present, and would pass
every one of the five prefix criteria except possibly fact 10's named
unit verification criterion. That publish's root slot is not present.

R3, single record publish, units entirely present. The publish has
exactly one record. Every unit that record names has at least one
readable, checksum matching copy, so fact 10's named unit verification
criterion would pass if it ran. The record itself is present, and passes
every one of the five prefix criteria. That publish's root slot is not
present.

R4, multi record publish, last record missing. The publish has at least
two records. Every record up to and including some record before the
last one is present and would pass every one of the five prefix criteria
except possibly fact 10. The publish's last record, the one carrying the
flag described in facts 11 and 13, and the only one naming the shared
blocks described in fact 12, is genuinely absent from the post crash
image, not merely failing its checksum. That publish's root slot is not
present.

R5, a second crash during the second writable mount. State exactly as
described in fact 23's second stage: a first crash already happened and
recovery already ran once; a second, writable mount then began, and
began writing at least one new publish of its own; before that new
publish's own barrier B and root slot write, this second mount itself
crashes. For this row, pick exactly one of R1, R2, or R3's own unit
presence pattern to describe how that new publish's own units and record
are torn by this second crash, state explicitly in your answer which one
of R1, R2, or R3 you picked, and hold that same pick fixed across all
four columns of this row. No probe evidence in the facts above covers
this row; state explicitly, in each cell, whether you are relying only
on facts 1 through 19 applied to a second crash, or whether you are
assuming, without any fact above confirming it, that a second crash
during a second mount behaves exactly like a first crash during the
first mount.

Column definitions. Four columns, each a combination of whether barrier
A is present and which recovery policy recovery uses.

C1, barrier A present, policy Consult. This is today's running code
together with today's default recovery policy.

C2, barrier A present, policy ConsultWithoutNamedVerification. This is
today's running code together with the test only policy from fact 6.

C3, barrier A removed, policy Consult. This is the change under study,
together with today's default recovery policy.

C4, barrier A removed, policy ConsultWithoutNamedVerification. This is
the change under study, together with the test only policy from fact 6.

Outcome labels. For every cell of the table, your stated outcome must be
exactly one of these five phrases, verbatim: "rolls back to the previous
version", "applies through this publish", "applies half a publish",
"mount is refused", or "this crash state cannot occur in this column".
Use the fifth phrase only when the facts above, read together, make that
row's own description impossible to satisfy under that column, and say
in one sentence, grounded in fact numbers, exactly which two stated
conditions of the row contradict each other under that column; do not
use the fifth phrase merely because you find the row unlikely.

Task, table 1. For each of the five rows R1 through R5, and for each of
the four columns C1 through C4, this produces twenty cells in total. For
every single cell, state five things. First, the outcome label, exactly
one of the five verbatim phrases given above. Second, whether the pool
level checker is red or not red for that cell. Third, whether the ideal
model is red or not red for that cell. Fourth, whether the record
auditor is red or not red for that cell, and if red, state whether it is
the root without record criterion, the claimed state missing unit
criterion, or both. Fifth, one or more fact numbers, from facts 1 through
23, that your first four answers for this cell rest on, plus one
sentence of justification tying those fact numbers to this specific row
and column. Do not skip any of the twenty cells. Organize table 1 by
row, then by column within each row, so the output has five blocks, one
per row, each block containing four column verdicts.

Questions.

1. Give table 1 in full, covering all five rows and all four columns,
following the instructions for table 1 above.

2. Compare column C1 against column C3 for each of rows R1 through R4,
holding policy Consult fixed and only changing whether barrier A is
present. List every row where your outcome label, or your red or not red
verdict for any of the three judges, differs between C1 and C3. For each
such row, name the fact numbers that make C1 and C3 differ for that row.
If you find no such row among R1 through R4, say so explicitly and add
the required "This would be refuted by:" sentence.

3. Compare column C3 against column C4 for each of rows R1 through R5,
holding barrier A removed fixed and only changing the recovery policy.
List every row where your outcome label, or your red or not red verdict
for any of the three judges, differs between C3 and C4. For each such
row, name the fact numbers that make C3 and C4 differ for that row. If
you find no such row, say so explicitly and add the required "This would
be refuted by:" sentence.

4. Row R4 concerns a publish's last record being genuinely absent, not a
checksum failure. Using only facts 1 through 4, 9, and 11 through 13,
does barrier A being present or removed change anything about whether
that publish's earlier records ever get copied into the recovered root,
for either value of the recovery policy? Answer yes or no, name the
exact fact numbers your answer rests on, and add the required "This
would be refuted by:" sentence.

5. For row R5, the row you had to extend past the given probe evidence
yourself. State plainly, in your own words, one specific thing a real
run of the actual checker tier test named in fact 23 could show that
would force you to revise your own R5 row of table 1, and state which
one of your four R5 cells it would most likely change first. Add the
required "This would be refuted by:" sentence, where the observation you
name in this answer and the refuting observation may be the same thing.

6. Across all twenty cells of table 1, is there any cell where you used
the outcome label "applies half a publish"? If yes, name that cell by
row and column, and, using fact 11's own wording about whole publish or
nothing application, explain how your cell's reasoning is compatible
with fact 11. If no cell used that label, say so explicitly, and state
in one sentence, grounded in fact 11, why that label may be structurally
unreachable given the replay mechanism described in facts 5, 9, and 11
through 14, regardless of barrier A or the recovery policy. Add the
required "This would be refuted by:" sentence.

7. Fact 18 states that experiment E77's own conclusion is that no
barrier is needed between units and records, provided verification runs
before replay applies a record. Using only your own twenty cells from
table 1, and stating explicitly which cells you are pointing at by row
and column, does your own filled table agree with fact 18's conclusion,
or does at least one of your cells contradict it? If you find a
contradicting cell, name it precisely and explain the contradiction
using fact numbers. If you find none, say so explicitly and add the
required "This would be refuted by:" sentence.
