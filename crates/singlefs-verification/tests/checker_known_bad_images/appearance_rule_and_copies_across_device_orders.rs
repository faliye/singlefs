//! 出现规则与两种设备次序：一层节点下的容器副本、回退之后共享的叶子容器、两份都校验不过的副本、被弄乱的共享根指针。

use super::*;

/// 码 3 打包记录单元头里记录宽（2 字节）的偏移（D18（块里携带什么信息） 已定项 11：记录数 69、记录宽 71）。
pub(super) const PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET: usize = 71;

/// 旋钮下长到层级 2 的那张镜像里两块盘上都没写过的一段（32 KiB）：远在单元区写过的那几槽之后、4 GiB 的盘之内（写之前核全零）。
const SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE: u64 = 180_000;

/// G3（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G3 那一行、第三节第 3 条）：层级 2 的树里类型段 0、
/// F-W 往下走那一份子树。墓碑树在旋钮下长到层级 2（[`image_with_a_tombstone_tree_of_level_two`]），根最后一条条目（类型段 0）指层级 1 的节点 N，
/// N 最后一条条目指墓碑容器 C（旋钮一片只装一条墓碑，最后一次 unlink 的墓碑另起一片：C 与 N 只有最新那几版引用）。C 两份的记录宽 40 → 39
/// （重封、补到根）；再把 C 的 32 KiB 原样字节写进位置条目 `location_index` 那块盘上一段没写过的槽 [`SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE`]、
/// 根那条条目的那条位置条目改指它、校验和换成它的 CRC（那一份自证成码 3，另一类；不写进 N 自己的槽：N 是 16 KiB，写 32 KiB 会盖掉下一槽的单元），
/// 另一条位置条目指的 N 不动（码 2，类型段 0 那一类，自证得过）。F-W：I-9.23 红，照类型段 0 那一支往下读 N 那一份，N 下面 C 的 I-1.7 违例照红。
/// 类型段 0 那一处见别类就停（正推 S1）时 N 不往下走，I-1.7 不红。
fn assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(
    location_index: usize,
) {
    let mut image = image_with_a_tombstone_tree_of_level_two(&format!(
        "known-bad-tombstone-level-two-node-copy-verifying-as-a-container-location-{location_index}"
    ));
    let root_pointer =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
    let root =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            common_pointer_head_corruption::the_slot_of(&root_pointer),
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("根解得开");
    assert_eq!(root.level, 2, "墓碑树根长到层级 2");
    let last_entry_index = root.entries.len() - 1;
    let node_pointer = child_pointer_of_an_entry_of_the_root_of_kind(
        &image,
        TREE_KIND_TOMBSTONE,
        last_entry_index,
    );
    let node_slot = common_pointer_head_corruption::the_slot_of(&node_pointer);
    let node =
        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            node_slot,
            common_pointer_head_corruption::node_bytes(),
        ))
        .expect("层级 1 的节点解得开");
    assert_eq!(node.level, 1, "根最后一条条目指层级 1 的节点");
    let container_pointer =
        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
            node.entries.last().expect("层级 1 的节点至少一条条目"),
            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
        ));
    let container_slot = common_pointer_head_corruption::the_slot_of(&container_pointer);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        container_slot,
        common_pointer_head_corruption::data_unit_bytes(),
        |container| {
            assert_eq!(
                container[UNIT_HEADER_CLASS_TAG_OFFSET], 3,
                "N 下面最后一个孩子是码 3 墓碑容器"
            );
            let width_field =
                PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET..PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET + 2;
            assert_eq!(
                u64::from(u16::from_le_bytes([
                    container[width_field.start],
                    container[width_field.start + 1]
                ])),
                singlefs_format::TOMBSTONE_RECORD_BYTES,
                "墓碑容器的记录宽是登记的 40"
            );
            container[width_field].copy_from_slice(
                &record_width_one_below(singlefs_format::TOMBSTONE_RECORD_BYTES).to_le_bytes(),
            );
        },
    );
    let container = common_pointer_head_corruption::unit_bytes_at(
        &image,
        container_slot,
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let device_of_the_container_copy = node_pointer.locations[location_index].device.0;
    assert!(
        read(
            &image,
            device_of_the_container_copy,
            SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE * SLOT,
            common_pointer_head_corruption::data_unit_bytes(),
        )
        .iter()
        .all(|byte| *byte == 0),
        "槽 {SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE} 起 32 KiB 在盘 {device_of_the_container_copy} 上没写过"
    );
    write(
        &mut image,
        device_of_the_container_copy,
        SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE * SLOT,
        &container,
    );
    let checksum_of_the_container = crc32_castagnoli(&container);
    let level = change_an_entry_of_the_root_of_kind(
        &mut image,
        TREE_KIND_TOMBSTONE,
        last_entry_index,
        |entry| {
            assert_eq!(
                entry_type_of(entry),
                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
                "层级 2 的根条目类型段 0"
            );
            set_the_slot_in_a_child_location(
                entry,
                location_index,
                SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE,
            );
            set_the_whole_unit_checksum_in_a_child_location(
                entry,
                location_index,
                checksum_of_the_container,
            );
        },
    );
    assert_eq!(level, 2, "墓碑树根长到层级 2");
    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
    assert!(
        matches!(
            verdict_under_the_tombstone_knob(&image, "I-1.7"),
            InvariantVerdict::Violated(detail)
                if detail.contains(&format!("记录宽 {}", singlefs_format::TOMBSTONE_RECORD_BYTES - 1))
        ),
        "码 3 那一份在位置条目 {location_index}：照类型段 0 那一支往下读 N 那一份，N 下面那片墓碑容器的 I-1.7 违例照红：{verdicts:?}"
    );
    assert!(
        violated_invariant_set(&verdicts).contains(TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
        "码 3 那一份在位置条目 {location_index}：根那条条目照判 I-9.23 红：{verdicts:?}"
    );
}

#[test]
fn a_tombstone_container_below_a_level_one_node_whose_first_copy_verifies_as_a_container_reddens_the_packed_container_invariant(
) {
    assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(0);
}

#[test]
fn a_tombstone_container_below_a_level_one_node_whose_second_copy_verifies_as_a_container_reddens_the_packed_container_invariant(
) {
    assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(1);
}

// ─── 删除线：一个单元的各把键按读回那一份的落点认（allrbmap）、F-ID-b 两份都没自证过、几处 F-KEY 与 G4 补的坏镜像 ───

/// allrbmap 的判法（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P1 没改的几把键那一行、第三节第 1 条；攻方
/// `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.2、1.3 节 today 臂换盘判得不同的那 6 对）：种类 `kind` 那棵树根第一条条目子指针的一条
/// 位置条目指着这一遍先走过、又被更早的候选根共享的码 2 节点（种类 `kind_walked_before` 那棵树的根）、整单元校验和对不上，另一条位置条目那一份是
/// 按自己的位置条目自证得过的容器（记录宽照 `record_width`）。两种盘序（指着先走过的节点的是位置条目 0 / 1）各判一次：红的都是 `expected_violations`，
/// I-4.9 与 I-3.5 都成立——读回那片容器的诞生代号记在它自己读回的落点上、不盖掉先走过那个节点的，I-3.5 的折叠表把那个落点记成它自己。
fn assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
    clean: &MemoryPool,
    kind: u16,
    record_width: RecordWidthOfTheFirstContainer,
    kind_walked_before: u16,
    expected_violations: &[&'static str],
) {
    assert_both_device_orders_judge_alike_with_the_unit_birth_rule_holding_and_the_appearance_rule(
        clean,
        kind,
        record_width,
        kind_walked_before,
        expected_violations,
        AppearanceRuleOnBothDeviceOrders::Holds,
    );
}

/// I-3.5 在两种盘序上该判成什么：成立，或谱系树读不全、要靠谱系那一半的落点报部分没判（[`assert_both_device_orders_judge_alike_with_the_unit_birth_rule_holding_and_the_appearance_rule`]）。
#[derive(Clone, Copy, Debug)]
enum AppearanceRuleOnBothDeviceOrders {
    Holds,
    /// 回退那一版（D，txg 9）谱系树里那片容器记录宽改坏、读不全：D 与它前一版（C，txg 8）那一对里，诞生代号不大于 8、落在整棵继承下来的树里的落点
    /// 照 `crates/singlefs-checker/src/walk.rs` 谱系读不全那一句报部分没判（`crates/singlefs-checker/invariants/05-空间记账.md` I-3.5 那一行：逐落点，
    /// 只把靠谱系那一半判的落点记部分没判，同一对里别的落点照判）。
    PartiallyJudgedWhereTheLineageTreeOfTheRollbackVersionDoesNotReadBack,
}

/// 谱系读不全那一句（`crates/singlefs-checker/src/walk.rs` 里 I-3.5 判出现规则那一支）里不随落点变的两段：这一对是谁，与谱系判不了的说法。
const APPEARANCE_RULE_PAIR_ENDING_AT_THE_ROLLBACK_VERSION: &str =
    "相邻两版 txg 8 → (实例 2、txg 9)：后一版新引用的落点";

const APPEARANCE_RULE_LINEAGE_HALF_NOT_JUDGED_AS_THE_LINEAGE_TREE_DOES_NOT_READ_BACK: &str =
    "落在整棵继承下来的树里，后一版的谱系树读不全（LineageTreeUnreadable），谱系那一半判不了";

/// 同 [`assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding`]，I-4.9 成立，I-3.5 在两种盘序上都照 `appearance_rule` 判：
/// 都成立，或都报部分没判、点名同一对与同一句谱系判不了的理由（没判的落点逐条列、带落点与折成的单元，只比这一对与理由那两段）。
fn assert_both_device_orders_judge_alike_with_the_unit_birth_rule_holding_and_the_appearance_rule(
    clean: &MemoryPool,
    kind: u16,
    record_width: RecordWidthOfTheFirstContainer,
    kind_walked_before: u16,
    expected_violations: &[&'static str],
    appearance_rule: AppearanceRuleOnBothDeviceOrders,
) {
    for location_index in 0..2 {
        let image = image_whose_first_root_entry_names_a_walked_node_beside_a_container(
            clean,
            kind,
            record_width,
            kind_walked_before,
            location_index,
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariant_set(&verdicts),
            expected_violations
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<&'static str>>(),
            "指着先走过的节点的是位置条目 {location_index}：两种盘序红的同一批：{verdicts:?}"
        );
        let verdict_of = |invariant: &str| {
            verdicts
                .iter()
                .find(|(identifier, _)| *identifier == invariant)
                .map(|(_, found)| found.clone())
                .expect("清单里有")
        };
        assert_eq!(
            verdict_of(UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT),
            InvariantVerdict::Holds,
            "指着先走过的节点的是位置条目 {location_index}：{UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT} 按落点认、读落点上单元头里的诞生代号，成立"
        );
        let appearance_rule_verdict = verdict_of(REFERENCE_INTERVALS_ARE_EXACT);
        match appearance_rule {
            AppearanceRuleOnBothDeviceOrders::Holds => assert_eq!(
                appearance_rule_verdict,
                InvariantVerdict::Holds,
                "指着先走过的节点的是位置条目 {location_index}：{REFERENCE_INTERVALS_ARE_EXACT} 按落点认、读落点上单元头里的诞生代号，成立"
            ),
            AppearanceRuleOnBothDeviceOrders::PartiallyJudgedWhereTheLineageTreeOfTheRollbackVersionDoesNotReadBack => {
                assert!(
                    matches!(
                        &appearance_rule_verdict,
                        InvariantVerdict::PartiallyJudged(unjudged)
                            if unjudged.starts_with(APPEARANCE_RULE_PAIR_ENDING_AT_THE_ROLLBACK_VERSION)
                                && unjudged.contains(APPEARANCE_RULE_LINEAGE_HALF_NOT_JUDGED_AS_THE_LINEAGE_TREE_DOES_NOT_READ_BACK)
                    ),
                    "指着先走过的节点的是位置条目 {location_index}：{REFERENCE_INTERVALS_ARE_EXACT} 在回退那一版谱系树读不全时，\
                     靠谱系那一半判的落点报部分没判、点名 C 与 D 那一对：{appearance_rule_verdict:?}"
                );
            }
        }
    }
}

/// 墓碑撞 extent 树根、容器记录宽 40 → 39（攻方 `base=tomb walked=extent variant=bad`）：三个单元的文件发完意图之后，extent 树根被更早那条候选根共享。
#[test]
fn a_tombstone_container_with_a_wrong_record_width_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
) {
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-record-width-location-naming-the-shared-extent-root",
        ),
        TREE_KIND_TOMBSTONE,
        RecordWidthOfTheFirstContainer::OneBelowTheRegistered {
            record_bytes: singlefs_format::TOMBSTONE_RECORD_BYTES,
        },
        TREE_KIND_EXTENT,
        &[
            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
        ],
    );
}

