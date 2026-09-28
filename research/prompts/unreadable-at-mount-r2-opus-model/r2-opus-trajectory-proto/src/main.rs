//! unreadable-at-mount-r2 云端攻方腿的原型（只在仓副本里跑，不是入库装置）：Y1「连挂轨迹」。
//! 每个起点盘面上连挂 N 次：每次先试可写，做成了就按「用户动作」发 k 次覆盖写、再按收尾方式（崩溃 / 正常卸载）结束；
//! 可写被拒就试只读挂载。逐次记：可写成没成、这一次尝试盘上变没变（录制流多没多一步、两块盘逐字节比）、系统配置各槽的世代号、
//! 被抛弃的根还在不在根环里、它那次发布写出的单元有没有被盖掉、池级 checker 的 I-7.4。每次挂载打一行 `name=r2traj …`。
//! 起点：撕裂轮换走层 0 的枚举域（D 那次发布的流，去掉第 0 段 COW 单元写之后剩下几段逐状态展开，
//! `enumerate_layer0_selecting_versions_observing_each_state`）；单槽持久读坏与被抛弃根的账持续读不出按设备故障造。
#![allow(dead_code, reason = "common 各函数只用到一部分")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    crash_state_devices, geometry, parameters, publish_overwrite_in_process,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state,
    layer0_state_count_with_torn_in_place_overwrites, Layer0SegmentExpansion,
    TearableInPlaceOverwrites,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_format::{SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::cell::RefCell;
use std::rc::Rc;

const MOUNTS_PER_TRAJECTORY: usize = 6;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn slot_spacing() -> u64 {
    u64::from(parameters().geometry.fixed_structure_slot_spacing)
}

// ---------------------------------------------------------------- 持续的读故障（盘坏了的那几段）

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadBack {
    DeviceError,
    Zeros,
}

/// 坏段之后被写过怎么办：`Sticky` 写了照样读坏（坏扇区不重映射），`HealsOnWrite` 被写罩到就好了（盘在写时重映射）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AfterAWrite {
    Sticky,
    HealsOnWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BadRange {
    device: DeviceIdentity,
    offset: u64,
    length: u64,
}

impl BadRange {
    fn overlaps(&self, device: DeviceIdentity, offset: u64, length: u64) -> bool {
        self.device == device && offset < self.offset + self.length && self.offset < offset + length
    }
}

/// 跨几次挂载共用的一份坏段：好了的段从表里删掉，之后的挂载也读得出。
#[derive(Clone)]
struct PersistentFaults(Rc<RefCell<(Vec<BadRange>, ReadBack, AfterAWrite)>>);

impl PersistentFaults {
    fn new(ranges: Vec<BadRange>, read_back: ReadBack, after_a_write: AfterAWrite) -> Self {
        Self(Rc::new(RefCell::new((ranges, read_back, after_a_write))))
    }
    fn remaining(&self) -> usize {
        self.0.borrow().0.len()
    }
}

struct FaultyDevice<Inner: BlockDevice> {
    identity: DeviceIdentity,
    inner: Inner,
    faults: PersistentFaults,
}

impl<Inner: BlockDevice> BlockDevice for FaultyDevice<Inner> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(buffer.len()).expect("长度");
        let state = self.faults.0.borrow();
        let failing: Vec<BadRange> = state
            .0
            .iter()
            .filter(|range| range.overlaps(self.identity, offset.0, length))
            .copied()
            .collect();
        if failing.is_empty() {
            return self.inner.read_at(offset, buffer);
        }
        match state.1 {
            ReadBack::DeviceError => Err(BlockDeviceError::InputOutput(std::io::Error::other("持续读坏的一段"))),
            ReadBack::Zeros => {
                self.inner.read_at(offset, buffer)?;
                for range in failing {
                    let start = range.offset.saturating_sub(offset.0);
                    let end = (range.offset + range.length - offset.0).min(length);
                    buffer[usize::try_from(start).expect("在缓冲区里")..usize::try_from(end).expect("在缓冲区里")].fill(0);
                }
                Ok(())
            }
        }
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.heal(offset.0, u64::try_from(bytes.len()).expect("长度"));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.heal(offset.0, length);
        self.inner.write_zeroes_at(offset, length)
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

impl<Inner: BlockDevice> FaultyDevice<Inner> {
    fn heal(&self, offset: u64, length: u64) {
        let mut state = self.faults.0.borrow_mut();
        if state.2 == AfterAWrite::HealsOnWrite {
            let identity = self.identity;
            state.0.retain(|range| !range.overlaps(identity, offset, length));
        }
    }
}

type Devices = Vec<(DeviceIdentity, FaultyDevice<RecordingBlockDevice<SparseBlockDevice>>)>;

fn devices_over(image: &MemoryPool, faults: &PersistentFaults, stream: &SharedStream) -> Devices {
    crash_state_devices(image, &[], &[], stream)
        .into_iter()
        .map(|(identity, inner)| (identity, FaultyDevice { identity, inner, faults: faults.clone() }))
        .collect()
}

fn image_of(devices: &Devices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    }
}

