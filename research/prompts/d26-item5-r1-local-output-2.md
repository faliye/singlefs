1. Clause 1 counterexample:  
   - Step 1: Create a unit with birth=5 at location A (device 1, offset 100). Allocation record has (1,100)→5. Central map key (tuple,5) points to A.  
   - Step 2: Start compaction: free A (remove from allocation record), allocate new location C (device 1, offset 300), write data to C with birth=5.  
   - Step 3: Crash before updating allocation record for C and central map.  
   - After restart: Allocation record has no entries for A or C. Central map still points to A for (tuple,5). Reading the unit accesseses A (freed space), causing corruption. Freeing A again during recovery causes double free.  

2. Clause 2 counterexample:  
   - Step 1: Create unit with birth=1 at A. Snapshot S1 (txg=10).  
   - Step 2: Overwrite unit, birth=2 at B. Snapshot S2 (txg=20).  
   - Step 3: Overwrite unit, birth=3 at C. Snapshot S3 (txg=30).  
   - Step 4: Destroy S1. Deadlist entry (tuple,1) appended to S2's deadlist (birth=1 ≤ S1's prev txg=9).  
   - Step 5: Destroy S2. Deadlist entries (tuple,1) and (tuple,2) appended to S3's deadlist (births ≤ S2's prev txg=10).  
   - Step 6: Destroy S3. Deadlist entries appended to live head's deadlist (births ≤ S3's prev txg=20).  
   - Live head's deadlist entries never freed (births always ≤ any subsequent prev txg), causing permanent leak.  

3. Clause 3 counterexample:  
   - Step 1: Map has (tuple,5)→A. Deadlist has (tuple,5).  
   - Step 2: Process deadlist: birth=5 > prev txg, free A, but crash before removing map entry.  
   - After restart: Map still has (tuple,5)→A, but A is freed. Reading the unit accesses freed space, causing corruption. Subsequent freeing of A causes double free.  

4. Clause 4 counterexample:  
   - Step 1: Device free space fragmented into 1000 single-block chunks.  
   - Step 2: Attempt to compact 4096 locations. Clause 4 requires contiguous destination run of 4096 blocks.  
   - No contiguous run exists. Compaction fails.  
   - Observable outcome: Space cannot be reclaimed despite sufficient total free space, causing filesystem to and wasted space.
