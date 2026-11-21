//! 对拍结论按块的测试：按块复用与现判、一段段并行判、几台几段分组、按序号区间分的几趟。

use super::*;

/// 场景：按块对拍同一条流（9 块），头一趟第二块（序号 [4, 8)）判出两边不一致（红）、别的块一致；第二趟起每块都判出一致；连跑三趟。
/// 预期：第二趟只现判记着红的那一块（判段只叫一次、只交那一块），别的 8 块复用；第三趟 9 块都复用、现判 0。
/// 判别力（`crates/mutations.tsv`）：「对拍结论：记着红的照复用」「对拍结论：按块那一路问复用不看红绿」两行各让这一条红。
#[test]
fn a_pairing_block_stored_red_is_the_only_block_judged_again_the_next_time() {
    // 一次性库留着：要从空库起数现判的块
    let disposable = DisposableLibrary::new("pairing-block-stored-red")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
    let flow_inputs_digest = [9u8; DIGEST_BYTES];
    let red_block = 4..8;
    let first_run = pairing_blocks_run_with_the_agreement(
        &library,
        &flow_inputs_digest,
        4 * 9,
        2,
        &|ordinals| {
            if *ordinals == red_block {
                PairingAgreement::SidesDisagree
            } else {
                PairingAgreement::BothSidesAgree
            }
        },
    );
    assert_eq!(
        first_run.blocks_of_the_flow.blocks_judged_now, 9,
        "前提：头一趟 9 块都现判"
    );
    let second_run = pairing_blocks_run(&library, &flow_inputs_digest, 4 * 9, 2);
    assert_eq!(
        (
            second_run.blocks_of_the_flow.blocks_judged_now,
            second_run.blocks_of_the_flow.blocks_reused
        ),
        (1, 8),
        "记着红的那一块现判，别的 8 块复用"
    );
    assert_eq!(
        second_run.blocks_asked_of_the_judge_each_time,
        vec![1],
        "判段只叫一次、只交记着红的那一块"
    );
    let third_run = pairing_blocks_run(&library, &flow_inputs_digest, 4 * 9, 2);
    assert_eq!(
        (
            third_run.blocks_of_the_flow.blocks_judged_now,
            third_run.blocks_of_the_flow.blocks_reused
        ),
        (0, 9),
        "现判出一致的那一份换掉了红的：第三趟 9 块都复用"
    );
}

/// 按块对拍一条流的一趟：交回几块复用几块现判、判段被叫了几次、每次交来几块，与按序号次序收到的每块结论。
struct PairingBlocksRun {
    blocks_of_the_flow: PairingBlocksOfTheFlow,
    blocks_asked_of_the_judge_each_time: Vec<usize>,
    conclusions_in_order: Vec<(std::ops::Range<u64>, Vec<u8>)>,
}

/// 在 `library` 上按块对拍一条流：判段给每块的结论是「判<起点>-<终点>」、两边都一致，收到的结论按次序记下。
fn pairing_blocks_run(
    library: &CrashLibrary,
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    state_count: u64,
    most_blocks_judged_together: usize,
) -> PairingBlocksRun {
    pairing_blocks_run_with_the_agreement(
        library,
        flow_inputs_digest,
        state_count,
        most_blocks_judged_together,
        &|_ordinals| PairingAgreement::BothSidesAgree,
    )
}

/// 同 [`pairing_blocks_run`]，判段给每块的两边一致不一致照 `agreement_of_the_block`。
fn pairing_blocks_run_with_the_agreement(
    library: &CrashLibrary,
    flow_inputs_digest: &[u8; DIGEST_BYTES],
    state_count: u64,
    most_blocks_judged_together: usize,
    agreement_of_the_block: &(dyn Fn(&std::ops::Range<u64>) -> PairingAgreement + Sync),
) -> PairingBlocksRun {
    let blocks_asked_of_the_judge_each_time = Mutex::new(Vec::new());
    let mut conclusions_in_order = Vec::new();
    let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
        library,
        flow_inputs_digest,
        state_count,
        std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
        std::num::NonZeroUsize::new(most_blocks_judged_together).expect("一次判的块数不是 0"),
        std::num::NonZeroUsize::new(1).expect("一段一段判"),
        &PairingBlocksShare::whole_flow_in_one_share(),
        &|blocks| {
            blocks_asked_of_the_judge_each_time
                .lock()
                .expect("判段不 panic")
                .push(blocks.len());
            Some(
                blocks
                    .iter()
                    .map(|ordinals| PairingJudged {
                        conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                        agreement: agreement_of_the_block(ordinals),
                    })
                    .collect(),
            )
        },
        &mut |ordinals, conclusion| {
            conclusions_in_order.push((ordinals.clone(), conclusion.to_vec()));
        },
    )
    .expect("库读写得了")
    .expect("判段没报本次未跑");
    PairingBlocksRun {
        blocks_of_the_flow,
        blocks_asked_of_the_judge_each_time: blocks_asked_of_the_judge_each_time
            .into_inner()
            .expect("判段不 panic"),
        conclusions_in_order,
    }
}

