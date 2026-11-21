//! E155 第二次跑：岔路单第 6 行 —— 三臂（甲 / wal_full-K10 / 乙-M-K10）在同一批取样点上的
//! fsync 行、摊销行、之比与 K10 相对 K9 的增量。确定性计数模型。
//!
//! 判据、失败条款、变异表的权威登记在 `research/prompts/e155-r2-prereg.md`（第五节 5.1 R1–R8、
//! 第六节 Q6.x、第七节 B1/B2/B4/B5/B7/B10/B11/B15–B19）。第一次跑的登记 `research/prompts/
//! e155-preregistration.md` 与它的二进制 `e155_fsync_write_volume.rs`（本文件从它拷出再改，
//! 第三节 3.2）、产物都不动——本文件是另一个二进制，`replay.sh` 里 E155 那一行仍指向第一次的产物。
//!
//! 本文件从第一次的二进制原样带过来的部分：常量、K1–K4 容量与树高、5.3 组公式的基础三步
//! （`continuous_groups_touch` / `scattered_group_touch` / `combine_touches`）、第 2–5 行的反事实
//! 上界函数与它们的单测（`row_2_to_4_counterfactual_tests`、`row_5_growth_tests`）——这些函数与单测
//! 不属于这一次的判据，留着只是为了让第一次的变异表（15 条）能在这个新二进制上照跑一遍（登记第九节）。
//! `main()` 不再输出第 1–5 行（那些数已经在第一次的产物里），只输出第 6 行主表、阳性对照、Q6.1–Q6.9、
//! 8.2、Q6.11。
//!
//! 第一段交付（登记 5.5）：另起二进制、停机条款 S1–S3、R6、R7、第 6 行主表五列、
//! 三条臂的阳性对照、B1/B2/B4/B5/B7/B10/B11/B15–B19、Q6.1–Q6.9、第 6 行判定格上的 8.2 与判别力自证、
//! 变异 M1–M8/M14/M15/M18/M19 与第一次变异表；另加主 agent 点名要跑的 Q6.11（第三节 3.2 四处
//! 改动前后甲的差，登记原写「够判后不跑」，这一次改成要跑）。
//!
//! 第二段交付（R8 组提交、第 7 行）：R8、第 7 行主表（`row7_grid`，1800 格）、B12–B14、
//! 阳性对照（`positive_control_row7`）、变异 M9–M13/M16/M17（表里的 R2_M9–R2_M17）、
//! 翻面点 Q7.3/Q7.3b（`row7_fold_point`）、反向取样点「N 按批计」（`geometry_sensitivity_sample_row7`
//! 的 `interval_counted_by_batch`）。**还没做**：第 7 行判定格上 φ=0.5/K11/g=0/pbs=4096/K12 五个
//! 反向取样点、「不共享取随机」（F9，只留了一条 `unshared_random_not_modeled_this_pass` 占位说明）、
//! 沿 k 轴的判别力自证——这些交主 agent 定要不要续（见岔路表）。

use e7_index_bench::Emitter;

#[cfg(test)]
mod capacity_tests;
#[cfg(test)]
mod convergence_and_saturation_status_tests;
mod counterfactual_shares_and_thresholds;
#[cfg(test)]
mod group_formula_tests;
mod group_touches_and_tree_layers;
#[cfg(test)]
mod jia_anchor_tests;
// ============================================================
// 五之二、legacy：第一次装置的原样实现（3.2 四处未改），只为 Q6.11 对比用，不进任何判据
// ============================================================
mod legacy;
mod pool_shape_and_fixed_point_solvers;
#[cfg(test)]
mod row7_group_commit_tests;
#[cfg(test)]
mod row_2_to_4_counterfactual_tests;
#[cfg(test)]
mod row_5_growth_tests;
#[cfg(test)]
mod write_ahead_log_anchor_tests;
mod write_ahead_log_solvers;

use self::counterfactual_shares_and_thresholds::*;
use self::group_touches_and_tree_layers::*;
use self::pool_shape_and_fixed_point_solvers::*;
use self::write_ahead_log_solvers::*;

// ============================================================
// 一、常量（与第一次逐字相同，手抄自 `crates/singlefs-format/src/lib.rs`；第七节 B1 锚点对拍）
// ============================================================

/// 码 2 节点占盘宽度（含头）。
const NODE_BYTES: u64 = 16384;
/// 数据单元与码 3 容器占盘宽度（含头）。
const UNIT_BYTES: u64 = 32768;
/// journal 记录固定宽度。
const RECORD_BYTES: u64 = 4096;
/// 系统配置槽宽度。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 记录头宽度（`journal_named_items_per_record` 用它反推容量）。
const RECORD_HEADER_BYTES: u64 = 311;
/// 一条点名项的宽度。
const NAMED_ITEM_BYTES: u64 = 56;
/// 两块盘（`D`）。
const DEVICE_COUNT: u64 = 2;
/// 主几何下 journal 环安全系数 `F`（D22 已定项 2）。
const RING_SAFETY_FACTOR_MAIN: u64 = 3;
/// journal 环默认容量。
const RING_DEFAULT_BYTES: u64 = 768 * 1024 * 1024;
/// 主几何 `physical_block_size`。
const PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 512;
/// 反向取样点 `physical_block_size`（8.2）。
const REVERSE_PHYSICAL_BLOCK_SIZE_BYTES: u64 = 4096;