/// 墓碑撞 extent 树根、容器不改（攻方 `base=tomb walked=extent variant=good`）。
#[test]
fn a_tombstone_container_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
) {
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-location-naming-the-shared-extent-root",
        ),
        TREE_KIND_TOMBSTONE,
        RecordWidthOfTheFirstContainer::AsRegistered,
        TREE_KIND_EXTENT,
        &["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
    );
}

/// 谱系撞 inode 树根、容器不改（攻方 `base=lineage walked=inode variant=good`）：回退到 A 之后，回退那一版照抄 A 的 inode 树。
/// 只改诞生代号那张表、不改 I-3.5 折叠表时这一形改红 I-3.5（攻方第 1.3 节）。记录宽 24 → 23 那一形见
/// [`a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before`]。
#[test]
fn a_lineage_container_whose_location_names_the_shared_inode_root_judges_both_device_orders_alike()
{
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-location-naming-the-shared-inode-root",
        )
        .image,
        TREE_KIND_LINEAGE,
        RecordWidthOfTheFirstContainer::AsRegistered,
        TREE_KIND_INODE,
        // I-3.1 照 C697（已分配统计在短环加大文件上误红） 的三判不红：记账多于遍历落在上沿之内（改三判之前红在这一边）。
        &["I-2.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
    );
}

/// 谱系撞 extent 树根、容器记录宽 24 → 23（攻方 `base=lineage walked=extent variant=bad`）。
/// 回退那一版的谱系树因此读不全：I-3.5 里 C 与 D 那一对要靠谱系那一半判的落点报部分没判，两种盘序判得一样。
#[test]
fn a_lineage_container_with_a_wrong_record_width_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
) {
    assert_both_device_orders_judge_alike_with_the_unit_birth_rule_holding_and_the_appearance_rule(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-record-width-location-naming-the-shared-extent-root",
        )
        .image,
        TREE_KIND_LINEAGE,
        RecordWidthOfTheFirstContainer::OneBelowTheRegistered {
            record_bytes: singlefs_format::LINEAGE_RECORD_BYTES,
        },
        TREE_KIND_EXTENT,
        &[
            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
        ],
        AppearanceRuleOnBothDeviceOrders::PartiallyJudgedWhereTheLineageTreeOfTheRollbackVersionDoesNotReadBack,
    );
}

