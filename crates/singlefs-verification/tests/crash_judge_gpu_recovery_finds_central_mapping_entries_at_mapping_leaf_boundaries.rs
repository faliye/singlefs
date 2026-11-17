//! 验证包模块：crash、crash_state_enumeration、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 恢复内核查中央映射（`crash_judge_common.wgsl` 的 `mapping_entry_holding_the_key`：映射读通时全表按 key 严格递增，先在叶之间二分、
//! 比每片的末条，再在那片叶里二分）落在映射叶边界上的那几把 key：长文件那条流（新池新建文件之后一次顺序写
//! [`DATA_UNITS_OF_THE_LONG_FILE`] 个单元，中央映射几片叶），映射里排在全表第一条、某片叶的末条与紧接着那片叶的首条的数据单元，
//! 两条位置提示都改指别的槽、重封到根：恢复读它们都要经映射回退，查映射要查到片边界上那几条条目。每个状态 GPU 恢复内核判得了、
//! 恢复那一半与 CPU 逐状态相同，有状态恢复落到被判那一版并读出文件。池级 checker 两个内核不在这条流的射程里（只比恢复内核写的那一半）。
//! 判别力：叶之间二分比每片的首条（片末那条落到下一片、查不到）、尾段那一段比反了方向，这两条红（`crates/mutations.tsv` 里本文件那几行）。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
//! 录流、改坏单元写并一路重封到根、两边各判一遍那几个函数同 `crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs` 那几个函数的写法。
#![cfg(feature = "gpu")]

mod common;

use common::{
    build_pool, file_content, geometry,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::pointer::{DataPointer, LocationEntry};
use singlefs_core::records::mapping_key_sort_key;
use singlefs_core::transaction::TransactionOutput;
use singlefs_core::unit::{data_unit_payload_capacity, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE};
use singlefs_format::{JOURNAL_HEADER_BYTES, JOURNAL_NAMED_ENTRY_BYTES, MAPPING_KEY_BYTES};
use singlefs_verification::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion,
};
use singlefs_verification::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_verification::crash_judge_gpu::{
    cross_check_words_of_cpu_judgement, cross_check_words_of_gpu_verdict, verdict_bits,
    CrossCheckWords, LandedRootWords, RECOVERY_OUTCOME_CODE_FILE_READ, VERDICT_WORDS,
};
use singlefs_verification::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_verification::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_verification::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run, GpuCardPoolStartedAtTheFirstLease,
};
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_verification::segments::StepKind;
use std::collections::{BTreeMap, BTreeSet};

/// 这份测试文件的仓内路径：CPU 那一侧取样过库的设点文件。
const THIS_FILE: &str = "crates/singlefs-verification/tests/crash_judge_gpu_recovery_finds_central_mapping_entries_at_mapping_leaf_boundaries.rs";
/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;
/// 长文件那次顺序写的单元数：一个单元一条映射条目，一片映射叶装不下（产品容量一片 294 条），映射叶几片、片边界两侧都是数据单元的条目。
const DATA_UNITS_OF_THE_LONG_FILE: usize = 900;
/// 至少几片映射叶：要有一处两片叶之间的边界。
const MAPPING_LEAVES_AT_THE_LEAST: usize = 2;

/// 一条流：起点镜像（被判那次之前的录制流施加上去）、被判那次的写表与段、模型认的每一版与被判那一版的 txg。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    judged_txg: CheckpointTxg,
}

/// 被判那次的写从 `operations_before_the_judged_publish` 起：之前的录制流施加成起点镜像，之后的切成写表与段。
fn recorded_flow_from_the_judged_publish_on(
    pool: &BuiltPool,
    operations_before_the_judged_publish: usize,
    versions: Vec<PublishedVersion>,
) -> RecordedFlow {
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
    }
}

/// 长文件的内容：`data_units` 个满载的数据单元，字节随位置换。
fn content_of_the_long_file(data_units: usize) -> Vec<u8> {
    (0..data_units * data_unit_payload_capacity())
        .map(|index| u8::try_from((index * 7 + index / 251) % 253).expect("小于 256"))
        .collect()
}