/// 码 2 节点头宽公式的常量部分：`86 + 2×key宽 + 29`（`anchors_r2*.py` 的 `hdr2`）。
const NODE_HEADER_BASE_BYTES: u64 = 86 + 29;
/// 节点指针宽（D19 已定项 8）。
const NODE_POINTER_BYTES: u64 = 86;
/// K11 反向取样点：内部条目再加身份引用 26 字节。
const IDENTITY_REFERENCE_EXTRA_BYTES: u64 = 26;

const EXTENT_KEY_BYTES: u64 = 24;
const EXTENT_LEAF_ENTRY_BYTES: u64 = 112;
const INODE_KEY_BYTES: u64 = 8;
const INODE_INTERNAL_ENTRY_BYTES: u64 = 120;
const ALLOCATION_KEY_BYTES: u64 = 10;
const ALLOCATION_LEAF_ENTRY_BYTES: u64 = 20;
const ACCOUNTING_KEY_BYTES: u64 = 22;
const ACCOUNTING_LEAF_ENTRY_BYTES: u64 = 34;
const MAPPING_KEY_BYTES: u64 = 27;
const MAPPING_LEAF_ENTRY_BYTES: u64 = 55;
const TREE_TABLE_KEY_BYTES: u64 = 8;
const TREE_TABLE_ENTRY_BYTES: u64 = 200;
/// 码 3 容器头宽（`UNIT_BYTES - 136`，独立于 `hdr2` 公式）。
const INODE_CONTAINER_HEADER_BYTES: u64 = 136;
const INODE_CONTAINER_ENTRY_BYTES: u64 = 140;

/// 记账池级（不带设备维）行数：inode 号水位、待删占用、已承诺预留。
const ACCOUNTING_POOL_WIDE_ENTRIES: u64 = 3;
/// 记账每块盘各一行的行数。
const ACCOUNTING_ENTRIES_PER_DEVICE: u64 = 6;
/// 树表条目数（主几何，7 棵）。
const TREE_TABLE_ENTRY_COUNT: u64 = 7;
/// 树表改根时相邻改动的条目数（extent、inode、分配记录、记账四棵树的根）。
const TREE_TABLE_TOUCHED_ROOTS: u64 = 4;
/// 一次发布点名项超过 67 时怎么切（K7 主口径）。
const NAMED_ITEMS_PER_RECORD_MAIN: u64 = (RECORD_BYTES - RECORD_HEADER_BYTES) / NAMED_ITEM_BYTES;

/// 码 2 节点头宽：`86 + 2×key宽 + 29`。
const fn node_header_bytes(key_bytes: u64) -> u64 {
    NODE_HEADER_BASE_BYTES + 2 * key_bytes
}

/// 一个节点能装的条目数：`(节点宽 − 头宽) / 条目宽`。
const fn node_capacity(key_bytes: u64, entry_bytes: u64) -> u64 {
    (NODE_BYTES - node_header_bytes(key_bytes)) / entry_bytes
}

/// 内部节点条目宽：分隔 key + 子指针（K3 主几何）；K11 反向取样点再加身份引用 26 字节。
const fn internal_entry_bytes(key_bytes: u64, identity_reference: bool) -> u64 {
    key_bytes + NODE_POINTER_BYTES + if identity_reference { IDENTITY_REFERENCE_EXTRA_BYTES } else { 0 }
}

fn inode_container_capacity() -> u64 {
    (UNIT_BYTES - INODE_CONTAINER_HEADER_BYTES) / INODE_CONTAINER_ENTRY_BYTES
}

// ============================================================
// 七、main：第 6 行主表（stage 1）、阳性对照、Q6.1–Q6.9、Q6.11、8.2、判别力自证
// ============================================================

