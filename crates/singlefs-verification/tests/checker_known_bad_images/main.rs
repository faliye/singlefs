//! checker 的已知坏镜像语料（C13（checker 判定失效））：第一版判的 36 条不变量，每条配一份「改坏了正好触发它」的镜像。
//! 每份都从一份干净镜像出发（写完新池新建文件的，或再覆盖写一次的），改一处；要让被改的那一条之前的判定不先挡住，
//! 被改的单元重封校验和，再把新的整单元校验和沿引用链一路补到根记录（父节点里的位置条目、中央映射条目、根槽的自证校验和）。
//! 写完新池新建文件那一份上的那张表（`known_bad_images`）与它用到的常量、辅助函数在库里（`singlefs_verification::known_bad_images`），E171 装置用的是同一份。

#[path = "../common/mod.rs"]
mod common;
#[path = "../common_deletion_in_progress/mod.rs"]
mod common_deletion_in_progress;
#[path = "../common_images_for_the_given_states_engine/mod.rs"]
mod common_images_for_the_given_states_engine;
#[path = "../common_pointer_head_corruption/mod.rs"]
mod common_pointer_head_corruption;
#[path = "../common_root_ring_holes/mod.rs"]
mod common_root_ring_holes;
#[path = "../common_shared_by_harness_scenarios/mod.rs"]
mod common_shared_by_harness_scenarios;
#[path = "../common_subtree_coverage_key_range/mod.rs"]
mod common_subtree_coverage_key_range;
// 借隔离两份副本的那一段历史（两块盘都带隔离记号、没有根引用的那个数据单元）造 C962 的坏镜像；两份场景文件交镜像的那一半这里不用。
#[path = "../common_images_for_the_given_states_engine/quarantine_mark_agrees_with_the_references.rs"]
mod scenarios_of_quarantine_mark_agrees_with_the_references;
#[path = "../common_images_for_the_given_states_engine/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;
// 借它那几个只供测试的开关的搭法（压回水位建 inode、覆盖写第一个文件）造 C479 的坏镜像；那份场景文件交镜像的那一半这里不用。
#[path = "../common_images_for_the_given_states_engine/inode_number_watermark_pressed_back_reissues_a_number.rs"]
mod scenarios_of_inode_number_watermark_pressed_back_reissues_a_number;

