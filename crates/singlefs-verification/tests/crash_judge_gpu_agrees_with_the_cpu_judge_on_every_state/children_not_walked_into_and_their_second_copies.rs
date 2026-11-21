//! 走读不往下读的孩子每一份副本照判 I-2.1、读不出孩子时的出生树、第二份副本的自证、实例表链每一片的出生身份（I-1.2）。

use super::*;

// ───────────────────────── 走读不往下读的孩子每一份副本照判 I-2.1 的 H-J3a、H-J3b 用例 ─────────────────────────

/// 码 2 节点第一条条目子指针里第 `location` 条位置条目（[`POINTER_LOCATION_ENTRY_OFFSETS`] 之一）的整单元校验和第 0 字节翻一下：
/// 孩子不动，那一份按哪一种长度都与位置条目里的校验和对不上。
pub(super) fn flip_a_byte_of_a_location_checksum_in_the_first_entry(
    bytes: &mut [u8],
    location: usize,
) {
    let field = first_entry_of_a_container_number_node()
        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + location
        + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
    bytes[field] ^= 0xFF;
}

/// 攻方 B1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行；harness 坏镜像
/// `an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
/// 同一形，录成流）：新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；
/// 叶容器不动。
fn record_new_pool_with_the_inode_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-root-entry-type-seven-two-location-checksums-wrong",
        &is_the_inode_tree_root_over_leaf_containers,
        false,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
            for location in POINTER_LOCATION_ENTRY_OFFSETS {
                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
            }
        },
    )
}

/// H-J3a：类型段不认得那一支不往下走孩子，两份副本照判 I-2.1、都对不上、记走读失败：CPU 池级 checker 红 I-9.2、I-2.1、I-4.8、
/// I-7.2、I-7.4；GPU 走读内核同改（`w_judge_every_copy_of_a_child_the_walk_does_not_go_into`），
/// 每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong",
        &record_new_pool_with_the_inode_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
            "crash-judge-gpu-i21-inode-root-entry-type-seven-two-location-checksums-wrong",
        ),
        &["I-9.2", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
    );
}

/// H-J3b 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3b 那一行；harness 坏镜像
/// `an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant` 同一形，录成流）：
/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；叶容器不动。
fn record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
    tag: &str,
) -> RecordedFlow {
    record_new_pool_with_a_resealed_field(
        tag,
        "new-pool-inode-root-entry-type-zero-first-location-checksum-wrong",
        &is_the_inode_tree_root_over_leaf_containers,
        false,
        &|bytes| {
            set_the_type_of_the_first_entry(bytes, 0);
            flip_a_byte_of_a_location_checksum_in_the_first_entry(
                bytes,
                POINTER_LOCATION_ENTRY_OFFSETS[0],
            );
        },
    )
}

/// H-J3b：第二份自证成码 3（类型段 0 说的是码 2），走「孩子是另一类」那一支，不往下走；第一份按哪一种长度都对不上，照判 I-2.1：
/// CPU 池级 checker 红 I-9.2、I-2.1、I-4.8、I-7.4；第二份对得上、不记走读失败，I-7.2 不红；
/// GPU 走读内核同改，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong",
        &record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
            "crash-judge-gpu-i21-inode-root-entry-type-zero-first-location-checksum-wrong",
        ),
        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
    );
}

// ───────────────────────── 读不出孩子时照判出生树那一小句（H-B）、孩子两份都看（H-V）、第二份副本的自证 ─────────────────────────

/// 位置条目里整单元校验和的偏移（盘 4、槽 6 之后）。
pub(super) const LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET: usize = 10;

/// 一个单元的这几份副本各翻一个载荷字节、不重封、不补引用链：指着它的位置条目还是翻之前的整单元 CRC，这几份都读不出。
fn flip_a_payload_byte_of_each_copy_without_resealing(flow: &mut RecordedFlow, copies: &[usize]) {
    for copy in copies {
        let WrittenContents::Bytes(bytes) = &mut flow.writes[*copy].contents else {
            panic!("单元写是普通写");
        };
        let flipped_byte = bytes.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
        bytes[flipped_byte] ^= 0xFF;
    }
}

/// 节点第一条条目的子指针第 0 条位置条目指的槽就是 `copies` 那个单元的槽（钉住改的条目指着要弄坏的那个孩子）。
pub(super) fn assert_the_first_entry_names_the_unit_of(
    flow: &RecordedFlow,
    node_copy: usize,
    copies: &[usize],
    what: &str,
) {
    let node = flow.writes[node_copy].bytes().expect("单元写带字节");
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &node[first_entry_of_a_container_number_node()
                + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[0],
        )
        .1,
        flow.writes[copies[0]].offset.0 / SLOT_BYTES,
        "{what}"
    );
}

