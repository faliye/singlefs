//! 崩溃放量流水线（里程碑二「增补 4」第一、四、十一项；用户 2026-09-28 定的形态）：
//!
//! 1. **崩溃点与路径**：一条测试流程是从 mkfs 出发、一个崩溃点接一个崩溃点走下来的路径；每段归含它最后一次写的那个点，上游不看下游。每个点的身份是
//!    `<仓内路径>::<崩溃点名称>::<哈希串>`（[`crate::crash_identity`]）。几条流程共用前缀，全部点连成一棵树（用户 2026-09-28：「这就是个 B 树的问题」）：
//!    路径编号是从根到这个点、每个点取输入键（点名、写表、父镜像的摘要）前 16 个十六进制、用 `/` 逐段接起来的串：
//!    前面几段相同就是同一段枝干，一棵子树就是一段连续的键。哈希串（复用键）只由流程定：这一步写下的字节与它起步的镜像；
//!    按 COW 的性质，一个点的输入不变，它的崩溃状态就不变。判法不进键：判定块按（块键，判法版本）存，判法改了旧行留作历史、
//!    只在同一判法版本下复用（用户 2026-09-29 定：结果只绑流程，崩溃点变了或树变了才重测，代码、commit、树都不进编号）。
//! 2. **先录后核**：录入把每个点的状态按块（点内序号区间）登进库的录入表；核对按批从库里取还没有判定的块，
//!    在 CPU 上（或 CPU + GPU：单元两道校验和上 GPU，与 CPU 对拍）判完、原子地写回判定块与违例；门禁一次读完。
//! 3. **对账**：一轮收尾，录入表里有、判定块表里没有的块就是没核完的（卡掉线、进程被杀），重新分给还在的一方补跑；
//!    领块只在内存里，库里不记谁领了，不会有块被一张死卡占着。
//! 4. **分工与跑法**：块按权重分给几方（一方一台机器的 CPU，或一张卡，权重取空闲显存），各写各的库，最后
//!    [`crate::verdict_store::VerdictStore::import_from`] 并成一个；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
#[cfg(feature = "gpu")]
use std::time::Instant;

use singlefs_harness::memory_pool::{MemoryPool, PublishedVersion, RetainedWrite, WrittenContents};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash::{
    judge_layer0_state_range, layer0_state_ranges_by_segment, Layer0SegmentExpansion,
    LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE, LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
    LAYER0_RED_PASS_POOL_CHECKER, LAYER0_RED_PASS_RECORD_CHECKER,
    RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT, RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};
use crate::crash_facts::{extract_flow_facts, verify_states_from_facts, FlowFacts};
use crate::crash_identity::{
    judging_code_of_the_judges, step_write_table_with_barriers, CoverageReport, CoverageState,
    JudgingCode, JudgingCodeError, NodeLabel, ReuseKey, SectorImage, StepAttributes,
};
use crate::verdict_store::{
    BlockStartStateIndex, EncodedVerdictVector, EnumerationPlanHash, InputFingerprint,
    JudgeVersion, StateOffsetInBlock, StreamOrNodeName, VerdictBlock, VerdictBlockKey,
    VerdictStore, VerdictStoreError,
};

/// 一块最多几个状态：一块是一个 GPU 批、一次原子写、对账与分工的一个单位（里程碑二「增补 4」第十一项「一块取 2¹⁶ 个状态」）。
pub const STATES_PER_BLOCK: u64 = 1 << 16;

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
pub struct CrashFlow<'flow> {
    pub base: &'flow MemoryPool,
    pub writes: &'flow [RetainedWrite],
    pub segments: &'flow [Vec<usize>],
    pub judged_root_index: usize,
    pub versions: &'flow [PublishedVersion],
    pub expansion: &'flow (dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
    pub crash_points: Vec<CrashPointSpan>,
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
/// 计划不看判法：判法是判定行的属性（[`PipelineJudgingCode::version`]），改了它树与块键都不动。
///
/// # Panics
/// 崩溃点的写区间不是首尾相接地罩住整张写表；某个点的名称或文件路径不合身份的规矩（同一个名字在一条流程里可以出现几次：
/// 同一个函数调几次就是几个点，树上的位置分得开）；
/// 各段的状态序号接不上（枚举计划与切段不一致，是这里的 bug）。
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
        planned.push(PlannedCrashPoint {
            name: point.name.clone(),
            ignored: point.ignored,
            segments: owned,
            ordinals,
            reuse_key,
            identity,
            path_number,
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

/// 配置里的一张卡在哪台机器上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardHost {
    Local,
    Peer,
}

impl CardHost {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Peer => "peer",
        }
    }

    /// 这个进程在哪台机器上：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机。
    ///
    /// # Errors
    /// 写的不是 `local` 也不是 `peer`。
    pub fn of_this_process() -> Result<Self, String> {
        match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE") {
            Err(_) => Ok(Self::Local),
            Ok(text) => Self::parse(&text),
        }
    }

    fn parse(text: &str) -> Result<Self, String> {
        match text.trim() {
            "local" => Ok(Self::Local),
            "peer" => Ok(Self::Peer),
            other => Err(format!("机器要写 local 或 peer，实际「{other}」")),
        }
    }
}