// ---------------------------------------------------------------- 盘面观察（直接读盘上的字节，不经读故障）

/// 每块盘两槽：`g<世代号>t<journal tail>` 或 `X`（自证不过、fsid 不同）。
fn system_configuration_slots_text(image: &MemoryPool) -> String {
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut parts = Vec::new();
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let mut slots = Vec::new();
        for offset in [0, slot_spacing()] {
            let bytes = image.devices[&device].read(DeviceOffsetInBytes(offset), slot_bytes);
            match singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes) {
                Ok(slot) if slot.immutable.filesystem_identifier == parameters().filesystem_identifier => {
                    slots.push(format!("g{}t{}", slot.quantities.slot_generation, slot.quantities.journal_tail));
                }
                _ => slots.push("X".to_string()),
            }
        }
        parts.push(format!("d{}[{}]", device.0, slots.join(",")));
    }
    parts.join("")
}

fn every_system_configuration_slot_verifies(image: &MemoryPool) -> bool {
    !system_configuration_slots_text(image).contains('X')
}

/// 被抛弃那次发布写出的单元里，此刻两块盘上逐字节已不是它写的那几个。
fn overwritten_units_of(image: &MemoryPool, abandoned: &TransactionOutput) -> usize {
    abandoned
        .units
        .iter()
        .filter(|unit| {
            [DeviceIdentity(0), DeviceIdentity(1)].iter().any(|device| {
                image.devices[device].read(DeviceOffsetInBytes(unit.slot.0 * SLOT_BYTES), unit.bytes.len()) != unit.bytes
            })
        })
        .count()
}

