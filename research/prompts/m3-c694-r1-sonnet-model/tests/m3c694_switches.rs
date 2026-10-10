//! C694 复现（调查员草稿，不入库）：崩溃恢复落到旧根、取新实例的那次挂载期间，被抛弃的根引用的单元被复用之后，
//! 正常卸载算影子账、读那条根的账读不出，在任何写之前报 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot …)`。
//! 造法照攻方原型 `research/prompts/m3-batch14-code-r1-opus-model/probe/main.rs` 的 `abandon_start`。

mod common_admission;

use common_admission::{plain_devices_on, start_plain, PoolUnderTest, OVERWRITE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::mount::{mount_writable_with_space_admission, unmount, MountError, Unmounted};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
};
use singlefs_core::recovery::{allocation_records_under_root, PoolReader};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const RECORDED_OVERWRITE_TIME: u64 = common_admission::FIRST_WRITE_TIME_SECONDS + 100_000;

/// 一次挂载期间点名的几段读回全 0；被写到的那一段不再藏（同 history.rs 私有的 `DeviceReadingZerosOverHiddenRanges`）。
struct Hiding {
    inner: SparseBlockDevice,
    hidden: Vec<(u64, u64)>,
}

impl BlockDevice for Hiding {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let length = buffer.len() as u64;
        for &(start, len) in &self.hidden {
            if start < offset.0 + length && offset.0 < start + len {
                let a = start.saturating_sub(offset.0) as usize;
                let b = ((start + len - offset.0).min(length)) as usize;
                buffer[a..b].fill(0);
            }
        }
        Ok(())
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let length = bytes.len() as u64;
        self.hidden.retain(|&(start, len)| !(start < offset.0 + length && offset.0 < start + len));
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.hidden.retain(|&(start, len)| !(start < offset.0 + length && offset.0 < start + len));
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

fn materialize(base: &MemoryPool, writes: &[RetainedWrite], persisted_through: usize) -> MemoryPool {
    let mut pool = base.clone();
    for write in writes.iter().take(persisted_through + 1) {
        write.contents.apply_to(pool.devices.get_mut(&write.device).expect("盘"), write.offset);
    }
    pool
}

pub struct Abandoned {
    pub pool: PoolUnderTest<SparseBlockDevice>,
    pub abandoned: (InstanceGeneration, CheckpointTxg),
    pub row_txg: u64,
    pub mount_abandoned_roots_unreadable: u64,
    pub mount_isolated: Vec<(DeviceIdentity, u64)>,
    /// 崩在根槽写落下那一刀的盘面（取新实例那次挂载之前）。
    pub crashed: MemoryPool,
}

/// 普通盘起点覆盖写 k 次；在拷贝上录一次覆盖写，崩在它的根槽写落下的那一刀；那一刀上可写挂载时把这次发布的单元写与根槽读回全 0：
/// 恢复落到上一版、取新实例，那条根留在环里成了被抛弃的根。挂载交回之后换回不藏的盘，再覆盖写 k2 次。
pub fn abandon_start(width: HistoryDeviceWidth, overwrites: u64, overwrites_after: u64) -> Result<Abandoned, String> {
    let mut pool = start_plain(width);
    for i in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite{i}:{e:?}"))?;
    }
    let image = pool.image();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&image)
        .into_iter()
        .map(|(id, d)| (id, RecordingBlockDevice::with_shared_stream(id, d, stream.clone())))
        .collect();
    let abandoned = {
        let session = pool.session.as_ref().expect("会话");
        let mut allocator = session.allocator.clone();
        let previous = session.current.file_version().expect("带文件").clone();
        let mut writer = PoolWriter::new(&pool.parameters, devices.as_mut_slice());
        let content = vec![0x5a_u8; OVERWRITE_BYTES];
        let root = publish_overwrite(&mut writer, &mut allocator, &previous,
            FirstFile { content: &content, write_time_seconds: RECORDED_OVERWRITE_TIME }, session.instance)
            .map_err(|e| format!("recorded-overwrite:{e:?}"))?
            .root;
        (root.instance, root.checkpoint_txg)
    };
    let (writes, _segments, _) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &width.fixed_geometry());
    let root_index = writes.iter().position(|w| w.kind == StepKind::RootRecordFua).ok_or("没有根槽写")?;
    let crashed = materialize(&image, &writes, root_index);
    let crashed_copy = crashed.clone();
    let hidden_of = |device: DeviceIdentity| -> Vec<(u64, u64)> {
        writes.iter().take(root_index + 1)
            .filter(|w| w.device == device && (w.kind == StepKind::UnitWrite || w.kind == StepKind::RootRecordFua))
            .map(|w| (w.offset.0, w.length_in_bytes()))
            .collect()
    };
    let mut hiding: Vec<(DeviceIdentity, Hiding)> = plain_devices_on(&crashed_copy)
        .into_iter()
        .map(|(id, d)| (id, Hiding { inner: d, hidden: hidden_of(id) }))
        .collect();
    pool.session = None;
    let mounted = mount_writable_with_space_admission(&pool.parameters, &mut hiding, SpaceAdmission::JudgedByTheFormula)
        .map_err(|e| format!("abandoning-mount:{e:?}"))?;
    let (output, session) = MountedSession::of_the_mount(mounted);
    let row_txg = output.row_publish.root().checkpoint_txg.0;
    pool.devices = hiding.into_iter().map(|(id, h)| (id, h.inner)).collect();
    pool.session = Some(session);
    for i in 0..overwrites_after {
        pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("after-abandon-overwrite{i}:{e:?}"))?;
    }
    Ok(Abandoned {
        pool,
        abandoned,
        row_txg,
        mount_abandoned_roots_unreadable: output.abandoned_roots_unreadable,
        mount_isolated: output.isolated_slots_per_device.clone(),
        crashed,
    })
}

