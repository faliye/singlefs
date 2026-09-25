//! 可写挂载（里程碑「第二个事务」步 3）：进程重开镜像之后先走一次恢复，从盘上重建可写态——所选根、施加前缀之后的根、
//! 上一版全部角色的单元与指针、分配器、下一条 jsn——再取实例代号、给上一个实例写行（D18（块里携带什么信息） 已定项 11）、
//! 推空发布直到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊），之后本实例的发布才接在后面。
//! 第一版没有干净关闭标记，重开一律走恢复。

use crate::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, JournalSequenceNumber, SlotNumber,
};
use crate::admission::{
    admission_reading_before_a_publish, admit_on_every_device, AdmissionRefusedOnSomeDevices,
    BytesOnOneDevice, DemandOnDevice, SpaceAdmission,
};
use crate::allocation_record_tree::{
    node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
};
use crate::allocator::{
    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PlacementRefusal, PoolAllocator,
    ReclaimedReuse, RootRingOccupancy, RootRingOccupant,
};
use crate::block_device::BlockDevice;
use crate::journal::back_chain_of;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
use crate::recovery::read_unit_via_locations;
use crate::recovery::{
    allocation_records_of_version_without_file, allocation_records_under_root, choose_root,
    choose_system_configuration, effective_rollback_floor, every_root_ring_slot_holds_a_root,
    highest_root_txg, highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
    instance_table_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
    readable_roots_with_ring_slots, rebuild_version, replay_journal, rollback_high_water_of_root,
    rollback_witness_of_the_pool, scan_journal, tree_table_has_no_entries,
    user_visible_tree_root_pointers, verified_system_configuration_slots, InstanceTableChain,
    JournalScanReport, PoolReader, RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
    UserVisibleTreeRootPointers,
};
use crate::rollback_witness::{
    rollback_witness_capacity, RollbackWitnessEntry, RollbackWitnessTable,
};
use crate::root_record::RootRecord;
use crate::root_ring::{target_for_publish, RootRingSlot};
use crate::transaction::{
    acquire_expected_instance, instance_generation_to_acquire, prepare_the_publish_without_units,
    prepare_the_row_publish_on_a_version_without_file, prepare_the_version_publish,
    publish_instance_table_on_version_without_file, publish_version, publish_without_units,
    role_of_allocation_record_tree_node, AcquisitionFailed, ExpectedInstanceAcquisitionFailed,
    InstanceTableOnlyPublishPlan, InstanceTablePlan, InstanceTableRewrite, PoolVersion, PoolWriter,
    PublishError, PublishPlan, PublishedUnit, ReleaseChecksumCheck, TransactionOutput,
    TransactionUnit, VersionWithoutFilePublishOutput, ZeroUnitPublishPlan,
};
use crate::write_accounting::WritesByStructureKind;
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

pub use crate::instance_table::{InstanceRow, InstanceTableRecords};

/// 可写挂载没做成。
#[derive(Debug)]
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 所选根下面有文件版本，而环里一条自证过的记录都没有：从盘上重建上一版要一条记录顶着，拿不出来
    /// （树表 0 条的根用不到记录，刚 mkfs 的池不走这里）。
    FileVersionWithoutAnyJournalRecord,
    /// 所选根（抬 F 时是现行那一版的根）指着的实例表沿链有一片读不出，或解不出行与链指针（D18（块里携带什么信息） 已定项 11「表不可读时」）：
    /// 判不了候选集，在任何写之前拒绝。
    InstanceTableMalformed,
    Acquisition(AcquisitionFailed),
    /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成：错，连同这次挂载的写入口交得出的写账（[`PublishSequenceFailed`]）。
    Publish(PublishSequenceFailed),
    /// 抬 F 那一串空发布（D16（发布语义） 已定项 1：推到每块盘上都有一条带新 F 的根才生效）预演过了、真发时有一次发不出去：错，连同抬 F 自己开的
    /// 写入口交得出的写账（[`PublishSequenceFailed`]，增补 2 收口表第 58 行，与可写挂载那一条同一个形态）。
    /// 落盘之前就报得出的错在预演里已经报了（`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`），走到这里的只剩落盘途中失败，
    /// 与真发时读盘核出对不上、同预演分叉的那一格（`raise_rollback_floor` 里那条注释）。
    /// `writes_of_persisted_publishes` 有几份，前面就有几次已经落盘——带新 F 的根在盘上、调用方的现行版本已经是最后落盘的那一版，
    /// F 还没在每块盘上生效，抬 F 回收的槽照旧扣着；一份都没有、第一次在任何写之前被拒时，分配器与抬 F 之前逐项相同
    /// （扣住的槽放开、回收的回到 defer，C546（抬 F 被拒时扣住的槽不退回））。`cause` 是下一次（份数 + 1，从 1 数）的错。`cause` 自己说的「在任何写之前」
    /// （例如 `PublishError::PlacementRefused`）只对出错的这一次成立，不对整串成立（C516（抬 F 那一串发布被拒时前面几次已落盘））。
    RaiseFloorSequencePublishFailed(PublishSequenceFailed),
    /// 抬 F 那一串 `publishes_in_the_sequence` 次空发布在任何写之前、在分配器的一份拷贝上整串预演（与真发同一段落盘之前的代码，
    /// `rehearse_the_publishes_raising_the_floor`），第 `refused_publish_in_the_sequence` 次（从 1 数）报了 `cause`：这一串一次都不发，
    /// 盘上逐字节不变，分配器与抬 F 之前逐项相同（扣住的槽放开、回收的回到 defer、补的隔离撤掉，C546（抬 F 被拒时扣住的槽不退回））。
    /// 改之前这一串逐次发，第 n 次（n ≥ 2）才被拒时前 n − 1 次已经落盘（`RaiseFloorSequencePublishFailed`，份数 ≥ 1）。
    RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
        refused_publish_in_the_sequence: usize,
        publishes_in_the_sequence: usize,
        cause: PublishError,
    },
    /// 回退的目标根不在回退候选集里，`exclusion` 说是哪一条（管理员要做的决定都是换一条目标；调用方按这个字段分流，不看给人看的文字——
    /// 增补 3 第 2 件代码三方第一轮判决第三节第 2 条：此前只带一句理由文字，胶水分不出是哪一条）。在任何写之前拒绝。
    RollbackTargetNotACandidate {
        target: RollbackTarget,
        exclusion: RollbackCandidateExclusion,
    },
    /// 要抬的 F 超过上限 min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)（D16（发布语义） 已定项 1）。
    RollbackFloorAboveCeiling {
        requested: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
    /// 树表 0 条、而根记录指着的实例表或树表不是 mkfs 写的那一版（指针的诞生 txg 不是 0）：这样一版的分配记录在哪没有条款，
    /// 拒绝挂载。详见 `format_time_allocator` 的文档注释。
    VersionWithoutFileNotWrittenByMakeFilesystem {
        root: RollbackTarget,
        instance_table_birth_txg: CheckpointTxg,
        tree_table_birth_txg: CheckpointTxg,
    },
    /// 树表 0 条、而根记录指着的实例表或树表那条指针的两条位置条目落在不同的槽：这一版的落点就是这两个单元，
    /// 而第一版的布局是两盘同槽（一个单元整个落在一列上、两盘各一份），两条位置条目各指一个槽的版本这个实现写不出来
    /// ⇒ 是一份坏镜像，拒绝挂载。槽号是**盘上读来的 6 字节**、不是我们的不变量，所以这里报错、不断言（panic 面普查 R5）。
    /// 拒的是「这一条指针的两条位置条目不同槽」，与「各盘算出来的落点不同」（`PlacementRefusal` 那两条第一版不支持的池形状）
    /// 不是同一件事：那两条说的是写侧算出来的落点，这一条说的是盘上已经写着的字节。在动分配器之前拒绝，盘上逐字节不变。
    FormatTimeUnitLocationsOnDifferentSlots {
        unit: TransactionUnit,
        disagreement: LocationEntriesOnDifferentSlots,
    },
    /// 算抬 F 的上限时一条有效根（不是被抛弃的、txg ≥ F）的树表读不出或解不开：它算不算非空判不了，读不出要走修复、不跳过也不猜，
    /// 这次抬 F 拒绝（不回收、不发布）。
    RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
        root: RollbackTarget,
        failure: RecoveryFailure,
    },
    /// 取号之前判定时算出的号与取号写之前重算的号不同（两次读系统配置之间有瞬时读错）：判定作废，一个字节都没写。
    InstanceGenerationChangedBeforeAcquisition {
        expected: InstanceGeneration,
        recomputed: InstanceGeneration,
    },
    /// 空间准入不够（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md`
    /// 第四节第 2 条把它接进可写挂载）：扣掉这次挂载的实例切换预留（挂载期承诺量，D28（挂载期承诺量） 已定项 3）与别的八项之后，
    /// 至少一块盘的可用(d) < 0——「实例切换的预留拿得到」这条可写挂载准入合取不成立（D2（RAID 条带策略） 已定项 13：任一不成立即只读挂载；
    /// 这里交回错误，要不要只读挂载由调用方定）。在**取号之前**返回，一个写都没发、盘上逐字节不变、两块盘系统配置里的实例代号不动。
    SpaceAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        refusal: AdmissionRefusedOnSomeDevices,
    },
    /// 写行那次发布在取号之前的预演（发布路径落盘之前那一段：释放核验、这次要换下的那条实例表旧链逐片核、两棵多层码 2 树的形状、
    /// 分配记录树重写集合的固定点）报错：在**取号之前**拒绝，盘上一个字节都不动、
    /// 两块盘系统配置里的实例代号不动（增补 2 第 20a 行，代码三方第一轮打中）。改之前这些只在发布路径里算，
    /// 取号（两次系统配置槽写 + 一道屏障）已经写完才报出来，而取号的回卷只管取号自己那几次写报错、管不到之后的发布失败 ⇒
    /// 那样的池此后每试一次可写挂载就再烧一个实例代号。
    RowPublishAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        cause: PublishError,
    },
    /// 暖机那几次空发布（写行之后推到本实例的根覆盖每块盘，D16（发布语义） 已定项 8 戊）里第几次在取号之前的预演报错：连写行那次一起
    /// 在取号之前预演，报错在任何写之前返回（增补 2 第 20a 行，2026-09-18 用户定案）。分配记录树按位置寻址之后
    /// （D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，暖机这几次还报得出的只剩两棵多层码 2 树长过 256 层、
    /// 分配记录树重写集合迭代不收敛（只在强制复用窗口为 0 的只供测试的开关下可能）这类，落点取不到另报
    /// `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`。
    WarmUpAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        /// 第几次暖机空发布算不过，从 1 数。
        warm_up_publish_index: usize,
        /// 这次挂载要推几次暖机空发布（按根环落点公式现算）。
        warm_up_publishes_planned: usize,
        cause: PublishError,
    },
    /// 取号之前在分配器的一份拷贝上把这次挂载取号之后要发的那一串（写行一次、暖机 `warm_up_publishes_planned` 次）逐次取落点，
    /// 第 `publish_index` 次（从 0 数，0 是写行那次）的 `unit` 取不到，`refusal` 是分配器给的原因
    /// （`dry_run_of_the_publishes_after_acquisition`）。**取号之前的准入怎么把这次挂载要写的落点算进去，条款没定**：
    /// 可写挂载的准入要「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13；D23（journal 的角色与格式） 已定项 14「切换要用的块在挂载准入时预留」），
    /// 预留按 D28（挂载期承诺量） 已定项 3 算，其中暖机那一半的 c_max 与已定项 4 的 checkpoint 保留池从哪读没有条款
    /// （C363（现算保留池时树高从哪读没有条款）），D28（挂载期承诺量） 已定项 1 那条式子另一边的「需求」怎么摊到每块盘也没有
    /// （C370（需求、可用与 df 没有共同单位））。第一版不算那条式子，只把「这一串自己的落点取不到」挪到取号之前：在任何写之前返回，
    /// 盘上逐字节不变、两块盘系统配置里的实例代号不动。改之前取号写完、写行或暖机才被落点拒绝（`Publish`），实例代号一去不回
    /// （里程碑「第二个事务」增补 2 收口表第 39 行那一族：单元区 240 槽的小盘上走得到）。
    PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
        instance_to_acquire: InstanceGeneration,
        publish_index: usize,
        warm_up_publishes_planned: usize,
        unit: TransactionUnit,
        refusal: PlacementRefusal,
    },
    /// 可写挂载准入不成立：`devices` 列出不带所选那一版的每一块盘（可写挂载是施加前缀之后的那一版，回退是 R_old 那一版——
    /// 新实例的第一次发布接在它后面、照抄它的单元），各带一样它缺的（[`SelectedVersionLackingOnDevice`]）。
    /// 依据：D2（RAID 条带策略） 已定项 13 挂载准入「可写设备数 ≥ w 的下限，任一不成立即只读挂载」，已定项 6「`w ≥ 2` 是硬下界」；
    /// D18（块里携带什么信息） 已定项 11「可写挂载的顺序」里「可见」= 独占打开成功且系统配置读得通，「前提」里打开过又掉了、
    /// 缺号期间池里发布过的盘整盘作废、只读到重同步完成。第一版每个单元两块盘各一份，拿这样的盘接着写，照抄过去的单元只剩一份。
    /// 在取号之前返回，一个写都没发、盘上逐字节不变。只读挂载照旧（`mounted_read::mount_read_only`：每个单元按位置条目逐条试，
    /// 读到的是验得过的那一份，也就是带着这一版的那块盘上的）。重同步第一版不做，留给 C120（分叉盘回归的判定与重同步）。
    WritableMountRefusedByDevicesWithoutTheSelectedVersion {
        selected_version: RollbackTarget,
        /// 所选那一版那次发布的末条记录的 jsn（`journal_position_of_the_selected_version`）：判「落后」拿它比。
        selected_version_journal_position: JournalSequenceNumber,
        devices: Vec<DeviceWithoutTheSelectedVersion>,
    },
    /// 可写挂载准入不成立：交进来的盘数 `devices_handed_in` 低于 w 的下限 `stripe_width_lower_bound`（[`STRIPE_WIDTH_LOWER_BOUND`]）。
    /// 依据：D2（RAID 条带策略） 已定项 13 挂载准入的第一条合取「可写设备数 ≥ w 的下限，任一不成立即只读挂载」，
    /// 已定项 6「`w ≥ 2` 是硬下界（零冗余的条带不许发出）」，已定项 9 第一版跑 2 块盘；D18（块里携带什么信息） 已定项 11
    /// 「可写挂载的顺序」先判这一条。可写设备数取交进来的盘数：准入在这次挂载的第一个写之前判，已定项 13 运行期那一行的探针写不在这里做。
    /// 在选系统配置、恢复、重建上一版、读分配记录树与任何写之前返回：一块盘都没读，盘上逐字节不变。
    /// 要不要只读挂载由调用方定（同 `SpaceAdmissionRefusedBeforeAcquisition`）。
    WritableDeviceCountBelowTheStripeWidthLowerBound {
        devices_handed_in: DeviceCount,
        stripe_width_lower_bound: DeviceCount,
    },
}

/// 盘的块数：可写挂载准入拿交进来的盘数与 w 的下限比（`MountError::WritableDeviceCountBelowTheStripeWidthLowerBound`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceCount(pub usize);

/// w 的下限：一条条带至少落几块盘（D2（RAID 条带策略） 已定项 6「`w ≥ 2` 是硬下界（零冗余的条带不许发出）」）。
pub const STRIPE_WIDTH_LOWER_BOUND: DeviceCount = DeviceCount(2);

