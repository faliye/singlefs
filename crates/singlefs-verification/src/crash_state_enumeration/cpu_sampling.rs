//! CPU 上的取样：按序号区间在 CPU 上判一批状态，取样的流同样经库录、读回。

use super::*;

/// 实验装置逐状态取样（`src/bin/` 下的 E170、E171 三份与它们那一类）：录入段把流写进库，判段读回、只在 CPU 上按序号走 `ordinals` 这一段，
/// 每个状态把镜像与 CPU 判器的判定交给 `judge_state`。`ordinals` 整块罩住的块，判完照 CPU 判器那一处（`JudgedBy::CpuJudge`）存进库
/// （「能走两段的都走两段」：取样的结论也入库，打一行 `CRASH_POINT_REPLAY_SAMPLED_CONCLUSIONS_STORED`）；不复用、不上卡：
/// 装置要的是每个状态恢复报告的全部字段，库里的结论不装它们，取样照旧逐状态现判。
/// `item.name` 只进行里报的名字（[`CRASH_POINT_NAME_OF_A_SAMPLED_FLOW`]）：同一条流按不同的项名、不同的序号段取样，录进库的字节相同，只录一次、只读回一次。
///
/// # Panics
/// 开不了库、写不进、读不回；`ordinals` 越过整条流的状态数。
#[allow(
    clippy::too_many_arguments,
    reason = "前六个是同一条流的几样输入，其余是展开、序号段与回调"
)]
pub fn sample_crash_states_on_the_cpu(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: &[PublishedVersion],
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(u64, &CrashImage<'_>, &CrashStateJudgement),
) -> StageOutcome<u64> {
    sample_crash_states_on_the_cpu_against_the_version_table(
        item,
        base,
        writes,
        segments,
        judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(versions),
        expansion,
        ordinals,
        judge_state,
    )
}

/// 同 [`sample_crash_states_on_the_cpu`]，oracle 对一张版本表判（带「没有文件的那几版」的删除流）。
///
/// # Panics
/// 同 [`sample_crash_states_on_the_cpu`]。
#[allow(
    clippy::too_many_arguments,
    reason = "与 sample_crash_states_on_the_cpu 同一组参数，版本表换成带「没有文件的那几版」的那一张"
)]
pub fn sample_crash_states_on_the_cpu_against_the_version_table(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    ordinals: Range<u64>,
    judge_state: &mut dyn FnMut(u64, &CrashImage<'_>, &CrashStateJudgement),
) -> StageOutcome<u64> {
    let flow_to_record = flow_of_the_inputs(
        item_naming_the_crash_points_of_a_sampled_flow(item),
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
    );
    let sampled_ordinals = ordinals.clone();
    sample_through_the_library(item, &flow_to_record, &sampled_ordinals, |flow, note| {
        crate::crash::judge_crash_state_range_of_the_read_back_flow(
            flow.base,
            flow.writes,
            flow.segments,
            flow.judged_root_index,
            flow.version_table(),
            flow.expansion,
            ordinals,
            &mut |ordinal, publish, image, judgement| {
                note(ordinal, publish, judgement);
                judge_state(ordinal, image, judgement);
            },
        )
    })
}

/// 一条线程切几段：段切得比线程数细、轮着领，靠后的状态历史长、判得慢，按线程数等分时最后一段拖住整趟。
const SAMPLING_SLICES_PER_THREAD: u64 = 8;

/// 同 [`sample_crash_states_on_the_cpu_against_the_version_table`]，判段分 `threads` 条线程：`ordinals` 切成每线程
/// [`SAMPLING_SLICES_PER_THREAD`] 段轮着领，`judge_state` 跨线程调、按序号各报各的，调用方要按序号排；录入段只录一次，
/// 流只从库里读回一次、各线程共用；判段的撕裂表也只建一份、各线程借用（判完核恰好建了一份，不是一份就 panic）。
/// 交回的与单线程那一支相同：整条流的状态数（不是 `ordinals` 的长度）；各线程判的状态数合起来等于 `ordinals` 的长度，不等就 panic。
///
/// # Panics
/// 同 [`sample_crash_states_on_the_cpu`]；判定线程 panic。
#[allow(
    clippy::too_many_arguments,
    reason = "与 sample_crash_states_on_the_cpu_against_the_version_table 同一组参数，多一个线程数、判法换成跨线程的"
)]
pub fn sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
    item: CrashPointReplayItem<'_>,
    base: &MemoryPool,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    judged_root_index: usize,
    versions: CrashPointReplayVersionTable<'_>,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
    ordinals: Range<u64>,
    threads: NonZeroUsize,
    judge_state: &(dyn Fn(u64, &CrashImage<'_>, &CrashStateJudgement) + Sync),
) -> StageOutcome<u64> {
    let flow_to_record = flow_of_the_inputs(
        item_naming_the_crash_points_of_a_sampled_flow(item),
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        expansion,
    );
    let sampled_ordinals = ordinals.clone();
    sample_through_the_library(item, &flow_to_record, &sampled_ordinals, |flow, note| {
        let state_count = ordinals.end.saturating_sub(ordinals.start);
        let threads = u64::try_from(threads.get()).expect("线程数装得进 u64");
        let slice = state_count
            .div_ceil(threads * SAMPLING_SLICES_PER_THREAD)
            .max(1);
        let slice_count = state_count.div_ceil(slice);
        let next_slice = AtomicU64::new(0);
        // 撕裂表只随流定，一份可上 GB（用过的盘上重做 mkfs 那条流两块盘各一次清零罩着旧 journal 环，一份留着 1.5 GiB）：
        // 判段只建这一份、各线程经同一份排好次序的状态借用，不在每一段里各建一份（`TornImageTablesOfTheReadBackFlow` 的文档
        // 「一个阶段只建一份」；每段各建时三十一条线程同时留着三十一份，挤掉了 40G 的内存上限）。建了几份数在这里，判完核恰好一份。
        let torn_image_tables_built = AtomicU64::new(0);
        let build_the_torn_image_tables_of_the_flow = || {
            torn_image_tables_built.fetch_add(1, Ordering::SeqCst);
            crate::crash::TornImageTablesOfTheReadBackFlow::of_the_read_back_flow(
                flow.base,
                flow.writes,
                flow.segments,
            )
        };
        let torn_image_tables_of_the_flow = build_the_torn_image_tables_of_the_flow();
        let crash_states_of_the_flow = torn_image_tables_of_the_flow.crash_states_planned_by(
            flow.base,
            flow.writes,
            flow.segments,
            flow.expansion,
            &crate::crash::no_segment_slices(flow.segments.len()),
        );
        if slice_count == 0 {
            // 空序号段：一个状态都不判，照单线程那一支交回整条流的状态数
            return crate::crash::judge_crash_state_range_of_the_read_back_flow_with_slices(
                &crash_states_of_the_flow,
                flow.judged_root_index,
                flow.version_table(),
                ordinals.clone(),
                &mut |_ordinal, _publish, _image, _judgement| {},
            );
        }
        let (judged, states_of_the_whole_flow) = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads.min(slice_count))
                .map(|_| {
                    let next_slice = &next_slice;
                    let ordinals = &ordinals;
                    let crash_states_of_the_flow = &crash_states_of_the_flow;
                    scope.spawn(move || {
                        let mut judged_here = 0u64;
                        let mut states_of_the_whole_flow: Option<u64> = None;
                        loop {
                            let index = next_slice.fetch_add(1, Ordering::SeqCst);
                            if index >= slice_count {
                                break;
                            }
                            let range = ordinals.start + index * slice
                                ..(ordinals.start + (index + 1) * slice).min(ordinals.end);
                            // 交回的是整条流的状态数（与单线程那一支同一个契约），不是这一段判了几个：判了几个按段的长度数
                            let states = crate::crash::judge_crash_state_range_of_the_read_back_flow_with_slices(
                                crash_states_of_the_flow,
                                flow.judged_root_index,
                                flow.version_table(),
                                range.clone(),
                                &mut |ordinal, publish, image, judgement| {
                                    note(ordinal, publish, judgement);
                                    judge_state(ordinal, image, judgement);
                                },
                            );
                            assert!(
                                range.end <= states,
                                "序号段 {range:?} 越过整条流的 {states} 个状态"
                            );
                            assert!(
                                states_of_the_whole_flow.is_none_or(|known| known == states),
                                "各段看到的整条流状态数不同：{states_of_the_whole_flow:?} 与 {states}"
                            );
                            states_of_the_whole_flow = Some(states);
                            judged_here += range.end - range.start;
                        }
                        (judged_here, states_of_the_whole_flow)
                    })
                })
                .collect();
            let mut judged = 0u64;
            let mut states_of_the_whole_flow: Option<u64> = None;
            for handle in handles {
                let (judged_here, states_here) = handle.join().expect("判定线程不 panic");
                judged += judged_here;
                if let Some(states_here) = states_here {
                    assert!(
                        states_of_the_whole_flow.is_none_or(|known| known == states_here),
                        "各线程看到的整条流状态数不同：{states_of_the_whole_flow:?} 与 {states_here}"
                    );
                    states_of_the_whole_flow = Some(states_here);
                }
            }
            (
                judged,
                states_of_the_whole_flow.expect("序号段不空，至少一条线程领到了段"),
            )
        });
        assert_eq!(
            torn_image_tables_built.load(Ordering::SeqCst),
            1,
            "判段的撕裂表只建一份、各线程借用，不按段各建一份"
        );
        assert_eq!(judged, state_count, "各线程判的状态数合起来是序号段的长度");
        states_of_the_whole_flow
    })
}

