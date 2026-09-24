//! 可写挂载（里程碑「第二个事务」步 3）：进程重开镜像之后先走一次恢复，从盘上重建可写态——所选根、施加前缀之后的根、
//! 上一版全部角色的单元与指针、分配器、下一条 jsn——再取实例代号、给上一个实例写行（D18（块里携带什么信息） 已定项 11）、
//! 推空发布直到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊），之后本实例的发布才接在后面。
//! 第一版没有干净关闭标记，重开一律走恢复。

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use crate::allocator::{
    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PoolAllocator, ReclaimedReuse,
    RootRingOccupancy, RootRingOccupant,
};
use crate::block_device::BlockDevice;
use crate::journal::back_chain_of;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
use crate::recovery::{
    allocation_records_of_version_without_file, allocation_records_under_root, choose_root,
    choose_system_configuration, effective_rollback_floor, highest_root_txg,
    highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
    instance_table_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
    readable_roots_with_ring_slots, rebuild_version, replay_journal, rollback_high_water_of_root,
    scan_journal, tree_table_has_no_entries, user_visible_tree_root_pointers, InstanceTableChain,
    JournalScanReport, RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
    UserVisibleTreeRootPointers,
};
use crate::root_record::RootRecord;
use crate::root_ring::{target_for_publish, RootRingSlot};
use crate::transaction::{
    acquire_expected_instance, instance_generation_to_acquire,
    publish_instance_table_on_version_without_file, publish_sequence_admission, publish_version,
    publish_without_units, AcquisitionFailed, ExpectedInstanceAcquisitionFailed,
    InstanceTableOnlyPublishPlan, InstanceTablePlan, PoolVersion, PoolWriter, PublishError,
    PublishPlan, PublishShape, TransactionOutput, TransactionUnit, ZeroUnitPublishPlan,
};
use crate::write_accounting::WritesByStructureKind;
use singlefs_format::{INSTANCE_TABLE_PAGE_RECORDS, ROOT_RING_REGIONS};
use std::collections::{BTreeMap, BTreeSet};

