//! 调查员副本：只靠崩溃恢复造被抛弃实例，再把 F 抬进它留下的空档（增补 2 收口表第 43 行复查）。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
use singlefs_core::transaction::TransactionOutput;
use singlefs_harness::history::{
    classify_failure, raised_floor_lands_only_on_abandoned_roots,
    row43_probe_any_ring_root_abandoned, FailureObservation, HistoryEnding,
    HistoryOperationKind, StepPosition,
};
use singlefs_harness::memory_pool::{MemoryPool, RecordCheck};

fn content_seeded_by(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite(pool: &mut BuiltPool, content: &[u8], instance: InstanceGeneration) -> TransactionOutput {
    let previous = pool.output.clone();
    let output = publish_overwrite_in_process(pool, &previous, content, FIXED_WRITE_TIME_SECONDS + 60, instance)
        .expect("覆盖写");
    pool.output = output.clone();
    output
}

fn violations_on(image: &MemoryPool) -> Vec<(&'static str, String)> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
        })
        .collect()
}

fn not_applicable_on(image: &MemoryPool, wanted: &str) -> Option<String> {
    check_pool_image(image).into_iter().find_map(|(invariant, verdict)| match verdict {
        InvariantVerdict::NotApplicable(reason) if invariant == wanted => Some(reason.to_string()),
        _ => None,
    })
}

/// 系统配置槽的偏移：每块盘两槽，槽 k 在 k × 间距。
fn system_configuration_slot_offsets() -> Vec<u64> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    (0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).map(|slot| slot * spacing).collect()
}

/// 变体乙：第三版 C 的根槽 FUA 持久、系统配置轮换没持久就崩（C 的轮换写全部退回崩溃前的字节），下一次可写挂载时 C 的根槽与数据单元
/// 在重读一次之后仍读不出；挂载交回之后原样写回。系统配置不坏任何一槽。
fn abandon_the_third_version_by_a_crash_before_its_rotation(pool: &mut BuiltPool, third: &TransactionOutput, saved_system_configuration: &[(usize, u64, Vec<u8>)]) -> singlefs_core::mount::Mounted {
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(third.root.checkpoint_txg, publish_parameters.geometry.root_ring_slots_per_region);
    let root_slot_device = publish_parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(target, publish_parameters.geometry.fixed_structure_slot_spacing);
    let root_slot_bytes = usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let mut devices = pool.reopen_recorded();
    // 崩溃：C 的系统配置轮换没落盘。
    for (index, offset, bytes) in saved_system_configuration {
        devices[*index].1.write_at(singlefs_core::address::DeviceOffsetInBytes(*offset), bytes, WriteDurability::Plain).expect("退回轮换之前的字节");
    }
    let places_to_hide = std::iter::once((root_slot_device, root_slot_offset, root_slot_bytes)).chain(
        third.data_pointers.iter().flat_map(|pointer| pointer.locations).map(|location| (location.device, location.slot.to_device_offset(), data_unit_bytes)),
    );
    let mut saved = Vec::new();
    for (identity, offset, length) in places_to_hide {
        let index = devices.iter().position(|(candidate, _)| *candidate == identity).expect("池里有这块盘");
        let mut bytes = vec![0u8; length];
        devices[index].1.read_at(offset, &mut bytes).expect("暂存");
        devices[index].1.write_at(offset, &vec![0u8; length], WriteDurability::Plain).expect("清零");
        saved.push((index, offset, bytes));
    }
    let mounted = mount_writable(&publish_parameters, &mut devices);
    for (index, offset, bytes) in &saved {
        devices[*index].1.write_at(*offset, bytes, WriteDurability::Plain).expect("原样写回");
    }
    let mounted = mounted.expect("变体乙：可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted.current.file_version().expect("落到的那一版带文件").clone();
    mounted
}

#[derive(Clone, Copy, Debug)]
enum Variant {
    /// 变体甲：`tests/common` 的造法（见证 C 的系统配置槽每盘坏一槽，C583 那一形）。
    DoubleFault,
    /// 变体乙：C 的轮换没落盘就崩，加 C 的根槽与数据单元一时读不出。
    CrashBeforeRotation,
}

fn pool_after_abandoning(variant: Variant, tag: &str) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    overwrite(&mut pool, &content_seeded_by(4100, 3), InstanceGeneration(1));
    let saved_system_configuration: Vec<(usize, u64, Vec<u8>)> = {
        let devices = pool.devices.as_ref().expect("镜像开着");
        let mut saved = Vec::new();
        for (index, (_, device)) in devices.iter().enumerate() {
            for offset in system_configuration_slot_offsets() {
                let mut bytes = vec![0u8; usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")];
                device.read_at(singlefs_core::address::DeviceOffsetInBytes(offset), &mut bytes).expect("读系统配置槽");
                saved.push((index, offset, bytes));
            }
        }
        saved
    };
    let stream_before_third = pool.stream.operation_count();
    let third = overwrite(&mut pool, &content_seeded_by(2500, 11), InstanceGeneration(1));
    if std::env::var_os("ROW43_STREAM").is_some() {
        let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
        for (index, operation) in pool.stream.operations().iter().enumerate().skip(stream_before_third) {
            let marker = if operation.offset.0 < 2 * spacing && operation.kind != singlefs_harness::RecordedOperationKind::Barrier { " <- 系统配置槽" } else { "" };
            eprintln!("ROW43-STREAM C 那次发布 #{index}: {}{marker}", operation.to_stream_line());
        }
        let target = singlefs_core::root_ring::target_for_publish(third.root.checkpoint_txg, parameters().geometry.root_ring_slots_per_region);
        eprintln!("ROW43-STREAM C 的根槽：区域 {} 偏移 {}", target.region, singlefs_core::root_ring::slot_offset(target, parameters().geometry.fixed_structure_slot_spacing).0);
    }
    let mounted = match variant {
        Variant::DoubleFault => abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third),
        Variant::CrashBeforeRotation => abandon_the_third_version_by_a_crash_before_its_rotation(&mut pool, &third, &saved_system_configuration),
    };
    eprintln!(
        "ROW43 {variant:?}: C=(inst {}, txg {}); recovery lands on txg {}, new instance {}, row txg {}, current txg {}",
        third.root.instance.0, third.root.checkpoint_txg.0,
        mounted.output.effective_root.checkpoint_txg.0, mounted.output.instance.0,
        mounted.output.row_publish.root().checkpoint_txg.0, pool.output.root.checkpoint_txg.0
    );
    (pool, third)
}

