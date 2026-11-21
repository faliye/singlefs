use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// C（第五节 5.3）：P=100、F1、seq、g=0、主几何其余不变——六棵树全是单节点，共享 = 不共享，三种位置策略同值。
fn control_shape_without_backlog() -> PoolShape {
    let shape = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    PoolShape { geometry: Geometry { backlog_generations: 0, ..shape.geometry }, ..shape }
}

/// B-r4-9：k ≤ N 与 k > N 两种情形下 N_b、间隔实际 fsync 数都要对上手算表。
#[test]
fn batches_and_effective_fsyncs_match_the_table_for_concurrent_fsync_count_around_interval() {
    assert_eq!(batches_in_interval_for_concurrent_fsync_count(16, 8), 2);
    assert_eq!(effective_fsyncs_in_interval(16, 8), 16);
    assert_eq!(batches_in_interval_for_concurrent_fsync_count(16, 16), 1);
    assert_eq!(effective_fsyncs_in_interval(16, 16), 16);
    assert_eq!(batches_in_interval_for_concurrent_fsync_count(16, 32), 1, "k > N：一批已经把间隔撑满，N_b = 1");
    assert_eq!(effective_fsyncs_in_interval(16, 32), 32, "k > N：间隔实际 fsync 数换成 k");
    assert_eq!(batches_in_interval_for_concurrent_fsync_count(256, 64), 4);
    assert_eq!(effective_fsyncs_in_interval(256, 64), 256);
    assert_eq!(batches_in_interval_for_concurrent_fsync_count(256, 512), 1);
    assert_eq!(effective_fsyncs_in_interval(256, 512), 512);
    assert_eq!(batches_in_interval_for_concurrent_fsync_count(4096, 1024), 4);
    assert_eq!(effective_fsyncs_in_interval(4096, 1024), 4096);
}

/// B-r4-0（停机条款 S1）：甲，P=1、F1、seq、k=1、主几何——与 `crates` 发布 B 逐项相同。
#[test]
fn jia_batch_at_concurrent_fsync_count_one_matches_the_crates_publish_anchor() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let jia = solve_jia_batch_publish(shape, 1, true);
    assert_eq!(jia.total_bytes(), 344_576);
    assert_eq!(jia.write_calls, 21);
    assert_eq!(jia.barriers, 4);
    assert_eq!(jia.fua_count, 1);
    assert_eq!(jia.record_count, 1);
}

/// B-r4-1（第七节 7.2）：C，甲——一批的（字节, 写调用, 单元数, 记录数）随 k 到 64。
#[test]
fn jia_batch_anchors_on_control_shape_match_the_hand_calculation() {
    let shape = control_shape_without_backlog();
    for (concurrent_fsync_count, shared, expected) in [
        (1u64, true, (344_576u64, 21u64, 8u64, 1u64)),
        (2, true, (410_112, 23, 9, 1)),
        (16, true, (1_327_616, 51, 23, 1)),
        (32, true, (2_376_192, 83, 39, 1)),
        (64, true, (4_481_536, 149, 71, 2)),
    ] {
        let jia = solve_jia_batch_publish(shape, concurrent_fsync_count, shared);
        assert_eq!(jia.total_bytes(), expected.0, "k={concurrent_fsync_count}");
        assert_eq!(jia.write_calls, expected.1, "k={concurrent_fsync_count}");
        assert_eq!(jia.named_items, expected.2, "k={concurrent_fsync_count}");
        assert_eq!(jia.record_count, expected.3, "k={concurrent_fsync_count}");
    }
}

