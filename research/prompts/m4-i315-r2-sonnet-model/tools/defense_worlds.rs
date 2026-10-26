//! 辩方腿草稿探针（m4-i315-r2，不进仓，只在副本里跑）：固定盘面，CPU 池级 checker 判，开关 I315_ARM（today / exempt / gen）。
//! 世界：A 的数据单元在两块盘上被换成别的住户的完整单元（吞写落在复用槽上的样子）或头写零，或隔离之后又改回原样（泄漏、两份都好）；
//! 覆盖写 B 隔离它，再接着覆盖写 N 次，让指着那一份的位置项转出根环（N 小的盘上位置项还在，走另一支）。
//! 每张盘打 I-3.1 / I-3.11 / I-3.15 的判定与「没有位置项可比」回退点的日志（NOENTRY：自证结果、头里诞生代号、记录的分配代）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;

use common::{build_pool, parameters, publish_overwrite_in_process, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use scenarios_of_release_checksum_quarantine::{content_of, corrupt_the_copy_on, data_slot_of_the_first_version};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, defense_take_log};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};

#[derive(Clone, Copy, Debug)]
enum Mode {
    /// 两块盘上 A 的槽被换成别的住户的完整单元（诞生代号不同），覆盖写 B 隔离它
    ForeignUnitOnBothDevices,
    /// 只盘 0 换成别的住户的完整单元，盘 1 好
    ForeignUnitOnDeviceZero,
    /// 盘 0 头扇区写零，盘 1 好
    ZeroedHeaderOnDeviceZero,
    /// 盘 0 改坏一个字节，覆盖写 B 隔离之后改回原样：记录留在已分配、两份都好（泄漏，与合法隔离不是同一份字节）
    LeakWithBothCopiesIntact,
    /// 隔离之后，A 的槽被换成别的住户的完整单元（泄漏之后槽被别的东西占了；字节与 ForeignUnitOnBothDevices 相同）
    LeakThenForeignUnit,
}

fn foreign_unit() -> Vec<u8> {
    let mut pool = build_pool("foreign-tenant");
    let first = pool.output.clone();
    let overwritten = publish_overwrite_in_process(&mut pool, &first, &content_of(2900, 11), FIXED_WRITE_TIME_SECONDS + 30, InstanceGeneration(1))
        .expect("别的池覆盖写");
    let slot = overwritten.data_pointers[0].locations[0].slot;
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let (_, device) = devices.iter().find(|(identity, _)| *identity == DeviceIdentity(0)).expect("盘 0");
    let mut bytes = vec![0u8; 32768];
    device.read_at(slot.to_device_offset(), &mut bytes).expect("读别的住户的单元");
    bytes
}

fn put(devices: &mut [(DeviceIdentity, Recorded)], device: DeviceIdentity, slot: SlotNumber, bytes: &[u8]) {
    let (_, block_device) = devices.iter_mut().find(|(identity, _)| *identity == device).expect("池里有这块盘");
    block_device.write_at(DeviceOffsetInBytes(slot.to_device_offset().0), bytes, WriteDurability::Plain).expect("写");
}

fn build_world(tag: &str, mode: Mode, further_publishes: usize) -> BuiltPool {
    let foreign = foreign_unit();
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        match mode {
            Mode::ForeignUnitOnBothDevices => {
                put(devices, DeviceIdentity(0), data_slot, &foreign);
                put(devices, DeviceIdentity(1), data_slot, &foreign);
            }
            Mode::ForeignUnitOnDeviceZero => put(devices, DeviceIdentity(0), data_slot, &foreign),
            Mode::ZeroedHeaderOnDeviceZero => put(devices, DeviceIdentity(0), data_slot, &[0u8; 4096]),
            Mode::LeakWithBothCopiesIntact | Mode::LeakThenForeignUnit => corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot),
        }
    }
    let mut current = publish_overwrite_in_process(&mut pool, &first, &content_of(2800, 5), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1))
        .expect("核出对不上时发布照成");
    println!("WORLD {tag}: mode={mode:?} 隔离 {} 份", current.quarantined_after_release_checksum_mismatch.len());
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        match mode {
            Mode::LeakWithBothCopiesIntact => corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot),
            Mode::LeakThenForeignUnit => {
                put(devices, DeviceIdentity(0), data_slot, &foreign);
                put(devices, DeviceIdentity(1), data_slot, &foreign);
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

fn judge_and_print(label: &str, pool: &BuiltPool) {
    let image = pool.memory_pool();
    let _ = defense_take_log();
    let verdicts = check_pool_image(&image);
    let log = defense_take_log();
    let of = |name: &str| verdicts.iter().find(|(n, _)| *n == name).map(|(_, v)| show(v)).unwrap_or("无此条");
    let violated: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect();
    let mut noentry: Vec<String> = log.iter().filter(|l| l.starts_with("NOENTRY")).cloned().collect();
    noentry.sort();
    noentry.dedup();
    let arm = std::env::var("I315_ARM").unwrap_or_else(|_| "today".into());
    println!(
        "SCEN arm={arm} {label} : I-3.1={} I-3.11={} I-3.15={} 全部违例={violated:?} noentry={noentry:?}",
        of("I-3.1"),
        of("I-3.11"),
        of("I-3.15")
    );
}

#[test]
fn quarantine_worlds_with_the_location_entry_rotated_out_of_the_root_ring() {
    let _ = parameters();
    let counts: Vec<usize> = std::env::var("DEFENSE_ROTATIONS")
        .unwrap_or_else(|_| "0,3,8,20".into())
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    for mode in [
        Mode::ForeignUnitOnBothDevices,
        Mode::ForeignUnitOnDeviceZero,
        Mode::ZeroedHeaderOnDeviceZero,
        Mode::LeakWithBothCopiesIntact,
        Mode::LeakThenForeignUnit,
    ] {
        for n in &counts {
            let pool = build_world(&format!("w-{mode:?}-{n}"), mode, *n);
            judge_and_print(&format!("{mode:?} 之后再覆盖写{n}次"), &pool);
        }
    }
}