/// 新池新建文件（txg 3），被判的是接着那一次顺序写（txg 4，[`DATA_UNITS_OF_THE_LONG_FILE`] 个单元）；连同那次顺序写的发布结果（数据指针与映射 key）交回。
fn record_the_long_file_flow(tag: &str) -> (RecordedFlow, TransactionOutput) {
    let mut pool = build_pool(tag);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let operations_before_the_judged_publish = pool.stream.operation_count();
    let content = content_of_the_long_file(DATA_UNITS_OF_THE_LONG_FILE);
    let output =
        sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    assert_eq!(
        output.data_pointers.len(),
        DATA_UNITS_OF_THE_LONG_FILE,
        "被判那一版第一个文件的单元数"
    );
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: output.root.checkpoint_txg,
        content,
    });
    let flow = recorded_flow_from_the_judged_publish_on(
        &pool,
        operations_before_the_judged_publish,
        versions,
    );
    (flow, output)
}

/// 一个状态 CPU 那一侧：对拍字、红了的项、有没有一遍恢复落到被判那一版、被判那次的根槽写落没落。
struct CpuJudgementOfAState {
    cross_check_words: CrossCheckWords,
    a_recovery_lands_on_the_judged_version: bool,
    judged_root_persisted: bool,
}

/// 每个状态两边各判一遍：GPU 结论字与 CPU 那一侧，按状态序号。卡借不到（等满了、或没有独显）交 `None`。
fn judge_every_state_on_both_sides(
    cards: GpuCardPoolStartedAtTheFirstLease,
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
    let item_name = format!("mapping_leaf_boundaries::{test_name}::cpu_side");
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

/// 一个状态 CPU 与 GPU 对拍字里恢复内核写的那一半：两遍 oracle 与记录核对器的红位、两遍落到的根（`LandedRootWords`）、两遍的走读结局与失败点。
#[derive(Debug, PartialEq, Eq)]
struct RecoveryHalfOfTheCrossCheckWords {
    red_bits: u32,
    consulted_pass_landed_root: LandedRootWords,
    ignored_pass_landed_root: LandedRootWords,
    consulted_pass_outcome: u32,
    ignored_pass_outcome: u32,
    consulted_pass_failure_point: u32,
    ignored_pass_failure_point: u32,
}

fn recovery_half_of_the_cross_check_words(
    words: &CrossCheckWords,
) -> RecoveryHalfOfTheCrossCheckWords {
    RecoveryHalfOfTheCrossCheckWords {
        red_bits: words.red_bits,
        consulted_pass_landed_root: words.consulted_pass_landed_root,
        ignored_pass_landed_root: words.ignored_pass_landed_root,
        consulted_pass_outcome: words.consulted_pass_outcome,
        ignored_pass_outcome: words.ignored_pass_outcome,
        consulted_pass_failure_point: words.consulted_pass_failure_point,
        ignored_pass_failure_point: words.ignored_pass_failure_point,
    }
}

/// 每个状态 GPU 恢复内核都判得了（结论第 0 字判不了那一位没置）、恢复那一半与 CPU 逐状态相同；不是的状态列出来（前几个）。
fn assert_every_state_recovered_on_the_gpu_like_the_cpu_recovery(
    flow_name: &str,
    gpu: &[[u32; VERDICT_WORDS]],
    cpu: &[CpuJudgementOfAState],
) {
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_judgement, gpu_words))| {
            let recovery_could_not_judge = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0;
            let gpu_half =
                recovery_half_of_the_cross_check_words(&cross_check_words_of_gpu_verdict(gpu_words));
            let cpu_half =
                recovery_half_of_the_cross_check_words(&cpu_judgement.cross_check_words);
            (recovery_could_not_judge || gpu_half != cpu_half).then(|| {
                format!(
                    "状态 {ordinal}：GPU 恢复内核判不了 {recovery_could_not_judge}（第 1 字 {:#x}）；GPU {gpu_half:?}；CPU {cpu_half:?}",
                    gpu_words[1]
                )
            })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "流 {flow_name}：{} 个状态 GPU 恢复内核判不了或恢复那一半与 CPU 不一致（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 要有被判那次的根槽写落了、看 journal 那一遍恢复落到那一版且读出了文件的状态（恢复读完了第一个文件的每个单元）。
fn assert_some_state_reads_the_file_of_the_judged_version(
    flow_name: &str,
    cpu: &[CpuJudgementOfAState],
) {
    assert!(
        cpu.iter().any(|judgement| {
            judgement.a_recovery_lands_on_the_judged_version
                && judgement.judged_root_persisted
                && judgement.cross_check_words.consulted_pass_outcome
                    == RECOVERY_OUTCOME_CODE_FILE_READ
        }),
        "流 {flow_name}：要有被判那次的根槽写落了、恢复落到那一版并读出文件的状态"
    );
}

// ───────── 改坏被判那次的单元写、一路重封到根

/// journal 记录头的字节数，点名项从这里起：随格式常量 `singlefs_format::JOURNAL_HEADER_BYTES` 走。
fn journal_record_header_bytes() -> usize {
    usize::try_from(JOURNAL_HEADER_BYTES).expect("journal 记录头的字节数装得进 usize")
}
/// journal 记录里一个点名项的字节数：随格式常量 `singlefs_format::JOURNAL_NAMED_ENTRY_BYTES` 走。
fn journal_named_entry_bytes() -> usize {
    usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("journal 点名项的字节数装得进 usize")
}
/// 一条位置条目的字节数：盘 4 + 槽 6 + 整单元校验和 4（D19（块指针的结构与宽度预算））。
const LOCATION_ENTRY_BYTES: usize = 14;
/// 单元头里类标签的偏移；码 2 节点头里层级与 key 宽的偏移（`unit::parse_index_node`）。
const UNIT_CLASS_OFFSET: usize = 6;
const INDEX_NODE_LEVEL_OFFSET: usize = 50;
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
/// 码 2 节点头去掉最小、最大两个 key 之后的字节数，与头末到第一条条目之间的字节数（`unit::parse_index_node`：头到 86 + 2 × key 宽为止，条目从头之后 29 字节起）。
const INDEX_NODE_HEADER_BYTES_WITHOUT_ITS_TWO_KEYS: usize = 86;
const INDEX_NODE_BYTES_BETWEEN_THE_HEADER_AND_THE_ENTRIES: usize = 29;
/// extent 下段叶的 key 宽（locality 8 + inode 8 + 偏移 8）。
const EXTENT_LOWER_LEAF_KEY_BYTES: u8 = 24;

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}

