//! m3-unlink-code-r3 攻方第二次抽样（s2）的原型：N1（丙1 卸载回收的槽立即可用）、N2（续做写账）、N7（模型分批与映射树上界）。
//! 只在冻结副本的拷贝里编（`patch-tree.py` 加工作区成员与一个只读可见性口子），两臂各编一份：c1 = 原样，held = mount.rs 第 2259 行改回。
//! 崩溃状态取层 0 的枚举域（`enumerate_layer0_selecting_versions_observing_each_state`，每一段都展开）。
#![allow(dead_code, unused_imports, unused_variables, clippy::all)]

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use common_admission::*;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites, Layer0Tally,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InodeNumber, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches,
    probe_s2_placements_referenced_by_root, unmount, DeletionQueueContinuationOutcome, MountError,
    ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, recover, walk_to_file, JournalPolicy,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
    MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordedPublishEntry, RecordingBlockDevice, SharedStream};
use std::collections::{BTreeMap, BTreeSet};

/// 臂名：编译时由 PROBE_ARM 环境变量写进来（rerun.sh 两臂各编一份）。
const ARM: &str = match option_env!("PROBE_ARM") {
    Some(arm) => arm,
    None => "unset",
};

fn batches_of(size: u64) -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(
            UnlinkBatchDataUnitsCappedForTests::new(size).expect("1..=144"),
        ),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 用户决定的那几步（N1 放开扫的维度）。
#[derive(Clone, Copy, Debug)]
struct Shape {
    /// 0 = X 那一族（窄盘、准入关着造、崩在两批删除之间、重挂续做）；1 = 普通小盘（崩了再挂、覆盖写、可选 unlink）。
    family: u8,
    unit_area_slots: u64,
    overwrites: usize,
    data_units: usize,
    /// family 0：意图之后又发几批；family 1：0 = 不 unlink，1 = 卸载之前经会话 unlink 第一个文件。
    batches: usize,
}

/// 卸载之前那一刻：池（镜像）、会话、参数、几何、每条根的内容。
struct BeforeUnmount {
    base: MemoryPool,
    session: MountedSession,
    parameters: singlefs_core::make_filesystem::MakeFilesystemParameters,
    versions: Vec<PublishedVersion>,
    note: String,
}

fn width_of(unit_area_slots: u64) -> HistoryDeviceWidth {
    match unit_area_slots {
        384 => HistoryDeviceWidth::UnitAreaOf384Slots,
        256 => HistoryDeviceWidth::UnitAreaOf256Slots,
        240 => HistoryDeviceWidth::UnitAreaOf240Slots,
        other => panic!("family 1 只认 384 / 256 / 240 槽：{other}"),
    }
}

/// 盘上每条读得出的根各自引用的落点（按账取的与按映射现算的并起来），与它读回的文件内容。在卸载之前、未被写过的镜像上算。
struct RootsOfTheBase {
    roots: BTreeMap<(u32, u64), RootRecord>,
    placements: BTreeMap<(u32, u64), BTreeSet<(u32, u64, u64)>>,
    versions: Vec<PublishedVersion>,
}

fn devices_on(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    plain_devices_on(image)
}

fn roots_of_the_base(base: &MemoryPool) -> RootsOfTheBase {
    let system_configuration = choose_system_configuration(base).expect("基镜像的系统配置");
    let roots = readable_roots(
        base,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let devices = devices_on(base);
    let mut out = RootsOfTheBase {
        roots: BTreeMap::new(),
        placements: BTreeMap::new(),
        versions: Vec::new(),
    };
    for root in roots {
        let key = (root.instance.0, root.checkpoint_txg.0);
        let (by_ledger, by_mapping) = probe_s2_placements_referenced_by_root(&devices, &root);
        let mut set = BTreeSet::new();
        for list in [by_ledger, by_mapping].into_iter().flatten() {
            for (device, slot, span) in list {
                set.insert((device.0, slot.0, span));
            }
        }
        let mut fallbacks = 0;
        if let Ok(Some(content)) = walk_to_file(base, &root, &mut fallbacks) {
            out.versions.push(PublishedVersion {
                instance: root.instance,
                checkpoint_txg: root.checkpoint_txg,
                content,
            });
        }
        out.placements.insert(key, set);
        out.roots.insert(key, root);
    }
    out
}

/// family 0：X 那一族（窄盘上崩在两批删除之间，重挂续做）。family 1：普通小盘。交回卸载之前那一刻。
fn before_unmount(shape: Shape) -> Result<BeforeUnmount, String> {
    let mut pool = if shape.family == 0 || shape.family == 2 {
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            shape.unit_area_slots,
        )
    } else {
        let mut pool = start_plain(width_of(shape.unit_area_slots));
        pool.crash_and_mount_writable()
            .map_err(|error| format!("崩了再挂：{error:?}"))?;
        pool
    };
    for index in 1..=shape.overwrites {
        pool.overwrite(OVERWRITE_BYTES)
            .map_err(|refusal| format!("第 {index} 次覆盖写：{refusal:?}"))?;
    }
    if shape.data_units > 1 {
        pool.sequential_write((shape.data_units - 1) * data_unit_payload_capacity() + 100)
            .map_err(|refusal| format!("顺序写：{refusal:?}"))?;
    }
    if shape.family == 2 {
        // family 2：窄盘（准入关着造）上覆盖写、顺序写，文件活着、不 unlink，不重挂，直接在这个进程的会话上卸载。
        return Ok(BeforeUnmount {
            base: pool.image(),
            session: pool.session.take().expect("会话"),
            parameters: pool.parameters.clone(),
            versions: Vec::new(),
            note: "窄盘活文件".into(),
        });
    }
    if shape.family == 0 {
        let PoolUnderTest {
            session,
            devices,
            parameters,
            ..
        } = &mut pool;
        let session = session.as_mut().expect("会话");
        let mut current = session
            .current
            .version_of_trees()
            .expect("有树")
            .clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            session.instance,
        )
        .map_err(|error| format!("意图：{error:?}"))?;
        for batch in 0..shape.batches {
            current = publish_unlink_step(
                &mut writer,
                &mut session.allocator,
                &current,
                InodeNumber(FIRST_INODE_NUMBER),
                batches_of(1).batch_size,
                session.instance,
            )
            .map_err(|error| format!("第 {batch} 批：{error:?}"))?;
        }
        pool.session = None;
        let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
            &pool.parameters,
            &mut pool.devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            batches_of(1),
        )
        .map_err(|error| format!("重挂：{}", short(&format!("{error:?}"))))?;
        let (output, session) = MountedSession::of_the_mount(mounted);
        let note = format!(
            "续做={} 准入推={} 续做推={}",
            match &output.deletion_queue_continuation_outcome {
                DeletionQueueContinuationOutcome::WalkedToEmpty => "走到空",
                DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. } => "推满仍放不下",
            },
            output.space_admission.floor_raises().len(),
            output.floor_raises_pushed_by_the_continuation.len()
        );
        pool.session = Some(session);
        return Ok(BeforeUnmount {
            base: pool.image(),
            session: pool.session.take().expect("会话"),
            parameters: pool.parameters.clone(),
            versions: Vec::new(),
            note,
        });
    }
    let mut note = String::from("普通");
    if shape.batches == 1 {
        let outcome = pool.publish(
            UserChange::Unlink {
                inode: InodeNumber(FIRST_INODE_NUMBER),
            },
            None,
        );
        note = format!("unlink={}", if outcome.is_ok() { "做成" } else { "被拒" });
        if let Err(refusal) = outcome {
            return Err(format!("unlink：{}", short(&format!("{refusal:?}"))));
        }
    }
    Ok(BeforeUnmount {
        base: pool.image(),
        session: pool.session.take().expect("会话"),
        parameters: pool.parameters.clone(),
        versions: Vec::new(),
        note,
    })
}

