//! 节点身份与树表的形状：空叶子的块身份、inode 记录的块数、树表条目的种类与次序、树号与树号水位、复用、条目宽度、区域个数。

use super::*;

/// 分配记录树第 1 层、盘 0 那一格的节点（写完新池新建文件那份镜像上：`UNITS` 里 50247，它下面只有盘 0 的叶 61）。
const ALLOCATION_LEVEL_ONE_NODE_OF_DEVICE_ZERO: u64 = 50247;

/// 挂空叶用的槽：新池新建文件写完之后紧挨着树表（50252）的第一个空槽，与已分配的那一段相连——占掉它，空闲段数与全空聚簇段数都不变。
const EMPTY_LEAF_SLOT: u64 = 50253;

/// 空叶的出生序号：这棵树在 txg 3 里发过的号之外的一个。
const EMPTY_LEAF_BIRTH_SEQUENCE: u32 = 200;

/// 写完新池新建文件的镜像上，往分配记录树的根之下挂一个空节点、其余全部照账补齐：
/// 一、空节点那一槽（50253）在每块盘上各记一条已分配记录（分配代 3），记在罩着它的那片叶 61 里；记账的已分配各加一槽、空闲各减一槽；
/// 二、盘 0 的叶 62（按位置罩 [62 × 812, 63 × 812)，那一段里一条记录都没有）写成一个一条条目都没有的节点，放在 50253，
///     挂到第 1 层盘 0 那一格下面；改过的每个单元重封、新的整单元校验和沿引用链补到根槽（`mutate_unit` / `propagate`）。
fn hang_an_empty_leaf_below_the_root_of_the_allocation_record_tree(image: &mut MemoryPool) {
    use singlefs_core::address::SlotNumber;
    use singlefs_core::allocation_record_tree::{
        AllocationRecordTreeGeometry, AllocationRecordTreeNode,
    };
    use singlefs_core::allocator::AllocationRecord;
    use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
    use singlefs_core::records::{STATISTIC_ALLOCATED_BYTES, STATISTIC_FREE_BYTES};
    use singlefs_core::unit::{build_index_node, parse_index_node};
    let geometry = AllocationRecordTreeGeometry::of_devices(
        &DEVICES.map(|device| (DeviceIdentity(device), image.device_size_in_bytes / SLOT)),
    );
    let filesystem_identifier = parameters().filesystem_identifier;
    for (device, leaf_slot) in DEVICES.into_iter().zip(ALLOCATION_LEAVES) {
        let leaf = parse_index_node(&read(image, 0, leaf_slot * SLOT, 16384)).expect("叶 61");
        let mut entries = leaf.entries.clone();
        entries.push(
            AllocationRecord {
                device: DeviceIdentity(device),
                slot: SlotNumber(EMPTY_LEAF_SLOT),
                span_slots: 1,
                generation: CheckpointTxg(3),
                is_released: false,
                quarantine_mark:
                    singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
            }
            .to_bytes(),
        );
        entries.sort();
        let rebuilt = build_index_node(
            leaf.tree,
            leaf.level,
            leaf.key_width,
            &leaf.smallest_key,
            &leaf.largest_key,
            leaf.birth_txg,
            &filesystem_identifier,
            leaf.instance,
            leaf.birth_sequence,
            u16::try_from(leaf.entry_width).expect("20"),
            &entries,
        );
        mutate_unit(
            image,
            leaf_slot,
            |bytes| bytes.copy_from_slice(&rebuilt),
            false,
        );
    }
    mutate_unit(
        image,
        ACCOUNTING_ROOT,
        |bytes| {
            for device in DEVICES {
                adjust_accounting(bytes, STATISTIC_ALLOCATED_BYTES, device, SLOT);
                reduce_accounting(bytes, STATISTIC_FREE_BYTES, device, SLOT);
            }
        },
        true,
    );
    let level_one = parse_index_node(&read(
        image,
        0,
        ALLOCATION_LEVEL_ONE_NODE_OF_DEVICE_ZERO * SLOT,
        16384,
    ))
    .expect("第 1 层盘 0 那一格");
    let (smallest_key, largest_key) = geometry.key_range_of(
        AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
            level: 0,
            device: DeviceIdentity(0),
            index_in_device: 62,
        }),
    );
    let empty_leaf = build_index_node(
        level_one.tree,
        0,
        level_one.key_width,
        &smallest_key,
        &largest_key,
        level_one.birth_txg,
        &filesystem_identifier,
        level_one.instance,
        BirthSequence(EMPTY_LEAF_BIRTH_SEQUENCE),
        u16::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
        &[],
    );
    for device in DEVICES {
        write(image, device, EMPTY_LEAF_SLOT * SLOT, &empty_leaf);
    }
    let empty_leaf_checksum = crc32_castagnoli(&empty_leaf);
    let pointer = NodePointer {
        head: PointerHead {
            birth_tree: level_one.tree,
            birth_txg: level_one.birth_txg,
        },
        locations: DEVICES.map(|device| LocationEntry {
            device: DeviceIdentity(device),
            slot: SlotNumber(EMPTY_LEAF_SLOT),
            unit_checksum: empty_leaf_checksum,
        }),
        instance: level_one.instance,
        birth_sequence: BirthSequence(EMPTY_LEAF_BIRTH_SEQUENCE),
    };
    let mut pointer_writer = singlefs_core::bytes::ByteWriter::new(
        usize::try_from(singlefs_format::NODE_POINTER_BYTES).expect("86"),
    );
    pointer.write_to(&mut pointer_writer);
    let mut entry_for_the_empty_leaf = smallest_key.clone();
    entry_for_the_empty_leaf.extend_from_slice(&pointer_writer.into_bytes());
    let mut level_one_entries = level_one.entries.clone();
    level_one_entries.push(entry_for_the_empty_leaf);
    let rebuilt_level_one = build_index_node(
        level_one.tree,
        level_one.level,
        level_one.key_width,
        &level_one.smallest_key,
        &level_one.largest_key,
        level_one.birth_txg,
        &filesystem_identifier,
        level_one.instance,
        level_one.birth_sequence,
        u16::try_from(level_one.entry_width).expect("96"),
        &level_one_entries,
    );
    mutate_unit(
        image,
        ALLOCATION_LEVEL_ONE_NODE_OF_DEVICE_ZERO,
        |bytes| bytes.copy_from_slice(&rebuilt_level_one),
        false,
    );
    // 中央映射（根兼叶）里加那片空叶的一条条目：分配记录树的每个节点都进映射（D19（块指针的结构与宽度预算） 已定项 8），
    // 不加的话它的未释放记录在映射里找不到相符的 key，I-3.15（N2 反查）跟着红，这份镜像就不只红 I-1.1 了。按 key 升序放好、叶头区间贴紧首末条目。
    let mapping = parse_index_node(&read(image, 0, MAPPING_ROOT * SLOT, 16384)).expect("映射根");
    let mut mapping_entries = mapping.entries.clone();
    mapping_entries.push(singlefs_core::records::build_mapping_entry(
        &singlefs_core::records::mapping_key_for_node(
            singlefs_core::unit::UNIT_CLASS_INDEX_NODE,
            pointer,
        ),
        pointer.locations,
    ));
    mapping_entries.sort_by_key(|entry| singlefs_core::records::mapping_key_sort_key(entry));
    let mapping_key_width = mapping.key_width;
    let rebuilt_mapping = build_index_node(
        mapping.tree,
        mapping.level,
        mapping_key_width,
        &mapping_entries.first().expect("有条目")[..mapping_key_width],
        &mapping_entries.last().expect("有条目")[..mapping_key_width],
        mapping.birth_txg,
        &filesystem_identifier,
        mapping.instance,
        mapping.birth_sequence,
        u16::try_from(mapping.entry_width).expect("55"),
        &mapping_entries,
    );
    mutate_unit(
        image,
        MAPPING_ROOT,
        |bytes| bytes.copy_from_slice(&rebuilt_mapping),
        false,
    );
}

