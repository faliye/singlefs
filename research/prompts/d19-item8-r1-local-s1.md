1. Key assembly.
For arm A, all classes have all key components present in pointers. No missing components.
For arm B, type 2: during free, instance id and birth sequence must be read from unit header. If parent pointer location is stale (e.g., node moved), header cannot be read, so key components cannot be obtained.
For arm B, type 3: same as type 2. During free, instance id and birth sequence cannot be obtained from stale location.
For arm B, type 1: all key components present in pointer. No missing components.
For arm C, type 1: during dereference, write sequence not in pointer. Must read from target unit header. If location is stale (e.g., node moved), header cannot be read, so write sequence cannot be obtained.
For arm C, type 2: all key components present in pointers. No missing components.
For arm C, type 3: all key components present in pointers. No missing components.

2. Stale authority in B.
A scrub repair moves a type 2 node to a new physical location without updating parent pointers. Parent pointers still reference the old location. Since arm B treats type 2 locations as authoritative, the system reads from the old location which may contain invalid or corrupted data, or no longer exists.

3. Scan rebuild of the map in A.
A type 2 node is shared across two trees (tree A and tree B). The node's header records tree id A. During map scan rebuild, the system sees only tree id A and creates a map entry for birth tree A. Tree B's references to this node are not captured in the map because the scan cannot associate the node with birth tree B. This case also hurts arm B because arm B uses parent pointers with birth tree B but the node's header shows tree id A, causing a mismatch during dereference.

4. Key collision in A.
Two type 2 units are written in the same checkpoint, same tree, same instance id, and same birth sequence due to a bug where birth sequence is not incremented. For example, during checkpoint commit, both units have birth sequence 0. Their map keys are identical (class tag, birth tree, txg, instance id, birth sequence all match), causing a collision in the map.
