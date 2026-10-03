//! 攻方腿 m3-formal-code-r1 的原型（只在拷贝里）：M3 不动点照条款字面（D13 已定项 1 M3 一行：前提只有「所选根之上的记录不超在飞上限」，
//! 同一张表、同一个读者、同一个开关）在随机小表上用编进产品的 replay_core 跑两次；按 I-8.12 成立与否分开数反例。
use singlefs_replay_prefix::{
    replay_core, CoreFailure, CoreRecord, JournalCounter, NamedUnitVerification,
    PrefixKernelCheckpointTxg, PrefixKernelInstanceGeneration,
    PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish,
    TransactionNumber,
};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

fn i812_holds(records: &[CoreRecord<Option<usize>>]) -> bool {
    records.iter().all(|earlier| {
        records.iter().all(|later| {
            !(later.instance == earlier.instance && later.counter.0 > earlier.counter.0)
                || earlier.checkpoint_txg.0 <= later.checkpoint_txg.0
        })
    })
}

#[test]
fn fixed_point_under_the_clause_premise_only() {
    let cases: u64 = std::env::var("ATTACK_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(200_000);
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let (mut premise_cases, mut violated_with_i812, mut violated_without_i812, mut first_ok, mut premise_without_i812) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut printed = 0;
    for _ in 0..cases {
        let root_instance = PrefixKernelInstanceGeneration(1 + u32::try_from(rng.below(2)).unwrap());
        let root_txg = PrefixKernelCheckpointTxg(3);
        let mut records: Vec<CoreRecord<Option<usize>>> = Vec::new();
        for instance in 1u32..=2 {
            for counter in 1u64..=7 {
                if rng.below(3) == 0 {
                    continue;
                }
                let index = records.len();
                records.push(CoreRecord {
                    instance: PrefixKernelInstanceGeneration(instance),
                    counter: JournalCounter(counter),
                    checkpoint_txg: PrefixKernelCheckpointTxg(2 + rng.below(5)),
                    transaction: TransactionNumber(rng.below(2)),
                    is_commit: rng.below(5) != 0,
                    ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish(1 + u32::try_from(rng.below(2)).unwrap()),
                    place_in_publish: if rng.below(3) == 0 {
                        PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                    } else {
                        PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish
                    },
                    new_root_source: Some(index),
                });
            }
        }
        if rng.below(2) == 0 {
            // 一半的表照 I-8.12 排：同一实例里 txg 按计数器次序不减（txg 的多重集不变，只重排）。
            for instance in 1u32..=2 {
                let positions: Vec<usize> = (0..records.len()).filter(|&i| records[i].instance.0 == instance).collect();
                let mut txgs: Vec<u64> = positions.iter().map(|&i| records[i].checkpoint_txg.0).collect();
                txgs.sort_unstable();
                for (&i, txg) in positions.iter().zip(txgs) {
                    records[i].checkpoint_txg = PrefixKernelCheckpointTxg(txg);
                }
            }
        }
        let limit = usize::try_from(1 + rng.below(6)).unwrap();
        let broken_mask = if rng.below(4) == 0 { rng.next() } else { 0 };
        let unverified_mask = if rng.below(4) == 0 { rng.next() } else { 0 };
        let verification = if rng.below(2) == 0 { NamedUnitVerification::Verify } else { NamedUnitVerification::Skip };
        let back_chain_broken = |index: usize| (broken_mask >> (index % 64)) & 1 == 1;
        let named_units_verified = |index: usize| (unverified_mask >> (index % 64)) & 1 == 0;
        let Ok(first) = replay_core(root_instance, root_txg, None, &records, limit, verification, back_chain_broken, named_units_verified) else {
            continue;
        };
        first_ok += 1;
        let above = records.iter().filter(|r| r.instance == root_instance && r.checkpoint_txg.0 > root_txg.0).count();
        if above > limit {
            continue;
        }
        premise_cases += 1;
        let holds = i812_holds(&records);
        if !holds {
            premise_without_i812 += 1;
        }
        let (landed_instance, landed_txg) = match first.root_source_after_replay {
            None => (root_instance, root_txg),
            Some(index) => (records[index].instance, records[index].checkpoint_txg),
        };
        let again = replay_core(landed_instance, landed_txg, first.root_source_after_replay, &records, limit, verification, back_chain_broken, named_units_verified);
        let stays = match &again {
            Ok(second) => second.report.prefix_applied == 0 && second.root_source_after_replay == first.root_source_after_replay,
            Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => landed_txg.0 == u64::MAX,
            Err(_) => false,
        };
        if !stays {
            if holds { violated_with_i812 += 1 } else { violated_without_i812 += 1 }
            if printed < 3 {
                printed += 1;
                let shape: Vec<String> = records.iter().map(|r| format!("({},{},txg{},{}{})", r.instance.0, r.counter.0, r.checkpoint_txg.0,
                    if r.place_in_publish == PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish { "L" } else { "M" }, if r.is_commit { "c" } else { "" })).collect();
                println!("name=fixed-point-witness i812={holds} root=({},{}) limit={limit} first_applied={} landed=({},{}) again={:?} records={}",
                    root_instance.0, root_txg.0, first.report.prefix_applied, landed_instance.0, landed_txg.0,
                    again.as_ref().map(|s| (s.report.prefix_applied, s.root_source_after_replay)).map_err(|e| match e { CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor => "no-successor".to_string(), _ => "more-than-one-last-flag".to_string() }), shape.join(" "));
            }
        }
    }
    println!("name=fixed-point-brute cases={cases} first_ok={first_ok} premise_cases={premise_cases} premise_cases_without_i812={premise_without_i812} violated_with_i812={violated_with_i812} violated_without_i812={violated_without_i812}");
    assert_eq!(violated_with_i812, 0, "I-8.12 成立时不动点有反例：与 Verus 的证明相冲");
}
