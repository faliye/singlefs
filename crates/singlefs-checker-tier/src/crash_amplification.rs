//! 崩溃放量流水线（里程碑二「增补 4」第一、四、十一项；用户 2026-09-28 定的形态）：
//!
//! 1. **崩溃点与路径**：一条测试流程是从 mkfs 出发、一个崩溃点接一个崩溃点走下来的路径；每段归含它最后一次写的那个点，上游不看下游。每个点的身份是
//!    `<仓内路径>::<崩溃点名称>::<哈希串>`（[`crate::crash_identity`]）。几条流程共用前缀，全部点连成一棵树（用户 2026-09-28：「这就是个 B 树的问题」）：
//!    路径编号是从根到这个点、每个点取输入键（点名、写表、父镜像的摘要，不含判法代码）前 16 个十六进制、用 `/` 逐段接起来的串：
//!    前面几段相同就是同一段枝干，一棵子树就是一段连续的键；判法代码改了树不动，只有各点的哈希串（复用键）失效。
//!    按 COW 的性质，一个点的输入（它之前的镜像、它自己带屏障的写表）与判法代码不变，它的判定就不变：同一路径编号、同一哈希串就复用。
//! 2. **先录后核**：录入把每个点的状态按块（点内序号区间）登进库的录入表；核对按批从库里取还没有判定的块，
//!    在 CPU 上（或 CPU + GPU：单元两道校验和上 GPU，与 CPU 对拍）判完、原子地写回判定块与违例；门禁一次读完。
//! 3. **对账**：一轮收尾，录入表里有、判定块表里没有的块就是没核完的（卡掉线、进程被杀），重新分给还在的一方补跑；
//!    领块只在内存里，库里不记谁领了，不会有块被一张死卡占着。
//! 4. **分工与跑法**：块按权重分给几方（一方一台机器的 CPU，或一张卡，权重取空闲显存），各写各的库，最后
//!    [`crate::verdict_store::VerdictStore::import_from`] 并成一个；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use singlefs_harness::memory_pool::{MemoryPool, PublishedVersion, RetainedWrite, WrittenContents};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::sha256_digest;

