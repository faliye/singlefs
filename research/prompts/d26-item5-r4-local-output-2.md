A counterexample exists when a refleflink references a unit from another tree. Consider:

- txg 0: Create Origin tree.
- txg 50: In Origin, create unit X (birth txg 50, birth tree Origin).
- txg 100: Clone Origin into A (clone point 100).
- txg 200: Clone Origin into B (clone point 200).
- txg 250: In B, create unit Y (birth txg 250, birth tree B).
- txg 300: In A, create a reflink to Y.

When A reads the pointer to Y, it sees birth txg 250. The clone ancestry chain for A is [Origin:100]. The derivation calculates birth tree as A (since 250 > 100). However, the true birth tree is B. This causes the central map key to use birth tree A instead of B. When freeing Y, the system checks reference counts under A's tree ID, but Y's actual reference count is tracked under B's tree ID. This results in incorrect reference counting: B still references Y, but the system may free Y prematurely because it sees zero references in A's part of the map.

The separate question: No. A walk over the ancestry chain is not O(1) as required by the settled proof. The chain length can grow arbitrarily (e.g., many clones), making the operation dependent on chain size, not constant time. This violates the proof that free decisions require only two inputs and O(1) time.
