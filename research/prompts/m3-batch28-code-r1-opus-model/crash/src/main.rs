//! m3-batch28-code-r1 攻方原型：合法历史的崩溃状态（只在仓副本上编、跑；不入库代码）。
//! 起点池建一次（内存稀疏盘），每段历史从内存里那一份拷；历史整段录制，录制流按层 0 的枚举域取崩溃状态——调
//! `singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state`，`expand` 恒真（每一段都展开，与
//! `enumerate_layer0_versions` 同一个域），它在每个崩溃镜像上跑恢复与池级 checker；观察者收下每个状态的持久集合，之后每个状态再做一次
//! 可写挂载（崩溃恢复）、挂载之后的镜像跑池级 checker，再建一个号、再跑一次。
//! checker 用的是 tree-probe（walk.rs 加了只计数的探针 `opus_probe_counts`），每段历史前后各读一次计数，差就是这段历史里各支走到了几次。
//! 用户动作（建 1 / 2 个号、删一个活号、回退到前面某一版、再挂）按种子随机抽，只固定起点。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../harness_tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, opus_probe_counts};
use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_unlink_intent,
    publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::memory_pool::{
    closed_form_state_count, root_identity_of_write, writes_and_segments, MemoryPool, PublishedVersion,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const WT: u64 = 1_788_000_100;
const PROBE_NAMES: [&str; 9] = [
    "calls", "other_class_returned", "copy_same_class", "copy_other_class", "hb_inode", "hb_container", "hb_red",
    "location_checksums_differ", "copy_unverified",
];

fn new_device() -> SparseBlockDevice {
    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

/// 一个池：设备、现行那一版、分配器、活着的号。
struct Live<D: BlockDevice> {
    devices: Vec<(DeviceIdentity, D)>,
    output: TransactionOutput,
    allocator: PoolAllocator,
    live: Vec<u64>,
    next_inode: u64,
}

impl<D: BlockDevice> Live<D> {
    fn instance(&self) -> InstanceGeneration {
        self.output.root.instance
    }
    fn mark(&self) -> (InstanceGeneration, u64, Vec<u64>, u64) {
        (self.instance(), self.output.root.checkpoint_txg.0, self.live.clone(), self.next_inode)
    }
    fn create(&mut self, count: u64) -> Result<(), String> {
        self.drain()?;
        let parameters = parameters();
        let previous = self.output.clone();
        let instance = self.instance();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        self.output = publish_new_inodes(&mut writer, &mut self.allocator, &previous, count, WT, instance)
            .map_err(|e| format!("{e:?}"))?;
        for _ in 0..count {
            self.live.push(self.next_inode);
            self.next_inode += 1;
        }
        Ok(())
    }
    fn delete(&mut self, inode: u64) -> Result<(), String> {
        self.drain()?;
        let parameters = parameters();
        let previous = self.output.clone();
        let instance = self.instance();
        {
            let mut writer = PoolWriter::new(&parameters, &mut self.devices);
            self.output = publish_unlink_intent(&mut writer, &mut self.allocator, &previous, InodeNumber(inode), WT, instance)
                .map_err(|e| format!("{e:?}"))?;
        }
        self.live.retain(|n| *n != inode);
        self.drain()
    }
    fn drain(&mut self) -> Result<(), String> {
        if self.output.deleted_inodes.is_empty() {
            return Ok(());
        }
        let parameters = parameters();
        let instance = self.instance();
        let mut current = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        publish_unlink_steps_until_the_queue_is_empty(&mut writer, &mut self.allocator, &mut current, UnlinkBatchSize::OneExtentLowerLeaf, instance)
            .map_err(|e| format!("{e:?}"))?;
        self.output = current;
        Ok(())
    }
    fn rollback(&mut self, target: &(InstanceGeneration, u64, Vec<u64>, u64)) -> Result<(), String> {
        let parameters = parameters();
        roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget { instance: target.0, checkpoint_txg: CheckpointTxg(target.1) },
        )
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
        self.live = target.2.clone();
        Ok(())
    }
    fn remount(&mut self) -> Result<(), String> {
        let parameters = parameters();
        let mounted = mount_writable(&parameters, &mut self.devices).map_err(|e| format!("{e:?}"))?;
        self.allocator = mounted.allocator.clone();
        self.output = mounted.current.version_of_trees().ok_or("树表 0 条")?.clone();
        Ok(())
    }
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    for (identity, device) in devices {
        image.devices.insert(*identity, device.image.clone());
    }
    image
}

fn sparse_devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = new_device();
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

fn violated_names(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then_some(invariant))
        .collect()
}

