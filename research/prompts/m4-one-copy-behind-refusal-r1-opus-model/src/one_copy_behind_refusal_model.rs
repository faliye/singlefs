//! 坏副本放行第一轮攻方腿的模型：手造盘面，每个盘面逐臂走可写挂载，挂得上的接着改动、读回、卸载、判池级 checker。
//! 盘全在内存稀疏盘上。臂由 `ONE_COPY_MODEL_ARM` 选（`apply_arms_patch.py` 接进副本的 `mount.rs`）。
//! 这份模型不调崩溃枚举函数，盘面是手挑的；枚举域那一半由坏盘用例在各臂下跑（报告「要主 agent 跑的命令」）。
mod common;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::Ordering;
use std::time::Instant;

use common::{geometry, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::journal::JournalRecord;
use singlefs_core::make_filesystem::{
    make_filesystem, root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, unmount, ONE_COPY_MODEL_ARM};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::pointer::LocationEntry;
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_core::system_configuration_repair::{
    rewrite_system_configuration_slots_for_the_asserted_version, AssertedVersion,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_sequential_write, warm_up, FirstFile, PoolWriter,
    FIRST_INODE_NUMBER,
};
use singlefs_core::unit::unit_filesystem_identifier;
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, SLOT_BYTES};
use singlefs_verification::memory_pool::{
    closed_form_state_count_of_write_index_segments, writes_and_segments_with_stream_indexes,
    MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_verification::mount_transparency::{read_recovery_fields, ComparedVersion};
use singlefs_verification::{RecordingBlockDevice, RetainedOperation, SharedStream};

type RecordedSparse = RecordingBlockDevice<SparseBlockDevice>;

const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
const ARMS: [(&str, u8); 5] = [("today", 0), ("c1", 1), ("c2", 2), ("c1-same", 3), ("c3", 4)];
const FLIPPED_BYTE_IN_THE_UNIT: u64 = 4000;
const FILE_BYTES: usize = 200_000;
const FULL_SWEEP_LIMIT_SECONDS: f64 = 2400.0;

fn content_of_the_file() -> Vec<u8> {
    (0..FILE_BYTES).map(|index| (index % 251) as u8).collect()
}

fn recorded_devices_of(image: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, RecordedSparse)> {
    image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters().geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect()
}

/// mkfs、取号、暖机、发第一个文件：交回盘与第一个文件各数据单元的两条位置条目。
fn pool_with_a_file(content: &[u8]) -> (MemoryPool, Vec<[LocationEntry; 2]>) {
    let publish_parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let empty = MemoryPool::with_devices(&[DEVICE_ZERO, DEVICE_ONE], IMAGE_BYTES);
    let mut devices = recorded_devices_of(&empty, &stream);
    let genesis = make_filesystem(&publish_parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DEVICE_ZERO, IMAGE_BYTES),
        DeviceFreeMap::new(DEVICE_ONE, IMAGE_BYTES),
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
    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
        &publish_parameters,
        &genesis,
    ));
    let output = {
        let mut pool = PoolWriter::new(&publish_parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warm_up_output = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        let first_file_content = common::file_content();
        let first_file = publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_file_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("新池新建文件");
        publish_sequential_write(
            &mut pool,
            &mut allocator,
            &first_file,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            },
            instance,
        )
        .expect("顺序写")
    };
    let mut image = empty.clone();
    image.apply(&stream.retained_operations());
    let locations = output
        .data_pointers
        .iter()
        .map(|pointer| pointer.locations)
        .collect();
    (image, locations)
}

/// 一段历史：`base` 上可写挂载，再建 `changes` 个空 inode，不卸载（所有写都落了）。交回录到的流与挂载那一串占了流里几步。
struct Run {
    base: MemoryPool,
    operations: Vec<RetainedOperation>,
    mount_operations: usize,
}

impl Run {
    fn of(base: &MemoryPool, changes: u64, write_time_seconds: u64) -> Self {
        let stream = SharedStream::retaining_contents();
        let mut devices = recorded_devices_of(base, &stream);
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        let mount_operations = stream.retained_operations().len();
        let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
        for change in 0..changes {
            session
                .publish_user_change(
                    &mut devices,
                    UserChange::NewInodes {
                        count: 1,
                        write_time_seconds: write_time_seconds + change,
                    },
                )
                .expect("建一个空 inode");
        }
        Self {
            base: base.clone(),
            operations: stream.retained_operations(),
            mount_operations,
        }
    }

