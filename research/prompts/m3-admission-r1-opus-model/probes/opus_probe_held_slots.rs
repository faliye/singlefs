//! 攻方腿 m3-admission-r1-opus，A3（C570、C546）：抬 F 那一串真发时第二次空发布的第一个写报错（`RaiseFloorSequencePublishFailed`），
//! 前一次已落盘、扣住位留在进程里；原样重发冻结的那一次之后，再发同一个用户写，看准入两判与推抬 F 各给什么。
//! 只在冻结副本的拷贝上跑（`~/.local/share/singlefs/work/m3-admission-r1-opus/repo/`），不进仓库的 crates/。
mod common_admission;
#[path = "opus_probe/common.rs"]
mod probe_common;

use common_admission::{
    fixed_geometry_of_the_384_slot_small_devices, DeviceOverASparseImage, PoolUnderTest,
};
use probe_common::{clone_pool, counts_of, line_of};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::make_filesystem::FIRST_VERSION_REGION_DEVICES;
use singlefs_core::mount::{unmount, FloorRaiseStop, MountError};
use singlefs_core::mounted_session::{UserChangePublished, UserChangeRefused};
use singlefs_core::transaction::{resend_the_frozen_publish, PoolWriter, PublishError};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;

fn bytes_of_units(units: usize) -> usize {
    units * data_unit_payload_capacity()
}

fn refusal_name(refusal: &PublishError) -> String {
    match refusal {
        PublishError::SpaceAdmissionRefused(short) => format!(
            "SpaceAdmissionRefused[{}]",
            short
                .short_devices
                .iter()
                .map(|d| format!(
                    "d{} avail_slots={} demand_slots={}",
                    d.device.0,
                    d.available.0 / 16384,
                    d.demand.0 / 16384
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
        PublishError::PlacementRefused { unit, refusal } => {
            format!("PlacementRefused unit={unit:?} refusal={refusal:?}")
        }
        other => format!("Other({})", format!("{other:?}").chars().take(120).collect::<String>()),
    }
}

fn stop_name(stop: &FloorRaiseStop) -> String {
    match stop {
        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded {
            publishes_pushed,
            publishes_of_the_next_raise,
        } => format!("BudgetExceeded pushed={publishes_pushed} next={publishes_of_the_next_raise}"),
        FloorRaiseStop::FloorAlreadyAtTheCeiling { floor, ceiling } => {
            format!("FloorAlreadyAtTheCeiling floor={} ceiling={}", floor.0, ceiling.0)
        }
        FloorRaiseStop::FloorRaiseRefusedForSpace { refusal } => {
            format!("FloorRaiseRefusedForSpace {}", mount_error_name(refusal))
        }
    }
}

fn mount_error_name(error: &MountError) -> String {
    match error {
        MountError::RaiseFloorSequencePublishFailed(failed) => format!(
            "RaiseFloorSequencePublishFailed persisted={} cause={}",
            failed.writes_of_persisted_publishes.len(),
            refusal_name(&failed.cause)
        ),
        MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
            refused_publish_in_the_sequence,
            publishes_in_the_sequence,
            cause,
        } => format!(
            "RehearsalRefused {refused_publish_in_the_sequence}/{publishes_in_the_sequence} cause={}",
            refusal_name(cause)
        ),
        other => format!("{}", format!("{other:?}").chars().take(160).collect::<String>()),
    }
}

pub fn outcome_name(outcome: &Result<UserChangePublished, UserChangeRefused>) -> String {
    match outcome {
        Ok(published) => format!(
            "Ok floor_raises={} raise_publishes={:?}",
            published.floor_raises.len(),
            published
                .floor_raises
                .iter()
                .map(|r| r.publishes.iter().map(|p| p.root.checkpoint_txg.0).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        ),
        Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space)) => format!(
            "NoSpaceAfterRaisingTheFloor floor_raises={} stop={} last_refusal={}",
            no_space.floor_raises.len(),
            stop_name(&no_space.stop),
            refusal_name(&no_space.last_refusal)
        ),
        Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed)) => format!(
            "FloorRaiseFailedWhilePushingForSpace cause={} pushed_by={}",
            mount_error_name(&failed.cause),
            refusal_name(&failed.refusal_that_pushed_the_failed_raise)
        ),
        Err(UserChangeRefused::Publish { cause, .. }) => format!("Publish {}", refusal_name(cause)),
        Err(other) => format!("Other {}", format!("{other:?}").chars().take(160).collect::<String>()),
    }
}