/// B-r4-2：C，wal_full——一批的（字节, 写调用, 单元数, 记录数）随 k 到 64。「单元数」不是
/// `named_items`（wal_full 的 `named_items` 恒 2，是记录里点名的树根数，不是写出的单元数）；
/// `write_calls = unit_count × DEVICE_COUNT + record_count × DEVICE_COUNT`（结构体不直接存
/// `unit_count`），这里按同一条公式反推出来，跟 `anchors_fourth_run.py` 的 `calls` 定义对齐。
#[test]
fn write_ahead_log_full_batch_anchors_on_control_shape_match_the_hand_calculation() {
    let shape = control_shape_without_backlog();
    for (concurrent_fsync_count, expected) in [
        (1u64, (204_800u64, 10u64, 4u64, 1u64)),
        (2, (270_336, 12, 5, 1)),
        (16, (1_187_840, 40, 19, 1)),
        (32, (2_236_416, 72, 35, 1)),
        (64, (4_333_568, 136, 67, 1)),
    ] {
        let write_ahead_log_full = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true);
        let unit_count = write_ahead_log_full.write_calls / DEVICE_COUNT - write_ahead_log_full.record_count;
        assert_eq!(write_ahead_log_full.total_bytes(), expected.0, "k={concurrent_fsync_count}");
        assert_eq!(write_ahead_log_full.write_calls, expected.1, "k={concurrent_fsync_count}");
        assert_eq!(unit_count, expected.2, "k={concurrent_fsync_count}");
        assert_eq!(write_ahead_log_full.record_count, expected.3, "k={concurrent_fsync_count}");
    }
}

/// B-r4-3：C，乙-M——一批的（字节, 写调用, 单元数, 记录数）随 k 到 64（单元数的算法同 B-r4-2）。
#[test]
fn write_ahead_log_leaf_batch_anchors_on_control_shape_match_the_hand_calculation() {
    let shape = control_shape_without_backlog();
    for (concurrent_fsync_count, expected) in [
        (1u64, (172_032u64, 8u64, 3u64, 1u64)),
        (2, (237_568, 10, 4, 1)),
        (16, (1_155_072, 38, 18, 1)),
        (32, (2_203_648, 70, 34, 1)),
        (64, (4_300_800, 134, 66, 1)),
    ] {
        let write_ahead_log_leaf = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, true);
        let unit_count = write_ahead_log_leaf.write_calls / DEVICE_COUNT - write_ahead_log_leaf.record_count;
        assert_eq!(write_ahead_log_leaf.total_bytes(), expected.0, "k={concurrent_fsync_count}");
        assert_eq!(write_ahead_log_leaf.write_calls, expected.1, "k={concurrent_fsync_count}");
        assert_eq!(unit_count, expected.2, "k={concurrent_fsync_count}");
        assert_eq!(write_ahead_log_leaf.record_count, expected.3, "k={concurrent_fsync_count}");
    }
}

/// B-r4-4：C 的 checkpoint（g=0 下与 k、N 无关）。
#[test]
fn checkpoint_anchors_on_control_shape_are_independent_of_concurrent_fsync_count_and_interval() {
    let shape = control_shape_without_backlog();
    let batches_in_interval = batches_in_interval_for_concurrent_fsync_count(256, 16);
    let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, 16, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert_eq!(checkpoint_full.total_bytes(), 147_968);
    assert_eq!(checkpoint_full.write_calls, 13);
    let checkpoint_leaf = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, 16, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert_eq!(checkpoint_leaf.total_bytes(), 180_736);
    assert_eq!(checkpoint_leaf.write_calls, 15);
}

/// B-r4-5：C，N=256 的间隔中间版 I（活数据单元 U=100）随 k 到 64。
#[test]
fn intermediate_version_totals_on_control_shape_match_the_hand_calculation() {
    let shape = control_shape_without_backlog();
    for (concurrent_fsync_count, expected_batches, expected_full, expected_leaf) in [(16u64, 16u64, 201.0, 186.0), (32, 8, 177.0, 170.0), (64, 4, 165.0, 162.0)] {
        let batches_in_interval = batches_in_interval_for_concurrent_fsync_count(256, concurrent_fsync_count);
        assert_eq!(batches_in_interval, expected_batches, "k={concurrent_fsync_count}");
        let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
        assert!((checkpoint_full.intermediate_version_total - expected_full).abs() < 1e-6, "k={concurrent_fsync_count} value={}", checkpoint_full.intermediate_version_total);
        let checkpoint_leaf = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
        assert!((checkpoint_leaf.intermediate_version_total - expected_leaf).abs() < 1e-6, "k={concurrent_fsync_count} value={}", checkpoint_leaf.intermediate_version_total);
    }
}