use crate::instance_table::instance_table_pages_for_rows;
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
    /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成：错，连同这次挂载的写入口交得出的写账（[`PublishAfterAcquisitionFailed`]）。
    Publish(PublishAfterAcquisitionFailed),
    /// 抬 F 那一串空发布（D16（发布语义） 已定项 1：推到每块盘上都有一条带新 F 的根才生效）里有一次发不出去：
    /// 前面 `publishes_persisted` 次已经落盘——带新 F 的根在盘上、调用方的现行版本已经是最后落盘的那一版，F 还没在每块盘上生效，
    /// 抬 F 回收的槽照旧扣着；`cause` 是第 `publishes_persisted + 1` 次（从 1 数）的错。`cause` 自己说的「在任何写之前」
    /// （例如 `PublishError::PlacementRefused`）只对出错的这一次成立，不对整串成立（C516（抬 F 那一串发布被拒时前面几次已落盘））。
    RaiseFloorSequencePublishFailed {
        publishes_persisted: usize,
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
    /// 写行那次发布的准入（这次之后的分配记录条数、这次要写的记账行数）算不过：在**取号之前**拒绝，盘上一个字节都不动、
    /// 两块盘系统配置里的实例代号不动（增补 2 第 20a 行，代码三方第一轮打中）。改之前这两条只在发布路径里算，
    /// 取号（两次系统配置槽写 + 一道屏障）已经写完才报出来，而取号的回卷只管取号自己那几次写报错、管不到之后的发布失败 ⇒
    /// 分配记录树满了的池此后每试一次可写挂载就再烧一个实例代号。
    RowPublishAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        cause: PublishError,
    },
    /// 暖机那几次空发布（写行之后推到本实例的根覆盖每块盘，D16（发布语义） 已定项 8 戊）里第几次的准入算不过：连写行那次一起
    /// 在取号之前算，算不过在任何写之前返回（增补 2 第 20a 行，2026-09-18 用户定案）。改之前取号之前只算写行那一次，
    /// 「写行装得下、暖机第 N 次装不下」的池是取号写完、写行也发完，暖机才报错——实例代号照样烧掉。
    WarmUpAdmissionRefusedBeforeAcquisition {
        instance_to_acquire: InstanceGeneration,
        /// 第几次暖机空发布算不过，从 1 数。
        warm_up_publish_index: usize,
        /// 这次挂载要推几次暖机空发布（按根环落点公式现算）。
        warm_up_publishes_planned: usize,
        cause: PublishError,
    },
    /// 写行那次发布要写（或要 COW 重写）一条多于一片的实例表链，而多于一片的写法有两处条款没写：
    /// ① 各片在提交内生块的 bump 次序里怎么排——D3（空间分配） 已定项 10 ⑤ 只写「实例表单元最前」，树 ID 升序、先叶后根、
    /// 同层按 key 升序那几条管的是「其余」；这个次序同时是各片的出生序号（D19（块指针的结构与宽度预算） 已定项 9），第二片的落点、
    /// 它头里与第一片链指针里的出生序号都随它定；② 行怎么分到各片（每片数据行 369 是上限，写满一片再开下一片没有逐字条款）。
    /// 两处定之前第一版不写第二片；在**取号之前**拒绝，盘上一个字节都不动、两块盘系统配置里的实例代号不动
    /// （改之前取号写完才在装实例表单元时越界 panic，池此后每试一次可写挂载就再烧一个实例代号，代码三方第二轮 Z1-a）。
    /// 两种情形都拒：这次之后要多于一片（`pages_after_this_publish` > 1，这一版的行数 + 这次要写的行数 + 1 > 370）；
    /// 这一版的表已经多于一片（`pages_in_version` > 1，只有别的实现写的或坏的镜像上有——把它整条重写成一片要逐片释放旧链，
    /// 这一格条款写了，第一版不写多片，也就不重写多片，一并拒）。
    InstanceTableChainLongerThanOnePageUndecided {
        instance_to_acquire: InstanceGeneration,
        rows_in_version: usize,
        pages_in_version: usize,
        rows_to_write: usize,
        pages_after_this_publish: usize,
    },
}

/// 可写挂载（或回退）取号之后那一串发布——写行一次、暖机推到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊）——里有一次没做成。
/// 这次挂载的写入口是挂载自己开的、随错一起丢掉，所以它交得出的写账都在这里：已经落盘的那几次发布各自的写，
/// 与失败那一次落盘阶段已记的写（增补 2 收口表第 58 行：不交出来，设备一层数到的写与程序交得出的账对不上）。
/// 取号那两次系统配置槽写不是发布，不在其内（取号的回卷另由 `AcquisitionFailed` 报）。
#[derive(Debug)]
pub struct PublishAfterAcquisitionFailed {
    pub cause: PublishError,
    /// 失败之前这次挂载里已经落盘的发布各自的写，按先后（写行那次在最前）；写行那次就失败时是空的。
    pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
    /// 这次挂载的写入口的失败账（`PoolWriter::writes_of_failed_publishes` 原样）：落盘阶段失败的那一次一份；
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
    /// 最新根指着的实例表里有那个实例的行 (i, Ti, Wi) 且目标的 txg > Ti：被抛弃时间线上的根（D23（journal 的角色与格式） 已定项 14
    /// 「按实例表判仍然有效」）。
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
    /// 这一版的实例表的全部行（写行时要在它后面接上这次的行，取号之前的准入也按它的行数算）。两臂都有表：
    /// 树表 0 条那一版的表是根记录直接指着的那条链（mkfs 种下的，或上一次挂载写行重写的）。
    fn instance_table(&self) -> &InstanceTableRecords {
        match self {
            PreviousVersion::WithoutFile { table, .. }
            | PreviousVersion::WithFile { table, .. } => &table.records,
        }
    }

