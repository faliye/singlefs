//! 代码三方 m2-closeout-code-r2（判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节 Y2-a、Y2-b，第二节「四个入口再核两样」）：
//! 挂着之后收盘表的每个入口——会话发布、正常卸载、抬 F、管理员回退——在「可见」
//! 那一判之后再核两样（D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口……核三样」）：
//! - 每块盘自证过的系统配置槽里记的本盘设备号等于盘表给它的身份（盘体对调的盘表，Y2-b）；
//! - 这块盘最新那份自证系统配置的 (实例代号, journal tail) 不落后于现行那一版末条记录的 jsn、落后就读现行那一版的单元，缺一份就拒
//!   （同池旧快照的盘，Y2-a；与可写挂载取号之前逐盘核的落后支同一判）。
//!
//! 任一不过都在任何写之前拒，两块盘逐字节不变；之后拿同一组盘可写挂载，照不经这些入口的对照一样被落后支拒。
//! 只剩一槽自证的盘（那一槽停在上一次轮换、单元都在）四个入口都不误拒（Y2-c）。
//!
//! 攻方用例（`research/prompts/m2-closeout-code-r2-opus-model/opus_r2_y2_session_and_entries_with_stale_or_swapped_devices.rs`）只打印、
//! 钉改之前的样子：旧快照经会话发布或卸载照常做成、之后可写挂载做成（实例 2）、池级 checker 红 I-2.1；盘体对调经会话照常发布。
//! 这里搬过来反过来钉，每形一条；攻方放开扫的 16 格（旧快照取自哪一刻 × 换哪块盘 × 之后会话发 1/2/3 次或卸载一次）单列一条。
//!
//! 起点同攻方：两块 4 GiB 内存稀疏盘上 mkfs、取号、暖机、第一个文件（txg 3），会话里覆盖写两次（txg 4、5）。
//!
//! C585（会话发布放进发布途中取下的旧快照；里程碑三二·2，用户定改法 C）：落后支改成「落后，并且缺现行那一版的单元，**或**现行那一版的根
//! 写在这块盘上而那个根环槽里不是它」才拒，四个入口与可写挂载取号之前的逐盘核同一判。攻方
//! （`research/prompts/m2-closeout-code-r3-opus-output.md`「X6-a」）把一次发布 P 途中（单元与记录已写、根槽 FUA 未写）取下的、P 的根
//! 住的那块盘换上来，会话覆盖写照常做成、池级 checker 红 I-3.1；抬 F、卸载、回退当时靠「已知根槽读坏」拒掉，不是落后支拒的。
//! 这里每个入口一条钉「在任何写之前拒成落后支缺根那一半、两块盘逐字节不变」；另钉两形不误拒：切在 P 的根槽 FUA 之后的那块盘
//! （根已落、轮换没写，等同崩溃状态）会话照常发布，切在根槽之前的那块盘上可写挂载照常施加 P 的记录挂上（P 的根不在根环里，不核缺根那一半）。
//!
//! 代码三方 m3-code-r1（判决 `research/prompts/m3-code-r1-main-verification.md` 第三节 H1–H3）补的几格：
//! - H1：切在 P 的根槽 FUA 之后的那块盘（落后、单元都在、根槽里是 P），缺根那一半第一次读那一槽碰上一次瞬时读错、重读读得出 P，
//!   会话发布与可写挂载都照常做成（攻方原型 `research/prompts/m3-code-r1-opus-model/proto/tests/z2_reread_one_transient_root_slot_read_error.rs`
//!   的甲那一形；去掉重读时两处都拒成落后支）。
//! - H2：P 的根住盘 1（txg 7，区域 1），与根环转过一圈、P 的根槽里是上一圈自证得过的另一条根（txg 27 那一槽里住着 txg 3）——
//!   「只查盘 0 的根槽」「根槽里自证得过就算是它」两种改错各让一格红。
//! - H3：同一块切在根槽之后的盘上可写挂载，所选那一版就是择到的根（写进了根环），那一槽在缺根那一半连着两次读错：
//!   拒成 `BehindTheSelectedVersionAndItsRootNotInItsRingSlot`，拒在任何写之前、两块盘逐扇区不变。
//!
//! 代码三方 m3-code-r2（判决 `research/prompts/m3-code-r2-main-verification.md` 第三节 J1–J3）补的几格：
//! - J1：P 是 txg 8（区域 2；`region_devices` 是 [盘 0, 盘 1, 盘 0]，区域 2 归盘 0）。「把区域号当盘号」时区域 2 被当成盘表里没有的盘 2，
//!   盘 0 的缺根不核、会话照常发布。
//! - J2：H1 那一形（缺根那一半第一次读 P 的根槽读坏、重读读得出 P）在正常卸载、准入抬 F、管理员回退三个入口各一格；
//!   这三个入口不重读时拒成缺根。
//! - J3：根环转过一圈的那一形在这三个入口各两格（管理员回退取现行那一版的前一版做目标）：txg 27（区域 0、住盘 0，槽里是上一圈
//!   第一个文件那一版 txg 3）与 txg 25（区域 1、住盘 1，槽里是上一圈暖机那一版 txg 1）。这一形上缺根那一半是三个入口唯一的一道
//!   （没有「重读仍坏」兜底）：「根槽里自证得过就算是它」两种 txg 都红；「只查区域 0 那块盘」只在 txg 25 红——txg 27 的根就住在区域 0 那块盘上。
//!
//! 代码三方 m3-code-r3（判决 `research/prompts/m3-code-r3-main-verification.md` 第三节 L1、L2）补的几格：
//! - L1：H1 的两格与 J2 的三格各加一版「根槽第一次读回的字节坏一位」（读报成功、自证不过），重读读得出 P，五个入口都照常做成、
//!   注入真发生了 1 次（D16（发布语义） 已定项 1「读不出或自证不过就重读一次」的后一半）。「只在第一次读不出时才重读」（攻方 RU）五格都红。
//! - L2：根环转过一圈的那一形在正常卸载、准入抬 F、管理员回退各加 txg 26 一格（区域 2、槽 0，区域 2 归盘 0，槽里是上一圈暖机写的
//!   第二条根 txg 2）。这三个入口上「把区域号当盘号」（攻方 RGC）时区域 2 被当成盘表里没有的盘 2，缺根不核、入口照常做成。

mod common_admission;