fn device_of_txg(txg: u64) -> u32 {
    FIRST_VERSION_REGION_DEVICES[usize::try_from(txg % 3).expect("区域号")].0
}

fn floor_and_txg<D: DeviceOverASparseImage>(pool: &PoolUnderTest<D>) -> String {
    let root = pool.session().current.root();
    format!("current_txg={} F={}", root.checkpoint_txg.0, root.rollback_floor.0)
}

/// 一种取样：每次顺序写 `units` 个单元，写到推得出一串抬 F 为止；在那一串的第 `failing_publish` 次（从 1 数）空发布的第一个写注入写错。
fn run_one(units: usize, failing_publish: usize) {
    println!("==== A3 units={units} failing_publish={failing_publish}");
    let plan = SharedFaultPlan::unarmed(fixed_geometry_of_the_384_slot_small_devices());
    let mut pool = PoolUnderTest::start_after_the_first_file(
        HistoryDeviceWidth::UnitAreaOf384Slots,
        |identity, device| FaultInjectingBlockDevice::new(identity, device, plan.clone()),
    );
    let mut writes_before_the_fault = None;
    for step in 1..=400 {
        let mut probe = clone_pool(&pool);
        let outcome = probe.sequential_write(bytes_of_units(units));
        match &outcome {
            Ok(published) if published.floor_raises.is_empty() => {
                let real = pool.sequential_write(bytes_of_units(units));
                assert!(real.is_ok(), "拷贝上做成的同一个写在原池上也做成：{}", outcome_name(&real));
            }
            Ok(published) => {
                let raise = &published.floor_raises[0];
                println!(
                    "PROBE found step={step} {} raise_publishes={:?} control_outcome={}",
                    floor_and_txg(&pool),
                    raise.publishes.iter().map(|p| p.root.checkpoint_txg.0).collect::<Vec<_>>(),
                    outcome_name(&outcome)
                );
                if raise.publishes.len() < failing_publish {
                    println!("PROBE skip: this raise has only {} publishes", raise.publishes.len());
                    return;
                }
                let mut writes = raise
                    .system_configuration_writes_before_the_first_publish
                    .total()
                    .write_calls;
                for publish in &raise.publishes[..failing_publish - 1] {
                    writes += publish.writes.total().write_calls;
                }
                writes_before_the_fault = Some(writes);
                break;
            }
            Err(_) => {
                println!("PROBE step={step} refused without a raise that made room: {}", outcome_name(&outcome));
                return;
            }
        }
    }
    let Some(writes_before_the_fault) = writes_before_the_fault else {
        println!("PROBE no raise within 400 steps");
        return;
    };
    println!("{}", line_of("before_fault", &counts_of(&pool.session().allocator, &pool.session().current)));
    plan.arm(FaultSchedule::the_nth_call_across_the_pool(
        InjectedFault::WriteFails,
        writes_before_the_fault + 1,
    ));
    let failed = pool.sequential_write(bytes_of_units(units));
    plan.disarm();
    println!("PROBE faulted_write outcome={} {}", outcome_name(&failed), floor_and_txg(&pool));
    println!("{}", line_of("after_fault", &counts_of(&pool.session().allocator, &pool.session().current)));
    let frozen = pool.session().allocator.frozen_publish().map(|f| f.checkpoint_txg().0);
    println!("PROBE frozen_publish={frozen:?}");
    // 原样重发冻结的那一次（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」），会话的现行那一版换成它。
    if frozen.is_some() {
        let mut session = pool.session.take().expect("会话");
        let resent = {
            let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
            resend_the_frozen_publish(&mut writer, &mut session.allocator)
        };
        match resent {
            Ok(Some(version)) => session.current = version,
            other => println!("PROBE resend outcome={other:?}"),
        }
        pool.session = Some(session);
    }
    let floor = pool.session().current.root().rollback_floor;
    let current_txg = pool.session().current.root().checkpoint_txg;
    // 带新 F 的根落在哪几块盘：这一串从 F 之后的第一次空发布到现行那一版，每个 txg 的根落区域 txg mod 3 的盘。
    let first_of_the_sequence = pool
        .content_of_each_root
        .keys()
        .map(|(_, txg)| txg.0)
        .filter(|txg| *txg <= current_txg.0)
        .max()
        .unwrap_or(0);
    let _ = first_of_the_sequence;
    println!(
        "PROBE after_resend {} devices_of_txgs_last3={:?}",
        floor_and_txg(&pool),
        (current_txg.0.saturating_sub(2)..=current_txg.0)
            .map(|txg| (txg, device_of_txg(txg)))
            .collect::<Vec<_>>()
    );
    let stuck = counts_of(&pool.session().allocator, &pool.session().current);
    println!("{}", line_of("stuck", &stuck));
    let _ = (floor, CheckpointTxg(0));

    // 同一个用户写再发一次。
    let mut retry = clone_pool(&pool);
    let retried = retry.sequential_write(bytes_of_units(units));
    println!("PROBE scan retry_same_write outcome={}", outcome_name(&retried));
    // 只放开扣住位（模拟「F 在每块盘上生效就放开」那一种改法），读数前后比一次，再发同一个写。
    let mut released = clone_pool(&pool);
    let before = counts_of(&released.session().allocator, &released.session().current);
    released.session.as_mut().expect("会话").allocator.release_reclaim_holds();
    let after = counts_of(&released.session().allocator, &released.session().current);
    println!(
        "PROBE scan release_holds formula_avail_before={:?} after={:?}",
        before.iter().map(|c| c.formula_available_slots).collect::<Vec<_>>(),
        after.iter().map(|c| c.formula_available_slots).collect::<Vec<_>>()
    );
    let after_release = released.sequential_write(bytes_of_units(units));
    println!("PROBE scan release_holds_then_same_write outcome={}", outcome_name(&after_release));
    // 小一些的写：从 1 个单元到这次的单元数。
    for smaller in 1..=units {
        let mut probe = clone_pool(&pool);
        let outcome = probe.sequential_write(bytes_of_units(smaller));
        println!("PROBE scan smaller_write units={smaller} outcome={}", outcome_name(&outcome));
    }
    // 先做 k 次 1 单元的覆盖写，再发同一个写。
    for k in [1usize, 2, 3, 4, 6, 8] {
        let mut probe = clone_pool(&pool);
        let mut small_ok = 0;
        for _ in 0..k {
            if probe.overwrite(2999).is_ok() {
                small_ok += 1;
            }
        }
        let outcome = probe.sequential_write(bytes_of_units(units));
        println!(
            "PROBE scan small_overwrites k={k} ok={small_ok} then_same_write outcome={} {}",
            outcome_name(&outcome),
            floor_and_txg(&probe)
        );
    }
    // 正常卸载、再可写挂载，再发同一个写。
    let mut probe = clone_pool(&pool);
    let mut session = probe.session.take().expect("会话");
    let unmounted = unmount(
        &probe.parameters,
        &mut probe.devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    );
    println!("PROBE scan unmount ok={}", unmounted.is_ok());
    match probe.crash_and_mount_writable() {
        Ok(_) => {
            let outcome = probe.sequential_write(bytes_of_units(units));
            println!("PROBE scan unmount_remount_then_same_write outcome={}", outcome_name(&outcome));
        }
        Err(error) => println!("PROBE scan remount_failed {}", mount_error_name(&error)),
    }
}

