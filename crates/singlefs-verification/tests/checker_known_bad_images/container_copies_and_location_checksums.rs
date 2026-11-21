//! 位置条目里的校验和、设备与槽号指错时，容器的打包不变量与条目不变量按哪一份副本判。

use super::*;

/// 一条树表条目里树的根指针那 86 字节：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 之后（D8（核心索引结构） 已定项 8 的字段表）。
pub(super) const TREE_TABLE_ENTRY_ROOT_POINTER_RANGE: std::ops::Range<usize> = 14..100;

/// 被崩溃恢复抛弃的根树表指针整条全零（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P2-e：攻方读码报
/// 「说明写的是单元已被重新分配或抹头」）：C (2, 8) 被实例 3 的实例表判出局之后，把 C 的根槽里树表指针写零、重封。
/// 被抛弃根那一遍照 core 的根记录读者跟过去、读不出，I-7.4 照旧红；说明写明是这条根自己的树表指针全零，不再说成它引用的单元被重新分配或抹头。
#[test]
fn an_abandoned_root_whose_tree_table_pointer_is_all_zero_reddens_the_reuse_invariant_naming_the_root_itself(
) {
    let (mut image, _, _, abandoned_txg) = image_after_a_recovery_abandoned_the_third_root(
        "known-bad-abandoned-root-all-zero-tree-table-pointer",
    );
    rewrite_the_root_slot_of(&mut image, CheckpointTxg(abandoned_txg), |bytes| {
        bytes[ROOT_RECORD_TREE_TABLE_POINTER_RANGE].fill(0);
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-7.4"],
        "被抛弃的根树表指针全零：只红 I-7.4（被抛弃根那一遍的判定丢掉）：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-7.4") else {
        panic!("上面判过 I-7.4 红");
    };
    assert!(
        detail.starts_with(&format!(
            "被抛弃的根（实例 2、txg {abandoned_txg}）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出"
        )),
        "I-7.4 说明写明是这条根自己的树表指针全零：{detail}"
    );
}

// ─── 删除线：墓碑树、谱系树层级 1 节点里类型段 0 的孩子照判 I-2.1 的 F-L0 用例、走读不往下读的孩子按哪种长度比的三形（测试尖度） ───

/// F-L0（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 第三支那一行、第三节第 1 条；攻方
/// `research/prompts/m3-batch29-code-r1-opus-output.md` 第一节 `tomb-L1T-sum01bad-type0z` / `lin-L1T-sum01bad-type0z` 那一形）：
/// 树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，子指针两条位置条目的整单元校验和各翻一字节，重封到根；孩子不动。
/// 两份按头里那一类（码 3）的长度都对不上，孩子不是另一类、走类型段 0 那一支，身份引用那一半成立、孩子读不出「子单元是码 2」不判，
/// 层级 1 的节点下面不往下走、记走读失败；孩子每一份副本照判 I-2.1：红 I-2.1、I-4.8、I-7.2、I-7.4，身份那一条不红。
fn assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
    mut image: MemoryPool,
    kind: u16,
    registered_type: u16,
    scene: &str,
) {
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry_type(
            entry,
            registered_type,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 1, "{scene}：树根直接管容器");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        scene,
    );
}

#[test]
fn a_tombstone_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
        common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-no-identity-both-location-checksums-wrong",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        "墓碑树根条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
    );
}

#[test]
fn a_lineage_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
        image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-no-identity-both-location-checksums-wrong",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        "谱系树根条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
    );
}

/// 内部条目（120 字节）子指针里第 `location_index` 条位置条目的整单元校验和写成 `checksum`（位置条目 = 盘 4 + 槽 6 + 整单元校验和 4）。
pub(super) fn set_the_whole_unit_checksum_in_a_child_location(
    entry: &mut [u8],
    location_index: usize,
    checksum: u32,
) {
    let checksum_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + location_index * POINTER_LOCATION_BYTES
        + LOCATION_CHECKSUM_OFFSET;
    entry[checksum_offset..checksum_offset + 4].copy_from_slice(&checksum.to_le_bytes());
}

