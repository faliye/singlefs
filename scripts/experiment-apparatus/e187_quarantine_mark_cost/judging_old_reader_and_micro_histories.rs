//! 记录的类别与记号状态、核对器判定、旧读者的算术、事件轨迹，以及微型历史的准备与量。

use super::*;

/// 位翻转分类表的记录类（登记第六节 `flip_classification_table`）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum RecordClass {
    QuarantineIntact,
    QuarantineGarbled,
    Leak,
    Normal,
}
pub(super) const RECORD_CLASSES: [RecordClass; 4] = [RecordClass::QuarantineIntact, RecordClass::QuarantineGarbled, RecordClass::Leak, RecordClass::Normal];

impl RecordClass {
    pub(super) fn label(self) -> &'static str {
        match self {
            RecordClass::QuarantineIntact => "quarantine_intact",
            RecordClass::QuarantineGarbled => "quarantine_garbled",
            RecordClass::Leak => "leak",
            RecordClass::Normal => "normal",
        }
    }

    pub(super) fn copies(self) -> CopyCondition {
        match self {
            RecordClass::QuarantineGarbled => CopyCondition::Garbled,
            RecordClass::QuarantineIntact | RecordClass::Leak | RecordClass::Normal => CopyCondition::Intact,
        }
    }

    fn is_quarantine(self) -> bool {
        match self {
            RecordClass::QuarantineIntact | RecordClass::QuarantineGarbled => true,
            RecordClass::Leak | RecordClass::Normal => false,
        }
    }
}

/// 记号写对、两盘一起写错、只一盘写错（登记第六节「另报只翻一盘」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum MarkState {
    WrittenCorrectly,
    WrongOnBothDevices,
    WrongOnOneDevice,
}
pub(super) const MARK_STATES: [MarkState; 3] = [MarkState::WrittenCorrectly, MarkState::WrongOnBothDevices, MarkState::WrongOnOneDevice];

impl MarkState {
    pub(super) fn label(self) -> &'static str {
        match self {
            MarkState::WrittenCorrectly => "written_correctly",
            MarkState::WrongOnBothDevices => "wrong_on_both_devices",
            MarkState::WrongOnOneDevice => "wrong_on_one_device",
        }
    }
}

/// 判哪一版：隔离那一版；I-3.1 候选窗口里最后一个引用它的根离开窗口之后那一版。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum JudgedVersion {
    TheQuarantineVersion,
    AfterTheLastReferencingRootLeavesTheWindow,
}
pub(super) const JUDGED_VERSIONS: [JudgedVersion; 2] = [JudgedVersion::TheQuarantineVersion, JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow];

impl JudgedVersion {
    pub(super) fn label(self) -> &'static str {
        match self {
            JudgedVersion::TheQuarantineVersion => "quarantine_version",
            JudgedVersion::AfterTheLastReferencingRootLeavesTheWindow => "after_window",
        }
    }
}

/// checker 判一条（两块盘上的那一对）记录时看得到的东西。`marked_on_device` 在甲是跨度段 0x4000 位，在乙是隔离树里那一条条目，
/// 在 `no_mark` 是同一位 0x4000（今天的格式里就是一个越界的跨度）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct RecordView {
    pub(super) is_released: bool,
    marked_on_device: [bool; 2],
    referenced_by_the_newest_root: bool,
    referenced_by_some_candidate_root: bool,
    unmapped_while_unreleased_in_some_candidate_version: bool,
    pub(super) copies: CopyCondition,
}

/// 一张盘面上哪几条判据判红。
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(super) struct CheckerVerdict {
    allocated_statistic_red: bool,
    allocated_minus_deferred_red: bool,
    unreleased_record_off_the_mapping_red: bool,
    pub(super) mark_agreement_red: bool,
    record_outside_its_leaf_red: bool,
}

impl CheckerVerdict {
    pub(super) fn is_red(&self) -> bool {
        self.allocated_statistic_red
            || self.allocated_minus_deferred_red
            || self.unreleased_record_off_the_mapping_red
            || self.mark_agreement_red
            || self.record_outside_its_leaf_red
    }

