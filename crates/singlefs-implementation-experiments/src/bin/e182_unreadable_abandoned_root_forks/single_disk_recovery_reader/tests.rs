use super::*;
use crate::helpers_shared_by_the_tests::two_device_pool_with_pattern_at;

#[test]
fn the_kept_disk_view_hides_the_other_disk_and_zeroes_a_masked_root_slot() {
    let pool = two_device_pool_with_pattern_at(8192, 1024);
    let image = CrashImage {
        base: &pool,
        writes: &[],
        persisted: Vec::new(),
    };
    let mut view = kept_disk(&image, DeviceIdentity(1));
    assert_eq!(
        PoolReader::device_identities(&view),
        vec![DeviceIdentity(0), DeviceIdentity(1)],
        "掉了的盘身份还在盘表里"
    );
    assert_eq!(
        PoolReader::read(&view, DeviceIdentity(0), DeviceOffsetInBytes(8192), 512),
        None,
        "另一块盘整块掉了，身份在、读不到"
    );
    assert_eq!(
        PoolReader::read(&view, DeviceIdentity(1), DeviceOffsetInBytes(8192), 1024),
        Some(vec![0xA5; 1024])
    );
    view.masked_root_slots.push((8192 + 512, 512));
    let read_back = PoolReader::read(&view, DeviceIdentity(1), DeviceOffsetInBytes(8192), 1024)
        .expect("留下的盘读得到");
    assert!(
        read_back[..512].iter().all(|byte| *byte == 0xA5),
        "没罩住的一半不动"
    );
    assert!(
        read_back[512..].iter().all(|byte| *byte == 0),
        "被避开的根槽读回全零"
    );
}

/// 被盖按单元自己所在的那块盘判：盘 1 上的单元被改，只留盘 0 看不出、只留盘 1 看得出。
#[test]
fn unit_is_covered_when_its_own_disks_bytes_differ_from_the_log() {
    let mut pool = two_device_pool_with_pattern_at(16384, 512);
    let log = RootLog {
        units: vec![
            UnitLog {
                device: 0,
                offset_in_bytes: 16384,
                bytes: vec![0xA5; 512],
            },
            UnitLog {
                device: 1,
                offset_in_bytes: 16384,
                bytes: vec![0xA5; 512],
            },
        ],
        account_readable: true,
    };
    pool.flip_byte(DeviceIdentity(1), DeviceOffsetInBytes(16384), 0);
    let image = CrashImage {
        base: &pool,
        writes: &[],
        persisted: Vec::new(),
    };
    assert!(
        !unit_differs_in_the_image(&image, &log.units[0]),
        "盘 0 上的单元没动"
    );
    assert!(
        unit_differs_in_the_image(&image, &log.units[1]),
        "盘 1 上的单元被改了，要读盘 1 才看得出"
    );
    assert!(!some_unit_of_the_root_is_covered(&image, &log, Some(0)));
    assert!(some_unit_of_the_root_is_covered(&image, &log, Some(1)));
    assert!(some_unit_of_the_root_is_covered(&image, &log, None));
}
