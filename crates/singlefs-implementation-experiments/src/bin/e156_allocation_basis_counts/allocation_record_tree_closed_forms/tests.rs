//! R-3（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后，一次发布新写与替换的节点数的闭式的单测。

use super::new_pool_file_creation_state;
use super::{
    e156_allocation_record_tree_new_node_count, e156_closed_form_expected,
    e156_existing_leaf_positions, e156_touched_leaf_positions, parameters,
    self_release_slots_of_this_publish, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, PoolAllocator};
use singlefs_core::transaction::{FirstFile, PoolWriter};
use std::collections::BTreeSet;

/// PC-闭式自测第一、二、四组（E156 第 3 次重跑登记「七」7.2 R-3c）：只在合成的叶位置集合上单测
/// 闭式函数本身，不依赖真实分配器。M27（闭式漏掉根）、M28（闭式每个改动的叶位置只算一块盘）应当
/// 分别让第一组从 5 变成 4 与 3。第三组的前提（61、170 号叶分落第一、第二个层级 1 节点）先用
/// `e156_level1_position_of_leaf` 现算钉住，不写成两边都是字面量的比较。
#[test]
fn pc_closed_form_matches_the_registered_anchor_r3c() {
    let one_leaf: BTreeSet<u64> = [61].into_iter().collect();
    let two_leaves: BTreeSet<u64> = [61, 62].into_iter().collect();
    let leaves_in_two_internal: BTreeSet<u64> = [61, 170].into_iter().collect();
    assert_eq!(
        super::e156_level1_position_of_leaf(61),
        0,
        "R-3c：61 号叶应落在第一个层级 1 节点里"
    );
    assert_eq!(
        super::e156_level1_position_of_leaf(170),
        1,
        "R-3c：170 号叶应落在第二个层级 1 节点里"
    );
    assert_eq!(
        e156_allocation_record_tree_new_node_count(&one_leaf, 2),
        5,
        "R-3c：只在第 61 片叶（两盘）"
    );
    assert_eq!(
        e156_allocation_record_tree_new_node_count(&two_leaves, 2),
        7,
        "R-3c：第 61、62 片叶"
    );
    assert_eq!(
        e156_allocation_record_tree_new_node_count(&leaves_in_two_internal, 2),
        9,
        "R-3c：第 61 与第 170 片叶（第 170 片在第二个层级 1 节点里）"
    );
    assert_eq!(
        e156_allocation_record_tree_new_node_count(&one_leaf, 1),
        3,
        "R-3c：一盘、只在第 61 片叶"
    );
}

/// PC-闭式第三组（R-3c，M29 的取样点）：合成一份记录——一条在第 61 片叶已释放、一条在第 170 片叶
/// 仍分配，两条 `generation` 都是这次的 txg。`e156_touched_leaf_positions` 应当把两条都算进
/// 「这次改动」，给出 9；M29（闭式的「改动的记录」只取这次分配的、不取这次释放的）应当让这一格
/// 只剩第 170 片叶一条，变成 5。
#[test]
fn pc_closed_form_third_group_counts_released_and_allocated_records() {
    let devices = vec![DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES)];
    let txg = CheckpointTxg(9);
    let released_slot = 50245u64;
    let allocated_slot = 170u64 * 812;
    let records = vec![
        AllocationRecord {
            device: DeviceIdentity(0),
            slot: SlotNumber(released_slot),
            span_slots: 1,
            generation: txg,
            is_released: true,
            quarantine_mark:
                singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        },
        AllocationRecord {
            device: DeviceIdentity(0),
            slot: SlotNumber(allocated_slot),
            span_slots: 1,
            generation: txg,
            is_released: false,
            quarantine_mark:
                singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
        },
    ];
    let allocator = PoolAllocator::rebuild_from_records(devices, records);
    let touched = e156_touched_leaf_positions(&allocator, DeviceIdentity(0), txg);
    assert_eq!(
        touched,
        BTreeSet::from([61, 170]),
        "PC-闭式第三组：应当同时看到释放在第 61 片、分配在第 170 片两条"
    );
    assert_eq!(
        e156_allocation_record_tree_new_node_count(&touched, 2),
        9,
        "R-3c：第 61 与第 170 片叶应给出 9"
    );
}

/// U10（第 3 次重跑登记第九节）：β0 之后连续覆盖写，逐次核对 R-3 的闭式；两个方向（落在 1 片叶 / 落在 ≥ 2 片叶）
/// 都至少出现一次。前 4 次另外钉 R-4 的绝对值（released_d0=14、record_delta=24）。U12（Q7d-1）：这段历史上逐次现量的
/// 最小值 14、最大值 ≥ 16——M30（Q7d-1 覆盖写那一组退回字面 10）应当让最小值变成 10。
#[test]
fn overwrite_steps_match_the_closed_form_and_cross_a_second_leaf() {
    let (mut allocator, first, mut devices) = new_pool_file_creation_state();
    let parameters = parameters();
    let instance = InstanceGeneration(1);
    let mut current = first;
    let mut release_samples: Vec<u64> = Vec::new();
    let mut saw_single_leaf_step = false;
    let mut saw_multi_leaf_step = false;
    for step in 1..=12u64 {
        let existing_leaves = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
        let records_before = allocator.records().len();
        current = {
            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
            singlefs_core::transaction::publish_overwrite(
                &mut pool,
                &mut allocator,
                &current,
                FirstFile {
                    content: &super::overwrite_content(step),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 + step,
                },
                instance,
            )
            .expect("U10 覆盖写")
        };
        let released = self_release_slots_of_this_publish(
            &allocator,
            DeviceIdentity(0),
            current.root.checkpoint_txg,
        );
        let touched =
            e156_touched_leaf_positions(&allocator, DeviceIdentity(0), current.root.checkpoint_txg);
        let (expected_released, expected_delta) =
            e156_closed_form_expected(&existing_leaves, &touched, true);
        let record_count = allocator.records().len();
        let delta =
            u64::try_from(record_count - records_before).expect("这一步的记录增量落在 u64 内");
        assert_eq!(
            released, expected_released,
            "U10：第 {step} 次覆盖写释放槽数应等于闭式"
        );
        assert_eq!(
            delta, expected_delta,
            "U10：第 {step} 次覆盖写记录增量应等于闭式"
        );
        if step <= 4 {
            assert_eq!(released, 14, "R-4：第 {step} 次覆盖写 D0 释放槽数应为 14");
            assert_eq!(delta, 24, "R-4：第 {step} 次覆盖写记录增量应为 24");
        }
        release_samples.push(released);
        if touched.len() <= 1 {
            saw_single_leaf_step = true;
        } else {
            saw_multi_leaf_step = true;
        }
    }
    assert!(saw_single_leaf_step, "U10：应当至少有一步只落在 1 片叶");
    assert!(
        saw_multi_leaf_step,
        "U10：应当至少有一步跨到 ≥ 2 片叶（第八节 8.2）"
    );
    let (minimum, maximum) = super::e156_minimum_and_maximum(&release_samples);
    assert_eq!(minimum, 14, "U12：Q7d-1 覆盖写那一组的最小值应为 14");
    assert!(
        maximum >= 16,
        "U12：Q7d-1 覆盖写那一组的最大值应 ≥ 16（跨叶时更贵）"
    );
}