use common_admission::{
    checker_violations_on, content_of, plain_devices_on, start_plain, PoolUnderTest,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish,
    unmount, CallerInputsDisagreeingWithTheDisk, DeviceBehindTheCurrentVersion,
    DeviceTableDisagreement, DeviceWithoutTheSelectedVersion, MountError,
    OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn, RingSlotReadingOtherThanTheVersionRoot,
    RollbackError, RollbackTarget, SelectedVersionLackingOnDevice, ShadowLedger, Unmounted,
    VersionLackingOnADeviceBehindIt, VersionRootNotInItsRingSlot,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::{verified_system_configuration_slots, BadRootRingSlotReading};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{FirstFile, PoolVersion, TransactionOutput};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, FlippedBit, InjectedFault, NamedRootRingSlots, RootRingSlotTarget,
    SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;

type SparseDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

/// 管理员回退那一格的目标：第一个文件那一版（实例 1、txg 3），F = 0 时是候选。
const FIRST_FILE_VERSION: RollbackTarget = RollbackTarget {
    instance: InstanceGeneration(1),
    checkpoint_txg: CheckpointTxg(3),
};

/// 镜像里 `identity` 那块盘的一份拷贝（改它不动镜像）。
fn copy_of_device(image: &MemoryPool, identity: DeviceIdentity) -> SparseBlockDevice {
    let mut device =
        SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
    device.image = image
        .devices
        .get(&identity)
        .cloned()
        .expect("镜像里有这块盘");
    device
}

/// 盘表每一项的身份与盘体从哪份镜像的哪块盘拷来：`(身份, 镜像, 那份镜像里的哪块盘)`。
fn devices_from(bodies: [(DeviceIdentity, &MemoryPool, DeviceIdentity); 2]) -> SparseDevices {
    bodies
        .into_iter()
        .map(|(identity, image, body)| (identity, copy_of_device(image, body)))
        .collect()
}

/// 盘表里每块盘此刻的镜像，按盘表次序。
fn images_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> Vec<SparseDevice> {
    devices
        .iter()
        .map(|(_, device)| device.image.clone())
        .collect()
}

/// 一组盘的拷贝（拿去可写挂载，原来那组不动）。
fn copies_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> SparseDevices {
    devices
        .iter()
        .map(|(identity, device)| {
            let mut copy =
                SparseBlockDevice::new(device.size_in_bytes(), PhysicalBlockSizeInBytes(512));
            copy.image = device.image.clone();
            (*identity, copy)
        })
        .collect()
}

/// 这组盘（拷贝）上可写挂载的结局，错误按 `Debug` 交回（对照与入口之后的那一次逐字比）。
fn writable_mount_outcome_on_a_copy_of(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
) -> Result<InstanceGeneration, String> {
    let mut copies = copies_of(devices);
    mount_writable(parameters, &mut copies)
        .map(|mounted| mounted.output.instance)
        .map_err(|error| format!("{error:?}"))
}

/// 第一个文件之后留一份镜像（旧快照），会话里再覆盖写两次（txg 4、5）：交回池、旧快照与此刻的镜像。
fn pool_with_a_snapshot_after_the_first_file(
) -> (PoolUnderTest<SparseBlockDevice>, MemoryPool, MemoryPool) {
    let mut pool = start_plain(DEVICE_WIDTH);
    let snapshot_after_the_first_file = pool.image();
    pool.overwrite(OVERWRITE_BYTES)
        .expect("会话里第一次覆盖写（txg 4）");
    pool.overwrite(OVERWRITE_BYTES)
        .expect("会话里第二次覆盖写（txg 5）");
    let now = pool.image();
    (pool, snapshot_after_the_first_file, now)
}

/// 会话此刻的现行那一版（第一个文件之后都带文件）：抬 F 与管理员回退要一份 `&mut TransactionOutput`。
fn file_version_of(current: &mut PoolVersion) -> &mut TransactionOutput {
    match current {
        PoolVersion::WithFile(current) | PoolVersion::WithTreesAndNoFile(current) => current,
        PoolVersion::WithoutFile(_) => panic!("第一个文件之后现行那一版带文件"),
    }
}

/// 一次覆盖写的改动（写入时刻接着池里的往后数）。
fn overwrite_after(pool: &PoolUnderTest<SparseBlockDevice>, steps_after: u64) -> (Vec<u8>, u64) {
    let write_time_seconds = pool.write_time_seconds + 1 + steps_after;
    (
        content_of(OVERWRITE_BYTES, write_time_seconds),
        write_time_seconds,
    )
}

/// 挂着之后收盘表的四个入口（D18（块里携带什么信息） 已定项 11「挂着之后收盘表的每个入口」）。
#[derive(Clone, Copy, Debug)]
enum EntryAfterTheMount {
    SessionPublish,
    NormalUnmount,
    FloorRaiseToTheAdmissionCeiling,
    AdministratorRollback,
}

const EVERY_ENTRY: [EntryAfterTheMount; 4] = [
    EntryAfterTheMount::SessionPublish,
    EntryAfterTheMount::NormalUnmount,
    EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
    EntryAfterTheMount::AdministratorRollback,
];

/// 四个入口各自被拒时的错误（[`go_through`] 做成交回 `None`）。
enum EntryRefusal {
    Session(UserChangeRefused),
    Mount(MountError),
    Rollback(RollbackError),
}

/// 管理员回退那个入口回退到哪一版（另三个入口不看它）。
#[derive(Clone, Copy, Debug)]
enum RollbackTargetOfTheEntry {
    /// 第一个文件那一版（[`FIRST_FILE_VERSION`]）。
    FirstFileVersion,
    /// 现行那一版的前一版（同一实例、txg 小一）：根环转过一圈之后第一个文件那一版的根槽已被覆盖、不再是候选，
    /// 前一版的根一直在根环里。
    VersionBeforeTheCurrent,
}

/// 拿池里那条会话，经 `entry` 把这组盘交进去一次（管理员回退回到第一个文件那一版）。
fn go_through<Device: BlockDevice>(
    entry: EntryAfterTheMount,
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Option<EntryRefusal> {
    go_through_rolling_back_to(
        entry,
        RollbackTargetOfTheEntry::FirstFileVersion,
        pool,
        devices,
    )
}

/// 拿池里那条会话，经 `entry` 把这组盘交进去一次；`entry` 是管理员回退时回退到 `rollback_target`。
fn go_through_rolling_back_to<Device: BlockDevice>(
    entry: EntryAfterTheMount,
    rollback_target: RollbackTargetOfTheEntry,
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Option<EntryRefusal> {
    let parameters = pool.parameters.clone();
    let (content, write_time_seconds) = overwrite_after(pool, 0);
    let session = pool.session.as_mut().expect("这一刻有可写会话");
    match entry {
        EntryAfterTheMount::SessionPublish => session
            .publish_user_change(
                devices,
                UserChange::Overwrite(FirstFile {
                    content: &content,
                    write_time_seconds,
                }),
            )
            .err()
            .map(EntryRefusal::Session),
        EntryAfterTheMount::NormalUnmount => unmount(
            &parameters,
            devices,
            &mut session.allocator,
            &mut session.current,
            ShadowLedger::On,
        )
        .map(|unmounted| match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(_) => (),
            Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => {
                panic!("现行那一版带文件，卸载要推那一串")
            }
        })
        .err()
        .map(EntryRefusal::Mount),
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling => {
            let MountedSession {
                allocator,
                current,
                shadow_ledger,
                ..
            } = session;
            raise_rollback_floor_to_the_admission_ceiling(
                &parameters,
                devices,
                allocator,
                file_version_of(current),
                *shadow_ledger,
            )
            .err()
            .map(EntryRefusal::Mount)
        }
        EntryAfterTheMount::AdministratorRollback => {
            let MountedSession {
                allocator, current, ..
            } = session;
            let target = match rollback_target {
                RollbackTargetOfTheEntry::FirstFileVersion => FIRST_FILE_VERSION,
                RollbackTargetOfTheEntry::VersionBeforeTheCurrent => {
                    let current_root = file_version_of(current).root;
                    RollbackTarget {
                        instance: current_root.instance,
                        checkpoint_txg: CheckpointTxg(
                            current_root
                                .checkpoint_txg
                                .0
                                .checked_sub(1)
                                .expect("现行那一版在第一个文件之后，txg 大于 0"),
                        ),
                    }
                }
            };
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                allocator,
                file_version_of(current),
                target,
            )
            .err()
            .map(EntryRefusal::Rollback)
        }
    }
}

/// 落后支拒的那一块盘：盘 `device` 被换成旧快照，最新那份自证系统配置停在第一个文件（实例 1、计数器 3），现行那一版末条记录是 (1, 5)。
fn assert_refused_as_behind(
    refusal: Option<EntryRefusal>,
    entry: EntryAfterTheMount,
    device: DeviceIdentity,
) {
    let devices_behind: Vec<DeviceBehindTheCurrentVersion> = match refusal {
        Some(EntryRefusal::Session(
            UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Mount(
            MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Rollback(RollbackError::CallerInputsDisagreeWithTheDisk(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices,
            },
        ))) => devices,
        other => panic!("{entry:?}：该被落后支拒，实际 {}", describe(&other)),
    };
    assert_eq!(
        devices_behind
            .iter()
            .map(|behind| behind.device)
            .collect::<Vec<_>>(),
        vec![device],
        "{entry:?}：点名换成旧快照的那一块"
    );
    let behind = &devices_behind[0];
    assert_eq!(
        (
            behind
                .newest_system_configuration_journal_tail
                .instance_generation,
            behind.newest_system_configuration_journal_tail.counter,
        ),
        (InstanceGeneration(1), 3),
        "{entry:?}：旧快照最新那份系统配置停在第一个文件那一次发布"
    );
    assert_eq!(
        (
            behind.current_version_journal_position.instance_generation,
            behind.current_version_journal_position.counter,
        ),
        (InstanceGeneration(1), 5),
        "{entry:?}：现行那一版末条记录"
    );
    assert!(
        matches!(
            &behind.lacking,
            VersionLackingOnADeviceBehindIt::UnitsMissing(units_missing) if !units_missing.is_empty()
        ),
        "{entry:?}：现行那一版有单元在旧快照上不在：{:?}",
        behind.lacking
    );
}

fn describe(refusal: &Option<EntryRefusal>) -> String {
    match refusal {
        None => "做成了".to_string(),
        Some(EntryRefusal::Session(refused)) => format!("{refused:?}"),
        Some(EntryRefusal::Mount(refused)) => format!("{refused:?}"),
        Some(EntryRefusal::Rollback(refused)) => format!("{refused:?}"),
    }
}

/// Y2-a 的一格：盘 1 换成第一个文件之后的旧快照，经 `entry` 交进去。在任何写之前拒成落后支、点名盘 1，两块盘逐字节不变、会话的现行版本不动；
/// 之后拿这组盘可写挂载，与不经入口的对照逐字相同（被落后支拒）。
fn stale_snapshot_of_device_one_through(entry: EntryAfterTheMount) {
    let (mut pool, snapshot, now) = pool_with_a_snapshot_after_the_first_file();
    let mut devices = devices_from([
        (DeviceIdentity(0), &now, DeviceIdentity(0)),
        (DeviceIdentity(1), &snapshot, DeviceIdentity(1)),
    ]);
    let control = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
    assert!(
        control.as_ref().is_err_and(
            |error| error.starts_with("WritableMountRefusedByDevicesWithoutTheSelectedVersion")
        ),
        "对照：不经入口直接可写挂载，被落后支拒：{control:?}"
    );
    let images_before = images_of(&devices);
    let current_before = pool.current_file_version().root;
    let refusal = go_through(entry, &mut pool, &mut devices);
    assert_refused_as_behind(refusal, entry, DeviceIdentity(1));
    assert_eq!(
        images_of(&devices),
        images_before,
        "{entry:?}：两块盘逐字节不变"
    );
    assert_eq!(
        pool.current_file_version().root,
        current_before,
        "{entry:?}：会话的现行版本不动"
    );
    assert_eq!(
        writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices),
        control,
        "{entry:?}：之后可写挂载照对照一样被拒"
    );
}

#[test]
fn a_session_publish_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write() {
    stale_snapshot_of_device_one_through(EntryAfterTheMount::SessionPublish);
}

#[test]
fn a_normal_unmount_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write() {
    stale_snapshot_of_device_one_through(EntryAfterTheMount::NormalUnmount);
}

#[test]
fn a_floor_raise_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write() {
    stale_snapshot_of_device_one_through(EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling);
}

#[test]
fn an_administrator_rollback_onto_a_stale_snapshot_of_device_one_is_refused_before_any_write() {
    stale_snapshot_of_device_one_through(EntryAfterTheMount::AdministratorRollback);
}

/// 本盘设备号那一判拒的：盘表 [0, 1] 交进来的是对调的盘体，每块盘两槽里记的都是另一个身份。
fn assert_refused_as_swapped(refusal: Option<EntryRefusal>, entry: EntryAfterTheMount) {
    let expected_pairs = vec![
        (DeviceIdentity(0), DeviceIdentity(1)),
        (DeviceIdentity(1), DeviceIdentity(0)),
    ];
    let pairs: Vec<(DeviceIdentity, DeviceIdentity)> = match refusal {
        Some(EntryRefusal::Session(UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable {
            disagreements,
        })) => disagreements
            .into_iter()
            .map(
                |OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn {
                     identity_handed_in,
                     own_device_number_on_disk,
                 }| (identity_handed_in, own_device_number_on_disk),
            )
            .collect(),
        Some(EntryRefusal::Mount(MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
            disagreeing_fields,
            disagreeing_device_table,
        }))
        | Some(EntryRefusal::Rollback(RollbackError::CallerInputsDisagreeWithTheDisk(
            CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
                disagreeing_fields,
                disagreeing_device_table,
            },
        ))) => {
            assert!(disagreeing_fields.is_empty(), "{entry:?}：参数都对得上");
            disagreeing_device_table
                .into_iter()
                .map(|disagreement| match disagreement {
                    DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                        identity_handed_in,
                        own_device_number_on_disk,
                    } => (identity_handed_in, own_device_number_on_disk),
                    DeviceTableDisagreement::DeviceCountDiffersFromTheSystemConfiguration { .. }
                    | DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn {
                        ..
                    } => panic!("{entry:?}：只该是本盘设备号不符，实际 {disagreement:?}"),
                })
                .collect()
        }
        other => panic!("{entry:?}：该被本盘设备号那一判拒，实际 {}", describe(&other)),
    };
    assert_eq!(
        pairs, expected_pairs,
        "{entry:?}：(盘表给的身份, 盘上记的本盘设备号)"
    );
}

