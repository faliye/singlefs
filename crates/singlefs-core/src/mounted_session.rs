//! 挂着的会话：一次可写挂载之后接着发布的那一层（D16（发布语义） 已定项 1「准入」那一行的发布那一处，
//! C283（准入失败时不先推发布就报 ENOSPC））。它持着这次挂载的分配器、现行那一版、实例代号与影子账那一臂，用户的改动经它发布：
//! 发布路径报空间准入不够（`PublishError::SpaceAdmissionRefused`）、或准入放行而落点取不到（`PublishError::PlacementRefused`）时，
//! 先推一串抬 F 的空发布（`mount::push_one_floor_raise_within_the_admission_budget`：F 抬到准入上限，一次准入最多
//! `mount::PUBLISHES_PER_ADMISSION_AT_MOST` 次发布，算上这次用户发布），再重判；做满仍不够、或抬不动了，才报空间不够。
//! 写行那次发布之前不推：会话在可写挂载交回之后才有，写行那次早已发完（可写挂载那一处的推归 `mount::establish_instance`）。
//!
//! 为什么要这一层：发布路径（`transaction::publish_overwrite` 这一族）只拿到一个写入口（`PoolWriter`），拿不到抬 F 要的设备表与参数
//! （抬 F 那一串自己开写入口、读根环与系统配置，`mount::raise_rollback_floor`）——安全设计轮第三轮辩方报的架构缺口
//! （`research/prompts/m2-safety-r3-sonnet-output.md` 第二节）。这一层怎么放是实现员提的（交主 agent 定）：它持着挂载认下的池
//! （`mount::ParametersAndDeviceTableOfTheMount`：盘上择到的系统配置里的参数、挂载时核过的盘表），写入口只按那份参数建、不收调用方的参数；
//! 盘本身不持，每次调用由调用方把盘表交进来——用例在两次调用之间照样摸得到盘（拷镜像、注入故障）——交进来的盘表与挂载时的逐项比，
//! 对不上在任何读写之前拒；身份对得上，还要每块盘「可见」（两个系统配置槽里至少一份本池自证过的），
//! 有一块不可见就在任何写之前拒（代码三方 m2-closeout-code-r1 Z3-A 乙：只比身份时，换上去的一块空盘照样收到本池的写）；
//! 「可见」之后再核每块盘自证槽里的本盘设备号是盘表给它的身份、这块盘不落后于现行那一版又缺它的单元或它的根（D18（块里携带什么信息） 已定项 11
//! 「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定；代码三方 m2-closeout-code-r2「盘体对调的盘表」「同池旧快照的盘经入口洗白」；
//! 缺根那一半是 C585（会话发布放进发布途中取下的旧快照） 用户定的改法 C）。
//! 发布路径与抬 F 各自开写入口的做法不变。

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES};

use crate::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
    SlotNumber, TreeIdentifier,
};
use crate::allocator::{AllocationRecord, PlacementOnDevice, PoolAllocator};
use crate::block_device::{
    check_aligned_and_in_range, BlockDevice, BlockDeviceError, DiscardOutcome,
    PhysicalBlockSizeInBytes, WriteDurability,
};
use crate::compaction_plan::{
    choose_region, compaction_starts_after_a_user_change, empty_cluster_segments_by_the_records,
    holes_admit_the_issue, holes_outside_the_region_in_segments_holding_user_data,
    plan_batch_skipping_the_units_this_publish_rewrites_anyway, publish_budget_of_a_compaction_run,
    region_holds_a_quarantined_slot, region_is_empty_by_the_records,
    region_scans_of_the_segments_without_an_unreleased_record, the_completed_intent_yielded,
    CompactionBatchSize, CompactionBatchUnits, CompactionOfThisMount, CompactionRunForcing,
    CompactionRunStop, PlannedBatch, PlannedRelocation, RegionChoice, RegionChoiceCriterion,
    UnitAreaOfOneDevice,
};
use crate::instance_table::InstanceRow;
use crate::instance_table_sweep::unit_head_read_by_sweep;
use crate::logged_operations::{
    live_compaction_intent_of_the_entries, CompactionIntent, CompactionRegion,
};
use crate::make_filesystem::MakeFilesystemParameters;
use crate::mount::{
    devices_behind_the_current_version, devices_without_a_self_verified_system_configuration,
    empty_publish_plan_after_file_version,
    own_device_numbers_on_disk_differing_from_the_identities_handed_in,
    parameters_of_the_pool_agreeing_with_the_caller_inputs,
    push_one_floor_raise_within_the_admission_budget, DeviceBehindTheCurrentVersion,
    FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop, MountError, MountOutput, Mounted,
    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
    RaisedFloor, ShadowLedger,
};
use crate::recovery::{instance_table_chain_of_root, PoolReader, ReadPassOverThePool};
use crate::region_scan::{scan_region, AllocationInRegion, CentralMappingEntry, RegionScan};
use crate::root_record::{RootRecord, UnmountMarker};
use crate::root_ring::{target_for_publish, RootRingSlot};
use crate::transaction::{
    prepare_the_version_publish, publish_compaction_step_in_the_session,
    publish_compaction_step_with_the_region_check_of_the_intent_deletion, publish_create_snapshot,
    publish_new_inodes, publish_overwrite, publish_overwrite_of_an_inode, publish_sequential_write,
    publish_sequential_write_of_an_inode, publish_size_change, publish_snapshot_destruction_intent,
    publish_unlink_intent, publish_version, walk_the_deletion_queue_until_it_is_empty,
    CentralMappingOfAVersionReadLeafByLeaf, CompactionIntentStep, CompactionStepRefused,
    DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup,
    PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter, PublishError, ReadFailedOnTheDisk,
    ReadFailingSelfVerification, RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck,
    RelocationBatchRefused, TransactionOutput, TransactionUnit,
};
use crate::unit::{unit_filesystem_identifier, UNIT_CLASS_DATA};
use crate::unlink::DeletionSwitches;
use singlefs_format::ROOT_RING_REGIONS;

/// 一次可写挂载之后的会话：接下来的发布接在 `current` 后面、用这个分配器与实例代号。
#[derive(Debug)]
pub struct MountedSession {
    pub allocator: PoolAllocator,
    pub current: PoolVersion,
    pub instance: InstanceGeneration,
    /// 这次挂载走的影子账那一臂（`MountOutput::shadow_ledger`）：推抬 F 时照它重算影子账。
    pub shadow_ledger: ShadowLedger,
    /// 这次挂载认下的池（`MountOutput::parameters_and_device_table`；没经过可写挂载的会话读盘造，
    /// `ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`）：写入口只按它的参数建，交进来的盘表与它的逐项比。
    pub parameters_and_device_table: ParametersAndDeviceTableOfTheMount,
    /// 整理在这次挂载里会话这一侧的状态（`compaction_plan::CompactionOfThisMount`：不合格段表、常驻隔离集、`B_gc` 数的发布数、观测；
    /// 丁 1 交判第 2 点）：生命周期一次挂载。可写挂载续做意图时建的那一份接着用（`mount::MountOutput::compaction_continuation`）；
    /// 没经过可写挂载的会话从 `CompactionOfThisMount::of_a_new_mount` 起。活意图不在这里：它在现行那一版的 logged_ops 里。
    pub compaction: CompactionOfThisMount,
}

/// 用户的一次改动，经会话发布（发布路径那三种接在带文件的一版后面的改动）。
#[derive(Clone, Copy, Debug)]
pub enum UserChange<'content> {
    /// 覆盖写一个数据单元以内的内容（`transaction::publish_overwrite`）。
    Overwrite(FirstFile<'content>),
    /// 顺序写整份内容，按切分纪律切成几个一单元事务（`transaction::publish_sequential_write`）。
    SequentialWrite(FirstFile<'content>),
    /// 建 `count` 个空 inode（`transaction::publish_new_inodes`）。
    NewInodes { count: u64, write_time_seconds: u64 },
    /// 删一个 inode（D8（核心索引结构） 已定项 16）：一次调用 = 先把现行那一版的 `deleted_inodes` 走空 + 意图那次发布 + 从尾部全部分批
    /// （`transaction::publish_unlink_intent`、`transaction::publish_unlink_steps_until_the_queue_is_empty`）。
    /// `write_time_seconds` 是意图那次写进这条记录三个时间秒的值，各批照抄它（用户 2026-10-03 定 Q-F 乙）。
    Unlink {
        inode: InodeNumber,
        write_time_seconds: u64,
    },
    /// 建一个只读快照（`transaction::publish_create_snapshot`，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S2）：
    /// 先把现行那一版的 `deleted_inodes` 走空（同别的改动，D8（核心索引结构） 已定项 16 第 6 条），再发一次独立发布，不与别的改动合在一起。
    /// 现行那一版有树而文件集合为空时照常发布（用户 2026-10-16 定「放开」，判决 `research/prompts/m4-segment-code-r1-main-verification.md`
    /// 第三节 F5）：快照抄的是那一刻的两棵空树，读它读到没有文件（`recovery::SnapshotFileReading::NoFile`）。
    CreateSnapshot,
    /// 删一个只读快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S4 与第七节岔路 2、4）：快照按它码 14 条目的树号点名
    /// （岔路 5）。一次调用 = 先把现行那一版的队列走空 + 意图那次发布（`transaction::publish_snapshot_destruction_intent`）+ 全部批
    /// （会话的走空接着走它们，`transaction::walk_the_deletion_queue_until_it_is_empty`），与 unlink 同形、同准入：被空间拒推抬 F 再试，
    /// 不对「先走空」开例外。现行那一版有树而文件集合为空时照样删（删快照不依赖活头里有没有文件：文件删光之后快照照样删得掉，
    /// 丙 2 交判第 5 条，主 agent 定是缺陷；D8（核心索引结构） 已定项 16 第 5 条那一句管的是写文件的改动）。
    DestroySnapshot { snapshot: TreeIdentifier },
    /// 把一个 inode 的 size 改成 `new_size`（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K1、K2、K5、K12，
    /// `transaction::publish_size_change`）：一次调用 = 先把现行那一版的承诺类队列走空 + 这一次发布 + 截断时之后的全部批（会话的走空接着走它们，
    /// 次序 `deleted_inodes` → 销毁中的快照 → 截断，`transaction::walk_the_deletion_queue_until_it_is_empty`），与 unlink 同形：
    /// 空间不够推抬 F 再试。N = S 只改记录、N > S 变长一次发布、N < S 意图 + 各批 + 最后一批。`write_time_seconds` 是写进记录三个时间秒的值。
    Truncate {
        inode: InodeNumber,
        new_size: u64,
        write_time_seconds: u64,
    },
    /// 按 inode 号覆盖写一个数据单元以内的内容（`transaction::publish_overwrite_of_an_inode`，里程碑「删除与多次 COW」一·1，
    /// 用户 2026-10-18 定）：把 `inode` 那个文件整份换成 `content`。`inode` 是第一个文件时同 [`UserChange::Overwrite`]。
    /// `inode` 不在现行那一版的 inode 树里 ⇒ `UserChangeRefused::Publish` 带 `WriteToAnInodeThatIsNotInTheTree`。
    OverwriteOfAnInode {
        inode: InodeNumber,
        content: &'content [u8],
        write_time_seconds: u64,
    },
    /// 按 inode 号顺序写整份内容，按切分纪律切成几个一单元事务（`transaction::publish_sequential_write_of_an_inode`）。
    /// `inode` 是第一个文件时同 [`UserChange::SequentialWrite`]；拒绝同 [`UserChange::OverwriteOfAnInode`]。
    SequentialWriteOfAnInode {
        inode: InodeNumber,
        content: &'content [u8],
        write_time_seconds: u64,
    },
}

/// 一次用户改动发成了：新的一版在会话的 `current` 里。
#[derive(Debug)]
pub struct UserChangePublished {
    /// 这次准入里推的那几串抬 F 的空发布，按先后；一次都没推的是空的。
    pub floor_raises: Vec<RaisedFloor>,
    /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）：`SpaceAdmissionRefused` 或 `PlacementRefused`。
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    /// 这次调用里推的抬 F 之外发成的每一版，按先后（[`VersionsPublishedInThisChange`]）。
    pub versions_published: VersionsPublishedInThisChange,
    /// 改动发成之后水位越线跑的那至多一批整理（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G8 丁；次序在删除续做、
    /// 为这次请求推的抬 F 之后）。没起跑是 `None`；起跑了的那一趟停在哪、发了哪几版在里面。整理没发成不让这次改动算没发成。
    /// 至多一批数的是搬了单元的发布，那一趟连同收尾、签发可以发几次见 `CompactionRunReach::AtMostOneBatchAfterAUserChange`。
    pub compaction_after_the_change: Option<CompactionRun>,
    /// 这次调用的发布都返回之后发的 discard（判决 G5，[`discard_the_segments_reusable_as_a_whole`]）。
    pub discards_after_the_change: DiscardsAfterThePublishes,
}

/// 发布返回之后发的 discard（判决 G5）：每块盘按 64 槽对齐的整段发，各段交回了哪一种结局。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscardsAfterThePublishes {
    /// 交回 `Applied` 的段：哪块盘、段起点。
    pub applied: Vec<(DeviceIdentity, SlotNumber)>,
    /// 交回 `Unsupported` 的段（后端不认 discard，一个字节没动）。
    pub unsupported: Vec<(DeviceIdentity, SlotNumber)>,
    /// 块设备报错的段：discard 只是告诉盘「这一段不用了」，报错不让发布算没发成，记下来报出。
    pub failed: Vec<(DeviceIdentity, SlotNumber)>,
    /// 装着还没清扫的孤儿头、没发的段（[`discard_the_segments_reusable_as_a_whole`]：discard 不当擦除，孤儿头留给实例表清扫抹）。
    pub withheld_holding_unswept_orphan_heads: Vec<(DeviceIdentity, SlotNumber)>,
    /// 判孤儿头时读不出实例表或段里的一个槽、没发的段（读不到不等于没有，按装着算）。
    pub withheld_after_a_failed_read: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 一次整理运行（显式入口 [`MountedSession::run_compaction`]，或用户改动之后的那至多一批）做成了什么。
#[derive(Debug)]
pub struct CompactionRun {
    /// 这一趟发成的每一版，按先后：先走空的删除续做、再是整理的各批。最后一个就是会话的 `current`。
    pub versions_published: Vec<TransactionOutput>,
    /// 其中整理批的个数：搬了单元的那几次（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2），签发、
    /// 一个没搬的删意图与丢意图不算。
    pub batches_published: u64,
    pub outcome: CompactionRunOutcome,
    /// 这一趟的发布都返回之后发的 discard（只有显式入口在这里发；用户改动之后那一趟的归改动那一份）。
    pub discards: DiscardsAfterThePublishes,
}

/// 一次整理运行停在哪：照判据停（[`CompactionRunStop`]），或一次发布没发成。
#[derive(Debug)]
pub enum CompactionRunOutcome {
    Stopped(CompactionRunStop),
    Failed(CompactionRunFailure),
}

/// 整理运行里一次发布没发成（被空间拒不在这里：那是暂停，[`CompactionRunStop::PausedShortOfSpace`]）。已发成的那几版留在盘上、`current` 停在最后那一版。
#[derive(Debug)]
pub enum CompactionRunFailure {
    /// 先走空的删除续做有一步没发成。
    DeletionContinuation(PublishError),
    /// 一批没发成（计划对不上上一版，或发布路径的错）。
    Batch(RelocationBatchRefused),
}

/// 整理运行在任何读写之前就拒了。
#[derive(Debug)]
pub enum CompactionRunRefused {
    /// 同 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]：交进来的盘表与这次挂载核过的那一份不同。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 现行那一版树表 0 条：没有可搬的单元。
    NoVersionOfTrees,
}

/// 一次用户改动里推的抬 F 之外发成的每一版，按先后：先走空的删除步（D8（核心索引结构） 已定项 16 第 6 条）、这次改动本身
/// （unlink 是意图那次）、unlink 之后走空的删除步；与推的抬 F 的空发布按 txg 交错。最后一个是会话的 `current`
/// （之后没再推抬 F 时）。交给调用方逐次比（模型对拍）：会话的 `current` 只是最后那一版。
pub type VersionsPublishedInThisChange = Vec<TransactionOutput>;

/// 一次用户改动没发成。成员按调用方要做的决定分：换一份盘表再发、换回这个池的盘、换一个版本再改、原样看发布路径的错、报空间不够。
#[derive(Debug)]
pub enum UserChangeRefused {
    /// 交进来的盘表的设备身份（按次序）与这次挂载核过的那一份不同（多一块、少一块、换了次序、同一个身份交两次）：
    /// 写入口按盘表逐项写系统配置与单元，拿别的盘表发，多出来的盘收到本池的写、设备数与本盘设备号被改写。
    /// 在任何读写之前返回，盘上逐字节不变、会话不动。推环入口与要出路先核同一判（[`RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount`]、
    /// [`WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount`]）：三个入口收盘表同一判。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 现行那一版没有可改的文件：树表 0 条（这几种改动都要接在有树的一版后面，第一个文件版本走 `transaction::publish_first_file`，
    /// 不经这一层），在任何读写之前返回；或有树而文件集合为空（`PoolVersion::WithTreesAndNoFile`，D8（核心索引结构） 已定项 16 第 5 条：
    /// 删完之后只许建 inode（size 0）、写行与空发布）而这次改动不是建 inode——先走空队列（那一版的 `deleted_inodes` 根指针置零，
    /// 走空一个写都不发；走空把带文件的一版走成文件集合为空的，那几步已落盘），再拒这次改动。
    /// 走空发成的那几版与走空里为空间推的那几串抬 F 已落盘、带在这里（与别的被拒成员一样交回被拒之前发成的，
    /// 调用方逐版比得到）；树表 0 条那一格与队列本来就空的那一格没有走空，三样都是空的。
    NoFileVersionToChange {
        floor_raises: Vec<RaisedFloor>,
        /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）。
        refusals_that_pushed_the_floor_raises: Vec<PublishError>,
        /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）：走空的每一步。
        versions_published: VersionsPublishedInThisChange,
    },
    /// 盘表的身份与这次挂载核过的那一份相同，`devices` 里的这几块（按交进来的次序）却不「可见」：两个系统配置槽里一份本池自证过的
    /// （整槽校验和过、fsid 与本池相同）都没有——换上去的空盘、别的池的盘、两槽都读不出（D18（块里携带什么信息） 已定项 11
    /// 「可写挂载的顺序」；与可写挂载取号之前的逐盘核第一支、挂着之后收盘表的入口同一判，`mount::devices_without_a_self_verified_system_configuration`）。
    /// 只比身份时，发布照样做成、本池的写落到那块盘上（代码三方 m2-closeout-code-r1 Z3-A 乙，用户 2026-09-27 定）。
    /// 读过每块盘的两个系统配置槽，一个写都没发：盘上逐字节不变、会话不动。调用方要做的是停下这块盘上的写（换回这个池的盘），
    /// 不是换一份盘表的次序再发（那是 `DeviceTableOtherThanTheOneOfTheMount`）。
    DevicesWithoutASelfVerifiedSystemConfiguration { devices: Vec<DeviceIdentity> },
    /// 盘表的身份与这次挂载核过的那一份相同、每块盘都「可见」，这几块盘（按交进来的次序）自证过的系统配置槽里记的本盘设备号却不是
    /// 盘表给它的身份——盘体对调了、或插错了位置（与可写挂载、挂着之后收盘表的入口同一判，
    /// `mount::own_device_numbers_on_disk_differing_from_the_identities_handed_in`）。只比身份时，发布照样做成、系统配置轮换把错的本盘设备号
    /// 写进两块盘、根写到不归它的那块盘上，之后这个池可写挂不上（代码三方 m2-closeout-code-r2「盘体对调的盘表」，用户 2026-09-27 定）。
    /// 读过每块盘的两个系统配置槽，一个写都没发：盘上逐字节不变、会话不动。调用方要做的是把盘体放回各自的身份再发。
    OwnDeviceNumbersDifferFromTheDeviceTable {
        disagreements: Vec<OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn>,
    },
    /// 盘表的身份对得上、每块盘都「可见」、本盘设备号也对得上，这几块盘（按交进来的次序）却落后于会话的现行那一版、又缺它的单元或它的根：
    /// 同池一份旧快照被换了上去（与可写挂载取号之前逐盘核的落后支、挂着之后收盘表的入口同一判，`mount::devices_behind_the_current_version`）。
    /// 不核这一判时发布照样做成、旧快照的系统配置被轮换到「跟得上」，之后可写挂载的落后支就认不出它、池级 checker 红 I-2.1
    /// （代码三方 m2-closeout-code-r2「同池旧快照的盘经入口洗白」，用户 2026-09-27 定）。旧快照取在一次发布途中（单元与记录已写、根槽 FUA
    /// 还没写）、那次发布的根又住在这块盘上的，单元都在而根槽里不是现行那一版：不核缺根那一半时发布照样做成、池级 checker 红 I-3.1
    /// （C585（会话发布放进发布途中取下的旧快照），用户定改法 C）。只剩一槽自证、那一槽停在上一次轮换的盘落后而单元都在、根槽里是它，不在这里。
    /// 读过每块盘的两个系统配置槽、落后那几块上现行那一版单元的落点与它的根槽，一个写都没发：盘上逐字节不变、会话不动。
    /// 调用方要做的是换回这个池现行的盘。
    DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
        devices: Vec<DeviceBehindTheCurrentVersion>,
    },
    /// 发布被拒、不是空间不够的那两种（内容装不下、释放判定对不上、块设备错、冻结着一次没重发……）：`cause` 原样，不因它推。
    /// `floor_raises` 是这之前已经因为空间不够推过的那几串（它们已落盘，现行那一版在它们后面）。
    Publish {
        cause: PublishError,
        floor_raises: Vec<RaisedFloor>,
        /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。
        versions_published: VersionsPublishedInThisChange,
    },
    /// 空间不够（报给用户就是 ENOSPC），推过的与不再推的原因在 [`NoSpaceAfterRaisingTheFloor`] 里。
    /// 装箱：不装箱 `UserChangeRefused` 就大过 clippy `result_large_err` 的 128 字节（`MountError` 那几个成员同一个做法）。
    NoSpaceAfterRaisingTheFloor(Box<NoSpaceAfterRaisingTheFloor>),
    /// 发布被拒是空间不够、为它推一串抬 F 时抬 F 自己报了不是空间不够的错（块设备错、读不出、预演被别的理由拒……）：不是「推满仍不够」，
    /// 不报空间不够（代码审阅第 23 条），错原样在 [`FloorRaiseFailedWhilePushingForSpace`] 里。抬 F 那一串自己也取不到落点、被空间准入拒的
    /// 是推满仍不够，走 `NoSpaceAfterRaisingTheFloor`。装箱同 `NoSpaceAfterRaisingTheFloor`。
    FloorRaiseFailedWhilePushingForSpace(Box<FloorRaiseFailedWhilePushingForSpace>),
}