/// 码 2 节点的条目宽、第一条条目的偏移与条数。
fn index_node_entry_width_entries_start_and_entry_count(bytes: &[u8]) -> (usize, usize, usize) {
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let header_end = INDEX_NODE_HEADER_BYTES_WITHOUT_ITS_TWO_KEYS + 2 * key_width;
    (
        usize::from(read_u16_at(bytes, header_end - 2)),
        header_end + INDEX_NODE_BYTES_BETWEEN_THE_HEADER_AND_THE_ENTRIES,
        usize::from(read_u16_at(bytes, header_end - 4)),
    )
}

/// 一条位置条目写在单元里的 14 个字节。
fn location_entry_bytes(location: &LocationEntry) -> [u8; LOCATION_ENTRY_BYTES] {
    let mut bytes = [0u8; LOCATION_ENTRY_BYTES];
    bytes[..4].copy_from_slice(&location.device.0.to_le_bytes());
    bytes[4..10].copy_from_slice(&location.slot.0.to_le_bytes()[..6]);
    bytes[10..].copy_from_slice(&location.unit_checksum.to_le_bytes());
    bytes
}

/// `needle` 在 `haystack` 里每一处的起点。
fn positions_of(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    haystack
        .windows(needle.len())
        .enumerate()
        .filter(|(_, window)| *window == needle)
        .map(|(position, _)| position)
        .collect()
}

