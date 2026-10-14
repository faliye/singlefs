//! 系统配置每盘一份买到的冗余，在恢复路径上真的兑现（D22（单元原子性怎么合成） 已定项 8 第 1 条）：
//! 一块盘的两个系统配置槽都交不出东西（读不出来、或者读出来自证不过）时，恢复要改用别的盘上那一份把池挂起来；
//! 只有池里每块盘的两个槽都废了才报 `RecoveryFailure::NoValidSystemConfiguration`。
//!
//! 场景取自 `research/prompts/m2-supp3-item4-code-r1-opus-model/opus_probe_one_device_system_configuration.rs`
//! （攻方腿 K1 的冻结证据，不回改也不搬进来）；那里的断言钉的是修之前「当场 return Err」的样子，这里跟着改法翻面。
//! 「读不出来」在这里摆在 `PoolReader` 这一层：它的文档写的就是「读不到（没有那块盘、越界）返回 None」，
//! 块设备报 I/O 错时恢复看到的也正是 None（`impl PoolReader for [(DeviceIdentity, Device)]` 里的 `.ok()?`）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/system_configuration_per_device_redundancy.rs"]
mod scenarios_of_system_configuration_per_device_redundancy;

use common::{build_pool, file_content, parameters};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_system_configuration_per_device_redundancy::{
    system_configuration_slot_offsets, BYTE_FLIPPED_IN_THE_RESERVED_AREA_OF_THE_SLOT,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;

/// 把点名的那几个系统配置槽读成「读不出来」：`PoolReader::read` 对那几个 (盘, 盘内偏移) 返回 None，
/// 别的盘、别的偏移原样透传给底下的镜像。根环、journal 环、单元区一个字节都没动，
/// 所以恢复挂不挂得上，只取决于它肯不肯用别的盘上那一份系统配置。
struct PoolWithUnreadableSystemConfigurationSlots<'image> {
    image: &'image MemoryPool,
    unreadable_slots: Vec<(DeviceIdentity, DeviceOffsetInBytes)>,
}

impl<'image> PoolWithUnreadableSystemConfigurationSlots<'image> {
    /// 点名的那几块盘上**两个**系统配置槽都读不出来：整块盘掉了、或者头两个固定槽坏在硬件上。
    fn with_both_slots_unreadable_on(
        image: &'image MemoryPool,
        devices: &[DeviceIdentity],
    ) -> Self {
        let mut unreadable_slots = Vec::new();
        for device in devices {
            for offset in system_configuration_slot_offsets() {
                unreadable_slots.push((*device, offset));
            }
        }
        Self {
            image,
            unreadable_slots,
        }
    }

    /// 只有槽 0 读不出来，槽 1 还在：随机注入今天摆得出的那一格（同一块盘上只坏一次读）。
    fn with_only_slot_zero_unreadable_on(
        image: &'image MemoryPool,
        device: DeviceIdentity,
    ) -> Self {
        Self {
            image,
            unreadable_slots: vec![(device, system_configuration_slot_offsets()[0])],
        }
    }
}

impl PoolReader for PoolWithUnreadableSystemConfigurationSlots<'_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.image.device_identities()
    }
    /// 这个包装只让系统配置槽读不出，不改几何：盘多大照问内层。
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.image.device_size_in_bytes(device)
    }
    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        if self.unreadable_slots.contains(&(device, offset)) {
            return None;
        }
        PoolReader::read(self.image, device, offset, length)
    }
    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.image
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

/// 完好的池恢复出来的那一版：第一个文件的内容与它那条根。往下每条用例都跟它逐项比。
fn file_read_by_a_healthy_pool(image: &MemoryPool) -> (InstanceGeneration, CheckpointTxg) {
    let healthy = recover(image, JournalPolicy::Consult);
    let RecoveryOutcome::FileRead { root, content } = healthy.outcome else {
        panic!("完好的池本来就读得回第一个文件：{:?}", healthy.outcome);
    };
    assert_eq!(content, file_content(), "完好的池读回的是第一个文件的内容");
    root
}

/// 缺了几个系统配置槽之后恢复出来的那一版，要与完好的池逐项相同。
fn assert_reads_the_same_file_back(
    reader: &dyn PoolReader,
    healthy_root: (InstanceGeneration, CheckpointTxg),
    what_is_broken: &str,
) {
    let recovery = recover(reader, JournalPolicy::Consult);
    let RecoveryOutcome::FileRead { root, content } = recovery.outcome else {
        panic!(
            "{what_is_broken}：整池仍要挂得上，实际是 {:?}",
            recovery.outcome
        );
    };
    assert_eq!(root, healthy_root, "{what_is_broken}：走的还是同一条根");
    assert_eq!(content, file_content(), "{what_is_broken}：读回同一份内容");
}

/// 盘 1 的两个系统配置槽都读不出来，盘 0 的两个完好：用盘 0 那一份挂起来。
#[test]
fn both_system_configuration_slots_unreadable_on_device_one_still_read_the_file_back() {
    let pool = build_pool("system-configuration-redundancy-device-one-unreadable");
    let image = pool.memory_pool();
    let healthy_root = file_read_by_a_healthy_pool(&image);
    let healthy_system_configuration =
        choose_system_configuration(&image).expect("完好的池择得出系统配置");

    let degraded = PoolWithUnreadableSystemConfigurationSlots::with_both_slots_unreadable_on(
        &image,
        &[DeviceIdentity(1)],
    );
    let chosen = choose_system_configuration(&degraded)
        .expect("盘 1 两个系统配置槽都读不出来时，改用盘 0 上那一份");
    assert_eq!(
        chosen, healthy_system_configuration,
        "择出来的就是盘 0 上那一份，字段一个不差"
    );
    assert_reads_the_same_file_back(&degraded, healthy_root, "盘 1 两个系统配置槽都读不出来");
}

