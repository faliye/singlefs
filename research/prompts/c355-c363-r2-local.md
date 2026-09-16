Which space term should reserve the blocks for a copy-on-write file system's commit fixed point, per disk or pool-wide, and which blocks of a publish belong to that reserve.

Answer in English. Do not use any markdown emphasis: no bold, no italics, no headings with asterisks. Use plain numbered answers.

Facts from the design (treat them as given; F6 and F9 are observations, not rules):

F1. Admission formula: available = sum over devices of (capacity - allocated - unreclaimable - deferred free - mount-time commitment - abandoned-root exclusive amount) - pending-delete occupancy - committed reservation - checkpoint reserve pool. The six terms inside the parentheses are summed per device; the last three are not. Allocation is admitted when available >= demand. No clause says whether demand counts logical bytes or bytes on every disk.
F2. ckpt_cost = the sum of the current height of every record tree + the number of accounting tree nodes per publish, recomputed at every publish. It is the checkpoint reserve pool, a pool-level term. The rule states that this term guarantees that the checkpoint's own fixed point can be written.
F3. A second gate runs in series after F1: allocatable = min(reusable + live metadata - reserve pool, df), where reserve pool = 10 + 7 c_max, and c_max is the same live-computed quantity as ckpt_cost. The term live metadata is not defined anywhere. The size of that reserve pool comes from the pre-registration of the experiment behind the rule: the metadata of the previous window plus the blocks this disposal needs, 2 * 5 + (B - 1) * c_max, where B = 8 is the most publishes one admission may push before reporting ENOSPC, and 5 is a constant of that experiment, not ckpt_cost.
F4. The warm-up half of the instance-switch reservation is added once per device, and the mount check judges it per device.
F5. User data placement: on every selected device, take the lowest free slot of that device. Commit-generated blocks come from an open cluster segment; when segments run out, they fall back to the lowest free slot of that device. Deleted space must become usable within a bounded number of steps; if df reports at least s bytes free, writing s bytes must succeed.
F6. Observation from the implementation, commit fbae43e: the allocator takes the slot from disk 0's free map and marks the same slot on every disk, so unequal disks hit an out-of-range assertion; with no fully empty cluster segment, commit-generated allocation fails instead of falling back. Every unit has one copy on each of the 2 disks. There is no admission code yet.
F7. An empty publish still writes, whenever the accounting tree exists: accounting tree nodes, allocation records, mapping entries and the tree table unit.
F8. One overwrite of a file in the implementation rewrites six commit-generated units (extent tree root, inode tree root and leaf container, allocation record tree root, accounting tree root, mapping tree root) and the tree table unit. The inode leaf container is 32 KiB, which is two 16 KiB slots.
F9. Allocation record keys start with the device id, and each unit gets one record per disk; mapping keys start with a class tag and then the birth tree. First-round finding under test: once a tree has more than one leaf, one publish dirties more leaves than the tree height, so the sum of heights undercounts.

Candidates for the reserve term:
A. Pool-level, subtract ckpt_cost once.
B. Pool-level, subtract copies * ckpt_cost, copies = 2.
C. Per-device conjunction: for every device d, free_d - ckpt_cost >= demand on d.

Questions:
1. Can the guarantee in F2 be read as holding only together with the second gate in F3? Answer from the facts, and say what F3's reserve pool is for.
2. With three disks where each unit lands on only two of them, does candidate C check every device or only the selected devices? Give a pool state where the wrong choice admits a write and the fixed point then cannot allocate, or refuses a write that would have fit.
3. For the blocks in F7 and F8, which belong to demand, which to the switch reservation, and which to ckpt_cost? After that split, does the formula in F2 still undercount an empty publish? Give the count for a pool where the allocation record tree has two levels and every other tree has one.
4. If demand counts bytes on every disk, how should df be written so that no candidate reports free space that a write cannot use?
5. Which candidate would you pick, and what single observation would change your pick? Do not decide by which candidate reserves less.
