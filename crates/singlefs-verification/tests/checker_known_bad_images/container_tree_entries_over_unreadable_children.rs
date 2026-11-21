//! 子节点读不回，或有一份副本按节点校验得过时，墓碑树、谱系树、inode 树的条目不变量怎么判。

use super::*;

/// 不在登记表里的类标签：孩子头里写成它之后，按哪一类的长度都自证不过，两份都读不出。
const UNIT_CLASS_NOT_REGISTERED: u8 = 9;

/// I-1.11：映射条目 key 里的类标签与它指的单元头里的类标签相等（叶容器头里的类标签改成 [`UNIT_CLASS_NOT_REGISTERED`] 自己就带它红）。
const MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER: &str = "I-1.11";

/// 新池新建文件那一版（`image`）：inode 叶容器头里的类标签 3 → 9（不重封，整单元校验和沿引用链补上），再照 `change` 改 inode 树根第一条条目、重封到根。
/// 孩子按自己的类标签认不出、按类型段的长度读不出：类型段认得的，读不出归 I-2.1 与走读失败。
fn image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
    mut image: MemoryPool,
    change: impl FnOnce(&mut [u8]),
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    assert_eq!(
        common_pointer_head_corruption::the_slot_of(&child).0,
        INODE_LEAF,
        "inode 树根第一条条目指着那片叶容器"
    );
    mutate_unit(
        &mut image,
        INODE_LEAF,
        |bytes| bytes[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED,
        false,
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, change);
    image
}

/// inode 树读不出孩子时 I-9.2 的判法与墓碑树、谱系树对齐（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行，
/// 正推 144 格矩阵里 inode 树不判的那 9 格）：inode 树根第一条条目类型段 2 → 0、身份三段不动（容器号、出生代不为 0），孩子读不出。
/// 「子单元是码 2」那一小句孩子读不出不判，身份三段不为 0 照判：I-9.2 红（读不出不记走读失败、不跳过这条条目）。
#[test]
fn an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
        build_pool("known-bad-inode-entry-type-zero-identity-unreadable-child").memory_pool(),
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_INODE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
        },
    );
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail)
                if detail.contains("孩子两份都读不出") && detail.contains("（要三段为 0）")
        ),
        "类型段 0 的条目身份三段不为 0、孩子读不出：I-9.2 照判身份红：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// 同上一条，条目的类型段 2 → 7（不在 {0, 2} 里）、孩子读不出：判该条目损坏，与墓碑树、谱系树类型段不认得那一支同判：
/// 不按 16 KiB 去读孩子，孩子按层级 1 下面的叶容器两槽登记引用。
/// 对照：同一张新池镜像只把这条条目的类型段改成 7、孩子完好（读得出、自证出码 3，按它自己那一类两槽登记引用、判 I-9.2 红、不往下走）。
/// 孩子读不出的这一张红的不变量 = 对照所红（I-9.2）∪ 叶容器类标签改成 9 那一改自己带的 I-1.11（中央映射条目 key 里的类标签 3 与单元头不符，
/// 与 inode 树怎么走无关）：不连带 I-2.1、不记走读失败（也就不连带 I-4.8、I-7.2、I-7.4），
/// 也不连带 I-3.1（孩子占的两槽都进引用并集）。
/// 「不连带 I-2.1」只对这一张镜像成立：这一支
/// 每一份副本照判 I-2.1，头里的类标签 9 不登记、16 KiB 与 32 KiB 两种长度都比，这张镜像的叶容器类标签改坏、整单元校验和沿引用链补齐了，
/// 按 32 KiB 对得上；位置条目里的校验和本身坏了的那一形见
/// [`an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`]。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant(
) {
    let clean =
        build_pool("known-bad-inode-entry-unregistered-type-unreadable-child").memory_pool();
    let change_to_an_unregistered_type = |entry: &mut [u8]| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
    };
    let mut image_with_a_readable_child = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_a_readable_child,
        TREE_KIND_INODE,
        0,
        change_to_an_unregistered_type,
    );
    let verdicts_with_a_readable_child = check_pool_image(&image_with_a_readable_child);
    assert_eq!(
        violated_invariants(&verdicts_with_a_readable_child),
        ["I-9.2"],
        "对照：类型段 7 的条目指着完好的叶容器，只红 I-9.2：{verdicts_with_a_readable_child:?}"
    );
    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
        clean,
        change_to_an_unregistered_type,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [
            MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER,
            violated_invariants(&verdicts_with_a_readable_child)[0]
        ],
        "类型段不认得、孩子读不出：红的 = 孩子完好的对照所红（I-9.2）∪ 类标签那一改自己带的 I-1.11，不连带 I-2.1、走读失败那几条与 I-3.1：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict(&image, MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER),
            InvariantVerdict::Violated(detail)
                if detail.contains("它指的单元头里的类标签是 9")
        ),
        "I-1.11 红在映射条目 key 里的类标签与改成 9 的叶容器头不符（类标签那一改自己带的，不是走 inode 树带出来的）：{:?}",
        verdict(&image, MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER)
    );
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail)
                if detail.contains("类型段 7 不在 {0, 2} 里（孩子没有一份按它头里那一类的长度自证过，不往下走）")
        ),
        "违例说明写出不认得的类型段、孩子不往下走：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// 内部条目（120 字节）子指针里第 `location_index` 条位置条目的整单元校验和第 0 字节翻一下（位置条目 = 盘 4 + 槽 6 + 整单元校验和 4，
