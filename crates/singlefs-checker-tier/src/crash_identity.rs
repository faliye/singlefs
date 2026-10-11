//! 崩溃放量节点的身份与复用键（里程碑二「增补 4」第一项；第三轮判决 `research/prompts/m3-prune-gpu-r3-main-verification.md`
//! 第四节的设计表，用户 2026-09-28 认了身份形态；E164（崩溃放量新设计的小范围功能测试） 第一、二段量过各件）。
//!
//! - 身份 = `<设这个崩溃点的文件的仓内路径>::<崩溃点名称>::<哈希串>`（用户 2026-09-28 定）：一个函数里设了几个崩溃点就是几个身份，
//!   名称绑在设它的函数上；前两段是标签，只给覆盖报告；哈希串是复用键。
//! - 复用键罩三样属性：这一步带屏障、FUA、盘、偏移、段号的整张写表；这一步之前的镜像（父输出，扇区终值）；判法代码
//!   （从节点的测试文件出发、顺着 `use` 与路径引用只收用到的模块，外加给这些类型写 `impl` 的同 crate 文件；自己原文被
//!   `include_str!` 读的文件取原文摘要，其余去注释、去位置取词法摘要），连同 `Cargo.lock` 与工具链。
//! - 身份只由仓内相对路径与内容算出，不含绝对路径、机器与用户：同一份代码在两台机器、两个目录下算出同一个身份。
//! - 标签类型不给格式化、不给比较（没有 `Display`、`Debug`、`Clone`、`Hash`、`PartialEq`、`Ord`），不能拿去当键（E164 F12）。
//! - 三种结论各一把版本（崩溃点重放合并第三轮判决 J8）：GPU 结论（GPU 判器闭包）、CPU 结论（CPU 判器闭包 ∪ 设点文件出发的闭包）、
//!   观察者结果（观察者代码出发的闭包 + 另读输入的摘要），闭包都在停止表（判器的 ∪ 流水线五份）处停。闭包算法在零依赖的
//!   `judging_closure.rs`，`build.rs` 照同一套在编译时算代码摘要表写进二进制，跑时从盘上再算，不等由调用方整趟不写库、退 78。

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite, WrittenContents};
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::layer0_progress::Layer0ToolchainIdentity;

const SECTOR_BYTES: u64 = 512;

// ============================== 标签与复用键 ==============================

/// 崩溃点标签：设这个点的文件的仓内路径（登记时冻结）、崩溃点名称。只能交给 [`CoverageReport`]，不能格式化、不能比较。
pub struct NodeLabel {
    code_file_in_repository: String,
    crash_point_name: String,
}

/// 标签不合规矩：仓内路径是绝对路径、带 `.` 或 `..` 段、为空；崩溃点名称为空。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeLabelRejected {
    pub reason: String,
}

impl NodeLabel {
    /// # Errors
    /// 仓内路径是绝对路径、带 `..` 段或为空；崩溃点名称为空。身份要能跨机器，路径只收仓内相对路径。
    pub fn new(
        code_file_in_repository: &str,
        crash_point_name: &str,
    ) -> Result<Self, NodeLabelRejected> {
        let rejected = |reason: &str| {
            Err(NodeLabelRejected {
                reason: reason.to_string(),
            })
        };
        if code_file_in_repository.is_empty() || code_file_in_repository.starts_with('/') {
            return rejected("代码文件名要写仓内相对路径");
        }
        if code_file_in_repository
            .split('/')
            .any(|part| part == ".." || part == ".")
        {
            return rejected("仓内路径里不许有 . 或 .. 段");
        }
        if crash_point_name.is_empty() {
            return rejected("崩溃点名称不许为空");
        }
        Ok(Self {
            code_file_in_repository: code_file_in_repository.to_string(),
            crash_point_name: crash_point_name.to_string(),
        })
    }

    /// 身份串：标签两段加复用键的十六进制。只在本 crate 里拼（写进判定存储的节点名、覆盖报告），标签本身不外露文字。
    pub(crate) fn identity_text(&self, reuse_key: &ReuseKey) -> String {
        format!(
            "{}::{}::{}",
            self.code_file_in_repository,
            self.crash_point_name,
            hexadecimal_text(&reuse_key.0)
        )
    }
}

/// 复用键：只能从属性算出（[`StepAttributes::reuse_key`]）。这一步的输入没变就是同一个键；判法变不变另按判法版本在判定行上分。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReuseKey([u8; DIGEST_BYTES]);

impl ReuseKey {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }
}

/// 一步的两样属性（各是 SHA-256）：这一步写下的字节（带屏障的写表）与它起步的镜像。只有流程进复用键：
/// 判法是判定行的属性（`verdict_store::JudgeVersion`），不在这里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepAttributes {
    pub write_table_with_barriers: [u8; DIGEST_BYTES],
    pub parent_output: [u8; DIGEST_BYTES],
}

impl StepAttributes {
    /// 两样按固定次序、各带名字拼起来取 SHA-256。
    #[must_use]
    pub fn reuse_key(&self) -> ReuseKey {
        let mut message = Vec::new();
        for (name, digest) in [
            ("write_table_with_barriers", &self.write_table_with_barriers),
            ("parent_output", &self.parent_output),
        ] {
            message.extend_from_slice(name.as_bytes());
            message.push(b'=');
            message.extend_from_slice(digest);
            message.push(b'\n');
        }
        ReuseKey(sha256_digest(&message))
    }
}

/// 一个崩溃点在这一趟里的去向：核了、复用了库里的判定、按测试代码里的 ignore 跳过了（分叉控制，用户 2026-09-28 定「预设的 ignore 就是剪枝」）、
/// 它的段按展开上限一个都没展开（没有状态可核，也是一种剪枝）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageState {
    Checked,
    Reused,
    Ignored,
    NotExpanded,
}

impl CoverageState {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Checked => "checked",
            Self::Reused => "reused",
            Self::Ignored => "ignored",
            Self::NotExpanded => "not_expanded",
        }
    }
}

/// 覆盖报告：只写。收下路径、标签与它这一趟是复用还是重跑，交回逐行 `<路径> <身份> <去向>`（给人看、给门禁数）。
#[derive(Default)]
pub struct CoverageReport {
    lines: Vec<String>,
}

impl CoverageReport {
    pub fn record(
        &mut self,
        path: &str,
        label: NodeLabel,
        reuse_key: &ReuseKey,
        state: CoverageState,
    ) {
        self.lines.push(format!(
            "{path} {} {}",
            label.identity_text(reuse_key),
            state.name()
        ));
    }

    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

// ============================== 录入的属性：写表与父输出 ==============================

/// 一次写的内容在写表哈希里的写法：普通写取字节的 SHA-256，整段清零只写长度。崩溃放量算判器另读的输入时
/// （`crash_amplification` 的计划哈希）也照这一份写，两处一个写法。
pub(crate) fn content_digest_text(contents: &WrittenContents) -> String {
    match contents {
        WrittenContents::Bytes(bytes) => hexadecimal_text(&sha256_digest(bytes)),
        WrittenContents::Zeros { length } => format!("zeros:{length}"),
    }
}

/// 一步的写表哈希（带屏障）：这一步的写按写表次序逐条，每条记盘、种类、FUA、偏移、长度、内容摘要、这次写在这一步里落在第几段。
/// 段由调用方按 D13（验证路线） 已定项 4 切好（`segments` 是整条流的段，写表下标）：屏障挪了、少了，段号就变，哈希跟着变
/// （E164 F1：只按字节算会漏掉去掉一道屏障的改动）。
///
/// # Panics
/// `step_writes` 里有写不属于任何段。
#[must_use]
pub fn step_write_table_with_barriers(
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    step_writes: std::ops::Range<usize>,
) -> [u8; DIGEST_BYTES] {
    let mut segment_of_write: BTreeMap<usize, usize> = BTreeMap::new();
    for (segment_index, segment) in segments.iter().enumerate() {
        for write_index in segment {
            segment_of_write.insert(*write_index, segment_index);
        }
    }
    let first_segment_of_step = step_writes
        .clone()
        .filter_map(|write_index| segment_of_write.get(&write_index).copied())
        .min()
        .unwrap_or(0);
    let mut text = String::new();
    for write_index in step_writes {
        let write = &writes[write_index];
        let segment = segment_of_write
            .get(&write_index)
            .unwrap_or_else(|| panic!("写 {write_index} 不属于任何段"));
        text.push_str(&format!(
            "W {} {} {} {} {} {} {}\n",
            write.device.0,
            write.kind.name(),
            u8::from(write.is_force_unit_access),
            write.offset.0,
            write.length_in_bytes(),
            content_digest_text(&write.contents),
            segment - first_segment_of_step
        ));
    }
    sha256_digest(text.as_bytes())
}

/// 扇区终值表：（盘，扇区号）→ 那个扇区最后留下的内容摘要。父输出 = 基镜像上叠这一步之前全部写之后的这张表。
#[derive(Clone, Default)]
pub struct SectorImage {
    sectors: BTreeMap<(u32, u64), [u8; DIGEST_BYTES]>,
}

impl SectorImage {
    /// 基镜像（mkfs 之后）里写过的每个扇区。
    #[must_use]
    pub fn of_base(base: &MemoryPool) -> Self {
        let mut sectors = BTreeMap::new();
        for (device, image) in &base.devices {
            for (sector, bytes) in image.written_sectors() {
                sectors.insert((device.0, *sector), sha256_digest(bytes));
            }
        }
        Self { sectors }
    }

    /// 按写表次序叠几次写。
    pub fn overlay(&mut self, writes: &[RetainedWrite]) {
        let sector_bytes = usize::try_from(SECTOR_BYTES).expect("512");
        for write in writes {
            let first_sector = write.offset.0 / SECTOR_BYTES;
            match &write.contents {
                WrittenContents::Bytes(bytes) => {
                    for (sector_offset, chunk) in bytes.chunks(sector_bytes).enumerate() {
                        let sector = first_sector
                            + u64::try_from(sector_offset).expect("扇区序号装得进 u64");
                        self.sectors
                            .insert((write.device.0, sector), sha256_digest(chunk));
                    }
                }
                WrittenContents::Zeros { length } => {
                    let zero_sector = sha256_digest(&vec![0u8; sector_bytes]);
                    for sector_offset in 0..length.div_ceil(SECTOR_BYTES) {
                        self.sectors
                            .insert((write.device.0, first_sector + sector_offset), zero_sector);
                    }
                }
            }
        }
    }

