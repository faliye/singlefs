//! checker 档模块：crash、crash_state_enumeration、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器按条数开的各张表在容量边界上：恰好装满的判得了、多一条的交 CPU，抬上限之前的边界上那几条流抬之后照样判得了。
//! 分配记录那两张表（恢复内核读一版分配记录树的 `MAX_ALLOCATION_RECORDS`，`crash_judge_common.wgsl`；扫描内核收一条根的分配记录的
//! `C_MAX_ALLOC_RECORDS`，`crash_judge_checker_common.wgsl`，最新根的已释放记录表 `C_MAX_RELEASED` 与它同宽）：第 59 批 L6 从 1024 抬到 2048
//! （判决 `research/prompts/m3-batch58-code-r1-main-verification.md` 第三节 L6、第五节用户 2026-10-10 定 ②），harness 场景交出来的镜像搬进 checker 档之后
//! 再抬到 4096（容器树旋钮压小的随机历史第 48、52 步起分配记录多于 2048 条，两个内核判不了；用户 2026-10-13 定「能上 GPU 的上 GPU」）。
//! 造法：新池新建文件之后逐次顺序写第一个文件，单元数长到 200 个、缩回 1 个为一轮（[`DATA_UNITS_OF_A_GROW_AND_SHRINK_ROUND`]），轮四遍或八遍，
//! 再长到若干个，最后被判的那次缩回 1 个。每个新写的单元每块盘一条分配记录，缩回去的单元那条记录照旧留着、改成已释放
//! （这段历史回退下界不抬，一条都不回收）；写表只录被判的那次发布（之前的录制流施加成起点镜像）。被判那一版的分配记录恰好 2048、2050、4098 条
//! （轮四遍再长到 56、57 个，轮八遍再长到 167 个；CPU 一侧现数的：四遍之后 1880 条、八遍之后 3712 条，每多长一个单元多两条）。
//! 两块盘互为镜像，每个落点每块盘一条记录，条数恒为偶数：上限之上最近造得出来的是加 2。
//! 2048、2050 那两条流每个状态 GPU 都判得了、对拍字（`crash_judge_gpu::CrossCheckWords`）与 CPU 逐状态相同（2050 条那一条钉「抬到 4096 判得到」：
//! 两张表改回 2048 时它红）；4098 那条流：两遍恢复有一遍落到被判那一版的状态，GPU 恢复内核记判不了（条目过多）；被判那次的根槽写落了的状态，
//! 池级 checker 记判不了（条目过多那一类）；别的状态 GPU 判得了、对拍字与 CPU 相同。这种不回收的流到 4096 条上下时，走读内核按单元记的几张表
//! （引用集合、位置项校验和、I-3.5 落点表各 4096 条、走过的单元 2048 个、数据单元 1024 个，CPU 一侧在全落的镜像上数到 4098、4098、4098、2049、1777）
//! 与分配记录一同到顶，恰好 4096 条那一形 GPU 判不全，不立；随机历史那几张镜像回收过，那几张表各至多八百来条。
//! 已释放那张表：扫描内核先把最新根的分配记录收进分配记录那张表，再从里面挑已释放的，已释放的条数到不了分配记录的条数（这一版自己的单元总有
//! 未释放的记录），所以已释放多一条那一形走不到；`C_MAX_RELEASED` 直接取 `C_MAX_ALLOC_RECORDS`，站点 29 在每个输入上都走不到：这条推理在扫描内核的
//! naga 中间表示上逐步落成断言（`site_twenty_nine_of_the_scan_kernel_is_unreachable_because_released_records_are_picked_from_the_allocation_record_table`，
//! 不用卡），指站点 29「抬过上限报判不了」的变异是等价变异，留档在那一条。三条流上 CPU 判器每个状态照旧判、都不红（流是合法的）。
//! journal 记录表 `MAX_RECORDS`（2048 条，两个内核都按它收环里的记录）这几条流 865、866、1780 条（CPU 一侧数的盘 0 上的记录写）。
//! 同样造法、同样几半的还有实例表行那两张（扫描内核读一条根自己实例表全部行的 `C_R_MAX_ROWS`，`crash_judge_checker_common.wgsl`，与恢复内核读
//! 同一种表的 `MAX_ROWS` 同宽）：256 → 512（取号之后崩溃 369 次再挂载写 370 行那条流）→ 24576（取号之后崩溃 65 × 369 次再挂载十二次那个场景，
//! 最新根的实例表 23997 行、66 片）。取号之后崩溃 511 / 512 / 24575 / 24576 次再可写挂载，被录那次挂载写出的每条根自己那张实例表
//! 512 / 513 / 24576 / 24577 行。前三条流每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同（24576 行那一条同时钉走读内核最新根实例表的行
//! `C_MAX_INSTANCE_ROWS` 从 1024 抬到 32768 与它的散列索引）；24577 那条流被判那条根落了的状态扫描内核在站点 32 记判不了、恢复落到那次挂载的根的状态
//! 恢复内核记条目过多，别的状态判得了、对拍字与 CPU 相同。
//! 主走读 inode → 对象出生代 `C_MAX_INODE_BIRTHS`（1024 → 16384，站点 13）、扫描内核一版的 inode 记录 `D_MAX_INODE_RECORDS`（1024 → 16384，站点 49）
//! 与 I-9.29 跨版本那张表 `D_MAX_OBJECT_BIRTHS`（2048 → 16384，站点 59）：一次建 2500 个空 inode、再建一个的那条流（各版合起来 2502 个号），
//! 每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同；三张表各改回原来的条数时它红。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{
    build_pool, file_content, geometry, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion,
};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};

