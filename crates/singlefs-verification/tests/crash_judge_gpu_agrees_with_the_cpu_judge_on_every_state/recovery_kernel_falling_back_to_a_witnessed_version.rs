//! GPU 恢复内核退回见证到的那一版（F2 的两形）、按全部版本走完之后的折叠表取单元、read_at 拼读。

use super::*;

// ─── GPU 恢复内核的 F2（F2_EXACT 那一形）、I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold）、read_at 拼读（H-S2-tail） ───

/// 根槽写里自证校验和字段的偏移（`reseal_root_record` 重封的那 32 字节）。
const ROOT_RECORD_CHECKSUM_FIELD_OFFSET: usize = 138;

/// 流里写出 txg `txg` 那条根的根槽写（恰好一次）。
pub(super) fn root_write_of_txg(flow: &RecordedFlow, txg: u64) -> usize {
    let root_writes: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::RootRecordFua
                && write.bytes().is_some_and(|bytes| {
                    u64::from_le_bytes(
                        bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
                            ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                            .try_into()
                            .expect("8 字节"),
                    ) == txg
                })
        })
        .map(|(index, _)| index)
        .collect();
    let [root_write] = root_writes.as_slice() else {
        panic!("流里写出 txg {txg} 那条根的根槽写恰好一次：{root_writes:?}")
    };
    *root_write
}

/// F2 那两条流的底：新池新建文件（txg 3）→ 同一个进程、实例 1 里覆盖写成第二份内容（V，txg 4）→ 再覆盖写成第三份内容（R，txg 5），整条录下来；
/// R 那次发布的树表两份副本各改坏末字节、不重封（[`break_both_copies_of_the_tree_table_named_by_the_root_write`]）：R 的根槽落了的状态上 R 走读失败。
/// 每次发布按单元、记录、根槽、系统配置轮换的次序落盘（`transaction::persist_publish_writes`），R 的根槽落了而它的系统配置轮换没落的状态上，
/// c_见证 是 V 那次发布末条记录的计数器，系统配置见证不到 R。
fn record_two_overwrites_whose_second_tree_table_does_not_read_back(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow = record_two_overwrites(tag, name);
    let root_of_the_judged_publish = flow.judged_root_index;
    break_both_copies_of_the_tree_table_named_by_the_root_write(
        &mut flow,
        root_of_the_judged_publish,
    );
    flow
}

/// F2 那两条流上的三版：V⁻（新池新建文件）、V（第一次覆盖写，已确认）、R（第二次覆盖写，走读失败），(实例代号, txg)。
const VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS: (u32, u64) = (1, 3);

pub(super) const CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS: (u32, u64) = (1, 4);

pub(super) const JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS: (u32, u64) = (1, 5);

/// 看 journal 那一遍 oracle 判「落到更旧的根」的那一项（`CrashStateJudgement::red_items` 里的写法）。
pub(super) fn consulted_oracle_item(kind: CrashPointReplayOracleViolationKind) -> String {
    format!(
        "{CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE}:{}",
        kind.name()
    )
}

/// 条件：看 journal 那一遍实际走的那条根是 `version`（实例代号与完整的 checkpoint_txg）、结局码是 `class`。
pub(super) fn consulted_recovery_lands_on(version: (u32, u64), class: u32) -> StateCondition {
    StateCondition::CpuConsultedPassLandsOn { version, class }
}

/// 数「看 journal 那一遍报读不出、报的根是 R、实际走的那条根也是 R、结局读失败」的状态（F2 那几条流上 R 走读失败、不退的那几个状态）。
fn states_reporting_the_judged_publish_unreadable() -> StateExamination {
    StateExamination::CountStatesWhere(StateCondition::AllOf(vec![
        StateCondition::CpuRedItemsContain(consulted_oracle_item(
            CrashPointReplayOracleViolationKind::ReadFailed,
        )),
        StateCondition::CpuReportedRootIs(Some(JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS)),
        consulted_recovery_lands_on(
            JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS,
            RECOVERY_WORD_CLASS_FAILED,
        ),
    ]))
}

/// 恢复摘要里看 journal 那一遍实际走的那条根，换成 (实例代号, 完整的 checkpoint_txg)；没落到根是 `None`。
pub(super) fn consulted_landed_root_of(
    summary: RecoveryRootAndOutcomesOfState,
) -> Option<(u32, u64)> {
    summary
        .consulted_pass_landed_root
        .landed_root()
        .map(|(instance, checkpoint_txg)| (instance.0, checkpoint_txg.0))
}

