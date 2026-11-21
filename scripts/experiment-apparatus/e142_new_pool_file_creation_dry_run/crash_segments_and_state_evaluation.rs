//! 崩溃段的闭合规则、按段与按步种类的计数、就地覆盖、三态汇总、栅栏被吞的点、被录制的写流与按状态的判定。

use super::*;

// ───────────────────────── 层 0 崩溃点枚举（D13 已定项 4）与 oracle（E77 判据 1） ─────────────────────────

/// 关段规则（E142 第十九次跑臂 N19 的 ②；D13（验证路线） 已定项 4 条文「当前段里每块有写的盘都被自己的屏障或
/// FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段」「带 FUA 的写只放行它自己那块盘」）。
/// `split_into_segments` 与 `segment_step_kinds` 共用这一条，两处不各写一份。
/// 写（含整段清零）把它那块盘记成没放行；屏障放行它那块盘；`fua_is_boundary` 时 FUA 写算一个写、并放行它自己那块盘，
/// 不开时 FUA 写与普通写一样（只给 `layer0_fua_not_boundary` 与 G6 用）。段里有写且没放行的盘为空才关段。
struct SegmentClosingRule {
    fua_is_boundary: bool,
    unreleased_devices: std::collections::BTreeSet<DeviceIdentity>,
    segment_has_write: bool,
}

impl SegmentClosingRule {
    pub(super) fn new(fua_is_boundary: bool) -> Self {
        Self { fua_is_boundary, unreleased_devices: std::collections::BTreeSet::new(), segment_has_write: false }
    }
    /// 吃下一步，返回这一步之后当前段是不是关上了。
    fn closes_after(&mut self, operation: &RecordedOperation) -> bool {
        match operation {
            RecordedOperation::Write(write) => {
                self.segment_has_write = true;
                self.unreleased_devices.insert(write.device);
                if self.fua_is_boundary && write.is_fua() {
                    self.unreleased_devices.remove(&write.device);
                }
            }
            RecordedOperation::Barrier(device) => {
                self.unreleased_devices.remove(device);
            }
        }
        let closes = self.segment_has_write && self.unreleased_devices.is_empty();
        if closes {
            self.segment_has_write = false;
        }
        closes
    }
}

