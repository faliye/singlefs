//! 攻方腿（m2-emptypool-nonempty-r1，Opus）的模型：只在副本上跑。Z1 / Z2 / Z3 / Z4 各一组。
#![allow(dead_code, reason = "攻方腿的探针")]

mod common;

use std::cell::Cell;
use std::panic::{catch_unwind, AssertUnwindSafe};

use common::{
    build_pool, file_content, format_pool, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RaisedFloor, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::recovery::{
    choose_superblock, readable_roots, recover, user_visible_tree_root_pointers, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_harness::crash::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice, SparseDevice,
};

fn devices_of_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    base.devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            for (write, is_persisted) in writes.iter().zip(persisted) {
                if *is_persisted && write.device == *identity {
                    device.image.write(write.offset, &write.bytes);
                }
            }
            (*identity, device)
        })
        .collect()
}

fn memory_pool_of_devices(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect::<std::collections::BTreeMap<DeviceIdentity, SparseDevice>>(),
        device_size_in_bytes: IMAGE_BYTES,
    }
}

fn flip_byte_on_sparse(device: &mut SparseBlockDevice, offset: u64, byte_index: usize) {
    let mut sector = device.image.read(DeviceOffsetInBytes(offset), 512);
    sector[byte_index] ^= 0xff;
    device.image.write(DeviceOffsetInBytes(offset), &sector);
}

fn outcome_name(result: &Result<singlefs_core::mount::Mounted, MountError>) -> String {
    match result {
        Ok(mounted) => format!(
            "Ok(instance {} row_txg {} warm_ups {})",
            mounted.output.instance.0,
            mounted.output.row_publish.root().checkpoint_txg.0,
            mounted.output.warm_up_publishes.len()
        ),
        Err(MountError::InstanceRowsOnVersionWithoutFileUnsupported {
            chosen_root,
            first_row_instance,
            instance_to_acquire,
        }) => format!(
            "InstanceRowsOnVersionWithoutFileUnsupported(chosen ({}, {}), rows [{}, {}))",
            chosen_root.instance.0,
            chosen_root.checkpoint_txg.0,
            first_row_instance.0,
            instance_to_acquire.0
        ),
        Err(other) => format!("Err({other:?})"),
    }
}

struct StreamParts {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
}

fn first_transaction_stream(tag: &str) -> StreamParts {
    let pool = build_pool(tag);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    StreamParts {
        base,
        writes,
        segments,
    }
}

fn formatted_pool_stream(tag: &str) -> StreamParts {
    let mut formatted = format_pool(tag);
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let mut allocator = mounted.allocator;
    let content = file_content();
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            mounted.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            mounted.current.record_bytes(),
        )
        .expect("第一个文件版本");
    }
    formatted.devices = Some(devices);
    let base = formatted.memory_pool_after_mkfs();
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    StreamParts {
        base,
        writes,
        segments,
    }
}

/// Z4：第三条流枚举的崩溃镜像与第一条流逐个相同（基线、写表、段都相等 ⇒ 每个 persisted 掩码给出同一份字节）。
#[test]
fn z4_third_stream_enumerates_the_same_crash_images_as_the_first_transaction_stream() {
    let first = first_transaction_stream("opus-z4-first");
    let third = formatted_pool_stream("opus-z4-third");
    let bytes_first: usize = first.writes.iter().map(|write| write.bytes.len()).sum();
    println!(
        "Z4 base_equal={} writes_equal={} segments_equal={} writes={} write_bytes={} segments={:?}",
        first.base == third.base,
        first.writes == third.writes,
        first.segments == third.segments,
        first.writes.len(),
        bytes_first,
        first.segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    assert!(first.base == third.base && first.writes == third.writes && first.segments == third.segments);
}

/// Z3 / Z4：在这条流的崩溃状态上重开走可写挂载（层 0 只跑恢复与 checker，不跑这一步）。
/// 展开 < 10 写的段（22 个状态）+ 18 写那一段取 24 个确定的掩码。
#[test]
fn z3_writable_remount_on_crash_states_of_the_first_mount_stream() {
    let stream = formatted_pool_stream("opus-z3-remount");
    let mut persisted_before = vec![false; stream.writes.len()];
    let mut lines: Vec<String> = Vec::new();
    let mut tally: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let mut classify = |label: String, persisted: &[bool], lines: &mut Vec<String>| {
        let mut devices = devices_of_state(&stream.base, &stream.writes, persisted);
        let result = mount_writable(&parameters(), &mut devices);
        let name = outcome_name(&result);
        let kind = name.split('(').next().unwrap_or("").to_string();
        *tally.entry(kind).or_insert(0) += 1;
        lines.push(format!("Z3STATE {label} -> {name}"));
    };
    for (segment_index, segment) in stream.segments.iter().enumerate() {
        let full_mask: u64 = (1u64 << segment.len()) - 1;
        let masks: Vec<u64> = if segment.len() < 10 {
            (0..full_mask).collect()
        } else {
            (0..24u64).map(|index| (index * 10_007 + 3) % full_mask).collect()
        };
        for mask in masks {
            let mut persisted = persisted_before.clone();
            for (bit, write_index) in segment.iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    persisted[*write_index] = true;
                }
            }
            classify(format!("seg{segment_index} mask{mask:#x}"), &persisted, &mut lines);
        }
        for write_index in segment {
            persisted_before[*write_index] = true;
        }
    }
    classify("all".to_string(), &persisted_before, &mut lines);
    for line in &lines {
        println!("{line}");
    }
    println!("Z3TALLY {tally:?}");
}

