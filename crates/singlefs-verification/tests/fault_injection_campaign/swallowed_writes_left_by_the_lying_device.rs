//! 被吞掉的写：只红分配统计与回收阈值的照说谎的盘可能留下的放过，步比对镜像只落在那一步之前吞掉的写，被隔离的单元与别的槽上的违例归说谎的盘，复用槽上吞掉的写交出两份隔离；吞掉的根槽写由去掉它的根重算的上限认出，留下的全零环槽、同一进程里的抬回收下限、挂载写的事务与之后写的行都照模型认出。

use super::*;

/// 说谎的设备留下的盘面不一致里只判红 `["I-3.1", "I-7.18"]` 的那一形的取样点（按注入点认，`fault_injection::left_by_the_lying_device`；
/// `crates/mutations.tsv` 钉着把被吞的写插回去那一步拿掉的变异）。默认批那 24 段里说谎的设备留下的不一致全落在另一组
/// （`["I-2.1", "I-4.8", "I-7.4"]`），这一组要单独钉。
///
/// 取样点写死成一段历史与一个注入点，不按种子从比重表里抽：种子抽出的历史随比重表变，写死的取样点不会丢。
/// 这一段：第一个文件（txg 3）之后覆盖写 23 次（txg 4–26），根环转过一圈之后 txg 25 那次（第 21 步）的根槽 FUA 写被吞，那一槽留着上一圈的根 txg 1
/// （场景见 `scenarios_of_fault_injection_campaign::the_fixed_history_with_its_root_slot_write_swallowed`）；harness 不跑池级 checker，
/// 带着注入跑的那一遍每一步的镜像、重开的盘与被吞的写插回去之后的盘经钩子交出去（随机注入那一批的那一遍归给定状态引擎两段），
/// 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）在交出去的盘上
/// 经给定状态引擎判池级 checker、交归类（核的是归类那一截）：txg 26 那次覆盖写之后 checker 只判红 I-3.1 与 I-7.18（洞里的 txg 1 把现算的
/// 回收门槛拉到最新根带的之下，记账少于下沿；被吞的写插回去就消失）。它要按注入点认成说谎的设备留下的，不算新发现——带着注入跑的那一遍里
/// 第一张判红的每步镜像、重开的那张盘，各一条。这里只核注入的结局与交出去的盘。
#[test]
fn swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_and_the_reclaim_threshold_invariants_is_excused_as_what_the_lying_device_may_leave(
) {
    let (injection, handed_out) = the_fixed_history_with_its_root_slot_write_swallowed();
    assert!(
        injection.new_findings.is_empty(),
        "harness 那一截（模型、RefFS、执行器）不许有新发现：{:?}",
        injection.new_findings
    );
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    assert!(
        images.image_with_the_swallowed_writes_landed.is_some(),
        "被吞了写，要交被吞的写插回去之后的那张盘"
    );
}

/// 写死那一段（[`the_fixed_history_with_its_root_slot_write_swallowed`]）的那一个注入点交出去的盘，与被吞那次写之后的头一张每步镜像在里面的下标。
fn the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write(
) -> (ImagesAfterAFault, usize) {
    let (_injection, handed_out) = the_fixed_history_with_its_root_slot_write_swallowed();
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    assert_eq!(images.swallowed_writes.len(), 1, "这一段只吞了一次写");
    let first_step_image_after_the_swallowed_write = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .position(|step_image| step_image.swallowed_writes_before_it > 0)
        .expect("被吞那次写之后还有每步镜像");
    (images.clone(), first_step_image_after_the_swallowed_write)
}

/// 合成的一份违例：`invariant` 判红，点名 `slots` 里的每一个（盘, 槽）。
fn violations_naming(invariant: &'static str, slots: &[(u32, u64)]) -> ViolationsNamingTheSites {
    ViolationsNamingTheSites {
        violations: vec![(invariant, "合成的违例".to_string())],
        violation_sites: slots
            .iter()
            .map(|(device, slot)| ViolationSite {
                invariant,
                device: *device,
                slot: *slot,
            })
            .collect(),
    }
}

/// 写死那一段交出去的盘上，第 `observed_step_image` 张每步镜像判红 `observed`、它的按步对照盘判红 `at_the_step`、
/// 被吞的写都插回去那张判红 `with_every_swallowed_write_landed`、重开那张不红：交归类器，交回归好的那几条。
fn classified_with_given_verdicts(
    images: &ImagesAfterAFault,
    observed_step_image: usize,
    observed: ViolationsNamingTheSites,
    at_the_step: ViolationsNamingTheSites,
    with_every_swallowed_write_landed: ViolationsNamingTheSites,
) -> Vec<FailureAfterAFault> {
    let mut violations_of_the_step_images =
        vec![Vec::new(); images.step_images_of_the_run_with_the_fault.len()];
    violations_of_the_step_images[observed_step_image] = observed.violations;
    classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
        images,
        PoolCheckerVerdictsOfTheImagesAfterAFault {
            violations_of_the_step_images,
            violation_sites_of_the_first_red_step_image: observed.violation_sites,
            after_the_fault: ViolationsNamingTheSites::default(),
            at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed: Some(
                at_the_step,
            ),
            with_the_swallowed_writes_landed: Some(with_every_swallowed_write_landed),
        },
    )
}

