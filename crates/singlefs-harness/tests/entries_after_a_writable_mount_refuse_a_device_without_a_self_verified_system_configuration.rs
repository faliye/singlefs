//! 代码三方 m2-closeout-code-r1 Z3-A（判决 `research/prompts/m2-closeout-code-r1-main-verification.md` 第一节 Z3-A、第二节「Z3-A 乙」；
//! 用户 2026-09-27 定「乙：每个入口都逐盘核，含会话发布」）：挂着之后收盘表的入口（正常卸载、管理员回退、抬 F、准入抬 F、
//! 一次准入里再推一串）与会话发布，交进来的每块盘要「可见」——两个系统配置槽里至少一份本池自证过的（整槽校验和过、fsid 与本池相同），
//! 与可写挂载取号之前的逐盘核第一支同一判（D18（块里携带什么信息） 已定项 11「取号之前逐盘核带不带所选那一版」）。
//!
//! 攻方那一轮的四条用例（`research/prompts/m2-closeout-code-r1-opus-model/opus_r1_z3_entries_with_a_substituted_device.rs`）钉的是改之前的样子：
//! 盘 1 换成一块全零的空盘，卸载报成功、空盘收到 28 次写，回退到现行那一版做成、空盘收到 12 次写，会话照样发布。这里搬过来反过来钉：
//! 这几个入口都在任何写之前拒、点名盘 1，两块盘逐字节不变；别的池的盘换进来，挂着之后的入口照旧被择系统配置那一步拒（对照），
//! 会话不择系统配置、由这一判拒；一块盘两槽坏一槽仍可见（「至少一份」的边界）。
//!
//! 起点同攻方：两块 4 GiB 内存稀疏盘上 mkfs、取号、暖机、第一个文件（txg 3），会话里覆盖写两次（txg 4、5）。

mod common_admission;

use common_admission::{
    content_of, start_plain, DeviceOverASparseImage, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    push_one_floor_raise_within_the_admission_budget, raise_rollback_floor,
    raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, unmount,
    CallerInputsDisagreeingWithTheDisk, DeviceTableDisagreement, MakeFilesystemParameterField,
    MountError, RollbackError, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::recovery::RecoveryFailure;
use singlefs_core::transaction::{FirstFile, PoolVersion, TransactionOutput};
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICE_WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::FourGibibytes;

/// 换上去的那块盘占盘表里的哪一个身份。
const SUBSTITUTED_DEVICE: DeviceIdentity = DeviceIdentity(1);

type RecordedDevice = RecordingBlockDevice<SparseBlockDevice>;

/// 第一个文件之后在会话里覆盖写两次（txg 4、5）：交回池与此刻的镜像（攻方那一轮的起点）。
fn pool_after_two_overwrites() -> (PoolUnderTest<SparseBlockDevice>, MemoryPool) {
    let mut pool = start_plain(DEVICE_WIDTH);
    pool.overwrite(OVERWRITE_BYTES)
        .expect("会话里第一次覆盖写（txg 4）");
    pool.overwrite(OVERWRITE_BYTES)
        .expect("会话里第二次覆盖写（txg 5）");
    let image = pool.image();
    (pool, image)
}

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

/// 一块与池里的盘同样大、一个字节都没写过的盘。
fn blank_device(image: &MemoryPool) -> SparseBlockDevice {
    SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512))
}

/// 另一个池（fsid 首字节翻转，别的参数相同）mkfs 之后的盘 1：两个系统配置槽都自证得过，只是 fsid 不是本池的。
fn device_one_of_another_pool() -> SparseBlockDevice {
    let mut other_parameters = DEVICE_WIDTH.parameters();
    other_parameters.filesystem_identifier[0] ^= 0xff;
    let mut other_devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(
                        DEVICE_WIDTH.device_bytes(),
                        PhysicalBlockSizeInBytes(512),
                    ),
                )
            })
            .collect();
    make_filesystem(&other_parameters, &mut other_devices).expect("另一个池 mkfs 做得成");
    other_devices
        .into_iter()
        .find(|(identity, _)| *identity == SUBSTITUTED_DEVICE)
        .map(|(_, device)| device)
        .expect("另一个池里有盘 1")
}