/// 推满仍不够：最后一次被拒是 `last_refusal`（`SpaceAdmissionRefused` 或 `PlacementRefused`），已经推了 `floor_raises` 那几串
/// （每一串因为 `refusals_that_pushed_the_floor_raises` 里对应的那一次被拒才推），`stop` 是不再推的原因
/// （一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）。
#[derive(Debug)]
pub struct NoSpaceAfterRaisingTheFloor {
    pub last_refusal: PublishError,
    pub floor_raises: Vec<RaisedFloor>,
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    pub stop: FloorRaiseStop,
    /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。
    pub versions_published: VersionsPublishedInThisChange,
    /// 这一次是不是被宽护卡住（[`WideProtectionAtTheCeiling`]）：F 已在上限、C 那一圈还隔离着槽时点名这一形、带上算出路要的东西
    /// （出路在调用方要的时候算，[`MountedSession::ways_out_of_the_wide_protection`]：推环那一条预演过放得下才列），
    /// 别的形是 [`WideProtectionAtTheCeiling::NotWhatStopsTheAdmission`]，照旧只是空间不够。
    pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
}

/// 推满仍不够时，挡着准入的是不是「这个进程写掉那一槽才放」的那一份隔离（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」：
/// 不自动推环，准入因宽护卡住时报一个点名「被宽护卡住」的错、带上出路；用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」：
/// 推环那一条出路预演过、推完这次改动放得下才列；同日弹窗定「改：预演延到要出路时才做」：报的时候不预演，调用方要出路时另调
/// [`MountedSession::ways_out_of_the_wide_protection`] 算，只要错误码、不看出路的调用方不付这一笔）。
///
/// 为什么不另开一个 [`UserChangeRefused`] 成员：checker 档的装置（`e158_root_choice_repair`、`e168_delete_ring_window_borrow`）对
/// `UserChangeRefused` 与 `FloorRaiseStop` 写的是穷举 `match`，多一个成员它们就编不过，而 checker 档这一批不改（第 10z 批第十六块派发）。
/// 点名这一形的成员住在这里，外层照旧是 `NoSpaceAfterRaisingTheFloor`，交回的已落盘的东西不变。
#[derive(Debug)]
pub enum WideProtectionAtTheCeiling {
    /// 不是这一形：不再推的原因不是「F 已在上限」（预算用完、抬 F 那一串自己被拒），或 C 那一圈这一刻一个槽都没隔离。
    NotWhatStopsTheAdmission,
    /// 被宽护卡住（[`StuckBehindTheWideProtection`]）。装箱同 `NoSpaceAfterRaisingTheFloor`。
    StuckBehindTheWideProtection(Box<StuckBehindTheWideProtection>),
}

/// 被宽护卡住：准入被拒、会话推抬 F 停在上限（`FloorRaiseStop::FloorAlreadyAtTheCeiling`），而 C 那一圈（水位之上没施加的记录点名的落点、
/// 那几次发布的账、比所选那一版新的可读根的账；认不出回退时并进 R 那一槽的「现行账里全部已释放的落点」就是宽护，它是这一圈里最大的一份）
/// 还隔离着槽。这一圈只在这个进程写掉那一槽时解开（`allocator::PoolAllocator::record_root_written_by_this_process`），写被拒就不发布、
/// 那一槽写不掉，不管它就隔离到进程结束（第 10z 批第十四块实现员报告第六节，384 槽盘 22 形写全被拒）。
///
/// 这里分不出 C 那一圈里哪一份是宽护、哪一份是窄的那几份：分得出要在根环表里另记一样（`PoolAllocator` 的 `{:?}` 随之变，checker 档 E171
/// 记它），这一份不记；出路对整圈算。
///
/// **宽护是挡着准入的之一，不是唯一**：同一刻没有 C 那一圈，准入照样可以被拒（第 10z 批代码轮第一轮攻方
/// `research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md` 第一节：同一个池停在 R 之前、C 一槽都没有的对照臂，同一个长度也被拒的 27 格，
/// 这里照样报这一形）。所以报这一形不说明写掉 R 那一槽之后放得下：推完放不放得下由调用方要出路时那一份预演说
/// （[`MountedSession::ways_out_of_the_wide_protection`]），放不下（推完上限退到环里最旧的根、推环自己换下的那几份要等环再转一圈才回收，
/// 同一节「读法」）就不列推环。
///
/// 报这一形只读分配器（内存）：不拷分配器、不预演、一个盘都不读。预演要的「被拒在哪一版上」「这次改动还剩什么」记在这里（私有，
/// 只有会话造得出这一份），这次改动的内容拷一份进来（错误不借调用方的内容）。
#[derive(Debug)]
pub struct StuckBehindTheWideProtection {
    /// 推环要写到的那一槽：C 那一圈还没解开、记着落点的槽里，从现行那一版起按 txg 往后最后一个轮到的（R 那一槽）。
    pub ring_slot_of_r: RootRingSlot,
    /// R 的 txg：根环表里那一槽记着的有效根（挂载那一刻按最新那条根的实例表有效、记成有效根，写行之后才被抛弃的那一条）；
    /// 那一槽记的不是有效根（读坏、读回全零、没有根的记录）时 `None`。
    pub checkpoint_txg_of_r: Option<CheckpointTxg>,
    /// 从现行那一版起要发几次 F 不变的空发布才写掉 R 那一槽（不超过环长 3S）。
    pub empty_publishes_until_the_ring_slot_of_r_is_written: u64,
    /// C 那一圈每块盘上这一刻隔离着的槽数（按分配器里盘的次序，同一个槽只数一次）：宽护挡着的槽数。只数 C 那一圈罩着的，
    /// 被抛弃根罩着、C 不罩的不在其内。
    pub isolated_slots_on_each_device: Vec<(DeviceIdentity, u64)>,
    /// 被拒的那一次要接在后面的那一版（报这一形那一刻会话的现行那一版）的实例代号与 txg：要出路时会话不在这一版上就不预演
    /// （[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。报里不记是哪个会话报的：
    /// 别的会话的报这两个数恰好相同时不拦（同一成员的文档）。
    instance_of_the_version_refused_on: InstanceGeneration,
    checkpoint_txg_of_the_version_refused_on: CheckpointTxg,
    /// 被拒的那一次之后这次用户改动还剩什么要发（内容拷一份）：要出路时推完环照它再预演一次。
    remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal,
}

/// 调用方要出路时预演推环与推完之后这次改动的准入（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」）：
/// 在分配器的一份拷贝上、对只进内存的盘走一遍推环入口与推完之后这次改动剩下的发布（`rehearse_the_push_then_the_remainder_of_the_change`），
/// 一个写都没发到盘上，会话不动。
#[derive(Debug)]
pub enum RootRingPushRehearsal {
    /// 推环做成，推完之后报里剩下的那一段放得下（还剩这次改动时是走空再发它；只剩走空时是那一段走空，unlink 的意图已发成）：出路列推环。
    /// `isolated_slots_released_by_the_push_on_each_device` 是推完之后每块盘隔离的槽数
    /// 比推之前少了几个（按分配器里盘的次序；推完反而多的那块盘记 0）：C 那一圈与被抛弃根罩着同一个槽时，被抛弃根还在环里那一槽就不放，
    /// 推环途中写掉被抛弃根那一槽时它独占的也放出来——与 `isolated_slots_on_each_device` 不是同一个数。
    ChangeAdmittedAfterThePush {
        isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)>,
    },
    /// 推环做成，推完之后这次改动照旧被拒（`refusal` 是预演里交回的那一种，例如推完上限退到环里最旧的根、推满仍不够）：出路不列推环。
    ChangeStillRefusedAfterThePush {
        isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)>,
        refusal: Box<UserChangeRefused>,
    },
    /// 推环自己在预演里被拒（`RootRingPushRefused` 的哪一种原样）：出路不列推环。
    PushRefused(Box<RootRingPushRefused>),
}

/// 调用方要出路时算出来的（[`MountedSession::ways_out_of_the_wide_protection`]）：那一刻的预演结局与照它列的出路。
#[derive(Debug)]
pub struct WaysOutOfTheWideProtection {
    /// 在分配器的拷贝与只进内存的盘上预演推环、推完再预演这次改动（连同它会推的抬 F）的结局：推环这条出路列不列按它定。
    pub pushing_the_root_ring: RootRingPushRehearsal,
    /// 出路，调用方照抄就能报给用户：卸载重挂一定列；推环（会丢掉最近可退到的状态，调用即确认）只在预演过推完这次改动放得下时列
    /// （[`RootRingPushRehearsal::ChangeAdmittedAfterThePush`]）。按这个次序。
    pub ways_out: Vec<String>,
}

/// 要出路时没预演。成员按调用方要做的决定分：再发一次这次改动拿新的报、换回挂载时那一份盘表。
/// 两种都在预演之前返回：一个盘都没读、一个写都没发，会话不动。
#[derive(Debug)]
pub enum WaysOutOfTheWideProtectionRefused {
    /// 会话的现行那一版已不是报「被宽护卡住」时被拒的那一次要接在后面的那一版（之后又发过：推过环、别的改动发成、走空走了几步）：
    /// 报里的 R 那一槽、要推的次数与挡着的槽数说的是那一刻，照这一刻预演会与它们对不上，按哪一刻算条款没写，第一版不支持。
    /// 调用方要做的是再发一次这次改动、拿新的那一份报再要出路。这一道在盘表那一道之前：会话已不在那一版上、交进来的盘表又与挂载时的不同，
    /// 报的是这一种。同一个池卸载或崩了之后再挂取新实例，现行那一版的实例代号随之变，再挂之前的报在这一道拒成这一种
    /// （`instance_of_the_current_version` 是新实例的）。
    ///
    /// 这一道只比实例代号与 txg，报里不记是哪个会话报的。别的会话（同一份 mkfs 起的另一个池、读盘造的另一条会话）
    /// 的报若这两个数恰好与这个会话的现行那一版相同，这里不拦：照这个会话的状态（分配器、盘、现行那一版）预演，预演发的是报里拷下的那一份改动
    /// （不是这个会话自己要发的），出路一的文字仍是报里的数
    /// （R 的 txg 或 R 那一槽），列出路二时其中的 R 那一槽与要推的次数也是报里的（第 10z 批代码轮攻方「打中一」；用户 2026-10-09 定撤掉会话身份、照实写）。
    /// 调用方要拿报去问报它的那个会话。
    SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
        instance_refused_on: InstanceGeneration,
        checkpoint_txg_refused_on: CheckpointTxg,
        instance_of_the_current_version: InstanceGeneration,
        checkpoint_txg_of_the_current_version: CheckpointTxg,
    },
    /// 会话还在被拒的那一版上（实例代号与 txg 与报里记的相同），交进来的盘表的设备身份（按次序）却与这次挂载核过的那一份不同（多一块、少一块、换了次序、
    /// 同一个身份交两次；与 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`] 同一判，`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`）。
    /// 预演经推环的本体走（`MountedSession::push_the_root_ring_on`，交的是包过的盘），本体里核盘表那一套不看次序：不先核这一判时，
    /// 次序对调的盘表预演放得下、列推环，照出路二推完再发这次改动却拒成 `DeviceTableOtherThanTheOneOfTheMount`（第 10z 批代码轮攻方「打中二」）。
    /// 推环公开入口与 [`MountedSession::publish_user_change`] 先核同一判（[`RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount`]）：
    /// 三个入口收盘表同一判。调用方要做的是换回挂载时那一份盘表再要出路。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
}

impl StuckBehindTheWideProtection {
    /// 照这一份报与要出路时那一份预演列出路：卸载重挂一定列；推环只在预演过推完这次改动放得下时列。按这个次序。
    /// 出路二末句按报里剩下什么分：还剩这次改动时说推完之后这次改动放得下（调用方推完再发它）；只剩走空时说意图已发成、
    /// 剩下的走空在下一次改动开头走、放得下，不再发这次改动。
    fn ways_out_after(&self, pushing_the_root_ring: &RootRingPushRehearsal) -> Vec<String> {
        let way_one = match self.checkpoint_txg_of_r {
            Some(txg_of_r) => format!(
                "出路一：正常卸载（mount::unmount）之后再可写挂载一次。再挂时 R（txg {}）按新实例的实例表判成被抛弃，影子账只护它自己账里引用的落点，宽护不再出现。R 的账读不出若是盘上永久坏，卸载之后再挂会被拒（C694 的 B），而会话在卸载时已断开。",
                txg_of_r.0
            ),
            None => format!(
                "出路一：正常卸载（mount::unmount）之后再可写挂载一次。R 那一槽（根环槽 {}.{}）没有读得出的有效根：再挂时择的是卸载推出来的最后一版，R 那几条记录不在它的水位之上，C 那一圈照它重算、不再有这一槽，宽护不再出现。",
                self.ring_slot_of_r.region, self.ring_slot_of_r.slot
            ),
        };
        let mut ways_out = vec![way_one];
        match pushing_the_root_ring {
            RootRingPushRehearsal::ChangeAdmittedAfterThePush {
                isolated_slots_released_by_the_push_on_each_device,
            } => {
                // 报里只剩走空时这次 unlink 的意图已发成：照「再发这次改动」再发，开头的走空走完、要 unlink 的文件已不在，被拒
                // （第 10z 批代码轮攻方 H 形，判决 `research/prompts/m3-batch10z-xy-code-r1-main-verification.md` 第三节甲）。
                let what_the_rehearsal_found_after_the_push = match &self.remainder_of_the_change {
                    RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(_) => {
                        "预演过：推完之后这次改动放得下。"
                    }
                    RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue => {
                        "这次 unlink 的意图已发成，报里只剩它之后的走空：预演过，推完之后剩下的走空在下一次改动开头走、放得下；不再发这次改动。"
                    }
                };
                let released_on_each_device: Vec<String> =
                    isolated_slots_released_by_the_push_on_each_device
                        .iter()
                        .map(|(device, released)| format!("盘 {} 上 {released} 个", device.0))
                        .collect();
                ways_out.push(format!(
                    "出路二：调 MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states（推环到 R 那一槽，根环槽 {}.{}）：从现行那一版起发 {} 次 F 不变的空发布，写掉根环里接下来轮到的 {} 槽，R 那一槽是最后一槽；宽护随之解开，隔离的槽放出来 {} 个（{}）。这几槽里原来的根换成同一个用户状态的空发布根，回退候选集可以掉到少于 4 个不同状态（D16（发布语义） 已定项 1），调用即确认。{}",
                    self.ring_slot_of_r.region,
                    self.ring_slot_of_r.slot,
                    self.empty_publishes_until_the_ring_slot_of_r_is_written,
                    self.empty_publishes_until_the_ring_slot_of_r_is_written,
                    isolated_slots_released_by_the_push_on_each_device
                        .iter()
                        .map(|(_, released)| *released)
                        .sum::<u64>(),
                    released_on_each_device.join("，"),
                    what_the_rehearsal_found_after_the_push,
                ));
            }
            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
            | RootRingPushRehearsal::PushRefused(_) => {}
        }
        ways_out
    }
}

/// 预演用的盘：读穿到底下那块盘（这一刻盘上的字节），写与写零只记进内存、盖在之后的读上，屏障什么都不做——一个写都不发到底下那块盘。
/// 要出路时预演推环与推完之后这次改动的准入用（[`rehearse_the_push_then_the_remainder_of_the_change`]）；可写挂载取号之前预演续做、
/// 续做要推抬 F 时，取号之后那一串连同续做也在它上面走一遍（`mount` 的 `walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`）：
/// 那一处底下那块盘是这次挂载共用的那一份读（[`DeviceReadOnceInThisMount`]），这里读成的块在那里记下，真做再读取的是同一份。
/// 底下那块盘只读借成 `&dyn BlockDevice`：改盘的三个动作要 `&mut`，类型上调不到。请求先过两个后端那一份判据
/// （`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒。
pub(crate) struct DeviceKeepingWritesInMemory<'disk> {
    disk: &'disk dyn BlockDevice,
    physical_block_size: PhysicalBlockSizeInBytes,
    /// 写过的物理块：块号（按 `physical_block_size`）→ 那一块现在的字节。写零也逐块记成全 0（发布路径不写零，只有 mkfs 写零）。
    written_blocks: BTreeMap<u64, Vec<u8>>,
}

impl<'disk> DeviceKeepingWritesInMemory<'disk> {
    pub(crate) fn over_the_disk(disk: &'disk dyn BlockDevice) -> Self {
        DeviceKeepingWritesInMemory {
            disk,
            physical_block_size: disk.probe_physical_block_size(),
            written_blocks: BTreeMap::new(),
        }
    }

    fn block_bytes(&self) -> u64 {
        u64::from(self.physical_block_size.0)
    }

    /// 两个后端那一份判据（`block_device::check_aligned_and_in_range`），设备大小取底下那块盘的。
    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        check_aligned_and_in_range(
            offset,
            length,
            self.physical_block_size,
            self.disk.size_in_bytes(),
        )
    }

    /// 块号 `first_unwritten_block` 起、到 `end_block` 之前那一段没写过的块从底下那块盘读进 `buffer` 里对应的那一截
    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）；`pass` 是当场重读那一遍时走底下那块盘的
    /// [`BlockDevice::reread_at_after_a_bad_read`]。
    fn read_unwritten_blocks_from_the_disk(
        &self,
        first_unwritten_block: u64,
        end_block: u64,
        first_block_of_the_buffer: u64,
        buffer: &mut [u8],
        pass: ReadPassOverThePool,
    ) -> Result<(), BlockDeviceError> {
        let block_bytes = self.block_bytes();
        let start_in_buffer =
            usize::try_from((first_unwritten_block - first_block_of_the_buffer) * block_bytes)
                .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
            .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let offset = DeviceOffsetInBytes(first_unwritten_block * block_bytes);
        let range_of_the_buffer = &mut buffer[start_in_buffer..end_in_buffer];
        match pass {
            ReadPassOverThePool::FirstRead => self.disk.read_at(offset, range_of_the_buffer),
            ReadPassOverThePool::RereadAfterABadRead => self
                .disk
                .reread_at_after_a_bad_read(offset, range_of_the_buffer),
        }
    }

    /// 读一段：写过的块取内存里那一份，没写过的那几段从底下那块盘读（`pass` 定走 [`BlockDevice::read_at`] 还是当场重读那一读）。
    fn read_over_the_blocks_kept_in_memory(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
        pass: ReadPassOverThePool,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length)?;
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        // 迭代上界是这次读的块数；跨轮带着还没读的那一段没写过的块从哪一块起；出口只有底下那块盘读错（原样交回）。
        let mut unwritten_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            match self.written_blocks.get(&block_index) {
                None => {
                    unwritten_run_start.get_or_insert(block_index);
                }
                Some(written) => {
                    if let Some(run_start) = unwritten_run_start.take() {
                        self.read_unwritten_blocks_from_the_disk(
                            run_start,
                            block_index,
                            first_block,
                            buffer,
                            pass,
                        )?;
                    }
                    let start_in_buffer = usize::try_from(block_index - first_block)
                        .expect("块数装得进 usize")
                        * block_length;
                    buffer[start_in_buffer..start_in_buffer + block_length]
                        .copy_from_slice(written);
                }
            }
        }
        if let Some(run_start) = unwritten_run_start {
            self.read_unwritten_blocks_from_the_disk(
                run_start,
                end_block,
                first_block,
                buffer,
                pass,
            )?;
        }
        Ok(())
    }

    fn keep_in_memory(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
            self.written_blocks
                .insert(first_block + block_offset, block.to_vec());
        }
    }
}

impl BlockDevice for DeviceKeepingWritesInMemory<'_> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_over_the_blocks_kept_in_memory(offset, buffer, ReadPassOverThePool::FirstRead)
    }

    /// 写过的块照旧取内存里那一份（这一趟写的，盘上没有）；没写过的走底下那块盘的当场重读那一读。
    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_over_the_blocks_kept_in_memory(
            offset,
            buffer,
            ReadPassOverThePool::RereadAfterABadRead,
        )
    }

    /// 没写过的那几段交给底下那块盘（它们是从那里重读来的）；写过的块是这一趟写在内存里的，不交。
    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
        for (block_index, block) in (first_block..).zip(bytes.chunks(block_length)) {
            if !self.written_blocks.contains_key(&block_index) {
                self.disk.keep_the_self_verified_reread(
                    DeviceOffsetInBytes(block_index * block_bytes),
                    block,
                );
            }
        }
    }

    /// 没写过的那几段交给底下那块盘（读坏的字节是从那里读来的，那一层记进读坏账、丢掉记下的那几块），一段连着的没写过的块交一次；
    /// 写过的块是这一趟写在内存里的，不交。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        let block_bytes = self.block_bytes();
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length_in_bytes / block_bytes;
        let hand_the_unwritten_run_to_the_disk = |run_start: u64, run_end: u64| {
            self.disk
                .note_the_read_failing_self_verification_and_forget_its_bytes(
                    DeviceOffsetInBytes(run_start * block_bytes),
                    (run_end - run_start) * block_bytes,
                );
        };
        // 迭代上界是这一段的块数；跨轮带着还没交的那一段没写过的块从哪一块起；没有提前出口。
        let mut unwritten_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            if self.written_blocks.contains_key(&block_index) {
                if let Some(run_start) = unwritten_run_start.take() {
                    hand_the_unwritten_run_to_the_disk(run_start, block_index);
                }
            } else {
                unwritten_run_start.get_or_insert(block_index);
            }
        }
        if let Some(run_start) = unwritten_run_start {
            hand_the_unwritten_run_to_the_disk(run_start, end_block);
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, u64::try_from(bytes.len()).expect("写长装得进 u64"))?;
        self.keep_in_memory(offset, bytes);
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        let zeros = vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
        self.keep_in_memory(offset, &zeros);
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.disk.size_in_bytes()
    }
}

/// 可写挂载这次挂载里的读，取号之前的预演与取号之后的真做共用一份（岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一
/// 「读一次两边共用」那一行，用户定）：共用的那一段里每一块在这次挂载里第一次读时才穿到底下那块盘，读到的字节记下；之后再读这一块——预演
/// （分配器拷贝上那一趟、只进内存那一趟，[`DeviceKeepingWritesInMemory`] 读没写过的块穿到它这里）还是真做（判号、写行、暖机、抬 F、续做）——
/// 都取记下的那一份，不再读盘。照旧读盘的几段不在共用的那一段里：取号之前逐盘核、真做取号写之前那几次读、实例表清扫
/// （[`ReadRouteOfThisMount::StraightToTheDisk`]），取号写与轮换写算世代号、覆写哪一槽那一读（[`BlockDevice::read_at_straight_from_the_disk`]），
/// 读坏之后当场重读那一读（见下）。这次挂载写成的块记成写下的字节，之后读它的读写下的。
/// 写报错的那几块原来记着的丢掉（盘上那几块现在是什么不知道），之后读它们穿到盘上。
/// 读错（盘报了错）的那一块不记，下一次读照样穿到盘上，读错记进 `reads_failed_on_the_disk`；读坏（读成了、读的一方核出自证不过，
/// [`BlockDevice::note_the_read_failing_self_verification_and_forget_its_bytes`]）的那几块从记下的那一份里丢掉（坏字节不留，下一次读穿到盘上），
/// 读坏记进 `reads_failing_self_verification`（岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路一「两样都做」那一行，用户定）。
/// 两本账几块盘各共用一份：取号之前预演没走到续做的结局时拿它们判（`mount::establish_instance`）。
/// 记下的那一份活到这次挂载交回为止，读过的、写过的块都在里面，没有上限（岔路二「先量再定」归计数实验）。
/// 读走哪一路看几块盘共用的 `read_route`（[`ReadRouteOfThisMount`]，`mount::establish_instance` 按段切）。
/// 读坏之后当场重读那一读（[`BlockDevice::reread_at_after_a_bad_read`]）不取记下的那一份、穿到盘上读；调用方核过自证交回来的那几字节
/// （[`BlockDevice::keep_the_self_verified_reread`]）换掉记下的那一份（[`ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies`]）。
/// 请求先过两个后端那一份判据（`block_device::check_aligned_and_in_range`），真盘会拒的这里照样拒、不发到盘上。
pub(crate) struct DeviceReadOnceInThisMount<'disk, 'log, Device> {
    identity: DeviceIdentity,
    disk: &'disk mut Device,
    physical_block_size: PhysicalBlockSizeInBytes,
    /// 这次挂载读成或写成的物理块：块号（按 `physical_block_size`）→ 那一块的字节。
    blocks_read_or_written_in_this_mount: RefCell<BTreeMap<u64, Vec<u8>>>,
    reads_failed_on_the_disk: &'log RefCell<Vec<ReadFailedOnTheDisk>>,
    reads_failing_self_verification: &'log RefCell<Vec<ReadFailingSelfVerification>>,
    read_route: &'log Cell<ReadRouteOfThisMount>,
}

