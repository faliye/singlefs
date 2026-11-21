//! 观察者的定义：观察者看一个崩溃状态的那个函数、它要读的范围、挂在哪几个点上（[`ObserverOnTheNodes`]），
//! 与观察者结论的版本（[`ObserverVersions`]）。

use super::*;

// ============================== 挂在节点上的观察者（两段与按恢复镜像去重） ==============================
//
// （原话在 `records/2026-10-13-复用键与全仓改名.md`「课题 4」）：观察者改走两段、池级 checker 上 GPU、按恢复结果去重。
// 录入段（[`record_the_observers_of_the_flow`]）在 CPU 上逐状态跑观察者，结果写进库：只记计数的那一种逐状态看（它可以读持久集合的结构），
// 交回镜像的那一种（可写挂载、续做）按恢复镜像的字节去重，同一张镜像只看一次。判段（[`judge_the_observers_of_the_flow`]）只从库里读，
// 对观察者交回的镜像跑池级 checker（配了 GPU 在卡上判，同一张镜像只判一次），逐状态拼回一块结论存进库；判段不跑恢复、不调观察者。

/// 只记计数的观察者看一个崩溃状态：镜像、看 journal 那一遍恢复的报告、往观察者计数里记；判不对就 panic（与崩溃点重放的逐状态观察者同一个形，
/// `crate::crash::CrashStateObserving`）。录入段在核对线程上跑，几条线程同时看不同的状态；逐状态看，不去重（它可以读持久集合的结构）。
pub type ObserveACrashState<'observer> = dyn Fn(u64, &CrashImage<'_>, &RecoveryReport, &mut CrashPointReplayObserverCounts)
    + Sync
    + 'observer;

/// 交回镜像的观察者看一个崩溃状态：在这张崩溃镜像上真跑实现（可写挂载、续做），记计数、判不对就 panic，交回它在镜像上发的写、它停下时的盘
/// 与它在镜像上读过的段（[`ImageAfterTheObserver`]）；那张盘上的池级 checker 不在这里跑，判段从库里读回来判（配了 GPU 在卡上判）。
/// 它交回的只许随它读到的字节、恢复报告（镜像的函数）与它另读的输入变，而且要确定（同一张镜像跑两次读同样的段、发同样的写）：
/// 录入段按恢复报告分桶、按读集见证去重（[`ObserverDeduplication`]），同一桶里读集字节相同的状态只看一次，别的状态取那一次的结果。
/// 它读镜像只许经录制设备（`singlefs_verification::RecordingBlockDevice`，开了记读的流），读过的段一段不漏地交回：漏了一段，读集相同的两个
/// 状态可能在漏的那一段上字节不同、却被合成一张。
pub type ObserveACrashStateHandingBackAnImage<'observer> = dyn Fn(
        u64,
        &CrashImage<'_>,
        &RecoveryReport,
        &mut CrashPointReplayObserverCounts,
    ) -> ImageAfterTheObserver
    + Sync
    + 'observer;

/// 交回镜像的观察者看完一个状态交回的：它在崩溃镜像上发的写（按发的次序；录入段把它们叠回崩溃镜像，与 `disks_after_the_observer` 逐字节核过
/// 才存）、它在崩溃镜像上读过的段（录入段按它算读集见证）、判段在那张盘上判哪几条不变量、判段判干净时给哪个计数加一。
pub struct ImageAfterTheObserver {
    pub writes_on_the_crash_image: Vec<RetainedWrite>,
    pub disks_after_the_observer: MemoryPool,
    pub ranges_read_on_the_crash_image: Vec<ReadRangeOnTheCrashImage>,
    pub invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver,
    pub counter_when_the_pool_checker_is_clean: String,
}

/// 交回镜像的观察者在崩溃镜像上读过的一段：盘、起扇区、止扇区（半开）。录制设备记下的每一次读与每一次 journal 记录槽提示问到的段按扇区对齐
/// 之后合并成这样几段（[`ReadRangeOnTheCrashImage::of_the_recorded_read_ranges`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadRangeOnTheCrashImage {
    pub device: u32,
    pub first_sector: u64,
    pub end_sector: u64,
}

