//! checker 档模块：crash
//! 两槽全废盘面观察者（`singlefs_checker_tier::crash_point_replay_provenance` 那一节，跑流的两处按来历表统一挂；G4 第三轮判决丙-置，用户 2026-10-22 定「按流登记+相对基镜像」
//! 与「你做你的 GPU版让GPU去做」，`research/prompts/m3-close-g4-rewrite-r3-main-verification.md`）的快档用例，判段口径：观察者在判段看判器交回的逐盘自证过的
//! 系统配置槽数（没配 GPU 时判段 CPU 那一路交的是 CPU 参照 `crash_judge_conclusions::self_verified_system_configuration_slots_of_each_device`，配了 GPU 是走读内核
//! 第 25 字，两边逐状态对拍在 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`），对这条流基镜像那一份比。手造的崩溃状态各取 CPU 参照那一份
//! 逐盘槽数交观察者看，钉住三件事——撕裂之后某块盘两槽全废（流的基镜像上它还有自证过的槽）判红；基镜像上已经两槽全废的盘不再记；全池皆废留给 I-8.5、
//! 观察者不重记。
//! 崩溃状态照崩溃点重放的枚举写表摆：录制流里的写在前、原地覆写的撕裂镜像接在后面，「原来那次写没持久、它的撕裂镜像持久」就是撕裂那一态
//! （`singlefs_checker_tier::crash` 文件头的枚举规则）；撕裂镜像的字节照 `crash::torn_image_of_in_place_overwrite` 的取法（旧新不同的那一截
//! 前一半取新、后一半取旧；那一份只在 crate 里可见，这里留一份）。基镜像是两块盘、第一个文件之后的池（`common::build_pool`）。
//! 判别力在 `crates/mutations.tsv`（名字以「两槽全废盘面观察者」起头的几行）：观察者不判、判段不比基镜像、判段恒有槽，各让这里一条红；
//! 判段那一判本身的三条在 `crash_judge_conclusions` 的库内单测，判段怎么从库里读回判器交的逐盘槽数在 `crash_amplification` 的库内单测。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, geometry};
use singlefs_checker::image::{
    system_configuration_redundancy_report, SystemConfigurationRedundancyReport,
};
use singlefs_checker_tier::crash::CrashPointReplayObserverCounts;
use singlefs_checker_tier::crash_judge_conclusions::{
    devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts,
    self_verified_system_configuration_slots_of_each_device,
};
use singlefs_checker_tier::crash_point_replay_provenance::{
    observe_the_devices_without_a_self_verified_system_configuration_slot,
    DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage,
    EVERY_DEVICE_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_COUNTER,
    SYSTEM_CONFIGURATION_SLOTS_JUDGED_AGAINST_THE_BASE_IMAGE_COUNTER,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::recovery::PoolReader;
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::memory_pool::{CrashImage, MemoryPool, RetainedWrite, WrittenContents};
use singlefs_harness::segments::StepKind;

/// 一块盘系统配置槽 `slot_index` 的设备内偏移：槽 i 在 i × 固定结构槽距（D22（单元原子性怎么合成） 已定项 16）。
fn system_configuration_slot_offset(slot_index: u64) -> DeviceOffsetInBytes {
    DeviceOffsetInBytes(slot_index * u64::from(geometry().fixed_structure_slot_spacing))
}

fn system_configuration_slot_bytes() -> usize {
    usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节装得进 usize")
}

fn read_system_configuration_slot(pool: &MemoryPool, device: u32, slot_index: u64) -> Vec<u8> {
    PoolReader::read(
        pool,
        DeviceIdentity(device),
        system_configuration_slot_offset(slot_index),
        system_configuration_slot_bytes(),
    )
    .expect("第一个文件之后的池里系统配置槽读得出")
}

/// 往盘 `device` 的系统配置槽 `slot_index` 上的一次写（整槽）。
fn system_configuration_slot_write(
    device: u32,
    slot_index: u64,
    contents: WrittenContents,
) -> RetainedWrite {
    RetainedWrite {
        device: DeviceIdentity(device),
        kind: StepKind::SystemConfigurationSlot,
        is_force_unit_access: false,
        offset: system_configuration_slot_offset(slot_index),
        contents,
    }
}

/// 把盘 `device` 的槽 `slot_index` 整槽清零的一次写。
fn system_configuration_slot_zeroed(device: u32, slot_index: u64) -> RetainedWrite {
    system_configuration_slot_write(
        device,
        slot_index,
        WrittenContents::Zeros {
            length: SYSTEM_CONFIGURATION_SLOT_BYTES,
        },
    )
}

/// 原地覆写写到一半的那份字节：旧字节与新字节不同的那一截，前一半取新、后一半取旧（与崩溃点重放撕裂第三态同一个取法）。
fn torn_image_of_an_in_place_overwrite(old: &[u8], new: &[u8]) -> Vec<u8> {
    assert_eq!(old.len(), new.len(), "旧字节与这次写一样长");
    let differs = |index: &usize| old[*index] != new[*index];
    let first_differing_byte = (0..new.len())
        .find(differs)
        .expect("前提：这次原地覆写的新旧字节有不同的（不同才撕得出第三态）");
    let last_differing_byte = (0..new.len())
        .rev()
        .find(differs)
        .expect("有第一个不同的字节就有最后一个");
    let differing_span_in_bytes = last_differing_byte - first_differing_byte + 1;
    let first_old_byte = first_differing_byte + differing_span_in_bytes / 2;
    let mut torn = new.to_vec();
    torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);
    torn
}