mod allocation_records_and_space_statistics;
mod appearance_rule_and_copies_across_device_orders;
mod clean_images_and_rewrites;
mod container_capacity_knobs;
mod container_copies_and_location_checksums;
mod container_tree_entries_over_unreadable_children;
mod container_tree_entry_rewrites;
mod container_tree_entry_types;
mod each_invariant_reddened_by_its_own_bad_image;
mod instance_table_pages_and_rows;
mod journal_transaction_and_publish_boundaries;
mod node_identity_and_tree_table_shape;
mod partially_judged_reads_and_pointer_heads;
mod pointer_journal_and_generation_fields;
mod retirement_and_lineage_records;
mod rollback_floor_and_birth_across_readable_roots;
mod row_high_water_and_reissued_numbers;
mod system_configuration_slots_ring_geometry_and_replay;
mod versions_applied_only_by_their_journal_records;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    copy_each_root_into_the_new_root_segment_of_its_journal_records, crash_state_devices,
    format_pool, geometry, memory_pool_of_sparse_devices,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    publish_overwrite_in_process, rewrite_journal_record_and_rechain_the_records_after_it,
    tear_the_root_slot_written_for_txg, BuiltPool, CONTENT_BYTES_OF_THE_NEWER_OVERWRITE,
    CONTENT_BYTES_OF_THE_OLDER_OVERWRITE, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use common_deletion_in_progress::{
    get_u32, get_u64, set_u16, set_u32, set_u64, Mutation,
    CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET, INODE_RECORD_BLOCKS_OFFSET,
    PACKED_UNIT_RECORDS_START, PACKED_UNIT_RECORD_COUNT_OFFSET, TREE_KIND_NOT_REGISTERED,
    TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET, TREE_TABLE_ENTRY_KIND_OFFSET,
};
use singlefs_checker::image::{
    InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
    ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
    ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION,
    COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
    DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
    DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE,
    DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
    FIRST_TXG_OF_EACH_INSTANCE_IS_ABOVE_EVERY_ROOT_AND_RECORD_OF_THE_OLDER_INSTANCES,
    FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM,
    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
    INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
    INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
    INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
    INSTANCE_TABLE_ROW_HIGH_WATER_IS_ONE_A_RECOVERY_COULD_HAVE_APPLIED,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE,
    JOURNAL_RING_GEOMETRY_FIELDS_AGREE, LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
    LIVE_SNAPSHOTS_ARE_ENUMERABLE, NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    NO_COMPACTION_INTENT_WITH_A_WELL_FORMED_REGION_IN_ANY_WALKED_VERSION,
    NO_INSTANCE_TABLE_ROW_UNDER_THE_NEWEST_ROOT,
    NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION,
    NO_MAPPING_KEY_PLACED_DIFFERENTLY_IN_TWO_WALKED_VERSIONS,
    NO_RECORD_CARRYING_THE_QUARANTINE_MARK_UNDER_THE_NEWEST_ROOT,
    NO_VERSION_APPLIED_ONLY_BY_RECORDS_REWRITING_ITS_ALLOCATION_RECORD_TREE,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES,
    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE, REFERENCE_INTERVALS_ARE_EXACT,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
    ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
    SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
    SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
    SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
    SYSTEM_CONFIGURATION_RECORDS_THE_WRITER_IDENTITY,
    SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
    TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
    UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
    UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
    WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
};
use singlefs_checker::walk::{check_pool_image, InstanceTableOfRootRecord};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish,
};
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
    INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, InstanceRow,
    MountError, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::records::{
    RetirementRecord, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
    TREE_KIND_DELETED_INODES, TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
    TREE_KIND_TOMBSTONE,
};
use singlefs_core::recovery::{
    instance_table_chain_of_root, recover, JournalPolicy, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_instance_table_on_version_without_file,
    publish_overwrite, publish_sequential_write, publish_without_units, warm_up, FirstFile,
    InstanceTableOnlyPublishPlan, InstanceTableRewrite, PoolVersion, PoolWriter, TransactionOutput,
    TransactionUnit, ZeroUnitPublishPlan,
};
use singlefs_format::{
    ALLOCATION_RECORD_TREE_LEAF_SLOTS, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
    JOURNAL_SAFETY_FACTOR, NODE_BYTES, TREE_IDENTIFIER_EXTENT,
};
use singlefs_verification::history::{
    allocation_statistic_mechanism, AllocationStatisticMechanism,
};
use singlefs_verification::known_bad_images::{
    adjust_accounting, clear_the_writer_implementation_identifier_of_the_chosen_slot_of_device_one,
    known_bad_images, location_pattern, mutate_allocation_leaves, mutate_system_configuration_slot,
    mutate_unit, mutate_unit_of, narrow_the_entry_width_to_the_key_width,
    narrow_the_record_of_the_data_unit_to_one_slot, read_bytes_of_the_image as read, replace_all,
    reseal_the_unit_header_and_payload, rewrite_the_data_unit_mapping_key_birth_txg,
    rewrite_unreleased_allocation_generation, root_slot_offsets_of_both_devices, set_accounting,
    set_allocation_record_span, swap_the_kinds_of_the_livelist_and_sparse_side_table_entries,
    swap_the_own_device_numbers_of_the_two_devices, system_configuration_slot_one_offset,
    system_configuration_slots_chosen_first, tree_table_entry_offset,
    write_bytes_into_the_image as write, ACCOUNTING_ROOT, ALLOCATION_LEAVES,
    ALLOCATION_NODE_ENTRY_COUNT_OFFSET, ALLOCATION_NODE_ENTRY_START, ALLOCATION_RECORD_BYTES,
    ALLOCATION_RECORD_RELEASED_FLAG, ALLOCATION_RECORD_TREE_ROOT, DATA_UNIT,
    DATA_UNIT_BIRTH_TXG_OFFSET, DEVICES, DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
    EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET, EXTENT_ROOT, GENESIS_TREE_TABLE, INODE_LEAF,
    INODE_ROOT, JOURNAL_RECORD_BACK_CHAIN_OFFSET, LIVELIST_TREE_TABLE_ENTRY_INDEX, MAPPING_ROOT,
    OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT, SLOT,
    SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET, TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
    TREE_TABLE, UNITS,
};
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RetainedOperation, SharedStream};
use std::sync::Arc;

use allocation_records_and_space_statistics::*;
use appearance_rule_and_copies_across_device_orders::*;
use clean_images_and_rewrites::*;
use container_copies_and_location_checksums::*;
use container_tree_entries_over_unreadable_children::*;
use container_tree_entry_rewrites::*;
use container_tree_entry_types::*;
use each_invariant_reddened_by_its_own_bad_image::*;
use instance_table_pages_and_rows::*;
use journal_transaction_and_publish_boundaries::*;
use node_identity_and_tree_table_shape::*;
use partially_judged_reads_and_pointer_heads::*;
use pointer_journal_and_generation_fields::*;
use retirement_and_lineage_records::*;
use rollback_floor_and_birth_across_readable_roots::*;
use row_high_water_and_reissued_numbers::*;
use system_configuration_slots_ring_geometry_and_replay::*;
use versions_applied_only_by_their_journal_records::*;
