//! 往一段历史里注入：测量跑记下每一步的调用计数与模型版本，按种子抽注入点与故障，逐个注入点重跑、累加结局；
//! 取号之后报了错、新号留在盘上的是哪一格也在这里判。

use super::*;

/// 测量跑里每一步跑完时记下的东西：这一步是什么、到这一步为止读 / 写 / 刷盘各调了多少次、模型到这一步提交过哪些版本、
/// 这一步跑完时盘上择到的系统配置带的实例代号（C378 那一格按它判「这一步取了号」）。
pub(super) struct StepMark {
    pub(super) position: StepPosition,
    pub(super) operation_kind: Option<HistoryOperationKind>,
    pub(super) calls: FaultCallCounts,
    pub(super) committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    system_configuration_instance: Option<InstanceGeneration>,
}

/// 取号之后这一步报了错、写进系统配置的新号没有回卷时，重开之后盘上是哪一格（C378（取号之后写行发布被拒，已烧掉的实例代号不回卷）；
/// 「认了，写成已知行为」，D23（journal 的角色与格式） 已定项 16 的射程）。这不是失败，是认下来的行为：
/// 只记账、在报告里点名，不进新发现。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AcquiredInstanceLeftAfterAFailedMount {
    /// 写行那次发布的根没落盘：新号一条根都没有，就此烧掉（下一次可写挂载按 max + 1 取下一个，给它写一行）。
    WithoutAnyRoot,
    /// 写行那次的根已落盘，之后暖机（或写行那次的系统配置槽轮换）报错：新号有根，挂载照样报错返回。
    WithTheRowPublishRoot,
}

impl AcquiredInstanceLeftAfterAFailedMount {
    /// 计数与报告里的名字。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot => "新号一条根都没有（烧掉）",
            AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot => "新号已有写行那次的根",
        }
    }
}

/// 一次注入落在取号之后、挂载报错、新号没回卷的那一格：注入点、这一步之前与重开之后盘上系统配置的实例代号、根环里最大的实例代号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcquiredInstanceLeft {
    pub drawn: DrawnFault,
    pub kind: AcquiredInstanceLeftAfterAFailedMount,
    pub system_configuration_instance_before_the_step: InstanceGeneration,
    pub system_configuration_instance_after_the_reopen: InstanceGeneration,
    pub highest_root_instance_after_the_reopen: Option<InstanceGeneration>,
}

/// 一段历史注入故障之后交回的东西。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryFaultInjection {
    pub seed: HistorySeed,
    /// 摆出来的注入点，按调用序号从小到大。
    pub drawn_faults: Vec<DrawnFault>,
    pub tally: FaultInjectionTally,
    pub known_red_hits: Vec<KnownRedAtAFault>,
    /// 同一段历史里同一个签名只留第一个。
    pub new_findings: Vec<FaultFinding>,
    /// 取号之后挂载报错、新号没回卷的那几次（C378；认下的已知行为，不算新发现）。
    pub acquired_instances_left: Vec<AcquiredInstanceLeft>,
}