/// 配置点名的一张卡（`GPU_CARDS_BY_PRIORITY` 里的一项）：在哪台机器、`nvidia-smi` 的序号、给崩溃放量的显存额度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardByPriority {
    pub host: CardHost,
    pub index: u32,
    pub memory_quota_mebibytes: u64,
}

/// 读 `GPU_CARDS_BY_PRIORITY` 的值：空白隔开的若干项，每项 `<local|peer>:<nvidia-smi 序号>:<显存额度>`，
/// 额度是正整数加 `G` 或 `M`（1024 进制）；写在前面的优先级高，任务先装它、装满了才给后面的。
///
/// # Errors
/// 一项不是三段、机器或序号或额度写法不对、同一张卡点名两次、一项都没有。
pub fn parse_cards_by_priority(text: &str) -> Result<Vec<CardByPriority>, String> {
    let mut cards: Vec<CardByPriority> = Vec::new();
    for entry in text.split_whitespace() {
        let fields: Vec<&str> = entry.split(':').collect();
        let [host, index, quota] = fields.as_slice() else {
            return Err(format!(
                "显卡优先级的一项要写 <local|peer>:<序号>:<显存额度>，实际「{entry}」"
            ));
        };
        let host = CardHost::parse(host)?;
        let index: u32 = index
            .parse()
            .map_err(|_| format!("显卡序号不是数：「{entry}」"))?;
        let (number, unit_mebibytes) = match quota.as_bytes().last() {
            Some(b'G') => (&quota[..quota.len() - 1], 1024u64),
            Some(b'M') => (&quota[..quota.len() - 1], 1u64),
            _ => {
                return Err(format!(
                    "显存额度要带单位 G 或 M（1024 进制），实际「{entry}」"
                ))
            }
        };
        let number: u64 = number
            .parse()
            .ok()
            .filter(|number| *number > 0)
            .ok_or_else(|| format!("显存额度不是正整数：「{entry}」"))?;
        if cards
            .iter()
            .any(|card| card.host == host && card.index == index)
        {
            return Err(format!("同一张卡点名了两次：「{entry}」"));
        }
        cards.push(CardByPriority {
            host,
            index,
            memory_quota_mebibytes: number * unit_mebibytes,
        });
    }
    if cards.is_empty() {
        return Err("显卡优先级一项都没写".to_string());
    }
    Ok(cards)
}

/// `键=值` 配置文件里一个键的值（去掉两头的空白与一层引号）；文件不在、键没写是 `None`。
#[must_use]
pub fn configuration_value(path: &Path, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    text.lines().find_map(|line| {
        let value = line.trim().strip_prefix(key)?.strip_prefix('=')?.trim();
        let value = value
            .strip_prefix('\'')
            .and_then(|rest| rest.strip_suffix('\''))
            .or_else(|| {
                value
                    .strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix('"'))
            })
            .unwrap_or(value);
        Some(value.to_string())
    })
}

/// 配置的显卡优先级：环境变量 `GPU_CARDS_BY_PRIORITY` 设了就照它（驱动脚本按清没清场从显卡配置里挑一份送过来，第二台的树副本也靠它），
/// 没设读显卡配置文件（`SINGLEFS_GPU_CARDS_CONFIG` 指的那一份，模板是仓根的 `gpu-cards.env.example`）里的同名键；
/// 都没有是 `None`（本机能用的卡按空闲显存从大到小用，不设额度）。
///
/// # Errors
/// 写了而写法不对（[`parse_cards_by_priority`]）。
pub fn cards_by_priority_from_configuration_or_environment(
    configuration_file: Option<&Path>,
) -> Result<Option<Vec<CardByPriority>>, String> {
    let text = match std::env::var("GPU_CARDS_BY_PRIORITY") {
        Ok(text) => Some(text),
        Err(_) => {
            configuration_file.and_then(|path| configuration_value(path, "GPU_CARDS_BY_PRIORITY"))
        }
    };
    match text {
        Some(text) if !text.trim().is_empty() => parse_cards_by_priority(&text).map(Some),
        _ => Ok(None),
    }
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
}