/// 节点第一条条目里 `field`（条目内偏移）起那 8 字节（出生树那两段之一）加 1。
fn add_one_to_a_birth_tree_of_the_first_entry(bytes: &mut [u8], field: usize) {
    let offset = first_entry_of_a_container_number_node() + field;
    let birth_tree = read_u64_at(bytes, offset);
    bytes[offset..offset + 8].copy_from_slice(&(birth_tree + 1).to_le_bytes());
}

/// H-B inode 那一形（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 1 条；harness 坏镜像
/// `an_inode_tree_entry_whose_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant` 同一形，录成流）：
/// 新池新建文件的流上 inode 树根（层级 1）第一条条目身份引用的出生树加 1、重封到根，它指的叶容器两份各翻一个载荷字节、不重封（两份都读不出）。
fn record_new_pool_with_the_inode_root_entry_identity_birth_tree_one_off_over_a_leaf_that_does_not_read_back(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-inode-root-entry-identity-birth-tree-one-off-unreadable-leaf";
    let mut flow = record_new_pool_file_creation(tag, name);
    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    let root_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &leaf_copies,
        "inode 树根第一条条目指着流里最后写的那片叶容器",
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        add_one_to_a_birth_tree_of_the_first_entry(bytes, CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET);
    });
    flip_a_payload_byte_of_each_copy_without_resealing(&mut flow, &leaf_copies);
    flow
}

/// H-B：CPU 池级 checker 读不出孩子时照判「子指针头部的出生树 == 身份引用的出生树」、红 I-9.2（条目身份与子头相符），类型段 2 那一支同样判，
/// 叶容器读不出、I-2.1 红；GPU 走读内核读不出那一支同判（`w_walk_inode_root`），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back",
        &record_new_pool_with_the_inode_root_entry_identity_birth_tree_one_off_over_a_leaf_that_does_not_read_back(
            "crash-judge-gpu-i92-inode-root-entry-identity-birth-tree-unreadable-leaf",
        ),
        &["I-9.2", "I-2.1"],
    );
}

/// H-B 墓碑那一形（harness 坏镜像
/// `a_tombstone_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_tombstone_entry_invariant`
/// 同一形，录成流）：删除中间态的流上墓碑树根（层级 1）第一条条目子指针头部的出生树加 1、重封到根，它指的墓碑容器两份各翻一个载荷字节、
/// 不重封（两份都读不出）。
fn record_deletion_with_the_tombstone_root_entry_pointer_birth_tree_one_off_over_a_container_that_does_not_read_back(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-root-entry-pointer-birth-tree-one-off-unreadable-container";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        add_one_to_a_birth_tree_of_the_first_entry(
            bytes,
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + POINTER_BIRTH_TREE_OFFSET,
        );
    });
    flip_a_payload_byte_of_each_copy_without_resealing(&mut flow, &container_copies);
    flow
}

/// H-B：CPU 池级 checker 容器读不出时照判「子指针头部出生树 == 身份引用出生树」、红 I-9.23（墓碑树条目身份与子头相符），不止记走读失败，
/// 容器读不出、I-2.1 红；GPU 走读内核容器树读不出那一支同判（`w_walk_container_tree_root`），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back",
        &record_deletion_with_the_tombstone_root_entry_pointer_birth_tree_one_off_over_a_container_that_does_not_read_back(
            "crash-judge-gpu-i923-tombstone-root-entry-pointer-birth-tree-unreadable-container",
        ),
        &["I-9.23", "I-2.1"],
    );
}

/// 新池新建文件的流上 inode 叶容器盘序大的那一份的前 16 KiB 换成原来那个 inode 树根（码 2）的原样字节；`checksum_of_the_second_location`
/// 说 inode 树根第一条条目子指针第 1 条位置条目的整单元校验和怎么办：换成那 16 KiB 的 CRC（重封到根，那一份自证成码 2），或不动（那一份自证不过）。
#[derive(Clone, Copy, Debug)]
pub(super) enum ChecksumOfTheSecondLocation {
    MatchesTheNodeBytes,
    LeftAsTheLeafChecksum,
}

