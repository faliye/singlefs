//! m2-closeout-code-r3 云端攻方腿 X6：core 读 325 / 371、与第一版常量不等整池拒；checker 的 I-7.13 补 325 / 371 / 417 三项。
//! 第二轮攻方 Y4 那五格（A：417 改 +64；B：325 改 1023；C：371 改 65；D：环长改到段边界外、417 跟着现算；E：371 改 0），
//! 再加 F（371 改 128）、G（325 改 2048），每格三种改法：四个槽都改、只改盘 1 世代号小的那一槽、只改盘 1 世代号大的那一槽。
//! 每格看 core 只读挂载、可写挂载、池级 checker 的 I-7.13 与别的违例。只打印，钉快照今天的行为。

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use common_admission::{checker_violations_on, start_plain};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const SLOT: usize = 4096;

fn read_u64(slot: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(slot[offset..offset + 8].try_into().expect("8"))
}

fn read_slot(image: &MemoryPool, identity: DeviceIdentity, offset: u64) -> Vec<u8> {
    let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
    device.image = image.devices[&identity].clone();
    let mut slot = vec![0u8; SLOT];
    device.read_at(DeviceOffsetInBytes(offset), &mut slot).expect("读槽");
    slot
}

/// 槽的世代号在偏移 4 + 2 + 96 + 16 + 20 + 1 + 4 + 4 之后？不猜偏移：用 core 的解析读。
fn generation_of(slot: &[u8]) -> Option<u64> {
    singlefs_core::system_configuration::SystemConfiguration::parse_slot(slot)
        .ok()
        .map(|system_configuration| system_configuration.quantities.slot_generation)
}

/// 选中的槽里偏移 `offset` 那 8 字节改成 `value`（`and_ring` 给了就同时改偏移 333 的环长），重算整槽校验和。
fn rewrite_field(image: &mut MemoryPool, targets: &[(DeviceIdentity, u64)], offset: usize, value: u64, and_ring: Option<u64>) {
    for (identity, slot_offset) in targets {
        let mut slot = read_slot(image, *identity, *slot_offset);
        slot[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        if let Some(ring) = and_ring {
            slot[333..341].copy_from_slice(&ring.to_le_bytes());
        }
        let digest = wide_checksum_with_field_zeroed(&slot, SLOT, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
        slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
        image.devices.get_mut(identity).expect("盘").write(DeviceOffsetInBytes(*slot_offset), &slot);
    }
}

fn short(text: String) -> String {
    text.chars().take(230).collect::<String>().replace('\n', " ")
}

fn judge(tag: &str, parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters, image: &MemoryPool) {
    let read_only = match mount_read_only(image) {
        Ok(mounted) => format!("ok chosen=({},{})", mounted.chosen_root.instance.0, mounted.chosen_root.checkpoint_txg.0),
        Err(error) => format!("err={}", short(format!("{error:?}"))),
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = common_admission::plain_devices_on(image);
    let mut caller = parameters.clone();
    let slot = read_slot(image, DeviceIdentity(0), 0);
    caller.geometry.journal_ring_bytes = read_u64(&slot, 333);
    let writable = match mount_writable(&caller, &mut devices) {
        Ok(mounted) => format!("ok instance={}", mounted.output.instance.0),
        Err(error) => format!("err={}", short(format!("{error:?}"))),
    };
    let violations = checker_violations_on(image);
    let i_7_13 = violations.iter().filter(|line| line.starts_with("I-7.13")).count();
    println!(
        "Y4R3 {tag} core_read_only={read_only} core_writable={writable} checker_violations={} i_7_13={i_7_13} list={:?}",
        violations.len(),
        violations.iter().map(|line| line.chars().take(110).collect::<String>()).collect::<Vec<_>>()
    );
}

#[test]
fn system_configuration_fields_core_and_checker_refuse_the_same_table_cell_by_cell() {
    let pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    let base = pool.image();
    let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    judge("baseline", &pool.parameters, &base);
    let ring = pool.parameters.geometry.journal_ring_bytes;
    let unit_area_start = 1024 + ring / 16384;
    // 盘 1 两槽里世代号小的与大的那一槽。
    let generations: Vec<(u64, Option<u64>)> = [0, spacing].iter().map(|offset| (*offset, generation_of(&read_slot(&base, DeviceIdentity(1), *offset)))).collect();
    println!("Y4R3 device1 slot generations {generations:?}");
    let older = generations.iter().min_by_key(|(_, generation)| *generation).expect("两槽").0;
    let newer = generations.iter().max_by_key(|(_, generation)| *generation).expect("两槽").0;
    let every_slot = vec![(DeviceIdentity(0), 0), (DeviceIdentity(0), spacing), (DeviceIdentity(1), 0), (DeviceIdentity(1), spacing)];
    let cells: Vec<(&str, usize, u64, Option<u64>)> = vec![
        ("A_417_plus_64", 417, unit_area_start + 64, None),
        ("B_325_is_1023", 325, 1023, None),
        ("C_371_is_65", 371, 65, None),
        ("D_ring_off_the_segment_boundary", 417, 1024 + (768 * 1024 * 1024 - 16384) / 16384, Some(768 * 1024 * 1024 - 16384)),
        ("E_371_is_0", 371, 0, None),
        ("F_371_is_128", 371, 128, None),
        ("G_325_is_2048", 325, 2048, None),
    ];
    for (name, offset, value, and_ring) in cells {
        for (scope, targets) in [
            ("every_slot", every_slot.clone()),
            ("device1_older_slot_only", vec![(DeviceIdentity(1), older)]),
            ("device1_newer_slot_only", vec![(DeviceIdentity(1), newer)]),
        ] {
            let mut image = base.clone();
            rewrite_field(&mut image, &targets, offset, value, and_ring);
            judge(&format!("{name} {scope}"), &pool.parameters, &image);
        }
    }
}

/// I-7.14：两块盘整块盘体对调（身份 0 的位置上装着盘 1 的整块盘面，反之亦然），池级 checker 报什么；core 的两种挂载怎么判。
/// 在第一个文件那一版与会话里覆盖写两次之后各试一次。
#[test]
fn swapped_device_bodies_on_the_pool_checker_and_both_mounts() {
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    for label in ["after_the_first_file", "after_two_overwrites"] {
        if label == "after_two_overwrites" {
            pool.overwrite(common_admission::OVERWRITE_BYTES).expect("覆盖写 1");
            pool.overwrite(common_admission::OVERWRITE_BYTES).expect("覆盖写 2");
        }
        let image = pool.image();
        let mut swapped = image.clone();
        let zero = image.devices[&DeviceIdentity(0)].clone();
        let one = image.devices[&DeviceIdentity(1)].clone();
        swapped.devices.insert(DeviceIdentity(0), one);
        swapped.devices.insert(DeviceIdentity(1), zero);
        judge(&format!("swapped_bodies {label}"), &pool.parameters, &swapped);
        judge(&format!("unswapped {label}"), &pool.parameters, &image);
    }
}
