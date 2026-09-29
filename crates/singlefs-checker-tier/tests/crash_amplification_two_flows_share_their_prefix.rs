//! checker 档模块：crash、layer0_progress、crash_identity、crash_amplification、verdict_store
//! 前缀共享（用户 2026-09-28 定：剪枝的基础）：两条流都从 mkfs → 取号加暖机 → 新池新建文件起，之后各自覆盖写不同的内容。
//! 前缀那些点由公共模块 `common_crash_points` 设，所以两条流里它们的身份、路径、块键逐个相同：流 A 核完之后，流 B 录入时前缀全部复用、
//! 只核自己的尾巴；尾巴同名不同数据，路径这一级并成一段、身份不同；判法摘要一份、对全部点相同，改了它树不动、每个点的复用键都变。
//! 路径在库的路径索引表里（路径 → 身份），块键里只有身份。要 `verdict-store` 特性。
#![cfg(feature = "verdict-store")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;

use std::path::PathBuf;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use common_crash_points::{overwrite_step, record_pool_build};
use singlefs_checker_tier::crash::Layer0SegmentExpansion;
use singlefs_checker_tier::crash_amplification::{
    check_recorded_blocks, plan_crash_points, record_crash_points, repository_relative_file,
    unjudged_blocks, CrashFlow, CrashPlan, CrashPointRecorder, CrashPointSpan, PipelineJudgingCode,
    UnitChecksumsOn, WeightedShares,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_checker_tier::verdict_store::VerdictStore;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;

const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs";
const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;

fn expansion(_segment_index: usize, segment: &[usize]) -> Layer0SegmentExpansion {
    if segment.len() <= LARGEST_EXPANDED_SEGMENT_WRITES {
        Layer0SegmentExpansion::EveryProperSubset
    } else {
        Layer0SegmentExpansion::NotExpanded
    }
}

fn content_of(seed: usize, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    crash_points: Vec<CrashPointSpan>,
}

impl RecordedFlow {
    fn flow(&self) -> CrashFlow<'_> {
        CrashFlow {
            base: &self.base,
            writes: &self.writes,
            segments: &self.segments,
            judged_root_index: self.judged_root_index,
            versions: &self.versions,
            expansion: &expansion,
            crash_points: self.crash_points.clone(),
        }
    }
}

/// 覆盖写，但崩溃点由这份测试文件自己设（不是公共模块）：流 B 的尾巴。
fn overwrite_step_set_here(
    recorder: &mut CrashPointRecorder,
    pool: &mut BuiltPool,
    content: &[u8],
) {
    recorder.step(file!(), "publish_overwrite", || {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile {
                content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("覆盖写");
    });
}

/// mkfs → 取号加暖机 → 新池新建文件（公共模块设点）→ 一次覆盖写（公共模块设点，或这份文件自己设点）。
fn record_flow(tag: &str, overwrite_content: &[u8], tail_set_in_common: bool) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
    record_pool_build(&mut recorder, &pool);
    if tail_set_in_common {
        overwrite_step(
            &mut recorder,
            &mut pool,
            overwrite_content,
            InstanceGeneration(1),
        );
    } else {
        overwrite_step_set_here(&mut recorder, &mut pool, overwrite_content);
    }
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    RecordedFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: overwrite_content.to_vec(),
            },
        ],
        crash_points,
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("仓根")
}

/// 前缀的点数：名字不以 `publish_overwrite.` 起头的都是前缀。
fn prefix_length(plan: &CrashPlan) -> usize {
    plan.points
        .iter()
        .take_while(|point| !point.name.starts_with("publish_overwrite."))
        .count()
}

struct TemporaryStore {
    directory: PathBuf,
}

impl Drop for TemporaryStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn two_flows_sharing_their_prefix_reuse_its_verdicts_and_fork_only_at_the_tail() {
    let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
    let root = repository_root();
    let flow_a = record_flow("prefix-share-a", &content_of(3, 4100), true);
    let flow_b = record_flow("prefix-share-b", &content_of(11, 4100), false);
    let judging = PipelineJudgingCode::of_the_judges(&root, &toolchain).expect("判法");
    let version = judging.version();
    let plan_a = plan_crash_points(&flow_a.flow());
    let plan_b = plan_crash_points(&flow_b.flow());