/// 按 `SegmentClosingRule` 把写流切成段，交回全部写与每段里写的下标。流尾没放行的写自成最后一段。
pub(super) fn split_into_segments(operations: &[RecordedOperation], fua_is_boundary: bool) -> (Vec<WriteRequest>, Vec<Vec<usize>>) {
    let mut writes = Vec::new();
    let mut segments: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut rule = SegmentClosingRule::new(fua_is_boundary);
    for operation in operations {
        if let RecordedOperation::Write(write) = operation {
            writes.push(write.clone());
            current.push(writes.len() - 1);
        }
        if rule.closes_after(operation) {
            segments.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    (writes, segments)
}

/// 每段的步骤种类，按与 `split_into_segments` 相同的切法分组——这是 D17（实现分层与第三方管道） 已定项 2
/// 说的「每段步骤种类集合」的机器可读输入（C316 ②）。关掉一段的那道屏障算进它关掉的那一段；
/// 段里还一个写都没有时（流首那道屏障）算进即将开始的那一段；流尾那一串只有屏障没有写的，并进上一段——
/// 这样录到的每一步都恰好出现在一个段里，段数也与 `split_into_segments` 一样。
/// 末尾的断言把两者钉在一起：逐段数出来的写数必须相等，对不上就是切法分了叉。
pub(super) fn segment_step_kinds(operations: &[RecordedOperation], fua_is_boundary: bool) -> Vec<Vec<RecordedStepKind>> {
    let mut groups: Vec<Vec<RecordedStepKind>> = Vec::new();
    let mut current: Vec<RecordedStepKind> = Vec::new();
    let mut writes_in_current: usize = 0;
    let mut rule = SegmentClosingRule::new(fua_is_boundary);
    for operation in operations {
        current.push(RecordedStepKind::of(operation));
        if let RecordedOperation::Write(_) = operation {
            writes_in_current += 1;
        }
        if rule.closes_after(operation) {
            groups.push(std::mem::take(&mut current));
            writes_in_current = 0;
        }
    }
    if !current.is_empty() {
        match (writes_in_current, groups.last_mut()) {
            (0, Some(last_group)) => last_group.append(&mut current),
            (_, _) => groups.push(current),
        }
    }
    let (_, segments) = split_into_segments(operations, fua_is_boundary);
    let writes_per_group: Vec<usize> = groups
        .iter()
        .map(|group| group.iter().filter(|kind| **kind != RecordedStepKind::Barrier).count())
        .collect();
    assert_eq!(writes_per_group, segments.iter().map(Vec::len).collect::<Vec<usize>>(), "步骤种类的切法与 split_into_segments 对不上");
    groups
}

/// 一组步骤种类写成规范多重集 `[种类×次数,…]`，按 `canonical_rank_in_kinds_text` 排，只出现一次的不带次数。
fn format_kind_multiset(group: &[RecordedStepKind]) -> String {
    let mut counts: BTreeMap<(u8, RecordedStepKind), usize> = BTreeMap::new();
    for kind in group {
        *counts.entry((kind.canonical_rank_in_kinds_text(), *kind)).or_insert(0) += 1;
    }
    let entries: Vec<String> = counts
        .into_iter()
        .map(|((_, kind), count)| if count == 1 { kind.tag().to_string() } else { format!("{}×{count}", kind.tag()) })
        .collect();
    format!("[{}]", entries.join(","))
}

/// 把每段的步骤种类写成一行可解析的文字：段之间用 `|`，段内排成规范多重集（`format_kind_multiset`）。
/// 规范序是为了让两段只要多重集相同、文字就一模一样。
pub(super) fn format_segment_kinds(groups: &[Vec<RecordedStepKind>]) -> String {
    groups.iter().map(|group| format_kind_multiset(group)).collect::<Vec<String>>().join("|")
}

/// E77 的闭式：1 + Σ(2^|段| − 1)。
pub(super) fn closed_form_state_count(segments: &[Vec<usize>]) -> u64 {
    1 + segments.iter().map(|segment| (1u64 << segment.len()) - 1).sum::<u64>()
}

/// 层 0 的扇区（D13（验证路线） 已定项 4 射程「层 0 的扇区取 512 字节」）：原地覆写要「长于一个扇区」。
const LAYER0_SECTOR_BYTES: usize = 512;

/// 臂 N19 的 ③：这次写算不算原地覆写（D13 已定项 4「不是单元写、长于一个扇区、罩住的范围里原来有东西的写」，
/// 登记 R39 的读法：`image` 是录制流里它之前的全部写都落了的那一版，这次写罩住的字节里至少一个非零）。
fn is_in_place_overwrite(image: &BTreeMap<DeviceIdentity, SparseDevice>, write: &WriteRequest) -> bool {
    let is_unit_write = match write.kind {
        StepKind::UnitWrite => true,
        StepKind::JournalRecord | StepKind::RootRecordFua | StepKind::SystemConfigurationSlot | StepKind::ZeroFill => false,
    };
    let is_longer_than_a_sector = write.bytes.len() > LAYER0_SECTOR_BYTES;
    if is_unit_write || !is_longer_than_a_sector {
        return false;
    }
    image.get(&write.device).is_some_and(|device| device.read(write.offset, write.bytes.len()).iter().any(|byte| *byte != 0))
}

/// 把一步落进镜像。整段清零在录制器里不带字节（`RecordingPool::record_zero_fill` 的说明），落了也不改镜像：
/// 这条流上每次整段清零都在那一段第一次被写之前（mkfs 开头），与真写一遍 0 没有分别。
fn land_operation_in_image(image: &mut BTreeMap<DeviceIdentity, SparseDevice>, operation: &RecordedOperation) {
    match operation {
        RecordedOperation::Write(write) => image.entry(write.device).or_default().write(write.offset, &write.bytes),
        RecordedOperation::Barrier(_) => {}
    }
}

/// 一条路径里每一次写是不是原地覆写（次序与 `split_into_segments` 交回的写一一对应）。
/// `earlier_operations` 是这条路径之前录下的全部步（从 mkfs 第一步起），先全落进镜像；路径里的写逐次先判、再落。
fn in_place_overwrite_flags(earlier_operations: &[RecordedOperation], path_operations: &[RecordedOperation]) -> Vec<bool> {
    let mut image: BTreeMap<DeviceIdentity, SparseDevice> = BTreeMap::new();
    for operation in earlier_operations {
        land_operation_in_image(&mut image, operation);
    }
    let mut flags = Vec::new();
    for operation in path_operations {
        if let RecordedOperation::Write(write) = operation {
            flags.push(is_in_place_overwrite(&image, write));
        }
        land_operation_in_image(&mut image, operation);
    }
    flags
}

/// 每段的原地覆写数 m。
fn in_place_overwrites_per_segment(segments: &[Vec<usize>], in_place_flags: &[bool]) -> Vec<usize> {
    segments.iter().map(|segment| segment.iter().filter(|index| in_place_flags[**index]).count()).collect()
}

/// 臂 N19 的 ③ 的闭式：1 + Σ(3^m · 2^(n−m) − 1)，n 是段的写数、m 是段里原地覆写的写数（登记 R32；
/// 一段各写取几态的全部组合是 3^m · 2^(n−m)，减去「这一段全都没持久」那一个，它与上一段全部持久是同一个状态）。
fn closed_form_three_state_count(segments: &[Vec<usize>], in_place_flags: &[bool]) -> u64 {
    let per_segment = in_place_overwrites_per_segment(segments, in_place_flags);
    1 + segments
        .iter()
        .zip(per_segment)
        .map(|(segment, in_place_count)| {
            let written = segment.len();
            let three_state_ways = 3u64.checked_pow(u32::try_from(in_place_count).expect("段内写数")).expect("三态组合数装得进 u64");
            let two_state_ways = 1u64 << (written - in_place_count);
            three_state_ways.checked_mul(two_state_ways).expect("组合数装得进 u64") - 1
        })
        .sum::<u64>()
}

/// 一条路径的三态那一行要报的数（`name=segments_three_state` 与 `name=segments_sensitivity` 共用）。
pub(super) struct ThreeStateSummary {
    pub(super) writes: usize,
    pub(super) in_place_overwrites: usize,
    pub(super) in_place_per_segment: String,
    pub(super) in_place_kinds: String,
    pub(super) closed_form_two_state: u64,
    pub(super) closed_form_three_state: u64,
}

pub(super) fn three_state_summary(earlier_operations: &[RecordedOperation], path_operations: &[RecordedOperation]) -> ThreeStateSummary {
    let (writes, segments) = split_into_segments(path_operations, true);
    let in_place_flags = in_place_overwrite_flags(earlier_operations, path_operations);
    assert_eq!(in_place_flags.len(), writes.len(), "原地覆写的标记与写一一对应");
    let in_place_kinds: Vec<RecordedStepKind> = writes.iter().zip(&in_place_flags).filter(|(_, is_in_place)| **is_in_place).map(|(write, _)| RecordedStepKind::Write(write.kind)).collect();
    ThreeStateSummary {
        writes: writes.len(),
        in_place_overwrites: in_place_flags.iter().filter(|is_in_place| **is_in_place).count(),
        in_place_per_segment: in_place_overwrites_per_segment(&segments, &in_place_flags).iter().map(ToString::to_string).collect::<Vec<_>>().join("+"),
        in_place_kinds: if in_place_kinds.is_empty() { "none".to_string() } else { format_kind_multiset(&in_place_kinds) },
        closed_form_two_state: closed_form_state_count(&segments),
        closed_form_three_state: closed_form_three_state_count(&segments, &in_place_flags),
    }
}

/// ⑥ 的两个取样点（⑧ 开的主几何）：点名与吞掉哪块盘那一步（登记第八节 8.2：G10a 吞盘 1，不是根所在的盘；G10b 吞盘 0）。
pub(super) const SWALLOWED_BARRIER_POINTS_WITH_TRAILING_BARRIER: [(&str, DeviceIdentity); 2] = [("G10a", DeviceIdentity(1)), ("G10b", DeviceIdentity(0))];
/// ⑩ 的两个取样点（⑧ 关，G11 的流上照 G10 各吞一步，登记 R19C-4 ⑩）。
pub(super) const SWALLOWED_BARRIER_POINTS_WITHOUT_TRAILING_BARRIER: [(&str, DeviceIdentity); 2] = [("G12a", DeviceIdentity(1)), ("G12b", DeviceIdentity(0))];

/// 臂 N19 的 ⑥（取样点 G10a / G10b，登记第八节 8.2）：在新池新建文件「最后一次 journal 记录写 → 根槽 FUA 写」
/// 之间那一串屏障里，删掉 `swallowed_device` 那一步，其余不动。那一串里没有这块盘的一步就是装置错。
pub(super) fn swallow_one_device_barrier_step_before_the_root(transaction_operations: &[RecordedOperation], swallowed_device: DeviceIdentity) -> Vec<RecordedOperation> {
    let last_journal_record = transaction_operations
        .iter()
        .rposition(|operation| matches!(operation, RecordedOperation::Write(write) if write.kind == StepKind::JournalRecord))
        .expect("新池新建文件里有 journal 记录写");
    let root_record = transaction_operations
        .iter()
        .position(|operation| matches!(operation, RecordedOperation::Write(write) if write.kind == StepKind::RootRecordFua))
        .expect("新池新建文件里有根槽写");
    let swallowed = (last_journal_record + 1..root_record)
        .find(|index| matches!(&transaction_operations[*index], RecordedOperation::Barrier(device) if *device == swallowed_device))
        .expect("记录与根槽之间有这块盘的那一步屏障");
    transaction_operations.iter().enumerate().filter(|(index, _)| *index != swallowed).map(|(_, operation)| operation.clone()).collect()
}

/// `name=segments_sensitivity` 一行（⑥ 的 G10a / G10b，⑩ 的 G11 / G12a / G12b）：字段次序照登记 R19C-4 ⑥ ⑩。
/// 取样点 G1B（一块盘、带屏障）上 mkfs 那一段的段序列行：切法（`split_into_segments(…, true)`）与字段同
/// `name=segments path=mkfs` 那一行，只换几何。`crates/` 只造两块盘，这一行只有装置一侧（E142 第二十五次跑登记第八节）。
pub(super) fn segments_geometry_line_of_mkfs_on_one_device_with_barriers() -> String {
    let (one_device_recording, _) = mkfs(&PoolParameters::one_device_with_barriers());
    let operations = &one_device_recording.operations;
    let (_, segments) = split_into_segments(operations, true);
    let sizes: Vec<String> = segments.iter().map(|segment| segment.len().to_string()).collect();
    format!(
        "name=segments_geometry point=G1B path=mkfs operations={} segments={} closed_form={} kinds={}",
        operations.len(),
        sizes.join("+"),
        closed_form_state_count(&segments),
        format_segment_kinds(&segment_step_kinds(operations, true))
    )
}

pub(super) fn segments_sensitivity_line(point_fields: &str, path_name: &str, earlier_operations: &[RecordedOperation], path_operations: &[RecordedOperation]) -> String {
    let (_, segments) = split_into_segments(path_operations, true);
    let summary = three_state_summary(earlier_operations, path_operations);
    format!(
        "name=segments_sensitivity {point_fields} path={path_name} operations={} segments={} closed_form={} closed_form_three_state={} in_place_per_segment={} kinds={}",
        path_operations.len(),
        segments.iter().map(|segment| segment.len().to_string()).collect::<Vec<_>>().join("+"),
        summary.closed_form_two_state,
        summary.closed_form_three_state,
        summary.in_place_per_segment,
        format_segment_kinds(&segment_step_kinds(path_operations, true)),
    )
}

/// 一条 mkfs → 取号 → 暖机两次 → 第一个文件版本的录制流与各路径的切点（取样点 G11 现录用；主几何那一条在 `main()` 里录）。
pub(super) struct RecordedNewPoolFileCreationStream {
    pub(super) operations: Vec<RecordedOperation>,
    pub(super) mkfs_operation_count: usize,
    pub(super) acquisition_operation_count: usize,
    pub(super) warm_up_operation_count: usize,
}

pub(super) fn record_new_pool_file_creation_stream(parameters: &PoolParameters, file_bytes: &[u8]) -> RecordedNewPoolFileCreationStream {
    let (mut recording, genesis) = mkfs(parameters);
    let mkfs_operation_count = recording.operations.len();
    let instance = acquire_instance(&mut recording, parameters);
    let acquisition_operation_count = recording.operations.len();
    let (_, last_warm_up_record) = warm_up(&mut recording, parameters, &genesis, instance);
    let warm_up_operation_count = recording.operations.len();
    publish_first_file(&mut recording, parameters, &genesis, file_bytes, instance, last_warm_up_record.as_deref(), NEW_POOL_FILE_CREATION_TXG);
    RecordedNewPoolFileCreationStream { operations: recording.operations, mkfs_operation_count, acquisition_operation_count, warm_up_operation_count }
}

/// 五条路径的切法（登记 R31，与 `main()` 里 `name=segments ` 那五行同一取法），每条带「这条路径之前录下的全部步」
/// （三态认原地覆写要从 mkfs 第一步起的镜像，R39）。
pub(super) fn new_pool_file_creation_stream_paths(
    operations: &[RecordedOperation],
    mkfs_operation_count: usize,
    acquisition_operation_count: usize,
    warm_up_operation_count: usize,
) -> [(&'static str, &[RecordedOperation], &[RecordedOperation]); 5] {
    [
        ("mkfs", &operations[..0], &operations[..mkfs_operation_count]),
        ("instance_acquisition", &operations[..mkfs_operation_count], &operations[mkfs_operation_count..acquisition_operation_count]),
        ("warm_up", &operations[..acquisition_operation_count], &operations[acquisition_operation_count..warm_up_operation_count]),
        ("transaction", &operations[..warm_up_operation_count], &operations[warm_up_operation_count..]),
        ("post_mkfs_stream", &operations[..mkfs_operation_count], &operations[mkfs_operation_count..]),
    ]
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(super) struct Layer0Tally {
    pub(super) states: u64,
    pub(super) violations: u64,
    pub(super) root_persisted_states: u64,
    pub(super) no_file_states: u64,
    pub(super) file_read_states: u64,
    /// `file_read_states` 的子集：读回的内容与期望不同（`oracle_violation` 的 `FileRead` 分支恒判这一种违例，
    /// Q142.34「file_read_wrong_content=」）。
    pub(super) file_read_wrong_content_states: u64,
    pub(super) failed_states: u64,
    pub(super) journal_differing_states: u64,
    pub(super) verification_ran_states: u64,
    pub(super) verification_failed_states: u64,
    pub(super) first_violation: Option<String>,
    /// S5（第八节）：读者乙在这一轮全部状态里走到 ①–④ 各分支的次数总和；`reader_mode` 不是 `Clause` 时恒 0。
    pub(super) reader_branches: ReaderClauseBranchCounts,
}

impl Layer0Tally {
    /// 把一片的计数并进总数：计数相加，`first_violation` 只在总数还没有时才收这一片的。分片按下标起点从小到大依次并入，
    /// 因此多片并行与一整片串行给出同一处「第一处违例」（`enumerate_layer0_grouped` 与 `enumerate_crash_states_with_threads` 共用）。
    pub(super) fn merge_into(merged_tally: &mut Layer0Tally, other_tally: Layer0Tally) {
        merged_tally.states += other_tally.states;
        merged_tally.violations += other_tally.violations;
        merged_tally.root_persisted_states += other_tally.root_persisted_states;
        merged_tally.no_file_states += other_tally.no_file_states;
        merged_tally.file_read_states += other_tally.file_read_states;
        merged_tally.file_read_wrong_content_states += other_tally.file_read_wrong_content_states;
        merged_tally.failed_states += other_tally.failed_states;
        merged_tally.journal_differing_states += other_tally.journal_differing_states;
        merged_tally.verification_ran_states += other_tally.verification_ran_states;
        merged_tally.verification_failed_states += other_tally.verification_failed_states;
        if merged_tally.first_violation.is_none() {
            merged_tally.first_violation = other_tally.first_violation;
        }
        merged_tally.reader_branches.add(&other_tally.reader_branches);
    }
}

pub(super) fn oracle_violation(outcome: &RecoveryOutcome, root_persisted: bool, expected_content: &[u8]) -> Option<String> {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => (content != expected_content).then(|| "读回的内容不对".to_string()),
        RecoveryOutcome::NoFile { .. } => root_persisted.then(|| "根槽已持久而恢复到旧态".to_string()),
        RecoveryOutcome::Failed { reason, .. } => Some(format!("走读失败：{reason}")),
    }
}

/// 一个状态上的判定，供 `enumerate_layer0`（只要聚合数）与逐态比较（Q142.23/25/36，还要每一态自己的判定）共用。
pub(super) struct EvaluatedState {
    pub(super) outcome: RecoveryOutcome,
    pub(super) is_violation: bool,
    /// E142 第十七次跑第二段（R17）：查 journal 与不查 journal 两遍恢复的 `RecoveryOutcome` 是否不同——
    /// 按发布分组时（`PublishGrouping`）逐组另报一次；`Layer0Tally.journal_differing_states` 是它的总和，
    /// 单独状态这一位不重复算一遍，只是把已经算出的布尔值带出来给调用方分组用。
    pub(super) journal_differs: bool,
}

pub(super) fn evaluate_state(base: &Pool, writes: &[WriteRequest], persisted: Vec<bool>, root_index: usize, expected_content: &[u8], reader_mode: ReaderMode, tally: &mut Layer0Tally) -> EvaluatedState {
    let root_persisted = persisted[root_index];
    let image = CrashImage { base, writes, persisted };
    let consulted = recover(&image, JournalPolicy::Consult, reader_mode);
    let ignored = recover(&image, JournalPolicy::Ignore, reader_mode);
    tally.reader_branches.add(&consulted.clause_branches);
    tally.states += 1;
    if root_persisted {
        tally.root_persisted_states += 1;
    }
    let journal_differs = consulted.outcome != ignored.outcome;
    if journal_differs {
        tally.journal_differing_states += 1;
    }
    if consulted.journal.verification_passed + consulted.journal.verification_failed > 0 {
        tally.verification_ran_states += 1;
    }
    if consulted.journal.verification_failed > 0 {
        tally.verification_failed_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { content, .. } => {
            tally.file_read_states += 1;
            if content.as_slice() != expected_content {
                tally.file_read_wrong_content_states += 1;
            }
        }
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    let violation_reason = oracle_violation(&consulted.outcome, root_persisted, expected_content);
    let is_violation = violation_reason.is_some();
    if let Some(reason) = violation_reason {
        tally.violations += 1;
        if tally.first_violation.is_none() {
            let persisted_kinds: Vec<&str> = image.persisted.iter().zip(writes).filter(|(is_persisted, _)| **is_persisted).map(|(_, write)| write.kind.tag()).collect();
            tally.first_violation = Some(format!("{reason}（持久的写：{}）", persisted_kinds.join("|")));
        }
    }
    EvaluatedState { outcome: consulted.outcome, is_violation, journal_differs }
}

/// 枚举一段要访问 2^写数 个状态，写数就是枚举的代价。主臂事务的 24 个单元写（12 个单元 × 两块盘）同在一段
/// （段序列 `2+2+1+2+2+1+2+24+2+1+2`，登记 7.4 C5），是这个装置要枚举的最大一段；别的几何的枚举段都比它短。
pub(super) const LARGEST_ENUMERATED_SEGMENT_WRITES: usize = 24;

/// 枚举之前先核：有一段比 `LARGEST_ENUMERATED_SEGMENT_WRITES` 长，就是切段规则坏了或写流变了，照枚举下去不会收尾；
/// 当场断言失败，不截断枚举（截断的状态数会被当成真结果）。
pub(super) fn assert_segments_are_enumerable(segments: &[Vec<usize>]) {
    for (segment_number, segment) in segments.iter().enumerate() {
        assert!(
            segment.len() <= LARGEST_ENUMERATED_SEGMENT_WRITES,
            "段号 {segment_number} 有 {} 个写，超过枚举上限 {LARGEST_ENUMERATED_SEGMENT_WRITES}：切段规则或写流不对，枚举 2^{} 个状态跑不完",
            segment.len(),
            segment.len()
        );
    }
}

/// D13（验证路线） 已定项 4 的枚举域，从 `enumerate_layer0` 里拆出来给逐态比较（`compare_layer0_predictions`）
/// 共用：前若干段全持久、当前段任一非满子集持久、之后各段都不持久，最后单独访问一次「全部持久」。
/// 只改了「访问一个状态」的方式（从直接调 `evaluate_state` 改成回调），枚举次序与被访问的持久掩码集合不变。
pub(super) fn for_each_layer0_state(writes_length: usize, segments: &[Vec<usize>], mut visit: impl FnMut(Vec<bool>)) {
    assert_segments_are_enumerable(segments);
    let mut persisted_before = vec![false; writes_length];
    for segment in segments {
        let full_mask = (1u64 << segment.len()) - 1;
        for mask in 0..full_mask {
            let mut persisted = persisted_before.clone();
            for (bit, write_index) in segment.iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
            visit(persisted);
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    visit(persisted_before);
}

/// 把全局下标区间 `[0, closed_form)` 切成至多 `threads` 段连续、不相交、按起点升序的区间（每段 `ceil(closed_form / threads)` 个下标，
/// 最后一段取余下的，空段丢掉）。`enumerate_layer0_grouped` 与 `enumerate_crash_states_with_threads` 共用这一个切法。
pub(super) fn split_index_ranges(closed_form: u64, threads: usize) -> Vec<std::ops::Range<u64>> {
    let chunk = closed_form.div_ceil(threads.max(1) as u64);
    (0..threads.max(1) as u64)
        .map(|worker| (worker * chunk).min(closed_form)..((worker + 1) * chunk).min(closed_form))
        .filter(|range| !range.is_empty())
        .collect()
}

/// 整轮枚举，线程数取 `parallel_thread_count()`。
#[cfg(test)]
pub(super) fn enumerate_layer0(base: &Pool, writes: &[WriteRequest], segments: &[Vec<usize>], expected_content: &[u8], reader_mode: ReaderMode) -> Layer0Tally {
    enumerate_crash_states_with_threads(parallel_thread_count(), base, writes, segments, expected_content, reader_mode)
}

/// 整轮枚举：经 `layer0_state_at_index` 把 `[0, closed_form)` 按下标切成至多 `threads` 段，各段在自己的线程里各建各的
/// `Layer0Tally`，按段起点从小到大并入（`Layer0Tally::merge_into`）。每个下标对应哪个状态与 `for_each_layer0_state`
/// 逐态相同，所以计数与「第一处违例」与串行（`threads = 1`）逐格相同。单段跑得久时每五分钟报一行这一段的区间与已判的数。
#[cfg(test)]
pub(super) fn enumerate_crash_states_with_threads(threads: usize, base: &Pool, writes: &[WriteRequest], segments: &[Vec<usize>], expected_content: &[u8], reader_mode: ReaderMode) -> Layer0Tally {
    assert_segments_are_enumerable(segments);
    // 被判的是这条流里最后一次根槽 FUA 写：暖机的两个根在它前面，主臂与阳性对照都取这一条。
    let root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("写流里有根槽那一条");
    let run_range = |range: std::ops::Range<u64>| -> Layer0Tally {
        const PROGRESS_INTERVAL: std::time::Duration = std::time::Duration::from_secs(300);
        const STATES_BETWEEN_CLOCK_READS: usize = 1 << 16;
        let mut tally = Layer0Tally::default();
        let mut last_report = std::time::Instant::now();
        for (judged, global_index) in range.clone().enumerate() {
            if judged.is_multiple_of(STATES_BETWEEN_CLOCK_READS) && last_report.elapsed() >= PROGRESS_INTERVAL {
                eprintln!("崩溃状态枚举进度：区间 [{}, {}) 已判 {judged} 个状态", range.start, range.end);
                last_report = std::time::Instant::now();
            }
            let (persisted, _) = layer0_state_at_index(writes.len(), segments, global_index);
            evaluate_state(base, writes, persisted, root_index, expected_content, reader_mode, &mut tally);
        }
        tally
    };
    let ranges = split_index_ranges(closed_form_state_count(segments), threads);
    let slice_tallies: Vec<Layer0Tally> = std::thread::scope(|scope| {
        let handles: Vec<_> = ranges.into_iter().map(|range| scope.spawn(|| run_range(range))).collect();
        handles.into_iter().map(|handle| handle.join().expect("整轮枚举的工作线程 panic")).collect()
    });
    let mut merged = Layer0Tally::default();
    for slice_tally in slice_tallies {
        Layer0Tally::merge_into(&mut merged, slice_tally);
    }
    merged
}

#[cfg(test)]
#[path = "crash_segments_and_state_evaluation_tests.rs"]
mod tests;
