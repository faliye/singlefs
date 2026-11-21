//! 空间记账（I-3 类）：已分配统计那几条判定在一片判与多片判下相同。

use super::*;

/// 记账叶里已分配（统计量 1）那一行的统计量码。
const STATISTIC_ALLOCATED_BYTES: u16 = 1;

/// 分配记录跨度段（值的头 2 字节）里跨度那几位与已释放那一位（D3（空间分配） 已定项 7 的字段表：位 15 已释放、位 14 隔离记号、低 14 位跨度）。
const ALLOCATION_RECORD_SPAN_BITS: u16 = 0x3FFF;

const ALLOCATION_RECORD_RELEASED_BIT: u16 = 0x8000;

/// 分配记录在叶里那一条的跨度段从哪个字节起（key 盘 4、槽 6 之后）。
const ALLOCATION_RECORD_SPAN_FIELD_OFFSET: usize = ALLOCATION_RECORD_TREE_KEY_BYTES;

/// 码 1 数据单元占的槽数，与它那条记录改坏之后的跨度。
const SLOTS_OF_A_DATA_UNIT: u16 = 2;

const NARROWED_SPAN_OF_A_DATA_UNIT_RECORD: u16 = 1;

/// 分配记录树的叶（码 2、key 宽盘 4 加槽 6、条目宽 20）。
fn is_an_allocation_record_leaf(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == ALLOCATION_RECORD_TREE_KEY_BYTES
        && usize::from(index_node_entry_width(bytes)) == ALLOCATION_LEAF_RECORD_BYTES
}

/// 新池新建文件的流上记账叶里已分配那几行怎么改：不改；按 key 排在最前的那一行（几块盘各一行）加 1（与 [`record_the_resealed_field_world`] 里 I-3.1 那一条同一形：
/// 已分配不是槽宽的整数倍，那块盘按哪一种读法数都配不上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocatedStatisticInTheAccountingLeaf {
    AsWritten,
    OneRowOverstatedByOneByte,
}

