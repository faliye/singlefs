1. Device has free slots 0-3 and 6-7. Dirty objects keys 6,7,1,2. Candidate B sorts to 1,2,6,7 and falls back to first-fit placing them in 0,1,6,7. Candidate C (arrival order) places 6 in 0,7 in 1,1 in 2,2 in 3. Walking in key order, candidate B has two runs (1-2 in 0-1 and 6-7 in 6-7) while candidate C has one contiguous run (0-3). Step where it goes wrong is when sorting forces non-contiguous placement despite a contiguous option being available with arrival order.

2. B is A with extra bookkeeping. 93.8% fallback rate means 93.8% of allocations use identical first-fit behavior as A. Only 6.2% of allocations differ due to bump-seg placement attempts. Arithmetic: 100% - 93.8% = 6.2% of allocations involve additional sorting and segment checks.

3. Device has free slots 5 and 6. Implementation 1 scans from low to high and places new object in slot 5. Implementation 2 scans from high to low and places in slot 6. Both claim first-fit but produce different placements. Rule must specify scanning direction to ensure consistency.

4. Defragmenter starts with 0 fully empty segments. Workload continuously writes into any segment freed by defragmenter. For policy candidate A (first-fit) immediately fills freed space with new writes. Stop condition never met because empty segments are never preserved. Candidate A is closest to this construction.

5. Device free slots 0-3 and 4-7. Dirty objects keys 1,5,2,6 in arrival order. Candidate C places them in 0,1,2,3. Walking in key order: key 1 at 0, key 2 at 2, key 5 at 1, key 6 at 3 → four separate runs. Candidate A and B place keys 1,2 in 0-1 and 5,6 in 4-5 → two runs. Candidate C has higher fragmentation.

6. Candidate D risks directory hotspots causing excessive fragmentation and unreclaimable space when large directories are written or deleted.
