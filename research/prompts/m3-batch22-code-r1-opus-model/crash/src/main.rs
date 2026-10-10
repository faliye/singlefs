//! m3-batch22-code-r1 攻方原型之二：崩溃状态（只在仓副本上编、跑；不入库代码）。
//! 起点池建一次（内存稀疏盘），之后在录制盘上走一段历史；录制流按层 0 的枚举域取崩溃状态——
//! 调 `singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state`，`expand` 恒真（每一段都展开，
//! 与 `enumerate_layer0_versions` 同一个域），它在每个崩溃镜像上跑两遍恢复与池级 checker；观察者把每个状态的持久集合收下来，
//! 之后在每个崩溃状态上可写挂载（崩溃恢复）、挂载之后的镜像再跑池级 checker，再建一个号（落到旧根之后重发）再跑一次。
#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../harness_tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;

use common::{file_content, geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
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

fn new_device() -> SparseBlockDevice {
    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512))
}

/// 一个池：设备、现行那一版、分配器。
struct Live<D: BlockDevice> {
    devices: Vec<(DeviceIdentity, D)>,
    output: TransactionOutput,
    allocator: PoolAllocator,
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
        let parameters = parameters();
        let previous = self.output.clone();
        let instance = self.instance();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        self.output = publish_new_inodes(&mut writer, &mut self.allocator, &previous, count, WT, instance)
            .map_err(|e| format!("{e:?}"))?;
        Ok(())
    }
    fn intent(&mut self, inode: u64) -> Result<(), String> {
        self.drain()?;
        let parameters = parameters();
        let previous = self.output.clone();
        let instance = self.instance();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
        self.output = publish_unlink_intent(&mut writer, &mut self.allocator, &previous, InodeNumber(inode), WT, instance)
            .map_err(|e| format!("{e:?}"))?;
        Ok(())
    }
    fn drain(&mut self) -> Result<(), String> {
        if self.output.deleted_inodes.is_empty() {
            return Ok(());
        }
        let parameters = parameters();
        let instance = self.instance();
        let mut current = self.output.clone();
        let mut writer = PoolWriter::new(&parameters, &mut self.devices);
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
    fn remount(&mut self) -> Result<(), String> {
        let parameters = parameters();
        let mounted = mount_writable(&parameters, &mut self.devices).map_err(|e| format!("{e:?}"))?;
        self.allocator = mounted.allocator.clone();
        self.output = mounted.current.version_of_trees().ok_or("树表 0 条")?.clone();
        Ok(())
    }
}

/// 起点：mkfs → 取号 → 暖机 → 新池新建文件 → 建 2、3 号（同一次）。内存稀疏盘、不录。
fn starting_pool() -> Live<SparseBlockDevice> {
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
    let mut pool = Live { devices, output, allocator };
    pool.create(2).expect("建 2、3 号");
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

fn violated_names(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then_some(invariant))
        .collect()
}

/// 用户动作（历史里那几步，写死在下面几段历史里；回退目标按名字取前面记下的版本）。
#[derive(Clone, Copy, Debug)]
enum Act {
    Create,
    Intent(u64),
    Drain,
    RollbackTo(&'static str),
    Remount,
}

/// 几段历史：前缀是起点池；故障是录制流里的每一个崩溃状态；用户动作在这几段之间换（回退到哪一版、回退之后先做什么）。
fn histories() -> Vec<(&'static str, Vec<(&'static str, Act)>)> {
    use Act::*;
    vec![
        ("H1 删->回退到意图之前->再删", vec![("intent", Intent(2)), ("done", Drain), ("rb", RollbackTo("start")), ("re-intent", Intent(2)), ("re-done", Drain)]),
        ("H2 删->回退到意图之前->再回退回删完那一版", vec![("intent", Intent(2)), ("done", Drain), ("rb", RollbackTo("start")), ("rb-done", RollbackTo("done"))]),
        ("H3 删->回退到意图那一版->进程退出再挂续做", vec![("intent", Intent(2)), ("done", Drain), ("rb", RollbackTo("intent")), ("remount", Remount)]),
        ("H4 建->回退到建之前->再建", vec![("create", Create), ("rb", RollbackTo("start")), ("create-again", Create)]),
        ("H5 删->回退到意图之前->删另一个号", vec![("intent", Intent(2)), ("done", Drain), ("rb", RollbackTo("start")), ("other", Intent(3)), ("other-done", Drain)]),
        ("H6 删->回退到意图那一版->删另一个号（先走空）", vec![("intent", Intent(2)), ("done", Drain), ("rb", RollbackTo("intent")), ("other", Intent(3)), ("other-done", Drain)]),
    ]
}

/// 一个崩溃状态的镜像：起点镜像加持久了的那几条写。
fn crash_image(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut image = base.clone();
    let selected: Vec<RetainedWrite> =
        writes.iter().zip(persisted).filter(|(_, kept)| **kept).map(|(write, _)| write.clone()).collect();
    image.apply_writes(&selected);
    image
}

