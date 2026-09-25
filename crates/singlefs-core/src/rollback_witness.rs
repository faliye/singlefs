//! 回退见证（D23（journal 的角色与格式） 已定项 14「回退见证」，C332（回退实例两个根都读不出时回退被撤销） 的修法，用户 2026-09-24 定）：
//! 管理员回退在系统配置槽里记一张见证表，一个条目记一次回退——新实例代号 N、回退目标 R_old 的实例代号 r_old 与 txg T_old。
//! 根 (i, T) 被条目 (N, r_old, T_old) 抛弃 ⟺ (r_old, T_old) < (i, T)（实例代号为主比）且 i < N；择根先跳过被任一条目抛弃的根。
//!
//! 盘上形态：系统配置槽内偏移 481 起（紧接字段表），条数 1 字节 + 47 个条目位 × 16 字节 = 753 字节定宽，罩在整槽校验和里、越过 512
//! （`singlefs_format::ROLLBACK_WITNESS_*`）。一个池的条数上限 = 根环槽数减 1（R × S − 1，S 读自这个池的系统配置）。
//! 条目按 (N, r_old, T_old) 升序排、不重复；条数之后的条目位全 0。读到别的样子就是这一槽读不出（见证表读不出就是系统配置槽读不出）。

use std::collections::BTreeSet;

use singlefs_format::{
    ROLLBACK_WITNESS_COUNT_BYTES, ROLLBACK_WITNESS_ENTRIES_MAXIMUM, ROLLBACK_WITNESS_ENTRY_BYTES,
    ROLLBACK_WITNESS_TABLE_BYTES, ROOT_RING_REGIONS,
};

use crate::address::{CheckpointTxg, InstanceGeneration};
use crate::root_ring::RootRingSlotsPerRegion;

/// 见证表定宽的条目位数（47）。数组长度要一个编译期的 `usize`，`try_from` 在常量里用不了。
#[allow(
    clippy::cast_possible_truncation,
    reason = "47 装得进任何宽度的 usize；下一句编译期断言钉住没丢值"
)]
const ENTRY_POSITIONS: usize = ROLLBACK_WITNESS_ENTRIES_MAXIMUM as usize;
const _: () = assert!(ENTRY_POSITIONS as u64 == ROLLBACK_WITNESS_ENTRIES_MAXIMUM);

/// 一次回退的见证：回退那一次挂载取的新实例代号，与管理员选的回退目标 R_old。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RollbackWitnessEntry {
    pub new_instance: InstanceGeneration,
    pub rollback_target_instance: InstanceGeneration,
    pub rollback_target_txg: CheckpointTxg,
}

impl RollbackWitnessEntry {
    /// 条目位全 0 的那个值（没用上的条目位写它）。
    const UNUSED_POSITION: Self = Self {
        new_instance: InstanceGeneration(0),
        rollback_target_instance: InstanceGeneration(0),
        rollback_target_txg: CheckpointTxg(0),
    };

    /// (实例代号, txg) 这一处（一条根，或一条 journal 记录的 (实例代号, checkpoint_txg)）被这次回退抛弃：
    /// (r_old, T_old) < (i, T)，按实例代号为主比，且 i < N。
    #[must_use]
    pub fn abandons(&self, instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> bool {
        (self.rollback_target_instance, self.rollback_target_txg) < (instance, checkpoint_txg)
            && instance < self.new_instance
    }
}

/// 一个池的见证表条数上限：根环槽数减 1（R × S − 1），S 是这个池系统配置里的每区槽数。
#[must_use]
pub fn rollback_witness_capacity(root_ring_slots_per_region: RootRingSlotsPerRegion) -> usize {
    usize::try_from(ROOT_RING_REGIONS * root_ring_slots_per_region.count() - 1)
        .expect("R × S − 1 至多 47")
}

/// 一张见证表：按 (N, r_old, T_old) 升序、不重复，条数不超过定宽的 47。`Copy`：它是系统配置槽内容的一部分，跟着系统配置按值传。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackWitnessTable {
    positions: [RollbackWitnessEntry; ENTRY_POSITIONS],
    count: usize,
}

/// 见证表装不下：条目数越过了这个池的上限（`capacity`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackWitnessTableFull {
    pub entries: usize,
    pub capacity: usize,
}

impl RollbackWitnessTable {
    /// 一条条目都没有的表：mkfs 写它，从没回退过的池一直是它（盘上那 753 字节全 0）。
    pub const EMPTY: Self = Self {
        positions: [RollbackWitnessEntry::UNUSED_POSITION; ENTRY_POSITIONS],
        count: 0,
    };

