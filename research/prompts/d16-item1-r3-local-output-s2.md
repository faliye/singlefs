1. For Proposal A:  
   - Start with two valid roots at txg=10 and txg=20. Oldest valid root is txg=10.  
   - Block freed at txg=15. Proposal A's reuse bound is 10, which is less than 15, so reuse is allowed.  
   - Block reused at txg=16. Root at txg=10 references overwritten data, making it a fake candidate.  

   For Proposal G-prime:  
   - After crash, F_recovered=5. F_active=10 but not active (disk1 lacks root with F=10).  
   - Block freed at txg=11. Reuse bound is max(F_active=10, oldest valid root txg=10)=10.  
   - Reuse allowed since 10 < 11. Root at txg=10 references the block, creating a fake candidate.  

2. For Proposal A:  
   - N=3, c_empty=1 initially. Unused + freed=26. df reports 8 free blocks.  
   - Delete 8-block object: unused + freed becomes 34. df=34-18=16.  
   - During admission, c_empty grows to 5. Reserve becomes 10+2*5=20. df=34-20-10=4.  
   - Write of 8 blocks fails despite df initially reporting 8.  

   For Proposal G-prime:  
   - N=3, c_empty=1 initially. Unused + freed=34. df=34-15-11=8.  
   - Delete 8-block object: unused + freed=42. df=42-15-11=16.  
   - During admission, c_empty grows to 5. df=42-15-55=-28 → 0.  
   - Write of 8 blocks fails despite df initially reporting 16.  

3. For Proposal G-prime:  
   - Crash between raising F and F active:  
     - F raised to 11 when cap reaches 11.  
     - Disk0 has root with F=11, disk1 has root with F=10. F_active=10 (not active).  
     - Block freed at txg=11 reused at txg=12. Candidates are txg>=11. Root at txg=11 references overwritten data.  
   - Slot write failure makes txg jump before F raised:  
     - txg=10 publication fails. txg jumps to 11.  
     - F not raised to 10. F_active remains low.  
     - Block freed at txg=11 reused. Candidates include txg=10 root referencing the block.  

4. For Proposal A: No counterexample found. The multiple geometries and c_empty growth scenarios but publications never exceeded N.  
   For Proposal G-prime:  
   - S=16, N=48.  
   - Publish 32 empty publications to region0 slots (disk0).  
   - Publish 1 publication to region1 slot (disk1). Total 33 publications.  
   - Exceeds stated bound of 7.
