Task for a local arithmetic reviewer (attacker role, arithmetic only, two independent parts).

You are given a fixed set of numbered facts about a filesystem's space-admission formula, about
two proposed fixes to that formula, and about a separate rule that governs when a deleted block
becomes reusable. You are asked two numbered items. For each item, fill in every row of the
worksheet given for that item with a number, or with an explicit statement "not determined by the
given facts, my estimate is ___, because ___". Do not answer any item with only yes or no. Show
your arithmetic. Reference facts only by their number (such as "Fact 6") or by the exact constant
or function name given in that fact; do not invent a source file name or a line number, and do not
use any line number even if you happen to know one, even if you already know one from elsewhere.
End every item with one sentence starting with "This would be refuted by:" describing a concrete
alternative computation, number, or missing term that would overturn your answer to that item.

Part 1 is about the admission formula (available and demand) at two real, directly measured points
in one history. Part 2 is about a separate rule governing when a deleted block becomes reusable.
The two parts do not share any fact; do not carry a number from one part into the other unless a
fact explicitly says so.

Do not use any markdown emphasis (no bold, no italics, no headings marked with number signs, no
backtick code spans) anywhere in your answer; plain numbered lines and plain tables using only
hyphens, letters and digits are fine. Do not write any file name or any line number anywhere in
your answer; refer only to fact numbers, row or column names, and the exact constant or function
names given in the facts below.

Part 1: the admission formula, today's code, and two candidate fixes.

Fact 1. A filesystem's admission formula defines, for every device d in a pool, available(d) as

available(d) = capacity(d) - allocated(d) - unreclaimable(d) - deferred_pending_release(d)
  - mount_time_commitment(d) - abandoned_root_exclusive(d)
  - pending_delete_occupancy / replicas - committed_reservation / replicas
  - checkpoint_reserve_pool / replicas

A write is admitted only if available(d) is at least demand(d) on every device d in the pool
separately (a per-device conjunction): a surplus on one device never offsets a shortfall on
another device. The first six terms (capacity, allocated, unreclaimable,
deferred_pending_release, mount_time_commitment, abandoned_root_exclusive) are each that device's
own value. The last three terms (pending_delete_occupancy, committed_reservation,
checkpoint_reserve_pool) are pool-wide quantities recorded as physical bytes summed over every
replica of the pool; each device's share of each of these three terms is that pool-wide sum
divided by the replica count. Every pool discussed in this task has exactly 2 devices and exactly
2 replicas (one copy of every unit on each device), so that division always has no remainder. One
physical slot is 16384 bytes (this constant is named SLOT_BYTES in the source).

Fact 2. mount_time_commitment(d), the fifth term in Fact 1, equals instance_switch_reserve(d)
only; checkpoint_reserve_pool is accounted for separately, as the last term of Fact 1's formula,
and is never added into mount_time_commitment(d) a second time.

Fact 3. The one history discussed in Part 1 of this task never deletes a file and never creates a
tombstone, so throughout that history: pending_delete_occupancy = 0 and committed_reservation = 0.
This history's pool is not zoned storage, so unreclaimable(d) = 0 on every device throughout.

Fact 4. abandoned_root_exclusive(d), the sixth term in Fact 1, equals the number of slots on
device d referenced only by roots abandoned by an administrator rollback. The one history
discussed in Part 1 of this task never performs an administrator rollback, so this term is 0 on
every device throughout that history.

Fact 5. checkpoint_reserve_pool (the last term in Fact 1) equals ckpt_cost times 16384 times the
replica count, summed over every replica of the pool; each device's share of it (that sum divided
by the replica count) is therefore exactly ckpt_cost times 16384 bytes, that is, ckpt_cost slots.
ckpt_cost is a count of 16 KiB metadata blocks, recomputed at every admission check from whatever
structure the allocator has at that moment; nothing about ckpt_cost's own internal composition
matters for this task, only its final value at each of the two checks given in Fact 13 below.

