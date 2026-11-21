//! 发布条件、追加与恢复各臂、历史族、历史的全集，以及一个格的记录内容与记录流的定义与规划。

use super::*;

// ===== 臂、条件、历史 =====

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum PublishCondition {
    /// 串落：每个事务自成一次发布，发布之间串行。
    SerialPublishes,
    /// 同发：k 个事务进同一次发布。
    SharedPublish,
    /// 多飞·根序：每个事务自成一次发布，发布之间任意交错，只加一条：F 与 C 都按 txg 先后发出（登记 5.1）。
    MultiFlightRootsOrdered,
    /// 多飞·根乱：每个事务自成一次发布，发布之间任意交错，不加 F、C 的次序约束（登记 5.1）。
    MultiFlightRootsUnordered,
}

impl PublishCondition {
    /// 第一段跑的两个条件（登记 5.6 第一段）。
    pub(super) const SEGMENT_ONE: [PublishCondition; 2] = [PublishCondition::SerialPublishes, PublishCondition::SharedPublish];
    /// 第二段跑的两个条件（登记 5.6 第二段）。
    pub(super) const SEGMENT_TWO: [PublishCondition; 2] = [PublishCondition::MultiFlightRootsOrdered, PublishCondition::MultiFlightRootsUnordered];
    pub(super) fn label(self) -> &'static str {
        match self {
            PublishCondition::SerialPublishes => "serial_publishes",
            PublishCondition::SharedPublish => "shared_publish",
            PublishCondition::MultiFlightRootsOrdered => "multi_flight_roots_ordered",
            PublishCondition::MultiFlightRootsUnordered => "multi_flight_roots_unordered",
        }
    }
    pub(super) fn publishes_fly_together(self) -> bool {
        match self {
            PublishCondition::SerialPublishes | PublishCondition::SharedPublish => false,
            PublishCondition::MultiFlightRootsOrdered | PublishCondition::MultiFlightRootsUnordered => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum AppendArm {
    /// 甲整段：发出 Ti 的第一条之前，每个 j < i 的 Tj 记录全部已发出或已作废；jsn 按发出次序给。
    JiaWhole,
    /// 甲交错：Ti 的第一条在每个 j < i 的 Tj 的第一条之后发出；jsn 按发出次序给。
    JiaInterleaved,
    /// 乙整段：取号时按事务号预留连号 jsn，各事务乱序写自己的槽，作废的槽写占位记录。
    YiReservedWhole,
    /// 对照丁：没有纪律，jsn 按发出次序给。
    DingNoDiscipline,
}

impl AppendArm {
    pub(super) const ALL: [AppendArm; 4] = [
        AppendArm::JiaWhole,
        AppendArm::JiaInterleaved,
        AppendArm::YiReservedWhole,
        AppendArm::DingNoDiscipline,
    ];
    pub(super) fn label(self) -> &'static str {
        match self {
            AppendArm::JiaWhole => "jia_whole",
            AppendArm::JiaInterleaved => "jia_interleaved",
            AppendArm::YiReservedWhole => "yi_reserved_whole",
            AppendArm::DingNoDiscipline => "ding_control",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum RecoveryArm {
    Bing0Today,
    Bing1TruncateBeforeTheIncompleteTransaction,
    Bing2DropTheIncompleteKeepTheComplete,
    Bing3ForbidPattern,
}

impl RecoveryArm {
    pub(super) const ALL: [RecoveryArm; 4] = [
        RecoveryArm::Bing0Today,
        RecoveryArm::Bing1TruncateBeforeTheIncompleteTransaction,
        RecoveryArm::Bing2DropTheIncompleteKeepTheComplete,
        RecoveryArm::Bing3ForbidPattern,
    ];
    pub(super) fn label(self) -> &'static str {
        match self {
            RecoveryArm::Bing0Today => "bing0_today",
            RecoveryArm::Bing1TruncateBeforeTheIncompleteTransaction => "bing1_truncate_before_a",
            RecoveryArm::Bing2DropTheIncompleteKeepTheComplete => "bing2_drop_a_keep_b",
            RecoveryArm::Bing3ForbidPattern => "bing3_forbid",
        }
    }
    pub(super) fn position(self) -> usize {
        match self {
            RecoveryArm::Bing0Today => 0,
            RecoveryArm::Bing1TruncateBeforeTheIncompleteTransaction => 1,
            RecoveryArm::Bing2DropTheIncompleteKeepTheComplete => 2,
            RecoveryArm::Bing3ForbidPattern => 3,
        }
    }
}

/// D13（验证路线） 已定项 2「已追加」的两种读法（修订 ④：两读都跑，判据门槛相同、各自判）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum AppendedReading {
    /// R2：记录全部已发出，且其后已有一道屏障或 FUA 完成。
    IssuedAndCovered,
    /// R1：记录全部已发出。
    IssuedOnly,
}

impl AppendedReading {
    pub(super) const ALL: [AppendedReading; 2] = [AppendedReading::IssuedAndCovered, AppendedReading::IssuedOnly];
    pub(super) fn label(self) -> &'static str {
        match self {
            AppendedReading::IssuedAndCovered => "r2_covered",
            AppendedReading::IssuedOnly => "r1_issued",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ReturnRule {
    /// D16（发布语义） 已定项 7：本事务所在发布的最后一道屏障（C 之后那道）完成。
    AfterOwnPublishLastBarrier,
    /// 阳性对照「早返回」：本事务的记录全部发出就返回（条件 ② 照旧）。
    EarlyAfterOwnRecordsIssued,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum AppendDiscipline {
    Kept,
    /// Q2 每条臂的阳性对照：拿掉这条臂自己的纪律。
    RemovedForPositiveControl,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PublishBarriers {
    AllThree,
    /// V_dangling 的阳性对照：拿掉单元之后与记录之后那两道屏障。
    WithoutBarriersAfterUnitsAndRecords,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum RecoveryControl {
    NoControl,
    /// V_dep 的阳性对照：每条臂关掉「不保后面」的那一处、改成按记录施加。
    KeepLaterRecordsRemoved,
    /// N_atom 的阳性对照：每条臂去掉提交标记的处理并去掉整次发布施加。
    CommitMarkerHandlingRemoved,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum VoidTiming {
    /// H1 作废·窄：取号之后、发出任何写之前作废。
    NarrowBeforeAnyWrite,
    /// H2 作废·宽：发出 U 与第一条记录之后作废。
    WideAfterFirstRecord,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[allow(clippy::enum_variant_names, reason = "三个变体就是登记里「无作废、窄作废、宽作废」三族的名字，共同的「作废」是登记用语，去掉它名字就不再说明是哪一族")]
pub(super) enum HistoryFamily {
    WithoutVoid,
    NarrowVoid,
    WideVoid,
}

impl HistoryFamily {
    pub(super) fn label(self) -> &'static str {
        match self {
            HistoryFamily::WithoutVoid => "H0",
            HistoryFamily::NarrowVoid => "H1",
            HistoryFamily::WideVoid => "H2",
        }
    }
    pub(super) fn is_legal(self) -> bool {
        match self {
            HistoryFamily::WithoutVoid | HistoryFamily::NarrowVoid => true,
            HistoryFamily::WideVoid => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) struct History {
    /// 事务 T1…Tk 各自的记录条数 r_i。
    pub(super) record_counts: Vec<u32>,
    pub(super) voided: Option<(u32, VoidTiming)>,
}

impl History {
    pub(super) fn family(&self) -> HistoryFamily {
        match self.voided {
            None => HistoryFamily::WithoutVoid,
            Some((_, VoidTiming::NarrowBeforeAnyWrite)) => HistoryFamily::NarrowVoid,
            Some((_, VoidTiming::WideAfterFirstRecord)) => HistoryFamily::WideVoid,
        }
    }
    pub(super) fn transaction_count(&self) -> u32 {
        u32::try_from(self.record_counts.len()).expect("k 只取 2、3")
    }
    pub(super) fn record_count_of(&self, transaction: u32) -> u32 {
        self.record_counts[transaction_position(transaction)]
    }
    pub(super) fn void_timing_of(&self, transaction: u32) -> Option<VoidTiming> {
        match self.voided {
            Some((voided_transaction, timing)) if voided_transaction == transaction => Some(timing),
            Some(_) | None => None,
        }
    }
    pub(super) fn record_counts_label(&self) -> String {
        self.record_counts.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
    }
    pub(super) fn voided_label(&self) -> String {
        match self.voided {
            None => "none".to_string(),
            Some((transaction, _)) => format!("T{transaction}"),
        }
    }
}

pub(super) fn transaction_position(transaction: u32) -> usize {
    usize::try_from(transaction - 1).expect("事务号从 1 起")
}

/// 登记 5.4：H0 是 k ∈ {2, 3}、r ∈ {1, 2}^k；H1 是 H0 × 作废事务 v；H2 是 H0 × r_v = 2 的 v。
pub(super) fn all_histories() -> Vec<History> {
    let mut record_count_choices: Vec<Vec<u32>> = Vec::new();
    for transaction_count in [2u32, 3] {
        let combinations = 1u32 << transaction_count;
        for combination in 0..combinations {
            let counts: Vec<u32> = (0..transaction_count)
                .map(|bit| if combination & (1 << (transaction_count - 1 - bit)) == 0 { 1 } else { 2 })
                .collect();
            record_count_choices.push(counts);
        }
    }
    let mut histories: Vec<History> = Vec::new();
    for counts in &record_count_choices {
        histories.push(History { record_counts: counts.clone(), voided: None });
    }
    for counts in &record_count_choices {
        for voided_transaction in 1..=u32::try_from(counts.len()).expect("k 只取 2、3") {
            histories.push(History {
                record_counts: counts.clone(),
                voided: Some((voided_transaction, VoidTiming::NarrowBeforeAnyWrite)),
            });
        }
    }
    for counts in &record_count_choices {
        for voided_transaction in 1..=u32::try_from(counts.len()).expect("k 只取 2、3") {
            if counts[transaction_position(voided_transaction)] == 2 {
                histories.push(History {
                    record_counts: counts.clone(),
                    voided: Some((voided_transaction, VoidTiming::WideAfterFirstRecord)),
                });
            }
        }
    }
    histories
}

// ===== 写与操作 =====

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum RecordOrigin {
    OwnRecord { index_within_transaction: u32 },
    Filler { voided_transaction: u32, slot_within_transaction: u32 },
}

/// 一条记录写进盘里的内容（登记 5.3「记录的属性」）。按内容去重时连这些字段一起比。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) struct RecordContent {
    pub(super) jsn: u64,
    pub(super) txg: u64,
    /// 发布的身份（按落盘次序编号，作废·宽在串落下留下的半截发布也占一个身份，但不占 txg）。
    pub(super) publish: u32,
    /// 事务号；占位记录是 0。
    pub(super) transaction: u32,
    pub(super) is_commit: bool,
    pub(super) ends_its_publish: bool,
    pub(super) ordinal_within_publish: u32,
    pub(super) origin: RecordOrigin,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum WriteContent {
    TransactionUnit { transaction: u32 },
    SharedUnit { publish: u32 },
    JournalRecord(RecordContent),
    RootSlot { publish: u32, txg: u64 },
    SystemConfiguration { publish: u32 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WriteDurabilityStates {
    NewOrMissing,
    NewMissingOrTorn,
}

impl WriteDurabilityStates {
    pub(super) fn forms(self) -> &'static [Option<DurableForm>] {
        match self {
            WriteDurabilityStates::NewOrMissing => &[None, Some(DurableForm::New)],
            WriteDurabilityStates::NewMissingOrTorn => &[None, Some(DurableForm::New), Some(DurableForm::Torn)],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Operation {
    Write { content: WriteContent, states: WriteDurabilityStates },
    ForcedUnitAccessWrite { content: WriteContent },
    /// 屏障只发到这一块盘（单盘模型，登记第一节 1.2「设备」）。
    Barrier,
}

impl Operation {
    pub(super) fn closes_a_segment(&self) -> bool {
        match self {
            Operation::Write { .. } => false,
            Operation::ForcedUnitAccessWrite { .. } | Operation::Barrier => true,
        }
    }
    pub(super) fn written_content(&self) -> Option<WriteContent> {
        match self {
            Operation::Write { content, .. } | Operation::ForcedUnitAccessWrite { content } => Some(*content),
            Operation::Barrier => None,
        }
    }
    pub(super) fn durability_choices(&self) -> Option<(WriteContent, WriteDurabilityStates)> {
        match self {
            Operation::Write { content, states } => Some((*content, *states)),
            Operation::ForcedUnitAccessWrite { content } => Some((*content, WriteDurabilityStates::NewOrMissing)),
            Operation::Barrier => None,
        }
    }
}

// ===== 一格的布局：哪些发布、每次发布写哪些单元与记录流 =====

#[derive(Clone, Copy, Debug)]
pub(super) struct PlannedRecord {
    /// 事务号；占位记录是 0。
    pub(super) transaction: u32,
    pub(super) origin: RecordOrigin,
    pub(super) is_commit: bool,
    pub(super) reserved_jsn: Option<u64>,
    /// 发出这一条的时候本发布还有没发出的计划记录（作废·宽那个事务的第一条：它的第二条当时还在计划里）。
    pub(super) successor_planned_at_issue: bool,
}

#[derive(Clone, Debug)]
pub(super) struct RecordStream {
    /// 自己的记录流写事务号；占位流写被作废的那个事务的号。
    pub(super) transaction: u32,
    pub(super) is_filler: bool,
    pub(super) records: Vec<PlannedRecord>,
    /// 作废·宽留下的占位：作废发生在那个事务的第一条发出之后，占位只能排在它后面（同一次发布里才有意义）。
    pub(super) must_follow_first_record_of: Option<u32>,
}

#[derive(Clone, Debug)]
pub(super) struct PublishLayout {
    pub(super) identity: u32,
    pub(super) txg: u64,
    /// 串落下作废·宽的那个事务的发布：发出 U、S、第一道屏障与第一条记录之后放弃，不写根槽、不占 txg。
    pub(super) is_abandoned: bool,
    pub(super) unit_transactions: Vec<u32>,
    pub(super) record_streams: Vec<RecordStream>,
}

impl PublishLayout {
    fn written_record_count(&self) -> usize {
        self.record_streams.iter().map(|stream| stream.records.len()).sum()
    }
}

#[derive(Clone, Debug)]
pub(super) struct TransactionLayout {
    pub(super) number: u32,
    pub(super) planned_records: u32,
    pub(super) voided: Option<VoidTiming>,
    pub(super) txg: Option<u64>,
}

#[derive(Clone, Debug)]
pub(super) struct CellLayout {
    pub(super) transactions: Vec<TransactionLayout>,
    pub(super) publishes: Vec<PublishLayout>,
}

impl CellLayout {
    pub(super) fn written_record_count_of(&self, publish: u32) -> usize {
        self.publishes
            .iter()
            .find(|layout| layout.identity == publish)
            .map_or(0, PublishLayout::written_record_count)
    }
}

pub(super) fn own_record_stream(transaction: u32, record_count: u32, timing: Option<VoidTiming>, reserved_first_jsn: Option<u64>) -> Option<RecordStream> {
    let reserved = |slot: u32| reserved_first_jsn.map(|first| first + u64::from(slot - 1));
    match timing {
        Some(VoidTiming::NarrowBeforeAnyWrite) => None,
        Some(VoidTiming::WideAfterFirstRecord) => Some(RecordStream {
            transaction,
            is_filler: false,
            records: vec![PlannedRecord {
                transaction,
                origin: RecordOrigin::OwnRecord { index_within_transaction: 1 },
                is_commit: false,
                reserved_jsn: reserved(1),
                successor_planned_at_issue: true,
            }],
            must_follow_first_record_of: None,
        }),
        None => Some(RecordStream {
            transaction,
            is_filler: false,
            records: (1..=record_count)
                .map(|slot| PlannedRecord {
                    transaction,
                    origin: RecordOrigin::OwnRecord { index_within_transaction: slot },
                    is_commit: slot == record_count,
                    reserved_jsn: reserved(slot),
                    successor_planned_at_issue: false,
                })
                .collect(),
            must_follow_first_record_of: None,
        }),
    }
}

pub(super) fn filler_stream(voided_transaction: u32, first_slot: u32, record_count: u32, reserved_first_jsn: u64, follows_first_record: bool) -> Option<RecordStream> {
    if first_slot > record_count {
        return None;
    }
    Some(RecordStream {
        transaction: voided_transaction,
        is_filler: true,
        records: (first_slot..=record_count)
            .map(|slot| PlannedRecord {
                transaction: 0,
                origin: RecordOrigin::Filler { voided_transaction, slot_within_transaction: slot },
                is_commit: true,
                reserved_jsn: Some(reserved_first_jsn + u64::from(slot - 1)),
                successor_planned_at_issue: false,
            })
            .collect(),
        must_follow_first_record_of: if follows_first_record { Some(voided_transaction) } else { None },
    })
}
