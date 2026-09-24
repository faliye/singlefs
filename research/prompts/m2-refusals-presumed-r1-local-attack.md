SINGLEFS REFUSAL AUDIT - LOCAL ATTACK LEG - ROUND m2-refusals-presumed-r1

You are one of three independent reviewers checking a batch of refusal
behaviors in singlefs, a copy on write filesystem being designed from
scratch. You are given a self contained set of facts below. Do not assume
anything not stated here. Read every fact carefully, every qualifier is
load bearing, a definition with one word removed can flip an answer. Do not
use any markdown emphasis anywhere in your answer, no bold text, no italic
text, no backtick code formatting, no asterisk bullets, no pipe tables.
Number your answers to match the four judgments given at the end. When your
answer needs to point at a source, point at it only by the fact number
given below, fact 1 through fact 20, or by naming a function by its own
name. Do not write any source file name together with a line number, and
do not write any line number at all anywhere in your answer, because the
underlying files may have moved to different line numbers by the time
anyone reads your answer.

This is a separate, independent document from other documents in this same
review round that ask about other refusal behaviors in the same codebase.
You are not expected to see those other documents and should not assume
anything about their content.

Background. Each of the four judgments below concerns one specific
refusal: a place in the code that, under some condition, returns an error
instead of writing anything, before any state changes. For each refusal
you must fill in a table with exactly four labeled parts, using the exact
fact numbers that justify each part. Do not answer any part with a bare
yes or no; always name the fact numbers your answer rests on and add one
sentence of justification.

Part one asks: before this specific refusal fires, has this function, or
anything it called before reaching the refusal, already written anything
to a storage device. Answer yes or no and name the fact that shows this.

Part two asks: among all the ways a real, non malicious user could reach
the exact condition that triggers this refusal, is there at least one way
that involves zero faults at all, meaning no disk corruption, no crash, no
torn write, no I/O error, nothing but ordinary sequential use of the
filesystem. Answer yes, no, or unknown. Unknown is an acceptable answer
and does not count as a failure to answer, but you must still name which
facts you considered and say specifically what additional fact would be
needed to turn your unknown into a yes or a no. If you answer yes, you
must describe the concrete sequence of ordinary operations that reaches
the condition. If you answer no, you must say what it is about the
condition that requires a fault.

Part three asks: does the name of the error value that this refusal
returns describe the actual condition being checked, without also
covering some other condition that would need a differently named error.
Answer yes or no and justify using the exact condition described in the
facts below.

Part four asks: considering the criterion given in fact two below, should
this specific gap be written as a new decision clause, should it be
registered as an unsupported case for the first version, or is it already
fully covered by an existing decision clause, such that neither a new
clause nor a registration is needed. Pick exactly one of these three and
justify it using the fact numbers for whatever existing clause you
considered, and using fact two's own criterion for telling a clause worthy
gap apart from an implementation detail that does not need a clause.

For every part of every judgment, end with one sentence starting with
"This would be refuted by:" describing the specific observation that
would prove that part wrong.

Fact 1. The project's own three part test for when a refusal that blocks
an operation instead of guessing is considered sound, rather than being a
wrongful refusal, stated in full. First, the refusal must fire early,
meaning before any state on disk has been touched at all, so that
refusing leaves every byte on disk exactly as it was. Second, the refusal
must be accurate, meaning that among the whole family of histories that
get refused by this condition, none of them is a history that is reachable
with zero faults and that a user would have a legitimate reason to want to
perform. Third, the refusal must be clear, meaning the name of the error
value returned states what is actually being refused, and is not reused
for some different shape of refusal.

Fact 2. The project's own criterion, fixed before any of these four
judgments were answered, for whether a given gap should become a new
written decision clause. A gap counts as needing a new clause only when
there are two or more different choices that each would be a defensible
way to fill the gap, and those different choices produce different bytes
on disk, or lead to different sets of reachable histories, for at least
some input. If every defensible choice for filling the gap would produce
identical bytes on disk and identical reachable histories, then the gap
does not need a new clause, and should instead be treated as an
implementation detail with no clause needed.

Fact 3. The exact question for judgment one. Before the very first file
version is ever published in a pool, which happens right after the second
of two warm up publishes that occur during the first ever writable mount
of a freshly made filesystem, does the code check the previous record in
the journal before writing the first file version, or does it write
without checking.