/// 按位置寻址的树根之下挂一个空节点（实二一报告「停下交主 agent」第 5 条，主 agent 定放在 I-1.1 那一格）：分配记录树第 1 层盘 0 那一格下面
/// 挂一片一条记录都没有的叶 62，账与引用链都补齐（`hang_an_empty_leaf_below_the_root_of_the_allocation_record_tree`）——
/// D8（核心索引结构） 已定项 14「缺席即全空闲、没有记录的一段不写节点」，实现的读者拒收它；池级 checker **只**红 I-1.1（块头自述逻辑地址），
/// 红在读那个节点那一步：`check_internal_node_separators` 对按位置寻址的节点判「节点不空」（`walk::read_index_node` 的
/// `PrescribedByThePositionJudgedByTheCaller` 那一臂）。
/// ⚠️ 派发规格写「今天的 checker 上必须不红、改完红」：这份镜像在实二五开工那一刻的 checker 上就只红 I-1.1（草稿副本上现跑过），
/// 那一判已经在，`walk.rs` 没改；这条用例钉住它，变异表里另有一行拿掉那一判时它必须红。
#[test]
fn an_empty_leaf_hung_below_the_root_of_the_allocation_record_tree_reddens_only_the_block_identity_invariant(
) {
    let clean = build_pool("known-bad-empty-leaf-below-the-root").memory_pool();
    assert_eq!(
        violated_invariants(&check_pool_image(&clean)),
        Vec::<&str>::new(),
        "干净镜像上一条不红"
    );
    let mut image = clean;
    hang_an_empty_leaf_below_the_root_of_the_allocation_record_tree(&mut image);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec!["I-1.1"],
        "只红 I-1.1：账、校验和、引用链都补齐了"
    );
    let detail = verdicts
        .iter()
        .find_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) if *invariant == "I-1.1" => Some(detail.clone()),
            InvariantVerdict::Violated(_)
            | InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .expect("I-1.1 红");
    assert!(
        detail.contains("层级 0 盘 0 第 62 格的节点"),
        "红在挂上去的那片空叶上：{detail}"
    );
}

/// I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）：新池新建文件那条记录 size 3000、blocks 6。三个取样值各改一份：
/// 64 是按分到一个 32 KiB 单元算的旧占位值，5 是向下取整，7 是多一块；每一份都**只**红 I-9.15，红在「blocks 与 ⌈3000 ÷ 512⌉ = 6 不等」。
/// 盘上原样的 6 就是干净镜像（`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 判它成立）。
#[test]
fn inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant(
) {
    let clean = build_pool("known-bad-inode-blocks").memory_pool();
    assert_eq!(
        verdict(&clean, "I-9.15"),
        InvariantVerdict::Holds,
        "干净镜像上 blocks = 6：I-9.15 真被评估过且成立"
    );
    for wrong_blocks in [64u64, 5, 7] {
        let mut image = clean.clone();
        mutate_unit(
            &mut image,
            INODE_LEAF,
            |bytes| set_u64(bytes, 136 + INODE_RECORD_BLOCKS_OFFSET, wrong_blocks),
            true,
        );
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            ["I-9.15"],
            "blocks 写成 {wrong_blocks}：判红的该只有 I-9.15：{verdicts:?}"
        );
        let InvariantVerdict::Violated(detail) = verdict(&image, "I-9.15") else {
            panic!("上面判过 I-9.15 红");
        };
        assert_eq!(
            detail,
            format!("inode 1 的记录 blocks {wrong_blocks} 不等于 ⌈size 3000 ÷ 512⌉ = 6"),
            "红在第一个文件那条记录上"
        );
    }
}

