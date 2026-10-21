//! 可写挂载取号写算世代号、覆写哪一槽那一读是这一刻读盘，算进「取号写之前那几次读」（代码轮
//! `research/prompts/m3-b44-code-r3-main-verification.md` 第二节 L1 ② 与第三节第 1 条；攻方改法表「丁」）：两槽轮换覆写的要是这一刻盘上
//! 较旧的那一槽（`transaction` 取号那一段「取号写覆写的是这块盘上较旧的那一槽」），这次挂载预演与真做共用的那一份不替它读。
//! 镜像是 `common_reads_shared_by_the_rehearsal_and_the_real_run` 的窄盘（没改坏的那一份）：取号写两轮（甲'）之后盘 0 槽 0 世代 10、槽 1 世代 11，
//! 最新的是槽 1（攻方原型量的时候只写一轮，是槽 0 世代 10、槽 1 世代 9）。
//! 故障照攻方原型那一形（`research/prompts/m3-b44-code-r3-opus-model/attack_shared_reads_probe.rs`，计划 `w:303:1:garbage`），落点跟着最新那一槽换到
//! 整池第 304 次读——这次挂载头一次把盘 0 槽 1 读进共用的那一份（判号那一读）——读回坏字节一次，盘上是好的。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_reads_shared_by_the_rehearsal_and_the_real_run;

use std::collections::BTreeSet;

