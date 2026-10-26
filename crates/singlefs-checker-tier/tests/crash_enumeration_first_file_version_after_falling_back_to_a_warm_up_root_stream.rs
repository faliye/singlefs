//! checker 档模块：crash、crash_state_enumeration
//! 崩溃恢复落到暖机根之后再发第一个文件版本的流（欠账 C511（回退到无文件那一版之后诞生代怎么接）：「一条用例走『崩溃恢复落到暖机根 → 再发第一个
//! 文件版本』，池级 checker 全绿，I-7.8、I-9.14、I-9.10 都判过且成立」；里程碑三六·2 补流）。
//!
//! 盘面：mkfs → 实例 1 取号、暖机两次（txg 1、2）、发第一个文件 A（txg 3，从水位 65536 发十三棵树）→ 进程退出；下一次可写挂载时 A 的根槽与它的
//! 数据单元一时读不出（暂存、清零），A 那次发布之后轮换写的系统配置槽写回成轮换之前的那一代（系统配置没见证到 A，C554 乙罩不到），择根落到暖机那条根
//! （txg 2，不带文件、水位停在 mkfs 那一档）；实例 2 写行、暖机，写行那次发布带的树 ID 水位取根环里的 max（D8（核心索引结构） 已定项 8 ②），
//! 之后把暂存的字节原样写回：A 的根又读得出，按实例 2 的实例表判是被抛弃的。造法照 harness 档那条用例
//! （`crates/singlefs-harness/tests/tree_identifier_watermark_after_the_first_file_root_is_abandoned.rs`，那一份的造法是私有的，这里照它另写一份）。
//! 基镜像是这之后的盘面，录的是实例 2 再发第一个文件版本（从写行带来的水位发树）与之后一次覆盖写（两条根的树表都罩着那十三棵树，I-9.14 跨根比得出来），
//! 按 D13（验证路线） 已定项 4 枚举崩溃状态：每个状态恢复两遍、多版本 oracle（实例 2 写行与暖机那几版没有文件）、池级 checker、记录核对器。
//!
//! **比已有的流多罩了什么**：`crash_points_tree_identifier_watermark_orphans.rs` 崩在第一个文件那次发布的每个前缀上、挂载之后再发第一个文件，
//! 那里落到暖机根的状态上 A 的根从没落盘（只剩孤儿）；`crash_enumeration_formatted_pool_remounted_without_a_file_stream.rs` 里根本没发过文件。
//! 这一条是 A 的根落了盘、读得出、被实例表抛弃，再从写行带来的水位发第一个文件版本：根环里同时有被抛弃的 A 与实例 2 的新树，
//! 新版本那一次发布的单元段、记录段、根槽写与轮换在这种盘面上第一次被枚举到。多跑的步没有（每个状态两遍恢复、池级 checker、记录核对器）。
//! 基线镜像、写表、段序列都与已有流不同，各自全量枚举。
//!
//! 快档按甲二展开；全量标 ignored，登记成崩溃枚举用例。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion, quick_tier_expansion,
    CrashPointReplaySegmentExpansion, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_point_replay_provenance::{
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow, RunOfTheFlow,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion, CrashPointReplayItem,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{mount_writable, Mounted};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, verified_system_configuration_slots,
};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, PublishedVersion, PublishedVersionWithoutAFile, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_first_file_version_after_falling_back_to_a_warm_up_root_stream.rs";

/// 崩溃点重放项名第一个 `::` 之前那一截（来历表里这条流的名字）。
const FLOW_NAME_AT_THE_ENUMERATION: &str =
    "first_file_version_after_falling_back_to_a_warm_up_root";

/// 实例 2 再发的第一个文件版本写进记录三个时间秒的值比 A 晚多少秒。
const SECONDS_OF_THE_FIRST_FILE_VERSION_AGAIN_AFTER_THE_FIRST_WRITE: u64 = 300;

/// 之后那次覆盖写写进记录三个时间秒的值比 A 晚多少秒。
const SECONDS_OF_THE_OVERWRITE_AFTER_THE_FIRST_WRITE: u64 = 360;