/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：写完新池新建文件那一版树表里 livelist（号 65541）与稀疏旁表（号 65542）两条互换种类。
/// 盘上次序仍按树 ID 升序（① 成立，树表的 key 就是树 ID，I-1.1 也不红），按发号次序 livelist 的号 65542 大过稀疏旁表的号 65541（② 不成立）：
/// **只**红 I-9.16，第一处违例写明最新那条根（实例 1、txg 3）与发号次序上相邻的那两条。两条的根都是空根，改种类不牵动别的判定。
/// 两条整条互换位置（只违反 ①）的那一份连 I-1.1 一起红，钉在 `tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`。
#[test]
fn a_tree_table_whose_two_entries_swapped_their_kinds_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean = build_pool("known-bad-tree-table-ordering").memory_pool();
    assert_eq!(
        verdict(&clean, "I-9.16"),
        InvariantVerdict::Holds,
        "干净镜像上 I-9.16 真被评估过且成立"
    );
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(bytes),
        true,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.16"],
        "两条互换种类：判红的该只有 I-9.16：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-9.16"),
        InvariantVerdict::Violated(
            "根（实例 1、txg 3）的树表：种类 6 那条（第 5 条，树 ID 65542）与发号次序上的下一棵、种类 7 那条（第 4 条，树 ID 65541）不按树 ID 严格升序（② 发号次序）"
                .to_owned()
        ),
        "红在最新那条根的树表、livelist 与稀疏旁表那两条"
    );
}

/// 根记录里中央映射树根指针的出生树：指针在根记录偏移 256（D22（单元原子性怎么合成） 已定项 7），出生树在指针偏移 34
/// （D19（块指针的结构与宽度预算） 已定项 11）。
const ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET: usize = 256 + 34;

/// 中央映射树不进树表：I-1.3（块头树 ID 一致） 对它的根，「实际引用它的树」按根记录里那条根指针的出生树取
/// （回退到树表 0 条的一版之后再发第一个文件版本，十二棵树从水位重新发号，映射树不再恒是 65540——C511（回退到无文件那一版之后诞生代怎么接））。
/// 两份坏镜像各只改一边、都只有 I-1.3 红：映射树根头里的树 ID 改成 65541（指针仍说 65540），与根记录里那条指针的出生树改成 65541
/// （头仍说 65540，重封根槽的自证校验和）。后一份钉的是「按指针判」这个读法：checker 退回写死 65540 时它不红。
/// 取 65541 不取一个大数：65541 是这个池发过的号（livelist 树）、低于水位 65548，改成水位之上的号会让 I-7.8（根记录树 ID 水位不低于全池最大树 ID）
/// 跟着红（它扫码 2 单元头），判别力就算不到 I-1.3 一条头上。
#[test]
fn central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant(
) {
    let clean = build_pool("known-bad-central-mapping-tree").memory_pool();
    let violated_on_the_clean_image = violated_invariants(&check_pool_image(&clean));
    assert!(
        violated_on_the_clean_image.is_empty(),
        "干净镜像上一条违例都没有：{violated_on_the_clean_image:?}"
    );
    let header_says_another_tree: Mutation = Box::new(|image: &mut MemoryPool| {
        mutate_unit(
            image,
            MAPPING_ROOT,
            |bytes| set_u64(bytes, 42, ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK),
            true,
        );
    });
    let root_pointer_says_another_tree: Mutation = Box::new(|image: &mut MemoryPool| {
        let (device, offset) = NEW_POOL_FILE_CREATION_ROOT_SLOT;
        let mut root_slot = read(image, device, offset, 512);
        assert_eq!(
            get_u64(&root_slot, ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET),
            65540,
            "新池新建文件那条根的映射树根指针说出生树 65540"
        );
        set_u64(
            &mut root_slot,
            ROOT_RECORD_MAPPING_POINTER_BIRTH_TREE_OFFSET,
            ANOTHER_ISSUED_TREE_BELOW_THE_WATERMARK,
        );
        let digest = wide_checksum_with_field_zeroed(&root_slot, 512, 138);
        root_slot[138..170].copy_from_slice(&digest);
        write(image, device, offset, &root_slot);
        copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
    });
    for (which_side, mutation) in [
        ("映射树根头里的树 ID", header_says_another_tree),
        (
            "根记录里映射树根指针的出生树",
            root_pointer_says_another_tree,
        ),
    ] {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        assert_eq!(
            violated_invariants(&verdicts),
            ["I-1.3"],
            "只改了{which_side}：判红的该只有 I-1.3：{verdicts:?}"
        );
    }
}

/// 只被最新根引用的单元被盖（A 的数据单元改内容重封，暖机根与种子根都不引用它）：I-7.4 在最新根那一格也要红，
/// 不能只在更早的候选根上判（两条不变量都写「候选集里每一条根」，最新根也在候选集里；
/// 根环种子 24 条 txg 0 的根让「候选集只剩最新根」构造不出来，所以用这一格钉最新根那一次判定）。
#[test]
fn reused_unit_referenced_only_by_the_newest_root_makes_the_reuse_invariant_red() {
    let mut image = build_pool("known-bad-newest-only").memory_pool();
    // 直接改盘上的单元、不把新校验和传播进位置条目：这就是「块被别的对象盖了」在盘上的样子。
    let mut unit = read(&image, 0, DATA_UNIT * SLOT, 32768);
    set_u64(&mut unit, 100, 77);
    reseal_the_unit_header_and_payload(&mut unit);
    for device in DEVICES {
        write(&mut image, device, DATA_UNIT * SLOT, &unit);
    }
    assert!(
        matches!(verdict(&image, "I-7.4"), InvariantVerdict::Violated(_)),
        "只被 A 引用的数据单元被盖，I-7.4 要在最新根上判红：{:?}",
        verdict(&image, "I-7.4")
    );
}

