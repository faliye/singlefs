You are one of three independent reviewers of a filesystem on-disk format proposal, fourth and
final round. The proposal is a conclusion candidate. Stance: break it. Construct concrete
histories under which a rule computes the wrong answer, or show a required input that cannot
be produced. Do not summarise. Do not agree politely. If you cannot break a claim, say so and
say what you tried.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Settled facts, not open for debate.
Authoritative on-disk state is units plus accounting plus roots; index trees and the allocation
record are derived. Every write is replicated w >= 2 times, byte identical copies at different
locations. Unit headers carry class tag, birth checkpoint, filesystem id, header checksum;
data units carry (tag, tree id, object id, object birth generation, anchor offset); packed
record containers carry (birth tree, packed type, container number, container birth), record
count, record width, payload checksum. The birth checkpoint is the number of the publish that
makes the unit durable; after a crash the in flight checkpoint number is reissued. A checkpoint
is triggered by a timer or a dirty byte threshold. Journal records carry an 8 byte transaction
id and a 1 byte commit mark; jsn is a 32 bit instance ordinal plus 48 bit counter per record;
recovery applies records in strictly consecutive jsn order, only complete transactions, only
records whose (instance, checkpoint) exceed the chosen root, and verifies named units first;
recovery normally chooses the root with the largest checkpoint. Durable order: units, barrier,
records, barrier, root slot. One superblock per disk, slot rotated; a pool with one disk
missing can still mount; the root ring keeps the last K roots on different disks as rollback
candidates. The allocation record maps each 16 KiB slot to an allocation generation. Freed
space is not reallocated within the last K checkpoints. The first version commits serially,
runs on non zoned media, without encryption.

The proposal, fourth version.
One. Each unit's class identity segment gains a 10 byte write sequence = (instance ordinal 4,
low 48 bits of the transaction id). The transaction id counts per instance from 1. A
transaction takes its id when it is assigned to a checkpoint, and that checkpoint number is
fixed for it at that moment; a checkpoint cut waits for all transactions already assigned to
it (quiesce); after taking the id, user data units may be written at any time; tree nodes are
written after the commit fixed point. Records are appended in transaction id order; a
transaction's fsync cannot return before all smaller ids have appended or the instance has
ended. Failure table: before taking an id, nothing happens; after taking an id but before any
unit is written, the id is simply burned (no unit, no record); after a unit is written, a
single replica failure retries at another location with the same write sequence, and any
unrecoverable failure ends the instance (read only, recovery on next mount writes rows).
W1: at most one net effect per key per transaction; equal write sequences that are not
replicas: tombstone wins.
Two. Instance table: a class 3 packed container of packed type 4, record width 64; row
records (kind 0, instance ordinal 4, checkpoint cap 8, transaction id cap 8, reserved 43);
the last record of a page is a chain pointer record (kind 1, pointer 59, reserved 4). Its four
tuple is (birth tree 0, type 4, page index, birth 0); the root record holds its pointer
directly; the whole chain is rewritten by copy on write whenever a row is written; mkfs writes
one empty page. Rows are unique per instance ordinal, later rows overwrite earlier ones. Rows
are written at the next recovery for every instance that did not end cleanly: (i, in flight
checkpoint, largest applied transaction id, or 0); on rollback (r_old, T_old, infinity) and
(i, 0, 0) for instances strictly between. A row may be deleted once the sweep confirms no
readable unit of that instance exceeds its caps. If the table is unreadable on a normal mount,
the pool stays writable but records a persistent flag; a scan rebuild without the table stays
at level 1 (read only). Published predicate, global with i_now the mounted root's instance:
i < i_now: no row means published, a row (i, T, W) means birth <= T and id <= W; i == i_now:
birth <= the mounted root's checkpoint; i > i_now: corruption. The instance ordinal lives in
each superblock; new ordinal = max(all visible superblocks, all root ring roots) + 1, taken on
every writable mount, written to all visible superblocks before any unit; a returning disk
whose superblock ordinal or timeline does not match the current ring is discarded and resynced
before any of its units may take part in version selection; whole image rollback (copying an
old image back) is declared unprotected.
Three. Replica merge first: readable units with identical class identity segments including
the write sequence are replicas of one logical version; they must be byte identical; the
current version carries all its replica locations. Rollback is an explicit exception to normal
recovery: the admin picks an older ring root R_old out of band; no record after it is applied;
a new instance ordinal is taken; rollback rows are written; the first new root's checkpoint =
max checkpoint of all ring roots + 1; runtime state (defer queue, allocator cursors, current
accounting values) is reloaded from R_old's accounting tree. Visible to root R iff published,
birth <= R's checkpoint, writer tree in R's ancestry with birth <= the clone point. Current =
largest write sequence among merged visible candidates unless a visible tombstone names the
key with death write sequence >= it. Rebuilt allocation record = every root's current
versions' replica locations plus index nodes newly written by the rebuild plus fixed
structures; old class 2 nodes found by the scan are garbage.
Four. The allocation record entry is not deleted on free; its value becomes (freed flag, free
checkpoint), point deleted K checkpoints later. Sweep admission: entry flagged freed, and free
checkpoint <= the oldest ring root's checkpoint, and header readable. Orphans (no entry and not
in the allocator's in flight overlay) are cleared iff their instance is older than the current
one and the published predicate says unpublished; units of the current instance are never
touched. The sweep is forbidden while a rebuilt image is not promoted; promotion needs a level
2 witness that does not exist, so a rebuilt image is permanently read only today. The sweep
watermark and the checkpoint of the latest root destruction are accounting statistics
rewritten at every publish, merged by maximum.
Five. Tombstone records carry a 10 byte death write sequence; retirement records carry no
write sequence; reclaim needs the snapshot condition and sweep watermark >= max(death or
retirement checkpoint, latest root destruction checkpoint).
Six. Clone rebirth with retirement of the old identity; tree table entries carry (origin tree,
clone point = origin snapshot checkpoint); stubs survive while descendants exist.

Attack in this order.
1. Quiesce: transaction A is assigned to checkpoint T and writes units; the timer fires; B is
assigned to T+1; A is still running; B's units are written with birth T+1 before A finishes.
Then a crash. Show what recovery and the rule compute for A's and B's units. Then: A takes an
id and burns it before writing (failure table row two); does anything break the prefix rule
or the rows.
2. Replica merge: two replicas of the same version where one replica is torn in the payload
and the class has no payload checksum (class 1 data unit). What does the rule do. Then: a
retry after a single replica failure leaves three readable copies; then a later free and
reuse of only one of the three locations.
3. Rollback: rollback to R_old, then before any new publish, crash; recovery chooses the root
with the largest checkpoint. Which root wins and is the rollback preserved. Then rollback
twice with a crash in between. Then rollback to a root whose instance already has a crash row
with a larger checkpoint cap.
4. Sweep with the freed flag: a slot freed at checkpoint C, point deleted from the allocation
record at C + K, then reallocated; then a rollback to a root older than C. Is the old unit
still needed by R_old, and was it cleared by the sweep before the rollback. Then: a slot freed,
then the allocation record rebuilt by a scan (derived state lost); does the rebuilt entry
carry the freed flag.
5. The published predicate with replicas and rollback together: a unit of instance 3 written
in the abandoned timeline, one replica on disk A and one on disk B; disk B was absent during
the rollback and comes back later with its superblock at ordinal 3. Walk the discard rule.
6. Anything contradicting a settled fact above.
For every counterexample give the exact sequence of events, which root is affected, what the
rule computes, and what the truth is.