/// Z3：txg 3 不成立却走进 `publish_first_file` 那条路。第一次可写挂载崩在「取号两写 + txg 1 的记录两写都持久、txg 1 的根槽没持久」，
/// 之后两块盘的超级块槽 0（取号写进号 1 的那一槽）各坏一个字节——超级块只剩 mkfs 的槽 1（号 0），根环里没有实例 1 的根 ⇒
/// 要取的号又是 1、要写的行为空、不拒绝；而环里那条 txg 1 的记录让新实例从 txg 2 起：写行 txg 2（落盘 0）、暖机 txg 3（盘 0）、txg 4（盘 1）。
/// 之后按用例的调法发第一个文件版本：写死 txg 3 / jsn 3，盖掉暖机 txg 3 的根槽与记录槽；最新根是 txg 4 的暖机根（树表 0 条）。
#[test]
fn z3_first_file_after_two_superblock_faults_lands_under_a_newer_empty_root() {
    let mut formatted = format_pool("opus-z3-two-faults");
    let mut devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut devices).expect("第一次可写挂载");
    formatted.devices = Some(devices);
    let base = formatted.memory_pool_after_mkfs();
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    println!("Z3F first-mount segments {:?}", segments.iter().map(Vec::len).collect::<Vec<_>>());
    let mut persisted = vec![false; writes.len()];
    for write_index in segments[0].iter().chain(segments[1].iter()) {
        persisted[*write_index] = true;
    }
    let mut crash_devices = devices_of_state(&base, &writes, &persisted);
    for (_, device) in crash_devices.iter_mut() {
        flip_byte_on_sparse(device, 0, 100);
    }
    let image = memory_pool_of_devices(&crash_devices);
    let superblock = choose_superblock(&image).expect("超级块");
    println!(
        "Z3F after faults: chosen superblock slot_generation={} journal_instance={} readable_roots={:?}",
        superblock.slot_generation,
        superblock.journal_instance.0,
        readable_roots(&image, &superblock.region_devices, &superblock.geometry, &superblock.filesystem_identifier)
            .iter()
            .map(|root| (root.instance.0, root.checkpoint_txg.0))
            .collect::<Vec<_>>()
    );
    let mounted = mount_writable(&parameters(), &mut crash_devices);
    println!("Z3F remount -> {}", outcome_name(&mounted));
    let mounted = mounted.expect("没拒绝");
    println!(
        "Z3F row_publish (inst {}, txg {}, jsn {}); warm-ups {:?}; current (txg {}, jsn {})",
        mounted.output.row_publish.root().instance.0,
        mounted.output.row_publish.root().checkpoint_txg.0,
        mounted.output.row_publish.record().counter,
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| (publish.root().checkpoint_txg.0, publish.record().counter))
            .collect::<Vec<_>>(),
        mounted.current.root().checkpoint_txg.0,
        mounted.current.record().counter
    );
    let mut allocator = mounted.allocator;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, crash_devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            mounted.current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            mounted.current.record_bytes(),
        )
    };
    let first = first.expect("publish_first_file 不核 txg，照样返回成功");
    println!(
        "Z3F publish_first_file returned Ok: root (inst {}, txg {}), jsn {}",
        first.root.instance.0, first.root.checkpoint_txg.0, first.record.counter
    );
    let image = memory_pool_of_devices(&crash_devices);
    let report = recover(&image, JournalPolicy::Consult);
    println!("Z3F cold recovery (Consult) -> {:?}", report.outcome);
    let ignored = recover(&image, JournalPolicy::Ignore);
    println!("Z3F cold recovery (Ignore) -> {:?}", ignored.outcome);
    for (invariant, verdict) in check_pool_image(&image) {
        if !matches!(verdict, InvariantVerdict::Holds) {
            println!("Z3F checker {invariant} {verdict:?}");
        }
    }
}