/// mkfs 之后、新池新建文件之前的那一刻：设备、分配器、取到的实例、暖机输出（录「新池新建文件」那条流从这里起）。
struct BeforeFirstFile {
    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: PoolAllocator,
    genesis_root: singlefs_core::root_record::RootRecord,
}

fn mkfs() -> BeforeFirstFile {
    let parameters = parameters();
    let mut devices = vec![(DeviceIdentity(0), new_device()), (DeviceIdentity(1), new_device())];
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(&parameters, &genesis));
    BeforeFirstFile { devices, allocator, genesis_root: genesis.root }
}

/// 取号、暖机、新池新建文件（在 `devices` 上做，录不录由调用方的设备定）。
fn first_file<D: BlockDevice>(
    devices: &mut Vec<(DeviceIdentity, D)>,
    allocator: &mut PoolAllocator,
    genesis_root: &singlefs_core::root_record::RootRecord,
) -> TransactionOutput {
    let parameters = parameters();
    let content = file_content();
    let mut pool = PoolWriter::new(&parameters, devices);
    let instance = acquire_instance(&mut pool).expect("取号");
    let warm = warm_up(&mut pool, genesis_root, instance).expect("暖机");
    publish_first_file(
        &mut pool,
        allocator,
        warm.roots.last().expect("暖机两代根"),
        FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
        instance,
        &warm.last_record_bytes,
    )
    .expect("新池新建文件")
}

#[derive(Clone, Copy, Debug)]
enum Act {
    Create(u64),
    Delete(usize),
    Rollback(usize),
    Remount,
}

struct SplitMix(u64);
impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
}

/// 一段历史的动作（种子定）：建 1 / 2 个号、删第 k 个活号、回退到前面第 k 个记下的版本、再挂。
fn random_acts(seed: u64, steps: usize) -> Vec<Act> {
    let mut rng = SplitMix(seed);
    (0..steps)
        .map(|_| match rng.below(10) {
            0..=2 => Act::Create(1 + rng.below(2)),
            3..=5 => Act::Delete(rng.below(8) as usize),
            6..=7 => Act::Rollback(rng.below(8) as usize),
            _ => Act::Remount,
        })
        .collect()
}

/// 在 `live` 上做一个动作；`marks` 是这段历史里记下的版本（起点那一版在第 0 个）。交回动作的短名（做不成的写 refused）。
fn apply<D: BlockDevice>(live: &mut Live<D>, act: Act, marks: &mut Vec<(InstanceGeneration, u64, Vec<u64>, u64)>) -> String {
    let outcome = match act {
        Act::Create(count) => live.create(count).map(|()| format!("C{count}")),
        Act::Delete(k) => {
            if live.live.is_empty() {
                Err("没有活号".to_string())
            } else {
                let inode = live.live[k % live.live.len()];
                live.delete(inode).map(|()| format!("D{inode}"))
            }
        }
        Act::Rollback(k) => {
            let target = marks[k % marks.len()].clone();
            live.rollback(&target).map(|()| format!("R{}t{}", target.0 .0, target.1))
        }
        Act::Remount => live.remount().map(|()| "M".to_string()),
    };
    match outcome {
        Ok(name) => {
            marks.push(live.mark());
            name
        }
        Err(reason) => format!("x({})", reason.chars().take_while(|c| c.is_alphanumeric()).collect::<String>()),
    }
}

/// 崩溃恢复（可写挂载）之后判一次；挂得上的再建一个号再判一次。
fn after_recovery(image: &MemoryPool) -> (String, Vec<&'static str>, Vec<&'static str>) {
    let parameters = parameters();
    let mut devices = sparse_devices_of(image);
    let mounted = match mount_writable(&parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            let kind: String = format!("{error:?}").chars().take_while(|c| c.is_alphanumeric()).collect();
            return (format!("mount-refused:{kind}"), Vec::new(), Vec::new());
        }
    };
    let Some(output) = mounted.current.version_of_trees().cloned() else {
        return ("mounted-without-trees".to_string(), violated_names(&image_of(&devices)), Vec::new());
    };
    let after_mount = violated_names(&image_of(&devices));
    let mut live = Live { devices, output, allocator: mounted.allocator.clone(), live: Vec::new(), next_inode: 0 };
    let after_create = match live.create(1) {
        Ok(()) => violated_names(&image_of(&live.devices)),
        Err(_) => return ("mounted;create-refused".to_string(), after_mount, Vec::new()),
    };
    ("mounted".to_string(), after_mount, after_create)
}

