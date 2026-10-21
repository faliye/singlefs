//! 攻方模型（m3-close-rootring-r2，云端攻方腿，草稿目录里的仓副本，攻补丁前的 `crates/`）：C755 这一轮的形态「三处用同一种失败判退不退、
//! 退到哪一版；落到那一版之后恢复、只读挂载、可写挂载成不成各照今天」。同一种失败取四种候选（走读、打开、建上一版、三样的并集）逐种现算，
//! 与今天（每处看自己那一样读者）逐处比。盘面：第一轮攻方那一族（覆盖写或顺序写三个数据单元的 R，k = 3、5），三种：
//! `witnessed`（R 整次落盘，系统配置见证 R）、`witness_lost`（R 整次落盘、已确认，读的那一刻每块盘较新的系统配置槽读回全零）、
//! `unwitnessed`（崩在 R 的根槽之后、轮换之前）；故障：R 写出的每一个单元两份读回全零，或 R 的根指着的实例表（R 没重写它）。
//! 「今天」在模型里也现算一遍（`own`），与真跑的恢复、只读挂载逐格比（`model_matches_today`），对不上的格不进汇总。
mod common_unwitnessed_publish;
mod rootring_r2_common;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, record_one_publish_after_overwrites, PublishRecordedAsR,
    RangesReadingBackAllZero,
};
use rootring_r2_common::{
    describe_three, key_of, newer_system_configuration_slot_of_each_device, three_places,
    trigger_label, Trigger, UNIFIED_TRIGGERS,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{choose_root, choose_system_configuration, recover, JournalPolicy, RecoveryOutcome};
use std::collections::BTreeMap;

/// 一处从今天到统一之后怎么变。
fn change(today: ((u32, u64), bool), unified: ((u32, u64), bool)) -> &'static str {
    let ((_, today_txg), today_ok) = today;
    let ((_, unified_txg), unified_ok) = unified;
    match (today_ok, unified_ok) {
        (true, true) if unified_txg == today_txg => "same",
        (true, true) if unified_txg < today_txg => "older_ok",
        (true, true) => "newer_ok",
        (true, false) => "ok_to_fail",
        (false, true) => "fail_to_ok",
        (false, false) => "same",
    }
}

#[test]
fn every_candidate_for_the_one_failure_against_today_on_single_unit_faults() {
    // (盘面, 候选, 开关, 处, 变化) → 格数。
    let mut tally: BTreeMap<(String, String, &'static str, &'static str, &'static str), u32> = BTreeMap::new();
    let mut mismatches = 0u32;
    let mut cells = 0u32;
    for (kind_label, kind) in [
        ("ow", PublishRecordedAsR::Overwrite),
        ("sw3", PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
    ] {
        for overwrites in [3u64, 5] {
            let recorded = record_one_publish_after_overwrites(overwrites, kind);
            let root_of_r = recorded.root();
            let whole = recorded.image_with_the_whole_publish_persisted();
            let witness_lost_ranges = newer_system_configuration_slot_of_each_device(&whole);
            for (image_label, image, base_ranges) in [
                ("witnessed", whole.clone(), Vec::new()),
                ("witness_lost", whole.clone(), witness_lost_ranges.clone()),
                ("unwitnessed", recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(), Vec::new()),
            ] {
                let mut faults: Vec<(String, RangesReadingBackAllZero)> = recorded
                    .unit_writes_grouped_by_unit()
                    .iter()
                    .enumerate()
                    .map(|(index, group)| {
                        (
                            format!("unit#{index}:len{}", group[0].length_in_bytes()),
                            RangesReadingBackAllZero(base_ranges.clone()).with_the_writes(group),
                        )
                    })
                    .collect();
                let mut table = base_ranges.clone();
                table.extend(root_of_r.instance_table.locations.iter().map(|location| {
                    (location.device, location.slot.to_device_offset().0, 32768)
                }));
                faults.push(("instance_table_of_r(shared)".to_string(), RangesReadingBackAllZero(table)));
                for (fault_label, ranges) in faults {
                    let devices = devices_reading_back_all_zero_on(&image, &ranges);
                    let configuration = choose_system_configuration(&devices).expect("择得到系统配置");
                    let chosen = choose_root(&devices, &configuration).expect("择得到根");
                    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
                        RecoveryOutcome::FileRead { root, .. } => Some((root.0 .0, root.1 .0)),
                        RecoveryOutcome::NoFile { root } => Some((root.0 .0, root.1 .0)),
                        RecoveryOutcome::Failed { .. } => None,
                    };
                    let read_only = mount_read_only(&devices)
                        .ok()
                        .map(|mounted| key_of(&mounted.effective_root));
                    let mut devices_of_the_mount = devices_reading_back_all_zero_on(&image, &ranges);
                    let writable = mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount)
                        .ok()
                        .map(|mounted| key_of(&mounted.output.effective_root));
                    let own = three_places(&devices, &chosen, Trigger::OwnReader, false);
                    let as_today = |outcome: ((u32, u64), bool)| outcome.1.then_some(outcome.0);
                    let model_matches_today =
                        as_today(own[0]) == recovery && as_today(own[1]) == read_only;
                    let writable_matches = as_today(own[2]) == writable;
                    cells += 1;
                    mismatches += u32::from(!model_matches_today);
                    let mut line = format!(
                        "R2UNIFY kind={kind_label} k={overwrites} img={image_label} fault={fault_label} chosen=({},{}) today rec={recovery:?} ro={read_only:?} rw={writable:?} model_matches_today={model_matches_today} writable_matches={writable_matches} | own: {}",
                        chosen.instance.0,
                        chosen.checkpoint_txg.0,
                        describe_three(&own)
                    );
                    for (kept, rule) in [(false, "exact"), (true, "same")] {
                        for trigger in UNIFIED_TRIGGERS {
                            let unified = three_places(&devices, &chosen, trigger, kept);
                            line.push_str(&format!(" | {}/{rule}: {}", trigger_label(trigger), describe_three(&unified)));
                            if model_matches_today {
                                for (place_index, place) in ["rec", "ro", "rw"].into_iter().enumerate() {
                                    *tally
                                        .entry((
                                            image_label.to_string(),
                                            trigger_label(trigger).to_string(),
                                            rule,
                                            place,
                                            change(own[place_index], unified[place_index]),
                                        ))
                                        .or_insert(0) += 1;
                                }
                            }
                        }
                    }
                    println!("{line}");
                }
            }
        }
    }
    for ((image, trigger, rule, place, what), count) in &tally {
        if *what != "same" {
            println!("R2UNIFY_TALLY img={image} trigger={trigger} rule={rule} place={place} change={what} cells={count}");
        }
    }
    println!("R2UNIFY_SUMMARY cells={cells} model_mismatches={mismatches}");
}