fn short(text: &str) -> String {
    text.chars().take(160).collect()
}

/// 卸载那一串录下来：结局、写表、段、写表下标对应的录制流、这一串的概要。
struct RecordedUnmount {
    outcome: String,
    raised: bool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    unmount_roots: Vec<(u32, u64, u32)>,
    first_publish_units_on_reclaimed_slots: usize,
    reclaimed_slots: usize,
    /// 这一串每次空发布写的单元落在哪几槽（两盘同槽），按先后。
    unit_slots_of_each_publish: Vec<Vec<(u64, u64)>>,
}

fn record_unmount(before: &BeforeUnmount, geometry_width: HistoryDeviceWidth) -> RecordedUnmount {
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = before
        .base
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(
                before.base.device_size_in_bytes,
                singlefs_core::block_device::PhysicalBlockSizeInBytes(512),
            );
            device.image = sparse.clone();
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
            )
        })
        .collect();
    let mut allocator = before.session.allocator.clone();
    let mut current = before.session.current.clone();
    let result = stream.record_entry(RecordedPublishEntry::Unmount, || {
        unmount(
            &before.parameters,
            &mut devices,
            &mut allocator,
            &mut current,
            before.session.shadow_ledger,
        )
    });
    let (writes, segments, _) = writes_and_segments_with_stream_indexes_and_entries(
        &stream.retained_operations(),
        &stream.entry_spans(),
        &geometry_width.fixed_geometry(),
    );
    let mut recorded = RecordedUnmount {
        outcome: String::new(),
        raised: false,
        writes,
        segments,
        unmount_roots: Vec::new(),
        first_publish_units_on_reclaimed_slots: 0,
        reclaimed_slots: 0,
        unit_slots_of_each_publish: Vec::new(),
    };
    match &result {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            recorded.raised = true;
            recorded.outcome = format!("卸载做成 F={}", raised.rollback_floor.0);
            recorded.reclaimed_slots = raised.reclaimed.len();
            let first = &raised.publishes[0];
            recorded.first_publish_units_on_reclaimed_slots = first
                .units
                .iter()
                .filter(|unit| first.rewritten.contains(&unit.identity))
                .filter(|unit| raised.reclaimed.iter().any(|placement| placement.slot == unit.slot))
                .count();
            for publish in &raised.publishes {
                recorded
                    .unit_slots_of_each_publish
                    .push(publish.units.iter().filter(|unit| publish.rewritten.contains(&unit.identity)).map(|unit| (unit.slot.0, (unit.bytes.len() as u64).div_ceil(SLOT_BYTES))).collect());
                let device = recorded
                    .writes
                    .iter()
                    .find(|write| {
                        write.kind == StepKind::RootRecordFua
                            && singlefs_harness::memory_pool::root_identity_of_write(write).1
                                == publish.root.checkpoint_txg
                    })
                    .map_or(u32::MAX, |write| write.device.0);
                recorded.unmount_roots.push((
                    publish.root.instance.0,
                    publish.root.checkpoint_txg.0,
                    device,
                ));
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => {
            recorded.outcome = "卸载：没有文件那一版，一个字节不写".into();
        }
        Err(error) => recorded.outcome = format!("卸载被拒：{}", short(&format!("{error:?}"))),
    }
    recorded
}

/// 一个崩溃状态叠成的镜像（基线 + 持久了的写，按写表次序）。
fn materialize(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
        if *is_persisted {
            write
                .contents
                .apply_to(pool.devices.get_mut(&write.device).expect("盘"), write.offset);
        }
    }
    pool
}

fn without_device(pool: &MemoryPool, lost: u32) -> MemoryPool {
    MemoryPool {
        devices: pool
            .devices
            .iter()
            .filter(|(identity, _)| identity.0 != lost)
            .map(|(identity, device)| (*identity, device.clone()))
            .collect(),
        device_size_in_bytes: pool.device_size_in_bytes,
    }
}

fn violated_names(pool: &MemoryPool) -> (String, Vec<String>) {
    let mut names = Vec::new();
    let mut details = Vec::new();
    for (invariant, verdict) in check_pool_image(pool) {
        if let InvariantVerdict::Violated(detail) = verdict {
            names.push(invariant.to_string());
            details.push(format!("{invariant}: {detail}"));
        }
    }
    (names.join(","), details)
}

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::recovery::PoolReader;

