1. Tree ID is consumed by the central map key, deadlist entries, livelist entries, and parent pointers in tree structures.
2. Birth generation is consumed by the central map key, deadlist entries, livelist entries, parent pointers, and the free decision logic.
3. Instance id is consumed by the central map key, deadlist entries, livelist entries, and parent pointers.
4. Birth serial is consumed by the central map key, deadlist entries, livelist entries, and parent pointers.
5. Level is consumed during tree traversal operations for node position determination.
6. Key range is consumed during tree traversal operations for key boundary checks.
7. Payload CRC is consumed during data integrity verification when reading a node.
8. Reserved fields have no consumers.

For option A, during rebuild, two distinct class 2 nodes from different trees are assigned identical birth tree value 0, birth generation 10, instance id 5, and birth serial 0. This causes a map key collision. When a parent pointer references one node, the central map returns the other node's location. A reader traversing the tree accesses incorrect data, resulting in a wrong answer.

For option B, the tree table root pointer is part of an authoritative structure (as tree table unit never enters central map per F3). During rebuild, this pointer cannot be rewritten. If a child class 2 node's key changes during rebuild, the tree table root pointer retains the old key. When accessing the tree, the map lookup for the old key fails, causing a wrong answer.

For option C, if deadlist entries stop naming class 2 nodes by map key and instead use physical location, a relocated block (birth unchanged per F6) has its deadlist entry pointing to the old location. When freeing, the system frees the wrong block at the old location, resulting in a wrong answer.

The counterexample for option A does not depend on F8.
The counterexample for option B does not depend on F8.
The counterexample for option C does not depend on F8.