    pub(super) fn isolation_criteria_red_count(&self) -> u64 {
        u64::from(self.allocated_statistic_red) + u64::from(self.allocated_minus_deferred_red) + u64::from(self.unreleased_record_off_the_mapping_red)
    }

    /// 产物里的那一格：判绿，或判红的判据名单。
    pub(super) fn label(&self) -> String {
        let mut names = Vec::new();
        if self.allocated_statistic_red {
            names.push("I-3.1");
        }
        if self.allocated_minus_deferred_red {
            names.push("I-3.11");
        }
        if self.unreleased_record_off_the_mapping_red {
            names.push("I-3.15");
        }
        if self.mark_agreement_red {
            names.push("mark_agrees_with_references");
        }
        if self.record_outside_its_leaf_red {
            names.push("I-1.1");
        }
        if names.is_empty() {
            "green".to_string()
        } else {
            names.join("+")
        }
    }
}

/// 今天的副本判据：有一份读盘核出对不上（读不出也算）才豁免（`walk.rs` 的 `some_copy_of_the_unit_fails_its_location_entries`）。
fn some_copy_fails_its_location_entries(copies: CopyCondition) -> bool {
    match copies {
        CopyCondition::Intact => false,
        CopyCondition::Garbled => true,
    }
}

/// 记号与引用相符（登记第五节「丙的 checker」三句）。
fn mark_agrees_with_references(view: &RecordView) -> bool {
    let marked_anywhere = view.marked_on_device.iter().any(|marked| *marked);
    let marked_everywhere = view.marked_on_device.iter().all(|marked| *marked);
    !(marked_anywhere && view.is_released) && !(marked_anywhere && view.referenced_by_the_newest_root) && (marked_everywhere || !marked_anywhere)
}

/// 一条臂的 checker 判一对记录（登记第五节「臂」最后一栏与「丙的 checker」）。
pub(super) fn judge_record(checker: CheckerUnderTest, view: &RecordView) -> CheckerVerdict {
    let marked_anywhere = view.marked_on_device.iter().any(|marked| *marked);
    let marked_everywhere = view.marked_on_device.iter().all(|marked| *marked);
    // 今天的读者不认 0x4000：跨度读成 16385 / 16386，落不进任何一片叶（`walk.rs` 的 I-1.1 落叶判），隔离判据那一遍不再看它。
    let today_reader_rejects_the_span = checker == CheckerUnderTest::Today && marked_anywhere;
    // 没有记号时怎么办：今天的副本判据；丙的两个变体一个只认记号、一个退回今天的副本判据。
    let exempt_without_a_mark = match checker {
        CheckerUnderTest::Today => some_copy_fails_its_location_entries(view.copies),
        CheckerUnderTest::MarkOnly => false,
        CheckerUnderTest::MarkOrToday => some_copy_fails_its_location_entries(view.copies),
    };
    // 记号齐（每块盘上那一份都带）的豁免、不看副本；只一盘带的不算齐、也不退回副本判据。
    let exempt = match checker {
        CheckerUnderTest::Today => exempt_without_a_mark,
        CheckerUnderTest::MarkOnly | CheckerUnderTest::MarkOrToday => marked_everywhere || (!marked_anywhere && exempt_without_a_mark),
    };
    let violates_mark_agreement = !mark_agrees_with_references(view);
    let mark_agreement_red = match checker {
        CheckerUnderTest::Today => false,
        CheckerUnderTest::MarkOnly | CheckerUnderTest::MarkOrToday => violates_mark_agreement,
    };
    let isolation_criteria_judged = !view.is_released && !today_reader_rejects_the_span;
    CheckerVerdict {
        allocated_statistic_red: isolation_criteria_judged && !view.referenced_by_some_candidate_root && !exempt,
        allocated_minus_deferred_red: isolation_criteria_judged && !view.referenced_by_the_newest_root && !exempt,
        unreleased_record_off_the_mapping_red: isolation_criteria_judged && view.unmapped_while_unreleased_in_some_candidate_version && !exempt,
        mark_agreement_red,
        record_outside_its_leaf_red: today_reader_rejects_the_span,
    }
}

