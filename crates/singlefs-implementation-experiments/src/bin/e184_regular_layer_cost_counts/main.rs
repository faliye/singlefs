//! E184（常规层与大负载层的代价计数）全部子命令（跑前登记 `research/prompts/e184-preregistration.md` 5.3 第二份 bin）。
//!
// admission: always 每跑一次都在今天这份 crates 上重排计划、重取块键、重录固定流，产物里的挂钟每次都是新观测；已有写完的转储与块键文件由各子命令自己整份跳过
// run-condition: command python3
//!
//! 入库装置（登记「装置写在哪」）：读主工作区现状的 `crates/`，块键一律调 crates 的函数算（`crash_points_of_the_steps` 切节点、
//! `plan_crash_points` 排计划、`PlannedCrashPoint::block_key` 取块键），不另写一份。
//! 转储的写法与读法只有 `e184_regular_layer_cost_counts_write_dump/mod.rs` 那一份（转储 bin `e184_regular_layer_cost_counts_write_dump.rs` 也引它），这里经 `#[path]` 当模块引进来。
//!
//! 子命令：
//!   selfcheck                                     停机 T1、T3、T4 与第七节锚点逐条核（`name=stop_check`、`name=anchor`）
//!   record-fixed-chains --out <目录>              覆盖写链三档与冒烟层四条流各写一份转储
//!   keys --dump <目录>… --out <目录> [--controls] 逐份转储排计划、取块键，写块键文件，打 `name=flow` 与阳性对照
//!   share --candidates <目录>… --against <目录>… --out <文件> [--pinned-smoke <文件>]   共用块键与态数区间重叠
//!   library --keys <目录>…                         读 prune-server 里那一份崩溃判定库，数候选流的块键在录入表里的个数（附带；一行都不写）
//!   compare --before <目录> --after <目录> --kind <commit-pair|patch> --pair <名> --out <文件>   作废块数
//!   compose --j1 <文件> --keys <目录>… --j3 <文件> --j2-small <日志> --j2-medium <日志> --library-bytes <文件> --recorded-blocks <文件>   提交那一趟的挂钟
//!
//! 线程数取 `E184_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值（转储模块里那一份读法）。
//! 块数上界（总态数除每块态数向上取整，加每点一块的零头）超过 2^26 的展开不排计划、不取块键，`name=flow` 的 `blocks_full` / `blocks_in_place_subsets` 写 `not_planned`，
//! 态数照报（登记第十二节修订 10）；转储往返对照比块键的上限再小一档，2^24。
//! 产物是 `E7RESULT` 行。退出码：0 正常；2 命令行不对；3 停机条款没过（产物照打、交主 agent）；4 阳性对照没过（作废）；
//! 78 准入与运行条件没满足（`preflight` 拒绝，带 `--force` 才照跑；强制跑的那一次，标准输出第一行多一行 `name=preflight forced=…`，`emitted=` 不数它）。
//! `preflight` 的实现只有转储模块里那一份（`write_dump::preflight`），这里传自己的源文件路径。
//!
//! 照抄的段与停机 T2（登记第十一节）：冒烟层三条流的建法照抄各测试文件里私有的建流函数（`prepare_the_fixed_script_stream`、`prepare_the_multi_record_publish_stream`、`prepare`）和它调到的私有函数，装置里用 `COPIED-BEGIN` / `COPIED-END`
//! 两行标记框出每一段。装置不读验证包的测试文件：每一段去掉缩进之后在对应的测试文件原文里逐字找不找得到（停机 T2），与 `PINNED_BY_THE_TESTS`、
//! `OVERWRITE_AND_UNLINK_FULL_STATES_PINNED_BY_THE_TEST` 钉的数还在不在测试文件里，由 `scripts/` 下的检查脚本把这份源文件与验证包测试文件都当数据读、两边对比。
//! 出处（文件在 `crates/singlefs-verification/tests/` 下，起止行与各段 `COPIED-BEGIN` 标记里写的相同，是照抄时现读的；对比只按文件名找原文、不按行号）：
//!   fixed_script_head      crash_enumeration_fixed_script_stream.rs:55-92      第二份内容常量、`PreparedFixedScriptStream`、`Script`
//!   fixed_script_body      crash_enumeration_fixed_script_stream.rs:139-574    谱系常量到 `prepare_the_fixed_script_stream` 末尾（`overwrite`、`unmount_recorded`、`writes_and_segments_after_mkfs` 在内）
//!   multi_record_publish   crash_enumeration_multi_record_publish_stream.rs:62-152   `PreparedMultiRecordPublishStream` 到 `prepare_the_multi_record_publish_stream` 末尾
//!   new_pool_prepare_body  crash_enumeration_new_pool_file_creation_stream.rs:859-877   `prepare` 的前半（到被判的根槽写那一步）
//! 允许的替换只有一种：包进 `mod` 的缩进（每行前多 4 个空格）。`crash_points_of_the_steps` 是 `pub`，不抄，装置直接调
//! `singlefs_verification::crash_state_enumeration::crash_points_of_the_steps`。造池用实验包库的 `experiment_pool`；照抄的段调到的顺序写、
//! 恰好要几个数据单元的内容、多记录发布流的发布边界输入（`multi_record_publish`）在这份文件的 `multi_record_steps` 模块里，建快照流在
//! `snapshot_creation_flow` 模块里摆：都是这个装置自己写的，录出的段形与态数由停机 T3 拿测试文件钉的数回比（`selfcheck` 与冒烟层四条流的单测）。
//!
//! 决定工作量、几何或判据门槛的常量都是本地常量（值抄自登记位，注释写出处），`selfcheck` 与 crates 的同名常量回比（停机 T1）：
//! 每块状态数 65536、种子基、闭式能算的最长段 63。步数档、种子数、盘宽档、比重组、覆盖写链的单元数、补丁位、提交清单不从 `crates/` 算，在登记第五节写死。

