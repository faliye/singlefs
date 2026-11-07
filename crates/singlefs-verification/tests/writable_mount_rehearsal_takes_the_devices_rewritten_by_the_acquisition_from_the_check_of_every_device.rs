//! 可写挂载取号之前的逐盘核排在续做预演之前，预演按逐盘核交回的补写盘表走（C874（补写那一支进不了崩溃测试、运行时看不出）；
//! 代码轮 `research/prompts/m3-close-b19-code-r1-main-verification.md` 第二节 T4，攻方探针
//! `research/prompts/m3-close-b19-code-r1-opus-model/attack_probe_rehearsal_reads_a_device_without_any_self_verified_system_configuration.rs`）。
//! 镜像是 `common_reads_shared_by_the_rehearsal_and_the_real_run` 的窄盘那一份：续做中途推抬 F，取号之前在只进内存的盘上把取号连同续做预演一遍；
//! 在它上面把盘 1 两个系统配置槽整槽写 0（两槽全废，丙' 补写那一形）。
//! 逐盘核排在预演之后、预演按空的补写盘表走时，只进内存那一趟取号读盘 1 两槽、一份自证过的都没有，那一趟停在取号；
//! 重走到上限在取号之前拒成「预演读坏」，逐盘核与补写闸一次都走不到。
//!
//! 另两形（代码轮 `research/prompts/m3-close-b19-code-r2-main-verification.md` 第二节 U1、U2，辩方探针
//! `research/prompts/m3-close-b19-code-r2-sonnet-model/tests/` 里 U1、U2 两份升格）：逐盘核那一拒与续做预演那一拒同时成立时报逐盘核那一拒；
//! 预演判两本账时不拿补写盘系统配置槽的读坏判，拒因点名真出故障的那一读。
//!
//! 改法丙（V1、V2；
//! `research/prompts/m3-close-b19-code-r3-opus-output.md` V1「能写成的会红用例」三形）：补写盘之外的成员盘一个系统配置槽在逐盘核那一刻已读坏
//! （整槽写 0）或已报读错（每次读都报块设备错），只进内存那一趟取号照样读到它、记进账；预演判两本账时剔掉落在这类槽上的那几条，
//! 拒因照旧点名真出故障的那份树表。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_reads_shared_by_the_rehearsal_and_the_real_run;

use std::cell::RefCell;
use std::rc::Rc;

use common::PoolImageGeometry;
use common_admission::plain_devices_on;
use common_reads_shared_by_the_rehearsal_and_the_real_run::{
    mount_writable_on_recording_devices, mount_writable_without_faults, narrow_image,
    narrow_image_with_the_retirement_container_at_the_top_of_its_range,
    ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy, switches,
    FaultedReads, ReadFault, ReadReachingTheDisk, BITS_FLIPPED_IN_THE_FIRST_BYTE,
    BYTES_OF_A_TREE_TABLE,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches,
    DeviceNotRewrittenByTheAcquisition, MountError, Mounted, RewriteGateNotMet, ShadowLedger,
    SystemConfigurationUnreadableWhen,
};
use singlefs_core::transaction::{PublishError, ReadFailingSelfVerification};
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_verification::fault_injection::injected_block_device_error;
use singlefs_verification::memory_pool::{MemoryPool, SparseBlockDevice};

/// 两个系统配置槽整槽写 0 的那块盘。
const DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED: DeviceIdentity = DeviceIdentity(1);