fn crash_image(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut image = base.clone();
    let selected: Vec<RetainedWrite> =
        writes.iter().zip(persisted).filter(|(_, kept)| **kept).map(|(write, _)| write.clone()).collect();
    image.apply_writes(&selected);
    image
}

fn probe_delta(before: [u64; 9]) -> String {
    let after = opus_probe_counts();
    PROBE_NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| format!("{name}={}", after[i] - before[i]))
        .collect::<Vec<_>>()
        .join(",")
}

/// 一条录好的流：在每个崩溃状态上跑层 0 的枚举（恢复 + 池级 checker），再在每个状态上做可写挂载、判、建一个号、判。
/// 交回这条流的状态数（没跑的交 0）。
fn judge_flow(name: &str, base_image: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>], base_output: &TransactionOutput, count_only: bool, threads: usize) -> u64 {
    let states = closed_form_state_count(segments);
    println!(
        "RESULT crash {name} writes={} segments={} longest_segment={} closed_form_states={states}",
        writes.len(),
        segments.len(),
        segments.iter().map(Vec::len).max().unwrap_or(0)
    );
    if count_only {
        return 0;
    }
    if states > 1_000_000 {
        println!("RESULT crash {name} skipped: closed_form_states {states} > 1e6");
        return 0;
    }
    let mut versions: Vec<PublishedVersion> = vec![PublishedVersion {
        instance: base_output.root.instance,
        checkpoint_txg: base_output.root.checkpoint_txg,
        content: file_content(),
    }];
    let mut judged_root_index = 0;
    for (write_index, write) in writes.iter().enumerate() {
        if write.kind == StepKind::RootRecordFua {
            let (instance, checkpoint_txg) = root_identity_of_write(write);
            if !versions.iter().any(|v| v.instance == instance && v.checkpoint_txg == checkpoint_txg) {
                versions.push(PublishedVersion { instance, checkpoint_txg, content: file_content() });
            }
            judged_root_index = write_index;
        }
    }
    let before = opus_probe_counts();
    let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
    let started = std::time::Instant::now();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        base_image,
        writes,
        segments,
        judged_root_index,
        &versions,
        &|_segment_index, _segment| true,
        &mut |image, _report| persisted_sets.push(image.persisted.clone()),
    );
    let mut checker_reds: Vec<String> = tally
        .checker_violated_states
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(invariant, count)| format!("{invariant}:{count}"))
        .collect();
    checker_reds.sort();
    println!(
        "RESULT crash {name} layer0 states={} oracle_violations={} failed_states={} checker_red_states={checker_reds:?} probe[{}] enumerate_seconds={:.0}",
        tally.states,
        tally.violations,
        tally.failed_states,
        probe_delta(before),
        started.elapsed().as_secs_f64()
    );
    for (invariant, count) in tally.checker_violated_states.iter().filter(|(_, count)| **count > 0) {
        println!(
            "DETAIL crash {name} layer0 checker {invariant} red_states={count} first={}",
            tally.checker_first_violation.get(invariant).map(|text| text.chars().take(200).collect::<String>()).unwrap_or_default()
        );
    }
    if std::env::var("OPUS_AFTER_RECOVERY").is_err() {
        return tally.states;
    }
    let before = opus_probe_counts();
    let started = std::time::Instant::now();
    let chunk = persisted_sets.len().div_ceil(threads.max(1)).max(1);
    let results: Vec<(String, Vec<&'static str>, Vec<&'static str>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = persisted_sets
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || part.iter().map(|persisted| after_recovery(&crash_image(base_image, writes, persisted))).collect::<Vec<_>>())
            })
            .collect();
        handles.into_iter().flat_map(|handle| handle.join().expect("线程")).collect()
    });
    let mut outcome_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut red_after_mount: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut red_after_create: BTreeMap<&'static str, u64> = BTreeMap::new();
    for (outcome, mount_reds, create_reds) in &results {
        *outcome_counts.entry(outcome.clone()).or_insert(0) += 1;
        for invariant in mount_reds {
            *red_after_mount.entry(invariant).or_insert(0) += 1;
        }
        for invariant in create_reds {
            *red_after_create.entry(invariant).or_insert(0) += 1;
        }
    }
    println!(
        "RESULT crash {name} recovered states={} outcomes={outcome_counts:?} red_after_mount={red_after_mount:?} red_after_create={red_after_create:?} probe[{}] seconds={:.0}",
        results.len(),
        probe_delta(before),
        started.elapsed().as_secs_f64()
    );
    tally.states
}

