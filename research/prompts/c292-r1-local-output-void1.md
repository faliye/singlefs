1. Tree ID is used in the central map key for locating class 2 nodes.  
2. Birth generation is used in the free decision process and the central map key.  
3. Instance ID is used in the central map key.  
4. Birth serial is used in the central map key.  
5. Write sequence is used in the central map key as the instance ID.  
6. Level is not consumed by any critical system component beyond rebuild.  
7. Key range is used during tree traversal but not for identity; rebuild can recreate it.  
8. Payload CRC is used for data integrity validation.  
9. Reserved fields have no known consumers.  

For A:  
During rebuild, two distinct class 2 nodes with different original keys (e.g., tree ID 1, txg 100, instance 1, serial 1 and tree ID 1, txg 200, instance 1, serial 2) are reissued to the same birth generation (e.g., 50), instance ID (current instance 1), and birth serial (1). The central map key becomes identical for both nodes. When a reader accesses the first node's key, it retrieves the second node's data, causing a wrong answer. This does not depend on F8.  

For B:  
During rebuild, rewritingrewriting the key for a class 2 node requires changing its birth generation. However, F6 explicitly prohibits changing birth on relocation. For example, moving a node to a new location and updating its birth generation to match the rebuild's reissued fields violates F6. This does not depend on F8.  

For C:  
Deadlist entries stop using the central map key and instead store block locations. When a block is relocated (e.g., during defragmentation), the deadlist entry's stored location becomes stale. During freeing, the system frees the stale location, which now holds live data for another block, causing a wrong answer. This does not depend on F8.