/// Y2-b 的一格：盘表身份照旧是 [0, 1]，盘 0 的位置交盘 1 的盘体、盘 1 的位置交盘 0 的盘体。经 `entry` 交进去，在任何写之前拒成
/// 本盘设备号不符、两块都点名，两块盘逐字节不变、会话的现行版本不动；盘体放回各自的身份之后照常可写挂载。
/// 会话那一格改之前只比盘表的身份次序：照常发布、把错的本盘设备号写进两块盘，之后这个池可写挂不上。
fn swapped_device_bodies_through(entry: EntryAfterTheMount) {
    let (mut pool, _, now) = pool_with_a_snapshot_after_the_first_file();
    let mut devices = devices_from([
        (DeviceIdentity(0), &now, DeviceIdentity(1)),
        (DeviceIdentity(1), &now, DeviceIdentity(0)),
    ]);
    let images_before = images_of(&devices);
    let current_before = pool.current_file_version().root;
    let refusal = go_through(entry, &mut pool, &mut devices);
    assert_refused_as_swapped(refusal, entry);
    assert_eq!(
        images_of(&devices),
        images_before,
        "{entry:?}：两块盘逐字节不变"
    );
    assert_eq!(
        pool.current_file_version().root,
        current_before,
        "{entry:?}：会话的现行版本不动"
    );
    let bodies_put_back: SparseDevices = vec![
        (DeviceIdentity(0), devices.remove(1).1),
        (DeviceIdentity(1), devices.remove(0).1),
    ];
    assert!(
        writable_mount_outcome_on_a_copy_of(&pool.parameters, &bodies_put_back).is_ok(),
        "{entry:?}：盘体放回各自的身份之后照常可写挂载"
    );
}

#[test]
fn a_session_publish_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    swapped_device_bodies_through(EntryAfterTheMount::SessionPublish);
}

#[test]
fn a_normal_unmount_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    swapped_device_bodies_through(EntryAfterTheMount::NormalUnmount);
}

#[test]
fn a_floor_raise_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    swapped_device_bodies_through(EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling);
}

#[test]
fn an_administrator_rollback_with_the_two_device_bodies_swapped_is_refused_before_any_write() {
    swapped_device_bodies_through(EntryAfterTheMount::AdministratorRollback);
}

/// Y2-c：盘 1 两个系统配置槽里较新（世代号大）那一槽写坏一个字节，只剩上一次轮换那一槽自证得过——它落后于现行那一版，而现行那一版的单元
/// 都在盘 1 上。四个入口都照常做成（不误拒），做完之后池级 checker 不红；入口写过系统配置轮换的，之后照常可写挂载，
/// 一次轮换都没写的，那一槽还坏着，之后的可写挂载按 C331（择根倒挂压过已确认的写） 取甲拒。
#[test]
fn a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused_by_any_entry() {
    for entry in EVERY_ENTRY {
        let (mut pool, _, now) = pool_with_a_snapshot_after_the_first_file();
        let mut devices = devices_from([
            (DeviceIdentity(0), &now, DeviceIdentity(0)),
            (DeviceIdentity(1), &now, DeviceIdentity(1)),
        ]);
        let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
        let newer_generation = verified_system_configuration_slots(
            &devices,
            DeviceIdentity(1),
            spacing,
            &pool.parameters.filesystem_identifier,
        )
        .into_iter()
        .map(|slot| slot.quantities.slot_generation)
        .max()
        .expect("盘 1 两槽自证得过");
        // 下一次写的槽 = 世代号 mod 2（D22（单元原子性怎么合成） 已定项 16）：世代号 g 那一份住槽 g mod 2。
        let newer_slot_offset = DeviceOffsetInBytes((newer_generation % 2) * spacing);
        let device_one = &mut devices[1].1;
        let mut newer_slot = vec![0u8; 512];
        device_one
            .read_at(newer_slot_offset, &mut newer_slot)
            .expect("读较新那一槽");
        newer_slot[100] ^= 0xff;
        device_one
            .write_at(newer_slot_offset, &newer_slot, WriteDurability::Plain)
            .expect("写坏较新那一槽");
        assert_eq!(
            verified_system_configuration_slots(
                &devices,
                DeviceIdentity(1),
                spacing,
                &pool.parameters.filesystem_identifier,
            )
            .len(),
            1,
            "{entry:?}：盘 1 只剩一槽自证"
        );
        let refusal = go_through(entry, &mut pool, &mut devices);
        assert!(
            refusal.is_none(),
            "{entry:?}：只剩一槽自证的盘不误拒：{}",
            describe(&refusal)
        );
        let after = MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.image.clone()))
                .collect(),
            device_size_in_bytes: now.device_size_in_bytes,
        };
        assert_eq!(
            common_admission::checker_violations_on(&after),
            Vec::<String>::new(),
            "{entry:?}：做完之后池级 checker 不红"
        );
        // 入口的写里有一次系统配置轮换就把写坏的那一槽重写了；入口一次轮换都没写（抬 F 到准入上限而没有可抬的），那一槽还自证不过——
        // 写坏的一槽读得出、自证不过，设备没报错：C331（择根倒挂压过已确认的写） 改取 R2 之后不判不出（用户 2026-10-03 定），
        // 两种都照常可写挂载（取甲时后一种拒）。
        let outcome = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
        assert!(
            outcome.is_ok(),
            "{entry:?}：写坏的那一槽读得出、只是自证不过，R2 判得出，之后照常可写挂载：{outcome:?}"
        );
    }
}

