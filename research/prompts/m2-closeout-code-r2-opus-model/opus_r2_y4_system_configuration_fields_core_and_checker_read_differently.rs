//! 代码轮第二轮云端攻方腿（m2-closeout-code-r2）Y4：单元区起点、journal 环起点、根环基址这几样，core 与 checker 各从哪里取。
//! core 择系统配置时环起点、根环基址取编译期常量、单元区起点按环长现算并要求与偏移 417 相等且在 64 槽段边界上；
//! checker 从槽里偏移 325、371、417 读。这里把一份合法镜像四个系统配置槽里的某一个字段改掉、重算整槽校验和，
//! 看 core 的两种挂载与池级 checker 各怎么判。另看 mkfs 收不收环长不是物理块整数倍的几何。
//! 用例只打印，钉快照今天的行为。
mod common_admission;

use common_admission::{checker_violations_on, start_plain};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::history::HistoryDeviceWidth;

const SLOT: usize = 4096;

fn read_u64(slot: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(slot[offset..offset + 8].try_into().expect("8"))
}

/// 四个系统配置槽里偏移 `offset` 那 8 字节改成 `value`（`and_ring` 给了就同时改偏移 333 的环长），重算整槽校验和。交回改之前的值。
fn rewrite_field(image: &mut MemoryPool, spacing: u64, offset: usize, value: u64, and_ring: Option<u64>) -> u64 {
    let mut old = 0;
    for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
        for slot_offset in [0, spacing] {
            let sparse = image.devices.get_mut(&identity).expect("盘");
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            let mut slot = vec![0u8; SLOT];
            device.read_at(DeviceOffsetInBytes(slot_offset), &mut slot).expect("读槽");
            old = read_u64(&slot, offset);
            slot[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
            if let Some(ring) = and_ring {
                slot[333..341].copy_from_slice(&ring.to_le_bytes());
            }
            let digest = wide_checksum_with_field_zeroed(&slot, SLOT, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
            slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
            sparse.write(DeviceOffsetInBytes(slot_offset), &slot);
        }
    }
    old
}

fn judge(tag: &str, parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters, image: &MemoryPool) {
    match mount_read_only(image) {
        Ok(mounted) => println!("Y4 {tag} core_read_only ok chosen=({},{})", mounted.chosen_root.instance.0, mounted.chosen_root.checkpoint_txg.0),
        Err(error) => println!("Y4 {tag} core_read_only err={}", format!("{error:?}").chars().take(260).collect::<String>()),
    }
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect();
    let mut caller = parameters.clone();
    // 调用方参数照盘上改过的那几样给（环长），免得拒在「调用方参数与盘上不一致」那一判上。
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let _ = spacing;
    let mut slot = vec![0u8; SLOT];
    devices[0].1.read_at(DeviceOffsetInBytes(0), &mut slot).expect("读槽");
    caller.geometry.journal_ring_bytes = read_u64(&slot, 333);
    match mount_writable(&caller, &mut devices) {
        Ok(mounted) => println!("Y4 {tag} core_writable ok instance={}", mounted.output.instance.0),
        Err(error) => println!("Y4 {tag} core_writable err={}", format!("{error:?}").chars().take(260).collect::<String>()),
    }
    let violations = checker_violations_on(image);
    println!("Y4 {tag} checker_violations={} list={:?}", violations.len(), violations.iter().map(|line| line.chars().take(120).collect::<String>()).collect::<Vec<_>>());
}

#[test]
fn system_configuration_fields_read_by_the_checker_but_not_by_core() {
    let pool = start_plain(HistoryDeviceWidth::FourGibibytes);
    let base = pool.image();
    let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
    judge("baseline", &pool.parameters, &base);
    // A：偏移 417 的单元区起点改成现算起点 + 64（仍在段边界上、环末端仍不越过它）。
    let mut image = base.clone();
    let old = rewrite_field(&mut image, spacing, 417, 50176 + 64, None);
    println!("Y4 A unit_area_start {old} -> {}", 50176 + 64);
    judge("A_unit_area_start_plus_64", &pool.parameters, &image);
    // B：偏移 325 的 journal 环起点槽号 1024 改成 1025（环末端 1025×16384+768MiB 仍不越过 417 那个 50176？越过就 checker 拒）。
    let mut image = base.clone();
    let old = rewrite_field(&mut image, spacing, 325, 1023, None);
    println!("Y4 B journal_ring_start {old} -> 1023");
    judge("B_journal_ring_start_1023", &pool.parameters, &image);
    // C：偏移 371 的根环基址槽号改掉。
    let mut image = base.clone();
    let old = rewrite_field(&mut image, spacing, 371, 65, None);
    println!("Y4 C root_ring_base {old} -> 65");
    judge("C_root_ring_base_65", &pool.parameters, &image);
    // E：偏移 371 的根环基址槽号改成 0：槽 1 整槽不再落在同一槽自述的根环起点之前（D22 已定项 16 第 5 句的槽距判据）。
    let mut image = base.clone();
    let old = rewrite_field(&mut image, spacing, 371, 0, None);
    println!("Y4 E root_ring_base {old} -> 0");
    judge("E_root_ring_base_0", &pool.parameters, &image);
    // D：环长改成 768 MiB − 16 KiB，417 跟着改成它现算的那个槽 50175（不在 64 槽段边界上）。
    let mut image = base.clone();
    let ring = 768 * 1024 * 1024 - 16384;
    let old = rewrite_field(&mut image, spacing, 417, 1024 + ring / 16384, Some(ring));
    println!("Y4 D ring {ring} unit_area_start {old} -> {}", 1024 + ring / 16384);
    judge("D_ring_off_the_segment_boundary", &pool.parameters, &image);
}

/// mkfs 收不收环长不是 512 / 4096 整数倍的几何：收了的话写到一半报错，还是写之前拒。
#[test]
fn make_filesystem_with_a_ring_length_that_is_not_a_multiple_of_the_block_size() {
    for ring in [1024 * 1024 - 100, 1024 * 1024 - 512, 1024 * 1024 - 2048] {
        let mut parameters = HistoryDeviceWidth::FourGibibytes.parameters();
        parameters.geometry.journal_ring_bytes = ring;
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| (identity, SparseBlockDevice::new(HistoryDeviceWidth::FourGibibytes.device_bytes(), PhysicalBlockSizeInBytes(512))))
            .collect();
        let outcome = make_filesystem(&parameters, &mut devices);
        let written: Vec<usize> = devices.iter().map(|(_, device)| device.image.written_sectors_in(DeviceOffsetInBytes(0), 1 << 40).len()).collect();
        println!(
            "Y4 mkfs ring={ring} ok={} err={:?} written_sectors={written:?}",
            outcome.is_ok(),
            outcome.as_ref().err().map(|error| format!("{error:?}").chars().take(200).collect::<String>())
        );
    }
}