/// 盘表 [(盘 0, 此刻盘 0 的拷贝), (盘 1, `device_one`)]，外面各包一层录制（两块盘共用 `stream`）。
fn recorded_devices_with_device_one_replaced(
    image: &MemoryPool,
    device_one: SparseBlockDevice,
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, RecordedDevice)> {
    vec![
        (
            DeviceIdentity(0),
            RecordingBlockDevice::with_shared_stream(
                DeviceIdentity(0),
                copy_of_device(image, DeviceIdentity(0)),
                stream.clone(),
            ),
        ),
        (
            SUBSTITUTED_DEVICE,
            RecordingBlockDevice::with_shared_stream(
                SUBSTITUTED_DEVICE,
                device_one,
                stream.clone(),
            ),
        ),
    ]
}

/// 盘表 [(盘 0, 此刻盘 0 的拷贝), (盘 1, `device_one`)]，不包录制（会话那几条按镜像比）。
fn plain_devices_with_device_one_replaced(
    image: &MemoryPool,
    device_one: SparseBlockDevice,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    vec![
        (DeviceIdentity(0), copy_of_device(image, DeviceIdentity(0))),
        (SUBSTITUTED_DEVICE, device_one),
    ]
}

/// 盘表里每块盘此刻的镜像，按盘表次序。
fn images_of<Device: DeviceOverASparseImage>(
    devices: &[(DeviceIdentity, Device)],
) -> Vec<SparseDevice> {
    devices
        .iter()
        .map(|(_, device)| device.sparse_image().clone())
        .collect()
}

/// 挂着之后收盘表的入口报的那一拒（`MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration`）的两张清单；
/// 做成了、或拒成别的成员就判红。
fn device_table_refusal_of<Outcome>(
    outcome: Result<Outcome, MountError>,
    entry: &str,
) -> (
    Vec<MakeFilesystemParameterField>,
    Vec<DeviceTableDisagreement>,
) {
    match outcome {
        Err(MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
            disagreeing_fields,
            disagreeing_device_table,
        }) => (disagreeing_fields, disagreeing_device_table),
        Err(other) => panic!(
            "{entry}：盘 1 不可见，要拒成 CallerParametersDisagreeWithTheSelectedSystemConfiguration，实际 {other:?}"
        ),
        Ok(_) => panic!("{entry}：盘 1 不可见，要在任何写之前拒，实际做成了（攻方那一轮的样子）"),
    }
}

/// 参数都对得上，盘表只报盘 1 一份本池自证过的系统配置都没有（D18 已定项 11 的「不可见」）。
fn assert_only_device_one_is_named_not_visible(
    disagreeing_fields: &[MakeFilesystemParameterField],
    disagreeing_device_table: &[DeviceTableDisagreement],
    entry: &str,
) {
    assert_eq!(
        (disagreeing_fields, disagreeing_device_table),
        (
            &[][..],
            &[
                DeviceTableDisagreement::NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn {
                    identity_handed_in: SUBSTITUTED_DEVICE,
                }
            ][..]
        ),
        "{entry}：参数都对得上，盘表只报盘 1 两槽一份本池自证过的系统配置都没有"
    );
}

/// 录制流一步都没有、两块盘逐字节与入口之前相同（换上去的那块盘也没被写）。
fn assert_nothing_written(
    stream: &SharedStream,
    devices: &[(DeviceIdentity, RecordedDevice)],
    images_before: &[SparseDevice],
    entry: &str,
) {
    assert_eq!(
        stream.operation_count(),
        0,
        "{entry}：拒在任何写与屏障之前，两块盘上一步都没录到"
    );
    assert_eq!(
        images_of(devices),
        images_before,
        "{entry}：两块盘逐字节不变"
    );
}

/// 正常卸载做成、推了抬 F 那一串（现行那一版带文件）；做成了别的样子或报错就判红。
fn assert_unmount_raised_the_floor(outcome: Result<Unmounted, MountError>, situation: &str) {
    match outcome {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(_)) => {}
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => {
            panic!("{situation}：现行那一版带文件，卸载要推抬 F 那一串，实际一个字节都没写")
        }
        Err(error) => panic!("{situation}：要照常卸载，实际 {error:?}"),
    }
}

