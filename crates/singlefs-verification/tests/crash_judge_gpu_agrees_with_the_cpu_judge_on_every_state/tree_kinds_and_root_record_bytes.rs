//! 树表种类重复与缺项（I-9.16 ③ ④）、根记录 MAC 与根记录之后的字节、回收门槛与钉住的根环槽、带树一版根记录里的分配记录树根。

use super::*;

// ───────────── I-9.16 ③ ④、根槽 437..466、带树一版的分配记录树根、码 2 节点指针头部的 extent 偏移 ─────────────

/// 树表条目里树根指针（86 字节的码 2 指针）的偏移（种类 2、flags 2 之后）。
pub(super) const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;

/// 稀疏旁表与 deadlist 的种类码（D18（块里携带什么信息） 已定项 24 第 6 条那一组）：第一版两棵树都不建根，树表条目的根指针全零，走读与恢复都不读。
pub(super) const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;

pub(super) const TREE_KIND_DEADLIST: u16 = 8;

/// 一个没登记的种类码：走读与恢复都不认它、不读它的根（退役树与只读快照的两种条目
/// （快照 inode 根、快照 extent 根），改用 16，与 harness 坏镜像里的 `TREE_KIND_NOT_REGISTERED` 同一个码）。
pub(super) const UNREGISTERED_TREE_KIND: u16 = 16;

/// 分配记录树的种类码（D8（核心索引结构） 已定项 9 的登记表）。
const TREE_KIND_ALLOCATION_RECORD: u16 = 3;

/// 根记录里树表指针的第 0 条位置条目（盘 4 + 槽 6）的偏移：树表指针在 36，位置条目在指针偏移 50。
const ROOT_RECORD_TREE_TABLE_POINTER_FIRST_LOCATION_OFFSET: usize = 86;

/// 根记录里「树表 0 条那一版的分配记录树根」指针的偏移（D22（单元原子性怎么合成） 已定项 7 的字段表）。
const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET: usize = 342;

/// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节的最后一个字节（437..466 的末字节）。
const ROOT_RECORD_LAST_MAC_BYTE_OFFSET: usize = 465;

/// 发布那张树表里种类是 `from_kind` 的那条（恰好一条、根指针全零）的种类改成 `to_kind`、重封到根。树 ID 与条目次序不动。
fn record_new_pool_with_a_tree_table_entry_kind_rewritten(
    tag: &str,
    name: &'static str,
    from_kind: u16,
    to_kind: u16,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(tag, name, &is_a_tree_table_unit, false, &|bytes| {
        let entry = only_tree_table_entry_of_kind(bytes, from_kind);
        let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
        assert_eq!(
            bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
            [0u8; NODE_POINTER_BYTES],
            "种类 {from_kind} 那条的根指针本来全零（这棵树第一版不建根，走读与恢复都不读它）"
        );
        let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
        bytes[kind..kind + 2].copy_from_slice(&to_kind.to_le_bytes());
    })
}

/// I-9.16 ③（每种至多一条）：deadlist 那条的种类改成稀疏旁表的 7——种类 7 两条、种类 8 一条都没有。① ② 照旧成立（② 按每种第一次出现的那条）。
/// core 的只读恢复在读任何一棵树之前拒（`tree_table_entries_each_kind_at_most_once`），GPU 恢复内核种类重复那一支同样拒。
fn record_new_pool_with_two_tree_table_entries_of_the_sparse_side_table_kind(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_tree_table_entry_kind_rewritten(
        tag,
        "new-pool-tree-table-two-entries-of-the-sparse-side-table-kind",
        TREE_KIND_DEADLIST,
        TREE_KIND_SPARSE_SIDE_TABLE,
    )
}

/// 树表非空时进树表的种类数（I-9.16 ④ 那一串：extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、
/// logged_ops、退役树），新池新建文件那一版的树表就是这十二种各一条。
const TREE_KINDS_OF_EVERY_NON_EMPTY_TREE_TABLE: usize = 12;

/// 根记录里树 ID 水位那 8 字节的偏移：树表指针（36 起、86 字节）之后（`singlefs_core::root_record` 的 `to_slot` 的字段次序）。
const ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET: usize = 122;

/// journal 记录新根段里树 ID 水位那 8 字节的偏移：新根段起点之后的树表指针、映射根指针各 86 字节之后。
const JOURNAL_NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK_OFFSET: usize =
    JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 2 * NODE_POINTER_BYTES;

/// 码 2 节点头里 key 区间上界的偏移（D18（块里携带什么信息） 已定项 18 的偏移表「52 key 区间 2k」：下界在 52、上界在 52 + key 宽），
/// 树表的 key 是树 ID、key 宽 8。
const TREE_TABLE_UNIT_LARGEST_KEY_OFFSET: usize = 52 + 8;

