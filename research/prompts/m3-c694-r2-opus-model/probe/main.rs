//! m3-c694-r2 攻方原型（Opus）。臂由环境变量选（见模型目录 make_patched.py 文件头与末段）：
//!   OPUS_C694_ARM = today / C / B / W（可用 + 连），OPUS_C_SELECT = all / last，OPUS_C_RELEASE = mount / ring / d16，
//!   OPUS_RECOVERY_FALLBACK = 空 / strict / loose。
//! 起点（前缀）：第一个文件之后在会话里覆盖写 k 次，再在盘面拷贝上录一次发布 R（rkind：ow = 覆盖写；swN = 顺序写 N 个数据单元），
//! 崩在 R 的根槽落盘那一刀（R 的单元、记录、根槽都持久，系统配置轮换没持久）。取新实例那次挂载 M1 时按 hide 把 R 的几样读回全 0：
//!   u = R 的单元写，r = R 的根槽，J = R 的全部记录，L = 末条记录，M = 中间一条记录，F = 第一条记录（可连写，如 ur、urL）。
//! 子命令：
//!   cm <宽> <k> <rkind> <hide>                 M1 的写流层 0 全量枚举（每一段都展开），每个崩溃状态撤故障恢复，按落到哪条根、读不读得出分类
//!   grid <宽> <k> <rkind> <hide> <L> <字母表>   M1 之后用户动作放开扫：字母表里每个字母一种动作（o 覆盖写、n 建一个 inode、s 顺序写 2 单元、
//!                                              u 删第一个文件），长度 1..=L 的每一串都走；每一步在拷贝上卸载、卸载之后的盘面跑池级 checker、
//!                                              R 还在环里就做 R 的可读性探针（新实例的根全抹掉再恢复）
//!   bcheck <宽> <k> <rkind> <hide> <L> <字母表> 同 grid 的每一步，卸载那一刻 R 的树表、映射根、分配记录树根读报错（瞬时），看 B / W 跳没跳、
//!                                              跳的那一刻 R 在无故障的盘面上账读不读得出（读得出 = 把瞬时读不出判成已被占）
//!   silent <宽> <k>                            恢复一侧补法会不会把本该报读不出的盘面悄悄读回旧版（三形，见函数注释）
//!   counts <宽> <k> <rkind> <hide>             只报 M1 写流的段与闭式状态数
#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use std::collections::{BTreeMap, BTreeSet};

use common_admission::{
    checker_violations_on, content_of, plain_devices_on, start_plain, PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::{mount_writable_with_space_admission, unmount, MountError, Unmounted};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover,
    JournalPolicy, PoolReader, RecoveryOutcome, RootRingSlotReading,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_overwrite, publish_sequential_write, FirstFile, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const RECORDED_TIME: u64 = common_admission::FIRST_WRITE_TIME_SECONDS + 100_000;

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).ok().filter(|v| !v.is_empty()).unwrap_or_else(|| default.to_string())
}

/// 臂的全名：C694 臂 / C 选哪些记录 / C 何时解开 / 恢复补法。
fn arm() -> String {
    format!(
        "{}/{}/{}/{}",
        env_or("OPUS_C694_ARM", "today"),
        env_or("OPUS_C_SELECT", "all"),
        env_or("OPUS_C_RELEASE", "mount"),
        env_or("OPUS_RECOVERY_FALLBACK", "none")
    )
}

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

/// R 的内容：ow 是 0x5a 填满的覆盖写，swN 是 N 个数据单元整的顺序写。
fn r_content(rkind: &str) -> Vec<u8> {
    if rkind == "ow" {
        vec![0x5a; OVERWRITE_BYTES]
    } else {
        let units: usize = rkind.strip_prefix("sw").and_then(|n| n.parse().ok()).expect("rkind 只认 ow / swN");
        content_of(units * singlefs_core::unit::data_unit_payload_capacity(), 0x77)
    }
}

/// 起点：第一个文件之后覆盖写 k 次（会话留着前缀那一版），在盘面拷贝上录一次发布 R。
struct Recorded {
    pool: PoolUnderTest<SparseBlockDevice>,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    root_index: usize,
    recorded: (InstanceGeneration, CheckpointTxg),
    content: Vec<u8>,
}

fn record_r(width: HistoryDeviceWidth, k: u64, rkind: &str) -> Result<Recorded, String> {
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
    let content = r_content(rkind);
    let recorded = {
        let session = pool.session.as_ref().expect("会话");
        let mut allocator = session.allocator.clone();
        let previous = session.current.file_version().expect("带文件").clone();
        let mut writer = PoolWriter::new(&pool.parameters, devices.as_mut_slice());
        let file = FirstFile { content: &content, write_time_seconds: RECORDED_TIME };
        let published = if rkind == "ow" {
            publish_overwrite(&mut writer, &mut allocator, &previous, file, session.instance)
        } else {
            publish_sequential_write(&mut writer, &mut allocator, &previous, file, session.instance)
        };
        let root = published.map_err(|e| format!("recorded-{rkind}:{e:?}"))?.root;
        (root.instance, root.checkpoint_txg)
    };
    pool.content_of_each_root.insert(recorded, content.clone());
    let (writes, _segments, _) =
        writes_and_segments_with_stream_indexes(&stream.retained_operations(), &width.fixed_geometry());
    let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).ok_or("没有根槽写")?;
    Ok(Recorded { pool, base, writes, root_index, recorded, content })
}

