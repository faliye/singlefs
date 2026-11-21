//! 历史里的盘、种子与伪随机源，一步操作与它抽出来的几样选择（内容长度、回退目标、按偏移写、截断、整理）。

use super::*;

/// 历史里的一块盘：内存盘外面包录制器（写与屏障进调用方给的那条流），录制器外面再包故障注入
/// （`crate::fault_injection::FaultInjectingBlockDevice`；不注入时只数调用）。
/// 注入层在最外面：报错的写、被吞掉的写与被吞掉的屏障都不进录制流。
pub type HistoryDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

/// 一段历史的种子。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HistorySeed(pub u64);

/// 手写的伪随机源：SplitMix64（Steele、Lea、Flood，OOPSLA 2014；`java.util.SplittableRandom` 的输出函数），不加依赖。
/// 同一个种子逐位复现同一串数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeededRandomSource {
    state: u64,
}

impl SeededRandomSource {
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_word(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }

    /// [0, `exclusive_upper_bound`) 里的一个数。取模的偏差在 2^64 面前可以不计：这里要的是复现，不是均匀到最后一位。
    ///
    /// # Panics
    /// `exclusive_upper_bound` 为 0（区间是空的）。
    pub fn below(&mut self, exclusive_upper_bound: u64) -> u64 {
        assert!(
            exclusive_upper_bound > 0,
            "区间 [0, 0) 是空的：调用方要先确认至少有一个可选项"
        );
        self.next_word() % exclusive_upper_bound
    }
}

/// 一段历史从哪里起。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HistoryStartingPoint {
    /// 只做过 mkfs、进程已退出：没有可写会话，要先挂载。
    AfterMakeFilesystem,
    /// mkfs 同一个进程里取号、暖机、发布第一个文件（E142 的 3000 字节），会话还开着（与 `tests/common` 的 `build_pool` 同一条路）。
    AfterFirstFile,
}

/// 发布的内容多长：执行时按数据单元的载荷容量（`data_unit_payload_capacity`）换成字节数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContentLength {
    Empty,
    /// [1, 载荷容量 − 2] 里按选择子取。
    InsideOneDataUnit {
        selector: u64,
    },
    OneByteBelowDataUnitPayloadCapacity,
    ExactlyDataUnitPayloadCapacity,
    /// 装不下：载荷容量 + 1。
    OneByteAboveDataUnitPayloadCapacity,
    /// 装不下：一个数据单元的整宽（含头与预留）。
    ExactlyDataUnitBytes,
    /// 跨 `data_units`（≥ 2）个数据单元的内容：前面的单元装满净荷容量，最后一个单元装 [1, 载荷容量] 里按选择子取的字节。
    /// 这一种走顺序写入口（[`ContentLength::whole_file_write_entry`]，`UserChange::SequentialWrite`）：覆盖写的契约是一个数据单元，
    /// 多于一片下段叶（144 个单元）的文件只有顺序写长得出来（代码轮 U13）。只有偏向删除多片下段叶那一组比重抽它
    /// （[`OverwriteContentDraw::AlsoSpanningSeveralDataUnits`]），别的几组比重的抽法与生成的历史不变。
    SpanningDataUnitsThroughTheSequentialWrite {
        data_units: u64,
        selector: u64,
    },
}

/// 整份重写文件内容走哪个入口（`mounted_session::UserChange` 的哪一个成员）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WholeFileWriteEntry {
    /// `UserChange::Overwrite`：契约是一个数据单元，装不下拒。
    Overwrite,
    /// `UserChange::SequentialWrite`：按切分纪律切成几个一单元事务，不按单元数拒。
    SequentialWrite,
}

impl ContentLength {
    #[must_use]
    pub fn in_bytes(self) -> usize {
        let capacity = data_unit_payload_capacity();
        match self {
            ContentLength::Empty => 0,
            ContentLength::InsideOneDataUnit { selector } => {
                let choices = u64::try_from(capacity - 2).expect("载荷容量三万多字节");
                1 + usize::try_from(selector % choices).expect("小于载荷容量")
            }
            ContentLength::OneByteBelowDataUnitPayloadCapacity => capacity - 1,
            ContentLength::ExactlyDataUnitPayloadCapacity => capacity,
            ContentLength::OneByteAboveDataUnitPayloadCapacity => capacity + 1,
            ContentLength::ExactlyDataUnitBytes => usize::try_from(DATA_UNIT_BYTES).expect("32768"),
            ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
                data_units,
                selector,
            } => {
                let capacity_in_bytes = u64::try_from(capacity).expect("载荷容量三万多字节");
                let bytes_of_the_full_units = (data_units - 1) * capacity_in_bytes;
                let bytes_of_the_last_unit = 1 + selector % capacity_in_bytes;
                usize::try_from(bytes_of_the_full_units + bytes_of_the_last_unit)
                    .expect("几百个单元的内容装得进 usize")
            }
        }
    }

    /// 这一种内容走哪个入口：跨几个数据单元的走顺序写，别的（含装不下一个单元的那两种：它们要钉的正是覆盖写的拒）走覆盖写。
    #[must_use]
    pub fn whole_file_write_entry(self) -> WholeFileWriteEntry {
        match self {
            ContentLength::Empty
            | ContentLength::InsideOneDataUnit { .. }
            | ContentLength::OneByteBelowDataUnitPayloadCapacity
            | ContentLength::ExactlyDataUnitPayloadCapacity
            | ContentLength::OneByteAboveDataUnitPayloadCapacity
            | ContentLength::ExactlyDataUnitBytes => WholeFileWriteEntry::Overwrite,
            ContentLength::SpanningDataUnitsThroughTheSequentialWrite { .. } => {
                WholeFileWriteEntry::SequentialWrite
            }
        }
    }

    /// 统计表里的名字（选择子不进名字）。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ContentLength::Empty => "0 字节",
            ContentLength::InsideOneDataUnit { .. } => "[1, 载荷容量 − 2]",
            ContentLength::OneByteBelowDataUnitPayloadCapacity => "载荷容量 − 1",
            ContentLength::ExactlyDataUnitPayloadCapacity => "载荷容量",
            ContentLength::OneByteAboveDataUnitPayloadCapacity => "载荷容量 + 1（装不下）",
            ContentLength::ExactlyDataUnitBytes => "数据单元整宽 32768（装不下）",
            ContentLength::SpanningDataUnitsThroughTheSequentialWrite { data_units, .. } => {
                if data_units <= EXTENT_TREE_LOWER_LEAF_DATA_UNITS {
                    "跨一片下段叶以内的多个单元（顺序写）"
                } else if data_units <= 2 * EXTENT_TREE_LOWER_LEAF_DATA_UNITS {
                    "跨两片下段叶（顺序写）"
                } else {
                    "跨三片及以上下段叶（顺序写）"
                }
            }
        }
    }
}