/// I-9.16 ③ 单独不成立的那一形（C675）：发布那张树表十二种各一条，末尾再接一条种类 7（稀疏旁表）——照抄种类 7 那一条
/// （根指针全零、头 ID 0、诞生 txg 照旧），只把树 ID 换成下一个可用号（第一次发布之后的水位 65549）；条目数加 1、声明长度加一条条目宽 200
/// （不加的话声明长度之后的补齐非 0，I-2.3 红、这一版树表读不出，I-9.16 判不到它）、头里 key 区间的上界跟到 65549、单元重封到根。① 盘上次序与 ② 发号次序照旧成立（② 按每种第一次出现的那条），④ 十二种都在，只剩 ③ 种类 7 两条。
/// 点 ③ 的另两条流（[`record_new_pool_with_two_tree_table_entries_of_the_sparse_side_table_kind`] 与
/// [`record_new_pool_with_two_tree_table_entries_of_a_kind_not_below_thirty_two`]）重复一种的同时都少了一种，④ 也红。
/// 发布那条根与它那几份 journal 记录新根段里的树 ID 水位一起从 65549 抬到 65550、重封：不抬的话新那条的树 ID 不低于水位，
/// I-7.8 跟着红；两处一起抬，I-8.10（记录新根段与同代根相同）照旧成立。
/// core 的只读恢复在读任何一棵树之前拒（`tree_table_entries_each_kind_at_most_once`），GPU 恢复内核种类重复那一支同样拒。
fn record_new_pool_with_every_tree_kind_and_one_more_entry_of_the_sparse_side_table_kind(
    tag: &str,
) -> RecordedFlow {
    let first_entry_offset =
        usize::try_from(singlefs_format::TREE_TABLE_UNIT_INDEX_NODE_HEADER_BYTES).expect("131");
    let entry_bytes = usize::try_from(singlefs_format::TREE_TABLE_ENTRY_BYTES).expect("200");
    let appended_tree_identifier = singlefs_format::TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH;
    let raised_watermark = appended_tree_identifier + 1;
    fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"))
    }
    let mut flow = record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-tree-table-every-kind-and-one-more-sparse-side-table-entry",
        &is_a_tree_table_unit,
        false,
        &|bytes| {
            let entry_count = entry_count_of_the_tree_table_read_by_its_key_width(bytes);
            let kinds: std::collections::BTreeSet<u16> = (0..entry_count)
                .map(|entry| {
                    let kind =
                        first_entry_offset + entry * entry_bytes + TREE_TABLE_ENTRY_KIND_OFFSET;
                    u16::from_le_bytes(bytes[kind..kind + 2].try_into().expect("2"))
                })
                .collect();
            assert_eq!(
                (entry_count, kinds.len()),
                (
                    TREE_KINDS_OF_EVERY_NON_EMPTY_TREE_TABLE,
                    TREE_KINDS_OF_EVERY_NON_EMPTY_TREE_TABLE
                ),
                "发布那张树表是十二种各一条（种类 {kinds:?}）"
            );
            let last_entry = first_entry_offset + (entry_count - 1) * entry_bytes;
            assert_eq!(
                read_u64_at(bytes, last_entry),
                appended_tree_identifier - 1,
                "最后一条是发号次序上最后发的那棵树（退役树），树 ID 比第一次发布之后的水位小 1"
            );
            assert_eq!(
                read_u64_at(bytes, TREE_TABLE_UNIT_LARGEST_KEY_OFFSET),
                appended_tree_identifier - 1,
                "树表头里 key 区间的上界是最后一条的树 ID"
            );
            let appended_entry = last_entry + entry_bytes;
            assert!(
                bytes[appended_entry..appended_entry + entry_bytes]
                    .iter()
                    .all(|byte| *byte == 0),
                "最后一条之后那 200 字节还空着"
            );
            let sparse_side_table_entry =
                only_tree_table_entry_of_kind(bytes, TREE_KIND_SPARSE_SIDE_TABLE);
            let root_pointer = sparse_side_table_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
            assert_eq!(
                bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
                [0u8; NODE_POINTER_BYTES],
                "种类 {TREE_KIND_SPARSE_SIDE_TABLE} 那条的根指针本来全零（这棵树第一版不建根，走读与恢复都不读它）"
            );
            let copied_entry =
                bytes[sparse_side_table_entry..sparse_side_table_entry + entry_bytes].to_vec();
            bytes[appended_entry..appended_entry + entry_bytes].copy_from_slice(&copied_entry);
            bytes[appended_entry..appended_entry + 8]
                .copy_from_slice(&appended_tree_identifier.to_le_bytes());
            bytes[TREE_TABLE_UNIT_LARGEST_KEY_OFFSET..TREE_TABLE_UNIT_LARGEST_KEY_OFFSET + 8]
                .copy_from_slice(&appended_tree_identifier.to_le_bytes());
            let entry_count_offset = 86 + 2 * usize::from(index_node_key_width(bytes)) - 4;
            bytes[entry_count_offset..entry_count_offset + 2].copy_from_slice(
                &u16::try_from(entry_count + 1)
                    .expect("十三条装得进 2 字节")
                    .to_le_bytes(),
            );
            let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
            assert_eq!(
                usize::from(declared_length),
                entry_count * entry_bytes,
                "树表单元的声明长度是条目数 × 200"
            );
            bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
                &(declared_length + u16::try_from(entry_bytes).expect("200")).to_le_bytes(),
            );
        },
    );
    let (publish_instance, publish_checkpoint_txg) = publish_root_identity(&flow);
    let publish_roots: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::RootRecordFua
                && write.bytes().is_some_and(|bytes| {
                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == publish_instance
                        && u64::from_le_bytes(bytes[28..36].try_into().expect("8"))
                            == publish_checkpoint_txg
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !publish_roots.is_empty(),
        "流 {} 里有发布那条根（实例 {publish_instance}、txg {publish_checkpoint_txg}）的根槽写",
        flow.name
    );
    let raise_the_watermark_at = |offset: usize| {
        move |bytes: &mut [u8]| {
            assert_eq!(
                read_u64_at(bytes, offset),
                appended_tree_identifier,
                "发布那一版的树 ID 水位是第一次发布之后的 65549"
            );
            bytes[offset..offset + 8].copy_from_slice(&raised_watermark.to_le_bytes());
        }
    };
    for publish_root in publish_roots {
        rewrite_root_record_and_reseal(
            &mut flow,
            publish_root,
            &raise_the_watermark_at(ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET),
        );
    }
    rewrite_every_journal_record_of_the_publish(
        &mut flow,
        &raise_the_watermark_at(JOURNAL_NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK_OFFSET),
    );
    flow
}