/// 不在池里的盘身份：两块盘是 0、1，写成 7 的位置条目照 I-2.5 仍按设备身份升序，那一份读不出。
pub(super) const DEVICE_IDENTITY_NOT_IN_THE_POOL: u32 = 7;

/// 内部条目子指针里第 `location_index` 条位置条目的盘写成 `device`（槽与整单元校验和不动）。
pub(super) fn set_the_device_in_a_child_location(
    entry: &mut [u8],
    location_index: usize,
    device: u32,
) {
    let device_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + location_index * POINTER_LOCATION_BYTES;
    entry[device_offset..device_offset + 4].copy_from_slice(&device.to_le_bytes());
}

/// 新池新建文件那一版 inode 树根（层级 1）第一条条目的孩子（码 3 叶容器，32 KiB）前 16 KiB 的 CRC-32C。
fn checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(image: &MemoryPool) -> u32 {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0);
    crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
        image,
        common_pointer_head_corruption::the_slot_of(&child),
        common_pointer_head_corruption::node_bytes(),
    ))
}

/// 测试尖度 X3（正推 `research/prompts/m3-batch29-code-r1-sonnet-output.md` T-a 那张变异表；判决同一份第二节 K1 L1 测试尖度那一行、第三节第 4 条）：
/// 新池新建文件那一版 inode 树根（层级 1）第一条条目类型段 2 → 7、子指针第一条位置条目的整单元校验和换成叶容器前 16 KiB 的 CRC，重封到根；
/// 叶容器不动。孩子不往下走，第一份头里是登记过的码 3，只按 32 KiB 比、对不上：红 I-9.2、I-2.1、I-4.8、I-7.4；第二份对得上，不记走读失败、
/// I-7.2 不红。登记过的类也按 16 KiB 比时第一份对得上，I-2.1 不红。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_leaf_whose_first_location_checksum_matches_only_its_first_sixteen_kibibytes_reddens_the_checksum_invariant(
) {
    let mut image = build_pool(
        "known-bad-inode-entry-unregistered-type-first-location-checksum-of-sixteen-kibibytes",
    )
    .memory_pool();
    let checksum_of_the_first_sixteen_kibibytes =
        checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(&image);
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            0,
            checksum_of_the_first_sixteen_kibibytes,
        );
    });
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
        "inode 树根条目类型段 7、第一条位置条目的校验和是叶容器前 16 KiB 的 CRC",
    );
}

/// 测试尖度 X4（同上那张变异表）：新池新建文件那一版 inode 叶容器的类标签改成没登记的 9（不重封头，新的整单元校验和沿引用链补到根），
/// 再把 inode 树根第一条条目类型段 2 → 7、子指针第一条位置条目的整单元校验和换成改过类标签的叶容器前 16 KiB 的 CRC，重封到根。
/// 孩子不往下走；两份头里的码都没登记，16 KiB、32 KiB 两种长度都试：第一份按 16 KiB 对得上、第二份按 32 KiB 对得上，I-2.1 不红
/// （红的是 I-9.2 与映射树那一遍的 I-1.11）。没登记的只按 32 KiB 比时第一份对不上、只按 16 KiB 比时第二份对不上，I-2.1 都红。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_leaf_of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes_leaves_the_checksum_invariant_holding(
) {
    let mut image = build_pool(
        "known-bad-inode-entry-unregistered-type-unregistered-class-first-location-checksum-of-sixteen-kibibytes",
    )
    .memory_pool();
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    let leaf_slot = common_pointer_head_corruption::the_slot_of(&child);
    let mut leaf = common_pointer_head_corruption::unit_bytes_at(
        &image,
        leaf_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    assert_eq!(leaf[UNIT_HEADER_CLASS_TAG_OFFSET], 3, "孩子是码 3 叶容器");
    leaf[UNIT_HEADER_CLASS_TAG_OFFSET] = UNREGISTERED_UNIT_CLASS_TAG;
    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(
        &mut image, leaf_slot, &leaf,
    );
    let checksum_of_the_first_sixteen_kibibytes =
        checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(&image);
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            0,
            checksum_of_the_first_sixteen_kibibytes,
        );
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariant_set(&verdicts),
        ["I-1.11", "I-9.2"]
            .into_iter()
            .collect::<std::collections::BTreeSet<&'static str>>(),
        "inode 叶容器类标签 9、根条目类型段 7、第一条位置条目的校验和是前 16 KiB 的 CRC：{verdicts:?}"
    );
}

