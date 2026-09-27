//! 实审 A1b Q2、Q3（主 agent 2026-09-27 定）：代码审阅第 17、18 条只守住了可写挂载那一处。挂着之后的入口——会话的发布
//! （`MountedSession::publish_user_change`）、抬 F（`raise_rollback_floor`、`raise_rollback_floor_to_the_admission_ceiling`、
//! `push_one_floor_raise_within_the_admission_budget`）与正常卸载（`unmount`）——每次拿调用方交进来的参数与设备表建写入口：
//! 挂载之后换一份参数，取号之后每次轮换照它改写盘上系统不可变配置（D22（单元原子性怎么合成） 已定项 26 第一档）；盘表里同一个身份交两次，
//! 多出来的那块盘收到本池的系统配置写与单元写。
//! 改成：会话用挂载交回的那一份（盘上择到的系统配置里的参数、挂载时核过的盘表），不再收调用方的参数，交进来的盘表与挂载时的逐项比、
//! 对不上在任何读写之前拒；调用方交参数与盘表的那几个自由函数照可写挂载同一套比对与重复身份检查，在任何写之前拒。
//! 管理员回退（`roll_back_by_a_forward_publish`）不在这一份里：它的拒绝要 `RollbackError` 加成员，卡在别的会话正改着的文件上（实审 A1b 报告）。

mod common;
mod common_admission;

use common::{
    build_pool, crash_state_devices, parameters, publish_overwrite_in_process, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    push_one_floor_raise_within_the_admission_budget, raise_rollback_floor,
    raise_rollback_floor_to_the_admission_ceiling, unmount, DeviceCount, DeviceTableDisagreement,
    MakeFilesystemParameterField, MountError, ParametersAndDeviceTableOfTheMount,
    RaiseToTheAdmissionCeiling, RepeatedDeviceIdentity, ShadowLedger,
    PUBLISHES_PER_ADMISSION_AT_MOST,
};
use singlefs_core::mounted_session::{UserChange, UserChangeRefused};
use singlefs_core::transaction::{FirstFile, PoolVersion, TransactionOutput};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type RecordedSparse = RecordingBlockDevice<SparseBlockDevice>;

/// 第一个事务（txg 3）之后在同一个进程里覆盖写三次（txg 4、5、6）：抬 F 的上限是 3。
fn pool_after_three_overwrites(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    for seed in 0..3_u8 {
        let previous = pool.output.clone();
        let content: Vec<u8> = (0..2000_usize)
            .map(|index| u8::try_from((index * 13 + usize::from(seed)) % 241).expect("小于 256"))
            .collect();
        pool.output = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content,
            FIXED_WRITE_TIME_SECONDS + 60,
            InstanceGeneration(1),
        )
        .expect("覆盖写");
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(6));
    pool
}

/// 这一刻两块盘的内存拷贝（写录进 `stream`），与这个进程的分配器、现行版本各一份拷贝：入口在拷贝上跑，原池不动。
fn copies_of(
    pool: &BuiltPool,
    stream: &SharedStream,
) -> (
    Vec<(DeviceIdentity, RecordedSparse)>,
    PoolAllocator,
    TransactionOutput,
) {
    (
        crash_state_devices(&pool.memory_pool(), &[], &[], stream),
        pool.allocator.clone(),
        pool.output.clone(),
    )
}

/// `image` 里 `source` 那块盘的一份拷贝，标成 `identity`；`source` 为 `None` 时是一块空盘。
fn extra_device(
    image: &MemoryPool,
    source: Option<DeviceIdentity>,
    identity: DeviceIdentity,
    stream: &SharedStream,
) -> (DeviceIdentity, RecordedSparse) {
    let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
    if let Some(source) = source {
        device.image = image.devices.get(&source).cloned().expect("镜像里有这块盘");
    }
    (
        identity,
        RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
    )
}

fn images_of(
    devices: &[(DeviceIdentity, RecordedSparse)],
) -> Vec<singlefs_harness::crash::SparseDevice> {
    devices
        .iter()
        .map(|(_, device)| device.inner().image.clone())
        .collect()
}