/// 一块读偏移 0（超级块槽 0）时按第几次读注入一次瞬时读错的盘；别的读写原样交给内层。
struct TransientSuperblockReadErrorDevice {
    inner: SparseBlockDevice,
    offset_zero_reads: Cell<u32>,
    failing_offset_zero_read_ordinals: Vec<u32>,
    failed_offset_zero_read_ordinals: std::cell::RefCell<Vec<u32>>,
}

impl BlockDevice for TransientSuperblockReadErrorDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if offset.0 == 0 {
            let ordinal = self.offset_zero_reads.get() + 1;
            self.offset_zero_reads.set(ordinal);
            if self.failing_offset_zero_read_ordinals.contains(&ordinal) {
                self.failed_offset_zero_read_ordinals.borrow_mut().push(ordinal);
                return Err(BlockDeviceError::InputOutput(std::io::Error::other("transient")));
            }
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
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

/// Z2：取号之前算的号与取号时算的号不同。镜像 = 只做过 mkfs 的池上第一次可写挂载崩在取号两写与那道屏障之后（盘上只多了号 1）。
/// 每块盘第 1、2 次读超级块槽 0 报瞬时读错（择超级块一次、`instance_generation_to_acquire` 一次）⇒ 拒绝判定看到的号是 1、区间为空、放行；
/// `acquire_instance` 再读时读得到 ⇒ 取到 2、两盘超级块写进号 2 ⇒ 之后 `rows_written` 不为空，断言在写之后 panic。
#[test]
fn z2_transient_superblock_read_errors_split_the_refusal_number_from_the_acquired_number() {
    let stream = formatted_pool_stream("opus-z2-transient");
    let mut persisted = vec![false; stream.writes.len()];
    for write_index in &stream.segments[0] {
        persisted[*write_index] = true;
    }
    for failing in [vec![], vec![1u32], vec![1u32, 2], vec![2u32]] {
        let plain = devices_of_state(&stream.base, &stream.writes, &persisted);
        let before = memory_pool_of_devices(&plain);
        let mut devices: Vec<(DeviceIdentity, TransientSuperblockReadErrorDevice)> = plain
            .into_iter()
            .map(|(identity, inner)| {
                (
                    identity,
                    TransientSuperblockReadErrorDevice {
                        inner,
                        offset_zero_reads: Cell::new(0),
                        failing_offset_zero_read_ordinals: failing.clone(),
                        failed_offset_zero_read_ordinals: std::cell::RefCell::new(Vec::new()),
                    },
                )
            })
            .collect();
        let result = catch_unwind(AssertUnwindSafe(|| mount_writable(&parameters(), &mut devices)));
        let outcome = match &result {
            Ok(mount_result) => outcome_name(mount_result),
            Err(payload) => format!(
                "panic: {}",
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|text| (*text).to_string()))
                    .unwrap_or_default()
            ),
        };
        let after = MemoryPool {
            devices: devices
                .iter()
                .map(|(identity, device)| (*identity, device.inner.image.clone()))
                .collect(),
            device_size_in_bytes: IMAGE_BYTES,
        };
        let superblock_instances: Vec<(u32, u32, u64)> = devices
            .iter()
            .flat_map(|(identity, device)| {
                [0u64, 4096].into_iter().filter_map(move |offset| {
                    singlefs_core::superblock::Superblock::parse_slot(
                        &device.inner.image.read(DeviceOffsetInBytes(offset), 4096),
                    )
                    .map(|superblock| (identity.0, superblock.journal_instance.0, superblock.slot_generation))
                })
            })
            .collect();
        println!(
            "Z2 failing_reads_per_device={failing:?} failed={:?} -> {outcome}; disk_changed={}; superblock (device, instance, generation)={superblock_instances:?}",
            devices.iter().map(|(_, device)| device.failed_offset_zero_read_ordinals.borrow().clone()).collect::<Vec<_>>(),
            before != after
        );
    }
}

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(pool: &mut BuiltPool, content: &[u8], instance: InstanceGeneration) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile { content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

fn build_through_rollback(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let mut reopened = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) },
        ShadowLedger::On,
    )
    .expect("回退");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator;
    pool.output = rolled_back.current.into_file_version().expect("带文件");
    pool
}

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(&parameters(), devices, &mut pool.allocator, &mut current, new_floor, ShadowLedger::On);
    pool.output = current;
    raised
}