/// 新池新建文件的流上，最后写的那一片分配记录叶里罩住码 1 数据单元的那条未释放记录，跨度从 2 槽改成 1 槽、重封到根（I-3.18 判红：记录的跨度与它罩住的
/// 单元那一类的槽数不符）。那块盘上 I-3.1 按记录自述的跨度三判不成立（数据单元第二槽遍历走到、不在记录里），按单元那一类再数一份整份成立：配不上的那块
/// 全落在 I-3.18 判红的单元里，那块盘记射程里判不了——
/// GPU 扫描内核记账按单元那一类那一轮真走。`allocated_statistic` 是 [`AllocatedStatisticInTheAccountingLeaf::OneRowOverstatedByOneByte`] 时再把记账叶里
/// 按 key 排在最前的那一行已分配加 1、重封到根：I-3.1 判红。
fn record_new_pool_with_the_record_of_a_data_unit_narrowed_to_one_slot(
    tag: &str,
    name: &'static str,
    allocated_statistic: AllocatedStatisticInTheAccountingLeaf,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    let data_unit_slots: std::collections::BTreeSet<(u32, u64)> = flow
        .writes
        .iter()
        .filter(|write| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| unit_class(bytes) == 1)
        })
        .map(|write| (write.device.0, write.offset.0 / SLOT_BYTES))
        .collect();
    assert!(!data_unit_slots.is_empty(), "流 {name} 里写过码 1 数据单元");
    let leaf = last_unit_copies(&flow, name, &is_an_allocation_record_leaf, false);
    let records_narrowed = std::cell::Cell::new(0usize);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &leaf, &|bytes| {
        let entry_count = usize::from(read_u16_at(
            bytes,
            index_node_entry_count_offset(ALLOCATION_RECORD_TREE_KEY_BYTES),
        ));
        for index in 0..entry_count {
            let record = index_node_entries_start(ALLOCATION_RECORD_TREE_KEY_BYTES)
                + index * ALLOCATION_LEAF_RECORD_BYTES;
            let mut slot = [0u8; 8];
            slot[..6]
                .copy_from_slice(&bytes[record + 4..record + ALLOCATION_RECORD_TREE_KEY_BYTES]);
            let span_field_offset = record + ALLOCATION_RECORD_SPAN_FIELD_OFFSET;
            let span_field = read_u16_at(bytes, span_field_offset);
            if span_field & ALLOCATION_RECORD_RELEASED_BIT == 0
                && span_field & ALLOCATION_RECORD_SPAN_BITS == SLOTS_OF_A_DATA_UNIT
                && data_unit_slots.contains(&(read_u32_at(bytes, record), u64::from_le_bytes(slot)))
            {
                let narrowed = (span_field & !ALLOCATION_RECORD_SPAN_BITS)
                    | NARROWED_SPAN_OF_A_DATA_UNIT_RECORD;
                bytes[span_field_offset..span_field_offset + 2]
                    .copy_from_slice(&narrowed.to_le_bytes());
                records_narrowed.set(records_narrowed.get() + 1);
            }
        }
    });
    assert!(
        records_narrowed.get() > 0,
        "流 {name}：最后写的那一片分配记录叶里有罩住码 1 数据单元的未释放记录（两份副本各改一次）"
    );
    match allocated_statistic {
        AllocatedStatisticInTheAccountingLeaf::AsWritten => {
            // 记账叶照写下的样子，不改
        }
        AllocatedStatisticInTheAccountingLeaf::OneRowOverstatedByOneByte => {
            let accounting_leaf = last_unit_copies(&flow, name, &is_an_accounting_leaf, false);
            corrupt_units_and_reseal_up_to_the_root(&mut flow, &accounting_leaf, &|bytes| {
                let entry_count = usize::from(read_u16_at(
                    bytes,
                    index_node_entry_count_offset(ACCOUNTING_KEY_BYTES),
                ));
                let allocated_row = (0..entry_count)
                    .map(|index| {
                        index_node_entries_start(ACCOUNTING_KEY_BYTES)
                            + index * ACCOUNTING_ROW_BYTES
                    })
                    .find(|row| read_u16_at(bytes, *row) == STATISTIC_ALLOCATED_BYTES)
                    .expect("记账叶里有已分配那一行");
                let value = read_u64_at(bytes, allocated_row + ACCOUNTING_ROW_VALUE_OFFSET);
                bytes[allocated_row + ACCOUNTING_ROW_VALUE_OFFSET
                    ..allocated_row + ACCOUNTING_ROW_VALUE_OFFSET + 8]
                    .copy_from_slice(&(value + 1).to_le_bytes());
            });
        }
    }
    flow
}

/// CPU 判器在一个状态上对 I-3.1 与 I-3.18 判出了什么（记账分片那条对拍数「I-3.1 判红的、记射程里判不了的各至少一个」用）。
#[derive(Clone, Copy, Debug)]
struct AllocatedStatisticVerdictsOnTheCpu {
    allocated_statistic_is_violated: bool,
    allocated_statistic_is_partially_judged: bool,
    allocation_record_span_invariant_is_violated: bool,
}

fn allocated_statistic_verdicts_on_the_cpu(
    judgement: &CrashStateJudgement,
) -> AllocatedStatisticVerdictsOnTheCpu {
    let verdict_of = |invariant: &str| {
        judgement
            .pool_checker
            .iter()
            .find(|(name, _)| *name == invariant)
            .map(|(_, verdict)| verdict)
    };
    AllocatedStatisticVerdictsOnTheCpu {
        allocated_statistic_is_violated: matches!(
            verdict_of("I-3.1"),
            Some(InvariantVerdict::Violated(_))
        ),
        allocated_statistic_is_partially_judged: matches!(
            verdict_of("I-3.1"),
            Some(InvariantVerdict::PartiallyJudged(_))
        ),
        allocation_record_span_invariant_is_violated: matches!(
            verdict_of("I-3.18"),
            Some(InvariantVerdict::Violated(_))
        ),
    }
}

/// 记账分片那几条流上 CPU 一侧要数出的 I-3.1 与 I-3.18 判定（[`assert_verdicts_are_the_same_whether_the_accounting_of_the_flow_is_judged_in_one_slice_or_in_many`]）：
/// 分片分得出差别的那几种状态。
enum AccountingVerdictsTheCpuShowsOnTheFlow {
    /// 合法流：不要求哪一样。
    NoneRequired,
    /// 至少一个状态 I-3.1 记射程里判不了（那块盘按单元那一类那一轮真走）。
    AllocatedStatisticJudgedInPart,
    /// 至少一个状态 I-3.1 判红，至少一个状态 I-3.1 与 I-3.18 同时判红。
    AllocatedStatisticRedAlsoTogetherWithTheRecordSpans,
}

