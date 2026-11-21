use e7_index_bench::i64_from_u64;
use super::*;

/// Q10：区间推法的窗口是半开的 `[分配代, 释放代)`（登记第一节「G12」谓词逐字），
/// 两端各测一遍，同一条测试钉住两处边界。
#[test]
fn interval_inference_window_is_half_open_on_the_release_end() {
    let mut arm = ArmExecutor::new(Arm::IntervalNarrow, PlacementPolicy::Low);
    arm.ledger.insert(100, LedgerEntry { span_in_slots: 1, allocation_generation: 5, released: true, release_generation: 9 });
    assert!(!interval_inferred_candidate_references(&arm, &[9]).contains(&100), "释放代本身不在半开区间 [5, 9) 里");
    assert!(interval_inferred_candidate_references(&arm, &[8]).contains(&100), "释放代前一格在区间内");
    assert!(interval_inferred_candidate_references(&arm, &[5]).contains(&100), "分配代本身在区间内（闭区间左端）");
    assert!(!interval_inferred_candidate_references(&arm, &[4]).contains(&100), "分配代之前不在区间内");
}

/// K2：条目宽度。
#[test]
fn record_width_bytes_match_registered_values() {
    assert_eq!(Arm::ThresholdConservative.record_width_bytes(), 20);
    assert_eq!(Arm::Baseline.record_width_bytes(), 20);
    assert_eq!(Arm::IntervalConservative.record_width_bytes(), 28);
    assert_eq!(Arm::IntervalNarrow.record_width_bytes(), 28);
}

/// M0.9（不在登记第九节的 U 编号里；S1 对拍之前主 agent 点名核实、由执行员补的单测）：
/// mkfs 是实例 0、不写行；首次挂载取实例 1、不写行（登记「mkfs 实例 0；首次挂载实例 1，不写行」），
/// 与 `crates/singlefs-core/src/mount.rs` 785-811 `establish_instance` 对只做过 mkfs 的池
/// 「取号 1、不写行」一致。改坏点：`first_mount_warmup` 若不取号，`current_instance` 停在 mkfs
/// 的 0，本测试第二条 `assert_eq!` 会从 `1` 变红成 `0`（已实测：注释掉取号那两行，此断言失败）。
#[test]
fn first_mount_warmup_acquires_instance_one_without_writing_a_row() {
    let mut world = World::mkfs(BASE_GEOMETRY);
    assert_eq!(world.pool.current_instance, 0, "mkfs 是实例 0");
    assert!(world.pool.instance_table_rows.is_empty(), "mkfs 不写行");
    for _ in world.first_mount_warmup() {
    }
    assert_eq!(world.pool.current_instance, 1, "首次挂载取实例 1（M0.9）");
    assert!(world.pool.instance_table_rows.is_empty(), "首次挂载不写行（M0.9：上一个实例是 0、要写的行为空）");
}

fn run_continuous_overwrite_history(cell: GeometryCell) -> (World, Vec<PublishOutcome>) {
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {
    }
    let ring_capacity = cell.ring_capacity();
    let mut outcomes = Vec::new();
    for outcome in world.run_workload_segment(6 * ring_capacity, cell.workload_period) {
        outcomes.push(outcome);
    }
    (world, outcomes)
}

