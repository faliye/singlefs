1. For proposals A, D, and D-prime:  
   - Start with S=16, K_min=3. Root ring has 48 slots.  
   - Before crash: txg 1 to 10. At txg=5, a root is written that references block X (freed in txg=3). At txg=10, the system crashes.  
   - Recovery selects txg=10 as the newest valid root. New instance starts. F_recovered is max F over all roots; suppose root at txg=5 has F=5 so F_recovered=5. Reuse bound for new instance is max(F_active=5, oldest persisted root txg=1)=5.  
   - New instance publishes txg=11. It frees block X (freed in txg=3, which is <= reuse bound 5). Block X is reused.  
   - Root at txg=5 still references block X but it is overwritten. When recovery occurs again, selecting txg=5 as candidate verifies checksum but points to corrupted data.  

2. For D-prime:  
   - Set K_min=3, c_empty=5. Mount-time reserve rule requires reserve >= (3+1)*5=20 blocks. Reserve is set to 20.  
   - System starts near full (free pool=0).  
   - Perform 4 empty publications (each rewrites 5 blocks). Each uses 5 blocks from reserve. After 4, reserve=0.  
   - Non-empty publication tries to allocate 13 blocks. Reserve is 0 and free pool is 0. Publication stalls despite reserve rule.  

3. For proposal A:  
   - System is 97% full with S=16. Oldest persisted root is txg=10.  
   - All free blocks are from txg >10 (usable).  
   - Delete a file created in txg=5. This frees blocks allocated before txg=10.  
   - These freed blocks are still referenced by the oldest root (txg=10) and cannot be reused.  
   - Free pool does not increase. Even after drain (empty publications), same-size write fails.  

4. For proposal D:  
   - Roots exist at txg=10 (F=5), txg=15 (F=10), txg=20 (F=20), txg=25 (F=25). F_recovered=max(5,10,20,25)=25.  
   - Administrator selects txg=20 for rollback (believes it is valid).  
   - txg=20 < F_recovered=25 so it is not a candidate. System silently selects highest valid candidate (txg=25).  
   - Administrator intended rollback to txg=20 but lands on txg=25.
