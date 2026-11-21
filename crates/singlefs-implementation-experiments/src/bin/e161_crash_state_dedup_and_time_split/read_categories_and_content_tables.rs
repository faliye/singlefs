//! 读的分类与内容表（G1、G5 的 P1、V2）。

use super::*;

// ───────────────────────────── 读的分类与内容表（G1、G5 P1、V2） ─────────────────────────────

/// 一次读落在哪一类（G5 P1 的四类；G1 的「单元」是单元区里 16384 或 32768 长的读）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum ReadCategory {
    /// 环起点之前：系统配置槽与根槽。
    FixedStructure,
    /// journal 环里。
    JournalRing,
    /// 单元区里、整单元长（16384 或 32768）。
    WholeUnit,
    /// 单元区里、别的长度（头扫描）。
    UnitHeaderScan,
}

pub(super) const READ_CATEGORIES: [ReadCategory; 4] = [
    ReadCategory::FixedStructure,
    ReadCategory::JournalRing,
    ReadCategory::WholeUnit,
    ReadCategory::UnitHeaderScan,
];

impl ReadCategory {
    pub(super) fn of(offset: u64, length: usize) -> Self {
        let length_in_bytes = u64::try_from(length).expect("读长");
        if offset < RING_START_BYTES {
            Self::FixedStructure
        } else if offset < UNIT_AREA_START_BYTES {
            Self::JournalRing
        } else if length_in_bytes == LOCAL_NODE_BYTES || length_in_bytes == LOCAL_DATA_UNIT_BYTES {
            Self::WholeUnit
        } else {
            Self::UnitHeaderScan
        }
    }

    pub(super) fn slot(self) -> usize {
        match self {
            Self::FixedStructure => 0,
            Self::JournalRing => 1,
            Self::WholeUnit => 2,
            Self::UnitHeaderScan => 3,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::FixedStructure => "fixed_structure",
            Self::JournalRing => "journal_ring",
            Self::WholeUnit => "whole_unit",
            Self::UnitHeaderScan => "unit_header_scan",
        }
    }
}

/// 一份内容在哪些读里出现过：`any_pass` 是恢复两遍与 checker 正常那一遍（P1），`checker_normal_pass` 只算 checker 正常那一遍
/// （G1 的 Q1a、Q1e、Q1f）；各记最小序号（轨迹按「新出现」记在序号最小的那个状态所在的区间，8.1）。
pub(super) struct ContentEntry {
    pub(super) bytes: Vec<u8>,
    pub(super) first_ordinal_any_pass: Option<u64>,
    pub(super) first_ordinal_checker_normal_pass: Option<u64>,
}

/// 按摘要分桶、桶里逐字节比的内容表（5.1）；摘要相同而字节不同记一次撞车（V2）。
#[derive(Default)]
pub(super) struct ContentTable {
    pub(super) entries: HashMap<Digest128, ContentEntry>,
    pub(super) collisions: u64,
}

fn minimum_ordinal(current: Option<u64>, candidate: Option<u64>) -> Option<u64> {
    match (current, candidate) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) => Some(left),
        (None, right) => right,
    }
}

impl ContentTable {
    pub(super) fn insert(
        &mut self,
        digest: Digest128,
        bytes: &[u8],
        ordinal: u64,
        is_checker_normal_pass: bool,
    ) {
        let checker_ordinal = is_checker_normal_pass.then_some(ordinal);
        match self.entries.get_mut(&digest) {
            Some(entry) => {
                if entry.bytes != bytes {
                    self.collisions += 1;
                }
                entry.first_ordinal_any_pass =
                    minimum_ordinal(entry.first_ordinal_any_pass, Some(ordinal));
                entry.first_ordinal_checker_normal_pass =
                    minimum_ordinal(entry.first_ordinal_checker_normal_pass, checker_ordinal);
            }
            None => {
                self.entries.insert(
                    digest,
                    ContentEntry {
                        bytes: bytes.to_vec(),
                        first_ordinal_any_pass: Some(ordinal),
                        first_ordinal_checker_normal_pass: checker_ordinal,
                    },
                );
            }
        }
    }

    pub(super) fn absorb(&mut self, other: ContentTable) {
        self.collisions += other.collisions;
        for (digest, entry) in other.entries {
            match self.entries.get_mut(&digest) {
                Some(existing) => {
                    if existing.bytes != entry.bytes {
                        self.collisions += 1;
                    }
                    existing.first_ordinal_any_pass = minimum_ordinal(
                        existing.first_ordinal_any_pass,
                        entry.first_ordinal_any_pass,
                    );
                    existing.first_ordinal_checker_normal_pass = minimum_ordinal(
                        existing.first_ordinal_checker_normal_pass,
                        entry.first_ordinal_checker_normal_pass,
                    );
                }
                None => {
                    self.entries.insert(digest, entry);
                }
            }
        }
    }
}

/// 四类各一张表。
#[derive(Default)]
pub(super) struct ContentTables {
    pub(super) by_category: [ContentTable; 4],
}

impl ContentTables {
    pub(super) fn insert(
        &mut self,
        category: ReadCategory,
        digest: Digest128,
        bytes: &[u8],
        ordinal: u64,
        is_checker_normal_pass: bool,
    ) {
        self.by_category[category.slot()].insert(digest, bytes, ordinal, is_checker_normal_pass);
    }

    pub(super) fn absorb(&mut self, other: ContentTables) {
        for (mine, theirs) in self.by_category.iter_mut().zip(other.by_category) {
            mine.absorb(theirs);
        }
    }

    pub(super) fn collisions(&self) -> u64 {
        self.by_category.iter().map(|table| table.collisions).sum()
    }
}
