//! 里程碑「新池新建文件」步 3 / 步 4 / 步 5 的验收：mkfs → 取号 → 暖机两次 → 新池新建文件，落到两个文件镜像上，
//! 录制流按路径切段与 E142（新池新建文件的干跑） 第九次跑的产物 `research/results/e142-new-pool-file-creation-dry-run-2026-09-16-instance-boundary.out`
//! 逐字对（`name=segments` 五行、`name=root_record` 的反向链、`name=accounting` 的数），盘上的每个单元都由 checker 另一份解析判过。

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};

use singlefs_checker::position_addressed::{
    allocation_record_tree_cell_range, extent_upper_cell_range, AllocationRecordTreeCell,
};
use singlefs_checker::{
    back_chain_of_record_header, check_index_node_keys, check_internal_node_separators,
    check_journal_record, check_root_slot, check_system_configuration_slot, check_unit,
    choose_system_configuration, crc32_castagnoli_bitwise, index_node_view,
    key_schema_for_tree_kind, packed_unit_view, KeySchema, Verdict, KEY_SCHEMA_MAPPING,
    KEY_SCHEMA_TREE_TABLE,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    SlotNumber, TreeIdentifier,
};
use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, FileBackedBlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::bytes::ByteReader;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::journal::record_offset;
use singlefs_core::make_filesystem::{
    make_filesystem, MakeFilesystemOutput, MakeFilesystemParameters, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::pointer::{DataPointer, NodePointer};
use singlefs_core::records::{
    mapping_key_for_data, AccountingEntry, InodeRecord, TreeTableEntry, STATISTIC_ALLOCATED_BYTES,
    STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
    STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
    STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
    STATISTIC_UNRECLAIMABLE_BYTES,
};
use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
    TransactionUnit, WarmUpOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{
    JOURNAL_RING_DEFAULT_BYTES, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_EXTENT,
    TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
};
use singlefs_harness::segments::{
    closed_form_state_count, segment_kinds_text, segment_sizes_text, split_into_segments,
    FixedGeometry,
};
use singlefs_harness::temporary_files::TemporaryPathRemovedOnDrop;
use singlefs_harness::{RecordedOperation, RecordingBlockDevice, SharedStream};

static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
const IMAGE_BYTES: u64 = 4 << 30;
/// E142 的第一个文件 3000 字节（`name=config file_bytes=3000`）。
const FILE_BYTES: usize = 3000;
/// E142 装置里的固定 fsid（`FIXED_FSID`）：用同一个 fsid，暖机第二条记录头的 CRC 才能与产物 `name=root_record … back_chain=…` 那一行逐字对上。
const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// E142 装置里的固定写入时间（`FIXED_WRITE_TIME_SECONDS`）。
const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 新池新建文件那条记录的反向链 = CRC32C(暖机第二条记录的 496 字节头)。E142 产物第 37 行（`back_chain=628216162`）是 307 字节头下的值；
/// 记录头加本次发布内序号（D23（journal 的角色与格式） 已定项 4）之后是 1057457588（那一段独立换算：307 字节头下的三条记录在提交标记
/// 之后插 4 字节序号 1）。记录标志位 0（原「填充 1」那个字节，偏移 7，D23（journal 的角色与格式） 已定项 4 / 已定项 17）在
/// w1、w4、t9 上都写 1（每次只有一条记录的发布那一条也带「本次发布末条」标志）之后，这个值又由一段独立的换算得出：
/// 拿改之前的代码写出的三条记录（311 字节头、偏移 7 写 0），自己实现 CRC-32C（先过公开校验值），先复现出 1057457588 当阳性对照，
/// 再把 w1、w4 偏移 7 那一字节写 1、重算 w4 的反向链（= CRC32C(新 w1 头) = 699178990），再算新 w4 头的 CRC32C，得 3937788698
/// （换算脚本与原样输出在实二十的报告里）。E142 装置按新代码重跑之后，产物里那一行应当逐字是这个数。
/// 树 ID 整体挪到 65536 起（D8（核心索引结构） 已定项 11，用户 2026-10-01 定案）之后，暖机第二条记录头里新根段的树 ID 水位从 11 变 65536，
/// 钉值随之从 3937788698 换成 2213468031（那一批实现员的副本上同一段流程现算，没有独立换算；E142 装置按新号重跑之后要逐字对上）。
/// 新根段加实例表指针、分配记录树根指针与卸载记号三项（头 311 → 487，D23（journal 的角色与格式） 已定项 4 / 已定项 15，用户 2026-10-02
/// 「回到甲」）之后，暖机第二条记录头多出那 176 字节（实例表指针照 mkfs 那一片、分配记录树根全零、卸载记号 0），钉值从 2213468031 换成
/// 2660362816：实现员 2026-10-03 在副本上同一段流程现算；同一条用例里每条记录的反向链另由池级 checker 自己那份逐位 CRC-32C 对盘上那 487 字节头
/// 现算一遍再比（`back_chain_of_record_header`），两边对得上，不是只抄实现的数。E142 装置按新头宽重跑之后要逐字对上。
/// 新根段末尾加回收门槛 t 8 字节与钉槽 p 1 字节（C773（读坏与全零根槽的回收门槛钉只在内存里），头 487 → 496：暖机第二条记录带 t = 0、p = 255，
/// fsid 与 MAC 后挪 9 字节），钉值从 2660362816 换成 2864069229（C773 实现员副本上这条断言红出来的 left 值；同一条用例里 checker 那一份逐位 CRC-32C 照旧现算再比）。
const E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION: u32 = 2_864_069_229;

/// 一份镜像的路径，建之前就由守卫认领：搭池途中哪一步 panic、用例做完、断言失败都删掉它。
fn image_path(tag: &str, device: u32) -> TemporaryPathRemovedOnDrop {
    let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    TemporaryPathRemovedOnDrop::claiming(std::env::temp_dir().join(format!(
        "singlefs-step5-{tag}-{}-{sequence}-dev{device}.img",
        std::process::id()
    )))
}

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
    }
}

fn file_content() -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect()
}

type Recorded = RecordingBlockDevice<FileBackedBlockDevice>;

struct BuiltPool {
    /// 只为丢掉时删镜像而留着。
    _images: Vec<TemporaryPathRemovedOnDrop>,
    devices: Vec<(DeviceIdentity, Recorded)>,
    stream: SharedStream,
    genesis: MakeFilesystemOutput,
    warm_up: WarmUpOutput,
    output: TransactionOutput,
    mkfs_operation_count: usize,
    acquisition_operation_count: usize,
    warm_up_operation_count: usize,
}

/// 整条路：mkfs → 取号 → 暖机 → 新池新建文件；每一步之后记下录制流的长度，按路径切段用。
fn build_pool(tag: &str) -> BuiltPool {
    let parameters = parameters();
    let stream = SharedStream::new();
    let mut images = Vec::new();
    let mut devices: Vec<(DeviceIdentity, Recorded)> = Vec::new();
    for device_number in 0..2u32 {
        let path = image_path(tag, device_number);
        let file = FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            IMAGE_BYTES,
            PhysicalBlockSizeInBytes(512),
        )
        .expect("建镜像");
        devices.push((
            DeviceIdentity(device_number),
            RecordingBlockDevice::with_shared_stream(
                DeviceIdentity(device_number),
                file,
                stream.clone(),
            ),
        ));
        images.push(path);
    }
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_operation_count = stream.operations().len();
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
    // mkfs 同一个进程里接着写的会话装上 mkfs 刚写下的根环（没装的分配器记根就断言，C551）。
    allocator.install_root_ring_occupancy(
        singlefs_core::make_filesystem::root_ring_occupancy_after_make_filesystem(
            &parameters,
            &genesis,
        ),
    );
    let content = file_content();
    // 暖机两次发布之后录制流有几步：新池新建文件的写从这之后起（量出来，不按新池新建文件写几步倒推）。
    let warm_up_operation_count;
    let (warm_up, output) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        let acquisition_operation_count = stream.operations().len();
        assert_eq!(
            acquisition_operation_count,
            mkfs_operation_count + 8,
            "取号写两轮（甲'），每轮两写 + 那一轮之后那道屏障，两块盘各记一步（录制器按设备记屏障，代码审阅第 1 条；C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案；暖机开场那道因此不再发）"
        );
        let warm_up = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        warm_up_operation_count = stream.operations().len();
        let output = publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up.last_record_bytes,
        )
        .expect("新池新建文件");
        (warm_up, output)
    };
    BuiltPool {
        _images: images,
        devices,
        stream,
        genesis,
        warm_up,
        output,
        mkfs_operation_count,
        // 取号那一截切到第二轮那两写为止（第二轮之后那道屏障归暖机那一截的第一段，与只写一轮时切在取号两写之后同一个切法）。
        acquisition_operation_count: mkfs_operation_count + 6,
        warm_up_operation_count,
    }
}

