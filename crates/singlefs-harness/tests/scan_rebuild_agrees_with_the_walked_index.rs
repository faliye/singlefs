//! 只读扫描重建与遍历方向的索引逐项比（里程碑四四·2，题面 `.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」）：
//! 池级 checker 的 I-1.13（扫描重建与遍历方向的索引相同）在新池新建文件、覆盖写几版、unlink 之后的镜像上成立；
//! 必红世界 ① 改掉遍历方向中央映射叶里一条条目的落点、② 抹掉一个码 1 单元头的对象 key，各自判违例且差异落在题面写的那张表；
//! 删除中的对象只对它自己两侧都不比（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5）：第一个文件删一半时
//! 别的文件照比、成立，改坏别的文件的一条映射照样红。
//! 读法 Z 加写序（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K6，用户 2026-10-16 定）与两侧按 size 滤（K7 T-5）：
//! 覆盖写变短留下的单元、先变短再变长留在洞里的旧单元、截断中间态 size 之外还挂着的单元，三张镜像上都成立。
//! 必红世界 ③（往重建模块里加一行调走树函数的代码）由 `crates/singlefs-checker/src/rebuild.rs` 末尾的白名单单测判。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_truncation;

use common::{
    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
};
use common_deletion_in_progress::{
    change_the_inode_record, content_of_three_data_units,
    image_after_the_intent_of_a_three_unit_file, pool_with_a_three_unit_file, pool_with_new_inodes,
    publish_new_inodes_and_advance_the_pool, publish_the_intent,
    publish_the_next_step_in_batches_of_one, unlink_to_the_end, INSTANCE_OF_THE_PUBLISHES,
    WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
};
use common_pointer_head_corruption::{
    newest_root, node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
    unit_bytes_at,
};
use common_truncation::{
    batches_of_one, change_the_size_of_the_first_file_and_advance, payload_capacity,
    publish_the_next_truncation_step_and_advance, truncation_entry_of,
    upper_leaf_tag_of_the_first_file,
};
use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::InodeNumber;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::pointer::LocationEntry;
use singlefs_core::records::{mapping_key_for_data, INODE_RECORD_TRUNCATION_FIELD_OFFSET};
use singlefs_core::transaction::{
    publish_sequential_write_of_an_inode, PoolWriter, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::seal_header_checksum;
use singlefs_harness::memory_pool::MemoryPool;

/// 中央映射树节点（码 2，key 宽 27）：载荷 CRC 在 76 + 2k、明文头到 86 + 2k（D18（块里携带什么信息） 已定项 18 的偏移表）；
/// 叶条目 55 = key 27 + 位置条目 14 × 2，位置条目 = 设备身份 4 + 槽号 6 + 整单元校验和 4（D19（块指针的结构与宽度预算） 已定项 6 / 10）。
const MAPPING_KEY_WIDTH: usize = 27;
const MAPPING_ENTRY_BYTES: usize = 55;
const FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY: usize = MAPPING_KEY_WIDTH + 4;
/// 码 1 单元头里对象 key 那一段（五元组：类标签 42、树 43、对象 51、对象出生代 59、锚点偏移 67，D18（块里携带什么信息） 已定项 7）。
const DATA_UNIT_OBJECT_KEY: std::ops::Range<usize> = 42..75;
/// 单元头所在的第一扇区（内存盘按 512 字节的扇区写）。
const HEADER_SECTOR_BYTES: usize = 512;
/// 码 1 数据单元在中央映射 key 里的类标签。
const MAPPING_KEY_CLASS_OF_A_DATA_UNIT: u8 = 1;
/// inode 记录里截断字段的宽：T（8 字节）与 N_w（8 字节），起点 [`INODE_RECORD_TRUNCATION_FIELD_OFFSET`]。
const INODE_RECORD_TRUNCATION_FIELD_BYTES: usize = 16;
/// 换上去的那个槽离原槽多远：单元区里新池新建文件之后没写过的一格（一个 32 KiB 单元占两槽，挪两槽正好落在下一个单元的位置上）。
const SLOTS_THE_MAPPING_LOCATION_IS_MOVED_BY: u64 = 2;

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

/// 一个和第一个文件一样长、内容不同的版本（第 `seed` 版）：覆盖写不改长度，上一版的单元全由新版本盖住。
fn content_of_the_same_length(seed: usize) -> Vec<u8> {
    file_content()
        .iter()
        .map(|byte| byte.wrapping_add(u8::try_from(seed).expect("版本序号小于 256")))
        .collect()
}

/// 码 2 节点改过之后重封：先算载荷 CRC（罩 [86 + 2k, 末尾)、住 76 + 2k），再算头校验和（罩 [0, 86 + 2k)）。
fn reseal_the_index_node(bytes: &mut [u8]) {
    let key_width = usize::from(bytes[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum_offset = 76 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_checksum_offset..payload_checksum_offset + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(bytes, header_end);
}

/// 必红世界 ①：最新根的中央映射树（新池新建文件之后是根兼叶）里第一个文件那个数据单元的条目，第一条位置条目的槽号挪到别的槽；
/// 映射叶重封、新的整单元校验和沿引用链补到根槽。数据单元与 extent 树都不动。交回原来那个槽。
fn move_the_first_location_of_the_data_unit_mapping_entry(image: &mut MemoryPool) -> u64 {
    let mapping_root = newest_root(image).mapping_root;
    let leaf_slot = the_slot_of(&mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(
        usize::from(leaf[51]),
        MAPPING_KEY_WIDTH,
        "中央映射树的根兼叶 key 宽 27"
    );
    assert_eq!(leaf[50], 0, "新池新建文件之后中央映射树只有一层：根就是叶");
    let entries_start = 115 + 2 * MAPPING_KEY_WIDTH;
    let entry_count = usize::from(u16::from_le_bytes(
        leaf[82 + 2 * MAPPING_KEY_WIDTH..84 + 2 * MAPPING_KEY_WIDTH]
            .try_into()
            .expect("2 字节"),
    ));
    let data_unit_entry_start = (0..entry_count)
        .map(|index| entries_start + index * MAPPING_ENTRY_BYTES)
        .find(|start| leaf[*start] == MAPPING_KEY_CLASS_OF_A_DATA_UNIT)
        .expect("中央映射里有第一个文件那个数据单元的条目");
    let slot_field_start = data_unit_entry_start + FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY;
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&leaf[slot_field_start..slot_field_start + 6]);
    let original_slot = u64::from_le_bytes(slot_bytes);
    let moved_slot = original_slot + SLOTS_THE_MAPPING_LOCATION_IS_MOVED_BY;
    leaf[slot_field_start..slot_field_start + 6].copy_from_slice(&moved_slot.to_le_bytes()[..6]);
    reseal_the_index_node(&mut leaf);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
    original_slot
}

/// 第一个文件删一半的那张镜像上，映射条目的落点挪多远：单元区里那几次发布都没写到的一格（挪两槽会落在 inode 2 的下一个单元上，
/// 先红 I-5.1、I-1.13 报不适用）。
const SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY: u64 = 4096;

/// 删除中的对象之外那一形（判决 m3-multi-file-unlink-r1 第三节 M5 的必红世界）：最新根的中央映射树（根兼叶）里 key 是 `mapping_key` 的那条
/// 条目，第一条位置条目的槽号挪 [`SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY`] 槽；映射叶重封、新的整单元校验和沿引用链补到根槽。
/// 交回原来那个槽。
fn move_the_first_location_of_the_mapping_entry_keyed(
    image: &mut MemoryPool,
    mapping_key: &[u8],
) -> u64 {
    let mapping_root = newest_root(image).mapping_root;
    let leaf_slot = the_slot_of(&mapping_root);
    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
    assert_eq!(leaf[50], 0, "这张镜像上中央映射树只有一层：根就是叶");
    let entries_start = 115 + 2 * MAPPING_KEY_WIDTH;
    let entry_count = usize::from(u16::from_le_bytes(
        leaf[82 + 2 * MAPPING_KEY_WIDTH..84 + 2 * MAPPING_KEY_WIDTH]
            .try_into()
            .expect("2 字节"),
    ));
    let entry_start = (0..entry_count)
        .map(|index| entries_start + index * MAPPING_ENTRY_BYTES)
        .find(|start| leaf[*start..*start + MAPPING_KEY_WIDTH] == *mapping_key)
        .expect("中央映射里有这把 key 的条目");
    let slot_field_start = entry_start + FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY;
    let mut slot_bytes = [0u8; 8];
    slot_bytes[..6].copy_from_slice(&leaf[slot_field_start..slot_field_start + 6]);
    let original_slot = u64::from_le_bytes(slot_bytes);
    let moved_slot = original_slot + SLOTS_A_MAPPING_LOCATION_OF_ANOTHER_FILE_IS_MOVED_BY;
    leaf[slot_field_start..slot_field_start + 6].copy_from_slice(&moved_slot.to_le_bytes()[..6]);
    reseal_the_index_node(&mut leaf);
    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
    original_slot
}

/// 攻方 F 段那一形（`research/prompts/m3-multi-file-unlink-r1-opus-output.md` U4-甲）：第一个文件顺序写成三个单元，建 inode 2、按 inode 号写它
/// 两个单元，再删第一个文件：发完意图与第一批（批大小 1）。这一版里 inode 1 删除中（墓碑压着、记录还在、剩两个单元），inode 2 活着。
fn pool_with_the_first_file_half_deleted_beside_inode_two(tag: &str) -> BuiltPool {
    let mut pool = pool_with_a_three_unit_file(tag);
    publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let content_of_inode_two: Vec<u8> = (0..2 * payload_capacity() - 5)
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

/// 必红世界 ②：第一个文件那个数据单元两份副本的头里，对象 key（五元组）清零、不重封——头校验和随之对不上，扫描重建判它损坏、不认领。
/// 只抹一份时另一份照样扫得到、重建出的三张表不变，那是降级镜像，不是这个世界。
fn zero_the_object_key_of_both_copies_of_the_data_unit(image: &mut MemoryPool, pool: &BuiltPool) {
    for location in pool.output.data_pointers[0].locations {
        zero_the_object_key_of_the_copy_at(image, location);
    }
}

/// 一份码 1 副本头里的对象 key 清零、不重封：内存盘按扇区写，读回头所在那一扇区、改、整扇区写回。
fn zero_the_object_key_of_the_copy_at(image: &mut MemoryPool, location: LocationEntry) {
    let device = image
        .devices
        .get_mut(&location.device)
        .expect("位置条目指的盘在池里");
    let sector_start = location.slot.to_device_offset();
    let mut first_sector = device.read(sector_start, HEADER_SECTOR_BYTES);
    first_sector[DATA_UNIT_OBJECT_KEY].fill(0);
    device.write(sector_start, &first_sector);
}

#[test]
fn the_scan_rebuild_agrees_with_the_walked_index_after_the_new_pool_file_creation() {
    let pool = build_pool("scan-rebuild-new-pool");
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "新池新建文件之后：一个数据单元、一片 inode 叶容器，扫描重建的三张表与遍历方向逐项相同"
    );
}

#[test]
fn the_scan_rebuild_agrees_with_the_walked_index_after_three_overwrites_of_the_same_length() {
    let mut pool = build_pool("scan-rebuild-overwrites");
    for seed in 1..=3 {
        overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &content_of_the_same_length(seed),
        );
    }
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "同一个文件覆盖写三版（长度不变）：盘上四版数据单元与四版 inode 叶容器都过已发布谓词，按 C113 定案 P3 择出最后一版，与遍历方向逐项相同"
    );
}

#[test]
fn the_scan_rebuild_agrees_with_the_walked_index_after_an_unlink_to_the_end() {
    let mut pool = pool_with_new_inodes("scan-rebuild-unlink", 2);
    unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER);
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "再建两个空 inode 之后把第一个文件 unlink 到底：它的记录删掉、墓碑留下，盘上那个数据单元由墓碑判死（D11 已定项 4），叶容器里还有另两条记录、不退役，三张表与遍历方向逐项相同"
    );
}