/// 一块不带所选那一版的盘，与它缺的是什么（`MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceWithoutTheSelectedVersion {
    pub device: DeviceIdentity,
    pub lacking: SelectedVersionLackingOnDevice,
}

/// 一块盘不带所选那一版，缺在哪。按次序判：先看系统配置，再看单元。
///
/// 系统配置不落后、只是缺所选那一版几个单元的盘（单份坏）**不在这里**：它自证过的系统配置跟得上这一版，不是空盘、
/// 也没有停在旧状态，条款没把它归进作废的那两类；它坏的那几份，读的时候按位置条目改读另一份。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedVersionLackingOnDevice {
    /// 两个系统配置槽里一份自证过的（整槽校验和过、fsid 与本池相同）都没有：空盘、别的池的盘、两槽都读不出。
    /// 这块盘不「可见」（D18（块里携带什么信息） 已定项 11「可写挂载的顺序」）。
    NoSelfVerifiedSystemConfiguration,
    /// 停在旧状态：这块盘自证过的系统配置里世代号最大的那一份，(实例代号, journal tail) 落后于所选那一版那次发布的末条记录的 jsn
    /// （每块盘在每次发布之后轮换的系统配置写的就是这一对，`transaction::CommitStep::RotateSystemConfigurationSlots`），
    /// 而且所选那一版有单元在这块盘上它的落点读不出、或者字节与验得过的那一份不同（`units_missing`，按角色与落点列出，
    /// 次序同所选那一版的单元清单）。只落后、单元都在的不算：发布的根落盘之后、系统配置轮换之前崩了，
    /// 或取号失败回卷时写回了 tail 0（`transaction::acquire_instance`），盘都是这个样子。
    BehindTheSelectedVersionAndMissingItsUnits {
        newest_system_configuration_journal_tail: JournalSequenceNumber,
        units_missing: Vec<(TransactionUnit, SlotNumber)>,
    },
}

/// 自己开写入口、接连推的一串发布里有一次没做成：可写挂载（或回退）取号之后那一串——写行一次、暖机推到本实例的根覆盖每块盘
/// （D16（发布语义） 已定项 8 戊）；抬 F 那一串空发布——推到每块盘上都有一条带新 F 的根（D16（发布语义） 已定项 1）。
/// 写入口是这一串自己开的、随错一起丢掉，所以它交得出的写账都在这里：已经落盘的那几次发布各自的写，
/// 与失败那一次落盘阶段已记的写（增补 2 收口表第 58 行：不交出来，设备一层数到的写与程序交得出的账对不上）。
/// 不是发布的写不在其内：可写挂载取号那两次系统配置槽写（取号的回卷另由 `AcquisitionFailed` 报）。
#[derive(Debug)]
pub struct PublishSequenceFailed {
    pub cause: PublishError,
    /// 失败之前这一串里已经落盘的发布各自的写，按先后（可写挂载写行那次在最前）；第一次就失败时是空的。
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
    /// 这一串的写入口的失败账（`PoolWriter::writes_of_failed_publishes` 原样）：落盘阶段失败的那一次一份；
    /// 落盘之前就被拒的（准入、释放判定、落点）一个写都没发，不记，这里是空的。
    pub writes_of_failed_publishes: Vec<WritesByStructureKind>,
}

/// 回退目标不在回退候选集里的是哪一条。`mount_rollback` 按成员的次序判，一条目标同时中几条时只报最先判到的那一条。
/// 候选集只有这三条（D16（发布语义） 已定项 1 / D23（journal 的角色与格式） 已定项 14）：目标那一版树表 0 条**不是**排除项，
/// 回退到它照常做（C493（回退候选集条文与实现说反话） 由此还清）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RollbackCandidateExclusion {
    /// 根环里没有那个 (实例, txg) 的可读根槽：候选集是根环里的根（D23（journal 的角色与格式） 已定项 14）。
    NotInRing,
    /// txg 低于生效的回退下界 F（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效）。
    BelowEffectiveFloor,
    /// 最新根指着的实例表里有那个实例的行 (i, Ti, Wi) 且目标的 txg > Ti，或者它被回退见证表抛弃：被抛弃时间线上的根
    /// （D23（journal 的角色与格式） 已定项 14「按实例表判仍然有效」与「回退见证」）。
    OnAbandonedTimeline,
}

/// 回退的目标：管理员带外从回退候选集里选的那条根（D23（journal 的角色与格式） 已定项 14 的显式例外）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackTarget {
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
}

/// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：关掉影子账，回退之后只被被抛弃根引用的槽照常可分配——
/// C314（回退可以复用被抛弃的根引用的单元） 那两格必红靠它强制进入；产品路径恒 `On`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowLedger {
    On,
    Off,
}

impl ShadowLedger {
    /// 这次挂载走的是哪一臂，运行时报得出（`.claude/rules/fs-design.md` 五条硬要求第 4 条：
    /// **分支必须可观测**）。⚠️ 光看 `MountOutput::isolated_slots_per_device` 分不开两臂：
    /// `Off` 恒 0，而 `On` 在「没有被抛弃根、或它们引用的槽都还被候选集引用着」时也是 0。
    #[must_use]
    pub const fn branch_name(self) -> &'static str {
        match self {
            ShadowLedger::On => "shadow_ledger=on",
            ShadowLedger::Off => "shadow_ledger=off",
        }
    }
}

impl From<RecoveryFailure> for MountError {
    fn from(failure: RecoveryFailure) -> Self {
        MountError::Recovery(failure)
    }
}

/// 一次可写挂载写出的东西。
#[derive(Debug)]
pub struct MountOutput {
    pub instance: InstanceGeneration,
    /// 恢复择到的根（施加前缀之前）。
    pub chosen_root: RootRecord,
    /// 施加前缀之后的根：写行与照抄都以它为准。
    pub effective_root: RootRecord,
    pub journal: JournalScanReport,
    /// 这次挂载写进实例表的行（上一个实例那一行，中间实例各一行）。
    pub rows_written: Vec<InstanceRow>,
    /// 写行那次发布（本实例的第一次发布）；所选根的树表 0 条、要写的行为空（上一个实例是 0）时是一次零单元发布。
    pub row_publish: PoolVersion,
    /// 之后的暖机空发布，直到本实例的根覆盖每块盘。
    pub warm_up_publishes: Vec<PoolVersion>,
    /// 这次挂载走的影子账分支名（`ShadowLedger::branch_name`）：两臂在这里永远不同，
    /// 而 `isolated_slots_per_device` 两臂都可能是 0，光看它分不开（五条硬要求第 4 条）。
    pub shadow_ledger_branch: &'static str,
    /// 影子账隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」，只住内存）：只被被抛弃根引用的槽，
    /// 仍被候选集里的根引用的不在其内（对用户 2026-09-16 定的窄读法措辞的收严，见 `isolate_slots_referenced_only_by_abandoned_roots`；预想）。
    pub isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
    /// 被抛弃根里树表或分配记录树读不出、解不开的条数：这样的根影子账罩不到，只计数、不拒绝挂载
    /// （步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
    pub abandoned_roots_unreadable: u64,
    /// 这次挂载从写行那次发布的系统配置轮换起写的回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：
    /// 盘上原有的先按所选根实例表里的回退行补回缺的条目，再按删除规则删过（`rollback_witness_tables_of_this_mount`），
    /// 回退时再加这一次的那一条。取号那两次写带的是删过、还没加的那一张。
    pub rollback_witness_written: RollbackWitnessTable,
}

/// 可写挂载的结果：写出的东西、重建并推进过的分配器、接下来的发布要接在后面的那一版。
#[derive(Debug)]
pub struct Mounted {
    pub output: MountOutput,
    pub allocator: PoolAllocator,
    pub current: PoolVersion,
}

fn map_rebuild_failure(failure: RebuildVersionFailure) -> MountError {
    match failure {
        RebuildVersionFailure::Walk(recovery_failure) => MountError::Recovery(recovery_failure),
        RebuildVersionFailure::NoRecordStandingForFileVersion => {
            MountError::FileVersionWithoutAnyJournalRecord
        }
    }
}

/// 恢复（或回退）之后从盘上重建出来的上一版，带文件的连同写行要接在后面的那一版实例表。
#[allow(
    clippy::large_enum_variant,
    reason = "一次挂载只有一个，两个成员差几百字节按值搬无所谓；装箱只多一层解引用"
)]
enum PreviousVersion {
    /// 树表 0 条：这一版只有根自己指着的实例表与树表两个单元。写行要在这一版那张实例表后面接行，所以连表一起解出来。
    WithoutFile {
        root: RootRecord,
        table: InstanceTableChain,
    },
    WithFile {
        output: TransactionOutput,
        table: InstanceTableChain,
    },
}

impl PreviousVersion {
    /// 这一版的实例表链：全部行（写行时要在它后面接上这次的行，取号之前的准入也按它的行数算）与每一片的指针
    /// （写行那次发布整条链重写，这几片逐片释放）。两臂都有表：树表 0 条那一版的表是根记录直接指着的那条链
    /// （mkfs 种下的，或上一次挂载写行重写的）。
    fn instance_table_chain(&self) -> &InstanceTableChain {
        match self {
            PreviousVersion::WithoutFile { table, .. }
            | PreviousVersion::WithFile { table, .. } => table,
        }
    }

    /// 带文件的那一版；树表 0 条的是 `None`（空间准入的 ckpt_cost 按它算，`admission::checkpoint_cost_of_the_version_to_build_on`）。
    fn file_version(&self) -> Option<&TransactionOutput> {
        match self {
            PreviousVersion::WithoutFile { .. } => None,
            PreviousVersion::WithFile { output, .. } => Some(output),
        }
    }

    /// 这一版的单元（角色、落点、验得过的那一份字节）：可写挂载在取号之前逐盘核每块盘带不带它们
    /// （`devices_without_the_selected_version`）。落点取第一条位置条目的槽——第一版两盘同槽（D2（RAID 条带策略） 已定项 10），
    /// 发布路径照抄一个单元时也按这一个落点对待它。
    ///
    /// 带文件的一版就是重建出来的 `units`（全部角色；提示与映射都读不出的数据单元字节是空的，逐盘核时跳过它）。
    /// 树表 0 条的一版是根记录直接指着的几个单元：实例表第 0 片、树表、写过行的一版那棵分配记录树的每个节点
    /// （与 `placements_referenced_by_root` 树表 0 条那一臂同一份清单，字节按位置条目读验得过的那一份）。
    /// 两臂都不列实例表链第 1 片起的各片：每次写行整条链 COW 重写（D18（块里携带什么信息） 已定项 11），后面各片与第 0 片同一次发布写出，
    /// 停在那次发布之前的盘缺它们，就也缺第 0 片。
    ///
    /// # Errors
    /// 树表 0 条那一版的实例表、树表或分配记录树这一次读不出、解不开（重建时读得出，两次读之间的瞬时读错）⇒ `Recovery`：
    /// 核不了就不放行，在任何写之前返回。
    #[allow(
        clippy::ptr_arg,
        reason = "allocation_records_of_version_without_file 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
    )]
    fn units<Device: BlockDevice>(
        &self,
        devices: &Vec<(DeviceIdentity, Device)>,
    ) -> Result<Cow<'_, [PublishedUnit]>, MountError> {
        match self {
            PreviousVersion::WithFile { output, .. } => Ok(Cow::Borrowed(&output.units)),
            PreviousVersion::WithoutFile { root, .. } => {
                let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
                let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
                let mut units = vec![
                    PublishedUnit {
                        slot: root.instance_table.locations[0].slot,
                        identity: TransactionUnit::InstanceTable,
                        bytes: read_unit_via_locations(
                            devices,
                            &root.instance_table.locations,
                            data_unit_bytes,
                        )?,
                    },
                    PublishedUnit {
                        slot: root.tree_table.locations[0].slot,
                        identity: TransactionUnit::TreeTable,
                        bytes: read_unit_via_locations(
                            devices,
                            &root.tree_table.locations,
                            node_bytes,
                        )?,
                    },
                ];
                if let Some(tree) = allocation_records_of_version_without_file(devices, root)? {
                    for (node, pointer) in &tree.version.nodes {
                        units.push(PublishedUnit {
                            slot: pointer.locations[0].slot,
                            identity: role_of_allocation_record_tree_node(*node),
                            bytes: read_unit_via_locations(
                                devices,
                                &pointer.locations,
                                node_bytes,
                            )?,
                        });
                    }
                }
                Ok(Cow::Owned(units))
            }
        }
    }
}

/// 写行那次发布的实例表链怎么重写：这一版那张表的行后面接上这次写的行，被换下的是这一版的整条链
/// （D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）。取号之前的准入与写行那次发布读的都是它，两处同一份输入。
fn instance_table_rewrite_of_the_row_publish(
    table: &InstanceTableChain,
    rows_written: &[InstanceRow],
) -> InstanceTableRewrite {
    let mut rows = table.records.rows.clone();
    rows.extend_from_slice(rows_written);
    InstanceTableRewrite {
        rows,
        replaced_chain: table.page_pointers.clone(),
    }
}

/// 重建上一版：树表 0 条就留根加它指着的那张实例表；带文件的连同重建出来的单元。两臂的实例表都从根记录里那条指针沿链读
/// （D18（块里携带什么信息） 已定项 11）——只解 `TransactionUnit::InstanceTable` 那一个单元的字节只有第 0 片，
/// 多于一片的表会被按前半张判准入、接行。
#[allow(
    clippy::ptr_arg,
    reason = "rebuild_version 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn rebuild_previous_version<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    root: &RootRecord,
    record_standing_for_root: Option<crate::journal::JournalRecord>,
) -> Result<PreviousVersion, MountError> {
    let rebuilt =
        rebuild_version(devices, root, record_standing_for_root).map_err(map_rebuild_failure)?;
    let table = instance_table_chain_of_root(devices, root)
        .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?;
    match rebuilt {
        RebuiltVersion::WithoutFile => Ok(PreviousVersion::WithoutFile { root: *root, table }),
        RebuiltVersion::WithFile(output) => Ok(PreviousVersion::WithFile { output, table }),
    }
}

/// 新实例写行那一行的来历：上一个实例（普通挂载）或被退回的实例（回退）那一行；中间实例的行由 `establish_instance` 补 (i, 0, 0)。
struct PreviousInstanceRow {
    instance: InstanceGeneration,
    selected_root_txg: CheckpointTxg,
    applied_transaction_high_water: u64,
    is_rollback: bool,
}

