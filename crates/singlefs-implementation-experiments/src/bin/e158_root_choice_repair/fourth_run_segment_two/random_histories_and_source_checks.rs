//! 第二段要跑的历史（harness 缺省种子那一组、最短复现、收尾第 43 行、按种子的随机历史、实七与实八的复现）与它们的来源核对。

use super::*;

/// 产物里这一段各族的几何名：几何是被复现的那段历史的一部分（登记 5.3：随机历史默认批的 4 GiB 盘与 harness 的几何），S6 对这几族不适用。
pub(super) const HARNESS_GEOMETRY_NAME: &str = "harness-4gib";

/// 随机历史默认批的段数与每段步数（登记 5.6：段数抄自 `crates/singlefs-verification/tests/random_histories.rs` 的 `DEFAULT_BATCH_SEEDS`，
/// 私有常量引不到；装置不读验证包的源码，段数与那份源码文本的回比由 `scripts/` 下的对比脚本做；每段步数是
/// `singlefs_verification::history::RANDOM_HISTORY_DEFAULT_SAMPLING` 的，开跑时按常量回比；现查的值记在第十二节）。
const DEFAULT_BATCH_SEEDS_LOCAL: u64 = 96;
const DEFAULT_BATCH_OPERATIONS_PER_HISTORY_LOCAL: usize = 30;

/// H-随 (a) 的 28 个种子偏移（登记 5.6、7.2：调查员报告 `research/prompts/m2-investigate-gate74-reds-report.md`「新发现 CheckerViolations { invariants: ["I-7.4"] }：第一个种子」那一行的同签名种子各减种子基）。
const RANDOM_HISTORY_RED_SEED_OFFSETS: [u64; 28] = [
    2, 6, 9, 11, 15, 16, 18, 21, 23, 24, 29, 31, 35, 52, 54, 55, 58, 65, 70, 71, 76, 79, 81, 84,
    85, 86, 90, 92,
];
/// 7.2：H-随 (a) 最小、最大的种子（登记第 7.2 节那一行的两个数）。
const RANDOM_HISTORY_RED_SEED_MINIMUM_REGISTERED: u64 = 7_463_871_032_432_355_115;
const RANDOM_HISTORY_RED_SEED_MAXIMUM_REGISTERED: u64 = 7_463_871_032_432_355_205;
/// 7.2：H-随 的段数 28 + 1 + 1（登记第 7.2 节那一行）。
pub(super) const RANDOM_HISTORY_SEGMENTS_REGISTERED: usize = 30;

/// 实八：崩溃注入种子基 + 2、24 步、4 个崩溃状态（r3 登记「5.3　历史族」那张表「实八」一行；
/// 步数与抽法同崩溃注入默认批 `crates/singlefs-verification/tests/crash_injection_campaign.rs` 的 `DEFAULT_BATCH_OPERATIONS_PER_HISTORY`、`DEFAULT_BATCH_DRAW`；
/// 与那两行文本的回比由 `scripts/` 下的对比脚本做，装置不读那份源码）。
const EIGHTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET: u64 = 2;
const EIGHTH_BATCH_CRASH_REPRODUCTION_OPERATIONS: usize = 24;
pub(super) const EIGHTH_BATCH_CRASH_REPRODUCTION_CRASH_POINTS: usize = 4;

/// S5 的签名（r3 登记第 604 行）：实七-甲 重开走到 (1, 3) 并报 `MappingStillUnreadable { slot: 50240 }`；实七-乙 在崩溃状态上读回同一个错；
/// 实八 (1, 4) 那次发布的单元在它还在根环里时被复用。
pub(super) const SEVENTH_BATCH_SIGNATURE_SLOT: u64 = 50240;
pub(super) const SEVENTH_BATCH_SIGNATURE_ROOT: TimelineRoot = (1, 3);
pub(super) const EIGHTH_BATCH_SIGNATURE_ROOT: TimelineRoot = (1, 4);