/// F2 设计里那一退（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行「设计那一退照旧落 V」）：R 的根槽落了、走读失败，
/// 系统配置没见证 R、见证到 V，CPU 恢复退到 V 本身、读出第二份内容，oracle 判「落到更旧的根」（oracle 不认 F2 那一退）。
/// GPU 恢复内核照 `recovery::newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to` 退（`crash_judge_recovery.wgsl` 的同名函数），每个状态逐位相同、
/// 落到同一条根。没有它的 GPU 在这几个状态上报读不出。F2 取 F2_NEWEST 那一形
/// 时这一退不变：[V, R) 里最新、不被 R 的实例表抛弃的根就是 V。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_newest_root_does_not_read_back_falls_back_to_the_version_the_system_configuration_witnessed";
    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(
        "crash-judge-gpu-f2-fall-back-to-the-witnessed-version",
        "two-overwrites-second-tree-table-unreadable-falls-back-to-the-witnessed-version",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let recovered_to_an_older_root =
        consulted_oracle_item(CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot);
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[StateExamination::CountStatesWhere(StateCondition::AllOf(
            vec![
                StateCondition::CpuRedItemsContain(recovered_to_an_older_root),
                consulted_recovery_lands_on(
                    CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS,
                    RECOVERY_WORD_CLASS_FILE_READ,
                ),
            ],
        ))],
    ) else {
        return;
    };
    let states_falling_back_to_the_confirmed_version = comparison.examined[0].states_counted;
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_falling_back_to_v={states_falling_back_to_the_confirmed_version}",
        flow.name, comparison.states
    );
    assert!(
        states_falling_back_to_the_confirmed_version > 0,
        "流 {}：R 的根槽落了而系统配置没见证 R 的状态上，CPU 看 journal 那一遍退到 V {CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS:?}、读出第二份内容",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 攻方 A 形（判决 `research/prompts/m3-batch33t-code-r1-main-verification.md` 第二节 T5 行，攻方 `research/prompts/m3-batch33t-code-r1-opus-output.md`
/// 第一节 1）：同上一条流，V 的根槽写自证校验和字段翻一位（V 的根槽读不出），R 走读失败、系统配置见证到 V 的状态上，F2_NEWEST只在
/// [V, R) 里找：V 的根读不出、区间里再没有别的根，就不退——CPU 报读不出（落到的根照旧是 R），一个状态都不退到 V⁻；GPU 恢复内核同判，每个状态逐位相同。
/// 退到「比所选根旧、不超过见证上限的最新可读根」或不设下界，会落到 V⁻、把已确认的 V 越过去，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_the_witnessed_version_whose_root_does_not_read_back";
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(
        "crash-judge-gpu-f2-witnessed-root-unreadable",
        "two-overwrites-second-tree-table-unreadable-and-the-witnessed-root-unreadable",
    );
    let root_of_the_confirmed_version =
        root_write_of_txg(&flow, CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS.1);
    let WrittenContents::Bytes(root_bytes) =
        &mut flow.writes[root_of_the_confirmed_version].contents
    else {
        panic!("根槽写是普通写");
    };
    root_bytes[ROOT_RECORD_CHECKSUM_FIELD_OFFSET] ^= 0x01;
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[
            states_reporting_the_judged_publish_unreadable(),
            states_where_the_consulted_recovery_falls_back_to(
                VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS,
            ),
        ],
    ) else {
        return;
    };
    let states_reporting_the_judged_publish_unreadable = comparison.examined[0].states_counted;
    let states_falling_back_past_the_confirmed_version = comparison.examined[1].states_counted;
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_reporting_r_unreadable={states_reporting_the_judged_publish_unreadable} states_falling_back_past_v={states_falling_back_past_the_confirmed_version}",
        flow.name, comparison.states
    );
    assert!(
        states_reporting_the_judged_publish_unreadable > 0,
        "流 {}：R 走读失败、V 的根读不出的状态上 CPU 看 journal 那一遍报读不出、报的根是 R {JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS:?}",
        flow.name
    );
    assert_eq!(
        states_falling_back_past_the_confirmed_version, 0,
        "流 {}：一个状态都不退到 V⁻ {VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS:?}（F2 只退到见证到的那一版 V 及之后、所选根之前）",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

// ─── GPU 恢复内核的 F2 改成 F2_NEWEST 那一形（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节
// K1 / K2 W4 H4 行、K1 V1 行与第三节第 2 条） ───

/// V 那次发布轮换写的系统配置槽：V 的根槽写之后、R 的根槽写之前的系统配置槽写（每次发布按单元、记录、根槽、系统配置轮换的次序落盘），每块盘一槽。
fn system_configuration_rotation_writes_of_the_confirmed_version(
    flow: &RecordedFlow,
) -> Vec<usize> {
    let root_of_the_confirmed_version =
        root_write_of_txg(flow, CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS.1);
    let root_of_the_judged_publish =
        root_write_of_txg(flow, JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS.1);
    let rotation: Vec<usize> = (root_of_the_confirmed_version + 1..root_of_the_judged_publish)
        .filter(|index| flow.writes[*index].kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        rotation.len(),
        singlefs_core::recovery::PoolReader::device_identities(&flow.base).len(),
        "V 那次轮换每块盘写一槽：{rotation:?}"
    );
    rotation
}

/// 攻方 H4 的底（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 / K2 W4 H4 行；攻方
/// `research/prompts/m3-batch35-36-code-r1-opus-output.md` 第四节，攻方的 Pm、P、R 是这里的 V⁻、V、R）：F2 那两条流的底上，V 那次发布轮换写的系统配置槽
/// （每块盘一槽，是两槽里较新的那一槽）写成全零。R 的根槽落了、R 的轮换没落的状态上，每块盘读得出的只剩 V⁻ 那次轮换写的那一槽，
/// c_见证 落回 V⁻ 那次发布末条记录的计数器，系统配置没见证 R；V 已确认、根与单元都读得出。
fn record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(tag, name);
    for rotation_write in system_configuration_rotation_writes_of_the_confirmed_version(&flow) {
        let slot_bytes = usize::try_from(flow.writes[rotation_write].length_in_bytes())
            .expect("系统配置槽写 4096 字节装得进 usize");
        flow.writes[rotation_write].contents = WrittenContents::Bytes(vec![0u8; slot_bytes]);
    }
    flow
}

/// 数看 journal 那一遍经 F2 退到 `version` 的状态：CPU 报的根是它，oracle 判「落到更旧的根」（所选根 R 落了而走的是更旧的这一版）。
fn states_where_the_consulted_recovery_falls_back_to(version: (u32, u64)) -> StateExamination {
    StateExamination::CountStatesWhere(StateCondition::AllOf(vec![
        StateCondition::CpuRedItemsContain(consulted_oracle_item(
            CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot,
        )),
        StateCondition::CpuReportedRootIs(Some(version)),
    ]))
}

