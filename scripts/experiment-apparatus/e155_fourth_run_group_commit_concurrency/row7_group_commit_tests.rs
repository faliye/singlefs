use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// B12（第七节 7.2）：P=100、F1、seq、concurrent_fsync_count=2（六棵树全是单节点，共享与不共享同值、位置策略不起作用）。
#[test]
fn batch_anchors_at_file_count_100_concurrent_fsync_count_2_match_the_hand_calculation() {
    for shared in [true, false] {
        let shape = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
        let jia = solve_jia_batch_publish(shape, 2, shared);
        assert_eq!(jia.total_bytes(), 410_112, "shared={shared}");
        assert_eq!(jia.write_calls, 23, "shared={shared}");
        assert_eq!(jia.allocation_layers, vec![1], "shared={shared}");

        let write_ahead_log_full = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, 2, shared);
        assert_eq!(write_ahead_log_full.total_bytes(), 270_336, "shared={shared}");
        assert_eq!(write_ahead_log_full.write_calls, 12, "shared={shared}");

        let write_ahead_log_leaf = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, 2, shared);
        assert_eq!(write_ahead_log_leaf.total_bytes(), 237_568, "shared={shared}");
        assert_eq!(write_ahead_log_leaf.write_calls, 10, "shared={shared}");
    }

    // 每次 fsync 摊到的字节 / 写请求：甲 = 一批 ÷ concurrent_fsync_count。
    let shape = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let jia = solve_jia_batch_publish(shape, 2, true);
    assert!((amortize_batch_per_fsync(jia.total_bytes(), 2) - 205_056.0).abs() < 1e-9);
    assert!((amortize_batch_per_fsync(jia.write_calls, 2) - 11.5).abs() < 1e-9);

    // g=0、N=16（N_b=8）：wal_full-K10 / 乙-M-K10 checkpoint 与摊销；中间版 21 / 14；
    // g=0 下 K10 与 K9 字节相同（积压乘数是 0）。
    let no_backlog_shape = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let no_backlog_shape = PoolShape { geometry: Geometry { backlog_generations: 0, ..no_backlog_shape.geometry }, ..no_backlog_shape };
    let write_ahead_log_full_fsync = solve_write_ahead_log_batch_fsync(no_backlog_shape, WriteAheadLogArm::WriteAheadLogFull, 2, true);
    let write_ahead_log_leaf_fsync = solve_write_ahead_log_batch_fsync(no_backlog_shape, WriteAheadLogArm::WriteAheadLogLeaf, 2, true);
    let batches_in_interval = batches_in_interval_by_fsync_count(16, 2);
    assert_eq!(batches_in_interval, 8, "N=16、concurrent_fsync_count=2 应该是 8 批（N_b）");
    let checkpoint_full_released_and_backlogged = solve_write_ahead_log_batch_checkpoint(no_backlog_shape, WriteAheadLogArm::WriteAheadLogFull, 2, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
    let checkpoint_leaf_released_and_backlogged = solve_write_ahead_log_batch_checkpoint(no_backlog_shape, WriteAheadLogArm::WriteAheadLogLeaf, 2, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
    let checkpoint_full_not_tracked = solve_write_ahead_log_batch_checkpoint(no_backlog_shape, WriteAheadLogArm::WriteAheadLogFull, 2, true, batches_in_interval, IntermediateVersionPolicy::NotTracked);
    assert_eq!(checkpoint_full_released_and_backlogged.total_bytes(), 147_968);
    assert_eq!(checkpoint_full_released_and_backlogged.write_calls, 13);
    assert_eq!(checkpoint_leaf_released_and_backlogged.total_bytes(), 180_736);
    assert_eq!(checkpoint_leaf_released_and_backlogged.write_calls, 15);
    assert_eq!(checkpoint_full_released_and_backlogged.intermediate_version_total, 21.0, "wal_full 中间版");
    assert_eq!(checkpoint_leaf_released_and_backlogged.intermediate_version_total, 14.0, "乙-M 中间版");
    assert_eq!(checkpoint_full_released_and_backlogged.total_bytes(), checkpoint_full_not_tracked.total_bytes(), "g=0 下 K10 与 K9 字节应该相同");

    let amortized_full = (write_ahead_log_full_fsync.total_bytes() as f64 * 8.0 + checkpoint_full_released_and_backlogged.total_bytes() as f64) / 16.0;
    let amortized_leaf = (write_ahead_log_leaf_fsync.total_bytes() as f64 * 8.0 + checkpoint_leaf_released_and_backlogged.total_bytes() as f64) / 16.0;
    assert!((amortized_full - 144_416.0).abs() < 1e-6, "amortized_full={amortized_full}");
    assert!((amortized_leaf - 130_080.0).abs() < 1e-6, "amortized_leaf={amortized_leaf}");
}

/// B13（第七节 7.2）：P=10⁴、concurrent_fsync_count=2，一批的用户树逐层脏节点，共享与不共享分开算。
#[test]
fn batch_dirty_nodes_per_layer_at_file_count_10000_concurrent_fsync_count_2_match_the_hand_calculation() {
    let shape_one_data_unit_per_file = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let shared_one_data_unit_per_file = solve_jia_batch_publish(shape_one_data_unit_per_file, 2, true);
    assert_eq!(shared_one_data_unit_per_file.extent_layers, vec![70, 1]);
    assert!((shared_one_data_unit_per_file.extent_dirty[0] - 1.006944).abs() < 1e-6);
    assert!((shared_one_data_unit_per_file.extent_dirty[1] - 1.0).abs() < 1e-9);
    assert_eq!(shared_one_data_unit_per_file.inode_layers, vec![43, 1]);
    assert!((shared_one_data_unit_per_file.inode_dirty[0] - 1.004292).abs() < 1e-6);

    let unshared_one_data_unit_per_file = solve_jia_batch_publish(shape_one_data_unit_per_file, 2, false);
    assert!((unshared_one_data_unit_per_file.extent_dirty[0] - 2.0).abs() < 1e-9);
    assert!((unshared_one_data_unit_per_file.inode_dirty[0] - 2.0).abs() < 1e-9);

    let shape_eight_adjacent = shape_at(10_000, Family::EightAdjacentDataUnitsPerFsync, Placement::Sequential, PositionPolicy::Balanced);
    let shared_eight_adjacent = solve_jia_batch_publish(shape_eight_adjacent, 2, true);
    assert_eq!(shared_eight_adjacent.extent_layers, vec![556, 4, 1]);
    assert!((shared_eight_adjacent.extent_dirty[0] - 1.104167).abs() < 1e-6);
    assert!((shared_eight_adjacent.extent_dirty[1] - 1.000709).abs() < 1e-6);
    assert!((shared_eight_adjacent.extent_dirty[2] - 1.0).abs() < 1e-9);
    // inode 同 F1（file_count 相同，与族无关）。
    assert!((shared_eight_adjacent.inode_dirty[0] - 1.004292).abs() < 1e-6);

    let unshared_eight_adjacent = solve_jia_batch_publish(shape_eight_adjacent, 2, false);
    assert!((unshared_eight_adjacent.extent_dirty[0] - 2.097222).abs() < 1e-6);
    assert!((unshared_eight_adjacent.extent_dirty[1] - 2.000661).abs() < 1e-6);
    assert!((unshared_eight_adjacent.inode_dirty[0] - 2.0).abs() < 1e-9);
}

/// B14（第七节 7.2）：每次 fsync 摊到的刷写与 FUA，随 concurrent_fsync_count、N 变化。
#[test]
fn flush_and_fua_per_fsync_match_the_hand_calculation() {
    // 每次 fsync 摊到的刷写 / FUA：一批的 barriers/fua_count（batch 结构体自己的字段，不是手算的常数）
    // 乘 N_b（这个间隔的批数）再加 checkpoint 自己的 barriers/fua_count，除以 N——用真实函数的
    // 输出算，M17（WAL 两臂每批的刷写按 concurrent_fsync_count 个 fsync 各算一次）改了 `barriers` 字段
    // 才会被这条测试看见。
    fn write_ahead_log_flushes_and_fua_per_fsync(shape: PoolShape, concurrent_fsync_count: u64, interval_fsyncs: u64) -> (f64, f64) {
        let batch = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true);
        let batches_in_interval = batches_in_interval_by_fsync_count(interval_fsyncs, concurrent_fsync_count);
        let checkpoint = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_in_interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let flushes = (batch.barriers as f64 * batches_in_interval as f64 + checkpoint.barriers as f64) / interval_fsyncs as f64;
        let fua = (0.0 * batches_in_interval as f64 + checkpoint.fua_count as f64) / interval_fsyncs as f64;
        (flushes, fua)
    }
    let shape = shape_at(1_000_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);

    // concurrent_fsync_count=2、N=16（N_b=8）：甲 2/0.5；WAL 两臂 2.25/0.0625。
    assert!((amortize_batch_per_fsync(2 * DEVICE_COUNT, 2) - 2.0).abs() < 1e-9);
    assert!((amortize_batch_per_fsync(1, 2) - 0.5).abs() < 1e-9);
    let (write_ahead_log_flushes_per_fsync, write_ahead_log_fua_per_fsync) = write_ahead_log_flushes_and_fua_per_fsync(shape, 2, 16);
    assert!((write_ahead_log_flushes_per_fsync - 2.25).abs() < 1e-9, "write_ahead_log_flushes_per_fsync={write_ahead_log_flushes_per_fsync}");
    assert!((write_ahead_log_fua_per_fsync - 0.0625).abs() < 1e-9);

    // concurrent_fsync_count=16、N=16（N_b=1）：甲 0.25/0.0625；WAL 0.5/0.0625。
    assert!((amortize_batch_per_fsync(2 * DEVICE_COUNT, 16) - 0.25).abs() < 1e-9);
    assert!((amortize_batch_per_fsync(1, 16) - 0.0625).abs() < 1e-9);
    let (write_ahead_log_flushes_per_fsync_at_concurrent_fsync_count_16, _) = write_ahead_log_flushes_and_fua_per_fsync(shape, 16, 16);
    assert!((write_ahead_log_flushes_per_fsync_at_concurrent_fsync_count_16 - 0.5).abs() < 1e-9, "value={write_ahead_log_flushes_per_fsync_at_concurrent_fsync_count_16}");

    // concurrent_fsync_count=2、N=4096（N_b=2048）：WAL 2.0009765625。
    let (write_ahead_log_flushes_per_fsync_at_interval_4096, write_ahead_log_fua_per_fsync_at_interval_4096) = write_ahead_log_flushes_and_fua_per_fsync(shape, 2, 4096);
    assert!((write_ahead_log_flushes_per_fsync_at_interval_4096 - 2.0009765625).abs() < 1e-9, "value={write_ahead_log_flushes_per_fsync_at_interval_4096}");
    assert!((write_ahead_log_fua_per_fsync_at_interval_4096 - 0.000244140625).abs() < 1e-9);
}