/// 单测的判段给一块的结论字节：只由块的序号段定。
fn conclusion_bytes_of_the_pairing_block(ordinals: &std::ops::Range<u64>) -> Vec<u8> {
    format!("判{}-{}", ordinals.start, ordinals.end).into_bytes()
}

/// 宿主内存随块数不涨：同一个一次判几块，5 块的流与 50 块的流手里同一时刻压着的块结论一样多（都是一次判的块数），
/// 判段每次交来的块都不超过它；每块的结论按序号次序各收一次。
/// 判别力（`crates/mutations.tsv`）：攒着的块到了一次判的块数也不交出去判（整条流攒完才判），50 块那一条流压着 50 块，这条红。
#[test]
fn pairing_blocks_hold_as_many_conclusions_at_once_whatever_the_number_of_blocks_of_the_flow() {
    // 一次性库留着：要从空库起数现判的块
    let disposable =
        DisposableLibrary::new("pairing-blocks-held").unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
    for (flow_byte, blocks) in [(5u8, 5u64), (6u8, 50u64)] {
        let run = pairing_blocks_run(&library, &[flow_byte; DIGEST_BYTES], 4 * blocks - 1, 3);
        assert_eq!(
            run.blocks_of_the_flow.most_block_conclusions_held_at_once, 3,
            "{blocks} 块的流：手里同一时刻至多压着一次判的 3 块结论，与块数无关"
        );
        assert!(
            run.blocks_asked_of_the_judge_each_time
                .iter()
                .all(|asked| *asked <= 3),
            "{blocks} 块的流：判段每次交来的块不超过 3 块：{:?}",
            run.blocks_asked_of_the_judge_each_time
        );
        let expected_order: Vec<std::ops::Range<u64>> = (0..blocks)
            .map(|block| 4 * block..(4 * block + 4).min(4 * blocks - 1))
            .collect();
        assert_eq!(
            run.conclusions_in_order
                .iter()
                .map(|(ordinals, _)| ordinals.clone())
                .collect::<Vec<_>>(),
            expected_order,
            "{blocks} 块的流：每块的结论按序号次序各收一次，末块只到状态数"
        );
    }
}

