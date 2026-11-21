//! 墓碑树、谱系树与 inode 树的条目（I-9 类）：根条目的身份与类型段、墓碑与谱系记录、叶里记录的次序与分隔 key、条目宽窄于字段表。

use super::*;

// ───────────────────────── 里程碑三·3 墓碑与谱系那九条与两处坏盘分叉 ─────────────────────────

/// 码 2 节点条目里身份引用的出生树（分隔 key 8 之后）与容器号（类型段 2 之后）。
pub(super) const CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET: usize = 8;

pub(super) const CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET: usize = 18;

const CONTAINER_TREE_ENTRY_CONTAINER_BIRTH_OFFSET: usize = 26;

/// 墓碑树、谱系树条目类型段条款之外的一个值（墓碑 {0, 1}、谱系 {0, 5} 之外）。
pub(super) const UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE: u16 = 7;

/// 改坏墓碑的对象 ID 时写的号：inode 树里没有这个号的记录，也不越过 inode 号水位（I-9.6 墓碑那一半照旧成立）。
const OBJECT_IDENTIFIER_WITHOUT_A_RECORD: u64 = 0;

/// I-9.21 那条流里墓碑容器旋钮压到的每片记录数：两次 unlink 各占一片，最左那片不是最右的、只装 1 条。
const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

/// N4-f、N4-j 那两条流里容器树旋钮压到的每个码 2 节点条目数：三片容器装不进一个根，根长到层级 2。
const CONTAINER_TREE_ENTRIES_PER_NODE_UNDER_THE_KNOB: usize = 2;

/// 码 2 节点头里层级那一字节的偏移（D18（块里携带什么信息） 已定项 18：42 树 ID / 50 层级 / 51 key 宽）。
const INDEX_NODE_LEVEL_OFFSET: usize = 50;

/// 墓碑树、谱系树、inode 树的码 2 节点里第一条条目的起点（key 是 8 字节的容器号或 inode 号）。
pub(super) fn first_entry_of_a_container_number_node() -> usize {
    index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
}

pub(super) fn is_the_tombstone_tree_root(bytes: &[u8]) -> bool {
    is_a_container_tree_root_over_containers_of_record_type(bytes, PACKED_RECORD_TYPE_TOMBSTONE)
}

pub(super) fn is_the_lineage_tree_root(bytes: &[u8]) -> bool {
    is_a_container_tree_root_over_containers_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE)
}

/// 码 2 节点头里的条目数。
pub(super) fn index_node_entry_count(bytes: &[u8]) -> u16 {
    read_u16_at(
        bytes,
        index_node_entry_count_offset(usize::from(index_node_key_width(bytes))),
    )
}

/// 码 2 节点头里的条目宽改成 `entry_width`（条目数 0：声明长度 0 = 0 × 条目宽照旧成立）。
fn set_the_entry_width_of_an_empty_index_node(bytes: &mut [u8], entry_width: u16) {
    assert_eq!(index_node_entry_count(bytes), 0, "零条目的码 2 节点");
    let entry_width_offset =
        index_node_entry_count_offset(usize::from(index_node_key_width(bytes))) + 2;
    bytes[entry_width_offset..entry_width_offset + 2].copy_from_slice(&entry_width.to_le_bytes());
}

/// 容器树根第一条条目里身份引用的容器号加 1（分隔 key 与子指针不动）。
fn add_one_to_the_container_number_of_the_first_entry(bytes: &mut [u8]) {
    let field =
        first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
    let container = read_u64_at(bytes, field);
    bytes[field..field + 8].copy_from_slice(&(container + 1).to_le_bytes());
}

/// 容器树根第一条条目的类型段写成 `entry_type`。
pub(super) fn set_the_type_of_the_first_entry(bytes: &mut [u8], entry_type: u16) {
    let field = first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET;
    bytes[field..field + 2].copy_from_slice(&entry_type.to_le_bytes());
}

/// 墓碑容器第一条墓碑偏移 `field` 起的 8 字节改成 `change` 给的值。
fn change_the_first_tombstone_field(bytes: &mut [u8], field: usize, change: impl Fn(u64) -> u64) {
    assert!(
        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
        "墓碑容器里至少一条墓碑"
    );
    let offset = PACKED_UNIT_RECORDS_START + field;
    let value = change(read_u64_at(bytes, offset));
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

/// 同 [`assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening`]，CPU 池级 checker 要红的是 `invariants` 里的每一条。
pub(super) fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
    test_name: &str,
    flow: &RecordedFlow,
    invariants: &[&'static str],
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    for invariant in invariants {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// I-9.17 容器那一支：固定脚本到回退 D 的流上谱系树根第一条条目（类型段 5、指谱系容器）身份引用里的容器号加 1、重封到根。
/// 容器头没动，条目身份与子头不符：CPU 池级 checker 判 I-9.17 红、不记走读失败，GPU 走读内核同判（在查这片容器走没走过之前判）。
/// 恢复读谱系树根时层级 1 的条目只数不读，两边恢复照旧。
fn record_rollback_with_the_lineage_root_entry_naming_another_container(tag: &str) -> RecordedFlow {
    record_rollback_with_a_resealed_field(
        tag,
        "rollback-lineage-root-entry-names-another-container",
        &is_the_lineage_tree_root,
        &add_one_to_the_container_number_of_the_first_entry,
    )
}

/// I-9.23 类型段不认得那一支：删除中间态的流上墓碑树根第一条条目的类型段 1 → 7、重封到根。CPU 先登记这条子指针的引用（层级 1 按容器两槽）
/// 再判 I-9.23 红、不往下走、不记走读失败；GPU 走读内核同样先登记引用再置位。删除态扫描读这棵墓碑树判不了，I-9.18 起那几条不判。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-seven",
        &is_the_tombstone_tree_root,
        &|bytes| set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE),
    )
}

