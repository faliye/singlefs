1. A grouping based on access heat (hotness) where nodes are grouped by how frequently they are accessed. This requires storing a group identity in the node header since access frequency isn't computable from keys. In a workload with concentrated hotspots, this grouping would keep frequently accessed nodes physically contiguous when rewritten, reducing runs significantly more than key-interval grouping which scatters hot nodes across groups. Model B did not test this scenario.

2. No. If the placement engine can read the parent node, the group identity for sibling nodes is derived from the parent's existing structure (e.g., parent node identifier or position), so no stored field in the node header is needed. The claim that group identity must be stored is invalidated because the placement engine can infer it from the parent.

3. None. Settling the format as no stored group identity and no layout change means the first transaction's index nodes use the same on-disk layout as before, so no bytes depend on this item.

4. Yes. Model A's node group arm was computed as key divided by G (key-interval grouping), so it is indistinguishable from key-computable grouping. The sentence should be rewritten as "a key-computable grouping policy halves the compaction policy". Model A's node group arm does nothing extra that key-interval grouping cannot.

5. Counting over nodes hides true fragmentation. The compaction watermarks (F7) use runs over keys, but counting over nodes inflates runs when nodes split (even though keys remain contiguous), making the measure inaccurate. The correct quantity must count runs over keys.

6. None. The inference does not contradict any settled fact.