/// 子指针从条目偏移 [`CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET`] 起）：孩子本身不动，那一份按哪一种长度都与位置条目里的校验和对不上。
pub(super) fn flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(
    entry: &mut [u8],
    location_index: usize,
) {
    let checksum_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + location_index * POINTER_LOCATION_BYTES
        + LOCATION_CHECKSUM_OFFSET;
    entry[checksum_offset] ^= 0xff;
}

/// 走读不往下读的孩子每一份副本照判 I-2.1（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a、H-J3b 两行与第三节第 1 条，
/// `crates/singlefs-checker/invariants/04-校验和.md` I-2.1「任一被引用的块，其校验和与内容匹配」）：`image` 红的恰好是 `expected_violations`，
/// I-2.1 的违例说明点名「走读不往下读的孩子」（红在新判的那一处，不是别的单元）；`expected_violations` 里有 I-7.2 的，
/// 它的违例说明点名两份都对不上（走读失败是这一处记的）。
pub(super) fn assert_the_child_the_walk_does_not_go_into_reddens_exactly(
    image: &MemoryPool,
    expected_violations: &[&'static str],
    scene: &str,
) {
    let verdicts = check_pool_image(image);
    let verdict_of = |invariant: &str| {
        verdicts
            .iter()
            .find(|(identifier, _)| *identifier == invariant)
            .expect("清单里有")
            .1
            .clone()
    };
    assert_eq!(
        violated_invariant_set(&verdicts),
        expected_violations
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<&'static str>>(),
        "{scene}：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict_of("I-2.1"),
            InvariantVerdict::Violated(detail) if detail.contains("走读不往下读的孩子")
        ),
        "{scene}：I-2.1 红在走读不往下读的那个孩子上：{:?}",
        verdict_of("I-2.1")
    );
    if expected_violations.contains(&"I-7.2") {
        assert!(
            matches!(
                verdict_of("I-7.2"),
                InvariantVerdict::Violated(detail)
                    if detail.contains("（走读不往下读的孩子）两份都与位置条目里的校验和对不上")
            ),
            "{scene}：I-7.2 红在走读不往下读的孩子两份都对不上：{:?}",
            verdict_of("I-7.2")
        );
    }
}

/// 攻方 B1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行，`research/prompts/m3-batch27-code-r1-opus-output.md`
/// 第一节）：新池新建文件那一版，inode 树根（层级 1）第一条条目的类型段 2 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；
/// 叶容器不动。孩子不往下走，两份副本照判 I-2.1、都对不上、记走读失败：红 I-9.2、I-2.1、I-4.8、I-7.2、I-7.4。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    let mut image =
        build_pool("known-bad-inode-entry-unregistered-type-both-location-checksums-wrong")
            .memory_pool();
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 1, "inode 树根直接管叶容器");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-9.2", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "inode 树根条目类型段 7、子指针两条位置条目的校验和都坏",
    );
}

/// 同上，墓碑树那一侧（I-9.23）：三个单元的文件发完意图之后，墓碑树根（层级 1）
/// 第一条条目的类型段 1 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；墓碑容器不动。
#[test]
fn a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-unregistered-type-both-location-checksums-wrong",
    );
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 1, "墓碑树根直接管容器");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &[
            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            "I-2.1",
            "I-4.8",
            "I-7.2",
            "I-7.4",
        ],
        "墓碑树根条目类型段 7、子指针两条位置条目的校验和都坏",
    );
}

