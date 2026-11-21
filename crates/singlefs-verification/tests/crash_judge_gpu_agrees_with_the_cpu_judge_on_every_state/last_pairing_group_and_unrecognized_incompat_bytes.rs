//! 改坏一处的流只判最后一组对拍块（变异行点名的小用例），系统配置 incompat 第一个字节不认得、格式版本先于 incompat 位判。

use super::*;

// ─── 改坏一处的流只判最后一组对拍块：变异行点名的小用例 ───

/// 只判最后一组对拍块的用例一条至多判几个状态（`.claude/rules/verification.md`「一个门禁：两种跑法」：指定范围的那几趟崩溃状态数压在 20 万量级）。
const MOST_STATES_OF_A_CASE_ON_THE_LAST_PAIRING_GROUP: u64 = 200_000;

/// 一条 `state_count` 个状态的流最后那一组对拍块的状态序号区间：起点是序号最大那个状态所在那一组的组界（一组
/// [`STATES_PER_PAIRING_BLOCK`] × [`PAIRING_BLOCKS_JUDGED_TOGETHER`] 个状态），终点是整条流的状态数。含序号最大的那个「全部持久」的状态
/// （流里每次写都落了），区间两头落在组的边界上或整条流的状态数上，块键与整条流跑时逐块相同。
///
/// # Panics
/// `state_count` 是 0。
fn ordinals_of_the_last_pairing_group(state_count: u64) -> std::ops::Range<u64> {
    let states_per_group = STATES_PER_PAIRING_BLOCK
        * u64::try_from(PAIRING_BLOCKS_JUDGED_TOGETHER).expect("装得进 u64");
    let last_ordinal = state_count
        .checked_sub(1)
        .expect("一条流至少一个状态（全部持久那一个）");
    last_ordinal / states_per_group * states_per_group..state_count
}

/// 场景：几种整条流的状态数（不满一组、恰好一组、不整除的十几组、恰好整除的两组）各取最后一组对拍块的序号区间（不上卡、不录流，只算区间）。
/// 预期：区间不空、终点是整条流的状态数、起点在组的边界上、不满一组，`crash_library::groups_of_pairing_blocks_holding_the_states` 收它、
/// 交回的恰好是最后一组；恰好整除时是整一组。
/// 判别力（`crates/mutations.tsv`）：起点照整条流的状态数（不是序号最大的那个状态）取到组界，恰好整除时区间是空的，这条红。
#[test]
fn the_last_pairing_group_starts_on_a_group_boundary_and_ends_at_the_state_count_of_the_flow() {
    let states_per_block = std::num::NonZeroU64::new(STATES_PER_PAIRING_BLOCK).expect("块宽不是 0");
    let blocks_per_group =
        std::num::NonZeroUsize::new(PAIRING_BLOCKS_JUDGED_TOGETHER).expect("一次判的块数不是 0");
    let states_per_group = STATES_PER_PAIRING_BLOCK
        * u64::try_from(PAIRING_BLOCKS_JUDGED_TOGETHER).expect("装得进 u64");
    for (state_count, expected) in [
        (1, 0..1),
        (states_per_group, 0..states_per_group),
        (
            32 * states_per_group + 52,
            32 * states_per_group..32 * states_per_group + 52,
        ),
        (2 * states_per_group, states_per_group..2 * states_per_group),
    ] {
        let ordinals = ordinals_of_the_last_pairing_group(state_count);
        assert_eq!(
            ordinals, expected,
            "{state_count} 个状态的流最后一组对拍块的序号区间"
        );
        let groups = groups_of_pairing_blocks_holding_the_states(
            &ordinals,
            state_count,
            states_per_block,
            blocks_per_group,
        )
        .unwrap_or_else(|refusal| panic!("{state_count} 个状态：{refusal}"));
        assert_eq!(
            groups,
            state_count.div_ceil(states_per_group) - 1..state_count.div_ceil(states_per_group),
            "{state_count} 个状态：区间 {ordinals:?} 恰好是最后一组"
        );
    }
}

