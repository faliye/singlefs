//! 池级 checker 按 I-1.10（码 2 条目宽等于字段表宽） 判 deadlist 树的条目宽：叶条目要等于 43（头的树 ID 8 + 纪元 8 + 中央映射 key 27，
//! value 空），内部节点条目要等于 129（key 43 + 子指针 86）。实现读 deadlist 时拒宽于字段表的条目（叶在 `crates/singlefs-core/src/records.rs` 判条目长不等于 43），改之前池级 checker
//! 只守「不窄于」，宽 44 的 deadlist 镜像上可写挂载拒、checker 不报（判决 `research/prompts/m3-close-w13-code-r1-main-verification.md`
//! 第二节 K5 那一行与「去向」表「K5 deadlist 跨 oracle」那一行）。
//!
//! 镜像都从同一个场景起：新池新建文件、建快照、快照下覆盖写，换下的单元进 deadlist，deadlist 树根兼叶、装着条目（造法照
//! `tree_readers_refuse_extent_inode_allocation_and_deadlist_entries_wider_than_the_field_table.rs` 的
//! `a_deadlist_leaf_whose_entries_are_wider_than_the_field_table_is_refused_by_the_writable_mount_before_any_write`）。
//! 坏镜像只改 deadlist 树根那一个单元，按原来的身份重建、重封，新的整单元校验和沿引用链补到根槽。
//! 宽度都按字段表写死在用例里，不从 checker 或实现里取：取了的话，两边把宽挪了这几条也跟着走。
//! 判别力：`crates/mutations.tsv` 里以「K5 deadlist 条目宽」开头的几行（把判等于改回只守不窄于、字段表算错），各让这里点名的用例红。

mod common;
mod common_pointer_head_corruption;

use common::{
    build_pool, parameters, publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
};
use common_pointer_head_corruption::{
    node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::InstanceGeneration;
use singlefs_core::code_two_tree::build_internal_entry;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::TREE_KIND_DEADLIST;
use singlefs_core::transaction::{publish_create_snapshot, PoolWriter};
use singlefs_core::unit::{build_index_node, parse_index_node, IndexNodeHeader};
use singlefs_harness::memory_pool::MemoryPool;

/// deadlist 叶条目字段表的宽：头的树 ID 8 + 纪元 8 + 中央映射 key 27，value 空（D5（快照 / 空间记账机制） 已定项 12）。
const DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES: usize = 43;
/// deadlist 树内部节点条目字段表的宽：key 43 + 子指针 86（D8（核心索引结构） 已定项 11）。
const DEADLIST_INTERNAL_ENTRY_FIELD_TABLE_BYTES: usize = 129;
/// 条目宽要等于字段表宽的那一条不变量。
const ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE: &str = "I-1.10";
/// 建快照与快照下覆盖写都在实例 1 上（`build_pool` 取的号）。
const INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE: InstanceGeneration = InstanceGeneration(1);

/// 新池新建文件之后建一个快照、再在快照下覆盖写一次：换下的数据单元、extent 根、inode 叶容器与根都生在快照之前，进 deadlist。
fn image_with_a_deadlist_after_an_overwrite_under_a_snapshot(tag: &str) -> MemoryPool {
    let mut pool: BuiltPool = build_pool(tag);
    let publish_parameters = parameters();
    let before_the_snapshot = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    pool.output = publish_create_snapshot(
        &mut writer,
        &mut pool.allocator,
        &before_the_snapshot,
        INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE,
    )
    .expect("建快照");
    let before_the_overwrite = pool.output.clone();
    let overwrite_content: Vec<u8> = (0..2700_u32)
        .map(|index| u8::try_from((index * 13 + 11) % 239).expect("小于 256"))
        .collect();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &before_the_overwrite,
        &overwrite_content,
        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
        INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE,
    )
    .expect("快照下覆盖写");
    pool.memory_pool()
}

/// deadlist 树根那一个节点（解开的头与条目）。前提：它根兼叶、装着条目、写者写的条目宽等于字段表宽。
fn deadlist_root_leaf_written_by_the_writer(image: &MemoryPool) -> IndexNodeHeader {
    let deadlist_root = tree_table_entry_of_kind(image, TREE_KIND_DEADLIST).root;
    let node = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&deadlist_root),
        node_bytes(),
    ))
    .expect("deadlist 树根解得开");
    assert!(
        node.level == 0
            && !node.entries.is_empty()
            && node.entry_width == DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES,
        "前提：deadlist 树根兼叶、装着条目，写者写的条目宽等于字段表宽（层级 {}、{} 条、条目宽 {}）",
        node.level,
        node.entries.len(),
        node.entry_width
    );
    node
}

/// deadlist 树根那一个单元换成 `replacement`（调用方按原来的身份建好），新的整单元校验和沿引用链补到根槽。
fn replace_the_deadlist_root(image: &mut MemoryPool, replacement: &[u8]) {
    let deadlist_root = tree_table_entry_of_kind(image, TREE_KIND_DEADLIST).root;
    replace_the_unit_and_propagate_its_checksum(image, the_slot_of(&deadlist_root), replacement);
}

/// 按 `leaf` 的身份（树、key 区间、出生代、实例、出生序号）建一个码 2 节点：层级、条目宽、条目由调用方给。
fn node_with_the_identity_of(
    leaf: &IndexNodeHeader,
    level: u8,
    entry_width: usize,
    entries: &[Vec<u8>],
) -> Vec<u8> {
    build_index_node(
        leaf.tree,
        level,
        leaf.key_width,
        &leaf.smallest_key,
        &leaf.largest_key,
        leaf.birth_txg,
        &parameters().filesystem_identifier,
        leaf.instance,
        leaf.birth_sequence,
        u16::try_from(entry_width).expect("条目宽装得进两字节"),
        entries,
    )
}

