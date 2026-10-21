//! harness 测试文件 `crates/singlefs-harness/tests/release_checksum_quarantine.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
//! [`a_quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup`] 在 harness 档那一侧的同名用例住在
//! `crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`（I-3.15 的合法世界与必红世界都在那一份），它收下这里交的镜像就地判。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, publish_overwrite_in_process, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    SlotNumber,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{raise_rollback_floor, ShadowLedger};
use singlefs_core::transaction::{QuarantinedCopyReading, TransactionUnit};

/// 单元头所在的那一个扇区（码 1 的明文头 105 字节在里面）：写零之后 magic 与头校验和都没了，单元头用不了。
const UNIT_HEADER_SECTOR_BYTES: usize = 512;

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 11 + seed) % 247).expect("小于 256"))
        .collect()
}

/// 改坏一块盘上某个槽起的那一份单元：翻它第 4096 字节起那个扇区的第一个字节（数据单元与索引节点都罩得住这一处），
/// 位置项里的校验和从此对不上。写经录制器落盘，与别的写一样进录制流。
pub fn corrupt_the_copy_on(
    devices: &mut [(DeviceIdentity, Recorded)],
    device: DeviceIdentity,
    slot: SlotNumber,
) {
    let (_, block_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let sector_offset = DeviceOffsetInBytes(slot.to_device_offset().0 + 4096);
    let mut sector = vec![0u8; 512];
    block_device
        .read_at(sector_offset, &mut sector)
        .expect("读那一个扇区");
    sector[0] ^= 0xff;
    block_device
        .write_at(sector_offset, &sector, WriteDurability::Plain)
        .expect("写回改坏的扇区");
}

/// A（txg 3）的数据单元：新池新建文件把它放在 50180–50181（字节表五）。
pub fn data_slot_of_the_first_version(pool: &BuiltPool) -> SlotNumber {
    let data_slot = pool.output.data_pointers[0].locations[0].slot;
    assert_eq!(
        data_slot,
        SlotNumber(50180),
        "A 的数据单元落在 50180–50181（字节表五）"
    );
    data_slot
}

/// A 的数据单元在 `corrupted_devices` 那几块盘上那一份改坏，覆盖写 B 隔离它（两盘那条记录都留在已分配），再覆盖写 C、D、E，抬 F 到 4——
/// A 的根（txg 3）掉出候选集，那两条记录从此没有任何有效根引用。交回池与 A 数据单元的起点槽。
pub fn pool_whose_quarantined_records_no_root_references(
    tag: &str,
    corrupted_devices: &[DeviceIdentity],
) -> (BuiltPool, SlotNumber) {
    let (pool, data_slot, _) =
        pool_whose_quarantined_records_no_root_references_with_the_newest_data_slot(
            tag,
            corrupted_devices,
        );
    (pool, data_slot)
}

/// 同 [`pool_whose_quarantined_records_no_root_references`]，再交回最新那一版（抬 F 那一串的最后一次空发布，数据单元照抄 E 的）
/// 数据单元的起点槽。
pub fn pool_whose_quarantined_records_no_root_references_with_the_newest_data_slot(
    tag: &str,
    corrupted_devices: &[DeviceIdentity],
) -> (BuiltPool, SlotNumber, SlotNumber) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for device in corrupted_devices {
            corrupt_the_copy_on(devices, *device, data_slot);
        }
    }
    let mut current = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(2800, 5),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("核出对不上时发布照成");
    assert_eq!(
        current.quarantined_after_release_checksum_mismatch.len(),
        2,
        "两盘那一份都隔离"
    );
    for (seed, seconds) in [(6, 120), (7, 180), (8, 240)] {
        current = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of(2700, seed),
            FIXED_WRITE_TIME_SECONDS + seconds,
            InstanceGeneration(1),
        )
        .expect("接着覆盖写");
    }
    assert_eq!(current.root.checkpoint_txg, CheckpointTxg(7));
    let raised = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        raise_rollback_floor(
            &publish_parameters,
            devices,
            &mut pool.allocator,
            &mut current,
            CheckpointTxg(4),
            ShadowLedger::On,
        )
        .expect("抬 F 到 4：A 的根（txg 3）掉出候选集")
    };
    assert!(!raised.publishes.is_empty(), "抬 F 发了空发布");
    let newest_data_slot = current.data_pointers[0].locations[0].slot;
    (pool, data_slot, newest_data_slot)
}