/// 头里的码不在登记表里（登记的是 1、2、3）。
const UNREGISTERED_UNIT_CLASS_TAG: u8 = 9;

/// 测试尖度 X5（同上那张变异表）：新池新建文件那一版 inode 树根第一条条目类型段 2 → 7、子指针第二条位置条目的盘换成不在池里的 7，
/// 重封到根。孩子不往下走；第二份读不出（头与整单元都读不出），算对不上：红 I-9.2、I-2.1、I-4.8、I-7.4；第一份对得上，不记走读失败。
/// 读不出算对得上时 I-2.1 不红。盘 7 上那一份的引用起点不归 I-3.21（走到的引用起点是最新根计入的分配记录的起点） 判：它与 I-3.1 同射程、
/// 只判池里的盘，池外那一份读不出由 I-2.1 判红（主 agent 定、零轮；不滤池外的盘时 I-3.21 点名盘 7 槽 50242、多红一条，合入后验证任务 6223 量的）。
#[test]
fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_second_copy_does_not_read_back_reddens_the_checksum_invariant(
) {
    let mut image = build_pool(
        "known-bad-inode-entry-unregistered-type-second-location-on-a-device-not-in-the-pool",
    )
    .memory_pool();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
        set_the_device_in_a_child_location(entry, 1, DEVICE_IDENTITY_NOT_IN_THE_POOL);
    });
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
        "inode 树根条目类型段 7、第二条位置条目指着不在池里的盘",
    );
}

// ─── 删除线：「走过」按读回那一份的落点认（F-KEY）、类型段 0 认「孩子是码 2」只认自证过的副本（F-ID-b）、层级 0 的根下 F-L0、
// 类型段 0 那一支的 F-W 与同长度别类（G1–G3） ───