/// 一次发布的内容：长度与填充的种子（字节由种子现算，同一个选择逐字节复现）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentChoice {
    pub length: ContentLength,
    pub fill_seed: u64,
}

impl ContentChoice {
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        let length = self.length.in_bytes();
        let mut source = SeededRandomSource::from_seed(self.fill_seed);
        let mut bytes = Vec::with_capacity(length + 8);
        while bytes.len() < length {
            bytes.extend_from_slice(&source.next_word().to_le_bytes());
        }
        bytes.truncate(length);
        bytes
    }
}

/// 回退的目标怎么取：执行时按 checker 的读法从盘上读根环（与实现的择根不共用代码）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RollbackTargetChoice {
    /// 根环里的一条可读根：按 (txg, 实例) 从新到旧排、去重，取第 `index_from_newest mod 条数` 条。
    /// 候选集里的与候选集外的（被抛弃时间线的、F 之下的、树表 0 条的）都会落到。
    RingRoot { index_from_newest: u64 },
    /// 不在根环里的目标：最新那条根的实例、txg = 最新 txg + 1 + `txg_beyond_newest mod 3`。
    BeyondNewestRoot { txg_beyond_newest: u64 },
    /// 候选集的下沿：根环里 txg 等于最新那条根带的 F 的那条根（同一个 txg 上有几条取实例最大的）；F 那一代已被盖掉时取最新那条。
    /// 只有 `RollbackTargetDraw::FloorRootHalfTheTime` 抽它（理想模型那一格 B2：回退到 txg = F_生效 的根被拒）。
    RingRootAtTheNewestFloor,
}

/// 回退的目标按什么抽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RollbackTargetDraw {
    /// 第一版的抽法：五分之二取最近四条、五分之二在整个根环里均匀取、五分之一取根环之外。随机历史的默认批、设了变量的那一批与第 121 行的取样点用它。
    RecentUniformOrBeyond,
    /// 先抽一次：一半取候选集的下沿（`RingRootAtTheNewestFloor`），另一半照第一版的抽法。
    FloorRootHalfTheTime,
    /// 先抽一次：一半取根环里第四新的那条根（`RingRoot { index_from_newest: 3 }`），另一半照第一版的抽法。unlink 一个两片下段叶的文件
    /// 一次写出三条根（意图那次、一个删除批、最后一批）；删完的那一版没有文件、回退不接（执行器记「前提不满足」），建一个 inode 之后才接，
    /// 这时第四新的根就是意图那一版（三片的是第一个删除批那一版），回退到它把号带回队列（D8（核心索引结构） 已定项 16 依据
    /// 「回退到意图那一版会把号带回队列」）、之后的用户改动先走空多步（代码轮 U13）。
    FourthNewestRootHalfTheTime,
}

/// 覆盖写（`HistoryOperation::PublishOverwrite`）的内容按什么抽。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverwriteContentDraw {
    /// 第一版的六种长度（`draw_content`）：都在一个数据单元以内或刚越过它，全走覆盖写入口。
    WithinOneDataUnitOrJustBeyond,
    /// 另抽跨几个数据单元、走顺序写入口的内容（[`ContentLength::SpanningDataUnitsThroughTheSequentialWrite`]），
    /// 多半多于一片下段叶（`draw_content_spanning_several_data_units`）。
    AlsoSpanningSeveralDataUnits,
}

/// 抬 F 的目标怎么取。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FloorTargetChoice {
    /// 新 F = 现行版本根记录的 F + `steps_above_current_floor mod (现行 txg − 现行 F + 3)`：最大到现行 txg + 2，
    /// 而上限（D16（发布语义） 已定项 1）不超过每块盘上最新的有效根 ≤ 现行 txg，所以超过上限的目标一定取得到。
    /// 生成器的前提：目标只取现行 F 及以上，不往下抬。没有条款明写，按入口的文档注释取的读法——
    /// `crates/singlefs-core/src/mount.rs` 的 `raise_rollback_floor` 文档注释「抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）」，已定项 1 在
    /// `.claude/kb/decisions/16-发布语义.md`「已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读」；往下的目标这个入口接不接、接了算不算合法没有条款（F 回落在增补 2 收口表第 ② 行里打回重议）。
    /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着）。
    pub steps_above_current_floor: u64,
}

/// 读、删哪一个快照：执行时按现行那一版树表里不在销毁中的码 14 条目（按（诞生 txg, 树号）升序，即建的先后）取第
/// `index_among_the_snapshots mod 个数` 个；一个都没有就记「前提不满足」。生成时不知道那一刻有几个快照，收缩删掉前面几步之后照样有意义。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SnapshotChoice {
    pub index_among_the_snapshots: u64,
}

