//! 崩溃放量节点的身份与复用键（里程碑二「增补 4」第一项 `research/prompts/m3-prune-gpu-r3-main-verification.md`
//! 第四节的设计表认了身份形态；E164（崩溃放量新设计的小范围功能测试） 第一、二段量过各件）。
//!
//! - 键只由内容定（键里不许有路径、名字、种子、行号、手写版本号、整文件或整表指纹、按提交算的东西）：
//!   块键 = 这一步带屏障、FUA、盘、偏移、段号的写表与它起步的镜像（扇区终值）的摘要，加段展开、判器另读的输入与段内序号
//!   （`verdict_store::VerdictBlockKey`）；判器那一半只经 [`judging_units_key`] 这一个入口取（[`JudgingUnitsKey`]，判法单元指纹）。
//! - 身份 `<设这个崩溃点的文件的仓内路径>::<崩溃点名称>::<复用键>` 只作索引：进路径索引表与覆盖报告，不进任何键。
//!   标签类型不给格式化、不给比较（没有 `Display`、`Debug`、`Clone`、`Hash`、`PartialEq`、`Ord`），不能拿去当键（E164 F12）。
//! - 判法单元指纹（4-2a）：构建脚本按入口登记表（`crate::judging_units_entries`）经 `singlefs-unit-graph` 逐组从入口 fn 收函数单元闭包、
//!   算键（单元去注释的指纹、工具链与 cargo 解析后的 profile、`Cargo.lock` 里闭包真引到的外部包），连同组成清单写进 `OUT_DIR`；
//!   这里只读编进二进制的那一份，跑时不读盘上的树。二进制比盘上的树旧时它的键对不上新树算出来的键，查库不中就现判。

use std::collections::{BTreeMap, BTreeSet};

use crate::hexadecimal::hexadecimal_text;
use crate::memory_pool::{MemoryPool, RetainedWrite, WrittenContents, SECTOR_BYTES};
use crate::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_unit_graph::entry_groups::{
    parse_entry_group_table, ReadBackEntryGroup, ReadBackEntryGroupTable, ReadBackEntryGroupVerdict,
};
use singlefs_unit_graph::FeaturesEnabledByTheBuild;

use crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity;

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

/// 复用键：只能从属性算出（[`StepAttributes::reuse_key`]）。这一步的输入没变就是同一个键；判法变不变另按判法单元指纹在判定行上分。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReuseKey([u8; DIGEST_BYTES]);

impl ReuseKey {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }
}

/// 一步的两样属性（各是 SHA-256）：这一步写下的字节（带屏障的写表）与它起步的镜像。只有流程进复用键：
/// 判法是判定行的属性（`verdict_store::JudgingUnitsFingerprint`），不在这里。
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

/// 一个崩溃点在这一趟里的去向：核了、复用了库里的判定、按测试代码里的 ignore 跳过了（分叉控制「预设的 ignore 就是剪枝」）、
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

// ============================== 三种结论 ==============================

/// 观察者的名字：非空，不超过 `u16::MAX` 字节。只当索引：拼进它那一组判法单元的组名（`observer_code:<文件>:<名字>`）与行里报的名字，
/// 不进任何键（结论种类里没有它，4-2a 报告第四节第 5 条：观察者是哪一个由它那一组判法单元的键担，即结论的版本）。
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

/// 一块存的两种结论：判镜像的结论、一个观察者的结果。判定库按（块键，结论种类，版本）存与记账，哪一种没有就只排那一种。
/// 镜像判定不分在哪判的（数据不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备）：GPU 判器与 CPU 判器判出的
/// 存在同一种类、同一版本下，版本是判定的定义（[`JudgingUnits::JudgementOfTheCrashImages`]）的键；在哪判的、GPU 判器是哪一版，
/// 连同驱动版本与卡型都是块的属性（判定库的块属性表），不进种类、不进版本。
/// 观察者不带名字：是哪一个观察者由版本担（它那一组判法单元的键按入口 fn 定，同一份文件里两个观察者两把键，4-2a 报告第四节第 5 条）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConclusionKind {
    ImageJudgement,
    Observer,
}

impl ConclusionKind {
    /// 种类写成文字：`image_judgement`、`observer`。
    #[must_use]
    pub fn version_text(&self) -> String {
        match self {
            Self::ImageJudgement => "image_judgement".to_string(),
            Self::Observer => "observer".to_string(),
        }
    }
}

/// 一种结论的版本（SHA-256）：判这种结论的那一组判法单元的键（[`judging_units_key`]），观察者另拼它另读的输入的摘要与判它交回的盘的
/// 判器那一把。同一块在几把版本下的结论并存，复用只认同一版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConclusionVersion(pub [u8; DIGEST_BYTES]);

// ============================== 判器那一半键：判法单元指纹（块键、核对键、录入键、对拍键都只经这一个入口取） ==============================

/// 哪一组判法单元：判这一样结论（或录、核、拼这一样）的那几个入口 fn 收出的函数单元闭包。入口登记在 `crate::judging_units_entries`，
/// 块键、核对键、录入键、对拍键里判器那一半都经 [`judging_units_key`] 这一个入口取，调用方不碰闭包算法、不碰任何手写的版本号。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JudgingUnits {
    /// GPU 判器：内核源码的拼法、上卡与派活、回读归位、结论字的解法（判崩溃镜像与给定状态表是同一个判器）。
    GpuJudge,
    /// 崩溃镜像该判出什么的定义：CPU 参照判器的闭包（`crash::judge_crash_image`，连同结论的编码与 CPU 路线判一块的入口
    /// `crash_judge_cpu_blocks::judge_a_piece_of_a_block_on_the_cpu`）。镜像判定的版本就是这一把，
    /// 不论这一块是 CPU 判的还是 GPU 判的（GPU 判器是加速路径，它的准入靠与这一把的对拍）。
    JudgementOfTheCrashImages,
    /// 给定状态该判出什么的定义：CPU 参照判器的闭包（`crash_given_states::judge_the_given_states_on_the_cpu`，连同它的计数向量与红项编码）；
    /// 给定状态表的判定不论在哪判的都存在这一版下。
    JudgementOfTheGivenStates,
    /// 第 ② 段核对的 CPU 参照（`crash_facts::verify_states_from_facts` 一族）。
    CpuFactsVerifier,
    /// 第 ② 段核对的 GPU 内核（`crash_verify_gpu::GpuFactsVerifier`）。
    GpuFactsVerifier,
    /// 录入段：写进库的那几样怎么从流算出、怎么编成字节（事实表抽取、判表建表与打包、录入的流）。
    Recording,
    /// 观察者结论拼块：观察者交回的盘怎么建判表、判不了的怎么记、一块怎么拼。
    ObserverConclusionComposition,
    /// 一种对拍的比法（对拍测试里逐状态比两边判定的那几个函数，登记表里的组名）：它的键拼进对拍结论的输入摘要，比法改了不复用早先的结论。
    PairingComparison { group_name: &'static str },
    /// 一个观察者：（观察者代码文件，观察者名）只用来在登记表里找它的入口 fn，键是那几个入口 fn 的闭包键（同一份文件里两个观察者各自一把）。
    ObserverCode {
        observer_code_file: String,
        observer_name: String,
    },
}

impl JudgingUnits {
    /// 行与组成清单里的种类名（观察者不带文件与名字）。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::GpuJudge => "gpu_judge",
            Self::JudgementOfTheCrashImages => "judgement_of_the_crash_images",
            Self::JudgementOfTheGivenStates => "judgement_of_the_given_states",
            Self::CpuFactsVerifier => "cpu_facts_verifier",
            Self::GpuFactsVerifier => "gpu_facts_verifier",
            Self::Recording => "recording",
            Self::ObserverConclusionComposition => "observer_conclusion_composition",
            Self::PairingComparison { .. } => "pairing_comparison",
            Self::ObserverCode { .. } => "observer_code",
        }
    }

    /// 编进二进制的那份表里这一组的组名（只当索引）：判器、核对、录入、拼块各是 [`Self::name`]，观察者是
    /// `crate::judging_units_entries::observer_group_name` 拼的那一个。
    #[must_use]
    pub fn group_name(&self) -> String {
        match self {
            Self::ObserverCode {
                observer_code_file,
                observer_name,
            } => {
                crate::judging_units_entries::observer_group_name(observer_code_file, observer_name)
            }
            Self::PairingComparison { group_name } => (*group_name).to_string(),
            Self::GpuJudge
            | Self::JudgementOfTheCrashImages
            | Self::JudgementOfTheGivenStates
            | Self::CpuFactsVerifier
            | Self::GpuFactsVerifier
            | Self::Recording
            | Self::ObserverConclusionComposition => self.name().to_string(),
        }
    }
}

/// 一组判法单元里的一个单元与它的指纹：组成清单的一行，随键落盘，库对不上时照它点名变了的单元。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JudgingUnitFingerprint {
    pub unit: String,
    pub fingerprint: [u8; DIGEST_BYTES],
}

/// 判器那一半键：只按内容取值（这一组判法单元的复用键），不带路径、名字、手写版本号。组成清单随键落盘
/// （判定库的组成清单表），作废时点名变了的单元。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgingUnitsKey {
    fingerprint: [u8; DIGEST_BYTES],
    composition: Vec<JudgingUnitFingerprint>,
}

impl JudgingUnitsKey {
    /// 从指纹与组成清单建（编进二进制的表读回的、库里读回的组成清单、测试里造的）。
    #[must_use]
    pub fn of_the_parts(
        fingerprint: [u8; DIGEST_BYTES],
        composition: Vec<JudgingUnitFingerprint>,
    ) -> Self {
        Self {
            fingerprint,
            composition,
        }
    }

    #[must_use]
    pub fn fingerprint(&self) -> &[u8; DIGEST_BYTES] {
        &self.fingerprint
    }

    #[must_use]
    pub fn composition(&self) -> &[JudgingUnitFingerprint] {
        &self.composition
    }

    /// 组成清单的文字：一个单元一行 `<单元>\t<指纹十六进制>`，按单元排。
    #[must_use]
    pub fn composition_text(&self) -> String {
        let mut sorted = self.composition.clone();
        sorted.sort();
        sorted
            .iter()
            .map(|unit| format!("{}\t{}\n", unit.unit, hexadecimal_text(&unit.fingerprint)))
            .collect()
    }

    /// [`Self::composition_text`] 的逆；有一行认不出交回 `None`。
    #[must_use]
    pub fn composition_from_text(text: &str) -> Option<Vec<JudgingUnitFingerprint>> {
        text.lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let (unit, hexadecimal) = line.split_once('\t')?;
                let bytes: Vec<u8> = (0..hexadecimal.len())
                    .step_by(2)
                    .map(|start| {
                        hexadecimal
                            .get(start..start + 2)
                            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                    })
                    .collect::<Option<Vec<u8>>>()?;
                Some(JudgingUnitFingerprint {
                    unit: unit.to_string(),
                    fingerprint: bytes.try_into().ok()?,
                })
            })
            .collect()
    }
}

/// 库里存着的那一把与这一趟的组成清单比：两边指纹不同、或只有一边有的单元，按单元排。两边一样是空的。
#[must_use]
pub fn units_changed_between(
    stored: &[JudgingUnitFingerprint],
    now: &[JudgingUnitFingerprint],
) -> Vec<String> {
    let stored_of_each_unit: BTreeMap<&str, &[u8; DIGEST_BYTES]> = stored
        .iter()
        .map(|unit| (unit.unit.as_str(), &unit.fingerprint))
        .collect();
    let now_of_each_unit: BTreeMap<&str, &[u8; DIGEST_BYTES]> = now
        .iter()
        .map(|unit| (unit.unit.as_str(), &unit.fingerprint))
        .collect();
    stored_of_each_unit
        .keys()
        .chain(now_of_each_unit.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|unit| stored_of_each_unit.get(**unit) != now_of_each_unit.get(**unit))
        .map(|unit| (*unit).to_string())
        .collect()
}

/// 库对不上时红句里点名变了的单元那一截：以「变了的单元：」起头，接逗号隔开的单元清单；组成清单两边都空、比不出时写明。
#[must_use]
pub fn changed_units_sentence(changed_units: &[String]) -> String {
    if changed_units.is_empty() {
        "变了的单元：（组成清单比不出：库里那一把或这一趟的组成清单是空表）".to_string()
    } else {
        format!("变了的单元：{}", changed_units.join("，"))
    }
}

/// 判器那一半键算不出：哪一组、什么形态。调用方照判、不入库，计入拒算（不退回整文件或整表指纹，不跳过）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgingUnitsKeyRefused {
    /// 编进二进制的表里没有这一组（观察者没登记），或这一组的入口在这一趟开的特性下一个都不编进来（例：没开 `gpu` 时取 GPU 判器那一把）。
    NotComputedAtTheBuild { units: JudgingUnits, reason: String },
    /// 编译时这一组的闭包里有认不出的形态（`singlefs-unit-graph` 的拒算行，点名单元与形态）。
    RefusedAtTheBuild {
        units: JudgingUnits,
        refusal_lines: Vec<String>,
    },
}

impl JudgingUnitsKeyRefused {
    /// 拒算的是哪一组。
    #[must_use]
    pub fn units(&self) -> &JudgingUnits {
        match self {
            Self::NotComputedAtTheBuild { units, .. } | Self::RefusedAtTheBuild { units, .. } => {
                units
            }
        }
    }

    /// 拒算的形态名（行里 `form=` 那一截）。
    #[must_use]
    pub fn form_name(&self) -> &'static str {
        match self {
            Self::NotComputedAtTheBuild { .. } => "not_computed_at_the_build",
            Self::RefusedAtTheBuild { .. } => "refused_at_the_build",
        }
    }

    /// 红句：哪一组、什么形态、为什么；闭包里认不出的点名拒算行。
    #[must_use]
    pub fn sentence(&self) -> String {
        match self {
            Self::NotComputedAtTheBuild { units, reason } => format!(
                "判法单元 {} 拒算（{}）：{reason}",
                units.group_name(),
                self.form_name()
            ),
            Self::RefusedAtTheBuild {
                units,
                refusal_lines,
            } => format!(
                "判法单元 {} 拒算（{}）：闭包里有认不出的形态：{}",
                units.group_name(),
                self.form_name(),
                refusal_lines.join(" | ").replace('\t', " ")
            ),
        }
    }
}

/// 构建脚本写进 `OUT_DIR` 的那份判器键表（`singlefs_unit_graph::entry_groups` 的写法，文件名与 `build.rs` 的
/// `JUDGING_UNITS_KEYS_FILE_NAME` 相同）。
const COMPILED_JUDGING_UNITS_KEYS_TEXT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/judging_units_keys.tsv"));

