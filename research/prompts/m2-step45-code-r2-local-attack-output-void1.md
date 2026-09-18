1a
No reachable point exists where current's instance-table content differs from a fresh choose_root call on disk. This is because each publish_version call writes the new root to disk before updating current to the output, ensuring current always reflects the latest on-disk state. This is refuted by: a publish_version call that updates current to a new root without first writing that root to disk, causing current's instance-table to mismatch the disk.

1b
A sub-floor non-abandoned root (checkpoint_txg < effective_floor) never changes the reclaim_floor output because reclaim_floor (G1) takes the maximum of the two values, so any value below effective_floor is ignored. This is refuted by: reclaim_floor(effective_floor=50, oldest_valid_root=30) returning 30 instead of 50.

1c
Yes, effective_rollback_floor can report a higher abandoned-timeline rollback_floor value, pushing reclaim_floor above the new timeline's justification and allowing reclaim_released_up_to to reclaim placements with release generation <= that inflated floor. However, this aligns with D1's definition of effective rollback floor, which includes all readable roots without abandonment filtering. This is refuted by: D1 explicitly requiring effective rollback floor to exclude abandoned roots, but D1's actual definition states "any readable root" without qualification.

1d
No, it is impossible for readable_roots to return all abandoned roots because the current mount's newest root cannot be abandoned by its own instance table (abandoned_by_table returns false for it, as the row's selected_root_txg equals its checkpoint_txg). This is refuted by: the newest root having a row in its instance table where selected_root_txg < checkpoint_txg.

2a
No reachable sequence exists where G7 picks a non-tree-table record for generation zero. Only mkfs creates records with generation zero (G10), and the instance table has span two while the tree table has span one, so G7's predicate correctly identifies the tree table. This is refuted by: a record created with generation zero outside of mkfs (e.g., via reuse or release), but G8 and G10 prevent this.

2b
Yes, rebuild_from_records would set format_time_tree_table to device one's unreleased copy. However, format_time_tree_table_to_release (G9) checks device zero's record for is_released before proceeding, and if it is released, returns an empty list, preventing a second release attempt. This is refuted by: format_time_tree_table_to_release not checking device zero's is_released status, but G9 explicitly does this check.

2c
No reachable state exists where publish_version is called with previous absent and tree-table role rewritten after the tree table has been durably released. The previous-equals-absent branch only occurs during mkfs, and after mkfs, previous is never absent. The rewritten-roles check is necessary to avoid unnecessary processing but not redundant, as it correctly filters non-tree-table publishes. This is refuted by: a publish_version call with previous absent and tree-table role rewritten after mkfs, but no such call path exists.

3a
No, it is impossible for a chosen root to simultaneously belong to the instance described by a row and carry that row in its table, because the row describing instance i's ending is always written into the next instance's table (not i's own table). This is refuted by: a root in instance i containing a row describing i's ending in its own table.

3b
Yes, adding a check that rejects if the newest root's instance is more than one step ahead of the target's instance could wrongly reject valid rollbacks. For example, if the newest root is instance 3 and the target is instance 1, the current check would allow it (if the high-water mark is valid), but the extra check would reject it despite being valid. This is refuted by: a valid rollback to instance 1 when the newest root is instance 3 being rejected by the extra check.

3c
No additional field distinct from the transaction field exists in the facts given that could be checked to make the non-empty root test stricter without adding false positives. This is refuted by: a field in journal records (e.g., "transaction_type" or "data_size") being described in the facts and usable for stricter checks, but no such field is mentioned.