/// R 的根槽持久、之后（系统配置轮换）一个没持久的那个崩溃状态的持久集合。
fn crash_after_r_root(recorded: &Recorded) -> Vec<bool> {
    (0..recorded.writes.len()).map(|i| i <= recorded.root_index).collect()
}

/// R 的记录按偏移分条（写的次序）。
fn record_groups(writes: &[RetainedWrite]) -> Vec<u64> {
    let mut offsets = Vec::new();
    for w in writes.iter().filter(|w| w.kind == StepKind::JournalRecord) {
        if !offsets.contains(&w.offset.0) {
            offsets.push(w.offset.0);
        }
    }
    offsets
}

/// 按 hide 记号挑 R 的写在 `device` 上的区间（这个持久集合里落了盘的）。
fn hidden_ranges_of(writes: &[RetainedWrite], persisted: &[bool], device: DeviceIdentity, hide: &str) -> Vec<(u64, u64)> {
    let groups = record_groups(writes);
    let mut hidden_records: BTreeSet<u64> = BTreeSet::new();
    if !groups.is_empty() {
        if hide.contains('J') {
            hidden_records.extend(groups.iter().copied());
        }
        if hide.contains('L') {
            hidden_records.insert(groups[groups.len() - 1]);
        }
        if hide.contains('F') {
            hidden_records.insert(groups[0]);
        }
        if hide.contains('M') {
            hidden_records.insert(groups[groups.len() / 2]);
        }
    }
    writes
        .iter()
        .zip(persisted)
        .filter(|(w, p)| {
            **p && w.device == device
                && ((w.kind == StepKind::UnitWrite && hide.contains('u'))
                    || (w.kind == StepKind::RootRecordFua && hide.contains('r'))
                    || (w.kind == StepKind::JournalRecord && hidden_records.contains(&w.offset.0)))
        })
        .map(|(w, _)| (w.offset.0, w.length_in_bytes()))
        .collect()
}

fn image_of<D>(devices: &[(DeviceIdentity, D)], device_bytes: u64, sparse: impl Fn(&D) -> &SparseBlockDevice) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(id, d)| (*id, sparse(d).image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    }
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

fn isolated_total(pool: &PoolUnderTest<SparseBlockDevice>) -> u64 {
    pool.session().allocator.devices.iter().map(|d| d.isolated_slots()).sum()
}

fn free_total(pool: &PoolUnderTest<SparseBlockDevice>) -> u64 {
    pool.session().allocator.devices.iter().map(|d| d.unblocked_slots()).sum()
}

/// 取新实例那次挂载（崩溃状态的镜像上，按 hide 把 R 的几样读回全 0；只在这次挂载调用期间藏）做成之后的池。
fn abandoning_mount(recorded: &Recorded, persisted: &[bool], hide: &str) -> Result<(PoolUnderTest<SparseBlockDevice>, String), String> {
    let crash_image = materialize(&recorded.base, &recorded.writes, persisted);
    let mut devices: Vec<(DeviceIdentity, Hiding<SparseBlockDevice>)> = plain_devices_on(&crash_image)
        .into_iter()
        .map(|(id, d)| (id, Hiding { inner: d, hidden: hidden_ranges_of(&recorded.writes, persisted, id, hide) }))
        .collect();
    let mounted = mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
        .map_err(|e| format!("M1:{}", short_error(&e)))?;
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
        "effective=({},{}) row_txg={} isolated={} free={}",
        effective.0 .0,
        effective.1 .0,
        output.row_publish.root().checkpoint_txg.0,
        isolated_total(&pool),
        free_total(&pool)
    );
    Ok((pool, summary))
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

/// 一条根的账里仍分配着的落点（读不出交 None）。
fn placements_of(image: &MemoryPool, root: &RootRecord) -> Option<Vec<(DeviceIdentity, u64, u64)>> {
    allocation_records_under_root(image, root)
        .ok()
        .map(|records| records.into_iter().filter(|r| !r.is_released).map(|r| (r.device, r.slot.0, u64::from(r.span_slots))).collect())
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
    let mut devices = plain_devices_on(image);
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
    let copy = image_of(&devices, image.device_size_in_bytes, |d| d);
    let report = recover(&copy, JournalPolicy::Consult);
    let landed = report.effective_root.map(|(i, t)| (i.0, t.0));
    match report.outcome {
        RecoveryOutcome::FileRead { content, .. } if landed == Some((abandoned.0 .0, abandoned.1 .0)) => {
            if content == expected { "onR:read-ok".to_string() } else { "onR:WRONG-CONTENT".to_string() }
        }
        RecoveryOutcome::Failed { failure, .. } if landed == Some((abandoned.0 .0, abandoned.1 .0)) || landed.is_none() => {
            format!("onR:FAILED({})", kind_of(&format!("{failure:?}")))
        }
        _ => format!("not-on-R:{landed:?}"),
    }
}

fn kind_of(text: &str) -> String {
    text.split([' ', '(', '{']).next().unwrap_or("").to_string()
}

fn short_error(error: &MountError) -> String {
    let text = format!("{error:?}");
    match error {
        MountError::NewerStateStillUnreadableAfterOneReread(still) => format!("StillUnreadable:{}", kind_of(&format!("{still:?}"))),
        _ => kind_of(&text),
    }
}