/// 录制流一步都没有、每块盘逐字节不变：拒在任何写之前。
fn assert_nothing_written(
    devices: &[(DeviceIdentity, RecordedSparse)],
    images_before: &[singlefs_harness::crash::SparseDevice],
    stream: &SharedStream,
) {
    assert_eq!(
        stream.operation_count(),
        0,
        "拒在任何写之前：录制流一步都没有（没发写、没发屏障）"
    );
    assert_eq!(
        images_of(devices),
        images_before,
        "拒在任何写之前：盘表里每块盘逐字节不变"
    );
}

/// 拒成 `CallerParametersDisagreeWithTheSelectedSystemConfiguration` 时交回参数与盘表不一致的那两张清单。
fn disagreements_of(
    refusal: MountError,
) -> (
    Vec<MakeFilesystemParameterField>,
    Vec<DeviceTableDisagreement>,
) {
    let MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
        disagreeing_fields,
        disagreeing_device_table,
    } = refusal
    else {
        panic!("该报 CallerParametersDisagreeWithTheSelectedSystemConfiguration，实际 {refusal:?}")
    };
    (disagreeing_fields, disagreeing_device_table)
}

fn parameters_with_minimum_input_output_bytes(
    minimum_input_output_bytes: u32,
) -> MakeFilesystemParameters {
    let mut changed = parameters();
    changed.geometry.minimum_input_output_bytes = minimum_input_output_bytes;
    changed
}

/// 挂着的会话（可写挂载做成之后接过来的）发布一次覆盖写，交进来的盘表多一块标成盘 0 的空盘。
/// 改之前会话照这份盘表开写入口，三块盘都收到这次发布的单元写与系统配置轮换写。现在拒，三块盘逐字节不变。
#[test]
fn the_session_refuses_a_device_table_other_than_the_one_of_its_mount_before_any_write() {
    let mut pool = common_admission::start_plain(HistoryDeviceWidth::FourGibibytes);
    pool.crash_and_mount_writable()
        .expect("第一个文件之后崩了再挂做成");
    let mut devices = std::mem::take(&mut pool.devices);
    devices.push((
        DeviceIdentity(0),
        SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512)),
    ));
    let images_before: Vec<_> = devices
        .iter()
        .map(|(_, device)| device.image.clone())
        .collect();
    let content = common_admission::content_of(common_admission::OVERWRITE_BYTES, 7);
    let session = pool.session.as_mut().expect("挂载做成之后会话开着");
    let root_before = *session.current.root();
    let outcome = session.publish_user_change(
        &mut devices,
        UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 3600,
        }),
    );
    let images_after: Vec<_> = devices
        .iter()
        .map(|(_, device)| device.image.clone())
        .collect();
    let Err(UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount {
        device_table_of_the_mount,
        device_table_handed_in,
    }) = outcome
    else {
        panic!("盘表与挂载时的不同（多一块盘 0），会话必须拒成 DeviceTableOtherThanTheOneOfTheMount，实际 {outcome:?}")
    };
    assert_eq!(
        (device_table_of_the_mount, device_table_handed_in),
        (
            vec![DeviceIdentity(0), DeviceIdentity(1)],
            vec![DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)]
        ),
        "报出挂载时核过的盘表与这次交进来的盘表"
    );
    assert_eq!(
        images_after, images_before,
        "拒在任何写之前：三块盘逐字节不变"
    );
    assert_eq!(*session.current.root(), root_before, "会话的现行版本不动");
}