/// 历史里的一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HistoryOperation {
    /// `transaction::publish_first_file`，接在现行版本后面（上一条记录取现行版本的；照抄实例表指针的那条根也取现行版本的）。
    PublishFirstFile(ContentChoice),
    /// 整份重写文件内容，经挂着的会话发，接在现行的带文件的版本后面：内容跨几个数据单元的
    /// （[`ContentLength::SpanningDataUnitsThroughTheSequentialWrite`]）走顺序写（`UserChange::SequentialWrite`，代码轮 U13），
    /// 别的走覆盖写（`UserChange::Overwrite`，`transaction::publish_overwrite`）。
    PublishOverwrite(ContentChoice),
    /// `transaction::publish_without_units`，接在现行的树表 0 条的版本后面。
    PublishWithoutUnits,
    /// 关掉这个进程的可写会话，`mount::mount_writable`。
    CloseAndMountWritable,
    /// 管理员回退：挂着时的一次向前发布（`mount::roll_back_by_a_forward_publish`），接在现行的带文件的版本上。
    RollBackWhileMounted(RollbackTargetChoice),
    /// `mount::raise_rollback_floor`（影子账开着），接在现行的带文件的版本上。
    RaiseRollbackFloor(FloorTargetChoice),
    /// 关掉这个进程的可写会话，冷启动 `recovery::recover`（看 journal）。
    ColdStartRecover,
    /// 崩溃恢复抛弃根（D23（journal 的角色与格式） 已定项 14：影子账与按实例表判抛弃留着，理由是崩溃恢复）：这个进程崩掉，
    /// 下一次可写挂载时根环里最新那条根（这个会话的现行版本）的根槽与它那次发布的记录点名的单元整次挂载读不出（读回全 0；
    /// `DeviceReadingZerosOverHiddenRanges`：录制流里不多出写，挂载自己写到那几处的照常落盘）。那条根系统配置见证过，
    /// C554 乙之后读阶段重读一次仍读不出、取号之前拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`），
    /// 这一步不再造出被抛弃的根；乙罩不到的「系统配置没见证到最新那条根」那一形（`tests/common/mod.rs` 的
    /// `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 另把见证它的系统配置槽清掉）这一步不造。
    CrashRecoveryAbandoningTheNewestRoot,
    /// 经挂着的会话 unlink 一个号（`mounted_session::UserChange::Unlink`，D8（核心索引结构） 已定项 16）：一次调用 = 意图那次 +
    /// 从尾部全部分批（文件多于一片下段叶的 144 个单元时先有不是最后一批的删除批，再最后一批），中间为空间推的抬 F 照会话交回的比。
    /// 删哪个号执行时按号选（`inode_the_unlink_names`：树里只有第一个文件时就是它；第一个文件旁边还有别的 inode 时第一个文件、带数据的别的文件、
    /// size 0 的别的文件都删得到，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M4）。
    /// 现行那一版有第一个文件那条记录（或第一个文件已删完而还有带数据单元的别的文件活着）、inode 树只有一片叶容器时调（删除中的那一版也调：
    /// 会话先走空、再删或拒）。
    Unlink,
    /// 经挂着的会话建一个 size 0 的 inode（`mounted_session::UserChange::NewInodes`，count 1；D8（核心索引结构） 已定项 16 第 5 条：
    /// 删完之后只许建 inode、写行与空发布）。只在第一个文件已删完的一版上调（有树而文件集合为空的一版、或在它上面建过 inode 的一版）：
    /// 在第一个文件旁边建 inode 的是 [`HistoryOperation::CreateInodes`]（只进按 inode 号写那一组比重），这一步照旧只接在删完之后，别的几组生成的历史与执行不变。
    CreateInode,
    /// 经挂着的会话整份重写文件内容（入口分法同 [`HistoryOperation::PublishOverwrite`]），两块盘在那次改动的第一个 journal 记录写
    /// 到达时掉电：之前发出的写与屏障都落了盘，从那个写起一个都不落、都报 I/O 错（这一份里的 `DeviceLosingPowerAtTheFirstJournalRecordWrite`），
    /// 这个进程随之崩掉；之后照 [`HistoryOperation::CloseAndMountWritable`] 可写挂载一次。那次改动写下的单元是这个会话那个实例的孤儿：
    /// 下一个实例写行时给它写的那一行按所选根判它们未发布，根环里这个实例的根全被轮出去之后，挂载末尾的实例表清扫抹它们的头、
    /// 再发清扫发布删那一行（D18（块里携带什么信息） 已定项 11「回收」一条：抹头那一段只有这样的孤儿走得到）。
    /// 掉电截在第一条 journal 记录之前，那次改动没有任何一样能让恢复读到它的内容：交给模型与 RefFS 时与可写挂载相同。
    OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(ContentChoice),
    /// 关掉这个进程的可写会话，可写挂载期间每块盘自证过、世代号最大的那一槽系统配置读回全 0（`DeviceReadingZerosOverHiddenRanges`）。
    /// 读回全 0 的槽读得出而自证不过、设备不报错，每块盘只剩一槽自证过、没有一块盘两槽都自证过：乙入判不出（C971（多处读坏时可写挂载写出
    /// 倒挂的新实例根），`research/prompts/e205-preregistration.md`「乙入判不出」那一行）把这几槽并进 R2 当读错计，判据 N-配置 按真处置，
    /// 重读一次仍读回全 0，取号之前拒可写、一个写都不发；模型照条款答拒（`IdealModel::answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots`）。
    /// 只进 [`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS`] 那一组，占小份数（带文件那张表与 unlink 之后那张表
    /// 各 1 份），那一组的取样点随机抽得到它；`history::applying_mounts_crash_recovery_and_rollbacks::tests` 里另有一段写死的历史钉它。
    WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
    /// 关掉这个进程的可写会话，可写挂载期间只有第一块盘自证过、世代号最大的那一槽系统配置读回全 0（单故障：D22（单元原子性怎么合成）
    /// 已定项 8「容错承诺是单故障：每盘一槽系统配置坏在承诺里」；`DeviceReadingZerosOverHiddenRanges`，挂载自己写到那一槽之后照常读得出）。
    /// 读回全 0 的槽自证不过、设备不报错；另一块盘两槽都自证过，R2 判得出（乙入判不出不成立），可写挂载照「一槽自证不过、另一槽自证过」往下走
    /// （同条 S-辨），取号那两轮写落到这块盘的两槽上：崩溃注入里在读故障下真的往盘上发写的步（C696（崩溃注入的读故障一支在随机历史里从没触发））。
    /// 只进它自己那一组比重（[`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS`]），已有各组生成的历史不变。
    WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros,
    /// 经挂着的会话建一个只读快照（`mounted_session::UserChange::CreateSnapshot`，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
    /// 第三节 S2）：会话先走空，再发一次独立发布。现行那一版有树时调（有树而文件集合为空的一版也调：会话照常发布、读它读到空，
    /// 「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F5）。
    /// 只进它自己那一组比重（[`GenerationWeights::SNAPSHOT_OPERATIONS`]），已有各组生成的历史不变。
    CreateSnapshot,
    /// 读一个只读快照（`recovery::read_the_first_file_of_a_snapshot`，经现行那一版的根）：不写盘，读出的内容交模型比（判决 S7「读快照」）。
    ReadSnapshot(SnapshotChoice),
    /// 经挂着的会话删一个只读快照（`mounted_session::UserChange::DestroySnapshot`，判决 S4 与第七节岔路 2、4）：一次调用 = 先走空 + 意图 +
    /// 全部批，中间为空间推的抬 F 照会话交回的比。
    DestroySnapshot(SnapshotChoice),
    /// 挂着时整理一趟（`mounted_session::MountedSession::run_compaction`，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7「对拍：
    /// `HistoryOperation` 加 `CompactWhileMounted`（含签发、批、批间崩溃）」与 G8 甲）：会话的分配器装上只供测试的整理开关
    /// （`compaction_plan::CompactionSwitches`：强制批大小与选段，执行器的可写挂载叫停整理、别处走不进去），推完换回产品值。一趟里先走空删除续做，
    /// 再签发（一次只写 logged_ops 的发布）、一批一批搬、最后一批同一次发布删意图；每一版交模型比（[`IdealModel::answer_compaction_publish`]：
    /// 只比做成的那几次发布的内容与意图在 logged_ops 里的样子，搬几批、停在哪模型不答）。
    /// [`CompactionRunEnding::CrashesAfterBatchesThenMountsWritable`] 那一种推够批数之后这个进程崩掉、照 [`HistoryOperation::CloseAndMountWritable`]
    /// 可写挂载一次。只进整理那几组比重（[`GenerationWeights::COMPACTION_WHILE_MOUNTED`]、
    /// [`GenerationWeights::COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`]，与故障注入用的
    /// [`GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED`]），已有各组生成的历史不变。
    CompactWhileMounted(CompactionChoice),
    /// 经挂着的会话改一个 inode 的 size（`mounted_session::UserChange::Truncate`，判决 `research/prompts/m4-trunc-r1-main-verification.md`
    /// 第三节 K8）：一次调用 = 先走空 + 改 size 那一次（缩短是截断意图，变长与 N = S 一次发布做完）+ 缩短时之后的全部批，中间为空间推的抬 F
    /// 照会话交回的比，每一版交模型比（[`IdealModel::answer_truncation`]）。目标 N 按类在执行时取（[`TruncationChoice`]：生成时不知道那一刻
    /// 文件多长）。现行那一版有第一个文件那条记录时改第一个文件；没有而删完之后建过 inode 时改其中一个（第一版拒，调度记录「截甲交回」第 ① 处）。
    /// 只进截断那一组比重（[`GenerationWeights::TRUNCATION`]）与带整理的那一组（[`GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED`]），
    /// 别的几组生成的历史不变。
    Truncate(TruncationChoice),
    /// 经挂着的会话一次建几个 size 0 的 inode（`mounted_session::UserChange::NewInodes`，里程碑「删除与多次 COW」一·1：按 inode 号写数据先要有
    /// 别的 inode）：第一个文件活着或已删完的一版上都调（[`IdealModel::answer_create_inodes`]）。建几个执行时压到最右那片叶容器装得下的条数
    /// （[`InodeCountChoice`]）。只进按 inode 号写那一组比重（[`GenerationWeights::WRITES_BY_INODE_NUMBER`]），别的几组生成的历史不变。
    CreateInodes(InodeCountChoice),
    /// 经挂着的会话按 inode 号整份写一个文件（`UserChange::OverwriteOfAnInode`、`UserChange::SequentialWriteOfAnInode`，里程碑「删除与多次 COW」
    /// 一·1）：号执行时在现行那一版树里活着的号里取（[`WriteOfAnInodeChoice::inode_among`]），入口按内容长度分（同 [`HistoryOperation::PublishOverwrite`]）。
    /// 第一个文件已删完、之后建过 inode 的一版上照调，必拒（K-U5「拦」）。只进按 inode 号写那一组比重。
    WriteOfAnInode(WriteOfAnInodeChoice),
    /// 经挂着的会话按偏移写一个活 inode（`mounted_session::UserChange::WriteAtOffset`，判决
    /// `research/prompts/m4-partial-write-r1-main-verification.md` 第二节）：号执行时在现行那一版树里活着的号里取（同
    /// [`WriteOfAnInodeChoice::inode_among`]），偏移与长度执行时按那一刻那个文件的 size 与单元数取（[`PartialWriteChoice`]），
    /// 会话交回的每一版交模型比（[`IdealModel::answer_partial_write`]）。只进按偏移写那一组比重（[`GenerationWeights::PARTIAL_WRITES`]），
    /// 已有各组生成的历史不变。
    WriteAtOffset(PartialWriteChoice),
}

