//! 判表摊成 u32 字的布局：各张表每条几个字、表头、恢复自称的那一版那张表的码、
//! 组合字落进版本字的位置、移过 16 KiB 零的 CRC，与各张表摊平之后的 `JudgeTablesWords`。

use super::*;

/// 恢复自称的那一版那一张表每个状态几个字（布局写在 [`JudgeTables::words`] 填它的地方；内核里的同名常量由 `crate::crash_judge_gpu` 从这里生成）。
pub const CLAIMED_VERSION_WORDS: usize = 4;
/// 那一张表每个状态第 0 字的码：没写（取看 journal 那一遍落到的根）、写了而那一版没有根、写了一条根（第 1–3 字是它的实例代号与 checkpoint_txg lo/hi）。
pub const CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY: u32 = 0;
pub const CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT: u32 = 1;
pub const CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT: u32 = 2;

/// 一个状态那一栏恢复自称的那一版在判表里的 [`CLAIMED_VERSION_WORDS`] 个字。
pub(super) fn claimed_version_words(
    claimed_version: ClaimedVersionOfAGivenState,
) -> [u32; CLAIMED_VERSION_WORDS] {
    match claimed_version {
        ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery => [
            CLAIMED_VERSION_LANDED_ON_BY_THE_CONSULTED_RECOVERY,
            JUDGE_TABLE_ENTRY_ABSENT,
            JUDGE_TABLE_ENTRY_ABSENT,
            JUDGE_TABLE_ENTRY_ABSENT,
        ],
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None) => [
            CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITHOUT_A_ROOT,
            JUDGE_TABLE_ENTRY_ABSENT,
            JUDGE_TABLE_ENTRY_ABSENT,
            JUDGE_TABLE_ENTRY_ABSENT,
        ],
        ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(Some((
            instance,
            checkpoint_txg,
        ))) => [
            CLAIMED_VERSION_NEWEST_ROOT_WRITTEN_WITH_A_ROOT,
            instance.0,
            u32::try_from(checkpoint_txg.0 & 0xFFFF_FFFF).expect("低 32 位"),
            u32::try_from(checkpoint_txg.0 >> 32).expect("高 32 位"),
        ],
    }
}

/// 段表每条几个 u32（布局写在 [`JudgeTables::words`] 填它的地方；内核里的同名常量由 `crate::crash_judge_gpu` 从这里生成）。
pub const SEGMENT_WORDS: usize = 11;

/// 表里「没有」的写法。
pub const JUDGE_TABLE_ENTRY_ABSENT: u32 = u32::MAX;
/// 版本表最前面几条是全零单元，不是哪个落点的版本：第 0 条 32 KiB、第 1 条 16 KiB（盘尾只装得下 16 KiB 的槽；布局写在 [`JudgeTables::words`]
/// 填它们的地方）。GPU 读者 `read_at` 读单元区里一个没有落点的槽时交回它们（判决 `research/prompts/m3-batch29-code-r1-main-verification.md`
/// 第三节第 2 条，攻方改法 F-零槽）：那一槽基线与枚举写表里哪一次写都没碰过，CPU 稀疏读者在每个崩溃状态里读出的都是零。
/// 落点的第 9–11 字说它之后哪几槽是这样的槽（[`UnitSlotsNothingWroteAfterALocation`]）。
pub const ZERO_UNIT_VERSIONS: usize = 2;

