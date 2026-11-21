//! C971（多处读坏时可写挂载写出倒挂的新实例根）最小历史的六格与三张手造盘面：根与系统配置槽各按几种之一读坏的六格都拒可写，只翻盘 0 的照挂、翻两块盘的拒、都不翻的照挂。

use super::*;

#[test]
fn the_minimal_history_with_roots_reporting_read_errors_and_system_configuration_slots_reading_back_zeros_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::DeviceReportsAReadError,
        ReadFaultKind::ReadsBackAllZeros,
    );
}

#[test]
fn the_minimal_history_with_roots_reporting_read_errors_and_system_configuration_slots_failing_self_verification_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::DeviceReportsAReadError,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
}

#[test]
fn the_minimal_history_with_roots_failing_self_verification_and_system_configuration_slots_reading_back_zeros_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
        ReadFaultKind::ReadsBackAllZeros,
    );
}

#[test]
fn the_minimal_history_with_roots_and_system_configuration_slots_failing_self_verification_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
}

#[test]
fn the_minimal_history_with_roots_and_system_configuration_slots_reading_back_zeros_is_refused() {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackAllZeros,
    );
}

#[test]
fn the_minimal_history_with_roots_reading_back_zeros_and_system_configuration_slots_failing_self_verification_is_refused(
) {
    a_six_cell_is_refused_before_any_write(
        ReadFaultKind::ReadsBackAllZeros,
        ReadFaultKind::ReadsBackBytesFailingSelfVerification,
    );
}

/// 乙的手造盘面：最小历史、不注读故障，`devices` 那几块盘最新那一槽系统配置在盘上翻一个字节（读得出而自证不过）。
fn minimal_history_with_the_newest_system_configuration_slot_flipped_on(
    parameters: &MakeFilesystemParameters,
    devices: &[DeviceIdentity],
) -> MemoryPool {
    let mut image = minimal_history_of_c971(parameters).image;
    for device in devices {
        let offset = newest_system_configuration_slot(&image, *device);
        image.flip_byte(
            *device,
            DeviceOffsetInBytes(offset),
            FLIPPED_BYTE_INSIDE_A_RANGE,
        );
    }
    image
}

/// 手造盘面一（只翻盘 0）：盘 1 两槽都自证过，R2 的另一半不成立，判得出，照常可写挂载、取到实例 2（乙入判不出「不拒」）。
#[test]
fn a_hand_made_image_with_only_device_zero_failing_self_verification_mounts_writable() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_newest_system_configuration_slot_flipped_on(
        &parameters,
        &[DeviceIdentity(0)],
    );
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert_eq!(
        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
        Some(InstanceGeneration(2)),
        "一块盘两槽都自证过：判得出，照常可写挂载；得到 {:?}",
        outcome.as_ref().err()
    );
}

/// 手造盘面二（两块盘都翻）：设备没报读错，没有一块盘两槽都自证过，读得出而自证不过的槽当读错计 ⇒ 判不出，拒可写、盘上逐字节不变
/// （乙入判不出「拒」）；挂载透明判官判这一拒合法。
#[test]
fn a_hand_made_image_with_every_device_failing_self_verification_is_refused() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image = minimal_history_with_the_newest_system_configuration_slot_flipped_on(
        &parameters,
        &[DeviceIdentity(0), DeviceIdentity(1)],
    );
    let (outcome, image_after) = writable_mount_under_read_faults(&parameters, &image, &[]);
    assert!(
        refused_as_failing_self_verification_on_both_reads(&outcome),
        "两块盘最新那一槽都自证不过：乙入判不出拒可写；得到 {:?}",
        outcome.as_ref().map(|mounted| mounted.output.instance)
    );
    assert!(image_after == image, "拒在取号之前：盘上逐字节不变");
    assert_eq!(
        verdict_of_the_refusal_on(&image, &outcome),
        WritableMountRefusalVerdict::Legal(
            RefusalReasonOnTheLegalRefusalList::NewerPublishWitnessedBySystemConfiguration
        ),
        "挂载透明判官在同一份盘面上复算乙入判不出为真：这一拒合法"
    );
}

/// 手造盘面三（都不翻），也是阳性对照：不注坏照常可写挂载、取到实例 2，新实例的第一次发布落在 D（txg 6）之后（乙入判不出「不拒」）。
#[test]
fn a_hand_made_image_without_any_flipped_system_configuration_slot_mounts_writable() {
    let parameters = pool_parameters(SLOTS_PER_REGION_OF_THE_MINIMAL_HISTORY);
    let image =
        minimal_history_with_the_newest_system_configuration_slot_flipped_on(&parameters, &[]);
    let (outcome, _) = writable_mount_under_read_faults(&parameters, &image, &[]);
    let mounted = outcome.expect("不注坏：照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2), "取到实例 2");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(TXG_OF_THE_NEWEST_VERSION)
        ),
        "择到 D"
    );
    assert!(
        mounted.output.row_publish.root().checkpoint_txg.0 > TXG_OF_THE_NEWEST_VERSION,
        "新实例的第一次发布落在 D 之后，没有倒挂"
    );
}
