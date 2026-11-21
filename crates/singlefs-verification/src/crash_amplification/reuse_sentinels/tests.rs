use super::*;

/// 复用哨兵抽一块当哨兵、一块强制现判（种子写死），交回抽的结果；前提：候选不少于两块。
fn reuse_sentinels_of_one_sentinel_and_one_forced_block(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) -> ReuseSentinelsOfTheRun {
    let sentinels = plan_the_reuse_sentinels(store, plan, conclusions, &|_| true, 1, 29, &[])
        .expect("读得了库")
        .expect("照键复用的那一趟抽哨兵");
    assert!(
        sentinels.candidate_count >= 2
            && sentinels.sentinels.len() == 1
            && sentinels.forced.is_some(),
        "前提：候选至少两块，抽到一块哨兵、一块强制现判：{sentinels:?}"
    );
    sentinels
}

/// 场景：每一块都有不红结论的库里抽一块哨兵、一块强制现判；抽完之后只把强制现判那一块照原样再存一遍（判段现判出同一个结论），收尾判哨兵。
/// 预期：抽的那一行带种子与块序号；抽完强制现判那一块问复用答不上、哨兵那一块答得上；收尾一处红都没有，强制现判表清掉（那一块又复用得上）；
/// 同一个种子抽到同一组。
/// 判别力（`crates/mutations.tsv`）：强制现判那一块问复用照旧答得上，这条红。
#[test]
fn reuse_sentinels_pass_when_only_the_forced_block_is_judged_again_with_the_same_conclusion() {
    let (_directory, mut store, plan, versions) =
        a_library_where_every_block_of_the_two_sector_flow_is_reusable("reuse-sentinels-green");
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let sentinels =
        reuse_sentinels_of_one_sentinel_and_one_forced_block(&mut store, &plan, &conclusions);
    let line = sentinels.line("unit_test_flow", 1);
    assert!(
            line.starts_with(&format!(
                "{REUSE_SENTINELS_LINE_PREFIX} flow=unit_test_flow seed=29 candidates={} sentinels={} forced=",
                sentinels.candidate_count, sentinels.sentinels[0].block_sequence
            )) && line.contains("reproduce=SINGLEFS_REUSE_SENTINEL_SEED=29"),
            "{line}"
        );
    let (forced, _stored) = sentinels.forced.clone().expect("抽到了强制现判那一块");
    let sentinel = sentinels.sentinels[0].clone();
    assert!(
        !conclusions
            .is_stored_for_reuse(&store, forced.point_index, &forced.key.0)
            .expect("读得了库"),
        "强制现判那一块这一趟问复用答不上"
    );
    assert!(
        conclusions
            .is_stored_for_reuse(&store, sentinel.point_index, &sentinel.key.0)
            .expect("读得了库"),
        "哨兵那一块照旧复用"
    );
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        forced.point_index,
        &forced.local,
        None,
        "unit_test_publish",
    );
    assert!(sentinels.forced_block_was_judged_here(&store));
    assert_eq!(
        sentinels
            .problems_after_judging(&mut store)
            .expect("读得了库"),
        Vec::<String>::new(),
        "只有强制现判那一块现判、结论相同：一处红都没有"
    );
    assert!(
        conclusions
            .is_stored_for_reuse(&store, forced.point_index, &forced.key.0)
            .expect("读得了库"),
        "收尾清掉强制现判表，那一块又复用得上"
    );
    let again =
        reuse_sentinels_of_one_sentinel_and_one_forced_block(&mut store, &plan, &conclusions);
    assert_eq!(
        (
            again.sentinels.clone(),
            again.forced.map(|(block, _stored_before_judging)| block)
        ),
        (vec![sentinel], Some(forced)),
        "同一个种子抽到同一组"
    );
}