/// 这份测试文件的仓内路径：崩溃点重放取样过库的设点文件。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_judge_gpu_tables_at_their_capacity.rs";
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
use singlefs_core::transaction::{
    acquire_instance, publish_new_inodes, PoolWriter, TransactionOutput,
};
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
/// 两张表此刻的上限：恢复内核一版的分配记录、扫描内核一条根的分配记录（已释放记录那张与它同宽）。
const RECORDS_THE_GPU_TABLES_HOLD: usize = 4096;
/// 两张表抬到 4096 之前装几条：恰好 2048 条那条流照旧判得了，2050 条那条流多于它（两张表改回 2048 时它红）。
const RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE: usize = 2048;
/// 第 59 批抬之前扫描内核已释放记录那张表装几条：2048 那条流上已释放的条数要多于它，改回去才红。
const RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE: usize = 1024;
/// 上限之上最近造得出来的条数：两块盘互为镜像，一个落点每块盘一条记录，条数恒为偶数。
const RECORDS_JUST_PAST_THE_GPU_TABLES: usize = RECORDS_THE_GPU_TABLES_HOLD + 2;
/// 抬到 4096 之前的上限之上最近造得出来的条数。
const RECORDS_JUST_PAST_THE_GPU_TABLES_BEFORE_THE_RAISE: usize =
    RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE + 2;
/// 恢复内核记判不了时结论第 1 字的原因：条目过多（`crash_judge_common.wgsl` 的 `UNDECIDABLE_TOO_MANY_ENTRIES`，分配记录那张表满了就记它）。
const RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES: u32 = 3;
/// 池级 checker 判不了的原因字的低 8 位：条目过多那一类（高位是站点号）。
const POOL_CHECKER_UNDECIDABLE_REASON_MASK: u32 = 0xFF;
/// 一轮顺序写：长到 200 个单元、缩回 1 个（每长一次多 400 多条记录，缩回去的那些改成已释放、照旧留着）。
const DATA_UNITS_OF_A_GROW_AND_SHRINK_ROUND: [usize; 2] = [200, 1];
/// 2048 / 2050 那两条流被判那次之前轮几遍（之后 1880 条记录，CPU 一侧现数的）。
const GROW_AND_SHRINK_ROUNDS_BEFORE_THE_OLD_CAPACITY: usize = 4;
/// 4098 那条流被判那次之前轮几遍（之后 3712 条记录，同上）。
const GROW_AND_SHRINK_ROUNDS_PAST_THE_CAPACITY: usize = 8;
/// 轮四遍之后再长到的单元数：被判那一版的分配记录恰好 [`RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE`] 条（每个单元每块盘一条，多一个单元多两条）。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_THE_OLD_CAPACITY: usize = 56;
/// 2050 那一形多长一个单元。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_THE_OLD_CAPACITY: usize =
    DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_THE_OLD_CAPACITY + 1;
