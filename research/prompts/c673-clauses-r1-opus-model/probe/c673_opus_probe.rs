//! c673-clauses-r1 云端攻方的原型（副本上跑，不入库）：K1 头 ID 两种读法、K2 新池新建文件落点、K3 写入时间。
//! 每个用例只打印 `name=` 行；断言只用来保证历史真的走到了要的那一步，判据的结论按打印的行读。
//! 盘一律是内存稀疏盘，不在磁盘上建镜像文件。

mod common;

use common::{
    crash_state_devices, file_content, geometry, memory_pool_of_sparse_devices, parameters,
    unreadable_root_slot_of, with_unreadable_ranges, without_unreadable_ranges,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::records::TreeTableEntry;
use singlefs_core::recovery::PoolReader;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite,
    publish_sequential_write, warm_up, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
    TransactionUnit,
};
use singlefs_core::unit::parse_index_node;
use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

type Mem = RecordingBlockDevice<SparseBlockDevice>;

const TREE_KIND_EXTENT: u16 = 1;
const TREE_KIND_INODE: u16 = 2;
const TREE_KIND_DELETED_INODES: u16 = 9;
const TREE_KIND_TOMBSTONE: u16 = 10;
const LITERAL_HEAD: u64 = 65537;

fn memory_devices(stream: &SharedStream, bytes: u64) -> Vec<(DeviceIdentity, Mem)> {
    (0..2u32)
        .map(|number| {
            (
                DeviceIdentity(number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(number),
                    SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// 从镜像字节读出现行那一版树表的条目（只读挂载择根 + 施加前缀之后那条根；与发布路径交回的内存结构不共用）。
fn tree_table_from_image(image: &MemoryPool) -> (u32, u64, Vec<TreeTableEntry>) {
    let mounted = mount_read_only(image).expect("只读挂载");
    let root = mounted.effective_root;
    let location = root.tree_table.locations[0];
    let bytes = PoolReader::read(image, location.device, location.slot.to_device_offset(), 16384)
        .expect("读树表");
    let node = parse_index_node(&bytes).expect("解树表");
    let entries = node
        .entries
        .iter()
        .map(|entry| TreeTableEntry::parse(entry).expect("解条目"))
        .collect();
    (root.instance.0, root.checkpoint_txg.0, entries)
}

/// 字面读法：D8 已定项 8「头 ID 8 字节的取值」逐字取 65537。
fn judge_literal(entries: &[TreeTableEntry]) -> Vec<String> {
    entries
        .iter()
        .filter_map(|entry| {
            let expected = match entry.kind {
                TREE_KIND_EXTENT | TREE_KIND_INODE | TREE_KIND_DELETED_INODES
                | TREE_KIND_TOMBSTONE => LITERAL_HEAD,
                _ => 0,
            };
            (entry.head_identifier != expected).then(|| {
                format!("kind{}:tree{}:head{}!={}", entry.kind, entry.tree.0, entry.head_identifier, expected)
            })
        })
        .collect()
}

/// 同表读法：inode 树条目恰好一条；它的头 ID = 它自己的树 ID；extent / deleted_inodes / 墓碑树的头 ID = 那条 inode 树条目的树 ID；其余 0。
fn judge_same_table(entries: &[TreeTableEntry]) -> Vec<String> {
    let inode_entries: Vec<&TreeTableEntry> =
        entries.iter().filter(|entry| entry.kind == TREE_KIND_INODE).collect();
    if inode_entries.len() != 1 {
        return vec![format!("inode_entries={}", inode_entries.len())];
    }
    let head = inode_entries[0].tree.0;
    entries
        .iter()
        .filter_map(|entry| {
            let expected = match entry.kind {
                TREE_KIND_EXTENT | TREE_KIND_INODE | TREE_KIND_DELETED_INODES
                | TREE_KIND_TOMBSTONE => head,
                _ => 0,
            };
            (entry.head_identifier != expected).then(|| {
                format!("kind{}:tree{}:head{}!={}", entry.kind, entry.tree.0, entry.head_identifier, expected)
            })
        })
        .collect()
}

fn violated(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, verdict)| format!("{invariant}:{}", format!("{verdict:?}").chars().take(300).collect::<String>()))
        .collect()
}

fn print_tree_table(label: &str, image: &MemoryPool) {
    let (instance, txg, entries) = tree_table_from_image(image);
    let table = entries
        .iter()
        .map(|entry| format!("{}/{}/{}", entry.kind, entry.tree.0, entry.head_identifier))
        .collect::<Vec<_>>()
        .join(",");
    let literal = judge_literal(&entries);
    let same_table = judge_same_table(&entries);
    println!(
        "name=k1_heads history={label} root=({instance},{txg}) entries={} kind/tree/head={table} literal_red={} literal_detail={} same_table_red={} same_table_detail={} checker_violated={:?}",
        entries.len(),
        literal.len(),
        literal.join(";"),
        same_table.len(),
        same_table.join(";"),
        violated(image)
    );
}

fn slots_of(output: &TransactionOutput) -> (u64, Vec<u64>) {
    let data = output.data_pointers[0].locations[0].slot.0;
    let mut internal: Vec<u64> = output
        .units
        .iter()
        .filter(|unit| !matches!(unit.identity, TransactionUnit::Data(_)))
        .map(|unit| unit.slot.0)
        .collect();
    internal.sort_unstable();
    (data, internal)
}

/// mkfs 同一个进程里那条流（mkfs → 取号 → 暖机两次 → 第一个文件）在给定几何上跑一遍。
fn new_pool_file(
    parameters: &MakeFilesystemParameters,
    image_bytes: u64,
    content: &[u8],
) -> Result<(u64, TransactionOutput, Vec<(DeviceIdentity, Mem)>, singlefs_core::allocator::PoolAllocator), String> {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream, image_bytes);
    let genesis = make_filesystem(parameters, &mut devices).map_err(|error| format!("mkfs:{error:?}"))?;
    let mut allocator = allocator_after_make_filesystem(parameters, &devices, &genesis);
    let output = {
        let mut pool = PoolWriter::new(parameters, &mut devices);
        let instance = acquire_instance(&mut pool).map_err(|error| format!("acquire:{error:?}"))?;
        let warm = warm_up(&mut pool, &genesis.root, instance).map_err(|error| format!("warm:{:?}", error.cause))?;
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机根"),
            FirstFile { content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .map_err(|error| format!("publish:{error:?}"))?
    };
    Ok((genesis.unit_area_start.slot().0, output, devices, allocator))
}

fn parameters_with_ring(ring_bytes: u64) -> MakeFilesystemParameters {
    let mut parameters = parameters();
    parameters.geometry.journal_ring_bytes = ring_bytes;
    parameters
}

fn records_in(image: &MemoryPool) -> Vec<(u64, u64, u64, u64)> {
    let mounted = mount_read_only(image).expect("只读挂载");
    mounted
        .mounted
        .inode_records()
        .iter()
        .map(|record| (record.inode, record.object_birth.0, record.change_count, record.write_time_seconds))
        .collect()
}

/// K1 / K2：崩溃恢复丢掉实例 2 的第一个文件版本（根槽与点名的数据单元一时读不出，现成用例
/// `writable_mount_of_a_formatted_pool.rs` 那一段历史的内存盘版本），实例 3 从环里的水位 65548 发第一个文件版本。
#[test]
fn k1_recovered_first_file_version_issues_trees_from_the_ring_watermark() {
    let stream = SharedStream::retaining_contents();
    let mut devices = memory_devices(&stream, common::IMAGE_BYTES);
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let mkfs_operations = stream.operations().len();
    let after_mkfs = memory_pool_of_sparse_devices(&devices);
    drop(mount_writable(&parameters(), &mut devices).expect("实例 1"));
    let second = mount_writable(&parameters(), &mut devices).expect("实例 2");
    assert_eq!(second.output.instance, InstanceGeneration(2));
    let PoolVersion::WithoutFile(version_two) = &second.current else { panic!("树表 0 条") };
    let mut allocator_two = second.allocator.clone();
    let dropped = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator_two,
            &version_two.root,
            FirstFile { content: &file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            second.output.instance,
            &version_two.record_bytes,
        )
        .expect("实例 2 的第一个文件版本")
    };
    let (dropped_data, dropped_internal) = slots_of(&dropped);
    println!(
        "name=k2_first_file_slots history=instance2_after_a_warm_up_only_mount root=({},{}) data_slot={dropped_data} internal_slots={dropped_internal:?} inode_tree={}",
        dropped.root.instance.0, dropped.root.checkpoint_txg.0, dropped.tree_identifiers.inode.0
    );
    print_tree_table("instance2_first_file_before_the_crash", &memory_pool_of_sparse_devices(&devices));
    let txg = dropped.root.checkpoint_txg;
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations[mkfs_operations..], &geometry());
    println!(
        "name=k1_stream_state_count writes={} segment_lengths={:?} closed_form_state_count={}",
        writes.len(),
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        singlefs_harness::memory_pool::closed_form_state_count(&segments)
    );
    let target = target_for_publish(txg, parameters().geometry.root_ring_slots_per_region);
    let region_device = parameters().region_devices[usize::try_from(target.region).expect("区域")];
    let root_offset = slot_offset(target, parameters().geometry.fixed_structure_slot_spacing);
    let root_write = writes
        .iter()
        .position(|write| write.is_force_unit_access && write.device == region_device && write.offset == root_offset)
        .expect("根槽 FUA");
    let persisted: Vec<bool> = (0..writes.len()).map(|index| index <= root_write).collect();
    let crashed = crash_state_devices(&after_mkfs, &writes, &persisted, &SharedStream::retaining_contents());
    let named = dropped.data_pointers[0].locations[0];
    let unreadable = SharedUnreadableRanges::new(
        vec![
            unreadable_root_slot_of(txg, FailingReadsOfARange::Every),
            UnreadableRange {
                device: named.device,
                offset_in_bytes: named.slot.to_device_offset().0,
                length_in_bytes: singlefs_format::DATA_UNIT_BYTES,
                failing_reads: FailingReadsOfARange::Every,
            },
        ],
        UnreadableRangeReadBack::DeviceError,
    );
    let mut faulty = with_unreadable_ranges(crashed, &unreadable);
    let third = mount_writable(&parameters(), &mut faulty).expect("实例 3");
    let mut reopened = without_unreadable_ranges(faulty);
    let PoolVersion::WithoutFile(version_three) = &third.current else { panic!("落到 (2, 4)") };
    println!(
        "name=k1_recovered_version instance={} selected=({},{}) watermark={}",
        third.output.instance.0,
        version_three.root.instance.0,
        version_three.root.checkpoint_txg.0,
        version_three.root.tree_identifier_watermark
    );
    let mut allocator_three = third.allocator.clone();
    let first_of_three = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator_three,
            &version_three.root,
            FirstFile { content: &file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 },
            third.output.instance,
            &version_three.record_bytes,
        )
        .expect("实例 3 的第一个文件版本")
    };
    let (data, internal) = slots_of(&first_of_three);
    println!(
        "name=k2_first_file_slots history=instance3_after_the_recovery_dropped_instance2s root=({},{}) data_slot={data} internal_slots={internal:?} inode_tree={}",
        first_of_three.root.instance.0, first_of_three.root.checkpoint_txg.0, first_of_three.tree_identifiers.inode.0
    );
    let image = memory_pool_of_sparse_devices(&reopened);
    print_tree_table("instance3_first_file_after_the_recovery", &image);
    // 之后再覆盖写一次：树 ID 一个都不再发，头 ID 照抄。
    let overwritten = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator_three,
            &first_of_three,
            FirstFile { content: &file_content()[..2000], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180 },
            third.output.instance,
        )
        .expect("覆盖写")
    };
    assert_eq!(overwritten.root.checkpoint_txg.0, first_of_three.root.checkpoint_txg.0 + 1);
    print_tree_table("instance3_overwrite_after_the_recovery", &memory_pool_of_sparse_devices(&reopened));
    // 变体 a：被丢掉的 (2, 5) 那条根的根槽此后一直读不出（永久故障，镜像里清零）。
    let dropped_root = unreadable_root_slot_of(txg, FailingReadsOfARange::Every);
    let mut permanently = memory_pool_of_sparse_devices(&reopened);
    permanently
        .devices
        .get_mut(&dropped_root.device)
        .expect("盘")
        .write(singlefs_core::address::DeviceOffsetInBytes(dropped_root.offset_in_bytes), &vec![0u8; 512]);
    print_tree_table("instance3_overwrite_with_the_dropped_root_slot_left_unreadable", &permanently);
    // 变体 b：故障全部解除，接着覆盖写直到 (2, 5) 那一槽被新根盖掉（根环 3 × 8 = 24 槽）。
    let mut previous = overwritten;
    while previous.root.checkpoint_txg.0 < txg.0 + 24 {
        let length = 1000 + usize::try_from(previous.root.checkpoint_txg.0).expect("小");
        let next = {
            let publish_parameters = parameters();
            let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut allocator_three,
                &previous,
                FirstFile { content: &file_content()[..length], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 240 },
                third.output.instance,
            )
            .expect("覆盖写")
        };
        previous = next;
    }
    println!("name=k1_ring_turned last_txg={}", previous.root.checkpoint_txg.0);
    print_tree_table("instance3_after_the_ring_turned_past_the_dropped_root", &memory_pool_of_sparse_devices(&reopened));
}