/// 改坏一处、CPU 池级 checker 红 `invariant` 的流只判最后一组对拍块（[`ordinals_of_the_last_pairing_group`]，含全部持久那个状态：
/// 改坏的那一处在最后一次发布里，全部持久的状态上 CPU 红它）：CPU 在这一段红 `invariant`，每个状态 GPU 与 CPU 的池级 checker、
/// 两遍恢复与整份对拍字逐位相同（[`assert_the_pool_checker_half_then_the_recovery_half_agree`]）。这一段不超过
/// [`MOST_STATES_OF_A_CASE_ON_THE_LAST_PAIRING_GROUP`] 个状态，块键与整条流那一条用例逐块相同（那一条判过的块照样复用）。
/// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_of_a_flow_reddening(
    test_name: &str,
    flow: &RecordedFlow,
    invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let state_count = crash_point_replay_plan_state_count(
        &flow.base,
        &flow.writes,
        &flow.segments,
        &expansion_up_to(expand_up_to()),
    );
    let ordinals = ordinals_of_the_last_pairing_group(state_count);
    assert!(
        ordinals.end - ordinals.start <= MOST_STATES_OF_A_CASE_ON_THE_LAST_PAIRING_GROUP,
        "流 {}：{state_count} 个状态，最后一组对拍块 {ordinals:?} 多于 {MOST_STATES_OF_A_CASE_ON_THE_LAST_PAIRING_GROUP} 个状态，这条小用例判不了",
        flow.name
    );
    let Some(comparison) =
        compare_on_every_state_with_ordinals_in(cards, flow, expand_up_to(), ordinals)
    else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains(invariant),
        "流 {} 最后一组对拍块上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert_the_pool_checker_half_then_the_recovery_half_agree(flow, &comparison);
}

/// 场景：I-9.26 那条流（inode 记录的改动计数越过所在容器的诞生代号）只判最后一组对拍块。
/// 预期：CPU 红 I-9.26，GPU 每个状态逐位相同。整条流的每个状态由
/// [`the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth`] 判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_when_an_inode_change_count_runs_past_its_container_birth(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_of_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_when_an_inode_change_count_runs_past_its_container_birth",
        &record_new_pool_with_an_inode_change_count_past_its_container_birth("crash-judge-gpu-i926"),
        "I-9.26",
    );
}

/// 场景：I-9.4 第四句那条流（inode 叶里两条记录对调）只判最后一组对拍块。
/// 预期：CPU 红 I-9.4，GPU 每个状态逐位相同。整条流的每个状态由
/// [`the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order`] 判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_when_the_records_of_an_inode_leaf_are_out_of_order(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_of_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_when_the_records_of_an_inode_leaf_are_out_of_order",
        &record_new_pool_with_the_two_records_of_the_inode_leaf_swapped(
            "crash-judge-gpu-i94-records-swapped",
        ),
        "I-9.4",
    );
}

/// 场景：I-9.7 截断字段之前那一段预留的最后一个字节写 1 那条流只判最后一组对拍块。
/// 预期：CPU 红 I-9.7，GPU 每个状态逐位相同。整条流的每个状态由
/// [`the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_before_the_truncation_field_are_not_zero`] 判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_when_the_reserved_bytes_before_the_truncation_field_are_not_zero(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_of_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_when_the_reserved_bytes_before_the_truncation_field_are_not_zero",
        &record_the_three_unit_file_with_its_record_changed(
            "crash-judge-gpu-reserved-byte-before-the-truncation-field",
            "reserved-byte-before-the-truncation-field",
            &|record| record[INODE_RECORD_LAST_RESERVED_BYTE_BEFORE_THE_TRUNCATION_FIELD] = 1,
        ),
        "I-9.7",
    );
}

/// 场景：I-9.7 截断字段之后那一段预留的最后一个字节写 1 那条流只判最后一组对拍块。
/// 预期：CPU 红 I-9.7，GPU 每个状态逐位相同。整条流的每个状态由
/// [`the_gpu_judge_agrees_with_the_cpu_when_the_reserved_bytes_after_the_truncation_field_are_not_zero`] 判。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_when_the_reserved_bytes_after_the_truncation_field_are_not_zero(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_of_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_on_the_last_pairing_group_when_the_reserved_bytes_after_the_truncation_field_are_not_zero",
        &record_the_three_unit_file_with_its_record_changed(
            "crash-judge-gpu-reserved-byte-after-the-truncation-field",
            "reserved-byte-after-the-truncation-field",
            &|record| record[INODE_RECORD_LAST_RESERVED_BYTE_AFTER_THE_TRUNCATION_FIELD] = 1,
        ),
        "I-9.7",
    );
}

