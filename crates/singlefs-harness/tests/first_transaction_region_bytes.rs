//! E142（第一个事务的干跑） 量 5 的实装一侧（`first_transaction_regions` 与 `first_transaction_region_bytes`）的验收：
//! ① 区域清单就是登记那 21 行，数目与顺序都对，而且与第一个事务真正发出的 21 条写逐条配得上；
//! ② 同一条路跑两次、结果行逐字节相同；
//! ③ 改动计数那 8 字节改一位，对应区域的 sha256 变、别的 20 行一个字符都不变。
//!
//! 期望值一律从 `.claude/kb/layout/01-first-txn.md` 零那一节的写清单抄成字面量（双份记账：
//! 代码里的常量表与这里的期望表分别抄一遍登记，抄错一处就对不上），不从被测的常量表反推。

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_format::{FIRST_TRANSACTION_TXG, ROOT_RING_REGION_DEVICES, TEST_IMAGE_DEFAULT_BYTES};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice, SECTOR_BYTES};
use singlefs_harness::first_transaction_regions::{
    region_result_lines, region_table_against_writes, FirstTransactionRegion, HexadecimalExtent,
    FIRST_TRANSACTION_REGIONS, FIRST_TRANSACTION_REGION_COUNT,
};
use singlefs_harness::scenario::{e142_parameters, run_first_transaction, ScenarioPoint};
use singlefs_harness::{RecordedOperation, RecordingBlockDevice, SharedStream};

/// 改动计数那 8 字节的盘上绝对偏移：inode 叶落在槽 50242，记录区从单元内 136 起，记录内偏移 88
/// ⇒ 50242 × 16384 + 136 + 88（跑前登记第七节乙类那一行的 823165152）。
const CHANGE_COUNT_FIELD_OFFSET: u64 = 50242 * 16384 + 136 + 88;
const CHANGE_COUNT_FIELD_BYTES: usize = 8;

struct FirstTransactionImage {
    image: MemoryPool,
    /// 录制流里第一个事务那一段（暖机之后的每一步，屏障在内）。
    first_transaction_operations: Vec<RecordedOperation>,
}

/// 与 `first_transaction_region_bytes` 那个 bin 同一条路、同参数：两块 4 GiB 内存盘上 mkfs → 取号 → 暖机 → 第一个事务。
fn run_on_memory_devices() -> FirstTransactionImage {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(TEST_IMAGE_DEFAULT_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let mut steps_before_first_transaction: Option<usize> = None;
    run_first_transaction(
        &parameters,
        &mut devices,
        &stream,
        |point, _devices| match point {
            ScenarioPoint::AfterInstanceAcquisition => {}
            ScenarioPoint::BeforeFirstTransaction => {
                steps_before_first_transaction = Some(stream.operations().len());
            }
        },
    )
    .expect("整条路在内存盘上跑得通");
    let steps_before_first_transaction =
        steps_before_first_transaction.expect("暖机之后那个口子一定被叫到");
    let operations = stream.operations();
    FirstTransactionImage {
        image: MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.inner().image.clone()))
                .collect(),
            device_size_in_bytes: TEST_IMAGE_DEFAULT_BYTES,
        },
        first_transaction_operations: operations[steps_before_first_transaction..].to_vec(),
    }
}

/// 翻一位（不是整字节）：读出那个扇区、异或最低位、写回去。
fn flip_lowest_bit(image: &mut MemoryPool, device: DeviceIdentity, offset: u64) {
    let sector_offset = DeviceOffsetInBytes(offset - offset % SECTOR_BYTES);
    let device_image = image.devices.get_mut(&device).expect("池里有这块盘");
    let mut sector = device_image.read(
        sector_offset,
        usize::try_from(SECTOR_BYTES).expect("扇区 512 字节"),
    );
    let index_in_sector = usize::try_from(offset % SECTOR_BYTES).expect("扇区内偏移");
    sector[index_in_sector] ^= 0x01;
    device_image.write(sector_offset, &sector);
}

fn read_bytes(image: &MemoryPool, device: DeviceIdentity, offset: u64, length: usize) -> Vec<u8> {
    let sector_offset = DeviceOffsetInBytes(offset - offset % SECTOR_BYTES);
    let index_in_sector = usize::try_from(offset % SECTOR_BYTES).expect("扇区内偏移");
    let sector = image.devices.get(&device).expect("池里有这块盘").read(
        sector_offset,
        usize::try_from(SECTOR_BYTES).expect("扇区 512 字节"),
    );
    sector[index_in_sector..index_in_sector + length].to_vec()
}

