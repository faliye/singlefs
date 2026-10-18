//! C834 正推腿的观测装置（草稿副本里的测试，不进仓）：同一段历史，两种交盘次序各做一次可写挂载，记结局与
//! 「写行那次发布的根落盘之前 R 的单元被写掉几份」。候选由环境变量 C834_VARIANT 选（见 mount-c834-variants.patch）。
//! 历史（M0 是取号的那次挂载，图都建在 `crash_image_with_the_root_persisted_and_the_rotation_lost` 上）：
//!   baseline     没有任何取号写（系统配置两盘都是 R 的实例代号）
//!   partial      M0 取号只落了第一块盘（盘 0 是新号、盘 1 是旧号），之后一个字节没写
//!   complete     M0 取号两块盘都落了，之后一个字节没写
//!   units_only   M0（R 的根槽、单元、记录读回全零那一形）取号做完、写行发布的单元写了、记录与根没落；M2 里 R 的记录读回全零
//!   urj          同 units_only，但写行发布的记录也落了（入库 urJ 那一形），M2 里没有读故障
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, record_one_publish_after_overwrites,
    recorded_devices_reading_back_all_zero_on, units_of_r_overwritten_on,
    fixed_geometry_of_the_384_slot_small_devices, PublishRecordedAsR, RangesReadingBackAllZero,
    RecordedPublish,
};
use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::verified_system_configuration_slots;
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;
use singlefs_core::address::DeviceIdentity as DeviceId;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::transaction::{acquire_instance, AcquisitionRollback, InstanceAcquisitionFailed, PoolWriter};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;

fn violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(identifier, _)| identifier)
        .collect()
}


const OVERWRITES_BEFORE_R: u64 = 5;

fn unit_writes_of_r_at<'recorded>(
    recorded: &'recorded RecordedPublish,
    pointer: &NodePointer,
) -> Vec<&'recorded RetainedWrite> {
    let writes = recorded.unit_writes_at(pointer);
    assert_eq!(writes.len(), 2);
    writes
}

fn every_record_of_r(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    (1..=recorded.record_count())
        .flat_map(|ordinal| recorded.record_writes_of_the_record(ordinal))
        .collect()
}

/// 入库用例 `a_newer_readable_root_whose_only_record_mapping_root_and_tree_table_read_back_zero_refuses...` 的那几段。
fn ranges_that_hide_the_account_of_r(recorded: &RecordedPublish) -> RangesReadingBackAllZero {
    let root_of_r = recorded.root();
    RangesReadingBackAllZero::default()
        .with_the_writes(&every_record_of_r(recorded))
        .with_the_writes(&unit_writes_of_r_at(recorded, &root_of_r.mapping_root))
        .with_the_writes(&unit_writes_of_r_at(recorded, &root_of_r.tree_table))
}

/// 取新实例的那次挂载（M0 或 M1）：`ranges` 读回全零，做成，交回它的全部写。
fn writes_of_the_mount_that_takes_a_new_instance(
    recorded: &RecordedPublish,
    ranges: &RangesReadingBackAllZero,
) -> Vec<RetainedWrite> {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(&crash_image, ranges, &stream);
    mount_writable(&recorded.pool.parameters, &mut devices)
        .expect("取新实例的那次挂载做得成（这一形的前提）");
    writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    )
    .0
}

struct History {
    name: &'static str,
    image: MemoryPool,
    ranges_at_the_mount_in_question: RangesReadingBackAllZero,
}


/// 取号失败并回卷之后的盘面：`schedule` 摆在取号上。回卷做成（`RolledBack`）。
fn image_after_a_rolled_back_acquisition(
    recorded: &RecordedPublish,
    base: &MemoryPool,
    schedule: FaultSchedule,
) -> MemoryPool {
    let plan = SharedFaultPlan::unarmed(fixed_geometry_of_the_384_slot_small_devices());
    let mut devices: Vec<(DeviceId, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        common_unwitnessed_publish::plain_devices_on(base)
            .into_iter()
            .map(|(identity, inner)| {
                (identity, FaultInjectingBlockDevice::new(identity, inner, plan.clone()))
            })
            .collect();
    plan.arm(schedule);
    let parameters = recorded.pool.parameters.clone();
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    match acquire_instance(&mut writer).expect_err("取号必须失败") {
        InstanceAcquisitionFailed::Acquisition(failure) => {
            assert!(matches!(failure.rollback, AcquisitionRollback::RolledBack), "{failure:?}")
        }
        other => panic!("{other:?}"),
    }
    drop(writer);
    let _ = PhysicalBlockSizeInBytes(512);
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: recorded.pool.device_bytes,
    }
}