/// 谱系撞 extent 树根、容器不改（攻方 `base=lineage walked=extent variant=good`）。
#[test]
fn a_lineage_container_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike()
{
    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-location-naming-the-shared-extent-root",
        )
        .image,
        TREE_KIND_LINEAGE,
        RecordWidthOfTheFirstContainer::AsRegistered,
        TREE_KIND_EXTENT,
        &["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
    );
}

/// 回退那一版（D）的 inode 树根：照抄回退目标 A 的那一个（两版经同一个树根共享叶容器），或原样另写一份到没写过的槽、D 的树表改指它
/// （两版各走自己的树根，叶容器由两个树根直接共享）。
#[derive(Clone, Copy, Debug)]
enum InodeRootOfTheRollbackPublish {
    SharedWithTheRollbackTarget,
    CopiedToASlotNothingWrote,
}

/// 两块盘上自己与前后各一槽都没写过的一个槽：单元区里写过的最后一槽再往后 16 槽。
fn a_slot_after_every_written_slot_of_the_unit_area(image: &MemoryPool) -> u64 {
    let last_written_slot = DEVICES
        .iter()
        .filter_map(|device| {
            image
                .devices
                .get(&DeviceIdentity(*device))
                .expect("盘")
                .written_sectors()
                .keys()
                .map(|sector| sector * 512 / SLOT)
                .filter(|slot| *slot >= UNIT_AREA_START_SLOT_OF_THE_POOL)
                .max()
        })
        .max()
        .expect("单元区写过");
    last_written_slot + 16
}

/// 分叉形（`raise_birth=true corrupt_loc0=true`）：
/// 回退到 A（txg 3）之后的镜像，D（txg 9）照抄 A 的 inode 树。inode 树根第一条条目指的叶容器 L 头里的诞生代号抬到 D 的 txg
/// （重封、沿引用链补到根：A、D 两版照旧引用同一个 L），再把 L 在位置条目 `garbled_location_index` 那块盘上的那一份写成别的字节
/// （不补校验和，那个槽上没有别的单元）。`inode_root` 定两版经同一个树根共享 L，还是各走自己的树根、直接共享 L。
fn image_after_the_rollback_whose_shared_leaf_container_is_born_after_the_target_with_one_copy_garbled(
    rolled: &ImageAfterTheRollbackToTheFirstRoot,
    garbled_location_index: usize,
    inode_root: InodeRootOfTheRollbackPublish,
) -> MemoryPool {
    let mut image = rolled.image.clone();
    let leaf = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
    let leaf_slot = common_pointer_head_corruption::the_slot_of(&leaf);
    let data_unit_bytes = common_pointer_head_corruption::data_unit_bytes();
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        leaf_slot,
        data_unit_bytes,
        |bytes| {
            let birth_txg = u64::from_le_bytes(bytes[73..81].try_into().expect("8 字节"));
            assert!(
                birth_txg <= rolled.rollback_target_txg,
                "L 的诞生代号 {birth_txg} 本来不晚于 A"
            );
            bytes[73..81].copy_from_slice(&rolled.rollback_publish_txg.to_le_bytes());
        },
    );
    match inode_root {
        InodeRootOfTheRollbackPublish::SharedWithTheRollbackTarget => {}
        InodeRootOfTheRollbackPublish::CopiedToASlotNothingWrote => {
            let inode_root_pointer =
                common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE)
                    .root;
            let inode_root_bytes = common_pointer_head_corruption::unit_bytes_at(
                &image,
                common_pointer_head_corruption::the_slot_of(&inode_root_pointer),
                common_pointer_head_corruption::node_bytes(),
            );
            let copy_slot = a_slot_after_every_written_slot_of_the_unit_area(&image);
            for device in DEVICES {
                write(&mut image, device, copy_slot * SLOT, &inode_root_bytes);
            }
            let pointer_bytes =
                common_pointer_head_corruption::node_pointer_bytes(&inode_root_pointer);
            common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
                &mut image,
                singlefs_core::address::SlotNumber(rolled.tree_table_slot_of_the_rollback_publish),
                common_pointer_head_corruption::node_bytes(),
                |table| {
                    let starts: Vec<usize> = (0..=table.len() - pointer_bytes.len())
                        .filter(|start| {
                            table[*start..*start + pointer_bytes.len()] == pointer_bytes[..]
                        })
                        .collect();
                    assert_eq!(starts.len(), 1, "D 的树表里 inode 树根指针恰好一处");
                    for location_index in 0..2 {
                        let slot_at = starts[0]
                            + POINTER_FIRST_LOCATION_OFFSET
                            + location_index * POINTER_LOCATION_BYTES
                            + LOCATION_DEVICE_BYTES;
                        table[slot_at..slot_at + LOCATION_SLOT_BYTES]
                            .copy_from_slice(&copy_slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
                    }
                },
            );
            // 中央映射是唯一入口、指针里的位置只是提示（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）：D 那一版的映射条目
            // 也改指那一份，D 才真走它自己的树根（只改树表里的指针，走读按映射认回原来那一个、两版照旧共享它）。
            point_the_newest_mapping_entries_of_a_slot_at(
                &mut image,
                common_pointer_head_corruption::the_slot_of(&inode_root_pointer).0,
                copy_slot,
            );
        }
    }
    write(
        &mut image,
        leaf.locations[garbled_location_index].device.0,
        leaf_slot.0 * SLOT,
        &vec![0xa5u8; data_unit_bytes],
    );
    image
}

/// 最新那条根的中央映射根（单层）里两条位置条目都在槽 `from_slot` 的那几条，两条位置条目的槽号改成 `to_slot`（整单元校验和不动），重封、
/// 新的整单元校验和沿引用链补到根槽。中央映射根里 key 宽 27：条目数 2 字节在 82 + 54、条目区从 86 + 54 + 29 起、每条 55
/// （key 27 + 位置条目 14 × 2，D19（块指针的结构与宽度预算） 已定项 10）。
fn point_the_newest_mapping_entries_of_a_slot_at(
    image: &mut MemoryPool,
    from_slot: u64,
    to_slot: u64,
) {
    const MAPPING_NODE_ENTRY_COUNT_OFFSET: usize = 82 + 54;
    const MAPPING_NODE_FIRST_ENTRY_OFFSET: usize = 86 + 54 + 29;
    const MAPPING_ENTRY_BYTES_IN_THE_NODE: usize = 55;
    const MAPPING_KEY_BYTES_IN_THE_ENTRY: usize = 27;
    let mapping_root = common_pointer_head_corruption::newest_root(image).mapping_root;
    let mut entries_changed = 0;
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        common_pointer_head_corruption::the_slot_of(&mapping_root),
        common_pointer_head_corruption::node_bytes(),
        |node| {
            let entry_count = usize::from(u16::from_le_bytes(
                node[MAPPING_NODE_ENTRY_COUNT_OFFSET..MAPPING_NODE_ENTRY_COUNT_OFFSET + 2]
                    .try_into()
                    .expect("2 字节"),
            ));
            for entry_index in 0..entry_count {
                let entry_start =
                    MAPPING_NODE_FIRST_ENTRY_OFFSET + entry_index * MAPPING_ENTRY_BYTES_IN_THE_NODE;
                let slot_at = |location_index: usize| {
                    entry_start
                        + MAPPING_KEY_BYTES_IN_THE_ENTRY
                        + location_index * POINTER_LOCATION_BYTES
                        + LOCATION_DEVICE_BYTES
                };
                let names_the_slot = (0..2).all(|location_index| {
                    node[slot_at(location_index)..slot_at(location_index) + LOCATION_SLOT_BYTES]
                        == from_slot.to_le_bytes()[..LOCATION_SLOT_BYTES]
                });
                if names_the_slot {
                    for location_index in 0..2 {
                        node[slot_at(location_index)
                            ..slot_at(location_index) + LOCATION_SLOT_BYTES]
                            .copy_from_slice(&to_slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
                    }
                    entries_changed += 1;
                }
            }
        },
    );
    assert_eq!(
        entries_changed, 1,
        "最新那一版的映射里恰好一条指着槽 {from_slot}"
    );
}