/// 恢复（或回退）之后建立新实例要带的东西。
struct InstanceStart {
    chosen_root: RootRecord,
    effective_root: RootRecord,
    journal: JournalScanReport,
    previous: PreviousVersion,
    previous_row: PreviousInstanceRow,
    first_txg: CheckpointTxg,
    next_counter: u64,
    /// 本实例第一次发布（写行那一次）要带的树 ID 水位：根环里全部自证过的根与环里全部自证通过的记录各自带的水位取 max
    /// （D8（核心索引结构） 已定项 8 ②；`recovery::highest_tree_identifier_watermark_in_the_ring`）。回退时它可以高于
    /// 回退到的那一版自己带的——被抛弃时间线上的根仍承载它们那一代发过的号，回退之后再发第一个文件版本就从这里往上发。
    tree_identifier_watermark_of_the_ring: u64,
    /// 新实例的根带的回退下界 = 恢复后生效的 F（各幸存盘所带 F 最大值的最小值）：与重建分配器时回收用的同一个值，
    /// 一条根带的 F 与它的记账行才说同一件事。
    effective_floor: CheckpointTxg,
    abandoned_roots_unreadable: u64,
    /// 这次挂载走的影子账那一臂：产品路径恒 `On`，回退那条路由调用方给。
    /// 带着它只为一件事——让 `MountOutput::shadow_ledger_branch` 报得出走了哪一条（五条硬要求第 4 条）。
    shadow_ledger: ShadowLedger,
    /// 恢复（或回退）择到的系统配置：回退见证表的条数上限（R × S − 1）与删除规则读的根环几何从它取。
    system_configuration: crate::system_configuration::SystemConfiguration,
    /// 所选根（`chosen_root`）指着的那张实例表的全部行：删除规则之前按其中的回退行补回见证表缺的条目
    /// （`rollback_witness_entries_recovered_from_the_instance_table`）。可写挂载取施加前缀之后那一版的表——施加记录照抄
    /// 所选根的实例表指针（`recovery::replay_journal`），两者是同一张；回退取最新根那一张（候选集按它判的那一张）。
    /// 两处都是这次挂载已经读出来的，不为它另读一次盘。
    rows_of_the_chosen_roots_instance_table: Vec<InstanceRow>,
    /// 判不判空间准入（只供测试的开关，`admission::SpaceAdmission`）：产品路径恒判。
    space_admission: SpaceAdmission,
    /// 所选那一版（`effective_root` 那一版）那次发布的末条记录的 jsn（`journal_position_of_the_selected_version`）：
    /// 取号之前逐盘核「这块盘落没落后于这一版」拿它比。
    selected_version_journal_position: JournalSequenceNumber,
}

/// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。
fn first_txg_of_new_instance<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
) -> CheckpointTxg {
    let highest_record_txg = records
        .values()
        .map(|record| record.checkpoint_txg)
        .max()
        .unwrap_or(CheckpointTxg(0));
    let highest_ring_txg = highest_root_txg(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .unwrap_or(CheckpointTxg(0));
    CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)
}

/// 本实例第一次发布要带的树 ID 水位（D8（核心索引结构） 已定项 8 ②「根环里全部根记录该字段的 max」）：
/// 根环里全部自证过的根与 `records`（环里全部自证通过的记录）各自带的水位取 max，再与这次要接在后面的那一版自己带的取 max——
/// 那一版是从环里读出来的根或从环里的记录施加出来的，按构造已经在内；再取一次是为了两次读环之间的瞬时读错
/// （择根那一遍读到了它、这一遍没读到）不把水位拉低。读不出的根怎么算见 `highest_tree_identifier_watermark_in_the_ring`。
fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    version_to_build_on: &RootRecord,
) -> u64 {
    highest_tree_identifier_watermark_in_the_ring(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        records,
    )
    .map_or(version_to_build_on.tree_identifier_watermark, |ring| {
        ring.max(version_to_build_on.tree_identifier_watermark)
    })
}

/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
/// 环里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由环里最旧的有效根接管。
#[must_use]
pub fn reclaim_floor(
    effective_floor: CheckpointTxg,
    oldest_valid_root: Option<CheckpointTxg>,
) -> CheckpointTxg {
    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}

/// 一条根按最新根指着的实例表判是不是被抛弃的：有它那个实例的行 (i, Ti, Wi) 且 txg > Ti（D23（journal 的角色与格式） 已定项 14
/// 回退段的候选集规则反过来）。
fn abandoned_by_table(root: &RootRecord, table: &InstanceTableRecords) -> bool {
    table
        .rows
        .iter()
        .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
}

/// 重建分配器之后要带回去的东西。
struct RebuiltAllocator {
    allocator: PoolAllocator,
    effective_floor: CheckpointTxg,
    abandoned_roots_unreadable: u64,
}

/// 影子账算完交回的：读不出账的被抛弃根条数，与每条读得出账的被抛弃根引用的全部落点（按根的 (实例, txg) 认；
/// 分配器那张根环表拿它判「这条根离开根环时清哪几个槽」「挂载内回收出来的槽还有没有被抛弃根引用」，`allocator::RootRingOccupant`）。
struct ShadowLedgerComputed {
    abandoned_roots_unreadable: u64,
    placements_referenced_by_abandoned_root:
        BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<PlacementOnDevice>>,
}

/// 影子账（D28（挂载期承诺量） 已定项 1 第九项）：只被被抛弃根引用的槽 = 被抛弃根引用的槽 − 候选集里的根引用的槽 − 当前这一版账里
/// 还分配着的槽；候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F。用户 2026-09-16 定的窄读法措辞是「仍被有效根引用的槽不在其内」、有效只按实例表判，
/// 豁免只给候选集里的根（多了 txg ≥ F、抬 F 之后按新 F 重算）是对那句措辞的收严（alloc-basis 那一轮的 G5 臂）：按原措辞，抬 F 到 11 之后
/// A 仍豁免 mkfs 实例表那 2 槽，它们被回收、发出去，而被抛弃根 B 还引用着，违反 D23（journal 的角色与格式） 已定项 14 的主句。
/// 预想、偏离用户定案的措辞，交用户。
/// 一条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
/// 树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载；候选根读不出就当它什么都不豁免（隔离只会多不会少）。
/// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；
/// 清只在被抛弃的根离开根环的那一次发布里清（`PoolAllocator::record_root_written_by_this_process`，按交回的每条根引用的落点）。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    roots: &[RootRecord],
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
    floor: CheckpointTxg,
    current_records: &[AllocationRecord],
) -> ShadowLedgerComputed {
    let mut referenced_by_candidates: BTreeSet<(u32, u64)> = current_records
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device.0, record.slot.0))
        .collect();
    for root in roots
        .iter()
        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
    {
        if let Some(placements) = placements_referenced_by_root(devices, root) {
            referenced_by_candidates.extend(
                placements
                    .iter()
                    .map(|(device, slot, _)| (device.0, slot.0)),
            );
        }
    }
    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut unreadable = 0;
    let mut placements_referenced_by_abandoned_root = BTreeMap::new();
    for root in roots.iter().filter(|root| is_abandoned(root)) {
        let Some(placements) = placements_referenced_by_root(devices, root) else {
            unreadable += 1;
            continue;
        };
        for (device, slot, span_slots) in placements.iter().copied() {
            let key = (device.0, slot.0);
            if referenced_by_candidates.contains(&key) || !isolated.insert(key) {
                continue;
            }
            allocator.isolate_abandoned(device, slot, span_slots);
        }
        placements_referenced_by_abandoned_root.insert(
            (root.instance, root.checkpoint_txg),
            placements
                .into_iter()
                .map(|(device, slot, span)| PlacementOnDevice { device, slot, span })
                .collect(),
        );
    }
    ShadowLedgerComputed {
        abandoned_roots_unreadable: unreadable,
        placements_referenced_by_abandoned_root,
    }
}

/// 一条根引用着的落点，逐盘：带文件的一版从它那一版的分配记录里取（账里已释放的是它换下的上一版的，不算它引用）；
/// **树表 0 条的一版**引用的就是根记录直接指着的那几个单元——实例表（连同链上后面各片）、树表，写过行的一版还有它自己那片
/// 分配记录树节点（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））——两盘同槽，不读分配记录树：
/// 那一片读不出时它自己的落点照样认得出。读不出、解不开交回 `None`（调用方只计数，不拒绝挂载）。
///
/// 树表 0 条那一臂不能省：写过行的树表 0 条根指着的是一片新实例表，它既不在任何分配记录树里、也不在 mkfs 那两个单元里，
/// 少了这一臂它在影子账里就成了「账读不出」的被抛弃根，那片实例表不被隔离、也不在重建出来的账里，
/// 回退之后当空闲槽发出去，把根环里还引用着它的那条根读坏（2026-09-23 崩溃注入打中：`CloseAndMountRollback` 里
/// I-2.1 / I-4.8 / I-7.4 三条红在「最新根引用的单元已被复用」）。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn placements_referenced_by_root<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    root: &RootRecord,
) -> Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>> {
    match tree_table_has_no_entries(devices, root) {
        Ok(true) => {
            let mut placements = Vec::new();
            // 实例表是一条链（D18（块里携带什么信息） 已定项 11）：认得出的每一片都是这条根引用的，只有一片时就是根记录里那一条。
            // 读不全的链照样把认得出的那几片算上——少算一片，被抛弃根的那一片就不隔离、回退之后可能被发出去。
            let instance_table_pages =
                instance_table_page_pointers_as_far_as_readable(devices, root);
            // 写过行的一版那棵分配记录树的节点同样是这条根引用的单元（D23（journal 的角色与格式） 已定项 14：被抛弃时间线的根离开根环之前，
            // 它们引用的单元不许重新分配）；漏了它们，被抛弃根的这几片既不隔离、也不在回退目标那一版的账里，回退之后是空闲槽。
            // 树可以多层（D8（核心索引结构） 已定项 14）：读得出多少认多少，读不出的节点它自己那一片照样认（父条目里有它的指针）。
            // mkfs 的第 0 代与照抄它的暖机根那一项全零，没有这棵树。
            let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
            let allocation_record_tree_nodes: Vec<crate::pointer::NodePointer> =
                if root.allocation_record_tree_root == crate::pointer::NodePointer::empty_root() {
                    Vec::new()
                } else {
                    node_pointers_as_far_as_readable(
                        &root.allocation_record_tree_root,
                        &AllocationRecordTreeGeometry::of_reader(devices),
                        &mut |pointer: &crate::pointer::NodePointer| {
                            read_unit_via_locations(devices, &pointer.locations, node_bytes).ok()
                        },
                    )
                };
            for (pointer, unit) in instance_table_pages
                .iter()
                .map(|page_pointer| (page_pointer, TransactionUnit::InstanceTable))
                .chain([(&root.tree_table, TransactionUnit::TreeTable)])
                .chain(
                    allocation_record_tree_nodes
                        .iter()
                        .map(|pointer| (pointer, TransactionUnit::AllocationTree)),
                )
            {
                let slot = slot_shared_by_both_location_entries(&pointer.locations).ok()?;
                for (identity, _) in devices {
                    placements.push((*identity, slot, unit.span_slots()));
                }
            }
            Some(placements)
        }
        Ok(false) => Some(
            allocation_records_under_root(devices, root)
                .ok()?
                .into_iter()
                .filter(|record| !record.is_released)
                .map(|record| (record.device, record.slot, u64::from(record.span_slots)))
                .collect(),
        ),
        Err(_failure) => None,
    }
}

/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收，再把只被被抛弃根引用的槽隔离（影子账；
/// `extra_abandoned` 是这次挂载新抛弃的根——回退时是 (txg, 实例) 大于 R_old 的那些，普通挂载没有）。
/// 影子账只住内存，所以每次挂载都要重算，不只回退那一次（步 4 / 步 5 代码三方第一轮云端攻方腿打中：回退之后普通重开一次隔离就归零）。
#[allow(
    clippy::ptr_arg,
    reason = "choose_root 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn rebuilt_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    previous: &PreviousVersion,
    first_txg: CheckpointTxg,
    extra_abandoned: &dyn Fn(&RootRecord) -> bool,
    shadow_ledger: ShadowLedger,
) -> Result<RebuiltAllocator, MountError> {
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
        .collect();
    let mut allocator = match previous {
        PreviousVersion::WithFile { output, .. } => {
            PoolAllocator::rebuild_from_records(device_maps, output.allocation_records.clone())
        }
        PreviousVersion::WithoutFile { root, .. } => {
            allocator_of_version_without_file(devices, device_maps, root)?
        }
    };
    let current_records = allocator.records().to_vec();
    let effective_floor = effective_rollback_floor(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    // 影子账与分配器那张根环表读的是同一遍根环：两遍之间的瞬时读错会让两边看见的根不一样。
    let roots_with_ring_slots = readable_roots_with_ring_slots(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let roots: Vec<RootRecord> = roots_with_ring_slots
        .iter()
        .map(|(_, root)| *root)
        .collect();
    let newest_table = choose_root(devices, system_configuration)
        .and_then(|newest| instance_table_of_root(devices, &newest));
    // 被回退见证表抛弃的根同样是被抛弃时间线上的根（D23（journal 的角色与格式） 已定项 14「回退见证」）：最新根落回 R_old 时
    // （回退实例的根都读不出）它那一版的实例表里没有回退行，只有见证表认得出它们。
    let rollback_witness = rollback_witness_of_the_pool(devices, system_configuration);
    let is_abandoned = |root: &RootRecord| {
        extra_abandoned(root)
            || rollback_witness.abandons(root.instance, root.checkpoint_txg)
            || newest_table
                .as_ref()
                .is_some_and(|table| abandoned_by_table(root, table))
    };
    let oldest_valid_root = roots
        .iter()
        .filter(|root| !is_abandoned(root))
        .map(|root| root.checkpoint_txg)
        .min();
    allocator.reclaim_released_up_to(
        reclaim_floor(effective_floor, oldest_valid_root),
        ReclaimedReuse::Immediately,
    );
    let shadow_ledger_computed = match shadow_ledger {
        ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            &mut allocator,
            &roots,
            &is_abandoned,
            effective_floor,
            &current_records,
        ),
        ShadowLedger::Off => ShadowLedgerComputed {
            abandoned_roots_unreadable: 0,
            placements_referenced_by_abandoned_root: BTreeMap::new(),
        },
    };
    let ShadowLedgerComputed {
        abandoned_roots_unreadable,
        placements_referenced_by_abandoned_root,
    } = shadow_ledger_computed;
    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = roots_with_ring_slots
        .iter()
        .map(|(ring_slot, root)| {
            let occupant = if is_abandoned(root) {
                RootRingOccupant::AbandonedRoot {
                    referenced_placements: placements_referenced_by_abandoned_root
                        .get(&(root.instance, root.checkpoint_txg))
                        .cloned()
                        .unwrap_or_default(),
                }
            } else {
                RootRingOccupant::ValidRoot {
                    checkpoint_txg: root.checkpoint_txg,
                }
            };
            (*ring_slot, occupant)
        })
        .collect();
    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
        system_configuration
            .immutable
            .sizes
            .root_ring_slots_per_region,
        occupants,
        effective_floor,
        CheckpointTxg(
            first_txg
                .0
                .checked_sub(1)
                .expect("新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1"),
        ),
    ));
    Ok(RebuiltAllocator {
        allocator,
        effective_floor,
        abandoned_roots_unreadable,
    })
}

/// 树表 0 条的那一版的账从哪来，只有两条路：
///
/// - 根记录那一项（分配记录树根指针）不是全零 ⇒ 这一版写过行，写行那次发布把这一版的全部分配记录写进了那棵分配记录树
///   （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案；按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）：
///   整棵读回来重建（挂载时分配记录树整棵读，用户 K4），被换下的实例表因此带着已释放标志回来，不会被当空闲槽发出去。
///   那棵树的节点与记录记进分配器——下一次发布要照抄或换下它们，而这一版没有上一版的内存态可查。
/// - 全零 ⇒ mkfs 的第 0 代（或照抄它的暖机根）：账由实例表与树表两条指针直接算（`format_time_allocator`）。
///
/// # Errors
/// 分配记录树读不出、解不开、位置对不上、条目宽或结构值判红 ⇒ `Recovery(...)`；某个节点两条位置条目不同槽 ⇒
/// `FormatTimeUnitLocationsOnDifferentSlots`；全零那一条上两条指针任一条不是 mkfs 写的那一版 ⇒
/// `VersionWithoutFileNotWrittenByMakeFilesystem`（见 `format_time_allocator`）。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_of_version_without_file 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn allocator_of_version_without_file<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    let Some(tree) =
        allocation_records_of_version_without_file(devices, root).map_err(MountError::Recovery)?
    else {
        return format_time_allocator(device_maps, root);
    };
    // 每个节点两条位置条目同槽（第一版两盘同槽）：下一次发布照抄或换下它们时按一个落点释放。
    for (node, pointer) in &tree.version.nodes {
        slot_shared_by_both_location_entries(&pointer.locations).map_err(|disagreement| {
            MountError::FormatTimeUnitLocationsOnDifferentSlots {
                unit: role_of_allocation_record_tree_node(*node),
                disagreement,
            }
        })?;
    }
    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, tree.records.clone());
    allocator.note_allocation_record_tree_of_the_version_without_file(tree);
    Ok(allocator)
}