/// 判段那一判：这个状态的逐盘槽数（判段 CPU 那一路交的 CPU 参照）对基镜像那一份（同一个 CPU 参照）比的结局。
fn judged_against_the_base_image(
    base: &MemoryPool,
    image: &CrashImage<'_>,
) -> DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage {
    devices_without_a_self_verified_system_configuration_slot_against_the_base_image_from_the_slot_counts(
        &self_verified_system_configuration_slots_of_each_device(image),
        &self_verified_system_configuration_slots_of_each_device(base),
    )
}

/// 观察者在判段看一个状态：交回它判红没有（panic 就是红，判段照样接住记成红项）与它记的数。交给它的是判段 CPU 那一路交回的这个状态的
/// 逐盘自证槽数与基镜像那一份（判段写结论块时连同存下的那两份）。
fn observer_on_the_state(
    base: &MemoryPool,
    image: &CrashImage<'_>,
) -> (bool, CrashPointReplayObserverCounts) {
    let mut counts = CrashPointReplayObserverCounts::default();
    let slots_on_the_crash_image = self_verified_system_configuration_slots_of_each_device(image);
    let slots_on_the_base_image = self_verified_system_configuration_slots_of_each_device(base);
    let reddened = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        observe_the_devices_without_a_self_verified_system_configuration_slot(
            &slots_on_the_crash_image,
            &slots_on_the_base_image,
            &mut counts,
        );
    }))
    .is_err();
    (reddened, counts)
}

/// 场景：第一个文件之后的池上，盘 0 一段里写了两次系统配置槽——槽 0 原地覆写成新内容（取盘 0 槽 1 那一份自证过的字节），槽 1 整槽清零
/// （攻方坏实现 MUT-B1 那一形：一段里一块盘的系统配置槽写不止一个）。枚举写表是这两次写，再接槽 0 那次写的撕裂镜像。
/// 预期：槽 0 撕裂、槽 1 清零落了的那一态，判段 CPU 那一路交回的逐盘槽数里盘 0 是 0、基镜像上它两槽都有，观察者判红、点名盘 0；
/// 这一段里只取整写的那几态（槽 0 没落或整落）盘 0 都还有一槽自证过，观察者不红——两槽全废只出在撕裂那一态上。
#[test]
fn a_rotation_torn_on_device_zero_while_its_other_slot_is_zeroed_reddens_the_observer_against_the_base_image(
) {
    let base = build_pool("two-slot-loss-observer-torn-rotation").memory_pool();
    assert_eq!(
        system_configuration_redundancy_report(&base),
        SystemConfigurationRedundancyReport {
            devices_without_a_self_verified_slot: Vec::new(),
            devices_with_fewer_than_two_self_verified_slots: Vec::new(),
        },
        "前提：基镜像上两块盘两槽都自证过"
    );
    let old_slot_zero = read_system_configuration_slot(&base, 0, 0);
    let new_slot_zero = read_system_configuration_slot(&base, 0, 1);
    let writes = vec![
        system_configuration_slot_write(0, 0, WrittenContents::Bytes(new_slot_zero.clone())),
        system_configuration_slot_zeroed(0, 1),
        system_configuration_slot_write(
            0,
            0,
            WrittenContents::Bytes(torn_image_of_an_in_place_overwrite(
                &old_slot_zero,
                &new_slot_zero,
            )),
        ),
    ];
    let torn = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![false, true, true],
    };
    assert_eq!(
        judged_against_the_base_image(&base, &torn),
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::New(vec![0]),
        "撕裂那一态：盘 0 槽 0 新旧都读不出、槽 1 清零了，基镜像上它两槽都有"
    );
    let (reddened, counts) = observer_on_the_state(&base, &torn);
    assert!(reddened, "撕裂那一态上观察者判红（panic）");
    assert_eq!(
        counts.count_named(SYSTEM_CONFIGURATION_SLOTS_JUDGED_AGAINST_THE_BASE_IMAGE_COUNTER),
        1,
        "判红之前先记一个看过的状态"
    );
    let slot_zero_not_persisted = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![false, true, false],
    };
    assert!(
        !observer_on_the_state(&base, &slot_zero_not_persisted).0,
        "槽 0 那次写没落、槽 1 清零落了：盘 0 槽 0 旧的那一份还自证过，观察者不红"
    );
    let slot_zero_persisted_whole = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true, true, false],
    };
    assert!(
        !observer_on_the_state(&base, &slot_zero_persisted_whole).0,
        "槽 0 那次写整落、槽 1 清零落了：盘 0 槽 0 新的那一份自证过，观察者不红"
    );
}