fn root_is_in_the_ring(image: &MemoryPool, abandoned: &TransactionOutput) -> bool {
    let Ok(system_configuration) = singlefs_core::recovery::choose_system_configuration(image) else {
        return false;
    };
    singlefs_core::recovery::readable_roots(
        image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .iter()
    .any(|root| root.instance == abandoned.root.instance && root.checkpoint_txg == abandoned.root.checkpoint_txg)
}

fn i_7_4(image: &MemoryPool) -> String {
    match check_pool_image(image).into_iter().find(|(name, _)| *name == "I-7.4") {
        Some((_, InvariantVerdict::Violated(_))) => "red".to_string(),
        Some((_, verdict)) => format!("{verdict:?}").chars().take(12).collect(),
        None => "absent".to_string(),
    }
}

fn observation(image: &MemoryPool, abandoned: &[TransactionOutput]) -> String {
    let mut text = format!("sc={}", system_configuration_slots_text(image));
    for (index, output) in abandoned.iter().enumerate() {
        text.push_str(&format!(
            " ab{index}=({},{}):in_ring={}:overwritten={}/{}",
            output.root.instance.0,
            output.root.checkpoint_txg.0,
            root_is_in_the_ring(image, output),
            overwritten_units_of(image, output),
            output.units.len()
        ));
    }
    if !abandoned.is_empty() {
        text.push_str(&format!(" I-7.4={}", i_7_4(image)));
    }
    text
}

// ---------------------------------------------------------------- 连挂 N 次

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum End {
    Crash,
    Unmount,
}

#[derive(Clone, Copy, Debug)]
struct Policy {
    publishes_per_writable_mount: usize,
    end: End,
}

const POLICIES: [Policy; 6] = [
    Policy { publishes_per_writable_mount: 0, end: End::Crash },
    Policy { publishes_per_writable_mount: 0, end: End::Unmount },
    Policy { publishes_per_writable_mount: 1, end: End::Crash },
    Policy { publishes_per_writable_mount: 1, end: End::Unmount },
    Policy { publishes_per_writable_mount: 3, end: End::Crash },
    Policy { publishes_per_writable_mount: 3, end: End::Unmount },
];

fn short(text: String, keep: usize) -> String {
    text.chars().take(keep).collect()
}

/// 连挂 `MOUNTS_PER_TRAJECTORY` 次，交回每次的模式（W 可写 / R 只读 / X 都不成）。
fn trajectory(label: &str, start: &MemoryPool, faults: &PersistentFaults, policy: Policy, abandoned: &[TransactionOutput]) -> String {
    let mut image = start.clone();
    let mut modes = String::new();
    let mut refusals_left_the_disk_unchanged = true;
    let mut first_writable: Option<usize> = None;
    let mut first_every_slot_verifies: Option<usize> = None;
    let mut first_abandoned_all_out_of_ring: Option<usize> = None;
    let params = parameters();
    for mount_index in 1..=MOUNTS_PER_TRAJECTORY {
        let before = image.clone();
        let stream = SharedStream::new();
        let mut devices = devices_over(&image, faults, &stream);
        let line = match mount_writable(&params, &mut devices) {
            Err(error) => {
                let operations_by_the_writable_attempt = stream.operations().len();
                let after_writable_attempt = image_of(&devices);
                let changed_by_the_writable_attempt = after_writable_attempt != before;
                let read_only = mount_read_only(&devices);
                let read_only_text = match &read_only {
                    Ok(mounted) => format!(
                        "ok({},{})",
                        mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0
                    ),
                    Err(error) => format!("err({})", short(format!("{error:?}"), 60)),
                };
                let operations_after_read_only = stream.operations().len();
                let after = image_of(&devices);
                let changed_by_the_read_only_mount = after != after_writable_attempt;
                if changed_by_the_writable_attempt || changed_by_the_read_only_mount || operations_after_read_only > 0 {
                    refusals_left_the_disk_unchanged = false;
                }
                modes.push(if read_only.is_ok() { 'R' } else { 'X' });
                image = after;
                format!(
                    "writable=refused[{}] ops_by_writable_attempt={operations_by_the_writable_attempt} disk_changed_by_writable_attempt={changed_by_the_writable_attempt} read_only={read_only_text} ops_after_read_only={operations_after_read_only} disk_changed_by_read_only={changed_by_the_read_only_mount}",
                    short(format!("{error:?}"), 110)
                )
            }
            Ok(mut mounted) => {
                first_writable.get_or_insert(mount_index);
                modes.push('W');
                let mut published = Vec::new();
                for publish_index in 0..policy.publishes_per_writable_mount {
                    let Some(previous) = mounted.current.file_version().cloned() else {
                        published.push("no_file_version".to_string());
                        break;
                    };
                    let result = {
                        let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
                        publish_overwrite(
                            &mut writer,
                            &mut mounted.allocator,
                            &previous,
                            FirstFile {
                                content: &content_of(3000 + 17 * mount_index + publish_index, 50 + mount_index * 7 + publish_index),
                                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1000 + 60 * u64::try_from(mount_index).expect("小"),
                            },
                            mounted.output.instance,
                        )
                    };
                    match result {
                        Ok(output) => {
                            published.push(format!("{}", output.root.checkpoint_txg.0));
                            mounted.current = PoolVersion::WithFile(output);
                        }
                        Err(error) => {
                            published.push(format!("err({})", short(format!("{error:?}"), 60)));
                            break;
                        }
                    }
                }
                let end_text = match policy.end {
                    End::Crash => "crash".to_string(),
                    End::Unmount => match unmount(&params, &mut devices, &mut mounted.allocator, &mut mounted.current, ShadowLedger::On) {
                        Ok(_) => format!("unmount_ok(txg {})", mounted.current.root().checkpoint_txg.0),
                        Err(error) => format!("unmount_err({})", short(format!("{error:?}"), 60)),
                    },
                };
                image = image_of(&devices);
                format!(
                    "writable=ok instance={} effective=({},{}) isolated={:?} abandoned_roots_unreadable={} published=[{}] end={end_text}",
                    mounted.output.instance.0,
                    mounted.output.effective_root.instance.0,
                    mounted.output.effective_root.checkpoint_txg.0,
                    mounted.output.isolated_slots_per_device,
                    mounted.output.abandoned_roots_unreadable,
                    published.join(",")
                )
            }
        };
        if first_every_slot_verifies.is_none() && every_system_configuration_slot_verifies(&image) {
            first_every_slot_verifies = Some(mount_index);
        }
        if first_abandoned_all_out_of_ring.is_none()
            && !abandoned.is_empty()
            && abandoned.iter().all(|output| !root_is_in_the_ring(&image, output))
        {
            first_abandoned_all_out_of_ring = Some(mount_index);
        }
        println!(
            "name=r2traj {label} policy=k{}{:?} mount={mount_index} {line} bad_ranges_left={} after: {}",
            policy.publishes_per_writable_mount,
            policy.end,
            faults.remaining(),
            observation(&image, abandoned)
        );
    }
    let summary = format!(
        "modes={modes} first_writable={} refusals_left_the_disk_unchanged={refusals_left_the_disk_unchanged} every_system_configuration_slot_verifies_from_mount={} abandoned_out_of_ring_from_mount={}",
        first_writable.map_or("none".to_string(), |index| index.to_string()),
        first_every_slot_verifies.map_or("none".to_string(), |index| index.to_string()),
        if abandoned.is_empty() { "n/a".to_string() } else { first_abandoned_all_out_of_ring.map_or("none".to_string(), |index| index.to_string()) }
    );
    println!(
        "name=r2traj-summary {label} policy=k{}{:?} {summary}",
        policy.publishes_per_writable_mount, policy.end
    );
    summary
}

// ---------------------------------------------------------------- 起点

fn second_instance_content(index: usize) -> Vec<u8> {
    content_of(4100 - 100 * index, 3 + index)
}

fn d_content() -> Vec<u8> {
    content_of(3300, 17)
}

/// A（实例 1，txg 3）→ 重开，实例 2 写行 4、暖机 5、B 6、C 7，全部确认返回（与第一轮 `c331_candidates_grid.rs` 的 `standard` 同一段）。
fn standard_history() -> (MemoryPool, Vec<PublishedVersion>, TransactionOutput, singlefs_core::allocator::PoolAllocator) {
    let mut pool = build_pool("r2-opus-trajectory");
    let a = pool.output.clone();
    let mut versions = vec![PublishedVersion {
        instance: a.root.instance,
        checkpoint_txg: a.root.checkpoint_txg,
        content: common::file_content(),
    }];
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("实例 2 可写挂载");
    for version in std::iter::once(&mounted.output.row_publish).chain(mounted.output.warm_up_publishes.iter()) {
        versions.push(PublishedVersion {
            instance: version.root().instance,
            checkpoint_txg: version.root().checkpoint_txg,
            content: common::file_content(),
        });
    }
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    for index in 0..2 {
        let previous = pool.output.clone();
        let next = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &second_instance_content(index),
            FIXED_WRITE_TIME_SECONDS + 60 * (1 + u64::try_from(index).expect("小")),
            InstanceGeneration(2),
        )
        .expect("实例 2 覆盖写");
        versions.push(PublishedVersion {
            instance: next.root.instance,
            checkpoint_txg: next.root.checkpoint_txg,
            content: second_instance_content(index),
        });
        pool.output = next;
    }
    let c = pool.output.clone();
    (pool.memory_pool(), versions, c, pool.allocator.clone())
}

