use super::*;

#[test]
fn region_of_one_segment_holds_one_mebibyte_on_each_device() {
    assert_eq!(region_bytes_per_device(), 1_048_576);
}

#[test]
fn issue_scan_reads_262144_bytes_at_one_quarter_and_786432_at_three_quarters() {
    assert_eq!(
        issue_scan_bytes(LiveSlotFraction::OneQuarter.data_units_per_segment()),
        262_144
    );
    assert_eq!(
        issue_scan_bytes(LiveSlotFraction::ThreeQuarters.data_units_per_segment()),
        786_432
    );
}

#[test]
fn batches_of_one_intent_are_the_ceiling_of_the_data_units_over_the_batch_units() {
    let expected = [
        ((8, 4), 2),
        ((8, 8), 1),
        ((8, 16), 1),
        ((24, 4), 6),
        ((24, 8), 3),
        ((24, 16), 2),
    ];
    for ((data_units, batch_units), batches) in expected {
        assert_eq!(
            batches_of_one_intent(data_units, batch_units),
            batches,
            "m = {data_units}、K = {batch_units}"
        );
    }
}

#[test]
fn intent_scan_bytes_add_the_issue_scan_to_every_batch_rescan() {
    let expected = [
        ((8, 4), 655_360),
        ((8, 8), 524_288),
        ((8, 16), 524_288),
        ((24, 4), 3_538_944),
        ((24, 8), 2_359_296),
        ((24, 16), 1_835_008),
    ];
    for ((data_units, batch_units), bytes) in expected {
        assert_eq!(
            intent_scan_bytes(data_units, batch_units),
            bytes,
            "m = {data_units}、K = {batch_units}"
        );
    }
}

#[test]
fn first_batch_copy_bytes_count_both_devices() {
    assert_eq!(first_batch_copy_bytes(24, 4), 262_144);
    assert_eq!(first_batch_copy_bytes(24, 8), 524_288);
    assert_eq!(first_batch_copy_bytes(24, 16), 1_048_576);
    assert_eq!(
        first_batch_copy_bytes(8, 16),
        524_288,
        "f = 0.25 时 K = 16 也只有 8 个单元"
    );
}

#[test]
fn intent_copy_bytes_are_524288_and_1572864() {
    assert_eq!(intent_copy_bytes(8), 524_288);
    assert_eq!(intent_copy_bytes(24), 1_572_864);
}

#[test]
fn expected_scan_read_bytes_count_sixteen_kibibytes_per_slot() {
    let records = [
        record(50_304, 2, false),
        record(50_306, 2, false),
        record(50_308, 2, true),
        record(50_310, 1, false),
        record(50_368, 2, false),
    ];
    assert_eq!(
        expected_scan_read_bytes(&records, region_at(50_304)),
        2 * 32_768 + 16_384,
        "R 里两个未释放的数据单元与一个节点；已释放的、R 外的不算"
    );
}

#[test]
fn doubling_pairs_keep_only_pairs_within_the_movable_units() {
    assert_eq!(doubling_pairs_within(8), vec![(4, 8)]);
    assert_eq!(doubling_pairs_within(24), vec![(4, 8), (8, 16)]);
}

#[test]
fn the_issue_call_is_not_counted_as_a_batch() {
    let calls = [kind_of_the_call(1, true, 0), kind_of_the_call(1, false, 8)];
    assert_eq!(calls, [CallKind::Issue, CallKind::FinalBatch]);
    let batches = calls
        .iter()
        .filter(|kind| counts_as_a_batch(**kind))
        .count();
    assert_eq!(batches, 1, "f = 0.25、K = 8：签发一次、一批（最后一批）");
    assert_eq!(kind_of_the_call(1, true, 4), CallKind::Batch);
    assert_eq!(kind_of_the_call(0, true, 0), CallKind::NoPublish);
    assert_eq!(
        kind_of_the_call(1, false, 0),
        CallKind::IntentDeletedWithoutABatch
    );
}