fn read(device: &Recorded, offset: DeviceOffsetInBytes, length: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; length];
    device.read_at(offset, &mut buffer).expect("读");
    buffer
}

fn device(pool: &BuiltPool, identity: DeviceIdentity) -> &Recorded {
    &pool
        .devices
        .iter()
        .find(|(candidate, _)| *candidate == identity)
        .expect("有这块盘")
        .1
}

fn read_unit_from(
    pool: &BuiltPool,
    identity: DeviceIdentity,
    slot: SlotNumber,
    length: usize,
) -> Vec<u8> {
    read(device(pool, identity), slot.to_device_offset(), length)
}

fn read_root_slot(pool: &BuiltPool, region: u64, slot: u64) -> Vec<u8> {
    let region_device = parameters().region_devices[usize::try_from(region).expect("区域号")];
    read(
        device(pool, region_device),
        slot_offset(RootRingSlot { region, slot }, 4096),
        512,
    )
}

fn read_journal_record(pool: &BuiltPool, identity: DeviceIdentity, counter: u64) -> Vec<u8> {
    read(
        device(pool, identity),
        record_offset(counter, JOURNAL_RING_DEFAULT_BYTES),
        4096,
    )
}

fn unit_length(identity: TransactionUnit) -> usize {
    match identity {
        TransactionUnit::DataOfAFileOtherThanTheFirst { .. } => 32768,
        TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. } => 16384,
        TransactionUnit::Data(_)
        | TransactionUnit::InodeLeafContainer(_)
        | TransactionUnit::TombstoneLeafContainer(_)
        | TransactionUnit::LineageLeafContainer(_)
        | TransactionUnit::RetirementLeafContainer(_)
        | TransactionUnit::InstanceTable
        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => 32768,
        TransactionUnit::ExtentRoot
        | TransactionUnit::ExtentLowerNode(_)
        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
        | TransactionUnit::InodeRoot
        | TransactionUnit::AllocationTree
        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
        | TransactionUnit::AccountingTree
        | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
        | TransactionUnit::DeadlistTree
        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
        | TransactionUnit::MappingTree
        | TransactionUnit::DeletedInodesRoot
        | TransactionUnit::LoggedOperationsRoot
        | TransactionUnit::TombstoneRoot
        | TransactionUnit::TombstoneNodeBelowTheRoot(_)
        | TransactionUnit::LineageRoot
        | TransactionUnit::LineageNodeBelowTheRoot(_)
        | TransactionUnit::RetirementRoot
        | TransactionUnit::RetirementNodeBelowTheRoot(_)
        | TransactionUnit::TreeTable => 16384,
    }
}

fn tree_table_entries_from_disk(pool: &BuiltPool) -> Vec<TreeTableEntry> {
    let root_slot = read_root_slot(pool, 0, 1);
    let root =
        singlefs_core::root_record::RootRecord::parse_slot(&root_slot, &E142_FILESYSTEM_IDENTIFIER)
            .expect("第 3 代根");
    let tree_table_location = root.tree_table.locations[0];
    let tree_table_unit = read_unit_from(
        pool,
        tree_table_location.device,
        tree_table_location.slot,
        16384,
    );
    assert_eq!(
        crc32_castagnoli_bitwise(&tree_table_unit),
        tree_table_location.unit_checksum,
        "根记录里的树表指针校验和"
    );
    let view = index_node_view(&tree_table_unit).expect("树表单元");
    view.entries
        .iter()
        .map(|bytes| TreeTableEntry::parse(bytes).expect("树表条目"))
        .collect()
}

#[test]
fn recorded_paths_match_the_registered_segment_sequences() {
    let pool = build_pool("segments");
    let operations = pool.stream.operations();
    let sizes =
        |slice: &[RecordedOperation]| segment_sizes_text(&split_into_segments(slice, &geometry()));
    let kinds =
        |slice: &[RecordedOperation]| segment_kinds_text(&split_into_segments(slice, &geometry()));
    let closed_form = |slice: &[RecordedOperation]| {
        closed_form_state_count(&split_into_segments(slice, &geometry()))
    };
    let mkfs_operations = &operations[..pool.mkfs_operation_count];
    let acquisition_operations =
        &operations[pool.mkfs_operation_count..pool.acquisition_operation_count];
    let warm_up_operations =
        &operations[pool.acquisition_operation_count..pool.warm_up_operation_count];
    let transaction_operations = &operations[pool.warm_up_operation_count..];
    let post_mkfs_operations = &operations[pool.mkfs_operation_count..];
    // 产物第 31–35 行逐字（operations= / segments= / closed_form= / kinds=）；mkfs 那一行 2026-09-22 起
    // 与产物**不同**：清 journal 环与清根环都是这之后加的，E142 干跑的产物还没重跑（重跑之前门禁 checker-tier-qemu-device-streams 红着）。
    // 录制器按设备记屏障（代码审阅第 1 条，用户 2026-09-27 定）之后，每道池屏障两块盘各记一步：操作数与种类串里的屏障数
    // 与产物不同（mkfs 21 → 23、暖机 14 → 18、新池新建文件 31 → 33、mkfs 之后整条 47 → 53），段序列与闭式不变；产物要重出。
    // C577（发布返回之前、系统配置轮换之后一道池屏障）之后每次发布末尾多两步屏障：暖机 18 → 20（第一次末尾那一道顶替了第二次开头
    // 那道没有写在前、写入口不发的屏障，只多第二次末尾 2 步），新池新建文件 33 → 35，mkfs 之后整条 2 + 20 + 35 = 57；
    // 暖机第二次的系统配置槽轮换不再与新池新建文件的 24 个单元写同段：整条流 26 写一段拆成 2 与 24，
    // 闭式 1 + (3 + 3 + 1 + 3 + 3 + 1 + 3 + 3 + 1 + 3) + (2^24 − 1) = 16777240（改之前 67108885）。
    // mkfs 清 [0, 根环基址)（C651 第一轮判决用户定案 W1）之后每块盘多一步整段清零、两块盘多 2 步，都在第一段（第一道屏障之前）：
    // mkfs 23 → 25 步，第一段 12 → 14 个写，闭式 1 + (2^14 − 1) + 1 + 1 + 1 + (2^4 − 1) = 16402（改之前 1 + 4095 + 3 + 15 = 4114）；
    // mkfs 之后的各段不变。
    // 两趟写序（D22（单元原子性怎么合成） 已定项 8 第 3 条，`research/prompts/m3-close-c656-r3-main-verification.md` 第二节 S3）：
    // 两块盘的 [0, 根环基址) 清零连同作废那道屏障（两块盘各一步）提到最前、自成一段，mkfs 25 → 27 步，段 14+1+1+1+4 → 2+12+1+1+1+4，
    // 闭式 16402 → 1 + 3 + (2^12 − 1) + 1 + 1 + 1 + (2^4 − 1) = 4117；mkfs 之后的各段不变。
    // 取号写两轮（甲'，`research/prompts/m3-close-sysconf-r3-main-verification.md` 第四节「落地」）之后取号那一截 2 步 → 6 步（两轮各两写，中间一道屏障两盘各一步），
    // 段 [2] → [2+2]、闭式 4 → 7；mkfs 之后整条 57 → 61 步，段前面多一个 2 写的段，闭式 16777240 → 16777243；暖机与新池新建文件那两截不变。
    assert_eq!(
        (
            mkfs_operations.len(),
            sizes(mkfs_operations),
            closed_form(mkfs_operations)
        ),
        (27, "2+12+1+1+1+4".to_string(), 4117)
    );
    assert_eq!(
        kinds(mkfs_operations),
        "[zero_fill×2,barrier×2]|[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]"
    );
    assert_eq!(
        (
            acquisition_operations.len(),
            sizes(acquisition_operations),
            closed_form(acquisition_operations)
        ),
        (6, "2+2".to_string(), 7)
    );
    assert_eq!(
        kinds(acquisition_operations),
        "[system_configuration_slot×2,barrier×2]|[system_configuration_slot×2]"
    );
    assert_eq!(
        (
            warm_up_operations.len(),
            sizes(warm_up_operations),
            closed_form(warm_up_operations)
        ),
        (20, "2+1+2+2+1+2".to_string(), 15)
    );
    assert_eq!(
        kinds(warm_up_operations),
        "[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]"
    );
    assert_eq!(
        (
            transaction_operations.len(),
            sizes(transaction_operations),
            closed_form(transaction_operations)
        ),
        (35, "24+2+1+2".to_string(), 16_777_223)
    );
    assert_eq!(
        kinds(transaction_operations),
        "[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]"
    );
    assert_eq!(
        (
            post_mkfs_operations.len(),
            sizes(post_mkfs_operations),
            closed_form(post_mkfs_operations)
        ),
        (61, "2+2+2+1+2+2+1+2+24+2+1+2".to_string(), 16_777_243)
    );
    assert_eq!(
        kinds(post_mkfs_operations),
        "[system_configuration_slot×2,barrier×2]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]"
    );
    // 每一步恰好落在一个段里。
    for slice in [
        mkfs_operations,
        acquisition_operations,
        warm_up_operations,
        transaction_operations,
        post_mkfs_operations,
    ] {
        assert_eq!(
            split_into_segments(slice, &geometry())
                .iter()
                .map(Vec::len)
                .sum::<usize>(),
            slice.len()
        );
    }
    // 事务的 24 次单元写次序 = 十二个单元每个两盘，偏移 = 槽号 × 16384：数据单元、extent 树上段根兼叶、inode 叶容器、inode 根，
    // 分配记录树五个节点（D8（核心索引结构） 已定项 14：按 (层级, 盘, 序号) 先叶后根——盘 0 叶 61、盘 1 叶 61、盘 0 第 1 层、盘 1 第 1 层、根），
    // 记账树、中央映射树、树表。E142 产物 `name=write_list` 那一行是八个单元的旧格式，重跑之前与这里不同。
    let unit_offsets: Vec<u64> = transaction_operations[..24]
        .iter()
        .map(|operation| operation.offset.0)
        .collect();
    let expected: Vec<u64> = [
        50180u64, 50240, 50242, 50244, 50245, 50246, 50247, 50248, 50249, 50250, 50251, 50252,
    ]
    .iter()
    .flat_map(|slot| [slot * 16384, slot * 16384])
    .collect();
    assert_eq!(unit_offsets, expected);
}

