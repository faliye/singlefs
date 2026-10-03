//! checker 档模块：crash、crash_identity、crash_amplification、verdict_store、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器与 CPU 判器逐状态对拍：六条流（新池新建文件；它改坏一次树表单元写两份副本之后的样子；它的 journal 记录提交标记改成 2、
//! 记录标志改成 0b11、系统配置槽距改成 512 三种重封过校验和的坏法；固定脚本到回退 D）上的每个
//! 崩溃状态，GPU 判出的红位（两遍 oracle 各七类、记录核对器两条）要与 CPU `judge_crash_image` 的 `red_items` 逐位相同，池级 checker
//! 每一条不变量的违例位（`IMPLEMENTED_INVARIANTS` 全部，不屏蔽）要与 CPU 池级 checker 逐位相同，一个状态都不许判不了。
//! 改坏那条流上 CPU 有红，GPU 要红在同样的状态、同样的项上——这是 GPU 判器分得出差别的自证；里程碑三二·1 那一批的每一条不变量
//! 另有一条只改坏它管的那个字段的流（`the_gpu_judge_agrees_with_the_cpu_when_…` 那一组）。
//! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
//! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
//! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
//! CPU 判器线程数 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（没设取本机可用并行度）：设了而认不出（线程数 0 也算）就 panic，不退回没设时的那一档。
#![cfg(all(feature = "verdict-store", feature = "gpu"))]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;

use std::time::Instant;

use common::{build_pool, file_content, geometry};
use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker_tier::crash::{
    judge_layer0_state_range, layer0_plan_state_count, Layer0OracleViolationKind,
    Layer0SegmentExpansion, Layer0StateJudgement,
};
use singlefs_checker_tier::crash_amplification::{
    gpu_recovery_words, judge_my_blocks_on_gpu_cards, plan_crash_points, record_crash_points,
    recovery_words_of, unjudged_blocks, CrashFlow, CrashPointRecorder, CrashPointSpan,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::crash_judge_dispatch::{
    states_per_scratch_lane, GpuCrashJudge, PackedJudgeTables,
};
use singlefs_checker_tier::crash_judge_gpu::{
    gpu_judge_digest, pool_checker_violated_bits, verdict_bits, JudgeKernelSources,
    POOL_CHECKER_REACHED_WORD, POOL_CHECKER_UNDECIDABLE_WORD, POOL_CHECKER_VIOLATED_BIT_CAPACITY,
    POOL_CHECKER_VIOLATED_FIRST_WORD, POOL_CHECKER_VIOLATED_WORDS, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    first_usable_card_or_report_not_run, usable_cards_or_report_not_run,
};
use singlefs_checker_tier::gpu_unit_checks::GpuCard;
use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::RollbackTarget;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use singlefs_harness::segments::StepKind;

const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO";
const DEFAULT_EXPAND_UP_TO: usize = 8;
const CARD_QUOTA_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES";
const CPU_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CPU_THREADS";
const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

/// 展开上限变量的值（`None` 是没设）：没设取 [`DEFAULT_EXPAND_UP_TO`]；设了而不是非负整数就 panic、消息带变量名与原值，不退回默认
/// （与 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO` 同一判法：缩没缩规模要看得出来，写错的值不许悄悄变成默认那一档）。
/// 这一族三个变量（展开上限、显存额度、CPU 线程数）都经 [`number_from_the_variable_value_panicking_when_unreadable`] 判。
///
/// # Panics
/// 设了而不是非负整数。
fn expand_up_to_from_the_value_of_the_variable(value: Option<&str>) -> usize {
    number_from_the_variable_value_panicking_when_unreadable(
        EXPAND_UP_TO_ENVIRONMENT_VARIABLE,
        value,
    )
    .unwrap_or(DEFAULT_EXPAND_UP_TO)
}

/// # Panics
/// 展开上限变量设了而认不出（[`expand_up_to_from_the_value_of_the_variable`]），或它的值不是 UTF-8。
fn expand_up_to() -> usize {
    expand_up_to_from_the_value_of_the_variable(
        process_environment_variable(EXPAND_UP_TO_ENVIRONMENT_VARIABLE).as_deref(),
    )
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=\"eight\" 认不出")]
fn an_unreadable_crash_judge_expand_up_to_value_panics_naming_the_variable_and_the_value() {
    let _limit = expand_up_to_from_the_value_of_the_variable(Some("eight"));
}

/// 显存额度变量的值（`None` 是没设）：没设是 `None`（不给这张卡定额度）；设了而不是非负整数就 panic、消息带变量名与原值，
/// 不悄悄当成没设（量速度时照的就不是配置的额度）。
///
/// # Panics
/// 设了而不是非负整数。
fn card_quota_mebibytes_from_the_value_of_the_variable(value: Option<&str>) -> Option<u64> {
    number_from_the_variable_value_panicking_when_unreadable(CARD_QUOTA_ENVIRONMENT_VARIABLE, value)
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES=\"4GiB\" 认不出")]
fn an_unreadable_crash_judge_card_quota_value_panics_naming_the_variable_and_the_value() {
    let _quota = card_quota_mebibytes_from_the_value_of_the_variable(Some("4GiB"));
}

/// CPU 判器线程数变量的值（`None` 是没设）：没设取本机可用并行度；设了而不是正整数（0 也算）就 panic、消息带变量名与原值，
/// 不悄悄换成至少 1 或本机并行度。
///
/// # Panics
/// 设了而不是正整数。
fn cpu_threads_from_the_value_of_the_variable(value: Option<&str>) -> std::num::NonZeroUsize {
    number_from_the_variable_value_panicking_when_unreadable(
        CPU_THREADS_ENVIRONMENT_VARIABLE,
        value,
    )
    .unwrap_or_else(|| std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN))
}

#[test]
#[should_panic(expected = "SINGLEFS_CRASH_JUDGE_CPU_THREADS=\"0\" 认不出")]
fn a_zero_crash_judge_cpu_threads_value_panics_naming_the_variable_and_the_value() {
    let _threads = cpu_threads_from_the_value_of_the_variable(Some("0"));
}

/// 取一张卡：空闲显存最大的那张。`SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` 给了就当这张卡的显存额度（量速度时照配置的额度量；
/// 写错就 panic，[`card_quota_mebibytes_from_the_value_of_the_variable`]）。
/// 本机没有 NVIDIA 独显适配器时打「本次未跑」那一行、交回 `None`，用例当场返回；有适配器而一张卡都没起来判红。
fn one_usable_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
    let quota = card_quota_mebibytes_from_the_value_of_the_variable(
        process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
    );
    let mut card = first_usable_card_or_report_not_run(test_name)?;
    if quota.is_some() {
        card.memory_quota_mebibytes = quota;
    }
    Some(card)
}

struct RecordedFlow {
    name: &'static str,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

fn record_new_pool_file_creation(tag: &str, name: &'static str) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
            content: file_content(),
        }],
    }
}

/// 改坏发布的根指着的树表单元（两份副本各一字节）：根落了的状态走读失败，CPU 与 GPU 都要红。
fn record_new_pool_with_a_broken_tree_table(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-broken-tree-table");
    let root_bytes = flow.writes[flow.judged_root_index]
        .bytes()
        .expect("根槽写带字节")
        .to_vec();
    // 树表指针在根记录偏移 36：两条位置条目在 50 与 64（盘 4、槽 6、校验和 4）
    let locations: Vec<(u32, u64)> = [86usize, 100]
        .iter()
        .map(|offset| {
            let device =
                u32::from_le_bytes(root_bytes[*offset..*offset + 4].try_into().expect("4"));
            let mut slot = [0u8; 8];
            slot[..6].copy_from_slice(&root_bytes[*offset + 4..*offset + 10]);
            (device, u64::from_le_bytes(slot))
        })
        .collect();
    let mut broken = 0;
    for write in &mut flow.writes {
        if write.kind != StepKind::UnitWrite {
            continue;
        }
        let slot = write.offset.0 / 16384;
        if locations
            .iter()
            .any(|(device, location_slot)| *device == write.device.0 && *location_slot == slot)
        {
            match &mut write.contents {
                WrittenContents::Bytes(bytes) => {
                    let last = bytes.len() - 1;
                    bytes[last] ^= 0x01;
                    broken += 1;
                }
                WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
            }
        }
    }
    assert_eq!(broken, 2, "树表单元的两份副本都改坏");
    flow
}

/// 改一次写里的几个字节之后把那个结构的整槽校验和字段重封（CRC-32C 住字段前 4 字节、后 28 字节 0）：拦住它的只剩被改的字段本身。
fn rewrite_and_reseal(
    flow: &mut RecordedFlow,
    kind: StepKind,
    nth_from_the_end: usize,
    cover_end: usize,
    field_offset: usize,
    rewrite: &dyn Fn(&mut Vec<u8>),
) {
    let index = flow
        .writes
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, write)| write.kind == kind)
        .nth(nth_from_the_end)
        .map(|(index, _)| index)
        .expect("流里有这一种写");
    match &mut flow.writes[index].contents {
        WrittenContents::Bytes(bytes) => {
            rewrite(bytes);
            let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
                bytes,
                cover_end,
                field_offset,
            );
            bytes[field_offset..field_offset + 32].copy_from_slice(&digest);
        }
        WrittenContents::Zeros { .. } => panic!("结构写是普通写"),
    }
}

/// 发布那条 journal 记录的提交标记字节改成 2（重封头校验和）：core 的读者当记录损坏，checker 的 I-8.8 判红。
fn record_new_pool_with_a_commit_byte_of_two(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-journal-commit-byte-2");
    rewrite_and_reseal(&mut flow, StepKind::JournalRecord, 0, 4096, 46, &|bytes| {
        bytes[86] = 2;
    });
    flow
}

/// 发布那条 journal 记录的记录标志改成 0b11（重封）：core 拒收这条记录，checker 的 I-8.9 判「标志其余位」红。
fn record_new_pool_with_record_flags_of_three(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-journal-flags-3");
    rewrite_and_reseal(&mut flow, StepKind::JournalRecord, 0, 4096, 46, &|bytes| {
        bytes[7] = 3;
    });
    flow
}

/// 最后一次系统配置槽写里固定结构槽距改成 512（重封）：那一槽自证过而带读者不收的值，core 整池拒（两遍恢复都没择到根），
/// checker 的 I-7.13 判红、别的不变量不判。
fn record_new_pool_with_a_refused_slot_spacing(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-sysconfig-spacing-512");
    rewrite_and_reseal(
        &mut flow,
        StepKind::SystemConfigurationSlot,
        0,
        4096,
        155,
        &|bytes| {
            bytes[429..433].copy_from_slice(&512u32.to_le_bytes());
        },
    );
    flow
}

/// 发布那次根槽写之前，同一个根槽上垫几次写：垫满 32 次，发布的根落没落就只差组合掩码的高 32 位。
const FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT: usize = 32;

/// 新池新建文件的流上，在暖机最后一次根槽写所在那一段之后另起一段：32 次根槽写，都写发布的根将要落的那个根槽、写的是它在基镜像里的
/// 字节（全零：那个槽在发布之前没有根）。这 32 次写不改任何一个崩溃态在盘上的字节（它们不带 journal 记录与单元，记录核对器也不看），
/// 只把发布那次根槽写挤成这个落点的第 33 次写：发布的根落了与没落的两个版本，组合掩码低 32 位相同（那 32 次都落了）、只差高位。
/// 判器查版本只比低 32 位时，把发布的根落了的状态读成全零的那一版，恢复落到暖机的根（C601（判器查版本不比组合掩码高 32 位时没有会红的流））。
fn record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-publish-root-slot-past-32-writes");
    let publish_root = flow.writes[flow.judged_root_index].clone();
    let slot_bytes_in_the_base = singlefs_core::recovery::PoolReader::read(
        &flow.base,
        publish_root.device,
        publish_root.offset,
        usize::try_from(publish_root.length_in_bytes()).expect("根槽写的长度装得进 usize"),
    )
    .expect("基镜像里读得到发布的根要落的槽");
    assert!(
        slot_bytes_in_the_base.iter().all(|byte| *byte == 0),
        "发布的根落在一个基镜像里空着的根槽上（垫的写写全零，不改任何崩溃态的盘上字节）"
    );
    assert!(
        flow.writes[..flow.judged_root_index]
            .iter()
            .all(|write| (write.device, write.offset) != (publish_root.device, publish_root.offset)),
        "发布之前流里没有别的写落在这个根槽上"
    );
    let warm_up_root = flow.writes[..flow.judged_root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布之前有暖机的根槽写");
    let segment_of_the_warm_up_root = flow
        .segments
        .iter()
        .position(|segment| segment.contains(&warm_up_root))
        .expect("暖机的根槽写在某一段里");
    let first_filler = *flow.segments[segment_of_the_warm_up_root]
        .last()
        .expect("段里至少一次写")
        + 1;
    let filler = RetainedWrite {
        device: publish_root.device,
        kind: StepKind::RootRecordFua,
        is_force_unit_access: publish_root.is_force_unit_access,
        offset: publish_root.offset,
        contents: WrittenContents::Bytes(slot_bytes_in_the_base),
    };
    flow.writes.splice(
        first_filler..first_filler,
        vec![filler; FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT],
    );
    for segment in &mut flow.segments {
        for write_index in segment.iter_mut() {
            if *write_index >= first_filler {
                *write_index += FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT;
            }
        }
    }
    flow.segments.insert(
        segment_of_the_warm_up_root + 1,
        (first_filler..first_filler + FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT).collect(),
    );
    flow.judged_root_index += FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT;
    flow
}