/// mkfs 写出的那一版（或照抄它的暖机根）的账：盘上没有分配记录树，这一版的全部落点就是根记录直接指着的实例表与树表两个单元
/// ——mkfs 写在单元区里的那两个（字节表五里它们两盘各一条、分配代 0）。与 mkfs 同一个进程里第一个事务用的分配器同一个起点
/// （`PoolAllocator::mark_format_time_units`），第一个文件版本换下 mkfs 那片树表时照样把它释放。
///
/// # Errors
/// 根记录指着的实例表或树表不是 mkfs 写的那一版（诞生 txg 不是 0）⇒ `VersionWithoutFileNotWrittenByMakeFilesystem`。
///
/// ⚠️ **这一道今天的依据（2026-09-23 随 C512（树表 0 条的一版上被换下的单元记在哪） 定案重判）**：
/// 它**不再**拦任何一条正当历史。此前拦得住的那一条是「建池 → 可写挂载 → 不写文件 → 退出 → 再可写挂载（写行）→ 退出 → 第三次可写挂载」：
/// 写行那次发布换下上一版那片实例表，而那一版没有地方记这条释放，重开之后那一片就成了空闲槽。
/// 定案之后写行那次发布**建起这一版自己的分配记录树**、根指针住根记录，那条历史走 `allocator_of_version_without_file` 的第一条路、
/// 根本到不了这里（`the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool` 钉住）。
/// 走得到这里的只剩「树表或实例表不是 mkfs 写的那一版、而根记录里的分配记录树根指针又是全零」——**实现一处都写不出这种根**
/// （写行那条路径写指针，带文件的那一版的树表不是 mkfs 那一片时早就不走这条分支），所以它今天只对坏镜像与外来镜像说话。
/// 用例要造它得手抹那一项（`plant_a_root_without_its_allocation_record_tree`）。
/// 这一格仍在动分配器、动盘之前拒绝，盘上逐字节不变。
///
/// 两个指针里任一条的两条位置条目槽号不等 ⇒ `FormatTimeUnitLocationsOnDifferentSlots`。两样都在动分配器之前返回。
fn format_time_allocator(
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    if root.instance_table.head.birth_txg != CheckpointTxg(0)
        || root.tree_table.head.birth_txg != CheckpointTxg(0)
    {
        return Err(MountError::VersionWithoutFileNotWrittenByMakeFilesystem {
            root: RollbackTarget {
                instance: root.instance,
                checkpoint_txg: root.checkpoint_txg,
            },
            instance_table_birth_txg: root.instance_table.head.birth_txg,
            tree_table_birth_txg: root.tree_table.head.birth_txg,
        });
    }
    let placement_of = |pointer: &crate::pointer::NodePointer, identity: TransactionUnit| {
        let slot =
            slot_shared_by_both_location_entries(&pointer.locations).map_err(|disagreement| {
                MountError::FormatTimeUnitLocationsOnDifferentSlots {
                    unit: identity,
                    disagreement,
                }
            })?;
        Ok::<Placement, MountError>(Placement {
            slot,
            span: identity.span_slots(),
        })
    };
    let instance_table_placement =
        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;
    let tree_table_placement = placement_of(&root.tree_table, TransactionUnit::TreeTable)?;
    let mut allocator = PoolAllocator::new(device_maps);
    allocator.mark_format_time_units(instance_table_placement, tree_table_placement);
    Ok(allocator)
}

/// 一条有效根的用户可见状态有没有变（D16（发布语义） 已定项 1「非空」从盘上怎么认，2026-09-17 用户定案）：它树表里 inode 树与 extent 树的
/// 根指针，与它前一条有效根树表里的，逐字节比；两棵树任一棵不同就算非空。最旧的有效根没有前一条，拿 `UserVisibleTreeRootPointers::ABSENT` 比。
#[must_use]
pub fn user_visible_trees_changed(
    root_pointers: &UserVisibleTreeRootPointers,
    previous_valid_root_pointers: &UserVisibleTreeRootPointers,
) -> bool {
    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree
}

/// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
/// 有效 = 自证合法 ∧ 按传进来的实例表不被抛弃 ∧ txg ≥ 今天的 F；非空 = 树表里 inode 树、extent 树的根指针与前一条有效根（按 (txg, 实例) 排、
/// 比它小的有效根里最大的那条）的不同（`user_visible_trees_changed`，2026-09-17 用户定案）。
///
/// # Errors
/// 一条有效根一条都没有 ⇒ `RecoveryFailure::NoValidRoot`；要比的一条有效根的树表读不出或解不开 ⇒
/// `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（读不出要走修复，不按空或非空猜）。
#[allow(
    clippy::ptr_arg,
    reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
pub fn rollback_floor_ceiling<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
) -> Result<CheckpointTxg, MountError> {
    let readable = readable_roots(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let valid: Vec<RootRecord> = readable
        .iter()
        .copied()
        .filter(|root| root.checkpoint_txg >= current_floor)
        .filter(|root| !abandoned_by_table(root, table))
        .collect();
    let mut newest_per_device: std::collections::BTreeMap<DeviceIdentity, CheckpointTxg> =
        std::collections::BTreeMap::new();
    for root in &valid {
        let device = system_configuration.immutable.region_devices[usize::try_from(
            target_for_publish(
                root.checkpoint_txg,
                system_configuration
                    .immutable
                    .sizes
                    .root_ring_slots_per_region,
            )
            .region,
        )
        .expect("区域号")];
        let newest = newest_per_device
            .entry(device)
            .or_insert(root.checkpoint_txg);
        *newest = (*newest).max(root.checkpoint_txg);
    }
    let newest_on_every_device = newest_per_device
        .values()
        .copied()
        .min()
        .ok_or(RecoveryFailure::NoValidRoot)?;
    // 读不出或解不开就拒绝：它算不算非空、它后面那条跟谁比都判不了，读不出要走修复（D16（发布语义） 已定项 1「非空」从盘上怎么认只说了比树表里的根指针）。
    let pointers_of = |root: &RootRecord| match user_visible_tree_root_pointers(devices, root) {
        Ok(pointers) => Ok(pointers),
        Err(failure) => Err(
            MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable {
                root: RollbackTarget {
                    instance: root.instance,
                    checkpoint_txg: root.checkpoint_txg,
                },
                failure,
            },
        ),
    };
    // 「前一条」只在有效根里找：被抛弃时间线的根与 F 之下的根不算（D16（发布语义） 已定项 1「非空」从盘上怎么认）。
    let previous_candidates: &[RootRecord] = &valid;
    let mut non_empty: Vec<CheckpointTxg> = Vec::new();
    for root in &valid {
        let previous_valid_root = previous_candidates
            .iter()
            .filter(|candidate| {
                (candidate.checkpoint_txg, candidate.instance)
                    < (root.checkpoint_txg, root.instance)
            })
            .max_by_key(|candidate| (candidate.checkpoint_txg, candidate.instance));
        let previous_pointers = match previous_valid_root {
            Some(previous) => pointers_of(previous)?,
            None => UserVisibleTreeRootPointers::ABSENT,
        };
        let root_pointers = pointers_of(root)?;
        if user_visible_trees_changed(&root_pointers, &previous_pointers) {
            non_empty.push(root.checkpoint_txg);
        }
    }
    Ok(
        ceiling_from_newest_and_non_empty_roots(newest_on_every_device, non_empty, &valid)
            .ok_or(RecoveryFailure::NoValidRoot)?,
    )
}

/// 上限 = min(每块盘上最新的有效根, 第 4 新的非空有效根)；非空有效根不足 4 个时取最旧的有效根；一条有效根都没有时 `None`。
fn ceiling_from_newest_and_non_empty_roots(
    newest_on_every_device: CheckpointTxg,
    mut non_empty: Vec<CheckpointTxg>,
    valid: &[RootRecord],
) -> Option<CheckpointTxg> {
    non_empty.sort_unstable_by(|left, right| right.cmp(left));
    let fourth_newest = non_empty
        .get(3)
        .copied()
        .or_else(|| valid.iter().map(|root| root.checkpoint_txg).min())?;
    Some(newest_on_every_device.min(fourth_newest))
}

/// 抬 F 的空发布做完之后的东西。
pub struct RaisedFloor {
    pub ceiling: CheckpointTxg,
    /// 带新 F 的空发布，直到每块盘上都有一条带新 F 的持久根（生效条件）。
    pub publishes: Vec<TransactionOutput>,
    /// 生效之后回收的落点（两盘同槽，按盘 0 报）。
    pub reclaimed: Vec<Placement>,
    /// 抬 F 重算影子账时读不出账、被跳过的被抛弃根的条数（与 `MountOutput::abandoned_roots_unreadable` 同一个量；
    /// 代码三方第三轮云端攻方腿打中：抬 F 那一处此前把计数丢掉了）。
    pub abandoned_roots_unreadable: u64,
}

/// 抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）。
///
/// ⚠️ **今天只有测试入口，没有产品路径**：`crates/*/src/` 里一个调用点都没有（2026-09-22 现查），
/// 调用点全在 harness 与用例里。D16（发布语义） 已定项 1 说的「准入不够就抬 F」那条产品触发**还没实现**，
/// 欠账 C482（只供测试的开关有三个走了哪一条看不出来） 第 ② 条。在它实现之前，这个函数是
/// 「只供测试强制进入」的那一档（`.claude/rules/fs-design.md` 五条硬要求第 2 条），而不是一个
/// 「绕过正常判据」的开关——正常判据本身还不存在。
/// 推空发布直到每块盘上都有带新 F 的根（生效），然后把释放代 ≤ 新 F 的已释放落点回收。
///
/// # Errors
/// 现行那一版的根指着的实例表读不出或解不开（`InstanceTableMalformed`）；`new_floor` 超过上限；
/// 那一串空发布在任何写之前的预演里有一次报错（`RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`，一次都不发）；
/// 预演过了、真发时有一次发不出去（`RaiseFloorSequencePublishFailed`，带着前面已经落盘的那几次与失败那一次的写账）。
pub fn raise_rollback_floor<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<RaisedFloor, MountError> {
    // 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：这一串的第一次空发布本来就会被拒，
    // 而下面的影子账与回收在发布之前就动分配器——重发成功时分配器整个换成那次发布之后的一份，这些改动会被一起丢掉。第一道就拒，一样都不动。
    if let Some(frozen) = allocator.frozen_publish() {
        return Err(MountError::RaiseFloorSequencePublishFailed(
            PublishSequenceFailed {
                cause: PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
                writes_of_persisted_publishes: Vec::new(),
                writes_of_failed_publishes: Vec::new(),
            },
        ));
    }
    // 下面的影子账重算与回收在第一次空发布之前就动分配器：第一次空发布在任何写之前被拒（落点、准入、释放判定），这一串一次都没落盘、
    // F 没有一条根带出去，分配器整个换回这一份——扣住的槽放开、回收的回到 defer、补的隔离撤掉（C546（抬 F 被拒时扣住的槽不退回））。
    let allocator_before_the_raise = allocator.clone();
    let system_configuration = choose_system_configuration(&*devices)?;
    // 候选集按现行那一版的实例表判：从它的根指着的第 0 片沿链真读出来、解出来（C502（抬 F 时现行版本里没有实例表单元）；
    // D18（块里携带什么信息） 已定项 11：一张表可以不止一片）。不看 `TransactionOutput::units`——那是这个进程内存里的角色列表，
    // 第一个文件版本那一版起就不带实例表单元，mkfs 同一个进程里后面发多少次都一样，而表就在根指针后面、读得出也解得开。
    let table = instance_table_chain_of_root(&*devices, &current.root)
        .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
        .records;
    let ceiling = rollback_floor_ceiling(
        devices,
        &system_configuration,
        current.root.rollback_floor,
        &table,
    )?;
    if new_floor > ceiling {
        return Err(MountError::RollbackFloorAboveCeiling {
            requested: new_floor,
            ceiling,
        });
    }
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(txg, parameters.geometry.root_ring_slots_per_region);
        parameters.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
    // 不许发出去：带新 F 的空发布自己就在分配固定点，开放段满了会开到刚回收空的那一段（alloc-basis 第二轮云端攻方腿打中），所以先扣住、
    // 落满每块盘之后再放开。记账按写那条根的那一刻的 F 算还是按它持久之后的 F_生效 算，口径交 alloc-basis 那一轮（预想）。
    let oldest_valid_root = readable_roots(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| !abandoned_by_table(root, &table))
    .map(|root| root.checkpoint_txg)
    .min();
    // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
    // 不然那个槽回收之后就发得出去（步 4 / 步 5 代码三方第二轮辩方腿：窄读法要按每次挂载与每次抬 F 的候选集现算）。
    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {
        let roots = readable_roots(
            devices,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        );
        isolate_slots_referenced_only_by_abandoned_roots(
            devices,
            allocator,
            &roots,
            &|root| abandoned_by_table(root, &table),
            new_floor,
            &current.allocation_records,
        )
        .abandoned_roots_unreadable
    } else {
        0
    };
    let reclaimed = allocator.reclaim_released_up_to(
        reclaim_floor(new_floor, oldest_valid_root),
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );
    let planned_txgs = txgs_of_the_publishes_carrying_the_floor_to_every_device(
        current.root.checkpoint_txg,
        &all_devices,
        &device_of_txg,
    );
    let publishes_in_the_sequence = planned_txgs.len();
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    // 这一串在任何写之前在分配器的一份拷贝上整串预演（与可写挂载取号之前那一串同一个做法，`dry_run_of_the_publishes_after_acquisition`）：
    // 第 n 次在拷贝上报错（取不到落点、别的落盘之前的错），这一串一次都不发——F 不带出去，分配器整个换回抬 F 之前那一份
    // （扣住的槽放开、回收的回到 defer、补的隔离撤掉，C546（抬 F 被拒时扣住的槽不退回））。不预演时前 n − 1 次已经落盘：
    // 盘上带着新 F 而 F 没在每块盘上生效，扣住的槽留在这个进程里、计数上是空闲而发不出去（实二五报告第六节 Q3），
    // 而回退留下空档时那条落了盘的新 F 让 checker 在合法状态上判 I-3.1 红（增补 2 收口表第 43 行那一形，
    // 代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节「越格线索」的两个种子）。
    let placements_rehearsed = match rehearse_the_publishes_raising_the_floor(
        &pool,
        allocator,
        current,
        new_floor,
        &planned_txgs,
    ) {
        Ok(placements_rehearsed) => placements_rehearsed,
        Err(refusal) => {
            *allocator = allocator_before_the_raise;
            return Err(
                MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                    refused_publish_in_the_sequence: refusal.refused_publish_in_the_sequence,
                    publishes_in_the_sequence,
                    cause: refusal.cause,
                },
            );
        }
    };
    let mut publishes = Vec::new();
    // 迭代上界是预演过的次数（至多根环区域数）；跨轮携带的是现行那一版（下一次的计划接着它）。
    for planned_txg in &planned_txgs {
        let published = publish_version(
            &mut pool,
            allocator,
            empty_publish_plan_raising_the_floor(current, new_floor),
            Some(&*current),
        );
        let next = match published {
            Ok(next) => next,
            Err(cause) => {
                // 预演过了还在这里报错的只剩两种：落盘途中失败（那一次冻结在分配器上等原样重发，它的字节按回收之后的账装的，不换），
                // 与真发时读盘核出对不上、那一份留在已分配，而那一槽在拷贝上被这一串自己回收又发了出去（两边分叉，
                // 同 `establish_instance` 那一格）。第一次就在任何写之前被拒的，这一串什么都没落盘，分配器换回抬 F 之前的那一份；
                // 第二次起被拒的，前面已落盘的根带着新 F 与回收之后的账，扣住位照旧留在这个进程里，下一次抬 F 做成才放开
                // （C516（抬 F 那一串发布被拒时前面几次已落盘）；这一格扣住位怎么放，条款没写）。
                if publishes.is_empty() && allocator.frozen_publish().is_none() {
                    *allocator = allocator_before_the_raise.clone();
                }
                // 抬 F 的写入口是这里开的、随错一起丢掉：已经落盘的那几次各自的写与失败那一次已记的写都随错交出（增补 2 收口表第 58 行）。
                return Err(MountError::RaiseFloorSequencePublishFailed(
                    publish_sequence_failed(
                        &pool,
                        publishes
                            .iter()
                            .map(|persisted: &TransactionOutput| persisted.writes.clone())
                            .collect(),
                        cause,
                    ),
                ));
            }
        };
        assert_eq!(
            next.root.checkpoint_txg, *planned_txg,
            "真发的 txg 与这一串计划里的那一个相同：计划从现行那一版的 txg 逐个加一，每一次都接着现行那一版加一"
        );
        publishes.push(next.clone());
        *current = next;
    }
    // 预演取的落点就是真发取的：同一个分配器、同一串分配动作。有一份换下的单元读盘核对不上时不比（那一份真发时留在已分配，
    // 拷贝上照常释放，那一槽若在这一串里被回收两边可以不同，同 `establish_instance` 那一格）。
    let some_copy_was_quarantined = publishes.iter().any(|version: &TransactionOutput| {
        !version
            .quarantined_after_release_checksum_mismatch
            .is_empty()
    });
    if !some_copy_was_quarantined {
        let placements_taken: Vec<PlacementsTakenByOnePublish> = publishes
            .iter()
            .map(placements_taken_by_a_file_version)
            .collect();
        assert_eq!(
            placements_taken, placements_rehearsed,
            "抬 F 之前在分配器的拷贝上预演取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
             不同说明两处的计划、次序或记根分叉了，预演判的不是真发的那一串"
        );
    }
    allocator.release_reclaim_holds();
    Ok(RaisedFloor {
        ceiling,
        publishes,
        reclaimed,
        abandoned_roots_unreadable,
    })
}