/// A4：H1 主历史最后一次 O 那一行，测量臂（甲-T1、G12 四条）的记账三项与该行 txg。
/// 三档 S 的数值取自登记第十三节 [C4] 的原样输出（独立算出、用命令核过，K2 的第二类锚点）。
#[test]
fn steady_state_accounting_matches_closed_form() {
    let cases: [(u64, u64, u64, u64, u64, u64); 6] = [
        // (S, period, expected_item1, expected_item5, expected_row_txg, _unused)
        (4, 1, 122, 110, 74, 0),
        (4, 4, 74, 62, 71, 0),
        (8, 1, 242, 230, 146, 0),
        (8, 4, 140, 128, 143, 0),
        (16, 1, 482, 470, 290, 0),
        (16, 4, 272, 260, 287, 0),
    ];
    for (slots_per_region, workload_period, expected_item1, expected_item5, expected_row_txg, _) in cases {
        let cell = GeometryCell { slots_per_region, placement: PlacementPolicy::Impl, workload_period };
        let (_world, outcomes) = run_continuous_overwrite_history(cell);
        let last_overwrite = outcomes.iter().rev().find(|outcome| outcome.is_nonempty).expect("H1 至少有一次 O");
        assert_eq!(last_overwrite.txg, expected_row_txg, "S={slots_per_region} period={workload_period}");
        for index in ARM_ORDER.iter().enumerate().filter(|(_, arm)| arm.is_measured()).map(|(index, _)| index) {
            let row = last_overwrite.accounting[index];
            assert_eq!(row.item1_occupied_slots, expected_item1, "{:?} S={slots_per_region} period={workload_period}", ARM_ORDER[index]);
            assert_eq!(row.item5_deferred_slots, expected_item5, "{:?} S={slots_per_region} period={workload_period}", ARM_ORDER[index]);
            assert_eq!(row.item2_free_slots, UNIT_AREA_CAPACITY_SLOTS - expected_item1, "{:?}", ARM_ORDER[index]);
        }
    }
}

/// A5：基线在 H1、ρ=1 下，主历史最后一次发布之前那一刻的多扣（不回收，单调涨）。
#[test]
fn baseline_over_held_grows_unboundedly_without_per_publish_reclaim() {
    for (slots_per_region, expected) in [(4u64, 591u64), (8, 1191), (16, 2391)] {
        let cell = GeometryCell { slots_per_region, placement: PlacementPolicy::Impl, workload_period: 1 };
        let mut world = World::mkfs(cell);
        for _ in world.first_mount_warmup() {
        }
        let ring_capacity = cell.ring_capacity();
        let mut last_pre = None;
        for outcome in world.run_workload_segment(6 * ring_capacity, 1) {
            last_pre = Some(outcome.pre[World::arm_index(Arm::Baseline)]);
        }
        let baseline_pre = last_pre.expect("H1 至少一次发布");
        assert_eq!(baseline_pre.over_held_slots, i64_from_u64(expected), "S={slots_per_region}");
        assert_eq!(baseline_pre.isolated_slots, 0);
    }
}

/// U1（M3 的锚点）：门槛用 max(F_生效, 最旧有效根)，不能只用最旧有效根。
/// 环 txg 10..33 连续有效、F_生效 20，释放代 15 的记录：门槛应是 20，原式可再分配。
#[test]
fn threshold_predicate_uses_maximum_of_floor_and_oldest_valid_root() {
    let mut pool = PoolState::new(8);
    for txg in 10..=33u64 {
        pool.instance_table_rows.clear();
        pool.commit_root(txg, 0, 20, false);
    }
    let threshold = pool.reclaim_threshold();
    assert_eq!(threshold, 20, "F_生效 20 > 最旧有效根 10");
    let record = pending_record(0, 0, 15);
    assert!(is_record_reclaimable(Predicate::ThresholdFloor, threshold, None, &record), "释放代 15 ≤ 门槛 20，原式可再分配");
    // M3 的变异：门槛丢 F，只用最旧有效根 10 —— 15 ≤ 10 为假，应当抓到。
    assert!(!is_record_reclaimable(Predicate::ThresholdFloor, 10, None, &record), "变异门槛 10 下 15 ≤ 10 为假");
}

