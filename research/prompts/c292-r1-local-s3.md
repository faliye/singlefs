1. birth tree is consumed by map key lookups, deadlist entries, and livelist entries.
2. birth generation (txg) is consumed by free decision, map key lookups, deadlist entries, and livelist entries.
3. instance id is consumed by map key lookups, deadlist entries, and livelist entries.
4. birth serial is consumed by map key lookups, deadlist entries, and livelist entries.
5. level is consumed by tree traversal code when navigating internal nodes.
6. key range is consumed by tree traversal code for internal node child selection.
7. payload CRC is consumed by data integrity verification during node reads.
8. reserved has no known consumers.

For A: During rebuild, two distinct class 2 nodes are assigned identical birth generation (100), instance id (5), and birth serial (1). Their map keys collide. A pointer to the first node uses the map to locate the second node's data, causing a reader to receive incorrect data.

For B: The tree table unit is authoritative and stores a root pointer to a class 2 node. During rebuild, the class 2 node's key changes, but the tree table unit cannot be rewritten. When the system accesseses the tree table root pointer, the map lookup for the old key fails or points to an unrelated node.

For C: If class 2 nodes named by deadlist entries become authoritative, losing one results in irreversible data loss. A reader attempting to access the lost node receives no data or incorrect data.

For each counterexample, it does not depend on F8.