    /// 按（盘，扇区）升序每行 `<盘> <扇区> <摘要>` 拼起来取 SHA-256。
    #[must_use]
    pub fn digest(&self) -> [u8; DIGEST_BYTES] {
        let mut text = String::new();
        for ((device, sector), digest) in &self.sectors {
            text.push_str(&format!("{device} {sector} {}\n", hexadecimal_text(digest)));
        }
        sha256_digest(text.as_bytes())
    }
}

// ============================== 核对的属性：判法代码摘要（按模块引用取） ==============================

// 闭包算法住在零依赖的 `judging_closure.rs`（只用 std）：`build.rs` 经 `#[path]` 引同一份，在编译时算三种结论的代码摘要。
// 它用这一层引进来的 `sha256_digest`、`DIGEST_BYTES` 与 `hexadecimal_text` 算摘要。
#[path = "judging_closure.rs"]
mod judging_closure;

pub use judging_closure::{
    conclusion_closure_stop_files, lexical_text, lexical_tokens, ClosureToolchainLines,
    ClosureTreeReader, ConclusionCode, ConclusionCodeDigestTable,
    ConclusionCodeDigestTableUnreadable, ConclusionCodeEntry, FileIntroducer, JudgingCode,
    JudgingCodeError, CONCLUSION_CANDIDATE_DIRECTORY, CONCLUSION_CODE_DIGEST_TABLE_FILE_NAME,
    CPU_JUDGE_ENTRY_FILE, FILES_THE_JUDGES_MUST_NOT_REACH, GPU_JUDGE_ENTRY_FILE,
    JUDGE_CLOSURE_STOP_FILES, JUDGE_ENTRY_FILES, PIPELINE_FILES_THE_CONCLUSION_CLOSURES_STOP_AT,
};
// ============================== 登记的崩溃枚举用例：随机历史脚手架经谁进判法闭包 ==============================

/// 随机历史脚手架（生成器、执行器、取样与复现入口，连同它带进来的理想模型、对拍、场景、故障注入）。登记的崩溃枚举用例的判法闭包里
/// 有了它，改随机历史就让那一格整格重判（E164 W5）。
pub const RANDOM_HISTORY_SCAFFOLDING_FILE: &str = "crates/singlefs-harness/src/history.rs";

/// 一条登记的崩溃枚举用例许经一份库模块引随机历史脚手架（用户 2026-10-02 定「按登记用例白名单」）：只给拿随机历史当输入的用例登记，
/// 理由写明它的判定为什么本来就随随机历史变。
pub struct RandomHistoryThroughALibraryModuleAllowed {
    /// `.claude/gate.d/stage-inputs.tsv` 里 `crash-case:` 行 `test=` 列的测试目标。
    pub test_target: &'static str,
    /// 引随机历史脚手架的那份库模块（仓内路径，`crates/<包>/src/` 下）。
    pub library_module: &'static str,
    pub reason: &'static str,
}

/// 登记表：哪条用例许经哪份库模块引随机历史脚手架。没登记的库模块把它带进登记的崩溃枚举用例，判法闭包那一核就拦
/// （[`random_history_scaffolding_introducers_judged`]）；登记了而那条用例的闭包里没有这一引的是死条目，同一核也拦。
pub const RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED: &[RandomHistoryThroughALibraryModuleAllowed] = &[
    RandomHistoryThroughALibraryModuleAllowed {
        test_target: "crash_injection_campaign",
        library_module: "crates/singlefs-checker-tier/src/crash_injection.rs",
        reason: "崩溃注入拿随机历史当输入：战役按种子生成随机历史、执行时注入崩溃、照理想模型判恢复落到的版本，判定本来就随随机历史的生成器与执行器变",
    },
    RandomHistoryThroughALibraryModuleAllowed {
        test_target: "crash_injection_campaign",
        library_module: "crates/singlefs-harness/src/model_comparison.rs",
        reason: "崩溃注入拿随机历史当输入：战役经 crash_injection.rs 用它把崩溃之后读回的结果对到理想模型；它只在自己的单元测试模块里引随机历史脚手架（词法闭包照收），判定随对拍的写法变",
    },
    RandomHistoryThroughALibraryModuleAllowed {
        test_target: "crash_injection_campaign",
        library_module: "crates/singlefs-harness/src/reference_filesystem_translation.rs",
        reason: "崩溃注入拿随机历史当输入：战役经 crash_injection.rs 用 RefFS 翻译层把历史操作、每步结局与崩溃时刻翻成 RefFS 操作再判恢复读出（形式化接入线 12-2），翻译层直接引随机历史脚手架的操作与观察类型，判定随历史怎么生成、怎么执行变",
    },
    RandomHistoryThroughALibraryModuleAllowed {
        test_target: "crash_injection_campaign",
        library_module: "crates/singlefs-harness/src/fault_injection.rs",
        reason: "崩溃注入拿随机历史当输入：RefFS 翻译层把历史操作翻成 RefFS 操作时要认每步的故障结局（形式化接入线 12-2），经它引故障注入的结局类型，故障注入又引随机历史脚手架的执行器；判定随带故障执行历史的写法变",
    },
    RandomHistoryThroughALibraryModuleAllowed {
        test_target: "crash_injection_read_faulted_states_never_hold_the_witness_criterion",
        library_module: "crates/singlefs-checker-tier/src/crash_injection.rs",
        reason: "读故障那一项拿随机历史当输入：按种子生成、带读故障执行随机历史，经 crash_injection.rs 取 mkfs 之后的基线与写表（HistoryAfterTheMakeFilesystem）交给定状态表判，判定本来就随随机历史与它的建表变",
    },
    RandomHistoryThroughALibraryModuleAllowed {
        test_target: "crash_injection_read_faulted_states_never_hold_the_witness_criterion",
        library_module: "crates/singlefs-harness/src/model_comparison.rs",
        reason: "读故障那一项拿随机历史当输入：经 crash_injection.rs 按文件收进闭包（崩溃注入用它对理想模型）；它只在自己的单元测试模块里引随机历史脚手架（词法闭包照收）",
    },
    RandomHistoryThroughALibraryModuleAllowed {
        test_target: "crash_injection_read_faulted_states_never_hold_the_witness_criterion",
        library_module: "crates/singlefs-harness/src/reference_filesystem_translation.rs",
        reason: "读故障那一项拿随机历史当输入：经 crash_injection.rs 按文件收进闭包（崩溃注入用 RefFS 翻译层判恢复读出），翻译层直接引随机历史脚手架的操作与观察类型",
    },
    RandomHistoryThroughALibraryModuleAllowed {
        test_target: "crash_injection_read_faulted_states_never_hold_the_witness_criterion",
        library_module: "crates/singlefs-harness/src/fault_injection.rs",
        reason: "读故障那一项拿随机历史当输入：带读故障执行历史，经 crash_injection.rs 与 RefFS 翻译层收进故障注入的结局类型，故障注入又引随机历史脚手架的执行器",
    },
];

/// 随机历史脚手架的一个引入者判下来是什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomHistoryIntroducerVerdict {
    /// 它就是测试目标的入口。
    EntryOfTheTestTarget,
    /// 测试目标自己的文件（`tests/` 下，连同经 `#[path]` 共用的搭建模块）直接引：放行。
    TestTargetFile,
    /// 库模块引，这条用例登记过它：放行，理由在登记表里。
    RegisteredLibraryModule { reason: &'static str },
    /// 库模块引，这条用例没登记它：拦。
    UnregisteredLibraryModule,
    /// `impl` 头点名了闭包里声明的名字把它带进来：拦，登记不了。
    BroughtInByAnImplHeader,
}

impl RandomHistoryIntroducerVerdict {
    #[must_use]
    pub fn is_let_through(&self) -> bool {
        match self {
            Self::EntryOfTheTestTarget
            | Self::TestTargetFile
            | Self::RegisteredLibraryModule { .. } => true,
            Self::UnregisteredLibraryModule | Self::BroughtInByAnImplHeader => false,
        }
    }
}

/// 随机历史脚手架的一个引入者与它的判定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgedRandomHistoryIntroducer {
    pub introducer: FileIntroducer,
    pub verdict: RandomHistoryIntroducerVerdict,
}

/// 仓内路径是不是某个包的库模块（`crates/<包>/src/` 下）；不是的是测试目标自己的文件（`crates/<包>/tests/` 下）。
fn is_a_library_module(file: &str) -> bool {
    let mut parts = file.split('/');
    parts.next() == Some("crates") && parts.next().is_some() && parts.next() == Some("src")
}

/// 逐份判一条登记的崩溃枚举用例里随机历史脚手架的引入者（用户 2026-10-02 定「逐份判」）。闭包从
/// `crates/singlefs-checker-tier/tests/<test_target>.rs` 出发按模块引用收、到 [`RANDOM_HISTORY_SCAFFOLDING_FILE`] 停：
/// 停在门外那一份的每一个引入者，都是不经随机历史脚手架自己就进了闭包、又引它的文件。只因随机历史脚手架才进闭包的模块
/// （它自己带进来的故障注入、对拍）回引它，不算把它带进来。交回每个引入者与判定（按引入者排序）；闭包够不着随机历史脚手架时是空的。
///
/// # Errors
/// 同 [`judging_code_by_module_references_stopping_at`]。
pub fn random_history_scaffolding_introducers_judged(
    repository_root: &Path,
    test_target: &str,
    toolchain: &Layer0ToolchainIdentity,
    allowed: &[RandomHistoryThroughALibraryModuleAllowed],
) -> Result<Vec<JudgedRandomHistoryIntroducer>, JudgingCodeError> {
    let entry = format!("crates/singlefs-checker-tier/tests/{test_target}.rs");
    let closure_stopped_at_the_scaffolding = judging_code_by_module_references_stopping_at(
        repository_root,
        &[entry.as_str()],
        &[RANDOM_HISTORY_SCAFFOLDING_FILE],
        toolchain,
    )?;
    let Some(introducers) = closure_stopped_at_the_scaffolding
        .introducers_of_each_file
        .get(RANDOM_HISTORY_SCAFFOLDING_FILE)
    else {
        return Ok(Vec::new());
    };
    let verdict_on_an_introducing_file = |file: &str| {
        if !is_a_library_module(file) {
            return RandomHistoryIntroducerVerdict::TestTargetFile;
        }
        match allowed
            .iter()
            .find(|row| row.test_target == test_target && row.library_module == file)
        {
            Some(row) => {
                RandomHistoryIntroducerVerdict::RegisteredLibraryModule { reason: row.reason }
            }
            None => RandomHistoryIntroducerVerdict::UnregisteredLibraryModule,
        }
    };
    Ok(introducers
        .iter()
        .map(|introducer| JudgedRandomHistoryIntroducer {
            introducer: introducer.clone(),
            verdict: match introducer {
                FileIntroducer::Entry => RandomHistoryIntroducerVerdict::EntryOfTheTestTarget,
                FileIntroducer::ReferencedBy { file }
                | FileIntroducer::ReadAsRawTextBy { file } => verdict_on_an_introducing_file(file),
                FileIntroducer::ImplHeaderNaming { .. } => {
                    RandomHistoryIntroducerVerdict::BroughtInByAnImplHeader
                }
            },
        })
        .collect())
}

