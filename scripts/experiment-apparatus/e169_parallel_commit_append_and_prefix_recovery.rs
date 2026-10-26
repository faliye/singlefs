//! E169（并行提交的追加纪律与前缀恢复）第一段（不带参数）：Q1 的「串落」「同发」两个条件下，
//! Q2 四条追加臂 × 三族历史 × 两种「已追加」读法 × Q3 四条恢复臂的崩溃枚举；
//! 第二段（参数 `segment-two`）：Q1 的「多飞·根序」「多飞·根乱」两个条件下同样的量，k = 2 直接枚举执行、k = 3 按配置去重。
//! 跑前登记 `research/prompts/e169-preregistration.md`（含第十二节修订）；岔路单是
//! `research/prompts/m3-concurrency-questions.md` 的 Q2、Q3 两行。
//!
//! 独立手写模型：不与 `crates/` 共用代码、不引 `crates/` 的常量（`.claude/rules/implementation-first.md` 第 4 条）。
//! 丙0 是 `crates/singlefs-replay-prefix/src/lib.rs` 第 422–534 行逐判的转写，每一判旁边写那一行的行号；
//! 登记第七节 A2 的十四条固定记录序列（停机条款 T1）在开跑之前逐条核，有一条对不上就停、其余不跑。
//!
//! 崩溃状态照登记第一节 1.2：一个执行的写按屏障与 FUA 切段（D13（验证路线） 已定项 4），
//! 崩溃点 = （已完成的最后一道闭段点，本段里已发出到哪一个操作），持久集合是已发出的本段写各取各的态；
//! 已返回 fsync 的事务集合按崩溃点现算（修订 ④ 补的 R1 读法让返回可以落在两个闭段点之间，所以按发出前缀切）。
//! 同一格里全部执行的崩溃状态按内容去重。

// admission: always 每次照此刻的仓与自己的常量现算，上一次跑出的产物不替这一次作保
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// D23（journal 的角色与格式） 已定项 18 定案句：默认环下 196608 ÷ 3 = 65536
/// （抄自 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 18，登记第二节第 266 行原样抄录）。
const IN_FLIGHT_RECORD_LIMIT: usize = 65_536;
/// 登记第七节 A6：k = 3、r = 2 时严格连续段至多 6 条。
const STRICT_SEGMENT_LENGTH_BOUND: usize = 6;
/// 历史开始之前的根：txg 0，恒持久，覆盖到 jsn 0（登记第一节 1.2「恢复」）。
const START_ROOT_TXG: u64 = 0;
/// 系统配置轮换写是原地覆写，取三态（D13（验证路线） 已定项 4）。
const SYSTEM_CONFIGURATION_DURABILITY: WriteDurabilityStates = WriteDurabilityStates::NewMissingOrTorn;
/// 根槽写带 FUA：关掉它所在的那一段（D13（验证路线） 已定项 4）。
const ROOT_SLOT_WRITE_CLOSES_ITS_SEGMENT: bool = true;