/// deadlist 树根换成层级 1 的内部节点：一条条目，分隔 key 是原来那片叶的最小 key、子指针全零，条目末尾补零补到 `entry_width`。
/// 子指针全零：宽于字段表的那一形，按 I-1.10 判过就不往下走、碰不到它；等于字段表的那一形往下走，按「父条目里的子指针全零」记走读失败，
/// 不碰 I-1.10。
fn image_whose_deadlist_root_is_an_internal_node_with_entries_of_width(
    tag: &str,
    entry_width: usize,
) -> MemoryPool {
    let mut image = image_with_a_deadlist_after_an_overwrite_under_a_snapshot(tag);
    let leaf = deadlist_root_leaf_written_by_the_writer(&image);
    let mut entry = build_internal_entry(&leaf.smallest_key, NodePointer::empty_root());
    assert_eq!(
        entry.len(),
        DEADLIST_INTERNAL_ENTRY_FIELD_TABLE_BYTES,
        "前提：实现建的 deadlist 内部条目宽等于字段表宽"
    );
    entry.resize(entry_width, 0);
    replace_the_deadlist_root(
        &mut image,
        &node_with_the_identity_of(&leaf, 1, entry_width, &[entry]),
    );
    image
}

/// 池级判决里判红的那几条（按清单次序）。
fn violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 池级判决里 I-1.10 那一格。
fn entry_width_verdict(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE)
        .unwrap_or_else(|| panic!("{ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE} 在池级 checker 的清单里"))
        .1
}

/// 场景：快照下覆盖写之后那一版，deadlist 树根（根兼叶）的条目宽 43 → 44，每条末尾多一个 0，别处不动。
/// 预期：改之前池级 checker 一条都不红（合法镜像上 I-1.10 不新报）；改之后只红 I-1.10，说明点名 deadlist 树、条目宽 44 与字段表宽 43。
#[test]
fn a_deadlist_leaf_whose_entries_are_wider_than_the_field_table_reddens_the_entry_width_invariant()
{
    let mut image =
        image_with_a_deadlist_after_an_overwrite_under_a_snapshot("checker-deadlist-leaf-wider");
    assert_eq!(
        violated_invariants(&image),
        Vec::<&str>::new(),
        "前提：快照下覆盖写之后的合法镜像上池级 checker 一条都不红"
    );
    let leaf = deadlist_root_leaf_written_by_the_writer(&image);
    let wider_entries: Vec<Vec<u8>> = leaf
        .entries
        .iter()
        .map(|entry| {
            let mut wider = entry.clone();
            wider.push(0);
            wider
        })
        .collect();
    replace_the_deadlist_root(
        &mut image,
        &node_with_the_identity_of(
            &leaf,
            0,
            DEADLIST_LEAF_ENTRY_FIELD_TABLE_BYTES + 1,
            &wider_entries,
        ),
    );
    assert_eq!(
        violated_invariants(&image),
        vec![ENTRY_WIDTH_EQUALS_THE_FIELD_TABLE],
        "deadlist 叶条目宽于字段表：镜像上只有条目宽这一处说话，只红 I-1.10"
    );
    match entry_width_verdict(&image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("deadlist 树")
                && detail.contains("条目宽 44 不等于这一层的条目字段表宽度 43"),
            "I-1.10 的说明点名 deadlist 树、条目宽 44 与字段表宽 43：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("deadlist 叶条目宽 44 要报 I-1.10，得到 {other:?}")
        }
    }
}

/// 场景：快照下覆盖写之后那一版，deadlist 树根换成层级 1 的内部节点，条目宽 130（字段表 129 多一个字节）。
/// 预期：池级 checker 报 I-1.10，说明点名 deadlist 树、条目宽 130 与字段表宽 129。
#[test]
fn a_deadlist_internal_node_whose_entries_are_wider_than_the_field_table_reddens_the_entry_width_invariant(
) {
    let image = image_whose_deadlist_root_is_an_internal_node_with_entries_of_width(
        "checker-deadlist-internal-node-wider",
        DEADLIST_INTERNAL_ENTRY_FIELD_TABLE_BYTES + 1,
    );
    match entry_width_verdict(&image) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains("deadlist 树")
                && detail.contains("条目宽 130 不等于这一层的条目字段表宽度 129"),
            "I-1.10 的说明点名 deadlist 树、条目宽 130 与字段表宽 129：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("deadlist 内部节点条目宽 130 要报 I-1.10，得到 {other:?}")
        }
    }
}

/// 场景：同上，内部节点的条目宽恰是字段表宽 129（子指针全零，往下走时记走读失败）。
/// 预期：I-1.10 不红——判的是「等于」内部节点那一张字段表，不是叶那一张，也不是「不窄于」。
#[test]
fn a_deadlist_internal_node_whose_entries_are_as_wide_as_the_field_table_does_not_redden_the_entry_width_invariant(
) {
    let image = image_whose_deadlist_root_is_an_internal_node_with_entries_of_width(
        "checker-deadlist-internal-node-as-wide",
        DEADLIST_INTERNAL_ENTRY_FIELD_TABLE_BYTES,
    );
    let verdict = entry_width_verdict(&image);
    assert!(
        !matches!(verdict, InvariantVerdict::Violated(_)),
        "deadlist 内部节点条目宽 129 等于字段表宽，I-1.10 不红，得到 {verdict:?}"
    );
}
