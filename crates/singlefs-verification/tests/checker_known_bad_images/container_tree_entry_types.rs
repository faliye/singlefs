//! 容器树条目的类型与身份：墓碑树、谱系树、inode 树的条目类型为零、未登记、身份指向别的容器，以及子节点的副本过期时。

use super::*;

pub(super) const CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET: usize = 18;

/// 墓碑树、谱系树根条目类型段里条款之外的一个值（I-9.23 登记 {0, 1}、I-9.17 登记 {0, 5}）。
pub(super) const CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED: u16 = 7;

/// 最新的根那一版树表里种类 `kind` 那棵容器树（墓碑树、谱系树）的码 2 根第 0 条条目改一处（`change` 拿到那 120 字节），
/// 根重封、新的整单元校验和沿引用链补到根槽。这几份镜像上这棵树的根是层级 1、直接管容器。
pub(super) fn change_the_first_entry_of_the_container_tree_root_of_kind(
    image: &mut MemoryPool,
    kind: u16,
    change: impl FnOnce(&mut [u8]),
) {
    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
    let root_slot = common_pointer_head_corruption::the_slot_of(&root_pointer);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        root_slot,
        common_pointer_head_corruption::node_bytes(),
        |root| {
            assert_eq!(
                root[INDEX_NODE_LEVEL_OFFSET], 1,
                "种类 {kind} 那棵容器树的根直接管容器"
            );
            let entries_start = 115 + 2 * usize::from(root[INDEX_NODE_KEY_WIDTH_OFFSET]);
            change(
                &mut root[entries_start
                    ..entries_start
                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES],
            );
        },
    );
}

/// 一条容器树根条目里身份引用的容器号加一（容器头不动）。
pub(super) fn add_one_to_the_identity_container_number(entry: &mut [u8]) {
    let field = CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET;
    let container = get_u64(entry, field);
    set_u64(entry, field, container + 1);
}

/// I-9.23（墓碑树条目身份与子头相符） 只红自己的那一形（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）：
/// 三个单元的文件发完意图之后，墓碑树根第一条条目身份引用里的容器号加一、容器头不动。身份不符单独判定、不再记走读失败：
/// 只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4（最新根走不完）。
#[test]
fn a_tombstone_tree_entry_whose_identity_names_another_container_reddens_only_the_tombstone_entry_invariant(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-identity",
    );
    assert_eq!(
        verdict(&clean, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
        InvariantVerdict::Holds,
        "意图之后的干净镜像上 I-9.23 真被评估过且成立"
    );
    let mut image = clean.clone();
    change_the_first_entry_of_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
        add_one_to_the_identity_container_number,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "墓碑树根条目的身份与容器头不符：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4：{verdicts:?}"
    );
}

/// I-9.23 类型段不认得那一支：墓碑树根第一条条目的类型段 1 → 7。孩子先登记引用再判红：
/// 不登记的话那片容器占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红。只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_with_an_unregistered_type_reddens_only_the_tombstone_entry_invariant() {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-unregistered-type",
    );
    let mut image = clean.clone();
    change_the_first_entry_of_the_container_tree_root_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
        |entry| {
            assert_eq!(
                u16::from_le_bytes([
                    entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET],
                    entry[CONTAINER_TREE_ENTRY_TYPE_OFFSET + 1]
                ]),
                1,
                "墓碑树根条目的类型段是 1"
            );
            set_u16(
                entry,
                CONTAINER_TREE_ENTRY_TYPE_OFFSET,
                CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
            );
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "墓碑树根条目的类型段不认得：只红 I-9.23，不连带 I-3.1：{verdicts:?}"
    );
}

/// N4-a（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行）：三个单元的文件发完意图之后，墓碑树根（层级 1）
/// 第一条条目的类型段 1 → 0，孩子还是那片码 3 墓碑容器。I-9.23 字面「0 ⇒ 子单元是码 2」不成立；孩子读多长按它头里的类标签定，
/// 不按类型段：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4（按 16 KiB 读码 3 孩子会读不出，这一小句放过、又记走读失败）。
#[test]
fn a_tombstone_tree_entry_of_type_zero_over_a_container_reddens_only_the_tombstone_entry_invariant()
{
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-type-zero-over-a-container",
    );
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
    });
    assert_eq!(level, 1, "墓碑树根直接管容器");
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "类型段 0 的条目指着码 3 容器：只红 I-9.23，不连带 I-4.8、I-7.2、I-7.4：{verdicts:?}"
    );
    assert_the_violation_names_the_child_class(
        &image,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
}