/// 之后那次覆盖写的内容。
const CONTENT_OF_THE_OVERWRITE: &[u8] =
    b"the first file as overwritten after the second instance published it again";

/// 不判违例之外，在这条流的状态上至少真被评估过一次的不变量：C511 点名的三条（根记录树 ID 水位不低于全池最大树 ID、树表条目的诞生 txg 跨根不变、
/// 对象出生代三处一致），连同 C467（新补的四条不变量的覆盖率没被穷举用例钉住） 那三条。
const INVARIANTS_EVALUATED_ON_SOME_STATE: [&str; 6] =
    ["I-7.8", "I-9.14", "I-9.10", "I-1.8", "I-7.3", "I-8.6"];

/// 盘上一段：哪块盘、从哪个字节起、多长。
struct PlaceOnADevice {
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length_in_bytes: usize,
}

/// 第一个文件 A 那条根被一次落到暖机根的崩溃恢复抛弃（文件头那一段的造法）：A 的根槽与它的数据单元（两份）暂存、清零，每块盘轮换写的那一槽
/// 系统配置换成另一槽的内容、世代号改成 g − 2（两槽都自证得过、见证值停在 A 之前），可写挂载（实例 2），再把暂存的字节原样写回。
/// 写都经录制设备进同一条流（落在基镜像之前）。交回那次挂载；池换成那次挂载之后的盘与分配器。
fn abandon_the_first_file_root_by_a_recovery_that_lands_on_the_warm_up_root(
    pool: &mut BuiltPool,
) -> Mounted {
    let first_file = pool.output.clone();
    let publish_parameters = parameters();
    let target = singlefs_core::root_ring::target_for_publish(
        first_file.root.checkpoint_txg,
        publish_parameters.geometry.root_ring_slots_per_region,
    );
    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
    let places_to_hide: Vec<PlaceOnADevice> = std::iter::once(PlaceOnADevice {
        device: publish_parameters.region_devices
            [usize::try_from(target.region).expect("区域号装得进 usize")],
        offset: singlefs_core::root_ring::slot_offset(
            target,
            publish_parameters.geometry.fixed_structure_slot_spacing,
        ),
        length_in_bytes: usize::try_from(publish_parameters.geometry.physical_block_size)
            .expect("根槽宽装得进 usize"),
    })
    .chain(
        first_file
            .data_pointers
            .iter()
            .flat_map(|pointer| pointer.locations)
            .map(|location| PlaceOnADevice {
                device: location.device,
                offset: location.slot.to_device_offset(),
                length_in_bytes: data_unit_bytes,
            }),
    )
    .collect();
    let mut devices = pool.reopen_recorded();
    let position_of = |devices_of_the_pool: &[(DeviceIdentity, common::Recorded)],
                       identity: DeviceIdentity| {
        devices_of_the_pool
            .iter()
            .position(|(candidate, _)| *candidate == identity)
            .expect("池里有这块盘")
    };
    let mut saved: Vec<(usize, DeviceOffsetInBytes, Vec<u8>)> = Vec::new();
    for place in places_to_hide {
        let position = position_of(&devices, place.device);
        let mut bytes = vec![0u8; place.length_in_bytes];
        devices[position]
            .1
            .read_at(place.offset, &mut bytes)
            .expect("暂存");
        devices[position]
            .1
            .write_at(
                place.offset,
                &vec![0u8; place.length_in_bytes],
                WriteDurability::Plain,
            )
            .expect("清零");
        saved.push((position, place.offset, bytes));
    }
    let slot_spacing_in_bytes = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
    let rotation_not_written_of_each_device: Vec<(DeviceOffsetInBytes, Vec<u8>)> = devices
        .iter()
        .map(|(identity, _)| {
            let mut verified_slots = verified_system_configuration_slots(
                devices.as_slice(),
                *identity,
                slot_spacing_in_bytes,
                &publish_parameters.filesystem_identifier,
            );
            verified_slots.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest_slot] =
                <[_; 2]>::try_from(verified_slots).expect("造被抛弃的根之前每块盘两槽都自证得过");
            let newest_generation = newest_slot.quantities.slot_generation;
            let mut rotation_not_written = older;
            rotation_not_written.quantities.slot_generation = newest_generation
                .checked_sub(2)
                .expect("A 之前至少还有两次轮换");
            (
                DeviceOffsetInBytes(
                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                ),
                rotation_not_written.to_slot(),
            )
        })
        .collect();
    for ((_, device), (newest_slot, rotation_not_written)) in
        devices.iter_mut().zip(rotation_not_written_of_each_device)
    {
        device
            .write_at(newest_slot, &rotation_not_written, WriteDurability::Plain)
            .expect("见证 A 的系统配置槽写回成轮换之前的那一代");
    }
    let mounted = mount_writable(&publish_parameters, &mut devices).unwrap_or_else(|error| {
        panic!("A 的根与数据单元读不出：择根落到暖机那条根，照常可写挂载：{error:?}")
    });
    for (position, offset, bytes) in &saved {
        devices[*position]
            .1
            .write_at(*offset, bytes, WriteDurability::Plain)
            .expect("原样写回");
    }
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator.clone();
    mounted
}

