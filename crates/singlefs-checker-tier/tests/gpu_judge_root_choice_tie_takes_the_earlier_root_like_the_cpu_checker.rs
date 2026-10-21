//! checker 档模块：crash、crash_state_enumeration、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器里池级 checker 那一份的择根平局（代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 辩方 Z1）：
//! 根环里两条自证过的根 (txg, 实例) 相同、内容不同时，CPU 的池级 checker 取读根环次序里最前的那条（`crates/singlefs-checker/src/walk.rs`
//! 择根的 fold，只在严格更大时换），GPU 判器要取同一条。造法：新池新建文件那条流，把发布的根抄一份、树表指针与映射根指针换成暖机那条根的、
//! 重封，在流尾另起一段写进发布的根的下一个根槽（读根环次序在它后面）。两条根都落了的状态上择到哪一条，池级 checker 的判定不同
//! （前提在取卡之前用 CPU 判：两条根换位的镜像判出的违例不同），GPU 与 CPU 逐状态的违例位要相同。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
#![cfg(feature = "gpu")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::BTreeSet;

use common::{build_pool, file_content, geometry};
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion, CrashStateJudgement,
};
use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};

/// 这份测试文件的仓内路径：崩溃点重放取样过库的设点文件。
const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs";
use singlefs_checker_tier::crash_judge_gpu::{
    pool_checker_violated_and_judged_bits, pool_checker_violated_bits, verdict_bits,
    POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS,
};
use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
    gpu_card_pool_or_report_not_run,
    image_with_every_write_landed_read_back_from_the_pairing_library,
};
use singlefs_core::address::{CheckpointTxg, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::recovery::PoolReader;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    WrittenContents,
};
use singlefs_harness::segments::StepKind;

/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;
/// 根记录里树表指针、映射根指针两段（D22（单元原子性怎么合成） 已定项 7 的根记录布局）与整槽校验和的偏移。
const ROOT_RECORD_TREE_TABLE_POINTER: std::ops::Range<usize> = 36..122;
const ROOT_RECORD_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 256..342;
const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
const ROOT_RECORD_CHECKSUM_BYTES: usize = 32;

/// 一条流：mkfs 之后的基镜像、写表与段、判的那条根、模型认的版本。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 重封根记录的整槽校验和（罩整个槽宽，校验和那一段按 0 算）。
fn reseal_root_record(bytes: &mut [u8]) {
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
        bytes,
        bytes.len(),
        ROOT_RECORD_CHECKSUM_OFFSET,
    );
    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + ROOT_RECORD_CHECKSUM_BYTES]
        .copy_from_slice(&digest);
}

fn bytes_of_the_root_write(write: &RetainedWrite) -> Vec<u8> {
    write.bytes().expect("根槽写带字节").to_vec()
}

/// 发布的根的字节，树表指针与映射根指针换成暖机那条根的、重封：(txg, 实例) 与发布的根相同，走读走的是暖机那一版的树。
fn root_with_the_publish_key_and_the_warm_up_trees(
    publish_root: &[u8],
    warm_up_root: &[u8],
) -> Vec<u8> {
    let mut bytes = publish_root.to_vec();
    bytes[ROOT_RECORD_TREE_TABLE_POINTER]
        .copy_from_slice(&warm_up_root[ROOT_RECORD_TREE_TABLE_POINTER]);
    bytes[ROOT_RECORD_MAPPING_ROOT_POINTER]
        .copy_from_slice(&warm_up_root[ROOT_RECORD_MAPPING_ROOT_POINTER]);
    reseal_root_record(&mut bytes);
    assert_ne!(bytes, publish_root, "换了树之后与发布的根内容不同");
    bytes
}

/// 两条同键的根各放哪一槽：发布的根自己的槽在读根环次序里排前，它的下一个根槽排后。
enum WhichRootIsReadFirst {
    ThePublishRoot,
    TheRootWithTheWarmUpTrees,
}