fn short(result: &Result<Unmounted, MountError>) -> String {
    match result {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!("Ok({})", raised.publishes.len()),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "Ok(nothing)".to_string(),
        Err(error) => format!("Err({})", short_error(error)),
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

fn clone_pool(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: plain_devices_on(&pool.image()),
        session: Some(MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

/// 用户动作：o 覆盖写、n 建一个 inode、s 顺序写 2 单元、u 删第一个文件。
fn apply_op<Dev: common_admission::DeviceOverASparseImage + BlockDevice>(pool: &mut PoolUnderTest<Dev>, op: char) -> Result<(), String> {
    let outcome = match op {
        'o' => pool.overwrite(OVERWRITE_BYTES),
        'n' => pool.create_one_inode(),
        's' => pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity()),
        'u' => {
            pool.write_time_seconds += 1;
            let write_time_seconds = pool.write_time_seconds;
            pool.publish(UserChange::Unlink { inode: InodeNumber(FIRST_INODE_NUMBER), write_time_seconds }, None)
        }
        other => panic!("动作只认 o / n / s / u：{other}"),
    };
    outcome.map(|_| ()).map_err(|e| kind_of(&format!("{e:?}")))
}

/// R 的账那几个单元（树表、中央映射根、分配记录树根）在两块盘上的区间。
fn account_ranges_of(root: &RootRecord) -> Vec<(DeviceIdentity, u64, u64)> {
    let mut ranges = Vec::new();
    for pointer in [&root.tree_table, &root.mapping_root, &root.allocation_record_tree_root] {
        for location in pointer.locations.iter().filter(|location| location.slot.0 != 0) {
            ranges.push((location.device, location.slot.0 * SLOT_BYTES, SLOT_BYTES));
        }
    }
    ranges
}

/// 一个结点：在拷贝上卸载、卸载后的盘面跑 checker、R 还在环里就探 R。
fn judge_node(pool: &PoolUnderTest<SparseBlockDevice>, recorded: &Recorded, crash_image: &MemoryPool, r_placements: &[(DeviceIdentity, u64, u64)]) -> String {
    let image = pool.image();
    let in_ring = ring_root(&image, recorded.recorded).is_some();
    let damaged = changed_placements(crash_image, &image, r_placements);
    let (unmounted, after) = unmount_on_a_copy(pool, |_, d| d, |d| d);
    let in_ring_after = unmounted.is_ok() && ring_root(&after, recorded.recorded).is_some();
    let probe = if in_ring_after { probe_recovery_onto(&after, recorded.recorded, &recorded.content) } else { "-".to_string() };
    let checker = if unmounted.is_ok() { checker_names(&after) } else { checker_names(&image) };
    format!(
        "in_ring={in_ring} dmg={damaged}/{} iso={} free={} unmount={} in_ring_after_unmount={in_ring_after} chk=[{checker}] probe={probe}",
        r_placements.len(),
        isolated_total(pool),
        free_total(pool),
        short(&unmounted)
    )
}

/// 长度 1..=depth 的每一串动作都走（树形，前缀共用）；每个结点交 judge 一行。
fn walk_sequences(
    pool: &PoolUnderTest<SparseBlockDevice>,
    alphabet: &[char],
    depth: usize,
    prefix: String,
    judge: &(dyn Fn(&PoolUnderTest<SparseBlockDevice>) -> String + Sync),
    lines: &mut Vec<String>,
) {
    lines.push(format!("seq={} {}", if prefix.is_empty() { "-" } else { &prefix }, judge(pool)));
    if depth == 0 {
        return;
    }
    for &op in alphabet {
        let mut child = clone_pool(pool);
        let sequence = format!("{prefix}{op}");
        match apply_op(&mut child, op) {
            Ok(()) => walk_sequences(&child, alphabet, depth - 1, sequence, judge, lines),
            Err(kind) => lines.push(format!("seq={sequence} OP_REFUSED {kind}")),
        }
    }
}

fn subcommand_grid(code: &str, k: u64, rkind: &str, hide: &str, depth: usize, alphabet: &str, with_bcheck: bool) {
    let head = format!("W={code} k={k} R={rkind} hide={hide} arm={}", arm());
    let recorded = match record_r(widths(code), k, rkind) {
        Ok(recorded) => recorded,
        Err(error) => return println!("G {head} PREFIX_FAILED {error}"),
    };
    let persisted = crash_after_r_root(&recorded);
    let crash_image = materialize(&recorded.base, &recorded.writes, &persisted);
    let Some(r_root) = ring_root(&crash_image, recorded.recorded) else { return println!("G {head} R_NOT_IN_RING") };
    let r_placements = placements_of(&crash_image, &r_root).unwrap_or_default();
    let (pool, summary) = match abandoning_mount(&recorded, &persisted, hide) {
        Ok(after) => after,
        Err(error) => return println!("G {head} {error}"),
    };
    println!("G {head} M1[{summary}] r=({},{}) r_records={}", recorded.recorded.0 .0, recorded.recorded.1 .0, record_groups(&recorded.writes).len());
    let alphabet: Vec<char> = alphabet.chars().collect();
    let judge = |p: &PoolUnderTest<SparseBlockDevice>| {
        let plain = judge_node(p, &recorded, &crash_image, &r_placements);
        if with_bcheck { format!("{plain} {}", bcheck_node(p, &recorded, &r_root, &crash_image, &r_placements)) } else { plain }
    };
    // 第一层分给线程：每个线程走一个首字母的子树。
    let mut lines = vec![format!("seq=- {}", judge(&pool))];
    let subtrees = parallel_map(&alphabet, |_, &op| {
        let mut child = clone_pool(&pool);
        let mut sub = Vec::new();
        match apply_op(&mut child, op) {
            Ok(()) => walk_sequences(&child, &alphabet, depth.saturating_sub(1), op.to_string(), &judge, &mut sub),
            Err(kind) => sub.push(format!("seq={op} OP_REFUSED {kind}")),
        }
        sub
    });
    lines.extend(subtrees.into_iter().flatten());
    let mut tally: BTreeMap<&str, u64> = BTreeMap::new();
    for line in &lines {
        println!("G {head} {line}");
        *tally.entry("nodes").or_default() += 1;
        for (name, needle) in [
            ("chk_I-7.4", "I-7.4"),
            ("probe_failed", "probe=onR:FAILED"),
            ("probe_wrong", "probe=onR:WRONG"),
            ("unmount_refused", "unmount=Err"),
            ("op_refused", "OP_REFUSED"),
            ("b_misjudge", "B_MISJUDGE"),
            ("b_skip", "skips=1"),
        ] {
            if line.contains(needle) {
                *tally.entry(name).or_default() += 1;
            }
        }
    }
    println!("GTALLY {head} {}", tally.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "));
}

/// 点名的几段读回给定的旧字节（「这一槽读回了写之前的内容」：丢了的写、读错成旧内容），被写到的那一段不再换。
struct Replacing<D: BlockDevice> {
    inner: D,
    replaced: Vec<(u64, Vec<u8>)>,
}

impl<D: BlockDevice> BlockDevice for Replacing<D> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let length = buffer.len() as u64;
        for (start, bytes) in &self.replaced {
            let len = bytes.len() as u64;
            if overlaps(*start, len, offset.0, length) {
                for i in 0..length {
                    let at = offset.0 + i;
                    if at >= *start && at < start + len {
                        buffer[i as usize] = bytes[(at - start) as usize];
                    }
                }
            }
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let length = bytes.len() as u64;
        self.replaced.retain(|(start, b)| !overlaps(*start, b.len() as u64, offset.0, length));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.replaced.retain(|(start, b)| !overlaps(*start, b.len() as u64, offset.0, length));
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

/// 卸载之后再可写挂载（这次挂载里 R 的账那几个单元瞬时读坏：fail = 读报错，stale = 读回 R 写它之前那一槽的旧内容），
/// 看 B / W 跳没跳；挂成就撤故障再覆盖写 3 次、卸载、探 R。跳了而 R 在无故障的盘面上账读得出，记 B_MISJUDGE。
fn bcheck_node(
    pool: &PoolUnderTest<SparseBlockDevice>,
    recorded: &Recorded,
    r_root: &RootRecord,
    _crash_image: &MemoryPool,
    _r_placements: &[(DeviceIdentity, u64, u64)],
) -> String {
    let (unmounted, image) = unmount_on_a_copy(pool, |_, d| d, |d| d);
    if unmounted.is_err() {
        return "bc=unmount-refused".to_string();
    }
    if ring_root(&image, recorded.recorded).is_none() {
        return "bc=R-gone".to_string();
    }
    let alive = placements_of(&image, r_root).is_some();
    let ranges = account_ranges_of(r_root);
    let mut parts = vec![format!("bc_alive={alive}")];
    for fault in ["fail", "stale"] {
        let before = singlefs_core::mount::OPUS_SKIPS.with(|c| c.get());
        let mounted_image;
        let mounted = if fault == "fail" {
            let mut devices: Vec<(DeviceIdentity, Failing)> = plain_devices_on(&image)
                .into_iter()
                .map(|(id, d)| (id, Failing { inner: d, failing: ranges.iter().filter(|r| r.0 == id).map(|r| (r.1, r.2)).collect() }))
                .collect();
            let m = mount_writable_with_space_admission(&pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula);
            mounted_image = image_of(&devices, pool.device_bytes, |d| &d.inner);
            m
        } else {
            let mut devices: Vec<(DeviceIdentity, Replacing<SparseBlockDevice>)> = plain_devices_on(&image)
                .into_iter()
                .map(|(id, d)| {
                    let replaced = ranges
                        .iter()
                        .filter(|r| r.0 == id)
                        .map(|r| (r.1, PoolReader::read(&recorded.base, id, DeviceOffsetInBytes(r.1), r.2 as usize).unwrap_or_else(|| vec![0; r.2 as usize])))
                        .collect();
                    (id, Replacing { inner: d, replaced })
                })
                .collect();
            let m = mount_writable_with_space_admission(&pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula);
            mounted_image = image_of(&devices, pool.device_bytes, |d| &d.inner);
            m
        };
        let skips = singlefs_core::mount::OPUS_SKIPS.with(|c| c.get()) - before;
        let tail = match mounted {
            Err(error) => format!("mount=Err({})", short_error(&error)),
            Ok(mounted) => {
                let (_output, session) = MountedSession::of_the_mount(mounted);
                let mut after = pool_from(recorded, &mounted_image, session, Vec::new());
                let mut steps = Vec::new();
                for _ in 0..3 {
                    steps.push(match apply_op(&mut after, 'o') { Ok(()) => "ok".to_string(), Err(kind) => kind });
                }
                let (u, final_image) = unmount_on_a_copy(&after, |_, d| d, |d| d);
                let probe = if u.is_ok() && ring_root(&final_image, recorded.recorded).is_some() {
                    probe_recovery_onto(&final_image, recorded.recorded, &recorded.content)
                } else {
                    "-".to_string()
                };
                format!("mount=Ok then=[{}] unmount={} probe={probe} chk=[{}]", steps.join(","), short(&u),
                    if u.is_ok() { checker_names(&final_image) } else { String::new() })
            }
        };
        let misjudge = if skips > 0 && alive { " B_MISJUDGE" } else { "" };
        parts.push(format!("bc_{fault}:skips={skips}{misjudge} {tail}"));
    }
    parts.join(" | ")
}

/// M1 的写流（藏着 hide 那几样做取新实例那次挂载）录下来，交回写、段、被判的根槽写、版本表与写行那次的 txg。
struct M1Stream {
    crash_image: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged: usize,
    versions: Vec<PublishedVersion>,
    row_txg: u64,
}

fn record_m1(code: &str, recorded: &Recorded, hide: &str) -> Result<M1Stream, String> {
    let persisted = crash_after_r_root(recorded);
    let crash_image = materialize(&recorded.base, &recorded.writes, &persisted);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, Hiding<RecordingBlockDevice<SparseBlockDevice>>)> = plain_devices_on(&crash_image)
        .into_iter()
        .map(|(id, d)| (id, Hiding { inner: RecordingBlockDevice::with_shared_stream(id, d, stream.clone()), hidden: hidden_ranges_of(&recorded.writes, &persisted, id, hide) }))
        .collect();
    let output = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(mounted) => MountedSession::of_the_mount(mounted).0,
        Err(error) => return Err(format!("MOUNT_ERR {}", short_error(&error))),
    };
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &widths(code).fixed_geometry());
    let judged = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).ok_or("M1 没有根槽写")?;
    let effective = (output.effective_root.instance, output.effective_root.checkpoint_txg);
    let mut versions = recorded.pool.published_versions();
    let content_now = recorded.pool.content_of_each_root.get(&effective).cloned().unwrap_or_default();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        versions.push(PublishedVersion { instance: root.instance, checkpoint_txg: root.checkpoint_txg, content: content_now.clone() });
    }
    Ok(M1Stream { crash_image, writes, segments, judged, versions, row_txg: output.row_publish.root().checkpoint_txg.0 })
}

