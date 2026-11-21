use super::*;

#[test]
fn local_constants_agree_with_the_core() {
    assert_eq!(local_constants_disagreeing_with_the_core(), Vec::new());
}

#[test]
fn publishes_of_one_intent_at_the_main_sampling_point_are_five() {
    assert_eq!(
        publishes_of_one_intent(BATCH_UNITS_AT_THE_MAIN_SAMPLING_POINT),
        1 + 4
    );
    assert_eq!(
        publishes_of_one_intent(BATCH_UNITS_AT_THE_SENSITIVITY_POINT),
        1 + 1
    );
}

#[test]
fn the_tiers_hold_half_their_unit_area_in_live_data_units() {
    let live: Vec<u64> = UnitAreaTier::EVERY_TIER
        .iter()
        .map(|tier| tier.live_data_units())
        .collect();
    assert_eq!(live, vec![36608, 65536, 131_072]);
    let devices: Vec<u64> = UnitAreaTier::EVERY_TIER
        .iter()
        .map(|tier| tier.device_bytes() >> 20)
        .collect();
    assert_eq!(devices, vec![3072, 4880, 8976]);
    let names: Vec<&str> = UnitAreaTier::EVERY_TIER
        .iter()
        .map(|tier| tier.name())
        .collect();
    assert_eq!(names, vec!["2288MiB", "4GiB", "8GiB"]);
}

/// 登记第十二节修订第 4 条随档改的乙类锚点：每块盘单元区槽数、整段数、碎片段数（P ÷ 32）。
#[test]
fn the_tiers_have_the_registered_slots_segments_and_fragment_segments() {
    let numbers: Vec<(u64, u64, u64)> = UnitAreaTier::EVERY_TIER
        .iter()
        .map(|tier| {
            (
                local_unit_area_slots_of_device(tier.device_bytes()),
                tier.unit_area_segments(),
                tier.fragment_segments(),
            )
        })
        .collect();
    assert_eq!(
        numbers,
        vec![
            (146_432, 2288, 1144),
            (262_144, 4096, 2048),
            (524_288, 8192, 4096)
        ]
    );
    assert_eq!(data_units_of_a_cluster_segment(), 32);
}

/// mkfs 那一条（环超过设备容量的四分之一就拒）拿真 mkfs 回比：差一个扇区不到四倍环长的盘拒，正好四倍的认。
#[test]
fn the_make_filesystem_refuses_a_device_one_sector_short_of_four_rings() {
    let parameters = make_filesystem_parameters_of_the_pool();
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::not_fed()));
    let smallest = smallest_device_bytes_the_make_filesystem_accepts();
    assert_eq!(smallest, 3072 << 20);
    let mut one_sector_short = empty_pool_devices(smallest - SECTOR_BYTES, &recording);
    assert!(matches!(
        make_filesystem(&parameters, &mut one_sector_short),
        Err(
            singlefs_core::make_filesystem::MakeFilesystemError::JournalRingTooLargeForDevice { .. }
        )
    ));
    let mut just_four_rings = empty_pool_devices(smallest, &recording);
    assert!(make_filesystem(&parameters, &mut just_four_rings).is_ok());
    assert!(the_make_filesystem_accepts_the_device_bytes(smallest));
    assert!(!the_make_filesystem_accepts_the_device_bytes(
        smallest - SECTOR_BYTES
    ));
}

/// 登记第十二节修订第 4 条的三档（每块盘 3072、4880、8976 MiB）在默认环长下 mkfs 全认：档预检判认，真 mkfs 在两块空内存盘上也做成。
#[test]
fn the_registered_device_bytes_of_every_tier_are_accepted_by_the_make_filesystem() {
    let parameters = make_filesystem_parameters_of_the_pool();
    let recording: SharedRecordingDigest = Arc::new(Mutex::new(RecordingDigest::not_fed()));
    let accepted: Vec<(u64, bool, bool)> = UnitAreaTier::EVERY_TIER
        .into_iter()
        .map(|tier| {
            let mut devices = empty_pool_devices(tier.device_bytes(), &recording);
            (
                tier.device_bytes() >> 20,
                the_make_filesystem_accepts_the_device_bytes(tier.device_bytes()),
                make_filesystem(&parameters, &mut devices).is_ok(),
            )
        })
        .collect();
    assert_eq!(
        accepted,
        [(3072, true, true), (4880, true, true), (8976, true, true)]
    );
}