/// I-1.10（码 2 条目宽等于字段表宽） 那份坏镜像**只**红在 I-1.10 上：记账树根自述的条目宽缩到 key 宽 22
/// （字段表宽 34），别的自描述字段跟着调成自洽的一份（见 `narrow_the_entry_width_to_the_key_width`）。
///
/// 跟着变的只有四条**不适用**：走读在判红的那一步停下、不按字段表解那 15 行记账条目，于是
/// I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读）、I-5.2（空闲统计对得上）与 I-9.6（水位大于两处最大号） 都没有了判的对象
/// （另有 I-3.13 与 I-3.4（可用空间扣待删占用） 两条读的也是记账行：删除态扫描里记账树条目宽不到 34 那一版报射程里判不了；
/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上） 的台阶行取自主走读走全了的记账树，走不全就没有行可判）。
/// 「停下不解」正是 I-1.10（码 2 条目宽等于字段表宽） 要的判定顺序（**先于**按字段表解条目，与
/// I-1.7（打包容器合法与判定顺序） 同一纪律），所以这四条转成不适用是这份镜像的结构决定的，照实钉下来。
#[test]
fn an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant() {
    let clean = build_pool("known-bad-entry-width").memory_pool();
    assert_eq!(
        verdict(&clean, "I-1.10"),
        InvariantVerdict::Holds,
        "干净镜像上 I-1.10 要真被评估过且成立（四棵树的根都比过条目宽）"
    );
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        ACCOUNTING_ROOT,
        |bytes| narrow_the_entry_width_to_the_key_width(bytes),
        true,
    );
    let verdicts = check_pool_image(&image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(violated, ["I-1.10"], "判红的该只有 I-1.10：{verdicts:?}");
    let turned_not_applicable: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::NotApplicable(_)))
        .map(|(name, _)| *name)
        .filter(|name| !matches!(verdict(&clean, name), InvariantVerdict::NotApplicable(_)))
        .collect();
    assert_eq!(
        turned_not_applicable,
        [
            "I-3.1",
            "I-3.11",
            "I-5.2",
            "I-9.6",
            "I-3.13",
            "I-9.30",
            FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
            FREE_ALLOCATED_AND_UNRECLAIMABLE_SUM_TO_THE_CAPACITY_AND_THE_COMMITTED_RESERVATION_MATCHES_ITS_CLOSED_FORM
        ],
        "跟着转成不适用的该只有这八条（走读在 I-1.10 那一步停下，记账条目不再解；I-3.13 判的就是记账行的代段；\
         I-9.30 的台阶行取自走全了的记账树；I-3.16 比的 runs 与全空段那两行也取自走全了的记账树；\
         I-3.20 比的空闲、已分配、不可回收与标签 6 那几行也取自走全了的记账树）：{verdicts:?}"
    );
    // I-3.4 的删除态扫描读记账树也按条目宽 ≥ 34 才解，读不全的那一版在射程里、判不了：报部分没判、点名记账树读不全
    // （C701（别的判法静默跳过读不出的部分），一律报部分没判）。
    match verdict_among(&verdicts, "I-3.4") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains("记账树读不全"),
            "I-3.4 的部分没判点名记账树读不全：{unjudged}"
        ),
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => {
            panic!("I-3.4 在记账树读不全的那一版判不了：要报部分没判，实际 {other_verdict:?}")
        }
    }
}

/// 系统配置自述的区域数 R 越过字段表给逐区域设备身份留的那三个字段（偏移 379 / 383 / 387）时，
/// `geometry_of` 交回 [`singlefs_checker::Verdict::RegionCountPastTheRegionDeviceFields`]，不夹成 3、也不 panic
/// （panic 面普查 R12：`root_slot_positions` 拿这个数下标一个长 3 的数组）。
///
/// 写死的字节：槽里偏移 361 那一字节（R）从 3 改成 4，整槽自证校验和照新内容重算 ⇒
/// `check_system_configuration_slot` 那一道照样过，**只有**新加的这一道拦得住它。
#[test]
fn region_count_past_the_region_device_fields_is_refused_by_the_geometry_reader() {
    let mut pool = build_pool("known-bad-region-count").memory_pool();
    let clean_slot = read(&pool, 0, 0, 4096);
    let clean_view =
        singlefs_checker::check_system_configuration_slot(&clean_slot).expect("干净槽自证过");
    assert_eq!(clean_slot[361], 3, "干净镜像上的区域数是 3");
    assert!(
        singlefs_checker::image::geometry_of(&clean_slot, &clean_view).is_ok(),
        "R = 3 正好用满那三个字段，要交得出几何"
    );
    mutate_system_configuration_slot(&mut pool, 0, 0, |bytes| bytes[361] = 4);
    let damaged_slot = read(&pool, 0, 0, 4096);
    let damaged_view = singlefs_checker::check_system_configuration_slot(&damaged_slot)
        .expect("改完重算过整槽校验和，自证那一道还过");
    assert_eq!(
        singlefs_checker::image::geometry_of(&damaged_slot, &damaged_view),
        Err(singlefs_checker::Verdict::RegionCountPastTheRegionDeviceFields),
        "R = 4 时第 4 个区域没有设备身份可读，这一槽不可择"
    );
}

/// 根环里每一个带根 magic 的槽：树 ID 水位改成 `watermark`、重封自证校验和（罩整槽 512，字段在 138）。返回改了几个槽。
/// 盘上留下的就是「每次发布都没把水位抬过这一代发出的号」的样子（I-7.8 那一行判别力 ① 那一类）。
fn set_the_tree_identifier_watermark_of_every_root(
    image: &mut MemoryPool,
    watermark: u64,
) -> usize {
    let mut rewritten = 0;
    for (device, offset) in root_slot_offsets_of_both_devices() {
        let mut bytes = read(image, device, offset, 512);
        if bytes[..4] != singlefs_core::root_record::ROOT_MAGIC {
            continue;
        }
        set_u64(
            &mut bytes,
            ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET,
            watermark,
        );
        let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
        bytes[138..170].copy_from_slice(&digest);
        write(image, device, offset, &bytes);
        rewritten += 1;
    }
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
    rewritten
}

