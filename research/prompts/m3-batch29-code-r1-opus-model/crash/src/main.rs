//! m3-batch29-code-r1 攻方原型：删除批几条合法历史的崩溃状态（只在攻方仓副本上编、跑；不入库代码）。
//! 起点池在内存稀疏盘上建一次（mkfs → 取号 → 暖机 → 新池新建文件 → 建 4 个空 inode），每段历史从内存里那一份拷；
//! 历史整段录制，录制流按层 0 的枚举域取崩溃状态：调 `singlefs_checker_tier::crash::enumerate_layer0_versions`（每一段都展开），
//! 它在每个崩溃镜像上跑恢复与池级 checker。checker 是攻方仓副本里打了只计数探针的那份 walk.rs（`opus_b29_probe_counts`），
//! 每段历史前后各读一次计数，差就是这段历史的全部崩溃状态里各支走到了几次。
//! 用户动作（删哪个号、删完回不回退、回退到哪一版、建几个号、墓碑容器压不压小）按清单逐段放开扫，只固定起点。
//! 环境变量：OPUS_COUNT_ONLY=1 只按 closed_form_state_count 数状态；OPUS_ONLY=<名,名> 只跑这几段；OPUS_STATE_BUDGET 一段的状态上限（默认 100000）。
#![allow(clippy::all)]

use singlefs_checker::walk::opus_b29_probe_counts;
use singlefs_checker_tier::crash::enumerate_layer0_versions;
use singlefs_core::address::{DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::container_tree::{
    ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests,
};
use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::tombstone_tree::{TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_unlink_intent,
    publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{
    closed_form_state_count, root_identity_of_write, writes_and_segments, MemoryPool, PublishedVersion,
    SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

const IMAGE_BYTES: u64 = 4 << 30;
const WRITE_TIME: u64 = 1_788_000_100;
const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const PROBE_NAMES: [&str; 6] = ["hv", "hb_inode", "hb_container", "unwalked", "unwalked_mismatch", "l1_type0"];

type Device = RecordingBlockDevice<SparseBlockDevice>;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: [0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00],
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
    (0..3000usize).map(|index| u8::try_from(index % 251).expect("小于 256")).collect()
}

fn capped() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(TombstoneRecordsPerContainerCappedForTests::new(1).expect("1")),
        lineage_container: LineageContainerCapacity::CappedForTests(LineageRecordsPerContainerCappedForTests::new(1).expect("1")),
        node_entries: ContainerTreeNodeEntries::CappedForTests(ContainerTreeNodeEntriesCappedForTests::new(2).expect("2")),
    }
}

/// 一个池：录着的稀疏盘、现行那一版、分配器、这段历史里发过的每一版（回退的目标从这里取）。
struct Pool {
    devices: Vec<(DeviceIdentity, Device)>,
    stream: SharedStream,
    output: TransactionOutput,
    allocator: PoolAllocator,
    versions: Vec<TransactionOutput>,
}

fn start() -> (Pool, usize) {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, Device)> = (0..2u32)
        .map(|number| {
            let identity = DeviceIdentity(number);
            (identity, RecordingBlockDevice::with_shared_stream(identity, SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)), stream.clone()))
        })
        .collect();
    let parameters = parameters();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES), DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES)]);
    allocator.mark_format_time_units(Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }, Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 });
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
    let content = file_content();
    let first = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(&mut writer, &mut allocator, warm.roots.last().expect("暖机两代根"), FirstFile { content: &content, write_time_seconds: WRITE_TIME }, instance, &warm.last_record_bytes)
            .expect("新池新建文件")
    };
    let created = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        publish_new_inodes(&mut writer, &mut allocator, &first, 4, WRITE_TIME, INSTANCE).expect("建 4 个空 inode")
    };
    let operation_count = stream.retained_operations().len();
    (Pool { devices, stream, output: created.clone(), allocator, versions: vec![first, created] }, operation_count)
}

#[derive(Clone, Copy, Debug)]
enum Act {
    /// 把这个号 unlink 到底（意图 + 各批），墓碑容器压不压小。
    Unlink(u64, bool),
    /// 只发意图。
    Intent(u64),
    /// 回退到这段历史里倒数第几版（1 = 上一版）。
    RollBack(usize),
    /// 建几个空 inode。
    Create(u64),
}

fn apply(pool: &mut Pool, act: Act) -> String {
    let parameters = parameters();
    match act {
        Act::Unlink(inode, capped_containers) => {
            let mut writer = PoolWriter::new(&parameters, &mut pool.devices);
            if capped_containers {
                writer.set_container_tree_capacities(capped());
            }
            let intent = publish_unlink_intent(&mut writer, &mut pool.allocator, &pool.output, InodeNumber(inode), WRITE_TIME, INSTANCE).expect("意图");
            let mut current = intent.clone();
            publish_unlink_steps_until_the_queue_is_empty(&mut writer, &mut pool.allocator, &mut current, UnlinkBatchSize::OneExtentLowerLeaf, INSTANCE).expect("各批");
            pool.versions.push(intent);
            pool.output = current.clone();
            pool.versions.push(current);
            format!("U{inode}{}", if capped_containers { "c" } else { "" })
        }
        Act::Intent(inode) => {
            let mut writer = PoolWriter::new(&parameters, &mut pool.devices);
            let intent = publish_unlink_intent(&mut writer, &mut pool.allocator, &pool.output, InodeNumber(inode), WRITE_TIME, INSTANCE).expect("意图");
            pool.output = intent.clone();
            pool.versions.push(intent);
            format!("I{inode}")
        }
        Act::RollBack(back) => {
            let target = pool.versions[pool.versions.len() - 1 - back].clone();
            roll_back_by_a_forward_publish(&parameters, &mut pool.devices, &mut pool.allocator, &mut pool.output, RollbackTarget { instance: target.root.instance, checkpoint_txg: target.root.checkpoint_txg })
                .expect("回退");
            pool.versions.push(pool.output.clone());
            format!("R{back}")
        }
        Act::Create(count) => {
            let mut writer = PoolWriter::new(&parameters, &mut pool.devices);
            let created = publish_new_inodes(&mut writer, &mut pool.allocator, &pool.output, count, WRITE_TIME, INSTANCE).expect("建号");
            pool.output = created.clone();
            pool.versions.push(created);
            format!("N{count}")
        }
    }
}