impl ReadRangeOnTheCrashImage {
    /// 录制设备记下的读（字节偏移与长度）按扇区对齐、按盘与起扇区排、相邻或相交的并成一段。
    #[must_use]
    pub fn of_the_recorded_read_ranges(ranges: &[crate::RecordedReadRange]) -> Vec<Self> {
        let sector_bytes = crate::memory_pool::SECTOR_BYTES;
        let mut aligned: Vec<Self> = ranges
            .iter()
            .filter(|range| range.length > 0)
            .map(|range| Self {
                device: range.device.0,
                first_sector: range.offset.0 / sector_bytes,
                end_sector: (range.offset.0 + range.length).div_ceil(sector_bytes),
            })
            .collect();
        aligned.sort_unstable();
        let mut merged: Vec<Self> = Vec::with_capacity(aligned.len());
        for range in aligned {
            match merged.last_mut() {
                Some(last)
                    if last.device == range.device && range.first_sector <= last.end_sector =>
                {
                    last.end_sector = last.end_sector.max(range.end_sector);
                }
                _ => merged.push(range),
            }
        }
        merged
    }

    /// 整个池的每块盘从头到尾：读了整张镜像（把整张崩溃镜像当盘交回去判的观察者）照实交回这一份，它只与镜像逐字节相同的状态合并。
    #[must_use]
    pub fn whole_pool(pool: &MemoryPool) -> Vec<Self> {
        let sectors = pool
            .device_size_in_bytes
            .div_ceil(crate::memory_pool::SECTOR_BYTES);
        pool.devices
            .keys()
            .map(|identity| Self {
                device: identity.0,
                first_sector: 0,
                end_sector: sectors,
            })
            .collect()
    }
}

/// 观察者交回的几段按盘与起扇区排、相邻或相交的并成一段（见证的段是集合，不看观察者交回的次序）。
pub(super) fn normalized_read_ranges(
    ranges: &[ReadRangeOnTheCrashImage],
) -> Vec<ReadRangeOnTheCrashImage> {
    let mut sorted: Vec<ReadRangeOnTheCrashImage> = ranges
        .iter()
        .copied()
        .filter(|range| range.end_sector > range.first_sector)
        .collect();
    sorted.sort_unstable();
    let mut merged: Vec<ReadRangeOnTheCrashImage> = Vec::with_capacity(sorted.len());
    for range in sorted {
        match merged.last_mut() {
            Some(last) if last.device == range.device && range.first_sector <= last.end_sector => {
                last.end_sector = last.end_sector.max(range.end_sector);
            }
            _ => merged.push(range),
        }
    }
    merged
}

/// 判段在观察者交回的盘上判哪几条不变量。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InvariantsJudgedOnTheImageAfterTheObserver {
    EveryImplementedInvariant,
    /// 只判这几条（`singlefs_checker::image::IMPLEMENTED_INVARIANTS` 里的名字）。
    Only(Vec<&'static str>),
}

impl InvariantsJudgedOnTheImageAfterTheObserver {
    pub(super) fn includes(&self, invariant: &'static str) -> bool {
        match self {
            Self::EveryImplementedInvariant => true,
            Self::Only(invariants) => invariants.contains(&invariant),
        }
    }
}

/// 交回镜像的观察者怎么去重（「恢复报告分桶 + 读集见证」）：产品路径先 `recover`、按恢复报告分桶，桶里第一个状态真跑观察者、
/// 把它读过的段与那几段的字节摘要存成见证，后来落进同一桶的状态只算自己镜像上同样几段的摘要，对上任何一个见证就取那一次的结果，一个都对不上
/// 才真跑、再添一个见证。为什么对：观察者交回的只随它读到的字节、恢复报告与它另读的输入变（桶键带后两样），而且确定；恢复只读不写
/// （`singlefs_core::recovery::recover` 收 `&dyn PoolReader`），所以恢复后的字节与崩溃镜像的字节是同一个量——按整张镜像的字节去重的话，
/// 枚举一个段的子集时每个子集盘上字节都不同，一张都合不掉（全量第 6 条实测 917504 张镜像复用 0）。
/// 关掉去重只给剪枝自证用（小输入上关掉跑一遍，逐状态判定与开着时逐项相同）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverDeduplication {
    ByTheRecoveryReportAndTheReadSet,
    OffForTheSelfProof,
}