Fact 6. instance_switch_reserve(d) (Fact 2's mount_time_commitment) is computed as follows. Let
rows0 be the number of rows in the instance table right after the current mount's own row-publish.
Let N_switch = 3. Let pages_of_chain = ceiling((rows0 + N_switch) / 369), with a minimum of 1 (one
instance-table page holds 369 rows of real data). Let chain_rewrite = pages_of_chain times 2 slots
(one instance-table page spans 2 slots). Let warm_up = ckpt_cost (Fact 5) times 3, in slots. Let
one_switch = chain_rewrite + warm_up. Then instance_switch_reserve(d) = one_switch times
(N_switch + 1) = one_switch times 4, the same number on every device, recomputed fresh at every
admission check from whatever rows0 and ckpt_cost that check observes at that moment.

Fact 7. Publish-time admission is judged by a function that, in the source, is named
admission_reading_before_a_publish; a mount-time admission check calls the exact same function
with the exact same formula (Fact 1). Both call sites read allocated(d) and deferred_pending_
release(d) from the same allocator state, at the moment each check runs, before anything is
written for the operation that check is judging. demand(d) for a publish is computed, at that same
call site, from the roles this publish is about to write (its new content, its new tree nodes),
before any of this publish's own old, about-to-be-superseded physical blocks are read, checksummed,
or released; nothing about the physical blocks this publish's own write is about to make obsolete
is visible to demand(d) or to available(d) at this same call, in today's code (see Fact 9 below for
exactly when they do become visible).

Fact 8. A code comment attached to the field that holds allocated(d) states, in the author's own
words: allocated(d) is the allocator's own count of slots it currently holds, and this count is
the sum of two kinds of slots: slots still actively in use, and slots that have already been
released by some publish but are still sitting inside that publish's defer window (not yet
eligible to be handed out again). The same comment goes on to warn, in the author's own words, that
this is a known double-count: deferred_pending_release(d), the fourth term in Fact 1, is exactly
the second of these two kinds of slots (released, still inside the defer window) counted a second
time, so today's formula subtracts every such slot from available(d) twice, once inside
allocated(d) and once again as its own separate term. Whether to remove this second subtraction is
an open, undecided question in this codebase; today's code, as it stands, keeps both subtractions.

Fact 9. A slot that is released and still inside its defer window is, by Fact 8's own wording,
counted inside allocated(d). By that same wording, once a slot is no longer inside the defer
window (it has become eligible to be handed out again), it is no longer one of the two kinds of
slots Fact 8 describes, so it is no longer counted inside allocated(d) either. deferred_pending_
release(d) and the "released, still inside the defer window" portion of allocated(d) are, by Fact
8's wording, one and the same set of slots read through two different fields; a slot therefore
leaves deferred_pending_release(d) at exactly the same moment it leaves that portion of
allocated(d), not at two different moments. If you think Fact 8's wording supports a different
reading of this, state your reading explicitly and use it instead, saying why.

Fact 10. Today's code computes available(d) exactly as written in Fact 1, with deferred_pending_
release(d) read directly from the allocator's current count of released-and-still-windowed slots,
and with the timing described in Fact 7 (a publish's own about-to-be-superseded blocks are not yet
reflected in either allocated(d) or deferred_pending_release(d) at that publish's own check; they
first appear, and are then double-counted per Fact 8, starting from the very next admission check
that runs afterward, once that publish has actually been applied).