/// F2 的四条流共用的判法：CPU 看 journal 那一遍至少一个状态退到 `landed_version`（读出它的内容）、一个状态都不退到 `version_not_landed_on`，
/// GPU 每个状态逐位相同、落到同一条根。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
fn assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
    test_name: &str,
    flow: &RecordedFlow,
    landed_version: (u32, u64),
    version_not_landed_on: (u32, u64),
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let recovered_to_an_older_root =
        consulted_oracle_item(CrashPointReplayOracleViolationKind::RecoveredToAnOlderRoot);
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        flow,
        expand_up_to(),
        &[
            StateExamination::CountStatesWhere(StateCondition::AllOf(vec![
                StateCondition::CpuRedItemsContain(recovered_to_an_older_root),
                StateCondition::CpuReportedRootIs(Some(landed_version)),
                consulted_recovery_lands_on(landed_version, RECOVERY_WORD_CLASS_FILE_READ),
            ])),
            states_where_the_consulted_recovery_falls_back_to(version_not_landed_on),
        ],
    ) else {
        return;
    };
    let states_falling_back_to_the_landed_version = comparison.examined[0].states_counted;
    let states_falling_back_to_the_other_version = comparison.examined[1].states_counted;
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_falling_back_to_{landed_version:?}={states_falling_back_to_the_landed_version} states_falling_back_to_{version_not_landed_on:?}={states_falling_back_to_the_other_version}",
        flow.name, comparison.states
    );
    assert!(
        states_falling_back_to_the_landed_version > 0,
        "流 {}：R 走读失败、系统配置没见证 R 的状态上，CPU 看 journal 那一遍退到 {landed_version:?}、读出它的内容",
        flow.name
    );
    assert_eq!(
        states_falling_back_to_the_other_version, 0,
        "流 {}：一个状态都不退到 {version_not_landed_on:?}",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 攻方 H4（底见 [`record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros`]）：
/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 退到 [V⁻, R) 里读得出、不被 R 的实例表抛弃的最新一条根——已确认、单元完好的 V；
/// F2_EXACT 只认 V⁻、把 V 越过去（攻方原样：三处都落 Pm）。CPU 看 journal 那一遍至少一个状态退到 V、一个状态都不退到 V⁻，GPU 逐位相同。
/// GPU 恢复内核读得出所选根的实例表也只认恰好相等的那条、或把下界换成见证上限时 GPU 落 V⁻，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_falls_back_to_the_newest_version_from_the_witnessed_one_when_the_newer_system_configuration_slots_read_back_as_zeros(
) {
    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
        "crash-judge-gpu-f2-newest-newer-rotation-zeroed",
        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed",
    );
    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_falls_back_to_the_newest_version_from_the_witnessed_one_when_the_newer_system_configuration_slots_read_back_as_zeros",
        &flow,
        CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS,
        VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS,
    );
}

/// 实例表那一片单元（码 3、记录类型 4）最前面插一行 (`instance`, `selected_root_txg`, W = 0)：原有的记录往后挪一条，记录数与声明长度各加一条
/// （单元自己的两道校验和由调用方重封）。
fn insert_an_instance_table_row_in_front(bytes: &mut [u8], instance: u32, selected_root_txg: u64) {
    let record_count = usize::from(u16::from_le_bytes(
        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    ));
    let records_end = PACKED_UNIT_RECORDS_START + record_count * INSTANCE_TABLE_RECORD_BYTES;
    bytes.copy_within(
        PACKED_UNIT_RECORDS_START..records_end,
        PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_RECORD_BYTES,
    );
    let row = &mut bytes
        [PACKED_UNIT_RECORDS_START..PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_RECORD_BYTES];
    row.fill(0);
    row[1..5].copy_from_slice(&instance.to_le_bytes());
    row[5..13].copy_from_slice(&selected_root_txg.to_le_bytes());
    let new_record_count = record_count + 1;
    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(new_record_count)
            .expect("记录数装得进 2 字节")
            .to_le_bytes(),
    );
    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
        &u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES)
            .expect("声明长度装得进 2 字节")
            .to_le_bytes(),
    );
}

/// 实例表一片的字节数（码 3 打包记录单元，占两槽）。
const INSTANCE_TABLE_PAGE_BYTES: u64 = 2 * SLOT_BYTES;

