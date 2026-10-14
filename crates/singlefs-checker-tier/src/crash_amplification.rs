//! 崩溃放量流水线（里程碑二「增补 4」第一、四、十一项；用户 2026-09-28 定的形态）：
//!
//! 1. **崩溃点与路径**：一条测试流程是从 mkfs 出发、一个崩溃点接一个崩溃点走下来的路径；每段归含它最后一次写的那个点，上游不看下游。每个点的身份是
//!    `<仓内路径>::<崩溃点名称>::<哈希串>`（[`crate::crash_identity`]）。几条流程共用前缀，全部点连成一棵树（用户 2026-09-28：「这就是个 B 树的问题」）：
//!    路径编号是从根到这个点、每个点取输入键（点名、写表、父镜像的摘要）前 16 个十六进制、用 `/` 逐段接起来的串：
//!    前面几段相同就是同一段枝干，一棵子树就是一段连续的键。哈希串（复用键）只由流程定：这一步写下的字节与它起步的镜像；
//!    按 COW 的性质，一个点的输入不变，它的崩溃状态就不变。判法不进键：判定块按（块键，判法版本）存，判法改了旧行留作历史、
//!    只在同一判法版本下复用（用户 2026-09-29 定：结果只绑流程，崩溃点变了或树变了才重测，代码、commit、树都不进编号）。
//!    判器判一个点的状态时另读的流程输入（这个点之前的写表、到这个点为止够得着的版本表、没做完的那次发布的根槽写；被判的根槽写不在里面，
//!    崩溃点重放合并第三轮判决 J3 的 M2-2 甲）
//!    不进复用键与身份（树不动），进这个点的计划哈希、块键（`JudgeInputsOfTheFlow`）：只差这些的两条流不互相复用判定。
//!    按发布分的态数要往下游找的点（段里与点里后面的段都没有根槽写），那几段归的发布（下游第一次根槽写，或后面没有根槽写）同样只进
//!    计划哈希（`publishes_looked_up_downstream_of_the_point`，用户 2026-10-11 定「乙」）：块上存的计数向量跟着下游变，下游不同的两条流不共享这几块。
//! 2. **先录后核**：录入把每个点的状态按块（点内序号区间）登进库的录入表；核对按批从库里取还没有判定的块，
//!    在 CPU 上（或 CPU + GPU：单元两道校验和上 GPU，与 CPU 对拍）判完、原子地写回判定块与违例；门禁一次读完。
//! 3. **对账**：一轮收尾，录入表里有、判定块表里没有的块就是没核完的（卡掉线、进程被杀），重新分给还在的一方补跑；
//!    领块只在内存里，库里不记谁领了，不会有块被一张死卡占着。
//! 4. **分工与跑法**：块按权重分给几方（一方一台机器的 CPU，或一张卡，权重取空闲显存），各写各的库，最后
//!    导入那几方的库时只点这一趟要的块（流水线是计划里不归本机的块，给定状态表是判到的、归别的机器的那一张，
//!    [`crate::verdict_store::VerdictStore::import_the_planned_rows_from`]）；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。
//! 5. **显存账**（`singlefs_harness::gpu_memory_ledger`，只防这台机器的显存 OOM、不压并发：用户 2026-10-06 定；只记这个进程借的、
//!    每次借现读卡上此刻的显存：用户 2026-10-07 定第 10t 批代码轮判决岔路 G1「二」，口径在那份文件头）：这个进程起过 GPU 用例共用的卡池
//!    （[`GpuCardsSource::TheCardPoolOfThisProcess`]）时，取卡照池里的卡、不登记也不起设备，借到哪张卡才在哪张上起（第 10t 批代码轮判决 K2-3）；
//!    第 ① 段判器的草稿区按派活装到每张卡上的状态数借（照 state_count 封顶，`singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`，
//!    显存按与 GPU 对拍用例同一份的 `crate::crash_judge_dispatch::mebibytes_a_judge_needs` 算，只记在那张卡上），每张卡装几个状态照这一刻
//!    这张卡能借给这个进程的量压（不照起池那一刻的空闲，第 10t 批代码轮攻方 S5）。几张卡按卡的先后每张当场试一次，借不到的卡这一趟
//!    不用（不装状态，它那一份留给后面的卡），一张都借不到就照对拍用例那一路打「本次未跑」那一行（同一个记号，原因写显存不够），
//!    这一趟照叫停的样子交回（`stopped_early`，判过的块在库里，分项驱动把这一项记成本次未跑；派出的项全没判完时整趟退 77）。不等。
//!    每张卡此刻都带不了一个状态而至少一张空着时带得了，同样是借不到、报本次未跑，不去借也不判红；空着时也带不了才判红
//!    （`singlefs_harness::gpu_memory_ledger::decide_scratch_before_booking`，第 10x 批代码轮判决 K2-2）。
//!    只排派活计划那一趟试判前同样借。
//!    第 ② 段核对（`crash_verify_gpu`）与第 ① 段由 CPU 判时的单元校验和对拍（`gpu_unit_checks`）在卡上建的缓冲同样进账：照卡的先后借第一张
//!    此刻放得下的，用借来的那张卡算，借不到报本次未跑（判过的块在库里）。量照
//!    `singlefs_harness::gpu_memory_ledger::mebibytes_of_buffers_held_at_once`，下面的字节数是照两份代码建的缓冲推的，没上卡量过。
//!    第 ② 段（`facts_verifier_buffer_footprint`）：上卡一次的六张事实表（`crate::crash_verify_gpu::GpuFactsVerifier::new`：
//!    录制流每次写 4 个字、每对重放 2 个、每段 9 个、段里每次写 1 个、每个单元 6 个、每条根 38 个，空表也建一个字），
//!    加一块一趟的头（12 个字）、结论与回读（每个状态各 1 个字），一块按 [`STATES_PER_BLOCK`] 个状态算；核对一块接一块，同时还活着的
//!    照 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD` 算。每写、每段、每单元、每根与头的字数取 `crash_verify_gpu` 导出的那几个常量（一处定义），
//!    每对重放两个字那一边没导出，这里另记一个。
//!    驱动的上下文与编好的管线算在每张卡第一次借时的驱动那一份里，不另记。
//!    账只记这个进程借的：两个进程同一刻读到同一份空闲、一起借超了，由运行时兜底：第 ① 段卡上建判器、判一块时 wgpu 报 Out of Memory，
//!    这张卡停下、这一块交回给别的卡，判不完才报本次未跑（原因写显存不够，`singlefs_harness::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`）；
//!    第 ② 段在卡上核时、只排派活计划那一趟试判时、卡池在借到的卡上起设备与编单元校验和管线时同样（第 10x 批代码轮判决 K1-3）。
//!    判器内核在另起的线程上编，那里的 Out of Memory 不进卡线程的错误范围：每个设备上装了接 wgpu 没接住的错误的回调，Out of Memory
//!    记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），卡线程上的活 panic 了就照它认是不是显存不够
//!    （`decide_the_work_on_the_card`），按这段活认、不按整台设备认（第 10z 批：别的线程一次 Out of Memory 不把这条线程上真正的 panic
//!    说成显存不够）；设备因为 Out of Memory 丢了，卡池下一次借这张卡时重起设备（驱动那一份在账上重新登记，放不下就报本次未跑）。
//!    第 ① 段在几张卡上判时，一张卡建判器或判一块时报了 Out of Memory，只停那一张：它手里那一块交回队里，别的卡照判
//!    （第 10z 批，第 10y 批代码轮判决 K2-2 P2）；剩下的块别的卡判完了这一趟照常交回。别的卡都已收工、还有交回的块没判时，收尾那一步
//!    再起一轮，只用没报过的卡（照这一刻能借的重新装状态、借草稿区）判剩下的块，判完的块照块序都入库（第 10z 批第三块，第 10z 批代码轮判决
//!    K2-1 第 5 条：状态全装在第一张卡时也起第二张）；报过的卡之外没有卡能判才报本次未跑。被叫停时卡照样领判完的块在等的那几块（交回的），
//!    判完的块照块序都入库再照叫停收尾，没判的留给下一趟（第 10z 批第五块，第 10z 批第三块代码轮判决 K1-5）；照叫停收尾而这一趟有卡报过
//!    Out of Memory，头一句照「本次未跑」那一行的形态打出来（第 10z 批第六块，第 10z 批第五块代码轮判决 K1-3）。叫停是叫停文件在不在，
//!    一趟里见过一次就记住（`singlefs_harness::gpu_memory_ledger::StopRequestRememberedInThisRun`，第 10z 批第五块代码轮判决 K2-2：
//!    外层循环、卡线程、一轮收尾、整趟收尾都问它，文件中途被删也不各读各的）。卡线程那一圈、起下一张卡、活交回那一刻记设备与按块序写库、
//!    几轮之间带着的与怎么走在 harness 档（`singlefs_harness::gpu_memory_ledger::judge_blocks_on_one_card`、`next_card_to_start`、
//!    `work_caught_noting_the_device_when_it_returned`、`BlocksStoredInOrder`、`RoundsOfTheCards`），不要卡就测得到。
//!    单元校验和对拍时一块的几段在同一个设备上并发，认一段是不是显存不够之前等同块的几段都收完（第 10z 批第三块，攻方：一段因设备丢了先失败，
//!    同块另一段范围里接到的 Out of Memory 还没记进设备时不判红）。
//!    没起过卡池的进程照旧不进账（[`GpuCardsSource::DevicesStartedForThisRun`]：这一趟自己在点名的卡上起设备），调用方是这几种：
//!    崩溃放量分项驱动（`scripts/crash/crash-amplification-item.sh`）起的、只有经 [`run_from_environment`] 跑的用例而没有一条取卡池的测试二进制
//!    （`tests/crash_amplification_fixed_script_through_rollback.rs`、`…_through_unmount.rs`、`crash_amplification_two_flows_share_their_prefix.rs`、
//!    经 `tests/common_crash_points` 跑的 `crash_enumeration_*` 那几份）；同一个测试二进制里有取卡池的 GPU 用例、而经 [`run_from_environment`]
//!    的用例跑在它们起池之前的（`crash_amplification_pipeline_records_checks_and_reuses.rs`、`record_checker_judges_absence_by_the_persisted_set.rs`）。
//!    直接调 `judge_my_blocks_on_gpu_cards` 的对拍用例不走这一条，卡由用例自己给、不进账。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
#[cfg(feature = "gpu")]
use std::time::Instant;

use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryReport};
use singlefs_core::root_record::ROOT_MAGIC;
use singlefs_harness::gpu_memory_ledger::StopRequestRememberedInThisRun;
#[cfg(feature = "gpu")]
use singlefs_harness::gpu_memory_ledger::{
    decide_scratch_before_booking, gpu_not_run_line, mebibytes_of_buffers_held_at_once,
    most_states_whose_need_fits_the_capacity, reason_for_a_booking_without_room_now,
    reason_for_running_out_of_gpu_memory, scratch_booking_refusal_outcome,
    scratch_states_to_book_on_each_card, BookingRefusal, NeedLargerThanEveryCard,
    ScratchBeforeBooking, ScratchBookingRefusalOutcome, ScratchStatesToBook,
    PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
};
#[cfg(feature = "gpu")]
use singlefs_harness::gpu_memory_ledger::{
    judge_blocks_on_one_card, next_card_to_start, work_caught_noting_the_device_when_it_returned,
    AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards,
    BlocksStoredInOrder, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards,
};
#[cfg(feature = "gpu")]
use singlefs_harness::gpu_memory_ledger::{WorkEndingTogether, WorksEndingTogetherOnADevice};
use singlefs_harness::heavy_threads::{
    own_variable_or_default_heavy_threads, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::{
    root_identity_of_write, root_identity_written_by, CrashImage, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, WrittenContents,
};
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use singlefs_checker::image::IMPLEMENTED_INVARIANTS;

use crate::crash::{
    crash_state_ranges_by_segment_with_slices,
    judge_crash_state_range_of_the_read_back_flow_with_slices, no_segment_slices,
    publish_of_each_segment, receive_until_the_senders_are_gone_with_wall_clock_lines,
    CrashPointReplayObserverCounts, CrashPointReplayPublishOfState,
    CrashPointReplaySegmentExpansion, CrashPointReplaySegmentSlice, CrashPointReplayVersionTable,
    ReceivedOrWallClockLineDue, StateCount1280Bits, TearableInPlaceOverwrites,
    WallClockProgressLines, STATE_COUNT_BYTES, STATE_COUNT_LIMBS,
};
use crate::crash_facts::{
    extract_flow_facts_with_slices, segment_expansion_facts,
    verify_states_from_facts_by_the_segment_expansions, FlowFacts, SegmentExpansionFact,
};
use crate::crash_given_states::{
    count_vector_of_the_given_states, judge_the_given_states_on_the_cpu,
    AbandonedRootNamedByThePoolChecker, ClaimedVersionOfAGivenState, ClaimedVersionsOfTheStates,
    GivenStateConclusion, GivenStatesConclusionsOfATable, GivenStatesConclusionsSource,
    GivenStatesJudge, GivenStatesOfATable, GivenStatesOfATableWithItsColumns,
};
use crate::crash_identity::{
    conclusion_code_entries_differing_from_the_build, conclusion_version_checked_against_the_build,
    content_digest_text, judging_code_of_the_judges, step_write_table_with_barriers,
    ConclusionKind, ConclusionVersion, ConclusionVersionRequest, CoverageReport, CoverageState,
    JudgingCode, JudgingCodeError, NodeLabel, ObserverName, ReuseKey, SectorImage, StepAttributes,
};
use crate::crash_judge_conclusions::{
    cross_check_words_of_a_stored_given_state, encoded_verdict_of_the_red_items,
    recovery_words_of_the_cross_check_words, red_items_of_a_given_state,
    red_items_of_the_encoded_verdict, CrossCheckWords, LandedRootWords, StateCountsOfABlock,
};
/// GPU 结论换红项、GPU 恢复字的换法在判器一侧（`crate::crash_judge_gpu`，崩溃点重放合并第三轮判决 J8：改结论的几处搬进判器一侧），
/// 这里照旧转出去：对拍用例经这条路径引。
#[cfg(feature = "gpu")]
pub use crate::crash_judge_gpu::{gpu_recovery_words, gpu_verdict_items};
#[cfg(feature = "gpu")]
use crate::crash_judge_gpu::{
    pool_checker_judged_bits, pool_checker_violated_bits, recovery_words_of_the_gpu_verdict,
};
use crate::crash_library::{
    crash_library_directory_of, CrashLibraryClass, CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
    CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE, CRASH_LIBRARY_LOCK_WAIT_SECONDS_ENVIRONMENT_VARIABLE,
};
pub use crate::crash_library::{
    refuse_a_library_that_a_reboot_would_erase, LibraryOpenInThisProcess, LibraryRole, RunStage,
    RUN_STAGE_ENVIRONMENT_VARIABLE,
};
use crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use crate::verdict_store::{
    BlockCountVector, BlockStartStateIndex, ConclusionBlockRecord, EnumerationPlanHash,
    InputFingerprint, JudgeOfTheProgress, JudgeVersion, JudgingDevice, RecoveryWordsOfState,
    SlicedSegmentProgress, SlicedSegmentProgressKey, StateOffsetInBlock, StoredViolation,
    StreamOrNodeName, VerdictBlock, VerdictBlockKey, VerdictStore, VerdictStoreError,
    VerdictVectorNumber,
};
use singlefs_checker::walk::{
    ContainerCapacityKnobs, LineageContainerCapacityKnob, RetirementContainerCapacityKnob,
    TombstoneContainerCapacityKnob,
};

/// 一块最多几个状态：一块是一个 GPU 批、一次原子写、对账与分工的一个单位（里程碑二「增补 4」第十一项「一块取 2¹⁶ 个状态」）。
pub const STATES_PER_BLOCK: u64 = 1 << 16;

/// 展开上限的环境变量：一段的写数不超过它就展开全部真子集，超过就不展开。全量脚本给全量的深度，没设由用例给默认。
pub const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO";

/// 展开上限：一段的写数不超过它，这一段展开全部真子集（[`CrashPointReplaySegmentExpansion::EveryProperSubset`]）；超过就不展开。
/// 每一段都不超过它，这一趟就是全域（`exhaustive=true`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpandUpToWritesPerSegment(pub usize);

impl ExpandUpToWritesPerSegment {
    /// 从 [`EXPAND_UP_TO_ENVIRONMENT_VARIABLE`] 取；没设用 `default_when_unset`。
    ///
    /// # Panics
    /// 设了而认不出（不是非负整数、空串、不是 UTF-8）：写明变量名与值，不悄悄退回默认（规模变量写错照样跑完、照样退 0，
    /// 跑的就不是要的那个规模）。
    #[must_use]
    pub fn from_environment_or(default_when_unset: usize) -> Self {
        Self::from_value_of_the_variable(
            process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),
            default_when_unset,
        )
    }

    /// 变量的值（`None` 是没设）换成展开上限：解析经数值环境变量共用的那一份
    /// （[`number_from_the_variable_value_panicking_when_unreadable`]），这里只给没设时的那一档。
    ///
    /// # Panics
    /// 设了而不是非负整数：消息带变量名与原值。
    #[must_use]
    pub fn from_value_of_the_variable(value: Option<&str>, default_when_unset: usize) -> Self {
        Self(
            number_from_the_variable_value_panicking_when_unreadable::<usize>(
                EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
                value,
            )
            .unwrap_or(default_when_unset),
        )
    }

    /// 一段按这个上限怎么展开。
    #[must_use]
    pub fn expansion_of(self, segment: &[usize]) -> CrashPointReplaySegmentExpansion {
        if segment.len() <= self.0 {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    }

    /// 每一段都展开到了（全域）。
    #[must_use]
    pub fn covers_every_segment(self, segments: &[Vec<usize>]) -> bool {
        segments.iter().all(|segment| segment.len() <= self.0)
    }
}

/// 一条流程里的一个崩溃点：名称（绑在设它的函数上）与它在写表里发出的写（下标区间）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashPointSpan {
    pub name: String,
    /// 设这个点的函数所在的文件（仓内路径）。共用前缀的点写在公共模块里：几条流调同一个函数，就得到同一个身份、同一个库里的节点。
    pub code_file_in_repository: String,
    pub writes: Range<usize>,
    /// 测试代码里预设的 ignore：这个点的段一律不展开（只留全持久那一个状态），路径照样往下走。
    pub ignored: bool,
}

/// 一条录好的测试流程：崩溃枚举要的五样，加上各点（名字、设它的文件、写区间；按流程次序、首尾相接）。
/// 版本表是两样：带文件的那几版（`versions`）与文件从那一版起没有了的那几版（`versions_without_a_file`，删除流的意图那一版，
/// D8（核心索引结构） 已定项 16 第 4 条；文件从不消失的流是空的），两个判器都按 [`CrashPointReplayVersionTable`] 认（第 33 批 R 报告「停下交主 agent」第一项）。
/// 同一个根两边都有是用例写错了：判的时候 [`CrashPointReplayVersionTable::new`] 停下。
pub struct CrashFlow<'flow> {
    pub base: &'flow MemoryPool,
    pub writes: &'flow [RetainedWrite],
    pub segments: &'flow [Vec<usize>],
    pub judged_root_index: usize,
    pub versions: &'flow [PublishedVersion],
    pub versions_without_a_file: &'flow [PublishedVersionWithoutAFile],
    pub expansion: &'flow (dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync),
    pub crash_points: Vec<CrashPointSpan>,
}

impl CrashFlow<'_> {
    /// 判器读的版本表（[`CrashPointReplayVersionTable::new`]）。
    #[must_use]
    pub fn version_table(&self) -> CrashPointReplayVersionTable<'_> {
        CrashPointReplayVersionTable::new(self.versions, self.versions_without_a_file)
    }
}

/// 一个崩溃点在这一趟里的计划：它管哪几段、这几段的状态在整条流里的序号区间、它的复用键与路径编号、分成几块。
pub struct PlannedCrashPoint {
    pub name: String,
    pub ignored: bool,
    pub segments: Vec<usize>,
    /// 这个点管的状态在整条流里的序号（首尾相接的一段；一段不展开时可能是空的）。
    pub ordinals: Range<u64>,
    pub reuse_key: ReuseKey,
    /// 从根到这个点的路径：每个点的名字（`<流程>.<周期>.<步>`）按次序用 `/` 接起来。同名同数据的前缀在几条流里是同一串；
    /// 名字相同、数据不同的分叉在路径这一级并成一段，靠身份（复用键里的父镜像与写表）区分。
    pub path_number: String,
    /// 身份 `<仓内路径>::<崩溃点名称>::<复用键>`：块键里的节点名只有它，路径不进块键（改名不使判定失效），路径 → 身份记在库的路径索引表里。
    pub identity: String,
    /// 判器判这个点的状态时另读的流程输入的摘要（`JudgeInputsOfTheFlow::digest_for_a_point`）；进 `plan_hash`。
    pub judge_inputs: [u8; DIGEST_BYTES],
    /// 点内每段的状态数与展开、判器另读的输入，加上按发布分的态数要往下游找的那几段各归哪次发布（`publishes_looked_up_downstream_of_the_point`）。
    /// 分片的段（C790）只写它整域的态数，不写这一趟的片起点与片长：几趟之间计划哈希不变。
    pub plan_hash: [u8; DIGEST_BYTES],
    /// 这个点管分片的段时（C790）它的状态按段切成的几截，按点内序号首尾相接；不管分片的段的点是空的（块照整点切，块键与分片之前逐个相同）。
    pub pieces_around_the_slices: Vec<PieceOfAPointWithASlice>,
}

/// 管分片的段的点里的一截状态（点内序号区间）：几段不分片的段连成一截，或一段分片的段自成一截。块按截切，块键按截取：
/// 分片的那一截锚在段内的绝对编号上（片起点 + 片内序号），几趟之间片起点、片长怎么变，同一个编号的块都是同一个块键；
/// 不分片的那几截锚在截内序号上（截号进计划哈希那一格），片长变了它们在点内的位置挪了，块键也不变。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PieceOfAPointWithASlice {
    pub local_states: Range<u64>,
    pub kind: KindOfAPiece,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一个点至多几截，分片那一截的两个 1280 位数按值存，比装箱多占的只是几百字节；读块键时不用再解一层引用"
)]
pub enum KindOfAPiece {
    /// 几段不分片的段：这个点里第几截这样的截（从 0 起）。
    SegmentsNotSliced { piece_index: usize },
    /// 一段分片的段：流里的段号、它在点内的段序、这一趟的片起点、整域的态数。
    SlicedSegment {
        segment_index: usize,
        segment_in_point: usize,
        start_within_the_domain: StateCount1280Bits,
        domain_state_count: StateCount1280Bits,
    },
}

impl PlannedCrashPoint {
    /// 这个点的块：点内序号区间（从 0 起），每块不超过 [`STATES_PER_BLOCK`] 个状态。管分片的段的点按截切（[`PieceOfAPointWithASlice`]），
    /// 每截从截头起切，最后一块可以不满。
    #[must_use]
    pub fn blocks(&self) -> Vec<Range<u64>> {
        let blocks_of = |states: &Range<u64>| {
            let count = states.end - states.start;
            (0..count.div_ceil(STATES_PER_BLOCK))
                .map(|index| {
                    states.start + index * STATES_PER_BLOCK
                        ..states.start + ((index + 1) * STATES_PER_BLOCK).min(count)
                })
                .collect::<Vec<Range<u64>>>()
        };
        if self.pieces_around_the_slices.is_empty() {
            return blocks_of(&(0..self.ordinals.end - self.ordinals.start));
        }
        self.pieces_around_the_slices
            .iter()
            .flat_map(|piece| blocks_of(&piece.local_states))
            .collect()
    }

    /// 点内序号 `local_start` 落在哪一截（管分片的段的点）。
    ///
    /// # Panics
    /// 这个点不管分片的段，或 `local_start` 不在任何一截里（块不是 [`Self::blocks`] 切出来的）。
    fn piece_of(&self, local_start: u64) -> &PieceOfAPointWithASlice {
        self.pieces_around_the_slices
            .iter()
            .find(|piece| piece.local_states.contains(&local_start))
            .expect("块是按截切的，块头落在某一截里")
    }

    /// `local` 这一块是不是分片的段里的块（录入时块长可以随片长变，[`VerdictStore::record_block_of_a_slice`]）。
    #[must_use]
    pub fn is_a_block_of_a_sliced_segment(&self, local: &Range<u64>) -> bool {
        !self.pieces_around_the_slices.is_empty()
            && matches!(
                self.piece_of(local.start).kind,
                KindOfAPiece::SlicedSegment { .. }
            )
    }

    pub fn block_key(&self, local: &Range<u64>) -> VerdictBlockKey {
        if self.pieces_around_the_slices.is_empty() {
            return self.block_key_under(self.plan_hash, local.start);
        }
        let piece = self.piece_of(local.start);
        let offset_in_the_piece = local.start - piece.local_states.start;
        match &piece.kind {
            KindOfAPiece::SegmentsNotSliced { piece_index } => {
                let mut message = self.plan_hash.to_vec();
                message.extend_from_slice(format!("piece_not_sliced={piece_index}\n").as_bytes());
                self.block_key_under(sha256_digest(&message), offset_in_the_piece)
            }
            KindOfAPiece::SlicedSegment {
                segment_in_point,
                start_within_the_domain,
                ..
            } => self.block_key_of_a_slice_block(
                *segment_in_point,
                &start_within_the_domain
                    .checked_add_u64(offset_in_the_piece)
                    .expect("片里的编号不越过整域，装得进 1280 位"),
            ),
        }
    }

    /// 点内第 `segment_in_point` 段（分片的段）里从段内编号 `start_within_the_domain` 起的那一块的块键：块起点那 8 字节是编号的低 64 位，
    /// 高位与段序进计划哈希那一格。只由计划哈希（不带片起点与片长）与编号定，几趟之间片怎么切都对得上。
    #[must_use]
    pub fn block_key_of_a_slice_block(
        &self,
        segment_in_point: usize,
        start_within_the_domain: &StateCount1280Bits,
    ) -> VerdictBlockKey {
        let (above_sixty_four_bits, low_sixty_four_bits) =
            start_within_the_domain.split_at_sixty_four_bits();
        let mut message = self.plan_hash.to_vec();
        message.extend_from_slice(
            format!(
                "sliced_segment_in_point={segment_in_point} block_start_above_sixty_four_bits={}\n",
                above_sixty_four_bits.decimal_text()
            )
            .as_bytes(),
        );
        self.block_key_under(sha256_digest(&message), low_sixty_four_bits)
    }

    fn block_key_under(&self, plan_hash: [u8; DIGEST_BYTES], block_start: u64) -> VerdictBlockKey {
        VerdictBlockKey {
            input_fingerprint: InputFingerprint(*self.reuse_key.as_bytes()),
            stream_or_node_name: StreamOrNodeName::new(&self.identity)
                .expect("节点名不超过 u16::MAX 字节"),
            enumeration_plan_hash: EnumerationPlanHash(plan_hash),
            block_start: BlockStartStateIndex(block_start),
        }
    }
}

/// 一条流程的整份计划：各点，加上每段归哪个点（ignore 的点的段不展开，判段怎么展开时要它）、每段这一趟的片（C790）。
pub struct CrashPlan {
    pub points: Vec<PlannedCrashPoint>,
    owner_of_segment: Vec<usize>,
    segment_slices: Vec<Option<CrashPointReplaySegmentSlice>>,
}

impl CrashPlan {
    /// 每段这一趟的片（与段同序，不分片的段是 `None`）：计划、事实表、判表、判与核对都照这一份。
    #[must_use]
    pub fn segment_slices(&self) -> &[Option<CrashPointReplaySegmentSlice>] {
        &self.segment_slices
    }

    /// 这一趟分片的段：（点号，那一截）。
    #[must_use]
    pub fn sliced_segments(&self) -> Vec<(usize, &PieceOfAPointWithASlice)> {
        self.points
            .iter()
            .enumerate()
            .flat_map(|(point_index, point)| {
                point
                    .pieces_around_the_slices
                    .iter()
                    .filter(|piece| matches!(piece.kind, KindOfAPiece::SlicedSegment { .. }))
                    .map(move |piece| (point_index, piece))
            })
            .collect()
    }

    /// 一段怎么展开：归 ignore 的点就不展开，其余照流程自己的展开方式。计划与核对都经它，两处不会不一致。
    #[must_use]
    pub fn segment_expansion(
        &self,
        flow: &CrashFlow<'_>,
        segment_index: usize,
        segment: &[usize],
    ) -> CrashPointReplaySegmentExpansion {
        if self.points[self.owner_of_segment[segment_index]].ignored {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            (flow.expansion)(segment_index, segment)
        }
    }
}

/// 每段归哪个点：含这一段最后一次写的那个点。一段跨了两个点（上一次发布的系统配置槽写与下一次发布的单元写没被屏障隔开时）归后面
/// 那个点：那一段里「上一个点收尾的几次写没落」的状态，是走进后面那个点的过程；上游的点只看自己与更前面的写，不因下游的改动而变
/// （用户 2026-09-28：输入决定从这个点往后要不要重验）。
fn owner_of_each_segment(flow: &CrashFlow<'_>) -> Vec<usize> {
    flow.segments
        .iter()
        .map(|segment| {
            let last = segment.iter().copied().max().expect("段里至少一次写");
            flow.crash_points
                .iter()
                .position(|point| point.writes.contains(&last))
                .expect("每次写都落在某个崩溃点里")
        })
        .collect()
}

/// 给一条流程的每个崩溃点定计划：含它最后一次写的段归它（[`owner_of_each_segment`]）；写表按它管的那几段取、父输出是它第一次管到的写
/// 之前的镜像；状态序号按点的次序首尾相接地划分整条流；ignore 的点的段不展开。最后那个全持久状态不归最后一个点，另立成这条流自己的
/// 末节点（[`END_OF_FLOW_NODE_NAME`]，计划里最后一项）：它的写表是空的、父输出是整张写表叠完的镜像、设点文件取最后一个点的
/// （崩溃点重放合并第三轮正文 H3「多记录流与删除流共享到 C」，主 agent 定）。最后一个点的计划因此只由它自己与它之前的写定，
/// 一条流在别的流的前缀上收尾时，那几个点的块键与接着往下走的流逐个相同——按发布分的态数要往下游找的点除外
/// （[`publishes_looked_up_downstream_of_the_point`]：那几段归的发布，也就是下游第一次根槽写的位置与身份或「后面没有根槽写」，进计划哈希；
/// 用户 2026-10-11 定「乙」）：多记录流的 C 尾（C 那次发布根槽写之后的系统配置槽轮换）后面没有根槽写，删除流的 C 尾归意图那次发布，
/// 两条流在这一块上不共享，C 尾之前的点照旧共享。
/// 计划不看判法：判法是判定行的属性（[`PipelineJudgingCode::version`]），改了它树与块键都不动。判器另读的流程输入（`JudgeInputsOfTheFlow`）
/// 进每个点的计划哈希，不进复用键与身份。
///
/// # Panics
/// 崩溃点的写区间不是首尾相接地罩住整张写表；某个点的名称或文件路径不合身份的规矩（同一个名字在一条流程里可以出现几次：
/// 同一个函数调几次就是几个点，树上的位置分得开）；
/// 各段的状态序号接不上（枚举计划与切段不一致，是这里的 bug）；
/// 对基镜像本身照 journal 恢复，落到的根不是基镜像里最新的那条根（B0 的前提，
/// `assert_journal_recovery_of_the_base_image_lands_on_its_newest_root`）。
#[must_use]
pub fn plan_crash_points(flow: &CrashFlow<'_>) -> CrashPlan {
    plan_crash_points_with_slices(flow, &no_segment_slices(flow.segments.len()))
}

/// 同 [`plan_crash_points`]，分片的段照 `segment_slices`（与段同序，C790）：它在流里的序号区间是片长；管它的点按截切块
/// （[`PieceOfAPointWithASlice`]），计划哈希里它只写整域的态数，不写片起点与片长。
///
/// # Panics
/// 同 [`plan_crash_points`]；`segment_slices` 与段数不同；片越过段的域，或分片的段整域越过 1280 位。
#[must_use]
pub fn plan_crash_points_with_slices(
    flow: &CrashFlow<'_>,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> CrashPlan {
    let mut expected_start = 0usize;
    for point in &flow.crash_points {
        assert_eq!(
            point.writes.start, expected_start,
            "崩溃点 {} 的写区间没有接上前一个点",
            point.name
        );
        expected_start = point.writes.end;
    }
    assert_eq!(
        expected_start,
        flow.writes.len(),
        "崩溃点的写区间没有罩住整张写表"
    );
    let owner_of_segment = owner_of_each_segment(flow);
    let ignored_points: Vec<bool> = flow
        .crash_points
        .iter()
        .map(|point| point.ignored)
        .collect();
    let expansion = |segment_index: usize, segment: &[usize]| {
        if ignored_points[owner_of_segment[segment_index]] {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            (flow.expansion)(segment_index, segment)
        }
    };
    let (ranges_by_segment, state_count) = crash_state_ranges_by_segment_with_slices(
        flow.base,
        flow.writes,
        flow.segments,
        &expansion,
        segment_slices,
    );
    // 分片的段整域的态数（进计划哈希，不写片起点与片长）；不分片的段是 None
    let tearable = segment_slices
        .iter()
        .any(Option::is_some)
        .then(|| TearableInPlaceOverwrites::of(flow.base, flow.writes));
    let domain_of_each_sliced_segment: Vec<Option<StateCount1280Bits>> = flow
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            segment_slices[segment_index].map(|_slice| {
                expansion(segment_index, segment)
                    .checked_state_count(
                        flow.writes,
                        segment,
                        tearable
                            .as_ref()
                            .expect("有分片的段才进到这里，可撕表建过了"),
                    )
                    .expect("分片的段整域的态数装得进 1280 位：录入段在写库之前核过")
            })
        })
        .collect();
    // 一段有没有状态：分片的段照它整域算（片走完了、这一趟片长是 0 也算有），计划哈希几趟之间不变
    let segment_has_states: Vec<bool> = ranges_by_segment
        .iter()
        .zip(&domain_of_each_sliced_segment)
        .map(|(range, domain)| match domain {
            Some(domain) => !domain.is_zero(),
            None => !range.is_empty(),
        })
        .collect();
    let publish_of_segment = publish_of_each_segment(flow.writes, flow.segments);
    let expansion_names: Vec<&'static str> = flow
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| expansion(segment_index, segment).name())
        .collect();
    let judge_inputs_of_the_flow = JudgeInputsOfTheFlow::of(flow);
    let mut parent_image = SectorImage::of_base(flow.base);
    let mut applied_writes = 0usize;
    let mut path_segments: Vec<String> = Vec::new();
    let mut next_ordinal = 0u64;
    let mut planned = Vec::new();
    for (point_index, point) in flow.crash_points.iter().enumerate() {
        let owned: Vec<usize> = (0..flow.segments.len())
            .filter(|segment| owner_of_segment[*segment] == point_index)
            .collect();
        let owned_writes: BTreeSet<usize> = owned
            .iter()
            .flat_map(|segment| flow.segments[*segment].iter().copied())
            .collect();
        let first_write = owned_writes.first().copied().unwrap_or(point.writes.start);
        let last_write = owned_writes.last().map_or(first_write, |last| last + 1);
        assert!(
            first_write >= applied_writes,
            "崩溃点 {} 管的写在前一个点已经叠进父镜像的写之前",
            point.name
        );
        parent_image.overlay(&flow.writes[applied_writes..first_write]);
        applied_writes = first_write;
        let owned_segments: Vec<Vec<usize>> = owned
            .iter()
            .map(|segment| flow.segments[*segment].clone())
            .collect();
        let attributes = StepAttributes {
            write_table_with_barriers: step_write_table_with_barriers(
                flow.writes,
                &owned_segments,
                first_write..last_write,
            ),
            parent_output: parent_image.digest(),
        };
        let reuse_key = attributes.reuse_key();
        let label =
            NodeLabel::new(&point.code_file_in_repository, &point.name).expect("崩溃点标签合规矩");
        let identity = label.identity_text(&reuse_key);
        assert!(
            !point.name.contains('/') && !point.name.contains('|'),
            "崩溃点名字 {} 里不许有 / 与 |：它是路径段，也是节点名的一截",
            point.name
        );
        path_segments.push(point.name.clone());
        let path_number = path_segments.join("/");
        let start_ordinal = next_ordinal;
        let mut plan_text = String::new();
        let mut pieces_around_the_slices: Vec<PieceOfAPointWithASlice> = Vec::new();
        let point_has_a_sliced_segment = owned
            .iter()
            .any(|segment| segment_slices[*segment].is_some());
        let mut states_outside_the_slices = 0u64;
        for (segment_in_point, segment) in owned.iter().enumerate() {
            let range = &ranges_by_segment[*segment];
            assert_eq!(
                range.start, next_ordinal,
                "段 {segment} 的状态序号没有接上前一段"
            );
            next_ordinal = range.end;
            let local_states = range.start - start_ordinal..range.end - start_ordinal;
            // 计划哈希只写点内的段序，不写流里的段序：同一个前缀节点在几条流里都是同一份计划
            match (
                &segment_slices[*segment],
                &domain_of_each_sliced_segment[*segment],
            ) {
                (Some(slice), Some(domain)) => {
                    // 分片的段只写整域的态数：片起点与片长不进计划哈希，几趟之间块键对得上
                    plan_text.push_str(&format!(
                        "segment={segment_in_point} sliced_domain_states={} expansion={}\n",
                        domain.decimal_text(),
                        expansion_names[*segment]
                    ));
                    pieces_around_the_slices.push(PieceOfAPointWithASlice {
                        local_states,
                        kind: KindOfAPiece::SlicedSegment {
                            segment_index: *segment,
                            segment_in_point,
                            start_within_the_domain: slice.start_within_the_domain,
                            domain_state_count: *domain,
                        },
                    });
                }
                (None, _) | (Some(_), None) => {
                    plan_text.push_str(&format!(
                        "segment={segment_in_point} states={} expansion={}\n",
                        range.end - range.start,
                        expansion_names[*segment]
                    ));
                    states_outside_the_slices += range.end - range.start;
                    if point_has_a_sliced_segment {
                        match pieces_around_the_slices.last_mut() {
                            Some(PieceOfAPointWithASlice {
                                local_states: piece_states,
                                kind: KindOfAPiece::SegmentsNotSliced { .. },
                            }) => piece_states.end = local_states.end,
                            Some(PieceOfAPointWithASlice {
                                kind: KindOfAPiece::SlicedSegment { .. },
                                ..
                            })
                            | None => {
                                let piece_index = pieces_around_the_slices
                                    .iter()
                                    .filter(|piece| {
                                        matches!(piece.kind, KindOfAPiece::SegmentsNotSliced { .. })
                                    })
                                    .count();
                                pieces_around_the_slices.push(PieceOfAPointWithASlice {
                                    local_states,
                                    kind: KindOfAPiece::SegmentsNotSliced { piece_index },
                                });
                            }
                        }
                    }
                }
            }
        }
        // 按发布分的态数要往下游找的段：它归的那次发布（下游第一次根槽写的位置与身份，或后面没有根槽写）进计划哈希，
        // 下游不同的两条流在这几块上不共享；不看下游的点计划文本不变，照旧共享
        let publishes_looked_up_downstream = publishes_looked_up_downstream_of_the_point(
            &owned,
            &owned_writes,
            &publish_of_segment,
            &segment_has_states,
        );
        for (segment_in_point, publish) in &publishes_looked_up_downstream {
            plan_text.push_str(&format!(
                "segment={segment_in_point} publish_looked_up_downstream={}\n",
                publish.name()
            ));
        }
        let ordinals = start_ordinal..next_ordinal;
        if point_has_a_sliced_segment {
            // 点的状态数随片长变：只写分片的段之外的（分片的段的整域已写在它那一行）
            plan_text.push_str(&format!(
                "states_outside_the_slices={states_outside_the_slices}\n"
            ));
        } else {
            plan_text.push_str(&format!("states={}\n", ordinals.end - ordinals.start));
        }
        // 判器另读的输入进计划哈希、不进复用键：树（身份、路径）照旧只由流程的这一步与它起步的镜像定，判定块按判器真读到的输入分
        let judge_inputs =
            judge_inputs_of_the_flow.digest_for_a_point(flow, first_write, last_write);
        plan_text.push_str(&format!(
            "judge_inputs={}\n",
            hexadecimal_text(&judge_inputs)
        ));
        planned.push(PlannedCrashPoint {
            name: point.name.clone(),
            ignored: point.ignored,
            segments: owned,
            ordinals,
            reuse_key,
            identity,
            path_number,
            judge_inputs,
            plan_hash: sha256_digest(plan_text.as_bytes()),
            pieces_around_the_slices,
        });
    }
    planned.push(end_of_flow_node(
        flow,
        &judge_inputs_of_the_flow,
        parent_image,
        applied_writes,
        &mut path_segments,
        next_ordinal..state_count,
    ));
    CrashPlan {
        points: planned,
        owner_of_segment,
        segment_slices: segment_slices.to_vec(),
    }
}

/// 一个点管的段里，按发布分的态数要往下游找的那几段：这一段归的那次发布（[`publish_of_each_segment`]，崩溃点重放计数与崩溃放量计数向量
/// 同一份归法）不是这个点自己管的写里的根槽写，而是这个点之后第一次根槽写，或者后面没有根槽写可归
/// （[`CrashPointReplayPublishOfState::AfterTheLastRoot`]）。交回（段在点内的序号，它归的那次发布），按点内的段序；一个状态都没有的段不算
/// （它不进计数向量）。同一个前缀节点在两条流里下游不同，这几段的态数就归到不同的发布上，块键得分开（用户 2026-10-11 定「乙」：
/// 看下游的块把后面第一次根槽写也算进块键，其余块照旧共享）。
///
/// # Panics
/// 归法给某一段交回了 [`CrashPointReplayPublishOfState::EveryWritePersisted`]：那一格只给整条流最后全部持久的那一个状态，不给段。
fn publishes_looked_up_downstream_of_the_point(
    owned_segments: &[usize],
    owned_writes: &BTreeSet<usize>,
    publish_of_segment: &[CrashPointReplayPublishOfState],
    segment_has_states: &[bool],
) -> Vec<(usize, CrashPointReplayPublishOfState)> {
    owned_segments
        .iter()
        .enumerate()
        .filter(|(_segment_in_point, segment)| segment_has_states[**segment])
        .filter_map(|(segment_in_point, segment)| {
            let publish = publish_of_segment[*segment];
            let is_looked_up_downstream = match publish {
                CrashPointReplayPublishOfState::UpToTheRootOf {
                    root_write_index, ..
                } => !owned_writes.contains(&root_write_index),
                CrashPointReplayPublishOfState::AfterTheLastRoot => true,
                CrashPointReplayPublishOfState::EveryWritePersisted => panic!(
                    "publish_of_each_segment 只交回根槽写那一格或 after_the_last_root，段 {segment} 归成了整条流最后全部持久那一个状态"
                ),
            };
            is_looked_up_downstream.then_some((segment_in_point, publish))
        })
        .collect()
}

/// 末节点的名字（[`plan_crash_points`]）：每条流最后全部持久的那一个状态单独一个节点。
pub const END_OF_FLOW_NODE_NAME: &str = "end_of_flow";

/// 一条流的末节点（[`plan_crash_points`]）：管各段之后剩下的那一个状态（整张写表全部持久），写表是空的、父输出是 `parent_image` 叠上
/// `applied_writes` 之后全部写的镜像，设点文件取最后一个点的；判器另读的输入按「这个点在写表末尾」取。计划哈希只写它的状态数与判器另读的输入。
///
/// # Panics
/// 流里一个崩溃点都没有；各段之后剩下的不是恰好一个状态（`crash_state_ranges_by_segment` 交回的状态数是各段之和加一）。
fn end_of_flow_node(
    flow: &CrashFlow<'_>,
    judge_inputs_of_the_flow: &JudgeInputsOfTheFlow,
    mut parent_image: SectorImage,
    applied_writes: usize,
    path_segments: &mut Vec<String>,
    ordinals: Range<u64>,
) -> PlannedCrashPoint {
    assert_eq!(
        ordinals.end - ordinals.start,
        1,
        "各段之后只剩整条流最后全部持久的那一个状态：各点的序号区间加末节点要正好是整条流的状态数"
    );
    let last_point = flow
        .crash_points
        .last()
        .expect("一条流至少一个崩溃点：末节点的设点文件取最后一个点的");
    parent_image.overlay(&flow.writes[applied_writes..]);
    let write_count = flow.writes.len();
    let attributes = StepAttributes {
        write_table_with_barriers: step_write_table_with_barriers(
            flow.writes,
            &[],
            write_count..write_count,
        ),
        parent_output: parent_image.digest(),
    };
    let reuse_key = attributes.reuse_key();
    let label = NodeLabel::new(&last_point.code_file_in_repository, END_OF_FLOW_NODE_NAME)
        .expect("最后一个点的设点文件合规矩，末节点的名字不空");
    let identity = label.identity_text(&reuse_key);
    path_segments.push(END_OF_FLOW_NODE_NAME.to_string());
    let judge_inputs = judge_inputs_of_the_flow.digest_for_a_point(flow, write_count, write_count);
    let plan_text = format!(
        "end_of_flow states={}\njudge_inputs={}\n",
        ordinals.end - ordinals.start,
        hexadecimal_text(&judge_inputs)
    );
    PlannedCrashPoint {
        name: END_OF_FLOW_NODE_NAME.to_string(),
        ignored: false,
        segments: Vec::new(),
        ordinals,
        reuse_key,
        path_number: path_segments.join("/"),
        identity,
        judge_inputs,
        plan_hash: sha256_digest(plan_text.as_bytes()),
        pieces_around_the_slices: Vec::new(),
    }
}

// ============================== 判器另读的流程输入：进计划哈希 ==============================

/// 写表里的一次写在判器输入里的写法：盘、种类、FUA、偏移、长度、内容摘要（与 `step_write_table_with_barriers` 同一套字段），
/// 后面接它在整条流里的段号（不接的写 `-`）。
fn judge_input_line_of_a_write(
    write: &RetainedWrite,
    segment_in_the_flow: Option<usize>,
) -> String {
    let segment_text =
        segment_in_the_flow.map_or_else(|| "-".to_string(), |segment| segment.to_string());
    format!(
        "W {} {} {} {} {} {} {segment_text}\n",
        write.device.0,
        write.kind.name(),
        u8::from(write.is_force_unit_access),
        write.offset.0,
        write.length_in_bytes(),
        content_digest_text(&write.contents),
    )
}

/// 一个点最后一次写之后的写表，判器判这个点的状态时读到多少（由整条流的根槽写定）。
enum WritesAfterThePoint {
    /// 根槽写的 txg 沿写表严格递增：之后的写只有「没做完的那次发布」的根槽写读得到（判据见 `JudgeInputsOfTheFlow`）。
    ReadOnlyThroughTheUnfinishedPublish,
    /// 根槽写的 txg 不是严格递增：之后的写都可能读到。第 k 项是写表第 k 次写往后逐行链起来的摘要（`k` 从 0 到写数）。
    MayAllBeRead {
        suffix_digests: Vec<[u8; DIGEST_BYTES]>,
    },
}

/// 判器（CPU 的 `crash::judge_crash_image` 与 GPU 判器，读的是同一份 [`CrashFlow`]）判一个点的状态时，复用键（这一步带屏障的写表、
/// 起步镜像的扇区终值）之外还读到的流程输入。R1（`research/prompts/test-script-align-code-r1-main-verification.md`「二、判定与去向」）：
/// 判器读到的每一样都进那一块的键，只差这些的两条流不互相复用；前缀共享在判器读到的真相同时保留。逐样：
///
/// - 这个点之前的写表（不只是扇区终值）：记录核对器按写表分发布、按写的种类找根槽写与系统配置槽写
///   （`crash::check_records_against`、`crash::reuse_is_not_proven_illegal_by_the_reclaim_predicate`），种类、次序变了而扇区终值相同时判定会变。
///   按（写、整条流里的段号）逐行链起来取摘要。
/// - 版本表：oracle 只按恢复落到的根（实例，txg）找那一版、找比它旧的版本（`crash::classified_oracle_violation_for_the_version_table`）；
///   「没有文件的那几版」（`CrashFlow::versions_without_a_file`）与带文件的那几版按同一个上界进键，一行都没有时一个字都不加
///   （文件从不消失的流键不变）。
///   落到的根要么是基镜像里本来就有的根（基镜像里带根 magic 的扇区；基镜像自己的 journal 不越过其中最新那条，排计划时断言：
///   `assert_journal_recovery_of_the_base_image_lands_on_its_newest_root`），要么是写表里到这个点为止写过的根槽写，要么是 journal 重放出来的
///   那一版——没做完的那次发布，它的根槽写在这个点之后、是下一条。够得着的版本是（txg，实例）不超过这几样根里最大那一条的行：
///   这个点之前一条根槽写都没有、又没有没做完的发布时（每条流的第一个点只有系统配置槽写），上界是基镜像里的根，版本表里够不着的行
///   不进，几条只差版本表的流在这里照样共享（test-script-align-code-r2 判决 J1）。没做完的那次发布在写表里没有根槽写，或基镜像里一条根都没有、
///   这个点之前也没有根槽写，上界取不出，整张表都进。
/// - 没做完的那次发布的根槽写（这个点最后一次写之前、最后一条根槽写之后还有单元写或记录写时，写表里下一条根槽写）：记录核对器按它分发布、
///   取 txg（`crash::checkpoint_txg_of_the_publish_that_made_the_write`），恢复落到它那一版时读它带的实例表
///   （`crash::instance_table_of_the_effective_root` 在整张写表里找写出这个根身份的根槽写）。
/// - 再往后的写：记录核对器对每次发布都看，但只在恢复落到的根 txg 不小于那次发布的 txg 时才核它的单元，只在它的根槽写的字节恰好在盘上时
///   才核「根在而记录不在」。根槽写的 txg 沿写表严格递增时这两样对这个点的状态都不成立：落到的根至多是没做完的那一版（前提是对基镜像本身
///   照 journal 恢复落到的就是它里面最新的那条根，不另越过一版：排计划时断言，`assert_journal_recovery_of_the_base_image_lands_on_its_newest_root`，
///   不成立的流排计划时就拒），更晚的根槽写 txg 都更大、字节不在盘上。
///   不严格递增时推不出，之后的写整段进摘要（[`WritesAfterThePoint::MayAllBeRead`]）。
/// - 被判的根槽写（`judged_root_index`）不进键（崩溃点重放合并第三轮判决 J3 的 M2-2 甲）：判器拿它记「根槽已持久」，不进逐项判定
///   （CPU 的 `crash::CrashStateJudgement::root_persisted`、GPU 判表头里那一个字都只记不判），被判根不同的两条流照样共享前缀；
///   「根槽已持久」的计数各项按自己的被判根事后数（[`states_with_the_root_slot_write_persisted`]，J7）。此前的写法把它在这个点之前
///   （每个状态都持久）、在点里（第几次写）、还是在之后（都不持久）拼进计划哈希，多记录流的 C 因此与删除流不共享。
/// - 起点镜像有哪几块盘、每块多大（扇区终值里看不出）。
struct JudgeInputsOfTheFlow {
    base_devices_text: String,
    /// 第 k 项：写表前 k 次写逐行链起来的摘要（`k` 从 0 到写数）。
    prefix_digests: Vec<[u8; DIGEST_BYTES]>,
    /// 每条根槽写：写表下标与它写下的根身份，（txg，实例）按择新的次序比。
    root_writes: Vec<(usize, (CheckpointTxg, InstanceGeneration))>,
    /// 基镜像里（txg，实例）最大的那条根；一条都没有是 `None`。
    newest_root_in_the_base_image: Option<(CheckpointTxg, InstanceGeneration)>,
    writes_after_the_point: WritesAfterThePoint,
}

/// 基镜像里（txg，实例）最大的那条根：写过的扇区里以根 magic 起头的（根槽写是一个物理块，崩溃点重放的池按 512 字节的扇区建），
/// 身份照根槽写的读法取（[`root_identity_written_by`]，不验自证校验和：判器输入只要知道恢复最多落到哪一代）。一条都没有是 `None`。
fn newest_root_in_the_base_image(base: &MemoryPool) -> Option<(CheckpointTxg, InstanceGeneration)> {
    base.devices
        .values()
        .flat_map(|device| device.written_sectors().values())
        .filter(|bytes| {
            bytes.len() >= ROOT_RECORD_IDENTITY_END && bytes[..ROOT_MAGIC.len()] == ROOT_MAGIC
        })
        .map(|bytes| {
            let (instance, checkpoint_txg) = root_identity_written_by(bytes);
            (checkpoint_txg, instance)
        })
        .max()
}

/// 根记录里身份那几个字节到哪为止：实例代号在偏移 24（4 字节）、checkpoint_txg 在偏移 28（8 字节）（[`root_identity_written_by`]）。
const ROOT_RECORD_IDENTITY_END: usize = 36;

/// B0 的前提（用户 2026-10-02 定「窄一」）：对基镜像本身照 journal 恢复，落到的就是基镜像里最新的那条根。
/// 恢复走判器那一条（`crash::judge_crash_image` 里的 `recover(…, JournalPolicy::Consult)`），读的是一次写都没落的崩溃镜像，也就是基镜像本身。
/// 基镜像是崩溃状态（journal 记录落了、根槽写没落）时，恢复落到基镜像里没有根的那一版；B0 按基镜像里的根截够得着的版本，
/// 就把这一版截掉了，判定不同的两条流会共用块键（test-script-align-code-r3 攻方 L1）。
///
/// # Panics
/// 落到的根与 `newest_root_of_the_base` 不同（连同一边有根、一边没有）。
fn assert_journal_recovery_of_the_base_image_lands_on_its_newest_root(
    base: &MemoryPool,
    newest_root_of_the_base: Option<(CheckpointTxg, InstanceGeneration)>,
) {
    let base_with_no_write_persisted = CrashImage {
        base,
        writes: &[],
        persisted: Vec::new(),
    };
    let landed_root = recover(&base_with_no_write_persisted, JournalPolicy::Consult)
        .effective_root
        .map(|(instance, checkpoint_txg)| (checkpoint_txg, instance));
    assert_eq!(
        landed_root, newest_root_of_the_base,
        "基镜像照 journal 恢复会越过它里面最新的根，B0 的前提不成立（要求恢复落到的正是基镜像里最新的那条根）：\
         恢复落到（txg，实例）{landed_root:?}，基镜像里最新的根是 {newest_root_of_the_base:?}。\
         B0 按基镜像里的根截够得着的版本，恢复落到别的根时会截掉判器读得到的那一版，判定不同的两条流共用块键。\
         出路：基镜像先可写挂载一次（恢复把 journal 落成根、再发布）之后再取，或让流从 mkfs 起步；\
         要收这种基镜像，先定 B0 的上界怎么并上恢复落到的根（交设计，不在这里放宽）"
    );
}

impl JudgeInputsOfTheFlow {
    fn of(flow: &CrashFlow<'_>) -> Self {
        let mut segment_of_write: Vec<Option<usize>> = vec![None; flow.writes.len()];
        for (segment_index, segment) in flow.segments.iter().enumerate() {
            for write_index in segment {
                segment_of_write[*write_index] = Some(segment_index);
            }
        }
        let lines: Vec<String> = flow
            .writes
            .iter()
            .zip(&segment_of_write)
            .map(|(write, segment)| judge_input_line_of_a_write(write, *segment))
            .collect();
        let chained = |previous: &[u8; DIGEST_BYTES], line: &str| {
            let mut message = previous.to_vec();
            message.extend_from_slice(line.as_bytes());
            sha256_digest(&message)
        };
        let mut prefix_digests = Vec::with_capacity(lines.len() + 1);
        prefix_digests.push(sha256_digest(b"writes before the point\n"));
        for line in &lines {
            let previous = *prefix_digests.last().expect("起头那一项先放进去了");
            prefix_digests.push(chained(&previous, line));
        }
        let root_writes: Vec<(usize, (CheckpointTxg, InstanceGeneration))> = flow
            .writes
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
            .map(|(index, write)| {
                let (instance, checkpoint_txg) = root_identity_of_write(write);
                (index, (checkpoint_txg, instance))
            })
            .collect();
        let txgs_of_the_root_writes_strictly_increase = root_writes
            .windows(2)
            .all(|pair| pair[0].1 .0 < pair[1].1 .0);
        let writes_after_the_point = if txgs_of_the_root_writes_strictly_increase {
            WritesAfterThePoint::ReadOnlyThroughTheUnfinishedPublish
        } else {
            let mut suffix_digests =
                vec![sha256_digest(b"writes after the point\n"); lines.len() + 1];
            for index in (0..lines.len()).rev() {
                suffix_digests[index] = chained(&suffix_digests[index + 1], &lines[index]);
            }
            WritesAfterThePoint::MayAllBeRead { suffix_digests }
        };
        let devices: Vec<String> = flow
            .base
            .devices
            .keys()
            .map(|device| device.0.to_string())
            .collect();
        let newest_root_of_the_base = newest_root_in_the_base_image(flow.base);
        assert_journal_recovery_of_the_base_image_lands_on_its_newest_root(
            flow.base,
            newest_root_of_the_base,
        );
        Self {
            base_devices_text: format!(
                "base_devices={} device_size_in_bytes={}\n",
                devices.join(","),
                flow.base.device_size_in_bytes
            ),
            prefix_digests,
            root_writes,
            newest_root_in_the_base_image: newest_root_of_the_base,
            writes_after_the_point,
        }
    }

    /// 管写表 `first_write..last_write` 的那个点：它的状态里判器另读的输入（逐样见 [`JudgeInputsOfTheFlow`]）逐行写下、取 SHA-256。
    fn digest_for_a_point(
        &self,
        flow: &CrashFlow<'_>,
        first_write: usize,
        last_write: usize,
    ) -> [u8; DIGEST_BYTES] {
        let mut text = self.base_devices_text.clone();
        text.push_str(&format!(
            "writes_before_the_point={}\n",
            hexadecimal_text(&self.prefix_digests[first_write])
        ));
        let writes_since_the_last_root = self
            .root_writes
            .iter()
            .rev()
            .find(|(index, _)| *index < last_write)
            .map_or(0, |(index, _)| index + 1)..last_write;
        let a_publish_is_unfinished = flow.writes[writes_since_the_last_root]
            .iter()
            .any(|write| matches!(write.kind, StepKind::UnitWrite | StepKind::JournalRecord));
        let unfinished_publish_root = if a_publish_is_unfinished {
            self.root_writes
                .iter()
                .find(|(index, _)| *index >= last_write)
        } else {
            None
        };
        match unfinished_publish_root {
            Some((index, _)) => text.push_str(&format!(
                "unfinished_publish_root={}",
                judge_input_line_of_a_write(&flow.writes[*index], None)
            )),
            None => text.push_str(if a_publish_is_unfinished {
                "unfinished_publish_root=absent_from_the_write_table\n"
            } else {
                "unfinished_publish_root=none\n"
            }),
        }
        let highest_reachable_root: Option<(CheckpointTxg, InstanceGeneration)> =
            if a_publish_is_unfinished && unfinished_publish_root.is_none() {
                None
            } else {
                self.root_writes
                    .iter()
                    .filter(|(index, _)| *index < last_write)
                    .chain(unfinished_publish_root)
                    .map(|(_, identity)| *identity)
                    .chain(self.newest_root_in_the_base_image)
                    .max()
            };
        match highest_reachable_root {
            Some((checkpoint_txg, instance)) => text.push_str(&format!(
                "versions_up_to_txg={} instance={}\n",
                checkpoint_txg.0, instance.0
            )),
            None => text.push_str("versions_up_to=every_version\n"),
        }
        for version in flow.versions.iter().filter(|version| {
            highest_reachable_root
                .is_none_or(|highest| (version.checkpoint_txg, version.instance) <= highest)
        }) {
            text.push_str(&format!(
                "version instance={} txg={} content={}\n",
                version.instance.0,
                version.checkpoint_txg.0,
                hexadecimal_text(&sha256_digest(&version.content))
            ));
        }
        for version in flow.versions_without_a_file.iter().filter(|version| {
            highest_reachable_root.is_none_or(|highest_root| {
                (version.checkpoint_txg, version.instance) <= highest_root
            })
        }) {
            text.push_str(&format!(
                "version_without_a_file instance={} txg={}\n",
                version.instance.0, version.checkpoint_txg.0
            ));
        }
        match &self.writes_after_the_point {
            WritesAfterThePoint::ReadOnlyThroughTheUnfinishedPublish => {
                text.push_str("writes_after_the_point=read_only_through_the_unfinished_publish\n")
            }
            WritesAfterThePoint::MayAllBeRead { suffix_digests } => text.push_str(&format!(
                "writes_after_the_point={}\n",
                hexadecimal_text(&suffix_digests[last_write])
            )),
        }
        sha256_digest(text.as_bytes())
    }
}

// ============================== 判法摘要：两个判器的闭包 ==============================

/// 流水线用的判法摘要：一份、对全部崩溃点相同，取 [`judging_code_of_the_judges`]（从 CPU 判器与 GPU 判器出发按模块引用收，
/// 到进度模块与 crate 根门前停）。判法改了每个点的复用键都变、树不动；设点的测试文件、流水线自己变了，摘要不变。
#[derive(Clone, Debug)]
pub struct PipelineJudgingCode {
    code: JudgingCode,
}

impl PipelineJudgingCode {
    /// # Errors
    /// 同 [`judging_code_of_the_judges`]。
    pub fn of_the_judges(
        repository_root: &Path,
        toolchain: &crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity,
    ) -> Result<Self, JudgingCodeError> {
        Ok(Self {
            code: judging_code_of_the_judges(repository_root, toolchain)?,
        })
    }

    #[must_use]
    pub fn digest(&self) -> &[u8; DIGEST_BYTES] {
        &self.code.digest
    }

    /// 收了哪些文件、在哪停、每份为什么收。
    #[must_use]
    pub fn closure(&self) -> &JudgingCode {
        &self.code
    }

    /// 进判定行的判法版本：就是这份摘要。
    #[must_use]
    pub fn version(&self) -> JudgeVersion {
        JudgeVersion(self.code.digest)
    }

    /// 写进库里判法版本表的描述：这一版闭包收的每份文件一行（按原文进摘要的标 `raw`），停在门外的另列。
    #[must_use]
    pub fn description(&self) -> String {
        let mut text = String::new();
        for file in &self.code.files {
            if self.code.raw_files.contains(file) {
                text.push_str("raw ");
            }
            text.push_str(file);
            text.push('\n');
        }
        for file in &self.code.stopped_at {
            text.push_str("stopped ");
            text.push_str(file);
            text.push('\n');
        }
        text
    }

    /// 换掉摘要（测试里模拟「判法改了」）。
    pub fn replace_digest(&mut self, digest: [u8; DIGEST_BYTES]) {
        self.code.digest = digest;
    }
}

/// 编译期 `file!()` 给的路径归一成仓内路径：去掉 `./`，把 `..` 与它前面那一段一起消掉（`#[path = "../../x/tests/common/mod.rs"]`
/// 引进来的模块，`file!()` 带着 `..`）。消不掉的 `..`（越出仓根）原样留着，交给 [`NodeLabel::new`] 拒绝。
#[must_use]
pub fn repository_relative_file(compile_time_file: &str) -> String {
    let mut components: Vec<&str> = Vec::new();
    for component in compile_time_file.split('/') {
        match component {
            "" | "." => {}
            ".." => match components.last() {
                Some(&last) if last != ".." => {
                    components.pop();
                }
                _ => components.push(component),
            },
            _ => components.push(component),
        }
    }
    components.join("/")
}

// ============================== 跑法与分工 ==============================

/// 跑法：两个开关（配置 `multi-host.env` 的 `ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`，判法在
/// `scripts/crash/crash-point-replay-shard-configuration-check.sh`）组出四种：单 CPU、双机 CPU、单机 CPU + GPU、双机 CPU + 多 GPU。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashAmplificationRunMode {
    SingleHostCpu,
    TwoHostsCpu,
    SingleHostCpuWithGpu,
    TwoHostsCpuWithGpus,
}

impl CrashAmplificationRunMode {
    #[must_use]
    pub fn from_switches(across_machines: bool, gpu: bool) -> Self {
        match (across_machines, gpu) {
            (false, false) => Self::SingleHostCpu,
            (true, false) => Self::TwoHostsCpu,
            (false, true) => Self::SingleHostCpuWithGpu,
            (true, true) => Self::TwoHostsCpuWithGpus,
        }
    }

    /// 配置文件在就照它读；不在（第二台上跑的树副本不带 `multi-host.env`）就读同名的两个环境变量，本机的驱动脚本把它们随命令送过去。
    #[must_use]
    pub fn from_configuration_file_or_environment(path: &Path) -> Self {
        if path.is_file() {
            return Self::from_configuration_file(path);
        }
        let switch = |name: &str| {
            std::env::var(name).is_ok_and(|value| value.trim().trim_matches('"') == "1")
        };
        Self::from_switches(switch("ENABLE_ACROSS_MACHINES"), switch("ENABLE_GPU"))
    }

    /// 从 `multi-host.env` 这类 `键=值` 文件读两个开关；文件不在、键没写都按关。
    #[must_use]
    pub fn from_configuration_file(path: &Path) -> Self {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let switch = |name: &str| {
            text.lines().any(|line| {
                line.trim()
                    .strip_prefix(name)
                    .and_then(|rest| rest.strip_prefix('='))
                    .is_some_and(|value| value.trim().trim_matches('"') == "1")
            })
        };
        Self::from_switches(switch("ENABLE_ACROSS_MACHINES"), switch("ENABLE_GPU"))
    }

    #[must_use]
    pub fn uses_gpu(self) -> bool {
        matches!(self, Self::SingleHostCpuWithGpu | Self::TwoHostsCpuWithGpus)
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::SingleHostCpu => "single_host_cpu",
            Self::TwoHostsCpu => "two_hosts_cpu",
            Self::SingleHostCpuWithGpu => "single_host_cpu_with_gpu",
            Self::TwoHostsCpuWithGpus => "two_hosts_cpu_with_gpus",
        }
    }
}

/// 块怎么分给几方：按权重（一台机器的 CPU 一方，或一张卡一方、权重取空闲显存 MiB）平滑轮转，
/// 第 k 块归哪一方只由权重表与 k 定，几方各算各的、不用商量。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedShares {
    weights: Vec<u64>,
}

impl WeightedShares {
    /// # Panics
    /// 一方都没有，或权重全是 0。
    #[must_use]
    pub fn new(weights: Vec<u64>) -> Self {
        assert!(
            weights.iter().any(|weight| *weight > 0),
            "至少一方权重大于 0"
        );
        Self { weights }
    }

    /// 平滑加权轮转：前 k 块里每一方分到的块数与权重成比例，任何时刻偏差不超过一块。交回第 `block_sequence` 块的归属方。
    #[must_use]
    pub fn owner_of_block(&self, block_sequence: u64) -> usize {
        let total: u64 = self.weights.iter().sum();
        let position = block_sequence % total;
        let mut current: Vec<i128> = vec![0; self.weights.len()];
        let mut owner = 0usize;
        for _ in 0..=position {
            for (index, weight) in self.weights.iter().enumerate() {
                current[index] += i128::from(*weight);
            }
            owner = (0..current.len())
                .max_by_key(|index| (current[*index], std::cmp::Reverse(*index)))
                .expect("至少一方");
            current[owner] -= i128::from(total);
        }
        owner
    }
}

/// 显卡配置的读法住在 [`crate::gpu_card_configuration`]（只开 `gpu` 的卡池也照它挑卡），这里转出来，调用方与单测照旧经这一份引。
pub use crate::gpu_card_configuration::{
    card_priority_lines_of_the_caller, cards_by_priority_from_configuration_or_environment,
    configuration_value, configured_cards_not_usable, multi_host_configuration_file,
    parse_cards_by_priority, CardByPriority, CardHost,
};

/// 一类计算在哪算。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputedOn {
    Cpu,
    Gpu,
}

/// 这一趟每一类计算在哪算（[`computation_routes`] 定，`run_from_environment` 照它选路）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComputationRoutes {
    /// 第 ① 段的判器。
    pub crash_judge: ComputedOn,
    /// 第 ② 段读事实表的核对。
    pub facts_verifier: ComputedOn,
}

impl ComputationRoutes {
    /// 有哪一类要卡。
    #[must_use]
    pub fn need_a_card(&self) -> bool {
        [self.crash_judge, self.facts_verifier].contains(&ComputedOn::Gpu)
    }
}

/// 照本机配置定每一类计算在哪算（崩溃点重放合并第三轮判决 J5）：选路只看本机多机配置的 GPU 开关与仓内豁免表
/// （`.claude/gate.d/gpu-exemptions.tsv`：豁免的那几截在代码里标 `// gpu-exempt: <截名>`，不走这里）——没开 GPU 三类都在 CPU 上；
/// 开了 GPU 三类都走 GPU（「测试能走 GPU 的必须走 GPU」，私有显卡配置里那份登记哪几类走 GPU 的正表已删）。
///
/// # Errors
/// 开了 GPU 而这份二进制没编进 `gpu` 特性（上不了卡，不悄悄退回 CPU）。
pub fn computation_routes(
    gpu_is_switched_on: bool,
    gpu_feature_is_compiled_in: bool,
) -> Result<ComputationRoutes, String> {
    if !gpu_is_switched_on {
        return Ok(ComputationRoutes {
            crash_judge: ComputedOn::Cpu,
            facts_verifier: ComputedOn::Cpu,
        });
    }
    if !gpu_feature_is_compiled_in {
        return Err(
            "配置写了 ENABLE_GPU=1，判器、核对与单元校验和都要走 GPU，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）"
                .to_string(),
        );
    }
    Ok(ComputationRoutes {
        crash_judge: ComputedOn::Gpu,
        facts_verifier: ComputedOn::Gpu,
    })
}

/// 配置开了 GPU 而这份二进制没编进 gpu 特性时判段报本次未跑的原因（用户 2026-10-13 定「配 GPU 必用 GPU」：提交时快档不带特性那一趟
/// 不因为配置开了 GPU 就拒跑，也不退回 CPU 判；录入段照录，带特性那一趟从库里读回在卡上判）；配置没开 GPU、或特性编进来了是 `None`。
#[must_use]
pub fn judge_not_run_without_the_gpu_feature(
    gpu_is_switched_on: bool,
    gpu_feature_is_compiled_in: bool,
) -> Option<String> {
    (gpu_is_switched_on && !gpu_feature_is_compiled_in).then(|| {
        "配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性：判段这一趟不跑（录入段照录、写进库），带 gpu 特性那一趟从库里读回在卡上判（cargo … --features verdict-store,gpu）"
            .to_string()
    })
}

/// 这份二进制在本机多机配置下判段跑不跑：配置开了 GPU 而没编进 gpu 特性时交回原因（[`judge_not_run_without_the_gpu_feature`]），
/// 跑得了是 `None`。在进程里拿给定状态表的结论接着往下走的装置与用例开头调它：交回原因就打本次未跑那一行、提前返回，
/// 不在占位结论上 panic 或断言（判归带 gpu 特性那一趟）。
///
/// # Errors
/// 多机配置读不出（[`multi_host_configuration_file`]）。
pub fn judge_not_run_in_this_binary(repository_root: &Path) -> Result<Option<String>, String> {
    let configuration_file = multi_host_configuration_file(repository_root)?;
    let mode = match &configuration_file {
        Some(configuration) => CrashAmplificationRunMode::from_configuration_file(configuration),
        None => CrashAmplificationRunMode::from_configuration_file_or_environment(
            &repository_root.join("multi-host.env"),
        ),
    };
    Ok(judge_not_run_without_the_gpu_feature(
        mode.uses_gpu(),
        cfg!(feature = "gpu"),
    ))
}

/// 在进程里拿给定状态表的结论往下判的用例（装置的内联单测、checker 档集成测试）开头调：这份二进制在本机配置下判段不跑
/// （[`judge_not_run_in_this_binary`]）就打门禁认的本次未跑那一行、交回 `true`，用例随即返回；判得了交回 `false`。
/// 多机配置读不出直接 panic（配置写坏了不该当成没开 GPU）。
#[must_use]
pub fn judging_test_not_run_without_the_gpu_feature(test_name: &str) -> bool {
    let repository_root = crate::crash_library::repository_root_of_this_crate();
    match judge_not_run_in_this_binary(&repository_root) {
        Ok(Some(reason)) => {
            println!(
                "{}",
                singlefs_harness::gpu_memory_ledger::gpu_not_run_line(test_name, &reason)
            );
            true
        }
        Ok(None) => false,
        Err(refusal) => panic!("读不出多机配置，判不了这份二进制的判段跑不跑：{refusal}"),
    }
}

/// 判段报本次未跑那一趟打的两行（[`judge_not_run_without_the_gpu_feature`]）：崩溃放量入口认的 `CRASH_AMPLIFICATION_NOT_RUN step=judge`，
/// 与门禁快档认的 GPU 用例那一族记号（`singlefs_harness::gpu_memory_ledger::gpu_not_run_line`）。入口与门禁记本次未跑、不算通过、不判红。
#[must_use]
pub fn not_run_lines_without_the_gpu_feature(flow_name: &str, reason: &str) -> Vec<String> {
    vec![
        format!(
            "{CRASH_AMPLIFICATION_NOT_RUN_LINE_PREFIX} step=judge test=crash_amplification::run_from_environment flow={flow_name} reason={reason}"
        ),
        singlefs_harness::gpu_memory_ledger::gpu_not_run_line(
            "crash_amplification::run_from_environment",
            reason,
        ),
    ]
}

/// 给定状态表的引擎判段报本次未跑那一行（[`judge_not_run_without_the_gpu_feature`]）：门禁快档认的 GPU 用例那一族记号
/// （`singlefs_harness::gpu_memory_ledger::gpu_not_run_line`，`NOT_RUN_<原因> test=<用例> reason=<一句>`），用例是引擎这一项的名字。
/// 不打 `GIVEN_STATES_JUDGE route=cpu`：判段不跑，也不退回 CPU 判。
#[must_use]
pub fn given_states_not_run_line_without_the_gpu_feature(item_name: &str, reason: &str) -> String {
    singlefs_harness::gpu_memory_ledger::gpu_not_run_line(
        &format!("given_states_engine::{item_name}"),
        reason,
    )
}

/// 崩溃放量本次未跑那一行的头（入口 `scripts/crash/crash-amplification-item.sh` 与双机驱动按它认）。
pub const CRASH_AMPLIFICATION_NOT_RUN_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_NOT_RUN";

/// 观察者交回的盘上的池级 checker 走哪一路（照第 ① 段的选路：配了 GPU 在卡上判，没配由 CPU 判）。
///
/// # Errors
/// 选路走 GPU 而这一趟没有起好的卡；这份二进制没编进 gpu 特性。
pub(crate) fn pool_checker_of_the_images_after_the_observers(
    route: ComputedOn,
    cards: Option<&CardsInUse>,
    taken_from: GpuCardsSource,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    match route {
        ComputedOn::Cpu => Ok(PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu),
        ComputedOn::Gpu => pool_checker_of_the_images_on_the_gpu(cards, taken_from),
    }
}

#[cfg(feature = "gpu")]
fn pool_checker_of_the_images_on_the_gpu(
    cards: Option<&CardsInUse>,
    taken_from: GpuCardsSource,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    let cards = cards.ok_or_else(|| {
        "配置开了 GPU，观察者交回的盘上的池级 checker 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器"
            .to_string()
    })?;
    Ok(PoolCheckerOfTheImagesAfterTheObservers::OnTheGpu {
        cards: cards.clone(),
        taken_from,
    })
}

/// 没编进 `gpu` 特性：选路走不到 GPU（[`computation_routes`] 先报错，判段报本次未跑的那一趟不判）。
///
/// # Errors
/// 恒报错。
#[cfg(not(feature = "gpu"))]
fn pool_checker_of_the_images_on_the_gpu(
    _cards: Option<&CardsInUse>,
    _taken_from: GpuCardsSource,
) -> Result<PoolCheckerOfTheImagesAfterTheObservers, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性：观察者交回的盘上的池级 checker 上不了卡（选路不该走到这里）".to_string())
}

/// 按优先级把 `pending_states` 个状态装到卡上：逐张装到这张卡一次派活带得了的状态数为止，装不下的才给下一张。
/// 交回每张卡装了多少，0 就是这一趟不用这张卡；每张卡都装满还有剩的，剩下的排队等前面的判完（不算在任何一张卡上）。
#[must_use]
pub fn place_states_on_cards_by_priority(pending_states: u64, capacities: &[u64]) -> Vec<u64> {
    let mut remaining = pending_states;
    capacities
        .iter()
        .map(|capacity| {
            let placed = remaining.min(*capacity);
            remaining -= placed;
            placed
        })
        .collect()
}

// ============================== 录入 ==============================

/// 录入的结果：这一趟录进库的块数、库里已有判定（复用）的块数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordingTally {
    pub blocks_recorded: u64,
    pub blocks_already_judged: u64,
    pub states_recorded: u64,
    pub states_already_judged: u64,
}

/// 录入：每个点的每一块登进录入表，路径 → 身份登进路径索引表；库里已经有同一块键在这一判法版本下的判定块就不再核
/// （别的判法版本判过的不算：旧行留着，这一版从头判）。
///
/// # Errors
/// 库读写失败。
pub fn record_crash_points(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    version: &JudgeVersion,
    report: &mut CoverageReport,
) -> Result<RecordingTally, VerdictStoreError> {
    record_crash_points_needing(
        store,
        plan,
        &[ConclusionsStoredUnder::TheJudgeVersion(version)],
        report,
    )
}

/// 同 [`record_crash_points`]，一块算复用要 `needed` 里每一处都有结论（三种结论那一套「哪一种没有就只排那一种」，第三轮判决 J8）；
/// 先写（[`write_the_recorded_blocks`]）再数（[`tally_the_recorded_blocks`]）。
///
/// # Errors
/// 库读写失败。
pub fn record_crash_points_needing(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    needed: &[ConclusionsStoredUnder<'_>],
    report: &mut CoverageReport,
) -> Result<RecordingTally, VerdictStoreError> {
    write_the_recorded_blocks(store, plan)?;
    tally_the_recorded_blocks(store, plan, needed, report)
}

/// 录入段写录入表与路径索引：每个点的每一块登一行，路径 → 身份登一行。
///
/// # Errors
/// 库写失败。
pub fn write_the_recorded_blocks(
    store: &mut VerdictStore,
    plan: &CrashPlan,
) -> Result<(), VerdictStoreError> {
    for point in &plan.points {
        store.record_path(&point.path_number, &point.identity)?;
        for local in point.blocks() {
            // 分片的段的块锚在段内编号上：上一趟录了、没判完就停、两趟之间片大小改了，同一块键的块长跟着变
            if point.is_a_block_of_a_sliced_segment(&local) {
                store.record_block_of_a_slice(&point.block_key(&local), local.end - local.start)?;
            } else {
                store.record_block(&point.block_key(&local), local.end - local.start)?;
            }
        }
    }
    Ok(())
}

/// 判段只读：照计划数录入的块里哪些已有 `needed` 每一处的结论（复用）、哪些要判，覆盖报告逐点记。不往库里写。
///
/// # Errors
/// 库读失败。
pub fn tally_the_recorded_blocks(
    store: &VerdictStore,
    plan: &CrashPlan,
    needed: &[ConclusionsStoredUnder<'_>],
    report: &mut CoverageReport,
) -> Result<RecordingTally, VerdictStoreError> {
    let mut tally = RecordingTally::default();
    for (point_index, point) in plan.points.iter().enumerate() {
        let mut point_reused = true;
        for local in point.blocks() {
            let key = point.block_key(&local);
            let count = local.end - local.start;
            let mut every_needed_conclusion_is_stored = true;
            for conclusions in needed {
                if !conclusions.is_stored(store, point_index, &key)? {
                    every_needed_conclusion_is_stored = false;
                }
            }
            if every_needed_conclusion_is_stored {
                tally.blocks_already_judged += 1;
                tally.states_already_judged += count;
            } else {
                point_reused = false;
                tally.blocks_recorded += 1;
                tally.states_recorded += count;
            }
        }
        report.record(
            &point.path_number,
            NodeLabel::new(point_code_file(point), &point.name).expect("计划里的点合规矩"),
            &point.reuse_key,
            if point.ignored {
                CoverageState::Ignored
            } else if point.blocks().is_empty() {
                CoverageState::NotExpanded
            } else if point_reused {
                CoverageState::Reused
            } else {
                CoverageState::Checked
            },
        );
    }
    Ok(tally)
}

fn point_code_file(point: &PlannedCrashPoint) -> &str {
    point
        .identity
        .split("::")
        .next()
        .expect("身份以仓内路径起头")
}

// ============================== 核对 ==============================

// ============================== 结论记在库里哪一处（崩溃点重放合并第三轮判决 J7、J8） ==============================

/// 一块的结论记在库里哪一处。
#[derive(Clone, Debug)]
pub enum ConclusionsStoredUnder<'versions> {
    /// 今天的判法版本那一套：两个判器合起来一把，结论按（块键，判法版本）存，只存判定与违例。直接调流水线各段的对拍用例与单测用它；
    /// [`run_from_environment`] 不用。
    TheJudgeVersion(&'versions JudgeVersion),
    /// 三种结论里的一种：判定库按（块键，结论种类，版本）存，计数向量、恢复字与在哪判的随块存。每个点各一把版本，与计划的点同序
    /// （GPU 判器每个点同一把，CPU 判器按设点文件各一把，[`ConclusionVersionsOfThePlan`]）。
    TheConclusionKind {
        kind: ConclusionKind,
        version_of_each_point: &'versions [ConclusionVersion],
    },
}

/// 随一块判定一起存的（三种结论那一套才存进库）：计数向量、每个状态的恢复字（CPU 判器没编进 `gpu` 特性时没有）、在哪判的。
struct StoredAlongTheVerdicts {
    count_vector: BlockCountVector,
    recovery_words: Option<Vec<RecoveryWordsOfState>>,
    judged_on: JudgingDevice,
}

impl ConclusionsStoredUnder<'_> {
    /// 第 `point_index` 个点那一把版本。
    ///
    /// # Panics
    /// 版本的个数与计划的点数不等（建版本时照计划建，见 [`ConclusionVersionsOfThePlan::of_the_flow`]）。
    fn conclusion_version_of_the_point(
        version_of_each_point: &[ConclusionVersion],
        point_index: usize,
    ) -> ConclusionVersion {
        *version_of_each_point.get(point_index).expect(
            "每个点各一把版本、与计划的点同序：ConclusionVersionsOfThePlan::of_the_flow 照计划的点建",
        )
    }

    /// 这一块在这一处有没有结论。
    fn is_stored(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        match self {
            Self::TheJudgeVersion(version) => Ok(store.read_verdict_block(key, version)?.is_some()),
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => Ok(store
                .conclusions_missing_for_block(
                    key,
                    &[(
                        kind.clone(),
                        Self::conclusion_version_of_the_point(version_of_each_point, point_index),
                    )],
                )?
                .is_empty()),
        }
    }

    /// 写一块：判法版本那一套只写判定与违例，三种结论那一套连同计数向量、恢复字与在哪判的一批写下去。
    fn store_block(
        &self,
        store: &mut VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
        block: VerdictBlock,
        violating: Vec<StateOffsetInBlock>,
        along: StoredAlongTheVerdicts,
    ) -> Result<(), VerdictStoreError> {
        match self {
            Self::TheJudgeVersion(version) => {
                store.store_verdict_block(key, version, &block, &violating)?;
                Ok(())
            }
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => {
                store.store_conclusion_block(
                    key,
                    kind,
                    &Self::conclusion_version_of_the_point(version_of_each_point, point_index),
                    &ConclusionBlockRecord {
                        block,
                        violating_states: violating,
                        count_vector: along.count_vector,
                        judged_on: along.judged_on,
                        recovery_words: along.recovery_words,
                    },
                )?;
                Ok(())
            }
        }
    }

    /// 这一块在这一处每个状态的判定向量编号；没有结论是 `None`。
    fn verdict_numbers_of_each_state(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<Option<Vec<VerdictVectorNumber>>, VerdictStoreError> {
        match self {
            Self::TheJudgeVersion(version) => Ok(store
                .read_verdict_block(key, version)?
                .map(|block| block.verdict_vector_number_of_each_state().to_vec())),
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => Ok(store
                .read_conclusion_block(
                    key,
                    kind,
                    &Self::conclusion_version_of_the_point(version_of_each_point, point_index),
                )?
                .map(|stored| stored.block.verdict_vector_number_of_each_state().to_vec())),
        }
    }

    /// 这一块在这一处的违例，按块内序号；没有结论是空的。
    fn violations_in_block(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<Vec<StoredViolation>, VerdictStoreError> {
        match self {
            Self::TheJudgeVersion(version) => store.violations_in_block(key, version),
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => Ok(store
                .read_conclusion_block(
                    key,
                    kind,
                    &Self::conclusion_version_of_the_point(version_of_each_point, point_index),
                )?
                .map_or_else(Vec::new, |stored| stored.violations)),
        }
    }

    /// 汇总行里这一处的版本：判法版本那一套是它的头 8 字节；三种结论那一套是种类连每把不同的版本的头 8 字节（按点的次序、去重）。
    #[must_use]
    pub fn version_text(&self) -> String {
        let head = |bytes: &[u8]| hexadecimal_text(&bytes[..8]);
        match self {
            Self::TheJudgeVersion(version) => head(&version.0),
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => {
                let mut distinct: Vec<String> = Vec::new();
                for version in version_of_each_point.iter() {
                    let text = head(&version.0);
                    if !distinct.contains(&text) {
                        distinct.push(text);
                    }
                }
                format!("{}:{}", kind.version_text(), distinct.join(","))
            }
        }
    }
}

/// 核对一趟的结果。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckingTally {
    pub blocks_checked: u64,
    pub states_checked: u64,
    pub red_states: u64,
    /// 每个红状态的违例正文（点名、块内序号、正文），给门禁与人看；只收前 [`RED_TEXTS_KEPT`] 条。
    pub red_texts: Vec<(String, u64, String)>,
}

pub const RED_TEXTS_KEPT: usize = 64;

/// 核对线程数的环境变量。
pub const CHECKING_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_THREADS";

/// 核对线程数：环境变量 [`CHECKING_THREADS_ENVIRONMENT_VARIABLE`]，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取
/// `.claude/heavy-thread-default` 的登记值；`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）。
///
/// # Panics
/// 同 [`checking_threads_from_the_variable_value`]；值不是 UTF-8 也 panic。
#[must_use]
pub fn checking_threads() -> usize {
    checking_threads_from_the_variable_value(
        process_environment_variable(CHECKING_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// 核对线程数变量的值（`None` 是没设）换成线程数：设了照它，没设取每台默认重活线程数（`thread_cap_value` 是 `SINGLEFS_THREAD_CAP`
/// 的值，判法在 `singlefs_harness::heavy_threads`）；解析经数值环境变量共用的那一份，线程数取 `NonZeroUsize`，与仓里别的线程数变量
/// 同一口径（C606：写错、写 0 都停下，不悄悄退回默认）。
///
/// # Panics
/// 设了而不是正整数（含 0、空串）：消息带变量名与原值；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
#[must_use]
pub fn checking_threads_from_the_variable_value(
    value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> usize {
    own_variable_or_default_heavy_threads(
        CHECKING_THREADS_ENVIRONMENT_VARIABLE,
        value,
        thread_cap_value,
    )
    .get()
}

/// 一块判完的结果（还没写库）：每个状态判红的各项、红状态的正文。
struct JudgedBlock {
    local: Range<u64>,
    items_of_each_state: Vec<Vec<String>>,
    /// 逐状态的恢复字（`CrashStateJudgement::recovery_words`，判器一侧 `crate::crash_judge_conclusions`；不挂 `gpu` 特性，每个状态都有）。
    recovery_words_of_each_state: Vec<RecoveryWordsOfState>,
    state_counts: StateCountsOfABlock,
    red_texts: Vec<(u64, String)>,
}

fn judge_one_block(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    point_index: usize,
    local: Range<u64>,
) -> JudgedBlock {
    let point = &plan.points[point_index];
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let mut items_of_each_state =
        Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
    // 恢复字的失败点码表在判器一侧、不挂 gpu 特性（`crate::crash_judge_conclusions`）：没编进卡的二进制判的 CPU 结论照样带恢复字
    let mut recovery_words_of_each_state: Vec<RecoveryWordsOfState> =
        Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
    let mut state_counts = StateCountsOfABlock::default();
    let mut red_texts = Vec::new();
    let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
    let stream_state_count = judge_crash_state_range_of_the_read_back_flow_with_slices(
        flow.base,
        flow.writes,
        flow.segments,
        flow.judged_root_index,
        flow.version_table(),
        &expansion,
        plan.segment_slices(),
        ordinals,
        &mut |ordinal, publish, image, judgement| {
            let items: Vec<String> = judgement.red_items();
            if !items.is_empty() && red_texts.len() < RED_TEXTS_KEPT {
                red_texts.push((
                    ordinal - point.ordinals.start,
                    judgement.violation_text(image),
                ));
            }
            items_of_each_state.push(items);
            state_counts.count_the_cpu_judgement(judgement);
            state_counts.count_the_publish_of_a_state(publish);
            recovery_words_of_each_state.push(judgement.recovery_words());
        },
    );
    assert!(
        stream_state_count >= point.ordinals.end,
        "这一块的序号越过了整条流的状态数"
    );
    eprintln!(
        "CRASH_AMPLIFICATION_BLOCK point={} block_start={} states={} red={}",
        point.name,
        local.start,
        local.end - local.start,
        items_of_each_state
            .iter()
            .filter(|items| !items.is_empty())
            .count()
    );
    JudgedBlock {
        local,
        items_of_each_state,
        recovery_words_of_each_state,
        state_counts,
        red_texts,
    }
}

/// 核对录入表里还没有判定的块：按分工只核 `owner` 那一方分到的；一块切成几段多线程判（[`checking_threads`]），
/// 每块在 CPU 上逐状态判（单元两道校验和也在 CPU 上，由池级 checker 判，这一段不对拍），
/// 判完一块就按块的次序单线程写回判定块与违例——判定向量的编号按块序登记，与线程数无关，输出与单线程逐字相同。
/// 同一块两方不会同时领：块的归属由权重表与块序号定死。
///
/// # Errors
/// 库读写失败；判定向量超过 256 种。
///
/// # Panics
/// 某个核对线程 panic（判状态的代码自己的断言）。
pub fn check_recorded_blocks(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    shares: &WeightedShares,
    owner: usize,
    version: &JudgeVersion,
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where(
        store,
        flow,
        plan,
        &|block_sequence| shares.owner_of_block(block_sequence) == owner,
        version,
    )
}

/// 核对 `is_mine` 判为自己的那些块（块序号 = 全部点的块按次序编号）；多线程与写库的做法见 [`check_recorded_blocks`]。
///
/// # Errors
/// 库读写失败；判定向量超过 256 种。
pub fn check_blocks_where(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    version: &JudgeVersion,
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where_until(
        store,
        flow,
        plan,
        is_mine,
        &ConclusionsStoredUnder::TheJudgeVersion(version),
        &|| false,
    )
}

/// 停下的请求：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE` 指的文件在，就不再领新的块（手里那一块判完、写进库），这一趟提早收尾。
/// 给「本机先判着、等别的机器准备好再重新分工」用：判过的块在库里，下一趟接着判。
#[must_use]
pub fn stop_is_requested_by_the_environment() -> bool {
    stop_file_of_the_environment().is_some_and(|path| path.exists())
}

/// 环境变量 `SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE` 指的叫停文件：读这个变量的只有这一处。
fn stop_file_of_the_environment() -> Option<PathBuf> {
    std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE").map(PathBuf::from)
}

/// 同 [`check_blocks_where`]，每领一块之前先问 `stop_is_requested`：要停就不再领，已经判完的块都写进了库。
///
/// # Errors
/// 同 [`check_blocks_where`]。
pub fn check_blocks_where_until(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    conclusions: &ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where_until_with_threads(
        store,
        flow,
        plan,
        is_mine,
        conclusions,
        stop_is_requested,
        checking_threads(),
    )
}

/// 同 [`check_blocks_where_until`]，核对线程数由调用方给（崩溃枚举用例的判段照 `SINGLEFS_CRASH_POINT_REPLAY_THREADS`）。
///
/// # Errors
/// 库读写失败；判定向量超过 256 种。
#[allow(
    clippy::too_many_arguments,
    reason = "与 check_blocks_where_until 同一组参数，多一个线程数"
)]
pub fn check_blocks_where_until_with_threads(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    conclusions: &ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
    threads: usize,
) -> Result<CheckingTally, VerdictStoreError> {
    let pending = pending_blocks(store, plan, conclusions, is_mine)?;
    let points: Vec<NamesOfACrashPoint> = plan
        .points
        .iter()
        .map(|point| NamesOfACrashPoint {
            name: point.name.clone(),
            path_number: point.path_number.clone(),
        })
        .collect();
    check_pending_blocks_in_pieces(
        &pending,
        &points,
        threads,
        stop_is_requested,
        &|point_index, local| judge_one_block(flow, plan, point_index, local),
        &mut |point_index, local, checked| {
            let mut numbers = Vec::with_capacity(checked.items_of_each_state.len());
            for items in &checked.items_of_each_state {
                numbers
                    .push(store.register_verdict_vector(&encoded_verdict_of_the_red_items(items))?);
            }
            let verdict_block = VerdictBlock::new(numbers).expect("一块至少一个状态");
            let recovery_words = (!checked.recovery_words_of_each_state.is_empty())
                .then_some(checked.recovery_words_of_each_state);
            conclusions.store_block(
                store,
                point_index,
                &plan.points[point_index].block_key(local),
                verdict_block,
                checked.violating,
                StoredAlongTheVerdicts {
                    count_vector: checked
                        .state_counts
                        .into_count_vector(local.end - local.start),
                    recovery_words,
                    judged_on: JudgingDevice::Cpu,
                },
            )
        },
    )
}

// ============================== 取样那一趟的 CPU 结论入库（用户 2026-10-13 定「能走两段的都走两段」） ==============================

/// 取样（`crate::crash_state_enumeration::sample_crash_states_on_the_cpu` 一族：实验装置从库里读回流、逐状态取样）那一趟在 CPU 上判过的状态里，
/// 整块都判到的块照 CPU 判器那一处（[`ConclusionsStoredUnder`]，`JudgedBy::CpuJudge`）存进库；块有一部分在取样序号段之外的不存（一块是库的
/// 原子单位，半块不存），库里这一处已经存着的块不再存。装置要的恢复报告字段库里的结论不装，判照旧逐状态现判、交回装置。
pub(crate) struct CpuJudgementsOfSampledBlocks<'plan> {
    plan: &'plan CrashPlan,
    conclusions: ConclusionsStoredUnder<'plan>,
    blocks: Vec<SampledBlock>,
    pub(crate) blocks_stored_here: u64,
    pub(crate) blocks_already_stored: u64,
}

/// 取样序号段整块罩住、还没存的一块：每个状态判到了就记下它的红项与恢复字。
struct SampledBlock {
    point_index: usize,
    local: Range<u64>,
    first_ordinal: u64,
    items_of_each_state: Vec<Option<Vec<String>>>,
    recovery_words_of_each_state: Vec<Option<RecoveryWordsOfState>>,
    state_counts: StateCountsOfABlock,
    states_noted: u64,
}

impl<'plan> CpuJudgementsOfSampledBlocks<'plan> {
    /// 计划里整块落在 `ordinals` 里、库里这一处还没存的块。
    ///
    /// # Errors
    /// 库读失败。
    pub(crate) fn for_the_ordinals(
        store: &VerdictStore,
        plan: &'plan CrashPlan,
        conclusions: ConclusionsStoredUnder<'plan>,
        ordinals: &Range<u64>,
    ) -> Result<Self, VerdictStoreError> {
        let mut blocks = Vec::new();
        let mut blocks_already_stored = 0u64;
        for (point_index, point) in plan.points.iter().enumerate() {
            for local in point.blocks() {
                let first_ordinal = point.ordinals.start + local.start;
                let end_ordinal = point.ordinals.start + local.end;
                if first_ordinal == end_ordinal
                    || first_ordinal < ordinals.start
                    || end_ordinal > ordinals.end
                {
                    continue;
                }
                if conclusions.is_stored(store, point_index, &point.block_key(&local))? {
                    blocks_already_stored += 1;
                    continue;
                }
                let state_count =
                    usize::try_from(local.end - local.start).expect("一块装得进 usize");
                blocks.push(SampledBlock {
                    point_index,
                    local,
                    first_ordinal,
                    items_of_each_state: vec![None; state_count],
                    recovery_words_of_each_state: vec![None; state_count],
                    state_counts: StateCountsOfABlock::default(),
                    states_noted: 0,
                });
            }
        }
        Ok(Self {
            plan,
            conclusions,
            blocks,
            blocks_stored_here: 0,
            blocks_already_stored,
        })
    }

    /// 取样判过的一个状态记进来；它那一块的状态都判到了就存进库。不在任何一块里（块有一部分在序号段外、或库里已存）的状态不记。
    ///
    /// # Errors
    /// 库写失败；判定向量超过 256 种。
    pub(crate) fn note_a_state(
        &mut self,
        store: &mut VerdictStore,
        ordinal: u64,
        publish: CrashPointReplayPublishOfState,
        judgement: &crate::crash::CrashStateJudgement,
    ) -> Result<(), VerdictStoreError> {
        let position = self.blocks.partition_point(|block| {
            block.first_ordinal + (block.local.end - block.local.start) <= ordinal
        });
        let Some(block) = self.blocks.get_mut(position) else {
            return Ok(());
        };
        if ordinal < block.first_ordinal {
            return Ok(());
        }
        let offset = usize::try_from(ordinal - block.first_ordinal).expect("块内序号装得进 usize");
        if block.items_of_each_state[offset].is_some() {
            return Ok(());
        }
        block.items_of_each_state[offset] = Some(judgement.red_items());
        block.recovery_words_of_each_state[offset] = Some(judgement.recovery_words());
        block.state_counts.count_the_cpu_judgement(judgement);
        block.state_counts.count_the_publish_of_a_state(publish);
        block.states_noted += 1;
        if block.states_noted < block.local.end - block.local.start {
            return Ok(());
        }
        let items_of_each_state: Vec<Vec<String>> = std::mem::take(&mut block.items_of_each_state)
            .into_iter()
            .map(|items| items.expect("这一块每个状态都记过"))
            .collect();
        let recovery_words: Vec<RecoveryWordsOfState> =
            std::mem::take(&mut block.recovery_words_of_each_state)
                .into_iter()
                .map(|words| words.expect("这一块每个状态都记过"))
                .collect();
        let mut numbers = Vec::with_capacity(items_of_each_state.len());
        let mut violating = Vec::new();
        for (offset_in_the_block, items) in items_of_each_state.iter().enumerate() {
            if !items.is_empty() {
                violating.push(StateOffsetInBlock(
                    u64::try_from(offset_in_the_block).expect("块内序号装得进 u64"),
                ));
            }
            numbers.push(store.register_verdict_vector(&encoded_verdict_of_the_red_items(items))?);
        }
        let point_index = block.point_index;
        let local = block.local.clone();
        let state_counts = std::mem::take(&mut block.state_counts);
        self.conclusions.store_block(
            store,
            point_index,
            &self.plan.points[point_index].block_key(&local),
            VerdictBlock::new(numbers).expect("一块至少一个状态"),
            violating,
            StoredAlongTheVerdicts {
                count_vector: state_counts.into_count_vector(local.end - local.start),
                recovery_words: Some(recovery_words),
                judged_on: JudgingDevice::Cpu,
            },
        )?;
        self.blocks_stored_here += 1;
        Ok(())
    }
}

/// [`check_pending_blocks_in_pieces`] 写一块的那一份：点在 `points` 里的次序、块内序号、拼好的一块。
type StoreACheckedBlock<'store> =
    dyn FnMut(usize, &Range<u64>, CheckedBlock) -> Result<(), VerdictStoreError> + 'store;

/// 各段拼好的一块：逐状态红项、红状态的块内序号、逐状态恢复字（没算是空的）、逐条不变量判到与违例的状态数。
struct CheckedBlock {
    items_of_each_state: Vec<Vec<String>>,
    violating: Vec<StateOffsetInBlock>,
    recovery_words_of_each_state: Vec<RecoveryWordsOfState>,
    state_counts: StateCountsOfABlock,
}

/// 一个崩溃点的名字与路径编号（核对时打进度行、记红状态的正文用）。
struct NamesOfACrashPoint {
    name: String,
    path_number: String,
}

/// [`check_blocks_where_until`] 的那一圈（不碰库与卡：判一段交 `judge_a_piece`，写一块交 `store_a_block`，单测用假的）：`pending` 里的块按次序，
/// 每领一块之前先问 `stop_is_requested`；一块的序号切成 `threads` 那么多段、一段一个线程交 `judge_a_piece`，
/// 收齐按段序拼回一块、当场交 `store_a_block`（点在 `points` 里的次序、块内序号、逐状态红项、红状态的块内序号）。
///
/// # Errors
/// `store_a_block` 的错。
///
/// # Panics
/// 判一段的线程 panic。
fn check_pending_blocks_in_pieces(
    pending: &[(usize, Range<u64>)],
    points: &[NamesOfACrashPoint],
    threads: usize,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
    judge_a_piece: &(dyn Fn(usize, Range<u64>) -> JudgedBlock + Sync),
    store_a_block: &mut StoreACheckedBlock<'_>,
) -> Result<CheckingTally, VerdictStoreError> {
    // 块是库与 GPU 批的粒度，不改小；一块的序号切成线程数那么多段、一段一个线程（一块 65536 个状态、32 线程时一段 2048 个），
    // 判完按段序拼回一块、当场写库：中途被停，写进库的块下一次不再判。
    let mut tally = CheckingTally::default();
    // 每一段判完打一行 `CRASH_AMPLIFICATION_BLOCK`；一段判得比进度间隔久时，等着收的这条线程隔一个间隔另打一行按挂钟的
    let started = std::time::Instant::now();
    let clock = || started.elapsed();
    let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
    for (pending_index, (point_index, local)) in pending.iter().enumerate() {
        if stop_is_requested() {
            break;
        }
        let point = &points[*point_index];
        let piece_length = (local.end - local.start)
            .div_ceil(u64::try_from(threads).expect("段数装得进 u64"))
            .max(1);
        let mut pieces: Vec<Range<u64>> = Vec::new();
        let mut start = local.start;
        while start < local.end {
            let end = (start + piece_length).min(local.end);
            pieces.push(start..end);
            start = end;
        }
        let piece_count = pieces.len();
        let mut judged: Vec<JudgedBlock> = std::thread::scope(|scope| {
            let (judged_sender, judged_receiver) = std::sync::mpsc::channel::<JudgedBlock>();
            for range in pieces.iter().cloned() {
                let judged_sender = judged_sender.clone();
                scope.spawn(move || {
                    let piece = judge_a_piece(*point_index, range);
                    // 发不出去说明收的这条线程已经走了（它 panic 了），这一段的结果没人要
                    let _ = judged_sender.send(piece);
                });
            }
            // 只留核对线程手里的发送端：它们都退出之后（判完或 panic），下面的接收才结束；有线程 panic 时 scope 收尾把它往外传
            drop(judged_sender);
            receive_the_pieces_of_one_block(
                &judged_receiver,
                &mut wall_clock_lines,
                &clock,
                &BlockInProgress {
                    point_name: &point.name,
                    block_start: local.start,
                    piece_count,
                    blocks_finished: tally.blocks_checked,
                    pending_blocks: pending.len(),
                    states_finished: tally.states_checked,
                },
                &mut |line| eprintln!("{line}"),
            )
        });
        assert_eq!(
            judged.len(),
            piece_count,
            "块 {pending_index} 的每一段都判完交回了（核对线程 panic 时上面的 scope 已经往外传）"
        );
        judged.sort_by_key(|piece| piece.local.start);
        let mut items_of_each_state: Vec<Vec<String>> =
            Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
        let mut recovery_words_of_each_state: Vec<RecoveryWordsOfState> = Vec::new();
        let mut state_counts = StateCountsOfABlock::default();
        let mut next_expected = local.start;
        for piece in judged {
            assert_eq!(
                piece.local.start, next_expected,
                "块 {pending_index} 的段没有接上"
            );
            next_expected = piece.local.end;
            items_of_each_state.extend(piece.items_of_each_state);
            recovery_words_of_each_state.extend(piece.recovery_words_of_each_state);
            state_counts.absorb(piece.state_counts);
            for (state_in_point, text) in piece.red_texts {
                if tally.red_texts.len() < RED_TEXTS_KEPT {
                    tally
                        .red_texts
                        .push((point.path_number.clone(), state_in_point, text));
                }
            }
        }
        assert_eq!(next_expected, local.end, "块 {pending_index} 的段没有拼满");
        let mut violating = Vec::new();
        for (offset, items) in items_of_each_state.iter().enumerate() {
            if !items.is_empty() {
                tally.red_states += 1;
                violating.push(StateOffsetInBlock(
                    u64::try_from(offset).expect("块内序号装得进 u64"),
                ));
            }
        }
        store_a_block(
            *point_index,
            local,
            CheckedBlock {
                items_of_each_state,
                violating,
                recovery_words_of_each_state,
                state_counts,
            },
        )?;
        tally.blocks_checked += 1;
        tally.states_checked += local.end - local.start;
    }
    Ok(tally)
}

/// 一块正在判的时候按挂钟那一行要报的：哪个点、块从点内第几个状态起、切成几段，这一趟已经判完几块（共几块）与几个状态。
struct BlockInProgress<'point> {
    point_name: &'point str,
    block_start: u64,
    piece_count: usize,
    blocks_finished: u64,
    pending_blocks: usize,
    states_finished: u64,
}

/// 收齐一块各段的判定：收到一段就记下；上一行之后 `clock` 走过整个进度间隔还没有新的一段，就交 `write_progress_line` 一行
/// `CRASH_AMPLIFICATION_BLOCK by=wall_clock …`（C611：一块判得比进度间隔久时照样隔一个间隔报一行）。核对线程都退出（发送端全丢掉）才交回，
/// 交回的是收到的各段（次序是到达的次序）。
fn receive_the_pieces_of_one_block(
    receiver: &std::sync::mpsc::Receiver<JudgedBlock>,
    wall_clock_lines: &mut WallClockProgressLines,
    clock: &dyn Fn() -> std::time::Duration,
    block: &BlockInProgress<'_>,
    write_progress_line: &mut dyn FnMut(String),
) -> Vec<JudgedBlock> {
    let mut arrived: Vec<JudgedBlock> = Vec::with_capacity(block.piece_count);
    receive_until_the_senders_are_gone_with_wall_clock_lines(
        receiver,
        wall_clock_lines,
        clock,
        &mut |event| {
            match event {
            ReceivedOrWallClockLineDue::Received(piece) => arrived.push(piece),
            ReceivedOrWallClockLineDue::WallClockLineDue { elapsed } => write_progress_line(format!(
                "CRASH_AMPLIFICATION_BLOCK by=wall_clock point={} block_start={} pieces_finished={}/{} blocks_finished={}/{} states_finished={} elapsed_seconds={:.1}",
                block.point_name,
                block.block_start,
                arrived.len(),
                block.piece_count,
                block.blocks_finished,
                block.pending_blocks,
                block.states_finished,
                elapsed.as_secs_f64()
            )),
        }
        },
    );
    arrived
}

/// GPU 判器判这一趟时等着收的那条线程隔一个进度间隔打的那一行：写进库几块（共几块）、几块判完了在等前面的块、几张卡收工了、开跑多久。
#[must_use]
pub fn gpu_judge_wall_clock_line(
    blocks_stored: usize,
    pending_blocks: usize,
    blocks_waiting_for_an_earlier_block: usize,
    cards_finished: usize,
    elapsed: std::time::Duration,
) -> String {
    format!(
        "CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu blocks_stored={blocks_stored}/{pending_blocks} blocks_waiting_for_an_earlier_block={blocks_waiting_for_an_earlier_block} cards_finished={cards_finished} elapsed_seconds={:.1}",
        elapsed.as_secs_f64()
    )
}

/// 一段在卡上建的缓冲合起来的字节数（文件头第 5 条），折成 MiB 照 `singlefs_harness::gpu_memory_ledger::mebibytes_of_buffers_held_at_once`。
#[cfg(feature = "gpu")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GpuBufferFootprint {
    bytes_uploaded_once: u64,
    bytes_of_one_pass: u64,
    passes_alive_at_once: u64,
}

#[cfg(feature = "gpu")]
impl GpuBufferFootprint {
    fn mebibytes(self) -> u64 {
        mebibytes_of_buffers_held_at_once(
            self.bytes_uploaded_once,
            self.bytes_of_one_pass,
            self.passes_alive_at_once,
        )
    }
}

/// `crate::crash_verify_gpu::GpuFactsVerifier` 建的几张缓冲里每一项占几个 u32：每写、每段（今天 11 个字：数位那一组写、状态区间、
/// 展开方式码、整组落的单元写那一组、分片的段片起点各位那一组）、每单元、每根与头取那边导出的常量，换成 u64 记账。
#[cfg(feature = "gpu")]
fn facts_verifier_words(words: usize) -> u64 {
    u64::try_from(words).expect("每项几个字装得进 u64")
}
/// 每对重放两个字（重放在枚举写表里的下标、被重放的那次写）：`crash_verify_gpu` 没导出这一个，这里记一处。
#[cfg(feature = "gpu")]
const FACTS_VERIFIER_WORDS_PER_REPLAY: u64 = 2;
#[cfg(feature = "gpu")]
const BYTES_PER_GPU_WORD: u64 = 4;

/// 第 ② 段在卡上核一条流（`crate::crash_verify_gpu::GpuFactsVerifier`）建的缓冲（文件头第 5 条）：六张事实表上卡一次（空表也建一个字），
/// 核一块一趟建头、结论与回读（每个状态各一个字），一块按 `states_in_the_largest_block` 个状态算；一块接一块核，
/// 同时还活着的照一条线程算。
#[cfg(feature = "gpu")]
fn facts_verifier_buffer_footprint(
    facts: &FlowFacts,
    states_in_the_largest_block: u64,
) -> GpuBufferFootprint {
    let count = |length: usize| u64::try_from(length).expect("表长装得进 u64");
    let replays: u64 = facts
        .torn_images
        .iter()
        .map(|torn| count(torn.replays.len()))
        .sum();
    // 段写表里接着段的写、分片的段（C790）再接片起点的各位，一位一个字
    let writes_of_the_segments: u64 = facts
        .segments
        .iter()
        .map(|segment| {
            count(segment.writes.len())
                + segment
                    .slice_start
                    .as_ref()
                    .map_or(0, |slice_start| count(slice_start.digits.len()))
        })
        .sum();
    use crate::crash_verify_gpu::{
        FACTS_VERIFIER_HEADER_WORDS, FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE,
        FACTS_VERIFIER_WORDS_PER_ROOT, FACTS_VERIFIER_WORDS_PER_SEGMENT,
        FACTS_VERIFIER_WORDS_PER_UNIT,
    };
    let words_of_each_table = [
        count(facts.is_tearable.len())
            * facts_verifier_words(FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE),
        replays * FACTS_VERIFIER_WORDS_PER_REPLAY,
        count(facts.segments.len()) * facts_verifier_words(FACTS_VERIFIER_WORDS_PER_SEGMENT),
        writes_of_the_segments,
        count(facts.units.len()) * facts_verifier_words(FACTS_VERIFIER_WORDS_PER_UNIT),
        count(facts.roots.len()) * facts_verifier_words(FACTS_VERIFIER_WORDS_PER_ROOT),
    ];
    GpuBufferFootprint {
        bytes_uploaded_once: words_of_each_table
            .iter()
            .map(|words| (*words).max(1) * BYTES_PER_GPU_WORD)
            .sum(),
        bytes_of_one_pass: (facts_verifier_words(FACTS_VERIFIER_HEADER_WORDS)
            + 2 * states_in_the_largest_block)
            * BYTES_PER_GPU_WORD,
        passes_alive_at_once: PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
    }
}

// ============================== 第 ① 段的 GPU 形态：GPU 判器判块，卡按优先级领活 ==============================

/// CPU 两遍恢复换成与 GPU 结论第 2、3、4、6 字同口径的四个数：看 journal 那一遍落到的实例代号与 txg 低 32 位
/// （没落到根时 GPU 写 `u32::MAX`、`u32::MAX`），两遍的结局类别（0 没有文件、1 读到文件、2 失败）。
/// 红项相同而落到的根不同也是判错（判器读错了一个落点的版本、两个版本都判绿时就只露在这里），对拍要连它一起比。
#[must_use]
pub fn recovery_words_of(judgement: &crate::crash::CrashStateJudgement) -> [u32; 4] {
    use singlefs_core::recovery::RecoveryOutcome;
    let (instance, txg) =
        judgement
            .consulted
            .effective_root
            .map_or((u32::MAX, u32::MAX), |(instance, txg)| {
                (
                    instance.0,
                    u32::try_from(txg.0 & 0xFFFF_FFFF).expect("低 32 位"),
                )
            });
    let class = |outcome: &RecoveryOutcome| match outcome {
        RecoveryOutcome::NoFile { .. } => 0,
        RecoveryOutcome::FileRead { .. } => 1,
        RecoveryOutcome::Failed { .. } => 2,
    };
    [
        instance,
        txg,
        class(&judgement.consulted.outcome),
        class(&judgement.ignored.outcome),
    ]
}

/// 一块的结论字里第一个 GPU 判不了的状态（块内序号）：恢复内核带「判不了」位、或 checker 内核的「判不了」字不为 0。没有就是 `None`。
#[cfg(feature = "gpu")]
#[must_use]
pub fn first_state_the_gpu_could_not_judge(
    words: &[[u32; crate::crash_judge_gpu::VERDICT_WORDS]],
) -> Option<usize> {
    use crate::crash_judge_gpu::{verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD};
    words.iter().position(|state_words| {
        state_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
            || state_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0
    })
}

/// 一张卡在一趟里的账：判了多少、多快、一次派活带几个状态、占多少显存、挂钟花在哪。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuCardOutcome {
    pub name: String,
    /// `nvidia-smi` 给这张卡的序号。
    pub index: u32,
    pub memory_budget_mebibytes: u64,
    pub states_per_dispatch: u64,
    pub scratch_mebibytes: u64,
    pub blocks_judged: u64,
    pub states_judged: u64,
    /// 表上卡与编内核的挂钟。
    pub build_seconds: f64,
    /// 这张卡从开始判到它领的最后一块判完的挂钟。
    pub judge_seconds: f64,
    pub dispatches: u64,
    /// 判的那一段里：备缓冲与提交（CPU）、等卡算完、回读（CPU）、把结论字拆成红项（CPU）。
    pub seconds_preparing: f64,
    pub seconds_waiting_for_the_card: f64,
    pub seconds_reading_back: f64,
    pub seconds_decoding: f64,
    /// 带得最多的一次派活几个状态、卡算得最久的一次派活多久（一次派活带几个按用时调，不一定到 `states_per_dispatch`）。
    pub states_in_the_widest_dispatch: u64,
    pub seconds_of_the_longest_dispatch: f64,
    /// 起过卡池的进程在显存账上给这张卡的判器借的草稿区（MiB，文件头第 5 条）；不进账（卡是这一趟自己起的、或调用方给的）是 `None`。
    pub scratch_booked_mebibytes: Option<u64>,
    /// 起过卡池的进程，这张卡的判器建好那一刻核账：草稿区那一笔借据还在账上没有（第 10t 批代码轮判决 K4-1：借了又在判之前还掉要红）；
    /// 不进账是 `None`。
    pub scratch_lease_on_the_books_when_the_judge_was_built: Option<bool>,
    /// 同上，这张卡领的最后一块判完那一刻再核一次（第 10x 批代码轮判决 K3-1：建好之后、判完之前还掉也要红）；不进账是 `None`。
    pub scratch_lease_on_the_books_when_the_judging_finished: Option<bool>,
}

impl GpuCardOutcome {
    #[must_use]
    pub fn states_per_second(&self) -> f64 {
        if self.judge_seconds > 0.0 {
            self.states_judged as f64 / self.judge_seconds
        } else {
            0.0
        }
    }
}

/// GPU 判器判这一趟的合计：判了多少、花了多久（编内核的时间另计）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuJudgeOutcome {
    pub cards_used: usize,
    pub blocks_judged: u64,
    pub states_judged: u64,
    pub red_states: u64,
    /// 表上卡与编内核的挂钟，几张卡加起来（卡按优先级一张一张备，不同时编）；驱动的盘上缓存命中时几秒，冷编按内核大小几分钟到二十分钟。
    pub compile_seconds: f64,
    /// 判块的挂钟：从第一张卡开始备到最后一块写进库，减去编内核的那一段。
    pub judge_seconds: f64,
    /// 写库的挂钟（CPU，单线程，按块序）。
    pub seconds_storing: f64,
    /// 用到的卡每收工一次一行，按优先级（一张卡在收尾再起的那一轮里又收工一次就有两行；用到几张卡看 `cards_used`）。
    pub cards: Vec<GpuCardOutcome>,
}

impl GpuJudgeOutcome {
    #[must_use]
    pub fn states_per_second(&self) -> f64 {
        if self.judge_seconds > 0.0 {
            self.states_judged as f64 / self.judge_seconds
        } else {
            0.0
        }
    }
}

/// 一块在 GPU 上判出来、还没写库的结果。
#[cfg(feature = "gpu")]
struct GpuJudgedBlock {
    pending_index: usize,
    items_of_each_state: Vec<Vec<String>>,
    /// 逐状态的恢复字（`crate::crash_judge_gpu::recovery_words_of_the_gpu_verdict`）。
    recovery_words_of_each_state: Vec<RecoveryWordsOfState>,
    state_counts: StateCountsOfABlock,
    /// 判这一块的卡（块属性里的卡型与驱动版本，`crate::gpu_unit_checks::GpuCard` 起卡时从 wgpu 的 adapter info 读的）。
    card_name: String,
    card_driver_version: String,
}

/// 这一趟要判的块（这一判法版本下还没有判定块的、归自己的），按点与块序。
pub(crate) fn pending_blocks(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<Vec<(usize, Range<u64>)>, VerdictStoreError> {
    let mut pending = Vec::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_mine(this_block) {
                continue;
            }
            if conclusions.is_stored(store, point_index, &point.block_key(&local))? {
                continue;
            }
            pending.push((point_index, local));
        }
    }
    Ok(pending)
}

/// 把一块的逐状态红项写成判定块与违例（与 [`check_blocks_where`] 同一个写法）。
#[cfg(feature = "gpu")]
fn store_judged_block(
    store: &mut VerdictStore,
    point_index: usize,
    point: &PlannedCrashPoint,
    local: &Range<u64>,
    conclusions: &ConclusionsStoredUnder<'_>,
    judged: GpuJudgedBlock,
    tally: &mut CheckingTally,
) -> Result<(), VerdictStoreError> {
    let mut numbers = Vec::with_capacity(judged.items_of_each_state.len());
    let mut violating = Vec::new();
    for (offset, items) in judged.items_of_each_state.iter().enumerate() {
        if !items.is_empty() {
            tally.red_states += 1;
            violating.push(StateOffsetInBlock(
                u64::try_from(offset).expect("块内序号装得进 u64"),
            ));
            if tally.red_texts.len() < RED_TEXTS_KEPT {
                tally.red_texts.push((
                    point.path_number.clone(),
                    local.start + u64::try_from(offset).expect("块内序号"),
                    items.join("；"),
                ));
            }
        }
        numbers.push(store.register_verdict_vector(&encoded_verdict_of_the_red_items(items))?);
    }
    let verdict_block = VerdictBlock::new(numbers).expect("一块至少一个状态");
    conclusions.store_block(
        store,
        point_index,
        &point.block_key(local),
        verdict_block,
        violating,
        StoredAlongTheVerdicts {
            count_vector: judged
                .state_counts
                .into_count_vector(local.end - local.start),
            recovery_words: Some(judged.recovery_words_of_each_state),
            judged_on: JudgingDevice::Gpu {
                card_type: judged.card_name,
                driver_version: judged.card_driver_version,
            },
        },
    )?;
    tally.blocks_checked += 1;
    tally.states_checked += local.end - local.start;
    Ok(())
}

/// 卡上的线程发给写库那条线程的话。
#[cfg(feature = "gpu")]
enum CardMessage {
    Block(GpuJudgedBlock),
    Finished(GpuCardOutcome),
    /// 这张卡判不了领到的块里的某个状态（表越过了内核的容量）：整趟停下、报错，不退回 CPU 判器。
    Refused(String),
    /// 这张卡（`index` 是 `nvidia-smi` 给它的序号）建判器或判一块时 wgpu 报了 Out of Memory（文件头第 5 条）：只停这一张，判一块时报的那一块已经交回队里；
    /// 带那一句为什么。
    OutOfMemory {
        index: u32,
        reason: String,
    },
}

/// 写库那条线程收完一轮卡上的线程发来的话之后手里的：收工的卡各自的账（按收到的次序）、卡拒判的理由、报了 Out of Memory 的卡（序号与那一句，
/// 按收到的次序）。判完的块按块序写进库几块、写库的错在几轮共用的那一份里（[`BlocksStoredInOrder`]）。
#[cfg(feature = "gpu")]
struct CardsMessagesReceived {
    finished_cards: Vec<GpuCardOutcome>,
    refusal: Option<String>,
    cards_out_of_memory: Vec<(u32, String)>,
}

/// 写库那条线程收卡上的线程发来的话，到发送端全丢掉为止：判完的块交 `stored_in_order` 按块序写库（`store_block` 写一块；后面的块先到就等前面那一块，
/// 到了一起写；写库失败之后到的块不再写），收工的卡记下它的账，拒判的留第一条理由，报了 Out of Memory 的卡都记下；上一次收到之后 `clock` 走过整个进度间隔
/// 还没有新的话，就交 `write_progress_line` 一行 `CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu …`（C611：卡上判一块、编内核比进度间隔久时
/// 照样隔一个间隔报一行，[`gpu_judge_wall_clock_line`]）。`pending_blocks` 是这一趟要判的块数，只进那一行。
#[cfg(feature = "gpu")]
fn receive_the_cards_messages_and_store_the_blocks_in_order(
    receiver: &std::sync::mpsc::Receiver<CardMessage>,
    wall_clock_lines: &mut WallClockProgressLines,
    clock: &dyn Fn() -> std::time::Duration,
    pending_blocks: usize,
    stored_in_order: &mut BlocksStoredInOrder<GpuJudgedBlock>,
    store_block: &mut dyn FnMut(GpuJudgedBlock) -> Result<(), String>,
    write_progress_line: &mut dyn FnMut(String),
) -> CardsMessagesReceived {
    let mut received = CardsMessagesReceived {
        finished_cards: Vec::new(),
        refusal: None,
        cards_out_of_memory: Vec::new(),
    };
    receive_until_the_senders_are_gone_with_wall_clock_lines(
        receiver,
        wall_clock_lines,
        clock,
        &mut |event| match event {
            ReceivedOrWallClockLineDue::WallClockLineDue { elapsed } => {
                write_progress_line(gpu_judge_wall_clock_line(
                    stored_in_order.blocks_stored(),
                    pending_blocks,
                    stored_in_order.blocks_waiting(),
                    received.finished_cards.len(),
                    elapsed,
                ));
            }
            ReceivedOrWallClockLineDue::Received(CardMessage::Finished(card_outcome)) => {
                received.finished_cards.push(card_outcome);
            }
            ReceivedOrWallClockLineDue::Received(CardMessage::Refused(reason)) => {
                received.refusal.get_or_insert(reason);
            }
            ReceivedOrWallClockLineDue::Received(CardMessage::OutOfMemory { index, reason }) => {
                received.cards_out_of_memory.push((index, reason));
            }
            ReceivedOrWallClockLineDue::Received(CardMessage::Block(arrived_block)) => {
                stored_in_order.arrive(arrived_block.pending_index, arrived_block, store_block);
            }
        },
    );
    received
}

/// 几张卡一起干的那份活：要判的块排成一队，备好的卡从队头领。
#[cfg(feature = "gpu")]
struct CardsAtWork<'work> {
    plan: &'work CrashPlan,
    pending: &'work [(usize, Range<u64>)],
    tables: &'work PackedJudgeTablesOfTheRun,
    cards: &'work [std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    /// 每张卡按优先级装到的状态数（一次派活带几个）；0 的这一趟不用。
    states_placed: &'work [u64],
    /// 每张卡的判器在显存账上借的草稿区（MiB）；不进账的是 `None`。
    scratch_booked_mebibytes: &'work [Option<u64>],
    /// 每张卡草稿区那一笔借据的借据号（判器建好那一刻核账用）；不进账的是 `None`。
    scratch_lease_identifiers: &'work [Option<u64>],
    blocks: BlocksForTheCards,
    /// 写库失败、有卡拒判之后置上：卡上的线程判完手里那一块就停、一块都不再领。一张卡报 Out of Memory 不置（只停那一张，第 10z 批）。
    stop: std::sync::atomic::AtomicBool,
    /// 调用方要停（每领一块之前问一次）：不再领新的块，只领判完的块在等的那几块（第 10z 批第五块），手里那一块判完照旧交去写库。
    /// 一趟里见过一次就记住（第 10z 批第六块，第 10z 批第五块代码轮判决 K2-2），与一轮收尾问的是同一个。
    stop_request: &'work StopRequestRememberedInThisRun<'work>,
}

#[cfg(feature = "gpu")]
impl CardsAtWork<'_> {
    /// 卡上的线程这一刻照什么停（[`judge_blocks_on_one_card`]、[`start_the_next_card_with_states`] 问）：判红了一块都不再领，
    /// 调用方叫停了只领判完的块在等的，都没有照常领。
    fn stop_of_the_cards(&self) -> StopOfTheCards {
        if self.stop.load(std::sync::atomic::Ordering::SeqCst) {
            StopOfTheCards::TheRunFailed
        } else if self.stop_request.is_requested() {
            StopOfTheCards::RequestedByTheCaller
        } else {
            StopOfTheCards::NotRequested
        }
    }
}

/// 卡上一段活做完之后的结局（[`decide_the_work_on_the_card`]）。
#[cfg(feature = "gpu")]
enum WorkOnTheCard<Outcome> {
    Done(Outcome),
    /// 显存不够：`detail` 说 wgpu 报的那一句（这条线程上的错误范围接住的，或别的线程上没接住、记在设备上的）。
    OutOfMemory {
        detail: String,
    },
    /// 活 panic 了，又说不成显存不够：照样往外抛。
    Panicked(Box<dyn std::any::Any + Send>),
}

/// 卡上一段活（`outcome`，panic 了是 `Err`）怎么收：这条线程上的 Out of Memory 范围接到了（`out_of_memory_in_the_scope`）就是显存不够，
/// 活成没成都一样；没接到而活 panic 了，看设备上从活开始起记的（`out_of_memory_on_the_device_since_the_work_began`，别的线程上没接住的
/// Out of Memory、或因它丢了设备）：有就是显存不够，没有照样抛（第 10x 批代码轮判决 K1-3：判器内核在另起的线程上编，那条线程上的
/// Out of Memory 让卡线程在 `join` 那里 panic，范围接不到）。活没 panic 就交它的结果。
#[cfg(feature = "gpu")]
fn decide_the_work_on_the_card<Outcome>(
    outcome: std::thread::Result<Outcome>,
    out_of_memory_in_the_scope: Option<String>,
    out_of_memory_on_the_device_since_the_work_began: Option<String>,
) -> WorkOnTheCard<Outcome> {
    match (
        outcome,
        out_of_memory_in_the_scope,
        out_of_memory_on_the_device_since_the_work_began,
    ) {
        (Ok(_) | Err(_), Some(detail), _) => WorkOnTheCard::OutOfMemory { detail },
        (Ok(outcome), None, _) => WorkOnTheCard::Done(outcome),
        (Err(_), None, Some(detail)) => WorkOnTheCard::OutOfMemory { detail },
        (Err(panic), None, None) => WorkOnTheCard::Panicked(panic),
    }
}

/// 在 `card` 上做 `work`，同时接住 wgpu 报的 Out of Memory（文件头第 5 条）：没报就交 `work` 的结果；报了交 `Err(那一句)`
/// （`what_ran_out` 说是哪一段，`work` 里因为缓冲没建成而 panic 的一并接住、不再往外抛）；没报而 `work` panic 了照样往外抛。
/// wgpu 的错误范围按线程分：这条线程上的接在范围里；`work` 另起的线程上没接住的记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），
/// 照 [`decide_the_work_on_the_card`] 一并认。设备上记的按这段活认（`singlefs_harness::gpu_memory_ledger::WorkOnTheCardOfThisThread`，
/// 第 10z 批：同一个设备上别的线程那段活的 Out of Memory 说不成这段活的显存不够，这段活真正的 panic 照样往外抛）。
#[cfg(feature = "gpu")]
fn on_the_card_catching_out_of_memory<Outcome>(
    card: &crate::gpu_unit_checks::GpuCard,
    what_ran_out: &str,
    work: impl FnOnce() -> Outcome,
) -> Result<Outcome, String> {
    let alone = WorksEndingTogetherOnADevice::new();
    on_the_card_catching_out_of_memory_ending_together(card, what_ran_out, alone.join(), work)
}

/// 同 [`on_the_card_catching_out_of_memory`]，这段活与同一个设备上别的几段一起开（`ending` 是它的报到，单元校验和对拍时一块切成的几段）：
/// 活交回那一刻记下设备丢没丢，这条线程上的错误范围接到的 Out of Memory 记进设备之后报到，等那几段都报到了再认设备上记的
/// （`singlefs_harness::gpu_memory_ledger::DeviceConditionOfACard::end_the_work_and_explain_its_failure`，第 10z 批第三块：一段因设备丢了先失败、
/// 同块另一段范围里接到的还没记进设备时，先认就认不出、判红；第 10z 批第五块：设备因显存不够丢的，不在同一组也认得出；
/// 交回那一刻设备还没丢的，之后别的段丢了设备不把它说成显存不够）。
#[cfg(feature = "gpu")]
fn on_the_card_catching_out_of_memory_ending_together<Outcome>(
    card: &crate::gpu_unit_checks::GpuCard,
    what_ran_out: &str,
    ending: WorkEndingTogether<'_>,
    work: impl FnOnce() -> Outcome,
) -> Result<Outcome, String> {
    let work_on_the_card = singlefs_harness::gpu_memory_ledger::WorkOnTheCardOfThisThread::begin();
    let out_of_memory_reports_before = card.device_condition().out_of_memory_reports();
    let out_of_memory_scope = card
        .device()
        .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    // 活交回的这一刻设备丢没丢：之后等同组几段时别的段丢了设备，不把这段活真正的 panic 说成显存不够（第 10z 批第五块；
    // 接住与记设备抽在 harness 档，第 10z 批第六块）
    let (outcome, device_when_the_work_returned) =
        work_caught_noting_the_device_when_it_returned(card.device_condition(), work);
    let out_of_memory_in_the_scope =
        pollster::block_on(out_of_memory_scope.pop()).map(|error| error.to_string());
    let out_of_memory_on_the_device = card
        .device_condition()
        .end_the_work_and_explain_its_failure(
            &work_on_the_card,
            out_of_memory_reports_before,
            device_when_the_work_returned,
            out_of_memory_in_the_scope
                .as_ref()
                .map(|caught| format!("序号 {} 的卡{what_ran_out}报了 {caught}", card.index)),
            ending,
        );
    match decide_the_work_on_the_card(
        outcome,
        out_of_memory_in_the_scope,
        out_of_memory_on_the_device,
    ) {
        WorkOnTheCard::Done(outcome) => Ok(outcome),
        WorkOnTheCard::OutOfMemory { detail } => Err(reason_for_running_out_of_gpu_memory(
            &format!("序号 {} 的卡{what_ran_out}（{detail}）", card.index),
        )),
        WorkOnTheCard::Panicked(panic) => std::panic::resume_unwind(panic),
    }
}

/// 一张卡的线程：表上卡、编内核；备好之后，队里还有活、下一张卡也装了状态，就起下一张卡的线程（卡一张一张备，不同时编内核）；
/// 然后从队头一块一块领、判、把结论字拆成红项、交给写库的线程。有一个状态判不了（结论字带「判不了」），
/// 就叫所有卡停下、发一条 [`CardMessage::Refused`]，不退回 CPU 判器（用户 2026-09-29 定：配了 GPU 就一定要 GPU 判，做不到就报错）。
/// 建判器或判一块时 wgpu 报 Out of Memory（[`on_the_card_catching_out_of_memory`]），只停这一张卡、发一条 [`CardMessage::OutOfMemory`]：
/// 判一块时报的，那一块交回队里给别的卡（[`BlocksForTheCards::give_back`]），不交去写库；建判器时报的，下一张卡照起
/// （第 10z 批，第 10y 批代码轮判决 K2-2 P2：一张卡 Out of Memory 不停整趟）。起过卡池的进程，判器建好那一刻与这张卡判完那一刻
/// 各核一次账：草稿区那一笔借据还在不在（第 10t 批代码轮判决 K4-1、第 10x 批代码轮判决 K3-1）。
#[cfg(feature = "gpu")]
fn judge_on_one_card_and_start_the_next<'scope, 'work>(
    scope: &'scope std::thread::Scope<'scope, 'work>,
    work: &'work CardsAtWork<'work>,
    position: usize,
    sender: std::sync::mpsc::Sender<CardMessage>,
) {
    use crate::crash_judge_dispatch::GpuCrashJudge;
    use std::sync::atomic::Ordering;
    let card: &crate::gpu_unit_checks::GpuCard = &work.cards[position];
    let build_started = Instant::now();
    let judge =
        match on_the_card_catching_out_of_memory(card, "建判器（表与草稿区）时", || {
            GpuCrashJudge::on_card_with_packed_tables(
                card,
                work.tables,
                usize::try_from(work.states_placed[position])
                    .expect("一次派活的状态数装得进 usize"),
            )
        }) {
            Ok(judge) => judge,
            Err(reason) => {
                // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
                let _ = sender.send(CardMessage::OutOfMemory {
                    index: card.index,
                    reason,
                });
                start_the_next_card_with_states(scope, work, position, &sender);
                return;
            }
        };
    let scratch_lease_is_on_the_books_now = || {
        work.scratch_lease_identifiers[position].map(|lease_identifier| {
            the_card_pool_this_run_takes_its_cards_from()
                .is_the_lease_on_the_books(card.index, lease_identifier)
        })
    };
    let scratch_lease_on_the_books_when_the_judge_was_built = scratch_lease_is_on_the_books_now();
    let build_seconds = build_started.elapsed().as_secs_f64();
    eprintln!(
        "CRASH_AMPLIFICATION_GPU_JUDGE card={:?} index={} memory_budget_mebibytes={} states_per_dispatch={} scratch_mebibytes={} build_seconds={build_seconds:.1}",
        card.name,
        card.index,
        card.memory_budget_mebibytes(),
        judge.states_per_dispatch(),
        judge.scratch_mebibytes()
    );
    start_the_next_card_with_states(scope, work, position, &sender);
    let judging_started = Instant::now();
    let mut outcome = GpuCardOutcome {
        name: card.name.clone(),
        index: card.index,
        memory_budget_mebibytes: card.memory_budget_mebibytes(),
        states_per_dispatch: u64::try_from(judge.states_per_dispatch()).expect("装得进 u64"),
        scratch_mebibytes: u64::try_from(judge.scratch_mebibytes()).expect("装得进 u64"),
        build_seconds,
        scratch_booked_mebibytes: work.scratch_booked_mebibytes[position],
        scratch_lease_on_the_books_when_the_judge_was_built,
        ..GpuCardOutcome::default()
    };
    let card_left = judge_blocks_on_one_card(
        &work.blocks,
        &|| work.stop_of_the_cards(),
        &mut |pending_index| {
            let (point_index, local) = &work.pending[pending_index];
            let point = &work.plan.points[*point_index];
            let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
            let words = match on_the_card_catching_out_of_memory(card, "判一块时", || {
                judge.judge_states(ordinals.clone())
            }) {
                Ok(words) => words,
                // 只停这一张卡：这一块交回给别的卡（第 10z 批）
                Err(reason) => return BlockJudgedOnACard::OutOfMemory { reason },
            };
            let decoding_started = Instant::now();
            if let Some(offset) = first_state_the_gpu_could_not_judge(&words) {
                let state_words = &words[offset];
                return BlockJudgedOnACard::Refused {
                    reason: format!(
                        "卡 {} 判不了点 {} 块内第 {} 个状态（恢复内核第 1 字 {}、checker 内核第 {} 字 {}：表越过了内核的容量）。\
                         配置开了 GPU，不退回 CPU 判器：照这两个字查是哪张表满了、改内核的容量再跑",
                        card.index,
                        point.name,
                        local.start + u64::try_from(offset).expect("块内序号"),
                        state_words[1],
                        crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD,
                        state_words[crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD]
                    ),
                };
            }
            let mut items_of_each_state: Vec<Vec<String>> = Vec::with_capacity(words.len());
            let mut recovery_words_of_each_state: Vec<RecoveryWordsOfState> =
                Vec::with_capacity(words.len());
            let mut state_counts = StateCountsOfABlock::default();
            for state_words in &words {
                items_of_each_state
                    .push(gpu_verdict_items(state_words).expect("判不了的状态上面已经拒过"));
                recovery_words_of_each_state.push(recovery_words_of_the_gpu_verdict(state_words));
                state_counts.count_the_gpu_bits(
                    pool_checker_judged_bits(state_words),
                    pool_checker_violated_bits(state_words),
                );
            }
            outcome.seconds_decoding += decoding_started.elapsed().as_secs_f64();
            eprintln!(
                "CRASH_AMPLIFICATION_BLOCK point={} block_start={} states={} red={} judge=gpu card={}",
                point.name,
                local.start,
                local.end - local.start,
                items_of_each_state
                    .iter()
                    .filter(|items| !items.is_empty())
                    .count(),
                card.index
            );
            outcome.blocks_judged += 1;
            outcome.states_judged += local.end - local.start;
            BlockJudgedOnACard::Judged((
                items_of_each_state,
                recovery_words_of_each_state,
                state_counts,
            ))
        },
        &mut |pending_index, (items_of_each_state, recovery_words_of_each_state, state_counts)| {
            match sender.send(CardMessage::Block(GpuJudgedBlock {
                pending_index,
                items_of_each_state,
                recovery_words_of_each_state,
                state_counts,
                card_name: card.name.clone(),
                card_driver_version: card.driver_version.clone(),
            })) {
                Ok(()) => BlockHandedToTheStore::Handed,
                Err(_) => BlockHandedToTheStore::TheStoreIsGone,
            }
        },
    );
    match card_left {
        CardLeftTheBlocks::NothingLeftOrStopped | CardLeftTheBlocks::TheStoreIsGone => {}
        CardLeftTheBlocks::OutOfMemory { reason } => {
            // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
            let _ = sender.send(CardMessage::OutOfMemory {
                index: card.index,
                reason,
            });
            return;
        }
        CardLeftTheBlocks::Refused { reason } => {
            work.stop.store(true, Ordering::SeqCst);
            // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
            let _ = sender.send(CardMessage::Refused(reason));
            return;
        }
    }
    outcome.judge_seconds = judging_started.elapsed().as_secs_f64();
    outcome.scratch_lease_on_the_books_when_the_judging_finished =
        scratch_lease_is_on_the_books_now();
    let spent = judge.time_spent();
    outcome.dispatches = spent.dispatches;
    outcome.seconds_preparing = spent.seconds_preparing;
    outcome.seconds_waiting_for_the_card = spent.seconds_waiting_for_the_card;
    outcome.seconds_reading_back = spent.seconds_reading_back;
    outcome.states_in_the_widest_dispatch = spent.states_in_the_widest_dispatch;
    outcome.seconds_of_the_longest_dispatch = spent.seconds_of_the_longest_dispatch;
    // 每张卡一行：规划显卡要的数（这张卡判了多少、多快、一次派活带几个状态、草稿区占多少显存、挂钟花在哪）
    eprintln!(
        "CRASH_AMPLIFICATION_GPU_CARD card={:?} index={} memory_budget_mebibytes={} blocks={} states={} judge_seconds={:.1} states_per_second={:.0} states_per_dispatch={} scratch_mebibytes={} dispatches={} seconds_preparing={:.1} seconds_waiting_for_the_card={:.1} seconds_reading_back={:.1} seconds_decoding={:.1} states_in_the_widest_dispatch={} seconds_of_the_longest_dispatch={:.3}",
        outcome.name,
        outcome.index,
        outcome.memory_budget_mebibytes,
        outcome.blocks_judged,
        outcome.states_judged,
        outcome.judge_seconds,
        outcome.states_per_second(),
        outcome.states_per_dispatch,
        outcome.scratch_mebibytes,
        outcome.dispatches,
        outcome.seconds_preparing,
        outcome.seconds_waiting_for_the_card,
        outcome.seconds_reading_back,
        outcome.seconds_decoding,
        outcome.states_in_the_widest_dispatch,
        outcome.seconds_of_the_longest_dispatch
    );
    // 写库的线程已经走了（写库失败）就没人收，丢了也无妨
    let _ = sender.send(CardMessage::Finished(outcome));
}

/// 第 ① 段借草稿区之前这一步怎么走（[`scratch_step_before_booking`]）。
#[cfg(feature = "gpu")]
#[derive(Clone, Debug, PartialEq, Eq)]
enum ScratchStepBeforeBooking {
    /// 至少一张卡此刻带得了一个状态：去借。
    BookOnTheCards,
    /// 每张卡此刻都带不了一个状态、至少一张空着时带得了：打这一行，报本次未跑（不借、不判红）。
    NotRunNoCardHoldsAStateNow { not_run_line: String },
    /// 空着时也带不了：往下走到没有一张卡装得下那一句，判红。
    RedNoCardHoldsAStateEvenWhenEmpty,
}

/// 照借之前的判定（[`decide_scratch_before_booking`]）定这一步：此刻借不到的那一行照对拍用例那一路写（同一个记号、原因写显存不够），
/// 带每张卡此刻能借的（额度减驱动那一份）与一个状态要借的 `need_of_one_state_mebibytes`。
#[cfg(feature = "gpu")]
fn scratch_step_before_booking(
    decision: ScratchBeforeBooking,
    run_cards: &[CardCapacityOfThisRun],
    need_of_one_state_mebibytes: u64,
) -> ScratchStepBeforeBooking {
    match decision {
        ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow => {
            ScratchStepBeforeBooking::BookOnTheCards
        }
        ScratchBeforeBooking::NoCardHoldsAStateNow => {
            let card_rooms_mebibytes: Vec<u64> = run_cards
                .iter()
                .map(|card| {
                    card.memory_budget_mebibytes
                        .saturating_sub(crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER)
                })
                .collect();
            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow {
                not_run_line: gpu_not_run_line(
                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                    &reason_for_a_booking_without_room_now(
                        need_of_one_state_mebibytes,
                        &card_rooms_mebibytes,
                    ),
                ),
            }
        }
        ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty => {
            ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty
        }
    }
}

/// 第 `position` 张卡备好了（或建判器时报了 Out of Memory、不判）：后面还有装了状态的卡、队里照此刻的叫停还有领得的块
/// （[`CardsAtWork::stop_of_the_cards`]，只在后面有装了状态的卡时问一次），就起下一张的线程（卡一张一张备，不同时编内核）。
#[cfg(feature = "gpu")]
fn start_the_next_card_with_states<'scope, 'work>(
    scope: &'scope std::thread::Scope<'scope, 'work>,
    work: &'work CardsAtWork<'work>,
    position: usize,
    sender: &std::sync::mpsc::Sender<CardMessage>,
) {
    // 起哪一张、看不看叫停抽在 harness 档（第 10z 批第六块，第 10z 批第五块代码轮判决 K3）；叫停交成闭包，没有下一张卡不问（第 10z 批第七块）
    if let Some(next_position) =
        next_card_to_start(work.states_placed, position, &work.blocks, || {
            work.stop_of_the_cards()
        })
    {
        let sender = sender.clone();
        scope.spawn(move || {
            judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
        });
    }
}

/// 一张卡照额度 `memory_budget_mebibytes`（MiB）一次派活带得了几个状态（表上卡之后剩下的显存装草稿区）：表比存储缓冲绑定上限
/// `max_storage_buffer_binding_size` 大、或额度装不下表与驱动那一份，是 0。
#[cfg(feature = "gpu")]
fn states_in_flight_within_the_budget(
    max_storage_buffer_binding_size: u64,
    memory_budget_mebibytes: u64,
    tables: &PackedJudgeTablesOfTheRun,
) -> u64 {
    if tables.mebibytes() * 1024 * 1024 > max_storage_buffer_binding_size
        || memory_budget_mebibytes
            < tables.mebibytes() + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
    {
        return 0;
    }
    u64::try_from(crate::crash_judge_dispatch::states_in_flight_within(
        max_storage_buffer_binding_size,
        memory_budget_mebibytes,
        tables,
    ))
    .expect("装得进 u64")
}

/// 每张卡按它的显存额度一次派活带得了几个状态（[`states_in_flight_within_the_budget`]），按 `cards` 的次序。
#[cfg(feature = "gpu")]
fn states_in_flight_of_each_card(
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    tables: &PackedJudgeTablesOfTheRun,
) -> Vec<u64> {
    cards
        .iter()
        .map(|card| {
            states_in_flight_within_the_budget(
                card.device().limits().max_storage_buffer_binding_size,
                card.memory_budget_mebibytes(),
                tables,
            )
        })
        .collect()
}

/// 这一趟本机的一张卡（按优先级）：序号、名字、存储缓冲绑定上限、给判器的额度（MiB），与照额度一次派活带得了几个状态。
/// 起过卡池的，额度是这一刻这张卡能借给这个进程的加驱动那一份（借来的卡就是这个额度），卡上还没起设备
/// （[`cards_of_the_pool_with_what_each_lends_now`]）；这一趟自己起的卡照卡的额度。
#[cfg(feature = "gpu")]
struct CardCapacityOfThisRun {
    index: u32,
    name: String,
    max_storage_buffer_binding_size: u64,
    memory_budget_mebibytes: u64,
    states_in_flight_capacity: u64,
    /// 这张卡空着时一次派活带得了几个状态（起过卡池的照这张卡空着时能借给这个进程的算；这一趟自己起的卡没有「此刻」与「空着时」之分，
    /// 同 `states_in_flight_capacity`）：每张卡此刻都带不了一个状态时照它分借不到与放不下（第 10x 批代码轮判决 K2-2）。
    states_in_flight_capacity_when_the_card_is_empty: u64,
}

/// 起过卡池的这一趟本机每张卡（池里每张，按池的优先级，不起设备）：额度是这一刻这张卡能借给这个进程的加驱动那一份，照它算一次派活
/// 带得了几个状态，再按判器按状态数借的量（`crate::crash_judge_dispatch::mebibytes_a_judge_needs`）压到能借的之内（装上的状态才借得到）。
/// 不照起池那一刻的空闲（第 10t 批代码轮攻方 S5）。
///
/// # Errors
/// 卡上的显存读不出。
#[cfg(feature = "gpu")]
fn cards_of_the_pool_with_what_each_lends_now(
    pool: &crate::gpu_unit_checks::gpu_test_start::GpuCardPool,
    tables: &PackedJudgeTablesOfTheRun,
) -> Result<Vec<CardCapacityOfThisRun>, String> {
    let cards = pool.cards_for_the_product_path();
    let card_indices: Vec<u32> = cards.iter().map(|card| card.index).collect();
    let rooms = pool.card_rooms_now_and_when_empty_of(&card_indices)?;
    // 照能借的（MiB）算一次派活带得了几个状态：额度是能借的加驱动那一份，再按判器按状态数借的量压到能借的之内
    let states_in_flight_for_a_room = |max_storage_buffer_binding_size: u64,
                                       room_mebibytes: u64| {
        let states_by_the_budget = states_in_flight_within_the_budget(
            max_storage_buffer_binding_size,
            room_mebibytes + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
            tables,
        );
        u64::try_from(most_states_whose_need_fits_the_capacity(
            usize::try_from(states_by_the_budget).unwrap_or(usize::MAX),
            room_mebibytes,
            &|states| crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, states),
        ))
        .expect("装得进 u64")
    };
    Ok(cards
        .into_iter()
        .zip(rooms)
        .map(|(card, room)| {
            let (states_in_flight_capacity, states_in_flight_capacity_when_the_card_is_empty) =
                states_in_flight_now_and_when_the_card_is_empty(room, &|room_mebibytes| {
                    states_in_flight_for_a_room(
                        card.max_storage_buffer_binding_size,
                        room_mebibytes,
                    )
                });
            CardCapacityOfThisRun {
                index: card.index,
                memory_budget_mebibytes: room.room_now_mebibytes
                    + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER,
                states_in_flight_capacity,
                states_in_flight_capacity_when_the_card_is_empty,
                name: card.name,
                max_storage_buffer_binding_size: card.max_storage_buffer_binding_size,
            }
        })
        .collect())
}

/// 一张池里的卡此刻与空着时一次派活各带得了几个状态（`states_in_flight_for_a_room` 照能借的 MiB 算）：此刻照这一刻能借的，
/// 空着时照空着时能借的（[`singlefs_harness::gpu_memory_ledger::CardRoomForThisProcess::ceiling_mebibytes`]，第 10x 批代码轮判决 K2-2）。
#[cfg(feature = "gpu")]
fn states_in_flight_now_and_when_the_card_is_empty(
    room: singlefs_harness::gpu_memory_ledger::CardRoomForThisProcess,
    states_in_flight_for_a_room: &dyn Fn(u64) -> u64,
) -> (u64, u64) {
    (
        states_in_flight_for_a_room(room.room_now_mebibytes),
        states_in_flight_for_a_room(room.ceiling_mebibytes),
    )
}

/// 这一趟本机每张卡（按优先级）：起过卡池的照池（[`cards_of_the_pool_with_what_each_lends_now`]，`cards` 是空的），
/// 这一趟自己起的照 `cards`（[`states_in_flight_of_each_card`]）。
///
/// # Errors
/// 起过卡池的进程读不出卡上的显存。
#[cfg(feature = "gpu")]
fn card_capacities_of_this_run(
    cards_taken_from: GpuCardsSource,
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    tables: &PackedJudgeTablesOfTheRun,
) -> Result<Vec<CardCapacityOfThisRun>, String> {
    match cards_taken_from {
        GpuCardsSource::TheCardPoolOfThisProcess => cards_of_the_pool_with_what_each_lends_now(
            the_card_pool_this_run_takes_its_cards_from(),
            tables,
        ),
        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => Ok(cards
            .iter()
            .zip(states_in_flight_of_each_card(cards, tables))
            .map(|(card, states_in_flight_capacity)| CardCapacityOfThisRun {
                index: card.index,
                name: card.name.clone(),
                max_storage_buffer_binding_size: card
                    .device()
                    .limits()
                    .max_storage_buffer_binding_size,
                memory_budget_mebibytes: card.memory_budget_mebibytes(),
                states_in_flight_capacity,
                states_in_flight_capacity_when_the_card_is_empty: states_in_flight_capacity,
            })
            .collect()),
    }
}

/// 第 ① 段在卡上。卡按 `cards` 的次序（优先级）用：要判的状态先装第一张卡，装到它的显存额度带得了的状态数为止，
/// 装不下的才给下一张（[`place_states_on_cards_by_priority`]）；用到的卡一张一张备（表上卡、编内核），备好就从同一队里领块。
/// 判完的块按块序一块一块写库（后判完的先到就等前面的）：中途被停，写进库的块下一次不再判；
/// 判定向量的编号按块序登记，与用了几张卡无关。配置开了 GPU 就一定由 GPU 判（用户 2026-09-29 定，不接受默认降级）：
/// 建不出表（有清零写、写与落点对不上）、有状态判不了（表越过内核容量），都报错，不退回 CPU 判器；报错之前判完的块照旧在库里。
/// 每领一块之前先问 `stop_is_requested`：要停就不再领新的，已经领的块判完、写进库再收尾（库里不留空洞）；有卡报了 Out of Memory 交回的块、
/// 判完的块在等它的，叫停了也照领照判（交回的块落到收尾再起的那一轮里也一样），判完的块照块序都入库、没判的留给下一趟（第 10z 批第五块，
/// 第 10z 批第三块代码轮判决 K1-5）。
///
/// # Errors
/// 库读写失败；建不出表；没有一张卡装得下这条流的表与一个状态的草稿区；有状态 GPU 判不了；卡上 wgpu 报了 Out of Memory
/// 而别的卡没把剩下的块判完（「本次未跑」那一行已经打了，报错之前判完的块在库里；只停报了的那一张卡，第 10z 批）。
///
/// # Panics
/// 卡上的线程 panic（内核没把状态判完、编内核之前量得内核超了预算）。
#[cfg(feature = "gpu")]
pub fn judge_my_blocks_on_gpu_cards(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    version: &JudgeVersion,
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
    // 判表先入库、再读库，GPU 只拿读回的那一份算
    let tables = crate::crash_library::record_the_judge_tables_and_read_them_back(
        store,
        &packed_judge_tables_of_the_plan(flow, plan)?,
    )?;
    match judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(
        store,
        flow,
        plan,
        &tables,
        is_mine,
        &ConclusionsStoredUnder::TheJudgeVersion(version),
        cards,
        GpuCardsSource::DevicesStartedForThisRun,
        stop_is_requested,
    )? {
        GpuJudgingOfMyBlocks::Judged { tally, outcome } => Ok((tally, outcome)),
        // 卡判成调用方自己起的就不借草稿区，走到这里只能是卡上 wgpu 报了 Out of Memory
        GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { reason, .. } => Err(reason),
    }
}

/// 第 ① 段在卡上判的结局。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时第 ① 段在卡上的那一份只报错，两种结局只在编进来时造"
    )
)]
pub(crate) enum GpuJudgingOfMyBlocks {
    Judged {
        tally: CheckingTally,
        outcome: GpuJudgeOutcome,
    },
    /// 显存不够：起过卡池的进程此刻在账上借不到草稿区（一块都没判），或卡上 wgpu 报了 Out of Memory（之前判完的块在库里，
    /// `tally` 是它们）。「本次未跑」那一行已经打了（`reason` 是它的原因），调用方照叫停的样子交回。
    NotRunTheGpuMemoryWasShort {
        tally: CheckingTally,
        reason: String,
    },
}

/// 「本次未跑」那一行里产品路径的名字（GPU 用例写用例名的那一格）。
#[cfg(feature = "gpu")]
const PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE: &str = "crash_amplification::run_from_environment";

/// 这一趟的卡判成照这个进程的卡池用（[`gpu_cards_in_use`] 交 [`GpuCardsSource::TheCardPoolOfThisProcess`]）时的那个池。
///
/// # Panics
/// 这个进程没起过卡池（调用方把卡的来路判错了）。
#[cfg(feature = "gpu")]
fn the_card_pool_this_run_takes_its_cards_from(
) -> &'static crate::gpu_unit_checks::gpu_test_start::GpuCardPool {
    crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process().expect(
        "这一趟的卡判成照这个进程的卡池用时池已经起好（gpu_cards_in_use 看过），起好的池在进程里不会再变回没起",
    )
}

/// 起过卡池的产品路径向卡池借不到时怎么报（`what_was_booked` 说借的是什么）：此刻借不到、借到的卡上起设备时显存不够，
/// 照对拍用例那一路打「本次未跑」那一行（同一个记号、原因写显存不够）、交 `Ok(那一行)`，调用方照叫停的样子交回；
/// 要的比每张卡空着时能借的都大、卡上的显存读不出判红。那一行怎么造、哪几种判红在 [`not_run_line_or_refusal_of_a_lease`]，
/// 这里只多打印那一行。
///
/// # Errors
/// 同 [`not_run_line_or_refusal_of_a_lease`]。
#[cfg(feature = "gpu")]
fn print_the_not_run_line_or_refuse(
    what_was_booked: &str,
    refusal: &crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal,
) -> Result<String, String> {
    let not_run_line = not_run_line_or_refusal_of_a_lease(what_was_booked, refusal)?;
    println!("{not_run_line}");
    Ok(not_run_line)
}

/// 给定状态表的判器向卡池借不到时报错的那一句（不打印）：此刻借不到、借到的卡上起设备时显存不够，照运行条件不满足报
/// [`GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX`] 那一行（原因照「本次未跑」那一行的原因写，行里不带「本次未跑」的记号：入口照这一行退 78，
/// 与流水线一侧「配了 GPU 卡用不上退 78、不降级」同；不退回 CPU 判器）；要的比每张卡空着时能借的都大、卡上的显存读不出照旧那一句（判红）。
#[cfg(feature = "gpu")]
fn given_states_lease_refusal_as_a_run_condition_not_met(
    table_name: &str,
    refusal: &crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal,
) -> String {
    match not_run_line_or_refusal_of_a_lease("给定状态表的判器", refusal) {
        Ok(not_run_line) => {
            let reason = not_run_line
                .split_once(" reason=")
                .map_or(not_run_line.as_str(), |(_, reason)| reason);
            format!(
                "{GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX} table={table_name} reason={reason}：配了 GPU，给定状态表的判器借不到显存，运行条件不满足（退 78），不换成 CPU 判器"
            )
        }
        Err(red) => red,
    }
}

/// 起过卡池的产品路径向卡池借不到时那一行「本次未跑」或判红的那一句，不打印（第 10z 批第七块：造那一行与打印分开，不要卡的用例只调这一半，
/// 测试段的标准输出里不出现门禁会收成本次未跑的那一行）：此刻借不到、借到的卡上起设备时显存不够交 `Ok(那一行)`。
///
/// # Errors
/// 要的比每张卡空着时能借的都大（说法与没有一张卡装得下这条流的表那一句同一类，分项驱动照它认成设备用不上）、卡上的显存读不出。
#[cfg(feature = "gpu")]
fn not_run_line_or_refusal_of_a_lease(
    what_was_booked: &str,
    refusal: &crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal,
) -> Result<String, String> {
    use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
    let refusal = match refusal {
        CardPoolLeaseRefusal::Booking(refusal) => refusal,
        CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice { reason } => {
            return Ok(gpu_not_run_line(
                PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                reason,
            ));
        }
    };
    if let BookingRefusal::NeedLargerThanEveryCard(NeedLargerThanEveryCard {
        need_mebibytes,
        largest_ceiling_mebibytes,
    }) = refusal
    {
        return Err(format!(
            "没有一张卡装得下这条流的表与{what_was_booked}：要借 {need_mebibytes} MiB，池里最大的一张卡空着时也只借得出 {largest_ceiling_mebibytes} MiB。\
             看配置里每张卡的显存额度与卡上此刻的空闲显存"
        ));
    }
    match scratch_booking_refusal_outcome(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, 0, refusal) {
        ScratchBookingRefusalOutcome::NotRun { not_run_line } => Ok(not_run_line),
        ScratchBookingRefusalOutcome::Red { reason } => Err(format!("{what_was_booked}：{reason}")),
    }
}

/// 第 ① 段每张卡上的判器用什么（只列装了状态的卡，按这一趟卡的次序）：借了草稿区的是借来的卡（额度是借的量加驱动那一份，
/// 判器照它开出来的草稿区正好是借的状态数），每张卡装几个状态（一次派活带几个，都大于 0），借了多少、借据号；
/// 连同这一趟本机每张卡（与 [`CardCapacityOfThisRun`] 的表逐张对上）各装了几个。
#[cfg(feature = "gpu")]
struct ScratchOfTheCards {
    cards_for_the_judges: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>,
    states_placed: Vec<u64>,
    scratch_booked_mebibytes: Vec<Option<u64>>,
    scratch_lease_identifiers: Vec<Option<u64>>,
    states_placed_on_each_card_of_this_run: Vec<u64>,
    /// 借据（不进账时是空的），判器都丢掉之后才还。
    leases: Vec<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>,
}

/// 第 ① 段借草稿区的结局。
#[cfg(feature = "gpu")]
enum ScratchOfEachCard {
    Ready(ScratchOfTheCards),
    /// 一张卡都借不到：「本次未跑」那一行已经打了（`not_run_line`），什么都没借着。
    NotRunTheGpuMemoryWasShort {
        not_run_line: String,
    },
}

/// 照卡的来路给第 ① 段装状态、借草稿区（[`scratch_states_to_book_on_each_card`]）。`run_cards` 是这一趟本机每张卡
/// （[`card_capacities_of_this_run`]），`states_each_card_holds` 是每张卡最多装几个（起过卡池的已照这一刻能借的压过）。
/// 起过卡池的：按卡的先后把 `pending_states` 个状态装到此刻借得到的卡上，每张在账的锁里现读、照此刻能借的再压一次再借
/// （显存按 `crate::crash_judge_dispatch::mebibytes_a_judge_needs`，与 GPU 对拍用例借卡同一份），借不到的卡不装；
/// 一张都借不到就打「本次未跑」那一行（[`print_the_not_run_line_or_refuse`]）。没起过卡池的不借、卡照原样（`cards`，与 `run_cards` 逐张对上），
/// 照 [`place_states_on_cards_by_priority`] 装。
///
/// # Errors
/// 每张卡空着时连一个状态都放不下、卡上的显存读不出（判红）。
#[cfg(feature = "gpu")]
fn scratch_of_each_card_booked_by_where_the_cards_came_from(
    cards_taken_from: GpuCardsSource,
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    run_cards: &[CardCapacityOfThisRun],
    tables: &PackedJudgeTablesOfTheRun,
    state_count: u64,
    pending_states: u64,
    states_each_card_holds: &[u64],
) -> Result<ScratchOfEachCard, String> {
    match scratch_states_to_book_on_each_card(cards_taken_from, states_each_card_holds, state_count)
    {
        ScratchStatesToBook::NotBooked => {
            let states_placed_on_each_card_of_this_run =
                place_states_on_cards_by_priority(pending_states, states_each_card_holds);
            let placed: Vec<(&std::sync::Arc<crate::gpu_unit_checks::GpuCard>, u64)> = cards
                .iter()
                .zip(&states_placed_on_each_card_of_this_run)
                .filter(|(_, states_placed)| **states_placed > 0)
                .map(|(card, states_placed)| (card, *states_placed))
                .collect();
            Ok(ScratchOfEachCard::Ready(ScratchOfTheCards {
                cards_for_the_judges: placed.iter().map(|(card, _)| (*card).clone()).collect(),
                states_placed: placed
                    .iter()
                    .map(|(_, states_placed)| *states_placed)
                    .collect(),
                scratch_booked_mebibytes: vec![None; placed.len()],
                scratch_lease_identifiers: vec![None; placed.len()],
                states_placed_on_each_card_of_this_run,
                leases: Vec::new(),
            }))
        }
        ScratchStatesToBook::OnEachCard(states_each_card_lends) => {
            let card_indices: Vec<u32> = run_cards.iter().map(|card| card.index).collect();
            let states_each_card_lends: Vec<u64> = states_each_card_lends
                .into_iter()
                .map(|states| u64::try_from(states).expect("状态数装得进 u64"))
                .collect();
            let booked = match the_card_pool_this_run_takes_its_cards_from()
                .book_the_scratch_of_the_cards_with_room_now(
                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                    &card_indices,
                    pending_states,
                    &states_each_card_lends,
                    &|states| {
                        crate::crash_judge_dispatch::mebibytes_a_judge_needs(
                            tables,
                            usize::try_from(states).unwrap_or(usize::MAX),
                        )
                    },
                ) {
                Ok(booked) => booked,
                Err(refusal) => {
                    let not_run_line =
                        print_the_not_run_line_or_refuse("第 ① 段判器的草稿区", &refusal)?;
                    return Ok(ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line });
                }
            };
            let mut scratch = ScratchOfTheCards {
                cards_for_the_judges: Vec::new(),
                states_placed: Vec::new(),
                scratch_booked_mebibytes: Vec::new(),
                scratch_lease_identifiers: Vec::new(),
                states_placed_on_each_card_of_this_run: vec![0; run_cards.len()],
                leases: Vec::new(),
            };
            for (card_slot, booked_on_the_card) in booked.into_iter().enumerate() {
                if let Some(crate::gpu_unit_checks::gpu_test_start::ScratchLeaseOnACard {
                    lease,
                    states,
                }) = booked_on_the_card
                {
                    scratch
                        .cards_for_the_judges
                        .push(std::sync::Arc::new(lease.card().clone()));
                    scratch.states_placed.push(states);
                    scratch
                        .scratch_booked_mebibytes
                        .push(Some(lease.mebibytes()));
                    scratch
                        .scratch_lease_identifiers
                        .push(Some(lease.lease_identifier()));
                    scratch.states_placed_on_each_card_of_this_run[card_slot] = states;
                    scratch.leases.push(lease);
                }
            }
            Ok(ScratchOfEachCard::Ready(scratch))
        }
    }
}

/// 同 [`judge_my_blocks_on_gpu_cards`]，卡的来路另给：起过卡池的（[`GpuCardsSource::TheCardPoolOfThisProcess`]，`cards` 是空的），
/// 本机每张卡装几个状态照这一刻能借的量压（[`cards_of_the_pool_with_what_each_lends_now`]），判之前把要判的状态装到此刻借得到草稿区的卡上
/// （[`scratch_of_each_card_booked_by_where_the_cards_came_from`]：借不到的卡这一趟不用，一张都借不到交
/// [`GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort`]、一块都不判）；判完先丢判器、再还借据。卡上 wgpu 报了 Out of Memory
/// 同样交 [`GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort`]（打「本次未跑」那一行，之前判完的块在库里）。
///
/// # Errors
/// 同 [`judge_my_blocks_on_gpu_cards`]（Out of Memory 那一形除外）；借草稿区时每张卡空着时连一个状态都放不下、卡上的显存读不出。
///
/// # Panics
/// 同 [`judge_my_blocks_on_gpu_cards`]。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、流、计划、分工、判法版本、卡、卡的来路、叫停，各是各的东西，凑成结构体只为让签名变短"
)]
#[allow(
    clippy::too_many_lines,
    reason = "备表、装卡、借草稿区、几张卡一起判、按块序写库、收尾，是第 ① 段的全部步骤，拆开要把十几个量来回传"
)]
fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    tables: &PackedJudgeTablesOfTheRun,
    is_mine: &dyn Fn(u64) -> bool,
    conclusions: &ConclusionsStoredUnder<'_>,
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    cards_taken_from: GpuCardsSource,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<GpuJudgingOfMyBlocks, String> {
    let pending = pending_blocks(store, plan, conclusions, is_mine)
        .map_err(|error| format!("读判定块失败：{error:?}"))?;
    // 按发布分的态数（J7）：GPU 判器不逐状态造镜像，存块时按段的序号区间现算
    let publishes_of_the_states = PublishesOfTheStates::of_the_plan(flow, plan);
    let mut tally = CheckingTally::default();
    let mut outcome = GpuJudgeOutcome::default();
    if pending.is_empty() {
        return Ok(GpuJudgingOfMyBlocks::Judged { tally, outcome });
    }
    let state_count = tables.state_count();
    // 几轮之间带着的（harness 档，不要卡就测得到）：按块序写库的那一份（上一轮判完、等着前面那一块的，下一轮那一块判完照块序一起写）、
    // 下一轮的那一队块、报过 Out of Memory 的卡与头一句、收过工的卡
    let mut rounds: RoundsOfTheCards<GpuJudgedBlock> = RoundsOfTheCards::new(pending.len());
    // 卡线程与一轮收尾问的是同一个叫停，见过一次就记住（第 10z 批第六块；调用方 `run_from_environment` 交的已经是记住的那一个）
    let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);
    // 这一趟本机每张卡的优先级（头一轮那一份，后面几轮只少不多）：收尾时每张卡的账照它排
    let mut card_indices_by_priority: Vec<u32> = Vec::new();
    let started = Instant::now();
    // 每再起一轮至少少一张卡（报过 Out of Memory 的不再用），轮数不超过卡数
    loop {
        let pending_states: u64 = rounds
            .blocks_of_the_next_round()
            .iter()
            .map(|pending_index| {
                let (_, local) = &pending[*pending_index];
                local.end - local.start
            })
            .sum();
        let cards_of_the_round: Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>> = cards
            .iter()
            .filter(|card| !rounds.is_short_of_memory(card.index))
            .cloned()
            .collect();
        let run_cards: Vec<CardCapacityOfThisRun> =
            card_capacities_of_this_run(cards_taken_from, &cards_of_the_round, tables)?
                .into_iter()
                .filter(|card| !rounds.is_short_of_memory(card.index))
                .collect();
        if card_indices_by_priority.is_empty() {
            card_indices_by_priority = run_cards.iter().map(|card| card.index).collect();
        }
        let capacities: Vec<u64> = run_cards
            .iter()
            .map(|card| card.states_in_flight_capacity)
            .collect();
        let capacities_when_the_cards_are_empty: Vec<u64> = run_cards
            .iter()
            .map(|card| card.states_in_flight_capacity_when_the_card_is_empty)
            .collect();
        // 借据活到这一轮收尾：几张卡上的判器在下面的线程里判完就丢，丢完才还
        let scratch = match scratch_step_before_booking(
            decide_scratch_before_booking(&capacities, &capacities_when_the_cards_are_empty),
            &run_cards,
            crate::crash_judge_dispatch::mebibytes_a_judge_needs(tables, 1),
        ) {
            ScratchStepBeforeBooking::BookOnTheCards => {
                match scratch_of_each_card_booked_by_where_the_cards_came_from(
                    cards_taken_from,
                    &cards_of_the_round,
                    &run_cards,
                    tables,
                    state_count,
                    pending_states,
                    &capacities,
                )? {
                    ScratchOfEachCard::Ready(scratch) => Some(scratch),
                    ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line } => {
                        return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
                            tally,
                            reason: not_run_line,
                        });
                    }
                }
            }
            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => {
                println!("{not_run_line}");
                return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort {
                    tally,
                    reason: not_run_line,
                });
            }
            ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty => None,
        };
        let states_placed_on_each_card_of_this_run = scratch.as_ref().map_or_else(
            || vec![0; run_cards.len()],
            |scratch| scratch.states_placed_on_each_card_of_this_run.clone(),
        );
        for ((card, capacity), placed) in run_cards
            .iter()
            .zip(&capacities)
            .zip(&states_placed_on_each_card_of_this_run)
        {
            eprintln!(
                "CRASH_AMPLIFICATION_GPU_PLACEMENT card={:?} index={} memory_budget_mebibytes={} states_in_flight_capacity={capacity} states_placed={placed} pending_states={pending_states} tables_mebibytes={}",
                card.name,
                card.index,
                card.memory_budget_mebibytes,
                tables.mebibytes()
            );
        }
        let Some(scratch) = scratch.filter(|scratch| !scratch.cards_for_the_judges.is_empty())
        else {
            if let Some(reason) = rounds.first_out_of_memory().map(str::to_string) {
                // 报过 Out of Memory 的卡之外没有一张装得下：没有卡能判剩下的块了
                println!(
                    "{}",
                    gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
                );
                return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason });
            }
            return Err(format!(
                "没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",
                tables.mebibytes()
            ));
        };
        let work = CardsAtWork {
            plan,
            pending: &pending,
            tables,
            cards: &scratch.cards_for_the_judges,
            states_placed: &scratch.states_placed,
            scratch_booked_mebibytes: &scratch.scratch_booked_mebibytes,
            scratch_lease_identifiers: &scratch.scratch_lease_identifiers,
            blocks: rounds.take_the_blocks_of_the_round(),
            stop: std::sync::atomic::AtomicBool::new(false),
            stop_request: &stop_request,
        };
        let received = std::thread::scope(|scope| {
            let (sender, receiver) = std::sync::mpsc::channel::<CardMessage>();
            let work = &work;
            scope.spawn(move || {
                judge_on_one_card_and_start_the_next(scope, work, 0, sender);
            });
            // 卡上判一块、编内核都可能比进度间隔久：等着收的这条线程隔一个间隔打一行按挂钟的
            let clock = || started.elapsed();
            let mut wall_clock_lines =
                WallClockProgressLines::every_progress_line_interval(clock());
            let received = receive_the_cards_messages_and_store_the_blocks_in_order(
                &receiver,
                &mut wall_clock_lines,
                &clock,
                pending.len(),
                rounds.stored_in_order(),
                &mut |block| {
                    let storing_started = Instant::now();
                    let (point_index, local) = &pending[block.pending_index];
                    let point = &plan.points[*point_index];
                    let mut block = block;
                    block
                        .state_counts
                        .absorb(publishes_of_the_states.state_counts_of(
                            &(point.ordinals.start + local.start..point.ordinals.start + local.end),
                        ));
                    let stored = store_judged_block(
                        store,
                        *point_index,
                        point,
                        local,
                        conclusions,
                        block,
                        &mut tally,
                    );
                    outcome.seconds_storing += storing_started.elapsed().as_secs_f64();
                    match stored {
                        Ok(()) => {
                            outcome.blocks_judged += 1;
                            outcome.states_judged += local.end - local.start;
                            Ok(())
                        }
                        Err(error) => {
                            work.stop.store(true, std::sync::atomic::Ordering::SeqCst);
                            Err(format!("写判定块失败：{error:?}"))
                        }
                    }
                },
                &mut |line| eprintln!("{line}"),
            );
            received
        });
        if let Some(error) = rounds.stored_in_order().storing_error() {
            return Err(error.to_string());
        }
        if let Some(reason) = received.refusal {
            return Err(reason);
        }
        let finished_card_indices: Vec<u32> = received
            .finished_cards
            .iter()
            .map(|card| card.index)
            .collect();
        outcome.cards.extend(received.finished_cards);
        let stop = stop_request.stop_of_the_run();
        let run_card_indices: Vec<u32> = run_cards.iter().map(|card| card.index).collect();
        // 收工之后的不变量（没有卡报 Out of Memory 时判完的块都写进了库、没叫停时块都领走了）也在这一步里判
        match rounds.after_the_round(
            &work.blocks,
            &run_card_indices,
            finished_card_indices,
            &received.cards_out_of_memory,
            stop,
        ) {
            AfterARoundOfTheCards::EveryBlockStored => {
                if rounds.first_out_of_memory().is_some() {
                    eprintln!(
                        "CRASH_AMPLIFICATION_GPU_OUT_OF_MEMORY_ON_A_CARD 有卡报了 Out of Memory、只停那一张，别的卡判完了全部 {} 块",
                        pending.len()
                    );
                }
                break;
            }
            AfterARoundOfTheCards::StoppedOnRequest => {
                // 叫停加有卡报过 Out of Memory：显存不够的缘由照本次未跑那一行打出来（第 10z 批第五块代码轮判决 K1-3）
                if let Some(not_run_line) = rounds.not_run_line_of_a_stop_after_an_out_of_memory(
                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                ) {
                    println!("{not_run_line}");
                }
                break;
            }
            AfterARoundOfTheCards::AnotherRoundWithoutTheCardsShortOfMemory => {
                eprintln!(
                    "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND 报了 Out of Memory 的卡 {:?} 收工之后还有 {} 块没写进库，收尾再起一轮、用别的卡判剩下的 {} 块",
                    rounds.cards_short_of_memory(),
                    pending.len() - rounds.blocks_stored(),
                    rounds.blocks_of_the_next_round().len()
                );
            }
            AfterARoundOfTheCards::AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor => {
                eprintln!(
                    "CRASH_AMPLIFICATION_GPU_ANOTHER_ROUND_AFTER_THE_STOP 叫停了；报了 Out of Memory 的卡 {:?} 交回的块前面判完的 {} 块在等，再起一轮、用别的卡只判它们在等的那几块，判完照叫停收尾",
                    rounds.cards_short_of_memory(),
                    rounds.blocks_waiting()
                );
            }
            AfterARoundOfTheCards::NotRunTheGpuMemoryWasShort => {
                let reason = rounds
                    .first_out_of_memory()
                    .expect("这一轮有卡报了 Out of Memory 才会走到本次未跑这一支")
                    .to_string();
                println!(
                    "{}",
                    gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
                );
                return Ok(GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason });
            }
        }
    }
    outcome.cards.sort_by_key(|card| {
        card_indices_by_priority
            .iter()
            .position(|index| *index == card.index)
    });
    // 一张卡在收尾再起的那一轮里又收工一次，`outcome.cards` 里有它两行，用到的卡只算一张（第 10z 批第五块）
    outcome.cards_used = rounds.cards_used();
    outcome.compile_seconds = outcome.cards.iter().map(|card| card.build_seconds).sum();
    outcome.judge_seconds = (started.elapsed().as_secs_f64() - outcome.compile_seconds).max(0.0);
    outcome.red_states = tally.red_states;
    Ok(GpuJudgingOfMyBlocks::Judged { tally, outcome })
}

/// GPU 判完这一趟打的那一行：判了多少、多快，连这一趟的展开上限与全域与否（[`run_from_environment`] 打，展开上限只有它知道）。
#[must_use]
pub fn gpu_total_line(
    outcome: &GpuJudgeOutcome,
    expand_up_to: ExpandUpToWritesPerSegment,
    exhaustive: bool,
) -> String {
    format!(
        "CRASH_AMPLIFICATION_GPU_TOTAL cards_used={} blocks={} states={} compile_seconds={:.1} judge_seconds={:.1} states_per_second={:.0} seconds_storing={:.1} expand_up_to={} exhaustive={exhaustive}",
        outcome.cards_used,
        outcome.blocks_judged,
        outcome.states_judged,
        outcome.compile_seconds,
        outcome.judge_seconds,
        outcome.states_per_second(),
        outcome.seconds_storing,
        expand_up_to.0
    )
}

// ============================== 三段流的第 ②、③ 段：读事实核对，与判器逐状态比 ==============================

/// 第 ② 段在哪算：CPU 参照（`crash_facts::verify_states_from_facts`）或一张卡（`crash_verify_gpu`）。
pub enum FactsVerifierOn {
    Cpu,
    #[cfg(feature = "gpu")]
    Gpu(std::sync::Arc<crate::gpu_unit_checks::GpuCard>),
}

impl FactsVerifierOn {
    /// 核对代码的摘要：进核对表的键。CPU 参照的是 `crash_facts.rs` 的原文摘要连版本串，GPU 的是着色器原文摘要连版本串。
    #[must_use]
    pub fn verifier_digest(&self) -> [u8; DIGEST_BYTES] {
        match self {
            Self::Cpu => {
                let mut message = b"cpu-facts-verifier-v2\n".to_vec();
                message.extend_from_slice(include_str!("crash_facts.rs").as_bytes());
                sha256_digest(&message)
            }
            #[cfg(feature = "gpu")]
            Self::Gpu(_) => crate::crash_verify_gpu::gpu_verifier_digest(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            #[cfg(feature = "gpu")]
            Self::Gpu(_) => "gpu",
        }
    }
}

/// 第 ② 段的结局：在哪核的（`cpu` / `gpu`）与核了多少。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerificationOutcome {
    pub verifier: &'static str,
    pub tally: VerificationTally,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VerificationTally {
    pub blocks_verified: u64,
    pub blocks_already_verified: u64,
    pub states_verified: u64,
    pub verifier_red_states: u64,
}

/// 第 ① 段的事实进库、第 ② 段按块核：`is_mine` 选这台机器领的块（与 [`check_blocks_where`] 同一套分工），
/// 库里这一版核对代码已核过的块不再核。事实表按流抽一次、上卡一次。
///
/// # Errors
/// 库读写失败；GPU 内核起不来。
pub fn verify_blocks_from_facts(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    on: &FactsVerifierOn,
) -> Result<VerificationTally, String> {
    let (facts, segment_expansions) = facts_and_segment_expansions_of_the_plan(flow, plan);
    // 事实表先入库、再读库，核对只拿读回的那一份
    let facts = crate::crash_library::record_the_facts_and_read_them_back(store, &facts)?;
    verify_blocks_from_these_facts(store, plan, &facts, &segment_expansions, is_mine, on)
}

/// 每一段照这份计划的拆法（第 ② 段两种核对都照它拆序号）：判段在库里读回的流程上照计划现算。
fn segment_expansions_of_the_plan(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> Vec<SegmentExpansionFact> {
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    segment_expansion_facts(flow.writes, flow.segments, &expansion)
}

/// 一条流照这份计划的展开抽的事实表，与每一段的拆法（第 ② 段两种核对都照它拆序号；G 那一路交给引擎的「事实表带上段的展开方式」）：
/// 展开取 [`CrashPlan::segment_expansion`]，ignore 的点的段与计划一样不展开，事实表的序号区间与计划的块对得上。
fn facts_and_segment_expansions_of_the_plan(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> (FlowFacts, Vec<SegmentExpansionFact>) {
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    (
        extract_flow_facts_with_slices(
            flow.base,
            flow.writes,
            flow.segments,
            &expansion,
            plan.segment_slices(),
        ),
        segment_expansion_facts(flow.writes, flow.segments, &expansion),
    )
}

/// 同 [`verify_blocks_from_facts`]，事实表由调用方给（流水线判段给的是从库里读回的那一份）；核对行写进库、带事实表摘要。
///
/// # Errors
/// 同 [`verify_blocks_from_facts`]。
fn verify_blocks_from_these_facts(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    facts: &crate::crash_facts::FlowFactsReadBack,
    segment_expansions: &[SegmentExpansionFact],
    is_mine: &dyn Fn(u64) -> bool,
    on: &FactsVerifierOn,
) -> Result<VerificationTally, String> {
    let facts_digest = facts.digest();
    let verifier_digest = on.verifier_digest();
    let verifier = BlockVerifier::new(on, facts, segment_expansions);
    let mut tally = VerificationTally::default();
    let mut block_sequence = 0u64;
    for point in &plan.points {
        for local in point.blocks() {
            let sequence = block_sequence;
            block_sequence += 1;
            if !is_mine(sequence) {
                continue;
            }
            let key = point.block_key(&local);
            if store
                .read_verifier_block(&key, &verifier_digest)
                .map_err(|error| format!("核对表读失败：{error:?}"))?
                .is_some()
            {
                tally.blocks_already_verified += 1;
                continue;
            }
            let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
            let bits = verifier.verify(facts, segment_expansions, ordinals);
            tally.verifier_red_states += bits.iter().filter(|value| **value != 0).count() as u64;
            tally.states_verified += bits.len() as u64;
            tally.blocks_verified += 1;
            store
                .store_verifier_block(&key, &verifier_digest, &facts_digest, &bits)
                .map_err(|error| format!("核对表写不进库：{error:?}"))?;
        }
    }
    Ok(tally)
}

/// 一条流在某一处核对：CPU 参照不用准备；GPU 要先把事实表上卡。
enum BlockVerifier<'card> {
    Cpu,
    #[cfg(feature = "gpu")]
    Gpu(Box<crate::crash_verify_gpu::GpuFactsVerifier<'card>>),
    #[cfg(not(feature = "gpu"))]
    #[expect(dead_code, reason = "没编进 gpu 时只替生命周期参数占位，从不建出")]
    Never(std::marker::PhantomData<&'card ()>),
}

impl<'card> BlockVerifier<'card> {
    fn new(
        on: &'card FactsVerifierOn,
        facts: &crate::crash_facts::FlowFactsReadBack,
        segment_expansions: &[SegmentExpansionFact],
    ) -> Self {
        match on {
            FactsVerifierOn::Cpu => {
                let _ = (facts, segment_expansions);
                Self::Cpu
            }
            #[cfg(feature = "gpu")]
            FactsVerifierOn::Gpu(card) => Self::Gpu(Box::new(
                crate::crash_verify_gpu::GpuFactsVerifier::new_by_the_segment_expansions(
                    card,
                    facts,
                    segment_expansions,
                ),
            )),
        }
    }

    fn verify(
        &self,
        facts: &crate::crash_facts::FlowFactsReadBack,
        segment_expansions: &[SegmentExpansionFact],
        ordinals: Range<u64>,
    ) -> Vec<u32> {
        match self {
            Self::Cpu => verify_states_from_facts_by_the_segment_expansions(
                facts,
                segment_expansions,
                ordinals,
            ),
            #[cfg(feature = "gpu")]
            Self::Gpu(verifier) => verifier.verify_states(ordinals),
            #[cfg(not(feature = "gpu"))]
            Self::Never(_) => unreachable!("没编进 gpu 特性就不会建出这一支"),
        }
    }
}

/// 第 ③ 段：核对表与判定块表逐状态比。核对红 ⇒ 判器也红，不然是不一致（核对那一侧错了或判器漏了，都要查）；
/// 判器红而核对绿是核对没覆盖到的那一类，只计数。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JudgeVerifierComparison {
    pub states_compared: u64,
    pub both_green: u64,
    pub both_red: u64,
    /// 核对红、判器绿的（路径，点内序号，结论位）——最多留 [`RED_TEXTS_KEPT`] 条，计数在 `disagreements`。
    pub disagreement_samples: Vec<(String, u64, u32)>,
    pub disagreements: u64,
    pub judge_red_only: u64,
    /// 判器判过而这一版核对代码没核的块（第 ② 段还欠的）。
    pub blocks_without_verifier: u64,
    /// 这一版核对代码核过而判器没判的块。
    pub blocks_without_judge: u64,
}

/// # Errors
/// 库读失败。
pub fn compare_judge_and_verifier(
    store: &VerdictStore,
    plan: &CrashPlan,
    verifier_digest: &[u8; DIGEST_BYTES],
    version: &JudgeVersion,
) -> Result<JudgeVerifierComparison, VerdictStoreError> {
    compare_the_conclusions_and_the_verifier(
        store,
        plan,
        verifier_digest,
        &ConclusionsStoredUnder::TheJudgeVersion(version),
    )
}

/// 同 [`compare_judge_and_verifier`]，判器那一侧是 `conclusions` 那一处的结论。
///
/// # Errors
/// 库读失败。
pub fn compare_the_conclusions_and_the_verifier(
    store: &VerdictStore,
    plan: &CrashPlan,
    verifier_digest: &[u8; DIGEST_BYTES],
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<JudgeVerifierComparison, VerdictStoreError> {
    let mut comparison = JudgeVerifierComparison::default();
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let key = point.block_key(&local);
            let judged = conclusions.verdict_numbers_of_each_state(store, point_index, &key)?;
            let verified = store.read_verifier_block(&key, verifier_digest)?;
            let (Some(judged), Some(row)) = (&judged, &verified) else {
                if judged.is_some() {
                    comparison.blocks_without_verifier += 1;
                }
                if verified.is_some() {
                    comparison.blocks_without_judge += 1;
                }
                continue;
            };
            for (offset, (number, bit)) in judged.iter().zip(&row.bits).enumerate() {
                let judge_red = store
                    .verdict_vector_of_number(*number)
                    .is_some_and(|vector| !vector.0.is_empty());
                let verifier_red = *bit != 0;
                comparison.states_compared += 1;
                match (judge_red, verifier_red) {
                    (false, false) => comparison.both_green += 1,
                    (true, true) => comparison.both_red += 1,
                    (true, false) => comparison.judge_red_only += 1,
                    (false, true) => {
                        comparison.disagreements += 1;
                        if comparison.disagreement_samples.len() < RED_TEXTS_KEPT {
                            comparison.disagreement_samples.push((
                                point.path_number.clone(),
                                local.start + offset as u64,
                                *bit,
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(comparison)
}

// ============================== 对账与门禁读 ==============================

/// 录了、在这一判法版本下还没有判定块的块（卡掉线、进程被杀留下的，或判法换了版本之后还没重判的）：一轮收尾按它重新分给活着的一方补跑。
///
/// # Errors
/// 库读失败。
pub fn unjudged_blocks(
    store: &VerdictStore,
    version: &JudgeVersion,
) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
    let judged: std::collections::HashSet<VerdictBlockKey> = store
        .stored_verdict_block_keys()?
        .into_iter()
        .filter(|(_, stored_version)| stored_version == version)
        .map(|(key, _)| key)
        .collect();
    Ok(store
        .recorded_blocks()?
        .into_iter()
        .filter(|(key, _)| !judged.contains(key))
        .collect())
}

/// 这条流自己的块里，在这一判法版本下还没有判定块的（块键、状态数），按点与块序。一份库装几条流时，一条流跑完对的是它自己的账：
/// 库里别的流录了还没判的块不算它的（[`unjudged_blocks`] 是整份库的账，全部分项跑完之后读）。
///
/// # Errors
/// 库读失败。
pub fn unjudged_blocks_of_the_flow(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
    let mut unjudged = Vec::new();
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let key = point.block_key(&local);
            if !conclusions.is_stored(store, point_index, &key)? {
                unjudged.push((key, local.end - local.start));
            }
        }
    }
    Ok(unjudged)
}

/// 库里的一个红状态：节点名、点内序号、判定向量的逐项文字。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredRedState {
    pub path: String,
    pub identity: String,
    pub state_in_point: u64,
    pub items: Vec<String>,
}

/// 这条流每个状态在库里、这一判法版本下的判定（判红的各项；绿的是空的），按整条流的状态序号排；这一判法版本下没判过的块，
/// 它的状态是 `None`。复用与剪枝的对照拿它与不经库现判的结果逐态比。
///
/// # Errors
/// 库读失败。
///
/// # Panics
/// 判定块里的编号没在这个库里登记过，或块里的状态数与计划不符（库坏了）。
pub fn read_verdict_items_of_every_state(
    store: &VerdictStore,
    plan: &CrashPlan,
    version: &JudgeVersion,
) -> Result<Vec<Option<Vec<String>>>, VerdictStoreError> {
    let mut items_of_each_state = Vec::new();
    for point in &plan.points {
        for local in point.blocks() {
            let state_count = usize::try_from(local.end - local.start).expect("一块装得进 usize");
            match store.read_verdict_block(&point.block_key(&local), version)? {
                None => items_of_each_state.extend(std::iter::repeat_n(None, state_count)),
                Some(block) => {
                    let numbers = block.verdict_vector_number_of_each_state();
                    assert_eq!(
                        numbers.len(),
                        state_count,
                        "点 {} 块 {} 存着的状态数与计划不符",
                        point.path_number,
                        local.start
                    );
                    for number in numbers {
                        let vector = store
                            .verdict_vector_of_number(*number)
                            .expect("判定块里的编号在这个库里登记过");
                        items_of_each_state.push(Some(if vector.0.is_empty() {
                            Vec::new()
                        } else {
                            String::from_utf8_lossy(&vector.0)
                                .split('\n')
                                .map(str::to_string)
                                .collect()
                        }));
                    }
                }
            }
        }
    }
    Ok(items_of_each_state)
}

/// 门禁一次读：这些点每一块在这一判法版本下的违例，按点、按序号交回。
///
/// # Errors
/// 库读失败。
pub fn read_violations(
    store: &VerdictStore,
    plan: &CrashPlan,
    version: &JudgeVersion,
) -> Result<Vec<StoredRedState>, VerdictStoreError> {
    read_the_violations_stored_under(
        store,
        plan,
        &ConclusionsStoredUnder::TheJudgeVersion(version),
    )
}

/// 同 [`read_violations`]，读 `conclusions` 那一处的违例。
///
/// # Errors
/// 库读失败。
pub fn read_the_violations_stored_under(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<Vec<StoredRedState>, VerdictStoreError> {
    let mut found = Vec::new();
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let key = point.block_key(&local);
            for violation in conclusions.violations_in_block(store, point_index, &key)? {
                let vector = store
                    .verdict_vector_of_number(violation.verdict_vector_number)
                    .expect("违例表里的编号在这个库里登记过");
                found.push(StoredRedState {
                    path: point.path_number.clone(),
                    identity: point.identity.clone(),
                    state_in_point: local.start + violation.offset_in_block.0,
                    items: String::from_utf8_lossy(&vector.0)
                        .split('\n')
                        .map(str::to_string)
                        .collect(),
                });
            }
        }
    }
    Ok(found)
}

// ============================== 录入段落库的流程（两段：录入段在 CPU 上跑数据、写进库；判段只从库里读） ==============================

/// 录入段写进库的一条流程：判段重建崩溃状态、排计划、判、核对要的全部输入，自己持有，不借录入段内存里的流程
/// （用户 2026-10-11 定：录入与判是两段，段与段之间不在内存里交数据）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedFlow {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
    pub versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    /// 每一段照录入段那一趟的展开方式（按段登记改过的也在内），段号即下标。
    pub segment_expansions: Vec<CrashPointReplaySegmentExpansion>,
    pub crash_points: Vec<CrashPointSpan>,
    /// 每一段这一趟的片（录入段定的片起点与片长，C790；不分片的段是 `None`），段号即下标：判段照它排计划、判、核对，不另算。
    pub segment_slices: Vec<Option<CrashPointReplaySegmentSlice>>,
}

/// 录入段那份字节的头：换了编码就换头，读到别的头拒绝，不猜（`SFRFLOW2`：每段多一格片，C790）。
const RECORDED_FLOW_MAGIC: &[u8; 8] = b"SFRFLOW2";

impl RecordedFlow {
    /// 录入段：把这一趟要判的流程（按段登记改过展开的那一份）抄成自己持有的一份；一段都不分片。
    #[must_use]
    pub fn of(flow: &CrashFlow<'_>) -> Self {
        Self::of_with_slices(flow, &no_segment_slices(flow.segments.len()))
    }

    /// 同 [`Self::of`]，带上每段这一趟的片（[`CrashPlan::segment_slices`]）。
    #[must_use]
    pub fn of_with_slices(
        flow: &CrashFlow<'_>,
        segment_slices: &[Option<CrashPointReplaySegmentSlice>],
    ) -> Self {
        Self {
            base: flow.base.clone(),
            writes: flow.writes.to_vec(),
            segments: flow.segments.to_vec(),
            judged_root_index: flow.judged_root_index,
            versions: flow.versions.to_vec(),
            versions_without_a_file: flow.versions_without_a_file.to_vec(),
            segment_expansions: flow
                .segments
                .iter()
                .enumerate()
                .map(|(segment_index, segment)| (flow.expansion)(segment_index, segment))
                .collect(),
            crash_points: flow.crash_points.clone(),
            segment_slices: segment_slices.to_vec(),
        }
    }

    /// 判段从库里读回的这一份当成一条流程用：展开方式照录入段登下的逐段取。
    pub fn with_the_flow<Outcome>(
        &self,
        use_the_flow: impl FnOnce(&CrashFlow<'_>) -> Outcome,
    ) -> Outcome {
        let expansion =
            |segment_index: usize, _segment: &[usize]| self.segment_expansions[segment_index];
        let flow = CrashFlow {
            base: &self.base,
            writes: &self.writes,
            segments: &self.segments,
            judged_root_index: self.judged_root_index,
            versions: &self.versions,
            versions_without_a_file: &self.versions_without_a_file,
            expansion: &expansion,
            crash_points: self.crash_points.clone(),
        };
        use_the_flow(&flow)
    }

    /// 写进库的字节（小端，变长的先写长度）。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = RECORDED_FLOW_MAGIC.to_vec();
        let put_u64 = |destination_bytes: &mut Vec<u8>, value: u64| {
            destination_bytes.extend_from_slice(&value.to_le_bytes())
        };
        let put_count = |destination_bytes: &mut Vec<u8>, count: usize| {
            destination_bytes
                .extend_from_slice(&u64::try_from(count).expect("长度装得进 u64").to_le_bytes());
        };
        let put_bytes = |destination_bytes: &mut Vec<u8>, content: &[u8]| {
            destination_bytes.extend_from_slice(
                &u64::try_from(content.len())
                    .expect("长度装得进 u64")
                    .to_le_bytes(),
            );
            destination_bytes.extend_from_slice(content);
        };
        put_pool_writes_and_segments(&mut bytes, &self.base, &self.writes, &self.segments);
        put_count(&mut bytes, self.judged_root_index);
        put_count(&mut bytes, self.versions.len());
        for version in &self.versions {
            bytes.extend_from_slice(&version.instance.0.to_le_bytes());
            put_u64(&mut bytes, version.checkpoint_txg.0);
            put_bytes(&mut bytes, &version.content);
        }
        put_count(&mut bytes, self.versions_without_a_file.len());
        for version in &self.versions_without_a_file {
            bytes.extend_from_slice(&version.instance.0.to_le_bytes());
            put_u64(&mut bytes, version.checkpoint_txg.0);
        }
        put_count(&mut bytes, self.segment_expansions.len());
        for expansion in &self.segment_expansions {
            bytes.push(segment_expansion_code(*expansion));
        }
        put_count(&mut bytes, self.crash_points.len());
        for point in &self.crash_points {
            put_bytes(&mut bytes, point.name.as_bytes());
            put_bytes(&mut bytes, point.code_file_in_repository.as_bytes());
            put_count(&mut bytes, point.writes.start);
            put_count(&mut bytes, point.writes.end);
            bytes.push(u8::from(point.ignored));
        }
        put_count(&mut bytes, self.segment_slices.len());
        for slice in &self.segment_slices {
            match slice {
                None => bytes.push(0),
                Some(slice) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&slice.start_within_the_domain.to_le_bytes());
                    put_u64(&mut bytes, slice.state_count);
                }
            }
        }
        bytes
    }

    /// 从库里的字节读回；头不对、字节不够、多出字节、码认不得都交回 `None`（判段照「库里那份读不回」停下，不猜）。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(RECORDED_FLOW_MAGIC.len())? != RECORDED_FLOW_MAGIC {
            return None;
        }
        let (base, writes, segments) = read_pool_writes_and_segments(&mut reader)?;
        let judged_root_index = reader.count()?;
        let mut versions = Vec::new();
        for _ in 0..reader.count()? {
            versions.push(PublishedVersion {
                instance: InstanceGeneration(reader.u32()?),
                checkpoint_txg: CheckpointTxg(reader.u64()?),
                content: reader.bytes()?.to_vec(),
            });
        }
        let mut versions_without_a_file = Vec::new();
        for _ in 0..reader.count()? {
            versions_without_a_file.push(PublishedVersionWithoutAFile {
                instance: InstanceGeneration(reader.u32()?),
                checkpoint_txg: CheckpointTxg(reader.u64()?),
            });
        }
        let mut segment_expansions = Vec::new();
        for _ in 0..reader.count()? {
            segment_expansions.push(segment_expansion_of_code(reader.u8()?)?);
        }
        let mut crash_points = Vec::new();
        for _ in 0..reader.count()? {
            let name = String::from_utf8(reader.bytes()?.to_vec()).ok()?;
            let code_file_in_repository = String::from_utf8(reader.bytes()?.to_vec()).ok()?;
            let start = reader.count()?;
            let end = reader.count()?;
            let ignored = match reader.u8()? {
                0 => false,
                1 => true,
                _ => return None,
            };
            crash_points.push(CrashPointSpan {
                name,
                code_file_in_repository,
                writes: start..end,
                ignored,
            });
        }
        let mut segment_slices = Vec::new();
        for _ in 0..reader.count()? {
            segment_slices.push(match reader.u8()? {
                0 => None,
                1 => Some(CrashPointReplaySegmentSlice {
                    start_within_the_domain: StateCount1280Bits::from_le_bytes(
                        reader.take(STATE_COUNT_BYTES)?.try_into().ok()?,
                    ),
                    state_count: reader.u64()?,
                }),
                _ => return None,
            });
        }
        if reader.at != bytes.len()
            || segment_expansions.len() != segments.len()
            || segment_slices.len() != segments.len()
        {
            return None;
        }
        Some(Self {
            base,
            writes,
            segments,
            judged_root_index,
            versions,
            versions_without_a_file,
            segment_expansions,
            crash_points,
            segment_slices,
        })
    }

    /// 这一份的摘要（库里按它存、按它读回核）。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        sha256_digest(&self.to_bytes())
    }
}

/// 读回的流逐段的展开方式（录入段登下的那一份），跨线程共用。
type ExpansionOfEachSegment =
    Box<dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion + Sync + Send>;

/// 从判定库里读回、核过摘要的一份流：判法（崩溃点重放枚举与按序号段判、判块、GPU 判、第 ② 段核对）只收它借出的 [`FlowOfTheLibrary`]。
/// 字段私有，只有 [`RecordedFlowReadBack::read_from_the_library`] 造得出：内存里现备的流交不进判法，要判先写进库再读回
/// （测试用 [`record_the_flow_and_read_it_back`]）。
pub struct RecordedFlowReadBack {
    recorded: RecordedFlow,
    expansion_of_each_segment: ExpansionOfEachSegment,
}

impl std::fmt::Debug for RecordedFlowReadBack {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RecordedFlowReadBack")
            .field("recorded", &self.recorded)
            .finish_non_exhaustive()
    }
}

impl RecordedFlowReadBack {
    /// 按摘要从库里读录下的流：读字节、核摘要、解码。
    ///
    /// # Errors
    /// 库读失败；库里没有这个摘要；字节的摘要对不上（库里那份被改过）；解不开。
    pub fn read_from_the_library(
        store: &VerdictStore,
        digest: &[u8; DIGEST_BYTES],
    ) -> Result<Self, String> {
        let bytes = store
            .read_recorded_flow(digest)
            .map_err(|error| format!("读录下的流程失败：{error:?}"))?
            .ok_or_else(|| "录下的流程不在库里：先跑录入段".to_string())?;
        if sha256_digest(&bytes) != *digest {
            return Err("录下的流程摘要与清单对不上：库里那份坏了，重跑录入段".to_string());
        }
        let recorded = RecordedFlow::from_bytes(&bytes)
            .ok_or_else(|| "录下的流程解不出（字节坏了或编码换了）：重跑录入段".to_string())?;
        let expansions = recorded.segment_expansions.clone();
        Ok(Self {
            recorded,
            expansion_of_each_segment: Box::new(move |segment_index, _segment| {
                expansions[segment_index]
            }),
        })
    }

    /// 读回的那一份（只读）。
    #[must_use]
    pub fn recorded(&self) -> &RecordedFlow {
        &self.recorded
    }

    /// 借出判法收的那一种流：展开方式照录入段登下的逐段取。
    #[must_use]
    pub fn flow(&self) -> FlowOfTheLibrary<'_> {
        FlowOfTheLibrary {
            flow: CrashFlow {
                base: &self.recorded.base,
                writes: &self.recorded.writes,
                segments: &self.recorded.segments,
                judged_root_index: self.recorded.judged_root_index,
                versions: &self.recorded.versions,
                versions_without_a_file: &self.recorded.versions_without_a_file,
                expansion: &*self.expansion_of_each_segment,
                crash_points: self.recorded.crash_points.clone(),
            },
        }
    }
}

/// 判法收的流：只能从 [`RecordedFlowReadBack::flow`] 借出（字段私有），所以判的一定是库里读回的那一份。读它照读 [`CrashFlow`]。
pub struct FlowOfTheLibrary<'flow> {
    flow: CrashFlow<'flow>,
}

impl<'flow> std::ops::Deref for FlowOfTheLibrary<'flow> {
    type Target = CrashFlow<'flow>;

    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

/// 测试与单进程跑法用：把内存里备好的流抄成 [`RecordedFlow`] 按摘要写进 `store`，再从 `store` 读回。判法只拿读回的那一份。
///
/// # Errors
/// 写不进库；读回失败（[`RecordedFlowReadBack::read_from_the_library`]）。
pub fn record_the_flow_and_read_it_back(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
) -> Result<RecordedFlowReadBack, String> {
    let recorded = RecordedFlow::of(flow);
    let digest = recorded.digest();
    store
        .store_recorded_flow(&digest, &recorded.to_bytes())
        .map_err(|error| format!("录下的流程写不进库：{error:?}"))?;
    drop(recorded);
    RecordedFlowReadBack::read_from_the_library(store, &digest)
}

/// 录入段那几份字节里共用的一截：基础镜像（盘大小、每块盘写过的扇区）、写表（盘、种类、FUA、偏移、内容或整段清零）、切段。
fn put_pool_writes_and_segments(
    bytes: &mut Vec<u8>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
) {
    let put_u64 = |destination_bytes: &mut Vec<u8>, value: u64| {
        destination_bytes.extend_from_slice(&value.to_le_bytes())
    };
    let put_count = |destination_bytes: &mut Vec<u8>, count: usize| {
        destination_bytes
            .extend_from_slice(&u64::try_from(count).expect("长度装得进 u64").to_le_bytes());
    };
    let put_bytes = |destination_bytes: &mut Vec<u8>, content: &[u8]| {
        destination_bytes.extend_from_slice(
            &u64::try_from(content.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        destination_bytes.extend_from_slice(content);
    };
    put_u64(bytes, base.device_size_in_bytes);
    put_count(bytes, base.devices.len());
    for (identity, device) in &base.devices {
        bytes.extend_from_slice(&identity.0.to_le_bytes());
        let sectors = device.written_sectors();
        put_count(bytes, sectors.len());
        for (sector, content) in sectors {
            put_u64(bytes, *sector);
            put_bytes(bytes, content);
        }
    }
    put_count(bytes, writes.len());
    for write in writes {
        bytes.extend_from_slice(&write.device.0.to_le_bytes());
        bytes.push(step_kind_code(write.kind));
        bytes.push(u8::from(write.is_force_unit_access));
        put_u64(bytes, write.offset.0);
        match &write.contents {
            WrittenContents::Bytes(content) => {
                bytes.push(0);
                put_bytes(bytes, content);
            }
            WrittenContents::Zeros { length } => {
                bytes.push(1);
                put_u64(bytes, *length);
            }
        }
    }
    put_count(bytes, segments.len());
    for segment in segments {
        put_count(bytes, segment.len());
        for write_index in segment {
            put_count(bytes, *write_index);
        }
    }
}

/// [`put_pool_writes_and_segments`] 写下的那一截读回来；字节不够、码认不得交回 `None`。
fn read_pool_writes_and_segments(
    reader: &mut RecordedBytesReader<'_>,
) -> Option<(MemoryPool, Vec<RetainedWrite>, Vec<Vec<usize>>)> {
    let device_size_in_bytes = reader.u64()?;
    let mut devices = BTreeMap::new();
    for _ in 0..reader.count()? {
        let identity = singlefs_core::address::DeviceIdentity(reader.u32()?);
        let mut device = singlefs_harness::memory_pool::SparseDevice::default();
        for _ in 0..reader.count()? {
            let sector = reader.u64()?;
            let content = reader.bytes()?;
            device.write(
                singlefs_core::address::DeviceOffsetInBytes(
                    sector.checked_mul(singlefs_harness::memory_pool::SECTOR_BYTES)?,
                ),
                content,
            );
        }
        devices.insert(identity, device);
    }
    let mut writes = Vec::new();
    for _ in 0..reader.count()? {
        let device = singlefs_core::address::DeviceIdentity(reader.u32()?);
        let kind = step_kind_of_code(reader.u8()?)?;
        let is_force_unit_access = match reader.u8()? {
            0 => false,
            1 => true,
            _ => return None,
        };
        let offset = singlefs_core::address::DeviceOffsetInBytes(reader.u64()?);
        let contents = match reader.u8()? {
            0 => WrittenContents::Bytes(reader.bytes()?.to_vec()),
            1 => WrittenContents::Zeros {
                length: reader.u64()?,
            },
            _ => return None,
        };
        writes.push(RetainedWrite {
            device,
            kind,
            is_force_unit_access,
            offset,
            contents,
        });
    }
    let mut segments = Vec::new();
    for _ in 0..reader.count()? {
        let mut segment = Vec::new();
        for _ in 0..reader.count()? {
            segment.push(reader.count()?);
        }
        segments.push(segment);
    }
    Some((
        MemoryPool {
            devices,
            device_size_in_bytes,
        },
        writes,
        segments,
    ))
}

/// 给定状态那一路写进库的一张表（崩溃注入、坏盘输入交来的状态表）：引擎先写进库、再从库里读回来判，判的是读回的这一份，
/// 不判调用方内存里的那一份（用户 2026-10-11 定：段与段之间不在内存里交数据）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedGivenStatesTable {
    pub table_name: String,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub seams: Vec<singlefs_harness::memory_pool::RecordStreamSeam>,
    pub persisted_sets: Vec<Vec<bool>>,
}

const RECORDED_GIVEN_STATES_TABLE_MAGIC: &[u8; 8] = b"SFGSTAB1";

impl RecordedGivenStatesTable {
    #[must_use]
    pub fn of(table: &crate::crash_given_states::GivenStatesOfATable<'_>) -> Self {
        Self {
            table_name: table.table_name.clone(),
            base: table.base.clone(),
            writes: table.writes.to_vec(),
            segments: table.segments.to_vec(),
            seams: table.seams.to_vec(),
            persisted_sets: table.persisted_sets.clone(),
        }
    }

    /// 读回的这一份当成一张给定状态表用。
    pub fn with_the_table<Outcome>(
        &self,
        use_the_table: impl FnOnce(&crate::crash_given_states::GivenStatesOfATable<'_>) -> Outcome,
    ) -> Outcome {
        use_the_table(&crate::crash_given_states::GivenStatesOfATable {
            table_name: self.table_name.clone(),
            base: &self.base,
            writes: &self.writes,
            segments: &self.segments,
            seams: &self.seams,
            persisted_sets: self.persisted_sets.clone(),
        })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        use singlefs_harness::memory_pool::RecordStreamSeam;
        let mut bytes = RECORDED_GIVEN_STATES_TABLE_MAGIC.to_vec();
        let put_u64 = |destination_bytes: &mut Vec<u8>, value: u64| {
            destination_bytes.extend_from_slice(&value.to_le_bytes())
        };
        let put_count = |destination_bytes: &mut Vec<u8>, count: usize| {
            destination_bytes
                .extend_from_slice(&u64::try_from(count).expect("长度装得进 u64").to_le_bytes());
        };
        put_count(&mut bytes, self.table_name.len());
        bytes.extend_from_slice(self.table_name.as_bytes());
        put_pool_writes_and_segments(&mut bytes, &self.base, &self.writes, &self.segments);
        put_count(&mut bytes, self.seams.len());
        for seam in &self.seams {
            match seam {
                RecordStreamSeam::AfterAPowerLoss {
                    first_write_after_the_power_loss,
                } => {
                    bytes.push(0);
                    put_count(&mut bytes, *first_write_after_the_power_loss);
                }
                RecordStreamSeam::AfterACrash {
                    first_write_after_the_crash,
                    version_landed_on_after_the_crash,
                } => {
                    bytes.push(1);
                    put_count(&mut bytes, *first_write_after_the_crash);
                    match version_landed_on_after_the_crash {
                        None => bytes.push(0),
                        Some((instance, checkpoint_txg)) => {
                            bytes.push(1);
                            bytes.extend_from_slice(&instance.0.to_le_bytes());
                            put_u64(&mut bytes, checkpoint_txg.0);
                        }
                    }
                }
            }
        }
        put_count(&mut bytes, self.persisted_sets.len());
        for persisted in &self.persisted_sets {
            put_count(&mut bytes, persisted.len());
            bytes.extend(persisted.iter().map(|is_persisted| u8::from(*is_persisted)));
        }
        bytes
    }

    /// 头不对、字节不够或多出、码认不得交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        use singlefs_harness::memory_pool::RecordStreamSeam;
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(RECORDED_GIVEN_STATES_TABLE_MAGIC.len())?
            != RECORDED_GIVEN_STATES_TABLE_MAGIC
        {
            return None;
        }
        let table_name = String::from_utf8(reader.bytes()?.to_vec()).ok()?;
        let (base, writes, segments) = read_pool_writes_and_segments(&mut reader)?;
        let mut seams = Vec::new();
        for _ in 0..reader.count()? {
            seams.push(match reader.u8()? {
                0 => RecordStreamSeam::AfterAPowerLoss {
                    first_write_after_the_power_loss: reader.count()?,
                },
                1 => RecordStreamSeam::AfterACrash {
                    first_write_after_the_crash: reader.count()?,
                    version_landed_on_after_the_crash: match reader.u8()? {
                        0 => None,
                        1 => Some((
                            InstanceGeneration(reader.u32()?),
                            CheckpointTxg(reader.u64()?),
                        )),
                        _ => return None,
                    },
                },
                _ => return None,
            });
        }
        let mut persisted_sets = Vec::new();
        for _ in 0..reader.count()? {
            let length = reader.count()?;
            let mut persisted = Vec::with_capacity(length);
            for byte in reader.take(length)? {
                persisted.push(match byte {
                    0 => false,
                    1 => true,
                    _ => return None,
                });
            }
            persisted_sets.push(persisted);
        }
        if reader.at != bytes.len() {
            return None;
        }
        Some(Self {
            table_name,
            base,
            writes,
            segments,
            seams,
            persisted_sets,
        })
    }
}

/// 给定状态表那一类的库里录下的一张表连同它的两栏（引擎录入段写进库的就是这一份，判段读回之后按两栏判）：
/// - 容器容量旋钮值（用户 2026-10-14 弹窗定「给定状态表加旋钮栏」）：CPU 判器调池级 checker 带旋钮值的入口，GPU 判器把旋钮值编进扫描内核。
///   旋钮不落盘，所以不在表的镜像里，跟着表另记一栏；没填的表是 [`ContainerCapacityKnobs::PRODUCT`]。
/// - 每个状态恢复自称的那一版（用户 2026-10-14 弹窗定「照字面改：表里记下根的来路」，[`ClaimedVersionOfAGivenState`]）：记录核对器第二条判据
///   照它取那一版（CPU 判器、GPU 恢复内核都照它）。逐状态一个，与表的持久集合同序同长；没填的表每个都是
///   [`ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedGivenStatesTableWithItsColumns {
    pub table: RecordedGivenStatesTable,
    pub container_capacity_knobs: ContainerCapacityKnobs,
    pub claimed_versions: Vec<ClaimedVersionOfAGivenState>,
}

/// 带两栏那一份字节的头（之后是三棵容器树各一个 8 字节小端的每片记录数，0 是没装旋钮；再是恢复自称的那一版的个数与逐个
/// （[`put_a_claimed_version`]）；再之后是 [`RecordedGivenStatesTable::to_bytes`]）。换了布局就换头：只带旋钮栏的那一种（`SFGSKNB1`）不认。
const RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC: &[u8; 8] = b"SFGSCOL1";

/// 恢复自称的那一版在录进库的字节里的码：看 journal 那一遍落到的那一版、写了而那一版没有根、写了一条根（后接实例代号 4 字节与 checkpoint_txg 8 字节，小端）。
const CLAIMED_VERSION_CODE_LANDED_ON_BY_THE_CONSULTED_RECOVERY: u8 = 0;
const CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT: u8 = 1;
const CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITH_A_ROOT: u8 = 2;

/// 一个状态那一栏恢复自称的那一版写进录进库的字节（码见 [`CLAIMED_VERSION_CODE_LANDED_ON_BY_THE_CONSULTED_RECOVERY`] 一族）。
fn put_a_claimed_version(bytes: &mut Vec<u8>, claimed_version: ClaimedVersionOfAGivenState) {
    match claimed_version {
        ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery => {
            bytes.push(CLAIMED_VERSION_CODE_LANDED_ON_BY_THE_CONSULTED_RECOVERY);
        }
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None) => {
            bytes.push(CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT);
        }
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
            instance,
            checkpoint_txg,
        ))) => {
            bytes.push(CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITH_A_ROOT);
            bytes.extend_from_slice(&instance.0.to_le_bytes());
            bytes.extend_from_slice(&checkpoint_txg.0.to_le_bytes());
        }
    }
}

/// [`put_a_claimed_version`] 的逆；码认不得、字节不够交回 `None`。
fn read_a_claimed_version(
    reader: &mut RecordedBytesReader<'_>,
) -> Option<ClaimedVersionOfAGivenState> {
    match reader.u8()? {
        CLAIMED_VERSION_CODE_LANDED_ON_BY_THE_CONSULTED_RECOVERY => {
            Some(ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery)
        }
        CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT => {
            Some(ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None))
        }
        CLAIMED_VERSION_CODE_NEWEST_ROOT_WRITTEN_WITH_A_ROOT => {
            let instance = InstanceGeneration(reader.u32()?);
            let checkpoint_txg = CheckpointTxg(reader.u64()?);
            Some(
                ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
                    instance,
                    checkpoint_txg,
                ))),
            )
        }
        _ => None,
    }
}

/// 一个状态那一栏恢复自称的那一版在块键输入里的一行（与录进库的字节同一组码）。
fn claimed_version_line_of_the_block_key(claimed_version: ClaimedVersionOfAGivenState) -> String {
    match claimed_version {
        ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery => {
            "claimed_version=landed_on_by_the_consulted_recovery".to_string()
        }
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None) => {
            "claimed_version=newest_root_written:none".to_string()
        }
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
            instance,
            checkpoint_txg,
        ))) => format!(
            "claimed_version=newest_root_written:{},{}",
            instance.0, checkpoint_txg.0
        ),
    }
}

impl RecordedGivenStatesTableWithItsColumns {
    #[must_use]
    pub fn of(table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>) -> Self {
        Self {
            table: RecordedGivenStatesTable::of(table_with_its_columns.table),
            container_capacity_knobs: table_with_its_columns.container_capacity_knobs,
            claimed_versions: table_with_its_columns
                .claimed_versions
                .of_every_state(table_with_its_columns.table.persisted_sets.len()),
        }
    }

    /// 读回的这一份当成一张带两栏的给定状态表用。
    pub fn with_the_table<Outcome>(
        &self,
        use_the_table: impl FnOnce(GivenStatesOfATableWithItsColumns<'_, '_>) -> Outcome,
    ) -> Outcome {
        let container_capacity_knobs = self.container_capacity_knobs;
        let claimed_versions = ClaimedVersionsOfTheStates::OfEachState(&self.claimed_versions);
        self.table.with_the_table(|table| {
            use_the_table(GivenStatesOfATableWithItsColumns {
                table,
                container_capacity_knobs,
                claimed_versions,
            })
        })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC.to_vec();
        for records_per_container in
            records_per_container_of_each_knob(self.container_capacity_knobs)
        {
            bytes.extend_from_slice(&records_per_container.to_le_bytes());
        }
        bytes.extend_from_slice(
            &u64::try_from(self.claimed_versions.len())
                .expect("状态数装得进 u64")
                .to_le_bytes(),
        );
        for claimed_version in &self.claimed_versions {
            put_a_claimed_version(&mut bytes, *claimed_version);
        }
        bytes.extend_from_slice(&self.table.to_bytes());
        bytes
    }

    /// 头不对、旋钮值越出它那一棵树的范围、恢复自称的那一版码认不得或个数与持久集合不等、表那一份解不出交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC.len())?
            != RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC
        {
            return None;
        }
        let tombstone = reader.u64()?;
        let lineage = reader.u64()?;
        let retirement = reader.u64()?;
        let container_capacity_knobs = ContainerCapacityKnobs {
            tombstone: match tombstone {
                0 => None,
                records_per_container => {
                    Some(TombstoneContainerCapacityKnob::new(records_per_container)?)
                }
            },
            lineage: match lineage {
                0 => None,
                records_per_container => {
                    Some(LineageContainerCapacityKnob::new(records_per_container)?)
                }
            },
            retirement: match retirement {
                0 => None,
                records_per_container => {
                    Some(RetirementContainerCapacityKnob::new(records_per_container)?)
                }
            },
        };
        let mut claimed_versions = Vec::new();
        for _ in 0..reader.count()? {
            claimed_versions.push(read_a_claimed_version(&mut reader)?);
        }
        let table = RecordedGivenStatesTable::from_bytes(&bytes[reader.at..])?;
        if claimed_versions.len() != table.persisted_sets.len() {
            return None;
        }
        Some(Self {
            table,
            container_capacity_knobs,
            claimed_versions,
        })
    }
}
/// 三棵容器树的旋钮值，按墓碑树、谱系树、退役树的次序各一个每片记录数，0 是没装旋钮（录进库的那一栏与块键都照这一种写法）。
fn records_per_container_of_each_knob(knobs: ContainerCapacityKnobs) -> [u64; 3] {
    [
        knobs
            .tombstone
            .map_or(0, TombstoneContainerCapacityKnob::records_per_container),
        knobs
            .lineage
            .map_or(0, LineageContainerCapacityKnob::records_per_container),
        knobs
            .retirement
            .map_or(0, RetirementContainerCapacityKnob::records_per_container),
    ]
}

/// 读录入段那份字节的游标：读一样往前走一样，不够就交回 `None`。
struct RecordedBytesReader<'bytes> {
    bytes: &'bytes [u8],
    at: usize,
}

impl<'bytes> RecordedBytesReader<'bytes> {
    fn take(&mut self, length: usize) -> Option<&'bytes [u8]> {
        let end = self.at.checked_add(length)?;
        let taken = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(taken)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|taken| taken[0])
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    fn count(&mut self) -> Option<usize> {
        usize::try_from(self.u64()?).ok()
    }
    fn bytes(&mut self) -> Option<&'bytes [u8]> {
        let length = self.count()?;
        self.take(length)
    }
}

fn step_kind_code(kind: StepKind) -> u8 {
    match kind {
        StepKind::ZeroFill => 0,
        StepKind::UnitWrite => 1,
        StepKind::JournalRecord => 2,
        StepKind::RootRecordFua => 3,
        StepKind::SystemConfigurationSlot => 4,
        StepKind::Barrier => 5,
    }
}

fn step_kind_of_code(code: u8) -> Option<StepKind> {
    match code {
        0 => Some(StepKind::ZeroFill),
        1 => Some(StepKind::UnitWrite),
        2 => Some(StepKind::JournalRecord),
        3 => Some(StepKind::RootRecordFua),
        4 => Some(StepKind::SystemConfigurationSlot),
        5 => Some(StepKind::Barrier),
        _ => None,
    }
}

fn segment_expansion_code(expansion: CrashPointReplaySegmentExpansion) -> u8 {
    match expansion {
        CrashPointReplaySegmentExpansion::NotExpanded => 0,
        CrashPointReplaySegmentExpansion::EveryProperSubset => 1,
        CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => 2,
    }
}

fn segment_expansion_of_code(code: u8) -> Option<CrashPointReplaySegmentExpansion> {
    match code {
        0 => Some(CrashPointReplaySegmentExpansion::NotExpanded),
        1 => Some(CrashPointReplaySegmentExpansion::EveryProperSubset),
        2 => Some(CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll),
        _ => None,
    }
}

/// 录入段一趟登在库里的清单（键是流名）：这条流录下的几样各自的摘要，加上录入段定下的按段登记与全域判定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedRunManifest {
    pub recorded_flow_digest: [u8; DIGEST_BYTES],
    pub facts_digest: [u8; DIGEST_BYTES],
    /// 配了 GPU 时录入段打包好的判表的摘要；没配 GPU 是 `None`。
    pub packed_judge_tables_digest: Option<[u8; DIGEST_BYTES]>,
    pub registered_segments: Vec<RegisteredSegment>,
    pub exhaustive: bool,
}

const RECORDED_RUN_MANIFEST_MAGIC: &[u8; 8] = b"SFRRUN01";

impl RecordedRunManifest {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = RECORDED_RUN_MANIFEST_MAGIC.to_vec();
        bytes.extend_from_slice(&self.recorded_flow_digest);
        bytes.extend_from_slice(&self.facts_digest);
        match &self.packed_judge_tables_digest {
            Some(digest) => {
                bytes.push(1);
                bytes.extend_from_slice(digest);
            }
            None => bytes.push(0),
        }
        bytes.push(u8::from(self.exhaustive));
        bytes.extend_from_slice(
            &u64::try_from(self.registered_segments.len())
                .expect("段数装得进 u64")
                .to_le_bytes(),
        );
        for registered in &self.registered_segments {
            bytes.extend_from_slice(
                &u64::try_from(registered.segment_index)
                    .expect("段号装得进 u64")
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(
                &u64::try_from(registered.write_count)
                    .expect("段写数装得进 u64")
                    .to_le_bytes(),
            );
            let text = registered.write_table_digest_text.as_bytes();
            bytes.extend_from_slice(&u64::try_from(text.len()).expect("装得进 u64").to_le_bytes());
            bytes.extend_from_slice(text);
        }
        bytes
    }

    /// 头不对、字节不够或多出、码认不得交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(RECORDED_RUN_MANIFEST_MAGIC.len())? != RECORDED_RUN_MANIFEST_MAGIC {
            return None;
        }
        let recorded_flow_digest: [u8; DIGEST_BYTES] =
            reader.take(DIGEST_BYTES)?.try_into().ok()?;
        let facts_digest: [u8; DIGEST_BYTES] = reader.take(DIGEST_BYTES)?.try_into().ok()?;
        let packed_judge_tables_digest = match reader.u8()? {
            0 => None,
            1 => Some(reader.take(DIGEST_BYTES)?.try_into().ok()?),
            _ => return None,
        };
        let exhaustive = match reader.u8()? {
            0 => false,
            1 => true,
            _ => return None,
        };
        let mut registered_segments = Vec::new();
        for _ in 0..reader.count()? {
            registered_segments.push(RegisteredSegment {
                segment_index: reader.count()?,
                write_count: reader.count()?,
                write_table_digest_text: String::from_utf8(reader.bytes()?.to_vec()).ok()?,
            });
        }
        if reader.at != bytes.len() {
            return None;
        }
        Some(Self {
            recorded_flow_digest,
            facts_digest,
            packed_judge_tables_digest,
            registered_segments,
            exhaustive,
        })
    }
}

/// 录入段（CPU 跑数据、写进库）：写录入表；把这条流程（按段登记改过展开的那一份）、照计划抽的事实表、配了 GPU 时打包好的判表
/// 各写进库；最后登这一趟的清单（键是流名）。判段只认清单指到的那几样。
///
/// # Errors
/// 库写失败；配了 GPU 而判器建不出这条流的表（不退回 CPU 判器）。
pub fn record_the_flow_into_the_library(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    flow_name: &str,
    plan: &CrashPlan,
    registered_segments: &[RegisteredSegment],
    exhaustive: bool,
    gpu_judges: bool,
) -> Result<RecordedRunManifest, String> {
    write_the_recorded_blocks(store, plan).map_err(|error| format!("录入失败：{error:?}"))?;
    let recorded = RecordedFlow::of_with_slices(flow, plan.segment_slices());
    let recorded_bytes = recorded.to_bytes();
    let recorded_flow_digest = sha256_digest(&recorded_bytes);
    store
        .store_recorded_flow(&recorded_flow_digest, &recorded_bytes)
        .map_err(|error| format!("录下的流程写不进库：{error:?}"))?;
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let facts = extract_flow_facts_with_slices(
        flow.base,
        flow.writes,
        flow.segments,
        &expansion,
        plan.segment_slices(),
    );
    let facts_digest = facts.digest();
    store
        .store_flow_facts(&facts_digest, &facts.to_bytes())
        .map_err(|error| format!("事实表写不进库：{error:?}"))?;
    #[cfg(not(feature = "gpu"))]
    if gpu_judges {
        return Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string());
    }
    #[cfg(feature = "gpu")]
    let packed_judge_tables_digest = if gpu_judges {
        let packed_bytes = packed_judge_tables_of_the_plan(flow, plan)?.to_bytes();
        let digest = sha256_digest(&packed_bytes);
        store
            .store_packed_judge_tables(&digest, &packed_bytes)
            .map_err(|error| format!("GPU 判表写不进库：{error:?}"))?;
        Some(digest)
    } else {
        None
    };
    #[cfg(not(feature = "gpu"))]
    let packed_judge_tables_digest = None;
    let manifest = RecordedRunManifest {
        recorded_flow_digest,
        facts_digest,
        packed_judge_tables_digest,
        registered_segments: registered_segments.to_vec(),
        exhaustive,
    };
    store
        .store_recorded_run_manifest(flow_name, &manifest.to_bytes())
        .map_err(|error| format!("录入段的清单写不进库：{error:?}"))?;
    Ok(manifest)
}

/// 判段从库里读回的 GPU 判表（GPU 判器只收它）：编进 `gpu` 特性时是读回的打包判表；没编进时不会有（配置开 GPU 而没编进就在选路时报错），占个类型。
#[cfg(feature = "gpu")]
pub type PackedJudgeTablesOfTheRun = crate::crash_judge_dispatch::PackedJudgeTablesReadBack;
/// 没编进 `gpu` 特性：不会有 GPU 判表，造不出这个类型的值。
#[cfg(not(feature = "gpu"))]
#[derive(Debug)]
pub enum PackedJudgeTablesOfTheRun {}

/// 照这份计划的展开把一条流的 GPU 判表建好、打包（录入段写进库的就是这一份；对拍用例直接调卡上判的入口时也照它建）。
///
/// # Errors
/// 判器建不出这条流的表（不退回 CPU 判器）。
#[cfg(feature = "gpu")]
pub fn packed_judge_tables_of_the_plan(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
) -> Result<crate::crash_judge_dispatch::PackedJudgeTables, String> {
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let judge_flow = crate::crash_judge_tables::JudgeFlow {
        base: flow.base,
        writes: flow.writes,
        segments: flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: flow.versions,
        expansion: &expansion,
    };
    crate::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices(
        &judge_flow,
        flow.versions_without_a_file,
        plan.segment_slices(),
    )
    .map(|tables| crate::crash_judge_dispatch::PackedJudgeTables::of(&tables))
    .map_err(|refusal| {
        format!(
            "GPU 判器建不出这条流的表（{refusal:?}）。配置开了 GPU，不退回 CPU 判器：让判器的表认得这种写再跑"
        )
    })
}

/// 判段按摘要从库里读回 GPU 判表（[`crate::crash_library::read_packed_judge_tables_back`]）。
#[cfg(feature = "gpu")]
pub(crate) fn packed_judge_tables_read_back(
    store: &VerdictStore,
    digest: &[u8; DIGEST_BYTES],
) -> Result<PackedJudgeTablesOfTheRun, String> {
    crate::crash_library::read_packed_judge_tables_back(store, digest)
}

/// 没编进 `gpu` 特性读不出 GPU 判表（也不会去读：要 GPU 判时选路先报错）。
#[cfg(not(feature = "gpu"))]
pub(crate) fn packed_judge_tables_read_back(
    _store: &VerdictStore,
    _digest: &[u8; DIGEST_BYTES],
) -> Result<PackedJudgeTablesOfTheRun, String> {
    Err("这个二进制没编进 gpu 特性，读不了 GPU 判表".to_string())
}

/// 判段从库里读回的这一趟：清单、流程、事实表、配了 GPU 时的判表。都是从库里的字节解出来的，不借录入段内存里的任何东西。
pub struct RecordedRunReadBack {
    pub manifest: RecordedRunManifest,
    pub flow: RecordedFlowReadBack,
    pub facts: crate::crash_facts::FlowFactsReadBack,
    pub packed_judge_tables: Option<PackedJudgeTablesOfTheRun>,
}

/// 判段：凭流名读录入段登的清单，再读清单指到的几样，逐样核摘要。
///
/// # Errors
/// 库里没有这条流的清单（录入段没跑）、清单指到的哪一样不在、字节解不出、摘要对不上；要 GPU 判而清单里没有判表。
pub fn read_back_the_recorded_run(
    store: &VerdictStore,
    flow_name: &str,
    gpu_judges: bool,
) -> Result<RecordedRunReadBack, String> {
    let manifest_bytes = store
        .read_recorded_run_manifest(flow_name)
        .map_err(|error| format!("读录入段的清单失败：{error:?}"))?
        .ok_or_else(|| {
            format!(
                "库里没有流 {flow_name} 的录入段清单：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）"
            )
        })?;
    let manifest = RecordedRunManifest::from_bytes(&manifest_bytes).ok_or_else(|| {
        format!("流 {flow_name} 的录入段清单解不出（字节坏了或编码换了）：重跑录入段")
    })?;
    let flow = RecordedFlowReadBack::read_from_the_library(store, &manifest.recorded_flow_digest)
        .map_err(|refusal| format!("流 {flow_name}：{refusal}"))?;
    let facts = crate::crash_library::read_flow_facts_back(store, &manifest.facts_digest)
        .map_err(|refusal| format!("流 {flow_name} 的清单指到的事实表：{refusal}"))?;
    let packed_judge_tables = match (&manifest.packed_judge_tables_digest, gpu_judges) {
        (Some(digest), true) => Some(
            packed_judge_tables_read_back(store, digest)
                .map_err(|refusal| format!("流 {flow_name} 的清单指到的 GPU 判表：{refusal}"))?,
        ),
        (None, true) => {
            return Err(format!(
                "配置开了 GPU，流 {flow_name} 的录入段清单里却没有 GPU 判表（录入段那一趟没开 GPU）：照这份配置重跑录入段"
            ));
        }
        (_, false) => None,
    };
    Ok(RecordedRunReadBack {
        manifest,
        flow,
        facts,
        packed_judge_tables,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // 扫源码的用例里，串里的花括号一律写成 \u 转义：门禁 code-source-discipline 的 mutation-tables 格配花括号时不跳字符串，
    // 串里落单的半边括号会让它把后面的用例算到别的模块下（第 10z 批第八块）。
    /// 这份文件测试模块的头两行（扫源码的用例从这里切掉测试模块）。
    const TESTS_MODULE_OPENING: &str = "#[cfg(test)]\nmod tests \u{7b}";
    /// 顶格的右括号自成一行（测试模块在这里收尾）。
    const TOP_LEVEL_CLOSING_BRACE_LINE: &str = "\n\u{7d}\n";

    /// 配置点名的卡少了哪几张：按配置的次序列出用不上的序号；都在就是空。
    #[test]
    fn configured_cards_that_are_not_usable_are_named_in_the_configured_order() {
        assert_eq!(
            configured_cards_not_usable(&[(0, 32), (1, 16), (2, 16)], &[1]),
            vec![0, 2]
        );
        assert_eq!(
            configured_cards_not_usable(&[(0, 32), (1, 16)], &[1, 0, 3]),
            Vec::<u32>::new()
        );
    }

    /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 7 处）：池里一张卡此刻照这一刻能借的算带得了几个状态，空着时照空着时能借的算。不要卡。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_pool_card_holds_states_now_by_its_room_now_and_when_empty_by_its_ceiling() {
        assert_eq!(
            states_in_flight_now_and_when_the_card_is_empty(
                singlefs_harness::gpu_memory_ledger::CardRoomForThisProcess {
                    room_now_mebibytes: 100,
                    ceiling_mebibytes: 1_000,
                },
                &|room_mebibytes| room_mebibytes / 10
            ),
            (10, 100)
        );
    }

    /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 6 处）：借草稿区之前每张卡此刻都带不了一个状态、至少一张空着时带得了，这一步是报本次未跑
    /// （GPU 用例的记号、原因写显存不够、带每张卡此刻能借的）；空着时也带不了才判红；有一张此刻带得了就去借。不要卡。
    #[cfg(feature = "gpu")]
    #[test]
    fn with_no_card_holding_a_state_now_the_step_before_booking_is_a_not_run_line_not_red() {
        let run_cards = [CardCapacityOfThisRun {
            index: 0,
            name: "造的一张卡".to_string(),
            max_storage_buffer_binding_size: 1 << 30,
            memory_budget_mebibytes: crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
                + 700,
            states_in_flight_capacity: 0,
            states_in_flight_capacity_when_the_card_is_empty: 3,
        }];
        match scratch_step_before_booking(ScratchBeforeBooking::NoCardHoldsAStateNow, &run_cards, 900)
        {
            ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => assert!(
                not_run_line.starts_with(
                    "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=显存不够："
                ) && not_run_line.contains("900 MiB")
                    && not_run_line.contains("[700]"),
                "{not_run_line}"
            ),
            other @ (ScratchStepBeforeBooking::BookOnTheCards
            | ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty) => {
                panic!("此刻借不到：要报本次未跑，判成了 {other:?}")
            }
        }
        assert_eq!(
            scratch_step_before_booking(
                ScratchBeforeBooking::NoCardHoldsAStateEvenWhenEmpty,
                &run_cards,
                900
            ),
            ScratchStepBeforeBooking::RedNoCardHoldsAStateEvenWhenEmpty
        );
        assert_eq!(
            scratch_step_before_booking(
                ScratchBeforeBooking::BookOnTheCardsThatHoldAStateNow,
                &run_cards,
                900
            ),
            ScratchStepBeforeBooking::BookOnTheCards
        );
    }

    /// 第 10z 批（第 10y 批代码轮正推 K3 列的第 5 处）：产品路径向卡池借不到时，借到的卡上起设备显存不够与此刻借不到都打本次未跑那一行、
    /// 交 `Ok`；要的比每张卡空着时都大、卡上的显存读不出判红。不要卡：结局照卡池交回的造；只调不打印的那一半
    /// （第 10z 批第七块：打印那一半会把造出来的那一行打进测试段的标准输出，门禁收成本次未跑）。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_device_short_of_gpu_memory_on_the_card_lent_is_reported_not_run_like_no_room_now() {
        use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
        assert!(
            not_run_line_or_refusal_of_a_lease(
                "单元校验和对拍的缓冲",
                &CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
                    reason: "显存不够：起设备时".to_string()
                }
            )
            .is_ok_and(|line| line.starts_with("NOT_RUN_NO_GPU_ADAPTER ")
                && line.ends_with("reason=显存不够：起设备时")),
            "起设备时显存不够：本次未跑"
        );
        assert!(
            not_run_line_or_refusal_of_a_lease(
                "单元校验和对拍的缓冲",
                &CardPoolLeaseRefusal::Booking(BookingRefusal::NoCardHasRoomNow {
                    need_mebibytes: 900,
                    card_rooms_mebibytes: vec![700],
                })
            )
            .is_ok_and(|line| line.starts_with("NOT_RUN_NO_GPU_ADAPTER ")),
            "此刻借不到：本次未跑"
        );
        assert!(
            not_run_line_or_refusal_of_a_lease(
                "单元校验和对拍的缓冲",
                &CardPoolLeaseRefusal::Booking(BookingRefusal::NeedLargerThanEveryCard(
                    NeedLargerThanEveryCard {
                        need_mebibytes: 40_000,
                        largest_ceiling_mebibytes: 29_550,
                    }
                ))
            )
            .is_err(),
            "比每张卡空着时都大：判红"
        );
        assert!(
            not_run_line_or_refusal_of_a_lease(
                "单元校验和对拍的缓冲",
                &CardPoolLeaseRefusal::Booking(BookingRefusal::CardMemoryUnreadable {
                    reason: "nvidia-smi 起不来".to_string()
                })
            )
            .is_err(),
            "读不出：判红"
        );
    }

    /// 给定状态引擎补的第 ③ 样：给定状态表的判器借不到显存（此刻借不到、借到的卡上起设备时显存不够）照运行条件不满足报
    /// （[`GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX`] 起头、点名那张表，入口照它退 78），行里不带「本次未跑」的记号（不报本次未跑、
    /// 不换成 CPU 判器的结论往下走）；要的比每张卡空着时能借的都大照旧那一句判红。不要卡：只造那一句。
    /// 判别力：那一句换成「本次未跑」的记号起头（`crates/mutations.tsv` 那一行），入口就把它记成本次未跑。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_given_states_judge_that_cannot_lease_gpu_memory_is_a_run_condition_not_met_and_not_a_not_run_line(
    ) {
        use crate::gpu_unit_checks::gpu_test_start::CardPoolLeaseRefusal;
        use singlefs_harness::gpu_memory_ledger::GPU_NOT_RUN_MARKER;
        for refusal in [
            CardPoolLeaseRefusal::Booking(BookingRefusal::NoCardHasRoomNow {
                need_mebibytes: 900,
                card_rooms_mebibytes: vec![100, 200],
            }),
            CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {
                reason: "显存不够：第 0 张卡上起设备时 wgpu 报了 Out of Memory".to_string(),
            },
        ] {
            let reason = given_states_lease_refusal_as_a_run_condition_not_met(
                "seed=1/first-crash",
                &refusal,
            );
            assert!(
                reason.starts_with(GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX)
                    && reason.contains("table=seed=1/first-crash")
                    && !reason.contains(GPU_NOT_RUN_MARKER),
                "借不到显存：运行条件不满足那一行、点名那张表、不带本次未跑的记号：{reason}"
            );
        }
        let larger_than_every_card = given_states_lease_refusal_as_a_run_condition_not_met(
            "seed=1/first-crash",
            &CardPoolLeaseRefusal::Booking(BookingRefusal::NeedLargerThanEveryCard(
                NeedLargerThanEveryCard {
                    need_mebibytes: 90_000,
                    largest_ceiling_mebibytes: 30_000,
                },
            )),
        );
        assert!(
            !larger_than_every_card.starts_with(GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX)
                && larger_than_every_card.contains("没有一张卡装得下"),
            "要的比每张卡空着时都大：照旧那一句（判红）：{larger_than_every_card}"
        );
    }

    /// 第 10z 批第七块：产品路径借不到时那一行照旧打进标准输出（造那一行与打印分开之后，打印那一半交 `Ok` 之前打它；
    /// 不要卡的用例调的是不打印的那一半，钉不住这一句）。扫这份源码，不要卡。
    #[test]
    fn the_product_path_prints_the_not_run_line_of_a_lease_it_could_not_get() {
        let lines = code_lines_outside_the_tests();
        let start = lines
            .iter()
            .position(|line| line.trim() == "fn print_the_not_run_line_or_refuse(")
            .expect("产品路径有打印借不到那一行的那一半");
        let body = lines[start..]
            .iter()
            .take_while(|line| line.as_str() != "\u{7d}")
            .map(|line| line.trim().to_string())
            .collect::<Vec<String>>();
        let line_made = body
            .iter()
            .position(|line| {
                line == "let not_run_line = not_run_line_or_refusal_of_a_lease(what_was_booked, refusal)?;"
            })
            .unwrap_or_else(|| panic!("打印那一半先照不打印的那一半造那一行：{body:?}"));
        let line_printed = body
            .iter()
            .position(|line| line == "println!(\"{not_run_line}\");")
            .unwrap_or_else(|| panic!("打印那一半要把造出来的那一行打进标准输出：{body:?}"));
        let line_handed_back = body
            .iter()
            .position(|line| line == "Ok(not_run_line)")
            .unwrap_or_else(|| panic!("打印那一半交回造出来的那一行：{body:?}"));
        assert!(
            line_made < line_printed && line_printed < line_handed_back,
            "先造、再打、再交回：{body:?}"
        );
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.contains("print_the_not_run_line_or_refuse("))
                .count(),
            3,
            "产品路径的两处借不到（第 ① 段判器的草稿区、第 ② 段核对的缓冲）都经打印的那一半（加上它自己的定义一处）"
        );
    }

    /// 一块里第一个判不了的状态：恢复内核的「判不了」位或 checker 内核的「判不了」字，哪一样都算；全判得了是 `None`。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_first_state_the_gpu_could_not_judge_is_found_by_either_kernel() {
        use crate::crash_judge_gpu::{verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS};
        let judged = [0u32; VERDICT_WORDS];
        let mut recovery_gave_up = judged;
        recovery_gave_up[0] = 1 << verdict_bits::UNDECIDABLE;
        let mut checker_gave_up = judged;
        checker_gave_up[POOL_CHECKER_UNDECIDABLE_WORD] = 3;
        assert_eq!(first_state_the_gpu_could_not_judge(&[judged, judged]), None);
        assert_eq!(
            first_state_the_gpu_could_not_judge(&[judged, recovery_gave_up, checker_gave_up]),
            Some(1)
        );
        assert_eq!(
            first_state_the_gpu_could_not_judge(&[judged, judged, checker_gave_up]),
            Some(2)
        );
    }

    /// 第 10w 批：第 ② 段核对在卡上建的缓冲进账的量（文件头第 5 条）。录制流 3 次写（12 个字）、一张撕裂镜像带 2 对重放（4 个字）、
    /// 2 段（每段 11 个字，22 个字；段表加了展开方式码、整组落的单元写那一组与分片的段片起点各位那一组之后的宽）、段里共 3 次写（3 个字），
    /// 都不分片，单元与根都没有（空表各建一个字）：上卡一次 43 个字 172 字节；一块 65 536 个状态一趟建头 12 个字、结论与回读各 65 536 个字，
    /// 同时两趟活着：合起来 1 048 844 字节，向上取整 2 MiB。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_facts_verifier_books_its_tables_once_and_two_blocks_of_header_verdicts_and_readback() {
        let facts = FlowFacts {
            recorded_write_count: 3,
            enumerated_write_count: 3,
            is_tearable: vec![true, false, false],
            torn_images: vec![crate::crash_facts::TornImageFact {
                write_index: 0,
                torn_image_index: 0,
                replays: vec![(0, 1), (1, 2)],
            }],
            segments: vec![
                crate::crash_facts::SegmentFact {
                    writes: vec![0, 1],
                    states: 0..4,
                    slice_start: None,
                },
                crate::crash_facts::SegmentFact {
                    writes: vec![2],
                    states: 4..6,
                    slice_start: None,
                },
            ],
            state_count: 6,
            units: Vec::new(),
            roots: Vec::new(),
            newest_base_root: None,
        };
        let footprint = facts_verifier_buffer_footprint(&facts, STATES_PER_BLOCK);
        assert_eq!(
            footprint,
            GpuBufferFootprint {
                bytes_uploaded_once: 172,
                bytes_of_one_pass: (12 + 2 * 65_536) * 4,
                passes_alive_at_once: 2,
            },
            "六张表上卡一次（空表各一个字），一块一趟建头、结论与回读"
        );
        assert_eq!(footprint.mebibytes(), 2);
    }

    /// 显卡优先级照写的次序读：写在前面的先装；额度带单位（G、M，1024 进制）。写法不对的每一种都拒。
    #[test]
    fn cards_by_priority_are_read_in_the_order_written_with_their_quotas() {
        assert_eq!(
            parse_cards_by_priority("local:0:15G  peer:2:10G"),
            Ok(vec![
                CardByPriority {
                    host: CardHost::Local,
                    index: 0,
                    memory_quota_mebibytes: 15 * 1024,
                },
                CardByPriority {
                    host: CardHost::Peer,
                    index: 2,
                    memory_quota_mebibytes: 10 * 1024,
                },
            ])
        );
        assert_eq!(
            parse_cards_by_priority("peer:1:512M"),
            Ok(vec![CardByPriority {
                host: CardHost::Peer,
                index: 1,
                memory_quota_mebibytes: 512,
            }])
        );
        for refused in [
            "",
            "0:15G",
            "local:0",
            "remote:0:15G",
            "local:first:15G",
            "local:0:15",
            "local:0:0G",
            "local:0:G",
            "local:0:15G local:0:10G",
        ] {
            assert!(
                parse_cards_by_priority(refused).is_err(),
                "「{refused}」该被拒"
            );
        }
        // 同一个序号在两台机器上是两张卡
        assert_eq!(
            parse_cards_by_priority("local:0:15G peer:0:10G").map(|cards| cards.len()),
            Ok(2)
        );
    }

    /// 任务先装第一张卡，装到它带得了的状态数为止，装不下的才给下一张；都装满还有剩的排队等，不算在任何一张卡上。
    #[test]
    fn states_fill_the_first_card_before_the_next_card_gets_any() {
        assert_eq!(
            place_states_on_cards_by_priority(100, &[150, 100]),
            [100, 0]
        );
        assert_eq!(
            place_states_on_cards_by_priority(150, &[150, 100]),
            [150, 0]
        );
        assert_eq!(
            place_states_on_cards_by_priority(151, &[150, 100]),
            [150, 1]
        );
        assert_eq!(
            place_states_on_cards_by_priority(1000, &[150, 100]),
            [150, 100]
        );
        assert_eq!(place_states_on_cards_by_priority(0, &[150, 100]), [0, 0]);
        // 第一张卡一个状态都带不了（表装不进它的额度）：整份给下一张
        assert_eq!(place_states_on_cards_by_priority(10, &[0, 100]), [0, 10]);
    }

    /// 配置的值去掉一层引号；键没写是 `None`；环境变量 `GPU_CARDS_BY_PRIORITY` 没设时显卡优先级从配置文件读。
    /// 环境走 K5 那一个守卫：那个变量设了就盖过配置文件，用例清掉它、跑完（或 panic）还原，不读调用方留下的值。
    #[test]
    fn the_card_priority_is_read_from_the_configuration_file_with_one_layer_of_quotes_removed() {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        let directory = std::env::temp_dir().join(format!(
            "singlefs-crash-amplification-card-priority-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("建得了");
        let file = directory.join("multi-host.env");
        std::fs::write(
            &file,
            "ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY='local:0:15G peer:2:10G'\nQUOTED=\"two words\"\n",
        )
        .expect("写得了");
        assert_eq!(
            configuration_value(&file, "GPU_CARDS_BY_PRIORITY").as_deref(),
            Some("local:0:15G peer:2:10G")
        );
        assert_eq!(
            configuration_value(&file, "QUOTED").as_deref(),
            Some("two words")
        );
        assert_eq!(
            configuration_value(&file, "ENABLE_GPU").as_deref(),
            Some("1")
        );
        assert_eq!(configuration_value(&file, "NOT_WRITTEN"), None);
        // 清没清场写明不清场：这条用例在清场包装之下跑时，清过场而只写了不清场那一份要报错（gpu_card_configuration 那一条用例钉），
        // 这里只钉「从配置文件读」这一样
        assert_eq!(
            crate::gpu_card_configuration::cards_by_priority_from(
                std::env::var("GPU_CARDS_BY_PRIORITY").ok(),
                Some(&file),
                crate::gpu_card_configuration::GpuQuiesce::NotQuiesced
            )
            .expect("写法对")
            .map(|cards| cards.len()),
            Some(2)
        );
        let without_the_key = directory.join("without-the-key.env");
        std::fs::write(&without_the_key, "ENABLE_GPU=1\n").expect("写得了");
        // 同上写明不清场：清过场而开了 GPU、两份显卡先后都没写要报错（gpu_card_configuration 那一条用例钉）
        assert_eq!(
            crate::gpu_card_configuration::cards_by_priority_from(
                None,
                Some(&without_the_key),
                crate::gpu_card_configuration::GpuQuiesce::NotQuiesced
            ),
            Ok(None)
        );
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// 库落在开机会清空的目录下就拒（没有临时库这一说，没有放行的开关）；家目录这类持久目录放行；目录还没建也按它的上层判。
    /// 环境走 K5 那一个守卫：清掉一趟读的变量、跑完（或 panic）还原。
    #[test]
    fn a_library_under_a_directory_a_reboot_erases_is_refused_and_a_persistent_one_is_not() {
        let environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        let refused = refuse_a_library_that_a_reboot_would_erase(Path::new(
            "/tmp/singlefs-crash-amplification-not-yet-created/library",
        ));
        assert!(
            refused
                .as_ref()
                .is_err_and(|reason| reason.contains("/tmp")),
            "/tmp 之下的库要拒（目录还没建也按上层判）：{refused:?}"
        );
        for erased in ["/var/tmp/x", "/dev/shm/x", "/run/user/1000/x"] {
            assert!(
                refuse_a_library_that_a_reboot_would_erase(Path::new(erased)).is_err(),
                "{erased} 之下的库要拒"
            );
        }
        let home = std::env::var_os("HOME").expect("有 HOME");
        let persistent = Path::new(&home).join("singlefs-crash-amplification-test-library");
        assert_eq!(
            refuse_a_library_that_a_reboot_would_erase(&persistent),
            Ok(()),
            "家目录下的库放行"
        );
        drop(environment);
    }

    #[test]
    fn weighted_shares_hand_out_blocks_in_proportion_and_never_skip_a_block() {
        let shares = WeightedShares::new(vec![3, 1]);
        let owners: Vec<usize> = (0..8).map(|block| shares.owner_of_block(block)).collect();
        assert_eq!(
            owners.iter().filter(|owner| **owner == 0).count(),
            6,
            "权重 3:1 的前 8 块里第一方拿 6 块"
        );
        assert_eq!(owners.iter().filter(|owner| **owner == 1).count(), 2);
        let equal = WeightedShares::new(vec![1, 1]);
        assert_eq!(
            (0..6)
                .map(|block| equal.owner_of_block(block))
                .collect::<Vec<usize>>(),
            vec![0, 1, 0, 1, 0, 1]
        );
    }

    #[test]
    fn the_four_run_modes_come_from_the_two_switches() {
        assert_eq!(
            CrashAmplificationRunMode::from_switches(false, false),
            CrashAmplificationRunMode::SingleHostCpu
        );
        assert_eq!(
            CrashAmplificationRunMode::from_switches(true, false),
            CrashAmplificationRunMode::TwoHostsCpu
        );
        assert_eq!(
            CrashAmplificationRunMode::from_switches(false, true),
            CrashAmplificationRunMode::SingleHostCpuWithGpu
        );
        assert_eq!(
            CrashAmplificationRunMode::from_switches(true, true),
            CrashAmplificationRunMode::TwoHostsCpuWithGpus
        );
        let directory =
            std::env::temp_dir().join(format!("singlefs-run-mode-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("建得了临时目录");
        let file = directory.join("multi-host.env");
        std::fs::write(&file, "ENABLE_ACROSS_MACHINES=1\nENABLE_GPU=\"0\"\n").expect("写得了");
        assert_eq!(
            CrashAmplificationRunMode::from_configuration_file(&file),
            CrashAmplificationRunMode::TwoHostsCpu
        );
        std::fs::remove_dir_all(&directory).expect("删得掉");
        assert_eq!(
            CrashAmplificationRunMode::from_configuration_file(&file),
            CrashAmplificationRunMode::SingleHostCpu,
            "文件不在按两个开关都关"
        );
    }

    /// 配置文件在就只看文件，不在才读同名的两个环境变量。环境走 K5 那一个守卫：两个开关由用例设、跑完（或 panic）还原。
    #[test]
    fn environment_switches_are_read_only_when_the_configuration_file_is_absent() {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[
            ("ENABLE_ACROSS_MACHINES", std::ffi::OsStr::new("1")),
            ("ENABLE_GPU", std::ffi::OsStr::new("\"1\"")),
        ]);
        let directory = std::env::temp_dir().join(format!(
            "singlefs-run-mode-environment-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("建得了临时目录");
        let file = directory.join("multi-host.env");
        std::fs::write(&file, "ENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\n").expect("写得了");
        assert_eq!(
            CrashAmplificationRunMode::from_configuration_file_or_environment(&file),
            CrashAmplificationRunMode::SingleHostCpu,
            "文件在就只看文件"
        );
        std::fs::remove_dir_all(&directory).expect("删得掉");
        assert_eq!(
            CrashAmplificationRunMode::from_configuration_file_or_environment(&file),
            CrashAmplificationRunMode::TwoHostsCpuWithGpus,
            "文件不在读同名环境变量"
        );
    }

    #[test]
    fn unit_writes_split_into_periods_by_the_unit_class_byte() {
        assert_eq!(period_of_unit_class(Some(1)), "data_unit");
        assert_eq!(period_of_unit_class(Some(2)), "index_node");
        assert_eq!(period_of_unit_class(Some(3)), "packed_records");
        assert_eq!(
            period_of_unit_class(Some(9)),
            "unit_write",
            "不认得的码不猜"
        );
        assert_eq!(
            period_of_unit_class(None),
            "unit_write",
            "短到没有第 6 字节"
        );
    }

    #[test]
    fn compile_time_file_paths_normalize_to_repository_relative_ones() {
        assert_eq!(
            repository_relative_file("crates/a/tests/../../b/tests/common/mod.rs"),
            "crates/b/tests/common/mod.rs",
            "`#[path = \"../../…\"]` 引进来的模块，file!() 带 .."
        );
        assert_eq!(repository_relative_file("./x/./y.rs"), "x/y.rs");
        assert_eq!(
            repository_relative_file("../outside.rs"),
            "../outside.rs",
            "越出仓根的原样留着，交给 NodeLabel::new 拒绝"
        );
    }

    #[test]
    fn relative_paths_from_the_environment_resolve_against_the_repository_root() {
        let root = Path::new("/repository/root");
        assert_eq!(
            path_relative_to_repository_root(root, "crash-amplification-library"),
            PathBuf::from("/repository/root/crash-amplification-library")
        );
        assert_eq!(
            path_relative_to_repository_root(root, "/tmp/x/library"),
            PathBuf::from("/tmp/x/library"),
            "绝对路径照用"
        );
    }

    /// 展开上限的变量写坏了（不是非负整数）：panic，消息带变量名与原值，不悄悄退回默认。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=\"2B\" 认不出")]
    fn an_unreadable_expand_up_to_value_panics_naming_the_variable_and_the_value() {
        let _ = ExpandUpToWritesPerSegment::from_value_of_the_variable(Some("2B"), 6);
    }

    /// 核对线程数写成 0：panic，消息带变量名与原值，不悄悄退回默认（与仓里别的线程数变量同一口径）。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_AMPLIFICATION_THREADS=\"0\" 认不出")]
    fn zero_checking_threads_panics_naming_the_variable_and_the_value() {
        let _ = checking_threads_from_the_variable_value(Some("0"), None);
    }

    /// 核对线程数写坏（不是数）：panic，消息带变量名与原值，不悄悄退回默认。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_AMPLIFICATION_THREADS=\"eight\" 认不出")]
    fn an_unreadable_checking_threads_value_panics_naming_the_variable_and_the_value() {
        let _ = checking_threads_from_the_variable_value(Some("eight"), None);
    }

    /// 核对线程数设了照它（两头的空白不算），没设读 `SINGLEFS_THREAD_CAP`，都没设取 `.claude/heavy-thread-default` 的登记值（不取本机核数）。
    #[test]
    fn checking_threads_are_the_value_written_or_the_heavy_thread_default_when_unset() {
        assert_eq!(
            checking_threads_from_the_variable_value(Some(" 3\n"), Some("5")),
            3
        );
        assert_eq!(checking_threads_from_the_variable_value(None, Some("5")), 5);
        assert_eq!(
            checking_threads_from_the_variable_value(None, None),
            singlefs_harness::heavy_threads::registered_heavy_threads().get()
        );
    }

    /// 没设取默认；设了照它（两头的空白不算）；按它判一段展不展开、整条流是不是全域。
    #[test]
    fn the_expand_up_to_value_is_the_default_when_unset_and_decides_each_segment() {
        assert_eq!(
            ExpandUpToWritesPerSegment::from_value_of_the_variable(None, 6),
            ExpandUpToWritesPerSegment(6)
        );
        let limit = ExpandUpToWritesPerSegment::from_value_of_the_variable(Some(" 28\n"), 6);
        assert_eq!(limit, ExpandUpToWritesPerSegment(28));
        assert_eq!(
            limit.expansion_of(&[0; 28]),
            CrashPointReplaySegmentExpansion::EveryProperSubset
        );
        assert_eq!(
            limit.expansion_of(&[0; 29]),
            CrashPointReplaySegmentExpansion::NotExpanded
        );
        assert!(limit.covers_every_segment(&[vec![0; 2], vec![0; 28]]));
        assert!(!limit.covers_every_segment(&[vec![0; 2], vec![0; 29]]));
    }

    #[test]
    fn blocks_of_a_point_cover_its_states_without_gap() {
        let point = PlannedCrashPoint {
            name: "p".to_string(),
            ignored: false,
            segments: Vec::new(),
            ordinals: 10..10 + STATES_PER_BLOCK * 2 + 5,
            reuse_key: StepAttributes {
                write_table_with_barriers: [0; 32],
                parent_output: [0; 32],
            }
            .reuse_key(),
            path_number: "a".to_string(),
            identity: "f::p::0".to_string(),
            judge_inputs: [0; 32],
            plan_hash: [0; 32],
            pieces_around_the_slices: Vec::new(),
        };
        let blocks = point.blocks();
        assert_eq!(
            blocks,
            vec![
                0..STATES_PER_BLOCK,
                STATES_PER_BLOCK..2 * STATES_PER_BLOCK,
                2 * STATES_PER_BLOCK..2 * STATES_PER_BLOCK + 5
            ]
        );
    }

    /// C611：一块判得比进度间隔久（假时钟每读一次走过一个间隔多一秒，那一段等到时钟读过两次才交来），等着收的那条线程打一行按挂钟的进度，
    /// 行里是这一块的点名、块起点、已收到几段（共几段）、这一趟判完几块与几个状态、开跑多久；那一段照样收进来。
    #[test]
    fn a_block_judged_for_longer_than_the_progress_interval_reports_a_wall_clock_line_naming_its_point(
    ) {
        use crate::crash::PROGRESS_LINE_INTERVAL_IN_SECONDS;
        use std::sync::atomic::{AtomicU64, Ordering};
        let one_interval_and_a_second =
            std::time::Duration::from_secs(PROGRESS_LINE_INTERVAL_IN_SECONDS + 1);
        let (piece_sender, piece_receiver) = std::sync::mpsc::channel::<JudgedBlock>();
        let (release_sender, release_receiver) = std::sync::mpsc::channel::<()>();
        let clock_reads = AtomicU64::new(0);
        // 第 0 次读是开跑那一刻；第 2 次读（已经过了一个间隔、按挂钟那一行到期）时才放那一段进来
        let clock = || {
            let reads = clock_reads.fetch_add(1, Ordering::SeqCst);
            if reads == 2 {
                // 放行的那一头只收一次，收过之后发不出去也无妨
                let _ = release_sender.send(());
            }
            one_interval_and_a_second * u32::try_from(reads).expect("一条用例里读不了几十亿次")
        };
        let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
        let mut printed: Vec<String> = Vec::new();
        let arrived = std::thread::scope(|scope| {
            scope.spawn(move || {
                release_receiver.recv().expect("时钟读到第 2 次就放行");
                piece_sender
                    .send(JudgedBlock {
                        local: 7..9,
                        items_of_each_state: vec![Vec::new(), Vec::new()],
                        recovery_words_of_each_state: Vec::new(),
                        state_counts: StateCountsOfABlock::default(),
                        red_texts: Vec::new(),
                    })
                    .expect("收的那一头还在等");
            });
            receive_the_pieces_of_one_block(
                &piece_receiver,
                &mut wall_clock_lines,
                &clock,
                &BlockInProgress {
                    point_name: "publish_first_file.journal_record.1",
                    block_start: 7,
                    piece_count: 1,
                    blocks_finished: 3,
                    pending_blocks: 5,
                    states_finished: 40,
                },
                &mut |line| printed.push(line),
            )
        });
        assert_eq!(arrived.len(), 1, "那一段照样收进来");
        assert_eq!(
            printed.first().map(String::as_str),
            Some(
                "CRASH_AMPLIFICATION_BLOCK by=wall_clock point=publish_first_file.journal_record.1 block_start=7 pieces_finished=0/1 blocks_finished=3/5 states_finished=40 elapsed_seconds=602.0"
            ),
            "过了一个间隔还没收到那一段，按挂钟打一行：{printed:?}"
        );
    }

    /// C611：GPU 判器那一处按挂钟的行：写进库几块（共几块）、几块在等前面的块、几张卡收工、开跑多久。
    #[test]
    fn the_gpu_judge_wall_clock_line_names_the_blocks_stored_and_waiting_and_the_cards_finished() {
        assert_eq!(
            gpu_judge_wall_clock_line(3, 10, 2, 1, std::time::Duration::from_secs(301)),
            "CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu blocks_stored=3/10 blocks_waiting_for_an_earlier_block=2 cards_finished=1 elapsed_seconds=301.0"
        );
    }

    /// C611 GPU 判器那一处：写库那条线程收卡上的话，过了一个进度间隔还没有新的话（块 1 先到、在等块 0；假时钟每读一次走过一个间隔多一秒，
    /// 那张卡收工的话等到时钟读过四次才发），打一行按挂钟的：写进库几块（共几块）、几块在等前面的块、几张卡收工、开跑多久；
    /// 之后收工的那张卡照样记下，没写成的块一块都不写。不要卡：卡上的线程换成一条发同样几句话的线程。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_gpu_judge_reports_a_wall_clock_line_while_it_waits_for_the_cards() {
        use crate::crash::PROGRESS_LINE_INTERVAL_IN_SECONDS;
        use std::cell::Cell;
        let one_interval_and_a_second =
            std::time::Duration::from_secs(PROGRESS_LINE_INTERVAL_IN_SECONDS + 1);
        let (card_sender, card_receiver) = std::sync::mpsc::channel::<CardMessage>();
        let (release_sender, release_receiver) = std::sync::mpsc::channel::<()>();
        card_sender
            .send(CardMessage::Block(GpuJudgedBlock {
                pending_index: 1,
                items_of_each_state: vec![Vec::new()],
                recovery_words_of_each_state: Vec::new(),
                state_counts: StateCountsOfABlock::default(),
                card_name: "测试造的卡".to_string(),
                card_driver_version: "测试造的驱动".to_string(),
            }))
            .expect("收的那一头还在");
        let clock_reads = Cell::new(0u32);
        // 第 0 次读是开跑那一刻、第 1 次读收到块 1、第 2 次读记下收到的时刻；第 4 次读（块 1 之后过了一个间隔、按挂钟那一行到期）时
        // 才放收工的那句话进来
        let clock = || {
            let reads = clock_reads.get();
            clock_reads.set(reads + 1);
            if reads == 4 {
                // 放行的那一头只收一次，收过之后发不出去也无妨
                let _ = release_sender.send(());
            }
            one_interval_and_a_second * reads
        };
        let mut wall_clock_lines = WallClockProgressLines::every_progress_line_interval(clock());
        let mut printed: Vec<String> = Vec::new();
        let mut blocks_handed_to_the_store = 0usize;
        let mut stored_in_order: BlocksStoredInOrder<GpuJudgedBlock> = BlocksStoredInOrder::new();
        let received = std::thread::scope(|scope| {
            scope.spawn(move || {
                release_receiver.recv().expect("时钟读到第 4 次就放行");
                let finished_card = GpuCardOutcome {
                    name: "stand-in card".to_string(),
                    index: 3,
                    memory_budget_mebibytes: 0,
                    states_per_dispatch: 0,
                    scratch_mebibytes: 0,
                    blocks_judged: 0,
                    states_judged: 0,
                    build_seconds: 0.0,
                    judge_seconds: 0.0,
                    dispatches: 0,
                    seconds_preparing: 0.0,
                    seconds_waiting_for_the_card: 0.0,
                    seconds_reading_back: 0.0,
                    seconds_decoding: 0.0,
                    states_in_the_widest_dispatch: 0,
                    seconds_of_the_longest_dispatch: 0.0,
                    scratch_booked_mebibytes: None,
                    scratch_lease_on_the_books_when_the_judge_was_built: None,
                    scratch_lease_on_the_books_when_the_judging_finished: None,
                };
                card_sender
                    .send(CardMessage::Finished(finished_card))
                    .expect("收的那一头还在等");
            });
            receive_the_cards_messages_and_store_the_blocks_in_order(
                &card_receiver,
                &mut wall_clock_lines,
                &clock,
                3,
                &mut stored_in_order,
                &mut |_block| {
                    blocks_handed_to_the_store += 1;
                    Ok(())
                },
                &mut |line| printed.push(line),
            )
        });
        assert_eq!(
            printed.first().map(String::as_str),
            Some(
                "CRASH_AMPLIFICATION_BLOCK by=wall_clock judge=gpu blocks_stored=0/3 blocks_waiting_for_an_earlier_block=1 cards_finished=0 elapsed_seconds=1204.0"
            ),
            "过了一个间隔还没收到新的话，按挂钟打一行：{printed:?}"
        );
        assert_eq!(
            received
                .finished_cards
                .iter()
                .map(|card| card.index)
                .collect::<Vec<u32>>(),
            vec![3],
            "收工的那张卡照样记下"
        );
        assert_eq!(
            (stored_in_order.blocks_stored(), blocks_handed_to_the_store),
            (0, 0),
            "块 1 在等块 0，一块都不写"
        );
        assert_eq!(stored_in_order.storing_error(), None);
        assert_eq!(received.refusal, None);
    }

    /// 第 10x 批代码轮判决 K1-3（攻方 R2）：卡线程上的错误范围接到 Out of Memory 就是显存不够（活成没成都一样）；没接到而活 panic 了，
    /// 设备上从活开始起记了 Out of Memory（另起的线程上编内核时没接住的）也是显存不够；都没有照样抛；活没 panic 交结果。
    /// 不要卡：几样输入照 [`on_the_card_catching_out_of_memory`] 收到的造。
    #[cfg(feature = "gpu")]
    #[test]
    fn a_panic_on_the_card_after_an_out_of_memory_on_another_thread_is_put_down_to_gpu_memory() {
        let panicked = || -> std::thread::Result<u32> {
            Err(Box::new("恢复内核的编译线程不 panic"))
        };
        assert!(
            matches!(
                decide_the_work_on_the_card(
                    panicked(),
                    None,
                    Some("另起的线程上没接住的 Out of Memory".to_string())
                ),
                WorkOnTheCard::OutOfMemory { ref detail } if detail == "另起的线程上没接住的 Out of Memory"
            ),
            "范围没接到、设备上记了：显存不够，不往外抛"
        );
        assert!(
            matches!(
                decide_the_work_on_the_card(panicked(), Some("Out of Memory".to_string()), None),
                WorkOnTheCard::OutOfMemory { .. }
            ),
            "范围接到了：显存不够"
        );
        assert!(
            matches!(
                decide_the_work_on_the_card(Ok(7u32), Some("Out of Memory".to_string()), None),
                WorkOnTheCard::OutOfMemory { .. }
            ),
            "活成了而范围接到 Out of Memory：缓冲没建成，判的不作数"
        );
        assert!(
            matches!(
                decide_the_work_on_the_card(panicked(), None, None),
                WorkOnTheCard::Panicked(_)
            ),
            "都没有：照样往外抛"
        );
        assert!(
            matches!(
                decide_the_work_on_the_card(Ok(7u32), None, None),
                WorkOnTheCard::Done(7)
            ),
            "活成了：交结果"
        );
    }

    /// 第 10x 批：卡上 wgpu 报了 Out of Memory（[`CardMessage::OutOfMemory`]，文件头第 5 条）记成显存不够、按卡记下，不记成拒判
    /// （拒判整趟判红，显存不够这一趟报本次未跑）；之前判完的块照写进库。不要卡：卡上的线程换成直接发这几句话。
    #[cfg(feature = "gpu")]
    #[test]
    fn an_out_of_memory_on_a_card_is_kept_apart_from_a_refusal_and_the_blocks_judged_before_it_are_stored(
    ) {
        let (card_sender, card_receiver) = std::sync::mpsc::channel::<CardMessage>();
        card_sender
            .send(CardMessage::Block(GpuJudgedBlock {
                pending_index: 0,
                items_of_each_state: vec![Vec::new()],
                recovery_words_of_each_state: Vec::new(),
                state_counts: StateCountsOfABlock::default(),
                card_name: "测试造的卡".to_string(),
                card_driver_version: "测试造的驱动".to_string(),
            }))
            .expect("收的那一头还在");
        for (index, reason) in [(0, "第一张卡报的那一句"), (1, "第二张卡报的那一句")]
        {
            card_sender
                .send(CardMessage::OutOfMemory {
                    index,
                    reason: reason.to_string(),
                })
                .expect("收的那一头还在");
        }
        drop(card_sender);
        let mut wall_clock_lines =
            WallClockProgressLines::every_progress_line_interval(std::time::Duration::ZERO);
        let mut blocks_handed_to_the_store = 0usize;
        let mut stored_in_order: BlocksStoredInOrder<GpuJudgedBlock> = BlocksStoredInOrder::new();
        let received = receive_the_cards_messages_and_store_the_blocks_in_order(
            &card_receiver,
            &mut wall_clock_lines,
            &|| std::time::Duration::ZERO,
            2,
            &mut stored_in_order,
            &mut |_block| {
                blocks_handed_to_the_store += 1;
                Ok(())
            },
            &mut |_line| {},
        );
        assert_eq!(
            received.cards_out_of_memory,
            vec![
                (0, "第一张卡报的那一句".to_string()),
                (1, "第二张卡报的那一句".to_string())
            ],
            "报了 Out of Memory 的卡按收到的次序都记下"
        );
        assert_eq!(
            received.refusal, None,
            "Out of Memory 不记成拒判：拒判整趟判红，显存不够报本次未跑"
        );
        assert_eq!(
            (stored_in_order.blocks_stored(), blocks_handed_to_the_store),
            (1, 1),
            "Out of Memory 之前判完的块 0 照写进库"
        );
    }

    /// K5：这份文件里每一处按名字读环境变量（`env::var("…")`、`env::var_os("…")`，测试模块之外），名字都在
    /// [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里：用例的环境守卫照这张表清、还原，漏一个就会读到调用方留下的值。
    #[test]
    fn every_environment_variable_this_file_reads_by_name_is_in_the_list_a_run_reads() {
        let names_read = environment_variables_read_by_name_outside_the_tests(include_str!(
            "crash_amplification.rs"
        ));
        // 多机配置与显卡先后改由 `crate::gpu_card_configuration` 读（崩溃点重放合并第三轮判决 J11），库目录改由 `crate::crash_library` 读，这一份按名字读的今天是七处
        assert!(
            names_read.len() >= 7,
            "按名字读的环境变量少得不对，扫法坏了：{names_read:?}"
        );
        assert!(
            names_read
                .iter()
                .any(|name| name == SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE),
            "按段登记表的路径按名字读：{names_read:?}"
        );
        for name in &names_read {
            assert!(
                ENVIRONMENT_VARIABLES_READ_BY_A_RUN.contains(&name.as_str()),
                "{name} 在这份文件里按名字读，却不在 ENVIRONMENT_VARIABLES_READ_BY_A_RUN 里"
            );
        }
    }

    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块之外按名字读的环境变量（`env::var("…")`、`env::var_os("…")`），照源码次序。
    fn environment_variables_read_by_name_outside_the_tests(source: &str) -> Vec<String> {
        let outside_the_tests = text_outside_the_tests_module(source);
        let mut names_read = Vec::new();
        for opening in ["env::var(\"", "env::var_os(\""] {
            for (position, _) in outside_the_tests.match_indices(opening) {
                let rest = &outside_the_tests[position + opening.len()..];
                let name = &rest[..rest.find('"').expect("变量名的串收得了尾")];
                names_read.push(name.to_string());
            }
        }
        names_read
    }

    /// `source`（这份源文件，或用例在它上面改过的一份）去掉测试模块之后的文本：从测试模块的头两行起，到与它配对的顶格右括号那一行止，
    /// 连同这两处一起去掉。头两行只认这一种，与卡池扫源码那一份（`gpu_unit_checks.rs` 的 `code_before_the_tests_module`）同一口径、
    /// 按行逐字：
    /// - 相邻两整行逐字是 [`TESTS_MODULE_OPENING`] 那两行（`#[cfg(test)]`，与 `mod tests`、一个空格、一个左花括号；行首起、行尾止，不带别的空白与注释，换行是 `\n`），在代码里（注释与字面量之外）；
    /// - 在花括号的第 0 层，也在圆括号、方括号的第 0 层；
    /// - 前一个词（空白、注释、字面量都跳过）是分号、右花括号、右方括号或文件开头。
    ///
    /// 照原文找第一处时，块注释里写一份就把切口挪到注释那里，注释与真测试模块之间的代码被当成测试切掉（第 10z 批第二十三块）；
    /// 拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进一次宏调用的圆括号里，那个「测试模块」在非测试构建里照编、里面藏的层照样可调用，
    /// 头两行却仍在花括号第 0 层（第 10z 批第二十五块，正推 K4-1 S14），宏调用的括号里头两行前一个词是 `(`，前面先放一条以分号收尾的
    /// 条目也还在圆括号里。这一份原先每行去掉行尾空白再比，第二十五块对齐成逐字：这两份文件是 rustfmt 出的，头两行就是这一种写法，
    /// 别的写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释、CRLF 换行）一律判红，是故意拒。
    ///
    /// # Panics
    /// 照上面那样认的头两行不是恰好一处，或它之后没有收尾的顶格右括号行。
    fn text_outside_the_tests_module(source: &str) -> String {
        let lines: Vec<&str> = source.split('\n').collect();
        let read_lines = lines_read_past_comments_and_literals(&lines);
        let (attribute_line, module_line) = TESTS_MODULE_OPENING
            .split_once('\n')
            .expect("测试模块的头两行是两行");
        let code_before_the_line = |line_index: usize| -> String {
            read_lines[..line_index]
                .iter()
                .map(|read_line| read_line.without_comments_and_literals.as_str())
                .collect::<Vec<&str>>()
                .join("\n")
        };
        let openings: Vec<usize> = read_lines
            .windows(2)
            .enumerate()
            .filter(|(line_index, pair)| {
                pair[0].without_comments_and_literals == attribute_line
                    && pair[1].without_comments_and_literals == module_line
                    && pair[0].brace_depth_at_the_start == 0
                    && parenthesis_and_bracket_depth_at_the_end_of(&code_before_the_line(
                        *line_index,
                    )) == 0
                    && {
                        let code_before_the_attribute_line = code_before_the_line(*line_index);
                        let last_code = code_before_the_attribute_line.trim_end();
                        last_code.is_empty() || last_code.ends_with([';', '\u{7d}', ']'])
                    }
            })
            .map(|(line_index, _pair)| line_index)
            .collect();
        assert_eq!(
            openings.len(),
            1,
            "代码里测试模块的头两行恰好一处（两整行逐字、括号第 0 层、前一个词是条目收尾；注释、字符串、宏调用括号里的不算），读到的行号（从 0 数）：{openings:?}"
        );
        let opening = openings[0];
        let closing = opening
            + 1
            + read_lines[opening + 1..]
                .iter()
                .position(|read_line| {
                    read_line.brace_depth_at_the_start == 1
                        && read_line.without_comments_and_literals.trim_end()
                            == TOP_LEVEL_CLOSING_BRACE_LINE.trim()
                })
                .expect("测试模块在顶格的右括号行收尾");
        lines[..opening]
            .iter()
            .chain(&lines[closing + 1..])
            .copied()
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// `code`（注释与字面量已换成空格）读到结尾时还没收尾的圆括号与方括号有几层。
    ///
    /// # Panics
    /// 右圆括号、右方括号多于左的。
    fn parenthesis_and_bracket_depth_at_the_end_of(code: &str) -> usize {
        code.chars().fold(0_usize, |depth, character| {
            if matches!(character, '(' | '[') {
                depth + 1
            } else if matches!(character, ')' | ']') {
                depth.checked_sub(1).expect("右圆括号、右方括号不多于左的")
            } else {
                depth
            }
        })
    }

    /// 这份源文件测试模块之外的代码行（注释行不算）。
    fn code_lines_outside_the_tests() -> Vec<String> {
        text_outside_the_tests_module(include_str!("crash_amplification.rs"))
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .map(str::to_string)
            .collect()
    }

    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K2-2，攻方 F2）：叫停文件只在一处读进这一趟记住的那一个叫停
    /// （`StopRequestRememberedInThisRun::new`），外层循环、卡线程、一轮收尾与整趟收尾不再各自现读文件；叫停文件的路径只有
    /// `stop_file_of_the_environment` 一处读环境变量，给定状态表的引擎起步时经它读一次路径、记进它自己那一个叫停。扫这份源码，不要卡。
    #[test]
    fn the_run_reads_the_stop_file_only_into_the_stop_request_it_remembers() {
        let lines_naming = |name: &str| -> Vec<String> {
            code_lines_outside_the_tests()
                .into_iter()
                .filter(|line| line.contains(name))
                .map(|line| line.trim().to_string())
                .collect()
        };
        assert_eq!(
            lines_naming(concat!("stop_is_requested_", "by_the_environment")),
            vec![
                concat!(
                    "pub fn stop_is_requested_",
                    "by_the_environment() -> bool \u{7b}"
                )
                .to_string(),
                concat!(
                    "let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_",
                    "by_the_environment);"
                )
                .to_string(),
            ],
            "读叫停文件在不在的只有定义它的那一处与流水线这一趟记住叫停的那一处"
        );
        assert_eq!(
            lines_naming(concat!(
                "var_os(\"SINGLEFS_CRASH_AMPLIFICATION_",
                "STOP_FILE\")"
            )),
            vec![concat!(
                "std::env::var_os(\"SINGLEFS_CRASH_AMPLIFICATION_",
                "STOP_FILE\").map(PathBuf::from)"
            )
            .to_string()],
            "读叫停文件路径这个环境变量的只有 stop_file_of_the_environment 那一处"
        );
        assert_eq!(
            lines_naming(concat!("stop_file_of_", "the_environment()")),
            vec![
                concat!(
                    "stop_file_of_",
                    "the_environment().is_some_and(|path| path.exists())"
                )
                .to_string(),
                concat!("fn stop_file_of_", "the_environment() -> Option<PathBuf> \u{7b}").to_string(),
                // 给定状态表的引擎（GivenStatesEngine）是另一趟：起的时候读一次路径、记住它自己那一个叫停，崩溃注入领片之前问的是这一个
                concat!("let stop_file = stop_file_of_", "the_environment();").to_string(),
            ],
            "读叫停文件路径的只有流水线那一侧现读在不在的那一处与给定状态表的引擎起步时记一次的那一处"
        );
    }

    /// 第 10z 批第六块（第 10z 批第五块代码轮判决 K1-3）：第 ① 段在卡上判、照叫停收尾的那一支打头一句 Out of Memory 那一行
    /// （`RoundsOfTheCards::not_run_line_of_a_stop_after_an_out_of_memory`，harness 档钉它的形态）。扫这份源码，不要卡。
    #[test]
    fn the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line() {
        let lines = code_lines_outside_the_tests();
        let arm = lines
            .iter()
            .position(|line| line.trim() == "AfterARoundOfTheCards::StoppedOnRequest => \u{7b}")
            .expect("第 ① 段几轮卡的收尾里有照叫停收尾那一支");
        let arm_body = lines[arm..]
            .iter()
            .take_while(|line| line.trim() != "break;")
            .cloned()
            .collect::<Vec<String>>()
            .join("\n");
        assert!(
            arm_body.contains(".not_run_line_of_a_stop_after_an_out_of_memory(")
                && arm_body.contains("PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE")
                && arm_body
                    .lines()
                    .any(|line| line.trim() == "println!(\"{not_run_line}\");"),
            "照叫停收尾那一支要把头一句 Out of Memory 那一行打进标准输出（整行比，`eprintln!` 不算）：{arm_body}"
        );
    }

    /// 第 10z 批第八块（第 10z 批第六块代码轮判决 K4，正推第 3 点：checker 档接线五种改回去，两条扫源码用例照绿）：这一趟记住的那一个叫停
    /// 一路交到一轮收尾（A）、起下一张卡（B）、卡线程判块那一圈（C）与整趟收尾（D）；E（照叫停收尾那一支打进标准输出）在上一条。扫这份源码，不要卡。
    #[test]
    fn the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run(
    ) {
        let lines: Vec<String> = code_lines_outside_the_tests()
            .iter()
            .map(|line| line.trim().to_string())
            .collect();
        let count = |expected: &str| {
            lines
                .iter()
                .filter(|line| line.as_str() == expected)
                .count()
        };
        for (expected, what) in [
            (
                concat!("let stop = stop_request.", "stop_of_the_run();"),
                "一轮收尾问这一趟记住的叫停（A）",
            ),
            (
                concat!("&|| work.", "stop_of_the_cards(),"),
                "卡线程判块那一圈问卡上的叫停（C）",
            ),
            (
                concat!(
                    "let stop_is_requested = || stop_",
                    "request.is_requested();"
                ),
                "整趟收尾问的是记住的那一个（D）",
            ),
            (
                concat!(
                    "let stop_was_requested = gpu_memory_was_short || stop_",
                    "is_requested();"
                ),
                "整趟收尾照它判提早收尾（D）",
            ),
        ] {
            assert_eq!(count(expected), 1, "{what}：要有一行 {expected}");
        }
        let start = lines
            .iter()
            .position(|line| line.starts_with("fn start_the_next_card_with_states<"))
            .expect("有起下一张卡的那一个函数");
        let next_card_body: Vec<&String> = lines[start..]
            .iter()
            .take_while(|line| line.as_str() != "\u{7d}")
            .collect();
        assert!(
            next_card_body
                .iter()
                .any(|line| line.as_str() == concat!("work.", "stop_of_the_cards()")),
            "起下一张卡照卡上的叫停问（B）：{next_card_body:?}"
        );
    }

    /// 第 10z 批第十一块（第 10z 批第七八九块代码轮判决 K5，正推六·二：交给第 ① 段在卡上判与 CPU 判器的叫停那一行换成永不叫停，
    /// 上一条照绿）：这一趟记住的那一个叫停（`stop_is_requested`）就是交给 `judge_my_blocks_on_gpu` 与 `check_selected_blocks` 的那一个
    /// （两处调用的最后一个参数整行比）。下游各层把收到的叫停原样往下交，换掉就是那一层的参数没用上、编译器报没用的变量。扫这份源码，不要卡。
    #[test]
    fn the_stop_request_remembered_in_this_run_is_the_one_handed_to_the_gpu_judge_and_the_cpu_checker(
    ) {
        let lines: Vec<String> = code_lines_outside_the_tests()
            .iter()
            .map(|line| line.trim().to_string())
            .collect();
        for (call_opening, what) in [
            (
                concat!("let gpu_judging = judge_my_blocks_on_gpu", "("),
                "第 ① 段在卡上判",
            ),
            (
                concat!("let checking = check_selected_blocks", "("),
                "第 ① 段 CPU 判器",
            ),
        ] {
            let openings: Vec<usize> = lines
                .iter()
                .enumerate()
                .filter(|(_line_index, line)| line.as_str() == call_opening)
                .map(|(line_index, _line)| line_index)
                .collect();
            assert_eq!(openings.len(), 1, "{what}：这份源码里有一处 {call_opening}");
            let arguments: Vec<&String> = lines[openings[0] + 1..]
                .iter()
                .take_while(|line| !line.starts_with(')'))
                .collect();
            assert_eq!(
                arguments.last().map(|argument| argument.as_str()),
                Some(concat!("&stop_is_", "requested,")),
                "{what}收到的叫停是这一趟记住的那一个：{arguments:?}"
            );
        }
    }

    /// `line` 里 `stop_is_requested` 作为整个名字出现了几次（前后不连着字母、数字、下划线：`stop_is_requested_by_the_environment`、
    /// `_stop_is_requested` 不算）。
    fn whole_name_uses_of_the_stop_request(line: &str) -> usize {
        whole_word_positions(line, STOP_REQUEST_PARAMETER_NAME).len()
    }

    /// 收叫停的那几层给叫停参数起的名字。
    const STOP_REQUEST_PARAMETER_NAME: &str = concat!("stop_is_", "requested");

    /// `text` 里 `word` 作为整个名字出现的每一处起点（前后不连着字母、数字、下划线）。
    fn whole_word_positions(text: &str, word: &str) -> Vec<usize> {
        let is_part_of_a_name = |neighbour: Option<char>| {
            neighbour.is_some_and(|neighbour_character| {
                neighbour_character.is_alphanumeric() || neighbour_character == '_'
            })
        };
        text.match_indices(word)
            .filter(|(position, _word)| {
                !is_part_of_a_name(text[..*position].chars().next_back())
                    && !is_part_of_a_name(text[position + word.len()..].chars().next())
            })
            .map(|(position, _word)| position)
            .collect()
    }

    /// 叫停参数的类型，去掉空白。
    const STOP_REQUEST_TYPE_WITHOUT_SPACES: &str = "&(dynFn()->bool+Sync)";

    /// `text` 跳过空白读下去恰以 `pattern_without_spaces` 起头时，那一截（连同中间的空白）收尾之后的位置；不是就交 `None`。
    fn end_of_a_prefix_ignoring_whitespace(
        text: &str,
        pattern_without_spaces: &str,
    ) -> Option<usize> {
        let mut pattern_characters = pattern_without_spaces.chars().peekable();
        for (offset, character) in text.char_indices() {
            let Some(expected_character) = pattern_characters.peek() else {
                return Some(offset);
            };
            if character.is_whitespace() {
                continue;
            }
            if character != *expected_character {
                return None;
            }
            pattern_characters.next();
        }
        pattern_characters.peek().is_none().then_some(text.len())
    }

    /// 源码里写着叫停类型的一处（参数、`let` 的类型标注、字段都算）。
    struct PlaceTakingTheStopRequest<'code> {
        /// 往前最近的 `fn` 后面那个名字（往前没有 `fn` 的交空串）。
        layer: &'code str,
        /// 类型前面 `名字:` 的那个名字（前面不是 `名字:` 的交空串）。
        parameter: &'code str,
        /// 参数名前面紧挨着的是不是属性的右方括号（`#[cfg(any())] stop_is_requested: …`）。
        is_the_parameter_under_an_attribute: bool,
        /// 类型起头（`&`）的位置。
        start_of_the_type: usize,
        /// 类型收尾之后的位置。
        end_of_the_type: usize,
    }

    /// `code`（注释与字面量已换成空格）里每一处叫停类型，照源码次序：类型里的空白随意、可以跨行，参数与别的参数挤在签名那一行、
    /// 参数换了名字的都找得到。
    fn places_taking_the_stop_request(code: &str) -> Vec<PlaceTakingTheStopRequest<'_>> {
        let is_identifier_character =
            |character: char| character.is_alphanumeric() || character == '_';
        let identifier_at_the_end_of = |text: &str| -> usize {
            text.char_indices()
                .rev()
                .take_while(|(_index, character)| is_identifier_character(*character))
                .last()
                .map_or(text.len(), |(index, _character)| index)
        };
        code.match_indices('&')
            .filter_map(|(ampersand, _ampersand)| {
                let end_of_the_type = ampersand
                    + end_of_a_prefix_ignoring_whitespace(
                        &code[ampersand..],
                        STOP_REQUEST_TYPE_WITHOUT_SPACES,
                    )?;
                let before_the_type = code[..ampersand].trim_end();
                let before_the_colon = before_the_type
                    .strip_suffix(':')
                    .filter(|before_the_colon| !before_the_colon.ends_with(':'))
                    .map(str::trim_end);
                let parameter = before_the_colon.map_or("", |before_the_name| {
                    &before_the_name[identifier_at_the_end_of(before_the_name)..]
                });
                let is_the_parameter_under_an_attribute =
                    before_the_colon.is_some_and(|before_the_name| {
                        before_the_name[..identifier_at_the_end_of(before_the_name)]
                            .trim_end()
                            .ends_with(']')
                    });
                let layer = whole_word_positions(&code[..ampersand], "fn")
                    .last()
                    .map_or("", |fn_keyword| {
                        let after_the_keyword = code[fn_keyword + 2..ampersand].trim_start();
                        let name_length = after_the_keyword
                            .find(|character: char| !is_identifier_character(character))
                            .unwrap_or(after_the_keyword.len());
                        &after_the_keyword[..name_length]
                    });
                Some(PlaceTakingTheStopRequest {
                    layer,
                    parameter,
                    is_the_parameter_under_an_attribute,
                    start_of_the_type: ampersand,
                    end_of_the_type,
                })
            })
            .collect()
    }

    /// 叫停那种闭包的形（不问参数）到返回类型之前那一截，去掉空白；`FnMut`、`FnOnce` 与函数指针 `fn() -> …` 也算。返回类型交 `bool`
    /// 的才算叫停（[`end_of_a_bool_return_type`]）。
    const STOP_REQUEST_SHAPES_UP_TO_THE_RETURN_TYPE_WITHOUT_SPACES: [&str; 4] =
        ["Fn()->", "FnMut()->", "FnOnce()->", "fn()->"];

    /// 叫停那种闭包交的 `bool` 的写法，去掉空白：按路径写到 `core`、`std` 里那个原始类型的也算（第 10z 批第二十五块，正推 K4-1 N01）。
    const WAYS_TO_WRITE_BOOL_WITHOUT_SPACES: [&str; 5] = [
        "bool",
        "::core::primitive::bool",
        "core::primitive::bool",
        "::std::primitive::bool",
        "std::primitive::bool",
    ];

    /// `text`（叫停那种闭包的形 `->` 之后那一截）以交 `bool` 的返回类型起头时，那个 `bool` 收尾之后的位置；不是就交 `None`。
    /// 空白随意，外面套几层括号的（`(bool)`，第 10z 批第二十五块，正推 K4-1 N02）、按路径写的（[`WAYS_TO_WRITE_BOOL_WITHOUT_SPACES`]）都算，
    /// 后面紧跟标识符的字（`boolean`）不算。
    fn end_of_a_bool_return_type(text: &str) -> Option<usize> {
        let is_identifier_character =
            |character: char| character.is_alphanumeric() || character == '_';
        let after_the_parentheses = text
            .trim_start_matches(|character: char| character == '(' || character.is_whitespace());
        let parentheses_length = text.len() - after_the_parentheses.len();
        WAYS_TO_WRITE_BOOL_WITHOUT_SPACES
            .iter()
            .find_map(|way_to_write_bool| {
                let bool_length =
                    end_of_a_prefix_ignoring_whitespace(after_the_parentheses, way_to_write_bool)?;
                (!after_the_parentheses[bool_length..]
                    .chars()
                    .next()
                    .is_some_and(is_identifier_character))
                .then_some(parentheses_length + bool_length)
            })
    }

    /// `code`（注释与字面量已换成空格）里写着叫停那种闭包的形、却不在 `places` 哪一处钉住的类型里的地方，一处一句（往前最近的 `fn`
    /// 后面那个名字连同那一行）：叫停写成 `&(dyn Fn() -> bool + Sync)` 之外的类型（多加 `Send`、`'static`，`&mut`、`&'static`，
    /// `Box<dyn …>`，`dyn Sync + Fn() -> bool`，`&dyn Fn() -> bool`，`impl`、泛型、这份源文件里的类型别名）收叫停的层，按钉住的类型找不到，
    /// 照这个形找得到。
    fn stop_request_shapes_outside_the_pinned_type(
        code: &str,
        places: &[PlaceTakingTheStopRequest<'_>],
    ) -> Vec<String> {
        let is_identifier_character =
            |character: char| character.is_alphanumeric() || character == '_';
        let mut found = Vec::new();
        for shape in STOP_REQUEST_SHAPES_UP_TO_THE_RETURN_TYPE_WITHOUT_SPACES {
            let first_word = &shape[..shape.find('(').expect("形里有左括号")];
            for position in whole_word_positions(code, first_word) {
                let Some(shape_length) =
                    end_of_a_prefix_ignoring_whitespace(&code[position..], shape)
                else {
                    continue;
                };
                if end_of_a_bool_return_type(&code[position + shape_length..]).is_none() {
                    continue;
                }
                if places.iter().any(|place| {
                    (place.start_of_the_type..place.end_of_the_type).contains(&position)
                }) {
                    continue;
                }
                let layer =
                    whole_word_positions(&code[..position], "fn")
                        .last()
                        .map_or("", |fn_keyword| {
                            let after_the_keyword = code[fn_keyword + 2..position].trim_start();
                            let name_length = after_the_keyword
                                .find(|character: char| !is_identifier_character(character))
                                .unwrap_or(after_the_keyword.len());
                            &after_the_keyword[..name_length]
                        });
                let line_start = code[..position]
                    .rfind('\n')
                    .map_or(0, |newline| newline + 1);
                let line_end = code[position..]
                    .find('\n')
                    .map_or(code.len(), |newline| position + newline);
                found.push(format!(
                    "{layer} 往后有一处叫停写成钉住的 &(dyn Fn() -> bool + Sync) 之外的类型，按钉住的类型找不到这一层：{}",
                    code[line_start..line_end].trim()
                ));
            }
        }
        found
    }

    /// 没有行为用例的那几层，扫源码钉住它们怎么用收到的叫停（`every_layer_handed_the_stop_request_hands_it_on_or_asks_it`）。
    enum StopRequestUseThatIsPinned {
        /// 函数体短：参数行之后到函数收尾（返回类型与函数体，去掉注释行与空白）整个等于这一句。
        WholeBodyAfterTheParameter(&'static str),
        /// 函数体太长、整句比不了：用到叫停整名的只有 `line` 这一行（去掉首尾空白），它是函数体顶层的一句；
        /// `binding` 在这一层只由它 `let` 绑一次，`type_name` 在这一层只出现在它里面。
        TopLevelStatementOfALongBody {
            line: &'static str,
            binding: &'static str,
            type_name: &'static str,
        },
    }

    /// 数花括号时，读到的这个字在源码的哪一种地方（跨行时带到下一行）。
    #[derive(Clone, Copy)]
    enum PlaceInTheSource {
        Code,
        InsideAString,
        /// 原始字符串（`r"…"`、`r#"…"#`、`br"…"`、`cr"…"`）：收尾是一个 `"` 后面跟这么多个 `#`。
        InsideARawString {
            closing_hashes: usize,
        },
        /// 块注释，可以嵌套：`nesting` 是还没收尾的 `/*` 有几层。
        InsideABlockComment {
            nesting: usize,
        },
    }

    /// 读过的一个字算源码里的哪一类（`lines_read_past_comments_and_literals`）。
    #[derive(Clone, Copy)]
    enum KindOfACharacterInTheSource {
        Code,
        /// 行注释、块注释里的字，连同 `//`、`/*`、`*/`。
        Comment,
        /// 字符串（普通的、原始的）与字符字面量里的字，连同引号与原始字符串的 `r`、`br`、`cr` 和 `#`。
        Literal,
    }

    /// 一行读过注释与字面量之后的样子（`lines_read_past_comments_and_literals`）。
    struct LineReadPastCommentsAndLiterals {
        /// 这一行起头处在第几层花括号里。
        brace_depth_at_the_start: usize,
        /// 注释里的字换成空格（一个字换成与它的 UTF-8 字节数一样多的空格，字节位置不变），字面量照原样。
        without_comments: String,
        /// 注释与字面量里的字都换成空格，字节位置不变：扫源码的用例照它找名字、数 `let`、数花括号，注释与字符串里写的不算。
        without_comments_and_literals: String,
    }

    /// `lines` 每一行起头处在第几层花括号里（第一行起头算第 0 层）：字符串（普通的、原始的）、字符字面量、行注释与块注释（可以嵌套）里的
    /// 花括号不算，字符串、块注释跨行时照跨着数。
    ///
    /// # Panics
    /// 右花括号多于左花括号。
    fn brace_depth_at_the_start_of_each_line(lines: &[&str]) -> Vec<usize> {
        lines_read_past_comments_and_literals(lines)
            .iter()
            .map(|read_line| read_line.brace_depth_at_the_start)
            .collect()
    }

    /// `lines` 逐行读过注释与字面量：每一行起头处的花括号层数，连同去掉注释、去掉注释与字面量的两份（字节位置与原行相同）。
    /// 字符串（普通的、原始的 `r"…"`、`r#"…"#`、`br`、`cr`）、字符字面量、行注释与块注释（可以嵌套）里的花括号不算，
    /// 字符串、块注释跨行时照跨着读。与卡池扫源码那一份分词（`gpu_unit_checks.rs` 的 `rust_tokens_without_comments_and_literals`）
    /// 认原始字符串的写法相同：`r`、`br`、`cr` 后面紧跟若干个 `#` 与一个 `"`。
    ///
    /// # Panics
    /// 右花括号多于左花括号。
    fn lines_read_past_comments_and_literals(
        lines: &[&str],
    ) -> Vec<LineReadPastCommentsAndLiterals> {
        let is_identifier_character =
            |character: char| character.is_alphanumeric() || character == '_';
        let mut depth = 0_usize;
        let mut place = PlaceInTheSource::Code;
        let mut read_lines = Vec::with_capacity(lines.len());
        for line in lines {
            let brace_depth_at_the_start = depth;
            let characters: Vec<char> = line.chars().collect();
            let mut kinds = vec![KindOfACharacterInTheSource::Code; characters.len()];
            let mut index = 0;
            while index < characters.len() {
                let character = characters[index];
                let next_character = characters.get(index + 1).copied();
                let read_from = index;
                let place_before = place;
                match place {
                    PlaceInTheSource::InsideAString => {
                        if character == '\\' {
                            index += 1;
                        } else if character == '"' {
                            place = PlaceInTheSource::Code;
                        }
                    }
                    PlaceInTheSource::InsideARawString { closing_hashes } => {
                        let closing_hashes_follow = characters
                            .get(index + 1..index + 1 + closing_hashes)
                            .is_some_and(|after_the_quote| {
                                after_the_quote.iter().all(|after| *after == '#')
                            });
                        if character == '"' && closing_hashes_follow {
                            index += closing_hashes;
                            place = PlaceInTheSource::Code;
                        }
                    }
                    PlaceInTheSource::InsideABlockComment { nesting } => {
                        if character == '/' && next_character == Some('*') {
                            index += 1;
                            place = PlaceInTheSource::InsideABlockComment {
                                nesting: nesting + 1,
                            };
                        } else if character == '*' && next_character == Some('/') {
                            index += 1;
                            place = match nesting.checked_sub(1) {
                                Some(0) | None => PlaceInTheSource::Code,
                                Some(nesting_left) => PlaceInTheSource::InsideABlockComment {
                                    nesting: nesting_left,
                                },
                            };
                        }
                    }
                    PlaceInTheSource::Code => {
                        if character == '"' {
                            place = PlaceInTheSource::InsideAString;
                        } else if character == '\'' {
                            // 字符字面量（`'x'`、`'\''`、`'\u{7b}'`）整个跳过；生命周期（`'static`）只跳这个引号
                            if next_character == Some('\\') {
                                index += 2;
                                while characters.get(index + 1).is_some_and(|next| *next != '\'') {
                                    index += 1;
                                }
                                index += 1;
                            } else if characters.get(index + 2) == Some(&'\'') {
                                index += 2;
                            }
                        } else if character == '/' && next_character == Some('/') {
                            kinds[index..].fill(KindOfACharacterInTheSource::Comment);
                            break;
                        } else if character == '/' && next_character == Some('*') {
                            index += 1;
                            place = PlaceInTheSource::InsideABlockComment { nesting: 1 };
                        } else if is_identifier_character(character) {
                            // 一整个词跳过；词是 r、br、cr 而后面紧跟若干个 # 与一个 " 的，是原始字符串的开头
                            let word_end = characters[index..]
                                .iter()
                                .position(|after| !is_identifier_character(*after))
                                .map_or(characters.len(), |word_length| index + word_length);
                            let word: String = characters[index..word_end].iter().collect();
                            let opening_hashes = characters[word_end..]
                                .iter()
                                .take_while(|after| **after == '#')
                                .count();
                            if matches!(word.as_str(), "r" | "br" | "cr")
                                && characters.get(word_end + opening_hashes) == Some(&'"')
                            {
                                index = word_end + opening_hashes;
                                place = PlaceInTheSource::InsideARawString {
                                    closing_hashes: opening_hashes,
                                };
                            } else {
                                index = word_end - 1;
                            }
                        } else if character == '\u{7b}' {
                            depth += 1;
                        } else if character == '\u{7d}' {
                            depth = depth.checked_sub(1).expect("右花括号不多于左花括号");
                        }
                    }
                }
                index += 1;
                let kind_read = match place_before {
                    PlaceInTheSource::InsideAString | PlaceInTheSource::InsideARawString { .. } => {
                        KindOfACharacterInTheSource::Literal
                    }
                    PlaceInTheSource::InsideABlockComment { .. } => {
                        KindOfACharacterInTheSource::Comment
                    }
                    PlaceInTheSource::Code => match place {
                        PlaceInTheSource::InsideAString
                        | PlaceInTheSource::InsideARawString { .. } => {
                            KindOfACharacterInTheSource::Literal
                        }
                        PlaceInTheSource::InsideABlockComment { .. } => {
                            KindOfACharacterInTheSource::Comment
                        }
                        // 一次读过不止一个字的引号是整个字符字面量；生命周期只读过那一个引号
                        PlaceInTheSource::Code if character == '\'' && index > read_from + 1 => {
                            KindOfACharacterInTheSource::Literal
                        }
                        PlaceInTheSource::Code => KindOfACharacterInTheSource::Code,
                    },
                };
                kinds[read_from..index.min(characters.len())].fill(kind_read);
            }
            let mut without_comments = String::with_capacity(line.len());
            let mut without_comments_and_literals = String::with_capacity(line.len());
            for (character, kind) in characters.iter().zip(&kinds) {
                let spaces = " ".repeat(character.len_utf8());
                match kind {
                    KindOfACharacterInTheSource::Code => {
                        without_comments.push(*character);
                        without_comments_and_literals.push(*character);
                    }
                    KindOfACharacterInTheSource::Comment => {
                        without_comments.push_str(&spaces);
                        without_comments_and_literals.push_str(&spaces);
                    }
                    KindOfACharacterInTheSource::Literal => {
                        without_comments.push(*character);
                        without_comments_and_literals.push_str(&spaces);
                    }
                }
            }
            read_lines.push(LineReadPastCommentsAndLiterals {
                brace_depth_at_the_start,
                without_comments,
                without_comments_and_literals,
            });
        }
        read_lines
    }

    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4：扫源码的用例被领块之前那一问写成 `if false && stop_is_requested()` 骗过，
    /// 攻方 crash-3）：四块要判，叫停在第三次被问时交真：领每一块之前各问一次、问到真就不再领，叫停之前领的两块判完写了，之后的不领，
    /// 叫停一共被问三次。不要卡、不碰库：判一段与写一块都用假的。
    #[test]
    fn the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken()
    {
        const ASK_THAT_FINDS_THE_STOP: usize = 3;
        let pending: Vec<(usize, Range<u64>)> = vec![(0, 0..2), (0, 2..4), (0, 4..6), (0, 6..8)];
        let points = vec![NamesOfACrashPoint {
            name: "一个点".to_string(),
            path_number: "0".to_string(),
        }];
        let asks = std::sync::atomic::AtomicUsize::new(0);
        let mut stored_block_starts = Vec::new();
        let tally = check_pending_blocks_in_pieces(
            &pending,
            &points,
            2,
            &|| {
                asks.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
                    >= ASK_THAT_FINDS_THE_STOP
            },
            &|_point_index, local| JudgedBlock {
                items_of_each_state: vec![
                    Vec::new();
                    usize::try_from(local.end - local.start)
                        .expect("一段装得进")
                ],
                local,
                recovery_words_of_each_state: Vec::new(),
                state_counts: StateCountsOfABlock::default(),
                red_texts: Vec::new(),
            },
            &mut |_point_index, local, _checked| {
                stored_block_starts.push(local.start);
                Ok(())
            },
        )
        .expect("假的写库不失败");
        assert_eq!(
            asks.load(std::sync::atomic::Ordering::SeqCst),
            ASK_THAT_FINDS_THE_STOP,
            "领第一、二块之前各问一次（不停），领第三块之前问到停、不再问"
        );
        assert_eq!(
            stored_block_starts,
            vec![0, 2],
            "叫停之前领的两块判完写了，之后的不领"
        );
        assert_eq!(tally.blocks_checked, 2);
    }

    /// 第 10z 批第十三块（第 10z 批第十、十一块代码轮判决 K5，正推：下游各层把收到的叫停换成永不叫停，`cargo check` 只报没用的变量、
    /// 变异跑道不带 `-D warnings`，照绿）：这份源码里收叫停参数（`stop_is_requested: &(dyn Fn() -> bool + Sync),`，测试那一段之外）的函数
    /// 照源码次序是这六层（多一层少一层都要回来看罩没罩到）。
    /// 第 10z 批第十五块（第 10z 批第十二、十三块代码轮判决 K4：「参数行之后整名用上至少一次」被只提一次名字而永不叫停的写法骗过，
    /// 正推 A7 `let _ = stop_is_requested`、攻方 crash-2 `&|| { let _ = stop_is_requested; false }`、crash-3 `if false && stop_is_requested()`）：
    /// 不要卡的两层改由行为用例数叫停被问了几次——`check_pending_blocks_in_pieces` 由本模块
    /// `the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken`，`check_blocks_where_until` 由
    /// `tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 的 `a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest`
    /// （叫停在第二次被问时交真，只判了一块）。另四层换不成不要卡的行为用例：第 ① 段在卡上判那三层要卡，`check_selected_blocks` 要录好的流与库
    /// （只有 `run_from_environment` 调它，经环境跑的那几条用例里叫停文件放上时块都判过了，问不到这一层），这里扫源码：参数行之后到函数收尾的
    /// 顶格右括号，用到叫停整名的行（去掉首尾空白）逐行等于钉住的那几行，往下交的那一行之后紧跟右括号（是调下一层的最后一个实参）。
    /// 整行比拦得住上面三种写法（多一行、或那一行不是原样）。
    /// 第 10z 批第十八块（第 10z 批第十四、十五块代码轮判决 K4，正推 N1：`check_selected_blocks` 里真调用的叫停换成 `&|| false`，钉住的那一行
    /// 连同它后面的右括号原样留在 `if false { … }` 死分支里的另一次调用里，编得过、整行比照绿）：函数体短的三层
    /// （`judge_my_blocks_on_gpu_cards`、`check_selected_blocks`、`judge_my_blocks_on_gpu`）改成整句比——参数行之后到函数收尾
    /// （返回类型与函数体，去掉注释行与空白）整个等于钉住的那一句，死分支、多一次调用、换实参、换被调的函数都红；函数体太长、整句比不了的
    /// `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 照旧整行比那一行，另要它是函数体顶层的一句
    /// （不在任何块里：`if false { … }`、`if false && … { … }` 包着的红）、`stop_request` 在这一层只由它 `let` 绑一次
    /// （后面再 `let` 一次就换掉了记住的叫停）、`StopRequestRememberedInThisRun` 在这一层只出现在它里面（块里 `use` 或另起同名类型会遮住它）。
    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K4，正推 K4-3：名单外三种写法拦不住）：
    /// 收叫停的层改成按叫停类型找（`&(dyn Fn() -> bool + Sync)`，空白随意、可以跨行），名单记层名连同参数名——参数与别的参数挤在签名那一行
    /// （`the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_shares_the_signature_line`）、参数换了名字
    /// （`the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_is_renamed`）的新层都在名单外、红；没编进 `gpu` 特性时那一份
    /// `judge_my_blocks_on_gpu`（参数 `_stop_is_requested`）因此也进了名单，整句钉成只交错；长的那一层数 `stop_request` 被 `let` 绑了几次
    /// 改成看每个 `let`（`if let`、`while let` 也算）到 `=` 或 `;` 之前那一截里有没有整个名字 `stop_request`——解构
    /// `let (stop_request, …) = …`、`let` 后面多几个空格都数得到
    /// （`the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces`）；
    /// 数花括号的层数认得原始字符串与块注释（`brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines`），
    /// 长的那一层里写原始字符串不再让这条用例当场红，`if false { /* } */` 也骗不过层数。
    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4）：测试模块只从代码里那一处头两行切（块注释里写一份头两行、藏住它与真测试模块
    /// 之间的新层，攻方 K4-2 v2，`the_tests_module_is_cut_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string`）；
    /// 找层、数 `let`、数用到叫停的行都在注释与字面量换成空格之后的代码上做，整句、整行比在去掉注释的代码上做——签名行尾注释里的 `fn`
    /// 不再被当成层名（攻方 v4），行尾注释、字符串里的 `let stop_request` 与钉住那一行、短层函数体上的行尾注释不再判红
    /// （`the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings`）；钉住那一行与叫停参数上挂属性
    /// （`#[cfg(any())]` 编译时去掉那一句、那个参数，正推 K4-1 A01）判红（`the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter`）；
    /// 叫停写成钉住之外的类型（多加 `Send`、`'static`，`&mut`、`&'static`，类型里夹注释，`Box<dyn …>`，`dyn Sync + Fn() -> bool`，
    /// `&dyn Fn() -> bool`，`impl`、泛型、这份源文件里的类型别名，函数指针）的新层照 `Fn() -> bool` 这个形找到、判红
    /// （`the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way`，正推 K4-2 C02–C12）。
    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K3、K4）：测试模块的头两行只认按行逐字、括号第 0 层、前一个词是条目收尾的那一处，
    /// 拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进宏调用的圆括号里（正推 K4-1 S14）、包着的假测试模块里藏一层（S19）都红，
    /// 恰好一处的上界与第 0 层两道各有用例钉（正推 K3-5 W8、W9、X6、X7，`the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item`）；
    /// 叫停闭包的返回类型写成 `::core::primitive::bool`、`std::primitive::bool`、套括号的 `(bool)` 的新层照形找到、红
    /// （正推 K4-1 N01、N02，`the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses`）。
    ///
    /// 拦不住的写法，一种一条：
    /// - 同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管；第 ① 段在卡上判那一层里 `stop_request` 往下怎么用由
    ///   `the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run` 管，也只看文本）；
    /// - 长的那一层：钉住那一行之前提早返回，或让它之后的路走不到；`stop_request` 经 `let` 之外的绑定（`match` 的臂、闭包参数、`for`）
    ///   在块里换成别的；
    /// - 被调的那一层自己改了（短的几层整句比钉的是调谁、怎么交，不是被调的函数里做什么）；harness 档的 `StopRequestRememberedInThisRun::new` 改了；
    /// - 叫停的类型写成别的源文件、别的 crate 里定义的别名或 trait（`&crate::某处::叫停类型`），或换成不带 `Fn() -> bool` 字样的类型
    ///   （`&StopRequestRememberedInThisRun`、自己的 trait）的新层：只扫这一份源文件、只认这个形；
    /// - 叫停闭包的返回类型写成别名（`type 别名 = bool;` 之后 `Fn() -> 别名`，正推 K4-1 N03）或类型宏（`Fn() -> 某宏!()`，N15）的新层：
    ///   只认 `bool`、按路径写到 `core`、`std` 里的那个原始类型与外面套的括号，别名与宏展开出来的是什么，扫这一份源文件看不出；
    /// - 在函数上面加 `#[cfg]`，让照上一条另写类型的同名函数编进来（同样写成钉住的类型或这份源文件里看得见的形的，读到的名单多一层、红）；
    /// - 长的那一层里宏展开出来的调用与绑定；
    /// - 过程宏（属性宏）挂在测试模块上、或把它包住，把测试模块编进非测试构建：切口只认头两行的写法与所在的层，宏做了什么扫源码看不见。
    ///
    /// 合法写法判红、是故意拒的，一种一条：
    /// - 测试模块头两行换写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释）
    ///   与 CRLF 换行（[`text_outside_the_tests_module`]，两份文件是 rustfmt 出的，头两行就是一种写法）；
    /// - 叫停闭包参数之外、不带参数而返回以 `bool` 开头的元组的闭包（`Fn() -> (bool, u8)`；第 10z 批第二十六块写明，第 10z 批第二十四、
    ///   二十五块代码轮判决第二节 K3，正推 `N16`）：[`end_of_a_bool_return_type`] 跳过开头的左括号读到 `bool` 就算，不往后找与它配对的
    ///   右括号，分不出 `(bool)` 与以 `bool` 开头的元组；要分就得把返回类型照类型的语法读到收尾（元组里还能套泛型、路径、括号），这一扫
    ///   只认形，方向是不过就红、不会放过一层。这份文件里没有这样的闭包，要写就把返回类型写成具名的结构体。
    #[test]
    fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it() {
        assert_eq!(
            ways_a_layer_handed_the_stop_request_could_drop_it(include_str!(
                "crash_amplification.rs"
            )),
            Vec::<String>::new(),
            "收叫停的每一层照钉住的那样往下交或问"
        );
    }

    /// `source`（这份源文件，或用例在它上面改过的一份）测试模块之外，收叫停的层没照钉住的那样往下交或问的地方，一处一句；一处没有交空。
    /// 判法与名单在 `every_layer_handed_the_stop_request_hands_it_on_or_asks_it`。
    fn ways_a_layer_handed_the_stop_request_could_drop_it(source: &str) -> Vec<String> {
        let pinned_layers_without_a_behaviour_test: [(&str, &str, StopRequestUseThatIsPinned); 5] = [
            (
                "judge_my_blocks_on_gpu_cards",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<(CheckingTally,GpuJudgeOutcome),String>\u{7b}",
                    "lettables=crate::crash_library::record_the_judge_tables_and_read_them_back(store,&packed_judge_tables_of_the_plan(flow,plan)?,)?;",
                    "matchjudge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(",
                    "store,flow,plan,&tables,is_mine,&ConclusionsStoredUnder::TheJudgeVersion(version),cards,GpuCardsSource::DevicesStartedForThisRun,stop_is_",
                    "requested,)?\u{7b}",
                    "GpuJudgingOfMyBlocks::Judged\u{7b}tally,outcome\u{7d}=>Ok((tally,outcome)),",
                    "GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort\u{7b}reason,..\u{7d}=>Err(reason),",
                    "\u{7d}"
                )),
            ),
            (
                "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::TopLevelStatementOfALongBody {
                    line: concat!(
                        "let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
                        "requested);"
                    ),
                    binding: "stop_request",
                    type_name: "StopRequestRememberedInThisRun",
                },
            ),
            (
                "check_selected_blocks",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<CheckingTally,VerdictStoreError>\u{7b}",
                    "check_blocks_where_until(store,flow,plan,",
                    "&|block_sequence|selected_block_sequences.contains(&block_sequence),",
                    "conclusions,stop_is_",
                    "requested,)"
                )),
            ),
            (
                "judge_my_blocks_on_gpu",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<GpuJudgingOfMyBlocks,String>\u{7b}",
                    "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(",
                    "store,flow,plan,tables,is_mine,conclusions,cards,cards_taken_from,stop_is_",
                    "requested,)"
                )),
            ),
            (
                "judge_my_blocks_on_gpu",
                concat!("_stop_is_", "requested"),
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<GpuJudgingOfMyBlocks,String>\u{7b}",
                    "Err(\"配置写了ENABLE_GPU=1，这份二进制没有编进gpu特性（cargo…--featuresverdict-store,gpu）\".to_string())"
                )),
            ),
        ];
        let outside_the_tests = text_outside_the_tests_module(source) + "\n";
        let outside_the_tests_lines: Vec<&str> = outside_the_tests.split('\n').collect();
        let read_lines = lines_read_past_comments_and_literals(&outside_the_tests_lines);
        // 两份与 outside_the_tests 逐字节同位：找名字、数 let 与花括号照 code（注释与字面量都不算），整句、整行比照 without_comments
        let without_comments = read_lines
            .iter()
            .map(|read_line| read_line.without_comments.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        let code = read_lines
            .iter()
            .map(|read_line| read_line.without_comments_and_literals.as_str())
            .collect::<Vec<&str>>()
            .join("\n");
        let mut found = Vec::new();
        let mut layers_handed_the_stop_request = Vec::new();
        let places = places_taking_the_stop_request(&code);
        for place in &places {
            let PlaceTakingTheStopRequest {
                layer,
                parameter,
                is_the_parameter_under_an_attribute,
                start_of_the_type: _,
                end_of_the_type,
            } = *place;
            if is_the_parameter_under_an_attribute {
                found.push(format!(
                    "{layer} 的叫停参数 {parameter} 前面挂着属性（#[cfg(any())] 让参数编译时去掉，函数体里的名字就落到别处同名的条目上）"
                ));
            }
            let layer_length = code[end_of_the_type..]
                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
                .unwrap_or(code.len() - end_of_the_type);
            let rest_of_the_layer: Vec<&str> = without_comments
                [end_of_the_type..end_of_the_type + layer_length]
                .lines()
                .collect();
            let rest_of_the_layer_code: Vec<&str> = code
                [end_of_the_type..end_of_the_type + layer_length]
                .lines()
                .collect();
            match pinned_layers_without_a_behaviour_test
                .iter()
                .find(|(pinned_layer, pinned_parameter, _pinned)| {
                    *pinned_layer == layer && *pinned_parameter == parameter
                })
                .map(|(_layer, _parameter, pinned)| pinned)
            {
                Some(StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(expected_body)) => {
                    let body_without_spaces: String = rest_of_the_layer
                        .iter()
                        .flat_map(|line| line.chars())
                        .filter(|character| !character.is_whitespace())
                        .collect();
                    // 参数后面的逗号（参数单独占一行时有）不算函数体
                    let body_after_the_parameter = body_without_spaces
                        .strip_prefix(',')
                        .unwrap_or(&body_without_spaces);
                    if body_after_the_parameter != *expected_body {
                        found.push(format!(
                            "{layer} 参数之后整句等于钉住的那一句（死分支里留一份原样的调用、真调用换实参，都多出或换掉一截）：{rest_of_the_layer:#?}"
                        ));
                    }
                }
                Some(StopRequestUseThatIsPinned::TopLevelStatementOfALongBody {
                    line: pinned_line,
                    binding,
                    type_name,
                }) => {
                    let lines_using_it: Vec<(usize, &str)> = rest_of_the_layer_code
                        .iter()
                        .enumerate()
                        .filter(|(_offset, line)| whole_name_uses_of_the_stop_request(line) > 0)
                        .map(|(offset, _line)| (offset, rest_of_the_layer[offset].trim()))
                        .collect();
                    if lines_using_it
                        .iter()
                        .map(|(_offset, line)| *line)
                        .collect::<Vec<&str>>()
                        != [*pinned_line]
                    {
                        found.push(format!(
                            "{layer} 用到叫停的只有钉住的那一行、原样（只提一次名字而永不叫停的写法多一行或那一行变了）：{rest_of_the_layer:#?}"
                        ));
                    }
                    let depths = brace_depth_at_the_start_of_each_line(&rest_of_the_layer_code);
                    for (pinned_offset, _line) in &lines_using_it {
                        if depths[*pinned_offset] != 1 {
                            found.push(format!(
                                "{layer} 的 {pinned_line} 是函数体顶层的一句，不在 `if false` 一类的块里（参数之后那一行开函数体，算第 1 层），读到的是第 {} 层",
                                depths[*pinned_offset]
                            ));
                        }
                        let code_before_the_pinned_line =
                            rest_of_the_layer_code[..*pinned_offset].join("\n");
                        if !code_before_the_pinned_line
                            .trim_end()
                            .ends_with([';', '\u{7b}', '\u{7d}'])
                        {
                            found.push(format!(
                                "{layer} 的 {pinned_line} 前面紧挨着的是上一句的收尾（分号、左右花括号），不是属性：#[cfg(any())] 挂在它上面编译时整句去掉，下面用的 {binding} 就落到别处同名的条目上"
                            ));
                        }
                    }
                    let rest_of_the_layer_text = rest_of_the_layer_code.join("\n");
                    let let_bindings = whole_word_positions(&rest_of_the_layer_text, "let")
                        .into_iter()
                        .filter(|let_keyword| {
                            let after_the_keyword = &rest_of_the_layer_text[let_keyword + 3..];
                            let pattern_end = after_the_keyword
                                .find(['=', ';'])
                                .unwrap_or(after_the_keyword.len());
                            !whole_word_positions(&after_the_keyword[..pattern_end], binding)
                                .is_empty()
                        })
                        .count();
                    if let_bindings != 1 {
                        found.push(format!(
                            "{layer} 里 {binding} 只由钉住的那一行 let 绑一次（后面再 let 一次、解构、if let 就换掉了记住的叫停），读到 {let_bindings} 次"
                        ));
                    }
                    let lines_naming_the_type = rest_of_the_layer_code
                        .iter()
                        .filter(|line| line.contains(type_name))
                        .count();
                    if lines_naming_the_type != 1 {
                        found.push(format!(
                            "{layer} 里只有钉住的那一行提到 {type_name}（块里 use 或另起同名的类型会遮住它），读到 {lines_naming_the_type} 行"
                        ));
                    }
                }
                None => {}
            }
            layers_handed_the_stop_request.push((layer, parameter));
        }
        found.extend(stop_request_shapes_outside_the_pinned_type(&code, &places));
        let expected_layers = [
            ("check_blocks_where_until", STOP_REQUEST_PARAMETER_NAME),
            (
                "check_blocks_where_until_with_threads",
                STOP_REQUEST_PARAMETER_NAME,
            ),
            (
                "check_pending_blocks_in_pieces",
                STOP_REQUEST_PARAMETER_NAME,
            ),
            ("judge_my_blocks_on_gpu_cards", STOP_REQUEST_PARAMETER_NAME),
            (
                "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from",
                STOP_REQUEST_PARAMETER_NAME,
            ),
            ("check_selected_blocks", STOP_REQUEST_PARAMETER_NAME),
            (
                "record_the_observers_of_the_flow",
                STOP_REQUEST_PARAMETER_NAME,
            ),
            ("judge_my_blocks_on_gpu", STOP_REQUEST_PARAMETER_NAME),
            ("judge_my_blocks_on_gpu", concat!("_stop_is_", "requested")),
        ];
        if layers_handed_the_stop_request != expected_layers {
            found.push(format!(
                "收叫停的是这八层（观察者的录入段那一层每录一块问一次），连同没编进 gpu 特性时那一份 judge_my_blocks_on_gpu（多一层少一层、参数换了名字都要回来看这条用例罩没罩到），读到的是 {layers_handed_the_stop_request:?}"
            ));
        }
        found
    }

    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K4，正推 K4-3 第 1 种 `bypass_extra_layer_one_line`）：新加一层收叫停、
    /// 叫停参数与别的参数挤在签名那一行里，照样找得到这一层，它不在名单里就红。扫这份源码改过的一份，不要卡。
    #[test]
    fn the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_shares_the_signature_line() {
        let new_layer = concat!(
            "fn an_extra_layer_never_asking(first_input: u8, stop_is_",
            "requested: &(dyn Fn() -> bool + Sync)) -> bool \u{7b} first_input == 0 \u{7d}"
        );
        let source_with_the_new_layer =
            format!("{}\n{new_layer}\n", include_str!("crash_amplification.rs"));
        let found = ways_a_layer_handed_the_stop_request_could_drop_it(&source_with_the_new_layer);
        assert!(
            found
                .iter()
                .any(|finding| finding.contains("(\"an_extra_layer_never_asking\", ")),
            "签名一行里收叫停的新层进了读到的名单、与钉住的名单对不上：{found:#?}"
        );
    }

    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K4，正推 K4-3 第 2 种 `bypass_extra_layer_other_name`）：新加一层收叫停、
    /// 叫停参数换了名字，照样找得到这一层，它不在名单里就红。扫这份源码改过的一份，不要卡。
    #[test]
    fn the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_is_renamed() {
        let new_layer = concat!(
            "fn an_extra_layer_with_a_renamed_stop(\n",
            "    first_input: u8,\n",
            "    stop: &(dyn Fn() -> bool + Sync),\n",
            ") -> bool \u{7b}\n",
            "    first_input == 0\n",
            "\u{7d}"
        );
        let source_with_the_new_layer =
            format!("{}\n{new_layer}\n", include_str!("crash_amplification.rs"));
        let found = ways_a_layer_handed_the_stop_request_could_drop_it(&source_with_the_new_layer);
        assert!(
            found
                .iter()
                .any(|finding| finding
                    .contains("(\"an_extra_layer_with_a_renamed_stop\", \"stop\")")),
            "参数换了名字的新层进了读到的名单、与钉住的名单对不上：{found:#?}"
        );
    }

    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮判决 K4，正推 K4-3 第 3 种 `bypass_tuple_let_rebind`、`bypass_let_extra_spaces`）：
    /// 长的那一层钉住那一行之后解构重绑 `stop_request`、或 `let` 与名字之间多几个空格重绑，都数成又 `let` 了一次。扫这份源码改过的一份，不要卡。
    #[test]
    fn the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces(
    ) {
        let source = include_str!("crash_amplification.rs");
        let pinned_line = concat!(
            "    let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
            "requested);\n"
        );
        assert_eq!(
            source.matches(pinned_line).count(),
            1,
            "长的那一层钉住的那一行在这份源码里恰好一处"
        );
        for (form, rebinding) in [
            (
                "解构",
                "    let (stop_request, _unused) = (the_never_stopping_request(), 0);\n",
            ),
            (
                "多几个空格",
                "    let   stop_request  =  the_never_stopping_request();\n",
            ),
        ] {
            let rebound = source.replacen(pinned_line, &format!("{pinned_line}{rebinding}"), 1);
            let found = ways_a_layer_handed_the_stop_request_could_drop_it(&rebound);
            assert!(
                found.iter().any(|finding| finding.contains(
                    "里 stop_request 只由钉住的那一行 let 绑一次（后面再 let 一次、解构、if let 就换掉了记住的叫停），读到 2 次"
                )),
                "{form}重绑数成第二次 let：{found:#?}"
            );
        }
    }

    /// 第 10z 批第二十一块（第 10z 批第十六至十九块代码轮正推 K5：数花括号遇到原始字符串当场 `assert!`）：原始字符串（里面的 `"`、
    /// 收尾的反斜杠、跨行）、块注释（嵌套、里面的右花括号）、跨行的普通字符串里的花括号都不算层数。不要卡。
    #[test]
    fn brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines() {
        let lines = [
            "let path = r\"C:\\\";",
            "if path.is_empty() \u{7b}",
            "    let quoted = r#\"say \"\u{7b}\" here\"#;",
            "    let bytes = br##\"\u{7d}\"#\u{7d}\"##;",
            "    let spanning = r#\"",
            "\u{7b}\u{7b}",
            "\"#;",
            "    /* a \u{7d} in a comment */",
            "    let nested = /* outer /* inner \u{7d} */ still \u{7d} */ 0;",
            "    let plain = \"",
            "\u{7b}\";",
            "\u{7d}",
            "after",
        ];
        assert_eq!(
            brace_depth_at_the_start_of_each_line(&lines),
            vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
            "只有 if 那一行的左花括号与倒数第二行的右花括号算层数：{lines:#?}"
        );
    }

    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，攻方 K4-2 v2）：真测试模块头两行之前写一份在块注释里（或跨行的字符串里），
    /// 中间放一层收叫停而不问的新层、一个按名字读名单外环境变量的函数：测试模块只从代码里那一处切，两样都照样扫得到。扫这份源码改过的一份，不要卡。
    #[test]
    fn the_tests_module_is_cut_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string() {
        let source = include_str!("crash_amplification.rs");
        assert_eq!(
            source.matches(TESTS_MODULE_OPENING).count(),
            1,
            "这份源码里测试模块的头两行原文只有代码里那一处"
        );
        let new_layer = concat!(
            "pub fn an_extra_layer_never_asking(first_input: u8, stop_is_",
            "requested: &(dyn Fn() -> bool + Sync)) -> bool \u{7b} let _ = stop_is_",
            "requested; first_input == 0 \u{7d}\n"
        );
        let reading_a_variable_left_out = concat!(
            "pub fn reads_a_variable_left_out_of_the_list() -> bool \u{7b} ",
            "std::env::var(\"SINGLEFS_A_VARIABLE_LEFT_OUT_OF_THE_LIST\").is_ok() \u{7d}\n"
        );
        for (form, copy_of_the_opening) in [
            ("块注释", format!("/*\n{TESTS_MODULE_OPENING}\n*/\n")),
            (
                "跨行的字符串",
                format!(
                    "pub const A_COPY_OF_THE_OPENING: &str = \"\n{TESTS_MODULE_OPENING}\n\";\n"
                ),
            ),
        ] {
            let hiding = source.replacen(
                TESTS_MODULE_OPENING,
                &format!(
                    "{copy_of_the_opening}{new_layer}{reading_a_variable_left_out}{TESTS_MODULE_OPENING}"
                ),
                1,
            );
            let found = ways_a_layer_handed_the_stop_request_could_drop_it(&hiding);
            assert!(
                found
                    .iter()
                    .any(|finding| finding.contains("(\"an_extra_layer_never_asking\", ")),
                "{form}里写一份测试模块的头两行，藏不住它与真测试模块之间收叫停的新层：{found:#?}"
            );
            let names_read = environment_variables_read_by_name_outside_the_tests(&hiding);
            assert!(
                names_read
                    .iter()
                    .any(|name| name == "SINGLEFS_A_VARIABLE_LEFT_OUT_OF_THE_LIST"),
                "{form}里写一份测试模块的头两行，藏不住它与真测试模块之间按名字读的环境变量：{names_read:?}"
            );
        }
    }

    /// 跑 `check`，它 panic 时交 panic 的消息，不 panic 交 `None`：扫源码的用例看判法在改过的源码上红没红、红在哪一句。
    fn panic_message_of(check: impl FnOnce() + std::panic::UnwindSafe) -> Option<String> {
        std::panic::catch_unwind(check).err().map(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|message| (*message).to_string())
                })
                .unwrap_or_default()
        })
    }

    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K3、K4，正推 K3-5 `X6`、`X7`，K4-1 `S14`、`S19`）：真测试模块头两行之前
    /// 写一份头两行，认不认照 [`text_outside_the_tests_module`] 那几条。宏调用圆括号里的（前一个词是左括号；里面藏一层收叫停的层，`S19`，
    /// 那一层留在测试模块之外、照样找得到；前面先放一条以分号收尾的条目）、花括号里的（前面先放一条以分号收尾的条目）、前一个词不是条目收尾的，
    /// 都不认，切口照旧在真测试模块；代码里两处都认的（恰好一处的上界）、真测试模块包进宏调用的圆括号（`S14`）、头两行换写法与 CRLF 换行
    /// （按行逐字的口径，故意拒），都红在「恰好一处」那一句。扫这份源码改过的一份，不要卡。
    /// 第 10z 批第二十六块（第 10z 批第二十四、二十五块代码轮判决第二节 K2，攻方 B2：括号那一道只数圆括号照绿）：真测试模块包进方括号的
    /// 宏调用（`keep_hidden![` 起、前面先放一条以分号收尾的条目、`];` 收尾，攻方 `c09`；rustc 编得过、非测试构建里照编），也红在「恰好一处」。
    #[test]
    fn the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item(
    ) {
        let source = include_str!("crash_amplification.rs");
        let outside_the_tests = text_outside_the_tests_module(source);
        let hidden_layer = concat!(
            "    pub fn a_layer_hidden_in_a_wrapped_copy_of_the_tests_module(stop: &(dyn Fn() -> bool + Sync)) -> bool ",
            "\u{7b} stop() \u{7d}\n"
        );
        let wrapped_copy_hiding_a_layer =
            format!("keep_hidden!(\n{TESTS_MODULE_OPENING}\n{hidden_layer}") + "\u{7d}\n);\n";
        for (form, copy_of_the_opening) in [
            (
                "宏调用圆括号里的一份，里面藏一层收叫停的层（S19）",
                wrapped_copy_hiding_a_layer.as_str(),
            ),
            (
                "宏调用圆括号里、前面先放一条以分号收尾的条目的一份",
                "keep_hidden!(\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n);\n",
            ),
            (
                "花括号里、前面先放一条以分号收尾的条目的一份",
                "mod a_module_holding_a_copy \u{7b}\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n\u{7d}\n",
            ),
            (
                "前一个词不是条目收尾的一份",
                "static NOT_ENDED_BY_A_SEMICOLON: u8 = 0\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n",
            ),
        ] {
            let with_a_copy = source.replacen(
                TESTS_MODULE_OPENING,
                &format!("{copy_of_the_opening}{TESTS_MODULE_OPENING}"),
                1,
            );
            assert_eq!(
                text_outside_the_tests_module(&with_a_copy).len(),
                outside_the_tests.len() + copy_of_the_opening.len(),
                "{form}不认，切口照旧在真测试模块"
            );
        }
        let with_a_wrapped_copy_hiding_a_layer = source.replacen(
            TESTS_MODULE_OPENING,
            &format!("{wrapped_copy_hiding_a_layer}{TESTS_MODULE_OPENING}"),
            1,
        );
        let found =
            ways_a_layer_handed_the_stop_request_could_drop_it(&with_a_wrapped_copy_hiding_a_layer);
        assert!(
            found
                .iter()
                .any(|finding| finding
                    .contains("a_layer_hidden_in_a_wrapped_copy_of_the_tests_module")),
            "宏包着的假测试模块里藏的那一层找得到、红（S19）：{found:#?}"
        );
        let opening_offset = source
            .find(TESTS_MODULE_OPENING)
            .expect("这份源文件里有测试模块的头两行");
        let end_of_the_tests_module = opening_offset
            + source[opening_offset..]
                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
                .expect("测试模块在顶格的右括号行收尾")
            + TOP_LEVEL_CLOSING_BRACE_LINE.len();
        let tests_wrapped_in_square_brackets_after_an_item =
            String::from(&source[..opening_offset])
                + "keep_hidden![\nconst BEFORE_THE_TESTS: u8 = 0;\n"
                + &source[opening_offset..end_of_the_tests_module]
                + "];\n"
                + &source[end_of_the_tests_module..];
        let mut refused_sources: Vec<(&str, String)> = vec![
            (
                "代码里两处都认的头两行",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &format!(
                        "{TESTS_MODULE_OPENING}{TOP_LEVEL_CLOSING_BRACE_LINE}{TESTS_MODULE_OPENING}"
                    ),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的圆括号（S14）",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &format!("keep_hidden!(\n{TESTS_MODULE_OPENING}"),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的圆括号、前面先放一条以分号收尾的条目",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &(String::from("keep_hidden!(\nconst BEFORE_THE_TESTS: u8 = 0;\n")
                        + TESTS_MODULE_OPENING),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的花括号、前面先放一条以分号收尾的条目",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &(String::from("keep_hidden! \u{7b}\nconst BEFORE_THE_TESTS: u8 = 0;\n")
                        + TESTS_MODULE_OPENING),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的方括号、前面先放一条以分号收尾的条目（攻方 c09）",
                tests_wrapped_in_square_brackets_after_an_item,
            ),
            ("CRLF 换行", source.replace('\n', "\r\n")),
        ];
        for (form, other_writing) in [
            ("mod 行少一个空格", "#[cfg(test)]\nmod tests\u{7b}"),
            ("写在一行", "#[cfg(test)] mod tests \u{7b}"),
            ("属性里多空格", "#[ cfg ( test ) ]\nmod tests \u{7b}"),
            ("mod 行多空格", "#[cfg(test)]\nmod  tests  \u{7b}"),
            ("头一行缩进", "    #[cfg(test)]\nmod tests \u{7b}"),
            ("头一行行尾带空白", "#[cfg(test)]  \nmod tests \u{7b}"),
            ("头一行行尾带注释", "#[cfg(test)] // 单测\nmod tests \u{7b}"),
            ("第二行行尾带注释", "#[cfg(test)]\nmod tests \u{7b} // 单测"),
        ] {
            refused_sources.push((
                form,
                source.replacen(TESTS_MODULE_OPENING, other_writing, 1),
            ));
        }
        for (form, refused_source) in &refused_sources {
            let message = panic_message_of(|| {
                drop(text_outside_the_tests_module(refused_source));
            });
            assert!(
                message
                    .as_deref()
                    .is_some_and(|message| message.contains("测试模块的头两行恰好一处")),
                "{form}：切口红在「恰好一处」那一句，读到 {message:?}"
            );
        }
    }

    /// 第 10z 批第二十五块（第 10z 批第二十二、二十三块代码轮判决 K4，正推 K4-1 N01、N02：叫停闭包的返回类型写成 `::core::primitive::bool`、
    /// `(bool)` 的新层，叫停原样可用，按形找不到、判绿）：返回类型按路径写到 `core`、`std` 里那个原始类型的、外面套括号的，
    /// 照 [`end_of_a_bool_return_type`] 认成 `bool`，新层找得到、红。扫这份源码改过的一份，不要卡。
    #[test]
    fn the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses()
    {
        let source = include_str!("crash_amplification.rs");
        for (marker, new_layer) in [
            (
                "an_extra_layer_whose_stop_returns_bool_by_the_core_path",
                "fn an_extra_layer_whose_stop_returns_bool_by_the_core_path(stop: &(dyn Fn() -> ::core::primitive::bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_returns_bool_by_a_relative_core_path",
                "fn an_extra_layer_whose_stop_returns_bool_by_a_relative_core_path(stop: &(dyn Fn() -> core::primitive::bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_returns_bool_by_the_std_path",
                "fn an_extra_layer_whose_stop_returns_bool_by_the_std_path(stop: &(dyn Fn() -> std :: primitive :: bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_returns_bool_in_parentheses",
                "fn an_extra_layer_whose_stop_returns_bool_in_parentheses(stop: &(dyn Fn() -> (bool) + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_returns_bool_by_a_path_in_two_parentheses",
                "fn an_extra_layer_whose_stop_returns_bool_by_a_path_in_two_parentheses(stop: &(dyn Fn() -> ( ( ::std::primitive::bool ) ) + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_a_function_pointer_returning_bool_in_parentheses",
                "fn an_extra_layer_whose_stop_is_a_function_pointer_returning_bool_in_parentheses(stop: fn() -> (bool)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_fn_mut_returning_bool_by_a_path",
                "fn an_extra_layer_whose_stop_is_fn_mut_returning_bool_by_a_path(stop: &mut dyn FnMut() -> core::primitive::bool) -> bool \u{7b} false \u{7d}",
            ),
        ] {
            let source_with_the_new_layer = format!("{source}\n{new_layer}\n");
            let found =
                ways_a_layer_handed_the_stop_request_could_drop_it(&source_with_the_new_layer);
            assert!(
                found.iter().any(|finding| finding.contains(marker)),
                "叫停闭包的返回类型换了写法的新层 {marker} 找得到、红：{found:#?}"
            );
        }
    }

    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，正推 K4-1 A01：长的那一层钉住那一行上面挂 `#[cfg(any())]`，模块里另放一个
    /// 叫 `stop_request` 的静态量，编译时那一行去掉、下面用的是静态量，整行比、层数、`let` 次数照绿）：钉住那一行前面紧挨着的（跳过注释）
    /// 要是上一句的收尾，挂着属性（一行的、跨行的、属性与那一行之间夹注释行的）都红；叫停参数前面挂属性同样红。扫这份源码改过的一份，不要卡。
    #[test]
    fn the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter() {
        let source = include_str!("crash_amplification.rs");
        let pinned_line = concat!(
            "    let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
            "requested);\n"
        );
        assert_eq!(
            source.matches(pinned_line).count(),
            1,
            "长的那一层钉住的那一行在这份源码里恰好一处"
        );
        for (form, attribute) in [
            ("一行的属性", "    #[cfg(any())]\n"),
            ("跨行的属性", "    #[cfg(\n        any()\n    )]\n"),
            (
                "属性与那一行之间夹注释行",
                "    #[cfg(any())]\n    // 编译时这一句去掉\n",
            ),
        ] {
            let under_an_attribute =
                source.replacen(pinned_line, &format!("{attribute}{pinned_line}"), 1);
            let found = ways_a_layer_handed_the_stop_request_could_drop_it(&under_an_attribute);
            assert!(
                found
                    .iter()
                    .any(|finding| finding.contains("前面紧挨着的是上一句的收尾")),
                "钉住那一行上面挂{form}，红：{found:#?}"
            );
        }
        let stop_parameter = concat!("\n    stop_is_", "requested: &(dyn Fn() -> bool + Sync),\n");
        let parameter_under_an_attribute = source.replacen(
            stop_parameter,
            &format!("\n    #[cfg(any())]{stop_parameter}"),
            1,
        );
        assert_ne!(
            parameter_under_an_attribute, source,
            "造的源码在叫停参数上挂了属性"
        );
        let found =
            ways_a_layer_handed_the_stop_request_could_drop_it(&parameter_under_an_attribute);
        assert!(
            found.iter().any(|finding| finding
                .contains(concat!("的叫停参数 stop_is_", "requested 前面挂着属性"))),
            "叫停参数上面挂属性，红：{found:#?}"
        );
    }

    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，正推 K4-2 C02–C12）：新加一层收叫停、叫停写成钉住的
    /// `&(dyn Fn() -> bool + Sync)` 之外的类型，照 `Fn() -> bool` 这个形（`FnMut`、`FnOnce`、函数指针也算）找到、红；类型里夹注释的，
    /// 注释不算，照钉住的类型找到这一层、它不在名单里红。扫这份源码改过的一份，不要卡。
    #[test]
    fn the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way() {
        let source = include_str!("crash_amplification.rs");
        for (marker, new_layer) in [
            (
                "an_extra_layer_whose_stop_is_also_send",
                "fn an_extra_layer_whose_stop_is_also_send(stop: &(dyn Fn() -> bool + Send + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_static",
                "fn an_extra_layer_whose_stop_is_static(stop: &(dyn Fn() -> bool + Sync + 'static)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_borrowed_mutably",
                "fn an_extra_layer_whose_stop_is_borrowed_mutably(stop: &mut (dyn Fn() -> bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_lives_for_the_whole_run",
                "fn an_extra_layer_whose_stop_lives_for_the_whole_run(stop: &'static (dyn Fn() -> bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_type_holds_a_comment",
                "fn an_extra_layer_whose_stop_type_holds_a_comment(stop: &(dyn Fn() -> bool /* 叫停 */ + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_boxed",
                "fn an_extra_layer_whose_stop_is_boxed(stop: Box<dyn Fn() -> bool + Sync>) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_names_sync_first",
                "fn an_extra_layer_whose_stop_names_sync_first(stop: &(dyn Sync + Fn() -> bool)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_a_bare_trait_object",
                "fn an_extra_layer_whose_stop_is_a_bare_trait_object(stop: &dyn Fn() -> bool) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_an_impl",
                "fn an_extra_layer_whose_stop_is_an_impl(stop: impl Fn() -> bool) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_generic",
                "fn an_extra_layer_whose_stop_is_generic<Stop: Fn() -> bool + Sync>(stop: &Stop) -> bool \u{7b} false \u{7d}",
            ),
            (
                "StopRequestOfAnExtraLayer",
                "type StopRequestOfAnExtraLayer = dyn Fn() -> bool + Sync;\nfn an_extra_layer_whose_stop_is_an_alias(stop: &StopRequestOfAnExtraLayer) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_fn_mut",
                "fn an_extra_layer_whose_stop_is_fn_mut(stop: &mut dyn FnMut() -> bool) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_a_function_pointer",
                "fn an_extra_layer_whose_stop_is_a_function_pointer(stop: fn() -> bool) -> bool \u{7b} false \u{7d}",
            ),
        ] {
            let source_with_the_new_layer = format!("{source}\n{new_layer}\n");
            let found =
                ways_a_layer_handed_the_stop_request_could_drop_it(&source_with_the_new_layer);
            assert!(
                found.iter().any(|finding| finding.contains(marker)),
                "叫停写成别的类型的新层 {marker} 找得到、红：{found:#?}"
            );
        }
    }

    /// 第 10z 批第二十三块（第 10z 批第二十、二十一块代码轮判决 K4，攻方 v4、正推 K4-2 A11、A12、B02）：注释与字符串里写的不算——签名行尾注释里的
    /// `fn` 不被当成层名，字符串、行尾注释里的 `let stop_request` 不算又绑一次，钉住那一行、短层函数体上的行尾注释不算改了那一句，都判绿。
    /// 扫这份源码改过的一份，不要卡。
    #[test]
    fn the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings(
    ) {
        let source = include_str!("crash_amplification.rs");
        let pinned_line = concat!(
            "    let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
            "requested);\n"
        );
        let signature_lines = concat!(
            "    cards_taken_from: GpuCardsSource,\n    stop_is_",
            "requested: &(dyn"
        );
        let call_line_of_a_short_layer =
            "        &|block_sequence| selected_block_sequences.contains(&block_sequence),\n";
        for (anchor, replacement, form) in [
            (
                signature_lines,
                signature_lines.replacen(",\n", ", // 判法版本，fn 外面定\n", 1),
                "签名行尾注释里写 fn",
            ),
            (
                pinned_line,
                format!("{pinned_line}    let _quoted = \"let stop_request = the_never_stopping_request();\";\n"),
                "字符串里写 let stop_request",
            ),
            (
                pinned_line,
                format!("{pinned_line}    let _counted_once = 0; // let stop_request = 换掉记住的叫停\n"),
                "行尾注释里写 let stop_request",
            ),
            (
                pinned_line,
                pinned_line.replacen(";\n", "; // 记住的叫停\n", 1),
                "钉住那一行带行尾注释",
            ),
            (
                call_line_of_a_short_layer,
                call_line_of_a_short_layer.replacen(",\n", ", // 只判点了名的块\n", 1),
                "短层函数体上带行尾注释",
            ),
        ] {
            assert!(
                source.contains(anchor),
                "{form}：这份源码里有要改的那一处 {anchor}"
            );
            let with_comments_and_strings = source.replacen(anchor, &replacement, 1);
            assert_eq!(
                ways_a_layer_handed_the_stop_request_could_drop_it(&with_comments_and_strings),
                Vec::<String>::new(),
                "{form}，判绿"
            );
        }
    }

    /// J5：配置开了 GPU、编进了 `gpu` 特性，判器、核对两类都走 GPU（没有登记哪几类走 GPU 的正表，豁免的那几截不走这里）。
    #[test]
    fn every_computation_runs_on_the_gpu_when_the_gpu_is_switched_on() {
        let routes = computation_routes(true, true).expect("开了 GPU、编进了 gpu 特性");
        assert_eq!(
            routes,
            ComputationRoutes {
                crash_judge: ComputedOn::Gpu,
                facts_verifier: ComputedOn::Gpu,
            }
        );
        assert!(routes.need_a_card());
    }

    /// J5：配置没开 GPU，两类都在 CPU 上，一张卡都不要；编没编进 `gpu` 特性都一样。
    #[test]
    fn every_computation_runs_on_the_cpu_when_the_gpu_is_switched_off() {
        for gpu_feature_is_compiled_in in [true, false] {
            let routes =
                computation_routes(false, gpu_feature_is_compiled_in).expect("没开 GPU 不报错");
            assert_eq!(
                routes,
                ComputationRoutes {
                    crash_judge: ComputedOn::Cpu,
                    facts_verifier: ComputedOn::Cpu,
                },
                "编进 gpu 特性：{gpu_feature_is_compiled_in}"
            );
            assert!(!routes.need_a_card());
        }
    }

    /// J5：开了 GPU 而这份二进制没编进 `gpu` 特性：报错并说要带的特性，不悄悄退回 CPU。
    #[test]
    fn the_gpu_switched_on_without_the_gpu_feature_compiled_in_is_refused() {
        let refused = computation_routes(true, false);
        assert!(
            refused.as_ref().is_err_and(
                |reason| reason.contains("gpu 特性") && reason.contains("verdict-store,gpu")
            ),
            "{refused:?}"
        );
    }

    /// J10：按段登记表五段一行，照第二段流名取行、按段号排：同一合表项别的流的行不取，第一段与流名相同而第二段不同的行也不取；
    /// `#` 起头的行、空行、行末第六段 `#` 说明都跳过。
    #[test]
    fn the_segments_registry_hands_back_the_rows_of_the_flow_sorted_by_segment() {
        let digest = "ab".repeat(DIGEST_BYTES);
        let text = format!(
            "# 注释\n\nposition-addressed-trees\tposition_addressed_Grows\t7\t26\t{digest}\t# 说明\n\
             position-addressed-trees\tposition_addressed_Inline\t3\t24\t{digest}\n\
             position_addressed_Grows\tposition_addressed_Other\t5\t9\t{digest}\n\
             position-addressed-trees\tposition_addressed_Grows\t0\t2\t{digest}\n"
        );
        let registered =
            registered_segments_of_the_flow(&text, "position_addressed_Grows").expect("写法都对");
        assert_eq!(
            registered,
            vec![
                RegisteredSegment {
                    segment_index: 0,
                    write_count: 2,
                    write_table_digest_text: digest.clone(),
                },
                RegisteredSegment {
                    segment_index: 7,
                    write_count: 26,
                    write_table_digest_text: digest.clone(),
                },
            ]
        );
        assert!(
            registered_segments_of_the_flow(&text, "position-addressed-trees")
                .expect("写法都对")
                .is_empty(),
            "拿合表项名来取，一行都取不到：引擎按第二段流名取"
        );
    }

    /// J10：去重按（流名，段号）：同一合表项的两条流各登自己的第 0 段放行，不当成重复。
    #[test]
    fn two_flows_of_one_item_may_each_register_their_own_segment_of_the_same_index() {
        let digest = "cd".repeat(DIGEST_BYTES);
        let text = format!(
            "position-addressed-trees\tposition_addressed_Grows\t0\t322\t{digest}\n\
             position-addressed-trees\tposition_addressed_Inline\t0\t12\t{digest}\n"
        );
        assert_eq!(
            registered_segments_of_the_flow(&text, "position_addressed_Inline"),
            Ok(vec![RegisteredSegment {
                segment_index: 0,
                write_count: 12,
                write_table_digest_text: digest.clone(),
            }]),
            "两条流同一段号各一行：不是同一条流同一段"
        );
    }

    /// J10、M6-10：登记了的段跑轻型两个域的并：写数不超过展开 6 的全展开（罩着甲二），超过的只取甲二，重型 ⊇ 轻型。
    #[test]
    fn a_registered_segment_takes_the_union_of_the_two_light_tier_domains() {
        assert_eq!(
            expansion_of_a_registered_segment(&[0, 1, 2, 3, 4, 5]),
            CrashPointReplaySegmentExpansion::EveryProperSubset,
            "6 次写：展开 6 那一域全展开"
        );
        assert_eq!(
            expansion_of_a_registered_segment(&[0, 1, 2, 3, 4, 5, 6]),
            CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll,
            "7 次写：只剩甲二那一域"
        );
    }

    /// J10：写坏的一行整趟拒，不当成「没登记」：错误里写第几行、哪一处不对。
    #[test]
    fn a_malformed_line_of_the_segments_registry_is_refused_naming_the_line() {
        let digest = "0f".repeat(DIGEST_BYTES);
        let item_and_flow = "multi-record-publish\tmulti_record_publish";
        for (line, reason) in [
            (format!("multi_record_publish\t7\t26\t{digest}"), "一行五段"),
            (format!("{item_and_flow}\t7\t26"), "一行五段"),
            (
                format!("\tmulti_record_publish\t7\t26\t{digest}"),
                "合表项名不许空",
            ),
            (format!("{item_and_flow}\t07\t26\t{digest}"), "段号"),
            (format!("{item_and_flow}\t7\t0\t{digest}"), "段写数"),
            (
                format!("{item_and_flow}\t7\t26\t{}", digest.to_uppercase()),
                "摘要",
            ),
            (format!("{item_and_flow}\t7\t26\t{}", &digest[1..]), "摘要"),
        ] {
            let text = format!("# 注释\n{line}\n");
            let refused = registered_segments_of_the_flow(&text, "multi_record_publish");
            assert!(
                refused
                    .as_ref()
                    .is_err_and(|error| error.contains("第 2 行") && error.contains(reason)),
                "「{line}」：{refused:?}"
            );
        }
        let twice = format!("{item_and_flow}\t7\t26\t{digest}\n{item_and_flow}\t7\t26\t{digest}\n");
        assert!(
            registered_segments_of_the_flow(&twice, "unlink_after_the_multi_record_publish")
                .is_err_and(|error| error.contains("同一条流同一段")),
            "同一条流同一段两行，别的流来读也拒"
        );
    }

    /// 驱动行新字段（C790 阻断二）：段写数 [322, 290, 1, 2] 的流、展开上限 6，没登记又盖不住的段逐个报，登了的、盖得住的不报。
    #[test]
    fn the_line_names_the_unregistered_segments_the_expansion_up_to_does_not_cover() {
        let segments_of_lengths = |lengths: &[usize]| -> Vec<Vec<usize>> {
            let mut next_write = 0usize;
            lengths
                .iter()
                .map(|length| {
                    let segment: Vec<usize> = (next_write..next_write + length).collect();
                    next_write += length;
                    segment
                })
                .collect()
        };
        let registered = |segment_index: usize, write_count: usize| RegisteredSegment {
            segment_index,
            write_count,
            write_table_digest_text: "ee".repeat(DIGEST_BYTES),
        };
        let fields_of = |lengths: &[usize], registered_segments: &[RegisteredSegment]| {
            let segments = segments_of_lengths(lengths);
            segment_fields_of_the_line(
                registered_segments,
                &unregistered_segments_beyond_the_expansion(
                    &segments,
                    registered_segments,
                    ExpandUpToWritesPerSegment(6),
                ),
                &[],
            )
        };
        let shape = [322, 290, 1, 2];
        assert_eq!(
            fields_of(&shape, &[registered(0, 322)]),
            " registered_segments=0:322 unregistered_segments_beyond_expansion=1:290 registered_segment_domains=- registered_segment_progress=-",
            "只登第 0 段：第 1 段 290 写没登、展开 6 盖不住"
        );
        assert_eq!(
            fields_of(&shape, &[registered(0, 322), registered(1, 290)]),
            " registered_segments=0:322,1:290 unregistered_segments_beyond_expansion=- registered_segment_domains=- registered_segment_progress=-",
            "两段长的都登了"
        );
        assert_eq!(
            fields_of(&shape, &[]),
            " registered_segments=- unregistered_segments_beyond_expansion=0:322,1:290 registered_segment_domains=- registered_segment_progress=-",
            "一段都没登：两段长的按段号升序都报"
        );
        assert_eq!(
            fields_of(&[6, 1, 2], &[]),
            " registered_segments=- unregistered_segments_beyond_expansion=- registered_segment_domains=- registered_segment_progress=-",
            "每段都不超过 6 次写：展开上限盖住全部段"
        );
    }

    /// 造一条流：每段按（单元写数，journal 记录写数）给，单元写在前；每次写一个扇区（512 字节，不取第三态）、各写各的块。
    /// 最后两段是一次根槽写、两次系统配置槽写（段写数 1、2）。一个崩溃点罩全部写、不 ignore。
    fn flow_parts_of_unit_and_journal_writes(
        unit_and_journal_writes_of_the_long_segments: &[(usize, usize)],
    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
        let one_sector_write = |write_index: usize, kind: StepKind| RetainedWrite {
            device: singlefs_core::address::DeviceIdentity(1),
            kind,
            is_force_unit_access: kind == StepKind::RootRecordFua,
            offset: singlefs_core::address::DeviceOffsetInBytes(
                u64::try_from(write_index).expect("写表下标装得进 u64") * 4096,
            ),
            contents: WrittenContents::Bytes(vec![1; 512]),
        };
        let mut writes = Vec::new();
        let mut segments = Vec::new();
        let mut segment_of_kinds =
            |kinds: Vec<StepKind>, writes_so_far: &mut Vec<RetainedWrite>| {
                let segment: Vec<usize> = kinds
                    .into_iter()
                    .map(|kind| {
                        writes_so_far.push(one_sector_write(writes_so_far.len(), kind));
                        writes_so_far.len() - 1
                    })
                    .collect();
                segments.push(segment);
            };
        for (unit_writes, journal_writes) in unit_and_journal_writes_of_the_long_segments {
            let kinds = std::iter::repeat_n(StepKind::UnitWrite, *unit_writes)
                .chain(std::iter::repeat_n(
                    StepKind::JournalRecord,
                    *journal_writes,
                ))
                .collect();
            segment_of_kinds(kinds, &mut writes);
        }
        segment_of_kinds(vec![StepKind::RootRecordFua], &mut writes);
        segment_of_kinds(vec![StepKind::SystemConfigurationSlot; 2], &mut writes);
        (writes, segments)
    }

    /// 溢出（C790）：322 写的长段登记之后，录入段排计划之前算态数的那一步不 panic，态数只按甲二 ∪ 展开 6，全量组合数（2^322）不算；
    /// 没登记的长段照展开上限不展开。
    #[test]
    fn a_registered_long_segment_is_counted_by_the_light_tier_domains_only() {
        let (writes, segments) = flow_parts_of_unit_and_journal_writes(&[(320, 2), (290, 0)]);
        assert_eq!(
            segments.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![322, 290, 1, 2]
        );
        let base = MemoryPool {
            devices: BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let limit = ExpandUpToWritesPerSegment(6);
        let expansion_of_the_caller =
            move |_segment_index: usize, segment: &[usize]| limit.expansion_of(segment);
        let caller_flow = CrashFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index: 612,
            versions: &[],
            versions_without_a_file: &[],
            expansion: &expansion_of_the_caller,
            crash_points: vec![CrashPointSpan {
                name: "probe".to_string(),
                code_file_in_repository: "crates/singlefs-checker-tier/src/crash_amplification.rs"
                    .to_string(),
                writes: 0..writes.len(),
                ignored: false,
            }],
        };
        let registered = vec![RegisteredSegment {
            segment_index: 0,
            write_count: 322,
            write_table_digest_text: "aa".repeat(DIGEST_BYTES),
        }];
        let expansion_in_the_recording_of_the_flow = |segment_index: usize, segment: &[usize]| {
            expansion_in_the_recording(&registered, &caller_flow, segment_index, segment)
        };
        let flow_to_record = CrashFlow {
            base: caller_flow.base,
            writes: caller_flow.writes,
            segments: caller_flow.segments,
            judged_root_index: caller_flow.judged_root_index,
            versions: caller_flow.versions,
            versions_without_a_file: caller_flow.versions_without_a_file,
            expansion: &expansion_in_the_recording_of_the_flow,
            crash_points: caller_flow.crash_points.clone(),
        };
        assert_eq!(
            states_of_the_flow_to_record(&flow_to_record, &no_segment_slices(4)),
            Ok(12),
            "第 0 段登记、322 写里 2 次 journal 记录写（原地写、一个扇区两态）：甲二 = 2^2 × 2（单元写全不落或全落）− 1 = 7；\
             第 1 段 290 写没登、展开 6 盖不住，0 个；第 2 段 1 写全展开 2^1 − 1 = 1；第 3 段 2 写全展开 2^2 − 1 = 3；\
             再加全部持久那一个：7 + 0 + 1 + 3 + 1 = 12"
        );
    }

    /// 造一条只有录入段那几样的流（C790 分片用）：`flow_parts_of_unit_and_journal_writes` 那一形，空的基础镜像，一个崩溃点罩全部写；
    /// 第 0 段登记、按 [`expansion_in_the_recording`] 展开，别的段照展开上限 6。交给 `use_the_flow`。
    fn with_a_flow_registering_its_first_segment<Outcome>(
        unit_and_journal_writes_of_the_long_segments: &[(usize, usize)],
        use_the_flow: impl FnOnce(&CrashFlow<'_>, &[RegisteredSegment]) -> Outcome,
    ) -> Outcome {
        with_a_flow_registering_its_first_segment_and_expanding_the_others_up_to(
            unit_and_journal_writes_of_the_long_segments,
            ExpandUpToWritesPerSegment(6),
            use_the_flow,
        )
    }

    /// 同 [`with_a_flow_registering_its_first_segment`]，别的段照 `expand_up_to` 展开。
    fn with_a_flow_registering_its_first_segment_and_expanding_the_others_up_to<Outcome>(
        unit_and_journal_writes_of_the_long_segments: &[(usize, usize)],
        expand_up_to: ExpandUpToWritesPerSegment,
        use_the_flow: impl FnOnce(&CrashFlow<'_>, &[RegisteredSegment]) -> Outcome,
    ) -> Outcome {
        let (writes, segments) =
            flow_parts_of_unit_and_journal_writes(unit_and_journal_writes_of_the_long_segments);
        let base = MemoryPool {
            devices: BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let registered = vec![RegisteredSegment {
            segment_index: 0,
            write_count: segments[0].len(),
            write_table_digest_text: "cc".repeat(DIGEST_BYTES),
        }];
        let expansion_with_the_registered_segment = |segment_index: usize, segment: &[usize]| {
            if segment_index == 0 {
                expansion_of_a_registered_segment(segment)
            } else {
                expand_up_to.expansion_of(segment)
            }
        };
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流有一次根槽写");
        let flow = CrashFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index,
            versions: &[],
            versions_without_a_file: &[],
            expansion: &expansion_with_the_registered_segment,
            crash_points: vec![CrashPointSpan {
                name: "probe".to_string(),
                code_file_in_repository: "crates/singlefs-checker-tier/src/crash_amplification.rs"
                    .to_string(),
                writes: 0..writes.len(),
                ignored: false,
            }],
        };
        use_the_flow(&flow, &registered)
    }

    /// 一个空的判定库（用例名做目录名，起跑先清空自己那一份）。
    fn an_empty_library_for(test_name: &str) -> (PathBuf, VerdictStore) {
        let directory = std::env::temp_dir().join(format!(
            "singlefs-c790-slices-{test_name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        let store = VerdictStore::create_empty(&directory).expect("建得了库");
        (directory, store)
    }

    /// 假的判段：计划里还没有判定块的块，每个状态记同一个判定向量（只钉块键、续点与进度，不判真的崩溃状态）。
    fn store_a_clean_verdict_for_every_unjudged_block(
        store: &mut VerdictStore,
        plan: &CrashPlan,
        version: &JudgeVersion,
    ) {
        let clean = store
            .register_verdict_vector(&crate::verdict_store::EncodedVerdictVector(
                b"clean".to_vec(),
            ))
            .expect("登记判定向量");
        for point in &plan.points {
            for local in point.blocks() {
                let key = point.block_key(&local);
                if store
                    .read_verdict_block(&key, version)
                    .expect("读块")
                    .is_none()
                {
                    let states =
                        usize::try_from(local.end - local.start).expect("一块装得进 usize");
                    store
                        .store_verdict_block(
                            &key,
                            version,
                            &VerdictBlock::new(vec![clean; states]).expect("一块至少一个状态"),
                            &[],
                        )
                        .expect("存块");
                }
            }
        }
    }

    /// 2^70 − 1：70 次两态原地写的一段按甲二（没有单元写）的整域。
    const DOMAIN_OF_SEVENTY_IN_PLACE_WRITES: &str = "1180591620717411303423";

    /// C790：登记的段整域装得进 u64（322 写里 2 次 journal 记录写，甲二 7 个态）是 `full`、不分片：计划里没有分片的段，
    /// 驱动行报 `registered_segment_domains=0:full registered_segment_progress=-`。
    #[test]
    fn a_registered_segment_whose_domain_fits_in_u64_is_full_and_is_not_sliced() {
        with_a_flow_registering_its_first_segment(&[(320, 2)], |flow, registered| {
            let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
            assert_eq!(
                domains,
                vec![DomainOfARegisteredSegment {
                    segment_index: 0,
                    domain_state_count: StateCount1280Bits::from_u64(7),
                    kind: DomainKindOfARegisteredSegment::Full,
                }],
                "2^2 × 2 − 1 = 7 装得进 u64"
            );
            assert_eq!(
                placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains),
                no_segment_slices(flow.segments.len())
            );
            let plan = plan_crash_points_with_slices(flow, &no_segment_slices(flow.segments.len()));
            assert!(plan.sliced_segments().is_empty());
            let (directory, store) = an_empty_library_for("full");
            let version = JudgeVersion([5; DIGEST_BYTES]);
            let domains_and_progress = domains_and_progress_of_the_registered_segments(
                &store,
                flow,
                &plan,
                registered,
                &ConclusionsStoredUnder::TheJudgeVersion(&version),
            )
            .expect("读库");
            assert!(
                segment_fields_of_the_line(registered, &[], &domains_and_progress)
                    .ends_with(" registered_segment_domains=0:full registered_segment_progress=-"),
                "{}",
                segment_fields_of_the_line(registered, &[], &domains_and_progress)
            );
            drop(store);
            std::fs::remove_dir_all(&directory).expect("删得掉临时库");
        });
    }

    /// C790：登记的段是 70 次两态原地写（2^70 − 1 个态，装不进 u64）是 `slice`。片长 300：第一趟从库里找续点是 0（逐块问库，一块都没判），
    /// 判 [0, 300)、库里有这一块、进度记录是 300、驱动行进度 `0:300/<2^70 − 1>`；同一个库第二趟续点照进度记录核过是 300，判 [300, 600)：
    /// 第一趟那一块照旧判过、这一趟没判的只有新片那一块，进度记录与驱动行是 600。总态数逐字是 2^70 − 1 的十进制。
    #[test]
    fn a_registered_segment_past_u64_is_judged_one_slice_per_run_from_the_resume_point_in_the_library(
    ) {
        with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
            let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
            assert_eq!(domains[0].kind, DomainKindOfARegisteredSegment::Slice);
            assert_eq!(
                domains[0].domain_state_count.decimal_text(),
                DOMAIN_OF_SEVENTY_IN_PLACE_WRITES
            );
            let plan_to_find_the_resume_points = plan_crash_points_with_slices(
                flow,
                &placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains),
            );
            let (directory, mut store) = an_empty_library_for("two-runs");
            let version = JudgeVersion([5; DIGEST_BYTES]);
            let conclusions = ConclusionsStoredUnder::TheJudgeVersion(&version);
            let progress_key = SlicedSegmentProgressKey {
                flow_name: StreamOrNodeName::new("流").expect("名字合规矩"),
                segment_index: 0,
                judge: JudgeOfTheProgress::TheJudgeVersion(version),
            };
            let mut previous_plan: Option<CrashPlan> = None;
            for (run, expected_start) in [(1u64, 0u64), (2, 300)] {
                let (segment_slices, resume_points) = segment_slices_of_the_recording(
                    &store,
                    "流",
                    flow.segments.len(),
                    &domains,
                    &plan_to_find_the_resume_points,
                    &conclusions,
                    300,
                )
                .expect("读库");
                assert_eq!(
                    segment_slices[0],
                    Some(CrashPointReplaySegmentSlice {
                        start_within_the_domain: StateCount1280Bits::from_u64(expected_start),
                        state_count: 300,
                    }),
                    "第 {run} 趟从段内编号 {expected_start} 起判 300 个"
                );
                assert_eq!(
                    resume_points[0].1.found_by,
                    if run == 1 {
                        ResumePointFoundBy::AWalkFromZeroThroughTheLibrary
                    } else {
                        ResumePointFoundBy::TheProgressRecordChecked
                    }
                );
                let plan = plan_crash_points_with_slices(flow, &segment_slices);
                write_the_recorded_blocks(&mut store, &plan).expect("录入");
                let unjudged =
                    unjudged_blocks_of_the_flow(&store, &plan, &conclusions).expect("对账");
                if let Some(previous_plan) = &previous_plan {
                    assert_eq!(
                        unjudged.len(),
                        1,
                        "第二趟只有新片那一块没判，别的点照旧复用：{unjudged:?}"
                    );
                    assert!(
                        unjudged_blocks_of_the_flow(&store, previous_plan, &conclusions)
                            .expect("对账")
                            .is_empty(),
                        "第一趟的块照旧判过，不重判"
                    );
                }
                store_a_clean_verdict_for_every_unjudged_block(&mut store, &plan, &version);
                advance_the_progress_of_the_sliced_segments(&mut store, "流", &plan, &conclusions)
                    .expect("推进进度");
                let judged_through = expected_start + 300;
                assert_eq!(
                    store
                        .read_sliced_segment_progress(&progress_key)
                        .expect("读进度"),
                    Some(SlicedSegmentProgress {
                        judged_through: StateCount1280Bits::from_u64(judged_through),
                        last_block_start: StateCount1280Bits::from_u64(expected_start),
                    })
                );
                let domains_and_progress = domains_and_progress_of_the_registered_segments(
                    &store,
                    flow,
                    &plan,
                    registered,
                    &conclusions,
                )
                .expect("读库");
                assert!(
                    segment_fields_of_the_line(registered, &[], &domains_and_progress).ends_with(
                        &format!(
                            " registered_segment_domains=0:slice registered_segment_progress=0:{judged_through}/{DOMAIN_OF_SEVENTY_IN_PLACE_WRITES}"
                        )
                    ),
                    "第 {run} 趟：{}",
                    segment_fields_of_the_line(registered, &[], &domains_and_progress)
                );
                previous_plan = Some(plan);
            }
            drop(store);
            std::fs::remove_dir_all(&directory).expect("删得掉临时库");
        });
    }

    /// C790 被打断与进度记录坏了：一片四块（片长 4 × 2^16），第一趟只判了前两块就停、进度记录没写；重跑从库里逐块问到第三块的起点接着，
    /// 不从头、不漏块。进度记录落后（指着已判的块）、指到没判过的编号，都核不过、退回逐块问库，同样从第三块接着；记得对的照它接着。
    #[test]
    fn an_interrupted_slice_resumes_at_its_first_unjudged_block_and_a_wrong_progress_record_falls_back_to_a_walk(
    ) {
        with_a_flow_registering_its_first_segment(&[(0, 70)], |flow, registered| {
            let domains = domains_of_the_registered_segments(flow, registered).expect("算得出域");
            let plan_to_find_the_resume_points = plan_crash_points_with_slices(
                flow,
                &placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains),
            );
            let (directory, mut store) = an_empty_library_for("interrupted");
            let version = JudgeVersion([6; DIGEST_BYTES]);
            let conclusions = ConclusionsStoredUnder::TheJudgeVersion(&version);
            let slice_states = 4 * STATES_PER_BLOCK;
            let resume = |the_library: &VerdictStore| {
                let (slices_of_this_run, resume_points) = segment_slices_of_the_recording(
                    the_library,
                    "流",
                    flow.segments.len(),
                    &domains,
                    &plan_to_find_the_resume_points,
                    &conclusions,
                    slice_states,
                )
                .expect("读库");
                (slices_of_this_run, resume_points[0].1)
            };
            let (segment_slices, _resume_point) = resume(&store);
            let plan = plan_crash_points_with_slices(flow, &segment_slices);
            write_the_recorded_blocks(&mut store, &plan).expect("录入");
            // 被打断：分片的段四块只判了前两块，进度记录没写
            let clean = store
                .register_verdict_vector(&crate::verdict_store::EncodedVerdictVector(
                    b"clean".to_vec(),
                ))
                .expect("登记判定向量");
            let (point_index, piece) = plan.sliced_segments()[0];
            let point = &plan.points[point_index];
            let slice_blocks: Vec<Range<u64>> = point
                .blocks()
                .into_iter()
                .filter(|local| piece.local_states.contains(&local.start))
                .collect();
            assert_eq!(slice_blocks.len(), 4, "一片四块");
            for local in &slice_blocks[..2] {
                let states = usize::try_from(local.end - local.start).expect("一块装得进 usize");
                store
                    .store_verdict_block(
                        &point.block_key(local),
                        &version,
                        &VerdictBlock::new(vec![clean; states]).expect("一块至少一个状态"),
                        &[],
                    )
                    .expect("存块");
            }
            let first_unjudged = StateCount1280Bits::from_u64(2 * STATES_PER_BLOCK);
            let expect_a_walk_to_the_third_block = |the_library: &VerdictStore, case: &str| {
                let (slices_from_the_resume_point, resume_point_found) = resume(the_library);
                assert_eq!(
                    resume_point_found,
                    ResumePointOfASlicedSegment {
                        judged_through: first_unjudged,
                        found_by: ResumePointFoundBy::AWalkFromZeroThroughTheLibrary,
                    },
                    "{case}：逐块问库走到第三块的起点"
                );
                assert_eq!(
                    slices_from_the_resume_point[0].map(|slice| slice.start_within_the_domain),
                    Some(first_unjudged),
                    "{case}：从第三块接着判"
                );
            };
            expect_a_walk_to_the_third_block(&store, "没有进度记录");
            let progress_key = SlicedSegmentProgressKey {
                flow_name: StreamOrNodeName::new("流").expect("名字合规矩"),
                segment_index: 0,
                judge: JudgeOfTheProgress::TheJudgeVersion(version),
            };
            for (case, judged_through, last_block_start) in [
                (
                    "进度记录落后一块（指着的下一块已判过）",
                    STATES_PER_BLOCK,
                    0,
                ),
                (
                    "进度记录指到没判过的编号",
                    5 * STATES_PER_BLOCK,
                    4 * STATES_PER_BLOCK,
                ),
                ("进度记录说的最后一块长度对不上", 2 * STATES_PER_BLOCK, 0),
            ] {
                store
                    .store_sliced_segment_progress(
                        &progress_key,
                        SlicedSegmentProgress {
                            judged_through: StateCount1280Bits::from_u64(judged_through),
                            last_block_start: StateCount1280Bits::from_u64(last_block_start),
                        },
                    )
                    .expect("写进度");
                expect_a_walk_to_the_third_block(&store, case);
            }
            store
                .store_sliced_segment_progress(
                    &progress_key,
                    SlicedSegmentProgress {
                        judged_through: first_unjudged,
                        last_block_start: StateCount1280Bits::from_u64(STATES_PER_BLOCK),
                    },
                )
                .expect("写进度");
            assert_eq!(
                resume(&store).1,
                ResumePointOfASlicedSegment {
                    judged_through: first_unjudged,
                    found_by: ResumePointFoundBy::TheProgressRecordChecked,
                },
                "记得对的进度记录核过就照它接着"
            );
            // 重跑同一条命令：从第三块起录、判，判完推进到片尾
            let (segment_slices_of_the_rerun, _resume_point_of_the_rerun) = resume(&store);
            let plan_of_the_rerun =
                plan_crash_points_with_slices(flow, &segment_slices_of_the_rerun);
            write_the_recorded_blocks(&mut store, &plan_of_the_rerun)
                .expect("录入（第三、四块上一趟录过、块长相同）");
            store_a_clean_verdict_for_every_unjudged_block(
                &mut store,
                &plan_of_the_rerun,
                &version,
            );
            advance_the_progress_of_the_sliced_segments(
                &mut store,
                "流",
                &plan_of_the_rerun,
                &conclusions,
            )
            .expect("推进进度");
            assert_eq!(
                store
                    .read_sliced_segment_progress(&progress_key)
                    .expect("读进度")
                    .map(|progress| progress.judged_through),
                Some(StateCount1280Bits::from_u64(6 * STATES_PER_BLOCK)),
                "第二趟从第三块起判一片四块，判到第六块末尾"
            );
            drop(store);
            std::fs::remove_dir_all(&directory).expect("删得掉临时库");
        });
    }

    /// C790：登记的段整域越过 1280 位（1281 次两态原地写，2^1281 − 1 个态）在写库之前拒跑，拒跑句点名 1280 位与用户定的位宽。
    #[test]
    fn a_registered_segment_past_1280_bits_is_refused_before_recording() {
        with_a_flow_registering_its_first_segment(&[(0, 1281)], |flow, registered| {
            let refused = domains_of_the_registered_segments(flow, registered);
            assert!(
                refused
                    .as_ref()
                    .is_err_and(|error| error.contains("第 0 段（1281 次写）")
                        && error.contains("越过 1280 位")
                        && error.contains("用户 2026-10-13 定的位宽")
                        && error.contains("没写库")),
                "{refused:?}"
            );
        });
    }

    /// C790：第 0 段登记、70 次两态原地写（2^70 − 1 个态）要分片；第 1 段没登记、65 次两态原地写，展开上限 65 盖得住、全展开
    /// （2^65 − 1 个态，装不进 u64）。录入段定片那一步拒跑、拒跑句点名第 1 段，不在排找续点那份计划（分片的段片长为 0）时 panic；
    /// 库里没写：没有录入的块、没有这条流的清单。
    #[test]
    fn an_unregistered_segment_past_u64_beside_a_sliced_one_is_refused_before_recording() {
        with_a_flow_registering_its_first_segment_and_expanding_the_others_up_to(
            &[(0, 70), (0, 65)],
            ExpandUpToWritesPerSegment(65),
            |flow, registered| {
                let domains =
                    domains_of_the_registered_segments(flow, registered).expect("算得出域");
                assert_eq!(
                    domains[0].kind,
                    DomainKindOfARegisteredSegment::Slice,
                    "第 0 段整域装不进 u64，要分片"
                );
                let (directory, store) =
                    an_empty_library_for("unregistered-past-u64-beside-a-slice");
                let toolchain = CrashPointReplayToolchainIdentity {
                    rustc_version_lines: "rustc 用例不问".to_string(),
                    cargo_version: "cargo 用例不问".to_string(),
                    target_triple: "x86_64-unknown-linux-gnu".to_string(),
                };
                let refused = segment_slices_of_the_recording_from_the_environment_and_the_library(
                    &store,
                    flow,
                    "流",
                    registered,
                    Path::new(env!("CARGO_MANIFEST_DIR")),
                    &toolchain,
                    ComputedOn::Cpu,
                );
                assert!(
                    refused
                        .as_ref()
                        .is_err_and(|error| error.contains("第 1 段（65 次写）")
                            && error.contains("装不进 u64")
                            && error.contains("没写库")),
                    "{refused:?}"
                );
                assert!(
                    store.recorded_blocks().expect("读库").is_empty(),
                    "拒跑之后库里没有录入的块"
                );
                assert_eq!(
                    store.read_recorded_run_manifest("流").expect("读库"),
                    None,
                    "拒跑之后库里没有这条流的清单"
                );
                drop(store);
                std::fs::remove_dir_all(&directory).expect("删得掉临时库");
            },
        );
    }

    /// 片长变量：没设是 2^32；十进制正整数照用；0、负数、带别的字符、超过 2^32 都拒，错误带变量名与原值。
    #[test]
    fn the_slice_states_variable_is_a_positive_decimal_up_to_two_to_the_thirty_second() {
        assert_eq!(
            registered_segment_slice_states_of_the_variable_value(None),
            Ok(1 << 32)
        );
        assert_eq!(
            registered_segment_slice_states_of_the_variable_value(Some("300")),
            Ok(300)
        );
        assert_eq!(
            registered_segment_slice_states_of_the_variable_value(Some("4294967296")),
            Ok(1 << 32)
        );
        for refused in ["0", "-3", "3e2", " 300", "", "4294967297"] {
            assert!(
                registered_segment_slice_states_of_the_variable_value(Some(refused))
                    .is_err_and(|error| error
                        .contains(REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE)
                        && error.contains(&format!("={refused}："))),
                "{refused:?} 该拒"
            );
        }
    }

    /// J10、M4 第 15 条：登记的段写数与段写表摘要都对得上才放行；写表里那一段改了一个字节、段写数不同、段号越过这条流，都拒跑。
    #[test]
    fn a_registered_segment_whose_write_table_changed_is_refused() {
        let unit_write = |offset: u64, byte: u8| RetainedWrite {
            device: singlefs_core::address::DeviceIdentity(1),
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(offset),
            contents: WrittenContents::Bytes(vec![byte; 512]),
        };
        let base = MemoryPool {
            devices: BTreeMap::new(),
            device_size_in_bytes: 1 << 20,
        };
        let segments = vec![vec![0], vec![1, 2]];
        let expansion = |_segment_index: usize, _segment: &[usize]| {
            CrashPointReplaySegmentExpansion::NotExpanded
        };
        let flow_over = |writes: &[RetainedWrite]| -> String {
            let flow = CrashFlow {
                base: &base,
                writes,
                segments: &segments,
                judged_root_index: 0,
                versions: &[],
                versions_without_a_file: &[],
                expansion: &expansion,
                crash_points: Vec::new(),
            };
            segment_write_table_digest_text(&flow, 1).expect("第 1 段连续")
        };
        let writes = vec![unit_write(0, 1), unit_write(4096, 2), unit_write(8192, 3)];
        let digest = flow_over(&writes);
        let check = |writes_of_the_flow: &[RetainedWrite], registered: RegisteredSegment| {
            let flow = CrashFlow {
                base: &base,
                writes: writes_of_the_flow,
                segments: &segments,
                judged_root_index: 0,
                versions: &[],
                versions_without_a_file: &[],
                expansion: &expansion,
                crash_points: Vec::new(),
            };
            check_the_registered_segments_against_the_flow(&flow, "probe", &[registered])
        };
        let registered = |segment_index: usize, write_count: usize| RegisteredSegment {
            segment_index,
            write_count,
            write_table_digest_text: digest.clone(),
        };
        assert_eq!(check(&writes, registered(1, 2)), Ok(()), "对得上就放行");
        let mut changed = writes.clone();
        changed[2] = unit_write(8192, 4);
        assert!(
            check(&changed, registered(1, 2)).is_err_and(|error| error.contains("写表摘要")),
            "那一段改了一个写的内容"
        );
        assert!(
            check(&writes, registered(1, 3)).is_err_and(|error| error.contains("3 次写")),
            "段写数不同"
        );
        assert!(
            check(&writes, registered(5, 2)).is_err_and(|error| error.contains("只有 2 段")),
            "段号越过这条流"
        );
    }

    /// J4：观察者挂的节点照设点文件与入口名认计划里的点：`<入口>.<周期>.<步>` 的每一段都是，名字只是同一个前缀的不是，别的文件里同名的不是。
    #[test]
    fn an_observer_node_holds_every_point_of_its_entry_in_its_setting_file_only() {
        let node = NodeOfAnObserver {
            setting_file: "crates/singlefs-checker-tier/tests/common_crash_points/mod.rs"
                .to_string(),
            entry_name: "sequential_write".to_string(),
        };
        let point_named = |code_file: &str, name: &str| PlannedCrashPoint {
            name: name.to_string(),
            ignored: false,
            segments: Vec::new(),
            ordinals: 0..0,
            reuse_key: StepAttributes {
                write_table_with_barriers: [0; DIGEST_BYTES],
                parent_output: [0; DIGEST_BYTES],
            }
            .reuse_key(),
            path_number: name.to_string(),
            identity: format!("{code_file}::{name}::00"),
            judge_inputs: [0; DIGEST_BYTES],
            plan_hash: [0; DIGEST_BYTES],
            pieces_around_the_slices: Vec::new(),
        };
        let common = "crates/singlefs-checker-tier/tests/common_crash_points/mod.rs";
        assert!(node.holds_the_point(&point_named(common, "sequential_write.journal_record.1")));
        assert!(node.holds_the_point(&point_named(common, "sequential_write")));
        assert!(!node.holds_the_point(&point_named(
            common,
            "sequential_write_twice.journal_record.1"
        )));
        assert!(!node.holds_the_point(&point_named(
            "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs",
            "sequential_write.journal_record.1"
        )));
    }

    // ------------------------------ 观察者两段与按恢复镜像去重（用户 2026-10-13 定） ------------------------------

    /// 观察者两段那几条单测的设点文件与观察者代码文件（只当名字用：单测的观察者版本照下面另造，不经编译时的摘要表）。
    const OBSERVER_UNIT_TEST_SETTING_FILE: &str =
        "crates/singlefs-checker-tier/src/crash_amplification.rs";
    /// 那条小流唯一那个崩溃点的名字。
    const OBSERVER_UNIT_TEST_POINT_NAME: &str = "two_writes_of_the_same_bytes";

    /// 一条小流：一块盘，一段两个单元写写同一个扇区、同一份字节（这一段展开任意真子集）。状态是一个都没落、只落前一个、只落后一个，
    /// 加末节点（两个都落）：后三个的恢复镜像逐字节相同，与第一个不同。崩溃点一个、罩住两个写。
    fn a_flow_of_two_writes_of_the_same_bytes_to_one_sector() -> RecordedFlow {
        let device = singlefs_core::address::DeviceIdentity(1);
        let write = RetainedWrite {
            device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(8192),
            contents: WrittenContents::Bytes(vec![5; 512]),
        };
        RecordedFlow {
            base: MemoryPool::with_devices(&[device], 1 << 20),
            writes: vec![write.clone(), write],
            segments: vec![vec![0, 1]],
            judged_root_index: 0,
            versions: Vec::new(),
            versions_without_a_file: Vec::new(),
            segment_expansions: vec![CrashPointReplaySegmentExpansion::EveryProperSubset],
            crash_points: vec![CrashPointSpan {
                name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
                code_file_in_repository: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                writes: 0..2,
                ignored: false,
            }],
            segment_slices: vec![None],
        }
    }

    /// 单测的观察者那几把版本：观察者那一把随另读的输入变，代码那一把不随（与 [`observer_versions_to_write_under`] 同形，摘要另造）。
    fn versions_of_the_unit_test_observer(
        extra_inputs_digest: [u8; DIGEST_BYTES],
    ) -> ObserverVersions {
        let observer = ConclusionVersion(sha256_digest(
            &[
                b"observer version of the unit test\n".as_slice(),
                &extra_inputs_digest,
            ]
            .concat(),
        ));
        let pool_checker_version =
            ConclusionVersion(sha256_digest(b"cpu judge version of the unit test\n"));
        ObserverVersions {
            observer,
            observer_code: ConclusionVersion(sha256_digest(
                b"observer code version of the unit test\n",
            )),
            pool_checker: JudgedBy::CpuJudge,
            pool_checker_version,
            conclusion: ConclusionVersion(sha256_digest(
                &[observer.0, pool_checker_version.0].concat(),
            )),
        }
    }

    /// 观察者两段那几条单测一趟的结果：录入段、判段的账，观察者真跑了几次，逐状态的红项、每块的计数向量与在哪判的、每块录入块里的观察结果几份。
    struct ObserverUnitTestRun {
        recording: Option<ObserverRecordingTally>,
        observing: Result<ObservingOfTheRun, String>,
        observer_calls: u64,
        items_of_each_state: Vec<Vec<String>>,
        count_vectors: Vec<BlockCountVector>,
        judged_on_of_each_block: Vec<JudgingDevice>,
        observations_of_each_block: Vec<usize>,
    }

    /// 在 `store` 上照 `stage` 跑那条小流的一个交回镜像的观察者（每看一张镜像加一次调用数，交回崩溃镜像本身、一个写都不发；判段一条不变量都不判，
    /// 判干净时 `judged_clean` 加一）：录入段只录、判段只读库（判段 CPU 那一路）。判段报错照原样交回。
    fn run_the_unit_test_observer(
        store: &mut VerdictStore,
        extra_inputs_digest: [u8; DIGEST_BYTES],
        deduplication: ObserverDeduplication,
        stage: RunStage,
    ) -> ObserverUnitTestRun {
        let recorded = a_flow_of_two_writes_of_the_same_bytes_to_one_sector();
        let calls = std::sync::atomic::AtomicU64::new(0);
        let observe = |_ordinal: u64,
                       image: &CrashImage<'_>,
                       _report: &RecoveryReport,
                       counts: &mut CrashPointReplayObserverCounts|
         -> ImageAfterTheObserver {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            counts.add("images_seen", 1);
            ImageAfterTheObserver {
                writes_on_the_crash_image: Vec::new(),
                disks_after_the_observer: pool_of_the_crash_image(
                    image.base,
                    image.writes,
                    &image.persisted,
                ),
                invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::Only(Vec::new()),
                counter_when_the_pool_checker_is_clean: "judged_clean".to_string(),
            }
        };
        let versions = [versions_of_the_unit_test_observer(extra_inputs_digest)];
        let observer_name = ObserverName::new("unit_test_image_observer").expect("名字合规矩");
        recorded.with_the_flow(|flow| {
            let plan = plan_crash_points(flow);
            let observers = [ObserverOnTheNodes {
                observer_name: observer_name.clone(),
                observer_code_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                extra_inputs_digest,
                nodes: vec![
                    NodeOfAnObserver {
                        setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                        entry_name: OBSERVER_UNIT_TEST_POINT_NAME.to_string(),
                    },
                    NodeOfAnObserver {
                        setting_file: OBSERVER_UNIT_TEST_SETTING_FILE.to_string(),
                        entry_name: END_OF_FLOW_NODE_NAME.to_string(),
                    },
                ],
                observe: ObserverCode::HandsBackAnImage {
                    observe: &observe,
                    deduplication,
                },
            }];
            let recording = stage.records().then(|| {
                record_the_observers_of_the_flow(
                    store,
                    flow,
                    &plan,
                    &observers,
                    &versions,
                    &|_| true,
                    &|| false,
                )
                .expect("录入段写得进库")
            });
            let observer_calls = calls.load(std::sync::atomic::Ordering::SeqCst);
            let observing = if stage.judges() {
                judge_the_observers_of_the_flow(
                    store,
                    &plan,
                    &observers,
                    &versions,
                    &PoolCheckerOfTheImagesAfterTheObservers::OnTheCpu,
                    &|_| true,
                )
            } else {
                Ok(ObservingOfTheRun::default())
            };
            assert_eq!(
                calls.load(std::sync::atomic::Ordering::SeqCst),
                observer_calls,
                "判段不调观察者：只从库里读"
            );
            let kind = ConclusionKind::Observer {
                observer_name: observer_name.clone(),
            };
            let mut run = ObserverUnitTestRun {
                recording,
                observing,
                observer_calls,
                items_of_each_state: Vec::new(),
                count_vectors: Vec::new(),
                judged_on_of_each_block: Vec::new(),
                observations_of_each_block: Vec::new(),
            };
            for point in &plan.points {
                for local in point.blocks() {
                    let key = point.block_key(&local);
                    if let Some(bytes) = store
                        .read_observer_block(&key, &kind, &versions[0].observer)
                        .expect("库读得了")
                    {
                        run.observations_of_each_block.push(
                            ObserverBlockRecord::from_bytes(&bytes)
                                .expect("录入块解得开")
                                .observations
                                .len(),
                        );
                    }
                    let Some(stored) = store
                        .read_conclusion_block(&key, &kind, &versions[0].conclusion)
                        .expect("库读得了")
                    else {
                        continue;
                    };
                    for number in stored.block.verdict_vector_number_of_each_state() {
                        run.items_of_each_state
                            .push(red_items_of_the_encoded_verdict(
                                store
                                    .verdict_vector_of_number(*number)
                                    .expect("块里的判定向量编号库里登过"),
                            ));
                    }
                    run.count_vectors.push(stored.count_vector);
                    run.judged_on_of_each_block.push(stored.judged_on);
                }
            }
            run
        })
    }

    /// 等价关系（用户 2026-10-13 定「按恢复结果去重」；剪枝只剪键相同与等价的）：两个不同崩溃态恢复出逐字节相同的镜像，观察者只看一次、
    /// 录入块里只记一份观察结果；四个状态两张不同的镜像，观察者真跑两次，另两个状态取那一次的结果（录入段「现判 2、去重复用 2」），
    /// 判段每张不同的镜像只判一次，每个状态照样拼进结论、计数照状态数。
    #[test]
    fn crash_states_recovering_to_byte_identical_images_are_observed_and_judged_once() {
        let (_directory, mut store) = an_empty_library_for("observer-identical-images");
        let run = run_the_unit_test_observer(
            &mut store,
            [3; DIGEST_BYTES],
            ObserverDeduplication::ByTheRecoveredImageBytes,
            RunStage::RecordThenJudge,
        );
        let observing = run.observing.expect("判段判得了");
        assert_eq!(
            run.items_of_each_state.len(),
            4,
            "前提：段内三个状态加末节点"
        );
        assert_eq!(
            run.observer_calls, 2,
            "四个状态两张不同的镜像：观察者只看两次"
        );
        let recording = run.recording.expect("录入段录了");
        assert_eq!(
            (
                recording.images_observed_here,
                recording.states_reusing_an_observed_image
            ),
            (2, 2),
            "录入段现判 2、去重复用 2"
        );
        assert_eq!(
            run.observations_of_each_block,
            vec![2, 1],
            "崩溃点那一块三个状态两份观察结果（只落前一个与只落后一个是同一份），末节点那一块一份"
        );
        assert_eq!(
            (
                observing.images_judged_here,
                observing.image_judgements_reused
            ),
            (2, 1),
            "判段每张不同的镜像只判一次：末节点那一块的镜像取前一块判过的"
        );
        let images_seen: u64 = run
            .count_vectors
            .iter()
            .map(|count_vector| {
                count_vector
                    .per_observer
                    .get("images_seen")
                    .copied()
                    .unwrap_or(0)
            })
            .sum();
        assert_eq!(images_seen, 4, "去重之后计数照状态数拼：每个状态都记一次");
        assert_eq!(
            run.judged_on_of_each_block,
            vec![JudgingDevice::Cpu, JudgingDevice::Cpu],
            "配置关 GPU：结论记的在哪判的是 CPU（实际判盘的那一路）"
        );
    }

    /// 另读的输入不同的两个观察者（名字、代码相同）看同一张镜像不合并：后一个照样逐张镜像看一遍，不取前一个的结果；同一份另读的输入再跑，
    /// 录入块都在库里，一张都不再看。
    #[test]
    fn observers_reading_different_extra_inputs_do_not_share_their_observed_images() {
        let (_directory, mut store) = an_empty_library_for("observer-extra-inputs");
        let first = run_the_unit_test_observer(
            &mut store,
            [3; DIGEST_BYTES],
            ObserverDeduplication::ByTheRecoveredImageBytes,
            RunStage::RecordThenJudge,
        );
        let second = run_the_unit_test_observer(
            &mut store,
            [4; DIGEST_BYTES],
            ObserverDeduplication::ByTheRecoveredImageBytes,
            RunStage::RecordThenJudge,
        );
        let again = run_the_unit_test_observer(
            &mut store,
            [4; DIGEST_BYTES],
            ObserverDeduplication::ByTheRecoveredImageBytes,
            RunStage::RecordThenJudge,
        );
        assert_eq!(
            (
                first.observer_calls,
                second.observer_calls,
                again.observer_calls
            ),
            (2, 2, 0),
            "另读的输入不同不合并；相同的再跑全复用"
        );
    }

    /// 剪枝自证（`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」）：小输入上关掉去重跑一遍，
    /// 逐状态的红项与每块的计数向量与开着时逐项相同；关掉时观察者每个状态都看一遍。
    #[test]
    fn turning_the_observer_deduplication_off_judges_every_state_as_with_it_on() {
        let (_on_directory, mut store_with_it_on) = an_empty_library_for("observer-dedup-on");
        let (_off_directory, mut store_with_it_off) = an_empty_library_for("observer-dedup-off");
        let with_it_on = run_the_unit_test_observer(
            &mut store_with_it_on,
            [3; DIGEST_BYTES],
            ObserverDeduplication::ByTheRecoveredImageBytes,
            RunStage::RecordThenJudge,
        );
        let with_it_off = run_the_unit_test_observer(
            &mut store_with_it_off,
            [3; DIGEST_BYTES],
            ObserverDeduplication::OffForTheSelfProof,
            RunStage::RecordThenJudge,
        );
        assert_eq!(
            (with_it_on.observer_calls, with_it_off.observer_calls),
            (2, 4),
            "开着去重看两张镜像，关掉每个状态都看"
        );
        assert_eq!(
            with_it_on.items_of_each_state,
            with_it_off.items_of_each_state
        );
        assert_eq!(with_it_on.count_vectors, with_it_off.count_vectors);
    }

    /// 观察者两段：只录那一趟库里只有录入块、没有观察者的结论；没录过就只判，判段报「先跑录入段」；录过之后只判那一趟从库里拼出结论、不调观察者。
    #[test]
    fn the_observer_record_stage_does_not_judge_and_the_judge_stage_only_reads_the_library() {
        let (_directory, mut store) = an_empty_library_for("observer-two-stages");
        let judged_before_recording = run_the_unit_test_observer(
            &mut store,
            [3; DIGEST_BYTES],
            ObserverDeduplication::ByTheRecoveredImageBytes,
            RunStage::JudgeOnly,
        );
        assert!(
            judged_before_recording
                .observing
                .as_ref()
                .is_err_and(|refusal| refusal.contains("先跑录入段")),
            "没录过就只判：{:?}",
            judged_before_recording.observing
        );
        let recorded_only = run_the_unit_test_observer(
            &mut store,
            [3; DIGEST_BYTES],
            ObserverDeduplication::ByTheRecoveredImageBytes,
            RunStage::RecordOnly,
        );
        assert_eq!(recorded_only.observer_calls, 2, "录入段看了两张镜像");
        assert!(
            recorded_only.items_of_each_state.is_empty(),
            "只录那一趟一块结论都不存"
        );
        assert_eq!(recorded_only.observations_of_each_block, vec![2, 1]);
        let judged_only = run_the_unit_test_observer(
            &mut store,
            [3; DIGEST_BYTES],
            ObserverDeduplication::ByTheRecoveredImageBytes,
            RunStage::JudgeOnly,
        );
        assert!(judged_only.observing.is_ok(), "{:?}", judged_only.observing);
        assert_eq!(judged_only.observer_calls, 0, "只判那一趟不调观察者");
        assert_eq!(
            judged_only.items_of_each_state.len(),
            4,
            "只判那一趟从库里拼出每个状态的结论"
        );
    }

    /// 用户 2026-10-13 定「配 GPU 必用 GPU」：配置开了 GPU 而这份二进制没编进 gpu 特性时判段报本次未跑（两行：入口认的那一行与门禁快档认的
    /// 那一族记号），不退回 CPU 判；配置没开 GPU、或编进了 gpu 特性，判段照跑。
    #[test]
    fn the_gpu_switched_on_without_the_gpu_feature_reports_the_judge_not_run() {
        let reason = judge_not_run_without_the_gpu_feature(true, false)
            .expect("配置开了 GPU、没编进特性：判段报本次未跑");
        assert!(
            reason.contains("ENABLE_GPU=1") && reason.contains("gpu 特性"),
            "{reason}"
        );
        assert_eq!(judge_not_run_without_the_gpu_feature(false, false), None);
        assert_eq!(judge_not_run_without_the_gpu_feature(true, true), None);
        assert_eq!(judge_not_run_without_the_gpu_feature(false, true), None);
        let lines = not_run_lines_without_the_gpu_feature("一条流", &reason);
        assert_eq!(lines.len(), 2);
        assert!(
            lines[0].starts_with(
                "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=一条流 reason="
            ),
            "{}",
            lines[0]
        );
        assert!(
            lines[1].starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason="
            ),
            "{}",
            lines[1]
        );
    }

    /// 给定状态引擎认选段（用户 2026-10-13 定）：`record` 那一趟只把表写进库、交占位结论（一张不判）；`judge` 那一趟只凭项名与表名从库里读回判——
    /// 调用方这一趟交来的表与录入段那一张状态数不同，判的是库里那一张；两段先后跑的那一趟照库里判过的复用。
    #[test]
    fn the_given_states_engine_records_only_in_the_record_stage_and_judges_the_table_read_back_in_the_judge_stage(
    ) {
        let directory =
            crate::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start(
                "given-states-engine-stages",
            )
            .expect("流水线单元测试那一类里这条用例的目录");
        let configuration = directory.join("multi-host-on-the-cpu.env");
        std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了配置");
        let library = directory.join("given-states-library");
        let device = singlefs_core::address::DeviceIdentity(0);
        let base = MemoryPool::with_devices(&[device], 1 << 20);
        let writes = vec![RetainedWrite {
            device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(8192),
            contents: WrittenContents::Bytes(vec![7; 512]),
        }];
        let segments = vec![vec![0]];
        let run_in_the_stage = |stage: Option<&str>, persisted_sets: Vec<Vec<bool>>| {
            let mut settings: Vec<(&str, &std::ffi::OsStr)> = vec![
                ("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str()),
                (
                    CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
                    library.as_os_str(),
                ),
            ];
            if let Some(stage) = stage {
                settings.push((RUN_STAGE_ENVIRONMENT_VARIABLE, std::ffi::OsStr::new(stage)));
            }
            let _environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
            let engine = GivenStatesEngine::start_from_environment(
                GivenStatesItem {
                    item_name: "given-states-engine-stages".to_string(),
                    setting_file: "crates/singlefs-checker-tier/src/crash_injection.rs".to_string(),
                    seeds_and_scale: "unit-test".to_string(),
                },
                &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
                &crate::crash_library::repository_root_of_this_crate(),
            )
            .expect("起得了给定状态表的引擎");
            let conclusions = engine
                .judge_the_given_states(&GivenStatesOfATable {
                    table_name: "two-states".to_string(),
                    base: &base,
                    writes: &writes,
                    segments: &segments,
                    seams: &[],
                    persisted_sets,
                })
                .expect("判得了这张表");
            (conclusions, engine.finish())
        };
        let (recorded, record_run) =
            run_in_the_stage(Some("record"), vec![vec![true], vec![false]]);
        assert_eq!(
            recorded.source,
            GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly
        );
        assert!(record_run.recorded_only());
        assert_eq!(
            (
                record_run.tally.tables_recorded_here,
                record_run.tally.tables_recorded_only,
                record_run.tally.tables_judged_here
            ),
            (1, 1, 0),
            "只录那一趟：表写进库，一张不判"
        );
        let (judged, judge_run) = run_in_the_stage(Some("judge"), vec![vec![true]]);
        assert_eq!(
            judged.states.len(),
            2,
            "只判那一趟判的是库里录的那一张（两个状态），不是调用方这一趟交来的（一个状态）"
        );
        assert_eq!(
            (
                judge_run.tally.tables_recorded_here,
                judge_run.tally.tables_judged_here,
                judge_run.tally.tables_reused
            ),
            (0, 1, 0),
            "只判那一趟不录、判一张"
        );
        let (_both, both_run) = run_in_the_stage(None, vec![vec![true], vec![false]]);
        assert_eq!(
            (
                both_run.tally.tables_recorded_here,
                both_run.tally.tables_reused
            ),
            (1, 1),
            "两段先后跑：录一遍、照库里判过的复用"
        );
    }

    /// 观察者交回的盘建判表那几条单测的基底：两块 4 GiB 的稀疏内存盘上做完 mkfs（判表从基底第一块盘的系统配置槽取落点几何）。
    fn a_formatted_pool_for_the_judge_tables_of_the_observer_images() -> MemoryPool {
        const DEVICE_BYTES: u64 = 4 << 30;
        let parameters = singlefs_harness::scenario::e142_parameters(512, 512);
        let mut devices: Vec<(
            singlefs_core::address::DeviceIdentity,
            singlefs_harness::memory_pool::SparseBlockDevice,
        )> = [0u32, 1]
            .into_iter()
            .map(|device_number| {
                (
                    singlefs_core::address::DeviceIdentity(device_number),
                    singlefs_harness::memory_pool::SparseBlockDevice::new(
                        DEVICE_BYTES,
                        singlefs_core::block_device::PhysicalBlockSizeInBytes(512),
                    ),
                )
            })
            .collect();
        singlefs_core::make_filesystem::make_filesystem(&parameters, &mut devices)
            .expect("两块空的内存盘上做得了 mkfs");
        MemoryPool {
            devices: devices
                .into_iter()
                .map(|(identity, device)| (identity, device.image))
                .collect(),
            device_size_in_bytes: DEVICE_BYTES,
        }
    }

    /// 写在一块盘系统配置槽 0 号（偏移 0）上的一次 4096 字节的写，内容是 `byte` 重复。
    fn a_system_configuration_slot_write(device_number: u32, byte: u8) -> RetainedWrite {
        RetainedWrite {
            device: singlefs_core::address::DeviceIdentity(device_number),
            kind: StepKind::SystemConfigurationSlot,
            is_force_unit_access: true,
            offset: singlefs_core::address::DeviceOffsetInBytes(0),
            contents: WrittenContents::Bytes(vec![byte; 4096]),
        }
    }

    /// 挂载把同一个位置（盘 0 的系统配置槽 0 号）写了超过判表一个落点的写数上限那么多次：基底枚举写表在那里写了上限加 6 次，一张盘全持久、
    /// 观察者又在那里写 3 次（不并是上限加 9 次，越过 [`crate::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION`]，单张盘也建不出判表）；另 40 张盘
    /// 各持久前 1 到 40 次，观察者都只发同一次写（盘 1 的系统配置槽，逐字节相同）。并过同一位置之后每张盘都落在某一张建得出的判表里、恰好一次；
    /// 交回的写相同的盘共用一份写：41 张盘只排成几张表，每张表里那一次共用的写只有一份；每张盘照判表的掩码按判表写表的次序叠在基底上，
    /// 与它自己的次序叠出的盘（[`pool_after_the_observer`]）逐字节相同。
    /// 判别力（`crates/mutations.tsv`）：每张盘同一位置的写不并，全持久的那一张建不出判表，这条红；交回的写相同的盘不共用（一张盘一张表），这条红。
    #[test]
    fn observer_images_writing_one_location_past_the_judge_table_limit_are_merged_and_share_their_writes(
    ) {
        use crate::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION;
        const ENUMERATED_REWRITES_OF_THE_SLOT: usize = MAXIMUM_WRITES_PER_LOCATION + 6;
        const PARTLY_PERSISTED_IMAGES: usize = 40;
        let base = a_formatted_pool_for_the_judge_tables_of_the_observer_images();
        let enumerated_writes: Vec<RetainedWrite> = (0..ENUMERATED_REWRITES_OF_THE_SLOT)
            .map(|rewrite| {
                a_system_configuration_slot_write(
                    0,
                    u8::try_from(rewrite + 1).expect("几十次装得进 u8"),
                )
            })
            .collect();
        let foundation = ObserverFoundation {
            base,
            enumerated_writes,
        };
        let foundation_digest = sha256_digest(b"foundation of the unit test");
        let shared_write = a_system_configuration_slot_write(1, 0xa5);
        let image_to_judge =
            |persisted_prefix: usize, writes_on_the_crash_image: Vec<RetainedWrite>| {
                ImageToJudgeAfterTheObserver {
                    foundation_digest,
                    persisted: (0..ENUMERATED_REWRITES_OF_THE_SLOT)
                        .map(|write_index| write_index < persisted_prefix)
                        .collect(),
                    writes_on_the_crash_image,
                    invariants_judged:
                        InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
                    counter_when_the_pool_checker_is_clean: "clean".to_string(),
                }
            };
        let mut images_to_judge = vec![image_to_judge(
            ENUMERATED_REWRITES_OF_THE_SLOT,
            vec![
                a_system_configuration_slot_write(0, 201),
                a_system_configuration_slot_write(0, 202),
                a_system_configuration_slot_write(0, 203),
                shared_write.clone(),
            ],
        )];
        for persisted_prefix in 1..=PARTLY_PERSISTED_IMAGES {
            images_to_judge.push(image_to_judge(persisted_prefix, vec![shared_write.clone()]));
        }
        let images: Vec<ImageAfterTheObserverToJudge<'_>> = images_to_judge
            .iter()
            .enumerate()
            .map(|(position, image)| ImageAfterTheObserverToJudge {
                deduplication_key: sha256_digest(&position.to_le_bytes()),
                foundation: &foundation,
                image,
            })
            .collect();
        let mut planned: Vec<PlannedJudgeTableOfImagesAfterTheObservers> = Vec::new();
        plan_the_judge_tables_of_the_images_after_the_observers(&images, &mut |table| {
            planned.push(table);
            Ok(())
        })
        .expect("收下每一张的那一步不报错");
        let mut tables_built = 0usize;
        let mut times_each_image_is_planned = vec![0usize; images.len()];
        for table in &planned {
            let built = match table {
                PlannedJudgeTableOfImagesAfterTheObservers::Built(built) => built,
                PlannedJudgeTableOfImagesAfterTheObservers::NotBuildableForOneImage {
                    position,
                    reason,
                } => panic!("第 {position} 张盘并过同一位置上的写之后要建得出判表：{reason}"),
            };
            tables_built += 1;
            assert_eq!(
                built
                    .writes
                    .iter()
                    .filter(|write| **write == shared_write)
                    .count(),
                1,
                "交回的写逐字节相同的几张盘在一张表里共用一份写"
            );
            let persisted_sets = match &built.tables.state_source {
                crate::crash_judge_tables::JudgeStateSource::GivenPersistedSets {
                    persisted_sets,
                    claimed_versions: _,
                } => persisted_sets,
                crate::crash_judge_tables::JudgeStateSource::SegmentExpansions => {
                    panic!("观察者交回的盘建的是给定状态的判表")
                }
            };
            assert_eq!(persisted_sets.len(), built.positions.len());
            assert_eq!(
                built.tables.state_count,
                u64::try_from(built.positions.len()).expect("张数装得进 u64"),
                "判表一张盘一个状态"
            );
            for (position, persisted) in built.positions.iter().zip(persisted_sets) {
                times_each_image_is_planned[*position] += 1;
                assert!(
                    persisted[built.writes.len()..]
                        .iter()
                        .all(|is_persisted| !*is_persisted),
                    "建表接上的撕裂镜像不在任何一张盘的掩码里"
                );
                let mut pool = foundation.base.clone();
                for (write, is_persisted) in built.writes.iter().zip(persisted) {
                    if *is_persisted {
                        pool.apply_writes(std::slice::from_ref(write));
                    }
                }
                assert!(
                    pools_hold_the_same_bytes(&pool, &pool_after_the_observer(&images[*position])),
                    "第 {position} 张盘照判表的掩码叠出来与它自己的次序叠出来逐字节相同"
                );
            }
        }
        assert_eq!(
            times_each_image_is_planned,
            vec![1; images.len()],
            "每张盘恰好落在一张判表里"
        );
        assert!(
            tables_built <= 4,
            "{} 张同一基底、交回的写相同的盘只排成几张表，排了 {tables_built} 张",
            images.len()
        );
    }

    /// 观察者交回的盘上「GPU 判不了」不是判器的结论（建表那一截在这份文件里、不在判器那一把版本里）：判段这一批判过的盘里只存判器判出来的那几张，
    /// 带着一张判不了的那一块不存进库，都判干净的那一块存。判别力（`crates/mutations.tsv`）：判不了的也当结论存，这条红。
    #[test]
    fn an_image_the_gpu_could_not_judge_is_not_stored_and_its_block_is_not_kept() {
        let foundation = ObserverFoundation {
            base: MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(0)], 1 << 20),
            enumerated_writes: Vec::new(),
        };
        let image = ImageToJudgeAfterTheObserver {
            foundation_digest: [0; DIGEST_BYTES],
            persisted: Vec::new(),
            writes_on_the_crash_image: Vec::new(),
            invariants_judged:
                InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
            counter_when_the_pool_checker_is_clean: "clean".to_string(),
        };
        let images = [
            ImageAfterTheObserverToJudge {
                deduplication_key: [1; DIGEST_BYTES],
                foundation: &foundation,
                image: &image,
            },
            ImageAfterTheObserverToJudge {
                deduplication_key: [2; DIGEST_BYTES],
                foundation: &foundation,
                image: &image,
            },
        ];
        let judged_clean = ImageJudgementRecord {
            violated_invariants: Vec::new(),
            could_not_judge: None,
            judged_on: JudgingDevice::Gpu {
                card_type: "单测".to_string(),
                driver_version: String::new(),
            },
        };
        let could_not_judge = ImageJudgementRecord {
            violated_invariants: Vec::new(),
            could_not_judge: Some("建不出 GPU 判表：单测".to_string()),
            judged_on: JudgingDevice::Cpu,
        };
        let stored =
            image_judgements_to_store(&images, &[judged_clean.clone(), could_not_judge.clone()]);
        assert_eq!(
            stored
                .iter()
                .map(|(deduplication_key, _)| *deduplication_key)
                .collect::<Vec<_>>(),
            vec![[1; DIGEST_BYTES]],
            "只存判器判出来的那一张"
        );
        assert!(!block_conclusion_is_kept_in_the_library(&[
            Some(judged_clean.clone()),
            Some(could_not_judge),
            None
        ]));
        assert!(block_conclusion_is_kept_in_the_library(&[
            Some(judged_clean),
            None
        ]));
    }

    /// 同一个表名下写表不同的两张给定状态表（甲报告问题 5）：录入段各录一张（各登各的清单），只判那一趟按调用方这一趟交来的那张表找到
    /// 录入段录的那一张、各判各的——一张一个状态、一张两个状态，判出来的状态数各是各的。判别力（`crates/mutations.tsv`）：清单名只按表名起，
    /// 只判那一趟两张都读到后录的那一张，这条红。
    #[test]
    fn two_given_states_tables_of_one_name_and_different_writes_are_each_judged_in_the_judge_stage()
    {
        let directory =
            crate::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start(
                "given-states-tables-of-one-name",
            )
            .expect("流水线单元测试那一类里这条用例的目录");
        let configuration = directory.join("multi-host-on-the-cpu.env");
        std::fs::write(&configuration, "ENABLE_GPU=0\n").expect("写得了配置");
        let library = directory.join("given-states-library");
        let device = singlefs_core::address::DeviceIdentity(0);
        let base = MemoryPool::with_devices(&[device], 1 << 20);
        let writes_with_the_byte = |byte: u8| {
            vec![RetainedWrite {
                device,
                kind: StepKind::UnitWrite,
                is_force_unit_access: false,
                offset: singlefs_core::address::DeviceOffsetInBytes(8192),
                contents: WrittenContents::Bytes(vec![byte; 512]),
            }]
        };
        let writes_of_the_one_state_table = writes_with_the_byte(7);
        let writes_of_the_two_state_table = writes_with_the_byte(9);
        let segments = vec![vec![0]];
        let one_state_table = || GivenStatesOfATable {
            table_name: "one-name".to_string(),
            base: &base,
            writes: &writes_of_the_one_state_table,
            segments: &segments,
            seams: &[],
            persisted_sets: vec![vec![true]],
        };
        let two_state_table = || GivenStatesOfATable {
            table_name: "one-name".to_string(),
            base: &base,
            writes: &writes_of_the_two_state_table,
            segments: &segments,
            seams: &[],
            persisted_sets: vec![vec![true], vec![false]],
        };
        let run_in_the_stage = |stage: &str| {
            let _environment = CrashAmplificationEnvironment::cleared_then_set(&[
                ("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str()),
                (
                    CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
                    library.as_os_str(),
                ),
                (RUN_STAGE_ENVIRONMENT_VARIABLE, std::ffi::OsStr::new(stage)),
            ]);
            let engine = GivenStatesEngine::start_from_environment(
                GivenStatesItem {
                    item_name: "given-states-tables-of-one-name".to_string(),
                    setting_file: "crates/singlefs-checker-tier/src/crash_injection.rs".to_string(),
                    seeds_and_scale: "unit-test".to_string(),
                },
                &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
                &crate::crash_library::repository_root_of_this_crate(),
            )
            .expect("起得了给定状态表的引擎");
            let state_counts = (
                engine
                    .judge_the_given_states(&one_state_table())
                    .expect("判得了一个状态的那张表")
                    .states
                    .len(),
                engine
                    .judge_the_given_states(&two_state_table())
                    .expect("判得了两个状态的那张表")
                    .states
                    .len(),
            );
            (state_counts, engine.finish())
        };
        let (_, record_run) = run_in_the_stage("record");
        assert_eq!(
            record_run.tally.tables_recorded_here, 2,
            "录入段同名的两张表各录一张"
        );
        let (judged_state_counts, judge_run) = run_in_the_stage("judge");
        assert_eq!(
            judged_state_counts,
            (1, 2),
            "只判那一趟同名的两张表各判各的：一张一个状态、一张两个状态"
        );
        assert_eq!(judge_run.tally.tables_judged_here, 2);
    }

    /// 用户 2026-10-13 定「配 GPU 必用 GPU、做不到报错不降级」：配置开了 GPU 而这份二进制没编进 gpu 特性（提交时快档不带特性那一趟），给定状态表的
    /// 引擎录入段照录（表写进库），判段不跑、交占位结论（一张不判，不退回 CPU 判），这一趟算只录（调用方照本次未跑交回）；开跑时打的本次未跑那一行
    /// 是门禁 checker-tier-crash-point-replay 的 `test_not_run_lines` 认的那一族记号、带着项名，不是 `GIVEN_STATES_JUDGE route=cpu`。
    /// 判别力（`crates/mutations.tsv`）：判段照 CPU 判下去，这条红。
    #[cfg(not(feature = "gpu"))]
    #[test]
    fn the_given_states_engine_without_the_gpu_feature_records_the_table_and_reports_the_judge_not_run(
    ) {
        let directory =
            crate::crash_library::amplification_unit_tests_library_directory_emptied_at_the_start(
                "given-states-engine-without-the-gpu-feature",
            )
            .expect("流水线单元测试那一类里这条用例的目录");
        let configuration = directory.join("multi-host-with-the-gpu.env");
        std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了配置");
        let library = directory.join("given-states-library");
        let device = singlefs_core::address::DeviceIdentity(0);
        let base = MemoryPool::with_devices(&[device], 1 << 20);
        let writes = vec![RetainedWrite {
            device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: singlefs_core::address::DeviceOffsetInBytes(8192),
            contents: WrittenContents::Bytes(vec![7; 512]),
        }];
        let segments = vec![vec![0]];
        let environment = CrashAmplificationEnvironment::cleared_then_set(&[
            ("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str()),
            (
                CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
                library.as_os_str(),
            ),
            // 配置只写开关：清场之下（变异门禁有一行要卡就整趟在清场包装之下）引擎起步读显卡先后，开了 GPU 而没写清场之后那一份就拒；
            // 显卡先后经环境变量交（空串：本机能用的卡全用）。没编进 gpu 特性，判段不跑、一张卡都不用，交哪一份都一样
            (
                crate::gpu_card_configuration::CARDS_BY_PRIORITY_KEY,
                std::ffi::OsStr::new(""),
            ),
        ]);
        let engine = GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "given-states-engine-without-the-gpu-feature".to_string(),
                setting_file: "crates/singlefs-checker-tier/src/crash_injection.rs".to_string(),
                seeds_and_scale: "unit-test".to_string(),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
            &crate::crash_library::repository_root_of_this_crate(),
        )
        .expect("配置开了 GPU、没编进特性：引擎照样起得了（判段不跑，不拒跑）");
        let conclusions = engine
            .judge_the_given_states(&GivenStatesOfATable {
                table_name: "one-state".to_string(),
                base: &base,
                writes: &writes,
                segments: &segments,
                seams: &[],
                persisted_sets: vec![vec![true]],
            })
            .expect("判段不跑也照常交回");
        let run = engine.finish();
        drop(environment);
        assert_eq!(
            conclusions.source,
            GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly,
            "交的是占位结论"
        );
        assert!(
            run.not_run_without_the_gpu_feature
                .as_ref()
                .is_some_and(|reason| reason.contains("ENABLE_GPU=1")),
            "{:?}",
            run.not_run_without_the_gpu_feature
        );
        assert!(run.recorded_only(), "这一趟算只录，调用方照本次未跑交回");
        assert_eq!(
            (
                run.tally.tables_recorded_here,
                run.tally.tables_not_judged_without_the_gpu_feature,
                run.tally.tables_judged_here,
                run.tally.states_judged_by_the_cpu_here,
                run.tally.states_judged_by_the_gpu_here
            ),
            (1, 1, 0, 0, 0),
            "表照录、一张不判（不退回 CPU 判）"
        );
        let line = given_states_not_run_line_without_the_gpu_feature(
            "given-states-engine-without-the-gpu-feature",
            run.not_run_without_the_gpu_feature
                .as_deref()
                .expect("上面判过有原因"),
        );
        assert!(
            line.starts_with(
                "NOT_RUN_NO_GPU_ADAPTER test=given_states_engine::given-states-engine-without-the-gpu-feature reason="
            ) && !line.contains("route=cpu"),
            "{line}"
        );
    }

    /// J8：版本对不上的三种结局打出来的那一行都以入口认的那个头起（入口照它退 78），带着是哪一种。
    #[test]
    fn every_conclusion_version_not_written_line_starts_with_the_marker_the_entry_reads() {
        for (not_writable, reason) in [
            (
                ConclusionVersionsNotWritable::TheTableDiffersFromTheBuild {
                    entries: vec!["GpuJudge".to_string()],
                },
                "reason=table_differs",
            ),
            (
                ConclusionVersionsNotWritable::AVersionDiffersFromTheBuild {
                    request: "CpuJudge".to_string(),
                    check: "DiffersFromTheBuild".to_string(),
                },
                "reason=version_differs",
            ),
            (
                ConclusionVersionsNotWritable::NotComputableFromTheTree {
                    reason: "读不了".to_string(),
                },
                "reason=not_computable",
            ),
        ] {
            let line = not_writable.not_written_line();
            assert!(
                line.starts_with(&format!(
                    "{CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX} {reason}"
                )),
                "{line}"
            );
        }
    }

    /// 两段的录入段写进库的一条小流程：一块盘写过两个扇区，三次写（普通写、FUA 写、整段清零），两段（一段展开、一段不展开），
    /// 一版带文件、一版没有文件，两个点（一个 ignore）。
    fn a_small_recorded_flow() -> RecordedFlow {
        let mut base =
            MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(3)], 1 << 20);
        let device = base
            .devices
            .get_mut(&singlefs_core::address::DeviceIdentity(3))
            .expect("刚建的盘");
        device.write(singlefs_core::address::DeviceOffsetInBytes(0), &[7u8; 512]);
        device.write(
            singlefs_core::address::DeviceOffsetInBytes(4096),
            &[9u8; 512],
        );
        RecordedFlow {
            base,
            writes: vec![
                RetainedWrite {
                    device: singlefs_core::address::DeviceIdentity(3),
                    kind: StepKind::UnitWrite,
                    is_force_unit_access: false,
                    offset: singlefs_core::address::DeviceOffsetInBytes(8192),
                    contents: WrittenContents::Bytes(vec![1, 2, 3, 4]),
                },
                RetainedWrite {
                    device: singlefs_core::address::DeviceIdentity(3),
                    kind: StepKind::RootRecordFua,
                    is_force_unit_access: true,
                    offset: singlefs_core::address::DeviceOffsetInBytes(512),
                    contents: WrittenContents::Bytes(vec![5; 512]),
                },
                RetainedWrite {
                    device: singlefs_core::address::DeviceIdentity(3),
                    kind: StepKind::ZeroFill,
                    is_force_unit_access: false,
                    offset: singlefs_core::address::DeviceOffsetInBytes(16384),
                    contents: WrittenContents::Zeros { length: 4096 },
                },
            ],
            segments: vec![vec![0, 1], vec![2]],
            judged_root_index: 1,
            versions: vec![PublishedVersion {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(5),
                content: b"content".to_vec(),
            }],
            versions_without_a_file: vec![PublishedVersionWithoutAFile {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(6),
            }],
            segment_expansions: vec![
                CrashPointReplaySegmentExpansion::EveryProperSubset,
                CrashPointReplaySegmentExpansion::NotExpanded,
            ],
            crash_points: vec![
                CrashPointSpan {
                    name: "写入".to_string(),
                    code_file_in_repository: "crates/a.rs".to_string(),
                    writes: 0..2,
                    ignored: false,
                },
                CrashPointSpan {
                    name: "清零".to_string(),
                    code_file_in_repository: "crates/b.rs".to_string(),
                    writes: 2..3,
                    ignored: true,
                },
            ],
            // 第 0 段（两次写全展开，3 个态）只判段内编号 [1, 3)：片起点与片长随流一起往返（C790）
            segment_slices: vec![
                Some(CrashPointReplaySegmentSlice {
                    start_within_the_domain: StateCount1280Bits::from_u64(1),
                    state_count: 2,
                }),
                None,
            ],
        }
    }

    /// 录入段写进库的流程读回来与原样逐样相同：盘上写过的扇区、每次写的盘、种类、FUA、偏移与内容（含整段清零）、分段、被判根、
    /// 两种版本、每段的展开方式、崩溃点（含 ignore）、每段的片（C790）；读回的再写一遍逐字节相同。
    #[test]
    fn a_recorded_flow_reads_back_from_its_bytes_exactly_as_it_was_recorded() {
        let recorded = a_small_recorded_flow();
        let bytes = recorded.to_bytes();
        let read_back = RecordedFlow::from_bytes(&bytes).expect("刚写的字节读得回");
        assert_eq!(read_back, recorded, "读回的与录下的逐样相同");
        assert_eq!(read_back.to_bytes(), bytes, "读回的再写一遍逐字节相同");
        assert_eq!(read_back.digest(), recorded.digest());
    }

    /// 库里那份字节坏了读不回：换了头、截短一个字节、多出一个字节、写的种类码认不得，都交回 `None`（判段照「读不回」停下，不猜）。
    #[test]
    fn a_damaged_recorded_flow_does_not_read_back() {
        let bytes = a_small_recorded_flow().to_bytes();
        let mut other_magic = bytes.clone();
        other_magic[0] ^= 1;
        assert_eq!(RecordedFlow::from_bytes(&other_magic), None, "换了头");
        assert_eq!(
            RecordedFlow::from_bytes(&bytes[..bytes.len() - 1]),
            None,
            "截短一个字节"
        );
        let mut longer = bytes.clone();
        longer.push(0);
        assert_eq!(RecordedFlow::from_bytes(&longer), None, "多出一个字节");
        let mut recorded = a_small_recorded_flow();
        recorded.writes.truncate(1);
        let one_write = recorded.to_bytes();
        // 第一次写的种类码在：头 8、盘大小 8、盘数 8、盘号 4、扇区数 8、两个扇区（各 8 + 8 + 512）、写数 8、盘号 4 之后
        let kind_at = 8 + 8 + 8 + 4 + 8 + 2 * (8 + 8 + 512) + 8 + 4;
        assert_eq!(one_write[kind_at], 1, "那一格是单元写的码");
        let mut unknown_kind = one_write.clone();
        unknown_kind[kind_at] = 0xEE;
        assert_eq!(
            RecordedFlow::from_bytes(&unknown_kind),
            None,
            "种类码认不得"
        );
    }

    /// 录入段登的清单读回来与原样相同：两个摘要、配了 GPU 时判表的摘要（与没配时的 `None`）、全域判定、登记了的段。
    #[test]
    fn a_recorded_run_manifest_reads_back_from_its_bytes() {
        for packed_judge_tables_digest in [None, Some([3u8; DIGEST_BYTES])] {
            let manifest = RecordedRunManifest {
                recorded_flow_digest: [1u8; DIGEST_BYTES],
                facts_digest: [2u8; DIGEST_BYTES],
                packed_judge_tables_digest,
                registered_segments: vec![RegisteredSegment {
                    segment_index: 7,
                    write_count: 26,
                    write_table_digest_text: "0f".repeat(DIGEST_BYTES),
                }],
                exhaustive: packed_judge_tables_digest.is_some(),
            };
            let bytes = manifest.to_bytes();
            assert_eq!(
                RecordedRunManifest::from_bytes(&bytes),
                Some(manifest.clone())
            );
            assert_eq!(
                RecordedRunManifest::from_bytes(&bytes[..bytes.len() - 1]),
                None,
                "截短一个字节"
            );
        }
    }

    /// 选段：不设两段先后跑，`record` 只跑录入段，`judge` 只跑判段；写别的值整趟拒、错误里点名变量与原值。
    #[test]
    fn the_run_stage_is_both_stages_unless_record_or_judge_is_named() {
        assert_eq!(
            RunStage::of_the_variable_value(None),
            Ok(RunStage::RecordThenJudge)
        );
        assert_eq!(
            RunStage::of_the_variable_value(Some("record")),
            Ok(RunStage::RecordOnly)
        );
        assert_eq!(
            RunStage::of_the_variable_value(Some("judge")),
            Ok(RunStage::JudgeOnly)
        );
        let refused = RunStage::of_the_variable_value(Some("both")).expect_err("写错的值拒");
        assert!(
            refused.contains(RUN_STAGE_ENVIRONMENT_VARIABLE) && refused.contains("both"),
            "{refused}"
        );
        assert!(RunStage::RecordOnly.records() && !RunStage::RecordOnly.judges());
        assert!(!RunStage::JudgeOnly.records() && RunStage::JudgeOnly.judges());
        assert!(RunStage::RecordThenJudge.records() && RunStage::RecordThenJudge.judges());
    }

    /// 判段只认库：录入段写进库之后把库里那份流程换成另一份（清单的摘要不改），判段读回时摘要对不上，整趟停下报「库里那份坏了」；
    /// 清单不在（录入段没跑）也停下，不去用调用方内存里的流程。
    #[test]
    fn the_judge_stage_reads_only_the_library_and_stops_when_the_recorded_flow_there_was_changed() {
        let directory = std::env::temp_dir().join(format!(
            "singlefs-two-stage-read-back-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&directory);
        let mut store = VerdictStore::create_empty(&directory).expect("建得了库");
        let missing = read_back_the_recorded_run(&store, "没录过的流", false)
            .err()
            .expect("没录过的流读不回");
        assert!(
            missing.contains("没录过的流") && missing.contains("先跑录入段"),
            "{missing}"
        );
        let recorded = a_small_recorded_flow();
        let bytes = recorded.to_bytes();
        let digest = sha256_digest(&bytes);
        store.store_recorded_flow(&digest, &bytes).expect("写得进");
        let facts = recorded.with_the_flow(|flow| {
            crate::crash_facts::extract_flow_facts(
                flow.base,
                flow.writes,
                flow.segments,
                flow.expansion,
            )
        });
        store
            .store_flow_facts(&facts.digest(), &facts.to_bytes())
            .expect("写得进");
        let manifest = RecordedRunManifest {
            recorded_flow_digest: digest,
            facts_digest: facts.digest(),
            packed_judge_tables_digest: None,
            registered_segments: Vec::new(),
            exhaustive: true,
        };
        store
            .store_recorded_run_manifest("一条流", &manifest.to_bytes())
            .expect("写得进");
        let read_back = read_back_the_recorded_run(&store, "一条流", false).expect("读得回");
        assert_eq!(
            read_back.flow.recorded(),
            &recorded,
            "读回的就是录下的那一份"
        );
        assert_eq!(*read_back.facts, facts);
        let mut changed = recorded.clone();
        changed.versions[0].content = b"another".to_vec();
        store
            .store_recorded_flow(&digest, &changed.to_bytes())
            .expect("写得进");
        let refused = read_back_the_recorded_run(&store, "一条流", false)
            .err()
            .expect("库里那份换了，摘要对不上");
        assert!(refused.contains("摘要与清单对不上"), "{refused}");
        let refused_for_the_gpu = {
            store.store_recorded_flow(&digest, &bytes).expect("写得进");
            read_back_the_recorded_run(&store, "一条流", true)
                .err()
                .expect("要 GPU 判而清单里没有判表")
        };
        assert!(
            refused_for_the_gpu.contains("没有 GPU 判表"),
            "{refused_for_the_gpu}"
        );
        drop(store);
        std::fs::remove_dir_all(&directory).expect("删得掉临时库");
    }

    /// 给定状态那一路写进库的一张表读回来与原样逐样相同：名字、基础镜像、写表、切段、两种接缝（掉电之后、崩溃之后带与不带落到的那一版）、
    /// 每个状态的持久集合；读回的再写一遍逐字节相同。截短一个字节、持久集合里写了 0、1 之外的值都读不回。
    #[test]
    fn a_recorded_given_states_table_reads_back_from_its_bytes_and_a_damaged_one_does_not() {
        use singlefs_harness::memory_pool::RecordStreamSeam;
        let flow = a_small_recorded_flow();
        let table = RecordedGivenStatesTable {
            table_name: "第一截/种子 3".to_string(),
            base: flow.base,
            writes: flow.writes,
            segments: flow.segments,
            seams: vec![
                RecordStreamSeam::AfterAPowerLoss {
                    first_write_after_the_power_loss: 1,
                },
                RecordStreamSeam::AfterACrash {
                    first_write_after_the_crash: 2,
                    version_landed_on_after_the_crash: Some((
                        InstanceGeneration(4),
                        CheckpointTxg(9),
                    )),
                },
                RecordStreamSeam::AfterACrash {
                    first_write_after_the_crash: 2,
                    version_landed_on_after_the_crash: None,
                },
            ],
            persisted_sets: vec![vec![true, false, true], vec![false, false, false]],
        };
        let bytes = table.to_bytes();
        let read_back = RecordedGivenStatesTable::from_bytes(&bytes).expect("刚写的字节读得回");
        assert_eq!(read_back, table, "读回的与写进去的逐样相同");
        assert_eq!(read_back.to_bytes(), bytes, "读回的再写一遍逐字节相同");
        assert_eq!(
            RecordedGivenStatesTable::from_bytes(&bytes[..bytes.len() - 1]),
            None,
            "截短一个字节"
        );
        let mut not_a_bool = bytes.clone();
        *not_a_bool.last_mut().expect("至少一个字节") = 2;
        assert_eq!(
            RecordedGivenStatesTable::from_bytes(&not_a_bool),
            None,
            "持久集合里写了 2"
        );
    }

    /// 带两栏的那一份读回来与原样逐样相同（表连同三棵容器树各自的旋钮值，没装旋钮的那一棵读回也没装；恢复自称的那一版三种写法逐个相同）；
    /// 旋钮值越出那一棵树的范围、恢复自称的那一版码认不得、个数与持久集合不等、头不对都读不回。
    #[test]
    fn a_recorded_given_states_table_with_its_columns_reads_back_with_its_knobs_and_an_out_of_range_knob_does_not(
    ) {
        use singlefs_checker::walk::{
            ContainerCapacityKnobs, LineageContainerCapacityKnob, TombstoneContainerCapacityKnob,
        };
        let flow = a_small_recorded_flow();
        let recorded = RecordedGivenStatesTableWithItsColumns {
            table: RecordedGivenStatesTable {
                table_name: "墓碑容器压到一片 2 条".to_string(),
                base: flow.base,
                writes: flow.writes,
                segments: flow.segments,
                seams: Vec::new(),
                persisted_sets: vec![
                    vec![true, true, false],
                    vec![true, false, false],
                    vec![false, false, false],
                ],
            },
            container_capacity_knobs: ContainerCapacityKnobs {
                tombstone: Some(TombstoneContainerCapacityKnob::new(2).expect("1..=815")),
                lineage: Some(LineageContainerCapacityKnob::new(1).expect("1..=1359")),
                retirement: None,
            },
            claimed_versions: vec![
                ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
                    InstanceGeneration(3),
                    CheckpointTxg((1 << 40) | 7),
                ))),
                ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None),
                ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery,
            ],
        };
        let bytes = recorded.to_bytes();
        assert_eq!(
            RecordedGivenStatesTableWithItsColumns::from_bytes(&bytes),
            Some(recorded.clone()),
            "读回的与写进去的逐样相同（退役树没装旋钮读回也没装）"
        );
        let tombstone_knob_offset = RECORDED_GIVEN_STATES_TABLE_WITH_ITS_COLUMNS_MAGIC.len();
        let mut out_of_range = bytes.clone();
        out_of_range[tombstone_knob_offset..tombstone_knob_offset + 8]
            .copy_from_slice(&816u64.to_le_bytes());
        assert_eq!(
            RecordedGivenStatesTableWithItsColumns::from_bytes(&out_of_range),
            None,
            "墓碑容器旋钮值 816 越出 1..=815"
        );
        assert_eq!(
            RecordedGivenStatesTableWithItsColumns::from_bytes(&recorded.table.to_bytes()),
            None,
            "不带两栏的那一种字节头不对"
        );
        let first_claimed_version_offset = tombstone_knob_offset + 3 * 8 + 8;
        let mut unknown_claimed_version_code = bytes.clone();
        unknown_claimed_version_code[first_claimed_version_offset] = 3;
        assert_eq!(
            RecordedGivenStatesTableWithItsColumns::from_bytes(&unknown_claimed_version_code),
            None,
            "恢复自称的那一版码 3 认不得"
        );
        let mut one_claimed_version_short = recorded.clone();
        one_claimed_version_short.claimed_versions.pop();
        assert_eq!(
            RecordedGivenStatesTableWithItsColumns::from_bytes(
                &one_claimed_version_short.to_bytes()
            ),
            None,
            "恢复自称的那一版比持久集合少一个"
        );
    }
}

// ============================== 在测试代码里给崩溃点写身份 ==============================

/// 一次写属于哪个周期：录制流只带写的种类；单元写再按单元头第 6 字节的类标签分成写数据（码 1）、更新索引树（码 2）、
/// 写打包记录（码 3）——单元自描述，不用核心层打标记（用户 2026-09-28 定周期是「写入、写日志、更新索引树」这一级）。
/// 认不出类标签的单元写仍叫 `unit_write`。
#[must_use]
pub fn period_of_write(write: &RetainedWrite) -> &'static str {
    match (write.kind, &write.contents) {
        (StepKind::UnitWrite, WrittenContents::Bytes(bytes)) => {
            period_of_unit_class(bytes.get(6).copied())
        }
        (StepKind::UnitWrite, WrittenContents::Zeros { .. }) => "unit_write",
        (
            StepKind::ZeroFill
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier,
            _,
        ) => write.kind.name(),
    }
}

/// 单元头第 6 字节的类标签 → 周期名（与池级 checker `unit_checksum_layout` 认的三个码相同）。
#[must_use]
pub fn period_of_unit_class(unit_class: Option<u8>) -> &'static str {
    match unit_class {
        Some(1) => "data_unit",
        Some(2) => "index_node",
        Some(3) => "packed_records",
        Some(_) | None => "unit_write",
    }
}

/// 崩溃点记录器：测试代码调一个入口（取号、暖机、发布、回退、抬 F、卸载）时包一层，记下它发出的那段录制流与设它的文件；
/// 录制流自己只记卸载入口（`RecordedPublishEntry::Unmount`），别的入口的名字只有调它的测试代码知道，所以身份在这里写。
/// 换成崩溃点时每个屏障段一个点，名字写成 `<流程>.<周期>.<步>`（流程 = 入口的名字，周期 = 这一段里写的种类，步 = 这个入口里这一种周期的第几段；
/// 用户 2026-09-28 定：路径段按「测哪条流程的哪个周期的第几步」写、节点切到周期与步）。同一个入口调几次就是几段路径，名字相同、位置不同。
pub struct CrashPointRecorder {
    stream: singlefs_harness::SharedStream,
    mkfs_operation_count: usize,
    /// （设它的文件，入口的名字，录制流下标的半开区间，绝对下标）。
    spans: Vec<(String, String, Range<usize>)>,
}

impl CrashPointRecorder {
    #[must_use]
    pub fn new(stream: singlefs_harness::SharedStream, mkfs_operation_count: usize) -> Self {
        Self {
            stream,
            mkfs_operation_count,
            spans: Vec::new(),
        }
    }

    /// 已经发出去的一段（例如 harness 的 `build_pool` 里取号加暖机那一段），按录制流的绝对下标记；`code_file` 给 `file!()`，
    /// 归一成仓内路径（[`repository_relative_file`]）。
    ///
    /// # Panics
    /// 这一段没接上前一段（中间有没归任何点的操作，或倒退）。
    pub fn record_span(&mut self, code_file: &str, name: &str, operations: Range<usize>) {
        let expected_start = self
            .spans
            .last()
            .map_or(self.mkfs_operation_count, |(_, _, span)| span.end);
        assert_eq!(
            operations.start, expected_start,
            "崩溃点 {name} 那一段没接上前一个点"
        );
        assert!(operations.end >= operations.start);
        self.spans.push((
            repository_relative_file(code_file),
            name.to_string(),
            operations,
        ));
    }

    /// 调一个入口，把它发出的那段流记下来；`code_file` 给 `file!()`。
    pub fn step<Output>(
        &mut self,
        code_file: &str,
        name: &str,
        body: impl FnOnce() -> Output,
    ) -> Output {
        let start = self.stream.operation_count();
        let output = body();
        let end = self.stream.operation_count();
        self.record_span(code_file, name, start..end);
        output
    }

    /// 换成写表上的崩溃点：每个屏障段一个点，归它最后一次写落在的那个入口；名字 `<入口>.<这一段里写的种类，`+` 接>.<这个入口里这一种周期的第几段>`。
    /// `writes`、`segments`、`stream_indexes` 是 `writes_and_segments_with_stream_indexes` 交回的三样（下标都在 mkfs 之后那段录制流里）。
    ///
    /// # Panics
    /// 记下的段没有罩住整条流（流尾还有没归任何点的操作）；段不是首尾相接的写区间；有写不属于任何段。
    #[must_use]
    pub fn into_crash_points(
        self,
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
        stream_indexes: &[usize],
    ) -> Vec<CrashPointSpan> {
        let end = self
            .spans
            .last()
            .map_or(self.mkfs_operation_count, |(_, _, span)| span.end);
        assert_eq!(
            end,
            self.stream.operation_count(),
            "流尾还有没归任何崩溃点的操作"
        );
        assert_eq!(
            writes.len(),
            stream_indexes.len(),
            "每次写各有一个录制流下标"
        );
        // 每个入口管的写区间（首尾相接）
        let mut entries: Vec<(String, String, Range<usize>)> = Vec::new();
        let mut next_write = 0usize;
        for (code_file, name, operations) in self.spans {
            let relative_end = operations.end - self.mkfs_operation_count;
            let write_end = stream_indexes[next_write..]
                .iter()
                .position(|index| *index >= relative_end)
                .map_or(stream_indexes.len(), |offset| next_write + offset);
            entries.push((code_file, name, next_write..write_end));
            next_write = write_end;
        }
        assert_eq!(next_write, stream_indexes.len(), "每次写都归了某个入口");
        let mut points = Vec::new();
        let mut expected_start = 0usize;
        let mut steps_of_period_in_entry: BTreeMap<(usize, String), usize> = BTreeMap::new();
        for segment in segments {
            let first = *segment.first().expect("段里至少一次写");
            let last = *segment.last().expect("段里至少一次写");
            assert_eq!(first, expected_start, "段没有接上前一段的写区间");
            assert!(
                segment.windows(2).all(|pair| pair[1] == pair[0] + 1),
                "一段里的写下标连续"
            );
            expected_start = last + 1;
            let entry_index = entries
                .iter()
                .position(|(_, _, range)| range.contains(&last))
                .expect("每一段的最后一次写归某个入口");
            let mut kinds: Vec<&'static str> = Vec::new();
            for write_index in segment {
                let kind = period_of_write(&writes[*write_index]);
                if !kinds.contains(&kind) {
                    kinds.push(kind);
                }
            }
            let period = kinds.join("+");
            let step = steps_of_period_in_entry
                .entry((entry_index, period.clone()))
                .or_insert(0);
            *step += 1; // 这一种周期在这个入口里的第几段
            let step_in_period = *step;
            let (code_file, entry_name, _) = &entries[entry_index];
            points.push(CrashPointSpan {
                name: format!("{entry_name}.{period}.{step_in_period}"),
                code_file_in_repository: code_file.clone(),
                writes: first..last + 1,
                ignored: false,
            });
        }
        assert_eq!(expected_start, writes.len(), "每次写都在某一段里");
        points
    }
}

// ============================== 按环境跑：一台机器（几张卡）领自己那一份 ==============================

/// 这台机器领哪一份：`SINGLEFS_CRASH_AMPLIFICATION_SHARE=<i>/<n>`，没设是 `0/1`（整份自己跑）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostShare {
    pub index: usize,
    pub count: usize,
}

impl HostShare {
    /// # Errors
    /// 写法不是 `<i>/<n>`、n 为 0、i 不小于 n。
    pub fn parse(text: &str) -> Result<Self, String> {
        let (index, count) = text
            .split_once('/')
            .ok_or_else(|| format!("分片写法要是 <i>/<n>，实际「{text}」"))?;
        let index: usize = index
            .trim()
            .parse()
            .map_err(|_| format!("分片序号不是数：「{text}」"))?;
        let count: usize = count
            .trim()
            .parse()
            .map_err(|_| format!("分片总数不是数：「{text}」"))?;
        if count == 0 || index >= count {
            return Err(format!("分片要 0 ≤ i < n，实际「{text}」"));
        }
        Ok(Self { index, count })
    }
}

/// 这台机器领哪一份（`SINGLEFS_CRASH_AMPLIFICATION_SHARE`，没设是整份）与几台机器的权重（`SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=<w0>,<w1>,…`，
/// 没设等分）：流水线（[`run_from_environment`]）与给定状态表（[`GivenStatesEngine`]）两台分片同一种读法。
///
/// # Errors
/// 分片写法错（[`HostShare::parse`]）；权重不是数、个数与分片总数不等。
pub(crate) fn host_share_and_weights_from_the_environment() -> Result<(HostShare, Vec<u64>), String>
{
    let share = match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_SHARE") {
        Ok(text) => HostShare::parse(&text)?,
        Err(_) => HostShare { index: 0, count: 1 },
    };
    let host_weights: Vec<u64> = match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS") {
        Ok(text) => {
            let weights: Vec<u64> = text
                .split(',')
                .map(|weight| {
                    weight
                        .trim()
                        .parse::<u64>()
                        .map_err(|_| format!("主机权重不是数：「{text}」"))
                })
                .collect::<Result<_, _>>()?;
            if weights.len() != share.count {
                return Err(format!(
                    "主机权重 {} 个，分片总数 {}",
                    weights.len(),
                    share.count
                ));
            }
            weights
        }
        Err(_) => vec![1; share.count],
    };
    Ok((share, host_weights))
}

/// 按环境跑一趟的结果：录了多少、核了多少、对账剩多少、违例几条（带秒数，只 `PartialEq`）。
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentRun {
    pub mode: CrashAmplificationRunMode,
    pub share: HostShare,
    pub cards_used: usize,
    pub recording: RecordingTally,
    pub checking: CheckingTally,
    pub imported_blocks: u64,
    /// 这条流自己的块里还没判的（[`unjudged_blocks_of_the_flow`]）；库里别的流的块不算。
    pub unjudged_blocks: usize,
    pub violations: Vec<StoredRedState>,
    pub coverage_lines: Vec<String>,
    pub library_directory: PathBuf,
    /// 第 ② 段跑了就有：在哪算、核了多少。
    pub verification: Option<VerificationOutcome>,
    /// 第 ③ 段：核对表与判定块逐状态比的结果。
    pub comparison: Option<JudgeVerifierComparison>,
    /// 第 ① 段在卡上判的（配置开了 GPU、表建得出）就有。
    pub gpu_judge: Option<GpuJudgeOutcome>,
    /// 这一趟只跑了录入段（[`RunStage::RecordOnly`]）：流程与事实表（配了 GPU 再加判表）已写进库、读回核过，一块都没判。
    pub recorded_only: bool,
    /// 这一趟只排了派活计划（`SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1`）：录入与导入照做，一块都没判。
    pub placement_only: bool,
    /// 这一趟被叫停了（[`stop_is_requested_by_the_environment`]），或显存不够（起过卡池的进程在显存账上此刻借不到、卡上 wgpu 报了
    /// Out of Memory）、打了「本次未跑」那一行（文件头第 5 条）：判过的块在库里，没做完的段留给下一趟，对账不为空是正常的；
    /// 分项驱动照它把这一项记成本次未跑。只排派活计划的那一趟借不到试判的草稿区也置上（`placement_only` 同时为真，那一趟没打
    /// `CRASH_AMPLIFICATION_PLACEMENT` 行）。
    pub stopped_early: bool,
    /// 这一趟每个点各种结论的版本（第三轮判决 J8）。
    pub conclusion_versions: ConclusionVersionsOfThePlan,
    /// 这一趟第 ① 段是哪一个判器判的（J5）；它那一处的结论见 [`EnvironmentRun::judged_conclusions`]。
    pub judged_by: JudgedBy,
    /// 这一趟每一类计算在哪算（[`computation_routes`]）。
    pub computation_routes: ComputationRoutes,
    /// 这一趟的展开上限（调用方给的，与流自己的展开方式逐段核过）。
    pub expand_up_to: ExpandUpToWritesPerSegment,
    /// 每一段都不超过展开上限（全域）；测试代码里 ignore 的点另报，不算在这里。
    pub exhaustive: bool,
    /// 这一行报的复用态数，取法见 [`reuse_and_scale_fields_of_the_line`]。
    pub states_reused: u64,
    /// 测试代码里预设 ignore 的点（按登记整批不判的一类，不算剪枝）与因此没展开的态数。
    pub ignored_points: usize,
    pub states_skipped_by_ignore: u64,
    /// 这一趟用的卡从哪来（[`GpuCardsSource`]）。
    pub gpu_cards_taken_from: GpuCardsSource,
    /// 起过卡池的进程第 ② 段在卡上核时，在显存账上为核对的缓冲借的（MiB，文件头第 5 条）；没借是 `None`。
    pub facts_verifier_buffers_booked_mebibytes: Option<u64>,
    /// 起过卡池的进程第 ② 段核完那一刻核账：核对的缓冲那一笔借据还在账上没有（同上）；没借是 `None`。
    pub facts_verifier_buffers_lease_on_the_books_after_verifying: Option<bool>,
    /// 按段登记表里这条流登记了的段（J10）：这一趟照「甲二」展开、计入跳过量，行里报段号与写数（[`reuse_and_scale_fields_of_the_line`]）。
    pub registered_segments: Vec<RegisteredSegment>,
    /// 这条流里没登记、展开上限又盖不住的段（[`unregistered_segments_beyond_the_expansion`]），行里报段号与写数。
    pub unregistered_segments_beyond_the_expansion: Vec<SegmentWithItsWriteCount>,
    /// 登记了的段各自的域（`full` / `slice`）与分片的段到这一趟末判到哪（[`domains_and_progress_of_the_registered_segments`]，C790）。
    pub registered_segment_domains: Vec<DomainAndProgressOfARegisteredSegment>,
    /// 挂在节点上的观察者的判段这一趟拼了什么、这条流挂了观察者的块读回来的计数（J4、J7）；没挂观察者、只录、只排派活计划、被叫停的那一趟是 `None`。
    pub observing: Option<ObservingOfTheRun>,
    /// 观察者的录入段这一趟录了什么（用户 2026-10-13 定「观察者改走两段」）：没挂观察者、只判的那一趟是 `None`。
    pub observer_recording: Option<ObserverRecordingTally>,
    /// 配置开了 GPU 而这份二进制没编进 gpu 特性：判段这一趟没跑（本次未跑，不判红、不退回 CPU 判），这是原因；录入段照录。
    /// 那一趟 `recorded_only` 也是真。
    pub not_run_without_the_gpu_feature: Option<String>,
    /// 计数向量与计划对不上的块，一块一句（J7，每一趟都判）；对得上是空的。只排派活计划、被叫停的那一趟不判，是空的。
    pub count_vector_disagreements: Vec<String>,
}

impl EnvironmentRun {
    /// 这一趟第 ① 段那个判器的结论记在库里哪一处（对账、读违例照它）。
    #[must_use]
    pub fn judged_conclusions(&self) -> ConclusionsStoredUnder<'_> {
        self.conclusion_versions.stored_under(self.judged_by)
    }
}

/// 一趟用的卡从哪来（定义在 harness 档，草稿区记不记账的判法 `singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`
/// 照它分，单测在 harness 档随时跑得到）。
pub use singlefs_harness::gpu_memory_ledger::GpuCardsSource;

/// 每一行 `CRASH_AMPLIFICATION ` 末尾都接的几个字段（汇总行、被叫停的行、`flow=` 行、两台跑的分份行与导入行，打法都在
/// `crates/singlefs-checker-tier/tests/` 里调 [`run_from_environment`] 的用例）：
///
/// ` states_reused=<n> exhaustive=<true|false> expand_up_to=<n> ignored_points=<n> states_skipped_by_ignore=<n> registered_segments=<…> unregistered_segments_beyond_expansion=<…>`
///
/// - `states_reused`：这一趟分给本机的块（`SINGLEFS_CRASH_AMPLIFICATION_SHARE=<i>/<n>` 那一份；没设是整份）里，录入时库中已有这一判法版本
///   的判定、这一趟因此没判的态数；导入之前数，导入进来的块不算。每一趟照实报自己的（R1 判决 L1：叫停、导入、只排计划都不写 0）：
///   被叫停的那一趟（[`stop_is_requested_by_the_environment`]）报它录入时库里已判的；导入那一趟（设了 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT`）
///   报导入之前库里已判的（两台跑时就是 `share=0/2` 那一行刚判过或复用过的那一份）；只排派活计划的那一趟同样报（它的
///   `CRASH_AMPLIFICATION_PLACEMENT` 行也带）。所以同一份库上几趟的行加起来会重复计：叫停之后接着跑的那一趟、导入那一趟数到的，
///   有一部分是同一项里前一行判过或报过的。
/// - `exhaustive`：每一段的写数都不超过展开上限（全域）。全量的完成标记只认 `true`。
/// - `expand_up_to`：这一趟的展开上限（[`ExpandUpToWritesPerSegment`]）。
/// - `ignored_points`、`states_skipped_by_ignore`：测试代码里预设 ignore 的点数，与它们的段因此没展开的态数（不 ignore 时照流自己的展开方式
///   该有的态数减去这一趟的）。ignore 是按登记整批不判的一类，不算剪枝，也不进复用的对照；入口的成功句照这两个数报跳过了多少。
/// - `registered_segments`：按段登记表里这条流登记了的段（J10），`<段号>:<段写数>` 逗号隔开，一段都没有写 `-`；这几段照「甲二」展开，
///   全量展开要的态数不报（H10：跳过量报段号与写数、不报十进制态数）。
/// - `unregistered_segments_beyond_expansion`：这条流里没登记、这一趟的展开上限又盖不住的段（[`unregistered_segments_beyond_the_expansion`]），
///   写法同 `registered_segments`，按段号升序；登了段的流照旧 `exhaustive=false`，入口照这个字段分得出登记段之外还有没有展不开的段。
/// - `registered_segment_domains`：登记了的每一段 `<段号>:<full|slice>`（C790：整域装得进 u64 全展开是 `full`，装不进分片是 `slice`），
///   一段都没登写 `-`。
/// - `registered_segment_progress`：`slice` 的段 `<段号>:<已判态数>/<域的总态数>`，已判态数是到这一趟末从编号 0 起连着判完的态数
///   （一趟判完是这一趟的片起点 + 片长），两个数都是精确的十进制；没有 `slice` 的段写 `-`。
///
/// 四个说段的字段由 [`segment_fields_of_the_line`] 拼。
#[must_use]
pub fn reuse_and_scale_fields_of_the_line(run: &EnvironmentRun) -> String {
    format!(
        " states_reused={} exhaustive={} expand_up_to={} ignored_points={} states_skipped_by_ignore={}{}",
        run.states_reused,
        run.exhaustive,
        run.expand_up_to.0,
        run.ignored_points,
        run.states_skipped_by_ignore,
        segment_fields_of_the_line(
            &run.registered_segments,
            &run.unregistered_segments_beyond_the_expansion,
            &run.registered_segment_domains
        )
    )
}

/// 不 ignore 时照流自己的展开方式该有的态数，减去这一份计划的态数（ignore 的点的段不展开）。一个点都没 ignore 是 0。
fn states_skipped_by_ignore(flow: &CrashFlow<'_>, plan: &CrashPlan) -> u64 {
    if !plan.points.iter().any(|point| point.ignored) {
        return 0;
    }
    let without_ignore = crash_state_ranges_by_segment_with_slices(
        flow.base,
        flow.writes,
        flow.segments,
        flow.expansion,
        plan.segment_slices(),
    )
    .1;
    let planned = plan.points.last().map_or(0, |point| point.ordinals.end);
    without_ignore
        .checked_sub(planned)
        .expect("ignore 只会让段少展开，态数不会比不 ignore 时多")
}

/// 这一份（`is_mine` 选的块）里录入时库中已有这一判法版本判定的态数。
fn states_already_judged_in_this_share(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<u64, VerdictStoreError> {
    let mut states = 0u64;
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if is_mine(this_block)
                && conclusions.is_stored(store, point_index, &point.block_key(&local))?
            {
                states += local.end - local.start;
            }
        }
    }
    Ok(states)
}

/// 只核给定序号的那些块（块序号 = 全部点的块按次序编号）；[`check_recorded_blocks`] 是它按权重分工的形态。
/// 每领一块之前先问 `stop_is_requested`（[`check_blocks_where_until`]）。
///
/// # Errors
/// 同 [`check_recorded_blocks`]。
pub fn check_selected_blocks(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    selected_block_sequences: &std::collections::HashSet<u64>,
    conclusions: &ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where_until(
        store,
        flow,
        plan,
        &|block_sequence| selected_block_sequences.contains(&block_sequence),
        conclusions,
        stop_is_requested,
    )
}

/// 环境变量里的路径：绝对的照用，相对的按仓根解析——cargo 把测试进程的当前目录设成包目录，第二台的驱动只知道树根。
#[must_use]
pub fn path_relative_to_repository_root(repository_root: &Path, text: &str) -> PathBuf {
    let path = Path::new(text);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repository_root.join(path)
    }
}

/// [`run_from_environment`] 一趟（连同它调的、这份文件里的函数）读的全部环境变量。调它的用例照这张表清掉、跑完还原
/// （`tests/common_crash_points/mod.rs` 的环境守卫），不读调用方与别的用例留下的值；这份文件里每一处按名字读环境变量都要在表里
/// （`crash_amplification::tests` 里有一条用例逐个核）。
pub const ENVIRONMENT_VARIABLES_READ_BY_A_RUN: &[&str] = &[
    "SINGLEFS_MULTI_HOST_CONFIG",
    "ENABLE_ACROSS_MACHINES",
    "ENABLE_GPU",
    "GPU_CARDS_BY_PRIORITY",
    SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE,
    "SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE",
    "SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY",
    "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
    "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS",
    CRASH_LIBRARY_DIRECTORY_ENVIRONMENT_VARIABLE,
    CRASH_LIBRARY_HOME_ENVIRONMENT_VARIABLE,
    CRASH_LIBRARY_LOCK_WAIT_SECONDS_ENVIRONMENT_VARIABLE,
    "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
    "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE",
    REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE,
    crate::crash_point_replay_progress::CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE,
    RUN_STAGE_ENVIRONMENT_VARIABLE,
    CHECKING_THREADS_ENVIRONMENT_VARIABLE,
    EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
];

/// 同一个测试二进制里改进程环境、读 [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里变量的用例一条接一条跑：几条一起改同一批变量会互相读到对方的值。
/// 这份库的单测与 `tests/` 下各个测试二进制各有一把（一个进程一份）。
static ONE_TEST_CHANGES_THE_ENVIRONMENT_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 拿着改环境的那把锁、不改环境：只读环境起引擎（[`GivenStatesEngine::start_from_environment`]）的用例在这把锁里起，
/// 别的用例拿着 [`CrashAmplificationEnvironment`] 改变量时它等着，不读到对方设的叫停文件、分片与库目录。引擎起步时把要读的都读完，
/// 锁在起完就放。拿着 [`CrashAmplificationEnvironment`] 的用例不许再拿它（同一把锁，不可重入）。
pub fn one_test_changes_the_environment_at_a_time() -> std::sync::MutexGuard<'static, ()> {
    ONE_TEST_CHANGES_THE_ENVIRONMENT_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// 给定状态表的引擎起步时读一次叫停文件的路径（[`stop_file_of_the_environment`]），之后每次问叫停只看那个文件在不在：同一个进程里
/// 别的用例拿着环境守卫换叫停文件时，跑着的引擎不被它叫停。记住叫停的那一个收 `'static` 的读法，闭包漏在堆上（一个引擎几十字节）。
fn stop_request_remembering_the_stop_file_of_the_engine_start(
) -> StopRequestRememberedInThisRun<'static> {
    let stop_file = stop_file_of_the_environment();
    StopRequestRememberedInThisRun::new(Box::leak(Box::new(move || {
        stop_file.as_ref().is_some_and(|path| path.exists())
    })))
}

/// 一批环境变量：建的时候记下原值，丢掉（用例收尾、panic）时逐个还原——原来没设的删掉，设了的写回原值。
pub struct EnvironmentRestoredOnDrop {
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl EnvironmentRestoredOnDrop {
    /// 记下 `names` 每个变量的原值，再全部删掉。
    #[must_use]
    pub fn saved_then_cleared(names: &[&'static str]) -> Self {
        // 卡池与用例抄的显卡先后只认调用方给这个进程的那一份：改任何一个变量之前先让它记下（还没记过的话，此刻的环境就是调用方的）
        let _ = crate::gpu_card_configuration::GpuConfigurationOfTheCaller::of_this_process();
        let saved = names
            .iter()
            .map(|name| (*name, std::env::var_os(name)))
            .collect();
        for name in names {
            std::env::remove_var(name);
        }
        Self { saved }
    }

    /// 设一个变量。
    ///
    /// # Panics
    /// 变量不在建它时记下的那一批里（丢掉时还原不了它）。
    pub fn set(&mut self, name: &str, value: &std::ffi::OsStr) {
        assert!(
            self.saved.iter().any(|(saved_name, _)| *saved_name == name),
            "{name} 不在记下原值的那一批里，丢掉时还原不了它"
        );
        std::env::set_var(name, value);
    }
}

impl Drop for EnvironmentRestoredOnDrop {
    fn drop(&mut self) {
        for (name, value) in &self.saved {
            match value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }
}

/// 读这份文件里环境变量的用例的环境（K5：用例自己清、自己还原，不读别的用例与调用方留下的值）：拿住这个测试二进制里改环境的锁，
/// 把一趟读的全部变量（[`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`]）记下原值、清掉，再设用例给的那几个；丢掉时（用例收尾、panic）逐个还原、放锁。
/// 经 [`run_from_environment`] 跑的用例（`tests/common_crash_points/mod.rs` 转出去用）与这份文件里改环境的单测走的都是它。
pub struct CrashAmplificationEnvironment {
    // 先还原、后放锁：字段按声明的次序丢
    variables: EnvironmentRestoredOnDrop,
    _one_test_at_a_time: std::sync::MutexGuard<'static, ()>,
}

impl CrashAmplificationEnvironment {
    /// # Panics
    /// `settings` 里有不在 [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里的变量。
    #[must_use]
    pub fn cleared_then_set(settings: &[(&str, &std::ffi::OsStr)]) -> Self {
        // 上一条拿着锁的用例 panic 了：它的环境已经在丢守卫时还原，锁照样能用
        let one_test_at_a_time = ONE_TEST_CHANGES_THE_ENVIRONMENT_AT_A_TIME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut variables =
            EnvironmentRestoredOnDrop::saved_then_cleared(ENVIRONMENT_VARIABLES_READ_BY_A_RUN);
        for (name, value) in settings {
            variables.set(name, value);
        }
        Self {
            variables,
            _one_test_at_a_time: one_test_at_a_time,
        }
    }

    /// 用例跑到一半再设一个（例如第二趟之前放叫停文件）。
    ///
    /// # Panics
    /// 同 [`EnvironmentRestoredOnDrop::set`]。
    pub fn set(&mut self, name: &str, value: &std::ffi::OsStr) {
        self.variables.set(name, value);
    }
}

/// 从环境跑一趟：配置定跑法，`SINGLEFS_CRASH_AMPLIFICATION_SHARE` 定这台机器领哪一份，`SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 是库目录
/// （没设就是家目录下崩溃放量那一类的长期库，`crash_library::crash_library_directory_of`；设了就要落在开机不清空的地方——
/// `/tmp`、`/var/tmp`、`/dev/shm`、`/run` 之下一律拒绝；没有临时库这一说），
/// `SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 是别的机器拷回来的库目录（`:` 分隔），核之前先导入；同一条命令再跑一次就是续跑：
/// 同一（块键，结论种类，版本）下判过的块全部复用，只补没判的；
/// 跑法带 GPU 时卡按配置的优先级用（多机配置里的 `GPU_CARDS_BY_PRIORITY`、清场之后的 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`，[`parse_cards_by_priority`]；
/// 这个进程在哪台机器看 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`）：任务先装第一张卡，装到它的显存额度为止，装不下的才给下一张；没配就按本机能用的卡的
/// 空闲显存从大到小用（`gpu` 特性没编进来而配置写了 `ENABLE_GPU=1`：判段这一趟报本次未跑、录入段照录，不悄悄退回 CPU 判，
/// [`judge_not_run_without_the_gpu_feature`]）。判在哪只看本机配置的 GPU 开关
/// （[`computation_routes`]，崩溃点重放合并第三轮判决 J5）：开了 GPU 判器、核对、单元校验和三类都走 GPU。
/// 开了 GPU 只由 GPU 判器判，CPU 不在同一段里判同一批块、不对拍；没开只由 CPU 判器判。
/// 结论按（块键，结论种类，版本）存（J8）：GPU 结论一把版本、CPU 结论按设点文件各一把（[`ConclusionVersionsOfThePlan`]），开库之前与编译时
/// 写进二进制的那一份核，对不上就整趟不写库、打 [`CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX`] 那一行交错（入口照它退 78）。
/// `SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1` 只排派活计划：打一行 `CRASH_AMPLIFICATION_PLACEMENT …`（要判多少、每张卡装多少、
/// 这台机器的卡试判几次派活量到的速度），一块都不判。
/// 主机之间等分（`SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=<w0>,<w1>,…` 可以改成按权重）。
/// `expand_up_to` 是这一趟的展开上限，进交回的结果与 `CRASH_AMPLIFICATION_GPU_TOTAL` 行；流自己的展开方式要与它逐段相同。
///
/// # Errors
/// 环境变量写法错、库打不开（连同按结论种类存之前的旧库：换一个新目录建库）、判定存储读写失败；
/// 流的展开方式与 `expand_up_to` 在某一段上不同（报的展开上限就不是这一趟枚举的那个，在任何录入之前拒）；结论版本与编译时对不上
/// （开库之前拒）；开了 GPU 而这一趟上不了卡；只排派活计划而没开 GPU。
#[allow(
    clippy::too_many_lines,
    reason = "读环境、核版本、开库、导入、录入、按卡分工核对与对照、对账、打行，是一趟的全部步骤，拆开要把十几个量来回传"
)]
pub fn run_from_environment(
    flow: &CrashFlow<'_>,
    flow_name: &str,
    toolchain: &CrashPointReplayToolchainIdentity,
    repository_root: &Path,
    expand_up_to: ExpandUpToWritesPerSegment,
) -> Result<EnvironmentRun, String> {
    run_from_environment_with_observers(
        flow,
        flow_name,
        &[],
        toolchain,
        repository_root,
        expand_up_to,
    )
}

/// 同 [`run_from_environment`]，另带挂在节点上的观察者（崩溃点重放合并第三轮判决 J4，[`ObserverOnTheNodes`]；两段：用户 2026-10-13 定）：
/// 录入段把这台机器领的块里落在观察者节点上的逐状态看一遍（[`record_the_observers_of_the_flow`]，CPU；交回镜像的按恢复镜像去重），结果写进库；
/// 判段从库里读回，观察者交回的盘上的池级 checker 配了 GPU 在卡上判、没配由 CPU 判，逐状态拼回结论存进库
/// （[`judge_the_observers_of_the_flow`]）；这条流挂了观察者的每一块（这一趟拼的与库里复用的）的计数读回来交在 [`EnvironmentRun::observing`]，
/// 调用方照它判观察者计数的断言（每一趟都判，J7）。观察者的版本开库之前与编译时那一份核，对不上同判器的版本整趟不写库。
///
/// # Errors
/// 同 [`run_from_environment`]；按段登记表读不了、写坏了、登记的段与这条流的写表对不上（[`RegisteredSegment`]）；观察者看一块时库读写失败。
#[allow(
    clippy::too_many_lines,
    reason = "读环境、核版本、开库、导入、录入、按卡分工核对与对照、看观察者、对账、打行，是一趟的全部步骤，拆开要把十几个量来回传"
)]
pub fn run_from_environment_with_observers(
    flow: &CrashFlow<'_>,
    flow_name: &str,
    observers: &[ObserverOnTheNodes<'_>],
    toolchain: &CrashPointReplayToolchainIdentity,
    repository_root: &Path,
    expand_up_to: ExpandUpToWritesPerSegment,
) -> Result<EnvironmentRun, String> {
    // 两段（用户 2026-10-11 定）：录入段在 CPU 上跑数据、写进库；判段只从库里读，段与段之间不在内存里交数据
    let stage = RunStage::from_the_environment()?;
    // 多机配置只有一份（J11、M4 第 4 条）：设了 SINGLEFS_MULTI_HOST_CONFIG 照它，没设按 git common-dir 找主工作树的 multi-host.env
    let configuration_file: Option<PathBuf> = multi_host_configuration_file(repository_root)?;
    let mode = match &configuration_file {
        Some(configuration) => CrashAmplificationRunMode::from_configuration_file(configuration),
        None => CrashAmplificationRunMode::from_configuration_file_or_environment(
            &repository_root.join("multi-host.env"),
        ),
    };
    // 显卡的先后与额度也写在这一份多机配置里（GPU_CARDS_BY_PRIORITY、清场之后 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）；
    // 环境变量 GPU_CARDS_BY_PRIORITY 设了照它（驱动挑好一份随命令送，第二台也靠它）
    let cards_by_priority =
        cards_by_priority_from_configuration_or_environment(configuration_file.as_deref())?;
    // 配置开了 GPU 而这份二进制没编进 gpu 特性（提交时快档不带特性那一趟）：判段这一趟不跑、报本次未跑（不判红、不退回 CPU 判），
    // 录入段照录（CPU 跑数据、写进库），带特性那一趟从库里读回、在卡上判（用户 2026-10-13 定「配 GPU 必用 GPU」）
    let not_run_without_the_gpu_feature =
        judge_not_run_without_the_gpu_feature(mode.uses_gpu(), cfg!(feature = "gpu"));
    if let Some(reason) = &not_run_without_the_gpu_feature {
        for line in not_run_lines_without_the_gpu_feature(flow_name, reason) {
            println!("{line}");
        }
    }
    let records = stage.records();
    let judges = stage.judges() && not_run_without_the_gpu_feature.is_none();
    // 判在哪只看本机配置的 GPU 开关（J5），开库之前定：配了 GPU 只由 GPU 判、没配只由 CPU 判；判段不跑的那一趟录入段照 CPU 录（不建 GPU 判表）
    let routes = computation_routes(
        mode.uses_gpu() && not_run_without_the_gpu_feature.is_none(),
        cfg!(feature = "gpu"),
    )?;
    let host = CardHost::of_this_process()?;
    let placement_only = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY")
        .is_ok_and(|value| value == "1");
    let (share, host_weights) = host_share_and_weights_from_the_environment()?;
    // 库：入口脚本按趟交的目录（SINGLEFS_CRASH_AMPLIFICATION_LIBRARY），没交就是家目录下崩溃放量那一类的长期库；没有临时库
    let library_directory =
        crash_library_directory_of(CrashLibraryClass::CrashAmplification, repository_root)?;
    let planning_started = std::time::Instant::now();
    let library =
        LibraryOpenInThisProcess::open_or_share(&library_directory, LibraryRole::ThisHostsLibrary)?;
    let mut store = library.lock();
    // 录入段：按段登记（J10）改展开、排计划、把流程与事实表（配了 GPU 再加判表）写进库、登清单
    if records {
        // 按段登记（J10）：登记了的段照「甲二」展开，开跑前核段写数与段写表摘要，对不上就拒跑（M4 第 15 条）
        let registered_segments =
            registered_segments_of_the_flow_from_the_environment(flow, flow_name)?;
        let is_registered = |segment_index: usize| {
            registered_segments
                .iter()
                .any(|registered| registered.segment_index == segment_index)
        };
        if let Some((segment_index, segment)) =
            flow.segments
                .iter()
                .enumerate()
                .find(|(segment_index, segment)| {
                    !is_registered(*segment_index)
                        && (flow.expansion)(*segment_index, segment)
                            != expand_up_to.expansion_of(segment)
                })
        {
            return Err(format!(
                "流的展开方式与报的展开上限 {} 在第 {segment_index} 段（{} 次写）上不同：流给 {}，上限该给 {}",
                expand_up_to.0,
                segment.len(),
                (flow.expansion)(segment_index, segment).name(),
                expand_up_to.expansion_of(segment).name()
            ));
        }
        // 登记了的段在重型里跑轻型那两个域的并（第三轮判决 J10、M6-10：重型 ⊇ 轻型）：甲二 ∪ 展开 6
        let expansion_with_the_registered_segments = |segment_index: usize, segment: &[usize]| {
            expansion_in_the_recording(&registered_segments, flow, segment_index, segment)
        };
        // 判的就是这条流，登记了的段换成「甲二」展开（没登记一段时与调用方给的逐段相同）
        let flow = &CrashFlow {
            base: flow.base,
            writes: flow.writes,
            segments: flow.segments,
            judged_root_index: flow.judged_root_index,
            versions: flow.versions,
            versions_without_a_file: flow.versions_without_a_file,
            expansion: &expansion_with_the_registered_segments,
            crash_points: flow.crash_points.clone(),
        };
        let exhaustive =
            expand_up_to.covers_every_segment(flow.segments) && registered_segments.is_empty();
        // 登记段整域（甲二 ∪ 展开 6）装得进 u64 的全展开，装不进的分片（C790）：越过 1280 位的在写库之前拒跑
        let segment_slices = segment_slices_of_the_recording_from_the_environment_and_the_library(
            &store,
            flow,
            flow_name,
            &registered_segments,
            repository_root,
            toolchain,
            routes.crash_judge,
        )?;
        // 排计划要算态数：先核装得进 u64，装不进在写库之前拒跑，不在算态数那一步 panic
        states_of_the_flow_to_record(flow, &segment_slices)?;
        let plan_of_the_recording = plan_crash_points_with_slices(flow, &segment_slices);
        let manifest = record_the_flow_into_the_library(
            &mut store,
            flow,
            flow_name,
            &plan_of_the_recording,
            &registered_segments,
            exhaustive,
            routes.crash_judge == ComputedOn::Gpu,
        )?;
        println!(
            "CRASH_AMPLIFICATION_RECORDED flow={flow_name} recorded_flow={} facts={} packed_judge_tables={} blocks={} exhaustive={exhaustive}",
            hexadecimal_text(&manifest.recorded_flow_digest),
            hexadecimal_text(&manifest.facts_digest),
            manifest
                .packed_judge_tables_digest
                .as_ref()
                .map_or_else(|| "-".to_string(), |digest| hexadecimal_text(digest)),
            plan_of_the_recording
                .points
                .iter()
                .map(|point| point.blocks().len())
                .sum::<usize>(),
        );
    }
    // 判段：只从库里读回录入段那一趟（清单、流程、事实表、配了 GPU 时的判表），下面判、核对、比对都只用读回的这几样
    let read_back =
        read_back_the_recorded_run(&store, flow_name, routes.crash_judge == ComputedOn::Gpu)?;
    let registered_segments = read_back.manifest.registered_segments.clone();
    let exhaustive = read_back.manifest.exhaustive;
    let flow_of_the_library = read_back.flow.flow();
    let unregistered_segments_beyond_the_expansion = unregistered_segments_beyond_the_expansion(
        flow_of_the_library.segments,
        &registered_segments,
        expand_up_to,
    );
    // 分片的段照录入段定的片（读回的流带着），不另算
    let plan = plan_crash_points_with_slices(
        &flow_of_the_library,
        &read_back.flow.recorded().segment_slices,
    );
    // 结论的版本（J8）：开库之前算、与编译时写进二进制的那一份核，对不上整趟不写库（第二轮判决 L3-1：不等就退 78）；观察者的版本同（J4）
    let judged_by = match routes.crash_judge {
        ComputedOn::Gpu => JudgedBy::GpuJudge,
        ComputedOn::Cpu => JudgedBy::CpuJudge,
    };
    let versions_to_write_under = ConclusionVersionsOfThePlan::of_the_plan(
        repository_root,
        &plan,
        toolchain,
    )
    .and_then(|conclusion_versions| {
        observers
            .iter()
            .map(|observer| {
                observer_versions_to_write_under(repository_root, observer, judged_by, toolchain)
            })
            .collect::<Result<Vec<ObserverVersions>, ConclusionVersionsNotWritable>>()
            .map(|observer_versions| (conclusion_versions, observer_versions))
    });
    let (conclusion_versions, observer_versions) = match versions_to_write_under {
        Ok(versions) => versions,
        Err(not_writable) => {
            let line = not_writable.not_written_line();
            println!("{line}");
            return Err(line);
        }
    };
    let observer_conclusion_versions: Vec<ConclusionVersion> = observer_versions
        .iter()
        .map(|versions| versions.conclusion)
        .collect();
    let judged_conclusions = conclusion_versions.stored_under(judged_by);
    let needed_conclusions = vec![judged_conclusions.clone()];
    // 登记了的段的域与分片进度（驱动行两个字段，C790）：只录、只排计划的那几趟报录入时库里的
    let registered_segment_domains_at_recording = domains_and_progress_of_the_registered_segments(
        &store,
        &flow_of_the_library,
        &plan,
        &registered_segments,
        &judged_conclusions,
    )
    .map_err(|error| format!("读分片进度失败：{error:?}"))?;
    let mut report = CoverageReport::default();
    let recording = tally_the_recorded_blocks(&store, &plan, &needed_conclusions, &mut report)
        .map_err(|error| format!("数录入的块失败：{error:?}"))?;
    let host_shares = WeightedShares::new(host_weights);
    let total_blocks: u64 = plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数装得进 u64"))
        .sum();
    let my_blocks: Vec<u64> = (0..total_blocks)
        .filter(|block| host_shares.owner_of_block(*block) == share.index)
        .collect();
    let selected: std::collections::HashSet<u64> = my_blocks.iter().copied().collect();
    // 复用数在导入之前数：导入进来的块是别的机器这一趟判的，不算复用。叫停、导入、只排计划的那一趟都照这个数报（L1）
    let import_list = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_IMPORT").ok();
    let states_reused_at_recording =
        states_already_judged_in_this_share(&store, &plan, &judged_conclusions, &|block| {
            selected.contains(&block)
        })
        .map_err(|error| format!("数复用失败：{error:?}"))?;
    let ignored_points = plan.points.iter().filter(|point| point.ignored).count();
    let states_skipped_by_ignore = states_skipped_by_ignore(&flow_of_the_library, &plan);
    // 这一趟的叫停：观察者的录入段、第 ① 段的外层循环、卡线程、一轮收尾与下面整趟收尾都问这一个，见过一次就记住（第 10z 批第六块，
    // 第 10z 批第五块代码轮判决 K2-2 攻方 F2：叫停文件中途被删时各自现读，轮末断言 panic、或整趟没记成提早收尾）
    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
    let stop_is_requested = || stop_request.is_requested();
    // 观察者的录入段（用户 2026-10-13 定「观察者改走两段」）：这台机器领的块里落在观察者节点上的，CPU 上逐状态看（交回镜像的按恢复镜像去重），
    // 结果写进库；看的是从库里读回的流，判段读的是同一份。只排派活计划的那一趟不录（它只要排计划的数，录观察者要真挂载）；叫停了就停在块边上
    // （录过的块在库里，下面第 ① 段照叫停的样子收尾，下一趟接着录）
    let observer_recording = if records && !placement_only && !observers.is_empty() {
        let recording_started = std::time::Instant::now();
        let observer_recording_tally = record_the_observers_of_the_flow(
            &mut store,
            &flow_of_the_library,
            &plan,
            observers,
            &observer_versions,
            &|block| selected.contains(&block),
            &stop_is_requested,
        )?;
        println!(
            "CRASH_AMPLIFICATION_OBSERVERS_RECORDED flow={flow_name} blocks_recorded_here={} states_recorded_here={} blocks_already_recorded={} states_counted_here={} images_observed_fresh={} states_reusing_an_observed_image={} seconds={:.1}",
            observer_recording_tally.blocks_recorded_here,
            observer_recording_tally.states_recorded_here,
            observer_recording_tally.blocks_already_recorded,
            observer_recording_tally.states_counted_here,
            observer_recording_tally.images_observed_here,
            observer_recording_tally.states_reusing_an_observed_image,
            recording_started.elapsed().as_secs_f64()
        );
        Some(observer_recording_tally)
    } else {
        None
    };
    // 只跑录入段的那一趟（连同配置开了 GPU 而这份二进制没编进 gpu 特性、判段报本次未跑的那一趟）：库里那几样已经读回、核过摘要、
    // 照它排出了计划，到这里交回，一块都不判
    if !judges {
        let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &judged_conclusions)
            .map_err(|error| format!("对账失败：{error:?}"))?
            .len();
        drop(store);
        return Ok(EnvironmentRun {
            mode,
            share,
            cards_used: 0,
            recording,
            checking: CheckingTally::default(),
            imported_blocks: 0,
            unjudged_blocks: unjudged,
            violations: Vec::new(),
            coverage_lines: report.lines().to_vec(),
            library_directory,
            verification: None,
            comparison: None,
            gpu_judge: None,
            placement_only: false,
            recorded_only: true,
            stopped_early: false,
            conclusion_versions,
            judged_by,
            computation_routes: routes,
            expand_up_to,
            exhaustive,
            states_reused: states_reused_at_recording,
            ignored_points,
            states_skipped_by_ignore,
            gpu_cards_taken_from: GpuCardsSource::NoCardNeeded,
            facts_verifier_buffers_booked_mebibytes: None,
            facts_verifier_buffers_lease_on_the_books_after_verifying: None,
            registered_segments: registered_segments.clone(),
            registered_segment_domains: registered_segment_domains_at_recording.clone(),
            unregistered_segments_beyond_the_expansion: unregistered_segments_beyond_the_expansion
                .clone(),
            observing: None,
            observer_recording: observer_recording.clone(),
            not_run_without_the_gpu_feature: not_run_without_the_gpu_feature.clone(),
            count_vector_disagreements: Vec::new(),
        });
    }
    // 别的机器判完取回的库：只点名这条流里不归本机这一份的块导进来（import_the_planned_blocks_of_the_other_hosts）
    #[cfg(feature = "gpu")]
    let verifier_digests = [
        FactsVerifierOn::Cpu.verifier_digest(),
        crate::crash_verify_gpu::gpu_verifier_digest(),
    ];
    #[cfg(not(feature = "gpu"))]
    let verifier_digests = [FactsVerifierOn::Cpu.verifier_digest()];
    let imported_blocks = match &import_list {
        Some(list) => import_the_planned_blocks_of_the_other_hosts(
            &mut store,
            repository_root,
            &plan,
            &|block| selected.contains(&block),
            &conclusion_versions,
            observers,
            &observer_conclusion_versions,
            &verifier_digests,
            list,
        )?,
        None => 0,
    };
    let seconds_planning_and_recording = planning_started.elapsed().as_secs_f64();
    // 起过卡池的进程取卡不登记、不起设备，借到哪张卡才在哪张上起（文件头第 5 条）
    let (cards, gpu_cards_taken_from) = if routes.need_a_card() {
        let GpuCardsOfThisRun { cards, taken_from } =
            gpu_cards_in_use(cards_by_priority.as_deref(), host)?;
        (Some(cards), taken_from)
    } else {
        (None, GpuCardsSource::NoCardNeeded)
    };
    // 只排派活计划、被叫停、显存不够的那几趟交回时都带着的
    if placement_only {
        let placement = match (routes.crash_judge, &cards) {
            (ComputedOn::Gpu, Some(cards)) => print_the_placement(
                &store,
                &plan,
                read_back.packed_judge_tables.as_ref().ok_or_else(|| {
                    "配置开了 GPU，判段从库里读回的这一趟却没有 GPU 判表：照这份配置重跑录入段"
                        .to_string()
                })?,
                &|block| selected.contains(&block),
                &judged_conclusions,
                cards,
                gpu_cards_taken_from,
                cards_by_priority.as_deref(),
                host,
                states_reused_at_recording,
            )?,
            (ComputedOn::Gpu, None) | (ComputedOn::Cpu, _) => {
                return Err(
                    "只排派活计划排的是 GPU 判器的活：要配置开着 GPU（ENABLE_GPU=1）".to_string(),
                );
            }
        };
        let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &judged_conclusions)
            .map_err(|error| format!("对账失败：{error:?}"))?
            .len();
        drop(store);
        return Ok(EnvironmentRun {
            mode,
            share,
            cards_used: 0,
            recording,
            checking: CheckingTally::default(),
            imported_blocks,
            unjudged_blocks: unjudged,
            violations: Vec::new(),
            coverage_lines: report.lines().to_vec(),
            library_directory,
            verification: None,
            comparison: None,
            gpu_judge: None,
            placement_only: true,
            recorded_only: false,
            // 起过卡池的进程借不到试判的草稿区：「本次未跑」那一行已经打了，没打 PLACEMENT 行（第 10v 批）
            stopped_early: placement == PlacementOfThisRun::NotRunTheGpuMemoryWasShort,
            conclusion_versions,
            judged_by,
            computation_routes: routes,
            expand_up_to,
            exhaustive,
            // 只排派活计划的那一趟一块都不判、不打 CRASH_AMPLIFICATION 行；复用数照实报（PLACEMENT 行也带）
            states_reused: states_reused_at_recording,
            ignored_points,
            states_skipped_by_ignore,
            gpu_cards_taken_from,
            facts_verifier_buffers_booked_mebibytes: None,
            facts_verifier_buffers_lease_on_the_books_after_verifying: None,
            registered_segments: registered_segments.clone(),
            registered_segment_domains: registered_segment_domains_at_recording.clone(),
            unregistered_segments_beyond_the_expansion: unregistered_segments_beyond_the_expansion
                .clone(),
            observing: None,
            observer_recording: observer_recording.clone(),
            not_run_without_the_gpu_feature: not_run_without_the_gpu_feature.clone(),
            count_vector_disagreements: Vec::new(),
        });
    }
    // 第 ① 段：配了 GPU 只由 GPU 判器判这台机器领的块（判不了就报错，不退回 CPU），CPU 不在这一段判同一批块；没配 GPU 只由 CPU 判器判，
    // 单元两道校验和也在 CPU 上算，这一段不对拍。显存不够（起过卡池的进程此刻借不到、卡上 wgpu 报了 Out of Memory）：
    // 「本次未跑」那一行已经打了，判过的块在库里，下面照叫停的样子交回（文件头第 5 条）
    let judging_started = std::time::Instant::now();
    let (checking, gpu_judge, gpu_memory_was_short) = match (routes.crash_judge, &cards) {
        (ComputedOn::Gpu, Some(cards)) => {
            let tables_read_back = read_back.packed_judge_tables.as_ref().ok_or_else(|| {
                "配置开了 GPU，判段从库里读回的这一趟却没有 GPU 判表：照这份配置重跑录入段"
                    .to_string()
            })?;
            let gpu_judging = judge_my_blocks_on_gpu(
                &mut store,
                &flow_of_the_library,
                &plan,
                tables_read_back,
                &|block| selected.contains(&block),
                &judged_conclusions,
                cards,
                gpu_cards_taken_from,
                &stop_is_requested,
            );
            match gpu_judging? {
                GpuJudgingOfMyBlocks::Judged { tally, outcome } => {
                    eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
                    (tally, Some(outcome), false)
                }
                GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason: _ } => {
                    (tally, None, true)
                }
            }
        }
        (ComputedOn::Gpu, None) => {
            return Err(
                "配置开了 GPU，判器（crash_judge_gpu）走 GPU，这一趟却没有起好的卡：不退回 CPU 判器"
                    .to_string(),
            );
        }
        (ComputedOn::Cpu, _) => {
            let checking = check_selected_blocks(
                &mut store,
                &flow_of_the_library,
                &plan,
                &selected,
                &judged_conclusions,
                &stop_is_requested,
            )
            .map_err(|error| format!("核对失败：{error:?}"))?;
            (checking, None, false)
        }
    };
    let stop_was_requested = gpu_memory_was_short || stop_is_requested();
    // 分片的段（C790）：判完的块已经落盘，进度记录推进到连着判完的地方（被叫停、显存不够提早收尾的那一趟也推进）
    advance_the_progress_of_the_sliced_segments(&mut store, flow_name, &plan, &judged_conclusions)
        .map_err(|error| format!("推进分片进度失败：{error:?}"))?;
    let registered_segment_domains_after_judging = domains_and_progress_of_the_registered_segments(
        &store,
        &flow_of_the_library,
        &plan,
        &registered_segments,
        &judged_conclusions,
    )
    .map_err(|error| format!("读分片进度失败：{error:?}"))?;
    let seconds_judging = judging_started.elapsed().as_secs_f64();
    // 第 ② 段：拿库里读回的事实表核，配置开了 GPU 就在一张卡上核（起过卡池的照账借第一张放得下的，第 ① 段的草稿区已经放掉），
    // 没开 GPU 由 CPU 参照核；被叫停的不核
    let verifying_started = std::time::Instant::now();
    let facts_verifier_route = (!stop_was_requested).then_some(routes.facts_verifier);
    let segment_expansions_read_back = segment_expansions_of_the_plan(&flow_of_the_library, &plan);
    let (
        verification,
        verifying_cards_used,
        facts_verifier_buffers_booked_mebibytes,
        facts_verifier_buffers_lease_on_the_books_after_verifying,
        facts_were_not_verified_for_short_gpu_memory,
    ) = match (facts_verifier_route, &cards) {
        (Some(ComputedOn::Gpu), Some(cards)) => match verify_my_blocks_on_the_first_card(
            &mut store,
            &plan,
            &read_back.facts,
            &segment_expansions_read_back,
            &my_blocks,
            cards,
            gpu_cards_taken_from,
        )? {
            FactsVerifiedOnACard::Verified {
                outcome,
                buffers_booked_mebibytes,
                lease_on_the_books_after_verifying,
            } => (
                Some(outcome),
                1,
                buffers_booked_mebibytes,
                lease_on_the_books_after_verifying,
                false,
            ),
            FactsVerifiedOnACard::NotRunTheGpuMemoryWasShort => (None, 0, None, None, true),
        },
        (Some(ComputedOn::Gpu), None) => {
            return Err(
                "配置开了 GPU，核对（crash_verify_gpu）走 GPU，这一趟却没有起好的卡：不退回 CPU 参照核"
                    .to_string(),
            );
        }
        (None, _) => (None, 0, None, None, false),
        (Some(ComputedOn::Cpu), _) => {
            let on = FactsVerifierOn::Cpu;
            let tally = verify_blocks_from_these_facts(
                &mut store,
                &plan,
                &read_back.facts,
                &segment_expansions_read_back,
                &|block| selected.contains(&block),
                &on,
            )?;
            (
                Some(VerificationOutcome {
                    verifier: on.name(),
                    tally,
                }),
                0,
                None,
                None,
                false,
            )
        }
    };
    if stop_was_requested || facts_were_not_verified_for_short_gpu_memory {
        // 被叫停，或显存不够（本次未跑）：判过的块已经在库里，没做的段留给接着判的那一趟
        let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &judged_conclusions)
            .map_err(|error| format!("对账失败：{error:?}"))?
            .len();
        eprintln!(
            "CRASH_AMPLIFICATION_STOPPED blocks_checked_here={} states_checked_here={} unjudged_blocks={unjudged}",
            checking.blocks_checked, checking.states_checked
        );
        drop(store);
        return Ok(EnvironmentRun {
            mode,
            share,
            cards_used: gpu_judge.as_ref().map_or(0, |outcome| outcome.cards_used),
            recording,
            checking,
            imported_blocks,
            unjudged_blocks: unjudged,
            violations: Vec::new(),
            coverage_lines: report.lines().to_vec(),
            library_directory,
            verification: None,
            comparison: None,
            gpu_judge,
            placement_only: false,
            recorded_only: false,
            stopped_early: true,
            conclusion_versions,
            judged_by,
            computation_routes: routes,
            expand_up_to,
            exhaustive,
            // 被叫停的那一趟照实报它录入时库里已判的（L1）；接着跑的那一趟另报它自己的（reuse_and_scale_fields_of_the_line）
            states_reused: states_reused_at_recording,
            ignored_points,
            states_skipped_by_ignore,
            gpu_cards_taken_from,
            facts_verifier_buffers_booked_mebibytes,
            facts_verifier_buffers_lease_on_the_books_after_verifying,
            registered_segments: registered_segments.clone(),
            registered_segment_domains: registered_segment_domains_after_judging.clone(),
            unregistered_segments_beyond_the_expansion: unregistered_segments_beyond_the_expansion
                .clone(),
            observing: None,
            observer_recording: observer_recording.clone(),
            not_run_without_the_gpu_feature: not_run_without_the_gpu_feature.clone(),
            count_vector_disagreements: Vec::new(),
        });
    }
    // 用了几张卡：第 ① 段在卡上判的按它用的，第 ② 段在卡上核的一张
    let cards_used = gpu_judge
        .as_ref()
        .map_or(0, |outcome| outcome.cards_used)
        .max(verifying_cards_used);
    let seconds_verifying = verifying_started.elapsed().as_secs_f64();
    // 第 ③ 段：核过就比
    let comparing_started = std::time::Instant::now();
    let comparison = match &verification {
        Some(outcome) => {
            let digest = match outcome.verifier {
                "cpu" => FactsVerifierOn::Cpu.verifier_digest(),
                #[cfg(feature = "gpu")]
                "gpu" => crate::crash_verify_gpu::gpu_verifier_digest(),
                other => return Err(format!("认不出的核对方式 {other}")),
            };
            Some(
                compare_the_conclusions_and_the_verifier(
                    &store,
                    &plan,
                    &digest,
                    &judged_conclusions,
                )
                .map_err(|error| format!("第 ③ 段比对失败：{error:?}"))?,
            )
        }
        None => None,
    };
    let seconds_comparing = comparing_started.elapsed().as_secs_f64();
    // 挂在节点上的观察者的判段（J4；用户 2026-10-13 定「观察者改走两段、池级 checker 上 GPU」）：这台机器领的块里落在观察者节点上、
    // 库里没有这一把结论的，从库里读录入段那一块，观察者交回的盘上的池级 checker 配了 GPU 在卡上判、没配由 CPU 判，逐状态拼回一块存
    let observing_started = std::time::Instant::now();
    let observing = if observers.is_empty() {
        None
    } else {
        let pool_checker = pool_checker_of_the_images_after_the_observers(
            routes.crash_judge,
            cards.as_ref(),
            gpu_cards_taken_from,
        )?;
        Some(judge_the_observers_of_the_flow(
            &mut store,
            &plan,
            observers,
            &observer_versions,
            &pool_checker,
            &|block| selected.contains(&block),
        )?)
    };
    let seconds_observing = observing_started.elapsed().as_secs_f64();
    // 计数向量每一趟都判（J7）：这台机器领的每一块，判器那一份结论读回来的计数向量与计划对得上（这一趟判的与复用的一样判）
    let count_vector_disagreements = count_vector_disagreements_of_the_flow(
        &store,
        &flow_of_the_library,
        &plan,
        &judged_conclusions,
        &|block| selected.contains(&block),
    )
    .map_err(|error| format!("读计数向量失败：{error:?}"))?;
    let reconciling_started = std::time::Instant::now();
    let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &judged_conclusions)
        .map_err(|error| format!("对账失败：{error:?}"))?
        .len();
    let violations = read_the_violations_stored_under(&store, &plan, &judged_conclusions)
        .map_err(|error| format!("读违例失败：{error:?}"))?;
    let coverage_lines = report.lines().to_vec();
    // 一趟的挂钟花在哪一段：排计划与录入、导入（CPU），第 ① 段判（GPU 判的含建表、表上卡、编内核、拆结论、写库），
    // 第 ② 段核（含抽事实表），第 ③ 段比与对账读违例（CPU）
    eprintln!(
        "CRASH_AMPLIFICATION_PHASES states_checked_here={} seconds_planning_and_recording={seconds_planning_and_recording:.1} seconds_judging={seconds_judging:.1} seconds_verifying={seconds_verifying:.1} seconds_comparing={seconds_comparing:.1} seconds_observing={seconds_observing:.1} seconds_reconciling={:.1}",
        checking.states_checked,
        reconciling_started.elapsed().as_secs_f64()
    );
    drop(store);
    Ok(EnvironmentRun {
        mode,
        share,
        cards_used,
        recording,
        checking,
        imported_blocks,
        unjudged_blocks: unjudged,
        violations,
        coverage_lines,
        library_directory,
        verification,
        comparison,
        gpu_judge,
        placement_only: false,
        recorded_only: false,
        stopped_early: false,
        conclusion_versions,
        judged_by,
        computation_routes: routes,
        expand_up_to,
        exhaustive,
        states_reused: states_reused_at_recording,
        ignored_points,
        states_skipped_by_ignore,
        gpu_cards_taken_from,
        facts_verifier_buffers_booked_mebibytes,
        facts_verifier_buffers_lease_on_the_books_after_verifying,
        registered_segments: registered_segments.clone(),
        registered_segment_domains: registered_segment_domains_after_judging,
        unregistered_segments_beyond_the_expansion: unregistered_segments_beyond_the_expansion
            .clone(),
        observing,
        observer_recording,
        not_run_without_the_gpu_feature,
        count_vector_disagreements,
    })
}

// ============================== 给定状态表：随机历史两项的入口（崩溃点重放合并第三轮判决 J3、J5、J6、J8；用户 2026-10-10 定的四件） ==============================

/// 随机历史的一项（第三轮正文 H3「随机历史两项（崩溃注入快档、读故障态不持有见证）加「给定状态表」一种展开，也是这个入口与这个判定库里的项」）：
/// 项名、设点文件（这一项的测试文件，仓内路径：CPU 结论的版本按它出发的闭包 ∪ CPU 判器闭包取）、种子与规模参数的写法
/// （H8「随机历史两项的键 = 种子 + 规模参数 + 测试文件出发的整份闭包，判定按段存」：种子与规模进块键，闭包进版本）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GivenStatesItem {
    pub item_name: String,
    pub setting_file: String,
    pub seeds_and_scale: String,
}

/// 往 GPU 那一侧塞的已知故障：只给证明 GPU 判与 CPU 判两趟分开跑、比结论会红的那条用例用，产品路径恒是 [`FaultPutOnTheGpuSide::NoFault`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPutOnTheGpuSide {
    NoFault,
    /// 每张表第一个状态的 GPU 结论里第一条不变量（`IMPLEMENTED_INVARIANTS[0]`）的违例位翻过来，判到位一起置上。
    FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable,
}

/// 给定状态表这一趟的账（随 [`GivenStatesEngine::finish`] 交回，那一刻打 [`GIVEN_STATES_LINE_PREFIX`] 那一行）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GivenStatesTally {
    pub tables_judged_here: u64,
    pub tables_reused: u64,
    pub states_judged_by_the_gpu_here: u64,
    pub states_judged_by_the_cpu_here: u64,
    pub states_reused: u64,
    /// GPU 判不了的表（建表交了拒因：哪张表满了）或状态（结论里判不了那一位），一条一句：照第三轮正文 H6 记一条不一致、判红。
    pub states_the_gpu_could_not_judge: Vec<String>,
    /// 两台分片时归别的机器判、这一趟库里又没有的表（第三轮正文 H5「配了双机就两台分片」）：这一趟交回的是占位的结论，不作数，
    /// 要等那一台判完、库取回来导入之后再跑一趟（[`GivenStatesRun::left_tables_to_the_other_hosts`]）。
    pub tables_left_to_the_other_hosts: u64,
    /// 判到归别的机器的表时，从别的机器取回的库（`SINGLEFS_CRASH_AMPLIFICATION_IMPORT`）点名导入了几块结论：一张表一种判器一块，
    /// 写下的加本库已有且相同的，对方库里没有的不算（[`GivenStatesEngine::import_the_table_from_the_other_hosts`]）。
    pub imported_blocks: u64,
    /// 这一趟只跑录入段（[`RUN_STAGE_ENVIRONMENT_VARIABLE`]`=record`）：写进库、没判的表数（交的是占位结论，[`GivenStatesRun::recorded_only`]）。
    pub tables_recorded_only: u64,
    /// 录入段写进库的表数（只录与两段先后跑都算；只判的那一趟是 0）。
    pub tables_recorded_here: u64,
    /// 配置开了 GPU 而这份二进制没编进 gpu 特性：判段不跑、交占位结论的表数（[`GivenStatesRun::not_run_without_the_gpu_feature`]）。
    pub tables_not_judged_without_the_gpu_feature: u64,
}

/// [`GivenStatesEngine::finish`] 交回的这一趟。
#[derive(Clone, Debug, PartialEq)]
pub struct GivenStatesRun {
    pub mode: CrashAmplificationRunMode,
    /// 这一趟跑哪一段（[`RUN_STAGE_ENVIRONMENT_VARIABLE`]）。
    pub stage: RunStage,
    pub share: HostShare,
    pub judged_by: JudgedBy,
    pub gpu_cards_taken_from: GpuCardsSource,
    pub library_directory: PathBuf,
    /// 这一趟见过叫停（[`stop_is_requested_by_the_environment`]，照流水线那一侧的叫停层：见过一次就记住）：崩溃注入不再领新的一片，
    /// 判过的表在库里；调用方照「本次未跑」交回。
    pub stopped_early: bool,
    /// 配置开了 GPU 而这份二进制没编进 gpu 特性时判段不跑的原因（[`judge_not_run_without_the_gpu_feature`]）：表照录、一张不判，开跑时打了
    /// 本次未跑那一行（[`given_states_not_run_line_without_the_gpu_feature`]），调用方照 [`GivenStatesRun::recorded_only`] 交回。
    pub not_run_without_the_gpu_feature: Option<String>,
    pub tally: GivenStatesTally,
}

impl GivenStatesRun {
    /// 这一趟有表归别的机器判、库里又没有（两台分片、还没导入那一台的库）：交回的结论里有占位的，调用方不照它判，照「本次未跑」交回。
    #[must_use]
    pub fn left_tables_to_the_other_hosts(&self) -> bool {
        self.tally.tables_left_to_the_other_hosts > 0
    }

    /// 这一趟只跑了录入段（[`RUN_STAGE_ENVIRONMENT_VARIABLE`]`=record`，或配置开了 GPU 而这份二进制没编进 gpu 特性、判段不跑）：
    /// 表都写进了库、一张没判，交回的结论是占位的，调用方不照它判，照「本次未跑」交回，等判段那一趟（`judge`，或不设、两段先后跑；
    /// 配了 GPU 的是带特性那一趟）从库里读回判。
    #[must_use]
    pub fn recorded_only(&self) -> bool {
        self.stage == RunStage::RecordOnly || self.not_run_without_the_gpu_feature.is_some()
    }

    /// 这一趟判红的那几样（GPU 判不了）一条一句；空就是不红（叫停、分片留给别的机器另看 [`GivenStatesRun::stopped_early`]、
    /// [`GivenStatesRun::left_tables_to_the_other_hosts`]）。
    #[must_use]
    pub fn red_items(&self) -> Vec<String> {
        self.tally
            .states_the_gpu_could_not_judge
            .iter()
            .map(|state| format!("GPU 判不了：{state}"))
            .collect()
    }
}

/// 给定状态表那一路每一趟收尾打的一行的头（项名、判器、对照、表数与态数、比不上与判不了的条数）。
pub const GIVEN_STATES_LINE_PREFIX: &str = "CRASH_POINT_REPLAY_GIVEN_STATES";

/// 配了 GPU 而给定状态表的判器借不到显存时打的那一行的头（运行条件不满足，入口照它退 78；不打「本次未跑」那一行、不换成 CPU 判器的结论往下走）。
pub const GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX: &str =
    "CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET";

/// 两台分片、导入了别的机器的库之后，归那一台判的表库里还是没有时报错的那一句的头（对账不为空，判红）。
pub const GIVEN_STATES_TABLE_JUDGED_BY_NO_HOST_PREFIX: &str =
    "CRASH_POINT_REPLAY_GIVEN_STATES_TABLE_JUDGED_BY_NO_HOST";

/// 给定状态表交崩溃点重放的引擎判（崩溃点重放合并第三轮判决 J3、J5、J6、J8）：照本机多机配置的 GPU 开关选路（没配走 CPU 判器、配了走 GPU 判器，
/// 卡用不上、借不到显存都报错、不退回 CPU）；两台分片照流水线那一侧的做法（`SINGLEFS_CRASH_AMPLIFICATION_SHARE` 领哪一份、
/// `…_HOST_WEIGHTS` 权重，一张表一块、按块键的输入指纹归一台；`…_IMPORT` 是别的机器取回的库，开跑时打开，判到归别的机器的表时
/// 从那几份库里点名导入这一张表的两种判器结论，[`GivenStatesEngine::import_the_table_from_the_other_hosts`]）；叫停照流水线那一侧的叫停层
/// （[`stop_is_requested_by_the_environment`]，见过一次就记住，崩溃注入照它不再领新的一片）；配了 GPU 只由 GPU 判器判、没配只由 CPU 判器判，
/// 每张表先写进判定库、再从库里读回来判（用户 2026-10-11 定：一段里 CPU 不既跑数据又对拍，段与段之间不在内存里交数据）；结论按（块键，结论种类，版本）存进判定库，一张表一块（H8「判定按段存」），
/// 同一把版本下判过的表照库里的复用。GPU 判不了的表（建不出表：掩码装不下、落点版本太多……）照 H6 记一条不一致、判红，这几个状态的结论取 CPU 判器的
/// （崩溃注入照它往下走，这一趟已经红了）。几条崩溃注入的工作线程共用一个：库与账各一把锁。
pub struct GivenStatesEngine {
    item: GivenStatesItem,
    /// 这一趟跑哪一段（用户 2026-10-13 定「给定状态引擎认选段」）：`record` 只把表写进库、按项名与表名登清单，`judge` 只凭项名与表名从库里读回判，
    /// 不设两段先后跑、中间照样经库。
    stage: RunStage,
    mode: CrashAmplificationRunMode,
    judged_by: JudgedBy,
    /// 配置开了 GPU 而这份二进制没编进 gpu 特性：判段不跑（表照录、交占位结论），见 [`GivenStatesRun::not_run_without_the_gpu_feature`]。
    not_run_without_the_gpu_feature: Option<String>,
    gpu_judge_version: ConclusionVersion,
    cpu_judge_version: ConclusionVersion,
    library: LibraryOpenInThisProcess,
    library_directory: PathBuf,
    #[cfg_attr(
        not(feature = "gpu"),
        allow(
            dead_code,
            reason = "没编进 gpu 特性时选路走不到 GPU，卡只在编进来时用"
        )
    )]
    cards: Option<CardsInUse>,
    gpu_cards_taken_from: GpuCardsSource,
    share: HostShare,
    host_shares: WeightedShares,
    /// 别的机器取回的库（`SINGLEFS_CRASH_AMPLIFICATION_IMPORT`，开跑时打开、这个进程里同一个目录共用一份）：不是空的时，归别的机器判的表
    /// 先从这几份库点名导入，导入之后库里还没有就判红（对账不为空），不再交占位的结论。
    libraries_to_import_from: Vec<LibraryOpenInThisProcess>,
    stop_request: StopRequestRememberedInThisRun<'static>,
    /// 崩溃注入问叫停时这一趟答过「叫停了」（之后它不再领新的一片）：收尾照它报 `stopped_early`，不在收尾那一刻再读一次叫停文件。
    answered_a_stop_request: std::sync::atomic::AtomicBool,
    #[cfg_attr(
        not(feature = "gpu"),
        allow(
            dead_code,
            reason = "没编进 gpu 特性时 GPU 那一侧不判，塞故障只在编进来时用"
        )
    )]
    fault_put_on_the_gpu_side: FaultPutOnTheGpuSide,
    #[cfg_attr(
        not(feature = "gpu"),
        allow(
            dead_code,
            reason = "没编进 gpu 特性时不在卡上判，轮着用卡只在编进来时用"
        )
    )]
    next_card: std::sync::atomic::AtomicUsize,
    /// 这一趟录入段登过的清单名与它指的表摘要（[`GivenStatesEngine::record_the_table`] 照它拒清单名分不开的两张表）。
    manifests_recorded_in_this_run: std::sync::Mutex<BTreeMap<String, [u8; DIGEST_BYTES]>>,
    tally: std::sync::Mutex<GivenStatesTally>,
}

/// GPU 判器判一张给定状态表的结局。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时不在卡上判，这几种结局只在编进来时造"
    )
)]
enum GivenStatesOnTheGpu {
    /// 判完：逐状态的对拍字（oracle 的位已去掉）、池级 checker 判 I-7.4 时违例细节里点名的被抛弃根、判不了的那几个状态判不了的原因
    /// （原因码与站点号，`crate::crash_judge_gpu::undecidable_reasons_of_the_gpu_verdict`；判得了的是 None）与在哪张卡上判的。
    Judged {
        words: Vec<CrossCheckWords>,
        abandoned_roots_named_of_each_state: Vec<Vec<AbandonedRootNamedByThePoolChecker>>,
        undecidable_reason_of_each_state: Vec<Option<String>>,
        judged_on: JudgingDevice,
    },
    /// 建不出这张表（[`crate::crash_judge_tables::JudgeTablesRefusal`] 的写法）：H6 的「哪张表满了」。
    TablesRefused { refusal: String },
}

impl GivenStatesEngine {
    /// 照环境起一趟（读法同 [`run_from_environment`]：多机配置、库目录、显卡先后）。
    ///
    /// # Errors
    /// 同 [`GivenStatesEngine::start_from_environment_putting_a_fault_on_the_gpu_side`]。
    pub fn start_from_environment(
        item: GivenStatesItem,
        toolchain: &CrashPointReplayToolchainIdentity,
        repository_root: &Path,
    ) -> Result<Self, String> {
        Self::start_from_environment_putting_a_fault_on_the_gpu_side(
            item,
            toolchain,
            repository_root,
            FaultPutOnTheGpuSide::NoFault,
        )
    }

    /// 同 [`GivenStatesEngine::start_from_environment`]，GPU 那一侧塞 `fault`（证明 GPU 与 CPU 两趟分开跑、比结论会红的那条用例用）。
    ///
    /// # Errors
    /// 多机配置读不了或写坏了；配置开了 GPU 而这份二进制没有 `gpu` 特性、或这一趟上不了卡（卡用不上不退回 CPU）；结论的版本与编译时对不上
    /// （打 [`CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX`] 那一行，整趟不写库）；库目录落在开机会清空的地方、库打不开；分片与权重写法错
    /// （[`host_share_and_weights_from_the_environment`]）；要导入的库打不开、就是本机这一份库（导入失败在判到那张表时报，
    /// [`GivenStatesEngine::import_the_table_from_the_other_hosts`]）。
    pub fn start_from_environment_putting_a_fault_on_the_gpu_side(
        item: GivenStatesItem,
        toolchain: &CrashPointReplayToolchainIdentity,
        repository_root: &Path,
        fault: FaultPutOnTheGpuSide,
    ) -> Result<Self, String> {
        let stage = RunStage::from_the_environment()?;
        let configuration_file: Option<PathBuf> = multi_host_configuration_file(repository_root)?;
        let mode = match &configuration_file {
            Some(configuration) => {
                CrashAmplificationRunMode::from_configuration_file(configuration)
            }
            None => CrashAmplificationRunMode::from_configuration_file_or_environment(
                &repository_root.join("multi-host.env"),
            ),
        };
        let cards_by_priority =
            cards_by_priority_from_configuration_or_environment(configuration_file.as_deref())?;
        // 配了 GPU 而这份二进制没编进 gpu 特性（提交时快档不带特性那一趟）：判段这一趟不跑、报本次未跑（不判红、不退回 CPU 判），录入段照录、
        // 表写进库，带特性那一趟从库里读回、在卡上判（用户 2026-10-13 定「配 GPU 必用 GPU、做不到报错不降级」，与 run_from_environment 同一种写法）
        let not_run_without_the_gpu_feature =
            judge_not_run_without_the_gpu_feature(mode.uses_gpu(), cfg!(feature = "gpu"));
        if let Some(reason) = &not_run_without_the_gpu_feature {
            println!(
                "{}",
                given_states_not_run_line_without_the_gpu_feature(&item.item_name, reason)
            );
        }
        let routes = computation_routes(
            mode.uses_gpu() && not_run_without_the_gpu_feature.is_none(),
            cfg!(feature = "gpu"),
        )?;
        let (share, host_weights) = host_share_and_weights_from_the_environment()?;
        // 配了 GPU 只由 GPU 判器判，没配只由 CPU 判器判：同一批状态这一趟只判一次，不在同一段里对拍；判段不跑的那一趟记配置的那一路（GPU）
        let judged_by = match (routes.crash_judge, &not_run_without_the_gpu_feature) {
            (ComputedOn::Cpu, None) => JudgedBy::CpuJudge,
            (ComputedOn::Gpu, None) | (ComputedOn::Cpu | ComputedOn::Gpu, Some(_)) => {
                JudgedBy::GpuJudge
            }
        };
        let host = CardHost::of_this_process()?;
        // 结论的版本（J8）开库之前算、与编译时写进二进制的那一份核，对不上整趟不写库（同 ConclusionVersionsOfThePlan::of_the_plan）
        let versions = conclusion_code_entries_differing_from_the_build(repository_root)
            .map_err(
                |error| ConclusionVersionsNotWritable::NotComputableFromTheTree {
                    reason: error.reason,
                },
            )
            .and_then(|differing| {
                if differing.is_empty() {
                    Ok(())
                } else {
                    Err(ConclusionVersionsNotWritable::TheTableDiffersFromTheBuild {
                        entries: differing.iter().map(|entry| format!("{entry:?}")).collect(),
                    })
                }
            })
            .and_then(|()| {
                let gpu_judge = conclusion_version_to_write_under(
                    repository_root,
                    &ConclusionVersionRequest::GpuJudge,
                    toolchain,
                )?;
                let cpu_judge = conclusion_version_to_write_under(
                    repository_root,
                    &ConclusionVersionRequest::CpuJudge {
                        setting_file: &item.setting_file,
                    },
                    toolchain,
                )?;
                Ok((gpu_judge, cpu_judge))
            });
        let (gpu_judge_version, cpu_judge_version) = match versions {
            Ok(versions) => versions,
            Err(not_writable) => {
                let line = not_writable.not_written_line();
                println!("{line}");
                return Err(line);
            }
        };
        // 库：入口脚本按趟交的目录（SINGLEFS_CRASH_AMPLIFICATION_LIBRARY），没交就是家目录下给定状态表那一类的长期库；没有临时库
        let library_directory =
            crash_library_directory_of(CrashLibraryClass::GivenStates, repository_root)?;
        let library = LibraryOpenInThisProcess::open_or_share(
            &library_directory,
            LibraryRole::ThisHostsLibrary,
        )?;
        // 别的机器判完、拷回来的库（同流水线那一侧的 SINGLEFS_CRASH_AMPLIFICATION_IMPORT）：开跑时只打开，不整库导入；判到归那一台的表时
        // 才点名导入那一张（这一趟要判哪几张表开跑时不知道，崩溃注入跑到哪张问哪张）
        let mut libraries_to_import_from = Vec::new();
        if let Ok(list) = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_IMPORT") {
            for directory in list.split(':').filter(|directory| !directory.is_empty()) {
                let other = LibraryOpenInThisProcess::open_or_share(
                    &path_relative_to_repository_root(repository_root, directory),
                    LibraryRole::LibraryToImportFrom,
                )?;
                // 同一份库在这个进程里只开一次、几个引擎共用：要导入的就是本机这一份时，导入那一刻两把锁是同一把，拒绝（整库导入时开第二次撞 RocksDB 的锁，也是开跑时报错）
                if other.key == library.key {
                    return Err(format!(
                        "要导入的库 {directory} 就是本机这一份库 {}",
                        library_directory.display()
                    ));
                }
                libraries_to_import_from.push(other);
            }
        }
        let (cards, gpu_cards_taken_from) = if routes.crash_judge == ComputedOn::Gpu {
            let GpuCardsOfThisRun { cards, taken_from } =
                gpu_cards_in_use(cards_by_priority.as_deref(), host)?;
            (Some(cards), taken_from)
        } else {
            (None, GpuCardsSource::NoCardNeeded)
        };
        Ok(Self {
            item,
            stage,
            mode,
            judged_by,
            not_run_without_the_gpu_feature,
            gpu_judge_version,
            cpu_judge_version,
            library,
            library_directory,
            cards,
            gpu_cards_taken_from,
            share,
            host_shares: WeightedShares::new(host_weights),
            libraries_to_import_from,
            stop_request: stop_request_remembering_the_stop_file_of_the_engine_start(),
            answered_a_stop_request: std::sync::atomic::AtomicBool::new(false),
            fault_put_on_the_gpu_side: fault,
            next_card: std::sync::atomic::AtomicUsize::new(0),
            manifests_recorded_in_this_run: std::sync::Mutex::new(BTreeMap::new()),
            tally: std::sync::Mutex::new(GivenStatesTally::default()),
        })
    }

    /// 收尾：关库，打 [`GIVEN_STATES_LINE_PREFIX`] 那一行，交回这一趟。
    #[must_use]
    pub fn finish(self) -> GivenStatesRun {
        let tally = self
            .tally
            .into_inner()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let stopped_early = self
            .answered_a_stop_request
            .load(std::sync::atomic::Ordering::SeqCst);
        drop(self.libraries_to_import_from);
        drop(self.library);
        println!(
            "{GIVEN_STATES_LINE_PREFIX} item={} stage={} mode={} judged_by={} tables_recorded_here={} tables_recorded_only={} tables_judged_here={} tables_reused={} states_judged_by_the_gpu_here={} states_judged_by_the_cpu_here={} states_reused={} states_the_gpu_could_not_judge={} share={}/{} imported_blocks={} tables_left_to_the_other_hosts={} stopped_early={}",
            self.item.item_name,
            self.stage.name(),
            self.mode.name(),
            self.judged_by.conclusion_kind().version_text(),
            tally.tables_recorded_here,
            tally.tables_recorded_only,
            tally.tables_judged_here,
            tally.tables_reused,
            tally.states_judged_by_the_gpu_here,
            tally.states_judged_by_the_cpu_here,
            tally.states_reused,
            tally.states_the_gpu_could_not_judge.len(),
            self.share.index,
            self.share.count,
            tally.imported_blocks,
            tally.tables_left_to_the_other_hosts,
            stopped_early
        );
        GivenStatesRun {
            mode: self.mode,
            stage: self.stage,
            share: self.share,
            judged_by: self.judged_by,
            gpu_cards_taken_from: self.gpu_cards_taken_from,
            library_directory: self.library_directory,
            stopped_early,
            not_run_without_the_gpu_feature: self.not_run_without_the_gpu_feature,
            tally,
        }
    }

    /// 这张表归不归这台机器判：块键的输入指纹头 8 个字节当块号，照几台机器的权重平滑轮转（[`WeightedShares::owner_of_block`]，
    /// 与流水线那一侧同一种分法；几台各算各的、不用商量）。
    fn owns_the_table(&self, key: &VerdictBlockKey) -> bool {
        let block_number = u64::from_le_bytes(
            key.input_fingerprint.0[..8]
                .try_into()
                .expect("输入指纹是 SHA-256，头 8 个字节总在"),
        );
        self.host_shares.owner_of_block(block_number) == self.share.index
    }

    fn version_of(&self, judged_by: JudgedBy) -> ConclusionVersion {
        match judged_by {
            JudgedBy::GpuJudge => self.gpu_judge_version,
            JudgedBy::CpuJudge => self.cpu_judge_version,
        }
    }

    fn note(&self, change: impl FnOnce(&mut GivenStatesTally)) {
        change(
            &mut self
                .tally
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
    }

    fn store(&self) -> std::sync::MutexGuard<'_, VerdictStore> {
        self.library.lock()
    }

    /// 一张归别的机器判的表：从开跑时打开的那几份别的机器取回的库里点名导入它的两种判器结论（GPU 判器、CPU 判器，各在这一趟的那一把版本下；
    /// 对方库里没有的跳过，本库已有且相同的不写，不同的报错），导入的块数记进账的 `imported_blocks`。只点这一张表的块键，导入的读写随这一趟
    /// 问到的、归别的机器的表数线性，不随那几份库里累计的别的项、别的版本的行涨（[`VerdictStore::import_the_planned_rows_from`]）。
    /// 先拿本机库的锁、再逐份拿要导入的库的锁（几个引擎都是这个次序）。
    ///
    /// # Errors
    /// 导入失败：同一（块键，种类，版本）两边结论不同、库读写失败这类。
    fn import_the_table_from_the_other_hosts(&self, key: &VerdictBlockKey) -> Result<(), String> {
        let planned = crate::verdict_store::PlannedImportKeys {
            conclusions: [JudgedBy::GpuJudge, JudgedBy::CpuJudge]
                .into_iter()
                .map(|judged_by| {
                    (
                        key.clone(),
                        judged_by.conclusion_kind(),
                        self.version_of(judged_by),
                    )
                })
                .collect(),
            paths: Vec::new(),
            verifier_blocks: Vec::new(),
        };
        let mut imported_blocks = 0u64;
        {
            let mut store = self.store();
            for other in &self.libraries_to_import_from {
                let tally = store
                    .import_the_planned_rows_from(&other.lock(), &planned)
                    .map_err(|error| {
                        format!(
                            "从 {} 导入给定状态表的结论失败：{error:?}",
                            other.key.display()
                        )
                    })?;
                imported_blocks +=
                    tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;
            }
        }
        self.note(|tally| tally.imported_blocks += imported_blocks);
        Ok(())
    }

    /// 一张给定状态表的块键：输入指纹是项（名字、种子与规模）、表名、基线镜像、写表（逐个写的盘、种类、FUA、偏移、长度、内容摘要与段号）、
    /// 接缝、每个持久集合的摘要与它那一栏恢复自称的那一版、这张表那一栏容器容量旋钮值——表的输入一样才复用（同名同写表、旋钮值或
    /// 恢复自称的那一版不同的两张表各是一块，不互相复用）；
    /// 节点名是 `<设点文件>::<项名>::<表名>`；计划哈希写状态数、写数与接缝数；一张表一块。
    ///
    /// # Errors
    /// 节点名超过判定库收的长度。
    fn block_key_of_the_table(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<VerdictBlockKey, String> {
        let table = table_with_its_columns.table;
        let mut segment_of_write: BTreeMap<usize, usize> = BTreeMap::new();
        for (segment_index, segment) in table.segments.iter().enumerate() {
            for write in segment {
                segment_of_write.insert(*write, segment_index);
            }
        }
        let [tombstone_knob, lineage_knob, retirement_knob] =
            records_per_container_of_each_knob(table_with_its_columns.container_capacity_knobs);
        let mut input = format!(
            "given-states-input-v3\nitem={}\nseeds_and_scale={}\ntable={}\nbase={}\ncontainer_capacity_knobs={tombstone_knob},{lineage_knob},{retirement_knob}\n",
            self.item.item_name,
            self.item.seeds_and_scale,
            table.table_name,
            hexadecimal_text(&SectorImage::of_base(table.base).digest())
        );
        for (write_index, write) in table.writes.iter().enumerate() {
            input.push_str(&judge_input_line_of_a_write(
                write,
                segment_of_write.get(&write_index).copied(),
            ));
            input.push('\n');
        }
        for seam in table.seams {
            input.push_str(&format!("seam={seam:?}\n"));
        }
        for (position, persisted) in table.persisted_sets.iter().enumerate() {
            input.push_str("persisted=");
            input.extend(
                persisted
                    .iter()
                    .map(|is_persisted| if *is_persisted { '1' } else { '0' }),
            );
            input.push('\n');
            input.push_str(&claimed_version_line_of_the_block_key(
                table_with_its_columns
                    .claimed_versions
                    .of_the_state(position),
            ));
            input.push('\n');
        }
        let node_name = format!(
            "{}::{}::{}",
            self.item.setting_file, self.item.item_name, table.table_name
        );
        Ok(VerdictBlockKey {
            input_fingerprint: InputFingerprint(sha256_digest(input.as_bytes())),
            stream_or_node_name: StreamOrNodeName::new(&node_name)
                .map_err(|_rejected| format!("给定状态表的节点名太长：{node_name}"))?,
            enumeration_plan_hash: EnumerationPlanHash(sha256_digest(
                format!(
                    "given-states-plan-v1\nstates={}\nwrites={}\nseams={}\n",
                    table.persisted_sets.len(),
                    table.writes.len(),
                    table.seams.len()
                )
                .as_bytes(),
            )),
            block_start: BlockStartStateIndex(0),
        })
    }

    /// 库里这一块在 `judged_by` 那一把版本下的结论读回成逐状态的对拍字（判到位不存，读回是 0）与计数向量；没存过是 `None`。
    ///
    /// # Errors
    /// 库读失败；存着的块不是这一路写得出的（红项认不得、没带恢复字、状态数与表不等）。
    fn stored_conclusions(
        &self,
        key: &VerdictBlockKey,
        judged_by: JudgedBy,
        state_count: usize,
    ) -> Result<Option<GivenStatesConclusionsOfATable>, String> {
        let store = self.store();
        let Some(stored) = store
            .read_conclusion_block(
                key,
                &judged_by.conclusion_kind(),
                &self.version_of(judged_by),
            )
            .map_err(|error| format!("读不了给定状态表的结论：{error:?}"))?
        else {
            return Ok(None);
        };
        let numbers = stored.block.verdict_vector_number_of_each_state();
        let recovery_words = stored
            .recovery_words
            .as_ref()
            .ok_or_else(|| "库里给定状态表的结论没带恢复字".to_string())?;
        let abandoned_roots_named_of_each_state = stored
            .abandoned_roots_named_by_the_pool_checker
            .as_ref()
            .ok_or_else(|| "库里给定状态表的结论没带 I-7.4 点名的被抛弃根".to_string())?;
        if numbers.len() != state_count
            || recovery_words.len() != state_count
            || abandoned_roots_named_of_each_state.len() != state_count
        {
            return Err(format!(
                "库里给定状态表的结论有 {} 个状态、{} 份恢复字、{} 份点名的被抛弃根，这张表 {state_count} 个状态",
                numbers.len(),
                recovery_words.len(),
                abandoned_roots_named_of_each_state.len()
            ));
        }
        let mut states = Vec::with_capacity(state_count);
        for ((number, words), abandoned_roots_named) in numbers
            .iter()
            .zip(recovery_words)
            .zip(abandoned_roots_named_of_each_state)
        {
            let vector = store
                .verdict_vector_of_number(*number)
                .ok_or_else(|| format!("库里没有判定向量 {}", number.as_u8()))?;
            states.push(GivenStateConclusion {
                words: cross_check_words_of_a_stored_given_state(
                    &red_items_of_the_encoded_verdict(vector),
                    *words,
                )?,
                pool_checker_violation_details: Vec::new(),
                abandoned_roots_named_by_the_pool_checker: abandoned_roots_named.clone(),
            });
        }
        Ok(Some(GivenStatesConclusionsOfATable {
            states,
            count_vector: stored.count_vector,
            source: GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
        }))
    }

    /// 一张表在 `judged_by` 那一把版本下的结论存成一块（红项、违例状态、计数向量、恢复字与在哪判的一批写下去）。
    ///
    /// # Errors
    /// 判定向量超过 256 种、这个（块键，种类，版本）下已经存着不同的结论、库写失败。
    fn store_the_conclusions(
        &self,
        key: &VerdictBlockKey,
        judged_by: JudgedBy,
        conclusions: &GivenStatesConclusionsOfATable,
        judged_on: JudgingDevice,
    ) -> Result<(), String> {
        let mut store = self.store();
        let mut numbers = Vec::with_capacity(conclusions.states.len());
        let mut violating_states = Vec::new();
        for (offset, state) in conclusions.states.iter().enumerate() {
            let items = red_items_of_a_given_state(&state.words);
            if !items.is_empty() {
                violating_states.push(StateOffsetInBlock(
                    u64::try_from(offset).expect("块内序号装得进 u64"),
                ));
            }
            numbers.push(
                store
                    .register_verdict_vector(&encoded_verdict_of_the_red_items(&items))
                    .map_err(|error| format!("登记不了判定向量：{error:?}"))?,
            );
        }
        let block = VerdictBlock::new(numbers)
            .map_err(|_empty| "给定状态表一个状态都没有，不存".to_string())?;
        let abandoned_roots_named_of_each_state: Vec<Vec<AbandonedRootNamedByThePoolChecker>> =
            conclusions
                .states
                .iter()
                .map(|state| state.abandoned_roots_named_by_the_pool_checker.clone())
                .collect();
        store
            .store_conclusion_block_naming_the_abandoned_roots(
                key,
                &judged_by.conclusion_kind(),
                &self.version_of(judged_by),
                &ConclusionBlockRecord {
                    block,
                    violating_states,
                    count_vector: conclusions.count_vector.clone(),
                    judged_on,
                    recovery_words: Some(
                        conclusions
                            .states
                            .iter()
                            .map(|state| recovery_words_of_the_cross_check_words(&state.words))
                            .collect(),
                    ),
                },
                &abandoned_roots_named_of_each_state,
            )
            .map_err(|error| format!("存不了给定状态表的结论：{error:?}"))?;
        Ok(())
    }

    /// GPU 判器判一张给定状态表：照 mkfs 之后的基线与这张表的写表、切段、接缝建表（每段不展开，状态是给定的持久掩码，按枚举写表补齐撕裂镜像
    /// 与重放那几位：随机历史不摆撕裂态，全是没持久），在一张卡上判（这一趟自己起的卡轮着用；起过卡池的照账借第一张此刻放得下的）。
    ///
    /// # Errors
    /// 起过卡池的进程借不到判器的显存（此刻借不到、借到的卡上起设备时显存不够：运行条件不满足，打
    /// [`GIVEN_STATES_RUN_CONDITION_NOT_MET_LINE_PREFIX`] 那一行，不报本次未跑、不换成 CPU 判器的结论往下走）、要的比每张卡空着时能借的都大、
    /// 卡上的显存读不出；这一趟没有起好的卡。
    #[cfg(feature = "gpu")]
    fn judge_the_table_on_the_gpu(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesOnTheGpu, String> {
        use crate::crash_judge_conclusions::cross_check_words_without_the_oracle;
        use crate::crash_judge_dispatch::{
            mebibytes_a_judge_needs, states_per_scratch_lane, GpuCrashJudge, PackedJudgeTables,
        };
        use crate::crash_judge_gpu::{
            abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict,
            cross_check_words_of_gpu_verdict, undecidable_reasons_of_the_gpu_verdict,
            JudgeKernelSources,
        };
        let table = table_with_its_columns.table;
        // 扫描内核照这张表那一栏旋钮值编（没填是产品容量那一份，与判法摘要罩的那三份逐字相同）
        let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(
            table_with_its_columns.container_capacity_knobs,
        );
        use crate::crash_judge_tables::{
            build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions,
            JudgeFlow,
        };
        let not_expanded = |_segment_index: usize, _segment: &[usize]| {
            CrashPointReplaySegmentExpansion::NotExpanded
        };
        let enumerated_write_count = crate::crash::crash_state_enumeration_tables(
            table.base,
            table.writes,
            table.segments,
            &not_expanded,
        )
        .enumerated_writes
        .len();
        let masks: Vec<Vec<bool>> = table
            .persisted_sets
            .iter()
            .map(|persisted| {
                let mut mask = persisted.clone();
                mask.resize(enumerated_write_count.max(persisted.len()), false);
                mask
            })
            .collect();
        // 每个状态那一栏恢复自称的那一版跟着进判表（恢复内核的记录核对器第二条判据照它取那一版）
        let claimed_versions = table_with_its_columns
            .claimed_versions
            .of_every_state(table.persisted_sets.len());
        let tables = match build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams_with_their_claimed_versions(
            &JudgeFlow {
                base: table.base,
                writes: table.writes,
                segments: table.segments,
                judged_root_index: 0,
                versions: &[],
                expansion: &not_expanded,
            },
            &[],
            masks,
            &claimed_versions,
            table.seams,
        ) {
            Ok(tables) => tables,
            Err(refusal) => {
                return Ok(GivenStatesOnTheGpu::TablesRefused {
                    refusal: format!("{refusal:?}"),
                })
            }
        };
        // 判表先入库、再读库，GPU 只拿读回的那一份算
        let packed = crate::crash_library::record_the_judge_tables_and_read_them_back(
            &mut self.library.lock(),
            &PackedJudgeTables::of(&tables),
        )?;
        drop(tables);
        let judged_on_the_card = |verdicts: Vec<[u32; crate::crash_judge_gpu::VERDICT_WORDS]>,
                                  judged_on: JudgingDevice|
         -> GivenStatesOnTheGpu {
            GivenStatesOnTheGpu::Judged {
                words: verdicts
                    .iter()
                    .map(|verdict| {
                        cross_check_words_without_the_oracle(cross_check_words_of_gpu_verdict(
                            verdict,
                        ))
                    })
                    .collect(),
                abandoned_roots_named_of_each_state: verdicts
                    .iter()
                    .map(abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict)
                    .collect(),
                undecidable_reason_of_each_state: verdicts
                    .iter()
                    .map(undecidable_reasons_of_the_gpu_verdict)
                    .collect(),
                judged_on,
            }
        };
        let no_card = || {
            "配置开了 GPU，给定状态表的判器（crash_judge_gpu）走 GPU，这一趟却没有起好的卡：不退回 CPU 判器".to_string()
        };
        match self.gpu_cards_taken_from {
            GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
                let cards = self.cards.as_ref().ok_or_else(no_card)?;
                if cards.is_empty() {
                    return Err(no_card());
                }
                let card = &cards[self
                    .next_card
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    % cards.len()];
                let judge = GpuCrashJudge::on_card_compiling_these_kernel_sources(
                    card,
                    &packed,
                    usize::MAX,
                    &kernel_sources,
                );
                let verdicts = judge.judge_states(0..packed.state_count());
                Ok(judged_on_the_card(
                    verdicts,
                    JudgingDevice::Gpu {
                        card_type: card.name.clone(),
                        driver_version: card.driver_version.clone(),
                    },
                ))
            }
            GpuCardsSource::TheCardPoolOfThisProcess => {
                let pool = the_card_pool_this_run_takes_its_cards_from();
                let states_per_dispatch =
                    states_per_scratch_lane(pool.smallest_max_storage_buffer_binding_size());
                match pool.lease_the_first_card_with_room_now(
                    PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                    mebibytes_a_judge_needs(&packed, states_per_dispatch),
                ) {
                    Ok(lease) => {
                        let judge = GpuCrashJudge::on_card_compiling_these_kernel_sources(
                            lease.card(),
                            &packed,
                            states_per_dispatch,
                            &kernel_sources,
                        );
                        let verdicts = judge.judge_states(0..packed.state_count());
                        let judged_on = JudgingDevice::Gpu {
                            card_type: lease.card().name.clone(),
                            driver_version: lease.card().driver_version.clone(),
                        };
                        drop(judge);
                        drop(lease);
                        Ok(judged_on_the_card(verdicts, judged_on))
                    }
                    Err(refusal) => {
                        let reason = given_states_lease_refusal_as_a_run_condition_not_met(
                            &table.table_name,
                            &refusal,
                        );
                        println!("{reason}");
                        Err(reason)
                    }
                }
            }
        }
    }

    /// 没编进 `gpu` 特性走不到这里（选路先报错）。
    ///
    /// # Errors
    /// 恒报错：这份二进制没有 `gpu` 特性。
    #[cfg(not(feature = "gpu"))]
    fn judge_the_table_on_the_gpu(
        &self,
        _table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesOnTheGpu, String> {
        Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
    }

    /// GPU 判完一张表之后：塞故障（只给 GPU 与 CPU 两趟分开跑、比结论会红的那条用例）、记判不了的状态、存一块，交回 GPU 那一份结论
    /// （有判不了的状态时整张表交 CPU 判器判，这一趟已经红了）。
    fn settle_the_gpu_judgement(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
        key: &VerdictBlockKey,
        mut gpu_words: Vec<CrossCheckWords>,
        abandoned_roots_named_of_each_state: Vec<Vec<AbandonedRootNamedByThePoolChecker>>,
        undecidable_reason_of_each_state: &[Option<String>],
        judged_on: JudgingDevice,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        let table = table_with_its_columns.table;
        assert_eq!(
            abandoned_roots_named_of_each_state.len(),
            gpu_words.len(),
            "GPU 判器逐状态交回结论字，点名的被抛弃根与对拍字同一个状态数"
        );
        assert_eq!(
            undecidable_reason_of_each_state.len(),
            gpu_words.len(),
            "GPU 判器逐状态交回结论字，判不了的原因与对拍字同一个状态数"
        );
        match self.fault_put_on_the_gpu_side {
            FaultPutOnTheGpuSide::NoFault => {}
            FaultPutOnTheGpuSide::FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable => {
                if let Some(first) = gpu_words.first_mut() {
                    first.pool_checker_violated_bits ^= 1;
                    first.pool_checker_judged_bits |= 1;
                }
            }
        }
        let could_not_judge: Vec<String> = gpu_words
            .iter()
            .enumerate()
            .filter(|(_, words)| words.gpu_could_not_judge)
            .map(|(ordinal, _)| {
                format!(
                    "{} 第 {ordinal} 个状态（结论里判不了那一位：{}）",
                    table.table_name,
                    undecidable_reason_of_each_state[ordinal].as_deref().expect(
                        "判不了那一位由恢复内核判不了那一位与池级 checker 判不了那一字置，原因就是这两样里不为 0 的那几个"
                    )
                )
            })
            .collect();
        let gpu_states = gpu_words.len();
        self.note(|tally| {
            tally.tables_judged_here += 1;
            tally.states_judged_by_the_gpu_here += u64::try_from(gpu_states).expect("装得进 u64");
            tally
                .states_the_gpu_could_not_judge
                .extend(could_not_judge.iter().cloned());
        });
        let gpu = GivenStatesConclusionsOfATable {
            count_vector: count_vector_of_the_given_states(&gpu_words),
            states: gpu_words
                .into_iter()
                .zip(abandoned_roots_named_of_each_state)
                .map(|(words, abandoned_roots_named)| GivenStateConclusion {
                    words,
                    pool_checker_violation_details: Vec::new(),
                    abandoned_roots_named_by_the_pool_checker: abandoned_roots_named,
                })
                .collect(),
            source: GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
        };
        if !could_not_judge.is_empty() {
            // GPU 判不了的状态不进库（照 H6 判红）；往下走的结论取 CPU 判器的
            return Ok(self.judged_on_the_cpu_here(table_with_its_columns));
        }
        self.store_the_conclusions(key, JudgedBy::GpuJudge, &gpu, judged_on)?;
        Ok(gpu)
    }

    fn judged_on_the_cpu_here(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> GivenStatesConclusionsOfATable {
        let cpu = judge_the_given_states_on_the_cpu(table_with_its_columns);
        let states = cpu.states.len();
        self.note(|tally| {
            tally.states_judged_by_the_cpu_here += u64::try_from(states).expect("装得进 u64");
        });
        cpu
    }
}

/// 给定状态表的清单在库里登的名字的头（接项名、种子与规模、表名、表除持久集合之外的摘要）：与流水线那一侧按流名登的清单同一张表，头不同不会撞。
const GIVEN_STATES_TABLE_MANIFEST_NAME_PREFIX: &str = "given-states-table::";

/// 一张给定状态表除持久集合之外那几样（表名、基线、写表、分段、接缝、旋钮栏）的摘要，与录进库的字节同一种写法、持久集合那一节空着。
/// 清单名带着它：同一个表名下写表不同或旋钮值不同的两张表各登各的清单。持久集合与逐状态那一栏恢复自称的那一版不进它：只判那一趟判的是
/// 录入段录下的那几个状态，调用方这一趟交来的持久集合与那一栏不作数（判段不收调用方内存里的表）。
fn frame_digest_of_a_given_states_table(
    table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
) -> [u8; DIGEST_BYTES] {
    let table = table_with_its_columns.table;
    let frame = RecordedGivenStatesTableWithItsColumns {
        table: RecordedGivenStatesTable {
            table_name: table.table_name.clone(),
            base: table.base.clone(),
            writes: table.writes.to_vec(),
            segments: table.segments.to_vec(),
            seams: table.seams.to_vec(),
            persisted_sets: Vec::new(),
        },
        container_capacity_knobs: table_with_its_columns.container_capacity_knobs,
        claimed_versions: Vec::new(),
    };
    sha256_digest(&frame.to_bytes())
}
/// 给定状态表清单的种类。
const GIVEN_STATES_TABLE_MANIFEST_KIND: &str = "given_states_table";
/// 给定状态表清单里表那一样的名字。
const GIVEN_STATES_TABLE_MANIFEST_TABLE: &str = "table";

/// 没判的一张表交回的占位结论（每个状态都记成不红、两遍都没落到根）：两台分片、归别的机器判而这一趟库里又没有
/// （[`GivenStatesRun::left_tables_to_the_other_hosts`]），或这一趟只跑录入段（[`GivenStatesRun::recorded_only`]）。崩溃注入照它往下走
/// 只为把这一批的表都判完或录进库，交回的报告不作数。
fn placeholder_conclusions_of_a_table(
    state_count: usize,
    source: GivenStatesConclusionsSource,
) -> GivenStatesConclusionsOfATable {
    let placeholder_words = CrossCheckWords {
        red_bits: 0,
        pool_checker_violated_bits: 0,
        consulted_pass_landed_root: LandedRootWords::NOT_LANDED,
        ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
        consulted_pass_outcome: 0,
        ignored_pass_outcome: 0,
        consulted_pass_failure_point: 0,
        ignored_pass_failure_point: 0,
        pool_checker_judged_bits: 0,
        gpu_could_not_judge: false,
    };
    let every_word = vec![placeholder_words; state_count];
    GivenStatesConclusionsOfATable {
        count_vector: count_vector_of_the_given_states(&every_word),
        states: every_word
            .into_iter()
            .map(|words| GivenStateConclusion {
                words,
                pool_checker_violation_details: Vec::new(),
                abandoned_roots_named_by_the_pool_checker: Vec::new(),
            })
            .collect(),
        source,
    }
}

impl GivenStatesJudge for GivenStatesEngine {
    fn stop_is_requested(&self) -> bool {
        let requested = self.stop_request.is_requested();
        if requested {
            self.answered_a_stop_request
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        requested
    }

    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        self.judge_a_table_with_its_columns(table.into())
    }

    fn judge_the_given_states_with_its_columns(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        self.judge_a_table_with_its_columns(table_with_its_columns)
    }
}

impl GivenStatesEngine {
    /// 判一张带两栏的给定状态表（[`GivenStatesJudge::judge_the_given_states`] 是两栏都不填的那一种）：两栏跟着表录进库
    /// （[`RecordedGivenStatesTableWithItsColumns`]）、进块键，判段按读回的两栏判（CPU 判器调池级 checker 带旋钮值的入口、记录核对器第二条判据
    /// 照那一栏恢复自称的那一版；GPU 判器把旋钮值编进扫描内核、那一栏进判表）。
    ///
    /// # Errors
    /// 同 [`GivenStatesJudge::judge_the_given_states`]。
    fn judge_a_table_with_its_columns(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        let table = table_with_its_columns.table;
        let state_count = table.persisted_sets.len();
        if state_count == 0 {
            return Ok(GivenStatesConclusionsOfATable {
                states: Vec::new(),
                count_vector: count_vector_of_the_given_states(&[]),
                source: GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
            });
        }
        if u64::try_from(state_count).expect("装得进 u64") > STATES_PER_BLOCK {
            return Err(format!(
                "给定状态表 {} 有 {state_count} 个状态，一张表一块、一块最多 {STATES_PER_BLOCK} 个：第一版不分块",
                table.table_name
            ));
        }
        // 两段（用户 2026-10-11 定；选段照 RunStage，用户 2026-10-13 定「给定状态引擎认选段」）：录入段把这张表写进库、按项名与表名登清单；
        // 判段只凭项名与表名从库里读回那一份判，块键与结论都从读回的那一份来，不收调用方内存里的表
        let manifest_name = self.manifest_name_of_the_table(table_with_its_columns);
        let recorded_digest = if self.stage.records() {
            Some(self.record_the_table(table_with_its_columns, &manifest_name)?)
        } else {
            None
        };
        // 配了 GPU 而这份二进制没编进 gpu 特性：表照录（上面），判段不跑、交占位结论，开跑时打过本次未跑那一行；不退回 CPU 判
        if self.not_run_without_the_gpu_feature.is_some() {
            self.note(|tally| tally.tables_not_judged_without_the_gpu_feature += 1);
            return Ok(placeholder_conclusions_of_a_table(
                state_count,
                GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly,
            ));
        }
        if !self.stage.judges() {
            self.note(|tally| tally.tables_recorded_only += 1);
            return Ok(placeholder_conclusions_of_a_table(
                state_count,
                GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly,
            ));
        }
        // 两段先后跑：按录入段刚写下的摘要从库里读回（同一个表名下几张内容不同的表各判各的）；只判的那一趟凭清单名找录入段登的那一张
        let digest = match recorded_digest {
            Some(recorded_digest) => recorded_digest,
            None => self.digest_of_the_recorded_table(&manifest_name)?,
        };
        let read_back_bytes = self
            .store()
            .read_given_states_table(&digest)
            .map_err(|error| format!("给定状态表 {} 从库里读不回：{error:?}", table.table_name))?
            .ok_or_else(|| {
                format!(
                    "给定状态表 {} 的清单指的那一份不在库里：先跑录入段",
                    table.table_name
                )
            })?;
        if sha256_digest(&read_back_bytes) != digest {
            return Err(format!(
                "给定状态表 {} 从库里读回的摘要与清单登的对不上",
                table.table_name
            ));
        }
        let read_back = RecordedGivenStatesTableWithItsColumns::from_bytes(&read_back_bytes)
            .ok_or_else(|| format!("给定状态表 {} 从库里读回的字节解不出", table.table_name))?;
        read_back.with_the_table(|table_read_back_from_the_library| {
            self.judge_a_table_read_back(table_read_back_from_the_library)
        })
    }
    /// 一张表在库里登清单的名字：项名接表名（项名与表名各是调用方写死的），再接这张表除持久集合之外那几样的摘要
    /// （[`frame_digest_of_a_given_states_table`]）：同一个表名下写表不同的两张表各登各的清单，只判那一趟各判各的（甲报告问题 5）。
    fn manifest_name_of_the_table(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> String {
        let table = table_with_its_columns.table;
        let frame_digest = frame_digest_of_a_given_states_table(table_with_its_columns);
        format!(
            "{GIVEN_STATES_TABLE_MANIFEST_NAME_PREFIX}{}::{}::{}::{}",
            self.item.item_name,
            self.item.seeds_and_scale,
            table.table_name,
            hexadecimal_text(&frame_digest)
        )
    }

    /// 录入段：这张表写进库、按清单名登它的摘要，交回摘要。
    ///
    /// # Errors
    /// 库写失败。
    fn record_the_table(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
        manifest_name: &str,
    ) -> Result<[u8; DIGEST_BYTES], String> {
        let table = table_with_its_columns.table;
        let recorded_bytes =
            RecordedGivenStatesTableWithItsColumns::of(table_with_its_columns).to_bytes();
        let recorded_digest = sha256_digest(&recorded_bytes);
        // 清单名分不开的两张表（同名、写表也相同、只差持久集合）这一趟各录一次：后录的盖掉先录的清单，只判那一趟两张都读到后录的那一张。
        // 拒掉，不悄悄判错
        if let Some(earlier_digest) = self
            .manifests_recorded_in_this_run
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(manifest_name.to_string(), recorded_digest)
            .filter(|earlier_digest| *earlier_digest != recorded_digest)
        {
            return Err(format!(
                "给定状态表 {} 这一趟录过另一张同名、写表也相同而持久集合不同的表（{} 与 {}）：清单名分不开它们，只判那一趟两张都会读到后录的那一张；表名里带上分得开它们的那一样",
                table.table_name,
                hexadecimal_text(&earlier_digest[..8]),
                hexadecimal_text(&recorded_digest[..8])
            ));
        }
        let manifest = crate::crash_library::ItemManifest {
            kind: GIVEN_STATES_TABLE_MANIFEST_KIND.to_string(),
            digests: vec![(
                GIVEN_STATES_TABLE_MANIFEST_TABLE.to_string(),
                recorded_digest,
            )],
        };
        {
            let mut store = self.store();
            store
                .store_given_states_table(&recorded_digest, &recorded_bytes)
                .map_err(|error| format!("给定状态表 {} 写不进库：{error:?}", table.table_name))?;
            store
                .store_recorded_run_manifest(manifest_name, &manifest.to_bytes())
                .map_err(|error| {
                    format!("给定状态表 {} 的清单写不进库：{error:?}", table.table_name)
                })?;
        }
        self.note(|tally| tally.tables_recorded_here += 1);
        Ok(recorded_digest)
    }

    /// 判段：凭清单名读回录入段登的那张表的摘要。
    ///
    /// # Errors
    /// 库读失败；清单不在（录入段没跑）、解不出、没有表那一样。
    fn digest_of_the_recorded_table(
        &self,
        manifest_name: &str,
    ) -> Result<[u8; DIGEST_BYTES], String> {
        let bytes = self
            .store()
            .read_recorded_run_manifest(manifest_name)
            .map_err(|error| format!("读给定状态表的清单 {manifest_name} 失败：{error:?}"))?
            .ok_or_else(|| {
                format!(
                    "给定状态表那一类的库里没有 {manifest_name} 的清单：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）"
                )
            })?;
        crate::crash_library::ItemManifest::from_bytes(&bytes)
            .ok_or_else(|| format!("给定状态表的清单 {manifest_name} 解不出：重跑录入段"))?
            .digest_named(GIVEN_STATES_TABLE_MANIFEST_TABLE)
    }

    /// 判一张从库里读回的表：块键、导入、复用、两台分片的占位、判都照读回的这一份。
    fn judge_a_table_read_back(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        let table = table_with_its_columns.table;
        let state_count = table.persisted_sets.len();
        let key = self.block_key_of_the_table(table_with_its_columns)?;
        // 归别的机器判的表，先从那一台取回的库点名导入这一张（判之前；导过来的照库里的复用），与流水线那一侧只点不归本机的块同一个分法
        if !self.libraries_to_import_from.is_empty() && !self.owns_the_table(&key) {
            self.import_the_table_from_the_other_hosts(&key)?;
        }
        if let Some(stored) = self.stored_conclusions(&key, self.judged_by, state_count)? {
            self.note(|tally| {
                tally.tables_reused += 1;
                tally.states_reused += u64::try_from(state_count).expect("装得进 u64");
            });
            return Ok(stored);
        }
        if !self.owns_the_table(&key) {
            if !self.libraries_to_import_from.is_empty() {
                return Err(format!(
                    "{GIVEN_STATES_TABLE_JUDGED_BY_NO_HOST_PREFIX} table={} share={}/{}：这张表归别的机器判，导入了那一台取回的库之后库里还是没有它的结论（那一台没判完，或库没取回全）：对账不为空，判红",
                    table.table_name, self.share.index, self.share.count
                ));
            }
            self.note(|tally| tally.tables_left_to_the_other_hosts += 1);
            return Ok(placeholder_conclusions_of_a_table(
                state_count,
                GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost,
            ));
        }
        self.judge_a_table_read_back_from_the_library(table_with_its_columns, &key, state_count)
    }

    /// 判一张从库里读回的给定状态表：配了 GPU 只由 GPU 判器判（GPU 判不了的整张交 CPU 判器），没配只由 CPU 判器判；结论存进库。
    fn judge_a_table_read_back_from_the_library(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
        key: &VerdictBlockKey,
        state_count: usize,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        let table = table_with_its_columns.table;
        let key = key.clone();
        match self.judged_by {
            JudgedBy::CpuJudge => {
                let cpu = self.judged_on_the_cpu_here(table_with_its_columns);
                self.note(|tally| tally.tables_judged_here += 1);
                self.store_the_conclusions(&key, JudgedBy::CpuJudge, &cpu, JudgingDevice::Cpu)?;
                Ok(cpu)
            }
            JudgedBy::GpuJudge => match self.judge_the_table_on_the_gpu(table_with_its_columns)? {
                GivenStatesOnTheGpu::Judged {
                    words,
                    abandoned_roots_named_of_each_state,
                    undecidable_reason_of_each_state,
                    judged_on,
                } => self.settle_the_gpu_judgement(
                    table_with_its_columns,
                    &key,
                    words,
                    abandoned_roots_named_of_each_state,
                    &undecidable_reason_of_each_state,
                    judged_on,
                ),
                GivenStatesOnTheGpu::TablesRefused { refusal } => {
                    self.note(|tally| {
                        tally.states_the_gpu_could_not_judge.push(format!(
                            "{} 建不出 GPU 判表（{state_count} 个状态都判不了）：{refusal}",
                            table.table_name
                        ));
                    });
                    Ok(self.judged_on_the_cpu_here(table_with_its_columns))
                }
            },
        }
    }
}

// ============================== 结论的版本、对照与「根槽已持久」（崩溃点重放合并第三轮判决 J6、J7、J8） ==============================

/// 这一趟判第 ① 段的是哪一个判器（第三轮判决 J5：配了 GPU 是 GPU 判器，没配是 CPU 判器）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgedBy {
    GpuJudge,
    CpuJudge,
}

impl JudgedBy {
    #[must_use]
    pub fn conclusion_kind(self) -> ConclusionKind {
        match self {
            Self::GpuJudge => ConclusionKind::GpuJudge,
            Self::CpuJudge => ConclusionKind::CpuJudge,
        }
    }
}

/// 版本对不上那一行的头：入口照它把这一趟记成运行条件不满足、退 78（第二轮判决 L3-1：版本编译时与跑时不等就整趟不写库）。
pub const CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX: &str =
    "CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD";

/// 这一趟写不了库：结论的版本编译时与跑时对不上、这个入口编译时没算，或从盘上算不出。一个字都没写。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConclusionVersionsNotWritable {
    /// 结论代码摘要表编译时与跑时有对不上的入口（二进制不是从盘上这棵树编的：编完又改了代码、拿别的树编的）。
    TheTableDiffersFromTheBuild { entries: Vec<String> },
    /// 这一把版本编译时与跑时不等，或这个入口编译时没算（设点文件不在 `crates/singlefs-checker-tier/tests` 下）。
    AVersionDiffersFromTheBuild { request: String, check: String },
    /// 从盘上算不出（读不了某份文件）。
    NotComputableFromTheTree { reason: String },
}

impl ConclusionVersionsNotWritable {
    /// 打进标准输出、也作为错误交回的那一行（以 [`CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX`] 起头）。
    #[must_use]
    pub fn not_written_line(&self) -> String {
        let detail = match self {
            Self::TheTableDiffersFromTheBuild { entries } => {
                format!("reason=table_differs entries={}", entries.join(","))
            }
            Self::AVersionDiffersFromTheBuild { request, check } => {
                format!("reason=version_differs request={request} check={check}")
            }
            Self::NotComputableFromTheTree { reason } => {
                format!("reason=not_computable detail={reason}")
            }
        };
        format!(
            "{CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX} {detail}（整趟不写库：重编再跑，或在提交那一趟的 worktree 里跑；退 78）"
        )
    }
}

/// 这一趟每个点各种结论记在哪一把版本下（第三轮判决 J8）：GPU 判器一把（每个点同一把），CPU 判器按设点文件各一把；与计划的点同序
/// （连同末节点，它的设点文件取最后一个点的）。
/// 每一把都与编译时写进二进制的那一把核过（`crash_identity::conclusion_version_checked_against_the_build`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionVersionsOfThePlan {
    pub gpu_judge_of_each_point: Vec<ConclusionVersion>,
    pub cpu_judge_of_each_point: Vec<ConclusionVersion>,
}

impl ConclusionVersionsOfThePlan {
    /// 先整张结论代码摘要表编译时与跑时比，再逐把取：GPU 判器一把，每个点的设点文件各一把 CPU 判器（同一份文件只算一次）。
    ///
    /// # Errors
    /// 见 [`ConclusionVersionsNotWritable`] 的三种。
    pub fn of_the_plan(
        repository_root: &Path,
        plan: &CrashPlan,
        toolchain: &CrashPointReplayToolchainIdentity,
    ) -> Result<Self, ConclusionVersionsNotWritable> {
        let not_computable =
            |error: JudgingCodeError| ConclusionVersionsNotWritable::NotComputableFromTheTree {
                reason: error.reason,
            };
        let differing = conclusion_code_entries_differing_from_the_build(repository_root)
            .map_err(not_computable)?;
        if !differing.is_empty() {
            return Err(ConclusionVersionsNotWritable::TheTableDiffersFromTheBuild {
                entries: differing.iter().map(|entry| format!("{entry:?}")).collect(),
            });
        }
        let version_to_write_under = |request: &ConclusionVersionRequest<'_>| {
            conclusion_version_to_write_under(repository_root, request, toolchain)
        };
        let gpu_judge = version_to_write_under(&ConclusionVersionRequest::GpuJudge)?;
        let mut cpu_judge_of_each_setting_file: BTreeMap<&str, ConclusionVersion> = BTreeMap::new();
        let mut cpu_judge_of_each_point = Vec::with_capacity(plan.points.len());
        for point in &plan.points {
            let setting_file = point_code_file(point);
            let version = match cpu_judge_of_each_setting_file.get(setting_file) {
                Some(version) => *version,
                None => {
                    let version = version_to_write_under(&ConclusionVersionRequest::CpuJudge {
                        setting_file,
                    })?;
                    cpu_judge_of_each_setting_file.insert(setting_file, version);
                    version
                }
            };
            cpu_judge_of_each_point.push(version);
        }
        Ok(Self {
            gpu_judge_of_each_point: vec![gpu_judge; plan.points.len()],
            cpu_judge_of_each_point,
        })
    }

    /// 这一种判器的结论记在哪一处。
    #[must_use]
    pub fn stored_under(&self, judged_by: JudgedBy) -> ConclusionsStoredUnder<'_> {
        ConclusionsStoredUnder::TheConclusionKind {
            kind: judged_by.conclusion_kind(),
            version_of_each_point: match judged_by {
                JudgedBy::GpuJudge => &self.gpu_judge_of_each_point,
                JudgedBy::CpuJudge => &self.cpu_judge_of_each_point,
            },
        }
    }
}

/// 「根槽已持久」的计数事后数（第三轮判决 J3 的 M2-2 甲、J7）：这条流按这份计划展开的状态里，写表第 `root_slot_write` 次写（一条根槽写）
/// 持久的有几个。不进块键、不进库：各项按自己的被判根现算。按段的序号区间数：这一次写所在的段之前一个都不持久；之后的段与最后全部持久
/// 那一个都持久；所在那一段展开了的，组合数是状态数加一（真子集之外的那一个全落），根槽写两态、持久的占一半，去掉全落那一个。
///
/// # Panics
/// `root_slot_write` 不是根槽写（只有根槽写两态、在甲二里也逐位拆，这里的数法只对它成立）；它不在任何一段里。
#[must_use]
pub fn states_with_the_root_slot_write_persisted(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    root_slot_write: usize,
) -> u64 {
    assert_eq!(
        flow.writes[root_slot_write].kind,
        StepKind::RootRecordFua,
        "第 {root_slot_write} 次写不是根槽写：这里只按根槽写两态数"
    );
    let segment_of_the_write = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&root_slot_write))
        .expect("写表里的每次写都在某一段里");
    // 分片的段里根槽写只占片里的一部分，这里按「组合数的一半」数不成立；崩溃枚举用例（crash_state_enumeration、tests/ 里的调用方）的计划
    // 都不分片，只有 run_from_environment 排出分片的计划，它不调这里
    assert!(
        plan.segment_slices()[segment_of_the_write].is_none(),
        "第 {root_slot_write} 次写所在的第 {segment_of_the_write} 段分了片：按段数「根槽已持久」只对不分片的段成立"
    );
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let (ranges_by_segment, state_count) = crash_state_ranges_by_segment_with_slices(
        flow.base,
        flow.writes,
        flow.segments,
        &expansion,
        plan.segment_slices(),
    );
    let range = &ranges_by_segment[segment_of_the_write];
    let states_of_the_segment = range.end - range.start;
    let persisted_inside_the_segment = if states_of_the_segment == 0 {
        0
    } else {
        // 组合数 = 状态数 + 1（偶数：根槽写两态）；根槽写持久的占一半，去掉全落那一个
        states_of_the_segment.div_ceil(2) - 1
    };
    // 之后的段与最后全部持久那一个：整条流的状态数减去到这一段为止的
    persisted_inside_the_segment + (state_count - range.end)
}

// ============================== 按段登记（崩溃点重放合并第三轮判决 J10、M4 第 15 条） ==============================

/// 按段登记表的路径从这个环境变量交给引擎（入口 `scripts/crash/crash-point-replay.sh` 开跑时送；表是
/// `.claude/gate.d/crash-point-replay-segments.tsv`，门禁与表归工具那一路）。没设就是一段都没登记。
pub const SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY";

/// 轻型崩溃那一域里「展开 6」的展开上限（`scripts/crash/crash-point-replay.sh` 文件头写的两个域之一）：登记了的段写数不超过它时全展开。
pub const LIGHT_TIER_EXPAND_UP_TO_WRITES_PER_SEGMENT: usize = 6;

/// 登记了的段怎么展开：轻型两个域（「甲二」与展开 6）的并——写数不超过 [`LIGHT_TIER_EXPAND_UP_TO_WRITES_PER_SEGMENT`] 的全展开（全展开罩着甲二），
/// 超过的只取甲二（第三轮判决 J10：重型对展不开的段跑轻型两个域，重型 ⊇ 轻型）。
#[must_use]
pub fn expansion_of_a_registered_segment(segment: &[usize]) -> CrashPointReplaySegmentExpansion {
    if segment.len() <= LIGHT_TIER_EXPAND_UP_TO_WRITES_PER_SEGMENT {
        CrashPointReplaySegmentExpansion::EveryProperSubset
    } else {
        CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
    }
}

/// 段写表摘要的十六进制位数（sha256）。
const SEGMENT_WRITE_TABLE_DIGEST_HEXADECIMAL_DIGITS: usize = 2 * DIGEST_BYTES;

/// 按段登记表里一条流登记了的一段：段号、段写数、段写表摘要（那一段带屏障的写表，`crate::crash_identity::step_write_table_with_barriers`
/// 只给这一段算的那一份，64 位小写十六进制）。整域按全量展开深度展不开的段登记在这里，引擎照轻型两个域的并展开
/// （[`expansion_of_a_registered_segment`]）、计入跳过量（第三轮正文 H10）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredSegment {
    pub segment_index: usize,
    pub write_count: usize,
    pub write_table_digest_text: String,
}

/// 读按段登记表的正文，取流名是 `flow_name` 的那几行登记的段，按段号排。一行 `<合表项名>\t<流名>\t<段号>\t<段写数>\t<段写表摘要>`，
/// 可再加一段 `#` 起头的说明；`#` 起头的行与空行跳过。第一段（合表项名）这里只核非空，在不在合表里归入口的核法
/// （`scripts/crash/lib_crash_point_replay_items.py`）；引擎按第二段（流名）取：一项有几条流，各条各登各的段。
///
/// # Errors
/// 有一行写坏了（不是五段、合表项名是空的、段号或段写数不是十进制非负整数或带前导 0、段写数是 0、摘要不是 64 位小写十六进制、
/// 同一条流同一段两行），错误里写第几行、哪一处不对与那一行原文：写坏的表不当成「没登记」，整趟拒跑。
pub fn registered_segments_of_the_flow(
    registry_text: &str,
    flow_name: &str,
) -> Result<Vec<RegisteredSegment>, String> {
    let decimal = |text: &str| -> Option<usize> {
        let is_decimal = !text.is_empty()
            && text.bytes().all(|byte| byte.is_ascii_digit())
            && (text == "0" || !text.starts_with('0'));
        if is_decimal {
            text.parse::<usize>().ok()
        } else {
            None
        }
    };
    let mut registered: Vec<RegisteredSegment> = Vec::new();
    let mut every_flow_and_segment: BTreeSet<(String, usize)> = BTreeSet::new();
    for (line_index, line) in registry_text.lines().enumerate() {
        let line_number = line_index + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let refused = |reason: &str| {
            Err(format!(
                "按段登记表第 {line_number} 行写坏了（{reason}）：「{line}」"
            ))
        };
        let mut fields: Vec<&str> = line.split('\t').collect();
        if fields.len() == 6 && fields[5].starts_with('#') {
            fields.pop();
        }
        let [item_field, flow_field, segment_field, write_count_field, digest_field] =
            fields.as_slice()
        else {
            return refused(
                "一行五段（合表项名、流名、段号、段写数、段写表摘要），制表符隔开，可再加一段 # 说明",
            );
        };
        if item_field.is_empty() {
            return refused("第一段合表项名不许空");
        }
        let Some(segment_index) = decimal(segment_field) else {
            return refused("段号要是十进制非负整数、不带前导 0");
        };
        let Some(write_count) = decimal(write_count_field).filter(|count| *count > 0) else {
            return refused("段写数要是大于 0 的十进制整数、不带前导 0");
        };
        let digest_is_lowercase_hexadecimal = digest_field.len()
            == SEGMENT_WRITE_TABLE_DIGEST_HEXADECIMAL_DIGITS
            && digest_field
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if !digest_is_lowercase_hexadecimal {
            return refused("段写表摘要要是 64 位小写十六进制（sha256）");
        }
        if !every_flow_and_segment.insert(((*flow_field).to_string(), segment_index)) {
            return refused("同一条流同一段只许一行");
        }
        if *flow_field == flow_name {
            registered.push(RegisteredSegment {
                segment_index,
                write_count,
                write_table_digest_text: (*digest_field).to_string(),
            });
        }
    }
    registered.sort_by_key(|segment| segment.segment_index);
    Ok(registered)
}

/// 这条流第 `segment_index` 段带屏障的写表摘要（按段登记表第五段的取法：只给这一段算 `step_write_table_with_barriers`）；
/// 这一段的写下标不连续时交回 `None`（录制流切出的段都是连续的，不连续说明这条流不是录出来的那种，登记对不上）。
fn segment_write_table_digest_text(flow: &CrashFlow<'_>, segment_index: usize) -> Option<String> {
    let segment = flow.segments.get(segment_index)?;
    let first = *segment.first()?;
    let last = *segment.last()?;
    let is_contiguous = segment.windows(2).all(|pair| pair[1] == pair[0] + 1);
    is_contiguous.then(|| {
        hexadecimal_text(&step_write_table_with_barriers(
            flow.writes,
            std::slice::from_ref(segment),
            first..last + 1,
        ))
    })
}

/// 登记的每一段与这条流对得上：段在、段写数相同、段写表摘要相同（M4 第 15 条：写表变了、段号挪位之后对不上就拒跑，不按写数重找、不照段号跳过）。
///
/// # Errors
/// 第一处对不上的段：项名、段号、登记的与这条流的。
pub fn check_the_registered_segments_against_the_flow(
    flow: &CrashFlow<'_>,
    flow_name: &str,
    registered_segments: &[RegisteredSegment],
) -> Result<(), String> {
    for registered in registered_segments {
        let Some(segment) = flow.segments.get(registered.segment_index) else {
            return Err(format!(
                "按段登记表登记了项 {flow_name} 的第 {} 段，这条流只有 {} 段：写表变了，登记对不上，拒跑（改登记表或改回写表）",
                registered.segment_index,
                flow.segments.len()
            ));
        };
        if segment.len() != registered.write_count {
            return Err(format!(
                "按段登记表登记项 {flow_name} 第 {} 段 {} 次写，这条流这一段 {} 次写：写表变了，登记对不上，拒跑",
                registered.segment_index,
                registered.write_count,
                segment.len()
            ));
        }
        let digest = segment_write_table_digest_text(flow, registered.segment_index);
        if digest.as_deref() != Some(registered.write_table_digest_text.as_str()) {
            return Err(format!(
                "按段登记表登记项 {flow_name} 第 {} 段的写表摘要 {}，这条流这一段是 {}：写表变了，登记对不上，拒跑",
                registered.segment_index,
                registered.write_table_digest_text,
                digest.as_deref().unwrap_or("（这一段的写下标不连续，算不出）")
            ));
        }
    }
    Ok(())
}

/// 这一趟这条流登记了的段：[`SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE`] 没设是一段都没有；设了就读那份表、取流名是 `flow_name` 的那几行
/// （[`registered_segments_of_the_flow`]），再逐段与这条流核（[`check_the_registered_segments_against_the_flow`]）。
///
/// # Errors
/// 表读不了、写坏了、登记的段对不上。
fn registered_segments_of_the_flow_from_the_environment(
    flow: &CrashFlow<'_>,
    flow_name: &str,
) -> Result<Vec<RegisteredSegment>, String> {
    let Some(registry_path) = std::env::var_os("SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY")
    else {
        return Ok(Vec::new());
    };
    let registry_path = PathBuf::from(registry_path);
    let registry_text = std::fs::read_to_string(&registry_path).map_err(|error| {
        format!(
            "{SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE} 指的按段登记表读不了：{}（{error}）",
            registry_path.display()
        )
    })?;
    let registered = registered_segments_of_the_flow(&registry_text, flow_name)?;
    check_the_registered_segments_against_the_flow(flow, flow_name, &registered)?;
    Ok(registered)
}

/// 录入段里第 `segment_index` 段怎么展开：登记了的段照轻型两个域的并（[`expansion_of_a_registered_segment`]，第三轮判决 J10、M6-10：
/// 重型 ⊇ 轻型），全量展开不算；没登记的照调用方给的（`flow.expansion`）。
fn expansion_in_the_recording(
    registered_segments: &[RegisteredSegment],
    flow: &CrashFlow<'_>,
    segment_index: usize,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    if registered_segments
        .iter()
        .any(|registered| registered.segment_index == segment_index)
    {
        expansion_of_a_registered_segment(segment)
    } else {
        (flow.expansion)(segment_index, segment)
    }
}

/// 按段登记的段整域装不进 u64 时（C790，用户 2026-10-13 定「先段内取样……每次走 2^32，然后通过剪枝慢慢的剪开」），一趟判几个态：
/// 这个环境变量给（要是正整数、不超过 [`LARGEST_REGISTERED_SEGMENT_SLICE_STATES`]），没设取 [`DEFAULT_REGISTERED_SEGMENT_SLICE_STATES`]。
pub const REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE: &str =
    "SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES";

/// 没设片长时一趟判 2^32 个态。
pub const DEFAULT_REGISTERED_SEGMENT_SLICE_STATES: u64 = 1 << 32;

/// 片长的上界 2^32：两个 GPU 内核把片内序号当 u32 加到片起点的各位上（序号 − 段起点的高 32 位要是 0）。
pub const LARGEST_REGISTERED_SEGMENT_SLICE_STATES: u64 = 1 << 32;

/// 片长变量的值（`None` 是没设）换成一趟判几个态。
///
/// # Errors
/// 设了而不是十进制正整数，或超过 [`LARGEST_REGISTERED_SEGMENT_SLICE_STATES`]：错误带变量名与原值（写错的值不悄悄变成默认那一档）。
pub fn registered_segment_slice_states_of_the_variable_value(
    value: Option<&str>,
) -> Result<u64, String> {
    let Some(text) = value else {
        return Ok(DEFAULT_REGISTERED_SEGMENT_SLICE_STATES);
    };
    let is_decimal = !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
    match text.parse::<u64>() {
        Ok(states) if is_decimal && states > 0 && states <= LARGEST_REGISTERED_SEGMENT_SLICE_STATES => {
            Ok(states)
        }
        Ok(_) | Err(_) => Err(format!(
            "{REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE}={text}：一趟判几个态要是十进制正整数、不超过 {LARGEST_REGISTERED_SEGMENT_SLICE_STATES}（GPU 内核按 32 位加片内序号）"
        )),
    }
}

/// 按段登记的一段在这一趟怎么判：整域装得进 u64 就全展开（`full`）；装不进就分片（`slice`），每趟从续点起判一片。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainKindOfARegisteredSegment {
    Full,
    Slice,
}

impl DomainKindOfARegisteredSegment {
    /// 驱动行 `registered_segment_domains=` 里的写法。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Slice => "slice",
        }
    }
}

/// 按段登记的一段：段号、整域的态数（[`StateCount1280Bits`]，按 [`expansion_of_a_registered_segment`] 的展开）与它在这一趟怎么判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainOfARegisteredSegment {
    pub segment_index: usize,
    pub domain_state_count: StateCount1280Bits,
    pub kind: DomainKindOfARegisteredSegment,
}

/// 录入段：按段登记的每一段整域的态数与它这一趟怎么判（`flow` 里登记的段已换成 [`expansion_in_the_recording`] 的那一份；ignore 的点的段
/// 不展开，同 [`plan_crash_points`]）。整域装得进 u64 是 `full`，装不进是 `slice`。
///
/// # Errors
/// 整域越过 1280 位（[`STATE_COUNT_LIMBS`] 个 u128，用户 2026-10-13 定的位宽）：拒跑，都在写库之前交回。
pub fn domains_of_the_registered_segments(
    flow: &CrashFlow<'_>,
    registered_segments: &[RegisteredSegment],
) -> Result<Vec<DomainOfARegisteredSegment>, String> {
    let owner_of_segment = owner_of_each_segment(flow);
    let tearable = TearableInPlaceOverwrites::of(flow.base, flow.writes);
    registered_segments
        .iter()
        .map(|registered| {
            let segment_index = registered.segment_index;
            let segment = &flow.segments[segment_index];
            let expansion = if flow.crash_points[owner_of_segment[segment_index]].ignored {
                CrashPointReplaySegmentExpansion::NotExpanded
            } else {
                (flow.expansion)(segment_index, segment)
            };
            let domain_state_count = expansion
                .checked_state_count(flow.writes, segment, &tearable)
                .ok_or_else(|| {
                    format!(
                        "按段登记表登记的第 {segment_index} 段（{} 次写）按轻型两个域的并（{}）展开的态数越过 {} 位（{STATE_COUNT_LIMBS} 个 u128，\
                         用户 2026-10-13 定的位宽）：装不下，拒跑（没写库）",
                        segment.len(),
                        expansion.name(),
                        STATE_COUNT_LIMBS * 128
                    )
                })?;
            let kind = match domain_state_count.to_u64() {
                Some(_) => DomainKindOfARegisteredSegment::Full,
                None => DomainKindOfARegisteredSegment::Slice,
            };
            Ok(DomainOfARegisteredSegment {
                segment_index,
                domain_state_count,
                kind,
            })
        })
        .collect()
}

/// 录入段排计划之前核：`flow`（登记了的段已换成 [`expansion_in_the_recording`] 的那一份）照计划的展开（ignore 的点的段不展开，
/// 同 [`plan_crash_points`]）、分片的段照 `segment_slices` 的片长，整条流的态数每段与合起来（再加全部持久那一个）都装得进 u64 才交回，
/// 排计划时不会在算态数那一步 panic。
///
/// # Errors
/// 不分片的段态数装不进 u64（登记了的段整域装不进 u64 的已经分片，到这里的是没登记的段）、整条流合起来装不下：拒跑，都在写库之前交回。
pub fn states_of_the_flow_to_record(
    flow: &CrashFlow<'_>,
    segment_slices: &[Option<CrashPointReplaySegmentSlice>],
) -> Result<u64, String> {
    let owner_of_segment = owner_of_each_segment(flow);
    let tearable = TearableInPlaceOverwrites::of(flow.base, flow.writes);
    let mut states = 1u64;
    for (segment_index, segment) in flow.segments.iter().enumerate() {
        let expansion = if flow.crash_points[owner_of_segment[segment_index]].ignored {
            CrashPointReplaySegmentExpansion::NotExpanded
        } else {
            (flow.expansion)(segment_index, segment)
        };
        let states_of_the_segment = match &segment_slices[segment_index] {
            Some(slice) => slice.state_count,
            None => expansion
                .checked_state_count(flow.writes, segment, &tearable)
                .and_then(|state_count| state_count.to_u64())
                .ok_or_else(|| {
                    format!(
                        "第 {segment_index} 段（{} 次写）按 {} 展开的态数装不进 u64，拒跑（没写库）",
                        segment.len(),
                        expansion.name()
                    )
                })?,
        };
        states = states.checked_add(states_of_the_segment).ok_or_else(|| {
            format!("整条流的态数到第 {segment_index} 段装不进 u64，拒跑（没写库）")
        })?;
    }
    Ok(states)
}

/// 录入段排计划之前定每段的片：按段登记的段整域装得进 u64 的全展开、不分片，装不进的分片，片长照
/// `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES`、片起点是库里的续点（每个分片的段往 stderr 打一行 `CRASH_AMPLIFICATION_SLICE`）。
/// `flow` 里登记了的段已换成 [`expansion_in_the_recording`] 的那一份。只读库，不写。
///
/// # Errors
/// 都在写库之前交回：登记段整域越过 1280 位；片长为 0 的那份计划（找续点用）算态数装不下，即没登记的段态数装不进 u64、
/// 或整条流合起来装不下；片长变量认不出；结论版本对不上（同时往 stdout 打不写库那一行）；库读失败。
pub fn segment_slices_of_the_recording_from_the_environment_and_the_library(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    flow_name: &str,
    registered_segments: &[RegisteredSegment],
    repository_root: &Path,
    toolchain: &CrashPointReplayToolchainIdentity,
    crash_judge: ComputedOn,
) -> Result<Vec<Option<CrashPointReplaySegmentSlice>>, String> {
    let domains = domains_of_the_registered_segments(flow, registered_segments)?;
    let slices_of_the_plan_to_find_the_resume_points =
        placeholder_slices_of_the_sliced_segments(flow.segments.len(), &domains);
    // 找续点那份计划同样要算态数（分片的段片长为 0）：先核装得进 u64，装不进在写库之前拒跑，不在排那份计划时 panic
    states_of_the_flow_to_record(flow, &slices_of_the_plan_to_find_the_resume_points)?;
    if !domains
        .iter()
        .any(|domain| domain.kind == DomainKindOfARegisteredSegment::Slice)
    {
        return Ok(no_segment_slices(flow.segments.len()));
    }
    let slice_states_text = std::env::var_os("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES")
        .map(|value| {
            value.into_string().map_err(|value| {
                format!(
                    "{REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE}={value:?}：不是 UTF-8"
                )
            })
        })
        .transpose()?;
    let slice_states =
        registered_segment_slice_states_of_the_variable_value(slice_states_text.as_deref())?;
    // 续点从库里读（进度记录核过才信，核不过从 0 起逐块问库）：块键只由计划哈希与段内编号定，先排一份片长为 0 的计划拿块键
    let plan_to_find_the_resume_points =
        plan_crash_points_with_slices(flow, &slices_of_the_plan_to_find_the_resume_points);
    let conclusion_versions = match ConclusionVersionsOfThePlan::of_the_plan(
        repository_root,
        &plan_to_find_the_resume_points,
        toolchain,
    ) {
        Ok(versions) => versions,
        Err(not_writable) => {
            let line = not_writable.not_written_line();
            println!("{line}");
            return Err(line);
        }
    };
    let judged_by_in_the_recording = match crash_judge {
        ComputedOn::Gpu => JudgedBy::GpuJudge,
        ComputedOn::Cpu => JudgedBy::CpuJudge,
    };
    let (segment_slices, resume_points) = segment_slices_of_the_recording(
        store,
        flow_name,
        flow.segments.len(),
        &domains,
        &plan_to_find_the_resume_points,
        &conclusion_versions.stored_under(judged_by_in_the_recording),
        slice_states,
    )
    .map_err(|error| format!("读分片的续点失败：{error:?}"))?;
    for (segment_index, resume_point) in &resume_points {
        eprintln!(
            "CRASH_AMPLIFICATION_SLICE flow={flow_name} segment={segment_index} slice_start={} slice_states={} resume_point_found_by={}",
            resume_point.judged_through.decimal_text(),
            segment_slices[*segment_index].map_or(0, |slice| slice.state_count),
            match resume_point.found_by {
                ResumePointFoundBy::TheProgressRecordChecked => "progress_record",
                ResumePointFoundBy::AWalkFromZeroThroughTheLibrary => "walk_from_zero",
            }
        );
    }
    Ok(segment_slices)
}

/// 分片进度记在哪一个判器、哪一把版本下：这一段所在的点那一把（与它那几块的结论存在同一处）。
fn judge_of_the_progress(
    conclusions: &ConclusionsStoredUnder<'_>,
    point_index: usize,
) -> JudgeOfTheProgress {
    match conclusions {
        ConclusionsStoredUnder::TheJudgeVersion(version) => {
            JudgeOfTheProgress::TheJudgeVersion(**version)
        }
        ConclusionsStoredUnder::TheConclusionKind {
            kind,
            version_of_each_point,
        } => JudgeOfTheProgress::TheConclusionKind {
            kind: kind.clone(),
            version: ConclusionsStoredUnder::conclusion_version_of_the_point(
                version_of_each_point,
                point_index,
            ),
        },
    }
}

/// 一条流第 `segment_index` 段在 `conclusions` 那一处的分片进度记录的键。
///
/// # Panics
/// 流名空着或超过 u16::MAX 字节（[`StreamOrNodeName::new`]）。
fn progress_key_of_the_sliced_segment(
    flow_name: &str,
    segment_index: usize,
    conclusions: &ConclusionsStoredUnder<'_>,
    point_index: usize,
) -> SlicedSegmentProgressKey {
    SlicedSegmentProgressKey {
        flow_name: StreamOrNodeName::new(flow_name).expect("流名不空、不超过 u16::MAX 字节"),
        segment_index: u64::try_from(segment_index).expect("段号装得进 u64"),
        judge: judge_of_the_progress(conclusions, point_index),
    }
}

/// 一段分片的段的续点是怎么找到的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResumePointFoundBy {
    /// 进度记录核过了：它指的那一块判完了、它之后那一块没判。
    TheProgressRecordChecked,
    /// 没有进度记录、或进度记录核不过：从编号 0 起逐块问库，走到第一块没判的（真相以块为准）。
    AWalkFromZeroThroughTheLibrary,
}

/// 一段分片的段从哪接着判：段内编号 [0, `judged_through`) 的块都判完了。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResumePointOfASlicedSegment {
    pub judged_through: StateCount1280Bits,
    pub found_by: ResumePointFoundBy,
}

/// 计划里第 `segment_index` 段（分片的段）落在哪个点、点内第几段：（点号，点内段序）。
fn point_and_segment_in_point_of_the_sliced_segment(
    plan: &CrashPlan,
    segment_index: usize,
) -> Option<(usize, usize)> {
    plan.sliced_segments()
        .into_iter()
        .find_map(|(point_index, piece)| match piece.kind {
            KindOfAPiece::SlicedSegment {
                segment_index: piece_segment,
                segment_in_point,
                ..
            } if piece_segment == segment_index => Some((point_index, segment_in_point)),
            KindOfAPiece::SlicedSegment { .. } | KindOfAPiece::SegmentsNotSliced { .. } => None,
        })
}

/// 录入段：分片的段从哪接着判（C790）。先读进度记录（[`VerdictStore::read_sliced_segment_progress`]）再核它——它记的最后判完的那一块
/// （段内编号 [`last_block_start`, `judged_through`)）在 `conclusions` 那一处有结论、录入表里的块长对得上，`judged_through` 那一块没有结论
/// （走到整域末尾的不用核这一半）；核过了就从 `judged_through` 接着。没有记录、或核不过，从编号 0 起逐块问库：一块有结论就照录入表里
/// 它的块长跳到下一块，走到第一块没有结论的停下，从那里接着（真相以块为准，进度记录只是加速）。
/// `plan` 是排出了这一段为分片的段的计划（块键只由计划哈希与段内编号定，不看这一份计划的片起点与片长）。
///
/// # Errors
/// 库读失败。
///
/// # Panics
/// `plan` 里第 `segment_index` 段不是分片的段。
pub fn resume_point_of_the_sliced_segment(
    store: &VerdictStore,
    flow_name: &str,
    plan: &CrashPlan,
    segment_index: usize,
    domain_state_count: &StateCount1280Bits,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<ResumePointOfASlicedSegment, VerdictStoreError> {
    let (point_index, segment_in_point) =
        point_and_segment_in_point_of_the_sliced_segment(plan, segment_index)
            .expect("计划里这一段是分片的段");
    let point = &plan.points[point_index];
    let is_judged = |start_within_the_domain: &StateCount1280Bits| {
        conclusions.is_stored(
            store,
            point_index,
            &point.block_key_of_a_slice_block(segment_in_point, start_within_the_domain),
        )
    };
    let key =
        progress_key_of_the_sliced_segment(flow_name, segment_index, conclusions, point_index);
    if let Some(progress) = store.read_sliced_segment_progress(&key)? {
        let last_block_is_judged_with_its_length = if progress.judged_through.is_zero() {
            true
        } else {
            let length_of_the_last_block = progress
                .judged_through
                .checked_sub(&progress.last_block_start)
                .and_then(|length| length.to_u64());
            let recorded_length = store.recorded_state_count_of_block(
                &point.block_key_of_a_slice_block(segment_in_point, &progress.last_block_start),
            )?;
            length_of_the_last_block.is_some()
                && length_of_the_last_block == recorded_length
                && is_judged(&progress.last_block_start)?
        };
        let next_block_is_not_judged =
            progress.judged_through == *domain_state_count || !is_judged(&progress.judged_through)?;
        if progress.judged_through <= *domain_state_count
            && last_block_is_judged_with_its_length
            && next_block_is_not_judged
        {
            return Ok(ResumePointOfASlicedSegment {
                judged_through: progress.judged_through,
                found_by: ResumePointFoundBy::TheProgressRecordChecked,
            });
        }
    }
    let mut judged_through = StateCount1280Bits::ZERO;
    while judged_through < *domain_state_count && is_judged(&judged_through)? {
        let Some(length) = store.recorded_state_count_of_block(
            &point.block_key_of_a_slice_block(segment_in_point, &judged_through),
        )?
        else {
            break;
        };
        if length == 0 {
            break;
        }
        judged_through = judged_through
            .checked_add_u64(length)
            .expect("录入表里的块不越过段的域，装得进 1280 位");
    }
    Ok(ResumePointOfASlicedSegment {
        judged_through,
        found_by: ResumePointFoundBy::AWalkFromZeroThroughTheLibrary,
    })
}

/// 这一趟分片的段各判到了哪：从这一趟的片起点起，`conclusions` 那一处有结论的块连着数，数到第一块没有结论的为止
/// （交回（段号，判到的段内编号，最后判完的那一块的起点））；这一趟一块都没判完的交回片起点、最后那一块的起点记 `None`。只读库。
///
/// # Errors
/// 库读失败。
pub fn judged_through_of_the_sliced_segments(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<Vec<(usize, StateCount1280Bits, Option<StateCount1280Bits>)>, VerdictStoreError> {
    let mut judged = Vec::new();
    for (point_index, piece) in plan.sliced_segments() {
        let KindOfAPiece::SlicedSegment {
            segment_index,
            start_within_the_domain,
            ..
        } = piece.kind
        else {
            continue;
        };
        let point = &plan.points[point_index];
        let mut judged_through = start_within_the_domain;
        let mut last_block_start = None;
        for local in point.blocks() {
            if !piece.local_states.contains(&local.start) {
                continue;
            }
            if !conclusions.is_stored(store, point_index, &point.block_key(&local))? {
                break;
            }
            let block_start = start_within_the_domain
                .checked_add_u64(local.start - piece.local_states.start)
                .expect("片里的编号装得进 1280 位");
            last_block_start = Some(block_start);
            judged_through = block_start
                .checked_add_u64(local.end - local.start)
                .expect("片里的编号装得进 1280 位");
        }
        judged.push((segment_index, judged_through, last_block_start));
    }
    Ok(judged)
}

/// 判段：这一趟分片的段判完的块落盘之后推进进度记录（[`judged_through_of_the_sliced_segments`]）；这一趟一块都没判完的段不写。
/// 被叫停、显存不够提早收尾的那一趟同样推进到判完的地方。
///
/// # Errors
/// 库读写失败。
pub fn advance_the_progress_of_the_sliced_segments(
    store: &mut VerdictStore,
    flow_name: &str,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<(), VerdictStoreError> {
    for (segment_index, judged_through, last_block_start) in
        judged_through_of_the_sliced_segments(store, plan, conclusions)?
    {
        let Some(last_block_start) = last_block_start else {
            continue;
        };
        let (point_index, _segment_in_point) =
            point_and_segment_in_point_of_the_sliced_segment(plan, segment_index)
                .expect("刚从计划里数出来的分片的段");
        store.store_sliced_segment_progress(
            &progress_key_of_the_sliced_segment(flow_name, segment_index, conclusions, point_index),
            SlicedSegmentProgress {
                judged_through,
                last_block_start,
            },
        )?;
    }
    Ok(())
}

/// 录入段每一段这一趟的片（与段同序），与每个分片的段（段号）的续点（[`segment_slices_of_the_recording`] 交回的）。
pub type SegmentSlicesAndResumePoints = (
    Vec<Option<CrashPointReplaySegmentSlice>>,
    Vec<(usize, ResumePointOfASlicedSegment)>,
);

/// 录入段：这一趟每一段的片。不分片的段是 `None`；分片的段从续点（[`resume_point_of_the_sliced_segment`]）起判 `slice_states` 个态，
/// 到整域末尾为止（走完了的段片长是 0）。`plan_to_find_the_resume_points` 是这几段已排成分片的段的一份计划（片起点与片长随便给，
/// 块键不看它们）。交回每段的片与每个分片的段的续点。
///
/// # Errors
/// 库读失败。
pub fn segment_slices_of_the_recording(
    store: &VerdictStore,
    flow_name: &str,
    segment_count: usize,
    domains: &[DomainOfARegisteredSegment],
    plan_to_find_the_resume_points: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    slice_states: u64,
) -> Result<SegmentSlicesAndResumePoints, VerdictStoreError> {
    let mut segment_slices = no_segment_slices(segment_count);
    let mut resume_points = Vec::new();
    for domain in domains {
        match domain.kind {
            DomainKindOfARegisteredSegment::Full => {}
            DomainKindOfARegisteredSegment::Slice => {
                let resume_point = resume_point_of_the_sliced_segment(
                    store,
                    flow_name,
                    plan_to_find_the_resume_points,
                    domain.segment_index,
                    &domain.domain_state_count,
                    conclusions,
                )?;
                let states_left = domain
                    .domain_state_count
                    .checked_sub(&resume_point.judged_through)
                    .expect("续点不越过整域");
                let state_count = states_left
                    .to_u64()
                    .map_or(slice_states, |left| left.min(slice_states));
                segment_slices[domain.segment_index] = Some(CrashPointReplaySegmentSlice {
                    start_within_the_domain: resume_point.judged_through,
                    state_count,
                });
                resume_points.push((domain.segment_index, resume_point));
            }
        }
    }
    Ok((segment_slices, resume_points))
}

/// 排出续点之前那一份计划要的片：分片的段从 0 起、片长 0（块键不看片起点与片长，只用它找续点）。
#[must_use]
pub fn placeholder_slices_of_the_sliced_segments(
    segment_count: usize,
    domains: &[DomainOfARegisteredSegment],
) -> Vec<Option<CrashPointReplaySegmentSlice>> {
    let mut segment_slices = no_segment_slices(segment_count);
    for domain in domains {
        match domain.kind {
            DomainKindOfARegisteredSegment::Full => {}
            DomainKindOfARegisteredSegment::Slice => {
                segment_slices[domain.segment_index] = Some(CrashPointReplaySegmentSlice {
                    start_within_the_domain: StateCount1280Bits::ZERO,
                    state_count: 0,
                });
            }
        }
    }
    segment_slices
}

/// 驱动行里一段登记了的段的域与进度（[`segment_fields_of_the_line`]）：`full` 的段只报段号；`slice` 的段报到这一趟末判到的段内编号与整域的态数。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainAndProgressOfARegisteredSegment {
    pub segment_index: usize,
    pub kind: DomainKindOfARegisteredSegment,
    /// `slice` 的段判到的段内编号（[0, 它) 的块都判完了）；`full` 的段是整域。
    pub judged_through: StateCount1280Bits,
    pub domain_state_count: StateCount1280Bits,
}

/// 判段：这一趟登记了的段各自的域与进度。分片的段照计划里的片（录入段定的、从库里读回的流带着）与库里已判的块数：
/// 从片起点起连着判完的块（[`judged_through_of_the_sliced_segments`]）；不分片的登记段是 `full`。
///
/// # Errors
/// 库读失败。
pub fn domains_and_progress_of_the_registered_segments(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    registered_segments: &[RegisteredSegment],
    conclusions: &ConclusionsStoredUnder<'_>,
) -> Result<Vec<DomainAndProgressOfARegisteredSegment>, VerdictStoreError> {
    let judged_through_of_the_slices =
        judged_through_of_the_sliced_segments(store, plan, conclusions)?;
    let tearable = TearableInPlaceOverwrites::of(flow.base, flow.writes);
    Ok(registered_segments
        .iter()
        .map(|registered| {
            let segment_index = registered.segment_index;
            let sliced =
                plan.sliced_segments()
                    .into_iter()
                    .find_map(|(_point_index, piece)| match piece.kind {
                        KindOfAPiece::SlicedSegment {
                            segment_index: piece_segment,
                            domain_state_count,
                            ..
                        } if piece_segment == segment_index => Some(domain_state_count),
                        KindOfAPiece::SlicedSegment { .. }
                        | KindOfAPiece::SegmentsNotSliced { .. } => None,
                    });
            match sliced {
                Some(domain_state_count) => DomainAndProgressOfARegisteredSegment {
                    segment_index,
                    kind: DomainKindOfARegisteredSegment::Slice,
                    judged_through: judged_through_of_the_slices
                        .iter()
                        .find(|(judged_segment, _, _)| *judged_segment == segment_index)
                        .map_or(StateCount1280Bits::ZERO, |(_, judged_through, _)| {
                            *judged_through
                        }),
                    domain_state_count,
                },
                None => {
                    let segment = &flow.segments[segment_index];
                    let domain_state_count = plan
                        .segment_expansion(flow, segment_index, segment)
                        .checked_state_count(flow.writes, segment, &tearable)
                        .expect("不分片的登记段整域装得进 u64：录入段核过");
                    DomainAndProgressOfARegisteredSegment {
                        segment_index,
                        kind: DomainKindOfARegisteredSegment::Full,
                        judged_through: domain_state_count,
                        domain_state_count,
                    }
                }
            }
        })
        .collect())
}
/// 一段的段号与写数：驱动行 `registered_segments=` 与 `unregistered_segments_beyond_expansion=` 两个字段的一项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentWithItsWriteCount {
    pub segment_index: usize,
    pub write_count: usize,
}

/// 这条流里没登记、而且展开上限盖不住（[`ExpandUpToWritesPerSegment::expansion_of`] 判 [`CrashPointReplaySegmentExpansion::NotExpanded`]）的段，
/// 按段号升序：登了段的流照旧报 `exhaustive=false`，入口靠它分得出「登记段之外还有展不开的段」（C790 阻断二）。
#[must_use]
pub fn unregistered_segments_beyond_the_expansion(
    segments: &[Vec<usize>],
    registered_segments: &[RegisteredSegment],
    expand_up_to: ExpandUpToWritesPerSegment,
) -> Vec<SegmentWithItsWriteCount> {
    segments
        .iter()
        .enumerate()
        .filter(|(segment_index, segment)| {
            !registered_segments
                .iter()
                .any(|registered| registered.segment_index == *segment_index)
                && expand_up_to.expansion_of(segment)
                    == CrashPointReplaySegmentExpansion::NotExpanded
        })
        .map(|(segment_index, segment)| SegmentWithItsWriteCount {
            segment_index,
            write_count: segment.len(),
        })
        .collect()
}

/// 驱动行末尾说段的四个字段：` registered_segments=<段号>:<写数>[,…] unregistered_segments_beyond_expansion=<段号>:<写数>[,…]
/// registered_segment_domains=<段号>:<full|slice>[,…] registered_segment_progress=<段号>:<已判态数>/<域的总态数>[,…]`，
/// 一段都没有的写 `-`（[`reuse_and_scale_fields_of_the_line`]）。`registered_segment_progress` 只列 `slice` 的段，两个数都是精确的十进制
/// （1280 位的 [`StateCount1280Bits`]，不写浮点、不写近似、不饱和）。
#[must_use]
pub fn segment_fields_of_the_line(
    registered_segments: &[RegisteredSegment],
    unregistered_segments_beyond_the_expansion: &[SegmentWithItsWriteCount],
    registered_segment_domains: &[DomainAndProgressOfARegisteredSegment],
) -> String {
    let field_text = |segments: Vec<SegmentWithItsWriteCount>| {
        if segments.is_empty() {
            "-".to_string()
        } else {
            segments
                .iter()
                .map(|segment| format!("{}:{}", segment.segment_index, segment.write_count))
                .collect::<Vec<String>>()
                .join(",")
        }
    };
    let registered = field_text(
        registered_segments
            .iter()
            .map(|registered| SegmentWithItsWriteCount {
                segment_index: registered.segment_index,
                write_count: registered.write_count,
            })
            .collect(),
    );
    let unregistered_beyond = field_text(unregistered_segments_beyond_the_expansion.to_vec());
    let or_a_dash = |items: Vec<String>| {
        if items.is_empty() {
            "-".to_string()
        } else {
            items.join(",")
        }
    };
    let domains = or_a_dash(
        registered_segment_domains
            .iter()
            .map(|domain| format!("{}:{}", domain.segment_index, domain.kind.name()))
            .collect(),
    );
    let progress = or_a_dash(
        registered_segment_domains
            .iter()
            .filter(|domain| domain.kind == DomainKindOfARegisteredSegment::Slice)
            .map(|domain| {
                format!(
                    "{}:{}/{}",
                    domain.segment_index,
                    domain.judged_through.decimal_text(),
                    domain.domain_state_count.decimal_text()
                )
            })
            .collect(),
    );
    format!(
        " registered_segments={registered} unregistered_segments_beyond_expansion={unregistered_beyond} registered_segment_domains={domains} registered_segment_progress={progress}"
    )
}

// ============================== 挂在节点上的观察者（崩溃点重放合并第三轮判决 J4；两段与按恢复镜像去重：用户 2026-10-13 定） ==============================
//
// 用户 2026-10-13 定（原话在 `records/2026-10-13-复用键与全仓改名.md`「课题 4」）：观察者改走两段、池级 checker 上 GPU、按恢复结果去重。
// 录入段（[`record_the_observers_of_the_flow`]）在 CPU 上逐状态跑观察者，结果写进库：只记计数的那一种逐状态看（它可以读持久集合的结构），
// 交回镜像的那一种（可写挂载、续做）按恢复镜像的字节去重，同一张镜像只看一次。判段（[`judge_the_observers_of_the_flow`]）只从库里读，
// 对观察者交回的镜像跑池级 checker（配了 GPU 在卡上判，同一张镜像只判一次），逐状态拼回一块结论存进库；判段不跑恢复、不调观察者。

/// 只记计数的观察者看一个崩溃状态：镜像、看 journal 那一遍恢复的报告、往观察者计数里记；判不对就 panic（与崩溃点重放的逐状态观察者同一个形，
/// `crate::crash::CrashStateObserving`）。录入段在核对线程上跑，几条线程同时看不同的状态；逐状态看，不去重（它可以读持久集合的结构）。
pub type ObserveACrashState<'observer> = dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)
    + Sync
    + 'observer;

/// 交回镜像的观察者看一个崩溃状态：在这张崩溃镜像上真跑实现（可写挂载、续做），记计数、判不对就 panic，交回它在镜像上发的写与它停下时的盘
/// （[`ImageAfterTheObserver`]）；那张盘上的池级 checker 不在这里跑，判段从库里读回来判（配了 GPU 在卡上判）。它交回的只许随镜像的字节、
/// 恢复报告（镜像的函数）与它另读的输入（进去重键）变：录入段按恢复镜像的字节去重（[`ObserverDeduplication`]），同一张镜像只看一次，
/// 别的状态取那一次的结果。
pub type ObserveACrashStateHandingBackAnImage<'observer> = dyn Fn(
        u64,
        &CrashImage<'_>,
        &RecoveryReport,
        &mut CrashPointReplayObserverCounts,
    ) -> ImageAfterTheObserver
    + Sync
    + 'observer;

/// 交回镜像的观察者看完一个状态交回的：它在崩溃镜像上发的写（按发的次序；录入段把它们叠回崩溃镜像，与 `disks_after_the_observer` 逐字节核过
/// 才存）、判段在那张盘上判哪几条不变量、判段判干净时给哪个计数加一。
pub struct ImageAfterTheObserver {
    pub writes_on_the_crash_image: Vec<RetainedWrite>,
    pub disks_after_the_observer: MemoryPool,
    pub invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver,
    pub counter_when_the_pool_checker_is_clean: String,
}

/// 判段在观察者交回的盘上判哪几条不变量。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InvariantsJudgedOnTheImageAfterTheObserver {
    EveryImplementedInvariant,
    /// 只判这几条（`singlefs_checker::image::IMPLEMENTED_INVARIANTS` 里的名字）。
    Only(Vec<&'static str>),
}

impl InvariantsJudgedOnTheImageAfterTheObserver {
    fn includes(&self, invariant: &'static str) -> bool {
        match self {
            Self::EveryImplementedInvariant => true,
            Self::Only(invariants) => invariants.contains(&invariant),
        }
    }
}

/// 交回镜像的观察者怎么去重：产品路径恒按恢复镜像的字节（同一个基础镜像上，镜像每个扇区的字节都相同就是同一张）；关掉去重只给剪枝自证用
/// （小输入上关掉跑一遍，逐状态判定与开着时逐项相同）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverDeduplication {
    ByTheRecoveredImageBytes,
    OffForTheSelfProof,
}

/// 观察者的代码是哪一种。
pub enum ObserverCode<'observer> {
    CountsOnly(&'observer ObserveACrashState<'observer>),
    HandsBackAnImage {
        observe: &'observer ObserveACrashStateHandingBackAnImage<'observer>,
        deduplication: ObserverDeduplication,
    },
}

/// 观察者挂在哪个节点上：设这个节点的文件（仓内路径）与入口的名字（计划里这个入口的每一段都是这个节点的点，`<入口>.<周期>.<步>`；
/// 末节点写 [`END_OF_FLOW_NODE_NAME`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeOfAnObserver {
    pub setting_file: String,
    pub entry_name: String,
}

impl NodeOfAnObserver {
    /// 计划里的这个点是不是这个节点的。
    #[must_use]
    pub fn holds_the_point(&self, point: &PlannedCrashPoint) -> bool {
        point_code_file(point) == self.setting_file
            && (point.name == self.entry_name
                || point
                    .name
                    .strip_prefix(self.entry_name.as_str())
                    .is_some_and(|rest| rest.starts_with('.')))
    }
}

/// 挂在几个节点上的一个观察者（登记表在 `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs`）：名字、代码文件（它的版本是从这份文件
/// 出发的闭包，`crate::crash_identity::ConclusionVersionRequest::Observer`）、另读的流程输入的摘要（进它的版本与去重键，第三轮判决 M2-3 乙；
/// 要真算，没有另读的输入才给全零）、挂在哪几个节点上、它的代码。观察者代码不进计划哈希、不进判器版本。
pub struct ObserverOnTheNodes<'observer> {
    pub observer_name: ObserverName,
    pub observer_code_file: String,
    pub extra_inputs_digest: [u8; DIGEST_BYTES],
    pub nodes: Vec<NodeOfAnObserver>,
    pub observe: ObserverCode<'observer>,
}

/// 一个观察者这一趟用的几把版本（[`observer_versions_to_write_under`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObserverVersions {
    /// 观察者代码出发的闭包、工具链与另读的输入：录入段的块按它存。
    pub observer: ConclusionVersion,
    /// 同上，另读的输入那一格取全零：另读的输入单独进去重键（[`observed_image_deduplication_key`]），去重键不靠版本带它。
    pub observer_code: ConclusionVersion,
    /// 判段对观察者交回的盘跑池级 checker 的判器（配了 GPU 是 GPU 判器，没配是 CPU 判器）与它那一把版本（CPU 判器的版本从观察者代码文件出发）。
    pub pool_checker: JudgedBy,
    pub pool_checker_version: ConclusionVersion,
    /// 这个观察者的结论按它存：观察者版本并上池级 checker 那一把（两样哪一样变了，结论都重拼）。
    pub conclusion: ConclusionVersion,
}

/// 观察者结论那一把版本里判段拼块那一截的写法号：观察者交回的盘怎么建判表、判不了的怎么记、一块怎么拼，代码在这份文件里，不在观察者代码与判器
/// 那两把版本的闭包里；这一截改了、判出来的会变，就抬它，库里早先拼的那几块不再复用（2：每张盘同一位置上的写并过、交回的写相同的盘共用一份写
/// 再建判表，判不了的不存进库；1 那一版单张盘越过一个落点的写数上限就判不了，判不了的那几块存进了库）。
const OBSERVER_CONCLUSION_COMPOSITION_VERSION: u32 = 2;

/// 一个观察者这一趟的几把版本，开库之前算、与编译时写进二进制的那一份核（[`conclusion_version_to_write_under`]），对不上整趟不写库。
///
/// # Errors
/// 某一把对不上或算不出。
pub(crate) fn observer_versions_to_write_under(
    repository_root: &Path,
    observer: &ObserverOnTheNodes<'_>,
    pool_checker: JudgedBy,
    toolchain: &CrashPointReplayToolchainIdentity,
) -> Result<ObserverVersions, ConclusionVersionsNotWritable> {
    let version_with_the_extra_inputs = |extra_inputs_digest: [u8; DIGEST_BYTES]| {
        conclusion_version_to_write_under(
            repository_root,
            &ConclusionVersionRequest::Observer {
                observer_name: &observer.observer_name,
                observer_code_file: &observer.observer_code_file,
                extra_inputs_digest,
            },
            toolchain,
        )
    };
    let observer_version = version_with_the_extra_inputs(observer.extra_inputs_digest)?;
    let observer_code = version_with_the_extra_inputs([0; DIGEST_BYTES])?;
    let pool_checker_version = match pool_checker {
        JudgedBy::GpuJudge => conclusion_version_to_write_under(
            repository_root,
            &ConclusionVersionRequest::GpuJudge,
            toolchain,
        )?,
        JudgedBy::CpuJudge => conclusion_version_to_write_under(
            repository_root,
            &ConclusionVersionRequest::CpuJudge {
                setting_file: &observer.observer_code_file,
            },
            toolchain,
        )?,
    };
    let conclusion = ConclusionVersion(sha256_digest(
        format!(
            "singlefs observer conclusion with the pool checker on its image {OBSERVER_CONCLUSION_COMPOSITION_VERSION}\nobserver={}\npool_checker={}:{}\n",
            hexadecimal_text(&observer_version.0),
            pool_checker.conclusion_kind().version_text(),
            hexadecimal_text(&pool_checker_version.0)
        )
        .as_bytes(),
    ));
    Ok(ObserverVersions {
        observer: observer_version,
        observer_code,
        pool_checker,
        pool_checker_version,
        conclusion,
    })
}

/// 观察者录入段这一趟的账（[`EnvironmentRun::observer_recording`]；行里报「现判 N、去重复用 M」）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObserverRecordingTally {
    /// 这一趟录的块数与状态数（库里这一把观察者版本早先录过、或结论已经存着的块不再录）。
    pub blocks_recorded_here: u64,
    pub states_recorded_here: u64,
    pub blocks_already_recorded: u64,
    /// 只记计数的观察者这一趟逐状态看的状态数。
    pub states_counted_here: u64,
    /// 交回镜像的观察者这一趟真看的恢复镜像张数（现判）。
    pub images_observed_here: u64,
    /// 交回镜像的观察者经过的状态里，那张镜像已经看过（这一趟早先的状态，或库里存着的）、这一趟不再看的状态数（去重复用）。
    pub states_reusing_an_observed_image: u64,
    /// 叫停了（[`stop_is_requested_by_the_environment`]）：停在块边上，录过的块在库里，下一趟接着录。
    pub stopped_early: bool,
}

impl ObserverRecordingTally {
    fn absorb(&mut self, other: &Self) {
        let Self {
            blocks_recorded_here,
            states_recorded_here,
            blocks_already_recorded,
            states_counted_here,
            images_observed_here,
            states_reusing_an_observed_image,
            stopped_early,
        } = other;
        self.blocks_recorded_here += blocks_recorded_here;
        self.states_recorded_here += states_recorded_here;
        self.blocks_already_recorded += blocks_already_recorded;
        self.states_counted_here += states_counted_here;
        self.images_observed_here += images_observed_here;
        self.states_reusing_an_observed_image += states_reusing_an_observed_image;
        self.stopped_early |= stopped_early;
    }
}

/// 一趟里观察者的判段判了什么、读回来什么（[`EnvironmentRun::observing`]）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObservingOfTheRun {
    /// 这一趟拼成结论的块数与状态数（库里这一把结论版本早先存过的块不再拼）。
    pub blocks_observed_here: u64,
    pub states_observed_here: u64,
    /// 观察者交回的盘这一趟跑池级 checker 的张数（现判），其中在卡上判的张数；库里这一把判器版本早先判过、这一趟取库里的张数（复用）。
    pub images_judged_here: u64,
    pub images_judged_on_the_gpu_here: u64,
    pub image_judgements_reused: u64,
    /// 这条流里每个观察者经过的状态数（这台机器领的块，这一趟拼的与库里复用的）、读回来的计数（按计数的名字合起来）与判红的状态数。
    pub states_of_each_observer: BTreeMap<String, u64>,
    pub counts_of_each_observer: BTreeMap<String, BTreeMap<String, u64>>,
    pub red_states_of_each_observer: BTreeMap<String, u64>,
    /// 这一趟拼出来的红状态，前 [`RED_TEXTS_KEPT`] 个：路径编号、点内序号、红的那几项。
    pub red_texts: Vec<(String, u64, String)>,
}

/// 观察者判红的那个状态存进判定向量的那一项：`observer:<观察者名>:<那一句>`。
fn observer_red_item(observer_name: &ObserverName, panic_message: &str) -> String {
    format!("observer:{}:{panic_message}", observer_name.as_str())
}

/// panic 带的那一句（`panic!`、`assert!` 的消息是字符串）；带的不是字符串时写一句说明。
fn panic_message_of(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|message| (*message).to_string())
        })
        .unwrap_or_else(|| "观察者 panic 时带的不是一句话".to_string())
}

// ------------------------------ 按恢复镜像的字节去重 ------------------------------

/// 一个扇区的字节摘要：SHA-256 的前 16 个字节（只在同一个基础镜像上比扇区等不等）。
type SectorDigest = [u8; 16];

fn sector_digest(sector_bytes: &[u8]) -> SectorDigest {
    sha256_digest(sector_bytes)[..16]
        .try_into()
        .expect("SHA-256 摘要 32 字节，前 16 个总在")
}

fn zero_sector_digest() -> SectorDigest {
    sector_digest(&[0; 512])
}

/// 枚举写表里一次写盖住的扇区：普通写逐扇区记字节摘要；整段清零记范围（清零那一段里只有写过的扇区会变）。
enum SectorsOfAWrite {
    Written(Vec<(u32, u64, SectorDigest)>),
    Zeroed {
        device: singlefs_core::address::DeviceIdentity,
        first_sector: u64,
        end_sector: u64,
    },
}

/// 一条流的崩溃镜像按字节去重要的几样（录入段每条流现算一次）：基础镜像的字节摘要、枚举写表每次写盖住的扇区与每个扇区的摘要、
/// 这几个扇区在基础镜像里的摘要。两个状态的镜像摘要相同 ⟺ 镜像每个扇区的字节相同（同一个基础镜像上；扇区摘要取 128 位）。
struct RecoveredImageBytesDigester<'flow> {
    base: &'flow MemoryPool,
    base_digest: [u8; DIGEST_BYTES],
    sectors_of_each_write: Vec<SectorsOfAWrite>,
    base_digest_of_each_written_sector: std::collections::HashMap<(u32, u64), SectorDigest>,
    zero_sector: SectorDigest,
}

impl<'flow> RecoveredImageBytesDigester<'flow> {
    fn of(base: &'flow MemoryPool, enumerated_writes: &[RetainedWrite]) -> Self {
        let sector_bytes = usize::try_from(singlefs_harness::memory_pool::SECTOR_BYTES)
            .expect("扇区字节数装得进 usize");
        let zero_sector = zero_sector_digest();
        let mut message = b"singlefs base image bytes 1\n".to_vec();
        message.extend_from_slice(&base.device_size_in_bytes.to_le_bytes());
        for (identity, device) in &base.devices {
            message.extend_from_slice(&identity.0.to_le_bytes());
            for (sector, bytes) in device.written_sectors() {
                if bytes.iter().any(|byte| *byte != 0) {
                    message.extend_from_slice(&sector.to_le_bytes());
                    message.extend_from_slice(bytes);
                }
            }
            message.extend_from_slice(b"\nend of device\n");
        }
        let base_digest = sha256_digest(&message);
        drop(message);
        let mut base_digest_of_each_written_sector = std::collections::HashMap::new();
        let sectors_of_each_write = enumerated_writes
            .iter()
            .map(|write| {
                let first_sector = write.offset.0 / singlefs_harness::memory_pool::SECTOR_BYTES;
                match &write.contents {
                    WrittenContents::Bytes(bytes) => SectorsOfAWrite::Written(
                        bytes
                            .chunks(sector_bytes)
                            .enumerate()
                            .map(|(sector_index, chunk)| {
                                let sector = first_sector
                                    + u64::try_from(sector_index).expect("扇区下标装得进 u64");
                                base_digest_of_each_written_sector
                                    .entry((write.device.0, sector))
                                    .or_insert_with(|| {
                                        Self::digest_of_a_base_sector(base, write.device, sector)
                                    });
                                (write.device.0, sector, sector_digest(chunk))
                            })
                            .collect(),
                    ),
                    WrittenContents::Zeros { length } => SectorsOfAWrite::Zeroed {
                        device: write.device,
                        first_sector,
                        end_sector: first_sector
                            + length.div_ceil(singlefs_harness::memory_pool::SECTOR_BYTES),
                    },
                }
            })
            .collect();
        Self {
            base,
            base_digest,
            sectors_of_each_write,
            base_digest_of_each_written_sector,
            zero_sector,
        }
    }

    fn digest_of_a_base_sector(
        base: &MemoryPool,
        device: singlefs_core::address::DeviceIdentity,
        sector: u64,
    ) -> SectorDigest {
        match base
            .devices
            .get(&device)
            .and_then(|sparse| sparse.written_sectors().get(&sector))
        {
            Some(bytes) => sector_digest(bytes),
            None => zero_sector_digest(),
        }
    }

    fn base_digest_of(&self, device: u32, sector: u64) -> SectorDigest {
        self.base_digest_of_each_written_sector
            .get(&(device, sector))
            .copied()
            .unwrap_or_else(|| {
                Self::digest_of_a_base_sector(
                    self.base,
                    singlefs_core::address::DeviceIdentity(device),
                    sector,
                )
            })
    }

    /// 一个状态（持久集合按枚举写表给）的镜像摘要：基础镜像的摘要，接上与基础镜像字节不同的每个扇区（盘、扇区号、扇区摘要，按盘与扇区号排）。
    ///
    /// # Panics
    /// 持久集合与枚举写表不一样长。
    fn image_digest(&self, persisted: &[bool]) -> [u8; DIGEST_BYTES] {
        assert_eq!(
            persisted.len(),
            self.sectors_of_each_write.len(),
            "持久集合与枚举写表逐条对应"
        );
        let mut overlay: BTreeMap<(u32, u64), SectorDigest> = BTreeMap::new();
        for (sectors, is_persisted) in self.sectors_of_each_write.iter().zip(persisted) {
            if !*is_persisted {
                continue;
            }
            match sectors {
                SectorsOfAWrite::Written(sectors) => {
                    for (device, sector, digest) in sectors {
                        overlay.insert((*device, *sector), *digest);
                    }
                }
                SectorsOfAWrite::Zeroed {
                    device,
                    first_sector,
                    end_sector,
                } => {
                    let written_in_the_base =
                        self.base
                            .devices
                            .get(device)
                            .map_or_else(Vec::new, |sparse| {
                                sparse.written_sectors_in(
                                    singlefs_core::address::DeviceOffsetInBytes(
                                        first_sector * singlefs_harness::memory_pool::SECTOR_BYTES,
                                    ),
                                    (end_sector - first_sector)
                                        * singlefs_harness::memory_pool::SECTOR_BYTES,
                                )
                            });
                    let written_by_earlier_writes: Vec<(u32, u64)> = overlay
                        .range((device.0, *first_sector)..(device.0, *end_sector))
                        .map(|(position, _)| *position)
                        .collect();
                    for sector in written_in_the_base {
                        overlay.insert((device.0, sector), self.zero_sector);
                    }
                    for position in written_by_earlier_writes {
                        overlay.insert(position, self.zero_sector);
                    }
                }
            }
        }
        let mut message = Vec::with_capacity(64 + overlay.len() * 28);
        message.extend_from_slice(b"singlefs recovered image bytes 1\n");
        message.extend_from_slice(&self.base_digest);
        for ((device, sector), digest) in &overlay {
            if *digest == self.base_digest_of(*device, *sector) {
                continue;
            }
            message.extend_from_slice(&device.to_le_bytes());
            message.extend_from_slice(&sector.to_le_bytes());
            message.extend_from_slice(digest);
        }
        sha256_digest(&message)
    }
}

/// 交回镜像的观察者按恢复镜像去重的键：观察者名、观察者代码那一把版本（另读的输入那一格全零）、另读的输入的摘要、镜像摘要。
/// 另读的输入不同的两个观察者看同一张镜像不合并；观察者代码改了，键跟着换、旧的结果留作历史。
fn observed_image_deduplication_key(
    observer: &ObserverOnTheNodes<'_>,
    versions: &ObserverVersions,
    image_digest: &[u8; DIGEST_BYTES],
) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs observed image deduplication key 1\n".to_vec();
    message.extend_from_slice(observer.observer_name.as_str().as_bytes());
    message.push(b'\n');
    message.extend_from_slice(&versions.observer_code.0);
    message.extend_from_slice(&observer.extra_inputs_digest);
    message.extend_from_slice(image_digest);
    sha256_digest(&message)
}

/// 剪枝自证关掉去重时每个状态各一个键：在 [`observed_image_deduplication_key`] 的那几样上再接块键与块内序号。
fn observed_image_key_without_deduplication(
    observer: &ObserverOnTheNodes<'_>,
    versions: &ObserverVersions,
    image_digest: &[u8; DIGEST_BYTES],
    block_key: &VerdictBlockKey,
    offset_in_block: u64,
) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs observed image key without deduplication 1\n".to_vec();
    message.extend_from_slice(&observed_image_deduplication_key(
        observer,
        versions,
        image_digest,
    ));
    message.extend_from_slice(&block_key.input_fingerprint.0);
    message.extend_from_slice(block_key.stream_or_node_name.as_str().as_bytes());
    message.extend_from_slice(&block_key.enumeration_plan_hash.0);
    message.extend_from_slice(&block_key.block_start.0.to_le_bytes());
    message.extend_from_slice(&offset_in_block.to_le_bytes());
    sha256_digest(&message)
}

/// 两份池的每个扇区字节相同（写过的全零扇区与没写过的算相同）、盘的身份与大小相同。
fn pools_hold_the_same_bytes(left: &MemoryPool, right: &MemoryPool) -> bool {
    let nonzero_sectors = |device: &singlefs_harness::memory_pool::SparseDevice| {
        device
            .written_sectors()
            .iter()
            .filter(|(_, bytes)| bytes.iter().any(|byte| *byte != 0))
            .map(|(sector, bytes)| (*sector, bytes.clone()))
            .collect::<Vec<(u64, Vec<u8>)>>()
    };
    left.device_size_in_bytes == right.device_size_in_bytes
        && left.devices.len() == right.devices.len()
        && left.devices.iter().zip(&right.devices).all(
            |((left_identity, left_device), (right_identity, right_device))| {
                left_identity == right_identity
                    && nonzero_sectors(left_device) == nonzero_sectors(right_device)
            },
        )
}

/// 崩溃镜像落成一个池：基础镜像叠上持久了的写（与 `crate::writable_mount_phases::materialized_crash_image` 同一条规则）。
fn pool_of_the_crash_image(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

// ------------------------------ 观察者两段写进库的几样 ------------------------------

const OBSERVER_FOUNDATION_MAGIC: &[u8; 8] = b"SFOBFND1";
const OBSERVED_IMAGE_MAGIC: &[u8; 8] = b"SFOBIMG1";
const OBSERVER_BLOCK_MAGIC: &[u8; 8] = b"SFOBBLK1";
const OBSERVED_IMAGE_JUDGEMENT_MAGIC: &[u8; 8] = b"SFOBJDG1";

/// 一条流的崩溃镜像共用的基底：基础镜像与枚举写表（录制流里的写在前，撕裂镜像与重放接在后面）。判段照它与持久集合重建崩溃镜像。
struct ObserverFoundation {
    base: MemoryPool,
    enumerated_writes: Vec<RetainedWrite>,
}

fn observer_foundation_bytes(base: &MemoryPool, enumerated_writes: &[RetainedWrite]) -> Vec<u8> {
    let mut bytes = OBSERVER_FOUNDATION_MAGIC.to_vec();
    put_pool_writes_and_segments(&mut bytes, base, enumerated_writes, &[]);
    bytes
}

impl ObserverFoundation {
    /// 头不对、字节不够或多出、带了切段交回 `None`。
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVER_FOUNDATION_MAGIC.len())? != OBSERVER_FOUNDATION_MAGIC {
            return None;
        }
        let (base, enumerated_writes, segments) = read_pool_writes_and_segments(&mut reader)?;
        (segments.is_empty() && reader.at == bytes.len()).then_some(Self {
            base,
            enumerated_writes,
        })
    }
}

/// 观察者交回、判段要在上面跑池级 checker 的那张盘：基底摘要、这个状态的持久集合（按枚举写表）、观察者在崩溃镜像上发的写、判哪几条、
/// 判干净时给哪个计数加一。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ImageToJudgeAfterTheObserver {
    foundation_digest: [u8; DIGEST_BYTES],
    persisted: Vec<bool>,
    writes_on_the_crash_image: Vec<RetainedWrite>,
    invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver,
    counter_when_the_pool_checker_is_clean: String,
}

/// 交回镜像的观察者在一张恢复镜像上看出的那一份。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedImageRecord {
    red_item: Option<String>,
    counts: Vec<(String, u64)>,
    image_to_judge: Option<ImageToJudgeAfterTheObserver>,
}

fn put_text(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend_from_slice(
        &u64::try_from(text.len())
            .expect("长度装得进 u64")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(text.as_bytes());
}

fn read_text(reader: &mut RecordedBytesReader<'_>) -> Option<String> {
    String::from_utf8(reader.bytes()?.to_vec()).ok()
}

fn put_optional_text(bytes: &mut Vec<u8>, text: Option<&str>) {
    match text {
        None => bytes.push(0),
        Some(text) => {
            bytes.push(1);
            put_text(bytes, text);
        }
    }
}

fn read_optional_text(reader: &mut RecordedBytesReader<'_>) -> Option<Option<String>> {
    match reader.u8()? {
        0 => Some(None),
        1 => Some(Some(read_text(reader)?)),
        _ => None,
    }
}

fn put_counts(bytes: &mut Vec<u8>, counts: &[(String, u64)]) {
    bytes.extend_from_slice(
        &u64::try_from(counts.len())
            .expect("长度装得进 u64")
            .to_le_bytes(),
    );
    for (name, count) in counts {
        put_text(bytes, name);
        bytes.extend_from_slice(&count.to_le_bytes());
    }
}

fn read_counts(reader: &mut RecordedBytesReader<'_>) -> Option<Vec<(String, u64)>> {
    let mut counts = Vec::new();
    for _ in 0..reader.count()? {
        let name = read_text(reader)?;
        counts.push((name, reader.u64()?));
    }
    Some(counts)
}

/// 不变量名字认回 `IMPLEMENTED_INVARIANTS` 里那一条；认不得交回 `None`（库是别的 checker 版本写的）。
fn implemented_invariant_named(name: &str) -> Option<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .copied()
        .find(|invariant| *invariant == name)
}

impl ObservedImageRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = OBSERVED_IMAGE_MAGIC.to_vec();
        put_optional_text(&mut bytes, self.red_item.as_deref());
        put_counts(&mut bytes, &self.counts);
        match &self.image_to_judge {
            None => bytes.push(0),
            Some(image) => {
                bytes.push(1);
                bytes.extend_from_slice(&image.foundation_digest);
                bytes.extend_from_slice(
                    &u64::try_from(image.persisted.len())
                        .expect("长度装得进 u64")
                        .to_le_bytes(),
                );
                bytes.extend(
                    image
                        .persisted
                        .iter()
                        .map(|is_persisted| u8::from(*is_persisted)),
                );
                put_pool_writes_and_segments(
                    &mut bytes,
                    &MemoryPool::default(),
                    &image.writes_on_the_crash_image,
                    &[],
                );
                match &image.invariants_judged {
                    InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant => {
                        bytes.push(0);
                    }
                    InvariantsJudgedOnTheImageAfterTheObserver::Only(invariants) => {
                        bytes.push(1);
                        bytes.extend_from_slice(
                            &u64::try_from(invariants.len())
                                .expect("长度装得进 u64")
                                .to_le_bytes(),
                        );
                        for invariant in invariants {
                            put_text(&mut bytes, invariant);
                        }
                    }
                }
                put_text(&mut bytes, &image.counter_when_the_pool_checker_is_clean);
            }
        }
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVED_IMAGE_MAGIC.len())? != OBSERVED_IMAGE_MAGIC {
            return None;
        }
        let red_item = read_optional_text(&mut reader)?;
        let counts = read_counts(&mut reader)?;
        let image_to_judge = match reader.u8()? {
            0 => None,
            1 => {
                let foundation_digest: [u8; DIGEST_BYTES] =
                    reader.take(DIGEST_BYTES)?.try_into().ok()?;
                let persisted_length = reader.count()?;
                let mut persisted = Vec::with_capacity(persisted_length);
                for byte in reader.take(persisted_length)? {
                    persisted.push(match byte {
                        0 => false,
                        1 => true,
                        _ => return None,
                    });
                }
                let (empty_pool, writes_on_the_crash_image, segments) =
                    read_pool_writes_and_segments(&mut reader)?;
                if !empty_pool.devices.is_empty() || !segments.is_empty() {
                    return None;
                }
                let invariants_judged = match reader.u8()? {
                    0 => InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
                    1 => {
                        let mut invariants = Vec::new();
                        for _ in 0..reader.count()? {
                            invariants.push(implemented_invariant_named(&read_text(&mut reader)?)?);
                        }
                        InvariantsJudgedOnTheImageAfterTheObserver::Only(invariants)
                    }
                    _ => return None,
                };
                let counter_when_the_pool_checker_is_clean = read_text(&mut reader)?;
                Some(ImageToJudgeAfterTheObserver {
                    foundation_digest,
                    persisted,
                    writes_on_the_crash_image,
                    invariants_judged,
                    counter_when_the_pool_checker_is_clean,
                })
            }
            _ => return None,
        };
        (reader.at == bytes.len()).then_some(Self {
            red_item,
            counts,
            image_to_judge,
        })
    }
}

/// 观察者录入段一块里一个状态落在哪一份观察结果上。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ObservationOfStates {
    /// 只记计数的观察者：这个状态的红项（观察者 panic 的那一句）与计数。
    Counted {
        red_item: Option<String>,
        counts: Vec<(String, u64)>,
    },
    /// 交回镜像的观察者：那张恢复镜像的去重键（结果在看过的恢复镜像那张表里）。
    ImageObserved {
        deduplication_key: [u8; DIGEST_BYTES],
    },
}

/// 观察者录入段一块：去重之后的观察结果清单、每个状态的下标、按发布分的态数（计数向量里观察者计数与不变量那两节空着）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ObserverBlockRecord {
    observations: Vec<ObservationOfStates>,
    observation_of_each_state: Vec<u32>,
    count_vector: BlockCountVector,
}

impl ObserverBlockRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = OBSERVER_BLOCK_MAGIC.to_vec();
        bytes.extend_from_slice(
            &u64::try_from(self.observations.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        for observation in &self.observations {
            match observation {
                ObservationOfStates::Counted { red_item, counts } => {
                    bytes.push(0);
                    put_optional_text(&mut bytes, red_item.as_deref());
                    put_counts(&mut bytes, counts);
                }
                ObservationOfStates::ImageObserved { deduplication_key } => {
                    bytes.push(1);
                    bytes.extend_from_slice(deduplication_key);
                }
            }
        }
        bytes.extend_from_slice(
            &u64::try_from(self.observation_of_each_state.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        for index in &self.observation_of_each_state {
            bytes.extend_from_slice(&index.to_le_bytes());
        }
        let count_vector = crate::verdict_store::count_vector_bytes(&self.count_vector);
        bytes.extend_from_slice(
            &u64::try_from(count_vector.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&count_vector);
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVER_BLOCK_MAGIC.len())? != OBSERVER_BLOCK_MAGIC {
            return None;
        }
        let mut observations = Vec::new();
        for _ in 0..reader.count()? {
            observations.push(match reader.u8()? {
                0 => ObservationOfStates::Counted {
                    red_item: read_optional_text(&mut reader)?,
                    counts: read_counts(&mut reader)?,
                },
                1 => ObservationOfStates::ImageObserved {
                    deduplication_key: reader.take(DIGEST_BYTES)?.try_into().ok()?,
                },
                _ => return None,
            });
        }
        let mut observation_of_each_state = Vec::new();
        for _ in 0..reader.count()? {
            let index = reader.u32()?;
            if usize::try_from(index).ok()? >= observations.len() {
                return None;
            }
            observation_of_each_state.push(index);
        }
        let count_vector = crate::verdict_store::count_vector_from_bytes(reader.bytes()?)?;
        (reader.at == bytes.len()).then_some(Self {
            observations,
            observation_of_each_state,
            count_vector,
        })
    }
}

/// 判段在观察者交回的一张盘上跑池级 checker 的结论：判哪几条里违例的、GPU 判不了的原因（第三轮正文 H6：记一条、判红）、在哪判的。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ImageJudgementRecord {
    violated_invariants: Vec<String>,
    could_not_judge: Option<String>,
    judged_on: JudgingDevice,
}

impl ImageJudgementRecord {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = OBSERVED_IMAGE_JUDGEMENT_MAGIC.to_vec();
        bytes.extend_from_slice(
            &u64::try_from(self.violated_invariants.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        for invariant in &self.violated_invariants {
            put_text(&mut bytes, invariant);
        }
        put_optional_text(&mut bytes, self.could_not_judge.as_deref());
        let device = self.judged_on.to_attribute_bytes();
        bytes.extend_from_slice(
            &u64::try_from(device.len())
                .expect("长度装得进 u64")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&device);
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(OBSERVED_IMAGE_JUDGEMENT_MAGIC.len())? != OBSERVED_IMAGE_JUDGEMENT_MAGIC {
            return None;
        }
        let mut violated_invariants = Vec::new();
        for _ in 0..reader.count()? {
            violated_invariants.push(read_text(&mut reader)?);
        }
        let could_not_judge = read_optional_text(&mut reader)?;
        let judged_on = JudgingDevice::from_attribute_bytes(reader.bytes()?)?;
        (reader.at == bytes.len()).then_some(Self {
            violated_invariants,
            could_not_judge,
            judged_on,
        })
    }

    /// 这一份结论在那个状态的判定向量里的红项：违例的不变量逐条、GPU 判不了的那一句。
    fn red_items(&self, observer_name: &ObserverName) -> Vec<String> {
        let mut items: Vec<String> = self
            .violated_invariants
            .iter()
            .map(|invariant| {
                observer_red_item(
                    observer_name,
                    &format!("观察者交回的盘上池级 checker 判违例：{invariant}"),
                )
            })
            .collect();
        if let Some(reason) = &self.could_not_judge {
            items.push(observer_red_item(
                observer_name,
                &format!("GPU 判不了观察者交回的盘：{reason}"),
            ));
        }
        items
    }

    /// 这一份是判器判出来的结论（能存进库、下一趟同一把判器版本照复用），还是「GPU 判不了」：建不出判表、表越过内核容量不是判器的结论，
    /// 建表那一截的代码（这份文件）不在判器那一把版本里，存了下一趟同一把版本照复用、建表改好了也还是判不了；这一趟照样判红，不存，下一趟重判。
    fn is_a_conclusion_of_the_judge(&self) -> bool {
        self.could_not_judge.is_none()
    }
}

/// 判段这一批判过的盘里存进库的那几张（去重键，结论字节）：只存判器判出来的结论（[`ImageJudgementRecord::is_a_conclusion_of_the_judge`]）。
fn image_judgements_to_store(
    images: &[ImageAfterTheObserverToJudge<'_>],
    judged: &[ImageJudgementRecord],
) -> Vec<([u8; DIGEST_BYTES], Vec<u8>)> {
    images
        .iter()
        .zip(judged)
        .filter(|(_, judgement)| judgement.is_a_conclusion_of_the_judge())
        .map(|(image, judgement)| (image.deduplication_key, judgement.to_bytes()))
        .collect()
}

/// 观察者拼好的一块存不存进库：块里每张盘的结论都是判器判出来的才存；有一张 GPU 判不了，这一块照样判红、计数，不存（下一趟重判那一张、重拼这一块）。
fn block_conclusion_is_kept_in_the_library(
    judgement_of_each_observation: &[Option<ImageJudgementRecord>],
) -> bool {
    judgement_of_each_observation
        .iter()
        .flatten()
        .all(ImageJudgementRecord::is_a_conclusion_of_the_judge)
}

// ------------------------------ 录入段 ------------------------------

/// 观察者录入段一批看过的恢复镜像攒到多少字节就落一次盘（攒着的记录在内存里，挂载续做的写一张镜像几百 KiB）。
const OBSERVED_IMAGES_FLUSHED_AT_BYTES: usize = 32 << 20;

/// 一个观察者录入一段状态（块的一段，一条核对线程）交回的：逐状态落在哪一份观察结果上、按发布分的态数、这一段的账。
struct RecordedObserverPiece {
    local: Range<u64>,
    observation_of_each_state: Vec<ObservationOfStates>,
    state_counts: StateCountsOfABlock,
    tally: ObserverRecordingTally,
}

/// 录入段的这一份看一个状态的观察者要的：这条流的去重摘要器与基底摘要（交回镜像的观察者才要）、这个进程这一趟认领过的去重键、库。
struct ObserverRecordingContext<'context, 'store> {
    digester: Option<&'context RecoveredImageBytesDigester<'context>>,
    foundation_digest: [u8; DIGEST_BYTES],
    claimed_keys: &'context std::sync::Mutex<std::collections::HashSet<[u8; DIGEST_BYTES]>>,
    store: &'context std::sync::Mutex<&'store mut VerdictStore>,
}

/// 录入段在 CPU 上看第 `point_index` 个点的块内序号 `local` 那一段：逐状态造崩溃镜像，看 journal 恢复一遍、交给观察者（观察者真跑实现：
/// 可写挂载、续做；这一截只能在 CPU 上，豁免表 `.claude/gate.d/gpu-exemptions.tsv` 的 writable-mount、continuation 两行；
/// 只记计数的观察者对恢复报告下的断言另提一行，见报告）。交回镜像的观察者按恢复镜像的字节去重：这个键这一趟别的状态认领过、或库里存着的，
/// 不再看；新认领的看完攒着、攒够一批原子地写进库。观察者 panic 的状态记成红项。
// gpu-exempt: writable-mount
// gpu-exempt: continuation
// gpu-exempt: observer-assertions-on-the-recovery-report
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "逐状态看、去重、核观察者交回的写、攒批落盘是这一段的全部步骤，拆开要把攒着的批与账来回传；参数是流、计划、点、块、段、观察者、版本与库，各是各的东西"
)]
fn record_one_observer_piece(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    point_index: usize,
    block: &Range<u64>,
    local: Range<u64>,
    observer: &ObserverOnTheNodes<'_>,
    versions: &ObserverVersions,
    context: &ObserverRecordingContext<'_, '_>,
) -> Result<RecordedObserverPiece, String> {
    let point = &plan.points[point_index];
    let block_key = point.block_key(block);
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let library_error = |error: VerdictStoreError| format!("观察者录入段读写库失败：{error:?}");
    let mut observation_of_each_state =
        Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
    let mut state_counts = StateCountsOfABlock::default();
    let mut tally = ObserverRecordingTally::default();
    let mut pending_records: Vec<([u8; DIGEST_BYTES], Vec<u8>)> = Vec::new();
    let mut pending_bytes = 0usize;
    let mut failure: Option<String> = None;
    let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
    let stream_state_count =
        crate::crash::visit_crash_state_range_of_the_read_back_flow_with_slices(
            flow.base,
            flow.writes,
            flow.segments,
            &expansion,
            plan.segment_slices(),
            ordinals,
            &mut |ordinal, publish, image| {
                state_counts.count_the_publish_of_a_state(publish);
                if failure.is_some() {
                    observation_of_each_state.push(ObservationOfStates::Counted {
                        red_item: None,
                        counts: Vec::new(),
                    });
                    return;
                }
                let observation = match &observer.observe {
                    ObserverCode::CountsOnly(observe) => {
                        let report = recover(image, JournalPolicy::Consult);
                        let mut counts_of_this_state = CrashPointReplayObserverCounts::default();
                        let observed =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                observe(ordinal, image, &report, &mut counts_of_this_state);
                            }));
                        tally.states_counted_here += 1;
                        match observed {
                            Ok(()) => ObservationOfStates::Counted {
                                red_item: None,
                                counts: counts_of_this_state
                                    .iter()
                                    .map(|(name, count)| (name.to_string(), count))
                                    .collect(),
                            },
                            Err(payload) => ObservationOfStates::Counted {
                                red_item: Some(observer_red_item(
                                    &observer.observer_name,
                                    &panic_message_of(payload.as_ref()),
                                )),
                                counts: Vec::new(),
                            },
                        }
                    }
                    ObserverCode::HandsBackAnImage {
                        observe,
                        deduplication,
                    } => {
                        let digester = context
                            .digester
                            .expect("有交回镜像的观察者时录入段先建了去重摘要器");
                        let image_digest = digester.image_digest(&image.persisted);
                        let offset_in_block = ordinal - point.ordinals.start - block.start;
                        let deduplication_key = match deduplication {
                            ObserverDeduplication::ByTheRecoveredImageBytes => {
                                observed_image_deduplication_key(observer, versions, &image_digest)
                            }
                            ObserverDeduplication::OffForTheSelfProof => {
                                observed_image_key_without_deduplication(
                                    observer,
                                    versions,
                                    &image_digest,
                                    &block_key,
                                    offset_in_block,
                                )
                            }
                        };
                        let newly_claimed = context
                            .claimed_keys
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .insert(deduplication_key);
                        let already_in_the_library = newly_claimed
                            && match context
                                .store
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .read_observed_image(&deduplication_key)
                            {
                                Ok(stored) => stored.is_some(),
                                Err(error) => {
                                    failure = Some(library_error(error));
                                    true
                                }
                            };
                        if newly_claimed && !already_in_the_library {
                            let report = recover(image, JournalPolicy::Consult);
                            let mut counts_of_this_state =
                                CrashPointReplayObserverCounts::default();
                            let observed =
                                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                    observe(ordinal, image, &report, &mut counts_of_this_state)
                                }));
                            let record = match observed {
                                Ok(image_after) => {
                                    let mut rebuilt = pool_of_the_crash_image(
                                        image.base,
                                        image.writes,
                                        &image.persisted,
                                    );
                                    rebuilt.apply_writes(&image_after.writes_on_the_crash_image);
                                    if pools_hold_the_same_bytes(
                                        &rebuilt,
                                        &image_after.disks_after_the_observer,
                                    ) {
                                        ObservedImageRecord {
                                            red_item: None,
                                            counts: counts_of_this_state
                                                .iter()
                                                .map(|(name, count)| (name.to_string(), count))
                                                .collect(),
                                            image_to_judge: Some(ImageToJudgeAfterTheObserver {
                                                foundation_digest: context.foundation_digest,
                                                persisted: image.persisted.clone(),
                                                writes_on_the_crash_image: image_after
                                                    .writes_on_the_crash_image,
                                                invariants_judged: image_after.invariants_judged,
                                                counter_when_the_pool_checker_is_clean: image_after
                                                    .counter_when_the_pool_checker_is_clean,
                                            }),
                                        }
                                    } else {
                                        ObservedImageRecord {
                                            red_item: Some(observer_red_item(
                                                &observer.observer_name,
                                                "观察者交回的写叠回崩溃镜像，与它停下时的盘字节对不上（录制流漏了写）：判段重建不出它判的那张盘",
                                            )),
                                            counts: Vec::new(),
                                            image_to_judge: None,
                                        }
                                    }
                                }
                                Err(payload) => ObservedImageRecord {
                                    red_item: Some(observer_red_item(
                                        &observer.observer_name,
                                        &panic_message_of(payload.as_ref()),
                                    )),
                                    counts: Vec::new(),
                                    image_to_judge: None,
                                },
                            };
                            let bytes = record.to_bytes();
                            pending_bytes += bytes.len();
                            pending_records.push((deduplication_key, bytes));
                            tally.images_observed_here += 1;
                            if pending_bytes >= OBSERVED_IMAGES_FLUSHED_AT_BYTES {
                                if let Err(error) = context
                                    .store
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                                    .store_observed_images(&pending_records)
                                {
                                    failure = Some(library_error(error));
                                }
                                pending_records.clear();
                                pending_bytes = 0;
                            }
                        } else {
                            tally.states_reusing_an_observed_image += 1;
                        }
                        ObservationOfStates::ImageObserved { deduplication_key }
                    }
                };
                observation_of_each_state.push(observation);
            },
        );
    assert!(
        stream_state_count >= point.ordinals.end,
        "这一段的序号越过了整条流的状态数"
    );
    if let Some(failure) = failure {
        return Err(failure);
    }
    context
        .store
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .store_observed_images(&pending_records)
        .map_err(library_error)?;
    Ok(RecordedObserverPiece {
        local,
        observation_of_each_state,
        state_counts,
        tally,
    })
}

/// 观察者的录入段（用户 2026-10-13 定「观察者改走两段」）：每个观察者、计划里落在它节点上的每个点、这台机器领的（`is_mine`）每一块，
/// 库里这一把观察者版本录过、或这一把结论版本已经存着的不再录；没录的切成核对线程数那么多段、一段一个线程看（[`record_one_observer_piece`]），
/// 拼回一块写进库：去重之后的观察结果清单、每个状态的下标、按发布分的态数。交回镜像的观察者看过的恢复镜像先于块落盘（块在、它引的镜像必在），
/// 被打断后重跑同一条命令只补没录的块，看过的镜像照库里的复用；每录一块之前问一次 `stop_is_requested`，叫停了就停在块边上交回
/// （`stopped_early`）。录入段不判池级 checker。
///
/// # Errors
/// 库读写失败。
///
/// # Panics
/// `versions` 与 `observers` 不一样长；看一段的核对线程自己 panic（观察者的 panic 在线程里接住，记成红项）。
#[allow(
    clippy::too_many_lines,
    reason = "建去重摘要器与基底、逐观察者逐块切段并行看、拼块写库是录入段的全部步骤，拆开要把摘要器、认领表与账来回传"
)]
pub(crate) fn record_the_observers_of_the_flow(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
    is_mine: &dyn Fn(u64) -> bool,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<ObserverRecordingTally, String> {
    assert_eq!(
        observers.len(),
        versions.len(),
        "每个观察者各一组版本、同序"
    );
    let library_error = |error: VerdictStoreError| format!("观察者录入段读写库失败：{error:?}");
    let mut tally = ObserverRecordingTally::default();
    if observers.is_empty() {
        return Ok(tally);
    }
    let threads = checking_threads();
    let hands_back_an_image = observers.iter().any(|observer| match observer.observe {
        ObserverCode::CountsOnly(_) => false,
        ObserverCode::HandsBackAnImage { .. } => true,
    });
    // 基底与去重摘要器：枚举写表与判段读回的流照同一套规则建（`crash::crash_state_enumeration_tables`，与展开方式无关）
    let not_expanded =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let enumerated_writes = if hands_back_an_image {
        crate::crash::crash_state_enumeration_tables(
            flow.base,
            flow.writes,
            flow.segments,
            &not_expanded,
        )
        .enumerated_writes
    } else {
        Vec::new()
    };
    let foundation_digest = if hands_back_an_image {
        let bytes = observer_foundation_bytes(flow.base, &enumerated_writes);
        let digest = sha256_digest(&bytes);
        store
            .store_observer_foundation(&digest, &bytes)
            .map_err(library_error)?;
        digest
    } else {
        [0; DIGEST_BYTES]
    };
    let digester =
        hands_back_an_image.then(|| RecoveredImageBytesDigester::of(flow.base, &enumerated_writes));
    let recording_started = std::time::Instant::now();
    let mut progress_last_printed = std::time::Instant::now();
    for (observer, observer_versions) in observers.iter().zip(versions) {
        let kind = ConclusionKind::Observer {
            observer_name: observer.observer_name.clone(),
        };
        let name = observer.observer_name.as_str().to_string();
        let claimed_keys = std::sync::Mutex::new(std::collections::HashSet::new());
        let mut block_sequence = 0u64;
        for (point_index, point) in plan.points.iter().enumerate() {
            let is_observed = observer
                .nodes
                .iter()
                .any(|node| node.holds_the_point(point));
            for local in point.blocks() {
                let this_block = block_sequence;
                block_sequence += 1;
                if !is_observed || !is_mine(this_block) {
                    continue;
                }
                if stop_is_requested() {
                    tally.stopped_early = true;
                    return Ok(tally);
                }
                let key = point.block_key(&local);
                let recorded_already = store
                    .read_observer_block(&key, &kind, &observer_versions.observer)
                    .map_err(library_error)?
                    .is_some();
                let concluded_already = store
                    .conclusions_missing_for_block(
                        &key,
                        &[(kind.clone(), observer_versions.conclusion)],
                    )
                    .map_err(library_error)?
                    .is_empty();
                if recorded_already || concluded_already {
                    tally.blocks_already_recorded += 1;
                    continue;
                }
                let piece_length = (local.end - local.start)
                    .div_ceil(u64::try_from(threads).expect("段数装得进 u64"))
                    .max(1);
                let mut pieces: Vec<Range<u64>> = Vec::new();
                let mut start = local.start;
                while start < local.end {
                    let end = (start + piece_length).min(local.end);
                    pieces.push(start..end);
                    start = end;
                }
                let shared_store = std::sync::Mutex::new(&mut *store);
                let context = ObserverRecordingContext {
                    digester: digester.as_ref(),
                    foundation_digest,
                    claimed_keys: &claimed_keys,
                    store: &shared_store,
                };
                let recorded: Vec<Result<RecordedObserverPiece, String>> =
                    std::thread::scope(|scope| {
                        let handles: Vec<_> = pieces
                            .iter()
                            .cloned()
                            .map(|piece| {
                                let context = &context;
                                let block = &local;
                                scope.spawn(move || {
                                    record_one_observer_piece(
                                        flow,
                                        plan,
                                        point_index,
                                        block,
                                        piece,
                                        observer,
                                        observer_versions,
                                        context,
                                    )
                                })
                            })
                            .collect();
                        handles
                            .into_iter()
                            .map(|handle| {
                                handle
                                    .join()
                                    .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
                            })
                            .collect()
                    });
                let mut recorded: Vec<RecordedObserverPiece> =
                    recorded.into_iter().collect::<Result<_, _>>()?;
                recorded.sort_by_key(|piece| piece.local.start);
                let mut observations: Vec<ObservationOfStates> = Vec::new();
                let mut index_of_each_observation: std::collections::HashMap<
                    ObservationOfStates,
                    u32,
                > = std::collections::HashMap::new();
                let mut observation_of_each_state: Vec<u32> = Vec::with_capacity(
                    usize::try_from(local.end - local.start).expect("一块装得进 usize"),
                );
                let mut state_counts = StateCountsOfABlock::default();
                let mut next_expected = local.start;
                for piece in recorded {
                    assert_eq!(piece.local.start, next_expected, "块的段没有接上");
                    next_expected = piece.local.end;
                    for observation in piece.observation_of_each_state {
                        let index = *index_of_each_observation
                            .entry(observation.clone())
                            .or_insert_with(|| {
                                observations.push(observation);
                                u32::try_from(observations.len() - 1)
                                    .expect("一块最多 2^16 个状态，观察结果的清单装得进 u32")
                            });
                        observation_of_each_state.push(index);
                    }
                    state_counts.absorb(piece.state_counts);
                    tally.absorb(&piece.tally);
                }
                assert_eq!(next_expected, local.end, "块的段没有拼满");
                let block_record = ObserverBlockRecord {
                    observations,
                    observation_of_each_state,
                    count_vector: state_counts.into_count_vector(local.end - local.start),
                };
                store
                    .store_observer_block(
                        &key,
                        &kind,
                        &observer_versions.observer,
                        &block_record.to_bytes(),
                    )
                    .map_err(library_error)?;
                tally.blocks_recorded_here += 1;
                tally.states_recorded_here += local.end - local.start;
                if progress_last_printed.elapsed()
                    >= std::time::Duration::from_secs(
                        crate::crash::PROGRESS_LINE_INTERVAL_IN_SECONDS,
                    )
                {
                    progress_last_printed = std::time::Instant::now();
                    eprintln!(
                        "CRASH_AMPLIFICATION_OBSERVER_RECORDING observer={name} point={} block_start={} blocks_recorded_here={} states_recorded_here={} images_observed_here={} states_reusing_an_observed_image={} seconds_recording={:.0}",
                        point.name,
                        local.start,
                        tally.blocks_recorded_here,
                        tally.states_recorded_here,
                        tally.images_observed_here,
                        tally.states_reusing_an_observed_image,
                        recording_started.elapsed().as_secs_f64()
                    );
                }
            }
        }
    }
    Ok(tally)
}

// ------------------------------ 判段：观察者交回的盘上的池级 checker ------------------------------

/// 判段在观察者交回的盘上跑池级 checker 走哪一路：配了 GPU 在卡上判（判不了的照第三轮正文 H6 记一条、判红），没配由 CPU 判
/// （CPU 判器只做参照与配置关 GPU 时的路）。
pub(crate) enum PoolCheckerOfTheImagesAfterTheObservers {
    OnTheCpu,
    #[cfg(feature = "gpu")]
    OnTheGpu {
        cards: CardsInUse,
        taken_from: GpuCardsSource,
    },
}

/// 判段要判的一张观察者交回的盘：去重键、它的基底、持久集合、观察者发的写、判哪几条。
struct ImageAfterTheObserverToJudge<'judge> {
    deduplication_key: [u8; DIGEST_BYTES],
    foundation: &'judge ObserverFoundation,
    image: &'judge ImageToJudgeAfterTheObserver,
}

impl PoolCheckerOfTheImagesAfterTheObservers {
    fn judged_by(&self) -> JudgedBy {
        match self {
            Self::OnTheCpu => JudgedBy::CpuJudge,
            #[cfg(feature = "gpu")]
            Self::OnTheGpu { .. } => JudgedBy::GpuJudge,
        }
    }

    /// 判这几张盘，交回与 `images` 同序的结论。
    ///
    /// # Errors
    /// 配了 GPU 而卡用不上、借不到显存；判表进库读回失败。
    #[cfg_attr(
        not(feature = "gpu"),
        allow(
            unused_variables,
            clippy::unnecessary_wraps,
            reason = "没编进 gpu 特性时只有 CPU 那一路：它不写库（判表只在卡上那一路进库）、不报错"
        )
    )]
    fn judge(
        &self,
        store: &mut VerdictStore,
        images: &[ImageAfterTheObserverToJudge<'_>],
    ) -> Result<Vec<ImageJudgementRecord>, String> {
        match self {
            Self::OnTheCpu => Ok(judge_the_images_after_the_observers_on_the_cpu(images)),
            #[cfg(feature = "gpu")]
            Self::OnTheGpu { cards, taken_from } => {
                judge_the_images_after_the_observers_on_the_gpu(store, cards, *taken_from, images)
            }
        }
    }
}

/// 观察者交回的盘：基底叠上这个状态持久了的写，再叠上观察者发的写。
fn pool_after_the_observer(image: &ImageAfterTheObserverToJudge<'_>) -> MemoryPool {
    let mut pool = pool_of_the_crash_image(
        &image.foundation.base,
        &image.foundation.enumerated_writes,
        &image.image.persisted,
    );
    pool.apply_writes(&image.image.writes_on_the_crash_image);
    pool
}

/// CPU 判器那一路（配置关 GPU 时）：逐张重建盘、跑池级 checker，按核对线程数并行。
fn judge_the_images_after_the_observers_on_the_cpu(
    images: &[ImageAfterTheObserverToJudge<'_>],
) -> Vec<ImageJudgementRecord> {
    let threads = checking_threads().max(1);
    let chunk_length = images.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let handles: Vec<_> = images
            .chunks(chunk_length)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|image| {
                            let pool = pool_after_the_observer(image);
                            let violated_invariants =
                                singlefs_checker::walk::check_pool_image(&pool)
                                    .into_iter()
                                    .filter(|(invariant, _)| {
                                        image.image.invariants_judged.includes(invariant)
                                    })
                                    .filter_map(|(invariant, verdict)| {
                                        match verdict {
                                    singlefs_checker::image::InvariantVerdict::Violated(_) => {
                                        Some(invariant.to_string())
                                    }
                                    singlefs_checker::image::InvariantVerdict::Holds
                                    | singlefs_checker::image::InvariantVerdict::NotApplicable(_)
                                    | singlefs_checker::image::InvariantVerdict::PartiallyJudged(
                                        _,
                                    ) => None,
                                }
                                    })
                                    .collect();
                            ImageJudgementRecord {
                                violated_invariants,
                                could_not_judge: None,
                                judged_on: JudgingDevice::Cpu,
                            }
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|payload| std::panic::resume_unwind(payload))
            })
            .collect()
    })
}

/// 一张观察者交回的盘在判表的写表里的一次写是哪一次：同一基底的几张盘持久了同一次枚举写、或观察者发了逐字节相同的写，判表里只放一份
/// （甲报告问题 3 的分组：挂载写相同的几张盘共用一份写，掩码只差持久集合）。
#[cfg(any(feature = "gpu", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum WriteIdentityInTheJudgeTable {
    /// 基底枚举写表里的第几次写。
    EnumeratedWrite(usize),
    /// 观察者在崩溃镜像上发的一次写，按盘、起点、种类、FUA 与内容取的摘要（[`digest_of_a_write_the_observer_issued`]）。
    WriteTheObserverIssued([u8; DIGEST_BYTES]),
}

/// 观察者交回的一张盘落成判表时的一次写：它是哪一次、写的什么。
#[cfg(any(feature = "gpu", test))]
#[derive(Clone, Copy)]
struct WriteOfAnImageAfterTheObserver<'image> {
    identity: WriteIdentityInTheJudgeTable,
    write: &'image RetainedWrite,
}

/// 观察者发的一次写的摘要：盘、起点、种类码、FUA、内容（清零只带长度）；几张盘发的写这几样逐字节相同就是同一次。
#[cfg(any(feature = "gpu", test))]
fn digest_of_a_write_the_observer_issued(write: &RetainedWrite) -> [u8; DIGEST_BYTES] {
    let mut message = b"singlefs write the observer issued 1\n".to_vec();
    message.extend_from_slice(&write.device.0.to_le_bytes());
    message.extend_from_slice(&write.offset.0.to_le_bytes());
    message.extend_from_slice(&crate::crash_judge_tables::step_kind_code(write.kind).to_le_bytes());
    message.push(u8::from(write.is_force_unit_access));
    match &write.contents {
        WrittenContents::Bytes(bytes) => {
            message.push(1);
            message.extend_from_slice(bytes);
        }
        WrittenContents::Zeros { length } => {
            message.push(0);
            message.extend_from_slice(&length.to_le_bytes());
        }
    }
    sha256_digest(&message)
}

/// 观察者交回的一张盘落成判表要的写：这个状态持久了的枚举写（按枚举写表次序）接上观察者发的写（按发的次序），同一位置（同一块盘、
/// 同一个起点）上的几次写并成最后落盘的那一次——后面有一次同起点、至少一样长的写，前一次的每个字节都被它盖住，不进判表（后面那次短的盖不全，
/// 两次都留）。照留下的这几次按次序叠在基底上，与 [`pool_after_the_observer`] 叠出的盘逐字节相同；判表一个落点最多
/// [`crate::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION`] 次写，挂载把系统配置槽写了几十次的盘不并就建不出表。
#[cfg(any(feature = "gpu", test))]
fn landed_writes_of_the_image_merged_on_each_location<'image>(
    image: &ImageAfterTheObserverToJudge<'image>,
) -> Vec<WriteOfAnImageAfterTheObserver<'image>> {
    let landed: Vec<WriteOfAnImageAfterTheObserver<'image>> = image
        .foundation
        .enumerated_writes
        .iter()
        .enumerate()
        .zip(&image.image.persisted)
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|((write_index, write), _)| WriteOfAnImageAfterTheObserver {
            identity: WriteIdentityInTheJudgeTable::EnumeratedWrite(write_index),
            write,
        })
        .chain(image.image.writes_on_the_crash_image.iter().map(|write| {
            WriteOfAnImageAfterTheObserver {
                identity: WriteIdentityInTheJudgeTable::WriteTheObserverIssued(
                    digest_of_a_write_the_observer_issued(write),
                ),
                write,
            }
        }))
        .collect();
    // 从最后落盘的那一次往前看：一个位置上更晚的写里最长的那一次有多长
    let mut longest_later_write_by_location: BTreeMap<(u32, u64), u64> = BTreeMap::new();
    let mut kept_from_the_last: Vec<WriteOfAnImageAfterTheObserver<'image>> =
        Vec::with_capacity(landed.len());
    for landed_write in landed.iter().rev() {
        let length = landed_write.write.length_in_bytes();
        let longest_later_write = longest_later_write_by_location
            .entry((landed_write.write.device.0, landed_write.write.offset.0))
            .or_insert(0);
        if *longest_later_write < length {
            kept_from_the_last.push(*landed_write);
        }
        *longest_later_write = (*longest_later_write).max(length);
    }
    kept_from_the_last.reverse();
    kept_from_the_last
}

/// 一组同基底的盘建出的判表：组里每张盘在交来的那一批里的位置（判表第 k 个状态就是第 k 张，持久掩码在判表的给定状态清单里）、
/// 判表的写表（几张盘并过之后的写去重：枚举写按枚举写表次序在前，观察者发的写按头一次出现的次序在后）、判表。
#[cfg(any(feature = "gpu", test))]
struct JudgeTableOfImagesAfterTheObservers {
    positions: Vec<usize>,
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "判表的写表留给单测核每张盘照掩码叠出来与原盘逐字节相同；GPU 那一路只用判表本身"
        )
    )]
    writes: Vec<RetainedWrite>,
    tables: crate::crash_judge_tables::JudgeTables,
}

/// 一批观察者交回的盘排出来的判表：建出了的一张，或单张盘也建不出（判段照第三轮正文 H6 记判不了、判红，不退回 CPU 判）。
#[cfg(any(feature = "gpu", test))]
enum PlannedJudgeTableOfImagesAfterTheObservers {
    Built(Box<JudgeTableOfImagesAfterTheObservers>),
    NotBuildableForOneImage { position: usize, reason: String },
}

/// 一组盘为什么建不出一张判表。
#[cfg(any(feature = "gpu", test))]
#[derive(Debug)]
enum JudgeTableOfAGroupNotBuildable {
    /// 两张盘里有两次彼此重叠的写（字节范围交叠、同一块盘）先后相反：一张写表排不出两种次序，分到两张表里。单张盘不会撞上它（照它自己的次序排）。
    OverlappingWritesInOppositeOrders,
    Refused(crate::crash_judge_tables::JudgeTablesRefusal),
}

/// 一张判表里最多几张盘：一个落点的版本至多是表里盘的张数，不超过一个落点的版本数上限。
#[cfg(any(feature = "gpu", test))]
const IMAGES_AFTER_THE_OBSERVERS_PER_JUDGE_TABLE_AT_MOST: usize =
    crate::crash_judge_tables::MAXIMUM_VERSIONS_PER_LOCATION;

/// 同一基底的盘按次序几张一组（每张盘的写先照 [`landed_writes_of_the_image_merged_on_each_location`] 并过）：一组并起来的不同写不超过判表掩码的宽
/// （[`crate::crash_facts::MAXIMUM_ENUMERATED_WRITES`]）的一半、同一位置上不同的写不超过一个落点的写数上限的一半、一组不超过
/// [`IMAGES_AFTER_THE_OBSERVERS_PER_JUDGE_TABLE_AT_MOST`] 张。一半：建表时每次原地覆写在枚举写表末尾接一条撕裂镜像（`crash::crash_state_enumeration_tables`），
/// 它也罩着那个落点、也占掩码的一位，判表里每次写一段（[`judge_table_of_a_group_of_images_after_the_observers`]）所以不另接重放；
/// 别的建表时才撞上的上限，由建表那一步对半再建。
#[cfg(any(feature = "gpu", test))]
fn packed_groups_of_images_sharing_their_writes(
    merged_writes_of_each_image: &[Vec<WriteOfAnImageAfterTheObserver<'_>>],
    positions: &[usize],
) -> Vec<Vec<usize>> {
    use crate::crash_facts::MAXIMUM_ENUMERATED_WRITES;
    use crate::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION;
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut group: Vec<usize> = Vec::new();
    let mut identities_in_the_group: std::collections::HashSet<WriteIdentityInTheJudgeTable> =
        std::collections::HashSet::new();
    let mut identities_by_location: BTreeMap<(u32, u64), usize> = BTreeMap::new();
    for position in positions {
        let mut new_identities: std::collections::HashSet<WriteIdentityInTheJudgeTable> =
            std::collections::HashSet::new();
        let mut new_identities_by_location: BTreeMap<(u32, u64), usize> = BTreeMap::new();
        for landed_write in &merged_writes_of_each_image[*position] {
            if !identities_in_the_group.contains(&landed_write.identity)
                && new_identities.insert(landed_write.identity)
            {
                *new_identities_by_location
                    .entry((landed_write.write.device.0, landed_write.write.offset.0))
                    .or_insert(0) += 1;
            }
        }
        let fits_in_the_group = group.len() < IMAGES_AFTER_THE_OBSERVERS_PER_JUDGE_TABLE_AT_MOST
            && identities_in_the_group.len() + new_identities.len()
                <= MAXIMUM_ENUMERATED_WRITES / 2
            && new_identities_by_location
                .iter()
                .all(|(location, new_writes)| {
                    identities_by_location.get(location).copied().unwrap_or(0) + new_writes
                        <= MAXIMUM_WRITES_PER_LOCATION / 2
                });
        if !fits_in_the_group && !group.is_empty() {
            groups.push(std::mem::take(&mut group));
            identities_in_the_group.clear();
            identities_by_location.clear();
            // 这张盘开一组新的：它自己的写全算新的
            new_identities.clear();
            new_identities_by_location.clear();
            for landed_write in &merged_writes_of_each_image[*position] {
                if new_identities.insert(landed_write.identity) {
                    *new_identities_by_location
                        .entry((landed_write.write.device.0, landed_write.write.offset.0))
                        .or_insert(0) += 1;
                }
            }
        }
        group.push(*position);
        identities_in_the_group.extend(new_identities);
        for (location, new_writes) in new_identities_by_location {
            *identities_by_location.entry(location).or_insert(0) += new_writes;
        }
    }
    if !group.is_empty() {
        groups.push(group);
    }
    groups
}

/// 一组同基底的盘建一张给定状态的判表：写表是这几张盘并过之后的写去重（枚举写按枚举写表次序在前、观察者发的写按头一次出现的次序在后），
/// 每次写一段、不展开（同一段里原地覆写之后重叠的写建表时各接一份重放，同一位置上几十次写放一段会接出平方那么多份）；
/// 第 k 个状态的持久掩码是第 k 张盘留下的那几次写。每张盘照掩码按写表次序叠在基底上，与它自己的次序叠出的字节相同：
/// 枚举写在每张盘里本来就按枚举写表次序，观察者的写都在枚举写之后，剩下观察者的写之间两张盘先后相反、又彼此重叠的，这一组排不出。
///
/// # Errors
/// 两张盘重叠的写先后相反；建表拒了（[`crate::crash_judge_tables::JudgeTablesRefusal`]）。
#[cfg(any(feature = "gpu", test))]
fn judge_table_of_a_group_of_images_after_the_observers(
    images: &[ImageAfterTheObserverToJudge<'_>],
    merged_writes_of_each_image: &[Vec<WriteOfAnImageAfterTheObserver<'_>>],
    group: &[usize],
) -> Result<JudgeTableOfImagesAfterTheObservers, JudgeTableOfAGroupNotBuildable> {
    use crate::crash_judge_tables::{
        build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams, JudgeFlow,
    };
    let foundation = images[group[0]].foundation;
    let mut enumerated_writes_used: BTreeSet<usize> = BTreeSet::new();
    let mut writes_the_observers_issued: Vec<([u8; DIGEST_BYTES], &RetainedWrite)> = Vec::new();
    let mut issued_already: std::collections::HashSet<[u8; DIGEST_BYTES]> =
        std::collections::HashSet::new();
    // 这一组的盘都没留下一次写（都就是基底）时，写表里放一次哪张盘都不持久的枚举写，判表的写表不空（被判根的下标指得到一次写）
    if group
        .iter()
        .all(|position| merged_writes_of_each_image[*position].is_empty())
        && !foundation.enumerated_writes.is_empty()
    {
        enumerated_writes_used.insert(0);
    }
    for position in group {
        for landed_write in &merged_writes_of_each_image[*position] {
            match landed_write.identity {
                WriteIdentityInTheJudgeTable::EnumeratedWrite(write_index) => {
                    enumerated_writes_used.insert(write_index);
                }
                WriteIdentityInTheJudgeTable::WriteTheObserverIssued(digest) => {
                    if issued_already.insert(digest) {
                        writes_the_observers_issued.push((digest, landed_write.write));
                    }
                }
            }
        }
    }
    let mut table_index_of_each_identity: std::collections::HashMap<
        WriteIdentityInTheJudgeTable,
        usize,
    > = std::collections::HashMap::new();
    let mut writes: Vec<RetainedWrite> =
        Vec::with_capacity(enumerated_writes_used.len() + writes_the_observers_issued.len());
    for write_index in &enumerated_writes_used {
        table_index_of_each_identity.insert(
            WriteIdentityInTheJudgeTable::EnumeratedWrite(*write_index),
            writes.len(),
        );
        writes.push(foundation.enumerated_writes[*write_index].clone());
    }
    for (digest, write) in &writes_the_observers_issued {
        table_index_of_each_identity.insert(
            WriteIdentityInTheJudgeTable::WriteTheObserverIssued(*digest),
            writes.len(),
        );
        writes.push((*write).clone());
    }
    let table_indexes_of_each_image: Vec<Vec<usize>> = group
        .iter()
        .map(|position| {
            merged_writes_of_each_image[*position]
                .iter()
                .map(|landed_write| table_index_of_each_identity[&landed_write.identity])
                .collect()
        })
        .collect();
    // 每张盘里彼此重叠的两次写，在写表里要与它自己的先后相同
    for (position, table_indexes) in group.iter().zip(&table_indexes_of_each_image) {
        let landed_writes = &merged_writes_of_each_image[*position];
        let mut by_start: Vec<usize> = (0..landed_writes.len()).collect();
        by_start.sort_by_key(|order| {
            (
                landed_writes[*order].write.device.0,
                landed_writes[*order].write.offset.0,
            )
        });
        for (rank, earlier_by_start) in by_start.iter().enumerate() {
            let first = landed_writes[*earlier_by_start].write;
            let first_end = first.offset.0 + first.length_in_bytes();
            for later_by_start in &by_start[rank + 1..] {
                let second = landed_writes[*later_by_start].write;
                if second.device != first.device || second.offset.0 >= first_end {
                    break;
                }
                let (earlier, later) = if earlier_by_start < later_by_start {
                    (*earlier_by_start, *later_by_start)
                } else {
                    (*later_by_start, *earlier_by_start)
                };
                if table_indexes[earlier] > table_indexes[later] {
                    return Err(JudgeTableOfAGroupNotBuildable::OverlappingWritesInOppositeOrders);
                }
            }
        }
    }
    let segments: Vec<Vec<usize>> = (0..writes.len())
        .map(|write_index| vec![write_index])
        .collect();
    let not_expanded =
        |_segment_index: usize, _segment: &[usize]| CrashPointReplaySegmentExpansion::NotExpanded;
    let enumerated_write_count = crate::crash::crash_state_enumeration_tables(
        &foundation.base,
        &writes,
        &segments,
        &not_expanded,
    )
    .enumerated_writes
    .len();
    let persisted_sets: Vec<Vec<bool>> = table_indexes_of_each_image
        .iter()
        .map(|table_indexes| {
            let mut persisted = vec![false; enumerated_write_count];
            for table_index in table_indexes {
                persisted[*table_index] = true;
            }
            persisted
        })
        .collect();
    let tables = build_judge_tables_for_the_given_persisted_sets_broken_at_the_seams(
        &JudgeFlow {
            base: &foundation.base,
            writes: &writes,
            segments: &segments,
            judged_root_index: 0,
            versions: &[],
            expansion: &not_expanded,
        },
        &[],
        persisted_sets,
        &[],
    )
    .map_err(JudgeTableOfAGroupNotBuildable::Refused)?;
    Ok(JudgeTableOfImagesAfterTheObservers {
        positions: group.to_vec(),
        writes,
        tables,
    })
}

/// 单张盘也建不出判表时判红的那一句：建表拒在哪，这张盘是哪一类（基底、并过之后几次写、其中持久的枚举写与观察者发的写各几次、
/// 清零写几次、同一位置上最多几次）。
#[cfg(any(feature = "gpu", test))]
fn reason_no_judge_table_holds_the_image_after_the_observer(
    image: &ImageAfterTheObserverToJudge<'_>,
    merged_writes: &[WriteOfAnImageAfterTheObserver<'_>],
    not_buildable: &JudgeTableOfAGroupNotBuildable,
) -> String {
    let enumerated_writes = merged_writes
        .iter()
        .filter(|landed_write| {
            matches!(
                landed_write.identity,
                WriteIdentityInTheJudgeTable::EnumeratedWrite(_)
            )
        })
        .count();
    let zero_fills = merged_writes
        .iter()
        .filter(|landed_write| matches!(landed_write.write.contents, WrittenContents::Zeros { .. }))
        .count();
    let mut writes_by_location: BTreeMap<(u32, u64), usize> = BTreeMap::new();
    for landed_write in merged_writes {
        *writes_by_location
            .entry((landed_write.write.device.0, landed_write.write.offset.0))
            .or_insert(0) += 1;
    }
    let most_writes_on_one_location = writes_by_location.values().copied().max().unwrap_or(0);
    let not_buildable_text = match not_buildable {
        JudgeTableOfAGroupNotBuildable::Refused(refusal) => format!("{refusal:?}"),
        JudgeTableOfAGroupNotBuildable::OverlappingWritesInOppositeOrders => {
            "两次重叠的写先后相反".to_string()
        }
    };
    format!(
        "建不出 GPU 判表：{not_buildable_text}（单张盘也建不出，并过同一位置上的写之后还是）；这张盘：基底 {}，并过之后 {} 次写，其中持久的枚举写 {enumerated_writes} 次、观察者发的写 {} 次（观察者共发 {} 次），清零写 {zero_fills} 次，同一位置上最多 {most_writes_on_one_location} 次",
        hexadecimal_text(&image.image.foundation_digest[..8]),
        merged_writes.len(),
        merged_writes.len() - enumerated_writes,
        image.image.writes_on_the_crash_image.len()
    )
}

/// 一批观察者交回的盘排成几张给定状态的判表：每张盘的写先照 [`landed_writes_of_the_image_merged_on_each_location`] 并过同一位置上的几次，
/// 同一基底的盘按次序几张一组（[`packed_groups_of_images_sharing_their_writes`]，交回的写逐字节相同的共用一份写），一组建不出就对半再建，
/// 单张盘也建不出的记一句判不了（[`reason_no_judge_table_holds_the_image_after_the_observer`]）。每张盘恰好落在一张判表或一句判不了里。
/// 排出一张交一张给 `hand_on`（判完一张再建下一张，内存里只放一张判表）；`hand_on` 报错就停下、照原样交回。
///
/// # Errors
/// `hand_on` 报的错。
#[cfg(any(feature = "gpu", test))]
fn plan_the_judge_tables_of_the_images_after_the_observers(
    images: &[ImageAfterTheObserverToJudge<'_>],
    hand_on: &mut dyn FnMut(PlannedJudgeTableOfImagesAfterTheObservers) -> Result<(), String>,
) -> Result<(), String> {
    let merged_writes_of_each_image: Vec<Vec<WriteOfAnImageAfterTheObserver<'_>>> = images
        .iter()
        .map(landed_writes_of_the_image_merged_on_each_location)
        .collect();
    let mut positions_by_foundation: BTreeMap<[u8; DIGEST_BYTES], Vec<usize>> = BTreeMap::new();
    for (position, image) in images.iter().enumerate() {
        positions_by_foundation
            .entry(image.image.foundation_digest)
            .or_default()
            .push(position);
    }
    let mut pending_groups: Vec<Vec<usize>> = positions_by_foundation
        .into_values()
        .flat_map(|positions| {
            packed_groups_of_images_sharing_their_writes(&merged_writes_of_each_image, &positions)
        })
        .collect();
    pending_groups.reverse();
    while let Some(group) = pending_groups.pop() {
        match judge_table_of_a_group_of_images_after_the_observers(
            images,
            &merged_writes_of_each_image,
            &group,
        ) {
            Ok(table) => hand_on(PlannedJudgeTableOfImagesAfterTheObservers::Built(Box::new(
                table,
            )))?,
            Err(not_buildable) => {
                if group.len() > 1 {
                    let (front, back) = group.split_at(group.len() / 2);
                    pending_groups.push(back.to_vec());
                    pending_groups.push(front.to_vec());
                } else {
                    hand_on(
                        PlannedJudgeTableOfImagesAfterTheObservers::NotBuildableForOneImage {
                            position: group[0],
                            reason: reason_no_judge_table_holds_the_image_after_the_observer(
                                &images[group[0]],
                                &merged_writes_of_each_image[group[0]],
                                &not_buildable,
                            ),
                        },
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// GPU 判器那一路：盘先排成判表（[`plan_the_judge_tables_of_the_images_after_the_observers`]：每张盘同一位置上的写并过、同一基底交回的写相同的
/// 共用一份写、建不出就对半再建），判表先入库、再读库上卡（[`crate::crash_library::record_the_judge_tables_and_read_them_back`]），
/// 只读池级 checker 那几个字。单张盘也建不出的照 H6 记判不了（那一句写明是哪一类盘），不退回 CPU 判。
///
/// # Errors
/// 卡用不上、借不到显存；判表进库读回失败。
#[cfg(feature = "gpu")]
fn judge_the_images_after_the_observers_on_the_gpu(
    store: &mut VerdictStore,
    cards: &CardsInUse,
    taken_from: GpuCardsSource,
    images: &[ImageAfterTheObserverToJudge<'_>],
) -> Result<Vec<ImageJudgementRecord>, String> {
    use crate::crash_judge_dispatch::{
        mebibytes_a_judge_needs, states_per_scratch_lane, GpuCrashJudge, PackedJudgeTables,
    };
    let mut judgements: Vec<Option<ImageJudgementRecord>> = vec![None; images.len()];
    let mut next_card = 0usize;
    plan_the_judge_tables_of_the_images_after_the_observers(images, &mut |planned| {
        let JudgeTableOfImagesAfterTheObservers {
            positions: group,
            writes: _,
            tables,
        } = match planned {
            PlannedJudgeTableOfImagesAfterTheObservers::Built(built) => *built,
            PlannedJudgeTableOfImagesAfterTheObservers::NotBuildableForOneImage {
                position,
                reason,
            } => {
                judgements[position] = Some(ImageJudgementRecord {
                    violated_invariants: Vec::new(),
                    could_not_judge: Some(reason),
                    judged_on: JudgingDevice::Cpu,
                });
                return Ok(());
            }
        };
        let packed = crate::crash_library::record_the_judge_tables_and_read_them_back(
            store,
            &PackedJudgeTables::of(&tables),
        )?;
        drop(tables);
        let (verdicts, judged_on) = match taken_from {
            GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
                if cards.is_empty() {
                    return Err(
                        "配置开了 GPU，观察者交回的盘上的池级 checker 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器"
                            .to_string(),
                    );
                }
                let card = &cards[next_card % cards.len()];
                next_card += 1;
                let judge = GpuCrashJudge::on_card_with_packed_tables(card, &packed, usize::MAX);
                (
                    judge.judge_states(0..packed.state_count()),
                    JudgingDevice::Gpu {
                        card_type: card.name.clone(),
                        driver_version: card.driver_version.clone(),
                    },
                )
            }
            GpuCardsSource::TheCardPoolOfThisProcess => {
                let pool = the_card_pool_this_run_takes_its_cards_from();
                let states_per_dispatch =
                    states_per_scratch_lane(pool.smallest_max_storage_buffer_binding_size());
                let lease = pool
                    .lease_the_first_card_with_room_now(
                        PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                        mebibytes_a_judge_needs(&packed, states_per_dispatch),
                    )
                    .map_err(|refusal| {
                        let reason = given_states_lease_refusal_as_a_run_condition_not_met(
                            "observer-images",
                            &refusal,
                        );
                        println!("{reason}");
                        reason
                    })?;
                let judge = GpuCrashJudge::on_card_with_packed_tables(
                    lease.card(),
                    &packed,
                    states_per_dispatch,
                );
                let verdicts = judge.judge_states(0..packed.state_count());
                let judged_on = JudgingDevice::Gpu {
                    card_type: lease.card().name.clone(),
                    driver_version: lease.card().driver_version.clone(),
                };
                drop(judge);
                drop(lease);
                (verdicts, judged_on)
            }
        };
        for (position, verdict) in group.iter().zip(&verdicts) {
            let words = crate::crash_judge_gpu::cross_check_words_of_gpu_verdict(verdict);
            let violated_bits = pool_checker_violated_bits(verdict);
            let violated_invariants = IMPLEMENTED_INVARIANTS
                .iter()
                .enumerate()
                .filter(|(bit, invariant)| {
                    violated_bits & (1u128 << bit) != 0
                        && images[*position]
                            .image
                            .invariants_judged
                            .includes(invariant)
                })
                .map(|(_, invariant)| (*invariant).to_string())
                .collect();
            judgements[*position] = Some(ImageJudgementRecord {
                violated_invariants,
                could_not_judge: words
                    .gpu_could_not_judge
                    .then(|| "结论里判不了那一位置上了（表越过内核容量）".to_string()),
                judged_on: judged_on.clone(),
            });
        }
        Ok(())
    })?;
    Ok(judgements
        .into_iter()
        .map(|judgement| judgement.expect("每一张盘都落在某一组里判过或记了判不了"))
        .collect())
}

/// 观察者的判段（用户 2026-10-13 定）：每个观察者、计划里落在它节点上的每个点、这台机器领的（`is_mine`）每一块，库里这一把结论版本存着的
/// 照复用；没有的读录入段那一块（没有就报「先跑录入段」）、读它引的看过的恢复镜像，观察者交回的盘里这一把判器版本还没判过的
/// 交 `pool_checker` 判（同一张盘只判一次）、结论写进库，再逐状态拼回一块存：判定向量（观察者 panic 的那一句、盘上池级 checker 的违例与判不了）、
/// 违例、计数向量（观察者计数加判干净那一个计数、按发布分的态数；不变量那一节空着）、在哪判的（盘上的池级 checker 在哪判的；只记计数的观察者
/// 是 CPU）。存完把这条流挂了观察者的每一块（这一趟拼的与复用的）读回来，计数按观察者合起来交回，调用方照它判观察者计数（J7，每一趟都判）。
/// GPU 判不了的盘（建不出判表、表越过内核容量）照样判红，它的那一句与带着它的那一块都不存进库（不是判器的结论，下一趟重判，
/// [`ImageJudgementRecord::is_a_conclusion_of_the_judge`]）；库里早先存着的「判不了」也不复用。
/// 判段不跑恢复、不调观察者、不收用例内存里的流。
///
/// # Errors
/// 库读写失败；录入段没录某一块、或它引的恢复镜像与基底不在库里（先跑录入段）；判定向量超过 256 种；配了 GPU 而卡用不上。
///
/// # Panics
/// `versions` 与 `observers` 不一样长。
#[allow(
    clippy::too_many_lines,
    reason = "读录入块与镜像、判盘、写判定、逐状态拼块、读回合计是判段的全部步骤，拆开要把几张表与账来回传"
)]
pub(crate) fn judge_the_observers_of_the_flow(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    observers: &[ObserverOnTheNodes<'_>],
    versions: &[ObserverVersions],
    pool_checker: &PoolCheckerOfTheImagesAfterTheObservers,
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<ObservingOfTheRun, String> {
    assert_eq!(
        observers.len(),
        versions.len(),
        "每个观察者各一组版本、同序"
    );
    let library_error = |error: VerdictStoreError| format!("观察者判段读写库失败：{error:?}");
    let mut observing = ObservingOfTheRun::default();
    let observing_started = std::time::Instant::now();
    let mut progress_last_printed = std::time::Instant::now();
    let mut foundations: BTreeMap<[u8; DIGEST_BYTES], ObserverFoundation> = BTreeMap::new();
    for (observer, observer_versions) in observers.iter().zip(versions) {
        assert_eq!(
            observer_versions.pool_checker,
            pool_checker.judged_by(),
            "观察者那一把池级 checker 的版本按这一趟判盘的那一路取"
        );
        let kind = ConclusionKind::Observer {
            observer_name: observer.observer_name.clone(),
        };
        let pool_checker_kind = observer_versions.pool_checker.conclusion_kind();
        let name = observer.observer_name.as_str().to_string();
        let mut block_sequence = 0u64;
        for point in &plan.points {
            let is_observed = observer
                .nodes
                .iter()
                .any(|node| node.holds_the_point(point));
            for local in point.blocks() {
                let this_block = block_sequence;
                block_sequence += 1;
                if !is_observed || !is_mine(this_block) {
                    continue;
                }
                let key = point.block_key(&local);
                let already_stored = store
                    .conclusions_missing_for_block(
                        &key,
                        &[(kind.clone(), observer_versions.conclusion)],
                    )
                    .map_err(library_error)?
                    .is_empty();
                let mut composed_but_not_stored: Option<ConclusionBlockRecord> = None;
                if !already_stored {
                    let block_record = store
                        .read_observer_block(&key, &kind, &observer_versions.observer)
                        .map_err(library_error)?
                        .ok_or_else(|| {
                            format!(
                                "观察者 {name} 在点 {} 块 {} 的录入段那一块不在库里：先跑录入段（{RUN_STAGE_ENVIRONMENT_VARIABLE}=record，或不设、两段先后跑）",
                                point.name, local.start
                            )
                        })?;
                    let block_record =
                        ObserverBlockRecord::from_bytes(&block_record).ok_or_else(|| {
                            format!(
                                "观察者 {name} 在点 {} 块 {} 的录入块解不出",
                                point.name, local.start
                            )
                        })?;
                    // 块里每一份观察结果读回：只记计数的照块里存的；交回镜像的读看过的恢复镜像那一份
                    let mut image_records: Vec<Option<ObservedImageRecord>> =
                        Vec::with_capacity(block_record.observations.len());
                    for observation in &block_record.observations {
                        image_records.push(match observation {
                            ObservationOfStates::Counted { .. } => None,
                            ObservationOfStates::ImageObserved { deduplication_key } => {
                                let bytes = store
                                    .read_observed_image(deduplication_key)
                                    .map_err(library_error)?
                                    .ok_or_else(|| {
                                        format!(
                                            "观察者 {name} 的录入块引的恢复镜像 {} 不在库里：先跑录入段",
                                            hexadecimal_text(&deduplication_key[..8])
                                        )
                                    })?;
                                Some(ObservedImageRecord::from_bytes(&bytes).ok_or_else(|| {
                                    format!(
                                        "观察者 {name} 看过的恢复镜像 {} 解不出",
                                        hexadecimal_text(&deduplication_key[..8])
                                    )
                                })?)
                            }
                        });
                    }
                    for record in image_records.iter().flatten() {
                        if let Some(image) = &record.image_to_judge {
                            if let std::collections::btree_map::Entry::Vacant(vacant) =
                                foundations.entry(image.foundation_digest)
                            {
                                let bytes = store
                                    .read_observer_foundation(&image.foundation_digest)
                                    .map_err(library_error)?
                                    .ok_or_else(|| "观察者基底不在库里：先跑录入段".to_string())?;
                                vacant.insert(
                                    ObserverFoundation::from_bytes(&bytes)
                                        .ok_or_else(|| "观察者基底解不出".to_string())?,
                                );
                            }
                        }
                    }
                    // 盘上的池级 checker：这一把判器版本判过的取库里的，没判过的一批交 pool_checker 判、写进库
                    let mut judgement_of_each_observation: Vec<Option<ImageJudgementRecord>> =
                        vec![None; block_record.observations.len()];
                    let mut to_judge: Vec<(usize, [u8; DIGEST_BYTES])> = Vec::new();
                    for (position, (observation, record)) in block_record
                        .observations
                        .iter()
                        .zip(&image_records)
                        .enumerate()
                    {
                        let (
                            ObservationOfStates::ImageObserved { deduplication_key },
                            Some(record),
                        ) = (observation, record)
                        else {
                            continue;
                        };
                        if record.image_to_judge.is_none() {
                            continue;
                        }
                        match store
                            .read_observed_image_judgement(
                                deduplication_key,
                                &pool_checker_kind,
                                &observer_versions.pool_checker_version,
                            )
                            .map_err(library_error)?
                        {
                            Some(bytes) => {
                                let stored_judgement = ImageJudgementRecord::from_bytes(&bytes)
                                    .ok_or_else(|| {
                                        "观察者交回的盘上池级 checker 的结论解不出".to_string()
                                    })?;
                                // 早先存进库的「GPU 判不了」不是判器的结论：不复用，这一趟重判
                                if stored_judgement.is_a_conclusion_of_the_judge() {
                                    judgement_of_each_observation[position] =
                                        Some(stored_judgement);
                                    observing.image_judgements_reused += 1;
                                } else {
                                    to_judge.push((position, *deduplication_key));
                                }
                            }
                            None => to_judge.push((position, *deduplication_key)),
                        }
                    }
                    if !to_judge.is_empty() {
                        let images: Vec<ImageAfterTheObserverToJudge<'_>> = to_judge
                            .iter()
                            .map(|(position, deduplication_key)| {
                                let image = image_records[*position]
                                    .as_ref()
                                    .and_then(|record| record.image_to_judge.as_ref())
                                    .expect("要判的盘挑的都是带盘的那几份");
                                ImageAfterTheObserverToJudge {
                                    deduplication_key: *deduplication_key,
                                    foundation: foundations
                                        .get(&image.foundation_digest)
                                        .expect("要判的盘引的基底上面读进来了"),
                                    image,
                                }
                            })
                            .collect();
                        let judged = pool_checker.judge(store, &images)?;
                        let records = image_judgements_to_store(&images, &judged);
                        store
                            .store_observed_image_judgements(
                                &pool_checker_kind,
                                &observer_versions.pool_checker_version,
                                &records,
                            )
                            .map_err(library_error)?;
                        observing.images_judged_here +=
                            u64::try_from(judged.len()).expect("装得进 u64");
                        observing.images_judged_on_the_gpu_here += u64::try_from(
                            judged
                                .iter()
                                .filter(|judgement| {
                                    matches!(judgement.judged_on, JudgingDevice::Gpu { .. })
                                })
                                .count(),
                        )
                        .expect("装得进 u64");
                        for ((position, _), judgement) in to_judge.iter().zip(judged) {
                            judgement_of_each_observation[*position] = Some(judgement);
                        }
                    }
                    // 每一份观察结果的红项与计数
                    let mut items_of_each_observation: Vec<Vec<String>> =
                        Vec::with_capacity(block_record.observations.len());
                    let mut counts_of_each_observation: Vec<Vec<(String, u64)>> =
                        Vec::with_capacity(block_record.observations.len());
                    for ((observation, record), judgement) in block_record
                        .observations
                        .iter()
                        .zip(&image_records)
                        .zip(&judgement_of_each_observation)
                    {
                        let (red_item, mut counts) = match (observation, record) {
                            (ObservationOfStates::Counted { red_item, counts }, _) => {
                                (red_item.clone(), counts.clone())
                            }
                            (ObservationOfStates::ImageObserved { .. }, Some(record)) => {
                                (record.red_item.clone(), record.counts.clone())
                            }
                            (ObservationOfStates::ImageObserved { .. }, None) => {
                                unreachable!("交回镜像的那几份上面都读回了它看过的恢复镜像")
                            }
                        };
                        let mut items: Vec<String> = red_item.into_iter().collect();
                        if let (Some(judgement), Some(image)) = (
                            judgement,
                            record
                                .as_ref()
                                .and_then(|record| record.image_to_judge.as_ref()),
                        ) {
                            let judgement_items = judgement.red_items(&observer.observer_name);
                            if judgement_items.is_empty() {
                                counts.push((
                                    image.counter_when_the_pool_checker_is_clean.clone(),
                                    1,
                                ));
                            }
                            items.extend(judgement_items);
                        }
                        items_of_each_observation.push(items);
                        counts_of_each_observation.push(counts);
                    }
                    let judged_on = judgement_of_each_observation
                        .iter()
                        .flatten()
                        .map(|judgement| judgement.judged_on.clone())
                        .next()
                        .unwrap_or(JudgingDevice::Cpu);
                    let number_of_each_observation: Vec<VerdictVectorNumber> =
                        items_of_each_observation
                            .iter()
                            .map(|items| {
                                store
                                    .register_verdict_vector(&encoded_verdict_of_the_red_items(
                                        items,
                                    ))
                                    .map_err(library_error)
                            })
                            .collect::<Result<_, _>>()?;
                    let mut numbers =
                        Vec::with_capacity(block_record.observation_of_each_state.len());
                    let mut violating = Vec::new();
                    let mut states_of_each_observation =
                        vec![0u64; block_record.observations.len()];
                    for (offset, index) in block_record.observation_of_each_state.iter().enumerate()
                    {
                        let index = usize::try_from(*index).expect("下标装得进 usize");
                        numbers.push(number_of_each_observation[index]);
                        states_of_each_observation[index] += 1;
                        if !items_of_each_observation[index].is_empty() {
                            let offset = u64::try_from(offset).expect("块内序号装得进 u64");
                            violating.push(StateOffsetInBlock(offset));
                            if observing.red_texts.len() < RED_TEXTS_KEPT {
                                observing.red_texts.push((
                                    point.path_number.clone(),
                                    local.start + offset,
                                    items_of_each_observation[index].join("；"),
                                ));
                            }
                        }
                    }
                    assert_eq!(
                        u64::try_from(numbers.len()).expect("装得进 u64"),
                        local.end - local.start,
                        "录入块的状态数与这一块的状态数相同"
                    );
                    let mut count_vector = block_record.count_vector.clone();
                    count_vector.per_invariant = BTreeMap::new();
                    let mut per_observer: BTreeMap<String, u64> = BTreeMap::new();
                    for (counts, states) in counts_of_each_observation
                        .iter()
                        .zip(&states_of_each_observation)
                    {
                        for (counter, count) in counts {
                            *per_observer.entry(counter.clone()).or_insert(0) += count * states;
                        }
                    }
                    count_vector.per_observer = per_observer;
                    let composed = ConclusionBlockRecord {
                        block: VerdictBlock::new(numbers).expect("一块至少一个状态"),
                        violating_states: violating,
                        count_vector,
                        judged_on,
                        recovery_words: None,
                    };
                    if block_conclusion_is_kept_in_the_library(&judgement_of_each_observation) {
                        store
                            .store_conclusion_block(
                                &key,
                                &kind,
                                &observer_versions.conclusion,
                                &composed,
                            )
                            .map_err(library_error)?;
                    } else {
                        // 这一块有盘 GPU 判不了：拼出的这一块照样判红、照样计数，不存进库（下一趟重判），收尾照这一份记
                        composed_but_not_stored = Some(composed);
                    }
                    observing.blocks_observed_here += 1;
                    observing.states_observed_here += local.end - local.start;
                    if progress_last_printed.elapsed()
                        >= std::time::Duration::from_secs(
                            crate::crash::PROGRESS_LINE_INTERVAL_IN_SECONDS,
                        )
                    {
                        progress_last_printed = std::time::Instant::now();
                        eprintln!(
                            "CRASH_AMPLIFICATION_OBSERVING observer={name} point={} block_start={} blocks_observed_here={} states_observed_here={} images_judged_here={} image_judgements_reused={} seconds_observing={:.0}",
                            point.name,
                            local.start,
                            observing.blocks_observed_here,
                            observing.states_observed_here,
                            observing.images_judged_here,
                            observing.image_judgements_reused,
                            observing_started.elapsed().as_secs_f64()
                        );
                    }
                }
                let (red_states, per_observer) = match composed_but_not_stored {
                    Some(composed) => (
                        composed.violating_states.len(),
                        composed.count_vector.per_observer,
                    ),
                    None => {
                        let stored = store
                            .read_conclusion_block(&key, &kind, &observer_versions.conclusion)
                            .map_err(library_error)?
                            .ok_or_else(|| {
                                format!(
                                    "观察者 {name} 在点 {} 块 {} 的结论刚存过、库里却读不回来",
                                    point.name, local.start
                                )
                            })?;
                        (stored.violations.len(), stored.count_vector.per_observer)
                    }
                };
                *observing
                    .states_of_each_observer
                    .entry(name.clone())
                    .or_insert(0) += local.end - local.start;
                *observing
                    .red_states_of_each_observer
                    .entry(name.clone())
                    .or_insert(0) += u64::try_from(red_states).expect("违例数装得进 u64");
                let counts = observing
                    .counts_of_each_observer
                    .entry(name.clone())
                    .or_default();
                for (counter, count) in per_observer {
                    *counts.entry(counter).or_insert(0) += count;
                }
            }
        }
    }
    Ok(observing)
}

// ============================== 计数向量每一趟都判（崩溃点重放合并第三轮判决 J7） ==============================

/// 每个状态归哪次发布，按段的序号区间现算（不逐状态造镜像）：GPU 判器那一路存块时照它记按发布分的态数，每一趟的核也照它算该有多少。
struct PublishesOfTheStates {
    ranges_and_publishes: Vec<(Range<u64>, CrashPointReplayPublishOfState)>,
    state_count: u64,
}

impl PublishesOfTheStates {
    fn of_the_plan(flow: &CrashFlow<'_>, plan: &CrashPlan) -> Self {
        let expansion = |segment_index: usize, segment: &[usize]| {
            plan.segment_expansion(flow, segment_index, segment)
        };
        let (ranges_by_segment, state_count) = crash_state_ranges_by_segment_with_slices(
            flow.base,
            flow.writes,
            flow.segments,
            &expansion,
            plan.segment_slices(),
        );
        Self {
            ranges_and_publishes: ranges_by_segment
                .into_iter()
                .zip(publish_of_each_segment(flow.writes, flow.segments))
                .collect(),
            state_count,
        }
    }

    /// 整条流序号 `ordinals` 那一段里按发布分的态数：每段与它重叠的那几个归这一段的那次发布，最后全部持久那一个（序号是状态数减一，
    /// 不在任何一段的区间里）归 [`CrashPointReplayPublishOfState::EveryWritePersisted`]。
    fn state_counts_of(&self, ordinals: &Range<u64>) -> StateCountsOfABlock {
        let mut counts = StateCountsOfABlock::default();
        for (range, publish) in &self.ranges_and_publishes {
            let overlap = range
                .end
                .min(ordinals.end)
                .saturating_sub(range.start.max(ordinals.start));
            counts.count_states_of_a_publish(*publish, overlap);
        }
        if ordinals.start < self.state_count && ordinals.end >= self.state_count {
            counts
                .count_states_of_a_publish(CrashPointReplayPublishOfState::EveryWritePersisted, 1);
        }
        counts
    }
}

/// 这条流这台机器领的每一块（`is_mine`），`conclusions` 那一处的计数向量与计划对不对得上（J7：闭式、按发布分的态数、逐条不变量计数每一趟都判，
/// 这一趟判的与库里复用的一样判）：按发布分的态数逐项等于按段的序号区间现算的（[`PublishesOfTheStates`]，合起来就是这一块的状态数，
/// 全部块合起来就是整条流的闭式）；清单里每条不变量一行，判到 + 不适用 = 状态数、违例不超过判到。一块对不上一句；判法版本那一套
/// （[`ConclusionsStoredUnder::TheJudgeVersion`]）不存计数向量，不判。还没判的块不判（对账另报）。
///
/// # Errors
/// 库读失败。
pub fn count_vector_disagreements_of_the_flow(
    store: &VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<Vec<String>, VerdictStoreError> {
    let ConclusionsStoredUnder::TheConclusionKind {
        kind,
        version_of_each_point,
    } = conclusions
    else {
        return Ok(Vec::new());
    };
    let publishes = PublishesOfTheStates::of_the_plan(flow, plan);
    let mut disagreements = Vec::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let version = ConclusionsStoredUnder::conclusion_version_of_the_point(
            version_of_each_point,
            point_index,
        );
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_mine(this_block) {
                continue;
            }
            let Some(stored) =
                store.read_conclusion_block(&point.block_key(&local), kind, &version)?
            else {
                continue;
            };
            let states = local.end - local.start;
            let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
            let expected_states_by_publish = publishes
                .state_counts_of(&ordinals)
                .into_count_vector(states)
                .states_by_publish;
            let place = format!("点 {} 块 {}", point.name, local.start);
            if stored.count_vector.states_by_publish != expected_states_by_publish {
                disagreements.push(format!(
                    "{place}：按发布分的态数 {:?}，按段的区间现算该是 {expected_states_by_publish:?}",
                    stored.count_vector.states_by_publish
                ));
            }
            let invariant_names: Vec<&str> = stored
                .count_vector
                .per_invariant
                .keys()
                .map(String::as_str)
                .collect();
            let mut expected_names: Vec<&str> = IMPLEMENTED_INVARIANTS.to_vec();
            expected_names.sort_unstable();
            if invariant_names != expected_names {
                disagreements.push(format!(
                    "{place}：逐条不变量计数的行与池级 checker 的清单不同（{} 行，清单 {} 条）",
                    invariant_names.len(),
                    expected_names.len()
                ));
            }
            for (invariant, counts) in &stored.count_vector.per_invariant {
                if counts.judged + counts.not_applicable != states
                    || counts.violated > counts.judged
                {
                    disagreements.push(format!(
                        "{place}：{invariant} 判到 {}、违例 {}、不适用 {}，这一块 {states} 个状态",
                        counts.judged, counts.violated, counts.not_applicable
                    ));
                }
            }
        }
    }
    Ok(disagreements)
}

/// 一种结论在一个入口上的版本，与编译时写进二进制的那一份核过（`crash_identity::conclusion_version_checked_against_the_build`）。
///
/// # Errors
/// 从盘上算不出；与编译时的不等或编译时没算（[`ConclusionVersionsNotWritable`]）。
pub(crate) fn conclusion_version_to_write_under(
    repository_root: &Path,
    request: &ConclusionVersionRequest<'_>,
    toolchain: &CrashPointReplayToolchainIdentity,
) -> Result<ConclusionVersion, ConclusionVersionsNotWritable> {
    let check = conclusion_version_checked_against_the_build(repository_root, request, toolchain)
        .map_err(
        |error| ConclusionVersionsNotWritable::NotComputableFromTheTree {
            reason: error.reason,
        },
    )?;
    check.version_to_write_under().ok_or_else(|| {
        ConclusionVersionsNotWritable::AVersionDiffersFromTheBuild {
            request: format!("{request:?}"),
            check: format!("{check:?}"),
        }
    })
}

/// 两台分片时从别的机器取回的库（`SINGLEFS_CRASH_AMPLIFICATION_IMPORT`，冒号分隔的几个目录）按这一趟的计划导：只点名这条流里不归本机
/// （`is_mine` 为假）的块，每块连同两种判器与挂在这个点上的观察者的结论版本、`verifier_digests` 里每一种核对代码的核对行，路径取计划里每个点的；
/// 对方库里没有的跳过。导入的读写随那一份的块数线性，不随库里累计的别的流、别的版本的行涨。交回导进来的块数（写下的加本库已有且相同的）。
///
/// # Errors
/// 要导入的库开不了、导入失败。
#[allow(
    clippy::too_many_arguments,
    reason = "库、仓根、计划、分工、三种结论的版本、核对代码、目录清单，各是各的东西"
)]
pub(crate) fn import_the_planned_blocks_of_the_other_hosts(
    store: &mut VerdictStore,
    repository_root: &Path,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    conclusion_versions: &ConclusionVersionsOfThePlan,
    observers: &[ObserverOnTheNodes<'_>],
    observer_versions: &[ConclusionVersion],
    verifier_digests: &[[u8; DIGEST_BYTES]],
    import_list: &str,
) -> Result<u64, String> {
    let mut planned = crate::verdict_store::PlannedImportKeys {
        conclusions: Vec::new(),
        paths: plan
            .points
            .iter()
            .map(|point| point.path_number.clone())
            .collect(),
        verifier_blocks: Vec::new(),
    };
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let version_of_the_point = |version_of_each_point: &[ConclusionVersion]| {
            *version_of_each_point.get(point_index).expect(
                "每个点各一把版本、与计划的点同序：ConclusionVersionsOfThePlan::of_the_plan 照计划的点建",
            )
        };
        let mut conclusions_of_the_point = vec![
            (
                JudgedBy::GpuJudge.conclusion_kind(),
                version_of_the_point(&conclusion_versions.gpu_judge_of_each_point),
            ),
            (
                JudgedBy::CpuJudge.conclusion_kind(),
                version_of_the_point(&conclusion_versions.cpu_judge_of_each_point),
            ),
        ];
        for (observer, observer_version) in observers.iter().zip(observer_versions) {
            if observer
                .nodes
                .iter()
                .any(|node| node.holds_the_point(point))
            {
                conclusions_of_the_point.push((
                    ConclusionKind::Observer {
                        observer_name: observer.observer_name.clone(),
                    },
                    *observer_version,
                ));
            }
        }
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if is_mine(this_block) {
                continue;
            }
            let key = point.block_key(&local);
            for (kind, conclusion_version) in &conclusions_of_the_point {
                planned
                    .conclusions
                    .push((key.clone(), kind.clone(), *conclusion_version));
            }
            for verifier_digest in verifier_digests {
                planned
                    .verifier_blocks
                    .push((key.clone(), *verifier_digest));
            }
        }
    }
    let mut imported_blocks = 0u64;
    for directory in import_list
        .split(':')
        .filter(|directory| !directory.is_empty())
    {
        let (other, _) = VerdictStore::open_existing(&path_relative_to_repository_root(
            repository_root,
            directory,
        ))
        .map_err(|error| format!("开不了要导入的库 {directory}：{error:?}"))?;
        let tally = store
            .import_the_planned_rows_from(&other, &planned)
            .map_err(|error| format!("导入 {directory} 失败：{error:?}"))?;
        imported_blocks +=
            tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;
    }
    Ok(imported_blocks)
}

/// 这一趟用的卡：编进 `gpu` 特性时是起好上下文的卡，按优先级；没编进来时没有这个东西。
#[cfg(feature = "gpu")]
pub(crate) type CardsInUse = Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>;
#[cfg(not(feature = "gpu"))]
pub(crate) type CardsInUse = ();

/// 起过卡池的进程为第 ② 段在显存账上借的那一笔：活到那一段算完；卡不是从池里来的不借（`None`）。
#[cfg(feature = "gpu")]
type BufferLeaseOnTheCardPool = Option<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>;

/// 一段算完那一刻核账：那一段的缓冲借的那一笔还在这个进程的账上没有（第 10t 批代码轮判决 K4-1：借了又在用完之前还掉要红）；没借是 `None`。
#[cfg(feature = "gpu")]
fn lease_on_the_books_after_its_work(lease: &BufferLeaseOnTheCardPool) -> Option<bool> {
    lease
        .as_ref()
        .map(crate::gpu_unit_checks::gpu_test_start::GpuCardLease::is_on_the_books)
}

/// 这一趟的卡（[`gpu_cards_in_use`]）：这一趟自己起的那几张（起过卡池的是空的：池里的卡借到哪张才在哪张上起设备，
/// 用的时候照池取，文件头第 5 条），与卡从哪来。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时取卡那一份只报错，这个结局只在编进来时造"
    )
)]
pub(crate) struct GpuCardsOfThisRun {
    pub(crate) cards: CardsInUse,
    pub(crate) taken_from: GpuCardsSource,
}

/// 这个进程用哪几张卡：配置点了名的只用这台机器那几张、照配置的次序与额度；没配就是本机能用的卡按空闲显存从大到小。
/// 这个进程起过 GPU 用例共用的卡池（[`crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process`]）就照池里的卡用
/// （池里的卡要与配置对得上，[`the_card_pool_matches_the_configured_cards`]），不另起一套设备，此刻也不登记、不起设备：
/// 借到哪张卡才在哪张上起（第 10t 批代码轮判决 K2-3）。没起过池才自己起。
#[cfg(feature = "gpu")]
pub(crate) fn gpu_cards_in_use(
    configured: Option<&[CardByPriority]>,
    host: CardHost,
) -> Result<GpuCardsOfThisRun, String> {
    if let Some(pool) = crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process()
    {
        the_card_pool_matches_the_configured_cards(
            &pool.cards_in_priority_order_as_configured(),
            configured,
            host,
        )?;
        return Ok(GpuCardsOfThisRun {
            cards: Vec::new(),
            taken_from: GpuCardsSource::TheCardPoolOfThisProcess,
        });
    }
    let cards = match configured {
        Some(configured) => {
            let mine: Vec<(u32, u64)> = configured
                .iter()
                .filter(|card| card.host == host)
                .map(|card| (card.index, card.memory_quota_mebibytes))
                .collect();
            if mine.is_empty() {
                return Err(format!(
                    "配置的 GPU_CARDS_BY_PRIORITY 里没有 {} 这台机器的卡",
                    host.name()
                ));
            }
            let cards = crate::gpu_unit_checks::gpu_cards_by_priority(&mine);
            // 点名的卡少一张都不跑（用户 2026-09-29 定：配了 GPU 就要它干活，不行就明确报错，不接受少用几张的默认降级）
            let usable: Vec<u32> = cards.iter().map(|card| card.index).collect();
            let missing = configured_cards_not_usable(&mine, &usable);
            if !missing.is_empty() {
                return Err(format!(
                    "配置点名给 {} 这台机器的卡里，序号 {missing:?} 用不了（上面 GPU_CARD_NOT_USABLE 那几行）：配置开了 GPU，点名的卡每一张都要用得上，不少用几张照跑",
                    host.name()
                ));
            }
            cards
        }
        None => crate::gpu_unit_checks::usable_gpu_cards(),
    };
    if cards.is_empty() {
        return Err(
            "配置写了 ENABLE_GPU=1，这台机器却没有一张用得了的卡（点了名的卡空闲显存不够，或没有 NVIDIA 独显）"
                .to_string(),
        );
    }
    Ok(GpuCardsOfThisRun {
        cards: cards.into_iter().map(std::sync::Arc::new).collect(),
        taken_from: GpuCardsSource::DevicesStartedForThisRun,
    })
}

/// 池里的卡（`pool_cards`：按池的优先级，每张的序号与配置给它的额度）对不对得上这一趟读到的显卡配置：配置点了名的（`configured` 里
/// 这台机器那几项），池里的卡要与它逐张同序号、同额度，对不上就报错（池与这一趟读到的显卡配置不同，不悄悄换卡）；没配就照池里的卡用。
///
/// # Errors
/// 两份显卡配置对不上。
#[cfg(feature = "gpu")]
fn the_card_pool_matches_the_configured_cards(
    pool_cards: &[(u32, Option<u64>)],
    configured: Option<&[CardByPriority]>,
    host: CardHost,
) -> Result<(), String> {
    if let Some(configured) = configured {
        let mine: Vec<(u32, Option<u64>)> = configured
            .iter()
            .filter(|card| card.host == host)
            .map(|card| (card.index, Some(card.memory_quota_mebibytes)))
            .collect();
        if mine != pool_cards {
            return Err(format!(
                "这个进程的卡池里是 {pool_cards:?}（序号，额度 MiB），这一趟的显卡配置给 {} 这台机器点的是 {mine:?}：两份显卡配置对不上，\
                 不另起一套设备、也不悄悄换卡",
                host.name()
            ));
        }
    }
    Ok(())
}

/// 配置要 GPU 而这份二进制没编进 `gpu` 特性：报错，不悄悄退回 CPU。
#[cfg(not(feature = "gpu"))]
pub(crate) fn gpu_cards_in_use(
    _configured: Option<&[CardByPriority]>,
    _host: CardHost,
) -> Result<GpuCardsOfThisRun, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}

/// 第 ① 段在卡上的入口（`gpu` 特性编进来时调 [`judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from`]：
/// 起过卡池的这一趟草稿区进账，显存不够交 [`GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort`]）。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、流、计划、分工、判法版本、卡、卡的来路、叫停，各是各的东西，凑成结构体只为让签名变短"
)]
pub(crate) fn judge_my_blocks_on_gpu(
    store: &mut VerdictStore,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    tables: &PackedJudgeTablesOfTheRun,
    is_mine: &dyn Fn(u64) -> bool,
    conclusions: &ConclusionsStoredUnder<'_>,
    cards: &CardsInUse,
    cards_taken_from: GpuCardsSource,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<GpuJudgingOfMyBlocks, String> {
    judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(
        store,
        flow,
        plan,
        tables,
        is_mine,
        conclusions,
        cards,
        cards_taken_from,
        stop_is_requested,
    )
}

/// 没编进 `gpu` 特性走不到这里（[`gpu_cards_in_use`] 先报错）。
#[cfg(not(feature = "gpu"))]
#[allow(
    clippy::too_many_arguments,
    reason = "与编进 gpu 特性的那一份同一个签名"
)]
pub(crate) fn judge_my_blocks_on_gpu(
    _store: &mut VerdictStore,
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
    _tables: &PackedJudgeTablesOfTheRun,
    _is_mine: &dyn Fn(u64) -> bool,
    _conclusions: &ConclusionsStoredUnder<'_>,
    _cards: &CardsInUse,
    _cards_taken_from: GpuCardsSource,
    _stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<GpuJudgingOfMyBlocks, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}

/// 第 ② 段在卡上核的结局（[`verify_my_blocks_on_the_first_card`]）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时第 ② 段在卡上的那一份只报错，两种结局只在编进来时造"
    )
)]
enum FactsVerifiedOnACard {
    /// 核完了：`buffers_booked_mebibytes` 是起过卡池的进程为核对的缓冲借的（不借是 `None`），`lease_on_the_books_after_verifying`
    /// 是核完那一刻核账、那一笔还在不在（不借是 `None`）。
    Verified {
        outcome: VerificationOutcome,
        buffers_booked_mebibytes: Option<u64>,
        lease_on_the_books_after_verifying: Option<bool>,
    },
    /// 显存不够（起过卡池的进程此刻借不到核对的缓冲，或卡上 wgpu 报了 Out of Memory）：「本次未跑」那一行已经打了，
    /// 核过的块在库里。
    NotRunTheGpuMemoryWasShort,
}

/// 第 ② 段在卡上：这台机器领到的块都在一张卡上核（第 ① 段的草稿区那时已经放掉，这一段一块一次派活、不占草稿区）。
/// 卡是这一趟自己起的用优先级最高的那张；起过卡池的照这条流的事实表与一块的缓冲（`facts_verifier_buffer_footprint`）在账上借
/// 第一张此刻放得下的，用借来的那张核，借不到打「本次未跑」那一行（文件头第 5 条）。核的时候 wgpu 报 Out of Memory 同样报本次未跑。
///
/// # Errors
/// 库读写失败；起过卡池的进程借缓冲时要的比每张卡空着时能借的都大、卡上的显存读不出。
#[cfg(feature = "gpu")]
fn verify_my_blocks_on_the_first_card(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    facts: &crate::crash_facts::FlowFactsReadBack,
    segment_expansions: &[SegmentExpansionFact],
    my_blocks: &[u64],
    cards: &CardsInUse,
    cards_taken_from: GpuCardsSource,
) -> Result<FactsVerifiedOnACard, String> {
    // 借据活到这一段核完（核对的缓冲在下面那一次调用里建、丢）
    let (card, lease): (_, BufferLeaseOnTheCardPool) = match cards_taken_from {
        GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => (
            cards
                .first()
                .expect("用的卡至少一张（gpu_cards_in_use 判过）")
                .clone(),
            None,
        ),
        GpuCardsSource::TheCardPoolOfThisProcess => {
            let need_mebibytes =
                facts_verifier_buffer_footprint(facts, STATES_PER_BLOCK).mebibytes();
            match the_card_pool_this_run_takes_its_cards_from().lease_the_first_card_with_room_now(
                PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                need_mebibytes,
            ) {
                Ok(lease) => (std::sync::Arc::new(lease.card().clone()), Some(lease)),
                Err(refusal) => {
                    print_the_not_run_line_or_refuse("第 ② 段核对的缓冲", &refusal)?;
                    return Ok(FactsVerifiedOnACard::NotRunTheGpuMemoryWasShort);
                }
            }
        }
    };
    let selected: std::collections::HashSet<u64> = my_blocks.iter().copied().collect();
    eprintln!(
        "CRASH_AMPLIFICATION_CARD name={:?} index={} memory_budget_mebibytes={} blocks={}",
        card.name,
        card.index,
        card.memory_budget_mebibytes(),
        selected.len()
    );
    let on = FactsVerifierOn::Gpu(card.clone());
    // 核对在这条线程上一块接一块派活（verify_blocks_from_these_facts），wgpu 的错误范围接得住
    let tally = match on_the_card_catching_out_of_memory(&card, "第 ② 段核对时", || {
        verify_blocks_from_these_facts(
            store,
            plan,
            facts,
            segment_expansions,
            &|block| selected.contains(&block),
            &on,
        )
    }) {
        Ok(tally) => tally?,
        Err(reason) => {
            println!(
                "{}",
                gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
            );
            return Ok(FactsVerifiedOnACard::NotRunTheGpuMemoryWasShort);
        }
    };
    Ok(FactsVerifiedOnACard::Verified {
        outcome: VerificationOutcome {
            verifier: "gpu",
            tally,
        },
        buffers_booked_mebibytes: lease
            .as_ref()
            .map(crate::gpu_unit_checks::gpu_test_start::GpuCardLease::mebibytes),
        lease_on_the_books_after_verifying: lease_on_the_books_after_its_work(&lease),
    })
}

/// 没编进 `gpu` 特性走不到这里（[`gpu_cards_in_use`] 先报错）。
#[cfg(not(feature = "gpu"))]
fn verify_my_blocks_on_the_first_card(
    _store: &mut VerdictStore,
    _plan: &CrashPlan,
    _facts: &FlowFacts,
    _segment_expansions: &[SegmentExpansionFact],
    _my_blocks: &[u64],
    _cards: &CardsInUse,
    _cards_taken_from: GpuCardsSource,
) -> Result<FactsVerifiedOnACard, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}

/// 只排派活计划那一趟的结局。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时只排计划的那一份只报错，打了那一行的结局只在编进来时造"
    )
)]
enum PlacementOfThisRun {
    /// 打了 `CRASH_AMPLIFICATION_PLACEMENT` 那一行。
    Printed,
    /// 起过卡池的进程此刻借不到试判的草稿区、或试判时卡上报了 Out of Memory：「本次未跑」那一行已经打了，没打 PLACEMENT 行。
    NotRunTheGpuMemoryWasShort,
}

/// 只排派活计划：这一趟要判多少、配置的每张卡按额度带得了多少、按优先级各装多少；这台机器装了状态的卡各试判几次派活量速度。
/// 打一行 `CRASH_AMPLIFICATION_PLACEMENT`（带这一趟录入时库里已判、因此不排的态数 `states_reused`）。别的机器上的卡带得了多少按它的额度算
/// （绑定上限按本机第一张卡估）。起过卡池的这一趟（`cards_taken_from`），本机的卡照池里的卡、每张装几个状态照这一刻能借的量压，
/// 试判之前按装的状态数借草稿区（同第 ① 段，[`scratch_of_each_card_booked_by_where_the_cards_came_from`]），借不到交
/// [`PlacementOfThisRun::NotRunTheGpuMemoryWasShort`]。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、流、计划、分工、判法版本、卡、卡的来路、配置、机器、复用数，各是各的东西，凑成结构体只为让签名变短"
)]
#[allow(
    clippy::too_many_lines,
    reason = "备表、按配置排每张卡、借草稿区、试判量速度、打行，是只排计划那一趟的全部步骤，拆开要把十几个量来回传"
)]
fn print_the_placement(
    store: &VerdictStore,
    plan: &CrashPlan,
    tables: &PackedJudgeTablesOfTheRun,
    is_mine: &dyn Fn(u64) -> bool,
    conclusions: &ConclusionsStoredUnder<'_>,
    cards: &CardsInUse,
    cards_taken_from: GpuCardsSource,
    configured: Option<&[CardByPriority]>,
    host: CardHost,
    states_reused: u64,
) -> Result<PlacementOfThisRun, String> {
    use crate::crash_judge_dispatch::{states_in_flight_within, GpuCrashJudge};
    /// 量完宽度之后计时判几次派活那么多的状态。
    const CALIBRATION_DISPATCHES: u64 = 8;
    let pending = pending_blocks(store, plan, conclusions, is_mine)
        .map_err(|error| format!("读判定块失败：{error:?}"))?;
    let pending_states: u64 = pending
        .iter()
        .map(|(_, local)| local.end - local.start)
        .sum();
    let states_of_the_flow = tables.state_count();
    let local_cards = card_capacities_of_this_run(cards_taken_from, cards, tables)?;
    let local_capacities: Vec<u64> = local_cards
        .iter()
        .map(|card| card.states_in_flight_capacity)
        .collect();
    let limit = local_cards
        .first()
        .expect("用的卡至少一张（gpu_cards_in_use 判过、起过卡池的池里至少一张）")
        .max_storage_buffer_binding_size;
    // 配置的每张卡一行，按优先级：这台机器的取实际起好的那张（点了名而用不了的带得了 0 个），别的机器的按额度算
    let rows: Vec<(CardHost, u32, u64)> = match configured {
        Some(configured) => configured
            .iter()
            .map(|entry| {
                let capacity = if entry.host == host {
                    local_cards
                        .iter()
                        .position(|card| card.index == entry.index)
                        .map_or(0, |position| local_capacities[position])
                } else if tables.mebibytes() * 1024 * 1024 > limit {
                    0
                } else {
                    u64::try_from(states_in_flight_within(
                        limit,
                        entry.memory_quota_mebibytes,
                        tables,
                    ))
                    .expect("装得进 u64")
                };
                (entry.host, entry.index, capacity)
            })
            .collect(),
        None => local_cards
            .iter()
            .zip(&local_capacities)
            .map(|(card, capacity)| (host, card.index, *capacity))
            .collect(),
    };
    let capacities: Vec<u64> = rows.iter().map(|(_, _, capacity)| *capacity).collect();
    let placed = place_states_on_cards_by_priority(pending_states, &capacities);
    let mut states_per_second_on_this_host = 0.0f64;
    // 试判取要判的块里最大的那一块：小块一次派活装不满，量出来的是每次派活的固定用时，不是这张卡的速度
    if let Some((point_index, local)) = pending
        .iter()
        .max_by_key(|(_, local)| local.end - local.start)
    {
        let point = &plan.points[*point_index];
        // 本机每张卡装的状态数（按这一趟卡的次序）：试判的草稿区照它借，借据活到试判完
        let states_placed_on_local_cards: Vec<u64> = local_cards
            .iter()
            .map(|card| {
                rows.iter()
                    .zip(&placed)
                    .find(|((row_host, index, _), _)| *row_host == host && *index == card.index)
                    .map_or(0, |(_, states_placed)| *states_placed)
            })
            .collect();
        // 借据活到试判完（这一段收尾）；本机的卡照装的状态数借，此刻借不到的卡不试判、不等它；本机一张卡都没装状态就不借
        let states_placed_on_this_host: u64 = states_placed_on_local_cards.iter().sum();
        let scratch = if states_placed_on_this_host == 0 {
            None
        } else {
            match scratch_of_each_card_booked_by_where_the_cards_came_from(
                cards_taken_from,
                cards,
                &local_cards,
                tables,
                states_of_the_flow,
                states_placed_on_this_host,
                &states_placed_on_local_cards,
            )? {
                ScratchOfEachCard::Ready(scratch) => Some(scratch),
                ScratchOfEachCard::NotRunTheGpuMemoryWasShort { not_run_line: _ } => {
                    return Ok(PlacementOfThisRun::NotRunTheGpuMemoryWasShort);
                }
            }
        };
        for (card, states_placed) in scratch.iter().flat_map(|scratch| {
            scratch
                .cards_for_the_judges
                .iter()
                .zip(&scratch.states_placed)
        }) {
            if *states_placed == 0 {
                continue;
            }
            // 建判器、试判都在这一层里：卡上报 Out of Memory（判器内核另起的线程上的也算）报本次未跑，不判红（第 10x 批代码轮判决 K1-3）
            let trial = on_the_card_catching_out_of_memory(
                card,
                "只排派活计划那一趟试判时",
                || {
                    let judge = GpuCrashJudge::on_card_with_packed_tables(
                        card,
                        tables,
                        usize::try_from(*states_placed).expect("装得进 usize"),
                    );
                    // 先判一小段让卡量出派活宽度（量宽度那几次派活不算速度），再计时判几次派活那么多
                    let first = point.ordinals.start + local.start;
                    let state_count = judge.state_count();
                    // 量宽度最宽试到这张卡一次带得了的上限，先判的这一段要比它长
                    let warm_end = (first
                        + 2 * u64::try_from(judge.states_per_dispatch()).expect("装得进 u64"))
                    .min(state_count);
                    let _warm = judge.judge_states(first..warm_end);
                    let timed_end = (warm_end
                        + u64::try_from(judge.dispatch_width()).expect("装得进 u64")
                            * CALIBRATION_DISPATCHES)
                        .min(state_count);
                    let started = Instant::now();
                    let words = judge.judge_states(warm_end..timed_end);
                    (words.len(), started.elapsed().as_secs_f64())
                },
            );
            let (states_timed, seconds) = match trial {
                Ok(timed) => timed,
                Err(reason) => {
                    println!(
                        "{}",
                        gpu_not_run_line(PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE, &reason)
                    );
                    return Ok(PlacementOfThisRun::NotRunTheGpuMemoryWasShort);
                }
            };
            if seconds > 0.0 {
                states_per_second_on_this_host += states_timed as f64 / seconds;
            }
        }
    }
    let card_fields: Vec<String> = rows
        .iter()
        .zip(&placed)
        .map(|((row_host, index, capacity), states_placed)| {
            format!("{}:{index}:{capacity}:{states_placed}", row_host.name())
        })
        .collect();
    let placed_on = |wanted: CardHost| -> u64 {
        rows.iter()
            .zip(&placed)
            .filter(|((row_host, _, _), _)| *row_host == wanted)
            .map(|(_, states_placed)| *states_placed)
            .sum()
    };
    println!(
        "CRASH_AMPLIFICATION_PLACEMENT host={} judge=gpu pending_blocks={} pending_states={pending_states} states_reused={states_reused} tables_mebibytes={} cards={} states_placed_on_local={} states_placed_on_peer={} states_per_second_on_this_host={states_per_second_on_this_host:.0} estimated_seconds_on_this_host={:.0}",
        host.name(),
        pending.len(),
        tables.mebibytes(),
        card_fields.join(","),
        placed_on(CardHost::Local),
        placed_on(CardHost::Peer),
        if states_per_second_on_this_host > 0.0 {
            pending_states as f64 / states_per_second_on_this_host
        } else {
            0.0
        }
    );
    Ok(PlacementOfThisRun::Printed)
}

/// 没编进 `gpu` 特性走不到这里（[`gpu_cards_in_use`] 先报错）。
#[cfg(not(feature = "gpu"))]
#[allow(
    clippy::too_many_arguments,
    reason = "与编进 gpu 特性的那一份同一个签名"
)]
fn print_the_placement(
    _store: &VerdictStore,
    _plan: &CrashPlan,
    _tables: &PackedJudgeTablesOfTheRun,
    _is_mine: &dyn Fn(u64) -> bool,
    _conclusions: &ConclusionsStoredUnder<'_>,
    _cards: &CardsInUse,
    _cards_taken_from: GpuCardsSource,
    _configured: Option<&[CardByPriority]>,
    _host: CardHost,
    _states_reused: u64,
) -> Result<PlacementOfThisRun, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}