fn histories(recorded: &RecordedPublish) -> Vec<History> {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let hide_account = ranges_that_hide_the_account_of_r(recorded);
    let mut histories = vec![History {
        name: "baseline",
        image: crash_image.clone(),
        ranges_at_the_mount_in_question: hide_account.clone(),
    }];
    // M0：没有读故障，R 被择、取新号；按系统配置槽写的个数截。
    let m0 = writes_of_the_mount_that_takes_a_new_instance(
        recorded,
        &RangesReadingBackAllZero::default(),
    );
    let system_configuration_writes: Vec<usize> = m0
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::SystemConfigurationSlot)
        .map(|(index, _)| index)
        .collect();
    assert!(
        system_configuration_writes.len() >= 2,
        "M0 的取号至少写两块盘各一槽：{system_configuration_writes:?}"
    );
    let mut partial = crash_image.clone();
    partial.apply_writes(&m0[..=system_configuration_writes[0]]);
    histories.push(History {
        name: "partial",
        image: partial,
        ranges_at_the_mount_in_question: hide_account.clone(),
    });
    let mut complete = crash_image.clone();
    complete.apply_writes(&m0[..=system_configuration_writes[1]]);
    histories.push(History {
        name: "complete",
        image: complete,
        ranges_at_the_mount_in_question: hide_account.clone(),
    });
    // M1：R 的根槽、单元、第一条记录读回全零（入库 urJ 的前提），取新号，写行发布的单元、记录、根。
    let m1_ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes())
        .with_the_writes(&recorded.record_writes_of_the_record(1));
    let m1 = writes_of_the_mount_that_takes_a_new_instance(recorded, &m1_ranges);
    let first_record = m1
        .iter()
        .position(|write| write.kind == StepKind::JournalRecord)
        .expect("M1 写行发布有记录写");
    let row_root = m1
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("M1 写行发布有根槽写");
    let mut units_only = crash_image.clone();
    units_only.apply_writes(&m1[..first_record]);
    histories.push(History {
        name: "units_only",
        image: units_only,
        ranges_at_the_mount_in_question: RangesReadingBackAllZero::default()
            .with_the_writes(&every_record_of_r(recorded)),
    });
    let mut urj = crash_image.clone();
    urj.apply_writes(&m1[..row_root]);
    histories.push(History {
        name: "urj",
        image: urj,
        ranges_at_the_mount_in_question: RangesReadingBackAllZero::default(),
    });
    // urj 再加：M2 里写行发布的记录也读回全零（两处故障）
    let row_record_writes: Vec<&RetainedWrite> = m1[..row_root]
        .iter()
        .filter(|write| write.kind == StepKind::JournalRecord)
        .collect();
    let mut urj_hidden = crash_image.clone();
    urj_hidden.apply_writes(&m1[..row_root]);
    histories.push(History {
        name: "urj_row_record_also_hidden",
        image: urj_hidden,
        ranges_at_the_mount_in_question: RangesReadingBackAllZero::default()
            .with_the_writes(&row_record_writes),
    });
    // 后一次挂载 M0（没有读故障，择的是 R）取号、写行发布的单元与记录都落了，根没落：R 的单元一份不少
    let first_root = m0
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("M0 写行发布有根槽写");
    let mut later_records = crash_image.clone();
    later_records.apply_writes(&m0[..first_root]);
    histories.push(History {
        name: "later_mount_records_landed_r_units_intact",
        image: later_records,
        ranges_at_the_mount_in_question: hide_account.clone(),
    });
    // 取号第二块盘的写报错，盘 0 那份回卷成旧号（盘 0 两槽 = [新号, 旧号] 但旧号的世代号更大）
    let rolled_back_one = image_after_a_rolled_back_acquisition(
        recorded,
        &crash_image,
        FaultSchedule {
            fault: InjectedFault::WriteFails,
            device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
            placement: FaultPlacement::OffsetBelow(
                singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
                    * u64::from(recorded.pool.parameters.geometry.fixed_structure_slot_spacing),
            ),
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        },
    );
    histories.push(History {
        name: "acquisition_failed_on_device_1_rolled_back",
        image: rolled_back_one,
        ranges_at_the_mount_in_question: hide_account.clone(),
    });
    // 取号两块盘都写了、写之后那道屏障报错：两块盘都回卷
    let rolled_back_both = image_after_a_rolled_back_acquisition(
        recorded,
        &crash_image,
        FaultSchedule {
            fault: InjectedFault::BarrierFails,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::AnyOffset,
            counting: FaultCounting::AcrossThePool,
            occurrence: FaultOccurrence::TheNthMatchingCall(3),
        },
    );
    histories.push(History {
        name: "acquisition_barrier_failed_rolled_back_on_both",
        image: rolled_back_both,
        ranges_at_the_mount_in_question: hide_account.clone(),
    });
    histories
}