/// 这块盘两个系统配置槽的偏移：槽 0 在偏移 0，槽 1 在一个固定结构槽距处。
fn system_configuration_slot_offsets(
    geometry: &PoolImageGeometry,
) -> impl Iterator<Item = DeviceOffsetInBytes> {
    let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
    (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
        .map(move |slot_index| DeviceOffsetInBytes(slot_index * spacing))
}

fn with_both_system_configuration_slots_zeroed(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    device: DeviceIdentity,
) -> MemoryPool {
    let mut zeroed = image.clone();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096 装得进 usize");
    let device_image = zeroed.devices.get_mut(&device).expect("池里有这块盘");
    for offset in system_configuration_slot_offsets(geometry) {
        device_image.write(offset, &vec![0u8; slot_bytes]);
    }
    zeroed
}

/// 这块盘从头到尾整块写 0：两槽全废，单元区里也没有所选那一版的单元（补写闸一过不了）。
fn with_the_whole_device_zeroed(image: &MemoryPool, device: DeviceIdentity) -> MemoryPool {
    let mut zeroed = image.clone();
    let device_size_in_bytes = zeroed.device_size_in_bytes;
    zeroed
        .devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .zero_fill(DeviceOffsetInBytes(0), device_size_in_bytes);
    zeroed
}

/// 场景：续做最后一批要开的容器号到顶（C841）的窄盘镜像上，把盘 1 整块写 0，可写挂载。逐盘核那一拒（盘 1 两槽全废、补写闸一没过）
/// 与续做预演那一拒（没改坏的这份镜像上是 C841）同时成立。
/// 预期：报逐盘核那一拒——`MountError::SystemConfigurationUnreadableAfterTheReadStage`，载荷点名盘 1、闸一没过；取号之前拒，一个写都没发到盘上。
/// 判别力：逐盘核放回续做预演之后（`crates/mutations.tsv`「C874 逐盘核放回续做预演之后」那一行：预演先按空的补写盘表走一趟）时，
/// 那一趟取号读盘 1 两槽一份自证过的都没有、停在取号，重走到上限在取号之前交预演读坏，这一条红在拒因那一条断言。
#[test]
fn whole_zeroed_device_on_an_image_whose_continuation_opens_a_container_past_the_top_of_its_range_is_refused_by_the_check_of_every_device(
) {
    let (image, geometry) = narrow_image_with_the_retirement_container_at_the_top_of_its_range();
    let unchanged = mount_writable_without_faults(&image, &geometry);
    assert!(
        matches!(
            &unchanged.result,
            Err(MountError::Publish(failed))
                if matches!(failed.cause, PublishError::NextContainerNumberPastTheTopOfItsRange(_))
        ),
        "前提：没改坏的这份镜像上续做预演那一拒成立（C841），实际 {:?}",
        unchanged.result.as_ref().err()
    );

    let zeroed =
        with_the_whole_device_zeroed(&image, DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED);
    let mounted = mount_writable_without_faults(&zeroed, &geometry);
    let Err(MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)) =
        &mounted.result
    else {
        panic!(
            "盘 1 整块写 0、续做预演报 C841：逐盘核排在预演之前，该报逐盘核那一拒，实际 {:?}",
            mounted.result.as_ref().err()
        );
    };
    assert_eq!(
        unreadable.unreadable,
        SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
            device: DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED,
            not_rewritten_because: DeviceNotRewrittenByTheAcquisition::RewriteGateNotMet(
                RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice
            ),
        },
        "拒因载荷点名整块写 0 的盘 1、补写闸一没过"
    );
    assert!(
        mounted.writes_reaching_the_disk.is_empty(),
        "取号之前拒，一个写都没发到盘上：{} 个",
        mounted.writes_reaching_the_disk.len()
    );
}