/// 一个落点之后、同一块盘上下一个落点之前那一段谁都没写过的槽（落点表第 9–11 字）：在单元区里、整 16 KiB 在盘内，
/// 槽号 [`Self::first_slot`, `Self::end_slot`)（没有这样的槽时两个都是 0）；`Self::first_slot_a_whole_data_unit_does_not_fit_from`
/// 是这块盘上从哪一槽起整 32 KiB 不在盘内（GPU 从那一槽起交 16 KiB 那条全零单元，同真落点在盘尾只有 16 KiB）。槽号都装得进 u32（盘不到 64 TiB）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitSlotsNothingWroteAfterALocation {
    pub first_slot: u32,
    pub end_slot: u32,
    pub first_slot_a_whole_data_unit_does_not_fit_from: u32,
}
/// 写表每条几个 u32。
pub const WRITE_WORDS: usize = 12;
/// 落点表每条几个 u32。
pub const LOCATION_WORDS: usize = 12;
/// 版本表每条几个 u32：组合第 0 字（第 0–31 次写）、字节起点、四个检查字（[`version_checks`]）、逐扇区等不等掩码起点、组合第 1 字
/// （第 32–63 次写）、第 8 字 [`VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES`]，再接组合其余的字
/// （[`version_word_of_the_combination_word`]）。内核里的同名常量由 `crate::crash_judge_gpu` 从这里生成。
pub const VERSION_WORDS: usize =
    VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE + COMBINATION_WORDS_OF_A_VERSION - 2;
/// 版本表第 8 字（[`crc_shifted_past_sixteen_zero_kibibytes`]）：GPU 读者 `read_at` 拼读跨过落点末尾的 32 KiB 时拼整段 CRC 用
/// （`crash_judge_common.wgsl` 同名常量）。
pub const VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES: usize = 8;
/// 版本表里组合第 2 字（第 64–95 次写）在一条版本的第几字：接在第 8 字之后，其余各字依次往后。
pub const VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE: usize =
    VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES + 1;

/// 组合掩码第 `combination_word` 个 u32（[`CombinationOfTheWritesOnALocation::thirty_two_bit_words`]）在一条版本的第几字：第 0 字在第 0 字、
/// 第 1 字在第 7 字，其余从 [`VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE`] 起依次放（`crash_judge_common.wgsl` 的同名函数照同一个布局读）。
///
/// # Panics
/// `combination_word` 不小于 [`COMBINATION_WORDS_OF_A_VERSION`]。
#[must_use]
pub fn version_word_of_the_combination_word(combination_word: usize) -> usize {
    assert!(
        combination_word < COMBINATION_WORDS_OF_A_VERSION,
        "组合掩码只有 {COMBINATION_WORDS_OF_A_VERSION} 个字"
    );
    match combination_word {
        0 => 0,
        1 => 7,
        past_the_sixty_fourth_write => {
            VERSION_WORD_COMBINATION_PAST_THE_SIXTY_FOURTH_WRITE + past_the_sixty_fourth_write - 2
        }
    }
}

/// 一条版本打包进版本表的 [`VERSION_WORDS`] 个字（布局见 [`VERSION_WORDS`]）。
pub(super) fn packed_version_words(
    combination: &CombinationOfTheWritesOnALocation,
    byte_start: u32,
    checks: [u32; 4],
    equal_masks_start: u32,
    crc_shifted_past_sixteen_zero_kibibytes_of_the_version: u32,
) -> [u32; VERSION_WORDS] {
    let mut words = [0u32; VERSION_WORDS];
    for (combination_word, word) in combination.thirty_two_bit_words().into_iter().enumerate() {
        words[version_word_of_the_combination_word(combination_word)] = word;
    }
    words[1] = byte_start;
    words[2..6].copy_from_slice(&checks);
    words[6] = equal_masks_start;
    words[VERSION_WORD_CRC_SHIFTED_PAST_SIXTEEN_ZERO_KIBIBYTES] =
        crc_shifted_past_sixteen_zero_kibibytes_of_the_version;
    words
}