/// 登记那 21 行抄一遍：(区域名, 设备号, 盘上绝对偏移, 长度, 十六进制打法)。
/// 偏移列由「16 KiB 槽号 × 16384」算出（字节表零那一节的写清单），固定结构三样各自的来历写在行尾注释里。
/// 一行一个登记行，所以这个函数不让 rustfmt 拆行（拆开之后「21 行」这件事在源码里就看不出来了）。
#[rustfmt::skip]
fn registry_rows() -> Vec<(&'static str, u32, u64, u64, HexadecimalExtent)> {
    use HexadecimalExtent::{HeadAndTail, WholeRegion};
    vec![
        ("data_unit",       0, 50180 * 16384, 32768, HeadAndTail), // t1
        ("data_unit",       1, 50180 * 16384, 32768, HeadAndTail),
        ("extent_root",     0, 50240 * 16384, 16384, HeadAndTail), // t2
        ("extent_root",     1, 50240 * 16384, 16384, HeadAndTail),
        ("inode_leaf",      0, 50242 * 16384, 32768, HeadAndTail), // t3
        ("inode_leaf",      1, 50242 * 16384, 32768, HeadAndTail),
        ("inode_root",      0, 50244 * 16384, 16384, HeadAndTail), // t4
        ("inode_root",      1, 50244 * 16384, 16384, HeadAndTail),
        ("allocation_root", 0, 50245 * 16384, 16384, HeadAndTail), // t5
        ("allocation_root", 1, 50245 * 16384, 16384, HeadAndTail),
        ("accounting_root", 0, 50246 * 16384, 16384, HeadAndTail), // t6
        ("accounting_root", 1, 50246 * 16384, 16384, HeadAndTail),
        ("mapping_root",    0, 50247 * 16384, 16384, HeadAndTail), // t7
        ("mapping_root",    1, 50247 * 16384, 16384, HeadAndTail),
        ("tree_table",      0, 50248 * 16384, 16384, HeadAndTail), // t8
        ("tree_table",      1, 50248 * 16384, 16384, HeadAndTail),
        // t10：第 3 代根记录，根环区域 0（起点 1 MiB）的槽 1（槽距 4096），槽宽 = physical_block_size 512
        ("root_record",     0, 1024 * 1024 + 4096, 512, WholeRegion),
        // t9：jsn (1, 3) 那条记录，journal 环从槽 1024（16 MiB）起，环内偏移 8192（两次暖机各占一条 4096）
        ("journal_record",  0, 16 * 1024 * 1024 + 8192, 4096, WholeRegion),
        ("journal_record",  1, 16 * 1024 * 1024 + 8192, 4096, WholeRegion),
        // t11：超级块槽 1（世代号 5、tail = 3），槽距 4096、槽宽 4096
        ("superblock",      0, 4096, 4096, WholeRegion),
        ("superblock",      1, 4096, 4096, WholeRegion),
    ]
}

fn describe(region: &FirstTransactionRegion) -> (&'static str, u32, u64, u64, HexadecimalExtent) {
    (
        region.name,
        region.device.0,
        region.offset.0,
        region.length_in_bytes,
        region.hexadecimal_extent,
    )
}

