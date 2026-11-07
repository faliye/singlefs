//! 验证包模块：crash、crash_state_enumeration、crash_amplification
//! 取号补写两槽全废的盘、补写中途崩溃（C874（补写那一支进不了崩溃测试、运行时看不出）：补写那一支要进崩溃测试；判决
//! `research/prompts/m3-close-b19-core-r1-main-verification.md` 第二节 R4「崩溃流罩取号写两轮的两段」；D22（单元原子性怎么合成） 已定项 8
//! 依据「取号那一步的补写例外」）。
//!
//! 流：单元区 384 槽那一档的两块内存小盘（`tests/common_admission_pool/mod.rs`），mkfs 同一个进程里取号 1、暖机、第一个文件、覆盖写一次
//! （所选那一版 txg 4）；盘 1 两个系统配置槽整槽写 0（欠账行「补写流的造法是第一个文件之后的池把一块盘两个系统配置槽清零当基镜像」，不手造写表），
//! 这一份是基镜像。录的是之后那一次可写挂载：盘 1 过了补写闸，取号写两轮把它两槽补写成新号 2（每轮逐盘写一次系统配置槽、轮后一道池屏障），
//! 再写行、暖机。取号写两轮各自成一段、各两写（盘 0、盘 1 各一）。盘 0 的写覆写读得出的旧槽，是原地覆写，取三态；盘 1 的两写落在整槽全 0 的槽上，
//! 照崩溃点重放的认法（`singlefs_verification::crash` 的 `TearableInPlaceOverwrites` 第 3 条「罩住的范围里原来有东西」）不算原地覆写、只取两态，
//! 所以每段的真子集是 3 · 2 − 1 = 5 个，不是两写都取三态时的 3² − 1 = 8 个（判决与两条腿按 8 个推的，没看这一条认法）。
//! 每个状态恢复、oracle、池级 checker、记录核对器，再当交回镜像的观察者重开可写挂载一次：挂得上，取到的号比崩溃镜像上读得出的每一个实例代号都大，
//! 挂完之后两块盘各有两槽自证过的系统配置（盘 1 在这个状态上还两槽全废的，重开那次再过一次补写闸、再补写；落了头一轮那一写的，重开那次照落后支挂上、
//! 取号写两轮把它两槽都写成新号）。槽数由池级 checker 那一份读法读（`singlefs_checker::image`），不用实现的读法。
//! 按来历挂的两槽全废盘面观察者（`singlefs_verification::crash_point_replay_provenance`）对基镜像比：盘 1 在基镜像上就两槽全废，不记；
//! 这条流在来历表里要有一行才跑得起（`PROVENANCE_OF_THE_FLOWS` 的 `acquisition_rewriting_a_device_without_a_self_verified_system_configuration`）。
//! 那个观察者「只判整写不判撕裂」在这条流上是等价变异（判决 `research/prompts/m3-close-b19-design-r2-main-verification.md` 第二节 Q10），
//! 等价性由 `the_intact_device_keeps_one_self_verified_slot_or_more_on_every_crash_state_of_the_rewrite_so_tearing_never_changes_the_dead_device_verdict` 留档。
//!
//! **比已有的流多罩了什么**：`crash_enumeration_crash_in_the_middle_of_the_acquisition_write_stream.rs` 的基镜像每块盘两槽都自证过，取号写不补写；
//! 这一条的基镜像盘 1 两槽全废，取号写两轮里盘 1 那两写是补写，崩在它们中间的状态（盘 1 仍两槽全废、只落了槽 1）只有这条流摆得出。
//! **多跑的一步**：每个状态上重开可写挂载，判补写闸再判一次（或照落后支挂上）之后两块盘两槽都补齐、取号不重号。
//! 补写失败的回卷（补写闸第六件）不在这条流里：盘报写错的分支录不到，由验证包的
//! `acquisition_rewriting_a_device_without_a_self_verified_system_configuration_rolls_back_on_a_write_failure.rs` 用写故障罩。

mod common_admission_pool;

