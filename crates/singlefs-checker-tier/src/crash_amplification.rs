//! 崩溃放量流水线（里程碑二「增补 4」第一、四、十一项；用户 2026-09-28 定的形态）：
//!
//! 1. **崩溃点与路径**：一条测试流程是从 mkfs 出发、一个崩溃点接一个崩溃点走下来的路径；每段归含它最后一次写的那个点，上游不看下游。每个点的身份是
//!    `<仓内路径>::<崩溃点名称>::<哈希串>`（[`crate::crash_identity`]）。几条流程共用前缀，全部点连成一棵树（用户 2026-09-28：「这就是个 B 树的问题」）：
//!    路径编号是从根到这个点、每个点取输入键（点名、写表、父镜像的摘要）前 16 个十六进制、用 `/` 逐段接起来的串：
//!    前面几段相同就是同一段枝干，一棵子树就是一段连续的键。哈希串（复用键）只由流程定：这一步写下的字节与它起步的镜像；
//!    按 COW 的性质，一个点的输入不变，它的崩溃状态就不变。判法不进键：判定块按（块键，判法版本）存，判法改了旧行留作历史、
//!    只在同一判法版本下复用（用户 2026-09-29 定：结果只绑流程，崩溃点变了或树变了才重测，代码、commit、树都不进编号）。
//!    判器判一个点的状态时另读的流程输入（这个点之前的写表、到这个点为止够得着的版本表、被判的根槽写落在哪、没做完的那次发布的根槽写）
//!    不进复用键与身份（树不动），进这个点的计划哈希、块键（`JudgeInputsOfTheFlow`）：只差这些的两条流不互相复用判定。
//! 2. **先录后核**：录入把每个点的状态按块（点内序号区间）登进库的录入表；核对按批从库里取还没有判定的块，
//!    在 CPU 上（或 CPU + GPU：单元两道校验和上 GPU，与 CPU 对拍）判完、原子地写回判定块与违例；门禁一次读完。
//! 3. **对账**：一轮收尾，录入表里有、判定块表里没有的块就是没核完的（卡掉线、进程被杀），重新分给还在的一方补跑；
//!    领块只在内存里，库里不记谁领了，不会有块被一张死卡占着。
//! 4. **分工与跑法**：块按权重分给几方（一方一台机器的 CPU，或一张卡，权重取空闲显存），各写各的库，最后
//!    [`crate::verdict_store::VerdictStore::import_from`] 并成一个；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。
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
//!    录制流每次写 4 个字、每对重放 2 个、每段 6 个、段里每次写 1 个、每个单元 6 个、每条根 38 个，空表也建一个字），
//!    加一块一趟的头（12 个字）、结论与回读（每个状态各 1 个字），一块按 [`STATES_PER_BLOCK`] 个状态算；核对一块接一块，同时还活着的
//!    照 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD` 算。几个字照 `crash_verify_gpu.rs` 里那几个常数抄的，那边改了布局这里要跟着改。
//!    单元校验和（`unit_checksum_buffer_footprint`）：一个状态一趟，这条流里每次单元写各两段
//!    （`crate::gpu_unit_checks::bytes_crc32c_of_ranges_needs`），核对线程数（[`checking_threads`]）条线程同时派活，
//!    每条同时还活着的照 `PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD` 算。
//!    驱动的上下文与编好的管线算在每张卡第一次借时的驱动那一份里，不另记。
//!    账只记这个进程借的：两个进程同一刻读到同一份空闲、一起借超了，由运行时兜底：第 ① 段卡上建判器、判一块时 wgpu 报 Out of Memory，
//!    这张卡停下、这一块交回给别的卡，判不完才报本次未跑（原因写显存不够，`singlefs_harness::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`）；
//!    第 ② 段在卡上核时、只排派活计划那一趟试判时、卡池在借到的卡上起设备与编单元校验和管线时同样（第 10x 批代码轮判决 K1-3）。
//!    判器内核在另起的线程上编，那里的 Out of Memory 不进卡线程的错误范围：每个设备上装了接 wgpu 没接住的错误的回调，Out of Memory
//!    记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），卡线程上的活 panic 了就照它认是不是显存不够
//!    （`decide_the_work_on_the_card`），按这段活认、不按整台设备认（第 10z 批：别的线程一次 Out of Memory 不把这条线程上真正的 panic
//!    说成显存不够）；设备因为 Out of Memory 丢了，卡池下一次借这张卡时重起设备（驱动那一份在账上重新登记，放不下就报本次未跑）。
//!    第 ① 段由 CPU 判时单元校验和对拍几条线程一起派活，每条线程那一段各自包进接 Out of Memory 的那一层（第 10z 批）：有一段报了，
//!    这一块不入库、不再领下一块，这一趟报本次未跑（`CheckingTally::stopped_because_the_gpu_memory_was_short`，之前判完的块在库里）。
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
//!    崩溃放量分项驱动（`research/scripts/crash-amplification-item.sh`）起的、只有经 [`run_from_environment`] 跑的用例而没有一条取卡池的测试二进制
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
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::root_record::ROOT_MAGIC;
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
use singlefs_harness::gpu_memory_ledger::{
    StopRequestRememberedInThisRun, WorkEndingTogether, WorksEndingTogetherOnADevice,
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

use crate::crash::{
    judge_layer0_state_range_against_the_version_table, layer0_state_ranges_by_segment,
    receive_until_the_senders_are_gone_with_wall_clock_lines, Layer0SegmentExpansion,
    Layer0VersionTable, ReceivedOrWallClockLineDue, WallClockProgressLines,
};
#[cfg(feature = "gpu")]
use crate::crash::{
    LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE, LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
    LAYER0_RED_PASS_POOL_CHECKER, LAYER0_RED_PASS_RECORD_CHECKER,
    RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT, RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};
use crate::crash_facts::{extract_flow_facts, verify_states_from_facts, FlowFacts};
use crate::crash_identity::{
    content_digest_text, judging_code_of_the_judges, step_write_table_with_barriers,
    CoverageReport, CoverageState, JudgingCode, JudgingCodeError, NodeLabel, ReuseKey, SectorImage,
    StepAttributes,
};
use crate::verdict_store::{
    BlockStartStateIndex, EncodedVerdictVector, EnumerationPlanHash, InputFingerprint,
    JudgeVersion, StateOffsetInBlock, StreamOrNodeName, VerdictBlock, VerdictBlockKey,
    VerdictStore, VerdictStoreError,
};

/// 一块最多几个状态：一块是一个 GPU 批、一次原子写、对账与分工的一个单位（里程碑二「增补 4」第十一项「一块取 2¹⁶ 个状态」）。
pub const STATES_PER_BLOCK: u64 = 1 << 16;

/// 展开上限的环境变量：一段的写数不超过它就展开全部真子集，超过就不展开。全量脚本给全量的深度，没设由用例给默认。
pub const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO";

/// 展开上限：一段的写数不超过它，这一段展开全部真子集（[`Layer0SegmentExpansion::EveryProperSubset`]）；超过就不展开。
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
    pub fn expansion_of(self, segment: &[usize]) -> Layer0SegmentExpansion {
        if segment.len() <= self.0 {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
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
/// D8（核心索引结构） 已定项 16 第 4 条；文件从不消失的流是空的），两个判器都按 [`Layer0VersionTable`] 认（第 33 批 R 报告「停下交主 agent」第一项）。
/// 同一个根两边都有是用例写错了：判的时候 [`Layer0VersionTable::new`] 停下。
pub struct CrashFlow<'flow> {
    pub base: &'flow MemoryPool,
    pub writes: &'flow [RetainedWrite],
    pub segments: &'flow [Vec<usize>],
    pub judged_root_index: usize,
    pub versions: &'flow [PublishedVersion],
    pub versions_without_a_file: &'flow [PublishedVersionWithoutAFile],
    pub expansion: &'flow (dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
    pub crash_points: Vec<CrashPointSpan>,
}

impl CrashFlow<'_> {
    /// 判器读的版本表（[`Layer0VersionTable::new`]）。
    #[must_use]
    pub fn version_table(&self) -> Layer0VersionTable<'_> {
        Layer0VersionTable::new(self.versions, self.versions_without_a_file)
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
    pub plan_hash: [u8; DIGEST_BYTES],
}

impl PlannedCrashPoint {
    /// 这个点的块：点内序号区间（从 0 起），每块不超过 [`STATES_PER_BLOCK`] 个状态。
    #[must_use]
    pub fn blocks(&self) -> Vec<Range<u64>> {
        let count = self.ordinals.end - self.ordinals.start;
        (0..count.div_ceil(STATES_PER_BLOCK))
            .map(|index| index * STATES_PER_BLOCK..((index + 1) * STATES_PER_BLOCK).min(count))
            .collect()
    }

    fn block_key(&self, local: &Range<u64>) -> VerdictBlockKey {
        VerdictBlockKey {
            input_fingerprint: InputFingerprint(*self.reuse_key.as_bytes()),
            stream_or_node_name: StreamOrNodeName::new(&self.identity)
                .expect("节点名不超过 u16::MAX 字节"),
            enumeration_plan_hash: EnumerationPlanHash(self.plan_hash),
            block_start: BlockStartStateIndex(local.start),
        }
    }
}

/// 一条流程的整份计划：各点，加上每段归哪个点（ignore 的点的段不展开，判段怎么展开时要它）。
pub struct CrashPlan {
    pub points: Vec<PlannedCrashPoint>,
    owner_of_segment: Vec<usize>,
}

impl CrashPlan {
    /// 一段怎么展开：归 ignore 的点就不展开，其余照流程自己的展开方式。计划与核对都经它，两处不会不一致。
    #[must_use]
    pub fn segment_expansion(
        &self,
        flow: &CrashFlow<'_>,
        segment_index: usize,
        segment: &[usize],
    ) -> Layer0SegmentExpansion {
        if self.points[self.owner_of_segment[segment_index]].ignored {
            Layer0SegmentExpansion::NotExpanded
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
/// 之前的镜像；状态序号按点的次序首尾相接地划分整条流，最后那个全持久状态归最后一个点；ignore 的点的段不展开。
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
            Layer0SegmentExpansion::NotExpanded
        } else {
            (flow.expansion)(segment_index, segment)
        }
    };
    let (ranges_by_segment, state_count) =
        layer0_state_ranges_by_segment(flow.base, flow.writes, flow.segments, &expansion);
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
        for (segment_in_point, segment) in owned.iter().enumerate() {
            let range = &ranges_by_segment[*segment];
            assert_eq!(
                range.start, next_ordinal,
                "段 {segment} 的状态序号没有接上前一段"
            );
            next_ordinal = range.end;
            // 计划哈希只写点内的段序，不写流里的段序：同一个前缀节点在几条流里都是同一份计划
            plan_text.push_str(&format!(
                "segment={segment_in_point} states={} expansion={}\n",
                range.end - range.start,
                expansion_names[*segment]
            ));
        }
        if point_index + 1 == flow.crash_points.len() {
            next_ordinal = state_count;
        }
        let ordinals = start_ordinal..next_ordinal;
        plan_text.push_str(&format!("states={}\n", ordinals.end - ordinals.start));
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
        });
    }
    assert_eq!(
        next_ordinal, state_count,
        "各点的序号区间加起来要正好是整条流的状态数"
    );
    CrashPlan {
        points: planned,
        owner_of_segment,
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
/// - 被判的根槽写（`judged_root_index`）：判器拿它记「根槽已持久」，不进逐项判定；照样进键：它在这个点之前（每个状态都持久）、
///   在点里（第几次写）、还是在之后（都不持久）。
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

/// 基镜像里（txg，实例）最大的那条根：写过的扇区里以根 magic 起头的（根槽写是一个物理块，层 0 的池按 512 字节的扇区建），
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
        let judged_root_slot_write = if flow.judged_root_index < first_write {
            "persisted_in_every_state".to_string()
        } else if flow.judged_root_index < last_write {
            format!(
                "inside_the_point_at_write_{}",
                flow.judged_root_index - first_write
            )
        } else {
            "never_persisted".to_string()
        };
        text.push_str(&format!(
            "judged_root_slot_write={judged_root_slot_write}\n"
        ));
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
        toolchain: &crate::layer0_progress::Layer0ToolchainIdentity,
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
/// `research/scripts/layer0-shard-configuration-check.sh`）组出四种：单 CPU、双机 CPU、单机 CPU + GPU、双机 CPU + 多 GPU。
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
    cards_by_priority_from_configuration_or_environment, configuration_value,
    configured_cards_not_usable, parse_cards_by_priority, CardByPriority, CardHost,
};

/// 显卡配置里登记哪几类计算走 GPU 的键（上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用上现有的硬件加速」；
/// 模板是仓根的 `gpu-cards.env.example`）。环境变量同名，设了就照它。
pub const GPU_COMPUTATION_CLASSES_KEY: &str = "GPU_COMPUTATION_CLASSES";

