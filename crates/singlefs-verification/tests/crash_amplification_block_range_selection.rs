//! 验证包模块：crash、crash_point_replay_progress、crash_identity、crash_library、crash_amplification
//! 崩溃放量按块序号区间选块（`crash_amplification::BLOCK_RANGE_ENVIRONMENT_VARIABLE`）：区间的写法与越界判错、起块等于止块的空区间一块不选、
//! 几段区间拼起来恰好罩全一条流的块且两两不相交、与份同给取交集、没点到的流交错（纯函数，不开库）；别的读份的路设了区间就拒；
//! 经 `run_from_environment` 在新池新建文件那条流上真跑：没点到这条流在录入之前交错，给了区间只判、只对账本片的块（本片判齐就不报没判的块），
//! 这台机器一块没选中（空区间、份与区间没有交集）的那一趟不判、照常交回、行上照打区间字段，没设区间、只按份分而这一份一块没领到的那一趟
//! 不走那一支、照判完那一趟交回，只录那一趟也打区间那一行，
//! 本片里存着的红照样读成违例、几片读到的违例加起来恰好是整条流的，不给区间时行与对账照整条流。
//! 测试共用的断言在设了区间时只对本片判：观察者看过状态的那一句（`common_crash_point_observers`），这一片里没有它节点上的块不要求、
//! 有块照旧要求（看 0 个状态、没挂上都红）；固定脚本两条放量共用的「判齐、核过、再跑一趟全部复用」（`common_blocks_judged_and_reused_by_the_run`），
//! 几片各自判齐不误红，这一片少判一块照样红。
//! 库是临时 prune-server 里的一次性库（`crash_library::DisposableLibrary`）；经环境跑的用例自己清、自己还原环境（公共模块的环境守卫）。

mod common;
mod common_blocks_judged_and_reused_by_the_run;
mod common_crash_point_observers;
mod common_crash_points;

use std::ffi::OsStr;
use std::path::PathBuf;