/// 轮八遍之后再长到的单元数：被判那一版的分配记录 [`RECORDS_JUST_PAST_THE_GPU_TABLES`] 条（CPU 一侧现数的：长到 167 个、再缩回 1 个之后 4098 条）。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_THE_CAPACITY: usize = 167;
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

/// 新池新建文件（txg 3），照 [`DATA_UNITS_OF_A_GROW_AND_SHRINK_ROUND`] 顺序写 `grow_and_shrink_rounds` 遍，再长到 `data_units_before_the_judged_publish` 个单元，
/// 被判的是最后那次缩回 [`DATA_UNITS_OF_THE_JUDGED_PUBLISH`] 个单元的发布：起点镜像是 mkfs 到它之前的整段录制流施加出来的那一份，写表只录它。
fn record_the_flow(
    tag: &str,
    grow_and_shrink_rounds: usize,
    data_units_before_the_judged_publish: usize,
) -> RecordedFlow {
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
    // 迭代上界：2 × grow_and_shrink_rounds + 1 次发布。
    for data_units in std::iter::repeat_n(
        DATA_UNITS_OF_A_GROW_AND_SHRINK_ROUND,
        grow_and_shrink_rounds,
    )
    .flatten()
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
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
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
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count, "表与枚举的状态数相同");
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)?;
    let judged_txg = flow.judged_txg;
    let mut cpu: Vec<CpuJudgementOfAState> = Vec::new();
    let item_name = format!("tables_at_their_capacity::{test_name}::cpu_side");
    let judged_on_the_cpu = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
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
    )
    .judged()?;
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

/// 每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同（`cpu_cross_check_words` 与 `gpu` 同序）；不是的状态列出来（前几个）。
fn assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
    flow_name: &str,
    gpu: &[[u32; VERDICT_WORDS]],
    cpu_cross_check_words: &[CrossCheckWords],
) {
    let mismatches: Vec<String> = cpu_cross_check_words
        .iter()
        .zip(gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_words, gpu_words))| {
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (gpu_could_not_judge(gpu_words) || gpu_cross_check_words != *cpu_words).then(|| {
                format!(
                    "状态 {ordinal}：GPU 判不了 {}（恢复第 1 字 {:#x}、池级 checker 原因字 {:#x}）；GPU {gpu_cross_check_words:?}；CPU {cpu_words:?}",
                    gpu_could_not_judge(gpu_words),
                    gpu_words[1],
                    gpu_words[POOL_CHECKER_UNDECIDABLE_WORD],
                )
            })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "流 {flow_name}：{} 个状态 GPU 判不了或与 CPU 不一致（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 被判那一版的分配记录恰好 `judged_records` 条的那条流（[`record_the_flow`] 轮 `grow_and_shrink_rounds` 遍、再长到 `data_units_before_the_judged_publish` 个单元）：
/// 取卡之前在 CPU 上核条数与已释放的条数（多于 `released_more_than`），再在卡上逐状态判：每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同；
/// 要有被判那次的根槽写落了、恢复落到那一版的状态（两个内核都读那一版的分配记录）。
fn assert_the_gpu_judge_judges_a_version_of_these_many_allocation_records_like_the_cpu_judge(
    test_name: &str,
    flow_name: &str,
    grow_and_shrink_rounds: usize,
    data_units_before_the_judged_publish: usize,
    judged_records: usize,
    released_more_than: usize,
) {
    let flow = record_the_flow(
        flow_name,
        grow_and_shrink_rounds,
        data_units_before_the_judged_publish,
    );
    assert_the_allocation_records_of_the_versions(flow_name, &flow, judged_records);
    let judged = flow.allocation_record_counts_of_the_judged_version();
    assert!(
        judged.released > released_more_than && judged.released <= RECORDS_THE_GPU_TABLES_HOLD,
        "流 {flow_name}：被判那一版已释放 {} 条，要多于 {released_more_than}、不多于 {RECORDS_THE_GPU_TABLES_HOLD}",
        judged.released
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, test_name, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(flow_name, &cpu);
    assert!(
        cpu.iter().any(|judgement| {
            judgement.a_recovery_lands_on_the_judged_version && judgement.judged_root_persisted
        }),
        "流 {flow_name}：要有被判那次的根槽写落了、恢复落到那一版的状态（恢复内核读那一版 {judged_records} 条、扫描内核挑它已释放的 {} 条）",
        judged.released
    );
    let cpu_cross_check_words: Vec<CrossCheckWords> = cpu
        .iter()
        .map(|judgement| judgement.cross_check_words)
        .collect();
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
        flow_name,
        &gpu,
        &cpu_cross_check_words,
    );
}

/// 被判那一版的分配记录恰好 2048 条、其中已释放 2010 条（多于第 59 批抬之前已释放那张表的 1024 条）：每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同。
/// 已释放那张表改回 1024 时，那一版的根槽写落了的状态扫描内核在站点 29 记判不了，这条红。
#[test]
fn the_gpu_judge_judges_a_version_of_exactly_two_thousand_and_forty_eight_allocation_records_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_a_version_of_these_many_allocation_records_like_the_cpu_judge(
        "the_gpu_judge_judges_a_version_of_exactly_two_thousand_and_forty_eight_allocation_records_like_the_cpu_judge",
        "allocation-records-at-the-old-capacity-of-the-gpu-tables",
        GROW_AND_SHRINK_ROUNDS_BEFORE_THE_OLD_CAPACITY,
        DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_THE_OLD_CAPACITY,
        RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE,
        RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE,
    );
}