#[test]
fn root_slots_system_configurations_and_journal_ring_hold_the_published_state() {
    let pool = build_pool("roots");
    let filesystem_identifier = unit_filesystem_identifier(&E142_FILESYSTEM_IDENTIFIER);
    // 根槽：新池新建文件 txg 3 落区域 0 槽 1；暖机 txg 1 / 2 落区域 1 / 2 的槽 0，分住两块盘（D22（单元原子性怎么合成） 已定项 16）。
    let newest = check_root_slot(&read_root_slot(&pool, 0, 1), &E142_FILESYSTEM_IDENTIFIER)
        .expect("第 3 代根");
    assert_eq!(
        (
            newest.checkpoint_txg,
            newest.instance,
            newest.tree_identifier_watermark,
            newest.rollback_floor
        ),
        (3, 1, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, 0)
    );
    assert_eq!(
        newest.tree_identifier_watermark, 65549,
        "D8 已定项 11：第一个文件版本那次之后树 ID 水位是 65549（十三棵树 65536..65548，退役树是第 50 批加的）"
    );
    assert_eq!(newest.record_bytes, pool.output.root.to_slot(512)[..466]);
    for (region, expected_txg) in [(1u64, 1u64), (2, 2)] {
        let warm = check_root_slot(
            &read_root_slot(&pool, region, 0),
            &E142_FILESYSTEM_IDENTIFIER,
        )
        .expect("暖机根");
        assert_eq!(
            (
                warm.checkpoint_txg,
                warm.instance,
                warm.tree_identifier_watermark
            ),
            (expected_txg, 1, 65536),
            "暖机根照 mkfs 的树表"
        );
        assert_eq!(
            warm.record_bytes,
            pool.warm_up.roots[usize::try_from(expected_txg - 1).expect("下标")].to_slot(512)
                [..466]
        );
    }
    let genesis = check_root_slot(&read_root_slot(&pool, 0, 0), &E142_FILESYSTEM_IDENTIFIER)
        .expect("第 0 代根");
    assert_eq!(
        (genesis.checkpoint_txg, genesis.instance),
        (0, 0),
        "mkfs 种的根还在"
    );
    // 系统配置：取号两轮写世代 2（槽 0）、3（槽 1），暖机写 4 / 5，事务写 6 槽 0、tail = jsn 3；两盘同。
    for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
        let slot_zero = read(device(&pool, identity), DeviceOffsetInBytes(0), 4096);
        let slot_one = read(device(&pool, identity), DeviceOffsetInBytes(4096), 4096);
        let (chosen_index, chosen) =
            choose_system_configuration(&[&slot_zero, &slot_one]).expect("可择");
        assert_eq!(chosen_index, 0);
        assert_eq!(
            (
                chosen.slot_generation,
                chosen.journal_tail,
                chosen.journal_instance,
                chosen.this_device
            ),
            (6, 3, 1, identity.0)
        );
        let older = check_system_configuration_slot(&slot_one).expect("槽 1");
        assert_eq!(
            (
                older.slot_generation,
                older.journal_tail,
                older.journal_instance
            ),
            (5, 2, 1),
            "暖机第二次写的槽 1"
        );
    }
    // journal 环：jsn 1、2 是空记录（事务号 0、不点名、新根段照 mkfs 根），jsn 3 点名 8 项；反向链按实例算，第一条 0。
    let mut previous_record: Option<Vec<u8>> = None;
    for counter in 1..=3u64 {
        let on_device_zero = read_journal_record(&pool, DeviceIdentity(0), counter);
        let on_device_one = read_journal_record(&pool, DeviceIdentity(1), counter);
        assert_eq!(on_device_zero, on_device_one, "两盘各一份、逐字节相同");
        let view =
            check_journal_record(&on_device_zero, filesystem_identifier).expect("记录自证过");
        assert_eq!(
            (
                view.instance,
                view.counter,
                view.checkpoint_txg,
                view.is_commit
            ),
            (1, counter, counter, true)
        );
        let expected_back_chain = previous_record
            .as_deref()
            .map_or(0, back_chain_of_record_header);
        assert_eq!(
            view.back_chain, expected_back_chain,
            "反向链 = 本实例内逻辑前一条记录的 496 字节头的 CRC-32C"
        );
        assert_eq!(
            view.ordinal_within_publish, 1,
            "三次发布各只有一条记录：本次发布内序号都是 1（D23（journal 的角色与格式） 已定项 4，空发布记录也写 1）"
        );
        assert_eq!(
            view.record_flags_byte, 1,
            "三次发布各只有一条记录：那一条就是本次发布末条，记录标志位 0 写 1、其余位 0（D23（journal 的角色与格式） 已定项 17，空发布记录也写 1）"
        );
        if counter < 3 {
            assert_eq!((view.transaction, view.named.len()), (0, 0), "空发布的记录");
            assert_eq!(
                (view.new_tree_identifier_watermark, view.new_rollback_floor),
                (65536, 0)
            );
            assert_eq!(
                &view.new_root_segment[..86],
                &pool.warm_up.roots[0].to_slot(512)[36..36 + 86],
                "新根段照 mkfs 的树表指针"
            );
        } else {
            assert_eq!(
                (view.transaction, view.named.len()),
                (1, 12),
                "新池新建文件点名十二个单元：七个角色加分配记录树五个节点（D8（核心索引结构） 已定项 14）"
            );
            assert_eq!(
                (view.new_tree_identifier_watermark, view.new_rollback_floor),
                (65549, 0),
                "新根段带的水位：十三棵树 65536..65548 之后是 65549（D8 已定项 11）"
            );
            assert_eq!(
                view.back_chain, E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION,
                "与 496 字节头下现算、checker 独立 CRC 核过的 back_chain 逐字相同（E142 第八次跑产物第 37 行是 307 字节头、偏移 7 写 0 下的值）"
            );
            assert_eq!(
                &view.new_root_segment[..86],
                &pool.output.root.to_slot(512)[36..36 + 86],
                "新根段的树表指针 = 根记录的"
            );
        }
        assert_eq!(
            check_journal_record(&on_device_zero, filesystem_identifier ^ 1),
            Err(Verdict::FilesystemIdentifierMismatch)
        );
        previous_record = Some(on_device_zero);
    }
    // 环里 jsn 4 那一槽还是 0：环没有别的记录。
    assert!(read_journal_record(&pool, DeviceIdentity(0), 4)
        .iter()
        .all(|byte| *byte == 0));
}