/// B-r4-6：C，N=256 每次 fsync 摊到的字节 / 写调用，随 k 到 64。
#[test]
fn amortized_bytes_and_calls_on_control_shape_match_the_hand_calculation() {
    let shape = control_shape_without_backlog();
    for (concurrent_fsync_count, expected_jia_bytes, expected_jia_calls, expected_full_bytes, expected_full_calls, expected_leaf_bytes, expected_leaf_calls) in [
        (1u64, 344_576.0, 21.0, 205_378.0, 10.05078125, 172_738.0, 8.05859375),
        (16, 82_976.0, 3.1875, 74_818.0, 2.55078125, 72_898.0, 2.43359375),
        (32, 74_256.0, 2.59375, 70_466.0, 2.30078125, 69_570.0, 2.24609375),
        (64, 70_024.0, 2.328125, 68_290.0, 2.17578125, 67_906.0, 2.15234375),
    ] {
        let jia = solve_jia_batch_publish(shape, concurrent_fsync_count, true);
        let jia_amortized_bytes = amortize_batch_per_fsync(jia.total_bytes(), concurrent_fsync_count);
        let jia_amortized_calls = amortize_batch_per_fsync(jia.write_calls, concurrent_fsync_count);
        assert!((jia_amortized_bytes - expected_jia_bytes).abs() < 1e-6, "k={concurrent_fsync_count} value={jia_amortized_bytes}");
        assert!((jia_amortized_calls - expected_jia_calls).abs() < 1e-9, "k={concurrent_fsync_count} value={jia_amortized_calls}");

        let batches_in_interval = batches_in_interval_for_concurrent_fsync_count(256, concurrent_fsync_count);
        let effective_fsyncs = effective_fsyncs_in_interval(256, concurrent_fsync_count);
        let write_ahead_log_full_fsync = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true);
        let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let amortized_full_bytes = (write_ahead_log_full_fsync.total_bytes() as f64 * batches_in_interval as f64 + checkpoint_full.total_bytes() as f64) / effective_fsyncs as f64;
        let amortized_full_calls = (write_ahead_log_full_fsync.write_calls as f64 * batches_in_interval as f64 + checkpoint_full.write_calls as f64) / effective_fsyncs as f64;
        assert!((amortized_full_bytes - expected_full_bytes).abs() < 1e-6, "k={concurrent_fsync_count} value={amortized_full_bytes}");
        assert!((amortized_full_calls - expected_full_calls).abs() < 1e-9, "k={concurrent_fsync_count} value={amortized_full_calls}");

        let write_ahead_log_leaf_fsync = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, true);
        let checkpoint_leaf = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let amortized_leaf_bytes = (write_ahead_log_leaf_fsync.total_bytes() as f64 * batches_in_interval as f64 + checkpoint_leaf.total_bytes() as f64) / effective_fsyncs as f64;
        let amortized_leaf_calls = (write_ahead_log_leaf_fsync.write_calls as f64 * batches_in_interval as f64 + checkpoint_leaf.write_calls as f64) / effective_fsyncs as f64;
        assert!((amortized_leaf_bytes - expected_leaf_bytes).abs() < 1e-6, "k={concurrent_fsync_count} value={amortized_leaf_bytes}");
        assert!((amortized_leaf_calls - expected_leaf_calls).abs() < 1e-9, "k={concurrent_fsync_count} value={amortized_leaf_calls}");
    }
}

/// B-r4-7：C，N=256 的刷写 / FUA（每次 fsync）。
#[test]
fn flush_and_fua_per_fsync_on_control_shape_match_the_hand_calculation() {
    let shape = control_shape_without_backlog();
    for (concurrent_fsync_count, expected_jia_flushes, expected_jia_fua, expected_write_ahead_log_flushes, expected_write_ahead_log_fua) in [(32u64, 0.125, 0.03125, 0.140625, 0.00390625), (64, 0.0625, 0.015625, 0.078125, 0.00390625)] {
        let jia_batch = solve_jia_batch_publish(shape, concurrent_fsync_count, true);
        let jia_flushes = amortize_batch_per_fsync(jia_batch.barriers, concurrent_fsync_count);
        let jia_fua = amortize_batch_per_fsync(jia_batch.fua_count, concurrent_fsync_count);
        assert!((jia_flushes - expected_jia_flushes).abs() < 1e-9, "k={concurrent_fsync_count} value={jia_flushes}");
        assert!((jia_fua - expected_jia_fua).abs() < 1e-9, "k={concurrent_fsync_count} value={jia_fua}");

        let batches_in_interval = batches_in_interval_for_concurrent_fsync_count(256, concurrent_fsync_count);
        let effective_fsyncs = effective_fsyncs_in_interval(256, concurrent_fsync_count);
        let write_ahead_log_full_fsync = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true);
        let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let write_ahead_log_flushes = (write_ahead_log_full_fsync.barriers as f64 * batches_in_interval as f64 + checkpoint_full.barriers as f64) / effective_fsyncs as f64;
        let write_ahead_log_fua = (0.0 * batches_in_interval as f64 + checkpoint_full.fua_count as f64) / effective_fsyncs as f64;
        assert!((write_ahead_log_flushes - expected_write_ahead_log_flushes).abs() < 1e-9, "concurrent_fsync_count={concurrent_fsync_count} value={write_ahead_log_flushes}");
        assert!((write_ahead_log_fua - expected_write_ahead_log_fua).abs() < 1e-9, "concurrent_fsync_count={concurrent_fsync_count} value={write_ahead_log_fua}");
    }
}

