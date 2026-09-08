1. Clause 1: Write unit T with birth=10 to (dev1, 100). Create snapshot S1 (txg=10). Start compaction: allocate (dev1, 200), write data with birth=10, free (dev1, 100), crash before updating central map. Later, write unit T2 with birth=20 to (dev1, 100). Read S1 for T: map points to (dev1, 100) containing T2's data, violating snapshot consistency.

2. Clause 2: Write unit T birth=10. Create S1 (txg=10). Write T birth=20. Create S2 (txg=20). Write T birth=30. Create S3 (txg=30). Destroy S3: previous snapshot txg=20. Deadlist entries for birth=10 and 20 are not freed (birth ≤ 20), but they are unreferenced by any snapshot, causing a permanent leak.

3. Clause 3: No counterexample.

4. Clause 4: Write 1000 units at even offsets (0, 2, ..., 1998) and 1000 at odd offsets (1, 3, ..., 1999) on dev1. Attempt to compact 4096 locations; no contiguous run exists. Compaction fails due to Clause 4's contiguous destination requirement, preventing space reclamation.
