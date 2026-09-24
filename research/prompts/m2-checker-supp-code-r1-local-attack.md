SINGLEFS FAULT INJECTION SCOPE - LOCAL ATTACK LEG - ROUND m2-checker-supp-code-r1

You are one of three independent reviewers checking a newly written batch of
changes to singlefs, a copy on write filesystem being designed from scratch.
You are given a self contained set of facts below. Do not assume anything
not stated here. Read every fact carefully, every qualifier is load bearing,
a definition with one word removed can flip an answer. Do not use any
markdown emphasis anywhere in your answer, no bold text, no italic text, no
backtick code formatting, no asterisk bullets, no pipe tables. Number your
answers to match the five judgments given at the end. For each judgment
give three separate labeled parts: a verdict stated as an explicit phrase,
not a bare yes or no and not merged with another judgment's verdict; one to
three sentences of justification that name the specific fact numbers your
verdict rests on; and one sentence starting with "This would be refuted
by:" describing the specific observation that would prove that verdict
wrong. When your answer needs to point at a source, point at it only by the
fact number given below, fact 1 through fact 16. Do not write any source
file name together with a line number, and do not write any line number at
all anywhere in your answer, because the underlying files may have moved to
different line numbers by the time anyone reads your answer; if you need to
name a piece of code, name the function by its own name or refer to it by
the fact number instead.

This is a separate, independent document from other documents in this same
review round that ask about other parts of the same batch of changes,
covering a transaction numbering scheme and its downstream consequences.
You are not expected to see those other documents and should not assume
anything about their content.

Background. The pool level checker walks a filesystem image and reports a
verdict for each of a fixed list of invariants: green, red with a reason,
or not applicable with a reason. A separate recovery module decides,
independently of the checker, whether a given image can be mounted at all.
Before the change under review, the checker's own early exit test was
stricter than the recovery module's own mount decision, so the two could
disagree about whether a given image was usable. The change under review
narrows the checker's early exit test to match the recovery module's own
mount decision. Separately, the same batch of changes adds three brand new
invariant judgments to the checker that did not exist before.

Fact 1. Before the change under review, the checker's early exit test was:
if any one device in the pool fails to yield a valid system configuration
from either of its two on disk slots, treat the whole image as unmountable,
report every invariant the checker implements as not applicable, and return
immediately without examining anything else. That same day, the recovery
module's own mount decision had already been written as a weaker test: the
recovery module only fails outright when every device in the pool fails to
yield a valid system configuration from either of its two slots; if even
one device can produce a valid system configuration, recovery proceeds
using that one configuration and mounts normally. Because these two tests
disagreed, an image where exactly one device's system configuration was
fully destroyed would recover and mount successfully, yet the checker would
report every invariant as not applicable and examine nothing. The change
under review narrows the checker's early exit test so that it now matches
the recovery module's own weaker test: the checker now reports every
invariant as not applicable and returns immediately only when every device
in the pool fails to yield a valid system configuration.

Fact 2. The documentation comment attached directly above the checker's own
early exit test, explaining the change. It states: when one device's two
system configuration slots are both invalid, do not exit early, because
recovery will fall back to the copy of the system configuration on another
device and mount the pool normally; exiting early in this situation would
report every invariant as not applicable on an image that is in fact
legitimate and mountable, and a targeted fault that destroys both slots on
exactly one device would let this happen silently today. The same comment
states that one gap is still left open by this change: which device's
system configuration was fully destroyed has no invariant number that
reports it today, and this gap is tracked separately as still owed.

Fact 3. The recovery module's own documentation comment on its
choose_system_configuration function, stating exactly the two ways it can
fail. It can fail because both slots on every device in the pool are
invalid, which returns one named failure kind carrying the identity of the
first such device found. Separately, it can fail because the devices that
each do produce their own valid system configuration disagree with each
other, either in filesystem identifier or in device count, or because the
pool has no devices at all, either of which returns a second, differently
named failure kind carrying no device identity.

Fact 4. The choose_system_configuration function's own control flow, from
direct observation of its source. It loops over every device in the pool.
For a device where neither of its two slots parses into a valid system
configuration, that device is skipped for the rest of the loop, and only
the identity of the very first such skipped device is remembered; the loop
continues on to the remaining devices regardless. Only after the loop over
every device has finished does the function check whether it ever found
even one device with a valid configuration: if it did, it returns success
using that one configuration, regardless of how many other devices had
none at all. Only if the loop never found even one valid device across the
whole pool does the function return the first named failure kind from fact
3, carrying the identity of the first invalid device it had remembered.

Fact 5. The same function's disagreement check, from direct observation.
Each time a device does produce a valid system configuration during the
loop in fact 4, the function compares it against the first valid
configuration it already found earlier in the same loop, if any: if the two
disagree in filesystem identifier, or disagree in device count, the
function returns the second named failure kind from fact 3 immediately, and
does not finish looping over the remaining devices.

