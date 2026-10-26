//! checker 档模块：crash、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! 判退看三样读者的并集（C755（恢复与两种挂载触发退回的失败不同），`recovery::fall_back_judged_on_the_three_readers`）的两条快档核对（F6）：
//! - 核心层的恢复（判退照并集）与 GPU 恢复内核（`crash_judge_recovery.wgsl` 的 `selected_version_under_the_three_readers`，用户 2026-10-22 定 F6
//!   「代码该做一定要做」，GPU 自己判并集）在新池新建文件那条流的轻型状态上逐态落到同一版；
//! - 并集与只看走读（走读失败在字节丢了的失败点上才问 F2 判法：GPU 恢复内核改判并集之前那一判）在手造的那一形上分得开，CPU 参照
//!   （`singlefs_checker_tier::crash_point_replay_provenance` 的 `fall_back_of_the_state`、`disagreement_of_the_state`，只比判退那一步）认得出。
//!
//! 按来历统一挂在每个状态上的并集对走读观察者撤了（主 agent 2026-10-22 定）：GPU 判并集之后两边分家的态正是 GPU 与 CPU 同结局的态，它在那里判红不对，
//! 又是全量代价的大头；GPU 与 CPU 的并集逐状态对拍归这里的轻型那一条与 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`（走读字节丢了、
//! 建上一版或打开拒收内容那两形各一条）。
//!
//! 轻型那一条取新池新建文件那条流（崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv` 的 `new-pool-file-creation` 那一项）在轻型「甲二」
//! 展开下的全部状态：一条流按它在崩溃点重放里的展开建 GPU 判表（`crash_judge_tables::build_judge_tables`），判表先过对拍那一类的库、再读回交 GPU 判器；
//! 序号 k 的状态就是判表交给 GPU 恢复内核的那一个持久掩码（`JudgeTables::persisted_writes_of_state`），镜像接在同一份计划的枚举写表上，交核心层的恢复。
//! 另一条快档核对（审计复核 T01，用户 2026-10-24 定 checkpoint_txg「存完整 u64」）：覆盖写那次发布的 checkpoint_txg 过了 2^32 的那条流上，
//! 两遍恢复 GPU 恢复内核落到的根与核心层的恢复落到的根四个字（连同高 32 位）逐态相同。
//! 要卡：本机没有 NVIDIA 独显适配器时报「本次未跑」。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
#[path = "../../singlefs-harness/tests/common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;

#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use common::{build_pool, file_content, geometry};
use common_unwitnessed_publish::{record_one_publish_after_overwrites, PublishRecordedAsR};
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use singlefs_checker_tier::crash::{crash_state_enumeration_tables, quick_tier_expansion};
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use singlefs_checker_tier::crash_judge_gpu::{cross_check_words_of_gpu_verdict, LandedRootWords};
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::crash_point_replay_provenance::{
    disagreement_of_the_state, fall_back_of_the_state, first_failure_of_each_reader, key_of,
    FallBackOfTheState,
};
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_core::journal::JournalRecord;
use singlefs_core::mounted_read::OpenPoolForReadFailure;
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_core::recovery::{FailureKindForTheFallBack, RebuildVersionFailure, RecoveryFailure};
use singlefs_core::unit::{parse_data_unit, unit_filesystem_identifier};
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use singlefs_harness::memory_pool::{writes_and_segments, CrashImage, PublishedVersion};
use singlefs_harness::memory_pool::{MemoryPool, RetainedWrite, WrittenContents};
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
use singlefs_harness::segments::StepKind;

/// 一条录下的流：基线、写表、切段、被判的根槽写与它那一版（新池新建文件那条流，与覆盖写那一版 checkpoint_txg 过了 2^32 的那条流）。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
struct RecordedFlowOnItsBase {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

#[cfg(all(feature = "verdict-store", feature = "gpu"))]
fn new_pool_file_creation_flow(tag: &str) -> RecordedFlowOnItsBase {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有新池新建文件那次发布的根槽写");
    RecordedFlowOnItsBase {
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: pool.output.root.instance,
            checkpoint_txg: pool.output.root.checkpoint_txg,
            content: file_content(),
        }],
    }
}

