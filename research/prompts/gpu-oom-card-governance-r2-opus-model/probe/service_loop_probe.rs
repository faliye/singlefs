
// ── 攻方腿探针（gpu-oom-card-governance-r2 第 7 问），rerun.sh 把这一段接在运行目录那棵树的
//    gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs 末尾：用那一份里的替身判器与起服务的那一套。
//    断言写的是本腿推出的结局；断言红就是本腿推错了。

/// 撤回的块记下它在卡 0 上显存不够过，可它头一截被卡 1 拿去判之后（`dispatch_the_wave` 派一截就从「在等的」里删掉它），
/// 剩下那几截不再在「在等的」里，调度就照常把它们派回卡 0（`src/service_loop.rs` 文件头「只是不派回它显存不够过的卡」那一句在这一格上不成立）。
/// 一个场景：卡 0 大（一次派活 64 态）、卡 1 小（一次派活 4 态），一条流一块 256 态，卡 0 头一回判它时显存不够一回。
#[test]
fn probe_a_put_back_piece_split_onto_the_smaller_card_returns_to_the_card_it_ran_out_of_memory_on() {
    const STATES: u64 = 256;
    let stream = stream_tagged(1);
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(stream),
        kind: StandInFaultKind::OutOfMemory,
        times: 1,
    }]);
    let judged_runs = JudgedRuns::default();
    let (summary, waves, _notices) = serve_with_stand_in_judges(
        vec![
            card_leaving(0, 4 + mebibytes_of_states(64)),
            card_leaving(1, 4 + mebibytes_of_states(4)),
        ],
        &CardEvents::default(),
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let registered = register(client, 1, 2, STATES);
            every_block_judged(client, vec![(1, registered, 0..STATES)]);
        },
    );
    assert_eq!(summary.out_of_memory_caught, 1, "{summary:?}");
    let retracted: Vec<Range<u64>> = waves
        .iter()
        .flat_map(|wave| wave.cards.iter())
        .find(|card_plan| card_plan.card == CardIndex(0) && !card_plan.batches.is_empty())
        .map(|card_plan| {
            card_plan
                .batches
                .iter()
                .flat_map(|batch| batch.pieces.iter().map(|piece| piece.ordinals.clone()))
                .collect()
        })
        .expect("卡 0 有过一波");
    let runs = judged_runs.lock().expect("服务停了").clone();
    let overlaps = |ordinals: &Range<u64>| {
        retracted
            .iter()
            .any(|put| put.start < ordinals.end && ordinals.start < put.end)
    };
    let retracted_judged_on_card0 = runs
        .iter()
        .filter(|(card, _, ordinals)| *card == CardIndex(0) && overlaps(ordinals))
        .count();
    let retracted_judged_on_card1 = runs
        .iter()
        .filter(|(card, _, ordinals)| *card == CardIndex(1) && overlaps(ordinals))
        .count();
    let counts = judged_state_counts(&judged_runs);
    println!(
        "OOM_ATTACK_PROBE name=put_back_piece_back_on_its_card retracted={retracted:?} waves={} retracted_runs_on_card0={retracted_judged_on_card0} retracted_runs_on_card1={retracted_judged_on_card1} states_judged={} judged_once={}",
        waves.len(),
        counts.len(),
        counts.values().all(|count| *count == 1)
    );
    assert_eq!(counts.len(), usize::try_from(STATES).expect("装得进"), "每个态都判过");
    assert!(counts.values().all(|count| *count == 1), "每个态只判一次");
    assert!(
        retracted_judged_on_card1 > 0,
        "取样前提：撤回的那一截头几态由卡 1 拿去判（卡 1 没拿到的重跑，不算推翻）：{runs:?}"
    );
    assert!(
        retracted_judged_on_card0 > 0,
        "本腿推的：撤回的那一截被卡 1 拿走头几态之后，剩下的照常派回它显存不够过的卡 0；没派回就是本腿推错了：{runs:?}"
    );
}