/// 记号这一格在这条臂上落在哪两块盘上（写对的样子按类定：隔离的带记号、泄漏与正常的不带；`no_mark` 写对就是都不带）。
fn marks_of(arm: Arm, class: RecordClass, mark_state: MarkState) -> [bool; 2] {
    let correct = match arm {
        Arm::SpanBit | Arm::QuarantineTree => class.is_quarantine(),
        Arm::NoMark | Arm::PlainRelease => false,
    };
    match mark_state {
        MarkState::WrittenCorrectly => [correct, correct],
        MarkState::WrongOnBothDevices => match arm {
            Arm::SpanBit | Arm::QuarantineTree => [!correct, !correct],
            Arm::NoMark | Arm::PlainRelease => [true, true],
        },
        MarkState::WrongOnOneDevice => match arm {
            Arm::SpanBit | Arm::QuarantineTree => [!correct, correct],
            Arm::NoMark | Arm::PlainRelease => [true, false],
        },
    }
}

/// 造出来的一张盘面（登记第六节「造出来的盘面」）：这一类记录在这一版上 checker 看得到的样子。
pub(super) fn constructed_view(arm: Arm, class: RecordClass, mark_state: MarkState, version: JudgedVersion) -> RecordView {
    let is_normal = class == RecordClass::Normal;
    RecordView {
        is_released: false,
        marked_on_device: marks_of(arm, class, mark_state),
        referenced_by_the_newest_root: is_normal,
        referenced_by_some_candidate_root: is_normal || version == JudgedVersion::TheQuarantineVersion,
        unmapped_while_unreleased_in_some_candidate_version: !is_normal,
        copies: class.copies(),
    }
}

/// 地真：隔离的两类与正常记录在记号写对时是合法盘；泄漏与任何写错的记号是缺陷（登记第一节「地真」）。
pub(super) fn is_a_legal_disk(class: RecordClass, mark_state: MarkState) -> bool {
    class != RecordClass::Leak && mark_state == MarkState::WrittenCorrectly
}

/// 放过（缺陷判绿）与误红（合法判红）的条数，按两个宇宙分开：不翻记号（写对）与翻记号（写错，两盘与一盘）。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(super) struct PassAndFalseRedCounts {
    pub(super) passed_defects: u64,
    pub(super) false_reds: u64,
    pub(super) judged_disks: u64,
}

impl PassAndFalseRedCounts {
    pub(super) fn count(&mut self, legal: bool, verdict: &CheckerVerdict, disks: u64) {
        self.judged_disks += disks;
        if legal && verdict.is_red() {
            self.false_reds += disks;
        }
        if !legal && !verdict.is_red() {
            self.passed_defects += disks;
        }
    }
}

/// 两个数怎么记（登记第六节 `pass_and_false_red_counts` 的判定）。
pub(super) fn pass_and_false_red_reading(marking: PassAndFalseRedCounts, no_mark: PassAndFalseRedCounts) -> &'static str {
    if marking.false_reds < no_mark.false_reds && marking.passed_defects <= no_mark.passed_defects {
        "mark_dominates"
    } else if marking.false_reds < no_mark.false_reds {
        "false_reds_traded_for_passes"
    } else {
        "mark_does_not_solve_this_shape"
    }
}

pub(super) fn universe_of(mark_state: MarkState) -> usize {
    match mark_state {
        MarkState::WrittenCorrectly => 0,
        MarkState::WrongOnBothDevices | MarkState::WrongOnOneDevice => 1,
    }
}
pub(super) const UNIVERSE_LABELS: [&str; 2] = ["marks_not_flipped", "marks_flipped"];

/// 旧读者的跨度读法：`span_field & !0x8000`（`allocator.rs` 的 `AllocationRecord::parse`、`walk.rs` 的 `parse_allocation_record`；
/// GPU 判器 `span_field & 0x7FFF` 同值）。
pub(super) fn old_reader_span(span_field: u16) -> u64 {
    u64::from(span_field & !ALLOCATION_RECORD_RELEASED_FLAG)
}