/// Z1：F 回落之后（D16 已定项 1「生效」行 2026-09-17 打回重议的那段历史）再抬 F，上限要读 F 之下、已被合法复用的树表单元 ⇒ 被当成「读不出、要走修复」拒绝。
/// 固定脚本到 D、四次覆盖写、抬 F 到 11（15、16）、E（17，数据落回 mkfs 树表 50178）、进程退出、把 txg 16 的根槽改坏一个字节（盘 1 上唯一带 F = 11 的根）、
/// 重开可写挂载（F_生效 = 0）、再抬 F。对照：不改坏 txg 16。
#[test]
fn z1_raising_the_floor_after_it_fell_back_is_refused_on_a_legitimately_reused_tree_table() {
    for damage in [false, true] {
        let mut pool = build_through_rollback(if damage { "opus-z1-fallback" } else { "opus-z1-control" });
        for seed in [17usize, 19, 23, 29] {
            overwrite_in_process(&mut pool, &content_of(3000 + seed, seed), InstanceGeneration(3));
        }
        raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
        let e = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
        let mut devices = pool.reopen_recorded();
        if damage {
            let target = target_for_publish(CheckpointTxg(16));
            let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
            let offset = slot_offset(target, 4096);
            let (_, recorded) = devices.iter_mut().find(|(identity, _)| *identity == device).expect("盘");
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("读根槽");
            bytes[100] ^= 0xff;
            recorded.write_at(offset, &bytes, WriteDurability::Plain).expect("改坏根槽");
        }
        let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
        pool.devices = Some(devices);
        let floor_after_mount = mounted.current.root().rollback_floor;
        pool.allocator = mounted.allocator;
        pool.output = mounted.current.into_file_version().expect("带文件");
        let image = pool.memory_pool();
        let genesis_root = readable_roots(&image, &parameters().region_devices, &parameters().geometry, &parameters().filesystem_identifier)
            .into_iter()
            .find(|root| root.checkpoint_txg == CheckpointTxg(0))
            .expect("第 0 代根还在环里");
        let pointers_of_genesis = user_visible_tree_root_pointers(&image, &genesis_root);
        let result = raise_floor(&mut pool, CheckpointTxg(12));
        println!(
            "Z1F damage_txg16={damage} E_data_slot={} floor_after_remount={} genesis_root_tree_table_slot={} genesis_pointers={:?} raise_to_12 -> {:?}",
            e.data_pointer.locations[0].slot.0,
            floor_after_mount.0,
            genesis_root.tree_table.locations[0].slot.0,
            pointers_of_genesis.as_ref().map(|_| "readable"),
            result.as_ref().map(|raised| (raised.ceiling.0, raised.publishes.len())).map_err(|error| format!("{error:?}"))
        );
    }
}

/// Z1：同一个内容连写两次，树表里两棵树的根指针照样不同（出生 txg 与位置条目随这次发布变）——「同样的内容写回」漏判不了。
#[test]
fn z1_writing_back_identical_content_still_changes_both_pointers() {
    let mut pool = build_pool("opus-z1-same-content");
    let content = file_content();
    let first = overwrite_in_process(&mut pool, &content, InstanceGeneration(1));
    let second = overwrite_in_process(&mut pool, &content, InstanceGeneration(1));
    let image = pool.memory_pool();
    let first_pointers = user_visible_tree_root_pointers(&image, &first.root).expect("读");
    let second_pointers = user_visible_tree_root_pointers(&image, &second.root).expect("读");
    let differing = |left: &Option<Vec<u8>>, right: &Option<Vec<u8>>| -> Vec<usize> {
        let (left, right) = (left.as_ref().expect("有"), right.as_ref().expect("有"));
        (0..86).filter(|index| left[*index] != right[*index]).collect()
    };
    println!(
        "Z1S same content txg {} vs {}: inode differing byte offsets {:?}; extent differing byte offsets {:?}",
        first.root.checkpoint_txg.0,
        second.root.checkpoint_txg.0,
        differing(&first_pointers.inode_tree, &second_pointers.inode_tree),
        differing(&first_pointers.extent_tree, &second_pointers.extent_tree)
    );
}