/// N4-i：同 N4-a，再把身份引用的出生树、容器号、出生代三段清零——这条条目除了孩子的类之外都是一条合法的类型段 0 条目。
/// 孩子读不出时「子单元是码 2」那一小句若放过，I-9.23 报成立（漏判）；只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_of_type_zero_with_a_zero_identity_over_a_container_reddens_only_the_tombstone_entry_invariant(
) {
    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-type-zero-zero-identity-over-a-container",
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_TOMBSTONE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "身份三段为 0 的类型段 0 条目指着码 3 容器：只红 I-9.23：{verdicts:?}"
    );
}

/// N4-c（谱系树那一侧，I-9.17 与 I-9.23 同形）：回退到 A 之后，谱系树根第一条条目的类型段 5 → 0，孩子还是那片码 3 谱系容器。只红 I-9.17。
#[test]
fn a_lineage_tree_entry_of_type_zero_over_a_container_reddens_only_the_lineage_entry_invariant() {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-type-zero");
    let mut image = rolled_back.image.clone();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_LINEAGE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "谱系树类型段 0 的条目指着码 3 容器：只红 I-9.17：{verdicts:?}"
    );
    assert_the_violation_names_the_child_class(
        &image,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
}

/// I-9.17 类型段不认得那一支（M1 谱系那一侧，判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K1 覆盖缺口那一行）：
/// 回退到 A 之后，谱系树根第一条条目的类型段 5 → 7。孩子先登记引用再判红：只红 I-9.17，不连带 I-3.1。
#[test]
fn a_lineage_tree_entry_with_an_unregistered_type_reddens_only_the_lineage_entry_invariant() {
    let rolled_back =
        image_after_rolling_back_to_the_first_root("known-bad-lineage-entry-unregistered-type");
    let mut image = rolled_back.image.clone();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_LINEAGE,
            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
        );
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "谱系树根条目的类型段不认得：只红 I-9.17，不连带 I-3.1：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict(&image, LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
            InvariantVerdict::Violated(detail) if detail.contains("类型段 7 不在 {0, 5} 里")
        ),
        "违例说明写出不认得的类型段"
    );
}

/// I-9.2（inode 树条目身份与子头相符，与 I-9.17 / I-9.23 同形）：新池新建文件那一版，inode 树根（层级 1）第一条条目的类型段 2 → 0，
/// 孩子还是那片码 3 叶容器。I-9.2 字面「类型段 0 ⇒ 子单元偏移 6 = 2」不成立；孩子读多长按它头里的类标签定：只红 I-9.2，
/// 不连带 I-2.1（按 16 KiB 读 32 KiB 的叶容器，校验和会对不上，I-2.1 红、记走读失败，I-9.2 一次都判不到）。
#[test]
fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_reddens_only_the_inode_entry_invariant() {
    let mut image = build_pool("known-bad-inode-entry-type-zero-over-a-leaf").memory_pool();
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
    });
    assert_eq!(level, 1, "inode 树根直接管叶容器");
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.2"],
        "inode 树类型段 0 的条目指着码 3 叶容器：只红 I-9.2：{verdicts:?}"
    );
    assert_the_violation_names_the_child_class(
        &image,
        "I-9.2",
        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        "PackedRecordUnit",
    );
}

/// 同上，再把身份引用三段清零：除了孩子的类，这是一条合法的类型段 0 条目。只红 I-9.2。
#[test]
fn an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant(
) {
    let mut image =
        build_pool("known-bad-inode-entry-type-zero-zero-identity-over-a-leaf").memory_pool();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
    });
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.2"],
        "身份三段为 0 的 inode 树类型段 0 条目指着码 3 叶容器：只红 I-9.2：{verdicts:?}"
    );
}

/// 写这几份长高的墓碑树时容器与节点压到的大小（照 `container_trees_grow_a_level_when_the_root_is_full.rs`）：一片 1 条、一个码 2 节点 2 条，
/// 第 3 片容器根满长一层。池级 checker 走带旋钮值的入口，旋钮值就是一片 1 条。
const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;