/// 同上，谱系树那一侧（I-9.17）：回退到 A 之后，谱系树根（层级 1）第一条条目的类型段 5 → 7、子指针两条位置条目的整单元校验和各翻一字节，
/// 重封到根；谱系容器不动。
#[test]
fn a_lineage_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    let mut image = image_after_rolling_back_to_the_first_root(
        "known-bad-lineage-entry-unregistered-type-both-location-checksums-wrong",
    )
    .image;
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_LINEAGE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 1, "谱系树根直接管容器");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &[
            LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            "I-2.1",
            "I-4.8",
            "I-7.2",
            "I-7.4",
        ],
        "谱系树根条目类型段 7、子指针两条位置条目的校验和都坏",
    );
}

/// H-J3b：新池新建文件那一版，
/// inode 树根（层级 1）第一条条目的类型段 2 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；叶容器不动。
/// 第二份自证成码 3（类型段 0 说的是码 2），走「孩子是另一类」那一支：I-9.2 红、不往下走；第一份按哪一种长度都对不上，照判 I-2.1 红，
/// I-4.8、I-7.4 跟着；第二份对得上，不记走读失败、I-7.2 不红。
#[test]
fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant(
) {
    let mut image =
        build_pool("known-bad-inode-entry-type-zero-over-a-leaf-first-location-checksum-wrong")
            .memory_pool();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
    });
    assert_the_violation_names_the_child_class(
        &image,
        "I-9.2",
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
        "inode 树根条目类型段 0 指着叶容器、子指针第一条位置条目的校验和坏",
    );
}

/// H-J3b 墓碑树那一侧（`Walk::judge_a_container_tree_entry_whose_child_is_of_another_class`，墓碑树与谱系树共用）：三个单元的文件发完意图之后，
/// 墓碑树根（层级 1）第一条条目的类型段 1 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；墓碑容器不动。
#[test]
fn a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant(
) {
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-type-zero-over-a-container-first-location-checksum-wrong",
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
    });
    assert_the_violation_names_the_child_class(
        &image,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &[
            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            "I-2.1",
            "I-4.8",
            "I-7.4",
        ],
        "墓碑树根条目类型段 0 指着容器、子指针第一条位置条目的校验和坏",
    );
}

/// 读不出孩子那两条坏镜像补「只改身份一段」三形：
/// inode 树根第一条条目类型段 2 → 0、身份引用三段先清零再只把 `field` 那一段写成 1，孩子读不出（同
/// [`an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant`]）。
/// 三段里只有一段不为 0，I-9.2 照判身份红，违例说明写出的三段就是 `expected_three_fields`（「出生树 / 容器号 / 出生代」）。
fn assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
    tag: &str,
    field: std::ops::Range<usize>,
    expected_three_fields: &str,
) {
    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
        build_pool(tag).memory_pool(),
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_INODE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
            entry[field].copy_from_slice(&1u64.to_le_bytes());
        },
    );
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail)
                if detail.contains("孩子两份都读不出")
                    && detail.contains(&format!("{expected_three_fields}（要三段为 0）"))
        ),
        "类型段 0 的条目身份只有一段不为 0（{expected_three_fields}）、孩子读不出：I-9.2 照判身份红：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// 条目身份引用里容器出生代那 8 字节（容器号 18..26 之后）。
const INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH: std::ops::Range<usize> = 26..34;

#[test]
fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_birth_tree_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
        "known-bad-inode-entry-type-zero-only-birth-tree-unreadable-child",
        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
        "1 / 0 / 0",
    );
}

#[test]
fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_container_number_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
        "known-bad-inode-entry-type-zero-only-container-number-unreadable-child",
        CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET..INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH.start,
        "0 / 1 / 0",
    );
}

#[test]
fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_container_birth_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
        "known-bad-inode-entry-type-zero-only-container-birth-unreadable-child",
        INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH,
        "0 / 0 / 1",
    );
}

/// 条目里子指针头部出生树那 8 字节：子指针在条目偏移 34 起，出生树在指针头部偏移 34（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）。
pub(super) const INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE: std::ops::Range<usize> =
    CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + 34..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + 42;

/// 条目里 `field` 那 8 字节（出生树那两段之一）加 1。
pub(super) fn add_one_to_a_birth_tree_field_of_the_entry(
    entry: &mut [u8],
    field: std::ops::Range<usize>,
) {
    let birth_tree = get_u64(entry, field.start);
    entry[field].copy_from_slice(&(birth_tree + 1).to_le_bytes());
}