/// 能登记走 GPU 的计算类，名字就是各自的模块名。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpuComputationClass {
    /// `gpu_unit_checks`：第 ① 段由 CPU 判时，每个状态被写过的单元两道校验和另交一张卡算、与 CPU 逐个对拍（`UnitChecksumsOn::Gpu`）。
    /// 第 ① 段整个在卡上判（登记了 `crash_judge_gpu`）时校验和在判器的内核里算，这一类没有自己的路径。
    GpuUnitChecks,
    /// `crash_verify_gpu`：第 ② 段读事实表的核对在卡上（`FactsVerifierOn::Gpu`）。
    CrashVerifyGpu,
    /// `crash_judge_gpu`：第 ① 段整个在卡上判（`judge_my_blocks_on_gpu_cards`）。
    CrashJudgeGpu,
}

impl GpuComputationClass {
    pub const EVERY_CLASS: [Self; 3] = [
        Self::GpuUnitChecks,
        Self::CrashVerifyGpu,
        Self::CrashJudgeGpu,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::GpuUnitChecks => "gpu_unit_checks",
            Self::CrashVerifyGpu => "crash_verify_gpu",
            Self::CrashJudgeGpu => "crash_judge_gpu",
        }
    }
}

/// 读 `GPU_COMPUTATION_CLASSES` 的值：空白隔开的类名（[`GpuComputationClass::name`]），重复的算一个。
///
/// # Errors
/// 有认不得的词：写明是哪个、认得的有哪几个。
pub fn parse_gpu_computation_classes(text: &str) -> Result<BTreeSet<GpuComputationClass>, String> {
    let mut classes = BTreeSet::new();
    for word in text.split_whitespace() {
        let class = GpuComputationClass::EVERY_CLASS
            .into_iter()
            .find(|class| class.name() == word)
            .ok_or_else(|| {
                format!(
                    "{GPU_COMPUTATION_CLASSES_KEY} 里的「{word}」认不得：认得的类名是 {}",
                    GpuComputationClass::EVERY_CLASS
                        .map(GpuComputationClass::name)
                        .join("、")
                )
            })?;
        classes.insert(class);
    }
    Ok(classes)
}

/// 登记了哪几类走 GPU：`environment_value`（环境变量 `GPU_COMPUTATION_CLASSES`，驱动脚本随命令送，第二台也靠它）给了就照它，
/// 没给读显卡配置文件里的同名键；都没有是 `None`。
///
/// # Errors
/// 写了而有认不得的类名（[`parse_gpu_computation_classes`]）。
pub fn gpu_computation_classes_from(
    environment_value: Option<&str>,
    configuration_file: Option<&Path>,
) -> Result<Option<BTreeSet<GpuComputationClass>>, String> {
    let text = match environment_value {
        Some(text) => Some(text.to_string()),
        None => configuration_file
            .and_then(|path| configuration_value(path, GPU_COMPUTATION_CLASSES_KEY)),
    };
    text.map(|text| parse_gpu_computation_classes(&text))
        .transpose()
}

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
    /// 第 ① 段由 CPU 判时，单元两道校验和另交一张卡对拍；第 ① 段在卡上判时不看它。
    pub unit_checksums: ComputedOn,
}

impl ComputationRoutes {
    /// 有哪一类要卡。
    #[must_use]
    pub fn need_a_card(&self) -> bool {
        [self.crash_judge, self.facts_verifier, self.unit_checksums].contains(&ComputedOn::Gpu)
    }
}

