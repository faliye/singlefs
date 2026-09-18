DEFENSE ROUND. You are the defense side in an adversarial code-review exercise
for singlefs, a from-scratch copy-on-write filesystem written in Rust. Three
fixes were just landed in the implementation after two earlier attack rounds
found problems in an administrator-rollback and space-reclamation code path.
Your job is to defend each of the three fixes.

Do not use any markdown emphasis (no asterisks, no bold, no italics, no
headers with # symbols). Answer in plain numbered prose. Do not use the Rust
path separator two-colons in your answer; refer to functions and types by
their plain name only (for example say "the isolate function", not a
qualified path).

For each of the three items below, do exactly three things, in this order,
and label them exactly as shown:

ITEM N OBJECTION:
First write out, in your own words, the strongest possible version of the
attack question given for that item. Elaborate it. Make it as sharp and
compelling as you can. You may add any additional angle you can think of
beyond what is quoted, as long as it is a genuine weakness and not a
strawman.

ITEM N DEFENSE:
Then answer: does the fix hold up against this objection? If yes, give the
specific mechanism (which function, which check, which decided rule) that
resolves it, precisely enough that someone could verify your claim by
reading that function. If it does not hold up, concede clearly and explain
exactly why not.

ITEM N REFUTING OBSERVATION:
State one concrete, specific scenario, sequence of events, or state of the
system that would, if actually observed, refute your defense (that is,
prove the objection right). Be specific: name which roots, which slots,
which records, which order of crashes or which floor values would have to
happen.

BACKGROUND FACTS (use these; you cannot read any other file)

Root ring: a fixed-size ring of on-disk slots (24 slots in the first
version: 3 regions of 8 slots each), each slot holding a root record with
fields including: instance (an instance generation number), checkpoint_txg
(called txg, a monotonically increasing publish counter), rollback_floor
(called F, the rollback floor), and pointers to an instance-table unit, a
tree-table unit, and an allocation-record tree.

Instance table: an on-disk table of rows, one row per instance that has
ended (by ordinary recovery, an instance switch, or an administrator
rollback). Each row is (instance, T, W, is_rollback): T is the
checkpoint_txg of the root that was selected when that instance ended; W is
the highest transaction number that instance's records were applied up to.
A root with (instance i, txg T) is judged valid by a given instance table
if and only if that table has no row for instance i, or it has a row (i,
Ti, Wi) with T less than or equal to Ti. A root judged not valid this way
is called abandoned.

F (rollback floor) and F_effective: F is a field on every root record; it
is only ever raised, never lowered, and only when the pool needs to
reclaim released space. F_effective is defined as: for each device, take
the maximum F carried by any currently readable root on that device; then
take the minimum of those per-device maxima across all devices (source:
.claude/kb/decisions/16-发布语义.md, line 375, exact words translated:
"takes effect once every surviving device carries a durable root with the
new F; the value after recovery equals, per surviving device, the maximum
F carried there, then the minimum of those across devices"). The decided
rollback candidate set (source: same file, line 376) is: valid by the
newest root's instance table, and txg greater than or equal to
F_effective. The decided reclaimable predicate (source: same file, line
371) is: released, and release-generation less than or equal to the
maximum of F_effective and the oldest currently-valid root's txg in the
ring.

Shadow ledger, general mechanism: an in-memory-only bookkeeping mechanism,
never persisted to disk. On every mount, whether an ordinary writable
mount or an administrator-rollback mount, and again every time F is
deliberately raised, the code determines which roots currently readable in
the ring are abandoned, per the instance table belonging to whichever root
is currently newest. For every abandoned root, the code attempts to read
that root's own allocation-record tree (by following that root's
tree-table entry to the allocation-record tree, then decoding every entry
stored directly in that tree's root node). The decided rule this exists to
satisfy (source: .claude/kb/decisions/23-journal的角色与格式.md, line 1209,
quoted in full, translated verbatim): "before roots on an abandoned
timeline leave the root ring, the units they reference must not be
reallocated nor have their headers erased -- the allocator additionally
checks the ledger of every readable root in the ring, and isolates only
the slots referenced solely by abandoned roots." A follow-up user decision
on 2026-09-16 (same source, same line) picked what is called the narrow
reading of that clause: read by the grammatical subject "roots on the
abandoned timeline", a slot still referenced by any currently valid root
does not count and should not be isolated, and validity there is judged
purely by the instance table, with no floor condition attached. That exact
decided sentence, translated verbatim: "still referenced by a valid root
does not fall under it".

ITEM 1. G5: exempting only candidate-set roots from shadow-ledger isolation,
not every currently-valid root, and recomputing after every floor raise.

Mechanism: the implementation landed this round (source:
crates/singlefs-core/src/mount.rs, doc comment at lines 201-208, code at
lines 213-258) computes an exempt set as the union of: every record still
unreleased in the ledger of the process's own current, in-flight account,
and every record still unreleased in the ledger of every root that is
simultaneously readable, judged valid by the instance table, and has txg
greater than or equal to F (this three-part test is called the candidate
set elsewhere in the same decided source, at line 1209: "candidate set =
roots in the ring judged still valid by the instance table, and with txg
greater than or equal to F_effective"). Only slots in an abandoned root's
own ledger that are not in that exempt union get isolated. If a candidate
root's own ledger cannot be read, the code treats it as exempting nothing
(so isolation can only grow, never shrink, when evidence is missing). This
whole computation is redone from scratch on every single mount, and redone
a second time, using the freshly-raised F value, immediately before
reclaiming, whenever an administrator deliberately raises F.

The code's own doc comment (source: same file, lines 202-205, translated
verbatim) states plainly that this is a deviation from the user's decided
wording, not a claimed satisfaction of it: "the user's 2026-09-16 decided
narrow-reading wording is: still referenced by a valid root does not fall
under it, with valid judged purely by the instance table. Exempting only
roots in the candidate set (adding txg greater than or equal to F,
recomputed against the new F after every floor raise) is a tightening of
that wording... under the original wording, once F is raised to 11, [an
earlier-valid root] would still exempt two mkfs-era instance-table slots,
which then get reclaimed and handed out while an abandoned root still
references them, violating the main clause of the same decided rule."
Treat this as an honest, already-disclosed fact: the implementer knows
this is not literally what the user wrote, flagged it as a pending
decision point, and shipped this behavior anyway pending the user's
confirmation. Do not treat it as something you need to discover; treat it
as something you must defend or concede.

There is a second, separate mismatch you may also use in your objection.
A different decided source (source: .claude/kb/decisions/28-挂载期承诺量.md,
line 30, translated verbatim) defines the accounting quantity this
mechanism exists to support, called the ninth item, exclusively-owned by
abandoned roots: "the upper bound equals the abandoned root's own
allocated-space statistic minus R_old's own allocated-space statistic",
where R_old means specifically the one particular old root the current
process is mounted onto or was rebuilt from -- not "every root in the
candidate set". The implementation's exempt-set union, though, subtracts
unreleased records from every root in the whole candidate set, which on
an ordinary mount (not immediately following a rollback) can contain many
roots simultaneously, not just one R_old.

Attack question (verbatim, source:
research/prompts/_m2-step45-code-r3-background.md, line 34, describing
this fix, translated): the fix isolates only
slots unreleased in an abandoned root's own ledger that are not also
referenced by any candidate root (readable and valid per the instance
table and txg greater than or equal to F) and not also referenced by the
current account's own unreleased records. If the user's own decided words
say a slot is exempt when it is still referenced by "a valid root", full
stop, and separately say validity there is judged purely by the instance
table, then on what basis does the code silently add an extra requirement
(txg greater than or equal to F) before a root's protection counts, given
that the project's own rules say an implementer who finds a decision's
literal wording produces a bad outcome must flag it and get the user to
re-decide, not quietly redefine the word "valid" into whatever avoids the
bad outcome? And separately: given that the only decided formula for this
exact mechanism (the ninth item) is written specifically in terms of one
root, R_old, not the whole candidate set, what justifies generalizing
"R_old" into "every root currently in the candidate set" when computing
which slots to exempt, when no decided text says the exemption is supposed
to track the whole candidate set rather than just the one root the mount
is actually built from?

ITEM 2. Skipping and counting an abandoned root whose own ledger cannot be
read, instead of refusing to mount.

Mechanism: inside the same isolation function described above (source:
crates/singlefs-core/src/mount.rs, lines 241-245), for every root judged
abandoned, the code tries to read that root's own allocation-record tree.
If that read fails for any reason (the tree-table unit is corrupt, or the
allocation-record tree it points to is corrupt), the code does not
isolate any slot for that root at all; it increments an in-memory counter
and moves on to the next abandoned root, and the mount proceeds normally.
That counter is exposed on the mount's output (source: same file, doc
comment at lines 99-101, field named abandoned_roots_unreadable), but
today, by the implementation's own admission (source:
research/prompts/_m2-step45-code-r3-background.md, line 26), "today has no
consumer: only test code reads it" -- there is no log line, no health
check, no operator-visible surface of any kind. Before this round, the
same failure (an unreadable abandoned root's ledger) caused the whole
mount to error out and refuse to open, because the code used the question
mark operator to propagate that read failure straight out of the
surrounding function.

Attack question (verbatim, source:
research/prompts/_m2-step45-code-r3-background.md, line 35, describing
this fix, translated): "the tree table or allocation-record tree of an
abandoned root cannot be read or decoded: skip that root, count it, do not
refuse the mount; the slots it references are not covered by isolation."
The decided rule this whole mechanism exists to satisfy says, in full and
without qualification (source:
.claude/kb/decisions/23-journal的角色与格式.md, line 1209, translated
verbatim): "before roots on an abandoned timeline leave the root ring, the
units they reference must not be reallocated nor have their headers
erased." That sentence contains no exception for the case where the
evidence needed to know which units those are has itself decayed. If
anything, an abandoned root whose own ledger is torn is the single
hardest case to protect correctly, precisely because the system has lost
the information needed to honor the guarantee -- yet the implementation's
response to losing that information is to silently give up on this one
root's protection and continue as if nothing happened, recording the fact
nowhere an operator would ever see it. The project's own stated pattern
elsewhere, for exactly this kind of situation (an incompatible or
unverifiable on-disk state), is to refuse to proceed rather than guess: an
unrecognized incompatible feature bit makes the whole pool refuse to
mount, specifically so that misuse becomes impossible rather than merely
invisible. Why is silently forfeiting an unconditional, previously-stated
guarantee, with no operator-visible trace, the right choice here, instead
of refusing the mount and forcing a human to notice and decide what to do
about the one corrupt abandoned root, exactly as an unrecognized
incompatible feature bit would force a human to notice?

ITEM 3. Reclaiming (updating the accounting) before F has taken effect on
every device, then holding the physical slots back from allocation until
it has.

Mechanism: a function named reclaim_floor (source:
crates/singlefs-core/src/mount.rs, line 178, parameter named
effective_floor, doc comment at lines 175-176 translated: "threshold for
the reclaimable predicate: released, and release-generation less than or
equal to the maximum of F_effective and the oldest currently-valid root in
the ring") is called from two different places. One call site, inside the
function that rebuilds the allocator on every ordinary mount (source: same
file, line 306), passes the real, freshly-computed F_effective value into
this parameter. The other call site, inside the function that deliberately
raises F (source: same file, line 466), instead passes new_floor: the
just-requested target floor value that the current process is in the
middle of trying to make durable, and that at the moment of this call has
not yet been written to any device at all. The function has no way to
tell which kind of value it was given; it treats whatever is passed as
if it were already F_effective. Immediately after this call, the reclaimed
slots have their accounting fully updated right away (the allocated bit
cleared, the allocated-slot count and the deferred-slot count both
decremented, the free-slot count incremented), but a separate flag on each
of those same slots, called held-until-floor-takes-effect, is also set at
the same time, and that flag alone is what stops the live allocator (both
the is_free check and the empty-segment search used to open a new bump
segment) from actually handing any of those slots out. The flag is
cleared, on every one of those slots at once, only after the code has
finished publishing new roots until every device in the pool carries one
with the new F (source: same file, function release_reclaim_holds, called
at the end of the raise function's publish loop).

The code's own comment at the raise-F call site (source: same file, lines
433-436, translated verbatim) states plainly that this is not a settled
question: "reclaiming happens before the first root carrying the new F is
written: a root's F and its own accounting row need to state the same
fact... but the reclaimed slots must not be handed out before it takes
effect (every device has a durable root with the new F): a floor-raise's
own empty publish is itself allocating fixed points, and if the open
segment is full it will open exactly the segment that was just reclaimed
... so hold them back first, release once every device is covered.
Whether the accounting should be computed against F at the moment that
root is written, or against F_effective after it becomes durable, is a
question left to a separate round." Treat this as an honest,
already-disclosed, currently unresolved question, not something you need
to discover.

Attack question (verbatim, source:
research/prompts/_m2-step45-code-r3-background.md, line 36, describing
this fix, translated): "the slots reclaimed by raising F are held back
until the root carrying the new F fills every device, with the accounting
moving first" -- if the function's own parameter is named effective_floor
and is documented as the value used for the decided reclaimable predicate,
which itself is written entirely in terms of F_effective (a quantity that,
by its own decided definition, only exists once every surviving device
already carries a durable root with the new F), and one call site
faithfully honors that name while the other call site knowingly passes a
value that is not yet F_effective by that same definition, is this not
exactly the class of naming bug the project's own rule about names exists
to catch -- a value passed under a name that asserts something about it
that is, at the moment of the call, false? Given that reclaiming already
mutates the allocated-slot count, the deferred-slot count, and the
free-slot count immediately, using a floor value that has not taken effect
on a single device yet, what guarantees that some other part of the
system reading those same three counters during the window before
release_reclaim_holds runs (for instance, a free-space or admission
computation) does not treat those slots as genuinely available, when the
one and only thing standing between them and being handed out is a
separate, easily-missed boolean flag that a reviewer would have to know
to look for, rather than the counters themselves?

CLOSING INSTRUCTIONS

Answer items 1 through 3 in order, using the exact three labels given at
the top (ITEM N OBJECTION, ITEM N DEFENSE, ITEM N REFUTING OBSERVATION)
for each item. Do not summarize all three items at the end. Do not add a
general conclusion beyond the three items. If you genuinely cannot
construct a convincing version of the objection for some item, say so
plainly instead of inventing a weak one, but try hard first: these are
real, carefully sourced weaknesses being raised against a real, just-landed
piece of code, not hypothetical concerns.