/// 编进二进制的那份判器键表，读回一次。
///
/// # Panics
/// 表认不回来：它是构建脚本用 `entry_group_table_text` 写的，认不回来是两边写法不一致的 bug。
#[must_use]
pub fn compiled_judging_units_keys() -> &'static ReadBackEntryGroupTable {
    static TABLE: std::sync::OnceLock<ReadBackEntryGroupTable> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        parse_entry_group_table(COMPILED_JUDGING_UNITS_KEYS_TEXT).unwrap_or_else(|reason| {
            panic!("构建脚本写进二进制的判器键表认不回来（两边写法不一致）：{reason}")
        })
    })
}

/// 核心层的包名（清单 `[package] name`，编进去的那份表 `build-feature` 行里点名它的写法）。
pub const CORE_LAYER_PACKAGE_NAME: &str = "singlefs-core";

/// 核心层按身份查的计数那个特性（验证包的 dev 依赖打开它）。
pub const LOOKUP_WORK_COUNTERS_FEATURE: &str = "lookup-work-counters";

/// 核心层发布触发阈值的只供测试来源那个特性（验证包的 dev 依赖打开它，判决 `research/prompts/m4-publish-trigger-r1-main-verification.md`
/// 第二节「只供测试的开关」那一格）。
pub const TEST_ONLY_PUBLISH_THRESHOLDS_FEATURE: &str = "test-only-publish-thresholds";

/// 核心层清单 `[features]` 里的特性，[`features_the_core_layer_was_compiled_with`] 认得出开没开的就是这几个（按清单里的名字排好）。
pub const CORE_LAYER_FEATURES_RECOGNIZED_FROM_ITS_CODE: [&str; 2] = [
    LOOKUP_WORK_COUNTERS_FEATURE,
    TEST_ONLY_PUBLISH_THRESHOLDS_FEATURE,
];

/// 这个二进制链进来的核心层实际开着哪些特性，从核心层编出来的代码里认、不经构建脚本：拿来对照编进去的那份表里依赖包的特性
/// （`build-feature` 行，构建脚本读核心层经 `links` 交出的那一份）。只编验证包、不编它的测试时一个都不开；编验证包的测试时，
/// 验证包的 dev 依赖替核心层并上 `lookup-work-counters` 与 `test-only-publish-thresholds`。一个都没开时交空表（与构建脚本取到空集时一样，不留核心层那一项）。
#[must_use]
pub fn features_the_core_layer_was_compiled_with() -> FeaturesEnabledByTheBuild {
    let mut features_of_each_package = BTreeMap::new();
    let mut features_of_the_core_layer = BTreeSet::new();
    if core_layer_feature_probe::lookup_work_counters_are_compiled_into_the_core_layer() {
        features_of_the_core_layer.insert(LOOKUP_WORK_COUNTERS_FEATURE.to_string());
    }
    if core_layer_publish_thresholds_probe::test_only_publish_thresholds_are_compiled_into_the_core_layer()
    {
        features_of_the_core_layer.insert(TEST_ONLY_PUBLISH_THRESHOLDS_FEATURE.to_string());
    }
    if !features_of_the_core_layer.is_empty() {
        features_of_each_package.insert(
            CORE_LAYER_PACKAGE_NAME.to_string(),
            features_of_the_core_layer,
        );
    }
    FeaturesEnabledByTheBuild {
        features_of_each_package,
    }
}

/// 认核心层开没开 `lookup-work-counters`：开着时核心层才有 `transaction::lookup_work::lookup_work_while`。函数体里 glob 引入那个模块，
/// 名字在那里找得到就用核心层的；找不到，落到本模块里同名的替身上。再按交回的计数是不是替身的类型认是哪一个。
/// 核心层把那个函数改名、或改成不挂特性，这里认错的那一边会让对照它的测试红，不会静默。
mod core_layer_feature_probe {
    /// 替身交回的计数（核心层没编进计数时才构造）。
    #[allow(
        dead_code,
        reason = "核心层开着计数时名字落到核心层那一个上，替身不构造这个值"
    )]
    pub(super) struct CountersNotCompiledIntoTheCoreLayer;

    /// 核心层没编进计数时，`lookup_work_while` 这个名字落到这里。
    #[allow(
        dead_code,
        reason = "核心层开着计数时名字落到核心层那一个上，替身用不到"
    )]
    pub(super) fn lookup_work_while<Output>(
        action: impl FnOnce() -> Output,
    ) -> (Output, CountersNotCompiledIntoTheCoreLayer) {
        (action(), CountersNotCompiledIntoTheCoreLayer)
    }

    pub(super) fn lookup_work_counters_are_compiled_into_the_core_layer() -> bool {
        #[allow(
            unused_imports,
            reason = "核心层没编进计数时这个模块里没有这里用得上的名字"
        )]
        use singlefs_core::transaction::lookup_work::*;
        let ((), work_while_doing_nothing) = lookup_work_while(|| ());
        !is_the_lookup_work_counters_stand_in(&work_while_doing_nothing)
    }

    fn is_the_lookup_work_counters_stand_in<Work: 'static>(_work: &Work) -> bool {
        std::any::TypeId::of::<Work>()
            == std::any::TypeId::of::<CountersNotCompiledIntoTheCoreLayer>()
    }
}

/// 认核心层开没开 `test-only-publish-thresholds`：开着时核心层的 `PublishTriggerThresholds` 才有固有的
/// `of_test_only_values_clamped_by_the_journal_ring`。按类型名调它：固有的那个先于本模块里同名的特征方法（替身），没有固有的才落到替身上；
/// 再按交回的值是不是替身的类型认是哪一个。核心层把那个函数改名、或改成不挂特性，这里认错的那一边会让对照它的测试红，不会静默。
mod core_layer_publish_thresholds_probe {
    use singlefs_core::system_configuration::PublishTriggerThresholds;

    /// 替身交回的值（核心层没编进只供测试来源时才构造）。
    #[allow(
        dead_code,
        reason = "核心层开着那个特性时名字落到核心层固有的那一个上，替身不构造这个值"
    )]
    pub(super) struct ThresholdsNotCompiledIntoTheCoreLayer;

    /// 核心层没编进只供测试来源时，`PublishTriggerThresholds::of_test_only_values_clamped_by_the_journal_ring` 这个名字落到这里。
    #[allow(
        dead_code,
        reason = "核心层开着那个特性时名字落到核心层固有的那一个上，替身用不到"
    )]
    trait StandInForTheTestOnlyPublishThresholds {
        fn of_test_only_values_clamped_by_the_journal_ring(
            time_threshold: std::time::Duration,
            dirty_threshold_before_the_clamp: u64,
            journal_ring_bytes: u64,
        ) -> ThresholdsNotCompiledIntoTheCoreLayer;
    }

    impl StandInForTheTestOnlyPublishThresholds for PublishTriggerThresholds {
        fn of_test_only_values_clamped_by_the_journal_ring(
            _time_threshold: std::time::Duration,
            _dirty_threshold_before_the_clamp: u64,
            _journal_ring_bytes: u64,
        ) -> ThresholdsNotCompiledIntoTheCoreLayer {
            ThresholdsNotCompiledIntoTheCoreLayer
        }
    }

    pub(super) fn test_only_publish_thresholds_are_compiled_into_the_core_layer() -> bool {
        let made = PublishTriggerThresholds::of_test_only_values_clamped_by_the_journal_ring(
            std::time::Duration::ZERO,
            0,
            0,
        );
        !is_the_publish_thresholds_stand_in(&made)
    }

    fn is_the_publish_thresholds_stand_in<Made: 'static>(_made: &Made) -> bool {
        std::any::TypeId::of::<Made>()
            == std::any::TypeId::of::<ThresholdsNotCompiledIntoTheCoreLayer>()
    }
}

/// 表里一组的组成清单换成组成清单行：一个单元一行，单元写 `<种类> <限定名> @ <文件>`（不带行号：行挪了不算单元变了），
/// 同一份文件里种类与限定名都相同的几个单元按出现次序另加 `#<第几个>`。
fn composition_of_the_compiled_group(group: &ReadBackEntryGroup) -> Vec<JudgingUnitFingerprint> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    group
        .composition
        .iter()
        .map(|unit| {
            let file = unit
                .location
                .rsplit_once(':')
                .map_or(unit.location.as_str(), |(file, _lines)| file);
            let base = format!("{} {} @ {file}", unit.kind, unit.qualified_name);
            let occurrence = seen.entry(base.clone()).or_insert(0);
            *occurrence += 1;
            let name = if *occurrence == 1 {
                base
            } else {
                format!("{base} #{occurrence}")
            };
            JudgingUnitFingerprint {
                unit: name,
                fingerprint: unit.fingerprint,
            }
        })
        .collect()
}

/// 判器那一半键的唯一入口：块键、核对键、录入键、对拍键都只经它取。读编进二进制的那份表（构建脚本按入口登记表逐组算的）里这一组的键与
/// 组成清单，跑时不读盘上的树，也不收仓根（仓根不是键的输入）：`toolchain` 这一版不用（工具链已在编译时进了键），签名照旧留给
/// `crash_library`、`crash_state_enumeration` 那几处调用方。
///
/// # Errors
/// 拒算（[`JudgingUnitsKeyRefused`]）：表里没有这一组、这一组在这一趟的特性下不编进来、编译时闭包里有认不出的形态。调用方照判、不入库，计入拒算。
pub fn judging_units_key(
    units: &JudgingUnits,
    _toolchain: &CrashPointReplayToolchainIdentity,
) -> Result<JudgingUnitsKey, JudgingUnitsKeyRefused> {
    let group_name = units.group_name();
    let Some(group) = compiled_judging_units_keys()
        .groups
        .iter()
        .find(|group| group.group_name == group_name)
    else {
        return Err(JudgingUnitsKeyRefused::NotComputedAtTheBuild {
            units: units.clone(),
            reason: format!(
                "编译时的判器键表里没有组 {group_name}（观察者要在 crate::judging_units_entries 的 OBSERVER_ENTRIES 登记它的入口 fn）"
            ),
        });
    };
    match &group.verdict {
        ReadBackEntryGroupVerdict::Keyed { key } => Ok(JudgingUnitsKey::of_the_parts(
            *key,
            composition_of_the_compiled_group(group),
        )),
        ReadBackEntryGroupVerdict::Refused { refusal_lines } => {
            Err(JudgingUnitsKeyRefused::RefusedAtTheBuild {
                units: units.clone(),
                refusal_lines: refusal_lines.clone(),
            })
        }
        ReadBackEntryGroupVerdict::NotCompiled { reason } => {
            Err(JudgingUnitsKeyRefused::NotComputedAtTheBuild {
                units: units.clone(),
                reason: reason.clone(),
            })
        }
    }
}

/// 别的包的构建脚本照同一套（`singlefs-unit-graph` 的 `entry_group_table_text_with_the_features_of_the_build`）写下的键表里点名一组的判法单元键：
/// 认表、取指纹、拼组成清单都是 [`judging_units_key`] 那一份代码。产品 GPU 包的测试包拿它取 CPU 参照那一侧的单元键，拼进对拍参照的录入键
/// （`crate::crash_library::pairing_input_and_reference_through_the_pairing_library`）。
///
/// # Errors
/// 表认不回来；表里没有这一组；这一组拒算或没编进来。交回那一句（调用方当拒算处置：照录、不复用）。
pub fn judging_units_key_of_the_group_in_the_table_text(
    table_text: &str,
    group_name: &str,
) -> Result<JudgingUnitsKey, String> {
    let table = parse_entry_group_table(table_text)
        .map_err(|reason| format!("判法单元键表认不回来：{reason}"))?;
    let group = table
        .groups
        .iter()
        .find(|group_of_the_table| group_of_the_table.group_name == group_name)
        .ok_or_else(|| format!("判法单元键表里没有组 {group_name}"))?;
    match &group.verdict {
        ReadBackEntryGroupVerdict::Keyed { key } => Ok(JudgingUnitsKey::of_the_parts(
            *key,
            composition_of_the_compiled_group(group),
        )),
        ReadBackEntryGroupVerdict::Refused { refusal_lines } => Err(format!(
            "组 {group_name} 的判法单元拒算：{}",
            refusal_lines.join("；")
        )),
        ReadBackEntryGroupVerdict::NotCompiled { reason } => {
            Err(format!("组 {group_name} 这一次没编进来：{reason}"))
        }
    }
}

/// 判定那一把版本的判法单元键：判定定义那一组（`definition` 是哪一组、`definition_units` 是它的键：判崩溃镜像的、或判给定状态的）与 GPU 判器那一组（`gpu_judge_units`）并成一把
/// （「GPU 判器代码进键」：GPU 判器那一组的单元并进每一块的键；CPU 判与 GPU 判同一份输入块键仍逐字相同，不与硬件绑定，
/// 与 D13（验证路线） 已定项 17 依据里「判定不与硬件、不与 CPU 还是 GPU 绑定」相容）。两条路线都取这同一把：GPU 判器那一组不论这一次
/// 开没开 `gpu` 特性都照开着收闭包、算键（`crate::judging_units_entries::GROUPS_KEYED_AS_COMPILED_WITH_THE_GPU_FEATURE`），改 GPU 内核里一个 fn，
/// 两条路线判过的块都换版本、全部现判。指纹是两把指纹按「定义在前、GPU 判器在后」拼起来的 SHA-256；组成清单是两组的单元各带组名起头
/// （库对不上时点名变了的是哪一组的哪个单元）。在哪判的、卡型与驱动照旧只进块属性。
#[must_use]
pub fn judgement_units_key_with_the_gpu_judge(
    definition: &JudgingUnits,
    definition_units: &JudgingUnitsKey,
    gpu_judge_units: &JudgingUnitsKey,
) -> JudgingUnitsKey {
    let mut message = b"singlefs judgement units with the gpu judge\n".to_vec();
    message.extend_from_slice(definition_units.fingerprint());
    message.extend_from_slice(gpu_judge_units.fingerprint());
    let named_with_the_group =
        |group: &JudgingUnits, unit: &JudgingUnitFingerprint| JudgingUnitFingerprint {
            unit: format!("{}: {}", group.name(), unit.unit),
            fingerprint: unit.fingerprint,
        };
    let mut composition: Vec<JudgingUnitFingerprint> = definition_units
        .composition()
        .iter()
        .map(|unit| named_with_the_group(definition, unit))
        .chain(
            gpu_judge_units
                .composition()
                .iter()
                .map(|unit| named_with_the_group(&JudgingUnits::GpuJudge, unit)),
        )
        .collect();
    composition.sort();
    JudgingUnitsKey::of_the_parts(sha256_digest(&message), composition)
}