/// 一条写落到盘上之后那一段是新的（1）、旧的（0），还是两样都不是（T，撕裂）。
fn landing_of(state: &MemoryPool, before: &MemoryPool, write: &RetainedWrite) -> char {
    let length = usize::try_from(write.length_in_bytes()).expect("长度");
    let now = state.devices[&write.device].read(write.offset, length);
    let old = before.devices[&write.device].read(write.offset, length);
    let mut new_image = before.clone();
    new_image.apply_writes(std::slice::from_ref(write));
    let new = new_image.devices[&write.device].read(write.offset, length);
    if now == new {
        '1'
    } else if now == old {
        '0'
    } else {
        'T'
    }
}

/// C331 那三格的来处：C 之后实例 2 在同一个进程里发 D (2, 8)，录下这条流。第 0 段（COW 单元写）整段当作已持久并进起点
/// （历史缩短：这一段的崩溃点只是「D 的单元写了一部分」，盘上见证 C 的系统配置没动），其余几段按层 0 的枚举域逐状态展开，
/// 每个状态在这份副本的产品代码上连挂 N 次。
fn part_torn_rotation(states_budget: &mut u64) {
    let (standard, versions, c, allocator_after_c) = standard_history();
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&standard, &[], &[], &stream);
    let d = {
        let params = parameters();
        let mut allocator = allocator_after_c.clone();
        let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &c,
            FirstFile { content: &d_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180 },
            InstanceGeneration(2),
        )
        .expect("D 在进程里发完")
    };
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry());
    let tearable = TearableInPlaceOverwrites::of(&standard, &writes);
    for (segment_index, segment) in segments.iter().enumerate() {
        let kinds: Vec<String> = segment
            .iter()
            .map(|index| {
                format!(
                    "{index}:{:?}@d{}+{}{}",
                    writes[*index].kind,
                    writes[*index].device.0,
                    writes[*index].offset.0,
                    if tearable.contains(*index) { ":tearable" } else { "" }
                )
            })
            .collect();
        println!(
            "name=r2traj-segment stream=d_publish segment={segment_index} writes={} kinds={}",
            segment.len(),
            if segment.len() <= 8 { kinds.join(",") } else { format!("{} writes, kinds {:?}", segment.len(), segment.iter().map(|index| writes[*index].kind).collect::<std::collections::BTreeSet<_>>()) }
        );
    }
    assert!(
        segments[0].iter().all(|index| writes[*index].kind == StepKind::UnitWrite),
        "第 0 段只有 COW 单元写，才能整段并进起点"
    );
    let mut base = standard.clone();
    base.apply_writes(&segments[0].iter().map(|index| writes[*index].clone()).collect::<Vec<_>>());
    let tail_indexes: Vec<usize> = segments[1..].iter().flatten().copied().collect();
    let tail_writes: Vec<RetainedWrite> = tail_indexes.iter().map(|index| writes[*index].clone()).collect();
    let mut next_index = 0usize;
    let tail_segments: Vec<Vec<usize>> = segments[1..]
        .iter()
        .map(|segment| {
            segment
                .iter()
                .map(|_| {
                    next_index += 1;
                    next_index - 1
                })
                .collect()
        })
        .collect();
    let full_expansion = |_segment_index: usize, _segment: &[usize]| Layer0SegmentExpansion::EveryProperSubset;
    let two_state = closed_form_state_count(&tail_segments);
    let with_torn = layer0_state_count_with_torn_in_place_overwrites(&base, &tail_writes, &tail_segments, &full_expansion);
    let full_stream_with_torn = layer0_state_count_with_torn_in_place_overwrites(&standard, &writes, &segments, &full_expansion);
    println!(
        "name=r2traj-plan stream=d_publish_after_segment_0 writes={} segments={} closed_form_state_count={two_state} layer0_state_count_with_torn={with_torn} whole_d_publish_layer0_state_count_with_torn={full_stream_with_torn} d=({},{})",
        tail_writes.len(),
        tail_segments.len(),
        d.root.instance.0,
        d.root.checkpoint_txg.0
    );
    assert!(with_torn <= 1_000_000, "一次全量不超过约 10^6 个状态");
    *states_budget += with_torn;
    let judged_root_index = tail_writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("D 的根槽写在第 0 段之后");
    let mut stream_versions = versions.clone();
    stream_versions.push(PublishedVersion {
        instance: d.root.instance,
        checkpoint_txg: d.root.checkpoint_txg,
        content: d_content(),
    });
    let mut states: Vec<MemoryPool> = Vec::new();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &base,
        &tail_writes,
        &tail_segments,
        judged_root_index,
        &stream_versions,
        &|_segment_index, _segment| true,
        &mut |image: &CrashImage<'_>, _report| {
            let stream = SharedStream::new();
            let devices = crash_state_devices(image.base, image.writes, &image.persisted, &stream);
            states.push(MemoryPool {
                devices: devices
                    .iter()
                    .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                    .collect(),
                device_size_in_bytes: common::IMAGE_BYTES,
            });
        },
    );
    println!(
        "name=r2traj-enumerated stream=d_publish_after_segment_0 states_enumerated={} states_observed={} red_states={}",
        tally.states,
        states.len(),
        tally.findings.red_states
    );
    for (state_index, state) in states.iter().enumerate() {
        let landing: String = tail_writes.iter().map(|write| landing_of(state, &base, write)).collect();
        let label = format!("start=torn_rotation state={state_index} landing={landing}");
        for policy in POLICIES {
            trajectory(&label, state, &PersistentFaults::new(Vec::new(), ReadBack::DeviceError, AfterAWrite::Sticky), policy, &[]);
        }
    }
}

