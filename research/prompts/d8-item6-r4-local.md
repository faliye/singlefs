You are one of three independent reviewers of a filesystem on-disk format proposal, fourth
round. Your assigned stance is: find counterexamples in the parts that changed since the
third round. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Background facts, all already settled and not open for debate.
The authoritative on-disk state is units plus accounting plus roots; index trees are derived.
A packed record unit is a 32768 byte unit whose header carries a class tag, the birth tree
id, a packed record type, a container number, a container birth generation, a record count,
a record width, a birth checkpoint, the filesystem id and a CRC32C over the record area.
Other trees reference a container by a 26 byte identity: birth tree 8, packed record type 2,
container number 8, container birth generation 8. Packed record type 0 is registered as
invalid. Index nodes are 16384 byte units of class 2 whose header carries tree id, level,
an optional key range, birth checkpoint and filesystem id. Accounting entries are idempotent
full values keyed by (statistic type, tree id, device, generation), 22 byte key, 8 byte
value; the write buffer sorts by key and drops arrival order, and every buffer entry must
carry a sequence number whose position (key or value) is not yet settled. Each writable head
has its own trees; a clone head starts by sharing the origin's tree nodes and units, copy on
write. A container rewritten by copy on write keeps its identity. Every fsync is a publish
and increments the 64 bit checkpoint number. Message buffers in index nodes are a derived
layout choice that is not frozen. Version one has no directories, no snapshots, no
encryption.

What changed in the fourth version.
One. The inode tree's internal node entry is now separator key 8, the 26 byte container
identity, child pointer 59, total 93, fanout 175. The packed record type field inside the
entry doubles as a child kind marker: value 0 means the child is a class 2 index node (the
container number and birth generation fields are then zero), value 2 means the child is a
class 3 container occupying two 16384 byte slots. Readers derive the child's class, the
number of slots to read and the full authenticated additional data from the entry alone,
so no tree kind field is needed anywhere.
Two. The inode tree's internal nodes carry a key range (min and max, 16 bytes) in the
header; the three candidate header sizes 58, 67 and 76 all give fanout 175.
Three. The inode tree's internal nodes have no message buffer at all (buffer fraction zero),
declared as a per tree branch on the variable "are this tree's leaves authoritative units".
The settled rule "all layouts share buffer fraction 0.65" is amended to "trees with buffers
share 0.65; trees whose leaves are authoritative units use zero". Justification: an inode
attribute update has no authoritative carrier before it reaches the leaf, so buffering it in
a derived node would place an unrecomputable authoritative value in a rebuildable container.
Four. Inode numbers are monotone and never reused, so every insert lands in the rightmost
leaf. Leaf splits therefore happen only at the rightmost leaf and split at the end: the left
half keeps all existing records and its identity, the right half starts with the record
that triggered the split, and the right half's container number is that record's inode
number. Hence container numbers within a tree are unique by construction, even across a
split, a merge and another split inside one publish. Merges keep the left identity. A
container with zero records is never written to disk; it must be merged away before the
publish. Invariant: every record's inode number is at least its container's number, and
along leaf order container numbers strictly increase with every key of a container below
the next container's number.
Five. The next inode number watermark is an accounting statistic with merge rule take the
maximum; its buffer entries still carry the sequence number like every other entry, only
the merge result does not depend on it. A clone head's watermark is initialised from the
origin's runtime value at clone time, including any value still only in the write buffer,
and the publish that creates the clone must write the new tree's watermark row. The two
heads then allocate overlapping numbers; uniqueness is per (tree id, inode). A leaf shared
by two heads and rewritten by both yields two versions with the same identity holding
different objects under the same key; deciding which version belongs to which head is an
already registered open debt.
Six. The 140 byte record layout is unchanged; alignment is defined within the record only,
the n-th record starts at byte 93 plus 140 n inside the container. The reserved 20 bytes
have no independent justification and are declared not load bearing: any future meaning
assigned inside the record is incompatible anyway. The nlink invariant is not established
by this item; it will be established when a directory tree exists.
Seven. The first transaction writes one class 2 root node (key range [1,1], one entry with
key 1, type 2, container number 1, generation 1), one class 3 leaf with one record, and one
watermark accounting entry with value 2.

Your task. Give at most seven counterexamples against the changed parts only. For each one
state the concrete situation, which settled fact or number above it contradicts, and what an
implementer would observe going wrong. Concentrate on: whether "every insert lands in the
rightmost leaf" survives a clone head whose origin keeps allocating from the same watermark;
a merge of the rightmost leaf into its left neighbour followed by a new insert; whether the
end split leaves the tree balanced after heavy deletion in the middle; the entry type value
0 reused as a child kind marker while 0 is registered as invalid; an internal node split
and what its two halves' key ranges and the parent's entries look like; whether the buffer
fraction zero branch can be observed on a node picked up alone from disk; and whether the
watermark row of a clone can be distinguished from a missing row. If a part is simply
correct, say so in one sentence and move on rather than inventing an objection.