/// 用例 10：I-3.1（已分配统计对得上） 与 I-3.11（已分配减 defer 等于最新根走读） 怎么认隔离的记录（用户 2026-09-25 定，
/// D19（块指针的结构与宽度预算） 已定项 5 的隔离；按单元判是主 agent 同日定的读法）：已分配而没有任何根引用的记录，它罩住的单元
/// 只要有一份 checker 自己读出来读不出或校验和对不上，那个单元每块盘上的记录都豁免；每一份都读得出且对得上的照旧判违例。
/// 1. 两盘那一份都改坏：两条记录豁免，I-3.1、I-3.11 都成立（改之前 checker 不认隔离，两条都红）；
/// 2. 把那两份改回对得上（再翻一次同一个字节）：同样两条记录不再豁免，I-3.1、I-3.11 照红。
pub fn quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, data_slot) = pool_whose_quarantined_records_no_root_references(
        "release-checksum-checker-exemption",
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    let image_with_the_damaged_copies = pool.memory_pool();
    // I-3.1、I-3.11：隔离的那两条记录没有根引用，checker 读那一份自证不过 ⇒ 豁免
    images.hand_out(
        "两盘那一份都改坏",
        || image_with_the_damaged_copies.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_holds("I-3.1")
            .and_holds("I-3.11"),
    );

    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot);
        corrupt_the_copy_on(devices, DeviceIdentity(1), data_slot);
    }
    let image_with_the_copies_intact_again = pool.memory_pool();
    // I-3.1、I-3.11：那两份改回对得上，同样两条记录读得出且对得上 ⇒ 不豁免、照红
    images.hand_out(
        "那两份改回对得上",
        || image_with_the_copies_intact_again.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated("I-3.1")
            .and_violated("I-3.11"),
    );
}

/// 用例 11（按单元判，主 agent 2026-09-25 定）：只盘 0 那一份改坏——覆盖写把两块盘的记录一起留在已分配（D19 已定项 5），
/// 盘 1 那一份对得上。那个单元有一份坏，两块盘上的两条记录都豁免：I-3.1、I-3.11 都成立（按份判的话盘 1 那条恒红）。
/// 把盘 0 那一份改回对得上：单元两份都好，两条记录照红。
pub fn unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, data_slot) = pool_whose_quarantined_records_no_root_references(
        "release-checksum-checker-exemption-one-copy",
        &[DeviceIdentity(0)],
    );
    let image_with_one_damaged_copy = pool.memory_pool();
    // I-3.1、I-3.11：单元有一份坏 ⇒ 两块盘上的两条记录都豁免
    images.hand_out(
        "只盘 0 那一份改坏",
        || image_with_one_damaged_copy.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_holds("I-3.1")
            .and_holds("I-3.11"),
    );
    corrupt_the_copy_on(
        pool.devices.as_mut().expect("镜像还开着"),
        DeviceIdentity(0),
        data_slot,
    );
    let image_with_both_copies_intact = pool.memory_pool();
    // I-3.1、I-3.11：单元两份都好 ⇒ 不豁免、照红
    images.hand_out(
        "单元两份都好",
        || image_with_both_copies_intact.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated("I-3.1")
            .and_violated("I-3.11"),
    );
}