/// 崩溃恢复（可写挂载）之后判一次；挂得上的再建一个号（落到旧根之后接着发）再判一次。
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
    let mut live = Live { devices, output, allocator: mounted.allocator.clone() };
    let after_create = match live.create(1) {
        Ok(()) => violated_names(&image_of(&live.devices)),
        Err(_) => return ("mounted;create-refused".to_string(), after_mount, Vec::new()),
    };
    ("mounted".to_string(), after_mount, after_create)
}

fn main() {
    let count_only = std::env::var("OPUS_COUNT_ONLY").is_ok();
    let threads: usize = std::env::var("SINGLEFS_THREAD_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
    let selected: Option<Vec<usize>> = std::env::var("OPUS_CRASH_HISTORIES")
        .ok()
        .map(|list| list.split(',').filter_map(|item| item.trim().parse().ok()).collect());
    let base = starting_pool();
    let base_image = image_of(&base.devices);
    let mut total_states = 0u64;
    for (index, (name, acts)) in histories().into_iter().enumerate() {
        if selected.as_ref().is_some_and(|list| !list.contains(&index)) {
            continue;
        }
        // OPUS_RECORD_FROM=k：前 k 个动作在不录的内存盘上做（前缀，只做一次），录制流只从第 k 个动作起——崩溃状态只取这一段。
        let record_from: usize = std::env::var("OPUS_RECORD_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        let mut prefix = Live { devices: sparse_devices_of(&base_image), output: base.output.clone(), allocator: base.allocator.clone() };
        let mut marks: BTreeMap<&str, (InstanceGeneration, u64)> = BTreeMap::new();
        marks.insert("start", prefix.mark());
        let mut refused = None;
        for (label, act) in acts.iter().take(record_from) {
            let outcome = match act {
                Act::Create => prefix.create(1),
                Act::Intent(inode) => prefix.intent(*inode),
                Act::Drain => prefix.drain(),
                Act::RollbackTo(target) => prefix.rollback(marks[target]),
                Act::Remount => prefix.remount(),
            };
            match outcome {
                Ok(()) => {
                    marks.insert(label, prefix.mark());
                }
                Err(reason) => refused = Some(format!("prefix {label}: {reason}")),
            }
        }
        let base_image = image_of(&prefix.devices);
        let base = Live { devices: Vec::<(DeviceIdentity, SparseBlockDevice)>::new(), output: prefix.output.clone(), allocator: prefix.allocator.clone() };
        let stream = SharedStream::retaining_contents();
        let devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = sparse_devices_of(&base_image)
            .into_iter()
            .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
            .collect();
        let mut live = Live { devices, output: base.output.clone(), allocator: base.allocator.clone() };
        for (label, act) in acts.iter().skip(record_from) {
            if refused.is_some() {
                break;
            }
            let outcome = match act {
                Act::Create => live.create(1),
                Act::Intent(inode) => live.intent(*inode),
                Act::Drain => live.drain(),
                Act::RollbackTo(target) => live.rollback(marks[target]),
                Act::Remount => live.remount(),
            };
            match outcome {
                Ok(()) => {
                    marks.insert(label, live.mark());
                }
                Err(reason) => {
                    refused = Some(format!("{label}: {reason}"));
                    break;
                }
            }
        }
        if let Some(reason) = refused {
            println!("RESULT crash {name} history-refused {}", reason.chars().take(100).collect::<String>());
            continue;
        }
        let operations = stream.retained_operations();
        let (writes, segments) = writes_and_segments(&operations, &geometry());
        let states = closed_form_state_count(&segments);
        let longest = segments.iter().map(Vec::len).max().unwrap_or(0);
        println!(
            "RESULT crash {name} marks={:?} writes={} segments={} longest_segment={longest} closed_form_states={states}",
            marks.values().map(|(i, t)| format!("i{}t{t}", i.0)).collect::<Vec<_>>(),
            writes.len(),
            segments.len()
        );
        println!("RESULT crash {name} segment_lengths={:?}", segments.iter().map(Vec::len).collect::<Vec<_>>());
        if count_only {
            continue;
        }
        // OPUS_DOMAIN=boundaries:K —— 不是层 0 全域：每个段界（前几段全持久、这一段一条都没持久）各一个状态，长度 ≤ K 的段全展开，
        // 末了全持久那一个。全域每段都超过 1e6、照派发规则不跑时用它补「崩在屏障段界上」那一格（报告里单列，不当层 0 覆盖）。
        if let Some(short_limit) = std::env::var("OPUS_DOMAIN").ok().and_then(|value| value.strip_prefix("boundaries:").and_then(|k| k.parse::<usize>().ok())) {
            let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
            let mut before = vec![false; writes.len()];
            for segment in &segments {
                if segment.len() <= short_limit {
                    for mask in 0..(1u64 << segment.len()) - 1 {
                        let mut persisted = before.clone();
                        for (bit, write_index) in segment.iter().enumerate() {
                            if mask & (1 << bit) != 0 {
                                persisted[*write_index] = true;
                            }
                        }
                        persisted_sets.push(persisted);
                    }
                } else {
                    persisted_sets.push(before.clone());
                }
                for write_index in segment {
                    before[*write_index] = true;
                }
            }
            persisted_sets.push(before);
            total_states += persisted_sets.len() as u64;
            let started = std::time::Instant::now();
            let chunk = persisted_sets.len().div_ceil(threads.max(1)).max(1);
            let results: Vec<(Vec<&'static str>, String, Vec<&'static str>, Vec<&'static str>)> = std::thread::scope(|scope| {
                let handles: Vec<_> = persisted_sets
                    .chunks(chunk)
                    .map(|part| {
                        let base_image = &base_image;
                        let writes = &writes;
                        scope.spawn(move || {
                            part.iter()
                                .map(|persisted| {
                                    let image = crash_image(base_image, writes, persisted);
                                    let before_recovery = violated_names(&image);
                                    let (outcome, after_mount, after_create) = after_recovery(&image);
                                    (before_recovery, outcome, after_mount, after_create)
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect();
                handles.into_iter().flat_map(|handle| handle.join().expect("线程")).collect()
            });
            let mut outcome_counts: BTreeMap<String, u64> = BTreeMap::new();
            let mut reds: BTreeMap<String, u64> = BTreeMap::new();
            for (state_index, (before_recovery, outcome, after_mount, after_create)) in results.iter().enumerate() {
                *outcome_counts.entry(outcome.clone()).or_insert(0) += 1;
                for (phase, list) in [("before_recovery", before_recovery), ("after_mount", after_mount), ("after_create", after_create)] {
                    for invariant in list {
                        *reds.entry(format!("{phase}:{invariant}")).or_insert(0) += 1;
                    }
                }
                if !before_recovery.is_empty() || !after_mount.is_empty() || !after_create.is_empty() {
                    println!("DETAIL crash {name} boundary-state={state_index} before={before_recovery:?} {outcome} after_mount={after_mount:?} after_create={after_create:?}");
                }
            }
            println!(
                "RESULT crash {name} boundaries:{short_limit} states={} outcomes={outcome_counts:?} reds={reds:?} seconds={:.0}",
                results.len(),
                started.elapsed().as_secs_f64()
            );
            continue;
        }
        if states > 1_000_000 {
            println!("RESULT crash {name} skipped: closed_form_states {states} > 1e6");
            continue;
        }
        let mut versions: Vec<PublishedVersion> = vec![PublishedVersion {
            instance: base.output.root.instance,
            checkpoint_txg: base.output.root.checkpoint_txg,
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
            println!(
                "RESULT crash {name} layer0 checker {invariant} red_states={count} first={}",
                tally.checker_first_violation.get(invariant).map(|text| text.chars().take(160).collect::<String>()).unwrap_or_default()
            );
        }
        if let Some(first) = &tally.first_violation {
            println!("DETAIL crash {name} layer0 first_oracle_violation={}", first.chars().take(200).collect::<String>());
        }
        // 崩溃恢复之后：每个状态可写挂载、判一次，再建一个号、判一次。按线程上限分块并行。
        let started = std::time::Instant::now();
        let chunk = persisted_sets.len().div_ceil(threads.max(1)).max(1);
        let results: Vec<(String, Vec<&'static str>, Vec<&'static str>)> = std::thread::scope(|scope| {
            let handles: Vec<_> = persisted_sets
                .chunks(chunk)
                .map(|part| {
                    let base_image = &base_image;
                    let writes = &writes;
                    scope.spawn(move || {
                        part.iter().map(|persisted| after_recovery(&crash_image(base_image, writes, persisted))).collect::<Vec<_>>()
                    })
                })
                .collect();
            handles.into_iter().flat_map(|handle| handle.join().expect("线程")).collect()
        });
        let mut outcome_counts: BTreeMap<String, u64> = BTreeMap::new();
        let mut red_after_mount: BTreeMap<&'static str, u64> = BTreeMap::new();
        let mut red_after_create: BTreeMap<&'static str, u64> = BTreeMap::new();
        for (state_index, (outcome, mount_reds, create_reds)) in results.iter().enumerate() {
            *outcome_counts.entry(outcome.clone()).or_insert(0) += 1;
            for invariant in mount_reds {
                *red_after_mount.entry(invariant).or_insert(0) += 1;
            }
            for invariant in create_reds {
                *red_after_create.entry(invariant).or_insert(0) += 1;
            }
            if (!mount_reds.is_empty() || !create_reds.is_empty()) && state_index < 1_000_000 {
                println!("DETAIL crash {name} state={state_index} {outcome} after_mount={mount_reds:?} after_create={create_reds:?}");
            }
        }
        println!(
            "RESULT crash {name} recovered states={} outcomes={outcome_counts:?} red_after_mount={red_after_mount:?} red_after_create={red_after_create:?} seconds={:.0}",
            results.len(),
            started.elapsed().as_secs_f64()
        );
    }
    println!("RESULT crash total_enumerated_states={total_states}");
}