use crate::crash::{
    judge_layer0_state_range, layer0_state_ranges_by_segment, Layer0SegmentExpansion,
};
use crate::crash_facts::{extract_flow_facts, verify_states_from_facts, FlowFacts};
use crate::crash_identity::{
    judging_code_by_module_references, step_write_table_with_barriers, CoverageReport,
    CoverageState, JudgingCode, JudgingCodeError, NodeLabel, ReuseKey, SectorImage, StepAttributes,
    SHA256_BYTES,
};
use crate::verdict_store::{
    BlockStartStateIndex, EncodedVerdictVector, EnumerationPlanHash, InputFingerprint,
    StateOffsetInBlock, StreamOrNodeName, VerdictBlock, VerdictBlockKey, VerdictStore,
    VerdictStoreError,
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
    pub plan_hash: [u8; SHA256_BYTES],
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
/// 判法代码按点的文件取（[`JudgingCodes`]），不按流的测试文件取。
///
/// # Panics
/// 崩溃点的写区间不是首尾相接地罩住整张写表；某个点的名称或文件路径不合身份的规矩（同一个名字在一条流程里可以出现几次：
/// 同一个函数调几次就是几个点，树上的位置分得开）；
/// 各段的状态序号接不上（枚举计划与切段不一致，是这里的 bug）。
#[must_use]
pub fn plan_crash_points(flow: &CrashFlow<'_>, judging_codes: &JudgingCodes) -> CrashPlan {
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
            judging_code: *judging_codes.digest_of(&point.code_file_in_repository),
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

// ============================== 判法代码：按点的文件取 ==============================

/// 流水线自己的判法入口：每个点的判法闭包都从它的代码文件与这一份出发收——判一个状态的代码（`crash` 的判器、池级 checker、
/// `verdict_store`）从这里够得到，公共模块里的点不引它们也一样收进来。
pub const PIPELINE_JUDGE_ENTRY_FILE: &str =
    "crates/singlefs-checker-tier/src/crash_amplification.rs";

/// 每个崩溃点代码文件一份判法代码。判法按点的文件取、不按流的测试文件取：改流 B 的测试文件，公共模块里的前缀节点复用键不动。
#[derive(Clone, Debug)]
pub struct JudgingCodes {
    code_of_file: BTreeMap<String, JudgingCode>,
}

impl JudgingCodes {
    /// 给这几份文件各算一份（每份从它自己与 [`PIPELINE_JUDGE_ENTRY_FILE`] 出发收）。
    ///
    /// # Errors
    /// 同 [`judging_code_by_module_references`]。
    pub fn of_files(
        repository_root: &Path,
        files: &[&str],
        toolchain: &crate::layer0_progress::Layer0ToolchainIdentity,
    ) -> Result<Self, JudgingCodeError> {
        let mut code_of_file = BTreeMap::new();
        for file in files {
            if code_of_file.contains_key(*file) {
                continue;
            }
            let code = judging_code_by_module_references(
                repository_root,
                &[file, PIPELINE_JUDGE_ENTRY_FILE],
                toolchain,
            )?;
            code_of_file.insert((*file).to_string(), code);
        }
        Ok(Self { code_of_file })
    }

    /// 给一条流里每个点的文件各算一份。
    ///
    /// # Errors
    /// 同 [`Self::of_files`]。
    pub fn of_the_flow(
        repository_root: &Path,
        flow: &CrashFlow<'_>,
        toolchain: &crate::layer0_progress::Layer0ToolchainIdentity,
    ) -> Result<Self, JudgingCodeError> {
        let files: Vec<&str> = flow
            .crash_points
            .iter()
            .map(|point| point.code_file_in_repository.as_str())
            .collect();
        Self::of_files(repository_root, &files, toolchain)
    }

    /// # Panics
    /// 这份文件没算过：流里有个点的文件不在建这份表时给的文件里。
    #[must_use]
    pub fn digest_of(&self, file: &str) -> &[u8; SHA256_BYTES] {
        &self.code_of(file).digest
    }

    /// # Panics
    /// 同 [`Self::digest_of`]。
    #[must_use]
    pub fn code_of(&self, file: &str) -> &JudgingCode {
        self.code_of_file.get(file).unwrap_or_else(|| {
            panic!("崩溃点文件 {file} 没算过判法代码：建 JudgingCodes 时要把流里每个点的文件都给上")
        })
    }

    /// 换掉一份文件的摘要（测试里模拟「这份文件的判法代码改了」）。
    ///
    /// # Panics
    /// 同 [`Self::digest_of`]。
    pub fn replace_digest(&mut self, file: &str, digest: [u8; SHA256_BYTES]) {
        self.code_of_file
            .get_mut(file)
            .unwrap_or_else(|| panic!("崩溃点文件 {file} 没算过判法代码"))
            .digest = digest;
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

// ============================== 录入 ==============================

/// 录入的结果：这一趟录进库的块数、库里已有判定（复用）的块数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordingTally {
    pub blocks_recorded: u64,
    pub blocks_already_judged: u64,
    pub states_recorded: u64,
    pub states_already_judged: u64,
}

/// 录入：每个点的每一块登进录入表，路径 → 身份登进路径索引表；库里已经有同一块键的判定块（同一身份下的同一块）就不再核。
///
/// # Errors
/// 库读写失败。
pub fn record_crash_points(
    store: &mut VerdictStore,
    plan: &CrashPlan,
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
            if store.read_verdict_block(&key)?.is_some() {
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

/// 核对录入表里还没有判定的块：按分工只核 `owner` 那一方分到的；块之间多线程（[`checking_threads`]，第 k 块归第 k mod 线程数 个线程），
/// 每块在 CPU 上逐状态判（`unit_checksums` 给了卡就把每个状态镜像里被写过的单元两道校验和交 GPU 算、与 CPU 逐个对拍），
/// 判完按块的次序单线程写回判定块与违例——判定向量的编号按块序登记，与线程数无关，输出与单线程逐字相同。
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
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where(
        store,
        flow,
        plan,
        &|block_sequence| shares.owner_of_block(block_sequence) == owner,
        unit_checksums,
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
) -> Result<CheckingTally, VerdictStoreError> {
    let mut pending: Vec<(usize, Range<u64>)> = Vec::new();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_mine(this_block) {
                continue;
            }
            if store
                .read_verdict_block(&point.block_key(&local))?
                .is_some()
            {
                continue;
            }
            pending.push((point_index, local));
        }
    }
    // 块是库与 GPU 批的粒度，不改小；块少线程多时把一块的序号再切成几段，一段一个线程，写库前按段序拼回一块。
    let threads = checking_threads();
    // 每块切成线程数那么多段：块少时也能把线程吃满（一块 65536 个状态、32 线程时一段 2048 个）。
    let pieces_per_block = threads;
    let mut work: Vec<(usize, usize, Range<u64>)> = Vec::new();
    for (pending_index, (_, local)) in pending.iter().enumerate() {
        let length = local.end - local.start;
        let piece_length = length
            .div_ceil(u64::try_from(pieces_per_block).expect("段数装得进 u64"))
            .max(1);
        let mut start = local.start;
        let mut piece = 0usize;
        while start < local.end {
            let end = (start + piece_length).min(local.end);
            work.push((pending_index, piece, start..end));
            start = end;
            piece += 1;
        }
    }
    let threads = threads.min(work.len()).max(1);
    let mut judged: Vec<(usize, usize, JudgedBlock)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|thread_index| {
                let work = &work;
                let pending = &pending;
                scope.spawn(move || {
                    work.iter()
                        .enumerate()
                        .filter(|(work_index, _)| work_index % threads == thread_index)
                        .map(|(_, (pending_index, piece, range))| {
                            let (point_index, _) = &pending[*pending_index];
                            (
                                *pending_index,
                                *piece,
                                judge_one_block(
                                    flow,
                                    plan,
                                    *point_index,
                                    range.clone(),
                                    unit_checksums,
                                ),
                            )
                        })
                        .collect::<Vec<(usize, usize, JudgedBlock)>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("核对线程没有 panic"))
            .collect()
    });
    judged.sort_by_key(|(pending_index, piece, _)| (*pending_index, *piece));
    let mut tally = CheckingTally::default();
    let mut judged = judged.into_iter().peekable();
    for (pending_index, (point_index, local)) in pending.iter().enumerate() {
        let point = &plan.points[*point_index];
        let mut items_of_each_state: Vec<Vec<String>> =
            Vec::with_capacity(usize::try_from(local.end - local.start).expect("一块装得进 usize"));
        let mut next_expected = local.start;
        while let Some((index, _, _)) = judged.peek() {
            if *index != pending_index {
                break;
            }
            let (_, _, piece) = judged.next().expect("刚 peek 到");
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
        store.store_verdict_block(&point.block_key(local), &verdict_block, &violating)?;
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
    pub fn verifier_digest(&self) -> [u8; SHA256_BYTES] {
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
    verifier_digest: &[u8; SHA256_BYTES],
) -> Result<JudgeVerifierComparison, VerdictStoreError> {
    let mut comparison = JudgeVerifierComparison::default();
    for point in &plan.points {
        for local in point.blocks() {
            let key = point.block_key(&local);
            let judged = store.read_verdict_block(&key)?;
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

/// 录了、还没有判定块的块（卡掉线、进程被杀留下的）：一轮收尾按它重新分给活着的一方补跑。
///
/// # Errors
/// 库读失败。
pub fn unjudged_blocks(
    store: &VerdictStore,
) -> Result<Vec<(VerdictBlockKey, u64)>, VerdictStoreError> {
    let judged: std::collections::HashSet<VerdictBlockKey> =
        store.stored_verdict_block_keys()?.into_iter().collect();
    Ok(store
        .recorded_blocks()?
        .into_iter()
        .filter(|(key, _)| !judged.contains(key))
        .collect())
}

/// 库里的一个红状态：节点名、点内序号、判定向量的逐项文字。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredRedState {
    pub path: String,
    pub identity: String,
    pub state_in_point: u64,
    pub items: Vec<String>,
}

/// 门禁一次读：这些点每一块的违例，按点、按序号交回。
///
/// # Errors
/// 库读失败。
pub fn read_violations(
    store: &VerdictStore,
    plan: &CrashPlan,
) -> Result<Vec<StoredRedState>, VerdictStoreError> {
    let mut found = Vec::new();
    for point in &plan.points {
        for local in point.blocks() {
            let key = point.block_key(&local);
            for violation in store.violations_in_block(&key)? {
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
                judging_code: [0; 32],
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

/// 按环境跑一趟的结果：录了多少、核了多少、对账剩多少、违例几条。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentRun {
    pub mode: CrashAmplificationRunMode,
    pub share: HostShare,
    pub cards_used: usize,
    pub recording: RecordingTally,
    pub checking: CheckingTally,
    pub imported_blocks: u64,
    pub unjudged_blocks: usize,
    pub violations: Vec<StoredRedState>,
    pub coverage_lines: Vec<String>,
    pub library_directory: PathBuf,
    /// 第 ② 段跑了就有：在哪算、核了多少。
    pub verification: Option<VerificationOutcome>,
    /// 第 ③ 段：核对表与判定块逐状态比的结果。
    pub comparison: Option<JudgeVerifierComparison>,
}

/// 只核给定序号的那些块（块序号 = 全部点的块按次序编号）；[`check_recorded_blocks`] 是它按权重分工的形态。
///
/// # Errors
/// 同 [`check_recorded_blocks`]。
pub fn check_selected_blocks(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    selected_block_sequences: &std::collections::HashSet<u64>,
    unit_checksums: &UnitChecksumsOn,
) -> Result<CheckingTally, VerdictStoreError> {
    check_blocks_where(
        store,
        flow,
        plan,
        &|block_sequence| selected_block_sequences.contains(&block_sequence),
        unit_checksums,
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
/// （没设建在临时目录、跑完删），`SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 是别的机器拷回来的库目录（`:` 分隔），核之前先导入；
/// 跑法带 GPU 时按本机能用的卡的空闲显存再分一层，每张卡各核一份（`gpu` 特性没编进来而配置写了 `ENABLE_GPU=1` 就报错，不悄悄退回 CPU）。
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
    judging_codes: &JudgingCodes,
    repository_root: &Path,
) -> Result<EnvironmentRun, String> {
    let mode = match std::env::var("SINGLEFS_MULTI_HOST_CONFIG") {
        Ok(text) => {
            let configuration = path_relative_to_repository_root(repository_root, &text);
            if !configuration.is_file() {
                return Err(format!(
                    "SINGLEFS_MULTI_HOST_CONFIG 指的配置不在：{}",
                    configuration.display()
                ));
            }
            CrashAmplificationRunMode::from_configuration_file(&configuration)
        }
        Err(_) => CrashAmplificationRunMode::from_configuration_file_or_environment(
            &repository_root.join("multi-host.env"),
        ),
    };
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
    let plan = plan_crash_points(flow, judging_codes);
    let mut report = CoverageReport::default();
    let recording = record_crash_points(&mut store, &plan, &mut report)
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
    // 第 ① 段：CPU 判器判这台机器领的块、写判定块（GPU 不再逐状态插在这里）
    let selected: std::collections::HashSet<u64> = my_blocks.iter().copied().collect();
    let checking = check_selected_blocks(
        &mut store,
        flow,
        &plan,
        &selected,
        &UnitChecksumsOn::CpuOnly,
    )
    .map_err(|error| format!("核对失败：{error:?}"))?;
    // 第 ② 段：配置开了 GPU 就在本机的卡上核（按空闲显存分块）；没开 GPU 而 SINGLEFS_CRASH_AMPLIFICATION_VERIFY=cpu 就用 CPU 参照核
    let (verification, cards_used) = if mode.uses_gpu() {
        verify_my_blocks_on_gpu_cards(&mut store, flow, &plan, &my_blocks)?
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
    // 第 ③ 段：核过就比
    let comparison = match &verification {
        Some(outcome) => {
            let digest = match outcome.verifier {
                "cpu" => FactsVerifierOn::Cpu.verifier_digest(),
                #[cfg(feature = "gpu")]
                "gpu" => crate::crash_verify_gpu::gpu_verifier_digest(),
                other => return Err(format!("认不出的核对方式 {other}")),
            };
            Some(
                compare_judge_and_verifier(&store, &plan, &digest)
                    .map_err(|error| format!("第 ③ 段比对失败：{error:?}"))?,
            )
        }
        None => None,
    };
    let unjudged = unjudged_blocks(&store)
        .map_err(|error| format!("对账失败：{error:?}"))?
        .len();
    let violations =
        read_violations(&store, &plan).map_err(|error| format!("读违例失败：{error:?}"))?;
    let coverage_lines = report.lines().to_vec();
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
    })
}

/// 第 ② 段在本机的卡上：这台机器领到的块按各卡的空闲显存再分一层，每张卡各核一份（`ENABLE_GPU=1`）。交回合计与用了几张卡。
#[cfg(feature = "gpu")]
fn verify_my_blocks_on_gpu_cards(
    store: &mut VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    my_blocks: &[u64],
) -> Result<(Option<VerificationOutcome>, usize), String> {
    let cards = crate::gpu_unit_checks::usable_gpu_cards();
    if cards.is_empty() {
        return Err("配置写了 ENABLE_GPU=1，本机却没有一张空闲显存够的 NVIDIA 独显".to_string());
    }
    let card_shares = WeightedShares::new(
        cards
            .iter()
            .map(|card| card.free_memory_mebibytes)
            .collect(),
    );
    let mut total = VerificationTally::default();
    let mut cards_used = 0usize;
    for (card_index, card) in cards.into_iter().enumerate() {
        let selected: std::collections::HashSet<u64> = my_blocks
            .iter()
            .enumerate()
            .filter(|(rank, _)| {
                card_shares.owner_of_block(u64::try_from(*rank).expect("块序装得进 u64"))
                    == card_index
            })
            .map(|(_, block)| *block)
            .collect();
        eprintln!(
            "CRASH_AMPLIFICATION_CARD name={:?} free_mebibytes={} blocks={}",
            card.name,
            card.free_memory_mebibytes,
            selected.len()
        );
        let on = FactsVerifierOn::Gpu(std::sync::Arc::new(card));
        let tally =
            verify_blocks_from_facts(store, flow, plan, &|block| selected.contains(&block), &on)?;
        total.blocks_verified += tally.blocks_verified;
        total.blocks_already_verified += tally.blocks_already_verified;
        total.states_verified += tally.states_verified;
        total.verifier_red_states += tally.verifier_red_states;
        cards_used += 1;
    }
    Ok((
        Some(VerificationOutcome {
            verifier: "gpu",
            tally: total,
        }),
        cards_used,
    ))
}

/// 配置要 GPU 而这份二进制没编进 `gpu` 特性：报错，不悄悄退回 CPU。
#[cfg(not(feature = "gpu"))]
fn verify_my_blocks_on_gpu_cards(
    _store: &mut VerdictStore,
    _flow: &CrashFlow<'_>,
    _plan: &CrashPlan,
    _my_blocks: &[u64],
) -> Result<(Option<VerificationOutcome>, usize), String> {
    Err("配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（cargo … --features verdict-store,gpu）".to_string())
}