use singlefs_implementation_experiments::experiment_pool;
#[allow(
    dead_code,
    reason = "转储模块是两份 bin 共用的：转储 bin 的子命令与随机历史的并行录制这个装置不调，只用转储格式、读回、录一条历史和线程数取法"
)]
#[path = "../e184_regular_layer_cost_counts_write_dump/mod.rs"]
mod write_dump;

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use singlefs_core::address::InstanceGeneration;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    in_place_subsets_with_copy_on_write_none_or_all_expansion, CrashPointReplaySegmentExpansion,
    StateCount1280Bits, TearableInPlaceOverwrites,
};
use singlefs_verification::crash_amplification::{plan_crash_points, CrashFlow, STATES_PER_BLOCK};
use singlefs_verification::crash_library::LibraryOpenInThisProcess;
use singlefs_verification::crash_state_enumeration::{
    crash_points_of_the_steps, CrashPointReplayItem,
};
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::memory_pool::{
    closed_form_state_count_of_write_index_segments, root_identity_written_by, writes_and_segments,
    MemoryPool, PublishedVersion, PublishedVersionWithoutAFile, RetainedWrite, WrittenContents,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::sha256::sha256_hexadecimal;
use singlefs_verification::verdict_store::{
    BlockStartStateIndex, EnumerationPlanHash, NodeInputDigest, VerdictBlockKey,
};
use singlefs_verification::RecordedOperationKind;

use write_dump::{
    Emitter, FlowDump, EXIT_CODE_OK, EXIT_CODE_POSITIVE_CONTROL_FAILED, EXIT_CODE_USAGE,
    SEED_BASE_OF_THE_RANDOM_HISTORIES,
};

mod command_line_and_statistics;
mod compare_command;
mod compose_command;
mod crash_point_plans_and_block_keys;
mod fixed_flows;
mod flow_lines;
mod keys_command;
mod keys_file_format;
mod library_command;
mod positive_controls;
mod reading_one_flow;
mod share_command;

use command_line_and_statistics::*;
use compare_command::*;
use compose_command::*;
use crash_point_plans_and_block_keys::*;
use fixed_flows::*;
use flow_lines::*;
use keys_command::*;
use keys_file_format::*;
use library_command::*;
use positive_controls::*;
use reading_one_flow::*;
use share_command::*;

// ───────────────────────────── 本地常量（停机 T1 回比） ─────────────────────────────

/// 每块状态数：值抄自 `crates/singlefs-verification/src/crash_amplification.rs` 的 `STATES_PER_BLOCK`（D13 已定项 16 没写块长，块长只在实现里）；
/// `selfcheck` 与 crates 那一份回比。
const LOCAL_STATES_PER_BLOCK: u64 = 65_536;
/// 闭式能算的最长段写数：值抄自 D13 已定项 16「整域装不进 u64」与 `memory_pool::closed_form_state_count_of_write_index_segments` 的注释（64 写的段 2^64 装不进 u64）；
/// `selfcheck` 核 crates 的闭式函数在 64 写的段上 panic。
const LONGEST_SEGMENT_WITH_A_CLOSED_FORM: usize = 63;
/// 登记 1.1 节「上千」读作写数不少于它。
const WRITES_OF_A_THOUSAND_WRITE_STREAM: usize = 1000;
/// 一条流排计划时块数上界的上限（登记第十二节修订 10）：2^26 块。`plan_blocks` 把每个点的块一次摆进内存（`PlannedCrashPoint::blocks` 交回一个
/// 按块数长的 `Vec<Range<u64>>`，每块 16 字节，再加每块一条 `BlockOfAPlan`），块数没上限时一个 56 写的段（覆盖写链 u = 16 每次覆盖写的单元段）
/// 就是 2^40 块、`Vec` 要 2^44 字节。这是工程上限，不是判据：超过它的流照报态数，只是不排计划、不取块键（和「装不进 u64」同一种待遇）。
const MOST_BLOCKS_PLANNED_PER_FLOW: u64 = 1 << 26;
/// 转储往返对照（`direct_base_control`）比块键时块数上界的上限：2^24 块。对照要同时摆两份计划，比 [`MOST_BLOCKS_PLANNED_PER_FLOW`] 再小一档；
/// 超过它的流只比基镜像。
const MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP: u64 = 1 << 24;

// ───────────────────────────── 判据的门槛（登记第六节，跑前写死） ─────────────────────────────

/// J1-乙：落在冒烟层态数区间里的占比不小于它记「重叠」。
const INSIDE_FRACTION_AT_LEAST_FOR_OVERLAP: f64 = 0.5;
/// J1-乙：不大于它记「分开」。
const INSIDE_FRACTION_AT_MOST_FOR_SEPARATE: f64 = 0.1;
/// J1-丙：族中位不小于它记「换名」。
const SHARE_MEDIAN_AT_LEAST_FOR_RENAMED: f64 = 0.5;
/// J1-丙：族最大不大于它记「独立」。
const SHARE_MAXIMUM_AT_MOST_FOR_INDEPENDENT: f64 = 0.05;
/// J3-乙：中位不小于它记「写路径提交大多作废过半」。
const INVALIDATED_MEDIAN_AT_LEAST_FOR_MORE_THAN_HALF: f64 = 0.5;
/// J3-乙：中位不大于它记「块级复用省下大半」。
const INVALIDATED_MEDIAN_AT_MOST_FOR_MOST_REUSED: f64 = 0.1;
/// J3-乙：算得出的提交对数不到它记「还差」（G2 的够判条件）。
const COMMIT_PAIRS_NEEDED_FOR_ENOUGH: usize = 10;
/// 挂钟档的界：2 小时取自当时 `scripts/hosts/detached-run.sh` 的 `--wall-clock within-2h / over-2h` 分界（该选项已删），24 小时是一天；两个界都不是从哪条臂的定义推出来的。
const SECONDS_OF_TWO_HOURS: f64 = 7200.0;
const SECONDS_OF_A_DAY: f64 = 86400.0;

/// 退出码（登记 5.3）：0 正常、2 命令行不对、4 阳性对照没过与转储 bin 同一份（`write_dump` 里的）；3 是这个装置自己的停机条款没过。
const EXIT_CODE_STOP_CLAUSE_FAILED: i32 = 3;

/// 节点的设点文件：这个装置自己的仓内路径（名字与路径都不进块键，登记 B3）。
const SETTING_FILE_OF_THE_FLOWS: &str =
    "crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts/main.rs";
/// 装置里排计划用的项名；换一个项名再排一遍是阳性对照「名字不进键」。
const ITEM_NAME_OF_THE_FLOWS: &str = "e184_flow";
const OTHER_ITEM_NAME_FOR_THE_NAME_CONTROL: &str = "e184_flow_under_another_name";
const OTHER_SETTING_FILE_FOR_THE_NAME_CONTROL: &str =
    "crates/singlefs-implementation-experiments/src/bin/e184_the_same_flow_under_another_file.rs";

/// 字段值：装不进 u64 的数写它，不报数（登记 1.1）。
const BEYOND_U64: &str = "beyond_u64";

// ───────────────────────────── 冒烟层与覆盖写链要的几步（装置自己写的） ─────────────────────────────

/// 多记录发布流与覆盖写链在新池新建文件之后要的几样：恰好要几个数据单元的内容、一次顺序写，连同照抄的多记录流建法要的发布边界输入
/// （`MultiRecordPublishBoundaryInputs` 与造它的 `multi_record_publish`）。顺序写把池的现行那一版换成写出的那一版（照抄的建法接着在它上面写）。
mod multi_record_steps;

/// 建快照流（A → S）按崩溃点重放要的样子摆好：`crash-case:snapshot-creation-stream`。
mod snapshot_creation_flow;

// ───────────────────────────── 照抄：冒烟层三条流的建法（测试文件里私有的建流函数） ─────────────────────────────
//
// 登记 1.1「冒烟层比较集 R4」：测试文件里私有的建流函数逐字照抄，抄的每一段由停机 T2 钉住（`scripts/` 下的检查脚本核，装置不核）。
// 每一段上下有 `COPIED-BEGIN` / `COPIED-END` 两行标记，标记里写出处（文件与起止行）；段里每一行去掉包进模块的 4 格缩进之后，
// 要在原文件里整段逐字找得到。允许的替换只有这一种：包进模块的缩进（每行前多 4 个空格）。
// 每个模块自己带 `#[rustfmt::skip]`，rustfmt 不会重排照抄的行。

/// 覆盖写固定脚本流（到 E 再正常卸载那一支，`Script::ReuseAfterRaisingFloorThenNormalUnmount`）：`crash-case:overwrite-fixed-script-stream`。
#[rustfmt::skip]
mod fixed_script_stream;

/// 多记录发布流（取号、暖机、A、B、C）：`crash-case:multi-record-publish-stream`。发布边界观察者另读的输入（`MultiRecordPublishBoundaryInputs`
/// 与造它的 `multi_record_publish`）连同顺序写与内容由这份文件的 `multi_record_steps` 模块给（装置自己写的，不抄）。
#[rustfmt::skip]
mod multi_record_publish_stream;

/// 新池新建文件流（取号、暖机、A）：`crash-case:new-pool-file-creation-stream`。`prepare` 的前半（到被判的根槽写那一步）逐字照抄，
/// 后半（`TransactionPublishInRecordedStream` 的自检）是那个测试文件观察者的东西，这个装置不用，不抄。
#[rustfmt::skip]
mod new_pool_file_creation_stream;

// ───────────────────────────── selfcheck：停机 T1、T3、T4 与第七节锚点 ─────────────────────────────

/// 这份源文件自己的文本：单测拿它核 [`OWN_SOURCE_PATH`] 指的就是编进来的这一份。
#[cfg(test)]
const OWN_SOURCE_TEXT: &str = include_str!("main.rs");

/// 这个 bin 的源文件路径（`write_dump::preflight` 判它头上的准入与运行条件）；写成 `concat!` 字面量，单元键收得到这份文件。
const OWN_SOURCE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/bin/e184_regular_layer_cost_counts/main.rs"
);