/// 判段从判器交回的逐盘自证过的系统配置槽数看一个崩溃状态的观察者（两槽全废盘面观察者「你做你的 GPU版让GPU去做」）：
/// 这个状态的逐盘槽数（判器交回的：配了 GPU 是走读内核第 25 字，没配是 CPU 参照）、这条流基镜像那一份、往观察者计数里记；判不对就 panic
/// （判段接住记成红项）。录入段对它什么都不跑；判段按块读判器存下的那一份（[`SelfVerifiedSystemConfigurationSlotsHandedBackForABlock`]），
/// 槽数相同的连着几个状态只看一次（它只随这两份槽数变）。
pub type ObserveTheSelfVerifiedSystemConfigurationSlotsOfACrashState<'observer> =
    dyn Fn(&[(u32, u32)], &[(u32, u32)], &mut CrashPointReplayObserverCounts) + Sync + 'observer;

/// 观察者的代码是哪一种。
#[derive(Clone, Copy)]
pub enum ObserverCode<'observer> {
    CountsOnly(&'observer ObserveACrashState<'observer>),
    HandsBackAnImage {
        observe: &'observer ObserveACrashStateHandingBackAnImage<'observer>,
        deduplication: ObserverDeduplication,
    },
    /// 判段从判器交回的逐盘自证槽数看（[`ObserveTheSelfVerifiedSystemConfigurationSlotsOfACrashState`]）：录入段不录它。
    FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(
        &'observer ObserveTheSelfVerifiedSystemConfigurationSlotsOfACrashState<'observer>,
    ),
}

impl ObserverCode<'_> {
    /// 录入段录不录它：判段从判器交回的逐盘槽数看的那一种不录（录入段对它什么都不跑），别的两种录。
    #[must_use]
    pub fn is_recorded_in_the_recording_stage(&self) -> bool {
        match self {
            Self::CountsOnly(_) | Self::HandsBackAnImage { .. } => true,
            Self::FromTheSelfVerifiedSystemConfigurationSlotsTheJudgeHandsBack(_) => false,
        }
    }
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
        self.holds_the_point_named(point_code_file(point), &point.name)
    }

    /// 设在 `code_file`、名叫 `point_name` 的那个点是不是这个节点的：同一份设点文件，点名就是入口名或以 `<入口名>.` 起头。
    #[must_use]
    pub fn holds_the_point_named(&self, code_file: &str, point_name: &str) -> bool {
        code_file == self.setting_file
            && (point_name == self.entry_name
                || point_name
                    .strip_prefix(self.entry_name.as_str())
                    .is_some_and(|rest| rest.starts_with('.')))
    }
}

/// 挂在几个节点上的一个观察者（登记表在 `crates/singlefs-verification/tests/common_crash_points/mod.rs`）：名字、代码文件（代码文件与名字拼成它的
/// 判定项标识，`crate::crash_identity::JudgingUnits::ObserverCode`，进它的版本）、另读的流程输入的摘要（进它的版本与去重键 乙；
/// 要真算，没有另读的输入才给全零）、挂在哪几个节点上、它的代码。观察者代码不进计划哈希、不进任何一把版本。
#[derive(Clone)]
pub struct ObserverOnTheNodes<'observer> {
    pub observer_name: ObserverName,
    pub observer_code_file: String,
    pub extra_inputs_digest: [u8; DIGEST_BYTES],
    pub nodes: Vec<NodeOfAnObserver>,
    pub observe: ObserverCode<'observer>,
}