/// 一个码 2 节点头里的 key 区间按什么判：按位置寻址的树是位置规定罩的那一段（层级也由位置定），别的贴紧首末两条。
enum DeclaredKeyRange {
    PrescribedByThePosition {
        level: u8,
        range: (Vec<u8>, Vec<u8>),
    },
    TightOverTheEntries {
        schema: KeySchema,
        level: u8,
    },
}

#[test]
fn every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold() {
    let pool = build_pool("nodes");
    let tree_table_entries = tree_table_entries_from_disk(&pool);
    let mut checked_index_nodes = 0;
    for unit in &pool.output.units {
        for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
            let on_disk = read_unit_from(&pool, identity, unit.slot, unit_length(unit.identity));
            assert_eq!(on_disk, unit.bytes, "{} 两盘各一份", unit.identity.tag());
            assert_eq!(
                check_unit(&on_disk).expect("头自检过"),
                unit.identity.unit_class()
            );
        }
        if unit.identity.unit_class() != 2 {
            continue;
        }
        let view = index_node_view(&unit.bytes).expect("码 2 节点");
        assert_eq!(
            view.tree_identifier,
            unit.identity.tree(&pool.output.tree_identifiers).0
        );
        let schema_of_the_tree_in_the_tree_table = || {
            let kind = tree_table_entries
                .iter()
                .find(|entry| entry.tree.0 == unit.identity.tree(&pool.output.tree_identifiers).0)
                .expect("在树表里")
                .kind;
            key_schema_for_tree_kind(kind).expect("有节点的树都有 key 形态")
        };
        // 按位置寻址的两棵树（D8（核心索引结构） 已定项 14）：头里的 key 区间是位置规定罩的那一段，不贴紧条目；层级也由位置定。
        // 4 GiB 两块盘上分配记录树根在第 2 层（每块盘按第 1 层 812 × 169 槽切出 2 段，共 4 ≤ 169），第一个文件只有 inode 1
        // 那一条 extent 上段叶条目 ⇒ extent 树根兼叶在第 0 层。别的码 2 节点头里的 key 区间贴紧首末两条，只有 inode 树根是层级 1。
        let declared_key_range = match unit.identity {
            TransactionUnit::ExtentRoot => DeclaredKeyRange::PrescribedByThePosition {
                level: 0,
                range: extent_upper_cell_range(0, 0),
            },
            TransactionUnit::AllocationTree => DeclaredKeyRange::PrescribedByThePosition {
                level: 2,
                range: allocation_record_tree_cell_range(AllocationRecordTreeCell::Root),
            },
            TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => {
                DeclaredKeyRange::PrescribedByThePosition {
                    level: position.level,
                    range: allocation_record_tree_cell_range(
                        AllocationRecordTreeCell::BelowTheRoot {
                            level: position.level,
                            device: position.device.0,
                            index: position.index_in_device,
                        },
                    ),
                }
            }
            // 新池新建文件只有一个数据单元（内联），extent 树没有下段、上段只有根兼叶。
            TransactionUnit::ExtentLowerNode(_)
            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
            | TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. } => {
                unreachable!("新池新建文件的 extent 树只有上段根兼叶")
            }
            TransactionUnit::MappingTree => DeclaredKeyRange::TightOverTheEntries {
                schema: KEY_SCHEMA_MAPPING,
                level: 0,
            },
            TransactionUnit::TreeTable => DeclaredKeyRange::TightOverTheEntries {
                schema: KEY_SCHEMA_TREE_TABLE,
                level: 0,
            },
            TransactionUnit::InodeRoot => DeclaredKeyRange::TightOverTheEntries {
                schema: schema_of_the_tree_in_the_tree_table(),
                level: 1,
            },
            TransactionUnit::AccountingTree => DeclaredKeyRange::TightOverTheEntries {
                schema: schema_of_the_tree_in_the_tree_table(),
                level: 0,
            },
            // 新池新建文件的两棵多层码 2 树都只有一个节点（根兼叶）：根之下的节点在这一版里不存在。
            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
            | TransactionUnit::MappingTreeNodeBelowTheRoot(_) => {
                unreachable!("新池新建文件的记账树与中央映射树都是根兼叶")
            }
            TransactionUnit::Data(_)
            | TransactionUnit::DataOfAFileOtherThanTheFirst { .. }
            | TransactionUnit::InodeLeafContainer(_)
            | TransactionUnit::InstanceTable
            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                unreachable!("上面按类跳过了")
            }
            TransactionUnit::DeletedInodesRoot
            | TransactionUnit::LoggedOperationsRoot
            | TransactionUnit::TombstoneLeafContainer(_)
            | TransactionUnit::TombstoneRoot
            | TransactionUnit::TombstoneNodeBelowTheRoot(_)
            | TransactionUnit::LineageLeafContainer(_)
            | TransactionUnit::LineageRoot
            | TransactionUnit::LineageNodeBelowTheRoot(_)
            | TransactionUnit::RetirementLeafContainer(_)
            | TransactionUnit::RetirementRoot
            | TransactionUnit::RetirementNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTreeNodeBelowTheRoot(_)
            | TransactionUnit::DeadlistTree => {
                unreachable!("新池新建文件不删除、不回退、没有快照：删除那两棵树、谱系树、退役树与 deadlist 树零根")
            }
        };
        match declared_key_range {
            DeclaredKeyRange::PrescribedByThePosition {
                level,
                range: (smallest_key, largest_key),
            } => {
                check_internal_node_separators(&view, schema_of_the_tree_in_the_tree_table())
                    .expect("条目严格递增、节点不空");
                assert_eq!(
                    (view.level, view.smallest_key, view.largest_key),
                    (level, smallest_key, largest_key),
                    "{}：层级与 key 区间是位置规定的",
                    unit.identity.tag()
                );
            }
            DeclaredKeyRange::TightOverTheEntries { schema, level } => {
                check_index_node_keys(&view, schema).expect("key 区间贴紧、条目严格递增");
                assert_eq!(view.level, level, "{} 的层级", unit.identity.tag());
            }
        }
        checked_index_nodes += 1;
    }
    let allocation_record_tree_nodes_below_the_root = pool
        .output
        .units
        .iter()
        .filter(|unit| {
            matches!(
                unit.identity,
                TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
            )
        })
        .count();
    assert_eq!(
        checked_index_nodes,
        6 + allocation_record_tree_nodes_below_the_root,
        "extent、inode 根、分配记录树根、记账、映射、树表，加分配记录树根之下的节点"
    );
    // Merkle（D4（校验和位置） 已定项 1）：每条父指针里的整单元校验和等于子单元算出来的。
    let checksum_of =
        |identity: TransactionUnit| crc32_castagnoli_bitwise(&pool.output.unit(identity).bytes);
    let root = &pool.output.root;
    assert_eq!(
        root.tree_table.locations[0].unit_checksum,
        checksum_of(TransactionUnit::TreeTable)
    );
    assert_eq!(
        root.mapping_root.locations[0].unit_checksum,
        checksum_of(TransactionUnit::MappingTree)
    );
    for entry in &tree_table_entries {
        let child = match entry.tree.0 {
            TREE_IDENTIFIER_EXTENT => Some(TransactionUnit::ExtentRoot),
            TREE_IDENTIFIER_INODE => Some(TransactionUnit::InodeRoot),
            singlefs_format::TREE_IDENTIFIER_ALLOCATION_RECORDS => {
                Some(TransactionUnit::AllocationTree)
            }
            singlefs_format::TREE_IDENTIFIER_ACCOUNTING => Some(TransactionUnit::AccountingTree),
            _ => None,
        };
        if let Some(child) = child {
            assert_eq!(
                entry.root.locations[0].unit_checksum,
                checksum_of(child),
                "树 {} 的根指针",
                entry.tree.0
            );
            assert_eq!(entry.root.locations[1].unit_checksum, checksum_of(child));
        } else {
            assert_eq!(
                entry.root,
                NodePointer::empty_root(),
                "day-1 只注册的树根指针全零"
            );
        }
    }
    assert_eq!(
        pool.output.data_pointers[0].locations[0].unit_checksum,
        checksum_of(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
    );
    let inode_root =
        index_node_view(&pool.output.unit(TransactionUnit::InodeRoot).bytes).expect("inode 根");
    let child_pointer = NodePointer::read_from(&mut ByteReader::at(&inode_root.entries[0], 8 + 26));
    assert_eq!(
        child_pointer.locations[0].unit_checksum,
        checksum_of(TransactionUnit::InodeLeafContainer(
            InodeLeafContainerIndexInTree::LEFTMOST
        )),
        "inode 内部条目的子指针"
    );
    // 位置条目按设备身份升序（I-2.5）。
    for entry in tree_table_entries
        .iter()
        .filter(|entry| entry.root != NodePointer::empty_root())
    {
        assert!(entry.root.locations[0].device < entry.root.locations[1].device);
    }
}

