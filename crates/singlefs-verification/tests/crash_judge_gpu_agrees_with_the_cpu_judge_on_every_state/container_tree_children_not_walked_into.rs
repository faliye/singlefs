//! 容器树四处走读不往下读的孩子、容器树两处另一份副本自证成别的类、候选根的实例表链途经已走过的片。

use super::*;

// ─── 容器树四处走读不往下读的孩子（含 F-L0）、容器树两处 F-W、inode 类型段 0 另一份自证成码 2、候选根链途经已走过的片 ───

/// F-L0（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 第三支那一行、第三节第 1 条；harness 坏镜像
/// `a_tombstone_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
/// 同一形，录成流）：删除中间态的流上墓碑树根（层级 1）第一条条目类型段 1 → 0、身份三段清零、子指针两条位置条目的整单元校验和各翻一字节，
/// 重封到根；墓碑容器不动。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity_over_two_wrong_location_checksums(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-zero-no-identity-two-location-checksums-wrong",
        &is_the_tombstone_tree_root,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, 0);
            clear_the_identity_of_the_first_entry_but_its_type(bytes);
            for location in POINTER_LOCATION_ENTRY_OFFSETS {
                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
            }
        },
    )
}

/// F-L0：孩子两份按头里的码 3 都对不上、不是另一类，走类型段 0 那一支；身份引用那一半成立，层级 1 的节点下面不往下走、记走读失败，
/// 每一份副本照判 I-2.1：CPU 池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4；GPU 走读内核
/// `w_walk_container_tree_root` 类型段 0、`n.level <= 1u` 那一处同改，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity_over_two_wrong_location_checksums(
            "crash-judge-gpu-i21-tombstone-root-entry-type-zero-no-identity-two-location-checksums-wrong",
        ),
        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
    );
}

/// 墓碑树那一侧的 H-J3a（harness 坏镜像
/// `a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
/// 同一形，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-seven-two-location-checksums-wrong",
        &is_the_tombstone_tree_root,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
            for location in POINTER_LOCATION_ENTRY_OFFSETS {
                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
            }
        },
    )
}

/// 容器树类型段不认得那一支（`w_walk_container_tree_root` 里 `entry_type != record_type` 那一处）：CPU 红 I-9.23、I-2.1、I-4.8、I-7.2、I-7.4，
/// GPU 每个状态逐位相同（判决同一份第二节 K1 L1 测试尖度那一行：容器树这一处调用没有对拍流）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-two-location-checksums-wrong",
        ),
        &["I-9.23", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
    );
}

/// 墓碑树那一侧的 H-J3b（harness 坏镜像
/// `a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant` 同一形，录成流）：
/// 删除中间态的流上墓碑树根第一条条目类型段 1 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；墓碑容器不动。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-zero-first-location-checksum-wrong",
        &is_the_tombstone_tree_root,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, 0);
            flip_a_byte_of_a_location_checksum_in_the_first_entry(
                bytes,
                POINTER_LOCATION_ENTRY_OFFSETS[0],
            );
        },
    )
}

/// 容器树类型段 0、孩子是另一类那一支（`class_of_the_child_of_an_internal_entry != NONE` 那一处）：第二份自证成码 3，不往下走；
/// 第一份对不上，照判 I-2.1：CPU 红 I-9.23、I-2.1、I-4.8、I-7.4（第二份对得上、不记走读失败），GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
            "crash-judge-gpu-i21-tombstone-root-entry-type-zero-first-location-checksum-wrong",
        ),
        &["I-9.23", "I-2.1", "I-4.8", "I-7.4"],
    );
}

/// 容器树类型段 = 记录类型、孩子是另一类那一支（`class_of_the_child_of_a_container_entry != NONE` 那一处）：
/// [`record_a_tombstone_tree_grown_to_level_two`] 的流上层级 2 的墓碑树根第一条条目类型段 0 → 1、子指针第一条位置条目的整单元校验和翻一字节，
/// 重封到根。孩子是层级 1 的码 2 节点：第一份按 16 KiB 对不上，第二份自证成码 2、没有一份是码 3，不往下走、按一槽登记引用。
fn record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container_over_a_wrong_first_location_checksum(
    tag: &str,
) -> RecordedFlow {
    let name = "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container-first-location-checksum-wrong";
    let mut flow = record_a_tombstone_tree_grown_to_level_two(tag, name, false);
    let targets = last_unit_copies(&flow, name, &is_a_tombstone_tree_root_of_level_two, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        set_the_type_of_the_first_entry(bytes, PACKED_RECORD_TYPE_TOMBSTONE);
        flip_a_byte_of_a_location_checksum_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
        );
    });
    flow
}

