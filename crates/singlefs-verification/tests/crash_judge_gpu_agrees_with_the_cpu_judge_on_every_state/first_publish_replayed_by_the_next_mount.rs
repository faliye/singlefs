//! 下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认：首条没落、前缀停在后面那次发布的点名单元上。

use super::*;

// ─── 下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1 的 GPU 一半） ───

/// I-5.4 在下一次挂载先施加的那一版上的违例说明的开头（`walk.rs` 按版本那一层的称呼）。
const VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR: &str =
    "下一次挂载先施加的那一版（实例 1、txg 4）";

/// 改法 R1 断号那一形（判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第二节 P1 与第三节第 1 条，攻方
/// `cut=records-2..2-of-2`；`checker_known_bad_images.rs` 的
/// `a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first` 同形）：三个数据单元那段历史录到顺序写那次发布
/// （txg 4，计数器 4、5、6 三条记录）的根槽写之前停（崩在那次根槽 FUA 之前，R2 的合法造法），那次发布写的分配记录叶里两条记录罩住同一个槽，
/// 重封进三条记录、后面的记录重链。三条记录两块盘六次写在同一段里、每个子集都展开：首条两份都没落而后两条落了的状态在里面。
fn record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot(
    tag: &str,
) -> RecordedFlow {
    let name = "three-unit-file-crashed-before-its-root-over-overlapping-allocation-records";
    let mut flow = record_three_unit_file_up_to_the_root_of_its_three_record_publish(tag, name);
    let targets = last_unit_copies(
        &flow,
        name,
        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        assert_eq!(
            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
            1,
            "挑叶时这一片改得动"
        );
    });
    let records = journal_record_writes_of_the_publish(&flow, 4);
    let (first_record_write, _) = *records.first().expect("顺序写那次发布有记录");
    let segment_of_the_records = flow
        .segments
        .iter()
        .find(|segment| segment.contains(&first_record_write))
        .expect("记录写在某一段里");
    assert!(
        records.len() == 6
            && records
                .iter()
                .all(|(index, _)| segment_of_the_records.contains(index))
            && segment_of_the_records.len() <= expand_up_to(),
        "流 {name}：txg 4 的三条记录两块盘六次写在同一段里、那一段不多于展开上限 {} 次写（每个子集都展开）",
        expand_up_to()
    );
    flow
}

/// 断号那一形那条流（[`record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot`]，全量域）的状态数：
/// 一条用例判不完，按状态序号区间拆成 [`ordinals_of_the_cases_of_the_flow_whose_first_record_did_not_land`] 那几条用例；每条用例开跑之前在 CPU 上核
/// 这条流算出的状态数等于它。
const STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND: u64 = 1_090_519_154;

/// 断号那一形那条流拆开时一条用例判几个状态（最后一条另带不满一条的零头）：对拍一组的状态数（[`STATES_PER_PAIRING_BLOCK`] ×
/// [`PAIRING_BLOCKS_JUDGED_TOGETHER`]）的整数倍，区间两头落在组的边界上，块键与整条流跑时逐块相同。
const STATES_PER_CASE_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND: u64 = 1 << 24;

/// 断号那一形那条流拆成的几条用例各判的状态序号区间，按序号排：每段 [`STATES_PER_CASE_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND`] 个状态，
/// 不满一段的零头并进最后一段（最后一段到整条流的状态数为止，含序号最大的那个「全部持久」的状态）。
fn ordinals_of_the_cases_of_the_flow_whose_first_record_did_not_land() -> Vec<std::ops::Range<u64>>
{
    let cases = STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND
        / STATES_PER_CASE_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND;
    let mut ordinals_of_the_cases: Vec<std::ops::Range<u64>> = (0..cases - 1)
        .map(|case| {
            case * STATES_PER_CASE_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND
                ..(case + 1) * STATES_PER_CASE_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND
        })
        .collect();
    ordinals_of_the_cases.push(
        (cases - 1) * STATES_PER_CASE_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND
            ..STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND,
    );
    ordinals_of_the_cases
}