/// 种类 `kind` 那棵树的根（层级 1）第一条条目指着的码 3 容器两份各翻一个载荷字节（不重封、不补引用链：两份都读不出），条目不改。
fn image_whose_first_container_of_the_root_of_kind_does_not_read_back(
    clean: &MemoryPool,
    kind: u16,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let mut image = clean.clone();
    break_both_copies_of_the_unit(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&child).0,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    image
}

/// H-B（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条；攻方
/// `research/prompts/m3-batch26-code-r1-opus-output.md` 第 2.2 节 U 段 BT / IDB 两种改法）的判法：种类 `kind` 那棵树的根（层级 1）第一条条目
/// 类型段是容器那一类（`container_entry_type`），它指的容器两份都读不出。对照（条目不改）：`entry_invariant` 不红——条目其余都对，
/// 「子指针头部的出生树 == 身份引用的出生树」成立，比孩子头的那几小句读不出不判（正推线索③：不过判）。
/// 再把条目里 `field`（子指针头部出生树或身份引用出生树）加 1：那一小句只比父条目里的两段、不看孩子，`entry_invariant` 红，
/// 违例说明写出两段出生树（读不出孩子时也判，`entry_invariant` 不报成立）。
fn assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
    clean: &MemoryPool,
    kind: u16,
    container_entry_type: u16,
    field: std::ops::Range<usize>,
    entry_invariant: &'static str,
) {
    let unreadable_child_only =
        image_whose_first_container_of_the_root_of_kind_does_not_read_back(clean, kind);
    assert!(
        !matches!(
            verdict(&unreadable_child_only, entry_invariant),
            InvariantVerdict::Violated(_)
        ),
        "对照：条目不改、容器两份都读不出，{entry_invariant} 不红：{:?}",
        verdict(&unreadable_child_only, entry_invariant)
    );
    let mut image = unreadable_child_only;
    let mut birth_trees_after_the_change = (0, 0);
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        assert_eq!(
            entry_type_of(entry),
            container_entry_type,
            "根第一条条目指容器"
        );
        add_one_to_a_birth_tree_field_of_the_entry(entry, field);
        birth_trees_after_the_change = (
            get_u64(entry, INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE.start),
            get_u64(entry, INTERNAL_ENTRY_IDENTITY_BIRTH_TREE.start),
        );
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    let (pointer_birth_tree, identity_birth_tree) = birth_trees_after_the_change;
    assert!(
        matches!(
            verdict(&image, entry_invariant),
            InvariantVerdict::Violated(detail)
                if detail.contains("两份都读不出")
                    && detail.contains(&format!(
                        "子指针头部的出生树 {pointer_birth_tree} 不等于身份引用的出生树 {identity_birth_tree}"
                    ))
        ),
        "出生树那两段不等、容器两份都读不出：{entry_invariant} 照判那一小句、红：{:?}",
        verdict(&image, entry_invariant)
    );
}

#[test]
fn an_inode_tree_entry_whose_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
        &build_pool("known-bad-inode-entry-identity-birth-tree-unreadable-leaf").memory_pool(),
        TREE_KIND_INODE,
        PACKED_RECORD_TYPE_INODE,
        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
        "I-9.2",
    );
}

