//! 里程碑「第二个事务」增补 1 第 1 件的验收：发布路径按结构种类报每次发布的写调用数与写字节。
//! 两条对照：一，每段窗口里按种类的合计与录制器在设备一层记下的写（写调用 = 一条写记录，字节 = 它的长度）逐项相等——录制器不知道写的是什么，
//! 漏计一种、多计一种都对不上；二，每次发布的合计与每一种都钉成字节表算好的绝对值（`.claude/kb/layout/01-first-txn.md`「零」的写清单：
//! 单元两盘各一份、journal 记录 4096 两盘各一份、根槽 512 一次 FUA、超级块槽 4096 两盘各一次），种类归错了合计不变、这一条红。
//! 两块内存盘（与宿主重跑虚机那条路同一种设备），不落文件。

use std::cell::Cell;
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemOutput, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    TransactionOutput, WarmUpOutput,
};
use singlefs_core::write_accounting::{
    WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::scenario::{e142_parameters, first_file_content, FIXED_WRITE_TIME_SECONDS};
use singlefs_harness::{
    RecordedOperation, RecordedOperationKind, RecordingBlockDevice, SharedStream,
};

const IMAGE_BYTES: u64 = 4 << 30;

/// 还许成功几次写：`None` = 不注入（一直放行），`Some(n)` = 再放行 n 次写，之后每次写都报错。
/// 装的是 `Rc<Cell<..>>`，测试拿着同一个句柄，写入口正拿着设备时也开得了、关得掉。
type RemainingSuccessfulWrites = Rc<Cell<Option<u64>>>;

/// 包在内存盘外面、录制器里面：按开关让写报错。放在录制器**里面**，失败的那次写就既不进录制流、也不进按种类的账
/// （录制器只记内层报成功的写，`RecordingBlockDevice::write_at`），两边口径仍相同。
struct WriteFailingDevice<Inner: BlockDevice> {
    inner: Inner,
    remaining_successful_writes: RemainingSuccessfulWrites,
}

impl<Inner: BlockDevice> BlockDevice for WriteFailingDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        match self.remaining_successful_writes.get() {
            None => {}
            Some(0) => {
                return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                    "注入的设备写错",
                )));
            }
            Some(remaining) => self.remaining_successful_writes.set(Some(remaining - 1)),
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

type MemoryDevice = RecordingBlockDevice<WriteFailingDevice<SparseBlockDevice>>;

fn calls_and_bytes(write_calls: u64, written_bytes: u64) -> WriteCallsAndBytes {
    WriteCallsAndBytes {
        write_calls,
        written_bytes,
    }
}

/// 录制器记下的一段操作里的写：每条写记录算一次写调用（普通写与 FUA 写都算），字节是记录的长度；屏障不算。
fn recorded_writes(operations: &[RecordedOperation]) -> WriteCallsAndBytes {
    operations
        .iter()
        .fold(WriteCallsAndBytes::NONE, |sum, operation| {
            match operation.kind {
                RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                    sum.plus(calls_and_bytes(1, operation.length))
                }
                RecordedOperationKind::Barrier => sum,
            }
        })
}

fn every_kind_matches(
    publish: &str,
    writes: &WritesByStructureKind,
    expected: &[(WrittenStructureKind, WriteCallsAndBytes)],
) {
    assert_eq!(
        expected.len(),
        WrittenStructureKind::IN_REPORT_ORDER.len(),
        "期望表每一种都列"
    );
    for (kind, expected_writes) in expected {
        assert_eq!(
            writes.of(*kind),
            *expected_writes,
            "{publish}：{kind:?} 的写调用与字节"
        );
    }
}

/// 固定结构三样：journal 记录两盘各一份 4096、根槽一次 FUA 512、超级块槽两盘各一次 4096（字节表零 t9..t11 / w1..w3）。
const JOURNAL_RECORD_ROOT_SLOT_SUPERBLOCK_SLOTS: [(WrittenStructureKind, WriteCallsAndBytes); 3] = [
    (
        WrittenStructureKind::JournalRecord,
        WriteCallsAndBytes {
            write_calls: 2,
            written_bytes: 8192,
        },
    ),
    (
        WrittenStructureKind::RootSlot,
        WriteCallsAndBytes {
            write_calls: 1,
            written_bytes: 512,
        },
    ),
    (
        WrittenStructureKind::SuperblockSlot,
        WriteCallsAndBytes {
            write_calls: 2,
            written_bytes: 8192,
        },
    ),
];

