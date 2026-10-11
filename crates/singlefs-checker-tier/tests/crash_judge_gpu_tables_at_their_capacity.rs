//! checker 档模块：crash、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器按条数开的各张表在容量边界上：恰好装满的判得了、多一条的交 CPU。先是分配记录那两张表在新上限上的边界（第 59 批 L6，判决 `research/prompts/m3-batch58-code-r1-main-verification.md` 第三节 L6、
//! 第五节用户 2026-10-10 定 ②「抬高 GPU 两张表的上限」）：恢复内核读一版分配记录树的那张表 `MAX_ALLOCATION_RECORDS`
//! （`crash_judge_common.wgsl`）与扫描内核挑最新根已释放记录的那张表 `C_MAX_RELEASED`（`crash_judge_checker_common.wgsl`）都是 2048 条。
//! 造法：新池新建文件之后逐次顺序写第一个文件，单元数照 [`DATA_UNITS_OF_THE_FOUR_GROW_AND_SHRINK_ROUNDS`]（长到 200 个、缩回 1 个，轮四遍），
//! 再长到 56 个（2050 那一形 57 个），最后被判的那次缩回 1 个。每个新写的单元每块盘一条分配记录，缩回去的单元那条记录照旧留着、改成已释放
//! （这段历史只有十次发布，回退下界不抬，一条都不回收）；写表只录被判的那次发布（之前的录制流施加成起点镜像）。
//! 被判那一版的分配记录恰好 2048 条（其中已释放 2010 条，多于抬之前已释放那张表的 1024 条）或 2050 条。
//! 两块盘互为镜像，每个落点每块盘一条记录，条数恒为偶数：2049 条造不出来，2048 之上最近的是 2050。
//! 已释放那张表：扫描内核先把最新根的分配记录收进分配记录那张表（`C_MAX_ALLOC_RECORDS`，也是 2048 条，满了记站点 20），再从里面挑已释放的，
//! 已释放的条数到不了分配记录的条数（这一版自己的单元总有未释放的记录），所以已释放 2049 条那一形走不到，2048 那条流上已释放 2010 条钉的是
//! 「新上限判得到」那一半（改回 1024 时站点 29 判不了）；判不了那一半落在分配记录那张表上。
//! `C_MAX_RELEASED` 直接取 `C_MAX_ALLOC_RECORDS`，站点 29 在每个输入上都走不到：这条推理在扫描内核的 naga 中间表示上逐步落成断言
//! （`site_twenty_nine_of_the_scan_kernel_is_unreachable_because_released_records_are_picked_from_the_allocation_record_table`，不用卡），
//! 指站点 29「抬过上限报判不了」的变异是等价变异，留档在那一条。
//! 2048 那条流：每个状态 GPU 都判得了、对拍字（`crash_judge_gpu::CrossCheckWords`）与 CPU 逐状态相同；
//! 2050 那条流：两遍恢复有一遍落到被判那一版的状态，GPU 恢复内核记判不了（条目过多）；被判那次的根槽写落了的状态，扫描内核记判不了（站点 20）；
//! 别的状态 GPU 判得了、对拍字与 CPU 相同。两条流上 CPU 判器每个状态照旧判、都不红（流是合法的）。
//! 别的表在这两条流上装得下：journal 记录表 `MAX_RECORDS`（2048 条，两个内核都按它收环里的记录）这两条流 864 / 865 条；
//! 走读内核的引用集合 `C_MAX_REFERENCES`、位置项校验和 `C_MAX_CHECKSUMS`、I-3.5 落点表 `C_APPEARANCE_MAX_PLACEMENTS`
//! 在不回收的流上与分配记录同数（2048 / 2050 条），三张都是 4096 条：2050 那条流上先满的是扫描内核的分配记录表（站点 20），
//! 走读内核判得完、交接段置位，扫描内核才跑得到站点 20（三张表还是 2048 条时走读内核先在站点 40、1、2 判不了，扫描内核不跑）。
//! 条数是调查 j4-after-e6 在 CPU 一侧同口径数的（池级 checker 的引用集合按 (盘, 槽, 跨度) 去重、各版累计），GPU 一侧没逐状态数过。
//! 同样造法、同样两半（恰好装满的判得了、多一条的交 CPU）的还有扫描内核读一条根自己实例表全部行的那张表 `C_R_MAX_ROWS`
//! （`crash_judge_checker_common.wgsl`，从 256 抬到 512、与恢复内核的 `MAX_ROWS` 同宽）：取号之后崩溃 511 / 512 次再可写挂载，
//! 被录那次挂载写出的每条根自己那张实例表 512 / 513 行。512 那条流每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同；
//! 513 那条流被判那条根落了的状态扫描内核在站点 32 记判不了、恢复落到那次挂载的根的状态恢复内核记条目过多，别的状态判得了、对拍字与 CPU 相同。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, file_content, geometry, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool, IMAGE_BYTES,
};
use singlefs_checker_tier::crash::{
    judge_layer0_state_range, layer0_plan_state_count, Layer0SegmentExpansion,
};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_judge_gpu::{
    checker_scan_kernel_source, cross_check_words_of_cpu_judgement,
    cross_check_words_of_gpu_verdict, verdict_bits, CrossCheckWords, POOL_CHECKER_UNDECIDABLE_WORD,
    VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run, GpuCardPool,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::instance_table_chain_of_root;
use singlefs_core::transaction::{acquire_instance, PoolWriter, TransactionOutput};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::memory_pool::{
    writes_and_segments, writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::HashSet;
use wgpu::naga::{
    BinaryOperator, Expression, Function, Handle, Literal, LocalVariable, Module, Statement,
};

/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;
/// 两张表的新上限：恢复内核一版的分配记录、扫描内核最新根的已释放记录（与扫描内核分配记录那张表同宽）。
const RECORDS_THE_GPU_TABLES_HOLD: usize = 2048;
/// 抬之前扫描内核已释放记录那张表装几条：2048 那条流上已释放的条数要多于它，改回去才红。
const RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE: usize = 1024;
/// 新上限之上最近造得出来的条数：两块盘互为镜像，一个落点每块盘一条记录，条数恒为偶数。
const RECORDS_JUST_PAST_THE_GPU_TABLES: usize = RECORDS_THE_GPU_TABLES_HOLD + 2;
/// 恢复内核记判不了时结论第 1 字的原因：条目过多（`crash_judge_common.wgsl` 的 `UNDECIDABLE_TOO_MANY_ENTRIES`，分配记录那张表满了就记它）。
const RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES: u32 = 3;
/// 扫描内核分配记录那张表满了时池级 checker 判不了的原因字（`crash_judge_checker_scan.wgsl` 的 `c_append_allocation_record`，站点 20）。
const SCAN_UNDECIDABLE_ALLOCATION_RECORD_TABLE_FULL: u32 =
    RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES | (20 << 8);
/// 被判那次之前逐次顺序写的单元数：长到 200 个、缩回 1 个，轮四遍（每长一次多 400 多条记录，缩回去的那些改成已释放、照旧留着）。
const DATA_UNITS_OF_THE_FOUR_GROW_AND_SHRINK_ROUNDS: [usize; 8] = [200, 1, 200, 1, 200, 1, 200, 1];
/// 四遍之后再长到的单元数：被判那一版的分配记录恰好 [`RECORDS_THE_GPU_TABLES_HOLD`] 条（每个单元每块盘一条，多一个单元多两条）。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_THE_CAPACITY: usize = 56;
/// 2050 那一形多长一个单元。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_THE_CAPACITY: usize =
    DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_THE_CAPACITY + 1;
/// 被判那次缩回的单元数。
const DATA_UNITS_OF_THE_JUDGED_PUBLISH: usize = 1;

/// 一版的分配记录几条、其中已释放几条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AllocationRecordCounts {
    every_record: usize,
    released: usize,
}

fn allocation_record_counts_of(output: &TransactionOutput) -> AllocationRecordCounts {
    AllocationRecordCounts {
        every_record: output.allocation_records.len(),
        released: output
            .allocation_records
            .iter()
            .filter(|record| record.is_released)
            .count(),
    }
}

/// 一条流：起点镜像（被判那次之前的录制流施加上去）、被判那次的写表与段、模型认的每一版，与每一版的分配记录条数。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    judged_txg: CheckpointTxg,
    allocation_record_counts_of_each_version: Vec<(CheckpointTxg, AllocationRecordCounts)>,
}

