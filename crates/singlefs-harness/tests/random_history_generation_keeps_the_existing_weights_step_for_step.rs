//! 随机历史的生成器加一种只进新比重的步（C696，`.claude/kb/checks-owed.md` 那一行；设计调查 `research/prompts/` 第 26 批 C696 候选「丁」）
//! 之后，已有的每一组比重在一批种子下生成的历史与加之前逐位相同：快档、各取样点、崩溃注入与实验装置的复现都按它们生成，钉着的取样点与
//! 变异表里「必须红」的命题才还是同一批历史。

use singlefs_harness::history::{generate_history_with_weights, GenerationWeights, HistorySeed};

/// 每组比重生成几段历史（种子 0 起）。
const SEEDS_PER_WEIGHTS: u64 = 64;
/// 每段历史几步：比快档（30 步）长，删除那一组 unlink 之后那张表也抽得到。
const OPERATIONS_PER_HISTORY: usize = 40;

/// 64 位 FNV-1a 的起始值与乘数：把一组比重生成的全部历史（`Debug` 写法：种子、起点、每一步与它抽到的参数）折成一个数。
const FINGERPRINT_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FINGERPRINT_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fingerprint_of_the_histories_generated_with(weights: &GenerationWeights) -> u64 {
    (0..SEEDS_PER_WEIGHTS)
        .map(|seed| {
            format!(
                "{:?}",
                generate_history_with_weights(HistorySeed(seed), OPERATIONS_PER_HISTORY, weights)
            )
        })
        .flat_map(String::into_bytes)
        .fold(FINGERPRINT_OFFSET_BASIS, |fingerprint, byte| {
            (fingerprint ^ u64::from(byte)).wrapping_mul(FINGERPRINT_PRIME)
        })
}

/// 已有七组比重各自在种子 0..64、每段 40 步下生成的历史，指纹与加新步之前在同一份源码上量的逐个相同（2026-10-05 加之前先在不改动的副本里
/// 照同一个折法把每一组打出来、抄进来）。新步只进它自己那一组比重：已有各组的份数表里没有它，抽法与份数之和不变，抽到的每一步与它的参数都不变。
/// 判别力：把新步按崩溃恢复抛弃根的份数放进快档那一组（`crates/mutations.tsv` 一行）时快档那一组的指纹变了，这一条红。
#[test]
fn every_existing_weights_generate_the_same_histories_as_before_the_newest_system_configuration_slot_step(
) {
    let fingerprints: Vec<(&'static str, u64)> = [
        GenerationWeights::BROAD,
        GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
        GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
        GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,
    ]
    .iter()
    .map(|weights| {
        (
            weights.environment_value(),
            fingerprint_of_the_histories_generated_with(weights),
        )
    })
    .collect();
    assert_eq!(
        fingerprints,
        vec![
            ("broad", 4_211_750_580_707_973_314),
            ("reuse", 8_928_070_955_642_935_750),
            ("rollback", 15_225_864_873_440_202_800),
            ("wall", 5_861_325_111_516_291_226),
            ("unit-area-wall", 13_275_214_430_534_233_912),
            ("several-leaves", 1_583_708_721_650_671_726),
            ("orphans", 10_448_260_464_455_344_558),
        ],
        "已有七组比重在种子 0..64、每段 40 步下生成的历史，与加新步之前逐位相同（指纹在副本里加之前量的）"
    );
}