/// 一张期望表：列出的单元种类各两盘一份（给定单元宽度），其余单元种类为零，末尾接固定结构三样。
fn expected_by_kind(
    units_written_to_both_devices: &[(WrittenStructureKind, u64)],
) -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
    let unit_kinds = [
        WrittenStructureKind::DataUnit,
        WrittenStructureKind::ExtentTreeNode,
        WrittenStructureKind::InodeTreeLeafContainer,
        WrittenStructureKind::InodeTreeRoot,
        WrittenStructureKind::AllocationRecordTreeNode,
        WrittenStructureKind::AccountingTreeNode,
        WrittenStructureKind::CentralMappingTreeNode,
        WrittenStructureKind::TreeTableUnit,
        WrittenStructureKind::InstanceTableUnit,
    ];
    let mut expected: Vec<(WrittenStructureKind, WriteCallsAndBytes)> = unit_kinds
        .iter()
        .map(|kind| {
            let writes = units_written_to_both_devices
                .iter()
                .find(|(written_kind, _)| written_kind == kind)
                .map_or(WriteCallsAndBytes::NONE, |(_, unit_bytes)| {
                    calls_and_bytes(2, 2 * unit_bytes)
                });
            (*kind, writes)
        })
        .collect();
    expected.extend(JOURNAL_RECORD_ROOT_SLOT_SUPERBLOCK_SLOTS);
    expected
}

/// 带文件的一版（第一个事务、覆盖写）：字节表零 t1..t8，数据单元与 inode 树叶容器 32768、其余六个 16384。
fn file_version_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
    expected_by_kind(&[
        (WrittenStructureKind::DataUnit, 32768),
        (WrittenStructureKind::ExtentTreeNode, 16384),
        (WrittenStructureKind::InodeTreeLeafContainer, 32768),
        (WrittenStructureKind::InodeTreeRoot, 16384),
        (WrittenStructureKind::AllocationRecordTreeNode, 16384),
        (WrittenStructureKind::AccountingTreeNode, 16384),
        (WrittenStructureKind::CentralMappingTreeNode, 16384),
        (WrittenStructureKind::TreeTableUnit, 16384),
    ])
}

/// 第一次可写挂载的暖机（树表 0 条，零单元）：字节表零 w1..w3。
fn zero_unit_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
    expected_by_kind(&[])
}

/// 后续可写挂载写行那次发布：实例表单元 32768 + 分配记录、记账、映射、树表四个固定点单元各 16384（字节表八「第一次之后的可写挂载（写行）」）。
fn row_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
    expected_by_kind(&[
        (WrittenStructureKind::AllocationRecordTreeNode, 16384),
        (WrittenStructureKind::AccountingTreeNode, 16384),
        (WrittenStructureKind::CentralMappingTreeNode, 16384),
        (WrittenStructureKind::TreeTableUnit, 16384),
        (WrittenStructureKind::InstanceTableUnit, 32768),
    ])
}

/// 后续可写挂载的暖机空发布：四个固定点单元各 16384（字节表八「空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元）」）。
fn later_warm_up_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
    expected_by_kind(&[
        (WrittenStructureKind::AllocationRecordTreeNode, 16384),
        (WrittenStructureKind::AccountingTreeNode, 16384),
        (WrittenStructureKind::CentralMappingTreeNode, 16384),
        (WrittenStructureKind::TreeTableUnit, 16384),
    ])
}

/// 第二版的内容：与虚机二进制 `second-transaction` 模式的发布 B 相同（4100 字节）。
fn second_file_content() -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// mkfs → 取号 → 暖机 → 第一个事务 → 发布 B，同一个写入口；记下每一步开始时录制流有几条。
struct PublishedThroughOverwrite {
    devices: Vec<(DeviceIdentity, MemoryDevice)>,
    /// 与 `devices` 同序：每块盘的注入开关。
    write_faults: Vec<RemainingSuccessfulWrites>,
    stream: SharedStream,
    allocator: PoolAllocator,
    warm_up: WarmUpOutput,
    first_transaction: TransactionOutput,
    overwrite: TransactionOutput,
    warm_up_start: usize,
    first_transaction_start: usize,
    overwrite_start: usize,
    overwrite_end: usize,
}