fn main() {
    let mut emitter = Emitter::new();
    println!("{}", emitter.emit_raw(&format!("name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} system_configuration_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} ring_default_bytes={RING_DEFAULT_BYTES} ring_default_in_flight_limit={} model=counting", default_ring_in_flight_limit())));
    println!(
        "{}",
        emitter.emit_raw(
            "name=scope stage=2 covers=第6行主表五列(甲/wal_full-K9'/乙-M-K9'/wal_full-K9/乙-M-K9),阳性对照,R1,R2,R3,R4,R6,R7,B1,B2,B4,B5,B7,B10,B11,B15-B19,Q6.1-Q6.9,Q6.11,8.2(第6行判定格),判别力自证(轴n),R8,第7行主表row7_grid(1800格),B12-B14,阳性对照row7,Q7.1-Q7.6,row7_fold_point,8.2(第7行,仅N按批计) missing=8.2(第7行phi0.5/K3'/g0/pbs4096/K9'-b五个反向取样点),F9不共享取随机(仅占位说明未建开关),判别力自证(轴k)"
        )
    );

    // B1：容量一览（与第一次相同）。
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=b1_capacities extent_leaf={} extent_internal={} inode_container={} inode_internal={} alloc_leaf={} alloc_internal={} accounting_leaf={} accounting_internal={} mapping_leaf={} mapping_internal={} tree_table={} named_items_per_record={}",
            node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES),
            node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false)),
            inode_container_capacity(),
            node_capacity(INODE_KEY_BYTES, INODE_INTERNAL_ENTRY_BYTES),
            node_capacity(ALLOCATION_KEY_BYTES, ALLOCATION_LEAF_ENTRY_BYTES),
            node_capacity(ALLOCATION_KEY_BYTES, internal_entry_bytes(ALLOCATION_KEY_BYTES, false)),
            node_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_LEAF_ENTRY_BYTES),
            node_capacity(ACCOUNTING_KEY_BYTES, internal_entry_bytes(ACCOUNTING_KEY_BYTES, false)),
            node_capacity(MAPPING_KEY_BYTES, MAPPING_LEAF_ENTRY_BYTES),
            node_capacity(MAPPING_KEY_BYTES, internal_entry_bytes(MAPPING_KEY_BYTES, false)),
            node_capacity(TREE_TABLE_KEY_BYTES, TREE_TABLE_ENTRY_BYTES),
            NAMED_ITEMS_PER_RECORD_MAIN,
        ))
    );

    // 阳性对照（5.5）：三条臂各自的格与「必须出现的差」，K9 与 K10 都报。
    let control_shape = PoolShape { file_count: 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
    let jia_fsync = solve_jia_publish(control_shape).total_bytes();
    let control_write_ahead_log_full_fsync = solve_write_ahead_log_fsync(control_shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let control_write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(control_shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    let checkpoint_full_released_and_backlogged = solve_write_ahead_log_checkpoint(control_shape, WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    let checkpoint_leaf_released_and_backlogged = solve_write_ahead_log_checkpoint(control_shape, WriteAheadLogArm::WriteAheadLogLeaf, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control cell=p1_f1_seq jia_fsync_bytes={jia_fsync} write_ahead_log_full_fsync_bytes={control_write_ahead_log_full_fsync} write_ahead_log_leaf_fsync_bytes={control_write_ahead_log_leaf_fsync} jia_minus_write_ahead_log_full={} write_ahead_log_full_minus_write_ahead_log_leaf={} jia_root_slots_per_16_fsync=16 write_ahead_log_full_root_slots_per_16_fsync=1 write_ahead_log_full_released_and_backlogged_intermediate_total={} write_ahead_log_full_released_and_backlogged_allocation_layers={:?} write_ahead_log_leaf_released_and_backlogged_intermediate_total={} write_ahead_log_leaf_released_and_backlogged_allocation_layers={:?}",
            jia_fsync - control_write_ahead_log_full_fsync,
            control_write_ahead_log_full_fsync - control_write_ahead_log_leaf_fsync,
            checkpoint_full_released_and_backlogged.intermediate_version_total,
            checkpoint_full_released_and_backlogged.allocation_layers,
            checkpoint_leaf_released_and_backlogged.intermediate_version_total,
            checkpoint_leaf_released_and_backlogged.allocation_layers,
        ))
    );

    // 第 6 行主表（登记 5.4）：P × 族 × 落点 × N × 三种位置策略，五列各报各的。
    let file_counts: [u64; 6] = [1, 100, 10_000, 1_000_000, 100_000_000, 145];
    let families = [Family::OneDataUnitPerFile, Family::EightAdjacentDataUnitsPerFsync];
    let placements = [Placement::Sequential, Placement::Random];
    let intervals: [u64; 4] = [1, 16, 256, 4096];
    let policies = [PositionPolicy::Near, PositionPolicy::Balanced, PositionPolicy::Far];

    for &file_count in &file_counts {
        for family in families {
            for placement in placements {
                for policy in policies {
                    let shape = PoolShape { file_count, family, placement, policy, geometry: Geometry::main() };
                    let jia = solve_jia_publish(shape);
                    let write_ahead_log_full_fsync = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull);
                    let write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf);
                    for &interval in &intervals {
                        let cp_full_not_tracked = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, interval, IntermediateVersionPolicy::NotTracked);
                        let cp_leaf_not_tracked = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, interval, IntermediateVersionPolicy::NotTracked);
                        let cp_full_released_and_backlogged = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
                        let cp_leaf_released_and_backlogged = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, interval, IntermediateVersionPolicy::ReleasedAndBacklogged);

                        let jia_bytes = jia.total_bytes();
                        let amortized_full_not_tracked = amortize(write_ahead_log_full_fsync.total_bytes(), cp_full_not_tracked.total_bytes(), interval);
                        let amortized_leaf_not_tracked = amortize(write_ahead_log_leaf_fsync.total_bytes(), cp_leaf_not_tracked.total_bytes(), interval);
                        let amortized_full_released_and_backlogged = amortize(write_ahead_log_full_fsync.total_bytes(), cp_full_released_and_backlogged.total_bytes(), interval);
                        let amortized_leaf_released_and_backlogged = amortize(write_ahead_log_leaf_fsync.total_bytes(), cp_leaf_released_and_backlogged.total_bytes(), interval);

                        // Q6.3：三条臂之比（fsync 行 / 摊销行都报，用 K10 的摊销值）。
                        let ratio_jia_over_full = jia_bytes as f64 / amortized_full_released_and_backlogged;
                        let ratio_jia_over_leaf = jia_bytes as f64 / amortized_leaf_released_and_backlogged;
                        let ratio_full_over_leaf = amortized_full_released_and_backlogged / amortized_leaf_released_and_backlogged;

                        // Q6.5 / Q6.5c：`s` = 1 − 摊销 ÷ 甲摊销（字节与写请求）。
                        let saved_share_write_ahead_log_full_bytes = 1.0 - amortized_full_released_and_backlogged / jia_bytes as f64;
                        let saved_share_write_ahead_log_leaf_bytes = 1.0 - amortized_leaf_released_and_backlogged / jia_bytes as f64;

                        // Q6.6：K10 比 K9 多写多少。
                        let delta_full_bytes = amortized_full_released_and_backlogged - amortized_full_not_tracked;
                        let delta_leaf_bytes = amortized_leaf_released_and_backlogged - amortized_leaf_not_tracked;

                        // F2（第十节）：不动点第一半（撞迭代上限未收敛）与第二半（某棵树第 0 层节点数 ≥ 64
                        // 且脏节点占比 ≥ 50%）——甲与四条 WAL checkpoint 核心（K9 / K10 × wal_full / 乙-M）分开判。
                        let (jia_unconverged, jia_saturated_trees) = jia_convergence_and_saturation_status(&jia);
                        let (full_not_tracked_unconverged, full_not_tracked_saturated_trees) = checkpoint_convergence_and_saturation_status(&cp_full_not_tracked);
                        let (full_released_and_backlogged_unconverged, full_released_and_backlogged_saturated_trees) = checkpoint_convergence_and_saturation_status(&cp_full_released_and_backlogged);
                        let (leaf_not_tracked_unconverged, leaf_not_tracked_saturated_trees) = checkpoint_convergence_and_saturation_status(&cp_leaf_not_tracked);
                        let (leaf_released_and_backlogged_unconverged, leaf_released_and_backlogged_saturated_trees) = checkpoint_convergence_and_saturation_status(&cp_leaf_released_and_backlogged);

                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=row6_grid p={file_count} family={} placement={} policy={} n={interval} \
                                 jia_fsync_bytes={jia_bytes} jia_calls={} jia_barriers={} jia_fua={} jia_amortized_bytes={jia_bytes} \
                                 wal_full_not_tracked_fsync_bytes={} wal_full_not_tracked_checkpoint_bytes={} wal_full_not_tracked_amortized_bytes={amortized_full_not_tracked} wal_full_not_tracked_alloc_layers={:?} \
                                 wal_full_released_and_backlogged_fsync_bytes={} wal_full_released_and_backlogged_checkpoint_bytes={} wal_full_released_and_backlogged_amortized_bytes={amortized_full_released_and_backlogged} wal_full_released_and_backlogged_alloc_layers={:?} wal_full_released_and_backlogged_intermediate_total={} \
                                 wal_leaf_not_tracked_fsync_bytes={} wal_leaf_not_tracked_checkpoint_bytes={} wal_leaf_not_tracked_amortized_bytes={amortized_leaf_not_tracked} \
                                 wal_leaf_released_and_backlogged_fsync_bytes={} wal_leaf_released_and_backlogged_checkpoint_bytes={} wal_leaf_released_and_backlogged_amortized_bytes={amortized_leaf_released_and_backlogged} wal_leaf_released_and_backlogged_alloc_layers={:?} wal_leaf_released_and_backlogged_intermediate_total={} \
                                 ratio_jia_over_wal_full={ratio_jia_over_full} ratio_jia_over_wal_leaf={ratio_jia_over_leaf} ratio_wal_full_over_wal_leaf={ratio_full_over_leaf} \
                                 saved_share_write_ahead_log_full_bytes={saved_share_write_ahead_log_full_bytes} saved_share_write_ahead_log_leaf_bytes={saved_share_write_ahead_log_leaf_bytes} \
                                 delta_released_and_backlogged_minus_not_tracked_wal_full_bytes={delta_full_bytes} delta_released_and_backlogged_minus_not_tracked_wal_leaf_bytes={delta_leaf_bytes} \
                                 jia_write_calls_per_fsync={} wal_full_released_and_backlogged_write_calls_per_fsync={} wal_leaf_released_and_backlogged_write_calls_per_fsync={} \
                                 extent_layers={} inode_layers={} allocation_layers={} mapping_layers={} fixed_point_iterations={} \
                                 jia_unconverged={jia_unconverged} jia_saturated_trees={jia_saturated_trees:?} \
                                 wal_full_not_tracked_unconverged={full_not_tracked_unconverged} wal_full_not_tracked_saturated_trees={full_not_tracked_saturated_trees:?} \
                                 wal_full_released_and_backlogged_unconverged={full_released_and_backlogged_unconverged} wal_full_released_and_backlogged_saturated_trees={full_released_and_backlogged_saturated_trees:?} \
                                 wal_leaf_not_tracked_unconverged={leaf_not_tracked_unconverged} wal_leaf_not_tracked_saturated_trees={leaf_not_tracked_saturated_trees:?} \
                                 wal_leaf_released_and_backlogged_unconverged={leaf_released_and_backlogged_unconverged} wal_leaf_released_and_backlogged_saturated_trees={leaf_released_and_backlogged_saturated_trees:?}",
                                family.tag(),
                                placement.tag(),
                                policy.tag(),
                                jia.write_calls,
                                jia.barriers,
                                jia.fua_count,
                                write_ahead_log_full_fsync.total_bytes(),
                                cp_full_not_tracked.total_bytes(),
                                cp_full_not_tracked.allocation_layers,
                                write_ahead_log_full_fsync.total_bytes(),
                                cp_full_released_and_backlogged.total_bytes(),
                                cp_full_released_and_backlogged.allocation_layers,
                                cp_full_released_and_backlogged.intermediate_version_total,
                                write_ahead_log_leaf_fsync.total_bytes(),
                                cp_leaf_not_tracked.total_bytes(),
                                write_ahead_log_leaf_fsync.total_bytes(),
                                cp_leaf_released_and_backlogged.total_bytes(),
                                cp_leaf_released_and_backlogged.allocation_layers,
                                cp_leaf_released_and_backlogged.intermediate_version_total,
                                jia.write_calls,
                                write_ahead_log_full_fsync.write_calls,
                                write_ahead_log_leaf_fsync.write_calls,
                                jia.extent_layers.len(),
                                jia.inode_layers.len(),
                                jia.allocation_layers.len(),
                                jia.mapping_layers.len(),
                                jia.fixed_point_iterations,
                            ))
                        );
                    }
                }
            }
        }
    }

    // Q6.11：第三节 3.2 四处改动前后甲的差，覆盖第 6 行主表同一批格（这一次改成要跑）；
    // 每格另判 F2 两半（legacy、fixed 分开判，F2 触发的格按登记「结论收窄」报，不当普通值用）。
    for &file_count in &file_counts {
        for family in families {
            for placement in placements {
                for policy in policies {
                    let shape = PoolShape { file_count, family, placement, policy, geometry: Geometry::main() };
                    let fixed_outcome = solve_jia_publish(shape);
                    let legacy_outcome = legacy::legacy_solve_jia_publish(shape);
                    let fixed = fixed_outcome.total_bytes();
                    let legacy = legacy_outcome.total_bytes();
                    let delta = fixed as i128 - legacy as i128;
                    let (fixed_unconverged, fixed_saturated_trees) = jia_convergence_and_saturation_status(&fixed_outcome);
                    let (legacy_unconverged, legacy_saturated_trees) = jia_convergence_and_saturation_status(&legacy_outcome);
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=q6_11_legacy_vs_fixed p={file_count} family={} placement={} policy={} fixed_bytes={fixed} legacy_bytes={legacy} delta_bytes={delta} differs={} \
                             fixed_unconverged={fixed_unconverged} fixed_saturated_trees={fixed_saturated_trees:?} legacy_unconverged={legacy_unconverged} legacy_saturated_trees={legacy_saturated_trees:?}",
                            family.tag(),
                            placement.tag(),
                            policy.tag(),
                            fixed != legacy,
                        ))
                    );
                }
            }
        }
    }

    // 8.2 几何敏感性：第 6 行判定格上逐个换到反向取样点（代表格 P=100000, F1, seq, Lbalanced, N=16）。
    {
        let representative_file_count = 100_000u64;
        let base_shape = PoolShape { file_count: representative_file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let base_jia = solve_jia_publish(base_shape).total_bytes();
        let base_write_ahead_log_full = solve_write_ahead_log_fsync(base_shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let base_write_ahead_log_leaf = solve_write_ahead_log_fsync(base_shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
        let base_checkpoint_full = solve_write_ahead_log_checkpoint(base_shape, WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let base_amortized_full = amortize(base_write_ahead_log_full, base_checkpoint_full.total_bytes(), 16);
        let base_saved_share_write_ahead_log_full_value = 1.0 - base_amortized_full / base_jia as f64;

        let half_fill_shape = PoolShape { geometry: Geometry { fill_ratio: FillRatio::Half, ..Geometry::main() }, ..base_shape };
        let reverse_identity_reference_shape = PoolShape { geometry: Geometry { identity_reference: true, ..Geometry::main() }, ..base_shape };
        let no_backlog_shape = PoolShape { geometry: Geometry { backlog_generations: 0, ..Geometry::main() }, ..base_shape };
        let reverse_pbs_shape = PoolShape { geometry: Geometry { physical_block_size_bytes: REVERSE_PHYSICAL_BLOCK_SIZE_BYTES, ..Geometry::main() }, ..base_shape };

        for (label, shape) in [("phi_0.5", half_fill_shape), ("k11", reverse_identity_reference_shape), ("g_0", no_backlog_shape), ("pbs_4096", reverse_pbs_shape)] {
            let jia = solve_jia_publish(shape).total_bytes();
            let write_ahead_log_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
            let write_ahead_log_leaf = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
            let checkpoint_full = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, 16, IntermediateVersionPolicy::ReleasedAndBacklogged);
            let amortized_full = amortize(write_ahead_log_full, checkpoint_full.total_bytes(), 16);
            let saved_share_write_ahead_log_full_value = 1.0 - amortized_full / jia as f64;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=geometry_sensitivity_sample_row6 point={label} p={representative_file_count} base_jia={base_jia} reverse_jia={jia} base_wal_full_fsync={base_write_ahead_log_full} reverse_wal_full_fsync={write_ahead_log_full} base_wal_leaf_fsync={base_write_ahead_log_leaf} reverse_wal_leaf_fsync={write_ahead_log_leaf} base_saved_share_wal_full={base_saved_share_write_ahead_log_full_value} reverse_saved_share_wal_full={saved_share_write_ahead_log_full_value} saved_share_sign_changes={}",
                    (base_saved_share_write_ahead_log_full_value > 0.0) != (saved_share_write_ahead_log_full_value > 0.0),
                ))
            );
        }

        // K12（R7 反向取样点）：中间版不进积压——只影响 s，不影响 jia/fsync。
        let intermediate_version_alternate_reading_note = "K12（中间版不进积压）与 N 按批计（第 7 行才有 concurrent_fsync_count 轴，这一次只报占位）在这一版未建独立开关，交主 agent";
        println!("{}", emitter.emit_raw(&format!("name=geometry_sensitivity_sample_row6 point=k12_not_modeled_this_pass p={representative_file_count} note={intermediate_version_alternate_reading_note:?}")));

        // 独立散点（R6 反向取样点）：rand 下把间隔并集当独立散点，不当散连续组。
        let rand_shape = PoolShape { placement: Placement::Random, ..base_shape };
        let rand_family_f8a_shape = PoolShape { family: Family::EightAdjacentDataUnitsPerFsync, ..rand_shape };
        let grouped_extent_touch = {
            let data_unit_count = rand_family_f8a_shape.family.data_unit_count(rand_family_f8a_shape.file_count);
            let extent_leaf_cap = node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES);
            let extent_internal_cap = node_capacity(EXTENT_KEY_BYTES, internal_entry_bytes(EXTENT_KEY_BYTES, false));
            let extent_layers = tree_layers(data_unit_count, extent_leaf_cap, extent_internal_cap, 1);
            scattered_continuous_groups_touch(extent_layers[0], extent_leaf_cap as f64, 16.0, 8.0, data_unit_count as f64)
        };
        let independent_extent_touch = {
            let data_unit_count = rand_family_f8a_shape.family.data_unit_count(rand_family_f8a_shape.file_count);
            let extent_leaf_cap = node_capacity(EXTENT_KEY_BYTES, EXTENT_LEAF_ENTRY_BYTES);
            scattered_group_touch(16.0 * 8.0, data_unit_count as f64, extent_leaf_cap as f64)
        };
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=geometry_sensitivity_sample_row6 point=independent_scattered_points p={representative_file_count} family=F8A placement=rand n=16 grouped_extent_leaf_dirty={grouped_extent_touch} independent_extent_leaf_dirty={independent_extent_touch} independent_makes_wal_more_expensive={}",
                independent_extent_touch > grouped_extent_touch,
            ))
        );
    }

    // 判别力自证（第八节 8.2）：`s` 的 10% 与 0 两道门槛在实际值序列里能不能翻面（沿 N 轴，P=1、F1、seq、Lbalanced）。
    {
        let shape = PoolShape { file_count: 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let jia_bytes = solve_jia_publish(shape).total_bytes();
        let fsync_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let mut saved_share_values = Vec::new();
        for &interval in &intervals {
            let checkpoint = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, interval, IntermediateVersionPolicy::ReleasedAndBacklogged);
            let amortized = amortize(fsync_full, checkpoint.total_bytes(), interval);
            let saved_share = 1.0 - amortized / jia_bytes as f64;
            saved_share_values.push((interval, saved_share));
        }
        let crosses_zero = saved_share_values.windows(2).any(|adjacent_pair| (adjacent_pair[0].1 > 0.0) != (adjacent_pair[1].1 > 0.0));
        let crosses_ten_percent = saved_share_values.windows(2).any(|adjacent_pair| (adjacent_pair[0].1 >= 0.10) != (adjacent_pair[1].1 >= 0.10));
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=discriminability_self_proof axis=n shape=p1_f1_seq_lbalanced values={saved_share_values:?} crosses_zero_threshold={crosses_zero} crosses_ten_percent_threshold={crosses_ten_percent}"
            ))
        );
    }

    // ============================================================
    // 第 7 行（第二段，R8 组提交）：P × 族 × 落点 × 策略 × N × concurrent_fsync_count × 共享/不共享，三臂。
    // ============================================================
    let row_seven_file_counts: [u64; 5] = [100, 10_000, 1_000_000, 100_000_000, 145];
    let row_seven_intervals: [u64; 3] = [16, 256, 4096];
    let concurrent_fsync_counts: [u64; 5] = [1, 2, 4, 8, 16];
    let sharing_options = [true, false];

    // 阳性对照（5.5，第 7 行）：B12/B13 那两格，逐字节钉在单测里；这里只报一行，指回单测名。
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control_row7 cell=p100_f1_seq_concurrent_fsync_count_2 jia_batch_bytes={} wal_full_batch_bytes={} wal_leaf_batch_bytes={} shared_extent_dirty_matches_unshared={}",
            solve_jia_batch_publish(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, 2, true).total_bytes(),
            solve_write_ahead_log_batch_fsync(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, WriteAheadLogArm::WriteAheadLogFull, 2, true).total_bytes(),
            solve_write_ahead_log_batch_fsync(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, WriteAheadLogArm::WriteAheadLogLeaf, 2, true).total_bytes(),
            solve_jia_batch_publish(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, 2, true).extent_bytes
                == solve_jia_batch_publish(PoolShape { file_count: 100, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }, 2, false).extent_bytes,
        ))
    );

    for &file_count in &row_seven_file_counts {
        for family in families {
            for placement in placements {
                for policy in policies {
                    for &interval in &row_seven_intervals {
                        for shared in sharing_options {
                            // Q7.3/Q7.3b：沿 concurrent_fsync_count 从小到大找 `s` 第一次 ≤ 0 / < 10% 的那个格（甲不比这条 WAL 臂贵）。
                            let mut fold_point_zero_full: Option<u64> = None;
                            let mut fold_point_ten_percent_full: Option<u64> = None;
                            let mut fold_point_zero_leaf: Option<u64> = None;
                            let mut fold_point_ten_percent_leaf: Option<u64> = None;

                            for &concurrent_fsync_count in &concurrent_fsync_counts {
                                if file_count < concurrent_fsync_count {
                                    continue; // P < concurrent_fsync_count 的格不适用（第一节「读法写死」）。
                                }
                                let shape = PoolShape { file_count, family, placement, policy, geometry: Geometry::main() };
                                let jia = solve_jia_batch_publish(shape, concurrent_fsync_count, shared);
                                let jia_batch_bytes = jia.total_bytes();
                                let jia_amortized_bytes = amortize_batch_per_fsync(jia_batch_bytes, concurrent_fsync_count);

                                let write_ahead_log_full_fsync = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, shared);
                                let write_ahead_log_leaf_fsync = solve_write_ahead_log_batch_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, shared);
                                // N 按 fsync 计（主）：一个间隔 interval 次 fsync = interval/concurrent_fsync_count 批。
                                let batches_in_interval_main = batches_in_interval_by_fsync_count(interval, concurrent_fsync_count);
                                let checkpoint_full = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, shared, batches_in_interval_main, IntermediateVersionPolicy::ReleasedAndBacklogged);
                                let checkpoint_leaf = solve_write_ahead_log_batch_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, concurrent_fsync_count, shared, batches_in_interval_main, IntermediateVersionPolicy::ReleasedAndBacklogged);

                                // WAL 两臂摊到每次 fsync：（N_b 批之和 + checkpoint）÷ interval（第一节「读法写死」）。
                                let amortized_full = (write_ahead_log_full_fsync.total_bytes() as f64 * batches_in_interval_main as f64 + checkpoint_full.total_bytes() as f64) / interval as f64;
                                let amortized_leaf = (write_ahead_log_leaf_fsync.total_bytes() as f64 * batches_in_interval_main as f64 + checkpoint_leaf.total_bytes() as f64) / interval as f64;

                                let ratio_jia_over_full = jia_amortized_bytes / amortized_full;
                                let ratio_jia_over_leaf = jia_amortized_bytes / amortized_leaf;
                                let ratio_full_over_leaf = amortized_full / amortized_leaf;
                                let saved_share_full = 1.0 - amortized_full / jia_amortized_bytes;
                                let saved_share_leaf = 1.0 - amortized_leaf / jia_amortized_bytes;

                                // N_b = 1 的格（concurrent_fsync_count = interval，或 interval=16 时 batches_in_interval_main=1）不算判据（第一节）。
                                let batches_in_interval_equals_one = batches_in_interval_main == 1;
                                if !batches_in_interval_equals_one {
                                    if saved_share_full <= 0.0 && fold_point_zero_full.is_none() {
                                        fold_point_zero_full = Some(concurrent_fsync_count);
                                    }
                                    if saved_share_full < 0.10 && fold_point_ten_percent_full.is_none() {
                                        fold_point_ten_percent_full = Some(concurrent_fsync_count);
                                    }
                                    if saved_share_leaf <= 0.0 && fold_point_zero_leaf.is_none() {
                                        fold_point_zero_leaf = Some(concurrent_fsync_count);
                                    }
                                    if saved_share_leaf < 0.10 && fold_point_ten_percent_leaf.is_none() {
                                        fold_point_ten_percent_leaf = Some(concurrent_fsync_count);
                                    }
                                }

                                // F2（第十节）：甲与两条 WAL checkpoint 核心（K10）分开判；不拿 F2 触发的格判臂（登记原文）。
                                let (jia_unconverged, jia_saturated_trees) = jia_convergence_and_saturation_status(&jia);
                                let (full_unconverged, full_saturated_trees) = checkpoint_convergence_and_saturation_status(&checkpoint_full);
                                let (leaf_unconverged, leaf_saturated_trees) = checkpoint_convergence_and_saturation_status(&checkpoint_leaf);

                                println!(
                                    "{}",
                                    emitter.emit_raw(&format!(
                                        "name=row7_grid p={file_count} family={} placement={} policy={} n={interval} concurrent_fsync_count={concurrent_fsync_count} shared={shared} \
                                         batches_in_interval={batches_in_interval_main} batches_in_interval_equals_one={batches_in_interval_equals_one} \
                                         jia_batch_bytes={jia_batch_bytes} jia_batch_write_calls={} jia_amortized_bytes={jia_amortized_bytes} \
                                         wal_full_batch_bytes={} wal_full_batch_write_calls={} wal_full_checkpoint_bytes={} wal_full_amortized_bytes={amortized_full} \
                                         wal_leaf_batch_bytes={} wal_leaf_batch_write_calls={} wal_leaf_checkpoint_bytes={} wal_leaf_amortized_bytes={amortized_leaf} \
                                         ratio_jia_over_wal_full={ratio_jia_over_full} ratio_jia_over_wal_leaf={ratio_jia_over_leaf} ratio_wal_full_over_wal_leaf={ratio_full_over_leaf} \
                                         saved_share_wal_full={saved_share_full} saved_share_wal_leaf={saved_share_leaf} \
                                         jia_unconverged={jia_unconverged} jia_saturated_trees={jia_saturated_trees:?} \
                                         wal_full_unconverged={full_unconverged} wal_full_saturated_trees={full_saturated_trees:?} \
                                         wal_leaf_unconverged={leaf_unconverged} wal_leaf_saturated_trees={leaf_saturated_trees:?}",
                                        family.tag(),
                                        placement.tag(),
                                        policy.tag(),
                                        jia.write_calls,
                                        write_ahead_log_full_fsync.total_bytes(),
                                        write_ahead_log_full_fsync.write_calls,
                                        checkpoint_full.total_bytes(),
                                        write_ahead_log_leaf_fsync.total_bytes(),
                                        write_ahead_log_leaf_fsync.write_calls,
                                        checkpoint_leaf.total_bytes(),
                                    ))
                                );
                            }

                            println!(
                                "{}",
                                emitter.emit_raw(&format!(
                                    "name=row7_fold_point p={file_count} family={} placement={} policy={} n={interval} shared={shared} \
                                     fold_point_zero_wal_full={:?} fold_point_ten_percent_wal_full={:?} \
                                     fold_point_zero_wal_leaf={:?} fold_point_ten_percent_wal_leaf={:?}",
                                    family.tag(),
                                    placement.tag(),
                                    policy.tag(),
                                    fold_point_zero_full,
                                    fold_point_ten_percent_full,
                                    fold_point_zero_leaf,
                                    fold_point_ten_percent_leaf,
                                ))
                            );
                        }
                    }
                }
            }
        }
    }

    // 8.2（第 7 行反向取样点）：N 按批计——一个间隔 concurrent_fsync_count×batches_in_interval 次 fsync，
    // 不是 interval 次；代表格 P=10⁴、F1、seq、Lbalanced、concurrent_fsync_count=2、interval=16。
    {
        let representative_shape = PoolShape { file_count: 10_000, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let concurrent_fsync_count = 2u64;
        let interval = 16u64;
        let write_ahead_log_full_fsync = solve_write_ahead_log_batch_fsync(representative_shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true);

        let batches_by_fsync = interval / concurrent_fsync_count; // 主：N 按 fsync 计。
        let batches_by_batch = interval; // 反向：N 按批计（一个间隔 interval 批）。

        let checkpoint_by_fsync = solve_write_ahead_log_batch_checkpoint(representative_shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_by_fsync, IntermediateVersionPolicy::ReleasedAndBacklogged);
        let checkpoint_by_batch = solve_write_ahead_log_batch_checkpoint(representative_shape, WriteAheadLogArm::WriteAheadLogFull, concurrent_fsync_count, true, batches_by_batch, IntermediateVersionPolicy::ReleasedAndBacklogged);

        let amortized_by_fsync = (write_ahead_log_full_fsync.total_bytes() as f64 * batches_by_fsync as f64 + checkpoint_by_fsync.total_bytes() as f64) / interval as f64;
        // 反向读法下，一个「间隔」实际跨越 interval×concurrent_fsync_count 次 fsync，摊销分母也要跟着换。
        let amortized_by_batch = (write_ahead_log_full_fsync.total_bytes() as f64 * batches_by_batch as f64 + checkpoint_by_batch.total_bytes() as f64) / (batches_by_batch * concurrent_fsync_count) as f64;

        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=geometry_sensitivity_sample_row7 point=interval_counted_by_batch p=10000 family=F1 placement=seq policy=Lbalanced concurrent_fsync_count=2 n=16 \
                 checkpoint_bytes_by_fsync={} checkpoint_bytes_by_batch={} amortized_by_fsync={amortized_by_fsync} amortized_by_batch={amortized_by_batch} \
                 makes_wal_full_cheaper={}",
                checkpoint_by_fsync.total_bytes(),
                checkpoint_by_batch.total_bytes(),
                amortized_by_batch < amortized_by_fsync,
            ))
        );

        // 不共享取随机（F9）：这一版没有建独立开关（与 K12、N 按批计不同，这一条完全未实现）。
        let unshared_random_note = "不共享取随机（k 个文件在全池均匀独立地落，R6 的 N_g=concurrent_fsync_count、s=d）这一版没有建可计算的开关，F9 答不了，交下一段或主 agent";
        println!("{}", emitter.emit_raw(&format!("name=geometry_sensitivity_sample_row7 point=unshared_random_not_modeled_this_pass p=10000 note={unshared_random_note:?}")));
    }

    println!("{}", emitter.finish());
}