Fact 6. The checker's own filesystem identifier check, from direct
observation of its source, which runs immediately after the early exit test
in fact 1 does not trigger. The checker first builds its own list of
devices, keeping only the devices that each did yield a valid system
configuration; a device that yielded no valid configuration on either slot
is not a member of this list at all. The checker then picks the geometry
carried by the first device in this list, and loops over exactly this same
list, judging one invariant for each member by comparing that member's own
filesystem identifier field against the filesystem identifier field carried
by the first device in the list. Nothing in this loop, and nothing anywhere
else the checker judges, ever reads or compares a device count field
carried inside a system configuration.

Fact 7. A separate invariant judging function inside the checker, about
instance numbers, from direct observation of its own documentation comment
and its source. For each device in the pool, independently of the list
built in fact 6, it computes the highest instance number found among that
device's own self certifying slots whose filesystem identifier matches the
pool's; if this per device computation yields nothing at all for even one
device in the pool, the whole function immediately reports its own
invariant as not applicable, and its own documentation comment gives the
stated reason: whether that device belongs to this pool at all is left for
the filesystem identifier check named in fact 6 to decide.

Fact 8. A test that measures how many invariants become newly not
applicable, from direct observation of its own source and its own
assertions. It runs the checker once on a clean image with several devices,
and once again after deliberately corrupting both system configuration
slots on exactly one of those devices, leaving every other device
untouched. It asserts that the total count of invariants reported stays
identical across both runs. It asserts that after the corruption, not every
invariant is reported not applicable. It then computes the set of
invariants that were judged, meaning reported as something other than not
applicable, on the clean image, and that became not applicable after the
corruption; it asserts that this set contains at most one invariant. Its
own failure message describes this bound as: corrupting one device's system
configuration may turn at most one invariant from judged into not judged.

Fact 9. The invariant named in the project's own invariant registry as
governing root ring health, stated in full. Besides the record carrying the
single highest generation number among the self certifying root records,
at least one record carrying a strictly earlier generation number must
still exist; if none exists, judge it red, because it means some publish
overwrote the previous generation outright and the next tear would have
nowhere to roll back to. There is exactly one exception: when every self
certifying root record present is generation zero, which is the generation
a fresh filesystem itself seeds into every region at creation time, judge
it green; an image that crashed and rolled back into this same all zero
state is likewise judged green.

Fact 10. The checker's own implementation of the invariant from fact 9,
from direct observation of its source and its own documentation comment. It
computes the highest generation number among the self certifying root
records it has on hand. It computes whether at least one of those records
carries a strictly lower generation number than that highest one.
Separately, it computes whether every one of those records carries
generation number zero. It judges the invariant green exactly when at least
one of these two computed conditions holds, red otherwise. Its own
documentation comment states that this exception is copied over unchanged
from the registry's own wording, and that the registry's further sentence
about a crashed and rolled back image being likewise judged green is
satisfied by this same code path, because the judgment only ever reads the
generation numbers present, never how the image reached that state.

Fact 11. The invariant named in the project's own invariant registry as
governing the journal's back chain algorithm, stated in full. For any
journal record, its back chain field must equal a checksum computed over
the 307 byte header of the logically previous record within the same
instance, with that previous record's own header checksum field treated as
all zero bytes for purposes of this computation; the first record ever
written by a given instance always has back chain equal to zero. Because
the chain is computed per instance, this invariant is never judged across
an instance boundary between two records at all; the instance boundary
itself is already guarded by a separate invariant that checks instance
numbers directly.

Fact 12. The checker's own implementation of the invariant from fact 11,
from direct observation of its source and its own documentation comment.
For a given record at a given counter position on a given device, it
computes what that record's own back chain field is expected to equal,
exhaustively by counter position. If the counter position is exactly the
very first position in the whole pool, the expected value is zero.
Otherwise it looks at whichever record sits exactly one counter position
earlier on the same device. If that earlier record exists and carries the
same instance number as the current record, the expected value is the
checksum computed from that earlier record's own header. If that earlier
record exists but carries a different instance number than the current
record, the current record is, by the registry's own separate numbering
rule for instances, necessarily the first record ever written by its own
instance, and the expected value is set to zero for that stated reason. If
that earlier record cannot be found on disk at all, this invariant is
skipped for the current record entirely. The function's own documentation
comment states explicitly, for the case where the earlier record carries a
different instance number, that the two records' own chain values are not
being compared against each other, and that the registry itself states
this invariant is not judged in that situation.

Fact 13. The invariant named in the project's own invariant registry as
governing total ordering after merging versions, stated in full so far as
it concerns grouping and ordering. Readable, published units of the two
content carrying unit classes are merged into groups keyed by all of the
fields making up each unit class's own class identity segment, together
with that unit class's own write order field. Among groups that share the
same key, the total order keys computed for those groups must be pairwise
distinct; for one of the two unit classes this total order key is the
triple of birth generation number, instance number, and transaction number,
with birth generation number placed first in the triple. Separately, and
independently of the ordering requirement, all members inside one merged
group must carry an identical payload, checked by a payload checksum field
for one unit class and by a payload CRC field for the other; if they
differ, the whole group is judged corrupted and none of its members are
selected.