#[test]
fn opus_probe_a3_held_slots_after_a_raise_failing_midway() {
    let units_list: Vec<usize> = std::env::var("OPUS_PROBE_UNITS")
        .ok()
        .map(|text| text.split(',').map(|n| n.parse().expect("单元数")).collect())
        .unwrap_or_else(|| vec![4, 8, 16, 32, 48]);
    for units in units_list {
        for failing_publish in [2usize, 3] {
            run_one(units, failing_publish);
        }
    }
}

/// 宽一些的池（单元区 `unit_area_slots` 槽，journal 环 `journal_ring_bytes`，其余参数取 384 槽那一档）：一次发布的需求大过式子里
/// 那几项常驻扣项（环窗口、切换预留、保留池）时，扣住位让「式子放行、落得下那一判拒」。历史：每次顺序写 `units` 个单元（整份重写，
/// 上一份进 defer），写到推得出一串抬 F 为止；在那一串第 2 次空发布的第一个写注入写错，原样重发冻结的那一次，再发同一个写。
fn run_wide(unit_area_slots: u64, journal_ring_bytes: u64, units: usize, small_overwrites: usize, failing_publish: usize) {
    use common_admission::parameters_of_the_384_slot_small_devices;
    use singlefs_core::allocator::UnitAreaStart;
    use singlefs_harness::segments::FixedGeometry;
    let mut parameters = parameters_of_the_384_slot_small_devices();
    parameters.geometry.journal_ring_bytes = journal_ring_bytes;
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let unit_area_start_slot = UnitAreaStart::following_the_journal_ring(journal_ring_bytes)
        .expect("环末尾在段边界上")
        .slot()
        .0;
    let device_bytes = (unit_area_start_slot + unit_area_slots) * 16384;
    let plan = SharedFaultPlan::unarmed(geometry.clone());
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters,
        device_bytes,
        |identity, device| FaultInjectingBlockDevice::new(identity, device, plan.clone()),
    );
    let start = counts_of(&pool.session().allocator, &pool.session().current);
    let reserves = i128::from(start[0].capacity) - i128::from(start[0].allocated) - i128::from(start[0].isolated)
        + i128::from(start[0].allocated_and_isolated)
        - start[0].formula_available_slots;
    println!("==== A3-wide unit_area_slots={unit_area_slots} journal_ring_bytes={journal_ring_bytes} units={units} small_overwrites={small_overwrites} failing_publish={failing_publish} reserves_slots={reserves}");
    println!("{}", line_of("wide_start", &start));
    // 大文件 k 单元 → 1 单元覆盖写 4 次（大文件的槽进 defer，释放代落在第 4 新的非空根之下）→ 再写 k 单元。
    let big = pool.sequential_write(bytes_of_units(units));
    println!("PROBE wide big_write units={units} outcome={} {}", outcome_name(&big), floor_and_txg(&pool));
    if big.is_err() {
        return;
    }
    for _ in 0..small_overwrites {
        let small = pool.overwrite(2999);
        assert!(small.is_ok(), "{}", outcome_name(&small));
    }
    let mut writes_before_the_fault = None;
    {
        let mut probe = clone_pool(&pool);
        let outcome = probe.sequential_write(bytes_of_units(units));
        println!("PROBE wide control_second_big_write={} {}", outcome_name(&outcome), floor_and_txg(&probe));
        if let Ok(published) = &outcome {
            if let Some(raise) = published.floor_raises.first() {
                if raise.publishes.len() >= failing_publish {
                    let mut writes = raise.system_configuration_writes_before_the_first_publish.total().write_calls;
                    for publish in &raise.publishes[..failing_publish - 1] {
                        writes += publish.writes.total().write_calls;
                    }
                    writes_before_the_fault = Some(writes);
                } else {
                    println!("PROBE wide raise has only {} publishes", raise.publishes.len());
                }
            }
        }
    }
    let Some(writes_before_the_fault) = writes_before_the_fault else {
        println!("PROBE wide control did not push a raise that made room");
        return;
    };
    println!("{}", line_of("wide_before_fault", &counts_of(&pool.session().allocator, &pool.session().current)));
    plan.arm(FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, writes_before_the_fault + 1));
    let failed = pool.sequential_write(bytes_of_units(units));
    plan.disarm();
    println!("PROBE wide faulted_write={} {}", outcome_name(&failed), floor_and_txg(&pool));
    let mut session = pool.session.take().expect("会话");
    let resent = {
        let mut writer = PoolWriter::new(&pool.parameters, pool.devices.as_mut_slice());
        resend_the_frozen_publish(&mut writer, &mut session.allocator)
    };
    let resent_ok = matches!(resent, Ok(Some(_)));
    if let Ok(Some(version)) = resent {
        session.current = version;
    }
    pool.session = Some(session);
    let current_txg = pool.session().current.root().checkpoint_txg.0;
    println!(
        "PROBE wide resent={resent_ok} {} devices_of_last2={:?}",
        floor_and_txg(&pool),
        (current_txg - 2..=current_txg).map(|t| (t, device_of_txg(t))).collect::<Vec<_>>()
    );
    println!("{}", line_of("wide_stuck", &counts_of(&pool.session().allocator, &pool.session().current)));
    let mut retry = clone_pool(&pool);
    let retried = retry.sequential_write(bytes_of_units(units));
    println!("PROBE wide retry_same_write={}", outcome_name(&retried));
    let mut released = clone_pool(&pool);
    released.session.as_mut().expect("会话").allocator.release_reclaim_holds();
    println!("{}", line_of("wide_released", &counts_of(&released.session().allocator, &released.session().current)));
    let after_release = released.sequential_write(bytes_of_units(units));
    println!("PROBE wide release_holds_then_same_write={}", outcome_name(&after_release));
    for k in [1usize, 2, 3, 4, 5] {
        let mut probe = clone_pool(&pool);
        let mut small_ok = 0;
        for _ in 0..k {
            if probe.overwrite(2999).is_ok() {
                small_ok += 1;
            }
        }
        let outcome = probe.sequential_write(bytes_of_units(units));
        println!(
            "PROBE wide small_overwrites k={k} ok={small_ok} then_same_write={} {}",
            outcome_name(&outcome),
            floor_and_txg(&probe)
        );
    }
    for smaller in [units / 4, units / 2, (units * 3) / 4] {
        let mut probe = clone_pool(&pool);
        let outcome = probe.sequential_write(bytes_of_units(smaller));
        println!("PROBE wide smaller_write units={smaller} outcome={}", outcome_name(&outcome));
    }
}