fn capacities_of_one_record_and_two_entries(
) -> singlefs_core::container_tree::ContainerTreeCapacities {
    singlefs_core::container_tree::ContainerTreeCapacities {
        tombstone_container:
            singlefs_core::tombstone_tree::TombstoneContainerCapacity::CappedForTests(
                singlefs_core::tombstone_tree::TombstoneRecordsPerContainerCappedForTests::new(
                    TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
                )
                .expect("1..=815"),
            ),
        lineage_container: singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
            singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(1)
                .expect("1..=1359"),
        ),
        retirement_container:
            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
        node_entries: singlefs_core::container_tree::ContainerTreeNodeEntries::CappedForTests(
            singlefs_core::container_tree::ContainerTreeNodeEntriesCappedForTests::new(2)
                .expect("2..=135"),
        ),
    }
}

/// unlink 一个空 inode 到底（意图，再走空），写入口装一片 1 条、一个节点 2 条的旋钮。
fn unlink_an_empty_inode_with_one_record_per_tombstone_container(pool: &mut BuiltPool, inode: u64) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    writer.set_container_tree_capacities(capacities_of_one_record_and_two_entries());
    let current = singlefs_core::transaction::publish_unlink_intent(
        &mut writer,
        &mut pool.allocator,
        &previous,
        singlefs_core::address::InodeNumber(inode),
        common_deletion_in_progress::WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
    )
    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
    let mut current = std::sync::Arc::new(current);
    singlefs_core::transaction::publish_unlink_steps_until_the_queue_is_empty(
        &mut writer,
        &mut pool.allocator,
        &mut current,
        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
    )
    .expect("空 inode 一批就走空");
    pool.output = std::sync::Arc::unwrap_or_clone(current);
}

/// 建 5 个空 inode（2..=6 号），在旋钮下 unlink `inodes` 里那几个：两个时墓碑树根层级 1、两片容器；三个时根满长到层级 2。
fn pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
    tag: &str,
    inodes: &[u64],
) -> BuiltPool {
    let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, 5);
    for inode in inodes {
        unlink_an_empty_inode_with_one_record_per_tombstone_container(&mut pool, *inode);
    }
    pool
}

/// 池级 checker 带旋钮值（一片 1 条）的入口。
pub(super) fn check_pool_image_under_the_tombstone_knob(
    image: &MemoryPool,
) -> Vec<(&'static str, InvariantVerdict)> {
    singlefs_checker::walk::check_pool_image_given_the_tombstone_container_capacity_knob(
        image,
        Some(
            singlefs_checker::walk::TombstoneContainerCapacityKnob::new(
                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
            )
            .expect("旋钮值在 1..=815 里"),
        ),
    )
}

/// 带旋钮值的入口判出来的这一条的判定。
pub(super) fn verdict_under_the_tombstone_knob(
    image: &MemoryPool,
    invariant: &str,
) -> InvariantVerdict {
    check_pool_image_under_the_tombstone_knob(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有")
        .1
}

/// 墓碑树长到层级 2 的干净镜像（2、3、4 号在旋钮下删完）；带旋钮值的入口上一条都不红。
pub(super) fn image_with_a_tombstone_tree_of_level_two(tag: &str) -> MemoryPool {
    let image =
        pool_with_empty_inodes_unlinked_under_the_tombstone_knob(tag, &[2, 3, 4]).memory_pool();
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        Vec::<&str>::new(),
        "长到层级 2 的干净镜像（带旋钮值）一条都不红：{verdicts:?}"
    );
    image
}