/// 一条根在一份镜像上走读的结局，简写成一个串（比两份镜像上同一条根读出来的是不是同一个东西）。
fn walk_summary(reader: &MemoryPool, root: &RootRecord) -> String {
    let mut fallbacks = 0;
    match walk_to_file(reader, root, &mut fallbacks) {
        Ok(Some(content)) => {
            let sum: u64 = content.iter().enumerate().map(|(i, b)| (i as u64 + 1) * u64::from(*b)).sum();
            format!("file{}b#{sum}", content.len())
        }
        Ok(None) => "nofile".into(),
        Err(error) => format!("ERR:{}", short(&format!("{error:?}")).chars().take(60).collect::<String>()),
    }
}

/// 一个崩溃状态、丢掉 `lost` 那块盘（None = 两块都在）之后：择到哪条根、它在卸载之前的镜像上引用的单元有几个在这块（几块）幸存盘上被改了字节、
/// 走读那条根与卸载之前同一条根走读是否相同、池级 checker 红哪几条（`run_checker`）。交回 (类别串, 细节)。
fn evaluate_survivor(
    base: &MemoryPool,
    roots: &RootsOfTheBase,
    base_walks: &mut BTreeMap<(u32, u64), String>,
    materialized: &MemoryPool,
    lost: Option<u32>,
    run_checker: bool,
) -> (String, String) {
    let survivor = match lost {
        None => materialized.clone(),
        Some(device) => without_device(materialized, device),
    };
    let report = recover(&survivor, JournalPolicy::Consult);
    let Some((instance, txg)) = report.effective_root else {
        let red = if run_checker { violated_names(&survivor).0 } else { "-".into() };
        return (format!("lost={lost:?} chosen=none red=[{red}]"), String::new());
    };
    let key = (instance.0, txg.0);
    let survivors: BTreeSet<u32> = survivor.devices.keys().map(|identity| identity.0).collect();
    let (in_base, overwritten, walk_same) = match roots.roots.get(&key) {
        None => ("new".to_string(), Vec::new(), "n/a".to_string()),
        Some(root) => {
            let overwritten: Vec<(u32, u64, u64)> = roots.placements[&key]
                .iter()
                .filter(|(device, _, _)| survivors.contains(device))
                .filter(|(device, slot, span)| {
                    let offset = DeviceOffsetInBytes(slot * SLOT_BYTES);
                    let length = usize::try_from(span * SLOT_BYTES).expect("长度");
                    PoolReader::read(&survivor, DeviceIdentity(*device), offset, length)
                        != PoolReader::read(base, DeviceIdentity(*device), offset, length)
                })
                .copied()
                .collect();
            let before = base_walks
                .entry(key)
                .or_insert_with(|| walk_summary(base, root))
                .clone();
            let after = walk_summary(&survivor, root);
            let same = if before == after { "same".to_string() } else { format!("DIFF({before}->{after})") };
            ("base".to_string(), overwritten, same)
        }
    };
    let (red, details) = if run_checker { violated_names(&survivor) } else { ("-".into(), Vec::new()) };
    // 只读挂载（择根、施加前缀、打开挂载态）与打开第一个文件读整份：用户看得到的那一层。
    let read_only = match singlefs_core::mounted_read::mount_read_only(&survivor) {
        Err(error) => format!("ro=ERR:{}", variant_of(&format!("{error:?}"))),
        Ok(mounted) => {
            let opened = match mounted.mounted.open_file(&survivor, InodeNumber(FIRST_INODE_NUMBER)) {
                Err(error) => format!("open=ERR:{}", variant_of(&format!("{error:?}"))),
                Ok(file) => {
                    let size = file.inode_record().size;
                    match file.read_at(&survivor, singlefs_core::address::FileOffsetInBytes(0), size) {
                        Ok(read) => {
                            let sum: u64 = read.bytes.iter().enumerate().map(|(i, b)| (i as u64 + 1) * u64::from(*b)).sum();
                            format!("open=ok read=ok/{size}b#{sum}")
                        }
                        Err(error) => format!("open=ok read=ERR:{}", variant_of(&format!("{error:?}"))),
                    }
                }
            };
            format!("ro=ok@{} {opened}", mounted.effective_root.checkpoint_txg.0)
        }
    };
    let class = format!(
        "lost={lost:?} chosen_txg={} root={in_base} overwritten_units={} {read_only} walk={} red=[{red}]",
        txg.0,
        overwritten.len(),
        if walk_same == "same" || walk_same == "n/a" { walk_same.clone() } else { "DIFF".into() }
    );
    let detail = format!(
        "overwritten={overwritten:?} walk={walk_same} checker_details={details:?}"
    );
    (class, detail)
}

fn geometry_width(shape: Shape) -> HistoryDeviceWidth {
    if shape.family == 0 || shape.family == 2 {
        HistoryDeviceWidth::UnitAreaOf384Slots
    } else {
        width_of(shape.unit_area_slots)
    }
}

/// X 那一刀：第一段带单元写的段之前全持久、那一段里只有单元写持久（不撕裂），其余都没持久。
fn is_the_x_cut(writes: &[RetainedWrite], segments: &[Vec<usize>], image: &CrashImage<'_>) -> bool {
    let Some(unit_segment) = segments
        .iter()
        .position(|segment| segment.iter().any(|index| writes[*index].kind == StepKind::UnitWrite))
    else {
        return false;
    };
    let mut target = vec![false; image.persisted.len()];
    for segment in &segments[..unit_segment] {
        for index in segment {
            target[*index] = true;
        }
    }
    for index in &segments[unit_segment] {
        if writes[*index].kind == StepKind::UnitWrite {
            target[*index] = true;
        }
    }
    target == image.persisted
}

