//! checker 档模块：crash、layer0_progress、crash_identity、crash_amplification、verdict_store
//! 前缀共享（剪枝的基础）：两条流都从 mkfs → 取号加暖机 → 新池新建文件起，之后各自覆盖写不同的内容。
//! 前缀那些点由公共模块 `common_crash_points` 设，所以两条流里它们的身份、路径、块键逐个相同：流 A 核完之后，流 B 录入时前缀全部复用、
//! 只核自己的尾巴；尾巴同名不同数据，路径这一级并成一段、身份不同；判法摘要一份、对全部点相同，改了它树不动、每个点的复用键都变。
//! 路径在库的路径索引表里（路径 → 身份），块键里只有身份。判器另读的流程输入（够得着的版本、被判的根槽写落在哪、之前的写表、没做完的那次发布的
//! 根槽写）进计划哈希：只差版本表的一对、只差一条根槽写的一对都不互相复用，前缀够不着的那几样不影响前缀复用（R1、M3）；每条流的第一个点
//! 之前一条根槽写都没有，够得着的版本按基镜像里的根截，几条只差版本表的流在第一个点照样共享（test-script-align-code-r2 判决 J1）。
//! 根槽写的 txg 不严格递增时判器输入罩着点之后的整段写表（J1 退成整段写表那一支）。基镜像照 journal 恢复会越过它里面最新的根的流
//! （基镜像是崩溃状态），排计划时就拒（B0 的前提，用户 2026-10-02 定「窄一」）。要 `verdict-store` 特性。
#![cfg(feature = "verdict-store")]

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;
mod common_crash_points;

use std::path::PathBuf;

