//! checker 档模块：crash、crash_state_enumeration、crash_judge_tables
//! io_min 8192（固定结构槽距 8192）的池上第一次可写挂载在崩溃点重放的证据：mkfs(io_min 8192) → 可写挂载（取号 1、写行发布 txg 1、暖机 txg 2），
//! 把 mkfs 之后的整条录制流按 D13（验证路线） 已定项 4 取崩溃状态，跑恢复 + 多版本 oracle、池级 checker、记录核对器。
//!
//! 比已有的流多罩的崩溃状态：已有的崩溃点重放流全是槽距 4096 的池（`crates/singlefs-harness/tests/common/mod.rs` 的 `parameters()`），
//! 这一条的基线镜像是槽距 8192 的池，系统配置槽 1 在 8192 处。取号那一段与暖机那次轮换那一段各有一个「两块盘的槽 0 同时撕裂」的状态
//! （全池没有可用的槽 0），恢复与池级 checker 都要逐档找槽 1（C651（没有可用槽 0 时 checker 与实现找槽 1 读法不同）
//! 第一轮判决第四节第 3 条，`research/prompts/m3-c651-r1-main-verification.md`）：checker 按最小槽距 4096 读时这两个状态判 I-8.5 红。
//! 多跑的一步：没有；流到第一次可写挂载为止（取号、两次零单元发布），不发文件版本，每个状态只跑恢复与 checker、不重开挂载。
//! 盘是内存稀疏盘外面包录制器（同一个进程里 mkfs 与挂载，不经文件镜像）。

use singlefs_checker::image::{
    chosen_system_configurations, system_configuration_slot_readings,
    SystemConfigurationSlotReading,
};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion,
    CrashPointReplayObserverCounts, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_quick_tier_versions, enumerate_crash_states_with_expansion,
    CrashPointReplayItem, CrashPointReplayObserver,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs";
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{choose_system_configuration, RecoveryReport};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const DEVICE_BYTES: u64 = 4 << 30;
const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-iomin8k";
/// mkfs 时探测的 io_min：槽距 = 4096 向上取整到它的整数倍 = 8192（D2（RAID 条带策略） 已定项 19）。
const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 8192;
const SLOT_SPACING_IN_BYTES: u64 = 8192;
/// 全量的状态数：七段，取号与两次轮换三个系统配置槽 2 写段（原地覆写各取三态）各 3² − 1，两个记录 2 写段各 2² − 1，
/// 两个根槽 1 写段各 1，再加全部持久那一个：1 + 3 × 8 + 2 × 3 + 2 × 1。
const FULL_STATES: u64 = 33;

fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: MINIMUM_INPUT_OUTPUT_BYTES,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(
                MINIMUM_INPUT_OUTPUT_BYTES,
            ),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn geometry() -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
        root_ring_slots_per_region: parameters().geometry.root_ring_slots_per_region,
    }
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// 被判的那次根槽 FUA 写：暖机那一代（txg 2）的根。
    judged_root_index: usize,
}

fn prepare() -> Prepared {
    assert_eq!(
        u64::from(parameters().geometry.fixed_structure_slot_spacing),
        SLOT_SPACING_IN_BYTES,
        "io_min 8192 的池槽距 8192"
    );
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters(), &mut devices).expect("io_min 8192 的池 mkfs");
    let mkfs_operation_count = stream.operations().len();
    mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], DEVICE_BYTES);
    base.apply(&operations[..mkfs_operation_count]);
    let (writes, segments) = writes_and_segments(&operations[mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 2],
        "取号两写 | 写行发布的记录两写 | 根 | 系统配置两写 | 暖机的记录两写 | 根 | 系统配置两写"
    );
    assert_eq!(
        segments
            .iter()
            .map(|segment| writes[segment[0]].kind)
            .collect::<Vec<StepKind>>(),
        vec![
            StepKind::SystemConfigurationSlot,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SystemConfigurationSlot,
            StepKind::JournalRecord,
            StepKind::RootRecordFua,
            StepKind::SystemConfigurationSlot,
        ],
        "七段的种类"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽写");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index,
    }
}