/// Z3：第一个文件版本之后两条路上的分配器（只住内存的开放段、游标、回收集、隔离位）是否相同，再各覆盖写两次比录制流。
#[test]
fn z3_allocator_and_the_next_two_overwrites_match_between_the_two_paths() {
    let mut reference = build_pool("opus-z3-next-reference");
    let mut formatted = format_pool("opus-z3-next-formatted");
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let mut allocator = mounted.allocator;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(&mut writer, &mut allocator, mounted.current.root(), FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS }, InstanceGeneration(1), mounted.current.record_bytes()).expect("第一个文件版本")
    };
    let allocators_equal_after_first = format!("{allocator:?}") == format!("{:?}", reference.allocator);
    let mut formatted_pool = BuiltPool {
        paths: std::mem::take(&mut formatted.paths),
        devices: Some(devices),
        stream: formatted.stream.clone(),
        genesis: formatted.genesis.clone(),
        warm_up: reference.warm_up.clone(),
        output: first,
        allocator,
        mkfs_operation_count: formatted.mkfs_operation_count,
    };
    for seed in [5usize, 7] {
        overwrite_in_process(&mut reference, &content_of(1000 + seed, seed), InstanceGeneration(1));
        overwrite_in_process(&mut formatted_pool, &content_of(1000 + seed, seed), InstanceGeneration(1));
    }
    let reference_operations = reference.retained_operations();
    let formatted_operations = formatted_pool.retained_operations();
    println!(
        "Z3N allocators_equal_after_first_file={allocators_equal_after_first} streams_after_mkfs_equal={} allocators_equal_after_two_overwrites={} outputs_equal={}",
        reference_operations[reference.mkfs_operation_count..] == formatted_operations[formatted_pool.mkfs_operation_count..],
        format!("{:?}", reference.allocator) == format!("{:?}", formatted_pool.allocator),
        reference.output == formatted_pool.output
    );
}