/// 注入那一步是不是一次挂载（起点段里有取号与暖机、可写挂载），是就交回这一步之前盘上系统配置的实例代号。
/// 起点段之前是 mkfs 写的 0；别的步取测量跑里上一步跑完时的那个。不是挂载的几类操作不取号，交回 `None`
/// （管理员回退是挂着时的一次向前发布，不取号）。挂着时整理那一步按以批间崩溃收尾的那一种算挂载（臂上的注释写了为什么）。
fn system_configuration_instance_before_a_mount_step(
    marks: &[StepMark],
    segment: FaultedSegment,
) -> Option<InstanceGeneration> {
    match segment {
        FaultedSegment::TheStartingPoint => Some(MKFS_INSTANCE_GENERATION),
        FaultedSegment::Operation { operation_kind, .. } => match operation_kind {
            // 崩溃恢复抛弃根、最新那一槽系统配置读回全 0 的那两种（每块盘都藏、只藏第一块盘）也是可写挂载：取号、写行、暖机；
            // 掉电在 journal 记录之前的那次覆盖写之后也挂一次（掉电那一截不写系统配置）。
            HistoryOperationKind::CloseAndMountWritable
            | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
            | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
            | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros
            // 挂着时整理以批间崩溃收尾的那一种（`CompactionRunEnding::CrashesAfterBatchesThenMountsWritable`）推完批之后，执行器
            // （`history::apply_compact_while_mounted`）调可写挂载那一步同一个 `apply_mount_writable` 挂一次：取号、写行、暖机。
            // 整理那几次发布用的是会话开着时那次挂载取的号、不换号，所以这一步之前的号就是上一步跑完时的那个。推到停的那一种不挂载、不取号，
            // 重开读回的号不比这一步之前大，`acquired_instance_left_after_a_failed_mount` 照样交回 `None`。故障注入的比重
            // `GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED`（`broad-compaction`）会抽到这一步。
            | HistoryOperationKind::CompactWhileMounted => {
                let index = marks
                    .iter()
                    .position(|mark| mark.position == segment.position())?;
                marks[index.checked_sub(1)?].system_configuration_instance
            }
            HistoryOperationKind::PublishFirstFile
            | HistoryOperationKind::PublishOverwrite
            | HistoryOperationKind::PublishWithoutUnits
            | HistoryOperationKind::RollBackWhileMounted
            | HistoryOperationKind::RaiseRollbackFloor
            | HistoryOperationKind::ColdStartRecover
            | HistoryOperationKind::Unlink
            | HistoryOperationKind::CreateInode
            | HistoryOperationKind::CreateSnapshot
            | HistoryOperationKind::ReadSnapshot
            | HistoryOperationKind::DestroySnapshot
            // 截断经挂着的会话发，不挂载、不取号。
            | HistoryOperationKind::Truncate
            // 建 inode、按 inode 号写同样经挂着的会话发。
            | HistoryOperationKind::CreateInodes
            | HistoryOperationKind::WriteOfAnInode
            // 按偏移写同样经挂着的会话发。
            | HistoryOperationKind::WriteAtOffset => None,
        },
    }
}

/// 一次挂载报了错之后重开的盘上，系统配置的实例代号比这一步之前大（这一步取了号、没回卷）：按根环里有没有带新号的根分两格。
/// 系统配置择不出、或者号没变大（取号自己那几次写报错、回卷做成了），交回 `None`。
pub(super) fn acquired_instance_left_after_a_failed_mount(
    image: &MemoryPool,
    marks: &[StepMark],
    drawn: DrawnFault,
) -> Option<AcquiredInstanceLeft> {
    let before = system_configuration_instance_before_a_mount_step(marks, drawn.segment)?;
    let system_configuration = choose_system_configuration(image).ok()?;
    let after = system_configuration.quantities.journal_instance;
    if after <= before {
        return None;
    }
    let roots = readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let kind = if roots.iter().any(|root| root.instance == after) {
        AcquiredInstanceLeftAfterAFailedMount::WithTheRowPublishRoot
    } else {
        AcquiredInstanceLeftAfterAFailedMount::WithoutAnyRoot
    };
    Some(AcquiredInstanceLeft {
        drawn,
        kind,
        system_configuration_instance_before_the_step: before,
        system_configuration_instance_after_the_reopen: after,
        highest_root_instance_after_the_reopen: roots.iter().map(|root| root.instance).max(),
    })
}

/// 这个镜像上，最新那条根带的回退下界 F 落不落在回退留下的空档里（与活盘面、崩溃状态两路同一个谓词、同一份实现）。
pub(super) fn raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(
    image: &MemoryPool,
) -> Option<bool> {
    let system_configuration = choose_system_configuration(image).ok()?;
    let newest_root = choose_root(image, &system_configuration)?;
    raised_floor_lands_only_on_abandoned_roots(image, newest_root.rollback_floor)
}