fn record_fixed_script_through_rollback(tag: &str) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
        content: file_content(),
    }];
    let second = overwrite_step(
        &mut recorder,
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(4),
        content: second_content(),
    });
    pool.output = second;
    let mounted = mount_writable_step(&mut recorder, &mut pool);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in 5..=7 {
        versions.push(PublishedVersion {
            instance: InstanceGeneration(2),
            checkpoint_txg: CheckpointTxg(txg),
            content: second_content(),
        });
    }
    let third = overwrite_step(
        &mut recorder,
        &mut pool,
        &third_content(),
        InstanceGeneration(2),
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(8),
        content: third_content(),
    });
    pool.output = third;
    roll_back_step(
        &mut recorder,
        &mut pool,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(2),
        checkpoint_txg: CheckpointTxg(9),
        content: file_content(),
    });
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedFlow {
        name: "fixed-script-through-rollback",
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

fn crc32c(bytes: &[u8]) -> u32 {
    singlefs_core::checksum::crc32_castagnoli(bytes)
}

/// 一个单元重封自己的两道校验和：载荷 CRC（按类的偏移，罩明文头之后到单元末）与头校验和（字段 10，罩明文头）。
fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[6] {
        1 => (105, 101),
        2 => {
            let key_width = usize::from(bytes[51]);
            (86 + 2 * key_width, 76 + 2 * key_width)
        }
        3 => (107, 89),
        other => panic!("单元类标签 {other} 不在 1 / 2 / 3 里"),
    };
    let payload_crc = crc32c(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, header_end, 10);
    bytes[10..42].copy_from_slice(&digest);
}

/// journal 记录重封：载荷校验和（95，罩点名项）再整条头校验和（46，罩 4096）。
fn reseal_journal_record(bytes: &mut [u8]) {
    let named = usize::try_from(u32::from_le_bytes(bytes[12..16].try_into().expect("4")))
        .expect("点名项数");
    let payload_crc = crc32c(
        &bytes[JOURNAL_RECORD_HEADER_BYTES
            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
    );
    bytes[95..99].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, 4096, 46);
    bytes[46..78].copy_from_slice(&digest);
}

/// 下一条记录该带的反向链：这一条 487 字节头、校验和字段按 0 参与的 CRC-32C。
fn journal_chain_value(bytes: &[u8]) -> u32 {
    let mut header = bytes[..JOURNAL_RECORD_HEADER_BYTES].to_vec();
    header[46..78].fill(0);
    crc32c(&header)
}

fn reseal_root_record(bytes: &mut [u8]) {
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, bytes.len(), 138);
    bytes[138..170].copy_from_slice(&digest);
}

/// 改坏一批单元写（同一个单元的两份副本，或只改其中一份）之后把校验和一路重封到根：被改的单元先重封自己的两道校验和，
/// 再把每一处指着它的位置条目（树节点、根记录、journal 点名项里的 盘 4 + 槽 6 + 整单元校验和 4）换成新的整单元 CRC，
/// 被改的那些结构再各自重封（单元继续往上传；journal 记录还要把同实例下一条的反向链改成它新的链值、一路传下去）。
/// 这样留下的坏法只剩被改的那个字段本身：CPU 的 checker 红在那个字段管的不变量上，GPU 要红在同样的状态、同样的项上。
fn corrupt_units_and_reseal_up_to_the_root(
    flow: &mut RecordedFlow,
    targets: &[usize],
    mutate: &dyn Fn(&mut [u8]),
) {
    let mut pending: Vec<(usize, u32)> = Vec::new();
    for target in targets {
        let write = &mut flow.writes[*target];
        assert_eq!(write.kind, StepKind::UnitWrite, "改的是单元写");
        let WrittenContents::Bytes(bytes) = &mut write.contents else {
            panic!("单元写是普通写");
        };
        let old_crc = crc32c(bytes);
        mutate(bytes);
        reseal_unit(bytes);
        pending.push((*target, old_crc));
    }
    while let Some((index, old_crc)) = pending.pop() {
        let (device, slot, new_crc) = {
            let write = &flow.writes[index];
            let bytes = write.bytes().expect("单元写带字节");
            (write.device.0, write.offset.0 / 16384, crc32c(bytes))
        };
        if old_crc == new_crc {
            continue;
        }
        let mut pattern = Vec::with_capacity(14);
        pattern.extend_from_slice(&device.to_le_bytes());
        pattern.extend_from_slice(&slot.to_le_bytes()[..6]);
        pattern.extend_from_slice(&old_crc.to_le_bytes());
        let mut journal_records_resealed: Vec<usize> = Vec::new();
        for referrer in 0..flow.writes.len() {
            let kind = flow.writes[referrer].kind;
            let WrittenContents::Bytes(bytes) = &mut flow.writes[referrer].contents else {
                continue;
            };
            let mut replaced = false;
            let mut position = 0;
            while position + 14 <= bytes.len() {
                if bytes[position..position + 14] == pattern[..] {
                    bytes[position + 10..position + 14].copy_from_slice(&new_crc.to_le_bytes());
                    replaced = true;
                    position += 14;
                } else {
                    position += 1;
                }
            }
            if !replaced {
                continue;
            }
            match kind {
                StepKind::UnitWrite => {
                    let referrer_old_crc = {
                        // 位置条目已换、校验和还没重封：整单元 CRC 要按重封之前的原样算，用写进去之前的字节复原
                        let mut before = bytes.clone();
                        let mut restore_position = 0;
                        while restore_position + 14 <= before.len() {
                            if before[restore_position..restore_position + 10] == pattern[..10]
                                && before[restore_position + 10..restore_position + 14]
                                    == new_crc.to_le_bytes()
                            {
                                before[restore_position + 10..restore_position + 14]
                                    .copy_from_slice(&old_crc.to_le_bytes());
                                restore_position += 14;
                            } else {
                                restore_position += 1;
                            }
                        }
                        crc32c(&before)
                    };
                    reseal_unit(bytes);
                    pending.push((referrer, referrer_old_crc));
                }
                StepKind::RootRecordFua => reseal_root_record(bytes),
                StepKind::JournalRecord => {
                    reseal_journal_record(bytes);
                    journal_records_resealed.push(referrer);
                }
                StepKind::ZeroFill | StepKind::SystemConfigurationSlot | StepKind::Barrier => {
                    panic!("位置条目不该出现在 {kind:?} 里")
                }
            }
        }
        carry_the_back_chain_forward_from(flow, journal_records_resealed);
    }
}

/// 反向链一路传下去：这几条 journal 记录写重封过，同盘、同实例、计数器大 1 的那一条带的是前一条头的链值——不等就改成它、重封，再往下传。
fn carry_the_back_chain_forward_from(
    flow: &mut RecordedFlow,
    mut journal_records_resealed: Vec<usize>,
) {
    while let Some(record_index) = journal_records_resealed.pop() {
        let (record_device, instance, counter, chain) = {
            let write = &flow.writes[record_index];
            let bytes = write.bytes().expect("记录写带字节");
            let mut counter = [0u8; 8];
            counter[..6].copy_from_slice(&bytes[20..26]);
            (
                write.device,
                u32::from_le_bytes(bytes[16..20].try_into().expect("4")),
                u64::from_le_bytes(counter),
                journal_chain_value(bytes),
            )
        };
        for next in 0..flow.writes.len() {
            if flow.writes[next].kind != StepKind::JournalRecord
                || flow.writes[next].device != record_device
            {
                continue;
            }
            let WrittenContents::Bytes(bytes) = &mut flow.writes[next].contents else {
                continue;
            };
            let mut next_counter = [0u8; 8];
            next_counter[..6].copy_from_slice(&bytes[20..26]);
            if u32::from_le_bytes(bytes[16..20].try_into().expect("4")) != instance
                || u64::from_le_bytes(next_counter) != counter + 1
                || bytes[91..95] == chain.to_le_bytes()
            {
                continue;
            }
            bytes[91..95].copy_from_slice(&chain.to_le_bytes());
            reseal_journal_record(bytes);
            journal_records_resealed.push(next);
        }
    }
}

/// 流里最后一个单元（两份副本：最后两条命中 `select` 的单元写，同槽同字节）的写下标；`only_first_copy` 只取盘序小的那一份。
fn last_unit_copies(
    flow: &RecordedFlow,
    what: &str,
    select: &dyn Fn(&[u8]) -> bool,
    only_first_copy: bool,
) -> Vec<usize> {
    let matching: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite && write.bytes().is_some_and(select))
        .map(|(index, _)| index)
        .collect();
    assert!(
        matching.len() >= 2,
        "流 {} 里命中「{what}」的单元写只有 {} 条，要至少两条（一个单元的两份副本）；流里的单元写按类 / key 宽 / 条目宽 / 记录类型：{:?}",
        flow.name,
        matching.len(),
        flow.writes
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .filter_map(|write| write.bytes())
            .map(|bytes| match unit_class(bytes) {
                2 => format!("2/k{}/w{}", index_node_key_width(bytes), index_node_entry_width(bytes)),
                3 => format!("3/t{}", packed_record_type(bytes)),
                other => format!("{other}"),
            })
            .collect::<Vec<_>>()
    );
    let copies = &matching[matching.len() - 2..];
    assert_eq!(
        flow.writes[copies[0]].offset, flow.writes[copies[1]].offset,
        "最后两条命中的单元写是同一个单元的两份副本（同槽）"
    );
    assert_eq!(
        flow.writes[copies[0]].bytes(),
        flow.writes[copies[1]].bytes(),
        "两份副本字节相同"
    );
    if only_first_copy {
        let first = if flow.writes[copies[0]].device.0 < flow.writes[copies[1]].device.0 {
            copies[0]
        } else {
            copies[1]
        };
        vec![first]
    } else {
        copies.to_vec()
    }
}

fn unit_class(bytes: &[u8]) -> u8 {
    bytes[6]
}
fn index_node_key_width(bytes: &[u8]) -> u8 {
    bytes[51]
}
fn index_node_entry_width(bytes: &[u8]) -> u16 {
    let header_end = 86 + 2 * usize::from(bytes[51]);
    u16::from_le_bytes(bytes[header_end - 2..header_end].try_into().expect("2"))
}
fn packed_record_type(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes[51..53].try_into().expect("2"))
}