/// 用例 14（实七；实四丙交回 ⑤ 那一形）：隔离的记录罩住的那一份是一个**完整的别的单元**——magic、头校验和、载荷 CRC 都过得去（自证过），
/// 与指着它的位置项里的单元校验和对不上。丢一次写落在复用过的槽上时就是这样：槽里留着上一个单元，写者释放之前读盘核按位置项核出对不上、
/// 隔离（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。checker 的豁免与写者同一个判据（对位置项里的校验和）：两条记录豁免，
/// I-3.1、I-3.11 都成立；按自证判（改之前的读法）两份都过得去、不豁免，两条都红。
/// 指着那一份的位置项只在 A 的根（txg 3）里，而 F 已抬到 4、A 不在候选集里：checker 要把根环里没走过的根也走一遍才收得到它。
pub fn quarantined_copy_that_is_a_whole_other_self_verifying_unit_is_exempted_against_the_checksum_in_its_location_entry(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, data_slot, newest_data_slot) =
        pool_whose_quarantined_records_no_root_references_with_the_newest_data_slot(
            "release-checksum-checker-location-entry",
            &[DeviceIdentity(0), DeviceIdentity(1)],
        );
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let (_, block_device) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device)
                .expect("池里有这块盘");
            let mut other_unit = vec![0u8; data_unit_bytes];
            block_device
                .read_at(newest_data_slot.to_device_offset(), &mut other_unit)
                .expect("读最新那一版的数据单元");
            assert!(
                singlefs_checker::check_unit(&other_unit).is_ok(),
                "搬过去的是一个自证过得去的完整单元"
            );
            block_device
                .write_at(
                    data_slot.to_device_offset(),
                    &other_unit,
                    WriteDurability::Plain,
                )
                .expect("整份写进 A 的数据单元那一槽");
        }
    }
    let image = pool.memory_pool();
    // I-3.1、I-3.11：隔离的那两份自证过得去、与 A 的根里的位置项对不上 ⇒ 按位置项判、豁免
    images.hand_out(
        "隔离的那两份是一个完整的别的单元",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_holds("I-3.1")
            .and_holds("I-3.11"),
    );
}

/// 合法世界「写者隔离了一份头用不了的单元」（判决 `research/prompts/m4-i315-r1-main-verification.md` 第三节 C1「乙二」，调查员那一形）：
/// 新池新建文件之后，数据单元盘 1 那一份的头扇区写零（单元头用不了）；覆盖写换下它时写者释放之前读盘核出盘 1 那一份对不上，
/// 两块盘上那两条记录一起留在已分配（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），覆盖写那一版的映射里没有 key 指着那一槽。
/// 隔离判据读这一单元每块盘的那一份，盘 1 那一份对不上位置项，这个单元是隔离的单元：I-3.15 成立；I-3.1、I-3.11 照它们那一份隔离判据也成立。
/// 组只取头用得了的那一份（改之前的读法）时盘 0 那一份对得上位置项，I-3.15 红。
/// GPU 与 CPU 对拍罩这一形：组里读不读头用不了、没打对不上映射记号的那一份，扫描内核与 `walk.rs` 各写一份，这一张镜像上两边分得开。
pub fn a_quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup(
    images: &mut PoolCheckerImages,
) {
    let mut pool = build_pool("release-checksum-quarantined-unit-with-an-unusable-header");
    let old_slot = pool
        .output
        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .slot;
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let (_, device_one) = devices
            .iter_mut()
            .find(|(identity, _)| *identity == DeviceIdentity(1))
            .expect("池里有盘 1");
        device_one
            .write_at(
                old_slot.to_device_offset(),
                &[0u8; UNIT_HEADER_SECTOR_BYTES],
                WriteDurability::Plain,
            )
            .expect("盘 1 那一份的头扇区写零");
    }
    let mut overwritten_content = file_content();
    overwritten_content[0] ^= 0xff;
    let overwrite =
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &overwritten_content);
    assert_ne!(
        overwrite
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .slot,
        old_slot,
        "覆盖写把数据单元写到了别的槽"
    );
    assert!(
        overwrite
            .quarantined_after_release_checksum_mismatch
            .iter()
            .any(|copy| copy.device == DeviceIdentity(1)
                && copy.placement.slot == old_slot
                && copy.reading == QuarantinedCopyReading::ChecksumMismatch),
        "写者释放之前核出盘 1 那一份对不上、隔离了它：{:?}",
        overwrite.quarantined_after_release_checksum_mismatch
    );
    let image = pool.memory_pool();
    // I-3.15、I-3.1、I-3.11：隔离的单元（盘 1 那一份头用不了、覆盖写那一版没有 key 指着那一槽）不判红
    images.hand_out(
        "盘 1 那一份头用不了、覆盖写那一版没有 key 指着那一槽",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_holds("I-3.15")
            .and_holds("I-3.1")
            .and_holds("I-3.11"),
    );
}