/// 按步对照盘（判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3）：被吞那次写之前的每步镜像没有按步对照盘（插回去就是它本身）；
/// 之后的每一张，按步对照盘是那一刻的录制流前缀插回那一次被吞的写——与被吞的写都插回去那张盘的头这么多步逐步相同。
#[test]
fn the_step_comparison_image_lands_only_the_writes_swallowed_before_that_step_image() {
    let (images, first_step_image_after_the_swallowed_write) =
        the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write();
    let landed = images
        .image_with_the_swallowed_writes_landed
        .as_ref()
        .expect("被吞了写，要交被吞的写插回去之后的那张盘");
    for (index, step_image) in images
        .step_images_of_the_run_with_the_fault
        .iter()
        .enumerate()
    {
        let at_the_step =
            images.image_at_the_step_image_with_the_swallowed_writes_before_it_landed(index);
        if index < first_step_image_after_the_swallowed_write {
            assert_eq!(
                at_the_step, None,
                "第 {index} 张每步镜像那一刻之前没吞写，没有按步对照盘"
            );
            continue;
        }
        let at_the_step = at_the_step
            .unwrap_or_else(|| panic!("第 {index} 张每步镜像那一刻之前吞过一次写，要有按步对照盘"));
        assert_eq!(
            at_the_step.operations.as_slice(),
            &landed.operations[..step_image.operations_recorded + 1],
            "第 {index} 张：那一刻的前缀插回被吞的那一次写，与全部插回去那张的头几步逐步相同"
        );
        assert_eq!(
            at_the_step.operations_written_by_make_filesystem,
            landed.operations_written_by_make_filesystem,
            "第 {index} 张：被吞的写落在 mkfs 之后，mkfs 的步数不跟着挪"
        );
    }
}

/// 按步对照，不对照末尾那张：被吞那次写之后的头一张每步镜像点名
/// （盘 0 与盘 1, 槽 777）判红 I-3.1，它的按步对照盘上同一处照红，被吞的写都插回去那张盘上一条都不红（那一处只在中间那一步的镜像上红，
/// 两腿 `leak_once` 量到的那一类）：仍红、不是被吞的写留下的，报新发现。对照末尾那张盘就把它归成说谎的设备。
#[test]
fn a_violation_still_named_at_its_slot_on_the_step_comparison_image_is_a_new_finding_though_the_last_comparison_image_no_longer_shows_it(
) {
    let (images, step_image) =
        the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write();
    let failures = classified_with_given_verdicts(
        &images,
        step_image,
        violations_naming("I-3.1", &[(0, 777), (1, 777)]),
        violations_naming("I-3.1", &[(0, 777), (1, 777)]),
        ViolationsNamingTheSites::default(),
    );
    assert!(
        matches!(
            failures.as_slice(),
            [FailureAfterAFault::NewFinding { signature, .. }]
                if *signature == FailureSignature::CheckerViolations { invariants: vec!["I-3.1"] }
        ),
        "按步对照盘上同一处照红：报新发现一条，判成 {failures:?}"
    );
}

/// 按（不变量, 盘, 槽）比，不按名比：被比那一张点名（盘 0, 槽 777）判红 I-3.1，按步对照盘上 I-3.1 也红、点名的却是
/// 另一处（盘 0, 槽 888）：被比那一张点名的那一处插回去就不红了，这一条是被吞的写留下的，归说谎的设备。按名比就照报新发现。
#[test]
fn a_violation_named_at_another_slot_on_the_step_comparison_image_is_left_by_the_lying_device() {
    let (images, step_image) =
        the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write();
    let failures = classified_with_given_verdicts(
        &images,
        step_image,
        violations_naming("I-3.1", &[(0, 777)]),
        violations_naming("I-3.1", &[(0, 888)]),
        violations_naming("I-3.1", &[(0, 888)]),
    );
    assert!(
        matches!(
            failures.as_slice(),
            [FailureAfterAFault::LeftByTheLyingDevice { .. }]
        ),
        "点名的那一处插回去就不红：归说谎的设备一条，判成 {failures:?}"
    );
}