/// I-9.7 截断字段 T = 0 而 N_w = 5（没写过的字段两段都是 0）：CPU 只红 I-9.7，两遍恢复都把这条记录判不可用。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_size_comes_without_its_txg() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_size_comes_without_its_txg";
    let flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-truncation-field-size-without-txg",
        "truncation-field-size-without-txg",
        &|record| write_the_truncation_field(record, 0, 5),
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.7"],
        &[],
    );
}

/// I-9.39 的必红世界：三个单元（txg 4 写下）那一版的记录截断字段写成 (5, 0)：⌈0 ÷ 净荷⌉.max(1) = 1，单元 1、2 在截断字段之外、
/// size 之内，诞生代号 4 早于 T——CPU 只红 I-9.39（I-1.13 报不适用，GPU 不比它）；GPU 扫描内核逐位跟上。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_live_data_unit_beyond_the_truncation_field_is_born_before_it(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_live_data_unit_beyond_the_truncation_field_is_born_before_it";
    let flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-live-unit-born-before-the-truncation-field",
        "live-unit-born-before-the-truncation-field",
        &|record| write_the_truncation_field(record, CHECKPOINT_TXG_OF_THE_THREE_UNIT_FILE + 1, 0),
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &["I-9.39"],
        &[],
    );
}

/// 合法世界：三个单元那一版的记录截断字段写成 (4, 0)，单元 1、2 在截断字段之外而恰在 txg T 写下、不早于它：CPU 一条都不红，
/// I-9.7 与 I-9.39 判过而成立。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_live_data_units_are_born_at_the_truncation_field_txg() {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_live_data_units_are_born_at_the_truncation_field_txg";
    let flow = record_the_three_unit_file_with_its_record_changed(
        "crash-judge-gpu-live-units-born-at-the-truncation-field-txg",
        "live-units-born-at-the-truncation-field-txg",
        &|record| write_the_truncation_field(record, CHECKPOINT_TXG_OF_THE_THREE_UNIT_FILE, 0),
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &[],
        &["I-9.7", "I-9.39"],
    );
}

/// 合法世界：新池新建文件（一个单元，txg 3 写下）的记录截断字段写成 (4, 0)：⌈0 ÷ 净荷⌉.max(1) = 1，单元 0 不在截断字段之外，
/// 诞生代号早于 T 也是活的——CPU 一条都不红、I-9.39 判过而成立（去掉 .max(1) 就判死单元 0）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_to_size_zero_keeps_the_first_data_unit(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_a_truncation_field_to_size_zero_keeps_the_first_data_unit";
    let mut flow = record_new_pool_file_creation(
        "crash-judge-gpu-truncation-field-to-size-zero",
        "truncation-field-to-size-zero-keeping-the-first-data-unit",
    );
    change_the_record_of_the_first_file_in_the_last_inode_leaf(&mut flow, &|record| {
        write_the_truncation_field(record, CHECKPOINT_TXG_OF_THE_NEW_POOL_FILE + 1, 0);
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &[],
        &["I-9.39"],
    );
}

/// 合法世界：缩到一个半净荷那次意图（txg 5）的记录写上截断字段 (5, 一个半净荷)：留下两个单元，单元 2 在 size 之外还挂着、
/// 诞生代号 4 早于 T——它是截断中间态等着放的单元、不是活单元，I-9.39 不判它（「活单元」取 size 之内的读法），CPU 一条都不红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_truncation_intent_version_carries_its_truncation_field(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_truncation_intent_version_carries_its_truncation_field";
    let payload = u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("32634");
    let mut flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-truncation-intent-with-its-field",
        "truncation-intent-with-its-truncation-field",
        TruncationsAndHolesFlowStop::AfterTheIntentOfTheFirstTruncation,
    );
    change_the_record_of_the_first_file_in_the_last_inode_leaf(&mut flow, &|record| {
        write_the_truncation_field(
            record,
            CHECKPOINT_TXG_OF_THE_FIRST_TRUNCATION_INTENT,
            payload + payload / 2,
        );
    });
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &[],
        &["I-9.39"],
    );
}

/// 合法世界：每一槽都是这一版布局的 incompat 第一个字节（0x06，录下的流原样），记录不带截断字段：两个池级 checker 内核与恢复内核都收这一槽，
/// CPU 一条都不红、I-9.7 判过而成立。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_incompat_bit_two()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_incompat_bit_two";
    let flow = record_new_pool_through_truncations_and_holes(
        "crash-judge-gpu-incompat-bit-two-on-every-slot",
        "incompat-bit-two-on-every-slot",
        TruncationsAndHolesFlowStop::AfterTheThreeUnitFile,
    );
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_truncation_field_flow(
        TEST_NAME,
        &flow,
        &[],
        &["I-9.7"],
    );
}