/// 一个单元重封自己的两道校验和：载荷 CRC（按类的偏移，罩明文头之后到单元末）与头校验和（字段 10，罩明文头）。
fn reseal_unit(bytes: &mut [u8]) {
    let (header_end, payload_crc_offset) = match bytes[UNIT_CLASS_OFFSET] {
        1 => (105, 101),
        2 => {
            let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
            (86 + 2 * key_width, 76 + 2 * key_width)
        }
        3 => (107, 89),
        other => panic!("单元类标签 {other} 不在 1 / 2 / 3 里"),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = wide_checksum_with_field_zeroed(bytes, header_end, 10);
    bytes[10..42].copy_from_slice(&digest);
}

/// journal 记录重封：载荷校验和（95，罩点名项）再整条头校验和（46，罩 4096）。
fn reseal_journal_record(bytes: &mut [u8]) {
    let named = usize::try_from(u32::from_le_bytes(bytes[12..16].try_into().expect("4")))
        .expect("点名项数");
    let payload_crc = crc32_castagnoli(
        &bytes[journal_record_header_bytes()
            ..journal_record_header_bytes() + named * journal_named_entry_bytes()],
    );
    bytes[95..99].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = wide_checksum_with_field_zeroed(bytes, 4096, 46);
    bytes[46..78].copy_from_slice(&digest);
}

/// 下一条记录该带的反向链：这一条记录头（[`journal_record_header_bytes`] 那么长）、校验和字段按 0 参与的 CRC-32C。
fn journal_chain_value(bytes: &[u8]) -> u32 {
    let mut header = bytes[..journal_record_header_bytes()].to_vec();
    header[46..78].fill(0);
    crc32_castagnoli(&header)
}

fn reseal_root_record(bytes: &mut [u8]) {
    let digest = wide_checksum_with_field_zeroed(bytes, bytes.len(), 138);
    bytes[138..170].copy_from_slice(&digest);
}

/// 改坏一批单元写（同一个单元的两份副本）之后把校验和一路重封到根：被改的单元先重封自己的两道校验和，再把每一处指着它的位置条目
/// （树节点、中央映射叶、根记录、journal 点名项里的 盘 4 + 槽 6 + 整单元校验和 4）换成新的整单元 CRC，被改的那些结构再各自重封
/// （单元继续往上传；journal 记录还要把同实例下一条的反向链改成它新的链值、一路传下去，等单元那一路传完再传一遍）。留下的坏法只剩被改的那几个字节。
fn corrupt_units_and_reseal_up_to_the_root(
    flow: &mut RecordedFlow,
    targets: &[usize],
    mutate: &dyn Fn(&mut [u8]),
) {
    let mut journal_records_resealed: BTreeSet<usize> = BTreeSet::new();
    let mut pending: Vec<(usize, u32)> = Vec::new();
    for target in targets {
        let write = &mut flow.writes[*target];
        assert_eq!(write.kind, StepKind::UnitWrite, "改的是单元写");
        let WrittenContents::Bytes(bytes) = &mut write.contents else {
            panic!("单元写是普通写");
        };
        let old_crc = crc32_castagnoli(bytes);
        mutate(bytes);
        reseal_unit(bytes);
        pending.push((*target, old_crc));
    }
    // 迭代上界：每一轮重封一个单元、往上传一层，到根记录与 journal 记录为止
    while let Some((index, old_crc)) = pending.pop() {
        let (device, slot, new_crc) = {
            let write = &flow.writes[index];
            let bytes = write.bytes().expect("单元写带字节");
            (
                write.device.0,
                write.offset.0 / 16384,
                crc32_castagnoli(bytes),
            )
        };
        if old_crc == new_crc {
            continue;
        }
        let mut pattern = Vec::with_capacity(LOCATION_ENTRY_BYTES);
        pattern.extend_from_slice(&device.to_le_bytes());
        pattern.extend_from_slice(&slot.to_le_bytes()[..6]);
        pattern.extend_from_slice(&old_crc.to_le_bytes());
        for referrer in 0..flow.writes.len() {
            let kind = flow.writes[referrer].kind;
            let WrittenContents::Bytes(bytes) = &mut flow.writes[referrer].contents else {
                continue;
            };
            // 码 1 数据单元里装的是文件内容、没有位置条目：长文件那条流上千个，不逐字节找
            if kind == StepKind::UnitWrite && bytes[UNIT_CLASS_OFFSET] == UNIT_CLASS_DATA {
                continue;
            }
            let mut replaced = false;
            let mut position = 0;
            while position + LOCATION_ENTRY_BYTES <= bytes.len() {
                if bytes[position..position + LOCATION_ENTRY_BYTES] == pattern[..] {
                    bytes[position + 10..position + LOCATION_ENTRY_BYTES]
                        .copy_from_slice(&new_crc.to_le_bytes());
                    replaced = true;
                    position += LOCATION_ENTRY_BYTES;
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
                        while restore_position + LOCATION_ENTRY_BYTES <= before.len() {
                            if before[restore_position..restore_position + 10] == pattern[..10]
                                && before
                                    [restore_position + 10..restore_position + LOCATION_ENTRY_BYTES]
                                    == new_crc.to_le_bytes()
                            {
                                before[restore_position + 10
                                    ..restore_position + LOCATION_ENTRY_BYTES]
                                    .copy_from_slice(&old_crc.to_le_bytes());
                                restore_position += LOCATION_ENTRY_BYTES;
                            } else {
                                restore_position += 1;
                            }
                        }
                        crc32_castagnoli(&before)
                    };
                    reseal_unit(bytes);
                    pending.push((referrer, referrer_old_crc));
                }
                StepKind::RootRecordFua => reseal_root_record(bytes),
                StepKind::JournalRecord => {
                    reseal_journal_record(bytes);
                    journal_records_resealed.insert(referrer);
                }
                StepKind::ZeroFill
                | StepKind::Discard
                | StepKind::SystemConfigurationSlot
                | StepKind::Barrier => {
                    panic!("位置条目不该出现在 {kind:?} 里")
                }
            }
        }
    }
    carry_the_back_chain_forward_from(flow, journal_records_resealed);
}

