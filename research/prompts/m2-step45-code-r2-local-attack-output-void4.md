1a
In the chain of publish_version calls after mount completion, current is updated to the output of each publish, which includes the instance-table unit written to disk. Therefore, current's instance-table content always matches the on-disk newest root's instance-table content. No reachable point exists where they differ. This is refuted by: a publish_version call that writes a new root to disk but does not update current to reflect the new root's instance-table unit, which contradicts the fact that current is reassigned to the publish's output after each success (G4 line 398).

1b
A root with checkpoint_txg below effective_floor cannot change the reclaim_floor output because G1 returns the larger of the two values. For example, with effective_floor=100 and oldest_valid_root=50, reclaim_floor returns 100. If oldest_valid_root were 120, it would return 120, but a value below effective_floor is always overridden by the max operation. This is refuted by: a scenario where reclaim_floor returns a value less than effective_floor when oldest_valid_root is below effective_floor, which violates G1's rule of returning the numerically larger value.

1c
The_rollback_floor includes abandoned roots, so it can be inflated by an abandoned timeline's higher rollback_floor values. However, reclaim_floor uses the max of effective_floor and oldest_valid_root (which is from non-abandoned roots). This is by design per D1 and does not cause unsafe reclaiming because placements released by the abandoned timeline are no longer in use. No mechanism is needed to prevent abandoned roots from exceeding the live timeline's rollback_floor, as it is intentional. This is refuted by: a placement still in use by the new timeline having a release generation <= the inflated effective_floor, but such a placement would not be released (is_released=false) and thus not reclaimedclaimed.

1d
An all-abandoned readable-root set is impossible because the newest root cannot be abandoned by its own table. G2 requires a row in the instance table where selected_root_txg < root's checkpoint_txg for abandonment, but the newest root's instance table row for its own instance has selected_root_txg equal to its checkpoint_txg (not strictly smaller). This is refuted by: the newest root having a row in its own instance table with selected_root_txg strictly less than its checkpoint_txg, which contradicts the instance table update logic for the newest root.

2a
No reachable sequence produces a third record with generation zero. G10 states only the two mkfs records have generation zero, and no operation changes a record's generation back to zero. A rollback to checkpoint_txg zero would not create new generation-zero records; it would reuse existing records with nonzero generations. This is refuted by: a record created with generation zero after mkfs, which violates G10's assertion that only the two mkfs records have generation zero.

2b
Rebuild_from_records would set format_time_tree_table to the placement from device one's unreleased copy. format_time_tree_table_to_release would check device zero's record and find is_released=true, so it returns an empty list and does not attempt a second release on device zero. This is refuted by: format_time_tree_table_to_release attempting to release the tree table on device zero when its record is already released, but the check in G9 line 586 prevents it by requiring is_released=false.

2c
No point exists after the tree table has been durably released on both devices where publish_version is called with previous equal to absent and the tree-table role rewritten. The only time previous is absent is during the initial publish (after mkfs), before any release. The rewritten-roles check is redundant because the function is only called when the tree-table role is rewritten, so the check is always true. This is refuted by: a publish_version call with previous absent and tree-table role rewritten after the tree table is durably released, which never occurs.

3a
It is impossible for a chosen root to belong to instance i and carry a row describing the ending of instance i in its own table. The facts state that such a row is always written into the first new root of the next instance's table, not instance i's table. This is refuted by: a root for instance i having a row in its own instance table for the ending of instance i, which contradicts the documented replay rule.

3b
Yes, an extra check rejecting when newest root's instance number is more than one step ahead of the target's instance number could wrongly reject a valid target. For example, if the newest root is for instance 2 and the target is for instance 0 (with a high-water mark <= target's checkpoint_txg), today's check accepts, but the extra check rejects because 2 > 0+1. This is refuted by: a rollback target for instance 0 when the newest root is for instance 2 and the high-water mark for instance 0 equals the target's checkpoint_txg, so today's check accepts but the extra check rejects.

3c
No additional field distinct from transaction can be identified from the facts given. The facts do not describe any other fields in a journal record that could be checked. This is refuted by: a field in the journal record described in G1-G10 that could be used in the combined test, but no such field is provided in the facts.