/// I-9.23 类型段 0 那一支（攻方 N4-a 那一形，`research/prompts/m3-batch22-code-r1-opus-output.md` 第一节）：删除中间态的流上墓碑树根（层级 1）
/// 第一条条目的类型段 1 → 0、重封到根。身份引用的出生树不是 0、孩子是码 3：CPU 只读孩子的单元头得码 3、与类型段 0 不符，判 I-9.23 红、
/// 按两槽登记引用、不往下走、不记走读失败；
/// GPU 走读内核同判同登记。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-zero",
        &is_the_tombstone_tree_root,
        &|bytes| {
            assert_ne!(
                read_u64_at(
                    bytes,
                    first_entry_of_a_container_number_node()
                        + CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET
                ),
                0,
                "指墓碑容器的条目身份引用里出生树不是 0"
            );
            set_the_type_of_the_first_entry(bytes, 0);
        },
    )
}

/// 容器树节点第一条条目的身份引用里出生树、容器号、容器出生代三段写 0（类型段那两字节不动）。
pub(super) fn clear_the_identity_of_the_first_entry_but_its_type(bytes: &mut [u8]) {
    let first_entry = first_entry_of_a_container_number_node();
    for field in [
        CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET,
        CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET,
        CONTAINER_TREE_ENTRY_CONTAINER_BIRTH_OFFSET,
    ] {
        bytes[first_entry + field..first_entry + field + 8].fill(0);
    }
}

/// I-9.23 类型段 0 那一支、身份三段也清零（攻方 N4-i 那一形）：删除中间态的流上墓碑树根（层级 1）第一条条目的类型段 1 → 0、
/// 出生树、容器号、容器出生代三段写 0、重封到根。身份引用那一半成立，只剩「子单元是码 2」那一小句不成立：孩子是码 3 容器。
/// CPU 只读孩子的单元头得码 3，判 I-9.23 红、不往下走、不记走读失败；
/// GPU 走读内核同判。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity(
    tag: &str,
) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-root-entry-type-zero-no-identity",
        &is_the_tombstone_tree_root,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, 0);
            clear_the_identity_of_the_first_entry_but_its_type(bytes);
        },
    )
}

/// I-9.2 同形（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行「inode 树 I-9.2 同形一并查」）：
/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、重封到根。条目说它指下一层码 2 节点，孩子是码 3 叶容器：
/// CPU 只读孩子的单元头得码 3、按 32 KiB 自证得过，判 I-9.2 红、按两槽登记引用、不往下走、不记走读失败；
/// GPU 走读内核同判同登记。
fn record_new_pool_with_the_inode_root_entry_of_type_zero_naming_its_leaf_container(
    tag: &str,
) -> RecordedFlow {
    let inode_tree_root_over_leaf_containers = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
            && read_u16_at(
                bytes,
                index_node_entries_start(INODE_TREE_KEY_BYTES)
                    + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
            ) == PACKED_RECORD_TYPE_INODE
    };
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-root-entry-type-zero-names-its-leaf-container",
        &inode_tree_root_over_leaf_containers,
        false,
        &|bytes| set_the_type_of_the_first_entry(bytes, 0),
    )
}

/// 节点指针里第一条位置条目的槽号（6 字节）的偏移：两条位置条目在 50 与 64（盘 4、槽 6、整单元校验和 4）。
const POINTER_FIRST_LOCATION_SLOT_OFFSET: usize = 54;

/// H1 的 inode 那一形：
/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、身份引用三段写 0、重封到根（除了孩子的类，这是一条合法的类型段 0 条目），
/// 再把它指着的叶容器盘序小的那一份写的前 16 KiB 换成原来那个 inode 树根的原样字节（陈旧副本：头里类标签 2，恰是类型段 0 说的那一类；
/// 不补校验和，位置条目还是叶容器原来那一份的），盘序大的那一份不动。CPU 认孩子的类时头里是类型段那一类的副本也要按那一类的长度自证，
/// 第一份自证不过看第二份、认出码 3，判 I-9.2 红、不往下走、不记走读失败（只认第一份的头、不验的话，I-9.2 报成立，另连带 I-7.2 等）；
/// GPU 走读内核同改（`w_child_unit_class_other_than_the_entry_type_names`）。
fn record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-inode-root-entry-type-zero-no-identity-first-leaf-copy-stale";
    let mut flow = record_new_pool_file_creation(tag, name);
    let inode_tree_root_over_leaf_containers = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
            && read_u16_at(
                bytes,
                index_node_entries_start(INODE_TREE_KEY_BYTES)
                    + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
            ) == PACKED_RECORD_TYPE_INODE
    };
    let inode_leaf_container = |bytes: &[u8]| {
        unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
    };
    let root_copies = last_unit_copies(&flow, name, &inode_tree_root_over_leaf_containers, false);
    let stale_bytes = flow.writes[root_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    let first_leaf_copy = last_unit_copies(&flow, name, &inode_leaf_container, true)[0];
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, 0);
        clear_the_identity_of_the_first_entry_but_its_type(bytes);
    });
    let root_after_the_change = flow.writes[root_copies[0]].bytes().expect("单元写带字节");
    let first_location_slot_offset = first_entry_of_a_container_number_node()
        + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_SLOT_OFFSET;
    let mut slot_named_by_the_first_entry = [0u8; 8];
    slot_named_by_the_first_entry[..6].copy_from_slice(
        &root_after_the_change[first_location_slot_offset..first_location_slot_offset + 6],
    );
    assert_eq!(
        u64::from_le_bytes(slot_named_by_the_first_entry),
        flow.writes[first_leaf_copy].offset.0 / 16384,
        "inode 树根第一条条目指着的就是流里最后写的那片叶容器"
    );
    assert_eq!(unit_class(&stale_bytes), 2, "陈旧副本头里的类是码 2");
    let WrittenContents::Bytes(leaf) = &mut flow.writes[first_leaf_copy].contents else {
        panic!("单元写是普通写");
    };
    leaf[..stale_bytes.len()].copy_from_slice(&stale_bytes);
    flow
}

