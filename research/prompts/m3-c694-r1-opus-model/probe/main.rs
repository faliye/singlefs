//! m3-c694-r1 攻方原型（Opus）。臂由环境变量选（见模型目录 make_patched.py）：OPUS_C694_ARM = today / A / B / C / B+C，
//! OPUS_C695_ARM = today / R / S。子命令：
//!   counts                      每个起点两条流（录的那次覆盖写、取新实例那次挂载）的层 0 全量状态数（closed_form_state_count）
//!   grid <宽> <k> <k2max>        H1：录的那次覆盖写每个崩溃状态（层 0 枚举）× 挂载之后再覆盖写 0..=k2max 次，每一步在拷贝上卸载、再挂再卸载、
//!                                再把新实例的根全抹掉看恢复落到被抛弃的根上读不读得出
//!   transient <宽> <k> <k2max>   H2：被抛弃的根挂载时读得出、账也读得出的起点上，卸载那一刻它的树表与映射根两槽读报错（瞬时）
//!   crashmount <宽> <k>          H5：取新实例那次挂载的写流（层 0 枚举）每个崩溃状态恢复，看落到哪条根、读不读得出
//!   c695                         K3：几种「树表 0 条」的版本逐条打开
#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use std::collections::{BTreeMap, BTreeSet};

use common_admission::{
    checker_violations_on, plain_devices_on, start_plain, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{mount_writable_with_space_admission, unmount, MountError, Unmounted};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, every_root_ring_slot,
    read_root_ring_slot, recover, JournalPolicy, PoolReader, RecoveryOutcome, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes, MemoryPool,
    PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const RECORDED_OVERWRITE_TIME: u64 = common_admission::FIRST_WRITE_TIME_SECONDS + 100_000;
const RECORDED_CONTENT_BYTE: u8 = 0x5a;

/// 读点名的几段读回全 0（盘上的字节不动）；被写到的那一段不再藏。
struct Hiding<D: BlockDevice> {
    inner: D,
    hidden: Vec<(u64, u64)>,
}

fn overlaps(start: u64, len: u64, offset: u64, length: u64) -> bool {
    start < offset + length && offset < start + len
}

impl<D: BlockDevice> BlockDevice for Hiding<D> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let length = buffer.len() as u64;
        for &(start, len) in &self.hidden {
            if overlaps(start, len, offset.0, length) {
                let a = start.saturating_sub(offset.0) as usize;
                let b = ((start + len - offset.0).min(length)) as usize;
                buffer[a..b].fill(0);
            }
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let length = bytes.len() as u64;
        self.hidden.retain(|&(start, len)| !overlaps(start, len, offset.0, length));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.hidden.retain(|&(start, len)| !overlaps(start, len, offset.0, length));
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

/// 读点名的几段报读错（瞬时故障：只在这一个设备对象活着时）。
struct Failing {
    inner: SparseBlockDevice,
    failing: Vec<(u64, u64)>,
}

impl BlockDevice for Failing {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        if self.failing.iter().any(|&(start, len)| overlaps(start, len, offset.0, buffer.len() as u64)) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("opus 注入的瞬时读错")));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
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

fn materialize(base: &MemoryPool, writes: &[RetainedWrite], persisted: &[bool]) -> MemoryPool {
    let mut pool = base.clone();
    for (write, is_persisted) in writes.iter().zip(persisted) {
        if *is_persisted {
            write.contents.apply_to(pool.devices.get_mut(&write.device).expect("盘"), write.offset);
        }
    }
    pool
}

fn widths(code: &str) -> HistoryDeviceWidth {
    match code {
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
        "240" => HistoryDeviceWidth::UnitAreaOf240Slots,
        "4g" => HistoryDeviceWidth::FourGibibytes,
        other => panic!("宽只认 384 / 256 / 240 / 4g：{other}"),
    }
}

fn arm() -> String {
    let arm = std::env::var("OPUS_C694_ARM").unwrap_or_default();
    if arm.is_empty() { "today".to_string() } else { arm }
}

/// 起点前缀：第一个文件之后在会话里覆盖写 k 次，然后在盘面拷贝上录一次覆盖写（内容全 0x5a）。会话留着（前缀那一版），盘没动。
struct Recorded {
    pool: PoolUnderTest<SparseBlockDevice>,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    root_index: usize,
    recorded: (InstanceGeneration, CheckpointTxg),
}

fn record_one_overwrite(width: HistoryDeviceWidth, k: u64) -> Result<Recorded, String> {
    let mut pool = start_plain(width);
    for i in 0..k {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite{i}:{e:?}"))?;
    }
    let base = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&base)
        .into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone())))
        .collect();
    let recorded = {
        let session = pool.session.as_ref().expect("会话");
        let mut allocator = session.allocator.clone();
        let previous = session.current.file_version().expect("带文件").clone();
        let mut writer = PoolWriter::new(&pool.parameters, devices.as_mut_slice());
        let content = vec![RECORDED_CONTENT_BYTE; OVERWRITE_BYTES];
        let root = publish_overwrite(
            &mut writer,
            &mut allocator,
            &previous,
            FirstFile { content: &content, write_time_seconds: RECORDED_OVERWRITE_TIME },
            session.instance,
        )
        .map_err(|e| format!("recorded-overwrite:{e:?}"))?
        .root;
        (root.instance, root.checkpoint_txg)
    };
    pool.content_of_each_root.insert(recorded, vec![RECORDED_CONTENT_BYTE; OVERWRITE_BYTES]);
    let (writes, segments, _) =
        writes_and_segments_with_stream_indexes(&stream.retained_operations(), &width.fixed_geometry());
    let root_index = writes
        .iter()
        .position(|w| w.kind == StepKind::RootRecordFua)
        .ok_or("没有根槽写")?;
    Ok(Recorded { pool, base, writes, segments, root_index, recorded })
}

