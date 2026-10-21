//! 正推腿模型（m3-close-rootring-r2，草稿目录里的仓副本，核心层带 `r2_switch` 开关）：Y3 与 Y5。
//! C755 的形：三处用同一种失败判「退不退、退到哪一版」，落到那一版之后恢复（走读）、只读挂载（打开）、可写挂载（建上一版）成不成各照今天。
//! 失败取哪一种是这一题的自变量：走读 `walk`、打开 `open`、建上一版 `rebuild`、三样并集 `any`（有一样读不出就算失败）、三样交集 `all`（三样都读不出才算失败）。
//! 先用「各处触发取自己那一种」把今天的结局算一遍，与真挂载逐格比（校准）；再逐个触发种类算「同一种失败」之后的落点与成败。
//! 盘面照第一轮攻方 `union.out` 那 100 格（覆盖写 / 顺序写三个数据单元 × k = 3、5 × 已确认 / 没见证 × R 写出的每个单元与共用的实例表）、记录那 4 格、
//! C751 两形各 6 组。同实例开关（Y2）开与关各走一遍：Y5 看两个形态放在一起。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::*;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
use singlefs_core::pointer::NodePointer;
use singlefs_core::r2_switch::set_same_instance;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, instance_table_chain_of_root,
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
    rebuild_version, recover, scan_journal, walk_to_file, JournalPolicy, PoolReader,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;
use std::collections::BTreeMap;

type Key = (u32, u64);

fn key_of(root: &RootRecord) -> Key {
    (root.instance.0, root.checkpoint_txg.0)
}

