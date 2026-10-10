// F3-c 抽函数前后种子逐位相同：把合入前 draw_second_crash_points 里内联的那段异或（diff 里 `-` 行）
// 与合入后 seed_of_the_second_crash_points（diff 里 `+` 行）逐字抄成两个函数，对随机输入比 u64。
// 常量与指纹函数取自 crates/singlefs-checker-tier/src/crash_injection.rs 第 109、112、115、2806 行起。
const SECOND_CRASH_POINT_SEED_SALT: u64 = 0x73_65_63_6f_6e_64_63_72;
const PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PERSISTED_SUBSET_FINGERPRINT_PRIME: u64 = 0x0000_0100_0000_01b3;
struct HistorySeed(u64);
struct CrashPoint { segment_index: usize, persisted_within_the_segment: Vec<bool> }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SecondCrashPointSeed(u64);
fn fingerprint_of_the_persisted_subset(p: &[bool]) -> u64 {
    p.iter().fold(PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS, |f, is| {
        (f ^ u64::from(*is)).wrapping_mul(PERSISTED_SUBSET_FINGERPRINT_PRIME)
    })
}
fn before(seed: &HistorySeed, crash_point: &CrashPoint) -> u64 {
    seed.0
        ^ SECOND_CRASH_POINT_SEED_SALT
        ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
        ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment)
}
fn seed_of_the_second_crash_points(seed: HistorySeed, crash_point: &CrashPoint) -> SecondCrashPointSeed {
    SecondCrashPointSeed(
        seed.0
            ^ SECOND_CRASH_POINT_SEED_SALT
            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
            ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),
    )
}
fn main() {
    let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; x };
    let mut checked = 0u64;
    for _ in 0..2_000_000 {
        let seed = next();
        let segment_index = (next() % 1000) as usize;
        let len = (next() % 80) as usize;
        let bits: Vec<bool> = (0..len).map(|_| next() & 1 == 1).collect();
        let cp = CrashPoint { segment_index, persisted_within_the_segment: bits };
        let a = before(&HistorySeed(seed), &cp);
        let b = seed_of_the_second_crash_points(HistorySeed(seed), &cp).0;
        assert_eq!(a, b);
        checked += 1;
    }
    // 穷举：长 0..=14 的全部子集 × 段号 0..40 × 种子 0 与 7
    let mut exhaustive = 0u64;
    for len in 0..=14usize {
        for mask in 0..(1u32 << len) {
            for segment_index in 0..40usize {
                for seed in [0u64, 7] {
                    let bits: Vec<bool> = (0..len).map(|k| (mask >> k) & 1 == 1).collect();
                    let cp = CrashPoint { segment_index, persisted_within_the_segment: bits };
                    assert_eq!(before(&HistorySeed(seed), &cp), seed_of_the_second_crash_points(HistorySeed(seed), &cp).0);
                    exhaustive += 1;
                }
            }
        }
    }
    println!("f3c-identity random={checked} exhaustive={exhaustive} mismatches=0");
}