/// K1 / K3：新池新建文件那条流之后接覆盖写、建 inode、顺序写，每一版的头 ID 与每条 inode 记录的写入时间。
#[test]
fn k1_k3_standard_history_heads_and_write_times() {
    let parameters = parameters();
    let (_, first, mut devices, mut allocator) =
        new_pool_file(&parameters, common::IMAGE_BYTES, &file_content()).expect("新池新建文件");
    let instance = first.root.instance;
    print_tree_table("new_pool_file", &memory_pool_of_sparse_devices(&devices));
    println!("name=k3_write_times step=new_pool_file parameter={FIXED_WRITE_TIME_SECONDS} records={:?}", records_in(&memory_pool_of_sparse_devices(&devices)));
    let overwrite = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(&mut writer, &mut allocator, &first,
            FirstFile { content: &file_content()[..1000], write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 }, instance)
        .expect("覆盖写")
    };
    println!("name=k3_write_times step=overwrite parameter={} records={:?}", FIXED_WRITE_TIME_SECONDS + 60, records_in(&memory_pool_of_sparse_devices(&devices)));
    let new_inodes = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_new_inodes(&mut writer, &mut allocator, &overwrite, 3, FIXED_WRITE_TIME_SECONDS + 120, instance)
            .expect("建 3 个 inode")
    };
    print_tree_table("new_pool_file_then_overwrite_then_three_new_inodes", &memory_pool_of_sparse_devices(&devices));
    println!("name=k3_write_times step=new_inodes parameter={} records={:?}", FIXED_WRITE_TIME_SECONDS + 120, records_in(&memory_pool_of_sparse_devices(&devices)));
    let long_content: Vec<u8> = (0..70_000usize).map(|index| u8::try_from(index % 241).expect("小")).collect();
    let _sequential = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_sequential_write(&mut writer, &mut allocator, &new_inodes,
            FirstFile { content: &long_content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180 }, instance)
        .expect("顺序写")
    };
    println!("name=k3_write_times step=sequential_write parameter={} records={:?}", FIXED_WRITE_TIME_SECONDS + 180, records_in(&memory_pool_of_sparse_devices(&devices)));
    print_tree_table("then_sequential_write", &memory_pool_of_sparse_devices(&devices));
}

