//! 从环境起一趟：一趟读的旋钮（[`EnvironmentRun`]）、覆盖行、改环境的锁与还原守卫，
//! 与整趟的入口（[`run_from_environment`]、[`run_from_environment_with_observers`]）。

use super::*;

/// 按环境跑一趟的结果：录了多少、核了多少、对账剩多少、违例几条（带秒数，只 `PartialEq`）。
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentRun {
    pub mode: CrashAmplificationRunMode,
    pub share: HostShare,
    pub cards_used: usize,
    pub recording: RecordingTally,
    pub checking: CheckingTally,
    /// 这条流自己的块里还没判的（[`unjudged_blocks_of_the_flow`]）；库里别的流的块不算。给了块区间（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）
    /// 只数本片（[`unjudged_blocks_of_the_flow_among_the_counted_blocks`]），[`EnvironmentRun::violations`] 同。
    pub unjudged_blocks: usize,
    pub violations: Vec<StoredRedState>,
    pub coverage_lines: Vec<String>,
    /// 库的名字（连的是 prune-server 哪一处、哪一个键空间）：只进行里报的名字。
    pub library_name: String,
    /// 第 ② 段跑了就有：在哪算、核了多少。
    pub verification: Option<VerificationOutcome>,
    /// 第 ③ 段：核对表与判定块逐状态比的结果。
    pub comparison: Option<JudgeVerifierComparison>,
    /// 第 ① 段在卡上判的（配置开了 GPU、表建得出）就有。
    pub gpu_judge: Option<GpuJudgeOutcome>,
    /// 这一趟只跑了录入段（[`RunStage::RecordOnly`]）：流程与事实表（配了 GPU 再加判表）已写进库、读回核过，一块都没判。
    pub recorded_only: bool,
    /// 这一趟只排了派活计划（`SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1`）：录入照做，一块都没判。
    pub placement_only: bool,
    /// 只排派活计划的那一趟打的那一行（[`placement_line_of_the_pending_blocks`]，`CRASH_AMPLIFICATION_PLACEMENT` 起头）；别的趟是 `None`。
    pub placement_line: Option<String>,
    /// 这一趟被叫停了（[`stop_is_requested_by_the_environment`]，等 GPU 批处理服务判完、核完的时候被叫停的也算），或第 ① 段判器显存不够
    /// （直接在卡上判的那一路，文件头第 5 条）：判过的块在库里，没做完的段留给下一趟，对账不为空是正常的；
    /// 分项驱动照它把这一项记成本次未跑。
    pub stopped_early: bool,
    /// 这一趟每个点各种结论的版本。
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
    /// 这一趟判段在这个进程里取的卡从哪来（[`GpuCardsSource`]）：第 ① 段、第 ② 段配了 GPU 交 GPU 批处理服务、只排派活计划那一趟只数块，
    /// 都不取卡，恒为 [`GpuCardsSource::NoCardNeeded`]；观察者交回的盘那一刻取的卡（[`pool_checker_of_the_images_after_the_observers`]）不在这里报。
    pub gpu_cards_taken_from: GpuCardsSource,
    /// 按段登记表里这条流登记了的段（J10）：这一趟照「甲二」展开、计入跳过量，行里报段号与写数（[`reuse_and_scale_fields_of_the_line`]）。
    pub registered_segments: Vec<RegisteredSegment>,
    /// 这条流里没登记、展开上限又盖不住的段（[`unregistered_segments_beyond_the_expansion`]），行里报段号与写数。
    pub unregistered_segments_beyond_the_expansion: Vec<SegmentWithItsWriteCount>,
    /// 登记了的段各自的域（`full` / `slice`）与分片的段到这一趟末判到哪（[`domains_and_progress_of_the_registered_segments`]，C790）。
    pub registered_segment_domains: Vec<DomainAndProgressOfARegisteredSegment>,
    /// 挂在节点上的观察者的判段这一趟拼了什么、这条流挂了观察者的块读回来的计数（J4、J7）；没挂观察者、只录、只排派活计划、被叫停、
    /// 这台机器一块没选中（[`EnvironmentRun::no_block_selected_here`]）的那一趟是 `None`。
    pub observing: Option<ObservingOfTheRun>,
    /// 观察者的录入段这一趟录了什么（「观察者改走两段」）：没挂观察者、只判的那一趟是 `None`。
    pub observer_recording: Option<ObserverRecordingTally>,
    /// 配置开了 GPU 而这份二进制没编进 gpu 特性：判段这一趟没跑（本次未跑，不判红、不退回 CPU 判），这是原因；录入段照录。
    /// 那一趟 `recorded_only` 也是真。
    pub not_run_without_the_gpu_feature: Option<String>,
    /// 计数向量与计划对不上的块，一块一句（J7，每一趟都判）；对得上是空的。只排派活计划、被叫停的那一趟不判，是空的。
    pub count_vector_disagreements: Vec<String>,
    /// 这一趟拒算的判法单元键，一把一句（[`JudgingUnitsKeyRefused::sentence`]）：不空时这一趟在一次性的库里照做、什么都没进长期库，
    /// 这条流的块全部算拒算（[`JUDGING_UNITS_REFUSED_LINE_PREFIX`] 那两行）。
    pub refused_judging_units: Vec<String>,
    /// 这一趟判这条流的哪一片（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]；没设是整条流）。
    pub block_range: BlockRangeOfTheFlow,
    /// 这条流照读回的计划一共几块（块序号 `0..blocks_of_the_flow`）。
    pub blocks_of_the_flow: u64,
    /// 这一趟打的 [`BLOCK_RANGE_LINE_PREFIX`] 那一行（[`block_range_line`]，登了分片段的流是 [`block_range_line_of_a_part`]）；
    /// 没设区间变量是 `None`、没打。
    pub block_range_line: Option<String>,
    /// 登了分片段的流设了区间：这一趟判的那一份与那一段（[`selection_of_the_part_range`]）；[`EnvironmentRun::block_range`] 是它在
    /// 这一趟计划里点到的块序号。没设区间、没登分片段的流是 `None`。
    pub part_of_a_sliced_flow: Option<PartOfASlicedFlowSelected>,
    /// 设了区间变量（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）、这台机器这一趟一块都没选中（空区间，或份与区间没有交集），判段在写待判行、
    /// 起判器之前交回：第 ①②③ 段、哨兵与观察者的判段都没跑，对账与读违例照这一片数。没设区间变量的那一趟恒为假（只按份分、
    /// 这一份一块没领到的也照判完那一趟交回）；只排派活计划的那一趟也不走这一支（它照常打 PLACEMENT 行，要判的块是空的）。
    pub no_block_selected_here: bool,
}

impl EnvironmentRun {
    /// 这一趟第 ① 段那个判器的结论记在库里哪一处（对账、读违例照它）。
    #[must_use]
    pub fn judged_conclusions(&self) -> ConclusionsStoredUnder<'_> {
        self.conclusion_versions.stored_under(self.judged_by)
    }
}

/// 一趟用的卡从哪来（定义在 `gpu_memory_ledger`）。
pub use crate::gpu_memory_ledger::GpuCardsSource;

/// 每一行 `CRASH_AMPLIFICATION ` 末尾都接的几个字段（汇总行、被叫停的行、`flow=` 行、两台跑的分份行，打法都在
/// `crates/singlefs-verification/tests/` 里调 [`run_from_environment`] 的用例）：
///
/// ` states_reused=<n> exhaustive=<true|false> expand_up_to=<n> ignored_points=<n> states_skipped_by_ignore=<n> registered_segments=<…> unregistered_segments_beyond_expansion=<…>`
///
/// - `states_reused`：这一趟分给本机的块（`SINGLEFS_CRASH_AMPLIFICATION_SHARE=<i>/<n>` 那一份；没设是整份）里，录入时库中已有这一判法单元指纹
///   的判定、这一趟因此没判的态数（别的机器判完写进服务那一份的块照复用算）。每一趟照实报自己的（R1 判决 L1：叫停、只排计划都不写 0）：
///   被叫停的那一趟（[`stop_is_requested_by_the_environment`]）报它录入时库里已判的；只排派活计划的那一趟同样报（它的
///   `CRASH_AMPLIFICATION_PLACEMENT` 行也带）。所以同一份库上几趟的行加起来会重复计：叫停之后接着跑的那一趟、两台各领一份之后不分片的那一趟数到的，
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
/// - `registered_segment_slices`：`slice` 的段按片记的账 `<段号>:<累计判完几片>/<共几片>:<every_state_judged|rest_not_run_this_time>`
///   （[`slice_account_of_a_sliced_segment`]；片数装不进 u64 写下界 `at_least_18446744073709551616`）。余下没判的这一段这一趟不算判过、
///   余下的是本次未跑，入口照它报；没有 `slice` 的段写 `-`。
///
/// - `block_range`、`blocks_of_the_flow`：只在设了块区间（[`BLOCK_RANGE_ENVIRONMENT_VARIABLE`]）时接在最后（[`BlockRangeOfTheFlow::fields_of_the_line`]）：
///   这一趟判的那一段 `<起块>-<止块>`（空区间照写 `<块>-<块>`）与这条流一共几块；没设时这两个字段不打。只录、只排派活计划的那一趟
///   不打这一行，同样两个字段在 [`BLOCK_RANGE_LINE_PREFIX`] 那一行上（每一趟都打）。登了分片段的流换成
///   ` block_range=<份号>/<起块>-<止块> parts_of_the_flow=<份数> blocks_of_the_part=<这一份的块数>`（[`PartOfASlicedFlowSelected::fields_of_the_line`]）。
///
/// 五个说段的字段由 [`segment_fields_of_the_line`] 拼。
#[must_use]
pub fn reuse_and_scale_fields_of_the_line(run: &EnvironmentRun) -> String {
    format!(
        " states_reused={} exhaustive={} expand_up_to={} ignored_points={} states_skipped_by_ignore={}{}{}",
        run.states_reused,
        run.exhaustive,
        run.expand_up_to.0,
        run.ignored_points,
        run.states_skipped_by_ignore,
        segment_fields_of_the_line(
            &run.registered_segments,
            &run.unregistered_segments_beyond_the_expansion,
            &run.registered_segment_domains
        ),
        run.part_of_a_sliced_flow.as_ref().map_or_else(
            || run.block_range.fields_of_the_line(run.blocks_of_the_flow),
            PartOfASlicedFlowSelected::fields_of_the_line
        )
    )
}