fn tally_line(label: &str, tally: &singlefs_checker_tier::crash::Layer0Tally) -> String {
    let checker: Vec<String> = tally
        .checker_violated_states
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(name, count)| format!("{name}={count}"))
        .collect();
    format!(
        "L0TALLY {label} states={} root_persisted={} violations={} failed={} no_file={} file_read={} checker_violated=[{}] first_violation={}",
        tally.states,
        tally.root_persisted_states,
        tally.violations,
        tally.failed_states,
        tally.no_file_states,
        tally.file_read_states,
        checker.join(","),
        tally
            .first_violation
            .as_deref()
            .map(|text| text.chars().take(160).collect::<String>())
            .unwrap_or_else(|| "-".to_string())
    )
}

/// 展不展开这一段：OPUS_FULL=1 时每一段都展开（层 0 全量）；不设时比 OPUS_SEGMENT_LIMIT（默认 12）个写多的段不展开
/// （只以整段持久进入后面的状态，前台 4 分钟跑不完全量时的缩法，报告里写明）。
fn expand_segment(_segment_index: usize, segment: &[usize]) -> bool {
    if std::env::var("OPUS_FULL").is_ok_and(|v| v == "1") {
        return true;
    }
    let limit: usize = std::env::var("OPUS_SEGMENT_LIMIT").ok().and_then(|v| v.parse().ok()).unwrap_or(12);
    segment.len() <= limit
}

/// 层 0 枚举一条流（展开哪几段见 [`expand_segment`]）；每个状态的持久集合交回（观察者按状态序号从小到大调）。
fn every_crash_state(
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    label: &str,
) -> Vec<Vec<bool>> {
    let mut states = Vec::new();
    let tally = singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state(
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &|segment_index, segment| expand_segment(segment_index, segment),
        &mut |image, _report| states.push(image.persisted.clone()),
    );
    println!("{}", tally_line(label, &tally));
    assert_eq!(states.len() as u64, tally.states, "观察者看过每一个状态");
    states
}

/// 录的那次发布在这个持久集合里落了盘的单元写与根槽写（每块盘的区间）：取新实例那次挂载时让它们读回全 0。
fn hidden_ranges_of(writes: &[RetainedWrite], persisted: &[bool], device: DeviceIdentity) -> Vec<(u64, u64)> {
    writes
        .iter()
        .zip(persisted)
        .filter(|(w, p)| **p && w.device == device && (w.kind == StepKind::UnitWrite || w.kind == StepKind::RootRecordFua
            || (w.kind == StepKind::JournalRecord && std::env::var("OPUS_HIDE_RECORD").is_ok_and(|v| v == "1"))))
        .map(|(w, _)| (w.offset.0, w.length_in_bytes()))
        .collect()
}

fn subcommand_counts() {
    for code in ["384", "256", "240", "4g"] {
        for k in 0..=10_u64 {
            match record_one_overwrite(widths(code), k) {
                Ok(recorded) => println!(
                    "COUNT recorded W={code} k={k} recorded={:?} writes={} segments={} segment_sizes={:?} closed_form={}",
                    recorded.recorded,
                    recorded.writes.len(),
                    recorded.segments.len(),
                    recorded.segments.iter().map(Vec::len).collect::<Vec<_>>(),
                    closed_form_state_count(&recorded.segments)
                ),
                Err(error) => println!("COUNT recorded W={code} k={k} failed={error}"),
            }
        }
    }
}

fn image_of<D>(devices: &[(DeviceIdentity, D)], device_bytes: u64, sparse: impl Fn(&D) -> &SparseBlockDevice) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(id, d)| (*id, sparse(d).image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    }
}

/// 取新实例那次挂载（在崩溃状态的镜像上，录的那次发布落了盘的单元写与根槽读回全 0）做成之后的池：会话是这次挂载的。
struct AfterTheAbandoningMount {
    pool: PoolUnderTest<SparseBlockDevice>,
    summary: String,
}

fn pool_from(recorded: &Recorded, image: &MemoryPool, session: MountedSession, content_now: Vec<u8>) -> PoolUnderTest<SparseBlockDevice> {
    PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices: plain_devices_on(image),
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_now,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    }
}

