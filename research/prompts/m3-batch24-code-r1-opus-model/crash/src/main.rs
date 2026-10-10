//! m3-batch24-code-r1 攻方原型之二：N3 在崩溃状态上（只在仓副本上编、跑；不入库代码）。
//! 起点池建一次（内存稀疏盘），前缀在不录的盘上做，录制流只录最后那一两次发布；崩溃状态照层 0 的枚举域取——
//! 调 `singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state`，`expand` 恒真（每一段都展开，
//! 与 `enumerate_layer0_versions` 同一个域）；观察者收下每个状态的持久集合，之后按线程上限并行在每个崩溃镜像上跑池级 checker，
//! 看删除线新九条的判定与不适用理由。Live 的写法照抄 `research/prompts/m3-batch22-code-r1-opus-model/crash/src/main.rs`。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../harness_tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    check_pool_image, check_pool_image_given_the_tombstone_container_capacity_knob, TombstoneContainerCapacityKnob,
    NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE, NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
    NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER, NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
    NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY, NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB,
    NO_TOMBSTONE_IN_ANY_WALKED_VERSION, NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD,
    NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION, PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB,
};
use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries, ContainerTreeNodeEntriesCappedForTests};
use singlefs_core::lineage_tree::{LineageContainerCapacity, LineageRecordsPerContainerCappedForTests};
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::tombstone_tree::{TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_unlink_intent,
    publish_unlink_steps_until_the_queue_is_empty, warm_up, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::memory_pool::{
    closed_form_state_count, root_identity_of_write, writes_and_segments, MemoryPool, PublishedVersion,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const WT: u64 = 1_788_000_100;
const NINE: [&str; 9] = ["I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.24", "I-9.29"];
const DEFAULT_REASON: &str = "这个镜像上没有它判的对象";

fn new_device() -> SparseBlockDevice {
    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

fn switches_of(per_container: u64) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).expect("1")),
        tombstone_container_capacity: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(per_container).expect("1..=815"),
        ),
        lineage_container_capacity: LineageContainerCapacity::CappedForTests(
            LineageRecordsPerContainerCappedForTests::new(per_container).expect("1..=2039"),
        ),
        container_tree_node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
        ),
    }
}

/// 一个池：设备、现行那一版、分配器、写入口与回退那次装的删除开关（`None` = 产品）。
struct Live<D: BlockDevice> {
    devices: Vec<(DeviceIdentity, D)>,
    output: TransactionOutput,
    allocator: PoolAllocator,
    switches: Option<DeletionSwitches>,
}

impl<D: BlockDevice> Live<D> {
    fn instance(&self) -> InstanceGeneration {
        self.output.root.instance
    }
    fn mark(&self) -> (InstanceGeneration, u64) {
        (self.instance(), self.output.root.checkpoint_txg.0)
    }
    fn create(&mut self, count: u64) -> Result<(), String> {
        self.drain()?;
        let previous = self.output.clone();
        let instance = self.instance();
        let switches = self.switches;
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        if let Some(switches) = switches {
            writer.set_container_tree_capacities(switches.container_tree_capacities());
        }
        self.output = publish_new_inodes(&mut writer, &mut self.allocator, &previous, count, WT, instance)
            .map_err(|e| format!("{e:?}"))?;
        Ok(())
    }
    fn intent(&mut self, inode: u64) -> Result<(), String> {
        self.drain()?;
        let previous = self.output.clone();
        let instance = self.instance();
        let switches = self.switches;
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        if let Some(switches) = switches {
            writer.set_container_tree_capacities(switches.container_tree_capacities());
        }
        self.output = publish_unlink_intent(&mut writer, &mut self.allocator, &previous, InodeNumber(inode), WT, instance)
            .map_err(|e| format!("{e:?}"))?;
        Ok(())
    }
    fn drain(&mut self) -> Result<(), String> {
        if self.output.deleted_inodes.is_empty() {
            return Ok(());
        }
        let instance = self.instance();
        let mut current = self.output.clone();
        let switches = self.switches;
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        if let Some(switches) = switches {
            writer.set_container_tree_capacities(switches.container_tree_capacities());
        }
        publish_unlink_steps_until_the_queue_is_empty(
            &mut writer,
            &mut self.allocator,
            &mut current,
            UnlinkBatchSize::OneExtentLowerLeaf,
            instance,
        )
        .map_err(|e| format!("{e:?}"))?;
        self.output = current;
        Ok(())
    }
    fn rollback(&mut self, target: (InstanceGeneration, u64)) -> Result<(), String> {
        let parameters = parameters();
        roll_back_by_a_forward_publish(
            &parameters,
            &mut self.devices,
            &mut self.allocator,
            &mut self.output,
            RollbackTarget { instance: target.0, checkpoint_txg: CheckpointTxg(target.1) },
        )
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
    }
}

