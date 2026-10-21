//! 扫描重建对谱系记录与退役记录的两条读法（用户 2026-10-14 定，`research/prompts/m4-batch2-code-r1-forks.md` 第 1 行「甲」）：
//! 谱系记录（C615（扫盘重建对挂着时的回退没有输入））按「不是任何现行版本的成员」认领，退役记录（C768（退役记录的扫描重建读侧没有消费者））
//! 按退役树认领。池级 checker 的 I-1.13（扫描重建与遍历方向的索引相同） 在挂着回退过、删空过一片叶、左片吸收右片之后的镜像上真判成立；
//! 必红世界：抹掉那条谱系记录、抹掉那条退役记录，I-1.13 各判违例。删除中由两侧一起认（判决 `research/prompts/m4-c860-r1-main-verification.md`
//! 第三节 C2，用户 2026-10-20 选丙：扫描一侧墓碑压着的记录，遍历方向也有同号、同对象出生代的、两侧 nlink 都是 0 才算）的两形必红世界：
//! C860（扫描重建认删除中只凭扫描一侧）那一形（抹掉退役记录、再把退役那片叶最后一版的映射条目补回遍历方向，判决第二节 K3（一））、
//! 第一个文件删一半时多一条墓碑压着带数据单元的活文件 inode 2（K3（二）），再叠上挪 inode 2 一条映射落点（判决
//! `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5 必红世界那一招），I-1.13 都判违例。
//! 另有一条慢档用例在随机历史上数成立 / 不适用 / 违例三个计数（判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 T8 的口径）。
//! 读法本身在 `crates/singlefs-checker/src/rebuild.rs`，两侧一起认删除中在 `crates/singlefs-checker/src/walk.rs` 的 `inodes_being_deleted_on_both_sides`。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::collections::BTreeMap;
use std::sync::Mutex;

use common::{
    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, BuiltPool,
};
use common_deletion_in_progress::{
    append_a_copy_of_the_first_tombstone_of_the_newest_root,
    change_the_unit_and_propagate_its_checksum, first_tombstone_container_slot_of_the_newest_root,
    object_birth_of, pool_with_a_three_unit_file, pool_with_new_inodes,
    publish_new_inodes_and_advance_the_pool, publish_the_intent,
    publish_the_next_step_in_batches_of_one, roll_back_to, set_tombstone_field, unlink_to_the_end,
    INSTANCE_OF_THE_PUBLISHES, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use common_pointer_head_corruption::{
    data_unit_bytes, newest_root, node_bytes, replace_the_unit_and_propagate_its_checksum,
    the_slot_of, unit_bytes_at,
};
use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::InodeNumber;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{
    build_mapping_entry, mapping_key_for_data, mapping_key_for_node, mapping_key_sort_key,
    TombstoneRecord, NLINK_OF_AN_INODE_BEING_DELETED,
};
use singlefs_core::transaction::{
    publish_sequential_write_of_an_inode, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::{data_unit_payload_capacity, seal_header_checksum, UNIT_CLASS_PACKED};
use singlefs_format::{
    CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES,
    TOMBSTONE_RECORD_BYTES,
};
use singlefs_harness::heavy_threads::{
    own_variable_or_default_heavy_threads, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistoryOperationKind, HistorySeed, PerStepChecker,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::numeric_environment_variable::process_environment_variable;
use singlefs_harness::SharedStream;

/// 码 3 单元头里声明长度（偏移 8，2 字节）、载荷 CRC（89，4 字节）、记录数（69，2 字节）、记录宽（71，2 字节）的位置，
/// 明文头到 107、记录从 136 起（D18（块里携带什么信息） 已定项 11 的类身份段表与已定项 16 的头宽）。
const PACKED_DECLARED_LENGTH_OFFSET: usize = 8;
const PACKED_PAYLOAD_CHECKSUM_OFFSET: usize = 89;
const PACKED_RECORD_COUNT_OFFSET: usize = 69;
const PACKED_RECORD_WIDTH_OFFSET: usize = 71;
const PACKED_PLAIN_HEADER_END: usize = 107;
const PACKED_RECORDS_OFFSET: usize = 136;

/// 中央映射树节点（码 2，key 宽 27）头里随条目变的几处：层级 50、最小 key 52、最大 key 79（各 27 字节）、载荷 CRC 130、条目数 136（2 字节），
/// 明文头到 140（D18（块里携带什么信息） 已定项 18 的偏移表：52 起 key 区间 2k，之后诞生代号 8、fsid 8、写序 4、出生序号 4、载荷 CRC 4、预留 2、
/// 条目数 2、条目宽 2）；声明长度与码 3 同在共同前缀那一处。
const MAPPING_NODE_LEVEL_OFFSET: usize = 50;
const MAPPING_NODE_SMALLEST_KEY_OFFSET: usize = 52;
const MAPPING_NODE_LARGEST_KEY_OFFSET: usize = 79;
const MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET: usize = 130;
const MAPPING_NODE_ENTRY_COUNT_OFFSET: usize = 136;
const MAPPING_NODE_PLAIN_HEADER_END: usize = 140;
const MAPPING_NODE_DECLARED_LENGTH_OFFSET: usize = PACKED_DECLARED_LENGTH_OFFSET;

/// 第一个文件之外再建的空 inode 个数：234 条记录装成 [1..=233]、[234] 两片 inode 叶容器（一片装 233 条，`singlefs_format::INODE_LEAF_RECORDS`）。
const NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER: u64 = 233;
/// 右边那片叶里唯一的那条记录：unlink 它，那片叶删空摘掉、退役。
const THE_ONLY_INODE_OF_THE_RIGHT_LEAF: u64 = 234;
/// 左边那片叶里的一条：unlink 它，左片剩 232 条、与右邻合计 233 条装得下一片，左片吸收右片、右片退役。
const AN_INODE_WHOSE_UNLINK_ABSORBS_THE_RIGHT_LEAF: u64 = 2;

fn verdict_of_the_scan_rebuild(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| *invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX)
        .expect("I-1.13 在池级 checker 的清单里")
        .1
}

fn violation_detail_of_the_scan_rebuild(image: &MemoryPool, what: &str) -> String {
    match verdict_of_the_scan_rebuild(image) {
        InvariantVerdict::Violated(detail) => detail,
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("{what}：I-1.13 应当判违例，实际 {other:?}")
        }
    }
}

/// 和第一个文件一样长、每个字节加一的内容：覆盖写不改长度，回退抛弃的那一版在同一个 (inode, 偏移) 上有一个更新的数据单元。
fn content_of_the_same_length_differing_in_every_byte() -> Vec<u8> {
    file_content()
        .iter()
        .map(|byte| byte.wrapping_add(1))
        .collect()
}

/// 第一个文件那一版之后覆盖写一版（同长），再挂着回退到第一个文件那一版：谱系树里一条 (回退那次的 txg, 第一个文件那一版的 txg)。
fn pool_rolled_back_past_an_overwrite_of_the_same_length(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    let first_file_version = pool.output.clone();
    overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_the_same_length_differing_in_every_byte(),
    );
    roll_back_to(&mut pool, &first_file_version);
    assert_eq!(
        pool.output
            .lineage_containers
            .iter()
            .map(|container| container.contents.records.len())
            .sum::<usize>(),
        1,
        "回退那次发布写了一条谱系记录"
    );
    pool
}