/// 几段块同时判：头一段判完之前只有它一段；之后三段一起在各自的线程上判、后起的先判完，结论照样按序号次序折进总账、逐块都存进库，
/// 手里压着的块结论到 3 段 × 2 块为止。后起的两段先交回、队头那一段后交回，由队头那一段等到它们的块存进库再交回钉住，不看线程时序。
/// 判别力（`crates/mutations.tsv`）：头一段也不单独判，头一段判的时候别的段已经起了，这条红；一次只许一段（几段一起判的数不认），
/// 后面三段凑不齐、等到点，这条红；判完一段不排队、谁先判完谁先折进总账，这条红。
#[test]
fn stretches_judged_together_on_their_own_threads_come_back_in_block_order_and_all_stored() {
    const STRETCHES_AT_ONCE: usize = 3;
    const LONGEST_WAIT_FOR_THE_OTHER_STRETCHES: std::time::Duration =
        std::time::Duration::from_secs(60);
    // 一起判的三段里排在队头的那一段从哪个序号起：头一段之后的那一段
    const FIRST_STATE_OF_THE_QUEUE_HEAD_AMONG_THE_STRETCHES_TOGETHER: u64 = 4 * 2;
    // 一次性库留着：要从空库起数现判的块
    let disposable = DisposableLibrary::new("pairing-blocks-together")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
    let flow_inputs_digest = [41u8; DIGEST_BYTES];
    // 一段 2 块、一块 4 个状态，头一段加后面一起判的三段，一共 8 块
    let state_count = 4 * 2 * (1 + u64::try_from(STRETCHES_AT_ONCE).expect("装得进 u64"));
    // 在飞的几段各从哪个序号起；后面三段是不是都在飞过了
    let in_flight: Mutex<(Vec<u64>, bool)> = Mutex::new((Vec::new(), false));
    let in_flight_changed = std::sync::Condvar::new();
    let threads_of_the_stretches: Mutex<Vec<std::thread::ThreadId>> = Mutex::new(Vec::new());
    let is_stored_in_the_library = |ordinals: std::ops::Range<u64>| {
        library
            .lock()
            .read_pairing_conclusion(&pairing_conclusion_key_of_the_inputs(
                &pairing_block_inputs_digest(&flow_inputs_digest, &ordinals),
            ))
            .expect("读得了库")
            .is_some()
    };
    let mut conclusions_in_order = Vec::new();
    let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
        &library,
        &flow_inputs_digest,
        state_count,
        std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
        std::num::NonZeroUsize::new(2).expect("一次判 2 块"),
        std::num::NonZeroUsize::new(STRETCHES_AT_ONCE).expect("三段一起判"),
        &PairingBlocksShare::whole_flow_in_one_share(),
        &|blocks| {
            let start = blocks[0].start;
            threads_of_the_stretches
                .lock()
                .expect("判段不留毒")
                .push(std::thread::current().id());
            let mut guarded = in_flight.lock().expect("判段不留毒");
            guarded.0.push(start);
            if guarded.0.len() == STRETCHES_AT_ONCE {
                guarded.1 = true;
            }
            in_flight_changed.notify_all();
            if start == 0 {
                // 放开锁等一会儿：别的段要是起了，醒来时在飞的就不止它一段
                let (still_guarded, _timed_out) = in_flight_changed
                    .wait_timeout(guarded, std::time::Duration::from_millis(500))
                    .expect("判段不留毒");
                guarded = still_guarded;
                assert_eq!(guarded.0, vec![0], "头一段判完之前只有它一段");
            } else {
                // 三段都在飞过了才往下走；序号比它大的都走了它才走：后起的先判完
                let (still_guarded, wait_outcome) = in_flight_changed
                    .wait_timeout_while(
                        guarded,
                        LONGEST_WAIT_FOR_THE_OTHER_STRETCHES,
                        |(starts, all_were_in_flight)| {
                            !*all_were_in_flight || starts.iter().any(|other| *other > start)
                        },
                    )
                    .expect("判段不留毒");
                guarded = still_guarded;
                assert!(
                    !wait_outcome.timed_out(),
                    "序号从 {start} 起的这一段等不到别的段：三段没有一起判（在飞的 {:?}）",
                    guarded.0
                );
            }
            guarded.0.retain(|other| *other != start);
            in_flight_changed.notify_all();
            drop(guarded);
            if start == FIRST_STATE_OF_THE_QUEUE_HEAD_AMONG_THE_STRETCHES_TOGETHER {
                // 后起的两段离开判段之后、交回结论之前还隔着一步：等到它们的块都存进了库（收的那一头收到一段先存再记），
                // 这一段才交回，收的次序就是后起的先到，不看那一步的时序
                let deadline = std::time::Instant::now() + LONGEST_WAIT_FOR_THE_OTHER_STRETCHES;
                // 迭代上界是到点：每一轮问一次库，到点就判红
                while !(start + 4 * 2..state_count)
                    .step_by(4)
                    .all(|block_start| is_stored_in_the_library(block_start..block_start + 4))
                {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "序号从 {start} 起的这一段等不到后起的两段存进库"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }
            Some(
                blocks
                    .iter()
                    .map(|ordinals| PairingJudged {
                        conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                        agreement: PairingAgreement::BothSidesAgree,
                    })
                    .collect(),
            )
        },
        &mut |ordinals, conclusion| {
            conclusions_in_order.push((ordinals.clone(), conclusion.to_vec()));
        },
    )
    .expect("库读写得了")
    .expect("判段没报本次未跑");
    assert_eq!(
        conclusions_in_order,
        (0..8)
            .map(|block: u64| {
                let ordinals = 4 * block..4 * block + 4;
                let conclusion = conclusion_bytes_of_the_pairing_block(&ordinals);
                (ordinals, conclusion)
            })
            .collect::<Vec<_>>(),
        "每块的结论按序号次序各收一次"
    );
    assert_eq!(
        (
            blocks_of_the_flow.blocks_judged_now,
            blocks_of_the_flow.blocks_reused,
            blocks_of_the_flow.most_block_conclusions_held_at_once
        ),
        (8, 0, STRETCHES_AT_ONCE * 2),
        "8 块都现判；三段后起的先判完，头上那一段判完时手里压着 3 段 × 2 块"
    );
    let threads = threads_of_the_stretches.into_inner().expect("判段不留毒");
    assert_eq!(
        threads
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        threads.len(),
        "每段在自己的线程上判"
    );
    let second_run = pairing_blocks_run(&library, &flow_inputs_digest, state_count, 2);
    assert_eq!(
        (
            second_run.blocks_of_the_flow.blocks_judged_now,
            second_run.blocks_of_the_flow.blocks_reused
        ),
        (0, 8),
        "一起判的几段逐块都存进了库：第二趟一段一段走，8 块都复用"
    );
}

