//! 池级 checker 只认隔离记号（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」，用户 2026-10-22 定落点甲、checker 只认记号，
//! 变体 `mark_only`）：两块盘都带记号的未释放记录 I-3.1、I-3.11、I-3.15 豁免、不看副本；新判据 I-3.17（隔离记号与引用相符）三句；
//! 没记号的已分配无引用记录照判违例。四个世界怎么搭见场景文件 `common_images_for_the_checker_tier/quarantine_mark_agrees_with_the_references.rs`；
//! 这里收下每个场景交的镜像就地判、核场景写下的那几样（GPU 与 CPU 对拍那一侧在 checker 档引同一份场景文件）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/quarantine_mark_agrees_with_the_references.rs"]
mod scenarios_of_quarantine_mark_agrees_with_the_references;
#[path = "common_images_for_the_checker_tier/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_quarantine_mark_agrees_with_the_references::{
    leave_the_account_as_the_writer_left_it, quarantined_unit_no_root_references,
};
use singlefs_checker::image::{InvariantVerdict, ViolationSite};
use singlefs_checker::walk::{check_pool_image, check_pool_image_naming_the_violation_sites};
use singlefs_core::address::DeviceIdentity;
use std::collections::BTreeSet;

/// 跑一个场景、收下它交的每一张镜像就地判，逐张核场景写下的「判成立」「判红」「不红」三样（这一份场景文件只用这三样）。
fn judge_every_image_the_scenario_hands_out(scenario: impl FnOnce(&mut PoolCheckerImages)) {
    let mut handed_out = PoolCheckerImages::collected_for_the_checker_tier();
    scenario(&mut handed_out);
    let collected = handed_out.into_collected();
    assert!(!collected.is_empty(), "场景至少交一张镜像");
    for image in collected {
        assert!(
            image.writes.is_empty() && image.expected.exactly_violated.is_none(),
            "{}：这一份场景只交整张镜像、只写三样期望",
            image.label
        );
        let verdicts = check_pool_image(&image.image);
        let verdict_of = |invariant: &str| {
            verdicts
                .iter()
                .find(|(candidate, _)| *candidate == invariant)
                .map(|(_, verdict)| verdict.clone())
                .unwrap_or_else(|| panic!("清单里有 {invariant}"))
        };
        for invariant in &image.expected.holds {
            assert_eq!(
                verdict_of(invariant),
                InvariantVerdict::Holds,
                "{}：{invariant} 该成立",
                image.label
            );
        }
        for invariant in &image.expected.violated {
            assert!(
                matches!(verdict_of(invariant), InvariantVerdict::Violated(_)),
                "{}：{invariant} 该判红，判成 {:?}",
                image.label,
                verdict_of(invariant)
            );
        }
        for invariant in &image.expected.not_violated {
            assert!(
                !matches!(verdict_of(invariant), InvariantVerdict::Violated(_)),
                "{}：{invariant} 不该判红，判成 {:?}",
                image.label,
                verdict_of(invariant)
            );
        }
    }
}

/// 场景见 `scenarios_of_quarantine_mark_agrees_with_the_references::a_quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies`。
#[test]
fn a_quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies() {
    judge_every_image_the_scenario_hands_out(
        scenarios_of_quarantine_mark_agrees_with_the_references::a_quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies,
    );
}

/// 场景见 `scenarios_of_quarantine_mark_agrees_with_the_references::a_quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence`。
#[test]
fn a_quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence() {
    judge_every_image_the_scenario_hands_out(
        scenarios_of_quarantine_mark_agrees_with_the_references::a_quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence,
    );
}

/// 场景见 `scenarios_of_quarantine_mark_agrees_with_the_references::a_record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence`。
#[test]
fn a_record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence(
) {
    judge_every_image_the_scenario_hands_out(
        scenarios_of_quarantine_mark_agrees_with_the_references::a_record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence,
    );
}