#[test]
fn inode_and_extent_lookups_from_the_root_read_the_first_file_back() {
    let pool = build_pool("lookup");
    let tree_table_entries = tree_table_entries_from_disk(&pool);
    // inode 树：根（层级 1）→ 内部条目（分隔 key = ino）→ 叶容器 → 140 字节记录。
    let inode_entry = tree_table_entries
        .iter()
        .find(|entry| entry.tree.0 == TREE_IDENTIFIER_INODE)
        .expect("inode 树在树表里");
    let inode_root_location = inode_entry.root.locations[1];
    let inode_root_unit = read_unit_from(
        &pool,
        inode_root_location.device,
        inode_root_location.slot,
        16384,
    );
    assert_eq!(
        crc32_castagnoli_bitwise(&inode_root_unit),
        inode_root_location.unit_checksum
    );
    let inode_root = index_node_view(&inode_root_unit).expect("inode 根");
    assert_eq!(inode_root.entries.len(), 1);
    let mut reader = ByteReader::at(&inode_root.entries[0], 0);
    assert_eq!(reader.get_u64(), FIRST_INODE_NUMBER, "分隔 key");
    assert_eq!(
        (
            reader.get_u64(),
            reader.get_u16(),
            reader.get_u64(),
            reader.get_u64()
        ),
        (65537, 2, 1, 3),
        "身份引用四元组"
    );
    let leaf_pointer = NodePointer::read_from(&mut reader);
    let leaf_unit = read_unit_from(
        &pool,
        leaf_pointer.locations[0].device,
        leaf_pointer.locations[0].slot,
        32768,
    );
    assert_eq!(
        crc32_castagnoli_bitwise(&leaf_unit),
        leaf_pointer.locations[0].unit_checksum
    );
    let leaf = packed_unit_view(&leaf_unit).expect("叶容器");
    assert_eq!(
        (
            leaf.birth_tree,
            leaf.record_type,
            leaf.container,
            leaf.container_birth,
            leaf.record_width
        ),
        (65537, 2, 1, 3, 140)
    );
    assert_eq!(leaf.records.len(), 1);
    assert_eq!(
        u64::from_le_bytes(
            leaf.records[0][48..56]
                .try_into()
                .expect("记录偏移 48 起 8 字节")
        ),
        6,
        "blocks = ⌈3000 ÷ 512⌉ = 6：逻辑长度的块数，不是分到的一个 32 KiB 单元的 64（D8（核心索引结构） 已定项 6，C480）"
    );
    let inode_record = InodeRecord::parse(&leaf.records[0]).expect("inode 记录");
    assert_eq!(
        inode_record,
        InodeRecord {
            inode: FIRST_INODE_NUMBER,
            object_birth: CheckpointTxg(3),
            nlink: singlefs_core::records::NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
            size: 3000,
            // 改动计数 = 最后一次改动所在发布的 checkpoint_txg（D8（核心索引结构） 已定项 6 偏移 88）：
            // 新池新建文件发布在 txg 3（增补 2 第 11 行，2026-09-18 用户定案；此前钉的 1 是暖机把 txg 推到 3 之前的值）。
            change_count: 3,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            // 新池新建文件是整份写：截断字段是 0（判决 m4-trunc-r2 R2）。
            truncation_field: singlefs_core::records::InodeRecordTruncationField::NOT_WRITTEN,
        },
        "逐字段回读等于写入"
    );
    // extent 树：上段根兼叶（D8（核心索引结构） 已定项 14）→ 上段叶条目 113 = key (0, ino, 0) + 标签 2（一个单元的文件内联）
    // + 数据指针 88 → 数据单元 → 载荷前 size 字节。
    let extent_entry = tree_table_entries
        .iter()
        .find(|entry| entry.tree.0 == TREE_IDENTIFIER_EXTENT)
        .expect("extent 树在树表里");
    let extent_unit = read_unit_from(
        &pool,
        extent_entry.root.locations[0].device,
        extent_entry.root.locations[0].slot,
        16384,
    );
    let extent_root = index_node_view(&extent_unit).expect("extent 根兼叶");
    assert_eq!(
        (
            extent_root.level,
            extent_root.entries.len(),
            extent_root.entry_width
        ),
        (0, 1, 113)
    );
    let mut extent_reader = ByteReader::at(&extent_root.entries[0], 0);
    assert_eq!(
        (
            extent_reader.get_u64(),
            extent_reader.get_u64(),
            extent_reader.get_u64()
        ),
        (0, FIRST_INODE_NUMBER, 0),
        "extent key"
    );
    assert_eq!(
        extent_reader.get_u8(),
        2,
        "标签 2：这个文件唯一那个数据单元的数据指针内联在上段叶条目里"
    );
    let pointer = DataPointer::read_from(&mut extent_reader);
    assert_eq!(pointer, pool.output.data_pointers[0]);
    assert_eq!(
        (
            pointer.head.birth_tree.0,
            pointer.head.birth_txg.0,
            pointer.write_order.instance.0,
            pointer.write_order.transaction
        ),
        (65536, 3, 1, 1)
    );
    for location in pointer.locations {
        let data_unit = read_unit_from(&pool, location.device, location.slot, 32768);
        assert_eq!(crc32_castagnoli_bitwise(&data_unit), location.unit_checksum);
        assert_eq!(check_unit(&data_unit).expect("数据单元"), 1);
        assert_eq!(
            &data_unit[134..134 + usize::try_from(inode_record.size).expect("size")],
            &file_content()[..],
            "读回内容逐字节相同"
        );
    }
}