/// 这一趟对账：没给块区间数整条流（[`unjudged_blocks_of_the_flow`]；两台各领一份时也数整条流，合并那一趟照它判齐），
/// 给了只数本片（那一段里的块，不分份；空区间一块不数）。交回没判的块数。
///
/// # Errors
/// 库读失败。
fn unjudged_blocks_counted_by_the_block_range(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    block_range: &BlockRangeOfTheFlow,
) -> Result<usize, String> {
    match block_range {
        BlockRangeOfTheFlow::EveryBlockOfTheFlow => {
            unjudged_blocks_of_the_flow(store, plan, conclusions)
        }
        BlockRangeOfTheFlow::BlocksInTheRange(_) => {
            unjudged_blocks_of_the_flow_among_the_counted_blocks(
                store,
                plan,
                conclusions,
                &|block| block_range.holds(block),
            )
        }
    }
    .map(|unjudged| unjudged.len())
    .map_err(|error| format!("对账失败：{error:?}"))
}

/// 这一趟读违例：数哪些块同 [`unjudged_blocks_counted_by_the_block_range`]（复用的块红过照样读到）。
///
/// # Errors
/// 库读失败。
pub(super) fn violations_counted_by_the_block_range(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    block_range: &BlockRangeOfTheFlow,
) -> Result<Vec<StoredRedState>, String> {
    match block_range {
        BlockRangeOfTheFlow::EveryBlockOfTheFlow => {
            read_the_violations_stored_under(store, plan, conclusions)
        }
        BlockRangeOfTheFlow::BlocksInTheRange(_) => {
            read_the_violations_stored_under_among_the_counted_blocks(
                store,
                plan,
                conclusions,
                &|block| block_range.holds(block),
            )
        }
    }
    .map_err(|error| format!("读违例失败：{error:?}"))
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

// ============================== 覆盖行：这一趟里每个（入口，提交步骤，段）展没展开 ==============================

/// 覆盖行的行首（[`CrashFlowCoverageLine::text`]）。
pub const CRASH_FLOW_COVERAGE_LINE_PREFIX: &str = "CRASH_FLOW_COVERAGE";

/// 一段在这一趟里落到哪一种（覆盖行的 `expansion=` K3「三样都含」：ignore 的点、按段登记的段、展开上限之外的段都算没展开）。
/// 判的次序与计划相同：归预设 ignore 的点先于按段登记，按段登记先于这一段自己的展开方式（[`CrashPlan::segment_expansion`]、`expansion_in_the_recording`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SegmentTreatmentInTheRun {
    /// 这一段照它的展开方式展开了（全部真子集或甲二）。
    Expanded,
    /// 这一段归一个预设 ignore 的点（[`CrashPointSpan::ignored`]），一律不展开。
    IgnoredPoint,
    /// 这一段在按段登记表里（[`RegisteredSegment`]），照这一趟的展开上限展开（[`expansion_of_a_registered_segment`]）。
    RegisteredSegment,
    /// 这一段的展开方式是不展开：崩溃放量是写数超过展开上限，崩溃枚举用例是调用方给的展开不展开它。
    NotExpanded,
}

impl SegmentTreatmentInTheRun {
    /// 覆盖行里的写法。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Expanded => "expanded",
            Self::IgnoredPoint => "ignored_point",
            Self::RegisteredSegment => "registered_segment",
            Self::NotExpanded => "not_expanded",
        }
    }
}

/// 一行覆盖行：一个（入口，提交步骤，段）在这一趟里怎么处置。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CrashFlowCoverageLine {
    /// 写所在崩溃点的点名 `<入口>.<写的种类>.<第几段>` 的第一截（点名里没有 `.` 时整个点名就是入口）。
    pub entry_name: String,
    pub step_kind: StepKind,
    /// 段在这条流里的序号，从 0 起，与 [`CrashFlow::segments`] 同序。
    pub segment_index: usize,
    pub treatment: SegmentTreatmentInTheRun,
}

impl CrashFlowCoverageLine {
    /// `CRASH_FLOW_COVERAGE entry=<入口名> step=<StepKind 成员名> segment=<段序号> expansion=<expanded|ignored_point|registered_segment|not_expanded>`。
    /// 覆盖表脚本按这几段逐字读：改写法要同改它。
    #[must_use]
    pub fn text(&self) -> String {
        format!(
            "{CRASH_FLOW_COVERAGE_LINE_PREFIX} entry={} step={} segment={} expansion={}",
            self.entry_name,
            step_kind_member_name(self.step_kind),
            self.segment_index,
            self.treatment.name()
        )
    }
}

/// [`StepKind`] 的成员名（枚举里声明的那个名字，不是种类串 [`StepKind::name`] 的蛇形写法）：覆盖表脚本拿它对枚举体的成员。
#[must_use]
pub fn step_kind_member_name(step_kind: StepKind) -> &'static str {
    match step_kind {
        StepKind::ZeroFill => "ZeroFill",
        StepKind::Discard => "Discard",
        StepKind::UnitWrite => "UnitWrite",
        StepKind::JournalRecord => "JournalRecord",
        StepKind::RootRecordFua => "RootRecordFua",
        StepKind::SystemConfigurationSlot => "SystemConfigurationSlot",
        StepKind::Barrier => "Barrier",
    }
}

/// 这一趟录下的写流的覆盖行：每次写取（它所在崩溃点的入口，它带的 [`StepKind`]，它所在的段），连同那一段在这一趟里怎么处置
/// （[`SegmentTreatmentInTheRun`]）；同一（入口，步骤，段，处置）只留一行，按段序、段内写序排。`registered_segments` 是这一趟
/// 按段登记的段（崩溃枚举用例没有按段登记，给空的）。
///
/// # Panics
/// 有一段没有写，或有一次写不落在任何崩溃点里（崩溃点的写区间没罩住整张写表；计划那一步同样拒）。
#[must_use]
pub fn crash_flow_coverage_lines(
    flow: &CrashFlow<'_>,
    registered_segments: &[RegisteredSegment],
) -> Vec<CrashFlowCoverageLine> {
    let owner_of_segment = owner_of_each_segment(flow);
    let treatment_of_segment: Vec<SegmentTreatmentInTheRun> = flow
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            if flow.crash_points[owner_of_segment[segment_index]].ignored {
                SegmentTreatmentInTheRun::IgnoredPoint
            } else if registered_segments
                .iter()
                .any(|registered| registered.segment_index == segment_index)
            {
                SegmentTreatmentInTheRun::RegisteredSegment
            } else {
                match (flow.expansion)(segment_index, segment) {
                    CrashPointReplaySegmentExpansion::NotExpanded => {
                        SegmentTreatmentInTheRun::NotExpanded
                    }
                    CrashPointReplaySegmentExpansion::EveryProperSubset
                    | CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll => {
                        SegmentTreatmentInTheRun::Expanded
                    }
                }
            }
        })
        .collect();
    let mut lines_already_taken: std::collections::HashSet<CrashFlowCoverageLine> =
        std::collections::HashSet::new();
    let mut lines = Vec::new();
    for (segment_index, segment) in flow.segments.iter().enumerate() {
        for write_index in segment {
            let point = flow
                .crash_points
                .iter()
                .find(|point| point.writes.contains(write_index))
                .expect("崩溃点的写区间首尾相接罩住整张写表：每次写都落在某个崩溃点里");
            let entry_name = point.name.split_once('.').map_or(
                point.name.as_str(),
                |(before_the_first_dot, _period_and_step)| before_the_first_dot,
            );
            let line = CrashFlowCoverageLine {
                entry_name: entry_name.to_string(),
                step_kind: flow.writes[*write_index].kind,
                segment_index,
                treatment: treatment_of_segment[segment_index],
            };
            if lines_already_taken.insert(line.clone()) {
                lines.push(line);
            }
        }
    }
    lines
}

/// 这一份（`is_mine` 选的块）里录入时库中已有这一处可复用结论的态数，照读回的那一份答（`reuse_read` 读过 `is_mine` 选的每一块，
/// [`ReuseOfTheBlocksReadOnce::is_stored_for_reuse`]），不再逐块读库。
fn states_already_judged_in_this_share(
    store: &VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    reuse_read: &ReuseOfTheBlocksReadOnce,
    is_mine: &dyn Fn(u64) -> bool,
) -> u64 {
    let mut states = 0u64;
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if is_mine(this_block)
                && reuse_read.is_stored_for_reuse(
                    store,
                    conclusions,
                    point_index,
                    &point.block_key(&local),
                    this_block,
                )
            {
                states += local.end - local.start;
            }
        }
    }
    states
}