/// [`DeviceReadOnceInThisMount`] 的读此刻走哪一路。写不分路：写成的块一律记成写下的字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReadRouteOfThisMount {
    /// 取记下的那一份，没记下的穿到盘上读、读成记下：预演与真做共用的那一份（岔路一「读一次两边共用」那一行）。
    SharedBetweenTheRehearsalAndTheRealRun,
    /// 穿到盘上读，不取也不改记下的那一份：条款要这一刻再读一次盘的那几段——取号之前逐盘核（读阶段之后的瞬时读错在这一遍报，
    /// D18（块里携带什么信息） 已定项 11）、真做取号（屏障前后各读一次见证值，C554 乙-配置续 Q1）、实例表清扫（抹之前重读每个头，
    /// 用户 2026-10-04 定 K1-b）。它们不在岔路单那一行说的「抬 F 与续做要读的东西」里，预演也不替它们读。
    StraightToTheDisk,
    /// 共用的那一段里读坏（读不出或自证不过）之后当场重读的那一读（[`BlockDevice::reread_at_after_a_bad_read`]；D19（块指针的结构与宽度预算）
    /// 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行要它再读一次盘）：绕过记下的那一份、穿到盘上读，读错照样记进读错账；
    /// 读到的字节调用方核过自证才交回来（[`BlockDevice::keep_the_self_verified_reread`]），换掉记下的那一份——预演里换了，真做再读取的就是它。
    /// 第一次读回来的坏字节记下了也挡不住这一读（主 agent 判定：抬 F 与续做里「读坏当场重读」那一读不许被共用的那一份吃掉）。
    /// 这一路只按每一次重读走，不装进几块盘共用的那一格。
    RereadStraightToTheDiskKeepingWhatSelfVerifies,
}

impl ReadRouteOfThisMount {
    /// 这一段里当场重读那一读走哪一路：共用的那一段改走穿到盘上、核过自证再换掉记下的那一份；照旧读盘的那几段本来就穿到盘上，
    /// 也不改记下的那一份。
    fn for_the_reread_after_a_bad_read(self) -> ReadRouteOfThisMount {
        match self {
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies
            }
            ReadRouteOfThisMount::StraightToTheDisk => ReadRouteOfThisMount::StraightToTheDisk,
        }
    }
}

impl<'disk, 'log, Device: BlockDevice> DeviceReadOnceInThisMount<'disk, 'log, Device> {
    pub(crate) fn over_the_disk(
        identity: DeviceIdentity,
        disk: &'disk mut Device,
        reads_failed_on_the_disk: &'log RefCell<Vec<ReadFailedOnTheDisk>>,
        reads_failing_self_verification: &'log RefCell<Vec<ReadFailingSelfVerification>>,
        read_route: &'log Cell<ReadRouteOfThisMount>,
    ) -> Self {
        let physical_block_size = disk.probe_physical_block_size();
        DeviceReadOnceInThisMount {
            identity,
            disk,
            physical_block_size,
            blocks_read_or_written_in_this_mount: RefCell::new(BTreeMap::new()),
            reads_failed_on_the_disk,
            reads_failing_self_verification,
            read_route,
        }
    }

    fn block_bytes(&self) -> u64 {
        u64::from(self.physical_block_size.0)
    }

    /// 两个后端那一份判据（`block_device::check_aligned_and_in_range`），设备大小取底下那块盘的。
    fn check_request(
        &self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        let device_size = self.disk.size_in_bytes();
        check_aligned_and_in_range(offset, length, self.physical_block_size, device_size)
    }

    /// 块号 `first_unrecorded_block` 起、到 `end_block` 之前那一段没记下的块从底下那块盘读进 `buffer` 里对应的那一截
    /// （`buffer` 从块号 `first_block_of_the_buffer` 起）：读成就逐块记下；读错记进读错账、原样交回。
    fn read_unrecorded_blocks_from_the_disk(
        &self,
        first_unrecorded_block: u64,
        end_block: u64,
        first_block_of_the_buffer: u64,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let start_in_buffer =
            usize::try_from((first_unrecorded_block - first_block_of_the_buffer) * block_bytes)
                .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let end_in_buffer = usize::try_from((end_block - first_block_of_the_buffer) * block_bytes)
            .expect("一次读的长度装得进 usize：它就是调用方给的缓冲区长度以内");
        let offset = DeviceOffsetInBytes(first_unrecorded_block * block_bytes);
        if let Err(error) = self
            .disk
            .read_at(offset, &mut buffer[start_in_buffer..end_in_buffer])
        {
            self.reads_failed_on_the_disk
                .borrow_mut()
                .push(ReadFailedOnTheDisk {
                    device: self.identity,
                    offset,
                    length_in_bytes: (end_block - first_unrecorded_block) * block_bytes,
                    error: block_device_error_with_the_same_content(&error),
                });
            return Err(error);
        }
        let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
        // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
        for (block_index, block) in (first_unrecorded_block..end_block)
            .zip(buffer[start_in_buffer..end_in_buffer].chunks(block_length))
        {
            recorded.insert(block_index, block.to_vec());
        }
        Ok(())
    }

    /// 穿到盘上读、不取也不改记下的那一份（当场重读那一读）：读错记进读错账、原样交回。
    fn reread_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if let Err(error) = self.disk.read_at(offset, buffer) {
            self.reads_failed_on_the_disk
                .borrow_mut()
                .push(ReadFailedOnTheDisk {
                    device: self.identity,
                    offset,
                    length_in_bytes,
                    error: block_device_error_with_the_same_content(&error),
                });
            return Err(error);
        }
        Ok(())
    }

    /// 读一段，照 `route` 走（[`ReadRouteOfThisMount`]）。
    fn read_by_the_route(
        &self,
        route: ReadRouteOfThisMount,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("读长装得进 u64");
        self.check_request(offset, length)?;
        match route {
            ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),
            ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                return self.reread_straight_from_the_disk(offset, buffer)
            }
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun => {}
        }
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        // 迭代上界是这次读的块数；跨轮带着还没读的那一段没记下的块从哪一块起；出口只有底下那块盘读错（原样交回）。
        let mut unrecorded_run_start: Option<u64> = None;
        for block_index in first_block..end_block {
            let is_recorded_in_this_mount = self
                .blocks_read_or_written_in_this_mount
                .borrow()
                .contains_key(&block_index);
            if !is_recorded_in_this_mount {
                unrecorded_run_start.get_or_insert(block_index);
                continue;
            }
            if let Some(run_start) = unrecorded_run_start.take() {
                self.read_unrecorded_blocks_from_the_disk(
                    run_start,
                    block_index,
                    first_block,
                    buffer,
                )?;
            }
            let start_in_buffer = usize::try_from(block_index - first_block)
                .expect("块数装得进 usize")
                * block_length;
            let recorded = self.blocks_read_or_written_in_this_mount.borrow();
            buffer[start_in_buffer..start_in_buffer + block_length].copy_from_slice(
                recorded
                    .get(&block_index)
                    .expect("这一块刚判过记着：判与取之间没有写"),
            );
        }
        if let Some(run_start) = unrecorded_run_start {
            self.read_unrecorded_blocks_from_the_disk(run_start, end_block, first_block, buffer)?;
        }
        Ok(())
    }

    /// 写成的那几块记成写下的字节。
    fn record_the_written_blocks(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        let block_bytes = self.block_bytes();
        let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
        let first_block = offset.0 / block_bytes;
        let recorded = self.blocks_read_or_written_in_this_mount.get_mut();
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for (block_offset, block) in (0u64..).zip(bytes.chunks(block_length)) {
            recorded.insert(first_block + block_offset, block.to_vec());
        }
    }

    /// 写报错的那几块：原来记着的丢掉，之后读它们穿到盘上。
    fn forget_the_blocks_of_a_failed_write(&mut self, offset: DeviceOffsetInBytes, length: u64) {
        let block_bytes = self.block_bytes();
        let first_block = offset.0 / block_bytes;
        let end_block = first_block + length / block_bytes;
        let recorded = self.blocks_read_or_written_in_this_mount.get_mut();
        // 迭代上界是这次写的块数；不跨轮带状态；没有提前出口。
        for block_index in first_block..end_block {
            recorded.remove(&block_index);
        }
    }
}

/// 读错记进读错账要一份自己的：盘报的错交回给读的那一方，账里另存一份内容相同的（`BlockDeviceError` 带着 `io::Error`，不能 `Clone`）。
fn block_device_error_with_the_same_content(error: &BlockDeviceError) -> BlockDeviceError {
    match error {
        BlockDeviceError::OutOfRange {
            offset,
            length,
            device_size,
        } => BlockDeviceError::OutOfRange {
            offset: *offset,
            length: *length,
            device_size: *device_size,
        },
        BlockDeviceError::Unaligned {
            offset,
            length,
            physical_block_size,
        } => BlockDeviceError::Unaligned {
            offset: *offset,
            length: *length,
            physical_block_size: *physical_block_size,
        },
        BlockDeviceError::InputOutput(input_output_error) => BlockDeviceError::InputOutput(
            std::io::Error::new(input_output_error.kind(), input_output_error.to_string()),
        ),
        BlockDeviceError::ProbeFailed { attribute } => BlockDeviceError::ProbeFailed { attribute },
        BlockDeviceError::ImageFileAlreadyExists { path } => {
            BlockDeviceError::ImageFileAlreadyExists { path: path.clone() }
        }
        BlockDeviceError::ImageFileMissing { path } => {
            BlockDeviceError::ImageFileMissing { path: path.clone() }
        }
    }
}

impl<Device: BlockDevice> BlockDevice for DeviceReadOnceInThisMount<'_, '_, Device> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(self.read_route.get(), offset, buffer)
    }

    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(
            self.read_route.get().for_the_reread_after_a_bad_read(),
            offset,
            buffer,
        )
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        match self.read_route.get().for_the_reread_after_a_bad_read() {
            ReadRouteOfThisMount::StraightToTheDisk => {}
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                let length = u64::try_from(bytes.len()).expect("重读的长度装得进 u64");
                assert!(
                    self.check_request(offset, length).is_ok(),
                    "交回来的是一次读成的重读：偏移与长度过了同一份对齐与越界判据"
                );
                let block_bytes = self.block_bytes();
                let block_length = usize::try_from(block_bytes).expect("物理块装得进 usize");
                let first_block = offset.0 / block_bytes;
                let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
                // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
                for (block_index, block) in (first_block..).zip(bytes.chunks(block_length)) {
                    recorded.insert(block_index, block.to_vec());
                }
            }
        }
    }

    /// 共用的那一段里（当场重读那一读也算）：记进读坏账、丢掉记下的那几块，下一次读它们穿到盘上（坏字节不留）。
    /// 照旧读盘的那几段里读到的字节本来没记下，什么都不做。
    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        match self.read_route.get().for_the_reread_after_a_bad_read() {
            ReadRouteOfThisMount::StraightToTheDisk => {}
            ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun
            | ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies => {
                assert!(
                    self.check_request(offset, length_in_bytes).is_ok(),
                    "报读坏的是一次读成的读：偏移与长度过了同一份对齐与越界判据"
                );
                self.reads_failing_self_verification.borrow_mut().push(
                    ReadFailingSelfVerification {
                        device: self.identity,
                        offset,
                        length_in_bytes,
                    },
                );
                let block_bytes = self.block_bytes();
                let first_block = offset.0 / block_bytes;
                let end_block = first_block + length_in_bytes / block_bytes;
                let mut recorded = self.blocks_read_or_written_in_this_mount.borrow_mut();
                // 迭代上界是这一段的块数；不跨轮带状态；没有提前出口。
                for block_of_the_bad_read in first_block..end_block {
                    recorded.remove(&block_of_the_bad_read);
                }
            }
        }
    }

    /// 不取也不改记下的那一份、直接读底下那块盘；读错不记进读错账（与 [`ReadRouteOfThisMount::StraightToTheDisk`] 那一路同一个读法）。
    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.read_by_the_route(ReadRouteOfThisMount::StraightToTheDisk, offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(bytes.len()).expect("写长装得进 u64");
        self.check_request(offset, length)?;
        match self.disk.write_at(offset, bytes, durability) {
            Ok(()) => {
                self.record_the_written_blocks(offset, bytes);
                Ok(())
            }
            Err(error) => {
                self.forget_the_blocks_of_a_failed_write(offset, length);
                Err(error)
            }
        }
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.check_request(offset, length)?;
        match self.disk.write_zeroes_at(offset, length) {
            Ok(()) => {
                let zeros =
                    vec![0u8; usize::try_from(length).expect("写零的长度装得进 usize")];
                self.record_the_written_blocks(offset, &zeros);
                Ok(())
            }
            Err(error) => {
                self.forget_the_blocks_of_a_failed_write(offset, length);
                Err(error)
            }
        }
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.disk.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.physical_block_size
    }

    fn size_in_bytes(&self) -> u64 {
        self.disk.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.disk
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// 被拒的那一次发布之后，这次用户改动还剩什么要发（「被宽护卡住」那一报记一份拷贝，要出路时推完环之后照它再预演一次，
/// [`rehearse_the_push_then_the_remainder_of_the_change`]）。
#[derive(Clone, Copy, Debug)]
enum RemainderOfTheUserChange<'content> {
    /// 先把现行那一版的 `deleted_inodes` 走空、再发这次改动（unlink 连同它之后的走空）：被拒的是走空的一步或这次改动本身。
    WalkTheQueueThenPublishTheChange(UserChange<'content>),
    /// 只剩走空：unlink 的意图已发成，被拒的是它之后走空的一步（别的改动之后队列本来就空）。
    WalkTheQueue,
}

/// [`RemainderOfTheUserChange`] 记进「被宽护卡住」那一份报里的拷贝：错误活得比调用方借给发布的内容长，内容拷一份。
#[derive(Debug)]
enum RemainderOfTheUserChangeKeptByTheRefusal {
    WalkTheQueueThenPublishTheChange(UserChangeKeptByTheRefusal),
    WalkTheQueue,
}

/// [`UserChange`] 的一份拷贝，内容自有。
#[derive(Debug)]
enum UserChangeKeptByTheRefusal {
    Overwrite {
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    SequentialWrite {
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    NewInodes {
        count: u64,
        write_time_seconds: u64,
    },
    Unlink {
        inode: InodeNumber,
        write_time_seconds: u64,
    },
    CreateSnapshot,
    DestroySnapshot {
        snapshot: TreeIdentifier,
    },
    Truncate {
        inode: InodeNumber,
        new_size: u64,
        write_time_seconds: u64,
    },
    OverwriteOfAnInode {
        inode: InodeNumber,
        content: Vec<u8>,
        write_time_seconds: u64,
    },
    SequentialWriteOfAnInode {
        inode: InodeNumber,
        content: Vec<u8>,
        write_time_seconds: u64,
    },
}

impl UserChangeKeptByTheRefusal {
    fn kept_from(change: UserChange<'_>) -> Self {
        match change {
            UserChange::Overwrite(file) => UserChangeKeptByTheRefusal::Overwrite {
                content: file.content.to_vec(),
                write_time_seconds: file.write_time_seconds,
            },
            UserChange::SequentialWrite(file) => UserChangeKeptByTheRefusal::SequentialWrite {
                content: file.content.to_vec(),
                write_time_seconds: file.write_time_seconds,
            },
            UserChange::NewInodes {
                count,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::NewInodes {
                count,
                write_time_seconds,
            },
            UserChange::Unlink {
                inode,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::Unlink {
                inode,
                write_time_seconds,
            },
            UserChange::CreateSnapshot => UserChangeKeptByTheRefusal::CreateSnapshot,
            UserChange::DestroySnapshot { snapshot } => {
                UserChangeKeptByTheRefusal::DestroySnapshot { snapshot }
            }
            UserChange::Truncate {
                inode,
                new_size,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::Truncate {
                inode,
                new_size,
                write_time_seconds,
            },
            UserChange::OverwriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::OverwriteOfAnInode {
                inode,
                content: content.to_vec(),
                write_time_seconds,
            },
            UserChange::SequentialWriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChangeKeptByTheRefusal::SequentialWriteOfAnInode {
                inode,
                content: content.to_vec(),
                write_time_seconds,
            },
        }
    }

    fn as_user_change(&self) -> UserChange<'_> {
        match self {
            UserChangeKeptByTheRefusal::Overwrite {
                content,
                write_time_seconds,
            } => UserChange::Overwrite(FirstFile {
                content,
                write_time_seconds: *write_time_seconds,
            }),
            UserChangeKeptByTheRefusal::SequentialWrite {
                content,
                write_time_seconds,
            } => UserChange::SequentialWrite(FirstFile {
                content,
                write_time_seconds: *write_time_seconds,
            }),
            UserChangeKeptByTheRefusal::NewInodes {
                count,
                write_time_seconds,
            } => UserChange::NewInodes {
                count: *count,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::Unlink {
                inode,
                write_time_seconds,
            } => UserChange::Unlink {
                inode: *inode,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::CreateSnapshot => UserChange::CreateSnapshot,
            UserChangeKeptByTheRefusal::DestroySnapshot { snapshot } => {
                UserChange::DestroySnapshot {
                    snapshot: *snapshot,
                }
            }
            UserChangeKeptByTheRefusal::Truncate {
                inode,
                new_size,
                write_time_seconds,
            } => UserChange::Truncate {
                inode: *inode,
                new_size: *new_size,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::OverwriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChange::OverwriteOfAnInode {
                inode: *inode,
                content,
                write_time_seconds: *write_time_seconds,
            },
            UserChangeKeptByTheRefusal::SequentialWriteOfAnInode {
                inode,
                content,
                write_time_seconds,
            } => UserChange::SequentialWriteOfAnInode {
                inode: *inode,
                content,
                write_time_seconds: *write_time_seconds,
            },
        }
    }
}

impl RemainderOfTheUserChangeKeptByTheRefusal {
    fn kept_from(remainder: RemainderOfTheUserChange<'_>) -> Self {
        match remainder {
            RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(
                    UserChangeKeptByTheRefusal::kept_from(change),
                )
            }
            RemainderOfTheUserChange::WalkTheQueue => {
                RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue
            }
        }
    }

    fn as_remainder(&self) -> RemainderOfTheUserChange<'_> {
        match self {
            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueueThenPublishTheChange(change) => {
                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change.as_user_change())
            }
            RemainderOfTheUserChangeKeptByTheRefusal::WalkTheQueue => {
                RemainderOfTheUserChange::WalkTheQueue
            }
        }
    }
}

/// 每块盘上这一刻隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项那一项），按分配器里盘的次序。只读内存。
fn isolated_slots_of_each_device(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64)> {
    allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect()
}

/// 要出路时的预演（用户 2026-10-09 弹窗定「甲：预演推环与准入，推不通就不列」「改：预演延到要出路时才做」，
/// [`MountedSession::ways_out_of_the_wide_protection`]）：在分配器的一份拷贝上、对只进内存的盘
/// （[`DeviceKeepingWritesInMemory`]）先走一遍推环入口（与真调推环入口同一段，[`MountedSession::push_the_root_ring_on`]：它自己的整串预演、
/// 真发与末尾的断言都在），推完再把这次改动剩下的照会话发布那一段走一遍（`remainder`：准入被拒照样推抬 F，抬 F 的上限按推完的根环算——
/// 读的是盖着推环那几次写的盘）。读盘照读底下那几块盘；写一个都不发到底下那几块盘；会话的分配器与现行那一版不动。
///
/// 预演里的准入再被拒时报不报「被宽护卡住」只读内存（[`wide_protection_at_the_ceiling`] 不预演）；推环做成之后 C 那一圈一槽都不记着
/// （推环入口末尾那条断言），报的是 `NotWhatStopsTheAdmission`。推环在预演里被拒就不往下走。
fn rehearse_the_push_then_the_remainder_of_the_change<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
) -> RootRingPushRehearsal {
    let mut devices_keeping_writes_in_memory: Vec<(
        DeviceIdentity,
        DeviceKeepingWritesInMemory<'_>,
    )> = devices
        .iter()
        .map(|(identity, device)| {
            let disk: &dyn BlockDevice = device;
            (*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))
        })
        .collect();
    let mut rehearsed_allocator = allocator.clone();
    let mut rehearsed_version = PoolVersion::of_a_version_of_trees(current.clone());
    let isolated_before_the_push = isolated_slots_of_each_device(&rehearsed_allocator);
    if let Err(refusal) = MountedSession::push_the_root_ring_on(
        parameters,
        &mut devices_keeping_writes_in_memory,
        &mut rehearsed_allocator,
        &mut rehearsed_version,
        instance,
    ) {
        return RootRingPushRehearsal::PushRefused(Box::new(refusal));
    }
    let isolated_slots_released_by_the_push_on_each_device: Vec<(DeviceIdentity, u64)> =
        isolated_before_the_push
            .iter()
            .zip(isolated_slots_of_each_device(&rehearsed_allocator))
            .map(|((device, before), (device_after, after))| {
                assert_eq!(
                    *device, device_after,
                    "推环前后数的是同一个分配器的拷贝：盘的次序不变"
                );
                (*device, before.saturating_sub(after))
            })
            .collect();
    let remainder_after_the_push = match remainder {
        RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
            MountedSession::publish_the_change_raising_the_floor_for_space(
                parameters,
                &mut devices_keeping_writes_in_memory,
                &mut rehearsed_allocator,
                &mut rehearsed_version,
                instance,
                shadow_ledger,
                change,
            )
            .map(|_published| ())
        }
        RemainderOfTheUserChange::WalkTheQueue => {
            let mut version_to_walk = rehearsed_version.into_version_of_trees().expect(
                "推环接在有树的一版后面、发的是带文件那一版的空发布（树表 0 条那一版推环入口先拒了），推完还是有树的一版",
            );
            publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                parameters,
                &mut devices_keeping_writes_in_memory,
                &mut rehearsed_allocator,
                &mut version_to_walk,
                instance,
                shadow_ledger,
                RemainderOfTheUserChange::WalkTheQueue,
                PublishedInThisChange {
                    floor_raises: Vec::new(),
                    refusals_that_pushed_the_floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                },
            )
            .map(|_published_in_this_change| ())
        }
    };
    match remainder_after_the_push {
        Ok(()) => RootRingPushRehearsal::ChangeAdmittedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
        },
        Err(refusal) => RootRingPushRehearsal::ChangeStillRefusedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
            refusal: Box::new(refusal),
        },
    }
}