/// journal 记录头里实例代号（4 字节，偏移 16）与计数器（6 字节，偏移 20）。
fn journal_record_instance_and_counter(bytes: &[u8]) -> (u32, u64) {
    let mut counter = [0u8; 8];
    counter[..6].copy_from_slice(&bytes[20..26]);
    (
        u32::from_le_bytes(bytes[16..20].try_into().expect("4")),
        u64::from_le_bytes(counter),
    )
}

/// 反向链一路传下去：这几条 journal 记录写重封过，同盘、同实例、计数器大 1 的那一条带的是前一条头的链值——不等就改成它、重封，再往下传。
/// 按（盘、实例、计数器）从小到大过一遍（每条的链值只看计数器小 1 的那一条最后的头）：长文件那条流一次发布上千条记录，每条至多重封一次。
fn carry_the_back_chain_forward_from(
    flow: &mut RecordedFlow,
    mut journal_records_resealed: BTreeSet<usize>,
) {
    if journal_records_resealed.is_empty() {
        return;
    }
    let mut journal_record_writes: BTreeMap<(u32, u32, u64), Vec<usize>> = BTreeMap::new();
    for (write_index, write) in flow.writes.iter().enumerate() {
        if write.kind != StepKind::JournalRecord {
            continue;
        }
        let Some(bytes) = write.bytes() else {
            continue;
        };
        let (instance, counter) = journal_record_instance_and_counter(bytes);
        journal_record_writes
            .entry((write.device.0, instance, counter))
            .or_default()
            .push(write_index);
    }
    let keys: Vec<(u32, u32, u64)> = journal_record_writes.keys().copied().collect();
    for (device, instance, counter) in keys {
        let Some(previous_counter) = counter.checked_sub(1) else {
            continue;
        };
        let Some(previous_records) =
            journal_record_writes.get(&(device, instance, previous_counter))
        else {
            continue;
        };
        let chains: Vec<u32> = previous_records
            .iter()
            .filter(|previous| journal_records_resealed.contains(*previous))
            .map(|previous| {
                journal_chain_value(flow.writes[*previous].bytes().expect("记录写带字节"))
            })
            .collect();
        let records = journal_record_writes[&(device, instance, counter)].clone();
        for chain in chains {
            for record in &records {
                let WrittenContents::Bytes(bytes) = &mut flow.writes[*record].contents else {
                    continue;
                };
                if bytes[91..95] == chain.to_le_bytes() {
                    continue;
                }
                bytes[91..95].copy_from_slice(&chain.to_le_bytes());
                reseal_journal_record(bytes);
                journal_records_resealed.insert(*record);
            }
        }
    }
}