/// 新池新建文件的流上改坏最后一个 `select` 命中的单元（两份或只一份）的一个字段、重封到根：`name` 是流名，`invariant` 是该红的那条。
fn record_new_pool_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    only_first_copy: bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    let targets = last_unit_copies(&flow, name, select, only_first_copy);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// 固定脚本到回退 D 的流上同样改坏最后一个命中的单元。
fn record_rollback_with_a_resealed_field(
    tag: &str,
    name: &'static str,
    select: &dyn Fn(&[u8]) -> bool,
    mutate: &dyn Fn(&mut [u8]),
) -> RecordedFlow {
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = name;
    let targets = last_unit_copies(&flow, name, select, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
    flow
}

/// 各条必红的世界：被改的字段与它管的不变量。
fn resealed_field_flows() -> Vec<(RecordedFlow, &'static str)> {
    let inode_leaf = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 2;
    let instance_table = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 4;
    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
    let allocation_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 10
            && index_node_entry_width(bytes) == 20
    };
    // 带至少一条已释放记录（跨度段位 15）的分配记录叶
    let allocation_leaf_with_a_released_record = |bytes: &[u8]| {
        if !allocation_leaf(bytes) {
            return false;
        }
        let header_end = 86 + 2 * usize::from(bytes[51]);
        let count = usize::from(u16::from_le_bytes(
            bytes[header_end - 4..header_end - 2].try_into().expect("2"),
        ));
        (0..count).any(|record| bytes[135 + record * 20 + 11] & 0x80 != 0)
    };
    let accounting_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 22
            && index_node_entry_width(bytes) == 34
    };
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    let mapping_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2 && index_node_key_width(bytes) == 27 && bytes[50] == 0
    };
    vec![
        (
            // inode 记录的 blocks 字段加 1：blocks ≠ ⌈size ÷ 512⌉
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i915",
                "new-pool-inode-blocks-plus-one",
                &inode_leaf,
                false,
                &|bytes| {
                    bytes[136 + 48] = bytes[136 + 48].wrapping_add(1);
                },
            ),
            "I-9.15",
        ),
        (
            // 数据单元五元组里的对象出生代加 1：与 inode 记录的对象出生代不符
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i910",
                "new-pool-data-unit-object-birth-plus-one",
                &data_unit,
                false,
                &|bytes| {
                    bytes[59] = bytes[59].wrapping_add(1);
                },
            ),
            "I-9.10",
        ),
        (
            // 分配记录叶第一条记录的分配代加 1：分配代 ≠ 它罩住的单元头里的诞生代号
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i310",
                "new-pool-allocation-generation-plus-one",
                &allocation_leaf,
                false,
                &|bytes| {
                    bytes[135 + 12] = bytes[135 + 12].wrapping_add(1);
                },
            ),
            "I-3.10",
        ),
        (
            // 分配记录叶第二条记录的槽号改成第一条的：两条记录罩住同一个槽
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i54",
                "new-pool-allocation-records-overlap",
                &allocation_leaf,
                false,
                &|bytes| {
                    let first = bytes[135..135 + 10].to_vec();
                    bytes[135 + 20..135 + 30].copy_from_slice(&first);
                },
            ),
            "I-5.4",
        ),
        (
            // 记账叶第一条的值加 1：已分配 / 空闲 / defer 的账对不上遍历
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i31",
                "new-pool-accounting-value-plus-one",
                &accounting_leaf,
                false,
                &|bytes| {
                    bytes[159 + 22] = bytes[159 + 22].wrapping_add(1);
                },
            ),
            "I-3.1",
        ),
        (
            // 树表前两条条目的树 ID 互换：不按树 ID 严格升序
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i916",
                "new-pool-tree-table-out-of-order",
                &tree_table,
                false,
                &|bytes| {
                    let first = bytes[131..131 + 8].to_vec();
                    let second = bytes[131 + 200..131 + 208].to_vec();
                    bytes[131..131 + 8].copy_from_slice(&second);
                    bytes[131 + 200..131 + 208].copy_from_slice(&first);
                },
            ),
            "I-9.16",
        ),
        (
            // 映射叶第一条 key 的出生 txg 加 1：映射 key 与单元头不符
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i111",
                "new-pool-mapping-key-birth-plus-one",
                &mapping_leaf,
                false,
                &|bytes| {
                    bytes[169 + 9] = bytes[169 + 9].wrapping_add(1);
                },
            ),
            "I-1.11",
        ),
        (
            // 回退流上最后一版实例表第一行的实例代号加 1：行不再低于挂载根的实例（新池流没有实例表写：取号那次写在 mkfs 那一段里）
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i38",
                "rollback-instance-row-not-below-mount",
                &instance_table,
                &|bytes| {
                    bytes[136 + 1] = bytes[136 + 1].wrapping_add(1);
                },
            ),
            "I-3.8",
        ),
        (
            // 数据单元只改一份副本的载荷首字节：归并成一组的两份载荷不同
            record_new_pool_with_a_resealed_field(
                "crash-judge-gpu-i18",
                "new-pool-data-unit-copies-differ",
                &data_unit,
                true,
                &|bytes| {
                    bytes[134] ^= 0xFF;
                },
            ),
            "I-1.8",
        ),
        (
            // 回退流上最后一版树表里 inode 树条目的诞生 txg 加 1：跨根不一
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i914",
                "rollback-tree-table-birth-txg-plus-one",
                &tree_table,
                &|bytes| {
                    for entry in 0..(bytes.len() - 131) / 200 {
                        let offset = 131 + entry * 200;
                        if u16::from_le_bytes(
                            bytes[offset + 10..offset + 12].try_into().expect("2"),
                        ) == 2
                        {
                            bytes[offset + 108] = bytes[offset + 108].wrapping_add(1);
                        }
                    }
                },
            ),
            "I-9.14",
        ),
        (
            // 回退流上最后一版分配记录叶里带已释放标志的第一条记录的释放代加 8：落到区间之外
            record_rollback_with_a_resealed_field(
                "crash-judge-gpu-i39",
                "rollback-release-generation-plus-eight",
                &allocation_leaf_with_a_released_record,
                &|bytes| {
                    let header_end = 86 + 2 * usize::from(bytes[51]);
                    let count = usize::from(u16::from_le_bytes(
                        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
                    ));
                    for record in 0..count {
                        let offset = 135 + record * 20;
                        if bytes[offset + 11] & 0x80 != 0 {
                            bytes[offset + 12] = bytes[offset + 12].wrapping_add(8);
                            break;
                        }
                    }
                },
            ),
            "I-3.9",
        ),
    ]
}

// ───────────── 里程碑三二·1 那一批：每条不变量一条只改坏它管的字段的流（GPU 与 CPU 逐状态逐位对拍，CPU 要在这条流上红那一条）─────────────

/// 系统配置槽：整槽校验和字段在 155、罩 4096 字节。
const SYSTEM_CONFIGURATION_CHECKSUM_FIELD: usize = 155;
/// journal 记录头里新根段的起点（树表指针 86、映射根指针 86、树 ID 水位 8、回退下界 F 8）。
const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;
/// 根记录里实例表指针的起点（86 字节的码 2 指针，出生树在指针偏移 34）。
const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
/// 码 2 / 码 3 指针头里出生树的偏移。
const POINTER_BIRTH_TREE_OFFSET: usize = 34;
/// journal 记录头 487 字节（D23（journal 的角色与格式） 已定项 4），点名项从这里起、每项 56 字节（诞生代号在点名项偏移 37）。
const JOURNAL_RECORD_HEADER_BYTES: usize = 487;
/// 点名项宽（D23（journal 的角色与格式） 已定项 17）。
const JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD: usize = 56;

/// 发布的根（`judged_root_index` 那次根槽写）的 (实例代号, checkpoint_txg)：根记录偏移 24、28。
fn publish_root_identity(flow: &RecordedFlow) -> (u32, u64) {
    let bytes = flow.writes[flow.judged_root_index]
        .bytes()
        .expect("根槽写带字节");
    (
        u32::from_le_bytes(bytes[24..28].try_into().expect("4")),
        u64::from_le_bytes(bytes[28..36].try_into().expect("8")),
    )
}

/// 改一次根槽写的字节，再重封根记录的整槽校验和（罩整个槽宽）。
fn rewrite_root_record_and_reseal(
    flow: &mut RecordedFlow,
    index: usize,
    rewrite: &dyn Fn(&mut [u8]),
) {
    assert_eq!(
        flow.writes[index].kind,
        StepKind::RootRecordFua,
        "改的是根槽写"
    );
    let WrittenContents::Bytes(bytes) = &mut flow.writes[index].contents else {
        panic!("根槽写是普通写");
    };
    rewrite(bytes);
    reseal_root_record(bytes);
}

/// 改一次 journal 记录写的字节，再重封载荷校验和与头校验和；同盘同实例计数器大 1 的那一条的反向链跟着改、一路传下去。
fn rewrite_journal_record_and_reseal(
    flow: &mut RecordedFlow,
    index: usize,
    rewrite: &dyn Fn(&mut [u8]),
) {
    assert_eq!(
        flow.writes[index].kind,
        StepKind::JournalRecord,
        "改的是 journal 记录写"
    );
    let WrittenContents::Bytes(bytes) = &mut flow.writes[index].contents else {
        panic!("记录写是普通写");
    };
    rewrite(bytes);
    reseal_journal_record(bytes);
    carry_the_back_chain_forward_from(flow, vec![index]);
}

/// 发布那次的每一份 journal 记录（每块盘一份：实例与 checkpoint_txg 都等于发布的根）各改一次、重封：重放与逐盘判定读到的都是改过的那一份。
fn rewrite_every_journal_record_of_the_publish(
    flow: &mut RecordedFlow,
    rewrite: &dyn Fn(&mut [u8]),
) {
    let (instance, checkpoint_txg) = publish_root_identity(flow);
    let records: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::JournalRecord
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[16..20].try_into().expect("4")) == instance
                        && u64::from_le_bytes(bytes[26..34].try_into().expect("8"))
                            == checkpoint_txg
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !records.is_empty(),
        "流 {} 里有发布那次（实例 {instance}、txg {checkpoint_txg}）的 journal 记录",
        flow.name
    );
    for index in records {
        rewrite_journal_record_and_reseal(flow, index, rewrite);
    }
}

/// 改流里最后一次系统配置槽写（它在那块盘上世代号最大，是那块盘择到的那一槽）的字节、重封整槽校验和。
fn rewrite_the_last_system_configuration_slot_and_reseal(
    flow: &mut RecordedFlow,
    rewrite: &dyn Fn(&mut Vec<u8>),
) {
    rewrite_and_reseal(
        flow,
        StepKind::SystemConfigurationSlot,
        0,
        4096,
        SYSTEM_CONFIGURATION_CHECKSUM_FIELD,
        rewrite,
    );
}

/// 在流尾另起一段，放这几次写（它们在最后全部持久的那个状态里都落了）。
fn append_a_segment_of_writes(flow: &mut RecordedFlow, writes: Vec<RetainedWrite>) {
    let first = flow.writes.len();
    let count = writes.len();
    flow.writes.extend(writes);
    flow.segments.push((first..first + count).collect());
}

/// I-1.12（指针出生树等于被指单元头）：发布的根记录里实例表指针的出生树加 1、重封根记录——实例表那一片单元头里的出生树是 0，指针不再与它相等。
fn record_new_pool_with_an_instance_table_pointer_naming_another_birth_tree(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-instance-table-pointer-birth-tree-plus-one");
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        let offset = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + POINTER_BIRTH_TREE_OFFSET;
        bytes[offset] = bytes[offset].wrapping_add(1);
    });
    flow
}

/// I-1.12（指针出生树等于被指单元头）数据单元那一支：发布那片 extent 树上段叶第一条内联的数据指针（条目偏移 25，标签 2）出生树加 1、
/// 重封到根——数据单元五元组里的树（头偏移 43）不再等于它。
fn record_new_pool_with_a_data_pointer_naming_another_birth_tree(tag: &str) -> RecordedFlow {
    let extent_upper_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 24
            && index_node_entry_width(bytes) == 113
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-data-pointer-birth-tree-plus-one",
        &extent_upper_leaf,
        false,
        &|bytes| {
            let first_entry = 115 + 2 * 24;
            assert_eq!(
                bytes[first_entry + 24],
                2,
                "上段叶第一条内联一个数据指针（标签 2）"
            );
            let birth_tree = first_entry + 25 + POINTER_BIRTH_TREE_OFFSET;
            bytes[birth_tree] = bytes[birth_tree].wrapping_add(1);
        },
    )
}

/// I-8.10（记录新根段与同代根相同）：发布那条记录新根段里的树 ID 水位加 1、重封——与同 (实例, txg) 的根里那 8 字节不再逐字节相同。
fn record_new_pool_with_a_last_record_carrying_another_watermark_than_its_root(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-last-record-watermark-plus-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let watermark = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86 + 86;
        bytes[watermark] = bytes[watermark].wrapping_add(1);
    });
    flow
}

/// I-8.11（点名项与被指单元头相同）：发布那条记录第一个点名项的诞生代号加 1、重封——它点名的单元整单元校验和照样对得上，头里的诞生代号不再等于它。
fn record_new_pool_with_a_named_entry_naming_another_birth_txg(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-named-entry-birth-txg-plus-one");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        assert!(
            u32::from_le_bytes(bytes[12..16].try_into().expect("4")) >= 1,
            "发布那条记录至少点名一个单元"
        );
        let birth_txg = JOURNAL_RECORD_HEADER_BYTES + 37;
        bytes[birth_txg] = bytes[birth_txg].wrapping_add(1);
    });
    flow
}