fn raise_and_classify(variant: Variant, overwrites_after: usize, floor: u64) -> HistoryEnding {
    raise_and_classify_with(variant, false, overwrites_after, floor)
}

fn raise_and_classify_with(variant: Variant, remount_before_overwrites: bool, overwrites_after: usize, floor: u64) -> HistoryEnding {
    let (mut pool, _third) = pool_after_abandoning(variant, &format!("row43-{variant:?}-{remount_before_overwrites}-{overwrites_after}-{floor}"));
    let mut instance = InstanceGeneration(2);
    if remount_before_overwrites {
        let mut devices = pool.reopen_recorded();
        let mounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载");
        eprintln!("ROW43 remount: instance {}, row txg {}, current txg {}", mounted.output.instance.0, mounted.output.row_publish.root().checkpoint_txg.0, mounted.current.file_version().expect("带文件").root.checkpoint_txg.0);
        instance = mounted.output.instance;
        pool.devices = Some(devices);
        pool.allocator = mounted.allocator.clone();
        pool.output = mounted.current.file_version().expect("带文件").clone();
    }
    for seed in [17usize, 19, 23, 29, 31, 37].into_iter().take(overwrites_after) {
        overwrite(&mut pool, &content_seeded_by(3000 + seed, seed), instance);
    }
    let image_before = pool.memory_pool();
    let violations_before = violations_on(&image_before);
    let lands = raised_floor_lands_only_on_abandoned_roots(&image_before, CheckpointTxg(floor));
    let abandoned_present = row43_probe_any_ring_root_abandoned(&image_before);
    let current_txg = pool.output.root.checkpoint_txg.0;
    let raised = raise_rollback_floor(
        &parameters(),
        pool.devices.as_mut().expect("镜像开着"),
        &mut pool.allocator,
        &mut pool.output,
        CheckpointTxg(floor),
        ShadowLedger::On,
    );
    let raised_summary = match &raised {
        Ok(raised) => format!("Ok(publishes {}, reclaimed {}, ceiling {}, abandoned_roots_unreadable {})", raised.publishes.len(), raised.reclaimed.len(), raised.ceiling.0, raised.abandoned_roots_unreadable),
        Err(error) => format!("Err({error:?})"),
    };
    let image_after = pool.memory_pool();
    let violations_after = violations_on(&image_after);
    let i31_not_applicable = not_applicable_on(&image_after, "I-3.1");
    let (newest_ring_root_txg, root_ring_slot_count) = singlefs_harness::history::newest_ring_root_and_slot_count(&image_after);
    let ending = if violations_after.is_empty() {
        HistoryEnding::Completed
    } else {
        classify_failure(FailureObservation {
            position: StepPosition::Operation(overwrites_after),
            operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
            violations: violations_after.clone(),
            panic: None,
            newest_ring_root_txg,
            root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: lands,
            record_check: RecordCheck::default(),
        })
    };
    let ending_kind = match &ending {
        HistoryEnding::Completed => "Completed".to_string(),
        HistoryEnding::KnownRed { form, .. } => format!("KnownRed(form {form})"),
        HistoryEnding::NewFinding { signature, .. } => format!("NewFinding({signature:?})"),
    };
    eprintln!(
        "ROW43-RESULT variant={variant:?} remount={remount_before_overwrites} overwrites_after={overwrites_after} current_txg_before_raise={current_txg} floor={floor} abandoned_present_before={abandoned_present:?} lands_only_on_abandoned={lands:?}\n  violations_before={violations_before:?}\n  raise={raised_summary}\n  violations_after={violations_after:?}\n  i31_not_applicable_after={i31_not_applicable:?}\n  ending={ending_kind}"
    );
    ending
}