/// 实现交回的隔离：被比那一张点名（两块盘, 槽 s）判红 I-3.1，按步对照盘上同一处照红，
/// 而槽 s 正是实现在那一刻之前交回的一份隔离、那一份压在被吞的写上——插回去之后两份都对得上、记录却留在已分配，是实现在说谎的设备上
/// 读盘核出对不上才隔离的：减掉之后不再仍红，归说谎的设备。不减（隔离清单不进比较）就照报新发现。槽 s 取被吞那次写落的槽，
/// 隔离那一份合成（这一段被吞的是根槽写，实现没有隔离）。
#[test]
fn a_violation_named_only_at_a_unit_the_implementation_quarantined_over_a_swallowed_write_is_left_by_the_lying_device(
) {
    let (mut images, step_image) =
        the_fixed_history_handed_out_and_the_first_step_image_after_the_swallowed_write();
    let swallowed = images.swallowed_writes[0].clone();
    let slot_of_the_swallowed_write = swallowed.offset.0 / singlefs_format::SLOT_BYTES;
    images
        .copies_quarantined_by_the_implementation
        .push(CopyQuarantinedByTheImplementation {
            step: images.step_images_of_the_run_with_the_fault[step_image].position,
            copy: CopyQuarantinedAfterReleaseChecksumMismatch {
                unit: TransactionUnit::InodeRoot,
                device: swallowed.device,
                placement: Placement {
                    slot: SlotNumber(slot_of_the_swallowed_write),
                    span: 1,
                },
                reading: QuarantinedCopyReading::ChecksumMismatch,
            },
        });
    images.step_images_of_the_run_with_the_fault[step_image]
        .copies_quarantined_by_the_implementation_before_it = 1;
    assert_eq!(
        unit_start_slots_quarantined_over_a_swallowed_write(
            &images.copies_quarantined_by_the_implementation,
            &images.swallowed_writes,
        ),
        BTreeSet::from([slot_of_the_swallowed_write]),
        "合成的那一份隔离压在被吞的写上"
    );
    let sites = [
        (0, slot_of_the_swallowed_write),
        (1, slot_of_the_swallowed_write),
    ];
    let failures = classified_with_given_verdicts(
        &images,
        step_image,
        violations_naming("I-3.1", &sites),
        violations_naming("I-3.1", &sites),
        ViolationsNamingTheSites::default(),
    );
    assert!(
        matches!(
            failures.as_slice(),
            [FailureAfterAFault::LeftByTheLyingDevice { .. }]
        ),
        "插回去之后仍红的只有压在被吞的写上的那一份隔离：归说谎的设备一条，判成 {failures:?}"
    );
}

/// 隔离判据那三条之外的不变量不减隔离（I-3.10 判的是分配代与诞生代号，隔离不豁免它）；点了槽的一边没有槽可比时按名比。
#[test]
fn only_the_quarantine_criterion_invariants_drop_the_quarantined_slots_and_a_side_without_slots_compares_by_name(
) {
    let quarantined = BTreeSet::from([777]);
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-3.10", &[(1, 777)]),
            Some(&violations_naming("I-3.10", &[(1, 777)])),
            &quarantined,
        ),
        BTreeSet::from(["I-3.10"]),
        "I-3.10 不归隔离判据：同一处照红就仍红"
    );
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-3.15", &[(1, 777)]),
            Some(&violations_naming("I-3.15", &[(1, 777)])),
            &quarantined,
        ),
        BTreeSet::new(),
        "I-3.15 随 I-3.1 那一份隔离判据：压在被吞的写上的隔离减掉"
    );
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-2.1", &[]),
            Some(&violations_naming("I-2.1", &[])),
            &quarantined,
        ),
        BTreeSet::from(["I-2.1"]),
        "没点槽的按名比：对照盘上也红就仍红"
    );
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-3.1", &[(0, 5)]),
            Some(&violations_naming("I-3.1", &[])),
            &quarantined,
        ),
        BTreeSet::from(["I-3.1"]),
        "对照盘上这一条红而一个槽都没点：按名比，仍红"
    );
    assert_eq!(
        invariants_still_violated_on_the_comparison_image(
            &violations_naming("I-3.1", &[(0, 5)]),
            None,
            &quarantined,
        ),
        BTreeSet::from(["I-3.1"]),
        "没有对照盘（那一刻之前没吞写）：每一条都仍红"
    );
}

/// 镜像上最新根那棵分配记录树里起点是某一槽的一条记录读回来的样子（盘上的隔离记号，D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」）。
#[derive(Debug, PartialEq, Eq)]
enum AllocatedRecordReadBack {
    Released,
    CarryingTheQuarantineMark,
    WithoutTheQuarantineMark,
}

/// 镜像上最新根那棵分配记录树里起点是 `slot` 的那几条记录（按盘号从小到大）各读成什么样。
fn records_of_the_newest_root_starting_at(
    image: &MemoryPool,
    slot: u64,
) -> Vec<AllocatedRecordReadBack> {
    let system_configuration = choose_system_configuration(image).expect("择系统配置");
    let newest = choose_root(image, &system_configuration).expect("最新根");
    let mut records: Vec<AllocationRecord> = allocation_records_under_root(image, &newest)
        .expect("最新根那棵分配记录树读得回来（读者认隔离记号）")
        .into_iter()
        .filter(|record| record.slot.0 == slot)
        .collect();
    records.sort_by_key(|record| record.device.0);
    records
        .iter()
        .map(|record| {
            if record.is_released {
                return AllocatedRecordReadBack::Released;
            }
            match record.quarantine_mark {
                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck => {
                    AllocatedRecordReadBack::CarryingTheQuarantineMark
                }
                AllocationRecordQuarantineMark::NotQuarantined => {
                    AllocatedRecordReadBack::WithoutTheQuarantineMark
                }
            }
        })
        .collect()
}