impl RecordedFlow {
    fn allocation_record_counts_of_the_judged_version(&self) -> AllocationRecordCounts {
        self.allocation_record_counts_of_each_version
            .iter()
            .find(|(txg, _)| *txg == self.judged_txg)
            .map(|(_, counts)| *counts)
            .expect("被判那一版在每一版的条数表里")
    }

    fn most_allocation_records_of_a_version_before_the_judged_one(&self) -> usize {
        self.allocation_record_counts_of_each_version
            .iter()
            .filter(|(txg, _)| *txg < self.judged_txg)
            .map(|(_, counts)| counts.every_record)
            .max()
            .expect("被判那一版之前还有版本")
    }
}

/// 第 `txg` 版文件的内容：`data_units` 个满载的数据单元，字节随 txg 换（相邻两版不同）。
fn content_of_the_version(data_units: usize, txg: CheckpointTxg) -> Vec<u8> {
    let shift = usize::try_from(txg.0 % 249).expect("小于 249");
    (0..data_units * data_unit_payload_capacity())
        .map(|index| u8::try_from((index + shift * 7) % 249).expect("小于 256"))
        .collect()
}

/// 在池上顺序写一次第一个文件（`data_units` 个单元），记下这一版的内容与分配记录条数。
fn publish_a_sequential_write(
    pool: &mut BuiltPool,
    data_units: usize,
    versions: &mut Vec<PublishedVersion>,
    counts: &mut Vec<(CheckpointTxg, AllocationRecordCounts)>,
) {
    let txg = CheckpointTxg(pool.output.root.checkpoint_txg.0 + 1);
    let content = content_of_the_version(data_units, txg);
    let output = sequential_write_in_process_as_instance_one_and_advance_the_pool(pool, &content);
    assert_eq!(
        output.root.checkpoint_txg, txg,
        "一次顺序写一次发布、txg 加一"
    );
    assert_eq!(
        output.data_pointers.len(),
        data_units,
        "txg {} 那一版的单元数",
        txg.0
    );
    counts.push((txg, allocation_record_counts_of(&output)));
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: txg,
        content,
    });
}

/// 新池新建文件（txg 3），照 [`DATA_UNITS_OF_THE_FOUR_GROW_AND_SHRINK_ROUNDS`] 顺序写四遍，再长到 `data_units_before_the_judged_publish` 个单元，
/// 被判的是最后那次缩回 [`DATA_UNITS_OF_THE_JUDGED_PUBLISH`] 个单元的发布：起点镜像是 mkfs 到它之前的整段录制流施加出来的那一份，写表只录它。
fn record_the_flow(tag: &str, data_units_before_the_judged_publish: usize) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let mut counts = vec![(
        pool.output.root.checkpoint_txg,
        allocation_record_counts_of(&pool.output),
    )];
    // 迭代上界：九次发布。
    for data_units in DATA_UNITS_OF_THE_FOUR_GROW_AND_SHRINK_ROUNDS
        .into_iter()
        .chain([data_units_before_the_judged_publish])
    {
        publish_a_sequential_write(&mut pool, data_units, &mut versions, &mut counts);
    }
    let operations_before_the_judged_publish = pool.stream.operation_count();
    publish_a_sequential_write(
        &mut pool,
        DATA_UNITS_OF_THE_JUDGED_PUBLISH,
        &mut versions,
        &mut counts,
    );
    let judged_txg = pool.output.root.checkpoint_txg;
    let operations = pool.retained_operations();
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&operations[pool.mkfs_operation_count..operations_before_the_judged_publish]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_judged_publish..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("被判那次发布有根槽写");
    RecordedFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        judged_txg,
        allocation_record_counts_of_each_version: counts,
    }
}

/// 一个状态 CPU 那一侧判出的：对拍字、红项、两遍恢复有没有一遍落到被判那一版、被判那次的根槽写落没落。
struct CpuJudgementOfAState {
    cross_check_words: CrossCheckWords,
    red_items: Vec<String>,
    a_recovery_lands_on_the_judged_version: bool,
    judged_root_persisted: bool,
}

/// 每个状态两边各判一遍：GPU 结论字与 CPU 那一侧，按状态序号。卡借不到（等满了、或没有独显）交 `None`。
fn judge_every_state_on_both_sides(
    cards: &'static GpuCardPool,
    test_name: &str,
    flow: &RecordedFlow,
) -> Option<(Vec<[u32; VERDICT_WORDS]>, Vec<CpuJudgementOfAState>)> {
    let expansion = |_segment_index: usize, segment: &[usize]| {
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
    .expect("建得出表");
    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count, "表与枚举的状态数相同");
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)?;
    let judged_txg = flow.judged_txg;
    let mut cpu: Vec<CpuJudgementOfAState> = Vec::new();
    let judged_on_the_cpu = judge_layer0_state_range(
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| {
            let lands_on_the_judged_version =
                |root: Option<(InstanceGeneration, CheckpointTxg)>| {
                    root.is_some_and(|(_, txg)| txg == judged_txg)
                };
            cpu.push(CpuJudgementOfAState {
                cross_check_words: cross_check_words_of_cpu_judgement(judgement),
                red_items: judgement.red_items(),
                a_recovery_lands_on_the_judged_version: lands_on_the_judged_version(
                    judgement.consulted.effective_root,
                ) || lands_on_the_judged_version(
                    judgement.ignored.effective_root,
                ),
                judged_root_persisted: judgement.root_persisted,
            });
        },
    );
    assert_eq!(judged_on_the_cpu, state_count, "CPU 判了每个状态");
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    Some((gpu, cpu))
}

fn gpu_could_not_judge(gpu_words: &[u32; VERDICT_WORDS]) -> bool {
    gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
        || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0
}

/// 流本身合法：CPU 判器每个状态都不红（这是「CPU 判器照旧判」那一半：GPU 判不了的状态 CPU 照样判完、判出来是绿的）。
fn assert_the_cpu_judge_finds_every_state_green(flow_name: &str, cpu: &[CpuJudgementOfAState]) {
    let red: Vec<String> = cpu
        .iter()
        .enumerate()
        .filter(|(_, judgement)| !judgement.red_items.is_empty())
        .take(8)
        .map(|(ordinal, judgement)| format!("状态 {ordinal}：{:?}", judgement.red_items))
        .collect();
    assert!(
        red.is_empty(),
        "流 {flow_name} 合法，CPU 判器每个状态都不该红（前几个）：\n{}",
        red.join("\n")
    );
}