pub const RED_TEXTS_KEPT: usize = 64;

/// 核对线程数：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_THREADS`，没设取本机可用并行度（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」）。
#[must_use]
pub fn checking_threads() -> usize {
    std::env::var("SINGLEFS_CRASH_AMPLIFICATION_THREADS")
        .ok()
        .and_then(|text| text.parse::<usize>().ok())
        .filter(|count| *count >= 1)
        .unwrap_or_else(|| {
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
        })
}

/// 一块判完的结果（还没写库）：每个状态判红的各项、红状态的正文、GPU 与 CPU 对不上的状态数。
struct JudgedBlock {
    local: Range<u64>,
    items_of_each_state: Vec<Vec<String>>,
    red_texts: Vec<(u64, String)>,
    gpu_cpu_disagreements: u64,
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
    let stream_state_count = judge_layer0_state_range(
        flow.base,
        flow.writes,
        flow.segments,
        flow.judged_root_index,
        flow.versions,
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
    }
}

/// 核对录入表里还没有判定的块：按分工只核 `owner` 那一方分到的；一块切成几段多线程判（[`checking_threads`]），
/// 每块在 CPU 上逐状态判（`unit_checksums` 给了卡就把每个状态镜像里被写过的单元两道校验和交 GPU 算、与 CPU 逐个对拍），
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
    // 块是库与 GPU 批的粒度，不改小；一块的序号切成线程数那么多段、一段一个线程（一块 65536 个状态、32 线程时一段 2048 个），
    // 判完按段序拼回一块、当场写库：中途被停，写进库的块下一次不再判。
    let threads = checking_threads();
    let mut tally = CheckingTally::default();
    for (pending_index, (point_index, local)) in pending.iter().enumerate() {
        if stop_is_requested() {
            break;
        }
        let point = &plan.points[*point_index];
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
        let judged: Vec<JudgedBlock> = std::thread::scope(|scope| {
            let handles: Vec<_> = pieces
                .iter()
                .map(|range| {
                    let range = range.clone();
                    scope.spawn(move || {
                        judge_one_block(flow, plan, *point_index, range, unit_checksums)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("核对线程没有 panic"))
                .collect()
        });
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
        let mut numbers = Vec::with_capacity(items_of_each_state.len());
        let mut violating = Vec::new();
        for (offset, items) in items_of_each_state.iter().enumerate() {
            if !items.is_empty() {
                tally.red_states += 1;
                violating.push(StateOffsetInBlock(
                    u64::try_from(offset).expect("块内序号装得进 u64"),
                ));
            }
            numbers.push(store.register_verdict_vector(&encoded_verdict(items))?);
        }
        let verdict_block = VerdictBlock::new(numbers).expect("一块至少一个状态");
        store.store_verdict_block(&point.block_key(local), version, &verdict_block, &violating)?;
        tally.blocks_checked += 1;
        tally.states_checked += local.end - local.start;
    }
    Ok(tally)
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

/// 配置点名的卡里哪几张（序号）不在用得上的卡里，按配置的次序。
#[must_use]
pub fn configured_cards_not_usable(configured: &[(u32, u64)], usable: &[u32]) -> Vec<u32> {
    configured
        .iter()
        .map(|(index, _)| *index)
        .filter(|index| !usable.contains(index))
        .collect()
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
        if violated & (1u64 << index) != 0 {
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
    /// 用到的每张卡一行，按优先级。
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
    next_pending: std::sync::atomic::AtomicUsize,
    /// 写库失败之后置上：卡上的线程判完手里那一块就停。
    stop: std::sync::atomic::AtomicBool,
    /// 调用方要停（每领一块之前问一次）：不再领新的块，手里那一块判完照旧交去写库。
    stop_is_requested: &'work (dyn Fn() -> bool + Sync),
}

#[cfg(feature = "gpu")]
impl CardsAtWork<'_> {
    fn must_stop(&self) -> bool {
        self.stop.load(std::sync::atomic::Ordering::SeqCst) || (self.stop_is_requested)()
    }
}

/// 一张卡的线程：表上卡、编内核；备好之后，队里还有活、下一张卡也装了状态，就起下一张卡的线程（卡一张一张备，不同时编内核）；
/// 然后从队头一块一块领、判、把结论字拆成红项、交给写库的线程。有一个状态判不了（结论字带「判不了」），
/// 就叫所有卡停下、发一条 [`CardMessage::Refused`]，不退回 CPU 判器（用户 2026-09-29 定：配了 GPU 就一定要 GPU 判，做不到就报错）。
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
    let judge = GpuCrashJudge::on_card_with_packed_tables(
        card,
        work.tables,
        usize::try_from(work.states_placed[position]).expect("一次派活的状态数装得进 usize"),
    );
    let build_seconds = build_started.elapsed().as_secs_f64();
    eprintln!(
        "CRASH_AMPLIFICATION_GPU_JUDGE card={:?} index={} memory_budget_mebibytes={} states_per_dispatch={} scratch_mebibytes={} build_seconds={build_seconds:.1}",
        card.name,
        card.index,
        card.memory_budget_mebibytes(),
        judge.states_per_dispatch(),
        judge.scratch_mebibytes()
    );
    let next_position =
        (position + 1..work.cards.len()).find(|candidate| work.states_placed[*candidate] > 0);
    if let Some(next_position) = next_position {
        if work.next_pending.load(Ordering::SeqCst) < work.pending.len() && !work.must_stop() {
            let sender = sender.clone();
            scope.spawn(move || {
                judge_on_one_card_and_start_the_next(scope, work, next_position, sender);
            });
        }
    }
    let judging_started = Instant::now();
    let mut outcome = GpuCardOutcome {
        name: card.name.clone(),
        index: card.index,
        memory_budget_mebibytes: card.memory_budget_mebibytes(),
        states_per_dispatch: u64::try_from(judge.states_per_dispatch()).expect("装得进 u64"),
        scratch_mebibytes: u64::try_from(judge.scratch_mebibytes()).expect("装得进 u64"),
        build_seconds,
        ..GpuCardOutcome::default()
    };
    while !work.must_stop() {
        let pending_index = work.next_pending.fetch_add(1, Ordering::SeqCst);
        if pending_index >= work.pending.len() {
            break;
        }
        let (point_index, local) = &work.pending[pending_index];
        let point = &work.plan.points[*point_index];
        let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
        let words = judge.judge_states(ordinals.clone());
        let decoding_started = Instant::now();
        if let Some(offset) = first_state_the_gpu_could_not_judge(&words) {
            let state_words = &words[offset];
            work.stop.store(true, Ordering::SeqCst);
            // 写库的线程已经走了（写库失败）就没人收，那一边的错先报
            let _ = sender.send(CardMessage::Refused(format!(
                "卡 {} 判不了点 {} 块内第 {} 个状态（恢复内核第 1 字 {}、checker 内核第 {} 字 {}：表越过了内核的容量）。\
                 配置开了 GPU，不退回 CPU 判器：照这两个字查是哪张表满了、改内核的容量再跑",
                card.index,
                point.name,
                local.start + u64::try_from(offset).expect("块内序号"),
                state_words[1],
                crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD,
                state_words[crate::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD]
            )));
            return;
        }
        let items_of_each_state: Vec<Vec<String>> = words
            .iter()
            .map(|state_words| gpu_verdict_items(state_words).expect("判不了的状态上面已经拒过"))
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
        if sender
            .send(CardMessage::Block(GpuJudgedBlock {
                pending_index,
                items_of_each_state,
            }))
            .is_err()
        {
            break;
        }
    }
    outcome.judge_seconds = judging_started.elapsed().as_secs_f64();
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

/// 每张卡按它的显存额度一次派活带得了几个状态（表上卡之后剩下的显存装草稿区），按 `cards` 的次序。
#[cfg(feature = "gpu")]
fn states_in_flight_of_each_card(
    cards: &[std::sync::Arc<crate::gpu_unit_checks::GpuCard>],
    tables: &crate::crash_judge_dispatch::PackedJudgeTables,
) -> Vec<u64> {
    cards
        .iter()
        .map(|card| {
            let limit = card.device().limits().max_storage_buffer_binding_size;
            if tables.mebibytes() * 1024 * 1024 > limit
                || card.memory_budget_mebibytes()
                    < tables.mebibytes()
                        + crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER
            {
                return 0;
            }
            u64::try_from(crate::crash_judge_dispatch::states_in_flight_within(
                limit,
                card.memory_budget_mebibytes(),
                tables,
            ))
            .expect("装得进 u64")
        })
        .collect()
}

/// 第 ① 段在卡上。卡按 `cards` 的次序（优先级）用：要判的状态先装第一张卡，装到它的显存额度带得了的状态数为止，
/// 装不下的才给下一张（[`place_states_on_cards_by_priority`]）；用到的卡一张一张备（表上卡、编内核），备好就从同一队里领块。
/// 判完的块按块序一块一块写库（后判完的先到就等前面的）：中途被停，写进库的块下一次不再判；
/// 判定向量的编号按块序登记，与用了几张卡无关。配置开了 GPU 就一定由 GPU 判（用户 2026-09-29 定，不接受默认降级）：
/// 建不出表（有清零写、写与落点对不上）、有状态判不了（表越过内核容量），都报错，不退回 CPU 判器；报错之前判完的块照旧在库里。
/// 每领一块之前先问 `stop_is_requested`：要停就不再领，已经领的块判完、写进库再收尾（领的块按块序连着，库里不留空洞）。
///
/// # Errors
/// 库读写失败；建不出表；没有一张卡装得下这条流的表与一个状态的草稿区；有状态 GPU 判不了。
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
    use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
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
    let tables = match build_judge_tables(&judge_flow) {
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
        return Ok((tally, outcome));
    }
    let tables = crate::crash_judge_dispatch::PackedJudgeTables::of(&tables);
    let pending_states: u64 = pending
        .iter()
        .map(|(_, local)| local.end - local.start)
        .sum();
    let capacities = states_in_flight_of_each_card(cards, &tables);
    let states_placed = place_states_on_cards_by_priority(pending_states, &capacities);
    for ((card, capacity), placed) in cards.iter().zip(&capacities).zip(&states_placed) {
        eprintln!(
            "CRASH_AMPLIFICATION_GPU_PLACEMENT card={:?} index={} memory_budget_mebibytes={} states_in_flight_capacity={capacity} states_placed={placed} pending_states={pending_states} tables_mebibytes={}",
            card.name,
            card.index,
            card.memory_budget_mebibytes(),
            tables.mebibytes()
        );
    }
    let Some(first_position) = states_placed.iter().position(|placed| *placed > 0) else {
        return Err(format!(
            "没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",
            tables.mebibytes()
        ));
    };
    let work = CardsAtWork {
        plan,
        pending: &pending,
        tables: &tables,
        cards,
        states_placed: &states_placed,
        next_pending: std::sync::atomic::AtomicUsize::new(0),
        stop: std::sync::atomic::AtomicBool::new(false),
        stop_is_requested,
    };
    let started = Instant::now();
    let mut storing_error: Option<String> = None;
    let mut refusal: Option<String> = None;
    std::thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::channel::<CardMessage>();
        let work = &work;
        scope.spawn(move || {
            judge_on_one_card_and_start_the_next(scope, work, first_position, sender);
        });
        // 写库：按块序。后面的块先判完就在这里等，前面那一块到了一起写
        let mut waiting: BTreeMap<usize, GpuJudgedBlock> = BTreeMap::new();
        let mut next_to_store = 0usize;
        for message in receiver {
            match message {
                CardMessage::Finished(card_outcome) => outcome.cards.push(card_outcome),
                CardMessage::Refused(reason) => {
                    refusal.get_or_insert(reason);
                }
                CardMessage::Block(block) => {
                    waiting.insert(block.pending_index, block);
                    while let Some(block) = waiting.remove(&next_to_store) {
                        if storing_error.is_some() {
                            continue;
                        }
                        let storing_started = Instant::now();
                        let (point_index, local) = &pending[block.pending_index];
                        match store_judged_block(
                            store,
                            &plan.points[*point_index],
                            local,
                            version,
                            &block.items_of_each_state,
                            &mut tally,
                        ) {
                            Ok(()) => {
                                outcome.blocks_judged += 1;
                                outcome.states_judged += local.end - local.start;
                                next_to_store += 1;
                            }
                            Err(error) => {
                                storing_error = Some(format!("写判定块失败：{error:?}"));
                                work.stop.store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                        outcome.seconds_storing += storing_started.elapsed().as_secs_f64();
                    }
                }
            }
        }
        if storing_error.is_none() && refusal.is_none() {
            // 领走的块（序号连着、从 0 起）每一块都写进了库；没被叫停时领走的就是全部
            let taken = work
                .next_pending
                .load(std::sync::atomic::Ordering::SeqCst)
                .min(pending.len());
            assert_eq!(next_to_store, taken, "领走的块每一块都写进了库");
            assert!(
                next_to_store == pending.len() || stop_is_requested(),
                "没人叫停，却有 {} 块没判",
                pending.len() - next_to_store
            );
        }
    });
    if let Some(error) = storing_error {
        return Err(error);
    }
    if let Some(reason) = refusal {
        return Err(reason);
    }
    outcome.cards.sort_by_key(|card| {
        cards
            .iter()
            .position(|candidate| candidate.index == card.index)
    });
    outcome.cards_used = outcome.cards.len();
    outcome.compile_seconds = outcome.cards.iter().map(|card| card.build_seconds).sum();
    outcome.judge_seconds = (started.elapsed().as_secs_f64() - outcome.compile_seconds).max(0.0);
    outcome.red_states = tally.red_states;
    eprintln!(
        "CRASH_AMPLIFICATION_GPU_TOTAL cards_used={} blocks={} states={} compile_seconds={:.1} judge_seconds={:.1} states_per_second={:.0} seconds_storing={:.1}",
        outcome.cards_used,
        outcome.blocks_judged,
        outcome.states_judged,
        outcome.compile_seconds,
        outcome.judge_seconds,
        outcome.states_per_second(),
        outcome.seconds_storing
    );
    Ok((tally, outcome))
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
    let facts_digest = facts.digest();
    store
        .store_flow_facts(&facts_digest, &facts.to_bytes())
        .map_err(|error| format!("事实表写不进库：{error:?}"))?;
    let verifier_digest = on.verifier_digest();
    let verifier = BlockVerifier::new(on, &facts);
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
            let bits = verifier.verify(&facts, ordinals);
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

    /// 配置的值去掉一层引号；键没写是 `None`；显卡优先级从配置文件读，文件不在才读环境。
    #[test]
    fn the_card_priority_is_read_from_the_configuration_file_with_one_layer_of_quotes_removed() {
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
    #[test]
    fn a_library_under_a_directory_a_reboot_erases_is_refused_unless_declared_temporary() {
        std::env::remove_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY");
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
        std::env::set_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY", "1");
        assert_eq!(
            refuse_a_library_that_a_reboot_would_erase(Path::new("/tmp/x")),
            Ok(()),
            "写明一次性的放行"
        );
        std::env::remove_var("SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY");
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

    #[test]
    fn environment_switches_are_read_only_when_the_configuration_file_is_absent() {
        let directory = std::env::temp_dir().join(format!(
            "singlefs-run-mode-environment-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("建得了临时目录");
        let file = directory.join("multi-host.env");
        std::fs::write(&file, "ENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\n").expect("写得了");
        std::env::set_var("ENABLE_ACROSS_MACHINES", "1");
        std::env::set_var("ENABLE_GPU", "\"1\"");
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
        std::env::remove_var("ENABLE_ACROSS_MACHINES");
        std::env::remove_var("ENABLE_GPU");
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
    /// 这一趟被叫停了（[`stop_is_requested_by_the_environment`]）：判过的块在库里，第 ②③ 段没做，对账不为空是正常的。
    pub stopped_early: bool,
    /// 这一趟判定行上的判法版本。
    pub judge_version: JudgeVersion,
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

/// 从环境跑一趟：配置定跑法，`SINGLEFS_CRASH_AMPLIFICATION_SHARE` 定这台机器领哪一份，`SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 是库目录
/// （没设建在临时目录、跑完删；设了就要落在开机不清空的地方——`/tmp`、`/var/tmp`、`/dev/shm`、`/run` 之下一律拒绝，
/// 只有测试自己知道库是一次性的时才写 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1` 放行），
/// `SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 是别的机器拷回来的库目录（`:` 分隔），核之前先导入；同一条命令再跑一次就是续跑：
/// 这一判法版本下判过的块全部复用，只补没判的；
/// 跑法带 GPU 时卡按配置的优先级用（`SINGLEFS_GPU_CARDS_CONFIG` 指的显卡配置里的 `GPU_CARDS_BY_PRIORITY`，[`parse_cards_by_priority`]；
/// 这个进程在哪台机器看 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`）：任务先装第一张卡，装到它的显存额度为止，装不下的才给下一张；没配就按本机能用的卡的
/// 空闲显存从大到小用（`gpu` 特性没编进来而配置写了 `ENABLE_GPU=1` 就报错，不悄悄退回 CPU）。
/// `SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1` 只排派活计划：打一行 `CRASH_AMPLIFICATION_PLACEMENT …`（要判多少、每张卡装多少、
/// 这台机器的卡试判几次派活量到的速度），一块都不判。
/// 主机之间等分（`SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=<w0>,<w1>,…` 可以改成按权重）。
///
/// # Errors
/// 环境变量写法错、库打不开、判定存储读写失败、配置要 GPU 而这份二进制没有。
#[allow(
    clippy::too_many_lines,
    reason = "读环境、开库、导入、录入、按卡分工核对、对账、打行，是一趟的全部步骤，拆开要把十几个量来回传"
)]
pub fn run_from_environment(
    flow: &CrashFlow<'_>,
    judging: &PipelineJudgingCode,
    repository_root: &Path,
) -> Result<EnvironmentRun, String> {
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
    let mut imported_blocks = 0u64;
    if let Ok(list) = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_IMPORT") {
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
    let seconds_planning_and_recording = planning_started.elapsed().as_secs_f64();
    let cards = if mode.uses_gpu() {
        Some(gpu_cards_in_use(cards_by_priority.as_deref(), host)?)
    } else {
        None
    };
    if placement_only {
        let Some(cards) = &cards else {
            return Err("只排派活计划要配置开着 GPU（ENABLE_GPU=1）".to_string());
        };
        print_the_placement(
            &store,
            flow,
            &plan,
            &|block| selected.contains(&block),
            &version,
            cards,
            cards_by_priority.as_deref(),
            host,
        )?;
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
            stopped_early: false,
            judge_version: version,
        });
    }
    // 第 ① 段：配置开了 GPU 就由 GPU 判器判这台机器领的块（判不了就报错，不退回 CPU）；没开 GPU 由 CPU 判器判
    let judging_started = std::time::Instant::now();
    let (checking, gpu_judge) = match &cards {
        Some(cards) => {
            let (checking, outcome) = judge_my_blocks_on_gpu(
                &mut store,
                flow,
                &plan,
                &|block| selected.contains(&block),
                &version,
                cards,
                &stop_is_requested_by_the_environment,
            )?;
            (checking, Some(outcome))
        }
        None => (
            check_selected_blocks(
                &mut store,
                flow,
                &plan,
                &selected,
                &UnitChecksumsOn::CpuOnly,
                &version,
                &stop_is_requested_by_the_environment,
            )
            .map_err(|error| format!("核对失败：{error:?}"))?,
            None,
        ),
    };
    if stop_is_requested_by_the_environment() {
        // 被叫停：判过的块已经在库里，第 ②③ 段留给接着判的那一趟
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
        });
    }
    let seconds_judging = judging_started.elapsed().as_secs_f64();
    // 第 ② 段：配置开了 GPU 就在优先级最高的那张卡上核（第 ① 段的草稿区已经放掉）；没开 GPU 而 SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu 就用 CPU 参照核
    let verifying_started = std::time::Instant::now();
    let (verification, cards_used) = if let Some(cards) = &cards {
        verify_my_blocks_on_the_first_card(&mut store, flow, &plan, &my_blocks, cards)?
    } else if std::env::var("SINGLEFS_CRASH_AMPLIFICATION_VERIFY").is_ok_and(|value| value == "cpu")
    {
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
        )
    } else {
        (None, 0)
    };
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

/// 这个进程用哪几张卡：配置点了名的只用这台机器那几张、照配置的次序与额度；没配就是本机能用的卡按空闲显存从大到小。
#[cfg(feature = "gpu")]
fn gpu_cards_in_use(
    configured: Option<&[CardByPriority]>,
    host: CardHost,
) -> Result<CardsInUse, String> {
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
    Ok(cards.into_iter().map(std::sync::Arc::new).collect())
}

/// 配置要 GPU 而这份二进制没编进 `gpu` 特性：报错，不悄悄退回 CPU。
#[cfg(not(feature = "gpu"))]
fn gpu_cards_in_use(
    _configured: Option<&[CardByPriority]>,
    _host: CardHost,
) -> Result<CardsInUse, String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}

/// 第 ① 段在卡上的入口（`gpu` 特性编进来时调 [`judge_my_blocks_on_gpu_cards`]）。
#[cfg(feature = "gpu")]
fn judge_my_blocks_on_gpu(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    version: &JudgeVersion,
    cards: &CardsInUse,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
    judge_my_blocks_on_gpu_cards(
        store,
        flow,
        plan,
        is_mine,
        version,
        cards,
        stop_is_requested,
    )
}

/// 没编进 `gpu` 特性走不到这里（[`gpu_cards_in_use`] 先报错）。
#[cfg(not(feature = "gpu"))]
fn judge_my_blocks_on_gpu(
    _store: &mut VerdictStore,
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
    _is_mine: &dyn Fn(u64) -> bool,
    _version: &JudgeVersion,
    _cards: &CardsInUse,
    _stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<(CheckingTally, GpuJudgeOutcome), String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}

/// 第 ② 段在卡上：这台机器领到的块都在优先级最高的那张卡上核（第 ① 段的草稿区那时已经放掉，这一段一块一次派活、不占草稿区）。
/// 交回合计与用了几张卡。
#[cfg(feature = "gpu")]
fn verify_my_blocks_on_the_first_card(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    my_blocks: &[u64],
    cards: &CardsInUse,
) -> Result<(Option<VerificationOutcome>, usize), String> {
    let card = cards
        .first()
        .expect("用的卡至少一张（gpu_cards_in_use 判过）")
        .clone();
    let selected: std::collections::HashSet<u64> = my_blocks.iter().copied().collect();
    eprintln!(
        "CRASH_AMPLIFICATION_CARD name={:?} index={} memory_budget_mebibytes={} blocks={}",
        card.name,
        card.index,
        card.memory_budget_mebibytes(),
        selected.len()
    );
    let on = FactsVerifierOn::Gpu(card);
    let tally =
        verify_blocks_from_facts(store, flow, plan, &|block| selected.contains(&block), &on)?;
    Ok((
        Some(VerificationOutcome {
            verifier: "gpu",
            tally,
        }),
        1,
    ))
}

/// 没编进 `gpu` 特性走不到这里（[`gpu_cards_in_use`] 先报错）。
#[cfg(not(feature = "gpu"))]
fn verify_my_blocks_on_the_first_card(
    _store: &mut VerdictStore,
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
    _my_blocks: &[u64],
    _cards: &CardsInUse,
) -> Result<(Option<VerificationOutcome>, usize), String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}