/// 单槽持久读坏：C 确认返回之后的盘面（`standard`），一块盘的一槽系统配置一直读坏（每盘一槽坏是单故障承诺，D22 已定项 8）。
fn part_single_slot(standard: &MemoryPool) {
    let newest_offset_of = |device: DeviceIdentity| {
        let newest = singlefs_core::recovery::verified_system_configuration_slots(
            standard,
            device,
            slot_spacing(),
            &parameters().filesystem_identifier,
        )
        .iter()
        .map(|slot| slot.quantities.slot_generation)
        .max()
        .expect("有");
        (newest % 2) * slot_spacing()
    };
    for (slot_name, device, offset) in [
        ("OlderOnDevice0", DeviceIdentity(0), slot_spacing() - newest_offset_of(DeviceIdentity(0))),
        ("NewestOnDevice0", DeviceIdentity(0), newest_offset_of(DeviceIdentity(0))),
    ] {
        for read_back in [ReadBack::DeviceError, ReadBack::Zeros] {
            for after_a_write in [AfterAWrite::Sticky, AfterAWrite::HealsOnWrite] {
                let label = format!("start=single_slot slot={slot_name} read_back={read_back:?} after_a_write={after_a_write:?}");
                for policy in POLICIES {
                    let faults = PersistentFaults::new(
                        vec![BadRange { device, offset, length: SYSTEM_CONFIGURATION_SLOT_BYTES }],
                        read_back,
                        after_a_write,
                    );
                    trajectory(&label, standard, &faults, policy, &[]);
                }
            }
        }
    }
}