/// 按偏移写的偏移落在哪一格（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R2、R5 那几格：已有单元中间、盖满一个单元、
/// 跨末尾、起点越过末尾留空隙、写进洞里）。生成时从前五格里抽一格（[`PartialWriteOffsetCell::DRAWN`]），执行时按那一刻文件的 size 与单元数换成偏移与
/// 长度（[`PartialWriteChoice::offset_and_length_in_bytes`]）；计数按这次写真落到的那一格记（[`PartialWriteOffsetCell::of_the_write`]）——文件太短、
/// 没有洞时抽到的那一格落不到，换成别的格，不冒充。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartialWriteOffsetCell {
    /// 落在一个已有单元里面、没盖满它，终点不越过末尾。
    InsideAnExistingDataUnit,
    /// 盖满至少一个已有单元的整个净荷（`[单元号 × 净荷, (单元号 + 1) × 净荷)`）。
    CoveringAWholeExistingDataUnit,
    /// 起点在末尾之内或恰在末尾（不在洞里），终点越过末尾：同时是一次变长。
    AcrossTheEnd,
    /// 起点越过末尾：写之前的末尾到偏移之间是空隙，补整单元零（R5）。
    PastTheEndLeavingAGap,
    /// 起点落在尾部的洞里（存在的单元之后、size 之内）。
    IntoTheHole,
    /// 只在归格时出现、生成时不抽：落在 size 之内、跨了单元边界而一个已有单元都没盖满。
    WithinTheSizeAcrossADataUnitBoundary,
}

