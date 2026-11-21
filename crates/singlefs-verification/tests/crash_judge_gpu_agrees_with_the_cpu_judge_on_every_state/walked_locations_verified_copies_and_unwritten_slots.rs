//! 「走过」按读回那一份的落点认、类型段 0 认孩子的类只认自证过的副本、单元区里没写过的槽读出全零。

use super::*;

// ─── 「走过」按读回那一份的落点认（F-KEY）、类型段 0 认孩子的类只认自证过的副本（F-ID-b）、单元区里没写过的槽读出全零（F-零槽） ───

/// 位置条目里槽号的起点与字节数（盘 4 + 槽 6 + 整单元校验和 4）。
pub(super) const LOCATION_ENTRY_SLOT_OFFSET: usize = 4;

pub(super) const LOCATION_ENTRY_SLOT_BYTES: usize = 6;

/// 码 2 节点第一条条目子指针里第 `location` 条位置条目（[`POINTER_LOCATION_ENTRY_OFFSETS`] 之一）的整单元校验和写成 `checksum`（盘与槽不动）。
fn set_the_checksum_of_a_location_in_the_first_entry(
    bytes: &mut [u8],
    location: usize,
    checksum: u32,
) {
    let field = first_entry_of_a_container_number_node()
        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + location
        + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
    bytes[field..field + 4].copy_from_slice(&checksum.to_le_bytes());
}

/// 码 2 节点第一条条目子指针里第 `location` 条位置条目的槽写成 `slot`、整单元校验和写成 `checksum`（盘不动）。
pub(super) fn set_the_slot_and_checksum_of_a_location_in_the_first_entry(
    bytes: &mut [u8],
    location: usize,
    slot: u64,
    checksum: u32,
) {
    let field = first_entry_of_a_container_number_node()
        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
        + location;
    bytes[field + LOCATION_ENTRY_SLOT_OFFSET
        ..field + LOCATION_ENTRY_SLOT_OFFSET + LOCATION_ENTRY_SLOT_BYTES]
        .copy_from_slice(&slot.to_le_bytes()[..LOCATION_ENTRY_SLOT_BYTES]);
    set_the_checksum_of_a_location_in_the_first_entry(bytes, location, checksum);
}

/// 翻进校验和里的几位：那一份按哪种长度都对不上。
const CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY: u32 = 0x5a5a_5a5a;

/// 两份副本里盘序小、盘序大的那一份（写下标）。
fn copies_in_device_order(flow: &RecordedFlow, copies: &[usize]) -> (usize, usize) {
    if flow.writes[copies[0]].device.0 < flow.writes[copies[1]].device.0 {
        (copies[0], copies[1])
    } else {
        (copies[1], copies[0])
    }
}

/// F-KEY（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 观察 O3 那一行、第三节第 2 条；harness 坏镜像
/// `a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before`
/// 位置条目 0 那一形，录成流）：删除中间态的流上墓碑容器两份的记录宽 40 → 39（重封、沿引用链补到根），再把墓碑树根第一条条目子指针第 0 条位置条目的槽
/// 改成流里最后写的 inode 树根的槽（这一遍在墓碑树之前走过）、整单元校验和写成那个根的 CRC 翻几位（这一份对不上），重封到根。
/// 读者读回的是盘 1 那一份带违例的容器。
fn record_deletion_with_a_wrong_record_width_in_the_tombstone_container_whose_first_location_names_the_inode_root(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-container-record-width-first-location-naming-the-inode-root";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    corrupt_units_and_reseal_up_to_the_root(
        &mut flow,
        &container_copies,
        &set_the_record_width_of_a_tombstone_container_one_below,
    );
    let inode_root_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_tree_root_over_leaf_containers,
        false,
    );
    let inode_root_slot = flow.writes[inode_root_copies[0]].offset.0 / SLOT_BYTES;
    let wrong_checksum_of_the_inode_root = crc32c(
        flow.writes[inode_root_copies[0]]
            .bytes()
            .expect("单元写带字节"),
    ) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
            inode_root_slot,
            wrong_checksum_of_the_inode_root,
        );
    });
    flow
}

