//! 代码三方 m2-closeout-code-r2（判决 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节 Y2-a、Y2-b，第二节「四个入口再核两样」；
//! 用户 2026-09-27 定「采纳，四个入口都核」）：挂着之后收盘表的每个入口——会话发布、正常卸载、抬 F、管理员回退——在「可见」
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

mod common_admission;

use common_admission::{content_of, start_plain, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish,
    unmount, CallerInputsDisagreeingWithTheDisk, DeviceBehindTheCurrentVersion,
    DeviceTableDisagreement, MountError, OwnDeviceNumberOnDiskDifferingFromTheIdentityHandedIn,
    RollbackError, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::transaction::{FirstFile, PoolVersion, TransactionOutput};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

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
        PoolVersion::WithFile(current) => current,
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

/// 拿池里那条会话，经 `entry` 把这组盘交进去一次。
fn go_through(
    entry: EntryAfterTheMount,
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    devices: &mut SparseDevices,
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
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                allocator,
                file_version_of(current),
                FIRST_FILE_VERSION,
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
            UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnits { devices },
        ))
        | Some(EntryRefusal::Mount(
            MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits { devices },
        ))
        | Some(EntryRefusal::Rollback(RollbackError::CallerInputsDisagreeWithTheDisk(
            CallerInputsDisagreeingWithTheDisk::DevicesBehindTheCurrentVersionAndMissingItsUnits {
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
        !behind.units_missing.is_empty(),
        "{entry:?}：现行那一版有单元在旧快照上不在"
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
/// 都在盘 1 上。四个入口都照常做成（不误拒），做完之后池级 checker 不红、照常可写挂载。
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
        assert!(
            writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices).is_ok(),
            "{entry:?}：之后照常可写挂载"
        );
    }
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
    // 线程数取 SINGLEFS_THREAD_CAP，没设取 available_parallelism；每格自己断言，一格红就在 join 时带着它的名字红。
    let thread_count = std::env::var("SINGLEFS_THREAD_CAP")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from))
        .clamp(1, cells.len());
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
                UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnits {
                    devices: behind_devices,
                },
            ))
            | Some(EntryRefusal::Mount(
                MountError::DevicesBehindTheCurrentVersionAndMissingItsUnits {
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