#[test]
fn central_mapping_holds_ten_entries_rebuilt_from_the_named_entries_and_excludes_itself() {
    let pool = build_pool("mapping");
    let mapping_location = pool.output.root.mapping_root.locations[0];
    let mapping_unit = read_unit_from(&pool, mapping_location.device, mapping_location.slot, 16384);
    let mapping = index_node_view(&mapping_unit).expect("映射根兼叶");
    check_index_node_keys(&mapping, KEY_SCHEMA_MAPPING).expect("映射 key 严格递增");
    assert_eq!(
        (
            mapping.tree_identifier,
            mapping.entries.len(),
            mapping.key_width,
            mapping.entry_width
        ),
        (65540, 10, 27, 55),
        "十条映射条目：数据单元、extent 根、inode 叶、inode 根、记账树各一条，分配记录树五个节点各一条"
    );
    // 按步 2 那个单元的逻辑身份查映射，位置等于 extent 树指针里的位置提示；key 三段与数据单元头同值。
    let data_key = mapping_key_for_data(
        pool.output.data_pointers[0].head,
        pool.output.data_pointers[0].write_order,
    );
    let data_entry = mapping
        .entries
        .iter()
        .find(|entry| entry[..27] == data_key[..])
        .expect("码 1 的条目在映射里");
    let mut reader = ByteReader::at(data_entry, 27);
    let locations = [
        singlefs_core::pointer::LocationEntry::read_from(&mut reader),
        singlefs_core::pointer::LocationEntry::read_from(&mut reader),
    ];
    assert_eq!(locations, pool.output.data_pointers[0].locations);
    let data_unit = &pool
        .output
        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .bytes;
    assert_eq!(
        &data_key[1..9],
        &data_unit[43..51],
        "出生树 = 数据单元头的树 ID"
    );
    assert_eq!(
        &data_key[9..17],
        &data_unit[75..83],
        "出生 txg = 数据单元头的诞生代号"
    );
    assert_eq!(
        &data_key[17..27],
        &data_unit[91..101],
        "写序 = 数据单元头的写序"
    );
    // 映射树自己的节点、树表单元不在映射里（自举豁免）；点名项凑出来的 key 罩住全部 10 条。
    assert!(mapping.entries.iter().all(|entry| u64::from_le_bytes(
        entry[1..9].try_into().expect("8")
    ) != TREE_IDENTIFIER_CENTRAL_MAPPING));
    assert!(
        mapping
            .entries
            .iter()
            .all(|entry| u64::from_le_bytes(entry[1..9].try_into().expect("8")) != 0),
        "树表单元不进映射"
    );
    let record = check_journal_record(
        &pool.output.record_bytes,
        unit_filesystem_identifier(&E142_FILESYSTEM_IDENTIFIER),
    )
    .expect("记录");
    let rebuilt: BTreeSet<Vec<u8>> = record
        .named
        .iter()
        .map(singlefs_checker::NamedEntryView::mapping_key)
        .collect();
    let in_mapping: BTreeSet<Vec<u8>> = mapping
        .entries
        .iter()
        .map(|entry| entry[..27].to_vec())
        .collect();
    assert_eq!(rebuilt.len(), 12, "12 项各自凑出不同的 key");
    assert!(
        in_mapping.is_subset(&rebuilt),
        "映射里的 10 条 key 全能从点名项凑出来"
    );
    assert_eq!(
        rebuilt.difference(&in_mapping).count(),
        2,
        "多出来的两把是映射树自己与树表单元"
    );
    assert_eq!(pool.output.mapping_keys.len(), 10);
}

#[test]
fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
    let pool = build_pool("accounting");
    let tree_table_entries = tree_table_entries_from_disk(&pool);
    let allocation_entry = tree_table_entries
        .iter()
        .find(|entry| entry.tree.0 == singlefs_format::TREE_IDENTIFIER_ALLOCATION_RECORDS)
        .expect("分配记录树");
    let allocation_unit = read_unit_from(
        &pool,
        allocation_entry.root.locations[0].device,
        allocation_entry.root.locations[0].slot,
        16384,
    );
    // 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，两条内部条目（96 = key 10 + 子指针 86）
    // 各指一块盘第 1 层那一个节点，它再各指单元区起点那片叶（叶 61 罩槽 [49532, 50344)）；记录全在两片叶里。
    let allocation = index_node_view(&allocation_unit).expect("分配记录树根");
    assert_eq!(
        (
            allocation.level,
            allocation.entries.len(),
            allocation.entry_width,
            allocation.key_width
        ),
        (2, 2, 96, 10)
    );
    let mut records: Vec<AllocationRecord> = Vec::new();
    let mut nodes_to_read = vec![allocation];
    let mut leaves_read = 0;
    // 迭代上界是树里的节点数（根、每盘第 1 层一个、每盘一片叶）；跨轮携带的是还没读的节点。
    while let Some(node) = nodes_to_read.pop() {
        if node.level == 0 {
            assert_eq!(
                (node.entry_width, node.key_width),
                (20, 10),
                "叶条目 20、key 10"
            );
            records.extend(node.entries.iter().map(|bytes| {
                AllocationRecord::parse(bytes)
                    .expect("上一句刚把这片叶的条目宽钉成 20（I-1.10：码 2 条目宽等于字段表宽）")
            }));
            leaves_read += 1;
            continue;
        }
        assert_eq!(node.entry_width, 96, "内部条目 96");
        for entry in &node.entries {
            let child = NodePointer::read_from(&mut ByteReader::at(entry, 10));
            let child_unit = read_unit_from(
                &pool,
                child.locations[0].device,
                child.locations[0].slot,
                16384,
            );
            assert_eq!(
                crc32_castagnoli_bitwise(&child_unit),
                child.locations[0].unit_checksum,
                "子指针里的整单元校验和"
            );
            nodes_to_read.push(index_node_view(&child_unit).expect("分配记录树节点"));
        }
    }
    assert_eq!(leaves_read, 2, "每块盘一片叶");
    records.sort_by_key(|record| (record.device, record.slot));
    let device_zero_slots: Vec<u64> = records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0))
        .map(|record| record.slot.0)
        .collect();
    assert_eq!(
        device_zero_slots,
        vec![
            50176, 50178, 50180, 50240, 50242, 50244, 50245, 50246, 50247, 50248, 50249, 50250,
            50251, 50252
        ]
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.generation == CheckpointTxg(0))
            .count(),
        2,
        "mkfs 的 m1 分配代 0；m2（第 0 版树表）被 A 换下，记录改写成已释放、释放代 3"
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.generation == CheckpointTxg(3))
            .count(),
        26,
        "A 的十二个落点（七个角色加分配记录树五个节点）各两盘 24 条，加 m2 那两条改写成释放代 3"
    );
    assert_eq!(
        records
            .iter()
            .map(|record| u64::from(record.span_slots))
            .sum::<u64>(),
        34,
        "每盘 17 槽"
    );
    assert_eq!(records, pool.output.allocation_records);
    // 记账树 15 行（产物第 61 行是旧格式：allocated_bytes_per_device=212992 free_bytes_per_device=3472670720 empty_cluster_segments_per_device=3310
    // fragmentation_runs_per_device=4 inode_watermark=2；分配记录树按位置寻址之后每盘多占 4 槽，已分配 17 槽 = 278528、空闲少 65536，
    // 全空聚簇段、碎片段数、inode 水位不变）。
    let accounting_entry = tree_table_entries
        .iter()
        .find(|entry| entry.tree.0 == singlefs_format::TREE_IDENTIFIER_ACCOUNTING)
        .expect("记账树");
    let accounting_unit = read_unit_from(
        &pool,
        accounting_entry.root.locations[1].device,
        accounting_entry.root.locations[1].slot,
        16384,
    );
    let accounting = index_node_view(&accounting_unit).expect("记账树");
    assert_eq!(
        (
            accounting.entries.len(),
            accounting.entry_width,
            accounting.key_width
        ),
        (15, 34, 22)
    );
    let rows: Vec<AccountingEntry> = accounting
        .entries
        .iter()
        .map(|bytes| {
            AccountingEntry::parse(bytes)
                .expect("上一句刚把这个节点的条目宽钉成 34（I-1.10：码 2 条目宽等于字段表宽）")
        })
        .collect();
    assert_eq!(rows, pool.output.accounting_entries);
    assert!(rows
        .iter()
        .all(|row| row.sequence == 1 && row.generation == CheckpointTxg(3)));
    let value_of = |statistic: u16| {
        rows.iter()
            .find(|row| row.statistic == statistic)
            .expect("统计量")
            .value
    };
    let devices_of = |statistic: u16| {
        rows.iter()
            .filter(|row| row.statistic == statistic)
            .map(|row| row.device.0)
            .collect::<Vec<u32>>()
    };
    assert_eq!(value_of(STATISTIC_ALLOCATED_BYTES), 278_528);
    assert_eq!(value_of(STATISTIC_FREE_BYTES), 3_472_605_184);
    assert_eq!(
        value_of(STATISTIC_ALLOCATED_BYTES) + value_of(STATISTIC_FREE_BYTES),
        211_968 * 16384,
        "已分配 + 空闲 = 单元区"
    );
    assert_eq!(value_of(STATISTIC_EMPTY_CLUSTER_SEGMENTS), 3310);
    assert_eq!(
        value_of(STATISTIC_FRAGMENTATION_RUNS),
        4,
        "[50179]、[50182, 50239]、[50241]、[50253, 末]"
    );
    assert_eq!(value_of(STATISTIC_INODE_WATERMARK), 2);
    for statistic in [
        STATISTIC_UNRECLAIMABLE_BYTES,
        STATISTIC_PENDING_DELETE_BYTES,
    ] {
        assert_eq!(value_of(statistic), 0, "不可回收与待删占用 day-1 写 0 行");
    }
    assert_eq!(
        value_of(STATISTIC_COMMITTED_RESERVATION_BYTES),
        2 * ((2 + 1 + 2 + 1) * 16384 + 41 + 35),
        "已承诺预留（标签 6）按这一版现算：两份副本 × (墓碑 3 槽 + 退役 3 槽常驻 + 41 字节 × 一个活对象 + 35 字节 × 一条 inode 记录)（D3 已定项 2，layout/03「记账标签 6 的值」；退役那一份是判决 m3-retire-r1 第三节第 6 条）"
    );
    assert_eq!(
        value_of(STATISTIC_DEFER_QUEUE_BYTES),
        16384,
        "defer 待释放 = mkfs 那片第 0 版树表单元（1 槽）：A 重写树表把它换下（D3 已定项 7）"
    );
    for statistic in [
        STATISTIC_ALLOCATED_BYTES,
        STATISTIC_FREE_BYTES,
        STATISTIC_UNRECLAIMABLE_BYTES,
        STATISTIC_DEFER_QUEUE_BYTES,
        STATISTIC_FRAGMENTATION_RUNS,
        STATISTIC_EMPTY_CLUSTER_SEGMENTS,
    ] {
        assert_eq!(devices_of(statistic), vec![0, 1], "带设备维的每盘一行");
    }
    for statistic in [
        STATISTIC_PENDING_DELETE_BYTES,
        STATISTIC_COMMITTED_RESERVATION_BYTES,
        STATISTIC_INODE_WATERMARK,
    ] {
        assert_eq!(
            devices_of(statistic),
            vec![STATISTIC_NO_DEVICE_DIMENSION],
            "池级一行、设备段取保留值"
        );
    }
    let watermark_row = rows
        .iter()
        .find(|row| row.statistic == STATISTIC_INODE_WATERMARK)
        .expect("水位");
    assert_eq!(
        watermark_row.tree,
        TreeIdentifier(TREE_IDENTIFIER_INODE),
        "第 12 项带树维"
    );
    // 运行时计数（C319）：取号序与 key 序不一致的次数，新池新建文件是 0。
    assert_eq!(pool.output.key_order_mismatches, 0);
}

