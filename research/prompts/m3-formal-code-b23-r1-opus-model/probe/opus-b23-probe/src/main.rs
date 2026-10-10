//! 攻方副本探针入口（m3-formal-code-b23-r1 opus）。子命令：k3a、f3-bitwise、f3-segments、f3-variants、a1、b1、b1-sweep。
#![allow(clippy::all)]

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    OVERWRITE_BYTES,
};
use singlefs_checker_tier::crash_injection::opus_probe;
use singlefs_checker_tier::crash_injection::{
    mount_and_publish_once_on_the_crash_image, CrashPoint, WritableMountOutcome,
};
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_space_admission, DeletionQueueContinuationOutcome};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::transaction::{
    publish_new_inodes, publish_overwrite, publish_unlink_intent, publish_unlink_step, FirstFile,
    PoolVersion, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::{
    HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker, StepPosition,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
use singlefs_harness::model_comparison::refusal_reason_of_publish_error;

/// 与入库 B1 用例 `crash_image_between_the_deletion_batches` 同一造法，四个数可调。
fn crash_image(slots: u64, overwrites: usize, units: usize, batches: usize) -> Result<MemoryPool, String> {
    let built = std::panic::catch_unwind(|| {
        let mut pool =
            start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
                HistoryDeviceWidth::UnitAreaOf384Slots,
                slots,
            );
        for _ in 0..overwrites {
            pool.overwrite(OVERWRITE_BYTES).map_err(|e| format!("overwrite {e:?}"))?;
        }
        pool.sequential_write((units - 1) * data_unit_payload_capacity() + 100)
            .map_err(|e| format!("sequential {e:?}"))?;
        let parameters = pool.parameters.clone();
        let write_time_seconds = pool.write_time_seconds;
        let session = pool.session.as_mut().ok_or("no session")?;
        let mut current = session.current.version_of_trees().ok_or("no trees")?.clone();
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), write_time_seconds, session.instance)
            .map_err(|e| format!("intent {e:?}"))?;
        for _ in 0..batches {
            current = publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER),
                UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).unwrap()), session.instance)
                .map_err(|e| format!("batch {e:?}"))?;
        }
        drop(writer);
        pool.session = None;
        Ok::<MemoryPool, String>(pool.image())
    });
    match built {
        Ok(result) => result,
        Err(_) => Err("panic while building".to_string()),
    }
}

fn short_member(text: &str) -> String {
    // 只留拒的种类与可用 / 需要几个数
    let kind: String = text.chars().take_while(|c| *c != '(' && *c != ' ' && *c != '{').collect();
    let nums: Vec<&str> = ["available: AvailableBytesOnOneDevice(", "demand: BytesOnOneDevice("]
        .iter()
        .filter_map(|key| text.find(key).map(|i| &text[i + key.len()..]))
        .map(|rest| rest.split(|c: char| !c.is_ascii_digit()).next().unwrap_or(""))
        .collect();
    format!("{kind}{nums:?}")
}

fn attempt(image: &MemoryPool, count: Option<u64>) -> String {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let mut devices = plain_devices_on(image);
    let mounted = match mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return format!("mount-refused:{}", format!("{e:?}").chars().take(40).collect::<String>()),
    };
    let PoolVersion::WithFile(current) = &mounted.current else { return "not-with-file".to_string() };
    let mut allocator = mounted.allocator;
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let content = vec![7u8; 100];
    let result = match count {
        Some(n) => publish_new_inodes(&mut writer, &mut allocator, current, n, 1_788_000_000, mounted.output.instance).map(|_| ()),
        None => publish_overwrite(&mut writer, &mut allocator, current, FirstFile { content: &content, write_time_seconds: 1_788_000_000 }, mounted.output.instance).map(|_| ()),
    };
    match result {
        Ok(()) => "Published".to_string(),
        Err(e) => if std::env::var("OPUS_FULL").is_ok() { format!("{:?}/{e:?}", refusal_reason_of_publish_error(&e)) } else { format!("{:?}/{}", refusal_reason_of_publish_error(&e), short_member(&format!("{e:?}"))) },
    }
}