/// K2：新池新建文件的落点在环长、镜像大小、文件长度上扫一遍；甲的认法与两种判法逐格打印。
#[test]
fn k2_new_pool_file_placement_matrix() {
    let mib = 1u64 << 20;
    let geometries: [(u64, u64); 6] = [
        (128 * mib, 4 << 30),
        (256 * mib, 4 << 30),
        (768 * mib, 4 << 30),
        (1024 * mib, 4 << 30),
        (768 * mib, 8 << 30),
        (1984 * mib, 8 << 30),
    ];
    let lengths: [usize; 5] = [0, 1, 3000, 32634, 32768];
    let mut default_relative: Option<Vec<u64>> = None;
    for (ring, image_bytes) in geometries {
        for length in lengths {
            let content: Vec<u8> = (0..length).map(|index| u8::try_from(index % 251).expect("小")).collect();
            match new_pool_file(&parameters_with_ring(ring), image_bytes, &content) {
                Err(error) => println!("name=k2_matrix ring_mib={} image_gib={} length={length} outcome=refused error={}", ring / mib, image_bytes >> 30, error.chars().take(160).collect::<String>()),
                Ok((start, output, devices, _)) => {
                    let (data, internal) = slots_of(&output);
                    let relative: Vec<u64> = internal.iter().map(|slot| slot - start).collect();
                    if ring == 768 * mib && image_bytes == 4 << 30 && length == 3000 {
                        default_relative = Some(relative.clone());
                    }
                    let records = records_in(&memory_pool_of_sparse_devices(&devices)).len();
                    let recognized = output.root.instance.0 == 1
                        && output.root.checkpoint_txg.0 == 3
                        && start == 50176
                        && output.data_pointers.len() == 1
                        && records == 1;
                    println!(
                        "name=k2_matrix ring_mib={} image_gib={} length={length} outcome=published root=({},{}) unit_area_start={start} data_slot={data} data_minus_start={} data_units={} records={records} internal_minus_start={relative:?} recognized_by_jia={recognized} judge_50180_red={} judge_start_plus_4_red={}",
                        ring / mib, image_bytes >> 30,
                        output.root.instance.0, output.root.checkpoint_txg.0,
                        data - start, output.data_pointers.len(),
                        recognized && data != 50180,
                        data != start + 4
                    );
                }
            }
        }
    }
    println!("name=k2_matrix_default_internal_minus_start value={default_relative:?}");
}