/// 系统配置 incompat 第一个字节换成 `first_incompat_byte` 的三种不认得的写法（不是这一版布局的 0x06）：流里每一次系统配置槽写都换，基镜像里 mkfs 写下的两槽不动
/// （判表照它们取落点几何；那两槽也换就建不出表、GPU 不上卡）。一块盘的两槽都被换过的写盖过之后，那块盘上择不到系统配置，
/// 两块盘都这样的状态上恢复失败、CPU 判器红；每个状态 GPU 与 CPU 整份相同，GPU 两个池级 checker 内核与恢复内核认 incompat 位的那一判在卡上走到。
fn assert_the_gpu_judge_agrees_with_the_cpu_when_every_slot_carries_an_unrecognized_first_incompat_byte(
    test_name: &str,
    tag: &str,
    name: &'static str,
    first_incompat_byte: u8,
) {
    let mut flow = record_new_pool_through_truncations_and_holes(
        tag,
        name,
        TruncationsAndHolesFlowStop::AfterTheThreeUnitFile,
    );
    let system_configuration_slot_writes_changed =
        change_the_first_incompat_byte_of_every_system_configuration_slot_write_on(
            &mut flow,
            &BOTH_DEVICES,
            &|_first_byte| first_incompat_byte,
        );
    assert!(
        system_configuration_slot_writes_changed > 0,
        "流 {} 里有系统配置槽写（每次发布轮换系统配置）：没有就一个状态都换不到不认得的槽",
        flow.name
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_red > 0,
        "流 {}：每一槽都带不认得的 incompat 位（第一个字节 {first_incompat_byte:#04x}），CPU 判器在某些状态上红",
        flow.name
    );
    assert_the_pool_checker_half_then_the_recovery_half_agree(&flow, &comparison);
}

/// incompat 第一个字节 0x02：只带布局身份那一位 1、不带位 2，两边都不认（位 2 是这一版布局身份的一部分）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_bit_one_without_incompat_bit_two(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_when_every_slot_carries_an_unrecognized_first_incompat_byte(
        "the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_bit_one_without_incompat_bit_two",
        "crash-judge-gpu-incompat-0x02",
        "incompat-first-byte-0x02",
        INCOMPAT_FIRST_SSD_LINE_BIT,
    );
}

/// incompat 第一个字节 0x04：只带位 2、不带布局身份那一位 1，两边都不认。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_incompat_bit_two_without_bit_one(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_when_every_slot_carries_an_unrecognized_first_incompat_byte(
        "the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_incompat_bit_two_without_bit_one",
        "crash-judge-gpu-incompat-0x04",
        "incompat-first-byte-0x04",
        INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT,
    );
}

/// incompat 第一个字节 0x0E：位 1、位 2 之外还带没分出去的位 3。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_an_unassigned_incompat_bit(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_when_every_slot_carries_an_unrecognized_first_incompat_byte(
        "the_gpu_judge_agrees_with_the_cpu_when_every_system_configuration_slot_carries_an_unassigned_incompat_bit",
        "crash-judge-gpu-incompat-0x0e",
        "incompat-first-byte-0x0e",
        INCOMPAT_FIRST_SSD_LINE_BIT | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT | INCOMPAT_UNASSIGNED_BIT_THREE,
    );
}

/// 系统配置槽里格式版本那两字节（小端）的偏移：紧跟 magic 之后（池级 checker 的 `FORMAT_VERSION_OFFSET`、核心层恢复读格式版本是同一处）。
const SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET: usize = 4;

