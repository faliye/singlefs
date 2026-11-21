use super::*;

/// 第九节变异「GPU 多判包装不再判」在这里红：包装把每条确认的校验和都在 CPU 上再判了一遍，对不上的条数是 0。
#[test]
fn the_rejudging_wrapper_rejudges_every_confirmed_checksum_on_the_cpu() {
    let log: SharedRejudgeLog = Arc::new(Mutex::new(RejudgeLog::default()));
    let mut wrapper = GpuComputeRejudgedOnTheCpu {
        inner: CpuBackedStandIn {
            corrupted_crc32c: false,
        },
        log: Arc::clone(&log),
    };
    let contents: Vec<Vec<u8>> = (0..3_u8).map(|seed| vec![seed; 32768]).collect();
    let decisions = wrapper
        .verify_unit_checksums(&units_to_verify(&contents))
        .expect("替身不出错");
    assert_eq!(decisions.len(), 3);
    let _submissions = wrapper.take_the_submissions_of_the_batch();
    let log = log.lock().expect("记录");
    assert_eq!(
        log.tally,
        RejudgeTally {
            entries_rejudged: 3,
            mismatches: 0
        }
    );
    assert_eq!(
        log.handed_in_of_each_batch,
        vec![EntriesHandedIn {
            unit_checksums: 3,
            region_records: 0,
            cluster_segments: 0
        }]
    );
    assert_eq!(log.clock_read_failures, 0);
}

#[test]
fn the_rejudging_wrapper_counts_a_checksum_the_gpu_confirmed_wrongly() {
    let log: SharedRejudgeLog = Arc::new(Mutex::new(RejudgeLog::default()));
    let mut wrapper = GpuComputeRejudgedOnTheCpu {
        inner: CpuBackedStandIn {
            corrupted_crc32c: true,
        },
        log: Arc::clone(&log),
    };
    let contents: Vec<Vec<u8>> = (0..2_u8).map(|seed| vec![seed; 32768]).collect();
    let _decisions = wrapper
        .verify_unit_checksums(&units_to_verify(&contents))
        .expect("替身不出错");
    assert_eq!(
        log.lock().expect("记录").tally,
        RejudgeTally {
            entries_rejudged: 2,
            mismatches: 2
        }
    );
}

#[test]
fn the_rejudging_wrapper_rejudges_every_confirmed_live_slot_count_and_checks_every_full_segment_was_handed_in(
) {
    let log: SharedRejudgeLog = Arc::new(Mutex::new(RejudgeLog::default()));
    let mut wrapper = GpuComputeRejudgedOnTheCpu {
        inner: CpuBackedStandIn {
            corrupted_crc32c: false,
        },
        log: Arc::clone(&log),
    };
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 4 * LOCAL_CLUSTER_SEGMENT_SLOTS,
    };
    let segments: Vec<ClusterSegmentEntryKey> = (0..4_u64)
        .map(|segment_index| ClusterSegmentEntryKey {
            device: DeviceIdentity(0),
            start_slot: SlotNumber(
                LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS,
            ),
        })
        .collect();
    let _decisions = wrapper
        .count_live_slots_per_cluster_segment(&[], unit_area, &segments)
        .expect("替身不出错");
    let _fewer = wrapper
        .count_live_slots_per_cluster_segment(&[], unit_area, &segments[..3])
        .expect("替身不出错");
    let log = log.lock().expect("记录");
    assert_eq!(
        log.tally,
        RejudgeTally {
            entries_rejudged: 4 + 3,
            mismatches: 0
        }
    );
    assert_eq!(log.live_slot_calls_not_handing_in_every_full_segment, 1);
}