/// 阳性对照（5.5，第 7 行）：共享 / 不共享在 B13 那一格上必须给出登记原文的差；
/// 甲组提交（concurrent_fsync_count=2 对 concurrent_fsync_count=1）必须省下固定点/记录/根槽/系统配置那一份重复开销。
#[test]
fn positive_controls_for_row_7_show_the_prescribed_difference() {
    let shape = shape_at(10_000, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let shared = solve_jia_batch_publish(shape, 2, true);
    let unshared = solve_jia_batch_publish(shape, 2, false);
    assert!(unshared.extent_bytes > shared.extent_bytes, "不共享应该比共享碰更多 extent 节点");

    let shape_100 = shape_at(100, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let batch_bytes_at_concurrent_fsync_count_one = solve_jia_batch_publish(shape_100, 1, true).total_bytes();
    let batch_bytes_at_concurrent_fsync_count_two = solve_jia_batch_publish(shape_100, 2, true).total_bytes();
    assert_eq!(batch_bytes_at_concurrent_fsync_count_one, 344_576, "concurrent_fsync_count=1 应该与第 6 行 P=100 一格相同");
    assert!((batch_bytes_at_concurrent_fsync_count_two as f64) < 2.0 * batch_bytes_at_concurrent_fsync_count_one as f64, "组提交应该比两次各自发布省钱：batch_bytes_at_concurrent_fsync_count_one×2={} batch_bytes_at_concurrent_fsync_count_two={}", batch_bytes_at_concurrent_fsync_count_one * 2, batch_bytes_at_concurrent_fsync_count_two);
}

/// concurrent_fsync_count=1 时批级函数应该逐字节回到第 6 行的读数（甲、wal_full、乙-M 都要核）。
#[test]
fn batch_functions_at_concurrent_fsync_count_1_match_the_row_6_functions() {
    for shared in [true, false] {
        for file_count in [1u64, 145, 10_000] {
            let shape = shape_at(file_count, Family::OneDataUnitPerFile, Placement::Random, PositionPolicy::Far);
            let batch = solve_jia_batch_publish(shape, 1, shared);
            let single = solve_jia_publish(shape);
            assert_eq!(batch.total_bytes(), single.total_bytes(), "shared={shared} file_count={file_count}");
            assert_eq!(batch.write_calls, single.write_calls, "shared={shared} file_count={file_count}");

            for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
                let batch_fsync = solve_write_ahead_log_batch_fsync(shape, arm, 1, shared);
                let single_fsync = solve_write_ahead_log_fsync(shape, arm);
                assert_eq!(batch_fsync.total_bytes(), single_fsync.total_bytes(), "arm={arm:?} shared={shared} file_count={file_count}");

                let batch_checkpoint = solve_write_ahead_log_batch_checkpoint(shape, arm, 1, shared, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
                let single_checkpoint = solve_write_ahead_log_checkpoint(shape, arm, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
                assert_eq!(batch_checkpoint.total_bytes(), single_checkpoint.total_bytes(), "arm={arm:?} shared={shared} file_count={file_count}");
                assert_eq!(batch_checkpoint.intermediate_version_total, single_checkpoint.intermediate_version_total, "arm={arm:?} shared={shared} file_count={file_count}");
            }
        }
    }
}