/// 旧读者的落叶判：跨度不为 0、末槽不越过叶末槽（`allocation_record_tree.rs` 的 `record_fits_in_its_leaf`、
/// `position_addressed.rs` 的 `allocation_record_fits_in_the_leaf`、GPU 判器同一句）。`start_in_leaf` 从叶起点数。
pub(super) fn old_reader_accepts(start_in_leaf: u64, span: u64) -> bool {
    span > 0 && start_in_leaf + span - 1 < ALLOCATION_RECORD_TREE_LEAF_SLOTS
}

/// 借一位之后，一片叶里每个合法起点上的记录被旧读者收下几条（登记第六节 `old_reader_field_table` 的装置算术）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct OldReaderArithmetic {
    pub(super) borrowed_bit: u16,
    pub(super) record_span: u16,
    pub(super) legal_starts: u64,
    pub(super) span_read: u64,
    pub(super) accepted: u64,
}

pub(super) fn old_reader_arithmetic(borrowed_bit: u16, record_span: u16) -> OldReaderArithmetic {
    let legal_starts: Vec<u64> =
        (0..ALLOCATION_RECORD_TREE_LEAF_SLOTS).filter(|start| start % u64::from(record_span) == 0 && start + u64::from(record_span) <= ALLOCATION_RECORD_TREE_LEAF_SLOTS).collect();
    let span_read = old_reader_span(record_span | borrowed_bit);
    let accepted = u64_of(legal_starts.iter().filter(|start| old_reader_accepts(**start, span_read)).count());
    OldReaderArithmetic { borrowed_bit, record_span, legal_starts: u64_of(legal_starts.len()), span_read, accepted }
}

/// 一次隔离事件在一段历史里的样子（轨迹与终态分类读它）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct QuarantineEventRecord {
    pub(super) txg: u64,
    pub(super) copies: CopyCondition,
    pub(super) marks_during_quarantine: [bool; 2],
    pub(super) revival: Option<RevivalRecord>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct RevivalRecord {
    pub(super) txg: u64,
    pub(super) marks_after_revival: [bool; 2],
}

/// 一个事件从隔离那一版起到历史末尾，I-3.1、I-3.11、I-3.15 各红不红之和（0 到 3）的轨迹（登记第八节「轨迹」）。
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(super) struct EventTrajectory {
    pub(super) peak: u64,
    pub(super) positive_versions: u64,
    pub(super) final_value: u64,
    pub(super) first_red_offset_of_the_allocated_statistic: Option<u64>,
    pub(super) mark_agreement_red_versions: u64,
}

pub(super) fn trajectory_of_an_event(checker: CheckerUnderTest, event: &QuarantineEventRecord, final_txg: u64, candidate_window: u64) -> EventTrajectory {
    let mut trajectory = EventTrajectory::default();
    for version in event.txg..=final_txg {
        let revived = event.revival.is_some_and(|revival| version >= revival.txg);
        let marks_now = match event.revival {
            Some(revival) if revived => revival.marks_after_revival,
            Some(_) | None => event.marks_during_quarantine,
        };
        let referenced_by_some_candidate_root = event.txg > oldest_candidate_txg(version, candidate_window) || revived;
        let newest_view = RecordView {
            is_released: false,
            marked_on_device: marks_now,
            referenced_by_the_newest_root: revived,
            referenced_by_some_candidate_root,
            unmapped_while_unreleased_in_some_candidate_version: false,
            copies: event.copies,
        };
        let newest = judge_record(checker, &newest_view);
        // I-3.15 每一版各判：窗口里还有隔离那几版（隔离到复活之前）时，那几版里这条记录未释放而映射里没有它。
        let window_start = oldest_candidate_txg(version, candidate_window).max(event.txg);
        let quarantine_end = event.revival.map_or(version, |revival| (revival.txg - 1).min(version));
        let quarantine_view = RecordView {
            is_released: false,
            marked_on_device: event.marks_during_quarantine,
            referenced_by_the_newest_root: false,
            referenced_by_some_candidate_root: true,
            unmapped_while_unreleased_in_some_candidate_version: window_start <= quarantine_end,
            copies: event.copies,
        };
        let off_the_mapping = judge_record(checker, &quarantine_view).unreleased_record_off_the_mapping_red;
        let value = u64::from(newest.allocated_statistic_red) + u64::from(newest.allocated_minus_deferred_red) + u64::from(off_the_mapping);
        if newest.allocated_statistic_red && trajectory.first_red_offset_of_the_allocated_statistic.is_none() {
            trajectory.first_red_offset_of_the_allocated_statistic = Some(version - event.txg);
        }
        trajectory.peak = trajectory.peak.max(value);
        trajectory.positive_versions += u64::from(value > 0);
        trajectory.final_value = value;
        trajectory.mark_agreement_red_versions += u64::from(newest.mark_agreement_red);
    }
    trajectory
}

