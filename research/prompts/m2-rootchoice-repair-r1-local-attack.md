SINGLEFS ROOT CHOICE, ROLLBACK, SWITCH AND REPAIR - LOCAL ATTACK LEG - ROUND m2-rootchoice-repair-r1 - DOCUMENTS K4 AND K6

You are one of three independent reviewers of a set of design questions for
singlefs, a copy on write filesystem being designed from scratch. You are
given a self contained set of numbered facts below. Do not assume anything
not stated in one of these facts. Read every fact carefully, every
qualifier is load bearing, a definition with one word removed can flip an
answer. Do not use any markdown emphasis anywhere in your answer: no bold
text, no italic text, no backtick code formatting, no asterisk bullets, no
pipe tables. Number your answers to match the judgments given below.
Whenever your answer needs to point at a source, point at it only by the
fact number given below, or by the candidate label given below, fact 1
through fact 16, or a label such as 2a or 5. Do not write any file name
together with a line number, and do not write any line number at all
anywhere in your answer, because the underlying files may have moved to
different line numbers by the time anyone reads your answer; if you need
to point at a piece of code, name the function by its name instead.

This single document covers two separate tasks, called K4 and K6 below.
Answer both. They are two other reviewers' separate documents in this same
round, called K1, K2, K3 and K5, that ask about different parts of the
same overall design question; you are not expected to see those other
documents and should not assume anything about their content beyond what
is repeated for you in the facts below.

Background. Singlefs keeps a small fixed size ring of on disk slots called
the root ring, each slot able to hold one self verifying root record; on
every publish a new root record is written to the next slot in rotation.
Singlefs also keeps an on disk instance table, a chain of pages of fixed
width rows, one possible row per past running instance of the filesystem,
used during recovery to decide which transactions have already taken
effect. A mount can be writable or forced read only; being forced read
only is meant to be a temporary, recoverable state, and a design in which
a filesystem can enter a state it can never leave without outside help
even though every individual physical device keeps working is called
here an absorbing state, and is treated as a defect if it appears with
no accepted decision covering it.

Fact 1. A row from the project's own list of known but not yet enforced
checks, tracking id C335, short name a persistently unreadable root ring
slot makes the instance table only grow and never shrink. Its definition
column, quoted in full: on a fixed date the project's human owner chose
the strict form of a recycle condition, this recycle condition was
defined for a different, separately tracked issue about a missing check
on an empty publish that should have been pushed before writing an
instance table row: an instance table row may be deleted only once every
single slot in the root ring has been read successfully, and only that
state of affairs counts as there being no root published by that instance
anywhere in the root ring. When a write to a root ring slot fails,
checkpoint_txg is advanced by one before the write is retried, and the
retry lands on a different slot, so the slot that just failed is skipped
for that attempt; this behavior is fixed by an established decision item
about the journal's role and format. The first shippable version of this
filesystem has no mechanism at all to relocate a root ring slot to a
different physical location. Given both of these two facts together, once
one specific root ring slot becomes persistently unreadable, it stays
unreadable in every subsequent attempt, forever. Separately: every
writable mount writes at least one new row into the instance table, and a
row can never be deleted while the recycle condition above cannot be
satisfied for it; therefore the row count, called rows0, keeps growing,
and every time rows0 passes another multiple of 369 rows, the reserve an
instance switch needs grows by the cost of one more shard of the chain,
which on the first transaction's on disk geometry is 8 more blocks per
physical device for each such increase; once rows0 has grown enough, the
switch reserve eventually becomes unobtainable; once the switch reserve is
unobtainable, the mount is forced read only; once the mount is read only,
a row can no longer be deleted, because deleting a row is itself a write
and requires a writable mount; this closes a loop with no exit, called an
absorbing state here. Unlike a separately documented absorbing state that
only appears when the pool is nearly full, this one does not require the
pool to be nearly full at all, a point established by a third round
defending party. A second, alternative design, written down and fixed
before this round of review started, under which a slot that stays
unreadable simply keeps being skipped until its turn comes around again to
be overwritten, and is otherwise treated purely as a matter for whoever
administers the physical devices, was not the option the project's human
owner chose. Its prerequisites column, quoted in full: there is no
decision item at all covering what should happen when a root ring slot
fails to read persistently, meaning neither relocating it nor treating it
purely as a matter for whoever administers the physical devices has
actually been decided.