/// I-9.18：删除中间态的流上墓碑容器里那一条墓碑的对象 ID 1 → 0、重封到根。队列 {1} 里的号 1 在墓碑树里没有墓碑：CPU 判 I-9.18 红；
/// 号 0 没有记录，I-9.10 墓碑那一个等号与 I-9.19 没有对象，号 0 不越过水位，I-9.6 墓碑那一半照旧成立。
fn record_deletion_with_the_tombstone_naming_an_object_without_a_record(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-names-an-object-without-a-record",
        &is_a_tombstone_container,
        &|bytes| {
            change_the_first_tombstone_field(bytes, 0, |_object| OBJECT_IDENTIFIER_WITHOUT_A_RECORD)
        },
    )
}

/// I-9.19：删除中间态的流上第一批那一版 inode 叶容器里第一个文件的记录 nlink 0 → 1、重封到根。意图那一版写的墓碑在第一批那一版的谱系上、
/// 指着号与对象出生代都相同而 nlink 1 的记录：CPU 判 I-9.19 红（队列里的号 nlink ≠ 0，I-9.5 同红）。
fn record_deletion_with_the_queued_record_back_at_nlink_one(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-queued-record-back-at-nlink-one",
        &is_an_inode_leaf_container,
        &|bytes| {
            let record = offset_of_the_inode_record(bytes, FIRST_FILE.0);
            assert_eq!(
                read_u32_at(bytes, record + INODE_RECORD_NLINK_OFFSET),
                0,
                "删除中的第一个文件 nlink 0"
            );
            bytes[record + INODE_RECORD_NLINK_OFFSET..record + INODE_RECORD_NLINK_OFFSET + 4]
                .copy_from_slice(&1u32.to_le_bytes());
        },
    )
}

/// I-9.20 管全部墓碑那几句：删除中间态的流上墓碑的死亡 txg 改成 0、重封到根。对象出生代 > 死亡 txg：CPU 判 I-9.20 红；
/// 死亡 txg 0 仍在谱系上（没有回退的一版从 0 到自己的 txg 整段在链上），I-9.18 照旧成立。
fn record_deletion_with_the_tombstone_dying_at_txg_zero(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-dies-at-txg-zero",
        &is_a_tombstone_container,
        &|bytes| {
            change_the_first_tombstone_field(
                bytes,
                TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
                |_death_txg| 0,
            )
        },
    )
}

/// I-9.20 不重号那一句：删除中间态的流上墓碑容器里那一条墓碑原样再追加一条（记录数加一、声明长度加 40）、重封到根。
/// 两条都在谱系上、(号, 对象出生代) 相同：CPU 判 I-9.20 红。
fn record_deletion_with_the_tombstone_written_twice(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-written-twice",
        &is_a_tombstone_container,
        &|bytes| {
            let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
            assert_eq!(record_count, 1, "意图那一次只写一条墓碑");
            let first = PACKED_UNIT_RECORDS_START;
            bytes.copy_within(
                first..first + TOMBSTONE_RECORD_BYTES,
                first + TOMBSTONE_RECORD_BYTES,
            );
            bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
                .copy_from_slice(&2u16.to_le_bytes());
            let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
            bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
                &(declared_length + u16::try_from(TOMBSTONE_RECORD_BYTES).expect("40"))
                    .to_le_bytes(),
            );
        },
    )
}

/// I-9.29 与 I-9.10 墓碑那一个等号：删除中间态的流上墓碑的对象出生代加 1、重封到根。号 1 的记录出生代与谱系上墓碑的不同：
/// CPU 判 I-9.10（按号认）、I-9.29（同号两个出生代）红，I-9.18 跟着红（没有出生代对得上的墓碑）。
fn record_deletion_with_the_tombstone_object_birth_off_its_record(tag: &str) -> RecordedFlow {
    record_deletion_with_a_resealed_field(
        tag,
        "deletion-tombstone-object-birth-off-its-record",
        &is_a_tombstone_container,
        &|bytes| {
            change_the_first_tombstone_field(
                bytes,
                TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
                |object_birth| object_birth + 1,
            )
        },
    )
}

/// I-9.4 第四句：新池新建文件、再建一个空 inode（2 号，与第一个文件同在最左那片叶里）的流上那片叶的两条记录对调、重封到根。
/// 记录序 [2, 1] 不严格递增：CPU 判 I-9.4 红（号都 ≥ 容器号、叶序上的第二、三句照旧成立）；恢复按号找第一个文件，不看次序。
pub(super) fn record_new_pool_with_the_two_records_of_the_inode_leaf_swapped(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
        tag,
        "new-pool-new-inode-leaf-records-swapped",
        &|bytes| {
            let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
            assert_eq!(record_count, 2, "第一个文件与空 inode 两条");
            let first = offset_of_the_inode_record(bytes, FIRST_FILE.0);
            let second = offset_of_the_inode_record(bytes, FIRST_FILE.0 + 1);
            assert!(first < second, "写者按号升序写记录");
            let first_record = bytes[first..first + INODE_RECORD_BYTES].to_vec();
            bytes.copy_within(second..second + INODE_RECORD_BYTES, first);
            bytes[second..second + INODE_RECORD_BYTES].copy_from_slice(&first_record);
        },
    )
}

/// 删到最后一批的流上最后一个零条目的码 2 节点（key 宽 `key_width`）条目宽改成 `entry_width`、重封到根：坏盘分叉。
fn record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
    tag: &str,
    name: &'static str,
    key_width: u8,
    entry_width: u16,
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_last_deletion_batch(tag, name);
    let empty_root = move |bytes: &[u8]| {
        unit_class(bytes) == 2
            && index_node_key_width(bytes) == key_width
            && index_node_entry_count(bytes) == 0
    };
    let targets = last_unit_copies(&flow, name, &empty_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        set_the_entry_width_of_an_empty_index_node(bytes, entry_width);
    });
    flow
}

