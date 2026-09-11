Q1 scan rebuild
K1: An index node in a tree that does not store key range information in its header. For example, a tree where index nodes have no key range lower bound stored in the header. When scanning units on disk, the header lacks the key range lower bound required for K1 key construction. The scan cannot rebuild the exact key because the key requires the key range lower bound which is absent in the header.
K2: No counterexample found.
K3: No counterexample found.
K4: No counterexample found.

Q2 key construction
K1: When dereferencing a data unit, the object birth generation must be read from the inode record. This requires an extra unit read or tree lookup. For example, during a file read operation, the extent tree lookup provides the pointer but not the object birth generation, which is needed to build the K1 key for the map.
K2: No counterexample found.
K3: No counterexample found.
K4: No counterexample found.

Q3 sharing
K1: No counterexample found.
K2: No counterexample found.
K3: No counterexample found.
K4: No counterexample found.

Q4 crash reuse
K1: No counterexample found. Object id uniqueness ensures different keys even after checkpoint reuse.
K2: No counterexample found. The pool-wide monotonic counter prevents reuse of the same key.
K3: A unit born in tree A checkpoint 10 sequence 5 is on disk. After a crash, checkpoint 10 is reused. A new unit is born in tree A checkpoint 10 sequence 5. Both units are on disk and share the same key. The map cannot distinguish them.
K4: A unit born at device 1 slot 100 checkpoint 5 is on disk. After a crash, the same slot 100 is reused for a new unit at device 1 checkpoint 5. Both units are on disk and share the same key. The map cannot distinguish them.

Q5 uniqueness under reuse
K1: No counterexample found. Object id uniqueness prevents same key for different units.
K2: No counterexample found. Monotonic counter ensures unique keys.
K3: A unit is freed. The same birth tree checkpoint and sequence are reused for a new unit. Both the freed unit (if not overwritten) and new unit are live with the same key. For example, free unit X in tree A checkpoint 10 sequence 5, then reuse the slot for unit Y in tree A checkpoint 10 sequence 5. Both units exist with the same key.
K4: A unit is freed. The same birth device slot and checkpoint are reused for a new unit. Both the freed unit (if not overwritten) and new unit are live with the same key. For example, free unit X at device 1 slot 100 checkpoint 5, then reuse slot 100 for unit Y at device 1 checkpoint 5. Both units exist with the same key.

Q6 a fifth candidate
K5: 16 byte universally unique identifier (UUID). The pointer carries the UUID. The unit header carries the UUID. Scan rebuild uses the header's UUID. This candidate is strictly better on Q4 and Q5 because UUID uniqueness prevents key collisions after crash reuse or reuse of slots. It is not worse on Q1 Q2 Q3: scan rebuild works (Q1), key is in pointer (Q2), sharing uses same UUID (Q3). Cost is 16 bytes per unit compared to K2's 8 bytes but avoids all key reuse issues.
