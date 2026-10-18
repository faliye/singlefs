#!/usr/bin/env bash
# K7：删掉的符号在冻结副本 crates/ 与主仓 scripts/ 里还剩几处命中（-F 字面串，排除 crates/mutations.tsv）
cd "$1" || exit 2
for n in 'JournalSequenceNumber::COUNTER_LIMIT' '.origin_of(' '.rightmost_path(' '.child_holding_data_unit(' '.upper_height()' '.lower_height()' '.previous_index_of_a_carried_container(' 'device_offset_of_the_head' 'EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES' '.data_unit_pointer()' '.data_unit_records()' 'txg_is_inside_the_one_revolution_window' 'tombstone_containers_one_root_node_holds' 'PublishShape::' 'every_unlink_step_from' 'NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS'; do
  c=$(grep -rnF --include='*.rs' --include='*.wgsl' -- "$n" crates "$2/scripts" 2>/dev/null | grep -v 'crates/singlefs-checker-tier/src/bin/e16[58]' | wc -l)
  echo "$n 命中 $c"
done