/// 零条目、层级 0 的 extent 上段根，条目宽 113 → 24（= key 宽）。core 冷走读逐条解条目时才核宽（`extent_tree.rs` 的 `upper_leaf_entries`），
/// 零条目时答没有文件；GPU 恢复内核同判（`read_upper_segment` 只在有条目时核宽）。池级 checker 两边都按字段表判 I-1.10 红。
fn record_deletion_through_the_last_batch_with_a_narrow_empty_extent_root(
    tag: &str,
) -> RecordedFlow {
    const EXTENT_KEY_WIDTH: u8 = 24;
    record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
        tag,
        "deletion-through-the-last-batch-empty-extent-root-of-entry-width-twenty-four",
        EXTENT_KEY_WIDTH,
        u16::from(EXTENT_KEY_WIDTH),
    )
}

/// 零条目的 inode 树根（层级 1），条目宽 120 → 8（= key 宽）。core 冷走读逐条 `inode_internal_entry_parsed_and_judged` 时才核宽，
/// 零条目时答没有文件；GPU 恢复内核同判（`walk_to_file` 只在有条目时核宽）。池级 checker 两边都按字段表判 I-1.10 红。
fn record_deletion_through_the_last_batch_with_a_narrow_empty_inode_root(
    tag: &str,
) -> RecordedFlow {
    let inode_key_width = u8::try_from(INODE_TREE_KEY_BYTES).expect("8");
    record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
        tag,
        "deletion-through-the-last-batch-empty-inode-root-of-entry-width-eight",
        inode_key_width,
        u16::from(inode_key_width),
    )
}

/// 新池新建文件的流上 extent 上段根兼叶（key 宽 24、条目宽 113、一条条目）每条条目末尾补一个 0、条目宽 113 → 114，按原来的头重建、
/// 重封到根（I-1.10（码 2 条目宽等于字段表宽） 写「等于」，宽了的同样拒）。core 冷走读逐条 `ExtentUpperLeafEntry::parse` 判宽不等于 113 就拒、
/// 报 `EntryWidthDiffersFromItsFieldTable`；GPU 恢复内核读上段那一判（`crash_judge_common.wgsl` 有条目时宽不等于
/// `EXTENT_UPPER_LEAF_ENTRY_BYTES` 就拒）同判。池级 checker 两边都按字段表判 I-1.10 红。
fn record_new_pool_with_the_extent_upper_leaf_entries_one_byte_wider(tag: &str) -> RecordedFlow {
    const EXTENT_KEY_WIDTH: u8 = 24;
    const EXTENT_UPPER_LEAF_ENTRY_FIELD_TABLE_BYTES: u16 = 113;
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-extent-upper-leaf-entries-one-byte-wider",
        &|bytes| {
            unit_class(bytes) == 2
                && index_node_key_width(bytes) == EXTENT_KEY_WIDTH
                && index_node_entry_width(bytes) == EXTENT_UPPER_LEAF_ENTRY_FIELD_TABLE_BYTES
                && index_node_entry_count(bytes) > 0
        },
        false,
        &|bytes| {
            let node = singlefs_core::unit::parse_index_node(bytes).expect("上段叶解得开");
            let wider_entries: Vec<Vec<u8>> = node
                .entries
                .iter()
                .map(|entry| {
                    let mut wider = entry.clone();
                    wider.push(0);
                    wider
                })
                .collect();
            let wider_node = singlefs_core::unit::build_index_node(
                node.tree,
                node.level,
                node.key_width,
                &node.smallest_key,
                &node.largest_key,
                node.birth_txg,
                &parameters().filesystem_identifier,
                node.instance,
                node.birth_sequence,
                EXTENT_UPPER_LEAF_ENTRY_FIELD_TABLE_BYTES + 1,
                &wider_entries,
            );
            bytes.copy_from_slice(&wider_node);
        },
    )
}

/// 从新池新建文件（txg 3）到 `pool` 现行那一版，每一版都读得出第一个文件（这几条流不碰它）。
pub(super) fn versions_with_the_first_file_through_the_last_publish(
    pool: &BuiltPool,
) -> Vec<PublishedVersion> {
    (3..=pool.output.root.checkpoint_txg.0)
        .map(|txg| PublishedVersion {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(txg),
            content: file_content(),
        })
        .collect()
}

/// I-9.21：新池新建文件、建两个空 inode（2、3 号），写入口装上只供测试的墓碑容器旋钮（一片 1 条），unlink 2、3 号各到底：
/// 墓碑树根两条条目各指一片容器、各装 1 条。这是旋钮下的合法镜像；CPU 判器走池级 checker 的旧入口（当没装旋钮），
/// 不是最右的那片只装 1 条、不是 815，判 I-9.21 红；GPU 扫描内核只有没装旋钮那一支，同判。
pub(super) fn record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
    tag: &str,
) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, 2);
    let capacities = ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(
                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
            )
            .expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    };
    for inode in [FIRST_FILE.0 + 1, FIRST_FILE.0 + 2] {
        unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
    }
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(
        &pool,
        "two-empty-inodes-unlinked-one-tombstone-per-container",
        versions,
    )
}

/// 写入口装上只供测试的容器树旋钮 `capacities`，把空 inode `inode` unlink 到底：意图那一次，再走空（空 inode 没有数据单元，直接是最后一批）。
pub(super) fn unlink_an_empty_inode_to_the_end_under_the_capacities(
    pool: &mut BuiltPool,
    inode: u64,
    capacities: ContainerTreeCapacities,
) {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities);
    let current = publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        InodeNumber(inode),
        WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    let mut current = Arc::new(current);
    publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        UnlinkBatchSize::OneExtentLowerLeaf,
        INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
    pool.output = Arc::unwrap_or_clone(current);
}

/// 墓碑容器一片 1 条、容器树一个节点 2 条（`crates/singlefs-verification/tests/container_trees_grow_a_level_when_the_root_is_full.rs` 的
/// `capacities_of_one_record_and_two_entries`）：unlink 第三个 inode 时墓碑树根满、长到层级 2。
fn capacities_of_one_tombstone_per_container_and_two_entries_per_node() -> ContainerTreeCapacities {
    ContainerTreeCapacities {
        tombstone_container: TombstoneContainerCapacity::CappedForTests(
            TombstoneRecordsPerContainerCappedForTests::new(
                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
            )
            .expect("1..=815"),
        ),
        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: ContainerTreeNodeEntries::CappedForTests(
            ContainerTreeNodeEntriesCappedForTests::new(
                CONTAINER_TREE_ENTRIES_PER_NODE_UNDER_THE_KNOB,
            )
            .expect("2..=135"),
        ),
    }
}