#[test]
fn a_mapping_entry_whose_location_names_another_slot_reddens_the_scan_rebuild_in_the_central_mapping_only(
) {
    let pool = build_pool("scan-rebuild-moved-mapping-location");
    let mut image = pool.memory_pool();
    let original_slot = move_the_first_location_of_the_data_unit_mapping_entry(&mut image);
    let detail =
        violation_detail_of_the_scan_rebuild(&image, "映射叶里数据单元那条条目的落点挪了槽");
    assert!(
        detail.contains("「中央映射」差 1 处"),
        "差异落在中央映射那张表、只差那一条：{detail}"
    );
    assert!(
        detail.contains(&format!("盘 0 槽 {original_slot}"))
            && detail.contains(&format!(
                "盘 0 槽 {}",
                original_slot + SLOTS_THE_MAPPING_LOCATION_IS_MOVED_BY
            )),
        "说明里写出两边的落点（扫描重建扫到的原槽、遍历方向映射里的新槽）：{detail}"
    );
    assert!(
        !detail.contains("「每个 inode 的数据单元集合」") && !detail.contains("「inode 记录集合」"),
        "extent 树与 inode 树都没动，另两张表不该有差异：{detail}"
    );
}

#[test]
fn a_data_unit_whose_object_key_is_zeroed_on_both_copies_reddens_the_scan_rebuild_in_the_data_unit_table(
) {
    let pool = build_pool("scan-rebuild-zeroed-object-key");
    let mut image = pool.memory_pool();
    zero_the_object_key_of_both_copies_of_the_data_unit(&mut image, &pool);
    let detail = violation_detail_of_the_scan_rebuild(&image, "数据单元两份的对象 key 清零");
    assert!(
        detail.contains("「每个 inode 的数据单元集合」差 1 处")
            && detail.contains("（inode 1、偏移 0）：扫描重建 没有这一条"),
        "那个单元判损坏、不认领：数据单元集合里 (inode 1, 偏移 0) 只有遍历方向那一边有：{detail}"
    );
    assert!(
        !detail.contains("「inode 记录集合」"),
        "inode 叶容器没动，inode 记录集合不该有差异：{detail}"
    );
}