// ---------------------------------------------------------------- C393：被抛弃根的账持续读不出

/// 第一轮 `c393_candidates_grid.rs` 的起点，原样：A、B（实例 1）、重开取号 2、C (2, 8)；崩溃恢复抛弃 C（落到 (2, 7)，实例 3 写行、暖机）、
/// C 写回。`second_abandonment` 时再接一次干净重开、D、第二次崩溃恢复抛弃 D。
fn c393_start(name: &'static str, second_abandonment: bool) -> (MemoryPool, Vec<TransactionOutput>) {
    let mut pool = build_pool(name);
    let first = pool.output.clone();
    pool.output = publish_overwrite_in_process(&mut pool, &first, &content_of(4100, 3), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(1))
        .expect("B");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("取号 2");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    let before = pool.output.clone();
    let c = publish_overwrite_in_process(&mut pool, &before, &content_of(2500, 11), FIXED_WRITE_TIME_SECONDS + 60, InstanceGeneration(2))
        .expect("C");
    let _ = abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &c);
    let mut abandoned = vec![c];
    if second_abandonment {
        let mut devices = pool.reopen_recorded();
        let remounted = mount_writable(&parameters(), &mut devices).expect("干净重开");
        pool.devices = Some(devices);
        pool.allocator = remounted.allocator;
        pool.output = remounted.current.into_file_version().expect("带文件");
        let previous = pool.output.clone();
        let d = publish_overwrite_in_process(&mut pool, &previous, &content_of(2900, 13), FIXED_WRITE_TIME_SECONDS + 120, remounted.output.instance)
            .expect("D");
        let _ = abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &d);
        abandoned.push(d);
    }
    let image = pool.memory_pool();
    println!("name=r2traj-start start={name} {}", observation(&image, &abandoned));
    (image, abandoned)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Account {
    None,
    TreeTable,
    AllocationRecordTreeRoot,
    AllocationRecordTreeNodeBelowTheRoot,
    TreeTableAndAllocationRecordTreeRoot,
}