fn abandoning_mount(recorded: &Recorded, persisted: &[bool]) -> Result<AfterTheAbandoningMount, String> {
    let crash_image = materialize(&recorded.base, &recorded.writes, persisted);
    let mut devices: Vec<(DeviceIdentity, Hiding<SparseBlockDevice>)> = plain_devices_on(&crash_image)
        .into_iter()
        .map(|(id, d)| (id, Hiding { inner: d, hidden: hidden_ranges_of(&recorded.writes, persisted, id) }))
        .collect();
    let mounted = mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
        .map_err(|e| format!("abandoning-mount:{}", format!("{e:?}").chars().take(160).collect::<String>()))?;
    let (output, session) = MountedSession::of_the_mount(mounted);
    let image = image_of(&devices, recorded.pool.device_bytes, |d| &d.inner);
    let effective = (output.effective_root.instance, output.effective_root.checkpoint_txg);
    let content_now = recorded.pool.content_of_each_root.get(&effective).cloned().unwrap_or_default();
    let mut pool = pool_from(recorded, &image, session, content_now.clone());
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        pool.content_of_each_root.insert((root.instance, root.checkpoint_txg), content_now.clone());
    }
    let summary = format!(
        "effective=({},{}) row_txg={} mount_abandoned_unreadable={} mount_isolated={:?}",
        effective.0 .0,
        effective.1 .0,
        output.row_publish.root().checkpoint_txg.0,
        output.abandoned_roots_unreadable,
        output.isolated_slots_per_device.iter().map(|(d, n)| (d.0, *n)).collect::<Vec<_>>()
    );
    Ok(AfterTheAbandoningMount { pool, summary })
}

fn ring_root(image: &MemoryPool, key: (InstanceGeneration, CheckpointTxg)) -> Option<RootRecord> {
    let configuration = choose_system_configuration(image).ok()?;
    every_root_ring_slot(&configuration.immutable.sizes).into_iter().find_map(|ring_slot| {
        match read_root_ring_slot(image, &configuration.immutable.region_devices, &configuration.immutable.sizes,
            &configuration.immutable.filesystem_identifier, ring_slot) {
            RootRingSlotReading::SelfVerified(root) if (root.instance, root.checkpoint_txg) == key => Some(root),
            _ => None,
        }
    })
}

/// 被抛弃的根引用的落点（崩溃那一刻读它的账，仍分配着的那几条；读不出交空）。
fn placements_of(image: &MemoryPool, root: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    allocation_records_under_root(image, root)
        .map(|records| records.into_iter().filter(|r| !r.is_released).map(|r| (r.device, r.slot.0, u64::from(r.span_slots))).collect())
        .unwrap_or_default()
}

fn changed_placements(before: &MemoryPool, after: &MemoryPool, placements: &[(DeviceIdentity, u64, u64)]) -> usize {
    placements
        .iter()
        .filter(|(device, slot, span)| {
            let offset = DeviceOffsetInBytes(slot * SLOT_BYTES);
            let length = usize::try_from(span * SLOT_BYTES).expect("跨度");
            PoolReader::read(before, *device, offset, length) != PoolReader::read(after, *device, offset, length)
        })
        .count()
}

/// R 的可读性探针：盘面拷贝上把实例代号高于 R 的根槽全写零（「新实例的根全坏了」），再恢复，看落到哪条根、读出什么。
fn probe_recovery_onto(image: &MemoryPool, abandoned: (InstanceGeneration, CheckpointTxg), expected: &[u8]) -> String {
    let Ok(configuration) = choose_system_configuration(image) else { return "no-sysconf".to_string() };
    let mut copy = image.clone();
    let mut devices = plain_devices_on(&copy);
    for ring_slot in every_root_ring_slot(&configuration.immutable.sizes) {
        if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(image, &configuration.immutable.region_devices,
            &configuration.immutable.sizes, &configuration.immutable.filesystem_identifier, ring_slot) {
            if root.instance > abandoned.0 {
                let device = configuration.immutable.region_devices[usize::try_from(ring_slot.region).expect("区域")];
                let offset = singlefs_core::root_ring::slot_offset(ring_slot, configuration.immutable.sizes.fixed_structure_slot_spacing);
                let length = u64::from(configuration.immutable.sizes.physical_block_size);
                for (id, d) in devices.iter_mut() {
                    if *id == device {
                        d.write_zeroes_at(offset, length).expect("写零");
                    }
                }
            }
        }
    }
    copy = image_of(&devices, image.device_size_in_bytes, |d| d);
    let report = recover(&copy, JournalPolicy::Consult);
    let landed = report.effective_root.map(|(i, t)| (i.0, t.0));
    match report.outcome {
        RecoveryOutcome::FileRead { content, .. } if landed == Some((abandoned.0 .0, abandoned.1 .0)) => {
            if content == expected { "onR:read-ok".to_string() } else { "onR:WRONG-CONTENT".to_string() }
        }
        RecoveryOutcome::Failed { failure, .. } if landed == Some((abandoned.0 .0, abandoned.1 .0)) || landed.is_none() => {
            format!("onR:FAILED({})", format!("{failure:?}").chars().take(60).collect::<String>())
        }
        _ => format!("not-on-R:{landed:?}"),
    }
}

