//! 正推腿探针（Q1，第二种造法）：不改盘上字节，让写者那一侧（探针开关放开断言）自己写出「已释放位与隔离位同置」的记录，
//! 走完整的发布路径，journal 记录与根都一致，只剩这一个异常。
mod common;

use common::{build_pool, publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 11 + seed) % 247).expect("小于 256"))
        .collect()
}

fn describe(label: &str, pool: &BuiltPool) {
    let image = pool.memory_pool();
    let verdicts = singlefs_checker::walk::check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    let recovery = recover(&image, JournalPolicy::Consult);
    let outcome = match &recovery.outcome {
        RecoveryOutcome::FileRead { content, .. } => format!("FileRead({} bytes)", content.len()),
        RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("Failed({failure:?})"),
    };
    println!("PROBE-Q1B {label} checker_violated={violated:?} recover={outcome}");
}

fn pool_with_a_released_record_then_a_bit_set_on(
    tag: &str,
    devices_to_set: &[DeviceIdentity],
) -> (BuiltPool, SlotNumber) {
    std::env::set_var("PROBE_ALLOW_BOTH", "1");
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let data_slot = first.data_pointers[0].locations[0].slot;
    let second = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(2500, 17),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    pool.output = second.clone();
    describe("after-B-no-anomaly", &pool);
    for device in devices_to_set {
        let mut record = *pool
            .allocator
            .record_for(*device, data_slot)
            .expect("A 的数据单元的记录");
        assert!(record.is_released, "B 释放了 A 的数据单元");
        record.is_quarantined = true;
        pool.allocator.probe_replace_record(record);
    }
    let third = publish_overwrite_in_process(
        &mut pool,
        &second,
        &content_of(2700, 23),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(1),
    )
    .expect("C");
    pool.output = third;
    (pool, data_slot)
}

#[test]
fn probe_q1b_writer_emits_both_bits_on_both_devices() {
    let (pool, _) = pool_with_a_released_record_then_a_bit_set_on(
        "q1b-both-devices",
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    describe("C-both-bits-on-both-devices", &pool);
}

#[test]
fn probe_q1b_writer_emits_both_bits_on_one_device() {
    let (pool, _) =
        pool_with_a_released_record_then_a_bit_set_on("q1b-one-device", &[DeviceIdentity(0)]);
    describe("C-both-bits-on-device-0-only", &pool);
}
