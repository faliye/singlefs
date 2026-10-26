//! m4-i315-r3 辩方腿固定盘面（不进仓，只在副本里跑）：A 的数据单元那一槽被一个旧住户占着（吞写落在复用槽上的样子），覆盖写 B 隔离它，
//! 再接着覆盖写 N 次让指着那一份的位置项转出根环；每张盘按九个臂各判一遍，打 I-3.1 / I-3.11 / I-3.15 与回退点的事实。
//! 环境变量：DEFENSE_ROTATIONS（逗号分隔，默认 0,8,20,24,30）、DEFENSE_ARMS、DEFENSE_MODES（逗号分隔的世界名，默认全部）、DEFENSE_DUMP=1。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;

use common::{build_pool, build_pool_with_first_file, build_pool_with_first_file_as_instance, file_content, parameters, publish_overwrite_in_process, set_fsid_tweak, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use scenarios_of_release_checksum_quarantine::{content_of, corrupt_the_copy_on, data_slot_of_the_first_version};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, defense_set_arm, defense_take_log};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// 两块盘上 A 的槽被换成同一个别的住户的完整单元（诞生代号 4，别的池覆盖写出来的），覆盖写 B 隔离它
    ForeignUnitOnBothDevices,
    /// 只盘 0 换成别的住户的完整单元，盘 1 好
    ForeignUnitOnDeviceZero,
    /// 盘 0 头扇区写零，盘 1 好
    ZeroedHeaderOnDeviceZero,
    /// 盘 0 改坏，覆盖写 B 隔离之后改回原样：记录留在已分配、两份都好（泄漏）
    LeakWithBothCopiesIntact,
    /// 隔离之后，A 的槽被换成别的住户的完整单元（泄漏之后槽被占了）
    LeakThenForeignUnit,
    /// 两块盘上是两个不同的别的住户
    DifferentForeignUnitsOnTheTwoDevices,
    /// 两块盘上同一个旧住户，诞生代号恰等于记录的分配代 3，写序实例、fsid 都相同，只有载荷不同（别的内容、同一个身份）
    SameIdentityOnBothDevices,
    /// 同上，只盘 0
    SameIdentityOnDeviceZero,
    /// 两块盘上同一个旧住户，诞生代号恰等于 3，fsid 不同（重新 mkfs 的池留下的单元）
    RemadePoolSameBirthOnBothDevices,
    /// 同上，只盘 0
    RemadePoolSameBirthOnDeviceZero,
    /// 主池的 A 由第二个实例写（取号两次），槽上留着第一个实例同一个 txg 写的旧单元（内容不同、fsid 相同）：上一次挂载写出单元、根没落盘就崩了
    EarlierInstanceLeftoverOnBothDevices,
    /// 同上，只盘 0
    EarlierInstanceLeftoverOnDeviceZero,
}

const ALL_MODES: [Mode; 12] = [
    Mode::ForeignUnitOnBothDevices,
    Mode::ForeignUnitOnDeviceZero,
    Mode::ZeroedHeaderOnDeviceZero,
    Mode::LeakWithBothCopiesIntact,
    Mode::LeakThenForeignUnit,
    Mode::DifferentForeignUnitsOnTheTwoDevices,
    Mode::SameIdentityOnBothDevices,
    Mode::SameIdentityOnDeviceZero,
    Mode::RemadePoolSameBirthOnBothDevices,
    Mode::RemadePoolSameBirthOnDeviceZero,
    Mode::EarlierInstanceLeftoverOnBothDevices,
    Mode::EarlierInstanceLeftoverOnDeviceZero,
];

fn unit_bytes_of(pool: &mut BuiltPool, slot: SlotNumber) -> Vec<u8> {
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let (_, device) = devices.iter().find(|(identity, _)| *identity == DeviceIdentity(0)).expect("盘 0");
    let mut bytes = vec![0u8; 32768];
    device.read_at(slot.to_device_offset(), &mut bytes).expect("读单元");
    bytes
}

/// 别的池覆盖写一次之后的数据单元（诞生代号 4）。
fn foreign_overwritten_unit(tag: &str, seed: usize) -> Vec<u8> {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let overwritten = publish_overwrite_in_process(&mut pool, &first, &content_of(2900, seed), FIXED_WRITE_TIME_SECONDS + 30, InstanceGeneration(1))
        .expect("别的池覆盖写");
    let slot = overwritten.data_pointers[0].locations[0].slot;
    unit_bytes_of(&mut pool, slot)
}

/// 别的内容的第一个文件的数据单元：诞生代号、写序实例、fsid 都与 A 的相同（同一套参数、同一条搭建路径），载荷不同。
fn foreign_first_unit(tag: &str, fsid_tweak: u64) -> Vec<u8> {
    set_fsid_tweak(fsid_tweak);
    let mut pool = build_pool_with_first_file(tag, &content_of(3000, 17));
    set_fsid_tweak(0);
    let slot = pool.output.data_pointers[0].locations[0].slot;
    unit_bytes_of(&mut pool, slot)
}

fn put(devices: &mut [(DeviceIdentity, Recorded)], device: DeviceIdentity, slot: SlotNumber, bytes: &[u8]) {
    let (_, block_device) = devices.iter_mut().find(|(identity, _)| *identity == device).expect("池里有这块盘");
    block_device.write_at(DeviceOffsetInBytes(slot.to_device_offset().0), bytes, WriteDurability::Plain).expect("写");
}