/// 放开扫那条用例的线程数变量（`research/scripts/capped.sh` 设它）。
const THREAD_CAP_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_THREAD_CAP";

/// 放开扫的线程数：`thread_cap_value` 是 [`THREAD_CAP_ENVIRONMENT_VARIABLE`] 的值（`None` 是没设），经共用解析
/// `number_from_the_variable_value_panicking_when_unreadable` 读成 `NonZeroUsize`；没设取本机并行度；不超过格数。
///
/// # Panics
/// 设了而认不出（含 0、空串）：写错的值不悄悄退回本机并行度（代码三方 test-script-align-code-r2 判决「数值解析收成一份」，C606 同族）。
fn sweep_thread_count(thread_cap_value: Option<&str>, cell_count: usize) -> usize {
    number_from_the_variable_value_panicking_when_unreadable::<std::num::NonZeroUsize>(
        THREAD_CAP_ENVIRONMENT_VARIABLE,
        thread_cap_value,
    )
    .map_or_else(
        || std::thread::available_parallelism().map_or(1, usize::from),
        usize::from,
    )
    .clamp(1, cell_count)
}

/// 放开扫的线程数变量写错：panic，带变量名与原值，不悄悄退回本机并行度。
#[test]
#[should_panic(expected = "SINGLEFS_THREAD_CAP=\"twelve\" 认不出")]
fn an_unreadable_thread_cap_for_the_swept_user_steps_panics_instead_of_falling_back_to_the_default()
{
    let _thread_count = sweep_thread_count(Some("twelve"), 16);
}

/// 放开扫的线程数变量设成 0：照共用解析的口径（`NonZeroUsize`）panic，不当成「取默认」、不悄悄抬成 1。
#[test]
#[should_panic(expected = "SINGLEFS_THREAD_CAP=\"0\" 认不出")]
fn a_thread_cap_of_zero_for_the_swept_user_steps_panics_instead_of_taking_the_default() {
    let _thread_count = sweep_thread_count(Some("0"), 16);
}

/// 攻方放开扫的 16 格（`stale_snapshot_substitution_swept_over_the_user_steps`）：旧快照取自第一个文件之后或覆盖写 1 之后 × 换盘 0 或盘 1 ×
/// 之后经会话发 1 / 2 / 3 次或正常卸载一次。每一格：对照（不经入口直接可写挂载）被拒；入口的每一次都拒成落后支、点名换掉的那一块；
/// 两块盘逐字节不变；之后可写挂载与对照逐字相同。改之前 16 格入口都做成、之后可写挂载做成（实例 2）、池级 checker 红 1–6 条 I-2.1。
#[test]
fn stale_snapshot_substitution_is_refused_by_every_entry_over_the_swept_user_steps() {
    let mut cells = Vec::new();
    for snapshot_after_overwrites in [0_usize, 1] {
        for stale_device in [DeviceIdentity(0), DeviceIdentity(1)] {
            for session_publishes in [Some(1_u64), Some(2), Some(3), None] {
                cells.push((snapshot_after_overwrites, stale_device, session_publishes));
            }
        }
    }
    // 16 格彼此独立（各自建池）：按线程数切片并行（`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」），
    // 线程数见 sweep_thread_count；每格自己断言，一格红就在 join 时带着它的名字红。
    let thread_count = sweep_thread_count(
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
        cells.len(),
    );
    let cells_per_thread = cells.len().div_ceil(thread_count);
    std::thread::scope(|scope| {
        for slice in cells.chunks(cells_per_thread) {
            scope.spawn(move || {
                for (snapshot_after_overwrites, stale_device, session_publishes) in slice {
                    eprintln!(
                        "扫到：旧快照在第 {snapshot_after_overwrites} 次覆盖写之后、换盘 {}、{session_publishes:?}",
                        stale_device.0
                    );
                    stale_snapshot_sweep_cell(*snapshot_after_overwrites, *stale_device, *session_publishes);
                }
            });
        }
    });
}

/// 放开扫的一格：旧快照取自第 `snapshot_after_overwrites` 次覆盖写之后、换的是 `stale_device`、之后经会话发 `session_publishes` 次
/// （`None` 是正常卸载一次）。
fn stale_snapshot_sweep_cell(
    snapshot_after_overwrites: usize,
    stale_device: DeviceIdentity,
    session_publishes: Option<u64>,
) {
    let cell = format!(
        "旧快照在第 {snapshot_after_overwrites} 次覆盖写之后、换盘 {}、{}",
        stale_device.0,
        session_publishes.map_or("正常卸载一次".to_string(), |count| format!(
            "会话发 {count} 次"
        ))
    );
    let mut pool = start_plain(DEVICE_WIDTH);
    let mut snapshot = pool.image();
    for overwrite in 0..2 {
        if overwrite == snapshot_after_overwrites {
            snapshot = pool.image();
        }
        pool.overwrite(OVERWRITE_BYTES).expect("会话里覆盖写");
    }
    let now = pool.image();
    let mut devices: SparseDevices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let image = if identity == stale_device {
                &snapshot
            } else {
                &now
            };
            (identity, copy_of_device(image, identity))
        })
        .collect();
    let control = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
    // 换盘 1 时对照拒在落后支；换盘 0 时最新那条根（txg 5）只在盘 0 上、所选那一版退到 txg 4，盘 1 的系统配置见证过 txg 5，
    // 先被 C554 乙拒（`NewerStateStillUnreadableAfterOneReread`）。两种都是可写挂载在取号之前拒，这里只钉「拒」与之后逐字相同。
    assert!(control.is_err(), "{cell}：对照可写挂载被拒：{control:?}");
    let images_before = images_of(&devices);
    let refusals: Vec<Option<EntryRefusal>> = match session_publishes {
        Some(count) => (0..count)
            .map(|_| go_through(EntryAfterTheMount::SessionPublish, &mut pool, &mut devices))
            .collect(),
        None => vec![go_through(
            EntryAfterTheMount::NormalUnmount,
            &mut pool,
            &mut devices,
        )],
    };
    for refusal in refusals {
        let devices_behind: Vec<DeviceIdentity> = match refusal {
            Some(EntryRefusal::Session(
                UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    devices: behind_devices,
                },
            ))
            | Some(EntryRefusal::Mount(
                MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    devices: behind_devices,
                },
            )) => behind_devices.iter().map(|behind| behind.device).collect(),
            other => panic!("{cell}：该被落后支拒，实际 {}", describe(&other)),
        };
        assert_eq!(
            devices_behind,
            vec![stale_device],
            "{cell}：点名换掉的那一块"
        );
    }
    assert_eq!(
        images_of(&devices),
        images_before,
        "{cell}：两块盘逐字节不变"
    );
    assert_eq!(
        writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices),
        control,
        "{cell}：之后可写挂载与对照逐字相同"
    );
}

// ─── C585：发布途中取下的、那次发布的根住的那块盘（落后支缺根那一半） ───

/// 会话里那次覆盖写 P 的录制流切在哪：P 的根槽 FUA 写之前（单元与记录已写、根槽没写：C585 那一形），或之后（根已落、系统配置轮换还没写）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhereThePublishIsCut {
    BeforeItsRootSlotWrite,
    AfterItsRootSlotWrite,
}

/// 会话做完 P 之后的池（会话的现行那一版是 P），盘表里 P 的根住的那块盘换成「P 之前的盘面 + P 的录制流切下的那一截」，另一块是 P 做完的样子。
struct RootDeviceOfThePublishTakenMidPublish {
    pool: PoolUnderTest<SparseBlockDevice>,
    devices: SparseDevices,
    root_device_of_the_publish: DeviceIdentity,
    ring_slot_of_the_publish_root: RootRingSlot,
}

/// P 的 txg：起点（第一个文件之后会话里覆盖写两次，txg 5）之后的下一次，根环槽在区域 0（盘 0）、槽 2，那一槽 P 之前没写过。
const PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO: CheckpointTxg = CheckpointTxg(6);
/// P 的 txg：根环槽在区域 1（盘 1，`region_devices` 是 [盘 0, 盘 1, 盘 0]）、槽 2，那一槽 P 之前没写过。
const PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ONE: CheckpointTxg = CheckpointTxg(7);
/// P 的 txg：根环 3 × 8 = 24 槽转过一圈，P 的根环槽（区域 0、槽 1）里住着上一圈 txg 3（第一个文件那一版）自证得过的根。
const PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE: CheckpointTxg = CheckpointTxg(27);
/// txg 27 那一版的根槽上一圈住着的那条根：第一个文件那一版（实例 1、txg 3）。
const ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE: RingSlotReadingOtherThanTheVersionRoot =
    RingSlotReadingOtherThanTheVersionRoot::AnotherRoot {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    };