#[test]
fn a_tombstone_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_tombstone_entry_invariant(
) {
    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-pointer-birth-tree-unreadable-container",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

#[test]
fn a_lineage_tree_entry_whose_identity_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_lineage_entry_invariant(
) {
    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-identity-birth-tree-unreadable-container",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

/// 正推线索③（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 3 条「补一条类型段 2 读不出孩子、I-9.2 不判的坏镜像（钉住不过判）」）：
/// 新池新建文件那一版，inode 树根第一条条目不改（类型段 2、两段出生树相等），它指的叶容器两份都读不出。
/// 比孩子头的那几小句读不出不判，「子指针头部的出生树 == 身份引用的出生树」成立：I-9.2 不红（报成立或不判都行）。
#[test]
fn an_inode_tree_entry_of_the_leaf_container_type_with_agreeing_birth_trees_over_a_leaf_that_does_not_read_back_leaves_the_inode_entry_invariant_unviolated(
) {
    let image = image_whose_first_container_of_the_root_of_kind_does_not_read_back(
        &build_pool("known-bad-inode-entry-agreeing-birth-trees-unreadable-leaf").memory_pool(),
        TREE_KIND_INODE,
    );
    assert!(
        matches!(verdict(&image, "I-2.1"), InvariantVerdict::Violated(_)),
        "叶容器两份都读不出：I-2.1 红（钉住这张镜像真把孩子弄成读不出）：{:?}",
        verdict(&image, "I-2.1")
    );
    assert!(
        !matches!(verdict(&image, "I-9.2"), InvariantVerdict::Violated(_)),
        "类型段 2、两段出生树相等、孩子读不出：I-9.2 不过判：{:?}",
        verdict(&image, "I-9.2")
    );
}

/// H-V（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H1 H-V 那一行、第三节第 2 条；攻方
/// `research/prompts/m3-batch26-code-r1-opus-output.md` 第 1.3 节）的造法：种类 `kind` 那棵树的根（层级 1）第一条条目先照 `change_the_entry` 改，
/// 再把它子指针第 `location_index` 条位置条目的整单元校验和换成这棵树根（码 2）原样 16 KiB 的 CRC、重封到根，那块盘上孩子的槽起写进这 16 KiB；
/// 另一块盘那一份不动。两份按各自的位置条目都自证得过：一份码 2，一份是原来那片码 3 容器。
pub(super) fn image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
    clean: &MemoryPool,
    kind: u16,
    location_index: usize,
    change_the_entry: &dyn Fn(&mut [u8]),
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let root_bytes = root_node_bytes_of_the_tree_of_kind(clean, kind);
    assert_eq!(
        root_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
        "写进孩子槽的那 16 KiB 是码 2 节点"
    );
    let checksum_of_the_root_bytes = crc32_castagnoli(&root_bytes);
    let mut image = clean.clone();
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry(entry);
        let pointer_field = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
            ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES;
        let mut pointer = singlefs_core::pointer::NodePointer::read_from(
            &mut singlefs_core::bytes::ByteReader::at(entry, pointer_field.start),
        );
        pointer.locations[location_index].unit_checksum = checksum_of_the_root_bytes;
        entry[pointer_field].copy_from_slice(&common_pointer_head_corruption::node_pointer_bytes(
            &pointer,
        ));
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    write(
        &mut image,
        child.locations[location_index].device.0,
        common_pointer_head_corruption::the_slot_of(&child).0 * SLOT,
        &root_bytes,
    );
    image
}

/// H-V 的判法：两种盘序（码 2 那一份在位置条目 0 / 1）各判一次，`entry_invariant` 都红、违例说明点名孩子头里的
/// `class_named_in_the_violation`，两种盘序红的同一批（不是只看第一份自证过的副本：条目不改时码 2 那份在盘 1 上不能漏判，
/// 条目改成类型段 0 时码 2 那份在盘 0 上不能漏判、也不连带 I-7.2 等）。
fn assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
    clean: &MemoryPool,
    kind: u16,
    change_the_entry: &dyn Fn(&mut [u8]),
    entry_invariant: &'static str,
    class_named_in_the_violation: &str,
) {
    let mut violated_on_each_device = Vec::new();
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            clean,
            kind,
            location_index,
            change_the_entry,
        );
        let verdicts = check_pool_image(&image);
        let entry_verdict = verdicts
            .iter()
            .find(|(identifier, _)| *identifier == entry_invariant)
            .map(|(_, entry_verdict)| entry_verdict)
            .expect("清单里有");
        assert!(
            matches!(
                entry_verdict,
                InvariantVerdict::Violated(detail)
                    if detail.contains(&format!("孩子头里的类标签是 {class_named_in_the_violation}"))
            ),
            "码 2 那一份在位置条目 {location_index}：{entry_invariant} 红、说明点名孩子头里的 {class_named_in_the_violation}：{verdicts:?}"
        );
        violated_on_each_device.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_device[0], violated_on_each_device[1],
        "同一张镜像只差码 2 那一份在哪块盘：红的同一批"
    );
}

/// 条目不改那一形：类型段说码 3，一份自证成码 2。
pub(super) fn leave_the_entry_as_it_is(_entry: &mut [u8]) {}

#[test]
fn an_inode_tree_entry_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &build_pool("known-bad-inode-entry-leaf-copy-verifying-as-a-node").memory_pool(),
        TREE_KIND_INODE,
        &leave_the_entry_as_it_is,
        "I-9.2",
        "IndexNode",
    );
}