fn build_world(tag: &str, mode: Mode, further_publishes: usize) -> BuiltPool {
    let (on_zero, on_one): (Option<Vec<u8>>, Option<Vec<u8>>) = match mode {
        Mode::ForeignUnitOnBothDevices | Mode::LeakThenForeignUnit => {
            let foreign = foreign_overwritten_unit("foreign-tenant", 11);
            (Some(foreign.clone()), Some(foreign))
        }
        Mode::ForeignUnitOnDeviceZero => (Some(foreign_overwritten_unit("foreign-tenant", 11)), None),
        Mode::ZeroedHeaderOnDeviceZero => (Some(vec![0u8; 4096]), None),
        Mode::LeakWithBothCopiesIntact => (None, None),
        Mode::DifferentForeignUnitsOnTheTwoDevices => (
            Some(foreign_overwritten_unit("foreign-tenant-a", 11)),
            Some(foreign_overwritten_unit("foreign-tenant-b", 12)),
        ),
        Mode::SameIdentityOnBothDevices => {
            let same = foreign_first_unit("same-identity", 0);
            (Some(same.clone()), Some(same))
        }
        Mode::SameIdentityOnDeviceZero => (Some(foreign_first_unit("same-identity", 0)), None),
        Mode::RemadePoolSameBirthOnBothDevices => {
            let remade = foreign_first_unit("remade-pool", 1);
            (Some(remade.clone()), Some(remade))
        }
        Mode::RemadePoolSameBirthOnDeviceZero => (Some(foreign_first_unit("remade-pool", 1)), None),
        Mode::EarlierInstanceLeftoverOnBothDevices => {
            let leftover = foreign_first_unit("earlier-instance", 0);
            (Some(leftover.clone()), Some(leftover))
        }
        Mode::EarlierInstanceLeftoverOnDeviceZero => (Some(foreign_first_unit("earlier-instance", 0)), None),
    };
    let instance_number: u32 = match mode {
        Mode::EarlierInstanceLeftoverOnBothDevices | Mode::EarlierInstanceLeftoverOnDeviceZero => 2,
        _ => 1,
    };
    let mut pool = build_pool_with_first_file_as_instance(tag, &file_content(), instance_number);
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        match mode {
            Mode::LeakWithBothCopiesIntact | Mode::LeakThenForeignUnit => {
                if mode == Mode::LeakWithBothCopiesIntact || mode == Mode::LeakThenForeignUnit {
                    corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot);
                }
            }
            _ => {
                if let Some(bytes) = &on_zero {
                    put(devices, DeviceIdentity(0), data_slot, bytes);
                }
                if let Some(bytes) = &on_one {
                    put(devices, DeviceIdentity(1), data_slot, bytes);
                }
            }
        }
    }
    let mut current = publish_overwrite_in_process(&mut pool, &first, &content_of(2800, 5), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(instance_number))
        .expect("核出对不上时发布照成");
    println!("WORLD {tag}: mode={mode:?} 隔离 {} 份", current.quarantined_after_release_checksum_mismatch.len());
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        match mode {
            Mode::LeakWithBothCopiesIntact => corrupt_the_copy_on(devices, DeviceIdentity(0), data_slot),
            Mode::LeakThenForeignUnit => {
                put(devices, DeviceIdentity(0), data_slot, on_zero.as_ref().expect("旧住户"));
                put(devices, DeviceIdentity(1), data_slot, on_one.as_ref().expect("旧住户"));
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
            InstanceGeneration(instance_number),
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

fn judge_and_print(label: &str, pool: &BuiltPool, arms: &[String]) {
    let image = pool.memory_pool();
    let mut facts_printed = false;
    for arm in arms {
        defense_set_arm(arm);
        let _ = defense_take_log();
        let verdicts = check_pool_image(&image);
        let log = defense_take_log();
        let of = |name: &str| verdicts.iter().find(|(n, _)| *n == name).map(|(_, v)| show(v)).unwrap_or("无此条");
        let violated: Vec<&str> = verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect();
        let others: Vec<&&str> = violated.iter().filter(|n| !["I-3.1", "I-3.11", "I-3.15"].contains(*n)).collect();
        println!("SCEN arm={arm} {label} : I-3.1={} I-3.11={} I-3.15={} 其余违例={others:?}", of("I-3.1"), of("I-3.11"), of("I-3.15"));
        if !facts_printed {
            let mut noentry: Vec<String> = log.iter().filter(|l| l.starts_with("NOENTRY")).map(|l| l.trim_start_matches("NOENTRY|").to_string()).collect();
            noentry.sort();
            noentry.dedup();
            println!("FACTS {label} : {noentry:?}");
            facts_printed = true;
        }
    }
    defense_set_arm("today");
}

#[test]
fn r3_quarantine_worlds_with_the_location_entry_rotated_out_of_the_root_ring() {
    let _ = parameters();
    let counts: Vec<usize> = std::env::var("DEFENSE_ROTATIONS")
        .unwrap_or_else(|_| "0,8,20,24,30".into())
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    let arms: Vec<String> = std::env::var("DEFENSE_ARMS")
        .unwrap_or_else(|_| "today,exempt,gen,genplus,cmp,both,bothplus,jrn,jrngen".into())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let wanted: Option<Vec<String>> = std::env::var("DEFENSE_MODES").ok().map(|v| v.split(',').map(|s| s.trim().to_string()).collect());
    for mode in ALL_MODES {
        if let Some(wanted) = &wanted {
            if !wanted.contains(&format!("{mode:?}")) {
                continue;
            }
        }
        for n in &counts {
            let pool = build_world(&format!("w-{mode:?}-{n}"), mode, *n);
            judge_and_print(&format!("{mode:?} N={n}"), &pool, &arms);
        }
    }
}