/// F-KEY：CPU 池级 checker「走过」按读回那一份（盘 1）的落点认，那片容器照判，红 I-1.7（另有 I-2.1 等：位置条目 0 那一份对不上、
/// 按两槽登记进了 inode 树根的槽）；GPU 走读内核 `w_visit_the_copy_read_back` 同改，每个状态逐位相同（按第 0 条位置条目认的话两边都不判那片容器）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before",
        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_whose_first_location_names_the_inode_root(
            "crash-judge-gpu-i17-tombstone-container-record-width-first-location-naming-the-inode-root",
        ),
        &["I-1.7", "I-2.1"],
    );
}

/// F-ID-b（harness 坏镜像
/// `a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device`
/// 位置条目 0 是 `c16` 那一形，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 0、身份三段清零，子指针第 0 条位置条目的整单元校验和换成
/// 墓碑容器前 16 KiB 的 CRC（盘 0 那一份头里是码 3、按 32 KiB 自证不过），第 1 条换成墓碑树根原样字节的 CRC，重封到根；盘 1 那份容器的前 16 KiB
/// 换成墓碑树根（码 2）的原样字节（按 16 KiB 自证得过）。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_the_root_bytes(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-root-entry-type-zero-container-checked-at-sixteen-kibibytes-beside-the-root-bytes";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    let (first_container_copy, second_container_copy) =
        copies_in_device_order(&flow, &container_copies);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
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
    let container = flow.writes[first_container_copy]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(unit_class(&container), 3, "墓碑容器是码 3");
    let checksum_of_the_first_sixteen_kibibytes = crc32c(&container[..node_bytes.len()]);
    let checksum_of_the_node_bytes = crc32c(&node_bytes);
    assert_eq!(
        device_and_slot_of_a_pointer_location(
            &node_bytes[first_entry_of_a_container_number_node()
                + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET..],
            POINTER_LOCATION_ENTRY_OFFSETS[1],
        )
        .0,
        flow.writes[second_container_copy].device.0,
        "第 1 条位置条目是盘序大的那块盘"
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, 0);
        clear_the_identity_of_the_first_entry_but_its_type(bytes);
        set_the_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
            checksum_of_the_first_sixteen_kibibytes,
        );
        set_the_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            checksum_of_the_node_bytes,
        );
    });
    let WrittenContents::Bytes(second_container) = &mut flow.writes[second_container_copy].contents
    else {
        panic!("单元写是普通写");
    };
    second_container[..node_bytes.len()].copy_from_slice(&node_bytes);
    flow
}

/// F-ID-b：CPU 池级 checker 认「孩子是码 2」只看按自己那一类长度自证过的副本（盘 1 那一份码 2），I-9.23 不红；F-L0 按头里的码 3 比盘 0 那一份
/// 对不上，红 I-2.1、I-4.8、I-7.2、I-7.4；GPU 走读内核 `w_class_of_the_first_copy_verifying_at_the_length_of_its_class` 同改，每个状态逐位相同
/// （取第一份按 16 KiB 对得上的话两边都认成码 3、I-9.23 红）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node",
        &record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_the_root_bytes(
            "crash-judge-gpu-i923-tombstone-root-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
        ),
        &["I-2.1", "I-7.2"],
    );
}

/// 流里谁都没写过、基镜像里全零的一段：两块盘上这一槽起 32 KiB（写之前由 [`assert_nothing_in_the_flow_wrote_the_slot`] 核），在单元区里、
/// 离 [`SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE`] 那几条流用的那一截也远。
pub(super) const SLOT_NOTHING_IN_THE_FLOW_WROTE: u64 = 190_000;

/// `slot` 起 32 KiB 在两块盘上流里没写过、基镜像里全零。
pub(super) fn assert_nothing_in_the_flow_wrote_the_slot(flow: &RecordedFlow, slot: u64) {
    let start = slot * SLOT_BYTES;
    let end = start + 2 * SLOT_BYTES;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert!(
            flow.writes.iter().all(|write| write.device != device
                || write.offset.0 + write.length_in_bytes() <= start
                || write.offset.0 >= end),
            "槽 {slot} 起 32 KiB 在盘 {} 上流里没写过",
            device.0
        );
        assert!(
            singlefs_core::recovery::PoolReader::read(
                &flow.base,
                device,
                singlefs_core::address::DeviceOffsetInBytes(start),
                usize::try_from(end - start).expect("32768"),
            )
            .expect("基镜像里读得到这一截")
            .iter()
            .all(|byte| *byte == 0),
            "槽 {slot} 起 32 KiB 在盘 {} 的基镜像里全零",
            device.0
        );
    }
}