fn closure_toolchain_lines(toolchain: &Layer0ToolchainIdentity) -> ClosureToolchainLines<'_> {
    ClosureToolchainLines {
        rustc_version_lines: &toolchain.rustc_version_lines,
        cargo_version: &toolchain.cargo_version,
        target_triple: &toolchain.target_triple,
    }
}

/// 判器的判法代码：从 [`JUDGE_ENTRY_FILES`] 出发、到 [`JUDGE_CLOSURE_STOP_FILES`] 停的模块引用闭包。
///
/// # Errors
/// 同 [`judging_code_by_module_references_stopping_at`]。
pub fn judging_code_of_the_judges(
    repository_root: &Path,
    toolchain: &Layer0ToolchainIdentity,
) -> Result<JudgingCode, JudgingCodeError> {
    let stop_files: Vec<&str> = JUDGE_CLOSURE_STOP_FILES
        .iter()
        .map(|(file, _reason)| *file)
        .collect();
    judging_code_by_module_references_stopping_at(
        repository_root,
        JUDGE_ENTRY_FILES,
        &stop_files,
        toolchain,
    )
}

/// 同 [`judging_code_by_module_references_stopping_at`]，不设停止文件。
///
/// # Errors
/// 同 [`judging_code_by_module_references_stopping_at`]。
pub fn judging_code_by_module_references(
    repository_root: &Path,
    entry_files: &[&str],
    toolchain: &Layer0ToolchainIdentity,
) -> Result<JudgingCode, JudgingCodeError> {
    judging_code_by_module_references_stopping_at(repository_root, entry_files, &[], toolchain)
}

/// 判法代码：从 `entry_files`（仓内路径）出发，顺着路径与 `use` 引用收文件；再收同 crate 里 `impl` 头点名了
/// 已收文件里所声明类型或 trait 的文件，反复到收不到新文件为止；`stop_files` 里的文件被引到就停在门外，不收、不往下收，记进 `stopped_at`。
/// 自己原文被 `include_str!` / `include_bytes!` 读的文件取原文摘要，其余取词法单元串。摘要另拼 `Cargo.lock`、收到的各 crate 的
/// `Cargo.toml`（原文）与工具链（算法在 [`ClosureTreeReader::judging_code_stopping_at`]）。
///
/// # Errors
/// 读不了某份文件、工作区的 crate 清单读不出来。
pub fn judging_code_by_module_references_stopping_at(
    repository_root: &Path,
    entry_files: &[&str],
    stop_files: &[&str],
    toolchain: &Layer0ToolchainIdentity,
) -> Result<JudgingCode, JudgingCodeError> {
    ClosureTreeReader::new(repository_root)?.judging_code_stopping_at(
        entry_files,
        stop_files,
        &closure_toolchain_lines(toolchain),
    )
}

// ============================== 三种结论各一把版本（崩溃点重放合并第三轮判决 J8） ==============================

/// 观察者的名字：非空，不超过 `u16::MAX` 字节（判定库的键里用两个字节记它的长度）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObserverName(String);

/// 给的观察者名字是空的，或超过 `u16::MAX` 字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObserverNameRejected {
    pub text: String,
}

impl ObserverName {
    /// # Errors
    /// 空，或超过 `u16::MAX` 字节。
    pub fn new(text: &str) -> Result<Self, ObserverNameRejected> {
        if text.is_empty() || u16::try_from(text.len()).is_err() {
            return Err(ObserverNameRejected {
                text: text.to_string(),
            });
        }
        Ok(Self(text.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 一块存的三种结论（第三轮判决 J8）：GPU 判器判镜像的结论、CPU 判器判镜像的结论、一个观察者的结果。判定库按（块键，结论种类，版本）
/// 存与记账，哪一种没有就只排那一种。驱动版本与卡型是块的属性（判定库的块属性表），不进种类、不进版本。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConclusionKind {
    GpuJudge,
    CpuJudge,
    Observer { observer_name: ObserverName },
}

impl ConclusionKind {
    /// 版本消息里的种类那一行：`gpu_judge`、`cpu_judge`、`observer:<名字>`。
    #[must_use]
    pub fn version_text(&self) -> String {
        match self {
            Self::GpuJudge => "gpu_judge".to_string(),
            Self::CpuJudge => "cpu_judge".to_string(),
            Self::Observer { observer_name } => format!("observer:{}", observer_name.as_str()),
        }
    }
}

/// 一种结论的版本（SHA-256）：种类、这种结论的代码摘要（[`ConclusionCodeEntry`] 出发的闭包）、工具链，观察者另加它另读的输入的摘要。
/// 同一块在几把版本下的结论并存，复用只认同一版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConclusionVersion(pub [u8; DIGEST_BYTES]);

/// 要哪一把版本。
/// - GPU 结论：GPU 判器闭包。
/// - CPU 结论：CPU 判器闭包 ∪ 设这个点的文件与它 `mod` 进来的测试模块出发的闭包，都在停止表（判器的 ∪ 流水线五份）处停。
/// - 观察者结果：观察者代码出发的闭包，加观察者另读的流程输入的摘要（第三轮判决 J4；由调用方算好给）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConclusionVersionRequest<'request> {
    GpuJudge,
    CpuJudge {
        setting_file: &'request str,
    },
    Observer {
        observer_name: &'request ObserverName,
        observer_code_file: &'request str,
        extra_inputs_digest: [u8; DIGEST_BYTES],
    },
}

/// 版本消息的头一行：消息的写法换了就换这一行。
const CONCLUSION_VERSION_HEADER: &str = "singlefs conclusion version 1";

impl ConclusionVersionRequest<'_> {
    #[must_use]
    pub fn kind(&self) -> ConclusionKind {
        match self {
            Self::GpuJudge => ConclusionKind::GpuJudge,
            Self::CpuJudge { .. } => ConclusionKind::CpuJudge,
            Self::Observer { observer_name, .. } => ConclusionKind::Observer {
                observer_name: (*observer_name).clone(),
            },
        }
    }

    /// 这一把版本的代码摘要按哪个入口算。
    #[must_use]
    pub fn code_entry(&self) -> ConclusionCodeEntry {
        match self {
            Self::GpuJudge => ConclusionCodeEntry::GpuJudge,
            Self::CpuJudge { setting_file } => ConclusionCodeEntry::CpuJudgeWithSettingFile {
                setting_file: (*setting_file).to_string(),
            },
            Self::Observer {
                observer_code_file, ..
            } => ConclusionCodeEntry::ObserverCode {
                observer_code_file: (*observer_code_file).to_string(),
            },
        }
    }

    /// 代码摘要拼上种类、工具链（与观察者另读的输入）取 SHA-256。
    fn version_of_code(
        &self,
        code_digest: &[u8; DIGEST_BYTES],
        toolchain: &Layer0ToolchainIdentity,
    ) -> ConclusionVersion {
        let extra_inputs = match self {
            Self::GpuJudge | Self::CpuJudge { .. } => "-".to_string(),
            Self::Observer {
                extra_inputs_digest,
                ..
            } => hexadecimal_text(extra_inputs_digest),
        };
        let message = format!(
            "{CONCLUSION_VERSION_HEADER}\nkind={}\ncode={}\nrustc={}\ncargo={}\ntarget={}\nextra_inputs={extra_inputs}\n",
            self.kind().version_text(),
            hexadecimal_text(code_digest),
            toolchain.rustc_version_lines,
            toolchain.cargo_version,
            toolchain.target_triple,
        );
        ConclusionVersion(sha256_digest(message.as_bytes()))
    }
}

/// `build.rs` 在编这个 crate 的那棵树上算的结论代码摘要表（文件名是 [`CONCLUSION_CODE_DIGEST_TABLE_FILE_NAME`]）。
const COMPILED_CONCLUSION_CODE_DIGEST_TABLE_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/conclusion_code_digests.tsv"));

/// 编译时算的结论代码摘要表。
///
/// # Panics
/// 写进二进制的表认不回来：它是 `build.rs` 用 [`ConclusionCodeDigestTable::to_table_text`] 写的，认不回来是两边写法不一致的 bug。
#[must_use]
pub fn compiled_conclusion_code_digest_table() -> ConclusionCodeDigestTable {
    ConclusionCodeDigestTable::from_table_text(COMPILED_CONCLUSION_CODE_DIGEST_TABLE_TEXT)
        .unwrap_or_else(|unreadable| {
            panic!("build.rs 写进二进制的结论代码摘要表认不回来（两边写法不一致）：{unreadable:?}")
        })
}

/// 从盘上这棵树现算整张结论代码摘要表（与 `build.rs` 同一个算法）。
///
/// # Errors
/// 读不了某份文件、候选目录或工作区的 crate 清单。
pub fn conclusion_code_digest_table_from_the_tree(
    repository_root: &Path,
) -> Result<ConclusionCodeDigestTable, JudgingCodeError> {
    ConclusionCodeDigestTable::of_the_tree(&mut ClosureTreeReader::new(repository_root)?)
}

/// 从盘上这棵树现算一把版本。
///
/// # Errors
/// 读不了某份文件、工作区的 crate 清单读不出来。
pub fn conclusion_version_from_the_tree(
    repository_root: &Path,
    request: &ConclusionVersionRequest<'_>,
    toolchain: &Layer0ToolchainIdentity,
) -> Result<ConclusionVersion, JudgingCodeError> {
    let code = ClosureTreeReader::new(repository_root)?
        .conclusion_code(&request.code_entry().entry_files())?;
    Ok(request.version_of_code(&code.digest, toolchain))
}

/// 编译时那张表里的这一把版本；这个入口编译时没算（不在 [`CONCLUSION_CANDIDATE_DIRECTORY`] 下）交回 None。
#[must_use]
pub fn conclusion_version_compiled_in(
    request: &ConclusionVersionRequest<'_>,
    toolchain: &Layer0ToolchainIdentity,
) -> Option<ConclusionVersion> {
    compiled_conclusion_code_digest_table()
        .digest_of_each_entry
        .get(&request.code_entry())
        .map(|code_digest| request.version_of_code(code_digest, toolchain))
}

/// 编译时与跑时比一把版本的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConclusionVersionCheck {
    /// 相同：这一把版本可以写库。
    SameAsTheBuild { version: ConclusionVersion },
    /// 不等：二进制不是从盘上这棵树编的（编完又改了代码、拿别的树编的）。调用方整趟不写库、退 78（第二轮判决 L3-1）。
    DiffersFromTheBuild {
        compiled_in: ConclusionVersion,
        from_the_tree: ConclusionVersion,
    },
    /// 这个入口编译时没算：比不了，调用方照「不等」处理。
    NotComputedAtTheBuild {
        code_entry: ConclusionCodeEntry,
        from_the_tree: ConclusionVersion,
    },
}