impl PartialWriteOffsetCell {
    /// 生成时抽的五格（那五格）。
    pub const DRAWN: [PartialWriteOffsetCell; 5] = [
        PartialWriteOffsetCell::InsideAnExistingDataUnit,
        PartialWriteOffsetCell::CoveringAWholeExistingDataUnit,
        PartialWriteOffsetCell::AcrossTheEnd,
        PartialWriteOffsetCell::PastTheEndLeavingAGap,
        PartialWriteOffsetCell::IntoTheHole,
    ];

    /// 统计表里的名字。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            PartialWriteOffsetCell::InsideAnExistingDataUnit => "已有单元中间",
            PartialWriteOffsetCell::CoveringAWholeExistingDataUnit => "盖满一个单元",
            PartialWriteOffsetCell::AcrossTheEnd => "跨末尾",
            PartialWriteOffsetCell::PastTheEndLeavingAGap => "起点越过末尾留空隙",
            PartialWriteOffsetCell::IntoTheHole => "写进洞里",
            PartialWriteOffsetCell::WithinTheSizeAcrossADataUnitBoundary => {
                "size 之内跨单元边界、没盖满一个单元"
            }
        }
    }

    /// 写之前 size 为 `size_in_bytes`、存在 `data_units_present` 个单元（从 0 起连号）的文件上写 `[offset_in_bytes, offset_in_bytes + length_in_bytes)`
    /// 落在哪一格。次序：起点越过末尾、起点在洞里、盖满一个已有单元、终点越过末尾、落在一个单元里，都不是的是跨单元边界那一格。`length_in_bytes`
    /// 至少 1。
    #[must_use]
    pub fn of_the_write(
        size_in_bytes: u64,
        data_units_present: u64,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> PartialWriteOffsetCell {
        let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
        let end_exclusive = offset_in_bytes.saturating_add(length_in_bytes);
        let end_of_the_units_present = data_units_present.saturating_mul(payload);
        let first_unit_starting_within_the_write = offset_in_bytes.div_ceil(payload);
        let covers_a_whole_existing_unit = first_unit_starting_within_the_write
            < data_units_present
            && (first_unit_starting_within_the_write + 1).saturating_mul(payload) <= end_exclusive;
        if offset_in_bytes > size_in_bytes {
            PartialWriteOffsetCell::PastTheEndLeavingAGap
        } else if end_of_the_units_present <= offset_in_bytes && offset_in_bytes < size_in_bytes {
            PartialWriteOffsetCell::IntoTheHole
        } else if covers_a_whole_existing_unit {
            PartialWriteOffsetCell::CoveringAWholeExistingDataUnit
        } else if end_exclusive > size_in_bytes {
            PartialWriteOffsetCell::AcrossTheEnd
        } else if offset_in_bytes / payload == (end_exclusive - 1) / payload {
            PartialWriteOffsetCell::InsideAnExistingDataUnit
        } else {
            PartialWriteOffsetCell::WithinTheSizeAcrossADataUnitBoundary
        }
    }
}

/// 按偏移写哪个、写在哪一格、写什么（[`HistoryOperation::WriteAtOffset`]）：生成时不知道那一刻文件多长，偏移与长度执行时按格与选择子取，
/// 字节由填充种子按那个长度现算（同一个选择、同一个文件逐字节复现）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PartialWriteChoice {
    /// 写哪个号：同 [`WriteOfAnInodeChoice::inode_among`]。
    pub inode_selector: u64,
    pub cell: PartialWriteOffsetCell,
    pub selector: u64,
    pub fill_seed: u64,
}

impl PartialWriteChoice {
    /// 从现行那一版树里活着的号（升序）里取写哪个：第一个文件活着而选择子不是 4 的倍数时取它（按偏移写第一个文件是这一组要罩的大头，
    /// RefFS 也只翻得出它；建出来的别的号多半 size 0，写上去只落得到「跨末尾」那一格），别的照 [`WriteOfAnInodeChoice::inode_among`]
    /// （选择子是 3 的倍数取最大的号，否则按选择子取模）。一个号都没有交回 `None`。
    #[must_use]
    pub fn inode_among(self, live_inodes_in_ascending_order: &[u64]) -> Option<u64> {
        let first_file = singlefs_core::transaction::FIRST_INODE_NUMBER;
        if !self.inode_selector.is_multiple_of(4)
            && live_inodes_in_ascending_order.contains(&first_file)
        {
            return Some(first_file);
        }
        WriteOfAnInodeChoice {
            inode_selector: self.inode_selector,
            content: ContentChoice {
                length: ContentLength::Empty,
                fill_seed: self.fill_seed,
            },
        }
        .inode_among(live_inodes_in_ascending_order)
    }