/// 每一版的分配记录条数：被判那一版恰好 `judged_version_records` 条，之前每一版都不超过两张表的新上限（那几版 GPU 都判得了）。
fn assert_the_allocation_records_of_the_versions(
    flow_name: &str,
    flow: &RecordedFlow,
    judged_version_records: usize,
) {
    let judged = flow.allocation_record_counts_of_the_judged_version();
    assert_eq!(
        judged.every_record, judged_version_records,
        "流 {flow_name}：被判那一版（txg {}）的分配记录条数（每一版：{:?}）",
        flow.judged_txg.0, flow.allocation_record_counts_of_each_version
    );
    assert!(
        flow.most_allocation_records_of_a_version_before_the_judged_one()
            <= RECORDS_THE_GPU_TABLES_HOLD,
        "流 {flow_name}：被判那一版之前每一版的分配记录不超过 {RECORDS_THE_GPU_TABLES_HOLD} 条（每一版：{:?}）",
        flow.allocation_record_counts_of_each_version
    );
}

/// 被判那一版的分配记录恰好 2048 条、其中已释放 2010 条：每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同。
/// 两张表里有一张小于 2048（改回 1024），有一遍恢复落到那一版的状态恢复内核就记判不了、或那一版的根槽写落了的状态扫描内核在站点 29 记判不了。
#[test]
fn the_gpu_judge_judges_a_version_of_exactly_two_thousand_and_forty_eight_allocation_records_like_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_judges_a_version_of_exactly_two_thousand_and_forty_eight_allocation_records_like_the_cpu_judge";
    const FLOW_NAME: &str = "allocation-records-at-the-capacity-of-the-gpu-tables";
    // 前提（取卡之前、CPU 上判）：被判那一版恰好 2048 条，已释放的多于抬之前那张表的 1024 条、不多于新上限。
    let flow = record_the_flow(
        "crash-judge-gpu-allocation-records-at-the-capacity",
        DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_THE_CAPACITY,
    );
    assert_the_allocation_records_of_the_versions(FLOW_NAME, &flow, RECORDS_THE_GPU_TABLES_HOLD);
    let judged = flow.allocation_record_counts_of_the_judged_version();
    assert!(
        judged.released > RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE
            && judged.released <= RECORDS_THE_GPU_TABLES_HOLD,
        "流 {FLOW_NAME}：被判那一版已释放 {} 条，要多于 {RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE}、不多于 {RECORDS_THE_GPU_TABLES_HOLD}",
        judged.released
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    let states_reading_the_judged_version_on_both_kernels = cpu
        .iter()
        .filter(|judgement| {
            judgement.a_recovery_lands_on_the_judged_version && judgement.judged_root_persisted
        })
        .count();
    assert!(
        states_reading_the_judged_version_on_both_kernels > 0,
        "流 {FLOW_NAME}：要有被判那次的根槽写落了、恢复落到那一版的状态（恢复内核读那一版 2048 条、扫描内核挑它已释放的 {} 条）",
        judged.released
    );
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(&gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_judgement, gpu_words))| {
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (gpu_could_not_judge(gpu_words)
                || gpu_cross_check_words != cpu_judgement.cross_check_words)
                .then(|| {
                    format!(
                        "状态 {ordinal}：GPU 判不了 {}（恢复第 1 字 {:#x}、池级 checker 原因字 {:#x}）；GPU {gpu_cross_check_words:?}；CPU {:?}",
                        gpu_could_not_judge(gpu_words),
                        gpu_words[1],
                        gpu_words[POOL_CHECKER_UNDECIDABLE_WORD],
                        cpu_judgement.cross_check_words
                    )
                })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "流 {FLOW_NAME}：{} 个状态 GPU 判不了或与 CPU 不一致（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 被判那一版的分配记录 2050 条（2048 之上最近造得出来的条数）：两遍恢复有一遍落到那一版的状态，GPU 恢复内核记判不了（条目过多）；
/// 那一版的根槽写落了的状态，扫描内核在站点 20 记判不了；别的状态 GPU 判得了、对拍字与 CPU 相同。CPU 判器每个状态照旧判、都不红。
#[test]
fn the_gpu_judge_leaves_a_version_of_two_thousand_and_fifty_allocation_records_to_the_cpu_judge() {
    const TEST_NAME: &str =
        "the_gpu_judge_leaves_a_version_of_two_thousand_and_fifty_allocation_records_to_the_cpu_judge";
    const FLOW_NAME: &str = "allocation-records-just-past-the-capacity-of-the-gpu-tables";
    let flow = record_the_flow(
        "crash-judge-gpu-allocation-records-just-past-the-capacity",
        DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_THE_CAPACITY,
    );
    assert_the_allocation_records_of_the_versions(
        FLOW_NAME,
        &flow,
        RECORDS_JUST_PAST_THE_GPU_TABLES,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    let mut states_left_to_the_cpu = 0usize;
    let mut problems: Vec<String> = Vec::new();
    for (ordinal, (cpu_judgement, gpu_words)) in cpu.iter().zip(&gpu).enumerate() {
        let recovery_could_not_judge = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0;
        let checker_reason = gpu_words[POOL_CHECKER_UNDECIDABLE_WORD];
        if cpu_judgement.a_recovery_lands_on_the_judged_version
            && !(recovery_could_not_judge && gpu_words[1] == RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES)
        {
            problems.push(format!(
                "状态 {ordinal}：恢复落到 2050 条那一版，GPU 恢复内核该记判不了（条目过多 {RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES}），实际判不了 {recovery_could_not_judge}、第 1 字 {:#x}",
                gpu_words[1]
            ));
        }
        if cpu_judgement.judged_root_persisted
            && checker_reason != SCAN_UNDECIDABLE_ALLOCATION_RECORD_TABLE_FULL
        {
            problems.push(format!(
                "状态 {ordinal}：2050 条那一版的根槽写落了，扫描内核该在站点 20 记判不了（{SCAN_UNDECIDABLE_ALLOCATION_RECORD_TABLE_FULL:#x}），实际原因字 {checker_reason:#x}"
            ));
        }
        if gpu_could_not_judge(gpu_words) {
            states_left_to_the_cpu += 1;
            continue;
        }
        let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
        if gpu_cross_check_words != cpu_judgement.cross_check_words {
            problems.push(format!(
                "状态 {ordinal}：GPU 判得了而与 CPU 不一致：GPU {gpu_cross_check_words:?}；CPU {:?}",
                cpu_judgement.cross_check_words
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "流 {FLOW_NAME}：{} 处不对（前几处）：\n{}",
        problems.len(),
        problems
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        states_left_to_the_cpu > 0,
        "流 {FLOW_NAME}：要有 GPU 判不了、交 CPU 判的状态（恢复落到 2050 条那一版，或它的根槽写落了）"
    );
}

// ───────── 扫描内核读一条根自己实例表全部行的那张表：恰好装满与多一行 ─────────

/// 扫描内核读一条根自己实例表全部行的那张表（`crash_judge_checker_common.wgsl` 的 `C_R_MAX_ROWS`）装几行：与恢复内核读同一种表的
/// `MAX_ROWS`（`crash_judge_common.wgsl`）同宽。
const INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD: usize = 512;
/// 扫描内核那张表抬之前装几行：恰好装满那条流的行数要多于它，改回去才红。
const INSTANCE_TABLE_ROWS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE: usize = 256;
const _: () = assert!(
    INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD
        > INSTANCE_TABLE_ROWS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE,
    "恰好装满那条流的行数要多于抬之前扫描内核那张表的行数"
);
/// 扫描内核那张表满了时池级 checker 判不了的原因字（`crash_judge_checker_scan.wgsl` 的 `r_instance_rows`，站点 32；条目过多那一类）。
const SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL: u32 =
    RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES | (32 << 8);
/// 两块盘（同 `crash_enumeration_position_addressed_trees.rs` 取号那条流）。
const DEVICES_OF_THE_ROW_PUBLISH_FLOW: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 一条录制写的稀疏盘：取号与可写挂载直接在它上面跑，录制流记下每一次写。
type RecordingSparseDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 一条流：起点镜像、被录那次可写挂载的写表与段、被判的那条根槽写，与那次挂载写出的每一条根（写行那次与之后的暖机）的
/// (实例, txg) 和它自己那张实例表沿链读出的行数。这一版没有文件，版本表为空。
struct RowPublishFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    roots_of_the_recorded_mount: Vec<(InstanceGeneration, CheckpointTxg, usize)>,
}

/// mkfs → 可写挂载一次（实例 1，树表 0 条，上一个实例是 0、不写行）→「取号之后崩溃」`instances_acquired_and_crashed` 次 →
/// 被录的那一次可写挂载：写行那次给实例 1 与中间每个取过号的实例各写一行，共 `instances_acquired_and_crashed + 1` 行
/// （一片实例表装 369 行，五百来行是两片），之后零单元暖机，暖机的根照抄这张表。造法同 `crash_enumeration_position_addressed_trees.rs`
/// 的 `VersionWithoutFileRowPublishWithItsOwnTree`（那条取号 369 次、写 370 行，扫描内核那张表 256 行时在站点 32 判不了）。
/// 写表只录那一次挂载（取号、写行、暖机）；起点镜像是它之前的整段录制流施加出来的。
fn record_the_row_publish_flow(instances_acquired_and_crashed: usize) -> RowPublishFlow {
    let stream = SharedStream::retaining_contents();
    let mut devices: RecordingSparseDevices = DEVICES_OF_THE_ROW_PUBLISH_FLOW
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    make_filesystem(&parameters(), &mut devices).expect("mkfs");
    let first_mount = mount_writable(&parameters(), &mut devices).expect("mkfs 之后第一次可写挂载");
    assert!(
        first_mount.output.rows_written.is_empty(),
        "上一个实例是 0：mkfs 之后第一次可写挂载不写行，被录那次挂载写的行就是那张表的全部行：{:?}",
        first_mount.output.rows_written
    );
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        // 迭代上界：`instances_acquired_and_crashed` 次取号。
        for _ in 0..instances_acquired_and_crashed {
            acquire_instance(&mut writer).expect("取号之后崩溃");
        }
    }
    let operations_before_the_mount = stream.operations().len();
    let mut base = MemoryPool::with_devices(&DEVICES_OF_THE_ROW_PUBLISH_FLOW, IMAGE_BYTES);
    base.apply(&stream.retained_operations()[..operations_before_the_mount]);
    let mounted = mount_writable(&parameters(), &mut devices).expect("树表 0 条的一版上写行");
    let operations = stream.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[operations_before_the_mount..], &geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写行与暖机各有根槽写");
    let roots_of_the_recorded_mount = std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|version| {
            let root = version.root();
            let rows = instance_table_chain_of_root(&devices, root)
                .expect("被录那次挂载刚写的实例表沿链读得出")
                .records
                .rows
                .len();
            (root.instance, root.checkpoint_txg, rows)
        })
        .collect();
    RowPublishFlow {
        base,
        writes,
        segments,
        judged_root_index,
        roots_of_the_recorded_mount,
    }
}

/// 被录那次挂载写出的每一条根自己那张实例表都恰好 `rows` 行。
fn assert_every_root_of_the_recorded_mount_has_an_instance_table_of(
    flow_name: &str,
    flow: &RowPublishFlow,
    rows: usize,
) {
    assert!(
        !flow.roots_of_the_recorded_mount.is_empty()
            && flow
                .roots_of_the_recorded_mount
                .iter()
                .all(|(_, _, rows_of_the_root)| *rows_of_the_root == rows),
        "流 {flow_name}：被录那次挂载写出的每一条根自己那张实例表都是 {rows} 行（每一条根的实例、txg、行数：{:?}）",
        flow.roots_of_the_recorded_mount
    );
}

/// 一个状态 CPU 那一侧判出的（实例表行那两条流）：对拍字、红项、看 journal 那一遍恢复落没落到被录那次挂载写的根、被判那次的根槽写落没落。
struct CpuJudgementOfARowPublishState {
    cross_check_words: CrossCheckWords,
    red_items: Vec<String>,
    consulted_recovery_lands_on_a_root_of_the_recorded_mount: bool,
    judged_root_persisted: bool,
}

/// 每个状态两边各判一遍（实例表行那两条流）：GPU 结论字与 CPU 那一侧，按状态序号。卡借不到（等满了、或没有独显）交 `None`。
fn judge_every_row_publish_state_on_both_sides(
    cards: &'static GpuCardPool,
    test_name: &str,
    flow: &RowPublishFlow,
) -> Option<(
    Vec<[u32; VERDICT_WORDS]>,
    Vec<CpuJudgementOfARowPublishState>,
)> {
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            Layer0SegmentExpansion::EveryProperSubset
        } else {
            Layer0SegmentExpansion::NotExpanded
        }
    };
    let no_versions: Vec<PublishedVersion> = Vec::new();
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &no_versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count, "表与枚举的状态数相同");
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)?;
    let roots_of_the_recorded_mount: HashSet<(InstanceGeneration, CheckpointTxg)> = flow
        .roots_of_the_recorded_mount
        .iter()
        .map(|(instance, txg, _)| (*instance, *txg))
        .collect();
    let mut cpu: Vec<CpuJudgementOfARowPublishState> = Vec::new();
    let judged_on_the_cpu = judge_layer0_state_range(
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &no_versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| {
            cpu.push(CpuJudgementOfARowPublishState {
                cross_check_words: cross_check_words_of_cpu_judgement(judgement),
                red_items: judgement.red_items(),
                consulted_recovery_lands_on_a_root_of_the_recorded_mount: judgement
                    .consulted
                    .effective_root
                    .is_some_and(|root| roots_of_the_recorded_mount.contains(&root)),
                judged_root_persisted: judgement.root_persisted,
            });
        },
    );
    assert_eq!(judged_on_the_cpu, state_count, "CPU 判了每个状态");
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    Some((gpu, cpu))
}