/// 一份干净镜像上 I-7.8 真被评估过且成立；把根环里每一条根的水位压到 `lowered_watermark` 之后**只**红 I-7.8、
/// 红在「盘上出现过的最大树 ID」= `highest_tree_identifier_seen_on_disk` 上，别的判定与干净镜像逐项相同。
pub(super) fn assert_lowering_the_watermark_reddens_only_the_watermark_invariant(
    clean: &MemoryPool,
    lowered_watermark: u64,
    highest_tree_identifier_seen_on_disk: u64,
    step: &str,
) {
    let verdicts_on_the_clean_image = check_pool_image(clean);
    assert_eq!(
        violated_invariants(&verdicts_on_the_clean_image),
        Vec::<&str>::new(),
        "{step}：干净镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
    );
    assert_eq!(
        verdict(clean, "I-7.8"),
        InvariantVerdict::Holds,
        "{step}：干净镜像上 I-7.8 要真被评估过且成立"
    );
    let mut image = clean.clone();
    assert!(
        set_the_tree_identifier_watermark_of_every_root(&mut image, lowered_watermark) > 0,
        "{step}：根环里至少改了一条根"
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-7.8"],
        "{step}：水位压到 {lowered_watermark} 只红 I-7.8：{verdicts:?}"
    );
    assert_eq!(
        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &["I-7.8"]),
        [],
        "{step}：I-7.8 之外每一条的判定与干净镜像逐项相同"
    );
    assert_eq!(
        verdict(&image, "I-7.8"),
        InvariantVerdict::Violated(format!(
            "根环水位最大 {lowered_watermark}，盘上出现过的最大树 ID {highest_tree_identifier_seen_on_disk}"
        )),
        "{step}：红在盘上出现过的最大树 ID {highest_tree_identifier_seen_on_disk} 上"
    );
}