/// U2（M4 的锚点）：G12 区间是半开 [分配代, 释放代)，不是闭区间。
/// 分配代 5、释放代 9、环 {9, 10}，原式可再分配（9 不在 [5,9) 里）。
#[test]
fn interval_predicate_is_half_open() {
    let blocking: BTreeSet<u64> = [9u64, 10].into_iter().collect();
    let record = pending_record(0, 5, 9);
    assert!(is_record_reclaimable(Predicate::IntervalBlocking, 0, Some(&blocking), &record), "9 不在半开区间 [5,9) 里，原式可再分配");
    // M4 的变异：区间闭合成 [5,9]，9 落进去，应当抓到。
    let closed_blocking_hit = blocking.range(5..=9).next().is_some();
    assert!(closed_blocking_hit, "闭区间下 9 会落进 [5,9]，变异应当不可再分配");
}

/// K4：全部 12 格，H4 回退那次 W 的 txg = 3N + 3。
#[test]
fn rollback_write_row_txg_matches_registered_formula() {
    for slots_per_region in [4u64, 8, 16] {
        for placement in [PlacementPolicy::Impl, PlacementPolicy::Low] {
            for workload_period in [1u64, 4] {
                let cell = GeometryCell { slots_per_region, placement, workload_period };
                let mut world = World::mkfs(cell);
                for _ in world.first_mount_warmup() {
                }
                let ring_capacity = cell.ring_capacity();
                for _ in world.run_workload_segment(3 * ring_capacity, workload_period) {
                }
                let newest_txg = world.pool.current_txg;
                let target_ceiling = newest_txg - 2; // d = 2
                let target_txg = world.pool.candidate_roots().iter().map(|entry| entry.txg).filter(|txg| *txg <= target_ceiling).max().expect("候选集里必有一条 ≤ target_ceiling");
                let outcomes = world.rollback_to(target_txg);
                let write_row_txg = outcomes.iter().find(|outcome| outcome.label == "write_row").map(|outcome| outcome.txg).expect("回退必产出一次写行发布");
                let expected_write_row_txg = 3 * ring_capacity + 3;
                assert_eq!(write_row_txg, expected_write_row_txg, "S={slots_per_region} placement={placement:?} period={workload_period}");
                // A2：warmup_until_both_devices() 真实产出的暖机次数要等于「按 txg mod 3 推」的公式
                // （这条断言是 warmup_stops_after_one_device 那条变异唯一的锚点：只手算 A2 的公式不
                // 会调用真实函数，抓不住把循环条件从 < 2 改成 < 1 这类破坏）。
                let warm_up_publish_count = outcomes.iter().filter(|outcome| outcome.label == "empty").count() as u64;
                let mut covered_after_write_row: BTreeSet<Device> = BTreeSet::new();
                covered_after_write_row.insert(root_ring_device(write_row_txg));
                let mut expected_warm_up_publish_count = 0u64;
                let mut probe_txg = write_row_txg;
                while covered_after_write_row.len() < 2 {
                    probe_txg += 1;
                    covered_after_write_row.insert(root_ring_device(probe_txg));
                    expected_warm_up_publish_count += 1;
                }
                assert_eq!(warm_up_publish_count, expected_warm_up_publish_count, "S={slots_per_region} placement={placement:?} period={workload_period}");
            }
        }
    }
}

/// K5：ρ = 1 的 6 格，H6 抬 F 那一刻：目标 F = 3N − 1（35 / 71 / 143），带新 F 的空发布推 2 次。
#[test]
fn raise_floor_target_and_publish_count_match_registered_values() {
    for slots_per_region in [4u64, 8, 16] {
        for placement in [PlacementPolicy::Impl, PlacementPolicy::Low] {
            let cell = GeometryCell { slots_per_region, placement, workload_period: 1 };
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {
            }
            let ring_capacity = cell.ring_capacity();
            for _ in world.run_workload_segment(3 * ring_capacity, 1) {
            }
            let outcomes = world.raise_floor();
            let expected_target_floor = 3 * ring_capacity - 1;
            assert_eq!(world.pool.current_rollback_floor, expected_target_floor, "S={slots_per_region} placement={placement:?}");
            assert_eq!(outcomes.len(), 2, "带新 F 的空发布推 2 次：S={slots_per_region} placement={placement:?}");
        }
    }
}