Fact 2. From an established decision item about what information a unit
carries, quoted in full: how a row gets written: keyed uniquely by
instance number, a later write overwrites an earlier one, and a row is
only ever written during recovery, during an in mount instance switch, or
during an administrator triggered rollback, as part of the very same
publish as the first new root; every writable mount writes a row (instance
zero never gets one; the publish that writes the row is the new
instance's very first publish, and by the time any root belonging to the
new instance is published, the instance table it points at already
contains rows for every instance from the greater of the selected root's
instance and one, up to but excluding the new instance; no empty
publish meant to raise the rollback floor is pushed before the row is
written, and the metadata cost of writing the row itself is paid out of
the reserve set aside for instance switches); every time a row is
written, the copy on
write mechanism rewrites the entire chain of table pages.

Fact 3. From the same established decision item, its rule for when a row
may be recycled, quoted in full: a row may be deleted if and only if one
complete sweep has reached a definite verdict for every single location in
the whole pool, read successfully and judged by its role, or found
already erased, or found already overwritten, and none of those locations
turned out to hold an unpublished unit belonging to that instance, and not
one single read failed anywhere during that sweep, a location that cannot
be read does not count as proven absent, this is a sector level concern,
and every physical device was online for the whole sweep, this is a
device level concern, and there is no root published by that instance
anywhere in the root ring, where every single slot in the root ring must
be read successfully before it counts as there being none, a slot that
cannot be read is treated as though it might still hold a root of that
instance; one root ring slot that is only temporarily unreadable can still
let the row be deleted once the slot becomes readable again, at which
point a root that had been abandoned can come back into the set of roots
an administrator could roll back to, even though the units it referenced
have already been swept and erased; the cost of this is that when a root
ring slot stays unreadable permanently, its row can never be deleted.

Fact 4. From an established decision item about the journal's role and
format, quoted in full: when a write to a root ring slot fails,
checkpoint_txg is advanced by one before it is republished, the root
ring's region is computed as txg modulo R, an already established
parameter; without advancing, a retry would just rewrite the very same
slot again and turn R separate failure domains into effectively one; as a
direct consequence, the same root ring slot failing on consecutive
attempts stops being a meaningful signal, because which slot gets tried
next rotates with txg, and once the number of allowed consecutive
instance switches is less than or equal to R, that particular slot can
never even be reached twice in a row; keeping that as a check would look
like a safeguard while actually catching nothing.

Fact 5. From an established decision item about what a mount commits to
for its own duration, its formula for rewriting the instance table chain,
quoted in full: number of mirror copies times 32768 times whichever is
larger of one, or the ceiling of rows0 plus N_switch divided by rows per
shard, all in bytes; number of
mirror copies equals 2; rows per shard equals the floor of 32768 minus 135
divided by the width of one row, minus 1, because the chain pointer
record always occupies the last slot of a shard; the width of one row is
88 bytes, which makes rows per shard equal to 369; rows0 is the row count
read at mount time plus however many rows this mount's own write will
add, and every writable mount writes at least one row.

Fact 6. From the same established decision item, a sentence from its
stated scope, quoted in full: on a pool that is close to full and also
has bad sectors, the state where the reserve cannot be obtained and the
mount is therefore forced read only may become an absorbing state, this
was inferred rather than measured, and it happens because every 369 rows
costs one more shard; separately, because every writable mount writes a
row, and a row can only be recycled once every single slot in the root
ring has been read successfully, a root ring slot that stays permanently
unreadable makes the instance table only grow and never shrink, and this
second absorbing state no longer requires the pool to be close to full at
all, this is the same tracked issue as fact 1 above.

Fact 7. From an established decision item about the RAID striping
strategy, its table of when a mount is forced read only, quoted in full
for both of its two rows: at the moment of mount admission, the
conjunction that must all hold is: the count of writable devices is at or
above the floor labeled w, and this mount has exclusively opened a strict
majority of the devices in the pool, a strict majority is only a
threshold, the new instance number still has to be written into every
single device this mount exclusively opened, not merely the majority, and
the instance table can be read, and the reserve an instance switch would
need can actually be obtained, the formula for that reserve is fact 5
above; if even one of these fails to hold, the mount is forced read only.
During normal operation, the same forced read only outcome is triggered
the moment the count of writable devices falls below the floor w, or the
moment the count of devices this mount still has exclusively open falls
below a strict majority.