/// D3（空间分配） 已定项 10 ⑤ 那张落点表给分配记录树五个节点（t5–t9）写的出生序号 0..4（C686）：按位置先叶后根、同层按 key 升序 bump，
/// 出生序号按同一个次序发（D19（块指针的结构与宽度预算） 已定项 9）。每个节点两块盘上的两份逐字节相同，头里的树 ID、层级、出生序号与表里那一行相同；
/// 树表条目与父节点条目里指向它的指针，落点与出生序号也与表里那一行相同；两片叶里的记录各是表里写的那块盘的。
#[test]
fn allocation_record_tree_nodes_carry_birth_sequences_zero_to_four_as_the_bump_order_table_lists() {
    let pool = build_pool("allocation-birth-sequences");
    // (表里那一行, 落点, 层级, 出生序号, 罩哪块盘)：照 D3 已定项 10 ⑤ 的表逐行抄；根罩两块盘，记 `None`。
    let bump_order_table: [(&str, u64, u8, u32, Option<DeviceIdentity>); 5] = [
        (
            "t5 分配记录树叶，设备 0",
            50245,
            0,
            0,
            Some(DeviceIdentity(0)),
        ),
        (
            "t6 分配记录树叶，设备 1",
            50246,
            0,
            1,
            Some(DeviceIdentity(1)),
        ),
        (
            "t7 分配记录树层级 1 节点，设备 0",
            50247,
            1,
            2,
            Some(DeviceIdentity(0)),
        ),
        (
            "t8 分配记录树层级 1 节点，设备 1",
            50248,
            1,
            3,
            Some(DeviceIdentity(1)),
        ),
        ("t9 分配记录树根，层级 2", 50249, 2, 4, None),
    ];
    for (row, slot, level, birth_sequence, covered_device) in bump_order_table {
        let copy_on_device_zero = read_unit_from(&pool, DeviceIdentity(0), SlotNumber(slot), 16384);
        let copy_on_device_one = read_unit_from(&pool, DeviceIdentity(1), SlotNumber(slot), 16384);
        assert_eq!(
            copy_on_device_zero, copy_on_device_one,
            "{row}：两块盘上的两份逐字节相同"
        );
        let view = index_node_view(&copy_on_device_zero).expect("分配记录树节点");
        assert_eq!(
            (view.tree_identifier, view.level, view.birth_sequence),
            (
                singlefs_format::TREE_IDENTIFIER_ALLOCATION_RECORDS,
                level,
                birth_sequence
            ),
            "{row}：头里的树 ID、层级、出生序号与 D3 已定项 10 ⑤ 的落点表相同"
        );
        if level == 0 {
            let devices_of_the_records: BTreeSet<DeviceIdentity> = view
                .entries
                .iter()
                .map(|bytes| {
                    AllocationRecord::parse(bytes)
                        .expect("分配记录树叶的条目是 20 字节的分配记录（同文件分配记录树那条用例钉过条目宽）")
                        .device
                })
                .collect();
            assert_eq!(
                devices_of_the_records.into_iter().collect::<Vec<_>>(),
                covered_device.into_iter().collect::<Vec<_>>(),
                "{row}：叶里的记录全是表里写的那块盘的"
            );
            continue;
        }
        let children: Vec<(u64, u32)> = view
            .entries
            .iter()
            .map(|entry| {
                let child = NodePointer::read_from(&mut ByteReader::at(entry, 10));
                (child.locations[0].slot.0, child.birth_sequence.0)
            })
            .collect();
        let children_in_the_table: Vec<(u64, u32)> = bump_order_table
            .iter()
            .filter(|(_, _, child_level, _, child_device)| {
                *child_level + 1 == level
                    && covered_device.is_none_or(|device| *child_device == Some(device))
            })
            .map(|(_, child_slot, _, child_birth_sequence, _)| (*child_slot, *child_birth_sequence))
            .collect();
        assert_eq!(
            children, children_in_the_table,
            "{row}：条目里孩子指针的落点与出生序号是表里低一层、同一块盘那几行的"
        );
    }
    let allocation_entry = tree_table_entries_from_disk(&pool)
        .into_iter()
        .find(|entry| entry.tree.0 == singlefs_format::TREE_IDENTIFIER_ALLOCATION_RECORDS)
        .expect("分配记录树");
    assert_eq!(
        (
            allocation_entry.root.locations[0].slot.0,
            allocation_entry.root.birth_sequence.0
        ),
        (50249, 4),
        "树表条目里分配记录树的根指针指着表里 t9 那一行"
    );
}