/// C 那一圈还没解开、记着落点的那几槽，与从 `current_txg` 起推环要推到哪儿：推环入口（[`MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`]）
/// 与「被宽护卡住」那一报（[`wide_protection_at_the_ceiling`]）读同一份。
struct RootRingPushTarget {
    /// 这几槽，按槽的次序。
    ring_slots: Vec<RootRingSlot>,
    /// 这几槽记着的全部落点。
    placements: Vec<PlacementOnDevice>,
    /// 最后一个轮到的那一槽（R 那一槽）与写掉它要发几次空发布。
    ring_slot_of_r: RootRingSlot,
    publishes_until_the_ring_slot_of_r_is_written: u64,
}

/// [`RootRingPushTarget`]：分配器没装根环表、或 C 那一圈这一刻没有记着落点的槽时 `None`。只读内存。
///
/// 每一槽要几次发布：txg 逐个加一、每次发布写 `target_for_publish(txg)` 那一槽（`root_ring::target_for_publish`），环长 3S 个连续的 txg
/// 里每一槽恰好轮到一次，所以从 `current_txg + 1` 起数到 3S 之内必找到；txg 加上去越过 u64 的那几个当找不到，取满一环 3S
/// （真发时那一次发布自己就按 txg 到顶拒，`PublishError::NextCheckpointTxgPastTheTopOfItsRange`，在推环的预演里先报出来）。
fn root_ring_push_target(
    allocator: &PoolAllocator,
    current_txg: CheckpointTxg,
) -> Option<RootRingPushTarget> {
    let occupancy = allocator.root_ring_occupancy()?;
    let holding = occupancy.ring_slots_holding_placements_named_by_unapplied_journal_records();
    let slots_per_region = occupancy.slots_per_region();
    let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
    let publishes_until_written = |ring_slot: RootRingSlot| -> u64 {
        (1..=ring_length_in_txgs)
            .find(|publishes_ahead| {
                current_txg
                    .0
                    .checked_add(*publishes_ahead)
                    .is_some_and(|txg| {
                        target_for_publish(CheckpointTxg(txg), slots_per_region) == ring_slot
                    })
            })
            .unwrap_or(ring_length_in_txgs)
    };
    let (publishes_until_the_ring_slot_of_r_is_written, ring_slot_of_r) = holding
        .keys()
        .map(|ring_slot| (publishes_until_written(*ring_slot), *ring_slot))
        .max()?;
    Some(RootRingPushTarget {
        ring_slots: holding.keys().copied().collect(),
        placements: holding.into_values().flatten().collect(),
        ring_slot_of_r,
        publishes_until_the_ring_slot_of_r_is_written,
    })
}

/// 推满仍不够时报不报「被宽护卡住」（[`WideProtectionAtTheCeiling`]）：`stop` 是 F 已在上限、C 那一圈还有记着落点的槽、它们这一刻至少隔离着一个槽，
/// 才报；别的都照旧。判这三样与报的那几个数只读分配器（内存）。报的时候不预演（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：
/// 只记下被拒在哪一版上（`current`）与这次改动还剩什么（`remainder`，内容拷一份），调用方要出路时
/// [`MountedSession::ways_out_of_the_wide_protection`] 照它们预演。
fn wide_protection_at_the_ceiling(
    allocator: &PoolAllocator,
    current: &TransactionOutput,
    remainder: RemainderOfTheUserChange<'_>,
    stop: &FloorRaiseStop,
) -> WideProtectionAtTheCeiling {
    match stop {
        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}
        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }
        | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {
            return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
        }
    }
    let Some(target) = root_ring_push_target(allocator, current.root.checkpoint_txg) else {
        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
    };
    let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);
    if isolated_slots_on_each_device
        .iter()
        .all(|(_, isolated)| *isolated == 0)
    {
        return WideProtectionAtTheCeiling::NotWhatStopsTheAdmission;
    }
    let checkpoint_txg_of_r = allocator.root_ring_occupancy().and_then(|occupancy| {
        occupancy.checkpoint_txg_of_the_valid_root_recorded_on(target.ring_slot_of_r)
    });
    WideProtectionAtTheCeiling::StuckBehindTheWideProtection(Box::new(
        StuckBehindTheWideProtection {
            ring_slot_of_r: target.ring_slot_of_r,
            checkpoint_txg_of_r,
            empty_publishes_until_the_ring_slot_of_r_is_written: target
                .publishes_until_the_ring_slot_of_r_is_written,
            isolated_slots_on_each_device,
            instance_of_the_version_refused_on: current.root.instance,
            checkpoint_txg_of_the_version_refused_on: current.root.checkpoint_txg,
            remainder_of_the_change: RemainderOfTheUserChangeKeptByTheRefusal::kept_from(remainder),
        },
    ))
}

/// 推环入口（[`MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`]）做成了：
/// 发成的每一版按先后（最后一版是会话的 `current`），与写掉的那几槽（C 那一圈在这几槽记着的落点都已解开）。
#[derive(Debug)]
pub struct RootRingPushed {
    pub versions_published: Vec<TransactionOutput>,
    pub ring_slots_written: Vec<RootRingSlot>,
}

/// 推环入口没做成。成员按调用方要做的决定分：换回挂载时那一份盘表、等冻结着的那次发布重发、换盘表或换回这个池的盘、这一形第一版不支持、不必推、
/// 空间不够推不过去、发布路径的错。
#[derive(Debug)]
pub enum RootRingPushRefused {
    /// 交进来的盘表的设备身份（按次序）与这次挂载核过的那一份不同（多一块、少一块、换了次序、同一个身份交两次）：入口第一道，
    /// 与 [`UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount`]、[`WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount`]
    /// 同一判（`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`，只比身份），三个入口收盘表同一判
    /// （用户 2026-10-10 弹窗定「也先比次序」）。不核这一判时次序对调的盘表照样推成（写的先后跟着盘表次序变），推完照它再发被拒成盘表那一种，
    /// 三个入口收不收同一份盘表不一致（判决 `research/prompts/m3-batch10z-xy-code-r1-main-verification.md` 第二节 K1-2）。
    /// 在任何读写之前返回：物理块大小没探、一个盘都没读、一个写都没发，盘上逐字节不变、分配器与会话不动。调用方要做的是换回挂载时那一份盘表再推。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 分配器上冻结着一次落盘途中失败、还没原样重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」；它的 txg 与实例代号）：
    /// 推环的第一次空发布本来就会被拒（`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`），而整串预演走的是发布路径落盘之前那一段、
    /// 不判冻结，会在冻结着的分配器拷贝上照样过。比过盘表次序（只比身份、不读盘）之后就拒，在核盘表那一套、预演与任何读写之前：
    /// 盘上逐字节不变、分配器与会话不动。调用方要做的是先让那次发布原样重发（会话这一层没有重发的入口，等进程重开）。
    PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg,
        instance: InstanceGeneration,
    },
    /// 交进来的盘表没过挂着之后收盘表的那一套核（与抬 F 那一串的空发布同一套：身份交了不止一次、参数或盘表与盘上择到的系统配置不一致、
    /// 有盘不「可见」、有盘落后于现行那一版又缺它的单元或它的根；系统配置择不出来），原样带着那一种 `MountError`。
    /// 身份与次序已由入口第一道比过（`DeviceTableOtherThanTheOneOfTheMount`；要出路时的预演由要出路那一道比过），到这里的盘表身份与挂载时那一份
    /// 逐项相同，身份交了不止一次那一支走不到；
    /// 盘体对调（身份照旧、盘上的本盘设备号对不上）这类读盘才看得出的落在这里。一个写都没发，盘上逐字节不变、会话不动。
    DevicesHandedIn(Box<MountError>),
    /// 现行那一版树表 0 条：推环的空发布在这一版上是零单元发布（D16（发布语义） 已定项 9），会话层的推环在这一形上条款没写，第一版不支持。
    /// 会话在这一版上也不会被宽护卡住（用户改动在这一版上一律 `NoFileVersionToChange`）。在任何读写之前返回，盘上逐字节不变、会话不动。
    VersionWithoutFileNotSupportedInTheFirstVersion,
    /// C 那一圈这一刻没有记着落点的槽，或记着落点而它们罩着的槽这一刻一个都没隔离（全在现行账里仍分配、或已释放还在 defer 窗口里）：
    /// 这个进程写掉哪一槽都放不出东西，不推。一个写都没发，盘上逐字节不变、会话不动。
    NothingHeldUntilThisProcessWritesARingSlot,
    /// 整串空发布在任何写之前的预演里（分配器的一份拷贝上走发布路径落盘之前那一段）第 `refused_publish_in_the_push` 次（从 1 数）被拒：
    /// 推到半路空发布取不到落点（`PublishError::PlacementRefused`）就是这里。一个写都没发，盘上逐字节不变、分配器与会话不动。
    RefusedByTheRehearsalBeforeAnyWrite {
        refused_publish_in_the_push: usize,
        publishes_in_the_push: usize,
        cause: PublishError,
    },
    /// 预演过了、真发时有一次发不出去（块设备错这一类：落盘途中失败的那一次冻结在分配器上，之后再调入口拒成
    /// `PublishFrozenAfterAWriteFailureIsNotResentYet`；冻结着时入口在预演之前就拒，不进预演，所以不在这里。空间上预演与真发只在
    /// 读盘核换下的那一份对不上时分叉）：[`RootRingPushFailedAfterTheRehearsal`]。
    PublishFailedAfterTheRehearsal(Box<RootRingPushFailedAfterTheRehearsal>),
}

/// 推环真发时第 `failed_publish_in_the_push` 次（从 1 数）发不出去：那一次没成立（分配器换回它之前，发布路径的做法），
/// 前面那几次已落盘、在 `versions_published` 里，会话的 `current` 停在最后一次落盘的那一版上。
#[derive(Debug)]
pub struct RootRingPushFailedAfterTheRehearsal {
    pub failed_publish_in_the_push: usize,
    pub publishes_in_the_push: usize,
    pub cause: PublishError,
    pub versions_published: Vec<TransactionOutput>,
}

/// 为空间不够推抬 F 时抬 F 自己报了错：`cause` 是抬 F 的错原样（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`），
/// 这一串是因为 `refusal_that_pushed_the_failed_raise` 那一次被拒才推的；之前推成的几串在 `floor_raises`（已落盘，
/// 每一串因为 `refusals_that_pushed_the_floor_raises` 里对应的那一次被拒才推）。报错那一串落盘途中失败时（`MountError::RaiseFloorSequencePublishFailed`），
/// 它前面已落盘的几次在会话的现行版本里、不在 `floor_raises` 里。
#[derive(Debug)]
pub struct FloorRaiseFailedWhilePushingForSpace {
    pub cause: MountError,
    pub refusal_that_pushed_the_failed_raise: PublishError,
    pub floor_raises: Vec<RaisedFloor>,
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    /// 报错之前发成的那几版（[`VersionsPublishedInThisChange`]）。
    pub versions_published: VersionsPublishedInThisChange,
}

/// 这一次发布被拒是不是空间不够的那两种（D16（发布语义） 已定项 1「准入」那一行：准入不够，或准入放行而落点取不到）——
/// 是就推一串抬 F 再重判，不是就原样交回。
#[must_use]
pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
    match refusal {
        PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,
        // 合取 7（全空聚簇段少于 `N₀`）：全空段数按记录口径数（`allocator::DeviceFreeMap::empty_segments`，defer 里已释放的槽本来就不算占用），
        // 抬 F 把 defer 里的槽回收不改这个数，推了只多发几次空发布；原样交回这个成员，调用方分得出「先整理」与「真没空间」。
        PublishError::EmptyClusterSegmentsBelowTheFloor(_)
        | PublishError::MultiLevelCodeTwoTreeRefused { .. }
        | PublishError::ReleaseNotInMapping { .. }
        | PublishError::ReleaseTargetNotAllocated { .. }
        | PublishError::ReleaseTargetAlreadyReleased { .. }
        | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
        | PublishError::ReleaseSpanMismatch { .. }
        | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
        | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
        | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
        | PublishError::ContentExceedsDataUnit { .. }
        | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
        | PublishError::InodeTreeWriteRefused(_)
        | PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. }
        | PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. }
        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        // 一次发布切出来的记录多于上限：推抬 F 腾不出记录的位置，要把改动切小（代码审阅第 26 条）；txg 到顶是坏镜像（第 36 条）。
        | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
        | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        // 墓碑树、谱系树或退役树最右那片的容器号已到顶（C841）：坏镜像，推抬 F 不会让号变小。
        | PublishError::NextContainerNumberPastTheTopOfItsRange(_)
        // 写删除中的 inode、写已经删完的 inode、unlink 一个不是活记录的号：推抬 F 也不会让它复活。
        | PublishError::WriteToAnInodeBeingDeleted { .. }
        | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
        // 删完第一个文件之后给别的 inode 写数据（K-U5，第一版不支持）：推抬 F 也不会让第一个文件回来。
        | PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. }
        | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
        // `deleted_inodes` 装不下：类型化错误（D8（核心索引结构） 已定项 16 第 6 条），与空间无关。
        | PublishError::DeletedInodesQueueRefused(_)
        // 快照已到第一版的上限、树 ID 水位到顶：推抬 F 回收不出快照的名额，也抬不低水位。
        | PublishError::SnapshotCreationRefused(_)
        // 删快照点名的快照不在、正在删、已有别的在删：推抬 F 改变不了树表。deadlist 条目留存的映射条目没有、落点核不过：盘上的事，与空间无关。
        | PublishError::SnapshotDestructionRefused(_)
        | PublishError::DeadlistEntryNotReleasable { .. }
        // 改 size 的对象不是活记录、目标 size 溢出、条款没写的那一格、尾单元读不出：推抬 F 改变不了。
        | PublishError::TruncationRefused(_)
        | PublishError::BlockDevice(_)
        // 取号之前预演读错、读坏那两拒只在可写挂载取号之前那一判里造，发布路径不报它们。
        | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
        | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => false,
    }
}

/// 在一个新开的写入口上把现行那一版的 `deleted_inodes` 走空（D8（核心索引结构） 已定项 16 第 6 条：会话里每一次用户改动先走空），
/// 每发成一步 `current` 就换成那一版——中途失败时 `current` 停在最后一次落盘的那一版上。批大小与墓碑容器容量取分配器上装的开关。
fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    instance: InstanceGeneration,
) -> DeletionQueueWalk {
    let switches: DeletionSwitches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    walk_the_deletion_queue_until_it_is_empty(
        &mut writer,
        allocator,
        current,
        switches.batch_size,
        instance,
    )
}

/// 在一个新开的写入口上发一次这个改动，接在 `previous` 后面。unlink 这里只发意图那一次，全部批由调用方接着走空。
fn publish_the_change_once<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    change: UserChange<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let switches: DeletionSwitches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    match change {
        UserChange::Unlink {
            inode,
            write_time_seconds,
        } => publish_unlink_intent(
            &mut writer,
            allocator,
            previous,
            inode,
            write_time_seconds,
            instance,
        ),
        UserChange::Overwrite(file) => {
            publish_overwrite(&mut writer, allocator, previous, file, instance)
        }
        UserChange::SequentialWrite(file) => {
            publish_sequential_write(&mut writer, allocator, previous, file, instance)
        }
        UserChange::NewInodes {
            count,
            write_time_seconds,
        } => publish_new_inodes(
            &mut writer,
            allocator,
            previous,
            count,
            write_time_seconds,
            instance,
        ),
        UserChange::CreateSnapshot => {
            publish_create_snapshot(&mut writer, allocator, previous, instance)
        }
        UserChange::DestroySnapshot { snapshot } => publish_snapshot_destruction_intent(
            &mut writer,
            allocator,
            previous,
            snapshot,
            instance,
        ),
        UserChange::Truncate {
            inode,
            new_size,
            write_time_seconds,
        } => publish_size_change(
            &mut writer,
            allocator,
            previous,
            inode,
            new_size,
            write_time_seconds,
            instance,
        ),
        UserChange::OverwriteOfAnInode {
            inode,
            content,
            write_time_seconds,
        } => publish_overwrite_of_an_inode(
            &mut writer,
            allocator,
            previous,
            inode,
            content,
            write_time_seconds,
            instance,
        ),
        UserChange::SequentialWriteOfAnInode {
            inode,
            content,
            write_time_seconds,
        } => publish_sequential_write_of_an_inode(
            &mut writer,
            allocator,
            previous,
            inode,
            content,
            write_time_seconds,
            instance,
        ),
    }
}

impl MountedSession {
    /// 一次可写挂载做成之后的会话：交回挂载写出的东西与接着它的会话。
    #[must_use]
    pub fn of_the_mount(mounted: Mounted) -> (MountOutput, MountedSession) {
        let Mounted {
            output,
            allocator,
            current,
        } = mounted;
        let session = MountedSession {
            allocator,
            current,
            instance: output.instance,
            shadow_ledger: output.shadow_ledger,
            parameters_and_device_table: output.parameters_and_device_table.clone(),
            compaction: output.compaction_continuation.state_of_this_mount.clone(),
        };
        (output, session)
    }