/// B-r4-8（阳性对照第四行）：P=10⁴、F1/F8A、k=1024，一批的用户树逐层脏节点（共享 / 不共享）。
#[test]
fn dirty_nodes_per_layer_at_file_count_10000_concurrent_fsync_count_1024_match_the_hand_calculation() {
    let shape_one_data_unit_per_file = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let shared_one_data_unit_per_file = solve_jia_batch_publish(shape_one_data_unit_per_file, 1024, true);
    assert_eq!(shared_one_data_unit_per_file.extent_layers, vec![70, 1]);
    assert!((shared_one_data_unit_per_file.extent_dirty[0] - 8.104_166_666_666_668).abs() < 1e-9, "value={}", shared_one_data_unit_per_file.extent_dirty[0]);
    assert!((shared_one_data_unit_per_file.extent_dirty[1] - 1.0).abs() < 1e-9);
    assert_eq!(shared_one_data_unit_per_file.inode_layers, vec![43, 1]);
    assert!((shared_one_data_unit_per_file.inode_dirty[0] - 5.390_557_939_914_163).abs() < 1e-9, "value={}", shared_one_data_unit_per_file.inode_dirty[0]);

    let unshared_one_data_unit_per_file = solve_jia_batch_publish(shape_one_data_unit_per_file, 1024, false);
    assert!((unshared_one_data_unit_per_file.extent_dirty[0] - 70.0).abs() < 1e-9, "unshared extent 应该饱和在 n_l=70：{}", unshared_one_data_unit_per_file.extent_dirty[0]);
    assert!((unshared_one_data_unit_per_file.inode_dirty[0] - 43.0).abs() < 1e-9, "unshared inode 应该饱和在 n_l=43：{}", unshared_one_data_unit_per_file.inode_dirty[0]);

    let shape_f8a = shape_at(10_000, Family::EightAdjacentDataUnitsPerFsync, Placement::Sequential, PositionPolicy::Balanced);
    let shared_f8a = solve_jia_batch_publish(shape_f8a, 1024, true);
    assert_eq!(shared_f8a.extent_layers, vec![556, 4, 1]);
    assert!((shared_f8a.extent_dirty[0] - 57.881_944_444_444_44).abs() < 1e-6, "value={}", shared_f8a.extent_dirty[0]);
    assert!((shared_f8a.extent_dirty[1] - 1.386_952_003_023_431_7).abs() < 1e-6, "value={}", shared_f8a.extent_dirty[1]);

    let unshared_f8a = solve_jia_batch_publish(shape_f8a, 1024, false);
    assert!((unshared_f8a.extent_dirty[0] - 556.0).abs() < 1e-9, "unshared F8A extent 应该饱和在 n_l=556：{}", unshared_f8a.extent_dirty[0]);

    let extent_saturated = saturated_layer_indices(&unshared_one_data_unit_per_file.extent_layers, &unshared_one_data_unit_per_file.extent_dirty);
    assert_eq!(extent_saturated, vec![0], "只有第 0 层饱和（Q7.16）");
    let inode_saturated = saturated_layer_indices(&unshared_one_data_unit_per_file.inode_layers, &unshared_one_data_unit_per_file.inode_dirty);
    assert_eq!(inode_saturated, vec![0]);
}