/// P 之前没写过那一槽（mkfs 清零之后）：两次读都自证不过。
const NEVER_WRITTEN_RING_SLOT: RingSlotReadingOtherThanTheVersionRoot =
    RingSlotReadingOtherThanTheVersionRoot::Bad(BadRootRingSlotReading::NotSelfVerified);
/// P 的 txg：根环槽在区域 2（`region_devices` 是 [盘 0, 盘 1, 盘 0]，区域 2 归盘 0）、槽 2，那一槽 P 之前没写过。
const PUBLISH_TXG_WITH_ITS_ROOT_IN_REGION_TWO: CheckpointTxg = CheckpointTxg(8);
/// P 的 txg：根环转过一圈，P 的根环槽（区域 1、槽 0，住盘 1）里住着上一圈 txg 1（取号之后暖机写的第一条根）自证得过的根。
const PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_ON_DEVICE_ONE: CheckpointTxg =
    CheckpointTxg(25);
/// txg 25 那一版的根槽上一圈住着的那条根：暖机写的第一条根（实例 1、txg 1）。
const ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE: RingSlotReadingOtherThanTheVersionRoot =
    RingSlotReadingOtherThanTheVersionRoot::AnotherRoot {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(1),
    };
/// P 的 txg：根环转过一圈，P 的根环槽（区域 2、槽 0；`region_devices` 是 [盘 0, 盘 1, 盘 0]，区域 2 归盘 0）里住着上一圈 txg 2
/// （取号之后暖机写的第二条根）自证得过的根。
const PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_IN_REGION_TWO: CheckpointTxg =
    CheckpointTxg(26);
/// txg 26 那一版的根槽上一圈住着的那条根：暖机写的第二条根（实例 1、txg 2）。
const ROOT_OF_THE_SECOND_WARM_UP_PUBLISH_A_LAP_BEFORE: RingSlotReadingOtherThanTheVersionRoot =
    RingSlotReadingOtherThanTheVersionRoot::AnotherRoot {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(2),
    };

/// 起点同 Y2-a（第一个文件之后会话里覆盖写两次，txg 5），会话里再覆盖写到 `publish_txg` 的前一版，P 是下一次覆盖写（txg `publish_txg`），
/// 录下它的写流。切下的那块盘：落后于 P（它的系统配置轮换在根槽之后），而 P 的单元都在它上面（单元与记录都在根槽之前写）——
/// 落后支缺单元那一半放它过去。这两样在这里先核，钉的是攻方那一形，不是别的形。
fn root_device_of_the_publish_taken_mid_publish(
    cut: WhereThePublishIsCut,
    publish_txg: CheckpointTxg,
) -> RootDeviceOfThePublishTakenMidPublish {
    let (mut pool, _, _) = pool_with_a_snapshot_after_the_first_file();
    let txg_before_the_extra_overwrites = pool.current_file_version().root.checkpoint_txg;
    for _ in txg_before_the_extra_overwrites.0 + 1..publish_txg.0 {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("会话里覆盖写到 P 的前一版");
    }
    let before_the_publish = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&before_the_publish)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
                )
            })
            .collect();
    let (content, write_time_seconds) = overwrite_after(&pool, 0);
    pool.session
        .as_mut()
        .expect("这一刻有可写会话")
        .publish_user_change(
            &mut recorded,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds,
            }),
        )
        .expect("P：会话里第三次覆盖写");
    pool.write_time_seconds = write_time_seconds;
    let after_the_publish = MemoryPool {
        devices: recorded
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: before_the_publish.device_size_in_bytes,
    };
    pool.devices = plain_devices_on(&after_the_publish);
    let publish = pool.current_file_version().clone();
    assert_eq!(
        publish.root.checkpoint_txg, publish_txg,
        "P 是 txg {} 那一版",
        publish_txg.0
    );
    let geometry = pool.parameters.geometry;
    let ring_slot_of_the_publish_root = target_for_publish(
        publish.root.checkpoint_txg,
        geometry.root_ring_slots_per_region,
    );
    let root_device_of_the_publish = pool.parameters.region_devices
        [usize::try_from(ring_slot_of_the_publish_root.region).expect("区域号小于 3")];
    let root_slot_offset = slot_offset(
        ring_slot_of_the_publish_root,
        geometry.fixed_structure_slot_spacing,
    );
    let operations = stream.retained_operations();
    let root_slot_write = operations
        .iter()
        .position(|retained| {
            retained.operation.device == root_device_of_the_publish
                && retained.operation.offset == root_slot_offset
                && retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess
        })
        .expect("P 的录制流里有它的根槽 FUA 写");
    let operations_through_the_cut = match cut {
        WhereThePublishIsCut::BeforeItsRootSlotWrite => root_slot_write,
        WhereThePublishIsCut::AfterItsRootSlotWrite => root_slot_write + 1,
    };
    let mut cut_image = before_the_publish.clone();
    cut_image.apply(&operations[..operations_through_the_cut]);
    let devices: SparseDevices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let image = if identity == root_device_of_the_publish {
                &cut_image
            } else {
                &after_the_publish
            };
            (identity, copy_of_device(image, identity))
        })
        .collect();
    let cut_device = &devices
        .iter()
        .find(|(identity, _)| *identity == root_device_of_the_publish)
        .expect("盘表里有 P 的根住的那块盘")
        .1;
    assert_ne!(
        cut_device.image, after_the_publish.devices[&root_device_of_the_publish],
        "{cut:?}：切下的那块盘与 P 做完的样子不同（系统配置轮换还没写）"
    );
    let spacing = u64::from(geometry.fixed_structure_slot_spacing);
    let newest_self_verified = verified_system_configuration_slots(
        &devices,
        root_device_of_the_publish,
        spacing,
        &pool.parameters.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|slot| slot.quantities.slot_generation)
    .expect("切下的那块盘上有自证过的系统配置");
    assert!(
        (
            newest_self_verified.quantities.journal_instance,
            newest_self_verified.quantities.journal_tail
        ) < (publish.record.instance, publish.record.counter),
        "{cut:?}：切下的那块盘落后于 P（系统配置停在 P 之前那次发布）"
    );
    for unit in publish.units.iter().filter(|unit| !unit.bytes.is_empty()) {
        let mut bytes_on_the_cut_device = vec![0u8; unit.bytes.len()];
        cut_device
            .read_at(unit.slot.to_device_offset(), &mut bytes_on_the_cut_device)
            .expect("读 P 的单元在切下的那块盘上的落点");
        assert_eq!(
            bytes_on_the_cut_device, unit.bytes,
            "{cut:?}：P 的单元 {:?} 在切下的那块盘上",
            unit.identity
        );
    }
    RootDeviceOfThePublishTakenMidPublish {
        pool,
        devices,
        root_device_of_the_publish,
        ring_slot_of_the_publish_root,
    }
}

/// 落后支缺根那一半拒的：点名 P 的根住的那块盘，缺的是根——那个根环槽两次读到的都是 `reading_of_the_ring_slot`
/// （这一槽 P 之前没写过：mkfs 清零之后自证不过；根环转过一圈：上一圈那条自证得过的根）。
fn assert_refused_as_behind_without_the_root(
    refusal: Option<EntryRefusal>,
    entry: EntryAfterTheMount,
    root_device_of_the_publish: DeviceIdentity,
    ring_slot_of_the_publish_root: RootRingSlot,
    reading_of_the_ring_slot: RingSlotReadingOtherThanTheVersionRoot,
) {
    let devices_behind: Vec<DeviceBehindTheCurrentVersion> = match refusal {
        Some(EntryRefusal::Session(
            UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Mount(
            MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { devices },
        ))
        | Some(EntryRefusal::Rollback(RollbackError::CallerInputsDisagreeWithTheDisk(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                devices,
            },
        ))) => devices,
        other => panic!(
            "{entry:?}：该被落后支缺根那一半拒，实际 {}",
            describe(&other)
        ),
    };
    let lacking: Vec<(DeviceIdentity, VersionLackingOnADeviceBehindIt)> = devices_behind
        .into_iter()
        .map(|behind| (behind.device, behind.lacking))
        .collect();
    assert_eq!(
        lacking,
        vec![(
            root_device_of_the_publish,
            VersionLackingOnADeviceBehindIt::RootNotInItsRingSlot(VersionRootNotInItsRingSlot {
                ring_slot: ring_slot_of_the_publish_root,
                first_reading: reading_of_the_ring_slot,
                reread: reading_of_the_ring_slot,
            }),
        )],
        "{entry:?}：点名 P 的根住的那块盘，单元都在、缺的是 P 的根"
    );
}

