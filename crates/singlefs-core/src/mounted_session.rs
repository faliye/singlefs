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
//! 有一块不可见就在任何写之前拒（代码三方 m2-closeout-code-r1 Z3-A 乙：只比身份时，换上去的一块空盘照样收到本池的写）。
//! 发布路径与抬 F 各自开写入口的做法不变。

use crate::address::{DeviceIdentity, InstanceGeneration};
use crate::allocator::PoolAllocator;
use crate::block_device::BlockDevice;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::mount::{
    devices_without_a_self_verified_system_configuration,
    push_one_floor_raise_within_the_admission_budget, FloorRaisePushedWithinTheAdmissionBudget,
    FloorRaiseStop, MountError, MountOutput, Mounted, ParametersAndDeviceTableOfTheMount,
    RaisedFloor, ShadowLedger,
};
use crate::transaction::{
    publish_new_inodes, publish_overwrite, publish_sequential_write, FirstFile, PoolVersion,
    PoolWriter, PublishError, TransactionOutput,
};

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
}

/// 一次用户改动发成了：新的一版在会话的 `current` 里。
#[derive(Debug)]
pub struct UserChangePublished {
    /// 这次准入里推的那几串抬 F 的空发布，按先后；一次都没推的是空的。
    pub floor_raises: Vec<RaisedFloor>,
    /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）：`SpaceAdmissionRefused` 或 `PlacementRefused`。
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
}

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
    /// 现行那一版树表 0 条：这三种改动都要接在带文件的一版后面（第一个文件版本走 `transaction::publish_first_file`，不经这一层）。
    /// 在任何读写之前返回。
    NoFileVersionToChange,
    /// 盘表的身份与这次挂载核过的那一份相同，`devices` 里的这几块（按交进来的次序）却不「可见」：两个系统配置槽里一份本池自证过的
    /// （整槽校验和过、fsid 与本池相同）都没有——换上去的空盘、别的池的盘、两槽都读不出（D18（块里携带什么信息） 已定项 11
    /// 「可写挂载的顺序」；与可写挂载取号之前的逐盘核第一支、挂着之后收盘表的入口同一判，`mount::devices_without_a_self_verified_system_configuration`）。
    /// 只比身份时，发布照样做成、本池的写落到那块盘上（代码三方 m2-closeout-code-r1 Z3-A 乙，用户 2026-09-27 定）。
    /// 读过每块盘的两个系统配置槽，一个写都没发：盘上逐字节不变、会话不动。调用方要做的是停下这块盘上的写（换回这个池的盘），
    /// 不是换一份盘表的次序再发（那是 `DeviceTableOtherThanTheOneOfTheMount`）。
    DevicesWithoutASelfVerifiedSystemConfiguration { devices: Vec<DeviceIdentity> },
    /// 发布被拒、不是空间不够的那两种（内容装不下、释放判定对不上、块设备错、冻结着一次没重发……）：`cause` 原样，不因它推。
    /// `floor_raises` 是这之前已经因为空间不够推过的那几串（它们已落盘，现行那一版在它们后面）。
    Publish {
        cause: PublishError,
        floor_raises: Vec<RaisedFloor>,
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
        | PublishError::BlockDevice(_) => false,
    }
}

/// 在一个新开的写入口上发一次这个改动，接在 `previous` 后面。
fn publish_the_change_once<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    change: UserChange<'_>,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let mut writer = PoolWriter::new(parameters, devices);
    match change {
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
    /// 交进来的盘表先与它逐项比（实审 A1b Q2、Q3），再读每块盘的两个系统配置槽核它「可见」（Z3-A 乙）。
    /// 「可见」在进循环之前核一次：一次调用里盘表借着、换不了盘；循环里推的每一串抬 F 在它自己的第一个写之前再核一遍
    /// （`mount::push_one_floor_raise_within_the_admission_budget` 经 `caller_inputs_agreeing_with_the_disk`），
    /// 所以这次调用发出的每一个写之前都核过。
    ///
    /// 循环：每一轮发一次；被拒是空间不够就推一串（至少一次空发布、推的次数增加）进下一轮，不推了就交回。
    /// 预算把推的次数压在 B − 1 以内 ⇒ 至多 8 轮；跨轮携带的是现行那一版（抬 F 就地推进它）与已推的几串。
    ///
    /// # Errors
    /// [`UserChangeRefused`] 的六种。
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
        let current_file_version = match current {
            PoolVersion::WithFile(current_file_version) => current_file_version,
            PoolVersion::WithoutFile(_) => return Err(UserChangeRefused::NoFileVersionToChange),
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
        let mut floor_raises: Vec<RaisedFloor> = Vec::new();
        let mut refusals_that_pushed_the_floor_raises: Vec<PublishError> = Vec::new();
        let mut publishes_pushed = 0;
        loop {
            let refusal = match publish_the_change_once(
                parameters,
                devices.as_mut_slice(),
                allocator,
                current_file_version,
                change,
                *instance,
            ) {
                Ok(published) => {
                    *current_file_version = published;
                    return Ok(UserChangePublished {
                        floor_raises,
                        refusals_that_pushed_the_floor_raises,
                    });
                }
                Err(refusal) => refusal,
            };
            if !refusal_is_short_of_space(&refusal) {
                return Err(UserChangeRefused::Publish {
                    cause: refusal,
                    floor_raises,
                });
            }
            match push_one_floor_raise_within_the_admission_budget(
                parameters,
                devices,
                allocator,
                current_file_version,
                *shadow_ledger,
                publishes_pushed,
            ) {
                Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
                    publishes_pushed += raised.publishes.len();
                    floor_raises.push(raised);
                    refusals_that_pushed_the_floor_raises.push(refusal);
                }
                Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                    return Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(Box::new(
                        NoSpaceAfterRaisingTheFloor {
                            last_refusal: refusal,
                            floor_raises,
                            refusals_that_pushed_the_floor_raises,
                            stop,
                        },
                    )))
                }
                Err(cause) => {
                    return Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(
                        Box::new(FloorRaiseFailedWhilePushingForSpace {
                            cause,
                            refusal_that_pushed_the_failed_raise: refusal,
                            floor_raises,
                            refusals_that_pushed_the_floor_raises,
                        }),
                    ))
                }
            }
        }
    }
}
