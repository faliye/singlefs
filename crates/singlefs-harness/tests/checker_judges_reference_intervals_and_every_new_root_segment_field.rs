//! 池级 checker 里程碑三第 8 批的几处判法，每条一份镜像（干净的，或改一处之后重封、沿引用链补到根槽的）：
//! - I-3.5（引用区间的精确性）的出现规则：诞生代号下界 (a)、按树继承下界（TF）、再次出现也判（β）、候选集有洞（gap）、
//!   实例表行接得上就照判（gap-row）、接不上时只放过用户可见树里的单元（N3）——`research/prompts/m3-birth-floor-r3-main-verification.md`，
//!   用户 2026-10-03 定丙；措辞是 `.claude/kb/invariants.md` I-3.5 那一行；
//! - I-9.25（树表条目的 previous_snapshot_txg 取值）：树表被几版共享时每一版拿自己的 txg 各判一次（回到 kb 字面）；
//! - I-8.10（记录新根段与同代根相同）：七段里新加的实例表指针、分配记录树根指针、卸载记号三段，与写行那次发布的记录（RR3 的观察者）；
//! - 记录新根段卸载记号位 0 之外有位为 1：池级 checker 与恢复收下同一批记录（m3-root-ring-code-r1 判决 RR1）。
mod common;

use common::{
    build_pool, format_pool, parameters, publish_overwrite_in_process,
    rewrite_journal_record_and_rechain_the_records_after_it, tear_the_root_slot_written_for_txg,
    BuiltPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::{
    InvariantVerdict, LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, REFERENCE_INTERVALS_ARE_EXACT,
    TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::journal::{back_chain_of, record_offset, JournalRecord};
use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{TREE_KIND_ACCOUNTING, TREE_KIND_EXTENT, TREE_KIND_INODE};
use singlefs_core::recovery::{
    choose_system_configuration, readable_roots, scan_journal, PoolReader,
};
use singlefs_core::root_record::UnmountMarker;
use singlefs_core::transaction::{
    publish_without_units, PoolWriter, TransactionOutput, ZeroUnitPublishPlan,
};
use singlefs_core::unit::{
    unit_filesystem_identifier, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
    UNIT_MAGIC,
};
use singlefs_format::{
    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES,
};
use singlefs_harness::memory_pool::MemoryPool;

const SLOT: u64 = 16384;
const DEVICES: [u32; 2] = [0, 1];
/// 单元区第一个槽（`common::build_pool` 那份 4 GiB 两盘池：mkfs 的实例表单元就落在这里）。
const FIRST_UNIT_SLOT: u64 = 50176;
/// 找单元时每个槽读开头一个扇区看单元头（内存池的读按扇区对齐，`MemoryPool` 的 `PoolReader::read`）。
const SECTOR_BYTES_READ_FOR_A_UNIT_HEADER: usize = 512;
/// 找单元时往后扫几个槽：这几份镜像上写过的单元都在这一段里（最长那段历史是回退那一份，十来次发布）。
const UNIT_SLOTS_SCANNED: u64 = 2048;
/// 根记录的自证校验和在 138 起 32 字节，罩整个 512 字节的根槽（D22（单元原子性怎么合成） 已定项 7）。
const ROOT_SLOT_BYTES: usize = 512;
const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
/// journal 记录头校验和在 46 起 32 字节，罩整条 4096（D23（journal 的角色与格式） 已定项 13）。
const JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET: usize = 46;
/// 新根段末尾卸载记号那 4 字节在记录里的偏移：新根段 99 起，前六段 360 字节之后（D23（journal 的角色与格式） 已定项 15）。
const JOURNAL_RECORD_UNMOUNT_MARKER_OFFSET: usize = 99 + 360;
/// 码 2 节点：key 宽在 51，条目数在 82 + 2k；树表单元（k = 8）的条目区从 131 起、每条 200（D8（核心索引结构） 已定项 8）。
const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
const TREE_TABLE_ENTRY_START: usize = 131;
const TREE_TABLE_ENTRY_BYTES_IN_THE_UNIT: usize = 200;
const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
const TREE_TABLE_ENTRY_ROOT_POINTER: std::ops::Range<usize> = 14..100;
const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 100;
/// 码 2 / 码 3 指针头里出生 txg 的偏移（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）。
const POINTER_BIRTH_TXG_OFFSET: usize = 42;
/// 单元头里诞生代号的偏移（D18（块里携带什么信息） 已定项 7 / 已定项 11）：码 1 在 75，码 3 在 73，码 2 在 52 + 2k。
const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;
/// 恢复择根之上一次发布都不施加时 I-4.1 报的不适用理由。
const NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT: &str =
    "所选根之上一次发布都不施加：重放之后那一版就是所选根，它可不可遍历归 I-7.2";

fn read(image: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(
        image,
        DeviceIdentity(device),
        DeviceOffsetInBytes(offset),
        length,
    )
    .expect("这几份镜像上读的都是盘内的落点")
}

fn write(image: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    image
        .devices
        .get_mut(&DeviceIdentity(device))
        .expect("这几份镜像都是两块盘")
        .write(DeviceOffsetInBytes(offset), bytes);
}

fn verdict(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("池级 checker 的清单里有这一条")
        .1
}

/// 这一条判违例，第一处违例的说明里有 `expected_in_the_detail`。
fn assert_violated_mentioning(image: &MemoryPool, invariant: &str, expected_in_the_detail: &str) {
    match verdict(image, invariant) {
        InvariantVerdict::Violated(detail) => assert!(
            detail.contains(expected_in_the_detail),
            "{invariant} 判违例，说明里该有「{expected_in_the_detail}」：{detail}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("{invariant} 应当判违例（说明里有「{expected_in_the_detail}」），实际 {other:?}")
        }
    }
}

/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
fn reseal_unit(bytes: &mut [u8]) {
    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let (header_end, payload_crc_offset) = match bytes[6] {
        UNIT_CLASS_DATA => (105usize, 101usize),
        UNIT_CLASS_PACKED => (107, 89),
        UNIT_CLASS_INDEX_NODE => (86 + 2 * key_width, 76 + 2 * key_width),
        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
    };
    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
    singlefs_core::unit::seal_header_checksum(bytes, header_end);
}

fn birth_txg_offset_in_the_unit_header(bytes: &[u8]) -> usize {
    match bytes[6] {
        UNIT_CLASS_DATA => DATA_UNIT_BIRTH_TXG_OFFSET,
        UNIT_CLASS_PACKED => PACKED_UNIT_BIRTH_TXG_OFFSET,
        UNIT_CLASS_INDEX_NODE => {
            INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS
                + 2 * usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET])
        }
        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
    }
}