/// 八段微历史（登记第五节「微历史」，手推钉值在第七节）。都在 `geometry_default` 上、映射树只有根兼叶。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum MicroHistory {
    CleanLeaf,
    CleanAncestor,
    DirtyLeaf,
    JournalCrossing,
    SecondQuarantineTreeCold,
    RevivalClean,
    RevivalTreeCold,
    ReleaseQuarantined,
}
pub(super) const MICRO_HISTORIES: [MicroHistory; 8] = [
    MicroHistory::CleanLeaf,
    MicroHistory::CleanAncestor,
    MicroHistory::DirtyLeaf,
    MicroHistory::JournalCrossing,
    MicroHistory::SecondQuarantineTreeCold,
    MicroHistory::RevivalClean,
    MicroHistory::RevivalTreeCold,
    MicroHistory::ReleaseQuarantined,
];

impl MicroHistory {
    pub(super) fn label(self) -> &'static str {
        match self {
            MicroHistory::CleanLeaf => "micro_clean_leaf",
            MicroHistory::CleanAncestor => "micro_clean_ancestor",
            MicroHistory::DirtyLeaf => "micro_dirty_leaf",
            MicroHistory::JournalCrossing => "micro_journal_crossing",
            MicroHistory::SecondQuarantineTreeCold => "micro_second_quarantine_tree_cold",
            MicroHistory::RevivalClean => "micro_revival_clean",
            MicroHistory::RevivalTreeCold => "micro_revival_tree_cold",
            MicroHistory::ReleaseQuarantined => "micro_release_quarantined",
        }
    }

    fn victim_slot(self) -> u64 {
        match self {
            MicroHistory::CleanAncestor => MICRO_CLEAN_ANCESTOR_VICTIM_SLOT,
            MicroHistory::DirtyLeaf => MICRO_DIRTY_LEAF_VICTIM_SLOT,
            MicroHistory::CleanLeaf
            | MicroHistory::JournalCrossing
            | MicroHistory::SecondQuarantineTreeCold
            | MicroHistory::RevivalClean
            | MicroHistory::RevivalTreeCold
            | MicroHistory::ReleaseQuarantined => MICRO_CLEAN_LEAF_VICTIM_SLOT,
        }
    }

    /// 这次发布（不含臂差）最后一个事务的点名数。
    fn named_roles_of_the_last_transaction(self) -> u64 {
        match self {
            MicroHistory::JournalCrossing => MICRO_CROSSING_NAMED_ROLES,
            MicroHistory::CleanLeaf
            | MicroHistory::CleanAncestor
            | MicroHistory::DirtyLeaf
            | MicroHistory::SecondQuarantineTreeCold
            | MicroHistory::RevivalClean
            | MicroHistory::RevivalTreeCold
            | MicroHistory::ReleaseQuarantined => MICRO_BASE_NAMED_ROLES,
        }
    }

    /// 第七节钉的「臂减 `no_mark`」：(甲的设备写, 甲的字节, 乙的设备写, 乙的字节)。
    pub(super) fn pinned_extra_writes(self) -> (u64, u64, u64, u64) {
        match self {
            MicroHistory::CleanLeaf => (4, 65536, 2, 32768),
            MicroHistory::CleanAncestor => (8, 131072, 2, 32768),
            MicroHistory::DirtyLeaf => (0, 0, 2, 32768),
            MicroHistory::JournalCrossing => (6, 73728, 2, 32768),
            MicroHistory::SecondQuarantineTreeCold => (4, 65536, 6, 98304),
            MicroHistory::RevivalClean => (4, 65536, 0, 0),
            MicroHistory::RevivalTreeCold => (4, 65536, 4, 65536),
            MicroHistory::ReleaseQuarantined => (0, 0, 0, 0),
        }
    }
}