/// C585 的一格：P（txg `publish_txg`）的根住的那块盘换成切在 P 的根槽 FUA 之前的样子，经 `entry` 交进去。在任何写之前拒成
/// 落后支缺根那一半（那一槽两次读到的都是 `reading_of_the_ring_slot`），两块盘逐字节不变、会话的现行版本不动。交回 P 的根住的那块盘。
/// 改回「落后且缺单元才拒」时会话发布照常做成（之后池级 checker 红 I-3.1），另三个入口拒在读根环那一步（`RootRingSlotKnownToHoldARoot`），
/// 报的不是落后支——根环转过一圈的那一形除外：P 的根槽里是上一圈自证得过的根，读根环那一步不拒，另三个入口也照常做成。
/// `entry` 是管理员回退时回退到 `rollback_target`。
fn root_device_taken_before_the_publish_root_slot_write_through(
    entry: EntryAfterTheMount,
    rollback_target: RollbackTargetOfTheEntry,
    publish_txg: CheckpointTxg,
    reading_of_the_ring_slot: RingSlotReadingOtherThanTheVersionRoot,
) -> DeviceIdentity {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        mut devices,
        root_device_of_the_publish,
        ring_slot_of_the_publish_root,
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::BeforeItsRootSlotWrite,
        publish_txg,
    );
    let images_before = images_of(&devices);
    let current_before = pool.current_file_version().root;
    let refusal = go_through_rolling_back_to(entry, rollback_target, &mut pool, &mut devices);
    assert_refused_as_behind_without_the_root(
        refusal,
        entry,
        root_device_of_the_publish,
        ring_slot_of_the_publish_root,
        reading_of_the_ring_slot,
    );
    assert_eq!(
        images_of(&devices),
        images_before,
        "{entry:?}：两块盘逐字节不变"
    );
    assert_eq!(
        pool.current_file_version().root,
        current_before,
        "{entry:?}：会话的现行版本不动"
    );
    root_device_of_the_publish
}

#[test]
fn a_session_publish_onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write(
) {
    root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        NEVER_WRITTEN_RING_SLOT,
    );
}

#[test]
fn a_normal_unmount_onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write(
) {
    root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::NormalUnmount,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        NEVER_WRITTEN_RING_SLOT,
    );
}

#[test]
fn a_floor_raise_onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write(
) {
    root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        NEVER_WRITTEN_RING_SLOT,
    );
}

#[test]
fn an_administrator_rollback_onto_the_root_device_taken_before_the_publish_root_slot_write_is_refused_before_any_write(
) {
    root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::AdministratorRollback,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        NEVER_WRITTEN_RING_SLOT,
    );
}

/// 不误拒：P 的根住的那块盘切在 P 的根槽 FUA 之后（根已落、系统配置轮换没写）。它落后、单元都在、根槽里就是 P 的根——
/// 与「根落盘之后、轮换之前崩」同一个盘面，会话照常发布，做完之后池级 checker 不红。
#[test]
fn a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused() {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        mut devices,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let refusal = go_through(EntryAfterTheMount::SessionPublish, &mut pool, &mut devices);
    assert!(
        refusal.is_none(),
        "根槽里是 P 的根：不误拒，实际 {}",
        describe(&refusal)
    );
    let after = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: pool.device_bytes,
    };
    assert_eq!(
        checker_violations_on(&after),
        Vec::<String>::new(),
        "做完之后池级 checker 不红"
    );
}

/// 不误拒：同一组盘（P 的根住的那块切在根槽 FUA 之前）上可写挂载。所选那一版是在 txg 5 的根上施加 P 的记录得来的，P 的根不在根环里，
/// 落后支不核缺根那一半；P 的单元都在，照常挂上、所选那一版就是 P（攻方 X6-a「崩了再可写挂载」那一格：择 P、之后 checker 0）。
#[test]
fn a_writable_mount_onto_the_root_device_taken_before_the_publish_root_slot_write_applies_the_publish_record_and_mounts(
) {
    let RootDeviceOfThePublishTakenMidPublish { pool, devices, .. } =
        root_device_of_the_publish_taken_mid_publish(
            WhereThePublishIsCut::BeforeItsRootSlotWrite,
            PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
        );
    let publish_root = pool.current_file_version().root;
    let mut copies = copies_of(&devices);
    let mounted = mount_writable(&pool.parameters, &mut copies)
        .unwrap_or_else(|error| panic!("P 的根不在根环里、单元都在：照常可写挂载，实际 {error:?}"));
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (publish_root.instance, publish_root.checkpoint_txg),
        "所选那一版是施加 P 的记录得来的 P"
    );
}

// ─── 代码三方 m3-code-r1 H2：P 的根住盘 1；根环转过一圈 ───

/// H2 的一格（攻方改错 A「缺根那一半只读 region 0 那块盘的根槽」）：P 是 txg 7，根环槽在区域 1、住盘 1。盘 1 换成切在 P 的根槽 FUA
/// 之前的样子，会话发布在任何写之前拒成落后支缺根那一半、点名盘 1。只查盘 0 的根槽时盘 1 的缺根不核、会话照常发布。
#[test]
fn a_session_publish_onto_device_one_holding_the_publish_root_taken_before_its_root_slot_write_is_refused_before_any_write(
) {
    let root_device_of_the_publish = root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ONE,
        NEVER_WRITTEN_RING_SLOT,
    );
    assert_eq!(
        root_device_of_the_publish,
        DeviceIdentity(1),
        "txg 7 的根环槽在区域 1、住盘 1"
    );
}

/// H2 的一格（攻方改错 E「根槽里自证得过的根都算是它」）：根环转过一圈，P 是 txg 27，根环槽（区域 0、槽 1）里住着上一圈
/// 第一个文件那一版（txg 3）自证得过的根。P 的根住的那块盘换成切在 P 的根槽 FUA 之前的样子，会话发布在任何写之前拒成
/// 落后支缺根那一半，两次读到的都是 `AnotherRoot`（实例 1、txg 3）。自证得过就算是它时上一圈的旧根被当成 P 的根、会话照常发布。
#[test]
fn a_session_publish_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    let root_device_of_the_publish = root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE,
        ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE,
    );
    assert_eq!(
        root_device_of_the_publish,
        DeviceIdentity(0),
        "txg 27 的根环槽在区域 0、住盘 0"
    );
}

/// J1 的一格（代码三方 m3-code-r2，攻方改错 RG「把区域号当盘号」）：P 是 txg 8，根环槽在区域 2、槽 2，区域 2 归盘 0。
/// 盘 0 换成切在 P 的根槽 FUA 之前的样子，会话发布在任何写之前拒成落后支缺根那一半、点名盘 0。
/// 拿区域号当盘号时区域 2 被当成盘表里没有的盘 2，盘 0 的缺根不核、会话照常发布（之后池级 checker 红 I-3.1）。
#[test]
fn a_session_publish_onto_device_zero_holding_the_publish_root_in_region_two_taken_before_its_root_slot_write_is_refused_before_any_write(
) {
    let root_device_of_the_publish = root_device_taken_before_the_publish_root_slot_write_through(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        PUBLISH_TXG_WITH_ITS_ROOT_IN_REGION_TWO,
        NEVER_WRITTEN_RING_SLOT,
    );
    assert_eq!(
        root_device_of_the_publish,
        DeviceIdentity(0),
        "txg 8 的根环槽在区域 2、区域 2 归盘 0"
    );
}

// ─── 代码三方 m3-code-r1 H1、H3：缺根那一半的「重读一次」 ───

/// 缺根那一半第一次读 P 的根槽，在会话发布里是那一槽被读的第几次（从 1 数）：会话发布读根环之前只核这一判。
const ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_SESSION_PUBLISH: u64 = 1;
/// 缺根那一半第一次读 P 的根槽，在可写挂载里是那一槽被读的第几次（从 1 数）：读阶段（择根、见证读、重建）先读过它 7 次，
/// 不出错时第 8 次就是取号之前逐盘核那一判的读，也是这一槽在取号之前的最后一次读（攻方原型扫出来的；
/// [`a_writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_whose_root_slot_fails_two_reads_in_a_row_is_refused_before_any_write`]
/// 从第 8 次起读坏、恰好坏两次就拒成缺根，钉住它是那一判的第一次读）。
const ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_WRITABLE_MOUNT: u64 = 8;
/// 缺根那一半第一次读 P 的根槽，在正常卸载、准入抬 F、管理员回退里是那一槽被读的第几次（从 1 数）：这三个入口在读根环之前先核盘表
/// （落后支在里面），第 1 次就是它（攻方 `research/prompts/m3-code-r2-opus-model/logs/` 里 txg 6 那一扫：原样代码第 1 次读坏照常做成、
/// 三个入口不重读时第 1 次读坏就拒成缺根）。
const ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_THE_OTHER_THREE_ENTRIES: u64 = 1;