use common_reads_shared_by_the_rehearsal_and_the_real_run::{
    assert_the_mount_is_made_and_the_continuation_walks_to_empty,
    mount_writable_on_recording_devices, mount_writable_without_faults, narrow_image, FaultedReads,
    ReadFault, ReadReachingTheDisk, WriteReachingTheDisk,
    READ_OF_SLOT_ONE_OF_DEVICE_ZERO_INTO_THE_SHARED_READS,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::recovery::PoolReader;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;

/// 镜像里盘 0 两槽的世代号（攻方 `out/sysconf-writes.log` 量的是只写一轮那一版：「slot0:generation=10 … slot1:generation=9」；
/// 取号写两轮之后槽 1 是 11）。
const SLOT_GENERATIONS_OF_DEVICE_ZERO_BEFORE_THE_MOUNT: [u64; 2] = [10, 11];

fn slot_generation_on_the_image(
    image: &impl PoolReader,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
) -> u64 {
    let bytes = image
        .read(
            device,
            offset,
            usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
        )
        .expect("系统配置槽读得到");
    SystemConfiguration::parse_slot(&bytes)
        .expect("镜像上那一槽自证得过")
        .quantities
        .slot_generation
}

/// 场景：没改坏的窄盘镜像上可写挂载一次，整池第 304 次读（盘 0 槽 1 头一次读进共用的那一份）读回坏字节一次；那一槽是这块盘最新的一槽（世代 11），
/// 盘上是好的。
/// 预期：挂载做成、续做走到空；这次挂载的第一个写（取号第一轮那一写）写盘 0 槽 0（较旧那一槽，世代 10），与不带故障那一次的第一个写相同；
/// 两块盘与不带故障那一次挂载之后逐字节相同；取号写之前，屏障之后那一遍见证值读（盘 1 两槽最后读）之后，盘 0 两槽又各读到盘上一次——
/// 取号写算世代号那一读这一刻读盘、不取共用的那一份。
/// 判别力：这一读走共用的那一份（`crates/mutations.tsv`「4-4 修红 W17 取号写与轮换写算世代号的那一读走共用的那一份」）时，
/// 见证值读之后、第一个写之前盘 0 两槽不再读到盘上，最后一条断言红。前面几条断言这时不红：判号那一读核出盘 0 槽 1 读坏、
/// 已从共用的那一份里丢掉（坏字节不留），之后读它的穿到盘上读到好的，取号写照样落在较旧那一槽。
#[test]
fn the_newest_slot_of_device_zero_read_back_garbage_once_into_the_shared_reads_leaves_the_acquisition_write_on_the_older_slot_read_from_the_disk(
) {
    let (image, geometry, _) = narrow_image();
    let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
    let slot_offsets = [DeviceOffsetInBytes(0), DeviceOffsetInBytes(spacing)];
    assert_eq!(
        slot_offsets.map(|offset| slot_generation_on_the_image(&image, DeviceIdentity(0), offset)),
        SLOT_GENERATIONS_OF_DEVICE_ZERO_BEFORE_THE_MOUNT,
        "前提：盘 0 槽 1 是这块盘最新的一槽"
    );
    let fault_free = mount_writable_without_faults(&image, &geometry);
    assert_the_mount_is_made_and_the_continuation_walks_to_empty(
        &fault_free.result,
        "前提：不带故障时没改坏的那一份",
    );
    let read_into_the_shared_reads = fault_free.reads_reaching_the_disk[usize::try_from(
        READ_OF_SLOT_ONE_OF_DEVICE_ZERO_INTO_THE_SHARED_READS - 1,
    )
    .expect("读序数装得进 usize")];
    assert!(
        read_into_the_shared_reads.device == DeviceIdentity(0)
            && read_into_the_shared_reads.offset == slot_offsets[1]
            && read_into_the_shared_reads.reads_a_system_configuration_slot(&geometry)
            && read_into_the_shared_reads.is_before_the_first_write_of_the_mount(),
        "前提：整池第 {READ_OF_SLOT_ONE_OF_DEVICE_ZERO_INTO_THE_SHARED_READS} 次读是第一个写之前读盘 0 槽 1：{read_into_the_shared_reads:?}"
    );
    let first_write_without_the_fault = *fault_free
        .writes_reaching_the_disk
        .first()
        .expect("不带故障时这次挂载有写");
    assert_eq!(
        first_write_without_the_fault,
        WriteReachingTheDisk {
            device: DeviceIdentity(0),
            offset: slot_offsets[0],
            length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
        },
        "前提：不带故障时第一个写是取号第一轮那一写、写盘 0 较旧的槽 0"
    );

    let faulted = mount_writable_on_recording_devices(
        &image,
        &geometry,
        ReadFault::ReadsBackCorruptedBytes,
        FaultedReads::AtTheOrdinals(
            [READ_OF_SLOT_ONE_OF_DEVICE_ZERO_INTO_THE_SHARED_READS]
                .into_iter()
                .collect::<BTreeSet<u64>>(),
        ),
    );
    let corrupted: Vec<(DeviceIdentity, DeviceOffsetInBytes)> = faulted
        .reads_that_carried_the_fault
        .iter()
        .map(|read| (read.device, read.offset))
        .collect();
    assert_eq!(
        corrupted,
        vec![(DeviceIdentity(0), slot_offsets[1])],
        "盘 0 槽 1 恰读回坏字节一次"
    );
    assert_the_mount_is_made_and_the_continuation_walks_to_empty(
        &faulted.result,
        "盘 0 槽 1 一次读回坏字节、盘上是好的",
    );
    assert_eq!(
        faulted.writes_reaching_the_disk.first().copied(),
        Some(first_write_without_the_fault),
        "取号第一轮那一写覆写的是盘 0 较旧的槽 0，不是最新那一槽"
    );
    assert!(
        faulted.images_after == fault_free.images_after,
        "两块盘与不带故障那一次挂载之后逐字节相同"
    );
    let reads_before_the_first_write: Vec<&ReadReachingTheDisk> = faulted
        .reads_reaching_the_disk
        .iter()
        .filter(|read| read.is_before_the_first_write_of_the_mount())
        .collect();
    let last_witness_read_of_device_one = reads_before_the_first_write
        .iter()
        .rposition(|read| read.device == DeviceIdentity(1) && read.offset == slot_offsets[1])
        .expect("第一个写之前读过盘 1 槽 1（屏障之后那一遍见证值读）");
    let device_zero_slots_read_after_the_witness: Vec<DeviceOffsetInBytes> =
        reads_before_the_first_write[last_witness_read_of_device_one + 1..]
            .iter()
            .filter(|read| {
                read.device == DeviceIdentity(0)
                    && read.reads_a_system_configuration_slot(&geometry)
            })
            .map(|read| read.offset)
            .collect();
    assert!(
        device_zero_slots_read_after_the_witness.starts_with(&slot_offsets),
        "见证值读之后、取号写之前盘 0 两槽各读到盘上一次（取号写算世代号那一读这一刻读盘）：{device_zero_slots_read_after_the_witness:?}"
    );
}