use common_blocks_judged_and_reused_by_the_run::{
    assert_the_run_judged_verified_and_reused_its_blocks, blocks_and_states_of_the_block_range,
};
use common_crash_point_observers::{
    blocks_selected_here_on_the_nodes, states_seen_by_the_observer_as_the_block_range_requires,
    whole_flow_assertion_is_left_to_the_run_without_a_block_range,
};
use common_crash_points::{CrashAmplificationEnvironment, NewPoolFileCreationAsOneCrashPoint};
use singlefs_core::recovery::RecoveryReport;
use singlefs_verification::crash::CrashPointReplayObserverCounts;
use singlefs_verification::crash_amplification::{
    block_range_line, blocks_of_this_share_in_the_block_range, plan_crash_points,
    repository_relative_file, run_from_environment, BlockRangeOfTheFlow, BlockRangesOfTheFlows,
    CrashFlow, CrashPlan, EnvironmentRun, ExpandUpToWritesPerSegment, GpuCardsSource, HostShare,
    NodeOfAnObserver, ObserveACrashState, ObserverCode, ObserverOnTheNodes, ObservingOfTheRun,
    WeightedShares, WrittenBlockRangeOfTheFlow, BLOCK_RANGE_ENVIRONMENT_VARIABLE,
    END_OF_FLOW_NODE_NAME,
};
use singlefs_verification::crash_identity::ObserverName;
use singlefs_verification::crash_library::{
    DisposableLibrary, PairingBlocksShare, RUN_STAGE_ENVIRONMENT_VARIABLE,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE;
use singlefs_verification::memory_pool::CrashImage;
use singlefs_verification::sha256::DIGEST_BYTES;

/// 纯函数那几条用的流：名字与块数（不开库，块数随手取一个不整除份数的）。
const SMALL_FLOW_NAME: &str = "small-flow";
const BLOCKS_OF_THE_SMALL_FLOW: u64 = 11;

/// 真跑那几条的流名：来历表里崩溃放量那一路「两处都不登」的那一条流名，`::` 之后是这条用例的标签（来历表只认 `::` 之前那一截，
/// 流名不进块键）。区间变量按 `::` 之后也算在内的整个名字点流。
const FLOW_NAME_JUDGED_BY_RANGES: &str = "line-fields::block-range-selection";
const FLOW_NAME_NOT_RUN_HERE: &str = "line-fields::block-range-selection-not-run";
const FLOW_NAME_WITH_A_WRONG_VERSION_TABLE: &str = "line-fields::block-range-selection-red";
const FLOW_NAME_SELECTING_NO_BLOCK: &str = "line-fields::block-range-selection-no-block";
const FLOW_NAME_RECORDED_ONLY: &str = "line-fields::block-range-selection-recorded-only";
const FLOW_NAME_OF_A_SHARE_WITHOUT_BLOCKS: &str =
    "line-fields::block-range-selection-share-without-blocks";
/// 两台分份时领哪一份的变量（`crash_amplification::host_share_and_weights_of_the_share_variables` 读它）。
const SHARE_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_SHARE";
/// 几台机器的权重（同一个函数读它）：权重 0 的那一份一块都分不到。
const HOST_WEIGHTS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS";
/// 真跑那几条的展开上限：只为一条流有几块可判，取得小。
const EXPAND_UP_TO_OF_THE_FLOW: usize = 2;

/// `small-flow:<起块>-<止块>` 读成这条小流的那一片。
fn range_of_the_small_flow(start: u64, end: u64) -> BlockRangeOfTheFlow {
    BlockRangesOfTheFlows::parse(&format!("{SMALL_FLOW_NAME}:{start}-{end}"))
        .expect("写法对")
        .range_of_the_flow(SMALL_FLOW_NAME, BLOCKS_OF_THE_SMALL_FLOW)
        .expect("不越界")
}

/// 几段首尾相接的区间罩住 `0..11`：每段选出的块并起来恰好是整条流的块、两两不相交；两台分份时每段再按份分，（段，份）选出的块同样恰好各一次。
#[test]
fn block_ranges_that_tile_a_flow_select_every_block_of_the_flow_exactly_once() {
    let tiling = [(0, 4), (4, 5), (5, BLOCKS_OF_THE_SMALL_FLOW)];
    for weights in [vec![1], vec![1, 1], vec![3, 1]] {
        let host_shares = WeightedShares::new(weights.clone());
        let mut times_selected =
            vec![0usize; usize::try_from(BLOCKS_OF_THE_SMALL_FLOW).expect("小")];
        for (start, end) in tiling {
            let block_range = range_of_the_small_flow(start, end);
            for share_index in 0..weights.len() {
                for block in blocks_of_this_share_in_the_block_range(
                    BLOCKS_OF_THE_SMALL_FLOW,
                    &host_shares,
                    share_index,
                    &block_range,
                ) {
                    assert!(
                        (start..end).contains(&block),
                        "区间 {start}-{end} 选出了区间外的块 {block}（权重 {weights:?}）"
                    );
                    times_selected[usize::try_from(block).expect("小")] += 1;
                }
            }
        }
        assert_eq!(
            times_selected,
            vec![1; times_selected.len()],
            "每一块恰好被一段区间、一份选中一次（权重 {weights:?}）"
        );
    }
}

/// 区间半开：起块大于止块判错；起块小于止块照收。
#[test]
fn a_block_range_whose_start_is_above_its_end_is_refused() {
    for written in ["small-flow:4-3", "small-flow:11-0"] {
        let refusal = BlockRangesOfTheFlows::parse(written).expect_err("起块大于止块要判错");
        assert!(
            refusal.contains("起块不能大于止块"),
            "{written} 判错的理由要说起块大于止块：{refusal}"
        );
    }
    assert_eq!(
        range_of_the_small_flow(3, 4),
        BlockRangeOfTheFlow::BlocksInTheRange(3..4)
    );
}

/// 起块等于止块是显式的空区间：照收，一块不选、一块不数，行上照打 `block_range=<块>-<块> blocks_of_the_flow=<块数>`；
/// 止块等于块数的空区间也收（止块不过块数）。
#[test]
fn an_empty_block_range_is_accepted_and_selects_no_block() {
    for block in [0, 3, BLOCKS_OF_THE_SMALL_FLOW] {
        let block_range = range_of_the_small_flow(block, block);
        assert_eq!(
            block_range,
            BlockRangeOfTheFlow::BlocksInTheRange(block..block)
        );
        for weights in [vec![1], vec![1, 1]] {
            for share_index in 0..weights.len() {
                assert!(
                    blocks_of_this_share_in_the_block_range(
                        BLOCKS_OF_THE_SMALL_FLOW,
                        &WeightedShares::new(weights.clone()),
                        share_index,
                        &block_range,
                    )
                    .is_empty(),
                    "空区间 {block}-{block} 一块不选（权重 {weights:?}，份 {share_index}）"
                );
            }
        }
        assert!(
            (0..BLOCKS_OF_THE_SMALL_FLOW).all(|sequence| !block_range.holds(sequence)),
            "空区间对账与读违例一块不数"
        );
        assert_eq!(
            block_range.fields_of_the_line(BLOCKS_OF_THE_SMALL_FLOW),
            format!(" block_range={block}-{block} blocks_of_the_flow=11")
        );
    }
}

/// 止块过了这条流的块数判错（越界，不悄悄截掉），空区间同样判；止块等于块数照收。
#[test]
fn a_block_range_past_the_last_block_of_the_flow_is_refused() {
    for written in ["small-flow:9-12", "small-flow:11-12", "small-flow:12-12"] {
        let refusal = BlockRangesOfTheFlows::parse(written)
            .expect("写法对")
            .range_of_the_flow(SMALL_FLOW_NAME, BLOCKS_OF_THE_SMALL_FLOW)
            .expect_err("止块过了块数要判错");
        assert!(refusal.contains("越界"), "{written}：{refusal}");
    }
    assert_eq!(
        range_of_the_small_flow(9, BLOCKS_OF_THE_SMALL_FLOW),
        BlockRangeOfTheFlow::BlocksInTheRange(9..BLOCKS_OF_THE_SMALL_FLOW)
    );
}

/// 不是 `<流名>:<起块>-<止块>` 的写法、同一条流写了两段，都判错。
#[test]
fn a_block_range_written_in_another_form_is_refused() {
    for written in [
        "",
        "small-flow",
        ":0-1",
        "small-flow:0",
        "small-flow:zero-1",
        "small-flow:0-1,",
        "small-flow:0-1,small-flow:1-2",
    ] {
        assert!(
            BlockRangesOfTheFlows::parse(written).is_err(),
            "「{written}」要判错"
        );
    }
    assert!(BlockRangesOfTheFlows::parse("small-flow:0-1,other-flow:1-2").is_ok());
}

/// 区间变量设了、没点到这条流：交错，错误文字点名变量、这条流的名字与变量里写了哪几条流（流名写错一个字不许绿着一块不判）；
/// 没设区间时行上不多打字段、不打区间那一行。
#[test]
fn a_flow_not_named_in_the_block_ranges_is_refused_naming_the_variable_the_flow_and_the_flows_written(
) {
    let block_ranges =
        BlockRangesOfTheFlows::parse("other-flow:0-2,third-flow:1-1").expect("写法对");
    let refusal = block_ranges
        .range_of_the_flow(SMALL_FLOW_NAME, BLOCKS_OF_THE_SMALL_FLOW)
        .expect_err("没点到的流要交错");
    for named in [
        BLOCK_RANGE_ENVIRONMENT_VARIABLE,
        SMALL_FLOW_NAME,
        "other-flow",
        "third-flow",
    ] {
        assert!(refusal.contains(named), "错误文字要点名 {named}：{refusal}");
    }
    assert_eq!(
        block_ranges.written_range_of_the_flow(SMALL_FLOW_NAME),
        Err(refusal),
        "录入之前那一道与排出计划之后那一道判的是同一件事"
    );
    assert_eq!(
        block_ranges.written_range_of_the_flow("third-flow"),
        Ok(WrittenBlockRangeOfTheFlow::BlocksOfTheFlow(1..1))
    );
    assert_eq!(
        BlockRangeOfTheFlow::EveryBlockOfTheFlow.fields_of_the_line(BLOCKS_OF_THE_SMALL_FLOW),
        ""
    );
    assert_eq!(
        block_range_line(
            SMALL_FLOW_NAME,
            HostShare { index: 0, count: 1 },
            &BlockRangeOfTheFlow::EveryBlockOfTheFlow,
            BLOCKS_OF_THE_SMALL_FLOW,
            11
        ),
        None
    );
}

/// 设了区间时每一趟打的那一行：流名、份、区间、块数与这台机器选中的块数都在上面。
#[test]
fn the_block_range_line_names_the_flow_the_share_the_range_and_the_blocks_of_the_flow() {
    assert_eq!(
        block_range_line(
            SMALL_FLOW_NAME,
            HostShare { index: 1, count: 2 },
            &range_of_the_small_flow(3, 8),
            BLOCKS_OF_THE_SMALL_FLOW,
            2
        )
        .as_deref(),
        Some(
            "CRASH_AMPLIFICATION_BLOCK_RANGE flow=small-flow share=1/2 block_range=3-8 blocks_of_the_flow=11 blocks_selected_here=2"
        )
    );
}

/// 区间与份同给取交集：某一份在区间里选的块，就是它不给区间时选的块里落在区间里的那些（块序号照整条流轮转）。
#[test]
fn a_share_and_a_block_range_together_select_the_blocks_of_the_share_inside_the_range() {
    let host_shares = WeightedShares::new(vec![1, 1]);
    let block_range = range_of_the_small_flow(3, 8);
    for share_index in 0..2 {
        let in_the_whole_flow = blocks_of_this_share_in_the_block_range(
            BLOCKS_OF_THE_SMALL_FLOW,
            &host_shares,
            share_index,
            &BlockRangeOfTheFlow::EveryBlockOfTheFlow,
        );
        let in_the_range = blocks_of_this_share_in_the_block_range(
            BLOCKS_OF_THE_SMALL_FLOW,
            &host_shares,
            share_index,
            &block_range,
        );
        let expected: Vec<u64> = in_the_whole_flow
            .into_iter()
            .filter(|block| (3..8).contains(block))
            .collect();
        assert!(!expected.is_empty(), "两份在 3-8 里各有块");
        assert_eq!(in_the_range, expected, "份 {share_index}");
    }
}

/// 别的读份的路（这里取对拍的分份 `PairingBlocksShare::of_this_process`）设了区间变量就拒、点名那个变量；没设照读份。
#[test]
fn paths_reading_the_share_outside_the_amplification_pipeline_refuse_a_block_range() {
    let refusal = {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[(
            BLOCK_RANGE_ENVIRONMENT_VARIABLE,
            OsStr::new("small-flow:0-1"),
        )]);
        PairingBlocksShare::of_this_process().expect_err("对拍的分份不认块区间，设了就拒")
    };
    assert!(
        refusal.contains(BLOCK_RANGE_ENVIRONMENT_VARIABLE),
        "拒的理由点名区间变量：{refusal}"
    );
    {
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        PairingBlocksShare::of_this_process().expect("没设区间照读份");
    }
}