/// F-零槽（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 位置条目指向没写过的槽那一行、第三节第 2 条；攻方
/// `research/prompts/m3-batch29-code-r1-opus-output.md` 第三节 `tomb-L1-loc1zero16-type7` 那一形，录成流）：删除中间态的流上墓碑树根第一条条目
/// 类型段 1 → 7、子指针第 1 条位置条目的槽改成 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成 16 KiB 全零的 CRC，重封到根。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote(
    tag: &str,
) -> RecordedFlow {
    let name =
        "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    let checksum_of_sixteen_zero_kibibytes =
        crc32c(&vec![0u8; usize::try_from(SLOT_BYTES).expect("16384")]);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            SLOT_NOTHING_IN_THE_FLOW_WROTE,
            checksum_of_sixteen_zero_kibibytes,
        );
    });
    flow
}

/// F-零槽：CPU 的稀疏读者对没写过的槽读出全零，头里类标签 0 不登记、16 KiB 与 32 KiB 两种长度都比，全零 16 KiB 的 CRC 对得上，
/// 那一份 I-2.1 成立；孩子按层级 1 的形状登记两槽引用，红 I-9.23、I-3.1、I-3.11（攻方 `RESULT … form=tomb-L1-loc1zero16-type7 red=[I-3.1,I-3.11,I-9.23]`）。
/// GPU 读者 `read_at` 在单元区里找不到落点、整段在盘内时交回全零单元（`crash_judge_common.wgsl` 的 `zero_unit_version_over_a_slot_nothing_wrote`），
/// 每个状态逐位相同（那个槽读不出的话，GPU 会多红 I-2.1、I-4.8、I-7.4）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote",
        ),
        &["I-9.23", "I-3.1"],
    );
}

/// 回退流上 A（txg 3）那片 inode 叶容器（码 3、打包记录类型 2、头里诞生代号 3：回退那一版 D 照抄 A 的 inode 树，两版经同一个树根引用它）。
fn is_the_inode_leaf_container_born_at_txg_three(bytes: &[u8]) -> bool {
    const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
    unit_class(bytes) == 3
        && packed_record_type(bytes) == 2
        && u64::from_le_bytes(
            bytes[PACKED_UNIT_BIRTH_TXG_OFFSET..PACKED_UNIT_BIRTH_TXG_OFFSET + 8]
                .try_into()
                .expect("8 字节"),
        ) == 3
}

/// 分叉形（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P1 那一行③、第三节第 1 条；攻方
/// `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.4 节 `raise_birth=true corrupt_loc0=true direct=false`，录成流）：
/// 固定脚本到回退 D 的流上，A 那片 inode 叶容器 L 两份头里的诞生代号 3 → 9（D 的 txg，重封、沿引用链补到根：A、D 两版照旧经同一个树根引用 L），
/// 再把盘序小的那一份写成别的字节（不补校验和，那个槽上没有别的单元）。harness 坏镜像
/// `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device` 同一形。
fn record_rollback_whose_shared_leaf_container_is_born_after_the_target_with_its_first_copy_garbled(
    tag: &str,
) -> RecordedFlow {
    const ROLLBACK_PUBLISH_TXG: u64 = 9;
    let mut flow = record_fixed_script_through_rollback(tag);
    flow.name = "rollback-shared-leaf-container-born-after-the-target-first-copy-garbled";
    let name = flow.name;
    let leaf_copies = last_unit_copies(
        &flow,
        name,
        &is_the_inode_leaf_container_born_at_txg_three,
        false,
    );
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &leaf_copies, &|bytes| {
        bytes[73..81].copy_from_slice(&ROLLBACK_PUBLISH_TXG.to_le_bytes());
    });
    let (first_copy, _second_copy) = copies_in_device_order(&flow, &leaf_copies);
    let WrittenContents::Bytes(first) = &mut flow.writes[first_copy].contents else {
        panic!("单元写是普通写");
    };
    first.fill(0xa5);
    flow
}