fn subcommand_counts(code: &str, k: u64, rkind: &str, hide: &str) {
    let recorded = record_r(widths(code), k, rkind).expect("前缀");
    match record_m1(code, &recorded, hide) {
        Ok(m1) => println!(
            "COUNT W={code} k={k} R={rkind} hide={hide} arm={} r_records={} m1_writes={} segment_sizes={:?} closed_form={}",
            arm(), record_groups(&recorded.writes).len(), m1.writes.len(), m1.segments.iter().map(Vec::len).collect::<Vec<_>>(),
            closed_form_state_count(&m1.segments)
        ),
        Err(error) => println!("COUNT W={code} k={k} R={rkind} hide={hide} arm={} {error}", arm()),
    }
}

/// M1 的写流层 0 全量枚举（每一段都展开），每个状态撤故障恢复，按落到哪条根、读不读得出分类。
fn subcommand_cm(code: &str, k: u64, rkind: &str, hide: &str) {
    let head = format!("W={code} k={k} R={rkind} hide={hide} arm={}", arm());
    let recorded = match record_r(widths(code), k, rkind) {
        Ok(recorded) => recorded,
        Err(error) => return println!("CM {head} PREFIX_FAILED {error}"),
    };
    let m1 = match record_m1(code, &recorded, hide) {
        Ok(m1) => m1,
        Err(error) => return println!("CM {head} {error}"),
    };
    let count = closed_form_state_count(&m1.segments);
    println!("CM {head} r=({},{}) row_txg={} m1_writes={} segment_sizes={:?} closed_form={count}",
        recorded.recorded.0 .0, recorded.recorded.1 .0, m1.row_txg, m1.writes.len(), m1.segments.iter().map(Vec::len).collect::<Vec<_>>());
    if count > 1_000_000 {
        return println!("CM {head} SKIPPED closed_form={count} > 1e6");
    }
    let r_key = (recorded.recorded.0 .0, recorded.recorded.1 .0);
    let mut classes: BTreeMap<String, u64> = BTreeMap::new();
    let mut first_image_of_class: BTreeMap<String, MemoryPool> = BTreeMap::new();
    let tally = singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state(
        &m1.crash_image, &m1.writes, &m1.segments, m1.judged, &m1.versions, &|_, _| true,
        &mut |image, report| {
            let class_key = {
                let landed = report.effective_root.map(|(i, t)| (i.0, t.0));
                format!("row_root_persisted={} landed={landed:?} failed={}", image.persisted[m1.judged], matches!(report.outcome, RecoveryOutcome::Failed { .. }))
            };
            first_image_of_class.entry(class_key).or_insert_with(|| materialize(image.base, image.writes, &image.persisted));
            let landed = report.effective_root.map(|(i, t)| (i.0, t.0));
            let row_root_persisted = image.persisted[m1.judged];
            let on = if landed == Some(r_key) { "onR".to_string() } else { format!("landed={landed:?}") };
            let class = match &report.outcome {
                RecoveryOutcome::FileRead { content, .. } if landed == Some(r_key) && *content != recorded.content => "WRONG-CONTENT".to_string(),
                RecoveryOutcome::FileRead { .. } => "read-ok".to_string(),
                RecoveryOutcome::NoFile { .. } => "NOFILE".to_string(),
                RecoveryOutcome::Failed { failure, .. } => format!("FAILED({})", kind_of(&format!("{failure:?}"))),
            };
            *classes.entry(format!("row_root_persisted={row_root_persisted} {on}:{class}")).or_default() += 1;
        },
    );
    let checker: Vec<String> = tally.checker_violated_states.iter().filter(|(_, n)| **n > 0).map(|(name, n)| format!("{name}={n}")).collect();
    println!(
        "L0TALLY CM {head} states={} root_persisted={} violations={} failed={} file_read={} checker_violated=[{}]",
        tally.states, tally.root_persisted_states, tally.violations, tally.failed_states, tally.file_read_states, checker.join(",")
    );
    for (class, states) in classes {
        println!("CMCLASS {head} {class} states={states}");
    }
    // 每一类的第一个状态上，撤故障之后真走一遍两条挂载路径：可写挂载（之后卸载）与只读挂载。
    for (class, image) in first_image_of_class {
        let mut devices = plain_devices_on(&image);
        let writable = match mount_writable_with_space_admission(&recorded.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
            Ok(mounted) => {
                let effective = (mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0);
                format!("Ok(effective={effective:?})")
            }
            Err(error) => format!("Err({})", short_error(&error)),
        };
        let read_only = match singlefs_core::mounted_read::mount_read_only(&image) {
            Ok(mounted) => format!("Ok(inodes={})", mounted.mounted.inode_records().len()),
            Err(error) => format!("Err({})", kind_of(&format!("{error:?}"))),
        };
        println!("CMMOUNT {head} first_state_of[{class}] writable={writable} read_only={read_only}");
    }
}