/// 一遍恢复落到的根在结论里的那四个字（`crash_judge_conclusions::LandedRootWords`：落没落到根的标记、实例代号、checkpoint_txg 低 32 位与
/// 高 32 位；没落到根是 `LandedRootWords::NOT_LANDED`），与 GPU 结论里那几个字同口径。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
fn landed_root_words_of_the_core_recovery(
    effective_root: Option<(
        singlefs_core::address::InstanceGeneration,
        singlefs_core::address::CheckpointTxg,
    )>,
) -> LandedRootWords {
    effective_root.map_or(LandedRootWords::NOT_LANDED, |(instance, checkpoint_txg)| {
        LandedRootWords::of_the_landed_root(instance, checkpoint_txg)
    })
}

/// 场景：新池新建文件那条流在轻型「甲二」展开（`crash::quick_tier_expansion`）下 GPU 判表交给恢复内核的每一个状态，GPU 判器与核心层的恢复各判一遍
/// 看 journal 那一遍。
/// 预期：每个状态上 GPU 恢复内核落到的根与核心层的恢复（判退照三样读者的并集）落到的根相同——CPU 并集与 GPU 并集逐态同结局；恢复走到判退那一步的
/// 状态（`fall_back_of_the_state` 交 `Judged`）上也比，打出走到判退几态、并集退了几态。
/// 判别力：`crates/mutations.tsv` 里「F6 轻型 GPU 结论看 journal 那一遍落到的实例代号解错一位」那一行让这一条红（每个状态 GPU 那一边都对不上）；
/// 并集那一判本身的判别力在 GPU 对拍文件里走读字节丢了、建上一版或打开拒收内容那两形（变异行「C755 GPU 判退并集漏了…」）。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
#[test]
fn the_gpu_recovery_kernel_lands_where_the_core_recovery_under_the_union_of_the_three_readers_lands_on_the_quick_tier_states_of_the_new_pool_file_creation_flow(
) {
    let test_name = "the_gpu_recovery_kernel_lands_where_the_core_recovery_under_the_union_of_the_three_readers_lands_on_the_quick_tier_states_of_the_new_pool_file_creation_flow";
    let flow = new_pool_file_creation_flow("fall-back-union-on-the-gpu-and-the-cpu-quick-tier");
    let judge_tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &quick_tier_expansion,
    })
    .expect("新池新建文件那条流的 GPU 判表建得出（崩溃点重放轻型照建）");
    let enumeration = crash_state_enumeration_tables(
        &flow.base,
        &flow.writes,
        &flow.segments,
        &quick_tier_expansion,
    );
    assert_eq!(
        (
            judge_tables.state_count,
            usize::try_from(judge_tables.enumerated_write_count).expect("装得进 usize")
        ),
        (enumeration.state_count, enumeration.enumerated_writes.len()),
        "前提：GPU 判表与崩溃点重放同一份计划（状态数、枚举写表的写数）"
    );
    // 卡从卡池借（`gpu_card_pool_or_report_not_run`，没卡报「本次未跑」）；判表先入库（对拍那一类的库，项名是用例名）、再读库，
    // GPU 判器只拿读回的那一份算，都在 `judge_every_state_on_a_card_leased_from_the_pool` 里；只录那一趟、借不到卡当场返回
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(gpu_words_of_each_state) =
        judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &judge_tables)
    else {
        return;
    };
    assert_eq!(
        u64::try_from(gpu_words_of_each_state.len()).expect("装得进 u64"),
        judge_tables.state_count,
        "GPU 判器每个状态交回一份结论"
    );
    let mut mismatches: Vec<String> = Vec::new();
    let mut states_judging_the_fall_back = 0u64;
    let mut states_fallen_back_under_the_union = 0u64;
    for (ordinal, gpu_words) in (0..judge_tables.state_count).zip(&gpu_words_of_each_state) {
        let image = CrashImage {
            base: &flow.base,
            writes: &enumeration.enumerated_writes,
            persisted: judge_tables.persisted_writes_of_state(ordinal),
        };
        let on_the_cpu = landed_root_words_of_the_core_recovery(
            recover(&image, JournalPolicy::Consult).effective_root,
        );
        let on_the_gpu = cross_check_words_of_gpu_verdict(gpu_words).consulted_pass_landed_root;
        if let FallBackOfTheState::Judged(judged) = fall_back_of_the_state(&image) {
            states_judging_the_fall_back += 1;
            if judged.under_the_union_of_the_three_readers.is_some() {
                states_fallen_back_under_the_union += 1;
            }
        }
        if on_the_cpu != on_the_gpu {
            mismatches.push(format!(
                "新池新建文件甲二第 {ordinal} 个状态：核心层的恢复落到 {on_the_cpu:?}，GPU 恢复内核落到 {on_the_gpu:?}；{}",
                disagreement_of_the_state("同一个状态", &image)
                    .unwrap_or_else(|| "并集与只看走读这一态同结局".to_string())
            ));
        }
    }
    println!(
        "FALL_BACK_UNION_ON_THE_GPU_AND_THE_CPU flow=new-pool-file-creation expansion=quick-tier states={} states_judging_the_fall_back={states_judging_the_fall_back} states_fallen_back_under_the_union={states_fallen_back_under_the_union} mismatches={}",
        judge_tables.state_count,
        mismatches.len()
    );
    assert!(
        mismatches.is_empty(),
        "{} 个状态上 GPU 恢复内核与核心层的恢复落到的根不同（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 覆盖写那一版之前，上一版的 checkpoint_txg 在进程里抬高多少（[`overwrite_flow_published_on_a_checkpoint_txg_past_thirty_two_bits`]）：
/// 2^32，覆盖写那次发布的 txg 是新池新建文件那一版（txg 3）加 2^32 再加一，低 32 位 4、高 32 位 1。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
const CHECKPOINT_TXG_RAISED_BY: u64 = 1 << 32;

/// 覆盖写那一版的内容。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
const CONTENT_OF_THE_OVERWRITE_PAST_THIRTY_TWO_BITS: &[u8] =
    b"overwrite published on a checkpoint txg past thirty two bits";

/// 新池新建文件之后同一个进程里再覆盖写一次，覆盖写之前把上一版的 checkpoint_txg 在进程里抬高 [`CHECKPOINT_TXG_RAISED_BY`]
/// （内存里的上一版与分配器的根环表一起抬：根环表照 mkfs 的第 0 代根与 txg 1..=3 那三条根重装、这个进程下一条根接在抬高之后的 txg 后面，
/// 不然分配器认出根的 txg 跳号就断言）：覆盖写那次发布的根与记录都带 txg 2^32 + 4，盘上之前那几条根还是 0..=3。基线是新池新建文件之后的整份盘面，
/// 写表是覆盖写那一段的写。审计复核 T01（`records/2026-10-24-singlefs-checker-tier-审计复核.md`）要的取样点：落到的根 checkpoint_txg 高 32 位非 0，
/// 根槽没落而记录落了的状态上看 journal 那一遍照记录落到 2^32 + 4、不看那一遍落到 txg 3（实现员在 harness 档的草稿探针里对核心层恢复量过这两样，
/// 探针没进仓）。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
fn overwrite_flow_published_on_a_checkpoint_txg_past_thirty_two_bits(
    tag: &str,
) -> RecordedFlowOnItsBase {
    use common::{parameters, publish_overwrite_in_process, FIXED_WRITE_TIME_SECONDS};
    use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
    use singlefs_core::allocator::{RootRingOccupancy, RootRingOccupant};
    use singlefs_core::make_filesystem::genesis_root_ring_slots;
    use singlefs_core::root_ring::target_for_publish;
    let mut pool = build_pool(tag);
    let base = pool.memory_pool();
    let operations_before_the_overwrite = pool.retained_operations().len();
    let mut previous = pool.output.clone();
    let checkpoint_txg_of_the_first_file = previous.root.checkpoint_txg;
    previous.root.checkpoint_txg =
        CheckpointTxg(checkpoint_txg_of_the_first_file.0 + CHECKPOINT_TXG_RAISED_BY);
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    let mut occupants: Vec<_> = genesis_root_ring_slots()
        .map(|ring_slot| {
            (
                ring_slot,
                RootRingOccupant::ValidRoot {
                    checkpoint_txg: pool.genesis.root.checkpoint_txg,
                },
            )
        })
        .collect();
    occupants.extend(
        (pool.genesis.root.checkpoint_txg.0 + 1..=checkpoint_txg_of_the_first_file.0).map(
            |checkpoint_txg| {
                (
                    target_for_publish(CheckpointTxg(checkpoint_txg), slots_per_region),
                    RootRingOccupant::ValidRoot {
                        checkpoint_txg: CheckpointTxg(checkpoint_txg),
                    },
                )
            },
        ),
    );
    pool.allocator
        .install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
            slots_per_region,
            occupants,
            previous.root.rollback_floor,
            previous.root.checkpoint_txg,
        ));
    let output = publish_overwrite_in_process(
        &mut pool,
        &previous,
        CONTENT_OF_THE_OVERWRITE_PAST_THIRTY_TWO_BITS,
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("上一版的 checkpoint_txg 抬过 2^32 之后覆盖写照常发布");
    assert_eq!(
        output.root.checkpoint_txg,
        CheckpointTxg(checkpoint_txg_of_the_first_file.0 + CHECKPOINT_TXG_RAISED_BY + 1),
        "前提：覆盖写那次发布的 txg 接在抬高之后的那一版后面"
    );
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[operations_before_the_overwrite..], &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有覆盖写那次发布的根槽写");
    RecordedFlowOnItsBase {
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: output.root.instance,
            checkpoint_txg: output.root.checkpoint_txg,
            content: CONTENT_OF_THE_OVERWRITE_PAST_THIRTY_TWO_BITS.to_vec(),
        }],
    }
}