fn birth_txg_in_the_unit_header(bytes: &[u8]) -> u64 {
    let offset = birth_txg_offset_in_the_unit_header(bytes);
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn set_birth_txg_in_the_unit_header(bytes: &mut [u8], birth_txg: u64) {
    let offset = birth_txg_offset_in_the_unit_header(bytes);
    bytes[offset..offset + 8].copy_from_slice(&birth_txg.to_le_bytes());
}

/// 盘 0 单元区开头那一段里写过的每个单元：(起点槽, 字节数)。两盘同槽（D2（RAID 条带策略） 已定项 10），读一块盘就够。
/// 迭代上界是扫的槽数；码 1 / 码 3 的单元占两槽，第二槽不当成单元头读。
fn units_on_disk(image: &MemoryPool) -> Vec<(u64, usize)> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
    let mut units = Vec::new();
    let mut slot = FIRST_UNIT_SLOT;
    while slot < FIRST_UNIT_SLOT + UNIT_SLOTS_SCANNED {
        let header = read(image, 0, slot * SLOT, SECTOR_BYTES_READ_FOR_A_UNIT_HEADER);
        if header[..4] != UNIT_MAGIC {
            slot += 1;
            continue;
        }
        if header[6] == UNIT_CLASS_INDEX_NODE {
            units.push((slot, node_bytes));
            slot += 1;
        } else {
            units.push((slot, data_unit_bytes));
            slot += 2;
        }
    }
    units
}

/// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，每区 8 槽、槽距 4096。
fn root_slots() -> Vec<(u32, u64)> {
    (0..3u64)
        .flat_map(|region| {
            (0..8u64).map(move |slot| {
                (
                    [0u32, 1, 0][usize::try_from(region).expect("区域号")],
                    64 * SLOT + region * 3 * (1 << 20) + slot * 4096,
                )
            })
        })
        .collect()
}

/// 一条位置条目：设备 4 + 槽号 6 + 整单元校验和 4。
fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
    [
        device.to_le_bytes().to_vec(),
        slot.to_le_bytes()[..6].to_vec(),
        checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
    let mut changed = false;
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == *needle {
            haystack[index..index + needle.len()].copy_from_slice(replacement);
            changed = true;
            index += needle.len();
        } else {
            index += 1;
        }
    }
    changed
}

/// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上（单元里的、根槽里的），被补的单元重封、
/// 再往上补，直到根槽；最后把每条根照抄进同 (实例, txg) 的 journal 记录的新根段（`common` 的那一份），I-8.10 不跟着红。
/// 迭代上界：每补一个单元，它自己的新校验和进待补的表一次；引用链是有向无环的（子先于父写），补到根槽就停。
fn propagate(image: &mut MemoryPool, units: &[(u64, usize)], mut changes: Vec<(u64, u32, u32)>) {
    while let Some((slot, old, new)) = changes.pop() {
        if old == new {
            continue;
        }
        for (container, length) in units.iter().copied() {
            if container == slot {
                continue;
            }
            let before = read(image, 0, container * SLOT, length);
            let mut bytes = before.clone();
            let mut touched = false;
            for device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(device, slot, old),
                    &location_pattern(device, slot, new),
                );
            }
            if touched {
                reseal_unit(&mut bytes);
                for device in DEVICES {
                    write(image, device, container * SLOT, &bytes);
                }
                changes.push((
                    container,
                    crc32_castagnoli(&before),
                    crc32_castagnoli(&bytes),
                ));
            }
        }
        for (device, offset) in root_slots() {
            let mut bytes = read(image, device, offset, ROOT_SLOT_BYTES);
            let mut touched = false;
            for location_device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(location_device, slot, old),
                    &location_pattern(location_device, slot, new),
                );
            }
            if touched {
                let digest = wide_checksum_with_field_zeroed(
                    &bytes,
                    ROOT_SLOT_BYTES,
                    ROOT_RECORD_CHECKSUM_OFFSET,
                );
                bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32]
                    .copy_from_slice(&digest);
                write(image, device, offset, &bytes);
            }
        }
    }
    common::copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// 改一个单元（两盘一起改）：改完重封它的两道校验和，新的整单元校验和沿引用链补到根槽。