/// 后面一段报「本次未跑」：交 `Ok(None)`，别的段判完的照存进库（下一趟复用）、报本次未跑的那一段一块都不存。
/// 判别力（`crates/mutations.tsv`）：判那一侧交 `None` 当没事，整条流照往下走、交 `Some`，这条红。
#[test]
fn a_stretch_reported_not_run_stops_the_flow_and_keeps_what_the_others_judged() {
    // 一次性库留着：要从空库起数现判的块
    let disposable = DisposableLibrary::new("pairing-blocks-not-run")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
    let flow_inputs_digest = [42u8; DIGEST_BYTES];
    let run_once = |block_reported_not_run: Option<u64>| {
        pairing_conclusions_of_the_blocks_reused_or_judged(
            &library,
            &flow_inputs_digest,
            4 * 4,
            std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
            std::num::NonZeroUsize::new(1).expect("一次判 1 块"),
            std::num::NonZeroUsize::new(1).expect("一段一段判"),
            &PairingBlocksShare::whole_flow_in_one_share(),
            &|blocks| {
                if block_reported_not_run == Some(blocks[0].start) {
                    return None;
                }
                Some(
                    blocks
                        .iter()
                        .map(|ordinals| PairingJudged {
                            conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                            agreement: PairingAgreement::BothSidesAgree,
                        })
                        .collect(),
                )
            },
            &mut |_ordinals, _conclusion| {},
        )
        .expect("库读写得了")
    };
    assert_eq!(run_once(Some(8)), None, "第三块报本次未跑：整条流交 None");
    let second_run = run_once(None).expect("这一趟没有报本次未跑的");
    assert_eq!(
        (second_run.blocks_reused, second_run.blocks_judged_now),
        (2, 2),
        "报本次未跑之前判完的两块存进了库；它自己与它后面的现判"
    );
}

/// 判一段块的线程上 panic：照原样往外抛（判红的消息不变），不当本次未跑。
/// 判别力（`crates/mutations.tsv`）：panic 了只记成本次未跑、不往外抛，这条红（没 panic）。
#[test]
#[should_panic(expected = "两边对不上：这一段判红")]
fn a_panic_on_the_thread_of_a_stretch_is_thrown_again_with_its_message() {
    let disposable = DisposableLibrary::new("pairing-blocks-panic")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
    let _not_reached = pairing_conclusions_of_the_blocks_reused_or_judged(
        &library,
        &[43u8; DIGEST_BYTES],
        4,
        std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
        std::num::NonZeroUsize::new(1).expect("一次判 1 块"),
        std::num::NonZeroUsize::new(2).expect("两段一起判"),
        &PairingBlocksShare::whole_flow_in_one_share(),
        &|_blocks| panic!("两边对不上：这一段判红"),
        &mut |_ordinals, _conclusion| {},
    );
}

/// 两份领的块不重不漏：几条流、几种权重上，每一组块恰好归一份，两份领的块数与权重成比例（偏差不超过一组）；整份（`0/1`）领全部。
/// 判别力（`crates/mutations.tsv`）：归不归这一份的判断反过来，或一律算归这一份，这条红。
#[test]
fn the_shares_of_a_flow_claim_every_group_of_blocks_exactly_once() {
    const GROUPS: u64 = 1000;
    for flow_inputs_digest in [
        [0u8; DIGEST_BYTES],
        [7u8; DIGEST_BYTES],
        [200u8; DIGEST_BYTES],
    ] {
        for weights in [vec![1u64, 1], vec![2, 3], vec![1, 1, 1]] {
            let shares: Vec<PairingBlocksShare> = (0..weights.len())
                .map(|index| {
                    PairingBlocksShare::with_this_share_of_these_weights(
                        crate::crash_amplification::HostShare {
                            index,
                            count: weights.len(),
                        },
                        weights.clone(),
                    )
                })
                .collect();
            let total_weight: u64 = weights.iter().sum();
            let mut groups_of_each_share = vec![0u64; weights.len()];
            for group in 0..GROUPS {
                let claiming: Vec<usize> = shares
                    .iter()
                    .enumerate()
                    .filter(|(_, share)| {
                        share.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group)
                    })
                    .map(|(index, _)| index)
                    .collect();
                assert_eq!(
                    claiming.len(),
                    1,
                    "权重 {weights:?}：第 {group} 组恰好归一份，实际归 {claiming:?}"
                );
                groups_of_each_share[claiming[0]] += 1;
            }
            for (groups, weight) in groups_of_each_share.iter().zip(&weights) {
                assert!(
                    groups.abs_diff(GROUPS * weight / total_weight) <= total_weight,
                    "权重 {weights:?}：各份领的组数 {groups_of_each_share:?} 与权重成比例"
                );
            }
        }
        for group in 0..GROUPS {
            assert!(
                PairingBlocksShare::whole_flow_in_one_share()
                    .claims_this_group_of_pairing_blocks(&flow_inputs_digest, group),
                "整份领全部"
            );
        }
    }
}

