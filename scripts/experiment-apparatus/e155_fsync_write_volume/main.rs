//! E155：每次持久化写多少 —— 三种 fsync 形态（甲 / write_ahead_log_full / 乙）的确定性计数模型。
//!
//! 判据、失败条款、变异表的权威登记在 `research/prompts/e155-preregistration.md`（跑前写死）。
//! 本文件只实现登记第五节定义的模型，不与 `crates/` 共用代码
//! （`.claude/rules/implementation-first.md` 第 4 条）。
//!
//! 这一次交付只做登记 5.7 的第一、二段：模型核心（K1–K8、5.3 不动点）、甲在第 1 行主表上的值、
//! 三条臂各自的阳性对照、Q1.1/Q1.2/Q1.3/Q1.4/Q1.5/Q1.5b/Q1.6/Q1.7/Q1.8。
//! 第三、四段（第 2–4 行反事实上界、第 5 行 P 门槛搜索）未做，运行记录里逐条标「够判后未跑」或「未做」。

use e7_index_bench::Emitter;

mod byte_widths_and_tree_shape;
mod family_geometry_and_jia_publish;
mod partial_and_whole_unit_rewrite_savings;
mod placement_and_dirty_node_counts;
mod tree_heights_and_growth_thresholds;
mod write_ahead_log_fsync_and_checkpoint;