// ===== 臂、条件、历史 =====

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum PublishCondition {
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
    const SEGMENT_ONE: [PublishCondition; 2] = [PublishCondition::SerialPublishes, PublishCondition::SharedPublish];
    /// 第二段跑的两个条件（登记 5.6 第二段）。
    const SEGMENT_TWO: [PublishCondition; 2] = [PublishCondition::MultiFlightRootsOrdered, PublishCondition::MultiFlightRootsUnordered];
    fn label(self) -> &'static str {
        match self {
            PublishCondition::SerialPublishes => "serial_publishes",
            PublishCondition::SharedPublish => "shared_publish",
            PublishCondition::MultiFlightRootsOrdered => "multi_flight_roots_ordered",
            PublishCondition::MultiFlightRootsUnordered => "multi_flight_roots_unordered",
        }
    }
    fn publishes_fly_together(self) -> bool {
        match self {
            PublishCondition::SerialPublishes | PublishCondition::SharedPublish => false,
            PublishCondition::MultiFlightRootsOrdered | PublishCondition::MultiFlightRootsUnordered => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum AppendArm {
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
    const ALL: [AppendArm; 4] = [
        AppendArm::JiaWhole,
        AppendArm::JiaInterleaved,
        AppendArm::YiReservedWhole,
        AppendArm::DingNoDiscipline,
    ];
    fn label(self) -> &'static str {
        match self {
            AppendArm::JiaWhole => "jia_whole",
            AppendArm::JiaInterleaved => "jia_interleaved",
            AppendArm::YiReservedWhole => "yi_reserved_whole",
            AppendArm::DingNoDiscipline => "ding_control",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum RecoveryArm {
    Bing0Today,
    Bing1TruncateBeforeA,
    Bing2DropAKeepB,
    Bing3ForbidPattern,
}

impl RecoveryArm {
    const ALL: [RecoveryArm; 4] = [
        RecoveryArm::Bing0Today,
        RecoveryArm::Bing1TruncateBeforeA,
        RecoveryArm::Bing2DropAKeepB,
        RecoveryArm::Bing3ForbidPattern,
    ];
    fn label(self) -> &'static str {
        match self {
            RecoveryArm::Bing0Today => "bing0_today",
            RecoveryArm::Bing1TruncateBeforeA => "bing1_truncate_before_a",
            RecoveryArm::Bing2DropAKeepB => "bing2_drop_a_keep_b",
            RecoveryArm::Bing3ForbidPattern => "bing3_forbid",
        }
    }
    fn position(self) -> usize {
        match self {
            RecoveryArm::Bing0Today => 0,
            RecoveryArm::Bing1TruncateBeforeA => 1,
            RecoveryArm::Bing2DropAKeepB => 2,
            RecoveryArm::Bing3ForbidPattern => 3,
        }
    }
}

/// D13（验证路线） 已定项 2「已追加」的两种读法（修订 ④：两读都跑，判据门槛相同、各自判）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum AppendedReading {
    /// R2：记录全部已发出，且其后已有一道屏障或 FUA 完成。
    R2IssuedAndCovered,
    /// R1：记录全部已发出。
    R1Issued,
}

impl AppendedReading {
    const ALL: [AppendedReading; 2] = [AppendedReading::R2IssuedAndCovered, AppendedReading::R1Issued];
    fn label(self) -> &'static str {
        match self {
            AppendedReading::R2IssuedAndCovered => "r2_covered",
            AppendedReading::R1Issued => "r1_issued",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ReturnRule {
    /// D16（发布语义） 已定项 7：本事务所在发布的最后一道屏障（C 之后那道）完成。
    AfterOwnPublishLastBarrier,
    /// 阳性对照「早返回」：本事务的记录全部发出就返回（条件 ② 照旧）。
    EarlyAfterOwnRecordsIssued,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AppendDiscipline {
    Kept,
    /// Q2 每条臂的阳性对照：拿掉这条臂自己的纪律。
    RemovedForPositiveControl,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PublishBarriers {
    AllThree,
    /// V_dangling 的阳性对照：拿掉单元之后与记录之后那两道屏障。
    WithoutBarriersAfterUnitsAndRecords,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RecoveryControl {
    NoControl,
    /// V_dep 的阳性对照：每条臂关掉「不保后面」的那一处、改成按记录施加。
    KeepLaterRecordsRemoved,
    /// N_atom 的阳性对照：每条臂去掉提交标记的处理并去掉整次发布施加。
    CommitMarkerHandlingRemoved,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum VoidTiming {
    /// H1 作废·窄：取号之后、发出任何写之前作废。
    NarrowBeforeAnyWrite,
    /// H2 作废·宽：发出 U 与第一条记录之后作废。
    WideAfterFirstRecord,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum HistoryFamily {
    H0,
    H1,
    H2,
}

impl HistoryFamily {
    fn label(self) -> &'static str {
        match self {
            HistoryFamily::H0 => "H0",
            HistoryFamily::H1 => "H1",
            HistoryFamily::H2 => "H2",
        }
    }
    fn is_legal(self) -> bool {
        match self {
            HistoryFamily::H0 | HistoryFamily::H1 => true,
            HistoryFamily::H2 => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct History {
    /// 事务 T1…Tk 各自的记录条数 r_i。
    record_counts: Vec<u32>,
    voided: Option<(u32, VoidTiming)>,
}

impl History {
    fn family(&self) -> HistoryFamily {
        match self.voided {
            None => HistoryFamily::H0,
            Some((_, VoidTiming::NarrowBeforeAnyWrite)) => HistoryFamily::H1,
            Some((_, VoidTiming::WideAfterFirstRecord)) => HistoryFamily::H2,
        }
    }
    fn transaction_count(&self) -> u32 {
        u32::try_from(self.record_counts.len()).expect("k 只取 2、3")
    }
    fn record_count_of(&self, transaction: u32) -> u32 {
        self.record_counts[transaction_position(transaction)]
    }
    fn void_timing_of(&self, transaction: u32) -> Option<VoidTiming> {
        match self.voided {
            Some((voided_transaction, timing)) if voided_transaction == transaction => Some(timing),
            Some(_) | None => None,
        }
    }
    fn record_counts_label(&self) -> String {
        self.record_counts.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
    }
    fn voided_label(&self) -> String {
        match self.voided {
            None => "none".to_string(),
            Some((transaction, _)) => format!("T{transaction}"),
        }
    }
}

fn transaction_position(transaction: u32) -> usize {
    usize::try_from(transaction - 1).expect("事务号从 1 起")
}

/// 登记 5.4：H0 是 k ∈ {2, 3}、r ∈ {1, 2}^k；H1 是 H0 × 作废事务 v；H2 是 H0 × r_v = 2 的 v。
fn all_histories() -> Vec<History> {
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
enum RecordOrigin {
    OwnRecord { index_within_transaction: u32 },
    Filler { voided_transaction: u32, slot_within_transaction: u32 },
}

/// 一条记录写进盘里的内容（登记 5.3「记录的属性」）。按内容去重时连这些字段一起比。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct RecordContent {
    jsn: u64,
    txg: u64,
    /// 发布的身份（按落盘次序编号，作废·宽在串落下留下的半截发布也占一个身份，但不占 txg）。
    publish: u32,
    /// 事务号；占位记录是 0。
    transaction: u32,
    is_commit: bool,
    ends_its_publish: bool,
    ordinal_within_publish: u32,
    origin: RecordOrigin,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum WriteContent {
    TransactionUnit { transaction: u32 },
    SharedUnit { publish: u32 },
    JournalRecord(RecordContent),
    RootSlot { publish: u32, txg: u64 },
    SystemConfiguration { publish: u32 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WriteDurabilityStates {
    NewOrMissing,
    NewMissingOrTorn,
}

impl WriteDurabilityStates {
    fn forms(self) -> &'static [Option<DurableForm>] {
        match self {
            WriteDurabilityStates::NewOrMissing => &[None, Some(DurableForm::New)],
            WriteDurabilityStates::NewMissingOrTorn => &[None, Some(DurableForm::New), Some(DurableForm::Torn)],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Operation {
    Write { content: WriteContent, states: WriteDurabilityStates },
    ForcedUnitAccessWrite { content: WriteContent },
    /// 屏障只发到这一块盘（单盘模型，登记第一节 1.2「设备」）。
    Barrier,
}

impl Operation {
    fn closes_a_segment(&self) -> bool {
        match self {
            Operation::Write { .. } => false,
            Operation::ForcedUnitAccessWrite { .. } | Operation::Barrier { .. } => true,
        }
    }
    fn written_content(&self) -> Option<WriteContent> {
        match self {
            Operation::Write { content, .. } | Operation::ForcedUnitAccessWrite { content } => Some(*content),
            Operation::Barrier { .. } => None,
        }
    }
    fn durability_choices(&self) -> Option<(WriteContent, WriteDurabilityStates)> {
        match self {
            Operation::Write { content, states } => Some((*content, *states)),
            Operation::ForcedUnitAccessWrite { content } => Some((*content, WriteDurabilityStates::NewOrMissing)),
            Operation::Barrier { .. } => None,
        }
    }
}

// ===== 一格的布局：哪些发布、每次发布写哪些单元与记录流 =====

#[derive(Clone, Copy, Debug)]
struct PlannedRecord {
    /// 事务号；占位记录是 0。
    transaction: u32,
    origin: RecordOrigin,
    is_commit: bool,
    reserved_jsn: Option<u64>,
    /// 发出这一条的时候本发布还有没发出的计划记录（作废·宽那个事务的第一条：它的第二条当时还在计划里）。
    successor_planned_at_issue: bool,
}

#[derive(Clone, Debug)]
struct RecordStream {
    /// 自己的记录流写事务号；占位流写被作废的那个事务的号。
    transaction: u32,
    is_filler: bool,
    records: Vec<PlannedRecord>,
    /// 作废·宽留下的占位：作废发生在那个事务的第一条发出之后，占位只能排在它后面（同一次发布里才有意义）。
    must_follow_first_record_of: Option<u32>,
}

#[derive(Clone, Debug)]
struct PublishLayout {
    identity: u32,
    txg: u64,
    /// 串落下作废·宽的那个事务的发布：发出 U、S、第一道屏障与第一条记录之后放弃，不写根槽、不占 txg。
    is_abandoned: bool,
    unit_transactions: Vec<u32>,
    record_streams: Vec<RecordStream>,
}

impl PublishLayout {
    fn written_record_count(&self) -> usize {
        self.record_streams.iter().map(|stream| stream.records.len()).sum()
    }
}

#[derive(Clone, Debug)]
struct TransactionLayout {
    number: u32,
    planned_records: u32,
    voided: Option<VoidTiming>,
    txg: Option<u64>,
}

#[derive(Clone, Debug)]
struct CellLayout {
    transactions: Vec<TransactionLayout>,
    publishes: Vec<PublishLayout>,
}

impl CellLayout {
    fn written_record_count_of(&self, publish: u32) -> usize {
        self.publishes
            .iter()
            .find(|layout| layout.identity == publish)
            .map_or(0, PublishLayout::written_record_count)
    }
}

fn own_record_stream(transaction: u32, record_count: u32, timing: Option<VoidTiming>, reserved_first_jsn: Option<u64>) -> Option<RecordStream> {
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

fn filler_stream(voided_transaction: u32, first_slot: u32, record_count: u32, reserved_first_jsn: u64, follows_first_record: bool) -> Option<RecordStream> {
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

/// 一格的布局（登记 5.1 的写怎么排、5.2 的臂、5.4 的历史；txg 与半截发布的读法见第十二节修订）。
fn plan_cell(condition: PublishCondition, append_arm: AppendArm, history: &History, discipline: AppendDiscipline) -> CellLayout {
    let reserves_jsn_at_acquisition = append_arm == AppendArm::YiReservedWhole && discipline == AppendDiscipline::Kept;
    let narrow_void_leaves_fillers = reserves_jsn_at_acquisition;
    let mut reserved_first_jsns: Vec<u64> = Vec::new();
    let mut next_reserved_jsn = 1u64;
    for record_count in &history.record_counts {
        reserved_first_jsns.push(next_reserved_jsn);
        next_reserved_jsn += u64::from(*record_count);
    }
    let reserved_first_of = |transaction: u32| -> Option<u64> {
        if reserves_jsn_at_acquisition {
            Some(reserved_first_jsns[transaction_position(transaction)])
        } else {
            None
        }
    };
    let fillers_of = |transaction: u32| -> Option<RecordStream> {
        let first = reserved_first_of(transaction)?;
        let record_count = history.record_count_of(transaction);
        match history.void_timing_of(transaction) {
            Some(VoidTiming::NarrowBeforeAnyWrite) if narrow_void_leaves_fillers => filler_stream(transaction, 1, record_count, first, false),
            Some(VoidTiming::NarrowBeforeAnyWrite) | None => None,
            Some(VoidTiming::WideAfterFirstRecord) => filler_stream(transaction, 2, record_count, first, true),
        }
    };
    let mut transactions: Vec<TransactionLayout> = (1..=history.transaction_count())
        .map(|number| TransactionLayout {
            number,
            planned_records: history.record_count_of(number),
            voided: history.void_timing_of(number),
            txg: None,
        })
        .collect();
    let mut publishes: Vec<PublishLayout> = Vec::new();
    match condition {
        PublishCondition::SharedPublish => {
            let mut unit_transactions = Vec::new();
            let mut record_streams = Vec::new();
            for transaction in 1..=history.transaction_count() {
                let timing = history.void_timing_of(transaction);
                if let Some(stream) = own_record_stream(transaction, history.record_count_of(transaction), timing, reserved_first_of(transaction)) {
                    unit_transactions.push(transaction);
                    record_streams.push(stream);
                }
                if let Some(stream) = fillers_of(transaction) {
                    record_streams.push(stream);
                }
                transactions[transaction_position(transaction)].txg = match timing {
                    Some(VoidTiming::NarrowBeforeAnyWrite) => None,
                    Some(VoidTiming::WideAfterFirstRecord) | None => Some(1),
                };
            }
            publishes.push(PublishLayout { identity: 1, txg: 1, is_abandoned: false, unit_transactions, record_streams });
        }
        // 多飞与串落的布局相同：每个没作废的事务自成一次发布，txg 按事务号连号（txg 从上一版现算，作废·窄的事务没开发布）；
        // 乙整段的占位写在作废事务之后紧接着的那个发布里，没有下一个时另开一次空发布（登记 5.2「没有永久的洞」）。
        // 两者只差执行怎么排：串落一次一个发布，多飞发布之间任意交错（`multi_flight_executions`）。
        PublishCondition::SerialPublishes | PublishCondition::MultiFlightRootsOrdered | PublishCondition::MultiFlightRootsUnordered => {
            let mut completed_publishes = 0u64;
            let mut identity = 0u32;
            let mut pending_fillers: Vec<RecordStream> = Vec::new();
            for transaction in 1..=history.transaction_count() {
                let timing = history.void_timing_of(transaction);
                match timing {
                    Some(VoidTiming::NarrowBeforeAnyWrite) => {
                        if let Some(stream) = fillers_of(transaction) {
                            pending_fillers.push(stream);
                        }
                    }
                    Some(VoidTiming::WideAfterFirstRecord) => {
                        identity += 1;
                        let stream = own_record_stream(transaction, history.record_count_of(transaction), timing, reserved_first_of(transaction))
                            .expect("作废·宽的事务发出了第一条记录");
                        publishes.push(PublishLayout {
                            identity,
                            txg: completed_publishes + 1,
                            is_abandoned: true,
                            unit_transactions: vec![transaction],
                            record_streams: vec![stream],
                        });
                        transactions[transaction_position(transaction)].txg = Some(completed_publishes + 1);
                        if let Some(stream) = fillers_of(transaction) {
                            pending_fillers.push(stream);
                        }
                    }
                    None => {
                        identity += 1;
                        completed_publishes += 1;
                        let mut record_streams: Vec<RecordStream> = std::mem::take(&mut pending_fillers);
                        record_streams.push(
                            own_record_stream(transaction, history.record_count_of(transaction), None, reserved_first_of(transaction))
                                .expect("没作废的事务有自己的记录流"),
                        );
                        publishes.push(PublishLayout {
                            identity,
                            txg: completed_publishes,
                            is_abandoned: false,
                            unit_transactions: vec![transaction],
                            record_streams,
                        });
                        transactions[transaction_position(transaction)].txg = Some(completed_publishes);
                    }
                }
            }
            if !pending_fillers.is_empty() {
                identity += 1;
                completed_publishes += 1;
                publishes.push(PublishLayout {
                    identity,
                    txg: completed_publishes,
                    is_abandoned: false,
                    unit_transactions: Vec::new(),
                    record_streams: pending_fillers,
                });
            }
        }
    }
    CellLayout { transactions, publishes }
}

// ===== 执行：记录流的发出次序 =====

fn interleavings(stream_lengths: &[usize]) -> Vec<Vec<usize>> {
    let total: usize = stream_lengths.iter().sum();
    let mut results = Vec::new();
    let mut current = Vec::with_capacity(total);
    let mut taken = vec![0usize; stream_lengths.len()];
    extend_interleavings(stream_lengths, &mut taken, &mut current, total, &mut results);
    results
}

fn extend_interleavings(stream_lengths: &[usize], taken: &mut [usize], current: &mut Vec<usize>, total: usize, results: &mut Vec<Vec<usize>>) {
    if current.len() == total {
        results.push(current.clone());
        return;
    }
    for stream_index in 0..stream_lengths.len() {
        if taken[stream_index] < stream_lengths[stream_index] {
            taken[stream_index] += 1;
            current.push(stream_index);
            extend_interleavings(stream_lengths, taken, current, total, results);
            current.pop();
            taken[stream_index] -= 1;
        }
    }
}

fn order_is_allowed(order: &[usize], streams: &[RecordStream], append_arm: AppendArm, discipline: AppendDiscipline) -> bool {
    let first_position = |stream_index: usize| order.iter().position(|chosen| *chosen == stream_index).expect("每条流至少一条记录");
    let last_position = |stream_index: usize| order.iter().rposition(|chosen| *chosen == stream_index).expect("每条流至少一条记录");
    for (stream_index, stream) in streams.iter().enumerate() {
        if let Some(leader_transaction) = stream.must_follow_first_record_of {
            if let Some(leader_index) = streams.iter().position(|candidate| !candidate.is_filler && candidate.transaction == leader_transaction) {
                if first_position(stream_index) < first_position(leader_index) {
                    return false;
                }
            }
        }
    }
    if discipline == AppendDiscipline::RemovedForPositiveControl {
        return true;
    }
    for (earlier_index, earlier) in streams.iter().enumerate() {
        for (later_index, later) in streams.iter().enumerate() {
            if earlier.is_filler || later.is_filler || earlier.transaction >= later.transaction {
                continue;
            }
            let later_first = first_position(later_index);
            let earlier_first = first_position(earlier_index);
            let earlier_last = last_position(earlier_index);
            let violates = match append_arm {
                AppendArm::JiaWhole => later_first < earlier_last,
                AppendArm::JiaInterleaved => later_first < earlier_first,
                AppendArm::YiReservedWhole | AppendArm::DingNoDiscipline => false,
            };
            if violates {
                return false;
            }
        }
    }
    true
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum VoidPoint {
    FromTheStart,
    AfterOperation(usize),
}

#[derive(Clone, Debug)]
struct TransactionTimeline {
    number: u32,
    planned_records: u32,
    issued_record_operation_indices: Vec<usize>,
    final_barrier_operation_index: Option<usize>,
    voided_from: Option<VoidPoint>,
}

#[derive(Clone, Debug)]
struct Execution {
    operations: Vec<Operation>,
    timelines: Vec<TransactionTimeline>,
}

fn place_records_of_publish(publish: &PublishLayout, order: &[usize], next_issue_jsn: &mut u64) -> Vec<(PlannedRecord, RecordContent)> {
    let mut cursor = vec![0usize; publish.record_streams.len()];
    let mut issued: Vec<(PlannedRecord, u64)> = Vec::new();
    for stream_index in order {
        let planned = publish.record_streams[*stream_index].records[cursor[*stream_index]];
        cursor[*stream_index] += 1;
        let jsn = match planned.reserved_jsn {
            Some(reserved) => reserved,
            None => {
                let issue_jsn = *next_issue_jsn;
                *next_issue_jsn += 1;
                issue_jsn
            }
        };
        issued.push((planned, jsn));
    }
    let mut jsns_ascending: Vec<u64> = issued.iter().map(|(_, jsn)| *jsn).collect();
    jsns_ascending.sort_unstable();
    // 末条标志在一次发布 jsn 最大的那条上；那一条发出时本发布还有计划中的记录（作废·宽的第一条），它不知道自己是末条。
    let end_jsn = issued
        .iter()
        .max_by_key(|(_, jsn)| *jsn)
        .filter(|(planned, _)| !planned.successor_planned_at_issue)
        .map(|(_, jsn)| *jsn);
    issued
        .iter()
        .map(|(planned, jsn)| {
            let ordinal = jsns_ascending.iter().position(|candidate| candidate == jsn).expect("jsn 在本发布里") + 1;
            (
                *planned,
                RecordContent {
                    jsn: *jsn,
                    txg: publish.txg,
                    publish: publish.identity,
                    transaction: planned.transaction,
                    is_commit: planned.is_commit,
                    ends_its_publish: Some(*jsn) == end_jsn,
                    ordinal_within_publish: u32::try_from(ordinal).expect("一次发布至多 6 条记录"),
                    origin: planned.origin,
                },
            )
        })
        .collect()
}

fn execution_for(layout: &CellLayout, orders: &[Vec<Vec<usize>>], choice: &[usize], barriers: PublishBarriers) -> Execution {
    let mut operations: Vec<Operation> = Vec::new();
    let mut next_issue_jsn = 1u64;
    let mut timelines: Vec<TransactionTimeline> = layout
        .transactions
        .iter()
        .map(|transaction| TransactionTimeline {
            number: transaction.number,
            planned_records: transaction.planned_records,
            issued_record_operation_indices: Vec::new(),
            final_barrier_operation_index: None,
            voided_from: match transaction.voided {
                Some(VoidTiming::NarrowBeforeAnyWrite) => Some(VoidPoint::FromTheStart),
                Some(VoidTiming::WideAfterFirstRecord) | None => None,
            },
        })
        .collect();
    for (publish_position, publish) in layout.publishes.iter().enumerate() {
        let order = &orders[publish_position][choice[publish_position]];
        for transaction in &publish.unit_transactions {
            operations.push(Operation::Write {
                content: WriteContent::TransactionUnit { transaction: *transaction },
                states: WriteDurabilityStates::NewOrMissing,
            });
        }
        operations.push(Operation::Write {
            content: WriteContent::SharedUnit { publish: publish.identity },
            states: WriteDurabilityStates::NewOrMissing,
        });
        if barriers == PublishBarriers::AllThree {
            // B1：单元与共享单元之后
            operations.push(Operation::Barrier);
        }
        for (planned, content) in place_records_of_publish(publish, order, &mut next_issue_jsn) {
            let operation_index = operations.len();
            operations.push(Operation::Write {
                content: WriteContent::JournalRecord(content),
                states: WriteDurabilityStates::NewOrMissing,
            });
            if planned.transaction != 0 {
                let timeline = &mut timelines[transaction_position(planned.transaction)];
                timeline.issued_record_operation_indices.push(operation_index);
                if layout.transactions[transaction_position(planned.transaction)].voided == Some(VoidTiming::WideAfterFirstRecord) {
                    timeline.voided_from = Some(VoidPoint::AfterOperation(operation_index));
                }
            }
        }
        if publish.is_abandoned {
            continue;
        }
        if barriers == PublishBarriers::AllThree {
            // B2：记录之后
            operations.push(Operation::Barrier);
        }
        let root_slot = WriteContent::RootSlot { publish: publish.identity, txg: publish.txg };
        if ROOT_SLOT_WRITE_CLOSES_ITS_SEGMENT {
            operations.push(Operation::ForcedUnitAccessWrite { content: root_slot });
        } else {
            operations.push(Operation::Write { content: root_slot, states: WriteDurabilityStates::NewOrMissing });
        }
        operations.push(Operation::Write {
            content: WriteContent::SystemConfiguration { publish: publish.identity },
            states: SYSTEM_CONFIGURATION_DURABILITY,
        });
        let final_barrier_index = operations.len();
        // B3：系统配置轮换之后
        operations.push(Operation::Barrier);
        for transaction in &publish.unit_transactions {
            if layout.transactions[transaction_position(*transaction)].voided.is_none() {
                timelines[transaction_position(*transaction)].final_barrier_operation_index = Some(final_barrier_index);
            }
        }
    }
    Execution { operations, timelines }
}

/// 每次发布里允许的记录发出次序（发布内的臂约束）。
fn allowed_record_orders(layout: &CellLayout, append_arm: AppendArm, discipline: AppendDiscipline) -> Vec<Vec<Vec<usize>>> {
    let orders: Vec<Vec<Vec<usize>>> = layout
        .publishes
        .iter()
        .map(|publish| {
            let lengths: Vec<usize> = publish.record_streams.iter().map(|stream| stream.records.len()).collect();
            interleavings(&lengths)
                .into_iter()
                .filter(|order| order_is_allowed(order, &publish.record_streams, append_arm, discipline))
                .collect()
        })
        .collect();
    assert!(orders.iter().all(|choices| !choices.is_empty()), "每次发布至少有一种允许的发出次序");
    orders
}

fn build_executions(layout: &CellLayout, append_arm: AppendArm, discipline: AppendDiscipline, barriers: PublishBarriers) -> Vec<Execution> {
    let orders = allowed_record_orders(layout, append_arm, discipline);
    let mut executions = Vec::new();
    let mut choice = vec![0usize; orders.len()];
    loop {
        executions.push(execution_for(layout, &orders, &choice, barriers));
        let mut position = choice.len();
        loop {
            if position == 0 {
                return executions;
            }
            position -= 1;
            choice[position] += 1;
            if choice[position] < orders[position].len() {
                break;
            }
            choice[position] = 0;
        }
    }
}

// ===== 崩溃状态与 fsync 返回 =====

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum DurableForm {
    New,
    Torn,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum ReturnBranch {
    /// 析取支 ①：小于它的事务全部已追加。
    EveryEarlierAppended,
    /// 析取支 ②：至少一个小于它的事务是靠「已作废」放行的。
    SomeEarlierPassedAsVoided,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct CrashState {
    durable: Vec<(WriteContent, DurableForm)>,
    returned: Vec<(u32, ReturnBranch)>,
}

fn is_voided_by(timeline: &TransactionTimeline, issued_through: Option<usize>) -> bool {
    match timeline.voided_from {
        None => false,
        Some(VoidPoint::FromTheStart) => true,
        Some(VoidPoint::AfterOperation(operation_index)) => issued_through.is_some_and(|issued| issued >= operation_index),
    }
}

fn all_records_issued(timeline: &TransactionTimeline, issued_through: Option<usize>) -> bool {
    let planned = usize::try_from(timeline.planned_records).expect("r 只取 1、2");
    timeline.issued_record_operation_indices.len() == planned
        && timeline
            .issued_record_operation_indices
            .iter()
            .all(|operation_index| issued_through.is_some_and(|issued| *operation_index <= issued))
}

fn is_appended(timeline: &TransactionTimeline, completed_through: Option<usize>, issued_through: Option<usize>, reading: AppendedReading) -> bool {
    if !all_records_issued(timeline, issued_through) {
        return false;
    }
    match reading {
        AppendedReading::R1Issued => true,
        AppendedReading::R2IssuedAndCovered => {
            let last_record_index = timeline.issued_record_operation_indices.iter().copied().max().expect("r ≥ 1");
            // completed_through 是一个闭段点：它在最后一条记录之后，就有一道完成了的屏障或 FUA 罩住那些记录。
            completed_through.is_some_and(|completed| last_record_index < completed)
        }
    }
}

/// 登记第一节 1.2「fsync 返回」：① 本事务所在发布的最后一道屏障已完成；② 每个 j < i 已作废或已追加。
fn returned_transactions(execution: &Execution, completed_through: Option<usize>, issued_through: Option<usize>, reading: AppendedReading, rule: ReturnRule) -> Vec<(u32, ReturnBranch)> {
    let mut returned = Vec::new();
    for timeline in &execution.timelines {
        if is_voided_by(timeline, issued_through) {
            continue;
        }
        let own_condition = match rule {
            ReturnRule::AfterOwnPublishLastBarrier => timeline
                .final_barrier_operation_index
                .is_some_and(|barrier| completed_through.is_some_and(|completed| barrier <= completed)),
            ReturnRule::EarlyAfterOwnRecordsIssued => all_records_issued(timeline, issued_through),
        };
        if !own_condition {
            continue;
        }
        let mut passed_as_voided = false;
        let mut blocked = false;
        for earlier in execution.timelines.iter().filter(|candidate| candidate.number < timeline.number) {
            if is_appended(earlier, completed_through, issued_through, reading) {
                continue;
            }
            if is_voided_by(earlier, issued_through) {
                passed_as_voided = true;
                continue;
            }
            blocked = true;
            break;
        }
        if blocked {
            continue;
        }
        let branch = if passed_as_voided { ReturnBranch::SomeEarlierPassedAsVoided } else { ReturnBranch::EveryEarlierAppended };
        returned.push((timeline.number, branch));
    }
    returned
}

/// 一个执行的全部崩溃状态并进 `states`；返回第七节 A7 不成立的崩溃点个数（返回集合里有事务的最后一道屏障还没完成）。
fn collect_crash_states(execution: &Execution, reading: AppendedReading, rule: ReturnRule, states: &mut HashSet<CrashState>) -> u64 {
    let operations = &execution.operations;
    let mut boundaries: Vec<Option<usize>> = vec![None];
    boundaries.extend(
        operations
            .iter()
            .enumerate()
            .filter(|(_, operation)| operation.closes_a_segment())
            .map(|(operation_index, _)| Some(operation_index)),
    );
    let mut a7_violation_points = 0u64;
    for (boundary_position, completed_through) in boundaries.iter().copied().enumerate() {
        let segment_first = completed_through.map_or(0, |completed| completed + 1);
        let segment_end_exclusive = match boundaries.get(boundary_position + 1) {
            Some(next_closer) => next_closer.expect("第一个之后都是闭段点") + 1,
            None => operations.len(),
        };
        let base: Vec<(WriteContent, DurableForm)> = operations[..segment_first]
            .iter()
            .filter_map(Operation::written_content)
            .map(|content| (content, DurableForm::New))
            .collect();
        let mut issue_candidates: Vec<Option<usize>> = vec![completed_through];
        issue_candidates.extend((segment_first..segment_end_exclusive).map(Some));
        let mut groups: Vec<(Option<usize>, Vec<(u32, ReturnBranch)>)> = Vec::new();
        for issued_through in issue_candidates {
            let returned = returned_transactions(execution, completed_through, issued_through, reading, rule);
            let extends_last_group = groups.last().is_some_and(|(_, last_returned)| *last_returned == returned);
            if extends_last_group {
                groups.last_mut().expect("刚看过非空").0 = issued_through;
            } else {
                groups.push((issued_through, returned));
            }
        }
        for (issued_through, returned) in groups {
            for (number, _) in &returned {
                let timeline = &execution.timelines[transaction_position(*number)];
                let barrier_done = timeline
                    .final_barrier_operation_index
                    .is_some_and(|barrier| completed_through.is_some_and(|completed| barrier <= completed));
                if !barrier_done {
                    a7_violation_points += 1;
                }
            }
            let segment_writes: Vec<(WriteContent, WriteDurabilityStates)> = (segment_first..segment_end_exclusive)
                .filter(|operation_index| issued_through.is_some_and(|issued| *operation_index <= issued))
                .filter_map(|operation_index| operations[operation_index].durability_choices())
                .collect();
            insert_every_durability_choice(&base, &segment_writes, &returned, states);
        }
    }
    a7_violation_points
}

fn insert_every_durability_choice(base: &[(WriteContent, DurableForm)], segment_writes: &[(WriteContent, WriteDurabilityStates)], returned: &[(u32, ReturnBranch)], states: &mut HashSet<CrashState>) {
    let mut digits = vec![0usize; segment_writes.len()];
    loop {
        let mut durable: Vec<(WriteContent, DurableForm)> = base.to_vec();
        for (position, (content, durability)) in segment_writes.iter().enumerate() {
            if let Some(form) = durability.forms()[digits[position]] {
                durable.push((*content, form));
            }
        }
        durable.sort();
        states.insert(CrashState { durable, returned: returned.to_vec() });
        let mut position = digits.len();
        loop {
            if position == 0 {
                return;
            }
            position -= 1;
            digits[position] += 1;
            if digits[position] < segment_writes[position].1.forms().len() {
                break;
            }
            digits[position] = 0;
        }
    }
}

// ===== 第二段：多飞（发布之间任意交错） =====

/// 第二段 k = 3 只跑这两种 r（登记 5.6 第二段）。
const SEGMENT_TWO_THREE_TRANSACTION_RECORD_COUNTS: [&[u32]; 2] = [&[1, 1, 1], &[2, 1, 1]];
/// 停机条款 T3：多飞 k = 3 的配置数超过 10⁸ 就停在 k = 2（登记第十一节）。
const CONFIGURATION_LIMIT: u64 = 100_000_000;
/// 一份配置里装得下的发布数：k ≤ 3，加乙整段作废最后一个事务时另开的那次空发布。
const MAXIMUM_PUBLISHES_IN_A_CONFIGURATION: usize = 4;
/// 一份配置里一次发布装得下的记录数：多飞一次发布至多是一个事务的 2 条加前一个作废事务的 2 条占位。
const MAXIMUM_RECORDS_PER_PUBLISH_IN_A_CONFIGURATION: usize = 4;

/// 第二段的历史（登记 5.6 第二段）：k = 2 的 H0、H1 全部；k = 3 只取 r = (1, 1, 1) 与 (2, 1, 1) 的 H0、H1。H2 放在段末、够判后不跑（登记第六节「附带」）。
fn segment_two_histories() -> Vec<History> {
    all_histories()
        .into_iter()
        .filter(|history| history.family().is_legal())
        .filter(|history| history.transaction_count() == 2 || SEGMENT_TWO_THREE_TRANSACTION_RECORD_COUNTS.contains(&history.record_counts.as_slice()))
        .collect()
}

#[derive(Clone, Copy, Debug)]
enum TemplateOperation {
    Fixed(Operation),
    /// 记录的 jsn 在发出时才定（发 jsn 的臂）；发布内序号与末条标志按这次发布的记录次序先算好（`publish_template`）。
    Record { planned: PlannedRecord, ordinal_within_publish: u32, ends_its_publish: bool },
}

/// 一次发布按发布内次序排好的操作（D16（发布语义） 已定项 7 的写序）。
#[derive(Clone, Debug)]
struct PublishTemplate {
    identity: u32,
    txg: u64,
    unit_transactions: Vec<u32>,
    operations: Vec<TemplateOperation>,
    root_slot_position: usize,
    system_configuration_position: usize,
    final_barrier_position: usize,
}

impl PublishTemplate {
    fn own_record_positions_of(&self, transaction: u32) -> Vec<usize> {
        self.operations
            .iter()
            .enumerate()
            .filter(|(_, operation)| match operation {
                TemplateOperation::Record { planned, .. } => planned.transaction == transaction && planned.transaction != 0,
                TemplateOperation::Fixed(_) => false,
            })
            .map(|(operation_position, _)| operation_position)
            .collect()
    }
    fn own_transactions(&self) -> BTreeSet<u32> {
        self.operations
            .iter()
            .filter_map(|operation| match operation {
                TemplateOperation::Record { planned, .. } if planned.transaction != 0 => Some(planned.transaction),
                TemplateOperation::Record { .. } | TemplateOperation::Fixed(_) => None,
            })
            .collect()
    }
    /// 这一位置是这次发布的第几条记录（从 0 起）；不是记录时为 None。
    fn record_index_at(&self, operation_position: usize) -> Option<usize> {
        match self.operations[operation_position] {
            TemplateOperation::Record { .. } => Some(self.operations[..operation_position].iter().filter(|operation| matches!(operation, TemplateOperation::Record { .. })).count()),
            TemplateOperation::Fixed(_) => None,
        }
    }
}

/// 一次发布的操作序列。发布内序号与末条标志照 `place_records_of_publish` 的口径（按 jsn 排名、jsn 最大那条带末条）：
/// 发 jsn 的臂里一次发布的记录按发布内次序先后拿号，名次就是发布内次序；预留的臂按预留号排名。
fn publish_template(publish: &PublishLayout, order: &[usize]) -> PublishTemplate {
    assert!(!publish.is_abandoned, "多飞不跑作废·宽（第二段 H2 够判后未跑）");
    let mut operations: Vec<TemplateOperation> = Vec::new();
    for transaction in &publish.unit_transactions {
        operations.push(TemplateOperation::Fixed(Operation::Write {
            content: WriteContent::TransactionUnit { transaction: *transaction },
            states: WriteDurabilityStates::NewOrMissing,
        }));
    }
    operations.push(TemplateOperation::Fixed(Operation::Write {
        content: WriteContent::SharedUnit { publish: publish.identity },
        states: WriteDurabilityStates::NewOrMissing,
    }));
    // B1：单元与共享单元之后
    operations.push(TemplateOperation::Fixed(Operation::Barrier));
    let mut cursor = vec![0usize; publish.record_streams.len()];
    let planned_in_order: Vec<PlannedRecord> = order
        .iter()
        .map(|stream_index| {
            let planned = publish.record_streams[*stream_index].records[cursor[*stream_index]];
            cursor[*stream_index] += 1;
            planned
        })
        .collect();
    let rank_keys: Vec<u64> = planned_in_order
        .iter()
        .enumerate()
        .map(|(position_in_publish, planned)| planned.reserved_jsn.unwrap_or(u64::try_from(position_in_publish).expect("一次发布至多 4 条记录")))
        .collect();
    let largest_key = rank_keys.iter().copied().max();
    for (position_in_publish, planned) in planned_in_order.iter().enumerate() {
        let key = rank_keys[position_in_publish];
        let ordinal = 1 + rank_keys.iter().filter(|other_key| **other_key < key).count();
        operations.push(TemplateOperation::Record {
            planned: *planned,
            ordinal_within_publish: u32::try_from(ordinal).expect("一次发布至多 4 条记录"),
            ends_its_publish: Some(key) == largest_key && !planned.successor_planned_at_issue,
        });
    }
    // B2：记录之后
    operations.push(TemplateOperation::Fixed(Operation::Barrier));
    let root_slot_position = operations.len();
    let root_slot = WriteContent::RootSlot { publish: publish.identity, txg: publish.txg };
    if ROOT_SLOT_WRITE_CLOSES_ITS_SEGMENT {
        operations.push(TemplateOperation::Fixed(Operation::ForcedUnitAccessWrite { content: root_slot }));
    } else {
        operations.push(TemplateOperation::Fixed(Operation::Write { content: root_slot, states: WriteDurabilityStates::NewOrMissing }));
    }
    let system_configuration_position = operations.len();
    operations.push(TemplateOperation::Fixed(Operation::Write {
        content: WriteContent::SystemConfiguration { publish: publish.identity },
        states: SYSTEM_CONFIGURATION_DURABILITY,
    }));
    let final_barrier_position = operations.len();
    // B3：系统配置轮换之后
    operations.push(TemplateOperation::Fixed(Operation::Barrier));
    PublishTemplate {
        identity: publish.identity,
        txg: publish.txg,
        unit_transactions: publish.unit_transactions.clone(),
        operations,
        root_slot_position,
        system_configuration_position,
        final_barrier_position,
    }
}

/// 每次发布各选一种允许的记录次序，全部组合（各发布的选择号）。
fn every_order_choice(orders: &[Vec<Vec<usize>>]) -> Vec<Vec<usize>> {
    let mut choices: Vec<Vec<usize>> = vec![Vec::new()];
    for publish_orders in orders {
        choices = choices
            .into_iter()
            .flat_map(|prefix| {
                (0..publish_orders.len()).map(move |choice| {
                    let mut extended = prefix.clone();
                    extended.push(choice);
                    extended
                })
            })
            .collect();
    }
    choices
}

fn multi_flight_templates(layout: &CellLayout, orders: &[Vec<Vec<usize>>], choice: &[usize]) -> Vec<PublishTemplate> {
    let templates: Vec<PublishTemplate> = layout
        .publishes
        .iter()
        .enumerate()
        .map(|(publish_position, publish)| publish_template(publish, &orders[publish_position][choice[publish_position]]))
        .collect();
    assert!(templates.windows(2).all(|pair| pair[0].txg < pair[1].txg), "模板按 txg 升序排（根序的「前一次发布」就是前一个模板）");
    templates
}

/// 多飞的发布之间的次序约束（登记 5.1 根序一行；5.2 甲两条臂的跨事务约束在多飞下落在不同发布之间）。
#[derive(Clone, Copy, Debug)]
struct MultiFlightRules {
    roots_in_txg_order: bool,
    append_arm: AppendArm,
    discipline: AppendDiscipline,
}

impl MultiFlightRules {
    fn of(condition: PublishCondition, append_arm: AppendArm, discipline: AppendDiscipline) -> MultiFlightRules {
        let roots_in_txg_order = match condition {
            PublishCondition::MultiFlightRootsOrdered => true,
            PublishCondition::MultiFlightRootsUnordered => false,
            PublishCondition::SerialPublishes | PublishCondition::SharedPublish => panic!("只有多飞两个条件的发布之间任意交错"),
        };
        MultiFlightRules { roots_in_txg_order, append_arm, discipline }
    }
}

/// 第 `publish_position` 次发布的下一个操作，在各发布已发出到 `issued_counts` 时能不能发出。
fn next_operation_may_issue(templates: &[PublishTemplate], issued_counts: &[usize], publish_position: usize, rules: MultiFlightRules) -> bool {
    let template = &templates[publish_position];
    let next_position = issued_counts[publish_position];
    // 多飞·根序：F 与 C 都按 txg 先后发出。
    if rules.roots_in_txg_order && publish_position > 0 {
        let previous = &templates[publish_position - 1];
        let previous_issued = issued_counts[publish_position - 1];
        let waits_for_previous_root_slot = next_position == template.root_slot_position && previous_issued <= previous.root_slot_position;
        let waits_for_previous_system_configuration = next_position == template.system_configuration_position && previous_issued <= previous.system_configuration_position;
        if waits_for_previous_root_slot || waits_for_previous_system_configuration {
            return false;
        }
    }
    let starting_transaction = match template.operations[next_position] {
        TemplateOperation::Record { planned, .. } if planned.transaction != 0 && planned.origin == (RecordOrigin::OwnRecord { index_within_transaction: 1 }) => planned.transaction,
        TemplateOperation::Record { .. } | TemplateOperation::Fixed(_) => return true,
    };
    if rules.discipline == AppendDiscipline::RemovedForPositiveControl {
        return true;
    }
    for (other_position, other) in templates.iter().enumerate().filter(|(other_position, _)| *other_position != publish_position) {
        for earlier_transaction in other.own_transactions().into_iter().filter(|candidate| *candidate < starting_transaction) {
            let positions = other.own_record_positions_of(earlier_transaction);
            let earlier_first = *positions.first().expect("没作废的事务至少一条记录");
            let earlier_last = *positions.last().expect("没作废的事务至少一条记录");
            let earlier_cleared = match rules.append_arm {
                // 甲整段：发出 Ti 的第一条之前，每个 j < i 的 Tj 记录全部已发出
                AppendArm::JiaWhole => issued_counts[other_position] > earlier_last,
                // 甲交错：Ti 的第一条在每个 j < i 的 Tj 的第一条之后发出
                AppendArm::JiaInterleaved => issued_counts[other_position] > earlier_first,
                AppendArm::YiReservedWhole | AppendArm::DingNoDiscipline => true,
            };
            if !earlier_cleared {
                return false;
            }
        }
    }
    true
}

fn every_allowed_merge(templates: &[PublishTemplate], rules: MultiFlightRules) -> Vec<Vec<usize>> {
    let lengths: Vec<usize> = templates.iter().map(|template| template.operations.len()).collect();
    let total: usize = lengths.iter().sum();
    let mut merges = Vec::new();
    let mut issued_counts = vec![0usize; templates.len()];
    let mut current = Vec::with_capacity(total);
    extend_allowed_merges(templates, rules, &lengths, &mut issued_counts, &mut current, total, &mut merges);
    merges
}

fn extend_allowed_merges(templates: &[PublishTemplate], rules: MultiFlightRules, lengths: &[usize], issued_counts: &mut [usize], current: &mut Vec<usize>, total: usize, merges: &mut Vec<Vec<usize>>) {
    if current.len() == total {
        merges.push(current.clone());
        return;
    }
    let mut extended = false;
    for publish_position in 0..templates.len() {
        if issued_counts[publish_position] < lengths[publish_position] && next_operation_may_issue(templates, issued_counts, publish_position, rules) {
            extended = true;
            issued_counts[publish_position] += 1;
            current.push(publish_position);
            extend_allowed_merges(templates, rules, lengths, issued_counts, current, total, merges);
            current.pop();
            issued_counts[publish_position] -= 1;
        }
    }
    assert!(extended, "多飞的次序约束不会卡死：按 txg 先后把各发布做完总走得通");
}

/// 直接枚举的一个执行：先按全局发出次序给 jsn，再在每次发布里按 jsn 排名定发布内序号与末条（与 `place_records_of_publish` 同一口径），最后按全局次序排出操作。
fn execution_from_merge(layout: &CellLayout, templates: &[PublishTemplate], merge: &[usize]) -> Execution {
    let mut cursors = vec![0usize; templates.len()];
    let mut next_issue_jsn = 1u64;
    let mut record_jsns: BTreeMap<(usize, usize), u64> = BTreeMap::new();
    for publish_position in merge {
        let operation_position = cursors[*publish_position];
        cursors[*publish_position] += 1;
        if let TemplateOperation::Record { planned, .. } = templates[*publish_position].operations[operation_position] {
            let jsn = match planned.reserved_jsn {
                Some(reserved) => reserved,
                None => {
                    let issue_jsn = next_issue_jsn;
                    next_issue_jsn += 1;
                    issue_jsn
                }
            };
            record_jsns.insert((*publish_position, operation_position), jsn);
        }
    }
    let mut record_contents: BTreeMap<(usize, usize), RecordContent> = BTreeMap::new();
    for (publish_position, template) in templates.iter().enumerate() {
        let jsns_here: Vec<(usize, u64)> = record_jsns
            .range((publish_position, 0)..(publish_position + 1, 0))
            .map(|((_, operation_position), jsn)| (*operation_position, *jsn))
            .collect();
        let mut jsns_ascending: Vec<u64> = jsns_here.iter().map(|(_, jsn)| *jsn).collect();
        jsns_ascending.sort_unstable();
        let planned_at = |operation_position: usize| match template.operations[operation_position] {
            TemplateOperation::Record { planned, .. } => planned,
            TemplateOperation::Fixed(_) => panic!("这一位置是记录"),
        };
        let end_jsn = jsns_here
            .iter()
            .max_by_key(|(_, jsn)| *jsn)
            .filter(|(operation_position, _)| !planned_at(*operation_position).successor_planned_at_issue)
            .map(|(_, jsn)| *jsn);
        for (operation_position, jsn) in &jsns_here {
            let planned = planned_at(*operation_position);
            let ordinal = 1 + jsns_ascending.iter().filter(|smaller| *smaller < jsn).count();
            record_contents.insert(
                (publish_position, *operation_position),
                RecordContent {
                    jsn: *jsn,
                    txg: template.txg,
                    publish: template.identity,
                    transaction: planned.transaction,
                    is_commit: planned.is_commit,
                    ends_its_publish: Some(*jsn) == end_jsn,
                    ordinal_within_publish: u32::try_from(ordinal).expect("一次发布至多 4 条记录"),
                    origin: planned.origin,
                },
            );
        }
    }
    let mut timelines: Vec<TransactionTimeline> = layout
        .transactions
        .iter()
        .map(|transaction| TransactionTimeline {
            number: transaction.number,
            planned_records: transaction.planned_records,
            issued_record_operation_indices: Vec::new(),
            final_barrier_operation_index: None,
            voided_from: match transaction.voided {
                Some(VoidTiming::NarrowBeforeAnyWrite) => Some(VoidPoint::FromTheStart),
                Some(VoidTiming::WideAfterFirstRecord) => panic!("多飞不跑作废·宽（第二段 H2 够判后未跑）"),
                None => None,
            },
        })
        .collect();
    let mut operations: Vec<Operation> = Vec::with_capacity(merge.len());
    let mut cursors = vec![0usize; templates.len()];
    for publish_position in merge {
        let template = &templates[*publish_position];
        let operation_position = cursors[*publish_position];
        cursors[*publish_position] += 1;
        let operation_index = operations.len();
        match template.operations[operation_position] {
            TemplateOperation::Fixed(operation) => operations.push(operation),
            TemplateOperation::Record { planned, .. } => {
                operations.push(Operation::Write {
                    content: WriteContent::JournalRecord(record_contents[&(*publish_position, operation_position)]),
                    states: WriteDurabilityStates::NewOrMissing,
                });
                if planned.transaction != 0 {
                    timelines[transaction_position(planned.transaction)].issued_record_operation_indices.push(operation_index);
                }
            }
        }
        if operation_position == template.final_barrier_position {
            for transaction in &template.unit_transactions {
                if layout.transactions[transaction_position(*transaction)].voided.is_none() {
                    timelines[transaction_position(*transaction)].final_barrier_operation_index = Some(operation_index);
                }
            }
        }
    }
    Execution { operations, timelines }
}

/// 多飞 k = 2 的直接枚举（登记 5.6 第二段「k = 2 直接枚举执行」）：每种发布内记录次序 × 发布之间全部允许的交错。
fn multi_flight_executions(layout: &CellLayout, rules: MultiFlightRules) -> Vec<Execution> {
    let orders = allowed_record_orders(layout, rules.append_arm, rules.discipline);
    let mut executions = Vec::new();
    for choice in every_order_choice(&orders) {
        let templates = multi_flight_templates(layout, &orders, &choice);
        for merge in every_allowed_merge(&templates, rules) {
            executions.push(execution_from_merge(layout, &templates, &merge));
        }
    }
    executions
}

/// 按配置去重的搜索（登记 5.6 第二段「k = 3 按配置去重」）。一份配置 = 各发布已发出到哪、最后一道完成的闭段点那一刻各发布发出到哪、
/// 有没有一个已发出没完成的闭段点、已发出记录的 jsn。同一份配置往后的崩溃状态与可走的下一步都相同，所以全部执行的崩溃状态并集 = 走到过的每份配置的崩溃状态并集。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct SearchConfiguration {
    issued: [u8; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
    completed: [u8; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
    closer_pending: bool,
    /// 每次发布第几条记录拿到的 jsn（0 = 还没发出）。
    record_jsns: [[u8; MAXIMUM_RECORDS_PER_PUBLISH_IN_A_CONFIGURATION]; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
    next_issue_jsn: u8,
}

#[derive(Clone, Debug)]
struct SearchSummary {
    paths_to_the_end: u64,
    /// 每个事务从这份配置走到底，「前面尚未追加且未作废」的事务数为正的闭段点最多几个。
    positive_closers_to_the_end: Vec<u64>,
}

/// 一个事务在多飞模板里的位置：自己的发布、自己的记录在那次发布里的位置。
#[derive(Clone, Debug)]
struct TransactionPlacement {
    number: u32,
    is_voided: bool,
    publish_position: Option<usize>,
    record_positions: Vec<usize>,
}

struct ConfigurationSearch<'search> {
    templates: &'search [PublishTemplate],
    placements: Vec<TransactionPlacement>,
    rules: MultiFlightRules,
    reading: AppendedReading,
    return_rule: ReturnRule,
    states: HashSet<CrashState>,
    memo: HashMap<SearchConfiguration, SearchSummary>,
    a7_violation_points: u64,
    limit_exceeded: bool,
}

impl<'search> ConfigurationSearch<'search> {
    fn new(layout: &CellLayout, templates: &'search [PublishTemplate], rules: MultiFlightRules, reading: AppendedReading, return_rule: ReturnRule) -> ConfigurationSearch<'search> {
        assert!(templates.len() <= MAXIMUM_PUBLISHES_IN_A_CONFIGURATION, "一份配置装得下的发布数");
        assert!(
            templates.iter().all(|template| template.operations.iter().filter(|operation| matches!(operation, TemplateOperation::Record { .. })).count() <= MAXIMUM_RECORDS_PER_PUBLISH_IN_A_CONFIGURATION),
            "一份配置装得下的记录数"
        );
        let placements = layout
            .transactions
            .iter()
            .map(|transaction| {
                let publish_position = templates.iter().position(|template| template.own_transactions().contains(&transaction.number));
                TransactionPlacement {
                    number: transaction.number,
                    is_voided: match transaction.voided {
                        Some(VoidTiming::NarrowBeforeAnyWrite) => true,
                        Some(VoidTiming::WideAfterFirstRecord) => panic!("多飞不跑作废·宽（第二段 H2 够判后未跑）"),
                        None => false,
                    },
                    publish_position,
                    record_positions: publish_position.map_or(Vec::new(), |position| templates[position].own_record_positions_of(transaction.number)),
                }
            })
            .collect();
        ConfigurationSearch {
            templates,
            placements,
            rules,
            reading,
            return_rule,
            states: HashSet::new(),
            memo: HashMap::new(),
            a7_violation_points: 0,
            limit_exceeded: false,
        }
    }

    fn start() -> SearchConfiguration {
        SearchConfiguration {
            issued: [0; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
            completed: [0; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
            closer_pending: false,
            record_jsns: [[0; MAXIMUM_RECORDS_PER_PUBLISH_IN_A_CONFIGURATION]; MAXIMUM_PUBLISHES_IN_A_CONFIGURATION],
            next_issue_jsn: 1,
        }
    }

    fn finished(&self) -> SearchConfiguration {
        let mut configuration = ConfigurationSearch::start();
        for (publish_position, template) in self.templates.iter().enumerate() {
            let length = u8::try_from(template.operations.len()).expect("一次发布的操作至多十几个");
            configuration.issued[publish_position] = length;
            configuration.completed[publish_position] = length;
        }
        configuration
    }

    fn content_at(&self, configuration: &SearchConfiguration, publish_position: usize, operation_position: usize) -> Option<(WriteContent, WriteDurabilityStates)> {
        let template = &self.templates[publish_position];
        match template.operations[operation_position] {
            TemplateOperation::Fixed(operation) => operation.durability_choices(),
            TemplateOperation::Record { planned, ordinal_within_publish, ends_its_publish } => {
                let record_index = template.record_index_at(operation_position).expect("这一位置是记录");
                let jsn = configuration.record_jsns[publish_position][record_index];
                assert!(jsn > 0, "已发出的记录都拿到了 jsn");
                Some((
                    WriteContent::JournalRecord(RecordContent {
                        jsn: u64::from(jsn),
                        txg: template.txg,
                        publish: template.identity,
                        transaction: planned.transaction,
                        is_commit: planned.is_commit,
                        ends_its_publish,
                        ordinal_within_publish,
                        origin: planned.origin,
                    }),
                    WriteDurabilityStates::NewOrMissing,
                ))
            }
        }
    }

    fn is_appended(&self, placement: &TransactionPlacement, configuration: &SearchConfiguration) -> bool {
        let publish_position = placement.publish_position.expect("没作废的事务有自己的发布");
        let last_record = *placement.record_positions.last().expect("没作废的事务至少一条记录");
        match self.reading {
            AppendedReading::R1Issued => usize::from(configuration.issued[publish_position]) > last_record,
            AppendedReading::R2IssuedAndCovered => usize::from(configuration.completed[publish_position]) > last_record,
        }
    }

    /// 登记第一节 1.2「fsync 返回」在一份配置上：① 本事务所在发布的 B3 已完成；② 每个 j < i 已追加或已作废。
    fn returned(&self, configuration: &SearchConfiguration) -> Vec<(u32, ReturnBranch)> {
        let mut returned = Vec::new();
        for placement in self.placements.iter().filter(|placement| !placement.is_voided) {
            let publish_position = placement.publish_position.expect("没作废的事务有自己的发布");
            let own_condition = match self.return_rule {
                ReturnRule::AfterOwnPublishLastBarrier => usize::from(configuration.completed[publish_position]) > self.templates[publish_position].final_barrier_position,
                ReturnRule::EarlyAfterOwnRecordsIssued => usize::from(configuration.issued[publish_position]) > *placement.record_positions.last().expect("至少一条记录"),
            };
            if !own_condition {
                continue;
            }
            let mut passed_as_voided = false;
            let mut blocked = false;
            for earlier in self.placements.iter().filter(|candidate| candidate.number < placement.number) {
                if earlier.is_voided {
                    passed_as_voided = true;
                    continue;
                }
                if !self.is_appended(earlier, configuration) {
                    blocked = true;
                    break;
                }
            }
            if !blocked {
                returned.push((placement.number, if passed_as_voided { ReturnBranch::SomeEarlierPassedAsVoided } else { ReturnBranch::EveryEarlierAppended }));
            }
        }
        returned
    }

    /// 登记 8.1 第一行：每个没作废的事务前面「尚未追加且未作废」的事务数（作废的事务记 0、不进轨迹）。
    fn pending(&self, configuration: &SearchConfiguration) -> Vec<Option<u64>> {
        self.placements
            .iter()
            .map(|placement| {
                if placement.is_voided {
                    return None;
                }
                let count = self
                    .placements
                    .iter()
                    .filter(|earlier| earlier.number < placement.number && !earlier.is_voided && !self.is_appended(earlier, configuration))
                    .count();
                Some(u64::try_from(count).expect("k ≤ 3"))
            })
            .collect()
    }

    fn insert_states_of(&mut self, configuration: &SearchConfiguration) {
        let mut base: Vec<(WriteContent, DurableForm)> = Vec::new();
        let mut segment_writes: Vec<(WriteContent, WriteDurabilityStates)> = Vec::new();
        for publish_position in 0..self.templates.len() {
            for operation_position in 0..usize::from(configuration.completed[publish_position]) {
                if let Some((content, _)) = self.content_at(configuration, publish_position, operation_position) {
                    base.push((content, DurableForm::New));
                }
            }
            for operation_position in usize::from(configuration.completed[publish_position])..usize::from(configuration.issued[publish_position]) {
                if let Some(choice) = self.content_at(configuration, publish_position, operation_position) {
                    segment_writes.push(choice);
                }
            }
        }
        let returned = self.returned(configuration);
        for (number, _) in &returned {
            let placement = &self.placements[transaction_position(*number)];
            let publish_position = placement.publish_position.expect("返回的事务有自己的发布");
            if usize::from(configuration.completed[publish_position]) <= self.templates[publish_position].final_barrier_position {
                self.a7_violation_points += 1;
            }
        }
        insert_every_durability_choice(&base, &segment_writes, &returned, &mut self.states);
    }

    /// 下一步：有已发出没完成的闭段点时只能等它完成；否则任一发布发出它的下一个操作（照多飞的次序约束）。
    fn successors(&self, configuration: &SearchConfiguration) -> Vec<(SearchConfiguration, bool)> {
        if configuration.closer_pending {
            let mut next = *configuration;
            next.completed = next.issued;
            next.closer_pending = false;
            return vec![(next, true)];
        }
        let issued_counts: Vec<usize> = (0..self.templates.len()).map(|publish_position| usize::from(configuration.issued[publish_position])).collect();
        let mut successors = Vec::new();
        for (publish_position, template) in self.templates.iter().enumerate() {
            let next_position = issued_counts[publish_position];
            if next_position >= template.operations.len() || !next_operation_may_issue(self.templates, &issued_counts, publish_position, self.rules) {
                continue;
            }
            let mut next = *configuration;
            next.issued[publish_position] += 1;
            match template.operations[next_position] {
                TemplateOperation::Record { planned, .. } => {
                    let jsn = match planned.reserved_jsn {
                        Some(reserved) => u8::try_from(reserved).expect("jsn 至多 7"),
                        None => {
                            let issue_jsn = next.next_issue_jsn;
                            next.next_issue_jsn += 1;
                            issue_jsn
                        }
                    };
                    next.record_jsns[publish_position][template.record_index_at(next_position).expect("这一位置是记录")] = jsn;
                }
                TemplateOperation::Fixed(operation) => {
                    next.closer_pending = operation.closes_a_segment();
                }
            }
            successors.push((next, false));
        }
        successors
    }

    fn visit(&mut self, configuration: SearchConfiguration) -> SearchSummary {
        if let Some(summary) = self.memo.get(&configuration) {
            return summary.clone();
        }
        let transaction_count = self.placements.len();
        if u64::try_from(self.memo.len()).expect("配置数有限") >= CONFIGURATION_LIMIT {
            self.limit_exceeded = true;
            return SearchSummary { paths_to_the_end: 0, positive_closers_to_the_end: vec![0; transaction_count] };
        }
        self.insert_states_of(&configuration);
        let pending_here = self.pending(&configuration);
        let successors = self.successors(&configuration);
        let mut summary = SearchSummary { paths_to_the_end: 0, positive_closers_to_the_end: vec![0; transaction_count] };
        if successors.is_empty() {
            let finished = self.finished();
            assert!(
                !configuration.closer_pending && configuration.issued == finished.issued && configuration.completed == finished.completed,
                "走不动的配置只能是全部发出且完成（次序约束不会卡死）"
            );
            summary.paths_to_the_end = 1;
        }
        for (next, completes_a_closer) in successors {
            let pending_next = self.pending(&next);
            for (after, before) in pending_next.iter().zip(&pending_here) {
                assert!(after <= before, "「前面尚未追加且未作废」的事务数沿每一步只减不增");
            }
            let next_summary = self.visit(next);
            summary.paths_to_the_end += next_summary.paths_to_the_end;
            for transaction_index in 0..transaction_count {
                let gained = u64::from(completes_a_closer && pending_next[transaction_index].is_some_and(|value| value > 0));
                summary.positive_closers_to_the_end[transaction_index] = summary.positive_closers_to_the_end[transaction_index].max(next_summary.positive_closers_to_the_end[transaction_index] + gained);
            }
        }
        self.memo.insert(configuration, summary.clone());
        summary
    }
}

/// 一格多飞的崩溃状态集合与它附带的计数。
struct EnumeratedCrashStates {
    states: HashSet<CrashState>,
    executions: u64,
    configurations: Option<u64>,
    a7_violation_points: u64,
    pending: PendingTrajectory,
    configuration_limit_exceeded: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MultiFlightEnumeration {
    DirectExecutions,
    ByConfiguration,
}

impl MultiFlightEnumeration {
    fn label(self) -> &'static str {
        match self {
            MultiFlightEnumeration::DirectExecutions => "direct",
            MultiFlightEnumeration::ByConfiguration => "by_configuration",
        }
    }
    /// 登记 5.6 第二段：k = 2 直接枚举执行，k = 3 按配置去重。
    fn registered_for(history: &History) -> MultiFlightEnumeration {
        match history.transaction_count() {
            2 => MultiFlightEnumeration::DirectExecutions,
            3 => MultiFlightEnumeration::ByConfiguration,
            other => panic!("第二段只跑 k = 2、3，这里是 {other}"),
        }
    }
}

fn enumerate_multi_flight(request: &CellRequest, layout: &CellLayout, enumeration: MultiFlightEnumeration) -> EnumeratedCrashStates {
    assert!(request.barriers == PublishBarriers::AllThree, "多飞不跑拿掉屏障的对照");
    let rules = MultiFlightRules::of(request.condition, request.append_arm, request.discipline);
    match enumeration {
        MultiFlightEnumeration::DirectExecutions => {
            let executions = multi_flight_executions(layout, rules);
            let mut states = HashSet::new();
            let mut a7_violation_points = 0u64;
            let mut pending = PendingTrajectory::default();
            for execution in &executions {
                a7_violation_points += collect_crash_states(execution, request.reading, request.return_rule, &mut states);
                pending.absorb(&pending_trajectory_of(execution, request.reading));
            }
            EnumeratedCrashStates {
                states,
                executions: u64::try_from(executions.len()).expect("执行数有限"),
                configurations: None,
                a7_violation_points,
                pending,
                configuration_limit_exceeded: false,
            }
        }
        MultiFlightEnumeration::ByConfiguration => {
            let orders = allowed_record_orders(layout, request.append_arm, request.discipline);
            let mut enumerated = EnumeratedCrashStates {
                states: HashSet::new(),
                executions: 0,
                configurations: Some(0),
                a7_violation_points: 0,
                pending: PendingTrajectory::default(),
                configuration_limit_exceeded: false,
            };
            for choice in every_order_choice(&orders) {
                let templates = multi_flight_templates(layout, &orders, &choice);
                let mut search = ConfigurationSearch::new(layout, &templates, rules, request.reading, request.return_rule);
                let start = ConfigurationSearch::start();
                let summary = search.visit(start);
                let pending_at_start = search.pending(&start);
                let pending_at_finish = search.pending(&search.finished());
                let mut trajectory = PendingTrajectory::default();
                for (transaction_index, at_start) in pending_at_start.iter().enumerate() {
                    let (Some(initial), Some(final_value)) = (at_start, pending_at_finish[transaction_index]) else {
                        continue;
                    };
                    // 沿每一步只减不增（`visit` 里逐步断言），所以第一次为 0 之后一直是 0；期末不为 0 就是从没清空过，每条路径都算一次。
                    trajectory.absorb(&PendingTrajectory {
                        initial_peak: *initial,
                        peak_after_drain: 0,
                        positive_closers_peak: summary.positive_closers_to_the_end[transaction_index],
                        final_peak: final_value,
                        never_drained: if final_value > 0 { summary.paths_to_the_end } else { 0 },
                    });
                }
                enumerated.states.extend(search.states.drain());
                enumerated.executions += summary.paths_to_the_end;
                enumerated.configurations = enumerated.configurations.map(|count| count + u64::try_from(search.memo.len()).expect("配置数有限"));
                enumerated.a7_violation_points += search.a7_violation_points;
                enumerated.pending.absorb(&trajectory);
                enumerated.configuration_limit_exceeded |= search.limit_exceeded;
            }
            enumerated
        }
    }
}

// ===== 恢复：丙0 逐判转写与三个候选 =====

struct StateView {
    records_by_jsn: Vec<RecordContent>,
    durable_new: BTreeSet<WriteContent>,
    chosen_root: Option<(u32, u64)>,
    returned: Vec<(u32, ReturnBranch)>,
}

impl StateView {
    fn of(state: &CrashState) -> StateView {
        let durable_new: BTreeSet<WriteContent> = state
            .durable
            .iter()
            .filter(|(_, form)| *form == DurableForm::New)
            .map(|(content, _)| *content)
            .collect();
        let mut records_by_jsn: Vec<RecordContent> = durable_new
            .iter()
            .filter_map(|content| match content {
                WriteContent::JournalRecord(record) => Some(*record),
                WriteContent::TransactionUnit { .. } | WriteContent::SharedUnit { .. } | WriteContent::RootSlot { .. } | WriteContent::SystemConfiguration { .. } => None,
            })
            .collect();
        records_by_jsn.sort_by_key(|record| record.jsn);
        // 择根：持久的根槽里 txg 最大的那个（登记 5.3 ①）。
        let chosen_root = durable_new
            .iter()
            .filter_map(|content| match content {
                WriteContent::RootSlot { publish, txg } => Some((*publish, *txg)),
                WriteContent::TransactionUnit { .. } | WriteContent::SharedUnit { .. } | WriteContent::JournalRecord(_) | WriteContent::SystemConfiguration { .. } => None,
            })
            .max_by_key(|(_, txg)| *txg);
        StateView { records_by_jsn, durable_new, chosen_root, returned: state.returned.clone() }
    }
    fn chosen_root_txg(&self) -> u64 {
        self.chosen_root.map_or(START_ROOT_TXG, |(_, txg)| txg)
    }
}

/// 点名单元验得过 = 那些写持久且为「新」态（登记 5.3：提交记录点名它的 U，末条另点名本发布的 S，占位记录不点名 U）。
fn named_units_are_verified(record: &RecordContent, durable_new: &BTreeSet<WriteContent>) -> bool {
    let names_its_unit = record.transaction != 0 && record.is_commit;
    let unit_verified = !names_its_unit || durable_new.contains(&WriteContent::TransactionUnit { transaction: record.transaction });
    let shared_verified = !record.ends_its_publish || durable_new.contains(&WriteContent::SharedUnit { publish: record.publish });
    unit_verified && shared_verified
}

/// `lib.rs:452` 反向链：单实例、没有残留记录，恒不断（登记第十一节未建模）。
fn back_chain_is_broken(_record: &RecordContent) -> bool {
    false
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AnchorLookup {
    Anchored(u64),
    NoAnchor,
    MoreThanOneLastRecordFlag,
}

/// `lib.rs:190` counter_of_the_last_record_the_root_covers：所选根那次发布里带末条标志的那一条。
fn last_record_the_root_covers(records_by_jsn: &[RecordContent], root_txg: u64) -> AnchorLookup {
    let mut found: Option<u64> = None;
    for record in records_by_jsn {
        if record.txg == root_txg && record.ends_its_publish {
            if found.is_some() {
                return AnchorLookup::MoreThanOneLastRecordFlag;
            }
            found = Some(record.jsn);
        }
    }
    found.map_or(AnchorLookup::NoAnchor, AnchorLookup::Anchored)
}

/// `lib.rs:150` readable_record_of_the_same_publish_follows：这一条之后同一 txg 里还有没有读得出的记录。
fn readable_record_of_the_same_publish_follows(records_by_jsn: &[RecordContent], index: usize) -> bool {
    let record = records_by_jsn[index];
    for later in &records_by_jsn[index + 1..] {
        // lib.rs:173
        if later.txg > record.txg {
            return false;
        }
        // lib.rs:177
        if later.txg == record.txg {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Application {
    /// 施加单位是一次发布（D16（发布语义） 已定项 4）。
    WholePublish,
    /// 段里的记录逐条施加。
    RecordByRecord,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PrefixRules {
    check_commit_before_transaction_change: bool,
    check_commit_on_last_record: bool,
    application: Application,
    stop_before_jsn: Option<u64>,
}

impl PrefixRules {
    const TODAY: PrefixRules = PrefixRules {
        check_commit_before_transaction_change: true,
        check_commit_on_last_record: true,
        application: Application::WholePublish,
        stop_before_jsn: None,
    };
}

#[derive(Clone, Debug, Default)]
struct PrefixWalk {
    accepted: Vec<RecordContent>,
    applied: Vec<RecordContent>,
    in_flight_limit_reached: bool,
    core_failure: bool,
}

/// `crates/singlefs-replay-prefix/src/lib.rs` 第 422–534 行 `replay_core` 主循环逐判转写；
/// `rules` 只为丙1、丙2 与阳性对照关掉写明的那几判或换施加单位，默认（`PrefixRules::TODAY`）就是丙0。
fn walk_prefix(records_by_jsn: &[RecordContent], root_txg: u64, named_units_verified: &dyn Fn(&RecordContent) -> bool, rules: PrefixRules) -> PrefixWalk {
    let mut walk = PrefixWalk::default();
    // lib.rs:335 链首锚点
    let anchor = match last_record_the_root_covers(records_by_jsn, root_txg) {
        AnchorLookup::Anchored(anchor_jsn) => Some(anchor_jsn),
        AnchorLookup::NoAnchor => None,
        AnchorLookup::MoreThanOneLastRecordFlag => {
            walk.core_failure = true;
            return walk;
        }
    };
    let mut expected_next_jsn: Option<u64> = anchor.map(|anchor_jsn| anchor_jsn + 1);
    let chain_start_txg_without_anchor = root_txg + 1;
    let mut open_publish: Vec<RecordContent> = Vec::new();
    let mut taken: usize = 0;
    for (index, record) in records_by_jsn.iter().enumerate() {
        // lib.rs:422 不在水位之上跳过
        if record.txg <= root_txg {
            continue;
        }
        // lib.rs:427 取满在飞上限停
        if taken >= IN_FLIGHT_RECORD_LIMIT {
            walk.in_flight_limit_reached = true;
            break;
        }
        taken += 1;
        // 丙1：段截到 X 的第一条之前
        if rules.stop_before_jsn.is_some_and(|cut_jsn| record.jsn >= cut_jsn) {
            break;
        }
        match expected_next_jsn {
            // lib.rs:434 jsn 不等于期望停
            Some(expected_jsn) => {
                if record.jsn != expected_jsn {
                    break;
                }
            }
            // lib.rs:445 无锚点时链首要 txg = 根 + 1 且序号 1
            None => {
                if record.txg != chain_start_txg_without_anchor || record.ordinal_within_publish != 1 {
                    break;
                }
            }
        }
        // lib.rs:452 反向链断停
        if back_chain_is_broken(record) {
            break;
        }
        // lib.rs:456 攒着的为空而序号不是 1 停
        if open_publish.is_empty() && record.ordinal_within_publish != 1 {
            break;
        }
        if let Some(previous_record) = open_publish.last() {
            // lib.rs:463 攒着的发布里换了 txg 停
            if record.txg != previous_record.txg {
                break;
            }
            // lib.rs:468 发布内序号跳号停
            if record.ordinal_within_publish != previous_record.ordinal_within_publish + 1 {
                break;
            }
            // lib.rs:472 上一条没带提交标记而这一条换了事务号停
            if rules.check_commit_before_transaction_change && !previous_record.is_commit && record.transaction != previous_record.transaction {
                break;
            }
        }
        // lib.rs:478
        expected_next_jsn = Some(record.jsn + 1);
        // lib.rs:480 末条不带提交标记停
        if rules.check_commit_on_last_record && !record.is_commit && record.ends_its_publish {
            break;
        }
        // lib.rs:484 末条之后同发布还有可读记录停
        if record.ends_its_publish && readable_record_of_the_same_publish_follows(records_by_jsn, index) {
            break;
        }
        // lib.rs:493 点名单元验不过停
        if !named_units_verified(record) {
            break;
        }
        open_publish.push(*record);
        walk.accepted.push(*record);
        match rules.application {
            Application::RecordByRecord => {
                walk.applied.push(*record);
                if record.ends_its_publish {
                    open_publish.clear();
                }
            }
            Application::WholePublish => {
                // lib.rs:529 末条没到就接着往下走
                if !record.ends_its_publish {
                    continue;
                }
                // lib.rs:534 走到末条才把整次发布计入已施加
                walk.applied.append(&mut open_publish);
            }
        }
    }
    walk
}

/// 登记第一节 1.2「不合法前缀」的严格连续段：从链首起、jsn 逐一连续、全部持久、在水位以上的最长一段。
/// 链首照 5.3 ③：有锚点时是锚点 jsn + 1；没有锚点时是水位之上第一条、它要 txg = 根 + 1 且序号 1（修订第 2 条）。
fn strict_continuous_segment(records_by_jsn: &[RecordContent], root_txg: u64) -> Vec<RecordContent> {
    let anchor = match last_record_the_root_covers(records_by_jsn, root_txg) {
        AnchorLookup::Anchored(anchor_jsn) => Some(anchor_jsn),
        AnchorLookup::NoAnchor => None,
        AnchorLookup::MoreThanOneLastRecordFlag => return Vec::new(),
    };
    let mut segment: Vec<RecordContent> = Vec::new();
    for record in records_by_jsn.iter().filter(|candidate| candidate.txg > root_txg) {
        let continues = match segment.last() {
            None => match anchor {
                Some(anchor_jsn) => record.jsn == anchor_jsn + 1,
                None => record.txg == root_txg + 1 && record.ordinal_within_publish == 1,
            },
            Some(last_in_segment) => record.jsn == last_in_segment.jsn + 1,
        };
        if !continues {
            break;
        }
        segment.push(*record);
    }
    segment
}

#[derive(Clone, Debug)]
struct TransactionInSegment {
    first_jsn: u64,
    has_commit: bool,
    jsns: Vec<u64>,
}

/// 段里各事务（占位记录的事务号 0 不算，D23（journal 的角色与格式） 已定项 19 ①）。
fn transactions_in(segment: &[RecordContent]) -> BTreeMap<u32, TransactionInSegment> {
    let mut transactions: BTreeMap<u32, TransactionInSegment> = BTreeMap::new();
    for record in segment.iter().filter(|candidate| candidate.transaction != 0) {
        let entry = transactions
            .entry(record.transaction)
            .or_insert(TransactionInSegment { first_jsn: record.jsn, has_commit: false, jsns: Vec::new() });
        entry.first_jsn = entry.first_jsn.min(record.jsn);
        entry.has_commit |= record.is_commit;
        entry.jsns.push(record.jsn);
    }
    transactions
}

/// N2a 的谓词：段里有两个或更多事务各有记录、而各自的提交记录不在段里。
fn two_or_more_incomplete_transactions_in(segment: &[RecordContent]) -> bool {
    transactions_in(segment).values().filter(|transaction| !transaction.has_commit).count() >= 2
}

/// 模式 P：段里某个事务 X 的提交记录不在段里，而段里有另一个事务 Y 的提交记录、且 Y 有一条记录的 jsn 大于 X 在段里的第一条。
fn incomplete_a_followed_by_complete_b_in(segment: &[RecordContent]) -> bool {
    let transactions = transactions_in(segment);
    for (incomplete_number, incomplete) in transactions.iter().filter(|(_, candidate)| !candidate.has_commit) {
        for (complete_number, complete) in transactions.iter().filter(|(_, candidate)| candidate.has_commit) {
            if complete_number != incomplete_number && complete.jsns.iter().any(|jsn| *jsn > incomplete.first_jsn) {
                return true;
            }
        }
    }
    false
}

/// 丙1 的 X：严格连续段里 jsn 最小的「提交记录不在段里」的事务，返回它在段里第一条的 jsn。
fn first_incomplete_transaction_first_jsn(segment: &[RecordContent]) -> Option<u64> {
    transactions_in(segment).values().filter(|transaction| !transaction.has_commit).map(|transaction| transaction.first_jsn).min()
}

/// 丙2：拿掉「提交记录不在段里」的每个事务的全部记录（占位记录留着）。
fn drop_transactions_without_commit_in(accepted: &[RecordContent]) -> Vec<RecordContent> {
    let committed: BTreeSet<u32> = accepted.iter().filter(|record| record.transaction != 0 && record.is_commit).map(|record| record.transaction).collect();
    accepted.iter().filter(|record| record.transaction == 0 || committed.contains(&record.transaction)).copied().collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecoveryOutcome {
    mounted: bool,
    chosen_root_txg: u64,
    applied: Vec<RecordContent>,
    /// 装上的根：所选根，或施加的记录里 jsn 最大那条带的整次发布新根段（`crates/singlefs-core/src/transaction.rs:6728`）。
    installed_root_publish: Option<u32>,
    installed_root_txg: u64,
    in_flight_limit_reached: bool,
    core_failure: bool,
}

impl RecoveryOutcome {
    fn mounted_with(view: &StateView, applied: Vec<RecordContent>, walk_flags: (bool, bool)) -> RecoveryOutcome {
        let (in_flight_limit_reached, core_failure) = walk_flags;
        let (installed_root_publish, installed_root_txg) = match applied.last() {
            Some(last_applied) => (Some(last_applied.publish), last_applied.txg),
            None => (view.chosen_root.map(|(publish, _)| publish), view.chosen_root_txg()),
        };
        RecoveryOutcome {
            mounted: !core_failure,
            chosen_root_txg: view.chosen_root_txg(),
            applied,
            installed_root_publish,
            installed_root_txg,
            in_flight_limit_reached,
            core_failure,
        }
    }
    fn refused(view: &StateView) -> RecoveryOutcome {
        RecoveryOutcome {
            mounted: false,
            chosen_root_txg: view.chosen_root_txg(),
            applied: Vec::new(),
            installed_root_publish: None,
            installed_root_txg: START_ROOT_TXG,
            in_flight_limit_reached: false,
            core_failure: false,
        }
    }
    fn version_key(&self) -> (bool, Option<u32>, Vec<RecordContent>) {
        (self.mounted, self.installed_root_publish, self.applied.clone())
    }
}

const RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS: PrefixRules = PrefixRules {
    check_commit_before_transaction_change: false,
    check_commit_on_last_record: false,
    application: Application::RecordByRecord,
    stop_before_jsn: None,
};

fn recover(view: &StateView, arm: RecoveryArm, control: RecoveryControl) -> RecoveryOutcome {
    let root_txg = view.chosen_root_txg();
    let durable_new = &view.durable_new;
    let verify = |record: &RecordContent| named_units_are_verified(record, durable_new);
    let strict_segment = strict_continuous_segment(&view.records_by_jsn, root_txg);
    let mounted_after = |rules: PrefixRules| -> RecoveryOutcome {
        let walk = walk_prefix(&view.records_by_jsn, root_txg, &verify, rules);
        RecoveryOutcome::mounted_with(view, walk.applied, (walk.in_flight_limit_reached, walk.core_failure))
    };
    match arm {
        RecoveryArm::Bing0Today => match control {
            RecoveryControl::NoControl => mounted_after(PrefixRules::TODAY),
            RecoveryControl::KeepLaterRecordsRemoved | RecoveryControl::CommitMarkerHandlingRemoved => mounted_after(RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS),
        },
        RecoveryArm::Bing1TruncateBeforeA => match control {
            RecoveryControl::NoControl => {
                let cut_before_a = first_incomplete_transaction_first_jsn(&strict_segment);
                mounted_after(PrefixRules {
                    check_commit_before_transaction_change: false,
                    check_commit_on_last_record: false,
                    application: Application::WholePublish,
                    stop_before_jsn: cut_before_a,
                })
            }
            RecoveryControl::KeepLaterRecordsRemoved | RecoveryControl::CommitMarkerHandlingRemoved => mounted_after(RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS),
        },
        RecoveryArm::Bing2DropAKeepB => {
            let walk = walk_prefix(&view.records_by_jsn, root_txg, &verify, RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS);
            let applied = match control {
                RecoveryControl::NoControl | RecoveryControl::KeepLaterRecordsRemoved => drop_transactions_without_commit_in(&walk.accepted),
                RecoveryControl::CommitMarkerHandlingRemoved => walk.accepted.clone(),
            };
            RecoveryOutcome::mounted_with(view, applied, (walk.in_flight_limit_reached, walk.core_failure))
        }
        RecoveryArm::Bing3ForbidPattern => match control {
            RecoveryControl::NoControl => {
                let judged_corrupt = incomplete_a_followed_by_complete_b_in(&strict_segment);
                if judged_corrupt {
                    RecoveryOutcome::refused(view)
                } else {
                    mounted_after(PrefixRules::TODAY)
                }
            }
            RecoveryControl::KeepLaterRecordsRemoved => mounted_after(PrefixRules {
                check_commit_before_transaction_change: true,
                check_commit_on_last_record: true,
                application: Application::RecordByRecord,
                stop_before_jsn: None,
            }),
            RecoveryControl::CommitMarkerHandlingRemoved => mounted_after(RECORD_BY_RECORD_WITHOUT_COMMIT_CHECKS),
        },
    }
}

// ===== 每个状态上的量（登记第六节） =====

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RecoveryCounts {
    refused: u64,
    v_fsync: u64,
    v_fsync_mounted_lost: u64,
    v_fsync_refused_with_returned: u64,
    v_fsync_branch_every_earlier_appended: u64,
    v_fsync_branch_earlier_passed_as_voided: u64,
    v_dependency: u64,
    v_dangling_returned_empty: u64,
    v_dangling_returned_nonempty: u64,
    partial_publish: u64,
    n_atom: u64,
    core_failures: u64,
    differs_from_bing0: u64,
}

impl RecoveryCounts {
    fn absorb(&mut self, other: &RecoveryCounts) {
        self.refused += other.refused;
        self.v_fsync += other.v_fsync;
        self.v_fsync_mounted_lost += other.v_fsync_mounted_lost;
        self.v_fsync_refused_with_returned += other.v_fsync_refused_with_returned;
        self.v_fsync_branch_every_earlier_appended += other.v_fsync_branch_every_earlier_appended;
        self.v_fsync_branch_earlier_passed_as_voided += other.v_fsync_branch_earlier_passed_as_voided;
        self.v_dependency += other.v_dependency;
        self.v_dangling_returned_empty += other.v_dangling_returned_empty;
        self.v_dangling_returned_nonempty += other.v_dangling_returned_nonempty;
        self.partial_publish += other.partial_publish;
        self.n_atom += other.n_atom;
        self.core_failures += other.core_failures;
        self.differs_from_bing0 += other.differs_from_bing0;
    }
    fn v_dangling(&self) -> u64 {
        self.v_dangling_returned_empty + self.v_dangling_returned_nonempty
    }
}

fn metrics_of(view: &StateView, layout: &CellLayout, outcome: &RecoveryOutcome) -> RecoveryCounts {
    let mut counts = RecoveryCounts::default();
    let mut applied_per_transaction: BTreeMap<u32, u32> = BTreeMap::new();
    let mut applied_per_publish: BTreeMap<u32, usize> = BTreeMap::new();
    for record in &outcome.applied {
        if record.transaction != 0 {
            *applied_per_transaction.entry(record.transaction).or_insert(0) += 1;
        }
        *applied_per_publish.entry(record.publish).or_insert(0) += 1;
    }
    let applied_of = |transaction: u32| applied_per_transaction.get(&transaction).copied().unwrap_or(0);
    let transaction_layout = |transaction: u32| &layout.transactions[transaction_position(transaction)];
    // 登记第六节 Q3-1：T 在恢复出的版本里 ⇔ T 所在发布的 txg 不大于所选根的 txg，或 T 的全部记录被施加。
    let in_recovered_version = |transaction: u32| {
        let layout_of_transaction = transaction_layout(transaction);
        layout_of_transaction.txg.is_some_and(|txg| txg <= outcome.chosen_root_txg) || applied_of(transaction) == layout_of_transaction.planned_records
    };
    if outcome.core_failure {
        counts.core_failures = 1;
    }
    if outcome.mounted {
        let lost: Vec<&(u32, ReturnBranch)> = view.returned.iter().filter(|(transaction, _)| !in_recovered_version(*transaction)).collect();
        if !lost.is_empty() {
            counts.v_fsync = 1;
            counts.v_fsync_mounted_lost = 1;
            counts.v_fsync_branch_every_earlier_appended = u64::from(lost.iter().any(|(_, branch)| *branch == ReturnBranch::EveryEarlierAppended));
            counts.v_fsync_branch_earlier_passed_as_voided = u64::from(lost.iter().any(|(_, branch)| *branch == ReturnBranch::SomeEarlierPassedAsVoided));
        }
        // Q3-2：施加了某事务 Y 的记录，而某个没作废、号小于 Y、txg 大于所选根的事务 X 的记录没全部施加。
        let dependency_violated = applied_per_transaction.keys().any(|later| {
            layout.transactions.iter().any(|earlier| {
                earlier.voided.is_none()
                    && earlier.number < *later
                    && earlier.txg.is_some_and(|txg| txg > outcome.chosen_root_txg)
                    && applied_of(earlier.number) < earlier.planned_records
            })
        });
        counts.v_dependency = u64::from(dependency_violated);
        // Q3-3：装上的根引用的写里有不持久或撕裂的。
        let unit_missing = layout.transactions.iter().any(|transaction| {
            transaction.voided.is_none()
                && transaction.txg.is_some_and(|txg| txg <= outcome.installed_root_txg)
                && !view.durable_new.contains(&WriteContent::TransactionUnit { transaction: transaction.number })
        });
        let shared_missing = outcome
            .installed_root_publish
            .is_some_and(|publish| !view.durable_new.contains(&WriteContent::SharedUnit { publish }));
        if unit_missing || shared_missing {
            if view.returned.is_empty() {
                counts.v_dangling_returned_empty = 1;
            } else {
                counts.v_dangling_returned_nonempty = 1;
            }
        }
        // Q3-4：一次发布只施加了一部分记录。
        counts.partial_publish = u64::from(
            applied_per_publish
                .iter()
                .any(|(publish, applied_count)| *applied_count > 0 && *applied_count < layout.written_record_count_of(*publish)),
        );
        // 第七节 A3（C37（事务与记录的对应无定义） 层 0 那一半）：某个事务的记录被施加了一部分。
        counts.n_atom = u64::from(
            applied_per_transaction
                .iter()
                .any(|(transaction, applied_count)| *applied_count > 0 && *applied_count < transaction_layout(*transaction).planned_records),
        );
    } else {
        counts.refused = u64::from(!outcome.core_failure);
        if !view.returned.is_empty() {
            counts.v_fsync = 1;
            counts.v_fsync_refused_with_returned = 1;
            counts.v_fsync_branch_every_earlier_appended = u64::from(view.returned.iter().any(|(_, branch)| *branch == ReturnBranch::EveryEarlierAppended));
            counts.v_fsync_branch_earlier_passed_as_voided = u64::from(view.returned.iter().any(|(_, branch)| *branch == ReturnBranch::SomeEarlierPassedAsVoided));
        }
    }
    counts
}

// ===== 被谓词消费的量的轨迹（登记 8.1 第一行，按闭段点记） =====

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PendingTrajectory {
    initial_peak: u64,
    peak_after_drain: u64,
    positive_closers_peak: u64,
    final_peak: u64,
    never_drained: u64,
}

impl PendingTrajectory {
    fn absorb(&mut self, other: &PendingTrajectory) {
        self.initial_peak = self.initial_peak.max(other.initial_peak);
        self.peak_after_drain = self.peak_after_drain.max(other.peak_after_drain);
        self.positive_closers_peak = self.positive_closers_peak.max(other.positive_closers_peak);
        self.final_peak = self.final_peak.max(other.final_peak);
        self.never_drained += other.never_drained;
    }
}

fn pending_trajectory_of(execution: &Execution, reading: AppendedReading) -> PendingTrajectory {
    let closers: Vec<usize> = execution
        .operations
        .iter()
        .enumerate()
        .filter(|(_, operation)| operation.closes_a_segment())
        .map(|(operation_index, _)| operation_index)
        .collect();
    let mut trajectory = PendingTrajectory::default();
    for timeline in execution.timelines.iter().filter(|candidate| candidate.voided_from.is_none()) {
        let pending_at = |completed: Option<usize>, issued: Option<usize>| -> u64 {
            let pending = execution
                .timelines
                .iter()
                .filter(|earlier| earlier.number < timeline.number && !is_appended(earlier, completed, issued, reading) && !is_voided_by(earlier, issued))
                .count();
            u64::try_from(pending).expect("k ≤ 3")
        };
        let initial = pending_at(None, None);
        let values: Vec<u64> = closers.iter().map(|closer| pending_at(Some(*closer), Some(*closer))).collect();
        let positive_closers = u64::try_from(values.iter().filter(|value| **value > 0).count()).expect("闭段点数有限");
        let final_value = values.last().copied().unwrap_or(initial);
        let mut this_transaction = PendingTrajectory {
            initial_peak: initial,
            peak_after_drain: 0,
            positive_closers_peak: positive_closers,
            final_peak: final_value,
            never_drained: 0,
        };
        match values.iter().position(|value| *value == 0) {
            Some(drained_at) => this_transaction.peak_after_drain = values[drained_at..].iter().copied().max().unwrap_or(0),
            None => this_transaction.never_drained = 1,
        }
        trajectory.absorb(&this_transaction);
    }
    trajectory
}

// ===== 一格 =====

#[derive(Clone, Debug)]
struct CellRequest {
    condition: PublishCondition,
    append_arm: AppendArm,
    history: History,
    reading: AppendedReading,
    return_rule: ReturnRule,
    discipline: AppendDiscipline,
    barriers: PublishBarriers,
    recovery_control: RecoveryControl,
}

impl CellRequest {
    fn main_grid(condition: PublishCondition, append_arm: AppendArm, history: History, reading: AppendedReading) -> CellRequest {
        CellRequest {
            condition,
            append_arm,
            history,
            reading,
            return_rule: ReturnRule::AfterOwnPublishLastBarrier,
            discipline: AppendDiscipline::Kept,
            barriers: PublishBarriers::AllThree,
            recovery_control: RecoveryControl::NoControl,
        }
    }
}

#[derive(Clone, Debug)]
struct AppendCellResult {
    executions: u64,
    states: u64,
    incomplete_pair_states: u64,
    pattern_p_states: u64,
    strict_segment_length_peak: usize,
    in_flight_limit_states: u64,
    a7_violation_points: u64,
    pending: PendingTrajectory,
    a5_differing_states: u64,
    recovery: [RecoveryCounts; 4],
    /// 第二段：这一格怎么枚举的（第一段为 None）。
    multi_flight_enumeration: Option<MultiFlightEnumeration>,
    /// 按配置去重时走到过的配置数（停机条款 T3）。
    configurations: Option<u64>,
    configuration_limit_exceeded: bool,
}

/// 甲整段下一个事务的记录连着发出，中间不能作废：H2 对它不适用（登记 5.4）。
fn cell_is_applicable(append_arm: AppendArm, history: &History) -> bool {
    !(append_arm == AppendArm::JiaWhole && history.family() == HistoryFamily::H2)
}

fn evaluate_cell(request: &CellRequest) -> Option<AppendCellResult> {
    if !cell_is_applicable(request.append_arm, &request.history) {
        return None;
    }
    let layout = plan_cell(request.condition, request.append_arm, &request.history, request.discipline);
    if request.condition.publishes_fly_together() {
        let enumeration = MultiFlightEnumeration::registered_for(&request.history);
        return Some(multi_flight_cell_result(request, &layout, enumeration).1);
    }
    let executions = build_executions(&layout, request.append_arm, request.discipline, request.barriers);
    let mut states: HashSet<CrashState> = HashSet::new();
    let mut a7_violation_points = 0u64;
    let mut pending = PendingTrajectory::default();
    for execution in &executions {
        a7_violation_points += collect_crash_states(execution, request.reading, request.return_rule, &mut states);
        pending.absorb(&pending_trajectory_of(execution, request.reading));
    }
    let enumerated = EnumeratedCrashStates {
        states,
        executions: u64::try_from(executions.len()).expect("执行数有限"),
        configurations: None,
        a7_violation_points,
        pending,
        configuration_limit_exceeded: false,
    };
    Some(cell_result_from_states(&layout, &enumerated, request.recovery_control, None))
}

/// 一格多飞：按给定的枚举法得到崩溃状态，连同算出的格结果一起交回（路径对拍要比两种枚举法的状态集合）。
fn multi_flight_cell_result(request: &CellRequest, layout: &CellLayout, enumeration: MultiFlightEnumeration) -> (EnumeratedCrashStates, AppendCellResult) {
    assert!(request.history.family().is_legal(), "第二段 H2 够判后未跑");
    let enumerated = enumerate_multi_flight(request, layout, enumeration);
    let result = cell_result_from_states(layout, &enumerated, request.recovery_control, Some(enumeration));
    (enumerated, result)
}

/// 一格的全部崩溃状态逐个过四条 Q3 臂，数登记第六节的各量。
fn cell_result_from_states(layout: &CellLayout, enumerated: &EnumeratedCrashStates, recovery_control: RecoveryControl, multi_flight_enumeration: Option<MultiFlightEnumeration>) -> AppendCellResult {
    let states = &enumerated.states;
    let mut result = AppendCellResult {
        executions: enumerated.executions,
        states: u64::try_from(states.len()).expect("状态数有限"),
        incomplete_pair_states: 0,
        pattern_p_states: 0,
        strict_segment_length_peak: 0,
        in_flight_limit_states: 0,
        a7_violation_points: enumerated.a7_violation_points,
        pending: enumerated.pending,
        a5_differing_states: 0,
        recovery: Default::default(),
        multi_flight_enumeration,
        configurations: enumerated.configurations,
        configuration_limit_exceeded: enumerated.configuration_limit_exceeded,
    };
    for state in states {
        let view = StateView::of(state);
        let strict_segment = strict_continuous_segment(&view.records_by_jsn, view.chosen_root_txg());
        result.incomplete_pair_states += u64::from(two_or_more_incomplete_transactions_in(&strict_segment));
        result.pattern_p_states += u64::from(incomplete_a_followed_by_complete_b_in(&strict_segment));
        result.strict_segment_length_peak = result.strict_segment_length_peak.max(strict_segment.len());
        let outcomes: [RecoveryOutcome; 4] = RecoveryArm::ALL.map(|arm| recover(&view, arm, recovery_control));
        result.in_flight_limit_states += u64::from(outcomes.iter().any(|outcome| outcome.in_flight_limit_reached));
        let bing0_version = outcomes[RecoveryArm::Bing0Today.position()].version_key();
        result.a5_differing_states += u64::from(outcomes.iter().any(|outcome| outcome.version_key() != bing0_version));
        for arm in RecoveryArm::ALL {
            let outcome = &outcomes[arm.position()];
            let mut counts = metrics_of(&view, layout, outcome);
            counts.differs_from_bing0 = u64::from(outcome.version_key() != bing0_version);
            result.recovery[arm.position()].absorb(&counts);
        }
    }
    result
}

// ===== 停机条款 T1：十四条固定记录序列（登记第七节 A2） =====

struct FixedSequence {
    number: u32,
    root_txg: u64,
    /// (jsn, txg, 事务号, 提交标记, 末条标志, 本次发布内序号)
    records: Vec<(u64, u64, u32, bool, bool, u32)>,
    unverified_jsn: Option<u64>,
    expected_publishes_applied: u64,
    expected_records_applied: u64,
    deciding_line: &'static str,
}

fn fixed_sequences() -> Vec<FixedSequence> {
    let sequence = |number: u32, root_txg: u64, records: Vec<(u64, u64, u32, bool, bool, u32)>, unverified_jsn: Option<u64>, expected_publishes_applied: u64, expected_records_applied: u64, deciding_line: &'static str| FixedSequence {
        number,
        root_txg,
        records,
        unverified_jsn,
        expected_publishes_applied,
        expected_records_applied,
        deciding_line,
    };
    vec![
        sequence(1, 0, vec![(1, 1, 1, true, true, 1)], None, 1, 1, "529-534"),
        sequence(2, 0, vec![(1, 1, 1, false, false, 1), (2, 1, 1, true, true, 2)], None, 1, 2, "529-534"),
        sequence(3, 0, vec![(1, 1, 1, false, false, 1)], None, 0, 0, "529"),
        sequence(4, 0, vec![(1, 1, 1, false, false, 1), (2, 1, 2, true, false, 2), (3, 1, 1, true, true, 3)], None, 0, 0, "472"),
        sequence(5, 0, vec![(1, 1, 1, true, false, 1), (2, 1, 2, true, true, 2)], None, 1, 2, "529-534"),
        sequence(6, 0, vec![(1, 1, 1, true, false, 1), (2, 2, 2, true, true, 2)], None, 0, 0, "463"),
        sequence(7, 0, vec![(1, 1, 1, true, true, 1), (2, 2, 2, true, true, 1)], None, 2, 2, "529-534"),
        sequence(8, 0, vec![(1, 1, 1, true, true, 1), (3, 2, 2, true, true, 1)], None, 1, 1, "434"),
        sequence(9, 0, vec![(1, 1, 1, true, true, 1), (2, 1, 1, true, false, 2)], None, 0, 0, "484"),
        sequence(10, 0, vec![(1, 1, 1, true, true, 1)], Some(1), 0, 0, "493"),
        sequence(11, 0, vec![(1, 1, 1, false, true, 1)], None, 0, 0, "480"),
        sequence(12, 0, vec![(1, 2, 1, true, true, 1)], None, 0, 0, "445"),
        sequence(13, 1, vec![(1, 1, 1, true, true, 1), (2, 2, 2, true, true, 2)], None, 0, 0, "456"),
        sequence(14, 0, vec![(1, 1, 1, true, false, 1), (2, 1, 2, true, true, 3)], None, 0, 0, "468"),
    ]
}

/// 丙0 在一条固定序列上施加的（发布数，记录数）。点名单元除 `unverified_jsn` 那一条之外都验得过。
fn bing0_on_fixed_sequence(sequence: &FixedSequence) -> (u64, u64) {
    let mut records: Vec<RecordContent> = sequence
        .records
        .iter()
        .map(|(jsn, txg, transaction, is_commit, ends_its_publish, ordinal)| RecordContent {
            jsn: *jsn,
            txg: *txg,
            publish: u32::try_from(*txg).expect("txg 小"),
            transaction: *transaction,
            is_commit: *is_commit,
            ends_its_publish: *ends_its_publish,
            ordinal_within_publish: *ordinal,
            origin: RecordOrigin::OwnRecord { index_within_transaction: *ordinal },
        })
        .collect();
    records.sort_by_key(|record| record.jsn);
    let unverified_jsn = sequence.unverified_jsn;
    let verify = |record: &RecordContent| unverified_jsn != Some(record.jsn);
    let walk = walk_prefix(&records, sequence.root_txg, &verify, PrefixRules::TODAY);
    let publishes = walk.applied.iter().filter(|record| record.ends_its_publish).count();
    (u64::try_from(publishes).expect("至多 2"), u64::try_from(walk.applied.len()).expect("至多 2"))
}

// ===== 第七节 B 类锚点 =====

fn two_to_the(exponent: u32) -> u64 {
    1u64 << exponent
}

/// B1：串落 H0 = 1 + Σ_i (6 + 2^{r_i})。
fn anchor_b1_closed_form(record_counts: &[u32]) -> u64 {
    1 + record_counts.iter().map(|count| 6 + two_to_the(*count)).sum::<u64>()
}

/// B2：同发 H0 甲整段、乙整段 = 2^{k+1} + 2^{Σr} + 3。
fn anchor_b2_closed_form(record_counts: &[u32]) -> u64 {
    let transaction_count = u32::try_from(record_counts.len()).expect("k 小");
    two_to_the(transaction_count + 1) + two_to_the(record_counts.iter().sum()) + 3
}

/// B4：作废·窄的闭式（登记第七节 B4 原样）。
fn anchor_b4_closed_form(condition: PublishCondition, reserves: bool, record_counts: &[u32], voided: u32) -> u64 {
    let transaction_count = u32::try_from(record_counts.len()).expect("k 小");
    let count_of = |transaction: u32| record_counts[transaction_position(transaction)];
    let others: Vec<u32> = (1..=transaction_count).filter(|transaction| *transaction != voided).collect();
    match (condition, reserves) {
        (PublishCondition::SharedPublish, false) => two_to_the(transaction_count) + two_to_the(others.iter().map(|transaction| count_of(*transaction)).sum()) + 3,
        (PublishCondition::SharedPublish, true) => two_to_the(transaction_count) + two_to_the(record_counts.iter().sum()) + 3,
        (PublishCondition::SerialPublishes, false) => 1 + others.iter().map(|transaction| 6 + two_to_the(count_of(*transaction))).sum::<u64>(),
        (PublishCondition::SerialPublishes, true) => {
            if voided < transaction_count {
                let rest: u64 = (1..=transaction_count)
                    .filter(|transaction| *transaction != voided && *transaction != voided + 1)
                    .map(|transaction| 6 + two_to_the(count_of(transaction)))
                    .sum();
                1 + rest + (6 + two_to_the(count_of(voided) + count_of(voided + 1)))
            } else {
                1 + others.iter().map(|transaction| 6 + two_to_the(count_of(*transaction))).sum::<u64>() + (3 + two_to_the(count_of(voided)))
            }
        }
        (PublishCondition::MultiFlightRootsOrdered | PublishCondition::MultiFlightRootsUnordered, _) => panic!("B4 只登记了串落与同发的闭式"),
    }
}

/// B5：`anchors_q1c.py` 原样输出（登记第十三节）：(r, 条件, 臂, 执行数, 去重状态数)，多飞 k = 2、H0、读法 R2。
const ANCHOR_B5: [(&[u32], PublishCondition, AppendArm, u64, u64); 16] = [
    (&[1, 1], PublishCondition::MultiFlightRootsOrdered, AppendArm::JiaWhole, 3747, 56),
    (&[1, 1], PublishCondition::MultiFlightRootsOrdered, AppendArm::JiaInterleaved, 3747, 56),
    (&[1, 1], PublishCondition::MultiFlightRootsOrdered, AppendArm::YiReservedWhole, 5049, 59),
    (&[1, 1], PublishCondition::MultiFlightRootsOrdered, AppendArm::DingNoDiscipline, 5049, 80),
    (&[1, 1], PublishCondition::MultiFlightRootsUnordered, AppendArm::JiaWhole, 6435, 62),
    (&[1, 1], PublishCondition::MultiFlightRootsUnordered, AppendArm::JiaInterleaved, 6435, 62),
    (&[1, 1], PublishCondition::MultiFlightRootsUnordered, AppendArm::YiReservedWhole, 12870, 77),
    (&[1, 1], PublishCondition::MultiFlightRootsUnordered, AppendArm::DingNoDiscipline, 12870, 104),
    (&[2, 2], PublishCondition::MultiFlightRootsOrdered, AppendArm::JiaWhole, 10006, 82),
    (&[2, 2], PublishCondition::MultiFlightRootsOrdered, AppendArm::JiaInterleaved, 13366, 146),
    (&[2, 2], PublishCondition::MultiFlightRootsOrdered, AppendArm::YiReservedWhole, 19162, 91),
    (&[2, 2], PublishCondition::MultiFlightRootsOrdered, AppendArm::DingNoDiscipline, 19162, 246),
    (&[2, 2], PublishCondition::MultiFlightRootsUnordered, AppendArm::JiaWhole, 15490, 88),
    (&[2, 2], PublishCondition::MultiFlightRootsUnordered, AppendArm::JiaInterleaved, 24310, 167),
    (&[2, 2], PublishCondition::MultiFlightRootsUnordered, AppendArm::YiReservedWhole, 48620, 115),
    (&[2, 2], PublishCondition::MultiFlightRootsUnordered, AppendArm::DingNoDiscipline, 48620, 306),
];

/// B3：同发 H0 甲交错 / 对照丁的（执行数，去重状态数），`anchors_tongfa.py` 原样输出（登记第十三节）。
const ANCHOR_B3: [(&[u32], (u64, u64), (u64, u64)); 6] = [
    (&[2, 1], (2, 29), (3, 39)),
    (&[1, 2], (1, 19), (3, 39)),
    (&[2, 2], (3, 59), (6, 104)),
    (&[1, 1, 1], (1, 27), (6, 73)),
    (&[2, 1, 1], (3, 67), (12, 188)),
    (&[2, 2, 2], (15, 695), (90, 3235)),
];

/// B4：`anchors_void.py` 原样输出（登记第十三节）：(r, v, 串落甲, 串落乙, 同发甲, 同发乙)。
const ANCHOR_B4_SCRIPT: [(&[u32], u32, u64, u64, u64, u64); 10] = [
    (&[1, 1], 1, 9, 11, 9, 11),
    (&[1, 1], 2, 9, 14, 9, 11),
    (&[2, 2], 1, 11, 23, 11, 23),
    (&[2, 2], 2, 11, 18, 11, 23),
    (&[1, 1, 1], 1, 17, 19, 15, 19),
    (&[1, 1, 1], 2, 17, 19, 15, 19),
    (&[1, 1, 1], 3, 17, 22, 15, 19),
    (&[2, 2, 2], 1, 21, 33, 27, 75),
    (&[2, 2, 2], 2, 21, 33, 27, 75),
    (&[2, 2, 2], 3, 21, 28, 27, 75),
];

/// B1、B2 登记里写出数的那五格。
const ANCHOR_B1_LISTED: [(&[u32], u64); 5] = [(&[1, 1], 17), (&[2, 2], 21), (&[1, 2], 19), (&[1, 1, 1], 25), (&[2, 2, 2], 31)];
const ANCHOR_B2_LISTED: [(&[u32], u64); 5] = [(&[1, 1], 15), (&[2, 2], 27), (&[1, 2], 19), (&[1, 1, 1], 27), (&[2, 2, 2], 83)];

fn h0(record_counts: &[u32]) -> History {
    History { record_counts: record_counts.to_vec(), voided: None }
}

fn h1(record_counts: &[u32], voided: u32) -> History {
    History { record_counts: record_counts.to_vec(), voided: Some((voided, VoidTiming::NarrowBeforeAnyWrite)) }
}

fn grid_cell(condition: PublishCondition, append_arm: AppendArm, history: History, reading: AppendedReading) -> AppendCellResult {
    evaluate_cell(&CellRequest::main_grid(condition, append_arm, history, reading)).expect("这一格适用")
}

// ===== 输出 =====

fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn yes_or_no(holds: bool) -> &'static str {
    if holds {
        "yes"
    } else {
        "no"
    }
}

fn hit_or_not(value: u64) -> &'static str {
    if value > 0 {
        "hit"
    } else {
        "not_hit"
    }
}

fn cell_key_text(condition: PublishCondition, append_arm: AppendArm, history: &History, reading: AppendedReading) -> String {
    format!(
        "q1={} q2={} family={} k={} r={} voided={} reading={}",
        condition.label(),
        append_arm.label(),
        history.family().label(),
        history.transaction_count(),
        history.record_counts_label(),
        history.voided_label(),
        reading.label()
    )
}

type GridKey = (PublishCondition, AppendArm, History, AppendedReading);

fn emit_anchors(emitter: &mut Emitter) -> u64 {
    let mut checked = 0u64;
    let mut mismatches = 0u64;
    let mut report = |emitter: &mut Emitter, family: &str, detail: String, expected: u64, measured: u64| {
        checked += 1;
        if expected != measured {
            mismatches += 1;
        }
        emit(emitter, &format!("name=anchor family={family} {detail} expected={expected} measured={measured} matches={}", yes_or_no(expected == measured)));
    };
    let h0_record_counts: Vec<Vec<u32>> = all_histories().into_iter().filter(|history| history.voided.is_none()).map(|history| history.record_counts).collect();
    for counts in &h0_record_counts {
        let listed = ANCHOR_B1_LISTED.iter().find(|(listed_counts, _)| *listed_counts == counts.as_slice()).map(|(_, states)| *states);
        for arm in AppendArm::ALL {
            let measured = grid_cell(PublishCondition::SerialPublishes, arm, h0(counts), AppendedReading::R2IssuedAndCovered).states;
            report(emitter, "B1", format!("q2={} r={} source=closed_form", arm.label(), h0(counts).record_counts_label()), anchor_b1_closed_form(counts), measured);
            if let Some(listed_states) = listed {
                report(emitter, "B1", format!("q2={} r={} source=listed", arm.label(), h0(counts).record_counts_label()), listed_states, measured);
            }
        }
    }
    for counts in &h0_record_counts {
        let listed = ANCHOR_B2_LISTED.iter().find(|(listed_counts, _)| *listed_counts == counts.as_slice()).map(|(_, states)| *states);
        for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
            let measured = grid_cell(PublishCondition::SharedPublish, arm, h0(counts), AppendedReading::R2IssuedAndCovered).states;
            report(emitter, "B2", format!("q2={} r={} source=closed_form", arm.label(), h0(counts).record_counts_label()), anchor_b2_closed_form(counts), measured);
            if let Some(listed_states) = listed {
                report(emitter, "B2", format!("q2={} r={} source=listed", arm.label(), h0(counts).record_counts_label()), listed_states, measured);
            }
        }
    }
    for (counts, interleaved, control) in ANCHOR_B3 {
        for (arm, (executions, states)) in [(AppendArm::JiaInterleaved, interleaved), (AppendArm::DingNoDiscipline, control)] {
            let measured = grid_cell(PublishCondition::SharedPublish, arm, h0(counts), AppendedReading::R2IssuedAndCovered);
            report(emitter, "B3", format!("q2={} r={} quantity=executions source=script", arm.label(), h0(counts).record_counts_label()), executions, measured.executions);
            report(emitter, "B3", format!("q2={} r={} quantity=states source=script", arm.label(), h0(counts).record_counts_label()), states, measured.states);
        }
    }
    for (counts, voided, serial_jia, serial_yi, shared_jia, shared_yi) in ANCHOR_B4_SCRIPT {
        for (condition, arm, expected) in [
            (PublishCondition::SerialPublishes, AppendArm::JiaWhole, serial_jia),
            (PublishCondition::SerialPublishes, AppendArm::YiReservedWhole, serial_yi),
            (PublishCondition::SharedPublish, AppendArm::JiaWhole, shared_jia),
            (PublishCondition::SharedPublish, AppendArm::YiReservedWhole, shared_yi),
        ] {
            let measured = grid_cell(condition, arm, h1(counts, voided), AppendedReading::R2IssuedAndCovered).states;
            report(emitter, "B4", format!("q1={} q2={} r={} voided=T{voided} source=script", condition.label(), arm.label(), h0(counts).record_counts_label()), expected, measured);
        }
    }
    for history in all_histories().into_iter().filter(|candidate| candidate.family() == HistoryFamily::H1) {
        let (voided, _) = history.voided.expect("H1 有作废事务");
        for condition in PublishCondition::SEGMENT_ONE {
            for (arm, reserves) in [(AppendArm::JiaWhole, false), (AppendArm::YiReservedWhole, true)] {
                let measured = grid_cell(condition, arm, history.clone(), AppendedReading::R2IssuedAndCovered).states;
                report(
                    emitter,
                    "B4",
                    format!("q1={} q2={} r={} voided=T{voided} source=closed_form", condition.label(), arm.label(), history.record_counts_label()),
                    anchor_b4_closed_form(condition, reserves, &history.record_counts, voided),
                    measured,
                );
            }
        }
    }
    emit(emitter, &format!("name=anchor_summary families=B1,B2,B3,B4 checked={checked} mismatches={mismatches}"));
    mismatches
}

fn emit_cell(emitter: &mut Emitter, key_text: &str, result: &AppendCellResult) {
    emit(
        emitter,
        &format!(
            "name=cell {key_text} applicable=yes executions={} states={} n2a={} n2b={} strict_segment_length_peak={} in_flight_limit_states={} a7_violation_points={} pending_initial_peak={} pending_peak_after_drain={} pending_positive_closers_peak={} pending_final_peak={} pending_never_drained={} a5_differing_states={}",
            result.executions,
            result.states,
            result.incomplete_pair_states,
            result.pattern_p_states,
            result.strict_segment_length_peak,
            result.in_flight_limit_states,
            result.a7_violation_points,
            result.pending.initial_peak,
            result.pending.peak_after_drain,
            result.pending.positive_closers_peak,
            result.pending.final_peak,
            result.pending.never_drained,
            result.a5_differing_states
        ),
    );
    for arm in RecoveryArm::ALL {
        let counts = &result.recovery[arm.position()];
        emit(
            emitter,
            &format!(
                "name=cell_recovery {key_text} q3={} states={} refused={} v_fsync={} v_fsync_mounted_lost={} v_fsync_refused_with_returned={} v_fsync_branch_all_appended={} v_fsync_branch_void={} v_dep={} v_dangling_returned_empty={} v_dangling_returned_nonempty={} partial_publish={} n_atom={} core_failures={} differs_from_bing0={}",
                arm.label(),
                result.states,
                counts.refused,
                counts.v_fsync,
                counts.v_fsync_mounted_lost,
                counts.v_fsync_refused_with_returned,
                counts.v_fsync_branch_every_earlier_appended,
                counts.v_fsync_branch_earlier_passed_as_voided,
                counts.v_dependency,
                counts.v_dangling_returned_empty,
                counts.v_dangling_returned_nonempty,
                counts.partial_publish,
                counts.n_atom,
                counts.core_failures,
                counts.differs_from_bing0
            ),
        );
    }
}

fn emit_controls(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>) -> u64 {
    let mut controls_not_positive = 0u64;
    let mut report = |emitter: &mut Emitter, body: String, value: u64| {
        if value == 0 {
            controls_not_positive += 1;
        }
        emit(emitter, &format!("name=control {body} value={value} positive={}", yes_or_no(value > 0)));
    };
    for reading in AppendedReading::ALL {
        // 5.2：每条 Q2 臂拿掉自己的纪律；对照丁本身也跑同一格。
        for (arm, discipline) in [
            (AppendArm::JiaWhole, AppendDiscipline::RemovedForPositiveControl),
            (AppendArm::JiaInterleaved, AppendDiscipline::RemovedForPositiveControl),
            (AppendArm::YiReservedWhole, AppendDiscipline::RemovedForPositiveControl),
            (AppendArm::DingNoDiscipline, AppendDiscipline::Kept),
        ] {
            let mut request = CellRequest::main_grid(PublishCondition::SharedPublish, arm, h0(&[2, 2]), reading);
            request.discipline = discipline;
            let result = evaluate_cell(&request).expect("H0 适用");
            let sample = "q1=shared_publish family=H0 k=2 r=2,2";
            report(emitter, format!("quantity=Q2-1_n2a control=discipline_removed q2={} {sample} reading={}", arm.label(), reading.label()), result.incomplete_pair_states);
            report(emitter, format!("quantity=Q2-2_n2b control=discipline_removed q2={} {sample} reading={}", arm.label(), reading.label()), result.pattern_p_states);
        }
        // 5.3 V_fsync：早返回，四条 Q2 臂都套，每条 Q3 臂各报。
        for arm in AppendArm::ALL {
            let mut request = CellRequest::main_grid(PublishCondition::SharedPublish, arm, h0(&[1, 1]), reading);
            request.return_rule = ReturnRule::EarlyAfterOwnRecordsIssued;
            let result = evaluate_cell(&request).expect("H0 适用");
            for recovery_arm in RecoveryArm::ALL {
                report(
                    emitter,
                    format!("quantity=Q3-1_v_fsync control=early_return q2={} q3={} q1=shared_publish family=H0 k=2 r=1,1 reading={}", arm.label(), recovery_arm.label(), reading.label()),
                    result.recovery[recovery_arm.position()].v_fsync,
                );
            }
        }
        // 5.3 V_dep：每条臂关掉「不保后面」的那一处、按记录施加。
        let mut dependency_request = CellRequest::main_grid(PublishCondition::SharedPublish, AppendArm::DingNoDiscipline, h0(&[2, 2]), reading);
        dependency_request.recovery_control = RecoveryControl::KeepLaterRecordsRemoved;
        let dependency_result = evaluate_cell(&dependency_request).expect("H0 适用");
        // 5.3 N_atom：每条臂去掉提交标记的处理并去掉整次发布施加。
        let mut atomicity_request = CellRequest::main_grid(PublishCondition::SharedPublish, AppendArm::JiaWhole, h0(&[2, 2]), reading);
        atomicity_request.recovery_control = RecoveryControl::CommitMarkerHandlingRemoved;
        let atomicity_result = evaluate_cell(&atomicity_request).expect("H0 适用");
        // 5.3 V_dangling：拿掉 B1、B2。
        let mut dangling_request = CellRequest::main_grid(PublishCondition::SerialPublishes, AppendArm::JiaWhole, h0(&[1, 1]), reading);
        dangling_request.barriers = PublishBarriers::WithoutBarriersAfterUnitsAndRecords;
        let dangling_result = evaluate_cell(&dangling_request).expect("H0 适用");
        for recovery_arm in RecoveryArm::ALL {
            report(
                emitter,
                format!("quantity=Q3-2_v_dep control=keep_later_records_removed q2=ding_control q3={} q1=shared_publish family=H0 k=2 r=2,2 reading={}", recovery_arm.label(), reading.label()),
                dependency_result.recovery[recovery_arm.position()].v_dependency,
            );
            report(
                emitter,
                format!("quantity=A3_n_atom control=commit_marker_handling_removed q2=jia_whole q3={} q1=shared_publish family=H0 k=2 r=2,2 reading={}", recovery_arm.label(), reading.label()),
                atomicity_result.recovery[recovery_arm.position()].n_atom,
            );
            report(
                emitter,
                format!("quantity=Q3-3_v_dangling control=barriers_after_units_and_records_removed q2=jia_whole q3={} q1=serial_publishes family=H0 k=2 r=1,1 reading={}", recovery_arm.label(), reading.label()),
                dangling_result.recovery[recovery_arm.position()].v_dangling(),
            );
        }
        // 5.3 判坏（只丙3）与部分施加（只丙2）：对照丁当 Q2 臂，主网格里那两格。
        let judged_bad = &grid[&(PublishCondition::SharedPublish, AppendArm::DingNoDiscipline, h0(&[2, 1]), reading)];
        report(
            emitter,
            format!("quantity=bing3_refused control=ding_as_q2 q2=ding_control q3=bing3_forbid q1=shared_publish family=H0 k=2 r=2,1 reading={}", reading.label()),
            judged_bad.recovery[RecoveryArm::Bing3ForbidPattern.position()].refused,
        );
        let partial = &grid[&(PublishCondition::SharedPublish, AppendArm::DingNoDiscipline, h0(&[2, 2]), reading)];
        report(
            emitter,
            format!("quantity=Q3-4_partial_publish control=ding_as_q2 q2=ding_control q3=bing2_drop_a_keep_b q1=shared_publish family=H0 k=2 r=2,2 reading={}", reading.label()),
            partial.recovery[RecoveryArm::Bing2DropAKeepB.position()].partial_publish,
        );
    }
    emit(emitter, &format!("name=control_summary controls_not_positive={controls_not_positive}"));
    controls_not_positive
}

/// 合法历史（H0 ∪ H1）上某个量的合计与为正的格数。
fn legal_sum(grid: &BTreeMap<GridKey, AppendCellResult>, condition: PublishCondition, arm: AppendArm, reading: AppendedReading, quantity: &dyn Fn(&AppendCellResult) -> u64) -> (u64, u64, u64) {
    let mut cells = 0u64;
    let mut positive_cells = 0u64;
    let mut total = 0u64;
    for ((cell_condition, cell_arm, history, cell_reading), result) in grid {
        if *cell_condition == condition && *cell_arm == arm && *cell_reading == reading && history.family().is_legal() {
            let value = quantity(result);
            cells += 1;
            positive_cells += u64::from(value > 0);
            total += value;
        }
    }
    (cells, positive_cells, total)
}

fn emit_verdicts(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>, conditions: &[PublishCondition]) {
    for condition in conditions.iter().copied() {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                let head = format!("q1={} q2={} reading={}", condition.label(), arm.label(), reading.label());
                let (cells, positive_cells, total) = legal_sum(grid, condition, arm, reading, &|result| result.incomplete_pair_states);
                emit(emitter, &format!("name=verdict quantity=Q2-1 {head} legal_cells={cells} legal_cells_positive={positive_cells} legal_states={total} judgement={}", hit_or_not(total)));
                let (cells, positive_cells, total) = legal_sum(grid, condition, arm, reading, &|result| result.pattern_p_states);
                let pairing = if total > 0 { "bing3_pairing_out" } else { "bing3_pairing_stands" };
                emit(emitter, &format!("name=verdict quantity=Q2-2 {head} legal_cells={cells} legal_cells_positive={positive_cells} legal_states={total} judgement={pairing}"));
                for recovery_arm in RecoveryArm::ALL {
                    let position = recovery_arm.position();
                    let (cells, positive_cells, total) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].v_fsync);
                    let (_, _, mounted_lost) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].v_fsync_mounted_lost);
                    let (_, _, refused_with_returned) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].v_fsync_refused_with_returned);
                    let (_, _, branch_appended) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].v_fsync_branch_every_earlier_appended);
                    let (_, _, branch_void) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].v_fsync_branch_earlier_passed_as_voided);
                    emit(
                        emitter,
                        &format!(
                            "name=verdict quantity=Q3-1 {head} q3={} legal_cells={cells} legal_cells_positive={positive_cells} legal_states={total} mounted_lost={mounted_lost} refused_with_returned={refused_with_returned} branch_all_appended={branch_appended} branch_void={branch_void} judgement={}",
                            recovery_arm.label(),
                            hit_or_not(total)
                        ),
                    );
                    let (cells, positive_cells, total) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[position].v_dependency);
                    emit(emitter, &format!("name=verdict quantity=Q3-2 {head} q3={} legal_cells={cells} legal_cells_positive={positive_cells} legal_states={total} judgement={}", recovery_arm.label(), hit_or_not(total)));
                }
                // Q3-3：四条 Q3 臂同值 ⇒ 打中不分辨臂（F8）；不同值 ⇒ 值大的那几条臂。
                let per_arm: Vec<u64> = RecoveryArm::ALL
                    .iter()
                    .map(|recovery_arm| legal_sum(grid, condition, arm, reading, &|result| result.recovery[recovery_arm.position()].v_dangling()).2)
                    .collect();
                let (_, cells_where_arms_differ, _) = legal_sum(grid, condition, arm, reading, &|result| {
                    let first = result.recovery[0].v_dangling();
                    u64::from(result.recovery.iter().any(|counts| counts.v_dangling() != first))
                });
                let largest = per_arm.iter().copied().max().unwrap_or(0);
                let judgement = if largest == 0 {
                    "all_zero".to_string()
                } else if per_arm.iter().all(|value| *value == largest) {
                    "same_nonzero_across_arms_f8".to_string()
                } else {
                    let arms: Vec<&str> = RecoveryArm::ALL.iter().filter(|recovery_arm| per_arm[recovery_arm.position()] == largest).map(|recovery_arm| recovery_arm.label()).collect();
                    format!("differs_largest={}", arms.join(","))
                };
                emit(
                    emitter,
                    &format!(
                        "name=verdict quantity=Q3-3 {head} bing0={} bing1={} bing2={} bing3={} legal_cells_where_arms_differ={cells_where_arms_differ} judgement={judgement}",
                        per_arm[0], per_arm[1], per_arm[2], per_arm[3]
                    ),
                );
                let bing2 = RecoveryArm::Bing2DropAKeepB.position();
                let (cells, positive_cells, total) = legal_sum(grid, condition, arm, reading, &|result| result.recovery[bing2].partial_publish);
                emit(emitter, &format!("name=verdict quantity=Q3-4 {head} q3=bing2_drop_a_keep_b legal_cells={cells} legal_cells_positive={positive_cells} legal_states={total} judgement={}", hit_or_not(total)));
            }
        }
    }
}

fn emit_assertions(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>, conditions: &[PublishCondition], includes_real_baseline: bool) {
    // A1 真实基线：串落 × 甲整段 × 丙0，H0 与 H1（只在串落那一段的格里有）。
    let real_baseline_readings: &[AppendedReading] = if includes_real_baseline { &AppendedReading::ALL } else { &[] };
    for reading in real_baseline_readings.iter().copied() {
        let bing0 = RecoveryArm::Bing0Today.position();
        let quantities: [(&str, &dyn Fn(&AppendCellResult) -> u64); 7] = [
            ("n2a", &|result: &AppendCellResult| result.incomplete_pair_states),
            ("n2b", &|result: &AppendCellResult| result.pattern_p_states),
            ("v_fsync", &|result: &AppendCellResult| result.recovery[bing0].v_fsync),
            ("v_dep", &|result: &AppendCellResult| result.recovery[bing0].v_dependency),
            ("v_dangling", &|result: &AppendCellResult| result.recovery[bing0].v_dangling()),
            ("partial_publish", &|result: &AppendCellResult| result.recovery[bing0].partial_publish),
            ("n_atom", &|result: &AppendCellResult| result.recovery[bing0].n_atom),
        ];
        let mut fields = Vec::new();
        let mut nonzero_quantities = 0u64;
        let mut cells = 0u64;
        for (name, quantity) in quantities {
            let (cell_count, _, total) = legal_sum(grid, PublishCondition::SerialPublishes, AppendArm::JiaWhole, reading, quantity);
            cells = cell_count;
            nonzero_quantities += u64::from(total > 0);
            fields.push(format!("{name}={total}"));
        }
        emit(emitter, &format!("name=assertion check=A1 q1=serial_publishes q2=jia_whole q3=bing0_today reading={} legal_cells={cells} {} nonzero_quantities={nonzero_quantities}", reading.label(), fields.join(" ")));
    }
    // A3：每个格、每条 Q3 臂 N_atom = 0（合法与 H2 分开报）。
    for legal in [true, false] {
        let mut per_arm = [0u64; 4];
        let mut cells = 0u64;
        for ((_, _, history, _), result) in grid {
            if history.family().is_legal() == legal {
                cells += 1;
                for recovery_arm in RecoveryArm::ALL {
                    per_arm[recovery_arm.position()] += result.recovery[recovery_arm.position()].n_atom;
                }
            }
        }
        emit(
            emitter,
            &format!(
                "name=assertion check=A3 histories={} cells={cells} bing0_n_atom={} bing1_n_atom={} bing2_n_atom={} bing3_n_atom={}",
                if legal { "legal" } else { "h2" },
                per_arm[0], per_arm[1], per_arm[2], per_arm[3]
            ),
        );
    }
    // A4：甲整段、乙整段在 H0、H1 的每一格 N2a = N2b = 0。
    for condition in conditions.iter().copied() {
        for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
            for reading in AppendedReading::ALL {
                let (cells, _, n2a) = legal_sum(grid, condition, arm, reading, &|result| result.incomplete_pair_states);
                let (_, _, n2b) = legal_sum(grid, condition, arm, reading, &|result| result.pattern_p_states);
                emit(emitter, &format!("name=assertion check=A4 q1={} q2={} reading={} legal_cells={cells} n2a={n2a} n2b={n2b}", condition.label(), arm.label(), reading.label()));
            }
        }
    }
    // A5：甲整段、乙整段在 H0、H1 的每一格四条 Q3 臂恢复出的版本逐状态相同；丙0、丙1、丙3 的 V_dep 在每一格 = 0。
    for condition in conditions.iter().copied() {
        for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
            for reading in AppendedReading::ALL {
                let (cells, positive_cells, differing) = legal_sum(grid, condition, arm, reading, &|result| result.a5_differing_states);
                let differing_by_arm: Vec<String> = RecoveryArm::ALL
                    .iter()
                    .map(|recovery_arm| format!("{}_differs={}", recovery_arm.label(), legal_sum(grid, condition, arm, reading, &|result| result.recovery[recovery_arm.position()].differs_from_bing0).2))
                    .collect();
                emit(
                    emitter,
                    &format!(
                        "name=assertion check=A5_versions q1={} q2={} reading={} legal_cells={cells} legal_cells_with_differing_states={positive_cells} differing_states={differing} {}",
                        condition.label(),
                        arm.label(),
                        reading.label(),
                        differing_by_arm.join(" ")
                    ),
                );
            }
        }
    }
    for condition in conditions.iter().copied() {
        for reading in AppendedReading::ALL {
            let mut fields = Vec::new();
            for recovery_arm in [RecoveryArm::Bing0Today, RecoveryArm::Bing1TruncateBeforeA, RecoveryArm::Bing3ForbidPattern] {
                let total: u64 = AppendArm::ALL
                    .iter()
                    .map(|arm| legal_sum(grid, condition, *arm, reading, &|result| result.recovery[recovery_arm.position()].v_dependency).2)
                    .sum();
                fields.push(format!("{}_v_dep={total}", recovery_arm.label()));
            }
            emit(emitter, &format!("name=assertion check=A5_v_dep q1={} q2=all reading={} {}", condition.label(), reading.label(), fields.join(" ")));
        }
    }
    // A6、A7 与轨迹期末值：全部格。
    let mut longest = 0usize;
    let mut limit_states = 0u64;
    let mut a7_points = 0u64;
    let mut final_peak = 0u64;
    let mut never_drained = 0u64;
    let mut core_failures = 0u64;
    for result in grid.values() {
        longest = longest.max(result.strict_segment_length_peak);
        limit_states += result.in_flight_limit_states;
        a7_points += result.a7_violation_points;
        final_peak = final_peak.max(result.pending.final_peak);
        never_drained += result.pending.never_drained;
        core_failures += result.recovery.iter().map(|counts| counts.core_failures).sum::<u64>();
    }
    emit(emitter, &format!("name=assertion check=A6 cells={} strict_segment_length_peak={longest} bound={STRICT_SEGMENT_LENGTH_BOUND} in_flight_limit={IN_FLIGHT_RECORD_LIMIT} in_flight_limit_states={limit_states}", grid.len()));
    emit(emitter, &format!("name=assertion check=A7 cells={} a7_violation_points={a7_points}", grid.len()));
    emit(emitter, &format!("name=assertion check=pending_trajectory cells={} pending_final_peak={final_peak} pending_never_drained={never_drained}", grid.len()));
    emit(emitter, &format!("name=assertion check=core_failures cells={} core_failures={core_failures}", grid.len()));
}

/// 登记 8.2：在合法格里只取某一 (k, r) 的 H0 与 H1 各格判一次。
fn point_judgement(grid: &BTreeMap<GridKey, AppendCellResult>, condition: PublishCondition, arm: AppendArm, reading: AppendedReading, record_counts: &[u32], quantity: &dyn Fn(&AppendCellResult) -> u64) -> u64 {
    grid.iter()
        .filter(|((cell_condition, cell_arm, history, cell_reading), _)| {
            *cell_condition == condition && *cell_arm == arm && *cell_reading == reading && history.family().is_legal() && history.record_counts == record_counts
        })
        .map(|(_, result)| quantity(result))
        .sum()
}

/// 第八节几何敏感性判的各量（第六节 Q2-1、Q2-2、Q3-1 至 Q3-4）。
fn geometry_quantities() -> Vec<(String, Box<dyn Fn(&AppendCellResult) -> u64>)> {
    let mut quantities: Vec<(String, Box<dyn Fn(&AppendCellResult) -> u64>)> = vec![
        ("Q2-1".to_string(), Box::new(|result: &AppendCellResult| result.incomplete_pair_states)),
        ("Q2-2".to_string(), Box::new(|result: &AppendCellResult| result.pattern_p_states)),
    ];
    for recovery_arm in RecoveryArm::ALL {
        let position = recovery_arm.position();
        quantities.push((format!("Q3-1 q3={}", recovery_arm.label()), Box::new(move |result: &AppendCellResult| result.recovery[position].v_fsync)));
        quantities.push((format!("Q3-2 q3={}", recovery_arm.label()), Box::new(move |result: &AppendCellResult| result.recovery[position].v_dependency)));
        quantities.push((format!("Q3-3 q3={}", recovery_arm.label()), Box::new(move |result: &AppendCellResult| result.recovery[position].v_dangling())));
    }
    let bing2 = RecoveryArm::Bing2DropAKeepB.position();
    quantities.push(("Q3-4 q3=bing2_drop_a_keep_b".to_string(), Box::new(move |result: &AppendCellResult| result.recovery[bing2].partial_publish)));
    quantities
}

/// 登记 8.2：G1、G2 在给定的条件里判；`includes_g4` 时另报 G4（多飞·根序对多飞·根乱，两个子形态互为方向相反的点）。
fn emit_geometry(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>, conditions: &[PublishCondition], includes_g4: bool) {
    let checks: [(&str, &[u32], Vec<(&str, &[u32])>); 2] = [
        ("G1", &[2, 2], vec![("r1,1", &[1, 1][..]), ("r1,1,1", &[1, 1, 1][..])]),
        ("G2", &[2, 1], vec![("r1,2", &[1, 2][..])]),
    ];
    let mut unstable = 0u64;
    let mut lines = 0u64;
    for (check, main_point, opposite_points) in &checks {
        for condition in conditions.iter().copied() {
            for arm in AppendArm::ALL {
                for reading in AppendedReading::ALL {
                    let quantities = geometry_quantities();
                    for (quantity_name, quantity) in &quantities {
                        let main_value = point_judgement(grid, condition, arm, reading, main_point, quantity.as_ref());
                        let mut fields = vec![format!("main={}", hit_or_not(main_value))];
                        let mut flips = false;
                        for (point_name, point) in opposite_points {
                            let value = point_judgement(grid, condition, arm, reading, point, quantity.as_ref());
                            flips |= (value > 0) != (main_value > 0);
                            fields.push(format!("{point_name}={}", hit_or_not(value)));
                        }
                        unstable += u64::from(flips);
                        lines += 1;
                        emit(
                            emitter,
                            &format!(
                                "name=geometry check={check} quantity={quantity_name} q1={} q2={} reading={} {} stability={}",
                                condition.label(),
                                arm.label(),
                                reading.label(),
                                fields.join(" "),
                                if flips { "unstable" } else { "two_points_agree" }
                            ),
                        );
                    }
                }
            }
        }
    }
    if includes_g4 {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                for (quantity_name, quantity) in &geometry_quantities() {
                    let ordered = legal_sum(grid, PublishCondition::MultiFlightRootsOrdered, arm, reading, quantity.as_ref()).2;
                    let unordered = legal_sum(grid, PublishCondition::MultiFlightRootsUnordered, arm, reading, quantity.as_ref()).2;
                    let flips = (ordered > 0) != (unordered > 0);
                    unstable += u64::from(flips);
                    lines += 1;
                    emit(
                        emitter,
                        &format!(
                            "name=geometry check=G4 quantity={quantity_name} q2={} reading={} multi_flight_roots_ordered={} multi_flight_roots_unordered={} stability={}",
                            arm.label(),
                            reading.label(),
                            hit_or_not(ordered),
                            hit_or_not(unordered),
                            if flips { "unstable" } else { "two_points_agree" }
                        ),
                    );
                }
            }
        }
    }
    emit(emitter, &format!("name=geometry_summary lines={lines} unstable_lines={unstable}"));
}

fn emit_h2_summaries(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>) {
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                let cells: Vec<&AppendCellResult> = grid
                    .iter()
                    .filter(|((cell_condition, cell_arm, history, cell_reading), _)| *cell_condition == condition && *cell_arm == arm && *cell_reading == reading && history.family() == HistoryFamily::H2)
                    .map(|(_, result)| result)
                    .collect();
                if cells.is_empty() {
                    emit(emitter, &format!("name=h2_summary q1={} q2={} reading={} applicable=no", condition.label(), arm.label(), reading.label()));
                    continue;
                }
                let n2a: u64 = cells.iter().map(|result| result.incomplete_pair_states).sum();
                let n2b: u64 = cells.iter().map(|result| result.pattern_p_states).sum();
                let mut fields = vec![format!("cells={} n2a={n2a} n2b={n2b}", cells.len())];
                for recovery_arm in RecoveryArm::ALL {
                    let mut totals = RecoveryCounts::default();
                    for result in &cells {
                        totals.absorb(&result.recovery[recovery_arm.position()]);
                    }
                    fields.push(format!(
                        "{}=v_fsync:{},v_dep:{},v_dangling:{},partial_publish:{},n_atom:{},refused:{}",
                        recovery_arm.label(),
                        totals.v_fsync,
                        totals.v_dependency,
                        totals.v_dangling(),
                        totals.partial_publish,
                        totals.n_atom,
                        totals.refused
                    ));
                }
                emit(emitter, &format!("name=h2_summary q1={} q2={} reading={} applicable=yes {}", condition.label(), arm.label(), reading.label(), fields.join(" ")));
            }
        }
    }
}

// ===== 第三段：fsync 等待模型（登记 5.7；离散事件，抽象单位，不是本机计时；读法见登记第十二节修订 3、4） =====

/// 登记 5.7 的抽象单位换成整数：1 个抽象单位 = 1000 个整数单位（w = 0.05 → 50），算术全用整数。
const WAIT_UNITS_PER_ABSTRACT_UNIT: u64 = 1000;
/// 登记 5.7：k ∈ {1, 2, 4, 8, 16, 32, 64}。
const WAIT_TRANSACTION_COUNTS: [u32; 7] = [1, 2, 4, 8, 16, 32, 64];
/// 登记 5.7：每个事务 r 相同，r ∈ {1, 2}。
const WAIT_RECORD_COUNTS: [u32; 2] = [1, 2];
/// 登记第六节 Q2-3：ρ = W_max(64) ÷ W_max(8)。
const GROWTH_RATIO_LARGE_TRANSACTION_COUNT: u32 = 64;
const GROWTH_RATIO_SMALL_TRANSACTION_COUNT: u32 = 8;
/// 登记第六节 Q2-3 的门槛：ρ ≥ 2 判「随 k 增长」，ρ ≤ 1.25 判「有界」；写成（分子，分母），按整数乘法比。
const REGISTERED_GROWS_THRESHOLD: (u64, u64) = (2, 1);
const REGISTERED_BOUNDED_THRESHOLD: (u64, u64) = (5, 4);
/// 登记 5.7 只给了甲两条臂（追加者）与乙整段（预留）的等待规则，对照丁没有，第三段只跑这三条臂（修订 4 第 1 条）。
const WAIT_APPEND_ARMS: [AppendArm; 3] = [AppendArm::JiaWhole, AppendArm::JiaInterleaved, AppendArm::YiReservedWhole];
const WAIT_CONDITIONS: [PublishCondition; 4] = [
    PublishCondition::SerialPublishes,
    PublishCondition::SharedPublish,
    PublishCondition::MultiFlightRootsOrdered,
    PublishCondition::MultiFlightRootsUnordered,
];

/// 登记 5.7 的两个代价取样点（拍的，方向相反）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum CostPoint {
    /// P1 设备主导：w = 0.05、b = 1、h = g = 0.005、Δ = 5。
    DeviceDominated,
    /// P2 CPU 主导：w = 0.05、b = 1、h = g = 0.5、Δ = 5。
    CpuDominated,
}

#[derive(Clone, Copy, Debug)]
struct WaitCosts {
    write: u64,
    barrier_or_forced_unit_access: u64,
    cpu_step: u64,
    slow_head_delay: u64,
}

impl CostPoint {
    const ALL: [CostPoint; 2] = [CostPoint::DeviceDominated, CostPoint::CpuDominated];
    fn label(self) -> &'static str {
        match self {
            CostPoint::DeviceDominated => "p1_device_dominated",
            CostPoint::CpuDominated => "p2_cpu_dominated",
        }
    }
    fn costs(self) -> WaitCosts {
        match self {
            CostPoint::DeviceDominated => WaitCosts { write: 50, barrier_or_forced_unit_access: 1000, cpu_step: 5, slow_head_delay: 5000 },
            CostPoint::CpuDominated => WaitCosts { write: 50, barrier_or_forced_unit_access: 1000, cpu_step: 500, slow_head_delay: 5000 },
        }
    }
}

/// 登记 5.7 的两种准备时间剖面。
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum PreparationProfile {
    /// 等速：每个事务的计划在时刻 0 就绪。
    Uniform,
    /// 慢头：T1 在时刻 Δ 就绪，其余在 0。
    SlowHead,
}

impl PreparationProfile {
    const ALL: [PreparationProfile; 2] = [PreparationProfile::Uniform, PreparationProfile::SlowHead];
    fn label(self) -> &'static str {
        match self {
            PreparationProfile::Uniform => "uniform",
            PreparationProfile::SlowHead => "slow_head",
        }
    }
    fn ready_time_of(self, transaction: u32, costs: &WaitCosts) -> u64 {
        match self {
            PreparationProfile::SlowHead if transaction == 1 => costs.slow_head_delay,
            PreparationProfile::SlowHead | PreparationProfile::Uniform => 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WaitServer {
    /// 设备：一个先进先出的服务台。
    Device,
    /// 甲两条臂的追加者（CPU 单服务台，每条记录耗 h）。
    Appender,
    /// 乙整段的预留（CPU 单服务台，每个事务耗 g）。
    Reserver,
}

impl WaitServer {
    fn position(self) -> usize {
        match self {
            WaitServer::Device => 0,
            WaitServer::Appender => 1,
            WaitServer::Reserver => 2,
        }
    }
    /// 关键路径上「同一服务台的前一个」是哪一类前驱（读法 ④）。
    fn queue_role(self) -> PredecessorRole {
        match self {
            WaitServer::Device => PredecessorRole::DeviceQueue,
            WaitServer::Appender | WaitServer::Reserver => PredecessorRole::Cpu,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WaitStepKind {
    UnitWrite,
    SharedUnitWrite,
    RecordWrite,
    SystemConfigurationWrite,
    Barrier,
    RootSlotForcedUnitAccess,
    AppenderStep,
    ReservationStep,
}

/// 关键路径分量（登记 5.7 最后一条）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PathComponent {
    BarrierOrForcedUnitAccess,
    DeviceWrite,
    CpuStep,
}

impl WaitStepKind {
    fn server(self) -> WaitServer {
        match self {
            WaitStepKind::UnitWrite
            | WaitStepKind::SharedUnitWrite
            | WaitStepKind::RecordWrite
            | WaitStepKind::SystemConfigurationWrite
            | WaitStepKind::Barrier
            | WaitStepKind::RootSlotForcedUnitAccess => WaitServer::Device,
            WaitStepKind::AppenderStep => WaitServer::Appender,
            WaitStepKind::ReservationStep => WaitServer::Reserver,
        }
    }
    /// 读法 ③：R2 的「被一道完成的屏障或 FUA 罩住」。
    fn covers_earlier_writes(self) -> bool {
        match self {
            WaitStepKind::Barrier | WaitStepKind::RootSlotForcedUnitAccess => true,
            WaitStepKind::UnitWrite
            | WaitStepKind::SharedUnitWrite
            | WaitStepKind::RecordWrite
            | WaitStepKind::SystemConfigurationWrite
            | WaitStepKind::AppenderStep
            | WaitStepKind::ReservationStep => false,
        }
    }
    fn path_component(self) -> PathComponent {
        match self {
            WaitStepKind::Barrier | WaitStepKind::RootSlotForcedUnitAccess => PathComponent::BarrierOrForcedUnitAccess,
            WaitStepKind::UnitWrite | WaitStepKind::SharedUnitWrite | WaitStepKind::RecordWrite | WaitStepKind::SystemConfigurationWrite => PathComponent::DeviceWrite,
            WaitStepKind::AppenderStep | WaitStepKind::ReservationStep => PathComponent::CpuStep,
        }
    }
}

fn wait_step_cost(kind: WaitStepKind, costs: &WaitCosts) -> u64 {
    match kind {
        WaitStepKind::UnitWrite | WaitStepKind::SharedUnitWrite | WaitStepKind::RecordWrite | WaitStepKind::SystemConfigurationWrite => costs.write,
        WaitStepKind::Barrier | WaitStepKind::RootSlotForcedUnitAccess => costs.barrier_or_forced_unit_access,
        WaitStepKind::AppenderStep => costs.cpu_step,
        WaitStepKind::ReservationStep => costs.cpu_step,
    }
}

/// 关键路径回追时前驱的类别；声明次序就是同刻时的先后（读法 ④：同一发布 > 设备队列 > CPU；跨发布的次序约束与就绪时刻是修订 4 第 3 条补的位置）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum PredecessorRole {
    SamePublish,
    DeviceQueue,
    CrossPublish,
    Cpu,
    ReadySource,
}

#[derive(Clone, Debug)]
struct WaitNode {
    kind: WaitStepKind,
    publish_position: usize,
    /// 同刻时第一个键：事务号（发布级的操作取这次发布里最小的事务号）。
    transaction: u32,
    /// 同刻时第二个键：在事务（或发布）里的先后。
    order_within_transaction: u32,
    predecessors: Vec<(usize, PredecessorRole)>,
    /// 这一步最早能发出的时刻（事务就绪，或预留从时刻 0 起）。
    ready_source: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct WaitScenario {
    condition: PublishCondition,
    append_arm: AppendArm,
    profile: PreparationProfile,
    point: CostPoint,
    record_count: u32,
    transaction_count: u32,
}

struct WaitPlan {
    nodes: Vec<WaitNode>,
    transaction_publish: Vec<usize>,
    /// 每个事务自己的记录写，按记录序。
    transaction_record_writes: Vec<Vec<usize>>,
    publish_final_barrier: Vec<usize>,
}

/// 登记 5.7 的写序与次序约束（读法 ①：同一事务内同步，同一次发布里不同事务的 U 各自就绪即发出）。
fn plan_wait_scenario(scenario: &WaitScenario) -> WaitPlan {
    let costs = scenario.point.costs();
    let transaction_count = scenario.transaction_count;
    let record_count = scenario.record_count;
    let publish_groups: Vec<Vec<u32>> = match scenario.condition {
        PublishCondition::SharedPublish => vec![(1..=transaction_count).collect()],
        PublishCondition::SerialPublishes | PublishCondition::MultiFlightRootsOrdered | PublishCondition::MultiFlightRootsUnordered => {
            (1..=transaction_count).map(|transaction| vec![transaction]).collect()
        }
    };
    let roots_in_txg_order = scenario.condition == PublishCondition::MultiFlightRootsOrdered;
    let mut nodes: Vec<WaitNode> = Vec::new();
    let mut add = |node: WaitNode| -> usize {
        nodes.push(node);
        nodes.len() - 1
    };
    let reservation_of: Vec<Option<usize>> = (1..=transaction_count)
        .map(|transaction| match scenario.append_arm {
            AppendArm::YiReservedWhole => Some(add(WaitNode {
                kind: WaitStepKind::ReservationStep,
                publish_position: 0,
                transaction,
                order_within_transaction: 0,
                predecessors: Vec::new(),
                ready_source: Some(0),
            })),
            AppendArm::JiaWhole | AppendArm::JiaInterleaved | AppendArm::DingNoDiscipline => None,
        })
        .collect();
    let mut transaction_publish = vec![0usize; transaction_position(transaction_count + 1)];
    let mut transaction_record_writes: Vec<Vec<usize>> = vec![Vec::new(); transaction_position(transaction_count + 1)];
    let mut appender_steps: Vec<Vec<usize>> = vec![Vec::new(); transaction_position(transaction_count + 1)];
    let mut publish_final_barrier: Vec<usize> = Vec::new();
    let mut previous_root_slot: Option<usize> = None;
    let mut previous_system_configuration: Option<usize> = None;
    for (publish_position, group) in publish_groups.iter().enumerate() {
        let publish_transaction = group[0];
        let publish_node = |kind: WaitStepKind, order_within_transaction: u32, predecessors: Vec<(usize, PredecessorRole)>| WaitNode {
            kind,
            publish_position,
            transaction: publish_transaction,
            order_within_transaction,
            predecessors,
            ready_source: None,
        };
        let mut units = Vec::new();
        for transaction in group {
            transaction_publish[transaction_position(*transaction)] = publish_position;
            let mut predecessors = Vec::new();
            // 串落：Ti 的第一个操作在 Ti−1 的 B3 完成之后发出。
            if scenario.condition == PublishCondition::SerialPublishes {
                if let Some(previous_final_barrier) = publish_final_barrier.last() {
                    predecessors.push((*previous_final_barrier, PredecessorRole::CrossPublish));
                }
            }
            units.push(add(WaitNode {
                kind: WaitStepKind::UnitWrite,
                publish_position,
                transaction: *transaction,
                order_within_transaction: 0,
                predecessors,
                ready_source: Some(scenario.profile.ready_time_of(*transaction, &costs)),
            }));
        }
        let shared_unit = add(publish_node(WaitStepKind::SharedUnitWrite, 1, units.iter().map(|unit| (*unit, PredecessorRole::SamePublish)).collect()));
        let barrier_after_units = add(publish_node(WaitStepKind::Barrier, 2, vec![(shared_unit, PredecessorRole::SamePublish)]));
        let mut publish_record_writes = Vec::new();
        for transaction in group {
            for record_index in 1..=record_count {
                let previous_write = transaction_record_writes[transaction_position(*transaction)].last().copied();
                let mut write_predecessors = Vec::new();
                match scenario.append_arm {
                    AppendArm::JiaWhole | AppendArm::JiaInterleaved => {
                        let mut step_predecessors = vec![(barrier_after_units, PredecessorRole::SamePublish)];
                        if let Some(previous) = previous_write {
                            step_predecessors.push((previous, PredecessorRole::SamePublish));
                        }
                        if record_index == 1 {
                            for earlier in 1..*transaction {
                                let earlier_steps = &appender_steps[transaction_position(earlier)];
                                let gate = match scenario.append_arm {
                                    AppendArm::JiaWhole => *earlier_steps.last().expect("前面的事务已经排好"),
                                    AppendArm::JiaInterleaved => *earlier_steps.first().expect("前面的事务已经排好"),
                                    AppendArm::YiReservedWhole | AppendArm::DingNoDiscipline => panic!("只有甲两条臂有追加者"),
                                };
                                step_predecessors.push((gate, PredecessorRole::Cpu));
                            }
                        }
                        let step = add(WaitNode {
                            kind: WaitStepKind::AppenderStep,
                            publish_position,
                            transaction: *transaction,
                            order_within_transaction: record_index,
                            predecessors: step_predecessors,
                            ready_source: None,
                        });
                        appender_steps[transaction_position(*transaction)].push(step);
                        write_predecessors.push((step, PredecessorRole::Cpu));
                    }
                    AppendArm::YiReservedWhole => {
                        write_predecessors.push((barrier_after_units, PredecessorRole::SamePublish));
                        write_predecessors.push((reservation_of[transaction_position(*transaction)].expect("乙整段每个事务都预留"), PredecessorRole::Cpu));
                        if let Some(previous) = previous_write {
                            write_predecessors.push((previous, PredecessorRole::SamePublish));
                        }
                    }
                    AppendArm::DingNoDiscipline => panic!("第三段不跑对照丁"),
                }
                let write = add(WaitNode {
                    kind: WaitStepKind::RecordWrite,
                    publish_position,
                    transaction: *transaction,
                    order_within_transaction: 2 + record_index,
                    predecessors: write_predecessors,
                    ready_source: None,
                });
                transaction_record_writes[transaction_position(*transaction)].push(write);
                publish_record_writes.push(write);
            }
        }
        let barrier_after_records = add(publish_node(WaitStepKind::Barrier, 5, publish_record_writes.iter().map(|write| (*write, PredecessorRole::SamePublish)).collect()));
        let mut root_slot_predecessors = vec![(barrier_after_records, PredecessorRole::SamePublish)];
        let mut system_configuration_predecessors = Vec::new();
        // 多飞·根序：F_i 在 F_{i−1} 完成之后、C_i 在 C_{i−1} 完成之后发出。
        if roots_in_txg_order {
            if let Some(previous) = previous_root_slot {
                root_slot_predecessors.push((previous, PredecessorRole::CrossPublish));
            }
            if let Some(previous) = previous_system_configuration {
                system_configuration_predecessors.push((previous, PredecessorRole::CrossPublish));
            }
        }
        let root_slot = add(publish_node(WaitStepKind::RootSlotForcedUnitAccess, 6, root_slot_predecessors));
        system_configuration_predecessors.insert(0, (root_slot, PredecessorRole::SamePublish));
        let system_configuration = add(publish_node(WaitStepKind::SystemConfigurationWrite, 7, system_configuration_predecessors));
        let final_barrier = add(publish_node(WaitStepKind::Barrier, 8, vec![(system_configuration, PredecessorRole::SamePublish)]));
        previous_root_slot = Some(root_slot);
        previous_system_configuration = Some(system_configuration);
        publish_final_barrier.push(final_barrier);
    }
    WaitPlan { nodes, transaction_publish, transaction_record_writes, publish_final_barrier }
}

struct WaitTimeline {
    ready: Vec<u64>,
    start: Vec<u64>,
    complete: Vec<u64>,
    duration: Vec<u64>,
    /// 同一服务台上紧挨在前面处理的那一步。
    server_previous: Vec<Option<usize>>,
    /// 设备上的处理次序。
    device_order: Vec<usize>,
    merged_barriers: u64,
}

/// 离散事件：每个服务台先进先出（按发出时刻，同刻按事务号、再按事务里的先后，读法 ②）；一步在「它发出」与「同一服务台前一步完成」两者较晚时开始。
fn simulate_wait(plan: &WaitPlan, costs: &WaitCosts) -> WaitTimeline {
    let node_count = plan.nodes.len();
    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    let mut unresolved: Vec<usize> = plan.nodes.iter().map(|node| node.predecessors.len()).collect();
    for (node_index, node) in plan.nodes.iter().enumerate() {
        for (predecessor, _) in &node.predecessors {
            successors[*predecessor].push(node_index);
        }
    }
    let mut ready: Vec<Option<u64>> = vec![None; node_count];
    let mut start = vec![0u64; node_count];
    let mut complete: Vec<Option<u64>> = vec![None; node_count];
    let mut duration = vec![0u64; node_count];
    let mut server_previous: Vec<Option<usize>> = vec![None; node_count];
    let mut device_order: Vec<usize> = Vec::new();
    let mut merged_barriers = 0u64;
    let mut queues: [BTreeSet<(u64, u32, u32, usize)>; 3] = [BTreeSet::new(), BTreeSet::new(), BTreeSet::new()];
    let mut in_progress: [Option<usize>; 3] = [None; 3];
    let mut busy_until = [0u64; 3];
    let mut last_on_server: [Option<usize>; 3] = [None; 3];
    let ready_time_of = |node_index: usize, complete: &[Option<u64>]| -> u64 {
        let node = &plan.nodes[node_index];
        let from_predecessors = node.predecessors.iter().map(|(predecessor, _)| complete[*predecessor].expect("前驱都完成了")).max();
        let from_source = node.ready_source;
        from_predecessors.into_iter().chain(from_source).max().expect("每一步不是有前驱就是有就绪时刻")
    };
    for node_index in 0..node_count {
        if unresolved[node_index] == 0 {
            let time = ready_time_of(node_index, &complete);
            ready[node_index] = Some(time);
            let node = &plan.nodes[node_index];
            queues[node.kind.server().position()].insert((time, node.transaction, node.order_within_transaction, node_index));
        }
    }
    loop {
        let next_completion = in_progress.iter().flatten().map(|node_index| (complete[*node_index].expect("在做的那一步有完成时刻"), *node_index)).min();
        let next_start = (0..3)
            .filter(|server| in_progress[*server].is_none())
            .filter_map(|server| queues[server].first().map(|(ready_time, ..)| ((*ready_time).max(busy_until[server]), server)))
            .min();
        // 同一时刻先把完成的放出去，再开始新的一步：同刻发出的几步才按事务号排得到。
        match (next_completion, next_start) {
            (None, None) => break,
            (Some((completion_time, node_index)), start_option) if start_option.is_none_or(|(start_time, _)| completion_time <= start_time) => {
                let server = plan.nodes[node_index].kind.server().position();
                in_progress[server] = None;
                for successor in &successors[node_index] {
                    unresolved[*successor] -= 1;
                    if unresolved[*successor] == 0 {
                        let time = ready_time_of(*successor, &complete);
                        ready[*successor] = Some(time);
                        let node = &plan.nodes[*successor];
                        queues[node.kind.server().position()].insert((time, node.transaction, node.order_within_transaction, *successor));
                    }
                }
            }
            (_, Some((start_time, server))) => {
                let (_, _, _, node_index) = queues[server].pop_first().expect("刚看过非空");
                let kind = plan.nodes[node_index].kind;
                // 读法 ⑤：连着的屏障（设备上紧挨在前的一步也是屏障，中间没有写）不占时间，跨发布也合并。
                let follows_a_barrier = last_on_server[server].is_some_and(|previous| plan.nodes[previous].kind == WaitStepKind::Barrier);
                let step_duration = if kind == WaitStepKind::Barrier && follows_a_barrier { 0 } else { wait_step_cost(kind, costs) };
                merged_barriers += u64::from(kind == WaitStepKind::Barrier && step_duration == 0);
                start[node_index] = start_time;
                duration[node_index] = step_duration;
                complete[node_index] = Some(start_time + step_duration);
                server_previous[node_index] = last_on_server[server];
                last_on_server[server] = Some(node_index);
                busy_until[server] = start_time + step_duration;
                in_progress[server] = Some(node_index);
                if server == WaitServer::Device.position() {
                    device_order.push(node_index);
                }
            }
            (Some(_), None) => panic!("有在做的一步却没放出去"),
        }
    }
    assert!(unresolved.iter().all(|count| *count == 0), "每一步的前驱都完成了：次序约束没有卡死");
    WaitTimeline {
        ready: ready.into_iter().map(|time| time.expect("每一步都发出了")).collect(),
        start,
        complete: complete.into_iter().map(|time| time.expect("每一步都做完了")).collect(),
        duration,
        server_previous,
        device_order,
        merged_barriers,
    }
}

/// 关键路径上的一个点：一步的完成，或一步的发出（R1 的「已发出」）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WaitPathPoint {
    CompletionOf(usize),
    IssueOf(usize),
}

/// 读法 ③：设备上在这条写之后处理的第一道屏障或 FUA（先进先出，它就是这条写完成之后最早完成的那一道，跨发布也算）。
fn first_covering_operation_after(plan: &WaitPlan, timeline: &WaitTimeline, write: usize) -> usize {
    let position = timeline.device_order.iter().position(|node_index| *node_index == write).expect("这条写在设备上做过");
    *timeline.device_order[position + 1..]
        .iter()
        .find(|node_index| plan.nodes[**node_index].kind.covers_earlier_writes())
        .expect("每次发布最后都有屏障")
}

/// D13（验证路线） 已定项 2 条件 ② 的「已追加」，每个事务一项：（时刻，关键路径从哪个点往回追）。
fn appended_events(plan: &WaitPlan, timeline: &WaitTimeline, reading: AppendedReading) -> Vec<(u64, WaitPathPoint)> {
    plan.transaction_record_writes
        .iter()
        .map(|writes| match reading {
            AppendedReading::R1Issued => {
                let last_issued = *writes.iter().max_by_key(|write| (timeline.ready[**write], **write)).expect("r ≥ 1");
                (timeline.ready[last_issued], WaitPathPoint::IssueOf(last_issued))
            }
            AppendedReading::R2IssuedAndCovered => {
                let last_record = *writes.iter().max_by_key(|write| timeline.start[**write]).expect("r ≥ 1");
                let cover = first_covering_operation_after(plan, timeline, last_record);
                (timeline.complete[cover], WaitPathPoint::CompletionOf(cover))
            }
        })
        .collect()
}

fn point_role(plan: &WaitPlan, point: WaitPathPoint, own_publish: usize) -> PredecessorRole {
    let node_index = match point {
        WaitPathPoint::CompletionOf(node_index) | WaitPathPoint::IssueOf(node_index) => node_index,
    };
    if plan.nodes[node_index].publish_position == own_publish {
        PredecessorRole::SamePublish
    } else {
        PredecessorRole::CrossPublish
    }
}

/// 一个事务的返回：时刻与它的候选前驱（本发布 B3 的完成；每个 j < i 的「已追加」）。
fn return_candidates(plan: &WaitPlan, timeline: &WaitTimeline, transaction: u32, appended: &[(u64, WaitPathPoint)]) -> Vec<(u64, PredecessorRole, Option<WaitPathPoint>)> {
    let own_publish = plan.transaction_publish[transaction_position(transaction)];
    let final_barrier = plan.publish_final_barrier[own_publish];
    let mut candidates = vec![(timeline.complete[final_barrier], PredecessorRole::SamePublish, Some(WaitPathPoint::CompletionOf(final_barrier)))];
    for (time, point) in &appended[..transaction_position(transaction)] {
        candidates.push((*time, point_role(plan, *point, own_publish), Some(*point)));
    }
    candidates
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CriticalPath {
    barriers_or_forced_unit_access: u64,
    device_writes: u64,
    cpu_steps: u64,
    merged_barriers: u64,
    tie_steps: u64,
    source_time: u64,
    duration_sum: u64,
}

/// 读法 ④：取定下这一刻的那个前驱；几个前驱同刻时按角色的声明次序取，同一步以几个角色出现时按最靠前的那个算，同刻的步数照报。
fn choose_predecessor(candidates: &[(u64, PredecessorRole, Option<WaitPathPoint>)], expected_time: u64, path: &mut CriticalPath) -> Option<WaitPathPoint> {
    let latest = candidates.iter().map(|(time, ..)| *time).max().expect("至少一个候选");
    assert_eq!(latest, expected_time, "这一刻由某个前驱定下");
    let mut at_latest: Vec<(PredecessorRole, Option<WaitPathPoint>)> = candidates.iter().filter(|(time, ..)| *time == latest).map(|(_, role, point)| (*role, *point)).collect();
    at_latest.sort_by_key(|(role, _)| *role);
    let mut distinct_points: Vec<Option<WaitPathPoint>> = Vec::new();
    for (_, point) in &at_latest {
        if !distinct_points.contains(point) {
            distinct_points.push(*point);
        }
    }
    path.tie_steps += u64::from(distinct_points.len() > 1);
    at_latest[0].1
}

fn critical_path_of(plan: &WaitPlan, timeline: &WaitTimeline, transaction: u32, appended: &[(u64, WaitPathPoint)], return_time: u64) -> CriticalPath {
    let mut path = CriticalPath::default();
    let mut point = choose_predecessor(&return_candidates(plan, timeline, transaction, appended), return_time, &mut path);
    let mut time_at_point = return_time;
    while let Some(current) = point {
        let (node_index, include_server_queue) = match current {
            WaitPathPoint::CompletionOf(node_index) => (node_index, true),
            WaitPathPoint::IssueOf(node_index) => (node_index, false),
        };
        let node = &plan.nodes[node_index];
        let expected_time = if include_server_queue {
            assert_eq!(timeline.complete[node_index], time_at_point, "路径上的完成时刻接得上");
            let step_duration = timeline.duration[node_index];
            path.duration_sum += step_duration;
            match (node.kind.path_component(), step_duration) {
                (PathComponent::BarrierOrForcedUnitAccess, 0) => path.merged_barriers += 1,
                (PathComponent::BarrierOrForcedUnitAccess, _) => path.barriers_or_forced_unit_access += 1,
                (PathComponent::DeviceWrite, _) => path.device_writes += 1,
                (PathComponent::CpuStep, _) => path.cpu_steps += 1,
            }
            timeline.start[node_index]
        } else {
            timeline.ready[node_index]
        };
        let mut candidates: Vec<(u64, PredecessorRole, Option<WaitPathPoint>)> = node
            .predecessors
            .iter()
            .map(|(predecessor, role)| (timeline.complete[*predecessor], *role, Some(WaitPathPoint::CompletionOf(*predecessor))))
            .collect();
        if let Some(source_time) = node.ready_source {
            candidates.push((source_time, PredecessorRole::ReadySource, None));
        }
        if include_server_queue {
            if let Some(previous) = timeline.server_previous[node_index] {
                candidates.push((timeline.complete[previous], node.kind.server().queue_role(), Some(WaitPathPoint::CompletionOf(previous))));
            }
        }
        point = choose_predecessor(&candidates, expected_time, &mut path);
        time_at_point = expected_time;
    }
    path.source_time = time_at_point;
    path
}

/// 登记 8.1 的轨迹：（初始值，吃光初始供给之后的峰值，值为正的事件步数，期末值）。事件步 = 时刻 0 与每一步的完成时刻（去重），截到期末那一刻为止（修订 4 第 6 条）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WaitTrajectory {
    initial: u64,
    peak_after_drain: u64,
    positive_steps: u64,
    final_value: u64,
}

fn trajectory_over(event_times: &[u64], end_time: u64, value_at: &dyn Fn(u64) -> u64) -> WaitTrajectory {
    let mut trajectory = WaitTrajectory { initial: value_at(0), peak_after_drain: 0, positive_steps: 0, final_value: 0 };
    let mut drained = false;
    for time in event_times.iter().copied().filter(|time| *time <= end_time) {
        let value = value_at(time);
        trajectory.positive_steps += u64::from(value > 0);
        if drained {
            trajectory.peak_after_drain = trajectory.peak_after_drain.max(value);
        }
        drained |= value == 0;
    }
    trajectory.final_value = value_at(end_time);
    trajectory
}

#[derive(Clone, Debug)]
struct WaitResult {
    return_times: Vec<u64>,
    w_max: u64,
    latest_transaction: u32,
    path: CriticalPath,
    pending: WaitTrajectory,
    /// 乙整段：已预留而还没写的槽里 jsn 最小那个与已写的最大 jsn 之差（甲两条臂为 None）。
    reserved_lag: Option<WaitTrajectory>,
    merged_barriers_on_device: u64,
}

impl WaitResult {
    fn w_sum(&self) -> u64 {
        self.return_times.iter().sum()
    }
}

fn evaluate_wait(scenario: &WaitScenario, reading: AppendedReading) -> WaitResult {
    let plan = plan_wait_scenario(scenario);
    let costs = scenario.point.costs();
    let timeline = simulate_wait(&plan, &costs);
    let appended = appended_events(&plan, &timeline, reading);
    let return_times: Vec<u64> = (1..=scenario.transaction_count)
        .map(|transaction| return_candidates(&plan, &timeline, transaction, &appended).iter().map(|(time, ..)| *time).max().expect("至少本发布的 B3"))
        .collect();
    let w_max = *return_times.iter().max().expect("k ≥ 1");
    // 同时最晚返回的取事务号最大的那个（修订 4 第 2 条）。
    let latest_transaction = (1..=scenario.transaction_count).rev().find(|transaction| return_times[transaction_position(*transaction)] == w_max).expect("有最晚的");
    let path = critical_path_of(&plan, &timeline, latest_transaction, &appended, w_max);
    assert_eq!(path.duration_sum + path.source_time, w_max, "关键路径上各步耗时之和加起点时刻等于返回时刻");
    let mut event_times: Vec<u64> = std::iter::once(0).chain(timeline.complete.iter().copied()).collect();
    event_times.sort_unstable();
    event_times.dedup();
    let appended_times: Vec<u64> = appended[..transaction_position(latest_transaction)].iter().map(|(time, _)| *time).collect();
    let pending = trajectory_over(&event_times, w_max, &|time| u64::try_from(appended_times.iter().filter(|appended| **appended > time).count()).expect("至多 63"));
    let reserved_lag = match scenario.append_arm {
        AppendArm::YiReservedWhole => {
            let record_count = u64::from(scenario.record_count);
            // (jsn, 预留完成时刻, 发出时刻)：Ti 预留 [(i − 1)·r + 1, i·r]。
            let slots: Vec<(u64, u64, u64)> = (1..=scenario.transaction_count)
                .flat_map(|transaction| {
                    let reservation = plan.nodes.iter().position(|node| node.kind == WaitStepKind::ReservationStep && node.transaction == transaction).expect("乙整段每个事务都预留");
                    let reserved_at = timeline.complete[reservation];
                    plan.transaction_record_writes[transaction_position(transaction)]
                        .iter()
                        .enumerate()
                        .map(move |(record_position, write)| (u64::from(transaction - 1) * record_count + u64::try_from(record_position).expect("r ≤ 2") + 1, reserved_at, *write))
                        .collect::<Vec<_>>()
                })
                .map(|(jsn, reserved_at, write)| (jsn, reserved_at, timeline.ready[write]))
                .collect();
            let lag_at = |time: u64| -> u64 {
                let largest_written = slots.iter().filter(|(_, _, issued_at)| *issued_at <= time).map(|(jsn, ..)| *jsn).max();
                let smallest_reserved_unwritten = slots.iter().filter(|(_, reserved_at, issued_at)| *reserved_at <= time && *issued_at > time).map(|(jsn, ..)| *jsn).min();
                match (largest_written, smallest_reserved_unwritten) {
                    (Some(written), Some(unwritten)) => written.saturating_sub(unwritten),
                    (Some(_), None) | (None, Some(_)) | (None, None) => 0,
                }
            };
            Some(trajectory_over(&event_times, w_max, &lag_at))
        }
        AppendArm::JiaWhole | AppendArm::JiaInterleaved | AppendArm::DingNoDiscipline => None,
    };
    WaitResult { return_times, w_max, latest_transaction, path, pending, reserved_lag, merged_barriers_on_device: timeline.merged_barriers }
}

fn format_wait_units(units: u64) -> String {
    format!("{}.{:03}", units / WAIT_UNITS_PER_ABSTRACT_UNIT, units % WAIT_UNITS_PER_ABSTRACT_UNIT)
}

/// 分数按四位小数四舍五入打出来（只给人看；判定用整数乘法比）。
fn format_ratio(numerator: u64, denominator: u64) -> String {
    let scaled = (numerator * 10_000 + denominator / 2) / denominator;
    format!("{}.{:04}", scaled / 10_000, scaled % 10_000)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GrowthJudgement {
    Grows,
    Bounded,
    Between,
}

impl GrowthJudgement {
    fn label(self) -> &'static str {
        match self {
            GrowthJudgement::Grows => "grows",
            GrowthJudgement::Bounded => "bounded",
            GrowthJudgement::Between => "between_not_judged",
        }
    }
}

/// ρ = large ÷ small 与两道门槛比：ρ ≥ grows ⇔ large·grows分母 ≥ grows分子·small。
fn growth_judgement(small_k_w_max: u64, large_k_w_max: u64, grows_threshold: (u64, u64), bounded_threshold: (u64, u64)) -> GrowthJudgement {
    if large_k_w_max * grows_threshold.1 >= grows_threshold.0 * small_k_w_max {
        GrowthJudgement::Grows
    } else if large_k_w_max * bounded_threshold.1 <= bounded_threshold.0 * small_k_w_max {
        GrowthJudgement::Bounded
    } else {
        GrowthJudgement::Between
    }
}

/// 一格等待模型的键：（Q1 条件，Q2 臂，剖面，取样点，r，读法，k）。
type WaitKey = (PublishCondition, AppendArm, PreparationProfile, CostPoint, u32, AppendedReading, u32);
/// 一条判决的键：等待格的键去掉 k。
type WaitVerdictKey = (PublishCondition, AppendArm, PreparationProfile, CostPoint, u32, AppendedReading);

fn wait_scenario_of(key: WaitKey) -> WaitScenario {
    let (condition, append_arm, profile, point, record_count, _, transaction_count) = key;
    WaitScenario { condition, append_arm, profile, point, record_count, transaction_count }
}

fn wait_grid() -> BTreeMap<WaitKey, WaitResult> {
    let mut grid = BTreeMap::new();
    for condition in WAIT_CONDITIONS {
        for arm in WAIT_APPEND_ARMS {
            for profile in PreparationProfile::ALL {
                for point in CostPoint::ALL {
                    for record_count in WAIT_RECORD_COUNTS {
                        for reading in AppendedReading::ALL {
                            for transaction_count in WAIT_TRANSACTION_COUNTS {
                                let key = (condition, arm, profile, point, record_count, reading, transaction_count);
                                grid.insert(key, evaluate_wait(&wait_scenario_of(key), reading));
                            }
                        }
                    }
                }
            }
        }
    }
    grid
}

/// 登记第七节 B6：k = 1、r = 1：甲两条臂 W = 4w + 4b + h，乙整段 W = 4w + 4b（g ≤ 2w + b），慢头各加 Δ。
fn anchor_b6_single_transaction(arm: AppendArm, profile: PreparationProfile, point: CostPoint) -> u64 {
    let costs = point.costs();
    let appender = match arm {
        AppendArm::JiaWhole | AppendArm::JiaInterleaved => costs.cpu_step,
        AppendArm::YiReservedWhole | AppendArm::DingNoDiscipline => 0,
    };
    4 * costs.write + 4 * costs.barrier_or_forced_unit_access + appender + profile.ready_time_of(1, &costs)
}

/// 登记第七节 B6：k = 2、r = 1、串落、等速、P1：甲整段 8.41、乙整段 8.4（写成整数单位）。
const ANCHOR_B6_SERIAL_TWO_TRANSACTIONS: [(AppendArm, u64); 2] = [(AppendArm::JiaWhole, 8410), (AppendArm::YiReservedWhole, 8400)];

fn emit_wait_anchors(emitter: &mut Emitter, grid: &BTreeMap<WaitKey, WaitResult>) -> u64 {
    let mut checked = 0u64;
    let mut mismatches = 0u64;
    let mut check = |emitter: &mut Emitter, key: WaitKey, expected: u64| {
        let measured = grid[&key].w_max;
        checked += 1;
        mismatches += u64::from(measured != expected);
        let (condition, arm, profile, point, record_count, reading, transaction_count) = key;
        emit(
            emitter,
            &format!(
                "name=wait_anchor family=B6 q1={} q2={} profile={} point={} r={record_count} reading={} k={transaction_count} expected_w_max={} measured_w_max={} matches={}",
                condition.label(),
                arm.label(),
                profile.label(),
                point.label(),
                reading.label(),
                format_wait_units(expected),
                format_wait_units(measured),
                yes_or_no(measured == expected)
            ),
        );
    };
    for condition in WAIT_CONDITIONS {
        for arm in WAIT_APPEND_ARMS {
            for profile in PreparationProfile::ALL {
                for point in CostPoint::ALL {
                    for reading in AppendedReading::ALL {
                        check(emitter, (condition, arm, profile, point, 1, reading, 1), anchor_b6_single_transaction(arm, profile, point));
                    }
                }
            }
        }
    }
    for (arm, expected) in ANCHOR_B6_SERIAL_TWO_TRANSACTIONS {
        for reading in AppendedReading::ALL {
            check(emitter, (PublishCondition::SerialPublishes, arm, PreparationProfile::Uniform, CostPoint::DeviceDominated, 1, reading, 2), expected);
        }
    }
    emit(emitter, &format!("name=wait_anchor_summary families=B6 checked={checked} mismatches={mismatches}"));
    mismatches
}

fn wait_verdict_key_text(key: WaitVerdictKey) -> String {
    let (condition, arm, profile, point, record_count, reading) = key;
    format!("q1={} q2={} profile={} point={} r={record_count} reading={}", condition.label(), arm.label(), profile.label(), point.label(), reading.label())
}

fn emit_wait_cell(emitter: &mut Emitter, key: WaitKey, result: &WaitResult) {
    let (condition, arm, profile, point, record_count, reading, transaction_count) = key;
    let lag = match result.reserved_lag {
        Some(lag) => format!("reserved_lag_initial={} reserved_lag_peak_after_drain={} reserved_lag_positive_steps={} reserved_lag_final={}", lag.initial, lag.peak_after_drain, lag.positive_steps, lag.final_value),
        None => "reserved_lag=not_applicable".to_string(),
    };
    emit(
        emitter,
        &format!(
            "name=wait_cell {} k={transaction_count} w_max={} w_mean={} latest_transaction={} n_bar={} n_dev={} n_cpu={} merged_barriers_on_path={} merged_barriers_on_device={} tie_steps={} path_source_time={} pending_initial={} pending_peak_after_drain={} pending_positive_steps={} pending_final={} {lag}",
            wait_verdict_key_text((condition, arm, profile, point, record_count, reading)),
            format_wait_units(result.w_max),
            format_ratio(result.w_sum(), u64::from(transaction_count) * WAIT_UNITS_PER_ABSTRACT_UNIT),
            result.latest_transaction,
            result.path.barriers_or_forced_unit_access,
            result.path.device_writes,
            result.path.cpu_steps,
            result.path.merged_barriers,
            result.merged_barriers_on_device,
            result.path.tie_steps,
            format_wait_units(result.path.source_time),
            result.pending.initial,
            result.pending.peak_after_drain,
            result.pending.positive_steps,
            result.pending.final_value
        ),
    );
}

/// Q2-4：k = 64 的分量大于 k = 8 的分量 ⇒ 这个分量随 k 长。返回（n_bar，n_dev，n_cpu）各自长没长。
fn growing_components(small: &WaitResult, large: &WaitResult) -> [(&'static str, u64, u64, bool); 3] {
    [
        ("n_bar", small.path.barriers_or_forced_unit_access, large.path.barriers_or_forced_unit_access),
        ("n_dev", small.path.device_writes, large.path.device_writes),
        ("n_cpu", small.path.cpu_steps, large.path.cpu_steps),
    ]
    .map(|(name, at_small, at_large)| (name, at_small, at_large, at_large > at_small))
}

fn growing_component_names(components: &[(&'static str, u64, u64, bool); 3]) -> Vec<&'static str> {
    components.iter().filter(|(_, _, _, grows)| *grows).map(|(name, ..)| *name).collect()
}

fn names_or_none(names: &[&'static str]) -> String {
    if names.is_empty() {
        "none".to_string()
    } else {
        names.join(",")
    }
}

fn wait_pair_of_grid(grid: &BTreeMap<WaitKey, WaitResult>, key: WaitVerdictKey) -> (&WaitResult, &WaitResult) {
    let (condition, arm, profile, point, record_count, reading) = key;
    (
        &grid[&(condition, arm, profile, point, record_count, reading, GROWTH_RATIO_SMALL_TRANSACTION_COUNT)],
        &grid[&(condition, arm, profile, point, record_count, reading, GROWTH_RATIO_LARGE_TRANSACTION_COUNT)],
    )
}

fn registered_growth_judgement(grid: &BTreeMap<WaitKey, WaitResult>, key: WaitVerdictKey) -> GrowthJudgement {
    let (small, large) = wait_pair_of_grid(grid, key);
    growth_judgement(small.w_max, large.w_max, REGISTERED_GROWS_THRESHOLD, REGISTERED_BOUNDED_THRESHOLD)
}

fn every_wait_verdict_key() -> Vec<WaitVerdictKey> {
    let mut keys = Vec::new();
    for condition in WAIT_CONDITIONS {
        for arm in WAIT_APPEND_ARMS {
            for profile in PreparationProfile::ALL {
                for point in CostPoint::ALL {
                    for record_count in WAIT_RECORD_COUNTS {
                        for reading in AppendedReading::ALL {
                            keys.push((condition, arm, profile, point, record_count, reading));
                        }
                    }
                }
            }
        }
    }
    keys
}

/// 第六节 Q2-3（ρ 与判定）与 Q2-4（三个分量各自长没长），每个（Q1 条件，Q2 臂，剖面，取样点，r，读法）各一行。
fn emit_wait_verdicts(emitter: &mut Emitter, grid: &BTreeMap<WaitKey, WaitResult>) {
    for key in every_wait_verdict_key() {
        let (small, large) = wait_pair_of_grid(grid, key);
        emit(
            emitter,
            &format!(
                "name=wait_verdict quantity=Q2-3 {} w_max_k8={} w_max_k64={} rho={} judgement={}",
                wait_verdict_key_text(key),
                format_wait_units(small.w_max),
                format_wait_units(large.w_max),
                format_ratio(large.w_max, small.w_max),
                registered_growth_judgement(grid, key).label()
            ),
        );
        let components = growing_components(small, large);
        let component_text: Vec<String> = components.iter().map(|(name, at_small, at_large, grows)| format!("{name}_k8={at_small} {name}_k64={at_large} {name}_grows={}", yes_or_no(*grows))).collect();
        emit(
            emitter,
            &format!("name=wait_verdict quantity=Q2-4 {} {} growing={}", wait_verdict_key_text(key), component_text.join(" "), names_or_none(&growing_component_names(&components))),
        );
    }
}

/// 第十节 F9、F10：同一（Q1 条件，剖面，取样点）下两条 Q2 臂的 Q2-3 判定配对（单一追加者的两种读法各与乙整段配一次）。
fn emit_wait_pairs(emitter: &mut Emitter, grid: &BTreeMap<WaitKey, WaitResult>) {
    let pairs = [(AppendArm::JiaWhole, AppendArm::YiReservedWhole), (AppendArm::JiaInterleaved, AppendArm::YiReservedWhole)];
    for condition in WAIT_CONDITIONS {
        for profile in PreparationProfile::ALL {
            for point in CostPoint::ALL {
                for record_count in WAIT_RECORD_COUNTS {
                    for reading in AppendedReading::ALL {
                        for (first, second) in pairs {
                            let first_key = (condition, first, profile, point, record_count, reading);
                            let second_key = (condition, second, profile, point, record_count, reading);
                            let first_judgement = registered_growth_judgement(grid, first_key);
                            let second_judgement = registered_growth_judgement(grid, second_key);
                            let (first_small, first_large) = wait_pair_of_grid(grid, first_key);
                            let (second_small, second_large) = wait_pair_of_grid(grid, second_key);
                            let first_growing = growing_component_names(&growing_components(first_small, first_large));
                            let second_growing = growing_component_names(&growing_components(second_small, second_large));
                            let shared_growing: Vec<&'static str> = first_growing.iter().filter(|name| second_growing.contains(*name)).copied().collect();
                            let clause = match (first_judgement, second_judgement) {
                                (GrowthJudgement::Grows, GrowthJudgement::Bounded) => "f10_first_arm_flips",
                                (GrowthJudgement::Bounded, GrowthJudgement::Grows) => "f10_second_arm_flips",
                                (GrowthJudgement::Grows, GrowthJudgement::Grows) => "f9_both_grow",
                                (
                                    GrowthJudgement::Grows | GrowthJudgement::Bounded | GrowthJudgement::Between,
                                    GrowthJudgement::Grows | GrowthJudgement::Bounded | GrowthJudgement::Between,
                                ) => "neither_f9_nor_f10",
                            };
                            emit(
                                emitter,
                                &format!(
                                    "name=wait_pair q1={} profile={} point={} r={record_count} reading={} first={} second={} first_judgement={} second_judgement={} first_growing={} second_growing={} shared_growing={} clause={clause}",
                                    condition.label(),
                                    profile.label(),
                                    point.label(),
                                    reading.label(),
                                    first.label(),
                                    second.label(),
                                    first_judgement.label(),
                                    second_judgement.label(),
                                    names_or_none(&first_growing),
                                    names_or_none(&second_growing),
                                    names_or_none(&shared_growing)
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
}

/// 登记 8.2 的 G3：主取样点 P1、等速、r = 1；方向相反的取样点 P2、慢头、r = 2（一次换一个旋钮）。
fn emit_wait_geometry(emitter: &mut Emitter, grid: &BTreeMap<WaitKey, WaitResult>) -> u64 {
    let mut unstable_lines = 0u64;
    let opposite_points: [(&str, PreparationProfile, CostPoint, u32); 3] = [
        ("p2_cpu_dominated", PreparationProfile::Uniform, CostPoint::CpuDominated, 1),
        ("slow_head", PreparationProfile::SlowHead, CostPoint::DeviceDominated, 1),
        ("r2", PreparationProfile::Uniform, CostPoint::DeviceDominated, 2),
    ];
    for condition in WAIT_CONDITIONS {
        for arm in WAIT_APPEND_ARMS {
            for reading in AppendedReading::ALL {
                let main_key = (condition, arm, PreparationProfile::Uniform, CostPoint::DeviceDominated, 1, reading);
                let main_judgement = registered_growth_judgement(grid, main_key);
                let (main_small, main_large) = wait_pair_of_grid(grid, main_key);
                let main_growing = growing_component_names(&growing_components(main_small, main_large));
                for (opposite_label, profile, point, record_count) in opposite_points {
                    let opposite_key = (condition, arm, profile, point, record_count, reading);
                    let opposite_judgement = registered_growth_judgement(grid, opposite_key);
                    let (opposite_small, opposite_large) = wait_pair_of_grid(grid, opposite_key);
                    let opposite_growing = growing_component_names(&growing_components(opposite_small, opposite_large));
                    let stable = main_judgement == opposite_judgement && main_growing == opposite_growing;
                    unstable_lines += u64::from(!stable);
                    emit(
                        emitter,
                        &format!(
                            "name=wait_geometry id=G3 q1={} q2={} reading={} main=p1_uniform_r1 opposite={opposite_label} main_q2_3={} opposite_q2_3={} main_q2_4_growing={} opposite_q2_4_growing={} stability={}",
                            condition.label(),
                            arm.label(),
                            reading.label(),
                            main_judgement.label(),
                            opposite_judgement.label(),
                            names_or_none(&main_growing),
                            names_or_none(&opposite_growing),
                            if stable { "stable" } else { "unstable" }
                        ),
                    );
                }
            }
        }
    }
    unstable_lines
}

fn run_segment_three() {
    let mut emitter = Emitter::new();
    let cost_text = |point: CostPoint| {
        let costs = point.costs();
        format!(
            "{}=w{},b{},h_or_g{},delta{}",
            point.label(),
            format_wait_units(costs.write),
            format_wait_units(costs.barrier_or_forced_unit_access),
            format_wait_units(costs.cpu_step),
            format_wait_units(costs.slow_head_delay)
        )
    };
    emit(
        &mut emitter,
        &format!(
            "name=config experiment=e169 segment=3 model=fsync_wait q1=serial_publishes,shared_publish,multi_flight_roots_ordered,multi_flight_roots_unordered q2=jia_whole,jia_interleaved,yi_reserved_whole ding_control=no_wait_rule_registered profiles=uniform,slow_head readings=r2_covered,r1_issued k=1,2,4,8,16,32,64 r=1,2 histories=H0 {} {} rho=w_max_k64_over_w_max_k8 grows_threshold=2 bounded_threshold=1.25",
            cost_text(CostPoint::DeviceDominated),
            cost_text(CostPoint::CpuDominated)
        ),
    );
    let grid = wait_grid();
    emit_wait_anchors(&mut emitter, &grid);
    for (key, result) in &grid {
        emit_wait_cell(&mut emitter, *key, result);
    }
    emit_wait_verdicts(&mut emitter, &grid);
    emit_wait_pairs(&mut emitter, &grid);
    let unstable_lines = emit_wait_geometry(&mut emitter, &grid);
    emit(&mut emitter, &format!("name=wait_geometry_summary id=G3 lines={} unstable_lines={unstable_lines}", WAIT_CONDITIONS.len() * WAIT_APPEND_ARMS.len() * AppendedReading::ALL.len() * 3));
    // 第八节 8.1：期末值必须是 0，不为 0 判装置错；关键路径之和在 `evaluate_wait` 里逐格断言过。
    let cells = grid.len();
    let pending_final_nonzero = grid.values().filter(|result| result.pending.final_value > 0).count();
    let pending_rebound = grid.values().filter(|result| result.pending.peak_after_drain > 0).count();
    let lag_cells = grid.values().filter(|result| result.reserved_lag.is_some()).count();
    let lag_final_nonzero = grid.values().filter(|result| result.reserved_lag.is_some_and(|lag| lag.final_value > 0)).count();
    let lag_peak = grid.values().filter_map(|result| result.reserved_lag.map(|lag| lag.peak_after_drain.max(lag.initial))).max().unwrap_or(0);
    emit(&mut emitter, &format!("name=wait_assertion check=critical_path_sum cells={cells} mismatches=0"));
    emit(&mut emitter, &format!("name=wait_assertion check=pending_trajectory cells={cells} pending_final_nonzero={pending_final_nonzero} pending_peak_after_drain_nonzero={pending_rebound}"));
    emit(&mut emitter, &format!("name=wait_assertion check=reserved_lag_trajectory cells={lag_cells} reserved_lag_final_nonzero={lag_final_nonzero} reserved_lag_largest_initial_or_rebound={lag_peak}"));
    for condition in WAIT_CONDITIONS {
        let judgements: Vec<GrowthJudgement> = every_wait_verdict_key().into_iter().filter(|key| key.0 == condition).map(|key| registered_growth_judgement(&grid, key)).collect();
        let count_of = |wanted: GrowthJudgement| judgements.iter().filter(|judgement| **judgement == wanted).count();
        emit(
            &mut emitter,
            &format!(
                "name=wait_summary quantity=Q2-3 q1={} verdict_lines={} grows={} bounded={} between_not_judged={}",
                condition.label(),
                judgements.len(),
                count_of(GrowthJudgement::Grows),
                count_of(GrowthJudgement::Bounded),
                count_of(GrowthJudgement::Between)
            ),
        );
    }
    println!("{}", emitter.finish());
}

// ===== 路径 3 对路径 4 的直接对拍：把第一、二段每个崩溃状态的恢复输入与模型丙0 的结果打出来，交给按路径依赖 `replay_core` 的临时 crate 逐份比 =====

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum ReplayCoreInputKind {
    /// 这个崩溃状态全部持久的记录（模型的丙0 在实验里吃的就是这一份）。
    AllDurableRecords,
    /// 这个崩溃状态的严格连续段（登记第一节 1.2；派发提示点名的那一份）。
    StrictSegment,
}

impl ReplayCoreInputKind {
    fn label(self) -> &'static str {
        match self {
            ReplayCoreInputKind::AllDurableRecords => "all_durable",
            ReplayCoreInputKind::StrictSegment => "strict_segment",
        }
    }
}

/// (jsn, txg, 事务号, 提交标记, 末条标志, 本次发布内序号, 点名单元验得过)
type ReplayCoreRecordKey = (u64, u64, u32, bool, bool, u32, bool);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Bing0Result {
    core_failure: bool,
    applied_records: usize,
    last_applied_jsn: Option<u64>,
    maximum_applied_transaction: u32,
}

fn cell_crash_states(request: &CellRequest) -> Option<HashSet<CrashState>> {
    if !cell_is_applicable(request.append_arm, &request.history) {
        return None;
    }
    let layout = plan_cell(request.condition, request.append_arm, &request.history, request.discipline);
    if request.condition.publishes_fly_together() {
        return Some(enumerate_multi_flight(request, &layout, MultiFlightEnumeration::registered_for(&request.history)).states);
    }
    let mut states = HashSet::new();
    for execution in build_executions(&layout, request.append_arm, request.discipline, request.barriers) {
        collect_crash_states(&execution, request.reading, request.return_rule, &mut states);
    }
    Some(states)
}

fn run_replay_core_inputs() {
    let mut emitter = Emitter::new();
    let mut inputs: BTreeMap<(ReplayCoreInputKind, u64, Vec<ReplayCoreRecordKey>), (u64, Bing0Result)> = BTreeMap::new();
    let segments: [(&str, [PublishCondition; 2], Vec<History>); 2] =
        [("one", PublishCondition::SEGMENT_ONE, all_histories()), ("two", PublishCondition::SEGMENT_TWO, segment_two_histories())];
    let mut state_counts: Vec<(&str, u64, u64)> = Vec::new();
    for (segment_label, conditions, histories) in &segments {
        let mut cells = 0u64;
        let mut states_in_segment = 0u64;
        for condition in conditions {
            for arm in AppendArm::ALL {
                for history in histories {
                    for reading in AppendedReading::ALL {
                        let Some(states) = cell_crash_states(&CellRequest::main_grid(*condition, arm, history.clone(), reading)) else {
                            continue;
                        };
                        cells += 1;
                        states_in_segment += u64::try_from(states.len()).expect("状态数有限");
                        for state in &states {
                            let view = StateView::of(state);
                            let root_txg = view.chosen_root_txg();
                            let verify = |record: &RecordContent| named_units_are_verified(record, &view.durable_new);
                            let strict_segment = strict_continuous_segment(&view.records_by_jsn, root_txg);
                            for (kind, records) in [(ReplayCoreInputKind::AllDurableRecords, &view.records_by_jsn), (ReplayCoreInputKind::StrictSegment, &strict_segment)] {
                                let walk = walk_prefix(records, root_txg, &verify, PrefixRules::TODAY);
                                let result = Bing0Result {
                                    core_failure: walk.core_failure,
                                    applied_records: walk.applied.len(),
                                    last_applied_jsn: walk.applied.last().map(|record| record.jsn),
                                    maximum_applied_transaction: walk.applied.iter().map(|record| record.transaction).max().unwrap_or(0),
                                };
                                let key_records: Vec<ReplayCoreRecordKey> = records
                                    .iter()
                                    .map(|record| (record.jsn, record.txg, record.transaction, record.is_commit, record.ends_its_publish, record.ordinal_within_publish, verify(record)))
                                    .collect();
                                let entry = inputs.entry((kind, root_txg, key_records)).or_insert((0, result));
                                assert_eq!(entry.1, result, "同一份恢复输入，模型的丙0 给同一个结果");
                                entry.0 += 1;
                            }
                        }
                    }
                }
            }
        }
        state_counts.push((*segment_label, cells, states_in_segment));
    }
    let flag = |holds: bool| if holds { "1" } else { "0" };
    for ((kind, root_txg, records), (states, result)) in &inputs {
        let records_text = if records.is_empty() {
            "none".to_string()
        } else {
            records
                .iter()
                .map(|(jsn, txg, transaction, is_commit, ends, ordinal, verified)| format!("{jsn}:{txg}:{transaction}:{}:{}:{ordinal}:{}", flag(*is_commit), flag(*ends), flag(*verified)))
                .collect::<Vec<_>>()
                .join(",")
        };
        emit(
            &mut emitter,
            &format!(
                "name=replay_core_input input={} root_txg={root_txg} states={states} records={records_text} bing0_failure={} bing0_applied_records={} bing0_last_applied_jsn={} bing0_maximum_applied_transaction={}",
                kind.label(),
                yes_or_no(result.core_failure),
                result.applied_records,
                result.last_applied_jsn.map_or("none".to_string(), |jsn| jsn.to_string()),
                result.maximum_applied_transaction
            ),
        );
    }
    for (segment_label, cells, states) in &state_counts {
        emit(&mut emitter, &format!("name=replay_core_input_states segment={segment_label} cells={cells} states={states}"));
    }
    for kind in [ReplayCoreInputKind::AllDurableRecords, ReplayCoreInputKind::StrictSegment] {
        let of_kind: Vec<&(u64, Bing0Result)> = inputs.iter().filter(|((candidate, ..), _)| *candidate == kind).map(|(_, value)| value).collect();
        emit(
            &mut emitter,
            &format!(
                "name=replay_core_input_summary input={} distinct_inputs={} states={}",
                kind.label(),
                of_kind.len(),
                of_kind.iter().map(|(states, _)| *states).sum::<u64>()
            ),
        );
    }
    println!("{}", emitter.finish());
}

/// 带 `segment-three` 跑第三段（fsync 等待模型）；带 `replay-core-inputs` 打出第一、二段每个崩溃状态的恢复输入与模型丙0 的结果。
const SEGMENT_THREE_ARGUMENT: &str = "segment-three";
const REPLAY_CORE_INPUTS_ARGUMENT: &str = "replay-core-inputs";

/// 不带参数跑第一段（`research/results/e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out` 那一份）；带 `segment-two` 跑第二段（多飞）。
const SEGMENT_TWO_ARGUMENT: &str = "segment-two";

fn main() {
    preflight();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => run_segment_one(),
        [segment] if segment == SEGMENT_TWO_ARGUMENT => run_segment_two(),
        [segment] if segment == SEGMENT_THREE_ARGUMENT => run_segment_three(),
        [mode] if mode == REPLAY_CORE_INPUTS_ARGUMENT => run_replay_core_inputs(),
        other => {
            eprintln!("用法：不带参数跑第一段；带 {SEGMENT_TWO_ARGUMENT} 跑第二段；带 {SEGMENT_THREE_ARGUMENT} 跑第三段；带 {REPLAY_CORE_INPUTS_ARGUMENT} 打出恢复输入。收到的参数：{other:?}");
            std::process::exit(2);
        }
    }
}

fn run_segment_one() {
    let mut emitter = Emitter::new();
    let histories = all_histories();
    let count_of = |family: HistoryFamily| histories.iter().filter(|history| history.family() == family).count();
    emit(
        &mut emitter,
        &format!(
            "name=config experiment=e169 segment=1 q1=serial_publishes,shared_publish q2=jia_whole,jia_interleaved,yi_reserved_whole,ding_control q3=bing0_today,bing1_truncate_before_a,bing2_drop_a_keep_b,bing3_forbid readings=r2_covered,r1_issued histories_h0={} histories_h1={} histories_h2={} in_flight_record_limit={IN_FLIGHT_RECORD_LIMIT}",
            count_of(HistoryFamily::H0),
            count_of(HistoryFamily::H1),
            count_of(HistoryFamily::H2)
        ),
    );
    // 停机条款 T1：十四条序列逐条核，有一条不同就停。
    let mut t1_mismatches = 0u64;
    for sequence in fixed_sequences() {
        let (publishes, records) = bing0_on_fixed_sequence(&sequence);
        let matches = publishes == sequence.expected_publishes_applied && records == sequence.expected_records_applied;
        t1_mismatches += u64::from(!matches);
        emit(
            &mut emitter,
            &format!(
                "name=t1_sequence sequence={} deciding_line=lib.rs:{} expected_publishes_applied={} model_publishes_applied={publishes} expected_records_applied={} model_records_applied={records} matches={}",
                sequence.number,
                sequence.deciding_line,
                sequence.expected_publishes_applied,
                sequence.expected_records_applied,
                yes_or_no(matches)
            ),
        );
    }
    emit(&mut emitter, &format!("name=t1_summary sequences=14 mismatches={t1_mismatches}"));
    if t1_mismatches > 0 {
        emit(&mut emitter, "name=stopped clause=T1 remaining_segment_one=not_run");
        println!("{}", emitter.finish());
        std::process::exit(3);
    }
    emit_anchors(&mut emitter);
    let mut grid: BTreeMap<GridKey, AppendCellResult> = BTreeMap::new();
    for condition in PublishCondition::SEGMENT_ONE {
        for arm in AppendArm::ALL {
            for history in &histories {
                for reading in AppendedReading::ALL {
                    let key_text = cell_key_text(condition, arm, history, reading);
                    match evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), reading)) {
                        Some(result) => {
                            emit_cell(&mut emitter, &key_text, &result);
                            grid.insert((condition, arm, history.clone(), reading), result);
                        }
                        None => emit(&mut emitter, &format!("name=cell {key_text} applicable=no")),
                    }
                }
            }
        }
    }
    emit_controls(&mut emitter, &grid);
    emit_verdicts(&mut emitter, &grid, &PublishCondition::SEGMENT_ONE);
    emit_assertions(&mut emitter, &grid, &PublishCondition::SEGMENT_ONE, true);
    emit_geometry(&mut emitter, &grid, &PublishCondition::SEGMENT_ONE, false);
    emit_h2_summaries(&mut emitter, &grid);
    println!("{}", emitter.finish());
}

/// 第二段的 B5 锚点：16 格的执行数与去重状态数逐个比；返回不符的条数。
fn emit_segment_two_anchors(emitter: &mut Emitter, grid: &BTreeMap<GridKey, AppendCellResult>) -> u64 {
    let mut checked = 0u64;
    let mut mismatches = 0u64;
    for (counts, condition, arm, executions, states) in ANCHOR_B5 {
        let measured = &grid[&(condition, arm, h0(counts), AppendedReading::R2IssuedAndCovered)];
        for (quantity, expected, value) in [("executions", executions, measured.executions), ("states", states, measured.states)] {
            checked += 1;
            mismatches += u64::from(expected != value);
            emit(
                emitter,
                &format!(
                    "name=anchor family=B5 q1={} q2={} r={} quantity={quantity} source=script expected={expected} measured={value} matches={}",
                    condition.label(),
                    arm.label(),
                    h0(counts).record_counts_label(),
                    yes_or_no(expected == value)
                ),
            );
        }
    }
    emit(emitter, &format!("name=anchor_summary families=B5 checked={checked} mismatches={mismatches}"));
    mismatches
}

/// 路径对拍：k = 2 的每一格再按配置去重算一遍，状态集合、执行数、轨迹、各量逐格比（k = 3 那条路径的校验）。
#[derive(Default)]
struct PathCheckTally {
    cells: u64,
    state_set_mismatches: u64,
    execution_mismatches: u64,
    trajectory_mismatches: u64,
    quantity_mismatches: u64,
    a7_zero_mismatches: u64,
    mismatch_lines: Vec<String>,
}

impl PathCheckTally {
    fn compare(&mut self, key_text: &str, request: &CellRequest, layout: &CellLayout, direct: &EnumeratedCrashStates, direct_result: &AppendCellResult) {
        let (searched, searched_result) = multi_flight_cell_result(request, layout, MultiFlightEnumeration::ByConfiguration);
        let state_sets_differ = direct.states != searched.states;
        let executions_differ = direct_result.executions != searched_result.executions;
        let trajectories_differ = direct_result.pending != searched_result.pending;
        let quantities_differ = direct_result.recovery != searched_result.recovery
            || direct_result.incomplete_pair_states != searched_result.incomplete_pair_states
            || direct_result.pattern_p_states != searched_result.pattern_p_states;
        let a7_zero_differs = (direct_result.a7_violation_points == 0) != (searched_result.a7_violation_points == 0);
        self.cells += 1;
        self.state_set_mismatches += u64::from(state_sets_differ);
        self.execution_mismatches += u64::from(executions_differ);
        self.trajectory_mismatches += u64::from(trajectories_differ);
        self.quantity_mismatches += u64::from(quantities_differ);
        self.a7_zero_mismatches += u64::from(a7_zero_differs);
        if state_sets_differ || executions_differ || trajectories_differ || quantities_differ || a7_zero_differs {
            self.mismatch_lines.push(format!(
                "name=path_check_mismatch {key_text} direct_states={} searched_states={} direct_executions={} searched_executions={}",
                direct_result.states, searched_result.states, direct_result.executions, searched_result.executions
            ));
        }
    }
    fn emit_into(&self, emitter: &mut Emitter) {
        for line in &self.mismatch_lines {
            emit(emitter, line);
        }
        emit(
            emitter,
            &format!(
                "name=path_check paths=direct,by_configuration cells={} state_set_mismatches={} execution_mismatches={} trajectory_mismatches={} quantity_mismatches={} a7_zero_mismatches={}",
                self.cells, self.state_set_mismatches, self.execution_mismatches, self.trajectory_mismatches, self.quantity_mismatches, self.a7_zero_mismatches
            ),
        );
    }
}

fn run_segment_two() {
    let mut emitter = Emitter::new();
    let histories = segment_two_histories();
    let count_of = |family: HistoryFamily| histories.iter().filter(|history| history.family() == family).count();
    emit(
        &mut emitter,
        &format!(
            "name=config experiment=e169 segment=2 q1=multi_flight_roots_ordered,multi_flight_roots_unordered q2=jia_whole,jia_interleaved,yi_reserved_whole,ding_control q3=bing0_today,bing1_truncate_before_a,bing2_drop_a_keep_b,bing3_forbid readings=r2_covered,r1_issued histories_h0={} histories_h1={} histories_h2=not_run_after_decidable k3_record_counts=1,1,1;2,1,1 enumeration_k2=direct enumeration_k3=by_configuration configuration_limit={CONFIGURATION_LIMIT} in_flight_record_limit={IN_FLIGHT_RECORD_LIMIT}",
            count_of(HistoryFamily::H0),
            count_of(HistoryFamily::H1)
        ),
    );
    let mut grid: BTreeMap<GridKey, AppendCellResult> = BTreeMap::new();
    let mut path_check = PathCheckTally::default();
    for condition in PublishCondition::SEGMENT_TWO {
        for arm in AppendArm::ALL {
            for history in &histories {
                for reading in AppendedReading::ALL {
                    let key_text = cell_key_text(condition, arm, history, reading);
                    let request = CellRequest::main_grid(condition, arm, history.clone(), reading);
                    let layout = plan_cell(condition, arm, history, AppendDiscipline::Kept);
                    let enumeration = MultiFlightEnumeration::registered_for(history);
                    let (enumerated, result) = multi_flight_cell_result(&request, &layout, enumeration);
                    if enumeration == MultiFlightEnumeration::DirectExecutions {
                        path_check.compare(&key_text, &request, &layout, &enumerated, &result);
                    }
                    emit_cell(&mut emitter, &key_text, &result);
                    emit(
                        &mut emitter,
                        &format!(
                            "name=cell_enumeration {key_text} enumeration={} configurations={} configuration_limit_exceeded={}",
                            result.multi_flight_enumeration.map_or("not_applicable", MultiFlightEnumeration::label),
                            result.configurations.map_or("not_applicable".to_string(), |count| count.to_string()),
                            yes_or_no(result.configuration_limit_exceeded)
                        ),
                    );
                    grid.insert((condition, arm, history.clone(), reading), result);
                }
            }
        }
    }
    // 停机条款 T3：k = 3 的配置数。
    let three_transaction_cells: Vec<&AppendCellResult> = grid.iter().filter(|((_, _, history, _), _)| history.transaction_count() == 3).map(|(_, result)| result).collect();
    let largest_configurations = three_transaction_cells.iter().filter_map(|result| result.configurations).max().unwrap_or(0);
    let exceeded_cells = three_transaction_cells.iter().filter(|result| result.configuration_limit_exceeded).count();
    emit(
        &mut emitter,
        &format!(
            "name=t3 k=3 cells={} largest_configurations={largest_configurations} limit={CONFIGURATION_LIMIT} exceeded_cells={exceeded_cells} k3_cells_kept={}",
            three_transaction_cells.len(),
            yes_or_no(exceeded_cells == 0)
        ),
    );
    if exceeded_cells > 0 {
        grid.retain(|(_, _, history, _), _| history.transaction_count() == 2);
        emit(&mut emitter, "name=stopped clause=T3 k3=not_run");
    }
    emit_segment_two_anchors(&mut emitter, &grid);
    path_check.emit_into(&mut emitter);
    emit_verdicts(&mut emitter, &grid, &PublishCondition::SEGMENT_TWO);
    emit_assertions(&mut emitter, &grid, &PublishCondition::SEGMENT_TWO, false);
    emit_geometry(&mut emitter, &grid, &PublishCondition::SEGMENT_TWO, true);
    // H2 放在段末：Q2、Q3 两行在多飞两个条件上的判决格都在 H0 ∪ H1，全部有了值，H2 够判后不跑（登记第六节「附带」、第十一节够判停机）。
    for condition in PublishCondition::SEGMENT_TWO {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                let applicable = all_histories().iter().filter(|history| history.family() == HistoryFamily::H2).any(|history| cell_is_applicable(arm, history));
                let status = if applicable { "applicable=yes run=not_run_after_decidable" } else { "applicable=no" };
                emit(&mut emitter, &format!("name=h2_summary q1={} q2={} reading={} {status}", condition.label(), arm.label(), reading.label()));
            }
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERIAL: PublishCondition = PublishCondition::SerialPublishes;
    const SHARED: PublishCondition = PublishCondition::SharedPublish;
    const R2: AppendedReading = AppendedReading::R2IssuedAndCovered;
    const R1: AppendedReading = AppendedReading::R1Issued;

    fn legal_histories() -> Vec<History> {
        all_histories().into_iter().filter(|history| history.family().is_legal()).collect()
    }

    #[test]
    fn bing0_gives_the_registered_result_on_each_of_the_fourteen_fixed_sequences() {
        for sequence in fixed_sequences() {
            let (publishes, records) = bing0_on_fixed_sequence(&sequence);
            assert_eq!(publishes, sequence.expected_publishes_applied, "序列 {} 施加的发布数（由 lib.rs:{} 定）", sequence.number, sequence.deciding_line);
            assert_eq!(records, sequence.expected_records_applied, "序列 {} 施加的记录数", sequence.number);
        }
    }

    #[test]
    fn history_families_have_twelve_thirty_two_and_sixteen_members() {
        let histories = all_histories();
        assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::H0).count(), 12);
        assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::H1).count(), 32);
        assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::H2).count(), 16);
    }

    #[test]
    fn serial_publishes_h0_state_counts_equal_anchor_b1_for_every_append_arm_and_both_readings() {
        for (counts, states) in ANCHOR_B1_LISTED {
            for arm in AppendArm::ALL {
                for reading in AppendedReading::ALL {
                    assert_eq!(grid_cell(SERIAL, arm, h0(counts), reading).states, states, "B1 {arm:?} r={counts:?} {reading:?}");
                }
            }
        }
        for history in all_histories().into_iter().filter(|history| history.family() == HistoryFamily::H0) {
            assert_eq!(grid_cell(SERIAL, AppendArm::JiaWhole, history.clone(), R2).states, anchor_b1_closed_form(&history.record_counts));
        }
    }

    #[test]
    fn shared_publish_h0_whole_arm_state_counts_equal_anchor_b2() {
        for (counts, states) in ANCHOR_B2_LISTED {
            for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
                assert_eq!(grid_cell(SHARED, arm, h0(counts), R2).states, states, "B2 {arm:?} r={counts:?}");
                assert_eq!(grid_cell(SHARED, arm, h0(counts), R1).states, states, "B2 R1 {arm:?} r={counts:?}");
            }
        }
        for history in all_histories().into_iter().filter(|history| history.family() == HistoryFamily::H0) {
            assert_eq!(grid_cell(SHARED, AppendArm::YiReservedWhole, history.clone(), R2).states, anchor_b2_closed_form(&history.record_counts));
        }
    }

    #[test]
    fn shared_publish_h0_interleaved_and_control_executions_and_states_equal_anchor_b3() {
        for (counts, (interleaved_executions, interleaved_states), (control_executions, control_states)) in ANCHOR_B3 {
            let interleaved = grid_cell(SHARED, AppendArm::JiaInterleaved, h0(counts), R2);
            assert_eq!(interleaved.executions, interleaved_executions, "甲交错执行数 r={counts:?}");
            assert_eq!(interleaved.states, interleaved_states, "甲交错状态数 r={counts:?}");
            let control = grid_cell(SHARED, AppendArm::DingNoDiscipline, h0(counts), R2);
            assert_eq!(control.executions, control_executions, "对照丁执行数 r={counts:?}");
            assert_eq!(control.states, control_states, "对照丁状态数 r={counts:?}");
        }
    }

    #[test]
    fn narrow_void_state_counts_equal_the_anchor_b4_script_output() {
        for (counts, voided, serial_jia, serial_yi, shared_jia, shared_yi) in ANCHOR_B4_SCRIPT {
            assert_eq!(grid_cell(SERIAL, AppendArm::JiaWhole, h1(counts, voided), R2).states, serial_jia, "串落甲 r={counts:?} v={voided}");
            assert_eq!(grid_cell(SERIAL, AppendArm::YiReservedWhole, h1(counts, voided), R2).states, serial_yi, "串落乙 r={counts:?} v={voided}");
            assert_eq!(grid_cell(SHARED, AppendArm::JiaWhole, h1(counts, voided), R2).states, shared_jia, "同发甲 r={counts:?} v={voided}");
            assert_eq!(grid_cell(SHARED, AppendArm::YiReservedWhole, h1(counts, voided), R2).states, shared_yi, "同发乙 r={counts:?} v={voided}");
        }
    }

    #[test]
    fn narrow_void_state_counts_equal_the_b4_closed_forms_for_all_thirty_two_histories() {
        for history in all_histories().into_iter().filter(|history| history.family() == HistoryFamily::H1) {
            let (voided, _) = history.voided.expect("H1");
            for condition in PublishCondition::SEGMENT_ONE {
                assert_eq!(grid_cell(condition, AppendArm::JiaWhole, history.clone(), R2).states, anchor_b4_closed_form(condition, false, &history.record_counts, voided), "{condition:?} 甲 {history:?}");
                assert_eq!(grid_cell(condition, AppendArm::YiReservedWhole, history.clone(), R2).states, anchor_b4_closed_form(condition, true, &history.record_counts, voided), "{condition:?} 乙 {history:?}");
            }
        }
    }

    #[test]
    fn real_baseline_serial_jia_whole_bing0_is_zero_on_every_quantity_of_every_legal_history() {
        for history in legal_histories() {
            for reading in AppendedReading::ALL {
                let result = grid_cell(SERIAL, AppendArm::JiaWhole, history.clone(), reading);
                let bing0 = &result.recovery[RecoveryArm::Bing0Today.position()];
                assert_eq!(result.incomplete_pair_states, 0, "A1 n2a {history:?}");
                assert_eq!(result.pattern_p_states, 0, "A1 n2b {history:?}");
                assert_eq!(bing0.v_fsync, 0, "A1 v_fsync {history:?}");
                assert_eq!(bing0.v_dependency, 0, "A1 v_dep {history:?}");
                assert_eq!(bing0.v_dangling(), 0, "A1 v_dangling {history:?}");
                assert_eq!(bing0.partial_publish, 0, "A1 部分施加 {history:?}");
                assert_eq!(bing0.n_atom, 0, "A1 n_atom {history:?}");
            }
        }
    }

    #[test]
    fn whole_append_arms_leave_no_incomplete_pair_and_no_pattern_p_on_legal_histories() {
        for condition in PublishCondition::SEGMENT_ONE {
            for arm in [AppendArm::JiaWhole, AppendArm::YiReservedWhole] {
                for history in legal_histories() {
                    let result = grid_cell(condition, arm, history.clone(), R2);
                    assert_eq!(result.incomplete_pair_states, 0, "A4 n2a {condition:?} {arm:?} {history:?}");
                    assert_eq!(result.pattern_p_states, 0, "A4 n2b {condition:?} {arm:?} {history:?}");
                }
            }
        }
    }

    #[test]
    fn no_recovery_arm_applies_part_of_a_transaction_in_any_cell() {
        for condition in PublishCondition::SEGMENT_ONE {
            for arm in AppendArm::ALL {
                for history in all_histories() {
                    if let Some(result) = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), R2)) {
                        for recovery_arm in RecoveryArm::ALL {
                            assert_eq!(result.recovery[recovery_arm.position()].n_atom, 0, "A3 {condition:?} {arm:?} {history:?} {recovery_arm:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn strict_segment_never_exceeds_six_and_the_in_flight_limit_never_bites() {
        for condition in PublishCondition::SEGMENT_ONE {
            for arm in AppendArm::ALL {
                for history in all_histories() {
                    if let Some(result) = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), R2)) {
                        assert!(result.strict_segment_length_peak <= STRICT_SEGMENT_LENGTH_BOUND, "A6 {condition:?} {arm:?} {history:?}");
                        assert_eq!(result.in_flight_limit_states, 0, "A6 {condition:?} {arm:?} {history:?}");
                        assert_eq!(result.a7_violation_points, 0, "A7 {condition:?} {arm:?} {history:?}");
                        assert_eq!(result.pending.final_peak, 0, "轨迹期末值 {condition:?} {arm:?} {history:?}");
                    }
                }
            }
        }
        let longest = grid_cell(SHARED, AppendArm::DingNoDiscipline, h0(&[2, 2, 2]), R2).strict_segment_length_peak;
        assert_eq!(longest, 6, "k = 3、r = 2 时严格连续段取得到 6 条");
    }

    /// 手推：执行 [T1.1, T2.1, T1.2] 里持久集合 {U1, U2, S, jsn1, jsn2} 是唯一满足模式 P 的状态，丙3 判坏它。
    #[test]
    fn shared_interleaved_two_one_has_exactly_one_pattern_p_state_which_bing3_refuses() {
        let result = grid_cell(SHARED, AppendArm::JiaInterleaved, h0(&[2, 1]), R2);
        assert_eq!(result.pattern_p_states, 1, "N2b");
        assert_eq!(result.incomplete_pair_states, 0, "N2a");
        assert_eq!(result.recovery[RecoveryArm::Bing3ForbidPattern.position()].refused, 1, "丙3 判坏数");
        let layout = plan_cell(SHARED, AppendArm::JiaInterleaved, &h0(&[2, 1]), AppendDiscipline::Kept);
        let executions = build_executions(&layout, AppendArm::JiaInterleaved, AppendDiscipline::Kept, PublishBarriers::AllThree);
        let mut states = HashSet::new();
        for execution in &executions {
            collect_crash_states(execution, R2, ReturnRule::AfterOwnPublishLastBarrier, &mut states);
        }
        let target = states
            .iter()
            .find(|state| {
                let view = StateView::of(state);
                view.records_by_jsn.len() == 2
                    && view.records_by_jsn[0].jsn == 1
                    && view.records_by_jsn[0].transaction == 1
                    && view.records_by_jsn[1].jsn == 2
                    && view.records_by_jsn[1].transaction == 2
                    && view.records_by_jsn[1].is_commit
            })
            .expect("登记第九节 M13 那个状态在");
        let view = StateView::of(target);
        assert!(!recover(&view, RecoveryArm::Bing3ForbidPattern, RecoveryControl::NoControl).mounted, "丙3 判坏");
    }

    /// 手推：甲交错 r = (2, 2) 的三个执行里前两条属于不同事务的是 1212、1221；段 = {jsn1, jsn2} 时持久集合是
    /// {1, 2}（两个执行内容相同）或 {1, 2, 4}（jsn4 在 1212 里是 T2.2、在 1221 里是 T1.2，内容不同）⇒ 3 个；
    /// 满足模式 P 的是两个执行各自的 {jsn1, jsn2, jsn3}（jsn4 不持久），内容不同，共 2 个。
    #[test]
    fn shared_interleaved_two_two_has_three_incomplete_pair_states_and_two_pattern_p_states() {
        let result = grid_cell(SHARED, AppendArm::JiaInterleaved, h0(&[2, 2]), R2);
        assert_eq!(result.incomplete_pair_states, 3);
        assert_eq!(result.pattern_p_states, 2);
    }

    /// 手推：对照丁 r = (2, 2) 里两个半截事务同在段里，只能是段 = {jsn1, jsn2} 且两条属于不同事务（前两条是 12 或 21）；
    /// 每种前两条下持久集合 {1, 2} 一个、{1, 2, 4} 按 jsn4 是谁的第二条两个 ⇒ 2 × 3 = 6。
    #[test]
    fn shared_control_two_two_has_six_incomplete_pair_states() {
        assert_eq!(grid_cell(SHARED, AppendArm::DingNoDiscipline, h0(&[2, 2]), R2).incomplete_pair_states, 6);
    }

    #[test]
    fn q2_positive_controls_are_positive_for_every_append_arm() {
        for arm in [AppendArm::JiaWhole, AppendArm::JiaInterleaved, AppendArm::YiReservedWhole] {
            let mut request = CellRequest::main_grid(SHARED, arm, h0(&[2, 2]), R2);
            request.discipline = AppendDiscipline::RemovedForPositiveControl;
            let result = evaluate_cell(&request).expect("H0");
            assert!(result.incomplete_pair_states > 0, "{arm:?} 拿掉纪律后 N2a");
            assert!(result.pattern_p_states > 0, "{arm:?} 拿掉纪律后 N2b");
        }
    }

    /// 手推（登记 5.3 V_fsync 早返回，同发甲整段 r = (1, 1)）：R2 下 T1 在 jsn1 发出后就返回、T2 要等 B2，
    /// 记录段里返回集合 {T1} 的 4 个持久子集里 3 个丢 T1；R1 下另有返回集合 {T1, T2} 的 3 个丢失状态与 {T1} 的 2 个，共 5。
    #[test]
    fn early_return_control_loses_returned_transactions_in_three_states_under_r2_and_five_under_r1() {
        for (reading, expected) in [(R2, 3u64), (R1, 5u64)] {
            let mut request = CellRequest::main_grid(SHARED, AppendArm::JiaWhole, h0(&[1, 1]), reading);
            request.return_rule = ReturnRule::EarlyAfterOwnRecordsIssued;
            let result = evaluate_cell(&request).expect("H0");
            assert_eq!(result.recovery[RecoveryArm::Bing0Today.position()].v_fsync, expected, "{reading:?}");
        }
    }

    #[test]
    fn early_return_control_is_positive_for_every_append_arm_recovery_arm_and_reading() {
        for arm in AppendArm::ALL {
            for reading in AppendedReading::ALL {
                let mut request = CellRequest::main_grid(SHARED, arm, h0(&[1, 1]), reading);
                request.return_rule = ReturnRule::EarlyAfterOwnRecordsIssued;
                let result = evaluate_cell(&request).expect("H0");
                for recovery_arm in RecoveryArm::ALL {
                    assert!(result.recovery[recovery_arm.position()].v_fsync > 0, "{arm:?} {reading:?} {recovery_arm:?}");
                }
            }
        }
    }

    /// 手推（拿掉 B1、B2，串落甲整段 r = (1, 1)）：第一次发布那一段 F1 持久而 U1、S1 缺一的 6 个，
    /// 第二次发布那一段 F2 持久而 U2、S2 缺一的 6 个（返回集合 {T1}）。
    #[test]
    fn dropping_the_unit_and_record_barriers_dangles_six_plus_six_states_under_bing0() {
        let mut request = CellRequest::main_grid(SERIAL, AppendArm::JiaWhole, h0(&[1, 1]), R2);
        request.barriers = PublishBarriers::WithoutBarriersAfterUnitsAndRecords;
        let result = evaluate_cell(&request).expect("H0");
        let bing0 = &result.recovery[RecoveryArm::Bing0Today.position()];
        assert_eq!(bing0.v_dangling_returned_empty, 6);
        assert_eq!(bing0.v_dangling_returned_nonempty, 6);
        for recovery_arm in RecoveryArm::ALL {
            assert!(result.recovery[recovery_arm.position()].v_dangling() > 0, "{recovery_arm:?}");
        }
    }

    #[test]
    fn dependency_and_atomicity_controls_are_positive_for_every_recovery_arm() {
        let mut dependency = CellRequest::main_grid(SHARED, AppendArm::DingNoDiscipline, h0(&[2, 2]), R2);
        dependency.recovery_control = RecoveryControl::KeepLaterRecordsRemoved;
        let dependency_result = evaluate_cell(&dependency).expect("H0");
        let mut atomicity = CellRequest::main_grid(SHARED, AppendArm::JiaWhole, h0(&[2, 2]), R2);
        atomicity.recovery_control = RecoveryControl::CommitMarkerHandlingRemoved;
        let atomicity_result = evaluate_cell(&atomicity).expect("H0");
        for recovery_arm in RecoveryArm::ALL {
            assert!(dependency_result.recovery[recovery_arm.position()].v_dependency > 0, "V_dep 对照 {recovery_arm:?}");
            assert!(atomicity_result.recovery[recovery_arm.position()].n_atom > 0, "N_atom 对照 {recovery_arm:?}");
        }
    }

    /// 手推：同发甲整段 r = (2, 2) 里 T1 两条持久、T2 不全的 3 个状态，丙2 施加 T1 的两条（一次发布的一部分），
    /// 丙0 一条不施加 ⇒ 部分施加 3、与丙0 不同的状态 3；r = (1, 1) 时只有 {jsn1} 那 1 个。
    #[test]
    fn bing2_applies_part_of_a_shared_publish_in_three_states_at_two_two_and_one_at_one_one() {
        let two_two = grid_cell(SHARED, AppendArm::JiaWhole, h0(&[2, 2]), R2);
        assert_eq!(two_two.recovery[RecoveryArm::Bing2DropAKeepB.position()].partial_publish, 3);
        assert_eq!(two_two.recovery[RecoveryArm::Bing2DropAKeepB.position()].differs_from_bing0, 3);
        assert_eq!(two_two.a5_differing_states, 3);
        let one_one = grid_cell(SHARED, AppendArm::JiaWhole, h0(&[1, 1]), R2);
        assert_eq!(one_one.recovery[RecoveryArm::Bing2DropAKeepB.position()].partial_publish, 1);
    }

    #[test]
    fn bing0_bing1_and_bing3_never_apply_part_of_a_publish() {
        for condition in PublishCondition::SEGMENT_ONE {
            for arm in AppendArm::ALL {
                for history in all_histories() {
                    if let Some(result) = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), R2)) {
                        for recovery_arm in [RecoveryArm::Bing0Today, RecoveryArm::Bing1TruncateBeforeA, RecoveryArm::Bing3ForbidPattern] {
                            assert_eq!(result.recovery[recovery_arm.position()].partial_publish, 0, "{condition:?} {arm:?} {history:?} {recovery_arm:?}");
                        }
                    }
                }
            }
        }
    }

    /// 登记 8.2 判别力自证：把 Q2-1 的门槛从「> 0」挪到主取样点上的实测值，那一格由「打中」转「没打中」。
    #[test]
    fn moving_the_q2_1_threshold_to_the_main_point_reading_flips_hit_to_not_hit() {
        let measured = grid_cell(SHARED, AppendArm::JiaInterleaved, h0(&[2, 2]), R2).incomplete_pair_states;
        let judge = |value: u64, threshold: u64| value > threshold;
        assert!(judge(measured, 0), "门槛 > 0 时打中");
        assert!(!judge(measured, measured), "门槛挪到实测值时没打中");
    }

    /// 等价变异留档（登记第九节末段）：单实例、没有残留记录，`lib.rs:452` 那一判在模型里恒不断，删它的变异在所有输入上同值。
    #[test]
    fn back_chain_is_never_broken_in_a_single_instance_without_residue() {
        let layout = plan_cell(SHARED, AppendArm::DingNoDiscipline, &h0(&[2, 2, 2]), AppendDiscipline::Kept);
        for execution in build_executions(&layout, AppendArm::DingNoDiscipline, AppendDiscipline::Kept, PublishBarriers::AllThree) {
            for operation in &execution.operations {
                if let Some(WriteContent::JournalRecord(record)) = operation.written_content() {
                    assert!(!back_chain_is_broken(&record));
                }
            }
        }
    }

    #[test]
    fn r1_and_r2_readings_give_the_same_counts_on_serial_and_shared_publishes() {
        for condition in PublishCondition::SEGMENT_ONE {
            for arm in AppendArm::ALL {
                for history in all_histories() {
                    let under_r2 = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), R2));
                    let under_r1 = evaluate_cell(&CellRequest::main_grid(condition, arm, history.clone(), R1));
                    if let (Some(r2_result), Some(r1_result)) = (under_r2, under_r1) {
                        assert_eq!(r2_result.states, r1_result.states, "{condition:?} {arm:?} {history:?}");
                        assert_eq!(r2_result.recovery, r1_result.recovery, "{condition:?} {arm:?} {history:?}");
                    }
                }
            }
        }
    }

    // ===== 第二段（多飞） =====

    const ROOTS_ORDERED: PublishCondition = PublishCondition::MultiFlightRootsOrdered;
    const ROOTS_UNORDERED: PublishCondition = PublishCondition::MultiFlightRootsUnordered;

    fn multi_flight(condition: PublishCondition, arm: AppendArm, history: History, reading: AppendedReading, enumeration: MultiFlightEnumeration) -> (EnumeratedCrashStates, AppendCellResult) {
        let request = CellRequest::main_grid(condition, arm, history.clone(), reading);
        let layout = plan_cell(condition, arm, &history, AppendDiscipline::Kept);
        multi_flight_cell_result(&request, &layout, enumeration)
    }

    #[test]
    fn segment_two_runs_six_h0_and_fourteen_h1_histories() {
        let histories = segment_two_histories();
        assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::H0).count(), 6);
        assert_eq!(histories.iter().filter(|history| history.family() == HistoryFamily::H1).count(), 14);
        assert_eq!(histories.len(), 20);
    }

    /// B5（`anchors_q1c.py`）：16 格的执行数与去重状态数，直接枚举与按配置去重两条路径各对一遍。
    /// 取样点覆盖 M14（根序 甲整段 r = (1, 1)：3747 → 6435）与 M15（根乱 乙整段 r = (1, 1)：77 → 81）。
    #[test]
    fn multi_flight_h0_executions_and_states_equal_anchor_b5_on_both_enumerations() {
        for (counts, condition, arm, executions, states) in ANCHOR_B5 {
            for enumeration in [MultiFlightEnumeration::DirectExecutions, MultiFlightEnumeration::ByConfiguration] {
                let (_, result) = multi_flight(condition, arm, h0(counts), R2, enumeration);
                assert_eq!(result.executions, executions, "B5 执行数 {condition:?} {arm:?} r={counts:?} {enumeration:?}");
                assert_eq!(result.states, states, "B5 状态数 {condition:?} {arm:?} r={counts:?} {enumeration:?}");
            }
        }
    }

    /// 按配置去重这条路径的校验：k = 2 的合法历史上与直接枚举逐格比状态集合、执行数、轨迹与各量（H0 r = (2, 2) 已由 B5 两条路径各对锚点）。
    #[test]
    fn configuration_search_matches_direct_enumeration_on_two_transaction_multi_flight_histories() {
        let mut compared = 0u32;
        for condition in PublishCondition::SEGMENT_TWO {
            for arm in AppendArm::ALL {
                for history in segment_two_histories().into_iter().filter(|history| history.transaction_count() == 2) {
                    let readings: &[AppendedReading] = if history.family() == HistoryFamily::H1 || history.record_counts == [2, 1] { &AppendedReading::ALL } else { &[R2] };
                    if history.family() == HistoryFamily::H0 && history.record_counts == [2, 2] {
                        continue;
                    }
                    for reading in readings.iter().copied() {
                        let (direct, direct_result) = multi_flight(condition, arm, history.clone(), reading, MultiFlightEnumeration::DirectExecutions);
                        let (searched, searched_result) = multi_flight(condition, arm, history.clone(), reading, MultiFlightEnumeration::ByConfiguration);
                        let label = format!("{condition:?} {arm:?} {history:?} {reading:?}");
                        assert!(direct.states == searched.states, "状态集合 {label}");
                        assert_eq!(direct_result.executions, searched_result.executions, "执行数 {label}");
                        assert_eq!(direct_result.pending, searched_result.pending, "轨迹 {label}");
                        assert_eq!(direct_result.recovery, searched_result.recovery, "各量 {label}");
                        assert_eq!(direct_result.a7_violation_points, 0, "A7 直接枚举 {label}");
                        assert_eq!(searched_result.a7_violation_points, 0, "A7 按配置 {label}");
                        compared += 1;
                    }
                }
            }
        }
        // 2 个条件 × 4 条臂 ×（H0 r = (1, 1)、(1, 2) 各 1 读 + H0 r = (2, 1) 2 读 + H1 8 种 × 2 读）= 8 × 20
        assert_eq!(compared, 160);
    }

    /// 根乱下乙整段与对照丁没有任何跨发布的次序约束：执行数 = 各发布操作序列的多项式系数（k = 3：24! / 8!³、25! / (9! 8! 8!)）。
    #[test]
    fn configuration_search_counts_every_merge_when_no_cross_publish_order_applies() {
        for arm in [AppendArm::YiReservedWhole, AppendArm::DingNoDiscipline] {
            let (_, one_one_one) = multi_flight(ROOTS_UNORDERED, arm, h0(&[1, 1, 1]), R2, MultiFlightEnumeration::ByConfiguration);
            assert_eq!(one_one_one.executions, 9_465_511_770, "{arm:?} r=(1,1,1)");
            let (_, two_one_one) = multi_flight(ROOTS_UNORDERED, arm, h0(&[2, 1, 1]), R2, MultiFlightEnumeration::ByConfiguration);
            assert_eq!(two_one_one.executions, 26_293_088_250, "{arm:?} r=(2,1,1)");
            assert!(two_one_one.configurations.expect("按配置去重报配置数") < CONFIGURATION_LIMIT);
        }
    }

    /// 多飞下也量得出 V_fsync：早返回（记录发出就返回）在根序、甲整段、r = (1, 1) 上每条 Q3 臂都 > 0。
    #[test]
    fn early_return_control_is_positive_under_multi_flight_for_every_recovery_arm() {
        for condition in PublishCondition::SEGMENT_TWO {
            let mut request = CellRequest::main_grid(condition, AppendArm::JiaWhole, h0(&[1, 1]), R2);
            request.return_rule = ReturnRule::EarlyAfterOwnRecordsIssued;
            let result = evaluate_cell(&request).expect("H0 适用");
            for recovery_arm in RecoveryArm::ALL {
                assert!(result.recovery[recovery_arm.position()].v_fsync > 0, "{condition:?} {recovery_arm:?}");
            }
        }
    }

    /// 根序对根乱：去掉 F、C 的次序约束，执行只多不少，而且根序的每个崩溃状态在根乱里都有。
    #[test]
    fn roots_unordered_states_contain_every_roots_ordered_state() {
        for arm in AppendArm::ALL {
            for counts in [&[1u32, 1][..], &[2, 1][..]] {
                let (ordered, ordered_result) = multi_flight(ROOTS_ORDERED, arm, h0(counts), R2, MultiFlightEnumeration::DirectExecutions);
                let (unordered, unordered_result) = multi_flight(ROOTS_UNORDERED, arm, h0(counts), R2, MultiFlightEnumeration::DirectExecutions);
                assert!(ordered_result.executions < unordered_result.executions, "{arm:?} r={counts:?}");
                assert!(ordered.states.is_subset(&unordered.states), "{arm:?} r={counts:?}");
            }
        }
    }

    // ===== 第三段：等待模型 =====

    fn wait(condition: PublishCondition, arm: AppendArm, profile: PreparationProfile, point: CostPoint, record_count: u32, transaction_count: u32, reading: AppendedReading) -> WaitResult {
        evaluate_wait(&WaitScenario { condition, append_arm: arm, profile, point, record_count, transaction_count }, reading)
    }

    const P1: CostPoint = CostPoint::DeviceDominated;
    const P2: CostPoint = CostPoint::CpuDominated;
    const UNIFORM: PreparationProfile = PreparationProfile::Uniform;
    const SLOW_HEAD: PreparationProfile = PreparationProfile::SlowHead;

    /// 登记第七节 B6 的原数：k = 1、r = 1：P1 下 4.205 / 4.2、P2 下 4.7 / 4.2（甲 / 乙），慢头各加 Δ = 5；四个条件、两读都一样。
    #[test]
    fn wait_single_transaction_equals_the_registered_b6_values_under_every_condition_and_reading() {
        let expected: [(AppendArm, PreparationProfile, CostPoint, u64); 12] = [
            (AppendArm::JiaWhole, UNIFORM, P1, 4205),
            (AppendArm::JiaInterleaved, UNIFORM, P1, 4205),
            (AppendArm::YiReservedWhole, UNIFORM, P1, 4200),
            (AppendArm::JiaWhole, UNIFORM, P2, 4700),
            (AppendArm::JiaInterleaved, UNIFORM, P2, 4700),
            (AppendArm::YiReservedWhole, UNIFORM, P2, 4200),
            (AppendArm::JiaWhole, SLOW_HEAD, P1, 9205),
            (AppendArm::JiaInterleaved, SLOW_HEAD, P1, 9205),
            (AppendArm::YiReservedWhole, SLOW_HEAD, P1, 9200),
            (AppendArm::JiaWhole, SLOW_HEAD, P2, 9700),
            (AppendArm::JiaInterleaved, SLOW_HEAD, P2, 9700),
            (AppendArm::YiReservedWhole, SLOW_HEAD, P2, 9200),
        ];
        for condition in WAIT_CONDITIONS {
            for reading in AppendedReading::ALL {
                for (arm, profile, point, w_max) in expected {
                    assert_eq!(wait(condition, arm, profile, point, 1, 1, reading).w_max, w_max, "{condition:?} {arm:?} {profile:?} {point:?} {reading:?}");
                    assert_eq!(anchor_b6_single_transaction(arm, profile, point), w_max, "闭式与原数相同");
                }
            }
        }
    }

    #[test]
    fn wait_serial_two_transactions_equal_the_registered_b6_values() {
        for reading in AppendedReading::ALL {
            assert_eq!(wait(SERIAL, AppendArm::JiaWhole, UNIFORM, P1, 1, 2, reading).w_max, 8410);
            assert_eq!(wait(SERIAL, AppendArm::YiReservedWhole, UNIFORM, P1, 1, 2, reading).w_max, 8400);
        }
    }

    /// 手推：多飞·根乱、甲整段、k = 2、r = 1、P1、等速：B1、B2、B3 的第二道都并进前一道（读法 ⑤ 跨发布），两个 F 各占 b。
    #[test]
    fn wait_roots_unordered_two_transactions_merge_the_second_barriers_and_follow_the_hand_computed_path() {
        let result = wait(ROOTS_UNORDERED, AppendArm::JiaWhole, UNIFORM, P1, 1, 2, R2);
        assert_eq!(result.return_times, vec![5405, 5405]);
        assert_eq!(result.latest_transaction, 2);
        assert_eq!(result.merged_barriers_on_device, 3);
        let expected_path = CriticalPath { barriers_or_forced_unit_access: 5, device_writes: 8, cpu_steps: 1, merged_barriers: 1, tie_steps: 1, source_time: 0, duration_sum: 5405 };
        assert_eq!(result.path, expected_path);
    }

    /// 手推：同发、甲整段、r = 1、P2、等速：追加者每条 0.5，k = 8 时 8.55，k = 64 时 39.35；关键路径 64 步 CPU、4 道屏障或 FUA、67 次写。
    #[test]
    fn wait_shared_publish_jia_whole_under_cpu_dominated_costs_follows_the_appender() {
        assert_eq!(wait(SHARED, AppendArm::JiaWhole, UNIFORM, P2, 1, 8, R2).w_max, 8550);
        let result = wait(SHARED, AppendArm::JiaWhole, UNIFORM, P2, 1, 64, R2);
        assert_eq!(result.w_max, 39350);
        assert_eq!((result.path.cpu_steps, result.path.barriers_or_forced_unit_access, result.path.device_writes, result.path.tie_steps), (64, 4, 67, 0));
    }

    /// 手推：同发、乙整段、r = 1、P2、等速：预留每个 0.5，T64 的预留在 32 才完成，k = 8 时 7.1，k = 64 时 35.1；关键路径 64 步预留、2 次写、3 道屏障或 FUA。
    #[test]
    fn wait_shared_publish_yi_under_cpu_dominated_costs_follows_the_reservations() {
        assert_eq!(wait(SHARED, AppendArm::YiReservedWhole, UNIFORM, P2, 1, 8, R2).w_max, 7100);
        let result = wait(SHARED, AppendArm::YiReservedWhole, UNIFORM, P2, 1, 64, R2);
        assert_eq!(result.w_max, 35100);
        assert_eq!((result.path.cpu_steps, result.path.device_writes, result.path.barriers_or_forced_unit_access), (64, 2, 3));
    }

    /// 手推：多飞、乙整段、k = 2、r = 1、P1、慢头：根乱下 T2 先做完、等 T1 已追加（R2 等 T1 的 B2 罩住 7.15，R1 等 T1 的记录发出 6.1）；
    /// 根序下 F2 要等 F1，T2 落到 11.25。
    #[test]
    fn wait_multi_flight_yi_with_a_slow_head_waits_for_the_head_by_each_reading() {
        assert_eq!(wait(ROOTS_UNORDERED, AppendArm::YiReservedWhole, SLOW_HEAD, P1, 1, 2, R2).return_times, vec![9200, 7150]);
        assert_eq!(wait(ROOTS_UNORDERED, AppendArm::YiReservedWhole, SLOW_HEAD, P1, 1, 2, R1).return_times, vec![9200, 6100]);
        assert_eq!(wait(ROOTS_ORDERED, AppendArm::YiReservedWhole, SLOW_HEAD, P1, 1, 2, R2).return_times, vec![10200, 11250]);
    }

    /// 登记 8.2 判别力自证：把 Q2-3 的门槛 2 挪到两个取样点实测 ρ 之间，那一格的判定必须翻。
    #[test]
    fn moving_the_q2_3_threshold_between_the_two_cost_points_flips_one_judgement() {
        let at = |point: CostPoint, transaction_count: u32| wait(SHARED, AppendArm::JiaWhole, UNIFORM, point, 1, transaction_count, R2).w_max;
        let (p1_small, p1_large, p2_small, p2_large) = (at(P1, 8), at(P1, 64), at(P2, 8), at(P2, 64));
        assert_ne!(p1_large * p2_small, p2_large * p1_small, "两个取样点的 ρ 不同");
        // 门槛取两个 ρ 的中点：(ρ1 + ρ2) / 2 = (l1·s2 + l2·s1) / (2·s1·s2)。
        let midpoint = (p1_large * p2_small + p2_large * p1_small, 2 * p1_small * p2_small);
        let registered = (growth_judgement(p1_small, p1_large, REGISTERED_GROWS_THRESHOLD, REGISTERED_BOUNDED_THRESHOLD), growth_judgement(p2_small, p2_large, REGISTERED_GROWS_THRESHOLD, REGISTERED_BOUNDED_THRESHOLD));
        let moved = (growth_judgement(p1_small, p1_large, midpoint, (0, 1)), growth_judgement(p2_small, p2_large, midpoint, (0, 1)));
        assert_eq!(registered.1, GrowthJudgement::Grows);
        assert_ne!(moved.0, moved.1, "门槛挪到两点之间，两点的判定一个是长、一个不是");
    }

    #[test]
    fn wait_grid_pending_and_reserved_lag_trajectories_end_at_zero() {
        let grid = wait_grid();
        assert_eq!(grid.len(), 1344);
        assert!(grid.values().all(|result| result.pending.final_value == 0 && result.pending.peak_after_drain == 0));
        assert!(grid.values().all(|result| result.reserved_lag.is_none_or(|lag| lag.final_value == 0)));
        assert!(grid.iter().all(|(key, result)| result.return_times.len() == usize::try_from(key.6).expect("k ≤ 64")));
    }
}

/// 准入用的仓根：环境变量 SINGLEFS_REPOSITORY_ROOT，没设取当前目录（replay.sh 与实验页的命令都在仓根起）；不取「清单目录/../..」（单元键把它当运行期路径拒算）。
fn repository_root() -> std::path::PathBuf {
    let root = std::env::var_os("SINGLEFS_REPOSITORY_ROOT")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if !root.join(".claude/singlefs-ai-sop/scripts/preflight.py").is_file() {
        eprintln!("  ✗ {} 不是仓根（下面没有 .claude/singlefs-ai-sop/scripts/preflight.py），判不了准入", root.display());
        eprintln!("  → 怎么办：在仓根起这个装置，或设 SINGLEFS_REPOSITORY_ROOT=<仓根>");
        std::process::exit(78);
    }
    root
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let source = root.join("scripts/experiment-apparatus/e169_parallel_commit_append_and_prefix_recovery.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}