/// 见 [`ChecksumOfTheSecondLocation`]：流名 `name`，叶容器盘序小的那一份不动（码 3，按它的位置条目自证得过）。
fn record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
    tag: &str,
    name: &'static str,
    checksum_of_the_second_location: ChecksumOfTheSecondLocation,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
        &mut flow,
        name,
        checksum_of_the_second_location,
    );
    flow
}

/// [`record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes`] 在一条录好的新池新建文件流上做的那一步：
/// 流里最后那片 inode 叶容器盘序大的那一份换成最后那份 inode 树根（码 2）的字节，位置条目照 `checksum_of_the_second_location`。
pub(super) fn hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
    flow: &mut RecordedFlow,
    name: &'static str,
    checksum_of_the_second_location: ChecksumOfTheSecondLocation,
) {
    let leaf_copies = last_unit_copies(flow, name, &is_an_inode_leaf_container, false);
    let second_leaf_copy =
        if flow.writes[leaf_copies[0]].device.0 < flow.writes[leaf_copies[1]].device.0 {
            leaf_copies[1]
        } else {
            leaf_copies[0]
        };
    let root_copies = last_unit_copies(
        flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    assert_the_first_entry_names_the_unit_of(
        flow,
        root_copies[0],
        &leaf_copies,
        "inode 树根第一条条目指着流里最后写的那片叶容器",
    );
    let node_bytes = flow.writes[root_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(
        unit_class(&node_bytes),
        2,
        "换进去的那 16 KiB 头里的类是码 2"
    );
    let second_location = first_entry_of_a_container_number_node()
        + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET
        + POINTER_LOCATION_ENTRY_OFFSETS[1];
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &flow.writes[root_copies[0]].bytes().expect("单元写带字节")
                [first_entry_of_a_container_number_node()
                    + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[1],
        )
        .0,
        flow.writes[second_leaf_copy].device.0,
        "第 1 条位置条目是盘序大的那块盘"
    );
    match checksum_of_the_second_location {
        ChecksumOfTheSecondLocation::MatchesTheNodeBytes => {
            let checksum_of_the_node_bytes = crc32c(&node_bytes);
            corrupt_units_and_reseal_up_to_the_root(flow, &root_copies, &|bytes| {
                let field = second_location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
                bytes[field..field + 4].copy_from_slice(&checksum_of_the_node_bytes.to_le_bytes());
            });
        }
        ChecksumOfTheSecondLocation::LeftAsTheLeafChecksum => {}
    }
    let WrittenContents::Bytes(leaf) = &mut flow.writes[second_leaf_copy].contents else {
        panic!("单元写是普通写");
    };
    leaf[..node_bytes.len()].copy_from_slice(&node_bytes);
}

/// H-V（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 2 条；攻方 `research/prompts/m3-batch26-code-r1-opus-output.md`
/// 第 1.3 节那一形，条目不改、自证成另一类的那份在盘序大的那一侧）：inode 树根第一条条目类型段 2，叶容器盘 0 那一份是码 3、自证得过，
/// 盘 1 那一份是码 2、按改过的位置条目也自证得过。CPU 两份都看、认出码 2，红 I-9.2（在第一份自证过的副本处停、交回 None 的话，I-9.2 报成立）；
/// GPU `w_child_unit_class_other_than_the_entry_type_names` 同改，每个状态逐位相同。这条流也走 GPU 第二份副本那一支自证得过的那一侧。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_verifies_as_a_node() {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_verifies_as_a_node",
        &record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
            "crash-judge-gpu-i92-inode-leaf-second-copy-verifying-as-a-node",
            "new-pool-inode-leaf-second-copy-verifying-as-a-node",
            ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
        ),
        &["I-9.2"],
    );
}

/// GPU 第二份副本那一支的对拍流与变异：
/// 同上一条，位置条目不改：盘 1 那一份头里是码 2、按叶容器的整单元校验和自证不过（陈旧副本）。CPU 不把它当另一类，照类型段 2 读盘 0 那一份，
/// I-9.2 不红，那一份对不上校验和、I-2.1 红；GPU 第二份副本那一支也要自证（只认头里的类的话 GPU 红 I-9.2、CPU 不红），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_stale_node_that_does_not_verify(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_stale_node_that_does_not_verify";
    let flow = record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
        "crash-judge-gpu-i92-inode-leaf-second-copy-stale-node",
        "new-pool-inode-leaf-second-copy-stale-node",
        ChecksumOfTheSecondLocation::LeftAsTheLeafChecksum,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-2.1"),
        "流 {} 上 CPU 池级 checker 要红 I-2.1（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        !comparison.cpu_checker_red_invariants.contains("I-9.2"),
        "流 {} 上自证不过的第二份不算另一类：CPU 池级 checker 不红 I-9.2（红过的：{:?}）",
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