/// 场景：覆盖写那次发布的 checkpoint_txg 过了 2^32（[`overwrite_flow_published_on_a_checkpoint_txg_past_thirty_two_bits`]），这条流在轻型「甲二」
/// 展开下 GPU 判表交给恢复内核的每一个状态，GPU 判器与核心层的恢复各判看 journal 与不看 journal 两遍。
/// 预期：每个状态每一遍 GPU 恢复内核落到的根与核心层的恢复落到的根四个字逐个相同（`LandedRootWords`：标记、实例代号、checkpoint_txg 低 32 位
/// 与高 32 位，GPU 那一侧高 32 位是恢复内核写的 `effective_txg.y`）；前提：有状态落到 checkpoint_txg 高 32 位非 0 的根（审计复核 T01 要的
/// 取样点）。
/// 判别力：`crates/mutations.tsv` 里以「T01 GPU 恢复内核写落到的根高 32 位」开头的那一行（GPU 那一侧高 32 位写成低 32 位）让这一条红。
#[cfg(all(feature = "verdict-store", feature = "gpu"))]
#[test]
fn the_gpu_recovery_kernel_lands_on_a_root_whose_checkpoint_txg_passes_thirty_two_bits_like_the_core_recovery(
) {
    let test_name = "the_gpu_recovery_kernel_lands_on_a_root_whose_checkpoint_txg_passes_thirty_two_bits_like_the_core_recovery";
    let flow = overwrite_flow_published_on_a_checkpoint_txg_past_thirty_two_bits(
        "gpu-recovery-lands-past-thirty-two-bits",
    );
    let judge_tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &quick_tier_expansion,
    })
    .expect("覆盖写那条流的 GPU 判表建得出");
    let enumeration = crash_state_enumeration_tables(
        &flow.base,
        &flow.writes,
        &flow.segments,
        &quick_tier_expansion,
    );
    assert_eq!(
        (
            judge_tables.state_count,
            usize::try_from(judge_tables.enumerated_write_count).expect("装得进 usize")
        ),
        (enumeration.state_count, enumeration.enumerated_writes.len()),
        "前提：GPU 判表与崩溃点重放同一份计划（状态数、枚举写表的写数）"
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(gpu_words_of_each_state) =
        judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &judge_tables)
    else {
        return;
    };
    assert_eq!(
        u64::try_from(gpu_words_of_each_state.len()).expect("装得进 u64"),
        judge_tables.state_count,
        "GPU 判器每个状态交回一份结论"
    );
    let mut mismatches: Vec<String> = Vec::new();
    let mut passes_landed_past_thirty_two_bits = 0u64;
    for (ordinal, gpu_words) in (0..judge_tables.state_count).zip(&gpu_words_of_each_state) {
        let image = CrashImage {
            base: &flow.base,
            writes: &enumeration.enumerated_writes,
            persisted: judge_tables.persisted_writes_of_state(ordinal),
        };
        let gpu_cross_check = cross_check_words_of_gpu_verdict(gpu_words);
        for (pass, policy, on_the_gpu) in [
            (
                "看 journal",
                JournalPolicy::Consult,
                gpu_cross_check.consulted_pass_landed_root,
            ),
            (
                "不看 journal",
                JournalPolicy::Ignore,
                gpu_cross_check.ignored_pass_landed_root,
            ),
        ] {
            let on_the_cpu =
                landed_root_words_of_the_core_recovery(recover(&image, policy).effective_root);
            if on_the_cpu.landed_marker == LandedRootWords::LANDED_ON_A_ROOT
                && on_the_cpu.checkpoint_txg_high != 0
            {
                passes_landed_past_thirty_two_bits += 1;
            }
            if on_the_cpu != on_the_gpu {
                mismatches.push(format!(
                    "第 {ordinal} 个状态{pass}那一遍：核心层的恢复落到 {on_the_cpu:?}，GPU 恢复内核落到 {on_the_gpu:?}"
                ));
            }
        }
    }
    println!(
        "GPU_RECOVERY_LANDS_PAST_THIRTY_TWO_BITS flow=overwrite-past-thirty-two-bits expansion=quick-tier states={} passes_landed_past_thirty_two_bits={passes_landed_past_thirty_two_bits} mismatches={}",
        judge_tables.state_count,
        mismatches.len()
    );
    assert!(
        passes_landed_past_thirty_two_bits > 0,
        "前提：至少一遍恢复落到 checkpoint_txg 高 32 位非 0 的根"
    );
    assert!(
        mismatches.is_empty(),
        "{} 遍恢复上 GPU 恢复内核与核心层的恢复落到的根不同（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 覆盖写 R 之前的覆盖写次数（与 harness 档 `fall_back_judged_on_the_union_of_the_three_readers_and_only_for_lost_bytes.rs` 同一个盘面）。
const OVERWRITES_BEFORE_R: u64 = 5;

/// R 的那条记录怎么交：原样，或第一个点名项的两条位置条目对调成按盘降序（字节读得到、整条校验和重封过，扫环照收；
/// 建上一版判位置条目按盘升序，不收——内容拒收，只有建上一版这一样读者读这条记录）。
#[derive(Clone, Copy, Debug)]
enum RecordOfR {
    AsWritten,
    FirstNamedUnitLocationsInDescendingDeviceOrder,
}

/// 崩在 R 的根槽之后、系统配置轮换之前（R 没见证、P 已确认），R 写出的那个数据单元两份都没落（盘上那两槽还是之前的字节：冷走读读不出、
/// 经映射仍读不出——字节丢了；打开不读数据单元，建上一版照抄读不出的数据单元的位置项）。R 的记录照 `record_of_r` 交。
/// 读的时候没有读故障，镜像就是一个持久集合（不是写序许可的那一种：数据单元缺而根落了，手造的）。
fn image_whose_data_unit_of_r_was_lost(
    recorded: &common_unwitnessed_publish::RecordedPublish,
    record_of_r: RecordOfR,
) -> MemoryPool {
    let data_unit_groups: Vec<Vec<&RetainedWrite>> = recorded
        .unit_writes_grouped_by_unit()
        .into_iter()
        .filter(|copies| parse_data_unit(copies[0].bytes().expect("单元写带字节")).is_ok())
        .collect();
    assert!(
        data_unit_groups.len() == 1 && data_unit_groups[0].len() == 2,
        "前提：覆盖写 R 只写了一个数据单元、两份"
    );
    assert_eq!(recorded.record_count(), 1, "前提：R 那次发布一条记录");
    let record_copies = recorded.record_writes_of_the_record(1);
    let expected_filesystem_identifier =
        unit_filesystem_identifier(&recorded.pool.parameters.filesystem_identifier);
    let record_bytes_handed = |written: &[u8]| -> Vec<u8> {
        let mut record =
            JournalRecord::parse(written, expected_filesystem_identifier).expect("R 的记录解得开");
        assert_eq!(
            record.to_bytes(),
            written,
            "前提：R 的记录解开再写回逐字节相同"
        );
        match record_of_r {
            RecordOfR::AsWritten => {}
            RecordOfR::FirstNamedUnitLocationsInDescendingDeviceOrder => {
                let locations = &mut record
                    .named
                    .first_mut()
                    .expect("R 的记录点名了单元")
                    .locations;
                assert!(
                    locations[0].device < locations[1].device,
                    "前提：写者按盘升序写位置条目"
                );
                locations.swap(0, 1);
            }
        }
        record.to_bytes()
    };
    let writes_up_to_the_root: Vec<RetainedWrite> = recorded.writes[..=recorded.root_index]
        .iter()
        .filter(|write| {
            !data_unit_groups[0]
                .iter()
                .any(|lost| std::ptr::eq(*lost, *write))
        })
        .map(|write| {
            if record_copies.iter().any(|copy| std::ptr::eq(*copy, write)) {
                let mut handed = write.clone();
                handed.contents = WrittenContents::Bytes(record_bytes_handed(
                    write.bytes().expect("记录写带字节"),
                ));
                handed
            } else {
                write.clone()
            }
        })
        .collect();
    let mut image = recorded.base.clone();
    image.apply_writes(&writes_up_to_the_root);
    image
}

/// 场景：没见证的 R 的数据单元两份都没落，R 的记录第一个点名项的位置条目按盘降序（重封过）：冷走读字节丢了，只读挂载的打开读得通，
/// 建上一版拒收这条记录（内容拒收）。同一份盘面换成原样的记录做对照。
/// 预期：原样的记录上两边都退到见证到的 P（三样读者只有字节丢了）；记录拒收的那一份上并集不退（有一样读者内容拒收，C866 改法甲）、
/// 只看走读退到 P——核对用例分得出两边，不是 GPU 对拍。
/// 判别力：`crates/mutations.tsv` 里以「F6 核对用例」开头的两行（只看走读那一判换成并集；核心层并集只看走读那一样的内容拒收），各让这一条红。
#[test]
fn a_selected_version_whose_walk_lost_bytes_and_whose_rebuild_refused_content_is_fallen_back_from_only_by_the_walk_alone(
) {
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let control = image_whose_data_unit_of_r_was_lost(&recorded, RecordOfR::AsWritten);
    let FallBackOfTheState::Judged(control_judged) = fall_back_of_the_state(&control) else {
        panic!("前提：对照盘面上恢复走到判退那一步");
    };
    assert_eq!(
        (
            key_of(&control_judged.chosen_root),
            key_of(&control_judged.selected_version)
        ),
        (recorded.key, recorded.key),
        "前提：所选根是 R，一条记录都没施加"
    );
    assert_eq!(
        (
            control_judged.under_the_union_of_the_three_readers,
            control_judged.under_the_walk_alone
        ),
        (Some(recorded.previous_key), Some(recorded.previous_key)),
        "对照：记录原样时三样读者只有字节丢了，两边都退到见证到的 P"
    );
    assert_eq!(
        disagreement_of_the_state("对照盘面", &control),
        None,
        "对照盘面上核对不报"
    );

    let image = image_whose_data_unit_of_r_was_lost(
        &recorded,
        RecordOfR::FirstNamedUnitLocationsInDescendingDeviceOrder,
    );
    let FallBackOfTheState::Judged(judged) = fall_back_of_the_state(&image) else {
        panic!("前提：恢复走到判退那一步");
    };
    let readers = first_failure_of_each_reader(
        &image,
        &judged.system_configuration,
        &judged.selected_version,
        &judged.records_of_the_site,
    );
    assert_eq!(
        readers
            .walk
            .as_ref()
            .map(RecoveryFailure::failure_kind_for_the_fall_back),
        Some(FailureKindForTheFallBack::BytesLost),
        "前提：冷走读字节丢了：{readers:?}"
    );
    assert_eq!(
        readers
            .opening
            .as_ref()
            .and_then(OpenPoolForReadFailure::failure_kind_for_the_fall_back),
        None,
        "前提：打开读得通（不读数据单元）：{readers:?}"
    );
    assert_eq!(
        readers
            .rebuild
            .as_ref()
            .map(RebuildVersionFailure::failure_kind_for_the_fall_back),
        Some(FailureKindForTheFallBack::ContentRefused),
        "前提：建上一版拒收这条记录（位置条目不按盘升序）：{readers:?}"
    );
    assert!(
        readers.instance_table_of_the_rebuild.is_none(),
        "前提：实例表读得出：{readers:?}"
    );
    assert_eq!(
        (
            judged.under_the_union_of_the_three_readers,
            judged.under_the_walk_alone
        ),
        (None, Some(recorded.previous_key)),
        "只有建上一版拒收内容、冷走读字节丢了：并集不退，只看走读退到 P"
    );
    let line = disagreement_of_the_state("手造盘面", &image).expect("核对在这份盘面上报两边不一致");
    assert!(
        line.contains("并集退到 None") && line.contains("三样读者各停在"),
        "报的那一行带两边各退到哪一版与三样读者各停在哪个失败上：{line}"
    );
}