/// 取样那一趟判过的一个状态交给库的那一份：序号、它归哪次发布、CPU 判器的判定（整块都判到的块照 CPU 判器那一处存进库）。
type NoteASampledState<'note> =
    dyn Fn(u64, CrashPointReplayPublishOfState, &CrashStateJudgement) + Sync + 'note;

/// 取样录进库的流里崩溃点的名字（`<名字>.<周期>.<步序>`，[`crash_points_of_the_steps`]）：不用调用方的项名。崩溃点名字进录下的流的字节
/// （[`RecordedFlow::to_bytes`]），字节定录入键与流的摘要；装置一条流取样几十次、每次一个项名时，用项名就是每次重录、重读一遍整条流。
/// 块键只由内容定（[`crate::crash_amplification::PlannedCrashPoint::block_key`]），名字换了块键与判定结论都不动。路径索引的键带这个名字，只作索引。
const CRASH_POINT_NAME_OF_A_SAMPLED_FLOW: &str = "sampled_flow";

/// 取样录流时用的那一项：设点文件照调用方，名字固定（[`CRASH_POINT_NAME_OF_A_SAMPLED_FLOW`]）。
fn item_naming_the_crash_points_of_a_sampled_flow(
    item: CrashPointReplayItem<'_>,
) -> CrashPointReplayItem<'_> {
    CrashPointReplayItem {
        name: CRASH_POINT_NAME_OF_A_SAMPLED_FLOW,
        setting_file: item.setting_file,
    }
}

