Q1 for candidate A: Two different code 2 index nodes with identical tree id, level, and lower key bound but different birth checkpoint numbers. For example, tree id 1, level 2, lower bound 100, birth checkpoint 5 for unit A and tree id 1, level 2, lower bound 100, birth checkpoint 6 for unit B. Both have the same key because the key for code 2 in candidate A is tree id, level, lower bound, birth checkpoint. However, this scenario is impossible as the tree structure requires unique lower bounds per level, so no duplicate keys exist for valid tree operations.

Q2 for candidate A: When scanning headers for code 2 index nodes, the header contains birth tree id and birth checkpoint but not level or lower bound. The key for code 2 requires tree id, level, lower bound, and birth checkpoint. Since level and lower bound are not stored in the header, the scan cannot reconstruct the exact key. For example, a code 2 header has birth tree id 3, birth checkpoint 7, but no level or lower bound data. The scan cannot determine these missing fields, so the key cannot be fully recovered.

Q3 for candidate A: When traversing the tree for a code 2 index node, the lookup path provides tree id, level, and lower bound but not birth checkpoint. To enter the map, the system needs the full key which includes birth checkpoint. Since birth checkpoint is not part of the lookup path, an extra unit read is required to fetch the header and obtain the birth checkpoint value. For example, during a tree traversal for tree id 4, level 3, lower bound 50, the system must read the header to get the birth checkpoint before entering the map.

Q1 for candidate B: Two code 1 units written in the same instance, checkpoint, and tree with identical birth sequence numbers. For example, instance id 2, checkpoint 10, tree id 5, birth sequence 3 for unit A and instance id 2, checkpoint 10, tree id 5, birth sequence 3 for unit B. However, birth sequence counts from 0 per (tree, checkpoint, instance), so each write increments the sequence, making duplicates impossible. No counterexample exists.

Q2 for candidate B: Scanning headers for any unit class retrieves all key components (birth tree, checkpoint, instance id, birth sequence) since these are stored in the header. For example, a code 3 container header has all required fields. The scan can exactly reconstruct the key without missing data.

Q3 for candidate B: When accessing a code 2 index node via tree traversal, the path provides tree id, level, and lower bound but not checkpoint, instance id, or birth sequence. The key requires these values, so an extra unit read is needed to fetch the header for checkpoint, instance id, and sequence. For example, a lookup for tree id 7, level 2, lower bound 30 requires reading the header to get checkpoint 12, instance id 3, and sequence 4.

Q1 for candidate C: Two code 1 units with identical birth tree, checkpoint, and write order. For example, birth tree 4, checkpoint 8, write order instance id 2 transaction 5 for unit A and birth tree 4, checkpoint 8, write order instance id 2 transaction 5 for unit B. However, each transaction writes exactly one code 1 unit, and transaction numbers are unique per instance, so write order is unique. No counterexample exists.

Q2 for candidate C: Scanning headers for code 2 units retrieves birth tree, checkpoint, instance id, and birth sequence (stored in header), so the exact key can be reconstructed. For example, a code 2 header has all required fields for its key. No missing data.

Q3 for candidate C: When accessing a code 2 index node, the lookup path provides tree id, level, and lower bound but not checkpoint, instance id, or birth sequence. The key requires these values, so an extra unit read is needed to fetch the header for checkpoint, instance id, and sequence. For example, a lookup for tree id 9, level 1, lower bound 25 requires reading the header to get checkpoint 15, instance id 1, and sequence 2.

Q4 for candidate C: Two different code 1 units with identical birth tree, checkpoint, and write order. For example, birth tree 5, checkpoint 10, write order instance id 3 transaction 7 for unit A and birth tree 5, checkpoint 10, write order instance id 3 transaction 7 for unit B. However, each transaction writes one code 1 unit, so write order is unique per unit. No counterexample exists.