/// 意图之后墓碑压着、记录还在（删除中）：只对那一个对象两侧都不比（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md`
/// 第三节 M5，此前整张报不适用），这张镜像上只有它一个文件，两侧去掉它之后三张表相同、成立。
#[test]
fn a_deletion_in_progress_leaves_only_that_object_out_and_the_scan_rebuild_holds() {
    let image = image_after_the_intent_of_a_three_unit_file("scan-rebuild-deletion-in-progress");
    assert_eq!(
        verdict_of_the_scan_rebuild(&image),
        InvariantVerdict::Holds,
        "删除中的对象两侧都不比，别的没有：成立"
    );
}

/// 第一个文件删一半（攻方 F 段）：扫描重建只对删除中的 inode 1 不比，inode 2 的记录、两个数据单元与它们的映射照比、两侧相同，成立
/// （判决 m3-multi-file-unlink-r1 第三节 M5；此前一个对象删除中就整张报不适用，别的文件在整段删除期间失去这一道）。
#[test]
fn while_the_first_file_is_half_deleted_the_scan_rebuild_still_judges_another_file_and_holds() {
    let pool = pool_with_the_first_file_half_deleted_beside_inode_two(
        "scan-rebuild-half-deleted-beside-inode-two",
    );
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "inode 1 删除中、inode 2 带两个单元：只对 inode 1 不比，inode 2 照比、成立"
    );
}