/// I-4.9（非最新候选根引用的单元不晚于它）：暖机最后那条根的树表指针与映射根指针换成发布的根的、重封——发布的根落了的状态上，
/// 这条更早的候选根引用的树表与映射是发布那一代诞生的（经「别的版本先走过」那一支判到）。
fn record_new_pool_with_an_older_root_pointing_at_the_trees_of_the_publish(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-warm-up-root-points-at-the-publish-trees");
    let publish_root = flow.writes[flow.judged_root_index]
        .bytes()
        .expect("根槽写带字节")
        .to_vec();
    let warm_up_root = flow.writes[..flow.judged_root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布之前有暖机的根槽写");
    let (_, publish_txg) = publish_root_identity(&flow);
    rewrite_root_record_and_reseal(&mut flow, warm_up_root, &|bytes| {
        let warm_up_txg = u64::from_le_bytes(bytes[28..36].try_into().expect("8"));
        assert!(warm_up_txg < publish_txg, "暖机那条根比发布的根早");
        bytes[36..122].copy_from_slice(&publish_root[36..122]);
        bytes[256..342].copy_from_slice(&publish_root[256..342]);
    });
    flow
}

/// I-9.25（树表条目的 previous_snapshot_txg 取值）：发布那张树表里第一条不是 inode 树的条目，previous_snapshot_txg（条目偏移 100）写成 1、重封到根。
fn record_new_pool_with_a_previous_snapshot_txg_on_a_tree_other_than_the_inode_tree(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-previous-snapshot-txg-on-a-non-inode-tree",
        &tree_table,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            let entry = (0..count)
                .map(|entry| 131 + entry * 200)
                .find(|offset| {
                    u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) != 2
                })
                .expect("树表里有不是 inode 树的条目");
            bytes[entry + 100] = 1;
        },
    )
}

/// I-9.27（树表条目的头 ID 取值）：发布那张树表里 extent 树那条（种类 1）的头 ID（条目偏移 116）写成 0、重封到根——
/// 它该等于同一张表里 inode 树条目的树 ID。
fn record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zero(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-extent-tree-entry-head-identifier-zero",
        &tree_table,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            let entry = (0..count)
                .map(|entry| 131 + entry * 200)
                .find(|offset| {
                    u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == 1
                })
                .expect("树表里有 extent 树那条");
            assert_ne!(
                bytes[entry + 116..entry + 124],
                [0u8; 8],
                "extent 树那条的头 ID 本来不是 0"
            );
            bytes[entry + 116..entry + 124].fill(0);
        },
    )
}

/// I-9.27 种类 2 那一支：发布那张树表里 inode 树那条（种类 2）的头 ID 从它自己的树 ID 改成树 ID + 1、重封到根。
fn record_new_pool_with_the_head_identifier_of_the_inode_tree_entry_off_its_own_tree_identifier(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-tree-entry-head-identifier-off-its-own-tree-identifier",
        &tree_table,
        false,
        &|bytes| {
            let entry = only_tree_table_entry_of_kind(bytes, 2);
            let own_tree_identifier =
                u64::from_le_bytes(bytes[entry..entry + 8].try_into().expect("8"));
            assert_eq!(
                u64::from_le_bytes(bytes[entry + 116..entry + 124].try_into().expect("8")),
                own_tree_identifier,
                "inode 树那条的头 ID 本来等于它自己的树 ID"
            );
            bytes[entry + 116..entry + 124]
                .copy_from_slice(&(own_tree_identifier + 1).to_le_bytes());
        },
    )
}

/// I-9.27 写 0 那一支：发布那张树表里分配记录树那条（种类 3）的头 ID 从 0 改成 inode 树条目的树 ID、重封到根。
fn record_new_pool_with_a_non_zero_head_identifier_on_the_allocation_record_tree_entry(
    tag: &str,
) -> RecordedFlow {
    let tree_table = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 8
            && index_node_entry_width(bytes) == 200
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-allocation-record-tree-entry-head-identifier-non-zero",
        &tree_table,
        false,
        &|bytes| {
            let entry = only_tree_table_entry_of_kind(bytes, 3);
            let inode_tree_entry = only_tree_table_entry_of_kind(bytes, 2);
            assert_eq!(
                bytes[entry + 116..entry + 124],
                [0u8; 8],
                "分配记录树那条的头 ID 本来是 0"
            );
            let inode_tree_identifier = bytes[inode_tree_entry..inode_tree_entry + 8].to_vec();
            bytes[entry + 116..entry + 124].copy_from_slice(&inode_tree_identifier);
        },
    )
}

/// 一张树表单元（码 2、条目宽 200）里种类是 `kind` 的那一条（恰好一条）在单元里的偏移。
fn only_tree_table_entry_of_kind(bytes: &[u8], kind: u16) -> usize {
    let header_end = 86 + 2 * usize::from(bytes[51]);
    let count = usize::from(u16::from_le_bytes(
        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
    ));
    let entries_of_the_kind: Vec<usize> = (0..count)
        .map(|entry| 131 + entry * 200)
        .filter(|offset| {
            u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == kind
        })
        .collect();
    let [entry] = entries_of_the_kind.as_slice() else {
        panic!("树表里种类 {kind} 的条目恰好一条：{entries_of_the_kind:?}")
    };
    *entry
}

/// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：新池新建文件那一版唯一的数据单元（两份副本，单元区起始槽号 + 4）挪到再往后两槽
/// （下一个 32 KiB 对齐的槽对，这条流上空着）——两次单元写的偏移、流里每一处指着它的位置条目（extent 上段叶内联的数据指针、
/// 中央映射叶、journal 记录）的槽号一起改，单元重封到根、记录重封。树表与诞生 txg 一个字节不动，这一版照样被认出，
/// 每块盘上落点最小的数据单元在 U + 6、不是 U + 4。
fn record_new_pool_with_the_first_data_unit_two_slots_further(tag: &str) -> RecordedFlow {
    const SLOTS_MOVED: u64 = 2;
    let mut flow = record_new_pool_file_creation(tag, "new-pool-first-data-unit-two-slots-further");
    let data_unit_copies: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write.bytes().is_some_and(|bytes| unit_class(bytes) == 1)
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        data_unit_copies.len(),
        2,
        "新池新建文件只写一个数据单元，两块盘各一份"
    );
    // (旧的位置条目 14 字节, 新的槽号 6 字节)
    let renamed_locations: Vec<(Vec<u8>, [u8; 6])> = data_unit_copies
        .iter()
        .map(|index| {
            let write = &flow.writes[*index];
            let slot = write.offset.0 / 16384;
            let mut location = Vec::with_capacity(14);
            location.extend_from_slice(&write.device.0.to_le_bytes());
            location.extend_from_slice(&slot.to_le_bytes()[..6]);
            location.extend_from_slice(&crc32c(write.bytes().expect("单元写带字节")).to_le_bytes());
            let mut new_slot = [0u8; 6];
            new_slot.copy_from_slice(&(slot + SLOTS_MOVED).to_le_bytes()[..6]);
            (location, new_slot)
        })
        .collect();
    for index in &data_unit_copies {
        flow.writes[*index].offset.0 += SLOTS_MOVED * 16384;
    }
    let old_locations: Vec<Vec<u8>> = renamed_locations
        .iter()
        .map(|(location, _)| location.clone())
        .collect();
    let names_the_data_unit = |bytes: &[u8]| {
        old_locations.iter().any(|location| {
            bytes
                .windows(14)
                .any(|window| window == location.as_slice())
        })
    };
    let rename = move |bytes: &mut [u8]| {
        for (location, new_slot) in &renamed_locations {
            let mut position = 0;
            while position + 14 <= bytes.len() {
                if bytes[position..position + 14] == location[..] {
                    bytes[position + 4..position + 10].copy_from_slice(new_slot);
                    position += 14;
                } else {
                    position += 1;
                }
            }
        }
    };
    let unit_referrers: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::UnitWrite
                && write
                    .bytes()
                    .is_some_and(|bytes| unit_class(bytes) != 1 && names_the_data_unit(bytes))
        })
        .map(|(index, _)| index)
        .collect();
    let journal_referrers: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::JournalRecord && write.bytes().is_some_and(names_the_data_unit)
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !unit_referrers.is_empty(),
        "流里有单元指着那个数据单元（extent 上段叶、中央映射叶）"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &unit_referrers, &rename);
    for index in journal_referrers {
        rewrite_journal_record_and_reseal(&mut flow, index, &rename);
    }
    flow
}

/// I-3.13（记账行代段等于这一版的 txg）：发布那片记账叶第二条的代段（key 偏移 14）加 1、重封到根。取中间那条：
/// 首末两条的 key 是叶头里的区间，改了它们 I-1.1 先红。
fn record_new_pool_with_an_accounting_row_of_another_generation(tag: &str) -> RecordedFlow {
    let accounting_leaf = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == 22
            && index_node_entry_width(bytes) == 34
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-accounting-row-generation-plus-one",
        &accounting_leaf,
        false,
        &|bytes| {
            let header_end = 86 + 2 * usize::from(bytes[51]);
            let count = usize::from(u16::from_le_bytes(
                bytes[header_end - 4..header_end - 2].try_into().expect("2"),
            ));
            assert!(count >= 3, "记账叶至少三条，改中间那条（有 {count} 条）");
            let generation = 159 + 34 + 14;
            bytes[generation] = bytes[generation].wrapping_add(1);
        },
    )
}

/// I-9.26（inode 改动计数落在出生代与容器诞生代号之间）：发布那片 inode 叶第一条记录的改动计数（偏移 88）最高字节写成 0x7F、重封到根——
/// 远大于容器头的诞生代号。
fn record_new_pool_with_an_inode_change_count_past_its_container_birth(tag: &str) -> RecordedFlow {
    let inode_leaf = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 2;
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-change-count-past-the-container-birth",
        &inode_leaf,
        false,
        &|bytes| {
            bytes[136 + 88 + 7] = 0x7F;
        },
    )
}

/// I-7.16（系统配置槽世代号与槽号的关系）：最后一次系统配置槽写的世代号（偏移 147）加 2、重封——奇偶照旧，与同盘另一槽相差 3。
fn record_new_pool_with_system_configuration_generations_three_apart(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-generation-plus-two");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[147] = bytes[147].wrapping_add(2);
    });
    flow
}

/// I-7.17（根槽位置等于 txg 算出的位置）：发布的根原样再写一份到同一区域的下一个槽（流尾另起一段）——那个槽按 txg 算不该住它。
fn record_new_pool_with_a_root_copied_into_the_next_slot(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-publish-root-copied-into-the-next-slot");
    let publish_root = flow.writes[flow.judged_root_index].clone();
    let next_slot = singlefs_core::address::DeviceOffsetInBytes(
        publish_root.offset.0 + u64::from(geometry().fixed_structure_slot_spacing),
    );
    let next_slot_in_the_base = singlefs_core::recovery::PoolReader::read(
        &flow.base,
        publish_root.device,
        next_slot,
        usize::try_from(publish_root.length_in_bytes()).expect("根槽写的长度装得进 usize"),
    )
    .expect("基镜像里读得到下一个根槽");
    assert!(
        next_slot_in_the_base.iter().all(|byte| *byte == 0),
        "下一个根槽在基镜像里空着（发布的根在区域里不是最后一槽）"
    );
    assert!(
        flow.writes
            .iter()
            .all(|write| (write.device, write.offset) != (publish_root.device, next_slot)),
        "流里没有别的写落在下一个根槽上"
    );
    append_a_segment_of_writes(
        &mut flow,
        vec![RetainedWrite {
            device: publish_root.device,
            kind: publish_root.kind,
            is_force_unit_access: publish_root.is_force_unit_access,
            offset: next_slot,
            contents: publish_root.contents.clone(),
        }],
    );
    flow
}

/// I-7.15（各盘所选系统配置槽设备数相同）：最后一次系统配置槽写的设备数（偏移 143）改成 3、重封——那块盘择到的这一槽与别的盘不同。
fn record_new_pool_with_a_system_configuration_of_another_device_count(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-device-count-three");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[143..147].copy_from_slice(&3u32.to_le_bytes());
    });
    flow
}

/// I-4.1（重放后树可遍历）：发布那条记录新根段里树表指针两条位置条目的整单元校验和各翻一遍、重封——点名单元照样验得过，
/// 发布的根没落而记录落了的状态上重放施加它，施加出来的那一版树表两份都对不上校验和、走不下去。
fn record_new_pool_with_a_replayed_version_whose_tree_table_cannot_be_read(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-replayed-tree-table-checksums-flipped");
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        for location_checksum in [
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 60,
            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 74,
        ] {
            for byte in &mut bytes[location_checksum..location_checksum + 4] {
                *byte ^= 0xFF;
            }
        }
    });
    flow
}