/// 跑一段历史：录卸载、算状态数、（不超过 `max_states` 时）层 0 全量枚举，逐状态三种丢盘各判一遍。
fn run_n1(shape: Shape, max_states: u64, run_checker: bool) {
    println!("== arm={ARM} shape={shape:?}");
    let before = match before_unmount(shape) {
        Ok(before) => before,
        Err(why) => {
            println!("SKIP 造不出卸载之前那一刻：{why}");
            return;
        }
    };
    let recorded = record_unmount(&before, geometry_width(shape));
    println!(
        "pre: {} current_txg={} unmount: {} roots(inst,txg,dev)={:?} reclaimed={} first_publish_units_on_reclaimed={} writes={} segments={:?}",
        before.note,
        before.session.current.root().checkpoint_txg.0,
        recorded.outcome,
        recorded.unmount_roots,
        recorded.reclaimed_slots,
        recorded.first_publish_units_on_reclaimed_slots,
        recorded.writes.len(),
        recorded.segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    let roots = roots_of_the_base(&before.base);
    let mut base_walks = BTreeMap::new();
    if !recorded.raised {
        for lost in [None, Some(0), Some(1)] {
            let (class, detail) =
                evaluate_survivor(&before.base, &roots, &mut base_walks, &before.base, lost, true);
            println!("NO-WRITE {class}\n  detail {detail}");
        }
        return;
    }
    let closed = closed_form_state_count(&recorded.segments);
    let states = layer0_state_count_with_torn_in_place_overwrites(
        &before.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    println!("states closed_form={closed} with_torn={states}");
    if states > max_states {
        println!("SKIP 状态数 {states} 超过这一次的上限 {max_states}");
        return;
    }
    let mut versions = roots.versions.clone();
    if let Some(current_root) = roots
        .roots
        .get(&(before.session.current.root().instance.0, before.session.current.root().checkpoint_txg.0))
    {
        let mut fallbacks = 0;
        if let Ok(Some(content)) = walk_to_file(&before.base, current_root, &mut fallbacks) {
            for (instance, txg, _) in &recorded.unmount_roots {
                versions.push(PublishedVersion {
                    instance: singlefs_core::address::InstanceGeneration(*instance),
                    checkpoint_txg: CheckpointTxg(*txg),
                    content: content.clone(),
                });
            }
        }
    }
    let judged = recorded
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("卸载那一串至少一次根槽写");
    let mut classes: BTreeMap<String, u64> = BTreeMap::new();
    let mut samples: BTreeMap<String, String> = BTreeMap::new();
    let started = std::time::Instant::now();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &before.base,
        &recorded.writes,
        &recorded.segments,
        judged,
        &versions,
        &|_, _| true,
        &mut |image, _report| {
            let materialized = materialize(image);
            let x_cut = is_the_x_cut(&recorded.writes, &recorded.segments, image);
            for lost in [None, Some(0), Some(1)] {
                let (class, detail) = evaluate_survivor(
                    &before.base,
                    &roots,
                    &mut base_walks,
                    &materialized,
                    lost,
                    run_checker && lost.is_some(),
                );
                if x_cut {
                    println!("X-CUT {class}\n  detail {detail}");
                }
                *classes.entry(class.clone()).or_insert(0) += 1;
                samples.entry(class).or_insert(detail);
            }
        },
    );
    println!(
        "tally states={} observed={} oracle_violations={} first={:?} checker_violated={:?} elapsed={:.1}s",
        tally.states,
        tally.observed_states,
        tally.violations,
        tally.first_violation,
        tally.checker_violated_states,
        started.elapsed().as_secs_f64()
    );
    for (class, count) in &classes {
        println!("CLASS {count:>7} {class}");
    }
    for (class, detail) in &samples {
        if class.contains("DIFF") || !class.contains("overwritten_units=0") {
            println!("SAMPLE {class}\n  {detail}");
        }
    }
}

fn numbers(text: &str) -> Vec<u64> {
    // "a-b" 或 "a,b,c"
    if let Some((low, high)) = text.split_once('-') {
        (low.parse().expect("数")..=high.parse().expect("数")).collect()
    } else {
        text.split(',').map(|part| part.parse().expect("数")).collect()
    }
}

/// 不枚举，只造历史与录卸载：每一格打一行（卸载结局、第一次空发布落在回收槽上的单元数、状态数），找哪几格回收的槽被这一串用上。
fn scan_n1(family: u8, slots: &[u64], overwrites: &[u64], units: &[u64], batches: &[u64]) {
    for &unit_area_slots in slots {
        for &overwrite_count in overwrites {
            for &data_units in units {
                for &batch_count in batches {
                    let shape = Shape {
                        family,
                        unit_area_slots,
                        overwrites: overwrite_count as usize,
                        data_units: data_units as usize,
                        batches: batch_count as usize,
                    };
                    let line = match before_unmount(shape) {
                        Err(why) => format!("SKIP {}", short(&why)),
                        Ok(before) => {
                            let recorded = record_unmount(&before, geometry_width(shape));
                            let states = if recorded.raised {
                                layer0_state_count_with_torn_in_place_overwrites(
                                    &before.base,
                                    &recorded.writes,
                                    &recorded.segments,
                                    &full_expansion,
                                )
                            } else {
                                0
                            };
                            let overlap = if recorded.raised {
                                static_overlap(&before.base, &roots_of_the_base(&before.base), &recorded)
                            } else {
                                "-".into()
                            };
                            format!(
                                "{} current_txg={} | {} | roots={:?} reclaimed={} first_on_reclaimed={} states={states} | {overlap}",
                                before.note,
                                before.session.current.root().checkpoint_txg.0,
                                recorded.outcome,
                                recorded.unmount_roots,
                                recorded.reclaimed_slots,
                                recorded.first_publish_units_on_reclaimed_slots
                            )
                        }
                    };
                    println!(
                        "SCAN arm={ARM} f={family} slots={unit_area_slots} ow={overwrite_count} units={data_units} b={batch_count} :: {line}"
                    );
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("n1") => {
            let shape = Shape {
                family: args[2].parse().expect("family"),
                unit_area_slots: args[3].parse().expect("slots"),
                overwrites: args[4].parse().expect("overwrites"),
                data_units: args[5].parse().expect("units"),
                batches: args[6].parse().expect("batches"),
            };
            let max_states = args.get(7).map_or(1_000_000, |text| text.parse().expect("max"));
            let run_checker = args.get(8).map_or(true, |text| text != "0");
            run_n1_parallel(shape, max_states, run_checker);
        }
        Some("n2") => {
            for &slots in &numbers(&args[2]) {
                for &overwrites in &numbers(&args[3]) {
                    for &units in &numbers(&args[4]) {
                        for &batches in &numbers(&args[5]) {
                            sweep_n2(Shape { family: 0, unit_area_slots: slots, overwrites: overwrites as usize, data_units: units as usize, batches: batches as usize });
                        }
                    }
                }
            }
        }
        Some("n1-debug") => debug_n1(Shape {
            family: args[2].parse().expect("family"),
            unit_area_slots: args[3].parse().expect("slots"),
            overwrites: args[4].parse().expect("overwrites"),
            data_units: args[5].parse().expect("units"),
            batches: args[6].parse().expect("batches"),
        }),
        Some("n1-scan") => scan_n1(
            args[2].parse().expect("family"),
            &numbers(&args[3]),
            &numbers(&args[4]),
            &numbers(&args[5]),
            &numbers(&args[6]),
        ),
        _ => eprintln!("用法：probe-s2 n1 <family> <slots> <ow> <units> <batches> [max_states] [checker] | n1-scan <family> <slots> <ow> <units> <batches>"),
    }
}

/// 同 [`run_n1`]，逐状态的丢盘判定挪出观察者：观察者只记下每个状态的持久集合（枚举器在调用线程上串行调它），
/// 枚举完之后按 PROBE_THREADS（默认 15）个线程并行判，状态一个不落。
fn run_n1_parallel(shape: Shape, max_states: u64, run_checker: bool) {
    println!("== arm={ARM} shape={shape:?} (parallel)");
    let before = match before_unmount(shape) {
        Ok(before) => before,
        Err(why) => {
            println!("SKIP 造不出卸载之前那一刻：{why}");
            return;
        }
    };
    let recorded = record_unmount(&before, geometry_width(shape));
    println!(
        "pre: {} current_txg={} unmount: {} roots(inst,txg,dev)={:?} reclaimed={} first_publish_units_on_reclaimed={} writes={} segments={:?}",
        before.note,
        before.session.current.root().checkpoint_txg.0,
        recorded.outcome,
        recorded.unmount_roots,
        recorded.reclaimed_slots,
        recorded.first_publish_units_on_reclaimed_slots,
        recorded.writes.len(),
        recorded.segments.iter().map(Vec::len).collect::<Vec<_>>()
    );
    let roots = roots_of_the_base(&before.base);
    if !recorded.raised {
        let mut base_walks = BTreeMap::new();
        for lost in [None, Some(0), Some(1)] {
            let (class, detail) =
                evaluate_survivor(&before.base, &roots, &mut base_walks, &before.base, lost, true);
            println!("NO-WRITE {class}\n  detail {detail}");
        }
        return;
    }
    let closed = closed_form_state_count(&recorded.segments);
    let states = layer0_state_count_with_torn_in_place_overwrites(
        &before.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    println!("states closed_form={closed} with_torn={states}");
    if states > max_states {
        println!("SKIP 状态数 {states} 超过这一次的上限 {max_states}");
        return;
    }
    let mut versions = roots.versions.clone();
    let current_key = (
        before.session.current.root().instance.0,
        before.session.current.root().checkpoint_txg.0,
    );
    if let Some(current_root) = roots.roots.get(&current_key) {
        let mut fallbacks = 0;
        if let Ok(Some(content)) = walk_to_file(&before.base, current_root, &mut fallbacks) {
            for (instance, txg, _) in &recorded.unmount_roots {
                versions.push(PublishedVersion {
                    instance: singlefs_core::address::InstanceGeneration(*instance),
                    checkpoint_txg: CheckpointTxg(*txg),
                    content: content.clone(),
                });
            }
        }
    }
    let judged = recorded
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("卸载那一串至少一次根槽写");
    let mut table: Option<Vec<RetainedWrite>> = None;
    let mut collected: Vec<(Vec<bool>, bool)> = Vec::new();
    let started = std::time::Instant::now();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &before.base,
        &recorded.writes,
        &recorded.segments,
        judged,
        &versions,
        &|_, _| true,
        &mut |image, _report| {
            if table.is_none() {
                table = Some(image.writes.to_vec());
            }
            let x_cut = is_the_x_cut(&recorded.writes, &recorded.segments, image)
                || prefix_cut_before_the_first_root_on(&recorded.writes, image, 0)
                || prefix_cut_before_the_first_root_on(&recorded.writes, image, 1);
            collected.push((image.persisted.clone(), x_cut));
        },
    );
    println!(
        "tally states={} observed={} oracle_violations={} first={:?} checker_violated={:?} enum_elapsed={:.1}s",
        tally.states,
        tally.observed_states,
        tally.violations,
        tally.first_violation,
        tally.checker_violated_states,
        started.elapsed().as_secs_f64()
    );
    let table = table.expect("至少一个状态");
    let threads: usize = std::env::var("PROBE_THREADS").ok().and_then(|text| text.parse().ok()).unwrap_or(15);
    let chunk = collected.len().div_ceil(threads).max(1);
    let results: Vec<Vec<(String, String, bool)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = collected
            .chunks(chunk)
            .map(|states_of_this_thread| {
                let base = &before.base;
                let roots = &roots;
                let table = &table;
                scope.spawn(move || {
                    let mut base_walks = BTreeMap::new();
                    let mut out = Vec::new();
                    for (persisted, x_cut) in states_of_this_thread {
                        let image = CrashImage { base, writes: table, persisted: persisted.clone() };
                        let materialized = materialize(&image);
                        for lost in [None, Some(0), Some(1)] {
                            let (class, detail) = evaluate_survivor(
                                base, roots, &mut base_walks, &materialized, lost,
                                run_checker && lost.is_some(),
                            );
                            out.push((class, detail, *x_cut));
                        }
                    }
                    out
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("线程")).collect()
    });
    let mut classes: BTreeMap<String, u64> = BTreeMap::new();
    let mut samples: BTreeMap<String, String> = BTreeMap::new();
    let mut judged_states = 0u64;
    for (class, detail, x_cut) in results.into_iter().flatten() {
        judged_states += 1;
        if x_cut {
            println!("X-CUT {class}\n  detail {detail}");
        }
        *classes.entry(class.clone()).or_insert(0) += 1;
        samples.entry(class).or_insert(detail);
    }
    println!(
        "judged survivor-states={judged_states} (= 3 x {}) total_elapsed={:.1}s",
        collected.len(),
        started.elapsed().as_secs_f64()
    );
    for (class, count) in &classes {
        println!("CLASS {count:>7} {class}");
    }
    for (class, detail) in &samples {
        if class.contains("DIFF") || !class.contains("overwritten_units=0") {
            println!("SAMPLE {class}\n  {detail}");
        }
    }
}

// ---------------- N2：续做写账，注错点放开扫 ----------------

use singlefs_core::write_accounting::{WriteCallsAndBytes, WritesByStructureKind};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::RecordedOperationKind;

type FaultDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

fn calls(ledgers: &[&WritesByStructureKind]) -> u64 {
    ledgers
        .iter()
        .fold(WriteCallsAndBytes::NONE, |sum, ledger| sum.plus(ledger.total()))
        .write_calls
}

fn landed(stream: &SharedStream) -> (u64, u64) {
    let operations = stream.operations();
    let count = |kinds: &[RecordedOperationKind]| {
        operations.iter().filter(|operation| kinds.contains(&operation.kind)).count() as u64
    };
    (
        count(&[
            RecordedOperationKind::Write,
            RecordedOperationKind::WriteForceUnitAccess,
            RecordedOperationKind::WriteZeroes,
        ]),
        count(&[RecordedOperationKind::WriteForceUnitAccess]),
    )
}

/// 窄盘上崩在两批删除之间的镜像（family 0 的前半，不挂）。
fn crashed_image(shape: Shape) -> Result<(MemoryPool, singlefs_core::make_filesystem::MakeFilesystemParameters), String> {
    let mut pool = start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
        HistoryDeviceWidth::UnitAreaOf384Slots,
        shape.unit_area_slots,
    );
    for index in 1..=shape.overwrites {
        pool.overwrite(OVERWRITE_BYTES).map_err(|refusal| format!("覆盖写 {index}：{refusal:?}"))?;
    }
    if shape.data_units > 1 {
        pool.sequential_write((shape.data_units - 1) * data_unit_payload_capacity() + 100)
            .map_err(|refusal| format!("顺序写：{refusal:?}"))?;
    }
    let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
    let session = session.as_mut().expect("会话");
    let mut current = session.current.version_of_trees().expect("有树").clone();
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), session.instance)
        .map_err(|error| format!("意图：{error:?}"))?;
    for batch in 0..shape.batches {
        current = publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), batches_of(1).batch_size, session.instance)
            .map_err(|error| format!("第 {batch} 批：{error:?}"))?;
    }
    pool.session = None;
    Ok((pool.image(), pool.parameters.clone()))
}