/// 场景：基镜像上盘 1 两槽已经清零（补写流那一形：流的起点就有一块死盘），这一态上盘 0 的槽 0 整写落了一份自证过的新内容。
/// 预期：盘 1 在崩溃镜像上一个自证过的槽都没有，但它在基镜像上已经如此，不算新出现的；观察者不红、只记一个看过的状态。
#[test]
fn a_device_already_without_a_self_verified_slot_on_the_base_image_is_not_reported_again() {
    let healthy = build_pool("two-slot-loss-observer-dead-on-the-base").memory_pool();
    let mut base = healthy.clone();
    base.apply_writes(&[
        system_configuration_slot_zeroed(1, 0),
        system_configuration_slot_zeroed(1, 1),
    ]);
    assert_eq!(
        system_configuration_redundancy_report(&base).devices_without_a_self_verified_slot,
        vec![1],
        "前提：基镜像上盘 1 两槽都没自证过、盘 0 有"
    );
    let writes = vec![system_configuration_slot_write(
        0,
        0,
        WrittenContents::Bytes(read_system_configuration_slot(&base, 0, 1)),
    )];
    let state = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true],
    };
    assert_eq!(
        judged_against_the_base_image(&base, &state),
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::NoneNew,
        "盘 1 在基镜像上就两槽全废，相对基镜像不是新出现的"
    );
    let (reddened, counts) = observer_on_the_state(&base, &state);
    assert!(!reddened, "基镜像上已经死的盘不再记，观察者不红");
    assert_eq!(
        counts.count_named(SYSTEM_CONFIGURATION_SLOTS_JUDGED_AGAINST_THE_BASE_IMAGE_COUNTER),
        1,
        "看过这一个状态"
    );
}

/// 场景：第一个文件之后的池上，两块盘四个系统配置槽都清零落了（全池皆废）。
/// 预期：这一态归 I-8.5（全池皆废判红），观察者不重记、不红，只把它数进全池皆废那一项。
#[test]
fn every_device_without_a_self_verified_slot_is_left_to_the_tail_slot_invariant() {
    let base = build_pool("two-slot-loss-observer-every-device").memory_pool();
    let writes: Vec<RetainedWrite> = [0, 1]
        .into_iter()
        .flat_map(|device| {
            [0, 1]
                .into_iter()
                .map(move |slot_index| system_configuration_slot_zeroed(device, slot_index))
        })
        .collect();
    let state = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true; writes.len()],
    };
    assert_eq!(
        judged_against_the_base_image(&base, &state),
        DevicesWithoutASelfVerifiedSystemConfigurationSlotAgainstTheBaseImage::EveryDeviceLeftToTheTailSlotInvariant,
        "两块盘都一个自证过的槽都没有：全池皆废"
    );
    let (reddened, counts) = observer_on_the_state(&base, &state);
    assert!(!reddened, "全池皆废归 I-8.5，观察者不重记");
    assert_eq!(
        counts.count_named(EVERY_DEVICE_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_COUNTER),
        1,
        "数进全池皆废那一项"
    );
}