/// I-8.1（环几何够大）三个字段那一支：最后一次系统配置槽写的安全系数 F（偏移 357）改成 2、重封——整池不作保，只红 I-8.1。
fn record_new_pool_with_a_safety_factor_of_two(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-system-configuration-safety-factor-2");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[357..361].copy_from_slice(&2u32.to_le_bytes());
    });
    flow
}

/// 两支整池不作保都中：最后一次系统配置槽写的槽距改成 512（读者不收）、F 改成 2，重封——走 I-7.13 那一支，只红 I-7.13、I-8.1 报不适用。
fn record_new_pool_with_a_refused_slot_spacing_and_a_safety_factor_of_two(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-system-configuration-spacing-512-and-safety-factor-2",
    );
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[429..433].copy_from_slice(&512u32.to_le_bytes());
        bytes[357..361].copy_from_slice(&2u32.to_le_bytes());
    });
    flow
}

/// I-8.5（tail 槽有效集非空）：流尾另起一段，两块盘的两个系统配置槽各写一份全零——它们都落了的状态上一个自证过的槽都没有。
fn record_new_pool_with_every_system_configuration_slot_zeroed_at_the_end(
    tag: &str,
) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-every-system-configuration-slot-zeroed");
    let spacing = u64::from(geometry().fixed_structure_slot_spacing);
    let zeroed_slots: Vec<RetainedWrite> = [0u32, 1]
        .into_iter()
        .flat_map(|device| {
            [0, spacing].into_iter().map(move |offset| RetainedWrite {
                device: singlefs_core::address::DeviceIdentity(device),
                kind: StepKind::SystemConfigurationSlot,
                is_force_unit_access: false,
                offset: singlefs_core::address::DeviceOffsetInBytes(offset),
                contents: WrittenContents::Bytes(vec![0u8; 4096]),
            })
        })
        .collect();
    append_a_segment_of_writes(&mut flow, zeroed_slots);
    flow
}

/// I-7.5（根槽按判定宽度对齐）：最后一次系统配置槽写的 io_min（偏移 425）改成 12288、重封——判定宽度变成 12288，根环起点 1 MiB 不是它的整数倍。
fn record_new_pool_with_an_io_min_the_root_slots_are_not_aligned_to(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-system-configuration-io-min-12288");
    rewrite_the_last_system_configuration_slot_and_reseal(&mut flow, &|bytes| {
        bytes[425..429].copy_from_slice(&12288u32.to_le_bytes());
    });
    flow
}

/// I-3.12（实例表行 T 低于引用它的根的 txg）：回退流上最后一版实例表第一行的 T（行偏移 5，8 字节）最高字节加 1、重封到根——T 不小于引用它的根的 txg。
fn record_rollback_with_an_instance_table_row_not_below_its_root(tag: &str) -> RecordedFlow {
    let instance_table = |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == 4;
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-instance-row-published-txg-past-its-root",
        &instance_table,
        &|bytes| {
            assert_eq!(bytes[136], 0, "实例表第一条是行（kind 0）");
            bytes[136 + 12] = bytes[136 + 12].wrapping_add(1);
        },
    )
}

/// I-8.12（同实例记录的 txg 不减）：回退流上最后一次 journal 记录写的 checkpoint_txg（偏移 26）改成 1、重封——同盘同实例更早的记录 txg 更大。
fn record_rollback_with_a_journal_record_txg_going_back(tag: &str) -> RecordedFlow {
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-last-journal-record-txg-one";
    let last_record = flow
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::JournalRecord)
        .expect("流里有 journal 记录写");
    let (device, instance) = {
        let write = &flow.writes[last_record];
        let bytes = write.bytes().expect("记录写带字节");
        (
            write.device,
            u32::from_le_bytes(bytes[16..20].try_into().expect("4")),
        )
    };
    assert!(
        flow.writes[..last_record].iter().any(|write| {
            write.kind == StepKind::JournalRecord
                && write.device == device
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[16..20].try_into().expect("4")) == instance
                        && u64::from_le_bytes(bytes[26..34].try_into().expect("8")) > 1
                })
        }),
        "同盘同实例有更早的、txg 大于 1 的记录"
    );
    rewrite_journal_record_and_reseal(&mut flow, last_record, &|bytes| {
        bytes[26..34].copy_from_slice(&1u64.to_le_bytes());
    });
    flow
}

/// GPU 判器与 CPU 判器在这条流的每个状态上红位与违例位逐位相同、落到同一条根，CPU 池级 checker 在这条流上红 `invariant`。
/// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
    test_name: &str,
    flow: &RecordedFlow,
    invariant: &'static str,
) {
    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
        return;
    };
    let comparison = compare_on_every_state(&card, flow, expand_up_to());
    assert!(
        comparison.cpu_checker_red_invariants.contains(invariant),
        "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn oracle_index(kind: Layer0OracleViolationKind) -> u32 {
    u32::try_from(
        Layer0OracleViolationKind::EVERY_KIND
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("七类之一"),
    )
    .expect("装得进 u32")
}

/// CPU 判定 → GPU 那一套红位。
fn expected_bits(judgement: &Layer0StateJudgement) -> u32 {
    let mut bits = 0u32;
    if let Some(violation) = &judgement.consulted_oracle {
        bits |= 1 << (verdict_bits::CONSULTED_ORACLE_FIRST + oracle_index(violation.kind));
    }
    if let Some(violation) = &judgement.ignored_oracle {
        bits |= 1 << (verdict_bits::IGNORED_ORACLE_FIRST + oracle_index(violation.kind));
    }
    if judgement.records.root_without_record {
        bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
    }
    if judgement.records.claimed_state_missing_unit {
        bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
    }
    bits
}

/// CPU 池级 checker 的违例位：位 i ↔ `IMPLEMENTED_INVARIANTS[i]`。
fn expected_pool_checker_bits(judgement: &Layer0StateJudgement) -> u128 {
    let mut bits = 0u128;
    for (invariant, verdict) in &judgement.pool_checker {
        if let InvariantVerdict::Violated(_) = verdict {
            let index = IMPLEMENTED_INVARIANTS
                .iter()
                .position(|candidate| candidate == invariant)
                .expect("池级 checker 报的不变量在清单里");
            bits |= 1u128 << index;
        }
    }
    bits
}

fn invariant_names(bits: u128) -> Vec<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(index, _)| bits & (1u128 << index) != 0)
        .map(|(_, name)| *name)
        .collect()
}

struct Comparison {
    states: u64,
    cpu_red: u64,
    gpu_red: u64,
    cpu_checker_red: u64,
    gpu_checker_red: u64,
    /// CPU 池级 checker 在这条流的哪几条不变量上红过。
    cpu_checker_red_invariants: std::collections::BTreeSet<&'static str>,
    /// 看 journal 那一遍落到的根或两遍的结局类别与 CPU 不同的状态数（算进不一致）。
    recovery_mismatches: u64,
    mismatches: Vec<String>,
}

fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) -> Comparison {
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables_started = Instant::now();
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count);
    let words = tables.words();
    eprintln!(
        "CRASH_JUDGE_TABLES flow={} states={state_count} locations={} version_bytes_words={} built_in={:?}",
        flow.name,
        tables.locations.len(),
        words.version_bytes.len(),
        tables_started.elapsed()
    );
    let gpu_started = Instant::now();
    let judge = GpuCrashJudge::new(card, &tables);
    let gpu = judge.judge_states(0..state_count);
    let gpu_elapsed = gpu_started.elapsed();
    assert_eq!(gpu.len(), usize::try_from(state_count).expect("装得进"));
    // CPU 那一侧按线程分段判（`SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设取本机可用并行度），合并按序号排。
    // 段切得比线程数细（每个线程八段、轮着领）：靠后的状态历史长、判得慢，按线程数等分时最后一段拖住整趟
    let cpu_started = Instant::now();
    let threads = u64::try_from(
        cpu_threads_from_the_value_of_the_variable(
            process_environment_variable(CPU_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        )
        .get(),
    )
    .expect("线程数装得进 u64");
    let slice = state_count.div_ceil(threads * 8).max(1);
    let next_slice = std::sync::atomic::AtomicU64::new(0);
    let slice_count = state_count.div_ceil(slice);
    type JudgedOnCpu = (u32, u128, String, String, [u32; 4]);
    let mut slices: Vec<(u64, Vec<JudgedOnCpu>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads.min(slice_count))
            .map(|_| {
                let flow_reference = &*flow;
                let next_slice = &next_slice;
                scope.spawn(move || {
                    let expansion_in_thread = move |_segment_index: usize, segment: &[usize]| {
                        if segment.len() <= limit {
                            Layer0SegmentExpansion::EveryProperSubset
                        } else {
                            Layer0SegmentExpansion::NotExpanded
                        }
                    };
                    let mut judged_slices: Vec<(u64, Vec<JudgedOnCpu>)> = Vec::new();
                    loop {
                        let index = next_slice.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        if index >= slice_count {
                            break;
                        }
                        let range = index * slice..((index + 1) * slice).min(state_count);
                        let mut judged: Vec<JudgedOnCpu> = Vec::new();
                        let _state_count = judge_layer0_state_range(
                            &flow_reference.base,
                            &flow_reference.writes,
                            &flow_reference.segments,
                            flow_reference.judged_root_index,
                            &flow_reference.versions,
                            &expansion_in_thread,
                            range,
                            &mut |_ordinal, _image, judgement| {
                                judged.push((
                                    expected_bits(judgement),
                                    expected_pool_checker_bits(judgement),
                                    format!("{:?}", judgement.red_items()),
                                    format!(
                                        "consulted={:?} effective={:?} ignored={:?}",
                                        judgement.consulted.outcome,
                                        judgement.consulted.effective_root,
                                        judgement.ignored.outcome
                                    ),
                                    recovery_words_of(judgement),
                                ));
                            },
                        );
                        judged_slices.push((index, judged));
                    }
                    judged_slices
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("判定线程不 panic"))
            .collect()
    });
    slices.sort_by_key(|(index, _)| *index);
    let mut cpu: Vec<JudgedOnCpu> = slices.into_iter().flat_map(|(_, judged)| judged).collect();
    assert_eq!(cpu.len(), gpu.len());
    let cpu_elapsed = cpu_started.elapsed();
    cpu.truncate(gpu.len());
    let mut comparison = Comparison {
        states: state_count,
        cpu_red: 0,
        gpu_red: 0,
        cpu_checker_red: 0,
        gpu_checker_red: 0,
        cpu_checker_red_invariants: std::collections::BTreeSet::new(),
        recovery_mismatches: 0,
        mismatches: Vec::new(),
    };
    for (ordinal, (cpu_bits, cpu_checker_bits, cpu_items, cpu_report, cpu_recovery)) in
        cpu.iter().enumerate()
    {
        let gpu_words: [u32; VERDICT_WORDS] = gpu[ordinal];
        let gpu_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
        let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
            || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0;
        let gpu_checker_bits = pool_checker_violated_bits(&gpu_words);
        if *cpu_bits != 0 {
            comparison.cpu_red += 1;
        }
        if gpu_bits != 0 {
            comparison.gpu_red += 1;
        }
        if *cpu_checker_bits != 0 {
            comparison.cpu_checker_red += 1;
            comparison
                .cpu_checker_red_invariants
                .extend(invariant_names(*cpu_checker_bits));
        }
        if gpu_checker_bits != 0 {
            comparison.gpu_checker_red += 1;
        }
        // 落到的根与两遍的结局也要相同：只比红位看不出「两遍恢复落到同一条根」这类漏洞（重放没扫环那一次就是这样漏的）
        let gpu_recovery = gpu_recovery_words(&gpu_words);
        if gpu_recovery != *cpu_recovery {
            comparison.recovery_mismatches += 1;
        }
        if undecidable
            || gpu_bits != *cpu_bits
            || gpu_checker_bits != *cpu_checker_bits
            || gpu_recovery != *cpu_recovery
        {
            if comparison.mismatches.len() < 12 {
                comparison.mismatches.push(format!(
                    "状态 {ordinal}：CPU {cpu_bits:#x} {cpu_items} {cpu_report} checker={:?}；GPU {gpu_words:?} checker={:?}",
                    invariant_names(*cpu_checker_bits),
                    invariant_names(gpu_checker_bits)
                ));
            } else {
                comparison.mismatches.push(String::new());
            }
        }
    }
    for line in comparison
        .mismatches
        .iter()
        .filter(|line| !line.is_empty())
        .take(3)
    {
        let shown: String = line.chars().take(700).collect();
        eprintln!("CRASH_JUDGE_GPU_MISMATCH flow={} {shown}", flow.name);
    }
    eprintln!(
        "CRASH_JUDGE_GPU flow={} states={} cpu_red={} gpu_red={} cpu_checker_red={} gpu_checker_red={} recovery_mismatches={} mismatches={} gpu_elapsed={:?} cpu_elapsed={:?}",
        flow.name,
        comparison.states,
        comparison.cpu_red,
        comparison.gpu_red,
        comparison.cpu_checker_red,
        comparison.gpu_checker_red,
        comparison.recovery_mismatches,
        comparison.mismatches.len(),
        gpu_elapsed,
        cpu_elapsed
    );
    comparison
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows() {
    let Some(card) = one_usable_card_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows",
    ) else {
        return;
    };
    assert_ne!(gpu_judge_digest(), [0u8; 32]);
    let limit = expand_up_to();
    let flows = [
        record_new_pool_file_creation("crash-judge-gpu-healthy", "new-pool"),
        record_new_pool_with_a_broken_tree_table("crash-judge-gpu-broken"),
        record_new_pool_with_a_commit_byte_of_two("crash-judge-gpu-commit-2"),
        record_new_pool_with_record_flags_of_three("crash-judge-gpu-flags-3"),
        record_new_pool_with_a_refused_slot_spacing("crash-judge-gpu-spacing-512"),
        record_fixed_script_through_rollback("crash-judge-gpu-rollback"),
    ];
    let mut failures = Vec::new();
    let resealed = resealed_field_flows();
    let every_flow: Vec<(&RecordedFlow, Option<&'static str>)> = flows
        .iter()
        .map(|flow| (flow, None))
        .chain(
            resealed
                .iter()
                .map(|(flow, invariant)| (flow, Some(*invariant))),
        )
        .collect();
    for (flow, must_red_invariant) in every_flow {
        let comparison = compare_on_every_state(&card, flow, limit);
        if let Some(invariant) = must_red_invariant {
            assert!(
                comparison.cpu_checker_red_invariants.contains(invariant),
                "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
                flow.name,
                comparison.cpu_checker_red_invariants
            );
        }
        if flow.name == "new-pool-broken-tree-table" {
            assert!(comparison.cpu_red > 0, "改坏树表的流上 CPU 判器要有红");
        }
        if flow.name.starts_with("new-pool-journal") || flow.name.starts_with("new-pool-sysconfig")
        {
            assert!(
                comparison.cpu_checker_red > 0,
                "流 {} 上 CPU 池级 checker 要有红",
                flow.name
            );
        }
        if flow.name.starts_with("new-pool-sysconfig") {
            assert!(
                comparison.cpu_red > 0,
                "系统配置带拒收值的流上两遍恢复要报没择到根"
            );
        }
        if !comparison.mismatches.is_empty() {
            failures.push(format!(
                "流 {}：{} 个状态不一致（前几个）：\n{}",
                flow.name,
                comparison.mismatches.len(),
                comparison
                    .mismatches
                    .iter()
                    .filter(|line| !line.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// 一个根槽上发布的根是第 33 次写（`record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times`）：发布的根落了与没落的
/// 两个版本组合掩码低 32 位相同、只差高位，字节也不同（一份是发布的根、一份全零）；GPU 判器在这条流的每个状态上与 CPU 判器红位逐位相同、
/// 落到同一条根。判器查版本只比低 32 位时，发布的根落了的状态被读成全零那一版、恢复落到暖机的根，这条红
/// （C601（判器查版本不比组合掩码高 32 位时没有会红的流））。表上的前提在取卡之前判，没卡的机器上也判到。
#[test]
fn the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write(
) {
    const EXPAND_UP_TO: usize = 8;
    const {
        assert!(
            FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT > EXPAND_UP_TO,
            "垫的那一段不展开（展开就是 2^32 个状态）"
        );
    }
    let flow = record_new_pool_with_the_publish_root_slot_written_past_thirty_two_times(
        "crash-judge-gpu-past-32-writes",
    );
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("一个落点 64 次写以内，建得出表");
    let publish_root = &flow.writes[flow.judged_root_index];
    let root_slot = tables
        .locations
        .iter()
        .find(|location| {
            location.device == publish_root.device.0 && location.offset == publish_root.offset.0
        })
        .expect("发布的根槽是判器表里的一个落点");
    assert_eq!(
        root_slot.writes.len(),
        FILLER_WRITES_BEFORE_THE_PUBLISH_ROOT + 1,
        "这个根槽上是垫的 32 次写加发布的根那一次"
    );
    let told_apart_only_by_the_high_bits = root_slot.versions.iter().any(|first| {
        root_slot.versions.iter().any(|second| {
            first.combination != second.combination
                && first.combination & 0xFFFF_FFFF == second.combination & 0xFFFF_FFFF
                && first.bytes != second.bytes
        })
    });
    assert!(
        told_apart_only_by_the_high_bits,
        "这个根槽要有两个低 32 位相同、只差高位、字节不同的版本，不然只比低 32 位的内核也判得对：{:?}",
        root_slot
            .versions
            .iter()
            .map(|version| version.combination)
            .collect::<Vec<_>>()
    );
    let Some(card) = one_usable_card_or_report_not_run(
        "the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write",
    ) else {
        return;
    };
    let comparison = compare_on_every_state(&card, &flow, EXPAND_UP_TO);
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 重封到根之后，发布那条 journal 记录要自洽：头校验和、载荷校验和对得上，每个点名项的两份位置条目都与全部写落盘的镜像上那份单元的
/// 整单元 CRC 相同——这是 `corrupt_units_and_reseal_up_to_the_root` 的自证（它错了，必红流红的就不只是被改的那个字段）。
#[test]
fn resealed_publish_record_names_units_that_match_on_disk() {
    use singlefs_core::recovery::PoolReader;
    let flow = record_new_pool_with_a_resealed_field(
        "crash-judge-gpu-diagnose-reseal",
        "new-pool-data-unit-object-birth-plus-one",
        &|bytes| unit_class(bytes) == 1,
        false,
        &|bytes| {
            bytes[59] = bytes[59].wrapping_add(1);
        },
    );
    let mut pool = flow.base.clone();
    pool.apply_writes(&flow.writes);
    let record_index = flow
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::JournalRecord)
        .expect("有记录写");
    let record = flow.writes[record_index].bytes().expect("记录写带字节");
    let named = usize::try_from(u32::from_le_bytes(record[12..16].try_into().expect("4")))
        .expect("点名项数");
    assert!(named >= 1, "发布的记录点名了单元");
    assert!(
        singlefs_core::checksum::wide_checksum_field_holds(record, 4096, 46),
        "记录头校验和重封过"
    );
    let payload_crc = crc32c(
        &record[JOURNAL_RECORD_HEADER_BYTES
            ..JOURNAL_RECORD_HEADER_BYTES + named * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD],
    );
    assert_eq!(
        payload_crc,
        u32::from_le_bytes(record[95..99].try_into().expect("4")),
        "记录载荷校验和重封过"
    );
    for entry in 0..named {
        let o = JOURNAL_RECORD_HEADER_BYTES + entry * JOURNAL_NAMED_ENTRY_BYTES_IN_THE_RECORD;
        let class = record[o + 28];
        let length = if class == 2 { 16384 } else { 32768 };
        for copy in 0..2 {
            let location = o + copy * 14;
            let device = u32::from_le_bytes(record[location..location + 4].try_into().expect("4"));
            let mut slot = [0u8; 8];
            slot[..6].copy_from_slice(&record[location + 4..location + 10]);
            let slot = u64::from_le_bytes(slot);
            let stored =
                u32::from_le_bytes(record[location + 10..location + 14].try_into().expect("4"));
            let on_disk = PoolReader::read(
                &pool,
                singlefs_core::address::DeviceIdentity(device),
                singlefs_core::address::DeviceOffsetInBytes(slot * 16384),
                length,
            )
            .map(|bytes| crc32c(&bytes));
            assert_eq!(
                on_disk,
                Some(stored),
                "点名项 {entry} 第 {copy} 份（类 {class}、盘 {device}、槽 {slot}）的校验和与盘上那份单元不符"
            );
        }
    }
}

/// 一次派活跨几条草稿缓冲（每条各派一次活、排在同一趟里）与一次派活只用一条缓冲，判出来的结论逐状态相同。
/// 流取固定脚本到回退 D、展开 16（131 195 态；这条流的段不是 6 次写以内就是 16 次以上，展开 14 只有 125 态、跨不过一条缓冲）；
/// 这张卡的显存额度装不下两条草稿缓冲就是环境不够，报出来，不悄悄只比一条。
/// 一条缓冲那一档与 CPU 判器的逐状态对拍在 `the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows`。
#[test]
fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several() {
    const EXPAND_UP_TO: usize = 16;
    let Some(card) = one_usable_card_or_report_not_run(
        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
    ) else {
        return;
    };
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-lanes");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    let packed = PackedJudgeTables::of(&tables);
    let states_per_lane =
        states_per_scratch_lane(card.device().limits().max_storage_buffer_binding_size);
    let within_one_lane = GpuCrashJudge::on_card_with_packed_tables(&card, &packed, 256);
    let across_lanes =
        GpuCrashJudge::on_card_with_packed_tables(&card, &packed, 3 * states_per_lane);
    println!(
        "CRASH_JUDGE_LANES states={state_count} states_per_lane={states_per_lane} states_per_dispatch_within_one_lane={} states_per_dispatch_across_lanes={}",
        within_one_lane.states_per_dispatch(),
        across_lanes.states_per_dispatch()
    );
    assert!(
        within_one_lane.states_per_dispatch() <= states_per_lane,
        "窄的那一档一次派活只用一条草稿缓冲"
    );
    assert!(
        across_lanes.states_per_dispatch() > states_per_lane,
        "这张卡的显存额度装不下两条草稿缓冲（一次派活只带得了 {} 个状态，一条缓冲 {states_per_lane} 个）：腾出显存再跑",
        across_lanes.states_per_dispatch()
    );
    assert!(
        state_count > 2 * u64::try_from(states_per_lane).expect("装得进 u64"),
        "这条流只有 {state_count} 个状态，跨不过两条草稿缓冲"
    );
    let narrow = within_one_lane.judge_states(0..state_count);
    let wide = across_lanes.judge_states(0..state_count);
    let first_difference = narrow
        .iter()
        .zip(&wide)
        .position(|(narrow_words, wide_words)| narrow_words != wide_words);
    assert_eq!(
        first_difference, None,
        "跨几条草稿缓冲判出来的结论与一条缓冲的不同，第一个不同的状态序号在这里"
    );
    assert_eq!(narrow.len(), wide.len());
}

/// 在走读内核清零违例位之后种一个假的违例位（下标取 64 与池级 checker 条数里大的那个，落在高 64 位、不撞真的不变量）、
/// 在扫描内核读回违例位之后种第 128 位（下标 127）：
/// 两份种过的内核判出的每个状态，违例位等于没种的那一份判出的再并上种的位（第 65 位每个状态都有，第 128 位只在扫描内核判完的状态上有），
/// 其余的字逐字相同。违例位的高 64 位在走读内核写出、扫描内核读回又写出、Rust 读出这三处任一处被截掉，这条红。
/// 池级 checker 今天 68 条（下标 0..=67；内核判下标 65 之外的每一条，下标 65 的 I-3.5 内核还没判，`crash_judge_gpu` 的「内核还没判」那张表），种的是第 69 位（下标 68）。
#[test]
fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels() {
    const FIRST_BIT_OF_THE_HIGH_HALF: usize = 64;
    const PLANTED_BY_THE_SCAN: usize = 127;
    const SCAN_KERNEL_JUDGED_THE_STATE: u32 = 5;
    let planted_by_the_walk = IMPLEMENTED_INVARIANTS.len().max(FIRST_BIT_OF_THE_HIGH_HALF);
    assert!(
        planted_by_the_walk < PLANTED_BY_THE_SCAN
            && PLANTED_BY_THE_SCAN < POOL_CHECKER_VIOLATED_BIT_CAPACITY,
        "种的两位都在违例位容量之内、不撞真的不变量（池级 checker {} 条）",
        IMPLEMENTED_INVARIANTS.len()
    );
    let judge_sources = JudgeKernelSources::of_the_judge();
    let walk_anchor = "\n    clear_violated();\n";
    let scan_anchor = "\n    load_violated(out);\n";
    assert_eq!(
        judge_sources.checker_walk.matches(walk_anchor).count(),
        1,
        "走读内核里清零违例位的调用恰好一处"
    );
    assert_eq!(
        judge_sources.checker_scan.matches(scan_anchor).count(),
        1,
        "扫描内核里读回违例位的调用恰好一处"
    );
    let planted_sources = JudgeKernelSources {
        recovery: judge_sources.recovery.clone(),
        checker_walk: judge_sources.checker_walk.replacen(
            walk_anchor,
            &format!("{walk_anchor}    violate({planted_by_the_walk}u);\n"),
            1,
        ),
        checker_scan: judge_sources.checker_scan.replacen(
            scan_anchor,
            &format!("{scan_anchor}    violate({PLANTED_BY_THE_SCAN}u);\n"),
            1,
        ),
    };
    let Some(card) = one_usable_card_or_report_not_run(
        "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
    ) else {
        return;
    };
    let limit = expand_up_to();
    let flow = record_new_pool_file_creation("crash-judge-gpu-planted-bit", "new-pool");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    let packed = PackedJudgeTables::of(&tables);
    let unplanted = GpuCrashJudge::on_card_with_packed_tables(&card, &packed, usize::MAX)
        .judge_states(0..state_count);
    let planted = GpuCrashJudge::on_card_compiling_these_kernel_sources(
        &card,
        &packed,
        usize::MAX,
        &planted_sources,
    )
    .judge_states(0..state_count);
    assert_eq!(unplanted.len(), planted.len());
    let mut states_the_scan_judged = 0u64;
    let mut mismatches = Vec::new();
    for (ordinal, (unplanted_words, planted_words)) in unplanted.iter().zip(&planted).enumerate() {
        assert_eq!(
            unplanted_words[POOL_CHECKER_UNDECIDABLE_WORD], 0,
            "状态 {ordinal}：没种的那一份池级 checker 判得了"
        );
        let scan_judged =
            unplanted_words[POOL_CHECKER_REACHED_WORD] == SCAN_KERNEL_JUDGED_THE_STATE;
        if scan_judged {
            states_the_scan_judged += 1;
        }
        let planted_bits = (1u128 << planted_by_the_walk)
            | if scan_judged {
                1u128 << PLANTED_BY_THE_SCAN
            } else {
                0
            };
        let expected_bits = pool_checker_violated_bits(unplanted_words) | planted_bits;
        let violated_words = POOL_CHECKER_VIOLATED_FIRST_WORD
            ..POOL_CHECKER_VIOLATED_FIRST_WORD + POOL_CHECKER_VIOLATED_WORDS;
        let other_words_agree = unplanted_words
            .iter()
            .zip(planted_words)
            .enumerate()
            .filter(|(word_index, _)| !violated_words.contains(word_index))
            .all(|(_, (unplanted_word, planted_word))| unplanted_word == planted_word);
        if pool_checker_violated_bits(planted_words) != expected_bits || !other_words_agree {
            mismatches.push(format!(
                "状态 {ordinal}：没种 {unplanted_words:?}；种过 {planted_words:?}；违例位要 {expected_bits:#x}，读出 {:#x}",
                pool_checker_violated_bits(planted_words)
            ));
        }
    }
    println!(
        "CRASH_JUDGE_PLANTED_BIT states={state_count} scan_judged={states_the_scan_judged} mismatches={}",
        mismatches.len()
    );
    assert!(
        states_the_scan_judged > 0,
        "这条流上要有扫描内核判完的状态，第 128 位那一路才走得到"
    );
    assert!(
        mismatches.is_empty(),
        "{} 个状态上种的位没走到结论（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 卡领块的时候被叫停：不再领新的块，已经领的判完写进库（块序连着）；下一趟只判剩下的，对账为空。
/// 配置开了 GPU、判器却建不出这条流的表（这里把一次写换成同样长的清零，判器的表不接清零写）：报错，
/// 不退回 CPU 判器，库里一块判定都不写（用户 2026-09-29 定：配了 GPU 就一定要 GPU 判，做不到就明确报错）。建表在上卡之前，不用卡。
#[test]
fn a_flow_whose_judge_tables_cannot_be_built_is_refused_and_not_judged_on_the_cpu() {
    const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs";
    let recorded = record_fixed_script_through_rollback("crash-judge-gpu-refused");
    let mut writes = recorded.writes.clone();
    let moved = *recorded.segments[0].first().expect("段里至少一次写");
    let length = writes[moved].length_in_bytes();
    writes[moved].contents = WrittenContents::Zeros { length };
    writes[moved].kind = StepKind::ZeroFill;
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= 8 {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        expansion: &expansion,
        crash_points: vec![CrashPointSpan {
            name: "every_write".to_string(),
            code_file_in_repository: THIS_FILE.to_string(),
            writes: 0..writes.len(),
            ignored: false,
        }],
    };
    let plan = plan_crash_points(&flow);
    let version = JudgeVersion([9u8; 32]);
    let directory = std::env::temp_dir().join(format!(
        "singlefs-crash-judge-gpu-refused-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    let mut store = VerdictStore::create_empty(&directory).expect("建得了判定库");
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let before = unjudged_blocks(&store, &version).expect("对得了账").len();
    assert!(before > 0, "这个点至少一块要判");
    let refused = judge_my_blocks_on_gpu_cards(
        &mut store,
        &flow,
        &plan,
        &|_block| true,
        &version,
        &[],
        &|| false,
    );
    let error = refused.expect_err("建不出表要报错，不退回 CPU 判器");
    assert!(
        error.contains("建不出这条流的表"),
        "报错说清是表建不出：{error}"
    );
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账").len(),
        before,
        "一块判定都没写进库"
    );
    drop(store);
    let _ = std::fs::remove_dir_all(&directory);
}

/// 流取固定脚本到回退 D、展开 16，每段一个崩溃点（几十块）；只用一张卡，叫停之前恰好领五块。
#[test]
fn the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest() {
    const EXPAND_UP_TO: usize = 16;
    const BLOCKS_BEFORE_THE_STOP: usize = 5;
    const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs";
    let Some(card) = one_usable_card_or_report_not_run(
        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
    ) else {
        return;
    };
    let card = std::sync::Arc::new(card);
    let recorded = record_fixed_script_through_rollback("crash-judge-gpu-stop");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let flow = CrashFlow {
        base: &recorded.base,
        writes: &recorded.writes,
        segments: &recorded.segments,
        judged_root_index: recorded.judged_root_index,
        versions: &recorded.versions,
        expansion: &expansion,
        crash_points: recorded
            .segments
            .iter()
            .enumerate()
            .map(|(index, segment)| CrashPointSpan {
                name: format!("segment_{index}"),
                code_file_in_repository: THIS_FILE.to_string(),
                writes: *segment.first().expect("段里至少一次写")
                    ..*segment.last().expect("段里至少一次写") + 1,
                ignored: false,
            })
            .collect(),
    };
    let plan = plan_crash_points(&flow);
    let version = JudgeVersion([7u8; 32]);
    let directory = std::env::temp_dir().join(format!(
        "singlefs-crash-judge-gpu-stop-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    let mut store = VerdictStore::create_empty(&directory).expect("建得了判定库");
    record_crash_points(&mut store, &plan, &version, &mut CoverageReport::default())
        .expect("录得进");
    let total = unjudged_blocks(&store, &version).expect("对得了账").len();
    assert!(
        total > BLOCKS_BEFORE_THE_STOP + 1,
        "这条流只有 {total} 块，不够在中间叫停"
    );
    let asked = std::sync::atomic::AtomicUsize::new(0);
    let stop_after_five_blocks =
        || asked.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= BLOCKS_BEFORE_THE_STOP;
    let (stopped_tally, stopped) = judge_my_blocks_on_gpu_cards(
        &mut store,
        &flow,
        &plan,
        &|_block| true,
        &version,
        std::slice::from_ref(&card),
        &stop_after_five_blocks,
    )
    .expect("判得完");
    println!(
        "CRASH_JUDGE_STOP total_blocks={total} blocks_before_the_stop={} states_before_the_stop={}",
        stopped.blocks_judged, stopped.states_judged
    );
    assert_eq!(
        stopped.blocks_judged,
        u64::try_from(BLOCKS_BEFORE_THE_STOP).expect("装得进 u64"),
        "叫停之前恰好领了五块"
    );
    assert_eq!(stopped_tally.blocks_checked, stopped.blocks_judged);
    assert_eq!(
        unjudged_blocks(&store, &version).expect("对得了账").len(),
        total - BLOCKS_BEFORE_THE_STOP,
        "领过的五块都在库里"
    );
    let (resumed_tally, resumed) = judge_my_blocks_on_gpu_cards(
        &mut store,
        &flow,
        &plan,
        &|_block| true,
        &version,
        std::slice::from_ref(&card),
        &|| false,
    )
    .expect("判得完");
    assert_eq!(
        resumed.blocks_judged,
        u64::try_from(total - BLOCKS_BEFORE_THE_STOP).expect("装得进 u64"),
        "接着判的那一趟只判剩下的块"
    );
    assert_eq!(
        stopped.states_judged + resumed.states_judged,
        plan.points.last().expect("有点").ordinals.end,
        "两趟合起来每个状态恰好判一次"
    );
    assert_eq!(stopped_tally.red_states + resumed_tally.red_states, 0);
    assert!(unjudged_blocks(&store, &version)
        .expect("对得了账")
        .is_empty());
    drop(store);
    let _ = std::fs::remove_dir_all(&directory);
}

/// 这个进程此刻在卡上占的显存（MiB，`nvidia-smi` 按进程报的；几张卡加起来）。读不到报 0。
fn memory_used_on_the_card_by_this_process() -> u64 {
    let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,used_memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return 0;
    };
    let mine = std::process::id().to_string();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (process, used) = line.split_once(',')?;
            (process.trim() == mine)
                .then(|| used.trim().parse::<u64>().ok())
                .flatten()
        })
        .sum()
}

/// 规划显卡用：每张能用的卡按整张卡的额度（空闲显存）开草稿区，从一条草稿缓冲开到这张卡装得下的条数，每一档照那一档的宽度派满三次，
/// 打每次派活卡算了多久、每个状态几微秒、这个进程在卡上占多少显存。看草稿区开到多大时每个状态的用时开始猛涨（缓冲被驱动放出显存）。
#[test]
#[ignore = "量显存上限用：要清场之后空着的卡，每张卡开十几档草稿区"]
fn dispatch_time_at_each_scratch_size_with_the_whole_card_given() {
    let Some(usable_cards) = usable_cards_or_report_not_run(
        "dispatch_time_at_each_scratch_size_with_the_whole_card_given",
    ) else {
        return;
    };
    let every_card: Vec<(u32, u64)> = usable_cards
        .iter()
        .map(|card| (card.index, card.free_memory_mebibytes))
        .collect();
    // 量的是这个进程在卡上占多少显存：取卡时起的上下文先放掉，下面按额度重起的那一份才是量的对象
    drop(usable_cards);
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-scratch-sizes");
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let packed = PackedJudgeTables::of(&tables);
    let state_count = tables.state_count;
    for card in singlefs_checker_tier::gpu_unit_checks::gpu_cards_by_priority(&every_card) {
        let limit_of_the_card = card.device().limits().max_storage_buffer_binding_size;
        let lane =
            singlefs_checker_tier::crash_judge_dispatch::states_per_scratch_lane(limit_of_the_card);
        let capacity = singlefs_checker_tier::crash_judge_dispatch::states_in_flight_within(
            limit_of_the_card,
            card.memory_budget_mebibytes(),
            &packed,
        );
        println!(
            "CRASH_JUDGE_SCRATCH_CARD card={:?} index={} free_mebibytes={} capacity_states={capacity} states_per_lane={lane} states={state_count}",
            card.name, card.index, card.free_memory_mebibytes
        );
        let mut lanes = 1usize;
        loop {
            let width = (lanes * lane).min(capacity);
            let judge = GpuCrashJudge::on_card_with_packed_tables(&card, &packed, width);
            let used = memory_used_on_the_card_by_this_process();
            // 从流的中间取，避开开头那些很快判完的小状态
            let first = (state_count / 2).min(state_count - u64::try_from(width).expect("装得进"));
            for round in 0..3 {
                let before = judge.time_spent().seconds_waiting_for_the_card;
                let verdicts = judge.judge_states_in_one_dispatch(
                    first..first + u64::try_from(width).expect("装得进"),
                );
                let seconds = judge.time_spent().seconds_waiting_for_the_card - before;
                println!(
                    "CRASH_JUDGE_SCRATCH card_index={} lanes={lanes} states={width} round={round} seconds={seconds:.3} microseconds_per_state={:.2} scratch_mebibytes={} process_memory_on_the_card_mebibytes={used}",
                    card.index,
                    seconds * 1e6 / verdicts.len() as f64,
                    judge.scratch_mebibytes()
                );
                if seconds > 5.0 {
                    break;
                }
            }
            drop(judge);
            if width == capacity {
                break;
            }
            lanes += 1;
        }
    }
}

/// 规划显卡用：同一条流（固定脚本到回退 D）在一张卡上按几种「一次派活的状态数」各判一遍，打每秒判多少态、草稿区占多少显存；
/// 换派活的粒度不许改判定：每一档的结论逐状态与第一档相同。到这张卡的上限（绑定上限、空闲显存）那一档为止。
#[test]
#[ignore = "量速度用：要一张空闲显存够的独显，几种派活粒度各把整条流判一遍"]
fn gpu_judge_throughput_by_states_per_dispatch() {
    let Some(card) =
        one_usable_card_or_report_not_run("gpu_judge_throughput_by_states_per_dispatch")
    else {
        return;
    };
    let flow = record_fixed_script_through_rollback("crash-judge-gpu-throughput");
    let limit = expand_up_to();
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let judge_flow = JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    };
    let tables = build_judge_tables(&judge_flow).expect("建得出表");
    let state_count = tables.state_count;
    println!(
        "CRASH_JUDGE_THROUGHPUT_CARD card={:?} free_mebibytes={} max_storage_buffer_binding_size={} max_buffer_size={} states={state_count}",
        card.name,
        card.free_memory_mebibytes,
        card.device().limits().max_storage_buffer_binding_size,
        card.device().limits().max_buffer_size
    );
    let mut first_verdicts: Option<Vec<[u32; VERDICT_WORDS]>> = None;
    let packed = PackedJudgeTables::of(&tables);
    let mut reached_the_limit_of_the_card = false;
    for requested in [256usize, 2170, 4340, 8680, 13_020, 17_360, 26_040, 34_720] {
        if reached_the_limit_of_the_card {
            break;
        }
        let build_started = Instant::now();
        let judge = GpuCrashJudge::on_card_with_packed_tables(&card, &packed, requested);
        let build_seconds = build_started.elapsed().as_secs_f64();
        let started = Instant::now();
        let verdicts = judge.judge_states(0..state_count);
        let seconds = started.elapsed().as_secs_f64();
        let spent = judge.time_spent();
        println!(
            "CRASH_JUDGE_THROUGHPUT requested={requested} states_per_dispatch={} scratch_mebibytes={} judge_seconds={seconds:.2} states_per_second={:.0} build_seconds={build_seconds:.1} dispatches={} seconds_preparing={:.2} seconds_waiting_for_the_card={:.2} seconds_reading_back={:.2} card_memory_used_mebibytes={}",
            judge.states_per_dispatch(),
            judge.scratch_mebibytes(),
            state_count as f64 / seconds,
            spent.dispatches,
            spent.seconds_preparing,
            spent.seconds_waiting_for_the_card,
            spent.seconds_reading_back,
            memory_used_on_the_card_by_this_process()
        );
        match &first_verdicts {
            None => first_verdicts = Some(verdicts),
            Some(first) => assert!(
                first == &verdicts,
                "一次派活 {requested} 个状态判出来的结论与第一档不同"
            ),
        }
        if judge.states_per_dispatch() < requested {
            reached_the_limit_of_the_card = true;
        }
    }
}

/// I-1.12：发布的根记录里实例表指针的出生树与实例表那一片单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_tree() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_tree",
        &record_new_pool_with_an_instance_table_pointer_naming_another_birth_tree("crash-judge-gpu-i112"),
        "I-1.12",
    );
}

/// I-1.12 数据单元那一支：extent 树里内联的数据指针的出生树与数据单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_another_birth_tree_than_its_unit() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_data_pointer_names_another_birth_tree_than_its_unit",
        &record_new_pool_with_a_data_pointer_naming_another_birth_tree("crash-judge-gpu-i112-data"),
        "I-1.12",
    );
}

/// I-8.10：发布那条记录的新根段与同代的根不逐字节相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_last_record_of_a_publish_carries_another_watermark_than_its_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_last_record_of_a_publish_carries_another_watermark_than_its_root",
        &record_new_pool_with_a_last_record_carrying_another_watermark_than_its_root("crash-judge-gpu-i810"),
        "I-8.10",
    );
}

/// I-8.11：点名项的诞生代号与它点名的单元头不符。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_named_entry_names_another_birth_txg_than_its_unit() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_named_entry_names_another_birth_txg_than_its_unit",
        &record_new_pool_with_a_named_entry_naming_another_birth_txg("crash-judge-gpu-i811"),
        "I-8.11",
    );
}

/// I-4.9：更早的候选根引用的单元晚于它诞生。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_older_candidate_root_references_units_born_after_it() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_older_candidate_root_references_units_born_after_it",
        &record_new_pool_with_an_older_root_pointing_at_the_trees_of_the_publish("crash-judge-gpu-i49"),
        "I-4.9",
    );
}