fn rewrite_unit_and_propagate(image: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>)) {
    let units = units_on_disk(image);
    let length = units
        .iter()
        .find(|(start, _)| *start == slot)
        .unwrap_or_else(|| panic!("槽 {slot} 是盘上写过的一个单元的起点"))
        .1;
    let before = read(image, 0, slot * SLOT, length);
    let mut bytes = before.clone();
    change(&mut bytes);
    reseal_unit(&mut bytes);
    for device in DEVICES {
        write(image, device, slot * SLOT, &bytes);
    }
    propagate(
        image,
        &units,
        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
    );
}

/// 两盘同槽：一条指针两条位置条目落在同一个槽号上，按一个槽号改两盘。
fn slot_shared_by_both_copies(pointer: &NodePointer) -> u64 {
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "两盘同槽：这份镜像的改法按一个槽号改两盘"
    );
    pointer.locations[0].slot.0
}

fn tree_table_slot(version: &TransactionOutput) -> u64 {
    slot_shared_by_both_copies(&version.root.tree_table)
}

fn root_slot_of_the_tree(version: &TransactionOutput, kind: u16) -> u64 {
    let entry = version
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == kind)
        .unwrap_or_else(|| panic!("这一版的树表里有种类 {kind} 的那一条"));
    slot_shared_by_both_copies(&entry.root)
}

fn data_unit_slot(version: &TransactionOutput) -> u64 {
    let [pointer] = version.data_pointers.as_slice() else {
        panic!(
            "这几版文件都只有一个数据单元：{} 条数据指针",
            version.data_pointers.len()
        );
    };
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "两盘同槽：数据单元按一个槽号改两盘"
    );
    pointer.locations[0].slot.0
}

/// 一片树表单元里种类为 `kind` 的那一条条目的起点。
fn tree_table_entry_offset_of_kind(tree_table: &[u8], kind: u16) -> usize {
    let key_width = usize::from(tree_table[INDEX_NODE_KEY_WIDTH_OFFSET]);
    let entry_count_offset = 82 + 2 * key_width;
    let entry_count = usize::from(u16::from_le_bytes([
        tree_table[entry_count_offset],
        tree_table[entry_count_offset + 1],
    ]));
    (0..entry_count)
        .map(|index| TREE_TABLE_ENTRY_START + TREE_TABLE_ENTRY_BYTES_IN_THE_UNIT * index)
        .find(|offset| {
            let kind_offset = offset + TREE_TABLE_ENTRY_KIND_OFFSET;
            u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]) == kind
        })
        .unwrap_or_else(|| panic!("树表里有种类 {kind} 的那一条"))
}

/// 覆盖写那几版的内容：每版一个数据单元装得下，字节各不相同。
fn content_of_the_overwrite(seed: usize) -> Vec<u8> {
    (0..4100usize)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 新池新建文件 A (1, 3) 之后同一个进程里再覆盖写 `overwrites` 次（B (1, 4)、C (1, 5)……）：交回池与每一版（A 在最前）。
fn pool_through_overwrites(tag: &str, overwrites: usize) -> (BuiltPool, Vec<TransactionOutput>) {
    let mut pool = build_pool(tag);
    let mut versions = vec![pool.output.clone()];
    for overwrite in 0..overwrites {
        let previous = versions.last().expect("至少有 A").clone();
        let next = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of_the_overwrite(overwrite + 3),
            FIXED_WRITE_TIME_SECONDS + 60 * (u64::try_from(overwrite).expect("次数") + 1),
            InstanceGeneration(1),
        )
        .expect("覆盖写");
        pool.output = next.clone();
        versions.push(next);
    }
    (pool, versions)
}

/// 挂着的时候回退到 A 之后的干净镜像：A (1, 3)、B (1, 4) → 重开取号 2、写行、暖机两次、C (2, 8) → 一次向前发布 D (2, 9)，
/// 新根的 extent 树、inode 树取 A 的，树表、记账、映射另写（D23（journal 的角色与格式） 已定项 14；`checker_known_bad_images.rs` 同一段历史）。
struct ImageRolledBackToTheFirstVersion {
    image: MemoryPool,
    first: TransactionOutput,
    version_before_the_rollback: TransactionOutput,
    rollback_publish: TransactionOutput,
}

fn image_rolled_back_to_the_first_version(tag: &str) -> ImageRolledBackToTheFirstVersion {
    let (mut pool, versions) = pool_through_overwrites(tag, 1);
    let first = versions[0].clone();
    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices_reopened_for_the_second_instance)
        .expect("重开取号 2");
    pool.devices = Some(devices_reopened_for_the_second_instance);
    pool.allocator = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let third = publish_overwrite_in_process(
        &mut pool,
        &current,
        &content_of_the_overwrite(11),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(2),
    )
    .expect("覆盖写 C");
    let mut rollback_publish = third.clone();
    let rollback_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    roll_back_by_a_forward_publish(
        &rollback_parameters,
        devices,
        &mut pool.allocator,
        &mut rollback_publish,
        RollbackTarget {
            instance: first.root.instance,
            checkpoint_txg: first.root.checkpoint_txg,
        },
    )
    .expect("挂着的时候回退到 A");
    assert_eq!(
        rollback_publish.root.checkpoint_txg.0,
        third.root.checkpoint_txg.0 + 1,
        "回退那次发布紧接在 C 之后"
    );
    assert_eq!(
        root_slot_of_the_tree(&rollback_publish, TREE_KIND_EXTENT),
        root_slot_of_the_tree(&first, TREE_KIND_EXTENT),
        "回退那一版的 extent 树根就是 A 的"
    );
    ImageRolledBackToTheFirstVersion {
        image: pool.memory_pool(),
        first,
        version_before_the_rollback: third,
        rollback_publish,
    }
}