/// 层级 2 的墓碑树根：码 2、key 宽 8、条目宽 120、层级 2、第一条条目类型段 0（指层级 1 的码 2 节点）。
/// 这几条流里谱系树是零根、inode 树只有一片叶容器（根层级 1），层级 2 的这种节点只有墓碑树根。
pub(super) fn is_a_tombstone_tree_root_of_level_two(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == CONTAINER_NUMBER_KEY_BYTES
        && index_node_entry_width(bytes) == CONTAINER_TREE_ENTRY_BYTES
        && bytes[INDEX_NODE_LEVEL_OFFSET] == 2
        && read_u16_at(
            bytes,
            first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
        ) == 0
}

/// 新池新建文件、建五个空 inode（2–6 号），写入口装 [`capacities_of_one_tombstone_per_container_and_two_entries_per_node`]，
/// unlink 2、3、4 号各到底：三片墓碑容器各装 1 条，第三次 unlink 的意图那次墓碑树根长到层级 2（根两条类型段 0 的条目，
/// 各指一个层级 1 的码 2 节点）。旋钮下的合法镜像；CPU 判器走旧入口当没装旋钮，I-9.21 红（同 I-9.21 那条流）。
/// `turn_the_root_ring_over` 时再建 `PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`（`common_deletion_in_progress`）次空 inode：
/// 墓碑树长高之前那几版都转出回退候选集，层级 1 那两个节点只剩现行这棵墓碑树（与之后照抄它的各版）引用。
pub(super) fn record_a_tombstone_tree_grown_to_level_two(
    tag: &str,
    name: &'static str,
    turn_the_root_ring_over: bool,
) -> RecordedFlow {
    let mut pool = pool_with_new_inodes(tag, 5);
    let capacities = capacities_of_one_tombstone_per_container_and_two_entries_per_node();
    for inode in [FIRST_FILE.0 + 1, FIRST_FILE.0 + 2, FIRST_FILE.0 + 3] {
        unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
    }
    assert_eq!(
        pool.output.tombstone_tree_index_nodes.shape.root_level(),
        2,
        "第三片墓碑容器让根长到层级 2"
    );
    if turn_the_root_ring_over {
        turn_the_root_ring_over_by_creating_inodes(&mut pool);
    }
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    recorded_flow_of_the_pool(&pool, name, versions)
}

/// I-9.23 类型段 = 记录类型那一支、孩子其实是码 2 节点（攻方 N4-f 那一形）：[`record_a_tombstone_tree_grown_to_level_two`] 的流上
/// 层级 2 的墓碑树根第一条条目类型段 0 → 1、重封到根。条目说它指一片墓碑容器，孩子是层级 1 的码 2 节点：
/// CPU 只读孩子的单元头得码 2、与类型段 1 不符，判 I-9.23 红、按孩子自述的类登记一槽引用、不往下走、不按数据单元长度去读、不记走读失败
/// GPU 走读内核同判同登记。
/// `turn_the_root_ring_over` 是攻方 N4-j 那一形：同一刀落在根环转过一圈之后，那个节点下面的容器没有别的版本再引用。
fn record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
    tag: &str,
    name: &'static str,
    turn_the_root_ring_over: bool,
) -> RecordedFlow {
    let mut flow = record_a_tombstone_tree_grown_to_level_two(tag, name, turn_the_root_ring_over);
    let targets = last_unit_copies(&flow, name, &is_a_tombstone_tree_root_of_level_two, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        set_the_type_of_the_first_entry(bytes, PACKED_RECORD_TYPE_TOMBSTONE);
    });
    flow
}

/// inode 记录里对象出生代那 8 字节的偏移（D18（块里携带什么信息） 已定项 16 的 inode 记录字段表：inode 号 0、对象出生代 8）。
pub(super) const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;

/// 实例表单元的打包记录类型（D18（块里携带什么信息） 已定项 11）。
pub(super) const PACKED_RECORD_TYPE_INSTANCE_TABLE: u16 = 4;

/// 翻坏一个单元的载荷时翻的那一字节离单元末尾几字节（`checker_known_bad_images.rs` 的 `break_both_copies_of_the_unit` 同一处）：
/// 不重封，位置条目里的整单元校验和对不上，两份都读不出。
pub(super) const BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE: usize = 100;