fn short(result: &Result<Unmounted, MountError>) -> String {
    match result {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!("Ok({})", raised.publishes.len()),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "Ok(nothing)".to_string(),
        Err(MountError::NewerStateStillUnreadableAfterOneReread(still)) => {
            let text = format!("{still:?}");
            format!("Err(StillUnreadable:{})", text.split([' ', '{']).next().unwrap_or(""))
        }
        Err(error) => format!("Err({})", format!("{error:?}").split([' ', '(', '{']).next().unwrap_or("")),
    }
}

/// 在这一刻盘面的拷贝上卸载（会话的分配器与现行版本各拷一份），交回结局与卸载之后的盘面。
fn unmount_on_a_copy<D: BlockDevice>(
    pool: &PoolUnderTest<SparseBlockDevice>,
    wrap: impl Fn(DeviceIdentity, SparseBlockDevice) -> D,
    unwrap: impl Fn(&D) -> &SparseBlockDevice,
) -> (Result<Unmounted, MountError>, MemoryPool) {
    let mut devices: Vec<(DeviceIdentity, D)> =
        plain_devices_on(&pool.image()).into_iter().map(|(id, d)| (id, wrap(id, d))).collect();
    let session = pool.session.as_ref().expect("会话");
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    let outcome = unmount(&pool.parameters, &mut devices, &mut allocator, &mut current, session.shadow_ledger);
    (outcome, image_of(&devices, pool.device_bytes, unwrap))
}

/// 进程退出，在这份盘面的拷贝上再可写挂载一次，接着卸载。
fn remount_then_unmount(pool: &PoolUnderTest<SparseBlockDevice>, image: &MemoryPool) -> String {
    let mut devices = plain_devices_on(image);
    match mount_writable_with_space_admission(&pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(mounted) => {
            let (output, mut session) = MountedSession::of_the_mount(mounted);
            let unmounted = unmount(&pool.parameters, &mut devices, &mut session.allocator, &mut session.current, session.shadow_ledger);
            format!("Ok(unreadable={})+unmount={}", output.abandoned_roots_unreadable, short(&unmounted))
        }
        Err(error) => short(&Err(error)),
    }
}

/// 几个线程按跨步分活，结果按序号排回。
fn parallel_map<T: Sync, R: Send>(items: &[T], f: impl Fn(usize, &T) -> R + Sync) -> Vec<R> {
    let threads: usize = std::env::var("OPUS_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(1).max(1);
    let mut results: Vec<(usize, R)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let f = &f;
                scope.spawn(move || {
                    items.iter().enumerate().skip(t).step_by(threads).map(|(i, item)| (i, f(i, item))).collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().expect("工作线程")).collect()
    });
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, r)| r).collect()
}

fn checker_names(image: &MemoryPool) -> String {
    let names: BTreeSet<String> = checker_violations_on(image)
        .into_iter()
        .map(|line| line.split(':').next().unwrap_or("").to_string())
        .collect();
    names.into_iter().collect::<Vec<_>>().join(",")
}

/// H1：录的那次覆盖写的每个崩溃状态上做取新实例那次挂载，之后会话覆盖写 0..=k2max 次；每一步：在拷贝上卸载、卸载之后的盘面跑 checker、
/// 被抛弃的根还在环里就做 R 的可读性探针；再在拷贝上重开可写挂载接着卸载。
fn subcommand_grid(code: &str, k: u64, k2max: u64) {
    let recorded = match record_one_overwrite(widths(code), k) {
        Ok(recorded) => recorded,
        Err(error) => return println!("H1 W={code} k={k} PREFIX_FAILED {error}"),
    };
    let versions = recorded.pool.published_versions();
    let label = format!("recorded W={code} k={k} arm={}", arm());
    let states = every_crash_state(&recorded.base, &recorded.writes, &recorded.segments, recorded.root_index, &versions, &label);
    let r_content = vec![RECORDED_CONTENT_BYTE; OVERWRITE_BYTES];
    let lines = parallel_map(&states, |ordinal, persisted| {
        let mut lines = Vec::new();
        let root_landed = persisted[recorded.root_index];
        let crash_image = materialize(&recorded.base, &recorded.writes, persisted);
        let r_root = ring_root(&crash_image, recorded.recorded);
        let r_placements = r_root.as_ref().map(|r| placements_of(&crash_image, r)).unwrap_or_default();
        let head = format!("W={code} k={k} arm={} root_landed={root_landed} r_in_ring_at_crash={}", arm(), r_root.is_some());
        let mut after = match abandoning_mount(&recorded, persisted) {
            Ok(after) => after,
            Err(error) => {
                lines.push((format!("{head} MOUNT_ERR {}", error.chars().take(90).collect::<String>()), ordinal, String::new()));
                return lines;
            }
        };
        for step in 0..=k2max {
            let image = after.pool.image();
            let in_ring = ring_root(&image, recorded.recorded).is_some();
            let damaged = changed_placements(&crash_image, &image, &r_placements);
            let (unmounted, image_after_unmount) = unmount_on_a_copy(&after.pool, |_, d| d, |d| d);
            let probe = if in_ring && unmounted.is_ok() {
                probe_recovery_onto(&image_after_unmount, recorded.recorded, &r_content)
            } else {
                "-".to_string()
            };
            let checker = if unmounted.is_ok() { checker_names(&image_after_unmount) } else { checker_names(&image) };
            let remount = remount_then_unmount(&after.pool, &image);
            lines.push((
                format!(
                    "{head} step={step} in_ring={in_ring} r_placements={} damaged_before_unmount={damaged} unmount={} remount={remount} probe={probe} checker=[{checker}]",
                    r_placements.len(),
                    short(&unmounted)
                ),
                ordinal,
                after.summary.clone(),
            ));
            if step < k2max {
                if let Err(error) = after.pool.overwrite(OVERWRITE_BYTES) {
                    let text = format!("{error:?}");
                    let kind = text.split([' ', '(', '{']).next().unwrap_or("").to_string();
                    let cause = if text.contains("AccountOfAnAbandonedRoot") { "+AccountOfAnAbandonedRoot" } else { "" };
                    lines.push((format!("{head} step={} OVERWRITE_FAILED {kind}{cause}", step + 1), ordinal, String::new()));
                    break;
                }
            }
        }
        lines
    });
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for (line, ordinal, summary) in lines.into_iter().flatten() {
        if line.contains("root_landed=true") {
            println!("H1DETAIL state={ordinal} {line} mount[{summary}]");
        }
        *counts.entry(line).or_default() += 1;
    }
    for (line, count) in counts {
        println!("H1 {line} states={count}");
    }
}