/// 随机注入抽 [`InjectedFault`] 的全部六种，等概率。前四种是里程碑「增补 3」第 4 件逐字写的
/// 「让任意一次写、读、刷盘返回 `BlockDeviceError`，或让一次读返回改坏的字节」；
/// 后两种是**设备说谎**（[`InjectedFault::WriteIsSwallowed`]、[`InjectedFault::BarrierIsSwallowed`]），
/// 改法一把它们也抽进样本——它们只有各自的孤立单测，那两条只证明注得进去，
/// 不证明被测代码在这两种故障下的行为被看过。
///
/// 说谎那两种的判定档按 [`InjectedFault::the_device_lies`] 走：panic 与「重开悄悄读回没提交过的内容」照判，
/// 盘上被丢掉一份内容留下的不一致单记一格、不算新发现。一次被吞的屏障在这套装置里不改变任何东西
/// （录制流留着每一次真正落盘的写，重建镜像不看屏障），它落在「注入了、什么也没发生」那一格，
/// 计数里照样有它一笔——`fired` 记的是注入发没发生，不是有没有后果。
fn draw_injected_fault(random: &mut SeededRandomSource) -> InjectedFault {
    let variety = usize::try_from(random.below(6)).expect("小于 6");
    match variety {
        0 => InjectedFault::WriteFails,
        1 => InjectedFault::ReadFails,
        2 => InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FlippedBit {
                byte_index: random.next_word(),
                bit_index: u32::try_from(random.below(8)).expect("小于 8"),
            },
        },
        3 => InjectedFault::BarrierFails,
        4 => InjectedFault::WriteIsSwallowed,
        5 => InjectedFault::BarrierIsSwallowed,
        // `below(6)` 的值域就是 0..6，第七种写不出来。
        other => unreachable!("按 below(6) 抽出来的只有 0..6，抽到了 {other}"),
    }
}

/// 这一段之前整池发过多少次读 / 写 / 刷盘：第 0 段（起点段）之前一次都没有。
fn calls_before(marks: &[StepMark], index: usize) -> FaultCallCounts {
    match index.checked_sub(1) {
        None => FaultCallCounts::default(),
        Some(previous) => marks[previous].calls,
    }
}

/// 按种子在测量跑记下的调用里摆注入点：先抽一种注入，再在「发过这一种调用」的那几段里均匀抽一段，
/// 最后在那一段发出的那几次调用里均匀抽一次。先抽段再抽调用（不是在整条调用流里均匀抽）是为了摊开到各类操作上——
/// 一次发布发几十次写、一次冷启动只发几十次读，整条流上均匀抽会把注入点全堆在写得最多的那几步里。
/// 同一个 (种类, 序号) 只摆一次。
///
/// 候选从第 0 段起算：第 0 段是起点段（mkfs、取号、暖机、第一个文件）。改法二把它放开，
/// 同一次把 `HistoryPool::start` 那四处 `expect` 改成可失败——只放开候选区间会把「注入之后 core panic 0 次」
/// 这个读数弄脏（判定五：攻方在只改候选区间的探针上实测 49 次 panic，全在起点段那几处 `expect` 上）。
fn draw_faults(
    seed: HistorySeed,
    marks: &[StepMark],
    faults_per_history: usize,
) -> Vec<DrawnFault> {
    let mut random = SeededRandomSource::from_seed(seed.0 ^ 0x46_41_55_4c_54_00_00_01);
    let mut drawn: Vec<DrawnFault> = Vec::new();
    let mut taken: BTreeSet<(FaultCallKind, u64)> = BTreeSet::new();
    for _ in 0..faults_per_history {
        let fault = draw_injected_fault(&mut random);
        let kind = fault.call_kind();
        let candidate_steps: Vec<usize> = (0..marks.len())
            .filter(|index| marks[*index].calls.of(kind) > calls_before(marks, *index).of(kind))
            .collect();
        if candidate_steps.is_empty() {
            continue;
        }
        let picked = candidate_steps[usize::try_from(
            random.below(u64::try_from(candidate_steps.len()).expect("段数")),
        )
        .expect("下标")];
        let first = calls_before(marks, picked).of(kind) + 1;
        let last = marks[picked].calls.of(kind);
        let call_ordinal = first + random.below(last - first + 1);
        if !taken.insert((kind, call_ordinal)) {
            continue;
        }
        let segment = match (marks[picked].position, marks[picked].operation_kind) {
            (StepPosition::StartingPoint, None | Some(_)) => FaultedSegment::TheStartingPoint,
            (StepPosition::Operation(step_index), Some(operation_kind)) => {
                FaultedSegment::Operation {
                    step_index,
                    operation_kind,
                }
            }
            // 每一步的观察都带操作（`execute_history_with_faults` 在 `StepPosition::Operation` 上一律给 `Some`）；
            // 真出现这一格就是观察者那边变了，那一次不摆。
            (StepPosition::Operation(_), None) => continue,
        };
        drawn.push(DrawnFault {
            fault,
            call_ordinal,
            segment,
        });
    }
    drawn.sort();
    drawn
}

