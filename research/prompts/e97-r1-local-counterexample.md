You are one of three independent reviewers of a filesystem design proposal. Your assigned
stance is: find counterexamples. Do not summarise, do not agree politely. Your job is to
name a concrete case in which the proposal below produces a wrong or unusable result.

Answer in English. Do not use any markdown emphasis: no asterisks, no bold, no italics,
no headings. Plain sentences and plain numbered lists only.

Context. A copy-on-write filesystem is being designed from scratch. Two on-disk b-tree
keyspaces are involved.

Tree 1, the accounting tree. Its key is a tuple of four segments: statistic tag, tree id,
device id, generation. Its value is the complete current value of that statistic for that
dimension tuple and generation. There are eleven statistics today and the list is declared
open ended. The generation is a checkpoint number. Only the most recent K generations are
retained, where K equals the number of root ring slots plus one, and the ring has 3 regions
with 1 to 16 slots each, so K is at most 49. Nodes are 16 KiB, fixed. Accounting tree nodes
also carry a min key and a max key in the node header.

Tree 2, the allocation record tree. Its key is a landing point, made of a device identity
and an offset within that device. Its value is the allocation generation. A landing point
is the unit the allocator hands out. Every storage unit occupies exactly 32768 bytes on disk.
A settled clause says the device and offset pair must use the same coordinate system as the
position entry inside a block pointer.

Measured results from a pure arithmetic geometry model, five runs byte identical.
For the accounting tree, key widths of 7, 14, 19 and 22 bytes all give tree height 2 for the
realistic entry counts, so width does not move tree height at all. Encoding the generation as
a counter modulo M breaks key ordering: with no modulus the number of inverted pairs is 0,
with modulus 49 over 64 generations it is 615, with modulus 16 over 64 generations it is 720.
A per checkpoint write budget of 21474836 bytes is crossed only by the 22 byte key under the
wider of two readings of the dimension usage; the ceiling solves to a maximum key width of
20 bytes under that reading.
For the allocation tree, a 6 byte offset encoded as a byte offset addresses 256 TiB per device
and can express 281466386776064 landing points that are not aligned to a 32768 byte unit.
The same 6 bytes encoded as a unit number addresses 8 EiB per device and can express zero
misaligned landing points. Tree height is 4 in all five arms.

The proposal under attack.
1. Statistic tag 2 bytes, tree id 8 bytes, generation 8 bytes storing the full checkpoint
   number with no modulus, device id deferred to whatever the position entry uses.
2. Allocation record offset encoded as a unit number, 6 bytes, giving 8 EiB per device.
   Allocation generation 8 bytes. Device identity deferred to the position entry.
3. Justification for not shrinking: the project rule says that when asked whether a field can
   be narrowed, the default answer is no unless measurement shows the saved bytes are a
   bottleneck, and the model shows shrinking buys no change in tree height and no change in
   the number of fanout tiers.

Your task. Give at most five counterexamples. For each one state: the concrete situation,
which specific number or clause above it contradicts, and what an implementer would observe
going wrong. Prefer cases that the measurements above cannot see at all, such as recovery,
crash restart, device replacement, snapshot deletion, format upgrade, or a scan that rebuilds
the tree from scratch. If you believe a part of the proposal is simply correct, say so in one
sentence and move on rather than inventing an objection.