    /// 这一版的实例表链有几片（取号之前的准入按它判：多于一片的链第一版不重写）。
    fn instance_table_pages(&self) -> usize {
        match self {
            PreviousVersion::WithoutFile { table, .. }
            | PreviousVersion::WithFile { table, .. } => table.page_pointers.len(),
        }
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
            // 写过行的一版那片分配记录树节点同样是这条根引用的单元（D23（journal 的角色与格式） 已定项 14：被抛弃时间线的根离开根环之前，
            // 它们引用的单元不许重新分配）；漏了它，被抛弃根的这一片既不隔离、也不在回退目标那一版的账里，回退之后是空闲槽。
            // mkfs 的第 0 代与照抄它的暖机根那一项全零，没有这一片。
            let allocation_record_node = (root.allocation_record_tree_root
                != crate::pointer::NodePointer::empty_root())
            .then_some((
                &root.allocation_record_tree_root,
                TransactionUnit::AllocationTree,
            ));
            for (pointer, unit) in instance_table_pages
                .iter()
                .map(|page_pointer| (page_pointer, TransactionUnit::InstanceTable))
                .chain([(&root.tree_table, TransactionUnit::TreeTable)])
                .chain(allocation_record_node)
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
    let is_abandoned = |root: &RootRecord| {
        extra_abandoned(root)
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
/// - 根记录那一项（分配记录树根指针）不是全零 ⇒ 这一版写过行，写行那次发布把这一版的全部分配记录写成了一个节点
///   （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案）：从它重建，被换下的那一片实例表因此带着已释放标志回来，
///   不会被当空闲槽发出去。那一片分配记录树节点自己的落点记进分配器——下一次发布要换下它，而这一版没有上一版的内存态可查。
/// - 全零 ⇒ mkfs 的第 0 代（或照抄它的暖机根）：账由实例表与树表两条指针直接算（`format_time_allocator`）。
///
/// # Errors
/// 分配记录树读不出、解不开、不止一层、条目宽或结构值判红 ⇒ `Recovery(...)`；
/// 全零那一条上两条指针任一条不是 mkfs 写的那一版 ⇒ `VersionWithoutFileNotWrittenByMakeFilesystem`（见 `format_time_allocator`）。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_of_version_without_file 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn allocator_of_version_without_file<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    let Some(records) =
        allocation_records_of_version_without_file(devices, root).map_err(MountError::Recovery)?
    else {
        return format_time_allocator(device_maps, root);
    };
    let node_placement = Placement {
        slot: slot_shared_by_both_location_entries(&root.allocation_record_tree_root.locations)
            .map_err(
                |disagreement| MountError::FormatTimeUnitLocationsOnDifferentSlots {
                    unit: TransactionUnit::AllocationTree,
                    disagreement,
                },
            )?,
        span: TransactionUnit::AllocationTree.span_slots(),
    };
    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, records);
    allocator.note_allocation_record_node_of_the_version_without_file(node_placement);
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
/// 那一串空发布里有一次发不出去（`RaiseFloorSequencePublishFailed`，带着前面已经落盘了几次）。
pub fn raise_rollback_floor<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
    shadow_ledger: ShadowLedger,
) -> Result<RaisedFloor, MountError> {
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
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publishes = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let publishes_persisted = publishes.len();
        let next = publish_version(
            &mut pool,
            allocator,
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
            },
            Some(&*current),
        )
        .map_err(|cause| MountError::RaiseFloorSequencePublishFailed {
            publishes_persisted,
            cause,
        })?;
        let device = device_of_txg(next.root.checkpoint_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        publishes.push(next.clone());
        *current = next;
    }
    allocator.release_reclaim_holds();
    Ok(RaisedFloor {
        ceiling,
        publishes,
        reclaimed,
        abandoned_roots_unreadable,
    })
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

/// 写行那次发布、上一版带文件：在上一版那张实例表后面接上这次写的行，重写实例表与四个固定点单元（D18（块里携带什么信息） 已定项 11；
/// 记账树已经存在 ⇒ 空发布也重写固定点单元，D16（发布语义） 已定项 9）。事务号 0、本实例第一条反向链 0。
fn publish_rows_on_file_version<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    table: &InstanceTableRecords,
    rows_written: &[InstanceRow],
    identity: RowPublishIdentity,
) -> Result<TransactionOutput, PublishError> {
    let mut table_after = table.clone();
    table_after.rows.extend_from_slice(rows_written);
    publish_version(
        pool,
        allocator,
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
            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: identity.tree_identifier_watermark,
            rollback_floor: identity.rollback_floor,
        },
        Some(previous),
    )
}