/// 场景：同上抽一块哨兵、一块强制现判；强制现判那一块照原样再存，哨兵那一块也照原样再存一遍（键相同却现判了）。
/// 预期：收尾报「复用失灵」、点名哨兵那一块与种子。
/// 判别力（`crates/mutations.tsv`）：不核哨兵这一趟交来存过没有，这条红。
#[test]
fn a_reuse_sentinel_judged_again_turns_the_run_red() {
    let (_directory, mut store, plan, versions) =
        a_library_where_every_block_of_the_two_sector_flow_is_reusable("reuse-sentinel-judged");
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let sentinels =
        reuse_sentinels_of_one_sentinel_and_one_forced_block(&mut store, &plan, &conclusions);
    let (forced, _stored) = sentinels.forced.clone().expect("抽到了强制现判那一块");
    let sentinel = sentinels.sentinels[0].clone();
    for block in [&forced, &sentinel] {
        store_a_judge_conclusion_of_the_block(
            &mut store,
            &plan,
            &conclusions,
            block.point_index,
            &block.local,
            None,
            "unit_test_publish",
        );
    }
    let problems = sentinels
        .problems_after_judging(&mut store)
        .expect("读得了库");
    assert!(
        problems.len() == 1
            && problems[0].starts_with(&format!("复用失灵：哨兵块 {}", sentinel.block_sequence))
            && problems[0].contains("种子 29"),
        "{problems:?}"
    );
}

/// 场景：同上抽一块哨兵、一块强制现判；强制现判那一块现判出的是红的（与库里那一份不同，存块那一处照红换上）。
/// 预期：收尾报「判定不稳」、点名强制现判那一块与种子。
/// 判别力（`crates/mutations.tsv`）：强制现判那一块不比现判出的与库里那一份，这条红。
#[test]
fn a_forced_block_judged_to_a_different_conclusion_turns_the_run_red() {
    let (_directory, mut store, plan, versions) =
        a_library_where_every_block_of_the_two_sector_flow_is_reusable(
            "reuse-sentinel-forced-differs",
        );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let sentinels =
        reuse_sentinels_of_one_sentinel_and_one_forced_block(&mut store, &plan, &conclusions);
    let (forced, _stored) = sentinels.forced.clone().expect("抽到了强制现判那一块");
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        forced.point_index,
        &forced.local,
        Some("unit_test_red_item"),
        "unit_test_publish",
    );
    let problems = sentinels
        .problems_after_judging(&mut store)
        .expect("读得了库");
    assert!(
        problems.len() == 1
            && problems[0]
                .starts_with(&format!("判定不稳：强制现判的块 {}", forced.block_sequence))
            && problems[0].contains("种子 29"),
        "{problems:?}"
    );
}

/// 场景：同上抽一块哨兵、一块强制现判；这一趟一块都没交来存（强制现判那一块照旧复用了）。
/// 预期：收尾报「强制现判没起作用」、点名强制现判那一块与种子。
/// 判别力（`crates/mutations.tsv`）：不核强制现判那一块这一趟交来存过没有，这条红。
#[test]
fn a_forced_block_still_reused_turns_the_run_red() {
    let (_directory, mut store, plan, versions) =
        a_library_where_every_block_of_the_two_sector_flow_is_reusable(
            "reuse-sentinel-forced-reused",
        );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let sentinels =
        reuse_sentinels_of_one_sentinel_and_one_forced_block(&mut store, &plan, &conclusions);
    let (forced, _stored) = sentinels.forced.clone().expect("抽到了强制现判那一块");
    let problems = sentinels
        .problems_after_judging(&mut store)
        .expect("读得了库");
    assert!(
        problems.len() == 1
            && problems[0].starts_with(&format!(
                "复用哨兵：抽中强制现判的块 {}",
                forced.block_sequence
            ))
            && problems[0].contains("强制现判没起作用")
            && problems[0].contains("种子 29"),
        "{problems:?}"
    );
}

