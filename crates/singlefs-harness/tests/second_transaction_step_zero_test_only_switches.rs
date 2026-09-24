//! 里程碑「第二个事务」步 0 的「只供测试的开关」这一段（`.claude/rules/fs-design.md` 五条硬要求第 2 条：
//! 每条分支必须能被测试强制进入，**进不去就等于没测**）里的两个：复用窗口置 0、按 (区域, 槽) 让根槽读返回失败。
//!
//! 每个开关一条用例，两条断言并排比：关掉时走哪一条、打开时走哪一条，比的是可观测的读数（落点、错误成员、计数），
//! 不是「没 panic」（步 0 验收第 81 行：「开关关着走 X、开着走 Y，运行时报出的分支名不同」）。
//!
//! 开关自己的用处不在这个文件里：复用窗口置 0 撑的是必红八条第四条（C22（刚释放的块立即重分配），同在这个文件，
//! 见 `reuse_window_forced_to_zero_...`）；根槽读失败撑的是必红八条第八条的 ② 格（在
//! `second_transaction_step_four_rollback.rs`），以及 C332（回退实例两个根都读不出时回退被撤销）、
//! C335（根槽持续读不出时实例表只增不减）、C393（被抛弃根的账读不出时修复没有条款）。

mod common;

use common::{build_pool, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::ReuseWindow;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, readable_roots, recover, JournalPolicy, PoolReader,
    RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, NamedRootRingSlots,
    PoolReaderWithUnreadableRootRingSlots, RootRingSlotTarget, SharedFaultPlan,
};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 一次跑的可观测读数：开关自己报的分支名、它在 `release` 里当场回收掉的落点数、两次覆盖写的数据落点、
/// 第一个文件那一版（txg 3，F = 0 时仍是候选根）的数据落点、checker 判红的不变量。
struct ReuseWindowRun {
    branch_name: &'static str,
    reclaimed_on_release: u64,
    first_file_data_slot: SlotNumber,
    overwrite_data_slots: [SlotNumber; 2],
    violated_invariants: Vec<String>,
}

/// 同一条固定脚本跑一遍：mkfs → 第一个事务（txg 3）→ 覆盖写两次（txg 4、5），复用窗口按入参装。
fn run_two_overwrites_with(reuse_window: ReuseWindow, tag: &str) -> ReuseWindowRun {
    let mut pool = build_pool(tag);
    pool.allocator.set_reuse_window(reuse_window);
    let first_file_data_slot = pool.output.data_pointers[0].locations[0].slot;
    let fourth = overwrite_in_process(&mut pool, &content_of(3100, 3), InstanceGeneration(1));
    let fifth = overwrite_in_process(&mut pool, &content_of(2900, 7), InstanceGeneration(1));
    assert_eq!(fourth.root.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(fifth.root.checkpoint_txg, CheckpointTxg(5));
    let verdicts = check_pool_image(&pool.memory_pool());
    ReuseWindowRun {
        branch_name: pool.allocator.reuse_window().name(),
        reclaimed_on_release: pool
            .allocator
            .placements_reclaimed_on_release_by_the_forced_zero_reuse_window(),
        first_file_data_slot,
        overwrite_data_slots: [
            fourth.data_pointers[0].locations[0].slot,
            fifth.data_pointers[0].locations[0].slot,
        ],
        violated_invariants: verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| (*name).to_string())
            .collect(),
    }
}