/// 抬 F 的三个入口（`singlefs_core::mount` 里收盘表的那三个）。
#[derive(Clone, Copy, Debug)]
enum FloorRaiseEntry {
    /// `raise_rollback_floor`：直接给新 F，只供测试强制进入的那一档。
    DirectlyToAGivenFloor,
    /// `raise_rollback_floor_to_the_admission_ceiling`：准入抬 F 到上限。
    ToTheAdmissionCeiling,
    /// `push_one_floor_raise_within_the_admission_budget`：一次准入里再推一串（会话推抬 F 走的那一个）。
    OnePushWithinTheAdmissionBudget,
}

impl FloorRaiseEntry {
    fn name(self) -> &'static str {
        match self {
            FloorRaiseEntry::DirectlyToAGivenFloor => "抬 F 到 3（直接给新 F）",
            FloorRaiseEntry::ToTheAdmissionCeiling => "准入抬 F 到上限",
            FloorRaiseEntry::OnePushWithinTheAdmissionBudget => "一次准入里再推一串",
        }
    }
}

/// 会话此刻的现行那一版（带文件）。
fn current_file_version(session: &MountedSession) -> TransactionOutput {
    session
        .current
        .file_version()
        .expect("第一个文件之后现行那一版带文件")
        .clone()
}

/// 盘 1 换成空盘，正常卸载在任何写之前拒、点名盘 1（攻方 `unmount_with_device_one_replaced_by_a_blank_disk`：改之前卸载报成功、
/// 空盘收到 28 次写，之后真盘再挂可写被拒、池级 checker 红 4 条）。两块盘逐字节不变，会话的现行版本不动；换回真的盘 1，同一个会话照常卸载。
#[test]
fn unmount_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_and_the_real_disk_still_unmounts(
) {
    let (mut pool, image) = pool_after_two_overwrites();
    let session = pool.session.as_mut().expect("会话开着");
    let root_before = *session.current.root();
    let stream = SharedStream::new();
    let mut devices =
        recorded_devices_with_device_one_replaced(&image, blank_device(&image), &stream);
    let images_before = images_of(&devices);
    let outcome = unmount(
        &pool.parameters,
        &mut devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    let (disagreeing_fields, disagreeing_device_table) =
        device_table_refusal_of(outcome, "正常卸载");
    assert_only_device_one_is_named_not_visible(
        &disagreeing_fields,
        &disagreeing_device_table,
        "正常卸载",
    );
    assert_nothing_written(&stream, &devices, &images_before, "正常卸载");
    assert_eq!(*session.current.root(), root_before, "会话的现行版本不动");
    let real_stream = SharedStream::new();
    let mut real_devices = recorded_devices_with_device_one_replaced(
        &image,
        copy_of_device(&image, SUBSTITUTED_DEVICE),
        &real_stream,
    );
    assert_unmount_raised_the_floor(
        unmount(
            &pool.parameters,
            &mut real_devices,
            &mut session.allocator,
            &mut session.current,
            ShadowLedger::On,
        ),
        "换回真的盘 1，同一个会话",
    );
}

/// 盘 1 换成空盘，管理员回退在任何写之前拒、点名盘 1：目标取现行那一版 (1, 5)（攻方：改之前做成、空盘收到 12 次写）与更旧的 (1, 3)
/// （改之前被复活集逐盘验顺手拒，现在在那之前由这一判拒）。两块盘逐字节不变，会话的现行版本不动。
#[test]
fn rolling_back_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_for_the_current_and_an_older_target(
) {
    let (mut pool, image) = pool_after_two_overwrites();
    let session = pool.session.as_mut().expect("会话开着");
    let current_txg = session.current.root().checkpoint_txg;
    assert_eq!(
        current_txg,
        CheckpointTxg(5),
        "起点：第一个文件 txg 3，之后覆盖写两次"
    );
    for target_txg in [current_txg, CheckpointTxg(3)] {
        let entry = format!("回退到 (1, {})", target_txg.0);
        let root_before = *session.current.root();
        let stream = SharedStream::new();
        let mut devices =
            recorded_devices_with_device_one_replaced(&image, blank_device(&image), &stream);
        let images_before = images_of(&devices);
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("第一个文件之后现行那一版带文件")
        };
        let outcome = roll_back_by_a_forward_publish(
            &pool.parameters,
            &mut devices,
            &mut session.allocator,
            current,
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: target_txg,
            },
        );
        let (disagreeing_fields, disagreeing_device_table) = match outcome {
            Err(RollbackError::CallerInputsDisagreeWithTheDisk(
                CallerInputsDisagreeingWithTheDisk::ParametersOrDeviceTableDisagreeWithTheSelectedSystemConfiguration {
                    disagreeing_fields,
                    disagreeing_device_table,
                },
            )) => (disagreeing_fields, disagreeing_device_table),
            Err(other) => panic!(
                "{entry}：盘 1 不可见，要拒成 CallerInputsDisagreeWithTheDisk(ParametersOrDeviceTable…)，实际 {other:?}"
            ),
            Ok(rolled_back) => panic!(
                "{entry}：盘 1 不可见，要在任何写之前拒，实际做成了（攻方那一轮的样子）：{rolled_back:?}"
            ),
        };
        assert_only_device_one_is_named_not_visible(
            &disagreeing_fields,
            &disagreeing_device_table,
            &entry,
        );
        assert_nothing_written(&stream, &devices, &images_before, &entry);
        assert_eq!(
            *session.current.root(),
            root_before,
            "{entry}：会话的现行版本不动"
        );
    }
}