Fact 4. The function that publishes the first file version, from direct
observation of its own source. Before doing anything else, it parses a
byte string that is supposed to contain the most recently written journal
record for this pool, extracting that record's own checkpoint transaction
number and its own counter field if the bytes parse successfully at all.
It then computes whether both of two things hold: the parsed record's
checkpoint transaction number is exactly one less than a fixed constant
transaction number reserved for the first file version, and the parsed
record's own counter field is also exactly one less than that same fixed
constant. If the bytes fail to parse at all, or if either of those two
equality checks fails, the function immediately returns a named error
carrying whatever it did manage to parse, if anything, and does not call
any function that would write a single byte to any storage device. Only
when both equality checks hold does the function go on to call a second
function that performs the actual allocation and writing of the first
file version.

Fact 5. The same refusal from fact 4, considered from two further angles.
First, from a search of the project's own tracked decision clauses, this
specific check has no covering decision clause at all; the code comment
attached to it says so explicitly. Second, from the code comment's own
stated history, this specific check was added only after a different
independent reviewer, in an earlier round of the same kind of review this
document belongs to, had already demonstrated a concrete failure without
this check in place: with two of the filesystem's redundant configuration
records corrupted, followed by one ordinary crash, the code as it stood
before this check was added would successfully write the first file
version at the fixed constant transaction number regardless of what the
previous record actually was, overwriting a warm up record that a cold
start recovery still needed, and the write reported success even though
the written version could not be read back afterward.

Fact 6. The exact question for judgment two. When a writable mount needs
to acquire the next instance number, is that number computed exactly
once, or is it computed once for the purpose of deciding what else this
mount is allowed to do, and then computed a second time immediately
before the write that actually claims the number, with the mount refusing
and writing nothing if the two computations disagree.

Fact 7. The function that carries out the second half of establishing a
writable mount, from direct observation of its own source. Early in this
function, before any refusal check runs and before anything is written,
it computes the instance number this mount intends to acquire, exactly
once, and uses that one computed value both to decide which rows this
mount will need to write into the instance table and to run several
admission checks that can themselves refuse the mount before anything is
written. After all of those checks pass, this function calls a second,
separate function, passing in that same previously computed value as the
expected number. That second function recomputes the instance number to
acquire completely independently, by reading the pool's on disk state
again from scratch. If the freshly recomputed number does not equal the
expected number that was passed in, this second function immediately
returns a named failure carrying both the expected and the recomputed
numbers, and does not call the function that would actually write the
claimed number to any device. Only when the two numbers agree does this
second function go on to call the function that performs the actual
write.

Fact 8. The failure value returned by the mismatch described in fact 7,
from direct observation of its own documentation comment attached
directly to its definition. The comment states plainly that when this
mismatch happens, not a single byte has been written by this mount so
far. A second, separate documentation comment, attached to the function
that performs the actual acquisition write and explaining why this
recomputation step exists at all, states that the recomputed number can
differ from the earlier expected number because of two separate reads of
the same on disk configuration data happening at two different times, and
that even a single transient read error occurring on just one of those
two reads is enough by itself to cause a mismatch, because a slot that
failed to be read on that one read is treated as though it were simply
absent from the on disk state at that moment.

Fact 9. The naming of the error returned by the mismatch described in
facts 6 through 8, from direct observation of its own source. The mount
level error value that is actually returned to the caller of the mount
operation is built directly from the named failure described in fact 7,
carrying forward the exact same expected number and the exact same
recomputed number without adding or removing any information, through a
one to one mapping with no other logic in between.

Fact 10. A separate, already closed entry in the project's own ledger of
tracked open questions, titled as being about how the barrier at the
instance acquisition step is placed, found in the closed section of that
ledger rather than its open section. The closed entry states it was
settled by three separate decision clauses together: one clause, in a
different decision record about the journal, adding a barrier that fires
right after acquisition together with two implementation obligations; a
second clause, in the same decision record that fact 11 below quotes from,
fixing the method for reading which number to acquire, the rule for
rolling back to an old number, and the barrier's error behavior; and a
third clause, in yet another decision record, noting that the number is
incremented by one separately on each device.

