DEFENSE ROUND. You are the defense side in an adversarial code review
exercise for singlefs, a from-scratch copy-on-write filesystem written in
Rust. A change just landed in the pool level checker, a separate crate
that reads a raw on-disk image and judges a fixed list of consistency
invariants. This change wires up two of those invariants, named I-7.4 and
I-4.8, using a mechanism that walks every root in something called the
rollback candidate set, and compares checksum-mismatch and
unreadable-block counts recorded right before and right after each root's
own walk. Your job is to defend three specific mechanism choices this
change makes.

Do not use any markdown emphasis (no asterisks, no bold, no italics, no
headers with number signs). Answer in plain numbered prose. Do not use
the Rust path separator two colons in your answer; refer to functions by
their plain name only (for example say "the walk_root function", not a
qualified path).

For each of the three items below, do exactly three things, in this
order, and label them exactly as shown:

ITEM N OBJECTION:
First write out, in your own words, the strongest possible version of
the attack question given for that item. Elaborate it. Make it as sharp
and compelling as you can. You may add any additional angle you can
think of beyond what is quoted, as long as it is a genuine weakness and
not a strawman.

ITEM N DEFENSE:
Then answer: does the mechanism hold up against this objection? If yes,
give the specific mechanism (which function, which field, which decided
rule) that resolves it, precisely enough that someone could verify your
claim by reading that function. If it does not hold up, concede clearly
and explain exactly why not, and say what would need to change.

ITEM N REFUTING OBSERVATION:
State one concrete, specific scenario, sequence of events, or state of
the system that would, if actually observed, refute your defense (that
is, prove the objection right). Be specific: name which root, which
device, which slot, which invariant, which test would need to fail.

BACKGROUND FACTS (use these; you cannot read any other file)

Root ring and self-certifying roots. A fixed-size ring of on-disk slots
holds root records. Each root record carries checkpoint_txg (called txg,
a monotonically increasing publish counter) and rollback_floor (called
F). A function named valid_roots collects the currently readable roots by
reading each slot and checking only that root slot's own self-certifying
checksum (source: crates/singlefs-checker/src/image.rs, function
valid_roots, lines 216 to 227); this function does not touch invariant
I-2.1 at all, because a root slot's own self-certifying checksum is a
different mechanism from I-2.1, which is about blocks a root references,
not the root slot itself.

Instance table validity. An on-disk table of rows, one row per instance
that ended. Each row is (instance, T, W): T is the checkpoint_txg of the
root that was selected when that instance ended; W is the highest
transaction number that instance's records were applied up to. A root
carrying (instance i, txg T) is judged valid by the instance table if and
only if the table has no row for instance i, or it has a row (i, Ti, Wi)
with T less than or equal to Ti. A root judged not valid this way is
called abandoned.

F_effective. For each device, take the maximum F carried by any currently
readable root on that device, then take the minimum of those per-device
maxima across all devices.

The rollback candidate set, the decided definition this change
implements (source: .claude/kb/decisions/23-journal的角色与格式.md, line
1209, translated verbatim, the defining sentence for the candidate set
both invariants below are written in terms of): "the candidate set equals
roots in the ring judged still valid by the instance table, and with txg
greater than or equal to F_effective." A second decided source restates
the same candidate set the same way (source:
.claude/kb/decisions/16-发布语义.md, line 376, translated verbatim): "the
rollback candidate set: judged still valid by the instance table, and txg
greater than or equal to F_effective."

By user decision, the floor F is raised only when the pool needs to
reclaim released space, and the raising mechanism is designed, whenever
possible, to keep at least four retrievable states in the candidate set
(source: .claude/kb/decisions/16-发布语义.md, lines 360 to 361, translated
verbatim): "user's exact decided words, 2026-09-13: pick option one, the
four most recent states; this used to be a promise covering every root in
the whole ring, but that promise was later given up." The same source,
line 374, gives the exact ceiling formula for how high F is allowed to
rise, translated: the minimum of (the newest durable valid root on each
device) and (the fourth newest non-empty valid root); if fewer than four
non-empty valid roots exist yet, the ceiling falls back to the oldest
valid root instead. In other words F is never deliberately raised so far
that fewer than four candidate states would remain, except in the
genuinely early life of a pool before four checkpoints have accumulated
yet.

