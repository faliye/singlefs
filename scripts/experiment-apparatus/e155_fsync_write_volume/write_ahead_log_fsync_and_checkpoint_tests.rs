use super::*;

fn shape_at(file_count: u64, family: Family, placement: Placement, policy: PositionPolicy) -> PoolShape {
    PoolShape { file_count, family, placement, policy, geometry: Geometry::main() }
}

/// B4（第七节 7.2）：write_ahead_log_full，P=1、F1、seq、主几何。
#[test]
fn write_ahead_log_full_at_file_count_1_matches_the_anchor() {
    let fsync = solve_write_ahead_log_fsync(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull);
    assert_eq!(fsync.total_bytes(), 204_800);
    assert_eq!(fsync.write_calls, 10);
    assert_eq!(fsync.barriers, 4);

    let checkpoint = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogFull, 16);
    assert_eq!(checkpoint.total_bytes(), 147_968);
    assert_eq!(checkpoint.write_calls, 13);
    assert_eq!(checkpoint.fua_count, 1);

    for interval in [1u64, 16] {
        let amortized = amortize(fsync.total_bytes(), checkpoint.total_bytes(), interval);
        let expected = if interval == 1 { 352_768.0 } else { 214_048.0 };
        assert!((amortized - expected).abs() < 1e-6, "interval={interval} amortized={amortized}");
    }
}

/// B5（第七节 7.2）：乙，P=1、F1、seq、主几何。
#[test]
fn write_ahead_log_leaf_at_file_count_1_matches_the_anchor() {
    let fsync = solve_write_ahead_log_fsync(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf);
    assert_eq!(fsync.total_bytes(), 172_032);
    assert_eq!(fsync.write_calls, 8);
    assert_eq!(fsync.barriers, 4);

    let checkpoint = solve_write_ahead_log_checkpoint(shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced), WriteAheadLogArm::WriteAheadLogLeaf, 16);
    assert_eq!(checkpoint.total_bytes(), 180_736);
    assert_eq!(checkpoint.write_calls, 15);

    for interval in [1u64, 16] {
        let amortized = amortize(fsync.total_bytes(), checkpoint.total_bytes(), interval);
        let expected = if interval == 1 { 352_768.0 } else { 183_328.0 };
        assert!((amortized - expected).abs() < 1e-6, "interval={interval} amortized={amortized}");
    }
}

/// B6（第七节 7.2）：三臂在锚点上的差——甲 − write_ahead_log_full = 139776，write_ahead_log_full − 乙 = 32768。
#[test]
fn arm_differences_at_the_anchor_match() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let jia = solve_jia_publish(shape).total_bytes();
    let write_ahead_log_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let write_ahead_log_leaf = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    assert_eq!(jia - write_ahead_log_full, 139_776);
    assert_eq!(write_ahead_log_full - write_ahead_log_leaf, 32_768);
}

/// B7（第七节 7.2）：P=145、F1、seq，write_ahead_log_full 与乙的 fsync 行。
#[test]
fn write_ahead_log_arms_at_file_count_145_match_the_anchor() {
    let shape = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    assert_eq!(solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes(), 237_568);
    assert_eq!(solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes(), 172_032);
}

/// 阳性对照（5.5）：三条臂各自的格与「必须出现的差」逐条核对，任一条不过整轮作废（V1）。
#[test]
fn positive_controls_show_the_prescribed_difference_for_every_arm() {
    let shape_at_file_count_1 = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let jia_fsync = solve_jia_publish(shape_at_file_count_1).total_bytes();
    let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(shape_at_file_count_1, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    assert_eq!(jia_fsync - write_ahead_log_full_fsync, 139_776, "甲：四样固定点 × 2 盘 + 根槽 + 系统配置槽 × 2");
    assert_eq!(write_ahead_log_full_fsync - write_ahead_log_leaf_fsync, 32_768, "write_ahead_log_full：inode 根 × 2 盘");

    let jia_root_slots_in_16 = 16u64; // 甲每次 fsync 都写根槽。
    assert_eq!(jia_root_slots_in_16, 16, "16 次 fsync 里甲写根槽 16 次");
    let write_ahead_log_full_root_slots_in_16 = 1u64; // write_ahead_log_full 只在 checkpoint 写根槽。
    assert_eq!(write_ahead_log_full_root_slots_in_16, 1, "16 次 fsync 里 write_ahead_log_full 写根槽 1 次（那次 checkpoint）");

    let shape_at_file_count_145 = shape_at(145, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    let write_ahead_log_leaf_at_file_count_145 = solve_write_ahead_log_fsync(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    let write_ahead_log_full_at_file_count_145 = solve_write_ahead_log_fsync(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    assert_eq!(write_ahead_log_leaf_at_file_count_145, 172_032, "乙 fsync 行不含 extent 与 inode 两个根");
    assert_eq!(write_ahead_log_full_at_file_count_145, 237_568);
    let checkpoint_at_file_count_145 = solve_write_ahead_log_checkpoint(shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf, 16);
    // 这 16 次 fsync 的 checkpoint 里乙写 extent 根、inode 根各 1 次：P=145 时 extent 树高 2，
    // 追赶的祖先恰是 1 个 extent 根 + 1 个 inode 根。
    assert!(checkpoint_at_file_count_145.extent_catch_up_bytes > 0, "extent 根追赶应该非零");
    assert!(checkpoint_at_file_count_145.inode_catch_up_bytes > 0, "inode 根追赶应该非零");
}

/// V2（作废条款）：任一格「乙 ≤ write_ahead_log_full ≤ 甲」（fsync 行字节）都不许违反。
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
                        "file_count={file_count} family={family:?} placement={placement:?} policy={policy:?}: 乙={write_ahead_log_leaf} write_ahead_log_full={write_ahead_log_full} 甲={jia}"
                    );
                }
            }
        }
    }
}

/// Q1.5：在飞记录峰值 `R = N × r_f + r_c`，要的环 = `3 × 4096 × R` 字节（F=3）；
/// 默认环装得下 ⟺ R ≤ 65536。P=1、N=16 一格算出的 R 与手算一致。
#[test]
fn in_flight_records_and_ring_bytes_match_the_hand_calculation() {
    let shape = shape_at(1, Family::OneDataUnitPerFile, Placement::Sequential, PositionPolicy::Balanced);
    for arm in [WriteAheadLogArm::WriteAheadLogFull, WriteAheadLogArm::WriteAheadLogLeaf] {
        let fsync = solve_write_ahead_log_fsync(shape, arm);
        let checkpoint = solve_write_ahead_log_checkpoint(shape, arm, 16);
        let peak_in_flight_records = 16 * fsync.record_count + checkpoint.record_count;
        // P=1 一格每次 fsync 与 checkpoint 都恰好 1 条记录：R = 16*1+1 = 17（M20 的反例：不计 checkpoint 那条会得 16）。
        assert_eq!(peak_in_flight_records, 17, "arm={arm:?}");
        let ring_bytes_needed = RING_SAFETY_FACTOR_MAIN * RECORD_BYTES * peak_in_flight_records;
        assert_eq!(ring_bytes_needed, 3 * 4096 * 17);
        let default_ring_in_flight_limit = RING_DEFAULT_BYTES / RECORD_BYTES / RING_SAFETY_FACTOR_MAIN;
        assert_eq!(default_ring_in_flight_limit, 65_536, "B9：默认环的在飞上限");
        assert!(peak_in_flight_records <= default_ring_in_flight_limit, "arm={arm:?} 默认环应该装得下 P=1 这一格");
    }
}