/// 场景：窄盘镜像上盘 1 两个系统配置槽整槽写 0（过补写闸、取号补写它；续做中途推抬 F，取号之前那一趟预演走只进内存的盘）；
/// 只进内存那一趟头一次读的那份树表，从那一次读起两块盘上那两份每一次读都读回坏字节（重走那一趟照样读坏）。
/// 预演停在续做的结局之前，重走到上限在取号之前拒成预演读坏。
/// 预期：拒因交的那一次读坏是那份树表（偏移、长度都是它的），不是补写盘的系统配置槽——只进内存那一趟取号读盘 1 两槽必然读坏，
/// 预演判两本账时剔掉这两条；取号之前拒，一个写都没发到盘上。
/// 判别力：判两本账时不剔补写盘系统配置槽的读坏（`crates/mutations.tsv`「C874 预演判两本账不剔补写盘系统配置槽的读坏」那一行）时，
/// 读坏账最先那一条是盘 1 槽 0（偏移 0、长 4096），这一条红在拒因那一条断言。
/// 预演按空的补写盘表走（`crates/mutations.tsv`「C874 续做预演按空的补写盘表走（逐盘核交回的那一份不交给预演）」那一行）时，
/// 只进内存那一趟停在取号，两本账剔掉逐盘核那一刻已坏的盘 1 两槽之后是空的，预演不拒、留给真做，挂载做成，
/// 这一条红在「预演重走到上限、在取号之前拒」那一处。
#[test]
fn tree_table_reading_back_garbage_every_time_is_named_by_the_rehearsal_refusal_instead_of_the_slots_of_a_device_rewritten_by_the_acquisition(
) {
    let (image, geometry, _) = narrow_image();
    let zeroed = with_both_system_configuration_slots_zeroed(
        &image,
        &geometry,
        DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED,
    );
    let fault_free = mount_writable_without_faults(&zeroed, &geometry);
    assert!(
        fault_free.result.is_ok(),
        "前提：盘 1 两槽整槽写 0、不带读故障的可写挂载做成（逐盘核把盘 1 交给取号补写），实际 {:?}",
        fault_free.result.as_ref().err()
    );
    let (window, offset_of_the_tree_table_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );
    let faulted = mount_writable_on_recording_devices(
        &zeroed,
        &geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::EveryReadOfTheRangeFromTheOrdinal {
            from_ordinal: *window
                .first()
                .expect("窗口是只进内存那一趟头一次读的树表两份那两次读"),
            offset: offset_of_the_tree_table_copies,
            length_in_bytes: BYTES_OF_A_TREE_TABLE,
        },
    );
    let Err(MountError::Publish(failed)) = &faulted.result else {
        panic!(
            "树表每一次读都读坏：预演重走到上限、在取号之前拒，实际 {:?}",
            faulted.result.as_ref().err()
        );
    };
    let PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(
        read_named_by_the_refusal,
    ) = &failed.cause
    else {
        panic!("拒因该是预演读坏，实际 {:?}", failed.cause);
    };
    assert_eq!(
        (
            read_named_by_the_refusal.offset,
            read_named_by_the_refusal.length_in_bytes
        ),
        (offset_of_the_tree_table_copies, BYTES_OF_A_TREE_TABLE),
        "拒因点名出故障的那份树表，不点名补写盘的系统配置槽：交的是盘 {:?} 偏移 {:?} 长 {}",
        read_named_by_the_refusal.device,
        read_named_by_the_refusal.offset,
        read_named_by_the_refusal.length_in_bytes
    );
    assert!(
        faulted.writes_reaching_the_disk.is_empty(),
        "取号之前拒，一个写都没发到盘上：{} 个",
        faulted.writes_reaching_the_disk.len()
    );
}