/// (a)：A (1, 3) → B (1, 4) 的干净镜像上 I-3.5 成立；B 新写的数据单元诞生代号改小到恰好等于 A 的 txg（J3-c），I-3.5 红，
/// 违例说的就是那个数据单元。「严格大于」写成「不小于」的这一条红。
#[test]
fn a_unit_new_in_an_overwrite_born_at_the_txg_of_the_version_before_reddens_the_reference_interval_invariant(
) {
    let (pool, versions) =
        pool_through_overwrites("reference-interval-born-at-the-previous-txg", 1);
    let (first, second) = (&versions[0], &versions[1]);
    let clean = pool.memory_pool();
    assert_eq!(
        verdict(&clean, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "A → B 的干净镜像上每个新出现的单元都生在前一版之后"
    );
    let mut image = clean.clone();
    let data_unit = data_unit_slot(second);
    rewrite_unit_and_propagate(&mut image, data_unit, |bytes| {
        assert_eq!(
            birth_txg_in_the_unit_header(bytes),
            second.root.checkpoint_txg.0,
            "B 的数据单元生在 B 那一版"
        );
        set_birth_txg_in_the_unit_header(bytes, first.root.checkpoint_txg.0);
    });
    assert_violated_mentioning(
        &image,
        REFERENCE_INTERVALS_ARE_EXACT,
        &format!("槽 {data_unit}"),
    );
}

/// TF（按树继承下界）：挂着的时候回退到 A 之后，回退那一版 D 引用 A 的 extent 树与 inode 树（诞生代号都是 3），C 不引用它们；
/// 它们落在 D 的用户可见树里、根指针出生 txg 3 小于 C 的 txg、子树里诞生代号都不大于 3：合法回退，I-3.5 成立。去掉 (b) 这一条红。
#[test]
fn a_rollback_inheriting_the_user_visible_trees_of_its_target_whole_holds_the_reference_interval_invariant(
) {
    let rolled_back = image_rolled_back_to_the_first_version("reference-interval-rollback-holds");
    assert!(
        rolled_back.first.root.checkpoint_txg.0
            < rolled_back
                .version_before_the_rollback
                .root
                .checkpoint_txg
                .0,
        "回退目标比回退之前那一版老"
    );
    assert_eq!(
        verdict(&rolled_back.image, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "回退那一版整棵继承 A 的用户可见树：(b) 罩住它们"
    );
}

/// TF 的「严格小于」：A → B 的镜像上把 B 的数据单元、B 的 extent 树根两个单元头的诞生代号与 B 树表里 extent 树那条根指针的出生 txg
/// 一起改成 A 的 txg（3）：那棵树的根指针出生 txg 等于前一版的 txg，不算整棵继承下来，I-3.5 红。「严格小于」写成「不大于」的这一条红。
#[test]
fn a_user_visible_tree_whose_root_pointer_was_born_at_the_txg_of_the_version_before_reddens_the_reference_interval_invariant(
) {
    let (pool, versions) =
        pool_through_overwrites("reference-interval-root-born-at-the-previous-txg", 1);
    let (first, second) = (&versions[0], &versions[1]);
    let previous_txg = first.root.checkpoint_txg.0;
    let mut image = pool.memory_pool();
    for unit in [
        data_unit_slot(second),
        root_slot_of_the_tree(second, TREE_KIND_EXTENT),
    ] {
        rewrite_unit_and_propagate(&mut image, unit, |bytes| {
            set_birth_txg_in_the_unit_header(bytes, previous_txg);
        });
    }
    rewrite_unit_and_propagate(&mut image, tree_table_slot(second), |bytes| {
        let pointer_birth_txg = tree_table_entry_offset_of_kind(bytes, TREE_KIND_EXTENT)
            + TREE_TABLE_ENTRY_ROOT_POINTER.start
            + POINTER_BIRTH_TXG_OFFSET;
        bytes[pointer_birth_txg..pointer_birth_txg + 8]
            .copy_from_slice(&previous_txg.to_le_bytes());
    });
    assert_violated_mentioning(&image, REFERENCE_INTERVALS_ARE_EXACT, "用户可见树 Some(");
}

/// TF 的「压得住」：回退到 A 之后，A 的数据单元头里的诞生代号改成 4（大于 extent 树根指针的出生 txg 3、不大于回退之前那一版的 txg）：
/// 那棵子树里有单元比根新，不算整棵继承下来，I-3.5 红，违例说的就是那个数据单元。去掉「子树里诞生代号都不大于根」那一半这一条红。
#[test]
fn a_unit_in_an_inherited_user_visible_tree_born_after_its_root_reddens_the_reference_interval_invariant(
) {
    let rolled_back =
        image_rolled_back_to_the_first_version("reference-interval-inherited-tree-not-held-down");
    let mut image = rolled_back.image.clone();
    let data_unit = data_unit_slot(&rolled_back.first);
    let birth_after_the_root = rolled_back.first.root.checkpoint_txg.0 + 1;
    assert!(
        birth_after_the_root
            < rolled_back
                .version_before_the_rollback
                .root
                .checkpoint_txg
                .0,
        "改成的诞生代号不大于回退之前那一版的 txg：(a) 罩不住它"
    );
    rewrite_unit_and_propagate(&mut image, data_unit, |bytes| {
        assert_eq!(
            birth_txg_in_the_unit_header(bytes),
            rolled_back.first.root.checkpoint_txg.0,
            "A 的数据单元生在 A 那一版"
        );
        set_birth_txg_in_the_unit_header(bytes, birth_after_the_root);
    });
    assert_violated_mentioning(
        &image,
        REFERENCE_INTERVALS_ARE_EXACT,
        &format!("槽 {data_unit}"),
    );
}

/// β（再次出现也判）：回退到 A 之后，把回退那一版 D 树表里记账树那条的根指针换成 A 树表里那一条的——A 的记账树根（诞生代号 3）
/// 在 A 里出现过、在 C 里没有、在 D 里再出现，它不在用户可见树里，I-3.5 红。只判第一次出现（以前哪一版引用过就不判）这一条红。
#[test]
fn a_fixed_point_tree_of_the_rollback_pointed_back_at_the_rollback_target_reddens_the_reference_interval_invariant(
) {
    let rolled_back =
        image_rolled_back_to_the_first_version("reference-interval-reappearing-accounting-root");
    let mut image = rolled_back.image.clone();
    let first_tree_table = read(
        &image,
        0,
        tree_table_slot(&rolled_back.first) * SLOT,
        usize::try_from(NODE_BYTES).expect("16384"),
    );
    let accounting_root_pointer_of_the_first: Vec<u8> = {
        let entry = tree_table_entry_offset_of_kind(&first_tree_table, TREE_KIND_ACCOUNTING);
        first_tree_table
            [entry + TREE_TABLE_ENTRY_ROOT_POINTER.start..entry + TREE_TABLE_ENTRY_ROOT_POINTER.end]
            .to_vec()
    };
    let accounting_root_of_the_first =
        root_slot_of_the_tree(&rolled_back.first, TREE_KIND_ACCOUNTING);
    assert_ne!(
        root_slot_of_the_tree(&rolled_back.rollback_publish, TREE_KIND_ACCOUNTING),
        accounting_root_of_the_first,
        "回退那一版本来另写了记账树"
    );
    rewrite_unit_and_propagate(
        &mut image,
        tree_table_slot(&rolled_back.rollback_publish),
        |bytes| {
            let entry = tree_table_entry_offset_of_kind(bytes, TREE_KIND_ACCOUNTING);
            bytes[entry + TREE_TABLE_ENTRY_ROOT_POINTER.start
                ..entry + TREE_TABLE_ENTRY_ROOT_POINTER.end]
                .copy_from_slice(&accounting_root_pointer_of_the_first);
        },
    );
    assert_violated_mentioning(
        &image,
        REFERENCE_INTERVALS_ARE_EXACT,
        &format!("槽 {accounting_root_of_the_first}（用户可见树 None）"),
    );
}

/// A (1, 3) → B (1, 4) → C (1, 5) 之后 B 的根槽改坏：候选集里 A 与 C 之间有洞，只有实例 1、实例表一行都没有，接不上。
fn image_with_a_hole_the_instance_table_rows_do_not_bridge(
    tag: &str,
    change_before_the_root_slot_is_torn: impl Fn(&mut MemoryPool, &[TransactionOutput]),
) -> MemoryPool {
    let (pool, versions) = pool_through_overwrites(tag, 2);
    let mut image = pool.memory_pool();
    change_before_the_root_slot_is_torn(&mut image, &versions);
    tear_the_root_slot_written_for_txg(&mut image, versions[1].root.checkpoint_txg.0);
    image
}

/// gap（缺口）：同一个改法——C 新写的数据单元诞生代号改小到 A 的 txg——在没有洞的镜像上红（与 B 相邻），在 B 的根槽读不出、
/// 实例表行接不上的那份上不判（N3 只放过用户可见树里的单元：盲区，`.claude/kb/invariants.md` I-3.5 射程），I-3.5 成立。
/// 去掉缺口那一条（洞上的用户可见树单元照判）这一条红。
#[test]
fn in_a_hole_the_instance_table_rows_do_not_bridge_a_user_visible_unit_born_before_the_hole_is_not_judged(
) {
    let lower_the_data_unit_of_the_third_version =
        |image: &mut MemoryPool, versions: &[TransactionOutput]| {
            let data_unit = data_unit_slot(&versions[2]);
            let first_txg = versions[0].root.checkpoint_txg.0;
            rewrite_unit_and_propagate(image, data_unit, |bytes| {
                set_birth_txg_in_the_unit_header(bytes, first_txg);
            });
        };
    let (pool, versions) =
        pool_through_overwrites("reference-interval-gap-user-visible-no-hole", 2);
    let mut without_the_hole = pool.memory_pool();
    lower_the_data_unit_of_the_third_version(&mut without_the_hole, &versions);
    assert_violated_mentioning(
        &without_the_hole,
        REFERENCE_INTERVALS_ARE_EXACT,
        &format!("槽 {}", data_unit_slot(&versions[2])),
    );
    let clean_with_the_hole = image_with_a_hole_the_instance_table_rows_do_not_bridge(
        "reference-interval-gap-clean",
        |_image, _versions| {},
    );
    assert_eq!(
        verdict(&clean_with_the_hole, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "B 的根槽读不出的干净镜像：A → C 那一对照 N3 判，树表、记账那几个单元都生在 A 之后"
    );
    let with_the_hole = image_with_a_hole_the_instance_table_rows_do_not_bridge(
        "reference-interval-gap-user-visible",
        lower_the_data_unit_of_the_third_version,
    );
    assert_eq!(
        verdict(&with_the_hole, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "洞上实例表行接不上：C 的数据单元落在用户可见树里，不判"
    );
}

/// N3：同一份有洞、实例表行接不上的镜像上，C 新写的树表单元（不在用户可见树里）诞生代号改小到 A 的 txg：照 (a) 判，I-3.5 红。
/// 洞上整对都不判这一条红。
#[test]
fn in_a_hole_the_instance_table_rows_do_not_bridge_a_fixed_point_unit_born_before_the_hole_reddens_the_reference_interval_invariant(
) {
    let tree_table_of_the_third_version = std::cell::Cell::new(0u64);
    let image = image_with_a_hole_the_instance_table_rows_do_not_bridge(
        "reference-interval-gap-fixed-point",
        |image, versions| {
            let tree_table = tree_table_slot(&versions[2]);
            tree_table_of_the_third_version.set(tree_table);
            let first_txg = versions[0].root.checkpoint_txg.0;
            rewrite_unit_and_propagate(image, tree_table, |bytes| {
                set_birth_txg_in_the_unit_header(bytes, first_txg);
            });
        },
    );
    assert_violated_mentioning(
        &image,
        REFERENCE_INTERVALS_ARE_EXACT,
        &format!(
            "槽 {}（用户可见树 None）",
            tree_table_of_the_third_version.get()
        ),
    );
}

/// gap-row：A (1, 3) 之后重开取号 2（写行、暖机）、覆盖写 D；实例 2 在 D 之前的几条根槽都改坏。候选集里 A 与 D 之间有洞，
/// 但 D 指着的实例表有一行 (1, 3)、D 的实例 2 紧接实例 1：接得上，这一对照常判。D 新写的数据单元诞生代号改小到 A 的 txg，I-3.5 红。
/// 去掉 gap-row（接得上也当接不上、只判用户可见树之外的）这一条红。
#[test]
fn a_hole_the_instance_table_rows_bridge_judges_the_user_visible_units_too() {
    let mut pool = build_pool("reference-interval-gap-row");
    let first = pool.output.clone();
    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices_reopened_for_the_second_instance)
        .expect("重开取号 2：写行、暖机");
    pool.devices = Some(devices_reopened_for_the_second_instance);
    pool.allocator = mounted.allocator;
    let current = mounted
        .current
        .into_file_version()
        .expect("A 之后重开，现行那一版带文件");
    let overwrite = publish_overwrite_in_process(
        &mut pool,
        &current,
        &content_of_the_overwrite(17),
        FIXED_WRITE_TIME_SECONDS + 120,
        InstanceGeneration(2),
    )
    .expect("实例 2 覆盖写 D");
    let txgs_of_the_second_instance_before_the_overwrite =
        first.root.checkpoint_txg.0 + 1..overwrite.root.checkpoint_txg.0;
    assert!(
        !txgs_of_the_second_instance_before_the_overwrite.is_empty(),
        "实例 2 在 D 之前写过行"
    );
    let tear_the_second_instance_before_the_overwrite = |image: &mut MemoryPool| {
        for txg in txgs_of_the_second_instance_before_the_overwrite.clone() {
            tear_the_root_slot_written_for_txg(image, txg);
        }
    };
    let mut clean = pool.memory_pool();
    tear_the_second_instance_before_the_overwrite(&mut clean);
    assert_eq!(
        verdict(&clean, REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "实例 2 写行与暖机的根槽读不出的干净镜像：A → D 那一对照常判，D 新写的都生在 A 之后"
    );
    let mut image = pool.memory_pool();
    let data_unit = data_unit_slot(&overwrite);
    rewrite_unit_and_propagate(&mut image, data_unit, |bytes| {
        set_birth_txg_in_the_unit_header(bytes, first.root.checkpoint_txg.0);
    });
    tear_the_second_instance_before_the_overwrite(&mut image);
    assert_violated_mentioning(
        &image,
        REFERENCE_INTERVALS_ARE_EXACT,
        &format!("槽 {data_unit}"),
    );
}

/// 新池新建文件 A (1, 3) 之后同一实例发一次零单元发布 (1, 4)：它照抄 A 的树表指针与映射根指针，与 A 共享那两棵。交回池、A 与那一版。
fn pool_with_a_zero_unit_publish_after_the_first_file(
    tag: &str,
) -> (
    BuiltPool,
    TransactionOutput,
    singlefs_core::transaction::VersionWithoutFilePublishOutput,
) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let publish_parameters = parameters();
    let zero_unit = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_without_units(
            &mut writer,
            &first.root,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(first.root.checkpoint_txg.0 + 1),
                counter: first.record.counter + 1,
                instance: first.root.instance,
                back_chain: back_chain_of(&first.record_bytes),
                rollback_floor: first.root.rollback_floor,
                tree_identifier_watermark: first.root.tree_identifier_watermark,
            },
        )
        .expect("A 之后一次零单元发布")
    };
    assert_eq!(
        (zero_unit.root.tree_table, zero_unit.root.mapping_root),
        (first.root.tree_table, first.root.mapping_root),
        "零单元发布与 A 共享 A 那片树表与映射根"
    );
    (pool, first, zero_unit)
}

/// 共享树表的那一版照样把共享的映射根并进它引用的单元：A 与它之后的零单元发布 (1, 4) 引用同一批单元，相邻两版之间一个新出现的都没有，
/// I-3.5 成立（新池新建文件那一对判到了）。共享树表时映射根不往下并，(1, 4) 先走、A 后走，A 的引用集合里少了映射树，
/// (A, (1, 4)) 那一对把映射树当成新出现、诞生代号 3 不大于 3，这一条红。
#[test]
fn a_zero_unit_publish_sharing_the_tree_table_and_the_mapping_root_holds_the_reference_interval_invariant(
) {
    let (pool, _first, _zero_unit) =
        pool_with_a_zero_unit_publish_after_the_first_file("reference-interval-shared-tree-table");
    assert_eq!(
        verdict(&pool.memory_pool(), REFERENCE_INTERVALS_ARE_EXACT),
        InvariantVerdict::Holds,
        "A 与零单元发布那一版引用同一批单元"
    );
}

/// I-9.25 回到 kb 字面：A (1, 3) 之后同一实例发一次零单元发布 (1, 4)，它与 A 共享 A 那片树表。树表里 inode 树那条的
/// previous_snapshot_txg 改成 4：(1, 4) 那一版成立，A 那一版（txg 3）不成立——共享树表的每一版各判一次，I-9.25 红。
/// 只在第一个走到它的那一版判（最新那条根先走、txg 最大）这一条不红。
#[test]
fn a_tree_table_shared_by_two_versions_is_judged_against_the_txg_of_each_version() {
    let (pool, first, zero_unit) = pool_with_a_zero_unit_publish_after_the_first_file(
        "previous-snapshot-txg-shared-tree-table",
    );
    let clean = pool.memory_pool();
    assert_eq!(
        verdict(&clean, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES),
        InvariantVerdict::Holds,
        "干净镜像上树表条目的 previous_snapshot_txg 都是 0"
    );
    let mut image = clean.clone();
    let later_txg = zero_unit.root.checkpoint_txg.0;
    rewrite_unit_and_propagate(&mut image, tree_table_slot(&first), |bytes| {
        let field = tree_table_entry_offset_of_kind(bytes, TREE_KIND_INODE)
            + TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET;
        assert_eq!(
            u64::from_le_bytes(bytes[field..field + 8].try_into().expect("8 字节")),
            0,
            "inode 树那条的 previous_snapshot_txg 本来是 0"
        );
        bytes[field..field + 8].copy_from_slice(&later_txg.to_le_bytes());
    });
    assert_violated_mentioning(
        &image,
        PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
        &format!("不大于这一版的 txg {}", first.root.checkpoint_txg.0),
    );
}

/// I-8.10 新加三段之一：A 那条记录（计数器 3，带末条标志）新根段里实例表指针第一条位置条目的校验和改一位、重封；A 的根照旧。
#[test]
fn a_last_record_whose_instance_table_pointer_differs_from_its_root_reddens_the_new_root_segment_invariant(
) {
    let pool = build_pool("new-root-segment-instance-table-pointer");
    let clean = pool.memory_pool();
    assert_eq!(
        verdict(
            &clean,
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH
        ),
        InvariantVerdict::Holds,
        "干净镜像上带末条标志的记录与同代根七段都相同"
    );
    let mut image = clean.clone();
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 3, |record| {
        record.new_instance_table.locations[0].unit_checksum ^= 1;
    });
    assert_violated_mentioning(
        &image,
        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
        "实例表指针",
    );
}

