//! m3-close-w13-code-r1 攻方腿的模型（K4、K5）：C867 改过的六处读者对「条目宽宽于字段表」的节点一律拒，
//! 池级 checker（`singlefs_checker::walk::check_pool_image`）对同一张坏镜像判什么。
//! 坏镜像的造法与 `tree_readers_refuse_extent_inode_allocation_and_deadlist_entries_wider_than_the_field_table.rs` 相同
//! （在发布路径写出的一版上把一个节点的每条条目末尾补一个 0、条目宽加一，重封、校验和沿引用链补到根槽），
//! 这里只多做一步：改之前、改之后各跑一次池级 checker，打出改之后新出现的非「成立」判定，再打出实现这一侧的结局。
//! 只在攻方腿的仓副本里跑（harness 档），不进库。

mod common;
mod common_pointer_head_corruption;

use common::{
    build_pool, parameters, publish_overwrite_in_process, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
};
use common_pointer_head_corruption::{
    first_entry_of_the_node_at, first_upper_leaf_entry, image_after_a_sequential_write_of,
    node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
    tree_table_entry_of_kind, unit_bytes_at, writable_devices_of,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::InstanceGeneration;
use singlefs_core::bytes::ByteReader;
use singlefs_core::extent_tree::ExtentUpperLeafTarget;
use singlefs_core::mount::mount_writable;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_INODE};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{publish_create_snapshot, PoolWriter};
use singlefs_core::unit::{build_index_node, parse_index_node};
use singlefs_harness::memory_pool::MemoryPool;

const ALLOCATION_RECORD_KEY_BYTES: usize = 10;
const INSTANCE_OF_THE_SNAPSHOT_AND_THE_OVERWRITE: InstanceGeneration = InstanceGeneration(1);

fn widen_every_entry_of_the_node_by_one_byte(image: &mut MemoryPool, pointer: &NodePointer) {
    let node = parse_index_node(&unit_bytes_at(image, the_slot_of(pointer), node_bytes()))
        .expect("要改的节点解得开");
    let wider_entries: Vec<Vec<u8>> = node
        .entries
        .iter()
        .map(|entry| {
            let mut wider = entry.clone();
            wider.push(0);
            wider
        })
        .collect();
    let wider_node = build_index_node(
        node.tree,
        node.level,
        node.key_width,
        &node.smallest_key,
        &node.largest_key,
        node.birth_txg,
        &parameters().filesystem_identifier,
        node.instance,
        node.birth_sequence,
        u16::try_from(node.entry_width + 1).expect("两字节"),
        &wider_entries,
    );
    replace_the_unit_and_propagate_its_checksum(image, the_slot_of(pointer), &wider_node);
}

fn level_and_entry_width(image: &MemoryPool, pointer: &NodePointer) -> (u8, usize) {
    let node = parse_index_node(&unit_bytes_at(image, the_slot_of(pointer), node_bytes()))
        .expect("节点解得开");
    (node.level, node.entry_width)
}

fn allocation_leaf_down_the_first_entries(image: &MemoryPool) -> NodePointer {
    let mut pointer = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION).root;
    let (mut level, _) = level_and_entry_width(image, &pointer);
    while level > 0 {
        pointer = NodePointer::read_from(&mut ByteReader::at(
            &first_entry_of_the_node_at(image, &pointer),
            ALLOCATION_RECORD_KEY_BYTES,
        ));
        level = level_and_entry_width(image, &pointer).0;
    }
    pointer
}

fn image_with_a_deadlist() -> MemoryPool {
    let mut pool: BuiltPool = build_pool("attack-w13-deadlist");
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

/// 非「成立」的判定（不变量号、种类、说明前 140 字）。
fn not_holding(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Holds => None,
            InvariantVerdict::Violated(detail) => Some(format!(
                "{invariant}:违例:{}",
                detail.chars().take(140).collect::<String>()
            )),
            InvariantVerdict::NotApplicable(why) => Some(format!("{invariant}:不适用:{why}")),
            InvariantVerdict::PartiallyJudged(detail) => Some(format!(
                "{invariant}:部分没判:{}",
                detail.chars().take(140).collect::<String>()
            )),
        })
        .collect()
}

fn implementation_outcomes(image: &MemoryPool) -> (String, String) {
    let cold = match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::Failed { failure, .. } => format!("恢复拒:{failure:?}"),
        RecoveryOutcome::FileRead { root, .. } => format!("恢复读出文件:{root:?}"),
        RecoveryOutcome::NoFile { root } => format!("恢复无文件:{root:?}"),
    };
    let writable = match mount_writable(&parameters(), &mut writable_devices_of(image)) {
        Ok(mounted) => format!("可写挂载挂上:实例{:?}", mounted.output.instance),
        Err(refusal) => format!("可写挂载拒:{refusal:?}"),
    };
    (cold, writable)
}

