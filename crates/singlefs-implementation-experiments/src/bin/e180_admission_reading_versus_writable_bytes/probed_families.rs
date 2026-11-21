//! 各族的驱动（填满、覆盖、先删后写、多次挂载、长挂载、弃根与净释放）与一族的跑法和线程。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 历史族（登记 5 的「历史族」表）：每族一条确定的脚本，种子写死
// ---------------------------------------------------------------------------------------------------------------------

/// 登记里覆盖写族、多次挂载族的循环大小。
const OVERWRITE_FAMILY_SEQUENTIAL_SIZES: [u64; 6] = [64, 512, 2048, 128, 1024, 64];
/// 登记里长挂载族的循环大小。
const LONG_MOUNT_SEQUENTIAL_SIZES: [u64; 5] = [1, 8, 64, 512, 4096];
const FILL_FAMILY_STEPS_UP_TO_THE_FULL_RANGE: u64 = 20;
/// 一条爬升梯最多爬几档（让循环有上界）。
const LADDER_CLIMBS_AT_MOST: u64 = 200;

/// M = 这个几何上段外槽对数的一半（登记 5：填满族）：在第一个状态上按各盘「用户数据落得下的槽对数」取小再除以 2。
fn half_of_the_pairs_outside_the_cluster_segments(pool: &Pool) -> u64 {
    pool.session
        .allocator
        .devices
        .iter()
        .map(|map| {
            map.unblocked_slot_pairs_outside_the_cluster_segments(
                pool.session.allocator.cluster_segments(),
            )
            .0
        })
        .min()
        .expect("至少一块盘")
        / 2
}

fn ladder_step_units(half_pairs: u64) -> u64 {
    half_pairs.div_ceil(FILL_FAMILY_STEPS_UP_TO_THE_FULL_RANGE)
}

fn change_for_the_overwrite_cycle(change_number: u64) -> ChangeKind {
    if change_number.is_multiple_of(20) {
        let cycle_index = usize::try_from(change_number / 20 - 1).expect("循环下标")
            % OVERWRITE_FAMILY_SEQUENTIAL_SIZES.len();
        ChangeKind::SequentialWriteOfUnits(OVERWRITE_FAMILY_SEQUENTIAL_SIZES[cycle_index])
    } else {
        ChangeKind::OverwriteWithOneUnit
    }
}

fn change_and_observe(run: &mut HistoryRun, kind: ChangeKind) -> ChangeOutcome {
    let outcome = apply_change(&mut run.pool, kind);
    run.observe("after_change", &format!("{kind:?}:{}", outcome.label()));
    outcome
}

/// 填满族：1 单元起，按 ⌈M ÷ 20⌉ 单元一档往上换写到被拒，换写回 1 单元，再往上换写到被拒。
fn drive_fill_family(run: &mut HistoryRun) {
    run.observe("mounted", "after_first_file_and_remount");
    let half_pairs = half_of_the_pairs_outside_the_cluster_segments(&run.pool);
    let step_units = ladder_step_units(half_pairs);
    run.lines.push(
        ProductLine::new("family_parameters")
            .field("geometry", run.pool.geometry.name)
            .field("family", run.family)
            .field("half_of_the_pairs_outside_the_cluster_segments", half_pairs)
            .field("ladder_step_units", step_units)
            .finish(),
    );
    for round in 0..2 {
        if round == 1 {
            change_and_observe(run, ChangeKind::SequentialWriteOfUnits(1));
        }
        for climb in 1..=LADDER_CLIMBS_AT_MOST {
            let units = (climb * step_units).min(PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS);
            let outcome = change_and_observe(run, ChangeKind::SequentialWriteOfUnits(units));
            if !matches!(outcome, ChangeOutcome::Published { .. }) {
                break;
            }
        }
    }
}

/// 覆盖写族：第一个文件 64 单元；120 次改动，每 20 次里 19 次覆盖写、1 次顺序写。
fn drive_overwrite_family(run: &mut HistoryRun) {
    run.observe("mounted", "after_first_file_and_remount");
    change_and_observe(run, ChangeKind::SequentialWriteOfUnits(64));
    for change_number in 1..=120 {
        change_and_observe(run, change_for_the_overwrite_cycle(change_number));
    }
}

