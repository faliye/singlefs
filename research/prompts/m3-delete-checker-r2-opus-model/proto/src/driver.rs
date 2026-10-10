//! 内存稀疏盘上的一段历史：mkfs → 取号 → 暖机 → 新池新建文件（实例 1，进程内）→ 可写挂载开会话，之后每一步经会话或核心入口发。
//! 每一步记下步前镜像、这一步在录制流里的区间与步后镜像，供逐步判与按层 0 枚举域枚举崩溃状态。原型自己造流，不从带 layer0 的用例拷。
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{choose_system_configuration, readable_roots};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, warm_up, FirstFile,
    InodeNumberWatermarkPressedBackForTests, InodeNumberWatermarkSource, PoolVersion, PoolWriter,
};
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

use crate::common::{geometry, parameters, FIXED_WRITE_TIME_SECONDS};

pub type Dev = RecordingBlockDevice<SparseBlockDevice>;

/// 一次挂载期间点名的几段读回全 0（与历史执行器「崩溃恢复抛弃根」同一造法：读照读、落在藏起来那几段里的字节清零；写到哪段哪段就不再藏）。
pub struct Hide {
    pub inner: Dev,
    pub hidden: Vec<(u64, u64)>,
}

fn overlaps(range: (u64, u64), offset: u64, length: u64) -> bool {
    range.0 < offset + length && offset < range.0 + range.1
}

impl BlockDevice for Hide {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let length = buffer.len() as u64;
        for range in &self.hidden {
            if !overlaps(*range, offset.0, length) {
                continue;
            }
            let start = range.0.saturating_sub(offset.0);
            let end = (range.0 + range.1 - offset.0).min(length);
            buffer[start as usize..end as usize].fill(0);
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let length = bytes.len() as u64;
        self.hidden.retain(|range| !overlaps(*range, offset.0, length));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.hidden.retain(|range| !overlaps(*range, offset.0, length));
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

/// 历史里的一步（用户决定的那几步都在这里，原型放开扫）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// 经会话建 n 个 size 0 的 inode。
    Create(u64),
    /// 经会话 unlink 号 k（走空 + 意图 + 全部批）。
    Unlink(u64),
    /// 经会话顺序写第一个文件，n 个数据单元的内容。
    Write(u64),
    /// 管理员回退：根环里按 (txg, 实例) 从新到旧第 k 条（取模）。
    Rollback(u64),
    /// 崩溃恢复抛弃最新那条根：崩在它的根槽 FUA 之后、系统配置轮换写之前，下一次可写挂载时它的根槽与它那次发布点名的单元读回全 0。
    CrashRecovery,
    /// 关掉会话、正常可写挂载一次（挂载续做）。
    Remount,
    /// 坏：只供测试的水位开关把起号压回 p，再建 n 个（C479）。
    PressBack(u64, u64),
    /// 崩溃：上一步写出的那一段只持久了前 k 段（first 为真时再加第 k 段的第一个写），之后正常可写挂载一次。
    /// 崩溃镜像本身与挂载之后的镜像各记一步。
    Crash(u64, bool),
}

impl Op {
    pub fn name(&self) -> String {
        match self {
            Op::Create(n) => format!("C{n}"),
            Op::Unlink(k) => format!("U{k}"),
            Op::Write(n) => format!("W{n}"),
            Op::Rollback(k) => format!("R{k}"),
            Op::CrashRecovery => "X".into(),
            Op::Remount => "M".into(),
            Op::PressBack(p, n) => format!("P{p}x{n}"),
            Op::Crash(k, first) => format!("Y{k}{}", if *first { "f" } else { "" }),
        }
    }
}

pub struct Step {
    pub label: String,
    pub base: MemoryPool,
    pub stream: SharedStream,
    pub start: usize,
    pub end: usize,
    pub after: MemoryPool,
    pub outcome: String,
}

pub struct Pool {
    pub devices: Vec<(DeviceIdentity, Dev)>,
    pub stream: SharedStream,
    /// 这条录制流开头的盘面（崩溃恢复换流之后是被截掉系统配置轮换写的那一份）。
    pub stream_base: MemoryPool,
    pub session: Option<MountedSession>,
    pub device_bytes: u64,
    pub time: u64,
    pub switches: DeletionSwitches,
    pub steps: Vec<Step>,
    /// 假：每步不留步前步后两份镜像（省内存；只要最后那一份时用）。
    pub keep_images: bool,
}

pub fn switches(batch_units: u64) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: if batch_units >= 144 {
            UnlinkBatchSize::OneExtentLowerLeaf
        } else {
            UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(batch_units).expect("1..=144"))
        },
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 只供测试的三个容量开关压小（墓碑容器 t 条、谱系容器 l 条、容器树码 2 节点 e 条）。
pub fn switches_with_knobs(batch_units: u64, tombstones: u64, lineage: u64, entries: usize) -> DeletionSwitches {
    use singlefs_core::container_tree::ContainerTreeNodeEntriesCappedForTests;
    use singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests;
    use singlefs_core::tombstone_tree::TombstoneRecordsPerContainerCappedForTests;
    let mut out = switches(batch_units);
    out.tombstone_container_capacity =
        TombstoneContainerCapacity::CappedForTests(TombstoneRecordsPerContainerCappedForTests::new(tombstones).expect("1..=815"));
    out.lineage_container_capacity =
        LineageContainerCapacity::CappedForTests(LineageRecordsPerContainerCappedForTests::new(lineage).expect("1..=2039"));
    out.container_tree_node_entries =
        ContainerTreeNodeEntries::CappedForTests(ContainerTreeNodeEntriesCappedForTests::new(entries).expect("2..=135"));
    out
}