/// 被判那一版的分配记录 2050 条（多于两张表抬到 4096 之前的 2048 条）：每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同。
/// 恢复内核那张表改回 2048 时，有一遍恢复落到那一版的状态恢复内核记条目过多；扫描内核那张改回 2048 时，那一版的根槽写落了的状态在站点 20 记判不了：
/// 两样都让这条红（容器树旋钮压小的随机历史第 48 步起分配记录多于 2048 条，就是这两处判不了）。
#[test]
fn the_gpu_judge_judges_a_version_of_two_thousand_and_fifty_allocation_records_past_the_old_capacity_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_a_version_of_these_many_allocation_records_like_the_cpu_judge(
        "the_gpu_judge_judges_a_version_of_two_thousand_and_fifty_allocation_records_past_the_old_capacity_like_the_cpu_judge",
        "allocation-records-just-past-the-old-capacity-of-the-gpu-tables",
        GROW_AND_SHRINK_ROUNDS_BEFORE_THE_OLD_CAPACITY,
        DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_THE_OLD_CAPACITY,
        RECORDS_JUST_PAST_THE_GPU_TABLES_BEFORE_THE_RAISE,
        RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE,
    );
}

/// 被判那一版的分配记录 4098 条（4096 之上最近造得出来的条数）：两遍恢复有一遍落到那一版的状态，GPU 恢复内核记判不了（条目过多）；
/// 那一版的根槽写落了的状态，池级 checker 记判不了、原因是条目过多那一类（站点不钉：这条不回收的流上走读内核按单元记的几张表
/// 与扫描内核的分配记录表一同到顶，CPU 一侧在全落的镜像上数到引用集合 4098 条、走过的单元 2049 个、数据单元 1777 个，哪一张先满看走读的次序）；
/// 别的状态 GPU 判得了、对拍字与 CPU 相同。CPU 判器每个状态照旧判、都不红。
#[test]
fn the_gpu_judge_leaves_a_version_of_four_thousand_and_ninety_eight_allocation_records_to_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_leaves_a_version_of_four_thousand_and_ninety_eight_allocation_records_to_the_cpu_judge";
    const FLOW_NAME: &str = "allocation-records-just-past-the-capacity-of-the-gpu-tables";
    let flow = record_the_flow(
        FLOW_NAME,
        GROW_AND_SHRINK_ROUNDS_PAST_THE_CAPACITY,
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
                "状态 {ordinal}：恢复落到 4098 条那一版，GPU 恢复内核该记判不了（条目过多 {RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES}），实际判不了 {recovery_could_not_judge}、第 1 字 {:#x}",
                gpu_words[1]
            ));
        }
        if cpu_judgement.judged_root_persisted
            && checker_reason & POOL_CHECKER_UNDECIDABLE_REASON_MASK
                != RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES
        {
            problems.push(format!(
                "状态 {ordinal}：4098 条那一版的根槽写落了，池级 checker 该记判不了、原因是条目过多（{RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES}），实际原因字 {checker_reason:#x}"
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
        "流 {FLOW_NAME}：要有 GPU 判不了、交 CPU 判的状态（恢复落到 4098 条那一版，或它的根槽写落了）"
    );
}

// ───────── 扫描内核读一条根自己实例表全部行的那张表：恰好装满与多一行 ─────────

/// 扫描内核读一条根自己实例表全部行的那张表（`crash_judge_checker_common.wgsl` 的 `C_R_MAX_ROWS`）此刻装几行：与恢复内核读同一种表的
/// `MAX_ROWS`（`crash_judge_common.wgsl`）同宽。
const INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD: usize = 24576;
/// 两张表抬到 24576 之前装几行：恰好 512 行那条流照旧判得了，513 行那条流多于它。
const INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE: usize = 512;
/// 第一次抬之前扫描内核那张表装几行：512 行那条流多于它。
const INSTANCE_TABLE_ROWS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE: usize = 256;
/// 走读内核最新根实例表的行那一张（`C_MAX_INSTANCE_ROWS`）抬到 32768 之前装几行：24576 行那条流多于它。
const INSTANCE_ROWS_THE_WALK_KERNEL_HELD_BEFORE_THE_RAISE: usize = 1024;
const _: () = assert!(
    INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE
        > INSTANCE_TABLE_ROWS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE
        && INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD
            > INSTANCE_ROWS_THE_WALK_KERNEL_HELD_BEFORE_THE_RAISE,
    "每条「判得了」的流的行数要多于那张表抬之前的行数"
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
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
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
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count, "表与枚举的状态数相同");
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)?;
    let roots_of_the_recorded_mount: HashSet<(InstanceGeneration, CheckpointTxg)> = flow
        .roots_of_the_recorded_mount
        .iter()
        .map(|(instance, txg, _)| (*instance, *txg))
        .collect();
    let mut cpu: Vec<CpuJudgementOfARowPublishState> = Vec::new();
    let item_name = format!("tables_at_their_capacity::{test_name}::row_publish_cpu_side");
    let judged_on_the_cpu = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
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
    )
    .judged()?;
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