/// 这个 bin 在 `src/bin/` 下的文件名：单测拿它核 [`OWN_SOURCE_PATH`] 指的是这一份。
#[cfg(test)]
const OWN_SOURCE_FILE_NAME: &str = "e184_regular_layer_cost_counts/main.rs";

/// 建快照流的段形（测试文件 `SEGMENT_SIZES`，也是登记锚 A1 的输入）。取号写两轮之后开头多一个 2 写的系统配置槽段（取号写一轮时 15 段、62 写）。
const SNAPSHOT_CREATION_SEGMENT_SIZES: [usize; 16] =
    [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2];
/// 建快照流前 12 段（新池新建文件流的段形，锚 A2 的输入；取号写一轮时前 11 段、41 写）。
const NEW_POOL_SEGMENT_SIZES: [usize; 12] = [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
/// 多记录流的段形（测试文件 `prepare_the_multi_record_publish_stream` 里钉的；取号写一轮时 19 段、115 写）。
const MULTI_RECORD_SEGMENT_SIZES: [usize; 20] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2,
];

/// 测试文件钉的常量：两态闭式、全量态数，和有的话甲二态数、段形、段数、写数（`None` 是测试文件没钉）。
#[derive(Clone, Copy, Debug)]
struct PinnedByTheTest {
    label: &'static str,
    two_state: u64,
    full: u64,
    in_place_subsets: Option<u64>,
    segment_sizes: Option<&'static [usize]>,
    segment_count: Option<usize>,
    writes: Option<usize>,
}