/// S1(b)（登记第十一节停机条款 S1，`plain_remount` 补的驱动）：A(txg3,inst1)、B(txg4,inst1，实例内覆盖写)、
/// 普通挂载取号 2（写行 txg5、暖机 txg6-7）、C(txg8,inst2)、回退到 A 的根 (1,3)。
/// 对拍目标（`crates/singlefs-verification/tests/rollback_by_a_forward_publish.rs` 已断言过的真实值，
/// 逐字段见该文件 `rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content`
/// 190-198 行与 `rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation`
/// 223-235 行）：回退发布（D）落 txg 9、实例 3；写的行 = [(1,3),(2,0)]（回退行 + 中间实例，(instance,T) 分量）；
/// 基线臂（「今」，Arm::Baseline）隔离槽数每盘 34。
/// `ArmFullSnapshot` 不携带 `root_accounts`，`restore_full` 不覆盖它：带上它会把 `root_accounts` 恢复到回退目标（A，txg3）那一刻的快照，
/// `recompute_shadow` 查 `abandoned_extended=[5,6,7,8]`（写行、暖机×2、C）时全部落空、隔离集恒空，隔离槽数为 0。
/// `root_accounts` 按自己的注释该一直反映「每条仍在环里的根」，由 `World::prune_evicted_accounts` 独立维护，不随回退时光倒流。
/// 该一直反映「每条仍在环里的根」，由 `World::prune_evicted_accounts` 独立维护，不随回退时光倒流。
#[test]
fn pairing_probe_segment_rollback_matches_real_implementation() {
    let cell = BASE_GEOMETRY;
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {
    }
    let first_overwrite = world.publish(PublishKind::Overwrite);
    assert_eq!(first_overwrite.txg, 3, "第一次覆盖写落 txg 3");
    let second_overwrite = world.publish(PublishKind::Overwrite);
    assert_eq!(second_overwrite.txg, 4, "第二次覆盖写落 txg 4，同实例覆盖写");
    let remount_outcomes = world.plain_remount();
    let write_row = remount_outcomes.iter().find(|outcome| outcome.label == "write_row").expect("普通挂载必有写行发布");
    assert_eq!(write_row.txg, 5, "写行发布落 txg 5");
    assert_eq!(write_row.instance, 2, "取号 2");
    let warm_up_count = remount_outcomes.iter().filter(|outcome| outcome.label == "empty").count();
    assert_eq!(warm_up_count, 2, "暖机两次覆盖两块盘");
    let third_overwrite = world.publish(PublishKind::Overwrite);
    assert_eq!(third_overwrite.txg, 8, "第三次覆盖写落 txg 8");
    assert_eq!(third_overwrite.instance, 2);

    let rollback_outcomes = world.rollback_to(3);
    let rollback_write_row = rollback_outcomes.iter().find(|outcome| outcome.label == "write_row").expect("回退必有写行发布");
    assert_eq!(rollback_write_row.txg, 9, "回退写行落 txg 9（真实实现 rollback_publish.root.checkpoint_txg 同为 9）");
    assert_eq!(rollback_write_row.instance, 3, "回退取实例 3");
    assert_eq!(
        world.pool.instance_table_rows,
        vec![(1, 3), (2, 0)],
        "写的行 = 回退行 (1,3) + 中间实例行 (2,0)（真实实现 rows_written 的 (instance,selected_root_txg) 分量同为这两条）"
    );

    let baseline_index = World::arm_index(Arm::Baseline);
    let isolated = world.arms[baseline_index].isolated.len();
    assert_eq!(isolated, 34, "基线臂隔离槽数每盘 34（真实实现 isolated_slots_per_device 同为每盘 34）");
}