/// 根槽这一次读回的字节坏的那一位：根记录里 checkpoint_txg 字段的头一个字节（魔数 4、文件系统标识 16、标志 4、实例代号 4 字节之后，
/// `crates/singlefs-core/src/root_record.rs` 的 `RootRecord::parse_slot`）的最低位。自证校验和罩着整个根槽，翻哪一位都自证不过；
/// 取 txg 字段是让「自证不过」不靠魔数那一判先拒。
const BIT_FLIPPED_IN_THE_CHECKPOINT_TXG_OF_THE_ROOT_SLOT: FlippedBit = FlippedBit {
    byte_index: 28,
    bit_index: 0,
};
/// 根槽这一次读报成功、读回的字节坏一位（L1：自证不过的那一种读坏）。
const ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT: InjectedFault =
    InjectedFault::ReadReturnsCorruptedBytes {
        flipped_bit: BIT_FLIPPED_IN_THE_CHECKPOINT_TXG_OF_THE_ROOT_SLOT,
    };

/// 盘表里每块盘包上同一份注入计划：P 的根环槽 `ring_slot` 上的读（整池数、只数落在这一槽里的读）按 `occurrence` 注入 `fault`
/// （报块设备错 [`InjectedFault::ReadFails`]，或读报成功而字节坏一位 [`ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT`]）。
fn with_a_fault_on_reads_of_the_publish_root_slot(
    devices: SparseDevices,
    pool: &PoolUnderTest<SparseBlockDevice>,
    ring_slot: RootRingSlot,
    fault: InjectedFault,
    occurrence: FaultOccurrence,
) -> (
    Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)>,
    SharedFaultPlan,
) {
    let plan = SharedFaultPlan::armed(
        DEVICE_WIDTH.fixed_geometry(),
        FaultSchedule {
            fault,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::WithinNamedRootRingSlots(RootRingSlotTarget {
                named_slots: NamedRootRingSlots::NONE.with(ring_slot),
                region_devices: pool.parameters.region_devices,
                fixed_structure_slot_spacing: pool.parameters.geometry.fixed_structure_slot_spacing,
            }),
            counting: FaultCounting::AcrossThePool,
            occurrence,
        },
    );
    let wrapped = devices
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    (wrapped, plan)
}

/// H1 的一格：P 的根住的那块盘切在 P 的根槽 FUA 之后（落后、单元都在、根槽里是 P），会话发布时缺根那一半第一次读那一槽读坏
/// （按 `fault`：读不出，或读回的字节坏一位、自证不过）、重读读得出 P：照常发布，那一次注入真发生了。
/// 去掉重读（第一次读不是它就拒）时两种读坏都拒成落后支缺根那一半；只在第一次读不出时才重读（攻方 RU）时字节坏一位的那一种拒。
fn session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
    fault: InjectedFault,
) {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        devices,
        ring_slot_of_the_publish_root,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let (mut wrapped, plan) = with_a_fault_on_reads_of_the_publish_root_slot(
        devices,
        &pool,
        ring_slot_of_the_publish_root,
        fault,
        FaultOccurrence::TheNthMatchingCall(
            ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_SESSION_PUBLISH,
        ),
    );
    let (content, write_time_seconds) = overwrite_after(&pool, 0);
    let outcome = pool
        .session
        .as_mut()
        .expect("这一刻有可写会话")
        .publish_user_change(
            &mut wrapped,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds,
            }),
        );
    assert_eq!(
        plan.fired_count(),
        1,
        "{fault:?}：P 的根槽上那一次读坏真发生了：{:?}",
        plan.fired()
    );
    assert!(
        outcome.is_ok(),
        "{fault:?}：一次读坏、重读读得出 P：会话照常发布，实际 {:?}",
        outcome.err()
    );
}

#[test]
fn a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
        InjectedFault::ReadFails,
    );
}

/// L1 的一格：H1 会话发布那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
#[test]
fn a_session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    session_publish_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

/// H1 的一格：同一组盘（P 的根住的那块切在根槽 FUA 之后）上可写挂载，所选那一版就是择到的 P（写进了根环），
/// 取号之前逐盘核时缺根那一半第一次读那一槽读坏（按 `fault`）、重读读得出 P：照常挂上、所选那一版是 P，那一次注入真发生了。
/// 去掉重读时两种读坏都拒成 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`；只在第一次读不出时才重读（攻方 RU）时
/// 字节坏一位的那一种拒。
fn writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
    fault: InjectedFault,
) {
    let RootDeviceOfThePublishTakenMidPublish {
        pool,
        devices,
        ring_slot_of_the_publish_root,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let publish_root = pool.current_file_version().root;
    let (mut wrapped, plan) = with_a_fault_on_reads_of_the_publish_root_slot(
        copies_of(&devices),
        &pool,
        ring_slot_of_the_publish_root,
        fault,
        FaultOccurrence::TheNthMatchingCall(ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_WRITABLE_MOUNT),
    );
    let mounted = mount_writable(&pool.parameters, &mut wrapped);
    assert_eq!(
        plan.fired_count(),
        1,
        "{fault:?}：P 的根槽上那一次读坏真发生了：{:?}",
        plan.fired()
    );
    let mounted = mounted.unwrap_or_else(|error| {
        panic!("{fault:?}：一次读坏、重读读得出 P：照常可写挂载，实际 {error:?}")
    });
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (publish_root.instance, publish_root.checkpoint_txg),
        "{fault:?}：所选那一版是 P"
    );
}

#[test]
fn a_writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
        InjectedFault::ReadFails,
    );
}