/// 取号写两轮之后照四份测试文件现钉的数重抄（取号写一轮时依次是 16777240、16777260、46、41 写；1678377089、1678377204、278、78 段；
/// 1358954604、1358954634、115 写；16842782、16842807），`selfcheck` 现录的四条流与它们逐个相等。
const PINNED_BY_THE_TESTS: [PinnedByTheTest; 4] = [
    PinnedByTheTest {
        label: "smoke-new_pool_file_creation_stream",
        two_state: 16_777_243,
        full: 16_777_268,
        in_place_subsets: Some(54),
        segment_sizes: Some(&NEW_POOL_SEGMENT_SIZES),
        segment_count: None,
        writes: Some(43),
    },
    PinnedByTheTest {
        label: "smoke-overwrite_fixed_script_stream",
        two_state: 1_678_377_095,
        full: 1_678_377_220,
        in_place_subsets: Some(294),
        segment_sizes: None,
        segment_count: Some(80),
        writes: None,
    },
    PinnedByTheTest {
        label: "smoke-multi_record_publish_stream",
        two_state: 1_358_954_607,
        full: 1_358_954_642,
        in_place_subsets: None,
        segment_sizes: Some(&MULTI_RECORD_SEGMENT_SIZES),
        segment_count: None,
        writes: Some(117),
    },
    PinnedByTheTest {
        label: "smoke-snapshot_creation_stream",
        two_state: 16_842_785,
        full: 16_842_815,
        in_place_subsets: None,
        segment_sizes: Some(&SNAPSHOT_CREATION_SEGMENT_SIZES),
        segment_count: None,
        writes: None,
    },
];