/// 在这一刻盘面的拷贝上卸载一次（会话的分配器与现行版本各拷一份），盘与会话都不动。
pub fn unmount_on_a_copy(pool: &PoolUnderTest<SparseBlockDevice>) -> Result<Unmounted, MountError> {
    let mut devices = plain_devices_on(&pool.image());
    let session = pool.session.as_ref().expect("会话");
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    unmount(&pool.parameters, &mut devices, &mut allocator, &mut current, session.shadow_ledger)
}

/// 被抛弃的那条根还在不在根环里（整环读一遍，找 (实例, txg) 相同、自证过的根）。
pub fn abandoned_root_still_in_the_ring(image: &MemoryPool, abandoned: (InstanceGeneration, CheckpointTxg)) -> bool {
    let configuration = choose_system_configuration(image).expect("系统配置读得出");
    every_root_ring_slot(&configuration.immutable.sizes).into_iter().any(|ring_slot| {
        match read_root_ring_slot(image, &configuration.immutable.region_devices, &configuration.immutable.sizes,
            &configuration.immutable.filesystem_identifier, ring_slot) {
            RootRingSlotReading::SelfVerified(root) => (root.instance, root.checkpoint_txg) == abandoned,
            RootRingSlotReading::Bad(_) => false,
        }
    })
}

/// 环里那条被抛弃的根（自证过的那一条）。
pub fn abandoned_root_in_the_ring(image: &MemoryPool, abandoned: (InstanceGeneration, CheckpointTxg)) -> Option<RootRecord> {
    let configuration = choose_system_configuration(image).expect("系统配置读得出");
    every_root_ring_slot(&configuration.immutable.sizes).into_iter().find_map(|ring_slot| {
        match read_root_ring_slot(image, &configuration.immutable.region_devices, &configuration.immutable.sizes,
            &configuration.immutable.filesystem_identifier, ring_slot) {
            RootRingSlotReading::SelfVerified(root) if (root.instance, root.checkpoint_txg) == abandoned => Some(root),
            _ => None,
        }
    })
}