/// 真跑那几条的库与配置：一份一次性库，它那份临时目录里放一份空的多机配置（单机 CPU）。
struct EnvironmentRunDirectory {
    library: DisposableLibrary,
    configuration: PathBuf,
}

impl EnvironmentRunDirectory {
    fn new(tag: &str) -> Self {
        let library = DisposableLibrary::new(tag).unwrap_or_else(|refusal| panic!("{refusal}"));
        let configuration = library.join("multi-host.env");
        std::fs::write(&configuration, "").expect("写得了空配置");
        Self {
            library,
            configuration,
        }
    }

    /// 在清掉一趟读的全部变量、只设了多机配置与（给了的话）块区间的环境里跑 `run`，跑完还原环境。
    fn run_with_the_block_range<Outcome>(
        &self,
        block_range: Option<String>,
        run: impl FnOnce() -> Outcome,
    ) -> Outcome {
        self.run_with_the_block_range_and_these_variables(block_range, &[], run)
    }

    /// 同 [`Self::run_with_the_block_range`]，另设 `variables` 里的几个变量（份、选段）。
    fn run_with_the_block_range_and_these_variables<Outcome>(
        &self,
        block_range: Option<String>,
        variables: &[(&str, &OsStr)],
        run: impl FnOnce() -> Outcome,
    ) -> Outcome {
        let mut settings: Vec<(&str, &OsStr)> = vec![(
            MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
            self.configuration.as_os_str(),
        )];
        if let Some(text) = &block_range {
            settings.push((BLOCK_RANGE_ENVIRONMENT_VARIABLE, OsStr::new(text.as_str())));
        }
        settings.extend_from_slice(variables);
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
        run()
    }
}

/// 经测试共用的跑法跑一趟（健康的流断言一个红都没有、本片判齐时库里本片没有红），交回结果与那一行。
fn amplify_through_the_common_module(
    prepared: &NewPoolFileCreationAsOneCrashPoint,
    flow_name: &str,
) -> (EnvironmentRun, String) {
    let (run, line) = common_crash_points::amplify_prepared_flow_and_hand_back_its_line(
        flow_name,
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        prepared.crash_points.clone(),
        EXPAND_UP_TO_OF_THE_FLOW,
    );
    (run, line.expect("判了的那一趟打一行"))
}

/// 不经测试共用的断言、直接调 `run_from_environment` 跑一趟（版本表写错的流要红，共用的那一层会当场断言失败）。
fn run_from_environment_directly(
    prepared: &NewPoolFileCreationAsOneCrashPoint,
    flow_name: &str,
) -> Result<EnvironmentRun, String> {
    run_from_environment_reporting_an_expansion_limit(
        prepared,
        flow_name,
        ExpandUpToWritesPerSegment(EXPAND_UP_TO_OF_THE_FLOW),
    )
}

/// 同 [`run_from_environment_directly`]，报给 `run_from_environment` 的展开上限另给（流自己照 `EXPAND_UP_TO_OF_THE_FLOW` 展开）：
/// 报的与流的不同时，那一趟在任何录入之前拒。
fn run_from_environment_reporting_an_expansion_limit(
    prepared: &NewPoolFileCreationAsOneCrashPoint,
    flow_name: &str,
    reported_limit: ExpandUpToWritesPerSegment,
) -> Result<EnvironmentRun, String> {
    let flow_limit = ExpandUpToWritesPerSegment(EXPAND_UP_TO_OF_THE_FLOW);
    let expansion =
        move |_segment_index: usize, segment: &[usize]| flow_limit.expansion_of(segment);
    let flow = CrashFlow {
        base: &prepared.base,
        writes: &prepared.writes,
        segments: &prepared.segments,
        judged_root_index: prepared.judged_root_index,
        versions: &prepared.versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points: prepared.crash_points.clone(),
    };
    run_from_environment(
        &flow,
        flow_name,
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        reported_limit,
    )
}

/// 区间变量设了、没点到这条流：`run_from_environment` 在录入之前交错，错误文字点名变量、这条流的名字与变量里写的那条流。
/// 「录入之前」靠次序钉：报的展开上限与流的不同（那一道也在任何录入之前拒），交回的是没点到这条流的错、不是展开上限的错。
#[test]
fn a_run_whose_flow_is_not_named_in_the_block_ranges_is_refused_before_recording() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-not-named",
        file!(),
        "new_pool_file_creation",
    );
    let directory = EnvironmentRunDirectory::new("block-range-selection-not-named");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let flow_limit = ExpandUpToWritesPerSegment(EXPAND_UP_TO_OF_THE_FLOW);
    let different_limit = [
        ExpandUpToWritesPerSegment(0),
        ExpandUpToWritesPerSegment(usize::MAX),
    ]
    .into_iter()
    .find(|reported| {
        prepared
            .segments
            .iter()
            .any(|segment| reported.expansion_of(segment) != flow_limit.expansion_of(segment))
    })
    .expect("这条流有一段在 0 或无上限下展开得与上限 2 不同");
    for reported_limit in [flow_limit, different_limit] {
        let refusal = directory
            .run_with_the_block_range(Some(format!("{FLOW_NAME_NOT_RUN_HERE}:0-1")), || {
                run_from_environment_reporting_an_expansion_limit(
                    &prepared,
                    FLOW_NAME_JUDGED_BY_RANGES,
                    reported_limit,
                )
            })
            .expect_err("区间变量设了、没点到这条流要交错");
        for named in [
            BLOCK_RANGE_ENVIRONMENT_VARIABLE,
            FLOW_NAME_JUDGED_BY_RANGES,
            FLOW_NAME_NOT_RUN_HERE,
        ] {
            assert!(
                refusal.contains(named),
                "错误文字要点名 {named}（报的展开上限 {reported_limit:?}）：{refusal}"
            );
        }
    }
}

