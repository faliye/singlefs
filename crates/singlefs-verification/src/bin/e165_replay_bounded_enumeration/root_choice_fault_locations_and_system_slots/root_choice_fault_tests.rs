use super::*;

#[test]
fn the_mount_fault_menu_has_thirteen_entries_in_the_registered_order_and_only_rnsb_is_outside_the_promise(
) {
    let world = FaultWorld::right_after_mkfs(RootChoiceDomain::BigRing);
    let menu = mount_fault_menu(&world, 2);
    let labels: Vec<&str> = menu.iter().map(|entry| entry.label()).collect();
    assert_eq!(labels, MOUNT_FAULT_MENU_LABELS);
    let outside_the_promise: Vec<&str> = menu
        .iter()
        .filter(|entry| !entry.is_inside_the_single_fault_promise())
        .map(|entry| entry.label())
        .collect();
    assert_eq!(outside_the_promise, ["RNSB-T1", "RNSB-TP"]);
    assert_eq!(mount_fault_menu(&world, 1).len(), 1, "挂载 1 开头没有故障");
    for entry in &menu {
        if entry.label().ends_with("-TP") {
            assert_eq!(
                entry.every_read_unreadable,
                entry.first_read_unreadable,
                "TP 每一遍都读不出：{}",
                entry.label()
            );
        } else {
            assert_eq!(
                entry.every_read_unreadable,
                FaultLocations::NOWHERE,
                "T1 只有第一遍读不出（「无」那一项也没有落点）：{}",
                entry.label()
            );
        }
    }
}

#[test]
fn system_configuration_write_takes_the_generation_after_the_largest_one_it_can_read() {
    let mut slots = system_slots_right_after_mkfs();
    let (first_write_slot, first_write_content) =
        system_slot_write_of(&slots, 0, 7, 3, FaultLocations::NOWHERE);
    assert_eq!(first_write_slot, 0);
    assert_eq!(
        first_write_content,
        SystemSlotContent::Verified {
            generation: 2,
            tail: 7,
            instance: 3
        }
    );
    slots[0][0] = first_write_content;
    let (second_write_slot, second_write_content) =
        system_slot_write_of(&slots, 0, 8, 3, FaultLocations::NOWHERE);
    assert_eq!(
        (second_write_slot, second_write_content),
        (
            1,
            SystemSlotContent::Verified {
                generation: 3,
                tail: 8,
                instance: 3
            }
        )
    );
    let unreadable_slot_zero = FaultLocations::NOWHERE.with_system_slot(0, 0);
    let (write_slot_beside_the_unreadable_one, write_content_beside_the_unreadable_one) =
        system_slot_write_of(&slots, 0, 9, 4, unreadable_slot_zero);
    assert_eq!(
        (
            write_slot_beside_the_unreadable_one,
            write_content_beside_the_unreadable_one
        ),
        (
            0,
            SystemSlotContent::Verified {
                generation: 2,
                tail: 9,
                instance: 4
            }
        ),
        "一直读不出的槽不算进最大世代号：只剩世代号 1 的那一槽，写 2"
    );
    slots[1][1] = SystemSlotContent::Torn;
    slots[1][0] = SystemSlotContent::Torn;
    let (_slot, write_content_when_no_slot_reads) =
        system_slot_write_of(&slots, 1, 1, 1, FaultLocations::NOWHERE);
    assert_eq!(
        write_content_when_no_slot_reads,
        SystemSlotContent::Verified {
            generation: 1,
            tail: 1,
            instance: 1
        },
        "一槽都读不出时世代号从 1 起"
    );
}

// ── r5 修订 1 第五段（`--gate r2`；登记 `research/prompts/e165-r5-prereg.md` 修1.七甲类、修1.八、修1.九甲表）

/// 一份手造的系统配置槽：`torn` 里列的 (盘, 槽) 撕裂，其余是 mkfs 那一份自证过的。
fn system_slots_with_torn(torn: &[(usize, usize)]) -> SystemSlots {
    let mut slots = system_slots_right_after_mkfs();
    for (device, slot) in torn {
        slots[*device][*slot] = SystemSlotContent::Torn;
    }
    slots
}

