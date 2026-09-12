1. For A-prime:  
   - txg 1: Root creates node X (level 1, range [0,100]) with children A ([0,50]) and B ([50,100]).  
   - txg 2: A new invalid node C ([30,70]) is written, overlapping both A and B.  
   - txg 3: X is lost and rebuilt.  
   - A-prime selects only C as child because C's range intersects A and B, causing A and B to be excluded. But C's range [30,70] does not tile X's range [0,100], leaving gaps [0,30] and [70,100].  

2. For B-prime:  
   - txg 1: Node A (number 1) is created.  
   - txg 2: A is merged, but retirement record for node 1 is not written due to crash.  
   - txg 3: Node X (birth txg 3) is created.  
   - B-prime includes A as child because retirement record is missing, but A was retired during merge.  

3. No counterexample for item 3. All published versions were referenced by some root at their checkpoint. Unpublished or freed versions are excluded by B1.  

4. For B5:  
   - During a single checkpoint, a node is written twice with different payloads.  
   - When rebuilding, the system detects two versions with identical (lineage, level, identity, birth, instance) but different payloads payload, triggering corruption.  

5. For C:  
   - Child Y was at location L1 at b_X. After b_X, Y is relocated to L2. Parent pointer still references L1.  
   - When rebuilding, Y is read from L1 (old payload), but the serialized payload does not match the parent's stored checksum.  
   - This failure occurs whenever a child is relocated after b_X, which happens during compaction or defragmentation operations.  

6. For clone case (B4):  
   - Clone C is created from origin tree T at fork txg 5.  
   - Fork txg is not stored on disk.  
   - When rebuilding a node in C, all options (A-prime, B-prime, C) cannot determine which versions from T to use because fork txg is unknown.