/// 同一份空库上按区间跑：先给这条流一个显式的空区间 `0-0`（一块不判、对账不数，行上报 `block_range=0-0` 与这条流的块数），再判 `0-1`、
/// `1-<块数>` 两片，每一片只判自己的块、判完对账为 0（别的片没判的块不算这一片没判完），行上报这一片的区间与这条流的块数；之后不给区间跑一趟，
/// 整条流全部复用、对账为 0、行上不打区间字段。止块过了块数的那一趟在判之前拒。
#[test]
fn a_run_given_a_block_range_judges_and_reconciles_only_the_blocks_of_its_range() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection",
        file!(),
        "new_pool_file_creation",
    );
    let directory = EnvironmentRunDirectory::new("block-range-selection");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();

    let (empty_run, empty_line) = directory
        .run_with_the_block_range(Some(format!("{FLOW_NAME_JUDGED_BY_RANGES}:0-0")), || {
            amplify_through_the_common_module(&prepared, FLOW_NAME_JUDGED_BY_RANGES)
        });
    let blocks_of_the_flow = empty_run.blocks_of_the_flow;
    assert!(
        blocks_of_the_flow >= 2,
        "这条流要至少两块才拆得成两片（崩溃点一块、末节点一块）：{blocks_of_the_flow}"
    );
    assert_eq!(
        empty_run.block_range,
        BlockRangeOfTheFlow::BlocksInTheRange(0..0)
    );
    assert_eq!(
        (empty_run.checking.blocks_checked, empty_run.unjudged_blocks),
        (0, 0),
        "空区间一块不判、对账一块不数：{empty_line}"
    );
    assert!(
        empty_line.contains(&format!(
            " block_range=0-0 blocks_of_the_flow={blocks_of_the_flow}"
        )),
        "{empty_line}"
    );

    for (start, end) in [(0, 1), (1, blocks_of_the_flow)] {
        let (slice_run, slice_line) = directory.run_with_the_block_range(
            Some(format!("{FLOW_NAME_JUDGED_BY_RANGES}:{start}-{end}")),
            || amplify_through_the_common_module(&prepared, FLOW_NAME_JUDGED_BY_RANGES),
        );
        assert_eq!(
            slice_run.block_range,
            BlockRangeOfTheFlow::BlocksInTheRange(start..end)
        );
        assert_eq!(
            slice_run.checking.blocks_checked,
            end - start,
            "这一片只判自己的 {start}-{end}：{slice_line}"
        );
        assert_eq!(
            slice_run.unjudged_blocks, 0,
            "这一片判齐了，别的片没判的块不算：{slice_line}"
        );
        assert!(
            slice_line.contains(&format!(
                " block_range={start}-{end} blocks_of_the_flow={blocks_of_the_flow}"
            )),
            "{slice_line}"
        );
    }

    let (whole_run, whole_line) = directory.run_with_the_block_range(None, || {
        amplify_through_the_common_module(&prepared, FLOW_NAME_JUDGED_BY_RANGES)
    });
    assert_eq!(
        whole_run.block_range,
        BlockRangeOfTheFlow::EveryBlockOfTheFlow
    );
    assert_eq!(
        (whole_run.checking.blocks_checked, whole_run.unjudged_blocks),
        (0, 0),
        "两片判过的块不给区间时全部复用，整条流对账为 0：{whole_line}"
    );
    assert!(
        !whole_line.contains("block_range="),
        "不给区间时行上不打区间字段：{whole_line}"
    );

    let refusal = directory
        .run_with_the_block_range(
            Some(format!(
                "{FLOW_NAME_JUDGED_BY_RANGES}:0-{}",
                blocks_of_the_flow + 1
            )),
            || run_from_environment_directly(&prepared, FLOW_NAME_JUDGED_BY_RANGES),
        )
        .expect_err("止块过了块数要拒");
    assert!(refusal.contains("越界"), "{refusal}");
}

/// 这台机器一块没选中的那一趟交回的样子：没领卡、判器与核对都没起、一块没判，行上照打区间字段与 `judge=none_selected_here`，
/// 区间那一行报这台机器选中 0 块。
fn assert_the_run_selected_no_block_and_judged_nothing(
    run: &EnvironmentRun,
    line: &str,
    expected_range: &str,
) {
    assert!(
        run.no_block_selected_here,
        "这台机器一块没选中，判段要在领卡之前交回：{line}"
    );
    assert_eq!(
        (
            run.checking.blocks_checked,
            run.checking.states_checked,
            run.cards_used
        ),
        (0, 0, 0),
        "一块没判、一张卡没用：{line}"
    );
    assert_eq!(run.gpu_cards_taken_from, GpuCardsSource::NoCardNeeded);
    assert!(
        run.verification.is_none() && run.comparison.is_none() && run.gpu_judge.is_none(),
        "第 ①②③ 段都没起：{line}"
    );
    assert!(
        !run.recorded_only && !run.placement_only && !run.stopped_early,
        "照常交回，不算只录、只排计划、被叫停"
    );
    let fields = format!(
        " block_range={expected_range} blocks_of_the_flow={}",
        run.blocks_of_the_flow
    );
    assert!(line.contains(&fields), "行上要有 {fields}：{line}");
    assert!(line.contains(" judge=none_selected_here"), "{line}");
    let block_range_line = run
        .block_range_line
        .as_deref()
        .expect("设了区间变量，这一趟打区间那一行");
    assert!(
        block_range_line.ends_with(&format!("{fields} blocks_selected_here=0")),
        "{block_range_line}"
    );
}

/// 显式空区间：这台机器一块不选，判段不领卡、不起判器，照常交回，`unjudged_blocks=0`，行上有 `block_range=<块>-<块>` 与这条流的块数。
#[test]
fn an_explicit_empty_block_range_hands_back_without_judging_and_prints_its_fields() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-empty",
        file!(),
        "new_pool_file_creation",
    );
    let directory = EnvironmentRunDirectory::new("block-range-selection-empty");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let (run, line) = directory
        .run_with_the_block_range(Some(format!("{FLOW_NAME_SELECTING_NO_BLOCK}:1-1")), || {
            amplify_through_the_common_module(&prepared, FLOW_NAME_SELECTING_NO_BLOCK)
        });
    assert_the_run_selected_no_block_and_judged_nothing(&run, &line, "1-1");
    assert_eq!(run.unjudged_blocks, 0, "空区间对账一块不数：{line}");
    assert!(run.violations.is_empty());
}

/// 份与区间没有交集（两台等分，区间只罩归另一份的那一块）：这台机器一块不选，同样不领卡、不起判器、照常交回；对账照这一片数、不分份，
/// 这一片那一块谁都还没判，`unjudged_blocks=1`（合并那一趟照它判这一片齐没齐）。
#[test]
fn a_share_without_blocks_inside_the_block_range_hands_back_without_judging() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-share",
        file!(),
        "new_pool_file_creation",
    );
    let host_shares = WeightedShares::new(vec![1, 1]);
    assert_eq!(
        host_shares.owner_of_block(0),
        0,
        "两台等分时块 0 归份 0：区间 0-1 在份 1 里一块都没有"
    );
    let directory = EnvironmentRunDirectory::new("block-range-selection-share");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let (run, line) = directory.run_with_the_block_range_and_these_variables(
        Some(format!("{FLOW_NAME_SELECTING_NO_BLOCK}:0-1")),
        &[(SHARE_ENVIRONMENT_VARIABLE, OsStr::new("1/2"))],
        || amplify_through_the_common_module(&prepared, FLOW_NAME_SELECTING_NO_BLOCK),
    );
    assert_eq!(run.share, HostShare { index: 1, count: 2 });
    assert_the_run_selected_no_block_and_judged_nothing(&run, &line, "0-1");
    assert_eq!(
        run.unjudged_blocks, 1,
        "对账照这一片数、不分份：这一片那一块归份 0、还没判：{line}"
    );
}