/// 暖机的一次空发布，接在现行那一版后面：带文件的一版重写四个固定点单元（记账树存在，D16（发布语义） 已定项 9），
/// 树表 0 条的一版写零个单元（同一条已定项「树表 0 条 ⇒ 零单元」）。
fn publish_empty_after<Device: BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut PoolAllocator,
    current: &PoolVersion,
    instance: InstanceGeneration,
) -> Result<PoolVersion, PublishError> {
    match current {
        PoolVersion::WithFile(current_file_version) => {
            let plan = PublishPlan {
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
            };
            // 取号之前那道准入按 `PublishShape::EMPTY_PUBLISH` 算这一次要加几条分配记录；这里钉住它算的就是这张计划的形状。
            // 形状要先把计划算成「这次之后 inode 树是什么样、重写哪些角色」才知道（`PublishPlan::resolve`），
            // 而算它只读、不发写：算不过就在这里交回，盘上逐字节不变。
            let resolved_empty_publish = plan.resolve(
                Some(current_file_version),
                &current_file_version.tree_identifiers,
            )?;
            assert_eq!(
                resolved_empty_publish.shape(),
                PublishShape::EMPTY_PUBLISH,
                "暖机这次空发布的形状与取号之前算准入用的那一个相同（不写文件内容、不碰 inode 树、实例表照抄）"
            );
            // 映射条目那一条准入在取号之前按上一版的叶容器数算过：这次不碰 inode 树 ⇒ 算出来的树与上一版逐片相同。
            assert_eq!(
                resolved_empty_publish.inode_tree.containers.len(),
                current_file_version.inode_leaf_containers.len(),
                "暖机这次空发布之后的叶容器数与取号之前算映射条目准入用的那一个相同"
            );
            publish_version(pool, allocator, plan, Some(current_file_version))
                .map(PoolVersion::WithFile)
        }
        PoolVersion::WithoutFile(current_version_without_file) => {
            let published = publish_without_units(
                pool,
                &current_version_without_file.root,
                ZeroUnitPublishPlan {
                    txg: CheckpointTxg(current_version_without_file.root.checkpoint_txg.0 + 1),
                    counter: current_version_without_file.record.counter + 1,
                    instance,
                    back_chain: back_chain_of(&current_version_without_file.record_bytes),
                    rollback_floor: current_version_without_file.root.rollback_floor,
                    // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
                    tree_identifier_watermark: current_version_without_file
                        .root
                        .tree_identifier_watermark,
                },
            )
            .map_err(PublishError::from)?;
            // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`）。
            allocator.record_root_written_by_this_process(published.root.checkpoint_txg);
            Ok(PoolVersion::WithoutFile(published))
        }
    }
}

