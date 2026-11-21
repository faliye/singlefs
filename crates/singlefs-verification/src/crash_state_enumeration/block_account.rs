//! 块账：这一份分到哪些块、块里哪些状态、哪些观察者的结论要齐，与 `CRASH_CASE_BLOCKS` 那一行。

use super::*;

// ============================== 两段 ==============================

/// 一条崩溃枚举用例跑完打的那一行的头（前五个字段次序照 `scripts/gate/admission.py` 的 `CRASH_CASE_BLOCKS_FIELDS`，它按字段名读、不看后面多出的
/// `states=`）。
pub const CRASH_CASE_BLOCKS_LINE_PREFIX: &str = "CRASH_CASE_BLOCKS";

/// 一条崩溃枚举用例这一趟这台机器那一份的块账（[`CRASH_CASE_BLOCKS_LINE_PREFIX`] 那一行）：`blocks = reused + fresh + refused`。
/// 判段那一趟：`reused` 是键相同、库里有不红判定的块，`fresh` 是这一趟现判的（记着红的块不复用、这一趟现判，算在这里：
/// 「记红、撤绿、下趟现判」）。`refused` 是判器那一半键拒算、照判不入库的，`red` 是判红的块（现判的与库里记着的都算）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrashCaseBlocks {
    pub blocks: u64,
    pub reused: u64,
    pub fresh: u64,
    pub red: u64,
    pub refused: u64,
    /// 这一份的块里一共几个状态（[`states_of_the_blocks_of_this_share`]，出口 10「每项态数现数」）。
    pub states: u64,
    /// 这一趟缩过域时用的展开上限（`crate::crash_amplification::EXPAND_UP_TO_ENVIRONMENT_VARIABLE` 的值）；没缩是 `None`。
    pub expand_up_to: Option<usize>,
}

impl CrashCaseBlocks {
    /// `CRASH_CASE_BLOCKS blocks=<n> reused=<n> fresh=<n> red=<n> refused=<n> states=<n>`，缩过域的那一趟末尾再接 ` expand_up_to=<n>`。
    #[must_use]
    pub fn line(&self) -> String {
        let counted_blocks = format!(
            "{CRASH_CASE_BLOCKS_LINE_PREFIX} blocks={} reused={} fresh={} red={} refused={}",
            self.blocks, self.reused, self.fresh, self.red, self.refused
        );
        let line = format!("{counted_blocks} states={}", self.states);
        match self.expand_up_to {
            Some(expand_up_to) => format!("{line} expand_up_to={expand_up_to}"),
            None => line,
        }
    }
}

/// 这一份的块里一共几个状态（`CRASH_CASE_BLOCKS` 行的 `states=`；出口 10「每项态数现数」：判段的块账顺带数，录入段本来就排了计划）：
/// 块是点内序号区间，状态数是区间长，块号的数法与 [`blocks_of_this_host`] 相同。分片的段（C790）只数这一趟的片，分片跑的流数出来的是这一趟那一截。
fn states_of_the_blocks_of_this_share(plan: &CrashPlan, is_mine: &dyn Fn(u64) -> bool) -> u64 {
    let mut states = 0u64;
    let mut block_sequence = 0u64;
    for point in &plan.points {
        for local in point.blocks() {
            if is_mine(block_sequence) {
                states += local.end - local.start;
            }
            block_sequence += 1;
        }
    }
    states
}

