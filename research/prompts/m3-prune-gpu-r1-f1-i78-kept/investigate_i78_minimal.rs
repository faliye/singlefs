//! 最小复现（不入库）：健康池（第一个文件 17000 字节，内容第 16250 字节起是一个码 2 节点头的字节）全部持久之后，
//! I-7.8 应当成立——数据单元内部不是单元头。今天的池级 checker 判它违反。

mod common;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter};
use singlefs_format::{DATA_UNIT_PAYLOAD_OFFSET, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

fn first_file_pool(content: &[u8]) -> Vec<RetainedOperation> {
    let parameters = common::parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = (0..2u32)
        .map(|n| {
            let device = SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            (DeviceIdentity(n), RecordingBlockDevice::with_shared_stream(DeviceIdentity(n), device, stream.clone()))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let mut writer = PoolWriter::new(&parameters, &mut devices);
    let instance = acquire_instance(&mut writer).expect("取号");
    let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let file = FirstFile { content, write_time_seconds: common::FIXED_WRITE_TIME_SECONDS };
    publish_first_file(&mut writer, &mut allocator, warm.roots.last().expect("根"), file, instance, &warm.last_record_bytes)
        .expect("新池新建文件");
    stream.retained_operations()
}

fn memory_pool(operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
    pool.apply(operations);
    pool
}

#[test]
fn user_content_shaped_like_a_node_header_at_the_second_slot_of_a_data_unit_is_not_a_unit_header() {
    let mut content: Vec<u8> = (0..17000usize).map(|index| u8::try_from(index % 251).expect("字节")).collect();
    // 抄本池一个真的码 2 节点头（第一次写出来的那个），树 ID 改 2^40、诞生代号改 1、CRC32C 头校验和重算。
    let probe = first_file_pool(&content);
    let mut header = probe
        .iter()
        .filter_map(|operation| operation.contents.as_deref())
        .find(|bytes| bytes.len() == 16384 && &bytes[..4] == b"SFSU" && bytes[6] == 2)
        .expect("码 2 节点")
        .to_vec();
    let key_span = 2 * usize::from(header[51]);
    header.truncate(86 + key_span);
    header[42..50].copy_from_slice(&(1u64 << 40).to_le_bytes());
    header[52 + key_span..60 + key_span].copy_from_slice(&1u64.to_le_bytes());
    header[10..42].fill(0);
    let checksum = singlefs_checker::crc32_castagnoli_bitwise(&header);
    header[10..14].copy_from_slice(&checksum.to_le_bytes());
    let place = usize::try_from(SLOT_BYTES - DATA_UNIT_PAYLOAD_OFFSET).expect("16250");
    content[place..place + header.len()].copy_from_slice(&header);

    let pool = memory_pool(&first_file_pool(&content));
    assert!(
        matches!(recover(&pool, JournalPolicy::Consult).outcome, RecoveryOutcome::FileRead { content: ref read, .. } if *read == content),
        "健康池：恢复读回写进去的内容"
    );
    let verdict = check_pool_image(&pool).into_iter().find(|(name, _)| *name == "I-7.8").expect("I-7.8").1;
    assert_eq!(verdict, InvariantVerdict::Holds);
}