/// 一个观察者这一趟用的几把版本（[`observer_versions_keeping_the_refused_out`]）：块的输入数据在块键里，这几把只把同一块上的几个观察者分开。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObserverVersions {
    /// 观察者的判定项标识与它引到的被检测产品代码（[`observer_identity_with_its_product_code_digest`]）与另读的输入：录入段的块按它存。
    /// 观察者自己的代码不进，改观察者不作废已录、已判的块；它调到的产品挂载、恢复代码改了，它的块现录、现判。
    pub observer: ConclusionVersion,
    /// 同上，另读的输入那一格取全零：另读的输入单独进去重键（[`observed_image_deduplication_key`]），去重键不靠版本带它。
    pub observer_code: ConclusionVersion,
    /// 判段对观察者交回的盘跑池级 checker 的判器（配了 GPU 是 GPU 判器，没配是 CPU 判器）与它那一把版本：由结论种类与判器引到的被检测产品代码定
    /// （[`ConclusionVersion::of_the_pool_checker_on_the_images_after_the_observers`]），判器的代码不进，两条路线同一把。
    pub pool_checker: JudgedBy,
    pub pool_checker_version: ConclusionVersion,
    /// 这个观察者的结论按它存：观察者版本并上池级 checker 那一把（观察者那一把变了，结论重拼；判器与拼块的代码不进）。
    pub conclusion: ConclusionVersion,
}

/// 一个观察者那一组判法单元：（观察者代码文件，观察者名）是它的判定项标识，在登记表里找它那一组取它引到的被检测产品代码的键。
pub(super) fn observer_code_units_of(observer: &ObserverOnTheNodes<'_>) -> JudgingUnits {
    JudgingUnits::ObserverCode {
        observer_code_file: observer.observer_code_file.clone(),
        observer_name: observer.observer_name.as_str().to_string(),
    }
}

/// 一个观察者的判定项标识：它那一组的组名（观察者代码文件与观察者名拼的，`crate::judging_units_entries::observer_group_name`）取 SHA-256。
/// 同一份文件里两个观察者名字不同，这一截就不同；观察者的代码不进。
#[must_use]
pub fn observer_identity_digest(units: &JudgingUnits) -> [u8; DIGEST_BYTES] {
    sha256_digest(format!("singlefs observer identity\n{}\n", units.group_name()).as_bytes())
}

/// 一个登记了的观察者进版本的那一截：判定项标识（[`observer_identity_digest`]）与它那一组的键（`product_code`：观察者引到的被检测产品代码，
/// 验证包那一张表的观察者组只留产品包的单元，[`crate::crash_identity::observer_product_code_key_in`]）拼起来取 SHA-256。
#[must_use]
pub fn observer_identity_with_its_product_code_digest(
    units: &JudgingUnits,
    product_code: &JudgingUnitsKey,
) -> [u8; DIGEST_BYTES] {
    sha256_digest(
        format!(
            "singlefs observer identity with its product code\nidentity={}\nproduct_code={}\n",
            hexadecimal_text(&observer_identity_digest(units)),
            hexadecimal_text(product_code.fingerprint())
        )
        .as_bytes(),
    )
}

/// 一个观察者这一趟的几把版本：块的输入数据在块键里，这几把用观察者的判定项标识与它引到的被检测产品代码
/// （[`observer_identity_with_its_product_code_digest`]）与它另读的输入把同一块上几个观察者的结论分开；观察者自己的代码、判它交回的盘的池级 checker
/// 与观察者结论拼块的代码都不进（池级 checker 那一把由结论种类与判器引到的被检测产品代码定）。
/// 观察者没登记（`crate::judging_units_entries::OBSERVER_ENTRIES` 里没有它）或它那一组拒算（[`crate::crash_identity::observer_product_code_key_in`]）的，标识那一截取占位
/// （[`placeholder_judging_units_key_of_the_refused`]）：交回版本、拒算的那一组与照判不入库的那几把版本（观察者那一把、代码那一把、
/// 结论那一把都是从占位拼出来的，都不入库）。别的照键入库同一份库。
pub(crate) fn observer_versions_keeping_the_refused_out(
    observer: &ObserverOnTheNodes<'_>,
    pool_checker: JudgedBy,
) -> (
    ObserverVersions,
    Vec<JudgingUnitsKeyRefused>,
    Vec<[u8; DIGEST_BYTES]>,
) {
    observer_versions_keeping_the_refused_out_with_the_observer_keys_in(
        observer,
        pool_checker,
        ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
    )
}

