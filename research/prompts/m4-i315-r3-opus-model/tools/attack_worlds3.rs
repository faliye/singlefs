//! m4-i315-r3 攻方腿固定盘面（不进仓，只在副本里跑；底子是第二轮辩方的 `defense_worlds.rs`，开关换成 apply_r3_checker.py 的 attack_set_arm）。
//! A（第一个文件，txg 3）之后覆盖写 B（txg 4 诞生、数据单元在 50182–50183）；B 的数据单元两块盘上的那一份按各形换掉或改坏，覆盖写 C 隔离它（或造成泄漏），
//! 再覆盖写 N 次让指着 A 的位置项转出根环；每张盘在四种判法下各判一次，打 I-3.1 / I-3.11 / I-3.15 / I-3.10 与其余违例。
//! 「别的住户」两种：别的池第一个文件那一份（诞生代号 3，早于 B 那条记录的分配代 4）、别的池覆盖写之后那一份（诞生代号 4，恰等于 B 的分配代、内容不同）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;

use common::{build_pool, parameters, publish_overwrite_in_process, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use scenarios_of_release_checksum_quarantine::{content_of, corrupt_the_copy_on, data_slot_of_the_first_version};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{attack_set_arm, attack_take_noloc_hits, check_pool_image};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};

#[derive(Clone, Copy, Debug)]
enum Mode {
    /// 合法（说谎的设备）：盘 1 上 B 那一份是诞生代号 3 的别的住户（K2-1 那一形），覆盖写 B 隔离它
    OlderTenantOnDeviceOne,
    /// 合法（两块盘同时丢写）：两块盘上 B 那一份都是同一个诞生代号 3 的别的住户
    SameOlderTenantOnBothDevices,
    /// 合法（代号核对推的盲区）：盘 1 上 B 那一份是诞生代号 4（等于记录分配代）、内容不同的别的住户
    EqualBirthTenantOnDeviceOne,
    /// 合法（两个盲区叠在一起）：两块盘上都是同一个诞生代号 4、内容不同的别的住户
    EqualBirthTenantOnBothDevices,
    /// 坏盘：泄漏（隔离之后改回原样，两份都好）
    LeakWithBothCopiesIntact,
    /// 坏盘：泄漏之后盘 1 那一份被诞生代号 4（等于分配代）的别的住户占了（字节与 EqualBirthTenantOnDeviceOne 相同）
    LeakThenEqualBirthTenantOnDeviceOne,
    /// 坏盘：泄漏之后盘 1 那一份被诞生代号 3 的别的住户占了（字节与 OlderTenantOnDeviceOne 相同）
    LeakThenOlderTenantOnDeviceOne,
    /// 坏盘：泄漏之后两块盘都被同一个诞生代号 3 的别的住户占了（字节与 SameOlderTenantOnBothDevices 相同）
    LeakThenSameOlderTenantOnBothDevices,
    /// 坏盘：泄漏之后盘 0 头扇区写零
    LeakThenZeroedHeaderOnDeviceZero,
}

/// 别的池里的一份数据单元：`overwritten` 为真取覆盖写之后那一版（50182 起，诞生代号 4），否则取第一个文件那一版（50180 起，诞生代号 3）。
fn foreign_unit(overwritten: bool) -> Vec<u8> {
    let mut pool = build_pool(if overwritten { "foreign-tenant-overwritten" } else { "foreign-tenant-first" });
    let first = pool.output.clone();
    let slot = if overwritten {
        publish_overwrite_in_process(&mut pool, &first, &content_of(2900, 11), FIXED_WRITE_TIME_SECONDS + 30, InstanceGeneration(1))
            .expect("别的池覆盖写")
            .data_pointers[0]
            .locations[0]
            .slot
    } else {
        first.data_pointers[0].locations[0].slot
    };
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let (_, device) = devices.iter().find(|(identity, _)| *identity == DeviceIdentity(0)).expect("盘 0");
    let mut bytes = vec![0u8; 32768];
    device.read_at(slot.to_device_offset(), &mut bytes).expect("读别的住户的单元");
    println!("FOREIGN overwritten={overwritten} slot={} birth_bytes={:?}", slot.0, &bytes[75..83]);
    bytes
}

fn put(devices: &mut [(DeviceIdentity, Recorded)], device: DeviceIdentity, slot: SlotNumber, bytes: &[u8]) {
    let (_, block_device) = devices.iter_mut().find(|(identity, _)| *identity == device).expect("池里有这块盘");
    block_device.write_at(DeviceOffsetInBytes(slot.to_device_offset().0), bytes, WriteDurability::Plain).expect("写");
}

