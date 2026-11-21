//! 预测与实际结局的类、单次发布的预测与比对、发布分组与计数、按分组的枚举与主臂判定。

use super::*;

// ───────────────────────── E142 第十七次跑：逐态预言器（第一节 R11、R10）与 PC1/G6/G7/PC2 ─────────────────────────
//
// 依赖集甲（R10）：这一窗口里全部单元写——根记录 → 树表与中央映射根；树表 → 各树的根，每棵树整棵往下
// （分配记录树整棵、extent 上段叶里的内联数据指针 → 数据单元）。D8（核心索引结构） 已定项 14「挂载怎么读：
// 分配记录树挂载时整棵读进挂载态」（用户 K4）与 D16（发布语义） 已定项 7「恢复重放在施加任何记录之前，
// 必须逐项验证点名单元的校验和」（这个装置的写清单把全部单元都点了名，第七节 A15）合起来，
// 使得「根槽已持久时挂载读到坏的分配树」与「根槽未持久时验证点名单元不过、不施加」两条路都要求同一个
// 依赖集（全部单元），这就是为什么单次发布的预言可以写成下面这一个分支结构，不需要分两套逻辑。

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum PredictedOutcomeClass {
    LegalFileRead,
    LegalNoFile,
    Violation,
}

impl PredictedOutcomeClass {
    pub(super) fn tag(self) -> &'static str {
        match self {
            PredictedOutcomeClass::LegalFileRead => "legal_file_read",
            PredictedOutcomeClass::LegalNoFile => "legal_no_file",
            PredictedOutcomeClass::Violation => "violation",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum ActualOutcomeClass {
    FileReadCorrect,
    FileReadWrongContent,
    NoFile,
    Failed,
}

impl ActualOutcomeClass {
    pub(super) fn of(outcome: &RecoveryOutcome, expected_content: &[u8]) -> Self {
        match outcome {
            RecoveryOutcome::FileRead { content, .. } if content == expected_content => ActualOutcomeClass::FileReadCorrect,
            RecoveryOutcome::FileRead { .. } => ActualOutcomeClass::FileReadWrongContent,
            RecoveryOutcome::NoFile { .. } => ActualOutcomeClass::NoFile,
            RecoveryOutcome::Failed { .. } => ActualOutcomeClass::Failed,
        }
    }
    pub(super) fn tag(self) -> &'static str {
        match self {
            ActualOutcomeClass::FileReadCorrect => "file_read_correct",
            ActualOutcomeClass::FileReadWrongContent => "file_read_wrong_content",
            ActualOutcomeClass::NoFile => "no_file",
            ActualOutcomeClass::Failed => "failed",
        }
    }
}

/// R20「对角」：legal_file_read → file_read_correct，legal_no_file → no_file，violation → oracle 判违例的任一实际类。
fn is_diagonal(predicted: PredictedOutcomeClass, actual: ActualOutcomeClass, actual_is_violation: bool) -> bool {
    match predicted {
        PredictedOutcomeClass::LegalFileRead => actual == ActualOutcomeClass::FileReadCorrect,
        PredictedOutcomeClass::LegalNoFile => actual == ActualOutcomeClass::NoFile,
        PredictedOutcomeClass::Violation => actual_is_violation,
    }
}

/// 逐态预言器（R11）：只看每次写的种类与这个状态的持久掩码，不调 `recover`、不读镜像。适用于「单次发布」
/// 的流（这一段之前的写全部已经持久，只有这一段自己的持久掩码在变化）：阳性对照、G6、G7、PC2，
/// 也适用于主臂那一次发布本身（`root_index`/`journal_indices` 只挂着最后一次发布，暖机两次空发布
/// 不写单元，见 `enumerate_layer0_grouped` 的说明）。
/// `dependency_groups`：依赖集甲的每一项，一项一组（组内任一份持久就算这一项在——两盘镜像时才用得到分组，
/// 一盘时每组恰一个下标）。`journal_indices`：这次发布的 journal 记录的全部镜像份数，任一份持久就算记录在
/// （主臂两盘时记录有 2 份镜像，「至少一份持久」才是 D23（journal 的角色与格式） 已定项 15 的字面；
/// 单发布流只有一份时切片长度恰 1）。
fn predict_single_publish(persisted: &[bool], root_index: usize, journal_indices: &[usize], dependency_groups: &[Vec<usize>]) -> PredictedOutcomeClass {
    let dependencies_ok = dependency_groups.iter().all(|group| group.iter().any(|&index| persisted[index]));
    let journal_available = journal_indices.iter().any(|&index| persisted[index]);
    if persisted[root_index] {
        if dependencies_ok { PredictedOutcomeClass::LegalFileRead } else { PredictedOutcomeClass::Violation }
    } else if journal_available && dependencies_ok {
        // D23（journal 的角色与格式） 已定项 15：记录持久、点名单元全在 ⇒ 施加，得到新态；这一态因此合法地读到文件，
        // 即便根槽本身没有持久（原登记「根槽未持久时新旧两态都合法」）。
        PredictedOutcomeClass::LegalFileRead
    } else {
        PredictedOutcomeClass::LegalNoFile
    }
}

/// 逐态比较的聚合（Q142.23/25/36）：checker（`evaluate_state`）对逐态预言器的判定，漏判、误判分开数；
/// 另带 3×4 结果类矩阵（Q142.24/36）。
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(super) struct PredictionComparison {
    pub(super) states: u64,
    pub(super) predicted_violations: u64,
    pub(super) agree: u64,
    /// 预言违例、checker 没判违例：对照本身坏了（F8）。
    pub(super) missed: u64,
    /// checker 判违例、预言没预言到：停机 S9。
    pub(super) false_alarm: u64,
    pub(super) first_missed: Option<String>,
    pub(super) first_false_alarm: Option<String>,
    pub(super) outcome_matrix: BTreeMap<(PredictedOutcomeClass, ActualOutcomeClass), u64>,
    pub(super) off_diagonal: u64,
}

/// 这一状态里持久的写，按角色名列出（`role_names[i]` 是 `writes[i]` 的角色；`role_names` 长度必须等于
/// `writes.len()`）；给 `first_missed` / `first_false_alarm` 用，没有持久的写时给 `"none"`。
fn persisted_role_list(persisted: &[bool], role_names: &[String]) -> String {
    let names: Vec<&str> = persisted.iter().zip(role_names).filter(|(is_persisted, _)| **is_persisted).map(|(_, name)| name.as_str()).collect();
    if names.is_empty() { "none".to_string() } else { names.join("|") }
}

/// 单次发布的写清单里每一次写的角色名：单元按 `units_by_slot` 的次序给 `descriptive_tag()`，
/// 其后依次是 journal 记录、根槽、系统配置槽（这个装置的单次发布流恰好各一次）。
pub(super) fn single_publish_role_names(writes_length: usize, units_by_slot: &[(SlotNumber, TransactionUnit, Vec<u8>)]) -> Vec<String> {
    let mut names: Vec<String> = units_by_slot.iter().map(|(_, unit, _)| unit.descriptive_tag().to_string()).collect();
    for extra in ["journal_record", "root_record", "system_configuration"] {
        if names.len() < writes_length {
            names.push(extra.to_string());
        }
    }
    while names.len() < writes_length {
        names.push("unknown".to_string());
    }
    names
}

/// 逐态跑 checker（`evaluate_state`）与逐态预言器，两边同一批状态、同一批持久掩码，谁都不知道对方判了什么。
#[allow(clippy::too_many_arguments, reason = "这些参数是这一次计算彼此独立的输入，合成只用一次的结构体只多一层包装")]
pub(super) fn compare_layer0_predictions(
    base: &Pool,
    writes: &[WriteRequest],
    segments: &[Vec<usize>],
    expected_content: &[u8],
    reader_mode: ReaderMode,
    journal_index: usize,
    dependency_groups: &[Vec<usize>],
    role_names: &[String],
) -> (Layer0Tally, PredictionComparison) {
    let root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("写流里有根槽那一条");
    let mut tally = Layer0Tally::default();
    let mut comparison = PredictionComparison::default();
    for_each_layer0_state(writes.len(), segments, |persisted| {
        // 单发布流只有一份 journal 镜像，切片长度恰 1。
        let predicted = predict_single_publish(&persisted, root_index, &[journal_index], dependency_groups);
        let predicted_violation = predicted == PredictedOutcomeClass::Violation;
        let role_list = persisted_role_list(&persisted, role_names);
        let evaluated = evaluate_state(base, writes, persisted, root_index, expected_content, reader_mode, &mut tally);
        comparison.states += 1;
        if predicted_violation {
            comparison.predicted_violations += 1;
        }
        let actual_class = ActualOutcomeClass::of(&evaluated.outcome, expected_content);
        *comparison.outcome_matrix.entry((predicted, actual_class)).or_insert(0) += 1;
        if !is_diagonal(predicted, actual_class, evaluated.is_violation) {
            comparison.off_diagonal += 1;
        }
        match (predicted_violation, evaluated.is_violation) {
            (true, true) | (false, false) => comparison.agree += 1,
            (true, false) => {
                comparison.missed += 1;
                if comparison.first_missed.is_none() {
                    comparison.first_missed = Some(role_list);
                }
            }
            (false, true) => {
                comparison.false_alarm += 1;
                if comparison.first_false_alarm.is_none() {
                    comparison.first_false_alarm = Some(role_list);
                }
            }
        }
    });
    (tally, comparison)
}

// ───────────────────────── E142 第十七次跑第二段：层 0 主臂的单独调用（R13，第五节 5.4，第六节 6.2） ─────────────────────────
//
// 「按发布分组」：主臂那条流里有三次根槽 FUA 写（两次暖机空发布各一次、主事务一次），`for_each_layer0_state`
// 按段挨段枚举，一个状态落在哪个分组只看它正在变化的那个段号相对三个根所在段号的位置——不需要重新走一遍
// mkfs/暖机/发布的语义。「结果类表」复用 Q142.24 已经有的 `PredictedOutcomeClass`/`ActualOutcomeClass`/`is_diagonal`，
// 预言器复用 `predict_single_publish`（R10 的依赖集判定本来就是「组内任一份持久」，两盘镜像只是把
// `journal_indices` 从一份扩成两份，见该函数上方的改动说明）。

/// 按全局下标（`0..closed_form_state_count(segments)`）直接给出这个下标对应的持久掩码，与它所在的段号
/// （`None` = `for_each_layer0_state` 最后单独访问的那一次「全部持久」）。次序与 `for_each_layer0_state`
/// 完全相同：段挨段、每段内 mask 从 0 到 `full_mask − 1`，最后单独访问一次全持久
/// （`layer0_state_at_index_matches_for_each_layer0_state_on_the_positive_control` 逐态核过两者一致）。
/// 并行切片只是把 `[0, closed_form)` 切成几段区间分给不同线程，每个下标对应哪个状态不因此改变。
pub(super) fn layer0_state_at_index(writes_length: usize, segments: &[Vec<usize>], mut global_index: u64) -> (Vec<bool>, Option<usize>) {
    let mut persisted = vec![false; writes_length];
    for (segment_index, segment) in segments.iter().enumerate() {
        let full_mask = (1u64 << segment.len()) - 1;
        if global_index < full_mask {
            for (bit, &write_index) in segment.iter().enumerate() {
                if global_index & (1 << bit) != 0 {
                    persisted[write_index] = true;
                }
            }
            return (persisted, Some(segment_index));
        }
        global_index -= full_mask;
        for &write_index in segment {
            persisted[write_index] = true;
        }
    }
    debug_assert_eq!(global_index, 0, "下标超出 closed_form_state_count 的范围");
    (persisted, None)
}

/// 每一次根槽 FUA 写所在的段号，按写序（因此也是 txg 序）排列（R13「按发布分组」）。单发布流只有一次根槽，
/// 返回长度 1 的向量。
fn publish_group_boundaries(writes: &[WriteRequest], segments: &[Vec<usize>]) -> Vec<usize> {
    writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(write_index, _)| segments.iter().position(|segment| segment.contains(&write_index)).expect("根槽写必在某个段里"))
        .collect()
}

