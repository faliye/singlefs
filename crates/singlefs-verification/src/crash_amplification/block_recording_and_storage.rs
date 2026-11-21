//! 录入与存库：把每个点的状态按块登进录入表、数录了多少（[`record_crash_points`]、[`tally_the_recorded_blocks`]），
//! 结论按哪一把版本键存、读、复用（[`ConclusionsStoredUnder`]），连同领还没判的块与把判完的块写回库。

use super::*;

// ============================== 录入 ==============================

/// 录入的结果：这一趟录进库的块数、库里已有判定（复用）的块数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordingTally {
    pub blocks_recorded: u64,
    pub blocks_already_judged: u64,
    pub states_recorded: u64,
    pub states_already_judged: u64,
}

/// 录入：每个点的每一块登进录入表，路径 → 身份登进路径索引表；库里已经有同一块键在这一判法单元指纹下的判定块就不再核
/// （别的判法单元指纹判过的不算：旧行留着，这一版从头判）。
///
/// # Errors
/// 库读写失败。
pub fn record_crash_points(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    version: &JudgingUnitsFingerprint,
    report: &mut CoverageReport,
) -> Result<RecordingTally, VerdictStoreError> {
    record_crash_points_needing(
        store,
        plan,
        &[ConclusionsStoredUnder::TheJudgingUnitsFingerprint(version)],
        report,
    )
}

/// 同 [`record_crash_points`]，一块算复用要 `needed` 里每一处都有结论（三种结论那一套「哪一种没有就只排那一种」）；
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

/// 同 [`write_the_recorded_blocks`]，每一行各取一次库锁、块键在锁外算：录入段不挡着这个进程里别的线程读写判定库。
///
/// # Errors
/// 库写失败。
pub(crate) fn write_the_recorded_blocks_taking_the_library_lock_row_by_row(
    library: &crate::crash_library::CrashLibrary,
    plan: &CrashPlan,
) -> Result<(), VerdictStoreError> {
    for point in &plan.points {
        library
            .lock()
            .record_path(&point.path_number, &point.identity)?;
        for local in point.blocks() {
            crate::crash_library::assert_this_thread_holds_no_library_lock("录入段算块键");
            let block_key = point.block_key(&local);
            let state_count = local.end - local.start;
            if point.is_a_block_of_a_sliced_segment(&local) {
                library
                    .lock()
                    .record_block_of_a_slice(&block_key, state_count)?;
            } else {
                library.lock().record_block(&block_key, state_count)?;
            }
        }
    }
    Ok(())
}