    /// 一段历史：`base` 上可写挂载，再按 inode 号顺序写一次 `content`（普通的多单元写：一次发布几条记录）。
    fn of_a_sequential_write(base: &MemoryPool, inode: InodeNumber, content: &[u8], write_time_seconds: u64) -> Self {
        let stream = SharedStream::retaining_contents();
        let mut devices = recorded_devices_of(base, &stream);
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        let mount_operations = stream.retained_operations().len();
        let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
        session
            .publish_user_change(
                &mut devices,
                UserChange::SequentialWriteOfAnInode {
                    inode,
                    content,
                    write_time_seconds,
                },
            )
            .expect("顺序写");
        Self {
            base: base.clone(),
            operations: stream.retained_operations(),
            mount_operations,
        }
    }

    fn image_after_everything(&self) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply(&self.operations);
        image
    }

    fn image_after_the_mount(&self) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply(&self.operations[..self.mount_operations]);
        image
    }

    fn writes_after_the_mount(&self) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
        let (writes, segments, _) = writes_and_segments_with_stream_indexes(
            &self.operations[self.mount_operations..],
            &geometry(),
        );
        (writes, segments)
    }

    fn writes_of_the_whole_run(&self) -> Vec<RetainedWrite> {
        writes_and_segments_with_stream_indexes(&self.operations, &geometry()).0
    }
}

fn is_a_journal_record_write(write: &RetainedWrite) -> bool {
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    (ring_start..ring_start + JOURNAL_RING_DEFAULT_BYTES).contains(&write.offset.0)
}

/// 流里第一对 journal 记录写的下标，按写的次序（盘 0 那一份在前）。
fn first_record_pair(writes: &[RetainedWrite]) -> (usize, usize) {
    let mut indexes = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| is_a_journal_record_write(write))
        .map(|(index, _)| index);
    let first = indexes.next().expect("流里有记录写");
    let second = indexes.next().expect("每块盘一份");
    assert_eq!(writes[first].device, DEVICE_ZERO);
    assert_eq!(writes[second].device, DEVICE_ONE);
    (first, second)
}

/// `image` 上施加第一对记录写之前的全部写，再加 `kept` 那块盘的那一份记录（另一份没落）。
fn crash_with_the_record_only_on(image: &MemoryPool, writes: &[RetainedWrite], kept: DeviceIdentity) -> MemoryPool {
    let (first, second) = first_record_pair(writes);
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..first]);
    crashed.apply_writes(std::slice::from_ref(if kept == DEVICE_ZERO {
        &writes[first]
    } else {
        &writes[second]
    }));
    crashed
}

/// 流里全部记录写都落了，只有最后一条记录只落 `kept` 那一块盘。
fn crash_with_the_last_record_only_on(image: &MemoryPool, writes: &[RetainedWrite], kept: DeviceIdentity) -> MemoryPool {
    let records: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| is_a_journal_record_write(write))
        .map(|(index, _)| index)
        .collect();
    let (first, second) = (records[records.len() - 2], records[records.len() - 1]);
    let mut crashed = image.clone();
    crashed.apply_writes(&writes[..first]);
    crashed.apply_writes(std::slice::from_ref(if kept == DEVICE_ZERO {
        &writes[first]
    } else {
        &writes[second]
    }));
    crashed
}

fn damage(image: &mut MemoryPool, location: &LocationEntry) {
    image.flip_byte(
        location.device,
        location.slot.to_device_offset(),
        FLIPPED_BYTE_IN_THE_UNIT,
    );
}

fn location_on(locations: &[LocationEntry; 2], device: DeviceIdentity) -> LocationEntry {
    *locations
        .iter()
        .find(|location| location.device == device)
        .expect("两块盘各一份")
}

fn first_chars(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

fn sparse_devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                image.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters().geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect()
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)], image: &MemoryPool) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    }
}

/// 现行那一版的单元里，这块盘上读出来字节不同的有几个（共几个，字节是空的不算）。
fn units_of_the_current_version_differing_on(
    session: &MountedSession,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    device: DeviceIdentity,
) -> (usize, usize) {
    let Some(version) = session.current.version_of_trees() else {
        return (0, 0);
    };
    let (_, disk) = devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("有这块盘");
    let units: Vec<_> = version
        .units
        .iter()
        .filter(|unit| !unit.bytes.is_empty())
        .collect();
    let differing = units
        .iter()
        .filter(|unit| {
            disk.image
                .read(unit.slot.to_device_offset(), unit.bytes.len())[..]
                != unit.bytes[..]
        })
        .count();
    (differing, units.len())
}