/// 场景：窄盘镜像上盘 1 两个系统配置槽整槽写 0（读得出、自证不过），可写挂载；这份镜像的续做中途推抬 F，取号之前那一趟预演因此在只进内存的盘上
/// 把取号连同续做走一遍。
/// 预期：取号之前的逐盘核先判出盘 1 过了补写闸、交进补写盘表；预演按这一份走，只进内存那一趟取号认盘 1 是要补写的盘、走到续做的结局；
/// 挂载做成，`devices_rewritten_by_the_acquisition` 恰是盘 1，盘 1 两槽挂完之后不再是整槽 0。
/// 预演按空的补写盘表走（`crates/mutations.tsv`「C874 续做预演按空的补写盘表走（逐盘核交回的那一份不交给预演）」）这一形上分不出：
/// 只进内存那一趟取号读盘 1 两槽一份自证过的都没有、停在取号，两本账里只有逐盘核那一刻已读坏的盘 1 两槽，
/// 改法丙判两本账时剔掉，剔完是空的、留给真做，真做照逐盘核那一份补写、挂载照样做成。那一行点名会红的
/// `tree_table_reading_back_garbage_every_time_is_named_by_the_rehearsal_refusal_instead_of_the_slots_of_a_device_rewritten_by_the_acquisition`。
#[test]
fn device_with_both_system_configuration_slots_zeroed_is_rewritten_by_the_acquisition_when_the_continuation_rehearses_on_devices_keeping_writes_in_memory(
) {
    let (image, geometry, _) = narrow_image();
    let unchanged = mount_writable_without_faults(&image, &geometry);
    let unchanged_mounted = unchanged
        .result
        .as_ref()
        .expect("前提：没改坏的窄盘镜像可写挂载做成");
    assert!(
        !unchanged_mounted
            .output
            .floor_raises_pushed_by_the_continuation
            .is_empty(),
        "前提：这份镜像的续做中途推抬 F（取号之前那一趟预演走只进内存的盘）"
    );
    assert!(
        unchanged_mounted
            .output
            .devices_rewritten_by_the_acquisition
            .is_empty(),
        "前提：没改坏的那一份没有要补写的盘"
    );

    let zeroed = with_both_system_configuration_slots_zeroed(
        &image,
        &geometry,
        DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED,
    );
    let mounted_on_the_zeroed_image = mount_writable_without_faults(&zeroed, &geometry);
    let mounted = mounted_on_the_zeroed_image
        .result
        .as_ref()
        .unwrap_or_else(|error| {
            panic!(
                "盘 1 两槽整槽写 0、续做中途推抬 F：逐盘核先把盘 1 交进补写盘表、预演照它走，挂载做成；挂载报的是 {error:?}"
            )
        });
    assert_eq!(
        mounted.output.devices_rewritten_by_the_acquisition,
        vec![DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED],
        "取号写两轮补写的恰是两槽整槽写 0 的盘 1"
    );
    let device_after_the_mount = mounted_on_the_zeroed_image
        .images_after
        .get(&DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED)
        .expect("挂完之后池里有盘 1");
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096 装得进 usize");
    for offset in system_configuration_slot_offsets(&geometry) {
        assert!(
            device_after_the_mount
                .read(offset, slot_bytes)
                .iter()
                .any(|byte| *byte != 0),
            "盘 1 偏移 {offset:?} 那一槽挂完之后被取号补写过，不再是整槽 0"
        );
    }
}

/// 补写盘之外的那块成员盘（窄盘镜像上盘 1 是补写那几形的补写盘，盘 0 两槽自证过）。
const DEVICE_NOT_REWRITTEN_BY_THE_ACQUISITION: DeviceIdentity = DeviceIdentity(0);
/// 成员盘上整槽写 0 的那个系统配置槽（槽 0，偏移 0）。
const INDEX_OF_THE_ZEROED_SLOT_OF_THE_MEMBER_DEVICE: u64 = 0;
/// 成员盘上每次读都报块设备错的那个系统配置槽（槽 1，偏移一个固定结构槽距）。
const INDEX_OF_THE_SLOT_REPORTING_A_READ_ERROR_ON_THE_MEMBER_DEVICE: u64 = 1;

fn system_configuration_slot_offset(
    geometry: &PoolImageGeometry,
    slot_index: u64,
) -> DeviceOffsetInBytes {
    DeviceOffsetInBytes(
        slot_index * u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing),
    )
}

/// `device` 上第 `slot_index` 个系统配置槽整槽写 0（读得出、自证不过；另一槽照旧自证得过）。
fn with_one_system_configuration_slot_zeroed(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    device: DeviceIdentity,
    slot_index: u64,
) -> MemoryPool {
    let mut zeroed = image.clone();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096 装得进 usize");
    zeroed
        .devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .write(
            system_configuration_slot_offset(geometry, slot_index),
            &vec![0u8; slot_bytes],
        );
    zeroed
}