fn mount_on(
    image: &MemoryPool,
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    schedule: Option<FaultSchedule>,
) -> (Result<singlefs_core::mount::Mounted, MountError>, SharedStream, SharedFaultPlan) {
    let geometry = HistoryDeviceWidth::UnitAreaOf384Slots.fixed_geometry();
    let plan = match schedule {
        Some(schedule) => SharedFaultPlan::armed(geometry, schedule),
        None => SharedFaultPlan::unarmed(geometry),
    };
    let plan_handle = plan.clone();
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, FaultDevice)> = plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (identity, FaultInjectingBlockDevice::new(identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()), plan.clone()))
        })
        .collect();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, batches_of(1),
    );
    (mounted, stream, plan_handle)
}

/// 错里按契约文字交得出的写账：(成员, 按写行那一处的口径合计, 按续做那一处的口径合计, 写行口径的已落盘份数, 续做口径的已落盘份数)。
/// 只有 FloorRaiseFailedAfterTheMountsPublishes 两种口径不同（结构文档：写行之后那一处另加各串的各次空发布，续做那一处不另加）。
fn ledger_of(error: &MountError) -> Option<(&'static str, u64, u64, u64, u64)> {
    match error {
        MountError::Publish(failed) => {
            let mut ledgers: Vec<&WritesByStructureKind> = vec![&failed.writes_before_the_first_publish];
            ledgers.extend(failed.writes_of_persisted_publishes.iter());
            ledgers.extend(failed.writes_of_failed_publishes.iter());
            let total = calls(&ledgers);
            let persisted = failed.writes_of_persisted_publishes.len() as u64;
            Some(("Publish", total, total, persisted, persisted))
        }
        MountError::FloorRaiseFailedAfterTheMountsPublishes(failed) => {
            let persisted_ledgers: Vec<&WritesByStructureKind> = failed.writes_of_persisted_publishes.iter().collect();
            let system_configuration: Vec<&WritesByStructureKind> = failed
                .floor_raises
                .iter()
                .map(|raised| &raised.system_configuration_writes_before_the_first_publish)
                .collect();
            let raise_publishes: Vec<&WritesByStructureKind> = failed
                .floor_raises
                .iter()
                .flat_map(|raised| raised.publishes.iter().map(|publish| &publish.writes))
                .collect();
            let (cause_total, cause_persisted, cause_name) = match &failed.cause {
                MountError::RaiseFloorSequencePublishFailed(inner) => {
                    let mut ledgers: Vec<&WritesByStructureKind> = vec![&inner.writes_before_the_first_publish];
                    ledgers.extend(inner.writes_of_persisted_publishes.iter());
                    ledgers.extend(inner.writes_of_failed_publishes.iter());
                    (calls(&ledgers), inner.writes_of_persisted_publishes.len() as u64, "FRF/SequencePublishFailed")
                }
                MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(inner) => {
                    (inner.writes.total().write_calls, 0, "FRF/SystemConfigurationWriteFailed")
                }
                _ => (0, 0, "FRF/other-cause"),
            };
            let base = calls(&persisted_ledgers) + calls(&system_configuration) + cause_total;
            let row_total = base + calls(&raise_publishes);
            let row_persisted = persisted_ledgers.len() as u64 + raise_publishes.len() as u64 + cause_persisted;
            let continuation_persisted = persisted_ledgers.len() as u64 + cause_persisted;
            Some((cause_name, row_total, base, row_persisted, continuation_persisted))
        }
        MountError::RaiseFloorSequencePublishFailed(inner) => {
            let mut ledgers: Vec<&WritesByStructureKind> = vec![&inner.writes_before_the_first_publish];
            ledgers.extend(inner.writes_of_persisted_publishes.iter());
            ledgers.extend(inner.writes_of_failed_publishes.iter());
            let total = calls(&ledgers);
            let persisted = inner.writes_of_persisted_publishes.len() as u64;
            Some(("RaiseFloorSequencePublishFailed(bare)", total, total, persisted, persisted))
        }
        _ => None,
    }
}