/// Z3：零单元暖机次数随区域归属变（mkfs 的 check_geometry 只核区域指的盘在不在，不核 0 / 1 / 0）。
/// 每种归属：mkfs → 可写挂载 → `publish_first_file`（写死 txg 3）；数根槽写里 (1, 3) 出现几次、冷恢复、checker 非成立项、18 写段不展开的快枚举。
#[test]
fn z3_region_assignments_accepted_by_mkfs_change_the_zero_unit_publish_count() {
    use singlefs_harness::crash::{enumerate_layer0_selecting_versions, PublishedVersion};
    use singlefs_harness::segments::StepKind;
    use singlefs_harness::{RecordingBlockDevice, SharedStream};
    for region_devices in [[0u32, 1, 0], [0, 1, 1], [1, 0, 0], [0, 0, 1]] {
        let mut publish_parameters = parameters();
        publish_parameters.region_devices = region_devices.map(DeviceIdentity);
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
            .map(|number| {
                (
                    DeviceIdentity(number),
                    RecordingBlockDevice::with_shared_stream(
                        DeviceIdentity(number),
                        SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        let mkfs = singlefs_core::make_filesystem::make_filesystem(&publish_parameters, &mut devices);
        let Ok(_genesis) = mkfs else {
            println!("Z3R regions={region_devices:?} mkfs refused: {:?}", mkfs.err());
            continue;
        };
        let mkfs_operation_count = stream.operations().len();
        let mounted = mount_writable(&publish_parameters, &mut devices).expect("可写挂载");
        let mount_summary = outcome_name(&Ok(mounted));
        // outcome_name 拿走了所有权，重挂一遍拿不到；改成重新读 current：从盘上最新根取。
        let image_after_mount = {
            let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
            pool.apply(&stream.retained_operations());
            pool
        };
        let newest = singlefs_core::recovery::choose_root(&image_after_mount, &choose_superblock(&image_after_mount).expect("超级块")).expect("根");
        let newest_record = singlefs_core::recovery::scan_journal(&image_after_mount, &choose_superblock(&image_after_mount).expect("超级块"))
            .values()
            .max_by_key(|record| record.counter)
            .cloned()
            .expect("记录");
        let mut allocator = singlefs_core::allocator::PoolAllocator::new(vec![
            singlefs_core::allocator::DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
            singlefs_core::allocator::DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
        ]);
        allocator.mark_format_time_units(
            singlefs_core::allocator::Placement { slot: newest.instance_table.locations[0].slot, span: 2 },
            singlefs_core::allocator::Placement { slot: newest.tree_table.locations[0].slot, span: 1 },
        );
        let content = file_content();
        let first = {
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_first_file(
                &mut writer,
                &mut allocator,
                &newest,
                FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                InstanceGeneration(1),
                &newest_record.to_bytes(),
            )
            .expect("第一个文件版本")
        };
        let operations = stream.retained_operations();
        let mut image = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        image.apply(&operations);
        let base = {
            let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
            pool.apply(&operations[..mkfs_operation_count]);
            pool
        };
        let (writes, segments) = writes_and_segments(&operations[mkfs_operation_count..], &geometry());
        let root_writes: Vec<(usize, u32, u64)> = writes
            .iter()
            .enumerate()
            .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
            .map(|(index, write)| {
                (
                    index,
                    u32::from_le_bytes(write.bytes[24..28].try_into().expect("4")),
                    u64::from_le_bytes(write.bytes[28..36].try_into().expect("8")),
                )
            })
            .collect();
        let judged = root_writes.last().expect("根槽写").0;
        let tally = enumerate_layer0_selecting_versions(
            &base,
            &writes,
            &segments,
            judged,
            &[PublishedVersion { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3), content: content.clone() }],
            &|_index, segment| segment.len() < 10,
        );
        let checker_violations: Vec<String> = tally
            .checker_violated_states
            .iter()
            .filter(|(_, count)| **count > 0)
            .map(|(name, count)| format!("{name}={count}:{}", tally.checker_first_violation.get(name).cloned().unwrap_or_default()))
            .collect();
        println!(
            "Z3R regions={region_devices:?} mount={mount_summary} newest_before_file=({}, {}) jsn {} first_file=({}, {}) jsn {} root_writes(index, instance, txg)={root_writes:?} segments={:?} recover={:?} fast_states={} oracle_violations={} first={:?} ignored_violations={} record_checker=({}, {}) checker_violations={checker_violations:?}",
            newest.instance.0,
            newest.checkpoint_txg.0,
            newest_record.counter,
            first.root.instance.0,
            first.root.checkpoint_txg.0,
            first.record.counter,
            segments.iter().map(Vec::len).collect::<Vec<_>>(),
            match recover(&image, JournalPolicy::Consult).outcome { RecoveryOutcome::FileRead { root, content: read } => format!("FileRead{root:?} content_ok={}", read == content), other => format!("{other:?}") },
            tally.states,
            tally.violations,
            tally.first_violation,
            tally.ignored_violations,
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        );
    }
}

/// V2 的射程（不落在 Z1–Z4 任何一条的触发句上，交回主 agent）：只做过 mkfs 的池可写挂载成功、一个文件都不写、进程正常退出（没有崩溃、没有坏块）⇒
/// 下一次可写挂载被拒（树表 0 条、要写 (1, 2, 0)），之后每一次都一样。
#[test]
fn v2_scope_mount_a_formatted_pool_write_nothing_exit_cleanly_then_every_later_writable_mount_is_refused() {
    let mut formatted = format_pool("opus-v2-clean-exit");
    let mut devices = formatted.reopen_recorded();
    let first = mount_writable(&parameters(), &mut devices);
    println!("V2 first mount -> {}", outcome_name(&first));
    drop(first);
    formatted.devices = Some(devices);
    for attempt in 0..2 {
        let mut devices = formatted.reopen_recorded();
        let again = mount_writable(&parameters(), &mut devices);
        println!("V2 remount attempt {attempt} -> {}", outcome_name(&again));
        formatted.devices = Some(devices);
    }
    let image = formatted.memory_pool();
    println!("V2 cold recovery -> {:?}", recover(&image, JournalPolicy::Consult).outcome);
}