/// 改法 D（I-7.8（根记录树 ID 水位不低于全池最大树 ID） 扫描方向排除「最新根的实例表判得出没发布过」的码 2 节点，
/// `research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y1、第四节第 1 条，被攻过零轮）的「T = 0 的行不排除」那一道
/// 配一份只红 I-7.8 的坏镜像：只做过 mkfs 的池可写挂载（实例 1，暖机），再挂一次（实例 2）发第一个文件版本；
/// 进程退出之后实例 2 的每一条根的根槽都清零，接着在实例 1 那一版上建实例 3：取号、写行 (1, T, 0) 与中间实例行 (2, 0, 0)、暖机——
/// 经事务层的入口直接写（`acquire_instance`、`publish_instance_table_on_version_without_file` 交这里拼好的行、零单元发布），
/// 不经可写挂载：系统配置见证过实例 2 的发布，C554 乙之后可写挂载重读一次仍读不出实例 2 的根就拒，走不到写中间实例行。
/// 实例 2 写出的码 2 节点只由 (2, 0, 0) 那一行说到，行里说不出它发布到哪一代，照旧算「出现过」（D8（核心索引结构） 已定项 8 ②）。
/// 根环里的水位全压到实例 2 发的第一个号，I-7.8 要红在盘上还留着的实例 2 节点里最大的那个号上。干净镜像上一条都不红
/// （实例 3 的水位照可写挂载的取法按环里的记录取，盖得住实例 2 发过的号）。
/// 回退行那一道（① 回退行不排除）随回退行删掉（D18（块里携带什么信息） 已定项 11），没有对象。
#[test]
fn published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark() {
    let mut pool = format_pool("known-bad-watermark-behind-an-intermediate-row");
    let mut devices = pool.reopen_recorded();
    let first_instance =
        mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    assert_eq!(first_instance.output.instance, InstanceGeneration(1));
    let PoolVersion::WithoutFile(version_of_the_first_instance) = &first_instance.current else {
        panic!("只做过 mkfs 的池上实例 1 那一版树表 0 条")
    };
    let newest_root_of_the_first_instance = version_of_the_first_instance.root;
    drop(first_instance);
    let second_instance = mount_writable(&parameters(), &mut devices).expect("再挂一次");
    assert_eq!(second_instance.output.instance, InstanceGeneration(2));
    let PoolVersion::WithoutFile(version) = &second_instance.current else {
        panic!("只做过 mkfs 的池上现行那一版树表 0 条")
    };
    let mut allocator = second_instance.allocator.clone();
    let content: Vec<u8> = (0..3300usize)
        .map(|index| u8::try_from((index * 7 + 17) % 253).expect("小于 256"))
        .collect();
    let first_file_of_the_second_instance = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            second_instance.output.instance,
            &version.record_bytes,
        )
        .expect("实例 2 发第一个文件版本")
    };
    let first_identifier_of_the_second_instance = version.root.tree_identifier_watermark;
    let mut roots_of_the_second_instance: Vec<CheckpointTxg> =
        vec![second_instance.output.row_publish.root().checkpoint_txg];
    roots_of_the_second_instance.extend(
        second_instance
            .output
            .warm_up_publishes
            .iter()
            .map(|warm_up| warm_up.root().checkpoint_txg),
    );
    roots_of_the_second_instance.push(first_file_of_the_second_instance.root.checkpoint_txg);
    drop(devices);
    let mut reopened = pool.reopen_recorded();
    let root_slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("512");
    for txg in &roots_of_the_second_instance {
        let (device, offset) = root_slot_of(*txg);
        let (_, recorded) = reopened
            .iter_mut()
            .find(|(identity, _)| identity.0 == device)
            .expect("根槽所在的盘");
        recorded
            .write_at(
                DeviceOffsetInBytes(offset),
                &vec![0u8; root_slot_bytes],
                singlefs_core::block_device::WriteDurability::Plain,
            )
            .expect("清零实例 2 的根槽");
    }
    let rows_of_the_third_instance = vec![
        InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: newest_root_of_the_first_instance.checkpoint_txg,
            applied_transaction_high_water: 0,
        },
        InstanceRow {
            instance: InstanceGeneration(2),
            selected_root_txg: CheckpointTxg(0),
            applied_transaction_high_water: 0,
        },
    ];
    let newest_root_of_the_third_instance = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
        let third_instance = acquire_instance(&mut writer).expect("取号 3");
        assert_eq!(third_instance, InstanceGeneration(3));
        // 账照可写挂载在实例 1 那一版上重建的样子：那一版分配记录树根指针全零，只有 mkfs 写在单元区里的两个单元；实例 2 的根清零了，
        // 影子账看不见它的单元。
        let mut allocator_rebuilt_on_the_first_instance = PoolAllocator::new(
            DEVICES
                .iter()
                .map(|device| DeviceFreeMap::new(DeviceIdentity(*device), common::IMAGE_BYTES))
                .collect(),
        );
        allocator_rebuilt_on_the_first_instance.mark_format_time_units(
            Placement {
                slot: INSTANCE_TABLE_SLOT,
                span: 2,
            },
            Placement {
                slot: TREE_TABLE_GENESIS_SLOT,
                span: 1,
            },
        );
        // 记根要分配器装着根环表（没装就断言，C551）。这张表只记实例 1 最新那一条根，F 0，下一条根就是写行那次：够这次发布记根用，
        // 不等于可写挂载在实例 1 那一版上读到的环——挂载读到的环里还有 mkfs 写的第 0 代根与实例 1 那次挂载更早发的根
        // （实例 2 的根槽清零了，那几条读不到），
        // 这张表比它少几条根，回收门槛因此偏高（环里最旧有效根取成实例 1 最新那一条）。门槛偏高也不多回收，靠的是这个前提：
        // 这个分配器上没有可回收的已释放记录——它只记着 mkfs 写在单元区里的两个单元、都未释放，写行那次释放的旧链释放代是写行那次的 txg，
        // 比门槛新。所以这一次一个都不回收，账与没装根环表时一样；往这个分配器上加已释放记录之前，先把表换成挂载读到的环。
        let row_publish_txg =
            CheckpointTxg(first_file_of_the_second_instance.root.checkpoint_txg.0 + 1);
        allocator_rebuilt_on_the_first_instance.install_root_ring_occupancy(
            singlefs_core::allocator::RootRingOccupancy::read_from_the_ring(
                parameters().geometry.root_ring_slots_per_region,
                vec![(
                    singlefs_core::root_ring::target_for_publish(
                        newest_root_of_the_first_instance.checkpoint_txg,
                        parameters().geometry.root_ring_slots_per_region,
                    ),
                    singlefs_core::allocator::RootRingOccupant::ValidRoot {
                        checkpoint_txg: newest_root_of_the_first_instance.checkpoint_txg,
                    },
                )],
                CheckpointTxg(0),
                CheckpointTxg(row_publish_txg.0 - 1),
            ),
        );
        let rewrite = InstanceTableRewrite {
            rows: rows_of_the_third_instance,
            replaced_chain: vec![newest_root_of_the_first_instance.instance_table],
        };
        // 写行那次的 txg 与 jsn 照可写挂载接在环里全部根与全部记录的最大值后面（D23（journal 的角色与格式） 已定项 14 第 3 条）：
        // 实例 2 的根清零了、记录还在，最大的是它第一个文件那一版；水位照可写挂载取环里的 max（D8（核心索引结构） 已定项 8 ②）。
        let row_publish = publish_instance_table_on_version_without_file(
            &mut writer,
            &mut allocator_rebuilt_on_the_first_instance,
            &newest_root_of_the_first_instance,
            InstanceTableOnlyPublishPlan {
                txg: row_publish_txg,
                counter: first_file_of_the_second_instance.record.counter + 1,
                instance: third_instance,
                back_chain: 0,
                rollback_floor: CheckpointTxg(0),
                instance_table: &rewrite,
                tree_identifier_watermark: first_file_of_the_second_instance
                    .root
                    .tree_identifier_watermark,
            },
        )
        .expect("实例 3 在实例 1 那一版上写行");
        // 暖机：本实例的根覆盖两块盘之前连推零单元发布（D16（发布语义） 已定项 8 戊，至多根环区域数那么多次）。
        let mut devices_holding_a_root_of_the_third_instance =
            std::collections::BTreeSet::from([root_slot_of(row_publish.root.checkpoint_txg).0]);
        let mut newest = row_publish;
        for _ in 0..parameters().region_devices.len() {
            if DEVICES
                .iter()
                .all(|device| devices_holding_a_root_of_the_third_instance.contains(device))
            {
                break;
            }
            let warm_up = publish_without_units(
                &mut writer,
                &newest.root,
                ZeroUnitPublishPlan {
                    txg: CheckpointTxg(newest.root.checkpoint_txg.0 + 1),
                    counter: newest.record.counter + 1,
                    instance: third_instance,
                    back_chain: back_chain_of(&newest.record_bytes),
                    rollback_floor: CheckpointTxg(0),
                    tree_identifier_watermark: newest.root.tree_identifier_watermark,
                },
            )
            .expect("实例 3 暖机");
            devices_holding_a_root_of_the_third_instance
                .insert(root_slot_of(warm_up.root.checkpoint_txg).0);
            newest = warm_up;
        }
        newest.root
    };
    pool.devices = Some(reopened);
    assert_eq!(
        instance_table_chain_of_root(&pool.memory_pool(), &newest_root_of_the_third_instance)
            .expect("实例 3 最新那条根的实例表读得出")
            .records
            .rows,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: newest_root_of_the_first_instance.checkpoint_txg,
                applied_transaction_high_water: 0,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
            }
        ],
        "盘上实例 3 最新那条根的表里是实例 1 那一行与中间实例行 (2, 0, 0)"
    );
    assert_eq!(
        (
            first_identifier_of_the_second_instance,
            first_file_of_the_second_instance
                .tree_identifiers
                .highest()
                .0
        ),
        (65536, 65548),
        "实例 2 发 65536..65548，其中 65541..65548 只有树表条目、没有单元 ⇒ 盘上扫得出来最大的是映射树 65540"
    );
    assert_lowering_the_watermark_reddens_only_the_watermark_invariant(
        &pool.memory_pool(),
        65536,
        65540,
        "中间实例行后面的实例 2",
    );
}

// ───────── I-9.16 ③ ④、I-5.5 部分没判、I-2.4 根槽留位、带树一版的分配记录树根（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`）─────────

/// 新池新建文件那一版树表里十二条按发号次序排（从 0 数）：分配记录树第 2 条、deadlist 第 6 条、logged_ops 第 10 条、退役树第 11 条（最末）。
pub(super) const ALLOCATION_TREE_TABLE_ENTRY_INDEX: usize = 2;

const DEADLIST_TREE_TABLE_ENTRY_INDEX: usize = 6;

const LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX: usize = 10;

const RETIREMENT_TREE_TABLE_ENTRY_INDEX: usize = 11;