/// 跑一段历史，按种子在它起点之后的读 / 写 / 刷盘里摆 `faults_per_history` 个注入点，每个注入点重跑一遍这段历史，
/// 判「返回错误而不是 panic」与「重开恢复到模型允许的版本」。
#[must_use]
pub fn inject_faults_into_history(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    faults_per_history: usize,
) -> HistoryFaultInjection {
    inject_faults_into_history_handing_out_the_images(
        history,
        execution,
        faults_per_history,
        &mut |_| {},
    )
}

/// 同 [`inject_faults_into_history`]，另把每个注入点重开的盘交给 `hand_out`（[`ImagesAfterAFault`]）：harness 不在上面跑池级 checker，
/// 那一遍归给定状态引擎两段，结论交回 [`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`] 归类。交出去的次序是注入点的次序。
#[must_use]
pub fn inject_faults_into_history_handing_out_the_images(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    faults_per_history: usize,
    hand_out: &mut dyn FnMut(ImagesAfterAFault),
) -> HistoryFaultInjection {
    let mut tally = FaultInjectionTally {
        histories: 1,
        ..FaultInjectionTally::default()
    };
    let marks = measure_history(history, execution, &mut tally);
    let drawn_faults = draw_faults(history.seed, &marks, faults_per_history);
    inject_the_drawn_faults(
        history,
        execution,
        &marks,
        drawn_faults,
        tally,
        PoolCheckerOfTheImagesAfterAFault::HandedOutToTheGivenStatesEngine(hand_out),
    )
}

/// 给定状态引擎判回的、一个注入点交出去的那几张盘上池级 checker 判红的（[`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`]
/// 收它）。违例细节可以是占位的一句（GPU 判的、从判定库读回的没有细节）：归类只看判红的是哪几条。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolCheckerVerdictsAfterAFault {
    /// 与 [`ImagesAfterAFault::step_images_of_the_run_with_the_fault`] 逐张对应。
    pub violations_of_the_step_images: Vec<Vec<(&'static str, String)>>,
    /// 重开那张盘（[`ImagesAfterAFault::image_after_the_fault`]）上判红的。
    pub violations_after_the_fault: Vec<(&'static str, String)>,
    /// 被吞的写插回去之后那张盘（[`ImagesAfterAFault::image_with_the_swallowed_writes_landed`]）上判红的不变量；没吞写时空。
    pub invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet<&'static str>,
}

/// 同 [`inject_faults_into_history`]，池级 checker 那一遍由给定状态引擎判（`pool_checker_verdicts_of` 收一个注入点交出去的盘，交回给定状态引擎
/// 从判定库读回的结论），照 harness 就地判、判红就停的口径并进 harness 自己的判定，交回与就地判逐字相同的失败观察与归类
/// （要产物不变的调用方是 E158 的故障注入）：
///
/// 1. 每个注入点先照不判池级 checker 跑一遍，交出去判；
/// 2. 带着注入跑的那一遍里一张每步镜像都不红：历史与判红就停那一档跑的是同一段，重开那张盘与被吞的写插回去那张盘的结论并进归类；
/// 3. 有一张判红：判红就停那一档的历史停在那一步（之后的步不跑，注入点在那之后的就没走到，重开的是那一步的盘）。照那几张的结论把这个注入点
///    截短重跑一遍（每步镜像那个钩子收调用方给的结论，`history::execute_history_stopping_at_the_given_pool_checker_violations`），
///    重跑那一遍交出去的盘再交一次判，重开那张盘与被吞的写插回去那张盘的结论并进归类。
///
/// 一个注入点因此交出去一次或两次。判红就停那一档里池级 checker 自己 panic 会被历史接住、记成那一步的 panic；这里给定状态引擎判不出
/// panic 来，那一格不罩（推的：E158 入库产物里没有这种行）。判红就停那一档在重开的盘上记的每条不变量判绿、不适用的计数
/// （[`FaultInjectionTally::invariant_holds`] 那几格）不记：结论里只有判红的。
///
/// # Panics
/// 交回的每步镜像结论与交出去的每步镜像张数不同；截短重跑那一遍交出去的每步镜像与整段那一遍判红那一张之前（含）逐张不同
/// （同一个种子、同一个注入点重跑，不同就是执行器不确定了）。
#[must_use]
pub fn inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    faults_per_history: usize,
    pool_checker_verdicts_of: &mut dyn FnMut(&ImagesAfterAFault) -> PoolCheckerVerdictsAfterAFault,
) -> HistoryFaultInjection {
    let mut tally = FaultInjectionTally {
        histories: 1,
        ..FaultInjectionTally::default()
    };
    let marks = measure_history(history, execution, &mut tally);
    let drawn_faults = draw_faults(history.seed, &marks, faults_per_history);
    inject_the_drawn_faults(
        history,
        execution,
        &marks,
        drawn_faults,
        tally,
        PoolCheckerOfTheImagesAfterAFault::MergedFromTheGivenStatesEngine(pool_checker_verdicts_of),
    )
}