/// 取样的两段：录入段把 `flow_to_record` 按摘要写进崩溃点重放枚举那一类的库、录入键下登清单；判段按录入键读清单、按摘要读回流，
/// 只把读回的那一份交 `judge_the_read_back_flow`，它每判一个状态交给第二个参数：取样序号段整块罩住的块，判完照 CPU 判器那一处
/// （`JudgedBy::CpuJudge`，结论版本照计划各点那一把）存进库（「能走两段的都走两段」：取样的结论也入库）。
/// 同一条连接上同一摘要的流录与读回只走一次（[`CrashLibrary::recorded_flow_and_its_plan_read_back_once`]）：留的是判段从库里读回的那一份，
/// 录入段内存里的流不留。只录那一趟交 `RecordedOnly`，什么都不留。
fn sample_through_the_library(
    item: CrashPointReplayItem<'_>,
    flow_to_record: &FlowToRecord<'_>,
    ordinals: &Range<u64>,
    judge_the_read_back_flow: impl FnOnce(&FlowOfTheLibrary<'_>, &NoteASampledState<'_>) -> u64,
) -> StageOutcome<u64> {
    let item_name = item_name_of(item);
    let recorded_bytes = flow_to_record.recorded_bytes();
    sample_the_recorded_bytes_through_the_library(
        item,
        &item_name,
        flow_to_record,
        &recorded_bytes,
        ordinals,
        judge_the_read_back_flow,
    )
    .unwrap_or_else(|refusal| panic!("崩溃点重放取样「{}」：{refusal}", item.name))
}