fn readers_on(reader: &dyn PoolReader, root: &RootRecord) -> (bool, bool, bool) {
    let mut fallbacks = 0usize;
    let walk = walk_to_file(reader, root, &mut fallbacks).is_ok();
    let open = open_pool_for_read(reader, root).is_ok();
    let configuration = choose_system_configuration(reader).expect("择得到系统配置");
    let records = scan_journal(reader, &configuration);
    let own_record = records
        .values()
        .filter(|record| {
            record.instance == root.instance
                && record.checkpoint_txg == root.checkpoint_txg
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
        .cloned();
    let rebuild = rebuild_version(reader, root, own_record).is_ok()
        && instance_table_chain_of_root(reader, root).is_ok();
    (walk, open, rebuild)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Trigger {
    Walk,
    Open,
    Rebuild,
    Any,
    All,
}
const TRIGGERS: [Trigger; 5] = [Trigger::Walk, Trigger::Open, Trigger::Rebuild, Trigger::Any, Trigger::All];

fn failed(trigger: Trigger, readers: (bool, bool, bool)) -> bool {
    match trigger {
        Trigger::Walk => !readers.0,
        Trigger::Open => !readers.1,
        Trigger::Rebuild => !readers.2,
        Trigger::Any => !(readers.0 && readers.1 && readers.2),
        Trigger::All => !readers.0 && !readers.1 && !readers.2,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mount {
    Recovery,
    ReadOnly,
    Writable,
}
const MOUNTS: [Mount; 3] = [Mount::Recovery, Mount::ReadOnly, Mount::Writable];

fn own_ok(mount: Mount, readers: (bool, bool, bool)) -> bool {
    match mount {
        Mount::Recovery => readers.0,
        Mount::ReadOnly => readers.1,
        Mount::Writable => readers.2,
    }
}

fn own_trigger(mount: Mount) -> Trigger {
    match mount {
        Mount::Recovery => Trigger::Walk,
        Mount::ReadOnly => Trigger::Open,
        Mount::Writable => Trigger::Rebuild,
    }
}

/// 一格里算出来的东西：所选根、F2 交回的那一版、两条根各自的三样读者。
struct Cell {
    chosen: RootRecord,
    target: Option<RootRecord>,
    readers_on_chosen: (bool, bool, bool),
    readers_on_target: Option<(bool, bool, bool)>,
}

fn cell_of(reader: &dyn PoolReader) -> Cell {
    let configuration = choose_system_configuration(reader).expect("择得到系统配置");
    let chosen = choose_root(reader, &configuration).expect("择得到根");
    let records = scan_journal(reader, &configuration);
    let target =
        newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
            reader,
            &configuration,
            &chosen,
            &chosen,
            &records,
        );
    let readers_on_chosen = readers_on(reader, &chosen);
    let readers_on_target = target.as_ref().map(|root| readers_on(reader, root));
    Cell { chosen, target, readers_on_chosen, readers_on_target }
}

/// 模型：某一处挂载在「触发取 `trigger`」的统一判法下落到哪一版、成不成。`lenient` 为真时，退过去的那一版自己的读者读不通而所选根自己的读者读得通，留在所选根。
fn model_outcome(cell: &Cell, mount: Mount, trigger: Trigger, lenient: bool) -> (Key, bool) {
    let chosen_key = key_of(&cell.chosen);
    if !failed(trigger, cell.readers_on_chosen) {
        return (chosen_key, own_ok(mount, cell.readers_on_chosen));
    }
    match (&cell.target, cell.readers_on_target) {
        (Some(target), Some(readers_on_target)) => {
            if own_ok(mount, readers_on_target) {
                (key_of(target), true)
            } else if lenient && own_ok(mount, cell.readers_on_chosen) {
                (chosen_key, true)
            } else {
                (key_of(target), false)
            }
        }
        _ => (chosen_key, own_ok(mount, cell.readers_on_chosen)),
    }
}

fn text_of(outcome: (Key, bool)) -> String {
    if outcome.1 { format!("Ok({},{})", outcome.0 .0, outcome.0 .1) } else { "Fail".to_string() }
}

fn real_outcomes(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> [String; 3] {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, .. } => format!("Ok({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::NoFile { root } => format!("Ok({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { .. } => "Fail".to_string(),
    };
    let read_only = match mount_read_only(&devices) {
        Ok(mounted) => format!(
            "Ok({},{})",
            mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0
        ),
        Err(_) => "Fail".to_string(),
    };
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
    let writable = match mount_writable(parameters, &mut devices_of_the_mount) {
        Ok(mounted) => format!(
            "Ok({},{})",
            mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0
        ),
        Err(_) => "Fail".to_string(),
    };
    [recovery, read_only, writable]
}

#[derive(Default)]
struct Aggregate {
    cells: u32,
    calibration_mismatch: u32,
    /// 同实例开关开与关两种下真挂载的结局（恢复、只读、可写）不同的格数。
    same_instance_switch_changes_a_real_outcome: u32,
    /// 触发 × 宽严 → 统计
    per_trigger: BTreeMap<(Trigger, bool), Counts>,
}
#[derive(Default, Clone, Copy)]
struct Counts {
    all_three_land_on_the_same_version: u32,
    some_mount_succeeded_today_and_fails: u32,
    some_mount_failed_today_and_succeeds: u32,
    some_mount_lands_on_another_version: u32,
    mounts_lost: u32,
    mounts_gained: u32,
}

fn run_cell(
    aggregate: &mut Aggregate,
    label: &str,
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
    verbose: bool,
) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let cell = cell_of(&devices);
    set_same_instance(false);
    let real_today = real_outcomes(parameters, image, ranges);
    set_same_instance(true);
    let real_with_same_instance = real_outcomes(parameters, image, ranges);
    set_same_instance(false);
    // 校准：各处取自己那一种触发（今天的规则）的模型结局，与真挂载比。
    let model_today: Vec<(Key, bool)> = MOUNTS
        .iter()
        .map(|mount| model_outcome(&cell, *mount, own_trigger(*mount), false))
        .collect();
    let calibration_matches = (0..3).all(|index| text_of(model_today[index]) == real_today[index]);
    aggregate.cells += 1;
    aggregate.calibration_mismatch += u32::from(!calibration_matches);
    aggregate.same_instance_switch_changes_a_real_outcome +=
        u32::from(real_today != real_with_same_instance);
    for trigger in TRIGGERS {
        for lenient in [false, true] {
            let outcomes: Vec<(Key, bool)> = MOUNTS
                .iter()
                .map(|mount| model_outcome(&cell, *mount, trigger, lenient))
                .collect();
            let counts = aggregate.per_trigger.entry((trigger, lenient)).or_default();
            counts.all_three_land_on_the_same_version +=
                u32::from(outcomes[0].0 == outcomes[1].0 && outcomes[1].0 == outcomes[2].0);
            for index in 0..3 {
                let (today, form) = (model_today[index], outcomes[index]);
                if today.1 && !form.1 {
                    counts.mounts_lost += 1;
                }
                if !today.1 && form.1 {
                    counts.mounts_gained += 1;
                }
                if today.0 != form.0 && (today.1 || form.1) {
                    counts.some_mount_lands_on_another_version += 1;
                }
            }
            counts.some_mount_succeeded_today_and_fails +=
                u32::from((0..3).any(|index| model_today[index].1 && !outcomes[index].1));
            counts.some_mount_failed_today_and_succeeds +=
                u32::from((0..3).any(|index| !model_today[index].1 && outcomes[index].1));
        }
    }
    if verbose || !calibration_matches {
        let per_trigger = TRIGGERS
            .iter()
            .map(|trigger| {
                let texts: Vec<String> = MOUNTS
                    .iter()
                    .map(|mount| text_of(model_outcome(&cell, *mount, *trigger, false)))
                    .collect();
                format!("{trigger:?}[{}]", texts.join(" "))
            })
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "Y3 {label} chosen={:?} f2_target={:?} readers_on_chosen(walk,open,rebuild)={:?} real_today[rec={} ro={} rw={}] real_with_same_instance[rec={} ro={} rw={}] model_today[{}] calibrated={calibration_matches} | {per_trigger}",
            key_of(&cell.chosen),
            cell.target.as_ref().map(key_of),
            cell.readers_on_chosen,
            real_today[0], real_today[1], real_today[2],
            real_with_same_instance[0], real_with_same_instance[1], real_with_same_instance[2],
            model_today.iter().map(|outcome| text_of(*outcome)).collect::<Vec<_>>().join(" "),
        );
    }
}

fn ranges_of_pointer(pointer: &NodePointer, length: u64) -> RangesReadingBackAllZero {
    RangesReadingBackAllZero(
        pointer
            .locations
            .iter()
            .map(|location| (location.device, location.slot.to_device_offset().0, length))
            .collect(),
    )
}

fn newer_system_configuration_slot_of_each_device(
    image: &MemoryPool,
) -> Vec<(DeviceIdentity, u64, u64)> {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let slot_spacing_in_bytes =
        u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
    let slot_bytes =
        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节");
    image
        .devices
        .iter()
        .map(|(device, sparse_device)| {
            let newer_slot_offset = [0, slot_spacing_in_bytes]
                .into_iter()
                .filter_map(|offset| {
                    SystemConfiguration::parse_slot(
                        &sparse_device.read(DeviceOffsetInBytes(offset), slot_bytes),
                    )
                    .ok()
                    .map(|slot| (slot.quantities.slot_generation, offset))
                })
                .max()
                .map(|(_, offset)| offset)
                .expect("每块盘至少一槽自证过");
            (*device, newer_slot_offset, SYSTEM_CONFIGURATION_SLOT_BYTES)
        })
        .collect()
}

fn report(title: &str, aggregate: &Aggregate) {
    println!(
        "Y3_SUMMARY {title} cells={} calibration_mismatch={} same_instance_switch_changes_a_real_outcome={}",
        aggregate.cells, aggregate.calibration_mismatch, aggregate.same_instance_switch_changes_a_real_outcome
    );
    for ((trigger, lenient), counts) in &aggregate.per_trigger {
        println!(
            "Y3_SUMMARY {title} trigger={trigger:?} retreat_target_failure={} all_three_land_on_the_same_version={} cells_where_a_mount_succeeded_today_and_fails={} cells_where_a_mount_failed_today_and_succeeds={} mounts_lost={} mounts_gained={} mounts_on_another_version={}",
            if *lenient { "stay_on_chosen" } else { "report_failure" },
            counts.all_three_land_on_the_same_version,
            counts.some_mount_succeeded_today_and_fails,
            counts.some_mount_failed_today_and_succeeds,
            counts.mounts_lost,
            counts.mounts_gained,
            counts.some_mount_lands_on_another_version,
        );
    }
}

#[test]
fn trigger_kinds_against_the_hundred_cells_the_record_cells_and_the_c751_shapes() {
    let mut confirmed = Aggregate::default();
    let mut unwitnessed = Aggregate::default();
    let mut records = Aggregate::default();
    for (kind_label, kind) in [
        ("ow", PublishRecordedAsR::Overwrite),
        ("sw3", PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
    ] {
        for overwrites in [3u64, 5] {
            let recorded = record_one_publish_after_overwrites(overwrites, kind);
            let root_of_r = recorded.root();
            let parameters = recorded.pool.parameters.clone();
            for (image_label, image) in [
                ("confirmed", recorded.image_with_the_whole_publish_persisted()),
                ("unwitnessed", recorded.crash_image_with_the_root_persisted_and_the_rotation_lost()),
            ] {
                let aggregate = if image_label == "confirmed" { &mut confirmed } else { &mut unwitnessed };
                for (index, group) in recorded.unit_writes_grouped_by_unit().iter().enumerate() {
                    let ranges = RangesReadingBackAllZero::default().with_the_writes(group);
                    run_cell(
                        aggregate,
                        &format!("kind={kind_label} k={overwrites} img={image_label} fault=unit#{index}"),
                        &parameters, &image, &ranges, false,
                    );
                }
                let ranges = ranges_of_pointer(&root_of_r.instance_table, 32768);
                run_cell(
                    aggregate,
                    &format!("kind={kind_label} k={overwrites} img={image_label} fault=instance_table_of_r(shared)"),
                    &parameters, &image, &ranges, true,
                );
                if image_label == "confirmed" {
                    let last = recorded.record_count();
                    let ranges = RangesReadingBackAllZero::default()
                        .with_the_writes(&recorded.record_writes_of_the_record(last));
                    run_cell(
                        &mut records,
                        &format!("kind={kind_label} k={overwrites} img=confirmed fault=last_record_of_r(#{last})"),
                        &parameters, &image, &ranges, true,
                    );
                }
            }
        }
    }
    report("confirmed", &confirmed);
    report("unwitnessed", &unwitnessed);
    report("record_cells", &records);
}

#[test]
fn the_c751_registered_shape_and_the_656_shape_under_the_trigger_kinds() {
    let mut aggregate = Aggregate::default();
    for overwrites in [3u64, 5] {
        for (kind_label, kind) in [
            ("ow", PublishRecordedAsR::Overwrite),
            ("sw3", PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
        ] {
            let recorded = record_one_publish_after_overwrites(overwrites, kind);
            let parameters = recorded.pool.parameters.clone();
            let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let mut ranges =
                RangesReadingBackAllZero(newer_system_configuration_slot_of_each_device(&crash_image))
                    .with_the_writes(&recorded.unit_writes());
            for location in recorded.root().instance_table.locations {
                ranges.0.push((location.device, location.slot.to_device_offset().0, 32768));
            }
            run_cell(
                &mut aggregate,
                &format!("registered_c751 kind={kind_label} k={overwrites}"),
                &parameters, &crash_image, &ranges, true,
            );
        }
    }
    report("c751_registered", &aggregate);
}