fn zeroed(image: &MemoryPool, ranges: &[(DeviceIdentity, u64, u64)]) -> MemoryPool {
    let mut devices = plain_devices_on(image);
    for (device, offset, length) in ranges {
        for (id, d) in devices.iter_mut() {
            if id == device {
                d.write_zeroes_at(DeviceOffsetInBytes(*offset), *length).expect("写零");
            }
        }
    }
    image_of(&devices, image.device_size_in_bytes, |d| d)
}

/// 每块盘两个系统配置槽里世代号大的那一槽的区间。
fn newest_system_configuration_slots(image: &MemoryPool) -> Vec<(DeviceIdentity, u64, u64)> {
    let configuration = choose_system_configuration(image).expect("系统配置");
    let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let mut ranges = Vec::new();
    for device in PoolReader::device_identities(image) {
        let newest = [0, spacing]
            .into_iter()
            .filter_map(|offset| {
                let bytes = PoolReader::read(image, device, DeviceOffsetInBytes(offset), 4096)?;
                let parsed = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes).ok()?;
                Some((parsed.quantities.slot_generation, offset))
            })
            .max();
        if let Some((_, offset)) = newest {
            ranges.push((device, offset, 4096));
        }
    }
    ranges
}

/// 恢复一侧补法会不会把本该报读不出的盘面悄悄读回旧版。V = 起点之后经写入口发的一次覆盖写（盘面拷贝上录的那一次）。
///   S1：V 整次发布落盘（系统配置已轮换，V 已确认）；恢复时 V 的单元与每块盘世代号大的那一个系统配置槽读回全 0。
///   S1c：对照，只藏 V 的单元。
///   S2：V 已确认；恢复时 V 的单元与 V 的记录读回全 0。
///   S3：崩在 V 的根槽落盘那一刀（没轮换，V 没确认）；恢复时 V 的单元读回全 0。
fn subcommand_silent(code: &str, k: u64) {
    let recorded = record_r(widths(code), k, "ow").expect("前缀");
    let previous_content = recorded.pool.content_of_the_current_version.clone();
    let full: Vec<bool> = vec![true; recorded.writes.len()];
    let window = crash_after_r_root(&recorded);
    let ranges_of = |persisted: &[bool], kinds: &[StepKind]| -> Vec<(DeviceIdentity, u64, u64)> {
        recorded.writes.iter().zip(persisted).filter(|(w, p)| **p && kinds.contains(&w.kind)).map(|(w, _)| (w.device, w.offset.0, w.length_in_bytes())).collect()
    };
    let confirmed = materialize(&recorded.base, &recorded.writes, &full);
    let in_window = materialize(&recorded.base, &recorded.writes, &window);
    let mut shapes: Vec<(&str, MemoryPool)> = Vec::new();
    let mut s1 = ranges_of(&full, &[StepKind::UnitWrite]);
    s1.extend(newest_system_configuration_slots(&confirmed));
    shapes.push(("S1", zeroed(&confirmed, &s1)));
    shapes.push(("S1c", zeroed(&confirmed, &ranges_of(&full, &[StepKind::UnitWrite]))));
    shapes.push(("S2", zeroed(&confirmed, &ranges_of(&full, &[StepKind::UnitWrite, StepKind::JournalRecord]))));
    shapes.push(("S3", zeroed(&in_window, &ranges_of(&window, &[StepKind::UnitWrite]))));
    let v_key = (recorded.recorded.0 .0, recorded.recorded.1 .0);
    for (name, image) in shapes {
        let report = recover(&image, JournalPolicy::Consult);
        let landed = report.effective_root.map(|(i, t)| (i.0, t.0));
        let what = match &report.outcome {
            RecoveryOutcome::FileRead { content, .. } if *content == recorded.content => "read-V".to_string(),
            RecoveryOutcome::FileRead { content, .. } if *content == previous_content => "read-OLDER(P)".to_string(),
            RecoveryOutcome::FileRead { .. } => "read-OTHER".to_string(),
            RecoveryOutcome::NoFile { .. } => "NOFILE".to_string(),
            RecoveryOutcome::Failed { failure, .. } => format!("FAILED({})", kind_of(&format!("{failure:?}"))),
        };
        println!("SILENT W={code} k={k} arm={} shape={name} v=({},{}) landed={landed:?} outcome={what}", arm(), v_key.0, v_key.1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let number = |i: usize| -> u64 { args.get(i).and_then(|v| v.parse().ok()).expect("参数是十进制数") };
    match args.get(1).map(String::as_str) {
        Some("cm") => subcommand_cm(&args[2], number(3), &args[4], &args[5]),
        Some("counts") => subcommand_counts(&args[2], number(3), &args[4], &args[5]),
        Some("grid") => subcommand_grid(&args[2], number(3), &args[4], &args[5], number(6) as usize, &args[7], false),
        Some("bcheck") => subcommand_grid(&args[2], number(3), &args[4], &args[5], number(6) as usize, &args[7], true),
        Some("silent") => subcommand_silent(&args[2], number(3)),
        Some("chaincm") => subcommand_chaincm(&args[2], number(3), &args[4], &args[5], &args[6], args[7].chars().next().expect("动作")),
        Some("stalecheck") => subcommand_stalecheck(&args[2], number(3), &args[4]),
        Some("narrow") => subcommand_narrow(&args[2], number(3), &args[4], &args[5], number(6)),
        _ => panic!("子命令：cm / counts / grid / bcheck / silent"),
    }
}

/// M1 之后照 `ops` 走一串用户动作，再把下一个动作 `last` 的写流录下来、层 0 全量枚举（每一段都展开）：每个崩溃状态跑池级 checker
/// （枚举自带）与 R 的可读性探针（R 还在环里时）。看「解开 C 隔离」的时机会不会让 R 还在环里时它的单元被写掉。
fn subcommand_chaincm(code: &str, k: u64, rkind: &str, hide: &str, ops: &str, last: char) {
    let head = format!("W={code} k={k} R={rkind} hide={hide} arm={} ops={ops} last={last}", arm());
    let recorded = record_r(widths(code), k, rkind).expect("前缀");
    let persisted = crash_after_r_root(&recorded);
    let crash_image = materialize(&recorded.base, &recorded.writes, &persisted);
    let r_root = ring_root(&crash_image, recorded.recorded).expect("R 在环里");
    let r_placements = placements_of(&crash_image, &r_root).unwrap_or_default();
    let (mut pool, summary) = abandoning_mount(&recorded, &persisted, hide).expect("M1");
    for op in ops.chars().filter(|c| *c != '-') {
        apply_op(&mut pool, op).expect("前面的动作做成");
    }
    let base = pool.image();
    let stream = SharedStream::retaining_contents();
    let session = pool.session.take().expect("会话");
    let mut recording = PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: plain_devices_on(&base).into_iter().map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone()))).collect(),
        session: Some(session),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    };
    let iso_before = recording.session().allocator.devices.iter().map(|d| d.isolated_slots()).sum::<u64>();
    let outcome = apply_op(&mut recording, last);
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &widths(code).fixed_geometry());
    let count = closed_form_state_count(&segments);
    let in_ring_before = ring_root(&base, recorded.recorded).is_some();
    let after_image = recording.image();
    println!("CC {head} M1[{summary}] in_ring_before={in_ring_before} in_ring_after={} iso_before={iso_before} last_outcome={outcome:?} dmg_by_last={} writes={} closed_form={count}",
        ring_root(&after_image, recorded.recorded).is_some(), changed_placements(&base, &after_image, &r_placements), writes.len());
    let Some(judged) = writes.iter().position(|w| w.kind == StepKind::RootRecordFua) else { return println!("CC {head} NO_ROOT_WRITE") };
    if count > 1_000_000 {
        // 全量超过约 10⁶ 个状态：不枚举（派发规则），只报一个点名的崩溃状态——根槽写之前的写全部持久、根槽与之后一个没持久
        // （层 0 枚举域里的那个整段前缀状态），给它跑池级 checker 与 R 的探针。报告里写明这一格没跑全量。
        let prefix: Vec<bool> = (0..writes.len()).map(|i| i < judged).collect();
        let state = materialize(&base, &writes, &prefix);
        let in_ring = ring_root(&state, recorded.recorded).is_some();
        let probe = if in_ring { probe_recovery_onto(&state, recorded.recorded, &recorded.content) } else { "-".to_string() };
        return println!(
            "CC {head} SKIPPED closed_form={count} named_state=before_root_write in_ring={in_ring} dmg={} chk=[{}] probe={probe}",
            changed_placements(&base, &state, &r_placements),
            checker_names(&state)
        );
    }
    let versions = recording.published_versions();
    let mut classes: BTreeMap<String, u64> = BTreeMap::new();
    let tally = singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state(
        &base, &writes, &segments, judged, &versions, &|_, _| true,
        &mut |image, _report| {
            let state = materialize(&base, &writes, &image.persisted);
            let in_ring = ring_root(&state, recorded.recorded).is_some();
            let dmg = changed_placements(&base, &state, &r_placements);
            let probe = if in_ring { probe_recovery_onto(&state, recorded.recorded, &recorded.content) } else { "-".to_string() };
            *classes.entry(format!("in_ring={in_ring} dmg={dmg} probe={probe}")).or_default() += 1;
        },
    );
    let checker: Vec<String> = tally.checker_violated_states.iter().filter(|(_, n)| **n > 0).map(|(name, n)| format!("{name}={n}")).collect();
    println!("L0TALLY CC {head} states={} violations={} failed={} checker_violated=[{}]", tally.states, tally.violations, tally.failed_states, checker.join(","));
    for (class, states) in classes {
        println!("CCCLASS {head} {class} states={states}");
    }
}