fn system_configuration_instances_per_device(
    image: &MemoryPool,
    recorded: &RecordedPublish,
) -> Vec<(DeviceIdentity, Vec<u32>)> {
    let parameters = &recorded.pool.parameters;
    image
        .devices
        .keys()
        .map(|device| {
            (
                *device,
                verified_system_configuration_slots(
                    image,
                    *device,
                    u64::from(parameters.geometry.fixed_structure_slot_spacing),
                    &parameters.filesystem_identifier,
                )
                .into_iter()
                .map(|slot| slot.quantities.journal_instance.0)
                .collect(),
            )
        })
        .collect()
}

/// 一次可写挂载：结局串、写行那次发布的根落盘之前 R 的单元被写掉几份（做成时才有）。
fn mount_once(
    recorded: &RecordedPublish,
    history: &History,
    reversed: bool,
) -> (String, Option<usize>) {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(
        &history.image,
        &history.ranges_at_the_mount_in_question,
        &stream,
    );
    if reversed {
        devices.reverse();
    }
    let _ = singlefs_core::mount::c834_take_judge_lines();
    let outcome = mount_writable(&recorded.pool.parameters, &mut devices);
    for line in singlefs_core::mount::c834_take_judge_lines() {
        println!("JUDGE history={} reversed={reversed} {line}", history.name);
    }
    match outcome {
        Ok(mounted) => {
            let effective = (
                mounted.output.effective_root.instance.0,
                mounted.output.effective_root.checkpoint_txg.0,
            );
            let writes = writes_and_segments_with_stream_indexes(
                &stream.retained_operations(),
                &fixed_geometry_of_the_384_slot_small_devices(),
            )
            .0;
            let row_root = writes
                .iter()
                .position(|write| write.kind == StepKind::RootRecordFua)
                .expect("做成的可写挂载有写行发布的根槽写");
            let mut before_the_row_root = history.image.clone();
            before_the_row_root.apply_writes(&writes[..row_root]);
            let overwritten = units_of_r_overwritten_on(recorded, &before_the_row_root);
            let already_overwritten_in_the_image =
                units_of_r_overwritten_on(recorded, &history.image);
            (
                format!("做成 落到(instance,txg)={effective:?} 写行根落盘前断电的盘面上池级 checker 红={:?}", violated_invariants(&before_the_row_root)),
                Some(overwritten.len().saturating_sub(already_overwritten_in_the_image.len())),
            )
        }
        Err(refusal) => {
            assert_eq!(stream.operation_count(), 0, "拒在任何写之前：{refusal:?}");
            let short = match &refusal {
                MountError::NewerStateStillUnreadableAfterOneReread(still) => {
                    format!("拒 NewerStateStillUnreadableAfterOneReread::{still:?}")
                }
                other => format!("拒 {other:?}"),
            };
            (short.chars().take(150).collect(), None)
        }
    }
}