pub fn content_of(bytes: usize) -> Vec<u8> {
    (0..bytes).map(|index| ((index * 13 + 5) % 251) as u8).collect()
}

fn image_of(devices: &[(DeviceIdentity, Dev)], device_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    }
}

fn devices_of(image: &MemoryPool, stream: &SharedStream) -> Vec<(DeviceIdentity, Dev)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
        })
        .collect()
}

/// 根环里读得出的根，按 (txg, 实例) 从新到旧、去重。
pub fn ring_roots_newest_first(image: &MemoryPool) -> Vec<(u64, u32)> {
    let Ok(configuration) = choose_system_configuration(image) else {
        return Vec::new();
    };
    let mut roots: Vec<(u64, u32)> = readable_roots(
        image,
        &configuration.immutable.region_devices,
        &configuration.immutable.sizes,
        &configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|root| (root.checkpoint_txg.0, root.instance.0))
    .collect();
    roots.sort_unstable_by(|left, right| right.cmp(left));
    roots.dedup();
    roots
}

impl Pool {
    /// 前缀固定：mkfs → 取号 → 暖机 → 新池新建文件（3000 字节，实例 1）→ 正常可写挂载开会话（实例 2）。
    pub fn build(device_bytes: u64, batch_units: u64) -> Pool {
        Pool::build_with(device_bytes, switches(batch_units))
    }

