SINGLEFS ROOT RING FLOOR ARITHMETIC - LOCAL ATTACK LEG - ROUND m2-rollback-forward-r3 - PART 1 OF 2

You are one of several independent reviewers examining a design proposal for
singlefs, a copy on write filesystem being designed from scratch. Your only
job in this document is arithmetic: given a fixed set of numbered facts below,
taken directly from the project's own decision records and from this design
round's own working notes, work out three numbered items, each asking you to
compute specific numbers by a specific, fully stated procedure. This is not a
design review and you are not being asked whether any of these three
candidate designs (called MAX, MAX+HOLD, and SYSCFG below) is a good choice;
you are only being asked to compute numbers correctly from the facts given,
and to say plainly if any two facts, or any fact and any item's stated
procedure, do not fit together.

Do not assume anything not stated in the facts or in the modeling assumption
below. Read every fact carefully; every qualifier is load bearing. Do not use
any markdown emphasis anywhere in your answer: no bold text, no italic text,
no backtick code formatting, no asterisk bullets, no pipe tables. Number your
answers to match the three items given at the end, and within each answer,
number your sub-answers to match the lettered sub-parts asked for in that
item. For every number you report, show the step that produced it, not just
the final value. For every numbered item, end with one sentence starting with
"This would be refuted by:" describing the specific fact you would have had
to misread, or the specific reasoning step you would have had to get wrong,
for your answer to that item to be wrong.

When your answer needs to point at where a number comes from, point at it
only by the fact number given below (fact 1 through fact 6). Do not write any
source file name together with a line number, and do not write any line
number at all anywhere in your answer, because the underlying files may have
moved to different line numbers by the time anyone reads your answer.

Two vocabulary notes before the facts. First, this project keeps its most
recent committed filesystem states in a fixed size circular structure on
disk called the "root ring". The root ring is split into fixed regions; each
region is permanently assigned to exactly one physical disk, decided once
and never changed afterward; each region holds some fixed number of storage
positions called "slots", one persisted root record per slot. Second, every
root record carries an 8 byte field called the "rollback floor F"; the whole
pool's currently governing floor value is called "F effective", and it is
computed from each disk's own current floor value, called "F sub d" for disk
d, by a formula that differs between the three candidate designs below.

Fact 1. This exercise's pool has exactly two disks, named disk 0 and disk 1.
The root ring has three regions, numbered region 0, region 1, and region 2 in
that order. Region 0 is permanently assigned to disk 0. Region 1 is
permanently assigned to disk 1. Region 2 is permanently assigned to disk 0.
Every region holds exactly 8 slots. Source: this design round's own dispatch
note (the row assigning this arithmetic task).

Fact 2. Independently of fact 1's specific two disk scenario, the project's
decision record fixes the root ring's region count at exactly 3 in general;
this 3 is the same 3 used in fact 1 above. Note: the decision record derives
this 3 from an unrelated formula, "region count equals a fault tolerance
target plus 1, with the fault tolerance target equal to 2"; that fault
tolerance target happens to also be called F in the decision record, but it
is a completely different quantity from the rollback floor F described in
the vocabulary note above and used in facts 4 through 6 below. Only the
resulting number 3 is relevant to this exercise. Source: decision record D22
(how unit atomicity is composed), the definition item about the root ring's
parameters.

Fact 3. F sub d is defined as the rollback floor F value carried by the
newest currently persistent, valid root record on disk d. Source: this
design round's own notation list at the top of its working notes.

Fact 4. Every root record carries an 8 byte rollback floor F field. In
ordinary operation this field's value does not change; it is only ever
raised, and only when disk space is tight. It is never automatically lowered
merely by publishing more root records. Source: decision record D22 (how
unit atomicity is composed), the definition item about the root record's
field layout.

Fact 5. Three candidate designs compute F effective from each disk's F sub d
as follows. Design MAX: F effective equals the maximum, taken over both
disks, of F sub d. Design MAX+HOLD: for the purpose of computing F effective,
this design is identical to design MAX; separately, this design also holds
back, after reclaiming it,
any freed slot whose release generation falls after the value that today's
baseline rule would compute (that is, the minimum of F sub d over both
disks) and up to and including the maximum of F sub d over both disks,
releasing it only once this instance's own root has been written to every
disk. Design SYSCFG: F is additionally written into two system configuration
slots on every disk; F effective equals the maximum of two quantities, the
maximum of F sub d over both disks, and the maximum rollback floor F value
that can currently be read back from the system configuration slots across
both disks. Source: this design round's own working notes, the row defining
these three candidate designs.