    // 一、前缀逐点相同：身份、路径、块键
    let prefix = prefix_length(&plan_a);
    assert!(prefix >= 2, "取号加暖机与新池新建文件至少两段：{prefix}");
    assert_eq!(prefix, prefix_length(&plan_b));
    for index in 0..prefix {
        let (a, b) = (&plan_a.points[index], &plan_b.points[index]);
        assert_eq!(a.identity, b.identity, "前缀第 {index} 个点身份相同");
        assert_eq!(a.path_number, b.path_number, "前缀第 {index} 个点路径相同");
        assert_eq!(a.reuse_key, b.reuse_key);
        assert!(
            a.identity
                .starts_with(&format!("{}::", common_crash_points::THIS_FILE)),
            "前缀的点由公共模块设：{}",
            a.identity
        );
    }
    assert!(
        plan_a.points[0]
            .path_number
            .starts_with("acquire_instance_and_warm_up."),
        "{}",
        plan_a.points[0].path_number
    );
    assert_eq!(
        plan_a.points[prefix].path_number,
        format!(
            "{}/{}",
            plan_a.points[prefix - 1].path_number,
            plan_a.points[prefix].name
        ),
        "路径 = 父路径 / 自己的名字"
    );

    // 二、尾巴：同名（同一入口、同一种写、同一步）、不同数据 ⇒ 路径这一级并成一段，身份不同；文件各是设它的那一份
    let tail_a = &plan_a.points[prefix];
    let tail_b = &plan_b.points[prefix];
    assert_eq!(tail_a.name, tail_b.name);
    assert_eq!(
        tail_a.path_number, tail_b.path_number,
        "同名分叉在路径这一级并成一段"
    );
    assert_ne!(tail_a.identity, tail_b.identity, "数据不同，身份不同");
    assert!(
        tail_a.identity.starts_with(&format!(
            "{}::publish_overwrite.",
            common_crash_points::THIS_FILE
        )),
        "{}",
        tail_a.identity
    );
    assert!(
        tail_b
            .identity
            .starts_with(&format!("{THIS_FILE}::publish_overwrite.")),
        "{}",
        tail_b.identity
    );
    assert_eq!(repository_relative_file(file!()), THIS_FILE);

    // 三、判法改了：计划不看判法，树与每个点的复用键都不动；判法版本只在判定行上分（第四段末尾核）
    let mut judging_changed = judging.clone();
    let mut flipped = *judging.digest();
    flipped[0] ^= 0x01;
    judging_changed.replace_digest(flipped);
    let changed_version = judging_changed.version();
    assert_ne!(changed_version, version);
    let plan_b_changed = plan_crash_points(&flow_b.flow());
    assert_eq!(plan_b_changed.points.len(), plan_b.points.len());
    for (changed, unchanged) in plan_b_changed.points.iter().zip(&plan_b.points) {
        assert_eq!(
            changed.path_number, unchanged.path_number,
            "判法改了树不动：{}",
            unchanged.name
        );
        assert_eq!(
            changed.reuse_key, unchanged.reuse_key,
            "判法改了复用键也不动：{}",
            unchanged.name
        );
    }

    // 四、库：A 录、核；B 录入时前缀全部复用、只核尾巴；路径索引表里前缀一条一份、尾巴一条（最后写的身份）
    let temporary = TemporaryStore {
        directory: std::env::temp_dir().join(format!(
            "singlefs-crash-amplification-prefix-share-{}",
            std::process::id()
        )),
    };
    let _ = std::fs::remove_dir_all(&temporary.directory);
    let mut store = VerdictStore::create_empty(&temporary.directory).expect("建得了库");
    let mut report_a = CoverageReport::default();
    let recording_a =
        record_crash_points(&mut store, &plan_a, &version, &mut report_a).expect("录得进");
    assert_eq!(recording_a.blocks_already_judged, 0);
    let checking_a = check_recorded_blocks(
        &mut store,
        &flow_a.flow(),
        &plan_a,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &version,
    )
    .expect("核得完");
    assert_eq!(checking_a.red_states, 0, "{:?}", checking_a.red_texts);
    assert!(unjudged_blocks(&store, &version)
        .expect("对得了账")
        .is_empty());
    // 判法版本换了：A 的每一块在新版本下都算没判，旧版本的行留着
    let mut report_changed = CoverageReport::default();
    let recording_changed =
        record_crash_points(&mut store, &plan_a, &changed_version, &mut report_changed)
            .expect("录得进");
    assert_eq!(
        recording_changed.blocks_already_judged, 0,
        "换了判法版本一块都不复用"
    );
    assert_eq!(
        unjudged_blocks(&store, &changed_version)
            .expect("对得了账")
            .len(),
        usize::try_from(recording_changed.blocks_recorded).expect("块数")
    );
    assert!(
        unjudged_blocks(&store, &version)
            .expect("对得了账")
            .is_empty(),
        "旧版本的判定还在"
    );