/// 被判那次写里一个单元的两份副本：命中 `select` 的单元写（同一个单元写两盘，字节相同）。
fn the_two_copies_of_the_unit(
    flow: &RecordedFlow,
    what: &str,
    select: &dyn Fn(&[u8]) -> bool,
) -> Vec<usize> {
    let copies: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite && write.bytes().is_some_and(select))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        copies.len(),
        2,
        "被判那次写里「{what}」的单元写是一个单元的两份副本"
    );
    assert_eq!(
        flow.writes[copies[0]].bytes(),
        flow.writes[copies[1]].bytes(),
        "「{what}」两份副本字节相同"
    );
    copies
}

/// 这一版的映射 key 里按中央映射树的序（[`mapping_key_sort_key`]：类标签、出生树、出生 txg、尾段逐段按整数比）排在 `key` 之前的有几条：
/// `key` 那一条在映射叶序里排第几（从 0 数）。不按字节比：key 里的整数是小端写的，码 1 尾段的写序事务号过 256 之后字节序与树里的序不同。
fn rank_among_the_mapping_keys(output: &TransactionOutput, key: &[u8]) -> usize {
    let sort_key = mapping_key_sort_key(key);
    output
        .mapping_keys
        .iter()
        .filter(|candidate| mapping_key_sort_key(candidate) < sort_key)
        .count()
}

/// 被判那次顺序写的一个单元（`data_pointer`）：extent 下段叶里它那条数据指针的两条位置提示改指另一个单元（`decoy`）的两个槽
/// （那两槽里是别的单元，整单元校验和对不上），下段叶两份副本都改、一路重封到根；映射里它那条条目不动。
/// 恢复（CPU 的冷走读与 GPU 恢复内核）读它都经映射回退，查映射要读到它那条条目。
fn make_the_location_hints_of_a_data_unit_stale(
    flow: &mut RecordedFlow,
    data_pointer: &DataPointer,
    decoy: &DataPointer,
) {
    let hints = data_pointer
        .locations
        .map(|location| location_entry_bytes(&location));
    let stale_hints = [0, 1].map(|copy| {
        assert_ne!(
            decoy.locations[copy].slot, data_pointer.locations[copy].slot,
            "两个单元不在同一槽"
        );
        let mut stale = hints[copy];
        stale[4..10].copy_from_slice(&decoy.locations[copy].slot.0.to_le_bytes()[..6]);
        stale
    });
    let lower_leaf_copies = the_two_copies_of_the_unit(
        flow,
        "那个单元的数据指针所在的 extent 下段叶",
        &|bytes| {
            bytes[UNIT_CLASS_OFFSET] == UNIT_CLASS_INDEX_NODE
                && bytes[INDEX_NODE_KEY_WIDTH_OFFSET] == EXTENT_LOWER_LEAF_KEY_BYTES
                && bytes[INDEX_NODE_LEVEL_OFFSET] == 0
                && hints
                    .iter()
                    .all(|hint| positions_of(bytes, hint).len() == 1)
        },
    );
    corrupt_units_and_reseal_up_to_the_root(flow, &lower_leaf_copies, &|bytes| {
        for (hint, stale) in hints.iter().zip(&stale_hints) {
            let position = positions_of(bytes, hint)[0];
            bytes[position..position + LOCATION_ENTRY_BYTES].copy_from_slice(stale);
        }
    });
}