/// 段号 → 发布分组标签：段号落在第 k 个根自己的段或更早（`index <= boundaries[k]`）⇒ `txg{k+1}`；
/// 晚于最后一个根 ⇒ `after_last_root`；`None`（`for_each_layer0_state` 最后那次「全部持久」访问）⇒
/// `every_write_persisted`（第七节 A20 的五组分）。
fn publish_group_label(boundaries: &[usize], segment_index: Option<usize>) -> String {
    match segment_index {
        None => "every_write_persisted".to_string(),
        Some(index) => match boundaries.iter().position(|&boundary| index <= boundary) {
            Some(order) => format!("txg{}", order + 1),
            None => "after_last_root".to_string(),
        },
    }
}

/// 一个发布分组里的三个量：Q142.31（states）、Q142.32（violations）、Q142.35（journal_differing_states，
/// 只有 txg1/txg2/txg3 三组会被消费——journal 差异只可能发生在判的根还没持久时）。
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(super) struct PublishBucket {
    pub(super) states: u64,
    pub(super) violations: u64,
    pub(super) journal_differing_states: u64,
}

impl PublishBucket {
    pub(super) fn record(&mut self, is_violation: bool, journal_differs: bool) {
        self.states += 1;
        if is_violation {
            self.violations += 1;
        }
        if journal_differs {
            self.journal_differing_states += 1;
        }
    }
    fn merge(&mut self, other: &Self) {
        self.states += other.states;
        self.violations += other.violations;
        self.journal_differing_states += other.journal_differing_states;
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(super) struct PublishGrouping {
    buckets: BTreeMap<String, PublishBucket>,
}

impl PublishGrouping {
    pub(super) fn record(&mut self, label: &str, is_violation: bool, journal_differs: bool) {
        self.buckets.entry(label.to_string()).or_default().record(is_violation, journal_differs);
    }
    fn merge(&mut self, other: &Self) {
        for (label, bucket) in &other.buckets {
            self.buckets.entry(label.clone()).or_default().merge(bucket);
        }
    }
    pub(super) fn get(&self, label: &str) -> PublishBucket {
        self.buckets.get(label).cloned().unwrap_or_default()
    }
}

/// 逗号连接的 `label:count`（Q142.31 `states_by_publish=`、Q142.35 `differing_by_publish=`：同一行里的一个字段）。
pub(super) fn format_publish_counts(grouping: &PublishGrouping, labels: &[&str], select: impl Fn(&PublishBucket) -> u64) -> String {
    labels.iter().map(|&label| format!("{label}:{}", select(&grouping.get(label)))).collect::<Vec<_>>().join(",")
}

/// 空格连接的 `label=count`（Q142.32「另一行 `name=layer0_violations_by_publish txg1= txg2= …`」）。
pub(super) fn format_publish_fields(grouping: &PublishGrouping, labels: &[&str], select: impl Fn(&PublishBucket) -> u64) -> String {
    labels.iter().map(|&label| format!("{label}={}", select(&grouping.get(label)))).collect::<Vec<_>>().join(" ")
}

/// 一个诊断用的角色名列表：`w{下标}_{种类}`，只给 `PredictionComparison::first_missed`/`first_false_alarm`
/// 当人读的文本用，不是 Q142.27 的角色名自检（那一套只认默认调用里 `name=role_labels` 定义的 t 号）。
pub(super) fn generic_role_names(writes: &[WriteRequest]) -> Vec<String> {
    writes.iter().enumerate().map(|(index, write)| format!("w{index}_{}", write.kind.tag())).collect()
}

/// R10 依赖集甲在主事务这次发布上的写法：`writes` 里全部 `UnitWrite`（暖机两次空发布不写单元，D16（发布语义）
/// 已定项 8「且不写单元」，所以整条主臂窗口里唯一会出现 `UnitWrite` 的就是主事务），按
/// `write_unit_to_every_device` 的写序（单元外层、设备内层，见该函数的调用点）逐 `device_count` 个一组，
/// 组内任一份持久就算这个逻辑单元在——与 `build_single_publish_flow` 给单发布流建的 `dependency_groups`
/// 是同一个判定（`predict_single_publish` 的 `dependencies_ok`），只是这里组的大小是 `device_count` 而不是 1。
pub(super) fn main_transaction_dependency_groups(writes: &[WriteRequest], device_count: usize) -> Vec<Vec<usize>> {
    let unit_indices: Vec<usize> = writes.iter().enumerate().filter(|(_, write)| write.kind == StepKind::UnitWrite).map(|(index, _)| index).collect();
    unit_indices.chunks(device_count.max(1)).map(<[usize]>::to_vec).collect()
}

#[derive(Default)]
struct GroupedAccumulator {
    tally: Layer0Tally,
    comparison: PredictionComparison,
    grouping: PublishGrouping,
}

impl GroupedAccumulator {
    /// 分片按起点从小到大依次 `merge`：`first_violation`/`first_missed`/`first_false_alarm` 只在自己还没有
    /// 时才收 `other` 的，因此串行（一整个分片 `0..closed_form`）与并行（多个分片按起点顺序折叠）给出
    /// 完全相同的这三个诊断字段，其余都是纯计数的加法——这就是
    /// `layer0_grouped_enumeration_agrees_between_serial_and_parallel_*` 要证的「逐格相等」。
    fn merge(&mut self, other: GroupedAccumulator) {
        Layer0Tally::merge_into(&mut self.tally, other.tally);

        let merged_comparison = &mut self.comparison;
        merged_comparison.states += other.comparison.states;
        merged_comparison.predicted_violations += other.comparison.predicted_violations;
        merged_comparison.agree += other.comparison.agree;
        merged_comparison.missed += other.comparison.missed;
        merged_comparison.false_alarm += other.comparison.false_alarm;
        if merged_comparison.first_missed.is_none() {
            merged_comparison.first_missed = other.comparison.first_missed;
        }
        if merged_comparison.first_false_alarm.is_none() {
            merged_comparison.first_false_alarm = other.comparison.first_false_alarm;
        }
        for (key, count) in other.comparison.outcome_matrix {
            *merged_comparison.outcome_matrix.entry(key).or_insert(0) += count;
        }
        merged_comparison.off_diagonal += other.comparison.off_diagonal;

        self.grouping.merge(&other.grouping);
    }
}

/// R13：层 0 主臂的单独调用——按发布分组（`PublishGrouping`）与结果类表（复用 Q142.24 那一套）接进枚举。
/// `threads = 1` 时单线程跑 `[0, closed_form)`；`threads > 1` 时把这个区间切成 `threads` 段，各自在独立线程
/// 里跑（`layer0_state_at_index` 保证每个下标对应哪个状态与串行完全一样），最后按分片起点顺序 `merge`。
/// 这个函数本身不知道调用方是单发布流还是多发布的主臂——`阳性对照`/`G6`/`G7` 用它时 `root_index`/
/// `journal_indices`/`dependency_groups` 就是它们自己单次发布的那几个，`publish_group_boundaries` 会得到
/// 长度 1 的分组列表（只有 `txg1` 与 `after_last_root`/`every_write_persisted`），这正是
/// `layer0_grouped_enumeration_agrees_between_serial_and_parallel_*` 用来证明并行机制本身没问题的取样点。
#[allow(clippy::too_many_arguments, reason = "分组枚举的入参逐项对应登记里的输入（日志下标、依赖分组、发布分界与各开关），拆结构体不会少传哪一项")]
pub(super) fn enumerate_layer0_grouped(
    threads: usize,
    base: &Pool,
    writes: &[WriteRequest],
    segments: &[Vec<usize>],
    expected_content: &[u8],
    reader_mode: ReaderMode,
    root_index: usize,
    journal_indices: &[usize],
    dependency_groups: &[Vec<usize>],
    role_names: &[String],
) -> (Layer0Tally, PredictionComparison, PublishGrouping) {
    assert_segments_are_enumerable(segments);
    let boundaries = publish_group_boundaries(writes, segments);
    let closed_form = closed_form_state_count(segments);
    let threads = threads.max(1);

    let run_range = |range: std::ops::Range<u64>| -> GroupedAccumulator {
        let mut accumulator = GroupedAccumulator::default();
        for global_index in range {
            let (persisted, segment_index) = layer0_state_at_index(writes.len(), segments, global_index);
            let predicted = predict_single_publish(&persisted, root_index, journal_indices, dependency_groups);
            let predicted_violation = predicted == PredictedOutcomeClass::Violation;
            let role_list = persisted_role_list(&persisted, role_names);
            let evaluated = evaluate_state(base, writes, persisted, root_index, expected_content, reader_mode, &mut accumulator.tally);
            accumulator.comparison.states += 1;
            if predicted_violation {
                accumulator.comparison.predicted_violations += 1;
            }
            let actual_class = ActualOutcomeClass::of(&evaluated.outcome, expected_content);
            *accumulator.comparison.outcome_matrix.entry((predicted, actual_class)).or_insert(0) += 1;
            if !is_diagonal(predicted, actual_class, evaluated.is_violation) {
                accumulator.comparison.off_diagonal += 1;
            }
            match (predicted_violation, evaluated.is_violation) {
                (true, true) | (false, false) => accumulator.comparison.agree += 1,
                (true, false) => {
                    accumulator.comparison.missed += 1;
                    if accumulator.comparison.first_missed.is_none() {
                        accumulator.comparison.first_missed = Some(role_list.clone());
                    }
                }
                (false, true) => {
                    accumulator.comparison.false_alarm += 1;
                    if accumulator.comparison.first_false_alarm.is_none() {
                        accumulator.comparison.first_false_alarm = Some(role_list.clone());
                    }
                }
            }
            let label = publish_group_label(&boundaries, segment_index);
            accumulator.grouping.record(&label, evaluated.is_violation, evaluated.journal_differs);
        }
        accumulator
    };

    if threads == 1 || closed_form <= 1 {
        let accumulator = run_range(0..closed_form);
        return (accumulator.tally, accumulator.comparison, accumulator.grouping);
    }

    let ranges = split_index_ranges(closed_form, threads);

    let results: Vec<GroupedAccumulator> = std::thread::scope(|scope| {
        let handles: Vec<_> = ranges.into_iter().map(|range| scope.spawn(|| run_range(range))).collect();
        handles.into_iter().map(|handle| handle.join().expect("层0分组并行工作线程 panic")).collect()
    });

    let mut merged = GroupedAccumulator::default();
    for result in results {
        merged.merge(result);
    }
    (merged.tally, merged.comparison, merged.grouping)
}

/// Q142.37 的判决字段（第六节 6.2 表最后一行）：从已经算出来的 `Layer0Tally`/`PredictionComparison`
/// 现导出，不在 `main()` 里内嵌判定逻辑——内嵌的判定 `cargo test` 下走不到（`enumerate_layer0_grouped`
/// 只在真的调了 `main()` 时才跑），第十二节修订第 8 条已经因为这个原因补过
/// 一批测试，这里直接把判定拆成纯函数，`layer0_main_arm_verdict_flags_every_threshold_independently`
/// 单测覆盖每一格。
pub(super) struct Layer0MainArmVerdict {
    pub(super) layer0_states_ok: bool,
    pub(super) layer0_violations: &'static str,
    pub(super) root_persisted_ok: bool,
    pub(super) file_read_ok: bool,
    pub(super) file_read_wrong_content_zero: bool,
    pub(super) journal_reading: &'static str,
    pub(super) main_outcome_matrix_ok: bool,
}

/// 判决行 `write_list_ok`（E142 第十九次跑臂 N19 的 ⑤ 与 N19C 的 ⑨，登记 7.4 D5）：新池新建文件一次发布写 29 次、
/// FUA 1 次，三道池屏障（D16（发布语义） 已定项 7「单元 → 屏障 → 记录 → 屏障 → 根槽 → 系统配置槽 → 屏障」）
/// 按设备每块盘各记一步。
pub(super) fn new_pool_file_creation_write_list_matches(write_count: usize, barrier_step_count: usize, fua_count: usize, device_count: usize) -> bool {
    write_count == 29 && barrier_step_count == 3 * device_count && fua_count == 1
}

pub(super) fn layer0_main_arm_verdict(tally: &Layer0Tally, closed_form: u64, comparison: &PredictionComparison) -> Layer0MainArmVerdict {
    Layer0MainArmVerdict {
        // ⑪（E142 第十九次跑登记 R19C-4）：发布尾那道屏障把暖机第二次的系统配置槽写与新池新建文件的单元写隔开，
        // 整条流 `2+2+1+2+2+1+2+24+2+1+2`，闭式照登记 7.4 C5 / C12。
        layer0_states_ok: tally.states == closed_form && closed_form == 16_777_240,
        layer0_violations: if tally.violations == 0 { "zero" } else { "nonzero" },
        root_persisted_ok: tally.root_persisted_states == 4,
        file_read_ok: tally.file_read_states == 7,
        file_read_wrong_content_zero: tally.file_read_wrong_content_states == 0,
        journal_reading: match tally.journal_differing_states {
            3 => "j_jia",
            9 => "j_yi",
            _ => "neither",
        },
        main_outcome_matrix_ok: comparison.off_diagonal == 0,
    }
}

#[cfg(test)]
#[path = "outcome_predictions_and_grouped_enumeration_tests.rs"]
mod tests;