/// 没设区间变量、只按份分而这一份一块没领到（两台权重 `1,0`，领权重 0 的份 1）：不走一块没选中的那一支，照判完那一趟往下走、交回，
/// 交回的字段与不给区间时同形：`no_block_selected_here` 为假、不打区间那一行、片是整条流、第 ② 段 CPU 核对照跑、对账数整条流，
/// 行上是 `judge=cpu`、没有区间字段。
#[test]
fn a_share_without_blocks_and_without_a_block_range_does_not_hand_back_early() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-share-without-blocks",
        file!(),
        "new_pool_file_creation",
    );
    let directory = EnvironmentRunDirectory::new("block-range-selection-share-without-blocks");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let (run, line) = directory.run_with_the_block_range_and_these_variables(
        None,
        &[
            (SHARE_ENVIRONMENT_VARIABLE, OsStr::new("1/2")),
            (HOST_WEIGHTS_ENVIRONMENT_VARIABLE, OsStr::new("1,0")),
        ],
        || amplify_through_the_common_module(&prepared, FLOW_NAME_OF_A_SHARE_WITHOUT_BLOCKS),
    );
    assert_eq!(run.share, HostShare { index: 1, count: 2 });
    assert!(
        blocks_of_this_share_in_the_block_range(
            run.blocks_of_the_flow,
            &WeightedShares::new(vec![1, 0]),
            1,
            &BlockRangeOfTheFlow::EveryBlockOfTheFlow,
        )
        .is_empty(),
        "权重 0 的份 1 一块都分不到"
    );
    assert!(
        !run.no_block_selected_here,
        "没设区间变量，一块没领到也不走提前交回那一支：{line}"
    );
    assert_eq!(run.block_range, BlockRangeOfTheFlow::EveryBlockOfTheFlow);
    assert_eq!(run.block_range_line, None, "没设区间变量不打区间那一行");
    assert_eq!(
        (
            run.checking.blocks_checked,
            run.checking.states_checked,
            run.cards_used
        ),
        (0, 0, 0),
        "一块没领到，一块没判、一张卡没用：{line}"
    );
    assert!(
        run.verification.is_some(),
        "照判完那一趟往下走，第 ② 段 CPU 核对照跑：{line}"
    );
    assert!(
        !run.recorded_only && !run.placement_only && !run.stopped_early,
        "照判完那一趟交回，不算只录、只排计划、被叫停"
    );
    assert_eq!(
        u64::try_from(run.unjudged_blocks).expect("块数装得进 u64"),
        run.blocks_of_the_flow,
        "对账数整条流：份 0 还没判，整条流的块都没判：{line}"
    );
    assert!(line.contains(" judge=cpu"), "{line}");
    assert!(
        !line.contains("block_range=") && !line.contains("none_selected_here"),
        "不给区间时行上没有区间字段、也不写一块没选中：{line}"
    );
}

/// 只录那一趟（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）不打 `CRASH_AMPLIFICATION` 行，区间与块数照样打在区间那一行上。
#[test]
fn a_recording_only_run_given_a_block_range_prints_the_block_range_line() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-recorded-only",
        file!(),
        "new_pool_file_creation",
    );
    let directory = EnvironmentRunDirectory::new("block-range-selection-recorded-only");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let (run, line) = directory.run_with_the_block_range_and_these_variables(
        Some(format!("{FLOW_NAME_RECORDED_ONLY}:0-1")),
        &[(RUN_STAGE_ENVIRONMENT_VARIABLE, OsStr::new("record"))],
        || {
            common_crash_points::amplify_prepared_flow_and_hand_back_its_line(
                FLOW_NAME_RECORDED_ONLY,
                &prepared.base,
                &prepared.writes,
                &prepared.segments,
                prepared.judged_root_index,
                &prepared.versions,
                prepared.crash_points.clone(),
                EXPAND_UP_TO_OF_THE_FLOW,
            )
        },
    );
    assert!(run.recorded_only, "选段是 record：这一趟只录");
    assert!(line.is_none(), "只录的那一趟不打 CRASH_AMPLIFICATION 行");
    assert_eq!(
        run.block_range_line.as_deref(),
        Some(
            format!(
                "CRASH_AMPLIFICATION_BLOCK_RANGE flow={FLOW_NAME_RECORDED_ONLY} share=0/1 block_range=0-1 blocks_of_the_flow={} blocks_selected_here=1",
                run.blocks_of_the_flow
            )
            .as_str()
        )
    );
}

/// 版本表写错的流（根落了的状态读回的内容对不上，要红）：不给区间判一趟整条流，库里存下红；再一片一块地按区间跑，每一片对账为 0、
/// 读到的违例只是本片的块里的，几片读到的加起来恰好是整条流的违例（不多读别的片的、不漏本片的）。
#[test]
fn red_states_stored_inside_a_block_range_are_read_as_the_violations_of_that_range() {
    let mut prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-red",
        file!(),
        "new_pool_file_creation",
    );
    prepared.versions[0].content[0] ^= 0x01;
    let directory = EnvironmentRunDirectory::new("block-range-selection-red");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();

    let whole_run = directory
        .run_with_the_block_range(None, || {
            run_from_environment_directly(&prepared, FLOW_NAME_WITH_A_WRONG_VERSION_TABLE)
        })
        .expect("按环境跑得完");
    assert!(
        whole_run.checking.red_states > 0,
        "版本表的内容写错，根落了的状态读回的内容对不上，必须红"
    );
    assert_eq!(whole_run.unjudged_blocks, 0, "整条流判完了");
    let violations_of_the_flow = whole_run.violations.len();
    assert!(violations_of_the_flow > 0, "库里存下了这条流的红");

    let mut violations_read_by_the_slices = 0usize;
    for start in 0..whole_run.blocks_of_the_flow {
        let slice_run = directory
            .run_with_the_block_range(
                Some(format!(
                    "{FLOW_NAME_WITH_A_WRONG_VERSION_TABLE}:{start}-{}",
                    start + 1
                )),
                || run_from_environment_directly(&prepared, FLOW_NAME_WITH_A_WRONG_VERSION_TABLE),
            )
            .expect("按环境跑得完");
        assert_eq!(
            slice_run.unjudged_blocks, 0,
            "块 {start} 这一片判齐了，别的片不算"
        );
        violations_read_by_the_slices += slice_run.violations.len();
    }
    assert_eq!(
        violations_read_by_the_slices, violations_of_the_flow,
        "一片一块读到的违例加起来恰好是整条流的违例"
    );
}

/// 观察者那几条真跑用的流名（来历表里「两处都不登」的那一条，`::` 之后是标签；一条流一份库，互不复用）。
const FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE: &str =
    "line-fields::block-range-selection-probe-outside-the-range";
const FLOW_NAME_WITH_THE_PROBE_INSIDE_THE_RANGE: &str =
    "line-fields::block-range-selection-probe-inside-the-range";
const FLOW_NAME_WITHOUT_THE_PROBE: &str = "line-fields::block-range-selection-probe-not-attached";
/// 固定脚本那几条共用的「判齐、核过、再跑一趟全部复用」断言真跑用的流名。
const FLOW_NAME_JUDGED_AND_REUSED_BY_SLICES: &str =
    "line-fields::block-range-selection-judged-and-reused-by-slices";