/// 只让所选根 R 认一张抛弃 V 的实例表：V⁻、V、R 指着的是同一张单片实例表，它在基镜像里（流里没有写它的单元写，同一实例里不重写）。
/// 照 R 的实例表指针第 0 条位置条目从基镜像读出它（整单元 CRC 与指针里的相同），最前面插一行 (V⁻ 的实例, V⁻ 的 txg)、重封，
/// 写进两块盘基镜像里流从没写过、全零的 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]：实例表单元头只带身份 (0, 4, 片号, 0) 与诞生、写序，不带自己的落点，
/// 换一槽照样自证得过（读者按位置条目读、只比整单元 CRC）。R 的根槽写与 R 那次发布的 journal 记录新根段里的实例表指针（写者照抄根记录里那一条）
/// 两条位置条目的槽改指这一槽、整单元校验和换成新的，各自重封（记录的反向链一路传下去）。V⁻、V 的根记录与原来那张表一个字节都不动。
fn point_the_chosen_root_alone_at_an_instance_table_abandoning_the_confirmed_version(
    flow: &mut RecordedFlow,
) {
    let root_record_pointer_field = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
        ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES;
    let instance_table_pointer_of = |recorded_flow: &RecordedFlow, root_write: usize| {
        recorded_flow.writes[root_write]
            .bytes()
            .expect("根槽写带字节")[root_record_pointer_field.clone()]
        .to_vec()
    };
    let root_of_the_judged_publish =
        root_write_of_txg(flow, JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS.1);
    let pointer_to_the_shared_table = instance_table_pointer_of(flow, root_of_the_judged_publish);
    assert_eq!(
        instance_table_pointer_of(
            flow,
            root_write_of_txg(flow, CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS.1)
        ),
        pointer_to_the_shared_table,
        "流 {}：V 与 R 的根记录指着同一张实例表",
        flow.name
    );
    let (device_of_the_first_location, slot_of_the_shared_table) =
        device_and_slot_of_a_pointer_location(
            &pointer_to_the_shared_table,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
        );
    let shared_table_start = slot_of_the_shared_table * SLOT_BYTES;
    let shared_table_end = shared_table_start + INSTANCE_TABLE_PAGE_BYTES;
    assert!(
        flow.writes.iter().all(|write| write.device.0
            != device_of_the_first_location
            || write.offset.0 + write.length_in_bytes() <= shared_table_start
            || write.offset.0 >= shared_table_end),
        "流 {}：R 指着的实例表（盘 {device_of_the_first_location} 槽 {slot_of_the_shared_table}）流里没写过，在基镜像里",
        flow.name
    );
    let shared_table = singlefs_core::recovery::PoolReader::read(
        &flow.base,
        DeviceIdentity(device_of_the_first_location),
        singlefs_core::address::DeviceOffsetInBytes(shared_table_start),
        usize::try_from(INSTANCE_TABLE_PAGE_BYTES).expect("32768"),
    )
    .expect("基镜像里读得到 R 指着的实例表");
    assert_eq!(
        crc32c(&shared_table),
        read_u32_at(
            &pointer_to_the_shared_table,
            POINTER_LOCATION_ENTRY_OFFSETS[0] + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET
        ),
        "流 {}：基镜像里那一槽的整单元 CRC 与 R 的实例表指针里的相同",
        flow.name
    );
    assert!(
        unit_class(&shared_table) == 3
            && packed_record_type(&shared_table) == PACKED_RECORD_TYPE_INSTANCE_TABLE,
        "流 {}：R 指着的是一片实例表（码 3、记录类型 4）",
        flow.name
    );
    let mut table_of_the_judged_publish = shared_table;
    insert_an_instance_table_row_in_front(
        &mut table_of_the_judged_publish,
        VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS.0,
        VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS.1,
    );
    reseal_both_checksums_of_the_unit_bytes(&mut table_of_the_judged_publish);
    for device in devices_of_an_instance_table_pointer(&pointer_to_the_shared_table) {
        assert!(
            device == DeviceIdentity(0) || device == DeviceIdentity(1),
            "R 的实例表指针两条位置条目落在盘 0、盘 1 上（基镜像只在这两块盘上写）：盘 {}",
            device.0
        );
    }
    assert_nothing_in_the_flow_wrote_the_slot(flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    write_into_the_base_on_both_devices(
        flow,
        SLOT_NOTHING_IN_THE_FLOW_WROTE,
        &table_of_the_judged_publish,
    );
    let pointer_to_the_table_of_the_judged_publish = pointer_naming_a_page_at(
        &pointer_to_the_shared_table,
        SLOT_NOTHING_IN_THE_FLOW_WROTE,
        crc32c(&table_of_the_judged_publish),
    );
    rewrite_root_record_and_reseal(flow, root_of_the_judged_publish, &|bytes| {
        bytes[root_record_pointer_field.clone()]
            .copy_from_slice(&pointer_to_the_table_of_the_judged_publish);
    });
    let journal_record_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
            + NODE_POINTER_BYTES;
    let journal_records_of_the_judged_publish: Vec<usize> = (0..flow.writes.len())
        .filter(|write_index| {
            flow.writes[*write_index].kind == StepKind::JournalRecord
                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET)
                        == JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS.0
                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
                            == JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS.1
                })
        })
        .collect();
    assert!(
        !journal_records_of_the_judged_publish.is_empty(),
        "流 {}：R 那次发布写了 journal 记录",
        flow.name
    );
    for record in journal_records_of_the_judged_publish {
        assert_eq!(
            &flow.writes[record].bytes().expect("记录写带字节")
                [journal_record_pointer_field.clone()],
            pointer_to_the_shared_table.as_slice(),
            "流 {}：R 那次发布的记录新根段里的实例表指针与 R 的根记录原来那一条相同",
            flow.name
        );
        rewrite_journal_record_and_reseal(flow, record, &|bytes| {
            bytes[journal_record_pointer_field.clone()]
                .copy_from_slice(&pointer_to_the_table_of_the_judged_publish);
        });
    }
}

/// 所选根的实例表抛弃了区间里最新的那条根：H4 的底上，只让 R 认一张最前面多一行 (1, 3) 的实例表
/// （[`point_the_chosen_root_alone_at_an_instance_table_abandoning_the_confirmed_version`]：改好的那一片写进基镜像里流从没写过的槽，R 的根记录与记录新根段改指它）——
/// 按 R 的实例表，实例 1 里 txg > 3 的根（V 与 R）在被抛弃的时间线上；V⁻、V 认的仍是原来那张表，V 自己照样有效。
/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 在 [V⁻, R) 里按 R 的实例表滤掉 V、落 V⁻。
/// CPU 看 journal 那一遍至少一个状态退到 V⁻、一个状态都不退到 V，GPU 逐位相同。GPU 恢复内核读得出所选根的实例表而不按它滤时落 V，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons(
) {
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
        "crash-judge-gpu-f2-newest-abandoned-by-the-chosen-instance-table",
        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-instance-table-abandons-v",
    );
    point_the_chosen_root_alone_at_an_instance_table_abandoning_the_confirmed_version(&mut flow);
    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons",
        &flow,
        VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS,
        CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS,
    );
}