fn assert_recovery_checker_and_record_checker_clean(tally: &CrashPointReplayTally) {
    assert_eq!(
        tally.violations, 0,
        "多版本 oracle 的第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "每个状态都恢复得了");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据恒 0"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例的状态数必须是 0：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
}

/// 判表建得出（崩溃点重放合并第三轮判决 J14 第二批开工前的事实核：io_min 8192 的池判表建不建得出；J5「能走 GPU 的必须走」）：
/// 这条流整域展开交 GPU 判器建表，不拒（槽距 8192 的系统配置槽 1、根环槽都是落点），表里的状态数就是全量的 33 个。只建表、不上卡，
/// 不要 `gpu` 特性。
#[test]
fn the_gpu_judge_tables_of_the_first_writable_mount_of_an_io_min_8192_pool_build_with_its_33_crash_states(
) {
    use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
    let prepared = prepare();
    let tables = build_judge_tables(&JudgeFlow {
        base: &prepared.base,
        writes: &prepared.writes,
        segments: &prepared.segments,
        judged_root_index: prepared.judged_root_index,
        versions: &[],
        expansion: &full_expansion,
    })
    .unwrap_or_else(|refusal| {
        panic!("io_min 8192 的池第一次可写挂载这条流 GPU 判器建不出表：{refusal:?}")
    });
    assert_eq!(
        tables.state_count, FULL_STATES,
        "判表的状态数与全量相同（原地覆写三态）"
    );
}

/// 快档：段序列与全量的状态数钉住（七段、33 态），每段按甲二展开跑一遍恢复与 checker。
#[test]
fn the_first_writable_mount_of_an_io_min_8192_pool_has_seven_segments_and_33_crash_states() {
    let prepared = prepare();
    assert_eq!(
        closed_form_state_count(&prepared.segments),
        1 + 5 * 3 + 2,
        "每次写只取两态时：1 + 五个 2 写段各 3 + 两个 1 写段各 1"
    );
    assert_eq!(
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &full_expansion,
        ),
        FULL_STATES,
        "原地覆写取三态的全量：1 + 3 × (3² − 1) + 2 × 3 + 2 × 1"
    );
    let Some(tally) = enumerate_crash_states_quick_tier_versions(
        CrashPointReplayItem {
            name: "io_min_8192_mount::quick_tier",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &[],
    )
    .judged() else {
        return;
    };
    assert!(tally.states > 1, "快档展开出了崩溃状态");
    assert_recovery_checker_and_record_checker_clean(&tally);
}

/// 观察者「槽 1 在 8192 处」（`slot_one_at_8192`）：全池没有可用的槽 0 时，checker 两块盘的槽 1 都在 8192 处可用、与恢复择同一个世代号；
/// 记没有可用槽 0 的状态数。观察者的身份是这个 fn 的闭包键（入口登记在 `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`）。
fn observe_slot_one_at_8192(
    _ordinal: u64,
    image: &CrashImage<'_>,
    _report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let readings = system_configuration_slot_readings(image);
    let no_usable_slot_zero = readings
        .iter()
        .all(|(_, slots)| !matches!(slots[0], SystemConfigurationSlotReading::Usable(..)));
    if !no_usable_slot_zero {
        return;
    }
    counts.add("states_without_a_usable_slot_zero", 1);
    let slot_one_spacings: Vec<Option<u64>> = readings
        .iter()
        .map(|(_, slots)| match &slots[1] {
            SystemConfigurationSlotReading::Usable(_, geometry) => Some(geometry.slot_spacing),
            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. }
            | SystemConfigurationSlotReading::NotSelfVerified
            | SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => None,
        })
        .collect();
    let checker_generation_of_device_zero = chosen_system_configurations(image)
        .into_iter()
        .find(|(device, _)| *device == 0)
        .and_then(|(_, chosen)| chosen.map(|(view, _)| view.slot_generation));
    let recovery_generation = choose_system_configuration(image)
        .ok()
        .map(|chosen| chosen.quantities.slot_generation);
    assert!(
            slot_one_spacings == vec![Some(SLOT_SPACING_IN_BYTES); 2]
                && checker_generation_of_device_zero.is_some()
                && checker_generation_of_device_zero == recovery_generation,
            "全池没有可用的槽 0 时，checker 两块盘的槽 1 都在 8192 处可用、与恢复择同一个世代号：槽 1 读到的槽距 {slot_one_spacings:?}、checker 盘 0 择的世代号 {checker_generation_of_device_zero:?}、恢复择的 {recovery_generation:?}"
        );
}

/// 全量：33 个崩溃状态逐个跑恢复与 checker，零违例；两块盘的槽 0 都不可用的状态恰好两个（取号那一段、暖机那次轮换那一段各一个
/// 两盘同撕），每一个上 checker 两块盘的槽 1 都在 8192 处读成可用，checker 与恢复择到的世代号相同。
#[test]
#[ignore = "checker 档全量：崩溃枚举用例，由门禁 checker-tier-crash-point-replay --full 按 crash-case 登记跑"]
fn every_crash_state_of_the_first_writable_mount_of_an_io_min_8192_pool_recovers_and_the_checker_reads_slot_one_at_8192(
) {
    let prepared = prepare();
    // 观察者经崩溃判定库那一路按块看：要记的数全经观察者计数；槽 1 读得不对的状态当场 panic（判段把那一句带出来、用例红）
    let observe = observe_slot_one_at_8192;
    let Some(tally) = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: "io_min_8192_mount::full",
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&[]),
        &full_expansion,
        &[CrashPointReplayObserver {
            name: "slot_one_at_8192",
            // 观察者另读的输入：槽 1 该在的槽距（镜像与恢复报告之外它只比这一个数）
            extra_inputs_digest: extra_inputs_digest_that_was_computed(
                singlefs_harness::sha256::sha256_digest(
                    format!("slot_spacing_in_bytes={SLOT_SPACING_IN_BYTES}\n").as_bytes(),
                ),
            ),
            observe: &observe,
        }],
    )
    .judged() else {
        return;
    };
    let states_without_a_usable_slot_zero = tally
        .observer_counts
        .count_named("states_without_a_usable_slot_zero");
    println!(
        "CRASH_POINT_REPLAY_IO_MIN_8192_MOUNT states={} closed_form={FULL_STATES} exhaustive={} states_without_a_usable_slot_zero={states_without_a_usable_slot_zero} violations={} failed={} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states == FULL_STATES,
        tally.violations,
        tally.failed_states,
        tally.checker_counts_by_invariant()
    );
    assert_eq!(tally.states, FULL_STATES, "全量 33 态");
    assert_recovery_checker_and_record_checker_clean(&tally);
    assert_eq!(
        states_without_a_usable_slot_zero, 2,
        "两块盘的槽 0 都不可用的状态：取号那一段两盘同撕、暖机那次轮换两盘同撕，各一个"
    );
}

/// 观察者另读的输入的摘要要真算：全零就是没算（另读输入不同的两趟会共用看过的块）。
fn extra_inputs_digest_that_was_computed(
    digest: [u8; singlefs_harness::sha256::DIGEST_BYTES],
) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
    assert_ne!(
        digest,
        [0; singlefs_harness::sha256::DIGEST_BYTES],
        "观察者另读的输入的摘要真算（全零就是没算：另读输入不同的两趟会共用看过的块）"
    );
    digest
}