    /// 发布一次用户改动：被拒是空间不够的那两种（[`refusal_is_short_of_space`]）就推一串抬 F 的空发布、再重判，
    /// 直到发成或不再推（D16（发布语义） 已定项 1「准入」那一行）。发成时 `current` 换成新的一版；推了的那几串已落盘，
    /// 没发成时 `current` 停在最后一次落盘的空发布上。写入口只按会话持着的那份参数建（`parameters_and_device_table`），
    /// 交进来的盘表先与它逐项比，再读每块盘的两个系统配置槽核它「可见」（Z3-A 乙），再核本盘设备号与落后支
    /// （D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」：「每次发布」按「每次用户改动」读）。
    /// 三样在进循环之前核一次：一次调用里盘表借着、换不了盘；循环里推的每一串抬 F 在它自己的第一个写之前再核一遍三样
    /// （`mount::push_one_floor_raise_within_the_admission_budget` 经 `caller_inputs_agreeing_with_the_disk`），
    /// 所以这次调用发出的每一个写之前都核过。
    ///
    /// 循环：每一轮发一次；被拒是空间不够就推一串（至少一次空发布、推的次数增加）进下一轮，不推了就交回。
    /// 预算把推的次数压在 B − 1 以内 ⇒ 至多 8 轮；跨轮携带的是现行那一版（抬 F 就地推进它）与已推的几串。
    ///
    /// 先把现行那一版的 `deleted_inodes` 走空（每一次用户改动都先走空，D8（核心索引结构） 已定项 16 第 6 条），再发这次改动；
    /// unlink 发成意图之后接着把它的全部批走完。走空里的删除类发布（不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）
    /// 被拒是空间不够时同样推一串抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」，每一次被拒的删除类发布各算一次准入），
    /// 推满仍不够交回 `NoSpaceAfterRaisingTheFloor`；推的那几串与意图那次推的一起报在 `floor_raises` 里。
    /// 删除那几次发布的两个只供测试的开关取分配器上装的（可写挂载的开关链装上去的，`unlink::DeletionSwitches`）。
    /// 每发成一次（走空的每一步、这次改动）`current` 都按 `PoolVersion::of_a_version_of_trees` 重新归成员；现行那一版是有树而文件集合为空的
    /// 一版时只放建 inode（D8（核心索引结构） 已定项 16 第 5 条），别的改动拒成 `NoFileVersionToChange`。
    ///
    /// # Errors
    /// [`UserChangeRefused`] 的八种。
    pub fn publish_user_change<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        change: UserChange<'_>,
    ) -> Result<UserChangePublished, UserChangeRefused> {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger,
            parameters_and_device_table,
            compaction,
        } = self;
        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
            return Err(UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        let parameters = parameters_and_device_table.parameters_on_disk();
        let current_version_of_trees = match &*current {
            PoolVersion::WithFile(current_version_of_trees)
            | PoolVersion::WithTreesAndNoFile(current_version_of_trees) => current_version_of_trees,
            PoolVersion::WithoutFile(_) => {
                return Err(no_file_version_to_change(PublishedInThisChange {
                    floor_raises: Vec::new(),
                    refusals_that_pushed_the_floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                }))
            }
        };
        let devices_not_visible =
            devices_without_a_self_verified_system_configuration(devices.as_slice(), parameters);
        if !devices_not_visible.is_empty() {
            return Err(
                UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration {
                    devices: devices_not_visible,
                },
            );
        }
        let own_device_numbers_differing =
            own_device_numbers_on_disk_differing_from_the_identities_handed_in(
                devices.as_slice(),
                parameters,
            );
        if !own_device_numbers_differing.is_empty() {
            return Err(
                UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable {
                    disagreements: own_device_numbers_differing,
                },
            );
        }
        let devices_behind = devices_behind_the_current_version(
            devices.as_slice(),
            parameters,
            current_version_of_trees,
        );
        if !devices_behind.is_empty() {
            return Err(
                UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    devices: devices_behind,
                },
            );
        }
        let mut published = Self::publish_the_change_raising_the_floor_for_space(
            parameters,
            devices,
            allocator,
            current,
            *instance,
            *shadow_ledger,
            change,
        )?;
        // 改动发成：整理「已经过去的发布数」从这里重新数（判决 G8 `B_gc`）。之后水位越线就跑至多一批（判决 G8 丁），
        // 次序在这次改动的删除续做与为它推的抬 F 之后；整理没发成不让这次改动算没发成。
        compaction.publishes_since_the_last_user_change = 0;
        if compaction_starts_after_a_user_change(allocator.compaction().switches.run_forcing) {
            published.compaction_after_the_change = Some(run_compaction_on(
                parameters,
                devices.as_mut_slice(),
                allocator,
                compaction,
                current,
                *instance,
                CompactionRunReach::AtMostOneBatchAfterAUserChange,
            ));
        }
        published.discards_after_the_change = discard_the_segments_reusable_as_a_whole(
            devices.as_mut_slice(),
            allocator,
            parameters,
            current.root(),
        );
        Ok(published)
    }

    /// 整理的显式入口（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G8 甲）：先把删除续做走空，再一次一次推整理意图——
    /// 没有活意图就选段、签发前判、发一次把意图写进 logged_ops 的发布（判决 G1）；有就对 R 重扫（不复用上一批的扫描，丁 1 交判第 3 点）、
    /// R 里有隔离槽就发一次删意图的发布丢掉它（攻方 K-c）、计划一批、发这一批，最后一批同一次发布删意图（D8（核心索引结构） 已定项 5）；
    /// 全局差值为假就不签下一条。这一趟至多发 `B_gc` 次整理发布（[`publish_budget_of_a_compaction_run`]，签发、搬一批、删意图各算一次）；
    /// 一次整理发布被空间拒就暂停（撤 R 排他、意图留在盘上）、不推抬 F。发布都返回之后发 discard（判决 G5）。
    /// 批大小条款没写，产品路径在签任何意图之前停下（`CompactionRunStop::BatchSizeNotDecidedByTheClauses`），只供测试的开关给出批大小才推得动。
    ///
    /// # Errors
    /// 盘表与挂载核过的不同、现行那一版树表 0 条 ⇒ [`CompactionRunRefused`]，在任何读写之前。
    pub fn run_compaction<Device: BlockDevice>(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<CompactionRun, CompactionRunRefused> {
        self.run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region(
            devices,
            RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        )
    }

    /// 同 [`Self::run_compaction`]，只差一处：一批照抄意图核出腾空了 R、照同一批删意图再发时，删意图那一支落盘之前的核照 `region_check` 判
    /// （判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2）。别处删意图（最后一批、R 已空的只删意图）照记录核，不读它。
    /// 产品路径只经 [`Self::run_compaction`] 进来（[`RegionCheckOfTheIntentDeletion::JudgedByTheRecords`]）；只供测试的
    /// [`RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`] 强制进入「删意图那一支核不过、照这一批不删意图再发」
    /// 那一支（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：发出去的那一版 R 按记录全空、意图还在，下一次发布照「R 已按记录全空」只删意图。
    ///
    /// # Errors
    /// 同 [`Self::run_compaction`]。
    pub fn run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        region_check: RegionCheckOfTheIntentDeletion,
    ) -> Result<CompactionRun, CompactionRunRefused> {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction,
        } = self;
        if !parameters_and_device_table.is_the_device_table_of_the_mount(devices) {
            return Err(CompactionRunRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        match &*current {
            PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
            PoolVersion::WithoutFile(_) => return Err(CompactionRunRefused::NoVersionOfTrees),
        }
        let parameters = parameters_and_device_table.parameters_on_disk();
        let mut run = run_compaction_on_with_the_region_check_of_the_intent_deletion(
            parameters,
            devices.as_mut_slice(),
            allocator,
            compaction,
            current,
            *instance,
            CompactionRunReach::UntilAStopOrTheBudget,
            region_check,
        );
        run.discards = discard_the_segments_reusable_as_a_whole(
            devices.as_mut_slice(),
            allocator,
            parameters,
            current.root(),
        );
        Ok(run)
    }

    /// [`Self::publish_user_change`] 核过盘表之后那一半：先走空、再发这次改动（unlink 连同它之后的走空），被拒是空间不够就推抬 F 再判。
    /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]），
    /// 所以拆成不收 `self` 的一段。
    fn publish_the_change_raising_the_floor_for_space<Device: BlockDevice>(
        parameters: &MakeFilesystemParameters,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        allocator: &mut PoolAllocator,
        current: &mut PoolVersion,
        instance: InstanceGeneration,
        shadow_ledger: ShadowLedger,
        change: UserChange<'_>,
    ) -> Result<UserChangePublished, UserChangeRefused> {
        let current_version_of_trees = match &*current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees,
            PoolVersion::WithoutFile(_) => {
                return Err(no_file_version_to_change(PublishedInThisChange {
                    floor_raises: Vec::new(),
                    refusals_that_pushed_the_floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                }))
            }
        };
        let mut published_in_this_change = PublishedInThisChange {
            floor_raises: Vec::new(),
            refusals_that_pushed_the_floor_raises: Vec::new(),
            versions_published: Vec::new(),
        };
        // 队列空（任何一版都是常态）时一个写都不发、不复制；不空时在一份复制上走，走到哪一步（连同推的抬 F）`current` 就是哪一版。
        // 销毁中的快照（判决 S4：删快照照 unlink 的过程做，第七节岔路 4）、截断条目（判决 m4-trunc-r1 K1）与 `deleted_inodes` 一起走空。
        if !current_version_of_trees.queue_of_committed_intents_is_empty() {
            let mut walked = current_version_of_trees.clone();
            let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                parameters,
                devices,
                allocator,
                &mut walked,
                instance,
                shadow_ledger,
                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                published_in_this_change,
            );
            *current = PoolVersion::of_a_version_of_trees(walked);
            published_in_this_change = walk?;
        }
        let mut publishes_pushed = 0;
        // 按成员分流（类型上放什么）：有树而文件集合为空的一版只接建 inode。走空把带文件的一版走成这一形时，走空的那几步与推的抬 F
        // 已落盘，带在这个拒绝里（`NoFileVersionToChange` 的三样）。
        let version_to_change = match current {
            PoolVersion::WithFile(file_version) => file_version,
            PoolVersion::WithTreesAndNoFile(emptied_version) => match change {
                UserChange::NewInodes { .. } | UserChange::DestroySnapshot { .. } => {
                    emptied_version
                }
                // 文件集合为空时建快照放行、照常发布，读它读到空（用户 2026-10-16 定「放开」，判决 m4-segment-code-r1 第三节 F5）。
                UserChange::CreateSnapshot => emptied_version,
                // 改 size 同写文件的改动：删完之后只许建 inode、写行与空发布（D8（核心索引结构） 已定项 16 第 5 条）。
                // 按 inode 号写同写第一个文件：删空之后的一版上 inode 树一条记录都没有，没有可写的文件。这一判只罩删空那一版本身；
                // 之后建过 inode、归带文件的那几版上给删完之后才建的 inode 写数据，由写入口按 K-U5 的 β 读法拒
                // （`transaction::PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`）。
                UserChange::Overwrite(_)
                | UserChange::SequentialWrite(_)
                | UserChange::OverwriteOfAnInode { .. }
                | UserChange::SequentialWriteOfAnInode { .. }
                | UserChange::Unlink { .. }
                | UserChange::Truncate { .. } => {
                    return Err(no_file_version_to_change(published_in_this_change));
                }
            },
            PoolVersion::WithoutFile(_) => {
                return Err(no_file_version_to_change(published_in_this_change))
            }
        };
        loop {
            let refusal = match publish_the_change_once(
                parameters,
                devices.as_mut_slice(),
                allocator,
                version_to_change,
                change,
                instance,
            ) {
                Ok(published) => {
                    // unlink：意图发成之后接着把全部批走完（一次调用 = 走空 + 意图 + 全部批）；别的改动之后队列本来就空，一个写都不发。
                    let mut published = published;
                    published_in_this_change
                        .versions_published
                        .push(published.clone());
                    let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                        parameters,
                        devices,
                        allocator,
                        &mut published,
                        instance,
                        shadow_ledger,
                        RemainderOfTheUserChange::WalkTheQueue,
                        published_in_this_change,
                    );
                    *current = PoolVersion::of_a_version_of_trees(published);
                    let PublishedInThisChange {
                        floor_raises,
                        refusals_that_pushed_the_floor_raises,
                        versions_published,
                    } = walk?;
                    return Ok(UserChangePublished {
                        floor_raises,
                        refusals_that_pushed_the_floor_raises,
                        versions_published,
                        compaction_after_the_change: None,
                        discards_after_the_change: DiscardsAfterThePublishes::default(),
                    });
                }
                Err(refusal) => refusal,
            };
            let publishes_of_this_raise;
            (published_in_this_change, publishes_of_this_raise) =
                push_one_floor_raise_for_a_refusal_for_space(
                    parameters,
                    devices,
                    allocator,
                    version_to_change,
                    shadow_ledger,
                    RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                    publishes_pushed,
                    published_in_this_change,
                    refusal,
                )?;
            publishes_pushed += publishes_of_this_raise;
        }
    }

    /// 推环到 R 那一槽（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」那一个管理入口）：在现行那一版后面接连发 F 不变的空发布
    /// （与可写挂载暖机同一张计划，`mount::empty_publish_plan_after_file_version`：重写固定点单元，不碰用户树、实例表照抄），
    /// 直到这个进程写掉 C 那一圈还没解开、记着落点的每一槽（R 那一槽最后轮到），宽护随之解开（`allocator::PoolAllocator::record_root_written_by_this_process`
    /// 摘下那一槽记着的落点、清掉别处不再护着的隔离位）。要几次见 [`StuckBehindTheWideProtection::empty_publishes_until_the_ring_slot_of_r_is_written`]，至多环长 3S。
    ///
    /// **调用即确认丢掉最近可退到的状态**：R 那一槽是 C 那一圈里最后轮到的一槽，推到它要写掉根环里从现行那一版起接下来轮到的那几槽（次数就是槽数，
    /// 不超过一环），那几槽里原来的根（回退候选）换成同一个用户状态的空发布根；推完之后回退候选集可以掉到少于 4 个不同状态，
    /// D16（发布语义） 已定项 1「回退候选集保留最近 4 个可退到的不同状态」只在调这个入口时破
    /// （第 10z 批调查报告 10z-p 第六节第 1 条：384 槽盘那一形推完只剩 2 个）。不自动调：准入被宽护卡住时只报 [`StuckBehindTheWideProtection`]，
    /// 调用方要出路时（[`Self::ways_out_of_the_wide_protection`]）预演过推完之后那次改动放得下才写明这一条出路。
    ///
    /// 第一道照 [`Self::publish_user_change`] 与 [`Self::ways_out_of_the_wide_protection`] 那样，把交进来的盘表与挂载时核过的那一份逐项比
    /// （次序也比，`mount::ParametersAndDeviceTableOfTheMount::is_the_device_table_of_the_mount`，只比身份、不读盘），不符就拒
    /// （`DeviceTableOtherThanTheOneOfTheMount`，带两份盘表），在任何读写之前：三个入口收盘表同一判（用户 2026-10-10 弹窗定「也先比次序」）。
    /// 这一判在这个公开入口里、不在本体（`push_the_root_ring_on`）里：要出路时的预演经本体走、交的是包过的盘，要出路那一边先核过同一判。
    /// 第二道看分配器上冻结着没重发的发布，冻结着就拒（`PublishFrozenAfterAWriteFailureIsNotResentYet`），在核盘表那一套与预演之前。
    /// C 那一圈记着落点、而它们罩着的槽这一刻一个都没隔离时不推（写掉那几槽放不出东西，`NothingHeldUntilThisProcessWritesARingSlot`），在任何写之前。
    /// 整串先在分配器的一份拷贝上预演（发布路径落盘之前那一段，与抬 F 那一串的预演同一个做法）：推到半路空发布取不到落点（没挡的槽不够这一串）
    /// 时一个写都不发，盘上逐字节不变、分配器与会话不动。预演过了才真发；真发时有一次发不出去，前面那几次已落盘、`current` 停在最后一次落盘的那一版上。
    /// 这些空发布不过准入两判（没有普通分配，D28（挂载期承诺量） 已定项 1），只受「取得到落点」约束。交进来的盘表照抬 F 那一串的
    /// 空发布同一套核（`mount::parameters_of_the_pool_agreeing_with_the_caller_inputs`），在任何写之前；写入口只按盘上那一份参数建。
    ///
    /// # Errors
    /// [`RootRingPushRefused`] 的七种。
    pub fn push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states<
        Device: BlockDevice,
    >(
        &mut self,
        devices: &mut Vec<(DeviceIdentity, Device)>,
    ) -> Result<RootRingPushed, RootRingPushRefused> {
        let MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger: _,
            parameters_and_device_table,
            compaction: _,
        } = self;
        let is_the_device_table_handed_in_the_one_of_the_mount =
            parameters_and_device_table.is_the_device_table_of_the_mount(devices);
        if !is_the_device_table_handed_in_the_one_of_the_mount {
            return Err(RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount {
                device_table_of_the_mount: parameters_and_device_table
                    .device_identities_in_table_order()
                    .to_vec(),
                device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
            });
        }
        Self::push_the_root_ring_on(
            parameters_and_device_table.parameters_on_disk(),
            devices,
            allocator,
            current,
            *instance,
        )
    }

    /// 被宽护卡住之后调用方要出路时调（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：照 `stuck` 那一份报记着的「这次改动还剩什么」，
    /// 在分配器的一份拷贝上、对只进内存的盘预演推环与推完之后这次改动剩下的发布（[`rehearse_the_push_then_the_remainder_of_the_change`]：
    /// 与真调推环入口、之后走报里剩下的那一段走同一段：报里还剩这次改动时是再发同一个改动；只剩走空时 unlink 的意图已发成，
    /// 是下一次改动开头的走空，不再发这次改动），照结局列出路（推完放得下才列推环）。报 `stuck` 的那一次不付这一笔。
    ///
    /// 收 `&self`：预演只读交进来的盘、在分配器的拷贝上走，会话的分配器、现行那一版一样都不动——类型上就动不了（`PoolAllocator` 里没有内部可变），
    /// 调用方不必为了要出路把会话交出去。盘表只读借：只进内存的盘只读借底下那块盘，写一个都发不到它上面。
    /// 交进来的盘表先照 [`Self::publish_user_change`] 与推环入口那样与挂载时核过的那一份逐项比（次序也比，三个入口同一判），
    /// 再照推环本体那一套核（在预演的推环里，核不过就是推环被拒、出路只列卸载再挂），与照出路二真推、再走报里剩下的那一段时同一个结局
    /// （报里还剩这次改动时是再发这次改动）。
    ///
    /// 第一道看会话还在不在被拒的那一版上（实例代号与 txg 都与报里记的相同），再比盘表：哪一道不过都不预演，一个盘都不读。
    /// 第一道只比这两个数，认不出 `stuck` 是不是这个会话报的：别的会话的报若这两个数恰好与这个会话的现行那一版相同，照样过，
    /// 照这个会话的状态预演（[`WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion`]）。
    /// 调用方要拿报去问报它的那个会话。
    ///
    /// # Errors
    /// [`WaysOutOfTheWideProtectionRefused`] 的两种。
    pub fn ways_out_of_the_wide_protection<Device: BlockDevice>(
        &self,
        devices: &[(DeviceIdentity, Device)],
        stuck: &StuckBehindTheWideProtection,
    ) -> Result<WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused> {
        let is_the_version_refused_on = |version_of_trees: &TransactionOutput| {
            version_of_trees.root.instance == stuck.instance_of_the_version_refused_on
                && version_of_trees.root.checkpoint_txg
                    == stuck.checkpoint_txg_of_the_version_refused_on
        };
        let version_refused_on = match &self.current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees)
                if is_the_version_refused_on(version_of_trees) =>
            {
                version_of_trees
            }
            PoolVersion::WithFile(_)
            | PoolVersion::WithTreesAndNoFile(_)
            | PoolVersion::WithoutFile(_) => {
                let current_root = self.current.root();
                return Err(
                    WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                        instance_refused_on: stuck.instance_of_the_version_refused_on,
                        checkpoint_txg_refused_on: stuck.checkpoint_txg_of_the_version_refused_on,
                        instance_of_the_current_version: current_root.instance,
                        checkpoint_txg_of_the_current_version: current_root.checkpoint_txg,
                    },
                );
            }
        };
        if !self
            .parameters_and_device_table
            .is_the_device_table_of_the_mount(devices)
        {
            return Err(
                WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount {
                    device_table_of_the_mount: self
                        .parameters_and_device_table
                        .device_identities_in_table_order()
                        .to_vec(),
                    device_table_handed_in: devices.iter().map(|(identity, _)| *identity).collect(),
                },
            );
        }
        let pushing_the_root_ring = rehearse_the_push_then_the_remainder_of_the_change(
            self.parameters_and_device_table.parameters_on_disk(),
            devices,
            &self.allocator,
            version_refused_on,
            self.instance,
            self.shadow_ledger,
            stuck.remainder_of_the_change.as_remainder(),
        );
        let ways_out = stuck.ways_out_after(&pushing_the_root_ring);
        Ok(WaysOutOfTheWideProtection {
            pushing_the_root_ring,
            ways_out,
        })
    }

    /// [`Self::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states`] 的本体，盘上择到的参数由调用方给：
    /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]）。
    ///
    /// # Errors
    /// 同那个入口。
    fn push_the_root_ring_on<Device: BlockDevice>(
        parameters_on_disk: &MakeFilesystemParameters,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        allocator: &mut PoolAllocator,
        current: &mut PoolVersion,
        instance: InstanceGeneration,
    ) -> Result<RootRingPushed, RootRingPushRefused> {
        // D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：冻结着的那次发布等原样重发。下面的整串预演走的是发布路径落盘之前那一段、
        // 不判冻结，在冻结着的分配器拷贝上照样过，真发第一次才被拒（第 10z 批代码轮第一轮攻方第二节「一处报法不准」）。
        if let Some(frozen) = allocator.frozen_publish() {
            return Err(
                RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {
                    checkpoint_txg: frozen.checkpoint_txg(),
                    instance: frozen.instance(),
                },
            );
        }
        let start = match &*current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees.clone(),
            PoolVersion::WithoutFile(_) => {
                return Err(RootRingPushRefused::VersionWithoutFileNotSupportedInTheFirstVersion);
            }
        };
        let parameters_of_the_pool = parameters_of_the_pool_agreeing_with_the_caller_inputs(
            parameters_on_disk,
            devices,
            &start,
        )
        .map_err(|refusal| RootRingPushRefused::DevicesHandedIn(Box::new(refusal)))?;
        let parameters = &parameters_of_the_pool;
        let Some(target) = root_ring_push_target(allocator, start.root.checkpoint_txg) else {
            return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
        };
        // C 那一圈记着落点、它们罩着的槽却一个都没隔离（全在现行账里仍分配、或已释放还在 defer 窗口里）：写掉那几槽放不出东西，
        // 与没有记着落点的槽同一个结局（判决 `research/prompts/m3-batch10z-pqrs-code-r1-main-verification.md` 第二节「K1-1 三个量」那一行，正推注入级那一形）。
        let slots_the_push_would_release_isolation_of =
            allocator.isolated_slots_covered_by(&target.placements);
        if slots_the_push_would_release_isolation_of
            .iter()
            .all(|(_, isolated)| *isolated == 0)
        {
            return Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot);
        }
        let publishes_in_the_push =
            usize::try_from(target.publishes_until_the_ring_slot_of_r_is_written)
                .expect("至多环长 3S ≤ 48 次");
        // 预演：迭代上界是这一串的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）与分配器的拷贝。
        {
            let rehearsal_writer = PoolWriter::new(parameters, devices.as_mut_slice());
            let mut copy = allocator.clone();
            let mut rehearsed = start.clone();
            for publish_index in 0..publishes_in_the_push {
                let (_, next) = prepare_the_version_publish(
                    &rehearsal_writer,
                    &mut copy,
                    &empty_publish_plan_after_file_version(&rehearsed, instance),
                    Some(&rehearsed),
                    rehearsed.tree_identifiers,
                    ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                    UnmountMarker::NotWrittenByTheUnmountSequence,
                    &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
                )
                .map_err(|cause| {
                    RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
                        refused_publish_in_the_push: publish_index + 1,
                        publishes_in_the_push,
                        cause,
                    }
                })?;
                rehearsed = next;
            }
        }
        // 真发：迭代上界同上；跨轮携带的是落盘了的现行那一版与发成的每一版；出口只有发不出去那一次。
        let mut last_published = start;
        let mut versions_published = Vec::with_capacity(publishes_in_the_push);
        for publish_index in 0..publishes_in_the_push {
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            match publish_version(
                &mut writer,
                allocator,
                empty_publish_plan_after_file_version(&last_published, instance),
                Some(&last_published),
            ) {
                Ok(next) => {
                    versions_published.push(next.clone());
                    last_published = next;
                }
                Err(cause) => {
                    *current = PoolVersion::of_a_version_of_trees(last_published);
                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(
                        Box::new(RootRingPushFailedAfterTheRehearsal {
                            failed_publish_in_the_push: publish_index + 1,
                            publishes_in_the_push,
                            cause,
                            versions_published,
                        }),
                    ));
                }
            }
        }
        *current = PoolVersion::of_a_version_of_trees(last_published);
        let still_holding = allocator
            .root_ring_occupancy()
            .expect("推环之前读到了 C 那一圈，装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        assert!(
            target
                .ring_slots
                .iter()
                .all(|ring_slot| !still_holding.contains_key(ring_slot)),
            "推环写到了 R 那一槽：每次发布 txg 加一、写 target_for_publish(txg) 那一槽并在取完落点之后摘下 C 在那一槽记着的落点，\
             R 那一槽是这几槽里最后轮到的；还记着说明有一次发布没经分配器记根"
        );
        Ok(RootRingPushed {
            versions_published,
            ring_slots_written: target.ring_slots,
        })
    }
}

/// 发布返回之后发 discard（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G5，判决 `research/prompts/m4-batch2-code-r1-main-verification.md`
/// 第二节 S6）：「可再分配」谓词为真（已回收、没被抬 F 扣住、没被隔离、F 已生效）的槽按 64 槽对齐的整段发——只发回收过槽、整段可再分配的段
/// （`allocator::PoolAllocator::take_the_segments_to_discard_on_every_device`），每块盘发它自己那几段。discard 取几态、设备日志怎么判不在这里
/// （归设备侧与崩溃点重放）。长度恒为一段（64 × 16384 字节），不发长度 0 的。
///
/// 装着还没清扫的孤儿头的段不发（主 agent 2026-10-15 定，零轮）：discard 的结局是三态之一（D13（验证路线） 已定项 4），不能拿它当擦除——
/// 抹孤儿头归实例表清扫（`instance_table_sweep`），清扫判过、抹过之前段里的孤儿头要原样留着。孤儿头 = 段里某个槽开头认得出本池的单元头
/// （清扫认头的那一判，`instance_table_sweep::unit_head_read_by_sweep`），它的写序实例在现行那一版的实例表里还有一行、按那一行判未发布
/// （`UnitHeadReadBySweep::is_unpublished_by_the_row`）。判之前读不出实例表、读不出段里的一个槽，这一段同样不发（读不到不等于没有）。
pub fn discard_the_segments_reusable_as_a_whole<Device: BlockDevice>(
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    parameters: &MakeFilesystemParameters,
    current_root: &RootRecord,
) -> DiscardsAfterThePublishes {
    let mut discards = DiscardsAfterThePublishes::default();
    let segments_to_discard = allocator.take_the_segments_to_discard_on_every_device();
    if segments_to_discard
        .iter()
        .all(|(_, segment_starts)| segment_starts.is_empty())
    {
        return discards;
    }
    let rows_of_the_instance_table: Option<BTreeMap<InstanceGeneration, InstanceRow>> =
        instance_table_chain_of_root(&*devices, current_root)
            .ok()
            .map(|chain| {
                chain
                    .records
                    .rows
                    .iter()
                    .map(|row| (row.instance, *row))
                    .collect()
            });
    let filesystem_identifier = unit_filesystem_identifier(&parameters.filesystem_identifier);
    for (device_identity, segment_starts) in segments_to_discard {
        for segment_start in segment_starts {
            let holding = match &rows_of_the_instance_table {
                Some(rows) => segment_holding_unswept_orphan_heads(
                    &*devices,
                    device_identity,
                    segment_start,
                    rows,
                    filesystem_identifier,
                ),
                None => SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed,
            };
            match holding {
                SegmentHoldingUnsweptOrphanHeads::No => {}
                SegmentHoldingUnsweptOrphanHeads::Yes => {
                    discards
                        .withheld_holding_unswept_orphan_heads
                        .push((device_identity, segment_start));
                    continue;
                }
                SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed => {
                    discards
                        .withheld_after_a_failed_read
                        .push((device_identity, segment_start));
                    continue;
                }
            }
            let Some((_, device)) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device_identity)
            else {
                continue;
            };
            match device.discard_at(
                segment_start.to_device_offset(),
                CLUSTER_SEGMENT_SLOTS * SLOT_BYTES,
            ) {
                Ok(DiscardOutcome::Applied) => {
                    discards.applied.push((device_identity, segment_start));
                }
                Ok(DiscardOutcome::Unsupported) => {
                    discards.unsupported.push((device_identity, segment_start));
                }
                Err(_cause) => discards.failed.push((device_identity, segment_start)),
            }
        }
    }
    discards
}

