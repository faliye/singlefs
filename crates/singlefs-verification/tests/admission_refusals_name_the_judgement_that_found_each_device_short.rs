//! 空间准入拒绝时，每块不够的盘报的是哪一判拒的（C575（一个错误成员装两种判据的载荷））：式子那一判
//! （`admit_on_every_device`）与「这次的单元落得下」那一判（`admit_the_units_landing_on_every_device`）都交回
//! `AdmissionRefusedOnSomeDevices`，两判的 `available` / `demand` 装的不是同一对量；`DeviceShortOfDemand::judged_short_by`
//! 说清是哪一判、哪一样，光看两个数分不出的两次拒绝靠它分得出。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::{
    admit_on_every_device, admit_the_units_landing_on_every_device, AdmissionReading,
    AdmissionRefusedOnSomeDevices, AvailableBytesOnOneDevice, BytesOnOneDevice,
    BytesSummedOverAllReplicas, DemandOnDevice, DeviceAdmissionTerms, DeviceShortOfDemand,
    JudgementThatFoundTheDeviceShort, PlacementRoomOnOneDevice, PoolWideCommitments, ReplicaCount,
    UnitsOfAPublishToLand,
};

const THE_ONLY_DEVICE: DeviceIdentity = DeviceIdentity(0);

/// 一块盘、只有容量、别的扣项都是 0 的式子读数。
fn reading_of_one_empty_device_of(capacity_in_slots: u64) -> AdmissionReading {
    AdmissionReading::new(
        vec![DeviceAdmissionTerms {
            device: THE_ONLY_DEVICE,
            capacity: BytesOnOneDevice::of_slots(capacity_in_slots),
            allocated: BytesOnOneDevice::ZERO,
            unreclaimable: BytesOnOneDevice::ZERO,
            mount_time_commitment: BytesOnOneDevice::ZERO,
            isolated_for_abandoned_roots_and_unapplied_journal_records: BytesOnOneDevice::ZERO,
            ring_window: BytesOnOneDevice::ZERO,
        }],
        PoolWideCommitments::of_the_first_version(
            BytesSummedOverAllReplicas::ZERO,
            BytesSummedOverAllReplicas::ZERO,
        ),
        ReplicaCount::new(1).expect("一份副本"),
    )
}

fn the_only_short_device(
    refusal: Result<(), AdmissionRefusedOnSomeDevices>,
    which_refusal: &str,
) -> DeviceShortOfDemand {
    let Err(AdmissionRefusedOnSomeDevices { short_devices }) = refusal else {
        panic!("{which_refusal}：这一判该拒");
    };
    let [short_device] = short_devices.as_slice() else {
        panic!("{which_refusal}：只有一块盘，拒绝里该恰好一块：{short_devices:?}");
    };
    *short_device
}

/// 式子那一判：容量 1 槽的盘要 2 槽 ⇒ 可用 1 槽 < 需求 2 槽。落得下那一判：没有数据单元、没挡的槽 1 个、提交内生块要 2 槽 ⇒
/// 剩下的 1 槽 < 2 槽。两次拒绝的盘、`available`、`demand` 逐个相同，只有 `judged_short_by` 分得出哪一判拒的。
/// 判别力：两判合并成一个读法（落得下那一判也报式子的成员）时，两次拒绝逐字段相同，最后一条断言红。
#[test]
fn formula_refusal_and_landing_refusal_with_the_same_bytes_are_told_apart_by_the_judgement() {
    let refused_by_the_formula = the_only_short_device(
        admit_on_every_device(
            &reading_of_one_empty_device_of(1),
            &[DemandOnDevice {
                device: THE_ONLY_DEVICE,
                bytes: BytesOnOneDevice::of_slots(2),
            }],
        ),
        "式子那一判",
    );
    let refused_by_the_landing = the_only_short_device(
        admit_the_units_landing_on_every_device(
            &[PlacementRoomOnOneDevice {
                device: THE_ONLY_DEVICE,
                slot_pairs_for_data_units: 0,
                slots_for_commit_generated_units: 1,
            }],
            UnitsOfAPublishToLand {
                data_units: 0,
                commit_generated_slots: 2,
            },
        ),
        "落得下那一判",
    );
    let one_slot = AvailableBytesOnOneDevice(i128::from(BytesOnOneDevice::of_slots(1).0));
    assert_eq!(
        (
            refused_by_the_formula.available,
            refused_by_the_formula.demand
        ),
        (one_slot, BytesOnOneDevice::of_slots(2)),
        "式子那一判：可用(d) 1 槽、需求(d) 2 槽"
    );
    assert_eq!(
        (
            refused_by_the_landing.available,
            refused_by_the_landing.demand
        ),
        (
            refused_by_the_formula.available,
            refused_by_the_formula.demand
        ),
        "落得下那一判报的两个数与式子那一判逐个相同：光看数分不出"
    );
    assert_eq!(
        refused_by_the_formula.judged_short_by,
        JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
        "式子那一判报式子的成员"
    );
    assert_eq!(
        refused_by_the_landing.judged_short_by,
        JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits,
        "落得下那一判短在提交内生块那一样，报那一样的成员"
    );
    assert_ne!(
        refused_by_the_formula, refused_by_the_landing,
        "同样的数、不同的判：两次拒绝分得出是哪一判拒的"
    );
}

/// 落得下那一判里的两样：没挡的槽 2 个、段外槽对 0 对，要 1 个数据单元 ⇒ 数据单元那一样短（段外 0 字节 < 2 槽）；
/// 同一块盘段外 1 对、没挡的槽 2 个，要 1 个数据单元与 1 槽提交内生块 ⇒ 数据单元取走 2 槽、剩 0 < 1，提交内生块那一样短。
/// 判别力：数据单元那一样报成提交内生块的成员（两样合成一个成员）时，第一条成员断言红。
#[test]
fn the_landing_judgement_names_whether_the_data_units_or_the_commit_generated_units_were_short() {
    let data_units_short = the_only_short_device(
        admit_the_units_landing_on_every_device(
            &[PlacementRoomOnOneDevice {
                device: THE_ONLY_DEVICE,
                slot_pairs_for_data_units: 0,
                slots_for_commit_generated_units: 2,
            }],
            UnitsOfAPublishToLand {
                data_units: 1,
                commit_generated_slots: 0,
            },
        ),
        "数据单元短",
    );
    let commit_generated_units_short = the_only_short_device(
        admit_the_units_landing_on_every_device(
            &[PlacementRoomOnOneDevice {
                device: THE_ONLY_DEVICE,
                slot_pairs_for_data_units: 1,
                slots_for_commit_generated_units: 2,
            }],
            UnitsOfAPublishToLand {
                data_units: 1,
                commit_generated_slots: 1,
            },
        ),
        "提交内生块短",
    );
    assert_eq!(
        data_units_short.judged_short_by,
        JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
        "段外没有槽对：数据单元那一样短"
    );
    assert_eq!(
        (data_units_short.available, data_units_short.demand),
        (AvailableBytesOnOneDevice(0), BytesOnOneDevice::of_slots(2)),
        "数据单元那一样：段外槽对的字节 0、一个数据单元 2 槽"
    );
    assert_eq!(
        commit_generated_units_short.judged_short_by,
        JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits,
        "数据单元取走两槽之后没挡的槽不剩：提交内生块那一样短"
    );
    assert_eq!(
        (
            commit_generated_units_short.available,
            commit_generated_units_short.demand
        ),
        (AvailableBytesOnOneDevice(0), BytesOnOneDevice::of_slots(1)),
        "提交内生块那一样：剩下的 0 字节、要 1 槽"
    );
}
