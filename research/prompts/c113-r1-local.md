You are one of three independent reviewers of a filesystem on-disk format proposal.
Your assigned stance is: find counterexamples. Construct concrete histories (sequences of
writes, deletes, snapshots, clones, crashes, recoveries, reclaims) under which the proposed
rule picks the wrong version for some root, or under which one of the proposed inputs cannot
be produced. Do not summarise. Do not agree politely. If you cannot find a counterexample for
a claim, say so and say what you tried.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Background facts, all already settled and not open for debate.
The authoritative on-disk state is units plus accounting plus roots. Index trees and the
allocation record tree are derived state and may be lost; a scan rebuild must recover them
from units, accounting and roots only. At run time the predicate "does this physical location
belong to a current version" is answered by the allocation record plus the snapshot deadlists;
that predicate has no input during a scan rebuild because those two structures are exactly
what is being rebuilt. An earlier experiment showed that scanning units alone cannot tell
which of several readable copies of the same logical key is current (the ambiguous count
equals the number of overwrites) and cannot see deletions (resurrections equal the number of
deletes); the arm that fixed it used an oracle boolean set by the model, not a mechanism.
Every unit has a header with a header checksum, a class tag, a birth checkpoint number, and a
filesystem id. Data units (class 1) carry a five tuple (tag, tree id, object id, object birth
generation, anchor offset). Packed record containers (class 3) carry (birth tree, packed
record type, container number, container birth generation), a record count, a record width
and a payload checksum; type 1 records are tombstones (object id, object birth generation,
range); type 2 records are inode records and a container of them is a leaf of the inode
tree; a container keeps its identity when rewritten by copy on write within the same head.
The birth checkpoint number of a unit is the number of the publish that made it durable; after
a crash the checkpoint number that was in flight is reissued, so an abandoned unit and the
retried unit can carry the same birth number and the same identity. Every publish increments
the checkpoint number by one. The journal sequence number jsn is a 32 bit instance ordinal plus
a 48 bit counter; the instance ordinal increases on every recovery; recovery applies only
journal records whose (instance, checkpoint) is greater than the chosen root, in strictly
consecutive jsn order, stopping at the first gap, only transactions with complete commit
marks, and it verifies every unit named by a record before applying it. The durable order of
a publish is: copy on write units, barrier, journal records, barrier, root slot. Each writable
head has its own tree; a clone head shares the origin snapshot's units, whose headers carry
the origin tree as tree id. Snapshot S references block b iff birth(b) <= S.txg < death(b).
Deleting writes a tombstone record; tombstones may be reclaimed once every snapshot that
still referenced the death is gone. Freed space produced in checkpoint C must not be
reallocated before C is published. Freeing space must itself never need to allocate space.
Accounting rows are idempotent full values keyed by (statistic, dimensions, generation) and
only the last K generations are kept, so a row survives only if it is rewritten at every
publish. The rebuild path is a degraded path with three levels; the level that allows
writing again has no witness today. For reference only, not as evidence: bcachefs keeps a
journal sequence number in every bset, its node scan ranks multiple found versions of a node
by (node seq, journal seq), and its journal seq blacklist makes recovery ignore bsets newer
than the newest journal entry that was written, recording the blacklisted seq in the
superblock until the node is rewritten. btrfs reuses the generation number of an uncommitted
transaction after a crash.

