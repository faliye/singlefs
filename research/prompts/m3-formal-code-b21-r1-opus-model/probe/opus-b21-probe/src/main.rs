//! 攻方副本探针入口（m3-formal-code-b21-r1 opus）。子命令：k3、k2a、k2b、k2b-sweep、f3、f3-arith。
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
use singlefs_harness::model_comparison::refusal_reason_of_publish_error;

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
        let mut current = session
            .current
            .version_of_trees()
            .ok_or("no trees")?
            .clone();
        let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
        current = publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            write_time_seconds,
            session.instance,
        )
        .map_err(|e| format!("intent {e:?}"))?;
        for _ in 0..batches {
            current = publish_unlink_step(
                &mut writer,
                &mut session.allocator,
                &current,
                InodeNumber(FIRST_INODE_NUMBER),
                UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).unwrap()),
                session.instance,
            )
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

#[derive(Clone, Copy)]
enum Attempt {
    NewInodes(u64),
    Overwrite,
}

fn attempt(image: &MemoryPool, what: Attempt) -> String {
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let mut devices = plain_devices_on(image);
    let mounted = match mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return format!("mount-refused:{:?}", refusal_reason_of_mount(&e)),
    };
    let PoolVersion::WithFile(current) = &mounted.current else {
        return "not-with-file".to_string();
    };
    let mut allocator = mounted.allocator;
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let content = vec![7u8; 100];
    let result = match what {
        Attempt::NewInodes(n) => publish_new_inodes(&mut writer, &mut allocator, current, n, 1_788_000_000, mounted.output.instance).map(|_| ()),
        Attempt::Overwrite => publish_overwrite(&mut writer, &mut allocator, current, FirstFile { content: &content, write_time_seconds: 1_788_000_000 }, mounted.output.instance).map(|_| ()),
    };
    match result {
        Ok(()) => "Published".to_string(),
        Err(e) => {
            if std::env::var("OPUS_MEMBER").is_ok() {
                format!("{:?}|{}", refusal_reason_of_publish_error(&e), format!("{e:?}").chars().take(160).collect::<String>().replace(' ', "_"))
            } else {
                format!("{:?}", refusal_reason_of_publish_error(&e))
            }
        }
    }
}

fn refusal_reason_of_mount(e: &singlefs_core::mount::MountError) -> String {
    let text = format!("{e:?}");
    text.chars().take(60).collect()
}

fn describe(slots: u64, overwrites: usize, units: usize, batches: usize) -> String {
    let image = match crash_image(slots, overwrites, units, batches) {
        Ok(image) => image,
        Err(e) => return format!("slots={slots} ow={overwrites} units={units} batches={batches} build-failed:{}", e.chars().take(80).collect::<String>()),
    };
    let parameters = HistoryDeviceWidth::UnitAreaOf384Slots.parameters();
    let (continuation, nlink) = {
        let mut devices = plain_devices_on(&image);
        match mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
            Ok(mounted) => {
                let short = matches!(
                    mounted.output.deletion_queue_continuation_outcome,
                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }
                );
                let nlink = match &mounted.current {
                    PoolVersion::WithFile(current) => format!("{:?}", current.inode_record.map(|r| r.nlink)),
                    _ => "not-with-file".to_string(),
                };
                (if short { "short".to_string() } else { format!("{:?}", mounted.output.deletion_queue_continuation_outcome).chars().take(40).collect() }, nlink)
            }
            Err(e) => (format!("mount-refused:{}", refusal_reason_of_mount(&e)), "-".to_string()),
        }
    };
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let run = mount_and_publish_once_on_the_crash_image(
        &image,
        &parameters,
        execution,
        HistorySeed(0),
        &CrashPoint { segment_index: 0, persisted_within_the_segment: Vec::new(), step: StepPosition::StartingPoint, operation_kind: None },
    );
    let arm = match &run.outcome {
        WritableMountOutcome::Published => "Published".to_string(),
        WritableMountOutcome::PublishRefused { reason, member } => if std::env::var("OPUS_MEMBER").is_ok() { format!("{reason:?}|{}", member.chars().take(160).collect::<String>().replace(' ', "_")) } else { format!("{reason:?}") },
        WritableMountOutcome::MountRefused { reason, .. } => format!("mount-refused:{reason:?}"),
    };
    format!(
        "slots={slots} ow={overwrites} units={units} batches={batches} continuation={continuation} nlink={nlink} arm={arm} create1={} create0={} create2={} overwrite={}",
        attempt(&image, Attempt::NewInodes(1)),
        attempt(&image, Attempt::NewInodes(0)),
        attempt(&image, Attempt::NewInodes(2)),
        attempt(&image, Attempt::Overwrite),
    )
}

const GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

fn inverse(a: u64) -> u64 {
    // 奇数在 2^64 下的乘法逆（牛顿迭代）。
    let mut x = a;
    for _ in 0..6 {
        x = x.wrapping_mul(2u64.wrapping_sub(a.wrapping_mul(x)));
    }
    x
}

fn f3_arith() {
    let (salt1, salt2) = opus_probe::salts();
    let gamma_inverse = inverse(GAMMA);
    assert_eq!(GAMMA.wrapping_mul(gamma_inverse), 1);
    // 段号 0..256，第 s 段长 1 + s % 16，每段全部子集（含全持久，给上界）。
    let mut keys: Vec<(u64, u32, u64)> = Vec::new();
    for segment in 0u64..256 {
        let length = 1 + (segment % 16) as usize;
        for mask in 0u64..(1u64 << length) {
            let persisted: Vec<bool> = (0..length).map(|bit| mask & (1 << bit) != 0).collect();
            keys.push((segment ^ opus_probe::fingerprint(&persisted), segment as u32, mask));
        }
    }
    let total = keys.len();
    let mut sorted: Vec<u64> = keys.iter().map(|k| k.0).collect();
    sorted.sort_unstable();
    sorted.dedup();
    println!("f3-arith keys(segment xor fingerprint)={total} distinct={}", sorted.len());
    // 低位：指纹最低位 = 起始值最低位 xor 子集持久个数的奇偶。
    let parity_ok = keys.iter().all(|(key, segment, mask)| {
        ((key ^ u64::from(*segment)) & 1) == ((0xcbf2_9ce4_8422_2325u64 & 1) ^ u64::from(mask.count_ones() & 1))
    });
    println!("f3-arith lowest-bit-is-parity={parity_ok}");
    for seed in [0u64, 7, 32768, 245760, 0xdead_beef] {
        // SplitMix64 的状态 t 步之后是 seed + t·γ：两个起始状态 a、b 的流互为平移，平移量 (b − a)·γ⁻¹。
        let mut positions: Vec<u64> = keys
            .iter()
            .map(|(key, _, _)| (seed ^ salt2 ^ key).wrapping_mul(gamma_inverse))
            .collect();
        positions.sort_unstable();
        let min_gap = positions.windows(2).map(|w| w[1] - w[0]).min().unwrap();
        let references = [("history", seed), ("first-crash", seed ^ salt1)];
        let mut nearest_reference = u64::MAX;
        for (_, reference) in references {
            let p = reference.wrapping_mul(gamma_inverse);
            for q in &positions {
                nearest_reference = nearest_reference.min(p.wrapping_sub(*q)).min(q.wrapping_sub(p));
            }
        }
        println!("f3-arith seed={seed} min_shift_between_two_second_crash_streams={min_gap} (log2 {:.1}) min_shift_to_history_or_first_crash_stream={nearest_reference} (log2 {:.1})", (min_gap as f64).log2(), (nearest_reference as f64).log2());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("k3") => print!("{}", opus_probe::k3_run(&args[2], &args[3])),
        Some("k3-sweep") => print!("{}", opus_probe::k3_sweep(args[2].parse().unwrap(), args[3].parse().unwrap(), args[4].parse().unwrap())),
        Some("k2a") => print!("{}", opus_probe::k2a_run(&args[2])),
        Some("k2b") => {
            let n = |i: usize| args[i].parse::<u64>().unwrap();
            println!("{}", describe(n(2), n(3) as usize, n(4) as usize, n(5) as usize));
        }
        Some("k2b-sweep") => {
            let from: u64 = args[2].parse().unwrap();
            let to: u64 = args[3].parse().unwrap();
            let step: u64 = args[4].parse().unwrap();
            let overwrites: usize = args[5].parse().unwrap();
            let units: usize = args[6].parse().unwrap();
            let batches: usize = args[7].parse().unwrap();
            let mut slots = from;
            while slots <= to {
                println!("{}", describe(slots, overwrites, units, batches));
                slots += step;
            }
        }
        Some("f3") => println!("{}", opus_probe::f3_run(args[2].parse().unwrap(), args[3].parse().unwrap(), args[4].parse().unwrap())),
        Some("f3-arith") => f3_arith(),
        Some("f3-inline") => println!("{}", opus_probe::f3_inline()),
        _ => eprintln!("用法：k3 <control|ff-record-lost> <with-rotation|no-rotation> | k2a <variant> | k2b <slots> <ow> <units> <batches> | k2b-sweep <from> <to> <step> <ow> <units> <batches> | f3 <seed> <segment_index> <segment_length> | f3-arith"),
    }
}
