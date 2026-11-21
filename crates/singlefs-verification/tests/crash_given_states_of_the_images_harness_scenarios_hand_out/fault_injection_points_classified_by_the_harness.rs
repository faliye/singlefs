//! 故障注入那一族：故障注入测试文件里同名的场景，与那份文件写死的注入点在给定状态引擎上判、池级 checker 的结论交 harness 归类。

use super::*;

// ─── `tests/fault_injection_campaign.rs` ───

/// 与 `crates/singlefs-verification/tests/fault_injection_campaign.rs` 里同名的用例同一个场景（场景、要判的镜像与该判成什么见
/// `scenarios_of_fault_injection_campaign::run_the_scenario_where_publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root`）。
#[test]
fn publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root(
) {
    judge_the_images_a_harness_scenario_hands_out(
        "publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root",
        scenarios_of_fault_injection_campaign::run_the_scenario_where_publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root,
    );
}

// ─── `tests/fault_injection_campaign.rs` 写死的那两条注入点：池级 checker 的结论交 harness 归类 ───

/// mkfs 在写到系统配置之前就报错时重开的那张盘：盘上没有一份自证过的系统配置，GPU 判器建不出判表，引擎照 H6 记一条「GPU 判不了」、
/// 交 CPU 判器的结论（`crates/singlefs-verification/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`
/// 那几张 mkfs 没做完的表同一种）。
const MAKE_FILESYSTEM_UNFINISHED: &str =
    "mkfs 在写到系统配置之前就报错：盘上没有一份自证过的系统配置，GPU 判器建不出判表";

/// 一个注入点交出去的盘（`fault_injection::ImagesAfterAFault`）重建成镜像（每步镜像、重开的那张、被吞的写插回去那张各一张表；第一张判红的每步镜像那一刻之前
/// 吞过写时，再加它的按步对照盘一张，另起一趟引擎判），经给定状态引擎判池级 checker，结论交 harness 的归类
/// （`fault_injection::classify_the_pool_checker_verdicts_of_the_images_after_a_fault`）。归类器按（不变量, 盘, 槽）比要的违例点名由 CPU 参照判同一张盘交
/// （[`violations_naming_the_sites_by_the_cpu_reference`]）。交回归好类的失败；这一趟不作数交 `None`。
/// `the_gpu_may_refuse_the_reopened_image_because` 给了就是许 GPU 建不出重开那一张的判表。
fn failures_after_a_fault_judged_through_the_library(
    scenario_name: &str,
    handed_out: &ImagesAfterAFault,
    the_gpu_may_refuse_the_reopened_image_because: Option<&'static str>,
) -> Option<Vec<FailureAfterAFault>> {
    let pool_images = scenarios_of_fault_injection_campaign::pool_images_after_a_fault(handed_out);
    let mut collected = PoolCheckerImages::collected_for_the_given_states_engine();
    for (index, step_image) in pool_images.step_images.iter().enumerate() {
        collected.hand_out(
            &format!("带着注入跑的那一遍的每步镜像（下标 {index}）"),
            || step_image.clone(),
            PoolCheckerExpectation::only_the_named_invariants(),
        );
    }
    collected.hand_out_allowing_the_gpu_to_refuse_its_tables(
        "注入之后重开的那张盘",
        || pool_images.after_the_fault.clone(),
        PoolCheckerExpectation::only_the_named_invariants(),
        the_gpu_may_refuse_the_reopened_image_because,
    );
    if let Some(landed) = &pool_images.with_the_swallowed_writes_landed {
        collected.hand_out(
            "被吞的写插回去之后的那张盘",
            || landed.clone(),
            PoolCheckerExpectation::only_the_named_invariants(),
        );
    }
    let images = collected.into_collected();
    let (conclusions, _run, _judged_afresh) = judge_the_collected_images(
        scenario_name,
        &images,
        ContainerCapacityKnobs::PRODUCT,
        JudgedAfreshRegardlessOfTheExpectations::OnlyIfAnImageExpectsIt,
    )?;
    let step_images = pool_images.step_images.len();
    let violations_of_the_step_images: Vec<Vec<(&'static str, String)>> = conclusions
        [..step_images]
        .iter()
        .map(pool_checker_violations_of_the_only_state)
        .collect();
    let first_red_step_image = violations_of_the_step_images
        .iter()
        .position(|violations| !violations.is_empty());
    let step_comparison_image = first_red_step_image
        .and_then(|index| {
            handed_out.image_at_the_step_image_with_the_swallowed_writes_before_it_landed(index)
        })
        .map(|stream| {
            scenarios_of_fault_injection_campaign::pool_image_of_a_recorded_stream(
                handed_out, &stream,
            )
        });
    let violations_at_the_step = match &step_comparison_image {
        Some(image) => {
            let mut collected_at_the_first_red_step_image =
                PoolCheckerImages::collected_for_the_given_states_engine();
            collected_at_the_first_red_step_image.hand_out(
                "第一张判红的每步镜像那一刻、那一刻之前被吞的写插回去的那张盘",
                || image.clone(),
                PoolCheckerExpectation::only_the_named_invariants(),
            );
            let (conclusions_at_the_first_red_step_image, _, _) = judge_the_collected_images(
                &format!("{scenario_name}/at-the-first-red-step-image"),
                &collected_at_the_first_red_step_image.into_collected(),
                ContainerCapacityKnobs::PRODUCT,
                JudgedAfreshRegardlessOfTheExpectations::OnlyIfAnImageExpectsIt,
            )?;
            Some(pool_checker_violations_of_the_only_state(
                &conclusions_at_the_first_red_step_image[0],
            ))
        }
        None => None,
    };
    let violation_sites_of_the_first_red_step_image = first_red_step_image
        .map(|index| {
            violations_naming_the_sites_by_the_cpu_reference(
                &pool_images.step_images[index],
                &violations_of_the_step_images[index],
                "第一张判红的每步镜像",
            )
            .violation_sites
        })
        .unwrap_or_default();
    let after_the_fault = violations_naming_the_sites_by_the_cpu_reference(
        &pool_images.after_the_fault,
        &pool_checker_violations_of_the_only_state(&conclusions[step_images]),
        "注入之后重开的那张盘",
    );
    let at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed =
        step_comparison_image
            .as_ref()
            .zip(violations_at_the_step.as_ref())
            .map(|(image, violations)| {
                violations_naming_the_sites_by_the_cpu_reference(
                    image,
                    violations,
                    "第一张判红的每步镜像的按步对照盘",
                )
            });
    let with_the_swallowed_writes_landed = pool_images
        .with_the_swallowed_writes_landed
        .as_ref()
        .map(|landed| {
            violations_naming_the_sites_by_the_cpu_reference(
                landed,
                &pool_checker_violations_of_the_only_state(&conclusions[step_images + 1]),
                "被吞的写都插回去之后的那张盘",
            )
        });
    Some(
        classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
            handed_out,
            PoolCheckerVerdictsOfTheImagesAfterAFault {
                violations_of_the_step_images,
                violation_sites_of_the_first_red_step_image,
                after_the_fault,
                at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed,
                with_the_swallowed_writes_landed,
            },
        ),
    )
}