/// 被抛弃的根直接指着的几个单元（树表、分配记录树根、中央映射根、实例表）：崩溃那一刻的盘面与这一刻比，槽里的字节变了几处；
/// 再用 recovery 的公开读者读它的分配记录（读得出交条数，读不出交错）。
pub fn account_units_of_the_abandoned_root(crashed: &MemoryPool, now: &MemoryPool, root: &RootRecord) -> String {
    let mut parts = Vec::new();
    for (name, pointer) in [("tree_table", &root.tree_table), ("alloc_tree_root", &root.allocation_record_tree_root),
        ("mapping_root", &root.mapping_root), ("instance_table", &root.instance_table)] {
        let mut changed = Vec::new();
        for location in &pointer.locations {
            let offset = DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES);
            let before = PoolReader::read(crashed, location.device, offset, SLOT_BYTES as usize);
            let after = PoolReader::read(now, location.device, offset, SLOT_BYTES as usize);
            changed.push(format!("d{}s{}{}", location.device.0, location.slot.0, if before == after { "=" } else { "!" }));
        }
        parts.push(format!("{name}[{}]", changed.join(",")));
    }
    let records = match allocation_records_under_root(now, root) {
        Ok(records) => format!("records=Ok({})", records.len()),
        Err(failure) => format!("records=Err({failure:?})"),
    };
    format!("{} {records}", parts.join(" "))
}

fn short(result: &Result<Unmounted, MountError>) -> String {
    match result {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => format!("Ok publishes={}", raised.publishes.len()),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "Ok NothingWritten".to_string(),
        Err(error) => format!("Err {error:?}"),
    }
}

fn widths() -> Vec<(u64, HistoryDeviceWidth)> {
    vec![
        (384, HistoryDeviceWidth::UnitAreaOf384Slots),
        (256, HistoryDeviceWidth::UnitAreaOf256Slots),
        (240, HistoryDeviceWidth::UnitAreaOf240Slots),
        (4, HistoryDeviceWidth::FourGibibytes),
    ]
}