/// I-9.16 ④（树表非空时十一种各恰好一条）：deadlist 那条的种类改成没登记的码（[`UNREGISTERED_TREE_KIND`]）——没有一种重复，deadlist 一条都没有。
/// core 的只读恢复照常读到文件（只要种类 1–4 四棵有根的树），可写挂载重建这一版时拒「树表里没有那棵树」；GPU 恢复内核也照常读到。
fn record_new_pool_with_no_tree_table_entry_of_the_deadlist_kind(tag: &str) -> RecordedFlow {
    record_new_pool_with_a_tree_table_entry_kind_rewritten(
        tag,
        "new-pool-tree-table-without-the-deadlist-kind",
        TREE_KIND_DEADLIST,
        UNREGISTERED_TREE_KIND,
    )
}

/// I-3.20 (b) 判不了那一形：发布那张树表里 `deleted_inodes` 那条（队列空，根指针全零）的种类改成没登记的码（[`UNREGISTERED_TREE_KIND`]）、重封到根。
/// 树表非空而一条种类 9 都没有（I-9.16 ④ 判它），已承诺预留照写者按闭式记的不动。
fn record_new_pool_with_no_tree_table_entry_of_the_deleted_inodes_kind(tag: &str) -> RecordedFlow {
    record_new_pool_with_a_tree_table_entry_kind_rewritten(
        tag,
        "new-pool-tree-table-without-the-deleted-inodes-kind",
        TREE_KIND_DELETED_INODES,
        UNREGISTERED_TREE_KIND,
    )
}

/// I-3.16 那道守（最新根那一版一棵分配记录树都没有，按记录口径的全空聚簇段数数不出，记射程里没判、没有违例时整条报不适用）走得到的形：发布那张树表里分配记录树那一条
/// （种类 3，带根）的种类改成没登记的码（[`UNREGISTERED_TREE_KIND`]）、重封到根。树 ID、根指针与条目次序不动，树表就缺了种类 3（I-9.16 ④ 判它）。
/// 分配记录树的节点照旧有中央映射条目，走读经映射条目把它们记进引用（`walk.rs` 的 `walk_central_mapping_entries`），I-3.1 那一份占着的槽不变，
/// I-3.16 走得过「占着的槽数乘 16 KiB 等于记账的已分配」那一格、走到按记录口径数全空段那一步，而这一版读到的分配记录树是 0 棵。
/// 不是 [`record_new_pool_with_a_tree_table_entry_kind_rewritten`]：那一个要被改的那一条根指针本来全零，分配记录树有根。
fn record_new_pool_with_no_tree_table_entry_of_the_allocation_record_kind(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-tree-table-without-the-allocation-record-kind",
        &is_a_tree_table_unit,
        false,
        &|bytes| {
            let entry = only_tree_table_entry_of_kind(bytes, TREE_KIND_ALLOCATION_RECORD);
            let root_pointer = entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
            assert_ne!(
                bytes[root_pointer..root_pointer + NODE_POINTER_BYTES],
                [0u8; NODE_POINTER_BYTES],
                "发布那一版的分配记录树有根"
            );
            let kind = entry + TREE_TABLE_ENTRY_KIND_OFFSET;
            bytes[kind..kind + 2].copy_from_slice(&UNREGISTERED_TREE_KIND.to_le_bytes());
        },
    )
}

/// I-2.4 根记录那一处：发布的根记录里 MAC 末字节（偏移 465）写 1、重封根记录。core 不读这 29 字节（`root_record.rs` 的 `parse_slot` 跳过），
/// 两遍恢复照常；CPU 池级 checker 对根环里每条自证过的根判这 29 字节全 0，发布的根落了的状态上红 I-2.4。
fn record_new_pool_with_a_non_zero_byte_in_the_root_record_mac(tag: &str) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, "new-pool-root-record-mac-last-byte-one");
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        assert_eq!(
            bytes[ROOT_RECORD_LAST_MAC_BYTE_OFFSET], 0,
            "加密关着时根记录的 MAC 恒 0"
        );
        bytes[ROOT_RECORD_LAST_MAC_BYTE_OFFSET] = 1;
    });
    flow
}