/// 只核给定序号的那些块（块序号 = 全部点的块按次序编号）；[`check_recorded_blocks`] 是它按权重分工的形态。
/// 每领一块之前先问 `stop_is_requested`（[`check_blocks_where_until`]）；库每一次读写现取现放（[`check_blocks_where_until_with_threads`]）。
///
/// # Errors
/// 同 [`check_recorded_blocks`]。
pub fn check_selected_blocks(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    selected_block_sequences: &std::collections::HashSet<u64>,
    conclusions: &ConclusionsStoredUnder<'_>,
    stop_is_requested: &(dyn Fn() -> bool + Sync),
) -> Result<CheckingTally, VerdictStoreError> {
    let threads = checking_threads();
    check_blocks_where_until_with_threads(
        library,
        flow,
        plan,
        &|block_sequence| selected_block_sequences.contains(&block_sequence),
        conclusions,
        stop_is_requested,
        threads,
    )
}

/// [`run_from_environment`] 一趟（连同它调的、这份文件与 [`crate::crash_library`] 里的函数）读的全部环境变量。调它的用例照这张表清掉、跑完还原
/// （`tests/common_crash_points/mod.rs` 的环境守卫），不读调用方与别的用例留下的值；这份文件与 `crash_library.rs` 里每一处按名字读环境变量
/// （连同按常量读的）都要在表里，`crash_library.rs` 读的连 prune-server 哪一处的三个变量除外（清掉它们，临时服务底下的用例会落到真服务上；
/// `crash_amplification::tests` 里两条用例逐个核，各管一份文件）。
pub const ENVIRONMENT_VARIABLES_READ_BY_A_RUN: &[&str] = &[
    "SINGLEFS_MULTI_HOST_CONFIG",
    "GPU_CARDS_BY_PRIORITY",
    // 借卡通道的三个尝试变量（`crate::gpu_card_configuration::CardLeaseChannelOfTheCaller`，三个都在才算有通道）：守卫里的用例看不见
    // 调用方的通道，丢守卫时照原值设回。判段取卡不问通道；卡池照调用方那一份（`GpuConfigurationOfTheCaller::of_this_process`，守卫改环境之前记下）借，不读此刻的环境
    crate::gpu_card_configuration::CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES[0],
    crate::gpu_card_configuration::CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES[1],
    crate::gpu_card_configuration::CARD_LEASE_ATTEMPT_ENVIRONMENT_VARIABLES[2],
    SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE,
    "SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY",
    "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
    BLOCK_RANGE_ENVIRONMENT_VARIABLE,
    "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS",
    "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE",
    REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE,
    crate::crash_point_replay_progress::CRASH_POINT_REPLAY_FINDINGS_FILE_ENVIRONMENT_VARIABLE,
    RUN_STAGE_ENVIRONMENT_VARIABLE,
    CHECKING_THREADS_ENVIRONMENT_VARIABLE,
    EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
    CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE,
    REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE,
    REUSE_SENTINEL_COUNT_ENVIRONMENT_VARIABLE,
];

/// 同一个测试二进制里改进程环境、读 [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里变量的用例一条接一条跑：几条一起改同一批变量会互相读到对方的值。
/// 这份库的单测与 `tests/` 下各个测试二进制各有一把（一个进程一份）。
static ONE_TEST_CHANGES_THE_ENVIRONMENT_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 拿在手里的改环境的那把锁（[`ONE_TEST_CHANGES_THE_ENVIRONMENT_AT_A_TIME`]），丢掉时放。
/// 次序：要开一次性库（`crate::crash_library::DisposableLibrary`，单测构建里取一次性库名额）的用例先开库、后拿这把锁；
/// 反过来拿，名额满着时拿着名额等这把锁的用例与拿着这把锁等名额的用例互相等。单测构建里拿着它的线程记一笔
/// （`this_thread_holds_the_environment_lock`），要新占名额的那一步见到就当场 panic（这条线程已经起着一份的不等名额、不查）。
pub struct EnvironmentLockOfThisTestBinary {
    _one_test_at_a_time: std::sync::MutexGuard<'static, ()>,
}

#[cfg(test)]
thread_local! {
    /// 这条线程此刻拿着改环境的那把锁（[`EnvironmentLockOfThisTestBinary`] 建时记、丢时清；那把锁不可重入，一条线程至多拿一份）。
    static THIS_THREAD_HOLDS_THE_ENVIRONMENT_LOCK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

impl EnvironmentLockOfThisTestBinary {
    fn taken_by_this_thread() -> Self {
        // 上一条拿着锁的用例 panic 了：它的环境已经在丢守卫时还原，锁照样能用
        let one_test_at_a_time = ONE_TEST_CHANGES_THE_ENVIRONMENT_AT_A_TIME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        #[cfg(test)]
        THIS_THREAD_HOLDS_THE_ENVIRONMENT_LOCK.with(|holds| holds.set(true));
        Self {
            _one_test_at_a_time: one_test_at_a_time,
        }
    }
}

#[cfg(test)]
impl Drop for EnvironmentLockOfThisTestBinary {
    fn drop(&mut self) {
        THIS_THREAD_HOLDS_THE_ENVIRONMENT_LOCK.with(|holds| holds.set(false));
    }
}

/// 单测构建里这条线程此刻拿没拿着改环境的那把锁（[`EnvironmentLockOfThisTestBinary`]）。
#[cfg(test)]
pub(crate) fn this_thread_holds_the_environment_lock() -> bool {
    THIS_THREAD_HOLDS_THE_ENVIRONMENT_LOCK.with(std::cell::Cell::get)
}

/// 拿着改环境的那把锁、不改环境：只读环境起引擎（[`GivenStatesEngine::start_from_environment`]）的用例在这把锁里起，
/// 别的用例拿着 [`CrashAmplificationEnvironment`] 改变量时它等着，不读到对方设的叫停文件与分片。引擎起步时把要读的都读完，
/// 锁在起完就放。拿着 [`CrashAmplificationEnvironment`] 的用例不许再拿它（同一把锁，不可重入）。要开一次性库的先开库、再拿它
/// （[`EnvironmentLockOfThisTestBinary`] 的次序）。
#[must_use]
pub fn one_test_changes_the_environment_at_a_time() -> EnvironmentLockOfThisTestBinary {
    EnvironmentLockOfThisTestBinary::taken_by_this_thread()
}

/// 给定状态表的引擎起步时读一次叫停文件的路径（[`stop_file_of_the_environment`]），之后每次问叫停只看那个文件在不在：同一个进程里
/// 别的用例拿着环境守卫换叫停文件时，跑着的引擎不被它叫停。记住叫停的那一个收 `'static` 的读法，闭包漏在堆上（一个引擎几十字节）。
pub(super) fn stop_request_remembering_the_stop_file_of_the_engine_start(
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
    pub fn set_variable(&mut self, name: &str, value: &std::ffi::OsStr) {
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
    _one_test_at_a_time: EnvironmentLockOfThisTestBinary,
}

impl CrashAmplificationEnvironment {
    /// 要开一次性库的用例先开库、再建它（[`EnvironmentLockOfThisTestBinary`] 的次序）。
    ///
    /// # Panics
    /// `settings` 里有不在 [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里的变量。
    #[must_use]
    pub fn cleared_then_set(settings: &[(&str, &std::ffi::OsStr)]) -> Self {
        let one_test_at_a_time = EnvironmentLockOfThisTestBinary::taken_by_this_thread();
        let mut variables =
            EnvironmentRestoredOnDrop::saved_then_cleared(ENVIRONMENT_VARIABLES_READ_BY_A_RUN);
        for (name, value) in settings {
            variables.set_variable(name, value);
        }
        Self {
            variables,
            _one_test_at_a_time: one_test_at_a_time,
        }
    }

    /// 用例跑到一半再设一个（例如第二趟之前放叫停文件）。
    ///
    /// # Panics
    /// 同 [`EnvironmentRestoredOnDrop::set_variable`]。
    pub fn set_variable(&mut self, name: &str, value: &std::ffi::OsStr) {
        self.variables.set_variable(name, value);
    }

    /// 把守卫清掉的显卡那几样设回调用方给这个进程的那一份（[`crate::gpu_card_configuration::GpuConfigurationOfTheCaller::of_this_process`]，
    /// 守卫改环境之前记下的）：多机配置在哪（`SINGLEFS_MULTI_HOST_CONFIG`，调用方设了才设回：仓副本不是 git 工作树，证红与合入验证在副本里跑时
    /// 配置只经这个变量交进来，清掉之后照 git common-dir 找不到），与服务分给这个进程的卡（`GPU_CARDS_BY_PRIORITY`，
    /// [`crate::gpu_card_configuration::GpuConfigurationOfTheCaller::cards_handed_out_by_the_service`]）。
    /// 用例照调用方的多机配置跑、那一趟要上卡时照服务交的卡取，不照多机配置自己挑卡（`crate::gpu_card_configuration::cards_by_priority_from`）。
    /// 没经服务给卡的不设卡：要卡的那一趟照没经服务给卡那一句拒（判段取卡不问借卡通道，守卫清掉的借卡通道不设回）。
    pub fn set_the_multi_host_configuration_and_the_cards_of_the_caller(&mut self) {
        self.set_the_multi_host_configuration_and_the_cards_of(
            crate::gpu_card_configuration::GpuConfigurationOfTheCaller::of_this_process(),
        );
    }

    /// 同 [`Self::set_the_multi_host_configuration_and_the_cards_of_the_caller`]，调用方那一份由参数给（单测造一份）。
    fn set_the_multi_host_configuration_and_the_cards_of(
        &mut self,
        caller: &crate::gpu_card_configuration::GpuConfigurationOfTheCaller,
    ) {
        if let Some(configuration) = caller.multi_host_configuration_handed_in() {
            self.set_variable(
                crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
                configuration.as_os_str(),
            );
        }
        if let Some(cards) = caller.cards_handed_out_by_the_service() {
            self.set_variable(
                crate::gpu_card_configuration::CARDS_BY_PRIORITY_KEY,
                std::ffi::OsStr::new(cards),
            );
        }
    }
}

/// 从环境跑一趟：配置定跑法，`SINGLEFS_CRASH_AMPLIFICATION_SHARE` 定这台机器领哪一份；库是 prune-server 里那一份
/// （[`LibraryOpenInThisProcess::open_or_share`]，两台各判各领的那一份、直接写进服务，不取回、不导入）；同一条命令再跑一次就是续跑：
/// 同一（块键，结论种类，版本）下判过的块全部复用，只补没判的；
/// 跑法带 GPU 时卡按服务交的优先级用（服务清场、分卡时经环境变量 `GPU_CARDS_BY_PRIORITY` 交，[`parse_cards_by_priority`]；没交而配置开了 GPU 报错，
/// [`crate::gpu_card_configuration::cards_by_priority_from`]；
/// 每项点 `HOSTS` 里的地址，这一台只取自己地址那几项、这一台是哪一项由多机配置判法认，[`crate::gpu_card_configuration::multi_host_addresses_of_the_caller`]）：任务先装第一张卡，装到它的显存额度为止，装不下的才给下一张；没交卡串（没设、空串或只有空白）
/// 不找本机的卡（`gpu` 特性没编进来而配置写了 `ENABLE_GPU=1`：判段这一趟报本次未跑、录入段照录，不悄悄退回 CPU 判，
/// [`judge_not_run_without_the_gpu_feature`]）。判在哪只看本机配置的 GPU 开关
/// （[`computation_routes`]）：开了 GPU 判器、核对、单元校验和三类都走 GPU。
/// 开了 GPU 只由 GPU 判器判，CPU 不在同一段里判同一批块、不对拍；没开只由 CPU 判器判。
/// 结论按（块键，结论种类，版本）存（J8）：两条路线同一把版本，由结论种类与判器引到的被检测产品代码定（[`ConclusionVersionsOfThePlan`]，判器的代码不进键）。
/// `SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1` 只排派活计划：打一行 `CRASH_AMPLIFICATION_PLACEMENT …`（第 ① 段要判、第 ② 段要核多少块、多少个状态，
/// [`placement_line_of_the_pending_blocks`]），不取卡、不试判，一块都不判。
/// 主机之间等分（`SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=<w0>,<w1>,…` 可以改成按权重）。
/// 设了 [`BLOCK_RANGE_ENVIRONMENT_VARIABLE`] 只判这条流点到的那一段块（与份取交集），对账与读违例只数那一段（[`EnvironmentRun::block_range`]），
/// 定下这一片之后打一行 [`BLOCK_RANGE_LINE_PREFIX`]；这台机器一块都没选中的那一趟在写待判行、起判器之前交回（[`EnvironmentRun::no_block_selected_here`]）。
/// 没设区间变量的那一趟不走这一支：只按份分、这一份一块没领到的也照判完那一趟往下走。
/// 登了分片段的流设了区间：分片的段的片由区间里的份与份内块定（[`selection_of_the_part_range`]，不读库里的进度），录入键带片，
/// 不推进分片进度，驱动行报的分片进度是库里从编号 0 起连着判完的地方。
/// `expand_up_to` 是这一趟的展开上限，进交回的结果与 `CRASH_AMPLIFICATION_GPU_TOTAL` 行；流自己的展开方式要与它逐段相同。
///
/// # Errors
/// 环境变量写法错、库打不开（连同不是这一版布局的库：换一个新目录建库）、判定存储读写失败；
/// 流的展开方式与 `expand_up_to` 在某一段上不同（报的展开上限就不是这一趟枚举的那个，在任何录入之前拒）；结论版本与编译时对不上
/// （开库之前拒）；开了 GPU 而第 ①、② 段的待判行写不成（判段的库在临时 prune-server 里而登记不成、判法单元拒算）；只排派活计划而没开 GPU；块区间写法错或没点到这条流（录入之前拒，
/// [`BlockRangesOfTheFlows::written_range_of_the_flow`]），或止块过了这条流的块数（[`BlockRangesOfTheFlows::range_of_the_flow`]，录入之后、判之前拒）。
#[allow(
    clippy::too_many_lines,
    reason = "读环境、核版本、开库、录入、按卡分工核对与对照、对账、打行，是一趟的全部步骤，拆开要把十几个量来回传"
)]
pub fn run_from_environment(
    flow: &CrashFlow<'_>,
    flow_name: &str,
    toolchain: &CrashPointReplayToolchainIdentity,
    expand_up_to: ExpandUpToWritesPerSegment,
) -> Result<EnvironmentRun, String> {
    run_from_environment_with_observers(flow, flow_name, &[], toolchain, expand_up_to)
}