    /// 由一组条目装一张表（排序、去重）。
    ///
    /// # Errors
    /// 去重之后多于 `capacity` 条（`capacity` 自己不超过定宽的 47）。
    pub fn of_entries(
        entries: impl IntoIterator<Item = RollbackWitnessEntry>,
        capacity: usize,
    ) -> Result<Self, RollbackWitnessTableFull> {
        let sorted: BTreeSet<RollbackWitnessEntry> = entries.into_iter().collect();
        let capacity = capacity.min(ENTRY_POSITIONS);
        if sorted.len() > capacity {
            return Err(RollbackWitnessTableFull {
                entries: sorted.len(),
                capacity,
            });
        }
        let mut table = Self::EMPTY;
        table.count = sorted.len();
        for (position, entry) in sorted.into_iter().enumerate() {
            table.positions[position] = entry;
        }
        Ok(table)
    }

    #[must_use]
    pub fn entries(&self) -> &[RollbackWitnessEntry] {
        &self.positions[..self.count]
    }

    /// (实例代号, txg) 这一处被表里任一条目抛弃。
    #[must_use]
    pub fn abandons(&self, instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> bool {
        self.entries()
            .iter()
            .any(|entry| entry.abandons(instance, checkpoint_txg))
    }

    /// 盘上的 753 字节：条数 1 字节，之后 47 个条目位，每位 新实例代号 4 + R_old 实例代号 4 + R_old txg 8（小端），没用上的位全 0。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes =
            Vec::with_capacity(usize::try_from(ROLLBACK_WITNESS_TABLE_BYTES).expect("753"));
        bytes.push(u8::try_from(self.count).expect("条数至多 47"));
        for entry in &self.positions {
            bytes.extend_from_slice(&entry.new_instance.0.to_le_bytes());
            bytes.extend_from_slice(&entry.rollback_target_instance.0.to_le_bytes());
            bytes.extend_from_slice(&entry.rollback_target_txg.0.to_le_bytes());
        }
        bytes
    }

    /// 读者：753 字节按上面的样子解。条数不超过这个池的上限（`capacity`）、条目按 (N, r_old, T_old) 严格升序、每一条 r_old < N、
    /// 条数之后的条目位全 0——任一不满足就是 `None`（这一槽的见证表读不出，等于这一槽读不出）。
    #[must_use]
    pub fn parse(bytes: &[u8], capacity: usize) -> Option<Self> {
        if bytes.len() < usize::try_from(ROLLBACK_WITNESS_TABLE_BYTES).expect("753") {
            return None;
        }
        let count = usize::from(bytes[0]);
        if count > capacity.min(ENTRY_POSITIONS) {
            return None;
        }
        let count_bytes = usize::try_from(ROLLBACK_WITNESS_COUNT_BYTES).expect("1");
        let entry_bytes = usize::try_from(ROLLBACK_WITNESS_ENTRY_BYTES).expect("16");
        let mut table = Self::EMPTY;
        for position in 0..ENTRY_POSITIONS {
            let start = count_bytes + position * entry_bytes;
            let field =
                |offset: usize, width: usize| &bytes[start + offset..start + offset + width];
            let entry = RollbackWitnessEntry {
                new_instance: InstanceGeneration(u32::from_le_bytes(
                    field(0, 4).try_into().expect("4 字节"),
                )),
                rollback_target_instance: InstanceGeneration(u32::from_le_bytes(
                    field(4, 4).try_into().expect("4 字节"),
                )),
                rollback_target_txg: CheckpointTxg(u64::from_le_bytes(
                    field(8, 8).try_into().expect("8 字节"),
                )),
            };
            if position < count {
                let follows_the_previous = position == 0 || table.positions[position - 1] < entry;
                if entry.rollback_target_instance >= entry.new_instance || !follows_the_previous {
                    return None;
                }
                table.positions[position] = entry;
            } else if entry != RollbackWitnessEntry::UNUSED_POSITION {
                return None;
            }
        }
        table.count = count;
        Some(table)
    }
}

/// 一个池此刻的见证：各盘择到的那一槽（两槽里自证过、世代号最大的）里的见证表取并集。
/// 各盘在一次轮换写的中途崩了会一新一旧，取并集才不丢那一条；各盘两槽里旧的那一槽不并进来——挂载删掉的条目留在旧槽里，
/// 并进来就删不掉（它只在择到的那一槽读不出、退回旧槽时回来，那时它已经抛弃不了环里的任何根，见 `mount` 的删除规则）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RollbackWitness {
    entries: BTreeSet<RollbackWitnessEntry>,
}