/// 没经过可写挂载的会话（mkfs 同一个进程里接着发布的那一条）认下的池从盘上读（`ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`）：
/// 交建池的两块盘，读出的参数就是 mkfs 用的那一份、盘表是 [盘 0, 盘 1]；把盘 1 换成盘 0 的拷贝，照可写挂载同一套核拒成
/// `CallerParametersDisagreeWithTheSelectedSystemConfiguration`（盘 1 的本盘设备号是 0），一个写都不发。
#[test]
fn the_pool_of_a_session_without_a_mount_is_read_from_the_disk_and_a_disagreeing_device_table_is_refused(
) {
    let pool = build_pool("a1b-pool-of-the-make-filesystem-session");
    let stream = SharedStream::new();
    let (devices, _, _) = copies_of(&pool, &stream);
    let read = ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices)
        .unwrap_or_else(|error| panic!("建池的两块盘：读得出、与盘表相符，实际 {error:?}"));
    assert_eq!(
        (
            read.parameters_on_disk(),
            read.device_identities_in_table_order()
        ),
        (&parameters(), &[DeviceIdentity(0), DeviceIdentity(1)][..]),
        "读出的参数是 mkfs 用的那一份，盘表是交进来的两块盘"
    );

    let image = pool.memory_pool();
    let mut devices_with_a_copy_of_zero = copies_of(&pool, &stream).0;
    devices_with_a_copy_of_zero.retain(|(identity, _)| *identity == DeviceIdentity(0));
    devices_with_a_copy_of_zero.push(extra_device(
        &image,
        Some(DeviceIdentity(0)),
        DeviceIdentity(1),
        &stream,
    ));
    let images_before = images_of(&devices_with_a_copy_of_zero);
    let refusal =
        ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices_with_a_copy_of_zero)
            .expect_err("盘 1 是盘 0 的拷贝，读盘上的池必须拒");
    assert_eq!(
        disagreements_of(refusal),
        (
            Vec::new(),
            vec![
                DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                    identity_handed_in: DeviceIdentity(1),
                    own_device_number_on_disk: DeviceIdentity(0),
                }
            ]
        ),
        "没有参数可比；盘 1 的本盘设备号是 0"
    );
    assert_nothing_written(&devices_with_a_copy_of_zero, &images_before, &stream);
}

/// 抬 F 到上限 3，调用方交的参数把 io_min 改成 4096（槽距仍 4096）：改之前抬 F 先写系统配置那一步与每次空发布的轮换把两块盘系统配置里的
/// io_min 改写成 4096。现在拒成 `CallerParametersDisagreeWithTheSelectedSystemConfiguration`，盘上逐字节不变。对照：建池的那一份参数照常抬成。
#[test]
fn raising_the_floor_with_other_parameters_is_refused_before_any_write() {
    let pool = pool_after_three_overwrites("a1b-raise-other-parameters");
    let stream = SharedStream::new();
    let (mut devices, mut allocator, mut current) = copies_of(&pool, &stream);
    let images_before = images_of(&devices);
    let refusal = raise_rollback_floor(
        &parameters_with_minimum_input_output_bytes(4096),
        &mut devices,
        &mut allocator,
        &mut current,
        CheckpointTxg(3),
        ShadowLedger::On,
    )
    .expect_err("调用方的参数与盘上系统配置不一致，抬 F 必须拒");
    assert_eq!(
        disagreements_of(refusal),
        (
            vec![MakeFilesystemParameterField::MinimumInputOutputBytes],
            Vec::new()
        ),
        "只有 io_min 不一致，盘表相符"
    );
    assert_nothing_written(&devices, &images_before, &stream);
    raise_rollback_floor(
        &parameters(),
        &mut devices,
        &mut allocator,
        &mut current,
        CheckpointTxg(3),
        ShadowLedger::On,
    )
    .unwrap_or_else(|error| panic!("建池的那一份参数照常抬到 3：{error:?}"));
}

/// 正常卸载，盘表多交一块标成盘 1 的空盘：改之前卸载那一串照这份盘表写，空盘收到系统配置写与单元写。
/// 现在拒成 `DeviceIdentitiesHandedInMoreThanOnce`，三块盘逐字节不变。
#[test]
fn unmounting_with_a_device_identity_handed_in_twice_is_refused_before_any_write() {
    let pool = pool_after_three_overwrites("a1b-unmount-device-twice");
    let stream = SharedStream::new();
    let (mut devices, mut allocator, current) = copies_of(&pool, &stream);
    devices.push(extra_device(
        &pool.memory_pool(),
        None,
        DeviceIdentity(1),
        &stream,
    ));
    let images_before = images_of(&devices);
    let mut current = PoolVersion::WithFile(current);
    let refusal = unmount(
        &parameters(),
        &mut devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    )
    .err()
    .expect("盘 1 交了两次，正常卸载必须拒");
    let MountError::DeviceIdentitiesHandedInMoreThanOnce { repeated } = refusal else {
        panic!("该报 DeviceIdentitiesHandedInMoreThanOnce，实际 {refusal:?}")
    };
    assert_eq!(
        repeated,
        vec![RepeatedDeviceIdentity {
            device: DeviceIdentity(1),
            times_handed_in: 2,
        }],
        "盘 1 交了两次，盘 0 一次"
    );
    assert_nothing_written(&devices, &images_before, &stream);
}