/// 判崩溃镜像那一把版本（[`judgement_units_key_with_the_gpu_judge`]）：判定定义那一组与 GPU 判器那一组都经 [`judging_units_key`] 取、并成一把。
///
/// # Errors
/// 两组里哪一组拒算（先看定义那一组）。
pub fn image_judgement_units_key(
    toolchain: &CrashPointReplayToolchainIdentity,
) -> Result<JudgingUnitsKey, JudgingUnitsKeyRefused> {
    let definition_units = judging_units_key(&JudgingUnits::JudgementOfTheCrashImages, toolchain)?;
    let gpu_judge_units = judging_units_key(&JudgingUnits::GpuJudge, toolchain)?;
    Ok(judgement_units_key_with_the_gpu_judge(
        &JudgingUnits::JudgementOfTheCrashImages,
        &definition_units,
        &gpu_judge_units,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::judging_units_entries::{
        features_of_the_group_request, observer_group_name, GPU_FEATURE,
        GPU_FILES_ONLY_THE_GPU_GROUPS_REACH, JUDGING_UNITS_ENTRY_GROUPS, OBSERVER_ENTRIES,
        PIPELINE_FILES_THE_JUDGES_MUST_NOT_REACH,
    };
    use singlefs_unit_graph::build_toolchain::{
        effective_profile_lines, toolchain_lines_of_the_build, BuildToolchainInputs,
    };
    use singlefs_unit_graph::entry_groups::{
        entry_group_key_not_following_the_listed_units, EntryGroupRequest, EntryGroupVerdict,
    };
    use singlefs_unit_graph::heavy_threads::default_heavy_threads;
    use singlefs_unit_graph::manifest::parse_crate_manifest;
    use singlefs_unit_graph::sha256::sha256_of;
    use singlefs_unit_graph::{load_workspace, parse_lock_file, RuleSwitches, TargetSelector};
    use std::path::{Path, PathBuf};

    /// 单测进程里层那一份额度同时容得下几次装工作区（`load_workspace`）：一次装工作区只要那一份的这么多分之一（至少 1 条），
    /// 这几条同时装工作区的用例各拿各的、一起跑，不互相排成一列（推的：这个模块里装工作区的用例几十条，同时到的有几条没量过）。
    const WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS: usize = 4;

    /// 里层那一份额度是 `inner_threads_of_the_share` 条时，一次装工作区要几条：那一份的
    /// [`WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS`] 分之一，向下取整，至少 1 条。
    fn workspace_loading_threads_wanted(inner_threads_of_the_share: usize) -> usize {
        (inner_threads_of_the_share / WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS).max(1)
    }

    /// 装工作区那一下开的线程：从单测进程里层那一份额度拿 [`workspace_loading_threads_wanted`] 条
    /// （[`crate::crash_library::InnerThreadsTaken`]，放不下就等别的用例还），不照每台默认重活线程数开：libtest 已照这个数并跑那么多条用例。
    /// 交回守卫与交给 `load_workspace` 的条数；守卫在 `load_workspace` 返回之后丢掉才还（它的线程在返回之前都收齐了）。
    fn workspace_loading_threads_from_the_test_process_share() -> (
        crate::crash_library::InnerThreadsTaken,
        std::num::NonZeroUsize,
    ) {
        let inner_threads_of_the_share =
            crate::crash_library::shares_of_the_test_process(default_heavy_threads().get())
                .inner_threads;
        let threads_taken = crate::crash_library::InnerThreadsTaken::take(
            workspace_loading_threads_wanted(inner_threads_of_the_share),
        );
        let threads = std::num::NonZeroUsize::new(threads_taken.threads())
            .expect("里层额度至少给一条（inner_threads_granted 取了 max(1)）");
        (threads_taken, threads)
    }

    /// 场景：里层那一份额度从 1 条到 64 条逐个取，每一种按 [`workspace_loading_threads_wanted`] 与
    /// `crate::crash_library::inner_threads_granted` 算一次装工作区拿到几条，再照 `crate::crash_library::inner_threads_fit`
    /// 把装工作区逐次放进这一份，放到 [`WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS`] 次或放不下为止。只算数，不碰单测进程此刻占着的额度，不等别的用例。
    /// 预期分两种：里层不少于 [`WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS`] 条时，那几次逐次都放得下（同时装工作区的用例不互相排成一列），
    /// 几次合起来拿到的与那一份差不到 [`WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS`] 条（分份只取整掉零头）；里层少于那个数时，
    /// 每次只拿 1 条，放得下的恰好是里层的条数那么多次（每条线程各装一次工作区，不多占也不空着）。
    /// 判别力（`crates/mutations.tsv`）：一次装工作区要整份里层额度，里层 2 条起第二次就放不下，放进的次数对不上，这条红。
    #[test]
    fn workspace_loadings_sharing_the_inner_threads_fit_in_the_share_together() {
        for inner_threads_of_the_share in 1..=64 {
            let granted = crate::crash_library::inner_threads_granted(
                workspace_loading_threads_wanted(inner_threads_of_the_share),
                inner_threads_of_the_share,
            );
            let mut in_use = 0;
            let mut loadings_placed = 0;
            // 迭代上界是 WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS 次；放不下就停
            while loadings_placed < WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS
                && crate::crash_library::inner_threads_fit(
                    in_use,
                    granted,
                    inner_threads_of_the_share,
                )
            {
                in_use += granted;
                loadings_placed += 1;
            }
            if inner_threads_of_the_share >= WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS {
                assert_eq!(
                    loadings_placed, WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS,
                    "里层 {inner_threads_of_the_share} 条、每次装工作区拿 {granted} 条：只放进 {loadings_placed} 次（占 {in_use} 条），同时装工作区的用例排成了一列"
                );
                assert!(
                    in_use + WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS > inner_threads_of_the_share,
                    "里层 {inner_threads_of_the_share} 条、每次装工作区拿 {granted} 条：几次合起来只 {in_use} 条，余下的够再分一轮"
                );
            } else {
                assert_eq!(
                    (granted, loadings_placed),
                    (1, inner_threads_of_the_share),
                    "里层 {inner_threads_of_the_share} 条少于 {WORKSPACE_LOADINGS_SHARING_THE_INNER_THREADS}：每次装工作区该拿 1 条、放得下里层条数那么多次"
                );
            }
        }
    }

    #[test]
    fn node_label_takes_only_relative_paths_inside_the_repository() {
        assert!(
            NodeLabel::new("/home/x/crates/a.rs", "t").is_err(),
            "绝对路径不收：身份要能跨机器"
        );
        assert!(NodeLabel::new("crates/../a.rs", "t").is_err());
        let label = NodeLabel::new("crates/singlefs-verification/tests/a.rs", "m::t")
            .expect("仓内相对路径");
        let key = StepAttributes {
            write_table_with_barriers: [1; 32],
            parent_output: [2; 32],
        }
        .reuse_key();
        let text = label.identity_text(&key);
        assert!(
            text.starts_with("crates/singlefs-verification/tests/a.rs::m::t::"),
            "身份三段、不带步号：{text}"
        );
        assert_eq!(
            text.len(),
            "crates/singlefs-verification/tests/a.rs::m::t::".len() + 64
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
                parent_output: base.parent_output,
            }
            .reuse_key(),
            key
        );
        assert_ne!(
            StepAttributes {
                write_table_with_barriers: base.write_table_with_barriers,
                parent_output: [9; 32],
            }
            .reuse_key(),
            key
        );
        assert_eq!(base.reuse_key(), key, "同样的属性同一个键");
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

    // ============================== 判法单元：编进二进制的那份判器键表 ==============================

    /// 组名 → 那一组的键（拒算或不编进来的是 `None`）。
    type KeysOfEachGroup = BTreeMap<String, Option<[u8; DIGEST_BYTES]>>;

    /// 这个测试二进制编进来的特性（构建脚本按 `CARGO_FEATURE_*` 取的同一组）。
    fn features_of_this_build() -> Vec<String> {
        let mut features = Vec::new();
        if cfg!(feature = "gpu") {
            features.push("gpu".to_string());
        }
        features
    }

    /// 编进二进制的工具链行，profile 那几行按 `root` 那棵树的根 `Cargo.toml` 重算（经构建脚本那一份拼法
    /// `toolchain_lines_of_the_build`）：`rustc -vV`、`cargo -V`、目标与 RUSTFLAGS 取编进去的那几行。
    fn toolchain_lines_under(root: &Path) -> Vec<String> {
        let compiled = &compiled_judging_units_keys().toolchain_lines;
        let value_after = |prefix: &str| -> String {
            compiled
                .iter()
                .find_map(|line| line.strip_prefix(prefix))
                .unwrap_or_else(|| panic!("编进去的工具链行里有「{prefix}」那一行：{compiled:?}"))
                .to_string()
        };
        let rustc_version_text = compiled
            .iter()
            .take_while(|line| !line.starts_with("cargo-version: "))
            .cloned()
            .collect::<Vec<String>>()
            .join("\n");
        let rustflags = value_after("rustflags: ")
            .split(' ')
            .filter(|flag| !flag.is_empty())
            .collect::<Vec<&str>>()
            .join("\u{1f}");
        let root_manifest =
            std::fs::read_to_string(root.join("Cargo.toml")).expect("根 Cargo.toml 读得了");
        let profile_lines =
            effective_profile_lines(&root_manifest, &value_after("profile: "), &|name| {
                std::env::var(name).ok()
            })
            .expect("profile 的生效值算得出");
        toolchain_lines_of_the_build(&BuildToolchainInputs {
            rustc_version_text: &rustc_version_text,
            cargo_version_text: &value_after("cargo-version: "),
            target_triple: &value_after("target: "),
            encoded_rustflags: &rustflags,
            profile_lines: &profile_lines,
        })
    }

    /// 一组收闭包时不沿它收的单元（登记表里那一组的那一份，与构建脚本交的相同）。
    fn units_not_followed_of_the_group_as_a_set(
        group_name: &str,
    ) -> std::collections::BTreeSet<String> {
        crate::judging_units_entries::units_not_followed_of_the_group(group_name)
            .iter()
            .map(|unit| (*unit).to_string())
            .collect()
    }

    /// GPU 判器那一组在 `root` 那棵树上照开着 `gpu` 收闭包，交（组成清单的行，登记了不沿它收、这一趟真够到的单元）；
    /// `units_not_followed` 给空的就是不带登记的那一份闭包。
    #[cfg(feature = "gpu")]
    fn gpu_judge_composition_under(
        root: &Path,
        units_not_followed: &std::collections::BTreeSet<String>,
    ) -> (Vec<String>, Vec<String>) {
        let switches = RuleSwitches::every_rule_intact();
        let (threads_taken, threads) = workspace_loading_threads_from_the_test_process_share();
        let workspace = load_workspace(root, threads, &switches).expect("工作区装得起来");
        drop(threads_taken);
        let packages = parse_lock_file(
            &std::fs::read_to_string(root.join("Cargo.lock")).expect("Cargo.lock 读得了"),
        )
        .expect("Cargo.lock 解析得了");
        let group = JUDGING_UNITS_ENTRY_GROUPS
            .iter()
            .find(|group| group.group_name == "gpu_judge")
            .expect("登记表里有 GPU 判器那一组");
        let request = EntryGroupRequest {
            group_name: group.group_name.to_string(),
            target: TargetSelector::parse(group.target).expect("登记表里的目标写法认得"),
            entry_paths: group
                .entry_functions
                .iter()
                .map(|entry| (*entry).to_string())
                .collect(),
            features: features_of_the_group_request(group.group_name, &features_of_this_build()),
        };
        let computed = entry_group_key_not_following_the_listed_units(
            &workspace,
            &request,
            units_not_followed,
            &compiled_judging_units_keys().features_enabled_by_the_build,
            &toolchain_lines_under(root),
            &packages,
            &switches,
        )
        .expect("GPU 判器那一组的入口找得到");
        (
            computed.key.composition_lines,
            computed.units_reached_and_not_followed,
        )
    }

    #[cfg(feature = "gpu")]
    const MEMORY_LEDGER_FILE: &str = "crates/singlefs-verification/src/gpu_memory_ledger.rs";

    /// 组成清单一行（`<种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹>`）里的限定名。
    #[cfg(feature = "gpu")]
    fn qualified_name_of_the_composition_line(line: &str) -> &str {
        line.split('\t').nth(1).expect("组成清单的行有四段")
    }

    /// 不带登记收闭包时显存记账里进 GPU 判器那一组的单元，恰好是登记了不沿它收的那几个：登记了而不在里面的（陈旧的登记）红，
    /// 在里面而没登记的（显存记账里新进这一组闭包的单元）也红——后一种由人判它改不改判定字，再决定登记还是让它进键。
    /// 带登记收时显存记账的单元一个都不在闭包里，真够到的那几个（从别的模块跨进来的头几个）都在登记里。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_gpu_judge_group_reaches_no_unit_of_the_memory_ledger_beyond_the_listed_ones() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("本 crate 在仓根的 crates/ 下")
            .to_path_buf();
        let listed = units_not_followed_of_the_group_as_a_set("gpu_judge");
        let listed_of_the_ledger: std::collections::BTreeSet<String> =
            crate::judging_units_entries::UNITS_OF_THE_MEMORY_LEDGER_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW
                .iter()
                .map(|unit| (*unit).to_string())
                .collect();
        let (following_everything, _) =
            gpu_judge_composition_under(&root, &std::collections::BTreeSet::new());
        let ledger_units_reached_following_everything: std::collections::BTreeSet<String> =
            following_everything
                .iter()
                .filter(|line| line.contains(MEMORY_LEDGER_FILE))
                // `use` 行不登记（限定名里带行号、一挪就变）：模块里的单元都不在了它自然不进闭包
                .filter(|line| !line.starts_with("use\t"))
                .map(|line| qualified_name_of_the_composition_line(line).to_string())
                .collect();
        assert_eq!(
            ledger_units_reached_following_everything, listed_of_the_ledger,
            "左边是不带登记时显存记账里进这一组闭包的单元，右边是登记：左边多的是新进来的单元（判它改不改判定字），右边多的是陈旧的登记"
        );
        let (composition, reached_and_not_followed) = gpu_judge_composition_under(&root, &listed);
        assert!(
            !reached_and_not_followed.is_empty()
                && reached_and_not_followed
                    .iter()
                    .all(|unit| listed.contains(unit)),
            "带登记收时真够到又没沿它收的单元都在登记里、至少一个：{reached_and_not_followed:?}"
        );
        let ledger_units_in_the_closure: Vec<&String> = composition
            .iter()
            .filter(|line| line.contains(MEMORY_LEDGER_FILE))
            .collect();
        assert!(
            ledger_units_in_the_closure.is_empty(),
            "带登记收时显存记账里还有单元在 GPU 判器那一组的闭包里：{ledger_units_in_the_closure:?}"
        );
    }

    /// 登记按限定名比：登记的每个限定名在工作区里恰好是一个单元（同名的两个单元——cfg 两臂、两个写法相同的 impl 块头——
    /// 会被一条登记一起排掉，其中没逐个判过的那一个就漏了）。
    #[cfg(feature = "gpu")]
    #[test]
    fn every_listed_unit_of_the_memory_ledger_names_exactly_one_unit_of_the_workspace() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("本 crate 在仓根的 crates/ 下")
            .to_path_buf();
        let switches = RuleSwitches::every_rule_intact();
        let (threads_taken, threads) = workspace_loading_threads_from_the_test_process_share();
        let workspace = load_workspace(&root, threads, &switches).expect("工作区装得起来");
        drop(threads_taken);
        let mut units_of_each_listed_name: std::collections::BTreeMap<String, usize> =
            units_not_followed_of_the_group_as_a_set("gpu_judge")
                .into_iter()
                .map(|name| (name, 0))
                .collect();
        for unit in &workspace.units {
            if let Some(count) = units_of_each_listed_name.get_mut(&unit.qualified_name) {
                *count += 1;
            }
        }
        let not_exactly_one: Vec<(&String, &usize)> = units_of_each_listed_name
            .iter()
            .filter(|(_, count)| **count != 1)
            .collect();
        assert!(
            not_exactly_one.is_empty(),
            "登记的限定名在工作区里不是恰好一个单元的（名字，单元数）：{not_exactly_one:?}"
        );
    }

    /// 不沿登记的单元收，少掉的恰好是登记的那些：登记了而没少掉的（不带登记时本来就不在闭包里，陈旧的登记）红，
    /// 少掉了而没登记的（跟着被切掉、没逐个判过的单元）也红。声明模块的 `mod` 行与 `use` 行不登记（限定名里带行号、一挪就变）：
    /// 它们是因为所在模块的单元都不在了才不进闭包，不算在比对里。
    #[cfg(feature = "gpu")]
    #[test]
    fn not_following_the_listed_units_drops_only_the_listed_ones() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("本 crate 在仓根的 crates/ 下")
            .to_path_buf();
        let listed = units_not_followed_of_the_group_as_a_set("gpu_judge");
        let (following_everything, _) =
            gpu_judge_composition_under(&root, &std::collections::BTreeSet::new());
        let (not_following_the_listed, _) = gpu_judge_composition_under(&root, &listed);
        let kept: std::collections::BTreeSet<&String> = not_following_the_listed.iter().collect();
        let dropped: std::collections::BTreeSet<String> = following_everything
            .iter()
            .filter(|line| !kept.contains(line))
            .filter(|line| !line.starts_with("mod\t") && !line.starts_with("use\t"))
            .map(|line| qualified_name_of_the_composition_line(line).to_string())
            .collect();
        let dropped_and_not_listed: Vec<&String> = dropped.difference(&listed).collect();
        let listed_and_not_dropped: Vec<&String> = listed.difference(&dropped).collect();
        assert!(
            dropped_and_not_listed.is_empty() && listed_and_not_dropped.is_empty(),
            "少掉而没登记的（逐个判过再登记）：{dropped_and_not_listed:?}；登记了而没少掉的（陈旧的登记，删掉那一行）：{listed_and_not_dropped:?}"
        );
    }

    /// 包装设备的值到不了这一组的入口，前提之一：带登记收出来的闭包里没有单元写到这些包装类型的名字
    /// （登记的 impl 头的自身类型，`<impl … for 名字<…>>` 与 `<impl 名字<…>>` 里那个名字）。有单元写了，就可能造出这种值、
    /// 调到登记了不沿它收的 impl：当场红，由人判是把那个 impl 从登记里拿掉还是改代码。
    #[cfg(feature = "gpu")]
    #[test]
    fn no_unit_the_gpu_judge_group_keeps_names_a_wrapper_type_it_does_not_follow() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("本 crate 在仓根的 crates/ 下")
            .to_path_buf();
        let listed = units_not_followed_of_the_group_as_a_set("gpu_judge");
        let wrapper_type_names: std::collections::BTreeSet<String> =
            crate::judging_units_entries::UNITS_REACHED_ONLY_THROUGH_WRAPPER_DEVICES_THE_GPU_JUDGE_GROUP_DOES_NOT_FOLLOW
                .iter()
                .filter_map(|unit| {
                    let head = &unit[unit.find("<impl ")? + "<impl ".len()..];
                    let self_type = head.rsplit(" for ").next().expect("rsplit 至少交一段");
                    let name: String = self_type
                        .trim_start_matches(['&', ' '])
                        .chars()
                        .take_while(|character| character.is_alphanumeric() || *character == '_')
                        .collect();
                    (!name.is_empty()).then_some(name)
                })
                .collect();
        assert!(
            wrapper_type_names.contains("FaultInjectingBlockDevice"),
            "对照：登记里认得出故障注入设备这个包装类型（认不出的话下面什么都没判）：{wrapper_type_names:?}"
        );
        let (kept, _) = gpu_judge_composition_under(&root, &listed);
        let mut units_naming_a_wrapper_type = Vec::new();
        for line in &kept {
            // `use` 行里列着这个名字只是把它引进作用域（同一条 use 还引进别的、在用的名字），不是写出这个类型的值
            if line.starts_with("use\t") || line.starts_with("mod\t") {
                continue;
            }
            let Some(position) = line.split('\t').nth(2) else {
                continue;
            };
            let Some((file, lines)) = position.rsplit_once(':') else {
                continue;
            };
            let Some((first_line, last_line)) = lines.split_once('-') else {
                continue;
            };
            let (Ok(first_line), Ok(last_line)) =
                (first_line.parse::<usize>(), last_line.parse::<usize>())
            else {
                continue;
            };
            let Ok(source) = std::fs::read_to_string(root.join(file)) else {
                continue;
            };
            let text: String = source
                .lines()
                .skip(first_line.saturating_sub(1))
                .take(last_line + 1 - first_line.min(last_line + 1))
                .filter(|source_line| !source_line.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            for word in
                text.split(|character: char| !(character.is_alphanumeric() || character == '_'))
            {
                if wrapper_type_names.contains(word) {
                    units_naming_a_wrapper_type.push(format!(
                        "{} 写了 {word}",
                        qualified_name_of_the_composition_line(line)
                    ));
                    break;
                }
            }
        }
        assert!(
            units_naming_a_wrapper_type.is_empty(),
            "带登记收出来的闭包里有单元写到登记了不沿它收的包装类型：{units_naming_a_wrapper_type:?}"
        );
    }

    /// 改显存记账里登记了不沿它收的一个单元（设备丢失那一行把错误文字接成一行那一处，`text_on_one_line`；派活回读时设备丢了记一笔
    /// 那条真调用边够到它）：GPU 判器那一组的键与镜像判定那一把都不变。
    #[cfg(feature = "gpu")]
    #[test]
    fn an_edit_of_a_listed_unit_of_the_memory_ledger_keeps_the_gpu_judge_key() {
        let (before, after) = keys_before_and_after(
            "memory-ledger-listed-unit",
            MEMORY_LEDGER_FILE,
            ".filter(|piece| !piece.is_empty())",
            ".filter(|piece| !(piece.is_empty()))",
        );
        assert_eq!(
            key_of_the_group(&before, "gpu_judge"),
            key_of_the_group(&after, "gpu_judge"),
            "GPU 判器那一组不变"
        );
        assert_eq!(
            version_of_the_image_judgement(&before),
            version_of_the_image_judgement(&after),
            "镜像判定那一把（块的版本）不变"
        );
    }

    /// 派活宽度那一档的判器一次派活最多带几个状态：比 1 宽、比一个就判完整条流窄，且不整除这条流的状态数（末一次派活不满）。
    #[cfg(feature = "gpu")]
    const DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH: usize = 7;

    /// 派活宽度那一条流的状态数上限：几百个以内的小流，单跑几秒。
    #[cfg(feature = "gpu")]
    const MOST_STATES_OF_THE_DISPATCH_WIDTH_FLOW: u64 = 1000;

    /// 派活宽度那一条流里段长不超过这个数的段逐个真子集展开，更长的段只以整段持久进入后面的状态（状态数压在几百个以内）。
    #[cfg(feature = "gpu")]
    const EXPAND_UP_TO_IN_THE_DISPATCH_WIDTH_FLOW: usize = 3;

    /// 一次派活多宽由调用方定、定宽度的函数（`gpu_memory_ledger::states_in_one_dispatch_capped_by_the_state_count`、
    /// `crate::crash_judge_dispatch::states_in_flight_within` 这几个）不在 GPU 判器那一组闭包里，靠的前提「一次派活带几个状态只是切法，
    /// 不进判定结论」（C1033（两个显存记账单元没有会红的检查））的蜕变断言：新池新建文件那条流（两块稀疏内存盘上 mkfs、取号、暖机、
    /// 新建文件，录 mkfs 之后那一段）在同一张借来的卡上，按一次派活最多 1 个、
    /// [`DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH`] 个与整条流的状态数各建一个判器——前两个照判器的派活宽度调度一次次派、
    /// 最后一个一次派活判完整条流——逐状态的结论字逐字相同，且与 CPU 参照判器逐状态对拍相同。
    /// 先核三个判器开出来的派活上限真是 1、7 与状态数（封顶那一处真起了作用），整条流的结论里至少两种不同的字（派活错位要判得出来）。
    /// 三个判器在同一张卡上依次建、依次判（比的就是同一批状态换一种切法），不并行；借的是三个判器各自要的显存之和。
    #[cfg(feature = "gpu")]
    #[test]
    fn verdict_words_of_every_state_are_the_same_at_a_dispatch_width_of_one_of_seven_and_of_every_state_and_agree_with_the_cpu_judge(
    ) {
        use crate::crash::{crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion};
        use crate::crash_judge_conclusions::{
            cross_check_words_agree_in_the_cross_check, cross_check_words_of_cpu_judgement,
            CrossCheckWords,
        };
        use crate::crash_judge_dispatch::{
            mebibytes_a_judge_needs, GpuCrashJudge, PackedJudgeTables,
        };
        use crate::crash_judge_gpu::{cross_check_words_of_gpu_verdict, VERDICT_WORDS};
        use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
        use crate::crash_library::{judge_tables_through_the_pairing_library, StageOutcome};
        use crate::crash_state_enumeration::{
            sample_crash_states_on_the_cpu, CrashPointReplayItem,
        };
        use crate::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
        use crate::memory_pool::{
            writes_and_segments_with_stream_indexes, PublishedVersion, SparseBlockDevice,
        };
        use crate::segments::FixedGeometry;
        use crate::{RecordingBlockDevice, SharedStream};
        use singlefs_core::address::DeviceIdentity;
        use singlefs_core::block_device::PhysicalBlockSizeInBytes;

        const TEST_NAME: &str = "verdict_words_of_every_state_are_the_same_at_a_dispatch_width_of_one_of_seven_and_of_every_state_and_agree_with_the_cpu_judge";
        const DEVICE_BYTES: u64 = 4 << 30;
        let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
            return;
        };
        let stream = SharedStream::retaining_contents();
        let mut devices = [0, 1].map(|device| {
            (
                DeviceIdentity(device),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device),
                    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        });
        let parameters = crate::known_bad_images::parameters_of_the_common_test_pool();
        let run = crate::scenario::run_new_pool_file_creation(
            &parameters,
            &mut devices,
            &stream,
            |_, _| {},
        )
        .expect("新池新建文件那条路做得成");
        let operations = stream.retained_operations();
        let mut base =
            MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], DEVICE_BYTES);
        base.apply(&operations[..run.mkfs_operation_count]);
        let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
            &operations[run.mkfs_operation_count..],
            &FixedGeometry {
                fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
                journal_ring_bytes: parameters.geometry.journal_ring_bytes,
                root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
            },
        );
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == crate::segments::StepKind::RootRecordFua)
            .expect("新建文件那次发布有一条根槽写");
        let versions = vec![PublishedVersion {
            instance: run.output.root.instance,
            checkpoint_txg: run.output.root.checkpoint_txg,
            content: crate::scenario::first_file_content(),
        }];
        let expansion = |_segment_index: usize, segment: &[usize]| {
            if segment.len() <= EXPAND_UP_TO_IN_THE_DISPATCH_WIDTH_FLOW {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        };
        let tables = build_judge_tables(&JudgeFlow {
            base: &base,
            writes: &writes,
            segments: &segments,
            judged_root_index,
            versions: &versions,
            expansion: &expansion,
        })
        .expect("新池新建文件那条流建得出判表");
        let state_count = tables.state_count;
        assert_eq!(
            state_count,
            crash_point_replay_plan_state_count(&base, &writes, &segments, &expansion),
            "判表的状态数与崩溃点重放的计划相同"
        );
        assert!(
            state_count
                > 2 * u64::try_from(DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH)
                    .expect("装得进 u64")
                && state_count <= MOST_STATES_OF_THE_DISPATCH_WIDTH_FLOW,
            "这条流 {state_count} 个状态：要跨过两次 {DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH} 个的派活，又在 {MOST_STATES_OF_THE_DISPATCH_WIDTH_FLOW} 个以内"
        );
        let every_state = usize::try_from(state_count).expect("装得进 usize");
        // 判表先入库、再读库，GPU 只拿读回的那一份算
        let packed = match judge_tables_through_the_pairing_library(
            TEST_NAME,
            &PackedJudgeTables::of(&tables),
        ) {
            StageOutcome::Judged(packed) => packed,
            StageOutcome::RecordedOnly | StageOutcome::JudgedMyShareOnly { .. } => {
                println!("JUDGE_TABLES_RECORDED_ONLY test={TEST_NAME}");
                return;
            }
        };
        let widths = [
            1,
            DISPATCH_WIDTH_THAT_LEAVES_A_SHORT_LAST_DISPATCH,
            every_state,
        ];
        let Some(lease) = cards.lease_card_or_report_not_run(
            TEST_NAME,
            widths
                .iter()
                .map(|width| mebibytes_a_judge_needs(&packed, *width))
                .sum::<u64>(),
        ) else {
            return;
        };
        let verdicts_at_each_width: Vec<Vec<[u32; VERDICT_WORDS]>> = widths
            .iter()
            .map(|width| {
                let judge =
                    GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, *width);
                assert_eq!(
                    judge.states_per_dispatch(),
                    *width,
                    "要一次派活至多 {width} 个状态的判器开出来的派活上限"
                );
                if *width == every_state {
                    judge.judge_states_in_one_dispatch(0..state_count)
                } else {
                    judge.judge_states(0..state_count)
                }
            })
            .collect();
        drop(lease);
        let in_one_dispatch = &verdicts_at_each_width[widths.len() - 1];
        assert_eq!(
            in_one_dispatch.len(),
            every_state,
            "一次派活判完的结论一个状态一份"
        );
        let distinct_verdicts: std::collections::BTreeSet<&[u32; VERDICT_WORDS]> =
            in_one_dispatch.iter().collect();
        assert!(
            distinct_verdicts.len() >= 2,
            "整条流的结论只有一种字：派活错了位也比不出来"
        );
        for (width, verdicts) in widths.iter().zip(&verdicts_at_each_width) {
            assert_eq!(
                verdicts.len(),
                every_state,
                "一次派活至多 {width} 个时的结论个数"
            );
            let first_difference = verdicts
                .iter()
                .zip(in_one_dispatch)
                .position(|(at_this_width, at_once)| at_this_width != at_once);
            assert_eq!(
                first_difference, None,
                "一次派活至多 {width} 个状态判出的结论字与一次派活判完整条流的不同，第一个不同的状态序号在这里"
            );
        }
        let mut cpu: Vec<CrossCheckWords> = Vec::new();
        let Some(judged_on_the_cpu) = sample_crash_states_on_the_cpu(
            CrashPointReplayItem {
                name: TEST_NAME,
                setting_file: "crates/singlefs-verification/src/crash_identity.rs",
            },
            &base,
            &writes,
            &segments,
            judged_root_index,
            &versions,
            &expansion,
            0..state_count,
            &mut |ordinal, _image, judgement| {
                assert_eq!(
                    ordinal,
                    u64::try_from(cpu.len()).expect("装得进 u64"),
                    "CPU 参照按序号次序交回"
                );
                cpu.push(cross_check_words_of_cpu_judgement(judgement));
            },
        )
        .judged() else {
            return;
        };
        assert_eq!(judged_on_the_cpu, state_count, "CPU 参照判了整条流");
        let disagreements: Vec<String> = cpu
            .iter()
            .zip(in_one_dispatch)
            .enumerate()
            .filter_map(|(ordinal, (cpu_words, gpu_verdict))| {
                let gpu_words = cross_check_words_of_gpu_verdict(gpu_verdict);
                (!cross_check_words_agree_in_the_cross_check(&gpu_words, cpu_words))
                    .then(|| format!("状态 {ordinal}：CPU {cpu_words:?}；GPU {gpu_words:?}"))
            })
            .collect();
        assert!(
            disagreements.is_empty(),
            "{} 个状态 GPU 与 CPU 对拍的字不同（前几个）：\n{}",
            disagreements.len(),
            disagreements
                .iter()
                .take(8)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
        println!(
            "CRASH_JUDGE_DISPATCH_WIDTHS test={TEST_NAME} states={state_count} widths={widths:?} distinct_verdicts={}",
            distinct_verdicts.len()
        );
    }

    /// 照入口登记表在 `root` 那棵树上逐组算键（与构建脚本同一套）：组名 → 键（拒算或不编进来的是 `None`）。
    /// 依赖包的特性取编进去的那份表里的 `build-feature` 行（构建脚本读依赖包经 `links` 交出的那一份）：编验证包的测试时，
    /// 那一趟核心层带着并上来的特性，编进去的键认它，重算这一侧也要认；那几行与核心层实际开着的对不对，另由
    /// `the_dependency_features_compiled_into_the_table_are_the_ones_the_core_layer_was_built_with` 判。
    fn registry_keys_under(root: &Path) -> KeysOfEachGroup {
        let switches = RuleSwitches::every_rule_intact();
        let (threads_taken, threads) = workspace_loading_threads_from_the_test_process_share();
        let workspace = load_workspace(root, threads, &switches).expect("工作区装得起来");
        drop(threads_taken);
        let toolchain_lines = toolchain_lines_under(root);
        let packages = parse_lock_file(
            &std::fs::read_to_string(root.join("Cargo.lock")).expect("Cargo.lock 读得了"),
        )
        .expect("Cargo.lock 解析得了");
        JUDGING_UNITS_ENTRY_GROUPS
            .iter()
            .map(|group| {
                (
                    group.group_name.to_string(),
                    group.target,
                    group.entry_functions,
                )
            })
            .chain(OBSERVER_ENTRIES.iter().map(|observer| {
                (
                    observer_group_name(observer.observer_code_file, observer.observer_name),
                    observer.target,
                    observer.entry_functions,
                )
            }))
            .map(|(group_name, target, entry_functions)| {
                let request = EntryGroupRequest {
                    group_name: group_name.clone(),
                    target: TargetSelector::parse(target).expect("登记表里的目标写法认得"),
                    entry_paths: entry_functions
                        .iter()
                        .map(|entry| (*entry).to_string())
                        .collect(),
                    features: features_of_the_group_request(&group_name, &features_of_this_build()),
                };
                let computed = entry_group_key_not_following_the_listed_units(
                    &workspace,
                    &request,
                    &units_not_followed_of_the_group_as_a_set(&group_name),
                    &compiled_judging_units_keys().features_enabled_by_the_build,
                    &toolchain_lines,
                    &packages,
                    &switches,
                )
                .unwrap_or_else(|error| panic!("组 {group_name} 的入口写错了：{error}"))
                .key;
                let key = match computed.verdict {
                    EntryGroupVerdict::Keyed { key } => Some(key.0),
                    EntryGroupVerdict::Refused { .. } | EntryGroupVerdict::NotCompiled { .. } => {
                        None
                    }
                };
                (group_name, key)
            })
            .collect()
    }

    /// 编进二进制的那份表里每一组的键（拒算或不编进来的是 `None`）。
    fn compiled_keys() -> KeysOfEachGroup {
        compiled_judging_units_keys()
            .groups
            .iter()
            .map(|group| {
                let key = match &group.verdict {
                    ReadBackEntryGroupVerdict::Keyed { key } => Some(*key),
                    ReadBackEntryGroupVerdict::Refused { .. }
                    | ReadBackEntryGroupVerdict::NotCompiled { .. } => None,
                };
                (group.group_name.clone(), key)
            })
            .collect()
    }

    /// 构建脚本算键时闭包读到、又不在 `crates/` 底下的数据文件（编进去的那份表的 `watched` 行里，`include_str!` 按清单目录往上读的那几份，
    /// 例：`singlefs_verification::heavy_threads` 读的 `.claude/heavy-thread-default`）：拷树重算时要连它们一起拷，两边才是同一套输入——
    /// 拷漏了，读它的那一组在拷贝上判 `data-file-unreadable` 拒算，键对不上编进去的那一把。根上的 `Cargo.toml`、`Cargo.lock` 另拷，不在这里。
    fn data_files_outside_the_crates_the_build_read() -> Vec<&'static str> {
        compiled_judging_units_keys()
            .watched_files
            .iter()
            .map(String::as_str)
            .filter(|path| {
                !path.starts_with("crates/") && *path != "Cargo.toml" && *path != "Cargo.lock"
            })
            .collect()
    }

    /// 编进去的那份表点名的、`crates/` 之外的数据文件里有登记的每台默认重活线程数那一份：观察者结论拼块那一组的闭包经核对线程数读它，
    /// 拷树重算时漏拷它，那一组在拷贝上拒算。
    /// 判别力（`crates/mutations.tsv`）：拷树只认 `crates/` 底下的文件，这条红。
    #[test]
    fn the_data_files_outside_the_crates_include_the_heavy_thread_registration() {
        assert!(
            data_files_outside_the_crates_the_build_read()
                .contains(&".claude/heavy-thread-default"),
            "构建时读到的 crates 之外的数据文件：{:?}",
            data_files_outside_the_crates_the_build_read()
        );
    }

    /// 构建脚本写进 `OUT_DIR` 的被监视文件摘要表（文件名与 `build.rs` 的 `WATCHED_FILE_DIGESTS_FILE_NAME` 相同）：
    /// 每行「SHA-256 小写十六进制<TAB>相对仓根的路径」，罩着构建脚本 `rerun-if-changed` 的每份文件。
    const COMPILED_WATCHED_FILE_DIGESTS_TEXT: &str =
        include_str!(concat!(env!("OUT_DIR"), "/watched_file_digests.tsv"));

    /// 摘要表（写法见 [`COMPILED_WATCHED_FILE_DIGESTS_TEXT`]）里与 `read_file` 此刻读到的内容摘要不同的路径，读不了（`None`）的也算，
    /// 按表里的次序。
    ///
    /// # Panics
    /// 表里有一行不是「摘要<TAB>路径」两段。
    fn watched_files_changed_since_the_build(
        digests_text: &str,
        read_file: &dyn Fn(&str) -> Option<Vec<u8>>,
    ) -> Vec<String> {
        digests_text
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let Some((digest_at_the_build, path)) = line.split_once('\t') else {
                    panic!("被监视文件摘要表一行不是「摘要<TAB>路径」两段：{line:?}");
                };
                let is_unchanged = read_file(path)
                    .is_some_and(|bytes| sha256_of(&bytes).hexadecimal() == digest_at_the_build);
                (!is_unchanged).then(|| path.to_string())
            })
            .collect()
    }

    /// 摘要表里的文件此刻有改过的、有读不了的，都点名出来；没改的不点名。
    /// 判别力（`crates/mutations.tsv`）：比摘要那一句改成一律当没改，这条红。
    #[test]
    fn watched_files_whose_digest_differs_from_the_build_are_named_and_unchanged_ones_are_not() {
        let unchanged_bytes = b"pub fn unchanged() {}\n".to_vec();
        let digests_text = format!(
            "{}\tcrates/sample/src/lib.rs\n{}\tcrates/sample/src/edited.rs\n{}\tcrates/sample/src/removed.rs\n",
            sha256_of(&unchanged_bytes).hexadecimal(),
            sha256_of(b"pub fn before_the_edit() {}\n").hexadecimal(),
            sha256_of(b"pub fn removed() {}\n").hexadecimal()
        );
        let read_file = |path: &str| match path {
            "crates/sample/src/lib.rs" => Some(unchanged_bytes.clone()),
            "crates/sample/src/edited.rs" => Some(b"pub fn after_the_edit() {}\n".to_vec()),
            _ => None,
        };
        assert_eq!(
            watched_files_changed_since_the_build(&digests_text, &read_file),
            [
                "crates/sample/src/edited.rs",
                "crates/sample/src/removed.rs"
            ],
            "改过的与读不了的点名、没改的不点名"
        );
    }

    /// 编进去的摘要表罩着判器键表点名的每份被监视文件，连同 `Cargo.lock` 与构建脚本自己（构建脚本照 `rerun-if-changed` 那一份写全）。
    /// 判别力（`crates/mutations.tsv`）：构建脚本只写判器键表点名的那几份，这条红在 `Cargo.lock` 那一句。
    #[test]
    fn the_compiled_watched_file_digests_cover_the_lock_file_the_build_script_and_every_watched_file(
    ) {
        let digested: BTreeSet<&str> = COMPILED_WATCHED_FILE_DIGESTS_TEXT
            .lines()
            .filter_map(|line| line.split_once('\t').map(|(_digest, path)| path))
            .collect();
        for path in ["Cargo.lock", "crates/singlefs-verification/build.rs"] {
            assert!(digested.contains(path), "摘要表里有 {path}：{digested:?}");
        }
        let not_digested: Vec<&String> = compiled_judging_units_keys()
            .watched_files
            .iter()
            .filter(|path| !digested.contains(path.as_str()))
            .collect();
        assert!(
            not_digested.is_empty(),
            "判器键表点名的被监视文件都在摘要表里：{not_digested:?}"
        );
    }

    /// 仓根下 `crates/*/Cargo.toml` 的 path 依赖指到的、不在 `crates/` 底下的 crate 目录（相对仓根，例：`prune-server`），顺那几个 crate
    /// 自己的 path 依赖再往下找：工作区装起来时顺 path 依赖收它们，拷树漏了它的清单，`load_workspace` 认不出这个 crate，闭包够到它的组
    /// 在拷贝上少收单元、键对不上编进去的那一把。
    ///
    /// # Panics
    /// 清单读不了、认不出，或 path 依赖指到仓根之外。
    fn crate_directories_outside_the_crates_directory(repository: &Path) -> BTreeSet<PathBuf> {
        let repository = repository.canonicalize().expect("仓根认得出真路径");
        let crates_directory = repository.join("crates");
        let mut manifest_directories: Vec<PathBuf> = std::fs::read_dir(&crates_directory)
            .expect("读得了 crates/")
            .flatten()
            .map(|entry| entry.path())
            .filter(|directory| directory.join("Cargo.toml").is_file())
            .collect();
        let mut outside = BTreeSet::new();
        while let Some(directory) = manifest_directories.pop() {
            let manifest_path = directory.join("Cargo.toml");
            let manifest = parse_crate_manifest(
                &std::fs::read_to_string(&manifest_path)
                    .unwrap_or_else(|error| panic!("读不了 {}：{error}", manifest_path.display())),
            )
            .unwrap_or_else(|error| panic!("认不出 {}：{error:?}", manifest_path.display()));
            for dependency_path in manifest
                .dependencies
                .iter()
                .filter_map(|dependency| dependency.path.as_ref())
            {
                let dependency_directory = directory
                    .join(dependency_path)
                    .canonicalize()
                    .unwrap_or_else(|error| {
                        panic!(
                            "{} 的 path 依赖 {dependency_path} 认不出真路径：{error}",
                            manifest_path.display()
                        )
                    });
                let relative = dependency_directory
                    .strip_prefix(&repository)
                    .unwrap_or_else(|_| {
                        panic!(
                            "{} 的 path 依赖 {dependency_path} 指到仓根之外",
                            manifest_path.display()
                        )
                    })
                    .to_path_buf();
                if !dependency_directory.starts_with(&crates_directory) && outside.insert(relative)
                {
                    manifest_directories.push(dependency_directory);
                }
            }
        }
        outside
    }

    /// 真仓上 `crates/` 之外经 path 依赖收进工作区的 crate 里有判定库服务 prune-server（验证包依赖它）。
    /// 判别力（`crates/mutations.tsv`）：只认 `crates/` 底下的依赖也收进来，这条红。
    #[test]
    fn the_crates_outside_the_crates_directory_include_the_prune_server() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("本 crate 在仓根的 crates/ 下")
            .to_path_buf();
        assert_eq!(
            crate_directories_outside_the_crates_directory(&repository),
            BTreeSet::from([PathBuf::from("prune-server")]),
            "crates/ 之外经 path 依赖收进来的 crate 目录"
        );
    }

    /// 这棵活树（编这个测试二进制的那一份仓）里的 `crates/`、`Cargo.toml`、`Cargo.lock`、`crates/` 之外经 path 依赖收进来的 crate 目录
    /// （[`crate_directories_outside_the_crates_directory`]）与构建时闭包读到的 `crates/` 之外的数据文件
    /// （[`data_files_outside_the_crates_the_build_read`]）拷进一个新的临时目录（离开作用域时删掉）。
    /// 用它的测试断言的就是真仓上的判器登记表（登记的入口都编得进、真仓上改一处哪几组键变），照旧拒算（断言本身关于真仓的留着）；
    /// 仓根只在这里现取，别的测试够不到它。
    struct RepositoryCopy {
        root: crate::crash_library::ExclusiveTemporaryDirectory,
    }

    impl RepositoryCopy {
        fn new(label: &str) -> Self {
            fn copy_directory(from: &Path, to: &Path) {
                std::fs::create_dir_all(to).expect("建得了目录");
                for entry in std::fs::read_dir(from).expect("读得了目录").flatten() {
                    let destination = to.join(entry.file_name());
                    if entry.file_type().expect("读得了类型").is_dir() {
                        if entry.file_name() != "target" {
                            copy_directory(&entry.path(), &destination);
                        }
                    } else {
                        std::fs::copy(entry.path(), &destination).expect("拷得了文件");
                    }
                }
            }
            let root = crate::crash_library::ExclusiveTemporaryDirectory::create(&format!(
                "judging-units-{label}"
            ))
            .expect("建得了一次性临时目录");
            let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .expect("本 crate 在仓根的 crates/ 下")
                .to_path_buf();
            copy_directory(&repository.join("crates"), &root.join("crates"));
            for file in ["Cargo.toml", "Cargo.lock"] {
                std::fs::copy(repository.join(file), root.join(file)).expect("拷得了根文件");
            }
            for crate_directory in crate_directories_outside_the_crates_directory(&repository) {
                copy_directory(
                    &repository.join(&crate_directory),
                    &root.join(&crate_directory),
                );
            }
            for data_file in data_files_outside_the_crates_the_build_read() {
                let destination = root.join(data_file);
                std::fs::create_dir_all(
                    destination
                        .parent()
                        .expect("仓内相对路径拼到临时根底下，总有上一级目录"),
                )
                .expect("建得了数据文件的目录");
                std::fs::copy(repository.join(data_file), &destination).unwrap_or_else(|error| {
                    panic!("拷得了构建时读到的数据文件 {data_file}：{error}")
                });
            }
            Self { root }
        }

        /// 先比摘要：拷贝里被监视的每份文件与编进二进制的摘要表逐份相同，才拿这份拷贝重算的键去比编进去的键；
        /// 对不上的，键算的已经不是编进去的那一份树，报「树在编译后变了」并点名那几份，不往下比键。
        ///
        /// # Panics
        /// 有被监视的文件此刻与编译时的摘要不同或读不了。
        fn assert_the_watched_files_are_the_ones_the_build_read(&self) {
            let changed = watched_files_changed_since_the_build(
                COMPILED_WATCHED_FILE_DIGESTS_TEXT,
                &|path| std::fs::read(self.root.join(path)).ok(),
            );
            assert!(
                changed.is_empty(),
                "树在编译后变了：这几份被监视的文件与编进二进制的摘要对不上（改过或读不了），键不比，重编再跑：{changed:?}"
            );
        }

        /// 一份文件里恰好出现一次的原文换成替换文；不是恰好一次就 panic（锚点腐化不许悄悄放过）。
        fn replace_exactly_once(&self, relative: &str, original: &str, replacement: &str) {
            let path = self.root.join(relative);
            let text = std::fs::read_to_string(&path).expect("读得了");
            assert_eq!(
                text.matches(original).count(),
                1,
                "{relative} 里「{original}」要恰好一处"
            );
            std::fs::write(&path, text.replacen(original, replacement, 1)).expect("写得了");
        }
    }

    /// 在临时拷贝上施加一处改动，交施加前与施加后逐组的键。
    fn keys_before_and_after(
        label: &str,
        relative: &str,
        original: &str,
        replacement: &str,
    ) -> (KeysOfEachGroup, KeysOfEachGroup) {
        let copy = RepositoryCopy::new(label);
        let before = registry_keys_under(&copy.root);
        copy.replace_exactly_once(relative, original, replacement);
        let after = registry_keys_under(&copy.root);
        (before, after)
    }

    /// 这几组在施加前后键都交得出（`Some`），交回键变了的那几组。
    fn groups_whose_key_changed(
        before: &KeysOfEachGroup,
        after: &KeysOfEachGroup,
    ) -> BTreeSet<String> {
        before
            .iter()
            .filter(|(group, key)| key.is_some() && after.get(*group) != Some(*key))
            .map(|(group, _key)| group.clone())
            .collect()
    }

    /// 构建脚本写进二进制的那份表，与这个测试在盘上这棵树上照同一套算法、同一份工具链行重算的逐组相同；编译之后改了树的先比摘要认出来、
    /// 报树变了，不比键（[`RepositoryCopy::assert_the_watched_files_are_the_ones_the_build_read`]）。
    #[test]
    fn the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree() {
        let compiled = compiled_keys();
        let copy = RepositoryCopy::new("compiled-keys");
        copy.assert_the_watched_files_are_the_ones_the_build_read();
        let from_the_tree = registry_keys_under(&copy.root);
        assert_eq!(
            compiled, from_the_tree,
            "编进去的键与盘上这棵树重算的逐组相同"
        );
        assert!(
            compiled
                .get("judgement_of_the_crash_images")
                .is_some_and(Option::is_some),
            "CPU 判器那一组算得出键：{compiled:?}"
        );
    }

    /// 编进去的那份表里依赖包的特性（`build-feature` 行，构建脚本读核心层经 `links` 交出的那一份），与这个测试二进制链进来的核心层
    /// 实际开着的（[`features_the_core_layer_was_compiled_with`]，从核心层编出来的代码里认）逐个相同：只编验证包、不编它的测试时两边都空，
    /// 编验证包的测试（`cargo test -p singlefs-verification`、`cargo test --all`）时两边都是核心层的
    /// `lookup-work-counters`。核心层清单里的特性也要都是认得出的那几个，多了一个没法认的，这一条先红。
    /// 判别力（`crates/mutations.tsv`，编测试的那一行）：构建脚本改回只读本包特性（不读核心层交出的那一份），编测试时这条红。
    #[test]
    fn the_dependency_features_compiled_into_the_table_are_the_ones_the_core_layer_was_built_with()
    {
        // 路径写成一串字面量（不经 `.parent()`）：变异判法的单元键要从字面路径认出读的是哪份文件。
        let core_manifest_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../singlefs-core/Cargo.toml");
        let core_manifest = singlefs_unit_graph::manifest::parse_crate_manifest(
            &std::fs::read_to_string(core_manifest_path).expect("核心层清单读得了"),
        )
        .expect("核心层清单解析得了");
        assert_eq!(
            core_manifest
                .features
                .keys()
                .map(String::as_str)
                .collect::<Vec<&str>>(),
            CORE_LAYER_FEATURES_RECOGNIZED_FROM_ITS_CODE,
            "核心层清单里的特性就是 features_the_core_layer_was_compiled_with 认得出的那几个（多了的要在那里加认法）"
        );
        assert_eq!(
            compiled_judging_units_keys().features_enabled_by_the_build,
            features_the_core_layer_was_compiled_with(),
            "编进去的表里依赖包的特性（构建脚本读到的）与链进来的核心层实际开着的逐个相同"
        );
    }

    /// 判器与核对的闭包里没有流水线的单元（停止表退役之后留下的那条设计约束）；CPU 那几组连卡上那一套（派活、起卡）都没有。
    #[test]
    fn the_judge_groups_reach_no_pipeline_unit_and_the_cpu_groups_no_card_unit() {
        let table = compiled_judging_units_keys();
        let judge_groups = [
            "gpu_judge",
            "judgement_of_the_crash_images",
            "judgement_of_the_given_states",
            "cpu_facts_verifier",
            "gpu_facts_verifier",
        ];
        let cpu_groups = [
            "judgement_of_the_crash_images",
            "judgement_of_the_given_states",
            "cpu_facts_verifier",
        ];
        let mut checked = 0;
        for group in &table.groups {
            if !judge_groups.contains(&group.group_name.as_str()) {
                continue;
            }
            checked += 1;
            let forbidden: Vec<&str> = if cpu_groups.contains(&group.group_name.as_str()) {
                PIPELINE_FILES_THE_JUDGES_MUST_NOT_REACH
                    .iter()
                    .chain(GPU_FILES_ONLY_THE_GPU_GROUPS_REACH)
                    .copied()
                    .collect()
            } else {
                PIPELINE_FILES_THE_JUDGES_MUST_NOT_REACH.to_vec()
            };
            let reached: Vec<String> = group
                .composition
                .iter()
                .filter(|unit| {
                    forbidden
                        .iter()
                        .any(|file| unit.location.starts_with(&format!("{file}:")))
                })
                .map(|unit| format!("{} {} {}", unit.kind, unit.qualified_name, unit.location))
                .collect();
            assert!(
                reached.is_empty(),
                "{} 的闭包里有不许有的单元：{reached:?}",
                group.group_name
            );
        }
        assert!(checked >= 3, "至少 CPU 那三组在表里：{checked}");
    }

    /// U3 阳性对照：判器闭包里的一个常量改了（池级 checker 的载荷校验和偏移），CPU 判镜像与判给定状态那两组的键变。
    #[test]
    fn constant_in_the_judge_closure_changes_the_cpu_judge_keys() {
        let (before, after) = keys_before_and_after(
            "constant-in-the-judge",
            "crates/singlefs-checker/src/lib.rs",
            "const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;",
            "const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 91;",
        );
        let changed = groups_whose_key_changed(&before, &after);
        for group in [
            "judgement_of_the_crash_images",
            "judgement_of_the_given_states",
        ] {
            assert!(
                changed.contains(group),
                "{group} 的键要变：变了的是 {changed:?}"
            );
        }
    }

    /// U3 阴性对照改的那一处：`crash_amplification::run_from_environment` 函数体里那一个表达式（转调带观察者的那一版）。只取调用本身、
    /// 不带缩进与换行，rustfmt 把它排成一行还是几行之前，换成块表达式都编得过；它在那份文件里要恰好一处，由
    /// [`the_anchor_of_the_pipeline_function_edit_is_in_the_pipeline_file_exactly_once`] 在编进来的那一份上钉住。
    const PIPELINE_FUNCTION_EDIT_ANCHOR: &str =
        "run_from_environment_with_observers(flow, flow_name, &[], toolchain, expand_up_to)";

    /// 锚点没腐化：U3 阴性对照要改的那一个表达式在编进这个测试二进制的那一份 `crash_amplification.rs` 里恰好一处（腐化了阴性对照只会在
    /// 读活树、拒算照跑的那一条里才红，这一条在编进来的源码上先红、点名锚点）。
    /// 判别力（`crates/mutations.tsv`）：`run_from_environment` 里那个调用的参数排法一变，这条红。
    #[test]
    fn the_anchor_of_the_pipeline_function_edit_is_in_the_pipeline_file_exactly_once() {
        assert_eq!(
            include_str!("crash_amplification.rs")
                .matches(PIPELINE_FUNCTION_EDIT_ANCHOR)
                .count(),
            1,
            "crash_amplification.rs 里「{PIPELINE_FUNCTION_EDIT_ANCHOR}」要恰好一处：U3 阴性对照改的就是它"
        );
    }

    /// U3 阴性对照：流水线里一个与判器无关的 fn 改了（`crash_amplification::run_from_environment` 的函数体），判器、核对与录入的键都不变。
    #[test]
    fn pipeline_function_changes_no_judge_verifier_or_recording_key() {
        let (before, after) = keys_before_and_after(
            "pipeline-function",
            "crates/singlefs-verification/src/crash_amplification.rs",
            PIPELINE_FUNCTION_EDIT_ANCHOR,
            &format!(
                "{{ let _pipeline_probe = flow_name.len(); {PIPELINE_FUNCTION_EDIT_ANCHOR} }}"
            ),
        );
        let changed = groups_whose_key_changed(&before, &after);
        for group in [
            "gpu_judge",
            "judgement_of_the_crash_images",
            "judgement_of_the_given_states",
            "cpu_facts_verifier",
            "gpu_facts_verifier",
            "recording",
        ] {
            assert!(
                !changed.contains(group),
                "{group} 的键不该变：变了的是 {changed:?}"
            );
        }
        assert!(
            before
                .get("judgement_of_the_crash_images")
                .is_some_and(Option::is_some),
            "改前算得出键（不是两边都拒算才相等）"
        );
    }

    /// 一组的键；拒算或不编进来的直接 panic，点名是哪一组（键两边都是 `None` 时相等，判不出变没变）。
    fn key_of_the_group(keys: &KeysOfEachGroup, group: &str) -> [u8; DIGEST_BYTES] {
        keys.get(group)
            .copied()
            .flatten()
            .unwrap_or_else(|| panic!("{group} 算得出键：{keys:?}"))
    }

    /// 镜像判定的版本（块存在哪一把下、块复用认的那一把）：判定定义那一组与 GPU 判器那一组并成的一把，两条路线取的都是它。
    fn version_of_the_image_judgement(keys: &KeysOfEachGroup) -> [u8; DIGEST_BYTES] {
        *judgement_units_key_with_the_gpu_judge(
            &JudgingUnits::JudgementOfTheCrashImages,
            &JudgingUnitsKey::of_the_parts(
                key_of_the_group(keys, "judgement_of_the_crash_images"),
                Vec::new(),
            ),
            &JudgingUnitsKey::of_the_parts(key_of_the_group(keys, "gpu_judge"), Vec::new()),
        )
        .fingerprint()
    }

    /// CPU 路线判一块那个入口（`crash_judge_cpu_blocks::judge_a_piece_of_a_block_on_the_cpu`）函数体里的一个表达式（红状态的点内序号），
    /// 只在那份文件里出现一次；换成加括号的写法，函数单元的指纹变、行为不变。
    const CPU_ROUTE_BLOCK_JUDGING_EDIT_ANCHOR: &str = "ordinal - ordinals_of_the_point.start";

    /// 场景：CPU 路线判一块的那个函数单元（`crash_judge_cpu_blocks::judge_a_piece_of_a_block_on_the_cpu`，崩溃放量判段没配 GPU 时逐状态判的那一段）
    /// 改一处写法，在临时拷贝上重算各组的键。
    /// 预期：判镜像的定义那一组的键变，GPU 判器那一组不变，两组并成的镜像判定那一把（块的版本，两条路线同一把）跟着变：CPU 路线的代码改了，
    /// 两条路线判过的块一起换版本、现判，不再照旧复用。
    /// 判别力（`crates/mutations.tsv`）：判镜像的定义那一组的入口里拿掉 CPU 路线判一块那个入口，改了它哪一组的键都不变，这一条红。
    #[test]
    fn function_unit_of_the_cpu_route_block_judging_changes_the_version_of_the_image_judgement() {
        let (before, after) = keys_before_and_after(
            "cpu-route-block-judging",
            "crates/singlefs-verification/src/crash_judge_cpu_blocks.rs",
            CPU_ROUTE_BLOCK_JUDGING_EDIT_ANCHOR,
            &format!("({CPU_ROUTE_BLOCK_JUDGING_EDIT_ANCHOR})"),
        );
        assert_ne!(
            key_of_the_group(&before, "judgement_of_the_crash_images"),
            key_of_the_group(&after, "judgement_of_the_crash_images"),
            "改了 CPU 路线判一块的函数单元，判镜像的定义那一组的键变"
        );
        assert_eq!(
            key_of_the_group(&before, "gpu_judge"),
            key_of_the_group(&after, "gpu_judge"),
            "GPU 判器那一组不变"
        );
        assert_ne!(
            version_of_the_image_judgement(&before),
            version_of_the_image_judgement(&after),
            "镜像判定那一把（块的版本）跟着变"
        );
    }

    /// 流水线里调 CPU 路线判一块那个入口的那一处（`crash_amplification::check_blocks_where_until_with_threads` 交给判段的闭包，日志也在那里打），
    /// 只在那份文件里出现一次；换成加括号的写法，调用方的指纹变、调的还是同一个函数。
    const CPU_ROUTE_CALLER_EDIT_ANCHOR: &str = "let judged = judge_a_piece_of_a_block_on_the_cpu(";

    /// 场景：流水线里调 CPU 路线判一块那个入口的调用方（闭包外：它调入口，入口不调它）改一处写法，在临时拷贝上重算各组的键。
    /// 预期：判镜像的定义那一组、GPU 判器那一组都不变，镜像判定那一把（块的版本）也不变：块照旧复用。
    /// 判别力（`crates/mutations.tsv`）：判镜像的定义那一组的入口里混进那个调用方，它的改动进了键，这一条红。
    #[test]
    fn an_edit_of_the_pipeline_calling_the_cpu_route_block_judging_keeps_the_version_of_the_image_judgement(
    ) {
        let (before, after) = keys_before_and_after(
            "cpu-route-caller",
            "crates/singlefs-verification/src/crash_amplification.rs",
            CPU_ROUTE_CALLER_EDIT_ANCHOR,
            "let judged = (judge_a_piece_of_a_block_on_the_cpu)(",
        );
        assert_eq!(
            key_of_the_group(&before, "judgement_of_the_crash_images"),
            key_of_the_group(&after, "judgement_of_the_crash_images"),
            "判镜像的定义那一组不变"
        );
        assert_eq!(
            key_of_the_group(&before, "gpu_judge"),
            key_of_the_group(&after, "gpu_judge"),
            "GPU 判器那一组不变"
        );
        assert_eq!(
            version_of_the_image_judgement(&before),
            version_of_the_image_judgement(&after),
            "镜像判定那一把（块的版本）不变"
        );
    }

    /// 场景：GPU 判器那一组的一个 WGSL 函数单元（`crash_judge_checker_common.wgsl` 的 `violate`）函数体改一处写法，在临时拷贝上重算各组的键。
    /// 预期：施加前后 GPU 判器那一组都算得出键（不论这个测试二进制开没开 `gpu`：那一组照开着 `gpu` 收闭包），它的键变；判定定义那一组不变；
    /// 两组并成的镜像判定那一把（`judgement_units_key_with_the_gpu_judge`，块的版本）跟着变：改一个 GPU 内核里的 fn，判过的块全部换版本、现判
    /// （「GPU 判器代码进键」）。
    /// 判别力（`crates/mutations.tsv`）：并成那一把时不拼 GPU 判器那一把、GPU 判器那一组照这一次的特性收闭包（不开 `gpu` 的编法里交不出键），各让这一条红。
    #[test]
    fn wgsl_function_of_the_gpu_judge_changes_the_version_of_the_image_judgement_but_not_the_definition_key(
    ) {
        let (before, after) = keys_before_and_after(
            "gpu-judge-wgsl-function",
            "crates/singlefs-verification/src/crash_judge_checker_common.wgsl",
            "fn violate(invariant: u32) {\n    let word_index = invariant / 32u;",
            "fn violate(invariant: u32) {\n    let word_index = (invariant) / 32u;",
        );
        let key_of = |keys: &KeysOfEachGroup, group: &str| {
            keys.get(group).copied().flatten().unwrap_or_else(|| {
                panic!("{group} 算得出键（GPU 判器那一组不论开没开 gpu 都照开着收）：{keys:?}")
            })
        };
        assert_ne!(
            key_of(&before, "gpu_judge"),
            key_of(&after, "gpu_judge"),
            "改了 GPU 判器的一个 WGSL 函数，它那一组的键变"
        );
        assert_eq!(
            key_of(&before, "judgement_of_the_crash_images"),
            key_of(&after, "judgement_of_the_crash_images"),
            "判定定义那一组不变"
        );
        let version_of = |keys: &KeysOfEachGroup| {
            *judgement_units_key_with_the_gpu_judge(
                &JudgingUnits::JudgementOfTheCrashImages,
                &JudgingUnitsKey::of_the_parts(
                    key_of(keys, "judgement_of_the_crash_images"),
                    Vec::new(),
                ),
                &JudgingUnitsKey::of_the_parts(key_of(keys, "gpu_judge"), Vec::new()),
            )
            .fingerprint()
        };
        assert_ne!(
            version_of(&before),
            version_of(&after),
            "并成的镜像判定那一把（块的版本）跟着变"
        );
    }

    /// 录入段建 GPU 判表的本体（`crash_judge_tables::build_judge_tables_from_the_state_source`，崩溃放量与给定状态表两个建表入口共用）函数体里的
    /// 一个表达式（枚举写数对掩码宽那一句），只在那份文件里出现一次；换成加括号的写法，函数单元的指纹变、行为不变。
    const JUDGE_TABLE_BUILDING_EDIT_ANCHOR: &str =
        "if enumerated.len() > MAXIMUM_ENUMERATED_WRITES {";

    /// 场景：录入段建 GPU 判表的那个函数单元（判段从库里按摘要读回、GPU 判器只读这一份表，判器读表这一跳走库）改一处写法，在临时拷贝上重算各组的键。
    /// 预期：GPU 判器那一组的键变（建表的单元登在它的入口里），两组并成的镜像判定那一把（块的版本，两条路线同一把）跟着变：改了建表，
    /// GPU 与 CPU 判过的块一起换版本、现判，不再照旧复用；判镜像的定义那一组不变（CPU 路线不读判表）。
    /// 判别力（`crates/mutations.tsv`）：「判器键：GPU 判器那一组的入口里拿掉建表的入口」这一行让这一条红。
    #[test]
    fn function_unit_building_the_gpu_judge_tables_changes_the_gpu_judge_key_and_the_version_of_the_image_judgement(
    ) {
        let (before, after) = keys_before_and_after(
            "gpu-judge-table-building",
            "crates/singlefs-verification/src/crash_judge_tables.rs",
            JUDGE_TABLE_BUILDING_EDIT_ANCHOR,
            "if (enumerated.len()) > MAXIMUM_ENUMERATED_WRITES {",
        );
        assert_ne!(
            key_of_the_group(&before, "gpu_judge"),
            key_of_the_group(&after, "gpu_judge"),
            "改了建 GPU 判表的函数单元，GPU 判器那一组的键变"
        );
        assert_eq!(
            key_of_the_group(&before, "judgement_of_the_crash_images"),
            key_of_the_group(&after, "judgement_of_the_crash_images"),
            "判镜像的定义那一组不变"
        );
        assert_ne!(
            version_of_the_image_judgement(&before),
            version_of_the_image_judgement(&after),
            "镜像判定那一把（块的版本）跟着变"
        );
    }

    /// 流水线里调建表入口的那一处（`crash_amplification::packed_judge_tables_of_the_plan`，录入段建表、打包的调用方），只在那份文件里出现一次；
    /// 换成加括号的写法，调用方的指纹变、调的还是同一个函数。
    const JUDGE_TABLE_BUILDING_CALLER_EDIT_ANCHOR: &str =
        "crate::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices(";

    /// 场景：流水线里调建表入口的调用方（闭包外：它调入口，入口不调它）改一处写法，在临时拷贝上重算各组的键。
    /// 预期：GPU 判器那一组、判镜像的定义那一组都不变，镜像判定那一把也不变：与判表怎么建无关的改动不让块换版本。
    /// 判别力（`crates/mutations.tsv`）：「判器键：GPU 判器那一组的入口里混进建表的调用方」这一行让这一条红。
    #[test]
    fn an_edit_of_the_pipeline_calling_the_judge_table_building_keeps_the_gpu_judge_key() {
        let (before, after) = keys_before_and_after(
            "gpu-judge-table-building-caller",
            "crates/singlefs-verification/src/crash_amplification.rs",
            JUDGE_TABLE_BUILDING_CALLER_EDIT_ANCHOR,
            "(crate::crash_judge_tables::build_judge_tables_against_the_version_table_with_slices)(",
        );
        assert_eq!(
            key_of_the_group(&before, "gpu_judge"),
            key_of_the_group(&after, "gpu_judge"),
            "GPU 判器那一组不变"
        );
        assert_eq!(
            version_of_the_image_judgement(&before),
            version_of_the_image_judgement(&after),
            "镜像判定那一把（块的版本）不变"
        );
    }

    /// GPU 判器那一组在 `root` 那棵树上照明写的特性收闭包算的键：这个测试二进制开了的特性（[`features_of_this_build`]）另并上
    /// [`GPU_FEATURE`]，按名字排好、不重。特性在这里自己并，不经 `features_of_the_group_request`：构建脚本取特性也经那一处，
    /// 拿它重算会跟着编进去的那一把一起改坏、照样相等。拒算或不编进来的交 `None`。
    fn gpu_judge_key_collected_with_the_gpu_feature_under(
        root: &Path,
    ) -> Option<[u8; DIGEST_BYTES]> {
        let switches = RuleSwitches::every_rule_intact();
        let (threads_taken, threads) = workspace_loading_threads_from_the_test_process_share();
        let workspace = load_workspace(root, threads, &switches).expect("工作区装得起来");
        drop(threads_taken);
        let packages = parse_lock_file(
            &std::fs::read_to_string(root.join("Cargo.lock")).expect("Cargo.lock 读得了"),
        )
        .expect("Cargo.lock 解析得了");
        let gpu_judge_group = JUDGING_UNITS_ENTRY_GROUPS
            .iter()
            .find(|group| group.group_name == JudgingUnits::GpuJudge.group_name())
            .expect("入口登记表里有 GPU 判器那一组");
        let mut features_with_the_gpu_feature: BTreeSet<String> =
            features_of_this_build().into_iter().collect();
        features_with_the_gpu_feature.insert(GPU_FEATURE.to_string());
        let request = EntryGroupRequest {
            group_name: gpu_judge_group.group_name.to_string(),
            target: TargetSelector::parse(gpu_judge_group.target).expect("登记表里的目标写法认得"),
            entry_paths: gpu_judge_group
                .entry_functions
                .iter()
                .map(|entry| (*entry).to_string())
                .collect(),
            features: features_with_the_gpu_feature.into_iter().collect(),
        };
        let features_enabled_on_the_dependencies =
            &compiled_judging_units_keys().features_enabled_by_the_build;
        let computed = entry_group_key_not_following_the_listed_units(
            &workspace,
            &request,
            &units_not_followed_of_the_group_as_a_set("gpu_judge"),
            features_enabled_on_the_dependencies,
            &toolchain_lines_under(root),
            &packages,
            &switches,
        )
        .unwrap_or_else(|error| panic!("GPU 判器那一组的入口写错了：{error}"))
        .key;
        match computed.verdict {
            EntryGroupVerdict::Keyed { key } => Some(key.0),
            EntryGroupVerdict::Refused { .. } | EntryGroupVerdict::NotCompiled { .. } => None,
        }
    }

    /// 编进这个测试二进制的那份表里 GPU 判器那一组的键，就是照开着 `gpu` 收闭包算的那一把（不论这一次开没开 `gpu`；对照的那一把由
    /// [`gpu_judge_key_collected_with_the_gpu_feature_under`] 另起一路算），镜像判定那一把（[`image_judgement_units_key`]）取得出、
    /// 与两组并成的那一把逐字相同，组成清单是两组的单元各带组名起头。
    /// 不开 `gpu` 编时，GPU 判器那一组照这一次的特性收闭包也算得出键：派活与内核那几个入口不编进来、被去掉，`crash_judge_conclusions`
    /// 里的入口还在，键照剩下的算，只看「算得出键」分不出两种收法，要与照开着 `gpu` 收的那一把比。
    /// 判别力（`crates/mutations.tsv`）：构建脚本给 GPU 判器那一组照这一次的特性收闭包，不开 `gpu` 编时这条红在对照那一句。
    #[test]
    fn the_image_judgement_units_key_merges_the_definition_and_the_gpu_judge_in_either_build() {
        let toolchain = CrashPointReplayToolchainIdentity {
            rustc_version_lines: String::new(),
            cargo_version: String::new(),
            target_triple: String::new(),
        };
        let definition = judging_units_key(&JudgingUnits::JudgementOfTheCrashImages, &toolchain)
            .expect("判定定义那一组编进了这一份二进制");
        let gpu_judge = judging_units_key(&JudgingUnits::GpuJudge, &toolchain)
            .expect("GPU 判器那一组在编进去的表里算得出键");
        let copy = RepositoryCopy::new("gpu-judge-collected-with-the-gpu-feature");
        copy.assert_the_watched_files_are_the_ones_the_build_read();
        assert_eq!(
            Some(*gpu_judge.fingerprint()),
            gpu_judge_key_collected_with_the_gpu_feature_under(&copy.root),
            "编进去的 GPU 判器那一把就是照开着 gpu 收闭包算的那一把（不开 gpu 编的这一份也是）"
        );
        let merged = image_judgement_units_key(&toolchain).expect("两组都算得出");
        assert_eq!(
            merged,
            judgement_units_key_with_the_gpu_judge(
                &JudgingUnits::JudgementOfTheCrashImages,
                &definition,
                &gpu_judge
            )
        );
        assert_ne!(merged.fingerprint(), definition.fingerprint());
        assert_eq!(
            merged.composition().len(),
            definition.composition().len() + gpu_judge.composition().len(),
            "组成清单是两组的单元"
        );
        assert!(
            merged.composition().iter().all(|unit| unit
                .unit
                .starts_with("judgement_of_the_crash_images: ")
                || unit.unit.starts_with("gpu_judge: ")),
            "每个单元带组名起头"
        );
    }

    /// controls 路 S12：派活回读归位（`judge_one_dispatch` 把结论字放回哪个状态那一处）改一处写法，GPU 判器那一组的键变——派活里决定结论
    /// 归位的函数在 GPU 判器的入口集合里（底稿 L7）。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_readback_placement_in_the_dispatch_changes_the_gpu_judge_key() {
        let (before, after) = keys_before_and_after(
            "dispatch-readback",
            "crates/singlefs-verification/src/crash_judge_dispatch.rs",
            "entries[state * VERDICT_WORDS..(state + 1) * VERDICT_WORDS]",
            "entries[(state) * VERDICT_WORDS..(state + 1) * VERDICT_WORDS]",
        );
        let changed = groups_whose_key_changed(&before, &after);
        assert!(changed.contains("gpu_judge"), "变了的是 {changed:?}");
    }

    /// 一次派活带几个状态的调度（量宽度的停止判据 `calibration_goes_on`）改一处写法：GPU 判器那一组的键不变，镜像判定那一把（块的版本）也不变——
    /// 调度只管把一段序号切成几次派活，切法不进结论，不在 GPU 判器的入口集合里；改了它判过的块照旧复用。
    /// 判别力（`crates/mutations.tsv`）：入口换回调度上面那一层 `GpuCrashJudge::judge_states`，调度进了闭包，这一条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn an_edit_of_the_dispatch_width_schedule_keeps_the_gpu_judge_key() {
        let (before, after) = keys_before_and_after(
            "dispatch-width-schedule",
            "crates/singlefs-verification/src/crash_judge_dispatch.rs",
            "seconds / width as f64 <= best_earlier * CALIBRATION_WORSE_THAN_THE_BEST",
            "(seconds / width as f64) <= best_earlier * CALIBRATION_WORSE_THAN_THE_BEST",
        );
        assert_eq!(
            key_of_the_group(&before, "gpu_judge"),
            key_of_the_group(&after, "gpu_judge"),
            "GPU 判器那一组不变"
        );
        assert_eq!(
            version_of_the_image_judgement(&before),
            version_of_the_image_judgement(&after),
            "镜像判定那一把（块的版本）不变"
        );
    }

    /// 定一次派活多宽、量显存额度的单元（限定名的末段）：判器照调用方给的宽度开草稿区，这几个不该在 GPU 判器那一组的闭包里。
    #[cfg(feature = "gpu")]
    const UNITS_SETTING_THE_DISPATCH_WIDTH: &[&str] = &[
        "MAXIMUM_SCRATCH_LANES",
        "MEBIBYTES_LEFT_FOR_THE_DRIVER",
        "states_in_flight_within",
        "states_per_dispatch_within_the_scratch_memory",
        "starting_narrow",
        "of_lanes_holding",
        "within_the_memory_budget_of_the_card",
        "states_in_one_dispatch_capped_by_the_state_count",
    ];

    /// 场景：GPU 判器那一组在这棵树上带登记收闭包，逐行看组成清单里单元限定名的末段。
    /// 预期：[`UNITS_SETTING_THE_DISPATCH_WIDTH`] 一个都不在；对照：上卡的入口 `on_card_with_the_given_dispatch_width` 在（末段认得出）。
    /// 判别力（`crates/mutations.tsv`）：入口登记换回照显存额度自己定宽度的 `GpuCrashJudge::on_card_compiling_these_kernel_sources`，这一条红。
    #[cfg(feature = "gpu")]
    #[test]
    fn the_gpu_judge_group_reaches_no_unit_setting_the_dispatch_width() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("本 crate 在仓根的 crates/ 下")
            .to_path_buf();
        let listed = units_not_followed_of_the_group_as_a_set("gpu_judge");
        let (composition, _) = gpu_judge_composition_under(&root, &listed);
        let last_segments: Vec<&str> = composition
            .iter()
            .filter(|line| !line.starts_with("use\t") && !line.starts_with("mod\t"))
            .map(|line| {
                qualified_name_of_the_composition_line(line)
                    .rsplit("::")
                    .next()
                    .expect("rsplit 至少交一段")
            })
            .collect();
        assert!(
            last_segments.contains(&"on_card_with_the_given_dispatch_width"),
            "对照：组成清单里认得出上卡的入口（认不出的话下面什么都没判）"
        );
        let setting_the_width: Vec<&&str> = last_segments
            .iter()
            .filter(|segment| UNITS_SETTING_THE_DISPATCH_WIDTH.contains(*segment))
            .collect();
        assert!(
            setting_the_width.is_empty(),
            "GPU 判器那一组的闭包里有定派活宽度、量显存额度的单元：{setting_the_width:?}"
        );
    }

    /// 场景：留给驱动的显存（`crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER`）在临时拷贝上改一个数。
    /// 预期：GPU 判器那一组的键、镜像判定那一把（块的版本）与观察者结论拼块那一组的键都不变——判器照调用方给的宽度开草稿区，
    /// 拼块那一组的 GPU 入口照调用方交进来的那一下上卡判（`crate::crash_amplification` 里照卡的来路排队、定宽度的那一层不在它的闭包里），
    /// 量显存额度的常量不在这几组的闭包里，改了它判过、拼过的块照旧复用。
    /// 判别力（`crates/mutations.tsv`）：拼块那一组的入口登记加上照卡的来路排队、量显存额度的那一层，这一条红在拼块那一句。
    #[cfg(feature = "gpu")]
    #[test]
    fn an_edit_of_the_card_memory_left_for_the_driver_keeps_the_gpu_judge_and_the_observer_composition_keys(
    ) {
        let (before, after) = keys_before_and_after(
            "card-memory-left-for-the-driver",
            "crates/singlefs-verification/src/crash_judge_dispatch.rs",
            "pub const MEBIBYTES_LEFT_FOR_THE_DRIVER: u64 = 2560;",
            "pub const MEBIBYTES_LEFT_FOR_THE_DRIVER: u64 = 2561;",
        );
        assert_eq!(
            key_of_the_group(&before, "gpu_judge"),
            key_of_the_group(&after, "gpu_judge"),
            "GPU 判器那一组不变"
        );
        assert_eq!(
            version_of_the_image_judgement(&before),
            version_of_the_image_judgement(&after),
            "镜像判定那一把（块的版本）不变"
        );
        assert_eq!(
            key_of_the_group(&before, JudgingUnits::ObserverConclusionComposition.name()),
            key_of_the_group(&after, JudgingUnits::ObserverConclusionComposition.name()),
            "观察者结论拼块那一组不变"
        );
    }

    /// 失败点码表的条目级宏调用改一个码（9 → 99）：GPU 判器那一组的键变（WGSL 外名 `FAILURE_POINT_*` 的着落做成边，宏调用进闭包）。
    #[cfg(feature = "gpu")]
    #[test]
    fn failure_point_code_changes_the_gpu_judge_key() {
        let (before, after) = keys_before_and_after(
            "failure-point-code",
            "crates/singlefs-verification/src/crash_judge_conclusions.rs",
            "    GPU_FAILURE_POINT_UNIT_UNREADABLE = 9,\n",
            "    GPU_FAILURE_POINT_UNIT_UNREADABLE = 99,\n",
        );
        let changed = groups_whose_key_changed(&before, &after);
        assert!(changed.contains("gpu_judge"), "变了的是 {changed:?}");
    }

    /// controls 路 S16：这一趟用的 profile 的 `overflow-checks` 改了（工具链行里 cargo 解析后的 profile 生效值进键），算得出键的每一组都变。
    #[test]
    fn overflow_checks_of_the_profile_in_use_change_every_key() {
        let profile_of_this_build = compiled_judging_units_keys()
            .toolchain_lines
            .iter()
            .find_map(|line| line.strip_prefix("profile: "))
            .expect("编进去的工具链行里有 profile 那一行")
            .to_string();
        let copy = RepositoryCopy::new("overflow-checks");
        let before = registry_keys_under(&copy.root);
        if profile_of_this_build == "release" {
            copy.replace_exactly_once(
                "Cargo.toml",
                "overflow-checks = true",
                "overflow-checks = false",
            );
        } else {
            let manifest = copy.root.join("Cargo.toml");
            let mut text = std::fs::read_to_string(&manifest).expect("读得了");
            text.push_str(&format!(
                "\n[profile.{profile_of_this_build}]\noverflow-checks = false\n"
            ));
            std::fs::write(&manifest, text).expect("写得了");
        }
        let after = registry_keys_under(&copy.root);
        let keyed: BTreeSet<String> = before
            .iter()
            .filter(|(_group, key)| key.is_some())
            .map(|(group, _key)| group.clone())
            .collect();
        assert!(!keyed.is_empty(), "改前有算得出键的组");
        assert_eq!(
            groups_whose_key_changed(&before, &after),
            keyed,
            "每一组都变"
        );
    }

    /// 同一份测试文件里的两个观察者各自一把键（身份是各自入口 fn 的闭包键，不是文件）；只改其中一个的函数体，只那一把变。
    #[cfg(feature = "gpu")]
    #[test]
    fn two_observers_in_one_file_have_two_keys_and_editing_one_moves_only_its_key() {
        let file =
            "crates/singlefs-verification/tests/crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs";
        let handing_back = observer_group_name(file, "crash_image_handed_back");
        let rewriting =
            observer_group_name(file, "system_configuration_slot_rewritten_past_the_limit");
        let compiled = compiled_keys();
        let handing_back_key = compiled
            .get(&handing_back)
            .copied()
            .flatten()
            .expect("交回崩溃镜像那个观察者算得出键");
        let rewriting_key = compiled
            .get(&rewriting)
            .copied()
            .flatten()
            .expect("重写系统配置槽那个观察者算得出键");
        assert_ne!(
            handing_back_key, rewriting_key,
            "同一文件里的两个观察者两把键"
        );
        let (before, after) = keys_before_and_after(
            "one-observer-body",
            file,
            "writes_on_the_crash_image: Vec::new(),",
            "writes_on_the_crash_image: Vec::with_capacity(0),",
        );
        let changed = groups_whose_key_changed(&before, &after);
        assert!(
            changed.contains(&handing_back),
            "改了它的函数体，它的键变：{changed:?}"
        );
        assert!(
            !changed.contains(&rewriting),
            "另一个观察者的键不变：{changed:?}"
        );
    }

    /// 没登记入口 fn 的观察者取键拒算（照判、不入库），红句点名组与形态。
    #[test]
    fn an_observer_without_a_registered_entry_is_refused_as_not_computed_at_the_build() {
        let refused = judging_units_key(
            &JudgingUnits::ObserverCode {
                observer_code_file: "crates/singlefs-verification/tests/nowhere.rs".to_string(),
                observer_name: "unregistered".to_string(),
            },
            &CrashPointReplayToolchainIdentity {
                rustc_version_lines: String::new(),
                cargo_version: String::new(),
                target_triple: String::new(),
            },
        )
        .expect_err("没登记就拒算");
        assert_eq!(refused.form_name(), "not_computed_at_the_build");
        assert!(
            refused.sentence().contains(
                "observer_code:crates/singlefs-verification/tests/nowhere.rs:unregistered"
            ),
            "{}",
            refused.sentence()
        );
    }

    /// 组成清单写成文字再读回逐行相同；两把清单比下来点名指纹不同与只有一边有的单元；红句以「变了的单元：」起头、接清单，
    /// 两边都空时写明比不出；编译时拒算的那一种红句带拒算行。
    #[test]
    fn the_composition_reads_back_and_the_red_sentence_names_the_changed_units() {
        let unit = |name: &str, byte: u8| JudgingUnitFingerprint {
            unit: name.to_string(),
            fingerprint: [byte; DIGEST_BYTES],
        };
        let stored = JudgingUnitsKey::of_the_parts(
            [1; DIGEST_BYTES],
            vec![unit("recover", 1), unit("check_unit", 2), unit("walk", 3)],
        );
        assert_eq!(
            JudgingUnitsKey::composition_from_text(&stored.composition_text()),
            Some({
                let mut sorted = stored.composition().to_vec();
                sorted.sort();
                sorted
            }),
            "组成清单读得回"
        );
        assert_eq!(
            JudgingUnitsKey::composition_from_text("recover\tzz\n"),
            None
        );
        let now = vec![unit("recover", 1), unit("check_unit", 9), unit("crc32", 4)];
        let changed = units_changed_between(stored.composition(), &now);
        assert_eq!(changed, vec!["check_unit", "crc32", "walk"]);
        let sentence = changed_units_sentence(&changed);
        assert!(
            sentence.starts_with("变了的单元：") && sentence.contains("check_unit，crc32，walk"),
            "{sentence}"
        );
        assert!(
            changed_units_sentence(&units_changed_between(&[], &[])).starts_with("变了的单元："),
            "组成清单是空表也照打这一截"
        );
        let refused = JudgingUnitsKeyRefused::RefusedAtTheBuild {
            units: JudgingUnits::GpuJudge,
            refusal_lines: vec![
                "refused\tcurrent-exe\tcrates/a/src/lib.rs:3\tcurrent_exe".to_string()
            ],
        };
        assert!(
            refused.sentence().contains("refused_at_the_build")
                && refused
                    .sentence()
                    .contains("current-exe crates/a/src/lib.rs:3"),
            "{}",
            refused.sentence()
        );
    }

    /// 编进去的组成清单换成组成清单行：单元写种类、限定名与文件，不带行号；同一份文件里种类与限定名都相同的几个按出现次序另加序号。
    #[test]
    fn the_compiled_composition_names_units_without_line_numbers_and_tells_repeated_units_apart() {
        let unit = |qualified_name: &str, location: &str, byte: u8| {
            singlefs_unit_graph::entry_groups::CompositionUnit {
                kind: "use".to_string(),
                qualified_name: qualified_name.to_string(),
                location: location.to_string(),
                fingerprint: [byte; DIGEST_BYTES],
            }
        };
        let group = ReadBackEntryGroup {
            group_name: "gpu_judge".to_string(),
            verdict: ReadBackEntryGroupVerdict::Keyed {
                key: [7; DIGEST_BYTES],
            },
            entries_not_compiled: Vec::new(),
            composition: vec![
                unit("crate_a::module_a", "crates/a/src/lib.rs:3-4", 1),
                unit("crate_a::module_a", "crates/a/src/lib.rs:9-9", 2),
            ],
        };
        let names: Vec<String> = composition_of_the_compiled_group(&group)
            .into_iter()
            .map(|composed| composed.unit)
            .collect();
        assert_eq!(
            names,
            [
                "use crate_a::module_a @ crates/a/src/lib.rs",
                "use crate_a::module_a @ crates/a/src/lib.rs #2",
            ]
        );
    }
}