/// 分叉形的判法：L 坏的那一份在位置条目 0 / 1 两种盘序各判一次，红的都是 `expected_violations`，I-4.9（A 引用 L、L 读得出的那一份诞生代号 9 > 3）
/// 与 I-3.5（D 引用、C 不引用的 inode 树根那一格：子树里 L 读得出、诞生代号 9 大于树根指针的出生 txg）都红。坏的是位置条目 0 那一份时，
/// 诞生代号与 I-3.5 的折叠表都按读回的盘 1 那一份的落点记（攻方 allrbmap），GPU 判器同改（对拍流
/// `the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy`）。
fn assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
    tag: &str,
    inode_root: InodeRootOfTheRollbackPublish,
    expected_violations: &[&'static str],
) {
    let rolled = image_after_rolling_back_to_the_first_root(tag);
    for garbled_location_index in 0..2 {
        let image = image_after_the_rollback_whose_shared_leaf_container_is_born_after_the_target_with_one_copy_garbled(
            &rolled,
            garbled_location_index,
            inode_root,
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariant_set(&verdicts),
            expected_violations
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<&'static str>>(),
            "L 坏的是位置条目 {garbled_location_index} 那一份：两种盘序红的同一批（I-4.9、I-3.5 都在里面）：{verdicts:?}"
        );
    }
}

/// 分叉形、两版经同一个 inode 树根共享叶容器（攻方 `direct=false`）：A 走到树根时「走过」命中，按树根下面记下的落点并进来。
/// A 那一版的账里叶容器那一槽的记录未释放、那一槽却装着诞生代号 9 的另一版叶：A 的映射里那把 key（诞生代号 3）对不上它，I-3.15（N2 反查）跟着红。
#[test]
fn a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device(
) {
    assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
        "known-bad-shared-leaf-container-born-after-the-target-garbled-copy",
        InodeRootOfTheRollbackPublish::SharedWithTheRollbackTarget,
        &[
            "I-1.11",
            "I-1.2",
            "I-2.1",
            "I-3.10",
            UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
            REFERENCE_INTERVALS_ARE_EXACT,
            "I-4.2",
            "I-4.8",
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            "I-7.4",
        ],
    );
}

/// 分叉形、两版各走自己的 inode 树根、直接共享叶容器（攻方 `direct=true`）：A 走到叶容器时「走过」命中，按叶容器读回那一份的落点并进来。
/// 树根另写一份的那一槽没有分配记录：D 的树表与映射都改指它，A 照旧引用原来那一槽，两版合起来多引用一槽、I-3.1 跟着红；
/// 那一槽是走到的引用起点、最新根的分配记录里没有一条起点在它的记录，I-3.21（走到的引用起点是最新根计入的分配记录的起点）也红（点名盘 0 槽 50372）。
/// D 自己引用的槽数照旧（原来那一槽换成另写的那一槽、分配记录一条不动），I-3.11 按槽数比看不见这一换；「映射换位、分配记录不动」由判决
/// `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 的新判 N2（I-3.15 反查）判红，A 那一版叶容器那一槽装着另一版叶也由它判红。
#[test]
fn a_leaf_container_directly_shared_and_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device(
) {
    assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
        "known-bad-directly-shared-leaf-container-born-after-the-target-garbled-copy",
        InodeRootOfTheRollbackPublish::CopiedToASlotNothingWrote,
        &[
            "I-1.11",
            "I-1.2",
            "I-2.1",
            "I-3.1",
            "I-3.10",
            UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
            REFERENCE_INTERVALS_ARE_EXACT,
            "I-4.2",
            "I-4.8",
            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
            "I-7.4",
            WALKED_REFERENCE_STARTS_ARE_STARTS_OF_ALLOCATION_RECORDS_THE_NEWEST_ROOT_COUNTS,
        ],
    );
}

/// 类型段 0 那条条目孩子的一份副本：这几种都不是按自己那一类的长度自证过的（正推 `research/prompts/m3-batch32-code-r1-sonnet-output.md`
/// 第 2.2 节那张网格里两份都没自证过的那几种）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CopyThatDoesNotVerifyAtTheLengthOfItsClass {
    /// 头里是码 3、位置条目里的校验和只对得上前 16 KiB（`C16`）。
    ContainerCheckedAtSixteenKibibytes,
    /// 位置条目里的校验和翻几位（`BAD`）。
    ChecksumBitsFlipped,
    /// 位置条目里的盘不在池里（`UNREAD`）。
    DeviceOutsideThePool,
    /// 位置条目指着没写过的槽、校验和是 16 KiB 全零的 CRC（`ZERO`：读出全零，头里类标签 0 不登记）。
    SlotNothingWroteWithTheChecksumOfZeros,
}

const COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS:
    [CopyThatDoesNotVerifyAtTheLengthOfItsClass; 4] = [
    CopyThatDoesNotVerifyAtTheLengthOfItsClass::ContainerCheckedAtSixteenKibibytes,
    CopyThatDoesNotVerifyAtTheLengthOfItsClass::ChecksumBitsFlipped,
    CopyThatDoesNotVerifyAtTheLengthOfItsClass::DeviceOutsideThePool,
    CopyThatDoesNotVerifyAtTheLengthOfItsClass::SlotNothingWroteWithTheChecksumOfZeros,
];

/// 回退到 A 之后的镜像与三个单元的文件发完意图之后的镜像里两块盘上都没写过的一槽（造之前核过全零）。
const SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES: u64 = 190_000;

/// 种类 `kind` 那棵树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，孩子（码 3 容器）的两条位置条目照 `copies` 各改成一种
/// 自证不过的副本。
fn image_whose_type_zero_root_entry_names_a_child_with_two_copies_that_do_not_verify(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    copies: [CopyThatDoesNotVerifyAtTheLengthOfItsClass; 2],
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
    let container = common_pointer_head_corruption::unit_bytes_at(
        clean,
        common_pointer_head_corruption::the_slot_of(&child),
        common_pointer_head_corruption::data_unit_bytes(),
    );
    let checksum_of_the_container = crc32_castagnoli(&container);
    let checksum_of_the_first_sixteen_kibibytes =
        crc32_castagnoli(&container[..common_pointer_head_corruption::node_bytes()]);
    let checksum_of_sixteen_zero_kibibytes =
        crc32_castagnoli(&vec![0u8; common_pointer_head_corruption::node_bytes()]);
    let mut image = clean.clone();
    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
        change_the_entry_type(
            entry,
            registered_type,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        for (location_index, copy) in copies.iter().enumerate() {
            match copy {
                CopyThatDoesNotVerifyAtTheLengthOfItsClass::ContainerCheckedAtSixteenKibibytes => {
                    set_the_whole_unit_checksum_in_a_child_location(
                        entry,
                        location_index,
                        checksum_of_the_first_sixteen_kibibytes,
                    );
                }
                CopyThatDoesNotVerifyAtTheLengthOfItsClass::ChecksumBitsFlipped => {
                    set_the_whole_unit_checksum_in_a_child_location(
                        entry,
                        location_index,
                        checksum_of_the_container ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY,
                    );
                }
                CopyThatDoesNotVerifyAtTheLengthOfItsClass::DeviceOutsideThePool => {
                    set_the_device_in_a_child_location(
                        entry,
                        location_index,
                        DEVICE_IDENTITY_NOT_IN_THE_POOL,
                    );
                }
                CopyThatDoesNotVerifyAtTheLengthOfItsClass::SlotNothingWroteWithTheChecksumOfZeros => {
                    set_the_slot_in_a_child_location(
                        entry,
                        location_index,
                        SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES,
                    );
                    set_the_whole_unit_checksum_in_a_child_location(
                        entry,
                        location_index,
                        checksum_of_sixteen_zero_kibibytes,
                    );
                }
            }
        }
    });
    assert_eq!(level, 1, "根层级 1、直接管容器");
    image
}