/// 所选根的实例表读不出：H4 的底上，R 的根槽写里实例表指针两条位置条目的整单元校验和各翻一位、重封根记录（V⁻、V 的根记录不动，它们指着的表照样读得出）。
/// R 走读失败、c_见证 落回 V⁻ 的状态上，F2_NEWEST 只认与见证到的那一版恰好相等的那条根、落 V⁻。CPU 看 journal 那一遍至少一个状态退到 V⁻、
/// 一个状态都不退到 V，GPU 逐位相同。GPU 恢复内核读不出所选根的实例表也照取最新时落 V，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_chosen_root_instance_table_does_not_read_back_falls_back_only_to_the_witnessed_version(
) {
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
        "crash-judge-gpu-f2-newest-chosen-instance-table-unreadable",
        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-chosen-instance-table-unreadable",
    );
    let root_of_the_judged_publish =
        root_write_of_txg(&flow, JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS.1);
    rewrite_root_record_and_reseal(&mut flow, root_of_the_judged_publish, &|bytes| {
        for checksum_offset in POINTER_LOCATION_CHECKSUM_OFFSETS {
            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + checksum_offset] ^= 0x01;
        }
    });
    assert_the_gpu_judge_agrees_with_the_cpu_falling_back_to(
        "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_whose_chosen_root_instance_table_does_not_read_back_falls_back_only_to_the_witnessed_version",
        &flow,
        VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS,
        CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS,
    );
}

// ─── GPU 恢复内核的 F2 不越过一次末条读得出、根读不出的发布（攻方 `research/prompts/m3-batch37-43-code-r1-opus-output.md`
// 第二节 Y4） ───

/// 攻方 Y4 `Ru+cfg+Proot`（攻方的 Pm、P、R 是这里的 V⁻、V、R）：H4 的底上（[`record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros`]：
/// R 的树表两份改坏、V 那次轮换写的系统配置槽写成全零），再把 V 的根槽写自证校验和字段翻一位（V 的根槽读不出）。R 走读失败、c_见证 落回 V⁻ 的状态上，
/// [V⁻, R) 里读得出、不被 R 的实例表抛弃的只有 V⁻；V⁻ 与 R 之间的 V 带末条标志的记录读得出、根读不出、不被 R 的实例表抛弃，退到 V⁻ 就越过已确认的 V——
/// CPU 看 journal 那一遍照报 R 读不出、一个状态都不退到 V⁻；GPU 恢复内核同判，每个状态逐位相同。GPU 不看这样一次发布时落 V⁻、CPU 报读不出，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_a_version_whose_last_record_reads_while_its_root_does_not(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_past_a_version_whose_last_record_reads_while_its_root_does_not";
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_and_whose_first_rotation_reads_back_as_zeros(
        "crash-judge-gpu-f2-no-skip-of-a-version-whose-root-does-not-read-back",
        "two-overwrites-second-tree-table-unreadable-first-rotation-zeroed-first-root-unreadable",
    );
    let root_of_the_confirmed_version =
        root_write_of_txg(&flow, CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS.1);
    let WrittenContents::Bytes(root_bytes) =
        &mut flow.writes[root_of_the_confirmed_version].contents
    else {
        panic!("根槽写是普通写");
    };
    root_bytes[ROOT_RECORD_CHECKSUM_FIELD_OFFSET] ^= 0x01;
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[
            states_reporting_the_judged_publish_unreadable(),
            states_where_the_consulted_recovery_falls_back_to(
                VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS,
            ),
        ],
    ) else {
        return;
    };
    let states_reporting_the_judged_publish_unreadable = comparison.examined[0].states_counted;
    let states_falling_back_to_the_version_before_the_confirmed_one =
        comparison.examined[1].states_counted;
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_reporting_r_unreadable={states_reporting_the_judged_publish_unreadable} states_falling_back_to_v_minus={states_falling_back_to_the_version_before_the_confirmed_one}",
        flow.name, comparison.states
    );
    assert!(
        states_reporting_the_judged_publish_unreadable > 0,
        "流 {}：R 走读失败、c_见证 落回 V⁻、V 的根读不出的状态上，CPU 看 journal 那一遍报读不出、报的根是 R {JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS:?}",
        flow.name
    );
    assert_eq!(
        states_falling_back_to_the_version_before_the_confirmed_one, 0,
        "流 {}：一个状态都不退到 V⁻ {VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS:?}（V 末条读得出、根读不出，退过去就越过它）",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 系统配置槽里 journal tail 的偏移（`system_configuration.rs` 的 TAIL_OFFSET，8 字节）。
const SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET: usize = 469;

/// journal 记录的字节数（一个记录槽）。
const JOURNAL_RECORD_SLOT_BYTES: u64 = 4096;

/// journal 环的起点（`JOURNAL_RING_START_SLOT` × 16 KiB = 16 MiB）。
const JOURNAL_RING_START_BYTES: u64 = 1024 * 16384;

/// 伪造的那条记录放在同一块盘上流里最后一个记录槽之后第几个槽：离开流里写过的槽，仍在环里。
const RECORD_SLOTS_PAST_THE_LAST_RECORD_OF_THE_FLOW: u64 = 8;

/// c_见证 = 0 而环里有一条计数器 0 的记录（判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第二节 K1 V1 行「建模差两处」之二，
/// 正推 `research/prompts/m3-batch35-36-code-r1-sonnet-output.md` V1 那一格）：
/// F2 设计那一退的底上，流里每一次系统配置槽写的 journal tail 改成 0、重封（c_见证 恒为 0）；流尾另起一段，写一条 V 那次发布末条记录的副本，
/// 计数器改成 0、重封，放在同一块盘上流里最后一个记录槽之后（写者写不出计数器 0 的记录，这是伪造的）。R 走读失败的状态上，那一段落没落，
/// CPU 与 GPU 都按 D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」不退、照报 R 读不出。
fn record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_the_confirmed_version(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut flow = record_two_overwrites_whose_second_tree_table_does_not_read_back(tag, name);
    let system_configuration_writes = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .count();
    for nth_from_the_end in 0..system_configuration_writes {
        rewrite_and_reseal(
            &mut flow,
            StepKind::SystemConfigurationSlot,
            nth_from_the_end,
            4096,
            SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
            &|bytes| {
                bytes[SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET
                    ..SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET + 8]
                    .fill(0);
            },
        );
    }
    let is_the_last_record_of_the_confirmed_version = |write: &RetainedWrite| {
        write.kind == StepKind::JournalRecord
            && write.bytes().is_some_and(|bytes| {
                u32::from_le_bytes(bytes[16..20].try_into().expect("4 字节"))
                    == CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS.0
                    && u64::from_le_bytes(bytes[26..34].try_into().expect("8 字节"))
                        == CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS.1
                    && bytes[7] & 1 == 1
            })
    };
    let last_record_of_the_confirmed_version = flow
        .writes
        .iter()
        .rposition(is_the_last_record_of_the_confirmed_version)
        .expect("流里有 V 那次发布带末条标志的记录");
    let device = flow.writes[last_record_of_the_confirmed_version].device;
    let mut forged_record = flow.writes[last_record_of_the_confirmed_version]
        .bytes()
        .expect("记录写带字节")
        .to_vec();
    forged_record[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6].fill(0);
    reseal_journal_record(&mut forged_record);
    let last_record_slot_of_the_flow = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::JournalRecord && write.device == device)
        .map(|write| write.offset.0)
        .max()
        .expect("这块盘上有记录写");
    let forged_record_offset = last_record_slot_of_the_flow
        + RECORD_SLOTS_PAST_THE_LAST_RECORD_OF_THE_FLOW * JOURNAL_RECORD_SLOT_BYTES;
    assert!(
        forged_record_offset + JOURNAL_RECORD_SLOT_BYTES
            <= JOURNAL_RING_START_BYTES + geometry().journal_ring_bytes,
        "伪造的记录在环里"
    );
    append_a_segment_of_writes(
        &mut flow,
        vec![RetainedWrite {
            device,
            kind: StepKind::JournalRecord,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(forged_record_offset),
            contents: WrittenContents::Bytes(forged_record),
        }],
    );
    flow
}