/// 墓碑与谱系那九条（I-9.24 在删除态扫描里判；I-9.22 崩溃点重放恒不适用）。
pub(super) const TOMBSTONE_AND_LINEAGE_INVARIANTS: [&str; 9] = [
    "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.24", "I-9.29",
];

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_lineage_root_entry_names_another_container_than_its_child(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_lineage_root_entry_names_another_container_than_its_child",
        &record_rollback_with_the_lineage_root_entry_naming_another_container(
            "crash-judge-gpu-i917-lineage-entry-container",
        ),
        "I-9.17",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type(
            "crash-judge-gpu-i923-tombstone-entry-type-seven",
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity()
{
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero(
            "crash-judge-gpu-i923-tombstone-entry-type-zero",
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_queued_inode_has_no_tombstone_on_the_lineage() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_queued_inode_has_no_tombstone_on_the_lineage",
        &record_deletion_with_the_tombstone_naming_an_object_without_a_record(
            "crash-judge-gpu-i918-tombstone-object",
        ),
        "I-9.18",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_names_a_live_object() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_names_a_live_object",
        &record_deletion_with_the_queued_record_back_at_nlink_one("crash-judge-gpu-i919-nlink-one"),
        "I-9.19",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_dies_before_its_object_is_born() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_dies_before_its_object_is_born",
        &record_deletion_with_the_tombstone_dying_at_txg_zero(
            "crash-judge-gpu-i920-death-txg-zero",
        ),
        "I-9.20",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_is_written_twice() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_is_written_twice",
        &record_deletion_with_the_tombstone_written_twice("crash-judge-gpu-i920-tombstone-twice"),
        "I-9.20",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record",
        &record_deletion_with_the_tombstone_object_birth_off_its_record(
            "crash-judge-gpu-i929-tombstone-object-birth",
        ),
        &["I-9.29", "I-9.10"],
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full",
        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
            "crash-judge-gpu-i921-one-tombstone-per-container",
        ),
        "I-9.21",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order",
        &record_new_pool_with_the_two_records_of_the_inode_leaf_swapped(
            "crash-judge-gpu-i94-records-swapped",
        ),
        "I-9.4",
    );
}

/// I-9.12 第三句（第 i 条的分隔 key 大于第 i−1 个孩子的最大 key）：新池新建文件之外再建 233 个空 inode（两片叶 [1..=233] 与 [234]）的流上，
/// inode 树根（最后写的那两份）右片那条条目的分隔 key 234 → 200、重封到根（判决 `research/prompts/m4-close-code-r3-main-verification.md` 第三节第 1 条，
/// 攻方 `separator_200` 那一形）。记录照叶序严格升序、都不小于分隔 key 与容器号：CPU 池级 checker 判 I-9.12 红，四个读者拒在 I-9.12；
/// GPU 恢复内核三样读者（冷走读、只读挂载的打开、可写挂载的建上一版，`crash_judge_common.wgsl` 与 `crash_judge_recovery.wgsl` 的
/// `inode_records_of_the_leaf_ascend_after_the_leaves_before`）同判。判别力：`crates/mutations.tsv` 里 GPU 那一判不比分隔 key 那一行，GPU 照读出文件、
/// 与 CPU 不一致，这一条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_separator_key_of_the_inode_root_is_not_above_the_leaf_before_it(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_when_a_separator_key_of_the_inode_root_is_not_above_the_leaf_before_it",
        &record_two_inode_leaves_with_the_separator_key_of_the_right_leaf_lowered(
            "crash-judge-gpu-i912-separator-lowered",
        ),
        "I-9.12",
    );
}

/// 改低之后右片那条条目的分隔 key：不大于左片最大号 233（I-9.12 第三句红），仍不大于右片最小号 234（I-9.4 第一句不红）。
pub(super) const SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_LOWERED: u64 = 200;

/// 改高之后右片那条条目的分隔 key：大于右片头一条记录号 234（I-9.4 第一句红），也大于左片最大号 233（I-9.12 第三句不红）。
pub(super) const SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_RAISED: u64 = 235;

/// [`the_gpu_judge_agrees_with_the_cpu_when_a_separator_key_of_the_inode_root_is_not_above_the_leaf_before_it`] 那条流：两片 inode 叶，
/// inode 树根右片那条条目的分隔 key 改成 [`SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_LOWERED`]、重封到根。
fn record_two_inode_leaves_with_the_separator_key_of_the_right_leaf_lowered(
    tag: &str,
) -> RecordedFlow {
    const NAME: &str = "two-inode-leaves-separator-key-of-the-right-leaf-lowered";
    let pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
    let leaves: Vec<Vec<u64>> = pool
        .output
        .inode_leaf_containers
        .iter()
        .map(|container| {
            container
                .contents
                .records
                .iter()
                .map(|record| record.inode)
                .collect()
        })
        .collect();
    assert_eq!(
        leaves,
        vec![
            (1..=INODE_RECORDS_PER_LEAF).collect::<Vec<u64>>(),
            vec![INODE_RECORDS_PER_LEAF + 1]
        ],
        "前提：234 条记录装成两片叶 [1..=233] 与 [234]"
    );
    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
    let mut flow = recorded_flow_of_the_pool(&pool, NAME, versions);
    let inode_tree_root_over_leaf_containers = |bytes: &[u8]| {
        unit_class(bytes) == 2
            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
            && index_node_entry_count(bytes) == 2
            && read_u16_at(
                bytes,
                index_node_entries_start(INODE_TREE_KEY_BYTES)
                    + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
            ) == PACKED_RECORD_TYPE_INODE
    };
    let targets = last_unit_copies(&flow, NAME, &inode_tree_root_over_leaf_containers, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let separator_of_the_right_leaf =
            first_entry_of_a_container_number_node() + usize::from(INODE_INTERNAL_ENTRY_BYTES);
        assert_eq!(
            read_u64_at(bytes, separator_of_the_right_leaf),
            INODE_RECORDS_PER_LEAF + 1,
            "改之前右片那条条目的分隔 key 是它的容器号 234"
        );
        bytes[separator_of_the_right_leaf..separator_of_the_right_leaf + 8]
            .copy_from_slice(&SEPARATOR_KEY_OF_THE_RIGHT_INODE_LEAF_LOWERED.to_le_bytes());
    });
    flow
}

/// 恢复那一半整份比（看 journal 与不看那一遍落到的根与结局）：零条目的 extent 上段根条目宽窄于字段表，两边都答没有文件。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_empty_extent_root_narrower_than_its_field_table_holds_no_file(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_an_empty_extent_root_narrower_than_its_field_table_holds_no_file",
        &record_deletion_through_the_last_batch_with_a_narrow_empty_extent_root(
            "crash-judge-gpu-empty-extent-root-narrow",
        ),
        "I-1.10",
    );
}

/// 同上，零条目的 inode 树根。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_empty_inode_root_narrower_than_its_field_table_holds_no_file(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_an_empty_inode_root_narrower_than_its_field_table_holds_no_file",
        &record_deletion_through_the_last_batch_with_a_narrow_empty_inode_root(
            "crash-judge-gpu-empty-inode-root-narrow",
        ),
        "I-1.10",
    );
}