/// 这张盘上走一遍可写挂载；`follow_on` 时接着一次改动、读回、卸载、判池级 checker。返回一行读数。
fn walk(image: &MemoryPool, content: &[u8], follow_on: bool) -> String {
    let mut devices = sparse_devices_of(image);
    let mounted = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => mounted,
        Err(refusal) => {
            return format!(
                "refused disk_unchanged={} refusal={}",
                image_of(&devices, image) == *image,
                first_chars(&format!("{refusal:?}"), 420)
            );
        }
    };
    let (_, mut session): (_, MountedSession) = MountedSession::of_the_mount(mounted);
    let (missing_zero, units) = units_of_the_current_version_differing_on(&session, &devices, DEVICE_ZERO);
    let (missing_one, _) = units_of_the_current_version_differing_on(&session, &devices, DEVICE_ONE);
    let mut line =
        format!("mounted units={units} differing_after_mount_on_0={missing_zero} on_1={missing_one}");
    if !follow_on {
        return line;
    }
    match session.publish_user_change(
        &mut devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 900,
        },
    ) {
        Ok(change) => line.push_str(&format!(
            " change=ok relocation={}",
            first_chars(
                &format!(
                    "{:?}",
                    change.relocation_after_the_change_of_the_units_read_with_one_copy_failing
                ),
                160
            )
        )),
        Err(refusal) => {
            line.push_str(&format!(" change_refused={}", first_chars(&format!("{refusal:?}"), 200)));
            return line;
        }
    }
    let read = session.read_the_file(
        &devices,
        InodeNumber(FIRST_INODE_NUMBER),
        FileOffsetInBytes(0),
        u64::try_from(content.len()).expect("文件字节数"),
    );
    line.push_str(&match read {
        Ok(read) => format!(" read_equal={}", read.bytes == content),
        Err(refusal) => format!(" read_refused={}", first_chars(&format!("{refusal:?}"), 200)),
    });
    let (missing_zero, _) = units_of_the_current_version_differing_on(&session, &devices, DEVICE_ZERO);
    let (missing_one, _) = units_of_the_current_version_differing_on(&session, &devices, DEVICE_ONE);
    line.push_str(&format!(" differing_after_change_on_0={missing_zero} on_1={missing_one}"));
    if let Err(refusal) = unmount(
        &parameters(),
        &mut devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    ) {
        line.push_str(&format!(" unmount_refused={}", first_chars(&format!("{refusal:?}"), 200)));
        return line;
    }
    let after = image_of(&devices, image);
    let violated: Vec<String> = check_pool_image(&after)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(why) => {
                Some(format!("{invariant}:{}", first_chars(&why, 100)))
            }
            _ => None,
        })
        .collect();
    line.push_str(&format!(" unmount=ok violations={} {violated:?}", violated.len()));
    line
}

fn every_arm(name: &str, image: &MemoryPool, content: &[u8], follow_on: bool) {
    let selected = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion)
        .report
        .effective_root;
    println!("MODEL_STATE {name} selected={selected:?}");
    for (arm, number) in ARMS {
        ONE_COPY_MODEL_ARM.store(number, Ordering::SeqCst);
        let line = catch_unwind(AssertUnwindSafe(|| walk(image, content, follow_on)))
            .unwrap_or_else(|_| "PANICKED".to_string());
        println!("MODEL_ARM {name} arm={arm} {line}");
    }
    ONE_COPY_MODEL_ARM.store(0, Ordering::SeqCst);
}

/// 把 `other` 的盘 0 换进 `image`（「旧快照的盘换回来」）。
fn with_device_zero_of(image: &MemoryPool, other: &MemoryPool) -> MemoryPool {
    let mut swapped = image.clone();
    swapped
        .devices
        .insert(DEVICE_ZERO, other.devices[&DEVICE_ZERO].clone());
    swapped
}