fn account_ranges(output: &TransactionOutput, account: Account) -> Vec<BadRange> {
    let pick = |matches: &dyn Fn(&TransactionUnit) -> bool| -> Vec<(u64, u64)> {
        output
            .units
            .iter()
            .filter(|unit| matches(&unit.identity))
            .take(1)
            .map(|unit| (unit.slot.0, u64::try_from(unit.bytes.len()).expect("长度")))
            .collect()
    };
    let units = match account {
        Account::None => Vec::new(),
        Account::TreeTable => pick(&|identity| *identity == TransactionUnit::TreeTable),
        Account::AllocationRecordTreeRoot => pick(&|identity| *identity == TransactionUnit::AllocationTree),
        Account::AllocationRecordTreeNodeBelowTheRoot => {
            pick(&|identity| matches!(identity, TransactionUnit::AllocationTreeNodeBelowTheRoot(_)))
        }
        Account::TreeTableAndAllocationRecordTreeRoot => {
            let mut units = pick(&|identity| *identity == TransactionUnit::TreeTable);
            units.extend(pick(&|identity| *identity == TransactionUnit::AllocationTree));
            units
        }
    };
    units
        .into_iter()
        .flat_map(|(slot, length)| {
            [DeviceIdentity(0), DeviceIdentity(1)].into_iter().map(move |device| BadRange { device, offset: slot * SLOT_BYTES, length })
        })
        .collect()
}

fn part_c393(start_name: &'static str, second_abandonment: bool) {
    let (image, abandoned) = c393_start(start_name, second_abandonment);
    let whiches: Vec<&'static [usize]> = if second_abandonment { vec![&[0], &[1], &[0, 1]] } else { vec![&[0]] };
    let accounts: &[Account] = if second_abandonment {
        &[Account::TreeTable, Account::AllocationRecordTreeRoot]
    } else {
        &[
            Account::TreeTable,
            Account::AllocationRecordTreeRoot,
            Account::AllocationRecordTreeNodeBelowTheRoot,
            Account::TreeTableAndAllocationRecordTreeRoot,
        ]
    };
    for policy in POLICIES {
        let label = format!("start={start_name} which=[] account=None read_back=- after_a_write=-");
        trajectory(&label, &image, &PersistentFaults::new(Vec::new(), ReadBack::DeviceError, AfterAWrite::Sticky), policy, &abandoned);
    }
    for which in whiches {
        for account in accounts {
            for read_back in [ReadBack::DeviceError, ReadBack::Zeros] {
                for after_a_write in [AfterAWrite::Sticky, AfterAWrite::HealsOnWrite] {
                    let ranges: Vec<BadRange> = which.iter().flat_map(|index| account_ranges(&abandoned[*index], *account)).collect();
                    let label = format!(
                        "start={start_name} which={which:?} account={account:?} read_back={read_back:?} after_a_write={after_a_write:?} bad_ranges={}",
                        ranges.len()
                    );
                    for policy in POLICIES {
                        trajectory(&label, &image, &PersistentFaults::new(ranges.clone(), read_back, after_a_write), policy, &abandoned);
                    }
                }
            }
        }
    }
}

fn main() {
    let part = std::env::args().nth(1).expect("用法：r2-opus-trajectory-proto <torn|single_slot|c393_single|c393_double>");
    let started = std::time::Instant::now();
    let mut states_budget = 0u64;
    match part.as_str() {
        "torn" => part_torn_rotation(&mut states_budget),
        "single_slot" => {
            let (standard, _, _, _) = standard_history();
            println!("name=r2traj-start start=single_slot sc={}", system_configuration_slots_text(&standard));
            part_single_slot(&standard);
        }
        "c393_single" => part_c393("c393-single", false),
        "c393_double" => part_c393("c393-double", true),
        other => panic!("不认得的部分 {other}"),
    }
    println!("name=r2traj-total part={part} layer0_states_enumerated_by_this_run={states_budget} seconds={}", started.elapsed().as_secs());
}