/// 被抛弃根 R 的账那两个单元（树表、中央映射根）在两块盘上的区间：瞬时读错注在这里。
fn account_ranges_of(root: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    let mut ranges = Vec::new();
    for pointer in [&root.tree_table, &root.mapping_root] {
        for location in &pointer.locations {
            ranges.push((location.device, location.slot.0 * SLOT_BYTES, SLOT_BYTES));
        }
    }
    ranges
}

fn failing_on(ranges: &[(DeviceIdentity, u64, u64)]) -> impl Fn(DeviceIdentity, SparseBlockDevice) -> Failing + '_ {
    move |id, d| Failing {
        inner: d,
        failing: ranges.iter().filter(|(device, _, _)| *device == id).map(|(_, start, len)| (*start, *len)).collect(),
    }
}

/// 会话里覆盖写一次，这一次调用期间 `ranges` 的读报错（这一次若推了抬 F，影子账就在读错下重算）。
fn overwrite_under_read_errors(pool: &mut PoolUnderTest<SparseBlockDevice>, ranges: &[(DeviceIdentity, u64, u64)]) -> String {
    let mut session = pool.session.take().expect("会话");
    let mut devices: Vec<(DeviceIdentity, Failing)> =
        plain_devices_on(&pool.image()).into_iter().map(|(id, d)| (id, failing_on(ranges)(id, d))).collect();
    pool.write_time_seconds += 1;
    let content = common_admission::content_of(OVERWRITE_BYTES, pool.write_time_seconds);
    let outcome = session.publish_user_change(
        &mut devices,
        singlefs_core::mounted_session::UserChange::Overwrite(FirstFile { content: &content, write_time_seconds: pool.write_time_seconds }),
    );
    pool.session = Some(session);
    pool.devices = plain_devices_on(&image_of(&devices, pool.device_bytes, |d| &d.inner));
    match outcome {
        Ok(published) => {
            pool.content_of_the_current_version = content;
            format!("Ok(raises={})", published.floor_raises.len())
        }
        Err(error) => {
            let text = format!("{error:?}");
            format!("Err({}{})", text.split([' ', '(', '{']).next().unwrap_or(""), if text.contains("AccountOfAnAbandonedRoot") { "+Account" } else { "" })
        }
    }
}

/// 卸载（拷贝上）：先在读错下卸一次，拒了就撤读错再卸一次（U2 那条路）；交回两次的结局与做成那一次之后的盘面。
fn unmount_under_read_errors_then_retry(pool: &PoolUnderTest<SparseBlockDevice>, ranges: &[(DeviceIdentity, u64, u64)]) -> (String, Option<MemoryPool>) {
    let (first, image_first) = unmount_on_a_copy(pool, failing_on(ranges), |d| &d.inner);
    if first.is_ok() {
        return (format!("{}", short(&first)), Some(image_first));
    }
    let (retry, image_retry) = unmount_on_a_copy(pool, |_, d| d, |d| d);
    let image = retry.is_ok().then_some(image_retry);
    (format!("{}->retry:{}", short(&first), short(&retry)), image)
}

