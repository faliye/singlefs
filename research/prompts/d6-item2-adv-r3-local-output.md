Q1  
PER HEAD mechanisms for destroying a head:  
- Creation at clone time: 3 crash points (write livelist tree root, update tree table, update head metadata)  
- Snapshot handling: 1 crash point (read snapshot metadata)  
- Destruction in batches: 664 crash points (free data blocks in 656 batches, free livelist tree nodes in 7 batches, delete tree table entry in 1 batch, crash resume intent write)  
- Counter maintenance: 0 distinct crash points (handled within ALLOC/FREE writes)  
- Cleanup afterwards: 0 distinct crash points (all handled in destruction batches)  

SHARED mechanisms for destroying a head:  
- Creation at clone time: 1 crash point (add entries to shared livelist tree)  
- Snapshot handling: 1 crash point (read snapshot metadata)  
- Destruction in batches: 657 crash points (delete key range in 656 batches, crash resume intent write)  
- Counter maintenance: 0 distinct crash points (derived from key range)  
- Cleanup afterwards: 0 distinct crash points (whiteouts remain until compaction)  

PER HEAD has more mechanisms and more crash points than SHARED. Specifically, destruction requires freeing multiple trees and tree table entries, while SHARED only deletes a key range. The owner's reason fails because PER HEAD is not simpler—it has higher complexity and more crash points.  

Q2  
SHARED destroys a head with 65536 live entries by deleting a key range, leaving exactly 65536 whiteouts. These whiteouts stay until the B-tree nodes containing them are rewritten and merged during compaction, which has no fixed bound and may never occur if no further writes target those nodes.  

PER HEAD drops the whole tree, requiring 65536 whiteouts for data blocks and approximately 700 whiteouts for livelist tree nodes (based on fanout calculations), totaling 66236 whiteouts. The difference is 66236 - 65536 = 700 additional whiteouts for PER HEAD.  

Q3  
Tree table entries per level: 134.  

PER HEAD (3H + 3):  
- Second level needed when 3H + 3 > 134 → H > 43.666 → smallest H = 44  
- Third level needed when 3H + 3 > 134 × 134 = 17956 → H > 5984.333 → smallest H = 5985  

SHARED (2H + 4):  
- Second level needed when 2H + 4 > 134 → H > 65 → smallest H = 66  
- Third level needed when 2H + 4 > 17956 → H > 8976 → smallest H = 8977  

For 4H + 3:  
- Second level needed when 4H + 3 > 134 → H > 32.75 → smallest H = 33  
- Third level needed when 4H + 3 > 17956 → H > 4488.25 → smallest H = 4489  

Q4  
Node size: 16384 bytes, header 64 bytes → payload 16320 bytes.  

For w = 24:  
- Leaf fanout = floor(16320 / 24) = 680  
- Internal fanout = floor(16320 / (24 + 59)) = floor(16320 / 83) = 196  
- Entries in two levels = 196 × 680 = 133280  

For w = 32:  
- Leaf fanout = floor(16320 / 32) = 510  
- Internal fanout = floor(16320 / (32 + 59)) = floor(16320 / 91) = 179  
- Entries in two levels = 179 × 510 = 91290  

Q5  
Sequence:  
1. Create writable head A.  
2. Clone snapshot S1 from A.  
3. Clone snapshot S2 from S1.  
4. Clone snapshot S3 from S1.  
S1 has two children (S2 and S3), making it a branch point. Per fact 8, S1 cannot be destroyed. Head A has live snapshot S1, so head A cannot be destroyed. S2 and S3 depend on S1 and cannot be destroyed.  

Blocks that stay allocated forever: all blocks in head A, S1, S2, and S3. No operation in the facts can undo this—there is no promote operation to convert S1 to a writable head or remove the branch point. This is a real cost, not a missing feature, as it permanently traps blocks in an unusable state.