/// allrbmap：L 盘 0 那一份读不出、读回的是盘 1 那一份。CPU 池级 checker 的诞生代号、并进前一版与 I-3.5 的折叠表都按读回那一份的落点认，
/// A 那条根（txg 3，不是最新的候选根）引用 L、L 读得出的那一份诞生代号 9 > 3，红 I-4.9；D 引用、C 不引用的 inode 树根那一格子树里 L 的诞生代号
/// 大于树根指针的出生 txg，红 I-3.5。GPU 走读内核同改（`w_read_unit_to_walk` 记下读回那一份的落点），每个状态逐位相同
/// （GPU 若把诞生代号记在盘 0 那个落点的走过表行上，那一行不存在，两条都漏）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy",
        &record_rollback_whose_shared_leaf_container_is_born_after_the_target_with_its_first_copy_garbled(
            "crash-judge-gpu-i49-shared-leaf-container-born-after-the-target-first-copy-garbled",
        ),
        &["I-4.9", "I-3.5"],
    );
}

/// 两份副本里离盘尾近、槽号装不进 32 位的位置：槽号加 2³²（`zero_unit_version_over_a_slot_nothing_wrote` 的守卫按 `offset.y >> (32 − SLOT_SHIFT)` 判时
/// 落在 [2³², 2³⁶) 的槽号回绕成低 32 位）。
const SLOTS_IN_THIRTY_TWO_BITS: u64 = 1 << 32;

/// P4 回绕（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P4 回绕那一行、第三节第 2 条；正推
/// `research/prompts/m3-batch32-code-r1-sonnet-output.md` 第八节第 1 条）：同 F-零槽那条流
/// （[`record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote`]），
/// 第 1 条位置条目的槽写成 `slot`、整单元校验和写成 16 KiB 全零的 CRC。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
    tag: &str,
    name: &'static str,
    slot: u64,
) -> RecordedFlow {
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    let checksum_of_sixteen_zero_kibibytes =
        crc32c(&vec![0u8; usize::try_from(SLOT_BYTES).expect("16384")]);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            slot,
            checksum_of_sixteen_zero_kibibytes,
        );
    });
    flow
}

/// P4 回绕：槽号是 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] 加 2³²。CPU 的稀疏读者越过盘尾读不到，那一份 I-2.1 红（I-4.8、I-7.4 跟着）；
/// GPU `read_at` 的全零单元守卫按 `SLOT_SHIFT` 判、拒槽号 ≥ 2³²，同样读不到，每个状态逐位相同（守卫按 `offset.y >> (32 − SLOT_SHIFT)` 判时
/// 回绕成槽 190000、交回全零单元，I-2.1 成立，少红三条）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_past_thirty_two_bits_with_the_checksum_of_zeros(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_past_thirty_two_bits_with_the_checksum_of_zeros",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-past-thirty-two-bits",
            "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-past-thirty-two-bits",
            SLOTS_IN_THIRTY_TWO_BITS + SLOT_NOTHING_IN_THE_FLOW_WROTE,
        ),
        &["I-2.1", "I-9.23"],
    );
}

/// P4 回绕：槽号是 2³² − 1（装得进 32 位，`first_slot + 读的槽数` 那一加在 32 位里回绕成 0）。CPU 越过盘尾读不到，I-2.1 红；GPU 先判起点
/// 不越过空段终点、再拿差比，同样读不到，每个状态逐位相同。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-last-slot-in-thirty-two-bits",
            "deletion-tombstone-root-entry-type-seven-second-location-naming-the-last-slot-in-thirty-two-bits",
            SLOTS_IN_THIRTY_TWO_BITS - 1,
        ),
        &["I-2.1", "I-9.23"],
    );
}

/// 攻方那一形（`research/prompts/m3-batch32-code-r1-opus-output.md` 第二节 `P4 p4 type=7 next_written=true sum=zero32`，录成流）：
/// 同 F-零槽那条流，第 1 条位置条目指着 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成 32 KiB 全零的 CRC；它的下一槽在两块盘的基镜像里
/// 写进 16 KiB 非零字节（流里谁都没写过，每个状态都在）。
fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote-before-a-written-slot";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
    let next_slot_offset = (SLOT_NOTHING_IN_THE_FLOW_WROTE + 1) * SLOT_BYTES;
    let bytes_of_the_next_slot = vec![0x5au8; usize::try_from(SLOT_BYTES).expect("16384")];
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        flow.base.devices.get_mut(&device).expect("盘在池里").write(
            singlefs_core::address::DeviceOffsetInBytes(next_slot_offset),
            &bytes_of_the_next_slot,
        );
    }
    let checksum_of_thirty_two_zero_kibibytes =
        crc32c(&vec![0u8; usize::try_from(2 * SLOT_BYTES).expect("32768")]);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            SLOT_NOTHING_IN_THE_FLOW_WROTE,
            checksum_of_thirty_two_zero_kibibytes,
        );
    });
    flow
}