/// [`sample_through_the_library`] 的本体：开库、录并读回一次、判。
///
/// # Errors
/// 开不了库；录入段、登清单、读清单、读回各自的错；取样的结论写不进库。
fn sample_the_recorded_bytes_through_the_library(
    item: CrashPointReplayItem<'_>,
    item_name: &ItemName,
    flow_to_record: &FlowToRecord<'_>,
    recorded_bytes: &[u8],
    ordinals: &Range<u64>,
    judge_the_read_back_flow: impl FnOnce(&FlowOfTheLibrary<'_>, &NoteASampledState<'_>) -> u64,
) -> Result<StageOutcome<u64>, String> {
    let library = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)?;
    let flow_digest = sha256_digest(recorded_bytes);
    let payload_bytes = u64::try_from(recorded_bytes.len()).expect("字节数装得进 u64");
    let read_back_once = library.recorded_flow_and_its_plan_read_back_once(&flow_digest, || {
        two_stages_under_the_recording_key(
            &library,
            RunStage::of_this_process(),
            item_name,
            &recording_key_of_the_scenario_and_the_inputs(
                item_name.as_str(),
                &FlowToRecord::recording_inputs_digest(recorded_bytes, false),
            ),
            |library| flow_to_record.record_into(library, false, recorded_bytes),
            |library, manifest| {
                let digest_in_the_manifest = manifest.digest_named(MANIFEST_FLOW)?;
                assert_eq!(
                    digest_in_the_manifest, flow_digest,
                    "录入键由这份字节的摘要算出：同一把键下的清单指到同一份流"
                );
                Ok(StageOutcome::Judged((
                    RecordedFlowAndItsPlan::read_outside_the_library_lock(
                        library,
                        &digest_in_the_manifest,
                    )?,
                    payload_bytes,
                )))
            },
        )
    })?;
    let flow_and_plan = match read_back_once {
        StageOutcome::RecordedOnly => return Ok(StageOutcome::RecordedOnly),
        StageOutcome::JudgedMyShareOnly {
            blocks_judged_here,
            blocks_left_to_other_hosts,
        } => {
            return Ok(StageOutcome::JudgedMyShareOnly {
                blocks_judged_here,
                blocks_left_to_other_hosts,
            })
        }
        StageOutcome::Judged(flow_and_plan) => flow_and_plan,
    };
    let flow = flow_and_plan.flow_read_back();
    let plan = flow_and_plan.plan_of_the_flow_read_back();
    // 取样的结论记在镜像判定那一把版本下（由结论种类与判器引到的被检测产品代码定，判器的代码不进键）
    let conclusion_versions = ConclusionVersionsOfThePlan::of_the_plan(plan);
    let sampled_blocks = CpuJudgementsOfSampledBlocks::for_the_ordinals(
        &library,
        plan,
        conclusion_versions.stored_under(JudgedBy::CpuJudge),
        ordinals,
    )
    .map_err(|error| format!("读判定块失败：{error:?}"))?;
    let sampled = std::sync::Mutex::new((sampled_blocks, None::<String>));
    let note =
        |ordinal: u64, publish: CrashPointReplayPublishOfState, judgement: &CrashStateJudgement| {
            let mut sampled = sampled
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if sampled.1.is_some() {
                return;
            }
            if let Err(error) = sampled
                .0
                .note_a_state(&library, ordinal, publish, judgement)
            {
                sampled.1 = Some(format!("取样的 CPU 结论写不进库：{error:?}"));
            }
        };
    let states = judge_the_read_back_flow(&flow, &note);
    let (sampled_blocks_after_judging, failure) = sampled
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(failure) = failure {
        return Err(failure);
    }
    println!(
        "CRASH_POINT_REPLAY_SAMPLED_CONCLUSIONS_STORED item={} judged_by=cpu_judge blocks_stored_here={} blocks_already_stored={}",
        item.name,
        sampled_blocks_after_judging.blocks_stored_here,
        sampled_blocks_after_judging.blocks_already_stored
    );
    Ok(StageOutcome::Judged(states))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crash_amplification::CrashAmplificationEnvironment;

    /// 取样单测的小流（三次单元写一段、一次根槽写一段，9 个状态）：底座是空池，只走录、读回、排计划，不真判状态。
    fn inputs_of_a_small_sampled_flow() -> (MemoryPool, Vec<RetainedWrite>, Vec<Vec<usize>>, usize)
    {
        let (writes, segments) = writes_and_segments_of(&[
            &[
                (StepKind::UnitWrite, 1, 0),
                (StepKind::UnitWrite, 2, 0),
                (StepKind::UnitWrite, 3, 0),
            ],
            &[(StepKind::RootRecordFua, 0, 1)],
        ]);
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        (base, writes, segments, judged_root_index)
    }

    /// 取样的项名不进录下的字节：两个项名录出同一份字节（同一把录入键、同一个流摘要）。直接用项名当崩溃点名字时字节不同（项名真的进字节），
    /// 块键却相同：换名字不动块键，判定结论照旧按块键复用。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 取样录流照用调用方的项名」这一行让这一条红。
    #[test]
    fn a_sampled_flow_records_the_same_bytes_and_block_keys_whatever_the_item_is_named() {
        let (base, writes, segments, judged_root_index) = inputs_of_a_small_sampled_flow();
        let item_named = |name: &'static str| CrashPointReplayItem {
            name,
            setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
        };
        let flow_under = |item: CrashPointReplayItem<'static>| {
            flow_of_the_inputs(
                item,
                &base,
                &writes,
                &segments,
                judged_root_index,
                CrashPointReplayVersionTable::with_a_file_only(&[]),
                &every_proper_subset,
            )
        };
        let block_keys_of = |flow_to_record: &FlowToRecord<'_>| {
            flow_to_record.with_the_flow(|flow| {
                blocks_of_the_plan(&plan_crash_points(flow))
                    .into_iter()
                    .map(|(_point_name, _states, block_key)| block_key)
                    .collect::<Vec<crate::verdict_store::VerdictBlockKey>>()
            })
        };
        let named_by_the_item = flow_under(item_named("sampled-probe-0"));
        let also_named_by_the_item = flow_under(item_named("sampled-probe-1"));
        assert_ne!(
            named_by_the_item.recorded_bytes(),
            also_named_by_the_item.recorded_bytes(),
            "前提：崩溃点名字进录下的字节，项名当名字时每个项名一份字节"
        );
        let sampled = flow_under(item_naming_the_crash_points_of_a_sampled_flow(item_named(
            "sampled-probe-0",
        )));
        let sampled_under_another_name = flow_under(
            item_naming_the_crash_points_of_a_sampled_flow(item_named("sampled-probe-1")),
        );
        assert_eq!(
            sampled.recorded_bytes(),
            sampled_under_another_name.recorded_bytes(),
            "取样的项名不进录下的字节"
        );
        assert_eq!(
            block_keys_of(&sampled),
            block_keys_of(&named_by_the_item),
            "崩溃点换名字，块键不动"
        );
    }

    /// 连接开着，同一条流按两个项名、两次取样：只录一次、只从库里读回一次，整条流的状态数两次相同。
    /// 这一条也钉录入段与判段读回路上的纯计算都在锁外（`assert_this_thread_holds_no_library_lock`）。
    /// 判别力（`crates/mutations.tsv`）：「判定库锁收窄 读回的流留着也不用」「判定库锁收窄 录入段排计划挪回锁里」「判定库锁收窄 录入段算块键挪回锁里」
    /// 「判定库锁收窄 录入段算流摘要挪回锁里」「判定库锁收窄 解流挪回锁里」「判定库锁收窄 判段排计划挪回锁里」「判定库锁收窄 取样挑块挪回锁里」这几行让这一条红。
    #[test]
    fn sampling_one_flow_under_two_item_names_records_and_reads_it_back_once() {
        // 一次性库留着：要从空库数录了几份清单、读回几次
        let disposable = crate::crash_library::DisposableLibrary::new("sampled-flow-read-once")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
        let _environment = CrashAmplificationEnvironment::cleared_then_set(&[]);
        let library_kept_open = CrashLibrary::open(CrashLibraryClass::CrashStateEnumeration)
            .expect("开得了设成这个进程那一份的一次性库");
        let (base, writes, segments, judged_root_index) = inputs_of_a_small_sampled_flow();
        for name in ["sampled-probe-0", "sampled-probe-1"] {
            let states = sample_crash_states_on_the_cpu(
                CrashPointReplayItem {
                    name,
                    setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
                },
                &base,
                &writes,
                &segments,
                judged_root_index,
                &[],
                &every_proper_subset,
                0..0,
                &mut |ordinal, _image, _judgement| panic!("空序号段不判状态，判到了 {ordinal}"),
            )
            .judged()
            .expect("两段先后跑：判段交回整条流的状态数");
            assert_eq!(states, 9, "整条流的状态数");
        }
        assert_eq!(
            library_kept_open.recorded_flows_read_over_the_connection(),
            1,
            "同一条流只从库里读回一次"
        );
        assert_eq!(
            library_kept_open
                .lock()
                .recorded_run_manifest_count()
                .expect("数得出清单"),
            1,
            "同一条流只登一份清单"
        );
    }
}
