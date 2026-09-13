1. Key-order batching is not implementable in the real system without first buffering the whole batch and sorting it, so the key-order arm is an artifact.

2. The lowest free slot is the slot with the smallest device number and, for slots with that device number, the smallest column number. This sentence does not change the runs result relative to arrival-order bump because it defines the fallback mechanism already used in arrival-order bump.

3. The measurable quantity is the proportion of allocations served from the bump segment without fallback. Its source is the inverse of the fallback rate; for arrival-order bump with fallback, it is 6 to 8 percent, while for lowest free slot only, it is 0 percent.

4. No, the placement of a small object before it is packed does not affect the runs or the empty-segment count after packing because the packing process reorganizes objects by key order. Thus, the clause need not treat small objects separately.

5. Yes, since the empty-segment count is always 0, F5's stop predicate can never fire because it requires the count to grow by at least one, which never happens. The clause under review states that compaction is driven by F5's watermarks, and F5 implies that empty segments come from compaction emptying segments, but in this model, compaction did not succeed in creating empty segments.

6. None.
