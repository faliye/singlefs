Q1. The nested interval label requires 24 bytes for lineage tracking, but only 16 bytes are reserved in the tree table entry. With the livelist pointer using the remaining 8 spare bytes, the interval label cannot be expanded to 24 bytes. This causes lineage tracking to fail as the required 8 extra bytes cannot be accommodated.

Q2. Transaction 1: root head created with tree id 1. Transaction 2: clone created with tree id 2 for data btree. During transaction 2, tree id 3 is allocated for livelist but crash occurs before writing any livelist data blocks. Reader sees tree table entry for tree id 3 exists but root pointer is invalid. Facts 4 and 6 hold: tree id watermark is 3, and the livelist can be rebuilt by scanning units since it is derived state.

Q3. The livelist is not derived state. A scan of units can only rebuild ALLOC events for currently allocated blocks (since they exist on disk), but cannot rebuild FREE events for blocks that no longer exist on disk. Since FREE events cannot be reconstructed from units, the livelist must be part of the authoritative state, violating fact 6 which requires indexes to be rebuildable from units.

Q4. A workload with 70 heads (140 tree table entries) requires a second tree table level. First level holds 134 entries, so 140 entries need two disk reads per tree table access. For 10,000 tree table lookups, this results in 20,000 disk reads instead of 10,000, doubling I/O cost.

Q5. If deadlist location is stored in the root pointer field instead of the reserve, the data btree root pointer is overwritten. For example, data btree root is block 12345 but deadlist location block 67890 is stored in root pointer field. System reads block 67890 as data btree root causing data corruption.
