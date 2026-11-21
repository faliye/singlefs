//! 崩溃枚举用例的入口：`enumerate_crash_states` 一族，照项、流、展开与观察者经库走录入段与判段；缩域取小的那个展开。

use super::*;

// ============================== 公开入口（崩溃枚举用例调的几种写法） ==============================

/// 单版本形态、每一段都展开（全量）。
pub fn enumerate_crash_states(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    root_index: usize,
    expected_content: &[u8],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_selecting(
        item,
        base,
        writes,
        segments,
        root_index,
        expected_content,
        &|_segment_index, _segment| true,
    )
}

/// 单版本形态，`expand` 选哪几段展开（展开就是任意真子集）。
pub fn enumerate_crash_states_selecting(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    root_index: usize,
    expected_content: &[u8],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> StageOutcome<CrashPointReplayTally> {
    let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_index]);
    let versions = [PublishedVersion {
        instance,
        checkpoint_txg,
        content: expected_content.to_vec(),
    }];
    enumerate_crash_states_selecting_versions(
        item, base, writes, segments, root_index, &versions, expand,
    )
}

/// 多版本、每一段都展开（全量）。
pub fn enumerate_crash_states_versions(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_selecting_versions(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &|_segment_index, _segment| true,
    )
}

/// 多版本，`expand` 选哪几段展开（展开就是任意真子集）。
pub fn enumerate_crash_states_selecting_versions(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_with_expansion(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
        &|segment_index, segment| {
            if expand(segment_index, segment) {
                CrashPointReplaySegmentExpansion::EveryProperSubset
            } else {
                CrashPointReplaySegmentExpansion::NotExpanded
            }
        },
        &[],
    )
}

/// 崩溃点重放枚举的总入口：`expansion` 定每一段怎么展开，`versions` 是 oracle 读的版本表（带「没有文件的那几版」的删除流也经它），
/// `observers` 逐状态看（没有就空）。录入段把流（连同每段的展开）写进库、登清单；判段读回、按块判、存结论、重建计数。
/// 调用方挂的观察者之外，按这条流在来历表里那一行另挂几个（`crate::crash_point_replay_provenance`：登记位上的流挂并集对走读观察者，
/// 来历是 mkfs 返回之后的另挂两槽全废盘面观察者），设点文件里不手挂。
///
/// # Panics
/// 开不了库、写不进、读不回；选路要卡而卡用不上（配了 GPU 不退回 CPU）；结论版本与编译时对不上（整趟不写库）；判段判完还有块没结论
/// （不是两台分片时）；计数向量与计划对不上；某个观察者在某个状态上 panic（它那一句原样带出）；项名撞了；来历表里没有这条流、有几行，
/// 或调用方挂的观察者与按来历挂的同名（录任何东西之前）。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个是同一条流的几样输入（与 crash::judge_crash_state_range_of_the_read_back_flow 同一组），其余是展开、版本表与观察者"
)]
pub fn enumerate_crash_states_with_expansion(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[CrashPointReplayObserver<'_>],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_with_every_kind_of_observer(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        observers,
        &[],
    )
}