/// 开关一：复用窗口置 0，证明它改变了走的分支。同一条脚本两遍，比的是**落点**：
/// 关着（产品路径）时释放的槽要等 F 抬到释放代之上才可再分配，两次覆盖写都往前拿新槽；
/// 开着时释放那一刻就回收，txg 5 那次覆盖写落回 txg 3 那一版的数据槽。
#[test]
fn the_forced_zero_reuse_window_switch_makes_the_second_overwrite_land_on_a_slot_that_the_gated_window_leaves_alone(
) {
    let gated = run_two_overwrites_with(
        ReuseWindow::GatedByTheRollbackFloor,
        "switch-reuse-window-gated",
    );
    let forced = run_two_overwrites_with(ReuseWindow::ForcedToZero, "switch-reuse-window-forced");

    assert_eq!(
        gated.branch_name, "reuse_window_gated_by_the_rollback_floor",
        "关着时运行时报的分支名"
    );
    assert_eq!(
        forced.branch_name, "reuse_window_forced_to_zero",
        "开着时运行时报的分支名"
    );
    assert_eq!(
        gated.reclaimed_on_release, 0,
        "关着时 release 里一个落点都不回收"
    );
    assert!(
        forced.reclaimed_on_release > 0,
        "开着时 release 里当场回收：{}",
        forced.reclaimed_on_release
    );
    assert_eq!(
        gated.first_file_data_slot, forced.first_file_data_slot,
        "两遍是同一条脚本：第一个文件那一版的数据落点相同"
    );
    assert_ne!(
        gated.overwrite_data_slots, forced.overwrite_data_slots,
        "开关改了走的分支：两遍的覆盖写落点必须不同"
    );
    assert!(
        gated
            .overwrite_data_slots
            .iter()
            .all(|slot| *slot > gated.first_file_data_slot),
        "关着时两次覆盖写都往前拿新槽，一个都不落回已释放的槽：{:?}",
        gated.overwrite_data_slots
    );
    assert!(
        gated.overwrite_data_slots[0] < gated.overwrite_data_slots[1],
        "关着时槽号只往前走：{:?}",
        gated.overwrite_data_slots
    );
    assert_eq!(
        forced.overwrite_data_slots[0], forced.overwrite_data_slots[1],
        "开着时同一个槽连着发两次——txg 4 那次的数据单元在 txg 5 那次里被释放、当场回收、又发了出去；\
         这件事在关着的那一遍里做不到"
    );
    assert!(
        forced.overwrite_data_slots[0] < forced.first_file_data_slot,
        "开着时拿到的是槽号更低的那个已释放槽（mkfs 那一版的树表槽），关着时它一直占着：{:?} vs {:?}",
        forced.overwrite_data_slots,
        forced.first_file_data_slot
    );
    // 绝对值钉住这条脚本上的几何（字节表五：50176–50177 是 mkfs 实例表、50178 是 mkfs 树表、50179 从没分配过）。
    assert_eq!(
        (
            gated.first_file_data_slot,
            gated.overwrite_data_slots,
            forced.overwrite_data_slots
        ),
        (
            SlotNumber(50180),
            [SlotNumber(50182), SlotNumber(50184)],
            [SlotNumber(50178), SlotNumber(50178)]
        ),
        "这条脚本上的落点"
    );
}

/// 必红八条第四条（复用窗口置 0，C22（刚释放的块立即重分配））：**走正常的发布路径**把复用造出来——
/// 开关装在分配器上，释放与再分配都由 `publish_overwrite` 自己做，用例一次都不调回收。
/// F = 0 ⇒ txg 3 那条根还在回退候选集里，而它引用的数据单元已经被 txg 5 那一版盖掉 ⇒
/// checker 从那条根走进去，校验和对不上（I-2.1）、近 K 代根不自洽（I-4.8）。
/// 关着时同一条脚本全绿：判别力自证就是这两遍的差。
#[test]
fn reuse_window_forced_to_zero_lets_a_publish_overwrite_a_unit_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    let gated = run_two_overwrites_with(
        ReuseWindow::GatedByTheRollbackFloor,
        "must-red-reuse-window-gated",
    );
    assert!(
        gated.violated_invariants.is_empty(),
        "关着时 checker 全绿：{:?}",
        gated.violated_invariants
    );

    let forced = run_two_overwrites_with(ReuseWindow::ForcedToZero, "must-red-reuse-window-forced");
    assert!(
        forced.violated_invariants.contains(&"I-2.1".to_string()),
        "被盖掉的那片数据的校验和与父指针对不上：{:?}",
        forced.violated_invariants
    );
    assert!(
        forced.violated_invariants.contains(&"I-4.8".to_string()),
        "从候选根出发的遍历要红（C22（刚释放的块立即重分配） 第 ② 条点名 I-4.8）：{:?}",
        forced.violated_invariants
    );
}

/// 一个池的根环几何：三个区域各住哪块盘、槽距多少，从系统配置里现读（不写死）。
fn root_ring_slot_target(
    image: &MemoryPool,
    named_slots: NamedRootRingSlots,
) -> RootRingSlotTarget {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    RootRingSlotTarget {
        named_slots,
        region_devices: system_configuration.immutable.region_devices,
        fixed_structure_slot_spacing: system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    }
}