/// 录下来的那一段，连同搭建自证要的几样：被抛弃的 A、那次挂载落到的根与写行那次发布的水位。
struct RecordedFirstFileVersionAfterTheFallBack {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions_with_a_file: Vec<PublishedVersion>,
    versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
    abandoned_first_file: TransactionOutput,
    landed_root: (InstanceGeneration, CheckpointTxg, u64),
    row_publish_watermark: u64,
    landed_version_has_no_file: bool,
}

/// 搭盘面（A 被抛弃、实例 2 写行暖机），录实例 2 再发第一个文件版本与之后一次覆盖写。
fn record_the_first_file_version_after_falling_back_to_a_warm_up_root(
) -> RecordedFirstFileVersionAfterTheFallBack {
    let publish_parameters = parameters();
    let mut pool =
        build_pool("crash-enumeration-first-file-version-after-falling-back-to-a-warm-up-root");
    let abandoned_first_file = pool.output.clone();
    let mut mounted =
        abandon_the_first_file_root_by_a_recovery_that_lands_on_the_warm_up_root(&mut pool);
    let landed_version_has_no_file = mounted.current.file_version().is_none();
    let landed_root = (
        mounted.output.effective_root.instance,
        mounted.output.effective_root.checkpoint_txg,
        mounted.output.effective_root.tree_identifier_watermark,
    );
    let row_publish_watermark = mounted.output.row_publish.root().tree_identifier_watermark;
    let versions_without_a_file: Vec<PublishedVersionWithoutAFile> =
        std::iter::once(&mounted.output.row_publish)
            .chain(&mounted.output.warm_up_publishes)
            .map(|version| PublishedVersionWithoutAFile {
                instance: version.root().instance,
                checkpoint_txg: version.root().checkpoint_txg,
            })
            .collect();
    let base = pool.memory_pool();
    let operations_before_the_first_file_version_again = pool.stream.retained_operations().len();
    let instance = mounted.output.instance;
    let content_of_the_first_file_version_again = file_content();
    let devices = pool.devices.as_mut().expect("镜像开着");
    let first_file_version_again = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            mounted.current.root(),
            FirstFile {
                content: &content_of_the_first_file_version_again,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS
                    + SECONDS_OF_THE_FIRST_FILE_VERSION_AGAIN_AFTER_THE_FIRST_WRITE,
            },
            instance,
            mounted.current.record_bytes(),
        )
        .unwrap_or_else(|error| panic!("实例 2 再发第一个文件版本：{error:?}"))
    };
    let overwrite = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &first_file_version_again,
            FirstFile {
                content: CONTENT_OF_THE_OVERWRITE,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS
                    + SECONDS_OF_THE_OVERWRITE_AFTER_THE_FIRST_WRITE,
            },
            instance,
        )
        .unwrap_or_else(|error| panic!("实例 2 再发第一个文件版本之后覆盖写：{error:?}"))
    };
    let recorded = pool.stream.retained_operations()
        [operations_before_the_first_file_version_again..]
        .to_vec();
    let (writes, segments) = writes_and_segments(&recorded, &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("再发第一个文件版本与覆盖写写了根槽");
    let versions_with_a_file = vec![
        PublishedVersion {
            instance: abandoned_first_file.root.instance,
            checkpoint_txg: abandoned_first_file.root.checkpoint_txg,
            content: file_content(),
        },
        PublishedVersion {
            instance: first_file_version_again.root.instance,
            checkpoint_txg: first_file_version_again.root.checkpoint_txg,
            content: content_of_the_first_file_version_again,
        },
        PublishedVersion {
            instance: overwrite.root.instance,
            checkpoint_txg: overwrite.root.checkpoint_txg,
            content: CONTENT_OF_THE_OVERWRITE.to_vec(),
        },
    ];
    println!(
        "FIRST_FILE_VERSION_AFTER_FALLING_BACK_TO_A_WARM_UP_ROOT landed={landed_root:?} row_publish_watermark={row_publish_watermark} first_file_version_again=({}, {}) watermark={} segments={:?}",
        first_file_version_again.root.instance.0,
        first_file_version_again.root.checkpoint_txg.0,
        first_file_version_again.root.tree_identifier_watermark,
        segments.iter().map(Vec::len).collect::<Vec<usize>>()
    );
    RecordedFirstFileVersionAfterTheFallBack {
        base,
        writes,
        segments,
        judged_root_index,
        versions_with_a_file,
        versions_without_a_file,
        abandoned_first_file,
        landed_root,
        row_publish_watermark,
        landed_version_has_no_file,
    }
}