/// N4-f：墓碑树长到层级 2 之后，根第一条条目（类型段 0，孩子是层级 1 的码 2 节点）类型段 0 → 1。I-9.23 字面「1 ⇒ 子单元是码 3」不成立；
/// 孩子读多长按它头里的类标签定（按 32 KiB 读 16 KiB 的节点，I-2.1 会红、记走读失败，身份那一判跑不到、I-9.23 报成立）。
/// 孩子按码 2 一槽登记引用、不往下走：只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_of_the_container_type_over_a_node_reddens_only_the_tombstone_entry_invariant(
) {
    let mut image = image_with_a_tombstone_tree_of_level_two(
        "known-bad-tombstone-entry-container-type-over-a-node",
    );
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            PACKED_RECORD_TYPE_TOMBSTONE,
        );
    });
    assert_eq!(level, 2, "墓碑树根长到层级 2");
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "类型段 1 的条目指着码 2 节点：只红 I-9.23，不连带 I-2.1、I-3.1、I-3.11、I-4.8、I-5.1、I-7.2、I-7.4：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict_under_the_tombstone_knob(&image, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
            InvariantVerdict::Violated(detail)
                if detail.contains("类型段 1 说孩子是") && detail.contains("孩子头里的类标签是 IndexNode")
        ),
        "违例说明写出类型段 1 与孩子头里的码 2"
    );
}

/// N4-j：同 N4-f，墓碑树长到层级 2 之后再建 30 次（根环转过一圈），根之下那几个节点只剩现行这棵墓碑树引用、别的候选版不再替它们登记引用。只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_of_the_container_type_over_a_node_after_the_root_ring_turned_over_reddens_only_the_tombstone_entry_invariant(
) {
    let mut pool = pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
        "known-bad-tombstone-entry-container-type-over-a-node-ring-turned-over",
        &[2, 3, 4],
    );
    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
    let clean = pool.memory_pool();
    assert_eq!(
        violated_invariants(&check_pool_image_under_the_tombstone_knob(&clean)),
        Vec::<&str>::new(),
        "根环转过一圈之后的干净镜像（带旋钮值）一条都不红"
    );
    let mut image = clean.clone();
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        change_the_entry_type(
            entry,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            PACKED_RECORD_TYPE_TOMBSTONE,
        );
    });
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "根环转过一圈之后类型段 1 的条目指着码 2 节点：只红 I-9.23：{verdicts:?}"
    );
}

/// M1（类型段 0 那一支的身份判定）：墓碑树长到层级 2 之后，根第一条条目（类型段 0、孩子是码 2 节点）
/// 身份引用的容器号写成 1——类型段 0 要身份三段为 0。只红 I-9.23。
#[test]
fn a_tombstone_tree_entry_of_type_zero_with_a_container_number_reddens_only_the_tombstone_entry_invariant(
) {
    let mut image = image_with_a_tombstone_tree_of_level_two(
        "known-bad-tombstone-entry-type-zero-container-number",
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
        assert_eq!(
            entry_type_of(entry),
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            "根长高之后第一条条目类型段 0"
        );
        assert_eq!(
            get_u64(entry, CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET),
            0,
            "类型段 0 的条目容器号是 0"
        );
        set_u64(entry, CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET, 1);
    });
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
        "类型段 0 的条目身份容器号不为 0：只红 I-9.23：{verdicts:?}"
    );
    assert!(
        matches!(
            verdict_under_the_tombstone_knob(&image, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
            InvariantVerdict::Violated(detail) if detail.contains("0 / 1 / 0（要三段为 0）")
        ),
        "违例说明写出身份三段"
    );
}

/// M5（I-9.21 / I-9.22「容器号沿叶序严格递增」）：旋钮下 2、3 号删完，墓碑树根层级 1、两片容器；
/// 第二片容器头里的容器号与根里指它那条条目的身份容器号一起改成第一片的号（身份照旧相符，分隔 key 不动）。带旋钮值的入口判 I-9.22：只红它。
#[test]
fn two_tombstone_containers_with_one_container_number_redden_only_the_capacity_knob_invariant() {
    let pool = pool_with_empty_inodes_unlinked_under_the_tombstone_knob(
        "known-bad-tombstone-containers-share-a-number",
        &[2, 3],
    );
    let mut image = pool.memory_pool();
    assert_eq!(
        verdict_under_the_tombstone_knob(
            &image,
            TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE
        ),
        InvariantVerdict::Holds,
        "两片容器的干净镜像上 I-9.22 真被评估过且成立"
    );
    let root_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
    let root =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            common_pointer_head_corruption::the_slot_of(&root_pointer),
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("墓碑树根解得开");
    assert_eq!(
        (root.level, root.entries.len()),
        (1, 2),
        "墓碑树根层级 1、两片容器"
    );
    let first_container_number = get_u64(
        &root.entries[0],
        CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
    );
    let second_container_pointer =
        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
            &root.entries[1],
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
        ));
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&second_container_pointer),
        common_pointer_head_corruption::data_unit_bytes(),
        |container| {
            assert!(
                get_u64(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET) > first_container_number,
                "第二片的容器号大于第一片的"
            );
            set_u64(
                container,
                PACKED_UNIT_CONTAINER_NUMBER_OFFSET,
                first_container_number,
            );
        },
    );
    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 1, |entry| {
        set_u64(
            entry,
            CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET,
            first_container_number,
        );
    });
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        [TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE],
        "两片墓碑容器同号：只红 I-9.22：{verdicts:?}"
    );
}