/// I-2.4 根槽尾（C728）：发布的根记录之后的头一个字节（偏移 466）写 1、重封根记录。core 不读这一段，两遍恢复照常；
/// CPU 池级 checker 对根环里每条自证过的根判 [466, 槽宽) 全 0，发布的根落了的状态上红 I-2.4。
fn record_new_pool_with_a_non_zero_byte_past_the_root_record(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-root-slot-byte-past-the-record-one");
    let publish_root = flow.judged_root_index;
    let record_end = usize::try_from(singlefs_format::ROOT_RECORD_BYTES).expect("466");
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        assert!(bytes.len() > record_end, "根槽宽过根记录");
        assert_eq!(bytes[record_end], 0, "第一版根记录之后写 0");
        bytes[record_end] = 1;
    });
    flow
}

/// 根记录里回收门槛 t 8 字节与钉槽 p 1 字节的偏移（C773；`crates/singlefs-core/src/root_record.rs` 的写者：分配记录树根指针 342..428 之后）。
const ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET: usize = 428;

const ROOT_RECORD_PINNED_RING_SLOT_OFFSET: usize = 436;

/// journal 记录里同一对字段：新根段从 99 起，t 在段内 364、p 在段内 372（`crates/singlefs-core/src/journal.rs` 的写者）。
const JOURNAL_RECORD_RECLAIM_THRESHOLD_OFFSET: usize = 99 + 364;

const JOURNAL_RECORD_PINNED_RING_SLOT_OFFSET: usize = 99 + 372;

/// 发布的根记录与它那次发布的每一份 journal 记录里的 t、p 一起改成给定的值、重封（I-8.10 照旧成立，只红 I-7.18 或 I-7.19）。
fn rewrite_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_publish(
    flow: &mut RecordedFlow,
    reclaim_threshold: u64,
    pinned_ring_slot: u8,
) {
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(flow, publish_root, &|bytes| {
        bytes[ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET..ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET + 8]
            .copy_from_slice(&reclaim_threshold.to_le_bytes());
        bytes[ROOT_RECORD_PINNED_RING_SLOT_OFFSET] = pinned_ring_slot;
    });
    rewrite_every_journal_record_of_the_publish(flow, &|bytes| {
        bytes[JOURNAL_RECORD_RECLAIM_THRESHOLD_OFFSET..JOURNAL_RECORD_RECLAIM_THRESHOLD_OFFSET + 8]
            .copy_from_slice(&reclaim_threshold.to_le_bytes());
        bytes[JOURNAL_RECORD_PINNED_RING_SLOT_OFFSET] = pinned_ring_slot;
    });
}

/// 发布的根记录里写着的 t 与 p。
fn reclaim_threshold_and_pinned_ring_slot_of_the_publish(flow: &RecordedFlow) -> (u64, u8) {
    let bytes = flow.writes[flow.judged_root_index]
        .bytes()
        .expect("根槽写带字节");
    (
        u64::from_le_bytes(
            bytes[ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET..ROOT_RECORD_RECLAIM_THRESHOLD_OFFSET + 8]
                .try_into()
                .expect("8"),
        ),
        bytes[ROOT_RECORD_PINNED_RING_SLOT_OFFSET],
    )
}

/// I-7.18（C773）：新池新建文件那次发布的根与记录里 t 写成 1。那一刻 F = 0、环里最旧有效根是第 0 代（txg 0）、没有钉着的槽，
/// 写者写 t = 0；CPU 池级 checker 现算 0，发布的根落了的状态上红 I-7.18。
fn record_new_pool_with_a_reclaim_threshold_above_the_recomputed_one(tag: &str) -> RecordedFlow {
    let mut flow =
        record_new_pool_file_creation(tag, "new-pool-publish-root-reclaim-threshold-one");
    assert_eq!(
        reclaim_threshold_and_pinned_ring_slot_of_the_publish(&flow),
        (0, 255),
        "新池新建文件那次发布写 t = 0、p = 255（F = 0，第 0 代根还在环里，没有钉着的槽）"
    );
    rewrite_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_publish(&mut flow, 1, 255);
    flow
}

/// I-7.19（C773）：新池新建文件那次发布的根与记录里 p 写成 1——还原出区域 1 槽 0，那一槽住着同一实例（1）写的暖机根（txg 1）。
/// t 不动：钉值 1 不比环里最旧有效根（第 0 代，txg 0）小，现算照旧是 0，I-7.18 不红。
fn record_new_pool_with_a_pinned_ring_slot_holding_a_root_of_the_same_instance(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-publish-root-pinned-on-a-root-of-the-same-instance",
    );
    rewrite_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_publish(&mut flow, 0, 1);
    flow
}

/// I-7.18 连带 I-3.1 钉带出来的那一截（C773）：固定脚本到回退 D，那次发布的根与记录里 t 改掉（写者写的 t 大于 0 就写成 0，
/// 那一截 (0, 不带钉现算的门槛] 里的已释放槽 CPU 不看引用并进遍历；是 0 就写成 1）。两遍都要 GPU 逐位跟上 CPU（I-3.1 判不判、红不红一并比）。
fn record_rollback_with_the_reclaim_threshold_of_the_publish_moved(tag: &str) -> RecordedFlow {
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-publish-root-reclaim-threshold-moved";
    let (reclaim_threshold, pinned_ring_slot) =
        reclaim_threshold_and_pinned_ring_slot_of_the_publish(&flow);
    let moved = if reclaim_threshold > 0 { 0 } else { 1 };
    rewrite_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_publish(
        &mut flow,
        moved,
        pinned_ring_slot,
    );
    flow
}