/// 抬到上限 3 之后 F 已在上限：准入抬 F 本来一个字节都不写、交回「F 已在上限」。盘表多交一块标成盘 2 的盘 1 的拷贝
/// （盘数 3 与 devs 2 不同）：改之前照样交回「F 已在上限」，调用方拿这份盘表接着发布就会把 devs 写成 3。现在在读根环之前就拒。
#[test]
fn raising_the_floor_to_the_admission_ceiling_with_a_third_device_is_refused_before_any_write() {
    let mut pool = pool_after_three_overwrites("a1b-ceiling-third-device");
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut current = pool.output.clone();
        raise_rollback_floor(
            &parameters(),
            devices,
            &mut pool.allocator,
            &mut current,
            CheckpointTxg(3),
            ShadowLedger::On,
        )
        .expect("抬到上限 3");
        pool.output = current;
    }
    let stream = SharedStream::new();
    let (mut devices, mut allocator, mut current) = copies_of(&pool, &stream);
    devices.push(extra_device(
        &pool.memory_pool(),
        Some(DeviceIdentity(1)),
        DeviceIdentity(2),
        &stream,
    ));
    let images_before = images_of(&devices);
    let outcome = raise_rollback_floor_to_the_admission_ceiling(
        &parameters(),
        &mut devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
    );
    let Err(refusal) = outcome else {
        panic!(
            "盘表多一块盘（盘数与 devs 不同），准入抬 F 必须拒，实际交回的是「F 已在上限」：{:?}",
            outcome.map(|done| matches!(
                done,
                RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { .. }
            ))
        )
    };
    assert_eq!(
        disagreements_of(refusal),
        (
            Vec::new(),
            vec![
                DeviceTableDisagreement::DeviceCountDiffersFromTheSystemConfiguration {
                    devices_handed_in: DeviceCount(3),
                    device_count_in_the_system_configuration: DeviceCount(2),
                },
                DeviceTableDisagreement::OwnDeviceNumberDiffersFromTheIdentityHandedIn {
                    identity_handed_in: DeviceIdentity(2),
                    own_device_number_on_disk: DeviceIdentity(1),
                },
            ]
        ),
        "参数相符；盘数 3 与 devs 2 不同、盘 2 的本盘设备号是 1"
    );
    assert_nothing_written(&devices, &images_before, &stream);
}

/// 一次准入里已经推了 7 次空发布，再推一串就超过预算：本来一个字节都不写、交回「预算用完」。调用方的参数换了 fsid：
/// 改之前照样按它的根环区域归属数这一串要推几次、交回「预算用完」。现在在判预算之前就拒。
#[test]
fn pushing_a_floor_raise_with_another_filesystem_identifier_is_refused_before_the_budget_is_judged()
{
    let pool = pool_after_three_overwrites("a1b-push-other-fsid");
    let stream = SharedStream::new();
    let (mut devices, mut allocator, mut current) = copies_of(&pool, &stream);
    let images_before = images_of(&devices);
    let mut changed = parameters();
    changed.filesystem_identifier[0] ^= 0xff;
    let outcome = push_one_floor_raise_within_the_admission_budget(
        &changed,
        &mut devices,
        &mut allocator,
        &mut current,
        ShadowLedger::On,
        PUBLISHES_PER_ADMISSION_AT_MOST - 1,
    );
    let Err(refusal) = outcome else {
        panic!("调用方的参数换了 fsid，推抬 F 必须拒，实际 {outcome:?}")
    };
    assert_eq!(
        disagreements_of(refusal),
        (
            vec![MakeFilesystemParameterField::FilesystemIdentifier],
            Vec::new()
        ),
        "只有 fsid 不一致，盘表相符"
    );
    assert_nothing_written(&devices, &images_before, &stream);
}
