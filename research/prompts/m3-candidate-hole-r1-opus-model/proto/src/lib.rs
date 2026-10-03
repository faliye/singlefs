//! m3-candidate-hole-r1 攻方原型共用件。建池、读故障与空发布的写法取自
//! `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/lib.rs`（参数加 journal 环长，判定改成在几组原型开关下各判一次）。
#![allow(clippy::missing_panics_doc, clippy::must_use_candidate)]

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::FixedGeometry;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

pub const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"M3HOLE-R1-OPUS01";
pub const IMAGE_BYTES: u64 = 256 << 20;
pub const WRITE_TIME: u64 = 1_788_000_000;

pub fn parameters(slots_per_region: u64, journal_ring_bytes: u64) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(slots_per_region)
                .expect("S 在 4..16"),
        },
    }
}

pub fn geometry(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

pub fn content(salt: u64) -> Vec<u8> {
    let salt = usize::try_from(salt % 251).expect("小");
    let length: usize = std::env::var("PROTO_CONTENT_BYTES").ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
    (0..length).map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256")).collect()
}

pub type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub fn pool_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

pub fn devices_of(pool: &MemoryPool) -> Devices {
    pool.devices
        .iter()
        .map(|(identity, image)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = image.clone();
            (*identity, device)
        })
        .collect()
}

pub fn recording_of(pool: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    devices_of(pool)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect()
}

/// mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写到 txg `last`（每次都落盘）。交回池与实例 1 的现行版本、分配器，供接着写。
pub fn build_plain(parameters: &MakeFilesystemParameters, last: u64) -> (Devices, singlefs_core::allocator::PoolAllocator, TransactionOutput) {
    let mut devices: Devices = (0..2u32)
        .map(|number| (DeviceIdentity(number), SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(parameters, &mut devices).expect("mkfs");
    let Mounted { output, mut allocator, current } = mount_writable(parameters, &mut devices).expect("可写挂载");
    let instance = output.instance;
    let mut current: TransactionOutput = {
        let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_first_file(&mut pool, &mut allocator, current.root(), FirstFile { content: &content(0)[..content(0).len().min(3000)], write_time_seconds: WRITE_TIME }, instance, current.record_bytes()).expect("第一个文件")
    };
    while current.root.checkpoint_txg.0 < last {
        current = overwrite(parameters, &mut devices, &mut allocator, &current);
    }
    (devices, allocator, current)
}

/// 在现行版本上覆盖写一次（内容按下一个 txg 取盐）。
pub fn overwrite<Device: singlefs_core::block_device::BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut singlefs_core::allocator::PoolAllocator,
    current: &TransactionOutput,
) -> TransactionOutput {
    let salt = current.root.checkpoint_txg.0 + 1;
    let mut pool = PoolWriter::new(parameters, devices);
    // PROTO_SEQ=1：走顺序写（内容可以多于一个数据单元，一次发布切成多个事务、多条记录；短 journal 环靠它转得动）。
    if std::env::var("PROTO_SEQ").is_ok_and(|v| v == "1") {
        return singlefs_core::transaction::publish_sequential_write(&mut pool, allocator, current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, current.root.instance).expect("顺序写");
    }
    publish_overwrite(&mut pool, allocator, current, FirstFile { content: &content(salt), write_time_seconds: WRITE_TIME + salt }, current.root.instance).expect("覆盖写")
}

/// 把 txg `txg` 那一槽根记录里第 40 个字节取反（这一槽自证不过；盘上别的字节不动）。
pub fn corrupt_root_slot(image: &mut MemoryPool, parameters: &MakeFilesystemParameters, txg: u64) {
    let target = target_for_publish(singlefs_core::address::CheckpointTxg(txg), parameters.geometry.root_ring_slots_per_region);
    let device = parameters.region_devices[usize::try_from(target.region).expect("区域")];
    let offset: DeviceOffsetInBytes = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
    image.flip_byte(device, offset, 40);
}

/// 一组原型开关：(HOLE_SKIP, HOLE_I31)。空串 = 今天。
pub const COMBOS: [(&str, &str); 12] = [
    ("rows", ""),
    ("rows", "e1ws"),
    ("rows", "rc"),
    ("rows", "rcw"),
    ("rows", "na"),
    ("rows", "e1"),
    ("rows", "e1w"),
    ("rows", "e1wu"),
    ("union", "na"),
    ("naive", "na"),
    ("counter", "na"),
    ("counter", "e1wu"),
];

/// 一个镜像在一组开关下的判定：红的不变量（各带前 140 字）、I-3.1 与 I-3.5 的判定、原型记下的洞表与区间。
pub fn judge_with(image: &MemoryPool, skip: &str, i31: &str) -> (Vec<String>, String, String, String) {
    std::env::set_var("HOLE_SKIP", skip);
    std::env::set_var("HOLE_I31", i31);
    let report = singlefs_checker::walk::check_pool_image(image);
    let note = singlefs_checker::walk::proto_take_hole_report();
    std::env::remove_var("HOLE_SKIP");
    std::env::remove_var("HOLE_I31");
    let short = |verdict: &singlefs_checker::image::InvariantVerdict| match verdict {
        singlefs_checker::image::InvariantVerdict::Violated(text) => format!("RED({})", text.chars().take(140).collect::<String>()),
        singlefs_checker::image::InvariantVerdict::Holds => "Holds".to_string(),
        singlefs_checker::image::InvariantVerdict::NotApplicable(reason) => format!("NA({})", reason.chars().take(40).collect::<String>()),
    };
    let reds: Vec<String> = report
        .iter()
        .filter(|(_, verdict)| matches!(verdict, singlefs_checker::image::InvariantVerdict::Violated(_)))
        .map(|(name, _)| (*name).to_string())
        .collect();
    let of = |name: &str| report.iter().find(|(n, _)| *n == name).map_or("missing".to_string(), |(_, v)| short(v));
    (reds, of("I-3.1"), of("I-3.5"), note)
}

/// 每组开关各判一次并打印；`tag` 进每一行。今天那一组另打印洞表。
pub fn show_all(image: &MemoryPool, tag: &str) {
    for (skip, i31) in COMBOS {
        let (reds, i31_verdict, i35_verdict, note) = judge_with(image, skip, i31);
        println!("JUDGE {tag} skip={skip} i31={i31:?} reds={reds:?} I-3.1={i31_verdict} I-3.5={i35_verdict}");
        if let Ok(name) = std::env::var("PROTO_SHOW") {
            if skip == "rows" && i31.is_empty() {
                std::env::set_var("HOLE_SKIP", skip);
                let shown = singlefs_checker::walk::check_pool_image(image).into_iter().find(|(n, _)| *n == name).map(|(_, v)| format!("{v:?}"));
                std::env::remove_var("HOLE_SKIP");
                let _ = singlefs_checker::walk::proto_take_hole_report();
                println!("  SHOW {tag} {name} {}", shown.unwrap_or_default().chars().take(400).collect::<String>());
            }
        }
        if std::env::var("PROTO_NOTES").is_ok_and(|v| v == "1") || (skip == "rows" && i31 == "e1w") {
            for line in note.lines() {
                println!("  NOTE {tag} skip={skip} i31={i31:?} {line}");
            }
        }
    }
}

/// 带文件的一版上的空发布（不改文件，只换根与记账）：取自 m3-root-ring-code-r2 攻方原型 lib.rs 的 `publish_empty`（同一个做法）。
pub fn publish_empty<Device: singlefs_core::block_device::BlockDevice>(
    pool: &mut PoolWriter<'_, Device>,
    allocator: &mut singlefs_core::allocator::PoolAllocator,
    previous: &TransactionOutput,
    instance: singlefs_core::address::InstanceGeneration,
) -> Result<TransactionOutput, singlefs_core::transaction::PublishError> {
    use singlefs_core::transaction::{publish_version, InstanceTablePlan, PublishPlan};
    publish_version(
        pool,
        allocator,
        PublishPlan {
            txg: singlefs_core::address::CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
            counter: previous.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: previous.highest_transaction_number_in_this_instance,
            instance,
            back_chain: singlefs_core::journal::back_chain_of(&previous.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}