use common_admission_pool::{
    device_bytes_of_the_384_slot_small_devices, fixed_geometry_of_the_384_slot_small_devices,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, PoolUnderTest,
    OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::root_record::RootRecord;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_verification::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_verification::crash_amplification::{
    ImageAfterTheObserver, InvariantsJudgedOnTheImageAfterTheObserver, ObserverDeduplication,
    ReadRangeOnTheCrashImage,
};
use singlefs_verification::crash_point_replay_provenance::{
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow, RunOfTheFlow,
    EVERY_DEVICE_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_COUNTER,
};
use singlefs_verification::crash_state_enumeration::{
    enumerate_crash_states_with_every_kind_of_observer, CrashPointReplayItem,
    CrashPointReplayObserverHandingBackAnImage,
};
use singlefs_verification::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str = "crates/singlefs-verification/tests/crash_enumeration_acquisition_rewriting_a_device_without_a_self_verified_system_configuration_stream.rs";

/// 两槽全废、被取号补写的盘。
const DEVICE_LOSING_BOTH_SLOTS: DeviceIdentity = DeviceIdentity(1);
/// 基镜像上两槽都自证过的盘：取号写每轮覆写它一槽（原地覆写，取三态），不补写。
const DEVICE_KEEPING_ITS_SLOTS: DeviceIdentity = DeviceIdentity(0);
/// 这条流在来历表里的名字（崩溃点重放项名第一个 `::` 之前那一截）。
const FLOW_NAME: &str =
    "acquisition_rewriting_a_device_without_a_self_verified_system_configuration";
/// 补写之前那段历史只有实例 1（mkfs 同一个进程取号 1）。
const INSTANCE_BEFORE_THE_ACQUISITION: InstanceGeneration = InstanceGeneration(1);
/// 取号写两轮，每轮之后一道池屏障，各成一段，排在录制流最前面。
const ACQUISITION_SEGMENTS: usize = 2;
/// 观察者每个状态加一的计数：重开可写挂载做成、取到的号比崩溃镜像上读得出的每一个实例代号都大、挂完之后两块盘各两槽自证过。
const REMOUNTED_WITH_TWO_SELF_VERIFIED_SLOTS_ON_EVERY_DEVICE: &str =
    "remounted_with_two_self_verified_slots_on_every_device";
/// 崩溃镜像上盘 1 一槽自证过的系统配置都没有的状态数（补写头一轮盘 1 那一写没落）。
const DEVICE_STILL_WITHOUT_A_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE: &str =
    "device_still_without_a_self_verified_slot_on_the_crash_image";
/// 崩溃镜像上盘 0 只剩一槽自证过的状态数（盘 0 那一写撕裂：撕裂镜像新旧两份校验和都对不上，另一槽这一轮没碰）。
const INTACT_DEVICE_WITH_ONE_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE: &str =
    "intact_device_with_one_self_verified_slot_on_the_crash_image";
/// 崩溃镜像上盘 0 一槽自证过的都没有的状态数（基镜像上它两槽都自证过：这是新出现的死盘；两轮之间有池屏障时一个都没有）。
const INTACT_DEVICE_WITHOUT_ANY_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE: &str =
    "intact_device_without_any_self_verified_slot_on_the_crash_image";
/// 判段在观察者交回的盘上判干净时加一的计数。
const CHECKER_CLEAN_AFTER_THE_REMOUNT: &str = "checker_clean_after_the_remount";

/// 录好的补写那次挂载：基镜像（盘 1 两槽全废）、那次挂载的写表与段、被判的根槽写（暖机最后那一次）、允许的版本、取号那几写。
struct RecordedRewritingMount {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    acquisition_writes: Vec<usize>,
}

/// 第一个文件之后覆盖写一次的池，盘 1 两个系统配置槽整槽写 0：交回这一份盘面与文件内容。
fn base_image_with_both_system_configuration_slots_of_device_one_zeroed() -> (MemoryPool, Vec<u8>) {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    pool.overwrite(OVERWRITE_BYTES)
        .expect("384 槽小盘上第一个文件之后覆盖写一次落得下");
    let content = pool.content_of_the_current_version.clone();
    let mut image = pool.image();
    let spacing = u64::from(
        parameters_of_the_384_slot_small_devices()
            .geometry
            .fixed_structure_slot_spacing,
    );
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_image = image
        .devices
        .get_mut(&DEVICE_LOSING_BOTH_SLOTS)
        .expect("池里有盘 1");
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        device_image.write(
            DeviceOffsetInBytes(slot_index * spacing),
            &vec![0u8; slot_bytes],
        );
    }
    (image, content)
}