/// 攻方那张网格：宽 384、256、240、4 GiB × 覆盖写 1、2、3、5、8 × 挂载之后再覆盖写 0、1、2、4。每格打一行；
/// 拒的那几格接着在同一会话上卸载一次，再逐次覆盖写，数到被抛弃的根离开根环、卸载做成各是第几次。
#[test]
fn c694_sweep_of_the_attackers_grid() {
    let mut refused = 0;
    let mut starts = 0;
    // C694_WIDTHS（逗号分隔的 384、256、240、4）只跑点名的几档；不设跑全部四档。
    let only: Option<Vec<u64>> = std::env::var("C694_WIDTHS").ok()
        .map(|value| value.split(',').map(|code| code.parse().expect("十进制")).collect());
    for (code, width) in widths().into_iter().filter(|(code, _)| only.as_ref().is_none_or(|only| only.contains(code))) {
        for k in [1_u64, 2, 3, 5, 8] {
            for k2 in [0_u64, 1, 2, 4] {
                let label = format!("W={code} k={k} k2={k2}");
                let mut start = match abandon_start(width, k, k2) {
                    Ok(start) => start,
                    Err(error) => {
                        println!("START_FAILED {label}: {}", error.chars().take(200).collect::<String>());
                        continue;
                    }
                };
                starts += 1;
                let first = unmount_on_a_copy(&start.pool);
                println!(
                    "START {label} abandoned={:?} row_txg={} current_txg={} mount_abandoned_unreadable={} mount_isolated={:?} in_ring={} unmount={}",
                    start.abandoned, start.row_txg, start.pool.session().current.root().checkpoint_txg.0,
                    start.mount_abandoned_roots_unreadable, start.mount_isolated,
                    abandoned_root_still_in_the_ring(&start.pool.image(), start.abandoned), short(&first)
                );
                let image = start.pool.image();
                match abandoned_root_in_the_ring(&image, start.abandoned) {
                    Some(root) => println!("  ACCOUNT {label} refused={} {}", first.is_err(), account_units_of_the_abandoned_root(&start.crashed, &image, &root)),
                    None => println!("  ACCOUNT {label} refused={} not-in-ring", first.is_err()),
                }
                if first.is_ok() {
                    continue;
                }
                refused += 1;
                // 同一会话接着再卸载一次（真盘上，不是拷贝；拒在任何写之前，分配器换回）。
                let again = {
                    let pool = &mut start.pool;
                    let session = pool.session.as_mut().expect("会话");
                    let mut allocator = session.allocator.clone();
                    let mut current = session.current.clone();
                    let before = pool.devices.iter().map(|(_, d)| d.image.clone()).collect::<Vec<_>>();
                    let outcome = unmount(&pool.parameters, &mut pool.devices, &mut allocator, &mut current, session.shadow_ledger);
                    let after = pool.devices.iter().map(|(_, d)| d.image.clone()).collect::<Vec<_>>();
                    let unchanged = before == after;
                    format!("{} disk_unchanged={unchanged}", short(&outcome))
                };
                println!("  AGAIN {label} unmount={again}");
                // 进程退出、在这份盘面的拷贝上再可写挂载一次（准入判着）：挂载自己的影子账（mount.rs 里另一处调用）读不读得出那条根的账。
                {
                    let mut devices = plain_devices_on(&start.pool.image());
                    let remount = match mount_writable_with_space_admission(&start.pool.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
                        Ok(mounted) => {
                            let (output, mut session) = MountedSession::of_the_mount(mounted);
                            let unmounted = unmount(&start.pool.parameters, &mut devices, &mut session.allocator, &mut session.current, session.shadow_ledger);
                            format!("Ok abandoned_unreadable={} then_unmount={}", output.abandoned_roots_unreadable, short(&unmounted))
                        }
                        Err(error) => format!("Err {error:?}"),
                    };
                    println!("  REMOUNT {label} mount={}", remount.chars().take(220).collect::<String>());
                }
                let mut left_ring_at = None;
                let mut unmount_ok_at = None;
                for publish in 1..=200_u64 {
                    if let Err(error) = start.pool.overwrite(OVERWRITE_BYTES) {
                        println!("  OVERWRITE_FAILED {label} at={publish}: {}", format!("{error:?}").chars().take(160).collect::<String>());
                        break;
                    }
                    let in_ring = abandoned_root_still_in_the_ring(&start.pool.image(), start.abandoned);
                    let outcome = unmount_on_a_copy(&start.pool);
                    if left_ring_at.is_none() && !in_ring {
                        left_ring_at = Some(publish);
                    }
                    if unmount_ok_at.is_none() && outcome.is_ok() {
                        unmount_ok_at = Some(publish);
                    }
                    println!("  STEP {label} publish={publish} txg={} in_ring={in_ring} unmount={}",
                        start.pool.session().current.root().checkpoint_txg.0, short(&outcome).chars().take(160).collect::<String>());
                    if left_ring_at.is_some() && unmount_ok_at.is_some() {
                        break;
                    }
                }
                println!("  RESULT {label} left_ring_at={left_ring_at:?} unmount_ok_at={unmount_ok_at:?}");
                if left_ring_at.is_none() || unmount_ok_at.is_none() {
                    push_out_with_raw_publishes(&mut start, &label);
                }
            }
        }
    }
    println!("SUMMARY starts={starts} refused={refused}");
}

/// 会话的覆盖写被同一个错拒了之后：绕开准入与推抬 F，直接在会话的分配器上 `publish_overwrite`（准入关成只供测试的开关），
/// 逐次发布，数到被抛弃的根离开根环、卸载做成各是这一段的第几次。
fn push_out_with_raw_publishes(start: &mut Abandoned, label: &str) {
    let mut left_ring_at = None;
    let mut unmount_ok_at = None;
    {
        let session = start.pool.session.as_mut().expect("会话");
        session.allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    }
    for publish in 1..=200_u64 {
        let outcome = {
            let pool = &mut start.pool;
            let session = pool.session.as_mut().expect("会话");
            let previous = session.current.file_version().expect("带文件").clone();
            let content = vec![(publish % 251) as u8; OVERWRITE_BYTES];
            let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
            publish_overwrite(&mut writer, &mut session.allocator, &previous,
                FirstFile { content: &content, write_time_seconds: RECORDED_OVERWRITE_TIME + publish }, session.instance)
        };
        match outcome {
            Ok(output) => start.pool.session.as_mut().expect("会话").current = PoolVersion::WithFile(output),
            Err(error) => {
                println!("  RAW_FAILED {label} at={publish}: {}", format!("{error:?}").chars().take(160).collect::<String>());
                break;
            }
        }
        let in_ring = abandoned_root_still_in_the_ring(&start.pool.image(), start.abandoned);
        let outcome = unmount_on_a_copy(&start.pool);
        if left_ring_at.is_none() && !in_ring {
            left_ring_at = Some(publish);
        }
        if unmount_ok_at.is_none() && outcome.is_ok() {
            unmount_ok_at = Some(publish);
        }
        println!("  RAW {label} publish={publish} txg={} in_ring={in_ring} unmount={}",
            start.pool.session().current.root().checkpoint_txg.0, short(&outcome).chars().take(160).collect::<String>());
        if left_ring_at.is_some() && unmount_ok_at.is_some() {
            break;
        }
    }
    println!("  RAW_RESULT {label} left_ring_at={left_ring_at:?} unmount_ok_at={unmount_ok_at:?}");
}