/// 窄盘代价：M1 之后一直覆盖写到被拒（至多 `steps` 次），报每一步隔离了几个槽、第几次被拒、拒因。
fn subcommand_narrow(code: &str, k: u64, rkind: &str, hide: &str, steps: u64) {
    let head = format!("W={code} k={k} R={rkind} hide={hide} arm={}", arm());
    let recorded = match record_r(widths(code), k, rkind) {
        Ok(recorded) => recorded,
        Err(error) => return println!("NARROW {head} PREFIX_FAILED {}", error.chars().take(120).collect::<String>()),
    };
    let persisted = crash_after_r_root(&recorded);
    let (mut pool, summary) = match abandoning_mount(&recorded, &persisted, hide) {
        Ok(after) => after,
        Err(error) => return println!("NARROW {head} r_records={} {error}", record_groups(&recorded.writes).len()),
    };
    let mut trail = Vec::new();
    let mut refused_at = None;
    for step in 1..=steps {
        match apply_op(&mut pool, 'o') {
            Ok(()) => trail.push(isolated_total(&pool).to_string()),
            Err(kind) => {
                refused_at = Some((step, kind));
                break;
            }
        }
    }
    // 被拒之后：卸载（拷贝上），再可写挂载一次、覆盖写一次——C 的隔离是不是只活在这一次挂载里。
    let (unmounted, image) = unmount_on_a_copy(&pool, |_, d| d, |d| d);
    let after = if unmounted.is_ok() {
        let mut devices = plain_devices_on(&image);
        match mount_writable_with_space_admission(&pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
            Ok(mounted) => {
                let (_output, session) = MountedSession::of_the_mount(mounted);
                let mut next = pool_from(&recorded, &image_of(&devices, pool.device_bytes, |d| d), session, Vec::new());
                let isolated = isolated_total(&next);
                let write = match apply_op(&mut next, 'o') { Ok(()) => "ok".to_string(), Err(kind) => kind };
                format!("remount=Ok(isolated={isolated}) write={write}")
            }
            Err(error) => format!("remount=Err({})", short_error(&error)),
        }
    } else {
        "-".to_string()
    };
    println!("NARROW {head} r_records={} M1[{summary}] refused_at={refused_at:?} isolated_after_each=[{}] unmount={} {after}", record_groups(&recorded.writes).len(), trail.join(","), short(&unmounted));
}

