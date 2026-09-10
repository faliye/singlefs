Operations:
- txg 0: Create Origin (tree id 1).
- txg 100: Clone Origin to A (tree id 2).
- txg 120: In Origin, create unit U with birth txg 120 and on-disk header tree id 1.
- txg 130: In A, create a reflink to U (pointer has birth txg 120).

Derivation in A: Ancestry chain [Origin (1), 100]. Birth txg 120 ≥ 100 → birth tree is A (tree id 2).

True birth tree: Origin (tree id 1).

Consequence: Central map key uses tree id 2 instead of 1. The unit U is stored under key (class, 1, ...), but the system searches for (class, 2, ...). This causes incorrect reference counting. For example, when freeing, the system fails to find the reference count for tree id 2, leading to premature freeing of U while it is still referenced in A's refllink.

The ancestry walk over the ancestry chain is incompatible with the settled O(1) free decision proof because the chain length can grow arbitrarily with each clone operation. A walk over the chain takes O(k) time where k is the chain length, violating the O(1) requirement which must hold regardless of chain size.