/// 这次挂载在取号之后要发的那几次——写行一次、暖机 `warm_up_publishes_planned` 次——的准入，在取号之前一串算完
/// （增补 2 第 20a 行：写行那一半是代码三方第一轮打中的，暖机那一半是 2026-09-18 用户定案）：分配记录树、记账树与中央映射树
/// 在每一次之后都要装得下，前面几次要新增的分配记录算进后面几次的基数；算不过就在任何写之前返回——取号写出去的实例代号一去不回，
/// 回卷只管取号自己那几次写报错，管不到取号之后的发布失败。
/// 读的是内存里这个分配器，与发布路径那一遍同一份输入（取号不碰它），发布路径那一遍仍在、是动分配器之前的最后一道。
/// 再加一项实例表（代码三方第二轮 Z1-a），按片数判：写行那次发布重写的实例表 = 这一版的行 + `rows_to_write` 行，每片末尾一条链指针记录
/// （`INSTANCE_TABLE_PAGE_RECORDS`，D18（块里携带什么信息） 已定项 11）；这一版的链已经多于一片、或这次之后要多于一片，都拒
/// （多于一片怎么写没有条款，`InstanceTableChainLongerThanOnePageUndecided`）。行数与片数读的是 `start.previous` 里那一版实例表
/// （从根记录沿链读的整条链）、要写的行由调用方按这次要取的号列出来——写行那次发布拼的正是这两样（`publish_rows_on_file_version`
/// 在那一版后面接上这几行），号在取号写之前重算、不等就不写（`acquire_expected_instance`），所以这里判的与写出去的是同一张表。
/// 可写挂载与回退各按自己的 `start` 算：回退的那一版是 R_old 指着的表、要写的是 [max(r_old, 1), 新实例)。
/// **实例表那一项两臂都判**：树表 0 条的一版上写行同样重写整张实例表（`publish_instance_table_on_version_without_file`），
/// 多于一片同样走不下去；那一版上要写的行数没有上界——每次挂载崩在取号之后、写行之前，下一次要写的区间就多一个实例。
/// 分配记录树、记账树与中央映射树那三条只在带文件的一版上判：树表 0 条 ⇒ 这一版没有这三棵树，写行与暖机都不写它们
/// （D16（发布语义） 已定项 9 那五样一样都不写）。
fn refuse_publishes_before_acquisition_that_do_not_pass_admission(
    allocator: &PoolAllocator,
    start: &InstanceStart,
    instance_to_acquire: InstanceGeneration,
    rows_to_write: usize,
    warm_up_publishes_planned: usize,
) -> Result<(), MountError> {
    if let PreviousVersion::WithFile { output, .. } = &start.previous {
        let shapes: Vec<PublishShape> = std::iter::once(PublishShape::ROW_PUBLISH)
            .chain(std::iter::repeat_n(
                PublishShape::EMPTY_PUBLISH,
                warm_up_publishes_planned,
            ))
            .collect();
        // 写行与暖机都不碰 inode 树、不写文件内容 ⇒ 这一串里每次发布之后的叶容器数与数据单元数都是上一版那两个数，
        // 映射条目那一条准入按它们算。
        let inode_leaf_containers = output.inode_leaf_containers.len();
        let data_units = output.data_pointers.len();
        publish_sequence_admission(allocator, &shapes, inode_leaf_containers, data_units).map_err(
            |refusal| match refusal.publish_index {
                0 => MountError::RowPublishAdmissionRefusedBeforeAcquisition {
                    instance_to_acquire,
                    cause: refusal.cause,
                },
                warm_up_publish_index => MountError::WarmUpAdmissionRefusedBeforeAcquisition {
                    instance_to_acquire,
                    warm_up_publish_index,
                    warm_up_publishes_planned,
                    cause: refusal.cause,
                },
            },
        )?;
    }
    // 实例表按片数算（D18（块里携带什么信息） 已定项 11：一片 370 条，链指针记录恒为最后一条）。多于一片的链怎么写有两处条款没写
    // （`InstanceTableChainLongerThanOnePageUndecided` 的文档注释），第一版只写一片：这一版的链已经多于一片、或这次之后要多于一片，
    // 都在这里拒。只有一片时写行那次发布只重写一个实例表单元，上面那串准入按一个角色数，与发布路径一致。
    let records_per_page = usize::try_from(INSTANCE_TABLE_PAGE_RECORDS).expect("370");
    let rows_in_version = start.previous.instance_table().rows.len();
    let pages_in_version = start.previous.instance_table_pages();
    let chain_longer_than_one_page = || MountError::InstanceTableChainLongerThanOnePageUndecided {
        instance_to_acquire,
        rows_in_version,
        pages_in_version,
        rows_to_write,
        pages_after_this_publish: instance_table_pages_for_rows(rows_in_version + rows_to_write),
    };
    if pages_in_version > 1 {
        return Err(chain_longer_than_one_page());
    }
    // 这次之后多于一片 ⟺ 行数 + 链指针记录 > 一片的记录数（`instance_table_pages_for_rows` 的 ⌈行数 / 369⌉ > 1 同一个判定）。
    if rows_in_version + rows_to_write + 1 > records_per_page {
        return Err(chain_longer_than_one_page());
    }
    Ok(())
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

/// 取号之后的一次发布失败了：错，连同这次挂载的写入口交得出的账——`persisted` 是这次挂载里已经落盘的那几次（按先后），
/// 失败那一次落盘阶段已记的写从写入口的失败账取。
fn publish_failed_after_acquisition<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    persisted: &[&PoolVersion],
    cause: PublishError,
) -> MountError {
    MountError::Publish(PublishAfterAcquisitionFailed {
        cause,
        writes_of_persisted_publishes: persisted
            .iter()
            .map(|version| match version {
                PoolVersion::WithoutFile(version_without_file) => {
                    version_without_file.writes.clone()
                }
                PoolVersion::WithFile(file_version) => file_version.writes.clone(),
            })
            .collect(),
        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
    })
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
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    // 暖机推哪几次，取号之前先算出来：准入按这一份算，取号之后的循环也按这一份推。
    let warm_up_publishes_planned = warm_up_publish_txgs(parameters, &all_devices, start.first_txg);
    let instance_to_acquire = instance_generation_to_acquire(&pool);
    // 要写的行按要取的号先列出来：实例表那一项准入按它算，取号之后写行也用这一份（号在写之前重算、不等就不写）。
    let rows_written = instance_rows_to_write(&start.previous_row, instance_to_acquire);
    refuse_publishes_before_acquisition_that_do_not_pass_admission(
        &allocator,
        &start,
        instance_to_acquire,
        rows_written.len(),
        warm_up_publishes_planned.len(),
    )?;
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

    // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishAfterAcquisitionFailed`）：写入口随错一起丢掉。
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, table } => {
            let published = publish_rows_on_file_version(
                &mut pool,
                &mut allocator,
                output,
                &table.records,
                &rows_written,
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
        PreviousVersion::WithoutFile { root, table } => {
            let mut table_after = table.records.clone();
            table_after.rows.extend_from_slice(&rows_written);
            let instance_table_records = table_after.to_records();
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
                    instance_table_records: &instance_table_records,
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
        },
        allocator,
        current,
    })
}