/// 孩子头里的类标签只在「按那一类的长度读回、整单元校验和对得上」时才算孩子的类（`walk` 认孩子类的读法）：新池新建文件那一版，
/// inode 叶容器（32 KiB）头里的类标签 3 → 2，不重封，整单元校验和沿引用链补上（位置条目对得上这 32 KiB）。类标签说码 2，
/// 按 16 KiB 读回对不上校验和，不当成「另一类的孩子」：照类型段 2 那一支读 32 KiB，I-9.2 判类标签不是 3（违例说明写「（类 2）」），
/// 孩子照两槽登记引用，I-3.1 不连带红。
#[test]
fn an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type() {
    let mut image = build_pool("known-bad-inode-leaf-class-tag-alone").memory_pool();
    mutate_unit(&mut image, INODE_LEAF, |bytes| bytes[6] = 2, false);
    assert!(
        matches!(
            verdict(&image, "I-9.2"),
            InvariantVerdict::Violated(detail) if detail.contains("（类 2）不符")
        ),
        "I-9.2 照类型段 2 那一支判出孩子头的类不是 3：{:?}",
        verdict(&image, "I-9.2")
    );
    assert!(
        !matches!(verdict(&image, "I-3.1"), InvariantVerdict::Violated(_)),
        "叶容器照两槽登记引用：I-3.1 不红：{:?}",
        verdict(&image, "I-3.1")
    );
}

/// 攻方 H1（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行）的判法：
/// 条目改坏（类型段与孩子的类不符）之外，孩子在一块盘上那一份陈旧、陈旧那一份头里的类恰好是类型段说的那一类，另一块盘那一份完好。
/// 两种盘序各判一次：红的恰好是「只有这份陈旧副本」那张镜像红的那几条再加 `entry_invariant`——陈旧副本自己连带的照红，
/// 条目那一条不随陈旧副本在哪块盘翻面，也不连带 I-7.2（陈旧的在盘 0 时若只认第一份的头、不验、不看第二份，
/// `entry_invariant` 就报成立，另多红 I-7.2，层级 2 与 inode 那两形再多 I-3.1、I-3.11、I-5.1）。
fn assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
    clean: &MemoryPool,
    image_with_the_entry_changed: &MemoryPool,
    child: &singlefs_core::pointer::NodePointer,
    stale_bytes: &[u8],
    entry_invariant: &'static str,
    check: impl Fn(&MemoryPool) -> Vec<(&'static str, InvariantVerdict)>,
) {
    let mut violated_on_each_device = Vec::new();
    for device in DEVICES {
        let mut stale_copy_only = clean.clone();
        make_the_copy_on_one_device_stale(&mut stale_copy_only, device, child, stale_bytes);
        let violated_by_the_stale_copy_only = violated_invariant_set(&check(&stale_copy_only));
        assert!(
            !violated_by_the_stale_copy_only.contains(entry_invariant),
            "只有盘 {device} 上那份陈旧副本、条目不改：{entry_invariant} 不红（{violated_by_the_stale_copy_only:?}）"
        );
        let mut image = image_with_the_entry_changed.clone();
        make_the_copy_on_one_device_stale(&mut image, device, child, stale_bytes);
        let verdicts = check(&image);
        let violated = violated_invariant_set(&verdicts);
        let mut expected = violated_by_the_stale_copy_only.clone();
        expected.insert(entry_invariant);
        assert_eq!(
            violated, expected,
            "条目改坏、孩子在盘 {device} 上那份陈旧：红的是只有陈旧副本时那几条再加 {entry_invariant}：{verdicts:?}"
        );
        assert!(
            !violated.contains("I-7.2"),
            "条目改坏、孩子在盘 {device} 上那份陈旧：不连带 I-7.2：{verdicts:?}"
        );
        violated_on_each_device.push(violated);
    }
    assert_eq!(
        violated_on_each_device[0], violated_on_each_device[1],
        "同一张镜像只差陈旧副本在哪块盘：红的同一批"
    );
}