Fact 8. A compressed summary, not a full quotation, of a different
tracked issue, tracking id C331, short name choosing the newest root can
override writes already confirmed to the caller: when every root
belonging to an older running instance is temporarily unreadable, a newer
instance's very first publish counts checkpoint_txg upward starting from
an even older root; if those temporarily unreadable roots become readable
again later, their higher checkpoint_txg wins root selection on the next
mount, silently overriding writes the newer instance had already
confirmed to its caller. Three candidate fixes are named for this issue,
none of which has completed a full three way review yet: candidate one,
restart the running counter from one more than the largest value found
anywhere in the whole root ring; candidate two, have the small on disk
system configuration structure carry its own txg value; candidate three,
use a low water mark computed by scanning records. A later note about
candidate one specifically states that, tested only on the attacking
party's own model, in a repeated cycle of mount, then a warm up record
landing on disk, then a crash before a root, candidate one overwrites a
record that a normal recovery still needed to replay, on the attempt
numbered one less than the length of the whole ring.

Fact 9. From the same established decision item about the journal's role
and format as fact 4, one of its numbered footnotes, quoted in full: the
running counter keeps counting across the whole pool: a new instance
continues writing immediately after the end of the previous prefix, it is
never reset to zero. The fixed size ring's slot for a given record is
decided by its sequence number; an earlier proposal to decouple slot
position from sequence number was already rejected elsewhere. Resetting
to zero would make a new instance's records land on slots the old prefix
still needs. checkpoint_txg works the same way: the very first publish of
any new instance, whether from a normal mount, an instance switch, or an
administrator triggered rollback, must use a checkpoint_txg at least one
more than the maximum of every txg among every root record in the whole
root ring and every checkpoint_txg among every self verifying record
anywhere in the ring that passes its own checksum; a normal mount and a
rollback both already scan the whole ring, an instance switch does not
rescan because its in memory txg is already no smaller than everything
this mount has already seen. This is stated to be the exact same quantity
as the low water mark candidate fix named in fact 8 above, and the two
are meant to be written up together once that issue is closed.

Fact 10. From an established decision item about publish semantics, the
first of its named riders, quoted in full: a root record must carry a 32
bit instance number, the very same counter as the journal's own instance
number from a separate established decision item, and whenever two roots
tie on checkpoint_txg, the one with the higher instance number wins root
selection; the reason this rider exists is that losing a device and later
regaining it can produce two roots that each pass their own self
verification and share the same txg, and the root ring by itself has
nothing else to tell them apart, only the journal can distinguish
timelines by instance number.

Fact 11. A compressed summary, not a full quotation, of a different
tracked issue, tracking id C332, short name an administrator triggered
rollback gets silently undone if both of the rollback instance's roots
become unreadable: after an administrator rollback, if the rollback
instance's root is unreadable on both physical devices, today's root
selection does not consult the instance table at all, so it can land back
on a root belonging to the timeline the rollback was meant to abandon, and
the rollback row written to protect against exactly this cannot help,
because the instance table version that root selection lands on for that
older root does not contain it. Three candidates are named for this in
the round's own planning material, none of them decided yet: candidate
one, have root selection also read the instance table; candidate two, do
not have root selection read the instance table, but instead give a
rollback its own separate durable witness recorded on disk; candidate
three, keep today's behavior unchanged and record this combination of two
simultaneous failures as a known, accepted limitation rather than
something the design protects against.

Fact 12. From the same established decision item about the journal's role
and format as fact 4 and fact 9, describing what a rollback leaves behind
on disk if it crashes before finishing, quoted in full: what remains on
disk is the new instance number that was already durably written into the
system configuration before this publish, taking a new instance number is
durable before the publish that uses it, together with whichever units
and records this same publish had already written but which never take
effect; that new number causes the very next attempt to take a number to
skip over it, and whichever number gets skipped this way is given its own
row under the row writing rule from fact 2 above.

Fact 13. A compressed summary, not a full quotation, of a different
tracked issue, tracking id C334, short name the definition of which root
an instance switch is anchored to has never been attacked: an in mount
instance switch is anchored to whichever root the in flight checkpoint
being reissued was itself based on, rows are written to cover only the
half open range from that root's instance up to but excluding the new
instance, and the reissued checkpoint is republished exactly as it was
originally written, with a reissue of an empty checkpoint counting as
zero. This definition has so far only ever been measured against its own
author's model, in four fixed test scenarios, and has never been attacked
by an independent party. Two rejected alternative readings are named
together with a stated bad consequence for each: reading it instead as
whichever readable root a recovery would pick as newest makes three units
abandoned by an earlier rollback get judged as published; reading it
instead as whichever root this mount's own recovery replay landed on
makes three units still referenced by mounted roots get judged as
unpublished during a warm up publish that writes no units of its own.

