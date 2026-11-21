//! 日志记录的名字与字段标签、系统配置的写、暖机、墓碑与退役记录的预付与已承诺预留。

use e7_index_bench::usize_from_u64;
use super::*;

/// D23（journal 的角色与格式） 已定项 18：记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`。
pub(super) fn journal_record_offset(counter: JournalCounter) -> DeviceOffset {
    DeviceOffset(JOURNAL_START_SLOT * SLOT_BYTES + ((counter.0 - 1) % JOURNAL_RING_SLOTS) * JOURNAL_RECORD_BYTES)
}

/// E142 第十四次跑（第三节 3.2 第 3399–3425 行）：给一个盘上绝对偏移，判它落在 w1 / w4 / 本次事务自己那条
/// journal 记录哪一条、记录内偏移是多少。三条记录都在 mkfs 之后紧接的三个 journal 计数器（1、2、3）上，
/// 与几何点无关，可以直接算，不用查 StructureCatalog。
///
/// E142 第十七次跑（重跑登记第三节 3.2、问题单第 8 行）：第三条记录的角色名不再写死 `"t9"`——那是分配记录树
/// 按位置寻址之前的旧编号（那时新池新建文件只有 8 个单元，journal 记录恰好排在 t9）；今天的写清单有 12 个单元
/// （`name=write_list` 的 `units=` 与 `name=role_labels` 都钉着 t1..t12），这条记录该叫 `t{单元数+1}`，
/// 由调用方按当次的写清单现算、传进来（R18：角色名从写清单现取），不在这里写字面。
pub(super) fn journal_record_name_for_offset(offset: u64, this_transaction_journal_label: &str) -> Option<(String, u64)> {
    for (label, counter) in [("w1", 1u64), ("w4", 2u64)] {
        let base = journal_record_offset(JournalCounter(counter)).0;
        if offset >= base && offset < base + JOURNAL_RECORD_BYTES {
            return Some((label.to_string(), offset - base));
        }
    }
    let base = journal_record_offset(JournalCounter(3)).0;
    if offset >= base && offset < base + JOURNAL_RECORD_BYTES {
        return Some((this_transaction_journal_label.to_string(), offset - base));
    }
    None
}

/// 主臂三条 journal 记录（暖机两条 + 本次事务自己那条）各自的 (角色名, journal 计数器) ——
/// 唯一定义这个对应关系的地方；`main()` 与角色名一致性单测都调它，不各自写一份字面
/// （M156 的会红检查：把第三项的角色名退回字面 `t9`）。
pub(super) fn journal_header_record_labels(this_transaction_journal_label: &str) -> [(String, u64); 3] {
    [("w1".to_string(), 1u64), ("w4".to_string(), 2u64), (this_transaction_journal_label.to_string(), 3u64)]
}

/// H/F 相减、P3 全盘相减两处共用的字段标签（记录内偏移 → 字段名，第七节 A2 的偏移表）。
pub(super) fn journal_header_field_tag(offset_in_record: u64) -> &'static str {
    match offset_in_record {
        JOURNAL_RECORD_FLAGS_OFFSET_U64 => "record_flags",
        46..=77 => "header_checksum",
        87..=90 => "ordinal_within_publish",
        91..=94 => "back_chain",
        _ => "other",
    }
}
pub(super) const JOURNAL_RECORD_FLAGS_OFFSET_U64: u64 = JOURNAL_RECORD_FLAGS_OFFSET as u64;

/// 发布之后那次系统配置槽写的世代号与落点（D22 已定项 16）：取号那次是 2，之后每次发布 +1，槽 = 世代号 mod 2。
pub(super) fn system_configuration_write_for_publish(checkpoint_txg: CheckpointTxg) -> (u64, usize) {
    let generation = checkpoint_txg.0 + SYSTEM_CONFIGURATION_GENERATION_AT_INSTANCE_ACQUISITION;
    (generation, (generation % 2) as usize)
}

