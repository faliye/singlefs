"""变异表的替换行与追加行：原文、替换文里的换行写成字面的 \\n（与 crates/mutations.tsv 同）。"""
import sys

def row(name, path, original, replacement, arguments, test):
    for field in (name, path, original, replacement, arguments, test):
        assert "\t" not in field
    return "\t".join([name, path, original.replace("\n", "\\n"), replacement.replace("\n", "\\n"), arguments, test])

MOUNT = "crates/singlefs-core/src/mount.rs"
RECOVERY = "crates/singlefs-core/src/recovery.rs"
FSYNC = "-p singlefs-harness --test second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version -- "
PARAMS = "-p singlefs-harness --test entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write -- "
REPLACEMENTS = [
    row("实二七（fsync 失败掉盘的核查，调度记录 2026-09-24 第三节；D2 已定项 13 挂载准入、D18 已定项 11 可写挂载的顺序与前提）：不看系统配置落不落后，跟得上这一版、只坏了一份的盘也被拒可写",
        MOUNT, "    if newest_system_configuration_journal_tail >= version_journal_position {\n        return None;\n    }\n",
        "    if false {\n        return None;\n    }\n",
        FSYNC + "current_device_one_with_one_corrupted_copy_still_mounts_writable", "current_device_one_with_one_corrupted_copy_still_mounts_writable"),
    row("实二七（fsync 失败掉盘的核查，调度记录 2026-09-24 第三节；D2 已定项 13 挂载准入、D18 已定项 11 可写挂载的顺序与前提）：只落后、单元都在的盘也被记成不带所选那一版",
        MOUNT, "    (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))",
        "    Some((newest_system_configuration_journal_tail, units_missing))",
        FSYNC + "device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies", "device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies"),
    row("实二七（fsync 失败掉盘的核查，调度记录 2026-09-24 第三节；D2 已定项 13 挂载准入、D18 已定项 11 可写挂载的顺序与前提）：字节是空的单元（N2：提示与映射都读不出的数据单元）也拿去读盘比，不接受零长度读的盘上被记成缺它",
        MOUNT, "        .filter(|unit| !unit.bytes.is_empty())\n", "        .filter(|_unit| true)\n",
        FSYNC + "device_one_behind_by_the_rotation_with_the_data_unit_unreadable_everywhere_still_mounts_writable", "device_one_behind_by_the_rotation_with_the_data_unit_unreadable_everywhere_still_mounts_writable"),
    row("实审A1b Q4：可写挂载不比每块盘系统配置里的本盘设备号与盘表给的身份",
        MOUNT, "                    *own_device_number_on_disk != identity_handed_in\n",
        "                    false && *own_device_number_on_disk != identity_handed_in\n",
        "-p singlefs-harness --test writable_mount_refuses_a_device_table_disagreeing_with_the_system_configuration -- mount_with_device_one_carrying_a_copy_of_device_zero_is_refused_before_any_write",
        "mount_with_device_one_carrying_a_copy_of_device_zero_is_refused_before_any_write"),
    row("实审A1b Q2：抬 F 那一串（准入与卸载共用）退回按调用方的参数建写入口、不核",
        MOUNT, "    let CallerInputsAgreeingWithTheDisk {\n        system_configuration,\n        parameters_of_the_pool,\n    } = caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;",
        "    let system_configuration = choose_system_configuration(&*devices)?;\n    let parameters_of_the_pool = parameters.clone();",
        PARAMS + "raising_the_floor_with_other_parameters_is_refused_before_any_write", "raising_the_floor_with_other_parameters_is_refused_before_any_write"),
    row("实审A1b Q2、Q4：准入抬 F 不核调用方的参数与盘表（F 已在上限那一格照样交回）",
        MOUNT, ") -> Result<RaiseToTheAdmissionCeiling, MountError> {\n    let caller_inputs_agreeing_with_the_disk =\n        caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;",
        ") -> Result<RaiseToTheAdmissionCeiling, MountError> {\n    let caller_inputs_agreeing_with_the_disk = CallerInputsAgreeingWithTheDisk {\n        system_configuration: choose_system_configuration(&*devices)?,\n        parameters_of_the_pool: parameters.clone(),\n    };",
        PARAMS + "raising_the_floor_to_the_admission_ceiling_with_a_third_device_is_refused_before_any_write", "raising_the_floor_to_the_admission_ceiling_with_a_third_device_is_refused_before_any_write"),
    row("实审A1b Q2：一次准入里再推一串不核调用方的参数（预算用完那一格照样交回）",
        MOUNT, ") -> Result<FloorRaisePushedWithinTheAdmissionBudget, MountError> {\n    let caller_inputs_agreeing_with_the_disk =\n        caller_inputs_agreeing_with_the_disk(parameters, devices, current)?;",
        ") -> Result<FloorRaisePushedWithinTheAdmissionBudget, MountError> {\n    let caller_inputs_agreeing_with_the_disk = CallerInputsAgreeingWithTheDisk {\n        system_configuration: choose_system_configuration(&*devices)?,\n        parameters_of_the_pool: parameters.clone(),\n    };",
        PARAMS + "pushing_a_floor_raise_with_another_filesystem_identifier_is_refused_before_the_budget_is_judged", "pushing_a_floor_raise_with_another_filesystem_identifier_is_refused_before_the_budget_is_judged"),
    row("A3a 第 38 条：固定结构槽距不判上界",
        RECOVERY, "        .is_none_or(|largest_spacing| fixed_structure_slot_spacing > largest_spacing)",
        "        .is_none_or(|largest_spacing| false && fixed_structure_slot_spacing > largest_spacing)",
        "-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- a_fixed_structure_slot_spacing_outside_the_format_range",
        "a_fixed_structure_slot_spacing_outside_the_format_range_is_refused"),
    row("实审 A3b Q6：抬 F 读根环时根槽重读仍读不出改回「当没有根」",
        RECOVERY, "            RootRingSlotReading::Bad(reread) => {\n                return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {\n                    ring_slot,\n                    first_reading,\n                    reread,\n                });\n            }",
        "            RootRingSlotReading::Bad(_reread) => {}",
        "-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over",
        "raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_unreadable_is_refused_before_any_write"),
]

