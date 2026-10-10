//! F3-e 三条取样（零轮）罩不罩得住更长的段、别的种子基：照 `crates/singlefs-checker-tier/src/crash_injection.rs` 的
//! `fingerprint_of_the_persisted_subset` 与 `seed_of_the_second_crash_points` 抄一份式子（那两个函数不公开），
//! 把入库的几条断言（F3-c、F3-d 的 A/B/C、F3-e 的 A2/B3/C2）在几种改坏上逐条算一遍，再算改坏在真历史的段长与战役种子上损多少。
use std::collections::BTreeSet;

const SECOND_CRASH_POINT_SEED_SALT: u64 = 0x73_65_63_6f_6e_64_63_72;
const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug)]
pub enum Variant {
    Shipped,
    /// 指纹只折段内前 32 个写。
    FoldFirst32,
    /// 指纹只折段内末 32 个写。
    FoldLast32,
    /// 历史种子只留最低位与最高位。
    SeedBitsZeroAndSixtyThree,
    /// 历史种子只留低 8 位与最高位。
    SeedLowByteAndBitSixtyThree,
    /// 历史种子对 3 取余。
    SeedModuloThree,
    /// 入库那三行变异：M7a 指纹只折段内末 12 个写、M7d 段号只取低 6 位、M7f 历史种子只取最低位。
    M7aFoldLast12,
    M7dSegmentLowSixBits,
    M7fSeedLowestBit,
}

fn fingerprint(variant: Variant, persisted: &[bool]) -> u64 {
    let folded: &[bool] = match variant {
        Variant::FoldFirst32 => &persisted[..persisted.len().min(32)],
        Variant::FoldLast32 => &persisted[persisted.len().saturating_sub(32)..],
        Variant::M7aFoldLast12 => &persisted[persisted.len().saturating_sub(12)..],
        _ => persisted,
    };
    folded
        .iter()
        .fold(OFFSET_BASIS, |fp, bit| (fp ^ u64::from(*bit)).wrapping_mul(PRIME))
}

pub fn seed(variant: Variant, history_seed: u64, segment_index: usize, persisted: &[bool]) -> u64 {
    let history = match variant {
        Variant::SeedBitsZeroAndSixtyThree => history_seed & 0x8000_0000_0000_0001,
        Variant::SeedLowByteAndBitSixtyThree => history_seed & 0x8000_0000_0000_00ff,
        Variant::SeedModuloThree => history_seed % 3,
        Variant::M7fSeedLowestBit => history_seed & 1,
        _ => history_seed,
    };
    let segment = match variant {
        Variant::M7dSegmentLowSixBits => segment_index as u64 & 63,
        _ => segment_index as u64,
    };
    history ^ SECOND_CRASH_POINT_SEED_SALT ^ segment ^ fingerprint(variant, persisted)
}

/// 段号 `segment`、前 `landed` 个写都落了、段末 12 个写取扣下集合非空的 4095 种。
fn varied_end(variant: Variant, history_seed: u64, segment: usize, landed: usize) -> Vec<u64> {
    (0..4095u32)
        .map(|mask| {
            let persisted: Vec<bool> = std::iter::repeat_n(true, landed)
                .chain((0..12).map(|bit| mask >> bit & 1 == 1))
                .collect();
            seed(variant, history_seed, segment, &persisted)
        })
        .collect()
}

fn all_distinct(seeds: &[u64]) -> bool {
    seeds.iter().copied().collect::<BTreeSet<u64>>().len() == seeds.len()
}

fn distinct(seeds: &[u64]) -> usize {
    seeds.iter().copied().collect::<BTreeSet<u64>>().len()
}

/// 长 `length` 的段，扣下其中一个或两个写：交回种子。
fn withhold_one_or_two(variant: Variant, length: usize, history_seed: u64) -> Vec<u64> {
    let mut seeds = Vec::new();
    for first in 0..length {
        let mut one = vec![true; length];
        one[first] = false;
        seeds.push(seed(variant, history_seed, 16, &one));
        for second in first + 1..length {
            let mut two = vec![true; length];
            two[first] = false;
            two[second] = false;
            seeds.push(seed(variant, history_seed, 16, &two));
        }
    }
    seeds
}

pub fn run() {
    let variants = [
        Variant::Shipped,
        Variant::FoldFirst32,
        Variant::FoldLast32,
        Variant::SeedBitsZeroAndSixtyThree,
        Variant::SeedLowByteAndBitSixtyThree,
        Variant::SeedModuloThree,
        Variant::M7aFoldLast12,
        Variant::M7dSegmentLowSixBits,
        Variant::M7fSeedLowestBit,
    ];
    let campaign_base = singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
    for variant in variants {
        let f3c = all_distinct(&varied_end(variant, 0, 16, 0));
        let f3d_a: Vec<u64> = (0..64).flat_map(|segment| varied_end(variant, 0, segment, 0)).collect();
        let f3d_b = distinct(&varied_end(variant, 0, 16, 16)) == 4095;
        let f3d_c: Vec<u64> = [0u64, 1].iter().flat_map(|history| varied_end(variant, *history, 16, 0)).collect();
        let a2_segments: Vec<usize> = (0..=63)
            .chain((6..=20).flat_map(|exponent| [1usize << exponent, (1usize << exponent) + 1]))
            .collect();
        let e_a2: Vec<u64> = a2_segments.iter().flat_map(|segment| varied_end(variant, 0, *segment, 0)).collect();
        let e_b3 = withhold_one_or_two(variant, 32, 0);
        let e_c2: Vec<u64> = [0u64, 1, 1 << 63].iter().flat_map(|history| varied_end(variant, *history, 16, 0)).collect();
        let long_48 = withhold_one_or_two(variant, 48, 0);
        let long_626 = withhold_one_or_two(variant, 626, 0);
        // 攻方提的两条（零轮）：C3 历史种子 0 与 2^k（k = 0..=63）各 4095 个；B4 长 1024 的段扣下一个写（1024 种）。
        let c3: Vec<u64> = std::iter::once(0u64)
            .chain((0..64).map(|bit| 1u64 << bit))
            .flat_map(|history| varied_end(variant, history, 16, 0))
            .collect();
        let b4: Vec<u64> = (0..1024)
            .map(|withheld| {
                let mut persisted = vec![true; 1024];
                persisted[withheld] = false;
                seed(variant, 0, 16, &persisted)
            })
            .collect();
        println!("F3PROPOSED {variant:?}: C3={} B4={}", green(all_distinct(&c3)), green(all_distinct(&b4)));
        let campaign: Vec<u64> = (0..80u64)
            .flat_map(|offset| varied_end(variant, campaign_base.wrapping_add(offset), 16, 0))
            .collect();
        println!(
            "F3 {variant:?}: F3-c={} F3-d-A={} F3-d-B={} F3-d-C={} F3-e-A2={} F3-e-B3={} F3-e-C2={} | 长48扣一两个写 {}/{} 不同 | 长626扣一两个写 {}/{} 不同 | 战役种子基+0..80、段16长12 {}/{} 不同",
            green(f3c),
            green(all_distinct(&f3d_a)),
            green(f3d_b),
            green(all_distinct(&f3d_c)),
            green(all_distinct(&e_a2)),
            green(all_distinct(&e_b3)),
            green(all_distinct(&e_c2)),
            distinct(&long_48),
            long_48.len(),
            distinct(&long_626),
            long_626.len(),
            distinct(&campaign),
            campaign.len()
        );
    }
}

fn green(holds: bool) -> &'static str {
    if holds {
        "绿"
    } else {
        "红"
    }
}