    /// size 为 `size_in_bytes`、存在 `data_units_present` 个单元的文件上这一步写的偏移与长度（长度至少 1、至多两个净荷出头，都装得进内存）：
    /// - 已有单元中间：在第 `选择子 mod 单元数` 个单元的声明长度以内取一段，短于一个净荷（不盖满它）；文件是空的时写偏移 0；
    /// - 盖满一个单元：从那个单元起点往前至多半个净荷起，盖住它整个净荷再多至多半个净荷；一个单元都没有时从 0 写一个净荷；
    /// - 跨末尾：从末尾往前退 1 到最后一个单元的声明长度减 1 起（不从它的起点起，不盖满它），越过末尾 1 到两个净荷；
    /// - 起点越过末尾留空隙：从末尾往后 1 到两个净荷起，写 1 到一个净荷；
    /// - 写进洞里：起点在存在的单元之后、size 之内，写 1 到一个净荷；没有洞时照「已有单元中间」取。
    #[must_use]
    pub fn offset_and_length_in_bytes(
        self,
        size_in_bytes: u64,
        data_units_present: u64,
    ) -> (u64, u64) {
        let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
        let declared_length_of =
            |unit: u64| size_in_bytes.saturating_sub(unit * payload).min(payload);
        let lower_selector = self.selector;
        let middle_selector = self.selector >> 21;
        let upper_selector = self.selector >> 42;
        let inside_an_existing_unit = || {
            if size_in_bytes == 0 || data_units_present == 0 {
                return (0, 1 + lower_selector % 64);
            }
            let unit = lower_selector % data_units_present;
            let declared_length = declared_length_of(unit).max(1);
            let offset_in_the_unit = middle_selector % declared_length;
            let longest = (declared_length - offset_in_the_unit)
                .min(payload - 1)
                .max(1);
            (
                unit * payload + offset_in_the_unit,
                1 + upper_selector % longest,
            )
        };
        match self.cell {
            PartialWriteOffsetCell::InsideAnExistingDataUnit
            | PartialWriteOffsetCell::WithinTheSizeAcrossADataUnitBoundary => {
                inside_an_existing_unit()
            }
            PartialWriteOffsetCell::CoveringAWholeExistingDataUnit => {
                if data_units_present == 0 {
                    return (0, payload);
                }
                let unit_start = (lower_selector % data_units_present) * payload;
                let before_the_unit = middle_selector % (unit_start.min(payload / 2) + 1);
                (
                    unit_start - before_the_unit,
                    before_the_unit + payload + upper_selector % (payload / 2),
                )
            }
            PartialWriteOffsetCell::AcrossTheEnd => {
                if size_in_bytes == 0 || data_units_present == 0 {
                    return (size_in_bytes, 1 + lower_selector % (2 * payload));
                }
                let last_declared_length = declared_length_of(data_units_present - 1).max(1);
                let back_from_the_end = 1 + lower_selector % (last_declared_length - 1).max(1);
                let offset = size_in_bytes - back_from_the_end.min(size_in_bytes);
                // 起点恰在一个单元的起点上时，写得短于一个净荷，不盖满它。
                let beyond_the_end = if offset.is_multiple_of(payload) {
                    1 + middle_selector % (payload - back_from_the_end - 1).max(1)
                } else {
                    1 + middle_selector % (2 * payload)
                };
                (offset, size_in_bytes - offset + beyond_the_end)
            }
            PartialWriteOffsetCell::PastTheEndLeavingAGap => (
                size_in_bytes + 1 + lower_selector % (2 * payload),
                1 + middle_selector % payload,
            ),
            PartialWriteOffsetCell::IntoTheHole => {
                let end_of_the_units_present = data_units_present * payload;
                if end_of_the_units_present >= size_in_bytes {
                    return inside_an_existing_unit();
                }
                (
                    end_of_the_units_present
                        + lower_selector % (size_in_bytes - end_of_the_units_present),
                    1 + middle_selector % payload,
                )
            }
        }
    }

    /// 这一步写的字节：`length_in_bytes` 个，由填充种子现算。
    #[must_use]
    pub fn bytes_of_length(self, length_in_bytes: u64) -> Vec<u8> {
        let length = usize::try_from(length_in_bytes).expect("至多两个净荷出头，装得进 usize");
        let mut source = SeededRandomSource::from_seed(self.fill_seed);
        let mut bytes = Vec::with_capacity(length + 8);
        while bytes.len() < length {
            bytes.extend_from_slice(&source.next_word().to_le_bytes());
        }
        bytes.truncate(length);
        bytes
    }
}

/// 执行器交给会话的一次按偏移写（按步号，[`HistoryRun::partial_writes_requested`]）：执行时取的号、偏移、长度与真落到的格。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartialWriteRequested {
    pub inode: u64,
    pub offset_in_bytes: u64,
    pub length_in_bytes: u64,
    pub offset_cell: PartialWriteOffsetCell,
}

/// 一次建几个 inode（[`HistoryOperation::CreateInodes`]）：执行时压到最右那片叶容器还装得下的条数（满 233 条在末尾分裂，一次只重写一片叶容器，
/// 模型只答这一形），至少 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InodeCountChoice {
    pub count_drawn: u64,
}

impl InodeCountChoice {
    /// 树里此刻有 `live_inode_records` 条活着的记录时这一步建几个：抽到的数，压到最右那片叶容器还装得下的条数，至少 1。
    #[must_use]
    pub fn count_fitting_the_rightmost_leaf_container(self, live_inode_records: u64) -> u64 {
        let room_in_the_rightmost_leaf_container =
            INODE_LEAF_RECORDS - live_inode_records % INODE_LEAF_RECORDS;
        self.count_drawn
            .clamp(1, room_in_the_rightmost_leaf_container)
    }
}

/// 一次建 inode 至多抽几个（[`HistoryOperation::CreateInodes`]）：抽到这么多时号能越过 extent 树上段一片叶罩的 143 个，让按 inode 号写的那一组
/// 取样点罩到「上段长高」与「写的号落在第 0 片叶之外」两形。
pub(super) const INODES_CREATED_AT_ONCE_DRAWN_AT_MOST: u64 = 160;

/// 一次建 inode 抽得少的那一半至多抽几个（[`HistoryOperation::CreateInodes`]）：树里的 inode 少，随机历史里的 unlink 删得到删空，
/// inode 树留在一片叶容器里（模型的删除那一臂只答一片的）。
pub(super) const INODES_CREATED_AT_ONCE_IN_A_SMALL_DRAW_AT_MOST: u64 = 3;