/// 抬 F 那一串空发布的 txg：从现行那一版的下一个 txg 起逐次加一，直到每块盘上都落过一条（那一次的根落在哪块盘由根环落点公式定，
/// `device_of_txg`），至多根环区域数那么多次（D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」）。预演与真发都按这一串推。
fn txgs_of_the_publishes_carrying_the_floor_to_every_device(
    current_txg: CheckpointTxg,
    all_devices: &[DeviceIdentity],
    device_of_txg: &dyn Fn(CheckpointTxg) -> DeviceIdentity,
) -> Vec<CheckpointTxg> {
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publishes: Vec<CheckpointTxg> = Vec::new();
    // 迭代上界是根环区域数；跨轮携带的是已经落到了哪几块盘与上一次的 txg。
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let txg = CheckpointTxg(publishes.last().map_or(current_txg, |previous| *previous).0 + 1);
        let device = device_of_txg(txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        publishes.push(txg);
    }
    publishes
}

/// 抬 F 那一串的一次空发布的计划：接在现行那一版之后，带新 F。预演与真发读同一张计划（`raise_rollback_floor`）。
fn empty_publish_plan_raising_the_floor<'plan>(
    current: &TransactionOutput,
    new_floor: CheckpointTxg,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
        counter: current.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current
            .highest_transaction_number_in_this_instance,
        instance: current.root.instance,
        back_chain: back_chain_of(&current.record_bytes),
        file: None,
        // 抬 F 的这几次空发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current.root.instance_table),
        tree_birth_txg: current.tree_birth_txg(),
        // 一个树 ID 都不发：现行那一版的水位就是根环里的 max（本会话每次发布只照抄或推高它）。
        tree_identifier_watermark: current.root.tree_identifier_watermark,
        rollback_floor: new_floor,
    }
}

/// 抬 F 那一串在预演里第几次（从 1 数）报了什么错。
struct RaiseFloorRehearsalRefusal {
    refused_publish_in_the_sequence: usize,
    cause: PublishError,
}

/// 抬 F 那一串空发布（txg 依次是 `planned_txgs`）在分配器的一份拷贝上逐次预演：只动拷贝、不碰盘，走的就是发布路径落盘之前那一段
/// （`transaction::prepare_the_version_publish`，真发的 `publish_version` 走同一段），换下的每一份不读盘核
/// （`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）、照常释放。交回每一次取到的落点。
///
/// # Errors
/// 第几次（从 1 数）的准备报了什么错（[`RaiseFloorRehearsalRefusal`]）。
fn rehearse_the_publishes_raising_the_floor<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    new_floor: CheckpointTxg,
    planned_txgs: &[CheckpointTxg],
) -> Result<Vec<PlacementsTakenByOnePublish>, RaiseFloorRehearsalRefusal> {
    let mut copy = allocator.clone();
    let mut rehearsed_current = current.clone();
    let mut taken_by_every_publish = Vec::with_capacity(planned_txgs.len());
    // 迭代上界是这一串的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）。
    for (publish_index, planned_txg) in planned_txgs.iter().enumerate() {
        let (_, rehearsed) = prepare_the_version_publish(
            pool,
            &mut copy,
            &empty_publish_plan_raising_the_floor(&rehearsed_current, new_floor),
            Some(&rehearsed_current),
            rehearsed_current.tree_identifiers,
            ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
        )
        .map_err(|cause| RaiseFloorRehearsalRefusal {
            refused_publish_in_the_sequence: publish_index + 1,
            cause,
        })?;
        assert_eq!(
            rehearsed.root.checkpoint_txg, *planned_txg,
            "预演的 txg 与这一串计划里的那一个相同：计划从现行那一版的 txg 逐个加一，预演每一次都接着预演出来的现行那一版加一"
        );
        taken_by_every_publish.push(placements_taken_by_a_file_version(&rehearsed));
        rehearsed_current = rehearsed;
    }
    Ok(taken_by_every_publish)
}

/// 写行那次发布的身份字段：新实例的第一次发布。
struct RowPublishIdentity {
    txg: CheckpointTxg,
    counter: u64,
    instance: InstanceGeneration,
    rollback_floor: CheckpointTxg,
    /// 根环里全部根记录的树 ID 水位取 max（`InstanceStart::tree_identifier_watermark_of_the_ring`）：写行那次发布一个号都不发，
    /// 新根的水位就是它（D8（核心索引结构） 已定项 8 ②）。
    tree_identifier_watermark: u64,
}

/// 写行那次发布、上一版带文件的计划：在上一版那张实例表后面接上这次写的行，重写整条实例表链与固定点单元（D18（块里携带什么信息） 已定项 11；
/// 记账树已经存在 ⇒ 空发布也重写固定点单元，D16（发布语义） 已定项 9）。事务号 0、本实例第一条反向链 0。
/// 取号之后的发布与取号之前的预演读的是同一张计划（`publish_rows_on_file_version`、`dry_run_of_the_publishes_after_acquisition`）。
fn row_publish_plan_on_file_version<'plan>(
    previous: &TransactionOutput,
    instance_table: InstanceTableRewrite,
    identity: &RowPublishIdentity,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: identity.txg,
        counter: identity.counter,
        transaction: 0,
        // 新实例的第一条记录（反向链恒 0），事务号按实例各算各的，从这里重新从 1 起。
        highest_transaction_number_before_this_publish: 0,
        instance: identity.instance,
        back_chain: 0,
        file: None,
        // 写行那次发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Rewrite(instance_table),
        tree_birth_txg: previous.tree_birth_txg(),
        tree_identifier_watermark: identity.tree_identifier_watermark,
        rollback_floor: identity.rollback_floor,
    }
}

/// 写行那次发布、上一版带文件：按 [`row_publish_plan_on_file_version`] 发。
fn publish_rows_on_file_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance_table: InstanceTableRewrite,
    identity: RowPublishIdentity,
) -> Result<TransactionOutput, PublishError> {
    publish_version(
        pool,
        allocator,
        row_publish_plan_on_file_version(previous, instance_table, &identity),
        Some(previous),
    )
}

/// 暖机的一次空发布接在带文件的现行那一版后面时的计划：重写固定点单元（记账树存在，D16（发布语义） 已定项 9），不写文件内容、不碰 inode 树、
/// 实例表照抄。取号之后的发布与取号之前的预演读的是同一张计划。
fn empty_publish_plan_after_file_version<'plan>(
    current_file_version: &TransactionOutput,
    instance: InstanceGeneration,
) -> PublishPlan<'plan> {
    PublishPlan {
        txg: CheckpointTxg(current_file_version.root.checkpoint_txg.0 + 1),
        counter: current_file_version.record.counter + 1,
        transaction: 0,
        highest_transaction_number_before_this_publish: current_file_version
            .highest_transaction_number_in_this_instance,
        instance,
        back_chain: back_chain_of(&current_file_version.record_bytes),
        file: None,
        // 暖机那几次空发布不碰 inode 树。
        new_inode_records: &[],
        instance_table: InstanceTablePlan::Carry(current_file_version.root.instance_table),
        tree_birth_txg: current_file_version.tree_birth_txg(),
        // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
        tree_identifier_watermark: current_file_version.root.tree_identifier_watermark,
        rollback_floor: current_file_version.root.rollback_floor,
    }
}

/// 暖机的一次空发布接在树表 0 条的现行那一版后面时的计划（零单元，D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）。
fn empty_publish_plan_after_version_without_file(
    current_version_without_file: &VersionWithoutFilePublishOutput,
    instance: InstanceGeneration,
) -> ZeroUnitPublishPlan {
    ZeroUnitPublishPlan {
        txg: CheckpointTxg(current_version_without_file.root.checkpoint_txg.0 + 1),
        counter: current_version_without_file.record.counter + 1,
        instance,
        back_chain: back_chain_of(&current_version_without_file.record_bytes),
        rollback_floor: current_version_without_file.root.rollback_floor,
        // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
        tree_identifier_watermark: current_version_without_file.root.tree_identifier_watermark,
    }
}

/// 暖机的一次空发布，接在现行那一版后面：带文件的一版重写固定点单元（记账树存在，D16（发布语义） 已定项 9），
/// 树表 0 条的一版写零个单元（同一条已定项「树表 0 条 ⇒ 零单元」）。
fn publish_empty_after<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    current: &PoolVersion,
    instance: InstanceGeneration,
) -> Result<PoolVersion, PublishError> {
    match current {
        PoolVersion::WithFile(current_file_version) => publish_version(
            pool,
            allocator,
            empty_publish_plan_after_file_version(current_file_version, instance),
            Some(current_file_version),
        )
        .map(PoolVersion::WithFile),
        PoolVersion::WithoutFile(current_version_without_file) => {
            let published = publish_without_units(
                pool,
                &current_version_without_file.root,
                empty_publish_plan_after_version_without_file(
                    current_version_without_file,
                    instance,
                ),
            )
            .map_err(PublishError::from)?;
            // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`）。
            allocator.record_root_written_by_this_process(published.root.checkpoint_txg);
            Ok(PoolVersion::WithoutFile(published))
        }
    }
}

/// 取号之后那一串里一次发布取到的落点：这次重写的角色，按取落点的次序（bump 次序）各一个槽。零单元的发布一个都不取。
type PlacementsTakenByOnePublish = Vec<(TransactionUnit, SlotNumber)>;

/// 取号之后那一串里第几次发布（从 0 数：0 是写行那次，之后是暖机）在分配器的拷贝上预演时报的错。
struct DryRunRefusal {
    publish_index: usize,
    cause: PublishError,
}