/// 挂着的每个观察者在块账里的样子（[`ObserverInTheBlockAccount`]），与 `observers_on_the_nodes` 同序。
///
/// # Panics
/// 观察者与它们的结论版本不一样多。
pub(super) fn observers_in_the_block_account(
    plan: &CrashPlan,
    observers_on_the_nodes: &[ObserverOnTheNodes<'_>],
    observer_conclusion_versions: &[ConclusionVersion],
) -> Vec<ObserverInTheBlockAccount> {
    assert_eq!(
        observers_on_the_nodes.len(),
        observer_conclusion_versions.len(),
        "每个观察者各一把结论版本、同序"
    );
    observers_on_the_nodes
        .iter()
        .zip(observer_conclusion_versions)
        .map(|(observer, conclusion)| ObserverInTheBlockAccount {
            holds_each_point: plan
                .points
                .iter()
                .map(|point| {
                    observer
                        .nodes
                        .iter()
                        .any(|node| node.holds_the_point(point))
                })
                .collect(),
            conclusion: *conclusion,
        })
        .collect()
}

/// 挂在计划第 `point_index` 个点上的观察者：（在 `observers` 里的位置，观察者结论的种类与这一把版本）。不挂这个点的观察者（只挂点名段的）
/// 在这个点上不要结论：判段收工问缺不缺、重建计数读结论都只问它。
pub(super) fn observers_holding_the_point(
    observers: &[ObserverInTheBlockAccount],
    point_index: usize,
) -> Vec<(usize, (ConclusionKind, ConclusionVersion))> {
    observers
        .iter()
        .enumerate()
        .filter(|(_position, observer)| observer.holds_each_point[point_index])
        .map(|(position, observer)| (position, (ConclusionKind::Observer, observer.conclusion)))
        .collect()
}

/// 只读库、一块不判：`is_counted` 点到的块里几块罩住、几块红。一块算罩住要两样都在：判器结论照判段「要不要判」同一个谓词在
/// （`ConclusionsStoredUnder::is_stored_for_reuse`，连同判器交回的逐盘自证槽数；记着红的不算）；挂在这个点上的每个观察者在
/// 这一把版本下都有结论、且不记着红（观察者版本换了、结论记着红就不算：判段照 `crate::crash_amplification` 观察者那一半同一个规矩现拼）。
/// 红块是判器结论里有违例、或任一观察者结论里存着违例的块；红块一块都不算罩住。判段的块账照它数。
///
/// # Errors
/// 库读失败。
pub(super) fn blocks_in_the_library(
    store: &VerdictStore,
    plan: &CrashPlan,
    judged_conclusions: &crate::crash_amplification::ConclusionsStoredUnder<'_>,
    observers: &[ObserverInTheBlockAccount],
    is_counted: &dyn Fn(u64) -> bool,
) -> Result<BlocksInTheLibrary, String> {
    let mut account = BlocksInTheLibrary {
        blocks: 0,
        covered: 0,
        red: 0,
        states: states_of_the_blocks_of_this_share(plan, is_counted),
    };
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        let observer_conclusions_of_the_point: Vec<(ConclusionKind, ConclusionVersion)> = observers
            .iter()
            .filter(|observer| observer.holds_each_point[point_index])
            .map(|observer| (ConclusionKind::Observer, observer.conclusion))
            .collect();
        for local in point.blocks() {
            let this_block = block_sequence;
            block_sequence += 1;
            if !is_counted(this_block) {
                continue;
            }
            account.blocks += 1;
            let key = point.block_key(&local);
            let judgement_is_reusable = judged_conclusions
                .is_stored_for_reuse(store, point_index, &key)
                .map_err(|error| format!("查判器结论失败：{error:?}"))?;
            let mut observer_conclusions_not_reusable = store
                .conclusions_missing_for_block(&key, &observer_conclusions_of_the_point)
                .map_err(|error| format!("查观察者结论失败：{error:?}"))?;
            let mut is_red = !judged_conclusions
                .violations_in_block(store, point_index, &key)
                .map_err(|error| format!("读违例失败：{error:?}"))?
                .is_empty();
            for (kind, version) in &observer_conclusions_of_the_point {
                let observed = store
                    .read_conclusion_block(&key, kind, version)
                    .map_err(|error| format!("读观察者结论失败：{error:?}"))?;
                if observed.is_some_and(|observed| !observed.violations.is_empty()) {
                    is_red = true;
                    // 记着红的观察者结论不复用、下一趟现判（「记红、撤绿、下趟现判」）：与缺的一样不算罩住
                    observer_conclusions_not_reusable.push((kind.clone(), *version));
                }
            }
            if judgement_is_reusable && observer_conclusions_not_reusable.is_empty() {
                account.covered += 1;
            }
            if is_red {
                account.red += 1;
            }
        }
    }
    Ok(account)
}