/// 见上：CPU 红 I-9.23、I-2.1（第一份那一份；第二份对得上、不记走读失败），GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong",
        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container_over_a_wrong_first_location_checksum(
            "crash-judge-gpu-i21-level-two-entry-names-a-node-first-location-checksum-wrong",
        ),
        &["I-9.23", "I-2.1"],
    );
}

/// 墓碑容器头里的记录宽 40 → 39：I-1.7 记录宽那一步判红（重封归调用方）。
pub(super) fn set_the_record_width_of_a_tombstone_container_one_below(bytes: &mut [u8]) {
    let width = read_u16_at(bytes, PACKED_UNIT_RECORD_WIDTH_OFFSET);
    assert_eq!(
        usize::from(width),
        TOMBSTONE_RECORD_BYTES,
        "墓碑容器的记录宽是登记的 40"
    );
    bytes[PACKED_UNIT_RECORD_WIDTH_OFFSET..PACKED_UNIT_RECORD_WIDTH_OFFSET + 2]
        .copy_from_slice(&(width - 1).to_le_bytes());
}

/// 删除中间态的流上墓碑容器两份的记录宽 40 → 39（重封、沿引用链补到根），再把盘序大的那一份的前 16 KiB 换成墓碑树根（码 2）的原样字节，
/// 墓碑树根第一条条目子指针第二条位置条目的整单元校验和换成那 16 KiB 的 CRC、`change_the_first_entry` 再改一处，重封到根
/// （攻方 `research/prompts/m3-batch28-code-r1-opus-output.md` 第 1.1 节墓碑那一形；harness 坏镜像
/// `a_tombstone_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device`
/// 位置条目 1 那一形，录成流）。盘序小的那一份是码 3、带违例、按它的位置条目自证得过。
fn record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
    tag: &str,
    name: &'static str,
    change_the_first_entry: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &container_copies,
        &set_the_record_width_of_a_tombstone_container_one_below,
    );
    let second_container_copy =
        if flow.writes[container_copies[0]].device.0 < flow.writes[container_copies[1]].device.0 {
            container_copies[1]
        } else {
            container_copies[0]
        };
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    let node_bytes = flow.writes[root_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(
        unit_class(&node_bytes),
        2,
        "换进去的那 16 KiB 头里的类是码 2"
    );
    let second_location = first_entry_of_a_container_number_node()
        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_LOCATION_ENTRY_OFFSETS[1];
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &node_bytes[first_entry_of_a_container_number_node()
                + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[1],
        )
        .0,
        flow.writes[second_container_copy].device.0,
        "第 1 条位置条目是盘序大的那块盘"
    );
    let checksum_of_the_node_bytes = crc32c(&node_bytes);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        let field = second_location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
        bytes[field..field + 4].copy_from_slice(&checksum_of_the_node_bytes.to_le_bytes());
        change_the_first_entry(bytes);
    });
    let WrittenContents::Bytes(container) = &mut flow.writes[second_container_copy].contents else {
        panic!("单元写是普通写");
    };
    container[..node_bytes.len()].copy_from_slice(&node_bytes);
    flow
}

/// 墓碑树根第一条条目不再改别处（类型段照旧是 1）。
fn leave_the_first_entry_as_it_is(_bytes: &mut [u8]) {}