I-7.4, full decided text (source: .claude/kb/invariants.md, line 50,
translated verbatim, defining sentence only): "for every root in the
rollback candidate set (judged still valid by the instance table, and
with txg greater than or equal to F_effective, per decision 23 item 14),
the physical range of every block that root references has neither been
reallocated to another object, nor swept and erased."

I-4.8, full decided text (source: .claude/kb/invariants.md, line 153,
translated verbatim): "after replaying to any crash point, starting from
any one root in the rollback candidate set (the same candidate set as
I-7.4) and walking it, every block's checksum must match what its parent
pointer recorded (the depth of the candidate set is set by the rollback
floor F: ordinarily the whole ring, but shrinking under space pressure to
the most recent four retrievable states, always including the newest
durable valid root on each device). Discriminating power: this is meant
to catch the class of bug where a block released within the current
transaction gets reallocated and written, which makes an old root's own
existing reference now point at new content, turning that old root into
a false rollback candidate; checking only the newest generation's root
alone would never catch this."

I-2.1, decided text (source: .claude/kb/invariants.md, line 107,
translated verbatim): "any referenced block, its checksum matches its
content." As of 2026-09-17 (same source, same line, translated verbatim):
"referenced is now scoped by the same candidate set I-3.1 uses: a block
referenced only by a root below F may already have been reclaimed and
reused, and is not judged on."

Project-wide independence rule for the checker (source:
.claude/kb/verification-build.md, lines 111 to 112, translated verbatim):
"independence is a hard requirement, not a style preference: the checker
is its own crate and does not depend on the implementation crate; it is
not allowed to link the implementation's own accounting and verification
code." The only thing checker and implementation are allowed to share is
one constants module generated from the project's own knowledge base. No
birth-generation field and no owner-object field exists anywhere in the
on-disk position-entry format the checker parses; this absence is a fact
stated directly in this round's own change description (source:
research/prompts/_m2-step6-checker-r1-background.md, line 18, translated
verbatim): "not present in the crates directory: an independent judgment
of erasure separate from the checksum (today it relies purely on
checksum mismatch or an unreadable header); an object-identity based
judgment of reallocated to another object (today it relies purely on the
checksum); any judgment at all of units referenced only by abandoned
roots (they are outside the candidate set; the shadow ledger that tracks
them lives only in memory)." Treat this as a given fact about the current
code, not something you need to verify.

Project-wide traversal rule (source: .claude/rules/fs-design.md, line 24,
translated verbatim, from a table naming which parts of the system are
required to do a full traversal): "checker and audit code: must
traverse; if a runtime decision path also used traversal to compute the
same thing, the checker's traversal and the runtime's traversal would
become the same computation, and the whole point of having an
independent check would drop to zero on the spot."

The clean baseline pool image used by this round's test suite already
carries three self-certifying roots before any mutation is applied
(source: research/prompts/_m2-step6-checker-r1-background.md, line 15,
translated verbatim): "the clean image is the pool from the first
transaction (root txg 1 and 2 are empty warm-up roots, txg 3 is the
write called A); the mkfs-era tree-table unit at slot 50178 was replaced
by A, and only the roots at txg 1 and 2 still reference it."