/// I-9.25：不是 inode 树的树表条目带了 previous_snapshot_txg。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_entry_other_than_the_inode_tree_carries_a_previous_snapshot_txg(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tree_table_entry_other_than_the_inode_tree_carries_a_previous_snapshot_txg",
        &record_new_pool_with_a_previous_snapshot_txg_on_a_tree_other_than_the_inode_tree("crash-judge-gpu-i925"),
        "I-9.25",
    );
}

/// I-9.27：extent 树那条树表条目的头 ID 写成 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree",
        &record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zero("crash-judge-gpu-i927"),
        "I-9.27",
    );
}

/// I-9.27 种类 2 那一支：inode 树那条树表条目的头 ID 不等于它自己的树 ID。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_inode_tree_entry_carries_a_head_identifier_other_than_its_own_tree_identifier(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_inode_tree_entry_carries_a_head_identifier_other_than_its_own_tree_identifier",
        &record_new_pool_with_the_head_identifier_of_the_inode_tree_entry_off_its_own_tree_identifier("crash-judge-gpu-i927-inode"),
        "I-9.27",
    );
}

/// I-9.27 写 0 那一支：分配记录树那条树表条目的头 ID 不是 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_allocation_record_tree_entry_carries_a_non_zero_head_identifier(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_allocation_record_tree_entry_carries_a_non_zero_head_identifier",
        &record_new_pool_with_a_non_zero_head_identifier_on_the_allocation_record_tree_entry("crash-judge-gpu-i927-allocation"),
        "I-9.27",
    );
}

