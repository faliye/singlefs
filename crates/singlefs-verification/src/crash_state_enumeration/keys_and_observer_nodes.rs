//! 观察者那几组判法单元的键与观察者挂到的节点：每个观察者那一组的键（判器的代码不进键），观察者点名的段落到哪些节点。

use super::*;

/// 这条用例这一趟挂着的每个观察者的几把版本（判定项标识、观察者引到的被检测产品代码与另读的输入；那一组的键在 `observer_keys` 点名的那一张键表里取）。判镜像的定义与 GPU 判器不取：
/// 镜像判定的版本由结论种类与判器引到的被检测产品代码定，两条路线同一把（那一组拒算时是这个进程的占位，开库时登成照判不入库）。`route` 定观察者交回的盘由哪个判器判（只进观察者那几把版本里判器种类那一格）。
pub(super) fn keys_of_the_item(
    route: CrashPointReplayJudgeRoute,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    observer_keys: ObserverJudgingUnitsKeyTable,
) -> KeysOfTheItem {
    let mut keys = KeysOfTheItem {
        observer_versions: Vec::with_capacity(observers_on_the_nodes.len()),
        refusal_sentences: Vec::new(),
        versions_kept_out_of_the_library: Vec::new(),
    };
    for observer in observers_on_the_nodes {
        let (versions, refusals, versions_kept_out) =
            observer_versions_keeping_the_refused_out_with_the_observer_keys_in(
                observer,
                route.judged_by(),
                observer_keys,
            );
        keys.refusal_sentences
            .extend(refusals.iter().map(|refusal| refusal.sentence()));
        keys.versions_kept_out_of_the_library
            .extend(versions_kept_out);
        keys.observer_versions.push(versions);
    }
    keys
}

/// 一条崩溃枚举用例的两个节点：它自己那个点（`<项名>.<周期>.<步序>` 都是它的）与末节点，设点文件就是用例文件。
pub(super) fn nodes_of_the_item(item: CrashPointReplayItem<'_>) -> Vec<NodeOfAnObserver> {
    vec![
        NodeOfAnObserver {
            setting_file: item.setting_file.to_string(),
            entry_name: item.name.to_string(),
        },
        NodeOfAnObserver {
            setting_file: item.setting_file.to_string(),
            entry_name: END_OF_FLOW_NODE_NAME.to_string(),
        },
    ]
}

/// 一条崩溃枚举用例自己挂的观察者照各自的挂法挂（[`CrashPointReplayObserverAttachment`]：整条流是它自己那个点与末节点，点名段的是罩着那几段的点；
/// 观察者代码文件是用例文件或点名的共用文件）；按来历挂的另由
/// `crate::crash_point_replay_provenance::observers_on_the_nodes_with_the_ones_attached_to_the_flow` 接在后面（整条流）。
///
/// # Errors
/// 观察者名字不合规矩；挂法点名的段集合是空的、段下标越过流的段数、点名的段一个点都罩不着。
pub(super) fn observers_on_the_nodes_of_the_item<'observer>(
    item: CrashPointReplayItem<'_>,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    observers: &[AttachedCrashPointReplayObserver<'observer>],
    observers_handing_back_images: &[AttachedCrashPointReplayObserverHandingBackAnImage<
        'observer,
    >],
) -> Result<Vec<ObserverOnTheNodes<'observer>>, String> {
    let observer_name = |name: &str| {
        ObserverName::new(name).map_err(|rejected| format!("观察者名字不合规矩：{rejected:?}"))
    };
    let mut observers_on_the_nodes = Vec::new();
    for attached in observers {
        let observer = attached.observer;
        observers_on_the_nodes.push(ObserverOnTheNodes {
            observer_name: observer_name(observer.name)?,
            observer_code_file: observer_code_file_of_the_attachment(item, attached.attachment)
                .to_string(),
            extra_inputs_digest: extra_inputs_digest_of_the_attachment(
                observer.extra_inputs_digest,
                attached.attachment.points,
            ),
            nodes: nodes_of_the_attachment(item, writes, segments, attached.attachment.points)?,
            observe: ObserverCode::CountsOnly(observer.observe),
        });
    }
    for attached in observers_handing_back_images {
        let observer = attached.observer;
        observers_on_the_nodes.push(ObserverOnTheNodes {
            observer_name: observer_name(observer.name)?,
            observer_code_file: observer_code_file_of_the_attachment(item, attached.attachment)
                .to_string(),
            extra_inputs_digest: extra_inputs_digest_of_the_attachment(
                observer.extra_inputs_digest,
                attached.attachment.points,
            ),
            nodes: nodes_of_the_attachment(item, writes, segments, attached.attachment.points)?,
            observe: ObserverCode::HandsBackAnImage {
                observe: observer.observe,
                deduplication: observer.deduplication,
            },
        });
    }
    Ok(observers_on_the_nodes)
}