/// 断号那一形那条流拆成的用例各判的状态序号区间（不上卡、不录流、不判状态，只算区间表）：按序号排、首尾相接、两两不交，并起来恰好是
/// `[0, 整条流的状态数)`；每段两头落在对拍分组的边界上（终点另可是整条流的状态数），各段的组区间同样首尾相接、盖满整条流的组；
/// 共 65 段，与下面 65 条用例一一对上（每条用例开跑前核自己那一段在表里）。
/// 判别力（`crates/mutations.tsv`）：区间表里整段那几段的终点多一个状态（相邻两段重叠一个状态），这条红。
#[test]
fn the_ordinals_of_the_cases_of_the_flow_whose_first_record_did_not_land_cover_every_state_once() {
    let ordinals_of_the_cases = ordinals_of_the_cases_of_the_flow_whose_first_record_did_not_land();
    assert_eq!(
        ordinals_of_the_cases.len(),
        65,
        "区间表 65 段，下面 65 条用例各判一段"
    );
    let states_per_block = std::num::NonZeroU64::new(STATES_PER_PAIRING_BLOCK).expect("块宽不是 0");
    let blocks_per_group =
        std::num::NonZeroUsize::new(PAIRING_BLOCKS_JUDGED_TOGETHER).expect("一次判的块数不是 0");
    let states_per_group = STATES_PER_PAIRING_BLOCK
        * u64::try_from(PAIRING_BLOCKS_JUDGED_TOGETHER).expect("装得进 u64");
    let mut next_ordinal = 0u64;
    let mut next_group = 0u64;
    for ordinals in &ordinals_of_the_cases {
        assert_eq!(
            ordinals.start, next_ordinal,
            "状态序号区间 {ordinals:?} 接着前一段的终点起：首尾相接、两两不交"
        );
        assert!(
            ordinals.start < ordinals.end
                && ordinals.end - ordinals.start
                    < 2 * STATES_PER_CASE_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND,
            "状态序号区间 {ordinals:?} 不空，至多一条用例的宽度加不满一条的零头"
        );
        let groups = groups_of_pairing_blocks_holding_the_states(
            ordinals,
            STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND,
            states_per_block,
            blocks_per_group,
        )
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        assert_eq!(
            groups.start, next_group,
            "状态序号区间 {ordinals:?} 的组区间 {groups:?} 接着前一段的组区间起"
        );
        next_ordinal = ordinals.end;
        next_group = groups.end;
    }
    assert_eq!(
        next_ordinal, STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND,
        "各段并起来到整条流的状态数为止"
    );
    assert_eq!(
        next_group,
        STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND.div_ceil(states_per_group),
        "各段的组区间并起来盖满整条流的组"
    );
}

/// 断号那一形那条流里「首条（计数器 4）两份都没落、计数器 5 与 6 四份都落了」那一类状态序号最小的一个：除了首条两份，录制流里每次写都落了
/// （CPU 一侧钉的那张「首条两份都不落、其余照落」的镜像就是它）。改法 R1 的 GPU 一半改回只看带末条标志的那一条记录时，GPU 在它上面认计数器 6、
/// 红 I-5.4，CPU 不红：`crates/mutations.tsv` 里那一行点名判它所在那一段的用例。
const ORDINAL_OF_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND: u64 = 1_090_519_150;

/// [`ORDINAL_OF_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND`] 落在的那一段用例的状态序号区间（区间表里到整条流的状态数为止的那一段）。
const ORDINALS_OF_THE_CASE_HOLDING_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND:
    std::ops::Range<u64> = 1_073_741_824..STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND;