/// 复用哨兵带观察者那几条的观察者：挂计划里每个点，结论与录入块各一把写死的版本，录入段录的那一种（读观察者块表里观察者那一行）。
fn an_observer_on_every_point_for_the_reuse_sentinels(
    plan: &CrashPlan,
) -> ObserverForTheReuseSentinels {
    ObserverForTheReuseSentinels {
        holds_each_point: vec![true; plan.points.len()],
        conclusion: ConclusionVersion(sha256_digest(
            b"observer conclusion version of the reuse sentinel unit tests\n",
        )),
        input_row: (
            ConclusionKind::Observer,
            ConclusionVersion(sha256_digest(
                b"observer recording version of the reuse sentinel unit tests\n",
            )),
        ),
    }
}

/// 复用哨兵带观察者那几条的库：那条两个扇区的小流，每一块存一份不红的判器结论；`has_the_observer_conclusion` 答是的块另存一份不红的观察者结论，
/// `has_the_observer_input_row` 答是的块另存观察者判段重拼要读的那一行（两个谓词收块序号）。交回库、计划、判器的版本、观察者与块数。
fn a_library_of_the_two_sector_flow_with_an_observer(
    test_name: &str,
    has_the_observer_conclusion: &dyn Fn(u64) -> bool,
    has_the_observer_input_row: &dyn Fn(u64) -> bool,
) -> (
    crate::crash_library::DisposableLibrary,
    VerdictStore,
    CrashPlan,
    Vec<ConclusionVersion>,
    ObserverForTheReuseSentinels,
    u64,
) {
    let (library, mut store, plan, versions) =
        a_library_where_every_block_of_the_two_sector_flow_is_reusable(test_name);
    let observer = an_observer_on_every_point_for_the_reuse_sentinels(&plan);
    let mut block_sequence = 0u64;
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            if has_the_observer_conclusion(block_sequence) {
                store_an_observer_conclusion_of_the_block(
                    &mut store,
                    &plan,
                    &observer,
                    point_index,
                    &local,
                    None,
                );
            }
            if has_the_observer_input_row(block_sequence) {
                store_the_observer_input_row_of_the_block(
                    &mut store,
                    &plan,
                    &observer,
                    point_index,
                    &local,
                );
            }
            block_sequence += 1;
        }
    }
    (library, store, plan, versions, observer, block_sequence)
}

/// 带观察者的复用哨兵抽一块当哨兵、一块强制现判（种子写死），交回抽的结果；前提：候选不少于两块，两块上各有观察者那一份结论的键。
fn reuse_sentinels_with_the_observer_of_one_sentinel_and_one_forced_block(
    store: &mut VerdictStore,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
    observer: &ObserverForTheReuseSentinels,
) -> ReuseSentinelsOfTheRun {
    let sentinels = plan_the_reuse_sentinels(
        store,
        plan,
        conclusions,
        &|_| true,
        1,
        29,
        std::slice::from_ref(observer),
    )
    .expect("读得了库")
    .expect("照键复用的那一趟抽哨兵");
    assert!(
        sentinels.candidate_count >= 2
            && sentinels.sentinels.len() == 1
            && sentinels.sentinels[0].observer_keys.len() == 1
            && sentinels.forced.is_some()
            && sentinels.forced_observer_conclusions.len() == 1,
        "前提：候选至少两块，抽到一块哨兵、一块强制现判，两块上各一份观察者结论：{sentinels:?}"
    );
    sentinels
}