/// M5 必红世界：同一张镜像（第一个文件删一半）上把 inode 2 第一个数据单元那条映射条目的落点挪两槽：inode 2 照比，I-1.13 红在中央映射那张表、
/// 只差那一条（此前这一版整张报不适用，这一处改坏看不出来）。
#[test]
fn moving_a_mapping_location_of_another_file_while_the_first_file_is_half_deleted_reddens_the_scan_rebuild(
) {
    let pool = pool_with_the_first_file_half_deleted_beside_inode_two(
        "scan-rebuild-half-deleted-moved-mapping-of-inode-two",
    );
    let first_data_pointer_of_inode_two = pool
        .output
        .file_other_than_the_first(InodeNumber(2))
        .expect("inode 2 带数据单元")
        .data_pointers[0];
    let mut image = pool.memory_pool();
    let original_slot = move_the_first_location_of_the_mapping_entry_keyed(
        &mut image,
        &mapping_key_for_data(
            first_data_pointer_of_inode_two.head,
            first_data_pointer_of_inode_two.write_order,
        ),
    );
    let detail = violation_detail_of_the_scan_rebuild(
        &image,
        "第一个文件删一半时 inode 2 那条映射条目的落点挪了槽",
    );
    assert!(
        detail.contains("「中央映射」差 1 处")
            && detail.contains(&format!("盘 0 槽 {original_slot}")),
        "差异落在中央映射那张表、只差 inode 2 那一条：{detail}"
    );
}