/// 取号之前把这次挂载取号之后要发的那一串——写行一次、暖机 `warm_up_publish_txgs` 那几次——在分配器的一份拷贝上整串预演一遍，
/// 交回每一次取到的落点。只动拷贝、不碰盘；**走的就是发布路径落盘之前那一段**（带文件的一版 `transaction::prepare_the_version_publish`，
/// 树表 0 条的一版上写行 `transaction::prepare_the_row_publish_on_a_version_without_file`，零单元发布
/// `transaction::prepare_the_publish_without_units`），计划也是取号之后那几次发布用的同一张（`row_publish_plan_on_file_version`、
/// `empty_publish_plan_after_file_version`、`empty_publish_plan_after_version_without_file`），不另推一份：
/// 这一串每一次重写哪些角色——分配记录树重写哪几个节点要在分配器上走到固定点才知道（D8（核心索引结构） 已定项 14）——
/// 与真发时是同一段代码算的。写行那次的释放核验、树的形状、落点取不到，都在这里先报出来，在取号之前拒（D18（块里携带什么信息） 已定项 11
/// 「可写挂载的顺序」第五个合取；增补 2 第 20a 行）。
///
/// 发布路径在取落点之前还读盘核换下的每一份的校验和，对不上（读不出也算）的那一份在它那块盘上的分配记录留在已分配、不释放
/// （D19（块指针的结构与宽度预算） 已定项 5），这里不读盘（`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）、照常释放。
/// 两边只在那一槽这一串里被回收时分叉：拷贝上它回收了、可能再发出去，真发时它一直占着；这一串里回收它，得这一串自己的根把根环里
/// 比它旧的有效根全盖掉（回退到环里最旧的那条根、其余全被抛弃时走得到）。所以这一串里没有一份核出对不上时，这里取到的与真发起来取到的逐项相同
/// （`establish_instance` 在这一串发完之后断言）；有一份核出对不上时可能不同，那一格见 `establish_instance` 的注释。
///
/// # Errors
/// 第几次（从 0 数）的准备报了什么错（[`DryRunRefusal`]）。
fn dry_run_of_the_publishes_after_acquisition<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    start: &InstanceStart,
    instance_table_rewrite: &InstanceTableRewrite,
    rows_to_write: usize,
    warm_up_publish_txgs: &[CheckpointTxg],
    instance_to_acquire: InstanceGeneration,
) -> Result<Vec<PlacementsTakenByOnePublish>, DryRunRefusal> {
    let mut copy = allocator.clone();
    let refused_at = |publish_index: usize| {
        move |cause: PublishError| DryRunRefusal {
            publish_index,
            cause,
        }
    };
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, .. } => {
            let plan = row_publish_plan_on_file_version(
                output,
                instance_table_rewrite.clone(),
                &RowPublishIdentity {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            prepare_the_version_publish(
                pool,
                &mut copy,
                &plan,
                Some(output),
                output.tree_identifiers,
                ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
            )
            .map(|(_, rehearsed)| PoolVersion::WithFile(rehearsed))
            .map_err(refused_at(0))?
        }
        PreviousVersion::WithoutFile { root, .. } if rows_to_write == 0 => {
            let (_, rehearsed) = prepare_the_publish_without_units(
                pool,
                root,
                ZeroUnitPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
            PoolVersion::WithoutFile(rehearsed)
        }
        PreviousVersion::WithoutFile { root, .. } => {
            prepare_the_row_publish_on_a_version_without_file(
                pool,
                &mut copy,
                root,
                InstanceTableOnlyPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    instance_table: instance_table_rewrite,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map(|(_, rehearsed)| PoolVersion::WithoutFile(rehearsed))
            .map_err(refused_at(0))?
        }
    };
    let mut taken_by_every_publish = vec![placements_taken_by(&row_publish)];
    let mut current = row_publish;
    // 迭代上界是暖机的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）。
    for (warm_up_position, planned_txg) in warm_up_publish_txgs.iter().enumerate() {
        let publish_index = warm_up_position + 1;
        let next = match &current {
            PoolVersion::WithFile(current_file_version) => {
                let plan = empty_publish_plan_after_file_version(
                    current_file_version,
                    instance_to_acquire,
                );
                prepare_the_version_publish(
                    pool,
                    &mut copy,
                    &plan,
                    Some(current_file_version),
                    current_file_version.tree_identifiers,
                    ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                )
                .map(|(_, rehearsed)| PoolVersion::WithFile(rehearsed))
                .map_err(refused_at(publish_index))?
            }
            PoolVersion::WithoutFile(current_version_without_file) => {
                let (_, rehearsed) = prepare_the_publish_without_units(
                    pool,
                    &current_version_without_file.root,
                    empty_publish_plan_after_version_without_file(
                        current_version_without_file,
                        instance_to_acquire,
                    ),
                );
                copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
                PoolVersion::WithoutFile(rehearsed)
            }
        };
        assert_eq!(
            next.root().checkpoint_txg,
            *planned_txg,
            "预演的暖机 txg 与计划里的那一个相同：计划逐个加一，计划接着现行那一版的 txg 加一"
        );
        taken_by_every_publish.push(placements_taken_by(&next));
        current = next;
    }
    Ok(taken_by_every_publish)
}

/// 带文件的一版的一次发布真取到的落点，按取落点的次序：这次重写的每个角色（`TransactionOutput::rewritten`）。
fn placements_taken_by_a_file_version(output: &TransactionOutput) -> PlacementsTakenByOnePublish {
    output
        .rewritten
        .iter()
        .map(|role| (*role, output.unit(*role).slot))
        .collect()
}

/// 一次发布真取到的落点，按取落点的次序：带文件的一版见 [`placements_taken_by_a_file_version`]；
/// 树表 0 条的一版是这次那条记录的点名项：写行那次按取落点的次序点名实例表链各片（尾片先）与分配记录树节点
/// （`transaction::publish_instance_table_on_version_without_file`；第 1 片起的指针不在根记录里，只在记录的点名项与上一片的链指针里），
/// 零单元发布一项都不点名、一个都没有。
fn placements_taken_by(version: &PoolVersion) -> PlacementsTakenByOnePublish {
    match version {
        PoolVersion::WithFile(output) => placements_taken_by_a_file_version(output),
        PoolVersion::WithoutFile(version_without_file) => {
            // 点名项按取落点的次序排在这次发布的各条记录里（末条再跨记录时前几条在 `earlier_records_of_this_publish`，
            // D23（journal 的角色与格式） 已定项 17），按记录的次序接起来就是整次发布的点名项。
            let named: Vec<&crate::journal::NamedUnit> = version_without_file
                .earlier_records_of_this_publish
                .iter()
                .map(|written| &written.record)
                .chain(std::iter::once(&version_without_file.record))
                .flat_map(|record| record.named.iter())
                .collect();
            assert_eq!(
                named.len(),
                version_without_file.rewritten.len(),
                "树表 0 条的一版上一次发布的点名项与它写出的角色一一对应（写行那次按同一张角色清单点名，零单元发布两边都空）"
            );
            version_without_file
                .rewritten
                .iter()
                .copied()
                .zip(named)
                .map(|(role, named_unit)| {
                (
                    role,
                    slot_shared_by_both_location_entries(&named_unit.locations).expect(
                        "这个进程这次发布刚写的指针，两条位置条目按一个落点写（两盘同槽，`PoolWriter::location_entries`）",
                    ),
                )
            })
            .collect()
        }
    }
}

/// 暖机要推的那几次空发布，按它们的 checkpoint_txg 列出来（D16（发布语义） 已定项 8 戊）：从写行那次发布的 txg 起逐个加一，
/// 按根环落点公式看落哪块盘，直到本实例的根覆盖每块盘，至多根环的区域数那么多次。
/// 纯算——只读区域归属表与池里的盘，不碰盘、不碰分配器、不看分配记录，所以取号之前算得出来。
/// 取号之前算准入与取号之后推发布共用这一份计划（`establish_instance` 按它推），两处的次数因此必定相同：
/// 各算各的时只要有一处多算一次，准入就罩不住实际推的那几次。
fn warm_up_publish_txgs(
    parameters: &MakeFilesystemParameters,
    all_devices: &[DeviceIdentity],
    row_publish_txg: CheckpointTxg,
) -> Vec<CheckpointTxg> {
    let device_of_txg = |txg: CheckpointTxg| {
        parameters.region_devices[usize::try_from(
            target_for_publish(txg, parameters.geometry.root_ring_slots_per_region).region,
        )
        .expect("区域号")]
    };
    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish_txg)];
    let mut warm_up_publishes: Vec<CheckpointTxg> = Vec::new();
    let mut latest_txg = row_publish_txg;
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next_txg = CheckpointTxg(latest_txg.0 + 1);
        let device = device_of_txg(next_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        warm_up_publishes.push(next_txg);
        latest_txg = next_txg;
    }
    warm_up_publishes
}

/// 这次挂载要写的行：给 [max(上一个实例, 1), 要取的号) 里每个实例各一行——上一个实例 (i, T, W)（回退时是回退行，flags bit0 = 1），
/// 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。纯算，取号之前就列得出来。
fn instance_rows_to_write(
    previous_row: &PreviousInstanceRow,
    instance_to_acquire: InstanceGeneration,
) -> Vec<InstanceRow> {
    let first_row_instance = previous_row.instance.0.max(1);
    (first_row_instance..instance_to_acquire.0)
        .map(|row_instance| {
            if row_instance == previous_row.instance.0 {
                InstanceRow {
                    instance: InstanceGeneration(row_instance),
                    selected_root_txg: previous_row.selected_root_txg,
                    applied_transaction_high_water: previous_row.applied_transaction_high_water,
                    is_rollback: previous_row.is_rollback,
                }
            } else {
                InstanceRow {
                    instance: InstanceGeneration(row_instance),
                    selected_root_txg: CheckpointTxg(0),
                    applied_transaction_high_water: 0,
                    is_rollback: false,
                }
            }
        })
        .collect()
}

/// 一串发布里有一次失败了：错，连同这一串的写入口交得出的账——`writes_of_persisted_publishes` 是这一串里已经落盘的那几次各自的写
/// （按先后），失败那一次落盘阶段已记的写从写入口的失败账取。可写挂载与抬 F 共用这一处。
fn publish_sequence_failed<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    writes_of_persisted_publishes: Vec<WritesByStructureKind>,
    cause: PublishError,
) -> PublishSequenceFailed {
    PublishSequenceFailed {
        cause,
        writes_of_persisted_publishes,
        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
    }
}

/// 取号之后的一次发布失败了：错，连同这次挂载的写入口交得出的账——`persisted` 是这次挂载里已经落盘的那几次（按先后）。
fn publish_failed_after_acquisition<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    persisted: &[&PoolVersion],
    cause: PublishError,
) -> MountError {
    MountError::Publish(publish_sequence_failed(
        pool,
        persisted
            .iter()
            .map(|version| match version {
                PoolVersion::WithoutFile(version_without_file) => {
                    version_without_file.writes.clone()
                }
                PoolVersion::WithFile(file_version) => file_version.writes.clone(),
            })
            .collect(),
        cause,
    ))
}

/// 回退见证的删除规则（D23（journal 的角色与格式） 已定项 14「回退见证」：条目什么时候删随实现；实二二三定，交代码三方）：
/// 条目 (N, r_old, T_old) 删掉 ⟺ 根环每一个槽都读得出、自证过（`every_root_ring_slot_holds_a_root`），而且其中没有一条根的
/// 实例代号落在 [r_old, N) 里。别的一律留着。
///
/// 为什么是这两条：
/// - 条目抛弃的是实例代号落在 (r_old, N) 里的根、与实例 r_old 里 txg 越过 T_old 的根；它还护着重放——所选根是实例 r_old、txg 不超过
///   T_old 的根时，重放会走到 r_old 越过 T_old 的被抛弃记录（`recovery::replay_journal` 那一判）。环里没有一条根的实例代号落在
///   [r_old, N) 里，这两样都没有对象；之后新写的根的实例代号都不小于这次挂载取的号（大于 N），环里再也长不出这样的根，删掉永远安全。
/// - 读不出、自证不过的槽按「可能住着这样一条根」算，与 D18（块里携带什么信息） 已定项 11 实例表行回收的根环条件同一个读法：
///   一个暂时读不出的槽能让条目被删、槽恢复之后被抛弃的根回到择根的候选里，就是 C332 那一格。代价：根环有一个槽持续读不出时条目永远删不掉。
fn rollback_witness_entries_still_needed(
    entries: &[RollbackWitnessEntry],
    every_ring_root: Option<&[RootRecord]>,
) -> Vec<RollbackWitnessEntry> {
    entries
        .iter()
        .copied()
        .filter(|entry| match every_ring_root {
            None => true,
            Some(roots) => roots.iter().any(|root| {
                entry.rollback_target_instance <= root.instance
                    && root.instance < entry.new_instance
            }),
        })
        .collect()
}

/// 回退见证删除规则的第二条（D23（journal 的角色与格式） 已定项 14「回退见证的实现取法」①，代码轮第二轮判决第四节第 1 条，
/// 主 agent 2026-09-25 定，被攻过零轮）：条目 (N1, r, T) 被同一张表里另一条 (N2, r2, T2) 罩住 ⟺ N2 ≥ N1 且 (r2, T2) ≤ (r, T)
/// （实例代号为主比），罩住的删掉。被罩住的那条抛弃的每一处 (i, t)——(r, T) < (i, t) 且 i < N1——罩住它的那条也抛弃：
/// (r2, T2) ≤ (r, T) < (i, t) 且 i < N1 ≤ N2，所以删掉之后整张表的并集判法逐字不变，这一条不靠测。
/// 只在同一张要写出去的表里比：罩住它的那条与它一起落盘，不拿一条还没落盘的条目去删已经落盘的那一条。
/// 两两比较的上界是表的条数（至多 R × S − 1 = 47）的平方。
fn rollback_witness_entries_not_covered_by_another(
    entries: &[RollbackWitnessEntry],
) -> Vec<RollbackWitnessEntry> {
    entries
        .iter()
        .copied()
        .filter(|entry| {
            !entries.iter().any(|other| {
                other != entry
                    && other.new_instance >= entry.new_instance
                    && (other.rollback_target_instance, other.rollback_target_txg)
                        <= (entry.rollback_target_instance, entry.rollback_target_txg)
            })
        })
        .collect()
}

/// 按所选根指着的实例表里的回退行，推出每一次回退该有的见证条目（D23（journal 的角色与格式） 已定项 14「回退见证的实现取法」⑥，
/// 代码轮第二轮判决第四节第 3 条，主 agent 2026-09-25 定，被攻过零轮）：回退行 (r_old, T_old, ·, 回退) 对应条目 (N, r_old, T_old)，
/// N 是做那次回退的实例——回退那一次挂载写的行是 [r_old, N)：回退行，与中间实例各一行 (i, 0, 0)；N 自己那一行由下一次挂载写，
/// T 是 N 在那次挂载里择到的根的 txg，恒大于 0（能指着这张表的根只有 N 与之后实例的根）。所以 N = 回退行之后第一条 T ≠ 0 的行的实例；
/// 回退行之后只有 (i, 0, 0) 或一行都没有 ⇒ 这张表是 N 自己写行那一次写的，N = 所选根自己的实例。
///
/// ⚠️ 派发规格的原话是「N 取实例表里回退行之后第一个有行的实例」：回退跨过实例时（例如固定脚本里实例 2 的世界回退到 (1, 3)、
/// 新实例 3 写行 (1, 3, 0, 回退) 与 (2, 0, 0)）照字面取到的是中间实例 2，条目 (2, 1, 3) 罩不住实例 2 的根 C——落回 C 正是这一条要挡的；
/// 实例 2 若自己也做过一次回退，(2, …) 还会与盘上那一条说两种目标（I-7.10）。这里取「T ≠ 0」那一读，交主 agent 定。
///
/// 回退到 mkfs 的第 0 代根（r_old = 0）不写行（D18（块里携带什么信息） 已定项 11：实例 0 不写行），实例表里没有回退行，这一条推不出来。
fn rollback_witness_entries_recovered_from_the_instance_table(
    rows_of_the_chosen_roots_instance_table: &[InstanceRow],
    chosen_root_instance: InstanceGeneration,
) -> Vec<RollbackWitnessEntry> {
    rows_of_the_chosen_roots_instance_table
        .iter()
        .filter(|row| row.is_rollback)
        .map(|rollback_row| {
            let new_instance = rows_of_the_chosen_roots_instance_table
                .iter()
                .filter(|later_row| {
                    later_row.instance > rollback_row.instance
                        && later_row.selected_root_txg != CheckpointTxg(0)
                })
                .map(|later_row| later_row.instance)
                .min()
                .unwrap_or(chosen_root_instance);
            RollbackWitnessEntry {
                new_instance,
                rollback_target_instance: rollback_row.instance,
                rollback_target_txg: rollback_row.selected_root_txg,
            }
        })
        .collect()
}

/// 这次挂载要写的两张回退见证表：取号那两次写带删过的旧表（回退那一条还不在里面：见证随写行之后的第一次系统配置轮换写，
/// D23（journal 的角色与格式） 已定项 14「回退见证」，写序 post），写行那次发布的轮换起带再加上这一次回退那一条的表（普通挂载两张相同）。
/// 盘上原有的见证读自各盘择到的那一槽（`recovery::rollback_witness_of_the_pool`）；删除规则之前先按所选根实例表里的回退行把缺的条目补回来
/// （`rollback_witness_entries_recovered_from_the_instance_table`：回退那一次挂载崩在写行的根落了、轮换还没落的那一格，
/// 条目没落盘而回退行已经在了），再按删除规则删（`rollback_witness_entries_still_needed`，
/// 与被同一张表里另一条罩住的删 `rollback_witness_entries_not_covered_by_another`）。
///
/// # Errors
/// 表装不下（条数越过 R × S − 1）⇒ `MountError::Recovery(RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided)`：
/// 有槽持续读不出，或每次回退都崩在写行轮换之后、暖机之前（根环全读得出也一样，代码轮第二轮判决第三节）时条目删不掉、表写得满——
/// 那时怎么办条款没写（C547（回退见证表的删除规则与写满没有条款））⇒ 第一版不支持，在任何写之前返回，盘上逐字节不变。
fn rollback_witness_tables_of_this_mount<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
    chosen_root: &RootRecord,
    rows_of_the_chosen_roots_instance_table: &[InstanceRow],
    previous_row: &PreviousInstanceRow,
    instance_to_acquire: InstanceGeneration,
) -> Result<(RollbackWitnessTable, RollbackWitnessTable), MountError> {
    let capacity = rollback_witness_capacity(
        system_configuration
            .immutable
            .sizes
            .root_ring_slots_per_region,
    );
    let witness = rollback_witness_of_the_pool(devices, system_configuration);
    let with_the_recovered_entries: BTreeSet<RollbackWitnessEntry> = witness
        .entries()
        .into_iter()
        .chain(rollback_witness_entries_recovered_from_the_instance_table(
            rows_of_the_chosen_roots_instance_table,
            chosen_root.instance,
        ))
        .collect();
    let every_ring_root = every_root_ring_slot_holds_a_root(devices, system_configuration);
    let still_needed = rollback_witness_entries_still_needed(
        &with_the_recovered_entries.into_iter().collect::<Vec<_>>(),
        every_ring_root.as_deref(),
    );
    let kept = rollback_witness_entries_not_covered_by_another(&still_needed);
    let this_rollback = previous_row.is_rollback.then_some(RollbackWitnessEntry {
        new_instance: instance_to_acquire,
        rollback_target_instance: previous_row.instance,
        rollback_target_txg: previous_row.selected_root_txg,
    });
    let full = |refused: crate::rollback_witness::RollbackWitnessTableFull| {
        MountError::Recovery(
            RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided {
                entries: refused.entries,
                capacity: refused.capacity,
            },
        )
    };
    let before_the_row_publish =
        RollbackWitnessTable::of_entries(kept.iter().copied(), capacity).map_err(full)?;
    let from_the_row_publish = RollbackWitnessTable::of_entries(
        rollback_witness_entries_not_covered_by_another(
            &kept.into_iter().chain(this_rollback).collect::<Vec<_>>(),
        ),
        capacity,
    )
    .map_err(full)?;
    Ok((before_the_row_publish, from_the_row_publish))
}