// ───────────────── H-V 子树（F-W）、实例表链上每一片判 I-1.2 的出生身份那一半（F-IT） ─────────────────

/// 叶容器第一条记录（新池新建文件那一版只有它一条）的 blocks 加 1：size 3000 要 6，写成 7，I-9.15 红。
fn add_one_to_the_blocks_of_the_first_record_of_an_inode_leaf(bytes: &mut [u8]) {
    let blocks_offset = PACKED_UNIT_RECORDS_START + INODE_RECORD_BLOCKS_OFFSET;
    let blocks = read_u64_at(bytes, blocks_offset);
    bytes[blocks_offset..blocks_offset + 8].copy_from_slice(&(blocks + 1).to_le_bytes());
}

/// F-W（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条；
/// harness 坏镜像 `an_inode_leaf_with_blocks_off_the_logical_length_beside_a_copy_verifying_as_a_node_reddens_the_blocks_invariant_on_either_device`
/// 同一形，录成流）：新池新建文件的流上 inode 叶容器两份第一条记录的 blocks 加 1（重封、沿引用链补到根），
/// 再照 [`hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf`] 把盘序大的那一份换成 inode 树根的码 2 字节、位置条目跟着换。
fn record_new_pool_with_blocks_off_in_the_inode_leaf_and_its_second_copy_holding_the_inode_root_bytes(
    tag: &str,
) -> RecordedFlow {
    let name = "new-pool-inode-leaf-blocks-off-second-copy-verifying-as-a-node";
    let mut flow = record_new_pool_file_creation(tag, name);
    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &leaf_copies,
        &add_one_to_the_blocks_of_the_first_record_of_an_inode_leaf,
    );
    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
        &mut flow,
        name,
        ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
    );
    flow
}

/// F-W 那一形：叶容器盘 0 那一份是码 3、带 blocks 的违例、自证得过，盘 1 那一份自证成码 2。CPU 认出码 2、I-9.2 红，另一份按类型段那一类自证过，
/// 照类型段 2 那一支走盘 0 那一份、判出 I-9.15（见别类就不往下走的话，I-9.15 报不适用）；槽数按类型段那一类登记，不连带 I-3.1、I-3.11、I-5.1。
/// GPU 走读内核 `w_walk_inode_root` 同改（`w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`），每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(
) {
    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node";
    let flow = record_new_pool_with_blocks_off_in_the_inode_leaf_and_its_second_copy_holding_the_inode_root_bytes(
        "crash-judge-gpu-i92-inode-leaf-blocks-off-second-copy-verifying-as-a-node",
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    for invariant in ["I-9.2", "I-9.15"] {
        assert!(
            comparison.cpu_checker_red_invariants.contains(invariant),
            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
            flow.name,
            comparison.cpu_checker_red_invariants
        );
    }
    for invariant_a_wrong_slot_count_drags_in in ["I-3.1", "I-3.11", "I-5.1"] {
        assert!(
            !comparison
                .cpu_checker_red_invariants
                .contains(invariant_a_wrong_slot_count_drags_in),
            "流 {} 上槽数按类型段那一类登记：CPU 池级 checker 不红 {invariant_a_wrong_slot_count_drags_in}（红过的：{:?}）",
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

/// 码 2 / 码 3 指针头里出生身份的三段（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）：诞生代号 8 字节在 42、写序实例 4 字节在 78、
/// 尾段（码 2 / 码 3 是出生序号）4 字节在 82。
#[derive(Clone, Copy, Debug)]
enum BirthIdentityFieldOfANodePointer {
    BirthTxg,
    Instance,
    Tail,
}

impl BirthIdentityFieldOfANodePointer {
    fn offset_in_the_pointer(self) -> usize {
        match self {
            BirthIdentityFieldOfANodePointer::BirthTxg => 42,
            BirthIdentityFieldOfANodePointer::Instance => 78,
            BirthIdentityFieldOfANodePointer::Tail => 82,
        }
    }
}

/// 攻方 F-IT 那三形（`research/prompts/m3-batch28-code-r1-opus-output.md` 第 4.2 节）：新池新建文件的流上，发布的根记录里实例表指针头部出生身份的
/// `field` 那一段最低字节加 1、重封根记录，发布那几条 journal 记录新根段里同一条指针的同一段跟着改、重封（新根段与同代的根逐字节相同，I-8.10 不红）。
/// 实例表那一片头里的诞生代号、写序实例、出生序号都是 0：指针记的不再与它相同。
fn record_new_pool_with_the_instance_table_pointer_birth_identity_off_by_one_in(
    tag: &str,
    name: &'static str,
    field: BirthIdentityFieldOfANodePointer,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    let field_offset = field.offset_in_the_pointer();
    let publish_root = flow.judged_root_index;
    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
        let offset = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + field_offset;
        bytes[offset] = bytes[offset].wrapping_add(1);
    });
    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
        let offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
            + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER
            + field_offset;
        bytes[offset] = bytes[offset].wrapping_add(1);
    });
    flow
}