/// 入库 B1 用例在这份盘面上判什么：前提（续做推满仍放不下、带文件、nlink = 0）与最后那条断言；`env` 是开关（判法只看 Some、那一支建 0 个）。
fn b1_test_verdict(image: &MemoryPool, env: &[(&str, &str)]) -> String {
    for (k, v) in env { std::env::set_var(k, v); }
    let execution = HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: HistoryDeviceWidth::UnitAreaOf384Slots, space_admission: SpaceAdmission::JudgedByTheFormula };
    let parameters = execution.device_width.parameters();
    let verdict = (|| {
        let mut devices = plain_devices_on(image);
        let mounted = match mount_writable_with_space_admission(&parameters, &mut devices, execution.space_admission) {
            Ok(m) => m,
            Err(_) => return "red(pre:mount)".to_string(),
        };
        if !matches!(mounted.output.deletion_queue_continuation_outcome, DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }) {
            return "red(pre:continuation)".to_string();
        }
        let PoolVersion::WithFile(current) = &mounted.current else { return "red(pre:with-file)".to_string() };
        if current.inode_record.map(|r| r.nlink) != Some(NLINK_OF_AN_INODE_BEING_DELETED) {
            return "red(pre:nlink)".to_string();
        }
        let run = mount_and_publish_once_on_the_crash_image(image, &parameters, execution, HistorySeed(0),
            &CrashPoint { segment_index: 0, persisted_within_the_segment: Vec::new(), step: StepPosition::StartingPoint, operation_kind: None });
        match &run.outcome {
            WritableMountOutcome::PublishRefused { reason, .. } if *reason == ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall) => "green".to_string(),
            WritableMountOutcome::PublishRefused { reason, .. } => format!("red(reason:{reason:?})"),
            other => format!("red(outcome:{})", format!("{other:?}").chars().take(20).collect::<String>()),
        }
    })();
    for (k, _) in env { std::env::remove_var(k); }
    verdict
}

fn b1(slots: u64, overwrites: usize, units: usize, batches: usize) -> String {
    let image = match crash_image(slots, overwrites, units, batches) {
        Ok(image) => image,
        Err(e) => return format!("b1 slots={slots} ow={overwrites} units={units} batches={batches} build-failed:{}", e.chars().take(60).collect::<String>()),
    };
    let correct = b1_test_verdict(&image, &[]);
    let some = b1_test_verdict(&image, &[("OPUS_K2B_SOME", "1")]);
    let zero = b1_test_verdict(&image, &[("OPUS_ARM", "count-zero")]);
    let class = match (correct.as_str(), some.starts_with("red"), zero.starts_with("red")) {
        ("green", true, true) => "GREEN-KILLS-BOTH",
        ("green", true, false) => "GREEN-KILLS-SOME-ONLY",
        ("green", false, true) => "GREEN-KILLS-ZERO-ONLY",
        ("green", false, false) => "GREEN-KILLS-NONE",
        _ => "TEST-RED-ON-CORRECT",
    };
    format!(
        "b1 slots={slots} ow={overwrites} units={units} batches={batches} class={class} test(correct)={correct} test(only-Some)={some} test(arm-builds-0)={zero} create0={} create1={} create2={} overwrite={}",
        attempt(&image, Some(0)), attempt(&image, Some(1)), attempt(&image, Some(2)), attempt(&image, None)
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = |i: usize| args[i].parse::<u64>().unwrap();
    match args.get(1).map(String::as_str) {
        Some("k3a") => println!("{}", opus_probe::k3a(&args[2], &args[3], n(4) as usize)),
        Some("f3-bitwise") => println!("{}", opus_probe::f3_bitwise(n(2) as usize)),
        Some("f3-segments") => println!("{}", opus_probe::f3_segments(&args[2], n(3), n(4) as usize)),
        Some("f3-variants") => print!("{}", opus_probe::f3_variants(&args[2], n(3), n(4) as usize)),
        Some("f3-fix") => print!("{}", opus_probe::f3_fix()),
        Some("a1") => println!("{}", opus_probe::a1()),
        Some("b1") => println!("{}", b1(n(2), n(3) as usize, n(4) as usize, n(5) as usize)),
        Some("b1-sweep") => {
            // b1-sweep <slots-from> <slots-to> <ow> <units> <batches>
            for slots in n(2)..=n(3) {
                println!("{}", b1(slots, n(4) as usize, n(5) as usize, n(6) as usize));
            }
        }
        _ => eprintln!("用法：k3a <宽度> <长度> <覆盖写次数> | f3-bitwise <样本数> | f3-segments <宽度> <历史数> <步数> | f3-variants <宽度> <历史数> <步数> | a1 | b1 <槽> <覆盖写> <单元> <批> | b1-sweep <槽起> <槽止> <覆盖写> <单元> <批>"),
    }
}