/// 盘面的前提（判器判完之后再核）：那次挂载落到的是暖机那一版（比 A 旧、不带文件、水位低于 A 的）；A 的根在基镜像的根环里读得出；
/// 写行那次发布带的水位等于基镜像根环里读得出的根上水位的最大值，也就是被抛弃的 A 的水位（C511 第 3 步），不是落到的那一版的。
fn assert_the_fall_back_to_the_warm_up_root(recorded: &RecordedFirstFileVersionAfterTheFallBack) {
    let abandoned_root = &recorded.abandoned_first_file.root;
    let (landed_instance, landed_txg, landed_watermark) = recorded.landed_root;
    assert!(
        recorded.landed_version_has_no_file
            && (landed_txg, landed_instance) < (abandoned_root.checkpoint_txg, abandoned_root.instance)
            && landed_watermark < abandoned_root.tree_identifier_watermark,
        "那次挂载落到比 A 旧、不带文件、水位低于 A 的暖机那一版：落到 {:?}，A 是（实例 {}、txg {}、水位 {}）",
        recorded.landed_root,
        abandoned_root.instance.0,
        abandoned_root.checkpoint_txg.0,
        abandoned_root.tree_identifier_watermark
    );
    let system_configuration =
        choose_system_configuration(&recorded.base).expect("基镜像的系统配置择得出");
    let roots_in_the_ring = readable_roots(
        &recorded.base,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    assert!(
        roots_in_the_ring.iter().any(|root| {
            (root.instance, root.checkpoint_txg)
                == (abandoned_root.instance, abandoned_root.checkpoint_txg)
        }),
        "被抛弃的 A 的根写回之后在基镜像的根环里读得出"
    );
    let highest_watermark_in_the_ring = roots_in_the_ring
        .iter()
        .map(|root| root.tree_identifier_watermark)
        .max()
        .expect("根环里有根");
    assert_eq!(
        (recorded.row_publish_watermark, highest_watermark_in_the_ring),
        (
            abandoned_root.tree_identifier_watermark,
            abandoned_root.tree_identifier_watermark
        ),
        "写行那次发布带的水位等于根环里读得出的根上水位的最大值（被抛弃的 A 的），不是落到的那一版的 {landed_watermark}"
    );
}

/// 按 `expansion` 枚举这条流；项名是 `<流名>::<标签>`。
fn enumerate_the_first_file_version_after_the_fall_back(
    recorded: &RecordedFirstFileVersionAfterTheFallBack,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let item_name = format!("{FLOW_NAME_AT_THE_ENUMERATION}::{item_label}");
    enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        CrashPointReplayVersionTable::new(
            &recorded.versions_with_a_file,
            &recorded.versions_without_a_file,
        ),
        expansion,
        &[],
    )
    .judged()
}

