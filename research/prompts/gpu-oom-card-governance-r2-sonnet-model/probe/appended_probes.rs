
// ───────── 以下是辩方腿追加的两条探针（接在 service_loop_with_judges_off_the_card.rs 全文之后，用它里面的替身与辅助函数）─────────

/// 探针（判决「不成立」第二条：放回的核对组把另一张卡的判器全部换出）在今天的代码上再判一次：
/// 两张卡各驻留一条流的判器，核对组头一回派到卡 0、核的时候显存不够；放回排头之后主循环在收到卡 0 的「空出来了」之前就给它找卡，
/// 此刻空着的只有卡 1，放回的组要「先腾空再核」，就把卡 1 驻留的判器全换出。
/// 推的：核对组再核的那一张卡是卡 1（卡 1 的判器被换出）；判决引的读数是回到卡 0。要的结果：回到刚腾空的卡 0、不碰卡 1。
/// 取样：两条流头一波各装在一张卡上（机器忙、第二块晚到时这一条红，重跑，不算推翻）。
#[test]
fn probe_a_facts_job_put_back_after_running_out_of_memory_stays_off_the_other_cards_judges() {
    const TABLE_MEBIBYTES: u64 = 2;
    const STATES: u64 = 10;
    const FREE_BESIDE_THE_JUDGE: u64 = 3;
    let capacity = TABLE_MEBIBYTES + mebibytes_of_states(STATES) + FREE_BESIDE_THE_JUDGE;
    let card_events = CardEvents::default();
    let verified_facts_jobs = VerifiedFactsJobs::default();
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Verifying,
        kind: StandInFaultKind::OutOfMemory,
        times: 1,
    }]);
    let answered_on = Arc::new(Mutex::new(Vec::new()));
    let answered_on_for_the_client = Arc::clone(&answered_on);
    let (summary, _waves, _notices) = serve_with_stand_in_judges(
        vec![card_leaving(0, capacity), card_leaving(1, capacity)],
        &card_events,
        &faults,
        &JudgedRuns::default(),
        &verified_facts_jobs,
        move |client| {
            let one_stream = register(client, 1, TABLE_MEBIBYTES, STATES);
            let other_stream = register(client, 2, TABLE_MEBIBYTES, STATES);
            every_block_judged(
                client,
                vec![(1, one_stream, 0..STATES), (2, other_stream, 0..STATES)],
            );
            let material: Arc<dyn FactsVerifierMaterial> =
                Arc::new(StandInFactsVerifierMaterial::empty());
            let verified = client
                .submit_facts_verifier_job(FactsVerifierJob {
                    material,
                    blocks: vec![0..3],
                    verifier_mebibytes: FREE_BESIDE_THE_JUDGE - 1,
                })
                .recv()
                .expect("服务交回这一组")
                .unwrap_or_else(|refusal| panic!("放回之后那一组被拒：{refusal:?}"));
            answered_on_for_the_client
                .lock()
                .expect("没有线程拿着锁 panic")
                .push(verified.card);
        },
    );
    let mut cards_that_loaded: Vec<u32> = card_events
        .lock()
        .expect("服务停了")
        .iter()
        .filter(|(_, event)| matches!(event, CardEvent::Loaded(_)))
        .map(|(card, _)| card.0)
        .collect();
    cards_that_loaded.sort_unstable();
    let answered = answered_on.lock().expect("服务停了").clone();
    let verified = verified_facts_jobs.lock().expect("服务停了").clone();
    println!(
        "R2_DEFENSE_PROBE name=facts_job_put_back_lands cards_that_loaded={cards_that_loaded:?} answered_card={answered:?} \
         verified_card_and_judges_on_it={:?} out_of_memory_caught={} facts_jobs_put_back={}",
        verified
            .iter()
            .map(|(card, loaded, _)| (card.0, loaded.len()))
            .collect::<Vec<_>>(),
        summary.out_of_memory_caught,
        summary.facts_jobs_put_back_after_running_out_of_memory
    );
    assert_eq!(
        cards_that_loaded,
        vec![0, 1],
        "取样：两条流头一波各装在一张卡上（机器忙、第二块晚到时这一条红，重跑，不算推翻）"
    );
    assert_eq!(summary.facts_jobs_put_back_after_running_out_of_memory, 1);
    assert_eq!(
        answered,
        vec![CardIndex(0)],
        "放回的核对组应当回到刚腾空的卡 0，不该把卡 1 驻留的判器全换出再核：{verified:?}"
    );
}

