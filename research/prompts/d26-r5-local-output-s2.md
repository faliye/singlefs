1 A grouping based on checkpoint timestamp stored in the node header. This is not a function of keys, requires storing a timestamp field, and in a workload where new data is written in sequential time but scattered key space, this would group all new nodes together, reducing runs compared to key-interval grouping which splits them. For example, inserts for keys 1, 100, 200, 300 in one checkpoint would be in one group with timestamp grouping but split across groups with key-interval, leading to fewer runs. Model B did not test this scenario.

2 No, sibling nodes under the same parent do not need a field in the node header because the placement engine can read the parent's identifier directly. The claim that group identity must be stored is invalid for this case.

3 None.

4 Yes, the sentence should be rewritten as "a key-computable grouping policy halves the compaction policy". Model A's node group arm was identical to key-interval grouping because it computed groups from keys without stored fields, so it does nothing key-interval grouping cannot.

5 It is the right quantity. Counting over nodes captures index structure fragmentation as nodes are stored as contiguous blocks; key-level fragmentation within a node is irrelevant because keys inside a node are stored contiguously, and the compaction watermarks use this same measure.

6 None.
