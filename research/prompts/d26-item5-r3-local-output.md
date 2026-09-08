Clause 1 breaks. Timeline: At txg 100, a unit with tuple T and write sequence W1 is written with two mirror copies. At txg 200, compaction relocates one mirror to a new location with write sequence W2. Both copies are readable. A scanner rebuilding state from headers sees two units with same tuple T but different write sequences W1 and W2. Each write sequence group has only one member, violating the invariant requiring two members per group. Scanner falsely reports broken mirror groups.

Clause 1 breaks. Timeline: Unit born at txg 50 with birth 50. Scrub watermark set to 100 at txg 100. Compaction relocates unit at txg 200 with birth unchanged at 50. Next scrub run at txg 300 uses watermark 100. Unit birth 50 < 100, so scrub skips it. Physical copy never checked for corruption, leading to undetected data errors.

Clause 2 breaks. Timeline: Unit U written at txg 100. Compaction relocates U at txg 200, freeing the old copy. Clause 2 states compaction touches deadlist zero times, so freed unit is not added to deadlist. Snapshot S is destroyed at txg 300. Deadlist for S lacks U, so U is never freed. Memory leak occurs as U remains allocated.

Clause 2a breaks. For 1000 unit relocations (4KB each), transaction size is 4MB. Journal max size is 1MB. Transaction cannot commit. Compaction fails entirely, leaving data uncompactable and disk space wasted.

Clause 3 is consistent under second encoding. Deadlist entries (41 bytes) can be resolved by looking up tuple in map and scanning version chain for matching birth. Background work exemption allows slow chain traversal. No counterexample.