/// 观察者代码文件：用例文件，或挂法点名的共用文件。
fn observer_code_file_of_the_attachment<'file>(
    item: CrashPointReplayItem<'file>,
    attachment: CrashPointReplayObserverAttachment<'file>,
) -> &'file str {
    match attachment.code_file {
        CrashPointReplayObserverCodeFile::TheCaseFile => item.setting_file,
        CrashPointReplayObserverCodeFile::SharedFile(shared_file) => shared_file,
    }
}

/// 观察者挂在哪几个节点上：整条流是它自己那个点与末节点（[`nodes_of_the_item`]），点名段的是罩着那几段的点（[`nodes_of_the_points_over_the_segments`]）。
///
/// # Errors
/// 同 [`nodes_of_the_points_over_the_segments`]。
fn nodes_of_the_attachment(
    item: CrashPointReplayItem<'_>,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    points: CrashPointReplayObserverPoints<'_>,
) -> Result<Vec<NodeOfAnObserver>, String> {
    match points {
        CrashPointReplayObserverPoints::EveryPointOfTheFlow => Ok(nodes_of_the_item(item)),
        CrashPointReplayObserverPoints::PointsOverTheSegments(named_segments) => {
            nodes_of_the_points_over_the_segments(item, writes, segments, named_segments)
        }
    }
}

/// 罩着点名那几段的点（照 [`crash_points_of_the_steps`] 同一种切法，点名就是计划里那个点的名字），一点一个节点；不含末节点。
///
/// # Errors
/// 段集合是空的；段下标越过流的段数；点名的段一个写都没有、一个点都罩不着。
fn nodes_of_the_points_over_the_segments(
    item: CrashPointReplayItem<'_>,
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    named_segments: &[usize],
) -> Result<Vec<NodeOfAnObserver>, String> {
    if named_segments.is_empty() {
        return Err(
            "观察者挂的段集合是空的：一个点都不挂的观察者什么都不看，不挂它就是".to_string(),
        );
    }
    if let Some(outside) = named_segments
        .iter()
        .find(|segment_index| **segment_index >= segments.len())
    {
        return Err(format!(
            "观察者挂的段 {outside} 不在这条流里（流一共 {} 段）",
            segments.len()
        ));
    }
    let named_writes: HashSet<usize> = named_segments
        .iter()
        .flat_map(|segment_index| segments[*segment_index].iter().copied())
        .collect();
    let nodes: Vec<NodeOfAnObserver> = crash_points_of_the_steps(item, writes, segments)
        .into_iter()
        .filter(|point| {
            point
                .writes
                .clone()
                .any(|write| named_writes.contains(&write))
        })
        .map(|point| NodeOfAnObserver {
            setting_file: point.code_file_in_repository,
            entry_name: point.name,
        })
        .collect();
    if nodes.is_empty() {
        return Err(format!(
            "观察者挂的段 {named_segments:?} 一个写都没有，一个点都罩不着"
        ));
    }
    Ok(nodes)
}