#[test]
fn an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &build_pool("known-bad-inode-entry-type-zero-leaf-copy-verifying-as-a-node").memory_pool(),
        TREE_KIND_INODE,
        &|entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_INODE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
        "I-9.2",
        "PackedRecordUnit",
    );
}

#[test]
fn a_tombstone_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-container-copy-verifying-as-a-node",
        ),
        TREE_KIND_TOMBSTONE,
        &leave_the_entry_as_it_is,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "IndexNode",
    );
}

#[test]
fn a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-container-copy-verifying-as-a-node",
        ),
        TREE_KIND_TOMBSTONE,
        &|entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_TOMBSTONE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "PackedRecordUnit",
    );
}

#[test]
fn a_lineage_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-container-copy-verifying-as-a-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        &leave_the_entry_as_it_is,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "IndexNode",
    );
}

#[test]
fn a_lineage_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device(
) {
    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-container-copy-verifying-as-a-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        &|entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_LINEAGE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "PackedRecordUnit",
    );
}

// ─── 删除线：H-V 子树（F-W）、inode 读不出孩子时子指针出生树大一、认孩子的类时第一份坏而第二份自证成别类 ───

/// F-W 的判法（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条；
/// 攻方 `research/prompts/m3-batch28-code-r1-opus-output.md` 第 1.1 节那几形）：`with_the_violation` 是 `clean` 上把种类 `kind` 那棵树根
/// 第一条条目指的那片码 3 容器两份一起改坏（重封、整单元校验和沿引用链补到根）的镜像，只它就红 `subtree_invariant`。
/// 再做 H-V 那一步（[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]，条目不改）：一份换成这棵树根的码 2 字节、
/// 按改过的位置条目自证得过，另一份就是那片带违例、按自己的位置条目自证得过的码 3 容器（类型段说的那一类，读者用的就是它）。
/// 两种盘序（码 2 那一份在位置条目 0 / 1）各判一次：`subtree_invariant` 红（子树照类型段那一支走、照判）、`entry_invariant` 红；
/// 槽数按类型段那一类登记，不连带 I-3.1、I-3.11、I-5.1；两种盘序红的同一批。
/// 对照：同一形只做 H-V、不放违例，`subtree_invariant` 不红（红的是那片容器里的违例，不是 H-V 那一步带出来的）。
/// 「孩子是另一类」那一支见别类照样往下走：`subtree_invariant` 两种盘序都红。
fn assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
    clean: &MemoryPool,
    with_the_violation: &MemoryPool,
    kind: u16,
    entry_invariant: &'static str,
    subtree_invariant: &'static str,
) {
    let violated_by_the_violation_only =
        violated_invariant_set(&check_pool_image(with_the_violation));
    assert!(
        violated_by_the_violation_only.contains(subtree_invariant),
        "对照：只放违例、不做 H-V，{subtree_invariant} 红（{violated_by_the_violation_only:?}）"
    );
    let mut violated_on_each_device = Vec::new();
    for location_index in 0..2 {
        let without_the_violation =
            image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
                clean,
                kind,
                location_index,
                &leave_the_entry_as_it_is,
            );
        let violated_without_the_violation =
            violated_invariant_set(&check_pool_image(&without_the_violation));
        assert!(
            !violated_without_the_violation.contains(subtree_invariant),
            "对照：码 2 那一份在位置条目 {location_index}、不放违例，{subtree_invariant} 不红（{violated_without_the_violation:?}）"
        );
        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            with_the_violation,
            kind,
            location_index,
            &leave_the_entry_as_it_is,
        );
        let verdicts = check_pool_image(&image);
        let violated = violated_invariant_set(&verdicts);
        assert!(
            violated.contains(subtree_invariant),
            "码 2 那一份在位置条目 {location_index}：类型段那一类那一份的子树照走，里面的违例 {subtree_invariant} 红：{verdicts:?}"
        );
        assert!(
            violated.contains(entry_invariant),
            "码 2 那一份在位置条目 {location_index}：条目那一条 {entry_invariant} 照判红：{verdicts:?}"
        );
        for invariant_a_wrong_slot_count_drags_in in ["I-3.1", "I-3.11", "I-5.1"] {
            assert!(
                !violated.contains(invariant_a_wrong_slot_count_drags_in),
                "码 2 那一份在位置条目 {location_index}：槽数按类型段那一类登记，不连带 {invariant_a_wrong_slot_count_drags_in}：{verdicts:?}"
            );
        }
        // G4（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G4 那一行、第三节第 4 条）：码 2 那一份按类型段那一类
        // （码 3）的 32 KiB 读回比校验和，对不上，I-2.1 红，I-4.8、I-7.4 跟着（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.2 / I-9.17 / I-9.23「另一份按同一长度读回比校验和，
        // 对不上照红 I-2.1」）。中央映射条目读那片容器时同一份也对不上，这三条不只由 F-W 这一读红。
        for invariant_of_the_checksum_family in ["I-2.1", "I-4.8", "I-7.4"] {
            assert!(
                violated.contains(invariant_of_the_checksum_family),
                "码 2 那一份在位置条目 {location_index}：按类型段那一类的长度读回对不上，{invariant_of_the_checksum_family} 红：{verdicts:?}"
            );
        }
        violated_on_each_device.push(violated);
    }
    assert_eq!(
        violated_on_each_device[0], violated_on_each_device[1],
        "同一张镜像只差码 2 那一份在哪块盘：红的同一批"
    );
}