/// 探针（判决第 6 条改法「撤回的块别的卡也能拿」的落点）：撤回的块照 `src/service_loop.rs` 文件头「接住 Out of Memory」写的「不派回它显存不够过的卡」，
/// 代码里这条限制挂在 `waiting_for_an_emptied_card` 上，那一块一旦被别的卡拿走头一截（`dispatch_the_wave` 把它从 `waiting_for_an_emptied_card` 里去掉），
/// 没派完的后半截就当普通待判块，`cards_the_block_ran_out_of_memory_on` 不再被问，可以再派回显存不够过的那张卡。
/// 一个场景：卡 0 一次派活装得下 64 个态、卡 1 只装得下 16 个，一块 64 个态；卡 0 判它时显存不够（设两回），放回的块卡 1 拿去判头 16 个，
/// 其余的在同一圈里又派回卡 0。
/// 推的：卡 0 上的毛病出两回（接住 Out of Memory 两回）；要的结果：一回（卡 1 空出来之前，没派完的那截不回卡 0）。
/// 取样：卡 0 头一回就显存不够（接住 Out of Memory 至少一回；机器忙、头一波只发给卡 1 时重跑）。
#[test]
fn probe_a_put_back_block_the_other_card_took_part_of_is_not_given_back_to_the_card_that_ran_out_of_memory(
) {
    const TABLE_MEBIBYTES: u64 = 2;
    const STATES: u64 = 64;
    const STATES_ON_THE_SMALL_CARD: u64 = 16;
    let stream = stream_tagged(1);
    let faults = faults_of(&[StandInFault {
        card: CardIndex(0),
        target: StandInFaultTarget::Judging(stream),
        kind: StandInFaultKind::OutOfMemory,
        times: 2,
    }]);
    let card_events = CardEvents::default();
    let judged_runs = JudgedRuns::default();
    let (summary, waves, _notices) = serve_with_stand_in_judges(
        vec![
            card_leaving(0, 2 * TABLE_MEBIBYTES + mebibytes_of_states(STATES)),
            card_leaving(
                1,
                2 * TABLE_MEBIBYTES + mebibytes_of_states(STATES_ON_THE_SMALL_CARD),
            ),
        ],
        &card_events,
        &faults,
        &judged_runs,
        &VerifiedFactsJobs::default(),
        |client| {
            let registered = register(client, 1, TABLE_MEBIBYTES, STATES);
            every_block_judged(client, vec![(1, registered, 0..STATES)]);
        },
    );
    let faults_on_the_card_that_ran_out = events_on(&card_events, CardIndex(0))
        .iter()
        .filter(|event| **event == CardEvent::FaultRaised)
        .count();
    let counts = judged_state_counts(&judged_runs);
    println!(
        "R2_DEFENSE_PROBE name=put_back_block_cut_by_the_other_card out_of_memory_caught={} \
         faults_raised_on_the_card_that_ran_out={faults_on_the_card_that_ran_out} waves={} \
         judged_states={} judged_more_than_once={}",
        summary.out_of_memory_caught,
        waves.len(),
        counts.len(),
        counts.values().filter(|count| **count > 1).count()
    );
    assert!(
        summary.out_of_memory_caught >= 1,
        "取样：卡 0 头一回就显存不够：{summary:?}"
    );
    assert_eq!(
        faults_on_the_card_that_ran_out, 1,
        "撤回的块被卡 1 拿走头一截之后，没派完的那截又派回了显存不够过的卡 0：{summary:?}"
    );
}
