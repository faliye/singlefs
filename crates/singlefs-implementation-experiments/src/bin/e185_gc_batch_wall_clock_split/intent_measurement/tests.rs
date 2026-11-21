use super::*;

fn recorded_read(
    device: DeviceIdentity,
    slot: u64,
    length: u64,
    is_before_the_first_write: bool,
) -> RecordedRead {
    RecordedRead {
        device,
        offset: slot * LOCAL_SLOT_BYTES,
        length,
        is_before_the_first_write,
    }
}

/// 一批的读照发布的次序排：区域扫描读 R 那块盘上每个活单元一遍，释放前读盘核读每个搬迁源两块盘各一遍，然后第一次写。
fn reads_of_a_batch(live_slots: &[u64], relocated_slots: &[u64]) -> Vec<RecordedRead> {
    let mut reads: Vec<RecordedRead> = live_slots
        .iter()
        .map(|slot| recorded_read(DeviceIdentity(0), *slot, LOCAL_DATA_UNIT_BYTES, true))
        .collect();
    for slot in relocated_slots {
        reads.push(recorded_read(
            DeviceIdentity(0),
            *slot,
            LOCAL_DATA_UNIT_BYTES,
            true,
        ));
        reads.push(recorded_read(
            DeviceIdentity(1),
            *slot,
            LOCAL_DATA_UNIT_BYTES,
            true,
        ));
    }
    reads
}

#[test]
fn release_check_read_bytes_of_a_batch_count_both_devices() {
    assert_eq!(release_check_read_bytes_of_a_batch(4), 262_144);
    assert_eq!(release_check_read_bytes_of_a_batch(8), 524_288);
    assert_eq!(release_check_read_bytes_of_a_batch(16), 1_048_576);
}

/// 扫描三个活单元、搬走其中两个：R 那块盘上搬迁源的再读与另一块盘上的那两读归释放前读盘核；不是搬迁源的槽上再读照旧归扫描；
/// R 外的读归别处；第一次写之后的读单列。
#[test]
fn reads_of_a_call_split_the_release_check_copies_from_the_scan_reads() {
    let mut reads = reads_of_a_batch(&[50_304, 50_306, 50_308], &[50_304, 50_306]);
    reads.push(recorded_read(
        DeviceIdentity(0),
        50_308,
        LOCAL_DATA_UNIT_BYTES,
        true,
    ));
    reads.push(recorded_read(
        DeviceIdentity(0),
        100,
        LOCAL_NODE_BYTES,
        true,
    ));
    reads.push(recorded_read(
        DeviceIdentity(1),
        100,
        LOCAL_NODE_BYTES,
        true,
    ));
    reads.push(recorded_read(
        DeviceIdentity(0),
        50_304,
        LOCAL_DATA_UNIT_BYTES,
        false,
    ));
    let relocated: BTreeSet<u64> = [50_304, 50_306].into_iter().collect();
    let split = split_the_reads_of_a_call(&reads, Some(region_at(50_304)), &relocated);
    let scan_bytes: u64 = split
        .scan_reads_in_the_region
        .iter()
        .map(|read| read.length)
        .sum();
    let release_check_bytes: u64 = split
        .release_check_reads
        .iter()
        .map(|read| read.length)
        .sum();
    assert_eq!(
        scan_bytes, 131_072,
        "三个活单元各一读，加上不是搬迁源的那一次再读"
    );
    assert_eq!(
        release_check_bytes,
        release_check_read_bytes_of_a_batch(2),
        "两个搬迁源两块盘各一份"
    );
    assert_eq!(
        split
            .release_check_reads
            .iter()
            .map(|read| read.device)
            .collect::<Vec<_>>(),
        vec![
            DeviceIdentity(0),
            DeviceIdentity(1),
            DeviceIdentity(0),
            DeviceIdentity(1)
        ]
    );
    assert_eq!(
        split.scan_read_bytes_elsewhere, 32_768,
        "R 外两块盘各一个节点"
    );
    assert_eq!(split.read_bytes_after_the_first_write, 32_768);
    let before_the_publish = split_the_reads_of_a_call(&reads, None, &BTreeSet::new());
    assert_eq!(
        before_the_publish.release_check_reads,
        Vec::<RecordedRead>::new(),
        "没有搬迁源就没有释放前读盘核"
    );
}

/// 反例：释放前读盘核的读并回扫描读（没有搬迁源可扣，等于这一栏没分出来），停机条款第 2 条必须红；分出来之后不红。
/// f = 0.25（m = 8）、K = 8：最后一批扫 8 个单元、搬 8 个。
#[test]
fn merging_the_release_check_reads_back_into_the_scan_trips_stop_clause_two() {
    let live_slots: Vec<u64> = (0..8).map(|unit| 50_304 + 8 * unit).collect();
    let reads = reads_of_a_batch(&live_slots, &live_slots);
    let relocated: BTreeSet<u64> = live_slots.iter().copied().collect();
    let scan_bytes_of = |split: &ReadsOfOneCall| -> u64 {
        split
            .scan_reads_in_the_region
            .iter()
            .map(|read| read.length)
            .sum()
    };
    let separated = split_the_reads_of_a_call(&reads, Some(region_at(50_304)), &relocated);
    let merged = split_the_reads_of_a_call(&reads, Some(region_at(50_304)), &BTreeSet::new());
    assert_eq!(
        separated
            .release_check_reads
            .iter()
            .map(|read| read.length)
            .sum::<u64>(),
        release_check_read_bytes_of_a_batch(8)
    );
    let intent_with_the_scan_bytes = |scan_bytes: u64| {
        intent_of(vec![
            call_reading(CallKind::Issue, 262_144, Some(262_144), 0, 0),
            call_reading(CallKind::FinalBatch, scan_bytes, Some(262_144), 524_288, 1),
        ])
    };
    assert_eq!(
        stop_findings_of(
            &intent_with_the_scan_bytes(scan_bytes_of(&separated)),
            Some(8),
            8
        ),
        Vec::<String>::new()
    );
    let findings = stop_findings_of(
        &intent_with_the_scan_bytes(scan_bytes_of(&merged)),
        Some(8),
        8,
    );
    assert_eq!(
        findings,
        vec!["clause_2_scan_read_bytes_at_call_1:read=524288:records=262144".to_string()]
    );
}