Fact 11. The decision text named in fact 10 as the second of the three
clauses that settled that ledger entry, describing the order of
operations for a writable mount that is about to claim a new instance
number, translated in full except for citations to other decision clause
numbers used only as further cross references, which are left out here
because they cannot be checked from this document alone. It states: the
mount first decides whether it is allowed to be writable at all, by
checking together that enough devices are writable to meet this pool's
minimum device count, that this mount holds exclusive access to more than
half of the devices in the pool, that the instance table can be read, and
that the reservation needed for an instance switch is available. Only
after all of that passes does the mount compute the new instance number,
once, as one plus the larger of two quantities: the highest instance
number found among the self certifying configuration slots belonging to
every device this mount holds exclusive access to, and the highest
instance number found among every root record present in the root ring.
This single freshly computed number is then written into every device in
that same exclusively held set, all or nothing. A single input or output
error on any one of these writes is retried until a fixed retry budget is
exhausted before being counted as a failure at all. Once that budget is
exhausted and the all or nothing write is judged to have failed, every
copy already written during this same attempt is rolled back to the old
number, defined as the highest instance number that was found among the
self certifying slots of this same set before this attempt began; this
rollback write happens before any unit is ever touched, and the mount
only falls back to a read only mount if even this rollback cannot be
completed. A separate barrier that fires immediately after this
acquisition step is likewise treated as causing the same kind of all or
nothing failure if it reports an error on any one device, and the mount
is not allowed to simply resend that barrier and continue as though
nothing happened. A read only mount never performs this acquisition step
at all. Instance numbers start at one, and zero is never a valid instance
number.

Fact 12. An observation about the same decision text quoted in fact 11,
from a direct, careful reading of it. Nowhere in that quoted text is there
any description of computing the new instance number a second time,
immediately before the write, separately from the one computation already
described, nor is there any description of comparing two separately
computed numbers against each other and refusing to write when they
disagree. The quoted text describes exactly one computation of the new
number, occurring after the admission checks and before the write.

Fact 13. The exact question for judgment three. When a filesystem is
first created on exactly two storage devices, is the assignment of which
of the three fixed root ring regions lives on which of the two devices
forced to one single fixed pattern, with any other pattern being refused
before anything is written, or can the caller who is creating the
filesystem choose a different assignment freely.

Fact 14. The function that checks filesystem creation time geometry
constraints, from direct observation of its own source. This function is
called once near the very start of the whole filesystem creation
operation, before any device is written to for any purpose. Among several
other checks it performs, when the number of devices given is exactly
two, it compares the caller supplied array that states which device each
of the three root ring regions is assigned to against one single fixed
constant array built into the source code itself. If the caller supplied
array does not exactly equal that fixed constant array, this function
immediately returns a named error carrying the caller supplied array, and
the filesystem creation operation as a whole then writes nothing to any
device at all. This caller supplied array is an ordinary field of the
parameters structure that the caller of the filesystem creation operation
must fill in directly; nothing in the filesystem creation code derives
this field automatically from the number of devices or from the devices'
own identities, so a caller is free to pass any array of three device
choices it likes when calling filesystem creation with exactly two
devices.

Fact 15. The project's own decision text that the fixed constant array
named in fact 14 is drawn from, stated in full for its definition and
scope paragraphs. The definition states: region assignment is no longer
computed by a formula from the device count, instead the device identity
for each region is written down explicitly at filesystem creation time;
for the first version, with exactly two devices, the three regions'
assignment is fixed at device zero, device one, device zero, for region
zero, region one, and region two respectively; filesystem creation writes
this into a twelve byte area of the system configuration, four bytes per
region; and this assignment is explicitly not a parameter that the caller
of filesystem creation gets to choose. The scope paragraph states, among
other things, that the user made two separate decisions here: first,
the general decision to switch from computing this assignment by a
formula to writing device identity down explicitly, and second, the
specific decision that this fixed pattern for exactly two devices is
zero, one, zero. Only the second of those two decisions was not put
through the same three way independent review process that this
present document is also an instance of, and that second decision is
explicitly stated as still being open to being overturned. The scope paragraph's own stated
justification for why this particular pattern, and not some other
assignment, was chosen states that once region numbering is fixed, this
particular pattern is the only conventional way to write it down, and
that making this assignment into a parameter the caller could choose
would only buy the ability to choose which one device carries two
regions instead of one, and that with exactly two devices this choice has
no observable difference at all.

Fact 16. Two further pieces of context about the same refusal from facts
13 through 15. First, from the code comment attached directly to the
named error in fact 14, this specific refusal was added after a different
independent reviewer, in an earlier round of the same kind of review this
document belongs to, had already demonstrated that if filesystem creation
were allowed to accept a different region assignment on two devices, one
particular different assignment would require the warm up sequence to run
twice as many times as expected, and would then collide the very first
file version with one of those warm up publishes at the exact same
combination of instance number and transaction number. Second, nothing in
the facts given in this document states that supplying a region
assignment other than the one fixed pattern would ever be blocked by
anything other than this one refusal itself; the field is an ordinary
parameter with no other gate in front of it.