/// 最小复现：384 槽盘，第一个文件之后覆盖写 5 次，造被抛弃的根（txg 9），挂载之后不再写，正常卸载被拒。
#[test]
fn c694_minimal_reproduction_unmount_is_refused_on_the_account_of_the_abandoned_root() {
    let start = abandon_start(HistoryDeviceWidth::UnitAreaOf384Slots, 5, 0).expect("起点造得出");
    assert_eq!(start.abandoned, (InstanceGeneration(1), CheckpointTxg(9)));
    let outcome = unmount_on_a_copy(&start.pool);
    println!("MINIMAL unmount={}", short(&outcome));
    match outcome {
        Err(MountError::NewerStateStillUnreadableAfterOneReread(still)) => {
            assert_eq!(format!("{still:?}"),
                "AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(9) } }");
        }
        other => panic!("C694 没复现：{}", short(&other)),
    }
}

/// 在一份盘面拷贝上把点名的几个槽（两块盘同槽）写零，再在会话的分配器与现行版本的拷贝上卸载。
fn unmount_after_zeroing(pool: &PoolUnderTest<SparseBlockDevice>, slots: &[u64]) -> Result<Unmounted, MountError> {
    let mut devices = plain_devices_on(&pool.image());
    for (_, device) in devices.iter_mut() {
        for slot in slots {
            device.write_zeroes_at(DeviceOffsetInBytes(slot * SLOT_BYTES), SLOT_BYTES).expect("写零");
        }
    }
    let session = pool.session.as_ref().expect("会话");
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    unmount(&pool.parameters, &mut devices, &mut allocator, &mut current, session.shadow_ledger)
}

/// 推翻条件的自证：卸载做得成的起点（覆盖写 3 次）上，只抹被抛弃根的树表槽（账读不出、映射现算还成）应照样做得成；
/// 树表槽与中央映射根槽一起抹（账与现算都不成）应报同一个错。哪一样不是这样，定位就错了。
#[test]
fn c694_falsifier_zeroing_the_account_units_of_an_unrefused_start() {
    let start = abandon_start(HistoryDeviceWidth::UnitAreaOf384Slots, 3, 0).expect("起点造得出");
    let image = start.pool.image();
    let root = abandoned_root_in_the_ring(&image, start.abandoned).expect("被抛弃的根在环里");
    let tree_table = root.tree_table.locations[0].slot.0;
    let mapping_root = root.mapping_root.locations[0].slot.0;
    println!("FALSIFIER abandoned={:?} tree_table_slot={tree_table} mapping_root_slot={mapping_root}", start.abandoned);
    println!("FALSIFIER baseline unmount={}", short(&unmount_on_a_copy(&start.pool)));
    println!("FALSIFIER zero tree_table unmount={}", short(&unmount_after_zeroing(&start.pool, &[tree_table])));
    println!("FALSIFIER zero mapping_root unmount={}", short(&unmount_after_zeroing(&start.pool, &[mapping_root])));
    println!("FALSIFIER zero tree_table+mapping_root unmount={}", short(&unmount_after_zeroing(&start.pool, &[tree_table, mapping_root])));
}