/// 第 `root_write` 次写（根槽写）指着的树表单元的字节：按根记录里树表指针第 0 条位置条目找根槽写之前落在那个槽上的最后一次单元写。
fn tree_table_unit_bytes_named_by_the_root_write(
    flow: &RecordedFlow,
    root_write: usize,
) -> Vec<u8> {
    let root_bytes = flow.writes[root_write].bytes().expect("根槽写带字节");
    let location = ROOT_RECORD_TREE_TABLE_POINTER_FIRST_LOCATION_OFFSET;
    let device = u32::from_le_bytes(root_bytes[location..location + 4].try_into().expect("4"));
    let mut slot = [0u8; 8];
    slot[..6].copy_from_slice(&root_bytes[location + 4..location + 10]);
    let slot = u64::from_le_bytes(slot);
    flow.writes[..root_write]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device.0 == device
                && write.offset.0 / 16384 == slot
        })
        .and_then(RetainedWrite::bytes)
        .expect("根槽写之前写过它指着的树表单元")
        .to_vec()
}

/// I-5.4 那一格「一条根只有一棵分配记录树」：发布的根（带树的一版）根记录偏移 342 那条分配记录树根本来全零，换成它自己树表里种类 3 那条的根指针、
/// 重封根记录。走读把它当树 0 再走一遍那棵树（另红 I-1.3、I-5.1），发布那条 journal 记录的新根段里这一项仍全零（另红 I-8.10）；
/// 两棵树指着同一个根，I-5.4 的区间判定只判一次、不红——红 I-5.4 的只有全零那一格。core 的只读恢复在带树的一版上不读这一项，两遍恢复照常。
fn record_new_pool_with_the_allocation_record_tree_root_of_the_root_record_not_zero(
    tag: &str,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(
        tag,
        "new-pool-root-record-allocation-record-tree-root-not-zero",
    );
    let publish_root = flow.judged_root_index;
    let tree_table = tree_table_unit_bytes_named_by_the_root_write(&flow, publish_root);
    let allocation_entry = only_tree_table_entry_of_kind(&tree_table, 3);
    let allocation_root_pointer =
        tree_table[allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
            ..allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET + NODE_POINTER_BYTES]
            .to_vec();
    assert_ne!(
        allocation_root_pointer,
        vec![0u8; NODE_POINTER_BYTES],
        "带树那一版的分配记录树有根"
    );
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        let held = ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET;
        assert_eq!(
            bytes[held..held + NODE_POINTER_BYTES],
            [0u8; NODE_POINTER_BYTES],
            "带树的一版根记录里这一项本来全零"
        );
        bytes[held..held + NODE_POINTER_BYTES].copy_from_slice(&allocation_root_pointer);
    });
    flow
}

/// I-2.4 指针那一半：发布那张树表里 inode 树那条（种类 2）的根指针头部 `field` 那一段写成非第一版的取值、重封到根。
/// 池级 checker 读码 2 节点（`read_index_node`）时判 MAC / nonce 与 extent 偏移，CPU 与 GPU 走读内核都红 I-2.4。core 读树表条目时判加密、压缩
/// 那几段与 extent 偏移、把那一条当损坏拒（`TreeTableEntry::parse`），GPU 恢复内核在 `walk_to_file` 解树表条目那一处同拒：这一版两遍恢复都是读失败。
pub(super) fn record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
    tag: &str,
    field: PointerHeadFieldWrittenNonZero,
) -> RecordedFlow {
    let name = match field {
        PointerHeadFieldWrittenNonZero::ExtentOffset => {
            "new-pool-inode-tree-root-pointer-extent-offset-one"
        }
        PointerHeadFieldWrittenNonZero::FirstMacByte => {
            "new-pool-inode-tree-root-pointer-first-mac-byte-one"
        }
        PointerHeadFieldWrittenNonZero::AlgorithmType => {
            "new-pool-inode-tree-root-pointer-algorithm-type-one"
        }
    };
    record_new_pool_with_a_resealed_field(tag, name, &is_a_tree_table_unit, false, &|bytes| {
        let entry = only_tree_table_entry_of_kind(bytes, 2);
        write_the_pointer_head_field_non_zero_at(
            bytes,
            entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET,
            field,
        );
    })
}

/// I-9.16 ③：树表里种类 40 有两条（恢复那一半 core 拒、GPU 恢复内核要同拒）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_a_kind_not_below_thirty_two(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_a_kind_not_below_thirty_two",
        &record_new_pool_with_two_tree_table_entries_of_a_kind_not_below_thirty_two(
            "crash-judge-gpu-i916-kind-forty-twice",
        ),
        "I-9.16",
    );
}

/// I-9.16 ③：树表里一种种类有两条。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_one_kind() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_two_entries_of_one_kind",
        &record_new_pool_with_two_tree_table_entries_of_the_sparse_side_table_kind(
            "crash-judge-gpu-i916-kind-twice",
        ),
        "I-9.16",
    );
}