/// PC-随 描述今天那一臂的三句（登记 5.4 PC-随 [今]，F19）：checker 第一次判 I-7.4 被抛弃根那一半红的那一步与点名的根。
/// (a) 种子基 + 2 那一段：Operation(4)、(实例 1, txg 4)；(b) 最短复现：Operation(2)；(c) 第 43 行那条：Operation(3)、(实例 1, txg 5)。
pub(super) const POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SEED_PLUS_TWO: (usize, TimelineRoot) =
    (4, (1, 4));
pub(super) const POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SHORTEST: usize = 2;
pub(super) const POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_ROW_FORTY_THREE: (usize, TimelineRoot) =
    (3, (1, 5));

/// harness 执行器撞到判红就停（它的缺省）。登记 5.3 要这一族把它关掉、照原操作序列走完；变异 M22 把它打开。
pub(super) const RANDOM_HISTORIES_STOP_AT_THE_FIRST_CHECKER_RED: bool = false;

/// 这一段跑的历史从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SegmentTwoFamily {
    /// H-随：调查员点名的 28 段、最短复现、收口表第 43 行那条（登记 5.3）。
    RandomRed,
    /// H-随全：随机历史默认批其余 68 段，只进 Q4、Q5（登记 5.5 主 agent 认定第 9 项）。
    RandomAll,
    /// 实七-甲：故障注入 512 段那一批种子基 + 110 那段历史，harness 摆的 6 个注入点各一个分支。
    SeventhFault,
    /// 实七-乙：崩溃注入默认批种子基 + 16 那段历史，harness 摆的 4 个崩溃状态各一个分支。
    SeventhCrash,
    /// 实八：崩溃注入种子基 + 2 那段历史，同实七-乙的做法。
    EighthCrash,
}

impl SegmentTwoFamily {
    pub(super) fn name(self) -> &'static str {
        match self {
            SegmentTwoFamily::RandomRed => "h-random",
            SegmentTwoFamily::RandomAll => "h-random-all",
            SegmentTwoFamily::SeventhFault => "seventh-fault",
            SegmentTwoFamily::SeventhCrash => "seventh-crash",
            SegmentTwoFamily::EighthCrash => "eighth-crash",
        }
    }

    /// 这一族的丢写进不进出局（H-随全 只进 Q4、Q5，认定第 9 项）。
    pub(super) fn measures_lost_writes(self) -> bool {
        match self {
            SegmentTwoFamily::RandomRed
            | SegmentTwoFamily::SeventhFault
            | SegmentTwoFamily::SeventhCrash
            | SegmentTwoFamily::EighthCrash => true,
            SegmentTwoFamily::RandomAll => false,
        }
    }

    /// 每一步之后跑不跑池级 checker（Q9 与 PC-随 要 H-随 的；实七、实八的主路径给 S5 与对照；H-随全 只要挂载）。
    pub(super) fn checks_every_step(self) -> bool {
        match self {
            SegmentTwoFamily::RandomRed
            | SegmentTwoFamily::SeventhFault
            | SegmentTwoFamily::SeventhCrash
            | SegmentTwoFamily::EighthCrash => true,
            SegmentTwoFamily::RandomAll => false,
        }
    }

    /// 历史走完之后接不接尾巴（关闭 C → 可写挂载 → 冷启动读回 → ③c；登记 5.3 H-随，实七、实八照同一条，第十二节第二段修订）。
    pub(super) fn runs_the_tail(self) -> bool {
        self.measures_lost_writes()
    }
}

/// 一段历史：族、在族里的名字（种子偏移或写死的那两条）、操作序列。
#[derive(Clone, Debug)]
pub(super) struct SegmentTwoHistory {
    pub(super) family: SegmentTwoFamily,
    pub(super) label: String,
    pub(super) history: GeneratedHistory,
}