/// 场景：每一块都有不红的判器结论与观察者结论、观察者判段要读的那一行也在的库里，抽一块哨兵、一块强制现判；抽完之后只把强制现判那一块的
/// 判器结论与观察者结论照原样再存一遍（两段都现判出同一个结论），收尾判哨兵。
/// 预期：抽完强制现判那一块的观察者结论问复用答不上、哨兵那一块的答得上；收尾一处红都没有，强制现判表清掉（那一份观察者结论又复用得上）。
/// 判别力（`crates/mutations.tsv`）：观察者结论问复用不看强制现判表，这条红。
#[test]
fn reuse_sentinels_with_an_observer_pass_when_the_forced_block_is_judged_again_alike_by_the_judge_and_the_observer(
) {
    let (_directory, mut store, plan, versions, observer, _block_count) =
        a_library_of_the_two_sector_flow_with_an_observer(
            "reuse-sentinels-observer-green",
            &|_| true,
            &|_| true,
        );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let sentinels = reuse_sentinels_with_the_observer_of_one_sentinel_and_one_forced_block(
        &mut store,
        &plan,
        &conclusions,
        &observer,
    );
    let (forced, _stored) = sentinels.forced.clone().expect("抽到了强制现判那一块");
    let sentinel = sentinels.sentinels[0].clone();
    assert!(
        !observer_conclusion_is_reused(&store, &forced.key.0, &observer.conclusion)
            .expect("读得了库"),
        "强制现判那一块的观察者结论这一趟问复用答不上"
    );
    assert!(
        observer_conclusion_is_reused(&store, &sentinel.key.0, &observer.conclusion)
            .expect("读得了库"),
        "哨兵那一块的观察者结论照旧复用"
    );
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        forced.point_index,
        &forced.local,
        None,
        "unit_test_publish",
    );
    store_an_observer_conclusion_of_the_block(
        &mut store,
        &plan,
        &observer,
        forced.point_index,
        &forced.local,
        None,
    );
    assert_eq!(
        sentinels
            .problems_after_judging(&mut store)
            .expect("读得了库"),
        Vec::<String>::new(),
        "只有强制现判那一块的两份结论现判、结论都相同：一处红都没有"
    );
    assert!(
        observer_conclusion_is_reused(&store, &forced.key.0, &observer.conclusion)
            .expect("读得了库"),
        "收尾清掉强制现判表，那一份观察者结论又复用得上"
    );
}

/// 场景：同上抽一块哨兵、一块强制现判；强制现判那一块的两份结论照原样再存，哨兵那一块的观察者结论也照原样再存一遍（键相同却现判了），
/// 它的判器结论没动。
/// 预期：收尾只报一处「复用失灵」，点名哨兵那一块、它的块键、观察者那一份结论与种子。
/// 判别力（`crates/mutations.tsv`）：不核哨兵上观察者那一份结论这一趟交来存过没有，这条红。
#[test]
fn a_reuse_sentinel_whose_observer_conclusion_is_judged_again_turns_the_run_red() {
    let (_directory, mut store, plan, versions, observer, _block_count) =
        a_library_of_the_two_sector_flow_with_an_observer(
            "reuse-sentinel-observer-judged",
            &|_| true,
            &|_| true,
        );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let sentinels = reuse_sentinels_with_the_observer_of_one_sentinel_and_one_forced_block(
        &mut store,
        &plan,
        &conclusions,
        &observer,
    );
    let (forced, _stored) = sentinels.forced.clone().expect("抽到了强制现判那一块");
    let sentinel = sentinels.sentinels[0].clone();
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        forced.point_index,
        &forced.local,
        None,
        "unit_test_publish",
    );
    for block in [&forced, &sentinel] {
        store_an_observer_conclusion_of_the_block(
            &mut store,
            &plan,
            &observer,
            block.point_index,
            &block.local,
            None,
        );
    }
    let problems = sentinels
        .problems_after_judging(&mut store)
        .expect("读得了库");
    assert!(
        problems.len() == 1
            && problems[0].starts_with(&format!("复用失灵：哨兵块 {}", sentinel.block_sequence))
            && problems[0].contains("上观察者那一份结论")
            && problems[0].contains(&format!(
                "块键 {}",
                hexadecimal_text(&sentinel.key.0.to_block_key_bytes())
            ))
            && problems[0].contains("种子 29"),
        "{problems:?}"
    );
}

