1a. After rebuild_from_records, free_slots for a device equals unit-area size minus allocated_slots, which includes both allocated and deferred-released slots. For example, with unit_area_slots=10, a released record of span=2: mark_allocated sets allocated_slots=2 and free_slots=8, then mark_released sets deferred_slots=2. free_slots=8 equals 10-2. No order or content causes discrepancy. Refuted by: a record with span=0 or negative, but facts state span is positive.

1b. Yes. Sequence: create file (allocates data unit), delete file (releases placement), restart. After restart, rebuild_from_records adds released span to allocated_slots. The next publish computes accounting tree's allocated bytes from this. The checker's walk of references excludes released units (not referenced by any root), so the walk sum is less than allocated_slots. This violates I-3.1. Refuted by: released units still referenced by an older root (e.g., a snapshot).

1c. Yes. Sequence: mkfs (FIRST_INODE_NUMBER=1), create two files (inodes 1 and 2), restart. Row-publish sets inode watermark to 2 (FIRST_INODE_NUMBER+1), but actual next inode should be 3. The accounting tree's inode watermark is silently wrong, and no check catches this. Refuted by: FIRST_INODE_NUMBER dynamically tracking allocated inodes.

1d. No. F14 states released records are overwritten when reallocated, so only one record per (device, slot) exists. F7's slot distinctness check is always valid. Refuted by: code writing a new record without overwriting the old one.

1e. A fact stating "the item 2 free statistic is derived from device_map.free_slots, which is unit_area_slots minus allocated_slots, and the checker's walk of references excludes released units because they are not referenced by any root". Refuted by: released units still referenced by an older root.

2a. Yes. With 4 devices and region_devices=[0,0,0,0], all three warm-up publishes map to device 0. Covered={0}, but devices 1-3 uncovered. mount_writable returns success with no error, violating F22's requirement. Refuted by: region_devices for 4 devices having distinct entries for all regions.

2b. No. The new mount_writable reads the disk and selects the newest persisted root (older than the crashed warm-up publish). It builds a new "covered" set from scratch, ignoring the previous attempt. Refuted by: journal replay including the crashed warm-up publish's record.

2c. Yes. If the checkpoint reserve is recomputed as sum of tree heights plus accounting-tree nodes touched, and prior warm-up publishes exhaust the reserve (e.g., tree heights high and accounting-tree nodes touched large), the next warm-up publish fails with NoSpaceFor. mount_writable returns error, but F3 has no handling, causing mount failure. Refuted by: checkpoint reserve always sufficient for warm-up publish cost.

2d. No. F23 describes instance-switch as not-yet-implemented, with no code overlap in facts. Warm-up is part of mount_writable restart; instance-switch is separate. Refuted by: warm-up code calling instance-switch mechanism.

2e. A fact stating "region_devices for a 4-device pool has all three regions mapping to device 0". Refuted by: region_devices for 4 devices having each region mapped to a distinct device.

3a. If root is persisted but instance-table unit is not: root's instance=6, but instance-table unit has stale rows (e.g., instance=5). judge_instance_table_rows checks rows <6 (passes), but actual state is inconsistent (unit not persisted). If instance-table unit is persisted (row instance=6) but root is not: oldest root instance=5, judge_instance_table_rows sees row=6 >=5 (fails). Both cases are correct outcomes for the state. Refuted by: instance-table unit bytes matching the root's version.

3b. Bounded. Instance-table unit has fixed maximum size (like data unit payload capacity 32634). Each row is ~5 bytes, so ~6500 rows. After 6500 restarts, write fails with panic due to overflow. Refuted by: instance-table unit having dynamic size or larger capacity.

3c. Yes. Superblocks have instance=5 on all devices. Root ring has root with instance=6. Instance-table rows have instances 1-5. I-3.8 checks rows <6 (passes). I-7.7 clause one fails (6>5). Contradiction. Refuted by: superblock instance numbers include 6.

4a. No. F15 states independently maintained and capacity-minus-allocated always produce identical values on correct systems. No sequence of operations distinguishes them. Refuted by: a test case where the computed value differs.

4b. No. Content-length check passes for <=32634 bytes. publish_admitted uses content for accounting entries, but no panic occurs if content fits. Refuted by: publish_admitted having a line that panics for valid content.

4c. No. Warm-up publishes occur before first file publish, so versions list has no file-bearing entries during warm-up. If effective root is warm-up root (no file), versions list has no older file-bearing entries. Rule only triggers if older file-bearing exists, but this never happens. Refuted by: a file-bearing PublishedVersion older than warm-up root.

4d. No. F13 unconditionally calls oracle_violation_for_versions on both "consulted" and "ignored" outcomes. Even if Ignore recovery panics, the outcome is still passed to oracle_violation_for_versions. Both are always tallied. Refuted by: oracle_violation_for_versions not called for Ignore if it panics.

4e. No. F14 states released records are overwritten when reallocated, so structurally only one record per (device, slot). F7's slot distinctness check is always valid. Refuted by: code writing a new record without overwriting the old one.