/// 在 `image` 上可写挂载两遍：头一遍不带读故障，从它读到盘上的那几次读里取只进内存那一趟头一次读的那份树表；
/// 第二遍让那份树表从那一次读起两块盘上两份每一次读都读回坏字节（U2 用例那一形）。交回头一遍、第二遍与那份树表的偏移。
fn mount_without_faults_then_with_the_tree_table_reading_back_garbage_every_time(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
) -> (
    common_reads_shared_by_the_rehearsal_and_the_real_run::WritableMountOnRecordingDevices,
    common_reads_shared_by_the_rehearsal_and_the_real_run::WritableMountOnRecordingDevices,
    DeviceOffsetInBytes,
) {
    let fault_free = mount_writable_without_faults(image, geometry);
    let (window, offset_of_the_tree_table_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );
    let faulted = mount_writable_on_recording_devices(
        image,
        geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::EveryReadOfTheRangeFromTheOrdinal {
            from_ordinal: *window
                .first()
                .expect("窗口是只进内存那一趟头一次读的树表两份那两次读"),
            offset: offset_of_the_tree_table_copies,
            length_in_bytes: BYTES_OF_A_TREE_TABLE,
        },
    );
    (fault_free, faulted, offset_of_the_tree_table_copies)
}

/// 取号之前预演读坏那一拒点名的那一次读；挂载报的不是这一拒就带着实际的拒因 panic。
fn read_named_by_the_refusal_on_a_read_failing_self_verification(
    result: &Result<Mounted, MountError>,
) -> &ReadFailingSelfVerification {
    let Err(MountError::Publish(failed)) = result else {
        panic!(
            "树表每一次读都读坏：预演重走到上限、在取号之前拒，实际 {:?}",
            result.as_ref().err()
        );
    };
    let PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(
        read_named_by_the_refusal,
    ) = &failed.cause
    else {
        panic!("拒因该是预演读坏（点名那份树表），实际 {:?}", failed.cause);
    };
    read_named_by_the_refusal
}

/// 场景：窄盘镜像上盘 1 两个系统配置槽整槽写 0（补写盘），另把盘 0 槽 0 整槽写 0（盘 0 槽 1 自证得过，逐盘核放行、不补写盘 0）；
/// 树表每一次读都读坏（同 U2 用例）。
/// 预期：逐盘核那一刻盘 0 槽 0 已读坏，只进内存那一趟取号再读它记进读坏账，预演判两本账时剔掉；拒因交的那一次读坏是那份树表，
/// 不是盘 0 槽 0；取号之前拒，一个写都没发到盘上。
/// 判别力：只剔补写盘的系统配置槽、不剔成员盘的（`crates/mutations.tsv`「改法丙 预演判两本账只剔补写盘的系统配置槽、成员盘逐盘核那一刻已坏的槽照旧进读坏账」那一行，
/// 的判法）时，读坏账最先那一条是盘 0 槽 0（偏移 0、长 4096），这一条红在拒因那一条断言。
#[test]
fn tree_table_reading_back_garbage_every_time_is_named_by_the_rehearsal_refusal_instead_of_a_zeroed_slot_of_the_member_device_beside_a_device_rewritten_by_the_acquisition(
) {
    let (image, geometry, _) = narrow_image();
    let rewritten = with_both_system_configuration_slots_zeroed(
        &image,
        &geometry,
        DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED,
    );
    let torn = with_one_system_configuration_slot_zeroed(
        &rewritten,
        &geometry,
        DEVICE_NOT_REWRITTEN_BY_THE_ACQUISITION,
        INDEX_OF_THE_ZEROED_SLOT_OF_THE_MEMBER_DEVICE,
    );
    let (fault_free, faulted, offset_of_the_tree_table_copies) =
        mount_without_faults_then_with_the_tree_table_reading_back_garbage_every_time(
            &torn, &geometry,
        );
    let fault_free_mounted = fault_free.result.as_ref().unwrap_or_else(|error| {
        panic!("前提：盘 1 两槽写 0、盘 0 槽 0 写 0、不带读故障的可写挂载做成，实际 {error:?}")
    });
    assert_eq!(
        fault_free_mounted
            .output
            .devices_rewritten_by_the_acquisition,
        vec![DEVICE_WITH_BOTH_SYSTEM_CONFIGURATION_SLOTS_ZEROED],
        "前提：逐盘核只把盘 1 交给取号补写，盘 0 槽 1 自证得过、走落后支放行"
    );
    let read_named_by_the_refusal =
        read_named_by_the_refusal_on_a_read_failing_self_verification(&faulted.result);
    assert_eq!(
        (
            read_named_by_the_refusal.offset,
            read_named_by_the_refusal.length_in_bytes
        ),
        (offset_of_the_tree_table_copies, BYTES_OF_A_TREE_TABLE),
        "拒因点名出故障的那份树表，不点名成员盘逐盘核那一刻已坏的系统配置槽：交的是盘 {:?} 偏移 {:?} 长 {}",
        read_named_by_the_refusal.device,
        read_named_by_the_refusal.offset,
        read_named_by_the_refusal.length_in_bytes
    );
    assert!(
        faulted.writes_reaching_the_disk.is_empty(),
        "取号之前拒，一个写都没发到盘上：{} 个",
        faulted.writes_reaching_the_disk.len()
    );
}