/// 场景：同上抽一块哨兵、一块强制现判；强制现判那一块的判器结论照原样再存，它的观察者结论现判出的是红的（与库里那一份不同，存块那一处照红换上）。
/// 预期：收尾只报一处「判定不稳」，点名强制现判那一块、它的块键、观察者那一份结论与种子。
/// 判别力（`crates/mutations.tsv`）：强制现判那一块上的观察者结论不比现判出的与库里那一份，这条红。
#[test]
fn a_forced_block_whose_observer_conclusion_is_judged_to_a_different_conclusion_turns_the_run_red()
{
    let (_directory, mut store, plan, versions, observer, _block_count) =
        a_library_of_the_two_sector_flow_with_an_observer(
            "reuse-sentinel-observer-forced-differs",
            &|_| true,
            &|_| true,
        );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let sentinels = reuse_sentinels_with_the_observer_of_one_sentinel_and_one_forced_block(
        &mut store,
        &plan,
        &conclusions,
        &observer,
    );
    let (forced, _stored) = sentinels.forced.clone().expect("抽到了强制现判那一块");
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        forced.point_index,
        &forced.local,
        None,
        "unit_test_publish",
    );
    store_an_observer_conclusion_of_the_block(
        &mut store,
        &plan,
        &observer,
        forced.point_index,
        &forced.local,
        Some("unit_test_observer_red_item"),
    );
    let problems = sentinels
        .problems_after_judging(&mut store)
        .expect("读得了库");
    assert!(
        problems.len() == 1
            && problems[0]
                .starts_with(&format!("判定不稳：强制现判的块 {}", forced.block_sequence))
            && problems[0].contains("上观察者那一份结论")
            && problems[0].contains(&format!(
                "块键 {}",
                hexadecimal_text(&forced.key.0.to_block_key_bytes())
            ))
            && problems[0].contains("种子 29"),
        "{problems:?}"
    );
}

/// 场景：同上抽一块哨兵、一块强制现判；强制现判那一块只有判器结论照原样再存，它的观察者结论这一趟没交来存（照旧复用了）。
/// 预期：收尾只报一处「强制现判没起作用」，点名强制现判那一块上观察者那一份结论与种子。
/// 判别力（`crates/mutations.tsv`）：不核强制现判那一块上的观察者结论这一趟交来存过没有，这条红。
#[test]
fn a_forced_block_whose_observer_conclusion_is_still_reused_turns_the_run_red() {
    let (_directory, mut store, plan, versions, observer, _block_count) =
        a_library_of_the_two_sector_flow_with_an_observer(
            "reuse-sentinel-observer-forced-reused",
            &|_| true,
            &|_| true,
        );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let sentinels = reuse_sentinels_with_the_observer_of_one_sentinel_and_one_forced_block(
        &mut store,
        &plan,
        &conclusions,
        &observer,
    );
    let (forced, _stored) = sentinels.forced.clone().expect("抽到了强制现判那一块");
    store_a_judge_conclusion_of_the_block(
        &mut store,
        &plan,
        &conclusions,
        forced.point_index,
        &forced.local,
        None,
        "unit_test_publish",
    );
    let problems = sentinels
        .problems_after_judging(&mut store)
        .expect("读得了库");
    assert!(
        problems.len() == 1
            && problems[0].starts_with(&format!(
                "复用哨兵：抽中强制现判的块 {}",
                forced.block_sequence
            ))
            && problems[0].contains("上观察者那一份结论")
            && problems[0].contains("强制现判没起作用")
            && problems[0].contains("种子 29"),
        "{problems:?}"
    );
}