/// 不注错那一次挂载里，续做开始之前一共交给设备几个写（取号两次 + 写行 + 暖机 + 准入推的各串）。
fn writes_before_the_continuation(output: &singlefs_core::mount::MountOutput) -> u64 {
    let mut ledgers: Vec<&WritesByStructureKind> = vec![output.row_publish.writes()];
    ledgers.extend(output.warm_up_publishes.iter().map(|version| version.writes()));
    for raised in output.space_admission.floor_raises() {
        ledgers.push(&raised.system_configuration_writes_before_the_first_publish);
        ledgers.extend(raised.publishes.iter().map(|publish| &publish.writes));
    }
    2 + calls(&ledgers)
}

/// 挂载做成时输出里交得出的写账合计（连取号两次）：写行、暖机、准入推的各串、续做的各步、续做推的各串。
fn ledger_of_a_mount_that_succeeded(output: &singlefs_core::mount::MountOutput) -> u64 {
    let mut ledgers: Vec<&WritesByStructureKind> = vec![output.row_publish.writes()];
    ledgers.extend(output.warm_up_publishes.iter().map(|version| version.writes()));
    for raised in output.space_admission.floor_raises().iter().chain(&output.floor_raises_pushed_by_the_continuation) {
        ledgers.push(&raised.system_configuration_writes_before_the_first_publish);
        ledgers.extend(raised.publishes.iter().map(|publish| &publish.writes));
    }
    ledgers.extend(output.deletion_queue_continuation.iter().map(|publish| &publish.writes));
    2 + calls(&ledgers)
}