/// 盘 1 换成空盘，抬 F 的三个入口（直接给新 F 的只供测试那一档、准入抬 F 到上限、一次准入里再推一串）都在任何写之前拒、点名盘 1
/// （攻方：改之前抬 F 被读根环那一判侧面拒，准入抬 F 与再推一串没试）。两块盘逐字节不变。
#[test]
fn raising_the_floor_with_device_one_replaced_by_a_blank_disk_is_refused_before_any_write_on_every_raise_entry(
) {
    let (pool, image) = pool_after_two_overwrites();
    let session = pool.session.as_ref().expect("会话开着");
    for entry in [
        FloorRaiseEntry::DirectlyToAGivenFloor,
        FloorRaiseEntry::ToTheAdmissionCeiling,
        FloorRaiseEntry::OnePushWithinTheAdmissionBudget,
    ] {
        let entry_name = entry.name();
        let stream = SharedStream::new();
        let mut devices =
            recorded_devices_with_device_one_replaced(&image, blank_device(&image), &stream);
        let images_before = images_of(&devices);
        let mut allocator = session.allocator.clone();
        let mut current = current_file_version(session);
        let (disagreeing_fields, disagreeing_device_table) = match entry {
            FloorRaiseEntry::DirectlyToAGivenFloor => device_table_refusal_of(
                raise_rollback_floor(
                    &pool.parameters,
                    &mut devices,
                    &mut allocator,
                    &mut current,
                    CheckpointTxg(3),
                    ShadowLedger::On,
                ),
                entry_name,
            ),
            FloorRaiseEntry::ToTheAdmissionCeiling => device_table_refusal_of(
                raise_rollback_floor_to_the_admission_ceiling(
                    &pool.parameters,
                    &mut devices,
                    &mut allocator,
                    &mut current,
                    ShadowLedger::On,
                ),
                entry_name,
            ),
            FloorRaiseEntry::OnePushWithinTheAdmissionBudget => device_table_refusal_of(
                push_one_floor_raise_within_the_admission_budget(
                    &pool.parameters,
                    &mut devices,
                    &mut allocator,
                    &mut current,
                    ShadowLedger::On,
                    0,
                ),
                entry_name,
            ),
        };
        assert_only_device_one_is_named_not_visible(
            &disagreeing_fields,
            &disagreeing_device_table,
            entry_name,
        );
        assert_nothing_written(&stream, &devices, &images_before, entry_name);
        assert_eq!(
            current.root,
            current_file_version(session).root,
            "{entry_name}：现行那一版不动"
        );
    }
}

