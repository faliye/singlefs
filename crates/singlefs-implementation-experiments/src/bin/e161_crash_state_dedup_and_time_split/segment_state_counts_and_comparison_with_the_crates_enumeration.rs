//! 停机条款 S3、S4 与锚点 A1：段内去重计数与 crates 回比。

use super::*;

// ───────────────────────────── 停机条款 S3、S4 与锚点 A1 ─────────────────────────────

/// A1：一段的三方状态数（装置计划逐个数出来的、crates 只展开那一段算的、公式算的）。
pub(super) struct SegmentCountCheck {
    pub(super) segment_index: usize,
    pub(super) writes: usize,
    pub(super) in_place_overwrites: u64,
    pub(super) counted_by_the_plan: u64,
    pub(super) counted_by_the_crates: u64,
    pub(super) counted_by_the_formula: u64,
}

/// 逐个数：只展开 `segment_index` 那一段的计划里，每个序号的持久集合真生成出来，数不同的有几个。取样段里一律逐个数（段内这一段的写、
/// 它们的撕裂镜像与重放至多 32 个位置时用位图，否则用集合）；取样段之外（覆盖写固定脚本流段 0–11 与段 29 起，一段至多 2²⁸ − 1 个）按计划的组合数报。
/// 交回（数出来的个数，是不是逐个数的）。
fn count_distinct_persisted_sets(
    stream: &PreparedStream,
    table: &TornWriteTable,
    plan: &LocalPlan<'_>,
    segment_index: usize,
    enumeration_limit: u64,
) -> (u64, bool) {
    if !is_sampled_segment(stream.name, segment_index)
        || plan.expanded_state_count > enumeration_limit
    {
        return (plan.expanded_state_count, false);
    }
    let mut positions: Vec<usize> = stream.segments[segment_index].clone();
    for write_index in &stream.segments[segment_index] {
        if let Some((torn_index, replays)) = table.torn_image_by_write.get(write_index) {
            positions.push(*torn_index);
            positions.extend(
                replays
                    .iter()
                    .map(|(replay_index, _replayed)| *replay_index),
            );
        }
    }
    let code_of = |persisted: &[bool]| -> u64 {
        positions
            .iter()
            .enumerate()
            .filter(|(_bit, position)| persisted[**position])
            .map(|(bit, _position)| 1u64 << bit)
            .sum()
    };
    if positions.len() <= 32 {
        let mut seen = vec![0u64; (1usize << positions.len()).div_ceil(64)];
        let mut distinct = 0u64;
        for ordinal in 0..plan.expanded_state_count {
            let code = usize::try_from(code_of(&plan.persisted_of(ordinal))).expect("至多 32 位");
            let (word, bit) = (code / 64, code % 64);
            if seen[word] & (1 << bit) == 0 {
                seen[word] |= 1 << bit;
                distinct += 1;
            }
        }
        (distinct, true)
    } else {
        let distinct: BTreeSet<Vec<bool>> = (0..plan.expanded_state_count)
            .map(|ordinal| plan.persisted_of(ordinal))
            .collect();
        (u64::try_from(distinct.len()).expect("个数"), true)
    }
}

/// A1：逐段只展开那一段，装置计划逐个数（取样段里持久集合真生成、不同即数）、crates 算、公式算。
pub(super) fn segment_count_checks(
    stream: &PreparedStream,
    table: &TornWriteTable,
) -> Vec<SegmentCountCheck> {
    segment_count_checks_up_to(stream, table, u64::MAX)
}

/// 同上，一段的状态数超过 `enumeration_limit` 时不逐个数、按组合数报（单测在调试构建下用小的上限）。
pub(super) fn segment_count_checks_up_to(
    stream: &PreparedStream,
    table: &TornWriteTable,
    enumeration_limit: u64,
) -> Vec<SegmentCountCheck> {
    (0..stream.segments.len())
        .map(|segment_index| {
            let expansions: Vec<LocalExpansion> = (0..stream.segments.len())
                .map(|index| {
                    if index == segment_index {
                        LocalExpansion::EveryCombination
                    } else {
                        LocalExpansion::Skipped
                    }
                })
                .collect();
            let plan = LocalPlan::new(stream, table, expansions);
            let (counted_by_the_plan, _enumerated) = count_distinct_persisted_sets(
                stream,
                table,
                &plan,
                segment_index,
                enumeration_limit,
            );
            let crates_count =
                singlefs_verification::crash::crash_state_count_with_torn_in_place_overwrites(
                    &stream.base,
                    &stream.writes,
                    &stream.segments,
                    &|index, _segment| {
                        if index == segment_index {
                            CrashPointReplaySegmentExpansion::EveryProperSubset
                        } else {
                            CrashPointReplaySegmentExpansion::NotExpanded
                        }
                    },
                ) - 1;
            let segment = &stream.segments[segment_index];
            let in_place_overwrites = u64::try_from(
                segment
                    .iter()
                    .filter(|write_index| table.is_in_place_overwrite[**write_index])
                    .count(),
            )
            .expect("个数");
            SegmentCountCheck {
                segment_index,
                writes: segment.len(),
                in_place_overwrites,
                counted_by_the_plan,
                counted_by_the_crates: crates_count,
                counted_by_the_formula: formula_segment_state_count(
                    u64::try_from(segment.len()).expect("写数"),
                    in_place_overwrites,
                ),
            }
        })
        .collect()
}