/// 每块盘上自证过、可择的系统配置槽数（池级 checker 那一份读法），按盘号。
fn self_verified_slots_of_each_device(image: &MemoryPool) -> Vec<(u32, usize)> {
    singlefs_checker::image::verified_system_configuration_slots(image)
        .into_iter()
        .map(|(device, usable)| (device, usable.len()))
        .collect()
}

fn record_the_rewriting_mount() -> RecordedRewritingMount {
    let (base, content) = base_image_with_both_system_configuration_slots_of_device_one_zeroed();
    assert!(
        self_verified_slots_of_each_device(&base).contains(&(DEVICE_LOSING_BOTH_SLOTS.0, 0)),
        "前提：基镜像上盘 1 两槽全废"
    );
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<_> = plain_devices_on(&base)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_space_admission(
        &parameters_of_the_384_slot_small_devices(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("盘 1 过了补写闸：可写挂载做成");
    assert_eq!(
        mounted.output.devices_rewritten_by_the_acquisition,
        vec![DEVICE_LOSING_BOTH_SLOTS],
        "取号写两轮补写盘 1"
    );
    let (writes, segments) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let acquisition_writes: Vec<usize> = writes
        .iter()
        .take_while(|write| write.kind == StepKind::SystemConfigurationSlot)
        .enumerate()
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        acquisition_writes
            .iter()
            .map(|index| writes[*index].device)
            .collect::<Vec<_>>(),
        vec![
            DeviceIdentity(0),
            DEVICE_LOSING_BOTH_SLOTS,
            DeviceIdentity(0),
            DEVICE_LOSING_BOTH_SLOTS
        ],
        "取号写两轮，每轮逐盘写一次系统配置槽，盘 1 那两写是补写"
    );
    let filesystem_identifier = parameters_of_the_384_slot_small_devices().filesystem_identifier;
    let mut versions = vec![PublishedVersion {
        instance: INSTANCE_BEFORE_THE_ACQUISITION,
        checkpoint_txg: mounted.output.effective_root.checkpoint_txg,
        content: content.clone(),
    }];
    for write in writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
    {
        let root =
            RootRecord::parse_slot(write.bytes().expect("根槽写带字节"), &filesystem_identifier)
                .expect("补写那次挂载写的根自证得过");
        versions.push(PublishedVersion {
            instance: root.instance,
            checkpoint_txg: root.checkpoint_txg,
            content: content.clone(),
        });
    }
    RecordedRewritingMount {
        base,
        judged_root_index: writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("写行与暖机写了根槽"),
        writes,
        segments,
        versions,
        acquisition_writes,
    }
}

/// 取号写两轮各自成一段、排在录制流最前面（只展开取号那两段的那一条按段号挑这两段，靠的就是这一形）。等价留档那一条不调它：
/// 它按写表取装着取号写的段，两轮之间的屏障被改坏时照样枚举下去，由观察者判红。
fn assert_the_two_acquisition_rounds_are_segments_of_their_own(recorded: &RecordedRewritingMount) {
    assert_eq!(
        recorded.segments[..ACQUISITION_SEGMENTS].concat(),
        recorded.acquisition_writes,
        "取号写之前一道屏障、每轮之后一道池屏障：两轮各两写、各自成一段（D23（journal 的角色与格式） 已定项 16「取号那一步的屏障」）"
    );
}

/// 盘上读得出的最大实例代号：每块盘两槽里可择的系统配置的实例代号与根环里自证过的根的实例代号取最大（池级 checker 那一份读法）。
fn highest_instance_readable_on(image: &MemoryPool) -> u32 {
    let slots = singlefs_checker::image::verified_system_configuration_slots(image);
    let geometry = slots
        .iter()
        .find_map(|(_, usable)| usable.first())
        .map(|(_, geometry)| geometry)
        .expect("盘 0 的取号写只覆写较旧那一槽：较新那一槽一直读得出，至少一块盘有可择的系统配置");
    let highest_in_the_slots = slots
        .iter()
        .flat_map(|(_, usable)| usable.iter().map(|(view, _)| view.journal_instance))
        .max()
        .expect("至少一个可择的槽");
    let highest_in_the_ring = singlefs_checker::image::valid_roots(image, geometry)
        .iter()
        .map(|(_, _, root)| root.instance)
        .max()
        .unwrap_or(0);
    highest_in_the_slots.max(highest_in_the_ring)
}

/// 观察者「补写中途崩溃之后重开」（`remount_after_the_crash_of_the_rewrite`）：把这个状态持久的写（撕裂镜像在内）落到基镜像上、可写挂载一次；
/// 挂不上、取到的号不比崩溃镜像上读得出的每一个实例代号都大、挂完之后有一块盘不是两槽都自证过，都当场 panic。挂完的盘交回，池级 checker 在判段判。
/// 观察者的身份是这个 fn 的闭包键（入口要登记在 `singlefs_verification::judging_units_entries::OBSERVER_ENTRIES`；没登记时取键拒算、照判不入库）。
// gpu-exempt: writable-mount
fn remount_after_the_crash_of_the_rewrite_with_both_slots_of_every_device_self_verified(
    base: &MemoryPool,
    image: &CrashImage<'_>,
    counts: &mut CrashPointReplayObserverCounts,
) -> ImageAfterTheObserver {
    let mut materialized = base.clone();
    for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
        if *is_persisted {
            write.contents.apply_to(
                materialized
                    .devices
                    .get_mut(&write.device)
                    .expect("写落在池里的盘上"),
                write.offset,
            );
        }
    }
    let slots_on_the_crash_image = self_verified_slots_of_each_device(&materialized);
    if slots_on_the_crash_image.contains(&(DEVICE_LOSING_BOTH_SLOTS.0, 0)) {
        counts.add(
            DEVICE_STILL_WITHOUT_A_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE,
            1,
        );
    }
    if slots_on_the_crash_image.contains(&(DEVICE_KEEPING_ITS_SLOTS.0, 1)) {
        counts.add(
            INTACT_DEVICE_WITH_ONE_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE,
            1,
        );
    }
    if slots_on_the_crash_image.contains(&(DEVICE_KEEPING_ITS_SLOTS.0, 0)) {
        counts.add(
            INTACT_DEVICE_WITHOUT_ANY_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE,
            1,
        );
    }
    let highest_on_the_crash_image = highest_instance_readable_on(&materialized);
    let stream = SharedStream::retaining_contents_and_reads();
    let mut devices: Vec<_> = plain_devices_on(&materialized)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    let mounted = mount_writable_with_space_admission(
        &parameters_of_the_384_slot_small_devices(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .unwrap_or_else(|error| panic!("补写中途崩溃之后重开可写挂载被拒：{error:?}"));
    assert!(
        mounted.output.instance.0 > highest_on_the_crash_image,
        "重号：重开取到 {:?}，崩溃镜像上已经读得出实例 {highest_on_the_crash_image}",
        mounted.output.instance
    );
    let disks_after_the_observer = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: materialized.device_size_in_bytes,
    };
    let slots_after_the_remount = self_verified_slots_of_each_device(&disks_after_the_observer);
    assert!(
        slots_after_the_remount
            .iter()
            .all(|(_, self_verified_slots)| *self_verified_slots == 2),
        "重开挂完之后每块盘两槽都自证过（盘 1 补齐）：{slots_after_the_remount:?}"
    );
    counts.add(REMOUNTED_WITH_TWO_SELF_VERIFIED_SLOTS_ON_EVERY_DEVICE, 1);
    let (writes_on_the_crash_image, _segments_of_the_remount) = writes_and_segments(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    ImageAfterTheObserver {
        writes_on_the_crash_image,
        disks_after_the_observer,
        ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(
            &stream.read_ranges(),
        ),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: CHECKER_CLEAN_AFTER_THE_REMOUNT.to_string(),
    }
}

/// 按 `expansion` 枚举补写那次挂载的写，每个状态挂上重开挂载的观察者。只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）流写进判定库就交回 `None`。
fn enumerate_remounting_after_each_crash(
    recorded: &RecordedRewritingMount,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts|
     -> ImageAfterTheObserver {
        remount_after_the_crash_of_the_rewrite_with_both_slots_of_every_device_self_verified(
            &recorded.base,
            image,
            counts,
        )
    };
    let item_name = format!("{FLOW_NAME}::{item_label}");
    enumerate_crash_states_with_every_kind_of_observer(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&recorded.versions),
        expansion,
        &[],
        // 观察者另读的输入：挂载的参数与补写之前那段历史的实例代号——不随状态变，进版本与去重键时只取它们的摘要
        &[CrashPointReplayObserverHandingBackAnImage {
            name: "remount_after_the_crash_of_the_rewrite",
            extra_inputs_digest: singlefs_verification::sha256::sha256_digest(
                format!(
                    "parameters={:?}\ninstance_before_the_acquisition={}\n",
                    parameters_of_the_384_slot_small_devices(),
                    INSTANCE_BEFORE_THE_ACQUISITION.0
                )
                .as_bytes(),
            ),
            observe: &observe,
            deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,
        }],
    )
    .judged()
}

fn assert_recovery_checker_record_checker_and_remounts_clean(tally: &CrashPointReplayTally) {
    assert_eq!(
        (tally.violations, tally.first_violation.clone()),
        (0, None),
        "恢复之后多版本 oracle 0 违例"
    );
    assert_eq!(
        (
            tally.ignored_violations,
            tally.first_ignored_violation.clone()
        ),
        (0, None),
        "不看 journal 那一遍恢复同样过 oracle"
    );
    assert_eq!(tally.failed_states, 0, "每个状态都恢复得了");
    assert_eq!(
        tally.checker_violated_states.values().sum::<u64>(),
        0,
        "恢复之后池级 checker 0 违例：{:?}",
        tally.checker_first_violation
    );
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(REMOUNTED_WITH_TWO_SELF_VERIFIED_SLOTS_ON_EVERY_DEVICE),
        tally.states,
        "每个崩溃状态之后都重开得了可写挂载，取号不重号，挂完每块盘两槽都自证过"
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(CHECKER_CLEAN_AFTER_THE_REMOUNT),
        tally.states,
        "每个崩溃状态重开挂载之后的盘，池级 checker 判段一条都不判违例"
    );
}