const FLOW_NAME_SHORT_OF_ONE_BLOCK: &str = "line-fields::block-range-selection-short-of-one-block";
/// 新池新建文件那一个崩溃点的名字：只数状态的观察者挂在它上面，末节点不挂。
const CRASH_POINT_OF_THE_PROBE: &str = "new_pool_file_creation";
/// 只数状态的观察者（没登记，取键拒算、照判不入库）与它的计数名。
const PROBE_OBSERVER_NAME: &str = "block_range_probe";
const STATES_SEEN_BY_THE_PROBE_COUNTER: &str = "states_seen_by_the_block_range_probe";
/// 挂在末节点上的另一个只数状态的观察者：这一片里有块的点上没挂探针时，判段照样有观察者可判（`EnvironmentRun::observing` 不空）。
const OBSERVER_ON_THE_END_OF_THE_FLOW_NAME: &str = "block_range_observer_on_the_end_of_the_flow";

/// 只数状态的观察者：每看一个状态加一次计数。
fn count_each_state_seen_by_the_block_range_probe(
    _ordinal: u64,
    _image: &CrashImage<'_>,
    _report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    counts.add(STATES_SEEN_BY_THE_PROBE_COUNTER, 1);
}

/// 只数状态的观察者 `name`，挂在设在这一份文件里的 `entry_name` 那个节点上。
fn counting_observer_on_the_node<'observer>(
    name: &str,
    entry_name: &str,
    observe: &'observer ObserveACrashState<'observer>,
) -> ObserverOnTheNodes<'observer> {
    ObserverOnTheNodes {
        observer_name: ObserverName::new(name).expect("观察者的名字非空、不长"),
        observer_code_file: repository_relative_file(file!()),
        extra_inputs_digest: [0; DIGEST_BYTES],
        nodes: vec![NodeOfAnObserver {
            setting_file: repository_relative_file(file!()),
            entry_name: entry_name.to_string(),
        }],
        observe: ObserverCode::CountsOnly(observe),
    }
}

/// 经测试共用的跑法带着观察者跑一趟（健康的流断言一个红都没有、本片判齐时库里本片没有红、计数向量对得上）。
fn amplify_with_observers_through_the_common_module(
    prepared: &NewPoolFileCreationAsOneCrashPoint,
    flow_name: &str,
    observers: &[ObserverOnTheNodes<'_>],
) -> EnvironmentRun {
    common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
        flow_name,
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &[],
        prepared.crash_points.clone(),
        EXPAND_UP_TO_OF_THE_FLOW,
        observers,
    )
    .0
}

/// 这一趟领的块落在哪几个点上：（点名，块数），按计划的次序。
fn points_selected_here(observing: &ObservingOfTheRun) -> Vec<(String, u64)> {
    observing
        .blocks_selected_here_on_each_point
        .iter()
        .map(|point| (point.point_name.clone(), point.blocks))
        .collect()
}

/// 观察者看过状态的那几条断言判红没有（判红交回那一句）：`states_seen_by_the_observer_as_the_block_range_requires` 照这一趟判。
fn observer_states_assertion_refusal(
    run: &EnvironmentRun,
    observing: &ObservingOfTheRun,
    flow_name: &str,
    nodes: Option<&[NodeOfAnObserver]>,
) -> Option<String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        states_seen_by_the_observer_as_the_block_range_requires(
            run,
            observing,
            flow_name,
            PROBE_OBSERVER_NAME,
            nodes,
            &format!("流 {flow_name}：观察者 {PROBE_OBSERVER_NAME} 一个状态都没看"),
        )
    }))
    .err()
    .map(|payload| {
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                payload
                    .downcast_ref::<&str>()
                    .map(|text| (*text).to_string())
            })
            .unwrap_or_default()
    })
}

/// 块区间落在没有观察者节点的块上（只罩末节点那一块，探针挂在崩溃点上）：这一片里落在探针节点上的块是 0，探针看过 0 个状态，
/// 看过状态的断言不要求它大于 0、用例照过；这一片照常判完，对账为 0、库里这一片没有红（经测试共用的跑法，那一层照判对账与违例）。
#[test]
fn an_observer_without_a_node_inside_the_block_range_is_not_required_to_have_seen_a_state() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-probe-outside-the-range",
        file!(),
        CRASH_POINT_OF_THE_PROBE,
    );
    let probe = count_each_state_seen_by_the_block_range_probe;
    let observers = [counting_observer_on_the_node(
        PROBE_OBSERVER_NAME,
        CRASH_POINT_OF_THE_PROBE,
        &probe,
    )];
    let directory = EnvironmentRunDirectory::new("block-range-selection-probe-outside-the-range");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let blocks_of_the_flow = directory
        .run_with_the_block_range(
            Some(format!("{FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE}:0-0")),
            || {
                amplify_with_observers_through_the_common_module(
                    &prepared,
                    FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE,
                    &observers,
                )
            },
        )
        .blocks_of_the_flow;
    assert!(
        blocks_of_the_flow >= 2,
        "这条流要至少两块（崩溃点一块、末节点一块）：{blocks_of_the_flow}"
    );
    let end_block = blocks_of_the_flow - 1;
    let run = directory.run_with_the_block_range(
        Some(format!(
            "{FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE}:{end_block}-{blocks_of_the_flow}"
        )),
        || {
            amplify_with_observers_through_the_common_module(
                &prepared,
                FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE,
                &observers,
            )
        },
    );
    let observing = run
        .observing
        .as_ref()
        .expect("这一片判了一块，挂了观察者，判段照跑");
    assert_eq!(
        points_selected_here(observing),
        vec![(END_OF_FLOW_NODE_NAME.to_string(), 1)],
        "这一片只领了末节点那一块"
    );
    assert_eq!(
        blocks_selected_here_on_the_nodes(observing, &observers[0].nodes),
        0,
        "这一片里没有探针节点上的块"
    );
    assert_eq!(
        observer_states_assertion_refusal(
            &run,
            observing,
            FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE,
            Some(observers[0].nodes.as_slice())
        ),
        None,
        "这一片里没有探针节点上的块：不要求它看过状态"
    );
    assert_eq!(
        states_seen_by_the_observer_as_the_block_range_requires(
            &run,
            observing,
            FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE,
            PROBE_OBSERVER_NAME,
            Some(observers[0].nodes.as_slice()),
            "探针一个状态都没看",
        ),
        0,
        "探针在这一片里一个状态都没看"
    );
    assert_eq!(
        (run.checking.blocks_checked, run.unjudged_blocks),
        (1, 0),
        "这一片照常判了末节点那一块，对账为 0"
    );
    assert!(run.violations.is_empty(), "健康的流，这一片读不到违例");
    // 对整条流的断言（截断续做真发过那一类）：这一片判不了、留给不设块区间的那一趟；不设块区间照判
    assert!(
        whole_flow_assertion_is_left_to_the_run_without_a_block_range(
            &run,
            FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE,
            "probe_saw_a_state_somewhere_in_the_flow",
        )
    );
    let mut whole_flow_run = run.clone();
    whole_flow_run.block_range = BlockRangeOfTheFlow::EveryBlockOfTheFlow;
    assert!(
        !whole_flow_assertion_is_left_to_the_run_without_a_block_range(
            &whole_flow_run,
            FLOW_NAME_WITH_THE_PROBE_OUTSIDE_THE_RANGE,
            "probe_saw_a_state_somewhere_in_the_flow",
        )
    );
}