/// 只走一段组的一趟：组区间外的组不领（两头外侧紧挨着的那一组在内），组区间里的每一组都领。
/// 判别力（`crates/mutations.tsv`）：组区间的判断一律算落在里面，这条红（区间外的组被领）。
#[test]
fn a_run_over_a_range_of_groups_claims_every_group_in_it_and_none_outside() {
    const GROUPS_TRIED: u64 = 20;
    let groups_of_the_run = 5..9;
    for flow_inputs_digest in [[0u8; DIGEST_BYTES], [7u8; DIGEST_BYTES]] {
        let share = PairingBlocksShare::whole_flow_in_one_share()
            .only_the_groups_of_pairing_blocks_in(groups_of_the_run.clone());
        let claimed: Vec<u64> = (0..GROUPS_TRIED)
            .filter(|group| share.claims_this_group_of_pairing_blocks(&flow_inputs_digest, *group))
            .collect();
        assert_eq!(
            claimed,
            groups_of_the_run.clone().collect::<Vec<u64>>(),
            "组区间 {groups_of_the_run:?}：领的恰好是区间里的每一组"
        );
        for group_just_outside in [groups_of_the_run.start - 1, groups_of_the_run.end] {
            assert!(
                    !share.groups_of_this_run_hold(group_just_outside),
                    "组区间 {groups_of_the_run:?}：紧挨着两头外侧的第 {group_just_outside} 组不落在里面"
                );
        }
    }
}

/// 组区间与几台分份叠用：一组要落在组区间里、又归这一份才领；两份各叠同一段组区间时，组区间里的每一组恰好一份领，区间外的哪一份都不领。
/// 判别力（`crates/mutations.tsv`）：两样只要一样领就算领，这条红。
#[test]
fn a_range_of_groups_stacked_on_the_shares_of_several_hosts_claims_a_group_only_when_both_do() {
    const GROUPS_TRIED: u64 = 200;
    let groups_of_the_run = 40..120;
    let flow_inputs_digest = [7u8; DIGEST_BYTES];
    let shares_of_the_hosts: Vec<PairingBlocksShare> = (0..2)
        .map(|index| {
            PairingBlocksShare::with_this_share_of_these_weights(
                crate::crash_amplification::HostShare { index, count: 2 },
                vec![1, 1],
            )
        })
        .collect();
    let stacked_shares: Vec<PairingBlocksShare> = shares_of_the_hosts
        .iter()
        .map(|share_of_the_host| {
            share_of_the_host
                .clone()
                .only_the_groups_of_pairing_blocks_in(groups_of_the_run.clone())
        })
        .collect();
    for (share_of_the_host, stacked) in shares_of_the_hosts.iter().zip(&stacked_shares) {
        let mut groups_of_the_host_outside_the_range = 0u64;
        let mut groups_in_the_range_left_to_the_other_host = 0u64;
        for group in 0..GROUPS_TRIED {
            let claimed_by_the_host =
                share_of_the_host.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group);
            let in_the_range = groups_of_the_run.contains(&group);
            assert_eq!(
                    stacked.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group),
                    claimed_by_the_host && in_the_range,
                    "第 {group} 组：归这一份 {claimed_by_the_host}、落在组区间 {groups_of_the_run:?} 里 {in_the_range}"
                );
            if claimed_by_the_host && !in_the_range {
                groups_of_the_host_outside_the_range += 1;
            }
            if !claimed_by_the_host && in_the_range {
                groups_in_the_range_left_to_the_other_host += 1;
            }
        }
        assert!(
                groups_of_the_host_outside_the_range > 0
                    && groups_in_the_range_left_to_the_other_host > 0,
                "取样的组里有归这一份而落在区间外的（{groups_of_the_host_outside_the_range} 组），也有落在区间里而归另一份的（{groups_in_the_range_left_to_the_other_host} 组）"
            );
    }
    for group in 0..GROUPS_TRIED {
        let claiming = stacked_shares
            .iter()
            .filter(|stacked| {
                stacked.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group)
            })
            .count();
        assert_eq!(
            claiming,
            usize::from(groups_of_the_run.contains(&group)),
            "第 {group} 组：组区间 {groups_of_the_run:?} 里的恰好一份领，区间外的哪一份都不领"
        );
    }
}

