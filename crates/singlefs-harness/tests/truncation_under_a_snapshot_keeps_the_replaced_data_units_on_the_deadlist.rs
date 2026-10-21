//! 快照下截断（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K4）：截断换下的单元——意图那次重写前的单元 k 与最后一批放掉的尾部——
//! 照 `deadlist_changes_of_this_publish` 判生死：生在建快照之前的进 deadlist、不释放；快照读出截断之前的整份内容，活头读出截后的。
//! 写法照 `snapshot_release_and_destroy_through_the_deadlist.rs`。
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use std::collections::BTreeSet;

use common::parameters;
use common_deletion_in_progress::{content_of_three_data_units, pool_with_a_three_unit_file};
use common_truncation::{
    assert_both_readers_read, change_the_size_of_the_first_file_and_advance, payload_capacity,
    walk_the_queues_and_advance, INSTANCE_OF_THE_SIZE_CHANGES,
};
use singlefs_core::address::DataUnitIndexInFile;
use singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT;
use singlefs_core::recovery::{read_the_first_file_of_a_snapshot, SnapshotFileReading};
use singlefs_core::transaction::{publish_create_snapshot, PoolWriter, TransactionUnit};
use singlefs_core::unlink::UnlinkBatchSize;

/// 三个单元的文件建快照之后截到单元 1 中间（N = 净荷 + 7）：意图那次重写单元 1、最后一批放掉单元 2。建快照那一版里单元 1、2 的映射 key
/// 都进了 deadlist（纪元 = 建快照那次的 txg），它们的落点不在这两次的释放里；快照读出截断之前的 2.5 个净荷，活头读出前 净荷 + 7 字节。
#[test]
fn a_truncation_under_a_snapshot_keeps_the_replaced_and_released_data_units_on_the_deadlist() {
    let mut pool = pool_with_a_three_unit_file("truncation-under-a-snapshot");
    let content = content_of_three_data_units();
    let created = {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let created = publish_create_snapshot(
            &mut writer,
            &mut pool.allocator,
            &previous,
            INSTANCE_OF_THE_SIZE_CHANGES,
        )
        .expect("建快照");
        pool.output = created.clone();
        created
    };
    let snapshot = created
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("一条码 14")
        .tree;
    let new_size = payload_capacity() + 7;
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, new_size);
    walk_the_queues_and_advance(&mut pool, UnlinkBatchSize::OneExtentLowerLeaf);
    let after = pool.output.clone();
    let mapping_key_of = |unit: u64| {
        created
            .mapped_units
            .iter()
            .find(|(role, _)| *role == TransactionUnit::Data(DataUnitIndexInFile(unit)))
            .expect("建快照那一版的数据单元进映射")
            .1
            .clone()
    };
    let deadlist_keys: BTreeSet<Vec<u8>> = after
        .deadlist_entries
        .iter()
        .filter(|entry| entry.epoch == created.root.checkpoint_txg)
        .map(|entry| entry.mapping_key.clone())
        .collect();
    for unit in [1, 2] {
        assert!(
            deadlist_keys.contains(&mapping_key_of(unit)),
            "单元 {unit}（生在建快照之前、这次截断换下）进 deadlist，纪元 = 建快照那次的 txg"
        );
        let slot = created
            .unit(TransactionUnit::Data(DataUnitIndexInFile(unit)))
            .slot;
        assert!(
            !intent
                .released
                .iter()
                .chain(after.released.iter())
                .any(|placement| placement.slot == slot),
            "单元 {unit} 被快照引用，不进释放"
        );
    }
    let image = pool.memory_pool();
    assert_eq!(
        read_the_first_file_of_a_snapshot(&image, &after.root, snapshot).expect("读快照"),
        SnapshotFileReading::File(content.clone()),
        "快照读出截断之前的整份内容"
    );
    assert_both_readers_read(
        &image,
        &after.root,
        &content[..usize::try_from(new_size).expect("长度")],
        "活头读出截后",
    );
}