Fact 14. From the same established decision item about the journal's role
and format as fact 4, fact 9 and fact 12, its definition of an in mount
instance switch, quoted in full: an instance switch counts as one
recovery performed inside a mount: take a new instance number, write a
row, and reissue whichever checkpoint was still in flight; the root this
is anchored to is whichever root the in flight checkpoint being reissued
was itself based on, the last root the previous instance ever published,
if it published one, otherwise the root this mount's own recovery replay
landed on, and if this happens during the very publish that carries out a
rollback, it is the rollback target root itself; rows are written only
for the half open range from that root's instance up to but excluding the
new instance; every instance k whose write ordering was being carried
forward gets one row of the form k, the last txg k ever published, or
zero if it never published one, the highest transaction number among
transactions that were being carried forward and belonged to k;
transactions numbered at or below W are carried forward unchanged,
transactions numbered above W are redone under the new write ordering or
reported as an error to a caller still waiting for a reply, and every
fixed point unit is entirely rewritten under the new instance.

Fact 15. From an established decision item about how a unit's on disk
atomicity is put together, its field table for the system configuration
structure, quoted in full for the relevant parts: the field table totals
481 bytes, living inside a 4096 byte system configuration slot, leaving
3615 bytes unused inside that slot; the fields are grouped by how
changeable they are into four groups: immutable system configuration, 147
plus 114 plus 128 equals 389 bytes, covering the boot header, all of the
day one reserved space for future encryption support, and nearly all of
the geometry section; changeable system configuration, 4 bytes, just the
node size inside the geometry section; tunable runtime configuration, 36
bytes, covering timing and space thresholds an operator can adjust; and a
fourth group explicitly labeled as not configuration at all, called
runtime quantities, totaling 52 bytes, made up of an 8 byte slot
generation number, a 32 byte whole slot checksum, an 8 byte journal tail
pointer, and a 4 byte instance number; this fourth group is described as
holding values that change on every publish and that a user is never able
to set, and whether the journal tail pointer and the instance number, 12
bytes together, even belong inside this structure at all is itself a
separately tracked open question.

Fact 16. A compressed summary, not a full quotation, of a different
tracked issue, tracking id C245, about how often the system configuration
structure actually gets rewritten to disk: the structure is only durably
rewritten once per checkpoint, not once per publish and not once per
fsync; the gap between these two frequencies can be three orders of
magnitude under an fsync heavy workload. A direct consequence stated for
this: any quantity that lives inside the system configuration structure
and that is expected to change with every single publish will instead sit
stale, unchanged, for an entire checkpoint's worth of time between one
durable rewrite and the next.

Document K4. Fact 1 describes a chain of six claims that together explain
why this filesystem, once a single root ring slot becomes persistently
unreadable, can never leave the read only state again even though every
physical device keeps working. The six claims, stated separately here so
you can answer about each one on its own:

Claim A. A root ring slot that becomes persistently unreadable stays
unreadable in every subsequent attempt, forever.

Claim B. Once claim A holds for one specific root ring slot, the recycle
condition can never again be satisfied for whichever instance's row that
slot's root would have to vouch for, so that row can never be deleted.

Claim C. Because every writable mount writes at least one new row, and
because of claim B, the total row count rows0 keeps growing without any
recycling ever happening again for that instance.

Claim D. Because of claim C, the reserve an instance switch needs keeps
growing every 369 rows and eventually becomes unobtainable, without
needing the pool to be nearly full first.

Claim E. Because of claim D, the mount is forced read only.

Claim F. Because of claim E, the very row that is blocking recycling can
never be deleted, since deleting a row itself requires a writable mount;
this is what closes the loop with no exit.

For document K4, answer six judgments, one for each claim above, labeled
judgment A through judgment F. For each judgment, give three separately
labeled parts.

Part one, called grounding: name, using only the fact numbers given
above, exactly which fact or facts make this specific claim true as
stated. If part of the claim rests on something no given fact actually
establishes, say so explicitly and name which fact documents that
absence instead, rather than silently skipping it.