/// 只展开取号写那两段（补写盘 1 的两轮），别的段只以整段持久进入后面的状态：每段盘 0 那一写三态、盘 1 那一写两态，真子集 3 · 2 − 1 = 5 个，
/// 1 + 2 × 5 = 11 个状态，最后一个是补写那次挂载全落。每个状态恢复、oracle、池级 checker、记录核对器，再重开可写挂载、判取号不重号与两槽补齐。
/// 盘 1 两槽全废的状态只在第一段里、盘 1 那一写没落的那几个：盘 0 那一写三态各一个，闭式 3 个（第二段里头一轮已整段落下，盘 1 槽 1 自证过）。
/// 盘 0 只剩一槽自证过的状态是盘 0 那一写撕裂的那几个：每段盘 0 那一写撕裂、盘 1 那一写两态各一个，闭式 2 × 2 = 4 个；一槽都没有的 0 个。
/// 判别力（`crates/mutations.tsv`）：「C874 补写流：取号写两轮之间不过屏障」两轮并成一段，第一段不再是两写，这条红在段形；
/// 「C874 补写流枚举侧：撕裂镜像退化成整次新写」盘 0 那一写的撕裂态与落了态成了同一张盘，盘 0 只剩一槽的状态数从 4 掉到 0，这条红。
/// 盘 1 两槽全废的那个数（3）对后一行不敏感：盘 1 那两写落在整槽全 0 的槽上、本来就只取两态，撕不撕裂它都不变。
#[test]
fn every_crash_in_the_middle_of_the_acquisition_rewriting_a_device_remounts_with_both_slots_of_every_device_self_verified(
) {
    let recorded = record_the_rewriting_mount();
    assert_the_two_acquisition_rounds_are_segments_of_their_own(&recorded);
    let expand_the_acquisition_segments_only = |segment_index: usize, _segment: &[usize]| {
        if segment_index < ACQUISITION_SEGMENTS {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let states = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expand_the_acquisition_segments_only,
    );
    assert_eq!(
        states,
        1 + 2 * (3 * 2 - 1),
        "取号写两轮各两写：盘 0 那一写原地覆写取三态，盘 1 那一写落在整槽全 0 的槽上取两态，每轮真子集 3 · 2 − 1，再加补写那次挂载全落那一个"
    );
    let Some(tally) = enumerate_remounting_after_each_crash(
        &recorded,
        "acquisition_segment",
        &expand_the_acquisition_segments_only,
    ) else {
        return;
    };
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        states,
        "只展开取号那两段的状态数照崩溃点重放的口径另算",
    );
    assert_recovery_checker_record_checker_and_remounts_clean(&tally);
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        tally
            .observer_counts
            .count_named(DEVICE_STILL_WITHOUT_A_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE),
        3u64,
        "盘 1 仍两槽全废的状态：第一段里盘 1 那一写没落、盘 0 那一写三态各一个（撕裂态不并进落了态）",
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        tally
            .observer_counts
            .count_named(INTACT_DEVICE_WITH_ONE_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE),
        2u64 * 2,
        "盘 0 只剩一槽自证过的状态：每段盘 0 那一写撕裂（撕裂镜像自证不过、另一槽这一轮没碰）、盘 1 那一写两态各一个，两段共 4 个",
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(INTACT_DEVICE_WITHOUT_ANY_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE),
        0,
        "盘 0 在每个状态上至少一槽自证过（两轮之间有池屏障）"
    );
}