#[test]
fn print_the_outcome_of_each_history_under_both_device_orders() {
    let variant = std::env::var("C834_CANDIDATE").unwrap_or_else(|_| "today".to_string());
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    println!("R = {:?}（实例, txg）", recorded.key);
    let mut divergent = 0;
    for history in histories(&recorded) {
        println!(
            "history={} 系统配置各盘各槽的实例代号={:?} 这张盘面本身池级 checker 红={:?}",
            history.name,
            system_configuration_instances_per_device(&history.image, &recorded),
            violated_invariants(&history.image)
        );
        let forward = mount_once(&recorded, &history, false);
        let reversed = mount_once(&recorded, &history, true);
        println!(
            "ROW variant={variant} history={} 盘序=0,1 => {} | 这次挂载新写掉的 R 单元份数={:?}（挂载前盘面上已被写掉 {} 份）",
            history.name, forward.0, forward.1, units_of_r_overwritten_on(&recorded, &history.image).len()
        );
        println!(
            "ROW variant={variant} history={} 盘序=1,0 => {} | 这次挂载新写掉的 R 单元份数={:?}（挂载前盘面上已被写掉 {} 份）",
            history.name, reversed.0, reversed.1, units_of_r_overwritten_on(&recorded, &history.image).len()
        );
        if forward != reversed {
            divergent += 1;
        }
    }
    println!("SUMMARY variant={variant} 两种盘序结局不同的历史数={divergent}");
    let _ = devices_reading_back_all_zero_on;
    let _ = InstanceGeneration(0);
}

/// 后一次挂载的记录落了、根没落、R 的单元一份不少（它择的是 R，写行发布建在 R 上）：M2 做成之后，写行根落盘之前与之后三次用户动作，
/// R 的根还在环里时 R 的单元一份不少吗（入库助手 `mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions`）。
#[test]
fn units_of_r_stay_when_a_later_mounts_records_landed_on_top_of_r() {
    let variant = std::env::var("C834_CANDIDATE").unwrap_or_else(|_| "today".to_string());
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let history = histories(&recorded)
        .into_iter()
        .find(|history| history.name == "later_mount_records_landed_r_units_intact")
        .expect("有这一形");
    let outcome =
        common_unwitnessed_publish::mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions(
            &recorded,
            &history.image,
            &history.ranges_at_the_mount_in_question,
            "后一次挂载的记录落在 R 之上",
        );
    println!("SAFE variant={variant} 结局={:?}", outcome.map_err(|refusal| format!("{refusal:?}").chars().take(60).collect::<String>()));
}

/// 条款的会红用例（候选 k6 定下的结局表）：同一段历史，两种交盘次序结局都必须是表里这个。
/// 「拒」= 取号之前拒可写、拒因是 `AccountOfAnUnappliedPublishWithIncompleteReadableRecords`、点名 R；「做成」= 可写挂载做成。
/// 变异：把凭据读法改回「读择到那一份的实例代号」（k1）等，下面这一条必须红。
#[test]
fn the_judgement_follows_the_clause_table_under_both_device_orders() {
    let variant = std::env::var("C834_CANDIDATE").unwrap_or_else(|_| "today".to_string());
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let expected = |name: &str| match name {
        "baseline" | "partial" | "complete" => "拒",
        "acquisition_failed_on_device_1_rolled_back"
        | "acquisition_barrier_failed_rolled_back_on_both" => "拒",
        "units_only" | "urj" | "urj_row_record_also_hidden"
        | "later_mount_records_landed_r_units_intact" => "做成",
        other => panic!("表里没有 {other}"),
    };
    let mut mismatches: Vec<String> = Vec::new();
    for history in histories(&recorded) {
        for (reversed, label) in [(false, "盘序0,1"), (true, "盘序1,0")] {
            let (outcome, newly_overwritten) = mount_once(&recorded, &history, reversed);
            let kind = if outcome.starts_with("做成") { "做成" } else { "拒" };
            let safe = newly_overwritten.unwrap_or(0) == 0;
            if kind != expected(history.name) || !safe {
                mismatches.push(format!(
                    "variant={variant} {} {label}：得 {kind}（新写掉 R 单元 {:?} 份），表要 {}",
                    history.name,
                    newly_overwritten,
                    expected(history.name)
                ));
            }
        }
    }
    for line in &mismatches {
        println!("MISMATCH {line}");
    }
    assert!(mismatches.is_empty(), "{} 格与条款表不符", mismatches.len());
}