use common::{
    build_pool, crash_state_devices, file_content, geometry, parameters, BuiltPool,
    FIXED_WRITE_TIME_SECONDS,
};
use common_crash_points::{overwrite_step, record_pool_build};
use singlefs_checker_tier::crash::{
    judge_layer0_state_range, layer0_plan_state_count, Layer0SegmentExpansion,
};
use singlefs_checker_tier::crash_amplification::{
    check_recorded_blocks, plan_crash_points, read_verdict_items_of_every_state,
    record_crash_points, repository_relative_file, unjudged_blocks, CheckingTally, CrashFlow,
    CrashPlan, CrashPointRecorder, CrashPointSpan, PipelineJudgingCode, RecordingTally,
    UnitChecksumsOn, WeightedShares,
};
use singlefs_checker_tier::crash_identity::CoverageReport;
use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::mount_writable;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::memory_pool::{
    root_identity_of_write, writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    RetainedWrite, WrittenContents,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs";
const LARGEST_EXPANDED_SEGMENT_WRITES: usize = 6;
/// 根记录里 checkpoint_txg 的偏移（8 字节，小端）：与 `memory_pool::root_identity_written_by` 读的同一处。
const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;

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
            versions_without_a_file: &[],
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
        .join("../../Cargo.lock")
        .canonicalize()
        .expect("工作区的 Cargo.lock 在 crate 往上两层（变异复跑的源码副本里也有它）")
        .parent()
        .expect("Cargo.lock 所在的目录就是仓根")
        .to_path_buf()
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

/// 把版本表里覆盖写那一版（txg 4）的内容改坏一个字节：根落到 txg 4 的状态读回的内容对不上，这条流的尾巴判红。
fn with_the_overwritten_version_miswritten(mut recorded: RecordedFlow) -> RecordedFlow {
    let overwritten = recorded
        .versions
        .iter_mut()
        .find(|version| version.checkpoint_txg == CheckpointTxg(4))
        .expect("版本表里有覆盖写那一版");
    overwritten.content[0] ^= 0x01;
    recorded
}

/// 录入、核完一条流（一方、CPU），交回核对的计数。
fn record_and_check(
    store: &mut VerdictStore,
    recorded: &RecordedFlow,
    plan: &CrashPlan,
    version: &JudgeVersion,
) -> CheckingTally {
    record_crash_points(store, plan, version, &mut CoverageReport::default()).expect("录得进");
    check_recorded_blocks(
        store,
        &recorded.flow(),
        plan,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        version,
    )
    .expect("核得完")
}

/// 复用与剪枝的对照（C605；上游 `test-script-discipline.md`「判定可复用：按输入存，按前缀剪」的剪枝自检）：
/// 同一条小流 B 开着复用与前缀共享判一次（库里先有共前缀的流 A、和另一个判法版本下判过的 B），再关掉复用（B 独自进一份空库）、
/// 关掉前缀共享（不分崩溃点、不进库，整条流的状态逐个现判）各判一次，三份逐态相同。
/// 库里先放的两样都故意判红：A 的尾巴（版本表写错）与旧判法版本下的 B（版本表写错）。判法版本不进判定的键，B 就拿到旧版本的红——
/// 开着复用的那一份就与另两份不同。只差版本表、只差一条根槽写的那两种一对一的对照在后面几条用例里。
/// ignore 的点是按登记整批不判的一类、不算剪枝，这两条流没有 ignore 的点。要 `verdict-store` 特性：快档靠 C604 才编进来。
#[test]
fn reused_verdicts_equal_state_by_state_the_verdicts_judged_without_the_library_or_the_prefix_split(
) {
    let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
    let root = repository_root();
    let judging = PipelineJudgingCode::of_the_judges(&root, &toolchain).expect("判法");
    let version = judging.version();
    let mut older_judging = judging.clone();
    let mut flipped = *judging.digest();
    flipped[0] ^= 0x01;
    older_judging.replace_digest(flipped);
    let older_version = older_judging.version();
    // 尾巴都由公共模块设点：A 与 B 的尾巴同名、同父镜像，只差写下的内容
    let flow_a = with_the_overwritten_version_miswritten(record_flow(
        "reuse-control-a",
        &content_of(3, 4100),
        true,
    ));
    let flow_b_judged_by_an_older_version = with_the_overwritten_version_miswritten(record_flow(
        "reuse-control-b-older",
        &content_of(11, 4100),
        true,
    ));
    let flow_b = record_flow("reuse-control-b", &content_of(11, 4100), true);
    let plan_a = plan_crash_points(&flow_a.flow());
    let plan_b = plan_crash_points(&flow_b.flow());
    let state_count =
        layer0_plan_state_count(&flow_b.base, &flow_b.writes, &flow_b.segments, &expansion);

    // 开着复用与前缀共享：A（尾巴红）→ 旧判法版本下的 B（尾巴红）→ B
    let shared = TemporaryStore {
        directory: std::env::temp_dir().join(format!(
            "singlefs-crash-amplification-reuse-control-shared-{}",
            std::process::id()
        )),
    };
    let _ = std::fs::remove_dir_all(&shared.directory);
    let mut shared_store = VerdictStore::create_empty(&shared.directory).expect("建得了库");
    let checking_a = record_and_check(&mut shared_store, &flow_a, &plan_a, &version);
    let checking_older = record_and_check(
        &mut shared_store,
        &flow_b_judged_by_an_older_version,
        &plan_crash_points(&flow_b_judged_by_an_older_version.flow()),
        &older_version,
    );
    let mut report_b = CoverageReport::default();
    let recording_b =
        record_crash_points(&mut shared_store, &plan_b, &version, &mut report_b).expect("录得进");
    check_recorded_blocks(
        &mut shared_store,
        &flow_b.flow(),
        &plan_b,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &version,
    )
    .expect("核得完");
    let with_reuse =
        read_verdict_items_of_every_state(&shared_store, &plan_b, &version).expect("读得了判定");

    // 关掉复用：B 独自进一份空库
    let alone = TemporaryStore {
        directory: std::env::temp_dir().join(format!(
            "singlefs-crash-amplification-reuse-control-alone-{}",
            std::process::id()
        )),
    };
    let _ = std::fs::remove_dir_all(&alone.directory);
    let mut alone_store = VerdictStore::create_empty(&alone.directory).expect("建得了库");
    record_and_check(&mut alone_store, &flow_b, &plan_b, &version);
    let without_reuse =
        read_verdict_items_of_every_state(&alone_store, &plan_b, &version).expect("读得了判定");

    // 关掉前缀共享：不分崩溃点、不进库，整条流的状态按序号逐个现判
    let mut without_prefix_split: Vec<Option<Vec<String>>> = Vec::new();
    let stream_state_count = judge_layer0_state_range(
        &flow_b.base,
        &flow_b.writes,
        &flow_b.segments,
        flow_b.judged_root_index,
        &flow_b.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| without_prefix_split.push(Some(judgement.red_items())),
    );
    assert_eq!(
        stream_state_count, state_count,
        "逐个现判的整条流状态数与计划相同"
    );

    assert_eq!(
        u64::try_from(without_prefix_split.len()).expect("状态数"),
        state_count,
        "整条流逐个现判，每个状态一份"
    );
    assert_eq!(
        with_reuse.len(),
        without_prefix_split.len(),
        "开着复用读回的状态数"
    );
    assert_eq!(
        without_reuse.len(),
        without_prefix_split.len(),
        "关掉复用读回的状态数"
    );
    for (ordinal, judged_fresh) in without_prefix_split.iter().enumerate() {
        assert_eq!(
            &without_reuse[ordinal], judged_fresh,
            "第 {ordinal} 个状态：关掉复用（空库）与关掉前缀共享（逐个现判）的判定不同"
        );
        assert_eq!(
            &with_reuse[ordinal], judged_fresh,
            "第 {ordinal} 个状态：开着复用与前缀共享读回的判定与逐个现判的不同（复用了不该复用的判定）"
        );
    }

    // 这份对照不是空的：B 的前缀确实复用了 A 的判定，库里先放的两样确实是红的
    let prefix = prefix_length(&plan_b);
    let prefix_blocks: u64 = plan_b.points[..prefix]
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数"))
        .sum();
    assert!(prefix_blocks > 0, "前缀至少一块");
    assert_eq!(
        recording_b.blocks_already_judged,
        prefix_blocks,
        "开着复用时 B 的前缀全部复用、尾巴一块都不复用：{:?}",
        report_b.lines()
    );
    assert!(
        checking_a.red_states > 0,
        "A 的版本表写错，尾巴必须红：库里要有一份不该被 B 复用的红判定"
    );
    assert!(
        checking_older.red_states > 0,
        "旧判法版本下的 B 版本表写错，必须红：库里要有一份不该被新判法版本复用的红判定"
    );
    assert!(
        without_prefix_split
            .iter()
            .all(|items| items.as_ref().is_some_and(Vec::is_empty)),
        "健康的 B 每个状态都是绿的"
    );
    drop(shared_store);
    drop(alone_store);
    drop(shared);
    drop(alone);
}

/// 计划里每个点的身份（树上的节点），按点的次序。
fn identities(plan: &CrashPlan) -> Vec<String> {
    plan.points
        .iter()
        .map(|point| point.identity.clone())
        .collect()
}

/// 前缀那几个点的块数。
fn prefix_blocks(plan: &CrashPlan) -> u64 {
    plan.points[..prefix_length(plan)]
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数"))
        .sum()
}

/// 不经库、不分崩溃点，整条流的状态按序号逐个现判（关掉复用与前缀共享的那一份）。
fn every_state_judged_fresh(recorded: &RecordedFlow) -> Vec<Option<Vec<String>>> {
    let state_count = layer0_plan_state_count(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        &expansion,
    );
    let mut judged: Vec<Option<Vec<String>>> = Vec::new();
    let stream_state_count = judge_layer0_state_range(
        &recorded.base,
        &recorded.writes,
        &recorded.segments,
        recorded.judged_root_index,
        &recorded.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| judged.push(Some(judgement.red_items())),
    );
    assert_eq!(
        stream_state_count, state_count,
        "逐个现判的整条流状态数与计划相同"
    );
    judged
}

/// 同一份库、同一判法版本：先录、核 `first`，再录、核 `later`；交回 `later` 录入时的计数与它在库里读回的逐态判定。
fn judge_in_one_library_and_read_back_the_later(
    first: &RecordedFlow,
    later: &RecordedFlow,
    tag: &str,
) -> (RecordingTally, Vec<Option<Vec<String>>>) {
    let toolchain = Layer0ToolchainIdentity::of_the_cargo_running_this_test();
    let version = PipelineJudgingCode::of_the_judges(&repository_root(), &toolchain)
        .expect("判法")
        .version();
    let temporary = TemporaryStore {
        directory: std::env::temp_dir().join(format!(
            "singlefs-crash-amplification-{tag}-{}",
            std::process::id()
        )),
    };
    let _ = std::fs::remove_dir_all(&temporary.directory);
    let mut store = VerdictStore::create_empty(&temporary.directory).expect("建得了库");
    record_and_check(
        &mut store,
        first,
        &plan_crash_points(&first.flow()),
        &version,
    );
    let plan_later = plan_crash_points(&later.flow());
    let recording_later = record_crash_points(
        &mut store,
        &plan_later,
        &version,
        &mut CoverageReport::default(),
    )
    .expect("录得进");
    check_recorded_blocks(
        &mut store,
        &later.flow(),
        &plan_later,
        &WeightedShares::new(vec![1]),
        0,
        &UnitChecksumsOn::CpuOnly,
        &version,
    )
    .expect("核得完");
    let read_back =
        read_verdict_items_of_every_state(&store, &plan_later, &version).expect("读得了判定");
    drop(store);
    drop(temporary);
    (recording_later, read_back)
}

/// 同一次覆盖写录两遍：一遍版本表照实，一遍把覆盖写那一版改坏一个字节。写表、段、身份逐个相同，只差版本表。
fn a_healthy_flow_and_its_twin_with_a_miswritten_version_table(
    tag: &str,
) -> (RecordedFlow, RecordedFlow) {
    let healthy = record_flow(&format!("{tag}-healthy"), &content_of(11, 4100), true);
    let miswritten = with_the_overwritten_version_miswritten(record_flow(
        &format!("{tag}-miswritten"),
        &content_of(11, 4100),
        true,
    ));
    (healthy, miswritten)
}

/// R1：只差版本表的一对，版本表写错的那一条先进库、健康的后进（同一判法版本）：健康的那一条读回的每个状态与它现判的相同——
/// 尾巴够得着覆盖写那一版，不复用写错那一条的红；前缀够不着它，判器读到的真相同，照样复用。
#[test]
fn a_healthy_flow_judged_after_its_twin_with_a_miswritten_version_table_reads_back_its_own_verdicts(
) {
    let (healthy, miswritten) =
        a_healthy_flow_and_its_twin_with_a_miswritten_version_table("version-pair-healthy-later");
    let plan_healthy = plan_crash_points(&healthy.flow());
    assert!(healthy.writes == miswritten.writes, "两遍录的写表相同");
    assert_eq!(
        identities(&plan_crash_points(&miswritten.flow())),
        identities(&plan_healthy),
        "身份逐个相同：这一对只差版本表"
    );
    let (recording, read_back) = judge_in_one_library_and_read_back_the_later(
        &miswritten,
        &healthy,
        "version-pair-healthy-later",
    );
    let fresh = every_state_judged_fresh(&healthy);
    assert_eq!(
        read_back, fresh,
        "健康的那一条读回的判定与现判的不同：复用了版本表写错那一条的判定"
    );
    assert_ne!(
        every_state_judged_fresh(&miswritten),
        fresh,
        "这一对判得不同：版本表写错的那一条有红"
    );
    assert_eq!(
        recording.blocks_already_judged,
        prefix_blocks(&plan_healthy),
        "前缀的块照样复用、尾巴一块都不复用"
    );
}

/// R1：只差版本表的一对，健康的那一条先进库、版本表写错的后进（同一判法版本）：写错的那一条读回的每个状态与它现判的相同（该红的红），
/// 尾巴不复用健康那一条的绿；前缀照样复用。
#[test]
fn a_flow_with_a_miswritten_version_table_judged_after_its_healthy_twin_reads_back_its_own_verdicts(
) {
    let (healthy, miswritten) = a_healthy_flow_and_its_twin_with_a_miswritten_version_table(
        "version-pair-miswritten-later",
    );
    let plan_miswritten = plan_crash_points(&miswritten.flow());
    assert!(healthy.writes == miswritten.writes, "两遍录的写表相同");
    assert_eq!(
        identities(&plan_crash_points(&healthy.flow())),
        identities(&plan_miswritten),
        "身份逐个相同：这一对只差版本表"
    );
    let (recording, read_back) = judge_in_one_library_and_read_back_the_later(
        &healthy,
        &miswritten,
        "version-pair-miswritten-later",
    );
    let fresh = every_state_judged_fresh(&miswritten);
    assert_eq!(
        read_back, fresh,
        "版本表写错的那一条读回的判定与现判的不同：复用了健康那一条的判定"
    );
    assert_ne!(
        every_state_judged_fresh(&healthy),
        fresh,
        "这一对判得不同：版本表写错的那一条有红"
    );
    assert_eq!(
        recording.blocks_already_judged,
        prefix_blocks(&plan_miswritten),
        "前缀的块照样复用、尾巴一块都不复用"
    );
}

/// R1：判器另读的输入只按这个点够得着的算——只差覆盖写那一版时，前缀每个点的判器输入摘要相同（前缀共享保留），尾巴每个点都不同。
#[test]
fn a_version_the_prefix_cannot_reach_leaves_the_judge_inputs_of_the_prefix_alone() {
    let (healthy, miswritten) =
        a_healthy_flow_and_its_twin_with_a_miswritten_version_table("version-reach");
    let plan_healthy = plan_crash_points(&healthy.flow());
    let plan_miswritten = plan_crash_points(&miswritten.flow());
    let prefix = prefix_length(&plan_healthy);
    assert!(prefix < plan_healthy.points.len(), "有尾巴");
    for index in 0..prefix {
        assert_eq!(
            plan_healthy.points[index].judge_inputs, plan_miswritten.points[index].judge_inputs,
            "前缀第 {index} 个点够不着覆盖写那一版，判器输入相同"
        );
    }
    for index in prefix..plan_healthy.points.len() {
        assert_ne!(
            plan_healthy.points[index].judge_inputs, plan_miswritten.points[index].judge_inputs,
            "尾巴第 {index} 个点（{}）够得着覆盖写那一版，判器输入不同",
            plan_healthy.points[index].name
        );
    }
}

/// R1：被判的根槽写换了位置（换成写表里第一条根槽写），树（身份）不动，管它前后的点判器另读的输入跟着变。
#[test]
fn moving_the_judged_root_slot_write_changes_the_judge_inputs_but_not_the_tree() {
    let recorded = record_flow("judged-root-moved", &content_of(11, 4100), true);
    let plan = plan_crash_points(&recorded.flow());
    let first_root = recorded
        .writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    assert_ne!(
        first_root, recorded.judged_root_index,
        "第一条根槽写不是被判的那一条"
    );
    let mut moved_flow = recorded.flow();
    moved_flow.judged_root_index = first_root;
    let moved_plan = plan_crash_points(&moved_flow);
    assert_eq!(
        identities(&moved_plan),
        identities(&plan),
        "被判的根槽写不动树"
    );
    assert!(
        moved_plan
            .points
            .iter()
            .zip(&plan.points)
            .any(|(moved, original)| moved.judge_inputs != original.judge_inputs),
        "被判的根槽写换了位置，判器另读的输入要跟着变"
    );
}

/// M3：只差一条根槽写的一对（覆盖写那一次的根槽写改坏第一个字节，根身份那几个字节不动）：管这条写的那个点父镜像相同、判器另读的输入相同、
/// 计划哈希相同，只有这一步的写表不同、有状态。健康的先进库，改坏的后进：改坏的那一条读回的每个状态与它现判的相同。
/// 两条断言各拦一种改坏（test-script-align-code-r2 判决 J3）：
/// - 键那一条（「这一步的写表不同，复用键不同」）拦复用键不罩写表（变异表「崩溃放量 复用键不罩写表」）。管根槽写的那个点只管一段 1 次写，
///   真子集只有「根槽写没落」那一个状态，两边盘上逐字节相同、判得一样，读回对不对分不出这一种改坏，只能直接比键；
/// - 判定那一条（读回与现判逐态相同）拦块键分不开这一对（变异表「崩溃放量 块键只接点名（M3 读回）」）：改坏的那一条的红落在下一个点
///   （根槽写已落），两条流同名的点块键一样就读回健康那一条的绿。
///
/// 最后一条只核这一对不是空对照：改坏的那一条有红、红在管根槽写的那个点之后。
#[test]
fn a_twin_whose_root_slot_write_alone_differs_reads_back_its_own_verdicts() {
    let healthy = record_flow("root-write-twin-healthy", &content_of(11, 4100), true);
    let mut twin = record_flow("root-write-twin-flipped", &content_of(11, 4100), true);
    let root_index = twin.judged_root_index;
    match &mut twin.writes[root_index].contents {
        WrittenContents::Bytes(bytes) => bytes[0] ^= 0x01,
        WrittenContents::Zeros { .. } => panic!("根槽写是普通写，带着字节"),
    }
    let plan_healthy = plan_crash_points(&healthy.flow());
    let plan_twin = plan_crash_points(&twin.flow());
    let owner = plan_twin
        .points
        .iter()
        .position(|point| {
            point
                .segments
                .iter()
                .any(|segment| twin.segments[*segment].contains(&root_index))
        })
        .expect("根槽写归某个点");
    let (healthy_point, twin_point) = (&plan_healthy.points[owner], &plan_twin.points[owner]);
    assert!(
        !twin_point.blocks().is_empty(),
        "管根槽写的那个点（{}）有状态",
        twin_point.name
    );
    assert_eq!(
        twin_point.judge_inputs, healthy_point.judge_inputs,
        "判器另读的输入相同"
    );
    assert_eq!(
        twin_point.plan_hash, healthy_point.plan_hash,
        "计划哈希相同：只剩复用键分得开这一对"
    );
    assert_ne!(
        twin_point.reuse_key, healthy_point.reuse_key,
        "这一步的写表不同，复用键不同"
    );
    let (_, read_back) =
        judge_in_one_library_and_read_back_the_later(&healthy, &twin, "root-write-twin");
    let fresh_twin = every_state_judged_fresh(&twin);
    assert_eq!(
        read_back, fresh_twin,
        "根槽写改坏的那一条读回的判定与现判的不同：复用了健康那一条的判定"
    );
    let after_the_owner = usize::try_from(twin_point.ordinals.end).expect("序号装得进 usize");
    let fresh_healthy = every_state_judged_fresh(&healthy);
    assert_eq!(
        fresh_healthy[..after_the_owner],
        fresh_twin[..after_the_owner],
        "到管根槽写的那个点为止（含它）这一对判得一样：那几个状态里改坏的根槽写都不在盘上"
    );
    assert_ne!(
        fresh_healthy[after_the_owner..],
        fresh_twin[after_the_owner..],
        "这一对不是空对照：根槽写改坏的那一条在管它的那个点之后（根槽写已落）有红"
    );
}

/// 改坏流里最后一次写（覆盖写那一次的系统配置槽写），交回改过的写表。
fn writes_with_the_last_write_flipped(writes: &[RetainedWrite]) -> Vec<RetainedWrite> {
    let mut flipped = writes.to_vec();
    match &mut flipped.last_mut().expect("写表不空").contents {
        WrittenContents::Bytes(bytes) => bytes[0] ^= 0x01,
        WrittenContents::Zeros { .. } => panic!("流里最后一次写是系统配置槽写，带着字节"),
    }
    flipped
}

/// J1 退成整段写表那一支：根槽写的 txg 不沿写表严格递增时（覆盖写那一条的 txg 改成与前一条根槽写相同），推不出「点之后的写判器读不到」，
/// 每个点的判器另读的输入都罩着它之后的整段写表——只改流里最后一次写，前面每个点的判器输入都跟着变、树（身份）不动。
/// 同一对改动放在 txg 照实严格递增的流上，前面每个点的判器输入都不变（之后的写只经没做完的那次发布的根槽写读得到）。
#[test]
fn root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_into_the_judge_inputs(
) {
    let recorded = record_flow("txgs-not-strictly-increasing", &content_of(11, 4100), true);
    let root_writes: Vec<usize> = recorded
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    let [.., previous_root, last_root] = root_writes.as_slice() else {
        panic!("至少两条根槽写：{root_writes:?}");
    };
    let last_write = recorded.writes.len() - 1;
    assert!(
        *last_root < last_write,
        "流里最后一次写在最后一条根槽写之后，前面每个点都在它之前"
    );
    let previous_txg = root_identity_of_write(&recorded.writes[*previous_root]).1;
    let mut not_increasing = recorded.writes.clone();
    match &mut not_increasing[*last_root].contents {
        WrittenContents::Bytes(bytes) => {
            bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                .copy_from_slice(&previous_txg.0.to_le_bytes());
        }
        WrittenContents::Zeros { .. } => panic!("根槽写是普通写，带着字节"),
    }
    assert_eq!(
        root_identity_of_write(&not_increasing[*last_root]).1,
        previous_txg,
        "最后一条根槽写的 txg 与前一条相同：不是严格递增"
    );
    let flow_over = |writes: &[RetainedWrite]| -> CrashPlan {
        plan_crash_points(&CrashFlow {
            base: &recorded.base,
            writes,
            segments: &recorded.segments,
            judged_root_index: recorded.judged_root_index,
            versions: &recorded.versions,
            versions_without_a_file: &[],
            expansion: &expansion,
            crash_points: recorded.crash_points.clone(),
        })
    };
    let increasing = flow_over(&recorded.writes);
    let increasing_with_the_last_write_flipped =
        flow_over(&writes_with_the_last_write_flipped(&recorded.writes));
    let not_increasing_plan = flow_over(&not_increasing);
    let not_increasing_with_the_last_write_flipped =
        flow_over(&writes_with_the_last_write_flipped(&not_increasing));
    let points_before_the_last = increasing.points.len() - 1;
    assert!(points_before_the_last >= 2, "最后一次写之前至少两个点");
    for index in 0..points_before_the_last {
        assert_eq!(
            identities(&not_increasing_plan)[index],
            identities(&not_increasing_with_the_last_write_flipped)[index],
            "只改最后一次写，第 {index} 个点的身份不动"
        );
        assert_ne!(
            not_increasing_plan.points[index].judge_inputs,
            not_increasing_with_the_last_write_flipped.points[index].judge_inputs,
            "txg 不严格递增：第 {index} 个点（{}）的判器输入罩着之后的整段写表，最后一次写改了它跟着变",
            not_increasing_plan.points[index].name
        );
        assert_eq!(
            increasing.points[index].judge_inputs,
            increasing_with_the_last_write_flipped.points[index].judge_inputs,
            "txg 严格递增：第 {index} 个点（{}）读不到最后一次写，判器输入不变",
            increasing.points[index].name
        );
    }
}

/// 「窄一」（用户 2026-10-02 定）：基镜像是崩溃状态的流排计划时被拒。造法照 test-script-align-code-r3 攻方 L1：
/// 两流用例那条流（mkfs → 取号加暖机 → 新池新建文件 txg 3 → 覆盖写 txg 4）崩在覆盖写的 journal 记录落盘之后、根槽写落盘之前，
/// 这个崩溃状态当基镜像；照它可写挂载（实例 2）、再覆盖写一次，录成一条新流。照 journal 恢复这个基镜像会落到 txg 4，基镜像里最新的根是 txg 3：
/// B0 按基镜像里的根截版本表会截掉判器读得到的 txg 4 那一版（攻方在这条流的第一个点上量到同键而 8 态判定不同）。
#[test]
#[should_panic(expected = "基镜像照 journal 恢复会越过它里面最新的根，B0 的前提不成立")]
fn a_flow_whose_base_image_is_a_crash_state_with_a_journal_record_past_its_newest_root_is_refused_at_planning(
) {
    let overwrite_before_the_crash = content_of(11, 4100);
    let overwrite_after_the_mount = content_of(23, 3000);
    let crashed = record_flow("crashed-base", &overwrite_before_the_crash, true);
    let root_slot_write_that_did_not_land = crashed.judged_root_index;
    assert!(
        crashed.writes[..root_slot_write_that_did_not_land]
            .iter()
            .rev()
            .take_while(|write| write.kind != StepKind::RootRecordFua)
            .any(|write| write.kind == StepKind::JournalRecord),
        "覆盖写那一次的根槽写之前有它的 journal 记录：崩溃状态里记录落了、根没落"
    );
    let mut base = crashed.base.clone();
    base.apply_writes(&crashed.writes[..root_slot_write_that_did_not_land]);

    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&base, &[], &[], &stream);
    let mut recorder = CrashPointRecorder::new(stream.clone(), 0);
    let mounted = recorder.step(file!(), "mount_writable_after_the_crash", || {
        mount_writable(&parameters(), &mut devices).expect("崩溃之后可写挂载")
    });
    let mut allocator = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("重开之后现行那一版带文件");
    recorder.step(file!(), "publish_overwrite", || {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            &current,
            FirstFile {
                content: &overwrite_after_the_mount,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            InstanceGeneration(2),
        )
        .expect("重开之后覆盖写");
    });
    let operations = stream.retained_operations();
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes(&operations, &geometry());
    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
    let root_slot_writes: Vec<(InstanceGeneration, CheckpointTxg)> = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(root_identity_of_write)
        .collect();
    let last_root = *root_slot_writes.last().expect("重开之后有根槽写");
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽写");
    // 版本表照实：崩前两版（txg 4 由 journal 记录重建），重开写出的每一代根读回的都是覆盖写 A，最后那次覆盖写那一代读回 B
    let mut versions = crashed.versions.clone();
    for (instance, checkpoint_txg) in root_slot_writes {
        if versions
            .iter()
            .any(|version| version.instance == instance && version.checkpoint_txg == checkpoint_txg)
        {
            continue;
        }
        let content = if (instance, checkpoint_txg) == last_root {
            overwrite_after_the_mount.clone()
        } else {
            overwrite_before_the_crash.clone()
        };
        versions.push(PublishedVersion {
            instance,
            checkpoint_txg,
            content,
        });
    }
    let _refused = plan_crash_points(&CrashFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index,
        versions: &versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points,
    });
}