/// 实例表行那两条流本身合法：CPU 判器每个状态都不红。
fn assert_the_cpu_judge_finds_every_row_publish_state_green(
    flow_name: &str,
    cpu: &[CpuJudgementOfARowPublishState],
) {
    let red: Vec<String> = cpu
        .iter()
        .enumerate()
        .filter(|(_, judgement)| !judgement.red_items.is_empty())
        .take(8)
        .map(|(ordinal, judgement)| format!("状态 {ordinal}：{:?}", judgement.red_items))
        .collect();
    assert!(
        red.is_empty(),
        "流 {flow_name} 合法，CPU 判器每个状态都不该红（前几个）：\n{}",
        red.join("\n")
    );
}

/// 取号之后崩溃 511 次再挂载：被录那次挂载写出的每条根自己那张实例表恰好 512 行（多于抬之前扫描内核那张表的 256 行）。
/// 每个状态 GPU 两个内核都判得了，对拍字与 CPU 逐状态相同；被判那条根落了的状态里扫描内核读那张 512 行的表。
/// 扫描内核那张表改回 256 行时，那几个状态在站点 32 记判不了，这条红。
#[test]
fn the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_five_hundred_and_twelve_rows_like_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_five_hundred_and_twelve_rows_like_the_cpu_judge";
    const FLOW_NAME: &str = "instance-table-rows-at-the-capacity-of-the-gpu-tables";
    // 前提（取卡之前、CPU 上判）：被录那次挂载的每条根恰好 512 行，多于抬之前那张表的 256 行。
    let flow = record_the_row_publish_flow(INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD - 1);
    assert_every_root_of_the_recorded_mount_has_an_instance_table_of(
        FLOW_NAME,
        &flow,
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_row_publish_state_on_both_sides(cards, TEST_NAME, &flow)
    else {
        return;
    };
    assert_the_cpu_judge_finds_every_row_publish_state_green(FLOW_NAME, &cpu);
    assert!(
        cpu.iter().any(|judgement| judgement.judged_root_persisted),
        "流 {FLOW_NAME}：要有被判那次的根槽写落了的状态（扫描内核读那条根 512 行的表）"
    );
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(&gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_judgement, gpu_words))| {
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (gpu_could_not_judge(gpu_words)
                || gpu_cross_check_words != cpu_judgement.cross_check_words)
                .then(|| {
                    format!(
                        "状态 {ordinal}：GPU 判不了 {}（恢复第 1 字 {:#x}、池级 checker 原因字 {:#x}）；GPU {gpu_cross_check_words:?}；CPU {:?}",
                        gpu_could_not_judge(gpu_words),
                        gpu_words[1],
                        gpu_words[POOL_CHECKER_UNDECIDABLE_WORD],
                        cpu_judgement.cross_check_words
                    )
                })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "流 {FLOW_NAME}：{} 个状态 GPU 判不了或与 CPU 不一致（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 取号之后崩溃 512 次再挂载：被录那次挂载写出的每条根自己那张实例表 513 行（多新上限一行）。被判那条根落了的状态，
/// 扫描内核读到第 513 行时在站点 32 记判不了（[`SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL`]）；看 journal 那一遍恢复落到那次挂载写的根的状态，
/// 恢复内核读那一版的实例表读到第 513 行同样记判不了（条目过多 [`RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES`]，两个内核的表同宽）。
/// 两个内核判不了的只许是这两个原因；两个内核都判得了的状态对拍字与 CPU 相同；CPU 判器每个状态照旧判、都不红。
#[test]
fn the_gpu_judge_leaves_roots_whose_own_instance_table_has_five_hundred_and_thirteen_rows_to_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_leaves_roots_whose_own_instance_table_has_five_hundred_and_thirteen_rows_to_the_cpu_judge";
    const FLOW_NAME: &str = "instance-table-rows-just-past-the-capacity-of-the-gpu-tables";
    let flow = record_the_row_publish_flow(INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD);
    assert_every_root_of_the_recorded_mount_has_an_instance_table_of(
        FLOW_NAME,
        &flow,
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD + 1,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_row_publish_state_on_both_sides(cards, TEST_NAME, &flow)
    else {
        return;
    };
    assert_the_cpu_judge_finds_every_row_publish_state_green(FLOW_NAME, &cpu);
    let mut states_left_to_the_cpu_at_site_thirty_two = 0usize;
    let mut problems: Vec<String> = Vec::new();
    for (ordinal, (cpu_judgement, gpu_words)) in cpu.iter().zip(&gpu).enumerate() {
        let recovery_could_not_judge = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0;
        let checker_reason = gpu_words[POOL_CHECKER_UNDECIDABLE_WORD];
        if recovery_could_not_judge && gpu_words[1] != RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES {
            problems.push(format!(
                "状态 {ordinal}：GPU 恢复内核判不了的原因 {:#x} 不是条目过多（{RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES}）",
                gpu_words[1]
            ));
        }
        if cpu_judgement.consulted_recovery_lands_on_a_root_of_the_recorded_mount
            && !recovery_could_not_judge
        {
            problems.push(format!(
                "状态 {ordinal}：看 journal 那一遍恢复落到 513 行那张表的根，GPU 恢复内核该记判不了（条目过多），实际判得了、第 1 字 {:#x}",
                gpu_words[1]
            ));
        }
        if checker_reason != 0 && checker_reason != SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL {
            problems.push(format!(
                "状态 {ordinal}：GPU 池级 checker 判不了的原因字 {checker_reason:#x} 不是站点 32（{SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL:#x}）"
            ));
        }
        if cpu_judgement.judged_root_persisted
            && checker_reason != SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL
        {
            problems.push(format!(
                "状态 {ordinal}：513 行那张表的被判那条根槽写落了，扫描内核该在站点 32 记判不了（{SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL:#x}），实际原因字 {checker_reason:#x}"
            ));
        }
        if checker_reason == SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL {
            states_left_to_the_cpu_at_site_thirty_two += 1;
        }
        if gpu_could_not_judge(gpu_words) {
            continue;
        }
        let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
        if gpu_cross_check_words != cpu_judgement.cross_check_words {
            problems.push(format!(
                "状态 {ordinal}：GPU 判得了而与 CPU 不一致：GPU {gpu_cross_check_words:?}；CPU {:?}",
                cpu_judgement.cross_check_words
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "流 {FLOW_NAME}：{} 处不对（前几处）：\n{}",
        problems.len(),
        problems
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        states_left_to_the_cpu_at_site_thirty_two > 0,
        "流 {FLOW_NAME}：要有扫描内核在站点 32 记判不了、交 CPU 判的状态（被判那条根槽写落了的状态）"
    );
}

// ───────── 已释放记录那张表：站点 29 走不到（等价变异留档，`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」） ─────────

/// 扫描内核里最新根已释放记录的条数（`crash_judge_checker_scan.wgsl` 的 `var<private>`）。
const RELEASED_RECORD_COUNTER: &str = "c_released_count";
/// 扫描内核里一个根的分配记录条数（同上）。
const ALLOCATION_RECORD_COUNTER: &str = "c_alloc_record_count";
/// 已释放记录那张表的上限：站点 29 拿已释放条数比的那个数。
const RELEASED_RECORD_TABLE_CAPACITY: &str = "C_MAX_RELEASED";
/// 分配记录那张表的上限：站点 20 拿分配记录条数比的那个数。
const ALLOCATION_RECORD_TABLE_CAPACITY: &str = "C_MAX_ALLOC_RECORDS";

/// 扫描内核的每个函数，连同入口：名字与中间表示。
fn every_function_of(module: &Module) -> Vec<(&str, &Function)> {
    module
        .functions
        .iter()
        .map(|(_, function)| (function.name.as_deref().unwrap_or("（没有名字）"), function))
        .chain(
            module
                .entry_points
                .iter()
                .map(|entry_point| (entry_point.name.as_str(), &entry_point.function)),
        )
        .collect()
}

/// 一串语句与它们里面套着的每一个语句块（`if` 两支、`switch` 每支、循环体与 continuing、裸的语句块），这一串自己在最前。
#[allow(
    clippy::wildcard_enum_match_arm,
    reason = "naga 的语句是外部的枚举，只有这四种里还套着语句块，其余一律不含语句块"
)]
fn statement_lists_nested_in(statements: &[Statement]) -> Vec<&[Statement]> {
    let mut lists = vec![statements];
    for statement in statements {
        match statement {
            Statement::Block(inner) => lists.extend(statement_lists_nested_in(inner)),
            Statement::If { accept, reject, .. } => {
                lists.extend(statement_lists_nested_in(accept));
                lists.extend(statement_lists_nested_in(reject));
            }
            Statement::Switch { cases, .. } => {
                for case in cases {
                    lists.extend(statement_lists_nested_in(&case.body));
                }
            }
            Statement::Loop {
                body, continuing, ..
            } => {
                lists.extend(statement_lists_nested_in(body));
                lists.extend(statement_lists_nested_in(continuing));
            }
            _ => {}
        }
    }
    lists
}

fn is_the_global_variable(
    module: &Module,
    function: &Function,
    expression: Handle<Expression>,
    name: &str,
) -> bool {
    matches!(
        function.expressions[expression],
        Expression::GlobalVariable(global) if module.global_variables[global].name.as_deref() == Some(name)
    )
}

fn is_a_load_of_the_global_variable(
    module: &Module,
    function: &Function,
    expression: Handle<Expression>,
    name: &str,
) -> bool {
    matches!(
        function.expressions[expression],
        Expression::Load { pointer } if is_the_global_variable(module, function, pointer, name)
    )
}

fn the_local_variable_loaded_by(
    function: &Function,
    expression: Handle<Expression>,
) -> Option<Handle<LocalVariable>> {
    let Expression::Load { pointer } = function.expressions[expression] else {
        return None;
    };
    let Expression::LocalVariable(local_variable) = function.expressions[pointer] else {
        return None;
    };
    Some(local_variable)
}

fn is_the_u32_literal(function: &Function, expression: Handle<Expression>, value: u32) -> bool {
    matches!(
        function.expressions[expression],
        Expression::Literal(Literal::U32(literal)) if literal == value
    )
}

fn is_the_named_constant(
    module: &Module,
    function: &Function,
    expression: Handle<Expression>,
    name: &str,
) -> bool {
    matches!(
        function.expressions[expression],
        Expression::Constant(constant) if module.constants[constant].name.as_deref() == Some(name)
    )
}

/// 一个 `u32` 模块常量的值：初值是字面量，或指到别的常量（顺着指过去，链长有限：WGSL 不许常量互相引用成环）。
fn value_of_the_u32_constant(module: &Module, name: &str) -> u32 {
    let (_, constant) = module
        .constants
        .iter()
        .find(|(_, constant)| constant.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("扫描内核里没有常量 {name}"));
    let mut initializer = constant.init;
    // 迭代上界：模块常量的个数（每一步换到另一个常量的初值）
    for _step in 0..=module.constants.len() {
        if let Expression::Literal(Literal::U32(value)) = module.global_expressions[initializer] {
            return value;
        }
        let Expression::Constant(other) = module.global_expressions[initializer] else {
            panic!(
                "常量 {name} 的初值既不是 u32 字面量也不是别的常量：{:?}",
                module.global_expressions[initializer]
            );
        };
        initializer = module.constants[other].init;
    }
    panic!(
        "常量 {name} 顺着指了 {} 步还没到字面量",
        module.constants.len() + 1
    );
}

/// 一次写一个计数（`u32` 的 `var<private>` 或循环变量）写成了什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CounterStore {
    SetToZero,
    IncrementedByOne,
    /// 写回之前读到的它自己的值（`let records_before = c_alloc_record_count;` 之后的 `c_alloc_record_count = records_before;`）。
    RestoredToAnEarlierReading,
    WrittenOtherwise,
}

/// 一个计数被写成 `value`；`is_a_load_of_the_counter` 认一个表达式是不是读这个计数。
fn counter_store_of(
    function: &Function,
    value: Handle<Expression>,
    is_a_load_of_the_counter: &dyn Fn(Handle<Expression>) -> bool,
) -> CounterStore {
    if is_the_u32_literal(function, value, 0) {
        return CounterStore::SetToZero;
    }
    if is_a_load_of_the_counter(value) {
        return CounterStore::RestoredToAnEarlierReading;
    }
    if let Expression::Binary {
        op: BinaryOperator::Add,
        left,
        right,
    } = function.expressions[value]
    {
        if is_a_load_of_the_counter(left) && is_the_u32_literal(function, right, 1) {
            return CounterStore::IncrementedByOne;
        }
    }
    CounterStore::WrittenOtherwise
}

/// 这一句写全局计数 `counter` 的话，写成了什么。
fn store_to_the_global_counter(
    module: &Module,
    function: &Function,
    statement: &Statement,
    counter: &str,
) -> Option<CounterStore> {
    let Statement::Store { pointer, value } = *statement else {
        return None;
    };
    is_the_global_variable(module, function, pointer, counter).then(|| {
        counter_store_of(function, value, &|expression| {
            is_a_load_of_the_global_variable(module, function, expression, counter)
        })
    })
}

/// 一串语句（连同里面套着的）里每一次写全局计数 `counter`。
fn stores_to_the_global_counter_in(
    module: &Module,
    function: &Function,
    statements: &[Statement],
    counter: &str,
) -> Vec<CounterStore> {
    statement_lists_nested_in(statements)
        .into_iter()
        .flat_map(|nested| nested.iter())
        .filter_map(|statement| store_to_the_global_counter(module, function, statement, counter))
        .collect()
}

/// 一串语句（连同里面套着的）里每一次写循环变量 `loop_variable`。
fn stores_to_the_local_variable_in(
    function: &Function,
    statements: &[Statement],
    loop_variable: Handle<LocalVariable>,
) -> Vec<CounterStore> {
    statement_lists_nested_in(statements)
        .into_iter()
        .flat_map(|nested| nested.iter())
        .filter_map(|statement| {
            let Statement::Store { pointer, value } = *statement else {
                return None;
            };
            matches!(function.expressions[pointer], Expression::LocalVariable(stored) if stored == loop_variable)
                .then(|| {
                    counter_store_of(function, value, &|expression| {
                        the_local_variable_loaded_by(function, expression) == Some(loop_variable)
                    })
                })
        })
        .collect()
}

/// 一串语句（连同里面套着的）里每一次调用：被调的函数与交过去的参数。
fn calls_in(statements: &[Statement]) -> Vec<(Handle<Function>, &[Handle<Expression>])> {
    statement_lists_nested_in(statements)
        .into_iter()
        .flat_map(|nested| nested.iter())
        .filter_map(|statement| {
            let Statement::Call {
                function,
                ref arguments,
                ..
            } = *statement
            else {
                return None;
            };
            Some((function, arguments.as_slice()))
        })
        .collect()
}

/// 直接或经调用写全局计数 `counter` 的函数（WGSL 不许递归，调用关系无环）。
fn functions_storing_the_global_counter(
    module: &Module,
    counter: &str,
) -> HashSet<Handle<Function>> {
    let mut storing: HashSet<Handle<Function>> = module
        .functions
        .iter()
        .filter(|(_, function)| {
            !stores_to_the_global_counter_in(module, function, &function.body, counter).is_empty()
        })
        .map(|(handle, _)| handle)
        .collect();
    // 迭代上界：函数的个数（每一轮至少多认出一个，认不出就停）
    for _round in 0..=module.functions.len() {
        let before = storing.len();
        for (handle, function) in module.functions.iter() {
            if calls_in(&function.body)
                .iter()
                .any(|(callee, _)| storing.contains(callee))
            {
                storing.insert(handle);
            }
        }
        if storing.len() == before {
            return storing;
        }
    }
    panic!("调用关系里有环：WGSL 不许递归，naga 解析得过的源码走不到这里");
}

/// 这一句是不是「计数到了上限就返回」那一判：`if (<计数> >= <上限常量>) { …; return; }`，没有 else。
fn is_a_guard_returning_once_the_counter_reaches_the_capacity(
    module: &Module,
    function: &Function,
    statement: &Statement,
    counter: &str,
    capacity: &str,
) -> bool {
    let Statement::If {
        condition,
        ref accept,
        ref reject,
    } = *statement
    else {
        return false;
    };
    let Expression::Binary {
        op: BinaryOperator::GreaterEqual,
        left,
        right,
    } = function.expressions[condition]
    else {
        return false;
    };
    is_a_load_of_the_global_variable(module, function, left, counter)
        && is_the_named_constant(module, function, right, capacity)
        && matches!(accept.last(), Some(Statement::Return { .. }))
        && reject.is_empty()
}

/// 挑最新根已释放记录的那个 for 循环。naga 把 `for (<初值>; <条件>; <每轮加>) { <循环体> }` 拆成：初值在外面那个语句块里，
/// `loop { if (<条件>) {} else { break; } { <循环体> } continuing { <每轮加> } }`。
struct ReleasedRecordPickingLoop<'module> {
    function_name: &'module str,
    function: &'module Function,
    enclosing_block: &'module [Statement],
    position_in_the_enclosing_block: usize,
    loop_variable: Handle<LocalVariable>,
    whole_body: &'module [Statement],
    for_body: &'module [Statement],
    continuing: &'module [Statement],
}

