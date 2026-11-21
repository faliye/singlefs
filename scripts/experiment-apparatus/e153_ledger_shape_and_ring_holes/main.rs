//! E153：账本形态与环上有洞的代价 —— 记账第 1 项与可再分配谓词两种形态、各配两种影子账（四条臂）
//! 与真实基线「今」的确定性计数模型。
//!
//! 判据、失败条款、变异表的权威登记在 research/prompts/e153-preregistration.md（跑前写死）。
//! 本文件只实现登记里定义的模型；不与 crates/ 共用代码（.claude/rules/implementation-first.md 第 4 条）。
//!
//! ## 骨架（对应登记第五节 M0）
//!
//! 两块盘 Disk0/Disk1，一个单元两盘同槽；根环 R=3 区域、区域设备归属 [Disk0,Disk1,Disk0]、槽序跨区轮转；
//! 每区槽数 S（几何旋钮）、N=3S。九个「角色」构成一次发布触碰的对象：实例表单元、树表单元、
//! 中央映射树、extent 根、inode 根、分配记录树、记账树、数据单元、inode 叶容器；
//! O 触碰其中 8 个（放 10 槽）、E 触碰其中 4 个（放 4 槽）、W = E 的 4 个加实例表单元（放 6 槽）。
//!
//! 记账第 1/5 项对「甲-T1」「G12」两条主谓词按登记第五节的「记账环」公式现算（与物理是否已回收无关）；
//! 第 2 项走独立的事件路径（分配减、记录由不可再分配变可再分配时加）。基线臂「今」的记账三项直接
//! 反映分配器的物理状态（`allocated_slots` / `free_slots` / `deferred_slots`，与 `crates/singlefs-core/src/allocator.rs`
//! 同构），回收只在挂载 / 切换 / 回退重建 / 抬 F 时发生。

use e7_index_bench::i64_from_u64;
use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet, HashMap};

mod accounting_and_candidate_references;
mod arm_executor;
mod ledger_pool_state_and_arms;
mod pool_geometry_and_root_ring;
mod roles_and_allocation_order;
#[cfg(test)]
mod test_support_for_the_world_runs;
mod world_of_the_two_disks;