/// D13 已定项 9 射程写的「并行线一那条流 12230590578 个状态」（锚 B1）。测试文件头说它是 C577 之前的数，预计与现录的多记录流不符。
const MULTI_RECORD_FULL_STATES_NAMED_BY_THE_CLAUSE: u64 = 12_230_590_578;

/// 一条检查的结果：停机条款（`stop_check`）或锚点（`anchor`）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckReading {
    item: String,
    passed: bool,
    detail: String,
}

impl CheckReading {
    fn new(item: &str, passed: bool, detail: String) -> Self {
        Self {
            item: item.to_string(),
            passed,
            detail,
        }
    }

    fn stop_check_body(&self) -> String {
        format!(
            "name=stop_check item={} result={} detail={}",
            self.item,
            if self.passed { "pass" } else { "fail" },
            self.detail.replace(' ', "_")
        )
    }
}

/// 停机 T1：文件头的本地常量与 crates 的同名常量回比。
fn local_constants_checks() -> Vec<CheckReading> {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let closed_form_of = |writes_in_the_segment: usize| {
        catch_unwind(|| {
            closed_form_state_count_of_write_index_segments(&[(0..writes_in_the_segment).collect()])
        })
    };
    let at_the_limit = closed_form_of(LONGEST_SEGMENT_WITH_A_CLOSED_FORM);
    let one_beyond = closed_form_of(LONGEST_SEGMENT_WITH_A_CLOSED_FORM + 1);
    std::panic::set_hook(previous_hook);
    vec![
        CheckReading::new(
            "local_states_per_block_matches_crates",
            LOCAL_STATES_PER_BLOCK == STATES_PER_BLOCK,
            format!("local={LOCAL_STATES_PER_BLOCK} crates={STATES_PER_BLOCK}"),
        ),
        CheckReading::new(
            "local_seed_base_matches_crates",
            SEED_BASE_OF_THE_RANDOM_HISTORIES == singlefs_verification::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            format!("local={SEED_BASE_OF_THE_RANDOM_HISTORIES} crates={}", singlefs_verification::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        ),
        CheckReading::new(
            "local_longest_closed_form_segment_matches_crates",
            at_the_limit.is_ok() && one_beyond.is_err() && two_state_closed_form(&[(0..=LONGEST_SEGMENT_WITH_A_CLOSED_FORM).collect()]).is_none(),
            format!(
                "limit={LONGEST_SEGMENT_WITH_A_CLOSED_FORM} crates_at_the_limit_panics={} crates_one_beyond_panics={}",
                at_the_limit.is_err(),
                one_beyond.is_err()
            ),
        ),
    ]
}

/// 每段写数的列表换成段（段里放 0..写数 的下标，闭式只看长度）。
fn segments_of_sizes(sizes: &[usize]) -> Vec<Vec<usize>> {
    sizes.iter().map(|size| (0..*size).collect()).collect()
}

/// 每个点的块数等于按本地块长算的块数（锚 A3 在真实计划上的形态）：一点 65537 态切 2 块、65536 态切 1 块靠 [`blocks_needed_for`]。
fn every_point_is_cut_into_the_blocks_the_local_block_length_gives(
    blocks: &[BlockOfAPlan],
) -> bool {
    let mut by_point: BTreeMap<usize, (u64, u64)> = BTreeMap::new();
    for block in blocks {
        let entry = by_point.entry(block.point_index).or_default();
        entry.0 += 1;
        entry.1 += block.states;
    }
    by_point
        .values()
        .all(|(block_count, states)| *block_count == blocks_needed_for(*states))
}

/// 一条冒烟层流重录的值与测试文件钉的常量回比（停机 T3、T4，锚 A5、A6、B2）。
fn recorded_flow_readings(prepared: &PreparedFlow, pinned: &PinnedByTheTest) -> Vec<CheckReading> {
    let dump = prepared.dump();
    let reading = match read_the_flow(&dump, &prepared.base) {
        Ok(reading) => reading,
        Err(message) => {
            return vec![CheckReading::new(
                &format!("recorded_values_match_the_test_pins_{}", prepared.label),
                false,
                message,
            )]
        }
    };
    let row = &reading.row;
    let mut mismatches = Vec::new();
    if pinned.label != prepared.label {
        mismatches.push(format!(
            "pinned_for_{}_but_recorded_{}",
            pinned.label, prepared.label
        ));
    }
    for (name, recorded, pin) in [
        ("two_state", row.two_state, Some(pinned.two_state)),
        ("full", row.full, Some(pinned.full)),
        (
            "in_place_subsets",
            row.in_place_subsets,
            pinned.in_place_subsets,
        ),
    ] {
        if let Some(pin) = pin.filter(|pin| Some(*pin) != recorded) {
            mismatches.push(format!("{name}_recorded={recorded:?}_pinned={pin}"));
        }
    }
    let sizes: Vec<usize> = dump.segments.iter().map(Vec::len).collect();
    if pinned
        .segment_sizes
        .is_some_and(|pin| pin != sizes.as_slice())
    {
        mismatches.push(format!("segment_sizes_recorded={sizes:?}"));
    }
    if pinned
        .segment_count
        .is_some_and(|count| count != sizes.len())
    {
        mismatches.push(format!("segment_count_recorded={}", sizes.len()));
    }
    if pinned
        .writes
        .is_some_and(|writes| writes != dump.writes.len())
    {
        mismatches.push(format!("writes_recorded={}", dump.writes.len()));
    }
    let full_plan = reading.full.as_ref();
    vec![
        CheckReading::new(
            &format!("recorded_values_match_the_test_pins_{}", prepared.label),
            mismatches.is_empty(),
            format!("mismatches=[{}]", mismatches.join(",")),
        ),
        CheckReading::new(
            &format!("two_counting_routes_agree_{}", prepared.label),
            reading.count_disagreements.is_empty(),
            format!("disagreements=[{}]", reading.count_disagreements.join(";")),
        ),
        CheckReading::new(
            &format!("two_state_not_above_full_{}", prepared.label),
            matches!((row.two_state, row.full), (Some(two_state), Some(full)) if two_state <= full),
            format!("two_state={:?} full={:?}", row.two_state, row.full),
        ),
        CheckReading::new(
            &format!(
                "blocks_per_point_follow_the_local_block_length_{}",
                prepared.label
            ),
            full_plan.is_some_and(|plan| {
                every_point_is_cut_into_the_blocks_the_local_block_length_gives(&plan.blocks)
            }),
            format!("blocks_full={:?}", row.blocks_full),
        ),
    ]
}

/// 锚点（登记 7.2 的 A1、A2、A4 与 7.1 的 B1）：一行一条，写预期、实测与对不对得上。
fn anchor_body(identifier: &str, expected: &str, actual: &str) -> (String, bool) {
    let matched = expected == actual;
    (
        format!("name=anchor identifier={identifier} expected={expected} actual={actual} match={matched}"),
        matched,
    )
}

/// `selfcheck`：停机 T1、T3、T4 与第七节锚点逐条核。有一条停机检查没过或 A 开头的锚点对不上退 3；B1 对不上是登记预计的（C577 改了段形），不在其内。
fn run_selfcheck_command(arguments: &[String]) -> i32 {
    if !arguments.is_empty() {
        return usage_error("selfcheck 不带参数");
    }
    let mut output = Output::new();
    let mut readings = local_constants_checks();
    let mut anchors = vec![
        anchor_body(
            "snapshot_creation_two_state",
            "16842785",
            &count_field(two_state_closed_form(&segments_of_sizes(
                &SNAPSHOT_CREATION_SEGMENT_SIZES,
            ))),
        ),
        anchor_body(
            "new_pool_two_state",
            "16777243",
            &count_field(two_state_closed_form(&segments_of_sizes(
                &NEW_POOL_SEGMENT_SIZES,
            ))),
        ),
        anchor_body(
            "new_pool_writes",
            "43",
            &NEW_POOL_SEGMENT_SIZES.iter().sum::<usize>().to_string(),
        ),
        anchor_body(
            "blocks_needed_at_and_one_state_beyond_the_states_per_block",
            "1,2",
            &format!(
                "{},{}",
                blocks_needed_for(LOCAL_STATES_PER_BLOCK),
                blocks_needed_for(LOCAL_STATES_PER_BLOCK + 1)
            ),
        ),
        anchor_body(
            "first_history_seed",
            &SEED_BASE_OF_THE_RANDOM_HISTORIES.to_string(),
            &write_dump::RandomHistoryRequest {
                steps: 1,
                weights: write_dump::WeightsChoice::Broad,
                width: singlefs_verification::history::HistoryDeviceWidth::FourGibibytes,
                seed_index: 0,
            }
            .seed()
            .to_string(),
        ),
    ];
    let mut multi_record_full = None;
    for (kind, pinned) in [
        (FixedFlowKind::NewPoolFileCreation, &PINNED_BY_THE_TESTS[0]),
        (FixedFlowKind::FixedScript, &PINNED_BY_THE_TESTS[1]),
        (FixedFlowKind::MultiRecordPublish, &PINNED_BY_THE_TESTS[2]),
        (FixedFlowKind::SnapshotCreation, &PINNED_BY_THE_TESTS[3]),
    ] {
        let label = kind.label();
        match catch_unwind(AssertUnwindSafe(|| kind.prepare_the_flow().0)) {
            Err(payload) => readings.push(CheckReading::new(
                &format!("recorded_values_match_the_test_pins_{label}"),
                false,
                format!("建流时 panic：{}", panic_text_of(payload.as_ref())),
            )),
            Ok(prepared) => {
                let recorded = catch_unwind(AssertUnwindSafe(|| {
                    recorded_flow_readings(&prepared, pinned)
                }));
                match recorded {
                    Err(payload) => readings.push(CheckReading::new(
                        &format!("two_counting_routes_agree_{label}"),
                        false,
                        format!("排计划时 panic：{}", panic_text_of(payload.as_ref())),
                    )),
                    Ok(flow_readings) => readings.extend(flow_readings),
                }
                let dump = prepared.dump();
                let full = state_count_of_the_flow(&prepared.base, &dump, Expansion::Full);
                anchors.push(anchor_body(
                    &format!("full_states_{label}"),
                    &pinned.full.to_string(),
                    &count_field(full),
                ));
                if kind == FixedFlowKind::MultiRecordPublish {
                    multi_record_full = full;
                }
            }
        }
    }
    let (clause_anchor_body, clause_anchor_matched) = anchor_body(
        "multi_record_full_named_by_the_clause",
        &MULTI_RECORD_FULL_STATES_NAMED_BY_THE_CLAUSE.to_string(),
        &count_field(multi_record_full),
    );
    for reading in &readings {
        output.emit(&reading.stop_check_body());
    }
    let anchors_mismatching = anchors.iter().filter(|(_, matched)| !matched).count();
    for (body, _matched) in anchors
        .iter()
        .chain([&(clause_anchor_body, clause_anchor_matched)])
    {
        output.emit(body);
    }
    let failed = readings.iter().filter(|reading| !reading.passed).count();
    output.emit_closing_line(
        "selfcheck_done",
        &format!("stop_checks={} failed={failed} anchors={} anchors_mismatching={anchors_mismatching} clause_anchor_matches={clause_anchor_matched}", readings.len(), anchors.len() + 1),
    );
    if failed > 0 || anchors_mismatching > 0 {
        EXIT_CODE_STOP_CLAUSE_FAILED
    } else {
        EXIT_CODE_OK
    }
}

fn main() {
    let forced_summary = write_dump::preflight(OWN_SOURCE_PATH);
    if let Some(summary) = forced_summary {
        println!("E7RESULT name=preflight forced={summary}");
    }
    let arguments =
        write_dump::arguments_without_the_force_flag(std::env::args().skip(1).collect());
    let exit_code = match arguments.split_first() {
        Some((subcommand, rest)) => match subcommand.as_str() {
            "selfcheck" => run_selfcheck_command(rest),
            "record-fixed-chains" => run_record_fixed_chains_command(rest),
            "keys" => run_keys_command(rest),
            "share" => run_share_command(rest),
            "library" => run_library_command(rest),
            "compare" => run_compare_command(rest),
            "compose" => run_compose_command(rest),
            other => usage_error(&format!("认不出的子命令 {other}")),
        },
        None => usage_error(
            "缺子命令：selfcheck、record-fixed-chains、keys、share、library、compare、compose 之一",
        ),
    };
    std::process::exit(exit_code);
}

#[cfg(test)]
mod helpers_shared_by_the_tests;
#[cfg(test)]
mod tests;