/// c_见证 = 0 而环里有一条计数器 0 的记录（底见 [`record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_the_confirmed_version`]）：
/// CPU 与 GPU 恢复内核都对 c_见证 = 0 设守卫、不退（D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」）。CPU 看 journal 那一遍至少一个状态
/// 照报 R 读不出、一个状态都不退到 V 或 V⁻，GPU 逐位相同。GPU 去掉守卫（找计数器 = 0 的那条记录、退到 V）时与 CPU 不一致，这条红；
/// CPU 去掉守卫时伪造那一段落了的状态退到 V，这条也红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero";
    let flow = record_two_overwrites_whose_second_tree_table_does_not_read_back_with_a_witnessed_counter_of_zero_naming_the_confirmed_version(
        "crash-judge-gpu-f2-witnessed-counter-zero",
        "two-overwrites-second-tree-table-unreadable-every-tail-zero-forged-record-of-counter-zero",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[
            states_reporting_the_judged_publish_unreadable(),
            states_where_the_consulted_recovery_falls_back_to(
                CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS,
            ),
            states_where_the_consulted_recovery_falls_back_to(
                VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS,
            ),
        ],
    ) else {
        return;
    };
    let states_reporting_the_judged_publish_unreadable = comparison.examined[0].states_counted;
    let states_falling_back_to_the_confirmed_version = comparison.examined[1].states_counted;
    let states_falling_back_to_the_version_before_the_confirmed_one =
        comparison.examined[2].states_counted;
    eprintln!(
        "CRASH_JUDGE_GPU_F2 flow={} states={} states_reporting_r_unreadable={states_reporting_the_judged_publish_unreadable} states_falling_back_to_v={states_falling_back_to_the_confirmed_version} states_falling_back_to_v_minus={states_falling_back_to_the_version_before_the_confirmed_one}",
        flow.name, comparison.states
    );
    assert!(
        states_reporting_the_judged_publish_unreadable > 0,
        "流 {}：R 走读失败、c_见证 = 0 的状态上 CPU 看 journal 那一遍报读不出、报的根是 R {JUDGED_PUBLISH_OF_THE_FALLBACK_FLOWS:?}",
        flow.name
    );
    assert_eq!(
        (states_falling_back_to_the_confirmed_version, states_falling_back_to_the_version_before_the_confirmed_one),
        (0, 0),
        "流 {}：c_见证 = 0 不退，一个状态都不退到 V {CONFIRMED_VERSION_OF_THE_FALLBACK_FLOWS:?} 或 V⁻ {VERSION_BEFORE_THE_CONFIRMED_ONE_OF_THE_FALLBACK_FLOWS:?}",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// extent 树的种类码（`singlefs_core::records::TREE_KIND_EXTENT`）。
const TREE_KIND_OF_THE_EXTENT_TREE: u16 = 1;

/// H-S1-fold 的 A 臂（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S1 行与第三节第 1 条；攻方
/// `research/prompts/m3-batch33s-code-r1-opus-output.md` 第一节 1.2 `A-newer-version-pointer-garbled`，录成流）：删除中间态的流上
/// 意图那一版（txg 5，extent 树照抄 txg 4 的）树表里 extent 树根指针第 0 条位置条目的整单元校验和异或几位（单元本身不动），重封那张树表、沿引用链补到根槽：
/// 意图那一版读 extent 树根时第 0 条那一份对不上、读回第 1 条那一份（I-3.5 的折叠表把那个落点改记成它自己）。
fn record_deletion_whose_intent_version_reads_the_shared_extent_root_from_its_second_copy(
    tag: &str,
) -> RecordedFlow {
    const TXG_OF_THE_INTENT: u64 = 5;
    let name = "deletion-intent-version-reads-the-shared-extent-root-from-its-second-copy";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let root_of_the_intent = root_write_of_txg(&flow, TXG_OF_THE_INTENT);
    let tree_table_copies =
        copies_of_the_tree_table_named_by_the_root_write(&flow, root_of_the_intent);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &tree_table_copies, &|bytes| {
        let checksum = only_tree_table_entry_of_kind(bytes, TREE_KIND_OF_THE_EXTENT_TREE)
            + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
            + POINTER_LOCATION_CHECKSUM_OFFSETS[0];
        for byte in &mut bytes[checksum..checksum + 4] {
            *byte ^= 0x5a;
        }
    });
    flow
}