/// L1 的一格：H1 可写挂载那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
#[test]
fn a_writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_with_one_bad_read_of_its_root_slot(
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

/// H3：同一组盘（P 的根住的那块切在根槽 FUA 之后）上可写挂载，所选那一版就是择到的 P，取号之前逐盘核时缺根那一半读那一槽
/// 连着两次读坏（从第 8 次起每一次都坏，真发生的恰好是两次：读一次、重读一次）：拒成 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，
/// 点名 P 的根住的那块盘、缺的是根（两次都读不出），拒在任何写之前（整池一次写都没发）、两块盘逐扇区不变。
/// 可写挂载一律不核缺根那一半（所选那一版一律当成没写进根环）时，这一形挂得上或拒成别的。
#[test]
fn a_writable_mount_onto_the_root_device_taken_after_the_publish_root_slot_write_whose_root_slot_fails_two_reads_in_a_row_is_refused_before_any_write(
) {
    let RootDeviceOfThePublishTakenMidPublish {
        pool,
        devices,
        root_device_of_the_publish,
        ring_slot_of_the_publish_root,
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let publish_root = pool.current_file_version().root;
    let images_before = images_of(&devices);
    let (mut wrapped, plan) = with_a_fault_on_reads_of_the_publish_root_slot(
        copies_of(&devices),
        &pool,
        ring_slot_of_the_publish_root,
        InjectedFault::ReadFails,
        FaultOccurrence::EveryMatchingCallFromTheNthOnward(
            ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_A_WRITABLE_MOUNT,
        ),
    );
    let refusal = mount_writable(&pool.parameters, &mut wrapped).err();
    let devices_without = match refusal {
        Some(MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
            selected_version,
            devices: devices_without,
            ..
        }) => {
            assert_eq!(
                selected_version,
                RollbackTarget {
                    instance: publish_root.instance,
                    checkpoint_txg: publish_root.checkpoint_txg,
                },
                "所选那一版是择到的 P"
            );
            devices_without
        }
        other => {
            panic!("该拒成 WritableMountRefusedByDevicesWithoutTheSelectedVersion，实际 {other:?}")
        }
    };
    let lacking: Vec<(DeviceIdentity, Option<VersionRootNotInItsRingSlot>)> = devices_without
        .into_iter()
        .map(|DeviceWithoutTheSelectedVersion { device, lacking }| {
            let root_not_in_its_ring_slot = match lacking {
                SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndItsRootNotInItsRingSlot {
                    root_not_in_its_ring_slot,
                    ..
                } => Some(root_not_in_its_ring_slot),
                SelectedVersionLackingOnDevice::BehindTheSelectedVersionAndMissingItsUnits {
                    ..
                } => None,
            };
            (device, root_not_in_its_ring_slot)
        })
        .collect();
    let unreadable =
        RingSlotReadingOtherThanTheVersionRoot::Bad(BadRootRingSlotReading::Unreadable);
    assert_eq!(
        lacking,
        vec![(
            root_device_of_the_publish,
            Some(VersionRootNotInItsRingSlot {
                ring_slot: ring_slot_of_the_publish_root,
                first_reading: unreadable,
                reread: unreadable,
            })
        )],
        "点名 P 的根住的那块盘，缺的是根（那一槽读一次、重读一次都读不出），不是单元"
    );
    let images_after: Vec<SparseDevice> = wrapped
        .iter()
        .map(|(_, device)| device.wrapped_device().image.clone())
        .collect();
    assert_eq!(plan.calls().writes, 0, "拒在任何写之前：整池一次写都没发");
    assert_eq!(images_after, images_before, "两块盘逐扇区不变");
    assert_eq!(
        plan.fired_count(),
        2,
        "那一槽真读坏的恰好两次（缺根那一半读一次、重读一次）：{:?}",
        plan.fired()
    );
}

// ─── 代码三方 m3-code-r2 J2：另三个入口上缺根那一半的「重读一次」 ───

/// J2 的一格（攻方改错 RP「正常卸载、准入抬 F、管理员回退不重读」）：P 的根住的那块盘切在 P 的根槽 FUA 之后（落后、单元都在、
/// 根槽里是 P），经 `entry` 交进去时缺根那一半第一次读那一槽读坏（按 `fault`：读不出，或读回的字节坏一位、自证不过）、
/// 重读读得出 P：照常做成，那一次注入真发生了，做完之后池级 checker 不红。
/// 这三个入口不重读（第一次读不是它就拒）时两种读坏都拒成落后支缺根那一半；只在第一次读不出时才重读（攻方 RU）时字节坏一位的那一种拒。
fn one_bad_read_of_the_publish_root_slot_through(entry: EntryAfterTheMount, fault: InjectedFault) {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        devices,
        ring_slot_of_the_publish_root,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let (mut wrapped, plan) = with_a_fault_on_reads_of_the_publish_root_slot(
        devices,
        &pool,
        ring_slot_of_the_publish_root,
        fault,
        FaultOccurrence::TheNthMatchingCall(
            ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_THE_OTHER_THREE_ENTRIES,
        ),
    );
    let refusal = go_through(entry, &mut pool, &mut wrapped);
    assert_eq!(
        plan.fired_count(),
        1,
        "{entry:?}，{fault:?}：P 的根槽上那一次读坏真发生了：{:?}",
        plan.fired()
    );
    assert!(
        refusal.is_none(),
        "{entry:?}，{fault:?}：一次读坏、重读读得出 P：照常做成，实际 {}",
        describe(&refusal)
    );
    let after = MemoryPool {
        devices: wrapped
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: pool.device_bytes,
    };
    assert_eq!(
        checker_violations_on(&after),
        Vec::<String>::new(),
        "{entry:?}，{fault:?}：做完之后池级 checker 不红"
    );
}

#[test]
fn a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    one_bad_read_of_the_publish_root_slot_through(
        EntryAfterTheMount::NormalUnmount,
        InjectedFault::ReadFails,
    );
}

#[test]
fn a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    one_bad_read_of_the_publish_root_slot_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        InjectedFault::ReadFails,
    );
}

#[test]
fn an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_transient_read_error_of_its_root_slot(
) {
    one_bad_read_of_the_publish_root_slot_through(
        EntryAfterTheMount::AdministratorRollback,
        InjectedFault::ReadFails,
    );
}

/// L1 的一格：J2 正常卸载那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
#[test]
fn a_normal_unmount_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    one_bad_read_of_the_publish_root_slot_through(
        EntryAfterTheMount::NormalUnmount,
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

/// L1 的一格：J2 准入抬 F 那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
#[test]
fn a_floor_raise_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    one_bad_read_of_the_publish_root_slot_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

/// L1 的一格：J2 管理员回退那一格换成根槽第一次读回的字节坏一位（读报成功、自证不过）。
#[test]
fn an_administrator_rollback_onto_the_root_device_taken_after_the_publish_root_slot_write_is_not_refused_for_one_corrupted_reading_of_its_root_slot(
) {
    one_bad_read_of_the_publish_root_slot_through(
        EntryAfterTheMount::AdministratorRollback,
        ROOT_SLOT_READ_RETURNING_A_FLIPPED_BIT,
    );
}

// ─── 代码三方 m3-code-r2 J3：另三个入口上根环转过一圈 ───

/// J3 的一格（攻方改错 EC「另三个入口上根槽里自证得过就算是它」、AC「另三个入口只查区域 0 那块盘」）：根环转过一圈，P 是
/// `publish_txg`，P 的根住的那块盘（`root_device_of_the_publish`）换成切在 P 的根槽 FUA 之前的样子，那一槽里是上一圈自证得过的根
/// （`root_of_the_previous_lap`）。经 `entry`（管理员回退取现行那一版的前一版做目标）交进去，在任何写之前拒成落后支缺根那一半，
/// 两次读到的都是上一圈那条根，两块盘逐字节不变。这一形上读根环那一步不拒（槽里自证得过），缺根那一半是这三个入口唯一的一道：
/// 去掉它、或上一圈的根被当成 P 的根、或根住的盘不是区域 0 那块而只查区域 0 那块时，入口照常做成、之后池级 checker 红 I-3.1。
fn root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
    entry: EntryAfterTheMount,
    publish_txg: CheckpointTxg,
    root_of_the_previous_lap: RingSlotReadingOtherThanTheVersionRoot,
    root_device_of_the_publish: DeviceIdentity,
) {
    assert_eq!(
        root_device_taken_before_the_publish_root_slot_write_through(
            entry,
            RollbackTargetOfTheEntry::VersionBeforeTheCurrent,
            publish_txg,
            root_of_the_previous_lap,
        ),
        root_device_of_the_publish,
        "{entry:?}：txg {} 的根住的那块盘",
        publish_txg.0
    );
}

#[test]
fn a_normal_unmount_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::NormalUnmount,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE,
        ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

#[test]
fn a_floor_raise_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE,
        ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

#[test]
fn an_administrator_rollback_onto_the_root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::AdministratorRollback,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE,
        ROOT_OF_THE_FIRST_FILE_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

#[test]
fn a_normal_unmount_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::NormalUnmount,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_ON_DEVICE_ONE,
        ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(1),
    );
}

#[test]
fn a_floor_raise_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_ON_DEVICE_ONE,
        ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(1),
    );
}

#[test]
fn an_administrator_rollback_onto_device_one_whose_ring_slot_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::AdministratorRollback,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_ON_DEVICE_ONE,
        ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(1),
    );
}

// ─── 代码三方 m3-code-r3 L2：另三个入口上根环转过一圈、根住区域 2 ───

/// L2 的一格（攻方改错 RGC「正常卸载、准入抬 F、管理员回退那条路上把区域号当盘号」）：根环转过一圈，P 是 txg 26，根环槽在区域 2、槽 0，
/// 区域 2 归盘 0，那一槽里是上一圈暖机写的第二条根（实例 1、txg 2）。盘 0 换成切在 P 的根槽 FUA 之前的样子，经正常卸载交进去，
/// 在任何写之前拒成落后支缺根那一半、点名盘 0，两次读到的都是上一圈那条根。把区域号当盘号时区域 2 被当成盘表里没有的盘 2，
/// 盘 0 的缺根不核，读根环那一步也不拒（槽里自证得过），入口照常做成。
#[test]
fn a_normal_unmount_onto_device_zero_whose_ring_slot_in_region_two_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::NormalUnmount,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_IN_REGION_TWO,
        ROOT_OF_THE_SECOND_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

/// L2 的一格：同上，经准入抬 F 交进去。
#[test]
fn a_floor_raise_onto_device_zero_whose_ring_slot_in_region_two_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::FloorRaiseToTheAdmissionCeiling,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_IN_REGION_TWO,
        ROOT_OF_THE_SECOND_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}

/// L2 的一格：同上，经管理员回退交进去（目标取现行那一版的前一版）。
#[test]
fn an_administrator_rollback_onto_device_zero_whose_ring_slot_in_region_two_holds_the_root_of_the_previous_lap_is_refused_before_any_write(
) {
    root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(
        EntryAfterTheMount::AdministratorRollback,
        PUBLISH_TXG_ONE_LAP_AFTER_THE_WARM_UP_WITH_ITS_ROOT_IN_REGION_TWO,
        ROOT_OF_THE_SECOND_WARM_UP_PUBLISH_A_LAP_BEFORE,
        DeviceIdentity(0),
    );
}