/// 读视图里报读错的 (盘, 槽)。
fn locations_with_read_errors(read_errors: &[(usize, usize)]) -> FaultLocations {
    read_errors
        .iter()
        .fold(FaultLocations::NOWHERE, |locations, (device, slot)| {
            locations.with_system_slot(*device, *slot)
        })
}

/// 修1.七甲类「判不出的真值」（D22（单元原子性怎么合成） 已定项 8 射程 R2 原句与 S-辨那一句）：一槽撕裂而没有槽报读错 ⇒ 判得出；
/// 一块盘报读错而另一块盘两槽都自证过 ⇒ 判得出；两块盘各有一槽报读错 ⇒ 判不出；一块盘报读错、另一块盘一槽撕裂 ⇒ 判不出。
/// 9.2「撕裂也算报读错」「R2 后一半拿掉」「两槽都自证过不看撕裂」点名这一条。
#[test]
fn the_read_error_rule_undecidability_holds_only_when_a_slot_reports_a_read_error_and_no_device_self_verified_both(
) {
    for (torn, read_errors, undecidable, case) in [
        (
            vec![(0, 0), (1, 0)],
            vec![],
            false,
            "两块盘各一槽撕裂、没有槽报读错",
        ),
        (
            vec![],
            vec![(0, 0)],
            false,
            "0 号盘一槽报读错、1 号盘两槽都自证过",
        ),
        (vec![], vec![(0, 0), (1, 1)], true, "两块盘各一槽报读错"),
        (
            vec![(1, 0)],
            vec![(0, 0)],
            true,
            "0 号盘一槽报读错、1 号盘一槽撕裂",
        ),
        (vec![], vec![], false, "全都自证过"),
    ] {
        assert_eq!(
            system_configuration_witness_is_undecidable_under_the_read_error_rule(
                &system_slots_with_torn(&torn),
                locations_with_read_errors(&read_errors)
            ),
            undecidable,
            "{case}"
        );
    }
}

/// 修1.七甲类「取号那一刻：某块盘两槽一份自证过的都没有 ⇒ 拒（即使判得出）」：0 号盘一槽报读错、另一槽撕裂，1 号盘两槽都自证过。
/// 9.2「取号那一刻不判 (b)」点名这一条。
#[test]
fn acquisition_refuses_a_device_without_a_self_verified_slot_even_when_the_witness_is_decidable() {
    let mut world = FaultWorld::right_after_mkfs_under_the_gate(
        RootChoiceDomain::BigRing,
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
    );
    world.system_slots = system_slots_with_torn(&[(0, 1)]);
    let locations = locations_with_read_errors(&[(0, 0)]);
    assert!(
        !system_configuration_witness_is_undecidable_under_the_read_error_rule(
            &world.system_slots,
            locations
        )
    );
    assert_eq!(
        acquisition_refusal_of(&world, locations),
        Some(MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot)
    );
}

/// 修1.九甲表「取号那一刻不判 (a)」的手造单测（界内没有取样点）：跳过闸，直接对「两块盘各一槽报读错」的读视图判取号那一刻，必须拒在判不出那一道。
#[test]
fn acquisition_refuses_an_undecidable_witness_when_each_device_reports_a_read_error_on_one_slot() {
    let world = FaultWorld::right_after_mkfs_under_the_gate(
        RootChoiceDomain::BigRing,
        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots,
    );
    assert_eq!(
        acquisition_refusal_of(&world, locations_with_read_errors(&[(0, 0), (1, 0)])),
        Some(MountRefusalReason::AcquisitionWitnessUndecidable)
    );
    assert_eq!(
        acquisition_refusal_of(&world, FaultLocations::NOWHERE),
        None,
        "四槽都自证过时取号那一刻不拒"
    );
}