/// 内部条目子指针里第 `location_index` 条位置条目的槽写成 `slot`（6 字节小端；盘与整单元校验和不动）。
pub(super) fn set_the_slot_in_a_child_location(entry: &mut [u8], location_index: usize, slot: u64) {
    let slot_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_FIRST_LOCATION_OFFSET
        + location_index * POINTER_LOCATION_BYTES
        + LOCATION_DEVICE_BYTES;
    entry[slot_offset..slot_offset + LOCATION_SLOT_BYTES]
        .copy_from_slice(&slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
}

/// 位置条目里盘身份的字节数（盘 4 + 槽 6 + 整单元校验和 4）。
pub(super) const LOCATION_DEVICE_BYTES: usize = 4;

/// 位置条目里槽号的字节数。
pub(super) const LOCATION_SLOT_BYTES: usize = 6;

/// 攻方 O3 的造法（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 观察 O3 那一行、第三节第 2 条；攻方
/// `research/prompts/m3-batch31-code-r1-opus-output.md` 第四节 `variant=bad` 那两形，模型 `research/prompts/m3-batch31-code-r1-opus-model/probe_kbi.rs`
/// 的 `o3_probe`）：种类 `kind` 那棵树根（层级 1）第一条条目的孩子（码 3 容器或叶容器）两份头里的记录宽减 1（重封、整单元校验和沿引用链补到根，
/// 只这一处是 I-1.7 违例）；再把那条条目子指针第 `location_index` 条位置条目的槽改成这一遍先走过的码 2 节点 X（种类 `kind_walked_before`
/// 那棵树的根）的槽、整单元校验和写成 X 那 16 KiB 的 CRC 翻几位（这一份对不上），重封到根。另一条位置条目那一份就是带违例、
/// 按自己的位置条目自证得过的容器，读者用的就是它。
fn image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width(
    clean: &MemoryPool,
    kind: u16,
    record_bytes: u64,
    kind_walked_before: u16,
    location_index: usize,
) -> MemoryPool {
    image_whose_first_root_entry_names_a_walked_node_beside_a_container(
        clean,
        kind,
        RecordWidthOfTheFirstContainer::OneBelowTheRegistered { record_bytes },
        kind_walked_before,
        location_index,
    )
}

/// 种类 `kind` 那棵树根第一条条目指的那片容器头里的记录宽：照登记的不动，或比登记的 `record_bytes` 少 1（I-1.7 违例）。
#[derive(Clone, Copy, Debug)]
pub(super) enum RecordWidthOfTheFirstContainer {
    AsRegistered,
    OneBelowTheRegistered { record_bytes: u64 },
}

/// 同 [`image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width`]，容器的记录宽照 `record_width`
/// （攻方 `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.1 节 O3 族的 `variant=bad` / `variant=good`）。
pub(super) fn image_whose_first_root_entry_names_a_walked_node_beside_a_container(
    clean: &MemoryPool,
    kind: u16,
    record_width: RecordWidthOfTheFirstContainer,
    kind_walked_before: u16,
    location_index: usize,
) -> MemoryPool {
    let mut image = clean.clone();
    match record_width {
        RecordWidthOfTheFirstContainer::AsRegistered => {}
        RecordWidthOfTheFirstContainer::OneBelowTheRegistered { record_bytes } => {
            common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
                &mut image,
                kind,
                record_width_one_below(record_bytes),
            );
        }
    }
    let walked_node_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, kind_walked_before).root;
    let walked_node_slot = common_pointer_head_corruption::the_slot_of(&walked_node_pointer);
    let wrong_checksum_of_the_walked_node =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            walked_node_slot,
            common_pointer_head_corruption::node_bytes(),
        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        set_the_slot_in_a_child_location(entry, location_index, walked_node_slot.0);
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            location_index,
            wrong_checksum_of_the_walked_node,
        );
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    image
}

/// 翻进校验和里的几位：那一份按哪种长度都对不上。
pub(super) const CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY: u32 = 0x5a5a_5a5a;

/// F-KEY 的判法：两种盘序（指着先走过那个节点的是位置条目 0 / 1）各判一次，I-1.7 都红、违例说明点名那片容器的记录宽；红的同一批
/// （`allowed_to_differ` 里的几条除外，见各用例）。「走过」按读回那一份的落点认，不按第 0 条位置条目认：位置条目 0 指着先走过的节点时读回来的那片容器不当成走过、不合并、不返回，
/// I-1.7 照判、红，位置条目 1 那一形同。
fn assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
    clean: &MemoryPool,
    kind: u16,
    record_bytes: u64,
    kind_walked_before: u16,
    expected_violations_on_location_one: &[&'static str],
    allowed_to_differ: &[&'static str],
) {
    let mut violated_on_each_location = Vec::new();
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width(
            clean,
            kind,
            record_bytes,
            kind_walked_before,
            location_index,
        );
        let verdicts = check_pool_image(&image);
        let packed_container_verdict = verdicts
            .iter()
            .find(|(identifier, _)| *identifier == "I-1.7")
            .map(|(_, found)| found.clone())
            .expect("清单里有");
        assert!(
            matches!(
                &packed_container_verdict,
                InvariantVerdict::Violated(detail)
                    if detail.contains(&format!("记录宽 {}", record_bytes - 1))
            ),
            "指着先走过的节点的是位置条目 {location_index}：读回来的那片容器照判，I-1.7 红在它的记录宽：{verdicts:?}"
        );
        violated_on_each_location.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_location[1],
        expected_violations_on_location_one
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<&'static str>>(),
        "位置条目 1 指着先走过的节点（攻方 `collide=loc1` 那一形）红的那一批"
    );
    let outside_the_allowed = |violated: &std::collections::BTreeSet<&'static str>| {
        violated
            .iter()
            .copied()
            .filter(|invariant| !allowed_to_differ.contains(invariant))
            .collect::<std::collections::BTreeSet<&'static str>>()
    };
    assert_eq!(
        outside_the_allowed(&violated_on_each_location[0]),
        outside_the_allowed(&violated_on_each_location[1]),
        "同一张镜像只差指着先走过的节点的是哪一条位置条目：{allowed_to_differ:?} 之外红的同一批"
    );
}

