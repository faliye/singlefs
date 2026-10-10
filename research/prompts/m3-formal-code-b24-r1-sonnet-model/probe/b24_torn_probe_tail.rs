
// ---- 正推腿追加（m3-formal-code-b24-r1 sonnet）：前面是冻结副本里 `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的原文 ----
/// 模块文档说「换一份盘面照常比可以为真」：在只落了覆盖写轮换的盘面上，再把盘 0 的两个系统配置槽都撕裂（盘 1 那一槽的轮换还在），
/// 盘 0 一份自证过的都没有，C670 那一条跳过撕裂的槽、从盘 1 取 c_见证，大于第一个文件那一版末条的计数器 ⇒ 复算为真 ⇒ 判不合法。
#[test]
fn b24_probe_a_torn_device_zero_with_the_rotation_landed_on_device_one_is_illegal() {
    let (healthy, rotation_only) = crash_image_where_only_the_rotation_of_an_overwrite_landed();
    let mut crash_image = rotation_only.clone();
    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for slot in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot * slot_spacing);
        let mut bytes = crash_image.read(DeviceIdentity(0), offset, slot_bytes).expect("读得出");
        bytes[TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT] ^= 0xFF;
        crash_image.devices.get_mut(&DeviceIdentity(0)).expect("有盘 0").write(offset, &bytes);
    }
    let (error, after) = refused_writable_mount_on(&crash_image);
    println!("b24-torn refusal comparison = {:?}", witnessed_counter_comparison_on_the_reread(&error));
    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
    let verdict = judge_writable_mount_refusal_of_a_crash_state(
        &crash_image,
        fields.report.effective_root,
        &error,
        disk_after(&crash_image, &after),
        read_only_read_back_against_the_first_file(&healthy, &crash_image),
        ReadFaultsInTheCrashState::NoneInjected,
    );
    println!("b24-torn verdict = {verdict:?}");
    assert!(matches!(verdict, WritableMountRefusalVerdict::Illegal { .. }), "{verdict:?}");
}