use self::accounting_and_candidate_references::*;
use self::arm_executor::*;
use self::ledger_pool_state_and_arms::*;
use self::pool_geometry_and_root_ring::*;
use self::roles_and_allocation_order::*;
#[cfg(test)]
use self::test_support_for_the_world_runs::*;
use self::world_of_the_two_disks::*;

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config unit_area_capacity_slots={UNIT_AREA_CAPACITY_SLOTS} ring_regions={ROOT_RING_REGIONS} record_width_single={RECORD_WIDTH_BYTES_SINGLE_GENERATION} record_width_dual={RECORD_WIDTH_BYTES_DUAL_GENERATION} records_per_node_single={RECORDS_PER_NODE_SINGLE_GENERATION} records_per_node_dual={RECORDS_PER_NODE_DUAL_GENERATION} model=counting file_ops=0"
        ))
    );
    println!(
        "{}",
        emitter.emit_raw(
            "name=scope covers=K1,K2,K3,K4,K5,A1,A2,A3,A4,A5,A6,A7,U1..U15,Q1,Q2,Q3,Q4i,Q4ii,Q5,Q6,Q6b(甲/乙两读法),Q7(G27),Q9a(峰值/均值/末值),Q9b(峰值/均值/末值，发布前步/重建步分开),Q10,Q11,Q12,Q13(H1,H2,H3phi0/1/2,H3sq,H4d2,H4dNm2,H5early,H5late,H6,H7,12格),Q8(部分：B1a/B1b/B2/Bk1/Bk2，β1/β2 两基底) missing=Q8(部分：B3a/B3b/B4a/B4b/B4c 未实现，需要重新跑一遍历史（回收步一律不回收）或强制落点碰撞，本段未建），crash_branches(c1/c2/L2/L3，登记估过是千万发布级，本段未建：需要一个与 plain_remount 不同的『施加一条 dangling 记录』恢复路径，语义复杂度与规模都超出本段能安全完成的范围，量级见下） note=第五段新增 Q6b/Q7/Q10 与 Q8 的 B1a/B1b/B2/Bk1/Bk2 五种坏镜像（20 条新单测、10 条新变异，全部先证明会红）；G27、Q10 推法分歧、Q6b 上界不成立三项发现见实验页「产物里的数」。崩溃支线（L2/L3）与 Q8 剩余三种坏镜像（B3a/B3b/B4a/B4b/B4c）留给下一段：前者需要新的恢复函数与千万级发布量的可行性评估（见跑前登记第十二节第五段），后者需要「重跑历史且禁用回收」与「强制落点碰撞」两种新机制"
        )
    );

    // K1 / A6：新池新建文件（P-impl，S=8，六格里报 S=8 一格；六格数值相同，K1 的单测覆盖了另外五格）。
    let new_pool_file_creation_cell = BASE_GEOMETRY;
    let new_pool_file_creation_world = new_pool_file_creation(new_pool_file_creation_cell);
    let threshold_conservative_arm = &new_pool_file_creation_world.arms[World::arm_index(Arm::ThresholdConservative)];
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=k1_new_pool_file_creation_placement s={} data_unit={} extent_root={} inode_leaf={} inode_root={} alloc_tree={} acct_tree={} central_map_tree={} table_unit={} slot_50241_free={}",
            new_pool_file_creation_cell.slots_per_region,
            threshold_conservative_arm.refs[&Role::DataUnit],
            threshold_conservative_arm.refs[&Role::ExtentRoot],
            threshold_conservative_arm.refs[&Role::InodeLeaf],
            threshold_conservative_arm.refs[&Role::InodeRoot],
            threshold_conservative_arm.refs[&Role::AllocationTree],
            threshold_conservative_arm.refs[&Role::AcctTree],
            threshold_conservative_arm.refs[&Role::CentralMapTree],
            threshold_conservative_arm.refs[&Role::TableUnit],
            threshold_conservative_arm.is_free(50241)
        ))
    );
    for arm in &new_pool_file_creation_world.arms {
        let row = arm.accounting_row();
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=a6_new_pool_file_creation_accounting arm={} item1={} item5={} item2={}",
                arm.arm.tag(),
                row.item1_occupied_slots,
                row.item5_deferred_slots,
                row.item2_free_slots
            ))
        );
    }

    // A4 / K4 / K5：对全部 12 格现算一遍，逐格写一行（与单测钉的绝对值同一批公式，产物给的是 12 格全量）。
    for cell in all_geometry_cells() {
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {}
        let ring_capacity = cell.ring_capacity();
        let mut last_overwrite_txg = None;
        let mut last_overwrite_accounting = [AccountingRow::default(); 5];
        for outcome in world.run_workload_segment(6 * ring_capacity, cell.workload_period) {
            if outcome.is_nonempty {
                last_overwrite_txg = Some(outcome.txg);
                last_overwrite_accounting = outcome.accounting;
            }
        }
        let measured_index = World::arm_index(Arm::ThresholdConservative);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=a4_steady_state s={} placement={:?} period={} last_overwrite_txg={} item1={} item5={}",
                cell.slots_per_region,
                cell.placement,
                cell.workload_period,
                last_overwrite_txg.unwrap_or(0),
                last_overwrite_accounting[measured_index].item1_occupied_slots,
                last_overwrite_accounting[measured_index].item5_deferred_slots
            ))
        );
    }

    // K4：全部 12 格现算一遍回退（d = 2），报写行发布 txg 与暖机次数。
    for cell in all_geometry_cells() {
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {}
        let ring_capacity = cell.ring_capacity();
        for _ in world.run_workload_segment(3 * ring_capacity, cell.workload_period) {}
        let newest_txg = world.pool.current_txg;
        let target_ceiling = newest_txg - 2;
        let target_txg = world.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| *txg <= target_ceiling).max();
        let Some(target_txg) = target_txg else {
            println!("{}", emitter.emit_raw(&format!("name=k4_rollback s={} placement={:?} period={} no_candidate=true", cell.slots_per_region, cell.placement, cell.workload_period)));
            continue;
        };
        let outcomes = world.rollback_to(target_txg);
        let write_row_txg = outcomes.iter().find(|outcome| outcome.label == "write_row").map(|outcome| outcome.txg).unwrap_or(0);
        let warm_up_publish_count = outcomes.iter().filter(|outcome| outcome.label == "empty").count();
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=k4_rollback s={} placement={:?} period={} write_row_txg={} warm_up_publish_count={}",
                cell.slots_per_region, cell.placement, cell.workload_period, write_row_txg, warm_up_publish_count
            ))
        );
    }

    // K5：ρ = 1 的 6 格现算一遍抬 F。
    for slots_per_region in [4u64, 8, 16] {
        for placement in [PlacementPolicy::Impl, PlacementPolicy::Low] {
            let cell = GeometryCell { slots_per_region, placement, workload_period: 1 };
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let ring_capacity = cell.ring_capacity();
            for _ in world.run_workload_segment(3 * ring_capacity, 1) {}
            let outcomes = world.raise_floor();
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=k5_raise_floor s={slots_per_region} placement={placement:?} target_floor={} publish_count={}",
                    world.pool.current_rollback_floor,
                    outcomes.len()
                ))
            );
        }
    }

    // 第四段：H1、H2、H3φ(0/1/2)、H3²、H4(d=2)、H4(d=N-2)、H5早、H5晚、H6、H7，全部 12 格，
    // Q1-Q6、Q9(峰值/均值/末值轨迹)、Q11-Q13。崩溃支线（L2/L3）、G27/坏镜像矩阵（Q7/Q8/Q10）留到第五段
    // （见跑前登记第十二节第四段与实验页「它答不了的」）。
    #[allow(clippy::type_complexity, reason = "每一项就是（族名，跑那一族历史的函数指针），只在这个函数里遍历，命名别名不会让哪一步更好验")]
    let families: [(&str, fn(&mut World) -> Option<Vec<PublishOutcome>>); 9] = [
        ("H1", |world| Some(world.history_continuous_overwrite())),
        ("H2", |world| Some(world.history_plain_remount_midway())),
        ("H3phi0", |world| Some(world.history_root_slot_write_failure_and_switch(0, false))),
        ("H3phi1", |world| Some(world.history_root_slot_write_failure_and_switch(1, false))),
        ("H3phi2", |world| Some(world.history_root_slot_write_failure_and_switch(2, false))),
        ("H3sq", |world| Some(world.history_root_slot_write_failure_and_switch(0, true))),
        ("H4d2", |world| world.history_single_rollback(2).map(|(outcomes, _, _)| outcomes)),
        ("H4dNm2", |world| { let rollback_depth = world.geometry.ring_capacity() - 2; world.history_single_rollback(rollback_depth).map(|(outcomes, _, _)| outcomes) }),
        ("H5early", |world| world.history_double_rollback_shallower_second()),
    ];
    #[allow(clippy::type_complexity, reason = "每一项就是（族名，跑那一族历史的函数指针），只在这个函数里遍历，命名别名不会让哪一步更好验")]
    let families_needing_first_rollback_target: [(&str, fn(&mut World) -> Option<Vec<PublishOutcome>>); 3] = [
        ("H5late", |world| world.history_double_rollback_to_first_rollback_write_row()),
        ("H6", |world| world.history_raise_floor_then_rollback()),
        ("H7", |world| world.history_rollback_deep_then_raise_floor()),
    ];
    for cell in all_geometry_cells() {
        for (family, runner) in families.iter().chain(families_needing_first_rollback_target.iter()) {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {
            }
            let ring_capacity = cell.ring_capacity();
            match runner(&mut world) {
                None => {
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=q_family family={family} s={} placement={:?} period={} q13_no_candidate=true",
                            cell.slots_per_region, cell.placement, cell.workload_period
                        ))
                    );
                }
                Some(outcomes) => {
                    // Q9a：五条臂各自的条目数轨迹（峰值/均值/末值），供下面算「多写」用。
                    let mut entries_peak_by_index = [0u64; 5];
                    let mut entries_mean_by_index = [0.0f64; 5];
                    for index in 0..5 {
                        let arm_tag = ARM_ORDER[index].tag();
                        let mut allocated_mismatch_arm_reading_count = 0u64;
                        let mut allocated_mismatch_today_reading_count = 0u64;
                        let mut free_mismatch_count = 0u64;
                        let mut allocated_minus_deferred_reference_mismatch_count = 0u64;
                        let mut over_held_peak = i64::MIN;
                        let mut over_held_positive_publishes = 0u64;
                        let mut isolated_peak = 0u64;
                        let mut isolated_positive_publishes = 0u64;
                        let mut oldest_valid_root_lag_peak = 0u64;
                        let mut oldest_valid_root_lag_at_or_above_ring_capacity_publishes = 0u64;
                        let mut floor_only_on_one_disk_publishes = 0u64;
                        for outcome in &outcomes {
                            if outcome.invariant_snapshot.allocated_statistic_mismatch_by_arm_reading[index] {
                                allocated_mismatch_arm_reading_count += 1;
                            }
                            if outcome.invariant_snapshot.allocated_statistic_mismatch_by_today_checker_reading[index] {
                                allocated_mismatch_today_reading_count += 1;
                            }
                            if outcome.invariant_snapshot.free_statistic_mismatch[index] {
                                free_mismatch_count += 1;
                            }
                            if outcome.invariant_snapshot.allocated_minus_deferred_reference_mismatch[index] {
                                allocated_minus_deferred_reference_mismatch_count += 1;
                            }
                            let over_held = outcome.pre[index].over_held_slots;
                            over_held_peak = over_held_peak.max(over_held);
                            if over_held > 0 {
                                over_held_positive_publishes += 1;
                            }
                            let isolated = outcome.pre[index].isolated_slots;
                            isolated_peak = isolated_peak.max(isolated);
                            if isolated > 0 {
                                isolated_positive_publishes += 1;
                            }
                            oldest_valid_root_lag_peak = oldest_valid_root_lag_peak.max(outcome.invariant_snapshot.oldest_valid_root_lag);
                            if outcome.invariant_snapshot.oldest_valid_root_lag >= ring_capacity {
                                oldest_valid_root_lag_at_or_above_ring_capacity_publishes += 1;
                            }
                            if outcome.invariant_snapshot.floor_only_on_one_disk {
                                floor_only_on_one_disk_publishes += 1;
                            }
                        }
                        let over_held_end = outcomes.last().map_or(0, |outcome| outcome.pre[index].over_held_slots);
                        let isolated_end = outcomes.last().map_or(0, |outcome| outcome.pre[index].isolated_slots);
                        let oldest_valid_root_lag_end = outcomes.last().map_or(0, |outcome| outcome.invariant_snapshot.oldest_valid_root_lag);

                        // Q9a：条目数（两盘合计不适用于本模型——落点跨两盘同槽，条目按 M0.1 只在单设备
                        // 视角下数一遍，见 `LedgerEntry`；这里报的是单份账本条目数）、条目字节、叶节点数。
                        let arm = ARM_ORDER[index];
                        let record_width_bytes = arm.record_width_bytes();
                        let records_per_node = arm.records_per_node();
                        let entries_series: Vec<u64> = outcomes.iter().map(|outcome| outcome.allocation_record_entries[index]).collect();
                        let derived_series: Vec<(u64, u64)> =
                            entries_series.iter().map(|entries| allocation_record_bytes_and_leaf_nodes(*entries, record_width_bytes, records_per_node)).collect();
                        let bytes_series: Vec<u64> = derived_series.iter().map(|(bytes, _)| *bytes).collect();
                        let leaf_nodes_series: Vec<u64> = derived_series.iter().map(|(_, leaf_nodes)| *leaf_nodes).collect();
                        let (entries_peak, entries_mean, entries_end) = peak_mean_end(&entries_series);
                        let (bytes_peak, bytes_mean, bytes_end) = peak_mean_end(&bytes_series);
                        let (leaf_nodes_peak, leaf_nodes_mean, leaf_nodes_end) = peak_mean_end(&leaf_nodes_series);
                        entries_peak_by_index[index] = entries_peak;
                        entries_mean_by_index[index] = entries_mean;

                        // Q9b：「发布前步」= 每次发布都有的回收步（`reclaim_step_reads`，登记第五节「回收步
                        // 的时点」第一句），逐条采样；「重建步」= 紧跟 `rebuild_allocator` / `raise_floor`
                        // 开头那次重算的采样点（`is_rebuild_step`），样本数可能为 0（这一族历史没有触发
                        // 挂载/切换/回退/抬F）。
                        let reclaim_reads_series: Vec<u64> = outcomes.iter().map(|outcome| outcome.reclaim_step_reads[index]).collect();
                        let (reclaim_reads_peak, reclaim_reads_mean, reclaim_reads_end) = peak_mean_end(&reclaim_reads_series);
                        let rebuild_shadow_reads_series: Vec<u64> =
                            outcomes.iter().filter(|outcome| outcome.is_rebuild_step).map(|outcome| outcome.shadow_recompute_reads[index]).collect();
                        let rebuild_step_count = rebuild_shadow_reads_series.len() as u64;
                        let (rebuild_shadow_reads_peak, rebuild_shadow_reads_mean, rebuild_shadow_reads_end) = peak_mean_end(&rebuild_shadow_reads_series);

                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=q_family family={family} s={} placement={:?} period={} arm={arm_tag} publishes={} \
                                 q1_i31_arm_red_states={allocated_mismatch_arm_reading_count} q2_i31_today_red_states={allocated_mismatch_today_reading_count} q3_i52_red_states={free_mismatch_count} \
                                 q7_g27_red_states={allocated_minus_deferred_reference_mismatch_count} \
                                 q4i_violations_candidate={} q4ii_violations_abandoned={} \
                                 q5_over_held_peak={over_held_peak} q5_over_held_positive_publishes={over_held_positive_publishes} q5_over_held_end={over_held_end} \
                                 q6_isolated_peak={isolated_peak} q6_isolated_positive_publishes={isolated_positive_publishes} q6_isolated_end={isolated_end} \
                                 q9a_entries_peak={entries_peak} q9a_entries_mean={entries_mean:.3} q9a_entries_end={entries_end} \
                                 q9a_bytes_peak={bytes_peak} q9a_bytes_mean={bytes_mean:.3} q9a_bytes_end={bytes_end} \
                                 q9a_leaf_nodes_peak={leaf_nodes_peak} q9a_leaf_nodes_mean={leaf_nodes_mean:.3} q9a_leaf_nodes_end={leaf_nodes_end} \
                                 q9b_publish_reclaim_reads_peak={reclaim_reads_peak} q9b_publish_reclaim_reads_mean={reclaim_reads_mean:.3} q9b_publish_reclaim_reads_end={reclaim_reads_end} \
                                 q9b_rebuild_shadow_reads_peak={rebuild_shadow_reads_peak} q9b_rebuild_shadow_reads_mean={rebuild_shadow_reads_mean:.3} q9b_rebuild_shadow_reads_end={rebuild_shadow_reads_end} q9b_rebuild_step_count={rebuild_step_count} \
                                 q11_lag_peak={oldest_valid_root_lag_peak} q11_lag_ge_n_publishes={oldest_valid_root_lag_at_or_above_ring_capacity_publishes} q11_lag_end={oldest_valid_root_lag_end} \
                                 q12_floor_only_on_one_disk_publishes={floor_only_on_one_disk_publishes} \
                                 interval_candidate_inference_mismatch_count={}",
                                cell.slots_per_region,
                                cell.placement,
                                cell.workload_period,
                                outcomes.len(),
                                world.arms[index].violations_candidate,
                                world.arms[index].violations_abandoned,
                                world.arms[index].interval_candidate_inference_mismatch_count,
                            ))
                        );

                        // Q6b：读法甲/乙上界（登记第六节 Q6b-甲/乙）——只在「这条历史已经回退过」时有
                        // 意义（`rollback_target_reference_count` 是 `Some`）；只要环里此刻有被抛弃根就采样一次
                        // （不要求这一刻恰好紧跟回退），逐格分别报「隔离 > 上界」与「上界 < 0」的发布数。
                        let mut abandoned_root_bound_sampled_publishes = 0u64;
                        let mut newest_abandoned_bound_exceeded_publishes = 0u64;
                        let mut maximum_abandoned_bound_exceeded_publishes = 0u64;
                        let mut newest_abandoned_bound_negative_publishes = 0u64;
                        let mut maximum_abandoned_bound_negative_publishes = 0u64;
                        for outcome in &outcomes {
                            let Some(rollback_target_reference_counts) = outcome.rollback_target_reference_count else { continue };
                            let (Some(newest), Some(maximum_value)) = outcome.abandoned_item1_bounds[index] else { continue };
                            abandoned_root_bound_sampled_publishes += 1;
                            let isolated = outcome.pre[index].isolated_slots;
                            let check = abandoned_root_bound_check(isolated, newest, maximum_value, rollback_target_reference_counts[index]);
                            if check.newest_bound_negative {
                                newest_abandoned_bound_negative_publishes += 1;
                            }
                            if check.maximum_bound_negative {
                                maximum_abandoned_bound_negative_publishes += 1;
                            }
                            if check.isolated_exceeds_newest_bound {
                                newest_abandoned_bound_exceeded_publishes += 1;
                            }
                            if check.isolated_exceeds_maximum_bound {
                                maximum_abandoned_bound_exceeded_publishes += 1;
                            }
                        }
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=q6b family={family} s={} placement={:?} period={} arm={arm_tag} \
                                 sampled={abandoned_root_bound_sampled_publishes} a_isolated_exceeds_bound_publishes={newest_abandoned_bound_exceeded_publishes} a_bound_negative_publishes={newest_abandoned_bound_negative_publishes} \
                                 b_isolated_exceeds_bound_publishes={maximum_abandoned_bound_exceeded_publishes} b_bound_negative_publishes={maximum_abandoned_bound_negative_publishes}",
                                cell.slots_per_region, cell.placement, cell.workload_period,
                            ))
                        );
                    }
                    // Q9a「多写」= G12 一侧 − 甲-T1 一侧（同影子账、同族同格），峰值与均值各一行；
                    // 另报其中归因于宽度的部分 = G12 条目数（峰值）× 8（20→28 字节多出的那一半，登记第六节）。
                    let threshold_conservative = World::arm_index(Arm::ThresholdConservative);
                    let threshold_narrow = World::arm_index(Arm::ThresholdNarrow);
                    let interval_conservative = World::arm_index(Arm::IntervalConservative);
                    let interval_narrow = World::arm_index(Arm::IntervalNarrow);
                    for (shadow_tag, interval_arm_index, threshold_index) in
                        [("conservative", interval_conservative, threshold_conservative), ("narrow", interval_narrow, threshold_narrow)]
                    {
                        let extra_write_peak = i64_from_u64(entries_peak_by_index[interval_arm_index]) - i64_from_u64(entries_peak_by_index[threshold_index]);
                        let extra_write_mean = entries_mean_by_index[interval_arm_index] - entries_mean_by_index[threshold_index];
                        let width_attributed_bytes_at_peak = entries_peak_by_index[interval_arm_index] * 8;
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=q9a_extra_write family={family} s={} placement={:?} period={} shadow={shadow_tag} \
                                 entries_peak_extra={extra_write_peak} entries_mean_extra={extra_write_mean:.3} width_attributed_bytes_at_peak={width_attributed_bytes_at_peak}",
                                cell.slots_per_region, cell.placement, cell.workload_period,
                            ))
                        );
                    }
                }
            }
        }
    }

    // 第五段：Q8 坏镜像矩阵——β1（H1 末态）与 β2（H4dNm2 回退后第 S 次工作负载之后）两个基底，
    // 全部 12 格、全部 5 条臂，B1a/B1b/B2 与 PC-F 的 Bk1/Bk2（登记第五节坏镜像、第九节 PC-F）。
    // B3a/B3b/B4a/B4b/B4c 要重新跑一遍历史（回收步一律不回收）或强制落点碰撞，本段未实现
    // （见跑前登记第十二节第五段「没做什么」）。
    #[allow(clippy::type_complexity, reason = "每一项就是（坏镜像名，造它的函数指针），只在这个函数里遍历，命名别名不会让哪一步更好验")]
    let bad_image_kinds: [(&str, fn(BaseImageState) -> BaseImageState); 5] =
        [("B1a", bad_image_b1a), ("B1b", bad_image_b1b), ("B2", bad_image_deferred_overcount_by_one), ("Bk1", bad_image_bk1), ("Bk2", bad_image_bk2)];
    for cell in all_geometry_cells() {
        let bases: [(&str, Option<World>); 2] = [("beta1", Some(beta_one_world(cell))), ("beta2", beta_two_world(cell))];
        for (base_tag, base_world) in bases {
            let Some(base_world) = base_world else {
                println!(
                    "{}",
                    emitter.emit_raw(&format!("name=q8_base base={base_tag} s={} placement={:?} period={} no_candidate=true", cell.slots_per_region, cell.placement, cell.workload_period))
                );
                continue;
            };
            let base_states = base_image_states(&base_world);
            for (index, arm_variant) in ARM_ORDER.iter().enumerate() {
                let arm_tag = arm_variant.tag();
                let base_checks = accounting_checks(base_states[index]);
                let base_already_red = base_checks != AccountingChecks::default();
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=q8_base_check base={base_tag} s={} placement={:?} period={} arm={arm_tag} \
                         allocated_statistic_mismatch_by_arm_reading={} allocated_statistic_mismatch_by_today_reading={} free_statistic_mismatch={} allocated_minus_deferred_red={}",
                        cell.slots_per_region,
                        cell.placement,
                        cell.workload_period,
                        base_checks.allocated_statistic_mismatch_by_arm_reading,
                        base_checks.allocated_statistic_mismatch_by_today_reading,
                        base_checks.free_statistic_mismatch,
                        base_checks.allocated_minus_deferred_red,
                    ))
                );
                for (image_tag, constructor) in bad_image_kinds {
                    if base_already_red {
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=q8 base={base_tag} image={image_tag} s={} placement={:?} period={} arm={arm_tag} base_already_red=true",
                                cell.slots_per_region, cell.placement, cell.workload_period
                            ))
                        );
                        continue;
                    }
                    let image_checks = accounting_checks(constructor(base_states[index]));
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=q8 base={base_tag} image={image_tag} s={} placement={:?} period={} arm={arm_tag} \
                             detected_by_allocated_statistic_arm_reading={} detected_by_allocated_statistic_today_reading={} detected_by_free_statistic={} detected_by_allocated_minus_deferred={}",
                            cell.slots_per_region,
                            cell.placement,
                            cell.workload_period,
                            image_checks.allocated_statistic_mismatch_by_arm_reading,
                            image_checks.allocated_statistic_mismatch_by_today_reading,
                            image_checks.free_statistic_mismatch,
                            image_checks.allocated_minus_deferred_red,
                        ))
                    );
                }
            }
        }
    }

    println!("{}", emitter.finish());
}