/// crates 那一遍枚举的判段这一趟没跑、交不回整份结论时，S3、S4 那一格报的本次未跑（不 panic、不判红，也不算过）：
/// 多机配置开了 GPU 而这份二进制没编进 `gpu` 特性（判段不跑、只录，`crash_amplification::judge_not_run_without_the_gpu_feature`），
/// 或设了只录那一段、两台分片里只判了自己那一份。装置不带 `gpu` 特性也编得过、跑得起（`Cargo.toml` 不登 `required-features`：
/// 登了之后不带特性的 `cargo build`、clippy 与门禁都不再编它、不跑它的内联单测，复跑登记的起法也跟着起不来）；要这一格的结论，
/// 配置开了 GPU 时编进 `gpu` 特性起（`cargo run … --features gpu`）。
pub(super) const CRATES_COMPARISON_NOT_RUN_EXIT_CODE: i32 = 77;

/// S3、S4 在一个取样域上的结论：判了（两条过没过），或 crates 那一遍枚举的判段这一趟没跑（[`CRATES_COMPARISON_NOT_RUN_EXIT_CODE`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CratesComparisonClauses {
    Judged {
        enumeration_clause_holds: bool,
        tally_clause_holds: bool,
    },
    JudgeStageNotRun,
}

/// 几个取样域上的 S3、S4 并起来：判了的那几格各自取与，有一格判段没跑就记下来（整个模式照本次未跑收尾）。
pub(super) struct CratesComparisonVerdicts {
    pub(super) enumeration_clause_holds: bool,
    pub(super) tally_clause_holds: bool,
    pub(super) judge_stage_not_run: bool,
}

impl CratesComparisonVerdicts {
    pub(super) fn new() -> Self {
        Self {
            enumeration_clause_holds: true,
            tally_clause_holds: true,
            judge_stage_not_run: false,
        }
    }

    pub(super) fn absorb(&mut self, clauses: CratesComparisonClauses) {
        match clauses {
            CratesComparisonClauses::Judged {
                enumeration_clause_holds,
                tally_clause_holds,
            } => {
                self.enumeration_clause_holds &= enumeration_clause_holds;
                self.tally_clause_holds &= tally_clause_holds;
            }
            CratesComparisonClauses::JudgeStageNotRun => self.judge_stage_not_run = true,
        }
    }
}

/// S3 与 S4 在一个取样域上的结果：crates 枚举交给观察者的写表与持久集合逐个与装置比，crates 的计数交回来给 S4 比。
pub(super) struct CratesComparison {
    pub(super) observed_states: u64,
    pub(super) compared_states: u64,
    pub(super) writes_table_differs: bool,
    pub(super) persisted_mismatches: u64,
    pub(super) crates_tally: CrashPointReplayTally,
}

/// S3 那个观察者另读的输入的摘要（进它的版本；取样域、装置的写表或计划一变它就变，看过的块不按旧的复用）：取样域的每个序号与装置计划在它上面的
/// 持久集合，装置那一张写表逐个写的盘、种类、FUA、偏移与内容（整段清零只记长度）。
fn crates_comparison_observer_extra_inputs_digest(
    table: &TornWriteTable,
    plan: &LocalPlan<'_>,
    ordinals: &[u64],
) -> [u8; singlefs_verification::sha256::DIGEST_BYTES] {
    let mut bytes: Vec<u8> = Vec::new();
    for ordinal in ordinals {
        bytes.extend_from_slice(format!("ordinal={ordinal} persisted=").as_bytes());
        bytes.extend(
            plan.persisted_of(*ordinal)
                .iter()
                .map(|is_persisted| u8::from(*is_persisted)),
        );
        bytes.push(b'\n');
    }
    for write in &table.writes {
        bytes.extend_from_slice(
            format!(
                "write device={:?} kind={:?} force_unit_access={} offset={:?} ",
                write.device, write.kind, write.is_force_unit_access, write.offset
            )
            .as_bytes(),
        );
        match &write.contents {
            WrittenContents::Bytes(contents) => {
                bytes.extend_from_slice(format!("bytes={} ", contents.len()).as_bytes());
                bytes.extend_from_slice(contents);
            }
            WrittenContents::Zeros { length } => {
                bytes.extend_from_slice(format!("zeros={length}").as_bytes());
            }
        }
        bytes.push(b'\n');
    }
    singlefs_verification::sha256::sha256_digest(&bytes)
}