fn tails_of(image: &MemoryPool) -> String {
    let publish_parameters = parameters();
    let spacing = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    [DEVICE_ZERO, DEVICE_ONE]
        .iter()
        .map(|device| {
            let newest = verified_system_configuration_slots(
                image,
                *device,
                spacing,
                &publish_parameters.filesystem_identifier,
            )
            .into_iter()
            .max_by_key(|slot| slot.quantities.slot_generation);
            newest.map_or_else(
                || format!("{}:none", device.0),
                |slot| {
                    format!(
                        "{}:({},{})",
                        device.0, slot.quantities.journal_instance.0, slot.quantities.journal_tail
                    )
                },
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn shape(name: &str, image: &MemoryPool, content: &[u8], follow_on: bool) {
    println!("MODEL_TAILS {name} {}", tails_of(image));
    every_arm(name, image, content, follow_on);
}

fn scenario(name: &str, body: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(body)).is_err() {
        println!("MODEL_SCENARIO_PANICKED {name}");
    }
}

fn counter_fork_sweep(before: &MemoryPool) {
    let started = Instant::now();
    let older_timeline: Vec<MemoryPool> = (0..=6)
        .map(|changes| Run::of(before, changes, FIXED_WRITE_TIME_SECONDS + 100).image_after_everything())
        .collect();
    let mounted_once = Run::of(before, 0, 0).image_after_everything();
    let newer_timeline: Vec<MemoryPool> = (0..=6)
        .map(|changes| Run::of(&mounted_once, changes, FIXED_WRITE_TIME_SECONDS + 300).image_after_everything())
        .collect();
    println!("MODEL_FORK timelines_built_seconds={:.1}", started.elapsed().as_secs_f64());
    let composites = older_timeline.len() * newer_timeline.len();
    let mut done = 0usize;
    for (k_older, older) in older_timeline.iter().enumerate() {
        for (k_newer, newer) in newer_timeline.iter().enumerate() {
            let composite = with_device_zero_of(newer, older);
            let one = Instant::now();
            println!("MODEL_FORK k_older={k_older} k_newer={k_newer} tails={}", tails_of(&composite));
            every_arm(&format!("fork-{k_older}-{k_newer}"), &composite, &[], false);
            done += 1;
            if done == 1 {
                let estimate = one.elapsed().as_secs_f64() * composites as f64;
                println!("MODEL_FORK estimate_seconds={estimate:.0} composites={composites}");
                if estimate > FULL_SWEEP_LIMIT_SECONDS {
                    println!("MODEL_FORK stopped_over_the_limit");
                    return;
                }
            }
        }
    }
}

#[test]
fn shapes_of_the_one_copy_behind_refusal() {
    let content = content_of_the_file();
    let (file_pool, data) = pool_with_a_file(&content);
    println!("MODEL_FILE data_units={}", data.len());
    let before = Run::of(&file_pool, 2, FIXED_WRITE_TIME_SECONDS + 60).image_after_everything();
    let publish = Run::of(&before, 1, FIXED_WRITE_TIME_SECONDS + 70);
    let after_mount = publish.image_after_the_mount();
    let (writes, segments) = publish.writes_after_the_mount();
    println!(
        "MODEL_STREAM writes={} segment_sizes={:?} closed_form_states={}",
        writes.len(),
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        closed_form_state_count_of_write_index_segments(&segments)
    );
    let (first, second) = first_record_pair(&writes);
    let record = JournalRecord::parse(
        writes[second].bytes().expect("记录写带字节"),
        unit_filesystem_identifier(&parameters().filesystem_identifier),
    )
    .expect("解得开");
    let named_on_zero = location_on(&record.named[0].locations, DEVICE_ZERO);
    let only_on_one = crash_with_the_record_only_on(&after_mount, &writes, DEVICE_ONE);
    let only_on_zero = crash_with_the_record_only_on(&after_mount, &writes, DEVICE_ZERO);
    let damaged = |mut image: MemoryPool, locations: &[LocationEntry]| {
        for location in locations {
            damage(&mut image, location);
        }
        image
    };
    let old_on_zero = location_on(&data[0], DEVICE_ZERO);
    let old_on_one = location_on(&data[0], DEVICE_ONE);
    let other_on_one = location_on(&data[1], DEVICE_ONE);
    let all_on_zero: Vec<LocationEntry> = data.iter().map(|l| location_on(l, DEVICE_ZERO)).collect();
    scenario("control", || shape("control-record-on-1-no-damage", &only_on_one, &content, true));
    scenario("older", || {
        shape("older-unit-on-0-record-on-1", &damaged(only_on_one.clone(), &[old_on_zero]), &content, true);
    });
    scenario("named", || {
        shape("named-unit-on-0-record-on-1", &damaged(only_on_one.clone(), &[named_on_zero]), &content, true);
    });
    scenario("mirror", || {
        shape("older-unit-on-1-record-on-0", &damaged(only_on_zero.clone(), &[old_on_one]), &content, true);
    });
    scenario("two", || {
        shape("older-on-0-other-on-1-record-on-1", &damaged(only_on_one.clone(), &[old_on_zero, other_on_one]), &content, true);
    });
    scenario("both", || {
        shape("same-unit-both-copies-record-on-1", &damaged(only_on_one.clone(), &[old_on_zero, old_on_one]), &content, true);
    });
    scenario("many", || {
        shape("every-data-unit-on-0-record-on-1", &damaged(only_on_one.clone(), &all_on_zero), &content, true);
        shape("every-data-unit-on-0-all-persisted", &damaged(publish.image_after_everything(), &all_on_zero), &content, true);
    });
    scenario("root-landed", || {
        let root_index = (second..writes.len())
            .find(|index| writes[*index].is_force_unit_access)
            .expect("根槽 FUA");
        let mut landed = after_mount.clone();
        landed.apply_writes(&writes[..=root_index]);
        landed.flip_byte(DEVICE_ZERO, writes[first].offset, 100);
        shape("root-landed-record-bad-on-0-older-on-0", &damaged(landed.clone(), &[old_on_zero]), &content, true);
        shape("root-landed-record-bad-on-0-named-on-0", &damaged(landed, &[named_on_zero]), &content, true);
    });
    scenario("lag", || {
        let mut both_records = after_mount.clone();
        both_records.apply_writes(&writes[..=second]);
        let remount = Run::of(&both_records, 0, 0);
        let lagging = crash_with_the_record_only_on(&both_records, &remount.writes_of_the_whole_run(), DEVICE_ONE);
        shape("lag-control", &lagging, &content, true);
        shape("lag-older-on-0", &damaged(lagging.clone(), &[old_on_zero]), &content, true);
    });
    scenario("multi-record", || {
        let second_file: Vec<u8> = (0..100_000usize).map(|index| (index % 241) as u8).collect();
        let write = Run::of_a_sequential_write(
            &before,
            InodeNumber(FIRST_INODE_NUMBER + 1),
            &second_file,
            FIXED_WRITE_TIME_SECONDS + 80,
        );
        let (write_stream, _) = write.writes_after_the_mount();
        let records = write_stream.iter().filter(|write| is_a_journal_record_write(write)).count();
        println!("MODEL_MULTI record_writes={records}");
        let crashed = crash_with_the_last_record_only_on(&write.image_after_the_mount(), &write_stream, DEVICE_ONE);
        shape("multi-record-control", &crashed, &content, true);
        shape("multi-record-older-on-0", &damaged(crashed.clone(), &[old_on_zero]), &content, true);
    });
    scenario("mirror-all-persisted", || {
        shape("older-unit-on-1-all-persisted", &damaged(publish.image_after_everything(), &[old_on_one]), &content, true);
        shape("older-unit-on-0-all-persisted", &damaged(publish.image_after_everything(), &[old_on_zero]), &content, true);
    });
    scenario("alpha", || alpha_prime_raises_the_tail_of_a_stale_disk(&before, &content));
    scenario("fork-same-instance", || {
        for changes in 1..=4 {
            let older = Run::of(&before, changes, FIXED_WRITE_TIME_SECONDS + 400).image_after_everything();
            let newer = Run::of(&before, changes, FIXED_WRITE_TIME_SECONDS + 600).image_after_everything();
            shape(&format!("same-instance-fork-{changes}"), &with_device_zero_of(&newer, &older), &[], false);
        }
    });
    scenario("fork-counter", || counter_fork_sweep(&before));
    println!("MODEL_DONE");
}

/// 旧快照的盘两槽都坏，离线出路断言现行那一版，把 tail 写到这块没收过单元的盘上；再走可写挂载。
fn alpha_prime_raises_the_tail_of_a_stale_disk(before: &MemoryPool, content: &[u8]) {
    let newest = Run::of(before, 2, FIXED_WRITE_TIME_SECONDS + 500).image_after_everything();
    let stale = with_device_zero_of(&newest, before);
    shape("stale-disk-control", &stale, content, false);
    let mut dead_slots = stale.clone();
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    for slot in 0..2 {
        dead_slots
            .devices
            .get_mut(&DEVICE_ZERO)
            .expect("有这块盘")
            .zero_fill(DeviceOffsetInBytes(slot * spacing), 4096);
    }
    shape("stale-disk-dead-slots-before-alpha", &dead_slots, content, false);
    let Some((instance, checkpoint_txg)) =
        read_recovery_fields(&dead_slots, ComparedVersion::TheRecoveredVersion)
            .report
            .effective_root
    else {
        println!("MODEL_ALPHA no_effective_root");
        return;
    };
    let mut devices = sparse_devices_of(&dead_slots);
    let rewritten = rewrite_system_configuration_slots_for_the_asserted_version(
        &parameters(),
        &mut devices,
        AssertedVersion {
            instance,
            checkpoint_txg,
        },
    );
    println!("MODEL_ALPHA outcome={}", first_chars(&format!("{rewritten:?}"), 300));
    shape("stale-disk-after-alpha", &image_of(&devices, &dead_slots), content, true);
}