/// H1 的 N4-i 那一形（攻方 `research/prompts/m3-batch24-code-r1-opus-model/expected-summary.txt` 第 391 / 392 行那一对）：三个单元的文件发完意图之后，
/// 墓碑树根（层级 1）第一条条目类型段 1 → 0、身份三段清零，孩子还是那片码 3 墓碑容器；孩子在一块盘上那一份换成墓碑树根（码 2）的原样字节。
#[test]
fn a_tombstone_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-entry-type-zero-over-a-container-stale-copy",
    );
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_TOMBSTONE, 0);
    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_TOMBSTONE);
    assert_eq!(
        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
        "陈旧副本头里的类就是类型段 0 说的码 2"
    );
    let mut image_with_the_entry_changed = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_the_entry_changed,
        TREE_KIND_TOMBSTONE,
        0,
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_TOMBSTONE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
    );
    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
        &clean,
        &image_with_the_entry_changed,
        &child,
        &stale_bytes,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        |image| check_pool_image(image),
    );
}

/// H1 的 N4-f 那一形（攻方 T2）：墓碑树长到层级 2 之后，根第一条条目类型段 0 → 1，孩子还是层级 1 的码 2 节点；
/// 孩子在一块盘上那一份换成一片墓碑容器的前 16 KiB（头里类标签 3）。带旋钮值的入口判。
#[test]
fn a_tombstone_tree_entry_of_the_container_type_over_a_node_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device(
) {
    let clean = image_with_a_tombstone_tree_of_level_two(
        "known-bad-tombstone-entry-container-type-over-a-node-stale-copy",
    );
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_TOMBSTONE, 0);
    let first_entry_of_the_child =
        common_pointer_head_corruption::first_entry_of_the_node_at(&clean, &child);
    let container_below_the_child =
        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
            &first_entry_of_the_child,
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
        ));
    let stale_bytes = common_pointer_head_corruption::unit_bytes_at(
        &clean,
        common_pointer_head_corruption::the_slot_of(&container_below_the_child),
        common_pointer_head_corruption::node_bytes(),
    );
    assert_eq!(
        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_PACKED_RECORD_UNIT,
        "陈旧副本头里的类就是类型段 1 说的码 3"
    );
    let mut image_with_the_entry_changed = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_the_entry_changed,
        TREE_KIND_TOMBSTONE,
        0,
        |entry| {
            change_the_entry_type(
                entry,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
                PACKED_RECORD_TYPE_TOMBSTONE,
            );
        },
    );
    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
        &clean,
        &image_with_the_entry_changed,
        &child,
        &stale_bytes,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        check_pool_image_under_the_tombstone_knob,
    );
}

/// H1 谱系树那一侧（I-9.17 与 I-9.23 同形）：回退到 A 之后，谱系树根第一条条目类型段 5 → 0、身份三段清零，孩子还是那片码 3 谱系容器；
/// 孩子在一块盘上那一份换成谱系树根（码 2）的原样字节。
#[test]
fn a_lineage_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_lineage_entry_invariant_on_either_device(
) {
    let clean = image_after_rolling_back_to_the_first_root(
        "known-bad-lineage-entry-type-zero-over-a-container-stale-copy",
    )
    .image;
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_LINEAGE, 0);
    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_LINEAGE);
    assert_eq!(
        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
        "陈旧副本头里的类就是类型段 0 说的码 2"
    );
    let mut image_with_the_entry_changed = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_the_entry_changed,
        TREE_KIND_LINEAGE,
        0,
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_LINEAGE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
    );
    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
        &clean,
        &image_with_the_entry_changed,
        &child,
        &stale_bytes,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        |image| check_pool_image(image),
    );
}

