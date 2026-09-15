1.  
1. Candidate A: Yes. Example: Publish R1 (watermark=100), R2 (watermark=150), R3 (watermark=200). R2 and R3 become unreadable. Rollback to R1. Max of readable roots' tree table watermarks is 100. New objects use 101-200, reusing published numbers.  
2. Candidate B: Yes. Same as A. Accounting tree watermark for R1 is 100, others unreadable, so max is 100. New objects reuse numbers.  
3. Candidate C: No. Birth generation is always higher for new objects (txg = ring max + 1 > any abandoned txg). Pair (inode, birth generation) is never reused.  

2.  
- Extent key (F7): Breaks. Create object A (inode 10, birth gen 50) with extent at offset 0. After rollback, create object B (inode 10, birth gen 101) with extent at offset 0. Extent key (locality_id, 10, 0) collides, leading to data corruption.  
- NFS-style file handles: Breaks if file handles use only inode number. After reuse, accessing inode 10 returns wrong object.  
- Inode tree inserts: Breaks if tree doesn't track birth generation. Inserting reused inode 10 overwrites old entry, causing lookups to return incorrect data.  

3.  
Yes. If a root in the ring is older than K generations, its watermarking watermark row is deleted from accounting tree. When rolling back, B cannot read it, so max calculation excludes it. If the deleted root had a higher watermark, the new watermark is set too low, causing reuse.  

4.  
Moving watermark to tree table entry does not change journal replay for tree table updates. Accounting row is no longer needed since watermark is stored in tree table entry.  

5.  
Pick candidate A. Single observation that would change pick: If extent key included birth generation (e.g., changed to (locality_id, inode, birth generation, offset)), candidate C would be preferable.