/// 攻方那一形：CPU 头里类标签 0 不登记，按 16 KiB 读出全零、按 32 KiB 读出「16 KiB 零 + 下一槽的字节」，两种长度都对不上 32 KiB 全零的 CRC，I-2.1 红。
/// GPU F-α 那一处比 32 KiB 时另按 32 KiB 读一次（空段只装得下一槽，读不到），同样对不上，每个状态逐位相同（拿按 16 KiB 读回的全零切片的 32 KiB
/// 检查字去比时判对得上，少红 I-2.1、I-4.8、I-7.4）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
) {
    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes",
        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-slot-nothing-wrote-before-a-written-slot",
        ),
        &["I-2.1", "I-9.23"],
    );
}

fn leave_the_records_before_the_mount_as_they_are(_flow: &mut RecordedFlow) {}

/// C744 ①（`.claude/kb/checks-owed.md` C744；正推 `research/prompts/m3-batch10o-code-r1-sonnet-output.md` K3-2 第 1 类）：三个数据单元那次发布
/// （txg 4）只落了三条记录、根槽没落，新进程可写挂载施加它、取号 2、写行、暖机，再以实例 2 覆盖写一次（记录照旧，
/// [`record_three_unit_file_applied_only_by_its_records_after_a_mount`]）。挂载写了实例 2 的行、覆盖写的记录落了而根槽没落的状态上，
/// txg 4 是由记录施加出来的那一版、覆盖写那一版是下一次挂载先施加的那一版，两样同在（推的，读录制的次序）：扫描内核
/// `judge_allocation_records_disjoint` 与 `judge_allocation_generations_against_unit_births` 的末圈与前两段都走到。每个状态 GPU 与 CPU 逐位相同。
/// 两样同在的状态至少一个（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S6 行）：CPU 看 journal 那一遍报的根（所选根，
/// 根环里最新的那条）是实例 2 的（挂载写行那一版起每条根的实例表都带实例 1 那一行 (1, 4)，txg 4 就是由记录施加出来的那一版），
/// 又从它往上施加了记录（实际走的那条根不是它：覆盖写那一版是下一次挂载先施加的那一版）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first(
) {
    const INSTANCE_OF_THE_MOUNT: u32 = 2;
    let test_name = "the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first";
    let flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
        "crash-judge-gpu-c744-version-applied-by-records-beside-the-next-mount-version",
        "three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first",
        &leave_the_records_before_the_mount_as_they_are,
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some(comparison) = compare_on_every_state_examining(
        cards,
        &flow,
        expand_up_to(),
        &[StateExamination::CountStatesWhere(
            StateCondition::CpuReportedRootOfTheInstanceWhileTheConsultedPassLandsElsewhere {
                instance: INSTANCE_OF_THE_MOUNT,
            },
        )],
    ) else {
        return;
    };
    let states_holding_both_versions = comparison.examined[0].states_counted;
    eprintln!(
        "CRASH_JUDGE_GPU_C744 flow={} states={} states_holding_both_versions={states_holding_both_versions}",
        flow.name, comparison.states
    );
    assert!(
        states_holding_both_versions > 0,
        "流 {}：至少一个状态同时带由记录施加出来的那一版（txg 4）与下一次挂载先施加的那一版（所选根是实例 {INSTANCE_OF_THE_MOUNT} 的、又施加了记录）",
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

/// F-ID-b 两份都没自证过（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条；
/// harness 坏镜像 `a_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_tombstone_entry_invariant_holding_on_either_device`
/// 位置条目 0 是 `C16`、1 是 `BAD` 那一格，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 0、身份三段清零，子指针第 0 条位置条目的
/// 整单元校验和换成墓碑容器前 16 KiB 的 CRC（头里是码 3、按 32 KiB 自证不过），第 1 条换成容器整单元 CRC 翻几位，重封到根。孩子读不出时这一小句不判（墓碑树条目身份与子头相符照旧成立）。
fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify(
    tag: &str,
) -> RecordedFlow {
    let name = "deletion-tombstone-root-entry-type-zero-two-copies-that-do-not-verify";
    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
    assert_the_first_entry_names_the_unit_of(
        &flow,
        root_copies[0],
        &container_copies,
        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
    );
    let container = flow.writes[container_copies[0]]
        .bytes()
        .expect("单元写带字节")
        .to_vec();
    assert_eq!(unit_class(&container), 3, "墓碑容器是码 3");
    let checksum_of_the_first_sixteen_kibibytes =
        crc32c(&container[..usize::try_from(SLOT_BYTES).expect("16384")]);
    let wrong_checksum_of_the_container =
        crc32c(&container) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
        set_the_type_of_the_first_entry(bytes, 0);
        clear_the_identity_of_the_first_entry_but_its_type(bytes);
        set_the_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[0],
            checksum_of_the_first_sixteen_kibibytes,
        );
        set_the_checksum_of_a_location_in_the_first_entry(
            bytes,
            POINTER_LOCATION_ENTRY_OFFSETS[1],
            wrong_checksum_of_the_container,
        );
    });
    flow
}

