1a. After rebuild_from_records, free_slots is not guaranteed to equal unit-area size minus allocated and deferred slots. For example, a single allocation of 10 slots followed by a release of the same 10 slots: allocated_slots becomes 10, free_slots remains 90 (unit-area 100 minus 10), and deferred_slots becomes 10. Unit-area size minus (allocated_slots + deferred_slots) is 80, but free_slots is 90, showing a difference of 10. Refuted by: if free_slots is computed as unit_area_slots minus allocated_slots, which it is.

1b. Yes. After publishing a file with data (allocating a slot), deleting the file (updating mapping tree to remove reference and marking allocation record as released), then performing a publish: the accounting tree's allocated_bytes will include the released slot's span (from allocated_slots), but the checker's walk of references will not count it (since the mapping tree no longer references it), violating I-3.1. Refuted by: if the walk of references includes released slots, which it should not.

1c. Yes. After creating an inode with number 3 (so inode watermark should be at least 3), a subsequent publish resets the inode watermark to FIRST_INODE_NUMBER + 1 (e.g., 2), silently discarding the correct value. No check in the facts verifies the inode watermark against actual inodes. Refuted by: if an invariant checks the inode watermark against the highest inode number.

1d. No. F14 states released records are overwritten in place when reallocated, so only one record exists per (device, slot) at any time. F7's uniqueness check cannot legitimately reject correct data. Refuted by: if the code allowed multiple records for the same slot without overwriting.

1e. A fact stating the exact relationship between allocated_slots, free_slots, and deferred_slots in the DeviceFreeMap code (e.g., "free_slots = unit_area_slots - allocated_slots" in the mark_allocated method). Refuted by: if the code explicitly defines free_slots as unit_area_slots minus allocated_slots.

2a. Yes. For a 4-device pool with region assignments [0,1,2,3], the warm-up loop publishes to regions 0,1,2, covering devices 0,1,2 but not 3. mount_writable returns success with no error, and the caller cannot detect the uncovered device. Refuted by: if the warm-up loop checks coverage and returns an error for uncovered devices.

2b. No. The new mount_writable call builds a fresh "covered" set from its own publishes; it does not inherit or check the previous process's covered set. Refuted by: if the restart procedure used the previous process's covered set.

2c. Yes. If the checkpoint reserve is 100 bytes and each warm-up publish requires 50 bytes, the third warm-up publish would exceed the reserve and trigger a NoSpaceFor error. mount_writable would propagate this error without handling it, as fact F3 shows no specific error handling for warm-up publishes. Refuted by: if the warm-up publish explicitly checks the reserve before proceeding.

2d. No. The warm-up procedure and instance switch mechanism are unrelated code paths; the facts show no shared code or interaction. Refuted by: if the warm-up code called the instance switch mechanism's code.

2e. A fact stating the device count and region-to-device mapping for the pool (e.g., "pool has 4 devices with region assignments [0,1,2,3]"). Refuted by: if the warm-up loop continues until all devices are covered.

3a. If the root is persisted but the instance-table unit is not, judge_instance_table_rows reads stale rows (missing the new row), but mount_root_instance is the new instance. Stale rows have instances below mount_root_instance, so the invariant passes incorrectly. If the instance-table unit is persisted with the new row but the root is not, the newest root is older; mount_root_instance is the older instance, and the new row's instance is higher, so the invariant fails (below_mount_root false). Refuted by: if the instance-table unit is always persisted before the root.

3b. Unbounded. The instance-table unit has a fixed maximum size (e.g., 32768 bytes), so after ~3276 rows (10 bytes each), it would overflow. On a real system, after 3277 restarts appendingappending one row each, the publish would panic with "unit exceeds capacity". Refuted by: if the instance-table unit has no size limit.

3c. Yes. Superblock instance is 5, root ring has a root with instance 10. I-3.8 checks rows below 10 (passing), but I-7.7 clause one fails (root ring instance 10 > superblock instance 5), and clause two fails (larger superblock instance 5 should not appear in root ring, but 10 does). Refuted by: if superblock instance numbers always match or exceed root ring instance numbers.

4a. No. The two methods (independently maintained vs capacity-minus-allocated) produce identical values on correct systems; no other invariant or test can distinguish them without a code mutation. Refuted by: if a test case fails only when the independent maintenance is removed.

4b. Yes. Content length 32634 (passes check), but the allocation tree has no space for new records (e.g., full), causing publish_admitted to return NoSpaceFor. Refuted by: if publish_admitted rechecks content length before allocation.

4c. No. For a warm-up root (no file), all older PublishedVersions are also no-file (e.g., row-publish from mkfs), so the rule does not trigger a violation. F1's sequence (mkfs → row-publish → warm-up → first file) ensures no older file-bearing version exists for warm-up roots. Refuted by: if a warm-up root could have an older PublishedVersion with file content.

4d. No. Both oracle calls happen unconditionally on the computed outcomes; a panic in Ignore recovery would still be passed to oracle_violation_for_versions and tallied. Refuted by: if the Ignore outcome was not checked when recovery panicked.

4e. No. F14's overwrite rule ensures only one record per (device, slot), so F24(e)'s fix cannot reject correct states. Refuted by: if the code allowed multiple records for the same slot without overwriting.