/// I-8.10 新加三段之二：A 那条记录新根段里分配记录树根指针（带文件的一版恒全零）换成一条非零指针、重封；A 的根照旧。
#[test]
fn a_last_record_whose_allocation_record_tree_root_pointer_differs_from_its_root_reddens_the_new_root_segment_invariant(
) {
    let pool = build_pool("new-root-segment-allocation-record-tree-root");
    let mut image = pool.memory_pool();
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 3, |record| {
        assert_eq!(
            record.new_allocation_record_tree_root,
            NodePointer::empty_root(),
            "带文件的一版，记录新根段里的分配记录树根指针恒全零"
        );
        record.new_allocation_record_tree_root = record.new_instance_table;
    });
    assert_violated_mentioning(
        &image,
        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
        "分配记录树根指针",
    );
}

/// I-8.10 新加三段之三：A 那条记录新根段里卸载记号 0 → 1（位 0 合法，读者照收）、重封；A 的根不带记号。
#[test]
fn a_last_record_whose_unmount_marker_differs_from_its_root_reddens_the_new_root_segment_invariant()
{
    let pool = build_pool("new-root-segment-unmount-marker");
    let mut image = pool.memory_pool();
    rewrite_journal_record_and_rechain_the_records_after_it(&mut image, 3, |record| {
        assert_eq!(
            record.new_unmount_marker,
            UnmountMarker::NotWrittenByTheUnmountSequence,
            "新池新建文件那条记录不带卸载记号"
        );
        record.new_unmount_marker = UnmountMarker::WrittenByTheUnmountSequence;
    });
    assert_violated_mentioning(
        &image,
        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
        "卸载记号",
    );
}