/// 同 [`inject_faults_into_history`]，注入点不按种子抽，由调用方写死：`segment` 那一段发出的第 `call_within_the_segment` 次
/// `fault` 那一种调用（1 起）。写死的用例用它把注入点摆在点名的那一次调用上（C378：取号之后的写行、暖机）。
///
/// # Panics
/// 测量跑没走到那一段，或者那一段发出的这一种调用不到 `call_within_the_segment` 次（写死的用例把注入点摆错了）。
#[must_use]
pub fn inject_one_fault_into_the_segment(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    fault: InjectedFault,
    segment: FaultedSegment,
    call_within_the_segment: u64,
) -> HistoryFaultInjection {
    inject_one_fault_into_the_segment_handing_out_the_images(
        history,
        execution,
        fault,
        segment,
        call_within_the_segment,
        &mut |_| {},
    )
}

/// 同 [`inject_one_fault_into_the_segment`]，另把那个注入点重开的盘交给 `hand_out`（同 [`inject_faults_into_history_handing_out_the_images`]）。
///
/// # Panics
/// 同 [`inject_one_fault_into_the_segment`]。
#[must_use]
pub fn inject_one_fault_into_the_segment_handing_out_the_images(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    fault: InjectedFault,
    segment: FaultedSegment,
    call_within_the_segment: u64,
    hand_out: &mut dyn FnMut(ImagesAfterAFault),
) -> HistoryFaultInjection {
    let mut tally = FaultInjectionTally {
        histories: 1,
        ..FaultInjectionTally::default()
    };
    let marks = measure_history(history, execution, &mut tally);
    let index = marks
        .iter()
        .position(|mark| mark.position == segment.position())
        .unwrap_or_else(|| panic!("测量跑没走到{}", segment.render()));
    let kind = fault.call_kind();
    let calls_in_the_segment = marks[index].calls.of(kind) - calls_before(&marks, index).of(kind);
    assert!(
        (1..=calls_in_the_segment).contains(&call_within_the_segment),
        "{}只发了 {calls_in_the_segment} 次{}，摆不到第 {call_within_the_segment} 次",
        segment.render(),
        kind.name()
    );
    let drawn = DrawnFault {
        fault,
        call_ordinal: calls_before(&marks, index).of(kind) + call_within_the_segment,
        segment,
    };
    inject_the_drawn_faults(
        history,
        execution,
        &marks,
        vec![drawn],
        tally,
        PoolCheckerOfTheImagesAfterAFault::HandedOutToTheGivenStatesEngine(hand_out),
    )
}

/// 测量跑：不注入，不跑池级 checker（那一档由随机历史与崩溃注入罩着），只要每一步的调用数、模型目录与系统配置的实例代号。
fn measure_history(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    tally: &mut FaultInjectionTally,
) -> Vec<StepMark> {
    let geometry = execution.device_width.fixed_geometry();
    let measurement_execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: execution.device_width,
        space_admission: execution.space_admission,
    };
    let measurement_plan = SharedFaultPlan::unarmed(geometry);
    let measurement_stream = SharedStream::new();
    let mut marks: Vec<StepMark> = Vec::new();
    let mut committed_so_far: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let measurement_run = execute_history_with_faults(
        history,
        measurement_execution,
        &measurement_stream,
        &measurement_plan,
        &mut |observation| {
            for (key, file) in observation.model.committed_versions() {
                committed_so_far.insert(key, file);
            }
            marks.push(StepMark {
                position: observation.position,
                operation_kind: observation.operation.map(HistoryOperation::kind),
                calls: measurement_plan.calls(),
                committed_versions: committed_so_far.clone(),
                system_configuration_instance: choose_system_configuration(observation.image)
                    .ok()
                    .map(|system_configuration| system_configuration.quantities.journal_instance),
            });
        },
    );
    match measurement_run.ending {
        HistoryEnding::Completed => {}
        HistoryEnding::KnownRed { .. } | HistoryEnding::NewFinding { .. } => {
            tally.histories_whose_measurement_run_stopped_early = 1;
        }
    }
    marks
}