/// 一条流按状态序号区间拆开（区间两头落在组的边界上，最后一段到整条流的状态数为止、最后一组不满）：各段算出的组区间拼起来恰好盖满
/// 整条流的每一组、两两不交，各段只走自己那段组的几趟，每一组恰好一趟领、整条流之外的组哪一趟都不领；起点对不齐、终点对不齐又不是整条流的状态数、
/// 空的、越过整条流的区间算不出组区间。
/// 判别力（`crates/mutations.tsv`）：组区间的上界当闭区间判（相邻两段交界上那一组两趟都领），这条红；终点按整除取组、不往上取整（最后那一组不满的哪一趟都不领），这条红。
#[test]
fn ranges_of_states_that_tile_a_flow_give_ranges_of_groups_that_tile_its_groups_without_overlap() {
    const STATES_PER_BLOCK: u64 = 4;
    const BLOCKS_PER_GROUP: usize = 3;
    const STATES_PER_GROUP: u64 = 12;
    const STATE_COUNT: u64 = 13 * STATES_PER_GROUP + 2;
    let states_per_block = std::num::NonZeroU64::new(STATES_PER_BLOCK).expect("块宽 4 不是 0");
    let blocks_per_group = std::num::NonZeroUsize::new(BLOCKS_PER_GROUP).expect("一组 3 块不是 0");
    let ranges_of_states = [
        0..2 * STATES_PER_GROUP,
        2 * STATES_PER_GROUP..4 * STATES_PER_GROUP,
        4 * STATES_PER_GROUP..8 * STATES_PER_GROUP,
        8 * STATES_PER_GROUP..STATE_COUNT,
    ];
    let ranges_of_groups: Vec<std::ops::Range<u64>> = ranges_of_states
        .iter()
        .map(|ordinals| {
            groups_of_pairing_blocks_holding_the_states(
                ordinals,
                STATE_COUNT,
                states_per_block,
                blocks_per_group,
            )
            .unwrap_or_else(|refusal| panic!("{refusal}"))
        })
        .collect();
    let groups_of_the_flow = STATE_COUNT.div_ceil(STATES_PER_GROUP);
    assert_eq!(
        ranges_of_groups,
        vec![0..2, 2..4, 4..8, 8..groups_of_the_flow],
        "各段的组区间：最后一段罩到不满的最后一组"
    );
    let flow_inputs_digest = [9u8; DIGEST_BYTES];
    let runs_over_the_ranges: Vec<PairingBlocksShare> = ranges_of_groups
        .iter()
        .map(|groups| {
            PairingBlocksShare::whole_flow_in_one_share()
                .only_the_groups_of_pairing_blocks_in(groups.clone())
        })
        .collect();
    for group in 0..groups_of_the_flow + 2 {
        let claiming = runs_over_the_ranges
            .iter()
            .filter(|run| run.claims_this_group_of_pairing_blocks(&flow_inputs_digest, group))
            .count();
        assert_eq!(
            claiming,
            usize::from(group < groups_of_the_flow),
            "第 {group} 组：整条流的 {groups_of_the_flow} 组里的恰好一趟领，之外的哪一趟都不领"
        );
    }
    for refused in [
        STATES_PER_BLOCK..2 * STATES_PER_GROUP,
        0..2 * STATES_PER_GROUP + STATES_PER_BLOCK,
        STATES_PER_GROUP..STATES_PER_GROUP,
        8 * STATES_PER_GROUP..STATE_COUNT + 1,
    ] {
        assert!(
            groups_of_pairing_blocks_holding_the_states(
                &refused,
                STATE_COUNT,
                states_per_block,
                blocks_per_group
            )
            .is_err(),
            "状态序号区间 {refused:?} 算不出组区间"
        );
    }
}