fn variant_name(error: &MountError) -> String {
    let text = format!("{error:?}");
    text.split(|c: char| c == '(' || c == ' ' || c == '{').next().unwrap_or("").to_string()
}

fn sweep_n2(shape: Shape) {
    println!("== N2 arm={ARM} shape={shape:?}");
    let (image, parameters) = match crashed_image(shape) {
        Ok(pair) => pair,
        Err(why) => {
            println!("SKIP {why}");
            return;
        }
    };
    let (baseline, stream, _) = mount_on(&image, &parameters, None);
    let baseline = match baseline {
        Ok(mounted) => mounted,
        Err(error) => {
            println!("SKIP 不注错的挂载被拒：{}", short(&format!("{error:?}")));
            return;
        }
    };
    let (baseline_writes, baseline_roots) = landed(&stream);
    let barriers = stream.operations().iter().filter(|operation| operation.kind == RecordedOperationKind::Barrier).count() as u64;
    let boundary = writes_before_the_continuation(&baseline.output);
    println!(
        "baseline writes={baseline_writes} roots={baseline_roots} barriers_recorded={barriers} writes_before_continuation={boundary} ok_ledger={} admission_raises={} continuation_raises={} continuation_steps={} outcome={}",
        ledger_of_a_mount_that_succeeded(&baseline.output),
        baseline.output.space_admission.floor_raises().len(),
        baseline.output.floor_raises_pushed_by_the_continuation.len(),
        baseline.output.deletion_queue_continuation.len(),
        match baseline.output.deletion_queue_continuation_outcome {
            DeletionQueueContinuationOutcome::WalkedToEmpty => "走到空",
            DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. } => "推满仍放不下",
        }
    );
    let mut classes: BTreeMap<String, u64> = BTreeMap::new();
    let mut shown = 0;
    let modes: [(&str, InjectedFault, bool); 4] = [
        ("write-only", InjectedFault::WriteFails, false),
        ("write-from", InjectedFault::WriteFails, true),
        ("barrier-only", InjectedFault::BarrierFails, false),
        ("barrier-from", InjectedFault::BarrierFails, true),
    ];
    for (mode, fault, from_on) in modes {
        let limit = if matches!(fault, InjectedFault::WriteFails) { baseline_writes } else { barriers * 3 + 10 };
        for ordinal in 1..=limit {
            let schedule = FaultSchedule {
                fault,
                device: FaultDeviceSelector::EveryDevice,
                placement: FaultPlacement::AnyOffset,
                counting: FaultCounting::AcrossThePool,
                occurrence: if from_on {
                    FaultOccurrence::EveryMatchingCallFromTheNthOnward(ordinal)
                } else {
                    FaultOccurrence::TheNthMatchingCall(ordinal)
                },
            };
            let (result, stream, plan) = mount_on(&image, &parameters, Some(schedule));
            let fired = plan.fired_count();
            let (writes, roots) = landed(&stream);
            let phase = if matches!(fault, InjectedFault::WriteFails) {
                if ordinal > boundary { "cont" } else { "row" }
            } else {
                "?"
            };
            let class = match &result {
                Ok(mounted) => {
                    let ok_ledger = ledger_of_a_mount_that_succeeded(&mounted.output);
                    if fired == 0 {
                        format!("{mode} Ok(fault-not-hit)")
                    } else {
                        format!(
                            "{mode} Ok AFTER-FIRED({fired}) same_writes_as_baseline={} ledger_matches={}",
                            writes == baseline_writes,
                            ok_ledger + 0 == writes
                        )
                    }
                }
                Err(error) => match ledger_of(error) {
                    None => format!("{mode} Err {} (no ledger)", variant_name(error)),
                    Some((name, row_total, continuation_total, row_persisted, continuation_persisted)) => {
                        let which = |row: bool, continuation: bool| match (row, continuation) {
                            (true, true) => "both",
                            (true, false) => "row-rule",
                            (false, true) => "cont-rule",
                            (false, false) => "NONE",
                        };
                        let totals = which(row_total + 2 == writes, continuation_total + 2 == writes);
                        // 份数：根落了而那次发布报错（落在根之后的系统配置轮换写上）时份数比根少 1——第二轮 H4 / V4 的老口径，单列。
                        let persisted_counts = match which(row_persisted == roots, continuation_persisted == roots) {
                            "NONE" if row_persisted + 1 == roots || continuation_persisted + 1 == roots => "one-root-more(H4)",
                            other => other,
                        };
                        let verdict_text = format!("totals={totals} persisted={persisted_counts}");
                        let verdict = if totals == "NONE" { "MISMATCH" } else { verdict_text.as_str() };
                        if verdict == "MISMATCH" && shown < 12 {
                            shown += 1;
                            println!(
                                "MISMATCH {mode} k={ordinal} phase={phase} {name} landed_writes={writes} landed_roots={roots} row_total+2={} cont_total+2={} row_persisted={row_persisted} cont_persisted={continuation_persisted} err={}",
                                row_total + 2, continuation_total + 2, short(&format!("{error:?}"))
                            );
                        }
                        format!("{mode} {name} phase={phase} {verdict_text}")
                    }
                },
            };
            *classes.entry(class).or_insert(0) += 1;
        }
    }
    for (class, count) in &classes {
        println!("N2CLASS {count:>5} {class}");
    }
}