/// 判器判的几样一个都不红；名单里的不变量都真被评估过；评估过的与不适用的加起来恰是状态数。
fn assert_the_judges_are_clean_and_the_named_invariants_were_judged(tally: &CrashPointReplayTally) {
    assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0, "走读一次都不失败");
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 判违例：{:?}",
            tally.checker_first_violation.get(invariant)
        );
        assert_eq!(
            tally
                .checker_evaluated_states
                .get(invariant)
                .copied()
                .unwrap_or(0)
                + tally
                    .checker_not_applicable_states
                    .get(invariant)
                    .copied()
                    .unwrap_or(0),
            tally.states,
            "{invariant} 评估过的状态数 + 不适用的状态数 = 状态数"
        );
    }
    for invariant in INVARIANTS_EVALUATED_ON_SOME_STATE {
        assert!(
            tally
                .checker_evaluated_states
                .get(invariant)
                .copied()
                .unwrap_or(0)
                > 0,
            "{invariant} 至少在一个状态上真被评估过：{}",
            tally.checker_counts_by_invariant()
        );
    }
}

/// 平时跑的那一份：每段按甲二展开。判器先判，盘面的前提在它之后核：改坏实现时先红在判器上。
#[test]
fn every_crash_state_of_the_first_file_version_after_falling_back_to_a_warm_up_root_keeps_the_tree_identifiers_apart(
) {
    let recorded = record_the_first_file_version_after_falling_back_to_a_warm_up_root();
    let Some(tally) = enumerate_the_first_file_version_after_the_fall_back(
        &recorded,
        "fast",
        &quick_tier_expansion,
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_FIRST_FILE_VERSION_AFTER_FALLING_BACK_TO_A_WARM_UP_ROOT_FAST states={} states_by_publish=[{}] checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant()
    );
    assert_the_judges_are_clean_and_the_named_invariants_were_judged(&tally);
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &recorded.base,
            &recorded.writes,
            &recorded.segments,
            &quick_tier_expansion,
        ),
        "甲二展开按崩溃点重放的枚举域数（原地覆写取三态）"
    );
    assert_the_fall_back_to_the_warm_up_root(&recorded);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        FLOW_NAME_AT_THE_ENUMERATION,
        &tally,
    );
}

/// 全量：每段都展开，每个状态两遍恢复 + checker。登记成崩溃枚举用例，
/// checker-tier-crash-point-replay 认下面打印的 `CRASH_POINT_REPLAY_FIRST_FILE_VERSION_AFTER_FALLING_BACK_TO_A_WARM_UP_ROOT` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量约 3400 万个状态（按草稿副本录出的再发第一个文件版本那四段 [24,2,1,2] 与覆盖写同量级推的两态闭式，崩溃点重放的枚举域以用例打出的为准）、每个两遍恢复 + checker；门禁 checker-tier-crash-point-replay --full 在 release 下跑，可双机分片"]
fn full_enumeration_of_the_first_file_version_after_falling_back_to_a_warm_up_root_stream_is_exhaustive_and_clean(
) {
    let recorded = record_the_first_file_version_after_falling_back_to_a_warm_up_root();
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &full_expansion,
    );
    let Some(tally) =
        enumerate_the_first_file_version_after_the_fall_back(&recorded, "full", &full_expansion)
    else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_FIRST_FILE_VERSION_AFTER_FALLING_BACK_TO_A_WARM_UP_ROOT states={} closed_form={closed_form} exhaustive={} violations={} states_by_publish=[{}] checker {} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.states_by_publish_text(),
        tally.checker_counts_by_invariant(),
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_the_judges_are_clean_and_the_named_invariants_were_judged(&tally);
    assert_eq!(tally.states, closed_form);
    assert_the_fall_back_to_the_warm_up_root(&recorded);
    assert_every_state_was_seen_by_the_observers_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        FLOW_NAME_AT_THE_ENUMERATION,
        &tally,
    );
}