/// 录一段：从 `start` 那份镜像拷出录制设备，在上面依次做 `acts`；交回 (流的写、段、动作短名)。
fn record_history(
    start_image: &MemoryPool,
    start_output: &TransactionOutput,
    start_allocator: &PoolAllocator,
    start_live: &[u64],
    start_next: u64,
    acts: &[Act],
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, Vec<String>) {
    let stream = SharedStream::retaining_contents();
    let devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = sparse_devices_of(start_image)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let mut live = Live {
        devices,
        output: start_output.clone(),
        allocator: start_allocator.clone(),
        live: start_live.to_vec(),
        next_inode: start_next,
    };
    let mut marks = vec![live.mark()];
    let names: Vec<String> = acts.iter().map(|act| apply(&mut live, *act, &mut marks)).collect();
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &geometry());
    (writes, segments, names)
}

fn main() {
    let count_only = std::env::var("OPUS_COUNT_ONLY").is_ok();
    let threads: usize = std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let mode = std::env::var("OPUS_MODE").unwrap_or_else(|_| "legal".to_string());
    let mut total_states = 0u64;
    if mode == "it" {
        // J1 / CPU 与 GPU：新池新建文件那条流，发布那条根（txg 3）的实例表指针头部某一段加 1、重封根记录（宽校验和 138..170）。
        for (field_name, field) in [("none", None), ("birth_txg", Some(42usize)), ("instance", Some(78)), ("tail", Some(82))] {
            let before = mkfs();
            let base_image = image_of(&before.devices);
            let stream = SharedStream::retaining_contents();
            let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = sparse_devices_of(&base_image)
                .into_iter()
                .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
                .collect();
            let mut allocator = before.allocator.clone();
            let output = first_file(&mut devices, &mut allocator, &before.genesis_root);
            let operations = stream.retained_operations();
            let (mut writes, segments) = writes_and_segments(&operations, &geometry());
            let root_write = writes
                .iter()
                .rposition(|write| write.kind == StepKind::RootRecordFua)
                .expect("有根槽写");
            if let Some(field) = field {
                let singlefs_harness::memory_pool::WrittenContents::Bytes(bytes) = &mut writes[root_write].contents else {
                    panic!("根槽写是普通写");
                };
                assert_eq!(u64::from_le_bytes(bytes[28..36].try_into().expect("8")), output.root.checkpoint_txg.0, "改的是发布那条根");
                let at = 170 + field;
                bytes[at] = bytes[at].wrapping_add(1);
                let digest = wide_checksum_with_field_zeroed(&bytes[..512], 512, 138);
                bytes[138..170].copy_from_slice(&digest);
            }
            let genesis_output_for_versions = output.clone();
            total_states += judge_flow(&format!("IT-{field_name}"), &base_image, &writes, &segments, &genesis_output_for_versions, count_only, threads);
        }
        println!("RESULT crash mode=it total_enumerated_states={total_states}");
        return;
    }
    // 起点（各建一次，内存稀疏盘，不录）：S0 = 新池新建文件 → 建 2、3 号；SD = S0 之后删 2 号（墓碑树有容器）；
    // SL = SD 之后回退到 S0 那一版（谱系树有容器）。
    let before = mkfs();
    let mut devices = before.devices;
    let mut allocator = before.allocator;
    let output = first_file(&mut devices, &mut allocator, &before.genesis_root);
    let mut s0 = Live { devices, output, allocator, live: vec![1], next_inode: 2 };
    s0.create(2).expect("建 2、3 号");
    let s0_mark = s0.mark();
    let mut starts: Vec<(&str, MemoryPool, TransactionOutput, PoolAllocator, Vec<u64>, u64)> = Vec::new();
    starts.push(("S0", image_of(&s0.devices), s0.output.clone(), s0.allocator.clone(), s0.live.clone(), s0.next_inode));
    let mut sd = Live { devices: sparse_devices_of(&image_of(&s0.devices)), output: s0.output.clone(), allocator: s0.allocator.clone(), live: s0.live.clone(), next_inode: s0.next_inode };
    sd.delete(2).expect("删 2 号");
    starts.push(("SD", image_of(&sd.devices), sd.output.clone(), sd.allocator.clone(), sd.live.clone(), sd.next_inode));
    let mut sl = Live { devices: sparse_devices_of(&image_of(&sd.devices)), output: sd.output.clone(), allocator: sd.allocator.clone(), live: sd.live.clone(), next_inode: sd.next_inode };
    sl.rollback(&s0_mark).expect("回退到 S0");
    starts.push(("SL", image_of(&sl.devices), sl.output.clone(), sl.allocator.clone(), sl.live.clone(), sl.next_inode));
    let which: Vec<String> = std::env::var("OPUS_STARTS").unwrap_or_else(|_| "S0,SD,SL".to_string()).split(',').map(str::to_string).collect();
    let seed_from: u64 = std::env::var("OPUS_SEED_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let histories: u64 = std::env::var("OPUS_HISTORIES").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let steps: usize = std::env::var("OPUS_STEPS").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
    let state_budget: u64 = std::env::var("OPUS_STATE_BUDGET").ok().and_then(|v| v.parse().ok()).unwrap_or(1_000_000);
    if mode == "final" {
        // 不枚举崩溃状态：每段历史在内存稀疏盘上照做（不录），每做完一步对那一刻的镜像跑一次池级 checker，看探针各支走到几次、红了什么。
        // 用户动作（建 1 / 2 个号、删一个活号、回退到这段历史里记下的任一版、再挂）按种子随机抽，只固定起点。
        for (start_name, image, output, allocator, live, next) in &starts {
            if !which.iter().any(|w| w == start_name) {
                continue;
            }
            let before = opus_probe_counts();
            let mut checked = 0u64;
            let mut red_counts: BTreeMap<&'static str, u64> = BTreeMap::new();
            let mut act_counts: BTreeMap<char, u64> = BTreeMap::new();
            for seed in seed_from..seed_from + histories {
                let mut pool = Live {
                    devices: sparse_devices_of(image),
                    output: output.clone(),
                    allocator: allocator.clone(),
                    live: live.clone(),
                    next_inode: *next,
                };
                let mut marks = vec![pool.mark()];
                for act in random_acts(seed, steps) {
                    let name = apply(&mut pool, act, &mut marks);
                    *act_counts.entry(name.chars().next().unwrap_or('?')).or_insert(0) += 1;
                    let reds = violated_names(&image_of(&pool.devices));
                    checked += 1;
                    if !reds.is_empty() {
                        println!("DETAIL final {start_name}-s{seed} after {name} reds={reds:?}");
                    }
                    for invariant in reds {
                        *red_counts.entry(invariant).or_insert(0) += 1;
                    }
                }
            }
            println!(
                "RESULT final start={start_name} seeds={seed_from}..{} steps={steps} checked_images={checked} acts={act_counts:?} reds={red_counts:?} probe[{}]",
                seed_from + histories,
                probe_delta(before)
            );
        }
        return;
    }
    for (start_name, image, output, allocator, live, next) in &starts {
        if !which.iter().any(|w| w == start_name) {
            continue;
        }
        for seed in seed_from..seed_from + histories {
            let acts = random_acts(seed, steps);
            let (writes, segments, names) = record_history(image, output, allocator, live, *next, &acts);
            let name = format!("{start_name}-s{seed}[{}]", names.join(" "));
            let states = closed_form_state_count(&segments);
            if !count_only && total_states + states > state_budget {
                println!("RESULT crash {name} skipped: budget {total_states} + {states} > {state_budget}");
                continue;
            }
            total_states += judge_flow(&name, image, &writes, &segments, output, count_only, threads);
        }
    }
    println!("RESULT crash mode=legal total_enumerated_states={total_states}");
}