/// inode 记录预留段（记录偏移 120 起 20 字节，I-9.7 那三段的最后一段）的最后一个字节。
pub(super) const INODE_RECORD_LAST_RESERVED_BYTE_OFFSET: usize = 139;

/// 攻方 1.1 inode 那一形（I-9.7）：新池新建文件那一版，inode 叶容器第一条记录的预留段写一个非零字节（两份、重封、补到根）。
#[test]
fn an_inode_leaf_with_a_non_zero_reserved_byte_beside_a_copy_verifying_as_a_node_reddens_the_reserved_bytes_invariant_on_either_device(
) {
    let clean = build_pool("known-bad-inode-leaf-reserved-byte-beside-a-copy-verifying-as-a-node")
        .memory_pool();
    let mut with_the_violation = clean.clone();
    mutate_unit(
        &mut with_the_violation,
        INODE_LEAF,
        |bytes| bytes[INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_LAST_RESERVED_BYTE_OFFSET] = 1,
        true,
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_INODE,
        "I-9.2",
        "I-9.7",
    );
}

/// 攻方 1.1 inode 那一形（I-9.15）：新池新建文件那一版，inode 叶容器第一条记录的 blocks 加 1（size 3000 ⇒ 要 6，写成 7）。
#[test]
fn an_inode_leaf_with_blocks_off_the_logical_length_beside_a_copy_verifying_as_a_node_reddens_the_blocks_invariant_on_either_device(
) {
    let clean =
        build_pool("known-bad-inode-leaf-blocks-beside-a-copy-verifying-as-a-node").memory_pool();
    let mut with_the_violation = clean.clone();
    mutate_unit(
        &mut with_the_violation,
        INODE_LEAF,
        |bytes| {
            let blocks_offset = INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_BLOCKS_OFFSET;
            let blocks = get_u64(bytes, blocks_offset);
            set_u64(bytes, blocks_offset, blocks + 1);
        },
        true,
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_INODE,
        "I-9.2",
        "I-9.15",
    );
}

/// 打包记录类型 `record_bytes`（登记的记录宽）少 1：I-1.7 记录宽那一步判红。
pub(super) fn record_width_one_below(record_bytes: u64) -> u16 {
    u16::try_from(record_bytes - 1).expect("登记的记录宽装得进 u16")
}

/// 攻方 1.1 inode 那一形（I-1.7）：新池新建文件那一版，inode 叶容器头里的记录宽 140 → 139。
#[test]
fn an_inode_leaf_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
) {
    let clean = build_pool("known-bad-inode-leaf-record-width-beside-a-copy-verifying-as-a-node")
        .memory_pool();
    let mut with_the_violation = clean.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut with_the_violation,
        TREE_KIND_INODE,
        record_width_one_below(singlefs_format::INODE_RECORD_BYTES),
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_INODE,
        "I-9.2",
        "I-1.7",
    );
}

/// 攻方 1.1 墓碑那一形（I-1.7）：三个单元的文件发完意图之后，墓碑容器头里的记录宽 40 → 39。
#[test]
fn a_tombstone_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-record-width-beside-a-copy-verifying-as-a-node",
    );
    let mut with_the_violation = clean.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut with_the_violation,
        TREE_KIND_TOMBSTONE,
        record_width_one_below(singlefs_format::TOMBSTONE_RECORD_BYTES),
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "I-1.7",
    );
}

