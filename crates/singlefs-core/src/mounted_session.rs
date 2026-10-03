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
//! 对不上在任何读写之前拒（实审 A1b Q2、Q3）；身份对得上，还要每块盘「可见」（两个系统配置槽里至少一份本池自证过的），
//! 有一块不可见就在任何写之前拒（代码三方 m2-closeout-code-r1 Z3-A 乙：只比身份时，换上去的一块空盘照样收到本池的写）；
//! 「可见」之后再核每块盘自证槽里的本盘设备号是盘表给它的身份、这块盘不落后于现行那一版又缺它的单元或它的根（D18（块里携带什么信息） 已定项 11
//! 「挂着之后收盘表的每个入口……核三样」，用户 2026-09-27 定；代码三方 m2-closeout-code-r2「盘体对调的盘表」「同池旧快照的盘经入口洗白」；
//! 缺根那一半是 C585（会话发布放进发布途中取下的旧快照） 用户定的改法 C）。
//! 发布路径与抬 F 各自开写入口的做法不变。

use crate::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use crate::allocator::PoolAllocator;
use crate::block_device::BlockDevice;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::mount::{
    devices_behind_the_current_version, devices_without_a_self_verified_system_configuration,
    own_device_numbers_on_disk_differing_from_the_identities_handed_in,
    push_one_floor_raise_within_the_admission_budget, DeviceBehindTheCurrentVersion,
    FloorRaisePushedWithinTheAdmissionBudget, FloorRaiseStop, MountError, MountOutput, Mounted,
    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, ParametersAndDeviceTableOfTheMount,
    RaisedFloor, ShadowLedger,
};
use crate::transaction::{
    publish_new_inodes, publish_overwrite, publish_sequential_write, publish_unlink_intent,
    walk_the_deletion_queue_until_it_is_empty, DeletionQueueWalk, FirstFile, PoolVersion,
    PoolWriter, PublishError, TransactionOutput,
};
use crate::unlink::DeletionSwitches;

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
}

/// 一次用户改动里推的抬 F 之外发成的每一版，按先后：先走空的删除步（D8（核心索引结构） 已定项 16 第 6 条）、这次改动本身
/// （unlink 是意图那次）、unlink 之后走空的删除步；与推的抬 F 的空发布按 txg 交错。最后一个是会话的 `current`
/// （之后没再推抬 F 时）。交给调用方逐次比（模型对拍）：会话的 `current` 只是最后那一版。
pub type VersionsPublishedInThisChange = Vec<TransactionOutput>;

/// 一次用户改动没发成。成员按调用方要做的决定分：换一份盘表再发、换回这个池的盘、换一个版本再改、原样看发布路径的错、报空间不够。
#[derive(Debug)]
pub enum UserChangeRefused {
    /// 交进来的盘表的设备身份（按次序）与这次挂载核过的那一份不同（多一块、少一块、换了次序、同一个身份交两次）：
    /// 写入口按盘表逐项写系统配置与单元，拿别的盘表发，多出来的盘收到本池的写、设备数与本盘设备号被改写（实审 A1b Q3）。
    /// 在任何读写之前返回，盘上逐字节不变、会话不动。
    DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount: Vec<DeviceIdentity>,
        device_table_handed_in: Vec<DeviceIdentity>,
    },
    /// 现行那一版没有可改的文件：树表 0 条（这几种改动都要接在有树的一版后面，第一个文件版本走 `transaction::publish_first_file`，
    /// 不经这一层），在任何读写之前返回；或有树而文件集合为空（`PoolVersion::WithTreesAndNoFile`，D8（核心索引结构） 已定项 16 第 5 条：
    /// 删完之后只许建 inode（size 0）、写行与空发布）而这次改动不是建 inode——先走空队列（那一版的 `deleted_inodes` 根指针置零，
    /// 走空一个写都不发；走空把带文件的一版走成文件集合为空的，那几步已落盘），再拒这次改动。
    /// 走空发成的那几版与走空里为空间推的那几串抬 F 已落盘、带在这里（代码轮 U10：与别的被拒成员一样交回被拒之前发成的，
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
    /// 是推满仍不够，走 `NoSpaceAfterRaisingTheFloor`（实审 A1b Q1）。装箱同 `NoSpaceAfterRaisingTheFloor`。
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
        PublishError::MultiLevelCodeTwoTreeRefused { .. }
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
        // 写删除中的 inode、写已经删完的 inode、unlink 一个不是活记录的号：推抬 F 也不会让它复活。
        | PublishError::WriteToAnInodeBeingDeleted { .. }
        | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
        | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
        // `deleted_inodes` 装不下：类型化错误（D8（核心索引结构） 已定项 16 第 6 条），与空间无关。
        | PublishError::DeletedInodesQueueRefused(_)
        | PublishError::BlockDevice(_) => false,
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
        };
        (output, session)
    }

    /// 发布一次用户改动：被拒是空间不够的那两种（[`refusal_is_short_of_space`]）就推一串抬 F 的空发布、再重判，
    /// 直到发成或不再推（D16（发布语义） 已定项 1「准入」那一行）。发成时 `current` 换成新的一版；推了的那几串已落盘，
    /// 没发成时 `current` 停在最后一次落盘的空发布上。写入口只按会话持着的那份参数建（`parameters_and_device_table`），
    /// 交进来的盘表先与它逐项比（实审 A1b Q2、Q3），再读每块盘的两个系统配置槽核它「可见」（Z3-A 乙），再核本盘设备号与落后支
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
        let mut published_in_this_change = PublishedInThisChange {
            floor_raises: Vec::new(),
            refusals_that_pushed_the_floor_raises: Vec::new(),
            versions_published: Vec::new(),
        };
        // 队列空（任何一版都是常态）时一个写都不发、不复制；不空时在一份复制上走，走到哪一步（连同推的抬 F）`current` 就是哪一版。
        if !current_version_of_trees.deleted_inodes.is_empty() {
            let mut walked = current_version_of_trees.clone();
            let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                parameters,
                devices,
                allocator,
                &mut walked,
                *instance,
                *shadow_ledger,
                published_in_this_change,
            );
            *current = PoolVersion::of_a_version_of_trees(walked);
            published_in_this_change = walk?;
        }
        let mut publishes_pushed = 0;
        // 按成员分流（类型上放什么）：有树而文件集合为空的一版只接建 inode。走空把带文件的一版走成这一形时，走空的那几步与推的抬 F
        // 已落盘，带在这个拒绝里（`NoFileVersionToChange` 的三样，代码轮 U10）。
        let version_to_change = match current {
            PoolVersion::WithFile(file_version) => file_version,
            PoolVersion::WithTreesAndNoFile(emptied_version) => match change {
                UserChange::NewInodes { .. } => emptied_version,
                UserChange::Overwrite(_)
                | UserChange::SequentialWrite(_)
                | UserChange::Unlink { .. } => {
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
                *instance,
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
                        *instance,
                        *shadow_ledger,
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
                    *shadow_ledger,
                    publishes_pushed,
                    published_in_this_change,
                    refusal,
                )?;
            publishes_pushed += publishes_of_this_raise;
        }
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
/// 不再推交回 `NoSpaceAfterRaisingTheFloor`，抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
/// 推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、预算已用的数、已落盘的、这一次被拒，不成一个概念"
)]
fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
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
/// （删除那一步或推的空发布）；发成的每一步记进 `versions_published`。
fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
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
                publishes_pushed_in_this_admission,
                published_in_this_change,
                refusal,
            )?;
        publishes_pushed_in_this_admission += publishes_of_this_raise;
    }
}