/// F-IT 的判法（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「实例表链上的 I-1.2」那一行、第三节第 2 条）：CPU 池级 checker
/// 走实例表链每一片都判指针与头的出生身份（`walk.rs` `walk_instance_table_chain`），这条流上只红 I-1.2；GPU 走读内核
/// `w_walk_instance_table_chain` 每一片同判（链循环要调 `w_judge_birth_identity`，不调则 I-1.2 报成立），每个状态逐位相同。
fn assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
    test_name: &str,
    tag: &str,
    name: &'static str,
    field: BirthIdentityFieldOfANodePointer,
) {
    let flow = record_new_pool_with_the_instance_table_pointer_birth_identity_off_by_one_in(
        tag, name, field,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert_eq!(
        comparison.cpu_checker_red_invariants,
        std::collections::BTreeSet::from(["I-1.2"]),
        "流 {} 上 CPU 池级 checker 只红 I-1.2（实例表指针头部 {field:?} 那一段与那一片头里的不符）",
        flow.name
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_txg_than_its_page(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_txg_than_its_page",
        "crash-judge-gpu-i12-instance-table-pointer-birth-txg",
        "new-pool-instance-table-pointer-birth-txg-plus-one",
        BirthIdentityFieldOfANodePointer::BirthTxg,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_instance_than_its_page(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_instance_than_its_page",
        "crash-judge-gpu-i12-instance-table-pointer-instance",
        "new-pool-instance-table-pointer-instance-plus-one",
        BirthIdentityFieldOfANodePointer::Instance,
    );
}

#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_sequence_than_its_page(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_sequence_than_its_page",
        "crash-judge-gpu-i12-instance-table-pointer-birth-sequence",
        "new-pool-instance-table-pointer-birth-sequence-plus-one",
        BirthIdentityFieldOfANodePointer::Tail,
    );
}

/// GPU 关门那一侧（判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条）：最新根的实例表照 `change` 改成主走读走不到末片的样子，
/// 门要关。CPU 一侧先钉流造对了——整条落盘的镜像上 I-9.29 报部分没判。
/// 流上 C 还是最新根的那几个状态 CPU 红 I-9.29；GPU 每个状态逐位相同（实例 3 的根落了、C 写回了的状态上，GPU 门要是开了，
/// 候选集里没有实例 2 那一行、滤不掉 C，GPU 红 I-9.29 而 CPU 不红）。
fn assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
    test_name: &str,
    tag: &str,
    change: ChangeOfTheNewestInstanceTable,
) {
    let (flow, image_with_the_newest_instance_table_readable) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            tag, change,
        );
    assert_eq!(
        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
        InvariantVerdict::Holds,
        "改实例表之前：被抛弃的 C 按最新根的实例表滤掉，I-9.29 真被评估过且成立"
    );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    match pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.starts_with("最新根的实例表读不出"),
            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => panic!(
            "流 {}：主走读沿最新根的实例表链走不到末片，I-9.29 要报部分没判，实际 {other_verdict:?}",
            flow.name
        ),
    }
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.29"),
        "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
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

/// 链指针记录不合法那一形（[`ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance`]）：
/// GPU 走读内核只在链指针记录说「没有下一片」时置「走到末片」（`chain == 0u`），把不合法的也当末片时这条流红。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
        "the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid",
        "crash-judge-gpu-i929-newest-instance-table-invalid-chain-record",
        ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance,
    );
}

/// 单片读不出那一形（[`ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable`]）：第 0 片读得出、第 1 片两份都读不出。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
        "the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back",
        "crash-judge-gpu-i929-newest-instance-table-second-page-unreadable",
        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable,
    );
}