// ===== 以下是正推腿加的探针（m3-c694-r1）：卸载 / 再可写挂载 / 会话覆盖写在被抛弃的根上的结局，前后各跑一遍池级 checker =====

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;

fn red_names(image: &MemoryPool) -> String {
    let mut names: Vec<String> = check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(name.to_string()),
            _ => None,
        })
        .collect();
    names.sort();
    names.dedup();
    format!("[{}]", names.join(","))
}

fn unmount_on_the_real_devices(start: &mut Abandoned) -> Result<Unmounted, MountError> {
    let pool = &mut start.pool;
    let session = pool.session.as_mut().expect("会话");
    let mut allocator = session.allocator.clone();
    let mut current = session.current.clone();
    unmount(&pool.parameters, &mut pool.devices, &mut allocator, &mut current, session.shadow_ledger)
}

fn remount_then_unmount(image: &MemoryPool, parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters) -> String {
    let mut devices = plain_devices_on(image);
    match mount_writable_with_space_admission(parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(mounted) => {
            let (output, mut session) = MountedSession::of_the_mount(mounted);
            let unmounted = unmount(parameters, &mut devices, &mut session.allocator, &mut session.current, session.shadow_ledger);
            format!("Ok abandoned_unreadable={} then_unmount={}", output.abandoned_roots_unreadable, short(&unmounted))
        }
        Err(error) => format!("Err {error:?}").chars().take(160).collect(),
    }
}

/// 每格：造起点；起点盘面上池级 checker 的红名；正常卸载（真盘）；卸载之后盘面的红名；卸载之后再可写挂载加卸载。
/// 另造一份同一起点，会话里接着覆盖写最多 `SESSION_WRITES` 次，数第一次被拒是第几次、被抛弃的根是否离开根环、最后盘面的红名。
const SESSION_WRITES: u64 = 40;

#[test]
fn m3c694_probe_after_the_abandoning_mount() {
    let only: Option<Vec<u64>> = std::env::var("C694_WIDTHS").ok()
        .map(|value| value.split(',').map(|code| code.parse().expect("十进制")).collect());
    let ks: Vec<u64> = std::env::var("C694_KS").ok()
        .map(|value| value.split(',').map(|code| code.parse().expect("十进制")).collect())
        .unwrap_or_else(|| vec![1, 2, 3, 5, 8]);
    for (code, width) in widths().into_iter().filter(|(code, _)| *code != 4 && only.as_ref().is_none_or(|only| only.contains(code))) {
        for &k in &ks {
            for k2 in [0_u64, 1, 2, 4] {
                let label = format!("W={code} k={k} k2={k2}");
                let Ok(mut start) = abandon_start(width, k, k2) else {
                    println!("PROBE {label} START_FAILED");
                    continue;
                };
                let before = red_names(&start.pool.image());
                let in_ring_before = abandoned_root_still_in_the_ring(&start.pool.image(), start.abandoned);
                let outcome = unmount_on_the_real_devices(&mut start);
                let after = red_names(&start.pool.image());
                let remount = remount_then_unmount(&start.pool.image(), &start.pool.parameters);
                println!("PROBE {label} abandoned={:?} in_ring_at_start={in_ring_before} reds_at_start={before} unmount={} reds_after_unmount={after} remount={}",
                    start.abandoned, short(&outcome), remount.chars().take(150).collect::<String>());
                // 会话路径
                let Ok(mut start) = abandon_start(width, k, k2) else { continue };
                let mut first_refused: Option<(u64, String)> = None;
                let mut done = 0;
                for publish in 1..=SESSION_WRITES {
                    match start.pool.overwrite(OVERWRITE_BYTES) {
                        Ok(_) => done = publish,
                        Err(error) => {
                            first_refused = Some((publish, format!("{error:?}").chars().take(110).collect()));
                            break;
                        }
                    }
                }
                let in_ring = abandoned_root_still_in_the_ring(&start.pool.image(), start.abandoned);
                println!("PROBE_SESSION {label} writes_done={done} first_refused={first_refused:?} abandoned_still_in_ring={in_ring} txg={} reds_at_end={}",
                    start.pool.session().current.root().checkpoint_txg.0, red_names(&start.pool.image()));
            }
        }
    }
}