    pub fn build_with(device_bytes: u64, chosen_switches: DeletionSwitches) -> Pool {
        let parameters = parameters();
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, Dev)> = (0..2u32)
            .map(|number| {
                (
                    DeviceIdentity(number),
                    RecordingBlockDevice::with_shared_stream(
                        DeviceIdentity(number),
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
        let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
        let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
        {
            let mut writer = PoolWriter::new(&parameters, &mut devices);
            let instance = acquire_instance(&mut writer).expect("取号");
            let warm = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
            publish_first_file(
                &mut writer,
                &mut allocator,
                warm.roots.last().expect("暖机根"),
                FirstFile { content: &content_of(3000), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                instance,
                &warm.last_record_bytes,
            )
            .expect("新池新建文件");
        }
        let empty = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], device_bytes);
        let mut pool = Pool {
            devices,
            stream,
            stream_base: empty,
            session: None,
            device_bytes,
            time: FIXED_WRITE_TIME_SECONDS,
            switches: chosen_switches,
            steps: Vec::new(),
            keep_images: true,
        };
        pool.run(&Op::Remount);
        pool
    }

    pub fn image(&self) -> MemoryPool {
        image_of(&self.devices, self.device_bytes)
    }

    /// 会话现行那一版记账里的 inode 号水位。
    pub fn watermark(&self) -> Option<u64> {
        let session = self.session.as_ref()?;
        match &session.current {
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => Some(version.inode_number_watermark()),
            PoolVersion::WithoutFile(_) => None,
        }
    }

    fn mount(&mut self, hidden: Option<Vec<(DeviceIdentity, (u64, u64))>>) -> String {
        self.session = None;
        let parameters = parameters();
        let result = match hidden {
            None => mount_writable_with_test_only_switches_and_deletion_switches(
                &parameters,
                &mut self.devices,
                SpaceAdmission::JudgedByTheFormula,
                ShadowLedger::On,
                self.switches,
            ),
            Some(ranges) => {
                let mut hiding: Vec<(DeviceIdentity, Hide)> = std::mem::take(&mut self.devices)
                    .into_iter()
                    .map(|(identity, device)| {
                        let hidden = ranges.iter().filter(|(of, _)| *of == identity).map(|(_, range)| *range).collect();
                        (identity, Hide { inner: device, hidden })
                    })
                    .collect();
                let result = mount_writable_with_test_only_switches_and_deletion_switches(
                    &parameters,
                    &mut hiding,
                    SpaceAdmission::JudgedByTheFormula,
                    ShadowLedger::On,
                    self.switches,
                );
                self.devices = hiding.into_iter().map(|(identity, hide)| (identity, hide.inner)).collect();
                result
            }
        };
        match result {
            Ok(mounted) => {
                let (output, session) = MountedSession::of_the_mount(mounted);
                let text = format!(
                    "mounted inst{} chosen=txg{} effective=txg{} continuation={}",
                    output.instance.0,
                    output.chosen_root.checkpoint_txg.0,
                    output.effective_root.checkpoint_txg.0,
                    output.deletion_queue_continuation.len()
                );
                self.session = Some(session);
                text
            }
            Err(error) => format!("MOUNT_REFUSED {}", format!("{error:?}").chars().take(160).collect::<String>()),
        }
    }

    /// 崩溃恢复抛弃最新那条根：现行一版是根环里最新的那条；去掉流尾最后一次根槽 FUA 之后的系统配置槽写（崩在轮换之前），
    /// 换一条新流，藏起最新根的根槽与它那次发布点名的单元，可写挂载。
    fn crash_recovery(&mut self) -> String {
        let Some(session) = self.session.as_ref() else {
            return "NA no-session".into();
        };
        let current = session.current.clone();
        let roots = ring_roots_newest_first(&self.image());
        if roots.first().copied() != Some((current.root().checkpoint_txg.0, current.root().instance.0)) || roots.len() < 2 {
            return "NA newest-is-not-current".into();
        }
        let earlier = match &current {
            PoolVersion::WithoutFile(version) => &version.earlier_records_of_this_publish,
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => &version.earlier_records_of_this_publish,
        };
        let named: Vec<_> = earlier
            .iter()
            .map(|written| &written.record)
            .chain(std::iter::once(current.record()))
            .flat_map(|record| record.named.iter())
            .flat_map(|named| named.locations)
            .collect();
        if named.is_empty() {
            return "NA names-no-unit".into();
        }
        let parameters = parameters();
        let slot = singlefs_core::root_ring::target_for_publish(current.root().checkpoint_txg, parameters.geometry.root_ring_slots_per_region);
        let root_device = parameters.region_devices[slot.region as usize];
        let root_offset = singlefs_core::root_ring::slot_offset(slot, parameters.geometry.fixed_structure_slot_spacing);
        let mut hidden = vec![(root_device, (root_offset.0, u64::from(parameters.geometry.physical_block_size)))];
        hidden.extend(named.into_iter().map(|location| (location.device, (location.slot.to_device_offset().0, SLOT_BYTES))));
        // 截掉流尾最后一次根槽 FUA 之后的系统配置槽写。
        let operations = self.stream.retained_operations();
        let geometry = geometry();
        let last_root = operations
            .iter()
            .rposition(|operation| geometry.classify(&operation.operation) == StepKind::RootRecordFua)
            .expect("流里有根槽写");
        let dropped: Vec<usize> = (last_root + 1..operations.len())
            .filter(|index| geometry.classify(&operations[*index].operation) == StepKind::SystemConfigurationSlot)
            .collect();
        if dropped.is_empty() {
            return "NA no-rotation-after-root".into();
        }
        let kept: Vec<_> = operations
            .iter()
            .enumerate()
            .filter(|(index, _)| !dropped.contains(index))
            .map(|(_, operation)| operation.clone())
            .collect();
        let mut crashed = self.stream_base.clone();
        crashed.apply(&kept);
        let stream = SharedStream::retaining_contents();
        self.devices = devices_of(&crashed, &stream);
        self.stream = stream;
        self.stream_base = crashed;
        let text = self.mount(Some(hidden));
        format!("dropped_rotation_writes={} {text}", dropped.len())
    }

    fn rollback(&mut self, k: u64) -> String {
        let roots = ring_roots_newest_first(&self.image());
        let Some(session) = self.session.as_mut() else {
            return "NA no-session".into();
        };
        if roots.is_empty() {
            return "NA no-root".into();
        }
        let (txg, instance) = roots[(k as usize) % roots.len()];
        let parameters = parameters();
        let current = match &mut session.current {
            PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
            PoolVersion::WithoutFile(_) => return "NA without-file".into(),
        };
        let target = RollbackTarget {
            instance: singlefs_core::address::InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(txg),
        };
        match roll_back_by_a_forward_publish(&parameters, &mut self.devices, &mut session.allocator, current, target) {
            Ok(_) => {
                let version = current.clone();
                session.current = PoolVersion::of_a_version_of_trees(version);
                format!("rolled-back-to txg{txg}/inst{instance} now txg{}", session.current.root().checkpoint_txg.0)
            }
            Err(error) => format!("REFUSED {}", format!("{error:?}").chars().take(120).collect::<String>()),
        }
    }

    fn user_change(&mut self, change: UserChange<'_>) -> String {
        let Some(session) = self.session.as_mut() else {
            return "NA no-session".into();
        };
        match session.publish_user_change(&mut self.devices, change) {
            Ok(published) => format!("ok versions={} now txg{}", published.versions_published.len(), session.current.root().checkpoint_txg.0),
            Err(refused) => format!("REFUSED {}", format!("{refused:?}").chars().take(120).collect::<String>()),
        }
    }

    fn press_back(&mut self, p: u64, n: u64) -> String {
        let Some(session) = self.session.as_mut() else {
            return "NA no-session".into();
        };
        let (PoolVersion::WithFile(previous) | PoolVersion::WithTreesAndNoFile(previous)) = &session.current else {
            return "NA without-file".into();
        };
        let previous = previous.clone();
        let parameters = parameters();
        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
        let Some(pressed) = InodeNumberWatermarkPressedBackForTests::new(p) else {
            return "NA press-value".into();
        };
        writer.set_inode_number_watermark_source(InodeNumberWatermarkSource::PressedBackForTests(pressed));
        match publish_new_inodes(&mut writer, &mut session.allocator, &previous, n, self.time, session.instance) {
            Ok(output) => {
                session.current = PoolVersion::of_a_version_of_trees(output);
                format!("pressed now txg{}", session.current.root().checkpoint_txg.0)
            }
            Err(error) => format!("REFUSED {}", format!("{error:?}").chars().take(120).collect::<String>()),
        }
    }

    fn crash_at(&mut self, k: u64, first: bool) -> String {
        let Some(previous) = self.steps.last() else {
            return "NA no-previous-step".into();
        };
        let operations = previous.stream.retained_operations();
        let (writes, segments) =
            singlefs_harness::memory_pool::writes_and_segments(&operations[previous.start..previous.end], &geometry());
        let k = k as usize;
        if k >= segments.len() || (k == 0 && !first) {
            return format!("NA k={k} segments={}", segments.len());
        }
        let mut persisted: Vec<usize> = segments[..k].iter().flatten().copied().collect();
        if first {
            persisted.push(segments[k][0]);
        }
        persisted.sort_unstable();
        let chosen: Vec<_> = persisted.iter().map(|index| writes[*index].clone()).collect();
        let mut crashed = previous.base.clone();
        crashed.apply_writes(&chosen);
        let lengths: Vec<usize> = segments.iter().map(Vec::len).collect();
        let stream = SharedStream::retaining_contents();
        self.session = None;
        self.devices = devices_of(&crashed, &stream);
        self.stream = stream;
        self.stream_base = crashed.clone();
        self.steps.push(Step {
            label: format!("Y{k}{}-image", if first { "f" } else { "" }),
            base: crashed.clone(),
            stream: self.stream.clone(),
            start: 0,
            end: 0,
            after: crashed,
            outcome: format!("crash image: persisted {} of {} writes, segments {lengths:?}", persisted.len(), writes.len()),
        });
        self.mount(None)
    }

    pub fn run(&mut self, op: &Op) -> String {
        self.time += 60;
        let time = self.time;
        let base = self.image();
        let start = self.stream.operation_count();
        let stream_before = self.stream.clone();
        let outcome = match op {
            Op::Create(n) => self.user_change(UserChange::NewInodes { count: *n, write_time_seconds: time }),
            Op::Unlink(k) => self.user_change(UserChange::Unlink { inode: InodeNumber(*k), write_time_seconds: time }),
            Op::Write(units) => {
                let content = content_of((*units as usize) * singlefs_core::unit::data_unit_payload_capacity() - 100);
                self.user_change(UserChange::SequentialWrite(FirstFile { content: &content, write_time_seconds: time }))
            }
            Op::Rollback(k) => self.rollback(*k),
            Op::CrashRecovery => self.crash_recovery(),
            Op::Remount => self.mount(None),
            Op::PressBack(p, n) => self.press_back(*p, *n),
            Op::Crash(k, first) => self.crash_at(*k, *first),
        };
        // 崩溃恢复换了流：这一步的枚举基线是被截掉轮换写的那份盘面，写是新流从头起的那一段。
        let (base, stream, start) = if matches!(op, Op::CrashRecovery | Op::Crash(..)) && !outcome.starts_with("NA") {
            (self.stream_base.clone(), self.stream.clone(), 0)
        } else {
            (base, stream_before, start)
        };
        let end = stream.operation_count();
        let (base, after) = if self.keep_images { (base, self.image()) } else { (MemoryPool::default(), MemoryPool::default()) };
        self.steps.push(Step { label: op.name(), base, stream, start, end, after, outcome: outcome.clone() });
        outcome
    }
}