/// 再建 233 个空 inode 之后 unlink `inode` 到底：右片叶退役，退役树里一条记录。
fn pool_whose_right_leaf_retired_after_the_unlink_of(tag: &str, inode: u64) -> BuiltPool {
    let mut pool = pool_with_new_inodes(tag, NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER);
    assert_eq!(
        pool.output.inode_leaf_containers.len(),
        2,
        "234 条记录装成两片叶"
    );
    unlink_to_the_end(&mut pool, inode);
    assert_eq!(
        (
            pool.output.inode_leaf_containers.len(),
            pool.output
                .retirement_containers
                .iter()
                .map(|container| container.contents.records.len())
                .sum::<usize>(),
        ),
        (1, 1),
        "unlink inode {inode} 到底之后只剩一片叶，右片退役、退役树里一条记录"
    );
    pool
}

/// 一片码 3 容器的记录全部抹掉：记录数、声明长度清零，记录那一段清零，载荷 CRC 与头校验和重封，新的整单元校验和沿引用链补到根槽。
/// 遍历方向读的三张表不经这片容器，只有扫描重建读它。
fn erase_every_record_of_the_container(image: &mut MemoryPool, pointer: &NodePointer) {
    let slot = the_slot_of(pointer);
    let mut unit = unit_bytes_at(image, slot, data_unit_bytes());
    let record_count = usize::from(u16::from_le_bytes(
        unit[PACKED_RECORD_COUNT_OFFSET..PACKED_RECORD_COUNT_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    ));
    let record_width = usize::from(u16::from_le_bytes(
        unit[PACKED_RECORD_WIDTH_OFFSET..PACKED_RECORD_WIDTH_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    ));
    assert!(record_count > 0, "要抹的那片容器里有记录");
    unit[PACKED_RECORDS_OFFSET..PACKED_RECORDS_OFFSET + record_count * record_width].fill(0);
    unit[PACKED_RECORD_COUNT_OFFSET..PACKED_RECORD_COUNT_OFFSET + 2].fill(0);
    unit[PACKED_DECLARED_LENGTH_OFFSET..PACKED_DECLARED_LENGTH_OFFSET + 2].fill(0);
    let payload_checksum = crc32_castagnoli(&unit[PACKED_PLAIN_HEADER_END..]);
    unit[PACKED_PAYLOAD_CHECKSUM_OFFSET..PACKED_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(&mut unit, PACKED_PLAIN_HEADER_END);
    replace_the_unit_and_propagate_its_checksum(image, slot, &unit);
}

/// 最新根的中央映射树（这张镜像上是根兼叶）里按 key 序插进 `entry` 那一条：条目数、声明长度、最小与最大 key 跟着改，
/// 载荷 CRC 与头校验和重封，新的整单元校验和沿引用链补到根槽。扫描重建不读码 2，只有遍历方向读到这一条。
fn insert_the_entry_into_the_newest_mapping_leaf(image: &mut MemoryPool, entry: &[u8]) {
    let key_width = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let entry_width = usize::try_from(MAPPING_ENTRY_BYTES).expect("55");
    let entries_start = usize::try_from(CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES).expect("169");
    assert_eq!(entry.len(), entry_width, "要插的是一条整的映射叶条目");
    let leaf_slot = the_slot_of(&newest_root(image).mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(
        leaf[MAPPING_NODE_LEVEL_OFFSET], 0,
        "这张镜像上中央映射树只有一层：根就是叶"
    );
    let entry_count = usize::from(u16::from_le_bytes(
        leaf[MAPPING_NODE_ENTRY_COUNT_OFFSET..MAPPING_NODE_ENTRY_COUNT_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    ));
    let mut entries: Vec<Vec<u8>> = (0..entry_count)
        .map(|index| {
            let start = entries_start + index * entry_width;
            leaf[start..start + entry_width].to_vec()
        })
        .collect();
    assert!(
        entries.windows(2).all(|neighbours| {
            mapping_key_sort_key(&neighbours[0][..key_width])
                < mapping_key_sort_key(&neighbours[1][..key_width])
        }),
        "映射叶的条目按 key 的字段序严格升序"
    );
    let sort_key_of_the_entry = mapping_key_sort_key(&entry[..key_width]);
    let position = entries.partition_point(|existing| {
        mapping_key_sort_key(&existing[..key_width]) < sort_key_of_the_entry
    });
    assert!(
        entries
            .get(position)
            .is_none_or(|next| next[..key_width] != entry[..key_width]),
        "插之前映射叶里没有这把 key"
    );
    entries.insert(position, entry.to_vec());
    assert!(
        entries_start + entries.len() * entry_width <= leaf.len(),
        "映射叶装得下多一条"
    );
    for (index, each_entry) in entries.iter().enumerate() {
        let start = entries_start + index * entry_width;
        leaf[start..start + entry_width].copy_from_slice(each_entry);
    }
    leaf[MAPPING_NODE_ENTRY_COUNT_OFFSET..MAPPING_NODE_ENTRY_COUNT_OFFSET + 2].copy_from_slice(
        &u16::try_from(entries.len())
            .expect("条目数 2 字节")
            .to_le_bytes(),
    );
    leaf[MAPPING_NODE_DECLARED_LENGTH_OFFSET..MAPPING_NODE_DECLARED_LENGTH_OFFSET + 2]
        .copy_from_slice(
            &u16::try_from(entries.len() * entry_width)
                .expect("声明长度 2 字节")
                .to_le_bytes(),
        );
    leaf[MAPPING_NODE_SMALLEST_KEY_OFFSET..MAPPING_NODE_SMALLEST_KEY_OFFSET + key_width]
        .copy_from_slice(&entries.first().expect("刚插进一条")[..key_width]);
    leaf[MAPPING_NODE_LARGEST_KEY_OFFSET..MAPPING_NODE_LARGEST_KEY_OFFSET + key_width]
        .copy_from_slice(&entries.last().expect("刚插进一条")[..key_width]);
    let payload_checksum = crc32_castagnoli(&leaf[MAPPING_NODE_PLAIN_HEADER_END..]);
    leaf[MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET..MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(&mut leaf, MAPPING_NODE_PLAIN_HEADER_END);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
}

#[test]
fn the_scan_rebuild_holds_after_a_rollback_while_mounted_past_an_overwrite_of_the_same_length() {
    let pool = pool_rolled_back_past_an_overwrite_of_the_same_length("scan-rebuild-rolled-back");
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "覆盖写那一版的数据单元与 inode 叶容器写序更大、却诞生在回退抛弃的那一段里：不是任何现行版本的成员，扫描重建择回退目标那一版，与遍历方向逐项相同"
    );
}

#[test]
fn erasing_the_lineage_record_reddens_the_scan_rebuild_on_the_rolled_back_pool() {
    let pool = pool_rolled_back_past_an_overwrite_of_the_same_length("scan-rebuild-lineage-erased");
    let mut image = pool.memory_pool();
    erase_every_record_of_the_container(&mut image, &pool.output.lineage_containers[0].pointer);
    let detail = violation_detail_of_the_scan_rebuild(&image, "抹掉回退留下的那条谱系记录");
    assert!(
        detail.contains("「每个 inode 的数据单元集合」差 1 处")
            && detail.contains("「inode 记录集合」差 1 处"),
        "没了谱系记录，扫描重建择回退抛弃的那一版（写序更大）：数据单元集合与 inode 记录集合各差那一处：{detail}"
    );
}

#[test]
fn the_scan_rebuild_holds_after_the_unlink_that_empties_and_retires_the_right_leaf() {
    let pool = pool_whose_right_leaf_retired_after_the_unlink_of(
        "scan-rebuild-emptied-leaf-retired",
        THE_ONLY_INODE_OF_THE_RIGHT_LEAF,
    );
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "右片叶删空退役：它最后那一版还扫得到、装着 inode 234 的记录，按退役树不认领，与遍历方向逐项相同"
    );
}

#[test]
fn the_scan_rebuild_holds_after_the_unlink_whose_left_leaf_absorbs_the_right_one() {
    let pool = pool_whose_right_leaf_retired_after_the_unlink_of(
        "scan-rebuild-absorbed-leaf-retired",
        AN_INODE_WHOSE_UNLINK_ABSORBS_THE_RIGHT_LEAF,
    );
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "左片吸收右片、右片退役：右片最后那一版里的 inode 234 与左片新版本里的同号，按退役树只认左片的，与遍历方向逐项相同"
    );
}

/// 抹掉右片叶删空退役留下的那条退役记录：扫描重建认领退役那片叶最后那一版（unlink 意图那次发布写的，只装 inode 234 那一条 nlink 0 的记录），
/// I-1.13 判违例，红落在两张表：「中央映射」那片叶最后一版的码 3 容器 key 只有扫描重建那一边有（遍历方向从最新根读不到这片叶），
/// 落点是意图那一版 `inode_leaf_containers[1]` 的槽；「inode 记录集合」差 inode 234 那一条。那一版里 234 的记录 nlink 0、墓碑 (234, 对象出生代)
/// 在谱系上，可遍历方向那一侧的 inode 树里已经没有 234：删除中由两侧一起认（判决 `research/prompts/m4-c860-r1-main-verification.md` 第三节 C2，
/// 用户 2026-10-20 选丙），遍历方向没有同号、同对象出生代的记录就不算删除中，扫描重建那条记录照比、对不上。234 没有数据单元，
/// 「每个 inode 的数据单元集合」不差。
/// 判别力：不算删除中的那条记录不放回扫描重建那一侧（`crates/mutations.tsv` 以「扫描重建 M5 退役记录：」起头那一行）时 234 的记录两侧都不在，
/// 「inode 记录集合」不差，这一条红；认删除中退回只凭扫描一侧（以「扫描重建丙：认删除中不核遍历方向那一侧」起头那一行）同样让它红。
#[test]
fn erasing_the_retirement_record_reddens_the_scan_rebuild_after_the_right_leaf_retired() {
    let mut pool = pool_with_new_inodes(
        "scan-rebuild-retirement-erased",
        NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER,
    );
    let intent = unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
    assert_eq!(
        intent.inode_leaf_containers.len(),
        2,
        "意图那一版还是两片叶"
    );
    assert_eq!(
        (
            pool.output.inode_leaf_containers.len(),
            pool.output
                .retirement_containers
                .iter()
                .map(|container| container.contents.records.len())
                .sum::<usize>(),
        ),
        (1, 1),
        "unlink inode {THE_ONLY_INODE_OF_THE_RIGHT_LEAF} 到底之后只剩一片叶，右片退役、退役树里一条记录"
    );
    let slot_of_the_right_leaf_at_the_intent =
        the_slot_of(&intent.inode_leaf_containers[1].pointer);
    let mut image = pool.memory_pool();
    erase_every_record_of_the_container(&mut image, &pool.output.retirement_containers[0].pointer);
    let detail = violation_detail_of_the_scan_rebuild(&image, "抹掉删空那片叶留下的退役记录");
    assert!(
        detail.contains("「中央映射」差 1 处，第一处 key （码 3、")
            && detail.contains(&format!("槽 {}、", slot_of_the_right_leaf_at_the_intent.0))
            && detail.contains("遍历方向没有这一条"),
        "没了退役记录，扫描重建认领退役那片叶最后那一版：中央映射差那片叶的码 3 容器 key、落在意图那一版右片叶的槽：{detail}"
    );
    assert!(
        detail.contains(&format!(
            "「inode 记录集合」差 1 处，第一处 key inode {THE_ONLY_INODE_OF_THE_RIGHT_LEAF}："
        )) && !detail.contains("「每个 inode 的数据单元集合」"),
        "原先按「只凭扫描一侧认删除中」写死的是「inode 记录集合」不差；用户 2026-10-20 选丙（判决 m4-c860-r1 第三节 C2）之后删除中要两侧都有\
         nlink 0 的同一条记录，遍历方向那一侧已没有 inode 234，它不算删除中、两侧照比，「inode 记录集合」差 234 那一处；它没有数据单元，\
         「每个 inode 的数据单元集合」不差：{detail}"
    );
}

/// C860（扫描重建认删除中只凭扫描一侧，`.claude/kb/checks-owed.md`）那一形：抹掉右片叶删空退役留下的退役记录，再把那片叶最后一版
/// （unlink 意图那次发布写的，只装 inode 234 一条 nlink 0 的记录）的映射条目补回遍历方向的中央映射叶。上一条用例里把 I-1.13 判红的
/// 「中央映射」那一差（那片叶的码 3 容器 key 只在扫描重建那一边）由此去掉，两侧唯一的差是早已删完的 inode 234 那条记录：扫描重建那一边
/// 认领了它、遍历方向那一边的 inode 树里没有它，而谱系上的墓碑 (234, 对象出生代) 压着它。用例先把这几样做成断言（补回之前两侧差那片叶的
/// 码 3 容器 key 与 234 那条记录），再断言 I-1.13 判红、红只落在「inode 记录集合」inode 234 那一处。
/// 判决 `research/prompts/m4-c860-r1-main-verification.md` 第二节 K3（一）：只凭扫描那一边的「墓碑 + 记录」认删除中时（甲），遍历方向那一边没有
/// 这条记录也认，这唯一的差被两侧一起摘掉、I-1.13 判成立。用户 2026-10-20 选丙（第三节 C2）：遍历方向也有同号、同对象出生代的记录，且两侧
/// nlink 都是 0 才算删除中（`crates/singlefs-checker/src/walk.rs` 的 `inodes_being_deleted_on_both_sides`），这里遍历方向没有 234，它不算，判违例。
/// 判别力：`crates/mutations.tsv` 以「扫描重建丙：认删除中不核遍历方向那一侧」起头那一行（退回甲）让这一条红在最后取违例那一步（实际 `Holds`）。
#[test]
fn a_retired_leaf_left_in_the_walked_mapping_without_its_retirement_record_leaves_only_the_deleted_inode_record_and_reddens_the_scan_rebuild(
) {
    let mut pool = pool_with_new_inodes(
        "scan-rebuild-retired-leaf-left-in-the-walked-mapping",
        NEW_INODES_FILLING_ONE_LEAF_AND_STARTING_ANOTHER,
    );
    let intent = unlink_to_the_end(&mut pool, THE_ONLY_INODE_OF_THE_RIGHT_LEAF);
    let right_leaf_at_the_intent = &intent.inode_leaf_containers[1];
    let records_of_the_right_leaf_at_the_intent: Vec<(u64, u32)> = right_leaf_at_the_intent
        .contents
        .records
        .iter()
        .map(|record| (record.inode, record.nlink))
        .collect();
    assert_eq!(
        records_of_the_right_leaf_at_the_intent,
        vec![(
            THE_ONLY_INODE_OF_THE_RIGHT_LEAF,
            NLINK_OF_AN_INODE_BEING_DELETED
        )],
        "右片叶最后那一版（意图那一版）只装 inode 234 一条、nlink 0"
    );
    let object_birth_of_the_deleted_inode =
        right_leaf_at_the_intent.contents.records[0].object_birth;
    assert!(
        pool.output
            .tombstone_containers
            .iter()
            .flat_map(|container| container.contents.records.iter())
            .any(|record| {
                record.object == InodeNumber(THE_ONLY_INODE_OF_THE_RIGHT_LEAF)
                    && record.object_birth == object_birth_of_the_deleted_inode
                    && record.range_start_in_bytes
                        == TombstoneRecord::WHOLE_OBJECT_RANGE_START_IN_BYTES
                    && record.range_end_in_bytes == TombstoneRecord::WHOLE_OBJECT_RANGE_END_IN_BYTES
            }),
        "最新那一版的墓碑树里有 (234, 对象出生代) 的整对象墓碑（没回退过，死亡 txg 都在谱系上）：{:?}",
        pool.output.tombstone_containers
    );
    assert!(
        pool.output
            .inode_leaf_containers
            .iter()
            .all(|container| container
                .contents
                .records
                .iter()
                .all(|record| record.inode != THE_ONLY_INODE_OF_THE_RIGHT_LEAF)),
        "遍历方向读的那一版 inode 树里没有 inode 234 的记录"
    );
    assert_eq!(
        pool.output
            .retirement_containers
            .iter()
            .map(|container| container.contents.records.len())
            .sum::<usize>(),
        1,
        "右片退役、退役树里一条记录"
    );
    let mut image = pool.memory_pool();
    erase_every_record_of_the_container(&mut image, &pool.output.retirement_containers[0].pointer);
    let slot_of_the_right_leaf_at_the_intent = the_slot_of(&right_leaf_at_the_intent.pointer);
    let detail_before_the_mapping_entry_is_put_back =
        violation_detail_of_the_scan_rebuild(&image, "抹掉退役记录、映射条目还没补回");
    assert!(
        detail_before_the_mapping_entry_is_put_back.contains("「中央映射」差 1 处，第一处 key （码 3、")
            && detail_before_the_mapping_entry_is_put_back
                .contains(&format!("槽 {}、", slot_of_the_right_leaf_at_the_intent.0))
            && detail_before_the_mapping_entry_is_put_back.contains("遍历方向没有这一条")
            && !detail_before_the_mapping_entry_is_put_back
                .contains("「每个 inode 的数据单元集合」")
            && detail_before_the_mapping_entry_is_put_back.contains(&format!(
                "「inode 记录集合」差 1 处，第一处 key inode {THE_ONLY_INODE_OF_THE_RIGHT_LEAF}："
            )),
        "补回映射条目之前，两侧差 inode 234 那条记录（遍历方向那一边没有它，丙下不算删除中、照比）与那片叶最后一版的码 3 容器 key：\
         {detail_before_the_mapping_entry_is_put_back}"
    );
    insert_the_entry_into_the_newest_mapping_leaf(
        &mut image,
        &build_mapping_entry(
            &mapping_key_for_node(UNIT_CLASS_PACKED, right_leaf_at_the_intent.pointer),
            right_leaf_at_the_intent.pointer.locations,
        ),
    );
    let detail = violation_detail_of_the_scan_rebuild(
        &image,
        "抹掉退役记录、把退役那片叶最后一版的映射条目补回遍历方向（两侧唯一的差是删完的 inode 234 那条记录）",
    );
    assert!(
        detail.contains(&format!(
            "「inode 记录集合」差 1 处，第一处 key inode {THE_ONLY_INODE_OF_THE_RIGHT_LEAF}："
        )) && !detail.contains("「中央映射」")
            && !detail.contains("「每个 inode 的数据单元集合」"),
        "两侧唯一的差是 inode 234 那条记录：扫描重建那一边有、遍历方向那一边没有，红只落在「inode 记录集合」：{detail}"
    );
}

/// 中央映射叶条目里第一条位置条目的槽号（6 字节）在 key 之后、设备身份 4 字节之后（D19（块指针的结构与宽度预算） 已定项 6 / 10）。
const FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY: usize = 27 + 4;
/// 第一个文件删一半的那张镜像上，映射条目的落点挪多远：单元区里那几次发布都没写到的一格（`scan_rebuild_agrees_with_the_walked_index.rs`
/// 的同名常量：挪两槽会落在 inode 2 的下一个单元上，先红 I-5.1、I-1.13 报不适用）。
const SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY: u64 = 4096;
/// 墓碑记录里对象 ID 的偏移（D18（块里携带什么信息） 已定项 24 第 2 条：对象 ID 0、对象出生代 8、区间起 16、区间止 24、死亡 txg 32）。
const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;
/// 码 3 容器里第一条记录的偏移（同 `PACKED_RECORDS_OFFSET`）之后第二条墓碑的起点。
const SECOND_TOMBSTONE_RECORD_OFFSET: usize = PACKED_RECORDS_OFFSET + 40;

/// 攻方 F 段那一形（`scan_rebuild_agrees_with_the_walked_index.rs` 的同名函数，那份文件不归这一批改，照抄一份）：第一个文件顺序写成三个单元，
/// 建 inode 2、按 inode 号写它两个单元，再删第一个文件：发完意图与第一批（批大小 1）。这一版里 inode 1 删除中（墓碑压着、记录还在、
/// 剩两个单元），inode 2 活着（nlink 1、两个数据单元）。
fn pool_with_the_first_file_half_deleted_beside_inode_two(tag: &str) -> BuiltPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let content_of_inode_two: Vec<u8> = (0..2 * data_unit_payload_capacity() - 5)
        .map(|index| u8::try_from((index * 7 + 3) % 251).expect("小于 256"))
        .collect();
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let written = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_sequential_write_of_an_inode(
            &mut writer,
            &mut pool.allocator,
            &previous,
            InodeNumber(2),
            &content_of_inode_two,
            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
            INSTANCE_OF_THE_PUBLISHES,
        )
        .expect("按 inode 号顺序写 inode 2 两个单元")
    };
    pool.output = written;
    publish_the_intent(&mut pool);
    let after_the_first_batch = publish_the_next_step_in_batches_of_one(&mut pool);
    assert_eq!(
        (
            after_the_first_batch.data_pointers.len(),
            after_the_first_batch.deleted_inodes.clone()
        ),
        (2, vec![InodeNumber(FIRST_INODE_NUMBER)]),
        "第一批之后第一个文件剩两个单元、还在删除队列里"
    );
    pool
}

/// 判决 `research/prompts/m4-c860-r1-main-verification.md` 第二节 K3（二）的坏法：最新根那一版墓碑树第一片容器里只有删除中那个文件的一条墓碑，
/// 原样追加一条，再把追加的那条改成压着 (`inode`, `object_birth`)：区间（整对象）与死亡 txg（在谱系上）照抄，重封、补校验和。
fn append_a_tombstone_pressing(image: &mut MemoryPool, inode: u64, object_birth: u64) {
    append_a_copy_of_the_first_tombstone_of_the_newest_root(image);
    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
    let tombstone_record_bytes = usize::try_from(TOMBSTONE_RECORD_BYTES).expect("40");
    assert_eq!(
        SECOND_TOMBSTONE_RECORD_OFFSET,
        PACKED_RECORDS_OFFSET + tombstone_record_bytes,
        "第二条墓碑紧接第一条"
    );
    change_the_unit_and_propagate_its_checksum(
        image,
        container_slot,
        data_unit_bytes(),
        |container| {
            let appended = &mut container[SECOND_TOMBSTONE_RECORD_OFFSET
                ..SECOND_TOMBSTONE_RECORD_OFFSET + tombstone_record_bytes];
            set_tombstone_field(appended, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET, inode);
            set_tombstone_field(appended, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET, object_birth);
        },
    );
}

/// M5 必红世界那一招（`scan_rebuild_agrees_with_the_walked_index.rs` 的同名函数照抄）：最新根的中央映射树（根兼叶）里 key 是 `mapping_key`
/// 的那条条目，第一条位置条目的槽号挪 [`SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY`] 槽；映射叶重封、新的整单元校验和沿引用链补到根槽。
/// 交回原来那个槽。
fn move_the_first_location_of_the_mapping_entry_keyed(
    image: &mut MemoryPool,
    mapping_key: &[u8],
) -> u64 {
    let key_width = usize::try_from(MAPPING_KEY_BYTES).expect("27");
    let entry_width = usize::try_from(MAPPING_ENTRY_BYTES).expect("55");
    let entries_start = usize::try_from(CENTRAL_MAPPING_TREE_INDEX_NODE_HEADER_BYTES).expect("169");
    assert_eq!(
        FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY,
        key_width + 4,
        "位置条目的槽号在 key 与设备身份之后"
    );
    let leaf_slot = the_slot_of(&newest_root(image).mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(
        leaf[MAPPING_NODE_LEVEL_OFFSET], 0,
        "这张镜像上中央映射树只有一层：根就是叶"
    );
    let entry_count = usize::from(u16::from_le_bytes(
        leaf[MAPPING_NODE_ENTRY_COUNT_OFFSET..MAPPING_NODE_ENTRY_COUNT_OFFSET + 2]
            .try_into()
            .expect("2 字节"),
    ));
    let entry_start = (0..entry_count)
        .map(|index| entries_start + index * entry_width)
        .find(|start| leaf[*start..*start + key_width] == *mapping_key)
        .expect("中央映射里有这把 key 的条目");
    let slot_field_start = entry_start + FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY;
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&leaf[slot_field_start..slot_field_start + 6]);
    let original_slot = u64::from_le_bytes(slot_bytes);
    let moved_slot = original_slot + SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY;
    leaf[slot_field_start..slot_field_start + 6].copy_from_slice(&moved_slot.to_le_bytes()[..6]);
    let payload_checksum = crc32_castagnoli(&leaf[MAPPING_NODE_PLAIN_HEADER_END..]);
    leaf[MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET..MAPPING_NODE_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(&mut leaf, MAPPING_NODE_PLAIN_HEADER_END);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
    original_slot
}

/// 第一个文件删一半那张镜像上多一条墓碑压着带两个数据单元的活文件 inode 2（判决 m4-c860-r1 第二节 K3（二）的镜像）。交回搭出那张镜像的池
/// （取 inode 2 的数据指针用）与改过的镜像。
fn image_with_an_extra_tombstone_pressing_the_live_inode_two(tag: &str) -> (BuiltPool, MemoryPool) {
    let pool = pool_with_the_first_file_half_deleted_beside_inode_two(tag);
    let object_birth_of_inode_two = object_birth_of(&pool.output, 2)
        .expect("inode 2 在最新那一版 inode 树里有记录")
        .0;
    let mut image = pool.memory_pool();
    append_a_tombstone_pressing(&mut image, 2, object_birth_of_inode_two);
    (pool, image)
}

/// 判决 `research/prompts/m4-c860-r1-main-verification.md` 第二节 K3（二）：第一个文件删一半时多一条墓碑压着活着的 inode 2（nlink 1、两个数据单元）。
/// 甲、乙都把 inode 2 整个移出比对、判成立；用户 2026-10-20 选丙（第三节 C2）：两侧那条记录 nlink 都是 1，不算删除中，两侧照比——
/// 扫描重建那一侧按墓碑不认领 inode 2 的两个数据单元，遍历方向照收，「每个 inode 的数据单元集合」差那两处，I-1.13 判违例；inode 2 的记录两侧相同。
/// 判别力：`crates/mutations.tsv` 以「扫描重建丙：认删除中不核 nlink」起头那一行（退回乙）让这一条红在取违例那一步。
#[test]
fn an_extra_tombstone_pressing_a_live_file_while_the_first_file_is_half_deleted_reddens_the_scan_rebuild(
) {
    let (_pool, image) = image_with_an_extra_tombstone_pressing_the_live_inode_two(
        "scan-rebuild-extra-tombstone-on-live-inode-two",
    );
    let detail = violation_detail_of_the_scan_rebuild(
        &image,
        "第一个文件删一半时多一条墓碑压着活着的 inode 2",
    );
    assert!(
        detail.contains("「每个 inode 的数据单元集合」差 2 处，第一处 key （inode 2、偏移 0）")
            && !detail.contains("「inode 记录集合」"),
        "inode 2 两侧 nlink 都是 1、不算删除中：它的两个数据单元扫描重建那一侧按墓碑不认领、遍历方向照收，差那两处；它的记录两侧相同：{detail}"
    );
}

/// 同上一条的镜像，再叠上 M5 必红世界那一招：挪 inode 2 第一个数据单元那条映射条目的落点（判决
/// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5）。甲、乙下 inode 2 整个移出比对、这一处改坏也被遮掉、判成立
/// （判决 m4-c860-r1 第二节 K3（二），腿报的）；丙下 inode 2 照比，I-1.13 判违例，中央映射差 inode 2 的两个数据单元 key。
/// 判别力：`crates/mutations.tsv` 以「扫描重建丙：墓碑压着、记录还在的对象的数据单元照认领」起头那一行让 inode 2 的单元认回来、
/// 中央映射只差挪过落点那一条，这一条红在「差 2 处」那一句断言上。
#[test]
fn an_extra_tombstone_on_a_live_file_does_not_hide_a_moved_mapping_location_of_that_file_from_the_scan_rebuild(
) {
    let (pool, mut image) = image_with_an_extra_tombstone_pressing_the_live_inode_two(
        "scan-rebuild-extra-tombstone-and-moved-mapping-of-inode-two",
    );
    let first_data_pointer_of_inode_two = pool
        .output
        .file_other_than_the_first(InodeNumber(2))
        .expect("inode 2 带数据单元")
        .data_pointers[0];
    move_the_first_location_of_the_mapping_entry_keyed(
        &mut image,
        &mapping_key_for_data(
            first_data_pointer_of_inode_two.head,
            first_data_pointer_of_inode_two.write_order,
        ),
    );
    let detail = violation_detail_of_the_scan_rebuild(
        &image,
        "多一条墓碑压着活着的 inode 2，又挪了 inode 2 一条映射条目的落点",
    );
    assert!(
        detail.contains("「中央映射」差 2 处")
            && detail.contains("「每个 inode 的数据单元集合」差 2 处，第一处 key （inode 2、偏移 0）"),
        "inode 2 不算删除中、照比：中央映射差它两个数据单元的 key（挪过落点的那一条也在里面），数据单元集合差那两处：{detail}"
    );
}

/// 随机历史的四批（判决 `research/prompts/m4-batch2-code-r1-main-verification.md` 第二节 T8 那一格的攻方 s2：
/// `research/prompts/m4-batch2-code-r1-opus-model/tools/rerun.sh` 的 s2 那一段）：(首个种子, 种子数, 比重)，每段 30 步、384 槽的小盘。
const RANDOM_HISTORY_BATCHES_OF_THE_T8_COUNT: [(u64, u64, &GenerationWeights); 4] = [
    (1000, 8, &GenerationWeights::BROAD),
    (
        2000,
        16,
        &GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
    ),
    (3000, 16, &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR),
    (
        4000,
        16,
        &GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,
    ),
];
const OPERATIONS_OF_EACH_RANDOM_HISTORY_OF_THE_T8_COUNT: usize = 30;
const WORKER_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_RANDOM_HISTORY_THREADS";

/// 一个镜像上 I-1.13 的判定归哪一类：成立、违例，或不适用连同理由。
fn verdict_class_of(verdict: &InvariantVerdict) -> String {
    match verdict {
        InvariantVerdict::Holds => "成立".to_string(),
        InvariantVerdict::Violated(_) => "违例".to_string(),
        InvariantVerdict::NotApplicable(reason) => format!("不适用：{reason}"),
        InvariantVerdict::PartiallyJudged(_) => "部分没判".to_string(),
    }
}

/// 数到的：全部状态与「删过一次文件之后」的状态（上一步是建 inode：随机历史只在删完一个文件之后才建 inode，T8 那一格的同一口径）
/// 各按判定类计数，违例逐条记种子、步号与说明。
#[derive(Default)]
struct TallyOfTheScanRebuildOverRandomHistories {
    every_state: BTreeMap<String, u64>,
    states_after_a_deletion: BTreeMap<String, u64>,
    violations: Vec<String>,
}

#[test]
#[ignore = "harness 慢档用例：56 段随机历史、每步跑一遍池级 checker，数 I-1.13 成立 / 不适用 / 违例；线程数 SINGLEFS_RANDOM_HISTORY_THREADS，没设取 SINGLEFS_THREAD_CAP，再没设取 .claude/heavy-thread-default 的登记值"]
fn the_scan_rebuild_is_never_violated_and_holds_after_deletions_over_random_histories() {
    let worker_threads = own_variable_or_default_heavy_threads(
        WORKER_THREADS_ENVIRONMENT_VARIABLE,
        process_environment_variable(WORKER_THREADS_ENVIRONMENT_VARIABLE).as_deref(),
        process_environment_variable(THREAD_CAP_ENVIRONMENT_VARIABLE).as_deref(),
    )
    .get();
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let histories: Vec<(HistorySeed, &GenerationWeights)> = RANDOM_HISTORY_BATCHES_OF_THE_T8_COUNT
        .iter()
        .flat_map(|(first_seed, seed_count, weights)| {
            (*first_seed..*first_seed + *seed_count).map(move |seed| (HistorySeed(seed), *weights))
        })
        .collect();
    let next_history = Mutex::new(0usize);
    let tally = Mutex::new(TallyOfTheScanRebuildOverRandomHistories::default());
    std::thread::scope(|scope| {
        for _ in 0..worker_threads {
            scope.spawn(|| loop {
                let history_index = {
                    let mut next = next_history.lock().expect("取下一段历史的锁没被毒化");
                    if *next >= histories.len() {
                        break;
                    }
                    *next += 1;
                    *next - 1
                };
                let (seed, weights) = histories[history_index];
                let history = generate_history_with_weights(
                    seed,
                    OPERATIONS_OF_EACH_RANDOM_HISTORY_OF_THE_T8_COUNT,
                    weights,
                );
                // 一段历史第一次失败就停（执行器文档），停之前那几步照样数进去：T8 那一格的攻方 s2 同一个数法。
                let _history_run = execute_history_with(
                    &history,
                    execution,
                    &SharedStream::new(),
                    &mut |observation| {
                        let verdict = verdict_of_the_scan_rebuild(observation.image);
                        let class = verdict_class_of(&verdict);
                        let is_after_a_deletion = observation.operation.is_some_and(|operation| {
                            operation.kind() == HistoryOperationKind::CreateInode
                        });
                        let mut tally = tally.lock().expect("计数的锁没被毒化");
                        *tally.every_state.entry(class.clone()).or_insert(0) += 1;
                        if is_after_a_deletion {
                            *tally.states_after_a_deletion.entry(class).or_insert(0) += 1;
                        }
                        if let InvariantVerdict::Violated(detail) = verdict {
                            tally.violations.push(format!(
                                "种子 {} {:?}：{detail}",
                                seed.0, observation.position
                            ));
                        }
                    },
                );
            });
        }
    });
    let tally = tally.into_inner().expect("计数的锁没被毒化");
    for (class, count) in &tally.every_state {
        println!("SCAN_REBUILD_TALLY every_state {count} {class}");
    }
    for (class, count) in &tally.states_after_a_deletion {
        println!("SCAN_REBUILD_TALLY after_a_deletion {count} {class}");
    }
    let total_states: u64 = tally.every_state.values().sum();
    let states_after_a_deletion: u64 = tally.states_after_a_deletion.values().sum();
    println!(
        "SCAN_REBUILD_TALLY_SUMMARY histories={} states={total_states} states_after_a_deletion={states_after_a_deletion} violated={}",
        histories.len(),
        tally.violations.len()
    );
    assert!(
        tally.violations.is_empty(),
        "随机历史上 I-1.13 判了违例 {} 次，前几条：{:?}",
        tally.violations.len(),
        tally.violations.iter().take(5).collect::<Vec<_>>()
    );
    assert!(
        tally
            .states_after_a_deletion
            .get("成立")
            .copied()
            .unwrap_or(0)
            > 0,
        "删过一次文件之后的 {states_after_a_deletion} 个状态里 I-1.13 一个都没判成立：{:?}",
        tally.states_after_a_deletion
    );
}