/// F-KEY 墓碑那一形：三个单元的文件发完意图之后，墓碑容器记录宽 40 → 39，墓碑树根第一条条目子指针的一条位置条目指着 inode 树根（先走过）。
#[test]
fn a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
) {
    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-record-width-location-naming-a-walked-node",
        ),
        TREE_KIND_TOMBSTONE,
        singlefs_format::TOMBSTONE_RECORD_BYTES,
        TREE_KIND_INODE,
        // I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判不红：记账多于遍历落在上沿之内（改三判之前红在这一边）。
        &["I-1.7", "I-2.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
        &[],
    );
}

/// F-KEY inode 那一形：新池新建文件那一版，inode 叶容器记录宽 140 → 139，inode 树根第一条条目子指针的一条位置条目指着 extent 树根（先走过）。
#[test]
fn an_inode_leaf_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
) {
    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
        &build_pool("known-bad-inode-leaf-record-width-location-naming-a-walked-node")
            .memory_pool(),
        TREE_KIND_INODE,
        singlefs_format::INODE_RECORD_BYTES,
        TREE_KIND_EXTENT,
        &[
            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
        ],
        &[],
    );
}

/// F-KEY 谱系那一形：回退到 A 之后，谱系容器记录宽 24 → 23，谱系树根第一条条目子指针的一条位置条目指着 inode 树根（先走过，
/// 回退那一版照抄 A 的 inode 树，A 那条根也引用它）。读回来的那片容器（这一版新写的）的诞生代号记在它读回的那个落点上，不盖掉 inode 树根的
/// （攻方 allrbmap，判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条）：两种盘序红的同一批，I-4.9 不多红。
#[test]
fn a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
) {
    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-record-width-location-naming-a-walked-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        singlefs_format::LINEAGE_RECORD_BYTES,
        TREE_KIND_INODE,
        // I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判不红：记账多于遍历落在上沿之内（改三判之前红在这一边）。
        &["I-1.7", "I-2.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
        &[],
    );
}

/// 攻方 A3 的造法（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行；攻方同一份第一节 A3，
/// 模型 `probe_kbi.rs` 的 `matrix_probe` 里 `c16` 与 `node` 两种副本）：种类 `kind` 那棵树根第一条条目类型段 `registered_type` → 0、身份三段清零；
/// 孩子（码 3 容器）在位置条目 `container_location_index` 那块盘上不动、那条位置条目的整单元校验和换成容器前 16 KiB 的 CRC（`c16`：头里是码 3，
/// 按 32 KiB 自证不过）；另一块盘上孩子的槽写进这棵树根（码 2）的原样字节、那条位置条目的校验和换成它的 CRC（`node`：按 16 KiB 自证得过）。
/// `root_level` 给了就先把根头里的层级改成它（层级 0 的根，攻方 `l0root`）。
fn image_whose_type_zero_root_entry_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    container_location_index: usize,
    root_level: Option<u8>,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let child_slot = common_pointer_head_corruption::the_slot_of(&child);
    let container = common_pointer_head_corruption::unit_bytes_at(
        clean,
        child_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    assert_eq!(
        container[UNIT_HEADER_CLASS_TAG_OFFSET], 3,
        "孩子是码 3 容器"
    );
    let checksum_of_the_first_sixteen_kibibytes =
        crc32_castagnoli(&container[..common_pointer_head_corruption::node_bytes()]);
    let node = root_node_bytes_of_the_tree_of_kind(clean, kind);
    let checksum_of_the_node = crc32_castagnoli(&node);
    let node_location_index = 1 - container_location_index;
    let mut image = clean.clone();
    write(
        &mut image,
        child.locations[node_location_index].device.0,
        child_slot.0 * SLOT,
        &node,
    );
    if let Some(level) = root_level {
        set_the_level_of_the_root_of_kind(&mut image, kind, level);
    }
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry_type(
            entry,
            registered_type,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            container_location_index,
            checksum_of_the_first_sixteen_kibibytes,
        );
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            node_location_index,
            checksum_of_the_node,
        );
    });
    assert_eq!(level, root_level.unwrap_or(1), "根的层级是这一形要的那一级");
    image
}