/// 崩溃那一刻的盘面上（取新实例那次挂载之前）journal 里水位之上的记录点名了哪些槽：被抛弃的根的树表、映射根、点名单元在不在其中。
#[test]
fn m3c694_probe_journal_records_above_the_chosen_root_at_the_abandoning_mount() {
    use singlefs_core::recovery::{scan_journal, choose_root};
    for (code, width) in widths().into_iter().filter(|(code, _)| *code != 4) {
        for k in [1_u64, 2, 3, 5, 8] {
            let label = format!("W={code} k={k}");
            let Ok(start) = abandon_start(width, k, 0) else { println!("JOURNAL {label} START_FAILED"); continue };
            // 取新实例那次挂载看见的盘面：被抛弃的根的根槽读回全 0
            let mut hidden_view = start.crashed.clone();
            {
                let configuration = choose_system_configuration(&hidden_view).expect("系统配置");
                let sizes = &configuration.immutable.sizes;
                for ring_slot in every_root_ring_slot(sizes) {
                    if let RootRingSlotReading::SelfVerified(root) = read_root_ring_slot(&hidden_view, &configuration.immutable.region_devices, sizes, &configuration.immutable.filesystem_identifier, ring_slot) {
                        if (root.instance, root.checkpoint_txg) == start.abandoned {
                            let device = configuration.immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
                            let offset = singlefs_core::root_ring::slot_offset(ring_slot, sizes.fixed_structure_slot_spacing);
                            hidden_view.devices.get_mut(&device).expect("盘").zero_fill(offset, sizes.physical_block_size.into());
                        }
                    }
                }
            }
            let crashed = &hidden_view;
            let configuration = choose_system_configuration(crashed).expect("系统配置");
            let chosen = choose_root(crashed, &configuration).expect("择根");
            let records = scan_journal(crashed, &configuration);
            let abandoned_root = abandoned_root_in_the_ring(&start.crashed, start.abandoned);
            let (tree_table_slot, mapping_root_slot) = match &abandoned_root {
                Some(root) => (root.tree_table.locations[0].slot.0, root.mapping_root.locations[0].slot.0),
                None => (u64::MAX, u64::MAX),
            };
            // 条目：水位之上（(实例, txg) 大于所选根）的记录
            let above: Vec<_> = records.values().filter(|r| (r.checkpoint_txg.0, r.instance.0) > (chosen.checkpoint_txg.0, chosen.instance.0)).collect();
            let names_tree_table = above.iter().any(|r| r.new_tree_table.locations[0].slot.0 == tree_table_slot);
            let names_mapping_root = above.iter().any(|r| r.new_mapping_root.locations[0].slot.0 == mapping_root_slot);
            println!("JOURNAL {label} chosen_txg={} abandoned_txg={} records_total={} records_above_water={} above_txgs={:?} names_tree_table_slot={names_tree_table} names_mapping_root_slot={names_mapping_root} named_units_in_above={}",
                chosen.checkpoint_txg.0, start.abandoned.1 .0, records.len(), above.len(),
                above.iter().map(|r| (r.checkpoint_txg.0, r.counter, r.is_commit)).collect::<Vec<_>>(),
                above.iter().map(|r| r.named.len()).sum::<usize>());
        }
    }
}

// ===== 候选开关（repo-sw）下的场景 =====
use singlefs_core::mount::candidate_switches;
use std::sync::atomic::Ordering;

fn set_switches(w1: bool, w2: bool, w4: bool) {
    candidate_switches::W1.store(w1, Ordering::SeqCst);
    candidate_switches::W2.store(w2, Ordering::SeqCst);
    candidate_switches::W4.store(w4, Ordering::SeqCst);
}

/// 一格一开关的全网格：与 `m3c694_probe_after_the_abandoning_mount` 同一份探针，开关由环境变量 `C694_SWITCHES`（逗号分隔：w1、w2、w4）定。
#[test]
fn m3c694_switched_probe_after_the_abandoning_mount() {
    let names = std::env::var("C694_SWITCHES").unwrap_or_default();
    set_switches(names.split(',').any(|n| n == "w1"), names.split(',').any(|n| n == "w2"), names.split(',').any(|n| n == "w4"));
    println!("SWITCHES {names}");
    m3c694_probe_after_the_abandoning_mount();
}