fn default_batch_history(offset: u64) -> GeneratedHistory {
    generate_history_with_weights(
        HistorySeed(SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + offset),
        DEFAULT_BATCH_OPERATIONS_PER_HISTORY_LOCAL,
        &GenerationWeights::BROAD,
    )
}

fn empty_content() -> ContentChoice {
    ContentChoice {
        length: ContentLength::Empty,
        fill_seed: 0,
    }
}

/// H-随 (b)：种子基 + 2 那一段收缩出来的 3 步（登记 5.3、5.6；调查员报告第 298–301 行）。
pub(super) fn shortest_reproduction_history() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + 2),
        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
        operations: vec![
            HistoryOperation::CloseAndMountWritable,
            HistoryOperation::PublishFirstFile(empty_content()),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
        ],
    }
}

/// H-随 (c)：收口表第 43 行那条用例的写死历史（登记 5.6；随机历史测试第 1082 行那条用例的 `operations`）。
pub(super) fn closeout_row_forty_three_history() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            HistoryOperation::PublishOverwrite(empty_content()),
            HistoryOperation::PublishOverwrite(empty_content()),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            HistoryOperation::PublishOverwrite(empty_content()),
            HistoryOperation::PublishOverwrite(empty_content()),
            HistoryOperation::PublishOverwrite(empty_content()),
            HistoryOperation::PublishOverwrite(empty_content()),
            HistoryOperation::RaiseRollbackFloor(FloorTargetChoice {
                steps_above_current_floor: 5,
            }),
        ],
    }
}

pub(super) const SHORTEST_REPRODUCTION_LABEL: &str = "shortest";
pub(super) const ROW_FORTY_THREE_LABEL: &str = "row43";

fn seed_label(offset: u64) -> String {
    format!("base+{offset}")
}

/// H-随 的 30 段：28 段种子、最短复现、第 43 行那条（次序照登记 5.3）。
pub(super) fn random_red_histories() -> Vec<SegmentTwoHistory> {
    let mut histories: Vec<SegmentTwoHistory> = RANDOM_HISTORY_RED_SEED_OFFSETS
        .iter()
        .map(|offset| SegmentTwoHistory {
            family: SegmentTwoFamily::RandomRed,
            label: seed_label(*offset),
            history: default_batch_history(*offset),
        })
        .collect();
    histories.push(SegmentTwoHistory {
        family: SegmentTwoFamily::RandomRed,
        label: SHORTEST_REPRODUCTION_LABEL.to_string(),
        history: shortest_reproduction_history(),
    });
    histories.push(SegmentTwoHistory {
        family: SegmentTwoFamily::RandomRed,
        label: ROW_FORTY_THREE_LABEL.to_string(),
        history: closeout_row_forty_three_history(),
    });
    histories
}

/// H-随全 的 68 段：随机历史默认批 96 段里 H-随 没取的那几段。
pub(super) fn random_all_histories() -> Vec<SegmentTwoHistory> {
    (0..DEFAULT_BATCH_SEEDS_LOCAL)
        .filter(|offset| !RANDOM_HISTORY_RED_SEED_OFFSETS.contains(offset))
        .map(|offset| SegmentTwoHistory {
            family: SegmentTwoFamily::RandomAll,
            label: seed_label(offset),
            history: default_batch_history(offset),
        })
        .collect()
}

pub(super) fn seventh_fault_history() -> SegmentTwoHistory {
    SegmentTwoHistory {
        family: SegmentTwoFamily::SeventhFault,
        label: seed_label(SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET),
        history: generate_history_with_weights(
            HistorySeed(
                SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + SEVENTH_BATCH_FAULT_REPRODUCTION_SEED_OFFSET,
            ),
            SEVENTH_BATCH_FAULT_REPRODUCTION_OPERATIONS,
            &GenerationWeights::BROAD,
        ),
    }
}