/// 单元槽的一版字节的前 16 KiB 后面接 16 KiB 全零，那 32 KiB 的 CRC-32C 异或 16 KiB 全零的 CRC-32C（版本表第 8 字）。CRC-32C 对异或线性：
/// 它再异或任意一段 16 KiB 的 CRC-32C，就是「这一版前 16 KiB 接那一段」32 KiB 的 CRC-32C（攻方 Z2，判决
/// `research/prompts/m3-batch33s-code-r1-main-verification.md` 第三节第 1 条）；GPU 读者拼「16 KiB 节点加下一槽」「空槽加下一已写槽前 16 KiB」
/// 两形的整段 CRC 靠它，不在内核里做 CRC 的乘法。别的落点（不是单元槽、或不到 16 KiB）记 0：不会被拼读。
#[must_use]
pub fn crc_shifted_past_sixteen_zero_kibibytes(kind: JudgeLocationKind, bytes: &[u8]) -> u32 {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    match kind {
        JudgeLocationKind::UnitSlot if bytes.len() >= node_bytes => {
            let mut followed_by_zeros = bytes[..node_bytes].to_vec();
            followed_by_zeros.resize(2 * node_bytes, 0);
            crc32_castagnoli(&followed_by_zeros) ^ crc32_castagnoli(&vec![0u8; node_bytes])
        }
        JudgeLocationKind::UnitSlot
        | JudgeLocationKind::SystemConfiguration
        | JudgeLocationKind::RootSlot
        | JudgeLocationKind::JournalRecord => 0,
    }
}
/// 更晚重叠写表每条几个 u32。
pub const LATER_WORDS: usize = 4;
/// 发布表每条几个 u32。
pub const PUBLISH_WORDS: usize = 8;
/// 期望版本表每条几个 u32：实例、txg 低高、单元 CRC 表起点、几个单元、有没有文件（第 5 字）。
pub const EXPECTED_WORDS: usize = 6;
/// 期望版本表第 5 字：这一版有文件（`crash_judge_recovery.wgsl` 同名常量）。
pub const EXPECTED_VERSION_WITH_A_FILE: u32 = 0;
/// 期望版本表第 5 字：文件从这一版起没有了（`crash_judge_recovery.wgsl` 同名常量）。
pub const EXPECTED_VERSION_WITHOUT_A_FILE: u32 = 1;
/// 头：12 个计数字段之后是盘的身份，共 32 个 u32（盘不超过 20 块）。
pub const HEADER_WORDS: usize = 32;

/// 摊平的几张 u32 表：GPU 内核的绑定各占一张。布局写在 [`JudgeTables::words`] 里填它们的地方。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgeTablesWords {
    pub header: Vec<u32>,
    pub recorded: Vec<u32>,
    pub replays: Vec<u32>,
    pub segments: Vec<u32>,
    pub segment_writes: Vec<u32>,
    pub writes: Vec<u32>,
    pub root_bytes: Vec<u32>,
    pub laters: Vec<u32>,
    pub locations: Vec<u32>,
    pub location_writes: Vec<u32>,
    pub versions: Vec<u32>,
    pub version_bytes: Vec<u32>,
    pub equal_masks: Vec<u32>,
    pub publishes: Vec<u32>,
    pub publish_records: Vec<u32>,
    pub copy_groups: Vec<u32>,
    pub copies: Vec<u32>,
    pub expected: Vec<u32>,
    pub chunks: Vec<u32>,
    /// 给定状态表的持久掩码（[`JudgeStateSource::GivenPersistedSets`]）；按段展开时是空的，内核按它空不空认状态从哪来。
    pub given_persisted_sets: Vec<u32>,
    /// 给定状态表每个状态那一栏恢复自称的那一版（[`CLAIMED_VERSION_WORDS`] 个字一个）；按段展开时是空的，内核按它空不空认要不要照它判第二条判据。
    pub claimed_versions: Vec<u32>,
}