/// 一段装没装着还没清扫的孤儿头（[`discard_the_segments_reusable_as_a_whole`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SegmentHoldingUnsweptOrphanHeads {
    No,
    Yes,
    /// 段里一个槽读不出：判不了，按装着算（不发）。
    NotJudgedReadFailed,
}

/// `device` 上从 `segment_start` 起的那一段：逐槽读一整槽，开头认得出本池的单元头、写序实例在 `rows` 里有一行、按那一行判未发布的，就是还没清扫的孤儿头。
/// 认出一个头就按它的整单元跨度跳（同清扫的认领规则）。迭代上界是一段 64 槽；提前出口是认出一个孤儿头与读失败。
fn segment_holding_unswept_orphan_heads<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    device: DeviceIdentity,
    segment_start: SlotNumber,
    rows: &BTreeMap<InstanceGeneration, InstanceRow>,
    filesystem_identifier: u64,
) -> SegmentHoldingUnsweptOrphanHeads {
    let slot_bytes = usize::try_from(SLOT_BYTES).expect("一槽 16384 字节");
    let mut slot = segment_start.0;
    while slot < segment_start.0 + CLUSTER_SEGMENT_SLOTS {
        let Some(bytes) = reader.read(device, SlotNumber(slot).to_device_offset(), slot_bytes)
        else {
            return SegmentHoldingUnsweptOrphanHeads::NotJudgedReadFailed;
        };
        let Some(head) = unit_head_read_by_sweep(&bytes) else {
            slot += 1;
            continue;
        };
        slot += head.span_in_slots();
        if head.filesystem_identifier == filesystem_identifier
            && rows
                .get(&head.writer_instance)
                .is_some_and(|row| head.is_unpublished_by_the_row(row))
        {
            return SegmentHoldingUnsweptOrphanHeads::Yes;
        }
    }
    SegmentHoldingUnsweptOrphanHeads::No
}

/// 一次整理运行推到哪为止。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompactionRunReach {
    /// 显式入口：推到停机原因或发布预算用完。
    UntilAStopOrTheBudget,
    /// 用户改动之后（判决 G8 丁）：至多一批。一批 = 搬了单元的那一次发布（判决 `research/prompts/m4-segment-code-r1-main-verification.md`
    /// 第三节 F2）；不搬单元的发布不算批：上一条意图的收尾（一个没搬就删意图、R 已按记录全空只删意图、丢意图）与签发。所以一趟可以发
    /// 「收尾 + 签发 + 一批」三次；签发之后计划里只剩这次发布本来就重写的节点时，那一次也是一个没搬就删意图，全局差值为真就再签再搬。
    /// 一趟发几次的上界是这一趟的发布预算 `B_gc`（[`publish_budget_of_a_compaction_run`]），每签一条之前照判签发前判与「上一条无产出就不签」
    /// （判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G4、G8；判决 `research/prompts/m4-segment-code-r3-main-verification.md`
    /// 第三节 J3 核过：与 G8 丁「至多一批」、G8 甲「至多签 `B_gc` 条」一致）。
    AtMostOneBatchAfterAUserChange,
    /// 可写挂载续做（D8（核心索引结构） 已定项 5 第 2 条；判决 G1）：只推盘上那一条未完成的意图，推到它删了、停机原因或预算用完，不签新的。
    ContinueTheLiveIntentAtTheMount,
}

/// 整理的一次发布做的是哪一种（[`CompactionStep::Published`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompactionPublishKind {
    /// 签发那次发布：意图写进 logged_ops。
    IntentIssued,
    /// 搬一批，意图照抄。
    Batch,
    /// 最后一批：搬完 R 按记录全空，同一次发布删意图（D8（核心索引结构） 已定项 5）。
    BatchCompletingTheIntent,
    /// R 已按记录全空（别的发布腾空的），单独一次发布删意图。
    IntentCompletedWithoutABatch,
    /// 丢意图：单独一次发布删它（攻方 K-c 的隔离槽、区域扫描拒了 R）。
    IntentDropped,
    /// 丢意图：R 里一个能搬的都没有、却还有未释放的记录；丢完这一趟停（[`CompactionRunStop::NothingMovableButTheRegionIsNotEmpty`]）。
    IntentDroppedWithNothingMovable,
}

/// 推一步整理的结局：发成一次整理发布，或照判据停。
enum CompactionStep {
    Published {
        version: Box<TransactionOutput>,
        kind: CompactionPublishKind,
    },
    Stopped(CompactionRunStop),
}

/// 整理一趟（显式入口、用户改动之后、可写挂载续做共用）：先走空删除续做（G8 次序），再照预算一次一次推（[`advance_compaction_by_one_publish`]）。
/// 预算 `B_gc` 数的是整理的每一次发布（签发、搬一批、删意图：每一次都用掉一个根槽）。discard 由调用方在这一趟之后发。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止"
)]
pub(crate) fn run_compaction_on<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    reach: CompactionRunReach,
) -> CompactionRun {
    run_compaction_on_with_the_region_check_of_the_intent_deletion(
        parameters,
        devices,
        allocator,
        compaction,
        current,
        instance,
        reach,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
    )
}

/// 同 [`run_compaction_on`]，一批腾空了 R、照同一批删意图再发时删意图那一支的核照 `region_check` 判
/// （[`MountedSession::run_compaction_with_the_region_check_of_the_intent_deletion_after_a_batch_emptying_the_region`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、推到哪为止、删意图那一支的核怎么判"
)]
fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    reach: CompactionRunReach,
    region_check: RegionCheckOfTheIntentDeletion,
) -> CompactionRun {
    compaction.observations.runs_started += 1;
    let mut versions_published: Vec<TransactionOutput> = Vec::new();
    let finish = |compaction_of_the_run: &mut CompactionOfThisMount,
                  versions_published_by_the_run: Vec<TransactionOutput>,
                  batches_published: u64,
                  outcome: CompactionRunOutcome| {
        if let CompactionRunOutcome::Stopped(stop) = &outcome {
            compaction_of_the_run.observations.last_stop = Some(*stop);
        }
        CompactionRun {
            versions_published: versions_published_by_the_run,
            batches_published,
            outcome,
            discards: DiscardsAfterThePublishes::default(),
        }
    };
    let switches = allocator.compaction().switches;
    match switches.run_forcing {
        CompactionRunForcing::ForcedToStopForTests => {
            return finish(
                compaction,
                versions_published,
                0,
                CompactionRunOutcome::Stopped(CompactionRunStop::ForcedToStopForTests),
            );
        }
        CompactionRunForcing::FollowTheWatermarks | CompactionRunForcing::ForcedToStartForTests => {
        }
    }
    let batch_units = match switches.batch_size {
        CompactionBatchSize::NotDecidedByTheClauses => {
            return finish(
                compaction,
                versions_published,
                0,
                CompactionRunOutcome::Stopped(CompactionRunStop::BatchSizeNotDecidedByTheClauses),
            );
        }
        CompactionBatchSize::ForcedForTests(batch_units) => batch_units,
    };
    // 次序（判决 G8）：删除续做在整理批之前。
    let queue_is_empty = match &*current {
        PoolVersion::WithFile(version_of_trees)
        | PoolVersion::WithTreesAndNoFile(version_of_trees) => {
            version_of_trees.queue_of_committed_intents_is_empty()
        }
        PoolVersion::WithoutFile(_) => true,
    };
    if !queue_is_empty {
        let mut walked = current
            .version_of_trees()
            .expect("队列不空的一版是有树的一版")
            .clone();
        let walk = publish_the_queue_until_it_is_empty(
            parameters,
            devices,
            allocator,
            &mut walked,
            instance,
        );
        *current = PoolVersion::of_a_version_of_trees(walked);
        versions_published.extend(walk.steps_published);
        if let Err(cause) = walk.stopped_by {
            return finish(
                compaction,
                versions_published,
                0,
                CompactionRunOutcome::Failed(CompactionRunFailure::DeletionContinuation(cause)),
            );
        }
    }
    let budget = publish_budget_of_a_compaction_run(
        parameters.geometry.root_ring_slots_per_region.count(),
        compaction.publishes_since_the_last_user_change,
    );
    let mut publishes_of_the_run: u64 = 0;
    let mut batches_published: u64 = 0;
    // 循环：每一轮发成一次整理发布或停。迭代上界是预算（每发成一次计一次，到 budget 停）；跨轮携带的是现行那一版、上一条完成的意图、
    // 已发的次数与批数。
    let mut last_completed_intent: Option<CompactionIntent> = None;
    loop {
        if publishes_of_the_run >= budget {
            return finish(
                compaction,
                versions_published,
                batches_published,
                CompactionRunOutcome::Stopped(CompactionRunStop::PublishBudgetSpent { budget }),
            );
        }
        if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {
            return finish(
                compaction,
                versions_published,
                batches_published,
                CompactionRunOutcome::Stopped(CompactionRunStop::OneBatchAfterTheUserChangeDone),
            );
        }
        match advance_compaction_by_one_publish(
            parameters,
            devices,
            allocator,
            compaction,
            current,
            instance,
            batch_units,
            reach,
            &mut last_completed_intent,
            region_check,
        ) {
            Ok(CompactionStep::Published { version, kind }) => {
                publishes_of_the_run += 1;
                compaction.publishes_since_the_last_user_change += 1;
                compaction.observations.root_ring_slots_used += 1;
                match kind {
                    // 一批 = 搬了单元的那一次（判决 m4-segment-code-r1 第三节 F2：没搬单元的发布不记成一批——删意图的那一批计划里的单元
                    // 都是这次发布本来就重写的节点时，它就是一次只删意图的发布）。
                    CompactionPublishKind::Batch
                    | CompactionPublishKind::BatchCompletingTheIntent
                        if !version.units_relocated_by_this_publish.is_empty() =>
                    {
                        batches_published += 1;
                        compaction.observations.batches_published += 1;
                        let relocated =
                            u64::try_from(version.units_relocated_by_this_publish.len())
                                .expect("一批的单元数");
                        compaction.observations.units_relocated += relocated;
                        compaction.observations.units_relocated_by_the_last_batch = relocated;
                    }
                    CompactionPublishKind::Batch
                    | CompactionPublishKind::BatchCompletingTheIntent
                    | CompactionPublishKind::IntentIssued
                    | CompactionPublishKind::IntentCompletedWithoutABatch
                    | CompactionPublishKind::IntentDropped
                    | CompactionPublishKind::IntentDroppedWithNothingMovable => {}
                }
                versions_published.push(*version);
                if kind == CompactionPublishKind::IntentDroppedWithNothingMovable {
                    return finish(
                        compaction,
                        versions_published,
                        batches_published,
                        CompactionRunOutcome::Stopped(
                            CompactionRunStop::NothingMovableButTheRegionIsNotEmpty,
                        ),
                    );
                }
            }
            Ok(CompactionStep::Stopped(stop)) => {
                return finish(
                    compaction,
                    versions_published,
                    batches_published,
                    CompactionRunOutcome::Stopped(stop),
                );
            }
            Err(refused) => {
                return finish(
                    compaction,
                    versions_published,
                    batches_published,
                    CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),
                );
            }
        }
    }
}

/// 一版里整理要读的几样：每个进映射单元的映射条目（按 key 在这一版的映射树里查，连同留存的），钉住的槽（豁免映射的三类：映射树各节点、
/// 树表、实例表链——根记录直接指着它们），码 1 数据单元落着的槽（签发前判「只装提交内生块的段不算非全空」用）。
struct VersionAsTheCompactionReadsIt {
    mapping_entries: Vec<CentralMappingEntry>,
    pinned_slots: BTreeSet<SlotNumber>,
    slots_holding_user_data: BTreeSet<SlotNumber>,
}

/// 整理读一版时中央映射那一半（[`central_mapping_as_the_compaction_reads_it`]）。
pub struct CentralMappingAsTheCompactionReadsIt {
    /// 每个进映射单元按 key 在这一版的映射树里查到的映射条目，按 `mapped_units` 的次序（查不到、条目窄于字段表的不收），
    /// 后面接这一版留存的映射条目，按留存的次序。
    pub mapping_entries: Vec<CentralMappingEntry>,
    /// 取上面那些条目时批量读者解过几片映射叶（[`CentralMappingOfAVersionReadLeafByLeaf::leaves_decoded`]）。
    pub mapping_leaves_decoded: usize,
}

/// 整理读一版时取映射条目的那一步：整理选段与搬单元读版本（`version_as_the_compaction_reads_it`）经它取，别处不调。
/// 公开给测试：七·6 的用例拿它逐项比逐 key 查（`central_mapping_locations_in_the_version`）的参照、数整理读一版解了几片叶，
/// 判整理读漏、读错与退回逐 key 查（七·6 代码轮第一轮判决第三节 T4）。
#[must_use]
pub fn central_mapping_as_the_compaction_reads_it(
    version: &TransactionOutput,
) -> CentralMappingAsTheCompactionReadsIt {
    // 每把 key 查一次，叶按片解一次（七·6）：答案与逐 key 查 `central_mapping_locations_in_the_version` 逐个相同。
    let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);
    let mut mapping_entries: Vec<CentralMappingEntry> = version
        .mapped_units
        .iter()
        .filter_map(|(_, key)| match central_mapping.locations_of_key(key) {
            MappingLookup::Found(locations) => Some(CentralMappingEntry {
                key: key.clone(),
                locations,
            }),
            MappingLookup::NodeMalformedOrNoEntryWithThisKey
            | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
        })
        .collect();
    mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(
        |(key, locations)| CentralMappingEntry {
            key: key.clone(),
            locations: *locations,
        },
    ));
    CentralMappingAsTheCompactionReadsIt {
        mapping_entries,
        mapping_leaves_decoded: central_mapping.leaves_decoded(),
    }
}

fn version_as_the_compaction_reads_it(
    version: &TransactionOutput,
) -> VersionAsTheCompactionReadsIt {
    let CentralMappingAsTheCompactionReadsIt {
        mapping_entries,
        mapping_leaves_decoded: _,
    } = central_mapping_as_the_compaction_reads_it(version);
    let mut pinned_slots = BTreeSet::new();
    for unit in &version.units {
        let is_exempt_from_the_mapping = match unit.identity {
            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTree
            | TransactionUnit::TreeTable
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => true,
            TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::Data(_)
            | TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentRoot
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InodeRoot
            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            | TransactionUnit::AllocationTree
            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::AccountingTree
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree
            | TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_) => false,
        };
        if is_exempt_from_the_mapping {
            for slot in unit.slot.0..unit.slot.0 + unit.identity.span_slots() {
                pinned_slots.insert(SlotNumber(slot));
            }
        }
    }
    let mut slots_holding_user_data = BTreeSet::new();
    for entry in &mapping_entries {
        if entry.key.first() == Some(&UNIT_CLASS_DATA) {
            let slot = entry.locations[0].slot;
            slots_holding_user_data.insert(slot);
            slots_holding_user_data.insert(SlotNumber(slot.0 + 1));
        }
    }
    VersionAsTheCompactionReadsIt {
        mapping_entries,
        pinned_slots,
        slots_holding_user_data,
    }
}

/// 在一个新开的写入口上发整理的一次发布（`transaction::publish_compaction_step`），接在 `previous` 后面；删意图那一支照记录核。
fn publish_the_compaction_step_once<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
        parameters,
        devices,
        allocator,
        previous,
        planned,
        intent_step,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        instance,
    )
}

/// 同 [`publish_the_compaction_step_once`]，删意图那一支的核照 `region_check_of_the_intent_deletion` 判
/// （[`publish_compaction_step_with_the_region_check_of_the_intent_deletion`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、上一版、计划、对意图做什么、删意图那一支的核怎么判、实例代号"
)]
fn publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, RelocationBatchRefused> {
    let switches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    publish_compaction_step_with_the_region_check_of_the_intent_deletion(
        &mut writer,
        allocator,
        previous,
        planned,
        intent_step,
        region_check_of_the_intent_deletion,
        instance,
    )
}

/// 整理的一次发布被拒是不是空间不够的那几种（[`refusal_is_short_of_space`]）：是就暂停、不推抬 F（判决 G8 g1）。
fn compaction_publish_refused_short_of_space(refused: &RelocationBatchRefused) -> bool {
    match refused {
        RelocationBatchRefused::Publish(cause) => refusal_is_short_of_space(cause),
        RelocationBatchRefused::PlannedUnitNotInTheVersion { .. }
        | RelocationBatchRefused::PinnedUnitInThePlan { .. }
        | RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition { .. }
        | RelocationBatchRefused::UnitBytesNotAvailable { .. }
        | RelocationBatchRefused::CompactionIntentAlreadyLive { .. }
        | RelocationBatchRefused::NoLiveCompactionIntent
        | RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
            ..
        }
        | RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried { .. } => {
            false
        }
    }
}