/// RR3 的观察者：只做过 mkfs 的池第一次可写挂载（取号 1、不写行）、第二次可写挂载（取号 2，写行：实例表换了、分配记录树根住根记录）。
/// 写行那条记录与它的根都在盘上，I-8.10 七段都比得到、成立。写行记录的实例表指针照抄上一版的、或分配记录树根写成空的，这一条红。
#[test]
fn the_row_publish_record_carries_every_new_root_segment_field_of_its_root() {
    let mut formatted = format_pool("new-root-segment-of-the-row-publish");
    let mut first_mount_devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first_mount_devices)
        .expect("第一次可写挂载：取号 1、不写行");
    drop(first_mount_devices);
    let mut second_mount_devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut second_mount_devices).expect("第二次可写挂载：取号 2、写行");
    formatted.devices = Some(second_mount_devices);
    let image = formatted.memory_pool();
    let system_configuration = choose_system_configuration(&image).expect("择得出系统配置");
    let roots = readable_roots(
        &image,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    );
    let row_root = roots
        .iter()
        .filter(|root| root.instance == InstanceGeneration(2))
        .min_by_key(|root| root.checkpoint_txg)
        .expect("实例 2 的写行那条根读得出");
    let root_before_the_row = roots
        .iter()
        .filter(|root| root.checkpoint_txg < row_root.checkpoint_txg)
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .expect("写行之前那一版的根读得出");
    assert_ne!(
        row_root.instance_table, root_before_the_row.instance_table,
        "写行那一版换了实例表"
    );
    assert_ne!(
        row_root.allocation_record_tree_root,
        NodePointer::empty_root(),
        "树表 0 条的一版写行之后，分配记录树根住根记录"
    );
    assert_eq!(
        verdict(
            &image,
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH
        ),
        InvariantVerdict::Holds,
        "写行那条记录的新根段七段与它的根逐字节相同"
    );
}

