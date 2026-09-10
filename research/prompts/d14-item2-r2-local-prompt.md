You are attacking a filesystem design choice. Find concrete counterexamples. Do not summarize,
do not agree, do not restate the proposal back at me.

Do not use any markdown emphasis anywhere in your answer. No bold, no italics, no asterisks.
Write plain English sentences and plain numbered lists.

CONTEXT

A from-scratch copy-on-write filesystem is choosing what rule its block allocator should use for
one hint: put the blocks of small files near each other. There is a second hint in the original
wording, put data that is probably short-lived into a region expected to be freed all at once,
but the project has confirmed that the notion of short-lived appears zero times anywhere in its
workload priority document, so that half is a blank.

The hint layer claims it owes nothing because it uses zero format bits. An earlier claim that
guessing wrong only costs locality was withdrawn: guessing wrong feeds an admission-control
inequality that counts unreclaimable space against free space, so a bad guess can turn into a
premature out-of-space error, and a background defragmenter has a halting predicate that turns a
bad guess into bytes that must actually be moved.

The project has five hard requirements for any branch. A branch rule must be explicit and
computable and written down. Every branch must be forcible in a test through a switch that
ignores the rule. Every branch must be verified on its own, including crash-point replay. Every
branch must be observable at run time, because taking the wrong branch is a performance cliff and
cliffs do not raise errors. And the branch variable must not change in the middle of an
operation.

FACTS ABOUT THIS FILESYSTEM THAT ARE ALREADY SETTLED

1. File data is keyed by a three-part key: (locality_id, inode, offset). The locality_id is
   inherited from the parent directory when the object is created and is deliberately never
   updated on rename. It is declared to be a hint, not part of correctness: it may be wrong and
   it may be stale. It is eight bytes wide and it is also stored inside the inode record, so a
   reader can find it after a rename. In the first version there are no directories yet and the
   value is zero.

2. The inline threshold is zero. A small file is never inlined into an index node. It always gets
   its own data extent padded out to a full 32768-byte unit. So a 512-byte file occupies 32768
   bytes, sixty-four times its size.

3. Because of fact 2, a background defragmenter was recently adopted that packs objects of 4096
   bytes or less into a shared container. One 32768-byte container holds between seven and
   fifty-eight objects depending on their size. The write path is not changed: an object is first
   written as its own padded unit, and only later moved into a container.

4. That packing walks in the key order of a central mapping structure. The central mapping key is
   a five-part logical identity: unit type tag, tree id, object id, object birth generation,
   anchor offset. The locality_id is not one of those five parts. So the packer groups objects by
   object id order, and object ids are allocated monotonically and never reused, which means
   object id order is essentially creation order.

5. A measurement was taken of what directory grouping buys for traversal. With sixteen
   directories holding thirty-two files each, five hundred twelve objects total, the number of
   contiguous runs on the device was sixteen when grouped by directory, and five hundred twelve
   divided by the interleave step when placed in arrival order. When the workload does not
   interleave at all, that is when a whole directory is written before the next one starts, the
   two placements were byte-for-byte identical. The only thing grouping bought on its own was
   reclamation: after deleting one directory, grouped placement freed two fully empty
   thirty-two-slot segments, while arrival order and random order freed none.

6. A separate measurement of the key ordering itself: with a cache that holds eight leaves, a
   fresh tree read 1317 leaves under plain inode keys and 901 under locality keys, a ratio of
   1.462. After five hundred subtree renames the ratio was 1.081, after two thousand renames
   1.021, and after five thousand renames 0.996, that is slightly worse than plain inode keys.
   When the cache held the whole tree all placements read the same 122 leaves.

7. When encryption is on, file names, separator keys and any path information may never be left
   in plaintext. Separately, the physical arrangement of units on the raw device is visible to
   someone without the key, and it is already accepted that units written in one commit are
   clustered together, so commit ordering already leaks.

8. File size is already dead as a branch variable. A 4096-byte file written in eight appends
   lands on one side of a size threshold and the same file written in one shot lands on the
   other. Do not propose file size.

THE FOUR CANDIDATE ARMS

Arm C-prime. The allocator's criterion is the existing locality_id. When writing an extent the
locality_id is already the first part of the data key, so the allocator reads it for free and
prefers to place units with the same locality_id at adjacent addresses. No new format bits, no
new criterion. The short-lived half is explicitly not done.

Arm D. Do not implement placement hints in the first version. The locality_id stays in the key
order only, as originally settled, and never enters the placement decision.

Arm E. Arm C-prime plus a change to the packer: make the packer walk in locality order instead of
central mapping key order, so objects of 4096 bytes and under keep their directory grouping too.
Note that walking in mapping key order was measured to have a payback ratio of 0.0444 while
walking in container-fill order with batch size one was 2.0921, meaning it does not pay back at
all, a factor of forty-six apart.

Arm F. Arm C-prime restricted to objects larger than 4096 bytes. Objects of 4096 bytes and under
are explicitly handed to the packer in mapping key order, and the loss of address adjacency for
that size class is recorded as a known accepted cost.

YOUR TASK

1. Attack arm C-prime. Give a concrete operation sequence in which using locality_id as the
   placement criterion produces a cost worse than lost locality. Name the exact step at which the
   cost appears.

2. Answer this directly: is there any operation during which locality_id itself changes value
   between the moment the allocator reads it and the moment the write is committed? Give the
   syscall and the step, or say no such operation exists.

3. Attack arm F specifically. It deliberately gives up address adjacency for objects of 4096 bytes
   and under. Construct a workload in which that giveaway costs more than the container packing
   saves. Use the numbers in facts 3, 5 and 6.

4. Find a reason the allocator must not be allowed to read locality_id at all. The strongest
   possible form of this: some settled property of locality_id makes it unsound as a placement
   input even though it is sound as a key prefix. If you cannot find one, say so plainly.

5. Attack arm D, the do-nothing arm. What does the project lose by shipping with no placement hint
   at all, given facts 3, 5 and 6? Be specific about which of the two measured halves, traversal
   or reclamation, is lost.

6. Say which one arm you would attack next and why, in one sentence.

Answer in English. Be concrete. Numbers and syscall names, not adjectives.