/// 必红世界那一段（场景 [`a_leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces`]）的实现那一半：
/// 实现交回两次隔离、各两份——X 那一次（泄漏的那个单元，不压在被吞的写上）与 Y 那一次（被吞的那一份所在的单元，压在被吞的写上）；
/// X 之后那一张每步镜像那一刻之前吞过写，交出去的盘里它的按步对照盘、被吞的写都插回去那张都在。盘上的记号（D19（块指针的结构与宽度预算）
/// 已定项 5「隔离的盘上记号」）：X 之后那一张每步镜像上泄漏的那个单元两块盘那两条记录仍分配、带记号；重开那张上它们仍分配、两块盘都不带记号
/// （泄漏），Y 隔离的那个单元两块盘那两条仍分配、都带记号（照条款豁免）。池级 checker 那一遍与归类在 同名的给定状态用例：重开那张报新发现。
#[test]
fn a_leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces() {
    let images =
        scenarios_of_fault_injection_campaign::run_the_scenario_where_a_leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces();
    let quarantined = &images.copies_quarantined_by_the_implementation;
    assert_eq!(quarantined.len(), 4, "两次隔离各两份：{quarantined:?}");
    let leaked_slot = quarantined[0].copy.placement.slot.0;
    let slot_quarantined_over_the_swallowed_write = quarantined[2].copy.placement.slot.0;
    let pool_images = scenarios_of_fault_injection_campaign::pool_images_after_a_fault(&images);
    let step_image_after_the_swallowed_write = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .position(|step_image| step_image.swallowed_writes_before_it > 0)
        .expect("X 之后那一张每步镜像那一刻之前吞过写");
    assert_eq!(
        records_of_the_newest_root_starting_at(
            &pool_images.step_images[step_image_after_the_swallowed_write],
            leaked_slot
        ),
        vec![
            AllocatedRecordReadBack::CarryingTheQuarantineMark,
            AllocatedRecordReadBack::CarryingTheQuarantineMark
        ],
        "X 之后那一张：泄漏的那个单元两块盘那两条记录仍分配、带着 X 写下的记号（头一张判红的每步镜像上没有泄漏）"
    );
    assert_eq!(
        records_of_the_newest_root_starting_at(&pool_images.after_the_fault, leaked_slot),
        vec![
            AllocatedRecordReadBack::WithoutTheQuarantineMark,
            AllocatedRecordReadBack::WithoutTheQuarantineMark
        ],
        "重开那张：泄漏的那个单元两块盘那两条记录仍分配、都不带记号"
    );
    assert_eq!(
        records_of_the_newest_root_starting_at(
            &pool_images.after_the_fault,
            slot_quarantined_over_the_swallowed_write
        ),
        vec![
            AllocatedRecordReadBack::CarryingTheQuarantineMark,
            AllocatedRecordReadBack::CarryingTheQuarantineMark
        ],
        "重开那张：Y 隔离的那个单元两块盘那两条记录仍分配、都带记号"
    );
    assert_ne!(
        leaked_slot, slot_quarantined_over_the_swallowed_write,
        "泄漏的那个单元与被吞的那一份所在的单元不是同一个"
    );
    assert_eq!(
        unit_start_slots_quarantined_over_a_swallowed_write(quarantined, &images.swallowed_writes),
        BTreeSet::from([slot_quarantined_over_the_swallowed_write]),
        "压在被吞的写上的只有 Y 那一次隔离的那个单元"
    );
    let step_image_of_the_swallowed_write = images
        .step_images_of_the_run_with_the_fault
        .iter()
        .position(|step_image| step_image.swallowed_writes_before_it > 0)
        .expect("X 之后那一张每步镜像那一刻之前吞过写");
    assert!(
        images
            .image_at_the_step_image_with_the_swallowed_writes_before_it_landed(
                step_image_of_the_swallowed_write
            )
            .is_some()
            && images.image_with_the_swallowed_writes_landed.is_some(),
        "按步对照盘与被吞的写都插回去那张盘都交出去"
    );
}