pub(super) fn seventh_crash_history() -> SegmentTwoHistory {
    SegmentTwoHistory {
        family: SegmentTwoFamily::SeventhCrash,
        label: seed_label(SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET),
        history: generate_history_with_weights(
            HistorySeed(
                SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + SEVENTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET,
            ),
            SEVENTH_BATCH_CRASH_REPRODUCTION_OPERATIONS,
            &GenerationWeights::BROAD,
        ),
    }
}

pub(super) fn eighth_crash_history() -> SegmentTwoHistory {
    SegmentTwoHistory {
        family: SegmentTwoFamily::EighthCrash,
        label: seed_label(EIGHTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET),
        history: generate_history_with_weights(
            HistorySeed(
                SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + EIGHTH_BATCH_CRASH_REPRODUCTION_SEED_OFFSET,
            ),
            EIGHTH_BATCH_CRASH_REPRODUCTION_OPERATIONS,
            &GenerationWeights::BROAD,
        ),
    }
}

pub(super) fn harness_execution(per_step_checker: PerStepChecker) -> HistoryExecution {
    HistoryExecution {
        per_step_checker,
        device_width: HistoryDeviceWidth::FourGibibytes,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

// ------------------------------------------------------------------------
// 13.1 开跑检查（登记 5.6 的 H-随 常量回比、7.2 的 H-随 锚点、11.2 S9 的前两句）。
//      调查员报告按文本读：报告不在副本里，路径由环境变量给（缺省是仓根下的相对路径，`replay.sh` 在仓根跑）。
//      验证包两份测试源码（随机历史、崩溃注入）不读：S9 (c) 第 43 行那条用例逐项比、5.6 默认批常量与源码文本回比由 `scripts/` 下的对比脚本做。
// ------------------------------------------------------------------------

const INVESTIGATOR_REPORT_DEFAULT_PATH: &str =
    "research/prompts/m2-investigate-gate74-reds-report.md";
const INVESTIGATOR_REPORT_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_E158_INVESTIGATOR_REPORT";
/// 调查员报告里种子那一行与最短复现那几行（登记 5.6：第 61 行、第 298–301 行）。
const INVESTIGATOR_REPORT_SEED_LINE: usize = 61;
const INVESTIGATOR_REPORT_SHORTEST_FIRST_LINE: usize = 298;

pub(super) fn investigator_report_text() -> Result<String, String> {
    let path = env::var(INVESTIGATOR_REPORT_ENVIRONMENT_VARIABLE)
        .unwrap_or_else(|_| INVESTIGATOR_REPORT_DEFAULT_PATH.to_string());
    std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))
}

/// 报告第 61 行里 `[` 与 `]` 之间的种子（逗号分隔）。
fn seeds_listed_on_the_line(line: &str) -> Vec<u64> {
    let Some(start) = line.find("同签名的种子 [") else {
        return Vec::new();
    };
    let rest = &line[start + "同签名的种子 [".len()..];
    let Some(end) = rest.find(']') else {
        return Vec::new();
    };
    rest[..end]
        .split(',')
        .filter_map(|seed| seed.trim().parse().ok())
        .collect()
}

/// 一行开检查的结果：名字、判定、细节。判定 fail 的那一条让它管的那一族停（S9 / V3）。
pub(super) struct SegmentTwoCheck {
    pub(super) item: String,
    pub(super) passed: bool,
    pub(super) detail: String,
}

fn segment_two_check(item: &str, passed: bool, detail: String) -> SegmentTwoCheck {
    SegmentTwoCheck {
        item: item.to_string(),
        passed,
        detail,
    }
}