/// 种类 `kind` 那棵树根头里的层级改成 `level`（重封、整单元校验和沿引用链补到根）。
fn set_the_level_of_the_root_of_kind(image: &mut MemoryPool, kind: u16, level: u8) {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        common_pointer_head_corruption::the_slot_of(&root_pointer),
        common_pointer_head_corruption::node_bytes(),
        |root| root[INDEX_NODE_LEVEL_OFFSET] = level,
    );
}

/// F-ID-b 的判法（只认自证过的副本）：两种盘序（`c16` 那一份在位置条目 0 / 1）各判一次，认「孩子是码 2」只看按自己那一类长度
/// 整份自证过的副本——`c16` 不算、`node` 那一份是码 2：`identity_invariant` 两种盘序都不红；红的恰好是 I-2.1 一族与走读失败
/// （I-2.1、I-4.8、I-7.2、I-7.4：F-L0 按头里的码 3 比 `c16` 那一份对不上，层级 ≤ 1 不往下走、记走读失败），两种盘序同一批；
/// 再加 I-3.15：换了字节的那一份头用得了、对不上映射，那条记录不带隔离记号，只认记号之后不豁免（D19（块指针的结构与宽度预算） 已定项 5
/// ）；再加 I-3.18（分配记录的跨度等于它罩住的单元那一类的槽数，
/// C411（分配记录的跨度段与单元类别不符没检查））：那一份头用得了、是码 2 一槽，罩住它的未释放记录照码 3 容器跨两槽。
/// 若取第一份按 16 KiB 对得上的副本：`c16` 在位置条目 0 时认成码 3，`identity_invariant` 红，换盘就不红（`c0=c16 c1=node` 对 `c0=node c1=c16`）。
fn assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    identity_invariant: &'static str,
    root_level: Option<u8>,
) {
    for container_location_index in 0..2 {
        let image = image_whose_type_zero_root_entry_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
            clean,
            kind,
            registered_type,
            container_location_index,
            root_level,
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariant_set(&verdicts),
            [
                "I-2.1",
                "I-3.15",
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
                "I-4.8",
                "I-7.2",
                "I-7.4"
            ]
            .into_iter()
            .collect::<std::collections::BTreeSet<&'static str>>(),
            "`c16` 那一份在位置条目 {container_location_index}：{identity_invariant} 不红，红的恰好是 I-2.1 一族、走读失败、I-3.15 与 I-3.18：{verdicts:?}"
        );
    }
}

#[test]
fn a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device(
) {
    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        None,
    );
}

#[test]
fn a_lineage_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_lineage_entry_invariant_holding_on_either_device(
) {
    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        None,
    );
}

/// 同上，墓碑树根头里的层级改成 0（攻方 `l0root` 那一对）。
#[test]
fn a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device(
) {
    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-root-level-zero-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        Some(0),
    );
}

/// 同上，谱系树根头里的层级改成 0。
#[test]
fn a_lineage_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_lineage_entry_invariant_holding_on_either_device(
) {
    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-root-level-zero-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        Some(0),
    );
}