impl JudgeTablesWords {
    /// 全部表按固定次序接成字节（摘要与上卡都用它）。
    #[must_use]
    pub fn digest_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for table in self.every_table() {
            out.extend_from_slice(&u32::try_from(table.len()).expect("表长").to_le_bytes());
            for word in table {
                out.extend_from_slice(&word.to_le_bytes());
            }
        }
        out
    }

    /// 每张表的名字，与 [`Self::every_table`] 同序（建表核目录时报哪张表满了用；内核里的表号常量也照这个次序）。
    #[must_use]
    pub fn every_table_name() -> [&'static str; JUDGE_TABLE_COUNT] {
        [
            "header",
            "recorded",
            "replays",
            "segments",
            "segment_writes",
            "writes",
            "root_bytes",
            "laters",
            "locations",
            "location_writes",
            "versions",
            "version_bytes",
            "equal_masks",
            "publishes",
            "publish_records",
            "copy_groups",
            "copies",
            "expected",
            "chunks",
            "given_persisted_sets",
            "claimed_versions",
        ]
    }

    /// 全部表，按绑定的次序。
    #[must_use]
    pub fn every_table(&self) -> [&Vec<u32>; JUDGE_TABLE_COUNT] {
        [
            &self.header,
            &self.recorded,
            &self.replays,
            &self.segments,
            &self.segment_writes,
            &self.writes,
            &self.root_bytes,
            &self.laters,
            &self.locations,
            &self.location_writes,
            &self.versions,
            &self.version_bytes,
            &self.equal_masks,
            &self.publishes,
            &self.publish_records,
            &self.copy_groups,
            &self.copies,
            &self.expected,
            &self.chunks,
            &self.given_persisted_sets,
            &self.claimed_versions,
        ]
    }
}

/// 打包表里几张表（内核里的 `TABLE_COUNT` 由 `crate::crash_judge_gpu` 从这里生成）。
pub const JUDGE_TABLE_COUNT: usize = 21;

/// 字节按小端 4 个一组装进 u32，末尾补 0。
#[must_use]
pub fn bytes_to_words(bytes: &[u8]) -> Vec<u32> {
    let (chunks, remainder) = bytes.as_chunks::<4>();
    let mut words: Vec<u32> = chunks
        .iter()
        .map(|chunk| u32::from_le_bytes(*chunk))
        .collect();
    if !remainder.is_empty() {
        let mut last = [0u8; 4];
        last[..remainder.len()].copy_from_slice(remainder);
        words.push(u32::from_le_bytes(last));
    }
    words
}

#[cfg(test)]
mod tests {
    use super::{crc_shifted_past_sixteen_zero_kibibytes, JudgeTablesWords};
    use crate::crash_judge_tables::JudgeLocationKind;
    use singlefs_core::checksum::crc32_castagnoli;

    /// 表名与表一一对应、不重。
    #[test]
    fn every_table_has_one_distinct_name() {
        let mut names = JudgeTablesWords::every_table_name().to_vec();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), JudgeTablesWords::every_table_name().len());
    }

    const SIXTEEN_KIBIBYTES: usize = 16384;

    fn sixteen_kibibytes_with_the_pattern(multiplier: usize, modulus: usize) -> Vec<u8> {
        (0..SIXTEEN_KIBIBYTES)
            .map(|index| u8::try_from(index * multiplier % modulus).expect("小于 256"))
            .collect()
    }

    /// 版本表第 8 字异或任意一段 16 KiB 的 CRC-32C，就是「这一版前 16 KiB 接那一段」32 KiB 的 CRC-32C：GPU 读者拼读空段最后一槽
    /// （前一半全零 16 KiB、后一半是下一个落点的前 16 KiB）靠这一条；前一半不是全零时同样成立。不是单元槽的落点记 0。
    #[test]
    fn the_shifted_crc_of_a_version_xored_with_the_crc_of_the_next_sixteen_kibibytes_is_the_crc_of_the_thirty_two_kibibytes(
    ) {
        let next = sixteen_kibibytes_with_the_pattern(13, 253);
        for first in [
            vec![0u8; SIXTEEN_KIBIBYTES],
            sixteen_kibibytes_with_the_pattern(7, 251),
        ] {
            let mut thirty_two_kibibytes = first.clone();
            thirty_two_kibibytes.extend_from_slice(&next);
            assert_eq!(
                crc_shifted_past_sixteen_zero_kibibytes(JudgeLocationKind::UnitSlot, &first)
                    ^ crc32_castagnoli(&next),
                crc32_castagnoli(&thirty_two_kibibytes),
                "拼出的 32 KiB CRC 与连续读出的 32 KiB 的 CRC 相同"
            );
        }
        assert_eq!(
            crc_shifted_past_sixteen_zero_kibibytes(
                JudgeLocationKind::JournalRecord,
                &next[..4096]
            ),
            0,
            "journal 记录槽不会被拼读，记 0"
        );
    }
}