/// 微历史的文件：8 个数据单元；被隔离的是第 7 个（落点由构造给定），同一次覆盖写的另三个是第 0–2 个；
/// 第二次隔离那一段先在第 3–6 个那一次隔离第 6 个；释放隔离记录那一次覆盖写第 3–6 个。
const MICRO_DATA_UNITS: u32 = 8;
pub(super) const MICRO_VICTIM: u32 = 7;
const MICRO_EARLIER_VICTIM: u32 = 6;
const MICRO_OVERWRITES_WITH_THE_VICTIM: [u32; 4] = [0, 1, 2, MICRO_VICTIM];
const MICRO_OTHER_OVERWRITES: [u32; 4] = [3, 4, 5, MICRO_EARLIER_VICTIM];
const MICRO_BASE_NAMED_ROLES: u64 = 20;
const MICRO_CROSSING_NAMED_ROLES: u64 = 63;
/// 叶 75（罩 [60900, 61712)）；槽 140000 在叶 172、层级 1 第二个节点罩 [137228, 274456)；叶 62 里的 50500；叶 70 里的 57000。
const MICRO_CLEAN_LEAF_VICTIM_SLOT: u64 = 60900;
const MICRO_CLEAN_ANCESTOR_VICTIM_SLOT: u64 = 140000;
const MICRO_DIRTY_LEAF_VICTIM_SLOT: u64 = 50500;
const MICRO_COLD_QUARANTINE_TREE_NODE_SLOT: u64 = 57000;
/// 垫在 mkfs 那两条之后的填充记录（不动的两槽记录）：别的微历史垫到 50400，这次发布的落点都在叶 62 里头；
/// `micro_dirty_leaf` 垫到 50750（只空出被隔离单元那一对），这次发布的落点都在 [50750, 51156)：W 减半成 406 时那是另一片叶。
const MICRO_FILLER_START: u64 = UNIT_AREA_START_SLOT + 2;
const MICRO_FILLER_END: u64 = 50400;
const MICRO_DIRTY_LEAF_FILLER_END: u64 = 50750;
pub(super) const MICRO_CANDIDATE_WINDOW: u64 = 8;

/// 搭好一段微历史，交回被量的那一次发布之前的池与那一次发布的请求。
pub(super) fn prepare_micro_history(micro: MicroHistory, arm: Arm) -> (PoolState, PublishRequest) {
    let mut state = PoolState::after_make_filesystem(PoolGeometry::DefaultFourGibibyteDevices, MICRO_CANDIDATE_WINDOW, true);
    let filler_end = if micro == MicroHistory::DirtyLeaf { MICRO_DIRTY_LEAF_FILLER_END } else { MICRO_FILLER_END };
    for slot in (MICRO_FILLER_START..filler_end).step_by(2) {
        if slot != micro.victim_slot() {
            state.occupy_with_a_record_that_never_changes(slot, 2);
        }
    }
    let create = PublishRequest {
        kind: PublishKind::CreateFile { data_unit_count: MICRO_DATA_UNITS },
        forced: vec![ForcedPlacement::DataUnit { index: MICRO_VICTIM, slot: micro.victim_slot() }],
    };
    let created_at = state.txg + 1;
    publish(&mut state, &create, arm);
    if micro == MicroHistory::SecondQuarantineTreeCold {
        let earlier = PublishRequest {
            kind: PublishKind::Overwrite { data_indices: MICRO_OTHER_OVERWRITES.to_vec(), quarantine_victim: Some(MICRO_EARLIER_VICTIM), release_quarantined: None },
            forced: vec![ForcedPlacement::NewQuarantineTreeNode { slot: MICRO_COLD_QUARANTINE_TREE_NODE_SLOT }],
        };
        publish(&mut state, &earlier, arm);
    }
    let quarantine_the_victim = PublishKind::Overwrite { data_indices: MICRO_OVERWRITES_WITH_THE_VICTIM.to_vec(), quarantine_victim: Some(MICRO_VICTIM), release_quarantined: None };
    let measured = match micro {
        MicroHistory::CleanLeaf | MicroHistory::CleanAncestor | MicroHistory::DirtyLeaf | MicroHistory::JournalCrossing | MicroHistory::SecondQuarantineTreeCold => {
            PublishRequest::plain(quarantine_the_victim)
        }
        MicroHistory::RevivalClean | MicroHistory::RevivalTreeCold => {
            let forced = if micro == MicroHistory::RevivalTreeCold {
                vec![ForcedPlacement::NewQuarantineTreeNode { slot: MICRO_COLD_QUARANTINE_TREE_NODE_SLOT }]
            } else {
                Vec::new()
            };
            publish(&mut state, &PublishRequest { kind: quarantine_the_victim, forced }, arm);
            PublishRequest::plain(PublishKind::RollBack { target_txg: created_at })
        }
        MicroHistory::ReleaseQuarantined => {
            publish(&mut state, &PublishRequest::plain(quarantine_the_victim), arm);
            let victim_unit = *state.quarantined.keys().next().expect("上一次隔离了第 7 个数据单元");
            PublishRequest::plain(PublishKind::Overwrite { data_indices: MICRO_OTHER_OVERWRITES.to_vec(), quarantine_victim: None, release_quarantined: Some(victim_unit) })
        }
    };
    (state, measured)
}

