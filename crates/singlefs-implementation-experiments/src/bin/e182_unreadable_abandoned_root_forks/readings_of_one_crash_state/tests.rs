use super::*;
use crate::helpers_shared_by_the_tests::two_device_pool_with_pattern_at;

/// B4 的类看两块盘：被盖的单元只在盘 1 上，也是类 b；没被盖时已确认档是类 a、未确认档是类 c。
#[test]
fn the_hidden_root_class_looks_at_the_units_on_both_disks() {
    let pool = two_device_pool_with_pattern_at(16384, 512);
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
    let intact = CrashImage {
        base: &pool,
        writes: &[],
        persisted: Vec::new(),
    };
    assert_eq!(
        class_of_the_hidden_root_in_the_state(&intact, &log, Confirmation::Confirmed),
        "class_confirmed_none_covered"
    );
    assert_eq!(
        class_of_the_hidden_root_in_the_state(&intact, &log, Confirmation::Unconfirmed),
        "class_unconfirmed_none_covered"
    );
    let mut covered_pool = pool.clone();
    covered_pool.flip_byte(DeviceIdentity(1), DeviceOffsetInBytes(16384), 0);
    let covered = CrashImage {
        base: &covered_pool,
        writes: &[],
        persisted: Vec::new(),
    };
    assert_eq!(
        class_of_the_hidden_root_in_the_state(&covered, &log, Confirmation::Confirmed),
        "class_some_unit_covered"
    );
    assert_eq!(
        class_of_the_hidden_root_in_the_state(&covered, &log, Confirmation::Unconfirmed),
        "class_some_unit_covered"
    );
}
