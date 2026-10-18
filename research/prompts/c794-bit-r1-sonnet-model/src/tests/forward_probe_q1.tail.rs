
// ───────── 正推腿探针（Q1）：已释放位与隔离位同置的记录，三处读者各怎么认 ─────────

fn probe_memory_pool_of(devices: &Devices) -> singlefs_harness::memory_pool::MemoryPool {
    singlefs_harness::memory_pool::MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: DEVICE_BYTES,
    }
}

fn probe_describe(label: &str, devices: &Devices) {
    use singlefs_checker::image::InvariantVerdict;
    let pool = probe_memory_pool_of(devices);
    let verdicts = singlefs_checker::walk::check_pool_image(&pool);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    let i31 = verdicts.iter().find(|(n, _)| *n == "I-3.1").map(|(_, v)| format!("{v:?}"));
    let i311 = verdicts.iter().find(|(n, _)| *n == "I-3.11").map(|(_, v)| format!("{v:?}"));
    let i39 = verdicts.iter().find(|(n, _)| *n == "I-3.9").map(|(_, v)| format!("{v:?}"));
    let i11 = verdicts.iter().find(|(n, _)| *n == "I-1.1").map(|(_, v)| format!("{v:?}"));
    println!(
        "PROBE-Q1 {label} checker_violated={violated:?} I-1.1={i11:?} I-3.1={i31:?} I-3.9={i39:?} I-3.11={i311:?}"
    );
    let recovery = singlefs_core::recovery::recover(&pool, singlefs_core::recovery::JournalPolicy::Consult);
    let outcome = match &recovery.outcome {
        singlefs_core::recovery::RecoveryOutcome::FileRead { content, .. } => format!("FileRead({} bytes)", content.len()),
        singlefs_core::recovery::RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
        singlefs_core::recovery::RecoveryOutcome::Failed { failure, .. } => format!("Failed({failure:?})"),
    };
    println!("PROBE-Q1 {label} recover_outcome={outcome}");
}

#[test]
fn probe_q1_released_and_quarantine_bits_together() {
    let (mut devices, _first) = pool_after_the_first_file();
    probe_describe("baseline-unmodified", &devices);
    let changed = rewrite_allocation_record_entries_of_the_newest_root(
        &mut devices,
        |record| record.is_released,
        |entry| {
            let span_field = u16::from_le_bytes([entry[10], entry[11]]);
            entry[10..12].copy_from_slice(
                &(span_field | singlefs_core::allocator::ALLOCATION_RECORD_QUARANTINED_FLAG).to_le_bytes(),
            );
        },
    );
    println!("PROBE-Q1 changed_records={changed}");
    probe_describe("both-bits-on-all-released-records", &devices);
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    println!("PROBE-Q1 writable_mount_refusal={refusal:?}");
}

/// 只改一条（盘 0 上的一条已释放记录），另一盘不动：两位同置只在一块盘上。
#[test]
fn probe_q1_released_and_quarantine_bits_together_on_one_device_only() {
    let (mut devices, _first) = pool_after_the_first_file();
    let changed = rewrite_allocation_record_entries_of_the_newest_root(
        &mut devices,
        |record| record.is_released && record.device == DeviceIdentity(0),
        |entry| {
            let span_field = u16::from_le_bytes([entry[10], entry[11]]);
            entry[10..12].copy_from_slice(
                &(span_field | singlefs_core::allocator::ALLOCATION_RECORD_QUARANTINED_FLAG).to_le_bytes(),
            );
        },
    );
    println!("PROBE-Q1 one-device changed_records={changed}");
    probe_describe("both-bits-on-device-0-released-records-only", &devices);
}

/// 对照：已有的「记录不在叶按位置罩的那一段里」这一坏法，core 拒、checker 红 I-1.1 吗？把一条记录的槽号改到叶外。
#[test]
fn probe_q1_control_record_outside_its_leaf() {
    let (mut devices, _first) = pool_after_the_first_file();
    let changed = rewrite_allocation_record_entries_of_the_newest_root(
        &mut devices,
        |record| !record.is_released && record.device == DeviceIdentity(0),
        |entry| {
            // 槽号 6 字节在偏移 4：加上一大块，落到别的叶的那一段
            let mut slot = [0u8; 8];
            slot[..6].copy_from_slice(&entry[4..10]);
            let moved = u64::from_le_bytes(slot) + 4_000;
            entry[4..10].copy_from_slice(&moved.to_le_bytes()[..6]);
        },
    );
    println!("PROBE-Q1 control changed_records={changed}");
    probe_describe("control-record-outside-its-leaf", &devices);
}