/// 所选那一版在 journal 里的位置：它那次发布的末条记录的 jsn（调用方从环里认出来的那一条，认不出时它顶上的那一条）；
/// 一条都拿不出来（只做过 mkfs 的池，环里没有记录）时取 (这一版的实例代号, 0)。每块盘在一次发布之后轮换的系统配置写的是
/// 同一对 (实例代号, 末条计数器)（`transaction::CommitStep::RotateSystemConfigurationSlots`），逐盘判「落后」拿它比。
fn journal_position_of_the_selected_version(
    root: &RootRecord,
    record_standing_for_root: Option<&crate::journal::JournalRecord>,
) -> JournalSequenceNumber {
    record_standing_for_root.map_or(
        JournalSequenceNumber {
            instance_generation: root.instance,
            counter: 0,
        },
        |record| JournalSequenceNumber {
            instance_generation: record.instance,
            counter: record.counter,
        },
    )
}

/// 可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13、D18（块里携带什么信息） 已定项 11）：交进来的每块盘带不带所选那一版，
/// 交回不带的那几块与各自缺的（[`SelectedVersionLackingOnDevice`]）。只读盘，不写。
///
/// 每块盘先看它两槽里自证过的系统配置：一份都没有就不带；世代号最大的那一份的 (实例代号, journal tail) 不落后于
/// `selected_version_journal_position` 就带（跟上了这一版的系统配置轮换，不读它的单元）；落后的再逐个读 `units` 在这块盘上的落点，
/// 读不出或字节与验得过的那一份不同就记下，记下了至少一个才算不带。字节是空的单元（提示与映射都读不出的数据单元，
/// D19（块指针的结构与宽度预算） 已定项 5）没有验得过的一份可比，跳过。
///
/// 循环：外层按交进来的盘（轮数 = 盘数），内层按 `units`（轮数 = 这一版的单元数）；跨轮只往交回的清单里追加。
/// 外层的提前出口两个，都是「这块盘判完了、看下一块」：没有自证过的系统配置（记下）、系统配置不落后（不记）。
fn devices_without_the_selected_version<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    selected_version_journal_position: JournalSequenceNumber,
    units: &[PublishedUnit],
) -> Vec<DeviceWithoutTheSelectedVersion> {
    let slot_spacing_in_bytes = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let mut devices_without = Vec::new();
    for device in reader.device_identities() {
        let newest_self_verified = verified_system_configuration_slots(
            reader,
            device,
            slot_spacing_in_bytes,
            &system_configuration.immutable.filesystem_identifier,
        )
        .into_iter()
        .max_by_key(|slot| slot.quantities.slot_generation);
        let Some(newest_self_verified) = newest_self_verified else {
            devices_without.push(DeviceWithoutTheSelectedVersion {
                device,
                lacking: SelectedVersionLackingOnDevice::NoSelfVerifiedSystemConfiguration,
            });
            continue;
        };
        let newest_system_configuration_journal_tail = JournalSequenceNumber {
            instance_generation: newest_self_verified.quantities.journal_instance,
            counter: newest_self_verified.quantities.journal_tail,
        };
        if newest_system_configuration_journal_tail >= selected_version_journal_position {
            continue;
        }
        let units_missing: Vec<(TransactionUnit, SlotNumber)> = units
            .iter()
            .filter(|unit| !unit.bytes.is_empty())
            .filter(|unit| {
                !reader
                    .read(device, unit.slot.to_device_offset(), unit.bytes.len())
                    .is_some_and(|bytes_on_this_device| bytes_on_this_device == unit.bytes)
            })
            .map(|unit| (unit.identity, unit.slot))
            .collect();
        if !units_missing.is_empty() {
            devices_without.push(DeviceWithoutTheSelectedVersion {
                device,
                lacking:
                    SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                        newest_system_configuration_journal_tail,
                        units_missing,
                    },
            });
        }
    }
    devices_without
}

/// 可写挂载准入的第一条合取（D2（RAID 条带策略） 已定项 13「挂载准入」：可写设备数 ≥ w 的下限；D18（块里携带什么信息） 已定项 11
/// 「可写挂载的顺序」先判这一条）：交进来的盘数低于 [`STRIPE_WIDTH_LOWER_BOUND`] 就拒。只比两个数，不读盘、不写盘。
/// 可写挂载与回退在读第一块盘之前调它；之后到取号为止交进来的那份盘表只借给重建与写入口、长短不变，取号写的就是这里数过的这几块盘。
fn admit_the_writable_device_count(devices_handed_in: DeviceCount) -> Result<(), MountError> {
    if devices_handed_in < STRIPE_WIDTH_LOWER_BOUND {
        return Err(
            MountError::WritableDeviceCountBelowTheStripeWidthLowerBound {
                devices_handed_in,
                stripe_width_lower_bound: STRIPE_WIDTH_LOWER_BOUND,
            },
        );
    }
    Ok(())
}

/// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘：可写挂载与回退共用的后半段。
fn establish_instance<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mut allocator: PoolAllocator,
    start: InstanceStart,
) -> Result<Mounted, MountError> {
    let isolated_slots_per_device: Vec<(DeviceIdentity, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    // 所选那一版的单元清单在开写入口之前取好（树表 0 条那一版要按位置条目读盘）：取号之前逐盘核拿它比。
    let units_of_the_selected_version = start.previous.units(devices)?;
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    // 暖机推哪几次，取号之前先算出来：准入按这一份算，取号之后的循环也按这一份推。
    let warm_up_publishes_planned = warm_up_publish_txgs(parameters, &all_devices, start.first_txg);
    let instance_to_acquire = instance_generation_to_acquire(&pool);
    // 要写的行按要取的号先列出来：实例表那一项准入按它算，取号之后写行也用这一份（号在写之前重算、不等就不写）。
    let rows_written = instance_rows_to_write(&start.previous_row, instance_to_acquire);
    let instance_table_rewrite = instance_table_rewrite_of_the_row_publish(
        start.previous.instance_table_chain(),
        &rows_written,
    );
    // 空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决第四节第 2 条接进可写挂载）：可写挂载的准入要
    // 「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13），预留是式子里的挂载期承诺量——每块盘上连它与别的八项都扣完，
    // 可用(d) ≥ 0（这一刻不另要需求：写行与暖机那一串的空间就在切换预留与 checkpoint 保留池里）。rows0 = 挂载时读到的行数加写行那次
    // 要写的行数（D28（挂载期承诺量） 已定项 3），挂载期间常量，记在分配器上给这次挂载之后的每次发布用。在取号之前拒，盘上逐字节不变。
    // 只供测试的开关（`SpaceAdmission::SkippedByTheTestOnlySwitch`）关掉准入时不判，这次挂载之后的发布也不判（开关装在分配器上）。
    allocator.record_instance_rows_after_this_mounts_row_publish(
        u64::try_from(instance_table_rewrite.rows.len()).expect("实例表的行数装得进 u64"),
    );
    allocator.set_space_admission(start.space_admission);
    match start.space_admission {
        SpaceAdmission::JudgedByTheFormula => {
            let no_demand_on_any_device: Vec<DemandOnDevice> = allocator
                .devices
                .iter()
                .map(|device_map| DemandOnDevice {
                    device: device_map.device,
                    bytes: BytesOnOneDevice::ZERO,
                })
                .collect();
            admit_on_every_device(
                &admission_reading_before_a_publish(&allocator, start.previous.file_version()),
                &no_demand_on_any_device,
            )
            .map_err(
                |refusal| MountError::SpaceAdmissionRefusedBeforeAcquisition {
                    instance_to_acquire,
                    refusal,
                },
            )?;
        }
        SpaceAdmission::SkippedByTheTestOnlySwitch => {}
    }
    // 取号之后要发的那一串（写行一次、暖机那几次）在取号之前整串预演一遍（分配器的拷贝上，走发布路径落盘之前那一段，
    // 与后面真发读同一个分配器——取号不碰它）：释放核验、树的形状、落点取不到，都在任何写之前返回。
    // 落点取不到怎么算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
    let placements_planned = match dry_run_of_the_publishes_after_acquisition(
        &pool,
        &allocator,
        &start,
        &instance_table_rewrite,
        rows_written.len(),
        &warm_up_publishes_planned,
        instance_to_acquire,
    ) {
        Ok(taken) => Some(taken),
        Err(DryRunRefusal {
            publish_index,
            cause: PublishError::PlacementRefused { unit, refusal },
        }) => {
            return Err(
                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
                    instance_to_acquire,
                    publish_index,
                    warm_up_publishes_planned: warm_up_publishes_planned.len(),
                    unit,
                    refusal,
                },
            )
        }
        Err(DryRunRefusal {
            publish_index: 0,
            cause,
        }) => {
            return Err(MountError::RowPublishAdmissionRefusedBeforeAcquisition {
                instance_to_acquire,
                cause,
            })
        }
        Err(DryRunRefusal {
            publish_index: warm_up_publish_index,
            cause,
        }) => {
            return Err(MountError::WarmUpAdmissionRefusedBeforeAcquisition {
                instance_to_acquire,
                warm_up_publish_index,
                warm_up_publishes_planned: warm_up_publishes_planned.len(),
                cause,
            })
        }
    };
    // 回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：装不下在取号之前拒；取号那两次写带删过的旧表，
    // 写行那次发布的轮换起带再加上这一次回退那一条的表。
    let (rollback_witness_before_the_row_publish, rollback_witness_from_the_row_publish) =
        rollback_witness_tables_of_this_mount(
            pool.devices,
            &start.system_configuration,
            &start.chosen_root,
            &start.rows_of_the_chosen_roots_instance_table,
            &start.previous_row,
            instance_to_acquire,
        )?;
    // 可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13、D18（块里携带什么信息） 已定项 11）：取号是这次挂载的第一个写，
    // 核在它之前、读的是紧接着要写的这批盘；有一块不带所选那一版就不取号，盘上逐字节不变。排在别的准入之后，
    // 那几条先判出来的仍报各自的成员。
    let devices_without = devices_without_the_selected_version(
        &*pool.devices,
        &start.system_configuration,
        start.selected_version_journal_position,
        &units_of_the_selected_version,
    );
    if !devices_without.is_empty() {
        return Err(
            MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
                selected_version: RollbackTarget {
                    instance: start.effective_root.instance,
                    checkpoint_txg: start.effective_root.checkpoint_txg,
                },
                selected_version_journal_position: start.selected_version_journal_position,
                devices: devices_without,
            },
        );
    }
    pool.write_rollback_witness_from_now_on(rollback_witness_before_the_row_publish);
    let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(
        |failure| match failure {
            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
                expected,
                recomputed,
            } => MountError::InstanceGenerationChangedBeforeAcquisition {
                expected,
                recomputed,
            },
            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {
                MountError::Acquisition(acquisition)
            }
        },
    )?;
    assert_eq!(
        instance, instance_to_acquire,
        "acquire_expected_instance 写之前重算、与判定时的号不等就不写：交回的就是列行与判准入用的那个号"
    );
    // 取号那两次写之后的第一次系统配置写就是写行那次发布的轮换（写行在根 FUA 之后才轮换，D16（发布语义） 已定项 7）：
    // 从这里起带着这一次回退那一条（D23（journal 的角色与格式） 已定项 14「回退见证」随写行之后的第一次系统配置轮换写）。
    pool.write_rollback_witness_from_now_on(rollback_witness_from_the_row_publish);

    // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishSequenceFailed`）：写入口随错一起丢掉。
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, .. } => {
            let published = publish_rows_on_file_version(
                &mut pool,
                &mut allocator,
                output,
                instance_table_rewrite,
                RowPublishIdentity {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            published.map(PoolVersion::WithFile)
        }
        // 树表 0 条的一版：要写的行为空（上一个实例是 0，第一次可写挂载）就走零单元发布，第一个事务的字节不变
        // （D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」；layout/01-first-txn.md 八「第一次之后的可写挂载（写行）」那一行 2026-09-14 的改名注）；
        // 要写的行不为空就只重写实例表那一个单元——这一版没有记账树，已定项 9 的那五样一样都不写，而 D18（块里携带什么信息） 已定项 11
        // 要求每次可写挂载都写行。
        PreviousVersion::WithoutFile { root, .. } if rows_written.is_empty() => {
            publish_without_units(
                &mut pool,
                root,
                ZeroUnitPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map_err(PublishError::from)
            .map(|published| {
                // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`）。
                allocator.record_root_written_by_this_process(published.root.checkpoint_txg);
                PoolVersion::WithoutFile(published)
            })
        }
        PreviousVersion::WithoutFile { root, .. } => {
            publish_instance_table_on_version_without_file(
                &mut pool,
                &mut allocator,
                root,
                InstanceTableOnlyPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    // 新实例的第一条记录，反向链恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    instance_table: &instance_table_rewrite,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map(PoolVersion::WithoutFile)
        }
    }
    .map_err(|cause| publish_failed_after_acquisition(&pool, &[], cause))?;

    // 暖机（D16（发布语义） 已定项 8 戊）：本实例的根覆盖两块盘之前连推空发布，推哪几次按取号之前算好的那一份计划
    // （`warm_up_publishes_planned`），不在这里另算一遍——两处各算各的，只要有一处多算一次，取号之前那道准入就罩不住实际推的那几次。
    assert_eq!(
        row_publish.root().checkpoint_txg,
        start.first_txg,
        "写行那次发布的 txg 就是算暖机计划用的那个：两条路（带文件的写行、树表 0 条的零单元发布）交给发布路径的 txg 都取 start.first_txg"
    );
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();
    for planned_txg in &warm_up_publishes_planned {
        assert_eq!(
            CheckpointTxg(current.root().checkpoint_txg.0 + 1),
            *planned_txg,
            "这次空发布要用的 txg 与计划里的那一个相同：`publish_empty_after` 取的是现行那一版的 txg 加一，计划也是逐个加一"
        );
        let next = publish_empty_after(&mut pool, &mut allocator, &current, instance).map_err(
            |cause| {
                let persisted: Vec<&PoolVersion> = std::iter::once(&row_publish)
                    .chain(&warm_up_publishes)
                    .collect();
                publish_failed_after_acquisition(&pool, &persisted, cause)
            },
        )?;
        warm_up_publishes.push(next.clone());
        current = next;
    }
    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——
    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（回退到环里最旧的那条根、其余全被抛弃），两边可以不同
    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)
        .chain(&warm_up_publishes)
        .map(placements_taken_by)
        .collect();
    let some_copy_was_quarantined = std::iter::once(&row_publish)
        .chain(&warm_up_publishes)
        .filter_map(PoolVersion::file_version)
        .any(|version| {
            !version
                .quarantined_after_release_checksum_mismatch
                .is_empty()
        });
    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {
        assert_eq!(
            &placements_taken, planned,
            "取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
             不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串"
        );
    }

    Ok(Mounted {
        output: MountOutput {
            instance,
            chosen_root: start.chosen_root,
            effective_root: start.effective_root,
            journal: start.journal,
            rows_written,
            row_publish,
            warm_up_publishes,
            shadow_ledger_branch: start.shadow_ledger.branch_name(),
            isolated_slots_per_device,
            abandoned_roots_unreadable: start.abandoned_roots_unreadable,
            rollback_witness_written: rollback_witness_from_the_row_publish,
        },
        allocator,
        current,
    })
}

/// 可写挂载：恢复 → 重建上一版与分配器 → 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘。只做过 mkfs 的池（所选根的树表 0 条、
/// 环里没有记录）同样走这条路：取号 1、不写行、零单元的发布推到本实例的根覆盖每块盘，第一个文件版本接在 `current` 后面。
///
/// # Errors
/// 交进来的盘数低于 w 的下限（`WritableDeviceCountBelowTheStripeWidthLowerBound`，在读任何一块盘之前）、
/// 恢复失败、所选根下有文件而环里一条记录都没有、实例表沿链读不出或解不开、取号之前的空间准入不过
/// （`SpaceAdmissionRefusedBeforeAcquisition`）、取号之前的预演报错、取号之后那一串在分配器的拷贝上取不到落点
/// （`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）、有盘不带所选那一版（空盘，或停在旧状态：
/// `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，那样的池只许只读挂载）、取号失败、发布失败。
pub fn mount_writable<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Result<Mounted, MountError> {
    mount_writable_with_space_admission(parameters, devices, SpaceAdmission::JudgedByTheFormula)
}

/// 同 [`mount_writable`]，判不判空间准入由调用方给（只供测试的开关 `SpaceAdmission`；产品路径走 `mount_writable`，恒判）。
/// 开关装在交回的分配器上，这次挂载之后的发布照它判不判。
///
/// # Errors
/// 同 [`mount_writable`]；`SkippedByTheTestOnlySwitch` 时不报 `SpaceAdmissionRefusedBeforeAcquisition`。
pub fn mount_writable_with_space_admission<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    space_admission: SpaceAdmission,
) -> Result<Mounted, MountError> {
    admit_the_writable_device_count(DeviceCount(devices.len()))?;
    let system_configuration = choose_system_configuration(&*devices)?;
    let chosen_root =
        choose_root(&*devices, &system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(&*devices, &system_configuration);
    let (journal, effective_root) = replay_journal(
        &*devices,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        true,
        rollback_high_water_of_root(&*devices, &chosen_root),
    )?;
    // 所选根覆盖的最后一条记录读得出就拿它当上一版的记录：同实例、同 checkpoint_txg 的记录里带「本次发布末条」标志的那一条
    // （D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定：「那条」按末条标志认；一次发布切成多条记录时它是那次发布的末条）。
    // 读不出（两份都撕了，或那次发布读得出的几条都不带标志）就拿环里最大 jsn 那条顶着——本实例的第一条反向链恒 0、
    // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录（只做过 mkfs 的池环里一条都没有）。
    let own_record = records
        .values()
        .filter(|record| {
            record.instance == effective_root.instance
                && record.checkpoint_txg == effective_root.checkpoint_txg
                && record.place_in_publish
                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned();
    let selected_version_journal_position =
        journal_position_of_the_selected_version(&effective_root, own_record.as_ref());
    let previous = rebuild_previous_version(devices, &effective_root, own_record)?;
    // 环里没有记录时从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条：计数器全池接着走）。
    let next_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0)
        + 1;
    let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records);
    let tree_identifier_watermark_of_the_ring = tree_identifier_watermark_of_the_ring(
        devices,
        &system_configuration,
        &records,
        &effective_root,
    );
    let rebuilt = rebuilt_allocator(
        devices,
        &system_configuration,
        &previous,
        first_txg,
        &|_| false,
        ShadowLedger::On,
    );
    let RebuiltAllocator {
        allocator,
        effective_floor,
        abandoned_roots_unreadable,
    } = rebuilt?;
    let previous_row = PreviousInstanceRow {
        instance: effective_root.instance,
        selected_root_txg: effective_root.checkpoint_txg,
        applied_transaction_high_water: journal.maximum_applied_transaction,
        is_rollback: false,
    };
    let rows_of_the_chosen_roots_instance_table =
        previous.instance_table_chain().records.rows.clone();
    establish_instance(
        parameters,
        devices,
        allocator,
        InstanceStart {
            chosen_root,
            effective_root,
            journal,
            previous,
            previous_row,
            first_txg,
            next_counter,
            tree_identifier_watermark_of_the_ring,
            effective_floor,
            abandoned_roots_unreadable,
            shadow_ledger: ShadowLedger::On,
            system_configuration,
            rows_of_the_chosen_roots_instance_table,
            space_admission,
            selected_version_journal_position,
        },
    )
}

/// 管理员回退（D23（journal 的角色与格式） 已定项 14 的显式例外）：带外选一条回退候选集里的旧根 R_old，不施加它之后的任何记录，
/// 取新实例代号，在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)，回退行、这次发布的单元与
/// 第一个新根同一次发布；只被被抛弃根引用的槽由影子账隔离（D28（挂载期承诺量） 已定项 1 第九项）；之后暖机同可写挂载。
/// 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 取 P2，预想、等用户定）。
///
/// # Errors
/// 交进来的盘数低于 w 的下限（同可写挂载，`WritableDeviceCountBelowTheStripeWidthLowerBound`，在读任何一块盘之前）、
/// 目标根不在根环、不在候选集、它自己那条记录读不出、取号之前的空间准入不过（同可写挂载，实例表按 R_old 那一版算）、
/// 取号之前的预演报错、取号之后那一串在分配器的拷贝上取不到落点（同可写挂载）、有盘不带 R_old 那一版（同可写挂载，
/// `WritableMountRefusedByDevicesWithoutTheSelectedVersion`）、取号或发布失败。
pub fn mount_rollback<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    target: RollbackTarget,
    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
    mount_rollback_with_space_admission(
        parameters,
        devices,
        target,
        shadow_ledger,
        SpaceAdmission::JudgedByTheFormula,
    )
}