/// 攻方 H-c 那一形的流，录 `pages` 片（[`ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance`]），
/// CPU 一侧先钉流造对了：整条落盘的镜像上主走读走到第 `pages` 片（末片）、门开，C 滤不掉，I-9.29 红。
fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
    tag: &str,
    pages: InstanceTableChainPages,
) -> RecordedFlow {
    let (flow, _) =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
            tag,
            ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
                pages,
            ),
        );
    let mut image_with_every_write = flow.base.clone();
    image_with_every_write.apply_writes(&flow.writes);
    assert!(
        matches!(
            pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29"),
            InvariantVerdict::Violated(_)
        ),
        "流 {}：CPU 主走读沿 {} 片的链走到末片、候选集里没有实例 2 那一行，被抛弃的 C 与现行线同号不同出生代，I-9.29 红：{:?}",
        flow.name,
        pages.count(),
        pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29")
    );
    flow
}

/// H-c 对照那一形：链恰好 256 片（GPU 走读内核走得完的片数），两边都走到末片、都比，每个状态逐位相同、一个状态都不判不了。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks(
) {
    let flow =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
            "crash-judge-gpu-i929-newest-instance-table-of-256-pages",
            InstanceTableChainPages::AsManyAsTheGpuWalks,
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks",
    ) else {
        return;
    };
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-9.29"),
        "流 {} 上 CPU 红 I-9.29（红过的：{:?}）",
        flow.name,
        comparison.cpu_checker_red_invariants
    );
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
        flow.name,
        comparison.mismatches.state_count(),
        mismatch_listing(&comparison)
    );
}

/// H-c（判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第二节 H-c 那一行与第三节第 1 条）：链 257 片，比 GPU 走读内核走得完的多一片。
/// CPU 走到末片、比、红 I-9.29；GPU 走读内核走满上限链还没完，池级 checker 记判不了、原因字是实例表链过长
/// （[`GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED`]），不当没走到末片（当成没走到末片的话门关、不比，GPU 不红 I-9.29 而 CPU 红）。
/// 逐状态：池级 checker 判了的，违例位与 CPU 逐位相同；判不了的，原因字只能是实例表链过长，且至少一个这样的状态上 CPU 红 I-9.29；
/// 恢复内核判不了的，原因只能是沿链读满 256 片还有下一片（[`GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES`]，
/// 恢复落到那条最新根的状态）；照流水线那样判不了就回落 CPU（`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。
#[test]
fn the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks() {
    let test_name =
        "the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks";
    let flow =
        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
            "crash-judge-gpu-i929-newest-instance-table-of-257-pages",
            InstanceTableChainPages::OneMoreThanTheGpuWalks,
        );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[StateExamination::LeftToTheCpuOnlyFor {
            reasons_left_to_the_cpu: vec![
                GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED,
            ],
            // 恢复落到链 257 片那条最新根的状态上恢复内核沿链读满 256 片还有下一片，按设计交 CPU（E9 那一趟这条流 28 个状态记 4）
            recovery_reasons_left_to_the_cpu:
                GPU_RECOVERY_REASONS_LEFT_TO_THE_CPU_ON_AN_INSTANCE_TABLE_CHAIN_OF_257_PAGES
                    .to_vec(),
            reasons_one_state_must_carry: Vec::new(),
            invariant_red_on_a_state_left_to_the_cpu: Some("I-9.29"),
        }],
    ) else {
        return;
    };
    let left_to_the_cpu = &comparison.examined[0];
    eprintln!(
        "CRASH_JUDGE_GPU_CHAIN_LIMIT flow={} states={} left_to_the_cpu_by_the_chain_limit={}",
        flow.name, comparison.states, left_to_the_cpu.states_left_to_the_cpu
    );
    assert!(
        left_to_the_cpu.states_left_to_the_cpu > 0,
        "流 {}：最新根的实例表链 257 片的状态上 GPU 池级 checker 要记判不了（实例表链过长）",
        flow.name
    );
    assert!(
        left_to_the_cpu.states_left_to_the_cpu_and_red_on_the_invariant > 0,
        "流 {}：GPU 因链过长判不了的状态里，至少一个 CPU 红 I-9.29（CPU 走到末片、比）",
        flow.name
    );
    assert!(
        left_to_the_cpu.disagreements.is_empty(),
        "流 {}：{} 处不一致（前几个）：\n{}",
        flow.name,
        left_to_the_cpu.disagreements.state_count(),
        left_to_the_cpu
            .disagreements
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