/// 删了再写族：换写到 M 的八成，换写回 1 单元，再 3 次覆盖写，之后每次改动都试换写回八成（被拒也是一个状态）。
fn drive_delete_then_write_family(run: &mut HistoryRun) {
    run.observe("mounted", "after_first_file_and_remount");
    let half_pairs = half_of_the_pairs_outside_the_cluster_segments(&run.pool);
    let eighty_percent_units = half_pairs * 8 / 10;
    run.lines.push(
        ProductLine::new("family_parameters")
            .field("geometry", run.pool.geometry.name)
            .field("family", run.family)
            .field("half_of_the_pairs_outside_the_cluster_segments", half_pairs)
            .field("eighty_percent_units", eighty_percent_units)
            .finish(),
    );
    change_and_observe(
        run,
        ChangeKind::SequentialWriteOfUnits(eighty_percent_units),
    );
    change_and_observe(run, ChangeKind::SequentialWriteOfUnits(1));
    for _ in 0..3 {
        change_and_observe(run, ChangeKind::OverwriteWithOneUnit);
    }
    for _ in 0..25 {
        change_and_observe(
            run,
            ChangeKind::SequentialWriteOfUnits(eighty_percent_units),
        );
    }
}

/// 多次挂载族：三轮「可写挂载 → 10 次改动（覆盖写族的循环接着数）→ 正常卸载」，取每次改动之后与每次挂载做成之后的状态。
fn drive_many_mounts_family(run: &mut HistoryRun) {
    let mut change_number = 0u64;
    for round in 0..3 {
        if round > 0 {
            let observed = unmount_and_mount_again(&mut run.pool);
            let expected_pushes = if observed.current_txg_before_the_unmount % 3 == 1 {
                3
            } else {
                2
            };
            run.tally.unmount_pushes_checked += 1;
            if observed.empty_publishes_pushed != expected_pushes {
                run.tally.unmount_pushes_differing.push(format!(
                    "{}:txg_before={}:pushed={}:expected={expected_pushes}",
                    run.pool.geometry.name,
                    observed.current_txg_before_the_unmount,
                    observed.empty_publishes_pushed
                ));
            }
        }
        run.observe("mounted", &format!("round_{round}"));
        for _ in 0..10 {
            change_number += 1;
            change_and_observe(run, change_for_the_overwrite_cycle(change_number));
        }
    }
}

/// 长挂载族：一次挂载里 2000 次改动，每 50 次里 49 次覆盖写、1 次顺序写，中间不卸载。
fn drive_long_mount_family(run: &mut HistoryRun) {
    run.observe("mounted", "after_first_file_and_remount");
    for change_number in 1..=2000u64 {
        let kind = if change_number.is_multiple_of(50) {
            let cycle_index = usize::try_from(change_number / 50 - 1).expect("循环下标")
                % LONG_MOUNT_SEQUENTIAL_SIZES.len();
            ChangeKind::SequentialWriteOfUnits(LONG_MOUNT_SEQUENTIAL_SIZES[cycle_index])
        } else {
            ChangeKind::OverwriteWithOneUnit
        };
        change_and_observe(run, kind);
    }
}