use self::byte_widths_and_tree_shape::*;
use self::family_geometry_and_jia_publish::*;
use self::partial_and_whole_unit_rewrite_savings::*;
use self::placement_and_dirty_node_counts::*;
use self::tree_heights_and_growth_thresholds::*;
use self::write_ahead_log_fsync_and_checkpoint::*;

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config node_bytes={NODE_BYTES} unit_bytes={UNIT_BYTES} record_bytes={RECORD_BYTES} system_configuration_slot_bytes={SYSTEM_CONFIGURATION_SLOT_BYTES} device_count={DEVICE_COUNT} named_items_per_record={NAMED_ITEMS_PER_RECORD_MAIN} ring_default_bytes={RING_DEFAULT_BYTES} ring_default_in_flight_limit={} effective_dirty_data_budget_bytes={EFFECTIVE_DIRTY_DATA_BUDGET_BYTES} model=counting file_ops=0",
            default_ring_in_flight_limit()
        ))
    );
    println!(
        "{}",
        emitter.emit_raw(
            "name=scope stage=1-2 covers=K1,K2,K3,K4(phi=1/0.5两点),K5,K6,K7,K8,K9,5.3不动点,5.4甲/write_ahead_log_full/乙,阳性对照(5.5),V1,V2,Q1.1,Q1.2,Q1.3,Q1.4(原始值齐/标签只算代表格),Q1.5,Q1.5b,Q1.6(值+轨迹/判据已撤),Q1.7,Q1.8,7.3手算格5条 missing=第2-4行反事实上界(Q2/Q3/Q4)、第5行P门槛搜索(Q5)、8.2全量反向几何扫描(只做代表格)、Q1.4全量标签矩阵、树表嵌套轴(Q2.5)、Q1.6b、块层写请求列"
        )
    );

    // 七 B1：容量一览。
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

    // 5.5 阳性对照：三条臂各自的格与「必须出现的差」。
    let control_shape = PoolShape { file_count: 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
    let jia_fsync = solve_jia_publish(control_shape).total_bytes();
    let control_write_ahead_log_full_fsync = solve_write_ahead_log_fsync(control_shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let control_write_ahead_log_leaf_fsync = solve_write_ahead_log_fsync(control_shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control cell=p1_f1_seq jia_fsync_bytes={jia_fsync} write_ahead_log_full_fsync_bytes={control_write_ahead_log_full_fsync} write_ahead_log_leaf_fsync_bytes={control_write_ahead_log_leaf_fsync} jia_minus_write_ahead_log_full={} write_ahead_log_full_minus_write_ahead_log_leaf={} jia_root_slots_per_16_fsync=16 write_ahead_log_full_root_slots_per_16_fsync=1",
            jia_fsync - control_write_ahead_log_full_fsync,
            control_write_ahead_log_full_fsync - control_write_ahead_log_leaf_fsync
        ))
    );
    let control_shape_at_file_count_145 = PoolShape { file_count: 145, ..control_shape };
    let write_ahead_log_full_at_file_count_145 = solve_write_ahead_log_fsync(control_shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
    let write_ahead_log_leaf_at_file_count_145 = solve_write_ahead_log_fsync(control_shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
    let checkpoint_leaf_at_file_count_145 = solve_write_ahead_log_checkpoint(control_shape_at_file_count_145, WriteAheadLogArm::WriteAheadLogLeaf, 16);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=positive_control cell=p145_f1_seq write_ahead_log_full_fsync_bytes={write_ahead_log_full_at_file_count_145} write_ahead_log_leaf_fsync_bytes={write_ahead_log_leaf_at_file_count_145} write_ahead_log_leaf_checkpoint_extent_catch_up_bytes={} write_ahead_log_leaf_checkpoint_inode_catch_up_bytes={}",
            checkpoint_leaf_at_file_count_145.extent_catch_up_bytes, checkpoint_leaf_at_file_count_145.inode_catch_up_bytes
        ))
    );

    // 第 1 行主表（5.6）：P × 族 × 落点 × N × 三种位置策略，三条臂同一行各报各的列。
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
                        let write_ahead_log_full_checkpoint = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogFull, interval);
                        let write_ahead_log_leaf_checkpoint = solve_write_ahead_log_checkpoint(shape, WriteAheadLogArm::WriteAheadLogLeaf, interval);
                        let jia_amortized = jia.total_bytes() as f64; // 甲每次 fsync 就是一次发布，两行同值（读法写死）。
                        let write_ahead_log_full_amortized = amortize(write_ahead_log_full_fsync.total_bytes(), write_ahead_log_full_checkpoint.total_bytes(), interval);
                        let write_ahead_log_leaf_amortized = amortize(write_ahead_log_leaf_fsync.total_bytes(), write_ahead_log_leaf_checkpoint.total_bytes(), interval);
                        let in_flight_peak_full = interval * write_ahead_log_full_fsync.record_count + write_ahead_log_full_checkpoint.record_count;
                        let in_flight_peak_leaf = interval * write_ahead_log_leaf_fsync.record_count + write_ahead_log_leaf_checkpoint.record_count;
                        let ring_bytes_needed_full = RING_SAFETY_FACTOR_MAIN * RECORD_BYTES * in_flight_peak_full;
                        let ring_bytes_needed_leaf = RING_SAFETY_FACTOR_MAIN * RECORD_BYTES * in_flight_peak_leaf;
                        let limit = default_ring_in_flight_limit();
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=row1_grid p={file_count} family={} placement={} policy={} n={interval} \
                                 jia_fsync_bytes={} jia_calls={} jia_barriers={} jia_fua={} jia_amortized_bytes={jia_amortized} \
                                 write_ahead_log_full_fsync_bytes={} write_ahead_log_full_fsync_calls={} write_ahead_log_full_checkpoint_bytes={} write_ahead_log_full_checkpoint_calls={} write_ahead_log_full_amortized_bytes={write_ahead_log_full_amortized} write_ahead_log_full_in_flight_peak={in_flight_peak_full} write_ahead_log_full_ring_bytes_needed={ring_bytes_needed_full} write_ahead_log_full_default_ring_ok={} \
                                 write_ahead_log_leaf_fsync_bytes={} write_ahead_log_leaf_fsync_calls={} write_ahead_log_leaf_checkpoint_bytes={} write_ahead_log_leaf_checkpoint_calls={} write_ahead_log_leaf_amortized_bytes={write_ahead_log_leaf_amortized} write_ahead_log_leaf_in_flight_peak={in_flight_peak_leaf} write_ahead_log_leaf_ring_bytes_needed={ring_bytes_needed_leaf} write_ahead_log_leaf_default_ring_ok={} \
                                 extent_layers={} inode_layers={} allocation_layers={} mapping_layers={} fixed_point_iterations={} g23_1_share={}",
                                family.tag(),
                                placement.tag(),
                                policy.tag(),
                                jia.total_bytes(),
                                jia.write_calls,
                                jia.barriers,
                                jia.fua_count,
                                write_ahead_log_full_fsync.total_bytes(),
                                write_ahead_log_full_fsync.write_calls,
                                write_ahead_log_full_checkpoint.total_bytes(),
                                write_ahead_log_full_checkpoint.write_calls,
                                in_flight_peak_full <= limit,
                                write_ahead_log_leaf_fsync.total_bytes(),
                                write_ahead_log_leaf_fsync.write_calls,
                                write_ahead_log_leaf_checkpoint.total_bytes(),
                                write_ahead_log_leaf_checkpoint.write_calls,
                                in_flight_peak_leaf <= limit,
                                jia.extent_layers.len(),
                                jia.inode_layers.len(),
                                jia.allocation_layers.len(),
                                jia.mapping_layers.len(),
                                jia.fixed_point_iterations,
                                jia.ancestor_and_fixed_point_share_numerator_bytes() as f64 / jia.total_bytes() as f64,
                            ))
                        );
                    }
                }
            }
        }
    }

    // Q1.8：另一口径（C310）—— 每次 fsync 记录数取 max(d, ⌈点名项/67⌉)；重算环大小是否敏感。
    for &file_count in &[1u64, 10_000, 100_000_000] {
        for family in families {
            let shape = PoolShape { file_count, family, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
            let jia = solve_jia_publish(shape);
            let fsync_data_unit_count = family.data_units_per_fsync();
            let alternate_record_count = fsync_data_unit_count.max(jia.named_items.div_ceil(NAMED_ITEMS_PER_RECORD_MAIN));
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=q1_8_record_count_alternate p={file_count} family={} main_record_count={} alternate_record_count={alternate_record_count} differs={}",
                    family.tag(),
                    jia.record_count,
                    jia.record_count != alternate_record_count
                ))
            );
        }
    }

    // 8.2 几何敏感性：只做代表格（P=100000, F1, seq, Lbalanced, N=16），不是全量 1800 格扫描（见「它答不了的」）。
    {
        let representative_file_count = 100_000u64;
        let base_shape = PoolShape { file_count: representative_file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let base_jia = solve_jia_publish(base_shape).total_bytes();
        let base_write_ahead_log_full = solve_write_ahead_log_fsync(base_shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
        let base_write_ahead_log_leaf = solve_write_ahead_log_fsync(base_shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();

        let half_fill_shape = PoolShape { geometry: Geometry { fill_ratio: FillRatio::Half, ..Geometry::main() }, ..base_shape };
        let reverse_identity_reference_shape = PoolShape { geometry: Geometry { identity_reference: true, ..Geometry::main() }, ..base_shape };
        let no_backlog_shape = PoolShape { geometry: Geometry { backlog_generations: 0, ..Geometry::main() }, ..base_shape };
        let reverse_pbs_shape = PoolShape { geometry: Geometry { physical_block_size_bytes: REVERSE_PHYSICAL_BLOCK_SIZE_BYTES, ..Geometry::main() }, ..base_shape };

        for (label, shape) in [
            ("phi_0.5", half_fill_shape),
            ("k11", reverse_identity_reference_shape),
            ("g_0", no_backlog_shape),
            ("pbs_4096", reverse_pbs_shape),
        ] {
            let jia = solve_jia_publish(shape).total_bytes();
            let write_ahead_log_full = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogFull).total_bytes();
            let write_ahead_log_leaf = solve_write_ahead_log_fsync(shape, WriteAheadLogArm::WriteAheadLogLeaf).total_bytes();
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=geometry_sensitivity_sample point={label} p={representative_file_count} base_jia={base_jia} reverse_jia={jia} base_write_ahead_log_full={base_write_ahead_log_full} reverse_write_ahead_log_full={write_ahead_log_full} base_write_ahead_log_leaf={base_write_ahead_log_leaf} reverse_write_ahead_log_leaf={write_ahead_log_leaf} jia_ratio_changes={} write_ahead_log_full_ratio_changes={}",
                    (base_jia as f64 / base_write_ahead_log_full as f64 - jia as f64 / write_ahead_log_full as f64).abs() > 1e-9,
                    (base_write_ahead_log_full as f64 / base_write_ahead_log_leaf as f64 - write_ahead_log_full as f64 / write_ahead_log_leaf as f64).abs() > 1e-9,
                ))
            );
        }

        // K10：只对 WAL 两臂 checkpoint 有意义（甲不受影响）。
        let checkpoint_full = solve_write_ahead_log_checkpoint(base_shape, WriteAheadLogArm::WriteAheadLogFull, 16).total_bytes();
        let checkpoint_leaf = solve_write_ahead_log_checkpoint(base_shape, WriteAheadLogArm::WriteAheadLogLeaf, 16).total_bytes();
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=geometry_sensitivity_sample point=k10_not_modeled p={representative_file_count} note=K10（间隔内被换掉的中间版也写一条已释放的分配记录）未建，checkpoint_bytes_k9_full={checkpoint_full} checkpoint_bytes_k9_leaf={checkpoint_leaf}"
            ))
        );
    }

    // 第 2–4 行（第三段）：第 1 行主表的 P × 族 × 落点 × 位置策略，只算甲（登记 5.6）。
    let row_2_to_4_file_counts: [u64; 6] = [1, 100, 10_000, 1_000_000, 100_000_000, 145];
    for &file_count in &row_2_to_4_file_counts {
        for family in families {
            for placement in placements {
                for policy in policies {
                    let shape = PoolShape { file_count, family, placement, policy, geometry: Geometry::main() };
                    let outcome = solve_jia_publish(shape);
                    let baseline_bytes = outcome.total_bytes();

                    let accounting_upper_bound = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { accounting: true, ..Default::default() });
                    let allocation_upper_bound = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { allocation: true, ..Default::default() });
                    let mapping_upper_bound = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { mapping: true, ..Default::default() });
                    let tree_table_upper_bound = fixed_point_counterfactual_share(shape, SuppressedFixedPoints { tree_table: true, ..Default::default() });
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=row2_fixed_point_upper_bounds p={file_count} family={} placement={} policy={} \
                             u_accounting={accounting_upper_bound} u_accounting_over_10pct={} \
                             u_allocation={allocation_upper_bound} u_allocation_over_10pct={} \
                             u_mapping={mapping_upper_bound} u_mapping_over_10pct={} \
                             u_tree_table={tree_table_upper_bound} u_tree_table_over_10pct={} \
                             direct_share_accounting={} direct_share_allocation={} direct_share_mapping={} direct_share_tree_table={}",
                            family.tag(),
                            placement.tag(),
                            policy.tag(),
                            accounting_upper_bound >= 0.10,
                            allocation_upper_bound >= 0.10,
                            mapping_upper_bound >= 0.10,
                            tree_table_upper_bound >= 0.10,
                            outcome.accounting_bytes as f64 / baseline_bytes as f64,
                            outcome.allocation_bytes as f64 / baseline_bytes as f64,
                            outcome.mapping_bytes as f64 / baseline_bytes as f64,
                            outcome.tree_table_bytes as f64 / baseline_bytes as f64,
                        ))
                    );

                    for pbs in [PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES, REVERSE_PHYSICAL_BLOCK_SIZE_BYTES] {
                        let whole_unit_rewrite_upper_bound = whole_unit_rewrite_counterfactual_share(shape, pbs);
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=row3_whole_unit_rewrite_upper_bound p={file_count} family={} placement={} policy={} pbs={pbs} u_3={whole_unit_rewrite_upper_bound} u_3_over_10pct={}",
                                family.tag(),
                                placement.tag(),
                                policy.tag(),
                                whole_unit_rewrite_upper_bound >= 0.10,
                            ))
                        );
                    }

                    for content_bytes in [100u64, 4000, 4100, 16_000, 32_634] {
                        for pbs in [PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES, REVERSE_PHYSICAL_BLOCK_SIZE_BYTES] {
                            let data_unit_rounding_upper_bound = data_unit_rounding_counterfactual_share(shape, content_bytes, pbs);
                            println!(
                                "{}",
                                emitter.emit_raw(&format!(
                                    "name=row4_data_unit_rounding_upper_bound p={file_count} family={} placement={} policy={} content_bytes={content_bytes} pbs={pbs} u_4={data_unit_rounding_upper_bound} u_4_over_10pct={}",
                                    family.tag(),
                                    placement.tag(),
                                    policy.tag(),
                                    data_unit_rounding_upper_bound >= 0.10,
                                ))
                            );
                        }
                    }
                }
            }
        }
    }

    // Q2.5：树表随树的棵数 T（登记 5.6 第 2 行，P = 10⁴、F1、seq）。
    {
        let shape = PoolShape { file_count: 10_000, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        for tree_table_entry_count in [7u64, 81, 82, 6561, 6562] {
            let tree_table_upper_bound = tree_table_counterfactual_share(shape, tree_table_entry_count);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=row2_tree_table_count_axis p=10000 family=F1 placement=seq policy=Lbalanced tree_table_entry_count={tree_table_entry_count} u_tree_table={tree_table_upper_bound} u_tree_table_over_10pct={}",
                    tree_table_upper_bound >= 0.10,
                ))
            );
        }
    }

    // 第 5 行（第四段，用户直接问的那一题）：甲每次持久化写字节随对象数怎么涨——只算甲、F1、seq 与 rand、三种位置策略。
    let row_5_file_counts: [u64; 9] = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000];
    for &file_count in &row_5_file_counts {
        for placement in placements {
            for policy in policies {
                let shape = PoolShape { file_count, family: Family::OneDataUnitPerFile, placement, policy, geometry: Geometry::main() };
                let (total_bytes, tree_bytes) = total_bytes_and_tree_bytes(shape);
                let (path_ratio_by_actual_height, path_ratio_by_maximum_height_node_width, path_ratio_by_maximum_height_unit_width) = rho_readings(shape);
                let outcome = solve_jia_publish(shape);
                let heights = tree_heights(&outcome);
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=row5_growth p={file_count} placement={} policy={} a={total_bytes} tree_bytes={tree_bytes} \
                         rho_a={path_ratio_by_actual_height} rho_a_over_1={} rho_b={path_ratio_by_maximum_height_node_width} rho_b_over_1={} rho_c={path_ratio_by_maximum_height_unit_width} rho_c_over_1={} \
                         extent_height={} inode_height={} allocation_height={} accounting_height={} mapping_height={} tree_table_height={} \
                         named_items={} record_count={}",
                        placement.tag(),
                        policy.tag(),
                        path_ratio_by_actual_height > 1.0,
                        path_ratio_by_maximum_height_node_width > 1.0,
                        path_ratio_by_maximum_height_unit_width > 1.0,
                        heights[0],
                        heights[1],
                        heights[2],
                        heights[3],
                        heights[4],
                        heights[5],
                        outcome.named_items,
                        outcome.record_count,
                    ))
                );
            }
        }
    }

    // 第 5 行：每棵树每次长高一层的门槛 P* 与 P* − 1（主几何下逐树搜出来；φ = 0.5 时另搜一遍），报 e 值。
    for (fill_ratio_label, fill_ratio) in [("phi_1", FillRatio::Full), ("phi_0.5", FillRatio::Half)] {
        for (tree_label, height_of) in [
            ("extent", 0usize),
            ("inode", 1usize),
            ("allocation", 2usize),
            ("mapping", 4usize),
        ] {
            let geometry = Geometry { fill_ratio, ..Geometry::main() };
            let height_at = |file_count: u64| {
                let shape = PoolShape { file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry };
                let outcome = solve_jia_publish(shape);
                let heights = tree_heights(&outcome);
                heights[height_of]
            };
            let thresholds = height_growth_thresholds(100_000_000, height_at);
            for &threshold in thresholds.iter().take(4) {
                let shape_before = PoolShape { file_count: threshold - 1, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry };
                let shape_at_threshold = PoolShape { file_count: threshold, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry };
                let (_, tree_bytes_before_threshold) = total_bytes_and_tree_bytes(shape_before);
                let (_, tree_bytes_at_threshold) = total_bytes_and_tree_bytes(shape_at_threshold);
                let bytes_per_layer_growth_ratio = (tree_bytes_at_threshold as f64 - tree_bytes_before_threshold as f64) / (NODE_BYTES as f64 * DEVICE_COUNT as f64);
                let mut other_trees_changed_height = Vec::new();
                for (other_label, other_height_of) in [("extent", 0usize), ("inode", 1usize), ("allocation", 2usize), ("mapping", 4usize)] {
                    if other_label == tree_label {
                        continue;
                    }
                    let before_outcome = solve_jia_publish(shape_before);
                    let at_outcome = solve_jia_publish(shape_at_threshold);
                    if tree_heights(&before_outcome)[other_height_of] != tree_heights(&at_outcome)[other_height_of] {
                        other_trees_changed_height.push(other_label);
                    }
                }
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=row5_height_threshold geometry={fill_ratio_label} tree={tree_label} p_star={threshold} p_star_minus_1={} e={bytes_per_layer_growth_ratio} e_over_1_05={} other_trees_changed_height={:?}",
                        threshold - 1,
                        bytes_per_layer_growth_ratio > 1.05,
                        other_trees_changed_height,
                    ))
                );
            }
        }
    }

    // 第 5 行 Q5.4：甲每次持久化的记录数随 P 的轨迹（只看 F1、seq、Lbalanced 这一条代表序列）。
    for &file_count in &row_5_file_counts {
        let shape = PoolShape { file_count, family: Family::OneDataUnitPerFile, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() };
        let outcome = solve_jia_publish(shape);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=row5_record_count_trend p={file_count} record_count={} named_items={}",
                outcome.record_count, outcome.named_items,
            ))
        );
    }

    println!("{}", emitter.finish());
}