impl ConclusionVersionCheck {
    /// 能写库的版本：只有与编译时相同才有；交回 None 时调用方整趟不写库、退 78。
    #[must_use]
    pub fn version_to_write_under(&self) -> Option<ConclusionVersion> {
        match self {
            Self::SameAsTheBuild { version } => Some(*version),
            Self::DiffersFromTheBuild { .. } | Self::NotComputedAtTheBuild { .. } => None,
        }
    }
}

/// 一把版本编译时算的与从盘上现算的比。
///
/// # Errors
/// 同 [`conclusion_version_from_the_tree`]。
pub fn conclusion_version_checked_against_the_build(
    repository_root: &Path,
    request: &ConclusionVersionRequest<'_>,
    toolchain: &Layer0ToolchainIdentity,
) -> Result<ConclusionVersionCheck, JudgingCodeError> {
    let from_the_tree = conclusion_version_from_the_tree(repository_root, request, toolchain)?;
    Ok(match conclusion_version_compiled_in(request, toolchain) {
        Some(compiled_in) if compiled_in == from_the_tree => {
            ConclusionVersionCheck::SameAsTheBuild {
                version: from_the_tree,
            }
        }
        Some(compiled_in) => ConclusionVersionCheck::DiffersFromTheBuild {
            compiled_in,
            from_the_tree,
        },
        None => ConclusionVersionCheck::NotComputedAtTheBuild {
            code_entry: request.code_entry(),
            from_the_tree,
        },
    })
}

