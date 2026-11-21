//! 崩溃镜像上的见证判据与可写挂载探针的单测。

use super::*;

/// 新池新建文件之后的整份镜像（`tests/common` 的 `build_pool`，与流一全持久态同一份）与它只读恢复落到的版本。
fn first_file_pool_and_its_selected_version(
    tag: &str,
) -> (MemoryPool, Option<(InstanceGeneration, CheckpointTxg)>) {
    let pool = experiment_pool::build_pool(tag).memory_pool();
    let selected =
        singlefs_core::recovery::recover(&pool, singlefs_core::recovery::JournalPolicy::Consult)
            .effective_root;
    (pool, selected)
}

/// 见证判据的两支：干净的池上两支都假、c_见证 是所选那一版末条记录的计数器；一块盘一槽系统配置撕裂（槽里一个字节翻一位、自证不过）记 (i)、
/// 不记 (ii)；每块盘两槽的 journal_tail 抬过所选那一版的末条（阳性对照「见证」的造法）记 (ii) 第一句。
#[test]
fn torn_system_configuration_slot_is_the_undecidable_branch_and_a_raised_tail_is_the_counter_branch(
) {
    let (clean, selected) =
        first_file_pool_and_its_selected_version("e171r1-unit-witness-branches");
    let clean_criterion = witness_criterion_on_the_crash_image(&clean, selected);
    assert!(!clean_criterion.some_device_has_fewer_than_two_verified_slots);
    assert_eq!(clean_criterion.counter_sentence, None);
    let mut torn = clean.clone();
    let device = torn.devices.get_mut(&DeviceIdentity(1)).expect("盘 1");
    let spacing = u64::from(
        experiment_pool::parameters()
            .geometry
            .fixed_structure_slot_spacing,
    );
    let mut slot_bytes = device.read(DeviceOffsetInBytes(spacing), 4096);
    slot_bytes[100] ^= 1;
    device.write(DeviceOffsetInBytes(spacing), &slot_bytes);
    let torn_criterion = witness_criterion_on_the_crash_image(&torn, selected);
    assert!(torn_criterion.some_device_has_fewer_than_two_verified_slots);
    assert_eq!(torn_criterion.counter_sentence, None);
    let mut raised = clean.clone();
    assert_eq!(
        raise_the_witnessed_counter_past_the_selected_version(&mut raised, selected),
        4
    );
    let raised_criterion = witness_criterion_on_the_crash_image(&raised, selected);
    assert_eq!(
        raised_criterion.counter_sentence,
        Some(WitnessCounterSentenceThatHolds::AboveTheLastRecordOfTheSelectedVersion)
    );
}

/// 每块盘两槽的 journal_tail 抬过所选那一版的末条之后，再撕盘 1 的一槽：只记 (i)、不记 (ii)，另三槽抬过的 tail 还在、c_见证 不变
/// （登记修订 2 第 2.4 节；V11：判不出那一支并进 (ii)，三句的第一句为真，这里红）。
#[test]
fn torn_slot_beside_raised_tails_is_only_the_undecidable_branch() {
    let (clean, selected) =
        first_file_pool_and_its_selected_version("e171r1-unit-witness-torn-beside-raised");
    let mut raised = clean.clone();
    assert_eq!(
        raise_the_witnessed_counter_past_the_selected_version(&mut raised, selected),
        4
    );
    let raised_criterion = witness_criterion_on_the_crash_image(&raised, selected);
    let mut torn = raised.clone();
    let device = torn.devices.get_mut(&DeviceIdentity(1)).expect("盘 1");
    let spacing = u64::from(
        experiment_pool::parameters()
            .geometry
            .fixed_structure_slot_spacing,
    );
    let mut slot_bytes = device.read(DeviceOffsetInBytes(spacing), 4096);
    slot_bytes[100] ^= 1;
    device.write(DeviceOffsetInBytes(spacing), &slot_bytes);
    let torn_criterion = witness_criterion_on_the_crash_image(&torn, selected);
    assert!(
        torn_criterion.some_device_has_fewer_than_two_verified_slots,
        "盘 1 撕了一槽，自证过的少于两槽：{torn_criterion:?}"
    );
    assert_eq!(
        torn_criterion.counter_sentence, None,
        "判不出那一支为真时计数那一支不判：{torn_criterion:?}"
    );
    assert_eq!(
        torn_criterion.witnessed_counter, raised_criterion.witnessed_counter,
        "另三槽抬过的 tail 还在，c_见证 与撕之前那份相同（取样点敏感）"
    );
}

/// 阳性对照「见证」那一态上的探针：可写挂载拒、拒因是系统配置见证、第二截判不合法、没满足的条件是 C670 那一条（第七节 A4；
/// V10：「没注入读故障」照「注入了」交，第二截判合法，这里红）。干净的池上可写挂载挂上。
#[test]
fn the_witness_control_is_refused_for_the_witness_and_judged_illegal_by_the_second_crop() {
    let (clean, selected) = first_file_pool_and_its_selected_version("e171r1-unit-witness-probe");
    let clean_probe = writable_mount_probe_on(&clean, selected, &CrashJudgeVerdict::Green);
    assert_eq!(clean_probe.mount, "mounted");
    let mut raised = clean.clone();
    raise_the_witnessed_counter_past_the_selected_version(&mut raised, selected);
    let probe = writable_mount_probe_on(&raised, selected, &CrashJudgeVerdict::Green);
    assert_eq!(probe.mount, "refused");
    assert!(
        probe.refused_for_the_system_configuration_witness,
        "{probe:?}"
    );
    assert!(
        probe.second_crop.starts_with("illegal:")
            && probe
                .second_crop
                .contains("witness_criterion_true_without_read_faults"),
        "{probe:?}"
    );
}
