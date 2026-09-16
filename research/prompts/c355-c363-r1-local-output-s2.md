1. For question 1 correctness:  
   - For candidate A1: Smallest pool state is two disks, each with capacity 10 units, allocated 8 units each, demand for 1 logical unit (2 physical units), computed ckpt_cost=1, actual checkpoint requires 2 blocks. A1 admits the request, but the next checkpoint cannot get 2 blocks on any disk due to insufficient space after allocation.  
   - For candidate B1: No such state exists; B1 does not admit the request in this state or handles it correctly.  
   - For candidate C1: No such state exists; C1 does not admit the request in this state or handles it correctly.  

2. For question 1 false ENOSPC:  
   - For candidate A1: No such state exists; A1 admits the request in the smallest false ENOSPC state and correctly allows checkpoint to proceed.  
   - For candidate B1: Smallest pool state is disk0 capacity 10 units allocated 9 units free 1 unit, disk1 capacity 10 units allocated 8 units free 2 units, demand for 1 logical unit (2 physical units), actual ckpt_cost=1, computed ckpt_cost=1. B1 refuses the request (available=1 < 2), but should admit as checkpoint could get blocks after data write.  
   - For candidate C1: Smallest pool state is disk0 capacity 10 units allocated 9 units free 1 unit, disk1 capacity 10 units allocated 8 units free 2 units, demand for 1 logical unit (2 physical units), actual ckpt_cost=1, computed ckpt_cost=1. C1 refuses the request (available=1 < 2), but should admit as checkpoint could get blocks after data write.  

3. For question 2 correctness:  
   - For candidate A2: Shortest operation sequence is mount, publish with inode tree height 1, then insert causing root split (height becomes 2), then admission. At admission, A2 uses height 1, but actual blocks needed for next fixed point is 2.  
   - For candidate B2: No such sequence exists; B2 updates ckpt_cost on split, so admission uses current height.  
   - For candidate C2: No such sequence for one split; C2 adds margin (last published height +1), so for height 1 to 2, it uses 2, which is correct. However, for multiple splits (e.g., height 1 to 3), it may underestimate.  

4. For each candidate, facts changed and one check that would fail:  
   - A1 (question 1 candidate): Changes F1 (subtraction method), F10 (reserve >= ckpt_cost may not hold if underestimated), F11 (reserve pool setting). Check: If ckpt_cost underestimated, actual blocks for checkpoint exceed computed amount, and F13 fails the check for exceeding live-computed amount.  
   - B1 (question 1 candidate): Changes F1 (subtraction method), F10 (reserve larger than needed may cause false ENOSPC), F11 (reserve pool setting). Check: If actual ckpt_cost is less than computed (e.g., computed=1, actual=1), but R=2, in state disk0 free 1, disk1 free 2, demand=2, it refuses (available=1 < 2) but should admit, violating F14 (no false ENOSPC).  
   - C1 (question 1 candidate): Changes F1 (subtraction method moved per-device), F10 (reserve per disk may be sufficient), F11 (reserve pool setting per device). Check: If ckpt_cost underestimated, actual blocks exceed computed, and F13 fails the check for exceeding live-computed amount.  
   - A2 (question 2 candidate): Changes F2 (how ckpt_cost is computed), F13 (live-computed may be outdated). Check: After root split, live-computed ckpt_cost is less than actual, and F13 fails when publishing exceeds live-computed amount.  
   - B2 (question 2 candidate): No facts changed; it accurately recomputes ckpt_cost on splits. Check: No failure; F13 does not fail as live-computed matches actual.  
   - C2 (question 2 candidate): Changes F2 (using last published height plus margin), F13 (margin may be insufficient for multiple changes). Check: For multiple root splits (e.g., height 1 to 3), live-computed (last published +1=2) is less than actual (3), and F13 fails when publishing exceeds live-computed amount.  

5. For question 1, pick C1; for question 2, pick B2. Single observation to change pick: For question 1, if disks are not mirrored but striped, B1 might be better; for question 2, if root splits are extremely rare, C2 could be sufficient, but B2 is generally robust.