/// 盘 1 换成空盘，会话发布在任何写之前拒、点名盘 1（攻方 `the_session_publishes_with_device_one_replaced_by_a_blank_disk`：
/// 改之前会话只比身份、照样发布、空盘收到写）。两块盘逐字节不变（空盘仍一个扇区都没有），会话的现行版本不动；换回真的盘 1 照常发布。
#[test]
fn the_session_refuses_to_publish_with_device_one_replaced_by_a_blank_disk_before_any_write() {
    let (mut pool, image) = pool_after_two_overwrites();
    let write_time_seconds = pool.write_time_seconds + 1;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let change = UserChange::Overwrite(FirstFile {
        content: &content,
        write_time_seconds,
    });
    let session = pool.session.as_mut().expect("会话开着");
    let root_before = *session.current.root();
    let mut devices = plain_devices_with_device_one_replaced(&image, blank_device(&image));
    let images_before = images_of(&devices);
    let outcome = session.publish_user_change(&mut devices, change);
    let Err(UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration {
        devices: devices_not_visible,
    }) = outcome
    else {
        panic!("盘 1 换成空盘，会话要拒成 DevicesWithoutASelfVerifiedSystemConfiguration，实际 {outcome:?}")
    };
    assert_eq!(devices_not_visible, vec![SUBSTITUTED_DEVICE], "点名盘 1");
    assert_eq!(
        images_of(&devices),
        images_before,
        "拒在任何写之前：两块盘逐字节不变"
    );
    assert_eq!(
        devices[1].1.image,
        SparseDevice::default(),
        "空盘仍一个扇区都没写过"
    );
    assert_eq!(*session.current.root(), root_before, "会话的现行版本不动");
    let mut real_devices =
        plain_devices_with_device_one_replaced(&image, copy_of_device(&image, SUBSTITUTED_DEVICE));
    session
        .publish_user_change(&mut real_devices, change)
        .expect("换回真的盘 1，会话照常发布");
    assert_eq!(
        session.current.root().checkpoint_txg,
        CheckpointTxg(root_before.checkpoint_txg.0 + 1),
        "照常发布：接在拒之前那一版后面"
    );
}

/// 盘 1 换成别的池的盘 1（两槽都自证得过，fsid 不是本池的）：会话不择系统配置，这一判照样拒、点名盘 1，一个写都不发。
#[test]
fn the_session_refuses_to_publish_with_device_one_replaced_by_device_one_of_another_pool() {
    let (mut pool, image) = pool_after_two_overwrites();
    let write_time_seconds = pool.write_time_seconds + 1;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let session = pool.session.as_mut().expect("会话开着");
    let root_before = *session.current.root();
    let mut devices = plain_devices_with_device_one_replaced(&image, device_one_of_another_pool());
    let images_before = images_of(&devices);
    let outcome = session.publish_user_change(
        &mut devices,
        UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds,
        }),
    );
    let Err(UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration {
        devices: devices_not_visible,
    }) = outcome
    else {
        panic!("盘 1 是别的池的，会话要拒成 DevicesWithoutASelfVerifiedSystemConfiguration，实际 {outcome:?}")
    };
    assert_eq!(devices_not_visible, vec![SUBSTITUTED_DEVICE], "点名盘 1");
    assert_eq!(
        images_of(&devices),
        images_before,
        "拒在任何写之前：两块盘逐字节不变（别的池那块盘也没被写）"
    );
    assert_eq!(*session.current.root(), root_before, "会话的现行版本不动");
}