/// 三个单元的文件覆盖写成一个单元：后两个单元放掉而还在盘上，没有新版本也没有墓碑。读法 Z 把 size 之外的不认领（判决 K6），
/// 遍历方向的 extent 树里也没有它们：三张表相同、成立（此前报不适用，C827 那一格）。
#[test]
fn a_data_unit_left_beyond_the_size_by_a_shorter_overwrite_is_not_claimed_and_the_scan_rebuild_holds(
) {
    let mut pool = build_pool("scan-rebuild-shorter-overwrite");
    sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_three_data_units(),
    );
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &file_content());
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "三个单元的文件覆盖写成一个单元：后两个单元在 size 之外，扫描重建不认领，与遍历方向逐项相同"
    );
}

/// 三个单元的文件覆盖写成一个单元（3000 字节）、再只改 size 变长到 3 个净荷减 1（真写路径：单元 0 补零重写到满、之后是洞，标签 4）：
/// 覆盖写放掉的旧单元 1、2 还在盘上、落进新的 size 之内，出生 txg 比重写过的单元 0 小，读法 Z 加写序判死、不认领；遍历方向那里是洞，
/// 三张表相同、成立（攻方 W5 打中 1 那一形，只改 size 不加写序时扫描重建把它们认回来、I-1.13 违例）。
#[test]
fn old_data_units_left_in_the_hole_after_a_shorter_overwrite_and_an_extension_are_dead_by_the_write_order(
) {
    let mut pool = build_pool("scan-rebuild-shorter-overwrite-then-extension");
    sequential_write_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_three_data_units(),
    );
    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &file_content());
    let extended =
        change_the_size_of_the_first_file_and_advance(&mut pool, 3 * payload_capacity() - 1);
    assert_eq!(
        (
            extended.data_pointers.len(),
            upper_leaf_tag_of_the_first_file(&extended)
        ),
        (1, 4),
        "变长之后只有单元 0、之后到 size 是洞（标签 4）"
    );
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "洞里放掉的旧单元由写序判死，扫描重建不认领，与遍历方向逐项相同"
    );
}

/// 三个单元截到 100、批大小 1 的意图那一版（判决 K2「先落目标」的中间态）：size 之外还挂着两个单元、它们的映射条目也还在。
/// 遍历方向不收 size 之外的单元与它们的映射 key，扫描重建照读法 Z 不认领，两侧各写各的、三张表相同、成立（判决 K7 T-5）。
#[test]
fn the_intent_version_of_a_truncation_in_batches_of_one_holds_the_scan_rebuild() {
    let mut pool = pool_with_a_three_unit_file("scan-rebuild-truncation-intent");
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, 100);
    assert!(
        truncation_entry_of(&intent).is_some() && intent.data_pointers.len() == 3,
        "意图那一版：条目挂着、三个单元都在"
    );
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "截断中间态：size 之外的两个单元两侧都不收"
    );
    publish_the_next_truncation_step_and_advance(&mut pool, batches_of_one());
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "那一批之后：size 之外还挂着一个，两侧都不收"
    );
}

