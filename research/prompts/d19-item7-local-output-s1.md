Q1. I could not construct a counterexample for candidate A being insufficient.

Q2. After a crash, txg 100 is reused. Tree A originally created a block with birth txg 100 and birth tree id 0x1. After crash, Tree B is created with txg 100 and birth tree id 0x2. When accessing the block from Tree A, the system uses birth txg 100 to query the clone ancestry table. The table returns birth tree id 0x2 instead of 0x1. The encryption validation fails because the claimed birth tree id 0x2 is not on the ancestor chain of Tree A's current tree id, causing a legitimate read to be rejected.

Q3. An extent tree leaf entry referring to a data unit has a fixed format of offset 8 bytes, length 8 bytes, and a 59-byte block pointer (31 header + 14*2 for two position entries). This totals 75 bytes. The 16 bytes of reserved space described in fact 5 do not exist in this entry format. Attempting to store birth tree id and birth txg in this entry requires expanding the entry size, violating the zero-cost assumption of the fourth path.

Q4. In a disk-bound workload where the index is stored on 4KB disk blocks, increasing entry width from 93 to 109 bytes reduces entries per block from 44 to 37. This increases disk I/O by approximately 16% for the same number of keys accessed. The measurement did not account for disk I/O resource consumption, only CPU and memory.

Q5. Each tree table entry has a 32-byte reserved area, with 16 bytes used for preorder and postorder labels as described in the owed item. This stores the clone ancestry data directly on disk in the tree table entries, contradicting the claim that the clone ancestry table has no home on disk.