// ───────── 映射叶边界上的数据单元

/// 一片映射叶里的一条码 1 条目：它的 27 字节 key、在全表里排第几（从 0 数），与它指的那个单元的数据指针（两条位置条目与它逐字节相同的那一条）。
struct DataUnitMappingEntry {
    key: Vec<u8>,
    rank: usize,
    data_pointer: DataPointer,
}

/// 被判那次写的一片中央映射叶（码 2、key 宽 27、层级 0）：首条与末条。
struct MappingLeafEnds {
    first: Vec<u8>,
    first_locations: Vec<u8>,
    last: Vec<u8>,
    last_locations: Vec<u8>,
}

/// 被判那次写里的每片中央映射叶（两份副本算一片），按首条的 key 在中央映射树里的序（[`mapping_key_sort_key`]）升序。
fn mapping_leaves_of_the_judged_publish(flow: &RecordedFlow) -> Vec<MappingLeafEnds> {
    let key_bytes = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let mut leaves: BTreeMap<(u8, u64, u64, u32, u64), MappingLeafEnds> = BTreeMap::new();
    for write in &flow.writes {
        let Some(bytes) = write.bytes() else {
            continue;
        };
        if write.kind != StepKind::UnitWrite
            || bytes[UNIT_CLASS_OFFSET] != UNIT_CLASS_INDEX_NODE
            || usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]) != key_bytes
            || bytes[INDEX_NODE_LEVEL_OFFSET] != 0
        {
            continue;
        }
        let (entry_width, entries_start, entry_count) =
            index_node_entry_width_entries_start_and_entry_count(bytes);
        assert!(entry_count > 0, "被判那次写的映射叶至少一条条目");
        let entry_at = |entry_index: usize| entries_start + entry_index * entry_width;
        let key_of = |entry_index: usize| {
            bytes[entry_at(entry_index)..entry_at(entry_index) + key_bytes].to_vec()
        };
        let locations_of = |entry_index: usize| {
            bytes[entry_at(entry_index) + key_bytes
                ..entry_at(entry_index) + key_bytes + 2 * LOCATION_ENTRY_BYTES]
                .to_vec()
        };
        leaves.insert(
            mapping_key_sort_key(&key_of(0)),
            MappingLeafEnds {
                first: key_of(0),
                first_locations: locations_of(0),
                last: key_of(entry_count - 1),
                last_locations: locations_of(entry_count - 1),
            },
        );
    }
    leaves.into_values().collect()
}

/// 两条位置条目与 `locations` 逐字节相同的那个单元的数据指针；`key` 不是码 1 条目交 `None`。
fn data_unit_mapping_entry(
    output: &TransactionOutput,
    key: &[u8],
    locations: &[u8],
) -> Option<DataUnitMappingEntry> {
    if key[0] != UNIT_CLASS_DATA {
        return None;
    }
    let data_pointer = *output
        .data_pointers
        .iter()
        .find(|pointer| {
            let mut bytes = Vec::with_capacity(2 * LOCATION_ENTRY_BYTES);
            for location in &pointer.locations {
                bytes.extend_from_slice(&location_entry_bytes(location));
            }
            bytes == locations
        })
        .expect("码 1 映射条目指着被判那次顺序写的一个单元");
    Some(DataUnitMappingEntry {
        key: key.to_vec(),
        rank: rank_among_the_mapping_keys(output, key),
        data_pointer,
    })
}