/// H2：被抛弃的根在重开时读得出、账读得出的起点。取新实例那次挂载之后卸载（无故障）、再可写挂载（这次挂载的影子账隔离了 R），
/// 之后会话覆盖写 0..=k2max 次，每一步两种瞬时读错：(u) 卸载那一刻 R 的树表与映射根两槽读报错；(w) 下一次覆盖写那一次调用期间读报错、
/// 之后再无故障覆盖写 2 次、卸载。看 R 引用的落点有没有被写掉、R 的可读性探针。
fn subcommand_transient(code: &str, k: u64, k2max: u64) {
    let recorded = match record_one_overwrite(widths(code), k) {
        Ok(recorded) => recorded,
        Err(error) => return println!("H2 W={code} k={k} PREFIX_FAILED {error}"),
    };
    let versions = recorded.pool.published_versions();
    let label = format!("recorded W={code} k={k} arm={} (H2)", arm());
    let states = every_crash_state(&recorded.base, &recorded.writes, &recorded.segments, recorded.root_index, &versions, &label);
    let r_content = vec![RECORDED_CONTENT_BYTE; OVERWRITE_BYTES];
    let lines = parallel_map(&states, |_ordinal, persisted| {
        let mut lines = Vec::new();
        let crash_image = materialize(&recorded.base, &recorded.writes, persisted);
        let Some(r_root) = ring_root(&crash_image, recorded.recorded) else { return lines };
        let r_placements = placements_of(&crash_image, &r_root);
        let ranges = account_ranges_of(&r_root);
        let head = format!("W={code} k={k} arm={}", arm());
        let Ok(after) = abandoning_mount(&recorded, persisted) else {
            lines.push(format!("{head} MOUNT_ERR"));
            return lines;
        };
        let (first_unmount, image) = unmount_on_a_copy(&after.pool, |_, d| d, |d| d);
        if first_unmount.is_err() {
            lines.push(format!("{head} FIRST_UNMOUNT {}", short(&first_unmount)));
            return lines;
        }
        let damaged_at_m2 = changed_placements(&crash_image, &image, &r_placements);
        let mut devices = plain_devices_on(&image);
        let mounted = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
            Ok(mounted) => mounted,
            Err(error) => {
                lines.push(format!("{head} M2 {}", short(&Err(error))));
                return lines;
            }
        };
        let (output, session) = MountedSession::of_the_mount(mounted);
        let mut pool = pool_from(&recorded, &image_of(&devices, recorded.pool.device_bytes, |d| d), session, Vec::new());
        let isolated: u64 = output.isolated_slots_per_device.iter().map(|(_, n)| *n).sum();
        for step in 0..=k2max {
            let base_image = pool.image();
            let damaged_now = changed_placements(&crash_image, &base_image, &r_placements);
            // (u) 卸载那一刻读错。
            let (outcome_u, image_u) = unmount_under_read_errors_then_retry(&pool, &ranges);
            let (damage_u, probe_u) = match &image_u {
                Some(image) => (changed_placements(&crash_image, image, &r_placements).to_string(), probe_recovery_onto(image, recorded.recorded, &r_content)),
                None => ("-".to_string(), "-".to_string()),
            };
            lines.push(format!("{head} m2_isolated={isolated} damaged_at_m2={damaged_at_m2} step={step} damaged_now={damaged_now} (u)unmount={outcome_u} damaged_after={damage_u} probe={probe_u}"));
            // (w) 下一次覆盖写那一次调用期间读错，之后无故障覆盖写 2 次、卸载。
            {
                let mut branch = PoolUnderTest {
                    parameters: pool.parameters.clone(),
                    device_bytes: pool.device_bytes,
                    devices: plain_devices_on(&pool.image()),
                    session: Some(MountedSession {
                        allocator: pool.session().allocator.clone(),
                        current: pool.session().current.clone(),
                        instance: pool.session().instance,
                        shadow_ledger: pool.session().shadow_ledger,
                        parameters_and_device_table: pool.session().parameters_and_device_table.clone(),
                    }),
                    write_time_seconds: pool.write_time_seconds + 500,
                    content_of_the_current_version: Vec::new(),
                    content_of_each_root: BTreeMap::new(),
                };
                let after_op = |branch: &PoolUnderTest<SparseBlockDevice>| {
                    let image = branch.image();
                    format!("[in_ring={} dmg={} chk={}]", ring_root(&image, recorded.recorded).is_some(),
                        changed_placements(&crash_image, &image, &r_placements), checker_names(&image))
                };
                let faulty = format!("{}{}", overwrite_under_read_errors(&mut branch, &ranges), after_op(&branch));
                let mut tail = Vec::new();
                for _ in 0..2 {
                    let outcome = match branch.overwrite(OVERWRITE_BYTES) { Ok(p) => format!("Ok(raises={})", p.floor_raises.len()), Err(e) => format!("Err({})", format!("{e:?}").split([' ', '(', '{']).next().unwrap_or("")) };
                    tail.push(format!("{outcome}{}", after_op(&branch)));
                }
                let (outcome, image) = unmount_on_a_copy(&branch, |_, d| d, |d| d);
                let (damage, probe) = if outcome.is_ok() {
                    (changed_placements(&crash_image, &image, &r_placements).to_string(), probe_recovery_onto(&image, recorded.recorded, &r_content))
                } else {
                    (changed_placements(&crash_image, &branch.image(), &r_placements).to_string(), "-".to_string())
                };
                lines.push(format!("{head} m2_isolated={isolated} step={step} (w)faulty_overwrite={faulty} then={} unmount={} damaged_after={damage} probe={probe}", tail.join(","), short(&outcome)));
            }
            if step < k2max {
                if let Err(error) = pool.overwrite(OVERWRITE_BYTES) {
                    lines.push(format!("{head} step={} OVERWRITE_FAILED {}", step + 1, format!("{error:?}").split([' ', '(', '{']).next().unwrap_or("")));
                    break;
                }
            }
        }
        lines
    });
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for line in lines.into_iter().flatten() {
        *counts.entry(line).or_default() += 1;
    }
    for (line, count) in counts {
        println!("H2 {line} states={count}");
    }
}