/// 必绿世界 K2-1 那一点（场景 [`the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot`]）的实现那一半：
/// 头一个注入点吞掉盘 1 上一次落在复用过的槽上的 16 KiB 写，实现之后释放那一版时读盘核出盘 1 那一份对不上、隔离两块盘那一份，
/// 清单经交出去的盘带出来（跨重挂累积：之后的挂载不清掉它）。池级 checker 那一遍与归类在 同名的给定状态用例：归说谎的设备，不报新发现。
#[test]
fn the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot_hands_out_the_quarantine_of_both_copies(
) {
    let images = scenarios_of_fault_injection_campaign::the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot();
    assert_eq!(
        images.drawn.fault,
        InjectedFault::WriteIsSwallowed,
        "头一个注入点是一次被吞的写：{}",
        images.drawn.render()
    );
    let [swallowed] = images.swallowed_writes.as_slice() else {
        panic!("吞了一次写：{:?}", images.swallowed_writes.len());
    };
    let swallowed_range = SwallowedWriteByteRange::of(swallowed);
    let swallowed_slot = swallowed.offset.0 / singlefs_format::SLOT_BYTES;
    assert_eq!(
        (
            swallowed.device.0,
            swallowed_range.end_in_bytes - swallowed_range.start_in_bytes
        ),
        (1, singlefs_format::SLOT_BYTES),
        "被吞的是盘 1 上一槽那么长的一次写"
    );
    assert_eq!(
        unit_start_slots_quarantined_over_a_swallowed_write(
            &images.copies_quarantined_by_the_implementation,
            &images.swallowed_writes,
        ),
        BTreeSet::from([swallowed_slot]),
        "实现隔离了被吞的写落的那一槽：{:?}",
        images.copies_quarantined_by_the_implementation
    );
    let devices_quarantined_at_the_slot: BTreeSet<u32> = images
        .copies_quarantined_by_the_implementation
        .iter()
        .filter(|quarantined| quarantined.copy.placement.slot.0 == swallowed_slot)
        .map(|quarantined| quarantined.copy.device.0)
        .collect();
    assert_eq!(
        devices_quarantined_at_the_slot,
        BTreeSet::from([0, 1]),
        "两块盘那一份一起隔离（账保持对称）"
    );
}

/// 设变量跑的一趟 512 段故障注入（这个测试周期的种子基起 512 段、每段 30 步、每段注入 6 次）撞出的一段，单独跑：种子 7463871032432355397，规模与执行方式与那一趟逐项相同。
/// 抽出的 6 个注入点里有两次被吞掉的写（`write_is_swallowed`）：整池第 481 次写调用（第 17 步覆盖写那次发布的根槽写）与第 448 次
/// （第 16 步挂着时回退那次向前发布的根槽写）。两个注入点上历史都走到第 18 步挂着时回退：判候选集读根环，这个进程知道住着根的那一槽
/// （被吞那次写的槽）读坏、重读仍坏，拒在任何写之前（`RollbackError::CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`，
/// D16（发布语义） 已定项 1「根槽这一次读坏」那一行的读法）。模型不知道根槽没落、说该成；回退点名的槽正是被吞那次写的槽，各记进说谎那一格一次，
/// 不算新发现。回退路径不把点名的槽交给模型比对时（`crates/mutations.tsv` 钉着的那一格），这一格判不出是说谎的设备留下的，照报新发现。
/// 池级 checker 那一遍交给定状态引擎走两段、harness 不停：第 448 次那个注入点上第 17 步之后的镜像判红 I-3.1、I-3.9（释放代落在停止引用它的那一格区间里），是被吞的根留下的，
/// 历史照走到第 18 步，两个注入点各记一次
/// （照判红就停那一档的口径并进给定状态引擎结论的是合并入口 `inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`）。
#[test]
fn rollback_refused_on_the_root_ring_slot_whose_write_the_lying_device_swallowed_is_excused_as_what_the_lying_device_may_leave(
) {
    let report = run_fault_injection_campaign(&FaultInjectionCampaign {
        first_seed: 7_463_871_032_432_355_397,
        seed_count: 1,
        operations_per_history: 30,
        faults_per_history: 6,
        weights: GenerationWeights::BROAD,
        execution: HANDING_EVERY_STEP_IMAGE_OUT,
        worker_threads: FaultInjectionWorkerThreads::from_the_environment(),
    });
    let rendered = report.render();
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "回退拒在被吞根槽写的那个槽上要记进说谎那一格，不算新发现；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert_eq!(
        report
            .tally
            .faults_by_injection_point
            .get("PublishOverwrite/root_record_fua"),
        Some(&1),
        "这一段抽出的注入点里要有那一次被吞的覆盖写根槽写：\n{rendered}"
    );
    assert_eq!(
        report
            .tally
            .lying_device_signatures
            .get("ModelDisagreement { aspect: \"模型说该成、实现拒了\" }"),
        Some(&2),
        "回退那两次「模型说该成、实现拒了」要按点名的槽等于被吞根槽写的槽放过、记进说谎那一格，两个注入点各一次：\n{rendered}"
    );
}