Fact 14. The checker's own implementation of the merge key from fact 13,
from direct observation of its source and its own documentation comment.
The merge key it actually builds is stated in its own comment as all of the
fields making up the class identity segment together with the write order
field, with the payload related field, payload CRC for one unit class and
payload checksum for the other, removed from that combination before
building the key. Its own documentation comment gives the stated reason:
the payload related field is exactly the quantity that the separate
identical payload check in fact 13 is supposed to compare, so if that field
stayed inside the merge key, two members would only ever land in the same
merged group when their payload related fields already agreed, which would
make the identical payload check trivially always true and unable to ever
catch a real difference.

Fact 15. The project's own field table defining the class identity segment
for the first of the two content carrying unit classes referenced in fact
13, from the project's own decision record for what each unit type carries
in its header. This table lists exactly five fields as making up this unit
class's own class identity segment: a five field tuple, a birth generation
number field, a filesystem identifier field, a write order field, and a
payload CRC field. The payload CRC field is listed as the fifth and last
field inside this same segment in this same table, with its own stated
purpose in that table being to let two copies of the same content be
compared for an identical payload.

Fact 16. The project's own field table defining the class identity segment
for the second of the two content carrying unit classes referenced in fact
13, from the same decision record referenced in fact 15. This table lists
twelve fields as making up this unit class's own class identity segment, in
byte offset order. Among those twelve fields, a payload checksum field is
listed between a birth generation number field and a write order field, at
a byte offset the table gives explicitly, with its own stated purpose in
that table being: since the whole unit's own authoritative checksum lives
in a parent pointer that does not exist yet during scan based
reconstruction, and the header checksum by itself only defends the header,
this field is what lets corruption specifically inside the packed payload
area be detected during that reconstruction.

Judgment 1. Considering facts 1, 2, 6, 7, and 8 together, on the specific
image fact 8 describes, where exactly one device's two system
configuration slots are both destroyed and every other device is
untouched: which single invariant, among everything the checker judges, is
the one invariant that fact 8's own assertion permits to newly become not
applicable on this image. Name it using whatever identifying description
you can construct from facts 1 through 16, or state that the facts given do
not let you identify it uniquely. Then state a second, separate part of
your verdict: is it correct and consistent, given fact 7's own stated
reason for reporting not applicable, that this invariant cannot be judged
at all on this image, or does fact 6 show that the check fact 7's own
comment says it is deferring to never actually runs for the device with no
valid configuration, making fact 7's stated reason for its own not
applicable verdict unsound. Give one explicit verdict covering both parts,
not a bare yes or no.

Judgment 2. Consider two candidate images, each with exactly two devices.
In candidate image A, each device's own system configuration slots parse
successfully and self certify on their own terms, but the two devices'
resulting configurations carry two different filesystem identifier values.
In candidate image B, each device's own system configuration slots
likewise parse successfully and self certify, and both carry the identical
filesystem identifier, but the two devices' resulting configurations carry
two different device count values. Considering facts 1, 3, 4, 5, and 6, for
each candidate image separately, state whether the checker's own early exit
test from fact 1 would trigger on it, and separately state whether
choose_system_configuration from facts 3 through 5 would return success or
would return its second named failure kind on it. Give one explicit verdict
choosing exactly one of: both candidate images make the checker's early
exit test and choose_system_configuration disagree about whether the pool
can be mounted; only candidate image A makes them disagree; only candidate
image B makes them disagree; neither candidate image makes them disagree.

Judgment 3. Considering fact 9's exact wording against fact 10's
implementation, does fact 10's implementation match fact 9's exception
clause word for word, does it add a condition fact 9 does not state, or
does it omit a qualifier fact 9 does state. Give one explicit verdict
naming which of these three holds, or a different explicit verdict if none
of them fit, and if you name an added condition or an omitted qualifier,
quote the specific words from fact 9 involved.

Judgment 4. Fact 11 states two separate rules: the first record written by
a given instance always has back chain zero, and this invariant is never
judged across an instance boundary between two records. Fact 12 describes
one specific case, where the record immediately before the current one in
counter order exists but carries a different instance number, and in that
case the implementation sets the current record's own expected back chain
to zero. Considering fact 11's exact wording, does this specific case from
fact 12 correctly fall under only the first rule, correctly fall under only
the second rule, incorrectly invoke both rules at once, or incorrectly
invoke neither rule. Give one explicit verdict naming which of these four
holds.

Judgment 5. Fact 13 states the merge key is built from all of the fields
making up the class identity segment together with the write order field.
Fact 14 states the implementation removes the payload related field from
that same combination before building the key. Facts 15 and 16 each list a
payload related field as one of the fields belonging to that same unit
class's own class identity segment, according to the project's own field
table for that segment. Considering facts 13 through 16 together, does the
implementation's merge key match fact 13's own stated definition of the
merge key word for word, or does removing the payload related field
described in fact 14 mean the implementation's merge key omits a field that
facts 15 and 16 count as belonging to the class identity segment. Give one
explicit verdict, and state whether your verdict changes depending on
whether the payload related field is treated as inside or outside the
class identity segment for purposes of fact 13's own wording.