/// 臂减 `no_mark` 的写（有正负：分叉之后的发布里两条臂的落点、节点年龄不同）。
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(super) struct WriteDifference {
    pub(super) device_writes: i64,
    pub(super) bytes_on_both_devices: i64,
}

pub(super) fn signed(value: u64) -> i64 {
    i64::try_from(value).expect("写量装得进 i64")
}

pub(super) fn difference(arm: WriteTally, baseline: WriteTally) -> WriteDifference {
    WriteDifference {
        device_writes: signed(arm.device_writes) - signed(baseline.device_writes),
        bytes_on_both_devices: signed(arm.bytes_on_both_devices) - signed(baseline.bytes_on_both_devices),
    }
}

/// 分结构的差（分配记录树节点、映射树节点、隔离树节点、journal 记录），产物里写成「次/字节」。
pub(super) fn structure_differences(arm: &PublishWrites, baseline: &PublishWrites) -> String {
    let mut pairs: Vec<(&str, WriteDifference)> = [WrittenStructure::AllocationRecordTreeNode, WrittenStructure::CentralMappingNode, WrittenStructure::QuarantineTreeNode]
        .iter()
        .map(|structure| (structure.label(), difference(arm.of(*structure), baseline.of(*structure))))
        .collect();
    pairs.push(("journal", difference(arm.journal, baseline.journal)));
    pairs.iter().map(|(name, value)| format!("extra_{name}={}/{}", value.device_writes, value.bytes_on_both_devices)).collect::<Vec<String>>().join(" ")
}

/// 一段微历史在一条臂上被量的那一次发布。
pub(super) struct MicroArmMeasurement {
    pub(super) arm: Arm,
    pub(super) writes: PublishWrites,
    pub(super) padding_named_roles: u64,
    pub(super) victim_leaf_rewritten: bool,
    /// 回退那两段：回退之后那一版上，复活的单元那一对记录被这条臂的 checker 判 `mark_agrees_with_references` 红不红（各变体取或）。
    pub(super) mark_agreement_red_after_revival: Option<bool>,
}

pub(super) fn marks_of_the_quarantined_record(state: &PoolState, arm: Arm, slot: u64) -> [bool; 2] {
    match arm {
        Arm::SpanBit => {
            let marked = state.records.get(&slot).is_some_and(|record| record.carries_the_quarantine_mark);
            [marked, marked]
        }
        Arm::QuarantineTree => quarantine_entries_of(slot).map(|entry| tree_contains(&state.quarantine_tree, &entry)),
        Arm::NoMark | Arm::PlainRelease => [false, false],
    }
}

fn arms_of_the_micro_history(micro: MicroHistory) -> Vec<Arm> {
    let mut arms = FORK_ARMS.to_vec();
    if micro == MicroHistory::CleanLeaf {
        arms.push(Arm::PlainRelease);
    }
    arms
}