/// 丙（W2，读不出一律放行）的一形：被抛弃的根的树表与映射根只是在这次挂载期间读不出（故障撤了就读得出），单元没被写掉。
/// 造法：W4 开着造起点（被抛弃根的单元完好），再把那两个单元的槽（两块盘）写零 = 这次挂载期间读不出；挂载之后会话覆盖写 `writes` 次；
/// 然后「故障撤了」：没被会话写过的槽（仍全零）照原字节还原，被写过的留着。数被抛弃根的单元有几个被会话写掉，池级 checker 红哪几条。
fn temporarily_unreadable_account(w1: bool, w2: bool, writes: u64) -> String {
    set_switches(false, false, true);
    let Ok(mut start) = abandon_start(HistoryDeviceWidth::UnitAreaOf384Slots, 5, 0) else { return "START_FAILED".to_string() };
    set_switches(w1, w2, false);
    let image = start.pool.image();
    let root = abandoned_root_in_the_ring(&image, start.abandoned).expect("被抛弃的根在环里");
    let mut saved: Vec<(DeviceIdentity, u64, Vec<u8>)> = Vec::new();
    for pointer in [&root.tree_table, &root.mapping_root] {
        for location in &pointer.locations {
            let bytes = PoolReader::read(&image, location.device, DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES), SLOT_BYTES as usize).expect("读得出");
            saved.push((location.device, location.slot.0, bytes));
        }
    }
    let reds_before = red_names(&image);
    // 故障：两个单元的槽读回全 0
    start.pool.session = None;
    for (device, slot, _) in &saved {
        let (_, disk) = start.pool.devices.iter_mut().find(|(identity, _)| identity == device).expect("盘");
        disk.write_zeroes_at(DeviceOffsetInBytes(slot * SLOT_BYTES), SLOT_BYTES).expect("写零");
    }
    let mount = mount_writable_with_space_admission(&start.pool.parameters, &mut start.pool.devices, SpaceAdmission::JudgedByTheFormula);
    let (mount_text, done) = match mount {
        Err(error) => (format!("Err {error:?}").chars().take(120).collect::<String>(), 0),
        Ok(mounted) => {
            let (output, session) = MountedSession::of_the_mount(mounted);
            start.pool.session = Some(session);
            let mut done = 0;
            for _ in 0..writes {
                if start.pool.overwrite(OVERWRITE_BYTES).is_err() { break; }
                done += 1;
            }
            (format!("Ok abandoned_unreadable={}", output.abandoned_roots_unreadable), done)
        }
    };
    // 故障撤了：仍全零的槽还原
    let mut overwritten_by_the_session = 0;
    for (device, slot, original) in &saved {
        let now = PoolReader::read(&start.pool.image(), *device, DeviceOffsetInBytes(slot * SLOT_BYTES), SLOT_BYTES as usize).expect("读得出");
        if now.iter().all(|byte| *byte == 0) {
            let (_, disk) = start.pool.devices.iter_mut().find(|(identity, _)| identity == device).expect("盘");
            disk.write_at(DeviceOffsetInBytes(slot * SLOT_BYTES), original, WriteDurability::Plain).expect("还原");
        } else {
            overwritten_by_the_session += 1;
        }
    }
    format!("W1={w1} W2={w2} reds_before_the_fault={reds_before} mount={mount_text} session_writes_done={done} account_unit_copies_overwritten_by_the_session={overwritten_by_the_session}/{} reds_after_the_fault_is_gone={}",
        saved.len(), red_names(&start.pool.image()))
}

#[test]
fn m3c694_a_temporarily_unreadable_account_under_each_candidate() {
    for writes in [1_u64, 3, 6, 20] {
        for (w1, w2) in [(false, false), (true, false), (false, true)] {
            println!("TEMPORARY writes={writes} {}", temporarily_unreadable_account(w1, w2, writes));
        }
    }
    set_switches(false, false, false);
}
