//! 正推腿探针（草稿副本里的，不进仓）：C694 的 C/B 与 I-7.4 的扫描。
//! 沿用 `common_unwitnessed_publish` 的造法：R 录在第一个文件之后覆盖写 k 次之后，崩在 R 的根槽落盘之后、系统配置轮换之前。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::{
    checker_violations_on, devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    plain_devices_on, pool_after_the_mount, record_one_publish_after_overwrites,
    units_of_r_overwritten_on, PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
    OVERWRITE_BYTES,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, unmount, ShadowLedger,
};

type Ranges = fn(&RecordedPublish) -> RangesReadingBackAllZero;

fn root_and_units(recorded: &RecordedPublish) -> RangesReadingBackAllZero {
    RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes())
}
fn root_only(recorded: &RecordedPublish) -> RangesReadingBackAllZero {
    RangesReadingBackAllZero::default().with_the_writes(&[recorded.root_slot_write()])
}
fn units_only(recorded: &RecordedPublish) -> RangesReadingBackAllZero {
    RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes())
}

fn i74(image: &singlefs_harness::memory_pool::MemoryPool) -> String {
    let violations: Vec<String> = checker_violations_on(image)
        .into_iter()
        .filter(|line| line.starts_with("I-7.4"))
        .collect();
    if violations.is_empty() {
        "-".to_string()
    } else {
        format!("RED({})", violations.len())
    }
}

#[test]
fn sweep_after_the_first_mount() {
    let variants: Vec<(&str, Ranges, PublishRecordedAsR)> = vec![
        ("ur/over", root_and_units, PublishRecordedAsR::Overwrite),
        ("root/over", root_only, PublishRecordedAsR::Overwrite),
        ("units/over", units_only, PublishRecordedAsR::Overwrite),
        ("ur/seq3", root_and_units, PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
        ("root/seq3", root_only, PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
    ];
    for (name, ranges, kind) in variants {
        for k in [3u64, 5, 7] {
            let recorded = record_one_publish_after_overwrites(k, kind);
            let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            println!("VARIANT {name} k={k} I74_on_crash_image={}", i74(&crash_image));
            let mut devices = devices_reading_back_all_zero_on(&crash_image, &ranges(&recorded));
            let mounted = match mount_writable(&recorded.pool.parameters, &mut devices) {
                Ok(mounted) => mounted,
                Err(error) => {
                    println!("  M1 refused: {error:?}");
                    continue;
                }
            };
            let image_after = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
            println!(
                "  M1 ok effective_root={:?} I74={} R_units_overwritten={}",
                (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg),
                i74(&image_after),
                units_of_r_overwritten_on(&recorded, &image_after).len()
            );
            let mut pool = pool_after_the_mount(&recorded, &image_after, mounted);
            let mut line = String::new();
            for step in 0..=30u32 {
                if step > 0 {
                    if let Err(error) = pool.overwrite(OVERWRITE_BYTES) {
                        println!("  overwrite {step} refused: {error:?}");
                        break;
                    }
                }
                let image = pool.image();
                let overwritten = units_of_r_overwritten_on(&recorded, &image).len();
                let verdict = i74(&image);
                line.push_str(&format!("{step}:{verdict}/{overwritten} "));
                if step % 10 == 0 {
                    let session = pool.session();
                    let mut d1 = plain_devices_on(&image);
                    let mut a1 = session.allocator.clone();
                    let mut c1 = session.current.clone();
                    let unmounted = unmount(&recorded.pool.parameters, &mut d1, &mut a1, &mut c1, ShadowLedger::On);
                    let mut d2 = plain_devices_on(&image);
                    let mut a2 = session.allocator.clone();
                    let mut c2 = session.current.file_version().expect("带文件").clone();
                    let raised = raise_rollback_floor_to_the_admission_ceiling(
                        &recorded.pool.parameters, &mut d2, &mut a2, &mut c2, ShadowLedger::On,
                    );
                    let mut d3 = plain_devices_on(&image);
                    let remounted = mount_writable(&recorded.pool.parameters, &mut d3);
                    println!(
                        "  step {step}: unmount={} raise={} remount={} I74_now={verdict} R_units_overwritten={overwritten}",
                        unmounted.as_ref().map(|_| "ok".to_string()).unwrap_or_else(|e| format!("{e:?}").chars().take(60).collect()),
                        raised.as_ref().map(|_| "ok".to_string()).unwrap_or_else(|e| format!("{e:?}").chars().take(60).collect()),
                        remounted.as_ref().map(|_| "ok".to_string()).unwrap_or_else(|e| format!("{e:?}").chars().take(60).collect()),
                    );
                }
            }
            println!("  SWEEP {line}");
        }
    }
}