/// 整张表编译时与跑时比：交回两边摘要不同、或只有一边有的入口（按入口排）；空的就是二进制与盘上这棵树相同。
/// 引擎开跑前调一次，不空就整趟不写库、退 78。
///
/// # Errors
/// 同 [`conclusion_code_digest_table_from_the_tree`]。
pub fn conclusion_code_entries_differing_from_the_build(
    repository_root: &Path,
) -> Result<Vec<ConclusionCodeEntry>, JudgingCodeError> {
    let compiled_in = compiled_conclusion_code_digest_table().digest_of_each_entry;
    let from_the_tree =
        conclusion_code_digest_table_from_the_tree(repository_root)?.digest_of_each_entry;
    let every_entry: BTreeSet<&ConclusionCodeEntry> =
        compiled_in.keys().chain(from_the_tree.keys()).collect();
    Ok(every_entry
        .into_iter()
        .filter(|entry| compiled_in.get(*entry) != from_the_tree.get(*entry))
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repository_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../Cargo.lock")
            .canonicalize()
            .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
            .parent()
            .expect("Cargo.lock 所在的目录就是仓根")
            .to_path_buf()
    }

    #[test]
    fn lexical_digest_matches_the_e164_anchors() {
        let digest =
            |source: &str| hexadecimal_text(&sha256_digest(lexical_text(source).as_bytes()));
        let anchor = "e7587d8d4f76653ed394fefe683bb459d4976354ee1366bf987fcd16750a418c";
        assert_eq!(
            digest("const A: u8 = 1;"),
            anchor,
            "E164 跑前登记 7.2 第一行"
        );
        assert_eq!(
            digest("/// doc\nconst A: u8 = 1; // x"),
            anchor,
            "注释与文档注释不进摘要"
        );
        assert_eq!(
            digest("const A: u8 = 2;"),
            "1869717063fbdfadfa523427dd8bd33001d9070e3bab056a405409a4adb11eae",
            "字面量改了摘要就变"
        );
        assert_eq!(
            digest("// only\n/* a /* nested */ */"),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "只有注释的文件是空串"
        );
    }

    #[test]
    fn node_label_takes_only_relative_paths_inside_the_repository() {
        assert!(
            NodeLabel::new("/home/x/crates/a.rs", "t").is_err(),
            "绝对路径不收：身份要能跨机器"
        );
        assert!(NodeLabel::new("crates/../a.rs", "t").is_err());
        let label = NodeLabel::new("crates/singlefs-checker-tier/tests/a.rs", "m::t")
            .expect("仓内相对路径");
        let key = StepAttributes {
            write_table_with_barriers: [1; 32],
            parent_output: [2; 32],
        }
        .reuse_key();
        let text = label.identity_text(&key);
        assert!(
            text.starts_with("crates/singlefs-checker-tier/tests/a.rs::m::t::"),
            "身份三段、不带步号：{text}"
        );
        assert_eq!(
            text.len(),
            "crates/singlefs-checker-tier/tests/a.rs::m::t::".len() + 64
        );
    }

    #[test]
    fn each_attribute_changes_the_reuse_key() {
        let base = StepAttributes {
            write_table_with_barriers: [1; 32],
            parent_output: [2; 32],
        };
        let key = base.reuse_key();
        assert_ne!(
            StepAttributes {
                write_table_with_barriers: [9; 32],
                ..base
            }
            .reuse_key(),
            key
        );
        assert_ne!(
            StepAttributes {
                parent_output: [9; 32],
                ..base
            }
            .reuse_key(),
            key
        );
        assert_eq!(base.reuse_key(), key, "同样的属性同一个键");
    }

    fn copy_tree(source: &Path, destination: &Path) {
        std::fs::create_dir_all(destination).expect("建得了目标目录");
        for entry in std::fs::read_dir(source).expect("读得了源目录").flatten() {
            let target = destination.join(entry.file_name());
            if entry.file_type().expect("读得了类型").is_dir() {
                if entry.file_name() != "target" {
                    copy_tree(&entry.path(), &target);
                }
            } else {
                std::fs::copy(entry.path(), &target).expect("拷得了文件");
            }
        }
    }

    /// 判器的判法闭包：两个判器、它们调的恢复（连同恢复调的取前缀核 `singlefs-replay-prefix`）与池级 checker、格式常量、内存池里的状态拼装、五份内核原文都在；流水线（录入、存储、
    /// 身份、对账）、进度模块、设点的测试文件、实现的写路径、装置二进制都不在。停止表里每一条都指到现存文件、都真的拦到了（没拦到的是死条目）。
    #[test]
    fn the_judging_closure_of_the_judges_holds_recovery_the_checker_and_the_kernels_and_stops_before_the_pipeline(
    ) {
        let root = repository_root();
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let code = judging_code_of_the_judges(&root, &toolchain).expect("算得出");
        println!(
            "JUDGING_CODE entry=judges files={} raw={} stopped={} digest={}",
            code.files.len(),
            code.raw_files.len(),
            code.stopped_at.len(),
            hexadecimal_text(&code.digest)
        );
        for file in &code.files {
            println!(
                "JUDGING_CODE_FILE entry=judges file={file} reason={}",
                code.reason_of_each_file[file]
            );
        }
        for file in &code.stopped_at {
            println!("JUDGING_CODE_STOPPED entry=judges file={file}");
        }
        let has = |path: &str| code.files.iter().any(|file| file == path);
        for required in [
            "crates/singlefs-checker-tier/src/crash.rs",
            "crates/singlefs-checker-tier/src/crash_judge_gpu.rs",
            "crates/singlefs-checker-tier/src/crash_judge_conclusions.rs",
            "crates/singlefs-checker-tier/src/crash_judge_tables.rs",
            "crates/singlefs-checker-tier/src/crash_facts.rs",
            "crates/singlefs-checker/src/walk.rs",
            "crates/singlefs-checker/src/image.rs",
            "crates/singlefs-core/src/recovery.rs",
            "crates/singlefs-replay-prefix/src/lib.rs",
            "crates/singlefs-format/src/lib.rs",
            "crates/singlefs-harness/src/memory_pool.rs",
        ] {
            assert!(has(required), "判器的闭包要含 {required}");
        }
        for kernel in [
            "crash_judge_common.wgsl",
            "crash_judge_recovery.wgsl",
            "crash_judge_checker_common.wgsl",
            "crash_judge_checker.wgsl",
            "crash_judge_checker_scan.wgsl",
        ] {
            let path = format!("crates/singlefs-checker-tier/src/{kernel}");
            assert!(has(&path), "内核 {path} 要在判器的闭包里");
            assert!(code.raw_files.contains(&path), "内核 {path} 要按原文进摘要");
        }
        for excluded in [
            "crates/singlefs-checker-tier/src/crash_amplification.rs",
            "crates/singlefs-checker-tier/src/verdict_store.rs",
            "crates/singlefs-checker-tier/src/crash_identity.rs",
            "crates/singlefs-checker-tier/src/layer0_progress.rs",
            "crates/singlefs-checker-tier/src/crash_verify_gpu.rs",
            "crates/singlefs-checker-tier/src/lib.rs",
            "crates/singlefs-core/src/lib.rs",
        ] {
            assert!(!has(excluded), "{excluded} 不判对错，不该在判器的闭包里");
        }
        let outside: Vec<&String> = code
            .files
            .iter()
            .filter(|file| file.contains("/tests/") || file.contains("/src/bin/"))
            .collect();
        assert!(
            outside.is_empty(),
            "测试文件与装置二进制不判对错，不该在判器的闭包里：{outside:?}"
        );
        for (file, reason) in FILES_THE_JUDGES_MUST_NOT_REACH {
            assert!(
                root.join(file).is_file(),
                "不许引的表指到不存在的文件 {file}（{reason}）"
            );
            assert!(
                !has(file) && !code.stopped_at.contains(*file),
                "判器引到了 {file}：它不该在判法闭包里（{reason}）"
            );
        }
        for (stop_file, reason) in JUDGE_CLOSURE_STOP_FILES {
            assert!(
                root.join(stop_file).is_file(),
                "停止表指到不存在的文件 {stop_file}（{reason}）"
            );
            assert!(
                code.stopped_at.contains(*stop_file),
                "停止表里的 {stop_file} 没被判器引到：死条目，删掉它"
            );
        }
    }

    /// 设崩溃点的七份文件（`tests/` 下调录制器 `step`、`record_span` 或公共模块 `one_crash_point_over_every_write` 的那几份）。
    const SETTING_FILES: &[&str] = &[
        "crates/singlefs-checker-tier/tests/common_crash_points/mod.rs",
        "crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs",
        "crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs",
        "crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs",
        "crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs",
        "crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs",
        "crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs",
    ];

    /// 判定只随判器变：在仓副本里给流水线五份、身份、核对、crate 根各加一条真语句，今天的判法摘要不动、三种结论的版本也一把都不动；
    /// 给 CPU 判器、GPU 判器、它的输入表、恢复、取前缀核、池级 checker 走树各加一条，今天的判法摘要变；一份内核原文改一个字节也变。
    /// 三种结论各一把版本（第三轮判决 J8）：改只在 GPU 判器闭包里的文件只换 GPU 版本；改一份设点文件只换它那把 CPU 版本（GPU 版本、
    /// 另一份设点文件的 CPU 版本、观察者版本不动）；改观察者代码只换观察者版本；CPU 与 GPU 两边都收的文件（恢复、走树）两把都换。
    #[test]
    fn the_judging_digest_ignores_the_pipeline_and_the_progress_module_but_follows_every_judge() {
        let root = repository_root();
        let copy = std::env::temp_dir().join(format!(
            "singlefs-judging-closure-copy-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&copy);
        copy_tree(&root.join("crates"), &copy.join("crates"));
        std::fs::copy(root.join("Cargo.lock"), copy.join("Cargo.lock")).expect("拷得了 Cargo.lock");
        const OBSERVER_CODE_FILE: &str =
            "crates/singlefs-checker-tier/tests/observer_code_probe_of_the_judging_contract.rs";
        std::fs::write(
            copy.join(OBSERVER_CODE_FILE),
            "pub fn observed_states_probe(states: u64) -> u64 {\n    states\n}\n",
        )
        .expect("副本里写得了观察者代码");
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let observer_name = ObserverName::new("judging_contract_probe").expect("名字非空、不长");
        let requests = [
            ConclusionVersionRequest::GpuJudge,
            ConclusionVersionRequest::CpuJudge {
                setting_file: SETTING_FILES[0],
            },
            ConclusionVersionRequest::CpuJudge {
                setting_file: SETTING_FILES[1],
            },
            ConclusionVersionRequest::Observer {
                observer_name: &observer_name,
                observer_code_file: OBSERVER_CODE_FILE,
                extra_inputs_digest: [7; DIGEST_BYTES],
            },
        ];
        const REQUEST_NAMES: [&str; 4] = [
            "gpu",
            "cpu_common_crash_points",
            "cpu_multi_record",
            "observer",
        ];
        let digest_and_versions = || {
            let digest = judging_code_of_the_judges(&copy, &toolchain)
                .expect("副本算得出")
                .digest;
            let versions: Vec<ConclusionVersion> = requests
                .iter()
                .map(|request| {
                    conclusion_version_from_the_tree(&copy, request, &toolchain)
                        .expect("副本算得出")
                })
                .collect();
            (digest, versions)
        };
        let (baseline_digest, baseline_versions) = digest_and_versions();
        let after_appending = |file: &str, appended: &str| {
            let path = copy.join(file);
            let original = std::fs::read(&path).expect("读得了副本里的文件");
            let mut edited = original.clone();
            edited.extend_from_slice(appended.as_bytes());
            std::fs::write(&path, &edited).expect("写得了副本");
            let digest_and_versions_after = digest_and_versions();
            std::fs::write(&path, &original).expect("还原得了副本");
            digest_and_versions_after
        };
        // 每份改动之后：今天的判法摘要变不变，四把版本（GPU、公共模块的 CPU、多记录流的 CPU、观察者）各自变不变。
        let expect_changes = |file: &str,
                              appended: &str,
                              judging_digest_changes: bool,
                              versions_change: [bool; 4]| {
            let (digest, versions) = after_appending(file, appended);
            assert_eq!(
                digest != baseline_digest,
                judging_digest_changes,
                "改 {file}：今天的判法摘要{}",
                if judging_digest_changes {
                    "要变"
                } else {
                    "不该变"
                }
            );
            for (index, version_changes) in versions_change.into_iter().enumerate() {
                assert_eq!(
                    versions[index] != baseline_versions[index],
                    version_changes,
                    "改 {file}：{} 那把结论版本{}",
                    REQUEST_NAMES[index],
                    if version_changes {
                        "要换"
                    } else {
                        "不该换"
                    }
                );
            }
        };
        const RUST_PROBE: &str = "\npub const JUDGING_CLOSURE_PROBE: u8 = 1;\n";
        for unchanged in [
            "crates/singlefs-checker-tier/src/crash_amplification.rs",
            "crates/singlefs-checker-tier/src/verdict_store.rs",
            "crates/singlefs-checker-tier/src/layer0_progress.rs",
            "crates/singlefs-checker-tier/src/crash_verify_gpu.rs",
            "crates/singlefs-checker-tier/src/crash_judge_dispatch.rs",
            "crates/singlefs-checker-tier/src/gpu_unit_checks.rs",
            "crates/singlefs-checker-tier/src/lib.rs",
            "crates/singlefs-core/src/lib.rs",
        ] {
            expect_changes(unchanged, RUST_PROBE, false, [false, false, false, false]);
        }
        // 身份（这一份）不在停止表里（主 agent 定，实现 V 报告第八节第 5 条）：引它的设点文件的 CPU 版本照换，不引的不换。
        // 副本上核过谁引它：多记录流、删除流与两流共享前缀那三份设点文件引（`step_write_table_with_barriers`、`SectorImage`、观察者名），
        // 公共模块不引；判器（GPU、CPU 那一半）与观察者代码都不引；今天的判法摘要从两个判器出发，也不收它。
        {
            let identity_file = "crates/singlefs-checker-tier/src/crash_identity.rs";
            let (digest, versions) = after_appending(identity_file, RUST_PROBE);
            assert_eq!(
                digest, baseline_digest,
                "改 {identity_file}：今天的判法摘要不该变（判器不引身份）"
            );
            for (index, version_changes) in [false, false, true, false].into_iter().enumerate() {
                assert_eq!(
                    versions[index] != baseline_versions[index],
                    version_changes,
                    "改 {identity_file}：{} 那把结论版本{}（身份不在停止表；多记录流的设点文件引它，公共模块、判器与观察者代码不引）",
                    REQUEST_NAMES[index],
                    if version_changes { "要换" } else { "不该换" }
                );
            }
        }
        expect_changes(
            SETTING_FILES[0],
            RUST_PROBE,
            false,
            [false, true, true, false],
        );
        expect_changes(
            SETTING_FILES[1],
            RUST_PROBE,
            false,
            [false, false, true, false],
        );
        for only_in_the_gpu_judge in [
            "crates/singlefs-checker-tier/src/crash_judge_gpu.rs",
            "crates/singlefs-checker-tier/src/crash_judge_tables.rs",
        ] {
            expect_changes(
                only_in_the_gpu_judge,
                RUST_PROBE,
                true,
                [true, false, false, false],
            );
        }
        expect_changes(
            "crates/singlefs-checker-tier/src/crash_judge_checker.wgsl",
            "\n// probe\n",
            true,
            [true, false, false, false],
        );
        // 改结论的几处搬进判器一侧（第三轮判决 J8、正文 H8）：红项编码、恢复字与失败点码表、计数向量在 `crash_judge_conclusions.rs`，
        // 两个判器的闭包都收它（CPU 判器经 `Layer0StateJudgement` 的 `impl` 头，GPU 判器 `use` 它），改一个字 GPU 与每份设点文件的 CPU 版本都换；
        // GPU 结论换红项、GPU 恢复字的换法在 `crash_judge_gpu.rs`（上面只换 GPU 那一把的那一组）。
        for in_both_judges in [
            "crates/singlefs-checker-tier/src/crash_judge_conclusions.rs",
            "crates/singlefs-checker-tier/src/crash.rs",
            "crates/singlefs-core/src/recovery.rs",
            "crates/singlefs-replay-prefix/src/lib.rs",
            "crates/singlefs-checker/src/walk.rs",
        ] {
            expect_changes(in_both_judges, RUST_PROBE, true, [true, true, true, false]);
        }
        expect_changes(
            OBSERVER_CODE_FILE,
            RUST_PROBE,
            false,
            [false, false, false, true],
        );
        let other_extra_inputs = ConclusionVersionRequest::Observer {
            observer_name: &observer_name,
            observer_code_file: OBSERVER_CODE_FILE,
            extra_inputs_digest: [8; DIGEST_BYTES],
        };
        assert_ne!(
            conclusion_version_from_the_tree(&copy, &other_extra_inputs, &toolchain)
                .expect("副本算得出"),
            baseline_versions[3],
            "观察者另读的输入换了，观察者版本要换"
        );
        std::fs::remove_dir_all(&copy).expect("删得掉副本");
    }

    /// 从七份设点文件出发的结论闭包都停在流水线五份门外（攻方模型那一行 `crash_amplification.rs=1` 变 0）；只按判器的停止表停时
    /// 五份里不在判器停止表的每一份都被某份设点文件引到（停止表每一条都真的拦到了，不是死条目）；GPU 与 CPU 判器的结论闭包
    /// 停下的只在判器的停止表里（判器自己引不到流水线）。
    #[test]
    fn the_conclusion_closures_from_every_setting_file_stop_before_the_five_pipeline_files() {
        let root = repository_root();
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let mut reader = ClosureTreeReader::new(&root).expect("读得了树");
        let judge_stop_files: Vec<&str> = JUDGE_CLOSURE_STOP_FILES
            .iter()
            .map(|(file, _reason)| *file)
            .collect();
        let mut reached_without_the_pipeline_stops: BTreeSet<String> = BTreeSet::new();
        for setting_file in SETTING_FILES.iter().copied() {
            let code = reader.conclusion_code(&[setting_file]).expect("算得出");
            println!(
                "CONCLUSION_CLOSURE entry={setting_file} files={} stopped={:?}",
                code.files.len(),
                code.stopped_at
            );
            for (pipeline_file, reason) in PIPELINE_FILES_THE_CONCLUSION_CLOSURES_STOP_AT {
                assert!(
                    root.join(pipeline_file).is_file(),
                    "流水线停止表指到不存在的文件 {pipeline_file}（{reason}）"
                );
                assert!(
                    !code.files.contains(*pipeline_file),
                    "从 {setting_file} 出发的结论闭包收了流水线的 {pipeline_file}（{reason}）"
                );
            }
            let without_the_pipeline_stops = judging_code_by_module_references_stopping_at(
                &root,
                &[setting_file],
                &judge_stop_files,
                &toolchain,
            )
            .expect("算得出");
            reached_without_the_pipeline_stops.extend(without_the_pipeline_stops.files);
        }
        for (pipeline_file, reason) in PIPELINE_FILES_THE_CONCLUSION_CLOSURES_STOP_AT {
            assert!(
                judge_stop_files.contains(pipeline_file)
                    || reached_without_the_pipeline_stops.contains(*pipeline_file),
                "流水线停止表里的 {pipeline_file} 不停也没有一份设点文件引到它：死条目，删掉它（{reason}）"
            );
        }
        for judge_entry in [GPU_JUDGE_ENTRY_FILE, CPU_JUDGE_ENTRY_FILE] {
            let code = reader.conclusion_code(&[judge_entry]).expect("算得出");
            let stopped_outside_the_judge_stops: Vec<&String> = code
                .stopped_at
                .iter()
                .filter(|file| !judge_stop_files.contains(&file.as_str()))
                .collect();
            assert!(
                stopped_outside_the_judge_stops.is_empty(),
                "{judge_entry} 的结论闭包引到了流水线（停在判器停止表之外的门口）：{stopped_outside_the_judge_stops:?}"
            );
        }
    }

    /// 编译时 `build.rs` 算的结论代码摘要表，与跑时从盘上这棵树现算的逐行相同；GPU 判器与每份设点文件那一把版本比下来都是「相同」、可以写库。
    #[test]
    fn the_conclusion_code_digest_table_compiled_in_equals_the_one_computed_from_the_tree() {
        let root = repository_root();
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let compiled_in = compiled_conclusion_code_digest_table();
        let from_the_tree = conclusion_code_digest_table_from_the_tree(&root).expect("算得出");
        println!(
            "CONCLUSION_CODE_TABLE rows={} compiled_in_rows={}",
            from_the_tree.digest_of_each_entry.len(),
            compiled_in.digest_of_each_entry.len()
        );
        assert_eq!(
            conclusion_code_entries_differing_from_the_build(&root).expect("算得出"),
            Vec::new(),
            "编译时与跑时的表有对不上的入口：二进制不是从这棵树编的"
        );
        assert_eq!(compiled_in, from_the_tree);
        assert!(
            compiled_in
                .digest_of_each_entry
                .contains_key(&ConclusionCodeEntry::GpuJudge),
            "表里有 GPU 判器那一行"
        );
        let mut requests = vec![ConclusionVersionRequest::GpuJudge];
        requests.extend(
            SETTING_FILES
                .iter()
                .copied()
                .map(|setting_file| ConclusionVersionRequest::CpuJudge { setting_file }),
        );
        for request in &requests {
            let check = conclusion_version_checked_against_the_build(&root, request, &toolchain)
                .expect("算得出");
            assert!(
                matches!(check, ConclusionVersionCheck::SameAsTheBuild { .. }),
                "{request:?}：编译时与跑时要相同：{check:?}"
            );
            assert_eq!(
                check.version_to_write_under(),
                conclusion_version_compiled_in(request, &toolchain),
                "{request:?}：相同时写库用的就是编译时那一把"
            );
        }
    }

    /// 编完之后改了设点文件：这一份的 CPU 版本比下来「不等」、交不出可写库的版本，整张表比下来只点名带它的入口；
    /// 编译时没算的入口（不在候选目录下）比下来「编译时没算」，同样交不出可写库的版本。
    #[test]
    fn a_tree_edited_after_the_build_is_reported_as_differing_and_yields_no_version_to_write_under()
    {
        let root = repository_root();
        let copy = std::env::temp_dir().join(format!(
            "singlefs-conclusion-version-copy-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&copy);
        copy_tree(&root.join("crates"), &copy.join("crates"));
        std::fs::copy(root.join("Cargo.lock"), copy.join("Cargo.lock")).expect("拷得了 Cargo.lock");
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let request = ConclusionVersionRequest::CpuJudge {
            setting_file: SETTING_FILES[1],
        };
        let before = conclusion_version_checked_against_the_build(&copy, &request, &toolchain)
            .expect("副本算得出");
        let compiled_in = conclusion_version_compiled_in(&request, &toolchain).expect("编译时算过");
        let edited_path = copy.join(SETTING_FILES[1]);
        let mut edited = std::fs::read(&edited_path).expect("读得了副本里的设点文件");
        edited.extend_from_slice(b"\npub const SETTING_FILE_EDITED_AFTER_THE_BUILD: u8 = 1;\n");
        std::fs::write(&edited_path, &edited).expect("写得了副本");
        let after = conclusion_version_checked_against_the_build(&copy, &request, &toolchain)
            .expect("副本算得出");
        let differing =
            conclusion_code_entries_differing_from_the_build(&copy).expect("副本算得出");
        let not_compiled_in = conclusion_version_checked_against_the_build(
            &copy,
            &ConclusionVersionRequest::CpuJudge {
                setting_file: "crates/singlefs-harness/tests/common/mod.rs",
            },
            &toolchain,
        )
        .expect("副本算得出");
        std::fs::remove_dir_all(&copy).expect("删得掉副本");
        assert_eq!(
            before,
            ConclusionVersionCheck::SameAsTheBuild {
                version: compiled_in
            },
            "副本没改之前与编译时相同"
        );
        assert!(
            matches!(after, ConclusionVersionCheck::DiffersFromTheBuild { compiled_in: in_the_binary, from_the_tree }
                if in_the_binary == compiled_in && from_the_tree != compiled_in),
            "改了设点文件之后要报不等：{after:?}"
        );
        assert_eq!(
            after.version_to_write_under(),
            None,
            "不等就交不出可写库的版本"
        );
        assert!(
            differing.contains(&ConclusionCodeEntry::CpuJudgeWithSettingFile {
                setting_file: SETTING_FILES[1].to_string()
            }) && !differing.contains(&ConclusionCodeEntry::GpuJudge),
            "整张表只点名带这份设点文件的入口：{differing:?}"
        );
        assert!(
            matches!(
                not_compiled_in,
                ConclusionVersionCheck::NotComputedAtTheBuild { .. }
            ),
            "候选目录外的入口编译时没算：{not_compiled_in:?}"
        );
        assert_eq!(not_compiled_in.version_to_write_under(), None);
    }

    /// 摘要表写成文字再读回逐行相同；表头不对、少一列、摘要不是 64 个小写十六进制字符、同一个入口两行，都读不回。
    #[test]
    fn the_conclusion_code_digest_table_reads_back_its_own_text_and_refuses_malformed_lines() {
        let table = ConclusionCodeDigestTable {
            digest_of_each_entry: BTreeMap::from([
                (ConclusionCodeEntry::GpuJudge, [1; DIGEST_BYTES]),
                (
                    ConclusionCodeEntry::CpuJudgeWithSettingFile {
                        setting_file: "crates/a/tests/b.rs".to_string(),
                    },
                    [2; DIGEST_BYTES],
                ),
                (
                    ConclusionCodeEntry::ObserverCode {
                        observer_code_file: "crates/a/tests/b.rs".to_string(),
                    },
                    [3; DIGEST_BYTES],
                ),
            ]),
        };
        let text = table.to_table_text();
        assert_eq!(
            ConclusionCodeDigestTable::from_table_text(&text),
            Ok(table),
            "自己写的读得回"
        );
        let header = text.lines().next().expect("有表头");
        let gpu_line = format!("gpu_judge\t-\t{}", "01".repeat(DIGEST_BYTES));
        for (malformed, reason) in [
            (format!("# other header\n{gpu_line}\n"), "表头不对"),
            (
                format!("{header}\ngpu_judge\t{}\n", "01".repeat(DIGEST_BYTES)),
                "不是三列",
            ),
            (
                format!("{header}\ngpu_judge\t-\t{}\n", "0G".repeat(DIGEST_BYTES)),
                "摘要不是 64 个小写十六进制字符",
            ),
            (
                format!("{header}\nsomething\t-\t{}\n", "01".repeat(DIGEST_BYTES)),
                "种类认不得",
            ),
            (
                format!("{header}\n{gpu_line}\n{gpu_line}\n"),
                "同一个入口出现两次",
            ),
        ] {
            assert_eq!(
                ConclusionCodeDigestTable::from_table_text(&malformed)
                    .map_err(|unreadable| unreadable.reason),
                Err(reason),
                "{malformed:?}"
            );
        }
    }

    /// 同一段代码摘要、同一套工具链：三种结论的版本两两不同；观察者另读的输入换了，观察者版本换；工具链换了，版本换。
    #[test]
    fn conclusion_versions_of_different_kinds_never_coincide_and_follow_the_toolchain() {
        let code_digest = [5; DIGEST_BYTES];
        let toolchain = Layer0ToolchainIdentity {
            rustc_version_lines: "rustc 0.0.0-probe".to_string(),
            cargo_version: "cargo 0.0.0-probe".to_string(),
            target_triple: "x86_64-probe".to_string(),
        };
        let observer_name = ObserverName::new("probe").expect("名字非空");
        let observer = |extra_inputs_digest| ConclusionVersionRequest::Observer {
            observer_name: &observer_name,
            observer_code_file: "crates/a/tests/b.rs",
            extra_inputs_digest,
        };
        let versions = [
            ConclusionVersionRequest::GpuJudge.version_of_code(&code_digest, &toolchain),
            ConclusionVersionRequest::CpuJudge {
                setting_file: "crates/a/tests/b.rs",
            }
            .version_of_code(&code_digest, &toolchain),
            observer([0; DIGEST_BYTES]).version_of_code(&code_digest, &toolchain),
            observer([1; DIGEST_BYTES]).version_of_code(&code_digest, &toolchain),
        ];
        assert_eq!(
            versions.iter().collect::<BTreeSet<_>>().len(),
            versions.len(),
            "三种结论、两份另读输入，四把版本两两不同：{versions:?}"
        );
        let other_toolchain = Layer0ToolchainIdentity {
            rustc_version_lines: "rustc 0.0.0-probe".to_string(),
            cargo_version: "cargo 0.0.1-probe".to_string(),
            target_triple: "x86_64-probe".to_string(),
        };
        assert_ne!(
            ConclusionVersionRequest::GpuJudge.version_of_code(&code_digest, &other_toolchain),
            versions[0],
            "工具链换了，版本换"
        );
    }

    #[test]
    fn observer_name_takes_one_to_u16_maximum_bytes() {
        assert_eq!(
            ObserverName::new(""),
            Err(ObserverNameRejected {
                text: String::new()
            })
        );
        let longest_name = "n".repeat(usize::from(u16::MAX));
        assert!(
            ObserverName::new(&longest_name).is_ok(),
            "u16::MAX 字节的名字收"
        );
        assert!(
            ObserverName::new(&format!("{longest_name}n")).is_err(),
            "超过 u16::MAX 字节的名字不收"
        );
    }

    /// 抽出之前（闭包算法还在 `crash_identity.rs` 里的那一版）在这个小仓上算出的文件集与摘要（工具链三行写死）。
    /// 小仓罩了路径与 `use` 树、`crate::`、外部 crate 路径、`include_str!` 读的原文、`impl` 头点名的类型、测试目标里的 `mod` 与
    /// `#[path]`、`super::` 落空、注释与文档注释、停止表。
    const SMALL_REPOSITORY_FILES: &[(&str, &str)] = &[
        ("Cargo.lock", "# lock\n"),
        ("crates/singlefs-alpha/Cargo.toml", "[package]\nname = \"singlefs-alpha\"\n"),
        ("crates/singlefs-alpha/src/lib.rs", "pub mod judge;\npub mod kernel_table;\npub mod pipeline;\n"),
        (
            "crates/singlefs-alpha/src/judge.rs",
            "use crate::kernel_table::KernelTable;\nuse singlefs_beta::format::BLOCK_BYTES;\n\
             pub struct Judgement { pub bytes: usize }\n\
             /// 文档注释不进摘要\npub fn judge(table: &KernelTable) -> Judgement {\n    let _ = crate::pipeline::PIPELINE_PROBE; // 行注释不进摘要\n    \
             Judgement { bytes: BLOCK_BYTES + table.words }\n}\n",
        ),
        (
            "crates/singlefs-alpha/src/kernel_table.rs",
            "pub struct KernelTable { pub words: usize }\npub const KERNEL: &str = include_str!(\"kernel.wgsl\");\n",
        ),
        ("crates/singlefs-alpha/src/kernel.wgsl", "struct Words { a: u32 }\n"),
        (
            "crates/singlefs-alpha/src/judgement_display.rs",
            "impl crate::judge::Judgement { pub fn twice(&self) -> usize { self.bytes * 2 } }\n",
        ),
        ("crates/singlefs-alpha/src/pipeline.rs", "pub const PIPELINE_PROBE: u8 = 1;\n"),
        (
            "crates/singlefs-alpha/tests/setting_case.rs",
            "#[path = \"../../singlefs-beta/tests/common/mod.rs\"]\nmod common;\nmod shared_points;\n\
             use singlefs_alpha::judge::judge;\n#[test]\nfn setting_case() { common::probe(); shared_points::point(); }\n",
        ),
        (
            "crates/singlefs-alpha/tests/shared_points/mod.rs",
            "use super::common::probe;\npub fn point() { probe(); }\n",
        ),
        ("crates/singlefs-beta/Cargo.toml", "[package]\nname = \"singlefs-beta\"\n"),
        ("crates/singlefs-beta/src/lib.rs", "pub mod format;\n"),
        ("crates/singlefs-beta/src/format.rs", "pub const BLOCK_BYTES: usize = 4096;\n"),
        ("crates/singlefs-beta/tests/common/mod.rs", "pub fn probe() {}\n"),
    ];

    /// 闭包算法抽成 `judging_closure.rs` 前后，同一棵树上算出的文件集、按原文取的文件、停在门外的文件与摘要逐字相同：
    /// 钉的值是抽出之前那一版在这个小仓上算的（草稿里把旧算法按行抽出来编成程序跑的，真仓上 145 组逐项比也全同）。
    #[test]
    fn the_extracted_closure_algorithm_gives_the_files_and_digests_pinned_before_the_extraction() {
        let root = std::env::temp_dir().join(format!(
            "singlefs-extracted-closure-small-repository-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        for (file, contents) in SMALL_REPOSITORY_FILES {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().expect("文件有上层")).expect("建得了目录");
            std::fs::write(&path, contents).expect("写得了小仓的文件");
        }
        let toolchain = Layer0ToolchainIdentity {
            rustc_version_lines: "rustc 0.0.0-probe\nbinary: rustc\ncommit-hash: probe".to_string(),
            cargo_version: "cargo 0.0.0-probe".to_string(),
            target_triple: "x86_64-probe".to_string(),
        };
        let judge = "crates/singlefs-alpha/src/judge.rs";
        let pipeline = "crates/singlefs-alpha/src/pipeline.rs";
        let setting_case = "crates/singlefs-alpha/tests/setting_case.rs";
        let computed = [
            judging_code_by_module_references_stopping_at(&root, &[judge], &[], &toolchain),
            judging_code_by_module_references_stopping_at(&root, &[judge], &[pipeline], &toolchain),
            judging_code_by_module_references_stopping_at(&root, &[setting_case], &[], &toolchain),
        ];
        std::fs::remove_dir_all(&root).expect("删得掉小仓");
        let pinned: [(Vec<&str>, Vec<&str>, &str); 3] = [
            (
                vec![
                    "crates/singlefs-alpha/src/judge.rs",
                    "crates/singlefs-alpha/src/judgement_display.rs",
                    "crates/singlefs-alpha/src/kernel.wgsl",
                    "crates/singlefs-alpha/src/kernel_table.rs",
                    "crates/singlefs-alpha/src/pipeline.rs",
                    "crates/singlefs-beta/src/format.rs",
                ],
                vec![],
                "d5f6e7546481a1b24236d89a7ec8367ebe8e0dea72d307ddc381edae14f8236d",
            ),
            (
                vec![
                    "crates/singlefs-alpha/src/judge.rs",
                    "crates/singlefs-alpha/src/judgement_display.rs",
                    "crates/singlefs-alpha/src/kernel.wgsl",
                    "crates/singlefs-alpha/src/kernel_table.rs",
                    "crates/singlefs-beta/src/format.rs",
                ],
                vec![pipeline],
                "cbd50ac7da534ea25cce0d105320ae9c2f77a28520e767e0e708ae77815a662e",
            ),
            (
                vec![
                    "crates/singlefs-alpha/src/judge.rs",
                    "crates/singlefs-alpha/src/judgement_display.rs",
                    "crates/singlefs-alpha/src/kernel.wgsl",
                    "crates/singlefs-alpha/src/kernel_table.rs",
                    "crates/singlefs-alpha/src/pipeline.rs",
                    "crates/singlefs-alpha/tests/setting_case.rs",
                    "crates/singlefs-alpha/tests/shared_points/mod.rs",
                    "crates/singlefs-beta/src/format.rs",
                    "crates/singlefs-beta/tests/common/mod.rs",
                ],
                vec![],
                "41e168aac5d580c288eb98c52008761d3060de0156c961da6d162c09ca3f212b",
            ),
        ];
        for (code, (files, stopped_at, digest)) in computed.into_iter().zip(pinned) {
            let code = code.expect("小仓算得出");
            assert_eq!(code.files, files, "收的文件与抽出之前相同");
            assert_eq!(
                code.raw_files,
                BTreeSet::from(["crates/singlefs-alpha/src/kernel.wgsl".to_string()]),
                "按原文取的只有内核那一份"
            );
            assert_eq!(
                code.stopped_at,
                stopped_at
                    .iter()
                    .map(|file| (*file).to_string())
                    .collect::<BTreeSet<String>>(),
                "停在门外的与抽出之前相同"
            );
            assert_eq!(
                hexadecimal_text(&code.digest),
                digest,
                "摘要与抽出之前逐字相同"
            );
        }
    }

    /// 身份要能跨机器、跨目录：同一份代码拷到别的路径，判法代码收的文件与摘要逐字相同。
    #[test]
    fn judging_code_is_the_same_in_a_copy_of_the_repository_at_another_path() {
        let root = repository_root();
        let copy =
            std::env::temp_dir().join(format!("singlefs-identity-copy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&copy);
        copy_tree(&root.join("crates"), &copy.join("crates"));
        std::fs::copy(root.join("Cargo.lock"), copy.join("Cargo.lock")).expect("拷得了 Cargo.lock");
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let entry =
            "crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs";
        let original =
            judging_code_by_module_references(&root, &[entry], &toolchain).expect("主仓算得出");
        let copied =
            judging_code_by_module_references(&copy, &[entry], &toolchain).expect("副本算得出");
        std::fs::remove_dir_all(&copy).expect("删得掉副本");
        assert_eq!(original.files, copied.files, "收的文件逐个相同");
        assert_eq!(original.digest, copied.digest, "摘要相同：身份不含绝对路径");
    }

    /// 崩溃枚举用例登记表在仓里的位置（相对仓根）。
    const CRASH_CASE_REGISTRY: &str = ".claude/gate.d/stage-inputs.tsv";

    /// 从仓根取崩溃枚举用例登记了哪些测试目标：登记表在就逐行取 `crash-case:` 行 `test=` 列的测试目标；
    /// 不在（只导出 `crates/` 的树，广谱变异的导出树就是这样）交回 [`CrashCaseRegistration::RegistryAbsent`]，不猜。
    #[derive(Debug, PartialEq, Eq)]
    enum CrashCaseRegistration {
        Registered { targets: Vec<String> },
        RegistryAbsent { looked_at: PathBuf },
    }

    /// # Panics
    /// 登记表在却读不出、或某一行 `crash-case:` 没有 `test=` 列或 `test=` 列里没有测试目标。
    fn crash_case_registration_under(root: &Path) -> CrashCaseRegistration {
        let registry = root.join(CRASH_CASE_REGISTRY);
        if !registry.is_file() {
            return CrashCaseRegistration::RegistryAbsent {
                looked_at: registry,
            };
        }
        let registrations = std::fs::read_to_string(&registry).expect("登记表在就读得出");
        let targets = registrations
            .lines()
            .filter(|line| line.starts_with("crash-case:"))
            .map(|line| {
                let test_column = line
                    .split('\t')
                    .find_map(|column| column.strip_prefix("test="))
                    .expect("test= 列");
                test_column.split(':').nth(1).expect("测试目标").to_string()
            })
            .collect();
        CrashCaseRegistration::Registered { targets }
    }

    /// 仓里登记表在，取到的是登记的那几条。
    #[test]
    fn crash_case_registration_reads_the_crash_cases_registered_in_the_repository() {
        match crash_case_registration_under(&repository_root()) {
            CrashCaseRegistration::Registered { targets } => assert!(
                targets
                    .iter()
                    .any(|target| target == "crash_enumeration_fixed_script_stream"),
                "仓里的登记表登记了固定脚本那条流（取到的：{targets:?}）"
            ),
            CrashCaseRegistration::RegistryAbsent { looked_at } => {
                panic!("仓里登记表 {} 在，却认成不在", looked_at.display())
            }
        }
    }

    /// 只有 `crates/` 的树（广谱变异的导出树）里登记表不在，认成「不在」，不认成「登记了 0 条」。
    #[test]
    fn crash_case_registration_reports_the_registry_absent_in_a_tree_of_only_crates() {
        let exported =
            std::env::temp_dir().join(format!("singlefs-exported-crates-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&exported);
        std::fs::create_dir_all(exported.join("crates")).expect("建得出导出树");
        let registration = crash_case_registration_under(&exported);
        std::fs::remove_dir_all(&exported).expect("删得掉导出树");
        assert_eq!(
            registration,
            CrashCaseRegistration::RegistryAbsent {
                looked_at: exported.join(CRASH_CASE_REGISTRY)
            },
            "只有 crates/ 的树里登记表不在"
        );
    }

    /// 每条登记的崩溃枚举用例的判法代码：含恢复与 checker 走树（判它们的正是这两样），不含装置二进制；随机历史脚手架只许由测试目标自己的
    /// 文件直接引，或经登记表（[`RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED`]）给这条用例登记过的库模块引，逐个引入者判
    /// （按 crate 闭包取会把整个 harness 收进来，E164 W5 量到崩溃枚举节点 20 个提交全部重跑；按模块引用取就不会）。登记表里每一行
    /// 都要在它那条用例里真的引到（没引到的是死条目）。
    /// 登记表不在（只导出 `crates/` 的树）时这一核没有对象：打一行 `CRASH_CASE_REGISTRY_ABSENT` 写明跳过、不判；
    /// 按 `tests/` 下的文件名认不出登记的那几条（登记的有不以 `crash_enumeration_` 起头的，以它起头的也有没登记的）。
    #[test]
    fn every_registered_crash_case_depends_on_recovery_and_the_checker_walk_but_not_on_apparatus_binaries(
    ) {
        let root = repository_root();
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let targets = match crash_case_registration_under(&root) {
            CrashCaseRegistration::Registered { targets } => targets,
            CrashCaseRegistration::RegistryAbsent { looked_at } => {
                println!(
                    "CRASH_CASE_REGISTRY_ABSENT registry={} 登记表不在（只导出 crates/ 的树），跳过「登记的崩溃枚举用例的判法代码」这一核",
                    looked_at.display()
                );
                return;
            }
        };
        let mut cases = 0usize;
        let mut blocked_introducers: Vec<(String, JudgedRandomHistoryIntroducer)> = Vec::new();
        let mut registrations_used: BTreeSet<(String, String)> = BTreeSet::new();
        for target in &targets {
            let entry = format!("crates/singlefs-checker-tier/tests/{target}.rs");
            let code = judging_code_by_module_references(&root, &[entry.as_str()], &toolchain)
                .expect("算得出");
            println!(
                "JUDGING_CODE entry={entry} files={} raw={} digest={}",
                code.files.len(),
                code.raw_files.len(),
                hexadecimal_text(&code.digest)
            );
            for file in &code.files {
                println!(
                    "JUDGING_CODE_FILE entry={target} file={file} reason={}",
                    code.reason_of_each_file[file]
                );
            }
            let has = |path: &str| code.files.iter().any(|file| file == path);
            assert!(
                has("crates/singlefs-core/src/recovery.rs"),
                "{target}：判定靠恢复"
            );
            assert!(
                has("crates/singlefs-checker/src/walk.rs"),
                "{target}：判定靠池级 checker 走树"
            );
            assert!(
                has("crates/singlefs-format/src/lib.rs"),
                "{target}：格式常量经 use 收进来（E164 V11）"
            );
            assert!(
                !code
                    .files
                    .iter()
                    .any(|file| file.starts_with("crates/singlefs-checker-tier/src/bin/")),
                "{target}：装置二进制不是节点判定走的代码"
            );
            let judged_introducers = random_history_scaffolding_introducers_judged(
                &root,
                target,
                &toolchain,
                RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED,
            )
            .expect("算得出");
            for judged in judged_introducers {
                println!(
                    "RANDOM_HISTORY_INTRODUCER entry={target} introducer={:?} verdict={:?}",
                    judged.introducer, judged.verdict
                );
                if let (
                    Some(file),
                    RandomHistoryIntroducerVerdict::RegisteredLibraryModule { .. },
                ) = (judged.introducer.introducing_file(), &judged.verdict)
                {
                    registrations_used.insert((target.clone(), file.to_string()));
                }
                if !judged.verdict.is_let_through() {
                    blocked_introducers.push((target.clone(), judged));
                }
            }
            cases += 1;
        }
        assert!(cases >= 8, "登记表里至少 8 条崩溃枚举用例，实际 {cases}");
        assert!(
            blocked_introducers.is_empty(),
            "随机历史脚手架经没登记的库模块或 impl 头进了登记的崩溃枚举用例的判法代码（用例，引入者，判定）：{blocked_introducers:?}。\
             拿随机历史当输入的用例在 RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED 登记那份库模块并写明理由；别的用例改成不经库模块引它"
        );
        for row in RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED {
            assert!(
                registrations_used
                    .contains(&(row.test_target.to_string(), row.library_module.to_string())),
                "登记表里 {} 经 {} 引随机历史脚手架那一行，在这条用例的判法闭包里没引到（或它不是登记的崩溃枚举用例）：死条目，删掉它（{}）",
                row.test_target,
                row.library_module,
                row.reason
            );
        }
    }

    /// 只有两个包的小仓：入口测试文件先直接引随机历史脚手架，再引一份库模块，那份库模块也引它——闭包里第二份文件经库模块引。
    /// 交回小仓的根；用完由调用方删。
    fn repository_where_the_second_file_in_the_closure_references_the_random_history_through_a_library_module(
        tag: &str,
    ) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "singlefs-random-history-introducers-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        for (file, contents) in [
            ("Cargo.lock", ""),
            (
                "crates/singlefs-harness/Cargo.toml",
                "[package]\nname = \"singlefs-harness\"\n",
            ),
            ("crates/singlefs-harness/src/lib.rs", "pub mod history;\n"),
            (
                RANDOM_HISTORY_SCAFFOLDING_FILE,
                "pub const RANDOM_HISTORY_PROBE: u8 = 1;\n",
            ),
            (
                "crates/singlefs-checker-tier/Cargo.toml",
                "[package]\nname = \"singlefs-checker-tier\"\n",
            ),
            (
                "crates/singlefs-checker-tier/src/lib.rs",
                "pub mod campaign_probe;\n",
            ),
            (
                "crates/singlefs-checker-tier/src/campaign_probe.rs",
                "use singlefs_harness::history::RANDOM_HISTORY_PROBE;\npub fn probe() -> u8 { RANDOM_HISTORY_PROBE }\n",
            ),
            (
                "crates/singlefs-checker-tier/tests/probe_case.rs",
                "use singlefs_harness::history::RANDOM_HISTORY_PROBE;\nuse singlefs_checker_tier::campaign_probe::probe;\n\
                 #[test]\nfn probe_case() { assert_eq!(probe(), RANDOM_HISTORY_PROBE); }\n",
            ),
        ] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().expect("文件有上层")).expect("建得了目录");
            std::fs::write(&path, contents).expect("写得了小仓的文件");
        }
        root
    }

    const LIBRARY_MODULE_OF_THE_SMALL_REPOSITORY: &str =
        "crates/singlefs-checker-tier/src/campaign_probe.rs";
    const ENTRY_OF_THE_SMALL_REPOSITORY: &str = "crates/singlefs-checker-tier/tests/probe_case.rs";

    /// 闭包里第二份文件经库模块引随机历史脚手架，那份库模块只给别的用例登记过：拦；入口自己直接引的那一处照旧放行。
    /// 入口先引到它，只记第一个引入者就看不见库模块那一引（这条变红）。
    #[test]
    fn a_library_module_second_in_the_closure_referencing_the_random_history_unregistered_for_the_case_is_blocked(
    ) {
        let root = repository_where_the_second_file_in_the_closure_references_the_random_history_through_a_library_module(
            "unregistered",
        );
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let registered_for_another_case = [RandomHistoryThroughALibraryModuleAllowed {
            test_target: "another_case",
            library_module: LIBRARY_MODULE_OF_THE_SMALL_REPOSITORY,
            reason: "只给别的用例登记",
        }];
        let judged = random_history_scaffolding_introducers_judged(
            &root,
            "probe_case",
            &toolchain,
            &registered_for_another_case,
        );
        std::fs::remove_dir_all(&root).expect("删得掉小仓");
        assert_eq!(
            judged.expect("小仓算得出"),
            vec![
                JudgedRandomHistoryIntroducer {
                    introducer: FileIntroducer::ReferencedBy {
                        file: LIBRARY_MODULE_OF_THE_SMALL_REPOSITORY.to_string()
                    },
                    verdict: RandomHistoryIntroducerVerdict::UnregisteredLibraryModule,
                },
                JudgedRandomHistoryIntroducer {
                    introducer: FileIntroducer::ReferencedBy {
                        file: ENTRY_OF_THE_SMALL_REPOSITORY.to_string()
                    },
                    verdict: RandomHistoryIntroducerVerdict::TestTargetFile,
                },
            ],
            "两个引入者都要判到：库模块那一引没给这条用例登记，拦；入口自己直接引，放行"
        );
    }

    /// 同一个小仓，那份库模块给这条用例登记了：放，判定带着登记的理由。
    #[test]
    fn a_library_module_second_in_the_closure_referencing_the_random_history_registered_for_the_case_is_let_through(
    ) {
        let root = repository_where_the_second_file_in_the_closure_references_the_random_history_through_a_library_module(
            "registered",
        );
        let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
        let registered_for_this_case = [RandomHistoryThroughALibraryModuleAllowed {
            test_target: "probe_case",
            library_module: LIBRARY_MODULE_OF_THE_SMALL_REPOSITORY,
            reason: "这条用例拿随机历史当输入",
        }];
        let judged = random_history_scaffolding_introducers_judged(
            &root,
            "probe_case",
            &toolchain,
            &registered_for_this_case,
        );
        std::fs::remove_dir_all(&root).expect("删得掉小仓");
        let judged = judged.expect("小仓算得出");
        assert!(
            judged
                .iter()
                .all(|introducer| introducer.verdict.is_let_through()),
            "登记过的库模块与入口自己都放行：{judged:?}"
        );
        assert!(
            judged.contains(&JudgedRandomHistoryIntroducer {
                introducer: FileIntroducer::ReferencedBy {
                    file: LIBRARY_MODULE_OF_THE_SMALL_REPOSITORY.to_string()
                },
                verdict: RandomHistoryIntroducerVerdict::RegisteredLibraryModule {
                    reason: "这条用例拿随机历史当输入"
                },
            }),
            "库模块那一引判成登记过、带着登记的理由：{judged:?}"
        );
    }
}