/// 每个注入点重开的盘上池级 checker 那一遍怎么办（[`inject_the_drawn_faults`]）。harness 自己一条不变量都不判。
enum PoolCheckerOfTheImagesAfterAFault<'hook> {
    /// 盘交给钩子，那一遍归给定状态引擎两段，结论交回 [`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`] 归类。
    HandedOutToTheGivenStatesEngine(&'hook mut dyn FnMut(ImagesAfterAFault)),
    /// 给定状态引擎判回的结论交进来，照判红就停那一档的口径并进 harness 的判定
    /// （[`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`]）。
    MergedFromTheGivenStatesEngine(
        &'hook mut dyn FnMut(&ImagesAfterAFault) -> PoolCheckerVerdictsAfterAFault,
    ),
}

/// 摆好的每个注入点各重跑一遍这段历史（[`run_one_fault_up_to_the_pool_checker`]），池级 checker 那一遍照 `pool_checker` 交出去或并进来，
/// 攒成一段的结果。
fn inject_the_drawn_faults(
    history: &GeneratedHistory,
    execution: HistoryExecution,
    marks: &[StepMark],
    drawn_faults: Vec<DrawnFault>,
    mut tally: FaultInjectionTally,
    pool_checker: PoolCheckerOfTheImagesAfterAFault<'_>,
) -> HistoryFaultInjection {
    if drawn_faults.is_empty() {
        tally.histories_without_any_injection_point = 1;
    }
    let mut known_red_hits = Vec::new();
    let mut new_findings: Vec<FaultFinding> = Vec::new();
    let mut acquired_instances_left: Vec<AcquiredInstanceLeft> = Vec::new();
    {
        let mut accumulator = FaultInjectionAccumulator {
            tally: &mut tally,
            known_red_hits: &mut known_red_hits,
            new_findings: &mut new_findings,
            acquired_instances_left: &mut acquired_instances_left,
        };
        match pool_checker {
            PoolCheckerOfTheImagesAfterAFault::HandedOutToTheGivenStatesEngine(hand_out) => {
                for drawn in &drawn_faults {
                    let one_fault = run_one_fault_up_to_the_pool_checker(
                        history, execution, marks, drawn, None,
                    );
                    settle_one_fault_with_the_pool_checker_verdicts(
                        &one_fault,
                        Vec::new(),
                        BTreeSet::new(),
                        &mut accumulator,
                    );
                    // 重开的盘交出去：池级 checker 那一遍归给定状态引擎两段
                    accumulator.tally.images_after_the_fault_handed_out += 1;
                    hand_out(one_fault.images);
                }
            }
            PoolCheckerOfTheImagesAfterAFault::MergedFromTheGivenStatesEngine(
                pool_checker_verdicts_of,
            ) => {
                for drawn in &drawn_faults {
                    merge_one_fault_with_the_pool_checker_verdicts(
                        history,
                        execution,
                        marks,
                        drawn,
                        pool_checker_verdicts_of,
                        &mut accumulator,
                    );
                }
            }
        }
    }
    HistoryFaultInjection {
        seed: history.seed,
        drawn_faults,
        tally,
        known_red_hits,
        new_findings,
        acquired_instances_left,
    }
}

/// 一批注入攒出来的东西：计数、「已知红」命中、新发现、取号之后挂载报错而新号没回卷的那几次。
pub(super) struct FaultInjectionAccumulator<'run> {
    pub(super) tally: &'run mut FaultInjectionTally,
    pub(super) known_red_hits: &'run mut Vec<KnownRedAtAFault>,
    pub(super) new_findings: &'run mut Vec<FaultFinding>,
    pub(super) acquired_instances_left: &'run mut Vec<AcquiredInstanceLeft>,
}