/// 树表 12 条（D18（块里携带什么信息） 已定项 24 第 6 条：7 → 11，`deleted_inodes`、墓碑树、谱系树、logged_ops 零根注册；
/// 退役树 11 → 12，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条），
/// 树 ID、种类码、头 ID 三列逐条钉条款登记的值：树 ID 从 65536 起（D8（核心索引结构） 已定项 11），种类码 9..12 接在 8 之后（已定项 9），
/// 头 ID 照已定项 8 / D5（快照 / 空间记账机制） 已定项 9（inode 树、extent 树、`deleted_inodes`、墓碑树写 inode 树的树 ID 65537，其余 0）；后四条根指针全零。
#[test]
fn tree_table_holds_twelve_entries_keyed_by_tree_identifier() {
    let pool = build_pool("treetable");
    let entries = tree_table_entries_from_disk(&pool);
    assert_eq!(
        entries.len(),
        12,
        "D18 已定项 24 第 6 条树表 7 → 11 条，退役树（判决 m3-retire-r1 第三节第 1 条）11 → 12 条"
    );
    assert_eq!(
        entries.iter().map(|entry| entry.tree.0).collect::<Vec<_>>(),
        vec![65536, 65537, 65538, 65539, 65541, 65542, 65543, 65544, 65545, 65546, 65547, 65548],
        "D8 已定项 11：树 ID 从 65536 起，中央映射树 65540 不进树表，退役树 65548 排最后"
    );
    assert_eq!(
        entries.iter().map(|entry| entry.kind).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13]
    );
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.head_identifier)
            .collect::<Vec<_>>(),
        vec![65537, 65537, 0, 0, 0, 0, 0, 65537, 65537, 0, 0, 65537],
        "D8 已定项 8 头 ID：inode 树、extent 树、deleted_inodes、墓碑树、退役树写 inode 树的树 ID，谱系树、logged_ops 与其余写 0"
    );
    assert!(
        entries[7..]
            .iter()
            .all(|entry| entry.root == NodePointer::empty_root()),
        "五棵新树零根注册：根指针全零"
    );
    assert!(entries
        .iter()
        .all(|entry| entry.birth_txg == CheckpointTxg(3)));
    assert_eq!(entries, pool.output.tree_table_entries);
    let tree_table_unit = &pool.output.unit(TransactionUnit::TreeTable).bytes;
    assert_eq!(
        u16::from_le_bytes([tree_table_unit[8], tree_table_unit[9]]),
        2400,
        "声明长度 12 × 200"
    );
    let view = index_node_view(tree_table_unit).expect("树表单元");
    assert_eq!(
        (view.tree_identifier, view.level, view.birth_sequence),
        (0, 0, 0)
    );
    assert_eq!(view.smallest_key, 65536u64.to_le_bytes());
    assert_eq!(view.largest_key, 65548u64.to_le_bytes());
    // mkfs 的第 0 版树表还在原槽、0 条：COW 出新版本，不改旧的。
    let genesis_unit = read_unit_from(&pool, DeviceIdentity(0), TREE_TABLE_GENESIS_SLOT, 16384);
    assert_eq!(genesis_unit, pool.genesis.tree_table_genesis_unit);
    assert_eq!(
        index_node_view(&genesis_unit)
            .expect("第 0 版")
            .entries
            .len(),
        0
    );
}

#[test]
fn mutations_are_caught_by_the_check_that_owns_them() {
    let pool = build_pool("mutations");
    // 叶容器头里记录宽改成 139：头校验和判红（头合法性）。
    let mut narrow_leaf = pool
        .output
        .unit(TransactionUnit::InodeLeafContainer(
            InodeLeafContainerIndexInTree::LEFTMOST,
        ))
        .bytes
        .clone();
    narrow_leaf[71..73].copy_from_slice(&139u16.to_le_bytes());
    assert_eq!(
        packed_unit_view(&narrow_leaf),
        Err(Verdict::ChecksumMismatch)
    );
    // 子节点改坏一字节：父指针里的校验和判红。
    let mut damaged_leaf = pool
        .output
        .unit(TransactionUnit::InodeLeafContainer(
            InodeLeafContainerIndexInTree::LEFTMOST,
        ))
        .bytes
        .clone();
    damaged_leaf[20000] ^= 1;
    let inode_root =
        index_node_view(&pool.output.unit(TransactionUnit::InodeRoot).bytes).expect("inode 根");
    let child_pointer = NodePointer::read_from(&mut ByteReader::at(&inode_root.entries[0], 8 + 26));
    assert_ne!(
        crc32_castagnoli_bitwise(&damaged_leaf),
        child_pointer.locations[0].unit_checksum
    );
    // 条目 key 逆序：checker 的 key 序检查判红；区间与首条不贴紧也判红。
    let mut reversed =
        index_node_view(&pool.output.unit(TransactionUnit::TreeTable).bytes).expect("树表");
    reversed.entries.reverse();
    assert_eq!(
        check_index_node_keys(&reversed, KEY_SCHEMA_TREE_TABLE),
        Err(Verdict::KeysNotStrictlyAscending)
    );
    let mut loose_range =
        index_node_view(&pool.output.unit(TransactionUnit::TreeTable).bytes).expect("树表");
    loose_range.smallest_key = 10u64.to_le_bytes().to_vec();
    assert_eq!(
        check_index_node_keys(&loose_range, KEY_SCHEMA_TREE_TABLE),
        Err(Verdict::KeyOutsideDeclaredRange)
    );
    assert_eq!(
        check_index_node_keys(&loose_range, KEY_SCHEMA_MAPPING),
        Err(Verdict::KeyWidthMismatch)
    );
    // 树表条目预留的最后一个字节塞成非零：条目解析拒收。下标从条目长度取，不写死（条目宽改过两次）。
    let mut tampered_entry = pool.output.tree_table_entries[0].to_bytes();
    let last_reserved_byte = tampered_entry.len() - 1;
    tampered_entry[last_reserved_byte] = 1;
    assert_eq!(TreeTableEntry::parse(&tampered_entry), None);
    // journal 记录补齐区改一字节：`header_csum` 罩整条 4096，判红。
    let mut torn_record = pool.output.record_bytes.clone();
    torn_record[4000] ^= 1;
    assert_eq!(
        check_journal_record(
            &torn_record,
            unit_filesystem_identifier(&E142_FILESYSTEM_IDENTIFIER)
        ),
        Err(Verdict::ChecksumMismatch)
    );
    // 声明长度与条目宽对不上：解析拒绝（E142 第八次跑变异 M62 逼出来的那条）。
    let mut wrong_width = pool.output.unit(TransactionUnit::TreeTable).bytes.clone();
    wrong_width[84 + 16..86 + 16].copy_from_slice(&199u16.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(&mut wrong_width, 86 + 16);
    assert_eq!(
        index_node_view(&wrong_width),
        Err(Verdict::DeclaredLengthMismatch)
    );
}

/// 内存池那一份闭式同一条：一段 64 次写装不进 u64，停下并点名用分片计数。
#[test]
#[should_panic(expected = "闭式只算写数少于 64 的段")]
fn the_memory_pool_closed_form_refuses_a_segment_of_sixty_four_writes() {
    let segments: Vec<Vec<usize>> = vec![(0..64).collect()];
    let _ = singlefs_harness::memory_pool::closed_form_state_count(&segments);
}