/// 恢复那一半整份比：extent 上段叶条目宽于字段表（113 → 114）的镜像上，两边都拒（CPU 冷走读报 `EntryWidthDiffersFromItsFieldTable`，
/// GPU 恢复内核同一个失败点）；CPU 池级 checker 判 I-1.10 红。判别力：`crates/mutations.tsv`「根环一簇 R9」那一行（GPU 读上段那一判改回
/// 只拒窄于），GPU 照读出文件、与 CPU 不一致，这一条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_an_extent_upper_leaf_wider_than_its_field_table_is_refused(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
        "the_gpu_judge_agrees_with_the_cpu_that_an_extent_upper_leaf_wider_than_its_field_table_is_refused",
        &record_new_pool_with_the_extent_upper_leaf_entries_one_byte_wider(
            "crash-judge-gpu-extent-upper-leaf-wider",
        ),
        "I-1.10",
    );
}

/// deadlist 叶条目字段表的宽：头的树 ID 8 + 纪元 8 + 中央映射 key 27，value 空（D5（快照 / 空间记账机制） 已定项 12）。写死在这里，
/// 不从 checker 或实现里取：取了的话，两边把宽挪了这条流也跟着走。
const DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES: u16 = 43;

/// 新池新建文件（txg 3）、建快照（txg 4）、快照下覆盖写（txg 5：换下的单元进 deadlist，deadlist 树根兼叶装着条目）的流上，
/// 覆盖写那一版的 deadlist 树根每条条目末尾补一个 0、条目宽 43 → 44，按原来的头重建、重封到根（I-1.10（码 2 条目宽等于字段表宽） 写「等于」）。
/// 核心层冷走读读 deadlist 树只数叶条目、不逐条解（`recovery::walk_to_file_judging_the_truncation_fields_as` 的 deadlist 树那一支）；
/// 叶条目宽不等于 43 就拒的那一判（C867，`recovery.rs` 的 `deadlist_entries_parsed`）在建上一版里，可写挂载那一样读者才走到。
/// GPU 恢复内核照冷走读只数不判（`crash_judge_common.wgsl` 的 `read_code_two_tree` deadlist 那一支）。池级 checker 两边都按字段表判 I-1.10 红。
fn record_new_pool_through_a_snapshot_and_an_overwrite_under_it_with_the_deadlist_leaf_entries_one_byte_wider(
    tag: &str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![published_version_of(&pool.output, file_content())];
    let created = create_a_snapshot_and_advance(&mut pool);
    versions.push(published_version_of(&created, file_content()));
    let overwritten = overwrite_under_the_snapshot_and_advance(&mut pool);
    assert!(
        !overwritten.deadlist_entries.is_empty(),
        "前提：快照下覆盖写换下的用户可见单元进 deadlist"
    );
    versions.push(published_version_of(
        &overwritten,
        content_overwritten_under_the_snapshot(),
    ));
    let mut flow = recorded_flow_of_the_pool(
        &pool,
        "new-pool-snapshot-overwrite-deadlist-leaf-entries-one-byte-wider",
        versions,
    );
    let deadlist_key_width = u8::try_from(DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES).expect("43");
    let targets = last_unit_copies(
        &flow,
        "deadlist 树根兼叶（key 宽 43、条目宽 43、有条目）",
        &|bytes| {
            unit_class(bytes) == 2
                && index_node_key_width(bytes) == deadlist_key_width
                && index_node_entry_width(bytes) == DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES
                && index_node_entry_count(bytes) > 0
        },
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
        let leaf = singlefs_core::unit::parse_index_node(bytes).expect("deadlist 树根解得开");
        assert_eq!(leaf.level, 0, "前提：deadlist 树根兼叶");
        let wider_entries: Vec<Vec<u8>> = leaf
            .entries
            .iter()
            .map(|entry| {
                let mut wider = entry.clone();
                wider.push(0);
                wider
            })
            .collect();
        let wider_leaf = singlefs_core::unit::build_index_node(
            leaf.tree,
            leaf.level,
            leaf.key_width,
            &leaf.smallest_key,
            &leaf.largest_key,
            leaf.birth_txg,
            &parameters().filesystem_identifier,
            leaf.instance,
            leaf.birth_sequence,
            DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES + 1,
            &wider_entries,
        );
        bytes.copy_from_slice(&wider_leaf);
    });
    flow
}