/// F-L0 在层级 0 的根上（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K1 N1 层级 0 的根下类型段 0 那一行；
/// 攻方 `PROBE l0root tree=tomb level=0 type=t0 c0=bad c1=bad`）：三个单元的文件发完意图之后，墓碑树根头里的层级 1 → 0，第一条条目类型段 1 → 0、
/// 身份三段清零、子指针两条位置条目的整单元校验和各翻一字节，重封到根。层级 0 的节点同样不往下走、记走读失败，孩子每一份副本照判 I-2.1：
/// 红 I-2.1、I-4.8、I-7.2、I-7.4（F-L0 只在层级恰为 1 时判的话，层级 0 这一形 I-2.1 不红）。
#[test]
fn a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
) {
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-root-level-zero-entry-type-zero-both-location-checksums-wrong",
    );
    set_the_level_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0);
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
    });
    assert_eq!(level, 0, "墓碑树根头里的层级改成了 0");
    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
        &image,
        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
        "墓碑树根层级 0、条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
    );
}

/// G1（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G1 那一行、第三节第 1 条）：类型段 0 那一支的 F-W。
/// 种类 `kind` 那棵树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，孩子在一块盘上那一份换成这棵树根（码 2）的原样字节、
/// 按 16 KiB 自证得过（类型段 0 那一类），另一份是原来那片码 3 容器、自证成另一类
/// （[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]）。两种盘序各判一次，红的恰好是 `expected_violations`
/// （攻方 `research/prompts/m3-batch31-code-r1-opus-output.md` 那份探针 `type=t0 c0=node c1=ok` 与 `c0=ok c1=node` 两行量到的那一批，
/// 再加 I-3.15：换成码 2 的那一份头用得了、对不上映射，那条记录不带隔离记号，只认记号之后不豁免，D19（块指针的结构与宽度预算） 已定项 5；
/// 再加 I-3.18：那一份头用得了、是码 2 一槽，罩住它的未释放记录照原来的码 3 跨两槽，C411（分配记录的跨度段与单元类别不符没检查）），其中
/// 只有照类型段 0 那一支往下走那一份才红的 `invariant_only_the_type_zero_branch_reddens`，违例说明含 `detail_only_the_type_zero_branch_writes`。
/// 类型段 0 那一处见别类就停（不走 F-W，变异 S1 / S2）时那一份不往下走，那一条不红。
fn assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    expected_violations: &[&'static str],
    invariant_only_the_type_zero_branch_reddens: &'static str,
    detail_only_the_type_zero_branch_writes: &str,
) {
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
            clean,
            kind,
            location_index,
            &|entry| {
                change_the_entry_type(
                    entry,
                    registered_type,
                    CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
                );
                zero_the_three_identity_fields(entry);
            },
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariant_set(&verdicts),
            expected_violations
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<&'static str>>(),
            "码 2 那一份在位置条目 {location_index}：照类型段 0 那一支往下走那一份，红的那一批：{verdicts:?}"
        );
        let only_the_type_zero_branch = verdicts
            .iter()
            .find(|(identifier, _)| *identifier == invariant_only_the_type_zero_branch_reddens)
            .map(|(_, found)| found.clone())
            .expect("清单里有");
        assert!(
            matches!(
                &only_the_type_zero_branch,
                InvariantVerdict::Violated(detail) if detail.contains(detail_only_the_type_zero_branch_writes)
            ),
            "码 2 那一份在位置条目 {location_index}：{invariant_only_the_type_zero_branch_reddens} 红在类型段 0 那一支往下走那一份上（说明含「{detail_only_the_type_zero_branch_writes}」）：{only_the_type_zero_branch:?}"
        );
    }
}

/// G1 墓碑那一形：三个单元的文件发完意图之后。类型段 0 那一支往下走那一份——层级 1 的节点下面不往下读、记走读失败（F-L0），I-7.2 红在那一句。
#[test]
fn a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
) {
    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-container-copy-verifying-as-a-node-goes-down",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        &[
            "I-2.1",
            "I-3.15",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            "I-4.8",
            "I-7.2",
            "I-7.4",
            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        ],
        "I-7.2",
        "的节点有类型段 0 的条目",
    );
}

/// G1 谱系那一形：回退到 A 之后。同墓碑那一形。
#[test]
fn a_lineage_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
) {
    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-container-copy-verifying-as-a-node-goes-down",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        &[
            "I-2.1",
            "I-3.15",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            "I-4.8",
            "I-7.2",
            "I-7.4",
            LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        ],
        "I-7.2",
        "的节点有类型段 0 的条目",
    );
}