/// 同 [`observer_versions_keeping_the_refused_out`]，观察者登没登记在 `observer_keys` 点名的那一张键表里认
/// （观察者登在实验包的，认实验包构建脚本编进它二进制的那一张）。
pub(crate) fn observer_versions_keeping_the_refused_out_with_the_observer_keys_in(
    observer: &ObserverOnTheNodes<'_>,
    pool_checker: JudgedBy,
    observer_keys: ObserverJudgingUnitsKeyTable,
) -> (
    ObserverVersions,
    Vec<JudgingUnitsKeyRefused>,
    Vec<[u8; DIGEST_BYTES]>,
) {
    observer_versions_keeping_the_refused_out_under_the_process_marker(
        observer,
        pool_checker,
        observer_keys,
        marker_of_this_process(),
    )
}

/// 同 [`observer_versions_keeping_the_refused_out_with_the_observer_keys_in`]，没登记的那一个的占位按调用方给的进程标记取
/// （[`placeholder_judging_units_key_of_the_refused_under_the_process_marker`]）。
pub(super) fn observer_versions_keeping_the_refused_out_under_the_process_marker(
    observer: &ObserverOnTheNodes<'_>,
    pool_checker: JudgedBy,
    observer_keys: ObserverJudgingUnitsKeyTable,
    process_marker: &[u8; DIGEST_BYTES],
) -> (
    ObserverVersions,
    Vec<JudgingUnitsKeyRefused>,
    Vec<[u8; DIGEST_BYTES]>,
) {
    let units = observer_code_units_of(observer);
    let (identity_of_the_observer, refusals) =
        match observer_product_code_key_in(&units, observer_keys) {
            Ok(product_code) => (
                observer_identity_with_its_product_code_digest(&units, &product_code),
                Vec::new(),
            ),
            Err(refusal) => (
                *placeholder_judging_units_key_of_the_refused_under_the_process_marker(
                    &units,
                    process_marker,
                )
                .fingerprint(),
                vec![refusal],
            ),
        };
    let versions =
        observer_versions_of_the_identity(observer, pool_checker, &identity_of_the_observer);
    let mut versions_kept_out = Vec::new();
    if !refusals.is_empty() {
        versions_kept_out.extend([
            versions.observer.0,
            versions.observer_code.0,
            versions.conclusion.0,
        ]);
    } else if matches!(
        crate::crash_identity::judged_product_code_of_this_binary(),
        crate::crash_identity::JudgedProductCodeOfThisBinary::RefusedInThisProcess { .. }
    ) {
        // 判器引到的被检测产品代码那一组拒算时，池级 checker 那一把是这个进程的占位，从它拼出来的结论那一把同样照判不入库
        versions_kept_out.push(versions.conclusion.0);
    }
    (versions, refusals, versions_kept_out)
}