/// 同 [`run_from_environment`]，另带挂在节点上的观察者（[`ObserverOnTheNodes`]；两段）：
/// 录入段把这台机器领的块里落在观察者节点上的逐状态看一遍（[`record_the_observers_of_the_flow`]，CPU；交回镜像的按恢复镜像去重），结果写进库；
/// 判段从库里读回，观察者交回的盘上的池级 checker 配了 GPU 在卡上判、没配由 CPU 判，逐状态拼回结论存进库
/// （[`judge_the_observers_of_the_flow`]）；这条流挂了观察者的每一块（这一趟拼的与库里复用的）的计数读回来交在 [`EnvironmentRun::observing`]，
/// 调用方照它判观察者计数的断言（每一趟都判，J7）。观察者的版本开库之前与编译时那一份核，对不上同判器的版本整趟不写库。
/// 调用方挂的观察者之外，按这条流在来历表里那一行另挂几个（`crate::crash_point_replay_provenance`：登记位上、来历是 mkfs 返回之后的流
/// 挂两槽全废盘面观察者，它在判段从判器交回的逐盘自证槽数看），挂在流的每个点与末节点上，结果同样在 [`EnvironmentRun::observing`] 里。
///
/// # Errors
/// 同 [`run_from_environment`]；按段登记表读不了、写坏了、登记的段与这条流的写表对不上（[`RegisteredSegment`]）；观察者看一块时库读写失败；
/// 来历表里没有这条流、有几行，或调用方挂的观察者与按来历挂的同名（开跑之前拒，什么都不录）。
///
/// 工具链那一个参数这里不用：工具链不进录入键（`crate::crash_library::recording_key_of_the_scenario_and_the_inputs`），
/// 观察者的版本不带工具链；调用方都交 `toolchain_of_this_process()`。
#[allow(
    clippy::too_many_lines,
    reason = "读环境、核版本、开库、录入、按卡分工核对与对照、看观察者、对账、打行，是一趟的全部步骤，拆开要把十几个量来回传"
)]
pub fn run_from_environment_with_observers(
    flow: &CrashFlow<'_>,
    flow_name: &str,
    observers_of_the_caller: &[ObserverOnTheNodes<'_>],
    _toolchain: &CrashPointReplayToolchainIdentity,
    expand_up_to: ExpandUpToWritesPerSegment,
) -> Result<EnvironmentRun, String> {
    // 按来历统一挂：查来历表在一切之前，查不到就拒、什么都不录
    let observers_with_the_ones_attached_by_the_provenance =
        crate::crash_point_replay_provenance::observers_on_the_nodes_with_the_ones_attached_to_the_flow(
            crate::crash_point_replay_provenance::RunOfTheFlow::CrashAmplification,
            flow_name,
            observers_of_the_caller,
            &crate::crash_point_replay_provenance::nodes_of_every_point_of_the_flow(flow),
        )
        .map_err(|refusal| format!("崩溃放量的流 {flow_name}：{}", refusal.sentence()))?;
    let observers = observers_with_the_ones_attached_by_the_provenance.as_slice();
    // 两段：录入段在 CPU 上跑数据、写进库；判段只从库里读，段与段之间不在内存里交数据
    let stage = RunStage::from_the_environment()?;
    // 多机配置只有一份（J11、M4 第 4 条）：设了 SINGLEFS_MULTI_HOST_CONFIG 照它，没设按当前目录的 git common-dir 找主工作树的 multi-host.env
    let MultiHostConfigurationOfThisProcess {
        configuration_file,
        mode,
    } = multi_host_configuration_of_this_process()?;
    // 观察者交回的盘上取卡的显卡来路，在入口照此刻的环境读一次、往下传，判段里不再读进程环境
    let card_sources_of_the_run =
        CardSourcesOfTheEngineStart::read_from_the_environment_now(configuration_file);
    // 配置开了 GPU 而这份二进制没编进 gpu 特性（不带特性编的那一趟）：判段这一趟不跑、报本次未跑（不判红、不退回 CPU 判），
    // 录入段照录（CPU 跑数据、写进库），带特性那一趟从库里读回、在卡上判（「配 GPU 必用 GPU」）
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
    let placement_only = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY")
        .is_ok_and(|value| value == "1");
    let (share, host_weights) = host_share_and_weights_of_the_share_variables()?;
    // 按块序号区间选块：写法与点没点到这条流在录入之前核，止块过没过块数等排出计划再核
    let block_ranges = BlockRangesOfTheFlows::from_the_environment()?;
    if let Some(ranges) = &block_ranges {
        ranges.written_range_of_the_flow(flow_name)?;
    }
    // 没登记的观察者：挂着的观察者开库之前在编进二进制的那一份登记里认一遍（不读树；判器、核对器与观察者的代码不进键），
    // 有一个没登记就打一行点名它那一组与形态；它照判不入库（取这个进程的占位，键里带着它的行只留在这个进程里），
    // 别的照键入库同一份库，这条流的块全部计入拒算（refused_blocks=），照常交回（不退 78）
    let refused_judging_units = refused_judging_units_of_the_run(observers);
    for refusal in &refused_judging_units {
        println!(
            "{JUDGING_UNITS_REFUSED_LINE_PREFIX} flow={flow_name} units={} form={} {}",
            refusal.units().group_name(),
            refusal.form_name(),
            refusal.sentence()
        );
    }
    // 库：prune-server 里那一份（几类测试同一份，不收选库的变量）；判法单元拒算的那几组照判不入库，同一份库（不另建一次性库）
    let planning_started = std::time::Instant::now();
    // 库每一次读写现取现放（LibraryTakenAtEachReadOrWrite）：判、核、看观察者、排计划时手里不握锁，同一个进程里别的用例照样读写这条连接
    let library_of_this_process = LibraryOpenInThisProcess::open_or_share()?;
    let library = LibraryTakenAtEachReadOrWrite::SharedInThisProcess(&library_of_this_process);
    let library_name = library.take().library_name().to_string();
    library
        .take()
        .keep_the_rows_of_these_versions_out_of_the_library(refused_judging_units.iter().map(
            |refusal| *placeholder_judging_units_key_of_the_refused(refusal.units()).fingerprint(),
        ));
    // 录入段的输入（两段都照它现算：判段照同一份输入算录入键、读清单，L11）：按段登记（J10）改展开、定片、排好要录的流程字节
    // 按段登记（J10）：登记了的段照「甲二」展开，开跑前核段写数与段写表摘要，对不上就拒跑（M4 第 15 条）
    let registered_segments_of_the_recording =
        registered_segments_of_the_flow_from_the_environment(flow, flow_name)?;
    let is_registered = |segment_index: usize| {
        registered_segments_of_the_recording
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
    // 登记了的段照这一趟的展开上限展开、超过的取甲二
    let expansion_with_the_registered_segments = |segment_index: usize, segment: &[usize]| {
        expansion_in_the_recording(
            &registered_segments_of_the_recording,
            flow,
            expand_up_to,
            segment_index,
            segment,
        )
    };
    // 判的就是这条流，登记了的段换成「甲二」展开（没登记一段时与调用方给的逐段相同）
    let flow_of_the_recording = &CrashFlow {
        base: flow.base,
        writes: flow.writes,
        segments: flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: flow.versions,
        versions_without_a_file: flow.versions_without_a_file,
        expansion: &expansion_with_the_registered_segments,
        crash_points: flow.crash_points.clone(),
    };
    let exhaustive_of_the_recording = expand_up_to.covers_every_segment(flow.segments)
        && registered_segments_of_the_recording.is_empty();
    // 设了块区间、这条流登了分片段（登记段整域装不进 u64）：片由区间里的份与份内块定，不读库里的进度（录入之前、写库之前核）
    let part_selection = match &block_ranges {
        Some(ranges) => {
            let domains_of_the_recording = domains_of_the_registered_segments(
                flow_of_the_recording,
                &registered_segments_of_the_recording,
            )?;
            if domains_of_the_recording
                .iter()
                .any(|domain| domain.kind == DomainKindOfARegisteredSegment::Slice)
            {
                let part_range = ranges.part_range_of_the_sliced_flow(flow_name)?;
                // 排找份的那份计划要算态数（分片的段片长为 0）：先核装得进 u64
                states_of_the_flow_to_record(
                    flow_of_the_recording,
                    &placeholder_slices_of_the_sliced_segments(
                        flow.segments.len(),
                        &domains_of_the_recording,
                    ),
                )?;
                Some(
                    selection_of_the_part_range_of_the_flow(
                        flow_of_the_recording,
                        &domains_of_the_recording,
                        &part_range,
                        BlocksPerPartAsAPowerOfTwo::OF_THE_AMPLIFICATION,
                    )
                    .map_err(|refused| refused.sentence(flow_name))?,
                )
            } else {
                None
            }
        }
        None => None,
    };
    let part_of_the_run = part_selection
        .as_ref()
        .map(|selection| selection.part.clone());
    // 登记段整域（甲二 ∪ 展开 6）装得进 u64 的全展开，装不进的分片（C790）：越过 1280 位的在写库之前拒跑
    let segment_slices = match &part_selection {
        Some(selection) => selection.segment_slices.clone(),
        None => segment_slices_of_the_recording_from_the_environment_and_the_library(
            &library.take(),
            flow_of_the_recording,
            flow_name,
            &registered_segments_of_the_recording,
            routes.crash_judge,
        )?,
    };
    // 排计划要算态数：先核装得进 u64，装不进在写库之前拒跑，不在算态数那一步 panic
    states_of_the_flow_to_record(flow_of_the_recording, &segment_slices)?;
    // 分片的段按片记账照这个进程的片长数（录入段定片时已经核过它；没有分片的段不读这个变量）
    let states_per_slice = if segment_slices.iter().any(Option::is_some) {
        registered_segment_slice_states_from_the_environment()?
    } else {
        DEFAULT_REGISTERED_SEGMENT_SLICE_STATES
    };
    // 录入键不随片变（[`recording_of_the_run`]）：只判那几趟照清单里录下的片判，不照此刻的续点另定片；
    // 登了分片段的流照区间判的那一趟录入键带片（[`recording_of_the_run_keyed_by_its_slices`]），只判那一段读到的是自己这一段的清单
    let RecordingOfTheRun {
        recorded_flow_bytes,
        recording_key: recording_key_of_this_run,
    } = match &part_selection {
        Some(_) => recording_of_the_run_keyed_by_its_slices(
            flow_name,
            flow_of_the_recording,
            &segment_slices,
            &registered_segments_of_the_recording,
            exhaustive_of_the_recording,
            routes.crash_judge == ComputedOn::Gpu,
        ),
        None => recording_of_the_run(
            flow_name,
            flow_of_the_recording,
            &segment_slices,
            &registered_segments_of_the_recording,
            exhaustive_of_the_recording,
            routes.crash_judge == ComputedOn::Gpu,
        ),
    };
    // 录入段：排计划、把流程与事实表（配了 GPU 再加判表）写进库、按录入键登清单；录入键下已有清单就不再录（L11：命中就跳过）
    if records {
        let plan_of_the_recording =
            plan_crash_points_with_slices(flow_of_the_recording, &segment_slices);
        // 录入键下的清单录的片与这一趟定的片相同才复用；片挪了（上一片判完了）就重录，同一把键下换上新的清单
        let manifest_under_the_recording_key =
            recorded_run_manifest_reused_under(&library.take(), &recording_key_of_this_run)?;
        let (manifest, recording_reused) = match manifest_under_the_recording_key
            .filter(|manifest| manifest.recorded_flow_digest == sha256_digest(&recorded_flow_bytes))
        {
            Some(manifest) => (manifest, true),
            None => (
                record_the_flow_into_the_library(
                    &library,
                    flow_of_the_recording,
                    &recorded_flow_bytes,
                    &plan_of_the_recording,
                    &registered_segments_of_the_recording,
                    exhaustive_of_the_recording,
                    routes.crash_judge == ComputedOn::Gpu,
                    &recording_key_of_this_run.key,
                )?,
                false,
            ),
        };
        println!(
            "CRASH_AMPLIFICATION_RECORDED flow={flow_name} recorded_flow={} facts={} packed_judge_tables={} blocks={} exhaustive={} recording_reused={recording_reused}",
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
            manifest.exhaustive,
        );
    }
    drop(recorded_flow_bytes);
    // 判段：只从库里读回录入段那一趟（清单、流程、事实表、配了 GPU 时的判表），下面判、核对、比对都只用读回的这几样
    let read_back = read_back_the_recorded_run(
        &library,
        &recording_key_of_this_run.key,
        flow_name,
        routes.crash_judge == ComputedOn::Gpu,
    )?;
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
    // 覆盖行：覆盖表脚本从这一趟的日志里读哪个（入口，提交步骤，段）在哪个域里展开了。照读回的流与清单里的按段登记打，
    // 只录、只排派活计划、被叫停的那一趟同样打（录下的流是同一条）
    for coverage_line in crash_flow_coverage_lines(&flow_of_the_library, &registered_segments) {
        println!("{}", coverage_line.text());
    }
    // 结论的版本（J8）：由结论种类与判器引到的被检测产品代码定（判器的代码不进键）；观察者的版本照它的判定项标识取，没登记的取这个进程的占位、照判不入库（J4）
    let judged_by = match routes.crash_judge {
        ComputedOn::Gpu => JudgedBy::GpuJudge,
        ComputedOn::Cpu => JudgedBy::CpuJudge,
    };
    let conclusion_versions = ConclusionVersionsOfThePlan::of_the_plan(&plan);
    let mut observer_versions = Vec::with_capacity(observers.len());
    for observer in observers {
        let (versions, _observer_refused, versions_kept_out) =
            observer_versions_keeping_the_refused_out(observer, judged_by);
        let mut store = library.take();
        store.keep_the_rows_of_these_versions_out_of_the_library(versions_kept_out);
        observer_versions.push(versions);
    }
    if !refused_judging_units.is_empty() {
        println!(
            "{JUDGING_UNITS_REFUSED_LINE_PREFIX} flow={flow_name} refused_blocks={}",
            plan.points
                .iter()
                .map(|point| point.blocks().len())
                .sum::<usize>()
        );
    }
    let judged_conclusions = conclusion_versions.stored_under(judged_by);
    let needed_conclusions = vec![judged_conclusions.clone()];
    // 登记了的段的域与分片进度（驱动行两个字段，C790）：只录、只排计划的那几趟报录入时库里的
    let registered_segment_domains_at_recording =
        domains_and_progress_of_the_registered_segments_of_the_run(
            &library.take(),
            &flow_of_the_library,
            &plan,
            &registered_segments,
            &judged_conclusions,
            states_per_slice,
            part_of_the_run.as_ref(),
        )
        .map_err(|error| format!("读分片进度失败：{error:?}"))?;
    let mut report = CoverageReport::default();
    // 判器那一份结论问复用要读库的那一半整份计划只读这一次：录入时数复用、数这一份已判的态数、抽复用哨兵都照它答
    let reuse_read_of_the_judged_conclusions = judged_conclusions
        .reuse_read_of_the_plan(&library.take(), &plan, &|_| true)
        .map_err(|error| format!("数录入的块失败：{error:?}"))?;
    let recording = {
        let store = library.take();
        tally_the_recorded_blocks_from_the_reuse_reads(
            &store,
            &plan,
            &needed_conclusions,
            std::slice::from_ref(&reuse_read_of_the_judged_conclusions),
            &mut report,
        )
    };
    let host_shares = WeightedShares::new(host_weights);
    let total_blocks: u64 = plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数装得进 u64"))
        .sum();
    // 这一片（给了块区间是点到这条流的那一段）与这一份的交集；对账与读违例照这一片数，不分份。
    // 登了分片段的流：这一片是区间在这一趟计划里点到的块序号（录入之前照同一份流、同一几片算的，块数要对得上）
    let block_range = match (&block_ranges, &part_selection) {
        (None, _) => BlockRangeOfTheFlow::EveryBlockOfTheFlow,
        (Some(_), Some(selection)) => {
            if selection.blocks_in_the_plan != total_blocks {
                return Err(format!(
                    "{BLOCK_RANGE_ENVIRONMENT_VARIABLE} 给流 {flow_name} 的区间{}：录入之前照区间算的计划 {} 块，读回的流排出的计划 {total_blocks} 块，\
                     块序号对不上，不判（录下的流与这一趟的流不是同一条）",
                    selection.part.fields_of_the_line(),
                    selection.blocks_in_the_plan
                ));
            }
            BlockRangeOfTheFlow::BlocksInTheRange(selection.blocks_of_the_plan.clone())
        }
        (Some(ranges), None) => ranges.range_of_the_flow(flow_name, total_blocks)?,
    };
    let my_blocks = blocks_of_this_share_in_the_block_range(
        total_blocks,
        &host_shares,
        share.index,
        &block_range,
    );
    let selected: std::collections::HashSet<u64> = my_blocks.iter().copied().collect();
    // 设了区间变量：这一片定下来就打一行（下面每一种交回都在它之后），入口照它核一条流的几片罩没罩全
    let block_range_line_of_the_run = match &part_of_the_run {
        Some(part) => Some(block_range_line_of_a_part(
            flow_name,
            share,
            part,
            my_blocks.len(),
        )),
        None => block_range_line(
            flow_name,
            share,
            &block_range,
            total_blocks,
            my_blocks.len(),
        ),
    };
    if let Some(line) = &block_range_line_of_the_run {
        println!("{line}");
    }
    // 复用数在判之前数（别的机器判完写进服务的块照复用算）。叫停、只排计划的那一趟都照这个数报（L1）
    let states_reused_at_recording = states_already_judged_in_this_share(
        &library.take(),
        &plan,
        &judged_conclusions,
        &reuse_read_of_the_judged_conclusions,
        &|block| selected.contains(&block),
    );
    let ignored_points = plan.points.iter().filter(|point| point.ignored).count();
    let states_skipped_by_ignore = states_skipped_by_ignore(&flow_of_the_library, &plan);
    // 这一趟的叫停：观察者的录入段、第 ① 段的外层循环、卡线程、一轮收尾与下面整趟收尾都问这一个，见过一次就记住（
    // 攻方 F2：叫停文件中途被删时各自现读，轮末断言 panic、或整趟没记成提早收尾）
    let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);
    let stop_is_requested = || stop_request.is_requested();
    // 观察者的录入段（「观察者改走两段」）：这台机器领的块里落在观察者节点上的，CPU 上逐状态看（交回镜像的按恢复镜像去重），
    // 结果写进库；看的是从库里读回的流，判段读的是同一份。只排派活计划的那一趟不录（它只要排计划的数，录观察者要真挂载）；叫停了就停在块边上
    // （录过的块在库里，下面第 ① 段照叫停的样子收尾，下一趟接着录）
    let observer_recording = if records && !placement_only && !observers.is_empty() {
        let recording_started = std::time::Instant::now();
        let observer_recording_tally = record_the_observers_of_the_flow(
            &library,
            &flow_of_the_library,
            &plan,
            observers,
            &observer_versions,
            &|block| selected.contains(&block),
            &stop_is_requested,
        )?;
        println!(
            "CRASH_AMPLIFICATION_OBSERVERS_RECORDED flow={flow_name} blocks_recorded_here={} states_recorded_here={} blocks_already_recorded={} states_counted_here={} images_observed_fresh={} states_reusing_an_observed_image={} buckets={} witness_misses={} states_recovered_here={} seconds={:.1}",
            observer_recording_tally.blocks_recorded_here,
            observer_recording_tally.states_recorded_here,
            observer_recording_tally.blocks_already_recorded,
            observer_recording_tally.states_counted_here,
            observer_recording_tally.images_observed_here,
            observer_recording_tally.states_reusing_an_observed_image,
            observer_recording_tally.buckets,
            observer_recording_tally.witness_misses,
            observer_recording_tally.states_recovered_here,
            recording_started.elapsed().as_secs_f64()
        );
        Some(observer_recording_tally)
    } else {
        None
    };
    // 只跑录入段的那一趟（连同配置开了 GPU 而这份二进制没编进 gpu 特性、判段报本次未跑的那一趟）：库里那几样已经读回、核过摘要、
    // 照它排出了计划，到这里交回，一块都不判
    if !judges {
        let unjudged = unjudged_blocks_counted_by_the_block_range(
            &library.take(),
            &plan,
            &judged_conclusions,
            &block_range,
        )?;
        return Ok(EnvironmentRun {
            mode,
            share,
            cards_used: 0,
            recording,
            checking: CheckingTally::default(),
            unjudged_blocks: unjudged,
            violations: Vec::new(),
            coverage_lines: report.lines().to_vec(),
            library_name,
            verification: None,
            comparison: None,
            gpu_judge: None,
            placement_only: false,
            placement_line: None,
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
            block_range_line: block_range_line_of_the_run.clone(),
            part_of_a_sliced_flow: part_of_the_run.clone(),
            no_block_selected_here: false,
            registered_segments: registered_segments.clone(),
            registered_segment_domains: registered_segment_domains_at_recording.clone(),
            unregistered_segments_beyond_the_expansion: unregistered_segments_beyond_the_expansion
                .clone(),
            observing: None,
            observer_recording: observer_recording.clone(),
            not_run_without_the_gpu_feature: not_run_without_the_gpu_feature.clone(),
            count_vector_disagreements: Vec::new(),
            block_range: block_range.clone(),
            blocks_of_the_flow: total_blocks,
            refused_judging_units: refused_judging_units
                .iter()
                .map(JudgingUnitsKeyRefused::sentence)
                .collect(),
        });
    }
    // 设了区间变量、这台机器这一趟一块都没选中（空区间，或份与区间没有交集）：下面哨兵与强制现判、第 ① 段判器、第 ② 段核对、
    // 观察者的判段与计数向量都只对选中的块做事，到这里交回，不写待判行、不起判器。对账与读违例照这一片数（同判完那一趟）。
    // 分片进度不推进：设了区间的那一趟片由区间定（登了分片段的流），或者流里没有分片的段
    // 没设区间变量的那一趟（只按份分、这一份恰好一块没领到的也算）不走这一支，照下面同一条代码走完；
    // 只排派活计划的那一趟也照常往下走（打 PLACEMENT 行；要判的块是空的）
    if block_ranges.is_some() && my_blocks.is_empty() && !placement_only {
        let registered_segment_domains_without_judging =
            domains_and_progress_of_the_registered_segments_of_the_run(
                &library.take(),
                &flow_of_the_library,
                &plan,
                &registered_segments,
                &judged_conclusions,
                states_per_slice,
                part_of_the_run.as_ref(),
            )
            .map_err(|error| format!("读分片进度失败：{error:?}"))?;
        let unjudged = unjudged_blocks_counted_by_the_block_range(
            &library.take(),
            &plan,
            &judged_conclusions,
            &block_range,
        )?;
        let violations = violations_counted_by_the_block_range(
            &library.take(),
            &plan,
            &judged_conclusions,
            &block_range,
        )?;
        return Ok(EnvironmentRun {
            mode,
            share,
            cards_used: 0,
            recording,
            checking: CheckingTally::default(),
            unjudged_blocks: unjudged,
            violations,
            coverage_lines: report.lines().to_vec(),
            library_name,
            verification: None,
            comparison: None,
            gpu_judge: None,
            placement_only: false,
            placement_line: None,
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
            gpu_cards_taken_from: GpuCardsSource::NoCardNeeded,
            block_range_line: block_range_line_of_the_run,
            part_of_a_sliced_flow: part_of_the_run,
            no_block_selected_here: true,
            registered_segments,
            registered_segment_domains: registered_segment_domains_without_judging,
            unregistered_segments_beyond_the_expansion,
            observing: None,
            observer_recording,
            not_run_without_the_gpu_feature,
            count_vector_disagreements: Vec::new(),
            block_range,
            blocks_of_the_flow: total_blocks,
            refused_judging_units: refused_judging_units
                .iter()
                .map(JudgingUnitsKeyRefused::sentence)
                .collect(),
        });
    }
    // 核对键（L8）：只由这一趟读回的事实表摘要定，CPU 参照与 GPU 核对内核同一把（核对器的代码不进键）
    let facts_digest_read_back = read_back.facts.digest();
    let verifier_row_key_of_the_run = facts_verifier_row_key(&facts_digest_read_back);
    let seconds_planning_and_recording = planning_started.elapsed().as_secs_f64();
    // 这个进程在判段不取卡：第 ① 段判器、第 ② 段核对配了 GPU 都写待判行交 GPU 批处理服务（卡只归服务），只排派活计划那一趟只数两段要判、要核的块与状态；
    // 观察者交回的盘那一路头一张要判的盘那一刻才取（[`pool_checker_of_the_images_after_the_observers`]），来路不在这一趟的结果里报
    let gpu_cards_taken_from = GpuCardsSource::NoCardNeeded;
    // 只排派活计划（两段交 GPU 批处理服务的活）：不建判器、不试判，打一行 PLACEMENT 交回
    if placement_only {
        let placement_line = match routes.crash_judge {
            ComputedOn::Gpu => placement_line_of_the_pending_blocks(
                &library,
                &plan,
                read_back.packed_judge_tables.as_ref().ok_or_else(|| {
                    "配置开了 GPU，判段从库里读回的这一趟却没有 GPU 判表：照这份配置重跑录入段"
                        .to_string()
                })?,
                &|block| selected.contains(&block),
                &judged_conclusions,
                states_reused_at_recording,
                &verifier_row_key_of_the_run,
            )?,
            ComputedOn::Cpu => {
                return Err(
                    "只排派活计划排的是 GPU 判器的活：要配置开着 GPU（ENABLE_GPU=1）".to_string(),
                );
            }
        };
        println!("{placement_line}");
        let unjudged = unjudged_blocks_counted_by_the_block_range(
            &library.take(),
            &plan,
            &judged_conclusions,
            &block_range,
        )?;
        return Ok(EnvironmentRun {
            mode,
            share,
            cards_used: 0,
            recording,
            checking: CheckingTally::default(),
            unjudged_blocks: unjudged,
            violations: Vec::new(),
            coverage_lines: report.lines().to_vec(),
            library_name,
            verification: None,
            comparison: None,
            gpu_judge: None,
            placement_only: true,
            placement_line: Some(placement_line),
            recorded_only: false,
            stopped_early: false,
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
            block_range_line: block_range_line_of_the_run.clone(),
            part_of_a_sliced_flow: part_of_the_run.clone(),
            no_block_selected_here: false,
            registered_segments: registered_segments.clone(),
            registered_segment_domains: registered_segment_domains_at_recording.clone(),
            unregistered_segments_beyond_the_expansion: unregistered_segments_beyond_the_expansion
                .clone(),
            observing: None,
            observer_recording: observer_recording.clone(),
            not_run_without_the_gpu_feature: not_run_without_the_gpu_feature.clone(),
            count_vector_disagreements: Vec::new(),
            block_range: block_range.clone(),
            blocks_of_the_flow: total_blocks,
            refused_judging_units: refused_judging_units
                .iter()
                .map(JudgingUnitsKeyRefused::sentence)
                .collect(),
        });
    }
    // 复用哨兵（`.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」）：开跑前从键相同、库里有不红结论的块里抽几块当哨兵、
    // 再抽一块强制现判，种子打进日志；第 ① 段判完照它判（哨兵被现判、强制现判仍复用、强制现判与库里那一份不同，整趟红）。
    // 复用数在这之前数（强制现判那一块按复用算，第 ① 段判完从现判的账里扣掉它）
    let reuse_sentinel_count = reuse_sentinel_count_of_this_process();
    let observers_for_the_reuse_sentinels_of_the_run =
        observers_for_the_reuse_sentinels(&plan, observers, &observer_versions);
    let reuse_sentinels = plan_the_reuse_sentinels_from_the_reuse_read(
        &mut library.take(),
        &plan,
        &judged_conclusions,
        &reuse_read_of_the_judged_conclusions,
        &|block| selected.contains(&block),
        reuse_sentinel_count,
        reuse_sentinel_seed_of_this_process(),
        &observers_for_the_reuse_sentinels_of_the_run,
    )
    .map_err(|error| format!("抽复用哨兵读库失败：{error:?}"))?;
    if let Some(sentinels) = &reuse_sentinels {
        println!("{}", sentinels.line(flow_name, reuse_sentinel_count));
        // 强制现判那一块上观察者要读的录入段那一行不在库里：回录入段只给它重录（判段要现拼它上面的观察者结论）
        if record_the_observers_of_the_forced_block_again(
            &library,
            &flow_of_the_library,
            &plan,
            observers,
            &observer_versions,
            sentinels,
            &stop_is_requested,
        )? {
            println!(
                "CRASH_AMPLIFICATION_REUSE_SENTINEL_FORCED_BLOCK_RECORDED_AGAIN flow={flow_name}"
            );
        }
    }
    // 第 ① 段：配了 GPU 只由 GPU 判器判这台机器领的块，判的是 GPU 批处理服务：这个进程把要判的块写成待判行、等服务判完
    // （judge_my_blocks_on_gpu：库在临时 prune-server 里的先在真服务里登记那份临时库，不问服务收不收；不退回 CPU、不在这个进程里借卡判），
    // CPU 不在这一段判同一批块；没配 GPU 只由 CPU 判器判，
    // 单元两道校验和也在 CPU 上算，这一段不对拍
    let judging_started = std::time::Instant::now();
    let (mut checking, gpu_judge) = match routes.crash_judge {
        ComputedOn::Gpu => {
            let tables_read_back = read_back.packed_judge_tables.as_ref().ok_or_else(|| {
                "配置开了 GPU，判段从库里读回的这一趟却没有 GPU 判表：照这份配置重跑录入段"
                    .to_string()
            })?;
            let gpu_judging = judge_my_blocks_on_gpu(
                &library,
                &read_back.manifest.recorded_flow_digest,
                &plan,
                tables_read_back,
                &|block| selected.contains(&block),
                &judged_conclusions,
                &stop_is_requested,
            );
            let (tally, outcome) = gpu_judging
                .map_err(|reason| gpu_judging_refusal_naming_the_flow(flow_name, reason))?;
            eprintln!("{}", gpu_total_line(&outcome, expand_up_to, exhaustive));
            // 每张卡每一趟一行（整段复用、一次提交都没有的流也打一行提交次数 0 的）：每条流都打，不看流名
            #[cfg(feature = "gpu")]
            for line in gpu_pass_lines_of_the_cards(flow_name, &outcome.cards) {
                eprintln!("{line}");
            }
            (tally, Some(outcome))
        }
        ComputedOn::Cpu => {
            let checking = check_selected_blocks(
                &library,
                &flow_of_the_library,
                &plan,
                &selected,
                &judged_conclusions,
                &stop_is_requested,
            )
            .map_err(|error| format!("核对失败：{error:?}"))?;
            (checking, None)
        }
    };
    let stop_was_requested = stop_is_requested();
    if let Some(sentinels) = &reuse_sentinels {
        // 强制现判那一块按复用算：从现判的账里扣掉（复用数开跑前已数进它）。哨兵判红在观察者判段之后（观察者结论也要核）
        if sentinels.forced_block_was_judged_here(&library.take()) {
            checking.blocks_checked = checking.blocks_checked.saturating_sub(1);
            checking.states_checked = checking
                .states_checked
                .saturating_sub(sentinels.states_of_the_forced_block());
        }
    }
    // 分片的段（C790）：判完的块已经落盘，进度记录推进到连着判完的地方（被叫停提早收尾的那一趟也推进）。
    // 登了分片段的流照区间判的那一趟不推进：片起点由区间定，不是续点，从它数的不是「从编号 0 起连着判完的」
    if part_of_the_run.is_none() {
        advance_the_progress_of_the_sliced_segments(
            &mut library.take(),
            &plan,
            &judged_conclusions,
        )
        .map_err(|error| format!("推进分片进度失败：{error:?}"))?;
    }
    let registered_segment_domains_after_judging =
        domains_and_progress_of_the_registered_segments_of_the_run(
            &library.take(),
            &flow_of_the_library,
            &plan,
            &registered_segments,
            &judged_conclusions,
            states_per_slice,
            part_of_the_run.as_ref(),
        )
        .map_err(|error| format!("读分片进度失败：{error:?}"))?;
    let seconds_judging = judging_started.elapsed().as_secs_f64();
    // 第 ② 段：拿库里读回的事实表核，配置开了 GPU 就把这台机器领的块写成核对待判行、等 GPU 批处理服务核完
    // （[`verify_my_blocks_on_gpu`]：判段的库在临时服务里的先登记那份临时库，不退回 CPU 参照、不在这个进程里持卡核），
    // 没开 GPU 由 CPU 参照核；被叫停的不核
    let verifying_started = std::time::Instant::now();
    let facts_verifier_route = (!stop_was_requested).then_some(routes.facts_verifier);
    let segment_expansions_read_back = segment_expansions_of_the_plan(&flow_of_the_library, &plan);
    let verification = match facts_verifier_route {
        Some(ComputedOn::Gpu) => Some(VerificationOutcome {
            verifier: "gpu",
            tally: verify_my_blocks_on_gpu(
                &library,
                &read_back.manifest.recorded_flow_digest,
                &plan,
                &facts_digest_read_back,
                &|block| selected.contains(&block),
                &stop_is_requested,
            )?,
        }),
        None => None,
        Some(ComputedOn::Cpu) => {
            let on = FactsVerifierOn::Cpu;
            let tally = verify_blocks_from_these_facts(
                &library,
                &plan,
                &read_back.facts,
                &segment_expansions_read_back,
                &|block| selected.contains(&block),
                &on,
            )?;
            Some(VerificationOutcome {
                verifier: on.name(),
                tally,
            })
        }
    };
    if stop_was_requested || stop_is_requested() {
        // 被叫停（等服务核完的时候被叫停的也算）：判过、核过的块已经在库里，没做的段留给接着判的那一趟。
        // 哨兵与强制现判那一块可能没跑到，照「没跑到不判」只撤掉这一趟的强制现判
        if let Some(sentinels) = &reuse_sentinels {
            sentinels.stop_forcing_the_forced_block(&mut library.take());
        }
        let unjudged = unjudged_blocks_counted_by_the_block_range(
            &library.take(),
            &plan,
            &judged_conclusions,
            &block_range,
        )?;
        eprintln!(
            "CRASH_AMPLIFICATION_STOPPED blocks_checked_here={} states_checked_here={} unjudged_blocks={unjudged}",
            checking.blocks_checked, checking.states_checked
        );
        return Ok(EnvironmentRun {
            mode,
            share,
            cards_used: gpu_judge.as_ref().map_or(0, |outcome| outcome.cards_used),
            recording,
            checking,
            unjudged_blocks: unjudged,
            violations: Vec::new(),
            coverage_lines: report.lines().to_vec(),
            library_name,
            verification: None,
            comparison: None,
            gpu_judge,
            placement_only: false,
            placement_line: None,
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
            block_range_line: block_range_line_of_the_run.clone(),
            part_of_a_sliced_flow: part_of_the_run.clone(),
            no_block_selected_here: false,
            registered_segments: registered_segments.clone(),
            registered_segment_domains: registered_segment_domains_after_judging.clone(),
            unregistered_segments_beyond_the_expansion: unregistered_segments_beyond_the_expansion
                .clone(),
            observing: None,
            observer_recording: observer_recording.clone(),
            not_run_without_the_gpu_feature: not_run_without_the_gpu_feature.clone(),
            count_vector_disagreements: Vec::new(),
            block_range: block_range.clone(),
            blocks_of_the_flow: total_blocks,
            refused_judging_units: refused_judging_units
                .iter()
                .map(JudgingUnitsKeyRefused::sentence)
                .collect(),
        });
    }
    // 这个进程用了几张卡：第 ① 段判器的结局里报的（写待判行那一路是 0）；第 ② 段配了 GPU 交服务核，不在这个进程里用卡
    let cards_used = gpu_judge.as_ref().map_or(0, |outcome| outcome.cards_used);
    let seconds_verifying = verifying_started.elapsed().as_secs_f64();
    // 第 ③ 段：核过就比；比的那一遍读与计数向量、读违例并在判段收尾那一遍里（[`read_the_judged_conclusions_once_at_the_end`]），
    // 这里只定核对表那一行的键
    let verifier_digest_to_compare_against = verification
        .as_ref()
        .map(|_outcome| verifier_row_key_of_the_run);
    // 设了块区间：只比这一片选中的块（整条流的比对留给不设区间的那一趟）；没设：照旧比整份计划
    let compares_only_the_selected_blocks = block_ranges.is_some();
    let is_compared = |block: u64| !compares_only_the_selected_blocks || selected.contains(&block);
    // 挂在节点上的观察者的判段（J4「观察者改走两段、池级 checker 上 GPU」）：这台机器领的块里落在观察者节点上、
    // 库里没有这一把结论的，从库里读录入段那一块，观察者交回的盘上的池级 checker 配了 GPU 在卡上判、没配由 CPU 判，逐状态拼回一块存
    // 判器与观察者两段都判完才判哨兵（叫停的那一趟上面已经交回）：次序在两处判段共用的那一处
    let observing_started = std::time::Instant::now();
    let pool_checker = if observers.is_empty() {
        None
    } else {
        Some(pool_checker_of_the_images_after_the_observers(
            routes.crash_judge,
            card_sources_of_the_run,
        )?)
    };
    let (observing, reuse_sentinel_problems, observers_kept_out_recorded_again) =
        judge_the_observers_then_the_reuse_sentinels(
            &library,
            &flow_of_the_library,
            &plan,
            pool_checker.as_ref().map(|pool_checker_of_the_images| {
                (
                    observers,
                    observer_versions.as_slice(),
                    pool_checker_of_the_images,
                )
            }),
            &|block| selected.contains(&block),
            reuse_sentinels.as_ref(),
            &stop_is_requested,
        )?;
    if observers_kept_out_recorded_again.blocks_recorded_here > 0 {
        println!(
            "CRASH_AMPLIFICATION_OBSERVERS_KEPT_OUT_RECORDED_AGAIN flow={flow_name} blocks_recorded_here={} states_recorded_here={}",
            observers_kept_out_recorded_again.blocks_recorded_here,
            observers_kept_out_recorded_again.states_recorded_here
        );
    }
    let seconds_observing = observing_started.elapsed().as_secs_f64();
    if !reuse_sentinel_problems.is_empty() {
        return Err(format!(
            "崩溃放量的流 {flow_name} 复用哨兵判红：{}",
            reuse_sentinel_problems.join("；")
        ));
    }
    // 判段收尾一遍读判器那一份结论（判器与观察者两段都判完、哨兵判过之后；这一趟这一处的结论此后不再写）：第 ③ 段的比对、
    // 计数向量每一趟都判（J7：这台机器领的每一块，判器那一份结论读回来的计数向量与计划对得上，这一趟判的与复用的一样判）、
    // 读这一片的违例，三样选的块里每一块结论块只读一次
    let comparing_started = std::time::Instant::now();
    let JudgedConclusionsReadOnceAtTheEnd {
        comparison,
        count_vector_disagreements,
        violations,
    } = read_the_judged_conclusions_once_at_the_end(
        &library.take(),
        &flow_of_the_library,
        &plan,
        &judged_conclusions,
        verifier_digest_to_compare_against
            .as_ref()
            .map(|verifier_digest| ComparisonWithTheVerifier {
                verifier_digest,
                is_compared: &is_compared,
            }),
        &|block| selected.contains(&block),
        &block_range,
    )
    .map_err(|error| {
        format!("判段收尾读判器结论（第 ③ 段比对、计数向量、读违例）失败：{error:?}")
    })?;
    let seconds_comparing = comparing_started.elapsed().as_secs_f64();
    let reconciling_started = std::time::Instant::now();
    let unjudged = unjudged_blocks_counted_by_the_block_range(
        &library.take(),
        &plan,
        &judged_conclusions,
        &block_range,
    )?;
    let coverage_lines = report.lines().to_vec();
    // 一趟的挂钟花在哪一段：排计划与录入（CPU），第 ① 段判（GPU 判的含建表、表上卡、编内核、拆结论、写库），
    // 第 ② 段核（含抽事实表），第 ③ 段比连同计数向量与读违例（判段收尾那一遍读，CPU），对账（CPU）
    eprintln!(
        "CRASH_AMPLIFICATION_PHASES states_checked_here={} seconds_planning_and_recording={seconds_planning_and_recording:.1} seconds_judging={seconds_judging:.1} seconds_verifying={seconds_verifying:.1} seconds_comparing={seconds_comparing:.1} seconds_observing={seconds_observing:.1} seconds_reconciling={:.1}",
        checking.states_checked,
        reconciling_started.elapsed().as_secs_f64()
    );
    Ok(EnvironmentRun {
        mode,
        share,
        cards_used,
        recording,
        checking,
        unjudged_blocks: unjudged,
        violations,
        coverage_lines,
        library_name,
        verification,
        comparison,
        gpu_judge,
        placement_only: false,
        placement_line: None,
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
        block_range_line: block_range_line_of_the_run,
        part_of_a_sliced_flow: part_of_the_run,
        no_block_selected_here: false,
        registered_segments: registered_segments.clone(),
        registered_segment_domains: registered_segment_domains_after_judging,
        unregistered_segments_beyond_the_expansion: unregistered_segments_beyond_the_expansion
            .clone(),
        observing,
        observer_recording,
        not_run_without_the_gpu_feature,
        count_vector_disagreements,
        block_range,
        blocks_of_the_flow: total_blocks,
        refused_judging_units: refused_judging_units
            .iter()
            .map(JudgingUnitsKeyRefused::sentence)
            .collect(),
    })
}

#[cfg(test)]
mod tests;