/// 推整理一次发布（判决 G1–G4、G8 的整串；意图住 logged_ops，判决 G1）。活意图读现行那一版的 logged_ops（盘上的权威），
/// 分配器这一侧的 R 排他与它一致（会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上）。先把发布路径隔离过的槽并进常驻隔离集。
/// 1. 没有活意图：可写挂载续做停在「没有活意图可续」（只续做、不签新的）；上一条完成的意图全局差值为假就停（攻方 K-h）；选段（强制选的那一段，
///    或 greedy 先扫后选），选不出就停；签发前判 R 外非全空段的空洞 ≥ R 里要搬的单元数（攻方 K-g），不够就停；签发：先在分配器上排他 R
///    （签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布（`c0` = 现行那一版的 txg，`segs0` = 签发那次发布之前、
///    现行那一版 R 那块盘按记录口径的全空段数，判决 m4-segment-code-r2 第三节 H3，用户 2026-10-16 定「取签发之前」）；
///    被空间拒就撤排他、停（意图没签成）。
/// 2. 有活意图：暂停过的重新排他；R 里出现隔离槽（直接查常驻隔离集，钉住与隔离同时成立也算，丁 1 交判第 4 点）、区域扫描拒了 R、扫出隔离类，
///    都发一次只删意图的发布丢掉它（攻方 K-c）；对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）、计划一批（跳过这次发布本来就重写的节点、补上 R 里
///    真要搬的单元，判决 m4-segment-code-r2 第三节 H4）：要搬就发——这一批罩住 R 里每一条未释放的记录时同一次发布删意图（D8（核心索引结构）
///    已定项 5；R 里只剩这次发布本来就重写的节点时也是这一支；发布在算好的账上核 R 按记录全空，核不过照这一批不删意图再发）；别的批照抄意图、
///    在算好的账上核 R 还有未释放的记录，这一批腾空了 R（计划外的、这次发布本来就重写的节点也离开 R）就照同一批删意图再发（判决 m4-segment-code-r1
///    第三节 F2），R 还不空而计划里有本来就重写、不另搬的单元就跳过它们再计划，所以照抄意图的一批至少真搬一个；一个都没搬的不记成一批；
///    R 已按记录全空就发一次只删意图的发布；
///    一个能搬的都没有而 R 不空就发一次只删意图的发布丢掉它、这一趟停。整理的发布被空间拒就暂停（撤 R 排他、意图留在盘上，攻方 K-k）、停。
///
/// 一次调用至多算 R 里能搬的单元数加三次账、发一次（每跳过一次本来就重写的单元再算一次；照抄那一支核出腾空了 R 换删意图再算，删意图那一支核不过
/// 换照抄再算），选段那一圈的上界是单元区的整段数。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、上一条完成的意图、一批腾空了 R 之后删意图那一支的核怎么判"
)]
fn advance_compaction_by_one_publish<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    compaction: &mut CompactionOfThisMount,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    batch_units: CompactionBatchUnits,
    reach: CompactionRunReach,
    last_completed_intent: &mut Option<CompactionIntent>,
    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
) -> Result<CompactionStep, RelocationBatchRefused> {
    compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());
    let version = current
        .version_of_trees()
        .expect("整理只在有树的一版上推：入口判过，整理的发布与删除续做都交回有树的一版")
        .clone();
    let live_on_disk = live_compaction_intent_of_the_entries(&version.logged_operations);
    assert_eq!(
        allocator
            .compaction()
            .live_intent()
            .map(|live| live.intent),
        live_on_disk.map(|(_, intent)| intent),
        "分配器这一侧的活意图就是现行那一版 logged_ops 里那一条：会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上"
    );
    let unit_area = {
        let device_map = allocator
            .devices
            .first()
            .expect("池里至少一块盘：两盘同槽，选段与判类按第一块盘的记录做（判决 G1 攻方 K-n）");
        UnitAreaOfOneDevice {
            device: device_map.device,
            start: device_map.unit_area_start(),
            slots: device_map.unit_area_slots(),
        }
    };
    let read_of_the_version = version_as_the_compaction_reads_it(&version);
    let quarantined_slots = compaction.quarantined_slots().clone();
    let scan_of = |region: CompactionRegion, devices_to_read: &[(DeviceIdentity, Device)]| {
        scan_region(
            region,
            &version.allocation_records,
            &read_of_the_version.mapping_entries,
            &read_of_the_version.pinned_slots,
            &quarantined_slots,
            |device, slot, bytes| devices_to_read.read(device, slot.to_device_offset(), bytes),
        )
    };
    let Some((_issued_txg, intent)) = live_on_disk else {
        if reach == CompactionRunReach::ContinueTheLiveIntentAtTheMount {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::NoLiveIntentLeftToContinue,
            ));
        }
        if let Some(completed) = *last_completed_intent {
            let empty_now =
                empty_cluster_segments_by_the_records(&version.allocation_records, unit_area);
            if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {
                return Ok(CompactionStep::Stopped(
                    CompactionRunStop::NoYieldFromTheLastIntent {
                        empty_cluster_segments_now: empty_now,
                        empty_cluster_segments_at_issue: completed.empty_cluster_segments_at_issue,
                    },
                ));
            }
        }
        // 钉住的槽与常驻隔离集里的槽都让一段没资格：带隔离槽的段签了也立刻要丢（攻方 K-c）。
        let slots_that_disqualify_a_segment: BTreeSet<SlotNumber> = read_of_the_version
            .pinned_slots
            .union(&quarantined_slots)
            .copied()
            .collect();
        let chosen_scan: Option<RegionScan> = match allocator.compaction().switches.forced_region {
            Some(forced_region) => {
                let is_disqualified = allocator
                    .cluster_segments()
                    .contains(&forced_region.start_slot())
                    || slots_that_disqualify_a_segment
                        .range(forced_region.start_slot()..forced_region.end_slot())
                        .next()
                        .is_some()
                    || compaction
                        .segments_refused_by_the_region_scan
                        .contains(forced_region.start_slot());
                match (is_disqualified, scan_of(forced_region, &*devices)) {
                    (false, Ok(scan)) if scan.movable_units().next().is_some() => Some(scan),
                    (false, Ok(_)) | (true, _) => None,
                    (false, Err(refusal)) => {
                        compaction
                            .segments_refused_by_the_region_scan
                            .note_refusal(forced_region, refusal);
                        None
                    }
                }
            }
            None => {
                let mut region_scans: Vec<RegionScan> =
                    region_scans_of_the_segments_without_an_unreleased_record(
                        &version.allocation_records,
                        unit_area,
                    );
                // 迭代上界：每答一次「先扫」下一次就多一段有结论（扫成了进 region_scans，被拒了进不合格段表），不超过单元区的整段数。
                loop {
                    match choose_region(
                        &version.allocation_records,
                        unit_area,
                        allocator.cluster_segments(),
                        None,
                        &slots_that_disqualify_a_segment,
                        &region_scans,
                        &compaction.segments_refused_by_the_region_scan,
                        RegionChoiceCriterion::Greedy,
                    ) {
                        RegionChoice::Chosen(region) => {
                            break region_scans.into_iter().find(|scan| scan.region == region);
                        }
                        RegionChoice::ScanBeforeChoosing(region) => {
                            match scan_of(region, &*devices) {
                                Ok(scan) => region_scans.push(scan),
                                Err(refusal) => compaction
                                    .segments_refused_by_the_region_scan
                                    .note_refusal(region, refusal),
                            }
                        }
                        RegionChoice::NoEligibleSegment => break None,
                    }
                }
            }
        };
        let Some(scan) = chosen_scan else {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::NoEligibleSegment,
            ));
        };
        let units_to_move = u64::try_from(scan.movable_units().count()).expect("一段 64 槽");
        let device_map = allocator
            .devices
            .first()
            .expect("池里至少一块盘（上面取单元区时判过）");
        let holes = holes_outside_the_region_in_segments_holding_user_data(
            &version.allocation_records,
            &read_of_the_version.slots_holding_user_data,
            unit_area,
            scan.region,
            |slot| device_map.is_free(slot),
        );
        if !holes_admit_the_issue(holes, units_to_move) {
            return Ok(CompactionStep::Stopped(
                CompactionRunStop::HolesOutsideTheRegionFewerThanItsUnitsToMove {
                    holes,
                    units_to_move,
                },
            ));
        }
        let issued = CompactionIntent {
            region: scan.region,
            allocation_generation_at_issue: version.root.checkpoint_txg,
            empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(
                &version.allocation_records,
                unit_area,
            ),
        };
        // R 从签发那次发布起就排他：那次发布自己的新落点也不进 R。没发成就撤掉（盘上没有这条意图）。
        allocator.compaction_mut().issue(issued);
        return match publish_the_compaction_step_once(
            parameters,
            devices,
            allocator,
            &version,
            &[],
            CompactionIntentStep::Issue(issued),
            instance,
        ) {
            Ok(published) => {
                compaction.observations.intents_issued += 1;
                *current = PoolVersion::of_a_version_of_trees(published.clone());
                Ok(CompactionStep::Published {
                    version: Box::new(published),
                    kind: CompactionPublishKind::IntentIssued,
                })
            }
            Err(refused) => {
                allocator.compaction_mut().withdraw_the_live_intent();
                if compaction_publish_refused_short_of_space(&refused) {
                    Ok(CompactionStep::Stopped(
                        CompactionRunStop::IssueRefusedShortOfSpace,
                    ))
                } else {
                    Err(refused)
                }
            }
        };
    };
    let is_the_region_excluded = allocator
        .compaction()
        .live_intent()
        .is_some_and(|live| live.is_the_region_excluded_from_placement);
    if !is_the_region_excluded {
        allocator.compaction_mut().resume_the_live_intent();
    }
    let region = intent.region;
    let scan = if region_holds_a_quarantined_slot(region, &quarantined_slots) {
        None
    } else {
        match scan_of(region, &*devices) {
            Ok(scan)
                if !scan
                    .allocations
                    .iter()
                    .any(|allocation| allocation.class == AllocationInRegion::Quarantined) =>
            {
                Some(scan)
            }
            Ok(_scan_with_a_quarantined_slot) => None,
            Err(refusal) => {
                compaction
                    .segments_refused_by_the_region_scan
                    .note_refusal(region, refusal);
                None
            }
        }
    };
    // 这一次发布要搬哪几个、对意图做什么、发成了是哪一种：计划跳过这次发布本来就重写的节点、补上 R 里真要搬的单元（判决
    // `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16 定「规划时补足」）。照抄意图的一批在算好的账上
    // 核计划里每个单元都真搬，核出本来就重写的那几把 key 就并进跳过的集合、再计划一次（[`compaction_publish_planned_for_the_region_scan`]）。
    // 迭代上界是 R 里能搬的单元数加一：被拒时点名的 key 都在这一次的计划里，计划不取已跳过的，所以每被拒一次跳过的集合至少多一把 key；
    // 能搬的都跳过之后计划要么罩住 R 里每一条记录（删意图那一支，不再核这一样），要么没有可搬的。跨轮携带的只有跳过的集合。
    let mut units_this_publish_rewrites_anyway: BTreeSet<Vec<u8>> = BTreeSet::new();
    let (published, kind) = loop {
        let (planned, intent_step, kind) = compaction_publish_planned_for_the_region_scan(
            scan.as_ref(),
            batch_units,
            &units_this_publish_rewrites_anyway,
            &version.allocation_records,
            region,
        );
        match publish_the_planned_compaction_step_falling_back_on_the_region_check(
            parameters,
            devices,
            allocator,
            &version,
            &planned,
            intent_step,
            kind,
            region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
            instance,
        ) {
            Ok(published_and_kind) => break published_and_kind,
            Err(CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish {
                mapping_keys,
            }) => {
                let skipped_before = units_this_publish_rewrites_anyway.len();
                units_this_publish_rewrites_anyway.extend(mapping_keys);
                assert!(
                    units_this_publish_rewrites_anyway.len() > skipped_before,
                    "被拒时点名的 key 都在这一次的计划里，计划不取已跳过的：跳过的集合每被拒一次至少多一把 key"
                );
            }
            Err(CompactionStepRefused::Refused(refused)) => {
                return pause_or_hand_up(allocator, refused)
            }
        }
    };
    match kind {
        CompactionPublishKind::BatchCompletingTheIntent
        | CompactionPublishKind::IntentCompletedWithoutABatch => {
            allocator.compaction_mut().withdraw_the_live_intent();
            compaction.observations.intents_completed += 1;
            *last_completed_intent = Some(intent);
        }
        CompactionPublishKind::IntentDropped
        | CompactionPublishKind::IntentDroppedWithNothingMovable => {
            allocator.compaction_mut().withdraw_the_live_intent();
            compaction.observations.intents_dropped += 1;
        }
        CompactionPublishKind::Batch | CompactionPublishKind::IntentIssued => {}
    }
    *current = PoolVersion::of_a_version_of_trees(published.clone());
    Ok(CompactionStep::Published {
        version: Box::new(published),
        kind,
    })
}

/// 有活意图时这一次整理发布的计划：要搬哪几个、对意图做什么、发成了是哪一种（[`advance_compaction_by_one_publish`]）。`scan` 是对 R 重扫的结果
/// （R 里有隔离槽、区域扫描拒了 R、扫出隔离类时是 `None`），`rewritten_anyway` 是这一次发布上已试算出的、本来就重写而不另搬的映射 key
/// （[`plan_batch_skipping_the_units_this_publish_rewrites_anyway`]）。
fn compaction_publish_planned_for_the_region_scan(
    scan: Option<&RegionScan>,
    batch_units: CompactionBatchUnits,
    rewritten_anyway: &BTreeSet<Vec<u8>>,
    allocation_records_of_the_version: &[AllocationRecord],
    region: CompactionRegion,
) -> (
    Vec<PlannedRelocation>,
    CompactionIntentStep,
    CompactionPublishKind,
) {
    match scan.map(|scan| {
        (
            scan,
            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
                scan,
                batch_units,
                rewritten_anyway,
            ),
        )
    }) {
        // R 里有隔离槽、区域扫描拒了 R、扫出隔离类：丢意图（攻方 K-c）。
        None => (
            Vec::new(),
            CompactionIntentStep::Drop,
            CompactionPublishKind::IntentDropped,
        ),
        Some((scan, PlannedBatch::Relocate(planned))) => {
            // 这一批罩住 R 里每一条未释放的记录（搬走，或 R 里只剩这次发布本来就重写的节点）：最后一批，同一次发布删意图
            // （D8（核心索引结构） 已定项 5；判决 m4-segment-code-r1 第三节 F2）。
            if planned.len() == scan.allocations.len() {
                (
                    planned,
                    CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                    CompactionPublishKind::BatchCompletingTheIntent,
                )
            } else {
                // 照抄意图那一支在算好的账上核两样：R 被这一批腾空了（计划外的、这次发布本来就重写的节点也离开 R）就照同一批删意图再发
                // （判决 m4-segment-code-r1 第三节 F2）；计划里有本来就重写、不另搬的就跳过它们再计划（判决 m4-segment-code-r2 第三节 H4）。
                (
                    planned,
                    CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,
                    CompactionPublishKind::Batch,
                )
            }
        }
        Some((_scan, PlannedBatch::RegionEmptied)) => {
            assert!(
                region_is_empty_by_the_records(allocation_records_of_the_version, region),
                "扫描里一条未释放记录都没有，R 就按记录全空：扫描与停机谓词读的是同一份记录"
            );
            (
                Vec::new(),
                CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                CompactionPublishKind::IntentCompletedWithoutABatch,
            )
        }
        Some((_scan, PlannedBatch::NothingMovableButTheRegionIsNotEmpty)) => (
            Vec::new(),
            CompactionIntentStep::Drop,
            CompactionPublishKind::IntentDroppedWithNothingMovable,
        ),
    }
}

/// 发这一次计划好的整理发布，删意图那一支或照抄那一支在算好的账上核不过时换个对意图的做法再算（[`advance_compaction_by_one_publish`]）。
/// 照抄意图那一支经 [`publish_compaction_step_in_the_session`] 发，计划里有这次发布本来就重写的单元就不发、交回
/// [`CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish`]（判决 m4-segment-code-r2 第三节 H4）。发成交回这一版与发成的是哪一种；
/// 别的被拒原样交回。一次调用至多算三次账、发一次。
///
/// 两条「核不过就照抄意图」的退路在产品路径上走得到（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H2）：
/// 计划里的单元在这次发布里都离开 R（搬走，或本来就重写），删意图那一核不过只有一种来历——释放之前读盘核把一份隔离、那条记录留在已分配，
/// 也就是一份副本读与重读都读不出（瞬时读错）；下一遍算账再读时它又读得出，照抄意图那一版里 R 就按记录全空了，下一次发布照「R 已按记录全空」
/// 只删意图。另外两处走得到：只供测试的开关关掉 R 排他、这次发布的新落点落进 R；只供测试的 `region_check` 强制删意图那一核不过。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、计划、对意图做什么、发成了是哪一种、一批腾空了 R 之后删意图那一支的核怎么判、实例代号"
)]
fn publish_the_planned_compaction_step_falling_back_on_the_region_check<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    version: &TransactionOutput,
    planned: &[PlannedRelocation],
    intent_step: CompactionIntentStep,
    kind: CompactionPublishKind,
    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
    instance: InstanceGeneration,
) -> Result<(TransactionOutput, CompactionPublishKind), CompactionStepRefused> {
    let switches = allocator.deletion_switches();
    let mut writer = PoolWriter::new(parameters, devices);
    writer.set_container_tree_capacities(switches.container_tree_capacities());
    let first_try = publish_compaction_step_in_the_session(
        &mut writer,
        allocator,
        version,
        planned,
        intent_step,
        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
        PlannedUnitsRewrittenAnyway::RefusedSoThatTheSessionPlansAgain,
        instance,
    );
    match first_try {
        Ok(published) => Ok((published, kind)),
        // 最后一批在算好的账上 R 还不按记录全空（一份副本读与重读都读不出而被隔离，那条记录留在已分配；或只供测试的开关关掉排他、
        // 这一批的新落点落进了 R）：这一批不删意图，照搬、照抄意图。
        Err(CompactionStepRefused::Refused(
            RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                ..
            },
        )) if kind == CompactionPublishKind::BatchCompletingTheIntent => {
            match publish_the_compaction_step_once(
                parameters,
                devices,
                allocator,
                version,
                planned,
                CompactionIntentStep::CarryTheIntent,
                instance,
            ) {
                Ok(published) => Ok((published, CompactionPublishKind::Batch)),
                Err(refused) => Err(refused.into()),
            }
        }
        // 这一批在算好的账上腾空了 R（计划外的、这次发布本来就重写的节点也离开 R）：同一批删意图再发（判决 m4-segment-code-r1 第三节 F2）。
        // 删意图那一支再核 R 按记录全空：两次只差 logged_ops 那一条，取落点不读它，核不过的来历同上一支（瞬时读错、关掉排他），
        // 照这一批不删意图再发（用例经只供测试的 `RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`
        // 或故障注入的瞬时读错强制进入）。
        Err(CompactionStepRefused::Refused(
            RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried { .. },
        )) if kind == CompactionPublishKind::Batch => {
            match publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
                parameters,
                devices,
                allocator,
                version,
                planned,
                CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
                instance,
            ) {
                Ok(published) => Ok((published, CompactionPublishKind::BatchCompletingTheIntent)),
                Err(
                    RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                        ..
                    },
                ) => match publish_the_compaction_step_once(
                    parameters,
                    devices,
                    allocator,
                    version,
                    planned,
                    CompactionIntentStep::CarryTheIntent,
                    instance,
                ) {
                    Ok(published) => Ok((published, CompactionPublishKind::Batch)),
                    Err(refused) => Err(refused.into()),
                },
                Err(refused) => Err(refused.into()),
            }
        }
        Err(refused) => Err(refused),
    }
}

/// 整理的一次发布没发成（活意图在盘上）：空间不够就暂停——撤 R 排他、意图留着（攻方 K-k），不推抬 F（判决 G8 g1）；别的原样交回。
fn pause_or_hand_up(
    allocator: &mut PoolAllocator,
    refused: RelocationBatchRefused,
) -> Result<CompactionStep, RelocationBatchRefused> {
    if compaction_publish_refused_short_of_space(&refused) {
        allocator.compaction_mut().pause_the_live_intent();
        Ok(CompactionStep::Stopped(
            CompactionRunStop::PausedShortOfSpace,
        ))
    } else {
        Err(refused)
    }
}

/// 现行那一版没有可改的文件：拒成 [`UserChangeRefused::NoFileVersionToChange`]，带上这次改动里已经落盘的（走空的那几步与推的抬 F）。
fn no_file_version_to_change(published_in_this_change: PublishedInThisChange) -> UserChangeRefused {
    let PublishedInThisChange {
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
    } = published_in_this_change;
    UserChangeRefused::NoFileVersionToChange {
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
    }
}

/// 这次用户改动里已经落盘的：推了的几串抬 F（按先后）与推它们的那几次被拒（一一对应），推的抬 F 之外发成的每一版（按先后）。
struct PublishedInThisChange {
    floor_raises: Vec<RaisedFloor>,
    refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    versions_published: VersionsPublishedInThisChange,
}

/// 一次发布被拒之后会话怎么接（D16（发布语义） 已定项 1「准入」那一行）：不是空间不够的那两种（[`refusal_is_short_of_space`]）原样交回 `Publish`；
/// 是就为被拒的那一次发布推一串抬 F 的空发布（这次准入已推 `publishes_pushed_in_this_admission` 次，预算照
/// `mount::push_one_floor_raise_within_the_admission_budget`），推成交回记上这一串之后的 `published_in_this_change` 与这一串推了几次空发布，
/// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」，报的话记下这次改动剩下的 `remainder`，要出路时照它预演，
/// [`wide_protection_at_the_ceiling`]），抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
/// 推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、这次改动还剩什么、预算已用的数、已落盘的、这一次被拒，不成一个概念"
)]
fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
    publishes_pushed_in_this_admission: usize,
    published_in_this_change: PublishedInThisChange,
    refusal: PublishError,
) -> Result<(PublishedInThisChange, usize), UserChangeRefused> {
    let PublishedInThisChange {
        mut floor_raises,
        mut refusals_that_pushed_the_floor_raises,
        versions_published,
    } = published_in_this_change;
    if !refusal_is_short_of_space(&refusal) {
        return Err(UserChangeRefused::Publish {
            cause: refusal,
            floor_raises,
            versions_published,
        });
    }
    match push_one_floor_raise_within_the_admission_budget(
        parameters,
        devices,
        allocator,
        current,
        shadow_ledger,
        publishes_pushed_in_this_admission,
    ) {
        Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
            let publishes_of_this_raise = raised.publishes.len();
            floor_raises.push(raised);
            refusals_that_pushed_the_floor_raises.push(refusal);
            Ok((
                PublishedInThisChange {
                    floor_raises,
                    refusals_that_pushed_the_floor_raises,
                    versions_published,
                },
                publishes_of_this_raise,
            ))
        }
        Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => Err(
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(Box::new(NoSpaceAfterRaisingTheFloor {
                last_refusal: refusal,
                floor_raises,
                refusals_that_pushed_the_floor_raises,
                // 先借 `stop` 判这一形，再把它挪进下一个字段（字段按写的次序求值）。
                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(
                    allocator, current, remainder, &stop,
                ),
                stop,
                versions_published,
            })),
        ),
        Err(cause) => Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(
            Box::new(FloorRaiseFailedWhilePushingForSpace {
                cause,
                refusal_that_pushed_the_failed_raise: refusal,
                floor_raises,
                refusals_that_pushed_the_floor_raises,
                versions_published,
            }),
        )),
    }
}

/// 把 `current` 的 `deleted_inodes` 走空（[`publish_the_queue_until_it_is_empty`]），走空里的删除类发布被拒是空间不够时
/// 推一串抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」：删除类发布不判式子那一判，落得下那一判照判，
/// 被它以空间拒时推抬 F 腾得出槽）；推满仍放不下、抬 F 自己报错、被拒不是空间不够，交回会话要报的那一种拒绝。
/// 每一次被拒的删除类发布各算一次准入（与 unlink 意图那次、别的用户改动那次一样，预算数的是为被判的那一次发布推了几次空发布）：
/// 这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，这次准入已推的数从 0 数起。`current` 停在最后一次落盘的那一版上
/// （删除那一步或推的空发布）；发成的每一步记进 `versions_published`。`remainder` 是走空之后这次改动还剩什么（推满仍不够、被宽护卡住时
/// 记进报里，要出路时预演推环之后照它再发一次，[`wide_protection_at_the_ceiling`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动还剩什么、已落盘的，不成一个概念"
)]
fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
    published_in_this_change: PublishedInThisChange,
) -> Result<PublishedInThisChange, UserChangeRefused> {
    let mut published_in_this_change = published_in_this_change;
    let mut publishes_pushed_in_this_admission = 0;
    // 迭代上界：每一轮要么走空交回，要么交回拒绝，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
    // 要么这一趟有一步落了盘（队首那个文件少了单元或删掉了，队列与单元数都有限）。跨轮携带的是 `current` 与已落盘的。
    loop {
        let DeletionQueueWalk {
            steps_published,
            stopped_by,
        } = publish_the_queue_until_it_is_empty(
            parameters,
            devices.as_mut_slice(),
            allocator,
            current,
            instance,
        );
        if !steps_published.is_empty() {
            publishes_pushed_in_this_admission = 0;
        }
        published_in_this_change
            .versions_published
            .extend(steps_published);
        let refusal = match stopped_by {
            Ok(()) => return Ok(published_in_this_change),
            Err(refusal) => refusal,
        };
        let publishes_of_this_raise;
        (published_in_this_change, publishes_of_this_raise) =
            push_one_floor_raise_for_a_refusal_for_space(
                parameters,
                devices,
                allocator,
                current,
                shadow_ledger,
                remainder,
                publishes_pushed_in_this_admission,
                published_in_this_change,
                refusal,
            )?;
        publishes_pushed_in_this_admission += publishes_of_this_raise;
    }
}

#[cfg(test)]
mod tests {
    //! 只进内存的盘（[`DeviceKeepingWritesInMemory`]）：写与写零只进内存、盖在之后的读上，读没写过的块穿到底下那块盘，底下那块盘一字不动；
    //! 请求判据与两个后端是同一份（`block_device::check_aligned_and_in_range`）。差分对拍照正推腿的做法
    //! （`research/prompts/m3-batch10z-tu-code-r1-sonnet-model/src/core-mounted-session-debug.diff` 的 `sonnet_k5_differential`），
    //! 参照换成一块把每个写直接写进去的字节盘（同一份请求判据）。
    use super::*;

    /// 物理块 512 字节、16 块的盘。
    const PHYSICAL_BLOCK_BYTES: u32 = 512;
    const BLOCKS_OF_THE_DISK: u64 = 16;

    /// 底下那块盘（也是差分对拍的参照）：整块字节放内存，请求判据照两个后端那一份。
    struct DeviceOfBytes {
        bytes: Vec<u8>,
        physical_block_size: PhysicalBlockSizeInBytes,
    }

    impl DeviceOfBytes {
        /// 物理块 [`PHYSICAL_BLOCK_BYTES`]、[`BLOCKS_OF_THE_DISK`] 块的盘，每个字节按位置与 `seed` 填一个图样：读回哪一块看得出来。
        fn filled_with_a_pattern(seed: u8) -> Self {
            Self::filled_with_a_pattern_on_blocks_of(
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
                BLOCKS_OF_THE_DISK,
                seed,
            )
        }

        /// 物理块 `physical_block_size`、`blocks_of_the_disk` 块的盘，图样同 [`Self::filled_with_a_pattern`]。
        fn filled_with_a_pattern_on_blocks_of(
            physical_block_size: PhysicalBlockSizeInBytes,
            blocks_of_the_disk: u64,
            seed: u8,
        ) -> Self {
            let device_bytes =
                usize::try_from(u64::from(physical_block_size.0) * blocks_of_the_disk)
                    .expect("盘长装得进 usize");
            DeviceOfBytes {
                bytes: (0..device_bytes)
                    .map(|position| {
                        u8::try_from(position % 251)
                            .expect("小于 256")
                            .wrapping_add(seed)
                    })
                    .collect(),
                physical_block_size,
            }
        }