Part two, called break: describe one minimal, concrete change to a
single named fact's content above that would make this specific claim,
and only this specific claim, false. State the change narrowly enough
that it does not simply restate breaking a different claim in this same
list of six. Do not propose, as your answer here, the second alternative
design named in fact 1, the one where an unreadable slot is simply kept
being skipped until its turn comes around again to be overwritten and is
otherwise treated purely as a matter for whoever administers the physical
devices; the project's human owner already declined that option, and if
your break for this claim turns out to be the same design in different
words, say so explicitly instead of proposing it as new.

Part three, called consequence: state which fact given above, if any,
would then directly conflict with, or no longer hold true under, the
change from part two. If no given fact would be contradicted, say so
explicitly rather than leaving the question unanswered; do not simply
skip part three.

Document K6. This document checks whether candidate fixes for four
different tracked issues would conflict with each other if adopted
together. The candidates are named here with short labels; use exactly
these labels in your answer.

Label 2a: restart the running counter from one more than the largest
value found anywhere in the whole root ring, the first candidate for the
issue in fact 8.

Label 2b: have the small on disk system configuration structure carry
its own txg value, the second candidate for the issue in fact 8.

Label 2c: use a low water mark computed by scanning records, the third
candidate for the issue in fact 8, stated in fact 9 to be the same
quantity as checkpoint_txg's own advancement rule.

Label 3a: have root selection also read the instance table, the first
candidate for the issue in fact 11.

Label 3b: give rollback its own separate durable witness recorded on
disk, instead of having root selection read the instance table, the
second candidate for the issue in fact 11.

Label 3c: keep today's behavior unchanged and record the double failure
in fact 11 as a known, accepted limitation, the third candidate for the
issue in fact 11.

Label 5: keep the adopted anchor root definition for an instance switch,
given in fact 13 and fact 14, exactly as documented, with no change.

Label 4: whichever single one of your own six answers to part two, break,
in document K4 above, you judge most likely to write to the same on disk
bytes as the system configuration structure or the root ring, restated
here in one sentence so it can be compared against the other labels below;
name which claim, A through F, it was your answer for.

For document K6, answer eighteen judgments, grouped as follows. Group one,
nine judgments, one for every pairing of a label from 2a, 2b, 2c with a
label from 3a, 3b, 3c: judgment 2a-3a, judgment 2a-3b, judgment 2a-3c,
judgment 2b-3a, judgment 2b-3b, judgment 2b-3c, judgment 2c-3a, judgment
2c-3b, judgment 2c-3c. Group two, three judgments, one for every pairing
of a label from 2a, 2b, 2c with label 5: judgment 2a-5, judgment 2b-5,
judgment 2c-5. Group three, three judgments, one for every pairing of a
label from 3a, 3b, 3c with label 5: judgment 3a-5, judgment 3b-5,
judgment 3c-5. Group four, three judgments, pairing label 4 with label
2b, with label 3b, and with label 5: judgment 4-2b, judgment 4-3b,
judgment 4-5.

For each of the eighteen judgments, give three separately labeled parts.

Part one, called same byte: name the specific on disk field, slot, or
structure that both of the two named changes in this pairing would have
to write to, citing it by fact number, quoting the field name given in
that fact. If no fact given above names a byte, field, or structure that
both changes would have to write to, state plainly that no shared byte is
named in any fact given above, rather than guessing.

Part two, called same invariant: name the specific established fact
number above whose stated truth both of the two named changes in this
pairing depend on for their own correctness. If no fact given above names
such a shared dependency, state plainly that no shared invariant is named
in any fact given above, rather than guessing.

Part three, called verdict: state either a specific conflict, meaning you
can name one exact byte or field from a fact number above where the two
named changes in this pairing would require two different values, or you
can name one exact sentence from a fact number above that adopting one of
the two named changes would make false; or, if you cannot do either of
those two things, state exactly the three words no conflict found, and do
not ever state that there is no conflict, since failing to find one is
not the same thing as proving there is none.

A final reminder on format. Answer document K4 first, in full, with its
six judgments A through F each carrying parts one, two and three; then
answer document K6, in full, with its eighteen judgments each carrying
parts one, two and three. Use plain sentences only, no markdown emphasis
of any kind anywhere in your answer. Do not write any file name or line
number anywhere in your answer.