/// 断号那一形那条流上，「首条两份都没落、后两条四份都落了」那一类状态只在三条记录那一段里（不上卡、不判状态，只录流、建表、按序号取持久集合）：
/// 发布先写单元、屏障，再写三条记录、屏障（`singlefs_core::transaction` 的 `persist_publish_writes`），流录到根槽 FUA 之前停，
/// 所以六次记录写自成一段；它之前的段里记录一次都没落，它之后的状态（全部持久那一个）首条落了。逐个取这一段的状态（全量域上 63 个），
/// 这一类里序号最小的就是 [`ORDINAL_OF_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND`]，它的持久集合是「除首条两份外每次写都落了」，
/// 落在 [`ORDINALS_OF_THE_CASE_HOLDING_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND`] 那一段用例里。
#[test]
fn the_state_where_only_the_first_record_did_not_land_is_judged_by_the_case_ending_at_the_state_count_of_the_flow(
) {
    assert!(
        ordinals_of_the_cases_of_the_flow_whose_first_record_did_not_land().contains(
            &ORDINALS_OF_THE_CASE_HOLDING_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND
        ),
        "状态序号区间 {:?} 是区间表里的一段",
        ORDINALS_OF_THE_CASE_HOLDING_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND
    );
    let flow =
        record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot(
            "crash-judge-gpu-r1-first-record-not-landed-ordinal",
        );
    let expansion = expansion_up_to(expand_up_to());
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables_against_the_version_table(&judge_flow, &[]).expect("建得出表");
    assert_eq!(
        tables.state_count, STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND,
        "流 {}：全量域上的状态数是区间表拆的那个数",
        flow.name
    );
    let records = journal_record_writes_of_the_publish(&flow, 4);
    let first_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .min()
        .expect("顺序写那次发布有记录");
    let copies_of_the_first_record: Vec<usize> = records
        .iter()
        .filter(|(_, counter)| *counter == first_counter)
        .map(|(index, _)| *index)
        .collect();
    let copies_of_the_later_records: Vec<usize> = records
        .iter()
        .filter(|(_, counter)| *counter != first_counter)
        .map(|(index, _)| *index)
        .collect();
    assert_eq!(
        (
            copies_of_the_first_record.len(),
            copies_of_the_later_records.len()
        ),
        (2, 4),
        "首条两块盘各一份，后两条两块盘共四份"
    );
    let segment_of_the_records = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&copies_of_the_first_record[0]))
        .expect("记录写在某一段里");
    let mut writes_of_the_segment = flow.segments[segment_of_the_records].clone();
    writes_of_the_segment.sort_unstable();
    let mut writes_of_the_records: Vec<usize> = records.iter().map(|(index, _)| *index).collect();
    writes_of_the_records.sort_unstable();
    assert_eq!(
        writes_of_the_segment, writes_of_the_records,
        "流 {}：三条记录两块盘六次写自成一段（单元写之后、记录之前有屏障，记录之后有屏障）",
        flow.name
    );
    let states_of_the_segment = tables.segments[segment_of_the_records].states.clone();
    let is_of_the_class = |persisted: &[bool]| {
        copies_of_the_first_record
            .iter()
            .all(|index| !persisted[*index])
            && copies_of_the_later_records
                .iter()
                .all(|index| persisted[*index])
    };
    let smallest_ordinal_of_the_class = states_of_the_segment
        .clone()
        .find(|ordinal| is_of_the_class(&tables.persisted_writes_of_state(*ordinal)))
        .unwrap_or_else(|| {
            panic!(
                "流 {}：三条记录那一段（状态序号 {states_of_the_segment:?}）里没有首条两份都没落、后两条四份都落了的状态",
                flow.name
            )
        });
    assert_eq!(
        smallest_ordinal_of_the_class,
        ORDINAL_OF_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND,
        "流 {}：这一类状态序号最小的一个（三条记录那一段的状态序号 {states_of_the_segment:?}）",
        flow.name
    );
    let persisted = tables.persisted_writes_of_state(smallest_ordinal_of_the_class);
    let landed_writes: Vec<usize> = (0..flow.writes.len())
        .filter(|index| persisted[*index])
        .collect();
    let every_write_but_the_first_record: Vec<usize> = (0..flow.writes.len())
        .filter(|index| !copies_of_the_first_record.contains(index))
        .collect();
    assert_eq!(
        landed_writes, every_write_but_the_first_record,
        "序号 {smallest_ordinal_of_the_class}：除首条两份外录制流里每次写都落了"
    );
    assert!(
        persisted[flow.writes.len()..]
            .iter()
            .all(|is_persisted| !is_persisted),
        "序号 {smallest_ordinal_of_the_class}：没有一次写撕裂（撕裂镜像一个都不持久）"
    );
    assert!(
        ORDINALS_OF_THE_CASE_HOLDING_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND
            .contains(&smallest_ordinal_of_the_class),
        "序号 {smallest_ordinal_of_the_class} 落在状态序号区间 {:?} 那一段用例里",
        ORDINALS_OF_THE_CASE_HOLDING_THE_STATE_WHERE_ONLY_THE_FIRST_RECORD_DID_NOT_LAND
    );
}

