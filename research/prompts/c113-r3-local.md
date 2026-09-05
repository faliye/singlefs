You are one of three independent reviewers of a filesystem on-disk format proposal, third round.
The proposal is now a conclusion candidate; your job is to break it. Stance: find counterexamples
and find contradictions with the settled facts. Do not summarise. Do not agree politely. If you
cannot break a claim, say so and say what you tried.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Settled facts, not open for debate.
Authoritative on-disk state is units plus accounting plus roots; index trees and the allocation
record are derived and may be lost. Units have a header with class tag, birth checkpoint number,
filesystem id, header checksum. Data units carry (tag, tree id, object id, object birth
generation, anchor offset). Packed record containers (class 3) carry (birth tree, packed record
type, container number, container birth generation), record count, record width, payload
checksum; type 1 records are tombstones, type 2 are inode records. A container keeps its
identity when rewritten by copy on write in the same head. After a crash the in flight
checkpoint number is reissued. Every publish increments the checkpoint number. The journal
sequence number jsn is a 32 bit instance ordinal plus a 48 bit counter; recovery applies records
in strictly consecutive jsn order, stops at the first gap, applies only transactions whose commit
mark is present, verifies every named unit before applying, and applies only records whose
(instance, checkpoint) exceed the chosen root. Journal record headers will carry a transaction id
and a commit mark. Durable order: units, barrier, records, barrier, root slot. The root record
holds the instance ordinal, the checkpoint, and pointers to a tree table unit; one superblock
per disk, slot rotated, with a checked generation; a pool with one disk missing can still mount;
a disk that went missing can come back. The last K roots are rollback candidates; space freed
in checkpoint C is not reallocated before C is published and, more strictly, not within the
journal replay window. Each writable head has its own tree; clones share the origin snapshot's
units. Zoned media cannot overwrite in place. The first version runs on non zoned media only.
Encryption is not in the first runnable version.