/// 在 `ordinals`（装置计划的序号，按次序）那个域上跑一遍 crates 的枚举（只展开 `expansion_of` 给的那几段），逐状态核 S3，交回 crates 的计数。
/// crates 最后多出的全部持久那一个（域里不含它时）不比。判段这一趟没跑、交不回整份结论时交 `None`（[`CRATES_COMPARISON_NOT_RUN_EXIT_CODE`]）。
pub(super) fn compare_with_the_crates(
    stream: &PreparedStream,
    table: &TornWriteTable,
    plan: &LocalPlan<'_>,
    ordinals: &[u64],
    expansion_of: &dyn Fn(usize) -> CrashPointReplaySegmentExpansion,
) -> Option<CratesComparison> {
    // 观察者的代码是具名的 fn（observe_the_crates_comparison）：它那一组判法单元按这个入口 fn 收闭包、登记在
    // 实验包的 `singlefs_implementation_experiments::judging_units_entries::OBSERVER_ENTRIES`，这里只把装置的写表、计划与取样域交给它；
    // 那一组的键从实验包编进来的那一张取（`OBSERVER_KEYS_OF_THE_EXPERIMENTS`）
    let observe =
        |ordinal: u64,
         image: &CrashImage<'_>,
         _report: &RecoveryReport,
         counts: &mut singlefs_verification::crash::CrashPointReplayObserverCounts| {
            observe_the_crates_comparison(table, plan, ordinals, ordinal, image, counts);
        };
    let crates_tally =
        singlefs_verification::crash_state_enumeration::enumerate_crash_states_with_expansion_keying_the_observers_in(
            singlefs_implementation_experiments::OBSERVER_KEYS_OF_THE_EXPERIMENTS,
            singlefs_verification::crash_state_enumeration::CrashPointReplayItem {
                name: "e161_crates_comparison",
                setting_file:
                    "crates/singlefs-implementation-experiments/src/bin/e161_crash_state_dedup_and_time_split.rs",
            },
            &stream.base,
            &stream.writes,
            &stream.segments,
            stream.judged_root_index,
            singlefs_verification::crash::CrashPointReplayVersionTable::with_a_file_only(
                &stream.versions,
            ),
            &|segment_index, _segment| expansion_of(segment_index),
            &[
                singlefs_verification::crash_state_enumeration::CrashPointReplayObserver {
                    name: "e161_crates_comparison",
                    extra_inputs_digest: crates_comparison_observer_extra_inputs_digest(
                        table, plan, ordinals,
                    ),
                    observe: &observe,
                },
            ],
        )
        .judged()?;
    let counts = &crates_tally.observer_counts;
    Some(CratesComparison {
        observed_states: counts.count_named("observed_states"),
        compared_states: counts.count_named("compared_states"),
        writes_table_differs: counts.count_named("writes_table_differs") > 0,
        persisted_mismatches: counts.count_named("persisted_mismatches"),
        crates_tally,
    })
}

/// 装置这一格的计数加上（覆盖写固定脚本流 S4 要比的）crates 多跑的全部持久那一个的照跑计数。
pub(super) fn tally_with_every_write_persisted(
    context: &EvaluationContext<'_>,
    tally: &CrashPointReplayTally,
) -> CrashPointReplayTally {
    let mut combined = CrashPointReplayTally::default();
    add_tally(&mut combined, tally);
    if !includes_every_write_persisted(context.stream.name) {
        let every_write = LocalPlan::new(
            context.stream,
            context.table,
            vec![LocalExpansion::Skipped; context.stream.segments.len()],
        );
        let image = CrashImage {
            base: &context.stream.base,
            writes: &context.table.writes,
            persisted: every_write.persisted_of(0),
        };
        let mut timings = StateTimings::default();
        let _reports = evaluate_baseline(
            context,
            &image,
            TimingControl::WALL_CLOCK_WITHOUT_BUSY_WAIT,
            &mut timings,
            &mut combined,
        );
    }
    combined
}