#[test]
fn opus_probe_a3_holds_exceeding_the_reserves_on_a_wider_pool() {
    let unit_area_slots: u64 = std::env::var("OPUS_PROBE_UNIT_AREA_SLOTS")
        .ok()
        .map_or(8192, |text| text.parse().expect("槽数"));
    let journal_ring_bytes: u64 = std::env::var("OPUS_PROBE_JOURNAL_RING_BYTES")
        .ok()
        .map_or(64 << 20, |text| text.parse().expect("字节数"));
    let units_list: Vec<usize> = std::env::var("OPUS_PROBE_WIDE_UNITS")
        .ok()
        .map(|text| text.split(',').map(|n| n.parse().expect("单元数")).collect())
        .unwrap_or_else(|| vec![600, 1000, 1500, 2000]);
    let small_overwrites_list: Vec<usize> = std::env::var("OPUS_PROBE_SMALL_OVERWRITES")
        .ok()
        .map(|text| text.split(',').map(|n| n.parse().expect("次数")).collect())
        .unwrap_or_else(|| vec![4, 5, 6]);
    for units in units_list {
        for &small_overwrites in &small_overwrites_list {
            for failing_publish in [2usize, 3] {
                run_wide(unit_area_slots, journal_ring_bytes, units, small_overwrites, failing_publish);
            }
        }
    }
}