/// 崩溃态：挂载甲发 P 之前的盘面，加 P 发布交给设备的写里到「P 的根槽 FUA 写」为止（含它）的那些，两块盘按全局写序号合起来。
/// P 的系统配置轮换在根槽写之后，不在内。
struct CrashStateAfterTheRootSlotWrite {
    pub(super) devices: Vec<(DeviceIdentity, Disk<'static>)>,
    root_slot_device_index: usize,
    root_slot_byte_range: Range<u64>,
    /// 每块盘上 P 的 journal 记录占的字节区间。
    journal_record_byte_ranges: Vec<Vec<Range<u64>>>,
}

fn crash_state_after_the_root_slot_write(
    before_publish: &[SectorStore],
    device_bytes: &[u64; 2],
    recordings: &[Vec<RecordedWrite>],
) -> Option<CrashStateAfterTheRootSlotWrite> {
    let (root_slot_device_index, root_write) = recordings
        .iter()
        .enumerate()
        .flat_map(|(index, writes)| writes.iter().map(move |write| (index, write)))
        .filter(|(_, write)| write.is_force_unit_access)
        .min_by_key(|(_, write)| write.sequence)?;
    let cutoff = root_write.sequence;
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let ring_end = ring_start + DEFAULT_JOURNAL_RING_BYTES;
    let mut devices = Vec::new();
    let mut journal_record_byte_ranges = Vec::new();
    for (index, store) in before_publish.iter().enumerate() {
        let mut disk = Disk::independent_copy_of(store, device_bytes[index]);
        let mut ranges = Vec::new();
        for write in recordings[index]
            .iter()
            .filter(|write| write.sequence <= cutoff)
        {
            disk.own
                .write_sectors(write.byte_offset, &write.bytes, false);
            let length = u64::try_from(write.bytes.len()).expect("写长");
            if write.byte_offset >= ring_start && write.byte_offset + length <= ring_end {
                ranges.push(write.byte_offset..write.byte_offset + length);
            }
        }
        devices.push((DeviceIdentity(u32::try_from(index).expect("盘号")), disk));
        journal_record_byte_ranges.push(ranges);
    }
    let root_slot_byte_range = root_write.byte_offset
        ..root_write.byte_offset + u64::try_from(root_write.bytes.len()).expect("写长");
    Some(CrashStateAfterTheRootSlotWrite {
        devices,
        root_slot_device_index,
        root_slot_byte_range,
        journal_record_byte_ranges,
    })
}

/// 被抛弃根与净释放族：挂载甲（大文件、同大小重写、换写成小文件 P，P 的根槽 FUA 写之后截断），挂载乙（P 的根槽与它的 journal 记录每次读都失败，
/// 可写挂载做成后撤故障，10 次改动，正常卸载），挂载丙（不注故障，20 次改动，含两次换写变小）。登记 5 的「历史族」表 HC 一行。
fn drive_abandoned_root_and_net_release_family(run: &mut HistoryRun) {
    run.observe("mounted", "mount_a");
    change_and_observe(run, ChangeKind::SequentialWriteOfUnits(256));
    // 会话的覆盖写入口只收一个单元的内容、会把文件换成一个单元；要让 P 是净释放，这里的「8 次覆盖写」用同样 256 单元的顺序写（登记里这一族的意图）。
    for _ in 0..8 {
        change_and_observe(run, ChangeKind::SequentialWriteOfUnits(256));
    }
    let before_publish: Vec<SectorStore> = run
        .pool
        .devices
        .iter()
        .map(|(_, disk)| disk.own.clone())
        .collect();
    for (_, disk) in &mut run.pool.devices {
        disk.recording = Some(Vec::new());
    }
    let outcome_of_the_shrinking_rewrite =
        apply_change(&mut run.pool, ChangeKind::SequentialWriteOfUnits(32));
    let recordings: Vec<Vec<RecordedWrite>> = run
        .pool
        .devices
        .iter_mut()
        .map(|(_, disk)| disk.recording.take().unwrap_or_default())
        .collect();
    run.observe(
        "after_change",
        &format!("p_net_release:{}", outcome_of_the_shrinking_rewrite.label()),
    );
    let txg_of_the_shrinking_rewrite = run
        .pool
        .log
        .user_change_txgs
        .iter()
        .max()
        .copied()
        .unwrap_or(0);
    let Some(crash_state) = crash_state_after_the_root_slot_write(
        &before_publish,
        &run.pool.geometry.device_bytes,
        &recordings,
    ) else {
        run.lines.push(
            ProductLine::new("abandoned_root_and_net_release_family_status")
                .field("status", "no_root_slot_write_recorded")
                .finish(),
        );
        return;
    };
    drop(before_publish);
    let mut devices_of_the_faulted_mount = crash_state.devices;
    devices_of_the_faulted_mount[crash_state.root_slot_device_index]
        .1
        .unreadable_byte_ranges
        .push(crash_state.root_slot_byte_range.clone());
    for (index, ranges) in crash_state.journal_record_byte_ranges.iter().enumerate() {
        devices_of_the_faulted_mount[index]
            .1
            .unreadable_byte_ranges
            .extend(ranges.iter().cloned());
    }
    let mut log_of_the_faulted_mount = run.pool.log.clone();
    log_of_the_faulted_mount
        .user_change_txgs
        .remove(&txg_of_the_shrinking_rewrite);
    let mounted = match mount_writable(&run.pool.parameters, &mut devices_of_the_faulted_mount) {
        Ok(mounted) => mounted,
        Err(error) => {
            run.lines.push(
                ProductLine::new("abandoned_root_and_net_release_family_status")
                    .field("status", "mount_b_refused")
                    .field("error", short_debug(&error))
                    .finish(),
            );
            return;
        }
    };
    let (output_of_the_faulted_mount, session_of_the_faulted_mount) =
        MountedSession::of_the_mount(mounted);
    log_of_the_faulted_mount.note_mount(&output_of_the_faulted_mount);
    run.pool.devices = devices_of_the_faulted_mount;
    run.pool.session = session_of_the_faulted_mount;
    run.pool.log = log_of_the_faulted_mount;
    for (_, disk) in &mut run.pool.devices {
        disk.unreadable_byte_ranges.clear();
    }
    run.observe("mounted", "mount_b_faults_removed");
    for change_number in 1..=10u64 {
        change_and_observe(run, change_for_the_overwrite_cycle(change_number));
    }
    unmount_and_mount_again(&mut run.pool);
    run.observe("mounted", "mount_c");
    for change_number in 1..=20u64 {
        let kind = match change_number {
            1 => ChangeKind::SequentialWriteOfUnits(128),
            10 => ChangeKind::SequentialWriteOfUnits(64),
            _ => ChangeKind::OverwriteWithOneUnit,
        };
        change_and_observe(run, kind);
    }
    let abandoned_after_the_fault_free_mount = run
        .pool
        .session
        .allocator
        .root_ring_occupancy()
        .map_or(0, |occupancy| {
            occupancy
                .ring_slots_recorded_as_holding_an_abandoned_root()
                .len()
        });
    run.lines.push(
        ProductLine::new("abandoned_root_and_net_release_family_status")
            .field("geometry", run.pool.geometry.name)
            .field("status", "mounts_made")
            .field(
                "abandoned_root_ring_slots_at_the_end",
                abandoned_after_the_fault_free_mount,
            )
            .finish(),
    );
}

// ---------------------------------------------------------------------------------------------------------------------
// 一个几何格：几族历史并行，产物行按固定次序打
// ---------------------------------------------------------------------------------------------------------------------

pub(super) const PROBED_FAMILIES: [&str; 5] = [
    "fill",
    "overwrite",
    "delete_then_write",
    "many_mounts",
    "abandoned_root_and_net_release",
];
pub(super) const LONG_MOUNT_FAMILY: &str = "long_mount";
pub(super) const THREAD_STACK_BYTES: usize = 256 << 20;
const TOTAL_THREADS_ENVIRONMENT_VARIABLE: &str = "E180_THREADS";

fn drive_family(family: &str, run: &mut HistoryRun) {
    match family {
        "fill" => drive_fill_family(run),
        "overwrite" => drive_overwrite_family(run),
        "delete_then_write" => drive_delete_then_write_family(run),
        "many_mounts" => drive_many_mounts_family(run),
        "abandoned_root_and_net_release" => drive_abandoned_root_and_net_release_family(run),
        "long_mount" => drive_long_mount_family(run),
        unknown => stop_with("unknown_family", &format!("family={unknown}")),
    }
}

pub(super) fn static_family_name(family: &str) -> &'static str {
    PROBED_FAMILIES
        .iter()
        .copied()
        .chain([LONG_MOUNT_FAMILY])
        .find(|known| *known == family)
        .unwrap_or_else(|| stop_with("unknown_family", &format!("family={family}")))
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct FamilyOutcome {
    pub(super) family: &'static str,
    pub(super) lines: Vec<String>,
    pub(super) tally: FamilyTally,
    pub(super) anchors: FirstFileAnchors,
    pub(super) seconds: u64,
    pub(super) unit_area_slots_per_device: Vec<u64>,
    pub(super) trajectory_names: [&'static str; 3],
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .map_or_else(
            || "panic_without_text".to_string(),
            |text| short_debug(&text),
        )
}

pub(super) fn run_family(
    geometry: Geometry,
    family: &'static str,
    probe_threads: usize,
) -> FamilyOutcome {
    let started = Instant::now();
    let (pool, anchors) = build_pool(geometry);
    let unit_area_slots_per_device: Vec<u64> = pool
        .session
        .allocator
        .devices
        .iter()
        .map(|map| map.unit_area_slots())
        .collect();
    let mut run = HistoryRun::new(pool, family, family != LONG_MOUNT_FAMILY, probe_threads);
    if family == LONG_MOUNT_FAMILY {
        run.emit_state_line_every = 50;
    }
    let result = catch_unwind(AssertUnwindSafe(|| drive_family(family, &mut run)));
    if let Err(payload) = result {
        let message = panic_message(payload.as_ref());
        run.lines.push(
            ProductLine::new("panic")
                .field("geometry", geometry.name)
                .field("family", family)
                .field("states_before", run.state_index)
                .field("message", &message)
                .finish(),
        );
        run.tally.panics.push(message);
    }
    FamilyOutcome {
        family,
        lines: std::mem::take(&mut run.lines.lines),
        tally: std::mem::take(&mut run.tally),
        anchors,
        seconds: started.elapsed().as_secs(),
        unit_area_slots_per_device,
        trajectory_names: TRAJECTORY_NAMES,
    }
}

pub(super) fn total_threads() -> usize {
    let own = std::env::var(TOTAL_THREADS_ENVIRONMENT_VARIABLE).ok();
    match own {
        Some(text) => text
            .parse::<usize>()
            .ok()
            .filter(|count| *count > 0)
            .unwrap_or_else(|| {
                stop_with(
                    "bad_thread_variable",
                    &format!("{TOTAL_THREADS_ENVIRONMENT_VARIABLE}={text}"),
                )
            }),
        None => singlefs_verification::heavy_threads::default_heavy_threads_of_the_cap_value(
            std::env::var(singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE)
                .ok()
                .as_deref(),
        )
        .get(),
    }
}