/// F-ID-b 两份都没自证过（当读不出、这一小句不判）：
/// 两条位置条目各取一种自证不过的副本、两两组合，身份那一条
/// （`identity_invariant`）一格都不红；互换两条位置条目，红的同一批（盘不在池里那一种放在位置条目 0 时位置条目的盘序不升序，I-2.5 那一条不比）。
/// 身份那一条不取第一份按 16 KiB 对得上的头里的类标签（组合里有 `C16`（码 3）或 `ZERO`（码 0）的那 12 格不判身份红，正推第 2.2 节第 3 点）。
fn assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
    clean: &MemoryPool,
    kind: u16,
    registered_type: u16,
    identity_invariant: &'static str,
) {
    for device in DEVICES {
        assert!(
            read(
                clean,
                device,
                SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES * SLOT,
                common_pointer_head_corruption::data_unit_bytes(),
            )
            .iter()
            .all(|byte| *byte == 0),
            "槽 {SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES} 起 32 KiB 在盘 {device} 上没写过"
        );
    }
    let mut violated_of_each_pair = std::collections::BTreeMap::new();
    for first in COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS {
        for second in COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS {
            let image =
                image_whose_type_zero_root_entry_names_a_child_with_two_copies_that_do_not_verify(
                    clean,
                    kind,
                    registered_type,
                    [first, second],
                );
            let verdicts = check_pool_image(&image);
            let violated = violated_invariant_set(&verdicts);
            assert!(
                !violated.contains(identity_invariant),
                "位置条目 0 是 {first:?}、1 是 {second:?}：两份都没自证过，「子单元是码 2」这一小句不判，{identity_invariant} 不红：{verdicts:?}"
            );
            violated_of_each_pair.insert(
                (format!("{first:?}"), format!("{second:?}")),
                violated
                    .into_iter()
                    .filter(|invariant| *invariant != "I-2.5")
                    .collect::<std::collections::BTreeSet<&'static str>>(),
            );
        }
    }
    for ((first, second), violated) in &violated_of_each_pair {
        assert_eq!(
            Some(violated),
            violated_of_each_pair.get(&(second.clone(), first.clone())),
            "位置条目 0 是 {first}、1 是 {second}，与互换之后红的同一批（I-2.5 之外）"
        );
    }
}

#[test]
fn a_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_tombstone_entry_invariant_holding_on_either_device(
) {
    assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
            "known-bad-tombstone-entry-type-zero-two-copies-that-do-not-verify",
        ),
        TREE_KIND_TOMBSTONE,
        PACKED_RECORD_TYPE_TOMBSTONE,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

#[test]
fn a_lineage_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_lineage_entry_invariant_holding_on_either_device(
) {
    assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
        &image_after_rolling_back_to_the_first_root(
            "known-bad-lineage-entry-type-zero-two-copies-that-do-not-verify",
        )
        .image,
        TREE_KIND_LINEAGE,
        PACKED_RECORD_TYPE_LINEAGE,
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    );
}

/// 一条位置条目的盘上原样：盘 4 + 槽 6 + 整单元校验和 4，小端。
fn bytes_of_a_location_entry(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    let mut bytes = device.to_le_bytes().to_vec();
    bytes.extend_from_slice(&slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
    bytes.extend_from_slice(&checksum.to_le_bytes());
    bytes
}

/// G4 那一读另造的一形（正推变异 E1）：
/// 三个单元的文件发完意图之后，H-V 那一步（墓碑树根第一条条目指的容器在位置条目 `location_index` 那块盘上的
/// 一份换成墓碑树根的码 2 字节、那条位置条目的校验和换成它的 CRC），再把中央映射里那片容器那一条在同一块盘上的位置条目校验和换成那块盘上
/// 此刻 32 KiB 的 CRC（重封、补到根）：映射那一读对得上，I-2.1 只剩 F-W 往下走时按类型段那一类（码 3）的 32 KiB 读那一份那一读来红。
fn image_whose_container_copy_verifying_as_a_node_matches_its_central_mapping_entry(
    clean: &MemoryPool,
    location_index: usize,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, TREE_KIND_TOMBSTONE, 0);
    let container_slot = common_pointer_head_corruption::the_slot_of(&child);
    let checksum_of_the_container =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            clean,
            container_slot,
            common_pointer_head_corruption::data_unit_bytes(),
        ));
    let mut image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
        clean,
        TREE_KIND_TOMBSTONE,
        location_index,
        &leave_the_entry_as_it_is,
    );
    let device = child.locations[location_index].device.0;
    let checksum_of_the_copy_now_on_the_device = crc32_castagnoli(&read(
        &image,
        device,
        container_slot.0 * SLOT,
        common_pointer_head_corruption::data_unit_bytes(),
    ));
    let mapping_root = common_pointer_head_corruption::newest_root(&image).mapping_root;
    let before = bytes_of_a_location_entry(device, container_slot.0, checksum_of_the_container);
    let after = bytes_of_a_location_entry(
        device,
        container_slot.0,
        checksum_of_the_copy_now_on_the_device,
    );
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        common_pointer_head_corruption::the_slot_of(&mapping_root),
        common_pointer_head_corruption::node_bytes(),
        |bytes| {
            let starts: Vec<usize> = (0..=bytes.len() - before.len())
                .filter(|start| bytes[*start..*start + before.len()] == before[..])
                .collect();
            assert_eq!(
                starts.len(),
                1,
                "中央映射树（根兼叶）里那片容器那一条在盘 {device} 上的位置条目恰好一处"
            );
            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
        },
    );
    image
}

/// G4 那一读：两种盘序（码 2 那一份在位置条目 0 / 1）各判一次，I-2.1 红、第一处违例说的是墓碑树内部条目的子指针那一读（中央映射那一读对得上）。
/// 去掉 F-W 往下走时那一读的 I-2.1（正推变异 E1）这一条红。
#[test]
fn a_tombstone_container_copy_verifying_as_a_node_whose_mapping_entry_matches_it_reddens_the_checksum_invariant_on_either_device(
) {
    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
        "known-bad-tombstone-container-copy-verifying-as-a-node-matching-its-mapping-entry",
    );
    for location_index in 0..2 {
        let image =
            image_whose_container_copy_verifying_as_a_node_matches_its_central_mapping_entry(
                &clean,
                location_index,
            );
        let verdicts = check_pool_image(&image);
        assert!(
            matches!(
                verdict(&image, "I-2.1"),
                InvariantVerdict::Violated(detail) if detail.contains("墓碑树内部条目的子指针")
            ),
            "码 2 那一份在位置条目 {location_index}：F-W 往下走时按码 3 的 32 KiB 读回那一份对不上，I-2.1 红在这一读：{verdicts:?}"
        );
    }
}