/// 映射叶边界上的数据单元：全表第一条（第一片叶的首条），与每一处两片相邻叶（左片末条、右片首条在全表里挨着）两侧都是码 1 条目时的那两条。
fn data_unit_mapping_entries_at_the_mapping_leaf_boundaries(
    flow: &RecordedFlow,
    output: &TransactionOutput,
) -> Vec<DataUnitMappingEntry> {
    let leaves = mapping_leaves_of_the_judged_publish(flow);
    assert!(
        leaves.len() >= MAPPING_LEAVES_AT_THE_LEAST,
        "被判那次写的中央映射有 {} 片叶，至少 {MAPPING_LEAVES_AT_THE_LEAST} 片",
        leaves.len()
    );
    let mut entries: Vec<DataUnitMappingEntry> = Vec::new();
    let first_leaf = &leaves[0];
    if let Some(entry) =
        data_unit_mapping_entry(output, &first_leaf.first, &first_leaf.first_locations)
    {
        assert_eq!(entry.rank, 0, "第一片叶的首条是全表第一条");
        entries.push(entry);
    }
    let mut boundaries_between_data_units = 0;
    for adjacent_leaves in leaves.windows(2) {
        let (left, right) = (&adjacent_leaves[0], &adjacent_leaves[1]);
        let (Some(left_last), Some(right_first)) = (
            data_unit_mapping_entry(output, &left.last, &left.last_locations),
            data_unit_mapping_entry(output, &right.first, &right.first_locations),
        ) else {
            continue;
        };
        assert_eq!(
            right_first.rank,
            left_last.rank + 1,
            "相邻两片映射叶：左片末条与右片首条在全表里挨着"
        );
        boundaries_between_data_units += 1;
        entries.push(left_last);
        entries.push(right_first);
    }
    assert!(
        boundaries_between_data_units > 0,
        "至少一处两片映射叶的边界两侧都是数据单元的条目"
    );
    entries
}

/// 长文件（[`DATA_UNITS_OF_THE_LONG_FILE`] 个单元、几片映射叶）映射里排在片边界上的数据单元（全表第一条、每处边界的左片末条与右片首条），
/// 两条位置提示都改指别的槽、重封到根：恢复读它们都要经映射回退、查到片边界上的条目。每个状态 GPU 恢复内核判得了、恢复那一半与 CPU 逐状态相同，
/// 有状态恢复落到被判那一版并读出文件。叶之间二分比每片的首条时左片末条落到右片、查不到，恢复失败，这条红。
#[test]
fn the_gpu_recovery_kernel_finds_central_mapping_entries_at_the_mapping_leaf_boundaries_like_the_cpu_recovery(
) {
    const TEST_NAME: &str = "the_gpu_recovery_kernel_finds_central_mapping_entries_at_the_mapping_leaf_boundaries_like_the_cpu_recovery";
    const FLOW_NAME: &str = "long-file-with-stale-hints-on-the-mapping-leaf-boundaries";
    let (mut flow, output) = record_the_long_file_flow("crash-judge-gpu-mapping-leaf-boundaries");
    let targets = data_unit_mapping_entries_at_the_mapping_leaf_boundaries(&flow, &output);
    let target_keys: BTreeSet<&[u8]> = targets.iter().map(|entry| entry.key.as_slice()).collect();
    assert_eq!(target_keys.len(), targets.len(), "片边界上的条目各不相同");
    let decoy = *output
        .data_pointers
        .iter()
        .find(|pointer| targets.iter().all(|entry| entry.data_pointer != **pointer))
        .expect("长文件里有不在片边界上的单元");
    for entry in &targets {
        make_the_location_hints_of_a_data_unit_stale(&mut flow, &entry.data_pointer, &decoy);
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, TEST_NAME, &flow) else {
        return;
    };
    assert_some_state_reads_the_file_of_the_judged_version(FLOW_NAME, &cpu);
    assert_every_state_recovered_on_the_gpu_like_the_cpu_recovery(FLOW_NAME, &gpu, &cpu);
}