/// 块区间含观察者节点上的块（只罩崩溃点那一块，探针挂在它上面）：探针看过状态，断言照旧要求大于 0、用例照过；
/// 让它看 0 个状态（同一趟的结果把它看过的状态数改成 0）、或这一趟根本没挂它（同一片另挂一个只在末节点上的观察者跑一趟），断言都红。
/// 不设块区间时它看 0 个状态同样红（整条流照旧要求）。
#[test]
fn an_observer_with_a_node_inside_the_block_range_is_still_required_to_have_seen_a_state() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-probe-inside-the-range",
        file!(),
        CRASH_POINT_OF_THE_PROBE,
    );
    let probe = count_each_state_seen_by_the_block_range_probe;
    let observers = [counting_observer_on_the_node(
        PROBE_OBSERVER_NAME,
        CRASH_POINT_OF_THE_PROBE,
        &probe,
    )];
    let probe_nodes = observers[0].nodes.clone();
    let directory = EnvironmentRunDirectory::new("block-range-selection-probe-inside-the-range");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let run = directory.run_with_the_block_range(
        Some(format!("{FLOW_NAME_WITH_THE_PROBE_INSIDE_THE_RANGE}:0-1")),
        || {
            amplify_with_observers_through_the_common_module(
                &prepared,
                FLOW_NAME_WITH_THE_PROBE_INSIDE_THE_RANGE,
                &observers,
            )
        },
    );
    let observing = run
        .observing
        .as_ref()
        .expect("这一片判了一块，挂了观察者，判段照跑");
    assert_eq!(
        points_selected_here(observing),
        vec![(CRASH_POINT_OF_THE_PROBE.to_string(), 1)],
        "这一片只领了崩溃点那一块"
    );
    assert_eq!(
        blocks_selected_here_on_the_nodes(observing, &probe_nodes),
        1
    );
    assert_eq!(
        observer_states_assertion_refusal(
            &run,
            observing,
            FLOW_NAME_WITH_THE_PROBE_INSIDE_THE_RANGE,
            Some(probe_nodes.as_slice())
        ),
        None,
        "探针看过这一片里它节点上的状态"
    );
    let states = observing
        .states_of_each_observer
        .get(PROBE_OBSERVER_NAME)
        .copied()
        .unwrap_or(0);
    assert!(states > 0, "探针看过状态");
    assert_eq!(
        observing
            .counts_of_each_observer
            .get(PROBE_OBSERVER_NAME)
            .and_then(|counts| counts.get(STATES_SEEN_BY_THE_PROBE_COUNTER))
            .copied(),
        Some(states),
        "探针自己数的状态数与判段交回的相同"
    );

    let mut saw_no_state = observing.clone();
    saw_no_state
        .states_of_each_observer
        .insert(PROBE_OBSERVER_NAME.to_string(), 0);
    let refusal_when_it_saw_no_state = observer_states_assertion_refusal(
        &run,
        &saw_no_state,
        FLOW_NAME_WITH_THE_PROBE_INSIDE_THE_RANGE,
        Some(probe_nodes.as_slice()),
    )
    .expect("这一片里有探针节点上的块，它看 0 个状态要红");
    assert!(
        refusal_when_it_saw_no_state.contains("一个状态都没看")
            && refusal_when_it_saw_no_state.contains("块区间 0-1"),
        "{refusal_when_it_saw_no_state}"
    );
    let mut whole_flow_run = run.clone();
    whole_flow_run.block_range = BlockRangeOfTheFlow::EveryBlockOfTheFlow;
    assert!(
        observer_states_assertion_refusal(
            &whole_flow_run,
            &saw_no_state,
            FLOW_NAME_WITH_THE_PROBE_INSIDE_THE_RANGE,
            Some(probe_nodes.as_slice())
        )
        .is_some(),
        "不设块区间时它看 0 个状态照旧红"
    );

    let other = count_each_state_seen_by_the_block_range_probe;
    let observers_without_the_probe = [counting_observer_on_the_node(
        OBSERVER_ON_THE_END_OF_THE_FLOW_NAME,
        END_OF_FLOW_NODE_NAME,
        &other,
    )];
    let run_without_the_probe = directory.run_with_the_block_range(
        Some(format!("{FLOW_NAME_WITHOUT_THE_PROBE}:0-1")),
        || {
            amplify_with_observers_through_the_common_module(
                &prepared,
                FLOW_NAME_WITHOUT_THE_PROBE,
                &observers_without_the_probe,
            )
        },
    );
    let observing_without_the_probe = run_without_the_probe
        .observing
        .as_ref()
        .expect("挂了末节点上的观察者，判段照跑");
    let refusal_without_the_probe = observer_states_assertion_refusal(
        &run_without_the_probe,
        observing_without_the_probe,
        FLOW_NAME_WITHOUT_THE_PROBE,
        Some(probe_nodes.as_slice()),
    )
    .expect("这一片里有探针节点上的块，这一趟没挂探针要红");
    assert!(
        refusal_without_the_probe.contains("一个状态都没看"),
        "{refusal_without_the_probe}"
    );
    assert!(
        observer_states_assertion_refusal(
            &run_without_the_probe,
            observing_without_the_probe,
            FLOW_NAME_WITHOUT_THE_PROBE,
            None
        )
        .is_some(),
        "调用方挂的观察者里没有探针，设了块区间照样红"
    );
}

/// 一条录好的小流经 `run_from_environment` 跑一趟，再经固定脚本两条放量共用的「判齐、核过、再跑一趟全部复用」断言判（再跑的那一趟在
/// 同一个环境里），交回头一趟的结果、测试自己排的计划与再跑那一趟的结果。环境由调用方的守卫定。
fn judge_and_reuse_through_the_shared_assertions(
    prepared: &NewPoolFileCreationAsOneCrashPoint,
    flow_name: &str,
) -> (EnvironmentRun, CrashPlan, EnvironmentRun) {
    let flow_limit = ExpandUpToWritesPerSegment(EXPAND_UP_TO_OF_THE_FLOW);
    let expansion =
        move |_segment_index: usize, segment: &[usize]| flow_limit.expansion_of(segment);
    let flow = CrashFlow {
        base: &prepared.base,
        writes: &prepared.writes,
        segments: &prepared.segments,
        judged_root_index: prepared.judged_root_index,
        versions: &prepared.versions,
        versions_without_a_file: &[],
        expansion: &expansion,
        crash_points: prepared.crash_points.clone(),
    };
    let toolchain = CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test();
    let plan = plan_crash_points(&flow);
    let run = run_from_environment(&flow, flow_name, &toolchain, flow_limit).expect("按环境跑得完");
    let mut rerun_kept = None;
    assert_the_run_judged_verified_and_reused_its_blocks(&run, &plan, || {
        let rerun =
            run_from_environment(&flow, flow_name, &toolchain, flow_limit).expect("再跑一趟跑得完");
        rerun_kept = Some(rerun.clone());
        rerun
    });
    (run, plan, rerun_kept.expect("整份自己跑，再跑了一趟"))
}