/// 场景：窄盘镜像上只把盘 0 槽 0 整槽写 0（盘 0 槽 1 与盘 1 两槽自证得过，没有补写盘）；树表每一次读都读坏（同 U2 用例）。
/// 预期：逐盘核那一刻盘 0 槽 0 已读坏，预演判两本账时剔掉只进内存那一趟取号读它记下的那一条；拒因交的那一次读坏是那份树表；
/// 取号之前拒，一个写都没发到盘上。
/// 判别力：只剔补写盘的系统配置槽（`crates/mutations.tsv`「改法丙 预演判两本账只剔补写盘的系统配置槽、成员盘逐盘核那一刻已坏的槽照旧进读坏账」那一行）时，
/// 补写盘表是空的、一条都不剔，读坏账最先那一条是盘 0 槽 0，这一条红在拒因那一条断言。
#[test]
fn tree_table_reading_back_garbage_every_time_is_named_by_the_rehearsal_refusal_instead_of_a_zeroed_slot_of_the_member_device_when_no_device_is_rewritten_by_the_acquisition(
) {
    let (image, geometry, _) = narrow_image();
    let torn = with_one_system_configuration_slot_zeroed(
        &image,
        &geometry,
        DEVICE_NOT_REWRITTEN_BY_THE_ACQUISITION,
        INDEX_OF_THE_ZEROED_SLOT_OF_THE_MEMBER_DEVICE,
    );
    let (fault_free, faulted, offset_of_the_tree_table_copies) =
        mount_without_faults_then_with_the_tree_table_reading_back_garbage_every_time(
            &torn, &geometry,
        );
    let fault_free_mounted = fault_free.result.as_ref().unwrap_or_else(|error| {
        panic!("前提：只有盘 0 槽 0 写 0、不带读故障的可写挂载做成，实际 {error:?}")
    });
    assert!(
        fault_free_mounted
            .output
            .devices_rewritten_by_the_acquisition
            .is_empty(),
        "前提：没有两槽全废的盘，补写盘表是空的"
    );
    let read_named_by_the_refusal =
        read_named_by_the_refusal_on_a_read_failing_self_verification(&faulted.result);
    assert_eq!(
        (
            read_named_by_the_refusal.offset,
            read_named_by_the_refusal.length_in_bytes
        ),
        (offset_of_the_tree_table_copies, BYTES_OF_A_TREE_TABLE),
        "拒因点名出故障的那份树表，不点名成员盘逐盘核那一刻已坏的系统配置槽：交的是盘 {:?} 偏移 {:?} 长 {}",
        read_named_by_the_refusal.device,
        read_named_by_the_refusal.offset,
        read_named_by_the_refusal.length_in_bytes
    );
    assert!(
        faulted.writes_reaching_the_disk.is_empty(),
        "取号之前拒，一个写都没发到盘上：{} 个",
        faulted.writes_reaching_the_disk.len()
    );
}