/// 只排派活计划：这一趟要判多少、配置的每张卡按额度带得了多少、按优先级各装多少；这台机器装了状态的卡各试判几次派活量速度。
/// 打一行 `CRASH_AMPLIFICATION_PLACEMENT`。别的机器上的卡带得了多少按它的额度算（绑定上限按本机第一张卡估）。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、流、计划、分工、判法版本、卡、配置、机器，各是各的东西，凑成结构体只为让签名变短"
)]
fn print_the_placement(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    is_mine: &dyn Fn(u64) -> bool,
    version: &JudgeVersion,
    cards: &CardsInUse,
    configured: Option<&[CardByPriority]>,
    host: CardHost,
) -> Result<(), String> {
    use crate::crash_judge_dispatch::{states_in_flight_within, GpuCrashJudge, PackedJudgeTables};
    use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
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
    let tables = match build_judge_tables(&judge_flow) {
        Ok(tables) => PackedJudgeTables::of(&tables),
        Err(refusal) => {
            return Err(format!(
                "GPU 判器建不出这条流的表（{refusal:?}；{} 块、{pending_states} 个状态要判）。配置开了 GPU，不退回 CPU 判器：让判器的表认得这种写再跑",
                pending.len()
            ));
        }
    };
    let local_capacities = states_in_flight_of_each_card(cards, &tables);
    let limit = cards
        .first()
        .expect("用的卡至少一张（gpu_cards_in_use 判过）")
        .device()
        .limits()
        .max_storage_buffer_binding_size;
    // 配置的每张卡一行，按优先级：这台机器的取实际起好的那张（点了名而用不了的带得了 0 个），别的机器的按额度算
    let rows: Vec<(CardHost, u32, u64)> = match configured {
        Some(configured) => configured
            .iter()
            .map(|entry| {
                let capacity = if entry.host == host {
                    cards
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
        None => cards
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
        for ((row_host, index, _), states_placed) in rows.iter().zip(&placed) {
            if *row_host != host || *states_placed == 0 {
                continue;
            }
            let card = cards
                .iter()
                .find(|card| card.index == *index)
                .expect("装了状态的本机卡是起好的");
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
            let seconds = started.elapsed().as_secs_f64();
            if seconds > 0.0 {
                states_per_second_on_this_host += words.len() as f64 / seconds;
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
        "CRASH_AMPLIFICATION_PLACEMENT host={} judge=gpu pending_blocks={} pending_states={pending_states} tables_mebibytes={} cards={} states_placed_on_local={} states_placed_on_peer={} states_per_second_on_this_host={states_per_second_on_this_host:.0} estimated_seconds_on_this_host={:.0}",
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
    Ok(())
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
    _configured: Option<&[CardByPriority]>,
    _host: CardHost,
) -> Result<(), String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}