/// 按注入点认说谎的设备留下的那两格（③ offset 339 与 ④ offset 284 那两形）：
/// 第一个文件（txg 3）之后覆盖写五次（4–8），txg 7 那次的根槽 FUA 写被吞；再可写挂载（实例 2，择到 txg 8、不施加记录，
/// 与模型对得上），接着抬 F 到 1。这个进程挂载时 txg 7 那个根槽读回全零（从没写对过），实现把它并进准入抬 F 的上限（D16（发布语义） 已定项 1
/// 「抬 F 的上限」那一行：能住的最大 txg 7 高于照读得出的根算的 4，上限照旧 4），报的上限按看不见 (1, 7) 算，
/// 模型按确认过的根算、多一个非空状态——
/// 历史停在这一步；按模型去掉 (1, 7) 之后重算的上限与实现报的相等，认成说谎的设备留下的。抬 F 那一步照实现的上限做成、写出的那几条
/// 空发布根不在模型认过的版本里，重开走到最新那一条、读回的是 txg 8 那一版的内容：「放行集并进历史停下那一步实现已写出的根」认下它。
/// 两格任一格不认，这一次注入就成了新发现。
/// （同一个进程里吞了根槽再抬 F 走的是另一格，见
/// `floor_raise_in_the_same_process_after_a_swallowed_root_slot_write_is_refused_for_that_slot_and_recognised_as_left_by_the_lying_device`。）
#[test]
fn a_swallowed_root_slot_write_is_recognised_by_the_ceiling_recomputed_without_its_root_and_the_floor_raise_it_left_behind_is_allowed_on_reopen(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            overwrite_with_fill_seed(2),
            overwrite_with_fill_seed(3),
            overwrite_with_fill_seed(4),
            overwrite_with_fill_seed(5),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 1,
            }),
        ],
    };
    // 每一步之后不跑池级 checker：txg 8 那次覆盖写之后 checker 就判红 I-3.1（被吞的根槽留下的，被吞的写插回去就消失），
    // 跑着的话历史停在那一步、走不到抬 F；重开之后那一次 checker 照跑、照判。
    let execution_without_the_per_step_checker = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let injection = inject_one_fault_into_the_segment(
        &history,
        execution_without_the_per_step_checker,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: 3,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(&history, 3),
    );
    let tally = &injection.tally;
    assert!(
        injection.new_findings.is_empty(),
        "按注入点认得出的两格不许成新发现：{:?}",
        injection.new_findings
    );
    assert_eq!(
        tally
            .lying_device_signatures
            .get("ModelDisagreement { aspect: \"抬 F 的上限\" }"),
        Some(&1),
        "抬 F 的上限那一格按去掉被吞的根之后重算的上限认下：{:?}",
        tally.lying_device_signatures
    );
    assert_eq!(
        tally.reopened_into_the_version_the_faulted_step_was_writing, 1,
        "重开走到停下那一步写出的根上，放行集认下它"
    );
}

/// 被吞的根槽写留下的全零槽卡住重挂之后的上限（判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 4 问；
/// 合入后验证 6223 那一红的形状：种子 7463871032432355616 实现报上限 19、模型 27）。
/// 场景：第一个文件（txg 3）之后覆盖写九次（4–12），txg 7 那次的根槽 FUA 写被吞；再可写挂载（实例 2，择到 txg 12，写行 13、暖机 14），
/// 接着抬 F 到 1。txg 7 那一槽（区域 1 槽 2，S = 8，环长 24 之内头一次被写）挂载那一刻读回全零、这个进程之后没写过，
/// 实现把它能住的最大 txg 7 并进上限（D16（发布语义） 已定项 1「抬 F 的上限」那一行）：照读得出的根算是
/// min(13, 9) = 9（第 4 新的不同状态 12、11、10、9），并进来 7，报 7；模型按确认过的根算 9，历史停在这一步、对不上的是「抬 F 的上限」。
/// 预期：模型去掉 (1, 7)、并进它留下的全零槽之后重算 7，与实现报的相等，认成说谎的设备留下的，不成新发现。
/// 判别力：故障注入只比不并全零槽的那个数（9）时这一次成新发现（`crates/mutations.tsv`「故障注入认上限只比不并全零槽的那个数」）。
#[test]
fn a_swallowed_root_slot_write_leaving_an_all_zero_ring_slot_that_holds_the_ceiling_after_the_remount_is_left_by_the_lying_device(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            overwrite_with_fill_seed(2),
            overwrite_with_fill_seed(3),
            overwrite_with_fill_seed(4),
            overwrite_with_fill_seed(5),
            overwrite_with_fill_seed(6),
            overwrite_with_fill_seed(7),
            overwrite_with_fill_seed(8),
            overwrite_with_fill_seed(9),
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 1,
            }),
        ],
    };
    // 每一步之后不跑池级 checker：txg 8 那次覆盖写之后 checker 就判红 I-3.1（被吞的根槽留下的），跑着的话历史停在那一步、走不到抬 F。
    let execution_without_the_per_step_checker = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let injection = inject_one_fault_into_the_segment(
        &history,
        execution_without_the_per_step_checker,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: 3,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(&history, 3),
    );
    let tally = &injection.tally;
    assert!(
        injection.new_findings.is_empty(),
        "实现报的上限是并进被吞的根留下的全零槽之后的那个，不许成新发现：{:?}",
        injection.new_findings
    );
    assert_eq!(
        tally
            .lying_device_signatures
            .get("ModelDisagreement { aspect: \"抬 F 的上限\" }"),
        Some(&1),
        "抬 F 的上限那一格按去掉被吞的根、并进它留下的全零槽之后重算的上限认下：{:?}",
        tally.lying_device_signatures
    );
}