/// 整池第 `from_ordinal` 次读起，读 `offset` 起 `length_in_bytes` 字节的那几次（`device` 是 `None` 时两块盘都算）照 `fault` 坏。
#[derive(Clone, Copy, Debug)]
struct ReadFaultOnARange {
    device: Option<DeviceIdentity>,
    offset: DeviceOffsetInBytes,
    length_in_bytes: u64,
    from_ordinal: u64,
    fault: ReadFault,
}

impl ReadFaultOnARange {
    fn covers(&self, read: &ReadReachingTheDisk) -> bool {
        read.ordinal_across_the_pool >= self.from_ordinal
            && self.device.is_none_or(|device| device == read.device)
            && read.offset == self.offset
            && read.length_in_bytes == self.length_in_bytes
    }
}

/// 几块盘共用的：读故障的计划（几条同时罩住一次读时取排在前面的那一条）、读到盘上的每一次读、这次挂载发到盘上的写的个数。
struct ReadsAndWritesOfThePool {
    faults: Vec<ReadFaultOnARange>,
    reads_reaching_the_disk: Vec<ReadReachingTheDisk>,
    writes_reaching_the_disk: u64,
}

/// 内存稀疏盘外面包一层：记下每一次读与写的个数，照计划让罩住的那几次读报块设备错或读回坏字节。
/// common 那一份（`mount_writable_on_recording_devices`）只收一条故障；成员盘一槽每次读报错叠上树表每次读坏要两条。
struct DeviceWithReadFaultsOnRanges {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    ledger: Rc<RefCell<ReadsAndWritesOfThePool>>,
}

impl BlockDevice for DeviceWithReadFaultsOnRanges {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let mut ledger = self.ledger.borrow_mut();
        let this_read = ReadReachingTheDisk {
            ordinal_across_the_pool: u64::try_from(ledger.reads_reaching_the_disk.len())
                .expect("读的个数装得进 u64")
                + 1,
            device: self.identity,
            offset,
            length_in_bytes: u64::try_from(buffer.len()).expect("读长装得进 u64"),
            writes_of_the_mount_before_it: ledger.writes_reaching_the_disk,
        };
        ledger.reads_reaching_the_disk.push(this_read);
        let fault_of_this_read = ledger
            .faults
            .iter()
            .find(|fault| fault.covers(&this_read))
            .map(|fault| fault.fault);
        drop(ledger);
        match fault_of_this_read {
            None => self.inner.read_at(offset, buffer),
            Some(ReadFault::ReportsADeviceError) => Err(injected_block_device_error("读")),
            Some(ReadFault::ReadsBackCorruptedBytes) => {
                self.inner.read_at(offset, buffer)?;
                buffer[0] ^= BITS_FLIPPED_IN_THE_FIRST_BYTE;
                Ok(())
            }
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.ledger.borrow_mut().writes_reaching_the_disk += 1;
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.ledger.borrow_mut().writes_reaching_the_disk += 1;
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// 一次可写挂载的结局、读到盘上的每一次读、这次挂载发到盘上的写的个数。
struct WritableMountWithReadFaultsOnRanges {
    result: Result<Mounted, MountError>,
    reads_reaching_the_disk: Vec<ReadReachingTheDisk>,
    writes_reaching_the_disk: u64,
}

/// 在 `image` 的一份拷贝上可写挂载一次（准入照式子判、影子账开、删除开关同 common 那一份），照 `faults` 让罩住的那几次读坏。
fn mount_writable_with_read_faults_on_ranges(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    faults: Vec<ReadFaultOnARange>,
) -> WritableMountWithReadFaultsOnRanges {
    let ledger = Rc::new(RefCell::new(ReadsAndWritesOfThePool {
        faults,
        reads_reaching_the_disk: Vec::new(),
        writes_reaching_the_disk: 0,
    }));
    let mut devices: Vec<(DeviceIdentity, DeviceWithReadFaultsOnRanges)> = plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceWithReadFaultsOnRanges {
                    identity,
                    inner,
                    ledger: Rc::clone(&ledger),
                },
            )
        })
        .collect();
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        &geometry.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(),
    );
    let ledger = ledger.borrow();
    WritableMountWithReadFaultsOnRanges {
        result,
        reads_reaching_the_disk: ledger.reads_reaching_the_disk.clone(),
        writes_reaching_the_disk: ledger.writes_reaching_the_disk,
    }
}