Q1 for candidate D: Two live units with identical birth device, slot, and checkpoint. For example, birth device 1, slot 100, checkpoint 5 for unit A and birth device 1, slot 100, checkpoint 5 for unit B. However, slots are not reused within the same checkpoint. After checkpoint publication, slot reuse occurs in new checkpoints, so checkpoint numbers differ. No counterexample exists.

Q2 for candidate D: Scanning headers retrieves birth device, slot, and checkpoint for all unit classes. For example, a code 1 header contains these fields. The scan can exactly reconstruct the key.

Q3 for candidate D: When accessing a code 2 index node via tree traversal, the path provides tree id, level, and lower bound but not device, slot, or checkpoint. The key requires these values, so an extra unit read is needed to fetch the header for device, slot, and checkpoint. For example, a lookup for tree id 6, level 3, lower bound 40 requires reading the header to get device 2, slot 50, checkpoint 12.

Q1 for candidate E: Two code 1 units in the same instance with identical counter values. For example, instance id 4, counter 15 for unit A and instance id 4, counter 15 for unit B. However, the counter starts at 1 per instance and increments for each write, so duplicates are impossible. No counterexample exists.

Q2 for candidate E: Scanning headers retrieves instance id and counter for all unit classes. For example, a code 3 header has these fields. The scan can exactly reconstruct the key.

Q3 for candidate E: When accessing a code 2 index node via tree traversal, the path provides tree id, level, and lower bound but not instance id or counter. The key requires these values, so an extra unit read is needed to fetch the header for instance id and counter. For example, a lookup for tree id 8, level 2, lower bound 35 requires reading the header to get instance id 5 and counter 10.

Q1 for candidate F: Two different units with identical random 16 byte identifiers. For example, both units have random value 0x00000000000000000000000000000000. Random identifiers can collide due to finite entropy, so this is possible. Operation sequence: two independent writes in the same instance where the random number generator produces the same value for both.

Q2 for candidate F: Scanning headers retrieves the random identifier for all unit classes. For example, a code 2 header contains the random value. The scan can exactly reconstruct the key.

Q3 for candidate F: When accessing a code 2 index node via tree traversal, the path provides tree id, level, and lower bound but not the random identifier. The key requires the random identifier, so an extra unit read is needed to fetch the header for the random value. For example, a lookup for tree id 10, level 1, lower bound 50 requires reading the header to get the random identifier.

Q1 for candidate G: Two live units with identical birth device, slot, and version after version wraparound. For example, birth device 3, slot 200, version 0 for unit A and birth device 3, slot 200, version 0 for unit B. This occurs if the version table wraps from 65535 to 0 and the previous unit with version 0 was not properly deleted. Operation sequence: slot 200 is written 65536 times (version 0 to 65535), then the next write sets version 0. If the first version 0 unit was not deleted due to a crash during deletion, both units remain live.

Q2 for candidate G: Scanning headers retrieves birth device, slot, and version (stored in header). For example, a code 1 header contains these fields. The scan can exactly reconstruct the key.

Q3 for candidate G: When accessing a code 2 index node via tree traversal, the path provides tree id, level, and lower bound but not device, slot, or version. The key requires these values, so an extra unit read is needed to fetch the header for device, slot, and version. For example, a lookup for tree id 11, level 3, lower bound 60 requires reading the header to get device 4, slot 250, version 1.

Q4 for candidate C: Not applicable as no counterexample exists.

Q5 for candidate G: The per-slot version table must live on disk in a persistent storage area. Crash scenario: slot 150 has version 1 written twice before a crash. The version table is not flushed to disk, so after remount, it reads version 0. A new write to slot 150 increments to 1. The previous unit with version 1 is still live, causing both units to share the key birth device 2, slot 150, version 1.

Q6: Eighth Eighth key family: birth device 4 bytes, birth slot 6 bytes, and birth version 2 bytes stored directly in the unit header. This avoids a persistent version table, eliminating crash-related version table issues present in G. For example, when writing a unit, the version is incremented and stored in the header. During scan, the version is read from the header. This is strictly better than G on Q5 as it removes the need for a separate version table, preventing the crash scenario where the version table is outdated.