/// 同一个进程里根槽写被吞之后再抬 F（实七报告 (b)，实八）：第一个文件（txg 3）之后覆盖写五次（4–8），txg 7 那次的根槽 FUA 写被吞，
/// 不重新挂载，接着抬 F 到 1。这个进程写过 txg 7 那个槽、FUA 返回过，算上限时读它读坏、重读仍坏，实现照 D16（发布语义） 已定项 1
/// 「根槽这一次读坏」那一行拒这次抬 F、报 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，点名的就是 txg 7 那个槽。
/// 模型不知道根槽没落，要它成或按上限拒，对拍报对不上（前半截钉住这一形）；按注入点认：读坏的槽正是被吞那次写的，认成说谎的设备留下的，
/// 不成新发现（后半截）。复现的随机种子是故障注入种子基 + 116（512 段那一趟）。
#[test]
fn floor_raise_in_the_same_process_after_a_swallowed_root_slot_write_is_refused_for_that_slot_and_recognised_as_left_by_the_lying_device(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            overwrite_with_fill_seed(2),
            overwrite_with_fill_seed(3),
            overwrite_with_fill_seed(4),
            overwrite_with_fill_seed(5),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 1,
            }),
        ],
    };
    // 每一步之后不跑池级 checker：txg 8 那次覆盖写之后 checker 就判红 I-3.1（被吞的根槽留下的），跑着的话历史停在那一步、走不到抬 F。
    let execution_without_the_per_step_checker = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let parameters = execution_without_the_per_step_checker
        .device_width
        .parameters();
    let root_slot_of_the_swallowed_publish = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(7),
        parameters.geometry.root_ring_slots_per_region,
    );
    let plan = SharedFaultPlan::armed(
        execution_without_the_per_step_checker
            .device_width
            .fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteIsSwallowed,
            device: FaultDeviceSelector::OnlyDevice(
                parameters.region_devices[usize::try_from(
                    root_slot_of_the_swallowed_publish.region,
                )
                .expect("区域号小于 3")],
            ),
            placement: FaultPlacement::OffsetExactly(singlefs_core::root_ring::slot_offset(
                root_slot_of_the_swallowed_publish,
                parameters.geometry.fixed_structure_slot_spacing,
            )),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(1),
        },
    );
    let run = execute_history_with_faults(
        &history,
        execution_without_the_per_step_checker,
        &SharedStream::new(),
        &plan,
        &mut |_| {},
    );
    assert_eq!(plan.fired().len(), 1, "只吞掉 txg 7 那一次根槽写");
    let HistoryEnding::NewFinding { observation, .. } = &run.ending else {
        panic!("不按注入点认时，抬 F 那一步对拍对不上：{:?}", run.ending);
    };
    assert_eq!(observation.position, StepPosition::Operation(5));
    let disagreement = observation
        .model_disagreement
        .as_ref()
        .expect("停在模型对不上");
    assert!(
        matches!(
            disagreement.aspect,
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                | ModelDisagreementAspect::RefusalReason
        ),
        "模型要它成或按上限拒，实现拒了：{disagreement:?}"
    );
    assert!(
        disagreement
            .implementation_answer
            .contains("RollbackFloorCeilingRootRingSlotStillBadAfterOneReread"),
        "实现报的是「根环槽读坏、重读仍坏」：{}",
        disagreement.implementation_answer
    );
    assert_eq!(
        disagreement.implementation_reported_unreadable_place,
        Some(
            ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread(
                singlefs_verification::model_comparison::model_ring_position(
                    root_slot_of_the_swallowed_publish
                )
            )
        ),
        "实现点名的读坏的槽是 txg 7 那一次被吞的根槽写的槽"
    );

    let injection = inject_one_fault_into_the_segment(
        &history,
        execution_without_the_per_step_checker,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: 3,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(&history, 3),
    );
    assert!(
        injection.new_findings.is_empty(),
        "读坏的槽正是被吞那次写的，不许成新发现：{:?}",
        injection.new_findings
    );
    let signature = format!(
        "{:?}",
        FailureSignature::ModelDisagreement {
            aspect: disagreement.aspect.name()
        }
    );
    assert_eq!(
        injection.tally.lying_device_signatures.get(&signature),
        Some(&1),
        "按注入点认下的是抬 F 那一格：{:?}",
        injection.tally.lying_device_signatures
    );
}

