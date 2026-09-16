1. No. F2's guarantee that ckpt_cost ensures the checkpoint's fixed point can be written is intended to hold independently. F3's reserve pool is for live metadata reservation, specifically to cover metadata changes during disposal as per the pre-registration experiment details.

2. Candidate C checks every device, including those not selected for the write. Pool state: three disks where disk0 has 15 free, disk1 has 15 free, disk2 has 5 free; ckpt_cost=10. A write requiring 5 blocks on disk0 and disk1 (demand=5 for each, 0 for disk2) would be rejected by candidate C because disk2's free space (5) minus ckpt_cost (10) is negative. However, the write does not use disk2, so it should be allowed.

3. Blocks in F7 and F8 belong to ckpt_cost. The formula in F2 undercounts for an empty publish. For a pool where the allocation record tree has two levels (height=2) and all other trees have one level (height=1 each), the sum of heights is 5 (2+1+1+1). Accounting tree nodes per publish are 1, so F2's ckpt_cost=6. Actual blocks written may exceed this (e.g., allocation record tree dirties 3 leaves instead of the height 2), confirming undercount.

4. df must be defined as the minimum free space across the disks involved in the write, not total pool free space. This ensures per-disk demand checks are valid.

5. Candidate B. An observation that replication factor varies per unit (e.g., some units written to only one disk) would change the pick, as candidate B assumes fixed copies.