/// 取号之后崩溃 `rows - 1` 次再挂载：被录那次挂载写出的每条根自己那张实例表恰好 `rows` 行（取卡之前在 CPU 上核）。
/// 每个状态 GPU 两个内核都判得了，对拍字与 CPU 逐状态相同；要有被判那条根落了的状态（扫描内核读那张 `rows` 行的表）。
fn assert_the_gpu_judge_judges_roots_whose_own_instance_table_has_these_many_rows_like_the_cpu_judge(
    test_name: &str,
    flow_name: &str,
    rows: usize,
) {
    let flow = record_the_row_publish_flow(rows - 1);
    assert_every_root_of_the_recorded_mount_has_an_instance_table_of(flow_name, &flow, rows);
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_row_publish_state_on_both_sides(cards, test_name, &flow)
    else {
        return;
    };
    assert_the_cpu_judge_finds_every_row_publish_state_green(flow_name, &cpu);
    assert!(
        cpu.iter().any(|judgement| judgement.judged_root_persisted),
        "流 {flow_name}：要有被判那次的根槽写落了的状态（扫描内核读那条根 {rows} 行的表）"
    );
    let cpu_cross_check_words: Vec<CrossCheckWords> = cpu
        .iter()
        .map(|judgement| judgement.cross_check_words)
        .collect();
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
        flow_name,
        &gpu,
        &cpu_cross_check_words,
    );
}