/// 全量：补写那次挂载的每一段都展开（D13（验证路线） 已定项 4），状态数照崩溃点重放的口径按闭式算；每个状态恢复、oracle、池级 checker、记录核对器，
/// 再重开可写挂载、判取号不重号与两槽补齐。登记成崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的
/// `crash-case:acquisition-rewriting-a-device-without-a-self-verified-system-configuration-stream`），门禁
/// crash-point-replay 在 release 下跑，认下面打的 `CRASH_POINT_REPLAY_ACQUISITION_REWRITING_A_DEVICE` 行里 `exhaustive=true`。
/// 判别力同只展开取号那两段的那一条（取号写两轮之间不过屏障）。
#[test]
fn every_crash_state_of_the_mount_rewriting_a_device_remounts_with_both_slots_of_every_device_self_verified(
) {
    let recorded = record_the_rewriting_mount();
    assert_the_two_acquisition_rounds_are_segments_of_their_own(&recorded);
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    let Some(tally) = enumerate_remounting_after_each_crash(&recorded, "full", &full_expansion)
    else {
        return;
    };
    // 门禁 crash-point-replay 只认行首 `CRASH_POINT_REPLAY_ACQUISITION_REWRITING_A_DEVICE ` 与 `exhaustive=true`，
    // 线程按 states= 找 CRASH_POINT_REPLAY_PARALLEL_FINISHED 那一行判；整行原样报出来。
    println!(
        "CRASH_POINT_REPLAY_ACQUISITION_REWRITING_A_DEVICE states={} closed_form={closed_form} exhaustive={} violations={} ignored_violations={} failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={} {REMOUNTED_WITH_TWO_SELF_VERIFIED_SLOTS_ON_EVERY_DEVICE}={} {DEVICE_STILL_WITHOUT_A_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE}={} {CHECKER_CLEAN_AFTER_THE_REMOUNT}={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.ignored_violations,
        tally.failed_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.checker_violated_states.values().sum::<u64>(),
        tally
            .observer_counts
            .count_named(REMOUNTED_WITH_TWO_SELF_VERIFIED_SLOTS_ON_EVERY_DEVICE),
        tally
            .observer_counts
            .count_named(DEVICE_STILL_WITHOUT_A_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE),
        tally
            .observer_counts
            .count_named(CHECKER_CLEAN_AFTER_THE_REMOUNT),
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    singlefs_verification::crash_state_enumeration::assert_the_counted_states_equal_the_closed_form_unless_narrowed(
        tally.states,
        closed_form,
        "acquisition_rewriting_a_device_without_a_self_verified_system_configuration_stream：用例自己的展开下的闭式",
    );
    assert_recovery_checker_record_checker_and_remounts_clean(&tally);
}

/// 两槽全废盘面观察者「只判整写不判撕裂」在这条流上是等价变异，这一条留档（判决 `research/prompts/m3-close-b19-design-r2-main-verification.md`
/// 第二节 Q10；`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」等价变异那一行）。
/// 那个观察者（`singlefs_verification::crash_point_replay_provenance::observe_the_devices_without_a_self_verified_system_configuration_slot`）
/// 只判「基镜像上有自证过的槽、崩溃镜像上一槽都没有」的盘。盘 1 在基镜像上就两槽全废，不归它判；撕裂只动盘 0：
/// 盘 0 每一轮只覆写一槽、两轮之间一道池屏障，头一轮那一写撕裂时另一槽这一轮没碰，后一轮那一写撕裂时头一轮那一槽已经落了，
/// 所以盘 0 在每个状态上至少一槽自证过。撕裂与否只让盘 0 在一槽与两槽之间换，跨不过「一槽都没有」：把撕裂当整写、或撕裂态不判的观察者，
/// 在这条流的每个状态上都与原观察者同判（不记新死盘）。
/// 钉住的两件：撕裂态在这条流上造得出（盘 0 只剩一槽的状态，闭式 2 × 2 = 4 个，不是 0：等价不是因为流上没有撕裂态）；
/// 盘 0 一槽都没有的状态 0 个，按来历挂的两槽全废盘面观察者看过每一个状态、一个都没判红（全池皆废的 0 个）。
/// 展开哪几段不按段号写死，按录下来的写表取装着取号写的段，也不先核段形：两轮之间的屏障被改坏时（`crates/mutations.tsv`
/// 里 C874 补写流那一行），两轮并成一段，盘 0 两写可以同时撕裂、两槽都废，两槽全废盘面观察者在那几个状态上判红，
/// 这一条红在观察者上（这条流上观察者真正的判别力）。
#[test]
fn the_intact_device_keeps_one_self_verified_slot_or_more_on_every_crash_state_of_the_rewrite_so_tearing_never_changes_the_dead_device_verdict(
) {
    let recorded = record_the_rewriting_mount();
    let segments_holding_an_acquisition_write: Vec<usize> = recorded
        .segments
        .iter()
        .enumerate()
        .filter(|(_, segment)| {
            segment
                .iter()
                .any(|write_index| recorded.acquisition_writes.contains(write_index))
        })
        .map(|(segment_index, _)| segment_index)
        .collect();
    let expand_the_segments_holding_an_acquisition_write =
        |segment_index: usize, _segment: &[usize]| {
            if segments_holding_an_acquisition_write.contains(&segment_index) {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        };
    let Some(tally) = enumerate_remounting_after_each_crash(
        &recorded,
        "segments_holding_an_acquisition_write",
        &expand_the_segments_holding_an_acquisition_write,
    ) else {
        return;
    };
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        FLOW_NAME,
        &tally,
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(EVERY_DEVICE_WITHOUT_A_SELF_VERIFIED_SYSTEM_CONFIGURATION_SLOT_COUNTER),
        0,
        "盘 0 在每个状态上至少一槽自证过：没有全池皆废的状态"
    );
    assert_eq!(
        tally
            .observer_counts
            .count_named(INTACT_DEVICE_WITHOUT_ANY_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE),
        0,
        "盘 0 在每个状态上至少一槽自证过：撕裂跨不过「一槽都没有」，两槽全废盘面观察者把撕裂当整写也判得一样"
    );
    singlefs_verification::crash_state_enumeration::assert_the_count_equals_its_full_domain_pin_unless_narrowed(
        tally
            .observer_counts
            .count_named(INTACT_DEVICE_WITH_ONE_SELF_VERIFIED_SLOT_ON_THE_CRASH_IMAGE),
        2u64 * 2,
        "撕裂态在这条流上造得出：每段盘 0 那一写撕裂、盘 1 那一写两态各一个，两段共 4 个盘 0 只剩一槽的状态",
    );
}