impl FaultInjectionAccumulator<'_> {
    /// 一条注入之后的失败归好类之后记账：「已知红」照形态计、说谎的设备留下的按签名计、新发现同一个签名只留第一个。
    pub(super) fn settle_a_failure_after_a_fault(
        &mut self,
        seed: HistorySeed,
        drawn: DrawnFault,
        classified: FailureAfterAFault,
        reopen_text: &str,
    ) {
        match classified {
            FailureAfterAFault::KnownRed(form) => self.record_a_known_red_hit(seed, form, drawn),
            FailureAfterAFault::LeftByTheLyingDevice { signature } => {
                self.tally
                    .faults_where_a_lying_device_left_the_image_inconsistent += 1;
                *self
                    .tally
                    .lying_device_signatures
                    .entry(format!("{signature:?}"))
                    .or_insert(0) += 1;
            }
            FailureAfterAFault::NewFinding {
                signature,
                observation,
            } => {
                self.tally.faults_ending_new_finding += 1;
                if !self
                    .new_findings
                    .iter()
                    .any(|finding| finding.signature == signature)
                {
                    self.new_findings.push(FaultFinding {
                        signature,
                        seed,
                        drawn,
                        observation: *observation,
                        reopen: reopen_text.to_string(),
                    });
                }
            }
        }
    }

    /// 一次注入以「已知红」收尾：按形态计一次，再记下是哪个种子上的哪一次注入。
    fn record_a_known_red_hit(
        &mut self,
        seed: HistorySeed,
        form: KnownRedFormAtAFault,
        drawn: DrawnFault,
    ) {
        *self.tally.faults_ending_known_red.entry(form).or_insert(0) += 1;
        self.known_red_hits
            .push(KnownRedAtAFault { seed, form, drawn });
    }
}

/// 注入那一段的结局是不是「入口返回了错误」：没 panic、checker 没判红，而且那一段要么返回了 `Err`，
/// 要么是冷启动或读快照、读回报了错。注入的块设备错在模型里没有对应的拒绝理由（`ObservedRefusalReason::Unexplained`），
/// 模型因此必然报「该成却拒了」——那一格不算失败，它就是这一件要的结果。
///
/// 起点段（改法二放开的那一段）按 `HarnessJudgement::TheStartingPointReturnedAnError` 认：起点段的四个入口
/// 在执行器里不再是 `expect`，报错时把是哪一步、报的什么交回来，那正是这一件要的结果。
pub(super) fn the_fault_surfaced_as_an_error(
    run: &HistoryRun,
    observation: &FailureObservation,
    segment: FaultedSegment,
) -> bool {
    if observation.position != segment.position()
        || observation.panic.is_some()
        || !observation.violations.is_empty()
    {
        return false;
    }
    if let FaultedSegment::TheStartingPoint = segment {
        return matches!(
            observation.harness_judgement,
            Some(HarnessJudgement::TheStartingPointReturnedAnError { .. })
        );
    }
    match run.outcomes.last() {
        Some(StepOutcome::Refused { .. }) => true,
        Some(StepOutcome::Applied(AppliedEffect::Recovered { read_back, .. })) => match read_back {
            ColdStartReadBack::Failed => true,
            ColdStartReadBack::NoFile | ColdStartReadBack::FileRead => false,
        },
        Some(StepOutcome::Applied(
            AppliedEffect::Published { .. }
            | AppliedEffect::Mounted { .. }
            | AppliedEffect::RaisedFloor { .. }
            | AppliedEffect::RolledBack { .. }
            | AppliedEffect::SnapshotPublished { .. }
            | AppliedEffect::Compacted { .. },
        ))
        | Some(StepOutcome::NotApplicable(_))
        | None => false,
        // 读快照读失败（注入的读错落在它要读的那几处上）：同冷启动读回报错，是注入的错浮出来了。
        Some(StepOutcome::Applied(AppliedEffect::SnapshotRead { read_back, .. })) => {
            match read_back {
                ObservedSnapshotReadBack::Failed { .. } => true,
                ObservedSnapshotReadBack::NoFile
                | ObservedSnapshotReadBack::File(_)
                | ObservedSnapshotReadBack::NoSnapshotWithThisTree => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 随机抽的六种全抽得到：`draw_injected_fault` 的 `below(6)` 改回 `below(4)`，说谎那两种一次都抽不到，这里必红
    /// （改法一钉的那一格；`crates/mutations.tsv` 同名的那一条）。
    #[test]
    fn the_random_draw_reaches_all_six_injected_faults() {
        let mut random = SeededRandomSource::from_seed(0x46_41_55_4c_54_44_52_57);
        let mut drawn: BTreeSet<&'static str> = BTreeSet::new();
        for _ in 0..200 {
            drawn.insert(draw_injected_fault(&mut random).name());
        }
        assert_eq!(
            drawn.len(),
            6,
            "六种注入都该抽得到，200 次里抽到的是 {drawn:?}"
        );
    }
}