Fact 11. Candidate A1 changes only the timing described in Fact 7 and Fact 10, and only for a
publish that overwrites existing content. At the admission check for such a publish, before
comparing available(d) against demand(d), candidate A1 adds this same publish's own about-to-be-
superseded physical blocks (the specific blocks this publish is about to replace) into deferred_
pending_release(d) for the purposes of this one check, instead of waiting for the next check to see
them (as today's code does, per Fact 10). Candidate A1 makes no other change: it keeps Fact 8's
double subtraction exactly as today's code has it, and it makes no change at all to a check for any
publish that does not itself supersede existing content (such as a write of fresh content into
previously-unallocated space, which has no physical blocks of its own to anticipate).

Fact 12. Candidate A2 is candidate A1 (Fact 11) plus one further change: in the list of terms
subtracted from capacity(d) to compute available(d) (Fact 1), candidate A2 removes deferred_
pending_release(d) from that list entirely (its contribution to the subtraction is 0), because
Fact 8 already establishes that allocated(d) itself includes every released-and-still-windowed
slot, so subtracting deferred_pending_release(d) a second time was double-counting them. Candidate
A2 keeps every other term in Fact 1 exactly as today's code has it, and keeps candidate A1's timing
change from Fact 11.

Fact 13. One history exercises a small pool (2 devices, unit-area width 240 slots each), starting
right after mkfs and the first file are published, then closed and mounted writable once, then a
sequence of overwrites of that same file are published one after another, every one of them using
content that still fits inside a single data unit and every one of them read by the real, unskipped
admission formula. Two checks in this one session, on the same device, with nothing else happening
in between them, were directly measured by instrumenting today's code and printing every one of
Fact 1's per-device terms right before the formula's own subtraction: checkpoint P is the check for
the fifth such overwrite (about to be applied, not yet applied); checkpoint Q is the check for a
sixth such overwrite, attempted immediately afterward in the same mount session (the pool is not
closed or remounted between P and Q). Both checks were re-run independently after the fact; the
values below are exactly the ones confirmed by that independent re-run (an earlier draft of this
same measurement had mislabeled checkpoint Q as happening at a subsequent mount rather than a
same-session overwrite, and had reported rows0 = 2 for it; the confirmed re-run found rows0 = 1 for
Q as well, with every other value below unchanged from that earlier draft).

At checkpoint P: rows0 = 1. ckpt_cost = 5 (16 KiB blocks). capacity(d) = 3932160 bytes.
allocated(d) = 1327104 bytes. unreclaimable(d) = 0. deferred_pending_release(d) = 1064960 bytes.
mount_time_commitment(d) = 1114112 bytes. abandoned_root_exclusive(d) = 0. pending_delete_
occupancy = 0. committed_reservation = 0. checkpoint_reserve_pool = 163840 bytes (summed over both
replicas). Today's code computed available(d) = 344064 bytes at this check, and the overwrite being
checked here was in fact admitted.

At checkpoint Q: rows0 = 1. ckpt_cost = 5 (16 KiB blocks). capacity(d) = 3932160 bytes.
allocated(d) = 1556480 bytes. unreclaimable(d) = 0. deferred_pending_release(d) = 1294336 bytes.
mount_time_commitment(d) = 1114112 bytes. abandoned_root_exclusive(d) = 0. pending_delete_
occupancy = 0. committed_reservation = 0. checkpoint_reserve_pool = 163840 bytes (summed over both
replicas). Today's code computed available(d) = -114688 bytes at this check, and the overwrite
being checked here was in fact refused.

The only physical event that happens between checkpoint P and checkpoint Q is the fifth overwrite
itself (the one checkpoint P was checking) being applied, once admitted; nothing else is written or
released on this device between the two checks.

Fact 14. Every unit written by an overwrite of an existing, unchanged-length file (content still
fitting inside one data unit, as in Fact 13's history) belongs to one of these kinds, each occupying
a fixed number of slots: one user data unit, 2 slots; one extent-tree node below the root, 1 slot,
only if the extent tree has more than one level and this overwrite's write path touches that node;
the extent-tree root, 1 slot; one packed-record-style container used to hold this file's own inode
entry, 2 slots, only if this overwrite's write path touches that container; the containing tree's
own root, 1 slot. A tree with only one node (its root is also its only leaf) has no nodes "below the
root" for that tree, so the "node below the root" and "container" rows above do not apply to a tree
that is still at that shape.

Item 1. Checkpoints P and Q of Fact 13.

Fill in every row below, once for checkpoint P and once for checkpoint Q, for one device d (both
devices in this pool are identical by symmetry, so one worksheet covers both), under each of three
readings of the admission formula: today's code (Fact 10), candidate A1 (Fact 11), candidate A2
(Fact 12).

Row 1. capacity(d), in bytes (Fact 13; same at both checkpoints).
Row 2. allocated(d), in bytes, under each of the three readings (state whether any of the three
  readings changes this row from Fact 13's given value, and why or why not).
Row 3. unreclaimable(d), in bytes (Fact 3).
Row 4. deferred_pending_release(d), in bytes, under each of the three readings. For checkpoint P
  under candidate A1 and candidate A2: use Fact 11's rule, anticipating the fifth overwrite's own
  about-to-be-superseded blocks; state the size of those blocks and how you got it (Fact 13 gives
  you the one physical event between P and Q to work from). For checkpoint Q under candidate A1 and
  candidate A2: state explicitly whether Fact 11's rule requires anticipating anything for the sixth
  overwrite's own release at this check, or whether Fact 13's own reading already fully reflects
  what today's code and candidate A1 and candidate A2 would each read at Q; say why.
Row 5. mount_time_commitment(d), in bytes (Fact 13; same at both checkpoints under every reading,
  since rows0 and ckpt_cost are the same at both checkpoints per Fact 13; state this explicitly).
Row 6. abandoned_root_exclusive(d), in bytes (Fact 4).
Row 7. pending_delete_occupancy divided by replicas, in bytes (Fact 3).
Row 8. committed_reservation divided by replicas, in bytes (Fact 3).
Row 9. checkpoint_reserve_pool divided by replicas, in bytes (Fact 13).
Row 10. available(d), in bytes, under each of the three readings: row 1 minus rows 2, 3, 5, 6, 7, 8
  and 9, minus row 4 for today's code and for candidate A1 (both keep Fact 8's double subtraction),
  minus nothing extra for row 4 under candidate A2 (Fact 12 removes that separate subtraction).
Row 11. demand(d), in bytes, for the overwrite being checked at each checkpoint, using Fact 14;
  state your assumption about whether the extent tree and the inode-holding tree are still at a
  one-node shape by the time of the fifth and sixth overwrites of the same file, and say why, or
  say this cannot be determined from the given facts and give your best estimate labeled as such.
Row 12. Is row 10 at least row 11, at this checkpoint, under this reading? State yes or no, and by
  how many bytes of slack or shortfall.

Using your row 12 answers: state explicitly, for checkpoint P, whether candidate A1 flips the
outcome from admitted (today's code, Fact 13) to refused, or leaves it admitted; state explicitly,
for checkpoint Q, whether candidate A2 flips the outcome from refused (today's code, Fact 13) to
admitted, or leaves it refused. Name the specific checkpoint and the specific candidate, if any,
where your row 10 numbers make available(d) cross from non-negative to negative or from negative to
non-negative, compared to today's code's own row 10 at that same checkpoint.

This would be refuted by: [one sentence].

Part 2: how many publishes must pass before a deleted block's space can be reused, under today's
code, candidate A1, and candidate A2.

Fact 15. A release rule states: a slot is reclaimable (eligible to be handed out again) if and only
if it is released and its release generation is at most the greater of two values, called the
effective floor and the ring's oldest valid root; ordinarily (when admission has not run short) the
effective floor is not raised, so this reduces to comparing the release generation against the
ring's oldest valid root. A second rule states: admission's notion of reclaimable space equals the
minimum of (reclaimable slots plus live metadata minus the reserve pool) and whatever df reports;
when admission finds too little, the filesystem first tries to push empty publishes to raise the
effective floor, up to a bounded number of tries per admission attempt, and only reports ENOSPC if
that is still not enough.

Fact 16. A cost that this codebase's design accepts, stated in the author's own words: after
deleting an object of s bytes, a write of the same size must succeed within a bounded number of
steps; the bound is 3 publishes that change user-visible state; the deleted block is held down by 3
of the older ones among the most recent 4 states that this rule keeps; 3 more such publishes have to
happen before the deleted block re-enters the reclaimable set; until then, df does not count it as
free, and writing that much space is, by df's own accounting, a genuine ENOSPC (not a broken
promise). Empty publishes that the filesystem itself pushes (Fact 15's raise-the-floor tries) do not
count toward these 3 publishes, because an empty publish does not change user-visible state.

Fact 17. Applying Fact 9 (from Part 1) to Fact 16's rule: while a deleted block is still one of the
3 older, held-down states, it is released and still inside its window, so by Fact 9 it is counted
both inside allocated(d) and inside deferred_pending_release(d). Once the 3rd further user-visible
publish after the delete has completed and the deleted block has re-entered the reclaimable set, by
Fact 9 it is no longer counted inside either allocated(d) or deferred_pending_release(d).

Fact 18. The following is a regulation for this one exercise, not a directly observed measurement;
treat every number in it as given. One device d has capacity(d) = 50 slots throughout this exercise.
30 of those slots are occupied by other data that is never released at any point in this exercise
(call this the fixed allocation); this fixed allocation of 30 slots counts toward allocated(d) at
every single check in this exercise, unaffected by anything else described here. At publish 0, the
pool deletes one object occupying s = 20 slots; publish 0 changes user-visible state. Immediately
after publish 0, these 20 slots are released and still inside their window; by Fact 17, at this
moment allocated(d) = 30 (fixed) + 20 (the deleted object) = 50, and deferred_pending_release(d) =
20 (only the deleted object; the fixed 30 slots are ordinary in-use allocation, never released, so
they are never part of deferred_pending_release(d) at any point in this exercise). Publishes 1, 2
and 3 each change user-visible state; none of them writes or releases anything else on device d
(their only role in this exercise is to advance Fact 16's count of 3); none of them is the retried
write this item asks about. Every other term in Fact 1's admission formula (unreclaimable(d),
mount_time_commitment(d), abandoned_root_exclusive(d), pending_delete_occupancy, committed_
reservation, checkpoint_reserve_pool) is exactly 0 throughout this exercise, on this device; state
explicitly wherever you rely on this. The retried write this item asks about requires demand(d) =
20 slots, the same size as the deleted object; it writes fresh content, not a rewrite of any
existing content still present on the device, so it does not itself supersede or release any block
of its own; it can only be satisfied out of capacity that is either already free or newly reclaimed.

Item 2. The scenario of Fact 18.

For each of publish 1, publish 2, publish 3 and publish 4's own admission check (checking whether
the retried write of Fact 18 may proceed at that check, before that check's own publish, if
admitted, is applied), fill in the rows below under each of three readings of the admission formula:
today's code (Fact 10), candidate A1 (Fact 11), candidate A2 (Fact 12).

Row 1. Using Fact 16's rule and Fact 18's timeline, state explicitly whether, at this check, the
20 deleted slots are still inside their defer window or have already re-entered the reclaimable
set; say which publish number is the first one at which they have re-entered it, and why, citing
Fact 16's exact count.
Row 2. allocated(d), in slots, at this check, under this reading (Fact 17, Fact 18).
Row 3. deferred_pending_release(d), in slots, at this check, under this reading (Fact 17, Fact 18).
Row 4. available(d), in slots, at this check, under this reading: capacity(d) (Fact 18) minus row 2,
  minus row 3 for today's code and for candidate A1, minus nothing extra for row 3 under candidate
  A2 (as in Item 1's row 10).
Row 5. demand(d), in slots, for the retried write (Fact 18; same at every check, under every
  reading).
Row 6. Is row 4 at least row 5, at this check, under this reading? State yes or no, and by how many
  slots of slack or shortfall.

State explicitly, for each of the three readings, at which publish number (1, 2, 3 or 4) the
retried write is first admitted. State explicitly whether candidate A1 (Fact 11) has any effect at
all on any row of this item, given that Fact 11 only changes a check for a publish that supersedes
its own existing content, and Fact 18 states the retried write supersedes nothing of its own; if you
conclude candidate A1 changes nothing in this item, say so in one sentence and say why; if you
conclude it does change something, name the specific row and reading where it does, and why.

This would be refuted by: [one sentence].

End of task. Answer Item 1 and Item 2 in order, each with its worksheet fully filled in for every
row and every reading, and each with its own closing "This would be refuted by:" sentence.