The Judgements accumulator (source:
crates/singlefs-checker/src/image.rs, struct at lines 44 to 49, methods
at lines 51 to 99). Method judge, lines 53 to 63: every single call
increments an evaluated counter for that invariant by one; if the
boolean argument passed in is false, it also increments a violations
counter for that invariant by one, and, the very first time only,
records a detail string. Method violation_count, lines 67 to 69: returns
the current violations counter for a given invariant, or zero if it has
never been touched; this counter only ever goes up, for the entire
lifetime of one checker run on one image, and it is never reset between
separate calls to judge. Method not_applicable, lines 71 to 77: records a
reason string for an invariant, but only if no reason has been recorded
for that invariant yet; calling it a second time keeps the first reason,
silently. Method into_report, lines 79 to 98: for every invariant in the
implemented list, if that invariant has at least one recorded violation,
the final verdict is Violated, carrying the very first violation's
detail text; otherwise, if the evaluated counter for that invariant is
greater than zero, the final verdict is Holds; otherwise, meaning the
evaluated counter is exactly zero, the final verdict is NotApplicable,
using whichever reason string not_applicable recorded, or a generic
default string if not_applicable was never called for that invariant.
Only the evaluated counter decides between Holds and NotApplicable; once
judge has been called at least one time for a given invariant, that
invariant can never come back as NotApplicable again on this image, no
matter what not_applicable is later told.

Function read_referenced_unit (source:
crates/singlefs-checker/src/image.rs, doc comment at line 300, function
body at lines 301 to 325): for each of the two on-disk locations recorded
for one pointer, it reads the physical bytes at that location, computes
whether their checksum matches the checksum recorded in that specific
location entry, calls judge for I-2.1 with that boolean result (this
happens once per location, unconditionally, every single time this
function is called, regardless of anything else), and returns the bytes
from the first location whose checksum matched, or nothing if neither
location matched.

Function read_index_node (source: crates/singlefs-checker/src/walk.rs,
lines 143 to 194): given a pointer to an index node, it first calls
read_referenced_unit exactly as described above (lines 158 to 164 of the
same file), which unconditionally judges I-2.1 for this specific
pointer's own recorded checksum against whatever is physically on disk
right now; if that call returns nothing (neither location's checksum
matched), a failure string is pushed onto walk_failures and the function
returns immediately. Only after that checksum check has already
succeeded does the function check a field called visited_units, a
BTreeSet of (device, slot) pairs (lines 169 to 174 of the same file): if
this exact (device, slot) pair was already present in the set (inserted
by an earlier call, possibly during an earlier root's walk), the function
returns immediately without decoding the node's entries, without judging
the node's own structural invariants (named I-1.3 and I-1.1 further down
the same function), and without recursing into any of that node's
children. If the pair was not already present, the function proceeds to
decode and structurally judge the node, and the caller then recurses into
its children as normal.

The Walk struct (source: crates/singlefs-checker/src/walk.rs, lines 47 to
65): visited_units is one single field on this struct, a plain BTreeSet;
it is never cleared or reset between calls. The function check_pool_image
(source: same file, lines 690 to 904) constructs exactly one Walk value,
then calls walk_root on it once for the newest root (line 777), and then,
inside a loop over every other root in the ring (lines 798 to 826), calls
walk_root again, on the very same Walk value, once for every root that
satisfies index does not equal newest_index, is not judged abandoned by
the newest root's own instance table, and has txg greater than or equal
to the newest root's own rollback_floor field (this three-part condition
is exactly the candidate set definition above, applied using the newest
root's own view of the instance table and its own F).