/// 新池新建文件那一版树表里 extent 树那条的树 ID（mkfs 的水位 65536）与 inode 树那条的树 ID（65537）：
/// 改成种类 1 的那一条头 ID 照 extent 条目写成 inode 树的号，I-9.27 不先红。
const INODE_TREE_IDENTIFIER_OF_THE_FIRST_FILE_VERSION: u64 = 65537;

/// 树表里第 `index` 条改种类：先核那一条原来的种类，再改（树 ID、根指针不动）。
fn set_the_kind_of_tree_table_entry(
    tree_table: &mut [u8],
    index: usize,
    kind_before: u16,
    kind_after: u16,
) {
    let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
    assert_eq!(
        u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
        kind_before,
        "树表第 {index} 条原来的种类"
    );
    set_u16(tree_table, kind_offset, kind_after);
}

/// 树表里第 `index` 条改成种类 1（extent 树），头 ID 写成 inode 树的号（种类 1 的头 ID 该等于同表种类 2 条目的树 ID，I-9.27）。
fn turn_tree_table_entry_into_a_second_extent_entry(
    tree_table: &mut [u8],
    index: usize,
    kind_before: u16,
) {
    set_the_kind_of_tree_table_entry(tree_table, index, kind_before, TREE_KIND_EXTENT);
    set_u64(
        tree_table,
        tree_table_entry_offset(index) + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
        INODE_TREE_IDENTIFIER_OF_THE_FIRST_FILE_VERSION,
    );
}

/// 树表里去掉第 `index` 条：它后面的条目往前挪 200、末尾 200 字节清零、条目数与声明长度各减一条
/// （码 2 头：key 宽 1 字节在 51、条目数 2 字节在 82 + 2k、声明长度 2 字节在 8；攻方 `harness-probe-tests.rs` 的 e4 造法）。
fn remove_tree_table_entry(tree_table: &mut [u8], index: usize) {
    let key_width = usize::from(tree_table[51]);
    let count_offset = 82 + 2 * key_width;
    let count = usize::from(u16::from_le_bytes([
        tree_table[count_offset],
        tree_table[count_offset + 1],
    ]));
    let start = tree_table_entry_offset(index);
    let end = tree_table_entry_offset(count);
    tree_table.copy_within(start + 200..end, start);
    tree_table[end - 200..end].fill(0);
    set_u16(
        tree_table,
        count_offset,
        u16::try_from(count - 1).expect("条目数装得进 u16"),
    );
    let declared = u16::from_le_bytes([tree_table[8], tree_table[9]]);
    set_u16(tree_table, 8, declared - 200);
}

/// 一份坏镜像上判红的只有 I-9.16、第一处违例是 `expected_violation`；干净镜像上 I-9.16 真被评估过且成立。
fn assert_only_the_tree_table_ordering_invariant_is_violated(
    clean: &MemoryPool,
    image: &MemoryPool,
    expected_violation: &str,
) {
    assert_eq!(
        verdict(clean, "I-9.16"),
        InvariantVerdict::Holds,
        "干净镜像上 I-9.16 真被评估过且成立"
    );
    let verdicts = check_pool_image(image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-9.16"],
        "判红的该只有 I-9.16：{verdicts:?}"
    );
    assert_eq!(
        verdict(image, "I-9.16"),
        InvariantVerdict::Violated(expected_violation.to_owned()),
        "红在 ③ ④ 那一道"
    );
}

/// 最新根的树表里一条种类 3 都没有（I-9.16 ④ 判红的那块畸形）：I-3.1（已分配统计对得上） 的记录口径无从比，报部分没判、说明点名 I-9.16，
/// 不判红（配不上的那块畸形别的不变量判红，那一处报部分没判）。
fn assert_the_allocated_statistic_is_left_to_the_tree_table_ordering_invariant(image: &MemoryPool) {
    match verdict(image, "I-3.1") {
        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
            unjudged.contains("一条种类 3（分配记录树）都没有（I-9.16 ④ 判它）"),
            "I-3.1 部分没判的说明点名 I-9.16 判的那块畸形：{unjudged}"
        ),
        other @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::NotApplicable(_)) => {
            panic!("树表里没有种类 3：I-3.1 该报部分没判、点名 I-9.16，实际 {other:?}")
        }
    }
}

/// I-9.16 ③（攻方 b2）：livelist 那一条改成种类 1、头 ID 照 extent 条目写成 inode 树的号——树表里种类 1 两条。
/// 树 ID 次序、发号次序（取先出现的那条）、头 ID 都对得上（core 读这一版报「树表里同一种树有两条」）。
#[test]
fn a_tree_table_whose_livelist_entry_became_a_second_extent_entry_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean =
        build_pool("known-bad-tree-table-second-extent-entry-from-the-livelist").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            turn_tree_table_entry_into_a_second_extent_entry(
                bytes,
                LIVELIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_LIVELIST,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表：种类 1 有 2 条（③ 每种至多一条）",
    );
}

/// I-9.16 ③（攻方 b3）：logged_ops 那一条（树表最末一条）改成种类 1、头 ID 写成 inode 树的号。
#[test]
fn a_tree_table_whose_logged_operations_entry_became_a_second_extent_entry_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean = build_pool("known-bad-tree-table-second-extent-entry-from-the-logged-operations")
        .memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            turn_tree_table_entry_into_a_second_extent_entry(
                bytes,
                LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_LOGGED_OPERATIONS,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表：种类 1 有 2 条（③ 每种至多一条）",
    );
}

/// I-9.16 ④（正推 e1）：分配记录那一条的种类改成没登记的码——树表里一条种类 3 都没有。
/// I-3.1 那一块盘上记录口径无从比：报部分没判、点名 I-9.16，不判红。
#[test]
fn a_tree_table_whose_allocation_entry_carries_an_unregistered_kind_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean =
        build_pool("known-bad-tree-table-allocation-entry-of-an-unregistered-kind").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            set_the_kind_of_tree_table_entry(
                bytes,
                ALLOCATION_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_ALLOCATION,
                TREE_KIND_NOT_REGISTERED,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表有 12 条，种类 3 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
    );
    assert_the_allocated_statistic_is_left_to_the_tree_table_ordering_invariant(&image);
}