/// 记账那一判拆成头、分片、决断、收尾几趟（分片两轮各切几片、每片一次提交），一条流的同一组状态整轮一片判、每轮 64 等份判、切得不齐（带一片零宽的）判、
/// 照判器自己的片宽判（[`GpuCrashJudge::judge_states`]），判出的结论逐状态逐字相同；整轮一片那一份与 CPU 判器逐状态对拍一致
/// （`.claude/rules/test-script-discipline.md`「用上现有的硬件加速」：加速路径留一份 CPU 参照实现，两边在同一组输入上判定逐项相同）；
/// CPU 一侧数出 `verdicts_the_cpu_shows` 要的那几种状态。展开 8；流由 `record_the_flow` 在取卡之后录。
/// 片的边界上那一项哪片都不做、宿主推片的上端到不了一轮的末尾、决断不记「按单元那一类再数一份」，各红在互比或对拍那一处
/// （`crates/mutations.tsv` 里点名这几条用例的几行）。显存额度装不下一个判器就报本次未跑。
/// 两类库里的项名是「`test_name`::流名」，合起来要在 1 到 200 字节以内（`crash_library::ItemName`，超了入库那一步当场 panic）。
fn assert_verdicts_are_the_same_whether_the_accounting_of_the_flow_is_judged_in_one_slice_or_in_many(
    test_name: &str,
    record_the_flow: &dyn Fn() -> RecordedFlow,
    verdicts_the_cpu_shows: AccountingVerdictsTheCpuShowsOnTheFlow,
) {
    const EXPAND_UP_TO: usize = 8;
    const EQUAL_SLICES: u32 = 64;
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let ends_of_the_slices_of_each_plan: [(&str, Vec<u32>); 3] = [
        ("整轮一片", vec![ACCOUNTING_SLICE_POSITIONS]),
        (
            "每轮 64 等份",
            (1..=EQUAL_SLICES)
                .map(|slice| slice * (ACCOUNTING_SLICE_POSITIONS / EQUAL_SLICES))
                .collect(),
        ),
        (
            "切得不齐、带一片零宽的",
            vec![
                1,
                3,
                3,
                700,
                20_000,
                ACCOUNTING_SLICE_POSITIONS - 1,
                ACCOUNTING_SLICE_POSITIONS,
            ],
        ),
    ];
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let mut states_whose_allocated_statistic_the_cpu_reddens = 0usize;
    let mut states_whose_allocated_statistic_the_cpu_judges_in_part = 0usize;
    let mut states_whose_allocated_statistic_and_record_spans_the_cpu_reddens = 0usize;
    let mut problems: Vec<String> = Vec::new();
    let recorded = record_the_flow();
    let flow = &recorded;
    let item_name = format!("{test_name}::{}", flow.name);
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    // 判表先入库、再读库（对拍那一类的库），GPU 只拿读回的那一份算；CPU 判器同样先把流入库、再从库里读回判
    let packed = packed_tables_through_the_library(&item_name, &tables);
    let cpu = every_state_judged_on_the_cpu(
        flow,
        &item_name,
        &[],
        &flow.segments,
        &expansion,
        ContainerCapacityKnobs::PRODUCT,
        state_count,
        0..state_count,
        &|_image, judgement| allocated_statistic_verdicts_on_the_cpu(judgement),
    );
    let (Some(packed), Some(cpu)) = (packed, cpu) else {
        return;
    };
    for (_, verdicts) in &cpu {
        states_whose_allocated_statistic_the_cpu_reddens +=
            usize::from(verdicts.allocated_statistic_is_violated);
        states_whose_allocated_statistic_the_cpu_judges_in_part +=
            usize::from(verdicts.allocated_statistic_is_partially_judged);
        states_whose_allocated_statistic_and_record_spans_the_cpu_reddens += usize::from(
            verdicts.allocated_statistic_is_violated
                && verdicts.allocation_record_span_invariant_is_violated,
        );
    }
    // 这条用例照判器的宽度一次派完（`judge_states_in_one_dispatch_slicing_the_accounting_at` 不按用时调宽度），开宽了一次提交在卡上算得太久、撞驱动的超时；
    // 判别力在记账怎么分片，不在宽度：一条缓冲
    let states_per_dispatch = states_of_one_scratch_lane_on_every_card_of(cards);
    let Some(lease) = cards.lease_card_or_report_not_run(
        &item_name,
        mebibytes_a_judge_needs(&packed, states_per_dispatch),
    ) else {
        return;
    };
    let Some((judged_by_each_plan, judged_following_the_slice_widths_of_the_judge)) =
        on_the_card_or_report_not_run_when_out_of_memory(
            &item_name,
            lease.card(),
            "建判器、按几种分片判这条流",
            || {
                let judge = GpuCrashJudge::on_card_with_packed_tables(
                    lease.card(),
                    &packed,
                    states_per_dispatch,
                );
                let dispatch_width =
                    u64::try_from(judge.states_per_dispatch()).expect("装得进 u64");
                let judged_by_each_plan: Vec<Vec<[u32; VERDICT_WORDS]>> =
                    ends_of_the_slices_of_each_plan
                        .iter()
                        .map(|(_, ends_of_the_slices)| {
                            let mut verdicts = Vec::new();
                            let mut start = 0u64;
                            while start < state_count {
                                let end = (start + dispatch_width).min(state_count);
                                verdicts.extend(
                                    judge.judge_states_in_one_dispatch_slicing_the_accounting_at(
                                        start..end,
                                        ends_of_the_slices,
                                    ),
                                );
                                start = end;
                            }
                            verdicts
                        })
                        .collect();
                (judged_by_each_plan, judge.judge_states(0..state_count))
            },
        )
    else {
        return;
    };
    drop(lease);
    let judged_in_one_slice = &judged_by_each_plan[0];
    let every_other_way = ends_of_the_slices_of_each_plan
        .iter()
        .zip(&judged_by_each_plan)
        .skip(1)
        .map(|((plan_name, _), verdicts)| (*plan_name, verdicts))
        .chain(std::iter::once((
            "照判器自己的片宽",
            &judged_following_the_slice_widths_of_the_judge,
        )));
    for (plan_name, verdicts) in every_other_way {
        assert_eq!(verdicts.len(), judged_in_one_slice.len());
        if let Some(ordinal) = judged_in_one_slice
            .iter()
            .zip(verdicts)
            .position(|(in_one_slice, other)| in_one_slice != other)
        {
            problems.push(format!(
                "流 {}：{plan_name}与整轮一片判出的结论不同，第一个不同的状态 {ordinal}：整轮一片 {:?}；{plan_name} {:?}",
                flow.name, judged_in_one_slice[ordinal], verdicts[ordinal]
            ));
        }
    }
    let cpu_judged: Vec<(JudgedOnCpu, SelfVerifiedSlotsOnTheCpu)> =
        cpu.into_iter().map(|(judged, _)| (judged, None)).collect();
    let comparison = compare_the_judgements_state_by_state(
        flow.name,
        0..state_count,
        &cpu_judged,
        judged_in_one_slice,
        &[],
        &tables.devices,
    );
    println!(
        "CRASH_JUDGE_ACCOUNTING_SLICES_AGREE flow={} states={state_count} mismatches_with_the_cpu={}",
        flow.name,
        comparison.mismatches.state_count()
    );
    if !comparison.mismatches.is_empty() {
        problems.push(format!(
            "流 {}：整轮一片那一份与 CPU 判器 {} 个状态不一致（前几个）：\n{}",
            flow.name,
            comparison.mismatches.state_count(),
            mismatch_listing(&comparison)
        ));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
    match verdicts_the_cpu_shows {
        AccountingVerdictsTheCpuShowsOnTheFlow::NoneRequired => {}
        AccountingVerdictsTheCpuShowsOnTheFlow::AllocatedStatisticJudgedInPart => {
            assert!(
                states_whose_allocated_statistic_the_cpu_judges_in_part > 0,
                "流 {}：CPU 判 I-3.1 记射程里判不了的状态 {states_whose_allocated_statistic_the_cpu_judges_in_part} 个，要至少一个（分片分得出差别的状态）",
                flow.name
            );
        }
        AccountingVerdictsTheCpuShowsOnTheFlow::AllocatedStatisticRedAlsoTogetherWithTheRecordSpans => {
            assert!(
                states_whose_allocated_statistic_the_cpu_reddens > 0
                    && states_whose_allocated_statistic_and_record_spans_the_cpu_reddens > 0,
                "流 {}：CPU 判 I-3.1 红的 {states_whose_allocated_statistic_the_cpu_reddens} 个、I-3.1 与 I-3.18 同时红的 \
                 {states_whose_allocated_statistic_and_record_spans_the_cpu_reddens} 个：两样都要至少一个（分片分得出差别的状态）",
                flow.name
            );
        }
    }
}

/// 场景：固定脚本到回退 D（合法流）。预期：记账整轮一片、每轮 64 等份、切得不齐、照判器自己的片宽判出的结论逐状态逐字相同，整轮一片那一份与 CPU 逐状态一致。
#[test]
fn accounting_verdicts_match_across_slicings_on_the_fixed_script_through_a_rollback() {
    assert_verdicts_are_the_same_whether_the_accounting_of_the_flow_is_judged_in_one_slice_or_in_many(
        "accounting_verdicts_match_across_slicings_on_the_fixed_script_through_a_rollback",
        &|| record_fixed_script_through_rollback("crash-judge-gpu-accounting-slices"),
        AccountingVerdictsTheCpuShowsOnTheFlow::NoneRequired,
    );
}

/// 场景：新池新建文件、最后那片分配记录叶里数据单元那条记录只记一槽（I-3.18 红，I-3.1 在那块盘上按单元那一类那一轮真走）。
/// 预期：几种分片判出的结论逐状态逐字相同、与 CPU 逐状态一致，CPU 判 I-3.1 记射程里判不了的状态至少一个。
#[test]
fn accounting_verdicts_match_across_slicings_on_a_new_pool_with_a_data_unit_record_narrowed_to_one_slot(
) {
    assert_verdicts_are_the_same_whether_the_accounting_of_the_flow_is_judged_in_one_slice_or_in_many(
        "accounting_verdicts_match_across_slicings_on_a_new_pool_with_a_data_unit_record_narrowed_to_one_slot",
        &|| {
            record_new_pool_with_the_record_of_a_data_unit_narrowed_to_one_slot(
                "crash-judge-gpu-accounting-slices-narrowed",
                "new-pool-data-unit-record-narrowed-to-one-slot",
                AllocatedStatisticInTheAccountingLeaf::AsWritten,
            )
        },
        AccountingVerdictsTheCpuShowsOnTheFlow::AllocatedStatisticJudgedInPart,
    );
}

/// 场景：新池新建文件、数据单元那条记录只记一槽那一形，再把记账叶里一行已分配加 1（I-3.1 红）。
/// 预期：几种分片判出的结论逐状态逐字相同、与 CPU 逐状态一致，CPU 判 I-3.1 红的状态、I-3.1 与 I-3.18 同时红的状态各至少一个。
#[test]
fn accounting_verdicts_match_across_slicings_on_a_new_pool_with_a_data_unit_record_narrowed_and_one_allocated_row_overstated(
) {
    assert_verdicts_are_the_same_whether_the_accounting_of_the_flow_is_judged_in_one_slice_or_in_many(
        "accounting_verdicts_match_across_slicings_on_a_new_pool_with_a_data_unit_record_narrowed_and_one_allocated_row_overstated",
        &|| {
            record_new_pool_with_the_record_of_a_data_unit_narrowed_to_one_slot(
                "crash-judge-gpu-accounting-slices-narrowed-and-overstated",
                "new-pool-data-unit-record-narrowed-to-one-slot-and-allocated-overstated",
                AllocatedStatisticInTheAccountingLeaf::OneRowOverstatedByOneByte,
            )
        },
        AccountingVerdictsTheCpuShowsOnTheFlow::AllocatedStatisticRedAlsoTogetherWithTheRecordSpans,
    );
}