/// F-ID-b 两份都没自证过：CPU 池级 checker 当读不出、「子单元是码 2」那一小句不判，I-9.23 不红；F-L0 每份照判 I-2.1（码 3 按 32 KiB 对不上、
/// 翻过位的对不上），红 I-2.1。GPU 走读内核不取「第一份按 16 KiB 对得上的」那一读，每个状态逐位相同（取了的话 GPU 读出码 3、红 I-9.23）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_child_whose_two_copies_do_not_verify(
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_child_whose_two_copies_do_not_verify",
    ) else {
        return;
    };
    let flow = record_deletion_with_the_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify(
        "crash-judge-gpu-i923-tombstone-root-entry-type-zero-two-copies-that-do-not-verify",
    );
    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
        return;
    };
    assert!(
        comparison.cpu_checker_red_invariants.contains("I-2.1")
            && !comparison.cpu_checker_red_invariants.contains("I-9.23"),
        "流 {} 上 CPU 红 I-2.1、一个状态都不红 I-9.23（红过的：{:?}）",
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

/// 删完的流（[`record_new_pool_through_the_last_deletion_batch`]）带「没有文件的那几版」：意图那一版（txg 5）起没有文件，三批（txg 6–8）不进表、
/// 照比它旧的最新那一项（意图那一版）判（D8（核心索引结构） 已定项 16 第 4 条）。
/// 恢复落到 txg 5–8 报没有文件的状态上 CPU 的 oracle 一个违例都不记；GPU 恢复内核的期望版本表第 5 字记这一版有没有文件、`classify_oracle` 照认，
/// 每个状态逐位相同（不认的话这几版报没有文件判成「更旧的根下面有文件」）。另钉 CPU 两遍 oracle 与记录核对器在每个状态上一个违例都没有、
/// 至少一个状态看 journal 那一遍落到意图那一版 (1, 5) 报没有文件（判决 `research/prompts/m3-batch33s-code-r1-main-verification.md` 第二节 S5 行：
/// 「没有文件的那几版」写错身份时 CPU、GPU 一起判违例、逐位仍相同，只比两边对不对得上看不出来）。
#[test]
fn the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on(
) {
    let Some(cards) = gpu_card_pool_or_report_not_run(
        "the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on",
    ) else {
        return;
    };
    let flow = record_new_pool_through_the_last_deletion_batch(
        "crash-judge-gpu-oracle-deleted-flow-without-a-file-from-the-intent-on",
        "deleted-flow-against-a-version-table-without-a-file-from-the-intent-on",
    );
    let versions_without_a_file = [PublishedVersionWithoutAFile {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(5),
    }];
    let intent_version = (
        INSTANCE_OF_THE_PUBLISHES.0,
        versions_without_a_file[0].checkpoint_txg.0,
    );
    let Some(comparison) = compare_on_every_state_against_the_version_table(
        cards,
        &flow,
        &versions_without_a_file,
        expand_up_to(),
        &[StateExamination::CountStatesWhere(
            StateCondition::CpuConsultedPassLandsOn {
                version: intent_version,
                class: RECOVERY_WORD_CLASS_NO_FILE,
            },
        )],
    ) else {
        return;
    };
    assert_eq!(
        comparison.cpu_red, 0,
        "流 {}：CPU 两遍 oracle 与记录核对器在每个状态上一个违例都没有（意图那一版起没有文件，表里照认）",
        flow.name
    );
    assert!(
        comparison.examined[0].states_counted > 0,
        "流 {}：至少一个状态看 journal 那一遍落到意图那一版 {intent_version:?}、报没有文件",
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