/// 对拍段落后半（登记第十一节停机条款 S1）：在上一个对拍测试的回退之后再覆盖写四次（实例 3），
/// 抬 F 到第一次释放代（=第一次覆盖写的 txg），比对回收的落点集合与之后一次复用的落点。
/// 对拍目标（`crates/singlefs-verification/tests/reuse_after_raising_the_floor.rs`
/// `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`
/// 144-192 行）：抬 F 上限 = 第一次覆盖写的 txg；两次带新 F 的空发布；复用发布落在最低可再分配的偶数槽对。
#[test]
fn pairing_probe_segment_raise_floor_and_reuse_matches_real_implementation() {
    let cell = BASE_GEOMETRY;
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {
    }
    let _first_overwrite = world.publish(PublishKind::Overwrite); // txg 3
    let _second_overwrite = world.publish(PublishKind::Overwrite); // txg 4
    let _remount = world.plain_remount(); // txg 5-7，实例 2
    let _third_overwrite = world.publish(PublishKind::Overwrite); // txg 8
    let _rollback = world.rollback_to(3); // txg 9-10，实例 3
    let baseline_index = World::arm_index(Arm::Baseline);
    // 「每次发布的记账三项」（S1(c) 条件文本）：与真实实现 `reuse_after_raising_the_floor.rs`
    // 重跑打印的 E153S1 tag=c_overwrite/c_after_raise/c_reuse 逐字节一致（第三段 S1 对拍，草稿目录
    // `/tmp/claude-1000/e153-runner-stage3/repo-copy/`，探针 `e153_s1_pairing_probe.rs` 段 c）。
    let expected_after_txg: [(u64, u64, u64, u64); 4] = [(11, 33, 211935, 21), (12, 43, 211925, 31), (13, 53, 211915, 41), (14, 63, 211905, 51)];
    for (expected_txg, expected_allocated, expected_free, expected_deferred) in expected_after_txg {
        let outcome = world.publish(PublishKind::Overwrite);
        assert_eq!(outcome.txg, expected_txg, "回退之后连续覆盖写的 txg 序列");
        let row = outcome.accounting[baseline_index];
        assert_eq!(row.item1_occupied_slots, expected_allocated, "txg={expected_txg} 第 1 项");
        assert_eq!(row.item2_free_slots, expected_free, "txg={expected_txg} 第 2 项");
        assert_eq!(row.item5_deferred_slots, expected_deferred, "txg={expected_txg} 第 5 项");
    }
    assert_eq!(world.pool.current_txg, 14, "回退之后四次覆盖写落 txg 11-14");

    let raise_outcomes = world.raise_floor();
    assert_eq!(world.pool.current_rollback_floor, 11, "抬 F 上限 = 第一次覆盖写的 txg 11（真实实现 raised.ceiling 同为 11）");
    assert_eq!(raise_outcomes.len(), 2, "两次带新 F 的空发布（真实实现 raised.publishes 同为两条，txg 15/16）");
    let after_raise = raise_outcomes.last().expect("两次空发布，至少一条").accounting[baseline_index];
    assert_eq!(after_raise.item1_occupied_slots, 50, "抬 F 生效之后（txg 16）第 1 项：真实实现 c_after_raise allocated=50");
    assert_eq!(after_raise.item2_free_slots, 211918, "抬 F 生效之后（txg 16）第 2 项：真实实现 c_after_raise free=211918（本段 raise_floor() 的 baseline_free_slots 修法要保这一行）");
    assert_eq!(after_raise.item5_deferred_slots, 38, "抬 F 生效之后（txg 16）第 5 项：真实实现 c_after_raise deferred=38");

    let reuse = world.publish(PublishKind::Overwrite); // txg 17
    assert_eq!(reuse.txg, 17, "复用发布落 txg 17（与真实实现 reuse.root.checkpoint_txg 同为 17）");
    let reuse_row = reuse.accounting[baseline_index];
    assert_eq!(reuse_row.item1_occupied_slots, 60, "复用发布之后第 1 项：真实实现 c_reuse allocated=60");
    assert_eq!(reuse_row.item2_free_slots, 211908, "复用发布之后第 2 项：真实实现 c_reuse free=211908");
    assert_eq!(reuse_row.item5_deferred_slots, 48, "复用发布之后第 5 项：真实实现 c_reuse deferred=48");
    let reused_slot = world.arms[baseline_index].refs[&Role::DataUnit];
    assert_eq!(
        reused_slot, 50178,
        "复用发布把数据单元落回 50178：mkfs 树表那 1 槽回收了、50179 从没分配过；mkfs 实例表那片 50176 也回收了但 B 的根还引用它、\
             影子账隔离着（真实实现 `reuse_after_raising_the_floor.rs` 文件头注释第 3 行同为 50178；\
             2026-09-17 第三段发现并修：raise_floor() 此前没有重算影子账，模型曾给出 50176，见跑前登记第十二节）"
    );
    let isolated = world.arms[baseline_index].isolated.contains(&50176);
    assert!(isolated, "mkfs 实例表片 50176 仍被 B 的根引用，应被影子账隔离（真实实现同一测试文件头注释）");
    let reused_entry = world.arms[baseline_index].ledger.get(&reused_slot).expect("复用的槽必有账目条目");
    println!(
        "S1C reuse_data_unit_slot={reused_slot} reused_record_allocation_generation={} reused_record_released={}",
        reused_entry.allocation_generation, reused_entry.released
    );
}