/// H5：录的那次覆盖写崩在根槽落盘那一刀（之前全持久、之后一个没持久），取新实例那次挂载（R 的根槽与单元读回全 0）的写流录下来，
/// 层 0 全量枚举它的每个崩溃状态、撤故障恢复：落到 R 上读不读得出（发布次序上 R 在新实例的写行根落盘之前一直是最新的根）。
fn subcommand_crashmount(code: &str, k: u64) {
    let recorded = match record_one_overwrite(widths(code), k) {
        Ok(recorded) => recorded,
        Err(error) => return println!("H5 W={code} k={k} PREFIX_FAILED {error}"),
    };
    let persisted: Vec<bool> = (0..recorded.writes.len()).map(|i| i <= recorded.root_index).collect();
    let crash_image = materialize(&recorded.base, &recorded.writes, &persisted);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, Hiding<RecordingBlockDevice<SparseBlockDevice>>)> = plain_devices_on(&crash_image)
        .into_iter()
        .map(|(id, d)| (id, Hiding { inner: RecordingBlockDevice::with_shared_stream(id, d, stream.clone()), hidden: hidden_ranges_of(&recorded.writes, &persisted, id) }))
        .collect();
    let output = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(mounted) => MountedSession::of_the_mount(mounted).0,
        Err(error) => return println!("H5 W={code} k={k} arm={} MOUNT_ERR {}", arm(), short(&Err(error))),
    };
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &widths(code).fixed_geometry());
    let judged = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("写行那次发布有根槽写");
    let count = closed_form_state_count(&segments);
    let effective = (output.effective_root.instance, output.effective_root.checkpoint_txg);
    let mut versions = recorded.pool.published_versions();
    let content_now = recorded.pool.content_of_each_root.get(&effective).cloned().unwrap_or_default();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        versions.push(PublishedVersion { instance: root.instance, checkpoint_txg: root.checkpoint_txg, content: content_now.clone() });
    }
    println!(
        "H5 W={code} k={k} arm={} m1_writes={} m1_segments={} segment_sizes={:?} closed_form={count} row_txg={} warm_up={}",
        arm(), writes.len(), segments.len(), segments.iter().map(Vec::len).collect::<Vec<_>>(),
        output.row_publish.root().checkpoint_txg.0, output.warm_up_publishes.len()
    );
    let expanded: u64 = 1 + segments.iter().enumerate().map(|(i, s)| if expand_segment(i, s) { (1u64 << s.len()) - 1 } else { 0 }).sum::<u64>();
    println!("H5 W={code} k={k} arm={} states_with_this_expansion={expanded} full={}", arm(), std::env::var("OPUS_FULL").unwrap_or_default());
    if count > 1_000_000 && std::env::var("OPUS_FULL").is_ok_and(|v| v == "1") {
        return println!("H5 W={code} k={k} arm={} SKIPPED closed_form={count} > 1e6", arm());
    }
    let r_key = (recorded.recorded.0 .0, recorded.recorded.1 .0);
    let r_content = vec![RECORDED_CONTENT_BYTE; OVERWRITE_BYTES];
    let mut classes: BTreeMap<String, u64> = BTreeMap::new();
    let label = format!("m1 W={code} k={k} arm={}", arm());
    let tally = singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state(
        &crash_image, &writes, &segments, judged, &versions, &|segment_index, segment| expand_segment(segment_index, segment),
        &mut |image, report| {
            let landed = report.effective_root.map(|(i, t)| (i.0, t.0));
            let row_root_persisted = image.persisted[judged];
            let class = match (&report.outcome, landed == Some(r_key)) {
                (RecoveryOutcome::FileRead { content, .. }, true) if *content == r_content => "onR:read-ok".to_string(),
                (RecoveryOutcome::FileRead { .. }, true) => "onR:WRONG-CONTENT".to_string(),
                (RecoveryOutcome::Failed { failure, .. }, true) => format!("onR:FAILED({})", format!("{failure:?}").split([' ', '{']).next().unwrap_or("")),
                (RecoveryOutcome::Failed { failure, .. }, false) => format!("landed={landed:?}:FAILED({})", format!("{failure:?}").split([' ', '{']).next().unwrap_or("")),
                (_, false) => format!("landed={landed:?}:ok"),
                (RecoveryOutcome::NoFile { .. }, true) => "onR:NOFILE".to_string(),
            };
            *classes.entry(format!("row_root_persisted={row_root_persisted} {class}")).or_default() += 1;
        },
    );
    println!("{}", tally_line(&label, &tally));
    for (class, states) in classes {
        println!("H5CLASS W={code} k={k} arm={} {class} states={states}", arm());
    }
}