/// 容器树类型段 = 记录类型那一处的 F-W：孩子盘 1 那一份自证成码 2，另一份是码 3、
/// 按 32 KiB 自证得过：CPU 认出码 2、I-9.23 红，再照类型段 1 那一支走盘 0 那一份、判出 I-1.7（见别类就停的话 I-1.7 报成立）；
/// GPU 走读内核 `class_of_the_child_of_a_container_entry = NONE` 那一处同改，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node",
        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
            "crash-judge-gpu-i923-tombstone-container-record-width-second-copy-verifying-as-a-node",
            "deletion-tombstone-container-record-width-second-copy-verifying-as-a-node",
            &leave_the_first_entry_as_it_is,
        ),
        &["I-9.23", "I-1.7"],
    );
}

/// 容器树类型段 0 那一处的 F-W：同上一条的镜像，墓碑树根第一条条目再把类型段 1 → 0。孩子盘 0 那一份自证成码 3（另一类），
/// 盘 1 那一份就是类型段 0 那一类（码 2）、按 16 KiB 自证得过：CPU 判 I-9.23 红，照类型段 0 那一支走那一份——层级 1 的节点下面不往下走、
/// 两份照判 I-2.1（都对得上）、记走读失败，红 I-7.2（见别类就停的话不记走读失败、I-7.2 不红）；GPU 走读内核
/// `class_of_the_child_of_an_internal_entry = NONE` 那一处同改，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node",
        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
            "crash-judge-gpu-i923-tombstone-entry-type-zero-container-second-copy-verifying-as-a-node",
            "deletion-tombstone-entry-type-zero-container-second-copy-verifying-as-a-node",
            &|bytes| set_the_type_of_the_first_entry(bytes, 0),
        ),
        &["I-9.23", "I-7.2"],
    );
}

/// GPU 结论第 1 字：恢复内核判不了（第 0 字带 [`verdict_bits::UNDECIDABLE`]）时是它的原因（`crash_judge_recovery.wgsl` 写结论那一段）。
/// 恢复内核的原因只有不带站点的 1–8（`crash_judge_common.wgsl` 的 `UNDECIDABLE_*`），池级 checker 原因字（[`POOL_CHECKER_UNDECIDABLE_WORD`]）
/// 多半带站点（低 8 位是原因、其上是站点号），是两套码：同值的几个只是两边共用同一个常量，不能拿一张码表同时比两个字。
const GPU_RECOVERY_UNDECIDABLE_REASON_WORD: usize = 1;

/// GPU 恢复内核沿实例表链读满 256 片、最后一片还说有下一片时记的原因（`crash_judge_common.wgsl` 的 `UNDECIDABLE_TOO_DEEP`）。
/// 恢复内核沿链读的三处（`crash_judge_common.wgsl` 的 `instance_table_chain_readable`、`crash_judge_recovery.wgsl` 的
/// `load_rows_of_the_instance_table_chain` 与 `load_instance_table_of_the_effective_root`）都设 256 片的上限；CPU 读链不设上限，
/// 链更长的状态按设计交 CPU（`load_rows_of_the_instance_table_chain` 上面那段注释：「链长过 256 片的记判不了」）。
pub(super) const GPU_RECOVERY_UNDECIDABLE_TOO_DEEP: u32 = 4;

/// 「判不了就回落 CPU」那几条用例里恢复内核那一半：恢复内核判不了的状态 `gpu_verdict_items` 也交回 None、回落 CPU，
/// 只看池级 checker 原因字的话这种状态既不计数、也不断言。原因不在 `recovery_reasons_left_to_the_cpu`（恢复内核自己那套码）里
/// 就交回一句不一致（给空的就是这条流上恢复内核每个状态都该判得了）。
pub(super) fn recovery_kernel_left_the_state_to_the_cpu_for_an_unlisted_reason(
    ordinal: u64,
    gpu_words: &[u32; VERDICT_WORDS],
    recovery_reasons_left_to_the_cpu: &[u32],
) -> Option<String> {
    let recovery_reason = gpu_words[GPU_RECOVERY_UNDECIDABLE_REASON_WORD];
    (gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
        && !recovery_reasons_left_to_the_cpu.contains(&recovery_reason))
    .then(|| {
        format!(
            "状态 {ordinal}：GPU 恢复内核判不了、原因 {recovery_reason:#x} 不在 {recovery_reasons_left_to_the_cpu:#x?} 里（这条流只许因点名的原因交 CPU）"
        )
    })
}

