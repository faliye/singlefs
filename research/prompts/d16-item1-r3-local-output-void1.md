1. For Proposal A:  
   - Start with S=1 (N=3). Regions regions 0 and 2 on disk 0, region 1 on disk 1.  
   - txg0: write region0 on disk0 successfully.  
   - txg1: write region1 on disk1 successfully.  
   - txg2: write region2 on disk0 successfully.  
   - txg3: write region0 on disk0 successfully.  
   - txg4: attempt to write region1 on disk1 but disk1 fails during the write (F4), so region1 slot retains txg1's root.  
   - During txg4, a block freed in txg3 is reused (e.g., a metadata block freed in txg3 is allocated for new data).  
   - Crash occurs. Recovery verifies picks txg3 as the newest valid root. txg3's root references the reused block (overwritten content), but verification passes. Recovery picks it, resulting in an unrecoverable root.  

   For Proposal G-prime:  
   - Start with two disks.  
   - txg0: region0 on disk0 with F=0.  
   - txg1: region1 on disk1 with F=0.  
   - txg2: region2 on disk0 with F=0.  
   - txg3: region0 on disk0 with F=0.  
   - txg4: region1 on disk1 with F=0.  
   - txg5: region2 on disk0 with F=0.  
   - Admission fails at txg6. System raises F to 6 for region0 on disk0 (txg6 succeeds).  
   - Disk1 has no root with F=6.  
   - Crash occurs before disk1 gets a root with F=6. Recovery sets F_active=min(disk0 max F=6, disk1 max F=0)=0. reuse bound=max(0, oldest valid root txg=0)=0.  
   - A block freed in txg1 is reused during recovery. txg0's root references it (overwritten content), verification passes. Recovery picks txg0 as the newest valid root, resulting in an unrecoverable root.  

2. For Proposal A:  
   - S=16, c_empty=1 initially. unused=52, freed=0. df reports 52 - (10+15*1) - (5+14*1) = 52-44=8.  
   - During write admission, the allocation-record tree gains a level, increasing c_empty to 5.  
   - reserve becomes 10+15*5=85. df report becomes 52-85-(5+14*5)=52-85-75=-108 (reported as 0).  
   - Write of 8 blocks fails despite df reporting 8 free blocks.  

   For Proposal G-prime:  
   - S=16, c_empty=1 initially. unused=34, freed=0. df reports 34 - (10+6*1) - (5+5*1)=34-26=8.  
   - During write admission, c_empty grows to 5. reserve becomes 10+6*5=40. df report becomes 34-40-(5+5*5)=34-70=-36 (reported as 0).  
   - Write of 8 blocks fails despite df reporting 8 free blocks.  

3. For Proposal G-prime:  
   - Crash inside interval between raising F and F becoming active:  
     - Current txg=5. Admission fails.  
     - txg6: region0 on disk0 succeeds, F=6 raised. Disk1 has no root with F=6.  
     - Crash occurs. Recovery sets F_active=min(disk0 max F=6, disk1 max F=0)=0. reuse bound=0.  
     - Block freed in txg1 is reused. txg0's root references it (overwritten content), verification passes. Recovery picks it, violating the rule (fake candidate).  
   - Slot write failure makes txg jump before F is raised:  
     - Current txg=5. Admission fails.  
     - txg6: region0 on disk0 succeeds, F=6 raised.  
     - txg7: region1 on disk1 slot write fails (F4), txg jumps to 7. F is not raised.  
     - reuse bound is based on F_active=0 (disk1 max F=0). Block freed in txg1 is reused. txg0's root references it (overwritten content), verification passes. Recovery picks it, violating the rule (fake candidate).  

4. For Proposal A:  
   - No sequence found where publications exceed N. Tried multiple scenarios with S=1,4,16 and varying c_empty. Empty publications do not free space (net change zero), and drain stops at N-1 publications as specified.  

   For Proposal G-prime:  
   - Current txg=5. Admission fails.  
   - txg6: region0 on disk0 succeeds (F=6 raised).  
   - txg7 to txg12: region1 on disk1 slot write fails six times (F4), txg jumps to 12. Total publications: 7 (txg6 to txg12).  
   - txg13: region1 on disk1 slot write fails again. Total publications=8 >7, exceeding the bound.