/// W（第一轮正推甲的凭据）在「读回写之前的旧内容」下会不会把 R 判成已被占：R 的树表与映射根各个位置条目上，R 写之前那一槽的内容
/// 是不是一个解得开、诞生身份与 R 的指针不同的码 2 节点（W 的判法照抄）。两样每个位置都是，就是 W 会跳过的盘面。
fn subcommand_stalecheck(code: &str, kmax: u64, rkind: &str) {
    for k in 0..=kmax {
        let Ok(recorded) = record_r(widths(code), k, rkind) else { println!("STALE W={code} k={k} R={rkind} PREFIX_FAILED"); continue };
        let full: Vec<bool> = vec![true; recorded.writes.len()];
        let image = materialize(&recorded.base, &recorded.writes, &full);
        let Some(r_root) = ring_root(&image, recorded.recorded) else { println!("STALE W={code} k={k} R={rkind} R_NOT_IN_RING"); continue };
        let foreign = |pointer: &singlefs_core::pointer::NodePointer| -> Vec<bool> {
            pointer.locations.iter().filter(|l| l.slot.0 != 0).map(|location| {
                PoolReader::read(&recorded.base, location.device, location.slot.to_device_offset(), 16384)
                    .and_then(|bytes| singlefs_core::unit::parse_index_node(&bytes).ok())
                    .is_some_and(|header| header.birth_txg != pointer.head.birth_txg || header.instance != pointer.instance || header.birth_sequence != pointer.birth_sequence)
            }).collect()
        };
        let tree = foreign(&r_root.tree_table);
        let mapping = foreign(&r_root.mapping_root);
        let w_would_skip = !tree.is_empty() && !mapping.is_empty() && tree.iter().chain(&mapping).all(|f| *f);
        println!("STALE W={code} k={k} R={rkind} tree_table_old_slot_foreign={tree:?} mapping_root_old_slot_foreign={mapping:?} w_would_skip_under_stale_reads={w_would_skip}");
    }
}