/// I-9.16 ③ 单独不成立（C675）：树表十二种齐、只多一条种类 7
/// （[`record_new_pool_with_every_tree_kind_and_one_more_entry_of_the_sparse_side_table_kind`]）。点 ③ 的另两条流同时缺一种、④ 也红，
/// GPU 走读内核 `w_judge_tree_table_entries_ordering` 关掉 ③ 那一处比较它们照样红 I-9.16；这条流上只有 ③ 不成立，关掉 ③ 的 GPU 判器
/// 在根落了的状态上不置 I-9.16、与 CPU 不一致。CPU 一侧先钉流造对了——整条落盘的镜像上 I-9.16 红、违例说明点名 ③ 种类 7 两条、
/// 不提 ① ② ④；I-7.8 与 I-8.10 成立（水位在根与记录新根段里一起抬过）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_every_kind_and_one_of_them_twice()
{
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_carries_every_kind_and_one_of_them_twice";
    let flow =
        record_new_pool_with_every_tree_kind_and_one_more_entry_of_the_sparse_side_table_kind(
            "crash-judge-gpu-i916-every-kind-and-one-twice",
        );
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-9.16") {
        InvariantVerdict::Violated(detail) => {
            let duplicated_kind =
                format!("种类 {TREE_KIND_SPARSE_SIDE_TABLE} 有 2 条（③ 每种至多一条）");
            assert!(
                detail.contains(&duplicated_kind),
                "I-9.16 的违例说明点名 ③「{duplicated_kind}」：{detail}"
            );
            for other_clause in ["①", "②", "④"] {
                assert!(
                    !detail.contains(other_clause),
                    "十二种齐、只多一条重复的树表只违反 ③，违例说明不提 {other_clause}：{detail}"
                );
            }
        }
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => panic!(
            "树表里种类 {TREE_KIND_SPARSE_SIDE_TABLE} 两条：I-9.16 要红在 ③，实际 {other_verdict:?}"
        ),
    }
    for invariant_that_holds in ["I-7.8", "I-8.10"] {
        let verdict =
            pool_checker_verdict_on_the_image(&image_after_the_whole_flow, invariant_that_holds);
        assert!(
            matches!(verdict, InvariantVerdict::Holds),
            "水位在根与记录新根段里一起抬过，{invariant_that_holds} 照旧成立，实际 {verdict:?}"
        );
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.16"),
        "流 {} 上 CPU 池级 checker 要红 I-9.16（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// I-9.16 ④：树表非空而十一种里缺 deadlist（种类 1–4 之外的那七种之一，CPU 的 ④ 按十一种判时才红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_lacks_the_deadlist_kind() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_lacks_the_deadlist_kind",
        &record_new_pool_with_no_tree_table_entry_of_the_deadlist_kind(
            "crash-judge-gpu-i916-kind-missing",
        ),
        "I-9.16",
    );
}