/// 回归测试：`rebuild_allocator` 的基线分支漏减 `baseline_free_slots`
/// 那一处 bug（见该函数的文档注释）——普通挂载重建之后，item1+item2 必须仍等于容量。
/// S=4 时 3N=36 次连续覆盖写让环转三圈，released-但未达门槛的槽有 110 个，漏减时 sum 会多 110。
#[test]
fn rebuild_allocator_keeps_baseline_item1_plus_item2_at_capacity_across_remount() {
    let cell = GeometryCell { slots_per_region: 4, placement: PlacementPolicy::Impl, workload_period: 1 };
    let mut world = World::mkfs(cell);
    for _ in world.first_mount_warmup() {
    }
    let ring_capacity = cell.ring_capacity();
    for outcome in world.run_workload_segment(3 * ring_capacity, 1) {
        let row = outcome.accounting[World::arm_index(Arm::Baseline)];
        assert_eq!(row.item1_occupied_slots + row.item2_free_slots, UNIT_AREA_CAPACITY_SLOTS, "重开取号前 txg={}", outcome.txg);
    }
    for outcome in world.plain_remount() {
        let row = outcome.accounting[World::arm_index(Arm::Baseline)];
        assert_eq!(
            row.item1_occupied_slots + row.item2_free_slots,
            UNIT_AREA_CAPACITY_SLOTS,
            "重建（普通挂载）之后 label={} txg={}",
            outcome.label,
            outcome.txg
        );
    }
    for outcome in world.run_workload_segment(3 * ring_capacity, 1) {
        let row = outcome.accounting[World::arm_index(Arm::Baseline)];
        assert_eq!(row.item1_occupied_slots + row.item2_free_slots, UNIT_AREA_CAPACITY_SLOTS, "重开取号后 txg={}", outcome.txg);
    }
}

