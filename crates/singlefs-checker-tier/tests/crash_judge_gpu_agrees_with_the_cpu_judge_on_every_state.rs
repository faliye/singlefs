//! checker 档模块：crash、crash_identity、crash_amplification、verdict_store、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器与 CPU 判器逐状态对拍：六条流（新池新建文件；它改坏一次树表单元写两份副本之后的样子；它的 journal 记录提交标记改成 2、
//! 记录标志改成 0b11、系统配置槽距改成 512 三种重封过校验和的坏法；固定脚本到回退 D）上的每个
//! 崩溃状态，GPU 判出的红位（两遍 oracle 各七类、记录核对器两条）要与 CPU `judge_crash_image` 的 `red_items` 逐位相同，池级 checker
//! 内核覆盖到的那几条不变量（`POOL_CHECKER_COVERED_INVARIANT_INDEXES`）的违例位要与 CPU 池级 checker 逐位相同，一个状态都不许判不了。
//! 改坏那条流上 CPU 有红，GPU 要红在同样的状态、同样的项上——这是 GPU 判器分得出差别的自证。
//! 要本机一张空闲显存够的 NVIDIA 独显（列不出就 panic，不悄悄跳过）；要 `verdict-store,gpu` 特性；
//! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）。
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
    gpu_judge_digest, pool_checker_violated_bits, verdict_bits,
    POOL_CHECKER_COVERED_INVARIANT_INDEXES, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::gpu_unit_checks::{usable_gpu_cards, GpuCard};
use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::RollbackTarget;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;

const DEFAULT_EXPAND_UP_TO: usize = 8;
const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn expand_up_to() -> usize {
    std::env::var("SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(DEFAULT_EXPAND_UP_TO)
}

/// 取一张卡：空闲显存最大的那张。`SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` 给了就当这张卡的显存额度（量速度时照配置的额度量）。
fn one_usable_card() -> GpuCard {
    let mut card = usable_gpu_cards()
        .into_iter()
        .next()
        .expect("本机要有一张空闲显存够的 NVIDIA 独显；没有就是这条测试的环境没搭好，不悄悄跳过");
    if let Some(quota) = std::env::var("SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES")
        .ok()
        .and_then(|text| text.parse::<u64>().ok())
    {
        card.memory_quota_mebibytes = Some(quota);
    }
    card
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
    let payload_crc = crc32c(&bytes[311..311 + named * 56]);
    bytes[95..99].copy_from_slice(&payload_crc.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(bytes, 4096, 46);
    bytes[46..78].copy_from_slice(&digest);
}

/// 下一条记录该带的反向链：这一条 311 字节头、校验和字段按 0 参与的 CRC-32C。
fn journal_chain_value(bytes: &[u8]) -> u32 {
    let mut header = bytes[..311].to_vec();
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
        // 反向链一路传下去：同盘、同实例、计数器大 1 的那一条带的是这一条头的链值
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
fn expected_pool_checker_bits(judgement: &Layer0StateJudgement) -> u64 {
    let mut bits = 0u64;
    for (invariant, verdict) in &judgement.pool_checker {
        if let InvariantVerdict::Violated(_) = verdict {
            let index = IMPLEMENTED_INVARIANTS
                .iter()
                .position(|candidate| candidate == invariant)
                .expect("池级 checker 报的不变量在清单里");
            bits |= 1u64 << index;
        }
    }
    bits
}

fn covered_mask() -> u64 {
    POOL_CHECKER_COVERED_INVARIANT_INDEXES
        .iter()
        .fold(0u64, |mask, index| mask | (1u64 << index))
}

fn invariant_names(bits: u64) -> Vec<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(index, _)| bits & (1u64 << index) != 0)
        .map(|(_, name)| *name)
        .collect()
}

struct Comparison {
    states: u64,
    cpu_red: u64,
    gpu_red: u64,
    cpu_checker_red: u64,
    gpu_checker_red: u64,
    /// CPU 池级 checker 在这条流的哪几条不变量上红过（覆盖到的那几条）。
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
    let threads: u64 = std::env::var("SINGLEFS_CRASH_JUDGE_CPU_THREADS")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism().map_or(1, |count| {
                u64::try_from(count.get()).expect("核数装得进 u64")
            })
        })
        .max(1);
    let slice = state_count.div_ceil(threads * 8).max(1);
    let next_slice = std::sync::atomic::AtomicU64::new(0);
    let slice_count = state_count.div_ceil(slice);
    type JudgedOnCpu = (u32, u64, String, String, [u32; 4]);
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
    let covered = covered_mask();
    for (ordinal, (cpu_bits, cpu_checker_bits, cpu_items, cpu_report, cpu_recovery)) in
        cpu.iter().enumerate()
    {
        let gpu_words: [u32; VERDICT_WORDS] = gpu[ordinal];
        let gpu_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
        let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
            || gpu_words[singlefs_checker_tier::crash_judge_gpu::POOL_CHECKER_UNDECIDABLE_WORD]
                != 0;
        let gpu_checker_bits = pool_checker_violated_bits(&gpu_words) & covered;
        let cpu_checker_covered = *cpu_checker_bits & covered;
        if *cpu_bits != 0 {
            comparison.cpu_red += 1;
        }
        if gpu_bits != 0 {
            comparison.gpu_red += 1;
        }
        if cpu_checker_covered != 0 {
            comparison.cpu_checker_red += 1;
            comparison
                .cpu_checker_red_invariants
                .extend(invariant_names(cpu_checker_covered));
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
            || gpu_checker_bits != cpu_checker_covered
            || gpu_recovery != *cpu_recovery
        {
            if comparison.mismatches.len() < 12 {
                comparison.mismatches.push(format!(
                    "状态 {ordinal}：CPU {cpu_bits:#x} {cpu_items} {cpu_report} checker={:?}（全部 {:?}）；GPU {gpu_words:?} checker={:?}",
                    invariant_names(cpu_checker_covered),
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
    let card = one_usable_card();
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
                "流 {} 上 CPU 池级 checker 覆盖到的那几条要有红",
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
    let payload_crc = crc32c(&record[311..311 + named * 56]);
    assert_eq!(
        payload_crc,
        u32::from_le_bytes(record[95..99].try_into().expect("4")),
        "记录载荷校验和重封过"
    );
    for entry in 0..named {
        let o = 311 + entry * 56;
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
    let card = one_usable_card();
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
    let card = std::sync::Arc::new(one_usable_card());
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
    let every_card: Vec<(u32, u64)> = usable_gpu_cards()
        .iter()
        .map(|card| (card.index, card.free_memory_mebibytes))
        .collect();
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
    let card = one_usable_card();
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