/// GPU 走读内核记判不了「类型段 0 的内部节点」时交回的原因字（`crash_judge_common.wgsl` 的 `UNDECIDABLE_TOO_DEEP`）。
const GPU_UNDECIDABLE_TOO_DEEP: u32 = 4;

/// 逐状态比 GPU 与 CPU，GPU 池级 checker 只许在 `reasons_left_to_the_cpu` 这几种原因字上判不了：判了的，违例位与 CPU 逐位相同；
/// 判不了的，原因字在 `reasons_left_to_the_cpu` 里；恢复内核判不了的，第 1 字的原因要在 `recovery_reasons_left_to_the_cpu` 里
/// （恢复内核自己那套码，见 [`GPU_RECOVERY_UNDECIDABLE_REASON_WORD`]）；至少一个状态的原因字在 `reasons_one_state_must_carry` 里；
/// 照流水线那样判不了就回落 CPU（`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。
/// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
pub(super) fn assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
    test_name: &str,
    flow: &RecordedFlow,
    reasons_left_to_the_cpu: &[u32],
    recovery_reasons_left_to_the_cpu: &[u32],
    reasons_one_state_must_carry: &[u32],
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        flow,
        expand_up_to(),
        &[StateExamination::LeftToTheCpuOnlyFor {
            reasons_left_to_the_cpu: reasons_left_to_the_cpu.to_vec(),
            recovery_reasons_left_to_the_cpu: recovery_reasons_left_to_the_cpu.to_vec(),
            reasons_one_state_must_carry: reasons_one_state_must_carry.to_vec(),
            invariant_red_on_a_state_left_to_the_cpu: None,
        }],
    ) else {
        return;
    };
    let left_to_the_cpu = &comparison.examined[0];
    eprintln!(
        "CRASH_JUDGE_GPU_LEFT_TO_THE_CPU flow={} states={} left_to_the_cpu={} carrying_a_required_reason={}",
        flow.name,
        comparison.states,
        left_to_the_cpu.states_left_to_the_cpu,
        left_to_the_cpu.states_counted
    );
    assert!(
        left_to_the_cpu.states_counted > 0,
        "流 {}：至少一个状态上 GPU 池级 checker 要记判不了、原因字在 {reasons_one_state_must_carry:#x?} 里",
        flow.name
    );
    assert!(
        left_to_the_cpu.disagreements.is_empty(),
        "流 {}：{} 处不一致（前几个）：\n{}",
        flow.name,
        left_to_the_cpu.disagreements.state_count(),
        left_to_the_cpu
            .disagreements
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// M 交回第 3 条那一形：新池新建文件的流上 inode 叶容器盘序大的那一份换成 inode 树根（码 2）的原样字节、位置条目跟着换
/// （[`hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf`]），再把 inode 树根第一条条目类型段 2 → 0、重封到根。
fn record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_leaf_whose_second_copy_holds_the_inode_root_bytes(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-inode-root-entry-type-zero-leaf-second-copy-verifying-as-a-node";
    let mut flow = record_new_pool_file_creation(tag, name);
    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
        &mut flow,
        name,
        ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
    );
    let root_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, 0);
    });
    flow
}

/// M 交回第 3 条：inode 树根条目类型段 0，孩子盘 0 那一份自证成码 3（另一类）、盘 1 那一份就是码 2、按 16 KiB 自证得过。
/// CPU 判 I-9.2 红、照类型段 0 那一支走那一份（F-W）；GPU 走读内核 inode 树只接层级 1、类型段 2，F-W 之后落到「类型段 0 的内部节点」那一句，
/// 池级 checker 记判不了（[`GPU_UNDECIDABLE_TOO_DEEP`]），交回 CPU 判。逐状态：判了的与 CPU 逐位相同，判不了的只许是这一种原因字、至少一个状态是，
/// 回落 CPU 之后红项相同。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node(
) {
    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
        "the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node",
        &record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_leaf_whose_second_copy_holds_the_inode_root_bytes(
            "crash-judge-gpu-i92-inode-root-entry-type-zero-leaf-second-copy-verifying-as-a-node",
        ),
        &[GPU_UNDECIDABLE_TOO_DEEP],
        // 恢复内核这条流上每个状态都判得了（判不了的 9 个状态都只是池级 checker 记的）；
        // 池级 checker 那一张里的 4 是走读内核的原因字，不是恢复内核的码
        &[],
        &[GPU_UNDECIDABLE_TOO_DEEP],
    );
}