/// 一条流按组区间拆成几趟：每一趟只判、只折进总账自己组区间里的块，别的数进 `blocks_outside_the_groups_of_this_run`（不算归别的份）；
/// 几趟判的块拼起来不重不漏；之后整条流一趟（`0/1`）一块都不现判、每块复用：块键不带组区间。
/// 判别力（`crates/mutations.tsv`）：组区间外的块也判，这条红。
#[test]
fn runs_over_ranges_of_groups_judge_every_block_once_and_a_run_over_the_whole_flow_reuses_all_of_them(
) {
    const BLOCKS: u64 = 40;
    const BLOCKS_OF_A_GROUP: usize = 3;
    let disposable = DisposableLibrary::new("pairing-blocks-ranges-of-groups")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
    let flow_inputs_digest = [45u8; DIGEST_BYTES];
    let state_count = 4 * BLOCKS;
    let run_the_share = |share: &PairingBlocksShare| {
        let blocks_asked_of_the_judge: Mutex<Vec<std::ops::Range<u64>>> = Mutex::new(Vec::new());
        let mut taken: Vec<std::ops::Range<u64>> = Vec::new();
        let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
            &library,
            &flow_inputs_digest,
            state_count,
            std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
            std::num::NonZeroUsize::new(BLOCKS_OF_A_GROUP).expect("一次判的块数不是 0"),
            std::num::NonZeroUsize::new(2).expect("两段一起判"),
            share,
            &|blocks| {
                blocks_asked_of_the_judge
                    .lock()
                    .expect("判段不 panic")
                    .extend_from_slice(blocks);
                Some(
                    blocks
                        .iter()
                        .map(|ordinals| PairingJudged {
                            conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                            agreement: PairingAgreement::BothSidesAgree,
                        })
                        .collect(),
                )
            },
            &mut |ordinals, _conclusion| taken.push(ordinals.clone()),
        )
        .expect("库读写得了")
        .expect("判段没报本次未跑");
        let mut judged = blocks_asked_of_the_judge
            .into_inner()
            .expect("判段不 panic");
        judged.sort_by_key(|ordinals| ordinals.start);
        (blocks_of_the_flow, judged, taken)
    };
    let blocks_of_a_group = u64::try_from(BLOCKS_OF_A_GROUP).expect("装得进 u64");
    let blocks_of_the_groups = |groups: &std::ops::Range<u64>| -> Vec<std::ops::Range<u64>> {
        (groups.start * blocks_of_a_group..(groups.end * blocks_of_a_group).min(BLOCKS))
            .map(|block| 4 * block..4 * block + 4)
            .collect()
    };
    let mut judged_by_the_runs: Vec<std::ops::Range<u64>> = Vec::new();
    for groups in [0..5, 5..BLOCKS.div_ceil(blocks_of_a_group)] {
        let (blocks_of_the_flow, judged, taken) = run_the_share(
            &PairingBlocksShare::whole_flow_in_one_share()
                .only_the_groups_of_pairing_blocks_in(groups.clone()),
        );
        let blocks_of_the_run = blocks_of_the_groups(&groups);
        let block_count_of_the_run = u64::try_from(blocks_of_the_run.len()).expect("装得进 u64");
        assert_eq!(
            judged, blocks_of_the_run,
            "组区间 {groups:?} 的一趟判的恰好是那几组的块"
        );
        assert_eq!(
            taken, blocks_of_the_run,
            "组区间 {groups:?} 的一趟按序号次序只把那几组的块折进总账"
        );
        assert_eq!(
            (
                blocks_of_the_flow.blocks_judged_now,
                blocks_of_the_flow.blocks_outside_the_groups_of_this_run,
                blocks_of_the_flow.blocks_left_to_other_shares
            ),
            (block_count_of_the_run, BLOCKS - block_count_of_the_run, 0),
            "组区间 {groups:?}：区间外的块数进区间外那一项，不算归别的份"
        );
        judged_by_the_runs.extend(judged);
    }
    judged_by_the_runs.sort_by_key(|ordinals| ordinals.start);
    assert_eq!(
        judged_by_the_runs,
        (0..BLOCKS)
            .map(|block| 4 * block..4 * block + 4)
            .collect::<Vec<_>>(),
        "几趟判的块拼起来不重不漏"
    );
    let (whole_run, judged_by_the_whole_run, _taken) =
        run_the_share(&PairingBlocksShare::whole_flow_in_one_share());
    assert_eq!(
        (
            whole_run.blocks_reused,
            whole_run.blocks_judged_now,
            whole_run.blocks_outside_the_groups_of_this_run
        ),
        (BLOCKS, 0, 0),
        "整条流那一趟每块复用：块键不带组区间"
    );
    assert!(
        judged_by_the_whole_run.is_empty(),
        "整条流那一趟判段一次都不叫"
    );
}