/// 量一段微历史：每条臂各搭一遍、量那一次发布；点名数按 `no_mark` 那一遍垫到登记给的数，各臂垫同样多。
pub(super) fn measure_micro_history(micro: MicroHistory) -> Vec<MicroArmMeasurement> {
    let mut prepared: Vec<(Arm, PoolState, PublishPlan)> = Vec::new();
    for arm in arms_of_the_micro_history(micro) {
        let (state, request) = prepare_micro_history(micro, arm);
        let plan = plan_publish(&state, &request, arm);
        prepared.push((arm, state, plan));
    }
    let natural_named_roles_of_no_mark = prepared
        .iter()
        .find(|(arm, _, _)| *arm == Arm::NoMark)
        .map(|(_, _, plan)| plan.writes(0).last_transaction_named_roles)
        .expect("每段微历史都跑 no_mark");
    let padding_named_roles = micro
        .named_roles_of_the_last_transaction()
        .checked_sub(natural_named_roles_of_no_mark)
        .expect("微历史本来的点名数不超过登记给的数");
    let mut measurements = Vec::new();
    for (arm, mut state, plan) in prepared {
        let writes = plan.writes(padding_named_roles);
        let victim_slot = micro.victim_slot();
        let victim_leaf_rewritten = plan.rewritten_allocation_positions.contains(&AllocationNodePosition { level: 0, device: 0, index_in_device: leaf_of_slot(victim_slot) });
        let revived_slots: Vec<u64> = plan.revived.iter().map(|unit| state.unit(*unit).slot).collect();
        apply_publish(&mut state, plan);
        let mark_agreement_red_after_revival = (!revived_slots.is_empty()).then(|| {
            revived_slots.iter().any(|slot| {
                let view = RecordView {
                    is_released: false,
                    marked_on_device: marks_of_the_quarantined_record(&state, arm, *slot),
                    referenced_by_the_newest_root: true,
                    referenced_by_some_candidate_root: true,
                    unmapped_while_unreleased_in_some_candidate_version: false,
                    copies: CopyCondition::Intact,
                };
                checkers_of(arm).iter().any(|checker| judge_record(*checker, &view).mark_agreement_red)
            })
        });
        measurements.push(MicroArmMeasurement { arm, writes, padding_named_roles, victim_leaf_rewritten, mark_agreement_red_after_revival });
    }
    measurements
}

/// 一段微历史在一条臂上的额外写（臂减 `no_mark`，不含根槽与系统配置槽）。
pub(super) fn micro_extra(measurements: &[MicroArmMeasurement], arm: Arm) -> WriteDifference {
    let of = |wanted: Arm| measurements.iter().find(|measurement| measurement.arm == wanted).expect("这段微历史跑过这条臂").writes;
    difference(of(arm).without_root_and_system_configuration(), of(Arm::NoMark).without_root_and_system_configuration())
}

/// 第二段的一格（登记第五节「第二段」）：密度 × 回退率 × U × K × 几何 × 种子，三臂各跑一遍。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) struct HistoryCell {
    pub(super) density: u64,
    pub(super) rollback_rate: u64,
    pub(super) file_size: u32,
    pub(super) candidate_window: u64,
    pub(super) geometry: PoolGeometry,
    pub(super) seed: u64,
}

impl HistoryCell {
    pub(super) fn label(&self) -> String {
        format!(
            "density={} rollback_rate={} file_size={} window={} geometry={} seed={}",
            self.density,
            self.rollback_rate,
            self.file_size,
            self.candidate_window,
            self.geometry.label(),
            self.seed
        )
    }
}

pub(super) fn all_history_cells() -> Vec<HistoryCell> {
    let mut cells = Vec::new();
    for density in DENSITIES_PER_THOUSAND_PUBLISHES {
        for rollback_rate in ROLLBACK_RATES_PER_THOUSAND_PUBLISHES {
            for file_size in FILE_SIZES_IN_DATA_UNITS {
                for candidate_window in CANDIDATE_WINDOWS {
                    for geometry in GEOMETRIES {
                        for seed in SEEDS {
                            cells.push(HistoryCell { density, rollback_rate, file_size, candidate_window, geometry, seed });
                        }
                    }
                }
            }
        }
    }
    cells
}
