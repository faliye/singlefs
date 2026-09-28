//! 调查（不入库）：第一个文件 17000 字节、内容第 16250 字节处摆一个码 2 节点头，落盘后在 32K 数据单元第二个 16K 槽开头。
//! 全部持久之后的 MemoryPool 上直接跑 check_pool_image，看 I-7.8 红不红，以及几个推翻它的对照。

mod common;

use singlefs_checker::image::{ImageReader, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::recovery::{readable_roots, recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
};
use singlefs_format::{DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET, NODE_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const FORGED_TREE_IDENTIFIER: u64 = 1 << 40;
const CONTENT_BYTES: usize = 17000;

struct Built {
    pool: MemoryPool,
    pool_from_device_images: MemoryPool,
    after_mkfs: MemoryPool,
    writes_after_mkfs: Vec<RetainedWrite>,
}

fn build(content: &[u8]) -> Built {
    let parameters = common::parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile { content, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件");
    }
    let operations = stream.retained_operations();
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
    pool.apply(&operations);
    let mut after_mkfs = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
    after_mkfs.apply(&operations[..mkfs_operation_count]);
    let (writes_after_mkfs, _segments) =
        writes_and_segments(&operations[mkfs_operation_count..], &common::geometry());
    let pool_from_device_images = common::memory_pool_of_sparse_devices(&devices);
    Built { pool, pool_from_device_images, after_mkfs, writes_after_mkfs }
}

fn plain_content() -> Vec<u8> {
    (0..CONTENT_BYTES).map(|index| u8::try_from(index % 251).expect("字节")).collect()
}

fn header_end_of(node: &[u8]) -> usize {
    86 + 2 * usize::from(node[51])
}

/// 从一次真的码 2 节点单元写里抄头，树 ID 改成 2^40、诞生代号改成 `birth`，头校验和重算（CRC32C，校验和字段 10..42 置零后算）。
fn forged_header(built: &Built, birth: u64) -> Vec<u8> {
    let node = built
        .writes_after_mkfs
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite && write.length_in_bytes() == NODE_BYTES)
        .filter_map(RetainedWrite::bytes)
        .find(|bytes| &bytes[..4] == b"SFSU" && bytes[6] == 2)
        .expect("有码 2 节点单元写")
        .to_vec();
    let header_end = header_end_of(&node);
    let mut header = node[..header_end].to_vec();
    header[42..50].copy_from_slice(&FORGED_TREE_IDENTIFIER.to_le_bytes());
    let birth_offset = 52 + 2 * usize::from(header[51]);
    header[birth_offset..birth_offset + 8].copy_from_slice(&birth.to_le_bytes());
    header[10..42].fill(0);
    let checksum = singlefs_checker::crc32_castagnoli_bitwise(&header);
    header[10..14].copy_from_slice(&checksum.to_le_bytes());
    header
}

fn content_with_header_at(header: &[u8], place: usize) -> Vec<u8> {
    let mut content = plain_content();
    content[place..place + header.len()].copy_from_slice(header);
    content
}

/// 内容第 16250 字节 = 数据单元第 134 + 16250 = 16384 字节 = 第二个 16K 槽开头。
fn place_at_second_slot() -> usize {
    usize::try_from(SLOT_BYTES - DATA_UNIT_PAYLOAD_OFFSET).expect("16250")
}

/// (盘, 数据单元起始槽)：32K 单元写里第二个 16K 槽开头就是这段头的那几次写。
fn data_units_carrying_at_second_slot(built: &Built, header: &[u8]) -> Vec<(u32, u64)> {
    let slot = usize::try_from(SLOT_BYTES).expect("16384");
    built
        .writes_after_mkfs
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite && write.length_in_bytes() == DATA_UNIT_BYTES)
        .filter(|write| write.bytes().is_some_and(|bytes| &bytes[slot..slot + header.len()] == header))
        .map(|write| (write.device.0, write.offset.0 / SLOT_BYTES))
        .collect()
}

fn not_holding(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image(reader)
        .into_iter()
        .filter(|(_, verdict)| *verdict != InvariantVerdict::Holds)
        .collect()
}

fn i78(reader: &dyn ImageReader) -> InvariantVerdict {
    check_pool_image(reader)
        .into_iter()
        .find(|(name, _)| *name == "I-7.8")
        .expect("I-7.8 每次都报")
        .1
}

fn watermark_by_core(pool: &MemoryPool) -> (u64, u64) {
    let parameters = common::parameters();
    let roots = readable_roots(
        pool,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    (
        roots.iter().map(|root| root.tree_identifier_watermark).max().expect("有根"),
        roots.iter().map(|root| root.checkpoint_txg.0).max().expect("有根"),
    )
}

/// 不经 checker 的扫描：每一次单元写的开头才是单元头；取其中码 2 头的树 ID 最大值。
fn highest_tree_identifier_at_unit_write_starts(built: &Built) -> u64 {
    built
        .writes_after_mkfs
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .filter_map(RetainedWrite::bytes)
        .filter(|bytes| &bytes[..4] == b"SFSU" && bytes[6] == 2)
        .map(|bytes| u64::from_le_bytes(bytes[42..50].try_into().expect("8")))
        .max()
        .expect("有码 2 单元写")
}

struct Filtered<'image> {
    image: &'image MemoryPool,
    removed: Vec<(u32, u64)>,
    full_scan: bool,
}