if __name__ == "__main__":
    out = sys.argv[1]
    open(out, "w", encoding="utf-8").write("".join(line + "\n" for line in REPLACEMENTS))

SYSTEM_CONFIGURATION = "crates/singlefs-core/src/system_configuration.rs"
SESSION = "crates/singlefs-core/src/mounted_session.rs"
UNIT_AREA = "-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over"
ENTRIES = "-p singlefs-harness --test entries_after_mount_refuse_swapped_or_behind_devices"
KNOWN_SLOT_ANCHOR = "        if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {\n            continue;\n        }\n        match read_the_slot(ring_slot) {"
KNOWN_SLOT_NARROWED = "        if first_reading == BadRootRingSlotReading::NotSelfVerified || !ring_slots_known_to_hold_a_root.contains(&ring_slot) {\n            continue;\n        }\n        match read_the_slot(ring_slot) {"
SESSION_BEHIND = ("        if !devices_behind.is_empty() {", "        if false {")
ENTRY_BEHIND = ("    if !devices_behind.is_empty() {\n        return Err(CallerInputsCheckRefused::Disagreeing(",
                "    if false {\n        return Err(CallerInputsCheckRefused::Disagreeing(")
OWN_NUMBER_FILTER = ("                    *own_device_number_on_disk != identity_handed_in\n",
                     "                    false && *own_device_number_on_disk != identity_handed_in\n")
WITNESS = ("    if !first_read.witnesses_a_publish_newer_than_the_selected_version() {",
           "    if true || !first_read.witnesses_a_publish_newer_than_the_selected_version() {")

def entries(test):
    return (ENTRIES + " -- " + test, test)