/// 场景：窄盘镜像上盘 0 槽 1 从这次挂载第一次读起每一次读都报块设备错（盘 0 槽 0 与盘 1 两槽自证得过：读阶段与取号的见证判得出，
/// 逐盘核放行盘 0、不补写）；再叠上树表每一次读都读坏（同 U2 用例，窗口取自只带槽读错那一遍）。
/// 预期：逐盘核那一刻盘 0 槽 1 已报读错，只进内存那一趟取号再读它记进读错账，预演判两本账时剔掉；读错账剔完是空的，
/// 拒因是预演读坏、点名那份树表；取号之前拒，一个写都没发到盘上。
/// 判别力：读错账不剔（`crates/mutations.tsv`「改法丙 预演判两本账不剔读错账里逐盘核那一刻已报读错的系统配置槽」那一行）时，
/// 读错账最先那一条是盘 0 槽 1，拒因成了预演读错，这一条红在「拒因该是预演读坏」。
#[test]
fn tree_table_reading_back_garbage_every_time_is_named_by_the_rehearsal_refusal_instead_of_a_slot_of_the_member_device_reporting_a_read_error_on_every_read(
) {
    let (image, geometry, _) = narrow_image();
    let slot_reporting_a_read_error = ReadFaultOnARange {
        device: Some(DEVICE_NOT_REWRITTEN_BY_THE_ACQUISITION),
        offset: system_configuration_slot_offset(
            &geometry,
            INDEX_OF_THE_SLOT_REPORTING_A_READ_ERROR_ON_THE_MEMBER_DEVICE,
        ),
        length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
        from_ordinal: 1,
        fault: ReadFault::ReportsADeviceError,
    };
    let with_the_slot_reporting_a_read_error = mount_writable_with_read_faults_on_ranges(
        &image,
        &geometry,
        vec![slot_reporting_a_read_error],
    );
    let mounted_with_the_slot_reporting_a_read_error = with_the_slot_reporting_a_read_error
        .result
        .as_ref()
        .unwrap_or_else(|error| {
            panic!("前提：只有盘 0 槽 1 每次读报错的可写挂载做成，实际 {error:?}")
        });
    assert!(
        mounted_with_the_slot_reporting_a_read_error
            .output
            .devices_rewritten_by_the_acquisition
            .is_empty(),
        "前提：盘 0 槽 0 自证得过，不补写盘 0"
    );
    let (window, offset_of_the_tree_table_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &with_the_slot_reporting_a_read_error.reads_reaching_the_disk,
        );
    let tree_table_reading_back_garbage_every_time = ReadFaultOnARange {
        device: None,
        offset: offset_of_the_tree_table_copies,
        length_in_bytes: BYTES_OF_A_TREE_TABLE,
        from_ordinal: *window
            .first()
            .expect("窗口是只进内存那一趟头一次读的树表两份那两次读"),
        fault: ReadFault::ReadsBackCorruptedBytes,
    };
    let faulted = mount_writable_with_read_faults_on_ranges(
        &image,
        &geometry,
        vec![
            slot_reporting_a_read_error,
            tree_table_reading_back_garbage_every_time,
        ],
    );
    let read_named_by_the_refusal =
        read_named_by_the_refusal_on_a_read_failing_self_verification(&faulted.result);
    assert_eq!(
        (
            read_named_by_the_refusal.offset,
            read_named_by_the_refusal.length_in_bytes
        ),
        (offset_of_the_tree_table_copies, BYTES_OF_A_TREE_TABLE),
        "拒因点名出故障的那份树表：交的是盘 {:?} 偏移 {:?} 长 {}",
        read_named_by_the_refusal.device,
        read_named_by_the_refusal.offset,
        read_named_by_the_refusal.length_in_bytes
    );
    assert_eq!(
        faulted.writes_reaching_the_disk, 0,
        "取号之前拒，一个写都没发到盘上"
    );
}