/// 恢复那一半整份比：deadlist 叶条目宽于字段表（43 → 44）的流上，每个状态两边两遍恢复的结局、失败点与落到的根都相同；全部写都落了的那个状态
/// CPU 看 journal 那一遍读出覆盖写那一版的文件（冷走读只数 deadlist 叶条目，C867 那一判在建上一版里，Q5 前提）；CPU 池级 checker 判 I-1.10 红。
/// 判别力：`crates/mutations.tsv`「GPU 恢复内核读 deadlist 叶条目时宽不等于 43 就拒」那一行（照可写挂载的建上一版判、不照冷走读），
/// GPU 那一遍在全部写都落了的状态上恢复失败、与 CPU 不一致，这一条红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_recovery_that_a_deadlist_leaf_wider_than_its_field_table_still_reads_the_file(
) {
    const TEST_NAME: &str =
        "the_gpu_judge_agrees_with_the_cpu_recovery_that_a_deadlist_leaf_wider_than_its_field_table_still_reads_the_file";
    let flow =
        record_new_pool_through_a_snapshot_and_an_overwrite_under_it_with_the_deadlist_leaf_entries_one_byte_wider(
            "crash-judge-gpu-deadlist-leaf-wider",
        );
    let overwrite_version = flow
        .versions
        .last()
        .map(|version| (version.instance.0, version.checkpoint_txg.0))
        .expect("流里有覆盖写那一版");
    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-1.10"),
        "流 {} 上 CPU 池级 checker 要红 I-1.10（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    let every_write_persisted = comparison
        .cpu_recovery_words_of_the_last_state
        .as_ref()
        .expect("流至少一个状态");
    assert_eq!(
        (
            consulted_landed_root_of(*every_write_persisted),
            every_write_persisted.consulted_pass_outcome
        ),
        (Some(overwrite_version), RECOVERY_WORD_CLASS_FILE_READ),
        "前提：全部写都落了的状态上 CPU 看 journal 那一遍落在覆盖写那一版、读出文件（冷走读不判 deadlist 叶条目宽）"
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// 墓碑树、谱系树条目的类型段与孩子的真实类不符的流（攻方 N4 那几形）：CPU 池级 checker 红 `identity_invariant`，
/// [`INVARIANTS_A_WALK_FAILURE_REDDENS`] 一个状态都不红（孩子的类标签只读单元头、不按类型段定读多长，不符就判身份、不往下走、不记走读失败，
/// 判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行）；GPU 每个状态逐位相同。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
    test_name: &str,
    flow: &RecordedFlow,
    identity_invariant: &'static str,
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison
            .cpu_checker_red_invariants
            .contains(identity_invariant),
        "流 {} 上 CPU 池级 checker 要红 {identity_invariant}（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    for invariant in INVARIANTS_A_WALK_FAILURE_REDDENS {
        assert!(
            !comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上身份不符不记走读失败，CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_container(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_container",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity(
            "crash-judge-gpu-i923-tombstone-entry-type-zero-no-identity",
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root",
        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
            "crash-judge-gpu-i923-level-two-entry-names-a-node",
            "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container",
            false,
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over",
        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container(
            "crash-judge-gpu-i923-level-two-entry-names-a-node-ring-turned-over",
            "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container-ring-turned-over",
            true,
        ),
        "I-9.23",
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container()
{
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container",
        &record_new_pool_with_the_inode_root_entry_of_type_zero_naming_its_leaf_container(
            "crash-judge-gpu-i92-inode-root-entry-type-zero",
        ),
        "I-9.2",
    );
}

/// H1 那一形的流（[`record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale`]）：
/// CPU 池级 checker 红 I-9.2、一个状态都不红 I-7.2（只认第一份陈旧副本的头的话，I-9.2 报成立，记走读失败让最新根走不完、I-7.2 连带红）；
/// 陈旧副本自己连带的 I-2.1 那几条照红，不在这里钉；GPU 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale";
    let flow =
        record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale(
            "crash-judge-gpu-i92-inode-root-entry-type-zero-first-leaf-copy-stale",
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.2"),
        "流 {} 上 CPU 池级 checker 要红 I-9.2（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-7.2"),
        "流 {} 上第一份陈旧副本不挡住第二份：CPU 池级 checker 不红 I-7.2（红过的：{:?}）",
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

/// 单元头里类标签的偏移。
const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;

/// 不在登记表里的单元类标签（登记的只有 1、2、3）：孩子头里写成它之后，按哪一类的长度都认不出它是哪一类。
const UNIT_CLASS_NOT_REGISTERED: u8 = 9;

/// inode 树根（层级 1，条目指叶容器）：码 2、key 宽 8、条目宽 120、第一条条目的类型段是 2。
pub(super) fn is_the_inode_tree_root_over_leaf_containers(bytes: &[u8]) -> bool {
    unit_class(bytes) == 2
        && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
        && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
        && read_u16_at(
            bytes,
            index_node_entries_start(INODE_TREE_KEY_BYTES)
                + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
        ) == PACKED_RECORD_TYPE_INODE
}

/// inode 树读不出孩子的那一形（
/// harness 坏镜像 `image_whose_inode_root_entry_names_a_child_that_does_not_read_back` 同一形，录成流）：新池新建文件的流上 inode 叶容器两份
/// 头里的类标签 3 → 9、不重封（[`corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root`] 取
/// [`OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange`]：指着它的位置条目换成改后的整单元 CRC、一路重封到根），
/// 再把 inode 树根第一条条目的类型段 2 → `entry_type`、重封到根（身份引用不动）。孩子头里的类标签不登记，按哪一类都认不出；
/// 类型段 0 按 16 KiB 去读，校验和对不上。
fn record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
    tag: &str,
    name: &'static str,
    entry_type: u16,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    let leaf_slot = flow.writes[leaf_copies[0]].offset.0 / SLOT_BYTES;
    corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
        &mut flow,
        &leaf_copies,
        &|bytes| bytes[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED,
        OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange,
    );
    let root_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    let root = flow.writes[root_copies[0]].bytes().expect("单元写带字节");
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &root[first_entry_of_a_container_number_node()
                + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[0],
        )
        .1,
        leaf_slot,
        "inode 树根第一条条目指着的就是改了类标签的那片叶容器"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, entry_type);
    });
    flow
}

/// inode 树根条目类型段 0、身份引用三段不为 0（容器号、出生代照旧），孩子读不出（类标签改成不登记的码、不重封）：
/// CPU 池级 checker 按 16 KiB 读孩子读不出，I-9.2 里不看孩子就判得了的身份三段照判、红（`judge_an_inode_entry_whose_child_does_not_read_back`），
/// I-2.1 红、记走读失败；GPU 走读内核读不出那一支同判（`w_walk_inode_root` 那一支），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back",
        &record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
            "crash-judge-gpu-i92-inode-root-entry-type-zero-unreadable-child",
            "new-pool-inode-root-entry-type-zero-over-an-unreadable-child",
            0,
        ),
        &["I-9.2", "I-2.1"],
    );
}

/// inode 树根条目类型段 2 → 7（不在 {0, 2} 里），孩子读不出（同上一条的叶容器）：CPU 池级 checker 不往下走孩子，
/// 按层级 1 下面的叶容器两槽登记引用、判 I-9.2 红，不记 I-2.1、不记走读失败（与墓碑树、谱系树类型段不认得那一支同判）；
/// GPU 走读内核同改，每个状态逐位相同（照旧按 16 KiB 读孩子的话 GPU 红 I-2.1、记走读失败，CPU 不红）。
/// 「不记 I-2.1」只对这一形成立：这一支
/// 每一份副本照判 I-2.1，头里的类标签 9 不登记、16 KiB 与 32 KiB 两种长度都比，这片叶容器的位置条目是改后的整单元 CRC，按 32 KiB 对得上；
/// 位置条目里的校验和本身坏了的那一形见
/// [`the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong`]。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back",
        &record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
            "crash-judge-gpu-i92-inode-root-entry-type-seven-unreadable-child",
            "new-pool-inode-root-entry-type-seven-over-an-unreadable-child",
            UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE,
        ),
        "I-9.2",
    );
}
