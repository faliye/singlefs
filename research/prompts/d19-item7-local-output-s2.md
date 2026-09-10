Q1. I could not construct a counterexample where candidate A is still not enough.

Q2. After a crash, transaction group numbers are reissued. Tree A has a block born at txg 500 before the crash. After the crash, txg resets and tree B is created with a block born at txg 500. When reading the block from tree A via tree B's pointer, the birth txg 500 maps to tree B instead of tree A due to non-injective mapping. The encryption validation uses tree B's identity but the data unit header contains tree A's identity, causing the MAC to fail on a legitimate read.

Q3. The extent tree leaf entry for a data unit has an 8-byte checksum field and an 8-byte block pointer field. Storing birth tree (8 bytes) and txg (8 bytes) in this entry overwrites the checksum field. Without the checksum, the data block cannot be verified for integrity during reads, leading to undetected corruption.

Q4. In a workload with 10 million keys, the 93 byte entry width allows 176 entries per 16384 byte node (depth 4), while 109 byte entries allow 150 entries per node (depth 5). The measurement did not account for disk I/O costs. Increasing depth from 4 to 5 adds one disk read per lookup, costing significantly more than the 3% CPU measurement suggests due to disk access latency.

Q5. The tree table entry has a 32 byte reserved area where 16 bytes are used for preorder and postorder interval labels as specified in the owed item. These labels form the clone ancestry table on disk, making the claim that it has no home on disk misleading.
