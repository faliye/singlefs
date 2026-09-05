You are one of three independent reviewers of a filesystem on-disk format proposal, second round.
Your assigned stance is: find counterexamples. Construct concrete histories under which the rule
picks the wrong version for some root, or under which a required input cannot be produced.
Do not summarise. Do not agree politely. If you cannot find a counterexample for a claim, say
so and say what you tried.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Background facts, all settled and not open for debate.
Authoritative on-disk state is units plus accounting plus roots. Index trees and the allocation
record are derived and may be lost. Every unit header has a class tag, a birth checkpoint
number, a filesystem id and a header checksum. Data units carry (tag, tree id, object id,
object birth generation, anchor offset). Packed record containers carry (birth tree, packed
record type, container number, container birth generation), a record count, a record width
and a payload checksum; type 1 records are tombstones, type 2 records are inode records and a
container of them is a leaf of the inode tree. The birth checkpoint number is the number of
the publish that made the unit durable; after a crash the in flight checkpoint number is
reissued. Every publish increments the checkpoint number. The journal sequence number jsn is
a 32 bit instance ordinal plus a 48 bit counter; the instance ordinal increases on every
recovery; recovery applies records in strictly consecutive jsn order and stops at the first
gap, applies only transactions with complete commit marks, and verifies every named unit
before applying. Durable order of a publish: copy on write units, barrier, journal records,
barrier, root slot. The root record is 127 bytes in a 512 byte slot and holds the instance
ordinal, the checkpoint number and a 59 byte pointer to a tree table unit; the superblock is
one per disk, updated by slot rotation, with a generation number that is checked. Each
writable head has its own tree; a clone head shares the origin snapshot's units. The last K
roots in the root ring are kept as rollback candidates and space freed in checkpoint C cannot
be reallocated before C is published (the defer window). Deleting writes a tombstone record.
Accounting rows keep only the last K generations. Zoned media cannot overwrite in place, only
append. There is an incremental scrub that verifies only blocks born after a watermark.
bcachefs keeps a per node rewrite sequence and a per bset journal sequence, ignores bsets newer
than the newest written journal entry, records those sequences in a superblock blacklist, and
garbage collects the blacklist once no on disk structure references the sequence and the
journal no longer reaches back that far.