/// 进观察者版本的另读的输入：整条流照原样（结论键不动），点名段的并上段集合（[`extra_inputs_digest_with_the_named_segments`]）。
fn extra_inputs_digest_of_the_attachment(
    extra_inputs_digest: [u8; DIGEST_BYTES],
    points: CrashPointReplayObserverPoints<'_>,
) -> [u8; DIGEST_BYTES] {
    match points {
        CrashPointReplayObserverPoints::EveryPointOfTheFlow => extra_inputs_digest,
        CrashPointReplayObserverPoints::PointsOverTheSegments(named_segments) => {
            extra_inputs_digest_with_the_named_segments(extra_inputs_digest, named_segments)
        }
    }
}

/// 点名的段集合（去重、排序：同一组段换个写法版本不变）并上观察者自己另读的输入，取 SHA-256。
fn extra_inputs_digest_with_the_named_segments(
    extra_inputs_digest: [u8; DIGEST_BYTES],
    named_segments: &[usize],
) -> [u8; DIGEST_BYTES] {
    let named_segments_in_order: std::collections::BTreeSet<usize> =
        named_segments.iter().copied().collect();
    sha256_digest(
        format!(
            "singlefs crash state enumeration observer attached to named segments\nsegments={}\nextra_inputs={}\n",
            named_segments_in_order
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(","),
            hexadecimal_text(&extra_inputs_digest)
        )
        .as_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 观察者挂法单测的小流：两次单元写一段、一次单元写一段、一次根槽写一段，写区间首尾相接，一段一步、一步一个点，加末节点共四个点。
    fn writes_and_segments_of_three_steps() -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
        writes_and_segments_of(&[
            &[(StepKind::UnitWrite, 1, 0), (StepKind::UnitWrite, 2, 0)],
            &[(StepKind::UnitWrite, 3, 0)],
            &[(StepKind::RootRecordFua, 0, 1)],
        ])
    }

    /// 挂法单测里的观察者：什么都不看，另读的输入是一份写死的非零摘要（看的是挂法怎么改它）。
    const EXTRA_INPUTS_OF_THE_ATTACHED_PROBE: [u8; DIGEST_BYTES] = [7; DIGEST_BYTES];

    /// 挂法 `points`、代码文件取用例文件的那个观察者，在 `writes` / `segments` 那条流上挂成的样子（只挂调用方这一个，不接按来历挂的）。
    fn attached_probe_on_the_nodes<'points>(
        writes: &[RetainedWrite],
        segments: &[Vec<usize>],
        points: CrashPointReplayObserverPoints<'points>,
    ) -> Result<ObserverOnTheNodes<'points>, String> {
        let observe: &'static ObserveACrashState<'static> = &observe_no_crash_state;
        observers_on_the_nodes_of_the_item(
            BLOCK_ACCOUNT_PROBE_ITEM,
            writes,
            segments,
            &[AttachedCrashPointReplayObserver {
                observer: CrashPointReplayObserver {
                    name: "attached_probe",
                    extra_inputs_digest: EXTRA_INPUTS_OF_THE_ATTACHED_PROBE,
                    observe,
                },
                attachment: CrashPointReplayObserverAttachment {
                    points,
                    code_file: CrashPointReplayObserverCodeFile::TheCaseFile,
                },
            }],
            &[],
        )
        .map(|mut observers| observers.remove(0))
    }

    /// 场景：三步的小流，一个观察者只挂记录前那一次单元写与根槽写两段（段下标 1、2），另一个挂整条流。
    /// 预期：只挂两段的那一个只挂那两步的点，不挂头一步（前缀）的点，也不挂末节点；挂整条流的每个点与末节点都挂。
    /// 判别力（`crates/mutations.tsv`）：点名段的挂法照旧挂整条流、罩着点名段的点不筛，各让这一条红。
    #[test]
    fn an_observer_attached_to_named_segments_holds_only_the_points_over_those_segments_and_not_the_end_of_the_flow(
    ) {
        let (writes, segments) = writes_and_segments_of_three_steps();
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        let plan = flow_of_the_inputs(
            BLOCK_ACCOUNT_PROBE_ITEM,
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        )
        .with_the_flow(plan_crash_points);
        assert_eq!(plan.points.len(), 4, "前提：三步各一个点，加末节点");
        let on_two_segments = attached_probe_on_the_nodes(
            &writes,
            &segments,
            CrashPointReplayObserverPoints::PointsOverTheSegments(&[1, 2]),
        )
        .expect("点名的两段在流里");
        let on_the_whole_flow = attached_probe_on_the_nodes(
            &writes,
            &segments,
            CrashPointReplayObserverPoints::EveryPointOfTheFlow,
        )
        .expect("整条流挂得上");
        let account = observers_in_the_block_account(
            &plan,
            &[on_two_segments, on_the_whole_flow],
            &[
                ConclusionVersion([1; DIGEST_BYTES]),
                ConclusionVersion([2; DIGEST_BYTES]),
            ],
        );
        assert_eq!(
            account[0].holds_each_point,
            vec![false, true, true, false],
            "只挂点名的两段：那两步的点挂，前缀那一步的点与末节点都不挂（不录、不判、块账不要它的结论）"
        );
        assert_eq!(
            account[1].holds_each_point,
            vec![true; 4],
            "整条流：每个点与末节点都挂"
        );
    }

    /// 场景：挂法点名一份共用的代码文件，与不点名（用例文件）各取一次观察者代码文件。
    /// 预期：点名的取点名的那一份（几条用例挂同一个观察者时找登记表里同一行），不点名的取用例文件。
    /// 判别力（`crates/mutations.tsv`）：点名共用文件也取用例文件，这一条红。
    #[test]
    fn an_observer_attached_with_a_shared_code_file_takes_that_file_and_otherwise_the_case_file() {
        let shared_file = "crates/singlefs-verification/tests/common_crash_point_observers/mod.rs";
        assert_eq!(
            observer_code_file_of_the_attachment(
                BLOCK_ACCOUNT_PROBE_ITEM,
                CrashPointReplayObserverAttachment {
                    points: CrashPointReplayObserverPoints::EveryPointOfTheFlow,
                    code_file: CrashPointReplayObserverCodeFile::SharedFile(shared_file),
                },
            ),
            shared_file,
            "点名共用文件：观察者代码文件就是它"
        );
        assert_eq!(
            observer_code_file_of_the_attachment(
                BLOCK_ACCOUNT_PROBE_ITEM,
                CrashPointReplayObserverAttachment::EVERY_POINT_WITH_THE_CASE_FILE,
            ),
            BLOCK_ACCOUNT_PROBE_ITEM.setting_file,
            "不点名：观察者代码文件是用例文件"
        );
    }

    /// 场景：三块的小流（单元写那一步、根槽写那一步、末节点），一个观察者挂整条流、一个只挂根槽写那一段；判器结论（带恢复字）与逐盘自证槽数
    /// 每一块都存，挂整条流的那个观察者结论每一块都存，只挂一段的那个只在它挂的那一块上存。
    /// 预期：块账三块全罩住（只挂一段的观察者在它不挂的两块上不要结论，前缀那一块不因它缺而现判）；判段收工问缺不缺：一块都不缺；
    /// 从库重建计数照常读回 9 个状态，看过的状态数 9（至少一个观察者挂着的点上的状态）。
    /// 判别力（`crates/mutations.tsv`）：挂没挂这个点不筛（收工问缺、重建计数都问它在不挂的点上的结论）、看过的状态数只数每个观察者都挂着的点，各让这一条红；
    /// 先拿改环境的锁、后开库（反着的次序），开库取名额那一步当场 panic，这条红。
    #[test]
    fn the_block_account_the_finishing_question_and_the_tally_leave_out_the_points_an_attached_observer_does_not_hold(
    ) {
        with_the_unit_writes_flow_and_an_empty_library(
            "observer-attached-to-named-segments",
            |flow, plan, store| {
                // 先开库、后拿改环境的锁（`crate::crash_amplification::EnvironmentLockOfThisTestBinary` 的次序）
                let _environment =
                    crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
                assert_eq!(
                    plan.points.len(),
                    3,
                    "前提：单元写那一步、根槽写那一步、末节点"
                );
                let observers_on_the_nodes = [
                    attached_probe_on_the_nodes(
                        flow.writes,
                        flow.segments,
                        CrashPointReplayObserverPoints::EveryPointOfTheFlow,
                    )
                    .expect("整条流挂得上"),
                    attached_probe_on_the_nodes(
                        flow.writes,
                        flow.segments,
                        CrashPointReplayObserverPoints::PointsOverTheSegments(&[1]),
                    )
                    .expect("根槽写那一段在流里"),
                ];
                let whole_flow_version = ConclusionVersion([2; DIGEST_BYTES]);
                let one_segment_version = ConclusionVersion([3; DIGEST_BYTES]);
                let observers = observers_in_the_block_account(
                    plan,
                    &observers_on_the_nodes,
                    &[whole_flow_version, one_segment_version],
                );
                assert_eq!(
                    observers[1].holds_each_point,
                    vec![false, true, false],
                    "前提：只挂一段的那个只挂根槽写那一步"
                );
                let judgement_version = ConclusionVersion([1; DIGEST_BYTES]);
                let judgement_version_of_each_point = vec![judgement_version; plan.points.len()];
                let judged_conclusions =
                    crate::crash_amplification::ConclusionsStoredUnder::TheConclusionKind {
                        kind: ConclusionKind::ImageJudgement,
                        version_of_each_point: &judgement_version_of_each_point,
                    };
                for (block_position, (_point_name, local, key)) in
                    blocks_of_the_plan(plan).iter().enumerate()
                {
                    let states = local.end - local.start;
                    store_a_conclusion_block_of_the_unit_test(
                        store,
                        key,
                        &ConclusionKind::ImageJudgement,
                        &judgement_version,
                        states,
                        &[],
                        Some(crate::verdict_store::RecoveryWordsOfState(
                            [0; crate::crash_judge_conclusions::RECOVERY_WORDS_PER_STATE],
                        )),
                    );
                    store
                        .store_observer_block(
                            key,
                            &ConclusionKind::ImageJudgement,
                            &judgement_version,
                            b"self verified slots of the unit test",
                        )
                        .expect("存得下");
                    store_a_conclusion_block_of_the_unit_test(
                        store,
                        key,
                        &ConclusionKind::Observer,
                        &whole_flow_version,
                        states,
                        &[],
                        None,
                    );
                    if observers[1].holds_each_point[block_position] {
                        store_a_conclusion_block_of_the_unit_test(
                            store,
                            key,
                            &ConclusionKind::Observer,
                            &one_segment_version,
                            states,
                            &[],
                            None,
                        );
                    }
                }
                let every_block = |_block: u64| true;
                let account = blocks_in_the_library(
                    store,
                    plan,
                    &judged_conclusions,
                    &observers,
                    &every_block,
                )
                .expect("读得了库");
                assert_eq!(
                    (account.blocks, account.covered, account.red),
                    (3, 3, 0),
                    "块账：三块全罩住，前缀那一块不因只挂一段的观察者缺结论而现判"
                );
                let without_every_conclusion = blocks_of_the_flow_without_every_conclusion(
                    store,
                    plan,
                    &judged_conclusions,
                    &observers,
                )
                .expect("读得了库");
                assert_eq!(
                    (
                        without_every_conclusion.missing_an_observer_conclusion,
                        without_every_conclusion.missing_the_judgement_or_an_observer_conclusion
                    ),
                    (0, 0),
                    "判段收工问缺不缺：只问挂着的观察者，一块都不缺"
                );
                let (tally, observer_red) = crash_point_replay_tally_from_the_library(
                    store,
                    flow,
                    plan,
                    ConclusionKind::ImageJudgement,
                    &judgement_version_of_each_point,
                    &observers,
                )
                .expect("挂着的观察者的结论都在");
                assert_eq!(
                    (tally.states, tally.observed_states),
                    (9, 9),
                    "重建计数读回整条流 9 个状态；每个状态都有观察者挂着，看过的状态数 9"
                );
                assert_eq!(
                    observer_red.red_states_of_each_observer,
                    vec![0, 0],
                    "两个观察者都没有红"
                );
            },
        );
    }

    /// 场景：一个只记计数的观察者，它那一组只登在别的包（实验包）构建脚本写的那一张键表里（单测照构建脚本那一套写一张）；
    /// 这条用例判器那一半的键分别照点名的那一张、照验证包那一张取。
    /// 预期：点名那一张时观察者那一组不拒算（拒算的那几句里没有它的组名）；照验证包那一张时它拒算（没在 `OBSERVER_ENTRIES` 登记）。
    /// 判别力（`crates/mutations.tsv`）：取这条用例的键时观察者那几组改回照验证包那一张取，这条红。
    #[test]
    fn an_observer_keyed_in_the_table_of_another_package_is_not_refused_when_the_case_names_that_table(
    ) {
        let observer_name = "keyed_in_the_experiments";
        let observer = ObserverOnTheNodes {
            observer_name: ObserverName::new(observer_name).expect("名字合规矩"),
            observer_code_file: "crates/singlefs-implementation-experiments/src/bin/probe.rs"
                .to_string(),
            extra_inputs_digest: [0; DIGEST_BYTES],
            nodes: Vec::new(),
            observe: ObserverCode::CountsOnly(&observe_no_crash_state),
        };
        let group_name = crate::judging_units_entries::observer_group_name(
            &observer.observer_code_file,
            observer_name,
        );
        let table_text = String::leak(singlefs_unit_graph::entry_groups::entry_group_table_text(
            &[],
            &[singlefs_unit_graph::entry_groups::EntryGroupKey {
                group_name: group_name.clone(),
                verdict: singlefs_unit_graph::entry_groups::EntryGroupVerdict::Keyed {
                    key: singlefs_unit_graph::sha256::Sha256Digest([0x5a; DIGEST_BYTES]),
                },
                entries_not_compiled: Vec::new(),
                composition_lines: Vec::new(),
                watched_files: std::collections::BTreeSet::new(),
            }],
        ));
        let route = CrashPointReplayJudgeRoute::Cpu {
            why_not_the_gpu: "unit_test",
        };
        let refuses_the_observer = |keys: &KeysOfTheItem| {
            keys.refusal_sentences
                .iter()
                .any(|sentence| sentence.contains(&group_name))
        };
        let keyed_in_the_named_table = keys_of_the_item(
            route,
            std::slice::from_ref(&observer),
            ObserverJudgingUnitsKeyTable::CompiledIntoAnotherPackage { table_text },
        );
        assert!(
            !refuses_the_observer(&keyed_in_the_named_table),
            "点名那一张时观察者那一组取得出键：{:?}",
            keyed_in_the_named_table.refusal_sentences
        );
        let keyed_in_the_verification_table = keys_of_the_item(
            route,
            std::slice::from_ref(&observer),
            ObserverJudgingUnitsKeyTable::OfTheVerificationPackage,
        );
        assert!(
            refuses_the_observer(&keyed_in_the_verification_table),
            "照验证包那一张取时它没登记、拒算：{:?}",
            keyed_in_the_verification_table.refusal_sentences
        );
    }

    /// 场景：同一个观察者在三步的小流上换几种挂法：整条流；只挂根槽写那一段；挂记录前那一次单元写与根槽写两段；同两段换个写法（次序倒过来、重复一个）。
    /// 预期：整条流的另读的输入照原样（挂整条流的观察者结论键不动）；点名段的把段集合并进另读的输入；换了段集合，观察者结论版本跟着换，
    /// 只看代码的那一把与判它交回的盘的池级 checker 那一把不动；同一组段换个写法版本不变；判器块键从流算，挂法不进它。
    /// 判别力（`crates/mutations.tsv`）：点名段时段集合不并进另读的输入、整条流也并一份空段集合、段集合不去重排序，各让这一条红。
    #[test]
    fn changing_the_named_segments_of_an_observer_moves_only_its_conclusion_version_and_not_the_block_keys(
    ) {
        let (writes, segments) = writes_and_segments_of_three_steps();
        let on = |points| {
            attached_probe_on_the_nodes(&writes, &segments, points).expect("点名的段在流里")
        };
        let on_the_whole_flow = on(CrashPointReplayObserverPoints::EveryPointOfTheFlow);
        let on_the_root = on(CrashPointReplayObserverPoints::PointsOverTheSegments(&[2]));
        let on_two_segments = on(CrashPointReplayObserverPoints::PointsOverTheSegments(&[
            1, 2,
        ]));
        let on_two_segments_written_otherwise =
            on(CrashPointReplayObserverPoints::PointsOverTheSegments(&[
                2, 1, 2,
            ]));
        assert_eq!(
            on_the_whole_flow.extra_inputs_digest, EXTRA_INPUTS_OF_THE_ATTACHED_PROBE,
            "整条流：另读的输入照原样"
        );
        assert_ne!(
            on_the_root.extra_inputs_digest, EXTRA_INPUTS_OF_THE_ATTACHED_PROBE,
            "点名段：段集合并进另读的输入"
        );
        let versions_of = |observer: &ObserverOnTheNodes<'_>| {
            crate::crash_amplification::observer_versions_keeping_the_refused_out(
                observer,
                JudgedBy::CpuJudge,
            )
            .0
        };
        let (root_versions, two_segment_versions) =
            (versions_of(&on_the_root), versions_of(&on_two_segments));
        assert_ne!(
            root_versions.conclusion, two_segment_versions.conclusion,
            "换了段集合：观察者结论版本跟着换"
        );
        assert_eq!(
            (
                root_versions.observer_code,
                root_versions.pool_checker_version
            ),
            (
                two_segment_versions.observer_code,
                two_segment_versions.pool_checker_version
            ),
            "只看代码的那一把与池级 checker 那一把不随段集合变"
        );
        assert_eq!(
            versions_of(&on_two_segments_written_otherwise).conclusion,
            two_segment_versions.conclusion,
            "同一组段换个写法（次序、重复）版本不变"
        );
        let base = MemoryPool {
            devices: std::collections::BTreeMap::new(),
            device_size_in_bytes: 1 << 30,
        };
        let block_keys =
            block_keys_of_each_node(BLOCK_ACCOUNT_PROBE_ITEM, &base, &writes, &segments);
        assert_eq!(
            block_keys.len(),
            4,
            "判器块键从流排的计划取（三步加末节点），观察者挂法不是计划的输入，换挂法块键不变"
        );
    }

    /// 场景：观察者挂法点名一个空的段集合。
    /// 预期：录任何东西之前停下，说明点名「段集合是空的」。
    /// 判别力（`crates/mutations.tsv`）：空段集合不拦，这一条红（落到「一个点都罩不着」那一句）。
    #[test]
    fn an_observer_attached_to_an_empty_set_of_segments_is_refused() {
        let (writes, segments) = writes_and_segments_of_three_steps();
        let Err(refusal) = attached_probe_on_the_nodes(
            &writes,
            &segments,
            CrashPointReplayObserverPoints::PointsOverTheSegments(&[]),
        ) else {
            panic!("空段集合要停下");
        };
        assert!(
            refusal.contains("段集合是空的"),
            "停下的说明点名空段集合：{refusal}"
        );
    }

    /// 场景：观察者挂法点名的段下标等于流的段数（越过最后一段）。
    /// 预期：录任何东西之前停下，说明点名那一段不在流里，不越界 panic。
    /// 判别力（`crates/mutations.tsv`）：越界只拦大于段数的，这一条红（下标等于段数时越界 panic）。
    #[test]
    fn an_observer_attached_to_a_segment_past_the_end_of_the_flow_is_refused() {
        let (writes, segments) = writes_and_segments_of_three_steps();
        let Err(refusal) = attached_probe_on_the_nodes(
            &writes,
            &segments,
            CrashPointReplayObserverPoints::PointsOverTheSegments(&[segments.len()]),
        ) else {
            panic!("越过最后一段要停下");
        };
        assert!(
            refusal.contains("不在这条流里"),
            "停下的说明点名那一段不在流里：{refusal}"
        );
    }
}
