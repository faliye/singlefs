For Proposal 1: A sequence where the oldest root in the ring has checkpoint_txg=8, and all freed blocks have free generation f=9 or 10. The system cannot allocate new blocks because no freed block satisfies f<=8, making publication impossible due to lack of space even though free blocks exist.

For Proposal 3: A journal record named item contains a unit type tag. Layer 4 changes the meaning of this tag after the journal component is frozen at layer 2. Journal records written after layer 4 freeze use the new tag meaning, but the frozen journal component cannot interpret it, causing replay failure.

For Proposal 4: The named item position entries require tree structure knowledge to locate the correct byte offset in the root image. The record checker does not implement tree semantics, so it cannot apply the pointer-layer changes synt correctly, resulting in mismatched expected and actual post-recovery images even with a correct implementation.