/// F-KEY 数据单元那一处（正推变异 E2）：新池新建文件那一版，第一个文件的数据单元头里五元组的对象号加 1（重封、补到根，I-1.1 违例），
/// 再把 extent 树根（上段叶）里内联它的那条数据指针第 `location_index` 条位置条目的槽改成 extent 树根自己的槽（这一遍刚走过）、
/// 校验和写成它那 16 KiB 的 CRC 翻几位（这一份对不上）。另一条位置条目那一份就是带违例的数据单元。
fn image_whose_inline_data_pointer_names_the_walked_extent_root_beside_a_data_unit_with_another_object(
    clean: &MemoryPool,
    location_index: usize,
) -> MemoryPool {
    const DATA_UNIT_OBJECT_OFFSET: usize = 51;
    let mut image = clean.clone();
    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(data_pointer) =
        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
    else {
        panic!("新池新建文件那一版只有一个数据单元，上段叶第一条内联它的数据指针");
    };
    assert_eq!(
        data_pointer.locations[0].slot.0, DATA_UNIT,
        "新池新建文件那一版唯一的数据单元在登记过的那一槽"
    );
    // 码 1 的重封：`mutate_unit` 认三种码（`common_deletion_in_progress` 那一份只认码 2 与码 3）。
    mutate_unit(
        &mut image,
        DATA_UNIT,
        |bytes| {
            let object = get_u64(bytes, DATA_UNIT_OBJECT_OFFSET);
            set_u64(bytes, DATA_UNIT_OBJECT_OFFSET, object + 1);
        },
        true,
    );
    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(resealed_pointer) =
        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
    else {
        panic!("改了数据单元之后上段叶第一条照旧内联它的数据指针");
    };
    let extent_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
    let extent_root_slot = common_pointer_head_corruption::the_slot_of(&extent_root);
    let before = common_pointer_head_corruption::data_pointer_bytes(&resealed_pointer);
    let mut pointer_naming_the_extent_root = resealed_pointer;
    pointer_naming_the_extent_root.locations[location_index].slot = extent_root_slot;
    pointer_naming_the_extent_root.locations[location_index].unit_checksum =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            extent_root_slot,
            common_pointer_head_corruption::node_bytes(),
        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let after = common_pointer_head_corruption::data_pointer_bytes(&pointer_naming_the_extent_root);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        extent_root_slot,
        common_pointer_head_corruption::node_bytes(),
        |bytes| {
            let starts: Vec<usize> = (0..=bytes.len() - before.len())
                .filter(|start| bytes[*start..*start + before.len()] == before[..])
                .collect();
            assert_eq!(starts.len(), 1, "上段叶里那条数据指针恰好一处");
            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
        },
    );
    image
}

/// F-KEY 数据单元那一处：两种盘序（指着先走过的 extent 树根的是位置条目 0 / 1）各判一次，读回的那份数据单元照判，I-1.1 红在它的五元组、
/// 红的同一批。「走过」按第 0 条位置条目认的话，位置条目 0 那一形把读回来的数据单元当成走过（它的五元组一个字节都不判）。
#[test]
fn a_data_unit_with_another_object_whose_location_names_the_walked_extent_root_reddens_its_five_tuple_on_either_device(
) {
    let clean = build_pool("known-bad-data-unit-object-location-naming-the-walked-extent-root")
        .memory_pool();
    let mut violated_on_each_location = Vec::new();
    for location_index in 0..2 {
        let image =
            image_whose_inline_data_pointer_names_the_walked_extent_root_beside_a_data_unit_with_another_object(
                &clean,
                location_index,
            );
        let verdicts = check_pool_image(&image);
        assert!(
            matches!(
                verdict(&image, "I-1.1"),
                InvariantVerdict::Violated(detail) if detail.contains("数据单元五元组")
            ),
            "指着先走过的节点的是位置条目 {location_index}：读回来的那份数据单元照判，I-1.1 红在它的五元组：{verdicts:?}"
        );
        violated_on_each_location.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_location[0], violated_on_each_location[1],
        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
    );
}

/// F-KEY inode 内部孩子那一处（正推变异 E4）：新池新建文件那一版，inode 树根第一条条目类型段 2 → 0、身份三段清零；位置条目 `location_index`
/// 的槽改成 extent 树根的槽（这一遍先走过）、校验和写成它那 16 KiB 的 CRC 翻几位；另一条位置条目那块盘上叶容器的槽换成 inode 树根的码 2 字节、
/// 那条位置条目的校验和换成它的 CRC（按 16 KiB 自证得过）。读回的就是那份码 2，当 inode 树的内部节点读（层级 1，不是父层级减一）。
fn image_whose_type_zero_inode_entry_names_the_walked_extent_root_beside_a_copy_verifying_as_a_node(
    clean: &MemoryPool,
    location_index: usize,
) -> MemoryPool {
    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, TREE_KIND_INODE, 0);
    let inode_root_bytes = root_node_bytes_of_the_tree_of_kind(clean, TREE_KIND_INODE);
    let extent_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(clean, TREE_KIND_EXTENT).root;
    let extent_root_slot = common_pointer_head_corruption::the_slot_of(&extent_root);
    let wrong_checksum_of_the_extent_root =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            clean,
            extent_root_slot,
            common_pointer_head_corruption::node_bytes(),
        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let other_location_index = 1 - location_index;
    let mut image = clean.clone();
    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
        change_the_entry_type(
            entry,
            PACKED_RECORD_TYPE_INODE,
            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
        );
        zero_the_three_identity_fields(entry);
        set_the_slot_in_a_child_location(entry, location_index, extent_root_slot.0);
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            location_index,
            wrong_checksum_of_the_extent_root,
        );
        set_the_whole_unit_checksum_in_a_child_location(
            entry,
            other_location_index,
            crc32_castagnoli(&inode_root_bytes),
        );
    });
    assert_eq!(level, 1, "新池新建文件那一版的 inode 树根层级 1");
    write(
        &mut image,
        child.locations[other_location_index].device.0,
        common_pointer_head_corruption::the_slot_of(&child).0 * SLOT,
        &inode_root_bytes,
    );
    image
}

/// F-KEY inode 内部孩子那一处：两种盘序各判一次，读回的那份码 2 照判，I-1.1 红在「inode 树的内部节点」那一句，红的同一批。
#[test]
fn a_type_zero_inode_entry_whose_location_names_the_walked_extent_root_walks_the_copy_verifying_as_a_node_on_either_device(
) {
    let clean =
        build_pool("known-bad-inode-entry-type-zero-location-naming-the-walked-extent-root")
            .memory_pool();
    let mut violated_on_each_location = Vec::new();
    for location_index in 0..2 {
        let image =
            image_whose_type_zero_inode_entry_names_the_walked_extent_root_beside_a_copy_verifying_as_a_node(
                &clean,
                location_index,
            );
        let verdicts = check_pool_image(&image);
        assert!(
            matches!(
                verdict(&image, "I-1.1"),
                InvariantVerdict::Violated(detail) if detail.contains("inode 树的内部节点")
            ),
            "指着先走过的节点的是位置条目 {location_index}：读回来的那份码 2 照判，I-1.1 红在「inode 树的内部节点」：{verdicts:?}"
        );
        violated_on_each_location.push(violated_invariant_set(&verdicts));
    }
    assert_eq!(
        violated_on_each_location[0], violated_on_each_location[1],
        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
    );
}