impl ImageReader for Filtered<'_> {
    fn devices(&self) -> Vec<u32> {
        ImageReader::devices(self.image)
    }
    fn device_bytes(&self, device: u32) -> Option<u64> {
        ImageReader::device_bytes(self.image, device)
    }
    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        ImageReader::read(self.image, device, offset, length)
    }
    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        if self.full_scan {
            return None;
        }
        let slots = ImageReader::candidate_unit_slots(self.image, device)?;
        Some(slots.into_iter().filter(|slot| !self.removed.contains(&(device, *slot))).collect())
    }
    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        ImageReader::candidate_journal_slots(self.image, device)
    }
}

#[test]
fn a_node_header_in_the_second_slot_of_a_data_unit_turns_i78_red_on_the_fully_persisted_memory_pool() {
    let probe = build(&plain_content());
    let header = forged_header(&probe, 1);
    let content = content_with_header_at(&header, place_at_second_slot());
    let built = build(&content);
    let carriers = data_units_carrying_at_second_slot(&built, &header);
    println!("name=carriers header_bytes={} data_units_device_start_slot={carriers:?}", header.len());
    assert!(!carriers.is_empty(), "头真的落在数据单元第二个槽开头");
    let recovered = recover(&built.pool, JournalPolicy::Consult);
    let content_matches = matches!(&recovered.outcome, RecoveryOutcome::FileRead { content: read, .. } if *read == content);
    println!("name=recover content_matches={content_matches} outcome_is_file_read={}", matches!(recovered.outcome, RecoveryOutcome::FileRead { .. }));
    assert!(content_matches, "恢复读回的就是写进去的 17000 字节");
    let (watermark, newest_txg) = watermark_by_core(&built.pool);
    let highest_at_unit_starts = highest_tree_identifier_at_unit_write_starts(&built);
    println!("name=definition watermark_by_core={watermark} newest_txg_by_core={newest_txg} highest_tree_id_at_unit_write_starts={highest_at_unit_starts} forged={FORGED_TREE_IDENTIFIER}");
    let mut second_slots_in_candidates = Vec::new();
    for (device, start) in &carriers {
        let candidates = ImageReader::candidate_unit_slots(&built.pool, *device).expect("候选");
        second_slots_in_candidates.push((*device, start + 1, candidates.contains(&(start + 1))));
    }
    println!("name=memory_pool_candidates second_slot_listed={second_slots_in_candidates:?}");
    let red = not_holding(&built.pool);
    println!("name=final_memory_pool not_holding={red:?}");
    let red_from_devices = not_holding(&built.pool_from_device_images);
    println!("name=final_pool_from_device_images not_holding={red_from_devices:?}");
    assert!(matches!(i78(&built.pool), InvariantVerdict::Violated(_)));
}

#[test]
fn control_same_history_without_the_forged_header_keeps_i78_holding() {
    let built = build(&plain_content());
    println!("name=control i78={:?} not_holding={:?}", i78(&built.pool), not_holding(&built.pool));
}

#[test]
fn control_header_shifted_512_bytes_off_the_slot_start_keeps_i78_holding() {
    let probe = build(&plain_content());
    let header = forged_header(&probe, 1);
    let built = build(&content_with_header_at(&header, place_at_second_slot() + 512));
    println!("name=shifted_512 i78={:?}", i78(&built.pool));
}

#[test]
fn control_header_with_a_birth_txg_above_every_published_root_keeps_i78_holding() {
    let probe = build(&plain_content());
    let (_, newest_txg) = watermark_by_core(&probe.pool);
    let header = forged_header(&probe, newest_txg + 1);
    let built = build(&content_with_header_at(&header, place_at_second_slot()));
    println!("name=birth_above_newest birth={} carriers={:?} i78={:?}", newest_txg + 1, data_units_carrying_at_second_slot(&built, &header), i78(&built.pool));
}

#[test]
fn control_header_with_a_broken_checksum_keeps_i78_holding() {
    let probe = build(&plain_content());
    let mut header = forged_header(&probe, 1);
    header[10] ^= 1;
    let built = build(&content_with_header_at(&header, place_at_second_slot()));
    println!("name=broken_checksum carriers={:?} i78={:?}", data_units_carrying_at_second_slot(&built, &header), i78(&built.pool));
}

#[test]
fn removing_only_the_second_slots_from_the_candidates_turns_i78_back_to_holding_and_a_full_scan_keeps_it_red() {
    let probe = build(&plain_content());
    let header = forged_header(&probe, 1);
    let built = build(&content_with_header_at(&header, place_at_second_slot()));
    let second_slots: Vec<(u32, u64)> = data_units_carrying_at_second_slot(&built, &header)
        .into_iter()
        .map(|(device, start)| (device, start + 1))
        .collect();
    let without = Filtered { image: &built.pool, removed: second_slots.clone(), full_scan: false };
    println!("name=without_second_slots removed={second_slots:?} i78={:?}", i78(&without));
    let full = Filtered { image: &built.pool, removed: Vec::new(), full_scan: true };
    println!("name=full_scan_none_candidates i78={:?}", i78(&full));
}

#[test]
fn overlay_of_every_write_on_the_mkfs_base_keeps_i78_holding() {
    let probe = build(&plain_content());
    let header = forged_header(&probe, 1);
    let built = build(&content_with_header_at(&header, place_at_second_slot()));
    let overlay = CrashImage {
        base: &built.after_mkfs,
        writes: &built.writes_after_mkfs,
        persisted: vec![true; built.writes_after_mkfs.len()],
    };
    let mut second_slot_listed = Vec::new();
    for (device, start) in data_units_carrying_at_second_slot(&built, &header) {
        let candidates = ImageReader::candidate_unit_slots(&overlay, device).expect("候选");
        second_slot_listed.push((device, start + 1, candidates.contains(&(start + 1))));
    }
    println!("name=overlay second_slot_listed={second_slot_listed:?} i78={:?}", i78(&overlay));
}