/// 改之前、改之后各判一次；交回改之后新出现的非「成立」判定。
fn report(case: &str, before: &MemoryPool, after: &MemoryPool) -> Vec<String> {
    let before_lines = not_holding(before);
    let after_lines = not_holding(after);
    let new_lines: Vec<String> = after_lines
        .iter()
        .filter(|line| !before_lines.contains(line))
        .cloned()
        .collect();
    let (cold_before, writable_before) = implementation_outcomes(before);
    let (cold_after, writable_after) = implementation_outcomes(after);
    println!("name=case-{case} checker_new_not_holding={} lines={new_lines:?}", new_lines.len());
    println!("name=case-{case} implementation_before cold={cold_before} writable={writable_before}");
    println!("name=case-{case} implementation_after cold={cold_after} writable={writable_after}");
    new_lines
}

fn mentions_one_ten(lines: &[String]) -> bool {
    lines.iter().any(|line| line.starts_with("I-1.10:违例"))
}

#[test]
fn checker_and_the_six_readers_on_entries_wider_than_the_field_table() {
    // extent 树内部条目 110 → 111（145 个数据单元，下段根在第 1 层）。
    let before = image_after_a_sequential_write_of("attack-w13-extent-internal", 145);
    let mut after = before.clone();
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&after).target
    else {
        panic!("标签 1");
    };
    assert_eq!(level_and_entry_width(&after, &lower_root), (1, 110));
    widen_every_entry_of_the_node_by_one_byte(&mut after, &lower_root);
    let extent_internal = report("extent-internal-110", &before, &after);

    // extent 叶记录 112 → 113（两个数据单元，下段根兼叶）。
    let before = image_after_a_sequential_write_of("attack-w13-extent-leaf", 2);
    let mut after = before.clone();
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&after).target
    else {
        panic!("标签 1");
    };
    assert_eq!(level_and_entry_width(&after, &lower_root), (0, 112));
    widen_every_entry_of_the_node_by_one_byte(&mut after, &lower_root);
    let extent_leaf = report("extent-leaf-112", &before, &after);

    // inode 树内部条目 120 → 121。
    let before = build_pool("attack-w13-inode-internal").memory_pool();
    let mut after = before.clone();
    let inode_root = tree_table_entry_of_kind(&after, TREE_KIND_INODE).root;
    assert_eq!(level_and_entry_width(&after, &inode_root), (1, 120));
    widen_every_entry_of_the_node_by_one_byte(&mut after, &inode_root);
    let inode_internal = report("inode-internal-120", &before, &after);

    // 分配记录 20 → 21。
    let before = build_pool("attack-w13-allocation-leaf").memory_pool();
    let mut after = before.clone();
    let leaf = allocation_leaf_down_the_first_entries(&after);
    assert_eq!(level_and_entry_width(&after, &leaf), (0, 20));
    widen_every_entry_of_the_node_by_one_byte(&mut after, &leaf);
    let allocation_leaf = report("allocation-record-20", &before, &after);

    // 分配记录树内部条目 96 → 97。
    let before = build_pool("attack-w13-allocation-internal").memory_pool();
    let mut after = before.clone();
    let allocation_root = tree_table_entry_of_kind(&after, TREE_KIND_ALLOCATION).root;
    assert_eq!(level_and_entry_width(&after, &allocation_root).1, 96);
    widen_every_entry_of_the_node_by_one_byte(&mut after, &allocation_root);
    let allocation_internal = report("allocation-internal-96", &before, &after);

    // deadlist 条目 43 → 44（快照下覆盖写之后，deadlist 根兼叶）。
    let before = image_with_a_deadlist();
    let mut after = before.clone();
    let deadlist_root = tree_table_entry_of_kind(&after, TREE_KIND_DEADLIST).root;
    assert_eq!(level_and_entry_width(&after, &deadlist_root), (0, 43));
    widen_every_entry_of_the_node_by_one_byte(&mut after, &deadlist_root);
    let deadlist = report("deadlist-43", &before, &after);

    println!(
        "name=summary I-1.10-red extent-internal={} extent-leaf={} inode-internal={} allocation-record={} allocation-internal={} deadlist={}",
        mentions_one_ten(&extent_internal),
        mentions_one_ten(&extent_leaf),
        mentions_one_ten(&inode_internal),
        mentions_one_ten(&allocation_leaf),
        mentions_one_ten(&allocation_internal),
        mentions_one_ten(&deadlist),
    );
    println!("name=summary deadlist-checker-new-not-holding={}", deadlist.len());
}