Fact 17. The exact question for judgment four. When the root record that
a mount selects points to an instance table or a tree table whose own
recorded birth transaction number is not the fixed generation number that
filesystem creation itself always uses, and the tree table on that same
version has zero entries in it, what does the mount do.

Fact 18. The function that works out, at mount time, which units on disk
belong to a version whose tree table has zero entries, from direct
observation of its own source and its own documentation comment. As the
very first thing this function does, before constructing anything that
tracks which storage locations are free or in use, and before touching
any unit on disk in any way, it checks whether the birth transaction
number recorded on the pointer to the instance table, or the birth
transaction number recorded on the pointer to the tree table, differs
from the fixed generation number that filesystem creation itself always
writes. If either one differs, the function immediately returns a named
error carrying the root's own identifying fields together with both of
those birth transaction numbers, and does not construct anything that
tracks free or used storage at all. The function's own documentation
comment states plainly, as the stated reason for this refusal, that where
such a version's own units would live is not written down in any decision
clause, and separately states that this implementation itself has no way
to ever produce a root that points to such a version; the comment states
that only a corrupted device, or some other piece of software entirely
that also writes to this same on disk format, could ever produce one.

Fact 19. The project's own decision text about whether an empty publish,
meaning a publish that writes no new file content at all, still writes
any units, stated in full for its definition and scope paragraphs. The
definition states: whenever the accounting tree already exists, every
publish, even an empty one, still rewrites the accounting row for every
row that has ever existed, together with the accounting tree's own
internal nodes, allocation records, mapping entries, and the tree table
unit; specifically, during the warm up publishes of the very first
writable mount of a freshly made filesystem, at the point where the tree
table still has zero entries, this means zero units are written; every
empty publish after that point writes however many blocks a separately
defined, freshly recalculated capacity value allows for at that time. The
scope paragraph states plainly that this decision text only decides
whether an empty publish writes units and which ones, and explicitly does
not decide how that separately defined capacity value is itself
calculated, and does not decide when an empty publish is triggered in the
first place.

Fact 20. An observation about the decision text quoted in fact 19,
considered against the exact condition in fact 17, from a careful reading
of both together. The decision text in fact 19 addresses one single
concrete situation where the tree table has zero entries: the warm up
publishes that happen during the very first writable mount of a freshly
made filesystem, where the tree table was never anything but empty from
the start. It says nothing about the different situation in fact 17,
where a root points to a tree table with zero entries whose own birth
transaction number is not the fixed generation number filesystem creation
uses, meaning this version of the tree table did not originate at
filesystem creation time at all.

Judgment 1. This judgment is about the refusal described in facts 3
through 5. Fill in the four part table described in the background
section above: part one, has anything been written before this refusal
fires; part two, is there a zero fault reachable history among the family
of histories this refusal blocks; part three, does the error name match
the actual condition being checked; part four, should this gap become a
new clause, be registered as unsupported for the first version, or is it
already covered by an existing clause. For part four, consider that fact
5 states no covering clause was found for this specific check.

Judgment 2. This judgment is about the refusal described in facts 6
through 12. Fill in the same four part table. For part two, consider
carefully whether a single transient read error, as described in fact 8,
counts as a fault by the standard given in fact 1, or whether it should
be counted as an ordinary condition that can occur during zero fault use.
For part four, weigh fact 10's closed ledger entry and fact 11's quoted
decision text against fact 12's observation that the text does not
describe this exact two step recompute and compare mechanism, and decide
whether this specific mechanism is already covered by an existing clause,
needs its own new clause, or should be registered as unsupported.

Judgment 3. This judgment is about the refusal described in facts 13
through 16. Fill in the same four part table. For part two, consider fact
14's statement that the region assignment field has no other gate in
front of it besides this one refusal, together with fact 16's account of
why this refusal was added. For part four, weigh fact 15's own stated
justification, that a caller chosen assignment would only buy the ability
to choose which single device carries two regions and that this choice
has no observable difference with exactly two devices, against fact two's
own criterion for when a gap needs a new clause; also take into account
that fact 15 states the second user decision behind this specific fixed
pattern was never put through the same three way review this document
belongs to and is still open to being overturned.

Judgment 4. This judgment is about the refusal described in facts 17
through 20. Fill in the same four part table. For part two, consider
fact 18's own documentation comment stating that this implementation
itself cannot produce a root meeting this condition, and that only a
corrupted device or some other piece of software writing to the same on
disk format could. For part four, weigh fact 19's decision text and fact
20's observation that this decision text addresses a different concrete
situation than the one in fact 17, and decide whether an existing clause
already covers fact 17's condition, whether a new clause is needed, or
whether this should be registered as unsupported for the first version.