/// 不枚举的静态判：丢掉一块盘、卸载那一串一个根都还没落时，幸存盘上恢复择到哪条根（直接在卸载之前的镜像上去掉那块盘恢复），
/// 它引用的落点（按账与按映射并起来）有几个与这一串第一次 / 全部空发布写的单元同槽。也报那条根引用了几个落点（0 说明口子没取到）。
fn static_overlap(base: &MemoryPool, roots: &RootsOfTheBase, recorded: &RecordedUnmount) -> String {
    let mut parts = Vec::new();
    for lost in [0u32, 1] {
        let survivor = without_device(base, lost);
        let report = recover(&survivor, JournalPolicy::Consult);
        let Some((instance, txg)) = report.effective_root else {
            parts.push(format!("lost{lost}:none"));
            continue;
        };
        let survivor_device = 1 - lost;
        let referenced: BTreeSet<(u64, u64)> = roots
            .placements
            .get(&(instance.0, txg.0))
            .map(|set| {
                set.iter()
                    .filter(|(device, _, _)| *device == survivor_device)
                    .map(|(_, slot, span)| (*slot, *span))
                    .collect()
            })
            .unwrap_or_default();
        let hits = |slots: &[(u64, u64)]| -> Vec<u64> {
            slots
                .iter()
                .copied()
                .filter(|(unit_slot, unit_span)| {
                    referenced
                        .iter()
                        .any(|(slot, span)| *unit_slot < slot + span && *slot < unit_slot + unit_span)
                })
                .map(|(unit_slot, _)| unit_slot)
                .collect()
        };
        // 窗口：这一串里幸存盘上落下第一条新根之前（含那一次发布）写的单元——那条根落盘之前，幸存盘单独恢复择到的是卸载之前它上面最新的那条根。
        let window_end = recorded
            .unmount_roots
            .iter()
            .position(|(_, _, device)| *device == survivor_device)
            .unwrap_or(recorded.unmount_roots.len().saturating_sub(1));
        let first: Vec<u64> = recorded.unit_slots_of_each_publish[..=window_end.min(recorded.unit_slots_of_each_publish.len().saturating_sub(1))]
            .iter()
            .flat_map(|slots| hits(slots))
            .collect();
        let all: Vec<u64> = recorded.unit_slots_of_each_publish.iter().flat_map(|slots| hits(slots)).collect();
        parts.push(format!(
            "lost{lost}:root{}/refs{}/hit_window{:?}/hit_all{}",
            txg.0,
            referenced.len(),
            first,
            all.len()
        ));
    }
    parts.join(" ")
}

/// 排查用：列出卸载之前两盘各自恢复择到的根引用的落点、这一串第一次空发布的单元（槽、跨度、角色），
/// 再在「整串都落盘」的镜像上逐个比那几个落点的字节。
fn debug_n1(shape: Shape) {
    let before = before_unmount(shape).expect("卸载之前那一刻");
    let recorded = record_unmount(&before, geometry_width(shape));
    let roots = roots_of_the_base(&before.base);
    let mut after = before.base.clone();
    after.apply_writes(&recorded.writes);
    println!("unmount: {} roots={:?}", recorded.outcome, recorded.unmount_roots);
    for (index, slots) in recorded.unit_slots_of_each_publish.iter().enumerate() {
        println!("publish#{index} units(slot,span)={slots:?}");
    }
    for (key, set) in &roots.placements {
        let changed: Vec<(u32, u64, u64)> = set
            .iter()
            .filter(|(device, slot, span)| {
                let offset = DeviceOffsetInBytes(slot * SLOT_BYTES);
                let length = usize::try_from(span * SLOT_BYTES).expect("长度");
                PoolReader::read(&after, DeviceIdentity(*device), offset, length)
                    != PoolReader::read(&before.base, DeviceIdentity(*device), offset, length)
            })
            .copied()
            .collect();
        println!(
            "root {key:?} refs={} changed_after_full_unmount={} changed={:?}",
            set.len(),
            changed.len(),
            changed
        );
    }
    let unit_writes: Vec<(u32, u64, u64)> = recorded
        .writes
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .map(|write| (write.device.0, write.offset.0 / SLOT_BYTES, write.length_in_bytes()))
        .collect();
    println!("recorded unit writes (dev, offset/SLOT_BYTES, bytes)={unit_writes:?}");
}

fn variant_of(debug: &str) -> String {
    debug
        .split(|c: char| c == '(' || c == ' ' || c == '{')
        .next()
        .unwrap_or("")
        .chars()
        .take(60)
        .collect()
}

/// 这一串里第一条落在盘 `device` 上的根槽写之前的写全持久、它与之后的都没持久的那个状态（那条根落盘之前的最后一刻；
/// 单独留下这块盘时，恢复择到的还是卸载之前它上面最新的根）。
fn prefix_cut_before_the_first_root_on(writes: &[RetainedWrite], image: &CrashImage<'_>, device: u32) -> bool {
    let Some(first_root) = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua && write.device.0 == device)
    else {
        return false;
    };
    image
        .persisted
        .iter()
        .enumerate()
        .all(|(index, persisted)| *persisted == (index < first_root))
}