/// 断号那一形（[`record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot`]）里状态序号落在 `ordinals` 的那一段。
/// CPU 一侧先钉流造对了：三条记录都落了的镜像上 txg 4 是下一次挂载先施加的那一版，I-5.4 红在它身上；首条（计数器 4）两份都不落、其余照落的镜像上
/// core 断号不施加 txg 4，CPU 照同一份重放不认它，I-5.4 在 txg 3 那一版上判过而成立（两张镜像便宜，每段都钉）。GPU 扫描内核读走读内核那一份重放交来的
/// 第一次发布，这一段每个状态逐位相同（只看带末条标志的那一条就认计数器 6，断号的状态上 GPU 红 I-5.4 而 CPU 不红）。「CPU 池级 checker 红过 I-5.4」
/// 只在含「全部持久」那个状态（序号最大，就是三条记录都落了的那张镜像）的最后一段上断言。`test_name` 是调它的用例名（本次未跑那一行点名它）。
fn assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
    test_name: &str,
    ordinals: std::ops::Range<u64>,
) {
    assert!(
        ordinals_of_the_cases_of_the_flow_whose_first_record_did_not_land().contains(&ordinals),
        "用例 {test_name} 判的状态序号 {ordinals:?} 是区间表里的一段"
    );
    let flow =
        record_three_unit_file_crashed_before_its_root_over_two_allocation_records_covering_one_slot(
            "crash-judge-gpu-r1-first-record-not-landed",
        );
    assert_eq!(
        crash_point_replay_plan_state_count(
            &flow.base,
            &flow.writes,
            &flow.segments,
            &expansion_up_to(expand_up_to())
        ),
        STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND,
        "流 {}：全量域上的状态数是区间表拆的那个数",
        flow.name
    );
    let mut image_with_every_record_landed = flow.base.clone();
    image_with_every_record_landed.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_every_record_landed, "I-5.4") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.starts_with(VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR),
            "三条记录都落了：I-5.4 红在下一次挂载先施加的那一版 txg 4 上：{detail}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "三条记录都落了：txg 4 是下一次挂载先施加的那一版，I-5.4 要红，实际 {other_verdict:?}"
        ),
    }
    let records = journal_record_writes_of_the_publish(&flow, 4);
    let first_counter = records
        .iter()
        .map(|(_, counter)| *counter)
        .min()
        .expect("顺序写那次发布有记录");
    let copies_of_the_first_record: Vec<usize> = records
        .iter()
        .filter(|(_, counter)| *counter == first_counter)
        .map(|(index, _)| *index)
        .collect();
    assert_eq!(copies_of_the_first_record.len(), 2, "首条两块盘各一份");
    let writes_without_the_first_record: Vec<RetainedWrite> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(index, _)| !copies_of_the_first_record.contains(index))
        .map(|(_, write)| write.clone())
        .collect();
    let mut image_with_the_first_record_not_landed = flow.base.clone();
    image_with_the_first_record_not_landed.apply_writes(&writes_without_the_first_record);
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_with_the_first_record_not_landed, "I-5.4"),
        InvariantVerdict::Holds,
        "首条两份都没落、后两条落了：core 断号不施加 txg 4，CPU 不认它，I-5.4 在 txg 3 那一版上判过而成立"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) =
        compare_on_every_state_with_ordinals_in(cards, &flow, expand_up_to(), ordinals.clone())
    else {
        return;
    };
    if ordinals.end == STATES_OF_THE_FLOW_WHOSE_FIRST_RECORD_DID_NOT_LAND {
        assert!(
            comparison.cpu_checker_red_invariants.contains("I-5.4"),
            "流 {}（状态序号 {ordinals:?}，含全部持久的那个状态）上 CPU 池级 checker 要红 I-5.4（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}（状态序号 {ordinals:?}）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

// 断号那一形那条流按状态序号区间拆成的用例：每条判一段（[`ordinals_of_the_cases_of_the_flow_whose_first_record_did_not_land`]），名字写区间的两个端点。

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_0_to_16777216(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_0_to_16777216",
        0..16777216,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_16777216_to_33554432(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_16777216_to_33554432",
        16777216..33554432,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_33554432_to_50331648(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_33554432_to_50331648",
        33554432..50331648,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_50331648_to_67108864(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_50331648_to_67108864",
        50331648..67108864,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_67108864_to_83886080(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_67108864_to_83886080",
        67108864..83886080,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_83886080_to_100663296(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_83886080_to_100663296",
        83886080..100663296,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_100663296_to_117440512(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_100663296_to_117440512",
        100663296..117440512,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_117440512_to_134217728(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_117440512_to_134217728",
        117440512..134217728,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_134217728_to_150994944(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_134217728_to_150994944",
        134217728..150994944,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_150994944_to_167772160(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_150994944_to_167772160",
        150994944..167772160,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_167772160_to_184549376(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_167772160_to_184549376",
        167772160..184549376,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_184549376_to_201326592(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_184549376_to_201326592",
        184549376..201326592,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_201326592_to_218103808(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_201326592_to_218103808",
        201326592..218103808,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_218103808_to_234881024(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_218103808_to_234881024",
        218103808..234881024,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_234881024_to_251658240(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_234881024_to_251658240",
        234881024..251658240,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_251658240_to_268435456(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_251658240_to_268435456",
        251658240..268435456,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_268435456_to_285212672(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_268435456_to_285212672",
        268435456..285212672,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_285212672_to_301989888(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_285212672_to_301989888",
        285212672..301989888,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_301989888_to_318767104(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_301989888_to_318767104",
        301989888..318767104,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_318767104_to_335544320(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_318767104_to_335544320",
        318767104..335544320,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_335544320_to_352321536(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_335544320_to_352321536",
        335544320..352321536,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_352321536_to_369098752(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_352321536_to_369098752",
        352321536..369098752,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_369098752_to_385875968(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_369098752_to_385875968",
        369098752..385875968,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_385875968_to_402653184(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_385875968_to_402653184",
        385875968..402653184,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_402653184_to_419430400(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_402653184_to_419430400",
        402653184..419430400,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_419430400_to_436207616(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_419430400_to_436207616",
        419430400..436207616,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_436207616_to_452984832(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_436207616_to_452984832",
        436207616..452984832,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_452984832_to_469762048(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_452984832_to_469762048",
        452984832..469762048,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_469762048_to_486539264(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_469762048_to_486539264",
        469762048..486539264,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_486539264_to_503316480(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_486539264_to_503316480",
        486539264..503316480,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_503316480_to_520093696(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_503316480_to_520093696",
        503316480..520093696,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_520093696_to_536870912(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_520093696_to_536870912",
        520093696..536870912,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_536870912_to_553648128(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_536870912_to_553648128",
        536870912..553648128,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_553648128_to_570425344(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_553648128_to_570425344",
        553648128..570425344,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_570425344_to_587202560(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_570425344_to_587202560",
        570425344..587202560,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_587202560_to_603979776(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_587202560_to_603979776",
        587202560..603979776,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_603979776_to_620756992(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_603979776_to_620756992",
        603979776..620756992,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_620756992_to_637534208(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_620756992_to_637534208",
        620756992..637534208,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_637534208_to_654311424(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_637534208_to_654311424",
        637534208..654311424,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_654311424_to_671088640(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_654311424_to_671088640",
        654311424..671088640,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_671088640_to_687865856(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_671088640_to_687865856",
        671088640..687865856,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_687865856_to_704643072(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_687865856_to_704643072",
        687865856..704643072,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_704643072_to_721420288(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_704643072_to_721420288",
        704643072..721420288,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_721420288_to_738197504(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_721420288_to_738197504",
        721420288..738197504,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_738197504_to_754974720(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_738197504_to_754974720",
        738197504..754974720,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_754974720_to_771751936(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_754974720_to_771751936",
        754974720..771751936,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_771751936_to_788529152(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_771751936_to_788529152",
        771751936..788529152,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_788529152_to_805306368(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_788529152_to_805306368",
        788529152..805306368,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_805306368_to_822083584(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_805306368_to_822083584",
        805306368..822083584,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_822083584_to_838860800(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_822083584_to_838860800",
        822083584..838860800,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_838860800_to_855638016(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_838860800_to_855638016",
        838860800..855638016,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_855638016_to_872415232(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_855638016_to_872415232",
        855638016..872415232,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_872415232_to_889192448(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_872415232_to_889192448",
        872415232..889192448,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_889192448_to_905969664(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_889192448_to_905969664",
        889192448..905969664,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_905969664_to_922746880(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_905969664_to_922746880",
        905969664..922746880,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_922746880_to_939524096(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_922746880_to_939524096",
        922746880..939524096,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_939524096_to_956301312(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_939524096_to_956301312",
        939524096..956301312,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_956301312_to_973078528(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_956301312_to_973078528",
        956301312..973078528,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_973078528_to_989855744(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_973078528_to_989855744",
        973078528..989855744,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_989855744_to_1006632960(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_989855744_to_1006632960",
        989855744..1006632960,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1006632960_to_1023410176(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1006632960_to_1023410176",
        1006632960..1023410176,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1023410176_to_1040187392(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1023410176_to_1040187392",
        1023410176..1040187392,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1040187392_to_1056964608(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1040187392_to_1056964608",
        1040187392..1056964608,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1056964608_to_1073741824(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1056964608_to_1073741824",
        1056964608..1073741824,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1073741824_to_1090519154(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states(
        "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first_on_the_states_from_1073741824_to_1090519154",
        1073741824..1090519154,
    );
}

/// 流里 (实例 1, `checkpoint_txg`) 那条根的最后一次根槽写，抄一份、根记录树表指针那一段里翻一个字节（不重封：那个根槽读不出）。
fn torn_copy_of_the_root_write_for_txg(flow: &RecordedFlow, checkpoint_txg: u64) -> RetainedWrite {
    let mut torn = flow
        .writes
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::RootRecordFua
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
                        && u64::from_le_bytes(
                            bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
                                ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                                .try_into()
                                .expect("8"),
                        ) == checkpoint_txg
                })
        })
        .unwrap_or_else(|| panic!("流 {} 里有 (1, {checkpoint_txg}) 的根槽写", flow.name))
        .clone();
    let WrittenContents::Bytes(bytes) = &mut torn.contents else {
        panic!("根槽写是普通写");
    };
    let flipped = usize::try_from(common::BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER)
        .expect("根记录里的下标");
    bytes[flipped] ^= 0xff;
    torn
}

/// 改法 R1 的另一半（harness
/// `a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first` 同形）：
/// 新池新建文件（A，txg 3）之后同一个进程、实例 1 里再覆盖写两次：B (1, 4)、C (1, 5)，各一条记录（计数器 4、5）；整段历史照录。
/// B 写的分配记录叶里两条记录罩住同一个槽，重封到 B 的根与记录、C 的记录重链。流尾另起一段（R2 改记录那一条流的造法：之前的写不动）：
/// B、C 两次的根槽各翻一个字节（两个根槽都读不出，与 harness 那一形的 `tear_the_root_slot_written_for_txg` 同一处），
/// C 那条记录第一个点名单元第二份落点上那一份翻一个字节、不重封（介质损坏）。这一段全落的状态上 core 择 A 的根、施加 B、停在 C 之前。
fn record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second(
    tag: &str,
) -> RecordedFlow {
    let name =
        "two-single-record-publishes-only-in-their-records-stopped-by-a-named-unit-of-the-second";
    let mut pool = build_pool(tag);
    common::overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &second_content());
    common::overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &third_content());
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        name,
        vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: second_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(5),
                content: third_content(),
            },
        ],
    );
    let targets = last_unit_copies(
        &flow,
        name,
        &is_an_allocation_leaf_born_at_txg_four_that_can_be_widened,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        assert_eq!(
            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
            1,
            "挑叶时这一片改得动"
        );
    });
    let (record_of_the_second_publish, _) = *journal_record_writes_of_the_publish(&flow, 5)
        .first()
        .expect("C 有记录");
    let record_bytes = flow.writes[record_of_the_second_publish]
        .bytes()
        .expect("记录写带字节");
    assert!(
        u32::from_le_bytes(record_bytes[12..16].try_into().expect("4")) > 0,
        "C 那条记录有点名项"
    );
    let second_location = JOURNAL_RECORD_HEADER_BYTES + 14;
    let named_device = u32::from_le_bytes(
        record_bytes[second_location..second_location + 4]
            .try_into()
            .expect("4"),
    );
    let mut named_slot = [0u8; 8];
    named_slot[..6].copy_from_slice(&record_bytes[second_location + 4..second_location + 10]);
    let named_offset = u64::from_le_bytes(named_slot) * SLOT_BYTES;
    let mut flipped_named_unit = flow
        .writes
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device.0 == named_device
                && write.offset.0 == named_offset
        })
        .expect("流里有 C 第一个点名单元第二份那一次单元写")
        .clone();
    let WrittenContents::Bytes(unit_bytes) = &mut flipped_named_unit.contents else {
        panic!("单元写是普通写");
    };
    unit_bytes[1000] ^= 0xff;
    let torn_roots =
        [4u64, 5].map(|checkpoint_txg| torn_copy_of_the_root_write_for_txg(&flow, checkpoint_txg));
    append_a_segment_of_writes(
        &mut flow,
        torn_roots.into_iter().chain([flipped_named_unit]).collect(),
    );
    flow
}

/// 前缀停在后面那次发布的点名单元上那一形（[`record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second`]）。
/// CPU 一侧先钉流造对了：整条落盘的镜像上 core 的只读恢复施加 B 一条、验不过 C 一条、落在 (1, 4)；I-5.4 照认 B（停之前已整次收下的第一次发布）、
/// 红在它身上，I-4.1 报不适用（起点含介质损坏）。GPU 走读内核那一份重放停在 C 时照交回 B 的末条，扫描内核同红，每个状态逐位相同
/// （停在点名单元上就交 NONE 的话，那个状态上 GPU 不红 I-5.4 而 CPU 红）。
#[test]
// cpu-reference-for-gpu-comparison
fn the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first(
) {
    let flow = record_two_single_record_publishes_stopped_by_a_named_unit_of_the_second(
        "crash-judge-gpu-r1-stopped-by-a-named-unit",
    );
    // 取卡之前的 CPU 前提判的是整条落盘那一张镜像经对拍那一类的库读回的那一份（CPU 参照也从库读）；只录那一趟当场返回
    let Some(image) =
        image_with_every_write_landed_read_back_from_the_pairing_library(&flow.base, &flow.writes)
    else {
        return;
    };
    let report =
        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Consult);
    assert_eq!(
        (
            report.effective_root,
            report.journal.prefix_applied,
            report.journal.verification_failed
        ),
        (Some((InstanceGeneration(1), CheckpointTxg(4))), 1, 1),
        "整条落盘：core 施加 B 那一条、验不过 C 那一条，落在 (1, 4)"
    );
    match pool_checker_verdict_on_the_image(&image, "I-5.4") {
        InvariantVerdict::Violated(detail) => assert!(
            detail.starts_with(VIOLATION_ON_THE_VERSION_THE_NEXT_MOUNT_APPLIES_FIRST_AT_TXG_FOUR),
            "重放停在 C 之前、B 照认：I-5.4 红在下一次挂载先施加的那一版 B 上：{detail}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("重放停在 C 之前、B 照认：I-5.4 要红，实际 {other_verdict:?}")
        }
    }
    match pool_checker_verdict_on_the_image(&image, "I-4.1") {
        InvariantVerdict::NotApplicable(reason) => assert!(
            reason.contains("介质损坏"),
            "I-4.1 报不适用的理由要说是起点含介质损坏：{reason}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("前缀停在 C 的点名单元上时 I-4.1 该报不适用，实际 {other_verdict:?}")
        }
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}