fn build_world(tag: &str, mode: Mode, further_publishes: usize, older: &[u8], equal: &[u8]) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let _ = data_slot_of_the_first_version(&pool);
    // 被隔离（或泄漏）的是第二版 B（txg 4 诞生）的数据单元：别的池覆盖写之后那一份同样 txg 4 诞生、落在同一槽、内容不同，
    // 正是「旧住户诞生代号恰等于记录分配代」；别的池第一个文件那一份（txg 3）与 A 逐字节相同，当诞生代号更早的旧住户用。
    let second = publish_overwrite_in_process(&mut pool, &first, &content_of(2800, 5), FIXED_WRITE_TIME_SECONDS + 30, InstanceGeneration(1))
        .expect("覆盖写 B");
    let data_slot = second.data_pointers[0].locations[0].slot;
    assert_eq!(data_slot, SlotNumber(50182), "B 的数据单元与别的池覆盖写那一份落在同一槽");
    let first = second;
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        match mode {
            Mode::OlderTenantOnDeviceOne => put(devices, DeviceIdentity(1), data_slot, older),
            Mode::SameOlderTenantOnBothDevices => {
                put(devices, DeviceIdentity(0), data_slot, older);
                put(devices, DeviceIdentity(1), data_slot, older);
            }
            Mode::EqualBirthTenantOnDeviceOne => put(devices, DeviceIdentity(1), data_slot, equal),
            Mode::EqualBirthTenantOnBothDevices => {
                put(devices, DeviceIdentity(0), data_slot, equal);
                put(devices, DeviceIdentity(1), data_slot, equal);
            }
            _ => corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot),
        }
    }
    let mut current = publish_overwrite_in_process(&mut pool, &first, &content_of(2750, 7), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1))
        .expect("核出对不上时发布照成");
    let quarantined = current.quarantined_after_release_checksum_mismatch.len();
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        match mode {
            Mode::LeakWithBothCopiesIntact => corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot),
            Mode::LeakThenEqualBirthTenantOnDeviceOne => {
                corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot);
                put(devices, DeviceIdentity(1), data_slot, equal);
            }
            Mode::LeakThenOlderTenantOnDeviceOne => {
                corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot);
                put(devices, DeviceIdentity(1), data_slot, older);
            }
            Mode::LeakThenSameOlderTenantOnBothDevices => {
                put(devices, DeviceIdentity(0), data_slot, older);
                put(devices, DeviceIdentity(1), data_slot, older);
            }
            Mode::LeakThenZeroedHeaderOnDeviceZero => {
                corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot);
                put(devices, DeviceIdentity(0), data_slot, &[0u8; 4096]);
            }
            _ => {}
        }
    }
    for k in 0..further_publishes {
        current = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of(2700 + (k % 7), 6 + k),
            FIXED_WRITE_TIME_SECONDS + 120 + 60 * k as u64,
            InstanceGeneration(1),
        )
        .expect("接着覆盖写");
    }
    println!("WORLD {tag}: mode={mode:?} 隔离 {quarantined} 份");
    pool
}

fn show(verdict: &InvariantVerdict) -> &'static str {
    match verdict {
        InvariantVerdict::Holds => "成立",
        InvariantVerdict::Violated(_) => "违例",
        InvariantVerdict::NotApplicable(_) => "不适用",
        InvariantVerdict::PartiallyJudged(_) => "部分判",
    }
}

fn image_read(image: &singlefs_harness::memory_pool::MemoryPool, device: u32, slot: u64, buffer: &mut [u8]) {
    use singlefs_checker::image::ImageReader;
    if let Some(bytes) = image.read(device, slot * 16384, buffer.len()) {
        buffer.copy_from_slice(&bytes);
    }
}

#[test]
fn attack_worlds3() {
    let _ = parameters();
    let counts: Vec<usize> = std::env::var("ATTACK_ROTATIONS")
        .unwrap_or_else(|_| "24,30".into())
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    let older = foreign_unit(false);
    let equal = foreign_unit(true);
    for mode in [
        Mode::OlderTenantOnDeviceOne,
        Mode::SameOlderTenantOnBothDevices,
        Mode::EqualBirthTenantOnDeviceOne,
        Mode::EqualBirthTenantOnBothDevices,
        Mode::LeakWithBothCopiesIntact,
        Mode::LeakThenEqualBirthTenantOnDeviceOne,
        Mode::LeakThenOlderTenantOnDeviceOne,
        Mode::LeakThenSameOlderTenantOnBothDevices,
        Mode::LeakThenZeroedHeaderOnDeviceZero,
    ] {
        for n in &counts {
            let pool = build_world(&format!("w3-{mode:?}-{n}"), mode, *n, &older, &equal);
            let image = pool.memory_pool();
            let mut digest = Vec::new();
            for device in [0u32, 1] {
                let mut unit = vec![0u8; 32768];
                image_read(&image, device, 50182, &mut unit);
                digest.push(format!("{:08x}", singlefs_checker::crc32_castagnoli_table(&unit)));
            }
            for (arm_name, arm) in [("selfcheck", 0u8), ("gen", 1), ("cmp", 2), ("both", 3)] {
                attack_set_arm(arm);
                let _ = attack_take_noloc_hits();
                let verdicts = check_pool_image(&image);
                let mut hits = attack_take_noloc_hits();
                attack_set_arm(0);
                hits.sort();
                hits.dedup();
                let of = |name: &str| verdicts.iter().find(|(n, _)| *n == name).map(|(_, v)| show(v)).unwrap_or("无此条");
                let others: Vec<&str> = verdicts
                    .iter()
                    .filter(|(n, v)| matches!(v, InvariantVerdict::Violated(_)) && !["I-3.1", "I-3.11", "I-3.15", "I-3.10"].contains(n))
                    .map(|(n, _)| *n)
                    .collect();
                let cell: Vec<String> = hits
                    .iter()
                    .filter(|h| !h.starts_with("LOC|"))
                    .map(|h| h.split('|').skip(1).collect::<Vec<_>>().join("|"))
                    .collect();
                println!(
                    "W3 arm={arm_name:<9} {:<38} N={n:>2} I-3.1={} I-3.11={} I-3.15={} I-3.10={} 其余违例={others:?} 两份CRC={digest:?} 这一格={cell:?}",
                    format!("{mode:?}"),
                    of("I-3.1"),
                    of("I-3.11"),
                    of("I-3.15"),
                    of("I-3.10")
                );
            }
        }
    }
}