/// `read_index_node` 那一处：
/// 新池新建文件那一版，inode 叶容器第一条记录的预留段写一个非零字节（两份、重封、补到根，I-9.7 违例）；再把树表里 inode 树那条条目的根指针
/// 第 `location_index` 条位置条目的槽改成树表单元自己的槽（这一遍先走过）、校验和写成它那 16 KiB 的 CRC 翻几位（这一份对不上）。
/// 另一条位置条目那一份就是 inode 树根，读者用的就是它。
fn image_whose_inode_root_pointer_names_the_walked_tree_table_above_a_leaf_with_a_non_zero_reserved_byte(
    clean: &MemoryPool,
    location_index: usize,
) -> MemoryPool {
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        INODE_LEAF,
        |bytes| bytes[INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_LAST_RESERVED_BYTE_OFFSET] = 1,
        true,
    );
    let inode_root =
        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let tree_table = common_pointer_head_corruption::newest_root(&image).tree_table;
    let tree_table_slot = common_pointer_head_corruption::the_slot_of(&tree_table);
    let before = common_pointer_head_corruption::node_pointer_bytes(&inode_root);
    let mut pointer_naming_the_tree_table = inode_root;
    pointer_naming_the_tree_table.locations[location_index].slot = tree_table_slot;
    pointer_naming_the_tree_table.locations[location_index].unit_checksum =
        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
            &image,
            tree_table_slot,
            common_pointer_head_corruption::node_bytes(),
        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let after = common_pointer_head_corruption::node_pointer_bytes(&pointer_naming_the_tree_table);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        tree_table_slot,
        common_pointer_head_corruption::node_bytes(),
        |bytes| {
            let starts: Vec<usize> = (0..=bytes.len() - before.len())
                .filter(|start| bytes[*start..*start + before.len()] == before[..])
                .collect();
            assert_eq!(starts.len(), 1, "树表里 inode 树根指针恰好一处");
            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
        },
    );
    image
}

/// `read_index_node` 那一处：两种盘序（指着先走过的树表单元的是位置条目 0 / 1）各判一次，读回的那份 inode 树根照走，它下面那片叶容器的
/// I-9.7 违例照红，红的同一批。「走过」按第 0 条位置条目认的话，位置条目 0 那一形把 inode 树根当成别的版本先走过（并进树表下面的落点、不往下走），
/// 叶容器一个字节都不判。
#[test]
fn an_inode_leaf_with_a_non_zero_reserved_byte_below_a_root_pointer_naming_the_walked_tree_table_reddens_the_reserved_bytes_invariant_on_either_device(
) {
    let clean = build_pool(
        "known-bad-inode-leaf-reserved-byte-below-a-root-pointer-naming-the-walked-tree-table",
    )
    .memory_pool();
    let mut violated_on_each_location = Vec::new();
    for location_index in 0..2 {
        let image =
            image_whose_inode_root_pointer_names_the_walked_tree_table_above_a_leaf_with_a_non_zero_reserved_byte(
                &clean,
                location_index,
            );
        let verdicts = check_pool_image(&image);
        let violated = violated_invariant_set(&verdicts);
        assert!(
            violated.contains("I-9.7"),
            "指着先走过的树表单元的是位置条目 {location_index}：读回的那份 inode 树根照走，叶容器里的 I-9.7 违例照红：{verdicts:?}"
        );
        violated_on_each_location.push(violated);
    }
    assert_eq!(
        violated_on_each_location[0], violated_on_each_location[1],
        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
    );
}

// ─── 删除线：I-3.5 按全部版本走完之后的折叠表取单元（H-S1-fold）、两份都读不出而第 0 条落点早被别的版本走过 ───

/// 一版的树表单元的槽（第一版两盘同槽）。
fn tree_table_slot_of_the_version(
    version: &singlefs_core::transaction::TransactionOutput,
) -> singlefs_core::address::SlotNumber {
    common_pointer_head_corruption::the_slot_of(&version.root.tree_table)
}

/// 那一版的树表单元里种类 `kind` 那条条目的根指针在单元里的起点（整条 86 字节逐字找，恰好一处）；那条根指针全零时 `None`。
fn root_pointer_start_in_the_tree_table_of(
    image: &MemoryPool,
    version: &singlefs_core::transaction::TransactionOutput,
    kind: u16,
) -> Option<usize> {
    let tree_table = common_pointer_head_corruption::unit_bytes_at(
        image,
        tree_table_slot_of_the_version(version),
        common_pointer_head_corruption::node_bytes(),
    );
    let root_pointer = singlefs_core::unit::parse_index_node(&tree_table)
        .expect("树表解得开")
        .entries
        .iter()
        .filter_map(|entry| singlefs_core::records::TreeTableEntry::parse(entry))
        .find(|entry| entry.kind == kind)
        .expect("树表里有这种树")
        .root;
    let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&root_pointer);
    if pointer_bytes.iter().all(|byte| *byte == 0) {
        return None;
    }
    let starts: Vec<usize> = (0..=tree_table.len() - pointer_bytes.len())
        .filter(|start| tree_table[*start..*start + pointer_bytes.len()] == pointer_bytes[..])
        .collect();
    let [start] = starts.as_slice() else {
        panic!("种类 {kind} 的根指针在树表里恰好一处：{starts:?}")
    };
    Some(*start)
}

/// 指针里第 0 条位置条目的起点、每条位置条目的宽、条目里整单元校验和的偏移（D19（块指针的结构与宽度预算） 已定项 7 的字段表）。
pub(super) const NODE_POINTER_FIRST_LOCATION_OFFSET: usize = 50;

pub(super) const NODE_POINTER_LOCATION_BYTES: usize = 14;

pub(super) const NODE_POINTER_LOCATION_CHECKSUM_OFFSET: usize = 10;

/// 那一版树表里种类 `kind` 那条根指针第 `location_indexes` 这几条位置条目的整单元校验和各异或几位（指着的单元本身不动），重封那张树表、
/// 沿引用链补到根槽：这一版读那棵树的根时那几份对不上（I-2.1 红）、读回别的那一份。
fn garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
    image: &mut MemoryPool,
    version: &singlefs_core::transaction::TransactionOutput,
    kind: u16,
    location_indexes: &[usize],
) {
    let start = root_pointer_start_in_the_tree_table_of(image, version, kind)
        .expect("这一版那棵树的根指针不是全零");
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        image,
        tree_table_slot_of_the_version(version),
        common_pointer_head_corruption::node_bytes(),
        |tree_table| {
            for location_index in location_indexes {
                let checksum = start
                    + NODE_POINTER_FIRST_LOCATION_OFFSET
                    + location_index * NODE_POINTER_LOCATION_BYTES
                    + NODE_POINTER_LOCATION_CHECKSUM_OFFSET;
                for byte in &mut tree_table[checksum..checksum + 4] {
                    *byte ^= 0x5a;
                }
            }
        },
    );
}

/// 三个单元的文件之后发意图、删第一批：交回镜像与顺序写（txg 4）、意图（txg 5，extent 树照抄 txg 4 的、树表另写一份）两版。
fn image_with_the_written_and_the_intent_versions_after_the_first_batch(
    tag: &str,
) -> (
    MemoryPool,
    singlefs_core::transaction::TransactionOutput,
    singlefs_core::transaction::TransactionOutput,
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(tag);
    let written = pool.output.clone();
    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
    common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
    (pool.memory_pool(), written, intent)
}

fn verdict_of_the_invariant(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .map(|(_, verdict)| verdict)
        .expect("清单里有")
}