/// 第三段冒烟：新增的六族历史函数在全部 12 格上都跑得完、不 panic，且落地一批基本形状——
/// 每族至少推进了 txg、Q3（I-5.2）全程不红（模型构造的两条累加路径应当恒等，PC-G 的变异改的是
/// 「让它不恒等」那一条独立路径，不影响正常跑）。
#[test]
fn all_history_families_run_to_completion_on_every_geometry_cell() {
    for cell in all_geometry_cells() {
        {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let outcomes = world.history_plain_remount_midway();
            assert!(!outcomes.is_empty(), "H2 s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period);
            for outcome in &outcomes {
                for index in 0..5 {
                    assert!(!outcome.invariant_snapshot.free_statistic_mismatch[index], "H2 I-5.2 红：s={} txg={} arm_index={index}", cell.slots_per_region, outcome.txg);
                }
            }
        }
        for phi in [0u64, 1, 2] {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let outcomes = world.history_root_slot_write_failure_and_switch(phi, false);
            assert!(!outcomes.is_empty(), "H3phi={phi} s={}", cell.slots_per_region);
        }
        {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let outcomes = world.history_root_slot_write_failure_and_switch(0, true);
            assert!(!outcomes.is_empty(), "H3^2 s={}", cell.slots_per_region);
            assert!(
                outcomes.iter().filter(|outcome| outcome.label == "write_row_root_slot_write_failure").count() >= 1,
                "H3^2 必须先看到写行也失败一次：s={}",
                cell.slots_per_region
            );
        }
        for rollback_depth in [2u64, cell.ring_capacity() - 2] {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let result = world.history_single_rollback(rollback_depth);
            assert!(result.is_some(), "H4 rollback_depth={rollback_depth} 不该无候选：s={} N={}", cell.slots_per_region, cell.ring_capacity());
        }
        // H5早/H5晚/H6 的第二次回退目标理论上仍可能因几何极端而不在候选集里——第十一节 V8「写死的
        // 回退目标不存在时记『无候选』，不计入任何量」，`None` 是合法结果，不是 panic 条件；这里只
        // 要求「调用不 panic」。**实测（第四段）在全部 12 格上两族都返回 `Some`**，见
        // `double_rollback_families_find_second_rollback_candidate_on_every_geometry_cell` 那条更强的断言；
        // Q13 的条数统计留给第三段主报告的族历史扫描（main()）。
        {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let _ = world.history_double_rollback_shallower_second();
        }
        {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let _ = world.history_double_rollback_to_first_rollback_write_row();
        }
        {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let _ = world.history_raise_floor_then_rollback();
        }
        {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let _ = world.history_rollback_deep_then_raise_floor();
        }
    }
}

/// 回归测试（见跑前登记第十二节）：`history_double_rollback_shallower_second`
/// 与 `history_double_rollback_to_first_rollback_write_row` 用不含尾段的 `history_rollback_prefix` 当「H4（d=S）」前缀：
/// 借 `history_single_rollback` 会把 H4d 自己的尾段工作负载段 3N 也借进去——环转过好几圈之后第一次回退的
/// 目标早被轮转挤出候选集，第二次回退在全部 12 格上都找不到候选，H5早 / H5晚 24 行 q_family 只有
/// `q13_no_candidate=true`，5 条臂一次都没跑起来。
/// 用 `history_rollback_prefix`，两族历史必须在全部 12 格上都跑出两次回退（各一条 `label == "write_row"`）。
#[test]
fn double_rollback_families_find_second_rollback_candidate_on_every_geometry_cell() {
    for cell in all_geometry_cells() {
        {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let outcomes = world.history_double_rollback_shallower_second();
            let outcomes = outcomes.unwrap_or_else(|| {
                panic!("H5早 不该无候选：s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period)
            });
            let write_row_count = outcomes.iter().filter(|outcome| outcome.label == "write_row").count();
            assert_eq!(write_row_count, 2, "H5早 必须两次回退各一条写行发布：s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period);
        }
        {
            let mut world = World::mkfs(cell);
            for _ in world.first_mount_warmup() {}
            let outcomes = world.history_double_rollback_to_first_rollback_write_row();
            let outcomes = outcomes.unwrap_or_else(|| {
                panic!("H5晚 不该无候选：s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period)
            });
            let write_row_count = outcomes.iter().filter(|outcome| outcome.label == "write_row").count();
            assert_eq!(write_row_count, 2, "H5晚 必须两次回退各一条写行发布：s={} placement={:?} period={}", cell.slots_per_region, cell.placement, cell.workload_period);
        }
    }
}