The older-candidate loop body (source: same file, lines 798 to 826, exact
code, translated variable and comment text only, code itself left as is):
for each qualifying older root, it reads violation_count for I-2.1 and
the length of walk_failures right before calling walk_root on that root
(mismatches_before, failures_before, lines 806 to 807), calls walk_root
on that specific root (line 808), then reads the same two counters again
right after (lines 811 to 813); if either counter grew during that one
walk, a boolean called walked_into_reused_or_erased_unit is set true. The
code's own comment directly above this (lines 809 to 810, translated
verbatim): "if a unit referenced by this candidate root has a checksum
that does not match, or its header cannot be read, that means the block
it points to has already been reallocated or swept and erased (I-7.4);
the traversal starting from it is then no longer self consistent
(I-4.8); the two invariants are each judged once per candidate root."
Both I-7.4 (lines 815 to 820) and I-4.8 (lines 821 to 824) are then
judged using the exact same boolean, negated; the I-7.4 violation message
(line 818, translated verbatim): "candidate root txg {root_txg}: a unit
it references has already been reallocated or swept and erased (checksum
mismatch or header unreadable)." An older_candidates counter is
incremented once per qualifying root (line 805); if it stays at zero
after the whole loop, meaning zero older roots qualified, the code calls
not_applicable for I-7.4 only, with the reason string, translated
verbatim (line 829): "the rollback candidate set only has the newest
root, there is no older root to judge." I-4.8 is never given this
not_applicable treatment anywhere in this function.

The newest root's own I-4.8 cell (source: same file, lines 776 to 785):
right after walk_root runs once for the newest root, and before the
older-candidate loop begins, the code captures newest_failures as a copy
of walk_failures at that exact point, then judges I-4.8 using the
boolean, translated variable names only: newest_failures is empty and
violation_count for I-2.1 equals exactly zero. I-7.4 is never judged for
the newest root anywhere in this function; the loop condition (line 804)
unconditionally excludes index equal to newest_index
from ever entering the loop body at all. Later in the same function
(line 832), I-7.2, a different, already-existing invariant, is judged
using the same newest_failures.is_empty() value on its own, translated
verbatim comment directly above the whole newest-root walk (line 776):
"walk the newest root first: whether its own traversal breaks is exactly
I-7.2; the accounting rows are only taken from beneath the newest root."

ITEM 1. Judging a candidate root's blocks as reused or erased purely by
whether the running I-2.1 violation count, or the length of
walk_failures, grew during that specific root's own call to walk_root,
using the exact same walk_root function and the exact same shared
visited_units cache already used for the newest root, instead of any
check of object identity or birth generation, and instead of writing a
second, dedicated traversal specifically for I-7.4 and I-4.8.

Attack question (verbatim, source: crates/singlefs-checker/src/walk.rs,
lines 809 to 810 and line 818, translated, describing this mechanism):
the code's own comment states plainly, as settled fact, that a checksum
mismatch or an unreadable header found while walking this specific
candidate root proves the block it points to has been reallocated to
another object or swept and erased; the violation message repeats the
exact same claim word for word. But the only thing actually being
checked, inside read_referenced_unit, is whether the physical bytes
currently on disk still match the checksum this specific candidate
root's own tree recorded for that location. Ordinary bit rot, a
corrupted read path, or any other kind of damage that the allocator
never touched at all would trip that exact same check in that exact same
way. Given that I-2.1 already exists, on its own, specifically to catch
any checksum-versus-content mismatch for any reason whatsoever, what does
this mechanism actually add beyond re-scoping I-2.1 to one particular
root's reachable set and stapling an unverifiable causal label onto
whatever I-2.1 already found? Separately: since visited_units is one
single field on the Walk struct, shared and never reset across the
newest root's walk and every subsequent candidate root's walk in the same
run, and read_index_node returns immediately, without decoding entries
and without recursing into children, the moment a node's (device, slot)
pair was already inserted by an earlier walk in this same run, isn't
reusing walk_root this way, instead of writing a traversal specifically
built for checking a root's own reachable set in isolation, exactly the
kind of shortcut that could silently skip the very re-descent needed to
discover a unit that only this particular older candidate root's own
unique path still reaches, if that older root happens to share an
already-visited ancestor node with the newest root or with an
earlier-processed candidate root in the same loop?

ITEM 2. The newest root gets exactly one standalone cell for I-4.8,
computed with an absolute check, violation_count for I-2.1 equals zero,
evaluated once right after its own walk and before any older root has
been walked at all; I-7.4 is never evaluated for the newest root at all,
anywhere in this function, not even as not_applicable.

