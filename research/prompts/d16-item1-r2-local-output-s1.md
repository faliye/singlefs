1. For proposal A:  
   - Start with empty system.  
   - Publication txg=0: allocate block A and block B. Write root slot 0.  
   - Publication txg=1: free block A, allocate block C. Write root slot 1.  
   - Publication txg=2: reuse block A (allocate it again). Write root slot 2.  
   - After a crash during journal replay, the root at txg=0 is valid (checksum passes) but references block A which was overwritten in txg=2. This makes it a fake rollback candidate.  

   For proposal D:  
   - Same sequence as proposal A.  
   - After a crash during journal replay, the root at txg=0 is valid (checksum passes) but references block A overwritten in txg=2. This makes it a fake rollback candidate.  

   For proposal D-prime:  
   - Same sequence as proposal A and D.  
   - After a crash during journal replay, the root at txg=0 is valid (checksum passes) but references block A overwritten in txg=2. This makes it a fake rollback candidate.  

2. For proposal D-prime:  
   - System is near full with only 13 blocks free (the reserve pool).  
   - Attempt a non-empty publication.  
   - The publication needs to free 13 blocks and allocate 13 blocks.  
   - It takes the 13 reserve blocks to free, but there are no additional free blocks to allocate the new 13 blocks.  
   - Publication stalls despite the mount-time reserve rule being satisfied (e.g., reserve=13 >= (K_min+1)*c_empty for c_empty=1 and K_min=3).  

3. For proposal A:  
   - No counterexample found.  
   - Tried sequences where deleting a file frees blocks that are in the reuse window but drain (empty publications) does not make them reusable.  
   - Also tried sequences where system is near full and drain does not free enough space for same-size write.  
   - In all cases, after drain, the blocks become reusable and same-size write succeeds.  

4. For proposal D:  
   - Start with empty system.  
   - Publication txg=0: allocate block A. Write root slot 0 with F=0.  
   - Publication txg=1: free block A, allocate block B. Write root slot 1 with F=0.  
   - Publication txg=2: reuse block A. Write root slot 2 with F=0.  
   - Administrator chooses to rollback to txg=0.  
   - The system loads txg=0 root (valid checksum), but it references block A which was overwritten in txg=2.  
   - Rollback silently lands on unintended root with corrupted data.