        fn range_of(
            &self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<std::ops::Range<usize>, BlockDeviceError> {
            check_aligned_and_in_range(
                offset,
                length,
                self.physical_block_size,
                self.size_in_bytes(),
            )?;
            let start = usize::try_from(offset.0).expect("判过不越过盘尾");
            let end = start + usize::try_from(length).expect("判过不越过盘尾");
            Ok(start..end)
        }
    }

    impl BlockDevice for DeviceOfBytes {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, u64::try_from(buffer.len()).expect("读长"))?;
            buffer.copy_from_slice(&self.bytes[range]);
            Ok(())
        }

        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            _durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, u64::try_from(bytes.len()).expect("写长"))?;
            self.bytes[range].copy_from_slice(bytes);
            Ok(())
        }

        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            let range = self.range_of(offset, length)?;
            self.bytes[range].fill(0);
            Ok(())
        }

        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }

        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            self.physical_block_size
        }

        fn size_in_bytes(&self) -> u64 {
            u64::try_from(self.bytes.len()).expect("盘长装得进 u64")
        }
    }

    fn offset_of_block(block: u64) -> DeviceOffsetInBytes {
        DeviceOffsetInBytes(block * u64::from(PHYSICAL_BLOCK_BYTES))
    }

    fn bytes_of_blocks(blocks: u64) -> usize {
        usize::try_from(blocks * u64::from(PHYSICAL_BLOCK_BYTES)).expect("块数装得进 usize")
    }

    /// 从 `first_block` 起读 `blocks` 块；读缓冲先填 0xAA，没读到的那一截露得出来。
    fn read_blocks(device: &dyn BlockDevice, first_block: u64, blocks: u64) -> Vec<u8> {
        let mut buffer = vec![0xAA_u8; bytes_of_blocks(blocks)];
        device
            .read_at(offset_of_block(first_block), &mut buffer)
            .expect("对齐、没越过盘尾的读做得成");
        buffer
    }

    fn bytes_of_the_disk_blocks(disk: &DeviceOfBytes, first_block: u64, blocks: u64) -> Vec<u8> {
        let start = bytes_of_blocks(first_block);
        disk.bytes[start..start + bytes_of_blocks(blocks)].to_vec()
    }

    #[test]
    fn a_write_kept_in_memory_reads_back_and_leaves_the_disk_below_unchanged() {
        let disk = DeviceOfBytes::filled_with_a_pattern(0);
        let bytes_before = disk.bytes.clone();
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let written = vec![0x5A_u8; bytes_of_blocks(1)];
        kept.write_at(
            offset_of_block(3),
            &written,
            WriteDurability::ForceUnitAccess,
        )
        .expect("对齐、没越过盘尾的写做得成");
        assert_eq!(
            read_blocks(&kept, 3, 1),
            written,
            "写过的块读回写进去的字节"
        );
        assert!(
            disk.bytes == bytes_before,
            "写只进内存：底下那块盘一个字节都没变"
        );
    }

    #[test]
    fn a_read_of_blocks_never_written_passes_through_to_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(7);
        let kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        assert_eq!(
            read_blocks(&kept, 5, 3),
            bytes_of_the_disk_blocks(&disk, 5, 3),
            "没写过的块读穿到底下那块盘"
        );
    }

    #[test]
    fn a_read_across_written_and_unwritten_blocks_stitches_the_kept_blocks_with_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(11);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let block_two = vec![0x22_u8; bytes_of_blocks(1)];
        let block_four = vec![0x44_u8; bytes_of_blocks(1)];
        let blocks_seven_and_eight: Vec<u8> = (0..bytes_of_blocks(2))
            .map(|position| u8::try_from(position % 199).expect("小于 256"))
            .collect();
        for (first_block, bytes) in [
            (2, &block_two),
            (4, &block_four),
            (7, &blocks_seven_and_eight),
        ] {
            kept.write_at(offset_of_block(first_block), bytes, WriteDurability::Plain)
                .expect("对齐、没越过盘尾的写做得成");
        }
        let mut expected = bytes_of_the_disk_blocks(&disk, 1, 9);
        for (first_block, bytes) in [
            (2, &block_two),
            (4, &block_four),
            (7, &blocks_seven_and_eight),
        ] {
            let start = bytes_of_blocks(first_block - 1);
            expected[start..start + bytes.len()].copy_from_slice(bytes);
        }
        assert!(
            read_blocks(&kept, 1, 9) == expected,
            "块 1 到 9 一次读：写过的块（2、4、7、8）读回写进去的字节，没写过的（1、3、5、6、9，含末尾那一段）读穿到底下那块盘"
        );
    }

    #[test]
    fn zeroes_written_in_memory_read_back_as_zero_over_the_disk_below() {
        let disk = DeviceOfBytes::filled_with_a_pattern(13);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        kept.write_zeroes_at(offset_of_block(3), u64::from(PHYSICAL_BLOCK_BYTES) * 3)
            .expect("对齐、没越过盘尾的写零做得成");
        let mut expected = bytes_of_the_disk_blocks(&disk, 2, 5);
        expected[bytes_of_blocks(1)..bytes_of_blocks(4)].fill(0);
        assert!(
            read_blocks(&kept, 2, 5) == expected,
            "写零的块 3 到 5 读回全 0，两头的块 2、6 读穿到底下那块盘"
        );
        assert!(
            bytes_of_the_disk_blocks(&disk, 3, 3)
                .iter()
                .any(|byte| *byte != 0),
            "底下那块盘那一段不是全 0（阳性对照：读回的 0 是写零记下的）"
        );
    }

    #[test]
    fn a_request_off_the_physical_block_boundaries_is_refused_before_anything_is_kept() {
        let disk = DeviceOfBytes::filled_with_a_pattern(17);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let unaligned_offset = DeviceOffsetInBytes(u64::from(PHYSICAL_BLOCK_BYTES) + 100);
        let one_block = vec![0x33_u8; bytes_of_blocks(1)];
        let short_of_a_block = vec![0x33_u8; 100];
        let refusals = [
            (
                "偏移不在物理块边界上的写",
                kept.write_at(unaligned_offset, &one_block, WriteDurability::Plain),
                unaligned_offset,
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
            (
                "长度不是物理块整数倍的写",
                kept.write_at(
                    offset_of_block(2),
                    &short_of_a_block,
                    WriteDurability::Plain,
                ),
                offset_of_block(2),
                100,
            ),
            (
                "长度不是物理块整数倍的写零",
                kept.write_zeroes_at(offset_of_block(4), 100),
                offset_of_block(4),
                100,
            ),
            (
                "偏移不在物理块边界上的读",
                kept.read_at(unaligned_offset, &mut vec![0_u8; bytes_of_blocks(1)]),
                unaligned_offset,
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::Unaligned {
                        offset: refused_offset,
                        length: refused_length,
                        physical_block_size: PHYSICAL_BLOCK_BYTES,
                    }) if refused_offset == offset && refused_length == length
                ),
                "{request}：拒成 Unaligned，带偏移 {} 与长度 {length}，而交回 {outcome:?}",
                offset.0
            );
        }
        assert!(
            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == disk.bytes,
            "被拒的写一个块都没记：整盘读回底下那块盘的字节"
        );
    }

    #[test]
    fn a_request_past_the_end_of_the_disk_below_is_refused_and_one_ending_at_the_end_is_kept() {
        let disk = DeviceOfBytes::filled_with_a_pattern(19);
        let device_size = disk.size_in_bytes();
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let last_block = vec![0x77_u8; bytes_of_blocks(1)];
        kept.write_at(
            offset_of_block(BLOCKS_OF_THE_DISK - 1),
            &last_block,
            WriteDurability::Plain,
        )
        .expect("最后一块正好写到盘尾：放行");
        assert_eq!(
            read_blocks(&kept, BLOCKS_OF_THE_DISK - 1, 1),
            last_block,
            "最后一块读回写进去的字节"
        );
        let two_blocks = vec![0x66_u8; bytes_of_blocks(2)];
        let offset_near_the_top =
            DeviceOffsetInBytes(u64::MAX - u64::from(PHYSICAL_BLOCK_BYTES) + 1);
        let refusals = [
            (
                "从最后一块起写两块（越过盘尾一块）",
                kept.write_at(
                    offset_of_block(BLOCKS_OF_THE_DISK - 1),
                    &two_blocks,
                    WriteDurability::Plain,
                ),
                offset_of_block(BLOCKS_OF_THE_DISK - 1),
                u64::from(PHYSICAL_BLOCK_BYTES) * 2,
            ),
            (
                "从盘尾起写零一块",
                kept.write_zeroes_at(
                    offset_of_block(BLOCKS_OF_THE_DISK),
                    u64::from(PHYSICAL_BLOCK_BYTES),
                ),
                offset_of_block(BLOCKS_OF_THE_DISK),
                u64::from(PHYSICAL_BLOCK_BYTES),
            ),
            (
                "偏移加长度越过 u64 的读",
                kept.read_at(offset_near_the_top, &mut vec![0_u8; bytes_of_blocks(2)]),
                offset_near_the_top,
                u64::from(PHYSICAL_BLOCK_BYTES) * 2,
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::OutOfRange {
                        offset: refused_offset,
                        length: refused_length,
                        device_size: refused_device_size,
                    }) if refused_offset == offset && refused_length == length && refused_device_size == device_size
                ),
                "{request}：拒成 OutOfRange，带偏移 {}、长度 {length} 与盘长 {device_size}，而交回 {outcome:?}",
                offset.0
            );
        }
        let mut expected = disk.bytes.clone();
        expected[bytes_of_blocks(BLOCKS_OF_THE_DISK - 1)..].copy_from_slice(&last_block);
        assert!(
            read_blocks(&kept, 0, BLOCKS_OF_THE_DISK) == expected,
            "被拒的写一个块都没记：整盘读回只有最后一块是写进去的"
        );
    }

    /// 差分对拍那一串请求的个数。
    const REQUESTS_OF_THE_RANDOM_STREAM: u64 = 20_000;

    /// 线性同余，取高 33 位（只要确定、够散）。
    fn next_random(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state >> 33
    }

    /// 偏移：多半落在物理块边界上、盘以内，少数不对齐、正好在盘尾、越过盘尾、贴着 u64 的顶。
    fn random_offset(state: &mut u64, device_size: u64, block_bytes: u64) -> u64 {
        match next_random(state) % 8 {
            0 => next_random(state) % (device_size + 2 * block_bytes),
            1 => u64::MAX - next_random(state) % 1024,
            2 => device_size,
            _ => (next_random(state) % (device_size / block_bytes)) * block_bytes,
        }
    }

    /// 长度：多半是一到几个物理块，少数是 0、不对齐、跨过盘尾。
    fn random_length(state: &mut u64, device_size: u64, block_bytes: u64) -> u64 {
        match next_random(state) % 8 {
            0 => 0,
            1 => next_random(state) % (2 * device_size),
            _ => (1 + next_random(state) % 4) * block_bytes,
        }
    }

    /// 一串随机请求（读、写、写零，偶尔一道屏障）同时交给盖在底下那块盘上的只进内存的盘与一块把写直接写进去的同样的盘之后看到的。
    struct RandomRequestStreamAgainstADeviceWrittenThrough {
        /// 两边对不上的请求，逐条写明第几个、什么请求、哪一样不同（读回的字节、交回值）。
        disagreements: Vec<String>,
        refused_requests: u64,
        kept_writes: u64,
        /// 一串请求之后整盘读回的字节与写穿的那块盘相同。
        whole_disk_reads_back_as_written_through: bool,
        /// 一串请求之后底下那块盘一个字节都没变。
        disk_below_is_unchanged: bool,
    }

    /// 物理块 `physical_block_size`、`blocks_of_the_disk` 块、图样种子 `seed` 的盘上跑 [`REQUESTS_OF_THE_RANDOM_STREAM`] 个随机请求。
    fn random_request_stream_against_a_device_written_through(
        physical_block_size: PhysicalBlockSizeInBytes,
        blocks_of_the_disk: u64,
        seed: u8,
    ) -> RandomRequestStreamAgainstADeviceWrittenThrough {
        let disk = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(
            physical_block_size,
            blocks_of_the_disk,
            seed,
        );
        let bytes_before = disk.bytes.clone();
        let device_size = disk.size_in_bytes();
        let block_bytes = u64::from(physical_block_size.0);
        let mut written_through = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(
            physical_block_size,
            blocks_of_the_disk,
            seed,
        );
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let mut state = 1_u64;
        let mut disagreements: Vec<String> = Vec::new();
        let mut refused_requests = 0_u64;
        let mut kept_writes = 0_u64;
        // 迭代上界是请求个数；跨轮带着两块盘、随机数的状态与两个计数；没有提前出口。
        for request_number in 1..=REQUESTS_OF_THE_RANDOM_STREAM {
            let offset = DeviceOffsetInBytes(random_offset(&mut state, device_size, block_bytes));
            let length = random_length(&mut state, device_size, block_bytes);
            let length_in_memory = usize::try_from(length).expect("长度不过两倍盘长");
            let (request, through, in_memory) = match next_random(&mut state) % 3 {
                0 => {
                    let mut through_buffer = vec![0xAA_u8; length_in_memory];
                    let mut in_memory_buffer = vec![0xAA_u8; length_in_memory];
                    let through = written_through.read_at(offset, &mut through_buffer);
                    let in_memory = kept.read_at(offset, &mut in_memory_buffer);
                    if through_buffer != in_memory_buffer {
                        disagreements.push(format!(
                            "第 {request_number} 个请求（读 {} 起 {length} 字节）：读回的字节不同",
                            offset.0
                        ));
                    }
                    ("读", through, in_memory)
                }
                1 => {
                    let bytes: Vec<u8> = (0..length_in_memory)
                        .map(|_| u8::try_from(next_random(&mut state) % 256).expect("小于 256"))
                        .collect();
                    (
                        "写",
                        written_through.write_at(offset, &bytes, WriteDurability::Plain),
                        kept.write_at(offset, &bytes, WriteDurability::ForceUnitAccess),
                    )
                }
                _ => (
                    "写零",
                    written_through.write_zeroes_at(offset, length),
                    kept.write_zeroes_at(offset, length),
                ),
            };
            let (through_returned, in_memory_returned) =
                (format!("{through:?}"), format!("{in_memory:?}"));
            if through_returned != in_memory_returned {
                disagreements.push(format!(
                    "第 {request_number} 个请求（{request} {} 起 {length} 字节）：写穿的盘交回 {through_returned}，只进内存的盘交回 {in_memory_returned}",
                    offset.0
                ));
            }
            match through {
                Ok(()) => kept_writes += u64::from(request != "读"),
                Err(_) => refused_requests += 1,
            }
            if next_random(&mut state).is_multiple_of(50) {
                kept.barrier().expect("屏障什么都不做");
            }
        }
        let mut whole_disk =
            vec![0xAA_u8; usize::try_from(device_size).expect("盘长装得进 usize")];
        kept.read_at(DeviceOffsetInBytes(0), &mut whole_disk)
            .expect("整盘一次读：对齐、没越过盘尾");
        RandomRequestStreamAgainstADeviceWrittenThrough {
            disagreements,
            refused_requests,
            kept_writes,
            whole_disk_reads_back_as_written_through: whole_disk == written_through.bytes,
            disk_below_is_unchanged: disk.bytes == bytes_before,
        }
    }

    /// 物理块 512 字节的盘上：只进内存的盘与写穿的盘逐个请求交回值、读回的字节相同，整盘读回相同，底下那块盘一字不动。
    #[test]
    fn the_device_keeping_writes_in_memory_agrees_with_a_device_written_through_on_a_random_request_stream(
    ) {
        let stream = random_request_stream_against_a_device_written_through(
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            BLOCKS_OF_THE_DISK,
            23,
        );
        assert!(
            stream.disagreements.is_empty(),
            "每个请求两边交回值与读回的字节相同，而 {} 个对不上，头几个：{:?}",
            stream.disagreements.len(),
            stream.disagreements.iter().take(5).collect::<Vec<_>>()
        );
        assert!(
            stream.refused_requests > 0 && stream.kept_writes > 0,
            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {} 个、写 {} 个",
            stream.refused_requests,
            stream.kept_writes
        );
        assert!(
            stream.whole_disk_reads_back_as_written_through,
            "整盘读回与写穿的那块盘相同"
        );
        assert!(
            stream.disk_below_is_unchanged,
            "一串写之后底下那块盘一个字节都没变"
        );
    }

    /// 物理块 4096 字节的盘（正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md` K2：入库用例原来只在 512 的盘上，
    /// 判据里把物理块写死 512 照绿）：同一串随机请求按 4096 的块取偏移与长度，两边逐个请求相同、整盘读回相同、底下那块盘一字不动。
    #[test]
    fn the_device_keeping_writes_in_memory_on_4096_byte_physical_blocks_agrees_with_a_device_written_through_on_a_random_request_stream(
    ) {
        let stream = random_request_stream_against_a_device_written_through(
            PhysicalBlockSizeInBytes(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES),
            BLOCKS_OF_THE_DISK,
            29,
        );
        assert!(
            stream.disagreements.is_empty(),
            "每个请求两边交回值与读回的字节相同，而 {} 个对不上，头几个：{:?}",
            stream.disagreements.len(),
            stream.disagreements.iter().take(5).collect::<Vec<_>>()
        );
        assert!(
            stream.refused_requests > 0 && stream.kept_writes > 0,
            "这一串里既有被拒的请求，也有做成的写与写零（阳性对照）：拒 {} 个、写 {} 个",
            stream.refused_requests,
            stream.kept_writes
        );
        assert!(
            stream.whole_disk_reads_back_as_written_through,
            "整盘读回与写穿的那块盘相同"
        );
        assert!(
            stream.disk_below_is_unchanged,
            "一串写之后底下那块盘一个字节都没变"
        );
    }

    /// 4096 字节的物理块。
    const FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES: u32 = 4096;

    /// 物理块 4096 字节的盘上，落在 512 字节边界上、不在 4096 边界上的请求（偏移或长度）照两个后端那一份判据拒成 `Unaligned`、
    /// 带 4096 这个物理块，一个块都不记；同一块盘上按 4096 对齐的写读得回来。判据里物理块写死 512 时这几个请求都放行。
    #[test]
    fn requests_on_512_byte_boundaries_off_the_4096_byte_physical_blocks_are_refused_before_anything_is_kept(
    ) {
        let block = PhysicalBlockSizeInBytes(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES);
        let block_bytes = u64::from(FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES);
        let disk = DeviceOfBytes::filled_with_a_pattern_on_blocks_of(block, BLOCKS_OF_THE_DISK, 31);
        let mut kept = DeviceKeepingWritesInMemory::over_the_disk(&disk);
        let one_block = vec![0x3C_u8; usize::try_from(block_bytes).expect("一块装得进 usize")];
        let one_sector = vec![0x3C_u8; 512];
        let offset_on_a_sector_inside_block_one = DeviceOffsetInBytes(block_bytes + 512);
        let refusals = [
            (
                "偏移在 512 边界上、不在 4096 边界上的写",
                kept.write_at(
                    offset_on_a_sector_inside_block_one,
                    &one_block,
                    WriteDurability::Plain,
                ),
                offset_on_a_sector_inside_block_one,
                block_bytes,
            ),
            (
                "长度 512 的写",
                kept.write_at(
                    DeviceOffsetInBytes(2 * block_bytes),
                    &one_sector,
                    WriteDurability::Plain,
                ),
                DeviceOffsetInBytes(2 * block_bytes),
                512,
            ),
            (
                "长度 512 的写零",
                kept.write_zeroes_at(DeviceOffsetInBytes(3 * block_bytes), 512),
                DeviceOffsetInBytes(3 * block_bytes),
                512,
            ),
            (
                "偏移在 512 边界上、不在 4096 边界上的读",
                kept.read_at(offset_on_a_sector_inside_block_one, &mut vec![0_u8; 512]),
                offset_on_a_sector_inside_block_one,
                512,
            ),
        ];
        for (request, outcome, offset, length) in refusals {
            assert!(
                matches!(
                    outcome,
                    Err(BlockDeviceError::Unaligned {
                        offset: refused_offset,
                        length: refused_length,
                        physical_block_size: FOUR_KIBIBYTE_PHYSICAL_BLOCK_BYTES,
                    }) if refused_offset == offset && refused_length == length
                ),
                "{request}：拒成 Unaligned，带偏移 {}、长度 {length} 与物理块 4096，而交回 {outcome:?}",
                offset.0
            );
        }
        kept.write_at(
            DeviceOffsetInBytes(5 * block_bytes),
            &one_block,
            WriteDurability::Plain,
        )
        .expect("按 4096 对齐的一块写做得成（阳性对照）");
        let mut expected = disk.bytes.clone();
        let start_of_block_five = usize::try_from(5 * block_bytes).expect("偏移装得进 usize");
        expected[start_of_block_five..start_of_block_five + one_block.len()]
            .copy_from_slice(&one_block);
        let mut whole_disk = vec![0xAA_u8; disk.bytes.len()];
        kept.read_at(DeviceOffsetInBytes(0), &mut whole_disk)
            .expect("整盘一次读：对齐、没越过盘尾");
        assert!(
            whole_disk == expected,
            "被拒的请求一个块都没记：整盘读回只有按 4096 对齐写的那一块是写进去的"
        );
    }

    /// 「被宽护卡住」那一份报拷下的这次改动（[`UserChangeKeptByTheRefusal`]），要出路时读回成交给预演的改动，与被拒那一次交进来的改动
    /// 逐项相同：种类、内容全长、写入时间、inode、个数（正推 `research/prompts/m3-batch10z-vw-code-r1-sonnet-output.md` K3-3：
    /// 记错这几项的 8 种改坏原来照绿，其中读回时把覆盖写当顺序写那一种只在读回这一步，报的 `{:?}` 看不出来）。
    #[test]
    fn each_kind_of_change_kept_by_the_refusal_reads_back_as_the_change_handed_in() {
        let overwrite_content: Vec<u8> = (0..2999_u32)
            .map(|position| u8::try_from(position % 241).expect("小于 256"))
            .collect();
        let sequential_write_content: Vec<u8> = (0..5003_u32)
            .map(|position| u8::try_from((position * 7) % 239).expect("小于 256"))
            .collect();
        let changes_handed_in = [
            UserChange::Overwrite(FirstFile {
                content: &overwrite_content,
                write_time_seconds: 1_000_003,
            }),
            UserChange::SequentialWrite(FirstFile {
                content: &sequential_write_content,
                write_time_seconds: 1_000_007,
            }),
            UserChange::NewInodes {
                count: 37,
                write_time_seconds: 1_000_011,
            },
            UserChange::Unlink {
                inode: InodeNumber(4_099),
                write_time_seconds: 1_000_013,
            },
            UserChange::CreateSnapshot,
            UserChange::DestroySnapshot {
                snapshot: TreeIdentifier(65_551),
            },
        ];
        for change_handed_in in changes_handed_in {
            let kept = UserChangeKeptByTheRefusal::kept_from(change_handed_in);
            assert_eq!(
                format!("{:?}", kept.as_user_change()),
                format!("{change_handed_in:?}"),
                "拷下的改动读回成被拒那一次交进来的改动（种类、内容全长、写入时间、inode、个数逐项相同）"
            );
        }
    }
}