/// H-S1-fold A 臂：CPU 池级 checker 判 I-3.5 时按全部版本走完之后的折叠表取每一版的单元（攻方 F1），txg 4 那一版先走、经第 0 条读回的
/// 那个落点在意图那一版读回第 1 条之后同算一个单元，意图那一版不算新引用它，I-3.5 一个状态都不红；第 0 条那一份照判 I-2.1 红。GPU 走读内核
/// 本来就按最终的落点表认，每个状态逐位相同（CPU 每版走完当场折时多红 I-3.5，随根环次序翻面）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_intent_version_reading_the_shared_extent_root_from_its_second_copy_does_not_redden_the_appearance_rule(
) {
    let flow =
        record_deletion_whose_intent_version_reads_the_shared_extent_root_from_its_second_copy(
            "crash-judge-gpu-i35-intent-reads-the-shared-extent-root-from-its-second-copy",
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_an_intent_version_reading_the_shared_extent_root_from_its_second_copy_does_not_redden_the_appearance_rule",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-2.1")
            && !comparison.cpu_checker_red_invariants.contains("I-3.5"),
        "流 {} 上 CPU 红 I-2.1、一个状态都不红 I-3.5（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 三个数据单元那条流（txg 4）下段叶第一条记录的数据指针：第 0 条位置条目的槽写成 `slot`、整单元校验和写成 `checksum`（盘不动），
/// 重封那片叶、沿引用链补到根。
fn redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
    flow: &mut RecordedFlow,
    name: &str,
    slot: u64,
    checksum: u32,
) {
    let extent_lower_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
    };
    let leaf_copies = last_unit_copies(flow, name, &extent_lower_leaf, false);
    corrupt_units_and_reseal_up_to_the_root(flow, &leaf_copies, &|bytes| {
        let location = index_node_entries_start(EXTENT_KEY_BYTES)
            + EXTENT_KEY_BYTES
            + POINTER_LOCATION_ENTRY_OFFSETS[0];
        bytes[location + LOCATION_ENTRY_SLOT_OFFSET
            ..location + LOCATION_ENTRY_SLOT_OFFSET + LOCATION_ENTRY_SLOT_BYTES]
            .copy_from_slice(&slot.to_le_bytes()[..LOCATION_ENTRY_SLOT_BYTES]);
        bytes[location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET
            ..location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&checksum.to_le_bytes());
    });
}

/// 两块盘的基镜像在槽 `slot` 写进 `bytes`（流里谁都没写过它，每个状态都在）。
fn write_into_the_base_on_both_devices(flow: &mut RecordedFlow, slot: u64, bytes: &[u8]) {
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        flow.base.devices.get_mut(&device).expect("盘在池里").write(
            singlefs_core::address::DeviceOffsetInBytes(slot * SLOT_BYTES),
            bytes,
        );
    }
}

/// 拼读那三形下一槽（[`SLOT_NOTHING_IN_THE_FLOW_WROTE`] + 1）在基镜像里写的 16 KiB。
fn bytes_of_the_slot_after_the_gap() -> Vec<u8> {
    vec![0x5au8; usize::try_from(SLOT_BYTES).expect("16384")]
}

/// 「16 KiB 全零接下一槽那 16 KiB」那 32 KiB 的 CRC-32C：空段最后一槽按 32 KiB 读，CPU 的稀疏读者读出的就是它。
fn checksum_of_zeros_followed_by_the_slot_after_the_gap() -> u32 {
    let mut thirty_two_kibibytes = vec![0u8; usize::try_from(SLOT_BYTES).expect("16384")];
    thirty_two_kibibytes.extend_from_slice(&bytes_of_the_slot_after_the_gap());
    crc32c(&thirty_two_kibibytes)
}

/// H-S2-tail（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S2 行、第三节第 1 条；攻方 `research/prompts/m3-batch33s-code-r1-opus-output.md`
/// 第二节 2.2 `S2-crc-of-zeros-then-next-slot`，录成流）：三个数据单元的流上，基镜像在 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] + 1 写进 16 KiB 非零字节，
/// txg 4 那一版下段叶第一条数据指针第 0 条位置条目改指空段右端那一槽 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、校验和写成「16 KiB 全零接下一槽」的 CRC。
fn record_three_unit_file_whose_first_data_pointer_names_the_last_slot_of_a_gap(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-first-data-pointer-naming-the-last-slot-of-a-gap";
    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    write_into_the_base_on_both_devices(
        &mut flow,
        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
        &bytes_of_the_slot_after_the_gap(),
    );
    redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
        &mut flow,
        name,
        SLOT_NOTHING_IN_THE_FLOW_WROTE,
        checksum_of_zeros_followed_by_the_slot_after_the_gap(),
    );
    flow
}

/// GPU 内核 read_at 拼读那一读的整段 CRC 对得上位置条目时记的判不了原因（`crash_judge_common.wgsl` 的 `UNDECIDABLE_SPLICED_READ_MATCHES`）。
const GPU_UNDECIDABLE_SPLICED_READ_MATCHES: u32 = 7;