/// 判段只读：照计划数录入的块里哪些已有 `needed` 每一处的结论（复用）、哪些要判，覆盖报告逐点记。不往库里写。
/// 每一处要读库的那一半整份计划一次读回（[`ConclusionsStoredUnder::reuse_read_of_the_plan`]），再照它逐块数（[`tally_the_recorded_blocks_from_the_reuse_reads`]）。
///
/// # Errors
/// 库读失败。
pub fn tally_the_recorded_blocks(
    store: &VerdictStore,
    plan: &CrashPlan,
    needed: &[ConclusionsStoredUnder<'_>],
    report: &mut CoverageReport,
) -> Result<RecordingTally, VerdictStoreError> {
    let reuse_reads = needed
        .iter()
        .map(|conclusions| conclusions.reuse_read_of_the_plan(store, plan, &|_| true))
        .collect::<Result<Vec<ReuseOfTheBlocksReadOnce>, VerdictStoreError>>()?;
    Ok(tally_the_recorded_blocks_from_the_reuse_reads(
        store,
        plan,
        needed,
        &reuse_reads,
        report,
    ))
}

/// 同 [`tally_the_recorded_blocks`]，每一处要读库的那一半照调用方读回的那一份（`reuse_reads` 与 `needed` 同序，每一份都读过计划里全部的块）：
/// 一趟里数复用、抽哨兵共用同一份，不再逐块读库。
///
/// # Panics
/// `reuse_reads` 与 `needed` 不一样长；某一份没读计划里的某一块。
pub(crate) fn tally_the_recorded_blocks_from_the_reuse_reads(
    store: &VerdictStore,
    plan: &CrashPlan,
    needed: &[ConclusionsStoredUnder<'_>],
    reuse_reads: &[ReuseOfTheBlocksReadOnce],
    report: &mut CoverageReport,
) -> RecordingTally {
    assert_eq!(
        needed.len(),
        reuse_reads.len(),
        "每一处各一份读回的复用、同序"
    );
    let mut tally = RecordingTally::default();
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let mut point_reused = true;
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            let key = point.block_key(&local);
            let count = local.end - local.start;
            let mut every_needed_conclusion_is_stored = true;
            for (conclusions, reuse_read) in needed.iter().zip(reuse_reads) {
                if !reuse_read.is_stored_for_reuse(
                    store,
                    conclusions,
                    point_index,
                    &key,
                    this_block,
                ) {
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
    tally
}

pub(super) fn point_code_file(point: &PlannedCrashPoint) -> &str {
    point
        .identity
        .split("::")
        .next()
        .expect("身份以仓内路径起头")
}

// ============================== 核对 ==============================

// ============================== 结论记在库里哪一处 ==============================

/// 一块的结论记在库里哪一处。
#[derive(Clone, Debug)]
pub enum ConclusionsStoredUnder<'versions> {
    /// 今天的判法单元指纹那一套：两个判器合起来一把，结论按（块键，判法单元指纹）存，只存判定与违例。直接调流水线各段的对拍用例与单测用它；
    /// [`run_from_environment`] 不用。
    TheJudgingUnitsFingerprint(&'versions JudgingUnitsFingerprint),
    /// 三种结论里的一种：判定库按（块键，结论种类，版本）存，计数向量、恢复字与在哪判的随块存。每个点各一把版本，与计划的点同序
    /// （GPU 判器每个点同一把，CPU 判器按设点文件各一把，[`ConclusionVersionsOfThePlan`]）。
    TheConclusionKind {
        kind: ConclusionKind,
        version_of_each_point: &'versions [ConclusionVersion],
    },
}

/// 随一块判定一起存的（三种结论那一套才存进库）：计数向量、每个状态的恢复字（CPU 判器没编进 `gpu` 特性时没有）、在哪判的、
/// 判器交回的逐盘自证过的系统配置槽数（[`SelfVerifiedSystemConfigurationSlotsHandedBackForABlock`]；取样那一路交不回，是 `None`）。
pub(super) struct StoredAlongTheVerdicts {
    pub(super) count_vector: BlockCountVector,
    pub(super) recovery_words: Option<Vec<RecoveryWordsOfState>>,
    pub(super) judged_on: JudgingDevice,
    pub(super) self_verified_system_configuration_slots:
        Option<SelfVerifiedSystemConfigurationSlotsHandedBackForABlock>,
}

// ------------------------------ 判器交回的逐盘自证槽数（两槽全废盘面观察者挪进判段：「你做你的 GPU版让GPU去做」） ------------------------------
//
// 两槽全废盘面观察者在判段判：判器（配了 GPU 是 GPU 走读内核第 25 字 `crate::crash_judge_gpu::SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_OF_EACH_DEVICE_WORD`，
// 没配是 CPU 参照 `crate::crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device`）逐状态交回逐盘自证过的系统配置槽数，
// 判段写结论块的同一处把它连同这条流基镜像那一份（CPU 参照，基镜像不是崩溃状态、不上卡）存进库；观察者的判段
// （[`judge_the_observers_of_the_flow`]）只从库里读这一份、对基镜像比，录入段对这个观察者什么都不跑（不恢复、不调池级 checker）。
// 存在哪：观察者录入块那张表（`VerdictStore::store_observer_block`），键是块键、判器结论的种类与这一块判器结论那一把版本——与结论块同一把，
// 结论块在库里而它不在的块不算复用得上（[`ConclusionsStoredUnder::is_stored_for_reuse`]），判段照判一遍补上。

/// 库里那一份的头 8 字节。
const SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_MAGIC: &[u8; 8] = b"SFSLOTS1";

fn put_slots_of_each_device(bytes: &mut Vec<u8>, slots_of_each_device: &[(u32, u32)]) {
    put_item_count(bytes, slots_of_each_device.len());
    for (device, self_verified_slots) in slots_of_each_device {
        bytes.extend_from_slice(&device.to_le_bytes());
        bytes.extend_from_slice(&self_verified_slots.to_le_bytes());
    }
}

fn read_slots_of_each_device(reader: &mut RecordedBytesReader<'_>) -> Option<Vec<(u32, u32)>> {
    let mut slots_of_each_device = Vec::new();
    for _ in 0..reader.count()? {
        slots_of_each_device.push((reader.u32()?, reader.u32()?));
    }
    Some(slots_of_each_device)
}

impl SelfVerifiedSystemConfigurationSlotsHandedBackForABlock {
    /// 存进库的字节：头 8 字节、基镜像那一份（盘数、逐盘 (盘, 槽数)）、段数、每段（状态数、盘数、逐盘 (盘, 槽数)），数都是小端。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_MAGIC.to_vec();
        put_slots_of_each_device(&mut bytes, &self.slots_of_each_device_on_the_base_image);
        put_item_count(&mut bytes, self.slots_of_the_states.runs.len());
        for run in &self.slots_of_the_states.runs {
            bytes.extend_from_slice(&run.states.to_le_bytes());
            put_slots_of_each_device(&mut bytes, &run.slots_of_each_device);
        }
        bytes
    }

    /// 从库里的字节读回；头不对、读不满、末尾多字节交回 `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut reader = RecordedBytesReader { bytes, at: 0 };
        if reader.take(SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_MAGIC.len())?
            != SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOTS_MAGIC
        {
            return None;
        }
        let slots_of_each_device_on_the_base_image = read_slots_of_each_device(&mut reader)?;
        let mut runs = Vec::new();
        for _ in 0..reader.count()? {
            let states = reader.u64()?;
            runs.push(RunOfStatesWithTheSameSelfVerifiedSlots {
                states,
                slots_of_each_device: read_slots_of_each_device(&mut reader)?,
            });
        }
        (reader.at == bytes.len()).then_some(Self {
            slots_of_each_device_on_the_base_image,
            slots_of_the_states: SelfVerifiedSystemConfigurationSlotsOfTheStates { runs },
        })
    }
}

/// 计划里每一块（块序号即下标）在一处问复用要读库的那一半，一次读回（[`ConclusionsStoredUnder::reuse_read_of_the_plan`]）：
/// 判过、连同判器交回的逐盘自证槽数、而不红的是 `Some(true)`；没选中读的块是 `None`。
/// 一趟里录入时数复用、数这一份已判的态数、抽复用哨兵都照这一份答（同一个进程里不再逐块读库）；这几步之间这个进程不写这一处的结论，
/// 别的进程这时写进库的块照读回的那一刻算（逐块现读时同样只看得到读的那一刻）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReuseOfTheBlocksReadOnce {
    judged_with_the_slots_and_not_red_of_each_block: Vec<Option<bool>>,
}

impl ReuseOfTheBlocksReadOnce {
    /// 照读回的那一份答 [`ConclusionsStoredUnder::is_stored_for_reuse`]：强制现判表现问（同一条连接上别的趟随时添、撤）。
    ///
    /// # Panics
    /// `block_sequence` 那一块读的时候没选中（调用方只问读过的块）。
    pub(crate) fn is_stored_for_reuse(
        &self,
        store: &VerdictStore,
        conclusions: &ConclusionsStoredUnder<'_>,
        point_index: usize,
        key: &VerdictBlockKey,
        block_sequence: u64,
    ) -> bool {
        let judged_with_the_slots_and_not_red = usize::try_from(block_sequence)
            .ok()
            .and_then(|index| {
                self.judged_with_the_slots_and_not_red_of_each_block
                    .get(index)
            })
            .copied()
            .flatten()
            .expect("问的块读的时候选中了：调用方只问读过的块");
        !conclusions.is_forced_to_be_judged_again(store, point_index, key)
            && judged_with_the_slots_and_not_red
    }
}

impl ConclusionsStoredUnder<'_> {
    /// 第 `point_index` 个点那一把版本。
    ///
    /// # Panics
    /// 版本的个数与计划的点数不等（建版本时照计划建，见 [`ConclusionVersionsOfThePlan::of_the_plan`]）。
    pub(super) fn conclusion_version_of_the_point(
        version_of_each_point: &[ConclusionVersion],
        point_index: usize,
    ) -> ConclusionVersion {
        *version_of_each_point.get(point_index).expect(
            "每个点各一把版本、与计划的点同序：ConclusionVersionsOfThePlan::of_the_plan 照计划的点建",
        )
    }

    /// 复用那一步问的：这一块在这一处有没有可复用的结论（[`Self::is_stored`]）。三种结论那一套另要判器交回的逐盘自证槽数也在
    /// （[`Self::self_verified_system_configuration_slots_are_stored`]）：结论在而它不在的块（取样那一路存的、别的机器导进来的）照判一遍补上，
    /// 两槽全废盘面观察者在判段读得到每一块。记着红的（结论里有违例）不复用：下一趟现判，判完照新结论写回（
    /// 「记红、撤绿、下趟现判」；存块那一处两份有一份红就换上，`VerdictBlockStoring::ReplacedBecauseOneOfTheTwoIsRed`）。复用哨兵这一趟抽中强制现判的
    /// 那一块（`VerdictStore::conclusion_block_is_forced_to_be_judged_again`）同样答没有。崩溃枚举用例判段的块账问「罩没罩住」
    /// 用的也是它（红块另数进红）。分片的段找续点不用它，用 [`Self::is_judged_with_the_slots`]：红块也是判过的，续点照旧往后走。
    pub(crate) fn is_stored_for_reuse(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        if self.is_forced_to_be_judged_again(store, point_index, key) {
            return Ok(false);
        }
        Ok(self.is_judged_with_the_slots(store, point_index, key)?
            && !self.is_red(store, point_index, key)?)
    }

    /// [`Self::is_stored_for_reuse`] 要读库的那一半（判过、连同判器交回的逐盘自证槽数，而不红），计划里 `is_read` 选中的每一块一次读回
    /// （块序号 = 全部点的块按次序编号；没选中的不读）。三种结论那一套成批读：块属性与违例表（[`VerdictStore::conclusion_blocks_stored_and_red`]），
    /// 存着的再成批读槽数那一行（[`VerdictStore::observer_block_rows_stored`]）；与逐块问的是同几行、同一个谓词。判法单元指纹那一套逐块问。
    /// 强制现判表不在里面：用的时候现问（[`ReuseOfTheBlocksReadOnce::is_stored_for_reuse`]）。
    ///
    /// # Errors
    /// 库读失败；块属性那一行不合布局。
    pub(crate) fn reuse_read_of_the_plan(
        &self,
        store: &VerdictStore,
        plan: &CrashPlan,
        is_read: &dyn Fn(u64) -> bool,
    ) -> Result<ReuseOfTheBlocksReadOnce, VerdictStoreError> {
        let mut blocks_read: Vec<(usize, VerdictBlockKey)> = Vec::new();
        let mut is_read_of_each_block: Vec<bool> = Vec::new();
        let mut block_sequence = 0u64;
        for (point_index, point) in plan.points.iter().enumerate() {
            for local in point.blocks() {
                let this_block = block_sequence;
                block_sequence += 1;
                let selected = is_read(this_block);
                is_read_of_each_block.push(selected);
                if selected {
                    blocks_read.push((point_index, point.block_key(&local)));
                }
            }
        }
        let reusable_of_the_blocks_read: Vec<bool> = match self {
            Self::TheJudgingUnitsFingerprint(_) => {
                let mut reusable = Vec::with_capacity(blocks_read.len());
                for (point_index, key) in &blocks_read {
                    reusable.push(
                        self.is_judged_with_the_slots(store, *point_index, key)?
                            && !self.is_red(store, *point_index, key)?,
                    );
                }
                reusable
            }
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => {
                let conclusions: Vec<crate::verdict_store::ConclusionOfABlockUnderAKey> =
                    blocks_read
                        .iter()
                        .map(|(point_index, key)| {
                            (
                                key.clone(),
                                kind.clone(),
                                Self::conclusion_version_of_the_point(
                                    version_of_each_point,
                                    *point_index,
                                ),
                            )
                        })
                        .collect();
                let conclusions_in_the_library =
                    store.conclusion_blocks_stored_and_red(&conclusions)?;
                // 槽数那一行与结论块同一把键（观察者录入块那张表，结论的种类与版本）：只读存着的那几块
                let slots_rows_to_read: Vec<crate::verdict_store::ConclusionOfABlockUnderAKey> =
                    conclusions
                        .iter()
                        .zip(&conclusions_in_the_library)
                        .filter(|(_conclusion, in_the_library)| {
                            **in_the_library != ConclusionBlockInTheLibrary::NotStored
                        })
                        .map(|(conclusion, _in_the_library)| conclusion.clone())
                        .collect();
                let mut slots_rows_stored = store
                    .observer_block_rows_stored(&slots_rows_to_read)?
                    .into_iter();
                conclusions_in_the_library
                    .into_iter()
                    .map(|in_the_library| match in_the_library {
                        ConclusionBlockInTheLibrary::NotStored => false,
                        ConclusionBlockInTheLibrary::StoredGreen => slots_rows_stored
                            .next()
                            .expect("存着的每一块各读了一行槽数"),
                        ConclusionBlockInTheLibrary::StoredRed => {
                            slots_rows_stored
                                .next()
                                .expect("存着的每一块各读了一行槽数");
                            false
                        }
                    })
                    .collect()
            }
        };
        let mut reusable_of_the_blocks_read = reusable_of_the_blocks_read.into_iter();
        Ok(ReuseOfTheBlocksReadOnce {
            judged_with_the_slots_and_not_red_of_each_block: is_read_of_each_block
                .into_iter()
                .map(|selected| {
                    selected.then(|| {
                        reusable_of_the_blocks_read
                            .next()
                            .expect("选中的每一块各读了一项")
                    })
                })
                .collect(),
        })
    }

    /// 这一块在这一处判过没有（结论在、三种结论那一套连同判器交回的逐盘自证槽数也在），不论红不红、不论这一趟读不读库：
    /// 分片的段找续点（[`resume_point_of_the_sliced_segment`]）照它走，红块也算判过，续点不卡在红块上（红块要不要现判由
    /// [`Self::is_stored_for_reuse`] 在判段那一步定）。
    pub(crate) fn is_judged_with_the_slots(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        Ok(self.is_stored(store, point_index, key)?
            && self.self_verified_system_configuration_slots_are_stored(store, point_index, key)?)
    }

    /// 这一块在这一处的结论是不是红的（有违例）；没有结论答不是。
    pub(crate) fn is_red(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        match self {
            Self::TheJudgingUnitsFingerprint(version) => {
                Ok(!store.violations_in_block(key, version)?.is_empty())
            }
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => store.conclusion_block_has_violations(
                key,
                kind,
                &Self::conclusion_version_of_the_point(version_of_each_point, point_index),
            ),
        }
    }

    /// 这一块在这一处是不是这一趟复用哨兵抽中强制现判的（只有三种结论那一套会被抽）。
    pub(super) fn is_forced_to_be_judged_again(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> bool {
        match self {
            Self::TheJudgingUnitsFingerprint(_) => false,
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => store.conclusion_block_is_forced_to_be_judged_again(
                key,
                kind,
                &Self::conclusion_version_of_the_point(version_of_each_point, point_index),
            ),
        }
    }

    /// 判器交回的逐盘自证槽数（[`SelfVerifiedSystemConfigurationSlotsHandedBackForABlock`]）这一块在这一处存没存：键与结论块同一把。
    /// 判法单元指纹那一套不存它（直接调流水线各段的对拍用例与单测用，不挂观察者），答存了。
    fn self_verified_system_configuration_slots_are_stored(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        match self {
            Self::TheJudgingUnitsFingerprint(_) => Ok(true),
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => Ok(store
                .read_observer_block(
                    key,
                    kind,
                    &Self::conclusion_version_of_the_point(version_of_each_point, point_index),
                )?
                .is_some()),
        }
    }

    /// 这一块在这一处有没有结论。
    pub(crate) fn is_stored(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<bool, VerdictStoreError> {
        match self {
            Self::TheJudgingUnitsFingerprint(version) => {
                Ok(store.read_verdict_block(key, version)?.is_some())
            }
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

    /// 写一块：判法单元指纹那一套只写判定与违例，三种结论那一套连同计数向量、恢复字与在哪判的一批写下去；判器交回了逐盘自证槽数的，
    /// 它那一行在同一把键下、与结论块同一批写（结论块在库里，它就在），结论已存着、逐项相同时只写它。
    pub(super) fn store_block(
        &self,
        store: &mut VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
        block: VerdictBlock,
        violating: Vec<StateOffsetInBlock>,
        along: StoredAlongTheVerdicts,
    ) -> Result<(), VerdictStoreError> {
        match self {
            Self::TheJudgingUnitsFingerprint(version) => {
                store.store_verdict_block(key, version, &block, &violating)?;
                Ok(())
            }
            Self::TheConclusionKind {
                kind,
                version_of_each_point,
            } => {
                let version =
                    Self::conclusion_version_of_the_point(version_of_each_point, point_index);
                let (record, slots_bytes) =
                    Self::conclusion_block_record_and_the_slots_bytes(block, violating, along);
                store.store_conclusion_block_with_the_slots_the_judge_handed_back(
                    key,
                    kind,
                    &version,
                    record,
                    slots_bytes,
                )?;
                Ok(())
            }
        }
    }

    /// 三种结论那一套存一块要交给判定库的整份与判器交回的逐盘自证槽数那一行的字节（判段存块 [`Self::store_block`] 与
    /// GPU 批处理服务编结论帧 [`conclusion_frame_of_a_gpu_pending_block`] 共用）。
    pub(super) fn conclusion_block_record_and_the_slots_bytes(
        block: VerdictBlock,
        violating: Vec<StateOffsetInBlock>,
        along: StoredAlongTheVerdicts,
    ) -> (ConclusionBlockRecord, Option<Vec<u8>>) {
        let slots_bytes = along
            .self_verified_system_configuration_slots
            .as_ref()
            .map(SelfVerifiedSystemConfigurationSlotsHandedBackForABlock::to_bytes);
        (
            ConclusionBlockRecord {
                block,
                violating_states: violating,
                count_vector: along.count_vector,
                judged_on: along.judged_on,
                recovery_words: along.recovery_words,
            },
            slots_bytes,
        )
    }

    /// 这一块在这一处每个状态的判定向量编号；没有结论是 `None`。
    pub(super) fn verdict_numbers_of_each_state(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<Option<Vec<VerdictVectorNumber>>, VerdictStoreError> {
        match self {
            Self::TheJudgingUnitsFingerprint(version) => Ok(store
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
    pub(crate) fn violations_in_block(
        &self,
        store: &VerdictStore,
        point_index: usize,
        key: &VerdictBlockKey,
    ) -> Result<Vec<StoredViolation>, VerdictStoreError> {
        match self {
            Self::TheJudgingUnitsFingerprint(version) => store.violations_in_block(key, version),
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

    /// 汇总行里这一处的版本：判法单元指纹那一套是它的头 8 字节；三种结论那一套是种类连每把不同的版本的头 8 字节（按点的次序、去重）。
    #[must_use]
    pub fn version_text(&self) -> String {
        let head = |bytes: &[u8]| hexadecimal_text(&bytes[..8]);
        match self {
            Self::TheJudgingUnitsFingerprint(version) => head(&version.0),
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

/// 这一趟要判的块（这一判法单元指纹下还没有判定块的、归自己的），按点与块序。
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
            if conclusions.is_stored_for_reuse(store, point_index, &point.block_key(&local))? {
                continue;
            }
            pending.push((point_index, local));
        }
    }
    Ok(pending)
}

/// 一块逐状态的红项登记成判定向量编号（按块内次序）：红项是空的（绝大多数状态）只编码、登记一次，后面的绿状态照那一个编号，
/// 与逐状态登记交回同一个编号（库里一种判定向量一个编号，[`VerdictStore::register_verdict_vector`]）。
///
/// # Errors
/// 登记不成（[`VerdictStore::register_verdict_vector`]）。
pub(super) fn verdict_vector_numbers_of_the_red_items_of_each_state(
    store: &mut VerdictStore,
    items_of_each_state: &[Vec<String>],
) -> Result<Vec<VerdictVectorNumber>, VerdictStoreError> {
    let mut number_of_the_green_verdict: Option<VerdictVectorNumber> = None;
    let mut numbers = Vec::with_capacity(items_of_each_state.len());
    for items in items_of_each_state {
        let number = match (items.is_empty(), number_of_the_green_verdict) {
            (true, Some(green)) => green,
            (true, None) => {
                let green =
                    store.register_verdict_vector(&encoded_verdict_of_the_red_items(items))?;
                number_of_the_green_verdict = Some(green);
                green
            }
            (false, _) => {
                store.register_verdict_vector(&encoded_verdict_of_the_red_items(items))?
            }
        };
        numbers.push(number);
    }
    Ok(numbers)
}

/// 把一块的逐状态红项写成判定块与违例（与 [`check_blocks_where`] 同一个写法）；判器交回的逐盘自证槽数连同基镜像那一份随块存。
#[cfg(feature = "gpu")]
#[allow(
    clippy::too_many_arguments,
    reason = "库、点、块、结论记在哪、判好的块、基镜像那一份逐盘槽数、账，各是各的东西"
)]
pub(super) fn store_judged_block(
    store: &mut VerdictStore,
    point_index: usize,
    point: &PlannedCrashPoint,
    local: &Range<u64>,
    conclusions: &ConclusionsStoredUnder<'_>,
    judged: GpuJudgedBlock,
    slots_of_each_device_on_the_base_image: &[(u32, u32)],
    tally: &mut CheckingTally,
) -> Result<(), VerdictStoreError> {
    for (offset, items) in judged.items_of_each_state.iter().enumerate() {
        if !items.is_empty() {
            tally.red_states += 1;
            if tally.red_texts.len() < RED_TEXTS_KEPT {
                tally.red_texts.push((
                    point.path_number.clone(),
                    local.start + u64::try_from(offset).expect("块内序号"),
                    items.join("；"),
                ));
            }
        }
    }
    let ready = gpu_judged_block_ready_to_store(
        store,
        local,
        judged,
        slots_of_each_device_on_the_base_image,
    )?;
    conclusions.store_block(
        store,
        point_index,
        &point.block_key(local),
        ready.verdict_block,
        ready.violating,
        ready.along,
    )?;
    tally.blocks_checked += 1;
    tally.states_checked += local.end - local.start;
    Ok(())
}

#[cfg(test)]
mod tests;