Attack question (verbatim, source: crates/singlefs-checker/src/walk.rs,
line 783, together with lines 811 to 813, translated, describing this
mechanism): every older candidate root's I-7.4 and I-4.8 cells are judged
by a delta, the running counters compared before that specific root's own
walk against the same counters after it. The newest root, walked using
the exact same walk_root function, gets an absolute check instead,
violation_count for I-2.1 equals exactly zero, evaluated once, with
nothing measured before it to take a delta against. On what principled
basis does the newest root's own I-4.8 cell get computed by a different
formula than every other root's, given that it goes through the identical
walk_root call? And separately, and more sharply: I-7.4's own decided
definition says, without exception, that it applies to every root in the
rollback candidate set; the newest root's own txg is by construction
always the maximum among all roots in the ring, and is therefore always
greater than or equal to F_effective, so the newest root is itself
ordinarily a full member of that same candidate set the invariant is
written in terms of. Yet the code excludes it from I-7.4 by construction,
silently, through a loop condition, with no not_applicable call, no
comment justifying the exclusion anywhere near that loop condition, and
no test anywhere in this round's twenty six case known-bad-image suite
that exercises what happens when the newest root's own referenced blocks
themselves fail this exact check. If I-7.4's own words say every root in
the candidate set, on what authority does the implementation quietly
narrow that to every root in the candidate set except the one that is
walked first?

ITEM 3. When the older-candidate loop finds zero qualifying roots, the
checker reports I-7.4 as NotApplicable, with the reason string the
rollback candidate set only has the newest root, there is no older root
to judge, rather than reporting Holds or Violated.

Attack question (verbatim, source: crates/singlefs-checker/src/walk.rs,
line 829, translated, describing this mechanism, together with a fact
about the test suite): first, is NotApplicable really the correct
verdict here, rather than Holds? There are exactly zero older candidate
roots in this case, so the universally quantified claim, every root in
the candidate set has not been reallocated or erased, is vacuously true
over an empty set of older roots to check, which is the ordinary reading
of a for-every statement ranging over nothing. Or, going the other
direction, given that a healthy pool is specifically designed, through
the floor-raising ceiling formula, to keep at least four retrievable
states in the candidate set whenever four exist yet, isn't a candidate
set that has collapsed down to exactly one root itself an unusual,
possibly degraded condition that an operator ought to be told about
directly, rather than one that gets folded into a routine, easy to
overlook NotApplicable line among twenty six other invariants? Second,
and more concretely verifiable: the project's own test suite (source:
crates/singlefs-harness/tests/checker_known_bad_images.rs, lines 544 to
549) asserts, for every single one of the twenty six implemented
invariants, that the clean baseline image used as the starting point for
every one of this round's known-bad-image mutations returns exactly
Holds, never NotApplicable; and that same clean baseline image already
carries three self-certifying roots (background fact above, from line
15). That means the NotApplicable branch this change adds has never
actually been observed to fire, correctly or otherwise, by any test that
exists anywhere in this repository right now. Given that a negative
result must be distinguishable from code that simply never ran, how is
anyone supposed to know this branch produces the verdict it claims to
produce, on any real image, rather than, for instance, silently never
being reachable at all, or producing some other verdict due to a mistake
in how evaluated and not_applicable interact inside into_report?

CLOSING INSTRUCTIONS

Answer items 1 through 3 in order, using the exact three labels given at
the top (ITEM N OBJECTION, ITEM N DEFENSE, ITEM N REFUTING OBSERVATION)
for each item. Do not summarize all three items at the end. Do not add a
general conclusion beyond the three items. If you genuinely cannot
construct a convincing defense for some item, concede plainly instead of
forcing one, but try hard first: these are real, carefully sourced
weaknesses being raised against a real, just-landed piece of code, not
hypothetical concerns.