/// G1 inode 那一形：新池新建文件那一版。类型段 0 那一支往下走那一份——码 2 那一份当 inode 树的内部节点读（它就是 inode 树根，层级 1，
/// 不是父层级减一），I-1.1 红在「inode 树的内部节点」那一句，槽数按码 2 一槽登记（叶容器另一槽没人引用：I-3.11、I-5.1，攻方
/// `research/prompts/m3-batch30-code-r1-opus-output.md` 观察 O1 那几条；I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判不红，
/// 记账多于遍历落在上沿之内，改三判之前红在这一边）。
#[test]
fn an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
) {
    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
        &build_pool("known-bad-inode-entry-type-zero-leaf-copy-verifying-as-a-node-goes-down")
            .memory_pool(),
        TREE_KIND_INODE,
        PACKED_RECORD_TYPE_INODE,
        &[
            "I-1.1",
            "I-1.2",
            "I-2.1",
            "I-3.11",
            "I-3.15",
            ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            "I-4.8",
            "I-5.1",
            "I-7.2",
            "I-7.4",
            "I-9.2",
        ],
        "I-1.1",
        "inode 树的内部节点",
    );
}

/// G2（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G2 那一行、第三节第 2 条）：新池新建文件那一版，
/// inode 树根第一条条目不改（类型段 2，说孩子是码 3），叶容器盘 1 那一份换成第一个文件的数据单元（码 1、32 KiB）原样字节、位置条目 1 的整单元校验和
/// 换成它的 CRC——按类型段那一类（码 3）的长度 32 KiB 校验和对得上、头里却是同长度的别类码 1；位置条目 0 的校验和翻一字节（盘 0 那一份自证不过）。
/// 没有一份头里是码 3 又自证得过：不走 F-W，I-9.2 红在「孩子头里的类标签是 DataUnit」那一句、不往下走，码 1 那一份不当叶容器判（I-1.3、I-1.7 不红）。
/// 只看整单元校验和、不看头里的类标签时（正推 S4），码 1 那一份算作类型段那一类自证过、照类型段那一支往下走，I-9.2 的说明写成往下走的那一句。
#[test]
fn an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy(
) {
    let mut image =
        build_pool("known-bad-inode-leaf-copy-matching-at-the-container-length-is-a-data-unit")
            .memory_pool();
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    let leaf_slot = common_pointer_head_corruption::the_slot_of(&child);
    let data_unit = common_pointer_head_corruption::unit_bytes_at(
        &image,
        singlefs_core::address::SlotNumber(DATA_UNIT),
        common_pointer_head_corruption::data_unit_bytes(),
    );
    assert_eq!(
        data_unit[UNIT_HEADER_CLASS_TAG_OFFSET], 1,
        "第一个文件的数据单元是码 1"
    );
    write(
        &mut image,
        child.locations[1].device.0,
        leaf_slot.0 * SLOT,
        &data_unit,
    );
    let checksum_of_the_data_unit = crc32_castagnoli(&data_unit);
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        set_the_whole_unit_checksum_in_a_child_location(entry, 1, checksum_of_the_data_unit);
        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
    });
    assert_the_violation_names_the_child_class(
        &image,
        "I-9.2",
        PACKED_RECORD_TYPE_INODE,
        "DataUnit",
    );
    let verdicts = check_pool_image(&image);
    let violated = violated_invariant_set(&verdicts);
    for invariant_of_walking_that_copy_as_a_leaf in ["I-1.3", "I-1.7"] {
        assert!(
            !violated.contains(invariant_of_walking_that_copy_as_a_leaf),
            "码 1 那一份不往下走、不当叶容器判：{invariant_of_walking_that_copy_as_a_leaf} 不红：{verdicts:?}"
        );
    }
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail) if !detail.contains("照类型段那一支往下走那一份")
        ),
        "I-9.2 红在「孩子是另一类、不往下走」那一句，不是 F-W 往下走那一句：{:?}",
        verdict(&image, "I-9.2")
    );
}