/// 可写挂载：恢复 → 重建上一版与分配器 → 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘。只做过 mkfs 的池（所选根的树表 0 条、
/// 环里没有记录）同样走这条路：取号 1、不写行、零单元的发布推到本实例的根覆盖每块盘，第一个文件版本接在 `current` 后面。
///
/// # Errors
/// 恢复失败、所选根下有文件而环里一条记录都没有、实例表沿链读不出或解不开、取号之前的准入不过（分配记录树、记账树、
/// 实例表多于一片）、取号失败、发布失败。
pub fn mount_writable<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Result<Mounted, MountError> {
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
    );
    // 所选根覆盖的最后一条记录读得出就拿它当上一版的记录：同实例、同 checkpoint_txg 的记录里 jsn 最大的那条
    // （D23（journal 的角色与格式） 已定项 14 注 1，P6 2026-09-23 定；一次发布切成多条记录时那次发布的末条）。
    // 读不出（两份都撕了）就拿最大 jsn 那条顶着——本实例的第一条反向链恒 0、
    // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录（只做过 mkfs 的池环里一条都没有）。
    let own_record = records
        .values()
        .filter(|record| {
            record.instance == effective_root.instance
                && record.checkpoint_txg == effective_root.checkpoint_txg
        })
        .max_by_key(|record| record.counter)
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned();
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
        },
    )
}

/// 管理员回退（D23（journal 的角色与格式） 已定项 14 的显式例外）：带外选一条回退候选集里的旧根 R_old，不施加它之后的任何记录，
/// 取新实例代号，在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)，回退行、这次发布的单元与
/// 第一个新根同一次发布；只被被抛弃根引用的槽由影子账隔离（D28（挂载期承诺量） 已定项 1 第九项）；之后暖机同可写挂载。
/// 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 取 P2，预想、等用户定）。
///
/// # Errors
/// 目标根不在根环、不在候选集、它自己那条记录读不出、取号之前的准入不过（同可写挂载，实例表按 R_old 那一版算）、取号或发布失败。
pub fn mount_rollback<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    target: RollbackTarget,
    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
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
    if newest_table
        .rows
        .iter()
        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)
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