/// H-S1-fold 的 A 臂（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S1 行与第三节第 1 条，攻方
/// `research/prompts/m3-batch33s-code-r1-opus-output.md` 第一节 1.2）：意图那一版（txg 5，较新、根环次序在 txg 4 之后走）树表里 extent 树根指针
/// 第 `location_index` 条位置条目的校验和坏，读回另一份。池级 checker 判 I-3.5 时按全部版本走完之后的折叠表取每版的单元（攻方 F1）：
/// txg 4 那一版先走、经第 0 条读回的那个落点，在意图那一版读回第 1 条那一份、把它改记成自己之后同算一个单元，意图那一版不算新引用它。
/// 两种盘序（坏的是第 0 / 1 条）I-3.5 都成立、I-2.1 都红（不是每版走完当场折：那样坏的是第 0 条时多红 I-3.5，与 GPU 走读内核的取法分叉）。
#[test]
fn an_intent_version_reading_the_shared_extent_root_from_either_copy_leaves_the_appearance_rule_holding(
) {
    let (clean, _written, intent) =
        image_with_the_written_and_the_intent_versions_after_the_first_batch(
            "known-bad-i35-intent-reads-the-shared-extent-root-from-either-copy",
        );
    for location_index in 0..2 {
        let mut image = clean.clone();
        garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
            &mut image,
            &intent,
            singlefs_core::records::TREE_KIND_EXTENT,
            &[location_index],
        );
        let verdicts = check_pool_image(&image);
        let set = violated_invariant_set(&verdicts);
        assert!(
            set.contains("I-2.1"),
            "坏的是第 {location_index} 条：那一份照判 I-2.1 红：{verdicts:?}"
        );
        assert_eq!(
            verdict_of_the_invariant(&image, REFERENCE_INTERVALS_ARE_EXACT),
            InvariantVerdict::Holds,
            "坏的是第 {location_index} 条：I-3.5 按全部版本走完之后的折叠表取单元，成立：{verdicts:?}"
        );
    }
}

/// 攻方 S1 扫描（同上第一节 1.3）的一种历史：`versions` 是按发布次序的各版（新池新建文件那一版也算），每一对相邻、树表各写一份的版本里，
/// 较新那一版树表里根指针与较旧那一版同种类的根指针逐字相同的每一棵树，各造一份坏镜像：较新那一版那条根指针第 0 条位置条目的校验和坏。
/// 每一份 I-3.5 都不红。交回造了几份。
fn assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
    clean: &MemoryPool,
    versions: &[singlefs_core::transaction::TransactionOutput],
) -> usize {
    let mut images_judged = 0;
    for pair in versions.windows(2) {
        let (older, newer) = (&pair[0], &pair[1]);
        if tree_table_slot_of_the_version(older) == tree_table_slot_of_the_version(newer) {
            continue;
        }
        for entry in &newer.tree_table_entries {
            if root_pointer_start_in_the_tree_table_of(clean, newer, entry.kind).is_none() {
                continue;
            }
            let pointer_bytes = common_pointer_head_corruption::node_pointer_bytes(&entry.root);
            let shared_with_the_older_version =
                older.tree_table_entries.iter().any(|older_entry| {
                    older_entry.kind == entry.kind
                        && common_pointer_head_corruption::node_pointer_bytes(&older_entry.root)
                            == pointer_bytes
                });
            if !shared_with_the_older_version {
                continue;
            }
            let mut image = clean.clone();
            garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
                &mut image,
                newer,
                entry.kind,
                &[0],
            );
            let verdict = verdict_of_the_invariant(&image, REFERENCE_INTERVALS_ARE_EXACT);
            assert!(
                !matches!(verdict, InvariantVerdict::Violated(_)),
                "txg {} → {}、种类 {} 的根指针第 0 条校验和坏：I-3.5 不红（{verdict:?}）",
                older.root.checkpoint_txg.0,
                newer.root.checkpoint_txg.0,
                entry.kind
            );
            images_judged += 1;
        }
    }
    images_judged
}

/// 攻方 S1 扫描 `write3-intent-batch`：三个单元的文件、意图、第一批（3 份坏镜像：txg 4 → 5 的 extent 树，5 → 6 的墓碑树与 `deleted_inodes`）。
#[test]
fn garbling_each_shared_root_pointer_of_a_newer_version_through_an_intent_and_a_batch_leaves_the_appearance_rule_unviolated(
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
        "known-bad-i35-scan-write-intent-batch",
    );
    let mut versions = vec![pool.output.clone()];
    versions.push(common_deletion_in_progress::publish_the_intent(&mut pool));
    versions.push(common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool));
    assert_eq!(
        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
            &pool.memory_pool(),
            &versions,
        ),
        3,
        "攻方这一种历史扫的格数"
    );
}

/// 攻方 S1 扫描 `write3-unlink-to-the-end`：三个单元的文件、意图、按批大小 1 删到队列空（6 份坏镜像）。
#[test]
fn garbling_each_shared_root_pointer_of_a_newer_version_through_an_unlink_to_the_end_leaves_the_appearance_rule_unviolated(
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
        "known-bad-i35-scan-unlink-to-the-end",
    );
    let mut versions = vec![pool.output.clone()];
    versions.push(common_deletion_in_progress::publish_the_intent(&mut pool));
    while !pool.output.deleted_inodes.is_empty() {
        versions
            .push(common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool));
    }
    assert_eq!(
        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
            &pool.memory_pool(),
            &versions,
        ),
        6,
        "攻方这一种历史扫的格数"
    );
}

/// 攻方 S1 扫描 `write3-new-inodes-x3`：三个单元的文件之后建三次空 inode（3 份坏镜像，都是 extent 树）。
#[test]
fn garbling_each_shared_root_pointer_of_a_newer_version_through_three_publishes_of_new_inodes_leaves_the_appearance_rule_unviolated(
) {
    let mut pool =
        common_deletion_in_progress::pool_with_a_three_unit_file("known-bad-i35-scan-new-inodes");
    let mut versions = vec![pool.output.clone()];
    for _publish in 0..3 {
        versions.push(
            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1),
        );
    }
    assert_eq!(
        assert_garbling_the_first_location_of_each_shared_root_pointer_of_the_newer_version_leaves_the_appearance_rule_unviolated(
            &pool.memory_pool(),
            &versions,
        ),
        3,
        "攻方这一种历史扫的格数"
    );
}

/// 正推 O1-1（`research/prompts/m3-batch33s-code-r1-sonnet-output.md` K1-S1 末段两处观察第一处；判决 `research/prompts/m3-batch33s-code-r1-main-verification.md`
/// 第三节第 1 条「正推第 4 条那一形补坏镜像」）：顺序写那一版（txg 4，根环次序在最新的意图那一版之后走）树表里 extent 树根指针两条位置条目的校验和都坏，
/// 第 0 条落点上那个单元（两版共享的 extent 树根）已被意图那一版先走过。「走过」按读回那一份的落点认：两份都读不出就没有读回的落点，
/// 记走读失败、两份照判 I-2.1，不当成「别的版本先走过」（不是读之前先按第 0 条落点查走过表：那样这一形会判成先走过、并进前一版、两份都不判、不记走读失败）。
#[test]
fn a_root_pointer_whose_two_copies_do_not_read_back_and_whose_first_location_names_a_unit_walked_before_records_a_walk_failure(
) {
    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
        "known-bad-two-copies-unreadable-first-location-walked-before",
    );
    let written = pool.output.clone();
    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
    let shared_extent_root = |version: &singlefs_core::transaction::TransactionOutput| {
        version
            .tree_table_entries
            .iter()
            .find(|entry| entry.kind == singlefs_core::records::TREE_KIND_EXTENT)
            .expect("树表里有 extent 树")
            .root
    };
    assert_eq!(
        common_pointer_head_corruption::node_pointer_bytes(&shared_extent_root(&written)),
        common_pointer_head_corruption::node_pointer_bytes(&shared_extent_root(&intent)),
        "意图那一版 extent 树照抄顺序写那一版的"
    );
    let mut image = pool.memory_pool();
    garble_location_checksums_of_a_root_pointer_in_the_tree_table_of(
        &mut image,
        &written,
        singlefs_core::records::TREE_KIND_EXTENT,
        &[0, 1],
    );
    let verdicts = check_pool_image(&image);
    let set = violated_invariant_set(&verdicts);
    assert!(
        set.contains("I-2.1") && set.contains("I-4.8"),
        "两份照判 I-2.1 红，走读失败记下、遍历读不出红 I-4.8：{verdicts:?}"
    );
}