/// 三个单元（2.5 个净荷）截到恰好一个净荷的意图那一版：单元 0 原来就满、不重写，三个单元出生 txg 相同，size 之外的单元 1、2 不由写序判死，
/// 只由读法 Z 的 size 那一条不认领；遍历方向也不收它们。三张表相同、成立。
#[test]
fn the_intent_version_of_a_truncation_at_a_data_unit_boundary_holds_the_scan_rebuild() {
    let mut pool = pool_with_a_three_unit_file("scan-rebuild-truncation-at-a-boundary");
    let before = pool.output.clone();
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, payload_capacity());
    assert!(
        truncation_entry_of(&intent).is_some()
            && intent.data_pointers.len() == 3
            && intent.data_pointers[0] == before.data_pointers[0],
        "意图那一版：条目挂着、三个单元都在、单元 0 没重写"
    );
    assert_eq!(
        verdict_of_the_scan_rebuild(&pool.memory_pool()),
        InvariantVerdict::Holds,
        "截在单元边界上的中间态：size 之外的两个单元两侧都不收"
    );
}

/// 同上那一版，再把第一个文件记录里的截断字段 (T, N_w) 清零：单元 0 不重写、三个单元出生 txg 相同，写序那一条判不死 size 之外的单元 1、2；
/// 截断字段清零，读法 Z 第四条（截断字段之外、早于 T）也不判它们；只剩 size 那一条不认领，遍历方向也不收它们，三张表相同、成立。
/// 读法 Z 的 size 那一条在 CPU 一侧的取样点：写路径写出的意图那一版带着截断字段，第四条先判死单元 1、2，那一形上改掉 size 那一条照样成立。
/// 判别力（`crates/mutations.tsv`「扫描重建 size 之外的单元照认领」那一行）：size 那一条不判，扫描重建认领单元 1、2，「中央映射」差 2 处，这条红。
#[test]
fn the_intent_version_of_a_truncation_at_a_data_unit_boundary_whose_truncation_field_is_cleared_holds_the_scan_rebuild(
) {
    let mut pool =
        pool_with_a_three_unit_file("scan-rebuild-truncation-at-a-boundary-field-cleared");
    let before = pool.output.clone();
    let intent = change_the_size_of_the_first_file_and_advance(&mut pool, payload_capacity());
    assert!(
        truncation_entry_of(&intent).is_some()
            && intent.data_pointers.len() == 3
            && intent.data_pointers[0] == before.data_pointers[0],
        "意图那一版：条目挂着、三个单元都在、单元 0 没重写"
    );
    let mut image = pool.memory_pool();
    let truncation_field_start =
        usize::try_from(INODE_RECORD_TRUNCATION_FIELD_OFFSET).expect("120 装得进 usize");
    let mut truncation_field_the_write_path_wrote = [0u8; INODE_RECORD_TRUNCATION_FIELD_BYTES];
    change_the_inode_record(&mut image, FIRST_INODE_NUMBER, |record| {
        let field = &mut record
            [truncation_field_start..truncation_field_start + INODE_RECORD_TRUNCATION_FIELD_BYTES];
        truncation_field_the_write_path_wrote.copy_from_slice(field);
        field.fill(0);
    });
    assert_ne!(
        truncation_field_the_write_path_wrote, [0u8; INODE_RECORD_TRUNCATION_FIELD_BYTES],
        "写路径写出的意图那一版带着截断字段（清零之前 T 与 N_w 不全为 0）"
    );
    assert_eq!(
        verdict_of_the_scan_rebuild(&image),
        InvariantVerdict::Holds,
        "截断字段清零之后只剩 size 那一条：size 之外的两个单元两侧都不收"
    );
}

#[test]
fn zeroing_the_object_key_of_one_copy_leaves_the_scan_rebuild_holding() {
    let pool = build_pool("scan-rebuild-one-copy-zeroed");
    let mut image = pool.memory_pool();
    zero_the_object_key_of_the_copy_at(&mut image, pool.output.data_pointers[0].locations[0]);
    assert_eq!(
        verdict_of_the_scan_rebuild(&image),
        InvariantVerdict::Holds,
        "只抹一份：另一份照样扫得到，两边各只留读得出的落点再比，三张表相同（降级不判红）"
    );
}