/// 按 inode 号写哪个、写什么（[`HistoryOperation::WriteOfAnInode`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WriteOfAnInodeChoice {
    pub inode_selector: u64,
    pub content: ContentChoice,
}

impl WriteOfAnInodeChoice {
    /// 从现行那一版树里活着的号（升序）里取写哪个：选择子是 3 的倍数时取最大的那个（新建的号多半在上段第 0 片叶之外），否则按选择子取模。
    /// 生成时不知道那一刻树里有哪几个号，收缩删掉前面几步之后照样有意义。一个号都没有交回 `None`。
    #[must_use]
    pub fn inode_among(self, live_inodes_in_ascending_order: &[u64]) -> Option<u64> {
        let largest = *live_inodes_in_ascending_order.last()?;
        if self.inode_selector.is_multiple_of(3) {
            return Some(largest);
        }
        let count = u64::try_from(live_inodes_in_ascending_order.len()).expect("条数装得进 u64");
        let index = usize::try_from((self.inode_selector / 3) % count).expect("小于条数");
        Some(live_inodes_in_ascending_order[index])
    }
}

/// 截断改到多长（[`HistoryOperation::Truncate`]）：执行时按那一刻文件的 size 与单元数把类与选择子换成字节数（[`TruncationChoice::new_size_in_bytes`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TruncationChoice {
    pub target: TruncationTarget,
    pub selector: u64,
}

/// 截断目标的类（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8「目标按类取」）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TruncationTarget {
    /// 缩到 0（K3：留一个声明长度 0 的单元）。
    ToZero,
    /// 缩到一个单元边界 k × 净荷（k 在 [1, 罩着现在 size 的单元数 − 1] 里按选择子取；文件只罩一个单元时取 0）。
    ToADataUnitBoundary,
    /// 缩到一个单元中间（k × 净荷 + r，r 在 [1, 净荷 − 1] 里，比现在短；现在不到一个净荷时在 [0, size) 里取）。
    IntoTheMiddleOfADataUnit,
    /// 在最后一个罩着的单元里变长（到那个单元的末尾为止；现在 size 恰在单元末尾时多 1 到半个净荷，跨进下一个单元留洞）。
    GrowingWithinTheLastDataUnit,
    /// 跨单元变长：多 1 到 3 个净荷再加 [0, 净荷) 字节（K5：留洞）。
    GrowingAcrossDataUnits,
    /// 变长之后截回洞里：文件尾部有洞（存在的单元少于 size 罩的）时，截到存在的单元末尾与 size 之间（含恰在末尾：洞整个截掉）；
    /// 没有洞时照 [`TruncationTarget::ToTheCurrentSize`]。
    BackIntoTheHole,
    /// 改成现在的 size（K12：只改记录）；收缩往这一头换。
    ToTheCurrentSize,
}

impl TruncationTarget {
    /// 统计表里的名字。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            TruncationTarget::ToZero => "缩到 0",
            TruncationTarget::ToADataUnitBoundary => "缩到单元边界",
            TruncationTarget::IntoTheMiddleOfADataUnit => "缩到单元中间",
            TruncationTarget::GrowingWithinTheLastDataUnit => "单元内变长",
            TruncationTarget::GrowingAcrossDataUnits => "跨单元变长",
            TruncationTarget::BackIntoTheHole => "变长后截回洞里",
            TruncationTarget::ToTheCurrentSize => "改成现在的 size",
        }
    }
}

impl TruncationChoice {
    /// 第一个文件现在 `size_in_bytes` 字节、盘上存在 `data_units_present` 个数据单元时这一步截到多少字节。都装得进内存（至多现在的 size
    /// 加 4 个净荷）。
    #[must_use]
    pub fn new_size_in_bytes(self, size_in_bytes: u64, data_units_present: u64) -> u64 {
        let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
        let data_units_within_the_size = size_in_bytes.div_ceil(payload).max(1);
        match self.target {
            TruncationTarget::ToZero => 0,
            TruncationTarget::ToADataUnitBoundary => {
                let inner_boundaries = data_units_within_the_size - 1;
                if inner_boundaries == 0 {
                    0
                } else {
                    (1 + self.selector % inner_boundaries) * payload
                }
            }
            TruncationTarget::IntoTheMiddleOfADataUnit => {
                if size_in_bytes <= payload {
                    self.selector % size_in_bytes.max(1)
                } else {
                    let unit = self.selector % (data_units_within_the_size - 1);
                    let middle = unit * payload + 1 + (self.selector / 7) % (payload - 1);
                    middle.min(size_in_bytes - 1)
                }
            }
            TruncationTarget::GrowingWithinTheLastDataUnit => {
                let end_of_the_last_unit = data_units_within_the_size * payload;
                if size_in_bytes < end_of_the_last_unit {
                    size_in_bytes + 1 + self.selector % (end_of_the_last_unit - size_in_bytes)
                } else {
                    size_in_bytes + 1 + self.selector % (payload / 2)
                }
            }
            TruncationTarget::GrowingAcrossDataUnits => {
                size_in_bytes + (1 + self.selector % 3) * payload + (self.selector / 3) % payload
            }
            TruncationTarget::BackIntoTheHole => {
                let end_of_the_units_present = data_units_present * payload;
                if end_of_the_units_present < size_in_bytes {
                    end_of_the_units_present
                        + self.selector % (size_in_bytes - end_of_the_units_present)
                } else {
                    size_in_bytes
                }
            }
            TruncationTarget::ToTheCurrentSize => size_in_bytes,
        }
    }
}

/// 第一个文件删完之后建的 inode 改成多少字节：[1, 3 × 净荷] 里按选择子取（比它现在的 size 长是变长、短是截断、相同是只改记录，
/// 模型都答，[`IdealModel::answer_size_change_of_an_inode_other_than_the_first`]）。
pub(super) fn new_size_of_an_inode_created_after_the_first_file_was_deleted(selector: u64) -> u64 {
    let payload = u64::try_from(data_unit_payload_capacity()).expect("净荷容量装得进 u64");
    1 + selector % (3 * payload)
}