/// 取号之后崩溃 511 次再挂载：被录那次挂载写出的每条根自己那张实例表恰好 512 行（多于第一次抬之前扫描内核那张表的 256 行）。
/// 每个状态 GPU 两个内核都判得了，对拍字与 CPU 逐状态相同。扫描内核那张表改回 256 行时，被判那条根落了的状态在站点 32 记判不了，这条红。
#[test]
fn the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_five_hundred_and_twelve_rows_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_roots_whose_own_instance_table_has_these_many_rows_like_the_cpu_judge(
        "the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_five_hundred_and_twelve_rows_like_the_cpu_judge",
        "instance-table-rows-at-the-old-capacity-of-the-gpu-tables",
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE,
    );
}

/// 取号之后崩溃 512 次再挂载：每条根自己那张实例表 513 行（多于两张表抬到 24576 之前的 512 行）。每个状态 GPU 两个内核都判得了，
/// 对拍字与 CPU 逐状态相同。扫描内核那张表改回 512 行时被判那条根落了的状态在站点 32 记判不了，恢复内核那张改回 512 行时
/// 恢复落到那次挂载的根的状态记条目过多：两样都让这条红。
#[test]
fn the_gpu_judge_judges_roots_whose_own_instance_table_has_five_hundred_and_thirteen_rows_past_the_old_capacity_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_roots_whose_own_instance_table_has_these_many_rows_like_the_cpu_judge(
        "the_gpu_judge_judges_roots_whose_own_instance_table_has_five_hundred_and_thirteen_rows_past_the_old_capacity_like_the_cpu_judge",
        "instance-table-rows-just-past-the-old-capacity-of-the-gpu-tables",
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE + 1,
    );
}

/// 取号之后崩溃 24575 次再挂载：每条根自己那张实例表恰好 24576 行（67 片）。每个状态 GPU 两个内核都判得了，对拍字与 CPU 逐状态相同：
/// 扫描内核、恢复内核那两张表恰好装满；走读内核最新根实例表的行那一张（`C_MAX_INSTANCE_ROWS`）多于抬之前的 1024 行，改回 1024 时
/// 走读那条根的状态在站点 8 记判不了，这条红（这一张的散列索引在这条流上建起来：几万行时按桶表找）。
#[test]
fn the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_twenty_four_thousand_five_hundred_and_seventy_six_rows_like_the_cpu_judge(
) {
    assert_the_gpu_judge_judges_roots_whose_own_instance_table_has_these_many_rows_like_the_cpu_judge(
        "the_gpu_judge_judges_roots_whose_own_instance_table_has_exactly_twenty_four_thousand_five_hundred_and_seventy_six_rows_like_the_cpu_judge",
        "instance-table-rows-at-the-capacity-of-the-gpu-tables",
        INSTANCE_TABLE_ROWS_THE_GPU_TABLES_HOLD,
    );
}

/// 取号之后崩溃 24576 次再挂载：被录那次挂载写出的每条根自己那张实例表 24577 行（多新上限一行）。被判那条根落了的状态，
/// 扫描内核读到第 24577 行时在站点 32 记判不了（[`SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL`]；走读内核那一张 32768 行装得下，扫描内核跑得到）；
/// 看 journal 那一遍恢复落到那次挂载写的根的状态，恢复内核读那一版的实例表读到第 24577 行同样记判不了（条目过多 [`RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES`]，两个内核的表同宽）。
/// 两个内核判不了的只许是这两个原因；两个内核都判得了的状态对拍字与 CPU 相同；CPU 判器每个状态照旧判、都不红。
#[test]
fn the_gpu_judge_leaves_roots_whose_own_instance_table_has_twenty_four_thousand_five_hundred_and_seventy_seven_rows_to_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_leaves_roots_whose_own_instance_table_has_twenty_four_thousand_five_hundred_and_seventy_seven_rows_to_the_cpu_judge";
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
                "状态 {ordinal}：看 journal 那一遍恢复落到 24577 行那张表的根，GPU 恢复内核该记判不了（条目过多），实际判得了、第 1 字 {:#x}",
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
                "状态 {ordinal}：24577 行那张表的被判那条根槽写落了，扫描内核该在站点 32 记判不了（{SCAN_UNDECIDABLE_INSTANCE_TABLE_ROW_TABLE_FULL:#x}），实际原因字 {checker_reason:#x}"
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

// ───────── 主走读 inode → 对象出生代、一版的 inode 记录、I-9.29 跨版本那张表：抬上限之后照样判得了 ─────────