/// 条件是「循环变量 < 分配记录条数」、循环体顶层有一句已释放条数加一的 for 循环，扫描内核里的每一个。
fn every_released_record_picking_loop(module: &Module) -> Vec<ReleasedRecordPickingLoop<'_>> {
    let mut loops = Vec::new();
    for (function_name, function) in every_function_of(module) {
        for enclosing_block in statement_lists_nested_in(&function.body) {
            for (position_in_the_enclosing_block, statement) in enclosing_block.iter().enumerate() {
                let Statement::Loop {
                    body: ref whole_body,
                    ref continuing,
                    break_if: None,
                } = *statement
                else {
                    continue;
                };
                let mut statements_other_than_emit = whole_body
                    .iter()
                    .filter(|body_statement| !matches!(body_statement, Statement::Emit(_)));
                let (Some(first), Some(second)) = (
                    statements_other_than_emit.next(),
                    statements_other_than_emit.next(),
                ) else {
                    continue;
                };
                let Statement::If {
                    condition,
                    ref accept,
                    ref reject,
                } = *first
                else {
                    continue;
                };
                let Statement::Block(ref for_body) = *second else {
                    continue;
                };
                let is_the_condition_of_a_for_loop = accept.is_empty()
                    && reject.len() == 1
                    && matches!(reject.first(), Some(Statement::Break));
                let Expression::Binary {
                    op: BinaryOperator::Less,
                    left,
                    right,
                } = function.expressions[condition]
                else {
                    continue;
                };
                let Some(loop_variable) = the_local_variable_loaded_by(function, left) else {
                    continue;
                };
                let increments_the_released_record_counter =
                    for_body.iter().any(|for_body_statement| {
                        store_to_the_global_counter(
                            module,
                            function,
                            for_body_statement,
                            RELEASED_RECORD_COUNTER,
                        ) == Some(CounterStore::IncrementedByOne)
                    });
                if is_the_condition_of_a_for_loop
                    && is_a_load_of_the_global_variable(
                        module,
                        function,
                        right,
                        ALLOCATION_RECORD_COUNTER,
                    )
                    && increments_the_released_record_counter
                {
                    loops.push(ReleasedRecordPickingLoop {
                        function_name,
                        function,
                        enclosing_block,
                        position_in_the_enclosing_block,
                        loop_variable,
                        whole_body,
                        for_body,
                        continuing,
                    });
                }
            }
        }
    }
    loops
}