/// B-r4-10：P=10⁴、F1、不共享，一批的脏叶数与记录条数随 k（乙-M 的 T5 轨迹）。
#[test]
fn write_ahead_log_leaf_dirty_leaves_and_record_count_track_saturation_at_large_concurrent_fsync_count() {
    let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    for (concurrent_fsync_count, expected_named_items, expected_record_count) in [(16u64, 32u64, 1u64), (64, 107, 2), (128, 113, 2), (256, 113, 2), (1024, 113, 2)] {
        let write_ahead_log_leaf = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, false);
        assert_eq!(write_ahead_log_leaf.named_items, expected_named_items, "k={concurrent_fsync_count}");
        assert_eq!(write_ahead_log_leaf.record_count, expected_record_count, "k={concurrent_fsync_count}");
    }
}

// ---- 六行阳性对照（5.3）----

/// 阳性对照第一行：甲（组提交，大 k），C，k=64。
#[test]
fn positive_control_row_1_jia_batch_at_large_concurrent_fsync_count_shows_the_prescribed_difference() {
    let shape = control_shape_without_backlog();
    let jia_batch = solve_jia_batch_publish(shape, 64, true);
    let write_ahead_log_full_batch = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 64, true);
    assert_eq!(jia_batch.total_bytes() - write_ahead_log_full_batch.total_bytes(), 147_968);
    assert_eq!(jia_batch.total_bytes(), 4_481_536);
    assert_eq!(jia_batch.write_calls, 149);
    let amortized = amortize_batch_per_fsync(jia_batch.total_bytes(), 64);
    assert!((amortized - 70_024.0).abs() < 1e-6);
}

/// 阳性对照第二行：wal_full-K10（大 k），C，k=64。
#[test]
fn positive_control_row_2_write_ahead_log_full_at_large_concurrent_fsync_count_shows_the_prescribed_difference() {
    let shape = control_shape_without_backlog();
    let write_ahead_log_full_batch = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 64, true);
    let write_ahead_log_leaf_batch = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 64, true);
    assert_eq!(write_ahead_log_full_batch.total_bytes() - write_ahead_log_leaf_batch.total_bytes(), 32_768);
    assert_eq!(write_ahead_log_full_batch.total_bytes(), 4_333_568);
    assert_eq!(write_ahead_log_full_batch.write_calls, 136);
    let batches_in_interval = batches_in_interval_for_concurrent_fsync_count(256, 64);
    let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, 64, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert!((checkpoint_full.intermediate_version_total - 165.0).abs() < 1e-6);
}

/// 阳性对照第三行：乙-M-K10（大 k），C，k=64。
#[test]
fn positive_control_row_3_write_ahead_log_leaf_checkpoint_at_large_concurrent_fsync_count_shows_the_prescribed_difference() {
    let shape = control_shape_without_backlog();
    let batches_in_interval = batches_in_interval_for_concurrent_fsync_count(256, 64);
    let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, 64, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
    let checkpoint_leaf = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, 64, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert_eq!(checkpoint_leaf.total_bytes() - checkpoint_full.total_bytes(), 32_768);
    assert_eq!(checkpoint_leaf.total_bytes(), 180_736);
    assert_eq!(checkpoint_leaf.write_calls, 15);
    let write_ahead_log_leaf_batch = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 64, true);
    assert_eq!(write_ahead_log_leaf_batch.total_bytes(), 4_300_800);
    assert_eq!(write_ahead_log_leaf_batch.write_calls, 134);
}

/// 阳性对照第四行：三臂各跑，共享 / 不共享在大 k 上分得开——见 B-r4-8。
#[test]
fn positive_control_row_4_shared_and_unshared_separate_at_large_concurrent_fsync_count() {
    let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let shared = solve_jia_batch_publish(shape, 1024, true);
    let unshared = solve_jia_batch_publish(shape, 1024, false);
    assert!(unshared.extent_dirty[0] > shared.extent_dirty[0], "不共享应该比共享碰更多 extent 节点");
    assert!(!saturated_layer_indices(&unshared.extent_layers, &unshared.extent_dirty).is_empty(), "不共享应该在第 0 层饱和");
    assert!(saturated_layer_indices(&shared.extent_layers, &shared.extent_dirty).is_empty(), "共享不该饱和");
}