/// 反过来坏盘 0：择出来的换成盘 1 上那一份（`this_device` 跟着变成 1），文件照样读得回。
/// 单独立一条是因为修之前的写法在第一块出问题的盘上就 `return Err`，坏哪一块都挂不上。
#[test]
fn both_system_configuration_slots_unreadable_on_device_zero_still_read_the_file_back() {
    let pool = build_pool("system-configuration-redundancy-device-zero-unreadable");
    let image = pool.memory_pool();
    let healthy_root = file_read_by_a_healthy_pool(&image);

    let degraded = PoolWithUnreadableSystemConfigurationSlots::with_both_slots_unreadable_on(
        &image,
        &[DeviceIdentity(0)],
    );
    let chosen = choose_system_configuration(&degraded)
        .expect("盘 0 两个系统配置槽都读不出来时，改用盘 1 上那一份");
    assert_eq!(
        chosen.immutable.this_device,
        DeviceIdentity(1),
        "择出来的是盘 1 自己那一份"
    );
    assert_eq!(
        chosen.immutable.region_devices,
        parameters().region_devices,
        "两块盘上记的根环区域落点是同一套，换一份不改择根的去处"
    );
    assert_reads_the_same_file_back(&degraded, healthy_root, "盘 0 两个系统配置槽都读不出来");
}

/// 盘 0 的两个系统配置槽读得出来、但都自证不过（各翻掉一个字节）：与读不出来走同一条分支，照样挂得上。
/// 坏的挑盘 0 是为了让「用的是不是另一块盘那一份」看得见：`this_device` 只有改用盘 1 那一份时才是 1。
#[test]
fn both_system_configuration_slots_corrupted_on_device_zero_still_read_the_file_back() {
    let pool = build_pool("system-configuration-redundancy-device-zero-corrupt");
    let mut image = pool.memory_pool();
    let healthy_root = file_read_by_a_healthy_pool(&image);

    for offset in system_configuration_slot_offsets() {
        image.flip_byte(
            DeviceIdentity(0),
            offset,
            BYTE_FLIPPED_IN_THE_RESERVED_AREA_OF_THE_SLOT,
        );
    }
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for offset in system_configuration_slot_offsets() {
        let bytes = PoolReader::read(&image, DeviceIdentity(0), offset, slot_bytes)
            .expect("字节还在盘上，只是自证不过");
        assert!(
            SystemConfiguration::parse_slot(&bytes).is_err(),
            "翻过字节的槽自证不过：盘 0 偏移 {}",
            offset.0
        );
    }
    let chosen = choose_system_configuration(&image)
        .expect("盘 0 两个系统配置槽都自证不过时，改用盘 1 上那一份");
    assert_eq!(
        chosen.immutable.this_device,
        DeviceIdentity(1),
        "择出来的是盘 1 自己那一份"
    );
    assert_reads_the_same_file_back(&image, healthy_root, "盘 0 两个系统配置槽都自证不过");
}

/// 对照：盘 1 只坏一个系统配置槽，另一个还在 ⇒ 修之前修之后都挂得上。
/// 这一条钉的是这次改法没有把「单槽坏」那条路一起改坏。
#[test]
fn one_unreadable_system_configuration_slot_still_reads_the_file_back() {
    let pool = build_pool("system-configuration-redundancy-one-slot");
    let image = pool.memory_pool();
    let healthy_root = file_read_by_a_healthy_pool(&image);
    let healthy_system_configuration =
        choose_system_configuration(&image).expect("完好的池择得出系统配置");

    let degraded = PoolWithUnreadableSystemConfigurationSlots::with_only_slot_zero_unreadable_on(
        &image,
        DeviceIdentity(1),
    );
    let chosen =
        choose_system_configuration(&degraded).expect("盘 1 只坏一个系统配置槽时还择得出系统配置");
    assert_eq!(
        chosen, healthy_system_configuration,
        "盘 1 的槽 1 与盘 0 上那一份对得上，择出来的仍是盘 0 那一份"
    );
    assert_reads_the_same_file_back(&degraded, healthy_root, "盘 1 只坏一个系统配置槽");
}

/// 池里每块盘的每个系统配置槽都读不出来：这时才失败，错误里点名的是设备次序里第一块两槽皆废的盘。
#[test]
fn no_valid_system_configuration_slot_on_any_device_fails_and_names_the_first_device() {
    let pool = build_pool("system-configuration-redundancy-all-dead");
    let image = pool.memory_pool();
    assert_eq!(
        image.device_identities(),
        vec![DeviceIdentity(0), DeviceIdentity(1)],
        "设备次序是盘 0 在前：点名的第一块盘按这个次序算"
    );

    let dead = PoolWithUnreadableSystemConfigurationSlots::with_both_slots_unreadable_on(
        &image,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    assert_eq!(
        choose_system_configuration(&dead).err(),
        Some(RecoveryFailure::NoValidSystemConfiguration {
            first_device_with_no_valid_system_configuration_slot: DeviceIdentity(0)
        }),
        "一块盘都不剩时才报这个，点名的是次序里第一块"
    );
    let recovery = recover(&dead, JournalPolicy::Consult);
    assert_eq!(
        recovery.outcome,
        RecoveryOutcome::Failed {
            root: None,
            failure: RecoveryFailure::NoValidSystemConfiguration {
                first_device_with_no_valid_system_configuration_slot: DeviceIdentity(0)
            }
        },
        "整条恢复把这个失败原样报出去"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_system_configuration_per_device_redundancy::the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration()
{
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_system_configuration_per_device_redundancy::the_pool_checker_still_judges_every_invariant_when_one_device_has_no_valid_system_configuration,
    );
}