/// 一张镜像经引擎判下来（一张镜像一个状态）池级 checker 判红的那几条。
fn pool_checker_violations_of_the_only_state(
    judged: &ConclusionsOfAnImage,
) -> Vec<(&'static str, String)> {
    let [state] = judged.states.as_slice() else {
        panic!("一张镜像一个状态，交回 {} 个", judged.states.len());
    };
    state.pool_checker_violations()
}

/// CPU 参照判同一张盘，交回引擎那一趟判红的那几条连同违例点名的（不变量, 盘, 槽）：GPU 判器只交违例位、交不回点名的槽，
/// 故障归类器按（不变量, 盘, 槽）比（判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3）要的槽由池级 checker 的 CPU 那一份交；
/// 判红的是哪几条与引擎那一趟（配了 GPU 是 GPU 判器）逐条对拍，对不上判红。引擎那一趟一条都不红时不判，点名为空。
// cpu-reference-for-gpu-comparison
fn violations_naming_the_sites_by_the_cpu_reference(
    image: &singlefs_verification::memory_pool::MemoryPool,
    violations_judged_through_the_engine: &[(&'static str, String)],
    which_image: &str,
) -> ViolationsNamingTheSites {
    if violations_judged_through_the_engine.is_empty() {
        return ViolationsNamingTheSites::default();
    }
    let named = check_pool_image_naming_the_violation_sites(image);
    let violated_on_the_cpu: BTreeSet<&'static str> = named
        .verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(*invariant),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    let violated_through_the_engine: BTreeSet<&'static str> = violations_judged_through_the_engine
        .iter()
        .map(|(invariant, _)| *invariant)
        .collect();
    assert_eq!(
        violated_on_the_cpu, violated_through_the_engine,
        "{which_image}：CPU 参照判红的与引擎那一趟判红的逐条相同"
    );
    ViolationsNamingTheSites {
        violations: violations_judged_through_the_engine.to_vec(),
        violation_sites: named.violation_sites,
    }
}

/// 与 `tests/fault_injection_campaign.rs` 里同名的用例同一个注入点（第一个文件之后覆盖写 23 次，根环转过一圈之后 txg 25 那次的
/// 根槽 FUA 写被吞，那一槽留着上一圈的根 txg 1；场景见 `scenarios_of_fault_injection_campaign::the_fixed_history_with_its_root_slot_write_swallowed`）：
/// 交出去的盘经引擎判池级 checker、交归类——txg 26 那次覆盖写之后 checker 只判红 I-3.1 与 I-7.18（洞里的 txg 1 把现算的回收门槛拉到
/// 最新根带的之下，记账少于下沿；被吞的写插回去就消失），按注入点认成说谎的设备留下的，不算新发现：带着注入跑的那一遍里第一张判红的
/// 每步镜像、重开的那张盘，各一条。I-3.1 改成对记录比（RCW）之前取样点是覆盖写五次、吞 txg 7 那次根槽写，RCW 之后那一段一条都不红。
#[test]
fn swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_and_the_reclaim_threshold_invariants_is_excused_as_what_the_lying_device_may_leave(
) {
    const TEST_NAME: &str = "swallowed_write_after_which_the_checker_flags_only_the_allocated_statistic_and_the_reclaim_threshold_invariants_is_excused_as_what_the_lying_device_may_leave";
    let (_injection, handed_out) =
        scenarios_of_fault_injection_campaign::the_fixed_history_with_its_root_slot_write_swallowed(
        );
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    let Some(failures) = failures_after_a_fault_judged_through_the_library(TEST_NAME, images, None)
    else {
        return;
    };
    let expected_signature = "CheckerViolations { invariants: [\"I-3.1\", \"I-7.18\"] }";
    assert_eq!(
        scenarios_of_fault_injection_campaign::rendered_failures_after_a_fault(&failures),
        vec![format!("说谎 {expected_signature}"), format!("说谎 {expected_signature}")],
        "被吞掉的那一次根槽写之后只判红 I-3.1 与 I-7.18、被吞的写插回去就消失，按注入点认成说谎的设备留下的——带着注入跑的那一遍里\
         第一张判红的每步镜像、重开的那张盘，各一条：{failures:?}"
    );
}

/// 与 `tests/fault_injection_campaign.rs` 里同名的用例同一个注入点（默认批种子基 + 7 那一段在整池第 1 次 barrier 上报错，
/// 落在 mkfs 第一道屏障上；场景见 `scenarios_of_fault_injection_campaign::the_history_whose_make_filesystem_fails_at_its_first_barrier`）：
/// 重开的盘上只红 I-8.5，归类记成故障注入的「已知红」那一条（mkfs 在系统配置槽落盘之前报错），不记新发现。
/// 判别力自证：判法改回不认这一类（`known_red_form_only_after_a_fault` 恒交回 None，`crates/mutations.tsv` 同名那一条），
/// 这一次落成新发现 `CheckerViolations { invariants: ["I-8.5"] }`，这里红。
#[test]
fn make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding(
) {
    const TEST_NAME: &str = "make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding";
    let (_injection, handed_out) =
        scenarios_of_fault_injection_campaign::the_history_whose_make_filesystem_fails_at_its_first_barrier();
    let [images] = handed_out.as_slice() else {
        panic!("一个注入点交出去一份盘：交了 {} 份", handed_out.len());
    };
    assert!(
        images.step_images_of_the_run_with_the_fault.is_empty(),
        "mkfs 自己报错，一张每步镜像都不交"
    );
    let Some(failures) = failures_after_a_fault_judged_through_the_library(
        TEST_NAME,
        images,
        Some(MAKE_FILESYSTEM_UNFINISHED),
    ) else {
        return;
    };
    assert_eq!(
        failures,
        vec![FailureAfterAFault::KnownRed(KnownRedFormAtAFault::OnlyAfterAFault(
            KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
        ))],
        "mkfs 在系统配置槽落盘之前报错、重开的盘上只红 I-8.5 是认下的「已知红」那一条，不是新发现"
    );
}

/// 与 `tests/fault_injection_campaign.rs` 里同名的用例同一个注入点（必绿世界 K2-1，判决 `research/prompts/m4-i315-r3-main-verification.md`
/// 第三节 C3「必红、必绿世界在入库装置上先验它」；场景见 `scenarios_of_fault_injection_campaign::the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot`）：
/// 被吞的写落在复用过的槽上，盘 1 那一份是上一个住户、实现释放之前读盘核出对不上隔离两份。交出去的盘经引擎判池级 checker、交归类：
/// 判红的每一处插回去（按步对照盘、被吞的写都插回去那张）都不红，每一条都归说谎的设备，不报新发现。
/// 判别力：按步对照盘不交进比较（每一条都算仍红，`crates/mutations.tsv` 同名那一条），这一点落成新发现，这里红。
#[test]
fn the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot_hands_out_the_quarantine_of_both_copies(
) {
    const TEST_NAME: &str = "the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot_hands_out_the_quarantine_of_both_copies";
    let images = scenarios_of_fault_injection_campaign::the_first_injection_point_of_the_history_swallowing_a_write_on_a_reused_slot();
    let Some(failures) =
        failures_after_a_fault_judged_through_the_library(TEST_NAME, &images, None)
    else {
        return;
    };
    assert!(
        !failures.is_empty(),
        "被吞的写落在复用过的槽上之后池级 checker 判红过（不然这一点什么也没测）"
    );
    assert!(
        failures
            .iter()
            .all(|failure| matches!(failure, FailureAfterAFault::LeftByTheLyingDevice { .. })),
        "必绿世界：每一条都归说谎的设备、不报新发现：{}",
        scenarios_of_fault_injection_campaign::rendered_failures_after_a_fault(&failures)
            .join("；")
    );
}

/// 与 `tests/fault_injection_campaign.rs` 里跑同一个场景函数的用例同一段写死的历史（必红世界：无关泄漏与吞写隔离同在一段历史里，判决
/// `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3；场景见
/// `scenarios_of_fault_injection_campaign::run_the_scenario_where_a_leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces`）。
/// 泄漏照 D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」造：池级 checker 只认记号，两块盘都带记号的未释放记录豁免 I-3.1、I-3.11、I-3.15，
/// 没有记号的已分配无引用记录照判违例。被吞的写逼出的隔离（Y 释放 X 那一版时隔离被吞的那一份）两块盘都带记号、照条款豁免；
/// 旁边 A 的数据单元那两条记录从 Y 起仍分配、两块盘都不带记号、抬 F 之后没有根引用，与被吞的写无关。
/// 归类两条：头一张判红的每步镜像（X 之后，A 那两条记录还带着记号）只红在被吞的写上，按步对照盘上一条都不红，归说谎的设备；
/// 重开那张红在被吞的写与泄漏两样上，被吞的写都插回去那张盘上泄漏在 A 那一槽照红（它不压在被吞的写上，不减），报新发现恰好一条。
/// 判别力：仍红的一条都不算（每一条都当被吞的写留下的，`crates/mutations.tsv` 同名那一条）时重开那张归成说谎的设备，这里红。
#[test]
fn leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces() {
    const TEST_NAME: &str =
        "leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces";
    /// 隔离判据那三条：没有记号的已分配无引用记录照判违例的三条（两块盘都带记号时豁免）。
    const INVARIANTS_OF_THE_QUARANTINE_CRITERION: [&str; 3] = ["I-3.1", "I-3.11", "I-3.15"];
    let images = scenarios_of_fault_injection_campaign::run_the_scenario_where_a_leak_unrelated_to_the_swallowed_write_beside_the_quarantine_the_swallowed_write_forces();
    let Some(failures) =
        failures_after_a_fault_judged_through_the_library(TEST_NAME, &images, None)
    else {
        return;
    };
    let rendered =
        scenarios_of_fault_injection_campaign::rendered_failures_after_a_fault(&failures);
    let [at_the_first_red_step_image, on_the_reopened_image] = failures.as_slice() else {
        panic!(
            "头一张判红的每步镜像一条、重开那张一条，判成 {}",
            rendered.join("；")
        );
    };
    let FailureAfterAFault::LeftByTheLyingDevice {
        signature:
            FailureSignature::CheckerViolations {
                invariants: invariants_left_by_the_swallowed_write,
            },
    } = at_the_first_red_step_image
    else {
        panic!(
            "头一张判红的每步镜像只红在被吞的写上，归说谎的设备：{}",
            rendered.join("；")
        );
    };
    assert!(
        !invariants_left_by_the_swallowed_write.is_empty()
            && INVARIANTS_OF_THE_QUARANTINE_CRITERION
                .iter()
                .all(|invariant| !invariants_left_by_the_swallowed_write.contains(invariant)),
        "归说谎的设备那一条是被吞的写留下的，隔离判据那三条一条都不在里面（A 那两条记录这时还带着记号）：{}",
        rendered.join("；")
    );
    let FailureAfterAFault::NewFinding { observation, .. } = on_the_reopened_image else {
        panic!(
            "必红世界：重开那张报新发现（与被吞的写无关的泄漏），判成 {}",
            rendered.join("；")
        );
    };
    let invariants_of_the_new_finding: BTreeSet<&str> = observation
        .violations
        .iter()
        .map(|(invariant, _)| *invariant)
        .collect();
    assert!(
        INVARIANTS_OF_THE_QUARANTINE_CRITERION
            .iter()
            .all(|invariant| invariants_of_the_new_finding.contains(invariant)),
        "报出来的那一条里有隔离判据那三条判出的泄漏：{observation:?}"
    );
}
