use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// B4（第七节 7.2）：wal_full-K10，P=1、F1、seq、主几何；K9 与 K10 在 N=1 逐字段相同（B19）。
#[test]
fn write_ahead_log_full_at_file_count_1_matches_the_anchor() {
    let fsync = solve_write_ahead_log_fsync(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull);
    assert_eq!(fsync.total_bytes(), 204_800);
    assert_eq!(fsync.write_calls, 10);
    assert_eq!(fsync.barriers, 4);

    let checkpoint = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::NotTracked);
    assert_eq!(checkpoint.total_bytes(), 147_968);
    assert_eq!(checkpoint.write_calls, 13);
    assert_eq!(checkpoint.fua_count, 1);
    assert_eq!(checkpoint.allocation_layers, vec![1], "K9：P=1、N=16 分配记录树仍单节点（402 条）");

    for interval in [1u64, 16] {
        let cp = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull, interval, IntermediateVersionPolicy::NotTracked);
        let amortized = amortize(fsync.total_bytes(), cp.total_bytes(), interval);
        let expected = if interval == 1 { 352_768.0 } else { 214_048.0 };
        assert!((amortized - expected).abs() < 1e-6, "interval={interval} amortized={amortized}");
    }
}

/// B5（第七节 7.2）：乙-M-K10，P=1、F1、seq、主几何。
#[test]
fn write_ahead_log_leaf_at_file_count_1_matches_the_anchor() {
    let fsync = solve_write_ahead_log_fsync(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf);
    assert_eq!(fsync.total_bytes(), 172_032);
    assert_eq!(fsync.write_calls, 8);
    assert_eq!(fsync.barriers, 4);

    let checkpoint = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf, 16, IntermediateVersionPolicy::NotTracked);
    assert_eq!(checkpoint.total_bytes(), 180_736);
    assert_eq!(checkpoint.write_calls, 15);

    for interval in [1u64, 16] {
        let cp = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf, interval, IntermediateVersionPolicy::NotTracked);
        let amortized = amortize(fsync.total_bytes(), cp.total_bytes(), interval);
        let expected = if interval == 1 { 352_768.0 } else { 183_328.0 };
        assert!((amortized - expected).abs() < 1e-6, "interval={interval} amortized={amortized}");
    }
}