/// 新池新建文件那条流，发布的根的槽与它的下一个根槽上是两条 (txg, 实例) 相同、内容不同的根：一条是发布的根，
/// 一条带暖机那一版的树（[`root_with_the_publish_key_and_the_warm_up_trees`]）。后一槽的那一次写在流尾另起一段。
fn new_pool_with_two_roots_of_the_publish_key(
    tag: &str,
    read_first: WhichRootIsReadFirst,
) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (mut writes, mut segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    let warm_up_root_index = writes[..judged_root_index]
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布之前有暖机的根槽写");
    let base = pool.memory_pool_after_mkfs();
    let publish_root_write = writes[judged_root_index].clone();
    let publish_root = bytes_of_the_root_write(&publish_root_write);
    let root_with_the_warm_up_trees = root_with_the_publish_key_and_the_warm_up_trees(
        &publish_root,
        &bytes_of_the_root_write(&writes[warm_up_root_index]),
    );
    let next_slot = DeviceOffsetInBytes(
        publish_root_write.offset.0 + u64::from(geometry().fixed_structure_slot_spacing),
    );
    let next_slot_in_the_base = PoolReader::read(
        &base,
        publish_root_write.device,
        next_slot,
        usize::try_from(publish_root_write.length_in_bytes()).expect("根槽写的长度装得进 usize"),
    )
    .expect("基镜像里读得到下一个根槽");
    assert!(
        next_slot_in_the_base.iter().all(|byte| *byte == 0),
        "下一个根槽在基镜像里空着（发布的根在区域里不是最后一槽）"
    );
    assert!(
        writes
            .iter()
            .all(|write| (write.device, write.offset) != (publish_root_write.device, next_slot)),
        "流里没有别的写落在下一个根槽上"
    );
    let (first_slot_bytes, next_slot_bytes) = match read_first {
        WhichRootIsReadFirst::ThePublishRoot => (publish_root, root_with_the_warm_up_trees),
        WhichRootIsReadFirst::TheRootWithTheWarmUpTrees => {
            (root_with_the_warm_up_trees, publish_root)
        }
    };
    writes[judged_root_index].contents = WrittenContents::Bytes(first_slot_bytes);
    let appended_write_index = writes.len();
    writes.push(RetainedWrite {
        device: publish_root_write.device,
        kind: publish_root_write.kind,
        is_force_unit_access: publish_root_write.is_force_unit_access,
        offset: next_slot,
        contents: WrittenContents::Bytes(next_slot_bytes),
    });
    segments.push(vec![appended_write_index]);
    RecordedFlow {
        base,
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

/// 全部写都落了的那个镜像上，CPU 池级 checker 判红的不变量：镜像的输入先经对拍那一类的库（写进去再读回，用户 2026-10-13 定：
/// CPU 参照也从库读），判的是读回的那一份；只录那一趟交 `None`。
// cpu-reference-for-gpu-comparison
fn invariants_violated_with_every_write_landed(
    flow: &RecordedFlow,
) -> Option<BTreeSet<&'static str>> {
    let image =
        image_with_every_write_landed_read_back_from_the_pairing_library(&flow.base, &flow.writes)?;
    let violated = check_pool_image(&image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    Some(violated)
}

/// CPU 池级 checker 的违例位：位 i ↔ `IMPLEMENTED_INVARIANTS[i]`（与 GPU 判器结论里那一段同一个编号）；经 `pool_checker_violated_and_judged_bits` 取
/// （与 CPU 一侧的对拍字同一个函数），与 GPU 那一份逐位整份比（I-1.13 由扫描重建那一截判、置位，内核不置位的表是空的，C821）。
fn pool_checker_bits_on_the_cpu(judgement: &CrashStateJudgement) -> u128 {
    let (violated, _judged) = pool_checker_violated_and_judged_bits(&judgement.pool_checker);
    violated
}

fn invariant_names(bits: u128) -> Vec<&'static str> {
    IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(index, _)| bits & (1u128 << index) != 0)
        .map(|(_, name)| *name)
        .collect()
}

#[test]
fn gpu_judge_takes_the_earlier_of_two_roots_with_the_same_key_like_the_cpu_checker() {
    let flow = new_pool_with_two_roots_of_the_publish_key(
        "gpu-judge-root-choice-tie",
        WhichRootIsReadFirst::ThePublishRoot,
    );
    // 前提（取卡之前、CPU 上判）：两条同键的根换位之后池级 checker 判出的违例不同——择到哪一条决定判定，
    // GPU 择错时会有状态的违例位与 CPU 不同；换位不改判定的话这条流分不出择根平局，用例无效。
    let Some(violated_with_the_publish_root_first) =
        invariants_violated_with_every_write_landed(&flow)
    else {
        return;
    };
    let Some(violated_with_the_warm_up_trees_first) =
        invariants_violated_with_every_write_landed(&new_pool_with_two_roots_of_the_publish_key(
            "gpu-judge-root-choice-tie-swapped",
            WhichRootIsReadFirst::TheRootWithTheWarmUpTrees,
        ))
    else {
        return;
    };
    assert_ne!(
        violated_with_the_publish_root_first, violated_with_the_warm_up_trees_first,
        "两条同键的根换位之后池级 checker 判出的违例要不同，这条流才分得出择根平局"
    );
    const TEST_NAME: &str =
        "gpu_judge_takes_the_earlier_of_two_roots_with_the_same_key_like_the_cpu_checker";
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
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
    assert_eq!(tables.state_count, state_count);
    let Some(gpu): Option<Vec<[u32; VERDICT_WORDS]>> =
        judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
    else {
        return;
    };
    let mut cpu: Vec<u128> = Vec::new();
    let Some(judged_on_the_cpu) = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: TEST_NAME,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| cpu.push(pool_checker_bits_on_the_cpu(judgement)),
    )
    .judged() else {
        return;
    };
    assert_eq!(judged_on_the_cpu, state_count);
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(&gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_bits, gpu_words))| {
            let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
                || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0;
            let gpu_bits = pool_checker_violated_bits(gpu_words);
            (undecidable || gpu_bits != *cpu_bits).then(|| {
                format!(
                    "状态 {ordinal}：CPU {:?}；GPU {:?}（判不了：{undecidable}）",
                    invariant_names(*cpu_bits),
                    invariant_names(gpu_bits)
                )
            })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} 个状态 GPU 与 CPU 的池级 checker 违例位不同（GPU 择了读根环次序在后的那条同键根）：\n{}",
        mismatches.len(),
        mismatches.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}