/// I-5.5：从 mkfs 水位建树的那一版，数据单元不落在单元区起始槽号 + 4。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four",
        &record_new_pool_with_the_first_data_unit_two_slots_further("crash-judge-gpu-i55"),
        "I-5.5",
    );
}

/// I-3.13：记账行的代段不是这一版的 txg。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version",
        &record_new_pool_with_an_accounting_row_of_another_generation("crash-judge-gpu-i313"),
        "I-3.13",
    );
}

/// I-9.26：inode 记录的改动计数越过所在容器的诞生代号。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_change_count_runs_past_its_container_birth",
        &record_new_pool_with_an_inode_change_count_past_its_container_birth("crash-judge-gpu-i926"),
        "I-9.26",
    );
}

/// I-3.12：实例表的一行的 T 不小于引用这张表的根的 txg（回退流）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_instance_table_row_is_not_below_the_root_referencing_it(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_an_instance_table_row_is_not_below_the_root_referencing_it",
        &record_rollback_with_an_instance_table_row_not_below_its_root("crash-judge-gpu-i312"),
        "I-3.12",
    );
}

/// I-7.16：同一块盘两个系统配置槽的世代号相差 3。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_two_system_configuration_slots_are_three_generations_apart(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_two_system_configuration_slots_are_three_generations_apart",
        &record_new_pool_with_system_configuration_generations_three_apart("crash-judge-gpu-i716"),
        "I-7.16",
    );
}

/// I-7.17：一条根住在按它的 txg 算不该住的槽里。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_root_lives_in_another_slot_than_its_txg_names() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_root_lives_in_another_slot_than_its_txg_names",
        &record_new_pool_with_a_root_copied_into_the_next_slot("crash-judge-gpu-i717"),
        "I-7.17",
    );
}

/// I-8.12：同一块盘同一实例的 journal 记录 txg 往回走（回退流）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_journal_record_txg_goes_back_within_an_instance() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_journal_record_txg_goes_back_within_an_instance",
        &record_rollback_with_a_journal_record_txg_going_back("crash-judge-gpu-i812"),
        "I-8.12",
    );
}

/// I-7.15：两块盘择到的系统配置设备数不同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_chosen_system_configurations_disagree_on_the_device_count(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_chosen_system_configurations_disagree_on_the_device_count",
        &record_new_pool_with_a_system_configuration_of_another_device_count("crash-judge-gpu-i715"),
        "I-7.15",
    );
}

/// I-4.1：重放施加出来的那一版走不下去。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_replayed_version_cannot_be_walked() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_replayed_version_cannot_be_walked",
        &record_new_pool_with_a_replayed_version_whose_tree_table_cannot_be_read(
            "crash-judge-gpu-i41",
        ),
        "I-4.1",
    );
}

/// I-8.1 三个字段那一支：系统配置槽里的 F 不是格式常量，整池不作保。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_records_a_safety_factor_other_than_the_format_constant(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_system_configuration_slot_records_a_safety_factor_other_than_the_format_constant",
        &record_new_pool_with_a_safety_factor_of_two("crash-judge-gpu-i81-fields"),
        "I-8.1",
    );
}

/// 两支整池不作保都中（槽距读者不收、F 不是格式常量）：走 I-7.13 那一支，CPU 只红 I-7.13、一个状态都不红 I-8.1，GPU 逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safety_factor_in_the_same_slot(
) {
    let flow = record_new_pool_with_a_refused_slot_spacing_and_a_safety_factor_of_two(
        "crash-judge-gpu-i713-over-i81",
    );
    let Some(card) = one_usable_card_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safety_factor_in_the_same_slot",
    ) else {
        return;
    };
    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-7.13")
            && !comparison.cpu_checker_red_invariants.contains("I-8.1"),
        "流 {} 上 CPU 池级 checker 红 I-7.13、不红 I-8.1（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        comparison
            .mismatches
            .iter()
            .filter(|line| !line.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// I-8.5：全池一个自证过的系统配置槽都没有。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_no_system_configuration_slot_verifies_itself() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_no_system_configuration_slot_verifies_itself",
        &record_new_pool_with_every_system_configuration_slot_zeroed_at_the_end(
            "crash-judge-gpu-i85",
        ),
        "I-8.5",
    );
}

/// I-7.5：根槽不是声明的 io_min 的整数倍。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_root_slots_are_not_aligned_to_the_declared_io_min() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_root_slots_are_not_aligned_to_the_declared_io_min",
        &record_new_pool_with_an_io_min_the_root_slots_are_not_aligned_to("crash-judge-gpu-i75"),
        "I-7.5",
    );
}

/// 两个池级 checker 内核把违例位的每个字都写出来：结论缓冲派活前整个填全 1，没被写的字读出来是全 1。
/// 新池新建文件那条流的每个状态（不种假位）上，违例位在池级 checker 条数之外的那些位全是 0、判得了——高 64 位那两个字只要有一个没写，这条红。
#[test]
fn the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count() {
    let Some(card) = one_usable_card_or_report_not_run(
        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
    ) else {
        return;
    };
    let limit = expand_up_to();
    let flow = record_new_pool_file_creation("crash-judge-gpu-violated-words-written", "new-pool");
    let expansion = move |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= limit {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count = tables.state_count;
    let verdicts = GpuCrashJudge::new(&card, &tables).judge_states(0..state_count);
    let invariant_count = IMPLEMENTED_INVARIANTS.len();
    let states_with_bits_past_the_invariant_count: Vec<String> = verdicts
        .iter()
        .enumerate()
        .filter(|(_, words)| {
            words[POOL_CHECKER_UNDECIDABLE_WORD] != 0
                || pool_checker_violated_bits(words) >> invariant_count != 0
        })
        .map(|(ordinal, words)| format!("状态 {ordinal}：{words:?}"))
        .collect();
    assert!(
        states_with_bits_past_the_invariant_count.is_empty(),
        "{} 个状态的违例位越过池级 checker 的 {invariant_count} 条或判不了（前几个）：\n{}",
        states_with_bits_past_the_invariant_count.len(),
        states_with_bits_past_the_invariant_count
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