/// I-3.16 那道守：树表缺种类 3（分配记录树）那一形（[`record_new_pool_with_no_tree_table_entry_of_the_allocation_record_kind`]）。
/// 读法（写段收尾件认的）：最新根那一版一棵分配记录树都没有，按记录口径的全空聚簇段数就不判（`walk.rs` 记射程里没判；C701 的口径改过之后
/// 射程里判不了的报部分没判、不报不适用）。
/// CPU 一侧先钉流造对了——整条落盘的镜像上 I-3.16 报部分没判、说明点名「一棵分配记录树都没有」（走到了按记录口径数全空段那一步，
/// 不是更早那几格的射程里没判），I-3.1 报部分没判、说明点名 I-9.16（树表非空而一条种类 3 都没有，记录口径无从比
/// ），I-9.16 红。流上发布的根落了的状态，CPU 与 GPU 扫描内核都不置 I-3.16 的判到位
/// （`crash_judge_checker_scan.wgsl` 里 `newest_allocation_record_trees_read == 0u` 那一支记射程里没判），每个状态逐位相同；没有这道守的扫描内核
/// 按空的记录表数出整段都空、与记账对不上而判红，与 CPU 不一致。
/// 同一形也是 I-3.21（走到的引用起点是最新根计入的分配记录的起点）前置不满足的那一形（判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md`
/// 第二节 Q2「归属」：前置是最新根的树表带种类 3 记录树且读全，不满足报这一条判不了）：整条落盘的镜像上 CPU 报 I-3.21 部分没判、说明点名
/// 一条种类 3 都没有；流上每个状态 CPU 都不判到 I-3.21（不适用或部分没判），GPU 扫描内核判了池级 checker 的状态上 I-3.21 的判到位与违例位都不置
/// （`c_judge_walked_reference_starts_against_the_records_of_the_newest_root` 里树表非空而没有种类 3 那一支记射程里判不了）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_the_statistics_of_a_newest_version_without_an_allocation_record_tree_are_not_judged(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_that_the_statistics_of_a_newest_version_without_an_allocation_record_tree_are_not_judged";
    let flow = record_new_pool_with_no_tree_table_entry_of_the_allocation_record_kind(
        "crash-judge-gpu-i316-no-allocation-record-tree",
    );
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.16") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains("一棵分配记录树都没有"),
            "部分没判的说明点名最新根那一版一棵分配记录树都没有（不是更早那几格）：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "树表缺种类 3、分配记录树的节点照旧在映射里：I-3.16 要走到按记录口径那一步、射程里判不了（报部分没判），实际 {other_verdict:?}"
        ),
    }
    match pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.1") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains("一条种类 3（分配记录树）都没有（I-9.16 ④ 判它）"),
            "I-3.1 部分没判的说明点名 I-9.16 判的那块畸形：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "树表非空而一条种类 3 都没有：I-3.1 的记录口径无从比，每块盘报部分没判、点名 I-9.16（GPU 扫描内核不跟这一条时照判红、与 CPU 逐位不同），实际 {other_verdict:?}"
        ),
    }
    match pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.21") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains("一条种类 3（分配记录树）都没有（I-9.16 ④ 判它）：走到的引用起点没比"),
            "I-3.21 部分没判的说明点名树表非空而一条种类 3 都没有：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "树表非空而一条种类 3 都没有：I-3.21 的前置（最新根的树表带种类 3 记录树且读全）不满足，报部分没判（判决 \
             research/prompts/m3-close-b19-reason-r2-main-verification.md 第二节 Q2「归属」），实际 {other_verdict:?}"
        ),
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[StateExamination::CountStatesWhere(
            StateCondition::GpuCheckerJudgedOrViolated("I-3.21"),
        )],
    ) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.16"),
        "流 {} 上树表缺种类 3，CPU 池级 checker 要红 I-9.16（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    // 这条流上最新根要么是树表 0 条的种子根、暖机根（I-3.21 不适用），要么是发布那一版、树表缺种类 3（部分没判）：一个状态都不判到 I-3.21
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-3.21")
            && !comparison.cpu_checker_held_invariants.contains("I-3.21"),
        "流 {}：最新根的树表 0 条或缺种类 3，CPU 池级 checker 在每个状态上都该报 I-3.21 不适用或部分没判（红过的：{:?}；判过而成立过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants,
        comparison.cpu_checker_held_invariants
    );
    let states_on_which_the_gpu_judged_the_walked_reference_starts = &comparison.examined[0];
    assert!(
        states_on_which_the_gpu_judged_the_walked_reference_starts.states_counted == 0,
        "流 {}：GPU 扫描内核在树表缺种类 3 的最新根上要记 I-3.21 射程里判不了、不置它的判到位与违例位；置了的状态 {} 个（前几个）：{:?}",
        flow.name,
        states_on_which_the_gpu_judged_the_walked_reference_starts.states_counted,
        states_on_which_the_gpu_judged_the_walked_reference_starts.first_ordinals_counted
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// I-3.20 (b) 判不了那一形（[`record_new_pool_with_no_tree_table_entry_of_the_deleted_inodes_kind`]）：最新根的树表非空而一条种类 9 都没有，
/// 已承诺预留该取 0 还是照闭式算认不出（在镜像上、
/// 在射程里而配不上的报部分没判，那块畸形 I-9.16 ④ 判红）。CPU 一侧先钉流造对了——整条落盘的镜像上 I-3.20 报部分没判、说明点名 I-9.16
/// （照「没有 `deleted_inodes` 条目取 0」判的话，写者按闭式记的非 0 预留在这里红），I-9.16 红。流上每个状态 CPU 与 GPU 扫描内核逐位相同：
/// `crash_judge_checker_scan.wgsl` 的 `judge_the_committed_reservation_against_its_closed_form` 里树表非空而没有 deleted_inodes 条目那一支
/// 记射程里判不了；没有那一支的扫描内核照取 0 判红、与 CPU 不一致。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_the_committed_reservation_of_a_newest_version_without_a_deleted_inodes_entry_is_not_judged(
) {
    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_that_the_committed_reservation_of_a_newest_version_without_a_deleted_inodes_entry_is_not_judged";
    let flow = record_new_pool_with_no_tree_table_entry_of_the_deleted_inodes_kind(
        "crash-judge-gpu-i320-no-deleted-inodes-entry",
    );
    let mut image_after_the_whole_flow = flow.base.clone();
    image_after_the_whole_flow.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_after_the_whole_flow, "I-3.20") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains("一条种类 9（deleted_inodes）都没有（I-9.16 ④ 判它）"),
            "I-3.20 部分没判的说明点名 I-9.16 判的那块畸形：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "树表非空而一条种类 9 都没有：I-3.20 的已承诺预留那一句没判、报部分没判、点名 I-9.16（照取 0 判时写者记的非 0 预留在这里红），实际 {other_verdict:?}"
        ),
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.16"),
        "流 {} 上树表缺种类 9，CPU 池级 checker 要红 I-9.16（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// I-7.18（C773）：建账的发布带的回收门槛高过现算的；GPU 走读内核逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_publish_root_carries_a_reclaim_threshold_above_the_recomputed_one(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_publish_root_carries_a_reclaim_threshold_above_the_recomputed_one",
        &record_new_pool_with_a_reclaim_threshold_above_the_recomputed_one(
            "crash-judge-gpu-i718-reclaim-threshold",
        ),
        "I-7.18",
    );
}