/// 场景：每一块都有不红的判器结论，观察者结论只在块序号 0 以外的块上有；抽哨兵的块数给到块数（候选全抽成哨兵）。
/// 预期：候选比块数少一块、块序号 0 那一块不在哨兵里（挂着的观察者没有结论的块不当哨兵：判段要现判它的观察者结论）。
/// 判别力（`crates/mutations.tsv`）：候选不看观察者结论，这条红。
#[test]
fn reuse_sentinel_candidates_leave_out_a_block_whose_observer_conclusion_is_missing() {
    let (_directory, mut store, plan, versions, observer, block_count) =
        a_library_of_the_two_sector_flow_with_an_observer(
            "reuse-sentinel-observer-missing",
            &|block_sequence| block_sequence != 0,
            &|_| true,
        );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let count_drawing_every_candidate = usize::try_from(block_count).expect("块数装得进 usize");
    let sentinels = plan_the_reuse_sentinels(
        &mut store,
        &plan,
        &conclusions,
        &|_| true,
        count_drawing_every_candidate,
        29,
        std::slice::from_ref(&observer),
    )
    .expect("读得了库")
    .expect("照键复用的那一趟抽哨兵");
    assert_eq!(
        (
            u64::try_from(sentinels.candidate_count).expect("装得进 u64"),
            sentinels
                .sentinels
                .iter()
                .any(|sentinel| sentinel.block_sequence == 0),
            sentinels.forced.is_none(),
        ),
        (block_count - 1, false, true),
        "没有观察者结论的那一块不是候选，候选全当了哨兵、没有剩下的强制现判：{sentinels:?}"
    );
}

/// 场景：每一块都有不红的判器结论与观察者结论，观察者判段重拼要读的那一行只在最后一块上有；不抽哨兵（块数给 0），只抽强制现判，种子取 0 到 15。
/// 预期：强制现判照种子抽、不看录入段那一行在不在：有种子抽中的不是最后那一块（录入段那一行不在库里的块照样抽，抽完回录入段给它重录，
/// [`record_the_observers_of_the_forced_block_again`]「不丢掉再抽」），候选照旧是全部块。
/// 判别力（`crates/mutations.tsv`）：「复用哨兵：强制现判只抽录入段那一行在库里的块（丢掉再抽）」这一行让这一条红。
#[test]
fn the_forced_block_is_drawn_whether_or_not_its_observer_input_row_is_stored() {
    let (_plan, block_count_of_the_flow) = plan_of_the_two_sector_flow_with_its_block_count();
    let last_block_sequence = u64::try_from(block_count_of_the_flow)
        .expect("块数装得进 u64")
        .checked_sub(1)
        .expect("前提：这条流至少一块");
    let (_directory, mut store, plan, versions, observer, block_count) =
        a_library_of_the_two_sector_flow_with_an_observer(
            "reuse-sentinel-observer-input-row",
            &|_| true,
            &|block_sequence| block_sequence == last_block_sequence,
        );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    assert!(block_count >= 2, "前提：至少两块，强制现判才有得挑");
    let mut forced_blocks_drawn: BTreeSet<u64> = BTreeSet::new();
    for seed in 0..16u64 {
        let sentinels = plan_the_reuse_sentinels(
            &mut store,
            &plan,
            &conclusions,
            &|_| true,
            0,
            seed,
            std::slice::from_ref(&observer),
        )
        .expect("读得了库")
        .expect("照键复用的那一趟抽哨兵");
        assert_eq!(
            (
                u64::try_from(sentinels.candidate_count).expect("装得进 u64"),
                sentinels.sentinels.len(),
            ),
            (block_count, 0),
            "种子 {seed}：候选是全部块、不抽哨兵"
        );
        // 这一个种子抽完就撤掉它的强制现判：下一个种子照全部块当候选
        sentinels.stop_forcing_the_forced_block(&mut store);
        let (forced, _stored) = sentinels.forced.expect("候选不空就抽一块强制现判");
        forced_blocks_drawn.insert(forced.block_sequence);
    }
    assert!(
            forced_blocks_drawn
                .iter()
                .any(|block_sequence| *block_sequence != last_block_sequence),
            "种子 0 到 15 里有抽中录入段那一行不在库里的块的（16 个种子全抽中最后那一块的概率是 2 的负 16 次方，推的）：{forced_blocks_drawn:?}"
        );
}