/// 照配置定每一类计算在哪算：没开 GPU（`ENABLE_GPU=0`）时登记不读，三类都在 CPU；开了 GPU，登记了的类只走 GPU、没登记的只走 CPU。
///
/// # Errors
/// 开了 GPU 而一类都没登记（没写这个键、或写的是空的：不默认全开，也不悄悄全走 CPU）；开了 GPU 而这份二进制没编进 `gpu` 特性
/// （登记了的类上不了卡，不悄悄退回 CPU）。
pub fn computation_routes(
    gpu_is_switched_on: bool,
    registered: Option<&BTreeSet<GpuComputationClass>>,
    gpu_feature_is_compiled_in: bool,
) -> Result<ComputationRoutes, String> {
    if !gpu_is_switched_on {
        return Ok(ComputationRoutes {
            crash_judge: ComputedOn::Cpu,
            facts_verifier: ComputedOn::Cpu,
            unit_checksums: ComputedOn::Cpu,
        });
    }
    let Some(registered) = registered.filter(|registered| !registered.is_empty()) else {
        return Err(format!(
            "配置写了 ENABLE_GPU=1，却没登记哪几类计算走 GPU：在显卡配置（SINGLEFS_GPU_CARDS_CONFIG 指的那一份，模板 gpu-cards.env.example）\
             或环境变量里写 {GPU_COMPUTATION_CLASSES_KEY}=<{}…>；不走 GPU 就把 ENABLE_GPU 改成 0",
            GpuComputationClass::EVERY_CLASS.map(GpuComputationClass::name).join(" ")
        ));
    };
    if !gpu_feature_is_compiled_in {
        return Err(format!(
            "配置写了 ENABLE_GPU=1、登记了 {} 走 GPU，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）",
            registered
                .iter()
                .map(|class| class.name())
                .collect::<Vec<_>>()
                .join("、")
        ));
    }
    let route_of = |class: GpuComputationClass| {
        if registered.contains(&class) {
            ComputedOn::Gpu
        } else {
            ComputedOn::Cpu
        }
    };
    Ok(ComputationRoutes {
        crash_judge: route_of(GpuComputationClass::CrashJudgeGpu),
        facts_verifier: route_of(GpuComputationClass::CrashVerifyGpu),
        unit_checksums: route_of(GpuComputationClass::GpuUnitChecks),
    })
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
    let mut tally = RecordingTally::default();
    for point in &plan.points {
        store.record_path(&point.path_number, &point.identity)?;
        let mut point_reused = true;
        for local in point.blocks() {
            let key = point.block_key(&local);
            let count = local.end - local.start;
            store.record_block(&key, count)?;
            if store.read_verdict_block(&key, version)?.is_some() {
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

/// 一个状态存进库的判定向量：`red_items` 逐项、`\n` 接起来的字节；绿的状态是空向量。
fn encoded_verdict(items: &[String]) -> EncodedVerdictVector {
    EncodedVerdictVector(items.join("\n").into_bytes())
}

/// 单元两道校验和在哪算：只在 CPU 上（照旧由池级 checker 判），或另交一张卡算一遍、与 CPU 逐个对拍（配置 `ENABLE_GPU=1`）。
pub enum UnitChecksumsOn {
    CpuOnly,
    #[cfg(feature = "gpu")]
    Gpu(std::sync::Arc<crate::gpu_unit_checks::GpuCard>),
}

/// 核对一趟的结果。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckingTally {
    pub blocks_checked: u64,
    pub states_checked: u64,
    pub red_states: u64,
    /// GPU 与 CPU 在单元校验和上对不上的状态数：不为 0 整趟不作数。
    pub gpu_cpu_disagreements: u64,
    /// 每个红状态的违例正文（点名、块内序号、正文），给门禁与人看；只收前 [`RED_TEXTS_KEPT`] 条。
    pub red_texts: Vec<(String, u64, String)>,
    /// 单元校验和交卡对拍时卡上 wgpu 报了 Out of Memory（第 10z 批）：那一块不入库、之后的块不领，这一趟报本次未跑；
    /// 是「本次未跑」那一行的原因（以「显存不够：」起头）。没报是 `None`。
    pub stopped_because_the_gpu_memory_was_short: Option<String>,
}

pub const RED_TEXTS_KEPT: usize = 64;

/// 核对线程数的环境变量。
pub const CHECKING_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_THREADS";

/// 核对线程数：环境变量 [`CHECKING_THREADS_ENVIRONMENT_VARIABLE`]，没设取本机可用并行度（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）。
///
/// # Panics
/// 同 [`checking_threads_from_the_variable_value`]；值不是 UTF-8 也 panic。
#[must_use]
pub fn checking_threads() -> usize {
    checking_threads_from_the_variable_value(
        process_environment_variable(CHECKING_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

/// 核对线程数变量的值（`None` 是没设）换成线程数：没设取本机可用并行度（取不到是 1）；解析经数值环境变量共用的那一份，
/// 线程数取 `NonZeroUsize`，与仓里别的线程数变量同一口径（C606：写错、写 0 都停下，不悄悄退回本机并行度）。
///
/// # Panics
/// 设了而不是正整数（含 0、空串）：消息带变量名与原值。
#[must_use]
pub fn checking_threads_from_the_variable_value(value: Option<&str>) -> usize {
    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
        CHECKING_THREADS_ENVIRONMENT_VARIABLE,
        value,
    )
    .unwrap_or_else(|| std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN))
    .get()
}

/// 一块判完的结果（还没写库）：每个状态判红的各项、红状态的正文、GPU 与 CPU 对不上的状态数；单元校验和交卡对拍时卡上报了
/// Out of Memory 的，`gpu_memory_was_short` 是「本次未跑」那一行的原因，前几样都是空的（这一段不作数）。
struct JudgedBlock {
    local: Range<u64>,
    items_of_each_state: Vec<Vec<String>>,
    red_texts: Vec<(u64, String)>,
    gpu_cpu_disagreements: u64,
    gpu_memory_was_short: Option<String>,
}

/// 判一块的一段（[`judge_one_block`]）：单元校验和交卡对拍的，这一段包进接 Out of Memory 的那一层（[`on_the_card_catching_out_of_memory`]，
/// 每条核对线程各认各的），报了就交一段不作数、带着原因的（第 10z 批，第 10y 批代码轮判决 K1-3）；`ending` 是这一段在同一块几段里的报到，
/// 认是不是显存不够之前等同一块的几段都收完（第 10z 批第三块，第 10z 批代码轮判决 K1-3 攻方：一段因设备丢了先失败，同块另一段范围里接到的
/// Out of Memory 还没记进设备）。只在 CPU 上判的不用它（丢掉就是报到）。
fn judge_one_piece_catching_out_of_memory_on_the_card(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    point_index: usize,
    local: Range<u64>,
    unit_checksums: &UnitChecksumsOn,
    ending: WorkEndingTogether<'_>,
) -> JudgedBlock {
    match unit_checksums {
        UnitChecksumsOn::CpuOnly => {
            drop(ending);
            judge_one_block(flow, plan, point_index, local, unit_checksums)
        }
        #[cfg(feature = "gpu")]
        UnitChecksumsOn::Gpu(card) => {
            match on_the_card_catching_out_of_memory_ending_together(
                card,
                "单元校验和对拍时",
                ending,
                || judge_one_block(flow, plan, point_index, local.clone(), unit_checksums),
            ) {
                Ok(judged) => judged,
                Err(reason) => JudgedBlock {
                    local,
                    items_of_each_state: Vec::new(),
                    red_texts: Vec::new(),
                    gpu_cpu_disagreements: 0,
                    gpu_memory_was_short: Some(reason),
                },
            }
        }
    }
}

fn judge_one_block(
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    point_index: usize,
    local: Range<u64>,
    unit_checksums: &UnitChecksumsOn,
) -> JudgedBlock {
    let point = &plan.points[point_index];
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let mut items_of_each_state =
        Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
    let mut red_texts = Vec::new();
    let mut gpu_cpu_disagreements = 0u64;
    let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
    let stream_state_count = judge_layer0_state_range_against_the_version_table(
        flow.base,
        flow.writes,
        flow.segments,
        flow.judged_root_index,
        flow.version_table(),
        &expansion,
        ordinals,
        &mut |ordinal, image, judgement| {
            let unit_checksum_items: Vec<String> = match unit_checksums {
                UnitChecksumsOn::CpuOnly => Vec::new(),
                #[cfg(feature = "gpu")]
                UnitChecksumsOn::Gpu(card) => gpu_unit_checksum_items(card, image),
            };
            let items: Vec<String> = judgement
                .red_items()
                .into_iter()
                .chain(unit_checksum_items)
                .collect();
            if items
                .iter()
                .any(|item| item.starts_with("gpu_cpu_disagreement"))
            {
                gpu_cpu_disagreements += 1;
            }
            if !items.is_empty() && red_texts.len() < RED_TEXTS_KEPT {
                red_texts.push((
                    ordinal - point.ordinals.start,
                    judgement.violation_text(image),
                ));
            }
            items_of_each_state.push(items);
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
        red_texts,
        gpu_cpu_disagreements,
        gpu_memory_was_short: None,
    }
}

/// 核对录入表里还没有判定的块：按分工只核 `owner` 那一方分到的；一块切成几段多线程判（[`checking_threads`]），
/// 每块在 CPU 上逐状态判（`unit_checksums` 给了卡就把每个状态镜像里被写过的单元两道校验和交 GPU 算、与 CPU 逐个对拍），
/// 判完一块就按块的次序单线程写回判定块与违例——判定向量的编号按块序登记，与线程数无关，输出与单线程逐字相同。
/// 同一块两方不会同时领：块的归属由权重表与块序号定死。单元校验和对拍时卡上报了 Out of Memory（每条核对线程各自接），那一块不写、
/// 不再领下一块，交回的账带着原因（[`CheckingTally::stopped_because_the_gpu_memory_was_short`]，调用方报本次未跑）。
///
/// # Errors
/// 库读写失败；判定向量超过 256 种。
///
/// # Panics
/// 某个核对线程 panic（判状态的代码自己的断言）。
pub fn check_recorded_blocks(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    shares: &WeightedShares,
    owner: usize,
    unit_checksums: &UnitChecksumsOn,
    version: &JudgeVersion,
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where(
        store,
        flow,
        plan,
        &|block_sequence| shares.owner_of_block(block_sequence) == owner,
        unit_checksums,
        version,
    )
}

/// 核对 `is_mine` 判为自己的那些块（块序号 = 全部点的块按次序编号）；多线程与写库的做法见 [`check_recorded_blocks`]。
///
/// # Errors
/// 库读写失败；判定向量超过 256 种。
pub fn check_blocks_where(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    unit_checksums: &UnitChecksumsOn,
    version: &JudgeVersion,
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where_until(store, flow, plan, is_mine, unit_checksums, version, &|| {
        false
    })
}

/// 停下的请求：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE` 指的文件在，就不再领新的块（手里那一块判完、写进库），这一趟提早收尾。
/// 给「本机先判着、等别的机器准备好再重新分工」用：判过的块在库里，下一趟接着判。
#[must_use]
pub fn stop_is_requested_by_the_environment() -> bool {
    std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE")
        .is_some_and(|path| Path::new(&path).exists())
}

/// 同 [`check_blocks_where`]，每领一块之前先问 `stop_is_requested`：要停就不再领，已经判完的块都写进了库。
///
/// # Errors
/// 同 [`check_blocks_where`]。
pub fn check_blocks_where_until(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &(dyn Fn(u64) -> bool + Sync),
    unit_checksums: &UnitChecksumsOn,
    version: &JudgeVersion,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<CheckingTally, VerdictStoreError> {
    let pending = pending_blocks(store, plan, version, is_mine)?;
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
        checking_threads(),
        stop_is_requested,
        &|point_index, local, ending| {
            judge_one_piece_catching_out_of_memory_on_the_card(
                flow,
                plan,
                point_index,
                local,
                unit_checksums,
                ending,
            )
        },
        &mut |point_index, local, items_of_each_state, violating| {
            let mut numbers = Vec::with_capacity(items_of_each_state.len());
            for items in items_of_each_state {
                numbers.push(store.register_verdict_vector(&encoded_verdict(items))?);
            }
            let verdict_block = VerdictBlock::new(numbers).expect("一块至少一个状态");
            store.store_verdict_block(
                &plan.points[point_index].block_key(local),
                version,
                &verdict_block,
                violating,
            )?;
            Ok(())
        },
    )
}

/// [`check_pending_blocks_in_pieces`] 写一块的那一份：点在 `points` 里的次序、块内序号、逐状态红项、红状态的块内序号。
type StoreACheckedBlock<'store> = dyn FnMut(
        usize,
        &Range<u64>,
        &[Vec<String>],
        &[StateOffsetInBlock],
    ) -> Result<(), VerdictStoreError>
    + 'store;

/// 一个崩溃点的名字与路径编号（核对时打进度行、记红状态的正文用）。
struct NamesOfACrashPoint {
    name: String,
    path_number: String,
}

/// [`check_blocks_where_until`] 的那一圈（不碰库与卡：判一段交 `judge_a_piece`，写一块交 `store_a_block`，单测用假的）：`pending` 里的块按次序，
/// 每领一块之前先问 `stop_is_requested`；一块的序号切成 `threads` 那么多段、一段一个线程交 `judge_a_piece`（带这一段在同一块几段里的报到），
/// 收齐按段序拼回一块、当场交 `store_a_block`（点在 `points` 里的次序、块内序号、逐状态红项、红状态的块内序号）。有一段报了显存不够，
/// 这一块不写、不再领下一块，交回的账带着原因。
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
    judge_a_piece: &(dyn Fn(usize, Range<u64>, WorkEndingTogether<'_>) -> JudgedBlock + Sync),
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
        // 每一段的报到在起线程之前都领出来：先收完的那一段等的是这一块全部的段
        let ending_together = WorksEndingTogetherOnADevice::new();
        let endings: Vec<WorkEndingTogether<'_>> =
            pieces.iter().map(|_| ending_together.join()).collect();
        let mut judged: Vec<JudgedBlock> = std::thread::scope(|scope| {
            let (judged_sender, judged_receiver) = std::sync::mpsc::channel::<JudgedBlock>();
            for (range, ending) in pieces.iter().cloned().zip(endings) {
                let judged_sender = judged_sender.clone();
                scope.spawn(move || {
                    let piece = judge_a_piece(*point_index, range, ending);
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
        // 单元校验和对拍时有一段报了 Out of Memory：这一块不入库、不再领下一块，判过的块在库里（文件头第 5 条）
        if let Some(reason) = reason_the_gpu_memory_was_short_in_one_of_the_pieces(&judged) {
            tally.stopped_because_the_gpu_memory_was_short = Some(reason);
            break;
        }
        let mut items_of_each_state: Vec<Vec<String>> =
            Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
        let mut next_expected = local.start;
        for piece in judged {
            assert_eq!(
                piece.local.start, next_expected,
                "块 {pending_index} 的段没有接上"
            );
            next_expected = piece.local.end;
            items_of_each_state.extend(piece.items_of_each_state);
            tally.gpu_cpu_disagreements += piece.gpu_cpu_disagreements;
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
        store_a_block(*point_index, local, &items_of_each_state, &violating)?;
        tally.blocks_checked += 1;
        tally.states_checked += local.end - local.start;
    }
    Ok(tally)
}

/// 一块按段序排好的几段里，单元校验和对拍时卡上报了 Out of Memory 的第一段的原因；都没报是 `None`。
fn reason_the_gpu_memory_was_short_in_one_of_the_pieces(pieces: &[JudgedBlock]) -> Option<String> {
    pieces
        .iter()
        .find_map(|piece| piece.gpu_memory_was_short.clone())
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

/// `crate::crash_verify_gpu::GpuFactsVerifier` 建的几张缓冲里每一项占几个 u32：照 `crash_verify_gpu.rs` 的 `RECORDED_WORDS`、
/// 每对重放两个字、`SEGMENT_WORDS`、`UNIT_WORDS`、`ROOT_WORDS`、`HEADER_WORDS` 抄（那几个常数那边是私有的，这一批碰不到那份文件），
/// 那边改了布局这里跟着改。
#[cfg(feature = "gpu")]
const FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE: u64 = 4;
#[cfg(feature = "gpu")]
const FACTS_VERIFIER_WORDS_PER_REPLAY: u64 = 2;
#[cfg(feature = "gpu")]
const FACTS_VERIFIER_WORDS_PER_SEGMENT: u64 = 6;
#[cfg(feature = "gpu")]
const FACTS_VERIFIER_WORDS_PER_UNIT: u64 = 6;
#[cfg(feature = "gpu")]
const FACTS_VERIFIER_WORDS_PER_ROOT: u64 = 2 + 4 * (1 + 2 * 4);
#[cfg(feature = "gpu")]
const FACTS_VERIFIER_HEADER_WORDS: u64 = 12;
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
    let writes_of_the_segments: u64 = facts
        .segments
        .iter()
        .map(|segment| count(segment.writes.len()))
        .sum();
    let words_of_each_table = [
        count(facts.is_tearable.len()) * FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE,
        replays * FACTS_VERIFIER_WORDS_PER_REPLAY,
        count(facts.segments.len()) * FACTS_VERIFIER_WORDS_PER_SEGMENT,
        writes_of_the_segments,
        count(facts.units.len()) * FACTS_VERIFIER_WORDS_PER_UNIT,
        count(facts.roots.len()) * FACTS_VERIFIER_WORDS_PER_ROOT,
    ];
    GpuBufferFootprint {
        bytes_uploaded_once: words_of_each_table
            .iter()
            .map(|words| (*words).max(1) * BYTES_PER_GPU_WORD)
            .sum(),
        bytes_of_one_pass: (FACTS_VERIFIER_HEADER_WORDS + 2 * states_in_the_largest_block)
            * BYTES_PER_GPU_WORD,
        passes_alive_at_once: PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
    }
}

/// 第 ① 段由 CPU 判、单元两道校验和交卡对拍（[`gpu_unit_checksum_items`]）建的缓冲（文件头第 5 条）：一个状态一趟，
/// 这条流里每次单元写各两段（`crate::gpu_unit_checks::bytes_crc32c_of_ranges_needs`），`checking_threads` 条线程同时派活。
#[cfg(feature = "gpu")]
fn unit_checksum_buffer_footprint(
    unit_writes_in_the_flow: usize,
    checking_threads: usize,
) -> GpuBufferFootprint {
    GpuBufferFootprint {
        bytes_uploaded_once: 0,
        bytes_of_one_pass: crate::gpu_unit_checks::bytes_crc32c_of_ranges_needs(
            2 * unit_writes_in_the_flow,
        ),
        passes_alive_at_once: u64::try_from(checking_threads).expect("线程数装得进 u64")
            * PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
    }
}

/// 一个崩溃镜像里被写表写过的每个单元（数据单元、节点、打包单元）两道校验和在 GPU 上算、与 CPU 对拍，
/// 交回判红的项（`gpu_unit:<盘>:<偏移>:<结论>`）。对不上的记 `gpu_cpu_disagreement:…`，整趟不作数。
#[cfg(feature = "gpu")]
fn gpu_unit_checksum_items(
    card: &crate::gpu_unit_checks::GpuCard,
    image: &singlefs_harness::memory_pool::CrashImage<'_>,
) -> Vec<String> {
    use crate::gpu_unit_checks::{check_unit_checksums_on_gpu, UnitChecksumVerdict};
    use singlefs_core::recovery::PoolReader;
    let mut positions = Vec::new();
    let mut units: Vec<Vec<u8>> = Vec::new();
    for (write, persisted) in image.writes.iter().zip(&image.persisted) {
        if !persisted || write.kind != StepKind::UnitWrite {
            continue;
        }
        let length = usize::try_from(write.length_in_bytes()).expect("单元长度装得进 usize");
        if let Some(bytes) = PoolReader::read(image, write.device, write.offset, length) {
            positions.push((write.device.0, write.offset.0));
            units.push(bytes);
        }
    }
    let borrowed: Vec<&[u8]> = units.iter().map(Vec::as_slice).collect();
    check_unit_checksums_on_gpu(card, &borrowed)
        .into_iter()
        .zip(positions)
        .filter_map(|(verdict, (device, offset))| match verdict {
            UnitChecksumVerdict::BothHold | UnitChecksumVerdict::NotAUnitThisCheckReads => None,
            UnitChecksumVerdict::HeaderChecksumMismatch => Some(format!(
                "gpu_unit:{device}:{offset}:header_checksum_mismatch"
            )),
            UnitChecksumVerdict::PayloadCrcMismatch => {
                Some(format!("gpu_unit:{device}:{offset}:payload_crc_mismatch"))
            }
            UnitChecksumVerdict::GpuCpuDisagreement => {
                Some(format!("gpu_cpu_disagreement:{device}:{offset}"))
            }
        })
        .collect()
}

// ============================== 第 ① 段的 GPU 形态：GPU 判器判块，卡按优先级领活 ==============================

/// CPU 两遍恢复换成与 GPU 结论第 2、3、4、6 字同口径的四个数：看 journal 那一遍落到的实例代号与 txg 低 32 位
/// （没落到根时 GPU 写 `u32::MAX`、`u32::MAX`），两遍的结局类别（0 没有文件、1 读到文件、2 失败）。
/// 红项相同而落到的根不同也是判错（判器读错了一个落点的版本、两个版本都判绿时就只露在这里），对拍要连它一起比。
#[must_use]
pub fn recovery_words_of(judgement: &crate::crash::Layer0StateJudgement) -> [u32; 4] {
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

/// GPU 一个状态的结论字里与 [`recovery_words_of`] 同口径的四个字（第 2、3、4、6 字）。
#[cfg(feature = "gpu")]
#[must_use]
pub fn gpu_recovery_words(words: &[u32; crate::crash_judge_gpu::VERDICT_WORDS]) -> [u32; 4] {
    [words[2], words[3], words[4], words[6]]
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

/// 一个状态的 GPU 结论字换成与 CPU `Layer0StateJudgement::red_items` 逐项同名、同序的红项；判不了的状态交回 `None`（回落到 CPU）。
#[cfg(feature = "gpu")]
#[must_use]
pub fn gpu_verdict_items(
    words: &[u32; crate::crash_judge_gpu::VERDICT_WORDS],
) -> Option<Vec<String>> {
    use crate::crash::Layer0OracleViolationKind;
    use crate::crash_judge_gpu::{verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD};
    let bits = words[0];
    if bits & (1 << verdict_bits::UNDECIDABLE) != 0 || words[POOL_CHECKER_UNDECIDABLE_WORD] != 0 {
        return None;
    }
    let mut items = Vec::new();
    for (index, kind) in Layer0OracleViolationKind::EVERY_KIND.iter().enumerate() {
        let shift = verdict_bits::CONSULTED_ORACLE_FIRST + u32::try_from(index).expect("七类");
        if bits & (1 << shift) != 0 {
            items.push(format!(
                "{LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE}:{}",
                kind.name()
            ));
        }
    }
    for (index, kind) in Layer0OracleViolationKind::EVERY_KIND.iter().enumerate() {
        let shift = verdict_bits::IGNORED_ORACLE_FIRST + u32::try_from(index).expect("七类");
        if bits & (1 << shift) != 0 {
            items.push(format!(
                "{LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE}:{}",
                kind.name()
            ));
        }
    }
    let violated = crate::crash_judge_gpu::pool_checker_violated_bits(words);
    for (index, invariant) in singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
    {
        if violated & (1u128 << index) != 0 {
            items.push(format!("{LAYER0_RED_PASS_POOL_CHECKER}:{invariant}"));
        }
    }
    if bits & (1 << verdict_bits::ROOT_WITHOUT_RECORD) != 0 {
        items.push(format!(
            "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
        ));
    }
    if bits & (1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT) != 0 {
        items.push(format!(
            "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
        ));
    }
    Some(items)
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
}

/// 这一趟要判的块（这一判法版本下还没有判定块的、归自己的），按点与块序。
fn pending_blocks(
    store: &VerdictStore,
    plan: &CrashPlan,
    version: &JudgeVersion,
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
            if store
                .read_verdict_block(&point.block_key(&local), version)?
                .is_some()
            {
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
    point: &PlannedCrashPoint,
    local: &Range<u64>,
    version: &JudgeVersion,
    items_of_each_state: &[Vec<String>],
    tally: &mut CheckingTally,
) -> Result<(), VerdictStoreError> {
    let mut numbers = Vec::with_capacity(items_of_each_state.len());
    let mut violating = Vec::new();
    for (offset, items) in items_of_each_state.iter().enumerate() {
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
        numbers.push(store.register_verdict_vector(&encoded_verdict(items))?);
    }
    let verdict_block = VerdictBlock::new(numbers).expect("一块至少一个状态");
    store.store_verdict_block(&point.block_key(local), version, &verdict_block, &violating)?;
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
    tables: &'work crate::crash_judge_dispatch::PackedJudgeTables,
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
            let items_of_each_state: Vec<Vec<String>> = words
                .iter()
                .map(|state_words| {
                    gpu_verdict_items(state_words).expect("判不了的状态上面已经拒过")
                })
                .collect();
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
            BlockJudgedOnACard::Judged(items_of_each_state)
        },
        &mut |pending_index, items_of_each_state| match sender.send(CardMessage::Block(
            GpuJudgedBlock {
                pending_index,
                items_of_each_state,
            },
        )) {
            Ok(()) => BlockHandedToTheStore::Handed,
            Err(_) => BlockHandedToTheStore::TheStoreIsGone,
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
    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
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
    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
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
    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
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
    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
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
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    version: &JudgeVersion,
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
    match judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(
        store,
        flow,
        plan,
        is_mine,
        version,
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
enum GpuJudgingOfMyBlocks {
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
    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
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
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    version: &JudgeVersion,
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    cards_taken_from: GpuCardsSource,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<GpuJudgingOfMyBlocks, String> {
    use crate::crash_judge_tables::{build_judge_tables_against_the_version_table, JudgeFlow};
    let pending = pending_blocks(store, plan, version, is_mine)
        .map_err(|error| format!("读判定块失败：{error:?}"))?;
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let judge_flow = JudgeFlow {
        base: flow.base,
        writes: flow.writes,
        segments: flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: flow.versions,
        expansion: &expansion,
    };
    let tables = match build_judge_tables_against_the_version_table(
        &judge_flow,
        flow.versions_without_a_file,
    ) {
        Ok(tables) => tables,
        Err(refusal) => {
            return Err(format!(
                "GPU 判器建不出这条流的表（{refusal:?}）。配置开了 GPU，不退回 CPU 判器：让判器的表认得这种写再跑"
            ));
        }
    };
    let mut tally = CheckingTally::default();
    let mut outcome = GpuJudgeOutcome::default();
    if pending.is_empty() {
        return Ok(GpuJudgingOfMyBlocks::Judged { tally, outcome });
    }
    let state_count = tables.state_count;
    let tables = crate::crash_judge_dispatch::PackedJudgeTables::of(&tables);
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
            card_capacities_of_this_run(cards_taken_from, &cards_of_the_round, &tables)?
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
            crate::crash_judge_dispatch::mebibytes_a_judge_needs(&tables, 1),
        ) {
            ScratchStepBeforeBooking::BookOnTheCards => {
                match scratch_of_each_card_booked_by_where_the_cards_came_from(
                    cards_taken_from,
                    &cards_of_the_round,
                    &run_cards,
                    &tables,
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
            tables: &tables,
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
                    let stored = store_judged_block(
                        store,
                        &plan.points[*point_index],
                        local,
                        version,
                        &block.items_of_each_state,
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
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    on: &FactsVerifierOn,
) -> Result<VerificationTally, String> {
    let facts = extract_flow_facts(flow.base, flow.writes, flow.segments, flow.expansion);
    verify_blocks_from_these_facts(store, plan, &facts, is_mine, on)
}

/// 同 [`verify_blocks_from_facts`]，事实表由调用方抽好（起过卡池的进程按事实表的大小借第 ② 段的缓冲，抽一次、借、再核）。
///
/// # Errors
/// 同 [`verify_blocks_from_facts`]。
fn verify_blocks_from_these_facts(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    facts: &FlowFacts,
    is_mine: &dyn Fn(u64) -> bool,
    on: &FactsVerifierOn,
) -> Result<VerificationTally, String> {
    let facts_digest = facts.digest();
    store
        .store_flow_facts(&facts_digest, &facts.to_bytes())
        .map_err(|error| format!("事实表写不进库：{error:?}"))?;
    let verifier_digest = on.verifier_digest();
    let verifier = BlockVerifier::new(on, facts);
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
            let bits = verifier.verify(facts, ordinals);
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
    fn new(on: &'card FactsVerifierOn, facts: &FlowFacts) -> Self {
        match on {
            FactsVerifierOn::Cpu => {
                let _ = facts;
                Self::Cpu
            }
            #[cfg(feature = "gpu")]
            FactsVerifierOn::Gpu(card) => Self::Gpu(Box::new(
                crate::crash_verify_gpu::GpuFactsVerifier::new(card, facts),
            )),
        }
    }

    fn verify(&self, facts: &FlowFacts, ordinals: Range<u64>) -> Vec<u32> {
        match self {
            Self::Cpu => verify_states_from_facts(facts, ordinals),
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
    let mut comparison = JudgeVerifierComparison::default();
    for point in &plan.points {
        for local in point.blocks() {
            let key = point.block_key(&local);
            let judged = store.read_verdict_block(&key, version)?;
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
            let numbers = judged.verdict_vector_number_of_each_state();
            for (offset, (number, bit)) in numbers.iter().zip(&row.bits).enumerate() {
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
    version: &JudgeVersion,
) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
    let judged: std::collections::HashSet<VerdictBlockKey> = store
        .stored_verdict_block_keys()?
        .into_iter()
        .filter(|(_, stored_version)| stored_version == version)
        .map(|(key, _)| key)
        .collect();
    let mut unjudged = Vec::new();
    for point in &plan.points {
        for local in point.blocks() {
            let key = point.block_key(&local);
            if !judged.contains(&key) {
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
    let mut found = Vec::new();
    for point in &plan.points {
        for local in point.blocks() {
            let key = point.block_key(&local);
            for violation in store.violations_in_block(&key, version)? {
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

    /// 第 10z 批（第 10y 批代码轮判决 K1-3）：一块的几段里，单元校验和对拍时卡上报了 Out of Memory 的，按段序取第一段的原因
    /// （那一块不入库、这一趟报本次未跑）；都没报是 `None`。不要卡：几段照核对线程交回的造。
    #[test]
    fn the_first_piece_short_of_gpu_memory_stops_the_checking_with_its_reason() {
        let piece = |start: u64, gpu_memory_was_short: Option<&str>| JudgedBlock {
            local: start..start + 2,
            items_of_each_state: Vec::new(),
            red_texts: Vec::new(),
            gpu_cpu_disagreements: 0,
            gpu_memory_was_short: gpu_memory_was_short.map(str::to_string),
        };
        assert_eq!(
            reason_the_gpu_memory_was_short_in_one_of_the_pieces(&[
                piece(0, None),
                piece(2, Some("显存不够：第二段")),
                piece(4, Some("显存不够：第三段")),
            ]),
            Some("显存不够：第二段".to_string())
        );
        assert_eq!(
            reason_the_gpu_memory_was_short_in_one_of_the_pieces(&[piece(0, None), piece(2, None)]),
            None
        );
    }

    /// 第 10z 批第三块（第 10z 批代码轮判决 K3 攻方第三节第 1445–1447 行）：一块的几段里有一段报了显存不够，这一块不写、不再领下一块，
    /// 之前判完的块写了，交回的账带着那一段的原因。不要卡、不碰库：判一段与写一块都用假的。
    #[test]
    fn a_block_with_a_piece_short_of_gpu_memory_is_not_stored_and_no_later_block_is_taken() {
        let pending: Vec<(usize, Range<u64>)> = vec![(0, 0..2), (0, 2..4), (0, 4..6), (0, 6..8)];
        let points = vec![NamesOfACrashPoint {
            name: "一个点".to_string(),
            path_number: "0".to_string(),
        }];
        let mut stored_block_starts = Vec::new();
        let tally = check_pending_blocks_in_pieces(
            &pending,
            &points,
            2,
            &|| false,
            &|_point_index, local, ending| {
                drop(ending);
                let is_short = local.start == 5;
                JudgedBlock {
                    items_of_each_state: if is_short {
                        Vec::new()
                    } else {
                        vec![
                            Vec::new();
                            usize::try_from(local.end - local.start).expect("一段装得进")
                        ]
                    },
                    local,
                    red_texts: Vec::new(),
                    gpu_cpu_disagreements: 0,
                    gpu_memory_was_short: is_short
                        .then(|| "显存不够：块 4..6 的第二段".to_string()),
                }
            },
            &mut |_point_index, local, items_of_each_state, _violating| {
                assert_eq!(
                    items_of_each_state.len(),
                    usize::try_from(local.end - local.start).expect("一块装得进"),
                    "写进库的块每个状态都有判定"
                );
                stored_block_starts.push(local.start);
                Ok(())
            },
        )
        .expect("假的写库不失败");
        assert_eq!(
            stored_block_starts,
            vec![0, 2],
            "报了的那一块与之后的都不写"
        );
        assert_eq!(tally.blocks_checked, 2);
        assert_eq!(tally.states_checked, 4);
        assert_eq!(
            tally.stopped_because_the_gpu_memory_was_short,
            Some("显存不够：块 4..6 的第二段".to_string())
        );
    }

    /// 第 10z 批第五块（第 10z 批第三块代码轮正推 K3 补充 2 第 2 条：第 D 条的接线）：一块切成的几段领的是同一组报到，先收完的那一段等到
    /// 同一块别的段都报到了才交回（认它是不是显存不够之前，同块别的段范围里接到的都记进了设备）。不要卡、不碰库：判一段与写一块都用假的；
    /// 段 0 一来就报到、等，别的段等段 0 报到之后过 100 毫秒再报到。每段各领一组时段 0 一报到就交回，那一刻别的段都还没报到。
    #[test]
    fn the_pieces_of_one_block_report_their_endings_in_one_group() {
        let pending: Vec<(usize, Range<u64>)> = vec![(0, 0..3)];
        let points = vec![NamesOfACrashPoint {
            name: "一个点".to_string(),
            path_number: "0".to_string(),
        }];
        let pieces_ended = std::sync::atomic::AtomicUsize::new(0);
        let pieces_ended_when_the_first_piece_returned = std::sync::Mutex::new(None);
        check_pending_blocks_in_pieces(
            &pending,
            &points,
            3,
            &|| false,
            &|_point_index, local, ending| {
                if local.start == 0 {
                    pieces_ended.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    ending.end_and_wait_for_the_others();
                    *pieces_ended_when_the_first_piece_returned
                        .lock()
                        .expect("段 0 只有一条线程写") =
                        Some(pieces_ended.load(std::sync::atomic::Ordering::SeqCst));
                } else {
                    let waiting_since = std::time::Instant::now();
                    while pieces_ended.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                        assert!(
                            waiting_since.elapsed() < std::time::Duration::from_secs(60),
                            "段 0 一分钟都没报到"
                        );
                        std::thread::yield_now();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    pieces_ended.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    ending.end_and_wait_for_the_others();
                }
                JudgedBlock {
                    items_of_each_state: vec![
                        Vec::new();
                        usize::try_from(local.end - local.start)
                            .expect("一段装得进")
                    ],
                    local,
                    red_texts: Vec::new(),
                    gpu_cpu_disagreements: 0,
                    gpu_memory_was_short: None,
                }
            },
            &mut |_point_index, _local, _items_of_each_state, _violating| Ok(()),
        )
        .expect("假的写库不失败");
        assert_eq!(
            pieces_ended_when_the_first_piece_returned
                .into_inner()
                .expect("段 0 的线程不 panic"),
            Some(3),
            "段 0 等到同一块的三段都报到才交回"
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
            4,
            "产品路径的三处借不到都经打印的那一半（加上它自己的定义一处）"
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
    /// 2 段（12 个字）、段里共 3 次写（3 个字），单元与根都没有（空表各建一个字）：上卡一次 33 个字 132 字节；一块 65 536 个状态一趟
    /// 建头 12 个字、结论与回读各 65 536 个字，同时两趟活着：合起来 1 048 804 字节，向上取整 2 MiB。
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
                },
                crate::crash_facts::SegmentFact {
                    writes: vec![2],
                    states: 4..6,
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
                bytes_uploaded_once: 132,
                bytes_of_one_pass: (12 + 2 * 65_536) * 4,
                passes_alive_at_once: 2,
            },
            "六张表上卡一次（空表各一个字），一块一趟建头、结论与回读"
        );
        assert_eq!(footprint.mebibytes(), 2);
    }

    /// 第 10w 批：单元校验和对拍在卡上建的缓冲进账的量（文件头第 5 条）。这条流 10 次单元写、32 条核对线程：一个状态一趟 20 段，
    /// 每段一个 32 KiB 的槽、一个长度字、一条结果与回读，加 16 字节派活参数，共 655 776 字节；每条线程同时两趟活着：64 趟，
    /// 合起来 41 969 664 字节，向上取整 41 MiB。
    #[cfg(feature = "gpu")]
    #[test]
    fn unit_checksums_book_two_ranges_per_unit_write_for_two_passes_on_every_checking_thread() {
        let footprint = unit_checksum_buffer_footprint(10, 32);
        assert_eq!(
            footprint,
            GpuBufferFootprint {
                bytes_uploaded_once: 0,
                bytes_of_one_pass: 20 * (32_768 + 4 + 16) + 16,
                passes_alive_at_once: 64,
            },
            "每次单元写两段、每条线程两趟"
        );
        assert_eq!(footprint.mebibytes(), 41);
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
        assert_eq!(
            cards_by_priority_from_configuration_or_environment(Some(&file))
                .expect("写法对")
                .map(|cards| cards.len()),
            Some(2)
        );
        let without_the_key = directory.join("without-the-key.env");
        std::fs::write(&without_the_key, "ENABLE_GPU=1\n").expect("写得了");
        assert_eq!(
            cards_by_priority_from_configuration_or_environment(Some(&without_the_key)),
            Ok(None)
        );
        let _ = std::fs::remove_dir_all(&directory);
    }

    /// 库落在开机会清空的目录下就拒；写明一次性才放行；家目录这类持久目录放行；目录还没建也按它的上层判。
    /// 环境走 K5 那一个守卫：清掉一趟读的变量、跑完（或 panic）还原，不读调用方留下的「一次性」开关。
    #[test]
    fn a_library_under_a_directory_a_reboot_erases_is_refused_unless_declared_temporary() {
        let mut environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
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
        environment.set(
            "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY",
            std::ffi::OsStr::new("1"),
        );
        assert_eq!(
            refuse_a_library_that_a_reboot_would_erase(Path::new("/tmp/x")),
            Ok(()),
            "写明一次性的放行"
        );
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

    /// 核对线程数写成 0：panic，消息带变量名与原值，不悄悄退回本机并行度（与仓里别的线程数变量同一口径）。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_AMPLIFICATION_THREADS=\"0\" 认不出")]
    fn zero_checking_threads_panics_naming_the_variable_and_the_value() {
        let _ = checking_threads_from_the_variable_value(Some("0"));
    }

    /// 核对线程数写坏（不是数）：panic，消息带变量名与原值，不悄悄退回本机并行度。
    #[test]
    #[should_panic(expected = "SINGLEFS_CRASH_AMPLIFICATION_THREADS=\"eight\" 认不出")]
    fn an_unreadable_checking_threads_value_panics_naming_the_variable_and_the_value() {
        let _ = checking_threads_from_the_variable_value(Some("eight"));
    }

    /// 核对线程数设了照它（两头的空白不算），没设取本机可用并行度。
    #[test]
    fn checking_threads_are_the_value_written_or_the_available_parallelism_when_unset() {
        assert_eq!(checking_threads_from_the_variable_value(Some(" 3\n")), 3);
        assert_eq!(
            checking_threads_from_the_variable_value(None),
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
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
            Layer0SegmentExpansion::EveryProperSubset
        );
        assert_eq!(
            limit.expansion_of(&[0; 29]),
            Layer0SegmentExpansion::NotExpanded
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
                        red_texts: Vec::new(),
                        gpu_cpu_disagreements: 0,
                        gpu_memory_was_short: None,
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
        assert!(
            names_read.len() >= 10,
            "按名字读的环境变量少得不对，扫法坏了：{names_read:?}"
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
    /// （`StopRequestRememberedInThisRun::new`），外层循环、卡线程、一轮收尾与整趟收尾不再各自现读文件。扫这份源码，不要卡。
    #[test]
    fn the_run_reads_the_stop_file_only_into_the_stop_request_it_remembers() {
        let lines_naming_the_stop_file_reader: Vec<String> = code_lines_outside_the_tests()
            .into_iter()
            .filter(|line| line.contains(concat!("stop_is_requested_", "by_the_environment")))
            .map(|line| line.trim().to_string())
            .collect();
        assert_eq!(
            lines_naming_the_stop_file_reader,
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
            "读叫停文件的只有定义它的那一处与这一趟记住叫停的那一处"
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
                concat!(
                    "(ComputedOn::Gpu, Some(cards)) => match judge_my_blocks_on_gpu",
                    "("
                ),
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
            &|_point_index, local, ending| {
                drop(ending);
                JudgedBlock {
                    items_of_each_state: vec![
                        Vec::new();
                        usize::try_from(local.end - local.start)
                            .expect("一段装得进")
                    ],
                    local,
                    red_texts: Vec::new(),
                    gpu_cpu_disagreements: 0,
                    gpu_memory_was_short: None,
                }
            },
            &mut |_point_index, local, _items_of_each_state, _violating| {
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
                    "matchjudge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(",
                    "store,flow,plan,is_mine,version,cards,GpuCardsSource::DevicesStartedForThisRun,stop_is_",
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
                    "unit_checksums,version,stop_is_",
                    "requested,)"
                )),
            ),
            (
                "judge_my_blocks_on_gpu",
                STOP_REQUEST_PARAMETER_NAME,
                StopRequestUseThatIsPinned::WholeBodyAfterTheParameter(concat!(
                    ")->Result<GpuJudgingOfMyBlocks,String>\u{7b}",
                    "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(",
                    "store,flow,plan,is_mine,version,cards,cards_taken_from,stop_is_",
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
                "check_pending_blocks_in_pieces",
                STOP_REQUEST_PARAMETER_NAME,
            ),
            ("judge_my_blocks_on_gpu_cards", STOP_REQUEST_PARAMETER_NAME),
            (
                "judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from",
                STOP_REQUEST_PARAMETER_NAME,
            ),
            ("check_selected_blocks", STOP_REQUEST_PARAMETER_NAME),
            ("judge_my_blocks_on_gpu", STOP_REQUEST_PARAMETER_NAME),
            ("judge_my_blocks_on_gpu", concat!("_stop_is_", "requested")),
        ];
        if layers_handed_the_stop_request != expected_layers {
            found.push(format!(
                "收叫停的是这六层，连同没编进 gpu 特性时那一份 judge_my_blocks_on_gpu（多一层少一层、参数换了名字都要回来看这条用例罩没罩到），读到的是 {layers_handed_the_stop_request:?}"
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

    fn registered(classes: &[GpuComputationClass]) -> BTreeSet<GpuComputationClass> {
        classes.iter().copied().collect()
    }

    fn routes_with_the_gpu_switched_on(classes: &[GpuComputationClass]) -> ComputationRoutes {
        computation_routes(true, Some(&registered(classes)), true)
            .expect("开了 GPU、登记了至少一类、编进了 gpu 特性")
    }

    /// H9：登记了 `crash_judge_gpu`，第 ① 段在卡上判。
    #[test]
    fn the_crash_judge_runs_on_the_gpu_when_crash_judge_gpu_is_registered() {
        assert_eq!(
            routes_with_the_gpu_switched_on(&[GpuComputationClass::CrashJudgeGpu]).crash_judge,
            ComputedOn::Gpu
        );
    }

    /// H9：没登记 `crash_judge_gpu`（登记了别的），第 ① 段在 CPU 上判。
    #[test]
    fn the_crash_judge_runs_on_the_cpu_when_crash_judge_gpu_is_not_registered() {
        assert_eq!(
            routes_with_the_gpu_switched_on(&[
                GpuComputationClass::CrashVerifyGpu,
                GpuComputationClass::GpuUnitChecks
            ])
            .crash_judge,
            ComputedOn::Cpu
        );
    }

    /// H9：登记了 `crash_verify_gpu`，第 ② 段在卡上核。
    #[test]
    fn the_facts_verifier_runs_on_the_gpu_when_crash_verify_gpu_is_registered() {
        assert_eq!(
            routes_with_the_gpu_switched_on(&[GpuComputationClass::CrashVerifyGpu]).facts_verifier,
            ComputedOn::Gpu
        );
    }

    /// H9：没登记 `crash_verify_gpu`（登记了别的），第 ② 段在 CPU 上核。
    #[test]
    fn the_facts_verifier_runs_on_the_cpu_when_crash_verify_gpu_is_not_registered() {
        assert_eq!(
            routes_with_the_gpu_switched_on(&[
                GpuComputationClass::CrashJudgeGpu,
                GpuComputationClass::GpuUnitChecks
            ])
            .facts_verifier,
            ComputedOn::Cpu
        );
    }

    /// H9：登记了 `gpu_unit_checks`，第 ① 段由 CPU 判时单元校验和交卡对拍。
    #[test]
    fn the_unit_checksums_run_on_the_gpu_when_gpu_unit_checks_is_registered() {
        assert_eq!(
            routes_with_the_gpu_switched_on(&[GpuComputationClass::GpuUnitChecks]).unit_checksums,
            ComputedOn::Gpu
        );
    }

    /// H9：没登记 `gpu_unit_checks`（登记了别的），单元校验和只在 CPU 上。
    #[test]
    fn the_unit_checksums_run_on_the_cpu_when_gpu_unit_checks_is_not_registered() {
        assert_eq!(
            routes_with_the_gpu_switched_on(&[
                GpuComputationClass::CrashJudgeGpu,
                GpuComputationClass::CrashVerifyGpu
            ])
            .unit_checksums,
            ComputedOn::Cpu
        );
    }

    /// H9：没开 GPU 时登记不读，三类都在 CPU 上，一张卡都不要。
    #[test]
    fn every_class_runs_on_the_cpu_when_the_gpu_is_switched_off_even_if_registered() {
        let routes = computation_routes(
            false,
            Some(&registered(&GpuComputationClass::EVERY_CLASS)),
            true,
        )
        .expect("没开 GPU 不报错");
        assert_eq!(
            routes,
            ComputationRoutes {
                crash_judge: ComputedOn::Cpu,
                facts_verifier: ComputedOn::Cpu,
                unit_checksums: ComputedOn::Cpu,
            }
        );
        assert!(!routes.need_a_card());
    }

    /// H9：开了 GPU 而一类都没登记（键没写、或写的是空的）：报错，不默认全开、也不悄悄全走 CPU。
    #[test]
    fn the_gpu_switched_on_with_no_class_registered_is_refused() {
        for registration in [None, Some(BTreeSet::new())] {
            let refused = computation_routes(true, registration.as_ref(), true);
            assert!(
                refused
                    .as_ref()
                    .is_err_and(|reason| reason.contains(GPU_COMPUTATION_CLASSES_KEY)),
                "登记 {registration:?}：{refused:?}"
            );
        }
    }

    /// H9：开了 GPU、登记了一类，而这份二进制没编进 `gpu` 特性：报错并点名登记的类，不悄悄退回 CPU。
    #[test]
    fn a_registered_class_without_the_gpu_feature_compiled_in_is_refused() {
        let refused = computation_routes(
            true,
            Some(&registered(&[GpuComputationClass::CrashVerifyGpu])),
            false,
        );
        assert!(
            refused.as_ref().is_err_and(
                |reason| reason.contains("gpu 特性") && reason.contains("crash_verify_gpu")
            ),
            "{refused:?}"
        );
    }

    /// H9：类名照模块名读，空白隔开、重复的算一个。
    #[test]
    fn the_three_class_names_read_back_as_the_three_classes() {
        assert_eq!(
            parse_gpu_computation_classes(
                "gpu_unit_checks  crash_verify_gpu\tcrash_judge_gpu crash_judge_gpu"
            ),
            Ok(registered(&GpuComputationClass::EVERY_CLASS))
        );
    }

    /// H9：认不得的类名报错，点名是哪一个。
    #[test]
    fn an_unknown_class_name_is_refused_naming_it() {
        let refused = parse_gpu_computation_classes("crash_judge_gpu crash_judge_cpu");
        assert!(
            refused
                .as_ref()
                .is_err_and(|reason| reason.contains("「crash_judge_cpu」")),
            "{refused:?}"
        );
    }

    /// H9：环境变量给了就照它，不读显卡配置里的同名键。
    #[test]
    fn the_classes_from_the_environment_come_before_the_configuration_file() {
        let directory = std::env::temp_dir().join(format!(
            "singlefs-gpu-computation-classes-environment-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("建得了临时目录");
        let file = directory.join("gpu-cards.env");
        std::fs::write(&file, "GPU_COMPUTATION_CLASSES='crash_judge_gpu'\n").expect("写得了");
        let classes = gpu_computation_classes_from(Some("crash_verify_gpu"), Some(&file));
        std::fs::remove_dir_all(&directory).expect("删得掉");
        assert_eq!(
            classes,
            Ok(Some(registered(&[GpuComputationClass::CrashVerifyGpu])))
        );
    }

    /// H9：环境变量没给就读显卡配置里的同名键（去一层引号）。
    #[test]
    fn the_classes_are_read_from_the_configuration_file_when_the_environment_gives_none() {
        let directory = std::env::temp_dir().join(format!(
            "singlefs-gpu-computation-classes-file-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("建得了临时目录");
        let file = directory.join("gpu-cards.env");
        std::fs::write(
            &file,
            "GPU_CARDS_BY_PRIORITY='local:0:15G'\nGPU_COMPUTATION_CLASSES='crash_judge_gpu gpu_unit_checks'\n",
        )
        .expect("写得了");
        let classes = gpu_computation_classes_from(None, Some(&file));
        std::fs::remove_dir_all(&directory).expect("删得掉");
        assert_eq!(
            classes,
            Ok(Some(registered(&[
                GpuComputationClass::CrashJudgeGpu,
                GpuComputationClass::GpuUnitChecks
            ])))
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
    /// 这一趟只排了派活计划（`SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1`）：录入与导入照做，一块都没判。
    pub placement_only: bool,
    /// 这一趟被叫停了（[`stop_is_requested_by_the_environment`]），或显存不够（起过卡池的进程在显存账上此刻借不到、卡上 wgpu 报了
    /// Out of Memory）、打了「本次未跑」那一行（文件头第 5 条）：判过的块在库里，没做完的段留给下一趟，对账不为空是正常的；
    /// 分项驱动照它把这一项记成本次未跑。只排派活计划的那一趟借不到试判的草稿区也置上（`placement_only` 同时为真，那一趟没打
    /// `CRASH_AMPLIFICATION_PLACEMENT` 行）。
    pub stopped_early: bool,
    /// 这一趟判定行上的判法版本。
    pub judge_version: JudgeVersion,
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
    /// 起过卡池的进程第 ① 段由 CPU 判、单元校验和交卡对拍时，在显存账上为对拍的缓冲借的（MiB，文件头第 5 条）；没借是 `None`。
    pub unit_checksum_buffers_booked_mebibytes: Option<u64>,
    /// 起过卡池的进程第 ② 段在卡上核时，在显存账上为核对的缓冲借的（MiB，文件头第 5 条）；没借是 `None`。
    pub facts_verifier_buffers_booked_mebibytes: Option<u64>,
    /// 起过卡池的进程单元校验和对拍完那一刻核账：对拍的缓冲那一笔借据还在账上没有（第 10t 批代码轮判决 K4-1）；没借是 `None`。
    pub unit_checksum_buffers_lease_on_the_books_after_checking: Option<bool>,
    /// 起过卡池的进程第 ② 段核完那一刻核账：核对的缓冲那一笔借据还在账上没有（同上）；没借是 `None`。
    pub facts_verifier_buffers_lease_on_the_books_after_verifying: Option<bool>,
}

/// 一趟用的卡从哪来（定义在 harness 档，草稿区记不记账的判法 `singlefs_harness::gpu_memory_ledger::scratch_states_to_book_on_each_card`
/// 照它分，单测在 harness 档随时跑得到）。
pub use singlefs_harness::gpu_memory_ledger::GpuCardsSource;

/// 每一行 `CRASH_AMPLIFICATION ` 末尾都接的几个字段（汇总行、被叫停的行、`flow=` 行、两台跑的分份行与导入行，打法都在
/// `crates/singlefs-checker-tier/tests/` 里调 [`run_from_environment`] 的用例）：
///
/// ` states_reused=<n> exhaustive=<true|false> expand_up_to=<n> ignored_points=<n> states_skipped_by_ignore=<n>`
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
#[must_use]
pub fn reuse_and_scale_fields_of_the_line(run: &EnvironmentRun) -> String {
    format!(
        " states_reused={} exhaustive={} expand_up_to={} ignored_points={} states_skipped_by_ignore={}",
        run.states_reused,
        run.exhaustive,
        run.expand_up_to.0,
        run.ignored_points,
        run.states_skipped_by_ignore
    )
}

/// 不 ignore 时照流自己的展开方式该有的态数，减去这一份计划的态数（ignore 的点的段不展开）。一个点都没 ignore 是 0。
fn states_skipped_by_ignore(flow: &CrashFlow<'_>, plan: &CrashPlan) -> u64 {
    if !plan.points.iter().any(|point| point.ignored) {
        return 0;
    }
    let without_ignore =
        layer0_state_ranges_by_segment(flow.base, flow.writes, flow.segments, flow.expansion).1;
    let planned = plan.points.last().map_or(0, |point| point.ordinals.end);
    without_ignore
        .checked_sub(planned)
        .expect("ignore 只会让段少展开，态数不会比不 ignore 时多")
}

/// 这一份（`is_mine` 选的块）里录入时库中已有这一判法版本判定的态数。
fn states_already_judged_in_this_share(
    store: &VerdictStore,
    plan: &CrashPlan,
    version: &JudgeVersion,
    is_mine: &dyn Fn(u64) -> bool,
) -> Result<u64, VerdictStoreError> {
    let mut states = 0u64;
    let mut block_sequence = 0u64;
    for point in &plan.points {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if is_mine(this_block)
                && store
                    .read_verdict_block(&point.block_key(&local), version)?
                    .is_some()
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
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    selected_block_sequences: &std::collections::HashSet<u64>,
    unit_checksums: &UnitChecksumsOn,
    version: &JudgeVersion,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where_until(
        store,
        flow,
        plan,
        &|block_sequence| selected_block_sequences.contains(&block_sequence),
        unit_checksums,
        version,
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
    "SINGLEFS_GPU_CARDS_CONFIG",
    "GPU_CARDS_BY_PRIORITY",
    GPU_COMPUTATION_CLASSES_KEY,
    "SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE",
    "SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY",
    "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
    "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS",
    "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
    "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY",
    "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
    "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE",
    "SINGLEFS_CRASH_AMPLIFICATION_VERIFY",
    CHECKING_THREADS_ENVIRONMENT_VARIABLE,
    EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
];

/// 同一个测试二进制里改进程环境、读 [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里变量的用例一条接一条跑：几条一起改同一批变量会互相读到对方的值。
/// 这份库的单测与 `tests/` 下各个测试二进制各有一把（一个进程一份）。
static ONE_TEST_CHANGES_THE_ENVIRONMENT_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 一批环境变量：建的时候记下原值，丢掉（用例收尾、panic）时逐个还原——原来没设的删掉，设了的写回原值。
pub struct EnvironmentRestoredOnDrop {
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl EnvironmentRestoredOnDrop {
    /// 记下 `names` 每个变量的原值，再全部删掉。
    #[must_use]
    pub fn saved_then_cleared(names: &[&'static str]) -> Self {
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
/// （没设建在临时目录、跑完删；设了就要落在开机不清空的地方——`/tmp`、`/var/tmp`、`/dev/shm`、`/run` 之下一律拒绝，
/// 只有测试自己知道库是一次性的时才写 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1` 放行），
/// `SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 是别的机器拷回来的库目录（`:` 分隔），核之前先导入；同一条命令再跑一次就是续跑：
/// 这一判法版本下判过的块全部复用，只补没判的；
/// 跑法带 GPU 时卡按配置的优先级用（`SINGLEFS_GPU_CARDS_CONFIG` 指的显卡配置里的 `GPU_CARDS_BY_PRIORITY`，[`parse_cards_by_priority`]；
/// 这个进程在哪台机器看 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`）：任务先装第一张卡，装到它的显存额度为止，装不下的才给下一张；没配就按本机能用的卡的
/// 空闲显存从大到小用（`gpu` 特性没编进来而配置写了 `ENABLE_GPU=1` 就报错，不悄悄退回 CPU）。哪几类计算走 GPU 照
/// `GPU_COMPUTATION_CLASSES`（环境变量优先，退到显卡配置里的同名键；[`computation_routes`]）：登记了的只走 GPU、没登记的只走 CPU。
/// `SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1` 只排派活计划：打一行 `CRASH_AMPLIFICATION_PLACEMENT …`（要判多少、每张卡装多少、
/// 这台机器的卡试判几次派活量到的速度），一块都不判。
/// 主机之间等分（`SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=<w0>,<w1>,…` 可以改成按权重）。
/// `expand_up_to` 是这一趟的展开上限，进交回的结果与 `CRASH_AMPLIFICATION_GPU_TOTAL` 行；流自己的展开方式要与它逐段相同。
///
/// # Errors
/// 环境变量写法错、库打不开、判定存储读写失败、配置要 GPU 而这份二进制没有；流的展开方式与 `expand_up_to` 在某一段上不同
/// （报的展开上限就不是这一趟枚举的那个，在任何录入之前拒）；开了 GPU 而 `GPU_COMPUTATION_CLASSES` 没登记、登记了认不得的类名
/// （在开库之前拒，[`computation_routes`]）；登记了的类这一趟上不了卡；只排派活计划而没登记 `crash_judge_gpu`。
#[allow(
    clippy::too_many_lines,
    reason = "读环境、开库、导入、录入、按卡分工核对、对账、打行，是一趟的全部步骤，拆开要把十几个量来回传"
)]
pub fn run_from_environment(
    flow: &CrashFlow<'_>,
    judging: &PipelineJudgingCode,
    repository_root: &Path,
    expand_up_to: ExpandUpToWritesPerSegment,
) -> Result<EnvironmentRun, String> {
    if let Some((segment_index, segment)) =
        flow.segments
            .iter()
            .enumerate()
            .find(|(segment_index, segment)| {
                (flow.expansion)(*segment_index, segment) != expand_up_to.expansion_of(segment)
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
    let exhaustive = expand_up_to.covers_every_segment(flow.segments);
    let configuration_file: Option<PathBuf> = match std::env::var("SINGLEFS_MULTI_HOST_CONFIG") {
        Ok(text) => {
            let configuration = path_relative_to_repository_root(repository_root, &text);
            if !configuration.is_file() {
                return Err(format!(
                    "SINGLEFS_MULTI_HOST_CONFIG 指的配置不在：{}",
                    configuration.display()
                ));
            }
            Some(configuration)
        }
        Err(_) => {
            let configuration = repository_root.join("multi-host.env");
            configuration.is_file().then_some(configuration)
        }
    };
    let mode = match &configuration_file {
        Some(configuration) => CrashAmplificationRunMode::from_configuration_file(configuration),
        None => CrashAmplificationRunMode::from_configuration_file_or_environment(
            &repository_root.join("multi-host.env"),
        ),
    };
    // 显卡的优先级与额度是这台开发机私有的，另放一份：`SINGLEFS_GPU_CARDS_CONFIG` 指到它；没设读环境变量（第二台由驱动随命令送）
    let gpu_cards_configuration: Option<PathBuf> = match std::env::var("SINGLEFS_GPU_CARDS_CONFIG")
    {
        Ok(text) => {
            let configuration = path_relative_to_repository_root(repository_root, &text);
            if !configuration.is_file() {
                return Err(format!(
                    "SINGLEFS_GPU_CARDS_CONFIG 指的配置不在：{}",
                    configuration.display()
                ));
            }
            Some(configuration)
        }
        Err(_) => None,
    };
    let cards_by_priority =
        cards_by_priority_from_configuration_or_environment(gpu_cards_configuration.as_deref())?;
    // 哪几类计算走 GPU：没开 GPU 不读登记；开了照登记，登记了的只走 GPU、没登记的只走 CPU，在开库之前定
    let registered_gpu_classes = if mode.uses_gpu() {
        gpu_computation_classes_from(
            std::env::var(GPU_COMPUTATION_CLASSES_KEY).ok().as_deref(),
            gpu_cards_configuration.as_deref(),
        )?
    } else {
        None
    };
    let routes = computation_routes(
        mode.uses_gpu(),
        registered_gpu_classes.as_ref(),
        cfg!(feature = "gpu"),
    )?;
    let host = CardHost::of_this_process()?;
    let placement_only = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY")
        .is_ok_and(|value| value == "1");
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
    let (library_directory, delete_after) =
        match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY") {
            Ok(text) => (
                path_relative_to_repository_root(repository_root, &text),
                false,
            ),
            Err(_) => (
                std::env::temp_dir().join(format!(
                    "singlefs-crash-amplification-{}",
                    std::process::id()
                )),
                true,
            ),
        };
    if !delete_after {
        refuse_a_library_that_a_reboot_would_erase(&library_directory)?;
    }
    let mut store = if library_directory.join("CURRENT").exists() {
        VerdictStore::open_existing(&library_directory)
            .map(|(store, _)| store)
            .map_err(|error| format!("开不了库 {}：{error:?}", library_directory.display()))?
    } else {
        std::fs::create_dir_all(library_directory.parent().unwrap_or(Path::new(".")))
            .map_err(|error| format!("建不了库的上层目录：{error}"))?;
        VerdictStore::create_empty(&library_directory)
            .map_err(|error| format!("建不了库 {}：{error:?}", library_directory.display()))?
    };
    let planning_started = std::time::Instant::now();
    let plan = plan_crash_points(flow);
    let version = judging.version();
    store
        .store_judge_version_description(&version, &judging.description())
        .map_err(|error| format!("登判法版本失败：{error:?}"))?;
    let mut report = CoverageReport::default();
    let recording = record_crash_points(&mut store, &plan, &version, &mut report)
        .map_err(|error| format!("录入失败：{error:?}"))?;
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
        states_already_judged_in_this_share(&store, &plan, &version, &|block| {
            selected.contains(&block)
        })
        .map_err(|error| format!("数复用失败：{error:?}"))?;
    let ignored_points = plan.points.iter().filter(|point| point.ignored).count();
    let states_skipped_by_ignore = states_skipped_by_ignore(flow, &plan);
    let mut imported_blocks = 0u64;
    if let Some(list) = &import_list {
        for directory in list.split(':').filter(|directory| !directory.is_empty()) {
            let (other, _) = VerdictStore::open_existing(&path_relative_to_repository_root(
                repository_root,
                directory,
            ))
            .map_err(|error| format!("开不了要导入的库 {directory}：{error:?}"))?;
            imported_blocks += store
                .import_from(&other)
                .map_err(|error| format!("导入 {directory} 失败：{error:?}"))?;
        }
    }
    let seconds_planning_and_recording = planning_started.elapsed().as_secs_f64();
    // 起过卡池的进程取卡不登记、不起设备，借到哪张卡才在哪张上起（文件头第 5 条）
    let (cards, gpu_cards_taken_from) = if routes.need_a_card() {
        let GpuCardsOfThisRun { cards, taken_from } =
            gpu_cards_in_use(cards_by_priority.as_deref(), host)?;
        (Some(cards), taken_from)
    } else {
        (None, GpuCardsSource::NoCardNeeded)
    };
    if placement_only {
        let placement = match (routes.crash_judge, &cards) {
            (ComputedOn::Gpu, Some(cards)) => print_the_placement(
                &store,
                flow,
                &plan,
                &|block| selected.contains(&block),
                &version,
                cards,
                gpu_cards_taken_from,
                cards_by_priority.as_deref(),
                host,
                states_reused_at_recording,
            )?,
            (ComputedOn::Gpu, None) | (ComputedOn::Cpu, _) => {
                return Err(format!(
                    "只排派活计划排的是 GPU 判器的活：要配置开着 GPU（ENABLE_GPU=1）、{GPU_COMPUTATION_CLASSES_KEY} 里登记 {}",
                    GpuComputationClass::CrashJudgeGpu.name()
                ));
            }
        };
        let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &version)
            .map_err(|error| format!("对账失败：{error:?}"))?
            .len();
        drop(store);
        if delete_after {
            let _ = std::fs::remove_dir_all(&library_directory);
        }
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
            // 起过卡池的进程借不到试判的草稿区：「本次未跑」那一行已经打了，没打 PLACEMENT 行（第 10v 批）
            stopped_early: placement == PlacementOfThisRun::NotRunTheGpuMemoryWasShort,
            judge_version: version,
            computation_routes: routes,
            expand_up_to,
            exhaustive,
            // 只排派活计划的那一趟一块都不判、不打 CRASH_AMPLIFICATION 行；复用数照实报（PLACEMENT 行也带）
            states_reused: states_reused_at_recording,
            ignored_points,
            states_skipped_by_ignore,
            gpu_cards_taken_from,
            unit_checksum_buffers_booked_mebibytes: None,
            facts_verifier_buffers_booked_mebibytes: None,
            unit_checksum_buffers_lease_on_the_books_after_checking: None,
            facts_verifier_buffers_lease_on_the_books_after_verifying: None,
        });
    }
    // 第 ① 段：登记了 crash_judge_gpu 就由 GPU 判器判这台机器领的块（判不了就报错，不退回 CPU）；没登记由 CPU 判器判，
    // 那时登记了 gpu_unit_checks 就把单元两道校验和另交一张卡对拍。显存不够（起过卡池的进程此刻借不到、卡上 wgpu 报了 Out of Memory）：
    // 「本次未跑」那一行已经打了，判过的块在库里，下面照叫停的样子交回（文件头第 5 条）
    let judging_started = std::time::Instant::now();
    // 这一趟的叫停：第 ① 段的外层循环、卡线程、一轮收尾与下面整趟收尾都问这一个，见过一次就记住（第 10z 批第六块，
    // 第 10z 批第五块代码轮判决 K2-2 攻方 F2：叫停文件中途被删时各自现读，轮末断言 panic、或整趟没记成提早收尾）
    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
    let stop_is_requested = || stop_request.is_requested();
    let (
        checking,
        gpu_judge,
        unit_checksum_buffers_booked_mebibytes,
        unit_checksum_buffers_lease_on_the_books_after_checking,
        gpu_memory_was_short,
    ) = match (routes.crash_judge, &cards) {
        (ComputedOn::Gpu, Some(cards)) => match judge_my_blocks_on_gpu(
            &mut store,
            flow,
            &plan,
            &|block| selected.contains(&block),
            &version,
            cards,
            gpu_cards_taken_from,
            &stop_is_requested,
        )? {
            GpuJudgingOfMyBlocks::Judged { tally, outcome } => {
                eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
                (tally, Some(outcome), None, None, false)
            }
            GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort { tally, reason: _ } => {
                (tally, None, None, None, true)
            }
        },
        (ComputedOn::Gpu, None) => {
            return Err(format!(
                "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 判器",
                GpuComputationClass::CrashJudgeGpu.name()
            ));
        }
        (ComputedOn::Cpu, _) => match unit_checksums_on(
            routes.unit_checksums,
            cards.as_ref(),
            gpu_cards_taken_from,
            flow,
        )? {
            // 借来的缓冲活到这一段判完；判完那一刻核账（第 10t 批代码轮判决 K4-1）
            UnitChecksumsOfThisRun::Ready {
                on,
                lease: buffers_held_until_checked,
                buffers_booked_mebibytes,
            } => {
                let checking = check_selected_blocks(
                    &mut store,
                    flow,
                    &plan,
                    &selected,
                    &on,
                    &version,
                    &stop_is_requested,
                )
                .map_err(|error| format!("核对失败：{error:?}"))?;
                // 对拍时卡上报了 Out of Memory：打「本次未跑」那一行，照叫停的样子交回（第 10z 批，文件头第 5 条）
                let gpu_memory_was_short = match &checking.stopped_because_the_gpu_memory_was_short
                {
                    Some(reason) => {
                        println!(
                            "{}",
                            singlefs_harness::gpu_memory_ledger::gpu_not_run_line(
                                PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                                reason
                            )
                        );
                        true
                    }
                    None => false,
                };
                (
                    checking,
                    None,
                    buffers_booked_mebibytes,
                    lease_on_the_books_after_its_work(&buffers_held_until_checked),
                    gpu_memory_was_short,
                )
            }
            UnitChecksumsOfThisRun::NotRunTheGpuMemoryWasShort => {
                (CheckingTally::default(), None, None, None, true)
            }
        },
    };
    let stop_was_requested = gpu_memory_was_short || stop_is_requested();
    let seconds_judging = judging_started.elapsed().as_secs_f64();
    // 第 ② 段：配置开了 GPU 就核，登记了 crash_verify_gpu 在一张卡上核（起过卡池的照账借第一张放得下的，第 ① 段的草稿区已经放掉）、
    // 没登记用 CPU 参照核；没开 GPU 而 SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu 也用 CPU 参照核；被叫停的不核
    let verifying_started = std::time::Instant::now();
    let facts_verifier_route = if stop_was_requested {
        None
    } else if mode.uses_gpu()
        || std::env::var("SINGLEFS_CRASH_AMPLIFICATION_VERIFY").is_ok_and(|value| value == "cpu")
    {
        Some(routes.facts_verifier)
    } else {
        None
    };
    let (
        verification,
        verifying_cards_used,
        facts_verifier_buffers_booked_mebibytes,
        facts_verifier_buffers_lease_on_the_books_after_verifying,
        facts_were_not_verified_for_short_gpu_memory,
    ) = match (facts_verifier_route, &cards) {
        (Some(ComputedOn::Gpu), Some(cards)) => match verify_my_blocks_on_the_first_card(
            &mut store,
            flow,
            &plan,
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
            return Err(format!(
                "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回 CPU 参照核",
                GpuComputationClass::CrashVerifyGpu.name()
            ));
        }
        (None, _) => (None, 0, None, None, false),
        (Some(ComputedOn::Cpu), _) => {
            let on = FactsVerifierOn::Cpu;
            let tally = verify_blocks_from_facts(
                &mut store,
                flow,
                &plan,
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
        let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &version)
            .map_err(|error| format!("对账失败：{error:?}"))?
            .len();
        eprintln!(
            "CRASH_AMPLIFICATION_STOPPED blocks_checked_here={} states_checked_here={} unjudged_blocks={unjudged}",
            checking.blocks_checked, checking.states_checked
        );
        drop(store);
        if delete_after {
            let _ = std::fs::remove_dir_all(&library_directory);
        }
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
            stopped_early: true,
            judge_version: version,
            computation_routes: routes,
            expand_up_to,
            exhaustive,
            // 被叫停的那一趟照实报它录入时库里已判的（L1）；接着跑的那一趟另报它自己的（reuse_and_scale_fields_of_the_line）
            states_reused: states_reused_at_recording,
            ignored_points,
            states_skipped_by_ignore,
            gpu_cards_taken_from,
            unit_checksum_buffers_booked_mebibytes,
            facts_verifier_buffers_booked_mebibytes,
            unit_checksum_buffers_lease_on_the_books_after_checking,
            facts_verifier_buffers_lease_on_the_books_after_verifying,
        });
    }
    // 用了几张卡：第 ① 段在卡上判的按它用的，第 ② 段在卡上核、单元校验和在卡上对拍的各一张
    let cards_used = [
        gpu_judge.as_ref().map_or(0, |outcome| outcome.cards_used),
        verifying_cards_used,
        usize::from(
            routes.crash_judge == ComputedOn::Cpu && routes.unit_checksums == ComputedOn::Gpu,
        ),
    ]
    .into_iter()
    .max()
    .expect("三样里取最大");
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
                compare_judge_and_verifier(&store, &plan, &digest, &version)
                    .map_err(|error| format!("第 ③ 段比对失败：{error:?}"))?,
            )
        }
        None => None,
    };
    let seconds_comparing = comparing_started.elapsed().as_secs_f64();
    let reconciling_started = std::time::Instant::now();
    let unjudged = unjudged_blocks_of_the_flow(&store, &plan, &version)
        .map_err(|error| format!("对账失败：{error:?}"))?
        .len();
    let violations = read_violations(&store, &plan, &version)
        .map_err(|error| format!("读违例失败：{error:?}"))?;
    let coverage_lines = report.lines().to_vec();
    // 一趟的挂钟花在哪一段：排计划与录入、导入（CPU），第 ① 段判（GPU 判的含建表、表上卡、编内核、拆结论、写库），
    // 第 ② 段核（含抽事实表），第 ③ 段比与对账读违例（CPU）
    eprintln!(
        "CRASH_AMPLIFICATION_PHASES states_checked_here={} seconds_planning_and_recording={seconds_planning_and_recording:.1} seconds_judging={seconds_judging:.1} seconds_verifying={seconds_verifying:.1} seconds_comparing={seconds_comparing:.1} seconds_reconciling={:.1}",
        checking.states_checked,
        reconciling_started.elapsed().as_secs_f64()
    );
    drop(store);
    if delete_after {
        let _ = std::fs::remove_dir_all(&library_directory);
    }
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
        stopped_early: false,
        judge_version: version,
        computation_routes: routes,
        expand_up_to,
        exhaustive,
        states_reused: states_reused_at_recording,
        ignored_points,
        states_skipped_by_ignore,
        gpu_cards_taken_from,
        unit_checksum_buffers_booked_mebibytes,
        facts_verifier_buffers_booked_mebibytes,
        unit_checksum_buffers_lease_on_the_books_after_checking,
        facts_verifier_buffers_lease_on_the_books_after_verifying,
    })
}

/// 开机会清空的目录：Ubuntu 的 tmpfiles 规则 `D /tmp` 与 `D /var/tmp` 在开机时删光内容，`/dev/shm` 与 `/run` 是内存文件系统。
/// 库落在这里，一次重启就续不上（2026-09-29 到 E 全域那一趟的库与树就丢在这里）。
const DIRECTORIES_A_REBOOT_ERASES: [&str; 4] = ["/tmp", "/var/tmp", "/dev/shm", "/run"];

/// 显式给的库目录落在开机会清空的地方就拒绝，除非环境写明这份库是一次性的（`SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1`，测试用）。
/// 判的是现存最长前缀的绝对路径（目录还没建时判它的上层）。
///
/// # Errors
/// 库目录落在 [`DIRECTORIES_A_REBOOT_ERASES`] 之下而环境没写明一次性。
pub fn refuse_a_library_that_a_reboot_would_erase(library_directory: &Path) -> Result<(), String> {
    if std::env::var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY")
        .is_ok_and(|value| value == "1")
    {
        return Ok(());
    }
    let mut probe = library_directory.to_path_buf();
    let resolved = loop {
        if let Ok(resolved) = probe.canonicalize() {
            break resolved;
        }
        match probe.parent() {
            Some(parent) if parent != probe => probe = parent.to_path_buf(),
            _ => break library_directory.to_path_buf(),
        }
    };
    for erased in DIRECTORIES_A_REBOOT_ERASES {
        if resolved == Path::new(erased) || resolved.starts_with(erased) {
            return Err(format!(
                "库目录 {} 落在开机会清空的 {erased} 之下：重启一次就续不上。放到家目录或别的持久目录下；\
                 只有测试里一次性的库才写 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1 放行",
                library_directory.display()
            ));
        }
    }
    Ok(())
}

/// 这一趟用的卡：编进 `gpu` 特性时是起好上下文的卡，按优先级；没编进来时没有这个东西。
#[cfg(feature = "gpu")]
type CardsInUse = Vec<std::sync::Arc<crate::gpu_unit_checks::GpuCard>>;
#[cfg(not(feature = "gpu"))]
type CardsInUse = ();

/// 起过卡池的进程为第 ② 段或单元校验和对拍在显存账上借的那一笔：活到那一段算完；卡不是从池里来的不借（`None`）。
/// 没编进 `gpu` 特性时没有这个东西。
#[cfg(feature = "gpu")]
type BufferLeaseOnTheCardPool = Option<crate::gpu_unit_checks::gpu_test_start::GpuCardLease>;
#[cfg(not(feature = "gpu"))]
type BufferLeaseOnTheCardPool = ();

/// 一段算完那一刻核账：那一段的缓冲借的那一笔还在这个进程的账上没有（第 10t 批代码轮判决 K4-1：借了又在用完之前还掉要红）；没借是 `None`。
#[cfg(feature = "gpu")]
fn lease_on_the_books_after_its_work(lease: &BufferLeaseOnTheCardPool) -> Option<bool> {
    lease
        .as_ref()
        .map(crate::gpu_unit_checks::gpu_test_start::GpuCardLease::is_on_the_books)
}

/// 没编进 `gpu` 特性时不借，没有可核的。
#[cfg(not(feature = "gpu"))]
fn lease_on_the_books_after_its_work(_lease: &BufferLeaseOnTheCardPool) -> Option<bool> {
    None
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
struct GpuCardsOfThisRun {
    cards: CardsInUse,
    taken_from: GpuCardsSource,
}

/// 这个进程用哪几张卡：配置点了名的只用这台机器那几张、照配置的次序与额度；没配就是本机能用的卡按空闲显存从大到小。
/// 这个进程起过 GPU 用例共用的卡池（[`crate::gpu_unit_checks::gpu_test_start::card_pool_started_in_this_process`]）就照池里的卡用
/// （池里的卡要与配置对得上，[`the_card_pool_matches_the_configured_cards`]），不另起一套设备，此刻也不登记、不起设备：
/// 借到哪张卡才在哪张上起（第 10t 批代码轮判决 K2-3）。没起过池才自己起。
#[cfg(feature = "gpu")]
fn gpu_cards_in_use(
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

/// 第 ① 段由 CPU 判时，单元两道校验和在哪对拍（[`unit_checksums_on`]）。
#[cfg_attr(
    not(feature = "gpu"),
    allow(
        dead_code,
        reason = "没编进 gpu 特性时单元校验和只在 CPU 上，借不到缓冲那一形只在编进来时造"
    )
)]
#[allow(
    clippy::large_enum_variant,
    reason = "一趟只造一个、交给调用方就拆开，不进集合：卡池的借据多带一份账上的设备（第 10z 批第八块）之后过了门槛，装箱省不下什么"
)]
enum UnitChecksumsOfThisRun {
    /// `on` 是在哪对拍；`lease` 是起过卡池的进程为对拍的缓冲借的那一笔（活到这一段判完），`buffers_booked_mebibytes` 是借的量。
    Ready {
        on: UnitChecksumsOn,
        lease: BufferLeaseOnTheCardPool,
        buffers_booked_mebibytes: Option<u64>,
    },
    /// 起过卡池的进程此刻借不到对拍的缓冲：「本次未跑」那一行已经打了。
    NotRunTheGpuMemoryWasShort,
}

/// 第 ① 段由 CPU 判时，单元两道校验和在哪对拍：登记了 `gpu_unit_checks` 交一张卡，没登记只在 CPU 上。
/// 卡是这一趟自己起的交优先级最高的那张；起过卡池的照对拍的缓冲（`unit_checksum_buffer_footprint`，这条流的单元写数、核对线程数）
/// 在账上借第一张此刻放得下的，用借来的那张算；借不到打「本次未跑」那一行（文件头第 5 条）。
///
/// # Errors
/// 登记了却没有起好的卡（不退回只在 CPU 上算）；起过卡池的进程借缓冲时要的比每张卡空着时能借的都大、卡上的显存读不出。
#[cfg(feature = "gpu")]
fn unit_checksums_on(
    route: ComputedOn,
    cards: Option<&CardsInUse>,
    cards_taken_from: GpuCardsSource,
    flow: &CrashFlow<'_>,
) -> Result<UnitChecksumsOfThisRun, String> {
    let ready_without_a_lease = |on: UnitChecksumsOn| UnitChecksumsOfThisRun::Ready {
        on,
        lease: None,
        buffers_booked_mebibytes: None,
    };
    match route {
        ComputedOn::Cpu => Ok(ready_without_a_lease(UnitChecksumsOn::CpuOnly)),
        ComputedOn::Gpu => {
            let no_card = || {
                format!(
                    "登记了 {} 走 GPU，这一趟却没有起好的卡：不退回只在 CPU 上算",
                    GpuComputationClass::GpuUnitChecks.name()
                )
            };
            let cards = cards.ok_or_else(no_card)?;
            match cards_taken_from {
                GpuCardsSource::NoCardNeeded | GpuCardsSource::DevicesStartedForThisRun => {
                    let first_card = cards.first().ok_or_else(no_card)?;
                    Ok(ready_without_a_lease(UnitChecksumsOn::Gpu(
                        first_card.clone(),
                    )))
                }
                GpuCardsSource::TheCardPoolOfThisProcess => {
                    let unit_writes = flow
                        .writes
                        .iter()
                        .filter(|write| write.kind == StepKind::UnitWrite)
                        .count();
                    let need_mebibytes =
                        unit_checksum_buffer_footprint(unit_writes, checking_threads()).mebibytes();
                    match the_card_pool_this_run_takes_its_cards_from()
                        .lease_the_first_card_with_room_now(
                            PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE,
                            need_mebibytes,
                        ) {
                        Ok(lease) => Ok(UnitChecksumsOfThisRun::Ready {
                            on: UnitChecksumsOn::Gpu(std::sync::Arc::new(lease.card().clone())),
                            buffers_booked_mebibytes: Some(lease.mebibytes()),
                            lease: Some(lease),
                        }),
                        Err(refusal) => {
                            print_the_not_run_line_or_refuse("单元校验和对拍的缓冲", &refusal)?;
                            Ok(UnitChecksumsOfThisRun::NotRunTheGpuMemoryWasShort)
                        }
                    }
                }
            }
        }
    }
}

/// 没编进 `gpu` 特性：登记了 `gpu_unit_checks` 走不到这里（[`computation_routes`] 先报错），走到了照样报错。
///
/// # Errors
/// 登记了 `gpu_unit_checks`。
#[cfg(not(feature = "gpu"))]
fn unit_checksums_on(
    route: ComputedOn,
    _cards: Option<&CardsInUse>,
    _cards_taken_from: GpuCardsSource,
    _flow: &CrashFlow<'_>,
) -> Result<UnitChecksumsOfThisRun, String> {
    match route {
        ComputedOn::Cpu => Ok(UnitChecksumsOfThisRun::Ready {
            on: UnitChecksumsOn::CpuOnly,
            lease: (),
            buffers_booked_mebibytes: None,
        }),
        ComputedOn::Gpu => Err(format!(
            "登记了 {} 走 GPU，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）",
            GpuComputationClass::GpuUnitChecks.name()
        )),
    }
}

/// 配置要 GPU 而这份二进制没编进 `gpu` 特性：报错，不悄悄退回 CPU。
#[cfg(not(feature = "gpu"))]
fn gpu_cards_in_use(
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
fn judge_my_blocks_on_gpu(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    version: &JudgeVersion,
    cards: &CardsInUse,
    cards_taken_from: GpuCardsSource,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<GpuJudgingOfMyBlocks, String> {
    judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from(
        store,
        flow,
        plan,
        is_mine,
        version,
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
fn judge_my_blocks_on_gpu(
    _store: &mut VerdictStore,
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
    _is_mine: &dyn Fn(u64) -> bool,
    _version: &JudgeVersion,
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
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    my_blocks: &[u64],
    cards: &CardsInUse,
    cards_taken_from: GpuCardsSource,
) -> Result<FactsVerifiedOnACard, String> {
    let facts = extract_flow_facts(flow.base, flow.writes, flow.segments, flow.expansion);
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
                facts_verifier_buffer_footprint(&facts, STATES_PER_BLOCK).mebibytes();
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
        verify_blocks_from_these_facts(store, plan, &facts, &|block| selected.contains(&block), &on)
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
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
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
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    version: &JudgeVersion,
    cards: &CardsInUse,
    cards_taken_from: GpuCardsSource,
    configured: Option<&[CardByPriority]>,
    host: CardHost,
    states_reused: u64,
) -> Result<PlacementOfThisRun, String> {
    use crate::crash_judge_dispatch::{states_in_flight_within, GpuCrashJudge, PackedJudgeTables};
    use crate::crash_judge_tables::{build_judge_tables_against_the_version_table, JudgeFlow};
    /// 量完宽度之后计时判几次派活那么多的状态。
    const CALIBRATION_DISPATCHES: u64 = 8;
    let pending = pending_blocks(store, plan, version, is_mine)
        .map_err(|error| format!("读判定块失败：{error:?}"))?;
    let pending_states: u64 = pending
        .iter()
        .map(|(_, local)| local.end - local.start)
        .sum();
    let expansion = |segment_index: usize, segment: &[usize]| {
        plan.segment_expansion(flow, segment_index, segment)
    };
    let judge_flow = JudgeFlow {
        base: flow.base,
        writes: flow.writes,
        segments: flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: flow.versions,
        expansion: &expansion,
    };
    let (states_of_the_flow, tables) = match build_judge_tables_against_the_version_table(
        &judge_flow,
        flow.versions_without_a_file,
    ) {
        Ok(tables) => (tables.state_count, PackedJudgeTables::of(&tables)),
        Err(refusal) => {
            return Err(format!(
                "GPU 判器建不出这条流的表（{refusal:?}；{} 块、{pending_states} 个状态要判）。配置开了 GPU，不退回 CPU 判器：让判器的表认得这种写再跑",
                pending.len()
            ));
        }
    };
    let local_cards = card_capacities_of_this_run(cards_taken_from, cards, &tables)?;
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
                        &tables,
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
                &tables,
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
                        &tables,
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
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
    _is_mine: &dyn Fn(u64) -> bool,
    _version: &JudgeVersion,
    _cards: &CardsInUse,
    _cards_taken_from: GpuCardsSource,
    _configured: Option<&[CardByPriority]>,
    _host: CardHost,
    _states_reused: u64,
) -> Result<PlacementOfThisRun, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}