/// 场景见 `scenarios_of_quarantine_mark_agrees_with_the_references::a_quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back`。
#[test]
fn a_quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back() {
    judge_every_image_the_scenario_hands_out(
        scenarios_of_quarantine_mark_agrees_with_the_references::a_quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back,
    );
}

/// 场景见 `scenarios_of_quarantine_mark_agrees_with_the_references::a_unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence`。
#[test]
fn a_unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence(
) {
    judge_every_image_the_scenario_hands_out(
        scenarios_of_quarantine_mark_agrees_with_the_references::a_unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence,
    );
}

/// 违例点名的（不变量, 盘, 槽）（判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3：checker 交回违例点名的槽，故障归类器按它比）：
/// 不带记号的那一形上 I-3.1、I-3.11 点名「已分配、没有根引用、没被豁免」的那个单元在每块盘上那条记录的起点槽，I-3.15 点名判红的那两条记录，
/// 别的槽一个都不点。
#[test]
fn the_violations_of_a_quarantine_without_the_mark_name_the_slot_of_the_unit_on_each_device() {
    let without_the_mark = quarantined_unit_no_root_references(
        "quarantine-mark-violation-sites-without-the-mark",
        |allocator, placement| {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                allocator.clear_the_quarantine_mark_of_a_record_the_rollback_target_references(
                    device, placement,
                );
            }
        },
        leave_the_account_as_the_writer_left_it,
    );
    let leak = without_the_mark.pool.memory_pool();
    let slot = without_the_mark.data_slot.0;
    let named_in_the_three: BTreeSet<ViolationSite> =
        check_pool_image_naming_the_violation_sites(&leak)
            .violation_sites
            .into_iter()
            .filter(|site| ["I-3.1", "I-3.11", "I-3.15"].contains(&site.invariant))
            .collect();
    let expected: BTreeSet<ViolationSite> = ["I-3.1", "I-3.11", "I-3.15"]
        .into_iter()
        .flat_map(|invariant| {
            [0, 1].into_iter().map(move |device| ViolationSite {
                invariant,
                device,
                slot,
            })
        })
        .collect();
    assert_eq!(
        named_in_the_three, expected,
        "不带记号的那个单元在两块盘上的起点槽，三条各点名一次"
    );
}

/// 两块盘都带记号的那一形，盘 0 那一份换成诞生代号不等的旧住户（E 的数据单元）：I-3.10 点名盘 0 那条记录，三条隔离判据一个槽都不点（豁免不看副本，
/// 也不静默放过：代号不等由 I-3.10 判）。
#[test]
fn an_old_tenant_under_a_quarantine_carrying_the_mark_is_named_by_the_birth_invariant_only() {
    let marked = quarantined_unit_no_root_references(
        "quarantine-mark-violation-sites-old-tenant",
        leave_the_account_as_the_writer_left_it,
        leave_the_account_as_the_writer_left_it,
    );
    let mut old_tenant = marked.pool.memory_pool();
    let tenant = scenarios_of_quarantine_mark_agrees_with_the_references::data_unit_in_the_image(
        &old_tenant,
        DeviceIdentity(0),
        marked.newest_data_slot,
    );
    scenarios_of_quarantine_mark_agrees_with_the_references::replace_the_copy_in_the_image(
        &mut old_tenant,
        DeviceIdentity(0),
        marked.data_slot,
        &tenant,
    );
    let sites = check_pool_image_naming_the_violation_sites(&old_tenant).violation_sites;
    assert!(
        sites.contains(&ViolationSite {
            invariant: "I-3.10",
            device: 0,
            slot: marked.data_slot.0,
        }),
        "代号不等的那条记录 I-3.10 点名它：{sites:?}"
    );
    assert!(
        sites
            .iter()
            .all(|site| !["I-3.1", "I-3.11", "I-3.15"].contains(&site.invariant)),
        "两块盘都带记号的隔离豁免，三条隔离判据一个槽都不点：{sites:?}"
    );
}