/// 「写的实例表行」那一形（实四丙交回第五节第 6 条没判的那一格；实七造、判）：第一个文件（txg 3）之后覆盖写一次（txg 4），
/// 它的根槽 FUA 写被吞——记录与单元落了盘、根槽没落；接着可写挂载。挂载里的恢复择到 txg 3、施加 txg 4 的记录（走到 (1, 4)），
/// 给实例 1 写的行是 (1, 4, W)，W 取这次恢复施加的记录里最大的事务号（D23（journal 的角色与格式） 已定项 14 第 4 条），
/// 实现照条款写。模型不知道根槽没落，按「不建崩溃、一条记录都不施加」答 (1, 4, 0)，对拍报「写的实例表行」。
/// 这里钉住不按注入点认时对拍看到的样子：行里所选根的 txg 两边一样，差的只有 W，而实现写的 W 正是那条记录的事务号。
/// 按注入点认的那一半见 `the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied`。
#[test]
fn mount_after_a_swallowed_root_slot_write_writes_the_transaction_of_the_record_it_applied_while_the_model_expects_none_applied(
) {
    let execution = HANDING_EVERY_STEP_IMAGE_OUT;
    let parameters = execution.device_width.parameters();
    let root_slot_of_the_overwrite = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(4),
        parameters.geometry.root_ring_slots_per_region,
    );
    let plan = SharedFaultPlan::armed(
        execution.device_width.fixed_geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteIsSwallowed,
            device: FaultDeviceSelector::OnlyDevice(
                parameters.region_devices
                    [usize::try_from(root_slot_of_the_overwrite.region).expect("区域号小于 3")],
            ),
            placement: FaultPlacement::OffsetExactly(singlefs_core::root_ring::slot_offset(
                root_slot_of_the_overwrite,
                parameters.geometry.fixed_structure_slot_spacing,
            )),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(1),
        },
    );
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let mut image_before_the_mount: Option<MemoryPool> = None;
    let run = execute_history_with_faults(
        &history,
        execution,
        &SharedStream::new(),
        &plan,
        &mut |observation| {
            if observation.position == StepPosition::Operation(0) {
                image_before_the_mount = Some(observation.image.clone());
            }
        },
    );
    assert_eq!(plan.fired().len(), 1, "只吞掉覆盖写那一次根槽写");
    let recovery_the_mount_runs = recover(
        &image_before_the_mount.expect("观察者见过覆盖写之后的镜像"),
        JournalPolicy::Consult,
    );
    assert_eq!(
        recovery_the_mount_runs.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "恢复择到 txg 3、施加 txg 4 的记录"
    );
    let applied_transaction = recovery_the_mount_runs.journal.maximum_applied_transaction;
    assert!(applied_transaction > 0, "施加的是一次带单元的覆盖写的记录");
    let HistoryEnding::NewFinding { observation, .. } = &run.ending else {
        panic!("今天这一格报成新发现：{:?}", run.ending);
    };
    assert_eq!(observation.position, StepPosition::Operation(1));
    let disagreement = observation
        .model_disagreement
        .as_ref()
        .expect("停在模型对不上");
    assert_eq!(disagreement.aspect, ModelDisagreementAspect::InstanceRows);
    for (who, rows, high_water) in [
        ("模型", &disagreement.model_answer, 0),
        (
            "实现",
            &disagreement.implementation_answer,
            applied_transaction,
        ),
    ] {
        assert!(
            rows.contains("selected_root_txg: ModelCheckpointTxg(4)")
                && rows.contains(&format!("applied_transaction_high_water: {high_water}")),
            "{who}写的行是 (1, 4, {high_water})：{rows}"
        );
    }
}

/// 「写的实例表行」那一形按注入点认（实七报告 (d)，实八）：同一段历史（第一个文件之后覆盖写一次、它的根槽写被吞、接着可写挂载），
/// 注入经 `inject_one_fault_into_the_segment` 走。模型拿历史停下那一步之前的自己、按「txg 4 的根槽没落、它的记录与单元落了」重算
/// 可写挂载写的行：恢复择到 txg 3、施加 txg 4 的记录，行是 (1, 4, W)，W 取那条记录的事务号（D23（journal 的角色与格式） 已定项 14 第 4 条）。
/// 实现写的正是这一行，认成说谎的设备留下的，不成新发现。
#[test]
fn the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied(
) {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite_with_fill_seed(1),
            HistoryOperation::CloseAndMountWritable,
        ],
    };
    let injection = inject_one_fault_into_the_segment(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        InjectedFault::WriteIsSwallowed,
        FaultedSegment::Operation {
            step_index: 0,
            operation_kind: HistoryOperationKind::PublishOverwrite,
        },
        ordinal_of_the_root_slot_write_within_the_step(&history, 0),
    );
    assert!(
        injection.new_findings.is_empty(),
        "实现写的行等于模型重算的行，不许成新发现：{:?}",
        injection.new_findings
    );
    let signature = format!(
        "{:?}",
        FailureSignature::ModelDisagreement {
            aspect: ModelDisagreementAspect::InstanceRows.name()
        }
    );
    assert_eq!(
        injection.tally.lying_device_signatures.get(&signature),
        Some(&1),
        "按注入点认下的是「写的实例表行」那一格：{:?}",
        injection.tally.lying_device_signatures
    );
}