/// I-7.19（C773）：建账的发布带的钉槽指着同一实例写的根；GPU 走读内核逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_pinned_ring_slot_holds_a_root_of_the_same_instance() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_pinned_ring_slot_holds_a_root_of_the_same_instance",
        &record_new_pool_with_a_pinned_ring_slot_holding_a_root_of_the_same_instance(
            "crash-judge-gpu-i719-pinned-ring-slot",
        ),
        "I-7.19",
    );
}

/// I-7.18 与 I-3.1 钉带出来的那一截（C773）：固定脚本到回退 D 那次发布的 t 改掉；GPU 走读内核（I-7.18）与扫描内核（I-3.1 那一截、I-3.16 占着的槽）
/// 逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_rollback_publish_root_carries_a_moved_reclaim_threshold(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_rollback_publish_root_carries_a_moved_reclaim_threshold",
        &record_rollback_with_the_reclaim_threshold_of_the_publish_moved(
            "crash-judge-gpu-i718-rollback-reclaim-threshold",
        ),
        "I-7.18",
    );
}

/// I-2.4：根记录里算法类型 / nonce / MAC 那 29 字节有非 0。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_root_record_carries_a_non_zero_mac_byte() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_root_record_carries_a_non_zero_mac_byte",
        &record_new_pool_with_a_non_zero_byte_in_the_root_record_mac(
            "crash-judge-gpu-i24-root-mac",
        ),
        "I-2.4",
    );
}

/// I-2.4（C728）：根记录之后到槽宽那一段有一个非 0 字节；GPU 走读内核对每条自证过的根判 [466, 槽宽) 全 0，逐位跟上 CPU。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_root_slot_carries_a_non_zero_byte_past_the_root_record()
{
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_root_slot_carries_a_non_zero_byte_past_the_root_record",
        &record_new_pool_with_a_non_zero_byte_past_the_root_record("crash-judge-gpu-i24-root-tail"),
        "I-2.4",
    );
}

/// I-5.4：带树的一版根记录里还持有一棵分配记录树。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record",
        &record_new_pool_with_the_allocation_record_tree_root_of_the_root_record_not_zero(
            "crash-judge-gpu-i54-held-root",
        ),
        "I-5.4",
    );
}

/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对「同一块盘、前一条正好接到后一条」的记录，前一条跨度加一（已释放标志照留）：
/// 两条罩住同一个槽。返回改了几条（0 或 1）。抄自 `checker_known_bad_images.rs` 的同名函数（测试目标之间互相引不到，这里另写一份）。
pub(super) fn widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
    bytes: &mut [u8],
) -> usize {
    const ALLOCATION_KEY_WIDTH: u8 = 10;
    const ALLOCATION_RECORD_BYTES: usize = 20;
    const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
    if bytes[50] != 0 || index_node_key_width(bytes) != ALLOCATION_KEY_WIDTH {
        return 0;
    }
    let header_end = 86 + 2 * usize::from(ALLOCATION_KEY_WIDTH);
    let record_count = usize::from(u16::from_le_bytes(
        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
    ));
    let entries_start = 115 + 2 * usize::from(ALLOCATION_KEY_WIDTH);
    let record_at = |leaf_bytes: &[u8], record_index: usize| {
        let start = entries_start + ALLOCATION_RECORD_BYTES * record_index;
        let device = u32::from_le_bytes(leaf_bytes[start..start + 4].try_into().expect("4"));
        let mut slot_bytes = [0u8; 8];
        slot_bytes[..6].copy_from_slice(&leaf_bytes[start + 4..start + 10]);
        let span_field = u16::from_le_bytes([leaf_bytes[start + 10], leaf_bytes[start + 11]]);
        (start, device, u64::from_le_bytes(slot_bytes), span_field)
    };
    for record_index in 0..record_count.saturating_sub(1) {
        let (start, device, slot, span_field) = record_at(bytes, record_index);
        let (_, next_device, next_slot, _) = record_at(bytes, record_index + 1);
        let span_slots = span_field & !ALLOCATION_RECORD_RELEASED_FLAG;
        if device == next_device && slot + u64::from(span_slots) == next_slot {
            let widened = (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | (span_slots + 1);
            bytes[start + 10..start + 12].copy_from_slice(&widened.to_le_bytes());
            return 1;
        }
    }
    0
}

/// I-5.4 按版本那一层多收下一次挂载先施加的那一版（改法 RV，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2）：
/// 新池新建文件那次发布写的最后一片分配记录叶里两条记录罩住同一个槽，重封到根与 journal 记录。发布那条记录落了、根槽没落的那几个状态上，
/// 这一版是下一次挂载先施加的那一版，只有改法 RV 之后的 I-5.4 判得到；根槽落了的状态上它是最新的根。两边每个状态逐位比。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first",
        &record_new_pool_with_a_resealed_field(
            "crash-judge-gpu-i54-overlapping-records",
            "new-pool-overlapping-allocation-records",
            &|bytes| {
                unit_class(bytes) == 2
                    && index_node_key_width(bytes) == 10
                    && bytes[50] == 0
                    && widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
                        &mut bytes.to_vec(),
                    ) == 1
            },
            false,
            &|bytes| {
                assert_eq!(
                    widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
                    1,
                    "挑叶时这一片改得动"
                );
            },
        ),
        "I-5.4",
    );
}
