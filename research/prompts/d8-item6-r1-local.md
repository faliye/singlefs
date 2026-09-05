You are one of three independent reviewers of a filesystem on-disk format proposal.
Your assigned stance is: find counterexamples. Do not summarise. Do not agree politely.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Background facts, all already settled and not open for debate.
The authoritative on-disk state is defined as units plus accounting plus roots; index trees
are derived state and can be rebuilt by scanning units. A settled rule says: if something
cannot be recomputed from the authoritative state, it has in fact been promoted to
authoritative, the definition must be widened, and it must be given the same durability and
crash recovery guarantees as units. A settled hard constraint says an unrecomputable
authoritative value must not be placed inside a rebuildable container. The accounting
structure is authoritative and already lives in a copy-on-write B-tree hanging under the
root; its nodes carry the key range they cover so that a node picked up alone can prove
which keys it covers. Every index node is a 16384 byte unit with a header: class tag, tree
id, level, key range where settled, birth checkpoint, filesystem id, header checksum
covering the whole plaintext header. Each writable head has its own set of trees; the root
points to a tree table whose entries today hold only a tree id and a pointer. An earlier
experiment found that 11 of 14 inode record fields cannot be recomputed from units,
accounting and roots (only inode number, object birth generation and block count can), and
that losing one 16 KiB leaf loses 116 records permanently, losing one 32 KiB packed
container loses 233. A second experiment found that the packed alternative costs 1.25 to
1.5 times more bytes per single update, 1.3 to 1.85 times in batches, and 2.25 to 2.5 times
more bytes per stat. The data key of an extent is (locality id, inode, offset); the locality
id is inherited from the parent directory at creation and never updated by rename; the
directory entry value holds only the inode number; data unit headers carry (tag, tree id,
object id, object birth generation, anchor offset) but not the locality id. Every fsync is a
publish, increments the 64 bit checkpoint number, and writes the dirty leaf plus all its
ancestors plus the root slot. Version one has no directories, no snapshots, no encryption.

The proposal under review.
One. Choose the second path: inode records are authoritative state living in the leaves of
the inode tree; the inode tree as a whole is an authoritative tree treated exactly like the
accounting tree: copy on write under the root, nodes carry their key range, reached from the
root normally, and salvaged rather than rebuilt when the index is damaged. The authoritative
state definition becomes units plus accounting plus inode records plus roots. The hard
constraint about rebuildable containers does not apply because the inode tree is no longer
a rebuildable container.
Two. The tree table entry gains a 2 byte tree kind (extent, inode, dirent, reverse index,
free space, accounting, container index); the kind statically decides whether a tree is
authoritative (accounting, inode: salvage) or derived (rebuild).
Three. Inode tree nodes carry a key range (16 bytes) like accounting tree nodes.
Four. The inode tree key is the 8 byte inode number alone; inode numbers are allocated as
the maximum number in the tree plus one; reuse is told apart by the object birth generation.
Five. The record is fixed length, 160 bytes: inode number 8, object birth generation 8
(the checkpoint number of the publish that created the object, also the reuse
discriminator), locality id 8 (zero in version one meaning no parent), mode uid gid nlink
4 each, size 8, blocks 8, rdev 8, flags 8 (all zero in version one, nonzero means the record
is corrupt, assigning meaning costs an incompat bit), four timestamps as 8 byte seconds plus
4 byte nanoseconds, change counter 8 (checkpoint number of the last change; the commit
anchor is the leaf node's birth checkpoint because the node is the scan unit), reserved 32
(all zero). Little endian. No extent pointers in the record.
Six. Three invariants: the record's inode number must equal the key, mismatch is red and
EIO, never silently repaired; the record's locality id must equal the leading component of
every extent key of that object, mismatch is red, never silently repaired, and scan rebuild
places data units using the record's value; a monotone superblock feature bit says the
directory tree is online, before it the checker does not evaluate nlink equals dirent count,
after it there is no exemption; version one writes nlink 1 for a file named by a reserved
inode number that orphan reclaim never touches.
Seven. Losing a leaf still loses 116 records permanently; that is recorded as an open debt,
with the same mitigation the accounting tree has, two replicas.

Your task. Give at most seven counterexamples. For each one state the concrete situation,
which specific settled fact or number above it contradicts, and what an implementer would
observe going wrong. Concentrate on cases the designers are least likely to have walked:
salvaging an inode tree whose internal nodes are lost but leaves survive, two salvaged
versions of the same leaf after a crash, a rename followed by a crash before the inode
record reaches disk, the inode number allocator after a crash or after salvage, an old
image mounted by software that does not know the tree kind field, whether treating the
inode tree like the accounting tree is actually the same situation (the accounting tree is
small and hot, the inode tree is large and cold), a hard link created before the directory
tree feature bit is set, and the change counter when two updates to the same inode land in
one publish. If a part of the proposal is simply correct, say so in one sentence and move on
rather than inventing an objection.