/// 两份各判自己的块、块键不带份额：两份都跑完之后，合并那一趟（`0/1`）一块都不现判，结论与从头整份判的逐块相同；
/// 每一份只把自己领的块折进总账、别的数进 `blocks_left_to_other_shares`；两份交给判段的每一段都是连着的整组。
/// 判别力（`crates/mutations.tsv`）：不归这一份的块也判，两份现判的块相加超过整条流，这条红；块一块一块轮着分、不按组分，判段拿不到整组连着的一段，这条红。
#[test]
fn two_shares_judge_every_block_once_and_the_merging_run_reuses_all_of_them() {
    const BLOCKS: u64 = 40;
    const BLOCKS_OF_A_GROUP: usize = 3;
    let disposable = DisposableLibrary::new("pairing-blocks-shares")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
    let flow_inputs_digest = [44u8; DIGEST_BYTES];
    let state_count = 4 * BLOCKS;
    let run_the_share = |share: &PairingBlocksShare| {
        let stretches_asked: Mutex<Vec<Vec<std::ops::Range<u64>>>> = Mutex::new(Vec::new());
        let mut taken: Vec<std::ops::Range<u64>> = Vec::new();
        let blocks_of_the_flow = pairing_conclusions_of_the_blocks_reused_or_judged(
            &library,
            &flow_inputs_digest,
            state_count,
            std::num::NonZeroU64::new(4).expect("块宽 4 不是 0"),
            std::num::NonZeroUsize::new(BLOCKS_OF_A_GROUP).expect("一次判的块数不是 0"),
            std::num::NonZeroUsize::new(2).expect("两段一起判"),
            share,
            &|blocks| {
                stretches_asked
                    .lock()
                    .expect("判段不 panic")
                    .push(blocks.to_vec());
                Some(
                    blocks
                        .iter()
                        .map(|ordinals| PairingJudged {
                            conclusion: conclusion_bytes_of_the_pairing_block(ordinals),
                            agreement: PairingAgreement::BothSidesAgree,
                        })
                        .collect(),
                )
            },
            &mut |ordinals, conclusion| {
                assert_eq!(
                    conclusion,
                    conclusion_bytes_of_the_pairing_block(ordinals),
                    "折进总账的是这一块的结论"
                );
                taken.push(ordinals.clone());
            },
        )
        .expect("库读写得了")
        .expect("判段没报本次未跑");
        (
            blocks_of_the_flow,
            stretches_asked.into_inner().expect("判段不 panic"),
            taken,
        )
    };
    let mut judged_by_the_shares: Vec<std::ops::Range<u64>> = Vec::new();
    for index in 0..2 {
        let (blocks_of_the_flow, stretches_asked, taken) =
            run_the_share(&PairingBlocksShare::with_this_share_of_these_weights(
                crate::crash_amplification::HostShare { index, count: 2 },
                vec![1, 1],
            ));
        assert_eq!(
            blocks_of_the_flow.blocks_judged_now + blocks_of_the_flow.blocks_left_to_other_shares,
            BLOCKS,
            "第 {index} 份：判的加留给别的份的是整条流"
        );
        assert!(
            blocks_of_the_flow.blocks_left_to_other_shares > 0,
            "第 {index} 份：两份等权，有留给别的份的"
        );
        assert_eq!(
            u64::try_from(taken.len()).expect("装得进 u64"),
            blocks_of_the_flow.blocks_judged_now,
            "第 {index} 份只把自己领的块折进总账"
        );
        for stretch in &stretches_asked {
            assert!(
                stretch.windows(2).all(|pair| pair[0].end == pair[1].start)
                    && stretch.len() <= BLOCKS_OF_A_GROUP,
                "第 {index} 份交给判段的一段是连着的块：{stretch:?}"
            );
        }
        assert!(
            stretches_asked
                .iter()
                .any(|stretch| stretch.len() == BLOCKS_OF_A_GROUP),
            "第 {index} 份领的是整组的块，判段拿到过整组连着的一段：{stretches_asked:?}"
        );
        judged_by_the_shares.extend(stretches_asked.into_iter().flatten());
    }
    judged_by_the_shares.sort_by_key(|ordinals| ordinals.start);
    assert_eq!(
        judged_by_the_shares,
        (0..BLOCKS)
            .map(|block| 4 * block..4 * block + 4)
            .collect::<Vec<_>>(),
        "两份判的块不重不漏"
    );
    let (merging_run, stretches_asked_of_the_merging_run, taken_by_the_merging_run) =
        run_the_share(&PairingBlocksShare::whole_flow_in_one_share());
    assert_eq!(
        (
            merging_run.blocks_reused,
            merging_run.blocks_judged_now,
            merging_run.blocks_left_to_other_shares
        ),
        (BLOCKS, 0, 0),
        "合并那一趟每块复用：块键不带份额"
    );
    assert!(
        stretches_asked_of_the_merging_run.is_empty(),
        "合并那一趟判段一次都不叫"
    );
    assert_eq!(
        taken_by_the_merging_run,
        (0..BLOCKS)
            .map(|block| 4 * block..4 * block + 4)
            .collect::<Vec<_>>(),
        "合并那一趟每块的结论按序号次序各收一次"
    );
}

/// 第二趟同一条流复用、现判 0：头一趟每块现判、一块都不复用（块键带序号段，别的块的结论不顶替这一块）；同一份输入的
/// 第二趟每块复用、判段一次都不叫，收到的结论与头一趟逐块相同。
/// 判别力（`crates/mutations.tsv`）：读库那一处一律当库里没有，第二趟照判、这条红；块键不带序号段，头一趟后面的块复用了前面块的结论、
/// 这条红。
#[test]
fn a_flow_paired_twice_by_blocks_reuses_every_block_and_judges_none_the_second_time() {
    // 一次性库留着：要从空库起数现判的块
    let disposable = DisposableLibrary::new("pairing-blocks-reuse")
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let _the_library_of_this_process = disposable.make_it_the_library_of_this_process();
    let library = CrashLibrary::open(CrashLibraryClass::JudgeTablesPairing).expect("开得了库");
    let flow_inputs_digest = [7u8; DIGEST_BYTES];
    let first_run = pairing_blocks_run(&library, &flow_inputs_digest, 4 * 9, 2);
    assert_eq!(
        (
            first_run.blocks_of_the_flow.blocks_judged_now,
            first_run.blocks_of_the_flow.blocks_reused
        ),
        (9, 0),
        "头一趟 9 块都现判、一块都不复用"
    );
    let second_run = pairing_blocks_run(&library, &flow_inputs_digest, 4 * 9, 2);
    assert_eq!(
        (
            second_run.blocks_of_the_flow.blocks_judged_now,
            second_run.blocks_of_the_flow.blocks_reused
        ),
        (0, 9),
        "第二趟 9 块都复用、现判 0"
    );
    assert!(
        second_run.blocks_asked_of_the_judge_each_time.is_empty(),
        "第二趟判段一次都不叫：{:?}",
        second_run.blocks_asked_of_the_judge_each_time
    );
    assert_eq!(
        second_run.conclusions_in_order, first_run.conclusions_in_order,
        "复用的结论与头一趟现判的逐块相同"
    );
}