The proposal, third version.
One. Every unit's class identity segment gains a 10 byte write sequence = (instance ordinal 4,
transaction id 6) of the transaction that wrote it; the transaction id is the journal record
header's transaction id field, counted per instance from 1, taken when the transaction enters
the commit pipeline (a serialization point); records are appended in transaction id order;
ids are never reused; user data units may be written as soon as the transaction has its id.
There is no abort after an id is taken: a single replica write failure retries at another
location with the same write sequence; an unrecoverable failure ends the instance (read only,
recovery on next mount), so the discarded transactions are always a suffix. W1: a transaction
leaves at most one net effect per key (unit, tombstone, or nothing); units written but not
named may be rewritten in place or left unnamed; when write sequences tie, the tombstone wins.
Two. Instance table: a class 3 packed record container of packed type 4, record width 64:
(instance ordinal 4, checkpoint cap 8, transaction id cap 6, reserved 46); a record with
instance ordinal 0 holds a 59 byte pointer to the next page; the root record points to the first
page (root 127 to 186 bytes); copy on write; mkfs writes an empty page. Rows: on crash recovery,
for each instance from the chosen root's up to but excluding the new one, (i, in flight
checkpoint, largest applied transaction id, or 0 if none applied); on rollback, (r_old, T_old,
infinity) and (i, 0, 0) for instances strictly between; rows are written in the same publish as
the first new root. A row may be deleted once the sweep confirms no readable unit of that
instance exceeds its caps. The table belongs to the root's indirection layer (root record, tree
table, instance table); if both replicas are unreadable the predicate degrades to birth <=
mounted root checkpoint, ties on (instance, birth) are reported as ambiguous, and rebuild stays
at level 1. Published predicate, global with i_now the mounted root's instance: for i < i_now,
no row means published, a row (i, T, W) means birth <= T and id <= W; for i == i_now, birth <=
the mounted root's checkpoint; i > i_now is corruption. Instance ordinal: lives in the
superblock; new ordinal = max(all visible superblocks, all root ring roots' ordinals, the
largest ordinal in the instance table) + 1; taken on every mount including clean mounts,
persisted before any unit is written; read only mounts take none; starts at 1.
Three. Rollback is defined as a recovery: the admin picks an older root from the ring; no
journal record after it is applied; a new instance ordinal is taken; rollback rows are written;
the first new root is published. Roots for the rule are the writable heads plus the snapshot
list (snapshot roots are assumed to carry only a checkpoint). Visible to root R iff published,
birth <= R's checkpoint, and writer tree in R's ancestry with birth <= that clone point. Current
version = the visible unit with the largest write sequence unless a visible tombstone names the
key with death write sequence >= it. Evaluation: first the current version of every container
identity; then tombstones and retirement records from current type 1 containers, and instance
rows from the type 4 chain (first page pinned by the root pointer); then data keys and type 2
containers. A torn latest version of a tombstone container belongs to a transaction that was
never applied, so falling back to the previous version is correct. Rebuilt allocation record =
every root's current locations, plus the index nodes newly written by the rebuild, plus fixed
structures; old class 2 nodes found by the scan are garbage; rebuilt state is read only until
promoted.
Four. Sweep: a background pass separate from incremental scrub. Admission is an explicit
conjunction: the allocation record says the location is free (including never allocated);
the free's checkpoint <= current minus K; the free's checkpoint <= the replay window's lower
bound; the header is still readable. Forbidden while the image is rebuilt and not promoted;
promotion needs a level 2 witness which does not exist today, so a rebuilt image is never swept
again for now. Non zoned: overwrite the header sector; zoned: unsupported in the first version,
tombstones and instance rows are simply not reclaimed there. The sweep watermark advances only
when a full pass completes; the watermark and the checkpoint of the most recent root
destruction are two accounting statistics merged by maximum.
Five. Tombstone records gain a 10 byte death write sequence. Retirement records (packed type 3)
are (packed type 2, container number 8, container birth 8) with no write sequence; merges,
emptied leaves and rebirths write one. Reclaim of tombstone or retirement records requires the
existing snapshot condition and sweep watermark >= max(death or retirement checkpoint, the
checkpoint of the most recent root destruction).
Six. A clone head's first rewrite of a shared container takes a new identity (writing head,
smallest inode number, this publish) and writes a retirement record for the old identity in
its own tombstone container; tree table entries carry (origin tree, clone point = origin
snapshot's checkpoint); destroying a tree with descendants leaves a stub.

Attack in this order.
1. The transaction id as write sequence: two transactions enter the pipeline in order A then B;
B's units are written first; the crash leaves B's records durable and A's not. What does the
prefix rule do, what are the rows, which units are judged published, and is the truth
consistent? Then: A's records durable, B's not, but B's units on disk. Then: a transaction
spanning three records where only the first two are durable.
2. The retry rule: a unit written to location L1 fails on one replica, retried at L2 with the
same write sequence; both L1 and L2 hold readable copies with identical write sequence and key.
What does the rule compute? Is the pairwise distinct write sequence invariant violated?
3. The max rule for the instance ordinal: construct a history with two disks, degraded
operation, a disk coming back, and a rollback, such that the new ordinal collides with an
ordinal whose units are still readable somewhere.
4. Rollback rows: after rollback to R_old with instance r_old, the row (r_old, T_old, infinity)
also covers r_old's units with birth <= T_old that were written in the rolled back timeline's
in flight checkpoint T_old + 1? Check: can a unit with instance r_old have birth <= T_old and yet
belong to the abandoned timeline? Then: rollback twice in a row; rollback to a root older than
the last crash; rollback after a clone was created in the abandoned timeline.
5. The instance table as a class 3 container: the first page is pinned by the root pointer; the
second page's pointer is a record inside the first page. When the table is rewritten by copy on
write on recovery, which pages are rewritten? Can an old page version be mistaken for current by
the rule in step one of the evaluation? Does a snapshot root have its own view of the instance
table?
6. The sweep admission: give a history where a location passes all four conditions and yet its
unit is still needed by some root or by a rollback candidate; give one where a unit is never
admitted so a tombstone can never be reclaimed even on non zoned media.
7. Anything that contradicts a settled fact above.
For every counterexample give the exact sequence of events, which root is affected, what the
rule computes, and what the truth is.