/// 起点之后放开扫的用户动作：长度 1 与 2 的序列，加「删 → 回退 → 再删」那一条长度 3。
fn histories() -> Vec<Vec<Act>> {
    let firsts = [Act::Unlink(2, false), Act::Unlink(2, true), Act::Intent(2), Act::Create(1)];
    let mut out: Vec<Vec<Act>> = firsts.iter().map(|act| vec![*act]).collect();
    for first in firsts {
        for second in [Act::Unlink(3, false), Act::Unlink(3, true), Act::RollBack(1), Act::RollBack(2), Act::Create(2)] {
            if matches!(first, Act::Intent(_)) && matches!(second, Act::Unlink(..)) {
                continue; // 队列没走空不许发下一个意图
            }
            out.push(vec![first, second]);
        }
    }
    out.push(vec![Act::Unlink(2, false), Act::RollBack(2), Act::Unlink(2, false)]);
    out.push(vec![Act::Unlink(2, true), Act::Unlink(3, true), Act::Unlink(4, true)]);
    out
}

fn main() {
    let count_only = std::env::var("OPUS_COUNT_ONLY").is_ok();
    let only: Option<Vec<String>> = std::env::var("OPUS_ONLY").ok().map(|text| text.split(',').map(str::to_string).collect());
    let budget: u64 = std::env::var("OPUS_STATE_BUDGET").ok().and_then(|text| text.parse().ok()).unwrap_or(100_000);
    let (start_pool, start_operations) = start();
    let start_stream = start_pool.stream.retained_operations();
    let base = {
        let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        image.apply(&start_stream[..start_operations]);
        image
    };
    let mut total = 0u64;
    for acts in histories() {
        // 每段历史从起点那一份内存池拷：录制设备包着的稀疏盘、分配器、现行版本都拷一份，新流从空开始录
        let stream = SharedStream::retaining_contents();
        let devices: Vec<(DeviceIdentity, Device)> = base
            .devices
            .iter()
            .map(|(identity, image)| {
                let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
                device.image = image.clone();
                (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
            })
            .collect();
        let mut pool = Pool { devices, stream: stream.clone(), output: start_pool.output.clone(), allocator: start_pool.allocator.clone(), versions: start_pool.versions.clone() };
        let names: Vec<String> = acts.iter().map(|act| apply(&mut pool, *act)).collect();
        let name = names.join("-");
        if only.as_ref().is_some_and(|list| !list.contains(&name)) {
            continue;
        }
        let operations: Vec<RetainedOperation> = stream.retained_operations();
        let (writes, segments) = writes_and_segments(&operations, &geometry());
        let states = closed_form_state_count(&segments);
        println!("RESULT crash {name} writes={} segments={} longest_segment={} closed_form_states={states}", writes.len(), segments.len(), segments.iter().map(Vec::len).max().unwrap_or(0));
        if count_only {
            continue;
        }
        if states > budget {
            println!("RESULT crash {name} not-run closed_form_states={states} > budget {budget}");
            continue;
        }
        let mut versions = vec![PublishedVersion { instance: start_pool.output.root.instance, checkpoint_txg: start_pool.output.root.checkpoint_txg, content: file_content() }];
        let mut judged_root_index = 0;
        for (index, write) in writes.iter().enumerate() {
            if write.kind == StepKind::RootRecordFua {
                let (instance, checkpoint_txg) = root_identity_of_write(write);
                if !versions.iter().any(|v| v.instance == instance && v.checkpoint_txg == checkpoint_txg) {
                    versions.push(PublishedVersion { instance, checkpoint_txg, content: file_content() });
                }
                judged_root_index = index;
            }
        }
        let before = opus_b29_probe_counts();
        let started = std::time::Instant::now();
        let tally = enumerate_layer0_versions(&base_of(&base, &start_pool), &writes, &segments, judged_root_index, &versions);
        let after = opus_b29_probe_counts();
        let probe: Vec<String> = PROBE_NAMES.iter().enumerate().map(|(i, probe_name)| format!("{probe_name}={}", after[i] - before[i])).collect();
        let mut reds: Vec<String> = tally.checker_violated_states.iter().filter(|(_, count)| **count > 0).map(|(invariant, count)| format!("{invariant}:{count}")).collect();
        reds.sort();
        println!(
            "RESULT crash {name} layer0 states={} oracle_violations={} failed_states={} checker_red_states={reds:?} probe[{}] seconds={:.0}",
            tally.states, tally.violations, tally.failed_states, probe.join(","), started.elapsed().as_secs_f64()
        );
        for (invariant, first) in &tally.checker_first_violation {
            println!("DETAIL crash {name} {invariant} first={}", first.chars().take(240).collect::<String>());
        }
        total += tally.states;
    }
    println!("RESULT crash total_states_run={total}");
}

/// 起点镜像：起点那段流施加到空内存盘上（与录制设备里的稀疏盘同一份字节）。
fn base_of(base: &MemoryPool, _start: &Pool) -> MemoryPool {
    base.clone()
}