/// 最新根那张实例表改造出来的第 1 片写在哪一槽：在 [`FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST`] 起那一截（候选根的新链占着）之前，
/// 同样是流里没写过、基镜像里全零的那一截（写之前由 [`writes_of_instance_table_pages`] 核）。
const SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE: u64 = 199_000;

/// 最新根那条链第 1 片的链指针记录指到候选根那条链的第几片（攻方 `research/prompts/m3-batch29-code-r1-opus-output.md` 第四节 `share=(1, 100)`）。
const PAGE_OF_THE_CANDIDATE_CHAIN_THE_NEWEST_CHAIN_RUNS_INTO: usize = 100;

/// 实例 3（崩溃恢复抛弃 C 之后那次可写挂载取的号）：它写行那次发布的根是候选根，最后一次暖机的根是最新根。
const INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY: u32 = 3;

/// 根记录里写序实例（4 字节）的偏移。
const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;

/// 攻方 L2 那一形（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K1 L2 两句走不走得到那一行、第三节第 3 条；
/// 攻方 `research/prompts/m3-batch29-code-r1-opus-output.md` 第四节 `newest=… other=257 role=candidate share=(1, 100)`）：
/// 被崩溃恢复抛弃 C 的那一段历史（[`record_a_root_abandoned_by_a_crash_recovery`]）里，实例 3 的根都指着实例 3 写行那次写的单片实例表 T。
/// 实例 3 写行那条根 R（回退候选集里、不是最新根）改指一条 257 片的新链：第 0 片放 T 的行，其余各片只有链指针记录（头照 T）；
/// R 的根槽写与它那次发布的 journal 记录新根段里的实例表指针改指新链第 0 片、重封，新链各片两份另起一段插在它们之前。
/// 再把 T 改成两片：第 0 片放原来的行、链指针记录指第 1 片，第 1 片只有一条链指针记录、指 R 那条链的第 100 片（容器号 100，
/// 在最新根那条链上是第 2 片：身份不对，坏镜像），T 重封到根，第 1 片两份另起一段插在写 T 那一段之前。
/// GPU 走读内核在上下文 0 里先走最新根的链、走到 R 那条链的第 100 片标成走过，再走候选根 R 的链走到第 100 片合并、停，不记站点 61；
/// 扫描内核从头读 R 的链、读满 256 片链还没完，记判不了（站点 37 或 38）。CPU 一侧先钉流造对了：整条落盘的镜像上最新根那条链第 2 片身份不对，I-1.1 红。
fn record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it(
    tag: &str,
) -> RecordedFlow {
    let FlowWithARootAbandonedByACrashRecovery { mut flow, .. } =
        record_a_root_abandoned_by_a_crash_recovery(
            tag,
            "candidate-root-own-instance-table-of-257-pages-the-newest-chain-runs-into",
        );
    let root_slot_writes_of_the_mount_after_the_recovery: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::RootRecordFua
                && flow.writes[*write_index].bytes().is_some_and(|root| {
                    read_u32_at(root, ROOT_RECORD_INSTANCE_OFFSET)
                        == INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY
                })
        })
        .collect();
    let txg_of = |recorded_flow: &RecordedFlow, write_index: usize| {
        read_u64_at(
            recorded_flow.writes[write_index]
                .bytes()
                .expect("根槽写带字节"),
            ROOT_RECORD_CHECKPOINT_TXG_OFFSET,
        )
    };
    let candidate_root_txg = root_slot_writes_of_the_mount_after_the_recovery
        .iter()
        .map(|write_index| txg_of(&flow, *write_index))
        .min()
        .expect("实例 3 至少写一次根槽");
    let newest_root_txg = root_slot_writes_of_the_mount_after_the_recovery
        .iter()
        .map(|write_index| txg_of(&flow, *write_index))
        .max()
        .expect("实例 3 至少写一次根槽");
    assert!(
        candidate_root_txg < newest_root_txg,
        "实例 3 写行那条根之后还有暖机的根：R 不是最新根"
    );
    let pointer_of = |recorded_flow: &RecordedFlow, write_index: usize| {
        recorded_flow.writes[write_index]
            .bytes()
            .expect("根槽写带字节")[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
            ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
            .to_vec()
    };
    let pointer_to_the_table_of_the_mount =
        pointer_of(&flow, root_slot_writes_of_the_mount_after_the_recovery[0]);
    assert!(
        root_slot_writes_of_the_mount_after_the_recovery
            .iter()
            .all(|write_index| pointer_of(&flow, *write_index) == pointer_to_the_table_of_the_mount),
        "实例 3 的根都指着同一张实例表 T"
    );
    let root_slot_writes_of_the_candidate: Vec<usize> =
        root_slot_writes_of_the_mount_after_the_recovery
            .iter()
            .copied()
            .filter(|write_index| txg_of(&flow, *write_index) == candidate_root_txg)
            .collect();
    let journal_records_of_the_candidate: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::JournalRecord
                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET)
                        == INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY
                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
                            == candidate_root_txg
                })
        })
        .collect();
    assert!(
        !journal_records_of_the_candidate.is_empty(),
        "R 那次发布写了 journal 记录"
    );
    let journal_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
            + NODE_POINTER_BYTES;
    for record in &journal_records_of_the_candidate {
        assert_eq!(
            &flow.writes[*record].bytes().expect("记录写带字节")[journal_pointer_field.clone()],
            pointer_to_the_table_of_the_mount.as_slice(),
            "R 那次发布的记录新根段里的实例表指针与 R 的根记录相同"
        );
    }
    let table_copies_of_the_mount = |recorded_flow: &RecordedFlow| -> Vec<usize> {
        let locations = POINTER_LOCATION_ENTRY_OFFSETS.map(|location| {
            device_and_slot_of_a_pointer_location(&pointer_to_the_table_of_the_mount, location)
        });
        (0..recorded_flow.writes.len())
            .filter(|write_index| {
                let write = &recorded_flow.writes[*write_index];
                write.kind == StepKind::UnitWrite
                    && locations.contains(&(write.device.0, write.offset.0 / SLOT_BYTES))
                    && write.bytes().is_some_and(|bytes| {
                        unit_class(bytes) == 3
                            && packed_record_type(bytes) == PACKED_RECORD_TYPE_INSTANCE_TABLE
                    })
            })
            .collect()
    };
    let table_copies_before_the_chain = table_copies_of_the_mount(&flow);
    assert_eq!(
        table_copies_before_the_chain.len(),
        2,
        "实例 3 写行那次写了 T 两份"
    );
    let table_of_the_mount = flow.writes[table_copies_before_the_chain[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    let mut rows = records_of_an_instance_table_page(&table_of_the_mount);
    assert_eq!(
        rows.pop(),
        Some(chain_record_of_the_last_page()),
        "T 只有一片"
    );
    let mut rows_of_each_page_of_the_candidate_chain = vec![rows.clone()];
    rows_of_each_page_of_the_candidate_chain.resize(
        InstanceTableChainPages::OneMoreThanTheGpuWalks.count(),
        Vec::new(),
    );
    let candidate_pages = instance_table_pages_from(
        &table_of_the_mount,
        &pointer_to_the_table_of_the_mount,
        0,
        &rows_of_each_page_of_the_candidate_chain,
    );
    let (slot_of_the_page_run_into, page_run_into) =
        candidate_pages[PAGE_OF_THE_CANDIDATE_CHAIN_THE_NEWEST_CHAIN_RUNS_INTO].clone();
    let pointer_to_the_first_candidate_page = pointer_naming_a_page_at(
        &pointer_to_the_table_of_the_mount,
        candidate_pages[0].0,
        crc32c(&candidate_pages[0].1),
    );
    for root_slot_write in &root_slot_writes_of_the_candidate {
        rewrite_root_record_and_reseal(&mut flow, *root_slot_write, &|bytes| {
            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
                .copy_from_slice(&pointer_to_the_first_candidate_page);
        });
    }
    for record in &journal_records_of_the_candidate {
        rewrite_journal_record_and_reseal(&mut flow, *record, &|bytes| {
            bytes[journal_pointer_field.clone()]
                .copy_from_slice(&pointer_to_the_first_candidate_page);
        });
    }
    let devices = devices_of_an_instance_table_pointer(&pointer_to_the_table_of_the_mount);
    let candidate_page_writes = writes_of_instance_table_pages(
        &flow,
        devices,
        candidate_pages,
        PagesAfterTheFirstInTheFlow::Intact,
    );
    let first_write_naming_the_candidate_chain = *root_slot_writes_of_the_candidate
        .iter()
        .chain(&journal_records_of_the_candidate)
        .min()
        .expect("至少一次写");
    insert_a_segment_of_writes_before_the_segment_of(
        &mut flow,
        first_write_naming_the_candidate_chain,
        candidate_page_writes,
    );
    let mut second_page_of_the_table = table_of_the_mount.clone();
    replace_the_records_of_an_instance_table_page(
        &mut second_page_of_the_table,
        &[chain_record_naming_the_next_page(
            &pointer_to_the_table_of_the_mount,
            slot_of_the_page_run_into,
            crc32c(&page_run_into),
        )],
    );
    second_page_of_the_table
        [PACKED_UNIT_CONTAINER_NUMBER_OFFSET..PACKED_UNIT_CONTAINER_NUMBER_OFFSET + 8]
        .copy_from_slice(&1u64.to_le_bytes());
    reseal_both_checksums_of_the_unit_bytes(&mut second_page_of_the_table);
    let mut first_page_records = rows;
    first_page_records.push(chain_record_naming_the_next_page(
        &pointer_to_the_table_of_the_mount,
        SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE,
        crc32c(&second_page_of_the_table),
    ));
    let table_copies = table_copies_of_the_mount(&flow);
    assert_eq!(table_copies.len(), 2, "插段之后 T 还是两份");
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &table_copies, &|bytes| {
        replace_the_records_of_an_instance_table_page(bytes, &first_page_records);
    });
    let second_page_writes = writes_of_instance_table_pages(
        &flow,
        devices,
        vec![(
            SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE,
            second_page_of_the_table,
        )],
        PagesAfterTheFirstInTheFlow::Intact,
    );
    let first_table_write = *table_copies.iter().min().expect("T 至少一份");
    insert_a_segment_of_writes_before_the_segment_of(
        &mut flow,
        first_table_write,
        second_page_writes,
    );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    assert!(
        matches!(
            pool_checker_verdict_on_the_image(&image_with_every_write, "I-1.1"),
            InvariantVerdict::Violated(_)
        ),
        "流 {}：最新根那条链第 2 片是 R 那条链的第 100 片、容器号不是 2，CPU 判 I-1.1 红：{:?}",
        flow.name,
        pool_checker_verdict_on_the_image(&image_with_every_write, "I-1.1")
    );
    flow
}

/// 扫描内核站点 37 / 38 那两句留作兜底，补走得到它们的那一形：
/// [`record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it`]。
/// CPU 读链不设片数上限；GPU 池级 checker 判不了的原因字只许是实例表链过长那三种（R 还是最新根的状态上走读内核走满记 61），
/// 且至少一个状态落在扫描内核那两处（37 或 38，推的：读码，攻方读码模型 `l2_chain_model.py` 也是这样，没在卡上跑过）；
/// 判了的与 CPU 逐位相同，判不了就回落 CPU 之后红项相同。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it(
) {
    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
        "the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it",
        &record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it(
            "crash-judge-gpu-candidate-instance-table-of-257-pages-the-newest-chain-runs-into",
        ),
        &[
            GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED,
            GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED,
            GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED,
        ],
        // 恢复读到 R（链 257 片）的状态上恢复内核沿链读满 256 片还有下一片，按设计交 CPU（E9 那一趟这条流 16 个状态记 4）
        GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES,
        &[
            GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED,
            GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED,
        ],
    );
}