/// 扫描内核 `source` 里站点 29（已释放记录那张表满了、记判不了）走不到，推理逐步落成断言：
/// ① 已释放条数只归零与加一，加一只有一处，两个计数都不当指针交给别的函数；
/// ② 那一处在挑已释放记录的循环里，进循环之前同一个语句块里归零；循环变量每轮只在 continuing 里加一，条件是「循环变量 < 分配记录条数」，
///    循环体里不改分配记录条数（直接写与经调用写都没有）；站点 29 那一判在加一之前。所以走到第 k 轮（从 0 数）那一判时，
///    已释放条数 ≤ k ≤ 循环变量 < 分配记录条数；
/// ③ 分配记录条数只归零、写回之前读到的值、或在同一个函数顶层先判「≥ 分配记录表上限就返回」再加一（中间不经调用改它），所以它不超过那个上限；
/// ④ 分配记录表上限 ≤ 已释放记录表上限。
/// 合起来那一判的条件「已释放条数 ≥ 已释放记录表上限」在每个输入上都不成立：指这一判「抬过上限报判不了」的变异（上限抬过分配记录表、
/// 删掉这一判、`>=` 改 `>`）是等价变异，不进变异表；会改行为的那一条是把上限压到已释放条数之下（2048 那条流红）。
fn assert_site_twenty_nine_of_the_scan_kernel_is_unreachable(source: &str) {
    let module = wgpu::naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("扫描内核解析失败：\n{}", error.emit_to_string(source)));
    // ①
    let mut released_record_counter_stores: Vec<(&str, CounterStore)> = Vec::new();
    for (function_name, function) in every_function_of(&module) {
        for store in stores_to_the_global_counter_in(
            &module,
            function,
            &function.body,
            RELEASED_RECORD_COUNTER,
        ) {
            released_record_counter_stores.push((function_name, store));
        }
    }
    assert!(
        released_record_counter_stores
            .iter()
            .all(|(_, store)| matches!(
                store,
                CounterStore::SetToZero | CounterStore::IncrementedByOne
            )),
        "已释放条数只该归零与加一：{released_record_counter_stores:?}"
    );
    assert_eq!(
        released_record_counter_stores
            .iter()
            .filter(|(_, store)| *store == CounterStore::IncrementedByOne)
            .count(),
        1,
        "已释放条数加一只有一处：{released_record_counter_stores:?}"
    );
    for counter in [RELEASED_RECORD_COUNTER, ALLOCATION_RECORD_COUNTER] {
        let functions_handing_the_counter_away: Vec<&str> = every_function_of(&module)
            .into_iter()
            .filter(|(_, function)| {
                calls_in(&function.body).iter().any(|(_, arguments)| {
                    arguments.iter().any(|argument| {
                        is_the_global_variable(&module, function, *argument, counter)
                    })
                })
            })
            .map(|(function_name, _)| function_name)
            .collect();
        assert!(
            functions_handing_the_counter_away.is_empty(),
            "{counter} 被当指针交给别的函数（那边写它这里看不见）：{functions_handing_the_counter_away:?}"
        );
    }
    // ②
    let picking_loops = every_released_record_picking_loop(&module);
    assert_eq!(
        picking_loops.len(),
        1,
        "挑已释放记录的循环（条件「循环变量 < {ALLOCATION_RECORD_COUNTER}」、循环体顶层 {RELEASED_RECORD_COUNTER} 加一）恰好一个，找到 {} 个：{:?}",
        picking_loops.len(),
        picking_loops
            .iter()
            .map(|picking_loop| picking_loop.function_name)
            .collect::<Vec<_>>()
    );
    let picking_loop = &picking_loops[0];
    let picking_function = picking_loop.function;
    assert!(
        picking_loop
            .enclosing_block
            .iter()
            .take(picking_loop.position_in_the_enclosing_block)
            .any(|statement| {
                store_to_the_global_counter(
                    &module,
                    picking_function,
                    statement,
                    RELEASED_RECORD_COUNTER,
                ) == Some(CounterStore::SetToZero)
            }),
        "{}：进挑已释放记录的循环之前，同一个语句块里先把 {RELEASED_RECORD_COUNTER} 归零",
        picking_loop.function_name
    );
    assert_eq!(
        stores_to_the_local_variable_in(
            picking_function,
            picking_loop.continuing,
            picking_loop.loop_variable
        ),
        vec![CounterStore::IncrementedByOne],
        "{}：循环变量在 continuing 里每轮加一、只写这一次",
        picking_loop.function_name
    );
    assert!(
        stores_to_the_local_variable_in(
            picking_function,
            picking_loop.whole_body,
            picking_loop.loop_variable
        )
        .is_empty(),
        "{}：循环体里不写循环变量",
        picking_loop.function_name
    );
    let functions_storing_the_allocation_record_counter =
        functions_storing_the_global_counter(&module, ALLOCATION_RECORD_COUNTER);
    for block in [picking_loop.whole_body, picking_loop.continuing] {
        assert!(
            stores_to_the_global_counter_in(
                &module,
                picking_function,
                block,
                ALLOCATION_RECORD_COUNTER
            )
            .is_empty(),
            "{}：挑已释放记录的循环里不写 {ALLOCATION_RECORD_COUNTER}",
            picking_loop.function_name
        );
        assert!(
            calls_in(block).iter().all(|(callee, arguments)| {
                !functions_storing_the_allocation_record_counter.contains(callee)
                    && arguments.iter().all(|argument| {
                        !matches!(
                            picking_function.expressions[*argument],
                            Expression::LocalVariable(handed) if handed == picking_loop.loop_variable
                        )
                    })
            }),
            "{}：挑已释放记录的循环里不调用写 {ALLOCATION_RECORD_COUNTER} 的函数，也不把循环变量交出去",
            picking_loop.function_name
        );
    }
    let position_of_site_twenty_nine = picking_loop.for_body.iter().position(|statement| {
        is_a_guard_returning_once_the_counter_reaches_the_capacity(
            &module,
            picking_function,
            statement,
            RELEASED_RECORD_COUNTER,
            RELEASED_RECORD_TABLE_CAPACITY,
        )
    });
    let position_of_the_released_record_increment =
        picking_loop.for_body.iter().position(|statement| {
            store_to_the_global_counter(
                &module,
                picking_function,
                statement,
                RELEASED_RECORD_COUNTER,
            ) == Some(CounterStore::IncrementedByOne)
        });
    assert!(
        matches!(
            (position_of_site_twenty_nine, position_of_the_released_record_increment),
            (Some(check), Some(increment)) if check < increment
        ),
        "{}：循环体顶层先有站点 29 那一判（{RELEASED_RECORD_COUNTER} >= {RELEASED_RECORD_TABLE_CAPACITY} 就返回）、再加一；\
         那一判在第 {position_of_site_twenty_nine:?} 句，加一在第 {position_of_the_released_record_increment:?} 句",
        picking_loop.function_name
    );
    // ③
    for (function_name, function) in every_function_of(&module) {
        let stores = stores_to_the_global_counter_in(
            &module,
            function,
            &function.body,
            ALLOCATION_RECORD_COUNTER,
        );
        assert!(
            stores.iter().all(|store| *store != CounterStore::WrittenOtherwise),
            "{function_name}：{ALLOCATION_RECORD_COUNTER} 只该归零、写回之前读到的值、或加一：{stores:?}"
        );
        let increments_anywhere = stores
            .iter()
            .filter(|store| **store == CounterStore::IncrementedByOne)
            .count();
        let increments_on_the_top_level: Vec<usize> = function
            .body
            .iter()
            .enumerate()
            .filter(|(_, statement)| {
                store_to_the_global_counter(&module, function, statement, ALLOCATION_RECORD_COUNTER)
                    == Some(CounterStore::IncrementedByOne)
            })
            .map(|(position, _)| position)
            .collect();
        assert_eq!(
            increments_on_the_top_level.len(),
            increments_anywhere,
            "{function_name}：{ALLOCATION_RECORD_COUNTER} 加一只写在函数体顶层"
        );
        for position_of_the_increment in increments_on_the_top_level {
            let position_of_the_guard = function.body[..position_of_the_increment]
                .iter()
                .rposition(|statement| {
                    is_a_guard_returning_once_the_counter_reaches_the_capacity(
                        &module,
                        function,
                        statement,
                        ALLOCATION_RECORD_COUNTER,
                        ALLOCATION_RECORD_TABLE_CAPACITY,
                    )
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{function_name}：{ALLOCATION_RECORD_COUNTER} 加一之前，函数体顶层先判「>= {ALLOCATION_RECORD_TABLE_CAPACITY} 就返回」"
                    )
                });
            let between_the_guard_and_the_increment =
                &function.body[position_of_the_guard + 1..position_of_the_increment];
            assert!(
                stores_to_the_global_counter_in(
                    &module,
                    function,
                    between_the_guard_and_the_increment,
                    ALLOCATION_RECORD_COUNTER
                )
                .is_empty()
                    && calls_in(between_the_guard_and_the_increment)
                        .iter()
                        .all(|(callee, _)| {
                            !functions_storing_the_allocation_record_counter.contains(callee)
                        }),
                "{function_name}：判过上限到加一之间不改 {ALLOCATION_RECORD_COUNTER}（直接写与经调用写都没有）"
            );
        }
    }
    // ④
    let allocation_record_table_capacity =
        value_of_the_u32_constant(&module, ALLOCATION_RECORD_TABLE_CAPACITY);
    let released_record_table_capacity =
        value_of_the_u32_constant(&module, RELEASED_RECORD_TABLE_CAPACITY);
    assert!(
        allocation_record_table_capacity <= released_record_table_capacity,
        "{ALLOCATION_RECORD_TABLE_CAPACITY} = {allocation_record_table_capacity} 不该大于 {RELEASED_RECORD_TABLE_CAPACITY} = {released_record_table_capacity}：\
         已释放记录是从分配记录表里挑的，已释放表窄了站点 29 就走得到"
    );
}

/// 扫描内核挑最新根已释放记录的那张表（`C_MAX_RELEASED`）装得下每一条已释放记录，站点 29 走不到：推理在
/// [`assert_site_twenty_nine_of_the_scan_kernel_is_unreachable`]，判的是库编进去的那一份扫描内核源码，只用 naga 解析、不用卡。
#[test]
fn site_twenty_nine_of_the_scan_kernel_is_unreachable_because_released_records_are_picked_from_the_allocation_record_table(
) {
    assert_site_twenty_nine_of_the_scan_kernel_is_unreachable(&checker_scan_kernel_source());
}
