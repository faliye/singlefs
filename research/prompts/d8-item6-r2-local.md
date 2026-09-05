You are one of three independent reviewers of a filesystem on-disk format proposal, second
round. Your assigned stance is: find counterexamples. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Background facts, all already settled and not open for debate.
The authoritative on-disk state is units plus accounting plus roots; index trees are derived
and are rebuilt by scanning units. An unrecomputable authoritative value must not be placed
inside a rebuildable container. There is a settled unit class called the packed record unit:
a 32768 byte unit whose header carries a class tag, the birth tree id, a packed record type,
a container number, a container birth generation, a record count, a record width, a birth
checkpoint, the filesystem id and a CRC32C over the record area; one container holds one
record type, one generation, one tree, fixed width records; a container index is the sole
holder of physical pointers to containers; a new packed record type is incompat by default.
Index nodes are 16384 byte units of class 2 with tree id, level, key range where settled,
birth checkpoint and filesystem id in the header; their internal layout may carry a message
buffer, and the settled rule says all layouts share the same buffer fraction. Each writable
head has its own trees. Deleting an inode moves it to a deleted-inodes tree reclaimed in the
background; the extent key is (locality id, inode, offset) with no birth generation.
Accounting is authoritative and keyed by (statistic type, tree id, device, generation) with
an idempotent full value. Every fsync is a publish, increments the checkpoint number, and
writes the dirty leaf plus all ancestors plus the root slot. Format extension rules: a new
field in a reserved area that old readers ignore is compat; changing the meaning of an
existing field is incompat. Version one has no directories, no snapshots, no encryption.

The proposal under review, second version.
One. The inode tree's leaves are packed record units of record type 2, each holding up to
233 fixed width 140 byte inode records; the internal nodes are ordinary class 2 index nodes
whose entries are separator key 8, container number 8, container birth generation 8, child
pointer 59. The internal nodes are the container index; no separate tree exists. Leaves are
authoritative units, internal nodes are derived and rebuilt from leaves (each leaf's key
range is computed from the inode numbers of its records). The inode tree's internal nodes
carry no message buffer; updates go straight to the leaf container by copy on write of the
leaf plus all ancestors. The record format is a permanent contract; the internal node
layout is not.
Two. The inode tree key is the 8 byte inode number; uniqueness is per (tree, inode). Inode
numbers are monotone and never reused. The next inode number is stored as an accounting
statistic keyed by (statistic type, tree id, device 0, generation), published with every
transaction.
Three. The record: inode 8, object birth generation 8 (checkpoint number of the creating
publish), locality id 8 (zero in version one), mode uid gid nlink 4 each, size 8, blocks 8,
rdev 8, three timestamps as 8 byte seconds and 4 byte nanoseconds, 4 bytes padding, change
counter 8 (checkpoint number of the last change; commit anchor is the leaf container's birth
checkpoint), flags 8 (must be zero; nonzero means corrupt; assigning meaning is incompat),
reserved 24 (unknown content is ignored; compat extension area). Little endian. No extent
pointers, no birth time.
Four. Invariants: record inode equals key, else red and EIO, never silently repaired; record
locality id equals the leading component of every extent key of that object, with the stated
scope that after a rebuild of the extent tree both sides come from the record and the check
has no power; nlink equals dirent count, gated by the compat feature bit that marks the
dirent keyspace as online; version one's single file is inode number 1 with nlink 1.
Five. First transaction: one leaf container (container number 1, one record), zero internal
nodes so the tree table entry points directly at the packed record unit, and one accounting
statistic for the next inode number.

Your task. Give at most seven counterexamples. For each one state the concrete situation,
which specific settled fact or number above it contradicts, and what an implementer would
observe going wrong. Concentrate on cases the designers are least likely to have walked: a
leaf container split (233 records, the 234th insert) and what the two new containers' numbers
and birth generations are; a crash after the new leaf containers are written but before the
root; rebuilding internal nodes from leaves when two leaves overlap in key range because one
is a stale version; the next-inode-number statistic when two heads clone from the same
origin; a tree whose root is a packed record unit rather than an index node (what a reader
that expects a class 2 node does); the reserved area that old readers ignore holding a value
a new reader needs for correctness; and whether a per-tree exception to the shared message
buffer fraction is representable in the format at all. If a part is simply correct, say so in
one sentence and move on rather than inventing an objection.