/// 改 A 那条记录（计数器 3）：新根段树表指针两条位置条目的校验和各翻一位（施加出来那一版走不完，I-4.1 看得见收没收），
/// 卸载记号那 4 字节写成 `field`，头校验和重封；两盘各一份。
fn set_the_unmount_marker_field_and_break_the_tree_table_pointer_of_the_first_file_record(
    image: &mut MemoryPool,
    field: u32,
) {
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    let offset = record_offset(3, JOURNAL_RING_DEFAULT_BYTES);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in DEVICES {
        let mut record = JournalRecord::parse(
            &read(image, device, offset.0, record_bytes),
            filesystem_identifier,
        )
        .expect("改之前 A 那条记录自证过");
        for location in &mut record.new_tree_table.locations {
            location.unit_checksum ^= 1;
        }
        let mut bytes = record.to_bytes();
        bytes[JOURNAL_RECORD_UNMOUNT_MARKER_OFFSET..JOURNAL_RECORD_UNMOUNT_MARKER_OFFSET + 4]
            .copy_from_slice(&field.to_le_bytes());
        let digest = wide_checksum_with_field_zeroed(
            &bytes,
            record_bytes,
            JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET,
        );
        bytes[JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET..JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        write(image, device, offset.0, &bytes);
    }
}

/// RR1（m3-root-ring-code-r1 判决第三节表一，正推腿 H1 那一盘面）：新池新建文件、A 的根槽改坏，A 那条记录的卸载记号字段写成 1、2、
/// 0x8000_0000（树表指针改坏，重放出来那一版走不完）。恢复扫环只收卸载记号位 0 之外全 0 的；池级 checker 收下同一批：
/// 收下时它重放那条记录、I-4.1 红，不收时所选根之上一次发布都不施加、I-4.1 报不适用。checker 不判那 4 字节时 2 与 0x8000_0000 两份红。
// harness-test-granularity:one-scenario 同一份盘面上卸载记号字段的三个取值各拷一份镜像比，循环里不建池
#[test]
fn a_record_whose_unmount_marker_has_bits_other_than_bit_zero_is_dropped_by_the_checker_as_by_the_recovery(
) {
    let pool = build_pool("unmount-marker-bits-other-than-bit-zero");
    let mut torn = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut torn, 3);
    for (field, kept_by_the_recovery) in [(1u32, true), (2u32, false), (0x8000_0000u32, false)] {
        let mut image = torn.clone();
        set_the_unmount_marker_field_and_break_the_tree_table_pointer_of_the_first_file_record(
            &mut image, field,
        );
        let system_configuration = choose_system_configuration(&image).expect("择得出系统配置");
        let kept_by_the_scan =
            scan_journal(&image, &system_configuration).contains_key(&(InstanceGeneration(1), 3));
        assert_eq!(
            kept_by_the_scan, kept_by_the_recovery,
            "卸载记号字段 {field:#x}：恢复扫环收不收 A 那条记录"
        );
        let replayed = verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE);
        if kept_by_the_recovery {
            assert!(
                matches!(replayed, InvariantVerdict::Violated(_)),
                "卸载记号字段 {field:#x}：恢复收下、checker 也收下，重放出来那一版走不完，I-4.1 红：{replayed:?}"
            );
        } else {
            assert_eq!(
                replayed,
                InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
                "卸载记号字段 {field:#x}：恢复不收，checker 也不收，所选根之上一次发布都不施加"
            );
        }
    }
}