/// 起点：mkfs → 取号 → 暖机 → 新池新建文件 → 建 `count` 个空号（同一次）。内存稀疏盘、不录。
fn starting_pool(count: u64, switches: Option<DeletionSwitches>) -> Live<SparseBlockDevice> {
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
    let content = file_content();
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warm.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    if let Some(switches) = switches {
        allocator.set_deletion_switches(switches);
    }
    let mut pool = Live { devices, output, allocator, switches };
    pool.create(count).expect("建空号");
    pool
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

#[derive(Clone, Copy, Debug)]
enum Act {
    Intent(u64),
    Drain,
    RollbackTo(&'static str),
}

fn reason_tag(reason: &str) -> String {
    let named: [(&str, &str); 11] = [
        (NO_QUEUED_INODE_IN_ANY_WALKED_VERSION, "R-no-queue"),
        (NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD, "R-no-tombstone-naming-record"),
        (NO_TOMBSTONE_IN_ANY_WALKED_VERSION, "R-no-tombstone"),
        (NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE, "R-no-number"),
        (NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER, "R-no-non-rightmost"),
        (NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY, "R-old-entry"),
        (PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB, "R-knob-product-not-judged"),
        (NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB, "R-no-container-under-knob"),
        (NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION, "R-no-lineage-entry"),
        (NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION, "R-no-tombstone-entry"),
        (DEFAULT_REASON, "R-DEFAULT"),
    ];
    for (text, tag) in named {
        if reason == text {
            return tag.to_string();
        }
    }
    format!("别的：{}", reason.chars().take(30).collect::<String>())
}

/// 一个崩溃镜像：新九条每条的判定（不适用带理由短名），与几条旗：缺省理由、九条里有红、I-9.24 报不适用、墓碑两条说法互相矛盾。
fn judge_nine(image: &MemoryPool, knob: Option<u64>) -> (Vec<String>, Vec<String>) {
    let verdicts = match knob {
        None => check_pool_image(image),
        Some(value) => check_pool_image_given_the_tombstone_container_capacity_knob(
            image,
            Some(TombstoneContainerCapacityKnob::new(value).expect("1..=815")),
        ),
    };
    let mut kinds = Vec::new();
    let mut flags = Vec::new();
    let tag_of = |name: &str| -> String {
        match &verdicts.iter().find(|(n, _)| *n == name).expect("清单里有").1 {
            InvariantVerdict::Holds => "成立".to_string(),
            InvariantVerdict::Violated(_) => "不成立".to_string(),
            InvariantVerdict::PartiallyJudged(_) => "部分没判".to_string(),
            InvariantVerdict::NotApplicable(reason) => format!("不适用 {}", reason_tag(reason)),
        }
    };
    for name in NINE {
        let tag = tag_of(name);
        if tag.contains("R-DEFAULT") {
            flags.push(format!("DEFAULT {name}"));
        }
        if tag == "不成立" {
            flags.push(format!("RED {name}"));
        }
        if name == "I-9.24" && tag.starts_with("不适用") {
            flags.push(format!("NA {name} {tag}"));
        }
        kinds.push(format!("{name} {tag}"));
    }
    let i920 = tag_of("I-9.20");
    let i923 = tag_of("I-9.23");
    if (i920 == "成立" || i920 == "部分没判") && i923.contains("R-no-tombstone-entry") {
        flags.push(format!("MISMATCH I-9.20={i920} I-9.23={i923}"));
    }
    if i920.contains("R-no-tombstone") && i923 == "成立" {
        flags.push(format!("MISMATCH I-9.20={i920} I-9.23={i923}"));
    }
    (kinds, flags)
}

/// 几段历史：(名字, 旋钮, 前缀动作（不录）, 录的动作)。用户动作在这几段之间换。
fn histories() -> Vec<(&'static str, Option<u64>, Vec<(&'static str, Act)>, Vec<(&'static str, Act)>)> {
    use Act::*;
    vec![
        ("C1 意图（空号）", None, vec![], vec![("intent", Intent(2))]),
        ("C2 意图之后删完那一步", None, vec![("intent", Intent(2))], vec![("done", Drain)]),
        ("C3 删完之后回退到意图之前", None, vec![("intent", Intent(2)), ("done", Drain)], vec![("rb", RollbackTo("start"))]),
        ("C4 回退之后再删同号的意图", None, vec![("intent", Intent(2)), ("done", Drain), ("rb", RollbackTo("start"))], vec![("re-intent", Intent(2))]),
        ("C5 旋钮下第三个意图（墓碑树根满长到层级 2）", Some(1), vec![("i2", Intent(2)), ("d2", Drain), ("i3", Intent(3)), ("d3", Drain)], vec![("i4", Intent(4))]),
        ("C6 旋钮下第三次回退（谱系树根满长到层级 2）", Some(1),
            vec![("i2a", Intent(2)), ("d2a", Drain), ("rba", RollbackTo("start")), ("i2b", Intent(2)), ("d2b", Drain), ("rbb", RollbackTo("rba"))],
            vec![("i2c", Intent(2)), ("d2c", Drain), ("rbc", RollbackTo("rbb"))]),
    ]
}

fn crash_image(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut image = base.clone();
    let selected: Vec<RetainedWrite> =
        writes.iter().zip(persisted).filter(|(_, kept)| **kept).map(|(write, _)| write.clone()).collect();
    image.apply_writes(&selected);
    image
}

fn main() {
    let count_only = std::env::var("OPUS_COUNT_ONLY").is_ok();
    let threads: usize = std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let selected: Option<Vec<usize>> = std::env::var("OPUS_CRASH_HISTORIES")
        .ok()
        .map(|list| list.split(',').filter_map(|item| item.trim().parse().ok()).collect());
    let state_cap: u64 = std::env::var("OPUS_STATE_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(1_000_000);
    let mut total_states = 0u64;
    for (index, (name, knob, prefix_acts, recorded_acts)) in histories().into_iter().enumerate() {
        if selected.as_ref().is_some_and(|list| !list.contains(&index)) {
            continue;
        }
        let switches = knob.map(switches_of);
        let mut prefix = starting_pool(if knob.is_some() { 5 } else { 2 }, switches);
        let mut marks: BTreeMap<&str, (InstanceGeneration, u64)> = BTreeMap::new();
        marks.insert("start", prefix.mark());
        let mut refused = None;
        for (label, act) in &prefix_acts {
            let outcome = match act {
                Act::Intent(inode) => prefix.intent(*inode),
                Act::Drain => prefix.drain(),
                Act::RollbackTo(target) => prefix.rollback(marks[target]),
            };
            match outcome {
                Ok(()) => {
                    marks.insert(label, prefix.mark());
                }
                Err(reason) => {
                    refused = Some(format!("prefix {label}: {reason}"));
                    break;
                }
            }
        }
        let base_image = image_of(&prefix.devices);
        let stream = SharedStream::retaining_contents();
        let devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = sparse_devices_of(&base_image)
            .into_iter()
            .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
            .collect();
        let mut live = Live { devices, output: prefix.output.clone(), allocator: prefix.allocator.clone(), switches };
        for (label, act) in &recorded_acts {
            if refused.is_some() {
                break;
            }
            let outcome = match act {
                Act::Intent(inode) => live.intent(*inode),
                Act::Drain => live.drain(),
                Act::RollbackTo(target) => live.rollback(marks[target]),
            };
            match outcome {
                Ok(()) => {
                    marks.insert(label, live.mark());
                }
                Err(reason) => refused = Some(format!("{label}: {reason}")),
            }
        }
        if let Some(reason) = refused {
            println!("RESULT crash {name} history-refused {}", reason.chars().take(100).collect::<String>());
            continue;
        }
        let operations = stream.retained_operations();
        let (writes, segments) = writes_and_segments(&operations, &geometry());
        let states = closed_form_state_count(&segments);
        println!(
            "RESULT crash {name} writes={} segments={} segment_lengths={:?} closed_form_states={states}",
            writes.len(),
            segments.len(),
            segments.iter().map(Vec::len).collect::<Vec<_>>()
        );
        if count_only {
            continue;
        }
        if states > state_cap {
            println!("RESULT crash {name} skipped: closed_form_states {states} > {state_cap}");
            continue;
        }
        let mut versions: Vec<PublishedVersion> = vec![PublishedVersion {
            instance: prefix.output.root.instance,
            checkpoint_txg: prefix.output.root.checkpoint_txg,
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
        let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
        let started = std::time::Instant::now();
        let tally = enumerate_layer0_selecting_versions_observing_each_state(
            &base_image,
            &writes,
            &segments,
            judged_root_index,
            &versions,
            &|_segment_index, _segment| true,
            &mut |image, _report| persisted_sets.push(image.persisted.clone()),
        );
        total_states += tally.states;
        println!(
            "RESULT crash {name} layer0 states={} oracle_violations={} failed_states={} enumerate_seconds={:.0}",
            tally.states,
            tally.violations,
            tally.failed_states,
            started.elapsed().as_secs_f64()
        );
        for (invariant, count) in tally.checker_violated_states.iter().filter(|(_, count)| **count > 0) {
            println!("RESULT crash {name} layer0 checker {invariant} red_states={count}");
        }
        let started = std::time::Instant::now();
        let chunk = persisted_sets.len().div_ceil(threads.max(1)).max(1);
        let results: Vec<(Vec<String>, Vec<String>)> = std::thread::scope(|scope| {
            let handles: Vec<_> = persisted_sets
                .chunks(chunk)
                .map(|part| {
                    let base_image = &base_image;
                    let writes = &writes;
                    scope.spawn(move || {
                        part.iter().map(|persisted| judge_nine(&crash_image(base_image, writes, persisted), knob)).collect::<Vec<_>>()
                    })
                })
                .collect();
            handles.into_iter().flat_map(|handle| handle.join().expect("线程")).collect()
        });
        let mut kind_counts: BTreeMap<String, u64> = BTreeMap::new();
        let mut flag_counts: BTreeMap<String, u64> = BTreeMap::new();
        for (state_index, (kinds, flags)) in results.iter().enumerate() {
            for kind in kinds {
                *kind_counts.entry(kind.clone()).or_insert(0) += 1;
            }
            for flag in flags {
                *flag_counts.entry(flag.clone()).or_insert(0) += 1;
            }
            if !flags.is_empty() && state_index < 200_000 {
                println!("DETAIL crash {name} state={state_index} flags={flags:?}");
            }
        }
        println!("RESULT crash {name} nine states={} judge_seconds={:.0}", results.len(), started.elapsed().as_secs_f64());
        for (kind, count) in &kind_counts {
            println!("RESULT crash {name} nine {kind} {count}");
        }
        for (flag, count) in &flag_counts {
            println!("RESULT crash {name} flag {flag} {count}");
        }
    }
    println!("RESULT crash total_enumerated_states={total_states}");
}