/// 固定脚本两条放量共用的「判齐、核过、再跑一趟全部复用」断言在设了块区间时按这一片判、不误红：同一份空库上判 `0-1`、`1-<块数>` 两片，
/// 每一片这一趟判的状态数加录入时这一片已判的，等于测试自己排的计划里这一片的状态数；再跑一趟这一片全部复用、一块不再核；
/// 整条流录入时还有别的片没判（照整条流数的那几句在这一片上会红）。之后不设块区间跑一趟，照整条流判：全部复用，判过的块第 ② 段都核过。
#[test]
fn the_shared_judged_and_reused_assertions_judge_only_the_slice_under_a_block_range() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-judged-and-reused-by-slices",
        file!(),
        CRASH_POINT_OF_THE_PROBE,
    );
    let directory = EnvironmentRunDirectory::new("block-range-selection-judged-and-reused");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let blocks_of_the_flow = directory
        .run_with_the_block_range(
            Some(format!("{FLOW_NAME_JUDGED_AND_REUSED_BY_SLICES}:0-0")),
            || run_from_environment_directly(&prepared, FLOW_NAME_JUDGED_AND_REUSED_BY_SLICES),
        )
        .expect("按环境跑得完")
        .blocks_of_the_flow;
    assert!(
        blocks_of_the_flow >= 2,
        "这条流要至少两块才拆得成两片：{blocks_of_the_flow}"
    );
    for (start, end) in [(0, 1), (1, blocks_of_the_flow)] {
        let (run, plan, rerun) = directory.run_with_the_block_range(
            Some(format!(
                "{FLOW_NAME_JUDGED_AND_REUSED_BY_SLICES}:{start}-{end}"
            )),
            || {
                judge_and_reuse_through_the_shared_assertions(
                    &prepared,
                    FLOW_NAME_JUDGED_AND_REUSED_BY_SLICES,
                )
            },
        );
        let covered = blocks_and_states_of_the_block_range(
            &plan,
            &BlockRangeOfTheFlow::BlocksInTheRange(start..end),
        );
        assert_eq!(covered.blocks, end - start, "这一片罩 {start}-{end}");
        assert_eq!(
            run.checking.states_checked, covered.states,
            "空库上这一片的状态这一趟都判了"
        );
        assert_eq!(rerun.states_reused, covered.states);
        if start == 0 {
            assert!(
                run.recording.states_recorded > run.checking.states_checked,
                "整条流录入时还有别的片没判：照整条流数「录下来要判的都核了」这一片会红"
            );
        }
    }
    let (whole_run, _plan, whole_rerun) = directory.run_with_the_block_range(None, || {
        judge_and_reuse_through_the_shared_assertions(
            &prepared,
            FLOW_NAME_JUDGED_AND_REUSED_BY_SLICES,
        )
    });
    assert_eq!(
        whole_run.block_range,
        BlockRangeOfTheFlow::EveryBlockOfTheFlow
    );
    assert_eq!(
        (
            whole_run.checking.blocks_checked,
            whole_rerun.checking.blocks_checked
        ),
        (0, 0),
        "两片判过的块不设区间时全部复用"
    );
    assert_eq!(
        whole_run
            .comparison
            .as_ref()
            .expect("第 ②③ 段跑了")
            .blocks_without_verifier,
        0,
        "两片判过的块第 ② 段都核过"
    );
}

/// 固定脚本两条放量共用的「判齐、再跑一趟全部复用」断言在设了块区间时照样判得出少判的块：空库上判 `0-<块数>` 一片（两块都在片里），
/// 把这一趟交回的判过的状态数少掉末节点那一块的状态数（这一片少判一块），断言红；再跑那一趟交回的复用状态数少掉一块、或核了一块，同样红。
/// 「判过的块第 ② 段都核过」数整条流：交回的没核块数是 1 时设了块区间不红（别的片判过、还没核到的块），不设块区间红。
#[test]
fn the_shared_judged_and_reused_assertions_go_red_when_the_slice_is_short_of_one_block() {
    let prepared = common_crash_points::new_pool_file_creation_as_one_crash_point(
        "crash-amplification-block-range-selection-short-of-one-block",
        file!(),
        CRASH_POINT_OF_THE_PROBE,
    );
    let directory = EnvironmentRunDirectory::new("block-range-selection-short-of-one-block");
    let _the_library_of_this_process = directory.library.make_it_the_library_of_this_process();
    let blocks_of_the_flow = directory
        .run_with_the_block_range(Some(format!("{FLOW_NAME_SHORT_OF_ONE_BLOCK}:0-0")), || {
            run_from_environment_directly(&prepared, FLOW_NAME_SHORT_OF_ONE_BLOCK)
        })
        .expect("按环境跑得完")
        .blocks_of_the_flow;
    let (run, plan, rerun) = directory.run_with_the_block_range(
        Some(format!(
            "{FLOW_NAME_SHORT_OF_ONE_BLOCK}:0-{blocks_of_the_flow}"
        )),
        || judge_and_reuse_through_the_shared_assertions(&prepared, FLOW_NAME_SHORT_OF_ONE_BLOCK),
    );
    let one_block = blocks_and_states_of_the_block_range(
        &plan,
        &BlockRangeOfTheFlow::BlocksInTheRange(blocks_of_the_flow - 1..blocks_of_the_flow),
    );
    assert_eq!(one_block.blocks, 1);
    let goes_red = |judged: &EnvironmentRun, rerun_handed_back: &EnvironmentRun| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_the_run_judged_verified_and_reused_its_blocks(judged, &plan, || {
                rerun_handed_back.clone()
            });
        }))
        .is_err()
    };
    assert!(!goes_red(&run, &rerun), "真跑的两趟照这一片判，不红");
    let mut short_of_one_block = run.clone();
    short_of_one_block.checking.states_checked -= one_block.states;
    short_of_one_block.checking.blocks_checked -= 1;
    assert!(
        goes_red(&short_of_one_block, &rerun),
        "这一片少判一块（录入时没判、这一趟也没判）要红"
    );
    let mut rerun_short_of_one_block = rerun.clone();
    rerun_short_of_one_block.states_reused -= one_block.states;
    assert!(
        goes_red(&run, &rerun_short_of_one_block),
        "再跑一趟这一片少复用一块要红"
    );
    let mut rerun_checking_one_block = rerun.clone();
    rerun_checking_one_block.checking.blocks_checked = 1;
    assert!(
        goes_red(&run, &rerun_checking_one_block),
        "再跑一趟这一片又核了一块要红"
    );
    // 第 ③ 段比对数整条流：别的片判过、第 ② 段还没核到的块（这里把交回的数改成 1 造这一种）这一片判不了、不红；不设块区间照整条流判、红
    let mut another_slice_left_unverified = run.clone();
    another_slice_left_unverified
        .comparison
        .as_mut()
        .expect("没开 GPU：第 ② 段由 CPU 参照核、第 ③ 段比了")
        .blocks_without_verifier = 1;
    assert!(
        !goes_red(&another_slice_left_unverified, &rerun),
        "设了块区间：整条流判过而没核的块这一片判不了，不红"
    );
    let mut whole_flow_left_unverified = another_slice_left_unverified.clone();
    whole_flow_left_unverified.block_range = BlockRangeOfTheFlow::EveryBlockOfTheFlow;
    whole_flow_left_unverified.checking.states_checked =
        whole_flow_left_unverified.recording.states_recorded;
    let mut whole_flow_rerun = rerun.clone();
    whole_flow_rerun.recording.blocks_already_judged =
        run.recording.blocks_recorded + run.recording.blocks_already_judged;
    assert!(
        goes_red(&whole_flow_left_unverified, &whole_flow_rerun),
        "不设块区间：判过的块第 ② 段没核完要红"
    );
}