/// 从观察者的标识那一截（登记了的是 [`observer_identity_with_its_product_code_digest`]，没登记或拒算的是占位）拼一个观察者的几把版本。
fn observer_versions_of_the_identity(
    observer: &ObserverOnTheNodes<'_>,
    pool_checker: JudgedBy,
    identity_of_the_observer: &[u8; DIGEST_BYTES],
) -> ObserverVersions {
    // 版本由标识那一截（判定项标识与观察者引到的被检测产品代码）与另读的输入组成，不带观察者自己的代码与手写的版本常量
    let version_with_the_extra_inputs = |extra_inputs_digest: [u8; DIGEST_BYTES]| {
        ConclusionVersion(sha256_digest(
            format!(
                "singlefs observer\nidentity={}\nextra_inputs={}\n",
                hexadecimal_text(identity_of_the_observer),
                hexadecimal_text(&extra_inputs_digest)
            )
            .as_bytes(),
        ))
    };
    let observer_version = version_with_the_extra_inputs(observer.extra_inputs_digest);
    let observer_code = version_with_the_extra_inputs([0; DIGEST_BYTES]);
    let pool_checker_version =
        ConclusionVersion::of_the_pool_checker_on_the_images_after_the_observers();
    let conclusion = ConclusionVersion(sha256_digest(
        format!(
            "singlefs observer conclusion with the pool checker on its image\nobserver={}\npool_checker={}:{}\n",
            hexadecimal_text(&observer_version.0),
            pool_checker.conclusion_kind().version_text(),
            hexadecimal_text(&pool_checker_version.0)
        )
        .as_bytes(),
    ));
    ObserverVersions {
        observer: observer_version,
        observer_code,
        pool_checker,
        pool_checker_version,
        conclusion,
    }
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
    /// 交回镜像的观察者这一趟真看的恢复镜像张数（现判；开着去重时每张都是一个新见证）。
    pub images_observed_here: u64,
    /// 交回镜像的观察者经过的状态里，那张镜像已经看过（这一趟早先的状态，或库里存着的）、这一趟不再看的状态数（去重复用）。
    pub states_reusing_an_observed_image: u64,
    /// 这一趟见过的桶数（恢复报告不同的张数；这一趟每个观察者的桶各数各的）。
    pub buckets: u64,
    /// 落进已有见证的桶、见证一个都对不上而真跑的次数（桶里第一次真跑不算）。
    pub witness_misses: u64,
    /// 叫停了（[`stop_is_requested_by_the_environment`]）：停在块边上，录过的块在库里，下一趟接着录。
    pub stopped_early: bool,
    /// 录入段这一趟恢复了几次（看 journal 那一遍）：一块里每个状态至多一次，这一块上要录几个观察者都一样。
    pub states_recovered_here: u64,
}

impl ObserverRecordingTally {
    pub(super) fn absorb(&mut self, other: &Self) {
        let Self {
            blocks_recorded_here,
            states_recorded_here,
            blocks_already_recorded,
            states_counted_here,
            images_observed_here,
            states_reusing_an_observed_image,
            buckets,
            witness_misses,
            stopped_early,
            states_recovered_here,
        } = other;
        self.blocks_recorded_here += blocks_recorded_here;
        self.states_recorded_here += states_recorded_here;
        self.blocks_already_recorded += blocks_already_recorded;
        self.states_counted_here += states_counted_here;
        self.images_observed_here += images_observed_here;
        self.states_reusing_an_observed_image += states_reusing_an_observed_image;
        self.buckets += buckets;
        self.witness_misses += witness_misses;
        self.stopped_early |= stopped_early;
        self.states_recovered_here += states_recovered_here;
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
    /// 这台机器这一趟领的块落在计划里哪几个点上、各几块（按计划的次序，一块都没领到的点不列；不看挂没挂观察者）。
    /// 设了块区间（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）时调用方照它与自己挂观察者的节点（[`NodeOfAnObserver::holds_the_point_named`]）
    /// 数出这一片里有几块落在某个观察者的节点上：有块的照旧要求那个观察者看过状态，一块都没有的不要求。
    pub blocks_selected_here_on_each_point: Vec<BlocksSelectedHereOnAPoint>,
}

/// 这台机器这一趟领的块里落在计划里某一个点上的块数（[`ObservingOfTheRun::blocks_selected_here_on_each_point`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlocksSelectedHereOnAPoint {
    /// 设这个点的文件（仓内路径）与点名（`<入口>.<写的种类>.<步>`，末节点是 [`END_OF_FLOW_NODE_NAME`]）。
    pub code_file: String,
    pub point_name: String,
    pub blocks: u64,
}

#[cfg(test)]
mod tests;