fn publish_through_overwrite() -> PublishedThroughOverwrite {
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::new();
    let write_faults: Vec<RemainingSuccessfulWrites> =
        (0..2).map(|_| Rc::new(Cell::new(None))).collect();
    let mut devices: Vec<(DeviceIdentity, MemoryDevice)> = (0..2u32)
        .map(|device_number| {
            let identity = DeviceIdentity(device_number);
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    WriteFailingDevice {
                        inner: SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        remaining_successful_writes: write_faults
                            [usize::try_from(device_number).expect("设备号")]
                        .clone(),
                    },
                    stream.clone(),
                ),
            )
        })
        .collect();
    let genesis: MakeFilesystemOutput = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    assert_eq!(instance, InstanceGeneration(1));
    let warm_up_start = stream.operations().len();
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    let first_transaction_start = stream.operations().len();
    let first_transaction = publish_first_file(
        &mut writer,
        &mut allocator,
        &genesis.root,
        FirstFile {
            content: &first_file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    )
    .expect("第一个事务");
    let overwrite_start = stream.operations().len();
    let overwrite = publish_overwrite(
        &mut writer,
        &mut allocator,
        &first_transaction,
        FirstFile {
            content: &second_file_content(),
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("发布 B");
    let overwrite_end = stream.operations().len();
    drop(writer);
    PublishedThroughOverwrite {
        devices,
        write_faults,
        stream,
        allocator,
        warm_up: warmed,
        first_transaction,
        overwrite,
        warm_up_start,
        first_transaction_start,
        overwrite_start,
        overwrite_end,
    }
}

/// 验收第 1 条：暖机两次、第一个事务、发布 B 各自按种类的合计与录制器逐项相等；发布 B 与第一个事务都是 21 次写调用、344 576 字节，
/// 每次暖机 5 次、16 896 字节；每一种钉成字节表的数。
#[test]
fn first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table()
{
    let published = publish_through_overwrite();
    let operations = published.stream.operations();
    assert_eq!(published.warm_up.writes.len(), 2, "暖机两次空发布各一份账");

    assert_eq!(
        published.warm_up.writes[0]
            .total()
            .plus(published.warm_up.writes[1].total()),
        recorded_writes(&operations[published.warm_up_start..published.first_transaction_start]),
        "暖机两次按种类的合计 == 录制器在这段里记下的写"
    );
    assert_eq!(
        published.first_transaction.writes.total(),
        recorded_writes(&operations[published.first_transaction_start..published.overwrite_start]),
        "第一个事务按种类的合计 == 录制器在这段里记下的写"
    );
    assert_eq!(
        published.overwrite.writes.total(),
        recorded_writes(&operations[published.overwrite_start..published.overwrite_end]),
        "发布 B 按种类的合计 == 录制器在这段里记下的写"
    );

    for (warm_up_index, warm_up_writes) in published.warm_up.writes.iter().enumerate() {
        assert_eq!(
            warm_up_writes.total(),
            calls_and_bytes(5, 16_896),
            "第 {} 次暖机：记录 2 + 根槽 1 + 超级块槽 2",
            warm_up_index + 1
        );
    }
    assert_eq!(
        published.first_transaction.writes.total(),
        calls_and_bytes(21, 344_576),
        "第一个事务：单元 8 × 2 盘 = 16 次 327 680 字节 + 记录 2 次 8192 + 根槽 1 次 512 + 超级块槽 2 次 8192"
    );
    assert_eq!(
        published.overwrite.writes.total(),
        calls_and_bytes(21, 344_576),
        "发布 B 与第一个事务写同样八个角色：E152 量到的 344 576 字节、21 次写调用"
    );

    for (warm_up_index, warm_up_writes) in published.warm_up.writes.iter().enumerate() {
        every_kind_matches(
            &format!("第 {} 次暖机", warm_up_index + 1),
            warm_up_writes,
            &zero_unit_publish_by_kind(),
        );
    }
    every_kind_matches(
        "第一个事务",
        &published.first_transaction.writes,
        &file_version_publish_by_kind(),
    );
    every_kind_matches(
        "发布 B",
        &published.overwrite.writes,
        &file_version_publish_by_kind(),
    );
}

/// 验收第 1 条里「写行与暖机的空发布各自一份」：发布 B 之后可写挂载——取号 2 次超级块槽写不属于任何一次发布；写行那次发布 15 次、
/// 213 504 字节；之后两次暖机空发布各 13 次、147 968 字节；整段录制器记下的写 == 取号 + 三次发布的合计。
#[test]
fn writable_remount_row_publish_and_each_warm_up_publish_add_up_to_the_recorded_writes() {
    let mut published = publish_through_overwrite();
    let parameters = e142_parameters(512, 512);
    let mount_start = published.stream.operations().len();
    let mounted = mount_writable(&parameters, &mut published.devices).expect("可写挂载");
    let operations = published.stream.operations();
    let row_publish = mounted
        .output
        .row_publish
        .file_version()
        .expect("B 带文件：写行发布是带文件的一版");
    let warm_up_publishes: Vec<&TransactionOutput> = mounted
        .output
        .warm_up_publishes
        .iter()
        .map(|version| {
            version
                .file_version()
                .expect("带文件的一版之后的暖机仍是带文件的一版")
        })
        .collect();
    let acquisition = calls_and_bytes(2, 8192);

    assert_eq!(
        warm_up_publishes.iter().fold(
            acquisition.plus(row_publish.writes.total()),
            |sum, warm_up_publish| sum.plus(warm_up_publish.writes.total())
        ),
        recorded_writes(&operations[mount_start..]),
        "取号两盘各一次超级块槽 + 写行发布 + 各次暖机按种类的合计 == 录制器在挂载这段里记下的写"
    );

    assert_eq!(
        warm_up_publishes.len(),
        2,
        "txg 6 落盘 0、txg 7 落盘 1：暖机两次"
    );
    assert_eq!(
        row_publish.writes.total(),
        calls_and_bytes(15, 213_504),
        "写行：实例表单元 + 四个固定点单元 × 2 盘 = 10 次 196 608 字节 + 记录、根槽、超级块槽 5 次 16 896"
    );
    for (warm_up_index, warm_up_publish) in warm_up_publishes.iter().enumerate() {
        assert_eq!(
            warm_up_publish.writes.total(),
            calls_and_bytes(13, 147_968),
            "第 {} 次暖机：四个固定点单元 × 2 盘 = 8 次 131 072 字节 + 5 次 16 896",
            warm_up_index + 1
        );
    }

    every_kind_matches("写行发布", &row_publish.writes, &row_publish_by_kind());
    for (warm_up_index, warm_up_publish) in warm_up_publishes.iter().enumerate() {
        every_kind_matches(
            &format!("挂载之后第 {} 次暖机", warm_up_index + 1),
            &warm_up_publish.writes,
            &later_warm_up_publish_by_kind(),
        );
    }
}

/// 中途失败的那次发布 C 已记的写：单元按 bump 次序两盘各一份，写到第六个单元（记账树节点）的盘 1 那一次时报错 ⇒
/// 前五个单元两盘各一份、记账树节点只有盘 0 那一份，映射树、树表与固定结构三样一次都没走到。十二种全列。
fn failed_midway_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
    vec![
        (WrittenStructureKind::DataUnit, calls_and_bytes(2, 65_536)),
        (
            WrittenStructureKind::ExtentTreeNode,
            calls_and_bytes(2, 32_768),
        ),
        (
            WrittenStructureKind::InodeTreeLeafContainer,
            calls_and_bytes(2, 65_536),
        ),
        (
            WrittenStructureKind::InodeTreeRoot,
            calls_and_bytes(2, 32_768),
        ),
        (
            WrittenStructureKind::AllocationRecordTreeNode,
            calls_and_bytes(2, 32_768),
        ),
        (
            WrittenStructureKind::AccountingTreeNode,
            calls_and_bytes(1, 16_384),
        ),
        (
            WrittenStructureKind::CentralMappingTreeNode,
            WriteCallsAndBytes::NONE,
        ),
        (
            WrittenStructureKind::TreeTableUnit,
            WriteCallsAndBytes::NONE,
        ),
        (
            WrittenStructureKind::InstanceTableUnit,
            WriteCallsAndBytes::NONE,
        ),
        (
            WrittenStructureKind::JournalRecord,
            WriteCallsAndBytes::NONE,
        ),
        (WrittenStructureKind::RootSlot, WriteCallsAndBytes::NONE),
        (
            WrittenStructureKind::SuperblockSlot,
            WriteCallsAndBytes::NONE,
        ),
    ]
}

/// 第三版的内容（发布 C）：中途失败那次与重试那次写同一份内容、同一个 txg。
fn third_file_content() -> Vec<u8> {
    (0..3700usize)
        .map(|index| u8::try_from((index * 11 + 5) % 251).expect("小于 256"))
        .collect()
}

/// 增补 1 验收加的那一条（增补 2 第 20b 行，代码三方第一轮打中，判决第二节第 2 行）：一次发布中途设备写报错、调用方重试成功之后，
/// 按种类的合计与设备一层记的仍对得上。
///
/// 一次发布的账是发布前后两次快照之差、两次都在成功路径上取 ⇒ 中途任一步返回时已落盘的写不属于任何一次发布的账；
/// 不把它交出去，这一段窗口里设备一层数到的写就比按种类的合计多（攻方探针：按种类 21 次 / 344 576，设备 25 次 / 442 880）。
/// 这里让盘 1 的第 6 次写报错（发布 C 写第六个单元时）：失败那次已记 11 次写、245 760 字节（数据单元、extent 树根、inode 树叶容器、
/// inode 树根、分配记录树节点两盘各一份 + 记账树节点盘 0 那一份），重试整整 21 次 / 344 576 字节，两份相加正好是录制器在这段里记下的。
#[test]
fn publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up() {
    let mut published = publish_through_overwrite();
    let parameters = e142_parameters(512, 512);
    let window_start = published.stream.operations().len();
    let content = third_file_content();
    // 盘 1 再放行 5 次写：发布 C 的第六个单元写到盘 1 时报错（单元按 bump 次序写，每个单元两盘各一次）。
    published.write_faults[1].set(Some(5));
    let (failed_publishes, retry) = {
        let mut writer = PoolWriter::new(&parameters, published.devices.as_mut_slice());
        let failure = publish_overwrite(
            &mut writer,
            &mut published.allocator,
            &published.overwrite,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            InstanceGeneration(1),
        )
        .expect_err("盘 1 的第 6 次写报错，发布必须失败");
        assert!(
            matches!(
                failure,
                singlefs_core::transaction::PublishError::BlockDevice(_)
            ),
            "中途失败的是设备写：{failure:?}"
        );
        published.write_faults[1].set(None);
        let retry = publish_overwrite(
            &mut writer,
            &mut published.allocator,
            &published.overwrite,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            InstanceGeneration(1),
        )
        .expect("盘好了，拿同一个上一版重试");
        (writer.writes_of_failed_publishes().to_vec(), retry)
    };
    let operations = published.stream.operations();

    assert_eq!(failed_publishes.len(), 1, "中途失败过一次，交出一份账");
    assert_eq!(
        failed_publishes[0].total(),
        calls_and_bytes(11, 245_760),
        "失败那次已记的写：五个单元两盘各一份 + 第六个单元盘 0 那一份"
    );
    every_kind_matches(
        "中途失败的发布 C",
        &failed_publishes[0],
        &failed_midway_publish_by_kind(),
    );
    assert_eq!(
        retry.writes.total(),
        calls_and_bytes(21, 344_576),
        "重试那次是完整的一次发布"
    );
    every_kind_matches(
        "重试的发布 C",
        &retry.writes,
        &file_version_publish_by_kind(),
    );
    assert_eq!(
        failed_publishes[0].total().plus(retry.writes.total()),
        recorded_writes(&operations[window_start..]),
        "失败那次已记的写 + 重试那次的账 == 录制器在这段窗口里记下的写"
    );
    assert_eq!(
        recorded_writes(&operations[window_start..]),
        calls_and_bytes(32, 590_336),
        "设备一层：11 + 21 次、245 760 + 344 576 字节"
    );
}