/// 整理一趟怎么推（[`HistoryOperation::CompactWhileMounted`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompactionChoice {
    /// 一批至多搬几个单元（`compaction_plan::CompactionBatchSize::ForcedForTests`），至少 1。
    pub batch_units: u64,
    /// 强制选哪一段（`CompactionSwitches::forced_region`）：执行时在现行那一版里按段起点升序排的候选段（[`segments_the_compaction_may_be_forced_onto`]）
    /// 里取第 `region_selector mod 个数` 段；一段都没有记「前提不满足」。生成时不知道那一刻盘上有哪几段。
    pub region_selector: u64,
    pub ending: CompactionRunEnding,
}

/// 整理一趟推到哪、之后怎样。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompactionRunEnding {
    /// 显式入口推到停（判据或发布预算 `B_gc`），会话照旧开着；意图没做完的照旧活着（R 排他跟着）。
    RunsUntilItStops,
    /// 至多推 `batches` 批（执行器把这一趟的发布预算压到这么多：式子里「已经过去的发布数」先抬到 3S − 4 − `batches`，盘上没有活意图时
    /// 再减签发那一次，判决 G8 `B_gc`），之后这个进程崩掉（不卸载；已发的写都落了盘），可写挂载一次。意图在 logged_ops 里（判决 G1），
    /// 推到半路崩掉就是批间崩溃：重建出来的那一版带着意图、按映射条目的位置覆盖提示；执行器的可写挂载装 [`compaction_switches_of_the_mounts_of_the_executor`]，
    /// 挂载里不续做，下一趟整理接着推盘上那一条。
    CrashesAfterBatchesThenMountsWritable { batches: u64 },
}

/// 操作的种类：统计与比重用。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HistoryOperationKind {
    PublishFirstFile,
    PublishOverwrite,
    PublishWithoutUnits,
    CloseAndMountWritable,
    RollBackWhileMounted,
    RaiseRollbackFloor,
    ColdStartRecover,
    CrashRecoveryAbandoningTheNewestRoot,
    Unlink,
    CreateInode,
    OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
    WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
    WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros,
    CreateSnapshot,
    ReadSnapshot,
    DestroySnapshot,
    CompactWhileMounted,
    Truncate,
    CreateInodes,
    WriteOfAnInode,
    WriteAtOffset,
}

impl HistoryOperationKind {
    pub const ALL: [HistoryOperationKind; 21] = [
        HistoryOperationKind::PublishFirstFile,
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::PublishWithoutUnits,
        HistoryOperationKind::CloseAndMountWritable,
        HistoryOperationKind::RollBackWhileMounted,
        HistoryOperationKind::RaiseRollbackFloor,
        HistoryOperationKind::ColdStartRecover,
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,
        HistoryOperationKind::Unlink,
        HistoryOperationKind::CreateInode,
        HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros,
        HistoryOperationKind::CreateSnapshot,
        HistoryOperationKind::ReadSnapshot,
        HistoryOperationKind::DestroySnapshot,
        HistoryOperationKind::CompactWhileMounted,
        HistoryOperationKind::Truncate,
        HistoryOperationKind::CreateInodes,
        HistoryOperationKind::WriteOfAnInode,
        HistoryOperationKind::WriteAtOffset,
    ];
}

impl HistoryOperation {
    #[must_use]
    pub fn kind(&self) -> HistoryOperationKind {
        match self {
            HistoryOperation::PublishFirstFile(_) => HistoryOperationKind::PublishFirstFile,
            HistoryOperation::PublishOverwrite(_) => HistoryOperationKind::PublishOverwrite,
            HistoryOperation::PublishWithoutUnits => HistoryOperationKind::PublishWithoutUnits,
            HistoryOperation::CloseAndMountWritable => HistoryOperationKind::CloseAndMountWritable,
            HistoryOperation::RollBackWhileMounted(_) => HistoryOperationKind::RollBackWhileMounted,
            HistoryOperation::RaiseRollbackFloor(_) => HistoryOperationKind::RaiseRollbackFloor,
            HistoryOperation::ColdStartRecover => HistoryOperationKind::ColdStartRecover,
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
            }
            HistoryOperation::Unlink => HistoryOperationKind::Unlink,
            HistoryOperation::CreateInode => HistoryOperationKind::CreateInode,
            HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
                HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
            }
            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            }
            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros => {
                HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros
            }
            HistoryOperation::CreateSnapshot => HistoryOperationKind::CreateSnapshot,
            HistoryOperation::ReadSnapshot(_) => HistoryOperationKind::ReadSnapshot,
            HistoryOperation::DestroySnapshot(_) => HistoryOperationKind::DestroySnapshot,
            HistoryOperation::CompactWhileMounted(_) => HistoryOperationKind::CompactWhileMounted,
            HistoryOperation::Truncate(_) => HistoryOperationKind::Truncate,
            HistoryOperation::CreateInodes(_) => HistoryOperationKind::CreateInodes,
            HistoryOperation::WriteOfAnInode(_) => HistoryOperationKind::WriteOfAnInode,
            HistoryOperation::WriteAtOffset(_) => HistoryOperationKind::WriteAtOffset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SplitMix64 的参考输出（Vigna 的 splitmix64.c，种子 0）：手写的实现逐位对得上，换一个混合常数这里就红。
    #[test]
    fn seeded_random_source_reproduces_the_splitmix64_reference_sequence() {
        let mut source = SeededRandomSource::from_seed(0);
        assert_eq!(source.next_word(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(source.next_word(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(source.next_word(), 0x06c4_5d18_8009_454f);
        let mut other = SeededRandomSource::from_seed(1_234_567);
        assert_eq!(other.next_word(), 0x599e_d017_fb08_fc85);
        assert_eq!(other.next_word(), 0x2c73_f084_5854_0fa5);
    }
}
