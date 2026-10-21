//! 攻方模型（m3-close-rootring-r1，云端攻方腿，草稿目录里的仓副本）：C755 用户选的支「三处看同一种失败触发退回，所选那一版建得出
//! （走读、打开、建上一版三样都读的单元的并集都读得出）才不退」。同一份盘面上先量今天三处的结局，再在测试里按那一支算统一之后的结局：
//! 并集建得出 ⇒ 落所选那一版；建不出 ⇒ 交 F2 判法，F2 交回的那一版并集也建得出 ⇒ 落它，否则照报失败。
//! 扫的是：录下来的 R 写出的每一个单元（两份同时读回全零，一样故障）与 R 的根指着的实例表（R 没重写它），R 已确认与 R 没见证两种盘面。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, record_one_publish_after_overwrites, PublishRecordedAsR,
    RangesReadingBackAllZero, RecordedPublish,
};
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_system_configuration,
    newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to,
    rebuild_version, recover, scan_journal, walk_to_file, JournalPolicy, PoolReader,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::MemoryPool;

fn key_of(root: &RootRecord) -> (u32, u64) {
    (root.instance.0, root.checkpoint_txg.0)
}

/// 三样读者各自读得通不通（走读、打开、建上一版），给统一之后的「建得出」用。
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
    let rebuild = rebuild_version(reader, root, own_record).is_ok();
    (walk, open, rebuild)
}

/// 统一之后（并集）落哪一版：`None` 是照报失败。
fn unified_outcome(reader: &dyn PoolReader, chosen: &RootRecord) -> Option<(u32, u64)> {
    let (walk, open, rebuild) = readers_on(reader, chosen);
    if walk && open && rebuild {
        return Some(key_of(chosen));
    }
    let configuration = choose_system_configuration(reader).expect("择得到系统配置");
    let records = scan_journal(reader, &configuration);
    let fallen_back_to =
        newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to(
            reader,
            &configuration,
            chosen,
            chosen,
            &records,
        )?;
    let (walk, open, rebuild) = readers_on(reader, &fallen_back_to);
    (walk && open && rebuild).then(|| key_of(&fallen_back_to))
}

fn describe_recovery(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::FileRead { root, .. } => format!("Ok({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::NoFile { root } => format!("NoFile({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { .. } => "Fail".to_string(),
    }
}

fn describe(outcome: Option<(u32, u64)>) -> String {
    outcome.map_or_else(|| "Fail".to_string(), |(instance, txg)| format!("Ok({instance},{txg})"))
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

/// 一格：今天三处各落哪一版、统一之后落哪一版，打一行；交回（今天只读或可写挂得上所选那一版、统一之后照报失败）这一条件真不真。
fn one_cell(
    label: &str,
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> (bool, bool) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let configuration = choose_system_configuration(&devices).expect("择得到系统配置");
    let chosen = singlefs_core::recovery::choose_root(&devices, &configuration).expect("择得到根");
    let recovery = describe_recovery(&recover(&devices, JournalPolicy::Consult).outcome);
    let read_only = match mount_read_only(&devices) {
        Ok(mounted) => describe(Some(key_of(&mounted.effective_root))),
        Err(_) => "Fail".to_string(),
    };
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
    let writable = match mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount) {
        Ok(mounted) => describe(Some((
            mounted.output.effective_root.instance.0,
            mounted.output.effective_root.checkpoint_txg.0,
        ))),
        Err(_) => "Fail".to_string(),
    };
    let (walk, open, rebuild) = readers_on(&devices, &chosen);
    let unified = unified_outcome(&devices, &chosen);
    let chosen_text = describe(Some(key_of(&chosen)));
    let mount_lost = (read_only == chosen_text || writable == chosen_text) && unified.is_none();
    let read_only_lost = read_only == chosen_text && unified.is_none();
    println!(
        "UNION {label} chosen={chosen_text} readers(walk,open,rebuild)=({walk},{open},{rebuild}) today rec={recovery} ro={read_only} rw={writable} | unified={} mount_lost={mount_lost} read_only_lost={read_only_lost}",
        describe(unified)
    );
    (mount_lost, read_only_lost)
}

/// 记录那一格：R 的末条记录（两份）读回全零，一样故障。条款写的并集是「单元」，记录不是单元；建上一版要一条记录顶着这一版，两种读法都打出来。
#[test]
fn the_last_record_of_a_confirmed_r_reading_back_zero_against_the_union() {
    for (kind_label, kind) in [
        ("ow", PublishRecordedAsR::Overwrite),
        ("sw3", PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
    ] {
        for overwrites in [3u64, 5] {
            let recorded = record_one_publish_after_overwrites(overwrites, kind);
            let image = recorded.image_with_the_whole_publish_persisted();
            let last = recorded.record_count();
            let ranges = RangesReadingBackAllZero::default()
                .with_the_writes(&recorded.record_writes_of_the_record(last));
            let label = format!("kind={kind_label} k={overwrites} img=confirmed fault=last_record_of_r(#{last})");
            one_cell(&label, &recorded, &image, &ranges);
        }
    }
}