/// 从读者那一侧看到的根环：择到的根是哪一版、环里读得出几条根。
fn roots_seen_by<Reader: PoolReader>(
    reader: &Reader,
    image: &MemoryPool,
) -> (Option<(InstanceGeneration, CheckpointTxg)>, usize) {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let chosen =
        choose_root(reader, &system_configuration).map(|root| (root.instance, root.checkpoint_txg));
    let readable = readable_roots(
        reader,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .len();
    (chosen, readable)
}

/// 开关二（读者那一侧的口子）：按 (区域, 槽) 让根槽读返回失败，证明它改变了走的分支。
/// 同一份镜像三遍并排比：不装开关 → 择到最新的根；点名最新那条根的槽 → 择到上一条根、环里少一条；
/// 点名整环 24 个槽 → 一条根都择不到，恢复报 `NoValidRoot` 这个错误成员。
#[test]
fn naming_a_root_ring_slot_makes_the_recovery_choose_a_different_root_and_naming_the_whole_ring_leaves_it_without_one(
) {
    let mut pool = build_pool("switch-root-slot-read-fails");
    overwrite_in_process(&mut pool, &content_of(3100, 3), InstanceGeneration(1));
    let image = pool.memory_pool();

    let (chosen_without_the_switch, readable_without_the_switch) = roots_seen_by(&image, &image);
    assert_eq!(
        chosen_without_the_switch,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "不装开关：择到最新的根"
    );

    let newest_slot = target_for_publish(
        CheckpointTxg(4),
        parameters().geometry.root_ring_slots_per_region,
    );
    let one_slot = PoolReaderWithUnreadableRootRingSlots::new(
        &image,
        root_ring_slot_target(&image, NamedRootRingSlots::NONE.with(newest_slot)),
    );
    let (chosen_with_one_slot, readable_with_one_slot) = roots_seen_by(&one_slot, &image);
    assert_eq!(
        chosen_with_one_slot,
        Some((InstanceGeneration(1), CheckpointTxg(3))),
        "点名 txg 4 那个槽：择到上一条根"
    );
    assert_eq!(
        readable_with_one_slot,
        readable_without_the_switch - 1,
        "环里读得出的根少一条"
    );
    assert!(
        one_slot.reads_refused() >= 1,
        "点名的槽上拦下过读：{}",
        one_slot.reads_refused()
    );

    let whole_ring = PoolReaderWithUnreadableRootRingSlots::new(
        &image,
        root_ring_slot_target(
            &image,
            NamedRootRingSlots::every_slot_in_the_ring(
                parameters().geometry.root_ring_slots_per_region,
            ),
        ),
    );
    let (chosen_with_the_whole_ring, readable_with_the_whole_ring) =
        roots_seen_by(&whole_ring, &image);
    assert_eq!(chosen_with_the_whole_ring, None, "整环点名：一条根都择不到");
    assert_eq!(readable_with_the_whole_ring, 0, "环里一条根都读不出");
    let outcome_with_the_whole_ring = recover(&whole_ring, JournalPolicy::Consult).outcome;
    assert_eq!(
        outcome_with_the_whole_ring,
        RecoveryOutcome::Failed {
            root: None,
            failure: RecoveryFailure::NoValidRoot
        },
        "整环读不出时恢复报的错误成员"
    );
}

/// 开关二要的是**持续**，不是一次瞬时错（C335（根槽持续读不出时实例表只增不减） 的整条论证建在
/// 「第一版没有根环槽的重定位 ⇒ 一个根槽持续读不出时它永远读不通」上）：同一个读者上把择根跑三遍，
/// 拦下的读数一遍比一遍多、结论三遍逐项相同。
#[test]
fn a_named_root_ring_slot_keeps_failing_every_read_not_just_the_first_one() {
    let mut pool = build_pool("switch-root-slot-read-keeps-failing");
    overwrite_in_process(&mut pool, &content_of(3100, 3), InstanceGeneration(1));
    let image = pool.memory_pool();
    let newest_slot = target_for_publish(
        CheckpointTxg(4),
        parameters().geometry.root_ring_slots_per_region,
    );
    let reader = PoolReaderWithUnreadableRootRingSlots::new(
        &image,
        root_ring_slot_target(&image, NamedRootRingSlots::NONE.with(newest_slot)),
    );

    let mut refusals_after_each_pass = Vec::new();
    let mut verdicts = Vec::new();
    for _ in 0..3 {
        verdicts.push(roots_seen_by(&reader, &image));
        refusals_after_each_pass.push(reader.reads_refused());
    }
    assert_eq!(
        verdicts,
        vec![verdicts[0]; 3],
        "三遍的结论逐项相同：{verdicts:?}"
    );
    assert_eq!(
        verdicts[0].0,
        Some((InstanceGeneration(1), CheckpointTxg(3))),
        "三遍都择到上一条根：点名的槽真的读不出，不是只把拦下的次数记了一笔"
    );
    assert!(
        refusals_after_each_pass[0] < refusals_after_each_pass[1]
            && refusals_after_each_pass[1] < refusals_after_each_pass[2],
        "每一遍都又拦下几次读（不是只拦第一次）：{refusals_after_each_pass:?}"
    );
}

/// 开关二（块设备那一侧的口子）：同一个 `RootRingSlotTarget` 摆进已有的通用故障注入设备包装
/// （`FaultInjectingBlockDevice` + `SharedFaultPlan`，增补 3 第 4 件），不另起一套包装。
/// 走 `Vec<(DeviceIdentity, Device)>` 的路径（可写挂载、回退、抬 F）要的是这个口子。
#[test]
fn the_same_named_root_ring_slots_drive_the_shared_fault_plan_on_the_block_device_side() {
    let mut pool = build_pool("switch-root-slot-block-device");
    overwrite_in_process(&mut pool, &content_of(3100, 3), InstanceGeneration(1));
    let image = pool.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let target = root_ring_slot_target(
        &image,
        NamedRootRingSlots::NONE.with(target_for_publish(
            CheckpointTxg(4),
            parameters().geometry.root_ring_slots_per_region,
        )),
    );

    let plan = SharedFaultPlan::unarmed(geometry());
    let devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = pool
        .reopen_cold()
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();

    let without_the_switch = choose_root(&devices, &system_configuration).expect("择得到根");
    assert_eq!(
        (
            without_the_switch.instance,
            without_the_switch.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4)),
        "不装开关：择到最新的根"
    );
    assert_eq!(plan.fired_count(), 0, "不装开关：一次都没注入");

    plan.arm(FaultSchedule::every_read_of_named_root_ring_slots_fails(
        target,
    ));
    let with_the_switch = choose_root(&devices, &system_configuration).expect("择得到上一条根");
    assert_eq!(
        (with_the_switch.instance, with_the_switch.checkpoint_txg),
        (InstanceGeneration(1), CheckpointTxg(3)),
        "点名 txg 4 那个槽：择到上一条根"
    );
    assert_eq!(plan.fired_count(), 1, "点名的槽上注入了一次");

    choose_root(&devices, &system_configuration).expect("再择一遍");
    assert_eq!(
        plan.fired_count(),
        2,
        "持续：第二遍择根在同一个槽上又注入了一次"
    );
}