/// 主走读 inode → 对象出生代（`C_MAX_INODE_BIRTHS`，站点 13）与扫描内核一版的 inode 记录（`D_MAX_INODE_RECORDS`，站点 49）抬到 16384 之前装几条。
const INODES_THE_TWO_KERNELS_HELD_BEFORE_THE_RAISE: u64 = 1024;
/// I-9.29 跨版本那张表（`D_MAX_OBJECT_BIRTHS`，站点 59）抬到 16384 之前装几个 (inode 树, 号)。
const OBJECT_BIRTHS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE: u64 = 2048;
/// 那条流第一次发布建几个空 inode：连第一个文件，之后每一版的号都多于上面两个数（CPU 一侧在全落的镜像上数到各版合起来 2502 个号）。
const NEW_INODES_PAST_THE_OLD_CAPACITIES: u64 = 2500;
/// 被判那次发布建几个空 inode。
const NEW_INODES_OF_THE_JUDGED_PUBLISH: u64 = 1;
const _: () = assert!(
    NEW_INODES_PAST_THE_OLD_CAPACITIES > INODES_THE_TWO_KERNELS_HELD_BEFORE_THE_RAISE
        && NEW_INODES_PAST_THE_OLD_CAPACITIES > OBJECT_BIRTHS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE,
    "建的 inode 要多于那三张表抬之前的条数"
);

/// 在池上发一次建 `count` 个空 inode（号从记账水位起、实例 1），记下这一版（第一个文件不变）与它的分配记录条数。
fn publish_new_inodes_and_note_the_version(
    pool: &mut BuiltPool,
    count: u64,
    versions: &mut Vec<PublishedVersion>,
    counts: &mut Vec<(CheckpointTxg, AllocationRecordCounts)>,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        count,
        FIXED_WRITE_TIME_SECONDS,
        InstanceGeneration(1),
    )
    .expect("建空 inode");
    counts.push((
        output.root.checkpoint_txg,
        allocation_record_counts_of(&output),
    ));
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: output.root.checkpoint_txg,
        content: file_content(),
    });
    pool.output = output;
}

/// 新池新建文件（txg 3）→ 一次建 [`NEW_INODES_PAST_THE_OLD_CAPACITIES`] 个空 inode（txg 4）→ 被判的是再建 [`NEW_INODES_OF_THE_JUDGED_PUBLISH`] 个的那一次（txg 5）：
/// 起点镜像是 mkfs 到它之前的整段录制流施加出来的那一份，写表只录它。每一版第一个文件都不变。
fn record_the_inode_flow(tag: &str) -> RecordedFlow {
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
    publish_new_inodes_and_note_the_version(
        &mut pool,
        NEW_INODES_PAST_THE_OLD_CAPACITIES,
        &mut versions,
        &mut counts,
    );
    let operations_before_the_judged_publish = pool.stream.operation_count();
    publish_new_inodes_and_note_the_version(
        &mut pool,
        NEW_INODES_OF_THE_JUDGED_PUBLISH,
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

/// 一次建 2500 个空 inode、再建一个：每一版两千五百来个号，多于主走读 inode → 对象出生代与一版 inode 记录那两张表抬之前的 1024 条、
/// I-9.29 跨版本那张表抬之前的 2048 个。每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同；三张表里有一张改回原来的条数，
/// 走读或扫描内核在站点 13、49、59 记判不了，这条红（几张表的散列索引在这条流上建起来：两千多条时按桶表找）。
#[test]
fn the_gpu_judge_judges_versions_of_two_thousand_five_hundred_inodes_past_the_old_capacities_like_the_cpu_judge(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_judges_versions_of_two_thousand_five_hundred_inodes_past_the_old_capacities_like_the_cpu_judge";
    const FLOW_NAME: &str = "inodes-past-the-old-capacities-of-the-gpu-tables";
    let flow = record_the_inode_flow(FLOW_NAME);
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(FLOW_NAME, &cpu);
    let cpu_cross_check_words: Vec<CrossCheckWords> = cpu
        .iter()
        .map(|judgement| judgement.cross_check_words)
        .collect();
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
        FLOW_NAME,
        &gpu,
        &cpu_cross_check_words,
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