/// 暖机（D16 已定项 8）：每次空发布照 D16 已定项 7 的顺序——屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障；
/// 空记录不点名任何单元、事务号 0（D23 已定项 19 ①：0 保留给不承载事务的记录）、提交标记 1，
/// 新根段照 mkfs 的根，根记录只改 checkpoint_txg。返回最后一条记录的字节，下一条记录的反向链要用它。
pub(super) fn warm_up(pool: &mut RecordingPool, parameters: &PoolParameters, genesis: &MkfsOutput, instance: InstanceGeneration) -> (Vec<RootRecord>, Option<Vec<u8>>) {
    let mut roots = Vec::new();
    let mut previous_record_bytes: Option<Vec<u8>> = None;
    for txg_number in 1..=WARM_UP_EMPTY_PUBLISHES {
        let txg = CheckpointTxg(txg_number);
        if parameters.barriers == BarrierPolicy::Settled {
            pool.barrier();
        }
        let counter = JournalCounter(txg_number);
        let record = JournalRecord {
            instance,
            counter,
            checkpoint_txg: txg,
            transaction: TransactionNumber(0),
            is_commit: true,
            // 这次发布只有这一条记录（暖机的空发布不切分）：本次发布内序号恒 1（已定项 4），
            // 标志按写者开关写（臂 F 0x01、臂 H 0x00，E142 第十四次跑第五节 5.1）。
            ordinal_within_publish: 1,
            record_flags: parameters.last_of_publish_flag,
            back_chain: previous_record_bytes.as_deref().map_or(0, journal_back_chain),
            fsid: unit_fsid(&parameters.fsid),
            new_tree_table: genesis.root.tree_table,
            new_mapping_root: genesis.root.mapping_root,
            new_tree_identifier_watermark: genesis.root.tree_identifier_watermark,
            new_rollback_floor: genesis.root.rollback_floor,
            // 新根段后三项取这次发布写的那条根的同名字段（D23 已定项 15「由记录重建那次发布的根」）：
            // 暖机的根照 mkfs 根（实例表指针照 mkfs、分配记录树根零、卸载记号 0，layout/01 第六节 w1 / w4）。
            new_instance_table: genesis.root.instance_table,
            new_allocation_record_tree_root: genesis.root.allocation_record_tree_root,
            new_unmount_marker: false,
            named: Vec::new(),
        };
        let record_bytes = record.to_bytes();
        for device in parameters.devices() {
            pool.write(device, journal_record_offset(counter), &record_bytes, StepKind::JournalRecord);
        }
        previous_record_bytes = Some(record_bytes);
        if parameters.barriers == BarrierPolicy::Settled {
            pool.barrier();
        }
        let root = RootRecord {
            fsid: genesis.root.fsid,
            unmount_marker: false, // 暖机的空发布不是正常卸载（D22 已定项 7）
            instance,
            checkpoint_txg: txg,
            tree_table: genesis.root.tree_table,
            tree_identifier_watermark: genesis.root.tree_identifier_watermark,
            rollback_floor: genesis.root.rollback_floor,
            instance_table: genesis.root.instance_table,
            mapping_root: genesis.root.mapping_root,
            // C512：零单元发布照抄上一版。
            allocation_record_tree_root: genesis.root.allocation_record_tree_root,
        };
        let (region, ring_slot) = ring_target_for_publish(txg);
        pool.write(parameters.region_devices[usize_from_u64(region)], ring_slot_offset(region, ring_slot), &root.to_slot(), StepKind::RootRecordFua);
        let (slot_generation, slot_index) = system_configuration_write_for_publish(txg);
        for device in parameters.devices() {
            let system_configuration = SystemConfiguration {
                fsid: parameters.fsid,
                this_device: device,
                device_count: u32::try_from(parameters.device_count).expect("设备数"),
                slot_generation,
                region_devices: parameters.region_devices,
                journal_tail: txg_number,
                journal_instance: instance,
                rollback_floor: CheckpointTxg(0), // 暖机的空发布没有抬 F（D16 已定项 1）
            };
            pool.write(device, DeviceOffset(SYSTEM_CONFIGURATION_SLOT_OFFSETS[slot_index]), &system_configuration.to_slot(), StepKind::SystemConfigurationSlot);
        }
        // ⑧ 发布尾那道屏障（D16 已定项 7「根槽（FUA）→ 系统配置槽 → 屏障」，登记 R43）：每次空发布各一道。
        if parameters.issues_trailing_publish_barrier() {
            pool.barrier();
        }
        roots.push(root);
    }
    (roots, previous_record_bytes)
}

/// D18（块里携带什么信息） 已定项 24：第一版墓碑记录 40 字节，装进码 3 容器一个容器 815 条
/// （(32768 − 107 − 29) ÷ 40 向下取整，I-1.7）；D3（空间分配） 已定项 2 的 41 由它定。
const TOMBSTONE_RECORDS_PER_CONTAINER: u64 = 815;
/// D3（空间分配） 已定项 2：H 是现行那一版墓碑树的内部层数，「零根与只有根时 0」。
const TOMBSTONE_TREE_INTERNAL_LEVELS_OF_A_ZERO_ROOT: u64 = 0;
/// D8（核心索引结构） 已定项 6 的偏移表：mode 24、uid 28、gid 32 各 4 字节，nlink 紧接在 36（`layout/01` 第四节「偏移 24：mode / uid / gid / nlink 16」）。
const INODE_RECORD_LINK_COUNT_OFFSET: usize = 36;