/// S9 前两句与 5.6、7.2 的 H-随 那几行：种子、最短复现、默认批每段步数。交回每一项，失败的那几项让 H-随 停。
/// S9 (c) 第 43 行那条用例与随机历史测试源码逐项比由 `scripts/` 下的对比脚本做，装置不读验证包的源码。
pub(super) fn random_history_source_checks(
    report: &Result<String, String>,
) -> Vec<SegmentTwoCheck> {
    let mut checks = Vec::new();
    let seeds: Vec<u64> = RANDOM_HISTORY_RED_SEED_OFFSETS
        .iter()
        .map(|offset| SEED_BASE_OF_THIS_TEST_CYCLE_LOCAL + offset)
        .collect();
    checks.push(segment_two_check(
        "7.2 H-随 (a) 种子个数、最小、最大",
        seeds.len() == RANDOM_HISTORY_RED_SEED_OFFSETS.len()
            && seeds.iter().min() == Some(&RANDOM_HISTORY_RED_SEED_MINIMUM_REGISTERED)
            && seeds.iter().max() == Some(&RANDOM_HISTORY_RED_SEED_MAXIMUM_REGISTERED),
        format!(
            "count={} minimum={:?} maximum={:?}",
            seeds.len(),
            seeds.iter().min(),
            seeds.iter().max()
        ),
    ));
    checks.push(segment_two_check(
        "7.2 H-随 段数 28 + 1 + 1",
        random_red_histories().len() == RANDOM_HISTORY_SEGMENTS_REGISTERED,
        format!("segments={}", random_red_histories().len()),
    ));
    match report {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            let seed_line = lines
                .get(INVESTIGATOR_REPORT_SEED_LINE - 1)
                .copied()
                .unwrap_or("");
            let listed = seeds_listed_on_the_line(seed_line);
            checks.push(segment_two_check(
                "S9 (a) 种子与调查员报告第 61 行逐个相等",
                listed == seeds,
                format!("listed={} local={}", listed.len(), seeds.len()),
            ));
            let shortest = shortest_reproduction_history();
            let first = INVESTIGATOR_REPORT_SHORTEST_FIRST_LINE - 1;
            let header_holds = lines
                .get(first)
                .is_some_and(|line| line.contains("最短复现：起点 AfterMakeFilesystem，3 步"))
                && shortest.starting_point == HistoryStartingPoint::AfterMakeFilesystem
                && shortest.operations.len() == 3;
            let steps_hold = shortest
                .operations
                .iter()
                .enumerate()
                .all(|(index, operation)| {
                    lines.get(first + 1 + index).is_some_and(|line| {
                        line.trim_start()
                            .starts_with(&format!("{index}. {operation:?} →"))
                    })
                });
            checks.push(segment_two_check(
                "S9 (b) 最短复现 3 步与调查员报告第 298–301 行逐项相等",
                header_holds && steps_hold,
                format!("header={header_holds} steps={steps_hold}"),
            ));
        }
        Err(error) => {
            checks.push(segment_two_check(
                "S9 (a)(b) 调查员报告读得出",
                false,
                format!("{error:?}"),
            ));
        }
    }
    checks.push(default_batch_operations_per_history_check());
    checks
}

/// 5.6：随机历史默认批每段步数按 `RANDOM_HISTORY_DEFAULT_SAMPLING` 回比。默认批首种子、段数与崩溃注入默认批步数、抽法写在验证包两份测试源码的私有常量里，
/// 装置不读那两份源码，与它们文本的回比由 `scripts/` 下的对比脚本做。不等的照本地常量跑、写进第十二节（登记 5.6 那一行），这一项只报不停。
fn default_batch_operations_per_history_check() -> SegmentTwoCheck {
    let library_operations =
        singlefs_verification::history::RANDOM_HISTORY_DEFAULT_SAMPLING.operations_per_history;
    segment_two_check(
        "5.6 随机历史默认批每段步数（与验证包库常量回比；不等只报，照本地常量跑）",
        library_operations == DEFAULT_BATCH_OPERATIONS_PER_HISTORY_LOCAL,
        format!(
            "operations={DEFAULT_BATCH_OPERATIONS_PER_HISTORY_LOCAL} library_operations={library_operations}"
        ),
    )
}

#[cfg(test)]
mod tests;