/// 同 [`enumerate_crash_states_with_expansion`]，另带交回镜像的观察者（[`CrashPointReplayObserverHandingBackAnImage`]：可写挂载之后的盘上的
/// 池级 checker 在判段判，配了 GPU 上卡）。
///
/// # Panics
/// 同 [`enumerate_crash_states_with_expansion`]。
#[allow(
    clippy::too_many_arguments,
    reason = "与 enumerate_crash_states_with_expansion 同一组参数，多一组交回镜像的观察者"
)]
pub fn enumerate_crash_states_with_every_kind_of_observer(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[CrashPointReplayObserver<'_>],
    observers_handing_back_images: &[CrashPointReplayObserverHandingBackAnImage<'_>],
) -> StageOutcome<CrashPointReplayTally> {
    let attachment = CrashPointReplayObserverAttachment::EVERY_POINT_WITH_THE_CASE_FILE;
    enumerate_crash_states_with_attached_observers(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        &observers
            .iter()
            .map(|observer| AttachedCrashPointReplayObserver {
                observer: *observer,
                attachment,
            })
            .collect::<Vec<_>>(),
        &observers_handing_back_images
            .iter()
            .map(
                |observer| AttachedCrashPointReplayObserverHandingBackAnImage {
                    observer: *observer,
                    attachment,
                },
            )
            .collect::<Vec<_>>(),
    )
}

/// 同 [`enumerate_crash_states_with_every_kind_of_observer`]，每个观察者各带挂法（[`CrashPointReplayObserverAttachment`]）：只挂在点名的段上的观察者
/// 在别的点上不录、不判，块账在它不挂的点上不要它的结论；代码文件可以点名几条用例共用的那一份。
///
/// # Panics
/// 同 [`enumerate_crash_states_with_expansion`]；挂法点名的段集合是空的、段下标越过流的段数、或点名的段一个点都罩不着（录任何东西之前）。
#[allow(
    clippy::too_many_arguments,
    reason = "与 enumerate_crash_states_with_expansion 同一组参数，观察者两组各带挂法"
)]
pub fn enumerate_crash_states_with_attached_observers(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[AttachedCrashPointReplayObserver<'_>],
    observers_handing_back_images: &[AttachedCrashPointReplayObserverHandingBackAnImage<'_>],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_through_the_library(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        observers,
        observers_handing_back_images,
        ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
    )
    .unwrap_or_else(|refusal| panic!("崩溃枚举用例「{}」：{refusal}", item.name))
}

/// 同 [`enumerate_crash_states_with_expansion`]，观察者那几组的判法单元键从 `observer_keys` 点名的那一张键表取
/// （`crate::crash_identity::ObserverJudgingUnitsKeyTable`）：观察者登在别的包（实验包）自己的观察者登记表、键由那个包的构建脚本算时，
/// 交那个包编进二进制的那一张；判器、池级 checker 与观察者结论拼块那几组照旧取验证包编进来的那一张。观察者挂在整条流上、代码文件是用例文件
/// （`item.setting_file`），同 [`enumerate_crash_states_with_expansion`]。
///
/// # Panics
/// 同 [`enumerate_crash_states_with_expansion`]；别的包那一张键表认不回来（`crate::crash_identity::judging_units_key_with_the_observers_keyed_in`）。
#[allow(
    clippy::too_many_arguments,
    reason = "与 enumerate_crash_states_with_expansion 同一组参数，多一个观察者键表"
)]
pub fn enumerate_crash_states_with_expansion_keying_the_observers_in(
    observer_keys: ObserverJudgingUnitsKeyTable,
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[CrashPointReplayObserver<'_>],
) -> StageOutcome<CrashPointReplayTally> {
    enumerate_crash_states_through_the_library(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
        &observers
            .iter()
            .map(|observer| AttachedCrashPointReplayObserver {
                observer: *observer,
                attachment: CrashPointReplayObserverAttachment::EVERY_POINT_WITH_THE_CASE_FILE,
            })
            .collect::<Vec<_>>(),
        &[],
        observer_keys,
    )
    .unwrap_or_else(|refusal| panic!("崩溃枚举用例「{}」：{refusal}", item.name))
}

#[allow(
    clippy::too_many_arguments,
    reason = "与 enumerate_crash_states_with_expansion 同一组参数"
)]
fn enumerate_crash_states_through_the_library(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    observers: &[AttachedCrashPointReplayObserver<'_>],
    observers_handing_back_images: &[AttachedCrashPointReplayObserverHandingBackAnImage<'_>],
    observer_keys: ObserverJudgingUnitsKeyTable,
) -> Result<StageOutcome<CrashPointReplayTally>, String> {
    // 按来历统一挂：来历表里查不到这条流就拒，在录任何东西之前
    let observers_on_the_nodes = observers_on_the_nodes_with_the_ones_attached_to_the_flow(
        RunOfTheFlow::CrashPointReplayEnumeration,
        item.name,
        &observers_on_the_nodes_of_the_item(
            item,
            writes,
            segments,
            observers,
            observers_handing_back_images,
        )?,
        &nodes_of_the_item(item),
    )
    .map_err(|refusal| refusal.sentence())?;
    // 多机配置在入口读一次：选路照它；判段走 GPU 那一路要的问服务的控制命令与观察者交回的盘上取卡的显卡来路同样在这里照此刻的环境读、往下传，
    // 判段里不再读进程环境（取不到控制命令的那一句留到判段真要问服务时才交回）
    let configuration = crate::crash_amplification::multi_host_configuration_of_this_process()?;
    let route = crash_point_replay_judge_route(&configuration);
    let gpu_judging_inputs =
        GpuJudgingInputsOfTheEntry::read_at_the_entry(configuration.configuration_file);
    let keys = keys_of_the_item(route, &observers_on_the_nodes, observer_keys);
    // 缩域（「允许缩域」）：设了展开上限变量就取用例自己的展开与它里小的那个；块键含段展开，缩过域的块各存各的
    let narrowing = ExpandUpToWritesPerSegment::narrowing_from_the_environment();
    let narrowed_expansion = |segment_index: usize, segment: &[usize]| {
        expansion_of_the_case_narrowed_by(narrowing, expansion(segment_index, segment), segment)
    };
    let flow_to_record = flow_of_the_inputs(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        &narrowed_expansion,
    );
    // 覆盖行：覆盖表脚本从这一趟的日志里读哪个（入口，提交步骤，段）在哪个域里展开了。崩溃枚举用例没有按段登记；
    // 只录、只判的那几趟同样打（录下的流是同一条）
    flow_to_record.with_the_flow(|flow| {
        for coverage_line in crash_flow_coverage_lines(flow, &[]) {
            println!("{}", coverage_line.text());
        }
    });
    let (outcome, blocks) = enumerate_the_flow_through_the_library_with_the_keys(
        item,
        &flow_to_record,
        &observers_on_the_nodes,
        route,
        keys,
        gpu_judging_inputs,
    )?;
    if let Some(mut blocks) = blocks {
        blocks.expand_up_to = narrowing.map(|limit| limit.0);
        println!("{}", blocks.line());
    }
    Ok(outcome)
}

/// 崩溃枚举用例一段的展开：没缩（`narrowing` 是 `None`）照用例自己的，缩了取用例自己的与上限里小的那个
/// （[`ExpandUpToWritesPerSegment::narrowed_expansion`]）。
fn expansion_of_the_case_narrowed_by(
    narrowing: Option<ExpandUpToWritesPerSegment>,
    own_expansion: CrashPointReplaySegmentExpansion,
    segment: &[usize],
) -> CrashPointReplaySegmentExpansion {
    narrowing.map_or(own_expansion, |limit| {
        limit.narrowed_expansion(own_expansion, segment)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 缩域：没设上限照用例自己的展开；上限 4 时 5 次写的段不展开、4 次写的段照用例自己的；缩过域那一趟的块账行末尾接 ` expand_up_to=4`，
    /// 没缩的不接。判别力（`crates/mutations.tsv`）：缩了也照用例自己的展开（不取小的那个），这条红。
    #[test]
    fn a_narrowed_case_takes_the_smaller_expansion_and_its_blocks_line_names_the_limit() {
        let limit = Some(ExpandUpToWritesPerSegment(4));
        assert_eq!(
            expansion_of_the_case_narrowed_by(
                None,
                CrashPointReplaySegmentExpansion::EveryProperSubset,
                &[0; 5]
            ),
            CrashPointReplaySegmentExpansion::EveryProperSubset,
            "没缩：照用例自己的"
        );
        assert_eq!(
            expansion_of_the_case_narrowed_by(
                limit,
                CrashPointReplaySegmentExpansion::EveryProperSubset,
                &[0; 5]
            ),
            CrashPointReplaySegmentExpansion::NotExpanded,
            "5 次写超过上限 4：不展开"
        );
        assert_eq!(
            expansion_of_the_case_narrowed_by(
                limit,
                CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll,
                &[0; 4]
            ),
            CrashPointReplaySegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll,
            "4 次写不超过上限：照用例自己的甲二"
        );
        let blocks = CrashCaseBlocks {
            blocks: 3,
            reused: 1,
            fresh: 2,
            red: 0,
            refused: 0,
            states: 9,
            expand_up_to: Some(4),
        };
        assert_eq!(
            blocks.line(),
            "CRASH_CASE_BLOCKS blocks=3 reused=1 fresh=2 red=0 refused=0 states=9 expand_up_to=4"
        );
        assert_eq!(
            CrashCaseBlocks {
                blocks: 3,
                reused: 1,
                fresh: 2,
                red: 0,
                refused: 0,
                states: 9,
                expand_up_to: None,
            }
            .line(),
            "CRASH_CASE_BLOCKS blocks=3 reused=1 fresh=2 red=0 refused=0 states=9",
            "没缩不接"
        );
    }
}