impl RollbackWitness {
    /// 把几张表并进来。
    #[must_use]
    pub fn of_tables<'table>(
        tables: impl IntoIterator<Item = &'table RollbackWitnessTable>,
    ) -> Self {
        Self {
            entries: tables
                .into_iter()
                .flat_map(|table| table.entries().iter().copied())
                .collect(),
        }
    }

    #[must_use]
    pub fn entries(&self) -> Vec<RollbackWitnessEntry> {
        self.entries.iter().copied().collect()
    }

    /// (实例代号, txg) 这一处被任一条目抛弃。
    #[must_use]
    pub fn abandons(&self, instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.abandons(instance, checkpoint_txg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(new_instance: u32, target_instance: u32, target_txg: u64) -> RollbackWitnessEntry {
        RollbackWitnessEntry {
            new_instance: InstanceGeneration(new_instance),
            rollback_target_instance: InstanceGeneration(target_instance),
            rollback_target_txg: CheckpointTxg(target_txg),
        }
    }

    /// 已定项 14「回退见证」的判法：(r_old, T_old) < (i, T)（实例代号为主比）且 i < N。回退到 (1, 3)、新实例 4：
    /// 实例 1 里 txg 3 之后的根、实例 2 与 3 的全部根被抛弃；R_old 自己、它之前的根、新实例及之后的根不被抛弃。
    #[test]
    fn an_entry_abandons_exactly_the_roots_after_the_rollback_target_and_before_the_new_instance() {
        let rollback = entry(4, 1, 3);
        for (instance, txg, abandoned) in [
            (1, 2, false),
            (1, 3, false),
            (1, 4, true),
            (1, 9, true),
            (2, 1, true),
            (3, 12, true),
            (4, 5, false),
            (5, 1, false),
            (0, 0, false),
        ] {
            assert_eq!(
                rollback.abandons(InstanceGeneration(instance), CheckpointTxg(txg)),
                abandoned,
                "({instance}, {txg})"
            );
        }
    }

    /// 盘上 753 字节来回一趟不变；条目位按 (N, r_old, T_old) 升序排、去重；没用上的位全 0。
    #[test]
    fn a_table_round_trips_through_its_753_bytes_sorted_and_deduplicated() {
        let table = RollbackWitnessTable::of_entries(
            [entry(7, 2, 10), entry(4, 1, 3), entry(7, 2, 10)],
            23,
        )
        .expect("两条装得下");
        assert_eq!(table.entries(), &[entry(4, 1, 3), entry(7, 2, 10)]);
        let bytes = table.to_bytes();
        assert_eq!(bytes.len(), 753);
        assert_eq!(bytes[0], 2);
        assert!(
            bytes[33..].iter().all(|byte| *byte == 0),
            "条数之后的条目位全 0"
        );
        assert_eq!(RollbackWitnessTable::parse(&bytes, 23), Some(table));
        assert_eq!(
            RollbackWitnessTable::parse(&RollbackWitnessTable::EMPTY.to_bytes(), 23),
            Some(RollbackWitnessTable::EMPTY),
            "全 0 的 753 字节就是空表"
        );
    }

    /// 读者的四格都当这一槽的见证表读不出：条数超过这个池的上限、条目不升序、r_old 不小于 N、条数之后的条目位不是 0。
    #[test]
    fn a_count_beyond_the_capacity_an_unsorted_entry_a_target_not_below_the_new_instance_or_a_nonzero_unused_position_is_unreadable(
    ) {
        let two = RollbackWitnessTable::of_entries([entry(4, 1, 3), entry(7, 2, 10)], 23)
            .expect("两条")
            .to_bytes();
        assert!(
            RollbackWitnessTable::parse(&two, 1).is_none(),
            "条数 2 超过上限 1"
        );
        let mut unsorted = two.clone();
        unsorted[1..17].copy_from_slice(&two[17..33]);
        unsorted[17..33].copy_from_slice(&two[1..17]);
        assert!(
            RollbackWitnessTable::parse(&unsorted, 23).is_none(),
            "不升序"
        );
        let mut target_not_below = two.clone();
        target_not_below[5..9].copy_from_slice(&4u32.to_le_bytes());
        assert!(
            RollbackWitnessTable::parse(&target_not_below, 23).is_none(),
            "r_old 不小于 N"
        );
        let mut trailing = two;
        trailing[40] = 1;
        assert!(
            RollbackWitnessTable::parse(&trailing, 23).is_none(),
            "条数之后的条目位不是 0"
        );
    }

    /// 装不下就报条数与上限，不截断。
    #[test]
    fn more_entries_than_the_capacity_are_refused_not_truncated() {
        assert_eq!(
            RollbackWitnessTable::of_entries([entry(4, 1, 3), entry(7, 2, 10)], 1),
            Err(RollbackWitnessTableFull {
                entries: 2,
                capacity: 1
            })
        );
    }
}