/// 阳性对照第五行：wal_full-K10 与乙-M-K10 的 K10 那一半，P=1、F1、seq、N=16、k=1、主几何。
#[test]
fn positive_control_row_5_released_and_backlogged_half_matches_the_second_run_anchor() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, 1, true, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert_eq!(checkpoint_full.intermediate_version_total, 60.0, "wal_full 中间版：数据 15+extent 15+inode 叶 15+inode 根 15");
    assert_eq!(checkpoint_full.allocation_layers, vec![5, 1]);
    let checkpoint_leaf = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, 1, true, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert_eq!(checkpoint_leaf.intermediate_version_total, 45.0);
    assert_eq!(checkpoint_leaf.allocation_layers, vec![4, 1]);
}

/// 阳性对照第六行（停机条款 S1）：甲（k=1 对拍），P=1、F1、seq、k=1、主几何——与 `crates` 发布 B 逐项。
#[test]
fn positive_control_row_6_jia_at_concurrent_fsync_count_1_matches_the_crates_anchor() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let jia = solve_jia_batch_publish(shape, 1, true);
    assert_eq!(jia.total_bytes(), 344_576);
    assert_eq!(jia.write_calls, 21);
    assert_eq!(jia.barriers, 4);
    assert_eq!(jia.fua_count, 1);
}

// ---- M1–M6 变异目标（数值锚点在 `scripts/mutation/tables/e155_fourth_run_group_commit_concurrency.tsv`）----

/// M1 的正确性锚点：`batches_in_interval_for_concurrent_fsync_count` 在 k > N 时必须是 1，不是 N/k。
#[test]
fn batches_in_interval_is_one_when_concurrent_fsync_count_exceeds_interval() {
    assert_eq!(batches_in_interval_for_concurrent_fsync_count(16, 32), 1);
}

/// M2 的正确性锚点：不共享在大 k 上必须封顶在 n_l，不能超过它。
#[test]
fn unshared_extent_layer_zero_is_capped_at_node_count() {
    let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let unshared = solve_jia_batch_publish(shape, 1024, false);
    assert_eq!(unshared.extent_layers[0], 70);
    assert!((unshared.extent_dirty[0] - 70.0).abs() < 1e-9, "value={}", unshared.extent_dirty[0]);
}

/// M3 的正确性锚点：共享在大 k 上按连续组公式碰的节点数，不是恒 1。
#[test]
fn shared_extent_layer_zero_uses_the_continuous_group_size() {
    let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let shared = solve_jia_batch_publish(shape, 1024, true);
    assert!((shared.extent_dirty[0] - 8.104_166_666_666_668).abs() < 1e-9, "value={}", shared.extent_dirty[0]);
}

/// M4 的正确性锚点：甲一批的记录条数按 ⌈W / 67⌉ 算，C、k=64 是 2 条。
#[test]
fn jia_batch_record_count_follows_named_items_per_record_at_large_concurrent_fsync_count() {
    let shape = control_shape_without_backlog();
    let jia = solve_jia_batch_publish(shape, 64, true);
    assert_eq!(jia.record_count, 2, "k=64 一批 71 个点名项，⌈71/67⌉=2");
    assert_eq!(jia.total_bytes(), 4_481_536);
    assert_eq!(jia.write_calls, 149);
}

/// M5 的正确性锚点：乙-M 一批的记录条数按 ⌈脏叶数 / 67⌉ 算，P=10⁴、不共享、k=128 是 2 条。
#[test]
fn write_ahead_log_leaf_batch_record_count_follows_dirty_leaves_at_large_concurrent_fsync_count() {
    let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let write_ahead_log_leaf = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 128, false);
    assert_eq!(write_ahead_log_leaf.named_items, 113);
    assert_eq!(write_ahead_log_leaf.record_count, 2, "⌈113/67⌉=2（B-r4-10）");
}

/// M6 的正确性锚点：`effective_fsyncs_in_interval` 在 k > N 时必须换成 k，不能停在 N。
#[test]
fn effective_fsyncs_switches_to_concurrent_fsync_count_when_it_exceeds_interval() {
    assert_eq!(effective_fsyncs_in_interval(16, 32), 32);
}