The proposal, second version.
One. Every unit's class identity segment gains a 10 byte write sequence equal to the
transaction number of the transaction that wrote it; the transaction number is the jsn of the
transaction's first journal record. Rules: the number is taken after the commit fixed point
converges and before any unit is written; a transaction that took a number always writes a
record, even an empty one if it aborts; records are appended in number order; numbers are
never reused. Rule W1: a transaction leaves at most one net effect per logical key, a unit or
a tombstone or nothing; a unit already written but not yet named may be rewritten in place or
simply not named; when a tombstone and a unit carry equal write sequences the tombstone wins.
For data units the field is plaintext next to the five tuple; for metadata classes it is in
the ciphertext when encryption is on.
Two. Published-ness lives in an instance table unit: a 16 KiB metadata unit pointed to
directly by the root record (a 59 byte pointer added to the root, 127 to 186 bytes), rows of
(instance ordinal 4, counter 6), chained when full. On recovery, for every instance ordinal
from the chosen root's instance up to but excluding the new instance, one row is written with
the largest counter that replay applied for that instance (0 if none), in the same publish as
the first root of the new instance. A row may be deleted once the sweep confirms that no
readable unit of that instance has a counter above the watermark. The instance ordinal itself
lives in the superblock; recovery increments it and persists it before touching any unit;
ordinals start at 1 and 0 is invalid. The published predicate is global: with i_now the
instance of the mounted root, a unit with write sequence (i, c) is published iff i < i_now
and (no row for i, or c <= watermark(i)); or i == i_now and birth checkpoint <= the mounted
root's checkpoint; i > i_now is corruption. Snapshot roots carry no instance ordinal.
Three. Roots are the writable heads plus the snapshot list; older roots in the ring do not
take part; after a rollback the older root becomes the mounted root and everything is judged
from it. Logical key of a data unit is (object id, object birth generation, anchor offset)
with writer tree the tree id in the five tuple; logical key of a container is its four tuple
with writer tree its birth tree. Visible to root R iff published, birth checkpoint <= R's
checkpoint, and the writer tree is in R's ancestry with birth checkpoint <= the clone point
for that ancestor. Current version of key k for R is the visible unit with the largest write
sequence unless a tombstone visible to R names k with a death write sequence >= that unit's
write sequence. Evaluation is in three steps: first the current version of every container
identity, then the tombstone and retirement records taken from the current type 1 and type 3
containers, then data keys and type 2 containers. A stale version of a tombstone container
mistaken for current only adds tombstones, which only kill smaller write sequences. The
rebuilt allocation record is the union of every location reached by walking every root (index
nodes, tree table, instance table, root ring, journal, superblock) plus every root's current
locations.
Four. Garbage sweeping is a background pass separate from incremental scrub: whole disk, low
frequency, resumable, on demand. It judges garbage by the run time allocation record only: a
location already in the allocatable set (past the defer window) whose header is still readable.
On non zoned media it overwrites the header sector; on zoned media the compaction job moves
live data out and resets the zone. It records a sweep watermark: the checkpoint up to which
all frees are confirmed unreadable. It also clears unpublished orphans of old instances.
Five. Tombstone records gain a 10 byte death write sequence. A new packed record type 3,
container retirement, records (type 2, container number, container birth, retirement write
sequence); merges, emptied leaves and rebirths write one. A tombstone or retirement record
may be reclaimed only when no snapshot still references what it kills (no root references any
version of a retired container) and the sweep watermark >= max(the death or retirement
checkpoint, the checkpoint of the most recent root destruction). Without sweeping, tombstones
only grow and hold reserved space.
Six. When a clone head first rewrites a shared container it takes a new identity (birth tree
= the writing head, container number = the smallest inode number in the container, birth =
this publish) and writes a retirement record for the old identity in its own tombstone
container. The tree table entry carries (origin tree, clone point = the origin snapshot's
checkpoint). Destroying a tree that has descendants leaves a stub entry keeping those two
fields until no descendant remains.
Seven. Invariants: write sequences of one key are pairwise distinct among readable published
units; no readable garbage below the sweep watermark; instance table rows are only deleted
under the reclaim condition; the superblock instance ordinal is >= every root's.

An experiment (E104) already models items one to six as a counting model: the full rule is
exact on 24 seeds of eight world classes including crashes with partial replay, double
crashes, snapshots, clones, container merges and tombstone reclamation; each ablation fails
on its targeted world by a closed form count; an instance-only header (no counter) fails
exactly when the in flight checkpoint was partially replayed.

Attack in this order.
1. Break the transaction number rules: a transaction that took its number, wrote some units,
then hit an I/O error and wrote an empty record; two transactions committing concurrently;
a transaction whose records span two journal records; a recovery that applies a prefix of a
transaction's records. Show a history where a unit's write sequence is judged published
though its transaction was not applied, or the reverse.
2. Break the instance table: recovery persists the incremented ordinal in the superblock then
crashes before writing any row and before any root; the superblock slot rotation loses the
newest slot; a rollback to an older root whose instance predates rows already in the table;
the table unit itself torn or lost; a row deleted by the sweep while a stale unit of that
instance still exists on a zoned zone that has not been reset.
3. Break the ancestry: a clone of a clone whose middle tree is destroyed and its stub later
removed; two heads cloned from the same snapshot both rebirthing the same container; a
snapshot taken of a clone after rebirth then the clone destroyed; a container rebirth whose
smallest inode number equals a container number the origin created after the clone point.
4. Break the sweep and reclaim: sweep watermark advanced on one device of a multi device pool
but not another; a root destroyed during a sweep; a tombstone reclaimed, then a rollback to an
older root that still needed it; an unpublished orphan of an old instance cleared by the sweep
while a rollback would have made it published (is that even possible?).
5. Break the tie rule: a transaction that deletes key k and recreates it, then aborts after
writing the unit; a transaction that writes k, deletes k, then writes k again.
6. Anything in the proposal that contradicts a background fact above.
For every counterexample give the exact sequence of events, which root is affected, what the
rule computes, and what the truth is.