/// 对照（攻方 `unmount_with_device_one_replaced_by_device_one_of_another_pool`）：别的池的盘 1 换进来，正常卸载照旧被择系统配置那一步拒
/// （两块盘上各有一份自证过、fsid 不同的系统配置，`RecoveryFailure::SystemConfigurationsDisagree`），不是被「可见」那一判拒——
/// 这一判在择系统配置之后。一个写都不发，两块盘逐字节不变。
#[test]
fn unmount_with_device_one_replaced_by_device_one_of_another_pool_is_still_refused_when_choosing_the_system_configuration(
) {
    let (mut pool, image) = pool_after_two_overwrites();
    let session = pool.session.as_mut().expect("会话开着");
    let stream = SharedStream::new();
    let mut devices =
        recorded_devices_with_device_one_replaced(&image, device_one_of_another_pool(), &stream);
    let images_before = images_of(&devices);
    let outcome = unmount(
        &pool.parameters,
        &mut devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match outcome {
        Err(MountError::Recovery(RecoveryFailure::SystemConfigurationsDisagree)) => {}
        Err(other) => panic!(
            "别的池的盘：要被择系统配置那一步拒（SystemConfigurationsDisagree），实际 {other:?}"
        ),
        Ok(_) => panic!("别的池的盘：要在任何写之前拒，实际卸载做成了"),
    }
    assert_nothing_written(&stream, &devices, &images_before, "别的池的盘换进来卸载");
}

/// 「至少一份」的边界：盘 1 的系统配置槽 0 改坏一个字节（整槽校验和罩整槽，自证不过），槽 1 仍自证得过——这块盘仍「可见」，
/// 会话照常发布、之后同一份盘表照常卸载。判成「两槽都要自证过」就把只坏了一槽的盘拒掉（单份坏由 D22（单元原子性怎么合成） 已定项 16 的
/// 择槽规则扛住，不是空盘）。
#[test]
fn a_device_with_one_of_its_two_system_configuration_slots_corrupted_is_still_visible_to_the_session_and_to_unmount(
) {
    let (mut pool, image) = pool_after_two_overwrites();
    let write_time_seconds = pool.write_time_seconds + 1;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let mut device_one = copy_of_device(&image, SUBSTITUTED_DEVICE);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut slot_zero = device_one.image.read(DeviceOffsetInBytes(0), slot_bytes);
    slot_zero[slot_bytes / 2] ^= 0xff;
    device_one
        .write_at(DeviceOffsetInBytes(0), &slot_zero, WriteDurability::Plain)
        .expect("改坏盘 1 的系统配置槽 0");
    let session = pool.session.as_mut().expect("会话开着");
    let root_before = *session.current.root();
    let mut devices = plain_devices_with_device_one_replaced(&image, device_one);
    session
        .publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds,
            }),
        )
        .expect("盘 1 只坏了一槽，另一槽自证得过：仍可见，会话照常发布");
    assert_eq!(
        session.current.root().checkpoint_txg,
        CheckpointTxg(root_before.checkpoint_txg.0 + 1),
        "照常发布：接在之前那一版后面"
    );
    assert_unmount_raised_the_floor(
        unmount(
            &pool.parameters,
            &mut devices,
            &mut session.allocator,
            &mut session.current,
            ShadowLedger::On,
        ),
        "盘 1 只坏了一槽（发布末尾的轮换写的是另一槽），仍可见",
    );
}

/// 会话这一判的代价：拒之前每块盘只读两次（两个系统配置槽各一次）、一个写一道屏障都不发。读次数由块层数（故障注入包装、计划不上膛，
/// `singlefs_harness::fault_injection::SharedFaultPlan::counts_of_device`），不看实现自己报的。
#[test]
fn the_session_refusal_reads_the_two_system_configuration_slots_of_each_device_and_writes_nothing()
{
    let (mut pool, image) = pool_after_two_overwrites();
    let write_time_seconds = pool.write_time_seconds + 1;
    let content = content_of(OVERWRITE_BYTES, write_time_seconds);
    let plan = SharedFaultPlan::unarmed(DEVICE_WIDTH.fixed_geometry());
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        plain_devices_with_device_one_replaced(&image, blank_device(&image))
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
                )
            })
            .collect();
    let session = pool.session.as_mut().expect("会话开着");
    let outcome = session.publish_user_change(
        &mut devices,
        UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds,
        }),
    );
    assert!(
        matches!(
            &outcome,
            Err(UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. })
        ),
        "盘 1 换成空盘，会话要拒，实际 {outcome:?}"
    );
    for device in [DeviceIdentity(0), SUBSTITUTED_DEVICE] {
        let counts = plan.counts_of_device(device);
        assert_eq!(
            (
                counts.reads,
                counts.writes,
                counts.barriers_forwarded,
                counts.barriers_swallowed
            ),
            (2, 0, 0, 0),
            "盘 {}：拒之前只读两个系统配置槽，不写、不发屏障",
            device.0
        );
    }
}