#[test]
fn row43_sweep_double_fault() {
    for overwrites_after in 0..=4 {
        for floor in 3..=7 {
            let _ = raise_and_classify(Variant::DoubleFault, overwrites_after, floor);
        }
    }
}

#[test]
fn row43_sweep_crash_before_rotation() {
    for overwrites_after in 0..=4 {
        for floor in 3..=7 {
            let _ = raise_and_classify(Variant::CrashBeforeRotation, overwrites_after, floor);
        }
    }
}

#[test]
fn row43_sweep_remount_before_overwrites() {
    for variant in [Variant::DoubleFault, Variant::CrashBeforeRotation] {
        for overwrites_after in 0..=6 {
            for floor in 3..=9 {
                let _ = raise_and_classify_with(variant, true, overwrites_after, floor);
            }
        }
    }
}

/// 最小复现：第一个文件 A（txg 3）→ 覆盖写 B（4）、C（5）→ C 的系统配置轮换没落盘就崩 → 下一次可写挂载时 C 的根槽与数据单元一时读不出
/// （重读一次仍读不出）：择根落到 B、实例 2 写行 6、暖机 7，C 按实例 2 的表判被抛弃 → 再可写挂载（实例 3，8–10）→ 覆盖写四次（11–14）
/// → 抬 F 到 5（C 那个 txg，空档）：只有 I-3.1 红，按「已知红」清单归到第 0 条。
#[test]
fn row43_minimal_crash_before_rotation_remount_four_overwrites_raise_into_the_gap() {
    let ending = raise_and_classify_with(Variant::CrashBeforeRotation, true, 4, 5);
    assert!(matches!(ending, HistoryEnding::KnownRed { form: 0, .. }), "归到已知红第 0 条");
}

fn print_segments(label: &str, pool: &BuiltPool) {
    for device_map in &pool.allocator.devices {
        eprintln!(
            "ROW43-C379 {label}: 盘 {} empty_segments={} lowest_empty_segment={:?} isolated_slots={} allocated_slots={}",
            device_map.device.0,
            device_map.empty_segments(),
            device_map.lowest_empty_segment().map(|slot| slot.0),
            device_map.isolated_slots(),
            device_map.allocated_slots()
        );
    }
}

/// C379：最小复现那条池（再挂载之后影子账隔离了 C 的单元）上，把每块盘最低的全空段整段交给影子账隔离（攻方探针的造法，
/// `PoolAllocator::isolate_abandoned`），再覆盖写一次让记账行落盘；比记账行「全空聚簇段数」与分配器能开的最低段。
#[test]
fn row43_c379_whole_segment_isolated() {
    let (mut pool, _third) = pool_after_abandoning(Variant::CrashBeforeRotation, "row43-c379");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载");
    let instance = mounted.output.instance;
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    pool.output = mounted.current.file_version().expect("带文件").clone();
    print_segments("再挂载之后（影子账只隔离了 C 的单元）", &pool);
    let isolate_whole_segment = std::env::var_os("ROW43_C379_NO_ISOLATION").is_none();
    if isolate_whole_segment {
        let targets: Vec<_> = pool
            .allocator
            .devices
            .iter()
            .map(|device_map| (device_map.device, device_map.lowest_empty_segment().expect("有全空段")))
            .collect();
        for (device, segment) in targets {
            pool.allocator.isolate_abandoned(device, segment, 64);
        }
        print_segments("整段隔离之后", &pool);
    }
    overwrite(&mut pool, &content_seeded_by(3001, 41), instance);
    print_segments("覆盖写一次之后（记账行落盘）", &pool);
    let _ = violations_on(&pool.memory_pool());
}
