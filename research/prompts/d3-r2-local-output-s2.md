1. No, key-order batching is not implementable without buffering the whole batch and sorting it; the key-order arm is an artifact because real systems process transactions in arrival order without prior sorting.

2. The lowest free slot is the first free slot when scanning devices in ascending order and for each device scanning columns in ascending order. This sentence does not change the runs result relative to arrival-order bump because it defines the existing lowest-free-slot arm used in the data.

3. The bump segment allocation rate (6-8% vs 0% for lowest free slot only), sourced from the fallback rate data showing the percentage of allocations using the bump segment versus falling back.

4. Yes, placement before packing affects runs; for example, F4 shows container arm runs are lower than bump arm runs in runs-of-8 workload due to initial placement impacting packing consolidation. Empty-segment count remains 0 regardless, so no effect there.

5. Yes, F5's stop predicate can never fire because the empty-segment count is always 0, meaning compaction cannot create a new empty segment to stop. The clause under review does not specify how empty segments are created, which is necessary for compaction to complete.

6. None.