    let mut report_b = CoverageReport::default();
    let recording_b =
        record_crash_points(&mut store, &plan_b, &version, &mut report_b).expect("录得进");
    let prefix_blocks: u64 = plan_a.points[..prefix]
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数"))
        .sum();
    assert_eq!(
        recording_b.blocks_already_judged,
        prefix_blocks,
        "前缀的块全部复用：{:?}",
        report_b.lines()
    );
    let tail_blocks: u64 = plan_b.points[prefix..]
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数"))
        .sum();
    assert_eq!(recording_b.blocks_recorded, tail_blocks, "只有尾巴要核");
    for line in &report_b.lines()[..prefix] {
        assert!(
            line.ends_with(" reused") || line.ends_with(" not_expanded"),
            "前缀的点要么复用，要么段大过展开上限没有状态可复用：{line}"
        );
    }
    assert!(
        report_b.lines()[..prefix]
            .iter()
            .any(|line| line.ends_with(" reused")),
        "{:?}",
        report_b.lines()
    );
    for line in &report_b.lines()[prefix..] {
        assert!(
            line.ends_with(" checked") || line.ends_with(" not_expanded"),
            "尾巴的点要么核了，要么段大过展开上限没有状态：{line}"
        );
    }
    assert!(
        report_b.lines()[prefix..]
            .iter()
            .any(|line| line.ends_with(" checked")),
        "{:?}",
        report_b.lines()
    );
    let checking_b = check_recorded_blocks(
        &mut store,
        &flow_b.flow(),
        &plan_b,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &version,
    )
    .expect("核得完");
    assert_eq!(checking_b.blocks_checked, tail_blocks);
    assert_eq!(checking_b.red_states, 0, "{:?}", checking_b.red_texts);
    assert!(unjudged_blocks(&store, &version)
        .expect("对得了账")
        .is_empty());

    let paths = store.paths_with_prefix("").expect("读得了路径索引");
    assert_eq!(
        paths.len(),
        plan_a.points.len(),
        "两条流同名，路径索引里一条路径一行：{paths:?}"
    );
    assert_eq!(
        store
            .identity_of_path(&tail_b.path_number)
            .expect("读得了")
            .as_deref(),
        Some(tail_b.identity.as_str()),
        "同一路径最后写的身份留在索引里"
    );
    let subtree = store
        .paths_with_prefix(&format!("{}/", plan_a.points[0].path_number))
        .expect("读得了");
    assert_eq!(
        subtree.len(),
        plan_a.points.len() - 1,
        "按前缀扫就是列子树：根之下的每个节点各一行"
    );

    // 五、改名只动路径索引：把流 B 的第一个点改名（祖先改名），它自己的身份变（名字在身份里），后代的身份不变、路径变、块照样复用
    let mut renamed_flow = flow_b.flow();
    renamed_flow.crash_points[0].name = format!("renamed_{}", renamed_flow.crash_points[0].name);
    let plan_renamed = plan_crash_points(&renamed_flow);
    assert_ne!(plan_renamed.points[0].identity, plan_b.points[0].identity);
    for index in 1..plan_b.points.len() {
        assert_eq!(
            plan_renamed.points[index].identity, plan_b.points[index].identity,
            "祖先改名，后代身份不变"
        );
        assert_ne!(
            plan_renamed.points[index].path_number, plan_b.points[index].path_number,
            "祖先改名，后代路径变"
        );
    }
    let mut report_renamed = CoverageReport::default();
    let recording_renamed =
        record_crash_points(&mut store, &plan_renamed, &version, &mut report_renamed)
            .expect("录得进");
    let first_blocks = u64::try_from(plan_renamed.points[0].blocks().len()).expect("块数");
    let all_blocks: u64 = plan_renamed
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数"))
        .sum();
    assert_eq!(
        recording_renamed.blocks_recorded, first_blocks,
        "只有改名的那个点要核"
    );
    assert_eq!(
        recording_renamed.blocks_already_judged,
        all_blocks - first_blocks,
        "路径不进块键：后代的判定照样复用 {:?}",
        report_renamed.lines()
    );
    drop(store);
    drop(temporary);
}