/// 点名的集合自己的边界：位图按 (区域, 槽) 一一对应，点名与没点名分得开；整环的槽数跟着这个池的 S 走
/// （S = 8 的池 24 个，S = 4 的池 12 个，S = 16 的池 48 个——位号按区间上界排，换 S 不会撞位）。
#[test]
fn named_root_ring_slots_name_exactly_the_pairs_they_were_given() {
    let named = NamedRootRingSlots::naming(&[
        RootRingSlot { region: 0, slot: 1 },
        RootRingSlot { region: 2, slot: 7 },
    ]);
    assert_eq!(named.count(), 2);
    assert!(named.contains(RootRingSlot { region: 0, slot: 1 }));
    assert!(named.contains(RootRingSlot { region: 2, slot: 7 }));
    assert!(
        !named.contains(RootRingSlot { region: 1, slot: 1 }),
        "区域号不同就不是同一个槽"
    );
    assert!(
        !named.contains(RootRingSlot { region: 0, slot: 2 }),
        "槽号不同就不是同一个槽"
    );
    assert_eq!(
        named.named_slots(),
        vec![
            RootRingSlot { region: 0, slot: 1 },
            RootRingSlot { region: 2, slot: 7 }
        ]
    );
    assert_eq!(
        NamedRootRingSlots::every_slot_in_the_ring(
            parameters().geometry.root_ring_slots_per_region
        )
        .count(),
        24,
        "这些用例建的池 S = 8：3 × 8"
    );
    for (declared, expected_count) in [(4u64, 12u32), (16, 48)] {
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(declared)
            .expect("4 与 16 都在区间里");
        assert_eq!(
            NamedRootRingSlots::every_slot_in_the_ring(slots_per_region).count(),
            expected_count,
            "整环的槽数按传进来的 S 算"
        );
        assert!(
            NamedRootRingSlots::every_slot_in_the_ring(slots_per_region)
                .contains(RootRingSlot { region: 2, slot: 0 }),
            "位号按区间上界排：换个 S，(区域 2, 槽 0) 还是同一位"
        );
    }
    assert_eq!(NamedRootRingSlots::NONE.count(), 0);
}