/// 阳性对照（5.5）：三条臂各自的格与「必须出现的差」（第 6 行）。
#[test]
fn positive_controls_show_the_prescribed_difference_for_every_arm() {
    let shape_at_file_count_1 = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let jia_fsync = solve_jia_publish(shape_at_file_count_1).total_bytes();
    let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    assert_eq!(jia_fsync - write_ahead_log_full_fsync, 139_776, "甲：四样固定点 × 2 盘 + 根槽 + 系统配置槽 × 2");
    assert_eq!(write_ahead_log_full_fsync - write_ahead_log_leaf_fsync, 32_768, "wal_full：inode 根 × 2 盘");

    let shape_at_file_count_145 = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let write_ahead_log_leaf_at_file_count_145 = solve_write_ahead_log_fsync(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    let write_ahead_log_full_at_file_count_145 = solve_write_ahead_log_fsync(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    assert_eq!(write_ahead_log_leaf_at_file_count_145, 172_032, "乙-M fsync 行不含 extent 与 inode 两个根");
    assert_eq!(write_ahead_log_full_at_file_count_145, 237_568);
    let checkpoint_at_file_count_145 = solve_write_ahead_log_checkpoint(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert!(checkpoint_at_file_count_145.extent_catch_up_bytes > 0, "extent 根追赶应该非零");
    assert!(checkpoint_at_file_count_145.inode_catch_up_bytes > 0, "inode 根追赶应该非零");

    // K10：P=1、N=16 一格 wal_full 中间版 60、分配记录树 [5,1]；乙-M 中间版 45、[4,1]（B11）。
    let checkpoint_full_released_and_backlogged = solve_write_ahead_log_checkpoint(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert_eq!(checkpoint_full_released_and_backlogged.intermediate_version_total, 60.0, "wal_full 中间版：数据 15+extent 15+inode 叶 15+inode 根 15");
    assert_eq!(checkpoint_full_released_and_backlogged.allocation_layers, vec![5, 1], "wal_full-K10 分配记录树 [5,1]");
    let checkpoint_leaf_released_and_backlogged = solve_write_ahead_log_checkpoint(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogLeaf, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    assert_eq!(checkpoint_leaf_released_and_backlogged.intermediate_version_total, 45.0, "乙-M 中间版：数据 15+extent 15+inode 叶 15（无 inode 根）");
    assert_eq!(checkpoint_leaf_released_and_backlogged.allocation_layers, vec![4, 1], "乙-M-K10 分配记录树 [4,1]");
}

/// B19：N_b=1（N=1、concurrent_fsync_count=1）的格上，K9 与 K10 逐字段相同——中间版恒 0。
#[test]
fn batch_count_equal_one_cells_have_zero_intermediate_versions_and_not_tracked_equals_released_and_backlogged() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
        let not_tracked = solve_write_ahead_log_checkpoint(shape, arm, 1, IntermediateVersionPolicy::NotTracked);
        let released_and_backlogged = solve_write_ahead_log_checkpoint(shape, arm, 1, IntermediateVersionPolicy::ReleasedAndBacklogged);
        assert_eq!(released_and_backlogged.intermediate_version_total, 0.0, "arm={arm:?}");
        assert_eq!(not_tracked.total_bytes(), released_and_backlogged.total_bytes(), "arm={arm:?}：N=1 时 K9 与 K10 应该逐字节相同");
        assert_eq!(not_tracked.write_calls, released_and_backlogged.write_calls, "arm={arm:?}");
    }
    // wal_full − 甲 = 8192（一条记录的差，第一节「读法写死」N_b=1 那一条）。
    let jia = solve_jia_publish(shape).total_bytes();
    let write_ahead_log_full_checkpoint = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, 1, IntermediateVersionPolicy::NotTracked);
    let write_ahead_log_full_amortized = amortize(solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes(), write_ahead_log_full_checkpoint.total_bytes(), 1);
    assert!((write_ahead_log_full_amortized - jia as f64 - 8192.0).abs() < 1e-6, "write_ahead_log_full_amortized={write_ahead_log_full_amortized} jia={jia}");
}

/// V2（作废条款）：任一格「乙-M ≤ wal_full ≤ 甲」（fsync 行字节）都不许违反。
#[test]
fn arm_containment_holds_on_the_grid() {
    for file_count in [1u64, 145, 10_000, 1_000_000] {
        for family in [Family::OneDataUnitPerFile, Family::EightAdjacentDataUnitsPerFsync] {
            for placement in [Placement::Sequential, Placement::Random] {
                for policy in [PositionPolicy::Near, PositionPolicy::Far, PositionPolicy::Balanced] {
                    let shape = shape_at(file_count, family, placement, policy);
                    let jia = solve_jia_publish(shape).total_bytes();
                    let write_ahead_log_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
                    let write_ahead_log_leaf = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
                    assert!(
                        write_ahead_log_leaf <= write_ahead_log_full && write_ahead_log_full <= jia,
                        "file_count={file_count} family={family:?} placement={placement:?} policy={policy:?}: 乙-M={write_ahead_log_leaf} wal_full={write_ahead_log_full} 甲={jia}"
                    );
                }
            }
        }
    }
}

/// V2：K10 的摊销字节 / 写请求不小于 K9（R7(b) 只加，不减）。
#[test]
fn released_and_backlogged_never_amortizes_cheaper_than_not_tracked() {
    for file_count in [1u64, 145, 10_000] {
        for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
            for placement in [Placement::Sequential, Placement::Random] {
                let shape = shape_at(file_count, Family::OneDataUnitPerFile, placement, PositionPolicy::Balanced);
                let fsync = solve_write_ahead_log_fsync(shape, arm);
                let not_tracked = solve_write_ahead_log_checkpoint(shape, arm, 16, IntermediateVersionPolicy::NotTracked);
                let released_and_backlogged = solve_write_ahead_log_checkpoint(shape, arm, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
                let amortized_not_tracked = amortize(fsync.total_bytes(), not_tracked.total_bytes(), 16);
                let amortized_released_and_backlogged = amortize(fsync.total_bytes(), released_and_backlogged.total_bytes(), 16);
                assert!(amortized_released_and_backlogged >= amortized_not_tracked - 1e-6, "file_count={file_count} arm={arm:?} placement={placement:?}: not_tracked={amortized_not_tracked} not_tracked'={amortized_released_and_backlogged}");
                assert!(released_and_backlogged.write_calls >= not_tracked.write_calls, "file_count={file_count} arm={arm:?} placement={placement:?}");
            }
        }
    }
}

/// Q6.9 支撑量：在飞记录峰值 `R = N × r_f + r_c`，P=1、N=16 一格与手算一致（M20 的反例：不计 checkpoint 那条会得 16）。
#[test]
fn in_flight_records_and_ring_bytes_match_the_hand_calculation() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
        let fsync = solve_write_ahead_log_fsync(shape, arm);
        let checkpoint = solve_write_ahead_log_checkpoint(shape, arm, 16, IntermediateVersionPolicy::NotTracked);
        let peak_in_flight_records = 16 * fsync.record_count + checkpoint.record_count;
        assert_eq!(peak_in_flight_records, 17, "arm={arm:?}");
        let ring_bytes_needed = RING_SAFETY_FACTOR_MAIN * RECORD_BYTES * peak_in_flight_records;
        assert_eq!(ring_bytes_needed, 3 * 4096 * 17);
        let default_ring_in_flight_limit = RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN;
        assert_eq!(default_ring_in_flight_limit, 65_536, "B9：默认环的在飞上限");
        assert!(peak_in_flight_records <= default_ring_in_flight_limit, "arm={arm:?} 默认环应该装得下 P=1 这一格");
    }
}
