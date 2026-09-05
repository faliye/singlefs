You are one of three independent reviewers of a filesystem on-disk format proposal, third
round. Your assigned stance is: find counterexamples in the parts that changed since the
second round. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Background facts, all already settled and not open for debate.
The authoritative on-disk state is units plus accounting plus roots; index trees are derived.
A packed record unit is a 32768 byte unit whose header carries a class tag, the birth tree
id, a packed record type, a container number, a container birth generation, a record count,
a record width, a birth checkpoint, the filesystem id and a CRC32C over the record area.
Accounting entries are idempotent full values keyed by (statistic type, tree id, device,
generation); today their merge rule is last writer wins, and the "last" is ordered by a
sequence number that is not yet settled; accounting goes through a write buffer that sorts
by key and drops arrival order. Each writable head has its own trees; a clone head starts
by sharing the origin's tree nodes and units copy on write. Deleting an inode moves it to a
deleted-inodes tree reclaimed in the background. Every fsync is a publish and increments the
64 bit checkpoint number. Index nodes are 16384 byte units; their internal layout reserves
65 percent of the space for a message buffer in every layout. Format extension: a new field
in a reserved area is compat only when nothing depends on its meaning; changing the meaning
of an existing field is incompat. Version one has no directories, no snapshots, no
encryption.

What changed in the third version.
One. The inode tree's leaves are packed record units of type 2 (233 records of 140 bytes);
internal nodes are ordinary index nodes whose entries are separator key 8, birth tree 8,
container number 8, container birth generation 8, child pointer 59 (fanout 179); the root is
always an index node even when the tree has one leaf. Internal nodes never hold buffered
messages for this tree; the buffer region exists but stays empty.
Two. A leaf container's number is the smallest inode number it held when it was created, and
its birth generation is the checkpoint of the publish that created it. On a split the left
half keeps its identity and the right half becomes a new container numbered by the split
key; on a merge the left identity survives and the right is retired. No allocator exists for
container numbers.
Three. Inode numbers are monotone and never reused, 64 bits wide, with a declared lifetime
bound of 2 to the 63. The next inode number is an accounting statistic keyed by (statistic
type, tree id, generation) with no device dimension, and its merge rule is take the maximum,
so it does not depend on the unsettled sequence number. A clone head's statistic is
initialised from the origin's value at clone time. An invariant requires the watermark to
exceed the largest inode number in the inode tree, in the deleted-inodes tree and among the
tree's tombstone records.
Four. The 140 byte record: inode 8, object birth generation 8, locality id 8, mode uid gid
nlink 4 each, size blocks rdev 8 each, three timestamp seconds 8 each, change counter 8,
three timestamp nanoseconds 4 each, padding 4, flags 8, reserved 20. Flags, padding and
reserved must all be zero; a nonzero byte makes that one record return EIO but does not
reject the container; giving any of those bytes a meaning later is incompat. There is no
compat extension path inside the record; the overflow route is a separate attribute keyspace
plus flag bits. The record width must equal the width the registry gives for type 2.
Five. Invariants: record inode equals key; record locality id equals the leading component
of every extent key with a three valued outcome (red, green, undecidable after a rebuild);
nlink equals dirent count gated by the existence of a dirent tree, not by a feature bit;
the watermark invariant above; the inode tree's internal buffers are empty.
Six. Rebuilding internal nodes from leaves groups containers by (birth tree, container
number, container birth generation); choosing the current version among several physical
versions of one identity is an already registered open debt; the rebuilt filesystem mounts
read only.

Your task. Give at most seven counterexamples against the changed parts only. For each one
state the concrete situation, which settled fact or number above it contradicts, and what an
implementer would observe going wrong. Concentrate on: a container number equal to the
smallest key when the smallest record is later deleted and the container is then split;
two containers with the same number and generation created by a split and a merge inside
one publish; a take-the-maximum merge when a rolled back transaction had written a larger
watermark; a clone taken while the origin's watermark is only in the write buffer; the
watermark invariant when the deleted-inodes tree is itself derived and being rebuilt; a
record with a nonzero reserved byte written by a newer version that an older version then
rewrites; and whether "the buffer stays empty" can be checked on a node that is derived and
rebuilt by the same software that would have filled it. If a part is simply correct, say so
in one sentence and move on rather than inventing an objection.