Fact 6. Every disk keeps exactly two physical system configuration slots.
Under normal conditions, both slots on the same disk hold identical content,
except for one 4 byte field recording that disk's own device number, which
necessarily differs by disk. When reading, the reader selects whichever of a
disk's two slots has a passing checksum and the higher generation number; if
that preferred slot's checksum does not pass, the reader automatically falls
back to the other slot on that same disk. Source: decision record D22 (how
unit atomicity is composed), the definition item about the system
configuration field layout.

Modeling assumption for this exercise, stated here only to make the
questions below well posed; this assumption is not drawn from any document.
Favorable case: on every disk, the single most recently published root
already carries a nonzero rollback floor F, but the very next older root
still persistent and valid on that same disk already carries rollback floor F
equal to 0; likewise, for design SYSCFG only, on every disk, whichever of the
two system configuration slots is currently selected under fact 6's selection
rule already carries a nonzero rollback floor F, but the other slot on that
same disk already carries rollback floor F equal to 0. Unfavorable case: on a
given disk, every slot that disk currently owns (per fact 1, for root slots;
per fact 6, for system configuration slots) already carries the same nonzero
rollback floor F value, so every one of that disk's slots of that kind would
have to be corrupted to remove it.

Item 1. This item is about design MAX only.
(a) Under the favorable case, compute the smallest number of root ring slots
that would have to be corrupted (made unreadable) on disk 0 alone for disk
0's contribution to F effective to stop being a positive number; do the same
for disk 1 alone; then report the combined total across both disks.
(b) Under the unfavorable case, using fact 1's slot counts per disk (disk 0
owns two regions, disk 1 owns one region), compute the largest number of root
ring slots that could have to be corrupted on disk 0 alone, and separately on
disk 1 alone, for the same purpose as part (a); then report the combined
total across both disks.
(c) State explicitly which of part (a)'s total or part (b)'s total answers
the question "what is the least number of root ring slots that must be
corrupted, across the whole pool, for F effective to become exactly 0 under
design MAX" - that is, whether "least number that must be corrupted" means
the smallest total that is sufficient in some possible history (part a), or
the largest total that could be forced by an unfavorable history (part b) -
and explain your choice using fact 5's MAX formula.

Item 2. This item is about design MAX+HOLD only.
First, state in one sentence, quoting the relevant clause of fact 5, whether
design MAX+HOLD's F effective formula is the textually same formula as design
MAX's F effective formula, or a different formula.
Then answer parts (a), (b), and (c) exactly as defined in item 1 above, but
for design MAX+HOLD's F effective formula instead of design MAX's. If your
answer to the first sentence of this item is that the two formulas are the
same, state whether that alone is sufficient reason for your three numeric
answers here to be identical to item 1's three numeric answers, or whether
something about MAX+HOLD's slot holding behavior (described in fact 5) could
still change any of these three numbers; give your reasoning either way.

Item 3. This item is about design SYSCFG only. SYSCFG's F effective formula
(fact 5) has two parts: a root ring part (the maximum of F sub d over both
disks, exactly as in design MAX) and a system configuration part (the maximum
rollback floor F readable from the system configuration slots).
(a) For the root ring part, answer parts (a), (b), and (c) exactly as defined
in item 1 above.
(b) For the system configuration part, using fact 6's two-slots-per-disk
structure and the modeling assumption above: under the favorable case,
compute the smallest number of system configuration slots that would have to
be corrupted on disk 0 alone, and separately on disk 1 alone, for that disk's
contribution to the system configuration part to stop being a positive
number; then report the combined total across both disks. Under the
unfavorable case, do the same computation but for the largest number of
system configuration slots that could have to be corrupted (using fact 6's
exact count of two slots per disk).
(c) Combine your answers: report the smallest possible combined total, root
ring slots plus system configuration slots together, needed for design
SYSCFG's whole F effective formula (fact 5) to become exactly 0 in some
possible history; separately report the largest such combined total that an
unfavorable history could force.