/// H-S2-tail：CPU 的稀疏读者从空段右端那一槽读 32 KiB，读出「16 KiB 全零 + 下一槽」、CRC 对得上，第 0 条那一份当数据单元读、往下判它的头。
/// GPU 读者 `read_at` 拼出整段 CRC（全零 16 KiB 那一条的版本表第 8 字异或下一个落点那一版前 16 KiB 的 CRC），对得上之后往下要读的字节跨过落点末尾、
/// 表里没有，池级 checker 与恢复内核都记判不了（[`GPU_UNDECIDABLE_SPLICED_READ_MATCHES`]），照流水线那样回落 CPU 之后每个状态的红项相同；
/// 判了的状态逐位相同。拼读之前 GPU 守卫拒这一读、判第 0 条读不出，多红 I-2.1。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
) {
    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
        "the_gpu_judge_leaves_to_the_cpu_a_data_pointer_whose_first_location_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot",
        &record_three_unit_file_whose_first_data_pointer_names_the_last_slot_of_a_gap(
            "crash-judge-gpu-s2-tail-data-pointer-last-slot-of-a-gap",
        ),
        &[GPU_UNDECIDABLE_SPLICED_READ_MATCHES],
        // 恢复内核读第一个文件时同一处拼读记同一个原因（两个内核共用 `crash_judge_common.wgsl` 的 read_at 与这个常量）
        &[GPU_UNDECIDABLE_SPLICED_READ_MATCHES],
        &[GPU_UNDECIDABLE_SPLICED_READ_MATCHES],
    );
}

/// 正推「16 KiB 节点加下一槽」那一形（`research/prompts/m3-batch33s-code-r1-sonnet-output.md` K1-S2 末段，录成流）：三个数据单元的流上，基镜像在
/// [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] 写进 txg 4 那片下段叶（码 2、16 KiB）的原样字节、下一槽写进 16 KiB 非零字节，下段叶第一条数据指针第 0 条位置条目
/// 改指那一槽、校验和写成「节点 16 KiB 接下一槽」那 32 KiB 的 CRC。
fn record_three_unit_file_whose_first_data_pointer_names_a_node_followed_by_a_written_slot(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-first-data-pointer-naming-a-node-followed-by-a-written-slot";
    let mut flow = record_new_pool_through_a_three_unit_file(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    let extent_lower_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == EXTENT_KEY_BYTES
            && index_node_entry_width(bytes) == EXTENT_LOWER_LEAF_RECORD_BYTES
    };
    let node = flow.writes[last_unit_copies(&flow, name, &extent_lower_leaf, false)[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(
        u64::try_from(node.len()).expect("单元长装得进 u64"),
        SLOT_BYTES,
        "码 2 节点 16 KiB"
    );
    write_into_the_base_on_both_devices(&mut flow, SLOT_NOTHING_IN_THE_FLOW_WROTE, &node);
    write_into_the_base_on_both_devices(
        &mut flow,
        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
        &bytes_of_the_slot_after_the_gap(),
    );
    let mut node_followed_by_the_next_slot = node;
    node_followed_by_the_next_slot.extend_from_slice(&bytes_of_the_slot_after_the_gap());
    redirect_the_first_location_of_the_first_lower_leaf_data_pointer(
        &mut flow,
        name,
        SLOT_NOTHING_IN_THE_FLOW_WROTE,
        crc32c(&node_followed_by_the_next_slot),
    );
    flow
}

/// 「16 KiB 节点加下一槽」：CPU 从那一槽读 32 KiB，CRC 对得上，第 0 条那一份当数据单元读（头是码 2，红）。GPU 不用拼：单元槽落点本来就是 32 KiB 长
/// （`crash_judge_tables` 的落点表，盘尾那一槽才 16 KiB），版本字节的后 16 KiB 就是基镜像里的下一槽，整段 CRC 照表比、往下照读，每个状态逐位相同，
/// 一个状态都不记判不了。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_a_node_with_the_checksum_of_the_node_and_the_next_slot(
) {
    let flow =
        record_three_unit_file_whose_first_data_pointer_names_a_node_followed_by_a_written_slot(
            "crash-judge-gpu-s2-node-followed-by-a-written-slot",
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_a_node_with_the_checksum_of_the_node_and_the_next_slot",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red > 0,
        "流 {}：第 0 条那一份当数据单元读、头是码 2，CPU 池级 checker 要有红",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 正推「空槽加下一已写槽前 16 KiB」那一形（同上 K1-S2 末段；`zero16+next`，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 7、
/// 子指针第 1 条位置条目指 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成「16 KiB 全零接下一槽」的 CRC，下一槽在两块盘的基镜像里写进 16 KiB 非零字节。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
    tag: &str,
) -> RecordedFlow {
    let name =
        "deletion-tombstone-root-entry-type-seven-second-location-naming-the-last-slot-of-a-gap";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    write_into_the_base_on_both_devices(
        &mut flow,
        SLOT_NOTHING_IN_THE_FLOW_WROTE + 1,
        &bytes_of_the_slot_after_the_gap(),
    );
    let checksum = checksum_of_zeros_followed_by_the_slot_after_the_gap();
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            SLOT_NOTHING_IN_THE_FLOW_WROTE,
            checksum,
        );
    });
    flow
}

/// 「空槽加下一已写槽前 16 KiB」：CPU 头里类标签 0 不登记，按 32 KiB 读出「16 KiB 零 + 下一槽」、对得上，那一份 I-2.1 成立，红 I-9.23。
/// GPU F-α 那一处比 32 KiB 时拿 `read_at` 拼出的整段 CRC 比（只比 CRC、不往下读，不记判不了），每个状态逐位相同（拼读之前那一读读不到、
/// 多红 I-2.1、I-4.8、I-7.4）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
) {
    let flow = record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot(
        "crash-judge-gpu-s2-tombstone-entry-type-seven-last-slot-of-a-gap",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_of_a_gap_with_the_checksum_of_zeros_and_the_next_slot",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.23")
            && !comparison.cpu_checker_red_invariants.contains("I-2.1"),
        "流 {} 上 CPU 红 I-9.23、一个状态都不红 I-2.1（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}
