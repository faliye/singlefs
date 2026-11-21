//! 批次调度要满足的八条（规格「批次调度要满足的」）与新装那一刻给表的暂存留地方那一条（`src/batch_scheduling.rs` 文件头「新装那一刻」），
//! 一条一个测试，都不上卡。新装的判器表在装的那一刻占两份，样本里卡的额度照两份表算。
//! 核对器借的显存（`src/card_memory_ledger.rs` 文件头「核对器」）同一个道理：六张事实表照两份算，另一条测试核。
//! 每条的弄坏开关（`GPU_VERIFICATION_SERVER_BREAK=<项>`，表在 `src/break_switches.rs` 的 `BreakSwitch::red_test`）打开时那一条必须红。

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use common::{
    apply_the_wave, card_leaving, configuration_with_the_limit, judge_the_wave,
    mebibytes_of_states, milliseconds, pool_with, stream_tagged, streams_of, undispatched,
};
use gpu_verification_server::batch_scheduling::{
    plan_the_next_wave, CardPlanOfTheWave, JudgeToLoad, PendingBlock, ResidentJudge,
    SchedulingInputs, WaveDecision, WavePlan,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::{
    bytes_of_one_facts_verifier_pass, bytes_of_the_facts_tables_on_the_card,
    mebibytes_of_a_facts_verifier_under, mebibytes_of_a_judge, mebibytes_of_one_state_rounded_up,
    mebibytes_while_loading_a_judge, states_fitting_in, CardIndex, CardMemoryLedger,
    CardOfTheService, BYTES_PER_MEBIBYTE,
};
use gpu_verification_server::pending_pool::{InMemoryPendingPool, PendingPool};
use gpu_verification_server::service_configuration::ServiceConfiguration;
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use singlefs_verification::crash_facts::FlowFacts;
use singlefs_verification::gpu_memory_ledger::PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD;

/// 新装判器那一条用例的一个场景：名字、卡上留着的判器、要入池的块（流、序号区间、入池时刻）。
type LoadingScenario = (
    &'static str,
    Vec<ResidentJudge>,
    Vec<(StreamKey, std::ops::Range<u64>, Duration)>,
);

/// 排一波：限时 `limit_milliseconds`，此刻 `now`。
fn decide(
    limit_milliseconds: u64,
    idle_cards: &[CardOfTheService],
    residents: &[ResidentJudge],
    streams: &BTreeMap<StreamKey, StreamFacts>,
    pending: &[PendingBlock],
    now: Duration,
) -> WaveDecision {
    plan_the_next_wave(
        &configuration_with_the_limit(limit_milliseconds),
        &SchedulingInputs {
            idle_cards,
            residents,
            streams,
            pending,
            now,
        },
    )
}

fn dispatched(decision: WaveDecision) -> WavePlan {
    match decision {
        WaveDecision::Dispatch(plan) => plan,
        other @ (WaveDecision::HoldTheEmptyPool
        | WaveDecision::HoldBelowTheWatermark { .. }
        | WaveDecision::HoldNothingPlaceableOnTheIdleCards) => {
            panic!("这一刻该发一波，调度却说 {other:?}")
        }
    }
}

fn card_plan(plan: &WavePlan, card: u32) -> &CardPlanOfTheWave {
    plan.cards
        .iter()
        .find(|card_plan| card_plan.card == CardIndex(card))
        .unwrap_or_else(|| panic!("计划里没有卡 {card}"))
}

fn batch_streams(card_plan: &CardPlanOfTheWave) -> Vec<StreamKey> {
    card_plan.batches.iter().map(|batch| batch.stream).collect()
}

/// 定死种子的线性同余数列：随机输入每趟相同。
struct ScenarioNumbers(u64);

impl ScenarioNumbers {
    fn below(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % bound
    }

    fn between(&mut self, low: u64, high: u64) -> u64 {
        low + self.below(high - low + 1)
    }
}

/// 一份随机输入：几张卡、几条流、每条流切成几块（有的这一刻入池，有的晚几波入池）。
struct RandomScenario {
    cards: Vec<CardOfTheService>,
    streams: BTreeMap<StreamKey, StreamFacts>,
    early_blocks: Vec<(StreamKey, std::ops::Range<u64>)>,
    late_blocks: Vec<(StreamKey, std::ops::Range<u64>)>,
}

fn random_scenario(numbers: &mut ScenarioNumbers) -> RandomScenario {
    let card_count = numbers.between(1, 3);
    let cards: Vec<CardOfTheService> = (0..card_count)
        .map(|index| {
            card_leaving(
                u32::try_from(index).expect("几张卡"),
                numbers.between(64, 2048),
            )
        })
        .collect();
    let largest = cards
        .iter()
        .map(CardOfTheService::mebibytes_for_the_judges)
        .max()
        .expect("至少一张卡");
    let mut streams = BTreeMap::new();
    let mut early_blocks = Vec::new();
    let mut late_blocks = Vec::new();
    for tag in 0..numbers.between(1, 5) {
        let stream = stream_tagged(u8::try_from(tag + 1).expect("几条流"));
        // 新装那一刻表占两份：表至多是最大那张卡给判器的减一个态之后的一半，每条流都有卡装得下
        let table_mebibytes = numbers.between(
            1,
            ((largest - mebibytes_of_one_state_rounded_up()) / 2).min(400),
        );
        let state_count = numbers.between(1, 300);
        streams.insert(
            stream,
            StreamFacts {
                table_mebibytes,
                state_count,
            },
        );
        let mut start = 0;
        while start < state_count {
            let end = (start + numbers.between(1, 120)).min(state_count);
            if numbers.below(3) == 0 {
                late_blocks.push((stream, start..end));
            } else {
                early_blocks.push((stream, start..end));
            }
            start = end;
        }
    }
    RandomScenario {
        cards,
        streams,
        early_blocks,
        late_blocks,
    }
}

/// 第 1 条：一张卡上同时驻留的各判器合计不超过这张卡给服务的额度减驱动那一份，任何一波都满足；越界的判器进不了显存账。
/// 弄坏开关：memory-ledger-unchecked（账收下越界的判器）、scratch-split-ignores-kept-judges（新装的判器分草稿区时不扣留着的判器）。
#[test]
fn judges_resident_on_a_card_never_exceed_the_quota_less_the_driver_share() {
    let card = card_leaving(0, 100);
    let mut ledger = CardMemoryLedger::empty(&card, &BreakSwitches::of_the_environment());
    let facts = StreamFacts {
        table_mebibytes: 50,
        state_count: 10,
    };
    assert!(
        mebibytes_of_a_judge(&facts, 10) > 100,
        "样本要越界：表 50 MiB 加 10 个态的草稿区"
    );
    assert!(
        ledger.admit(stream_tagged(1), &facts, 10).is_err(),
        "越过额度的判器账上要拒"
    );
    assert_eq!(ledger.free_mebibytes(), 100, "拒了之后账不动");

    let mut numbers = ScenarioNumbers(0x5eed_0001);
    let mut waves_checked = 0u64;
    for _scenario in 0..200 {
        let scenario = random_scenario(&mut numbers);
        let mut pool = InMemoryPendingPool::empty(BreakSwitches::of_the_environment());
        for (stream, ordinals) in &scenario.early_blocks {
            pool.add_block(*stream, ordinals.clone(), Duration::ZERO)
                .expect("非空块");
        }
        let mut residents: Vec<ResidentJudge> = Vec::new();
        for wave in 0..40u64 {
            if wave == 2 {
                for (stream, ordinals) in &scenario.late_blocks {
                    pool.add_block(*stream, ordinals.clone(), milliseconds(wave))
                        .expect("非空块");
                }
            }
            let pending = undispatched(&pool);
            let decision = decide(
                0,
                &scenario.cards,
                &residents,
                &scenario.streams,
                &pending,
                milliseconds(wave),
            );
            let WaveDecision::Dispatch(plan) = decision else {
                if wave >= 2 {
                    break;
                }
                continue;
            };
            for card_plan in &plan.cards {
                let planned_card = scenario
                    .cards
                    .iter()
                    .find(|input_card| input_card.card == card_plan.card)
                    .expect("计划里的卡是输入里的卡");
                let evicted: BTreeSet<StreamKey> = card_plan.evictions.iter().copied().collect();
                let staying: u64 = residents
                    .iter()
                    .filter(|resident| {
                        resident.card == planned_card.card && !evicted.contains(&resident.stream)
                    })
                    .map(|resident| {
                        mebibytes_of_a_judge(
                            &scenario.streams[&resident.stream],
                            resident.states_per_dispatch,
                        )
                    })
                    .sum();
                let loaded: u64 = card_plan
                    .loads
                    .iter()
                    .map(|load| {
                        mebibytes_of_a_judge(
                            &scenario.streams[&load.stream],
                            load.states_per_dispatch,
                        )
                    })
                    .sum();
                assert!(
                    staying + loaded <= planned_card.mebibytes_for_the_judges(),
                    "卡 {:?} 这一波之后驻留 {} MiB，额度减驱动那一份只有 {} MiB",
                    planned_card.card,
                    staying + loaded,
                    planned_card.mebibytes_for_the_judges()
                );
                assert_eq!(
                    card_plan.resident_after_the_wave().occupied_mebibytes(),
                    staying + loaded,
                    "计划里记的驻留量与按换出、装上算的相同"
                );
                waves_checked += 1;
            }
            residents = apply_the_wave(&mut pool, &plan, &residents);
            judge_the_wave(&mut pool, &plan);
        }
    }
    assert!(
        waves_checked > 200,
        "随机输入只核了 {waves_checked} 份卡的计划"
    );
}

/// 第 2 条：一波里每个待判的态恰好派一次；没派到的留在池里，下一波接着取，取完为止每个态只派过一次。
/// 弄坏开关：carved-states-not-consumed（切过的态下一批又切）、pool-drops-the-rest-of-a-partly-dispatched-block（池把一块没派的那一截也当派了）。
#[test]
fn every_pending_state_is_dispatched_exactly_once_and_the_rest_stays_in_the_pool() {
    let long = stream_tagged(1);
    let short = stream_tagged(2);
    let streams = streams_of(&[(long, 2, 100), (short, 2, 35)]);
    let cards = [
        card_leaving(0, 4 + mebibytes_of_states(10)),
        card_leaving(1, 4 + mebibytes_of_states(10)),
    ];
    let mut pool = pool_with(&[
        (long, 0..40, milliseconds(0)),
        (short, 0..35, milliseconds(1)),
        (long, 40..100, milliseconds(2)),
    ]);
    let states_of = |blocks: &[PendingBlock]| -> BTreeSet<(StreamKey, u64)> {
        blocks
            .iter()
            .flat_map(|block| {
                block
                    .undispatched
                    .clone()
                    .map(|ordinal| (block.stream, ordinal))
            })
            .collect()
    };
    let mut residents = Vec::new();
    let mut dispatched_so_far: BTreeSet<(StreamKey, u64)> = BTreeSet::new();
    for wave in 0..200u64 {
        let before = undispatched(&pool);
        if before.is_empty() {
            break;
        }
        let plan = dispatched(decide(
            0,
            &cards,
            &residents,
            &streams,
            &before,
            milliseconds(wave),
        ));
        let mut in_this_wave: BTreeMap<(StreamKey, u64), u32> = BTreeMap::new();
        for card_plan in &plan.cards {
            for batch in &card_plan.batches {
                for piece in &batch.pieces {
                    for ordinal in piece.ordinals.clone() {
                        *in_this_wave.entry((batch.stream, ordinal)).or_default() += 1;
                    }
                }
            }
        }
        let twice: Vec<_> = in_this_wave
            .iter()
            .filter(|(_, count)| **count != 1)
            .collect();
        assert!(
            twice.is_empty(),
            "第 {wave} 波里有态派了不止一次：{twice:?}"
        );
        let pending_before = states_of(&before);
        let this_wave: BTreeSet<(StreamKey, u64)> = in_this_wave.keys().copied().collect();
        assert!(
            this_wave.is_subset(&pending_before),
            "第 {wave} 波派了池里没有的态"
        );
        assert!(
            dispatched_so_far.is_disjoint(&this_wave),
            "第 {wave} 波又派了前几波派过的态"
        );
        residents = apply_the_wave(&mut pool, &plan, &residents);
        let expected_left: BTreeSet<_> = pending_before.difference(&this_wave).copied().collect();
        assert_eq!(
            states_of(&undispatched(&pool)),
            expected_left,
            "第 {wave} 波之后池里留的不是没派到的那些"
        );
        judge_the_wave(&mut pool, &plan);
        dispatched_so_far.extend(this_wave);
    }
    let every_state: BTreeSet<(StreamKey, u64)> = (0..100)
        .map(|ordinal| (long, ordinal))
        .chain((0..35).map(|ordinal| (short, ordinal)))
        .collect();
    assert_eq!(dispatched_so_far, every_state, "取完时每个态都派过");
    assert!(undispatched(&pool).is_empty(), "取完时池空");
}

/// 第 3 条：池里的态够把每张空着的卡各填满一次派活时发一波；不够而最老的待判等过限时（配置参数）也发；空池不发。
/// 弄坏开关：watermark-ignored（池里有态就发）、wait-limit-ignored（等多久都不发）。
#[test]
fn a_wave_goes_out_when_every_card_fills_once_or_the_oldest_waited_the_limit_and_never_from_an_empty_pool(
) {
    let stream = stream_tagged(1);
    let streams = streams_of(&[(stream, 1, 1000)]);
    let cards = [
        card_leaving(0, 1 + 1 + mebibytes_of_states(10)),
        card_leaving(1, 1 + 1 + mebibytes_of_states(10)),
    ];
    assert_eq!(
        decide(100, &cards, &[], &streams, &[], milliseconds(10)),
        WaveDecision::HoldTheEmptyPool,
        "空池不发"
    );
    let fifteen = undispatched(&pool_with(&[(stream, 0..15, milliseconds(0))]));
    match decide(100, &cards, &[], &streams, &fifteen, milliseconds(10)) {
        WaveDecision::HoldBelowTheWatermark {
            waited_longest,
            wait_left,
        } => {
            assert_eq!(waited_longest, milliseconds(10));
            assert_eq!(wait_left, milliseconds(90));
        }
        other @ (WaveDecision::HoldTheEmptyPool
        | WaveDecision::HoldNothingPlaceableOnTheIdleCards
        | WaveDecision::Dispatch(_)) => {
            panic!("15 个态填不满两张各装 10 个态的卡、最老的才等 10 毫秒（限时 100）：该等，调度却说 {other:?}")
        }
    }
    let plan_after_the_limit = dispatched(decide(
        100,
        &cards,
        &[],
        &streams,
        &fifteen,
        milliseconds(150),
    ));
    assert_eq!(
        plan_after_the_limit.batch_count(),
        2,
        "等过限时就发，两张卡各一批"
    );
    let plan_with_the_shorter_limit =
        dispatched(decide(5, &cards, &[], &streams, &fifteen, milliseconds(10)));
    assert_eq!(
        plan_with_the_shorter_limit.batch_count(),
        2,
        "限时是配置给的：限时 5 毫秒时等了 10 毫秒就发"
    );
    let twenty = undispatched(&pool_with(&[(stream, 0..20, milliseconds(0))]));
    let plan_when_every_card_fills = dispatched(decide(
        100,
        &cards,
        &[],
        &streams,
        &twenty,
        milliseconds(10),
    ));
    for card in [0, 1] {
        assert_eq!(
            card_plan(&plan_when_every_card_fills, card).batches[0].state_count(),
            10,
            "20 个态把两张卡各填满一次派活：没等过限时也发，每批 10 个态"
        );
    }
    let configuration = ServiceConfiguration::of_the_repository();
    assert!(
        configuration.oldest_pending_wait_limit > Duration::ZERO,
        "仓里的配置写了限时"
    );
    let every_item =
        "OLDEST_PENDING_WAIT_LIMIT_MILLISECONDS=42\nLIBRARY_SCAN_INTERVAL_MILLISECONDS=7\n\
                      CLAIM_LEASE_MILLISECONDS=9000\n";
    assert_eq!(
        ServiceConfiguration::from_the_text(&format!("# 注释\n{every_item}")),
        Ok(ServiceConfiguration {
            oldest_pending_wait_limit: milliseconds(42),
            library_scan_interval: milliseconds(7),
            claim_lease: milliseconds(9000),
        })
    );
    for refused in [
        String::new(),
        every_item.replace(
            "CLAIM_LEASE_MILLISECONDS=9000",
            "CLAIM_LEASE_MILLISECONDS=0",
        ),
        every_item.replace(
            "CLAIM_LEASE_MILLISECONDS=9000",
            "CLAIM_LEASE_MILLISECONDS=4294967296",
        ),
        every_item.replace(
            "LIBRARY_SCAN_INTERVAL_MILLISECONDS=7",
            "LIBRARY_SCAN_INTERVAL_MILLISECONDS=0",
        ),
        format!("{every_item}CLAIM_LEASE_MILLISECONDS=1\n"),
        format!("{every_item}UNKNOWN_ITEM=1\n"),
    ] {
        assert!(
            ServiceConfiguration::from_the_text(&refused).is_err(),
            "少写一项、租期或扫库间隔是 0、租期装不进 u32、写两次、认不得的名都要拒：{refused}"
        );
    }
}

/// 第 4 条：同一张卡上新装的几条流照剩下的显存均分一次派活的态数，不撞显存；一条流的态数少于分到的就取它的态数，多出来的给别的流。
/// 弄坏开关：batch-not-capped-by-the-state-count（不按态数压）、scratch-split-by-entry-order（先到先拿、不均分）。
#[test]
fn streams_sharing_a_card_split_the_remaining_memory_and_a_short_stream_takes_its_own_state_count()
{
    let long = stream_tagged(1);
    let short = stream_tagged(2);
    let streams = streams_of(&[(long, 4, 1000), (short, 2, 6)]);
    let cards = [card_leaving(0, 2 * (4 + 2) + mebibytes_of_states(40))];
    let pending = undispatched(&pool_with(&[
        (long, 0..1000, milliseconds(0)),
        (short, 0..6, milliseconds(1)),
    ]));
    let plan = dispatched(decide(0, &cards, &[], &streams, &pending, milliseconds(0)));
    let on_the_card = card_plan(&plan, 0);
    let long_states = states_fitting_in(mebibytes_of_states(40) - mebibytes_of_states(6));
    assert_eq!(
        on_the_card.loads,
        vec![
            JudgeToLoad {
                stream: long,
                states_per_dispatch: long_states
            },
            JudgeToLoad {
                stream: short,
                states_per_dispatch: 6
            },
        ],
        "短流取它自己的 6 个态，剩下的都给长流"
    );
    let batch_states: Vec<u64> = on_the_card
        .batches
        .iter()
        .map(|batch| batch.state_count())
        .collect();
    assert_eq!(batch_states, vec![long_states, 6]);
    let resident = on_the_card.resident_after_the_wave();
    assert!(
        resident.occupied_mebibytes() <= resident.capacity_mebibytes(),
        "不撞显存"
    );

    let other_long = stream_tagged(3);
    let two_long_streams = streams_of(&[(long, 4, 1000), (other_long, 4, 1000)]);
    let cards_for_two_long_streams = [card_leaving(0, 2 * 8 + mebibytes_of_states(40))];
    let pending_of_two_long_streams = undispatched(&pool_with(&[
        (long, 0..1000, milliseconds(0)),
        (other_long, 0..1000, milliseconds(1)),
    ]));
    let plan_for_two_long_streams = dispatched(decide(
        0,
        &cards_for_two_long_streams,
        &[],
        &two_long_streams,
        &pending_of_two_long_streams,
        milliseconds(0),
    ));
    let each = states_fitting_in(mebibytes_of_states(40) / 2);
    let sizes: Vec<u64> = card_plan(&plan_for_two_long_streams, 0)
        .loads
        .iter()
        .map(|load| load.states_per_dispatch)
        .collect();
    assert_eq!(sizes, vec![each, each], "两条长流均分剩下的显存");
}

/// 第 5 条：同一张卡上驻留几条流时，批按流轮流派，一条流的批不连着派完才轮到下一条：每一波各流一批，几波下来 长、另一条、长、另一条……
/// 弄坏开关：batches-of-a-stream-back-to-back（一条流的批在一波里连着派完）。
#[test]
fn batches_of_streams_sharing_a_card_alternate_from_wave_to_wave() {
    let earlier = stream_tagged(1);
    let later = stream_tagged(2);
    let streams = streams_of(&[(earlier, 4, 1000), (later, 4, 1000)]);
    let cards = [card_leaving(0, 2 * 8 + mebibytes_of_states(20))];
    let each = states_fitting_in(mebibytes_of_states(20) / 2);
    let mut pool = pool_with(&[
        (earlier, 0..3 * each, milliseconds(0)),
        (later, 0..3 * each, milliseconds(1)),
    ]);
    let mut residents = Vec::new();
    let mut sequence = Vec::new();
    for wave in 0..3u64 {
        let pending = undispatched(&pool);
        let plan = dispatched(decide(
            0,
            &cards,
            &residents,
            &streams,
            &pending,
            milliseconds(wave),
        ));
        assert_eq!(
            batch_streams(card_plan(&plan, 0)),
            vec![earlier, later],
            "第 {wave} 波：两条流各一批，先入池的在前"
        );
        sequence.extend(batch_streams(card_plan(&plan, 0)));
        residents = apply_the_wave(&mut pool, &plan, &residents);
        judge_the_wave(&mut pool, &plan);
    }
    assert_eq!(sequence, [earlier, later].repeat(3));
    assert!(undispatched(&pool).is_empty(), "三波派完两条流各三批");
}

/// 第 6 条：先来先取但不饿死。池里按入池先后排（键大的先入池就先排）；长流占满整张卡时，短流等过限时就整卡重装、与长流一起分，
/// 短流的批不排在长流全部批之后。
/// 弄坏开关：pool-order-ignored（按键排、不按入池先后）、starving-streams-not-rescued（等过限时也不腾卡）。
#[test]
fn streams_are_taken_in_entry_order_and_a_short_stream_is_not_left_behind_a_long_one() {
    let entered_first = stream_tagged(9);
    let entered_next = stream_tagged(1);
    let entry_order_streams = streams_of(&[(entered_first, 4, 1000), (entered_next, 4, 1000)]);
    let entry_order_cards = [card_leaving(0, 2 * 8 + mebibytes_of_states(20))];
    let entry_order_pending = undispatched(&pool_with(&[
        (entered_first, 0..1000, milliseconds(0)),
        (entered_next, 0..1000, milliseconds(1)),
    ]));
    let entry_order_plan = dispatched(decide(
        0,
        &entry_order_cards,
        &[],
        &entry_order_streams,
        &entry_order_pending,
        milliseconds(1),
    ));
    assert_eq!(
        batch_streams(card_plan(&entry_order_plan, 0)),
        vec![entered_first, entered_next],
        "先入池的先排（与键的大小无关）"
    );

    let long = stream_tagged(1);
    let short = stream_tagged(2);
    let streams = streams_of(&[(long, 4, 10_000), (short, 4, 5)]);
    let cards = [card_leaving(0, 4 + mebibytes_of_states(30))];
    let whole_card = states_fitting_in(mebibytes_of_states(30));
    let mut pool = pool_with(&[
        (long, 0..10_000, milliseconds(0)),
        (short, 0..5, milliseconds(10)),
    ]);
    let mut residents = vec![ResidentJudge {
        card: CardIndex(0),
        stream: long,
        states_per_dispatch: whole_card,
    }];
    let pending = undispatched(&pool);
    let plan = dispatched(decide(
        100,
        &cards,
        &residents,
        &streams,
        &pending,
        milliseconds(20),
    ));
    assert_eq!(
        batch_streams(card_plan(&plan, 0)),
        vec![long],
        "短流还没等过限时：长流占着整张卡，这一波只有长流"
    );
    residents = apply_the_wave(&mut pool, &plan, &residents);
    judge_the_wave(&mut pool, &plan);
    let pending_after_the_first_wave = undispatched(&pool);
    let plan_after_the_short_stream_waited = dispatched(decide(
        100,
        &cards,
        &residents,
        &streams,
        &pending_after_the_first_wave,
        milliseconds(200),
    ));
    let on_the_card = card_plan(&plan_after_the_short_stream_waited, 0);
    assert_eq!(
        on_the_card.evictions,
        vec![long],
        "整卡重装：长流的判器换出"
    );
    let loaded: Vec<StreamKey> = on_the_card.loads.iter().map(|load| load.stream).collect();
    assert_eq!(loaded, vec![long, short], "长流与短流一起重装、均分");
    let short_batch = on_the_card
        .batches
        .iter()
        .find(|batch| batch.stream == short)
        .expect("短流等过限时，这一波有它的批");
    assert_eq!(short_batch.state_count(), 5);
    residents = apply_the_wave(&mut pool, &plan_after_the_short_stream_waited, &residents);
    judge_the_wave(&mut pool, &plan_after_the_short_stream_waited);
    assert!(
        undispatched(&pool).iter().any(|block| block.stream == long),
        "短流判完时长流还有没派的态：短流没排在长流全部批之后"
    );
    assert_eq!(residents.len(), 2);
}

/// 第 7 条：驻留优先：已经在某张卡上有判器的流派回那张卡（不新装）；要给新流腾显存时，只换出没有待判态的判器。
/// 弄坏开关：residence-ignored（不认驻留、当新流重装）、every-resident-judge-evicted（腾显存时有待判态的也换出）。
#[test]
fn a_resident_stream_goes_back_to_its_card_and_only_judges_without_pending_states_are_evicted() {
    let resident_stream = stream_tagged(1);
    let streams = streams_of(&[(resident_stream, 4, 1000)]);
    let cards = [
        card_leaving(0, 4 + mebibytes_of_states(10)),
        card_leaving(1, 4 + mebibytes_of_states(10)),
    ];
    let residents = [ResidentJudge {
        card: CardIndex(1),
        stream: resident_stream,
        states_per_dispatch: 10,
    }];
    let pending = undispatched(&pool_with(&[(resident_stream, 0..10, milliseconds(0))]));
    let plan = dispatched(decide(
        0,
        &cards,
        &residents,
        &streams,
        &pending,
        milliseconds(0),
    ));
    assert_eq!(
        batch_streams(card_plan(&plan, 1)),
        vec![resident_stream],
        "派回驻留的那张卡"
    );
    assert!(card_plan(&plan, 1).loads.is_empty(), "驻留的判器不重装");
    assert!(card_plan(&plan, 1).evictions.is_empty());
    assert!(card_plan(&plan, 0).loads.is_empty(), "别的卡不为它新装判器");

    let pending_stream = stream_tagged(1);
    let idle_stream = stream_tagged(2);
    let new_stream = stream_tagged(3);
    let streams_with_an_idle_resident = streams_of(&[
        (pending_stream, 4, 1000),
        (idle_stream, 4, 1000),
        (new_stream, 4, 1000),
    ]);
    let judge_mebibytes = 4 + mebibytes_of_states(5);
    let cards_holding_two_judges = [card_leaving(0, 2 * judge_mebibytes + 2)];
    let pending_and_idle_residents = [
        ResidentJudge {
            card: CardIndex(0),
            stream: pending_stream,
            states_per_dispatch: 5,
        },
        ResidentJudge {
            card: CardIndex(0),
            stream: idle_stream,
            states_per_dispatch: 5,
        },
    ];
    let pending_of_the_pending_and_new_streams = undispatched(&pool_with(&[
        (pending_stream, 0..100, milliseconds(0)),
        (new_stream, 0..100, milliseconds(1)),
    ]));
    let plan_evicting_the_idle = dispatched(decide(
        0,
        &cards_holding_two_judges,
        &pending_and_idle_residents,
        &streams_with_an_idle_resident,
        &pending_of_the_pending_and_new_streams,
        milliseconds(1),
    ));
    let on_the_card = card_plan(&plan_evicting_the_idle, 0);
    assert_eq!(
        on_the_card.evictions,
        vec![idle_stream],
        "只换出没有待判态的那一个"
    );
    let loaded: Vec<StreamKey> = on_the_card.loads.iter().map(|load| load.stream).collect();
    assert_eq!(loaded, vec![new_stream], "有待判态的不重装，新流装上");
    assert_eq!(batch_streams(on_the_card), vec![pending_stream, new_stream]);
}

/// 第 8 条：同一份输入（卡的额度、池的内容、各流的表大小与态数、驻留的判器）出同一份计划，调用方给的次序打乱了也一样。
/// 弄坏开关：input-order-followed（照调用方给的次序走）。
#[test]
fn the_same_inputs_give_the_same_plan_whatever_order_they_come_in() {
    let streams = streams_of(&[
        (stream_tagged(1), 3, 500),
        (stream_tagged(2), 60, 40),
        (stream_tagged(3), 10, 900),
        (stream_tagged(4), 1, 7),
    ]);
    let cards = vec![
        card_leaving(0, 70 + mebibytes_of_states(30)),
        card_leaving(1, 20 + mebibytes_of_states(12)),
        card_leaving(2, 100 + mebibytes_of_states(50)),
    ];
    let residents = vec![
        ResidentJudge {
            card: CardIndex(0),
            stream: stream_tagged(2),
            states_per_dispatch: 9,
        },
        ResidentJudge {
            card: CardIndex(2),
            stream: stream_tagged(3),
            states_per_dispatch: 20,
        },
    ];
    let pending = undispatched(&pool_with(&[
        (stream_tagged(3), 0..300, milliseconds(0)),
        (stream_tagged(1), 0..200, milliseconds(1)),
        (stream_tagged(2), 0..40, milliseconds(2)),
        (stream_tagged(4), 0..7, milliseconds(3)),
        (stream_tagged(1), 200..500, milliseconds(4)),
    ]));
    let once = dispatched(decide(
        0,
        &cards,
        &residents,
        &streams,
        &pending,
        milliseconds(50),
    ));
    let again = dispatched(decide(
        0,
        &cards,
        &residents,
        &streams,
        &pending,
        milliseconds(50),
    ));
    assert_eq!(once, again, "同一份输入排两次");
    let reversed_cards: Vec<_> = cards.iter().rev().copied().collect();
    let reversed_residents: Vec<_> = residents.iter().rev().copied().collect();
    let reversed_pending: Vec<_> = pending.iter().rev().cloned().collect();
    let shuffled = dispatched(decide(
        0,
        &reversed_cards,
        &reversed_residents,
        &streams,
        &reversed_pending,
        milliseconds(50),
    ));
    assert_eq!(once, shuffled, "输入的次序打乱了，计划照旧");
    assert!(once.batch_count() >= 3, "样本里三张卡都有活：{once:?}");
}

/// 一张卡这一波新装那一刻占的（MiB）：留着的判器照驻留的账，新装的每个判器照新装那一刻（表两份）。
fn mebibytes_while_the_wave_loads(
    card_plan: &CardPlanOfTheWave,
    residents: &[ResidentJudge],
    streams: &BTreeMap<StreamKey, StreamFacts>,
) -> u64 {
    let evicted: BTreeSet<StreamKey> = card_plan.evictions.iter().copied().collect();
    let staying: u64 = residents
        .iter()
        .filter(|resident| resident.card == card_plan.card && !evicted.contains(&resident.stream))
        .map(|resident| {
            mebibytes_of_a_judge(&streams[&resident.stream], resident.states_per_dispatch)
        })
        .sum();
    let loading: u64 = card_plan
        .loads
        .iter()
        .map(|load| {
            mebibytes_while_loading_a_judge(&streams[&load.stream], load.states_per_dispatch)
        })
        .sum();
    staying + loading
}

/// 第 9 条：新装那一刻（`src/batch_scheduling.rs` 文件头「新装那一刻」）：判表上卡时另有一份同样大的暂存（验证包 `create_buffer_init`，
/// wgpu 下一次提交做完才放），新装的判器分到的草稿区加上表两份不超过这张卡给判器的额度：一条流独占一张空卡、两条流一起新装、
/// 卡上留着一个有待判态的判器再新装一条，三种都照这一条；这一波之后驻留的照旧只记一份表。
/// 弄坏开关：table-staging-not-reserved（分草稿区时只给表留一份）。
#[test]
fn a_newly_loaded_judge_leaves_room_for_a_second_copy_of_its_table_while_it_loads() {
    let large = stream_tagged(1);
    let other = stream_tagged(2);
    let streams = streams_of(&[(large, 300, 10_000), (other, 100, 10_000)]);
    let card = card_leaving(0, 600 + mebibytes_of_states(40));
    let kept = ResidentJudge {
        card: CardIndex(0),
        stream: other,
        states_per_dispatch: 5,
    };
    let scenarios: [LoadingScenario; 3] = [
        (
            "一条流独占一张空卡",
            Vec::new(),
            vec![(large, 0..10_000, milliseconds(0))],
        ),
        (
            "两条流一起新装",
            Vec::new(),
            vec![
                (large, 0..10_000, milliseconds(0)),
                (other, 0..10_000, milliseconds(1)),
            ],
        ),
        (
            "卡上留着一个有待判态的判器再新装一条",
            vec![kept],
            vec![
                (other, 0..10_000, milliseconds(0)),
                (large, 0..10_000, milliseconds(1)),
            ],
        ),
    ];
    for (scenario, residents, blocks) in scenarios {
        let pending = undispatched(&pool_with(&blocks));
        let plan = dispatched(decide(
            0,
            &[card],
            &residents,
            &streams,
            &pending,
            milliseconds(2),
        ));
        let on_the_card = card_plan(&plan, 0);
        assert!(
            !on_the_card.loads.is_empty(),
            "{scenario}：这一波有新装的判器"
        );
        let while_loading = mebibytes_while_the_wave_loads(on_the_card, &residents, &streams);
        assert!(
            while_loading <= card.mebibytes_for_the_judges(),
            "{scenario}：新装那一刻占 {while_loading} MiB（新表两份），这张卡给判器的只有 {} MiB：{:?}",
            card.mebibytes_for_the_judges(),
            on_the_card.loads
        );
        let tables_loaded: u64 = on_the_card
            .loads
            .iter()
            .map(|load| streams[&load.stream].table_mebibytes)
            .sum();
        assert_eq!(
            on_the_card.resident_after_the_wave().occupied_mebibytes() + tables_loaded,
            while_loading,
            "{scenario}：这一波之后驻留的照旧只记一份表"
        );
    }
}

/// 核对器借的显存（`src/card_memory_ledger.rs` 文件头「核对器」）：六张事实表照两份算（验证包用 `create_buffer_init` 建表，wgpu 另开一份同样大的暂存，
/// 下一次提交做完才放；同判器新装那一刻），加同时还活着的几趟：借的不少于「表两份加几趟」折成的 MiB。样本的录制流写表 64 MiB（每次写 4 个字），
/// 表只算一份时差出整整一份表、不被向上取整盖住。
/// 弄坏开关：facts-tables-staging-not-reserved（表只算一份）。
#[test]
fn a_facts_verifier_borrows_two_copies_of_its_tables_and_its_passes() {
    const RECORDED_WRITES: usize = 1 << 22;
    const STATES_OF_THE_LARGEST_BLOCK: u64 = 64;
    let facts = FlowFacts {
        recorded_write_count: 0,
        enumerated_write_count: 0,
        is_tearable: vec![false; RECORDED_WRITES],
        torn_images: Vec::new(),
        segments: Vec::new(),
        state_count: STATES_OF_THE_LARGEST_BLOCK,
        units: Vec::new(),
        roots: Vec::new(),
        newest_base_root: None,
    };
    let tables = bytes_of_the_facts_tables_on_the_card(&facts);
    assert!(
        tables >= 4 * BYTES_PER_MEBIBYTE,
        "取样：表要大到一份表差得出整 MiB（{tables} 字节）"
    );
    let passes = bytes_of_one_facts_verifier_pass(STATES_OF_THE_LARGEST_BLOCK)
        * PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD;
    let lent = mebibytes_of_a_facts_verifier_under(
        &facts,
        STATES_OF_THE_LARGEST_BLOCK,
        &BreakSwitches::of_the_environment(),
    );
    let at_least = (2 * tables + passes).div_ceil(BYTES_PER_MEBIBYTE);
    assert!(
        lent >= at_least,
        "核对器借 {lent} MiB，少于表两份加几趟的 {at_least} MiB（表 {tables} 字节、几趟 {passes} 字节）"
    );
}