/// 这台机器领的块（块序号 = 全部点的块按次序编号）：两台分片与崩溃放量同一种读法，按权重平滑轮转。
///
/// # Errors
/// 分片与权重写法错。
pub(super) fn blocks_of_this_host(plan: &CrashPlan) -> Result<HashSet<u64>, String> {
    let (share, host_weights) =
        crate::crash_amplification::host_share_and_weights_from_the_environment()?;
    let host_shares = WeightedShares::new(host_weights);
    let total_blocks: u64 = plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("块数装得进 u64"))
        .sum();
    Ok((0..total_blocks)
        .filter(|block| host_shares.owner_of_block(*block) == share.index)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 两台分活的权重（本机、第二台），与双机四条用例那一趟的多机配置相同。
    const HOST_WEIGHTS_OF_THE_UNIT_TEST: [u64; 2] = [2, 3];

    /// 场景：三块的小流，每一块判器结论都存（干净、带恢复字）、观察者结论都存，其中按权重 2,3 归第二台（不归 0/2 那一份）的头一块的观察者结论
    /// 存一个违例状态（第二台那一份判出、写进服务那一份的那种块）。
    /// 预期：从库重建整条流的计数照常读回 9 个状态；整条流观察者红状态数 = 1，头一块存着红的就是那一块。
    /// 判别力（`crates/mutations.tsv`）：累加观察者红状态数那一步加 0（不分片的那一趟读不到第二台领的块上存着的观察者红），这条红。
    #[test]
    fn the_whole_flow_tally_counts_the_observer_red_stored_on_a_block_the_other_host_owns() {
        with_the_unit_writes_flow_and_an_empty_library(
            "observer-red-of-the-whole-flow",
            |flow, plan, store| {
                let judgement_version = ConclusionVersion([1; DIGEST_BYTES]);
                let observer_version = ConclusionVersion([2; DIGEST_BYTES]);
                let host_shares = WeightedShares::new(HOST_WEIGHTS_OF_THE_UNIT_TEST.to_vec());
                let blocks = blocks_of_the_plan(plan);
                let (red_point_name, red_local, red_key) = (0u64..)
                    .zip(&blocks)
                    .find(|(block_sequence, _)| host_shares.owner_of_block(*block_sequence) == 1)
                    .map(|(_, block)| block.clone())
                    .expect("前提：有一块按权重 2,3 归第二台");
                for (_point_name, local, key) in &blocks {
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
                    let red_offsets: &[u64] = if *key == red_key { &[0] } else { &[] };
                    store_a_conclusion_block_of_the_unit_test(
                        store,
                        key,
                        &ConclusionKind::Observer,
                        &observer_version,
                        states,
                        red_offsets,
                        None,
                    );
                }
                let (tally, observer_red) = crash_point_replay_tally_from_the_library(
                    store,
                    flow,
                    plan,
                    ConclusionKind::ImageJudgement,
                    &vec![judgement_version; plan.points.len()],
                    &[ObserverInTheBlockAccount {
                        holds_each_point: vec![true; plan.points.len()],
                        conclusion: observer_version,
                    }],
                )
                .expect("库里每一块两种结论都在");
                assert_eq!(tally.states, 9, "前提：整条流 9 个状态都读回");
                assert_eq!(
                    observer_red.red_states_of_each_observer,
                    vec![1],
                    "整条流观察者红状态数：第二台领的那一块上存着的那一个"
                );
                assert_eq!(
                    observer_red.first_red_block,
                    Some(FirstObserverRedBlock {
                        observer_position: 0,
                        point_name: red_point_name,
                        block_start: red_local.start,
                    }),
                    "头一块存着红的就是第二台领的那一块"
                );
            },
        );
    }

    /// 场景：三块的小流，一个观察者挂在头一个点与末节点上（中间那个点不挂）。
    /// ① 判器结论与逐盘自证槽数每一块都存、观察者结论一块都不存；② 再给观察者挂着的块补上观察者结论，其中一块存一个违例状态；
    /// ③ 另一份库：判器结论每一块都存、逐盘自证槽数只缺头一块、不挂观察者。
    /// 预期：① 罩住的只有不挂观察者的那一块（没罩住的数 = 挂观察者的块数 2）；② 罩住 2 块、红 1 块（观察者结论记着红的那一块不罩住：
    /// 下一趟现判「记红、撤绿、下趟现判」），三块都有判定；③ 罩住 2 块（缺逐盘自证槽数的那一块不算）。
    /// 判别力（`crates/mutations.tsv`）：块账不问观察者结论、红块不数观察者结论里的违例、问判器结论不要逐盘自证槽数，各让这一条红；
    /// 先拿改环境的锁、后开库（反着的次序），开库取名额那一步当场 panic，这条红。
    #[test]
    fn the_block_account_needs_every_observer_conclusion_and_the_self_verified_slots_and_counts_observer_red(
    ) {
        with_the_unit_writes_flow_and_an_empty_library(
            "block-account-with-observers",
            |_flow, plan, store| {
                // 先开库、后拿改环境的锁（`crate::crash_amplification::EnvironmentLockOfThisTestBinary` 的次序）：③ 要的另一份空库也在拿锁之前开
                // 一次性库留着：块账的自证（造的结论与版本），要另一份空库
                let other_library =
                    crate::crash_library::DisposableLibrary::new("block-account-without-the-slots")
                        .unwrap_or_else(|refusal| panic!("{refusal}"));
                let _environment =
                    crate::crash_amplification::CrashAmplificationEnvironment::cleared_then_set(&[]);
                let judgement_version = ConclusionVersion([1; DIGEST_BYTES]);
                let judgement_version_of_each_point = vec![judgement_version; plan.points.len()];
                let judged_conclusions =
                    crate::crash_amplification::ConclusionsStoredUnder::TheConclusionKind {
                        kind: ConclusionKind::ImageJudgement,
                        version_of_each_point: &judgement_version_of_each_point,
                    };
                let observer_version = ConclusionVersion([2; DIGEST_BYTES]);
                assert_eq!(
                    plan.points.len(),
                    3,
                    "前提：单元写那一步、根槽写那一步、末节点"
                );
                let observers = [ObserverInTheBlockAccount {
                    holds_each_point: vec![true, false, true],
                    conclusion: observer_version,
                }];
                let blocks = blocks_of_the_plan(plan);
                assert_eq!(blocks.len(), 3, "前提：每个节点一块");
                let store_the_judgement_of_every_block =
                    |library_to_fill: &mut VerdictStore, blocks_without_the_slots: &[usize]| {
                        for (block_position, (_point_name, local, key)) in blocks.iter().enumerate()
                        {
                            store_a_conclusion_block_of_the_unit_test(
                                library_to_fill,
                                key,
                                &ConclusionKind::ImageJudgement,
                                &judgement_version,
                                local.end - local.start,
                                &[],
                                None,
                            );
                            if !blocks_without_the_slots.contains(&block_position) {
                                library_to_fill
                                    .store_observer_block(
                                        key,
                                        &ConclusionKind::ImageJudgement,
                                        &judgement_version,
                                        b"self verified slots of the unit test",
                                    )
                                    .expect("存得下");
                            }
                        }
                    };
                store_the_judgement_of_every_block(store, &[]);
                let every_block = |_block: u64| true;
                let without_observer_conclusions = blocks_in_the_library(
                    store,
                    plan,
                    &judged_conclusions,
                    &observers,
                    &every_block,
                )
                .expect("读得了库");
                assert_eq!(
                    (
                        without_observer_conclusions.blocks,
                        without_observer_conclusions.blocks - without_observer_conclusions.covered,
                        without_observer_conclusions.red
                    ),
                    (3, 2, 0),
                    "① 观察者结论全缺：没罩住的数 = 挂观察者的块数"
                );
                for (block_position, (_point_name, local, key)) in blocks.iter().enumerate() {
                    if observers[0].holds_each_point[block_position] {
                        let red_offsets: &[u64] = if block_position == 0 { &[0] } else { &[] };
                        store_a_conclusion_block_of_the_unit_test(
                            store,
                            key,
                            &ConclusionKind::Observer,
                            &observer_version,
                            local.end - local.start,
                            red_offsets,
                            None,
                        );
                    }
                }
                let with_observer_conclusions = blocks_in_the_library(
                    store,
                    plan,
                    &judged_conclusions,
                    &observers,
                    &every_block,
                )
                .expect("读得了库");
                assert_eq!(
                    (
                        with_observer_conclusions.covered,
                        with_observer_conclusions.red
                    ),
                    (2, 1),
                    "② 观察者结论补齐、一块带违例：记着红的那一块不罩住、红 1 块（观察者结论里的违例照数）"
                );
                let mut other_store = other_library
                    .open_store(crate::verdict_store::PruneServerFailureHandling::ReturnTheError)
                    .expect("开得了一次性库");
                store_the_judgement_of_every_block(&mut other_store, &[0]);
                let without_the_slots_of_one_block = blocks_in_the_library(
                    &other_store,
                    plan,
                    &judged_conclusions,
                    &[],
                    &every_block,
                )
                .expect("读得了库");
                assert_eq!(
                    without_the_slots_of_one_block.covered,
                    2,
                    "③ 判器结论在而逐盘自证槽数不在的那一块不算罩住（与判段「要不要判」同一个谓词）"
                );
            },
        );
    }

    /// 出口 10「每项态数现数」的办法 (a)：`CRASH_CASE_BLOCKS` 行带 `states=`，数的是这一份的块里的状态。三次单元写一段、一次根槽写一段的流
    /// （盘上原本没有内容，没有原地覆写，每次写两态），全量展开：每段 2^|段| − 1 个、末节点全部持久 1 个，(2³ − 1) + (2¹ − 1) + 1 = 9 个，
    /// 与崩溃点重放排的状态计划（`crash_state_ranges_by_segment`）数的一样；只领一部分块时只数领到的。
    /// 判别力（`crates/mutations.tsv`）：一块里的状态只记一个，这条红。
    #[test]
    fn the_blocks_line_reports_the_states_in_the_blocks_of_this_share() {
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
        let item = CrashPointReplayItem {
            name: "states_probe",
            setting_file: "crates/singlefs-verification/src/crash_state_enumeration.rs",
        };
        let judged_root_index = writes
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("造的流最后一段是一次根槽写");
        let (states_of_every_block, states_of_no_block) = flow_of_the_inputs(
            item,
            &base,
            &writes,
            &segments,
            judged_root_index,
            CrashPointReplayVersionTable::with_a_file_only(&[]),
            &every_proper_subset,
        )
        .with_the_flow(|flow| {
            let plan = plan_crash_points(flow);
            (
                states_of_the_blocks_of_this_share(&plan, &|_block| true),
                states_of_the_blocks_of_this_share(&plan, &|_block| false),
            )
        });
        assert_eq!(states_of_every_block, 9, "(2³ − 1) + (2¹ − 1) + 1");
        assert_eq!(
            states_of_every_block,
            crash_state_ranges_by_segment(&base, &writes, &segments, &every_proper_subset).1,
            "与状态计划数的一样"
        );
        assert_eq!(states_of_no_block, 0, "一块都没领就是 0 个");
        let line = CrashCaseBlocks {
            blocks: 0,
            reused: 0,
            fresh: 0,
            red: 0,
            refused: 0,
            states: states_of_every_block,
            expand_up_to: None,
        }
        .line();
        assert!(line.ends_with(" states=9"), "{line}");
    }
}