/// 同 [`mount_rollback`]，判不判空间准入由调用方给（只供测试的开关 `SpaceAdmission`，同 [`mount_writable_with_space_admission`]）。
///
/// # Errors
/// 同 [`mount_rollback`]；`SkippedByTheTestOnlySwitch` 时不报 `SpaceAdmissionRefusedBeforeAcquisition`。
pub fn mount_rollback_with_space_admission<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    target: RollbackTarget,
    shadow_ledger: ShadowLedger,
    space_admission: SpaceAdmission,
) -> Result<Mounted, MountError> {
    admit_the_writable_device_count(DeviceCount(devices.len()))?;
    let system_configuration = choose_system_configuration(&*devices)?;
    let newest_root =
        choose_root(&*devices, &system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(&*devices, &system_configuration);
    let roots = readable_roots(
        &*devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let target_root = roots
        .iter()
        .find(|root| {
            root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg
        })
        .copied()
        .ok_or(MountError::RollbackTargetNotACandidate {
            target,
            exclusion: RollbackCandidateExclusion::NotInRing,
        })?;
    // 候选集：按最新根指着的实例表判仍然有效——(i, T) 可选 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti；且 txg ≥ F_生效。
    let newest_table = instance_table_of_root(&*devices, &newest_root)
        .ok_or(MountError::InstanceTableMalformed)?;
    // txg ≥ F_生效（各幸存盘所带 F 最大值的最小值），不是最新根自己带的 F（步 4 / 步 5 代码三方第一轮正推腿判「窄化」）。
    let effective_floor = effective_rollback_floor(
        &*devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    if target.checkpoint_txg < effective_floor {
        return Err(MountError::RollbackTargetNotACandidate {
            target,
            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
        });
    }
    // 被回退见证表抛弃的根同样在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14「回退见证」）：最新根落回 R_old 时
    // （回退实例的根都读不出）它那一版的实例表里没有回退行，只按实例表判会把被抛弃的根放回候选集。
    if newest_table
        .rows
        .iter()
        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)
        || rollback_witness_of_the_pool(&*devices, &system_configuration)
            .abandons(target.instance, target.checkpoint_txg)
    {
        return Err(MountError::RollbackTargetNotACandidate {
            target,
            exclusion: RollbackCandidateExclusion::OnAbandonedTimeline,
        });
    }
    // 回退到树表 0 条的根（例如第一个事务里 txg 1、2 的暖机根）：候选集只有上面那三条（D23（journal 的角色与格式） 已定项 14），
    // 「树表 0 条」不在其内，照常回退——回退行走写行那条只重写实例表的发布路径
    // （`publish_instance_table_on_version_without_file`；C493（回退候选集条文与实现说反话） 由此还清）。
    // 环里还留着带文件版本的根时同样照常回退（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步）：回退行那次发布带环里的
    // 树 ID 水位 max（`tree_identifier_watermark_of_the_ring`），之后再发第一个文件版本从它往上发号，不重发被抛弃的根用过的号；
    // I-9.14（树表条目的诞生 txg 跨根不变） 只比同一条时间线上的根，被这次回退切掉的那几条不与新线比。
    let target_has_no_file = tree_table_has_no_entries(&*devices, &target_root)?;
    // R_old 自己那条记录读得出就拿它当上一版的记录（只有 jsn 会被用到），读不出就拿最大 jsn 那条顶着，与可写挂载同一条路。
    // 树表 0 条的目标用不到记录（只做过 mkfs 的池环里一条都没有）：那一格一条都拿不出来也照走，与 `mount_writable` 同一条读法；
    // 「一条记录都拿不出来」只有带文件版本的目标才是错（从盘上重建上一版要一条记录顶着）。
    let own_record = records
        .values()
        .filter(|record| {
            record.instance == target.instance && record.checkpoint_txg == target.checkpoint_txg
        })
        .max_by_key(|record| record.counter)
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned();
    let own_record = match (own_record, target_has_no_file) {
        (Some(record), _) => Some(record),
        (None, true) => None,
        (None, false) => return Err(MountError::FileVersionWithoutAnyJournalRecord),
    };
    // 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 的 P2；预想、等用户定）：
    // 接 R_old 那条之后（P1）会把「B 的记录已提交、根还没落盘」那个窗口里 B 的记录盖掉，崩在回退生效之前那次恢复就不再
    // 「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中）。被抛弃的记录原样留在盘上，靠候选集与实例表挡。
    let highest_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0);
    let next_counter = highest_counter + 1;
    let selected_version_journal_position =
        journal_position_of_the_selected_version(&target_root, own_record.as_ref());
    let previous = rebuild_previous_version(devices, &target_root, own_record)?;
    // 不施加 R_old 之后的任何记录：扫描报告只记环里有多少条自证过的。
    let journal = JournalScanReport {
        valid_records: records.len(),
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: 0,
    };
    let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records);
    // 回退到的那一版自己带的水位可以低于环里被这次回退抛弃的根带的（回退到树表 0 条的暖机根，而环里还留着带文件版本的根）：
    // 那些根发过的号不许再发（D8（核心索引结构） 已定项 8 ②），回退行那次发布带的是环里的 max，
    // 之后再发第一个文件版本就从它往上发（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步）。
    let tree_identifier_watermark_of_the_ring = tree_identifier_watermark_of_the_ring(
        devices,
        &system_configuration,
        &records,
        &target_root,
    );
    // 影子账：这次回退新抛弃的根 = 根环里 (txg, 实例) 大于 R_old 的每一条可读根；只被它们引用的槽隔离，
    // 连同按实例表早已被抛弃的根一起在重建里算。
    let newly_abandoned = |root: &RootRecord| {
        (root.checkpoint_txg, root.instance) > (target.checkpoint_txg, target.instance)
    };
    let RebuiltAllocator {
        allocator,
        effective_floor: _,
        abandoned_roots_unreadable,
    } = rebuilt_allocator(
        devices,
        &system_configuration,
        &previous,
        first_txg,
        &newly_abandoned,
        shadow_ledger,
    )?;
    let previous_row = PreviousInstanceRow {
        instance: target.instance,
        selected_root_txg: target.checkpoint_txg,
        applied_transaction_high_water: 0,
        is_rollback: true,
    };
    establish_instance(
        parameters,
        devices,
        allocator,
        InstanceStart {
            chosen_root: newest_root,
            effective_root: target_root,
            journal,
            previous,
            previous_row,
            first_txg,
            next_counter,
            tree_identifier_watermark_of_the_ring,
            effective_floor,
            abandoned_roots_unreadable,
            shadow_ledger,
            system_configuration,
            rows_of_the_chosen_roots_instance_table: newest_table.rows,
            space_admission,
            selected_version_journal_position,
        },
    )
}

#[cfg(test)]
mod non_empty_root_tests {
    use super::user_visible_trees_changed;
    use crate::recovery::UserVisibleTreeRootPointers;

    fn pointers(inode_tree: u8, extent_tree: u8) -> UserVisibleTreeRootPointers {
        UserVisibleTreeRootPointers {
            inode_tree: Some(vec![inode_tree; 86]),
            extent_tree: Some(vec![extent_tree; 86]),
        }
    }

    /// 两棵树任一棵的根指针变了就算非空：只换 extent 树（同一个 inode 的内容覆盖写、inode 记录原地不动的那种发布）与只换 inode 树都要认出来，
    /// 两棵都没变的（空发布、回退那次照抄上一版的）不算；最旧的有效根与「树表里没有这两棵树的条目」比。
    #[test]
    fn root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root(
    ) {
        assert!(
            user_visible_trees_changed(&pointers(1, 2), &pointers(1, 3)),
            "只有 extent 树的根指针变了"
        );
        assert!(
            user_visible_trees_changed(&pointers(4, 2), &pointers(1, 2)),
            "只有 inode 树的根指针变了"
        );
        assert!(
            !user_visible_trees_changed(&pointers(1, 2), &pointers(1, 2)),
            "两棵树都照抄：空发布"
        );
        assert!(
            user_visible_trees_changed(&pointers(1, 2), &UserVisibleTreeRootPointers::ABSENT),
            "最旧的有效根下面有文件"
        );
        assert!(
            !user_visible_trees_changed(
                &UserVisibleTreeRootPointers::ABSENT,
                &UserVisibleTreeRootPointers::ABSENT
            ),
            "mkfs 的第 0 代根与暖机根：树表里没有这两棵树"
        );
    }
}

#[cfg(test)]
mod reclaim_floor_tests {
    use super::reclaim_floor;
    use crate::address::CheckpointTxg;

    /// 第 0 代根还在环里时门槛就是 F_生效；被盖之后环里最旧的有效根接管（比 F 大时取它）。
    #[test]
    fn reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root() {
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(0))),
            CheckpointTxg(11)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(0), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(reclaim_floor(CheckpointTxg(11), None), CheckpointTxg(11));
    }
}