/// I-9.16 ③（攻方 e2）：deadlist 那一条改成种类 3——树表里种类 3 两条（deadlist 的根是空根，改种类不牵动别的判定）。
#[test]
fn a_tree_table_whose_deadlist_entry_became_a_second_allocation_entry_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean =
        build_pool("known-bad-tree-table-second-allocation-entry-from-the-deadlist").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            set_the_kind_of_tree_table_entry(
                bytes,
                DEADLIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_DEADLIST,
                TREE_KIND_ALLOCATION,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表：种类 3 有 2 条（③ 每种至多一条）",
    );
}

/// I-9.16 ④（攻方 e4）：整条去掉分配记录那一条——树表 10 条、没有种类 3。
/// I-3.1 报部分没判、点名 I-9.16，不判红（同上一份）。
#[test]
fn a_tree_table_without_its_allocation_entry_reddens_only_the_tree_table_ordering_invariant() {
    let clean = build_pool("known-bad-tree-table-without-the-allocation-entry").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            let kind_offset = tree_table_entry_offset(ALLOCATION_TREE_TABLE_ENTRY_INDEX)
                + TREE_TABLE_ENTRY_KIND_OFFSET;
            assert_eq!(
                u16::from_le_bytes([bytes[kind_offset], bytes[kind_offset + 1]]),
                TREE_KIND_ALLOCATION,
                "去掉的是分配记录那一条"
            );
            remove_tree_table_entry(bytes, ALLOCATION_TREE_TABLE_ENTRY_INDEX);
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表有 11 条，种类 3 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
    );
    assert_the_allocated_statistic_is_left_to_the_tree_table_ordering_invariant(&image);
}

/// 树表里去掉第 `index` 条，先核那一条原来的种类是 `kind`。
fn remove_tree_table_entry_of_kind(tree_table: &mut [u8], index: usize, kind: u16) {
    let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
    assert_eq!(
        u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
        kind,
        "树表第 {index} 条原来的种类"
    );
    remove_tree_table_entry(tree_table, index);
}

/// I-9.16 ④ 十二种（正推 C1，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-9.16 ④ 放过缺种类的树表（C1）」
/// 那一行）：整条去掉 deadlist 那一条——树表 11 条、没有种类 8。core 的可写挂载重建这一版时
/// 逐种取号报「树表里没有那棵树」（正推探针 `research/prompts/m3-formal-code-b26-r1-sonnet-model/probe/known_bad_probe.rs.append`）。
#[test]
fn a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant() {
    let clean = build_pool("known-bad-tree-table-without-the-deadlist-entry").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            remove_tree_table_entry_of_kind(
                bytes,
                DEADLIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_DEADLIST,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表有 11 条，种类 8 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
    );
}

/// I-9.16 ④ 十二种：整条去掉退役树那一条（树表最末一条）——树表 11 条、没有种类 13。
/// 走读另红 I-1.1（树表头里的 key 区间末端还是退役树的号），照实钉住。
#[test]
fn a_tree_table_without_its_retirement_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk(
) {
    let clean = build_pool("known-bad-tree-table-without-the-retirement-entry").memory_pool();
    assert_eq!(
        verdict(&clean, "I-9.16"),
        InvariantVerdict::Holds,
        "干净镜像上 I-9.16 真被评估过且成立"
    );
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            remove_tree_table_entry_of_kind(
                bytes,
                RETIREMENT_TREE_TABLE_ENTRY_INDEX,
                singlefs_core::records::TREE_KIND_RETIREMENT,
            );
        },
        true,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        ["I-1.1", "I-9.16"],
        "判红的是 I-9.16 加走读带出来的 I-1.1：{verdicts:?}"
    );
    assert_eq!(
        verdict(&image, "I-9.16"),
        InvariantVerdict::Violated(
            "根（实例 1、txg 3）的树表有 11 条，种类 13 一条都没有（④ 树表非空时进树表的十二种各恰好一条）"
                .to_owned()
        ),
        "红在 ④ 那一道"
    );
}

/// I-9.16 ④ 十二种：整条去掉 logged_ops 那一条（它不是最末一条，树表头里的 key 区间不变）——树表 11 条、没有种类 12，只红 I-9.16。
#[test]
fn a_tree_table_without_its_logged_operations_entry_reddens_only_the_tree_table_ordering_invariant()
{
    let clean =
        build_pool("known-bad-tree-table-without-the-logged-operations-entry").memory_pool();
    let mut image = clean.clone();
    mutate_unit(
        &mut image,
        TREE_TABLE,
        |bytes| {
            remove_tree_table_entry_of_kind(
                bytes,
                LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_LOGGED_OPERATIONS,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 3）的树表有 11 条，种类 12 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
    );
}

/// I-9.16 ③（攻方 f2）：覆盖写之后那一版（txg 4，不是从 mkfs 水位建树的那一版，I-5.5 不认它）的树表里 deadlist 改成种类 3。
/// I-9.16 按每张树表第一次被走到时判，这一张是 B 自己的。
#[test]
fn the_tree_table_after_the_overwrite_whose_deadlist_entry_became_a_second_allocation_entry_reddens_only_the_tree_table_ordering_invariant(
) {
    let clean = image_after_the_overwrite(
        "known-bad-tree-table-after-the-overwrite-second-allocation-entry",
    );
    let mut image = clean.clone();
    mutate_unit_of(
        &mut image,
        &UNITS_AFTER_OVERWRITE,
        TREE_TABLE_AFTER_OVERWRITE,
        |bytes| {
            set_the_kind_of_tree_table_entry(
                bytes,
                DEADLIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_DEADLIST,
                TREE_KIND_ALLOCATION,
            );
        },
        true,
    );
    assert_only_the_tree_table_ordering_invariant_is_violated(
        &clean,
        &image,
        "根（实例 1、txg 4）的树表：种类 3 有 2 条（③ 每种至多一条）",
    );
}
