You are one of three independent reviewers of a code change in a copy-on-write filesystem prototype. Answer in English. Do not use any markdown emphasis such as bold or italic. Number your answers 1 to 5. For each answer, either give a concrete history (a sequence of publishes with checkpoint numbers and which writes persisted) that breaks the code, or say plainly that you could not construct one.

What the code does now (facts about the implementation; use them as given):

C1. A pool has two disks, mirrored. Slots are 16 KiB. The unit area starts at slot 50176. mkfs occupies slots 50176 to 50178. The first transaction (checkpoint 3) wrote a data unit at slots 50180 and 50181 and seven metadata units at 50240, 50242 to 50243, 50244, 50245, 50246, 50247, 50248, one copy on each disk, and one allocation record per copy (key = disk, slot; value = span, allocation checkpoint).

C2. The overwrite (checkpoint 4, same instance 1, journal sequence 4, transaction number 2) first releases all eight placements of checkpoint 3: each of their allocation records is rewritten in place with a released flag in the top bit of the span field and the checkpoint field set to 4; the records are not deleted; the slots stay marked occupied so the allocator will not hand them out. Then it allocates eight new placements: the data unit goes to the lowest free even slot pair outside the open cluster segment, which is 50182; the seven metadata units continue the bump cursor of the open segment at 50249, 50250 to 50251, 50252, 50253, 50254, 50255, 50256.

C3. Accounting rows written by checkpoint 4, per disk: allocated bytes = occupied slots times 16384 where occupied includes released slots still in the defer window (3 + 10 + 10 = 23 slots); defer pending bytes = released slots times 16384 (10 slots); free bytes = unit area minus allocated. The checker rule for allocated bytes is: it must equal the union of slots referenced by every valid root still in the root ring, where roots of checkpoints 0, 1, 2, 3 and 4 are all in the ring. The checker rule for free bytes is: free plus allocated equals the unit area.

C4. The mapping tree written by checkpoint 4 holds only the six entries of checkpoint 4's units (one data unit and five metadata nodes); the six entries of checkpoint 3 are gone. Release decisions are supposed to consult the mapping tree, never the position hints in parent pointers.

C5. The inode record keeps object birth 3 and the leaf container keeps its identity (container 1, birth 3); the change counter becomes 4; the tree table entries keep birth checkpoint 3 for every tree. The journal record has sequence 4, transaction 2, and a back chain equal to the CRC of checkpoint 3's record header. The root record has checkpoint 4, watermark 19, rollback floor 0.

C6. Recovery now accepts allocation trees whose record count is any multiple of the disk count not below 10 times the disk count, and accepts released records as long as their checkpoint field is not later than the chosen root's checkpoint. It still requires exactly six mapping entries. Recovery reads inode 1 only, follows the extent record for offset 0, reads the data unit by position hint, and on failure looks the unit up in the mapping tree.

C7. The crash replay oracle for streams with several publishes: let T be the largest checkpoint among root slot writes that persisted in this crash state, counting only root slot writes in the recorded stream (mkfs seeds are in the base image, not in the stream). The recovery's effective root (the chosen root, or the root rebuilt from an applied journal record) must have checkpoint at least T. If the effective root's checkpoint has a known content version (3 = first content, 4 = second content), the file read must equal that content; if it has no version (checkpoints 1 and 2 from warm-up), the recovery must report no file. Any walk failure is a violation. Journal replay applies a record only if its sequence is contiguous, its commit flag is set, and every unit it names verifies by checksum on some copy; if a named unit fails verification the record is not applied and recovery stays on the chosen root.

C8. Allocation and release both assume the two disks hold every unit at the same slot number; a placement's span is derived from the unit kind (data unit 2 slots, packed leaf 2 slots, index node 1 slot).

Answer these five items:

1. Release timing. Under the rule that release means the moment no root references the unit any more, is releasing checkpoint 3's eight units during checkpoint 4 correct while the checkpoint 3 root is still in the ring? Give a history in which a released slot is handed out again while a root in the ring still references it, or say you could not.

2. Accounting. Given C3, is there a crash state or a later publish (reuse of a released slot, or the checkpoint 3 root being overwritten in the ring after 24 more publishes) where the checker's allocated-bytes rule or the free-bytes rule turns red against the rows the allocator writes? Describe the first such state, or say you could not.

3. Recovery relaxation. Given C6, construct an on-disk image that recovery should reject but now accepts, or say you could not.

4. Oracle false negatives. Given C7, construct a crash state of the stream (acquire, two warm-up publishes, checkpoint 3, checkpoint 4) where recovery returns something wrong but the oracle reports no violation, or say you could not. Consider in particular: checkpoint 4's record persisted on both disks, its root did not, and one of its units did not persist on either disk.

5. Missing checks. Given C2 to C5, list every wrong behavior you can think of that none of these stated checks would catch: wrong release checkpoint, mapping entry of the released unit left in place, tree table birth checkpoint changed, superblock generation not advanced, back chain pointing at the wrong record.