/// 流里每一次系统配置槽写的格式版本改成 `format_version`、incompat 第一个字节改成 `first_incompat_byte`，重封整槽校验和，交回改了几次。
/// 基镜像里 mkfs 写下的两槽不动：判表照它们取落点几何（那两槽也改就建不出表、GPU 不上卡，同
/// [`change_the_first_incompat_byte_of_every_system_configuration_slot_write_on`]）。
fn rewrite_the_format_version_and_the_first_incompat_byte_of_every_system_configuration_slot_write(
    flow: &mut RecordedFlow,
    format_version: u16,
    first_incompat_byte: u8,
) -> usize {
    let system_configuration_slot_writes = flow
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .count();
    for nth_from_the_end in 0..system_configuration_slot_writes {
        rewrite_and_reseal(
            flow,
            StepKind::SystemConfigurationSlot,
            nth_from_the_end,
            4096,
            SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
            &|bytes| {
                assert_eq!(&bytes[..4], b"SFSB", "流里的系统配置槽写是带 magic 的整槽");
                let format_version_bytes = SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET
                    ..SYSTEM_CONFIGURATION_FORMAT_VERSION_OFFSET + 2;
                assert_eq!(
                    bytes[format_version_bytes.clone()],
                    singlefs_core::system_configuration::FORMAT_VERSION.to_le_bytes(),
                    "写者写下的格式版本是这一版读者认的那一版"
                );
                bytes[format_version_bytes].copy_from_slice(&format_version.to_le_bytes());
                bytes[FIRST_INCOMPAT_BYTE_OFFSET] = first_incompat_byte;
            },
        );
    }
    system_configuration_slot_writes
}

/// C663「格式版本先」：新池新建文件流里每一次系统配置槽写 incompat 第一个字节写 0x0E（位 1、位 2 之外带没分出去的位 3）、
/// 格式版本写 2，重封整槽校验和——一槽同时带不认得的 incompat 位与不是这一版读者写的格式版本。核心层恢复先判格式版本、整池拒
/// （`SystemConfigurationValueRefused`，失败点 6），不走 incompat 位不认得那一路（失败点 2：那一路这一槽不可择、换一槽换一盘接着择）；
/// 池级 checker 读这一槽同样先判格式版本、读成带不收之值，红 I-7.13。改过的槽落了的状态上两遍恢复都整池拒，没有一个状态报 incompat 位不认得；
/// GPU 恢复内核与两个池级 checker 内核逐态跟上，每个状态整份相同。
/// 判别力（`crates/mutations.tsv`）：GPU 恢复内核在 incompat 位不认得那一臂里不先判格式版本，改过的槽落了的状态上 GPU 把它当不可择的槽、
/// 换槽接着择，两遍恢复的失败点与落到的根跟 CPU 不同，这条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_format_version_not_recognized_refuses_the_pool_before_an_unrecognized_incompat_bit(
) {
    use singlefs_verification::crash_judge_gpu::{
        GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED,
        GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED,
    };
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_that_a_format_version_not_recognized_refuses_the_pool_before_an_unrecognized_incompat_bit";
    let mut flow = record_new_pool_file_creation(
        "crash-judge-gpu-format-version-2-with-incompat-0x0e",
        "new-pool-system-configuration-format-version-2-and-incompat-first-byte-0x0e",
    );
    let system_configuration_slot_writes_rewritten =
        rewrite_the_format_version_and_the_first_incompat_byte_of_every_system_configuration_slot_write(
            &mut flow,
            singlefs_core::system_configuration::FORMAT_VERSION + 1,
            INCOMPAT_FIRST_SSD_LINE_BIT
                | INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT
                | INCOMPAT_UNASSIGNED_BIT_THREE,
        );
    assert!(
        system_configuration_slot_writes_rewritten > 0,
        "流 {} 里有系统配置槽写（每次发布轮换系统配置）：没有就一个状态都落不到改过的槽",
        flow.name
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[
            StateExamination::CountStatesWhere(StateCondition::GpuBothPassesFailAt(
                GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED,
            )),
            StateExamination::CountStatesWhere(StateCondition::GpuEitherPassFailsAt(
                GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED,
            )),
        ],
    ) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-7.13"),
        "流 {} 上 CPU 池级 checker 先判格式版本、把改过的槽读成带不收之值，红 I-7.13（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert_the_pool_checker_half_then_the_recovery_half_agree(&flow, &comparison);
    // 两边逐态整份相同（上一句），下面两个数 GPU 与 CPU 一样
    let states_whose_both_passes_refuse_the_pool_for_the_format_version =
        comparison.examined[0].states_counted;
    let states_naming_an_unrecognized_incompat_bit = comparison.examined[1].states_counted;
    assert!(
        states_whose_both_passes_refuse_the_pool_for_the_format_version > 0,
        "流 {}：改过的槽落了的状态上两遍恢复都按格式版本整池拒（失败点 {GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED}），实际 0 个状态",
        flow.name
    );
    assert_eq!(
        states_naming_an_unrecognized_incompat_bit, 0,
        "流 {}：格式版本先于 incompat 位判，没有一个状态报 incompat 位不认得（失败点 {GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED}）",
        flow.name
    );
}
