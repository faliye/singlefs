use super::*;

#[test]
fn fixed_regions_classify_offsets_by_their_landing_zone() {
    let regions = FixedRegions::of_geometry(4096, 8, JOURNAL_RING_DEFAULT_BYTES);
    assert_eq!(regions.region_of(0), StoredRegion::SystemConfiguration);
    assert_eq!(regions.region_of(8191), StoredRegion::SystemConfiguration);
    assert_eq!(
        regions.region_of(8192),
        StoredRegion::UnitAreaAndEverythingElse
    );
    assert_eq!(regions.region_of(1 << 20), StoredRegion::RootRing);
    assert_eq!(regions.region_of(16 << 20), StoredRegion::JournalRing);
    assert_eq!(
        regions.region_of(JOURNAL_RING_START_SLOT * SLOT_BYTES + JOURNAL_RING_DEFAULT_BYTES - 1),
        StoredRegion::JournalRing
    );
    assert_eq!(
        regions.region_of(UNIT_AREA_START_SLOT_AT_THE_DEFAULT_RING * SLOT_BYTES),
        StoredRegion::UnitAreaAndEverythingElse
    );
}

#[test]
fn counting_block_device_counts_reads_and_writes_in_the_region_they_land_in() {
    let regions = FixedRegions::of_geometry(4096, 8, MEBIBYTE);
    let mut device = CountingBlockDevice::new(
        SparseBlockDevice::new(64 * MEBIBYTE, PhysicalBlockSizeInBytes(512)),
        regions,
    );
    let journal_offset = DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES);
    device
        .write_at(journal_offset, &[7u8; 4096], WriteDurability::Plain)
        .expect("journal 环里写 4096 字节");
    let mut buffer = [0u8; 512];
    device.read_at(journal_offset, &mut buffer).expect("读回");
    device
        .read_at(DeviceOffsetInBytes(0), &mut buffer)
        .expect("读系统配置槽");
    assert_eq!(buffer, [0u8; 512]);
    let tally = device.tally_now();
    assert_eq!(
        tally.journal_ring.writes,
        CallsAndBytes {
            calls: 1,
            bytes: 4096
        }
    );
    assert_eq!(
        tally.journal_ring.reads,
        CallsAndBytes {
            calls: 1,
            bytes: 512
        }
    );
    assert_eq!(
        tally.system_configuration.reads,
        CallsAndBytes {
            calls: 1,
            bytes: 512
        }
    );
    assert_eq!(tally.unit_area, ReadsAndWrites::default());
    assert_eq!(tally.root_ring, ReadsAndWrites::default());
    let before = tally;
    device
        .write_at(journal_offset, &[8u8; 512], WriteDurability::Plain)
        .expect("再写一次");
    assert_eq!(
        device.tally_now().minus(before).journal_ring.writes,
        CallsAndBytes {
            calls: 1,
            bytes: 512
        }
    );
}