/// H1 的 inode 那一形（攻方 I0 type0-zero-identity）：新池新建文件那一版，inode 树根（层级 1）第一条条目类型段 2 → 0、身份三段清零，
/// 孩子还是那片码 3 叶容器；孩子在一块盘上那一份换成 inode 树根（码 2）的原样字节。
#[test]
fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_with_a_stale_copy_of_the_child_reddens_the_inode_entry_invariant_on_either_device(
) {
    let clean = build_pool("known-bad-inode-entry-type-zero-over-a-leaf-stale-copy").memory_pool();
    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_INODE, 0);
    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_INODE);
    assert_eq!(
        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
        "陈旧副本头里的类就是类型段 0 说的码 2"
    );
    let mut image_with_the_entry_changed = clean.clone();
    change_an_entry_of_the_root_of_kind(
        &mut image_with_the_entry_changed,
        TREE_KIND_INODE,
        0,
        |entry| {
            change_the_entry_type(
                entry,
                PACKED_RECORD_TYPE_INODE,
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
            );
            zero_the_three_identity_fields(entry);
        },
    );
    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
        &clean,
        &image_with_the_entry_changed,
        &child,
        &stale_bytes,
        "I-9.2",
        |image| check_pool_image(image),
    );
}

/// C297（inode 内部条目里出生树两份没有定处理） 孩子读得出的那一支的判法：种类 `kind` 那棵树的根（层级 1）第一条条目类型段是容器那一类
/// （`container_entry_type`），只把条目里子指针头部的出生树加 1（身份引用、孩子都不改，根重封、新的整单元校验和沿引用链补到根槽）。
/// 孩子照指针自己的位置条目读得出（映射按改过的出生树查不到这把 key，走读退回位置条目），孩子头与身份引用逐段相等，
/// 只有「子指针头部的出生树 == 身份引用的出生树」那一比不成立：`entry_invariant` 红，违例说明是孩子读得出那一句（不是「两份都读不出」那一句）。
/// 干净镜像上 `entry_invariant` 判过且成立。判别力：拿掉读得出那一支里的那一比（`walk.rs` 的 `entry_holds` / `container_entry_holds`），这一条红。
fn assert_a_child_pointer_birth_tree_one_off_over_a_child_that_reads_back_reddens_the_entry_invariant(
    clean: &MemoryPool,
    kind: u16,
    container_entry_type: u16,
    entry_invariant: &'static str,
) {
    assert_eq!(
        verdict(clean, entry_invariant),
        InvariantVerdict::Holds,
        "对照：干净镜像上 {entry_invariant} 判过且成立"
    );
    let mut image = clean.clone();
    let mut birth_trees_after_the_change = (0, 0);
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        assert_eq!(
            entry_type_of(entry),
            container_entry_type,
            "根第一条条目指容器"
        );
        add_one_to_a_birth_tree_field_of_the_entry(entry, INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE);
        birth_trees_after_the_change = (
            get_u64(entry, INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE.start),
            get_u64(entry, INTERNAL_ENTRY_IDENTITY_BIRTH_TREE.start),
        );
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    let (pointer_birth_tree, identity_birth_tree) = birth_trees_after_the_change;
    assert_eq!(
        pointer_birth_tree,
        identity_birth_tree + 1,
        "子指针头部的出生树比身份引用的大 1"
    );
    let found = verdict(&image, entry_invariant);
    assert!(
        matches!(
            &found,
            InvariantVerdict::Violated(detail)
                if detail.contains("内部条目的身份") && !detail.contains("两份都读不出")
        ),
        "子指针头部的出生树 {pointer_birth_tree} 不等于身份引用的出生树 {identity_birth_tree}、孩子读得出：{entry_invariant} 照读得出那一支红：{found:?}"
    );
}

#[test]
fn an_inode_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_leaf_that_reads_back_reddens_the_inode_entry_invariant(
) {
    assert_a_child_pointer_birth_tree_one_off_over_a_child_that_reads_back_reddens_the_entry_invariant(
        &build_pool("known-bad-inode-entry-pointer-birth-tree-readable-leaf").memory_pool(),
        TREE_KIND_INODE,
        PACKED_RECORD_TYPE_INODE,
        "I-9.2",
    );
}

#[test]
fn a_tombstone_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_container_that_reads_back_reddens_the_tombstone_entry_invariant(
) {
    assert_a_child_pointer_birth_tree_one_off_over_a_child_that_reads_back_reddens_the_entry_invariant(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-pointer-birth-tree-readable-container",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}