fn open_short(reader: &dyn PoolReader, root: &RootRecord) -> String {
    match singlefs_core::mounted_read::open_pool_for_read(reader, root) {
        Ok(mounted) => format!("Ok(inodes={})", mounted.inode_records().len()),
        Err(error) => format!("Err({})", format!("{error:?}").chars().take(70).collect::<String>()),
    }
}

/// K3：第一个文件之后（覆盖写 0 次）每条根在两块盘都在时直接打开；丢一块盘之后只读挂载、恢复与池级 checker；
/// 丢 0 号盘那一份上，所选那一版之后的记录点名的单元在幸存盘上那一份读不读得出。
fn subcommand_c695() {
    for code in ["384", "256", "240", "4g"] {
        let pool = start_plain(widths(code));
        let image = pool.image();
        let configuration = choose_system_configuration(&image).expect("系统配置");
        for ring_slot in every_root_ring_slot(&configuration.immutable.sizes) {
            if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(&image, &configuration.immutable.region_devices,
                &configuration.immutable.sizes, &configuration.immutable.filesystem_identifier, ring_slot) {
                println!(
                    "K3 W={code} arm={} both root=({},{}) mapping_root_empty={} alloc_tree_empty={} open={}",
                    std::env::var("OPUS_C695_ARM").unwrap_or_default(),
                    root.instance.0, root.checkpoint_txg.0,
                    root.mapping_root == singlefs_core::pointer::NodePointer::empty_root(),
                    root.allocation_record_tree_root == singlefs_core::pointer::NodePointer::empty_root(),
                    open_short(&image, &root)
                );
            }
        }
        for lost in [0_u32, 1] {
            let mut lone = image.clone();
            lone.devices.remove(&DeviceIdentity(lost));
            let read_only = match singlefs_core::mounted_read::mount_read_only(&lone) {
                Ok(mounted) => format!("Ok(inodes={})", mounted.mounted.inode_records().len()),
                Err(error) => format!("Err({})", format!("{error:?}").chars().take(70).collect::<String>()),
            };
            let report = recover(&lone, JournalPolicy::Consult);
            let outcome = match &report.outcome {
                RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
                RecoveryOutcome::FileRead { content, .. } => format!("FileRead({} bytes)", content.len()),
                RecoveryOutcome::Failed { failure, .. } => format!("Failed({})", format!("{failure:?}").chars().take(200).collect::<String>()),
            };
            let mut unapplied = Vec::new();
            if let Ok(lone_configuration) = choose_system_configuration(&lone) {
                let records = singlefs_core::recovery::scan_journal(&lone, &lone_configuration);
                let effective_txg = report.effective_root.map_or(0, |(_, t)| t.0);
                for record in records.values().filter(|r| r.checkpoint_txg.0 > effective_txg) {
                    let surviving = DeviceIdentity(1 - lost);
                    let mut readable_on_the_survivor = 0;
                    for named in &record.named {
                        let bytes = if named.unit_class == singlefs_core::unit::UNIT_CLASS_INDEX_NODE { 16384 } else { 32768 };
                        if named.locations.iter().any(|location| location.device == surviving
                            && PoolReader::read(&lone, surviving, location.slot.to_device_offset(), bytes)
                                .is_some_and(|b| singlefs_core::checksum::crc32_castagnoli(&b) == location.unit_checksum)) {
                            readable_on_the_survivor += 1;
                        }
                    }
                    unapplied.push(format!("({},{}):named={}:survivor_copy_ok={}", record.instance.0, record.checkpoint_txg.0, record.named.len(), readable_on_the_survivor));
                }
            }
            println!(
                "K3 W={code} arm={} lost=dev{lost} effective={:?} read_only={read_only} recover={outcome} unapplied_records_above_effective=[{}] checker=[{}]",
                std::env::var("OPUS_C695_ARM").unwrap_or_default(),
                report.effective_root.map(|(i, t)| (i.0, t.0)), unapplied.join(" "), checker_names(&lone)
            );
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let number = |i: usize| -> u64 { args.get(i).and_then(|v| v.parse().ok()).expect("参数是十进制数") };
    match args.get(1).map(String::as_str) {
        Some("counts") => subcommand_counts(),
        Some("grid") => subcommand_grid(&args[2], number(3), number(4)),
        Some("transient") => subcommand_transient(&args[2], number(3), number(4)),
        Some("crashmount") => subcommand_crashmount(&args[2], number(3)),
        Some("c695") => subcommand_c695(),
        Some("l0only") => {
            let recorded = record_one_overwrite(widths(&args[2]), number(3)).expect("前缀");
            let versions = recorded.pool.published_versions();
            let states = every_crash_state(&recorded.base, &recorded.writes, &recorded.segments, recorded.root_index, &versions,
                &format!("recorded W={} k={} arm={} (l0only)", args[2], number(3), arm()));
            let root_landed = states.iter().filter(|p| p[recorded.root_index]).count();
            println!("L0ONLY W={} k={} states={} root_landed_states={root_landed}", args[2], number(3), states.len());
        }
        _ => panic!("子命令：counts / grid / transient / crashmount / c695"),
    }
}