APPENDS = [
    # 代码三方第二轮 Y2：挂着之后的入口再核两样
    row("代码三方第二轮 Y2-a：会话发布不核落后支（旧快照的盘照常收到本池的写）", SESSION, *SESSION_BEHIND,
        *entries("a_session_publish_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write")),
    row("代码三方第二轮 Y2-a：会话发布不核落后支（放开扫的 16 格）", SESSION, *SESSION_BEHIND,
        *entries("stale_snapshot_substitution_is_refused_by_every_entry_over_the_swept_user_steps")),
    row("代码三方第二轮 Y2-a：挂着之后收盘表的入口不核落后支（正常卸载）", MOUNT, *ENTRY_BEHIND,
        *entries("a_normal_unmount_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write")),
    row("代码三方第二轮 Y2-a：挂着之后收盘表的入口不核落后支（准入抬 F）", MOUNT, *ENTRY_BEHIND,
        *entries("a_floor_raise_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write")),
    row("代码三方第二轮 Y2-a：挂着之后收盘表的入口不核落后支（管理员回退）", MOUNT, *ENTRY_BEHIND,
        *entries("an_administrator_rollback_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write")),
    row("代码三方第二轮 Y2-b：会话发布不比本盘设备号（盘体对调照常发布）", SESSION,
        "        if !own_device_numbers_differing.is_empty() {", "        if false {",
        *entries("a_session_publish_with_the_two_device_bodies_swapped_is_refused_before_any_write")),
    row("代码三方第二轮 Y2-b：挂着之后收盘表的入口不比本盘设备号（正常卸载）", MOUNT, *OWN_NUMBER_FILTER,
        *entries("a_normal_unmount_with_the_two_device_bodies_swapped_is_refused_before_any_write")),
    row("代码三方第二轮 Y2-b：挂着之后收盘表的入口不比本盘设备号（准入抬 F）", MOUNT, *OWN_NUMBER_FILTER,
        *entries("a_floor_raise_with_the_two_device_bodies_swapped_is_refused_before_any_write")),
    row("代码三方第二轮 Y2-b：挂着之后收盘表的入口不比本盘设备号（管理员回退）", MOUNT, *OWN_NUMBER_FILTER,
        *entries("an_administrator_rollback_with_the_two_device_bodies_swapped_is_refused_before_any_write")),
    row("代码三方第二轮 Y2-c：落后支只看落后、不读单元（只剩一槽自证的盘被误拒）", MOUNT,
        "    (!units_missing.is_empty()).then_some((newest_system_configuration_journal_tail, units_missing))",
        "    Some((newest_system_configuration_journal_tail, units_missing))",
        *entries("a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry")),
    # 代码三方第二轮 Y4-a：core 读 325 / 371
    row("代码三方第二轮 Y4-a：读者不判根环起点等不等第一版常量（写 65 照挂）", RECOVERY,
        "    if recorded_root_ring_base_slot != ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION {", "    if false {",
        UNIT_AREA + " -- a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write",
        "a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write"),
    row("代码三方第二轮 Y4-a：读者不判 journal 环起点等不等第一版常量（写 1023 照挂）", RECOVERY,
        "    if recorded_journal_ring_start_slot != JOURNAL_RING_START_SLOT_OF_THE_FIRST_VERSION {", "    if false {",
        UNIT_AREA + " -- a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write",
        "a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write"),
    row("代码三方第二轮 Y4-a：判槽距上界退回编译期常量根环起点（根环起点写 0 那一格报成根环起点不等，不报槽距越界）", RECOVERY,
        "        || largest_fixed_structure_slot_spacing_under_the_root_ring_base(\n            system_configuration.immutable.root_ring_base_slot,\n        )",
        "        || largest_fixed_structure_slot_spacing_under_the_root_ring_base(\n            ROOT_RING_BASE_SLOT_OF_THE_FIRST_VERSION,\n        )",
        UNIT_AREA + " -- a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write",
        "a_system_configuration_field_the_reader_does_not_accept_is_refused_by_both_mounts_before_any_write"),
    row("代码三方第二轮 Y4-a：解槽读根环起点读错了位置（偏移 371 往前挪 8）", SYSTEM_CONFIGURATION,
        "                usize::try_from(ROOT_RING_BASE_SLOT_OFFSET).expect(\"371\"),",
        "                usize::try_from(ROOT_RING_BASE_SLOT_OFFSET - 8).expect(\"371\"),",
        UNIT_AREA + " -- the_chosen_system_configuration_carries_the_first_version_journal_ring_start_and_root_ring_base",
        "the_chosen_system_configuration_carries_the_first_version_journal_ring_start_and_root_ring_base"),
    row("代码三方第二轮 Y4-a：解槽读 journal 环起点读错了位置（偏移 325 往前挪 8）", SYSTEM_CONFIGURATION,
        "                usize::try_from(JOURNAL_RING_START_SLOT_OFFSET).expect(\"325\"),",
        "                usize::try_from(JOURNAL_RING_START_SLOT_OFFSET - 8).expect(\"325\"),",
        UNIT_AREA + " -- the_chosen_system_configuration_carries_the_first_version_journal_ring_start_and_root_ring_base",
        "the_chosen_system_configuration_carries_the_first_version_journal_ring_start_and_root_ring_base"),
    row("代码三方第二轮 Y4-a：写者写 journal 环起点写的不是内存里那一份（写成常量 + 1）", SYSTEM_CONFIGURATION,
        "            writer.put_u64(self.immutable.journal_ring_start_slot.0);",
        "            writer.put_u64(JOURNAL_RING_START_SLOT + 1);",
        UNIT_AREA + " -- the_chosen_system_configuration_carries_the_first_version_journal_ring_start_and_root_ring_base",
        "the_chosen_system_configuration_carries_the_first_version_journal_ring_start_and_root_ring_base"),
    # D16 已定项 1「根槽这一次读坏」全句
    row("代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽自证不过不重读、当没有根（只管读不出）", RECOVERY,
        KNOWN_SLOT_ANCHOR, KNOWN_SLOT_NARROWED,
        UNIT_AREA + " -- raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write",
        "raising_the_floor_while_a_root_ring_slot_known_to_hold_a_root_stays_not_self_verified_is_refused_before_any_write"),
    row("代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽自证不过不重读（管理员回退，step_four 那一形）", RECOVERY,
        KNOWN_SLOT_ANCHOR, KNOWN_SLOT_NARROWED,
        "-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write",
        "rolling_back_while_a_root_slot_this_process_wrote_reads_not_self_verified_is_refused_before_any_write"),
    row("代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽自证不过不重读（管理员回退，step_five 那一形）", RECOVERY,
        KNOWN_SLOT_ANCHOR, KNOWN_SLOT_NARROWED,
        "-p singlefs-harness --test second_transaction_step_five_reuse -- rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write",
        "rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write"),
    row("代码三方第二轮之后规格第 3 条（D16 第 37 行全句）：知道住着根的槽读坏不重读、第一遍坏就拒", RECOVERY,
        "        match read_the_slot(ring_slot) {\n            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),\n            RootRingSlotReading::Bad(reread) => {",
        "        match RootRingSlotReading::Bad(first_reading) {\n            RootRingSlotReading::SelfVerified(root) => roots.push((ring_slot, root)),\n            RootRingSlotReading::Bad(reread) => {",
        UNIT_AREA + " -- a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised",
        "a_root_ring_slot_known_to_hold_a_root_not_self_verified_once_is_read_on_the_reread_and_the_floor_is_raised"),
    # 调查三条红之后主 agent 定：fsync_drop 两格改钉乙；C579 定案之后 c366 改钉乙
    row("调查三条红（主 agent 定）：C554 乙关掉，读路径那一格（盘 1 每次读都报错）走到逐盘核、不再被乙拒", MOUNT, *WITNESS,
        "-p singlefs-harness --test second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version -- mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write",
        "mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write"),
    row("调查三条红（主 agent 定）：C554 乙关掉，盘 0 是空盘那一格走到逐盘核、不再被乙拒", MOUNT, *WITNESS,
        "-p singlefs-harness --test second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version -- blank_device_refuses_the_writable_mount_by_name_before_any_write",
        "blank_device_refuses_the_writable_mount_by_name_before_any_write"),
    row("C579（用户 2026-09-27 定：乙判不出按真）：乙把「判不出」按假处置，所选根自己那条记录读不出时可写挂载照做", MOUNT,
        "            WitnessedCounterComparison::Undecidable => true,", "            WitnessedCounterComparison::Undecidable => false,",
        "-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- c366_when_the_chosen_root_own_record_is_unreadable",
        "c366_when_the_chosen_root_own_record_is_unreadable_the_writable_mount_is_refused_by_the_undecidable_witness_and_the_read_only_mount_opens_the_chosen_root"),
]

WATERMARK_ROW = row("实三（D23 已定项 14「水位」）：回退那次的 inode 号水位只取环里读得出的根，不与内存里现行那一版的取 max", MOUNT,
    "    .map_or(current.inode_number_watermark(), |ring| {\n        ring.max(current.inode_number_watermark())\n    });",
    "    .map_or(current.inode_number_watermark(), |ring| ring);",
    UNIT_AREA + " -- the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root",
    "the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root")
REPLACEMENTS.append(WATERMARK_ROW)

DELETES = [
    "实二八（D2 已定项 13 挂载准入「可写设备数 ≥ w 的下限」、已定项 6「w ≥ 2 是硬下界」；调度记录 2026-09-24 第三节实二七交回 Q4）：读路径那一格（盘 1 每次读都报错）：没有自证过的系统配置的盘不记下，盘 1 不被点名",
]

if __name__ == "__main__" and len(sys.argv) > 2:
    open(sys.argv[2], "w", encoding="utf-8").write("".join(line + "\n" for line in APPENDS))
    open(sys.argv[3], "w", encoding="utf-8").write("".join(name + "\n" for name in DELETES))
    open(sys.argv[1], "w", encoding="utf-8").write("".join(line + "\n" for line in REPLACEMENTS))