#[test]
fn relocation_copy_bytes_count_only_the_overlap_with_the_destinations() {
    let writes = [
        RecordedWrite {
            device: DeviceIdentity(0),
            offset: 1 << 30,
            length: 32_768,
        },
        RecordedWrite {
            device: DeviceIdentity(1),
            offset: 1 << 30,
            length: 32_768,
        },
        RecordedWrite {
            device: DeviceIdentity(0),
            offset: 2 << 30,
            length: 16_384,
        },
    ];
    let destinations = [
        (DeviceIdentity(0), 1 << 30, 32_768),
        (DeviceIdentity(1), 1 << 30, 32_768),
    ];
    assert_eq!(
        relocation_copy_bytes(&writes, &destinations),
        (65_536, true)
    );
    assert_eq!(
        relocation_copy_bytes(&writes[..1], &destinations),
        (32_768, false),
        "只写了一块盘"
    );
}

/// 端到端（内存盘、小池）：造池、核几何、量一条意图，第七节锚点与停机条款第 2、3、5、6 条不触发，第二段的重放（停机条款第 4 条）对得上。
#[test]
fn small_pool_relocates_one_intent_with_the_anchors_and_the_replay_holding() {
    let point = SamplingPoint {
        fraction: LiveSlotFraction::ThreeQuarters,
        pool_data_units: 128,
    };
    let prepared = prepare_the_sampling_point(point);
    assert!(
        prepared.is_the_geometry_formed,
        "128 个数据单元造得出至少一个 f 段"
    );
    assert!(
        prepared.pool.is_every_overwritten_slot_released,
        "被覆盖写的旧落点都已释放"
    );
    let batch_units = 8;
    let reading = measure_on_the_memory_device(
        &prepared.pool,
        measured_switches(batch_units, prepared.choice.forced_region()),
        WriteInjection::Nothing,
        ReplayCapture::On,
    );
    assert_eq!(reading.mount_failure, None);
    let findings = stop_findings_of(&reading, Some(24), batch_units);
    assert_eq!(findings, Vec::<String>::new());
    assert_eq!(
        reading.calls.first().map(|call| call.kind),
        Some(CallKind::Issue)
    );
    assert_eq!(
        reading
            .calls
            .iter()
            .filter(|call| counts_as_a_batch(call.kind))
            .count(),
        3,
        "m = 24、K = 8：三批"
    );
    assert_eq!(
        reading
            .calls
            .iter()
            .map(|call| call.scan_read_bytes_in_the_region)
            .sum::<u64>(),
        intent_scan_bytes(24, batch_units),
        "签发加各批扫描读到 R 的字节之和等于锚点"
    );
    for call in &reading.calls {
        assert_eq!(
            call.release_check_read_bytes,
            release_check_read_bytes_of_a_batch(call.relocated_units),
            "释放前读盘核：这一批每个搬迁源两块盘各读一份整单元（{:?}）",
            call.kind
        );
    }
    assert_eq!(
        reading
            .calls
            .iter()
            .map(|call| call.release_check_read_bytes)
            .sum::<u64>(),
        release_check_read_bytes_of_a_batch(24),
        "一条意图搬走 24 个数据单元，释放前读盘核读 2 × 24 × 32768"
    );
    for material in &reading.replay_material {
        let replay = replay_the_stages_of_a_call(
            material,
            reading.region,
            prepared.choice,
            batch_units,
            &prepared.pool.devices,
        );
        assert_eq!(
            replay.stop_finding, None,
            "第 {} 次调用的重放",
            material.call_index
        );
        assert_eq!(
            replay.crc_bytes,
            material
                .scan_reads_in_the_region
                .iter()
                .map(|read| read.length)
                .sum::<u64>(),
            "甲段字节等于扫描读字节"
        );
        assert_eq!(
            replay.release_check_crc_bytes,
            reading.calls[material.call_index].release_check_read_bytes,
            "甲段释放前读盘核那一项的字节等于这一批 release_check_read_bytes（第 {} 次调用）",
            material.call_index
        );
    }
}