#[test]
fn the_region_table_lists_the_twenty_one_registered_regions_in_order() {
    let expected = registry_rows();
    assert_eq!(
        expected.len(),
        FIRST_TRANSACTION_REGION_COUNT,
        "登记第一节第 3 条写死 21 行：8 个单元 × 2 盘 + 根槽 1 + journal 记录 × 2 盘 + 超级块槽 × 2 盘"
    );
    assert_eq!(
        FIRST_TRANSACTION_REGIONS.len(),
        FIRST_TRANSACTION_REGION_COUNT
    );
    let actual: Vec<(&'static str, u32, u64, u64, HexadecimalExtent)> =
        FIRST_TRANSACTION_REGIONS.iter().map(describe).collect();
    assert_eq!(
        actual, expected,
        "区域清单要与登记那 21 行逐行相同，顺序也相同"
    );
    let whole_region_rows = FIRST_TRANSACTION_REGIONS
        .iter()
        .filter(|region| region.hexadecimal_extent == HexadecimalExtent::WholeRegion)
        .count();
    assert_eq!(
        whole_region_rows, 5,
        "整段十六进制照打的只有固定结构那 5 行：根槽 512、journal 记录 4096 两份、超级块槽 4096 两份"
    );
}

#[test]
fn the_region_table_matches_the_writes_the_first_transaction_really_issues() {
    let run = run_on_memory_devices();
    let against_writes = region_table_against_writes(&run.first_transaction_operations);
    assert_eq!(
        against_writes.regions_without_a_write,
        Vec::<&str>::new(),
        "表里每一行都要有一条写落在它上面"
    );
    assert_eq!(
        against_writes.writes_outside_the_table,
        Vec::<(DeviceIdentity, DeviceOffsetInBytes, u64)>::new(),
        "第一个事务发出的每一条写都要在表里"
    );
    assert_eq!(
        against_writes.write_calls, FIRST_TRANSACTION_REGION_COUNT,
        "字节表零那一节：事务本身 21 条写请求"
    );
    assert!(against_writes.matches());

    // 根槽那一行不靠表自己说了算：区域与槽号由 root_ring 按 txg 算，落哪块盘由 mkfs 的 region_devices 定。
    let root_target = target_for_publish(CheckpointTxg(FIRST_TRANSACTION_TXG));
    let root_row = FIRST_TRANSACTION_REGIONS
        .iter()
        .find(|region| region.name == "root_record")
        .expect("表里有根槽那一行");
    assert_eq!(root_row.offset, slot_offset(root_target, 4096));
    assert_eq!(
        root_row.device,
        DeviceIdentity(
            ROOT_RING_REGION_DEVICES[usize::try_from(root_target.region).expect("区域号")]
        )
    );
}

#[test]
fn running_the_same_pipeline_twice_gives_byte_identical_result_lines() {
    let first_run = run_on_memory_devices();
    let second_run = run_on_memory_devices();
    let first_lines = region_result_lines(&first_run.image);
    assert_eq!(first_lines.len(), FIRST_TRANSACTION_REGION_COUNT);
    assert_eq!(
        first_lines,
        region_result_lines(&first_run.image),
        "同一个镜像打两次，结果行逐字节相同"
    );
    assert_eq!(
        first_lines,
        region_result_lines(&second_run.image),
        "整条路跑两次（不取系统时钟、不取随机数），21 行结果行逐字节相同"
    );
}

#[test]
fn flipping_one_bit_of_the_change_count_moves_only_that_regions_digest() {
    let run = run_on_memory_devices();
    let before = region_result_lines(&run.image);
    assert_eq!(
        read_bytes(
            &run.image,
            DeviceIdentity(0),
            CHANGE_COUNT_FIELD_OFFSET,
            CHANGE_COUNT_FIELD_BYTES
        ),
        FIRST_TRANSACTION_TXG.to_le_bytes(),
        "改动计数那 8 字节在 inode 叶记录区偏移 88（盘上绝对偏移 {CHANGE_COUNT_FIELD_OFFSET}），第一个事务写 checkpoint_txg"
    );

    let mut flipped_image = run.image.clone();
    flip_lowest_bit(
        &mut flipped_image,
        DeviceIdentity(0),
        CHANGE_COUNT_FIELD_OFFSET,
    );
    let after = region_result_lines(&flipped_image);

    let changed: Vec<usize> = before
        .iter()
        .zip(&after)
        .enumerate()
        .filter(|(_index, (old_line, new_line))| old_line != new_line)
        .map(|(index, _lines)| index)
        .collect();
    assert_eq!(
        changed,
        vec![4],
        "翻的是盘 0 的 inode 叶那一位：只有第 5 行（inode_leaf device=0）该变，别的 20 行一个字符都不许动"
    );
    let changed_region = &FIRST_TRANSACTION_REGIONS[4];
    assert_eq!(changed_region.name, "inode_leaf");
    assert_eq!(changed_region.device, DeviceIdentity(0));
    assert_ne!(
        sha256_field(&before[4]),
        sha256_field(&after[4]),
        "改动计数改一位，这个区域的 sha256 必须跟着变"
    );
}

/// 结果行里 `sha256=` 那一格。
fn sha256_field(line: &str) -> &str {
    line.split_whitespace()
        .find_map(|field| field.strip_prefix("sha256="))
        .expect("每一行结果行都有 sha256= 那一格")
}