The proposal under review.
One. Every unit header, all classes, gains a 10 byte write sequence, equal to the jsn of the
journal record that names the unit. The jsn is reserved when the transaction forms, before
the units are written; the durable order is unchanged. Write path rule W1: a transaction
writes at most one unit per logical key; a unit already written but not yet named by a
durable record may be rewritten in place. Hence all written units of one logical key are
totally ordered by write sequence.
Two. Published-ness lives in the accounting tree as a statistic "instance watermark":
one row per instance that ended in a crash, written by the recovering instance before its
first publish, value equal to the last jsn counter that recovery applied for that instance;
clean unmounts write nothing. Rows of this statistic are exempt from the K generation
deletion (the statistic registry gains a retention column). A unit with write sequence
(i, c) is published iff: i is older than the chosen root's instance and either no row exists
for i or c <= watermark(i); or i equals the root's instance and the unit's birth checkpoint
is <= the root's checkpoint. An instance newer than the root's is corruption.
Three. The rule. Roots are the writable heads in the tree table plus the persistent snapshot
list. The ancestry of a tree is the tree itself plus its origin chain, each ancestor visible
only up to the earliest clone point on the path; the tree table entry carries (origin tree,
clone checkpoint). The logical key of a data unit is (object id, object birth generation,
anchor offset) and its writer tree is the tree id in the five tuple; the logical key of a
container is its four tuple and its writer tree is its birth tree. A unit is visible to root R
iff it is published, its birth checkpoint <= R's checkpoint, and its writer tree is in R's
ancestry with birth checkpoint <= the corresponding clone point. The current version of key k
for R is the visible unit with the largest write sequence, unless a tombstone visible to R
names k with a death write sequence larger than that unit's write sequence, in which case k is
dead. For inode containers, when two visible containers hold a record for the same inode
number, the record from the container with the larger write sequence wins. From this rule the
allocation record (locations current for any root), each head's deadlist (current for a
snapshot in its lineage but not for the head), and the garbage set (readable, published,
current for no root) are all computed.
Four. Freed units are not touched on the free path. Scrub, which reads the whole disk anyway,
invalidates the header of every readable published unit that is current for no root and
records a scrub watermark: the checkpoint up to which all frees have been invalidated.
A tombstone may be reclaimed only if the existing snapshot condition holds and the scrub
watermark is >= the tombstone's death checkpoint. If scrub never runs, tombstones only grow,
bounded by the number of deletions.
Five. Tombstone records gain a 10 byte death write sequence. A new packed record type 3,
container retirement, records (packed record type, container number, container birth
generation, retirement write sequence); the inode tree merge that retires a right sibling
writes one. Without it the retired container's last version stays readable, has no newer
version of its own identity, and the rule would treat it as current.
Six. When a clone head first rewrites a container it shares with its origin, the new version
takes a new identity whose birth tree is the writing head. The tree table entry carries
(origin tree, clone checkpoint), which is the clone ancestry table.
Seven. Invariants: the existing "birth checkpoint <= root checkpoint" check becomes one
branch of the published-ness predicate; new checks that write sequences of one key are all
distinct, that readable garbage is always shadowed by a newer version or a visible tombstone,
and that instance watermark rows are never deleted. The checker implements the rule
independently and compares the rebuilt allocation record and deadlists with the run time
trees.
Eight. An experiment will model all of this with an oracle arm, a full rule arm and six
ablation arms, each ablation required to fail on a targeted world with a closed form count.

Attack in this order.
1. Build a history where the full rule of item three returns a wrong current version for some
root. Consider: a key deleted and recreated inside one transaction; a key overwritten twice
in one checkpoint; a crash where some transactions of the in flight checkpoint were replayed
and others abandoned, followed by a rewrite of only some keys; a clone head and its origin
both rewriting the same shared unit; a snapshot taken between the two; a container split
followed by a merge followed by a split; a tombstone reclaimed before scrub ran; a freed
location reused by an unrelated unit while an older version of the key remains readable;
a snapshot destroyed and its deadlist merged.
2. Show whether the instance watermark of item two can be wrong or missing: the recovering
instance crashes again before writing the row; the row is written but the accounting tree
row is lost with the index; two recoveries in a row; a root chosen that is older than the
crash.
3. Show whether reserving the jsn before writing units breaks any of the settled journal rules
(strictly consecutive jsn, stopping at the first gap, complete commit marks) when a
transaction is abandoned after reserving its number.
4. Show whether the scrub gate of item four leaves a window where a resurrection is still
possible, and whether the scrub watermark can itself be wrong after a crash during scrub.
5. Show whether container rebirth of item six breaks anything for the origin head, for a
second clone of the same origin, or for a snapshot of the clone taken before the rebirth.
6. Anything in the proposal that contradicts a background fact stated above.
For every counterexample give the exact sequence of events, which root is affected, what the
rule computes, and what the truth is.