/// `layout/03`「退役容器」那一行：退役记录 34 字节，一个 32 KiB 码 3 容器（含 29 字节预留位的头 136）装 ⌊(32768 − 136) ÷ 34⌋ = 959 条；
/// D3（空间分配） 已定项 2 的 35 由它定。字面照登记值写，旁边一条单测核这道算术。
const RETIREMENT_RECORDS_PER_CONTAINER: u64 = 959;
/// D3（空间分配） 已定项 2：H_退役 是现行那一版退役树的内部层数，「零根与只有根时 0」。
const RETIREMENT_TREE_INTERNAL_LEVELS_OF_A_ZERO_ROOT: u64 = 0;

/// D3（空间分配） 已定项 2：每份副本、每个活对象摊的墓碑预付，41 = ⌈2 × 16384 ÷ 815⌉，先取整再乘 n_live。
fn tombstone_prepayment_bytes_per_live_object() -> u64 {
    (2 * SLOT_BYTES).div_ceil(TOMBSTONE_RECORDS_PER_CONTAINER)
}

/// D3（空间分配） 已定项 2：每份副本、每条 inode 记录摊的退役记录预付，35 = ⌈32768 ÷ 959⌉，先取整再乘 n_记录。
fn retirement_prepayment_bytes_per_inode_record() -> u64 {
    (2 * SLOT_BYTES).div_ceil(RETIREMENT_RECORDS_PER_CONTAINER)
}

/// D3（空间分配） 已定项 2 条文的式子：标签 6（已承诺预留）=
/// 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)，单位字节、按副本之和记；
/// 树表里没有 `deleted_inodes` 条目的版本取 0。
pub(super) fn committed_reservation_bytes(
    replica_count: u64,
    tombstone_tree_internal_levels: u64,
    retirement_tree_internal_levels: u64,
    live_object_count: u64,
    inode_record_count: u64,
    tree_table_has_deleted_inodes_entry: bool,
) -> u64 {
    if !tree_table_has_deleted_inodes_entry {
        return 0;
    }
    let tombstone_resident_slots = 2 + tombstone_tree_internal_levels + 1;
    let retirement_resident_slots = 2 + retirement_tree_internal_levels + 1;
    let resident_slots = tombstone_resident_slots + retirement_resident_slots;
    replica_count
        * (resident_slots * SLOT_BYTES
            + tombstone_prepayment_bytes_per_live_object() * live_object_count
            + retirement_prepayment_bytes_per_inode_record() * inode_record_count)
}

/// H：从墓碑树的根指针现读。新池新建文件的墓碑树恒为零根（D18 已定项 24 第 6 条）；非零根时要从根节点码 2 头现读层数，
/// 不在 E142 第二十二次跑的射程里，走到就停。
pub(super) fn tombstone_tree_internal_levels(tombstone_root: &NodePointer) -> u64 {
    if tombstone_root.is_empty_root() {
        return TOMBSTONE_TREE_INTERNAL_LEVELS_OF_A_ZERO_ROOT;
    }
    panic!("这条流里墓碑树恒为零根；非零根时 H 要从根节点码 2 头现读（D3 已定项 2），装置不写这一支")
}

/// H_退役：从树表里退役树那一条的根指针现读。新池新建文件的退役树恒为零根（D18 已定项 24 第 6 条）；
/// 非零根时要从根节点码 2 头现读层数，同墓碑树那一支不在射程里，走到就停。
pub(super) fn retirement_tree_internal_levels(retirement_root: &NodePointer) -> u64 {
    if retirement_root.is_empty_root() {
        return RETIREMENT_TREE_INTERNAL_LEVELS_OF_A_ZERO_ROOT;
    }
    panic!("这条流里退役树恒为零根；非零根时 H_退役 要从根节点码 2 头现读（D3 已定项 2），装置不写这一支")
}

fn inode_record_link_count(record_bytes: &[u8]) -> u32 {
    let link_count_bytes: [u8; 4] = record_bytes[INODE_RECORD_LINK_COUNT_OFFSET..INODE_RECORD_LINK_COUNT_OFFSET + 4].try_into().expect("切了 4 字节");
    u32::from_le_bytes(link_count_bytes)
}

/// n_live：这一版 inode 树叶容器里 nlink ≥ 1 的记录数（D3 已定项 2）。
pub(super) fn live_object_count_of_inode_records(inode_record_bytes: &[Vec<u8>]) -> u64 {
    inode_record_bytes.iter().filter(|record| inode_record_link_count(record) >= 1).count() as u64
}

/// n_记录：这一版 inode 树叶容器里全部记录数，含 nlink = 0 的（D3 已定项 2：退役项数 n_记录，不数 n_live）。
pub(super) fn record_count_of_inode_records(inode_record_bytes: &[Vec<u8>]) -> u64 {
    u64::try_from(inode_record_bytes.len()).expect("记录数装得进 u64")
}

#[cfg(test)]
#[path = "journal_labels_warm_up_and_reservation_tests.rs"]
mod tests;