/// 攻方 1.1 谱系那一形（I-1.7）：回退到 A 之后，谱系容器头里的记录宽 24 → 23。
#[test]
fn a_lineage_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
) {
    let clean = image_after_rolling_back_to_the_first_root(
        "known-bad-lineage-record-width-beside-a-copy-verifying-as-a-node",
    )
    .image;
    let mut with_the_violation = clean.clone();
    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
        &mut with_the_violation,
        TREE_KIND_LINEAGE,
        record_width_one_below(singlefs_format::LINEAGE_RECORD_BYTES),
    );
    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
        &clean,
        &with_the_violation,
        TREE_KIND_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        "I-1.7",
    );
}

/// X2 / X11：
/// inode 那一形只把身份引用出生树加 1，子指针出生树比它小，
/// 读不出孩子时那一小句的 `==` 改成 `>=` 照样红、钉不住。这一形把子指针头部出生树加 1（比身份引用出生树大）：I-9.2 照判、红。
#[test]
fn an_inode_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant(
) {
    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
        &build_pool("known-bad-inode-entry-pointer-birth-tree-unreadable-leaf").memory_pool(),
        TREE_KIND_INODE,
        PACKED_RECORD_TYPE_INODE,
        INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE,
        "I-9.2",
    );
}

/// X13 / X14 的判法：新池新建文件那一版，inode 树根第一条条目不改（类型段 2），
/// 叶容器盘序大那一份换成 inode 树根的码 2 字节、按改过的位置条目自证得过（[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]），
/// 再照 `break_the_first_copy` 把位置条目 0 那一份弄坏（自证不过）。认孩子的类时第一份那一处 `continue` 要往下看第二份：
/// 认出码 2，I-9.2 红、违例说明点名 IndexNode（改成 `break` 的话认不出别类，照类型段 2 读、两份都读不出，两段出生树相等，I-9.2 报成立）。
/// 两份里没有一份按码 3 自证得过：不走 F-W 那一支。
fn assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
    tag: &str,
    break_the_first_copy: &dyn Fn(&mut MemoryPool),
) {
    let clean = build_pool(tag).memory_pool();
    let mut image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
        &clean,
        TREE_KIND_INODE,
        1,
        &leave_the_entry_as_it_is,
    );
    break_the_first_copy(&mut image);
    assert_the_violation_names_the_child_class(
        &image,
        "I-9.2",
        PACKED_RECORD_TYPE_INODE,
        "IndexNode",
    );
}

/// X13：位置条目 0 那一份头里的类标签 3 → 9（不在登记表里，也就按哪一类的长度都自证不过）。
#[test]
fn an_inode_leaf_whose_first_copy_carries_an_unregistered_class_tag_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant(
) {
    assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
        "known-bad-inode-leaf-first-copy-unregistered-class-second-copy-node",
        &|image| {
            let first = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0)
                .locations[0];
            let offset = first.slot.0 * SLOT;
            let mut header =
                read(image, first.device.0, offset, common_pointer_head_corruption::node_bytes());
            header[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED;
            write(image, first.device.0, offset, &header);
        },
    );
}

/// X14：位置条目 0 那一份读不出（那条位置条目的槽写成盘尾之后的第一槽，读者按盘的契约拒读）。
#[test]
fn an_inode_leaf_whose_first_copy_does_not_read_back_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant(
) {
    assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
        "known-bad-inode-leaf-first-copy-unreadable-second-copy-node",
        &|image| {
            let first_slot_past_the_device_end =
                singlefs_core::address::SlotNumber(image.device_size_in_bytes / SLOT);
            change_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0, |entry| {
                let pointer_field = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
                    ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES;
                let mut pointer = singlefs_core::pointer::NodePointer::read_from(
                    &mut singlefs_core::bytes::ByteReader::at(entry, pointer_field.start),
                );
                pointer.locations[0].slot = first_slot_past_the_device_end;
                entry[pointer_field].copy_from_slice(
                    &common_pointer_head_corruption::node_pointer_bytes(&pointer),
                );
            });
            let first = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0)
                .locations[0];
            assert!(
                PoolReader::read(
                    image,
                    first.device,
                    DeviceOffsetInBytes(first.slot.0 * SLOT),
                    common_pointer_head_corruption::node_bytes()
                )
                .is_none(),
                "位置条目 0 那一份的头读不出"
            );
        },
    );
}
