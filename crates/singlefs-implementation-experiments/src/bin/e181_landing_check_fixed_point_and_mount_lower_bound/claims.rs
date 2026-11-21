use super::*;
use singlefs_core::allocator::PlacementRefusal;

fn first_file_pool(geometry: Geometry) -> Pool {
    build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat)
}

/// A1 与 B2：两块 4 GiB 盘单元区 [50176, 262144)，层级 0 每盘罩 262 个位置（⌊262143 ÷ 812⌋ − ⌊50176 ÷ 812⌋ + 1 = 322 − 61 + 1）、
/// 层级 1 每盘罩 2 个（⌊262143 ÷ 137228⌋ − ⌊50176 ÷ 137228⌋ + 1 = 1 − 0 + 1）、层级 2 每盘 1 个；根层级 1、2、3 的 K1 各是 525、529、531。
pub fn check_that_allocation_record_tree_nodes_of_two_four_gibibyte_devices_are_525_529_and_531_at_root_levels_one_two_and_three(
) {
    let unit_areas = [(50176, 211_968), (50176, 211_968)];
    assert_eq!(
        local_allocation_record_tree_nodes(1, &unit_areas),
        1 + 2 * 262
    );
    assert_eq!(local_allocation_record_tree_nodes(2, &unit_areas), 529);
    assert_eq!(
        local_allocation_record_tree_nodes(3, &unit_areas),
        1 + 2 * (262 + 2 + 1)
    );
    assert_eq!(
        local_allocation_record_tree_nodes(1, &unit_areas),
        525,
        "根层级 1 的分配记录树节点数是 525（手算 1 + 2 × 262）"
    );
    assert_eq!(
        local_allocation_record_tree_nodes(3, &unit_areas),
        531,
        "根层级 3 的分配记录树节点数是 531（手算 1 + 2 × 265）"
    );
}

/// 小几何单元区 [7168, 27648)：层级 0 罩 ⌊27647 ÷ 812⌋ − ⌊7168 ÷ 812⌋ + 1 = 34 − 8 + 1 = 27 个，层级 1 罩 1 个。
pub fn check_that_allocation_record_tree_nodes_of_the_small_geometry_are_55_at_root_level_one_and_57_at_root_level_two(
) {
    let unit_areas = [(7168, 20_480), (7168, 20_480)];
    assert_eq!(local_allocation_record_tree_nodes(1, &unit_areas), 55);
    assert_eq!(local_allocation_record_tree_nodes(2, &unit_areas), 57);
}

/// 手算：每层节点数 [3, 1]、容量 8 / 4、已有 100 条、删 4 插 6：叶层 3 个节点各至少切一次（⌈8 ÷ 2⌉ = 4 条才再切）切 3 次，
/// 重写 min(3, 10) + 3 = 6；内部层 3 + 3 = 6 个孩子 > 4，原有 1 个节点至少切 1 次、其余 3 − 1 = 2 条每 2 条切一次，共 2 次，
/// 重写 min(1, 10) + 2 = 3；根切开长高，新根起步 3 个孩子 ≤ 4，不再切，加 1。合计 6 + 3 + 1 = 10。
pub fn check_that_central_mapping_nodes_rewritten_at_most_pins_the_hand_computed_examples() {
    let capacity = SMALL_TREE_NODE_CAPACITY;
    assert_eq!(
        local_central_mapping_tree_nodes_rewritten_at_most(&[3, 1], 100, 4, 6, capacity),
        10
    );
    // 一片叶、已有 5 条、删 1 插 1：装得下，不切，重写 min(1, 2) = 1。
    assert_eq!(
        local_central_mapping_tree_nodes_rewritten_at_most(&[1], 5, 1, 1, capacity),
        1
    );
    // 一片叶、已有 8 条（满）、插 4：切 1 次，重写 min(1, 4) + 1 = 2，根切开长高加 1 个新根，合计 3。
    assert_eq!(
        local_central_mapping_tree_nodes_rewritten_at_most(&[1], 8, 0, 4, capacity),
        3
    );
    assert_eq!(
        local_splits_at_one_level_at_most(2, 5, 3),
        3,
        "2 + ⌊(5 − 2) ÷ 3⌋ = 3"
    );
}

pub fn check_that_sparse_layers_share_reads_and_keep_writes_apart() {
    let mut base = ForkedDevice::empty(1 << 20);
    let first = vec![7_u8; 1024];
    base.write_at(DeviceOffsetInBytes(512), &first, WriteDurability::Plain)
        .expect("写");
    let mut fork = base.fork();
    let mut read = vec![0_u8; 1024];
    fork.read_at(DeviceOffsetInBytes(512), &mut read)
        .expect("读");
    assert_eq!(read, first, "拷贝读到封存的层");
    fork.write_at(
        DeviceOffsetInBytes(512),
        &[9_u8; 512],
        WriteDurability::Plain,
    )
    .expect("写");
    base.read_at(DeviceOffsetInBytes(512), &mut read)
        .expect("读");
    assert_eq!(read, first, "拷贝上的写不进原来的盘");
    fork.read_at(DeviceOffsetInBytes(512), &mut read)
        .expect("读");
    assert_eq!(&read[..512], &[9_u8; 512][..], "拷贝读到自己的写");
    assert_eq!(&read[512..], &[7_u8; 512][..], "没写的扇区读到封存的层");
}

pub fn check_that_zeroing_masks_the_sealed_layers_and_a_later_write_shows_through_the_mask() {
    let mut base = ForkedDevice::empty(1 << 20);
    base.write_at(
        DeviceOffsetInBytes(0),
        &[5_u8; 2048],
        WriteDurability::Plain,
    )
    .expect("写");
    let mut fork = base.fork();
    fork.write_zeroes_at(DeviceOffsetInBytes(512), 1024)
        .expect("清零");
    let mut read = vec![1_u8; 2048];
    fork.read_at(DeviceOffsetInBytes(0), &mut read).expect("读");
    assert_eq!(&read[..512], &[5_u8; 512][..]);
    assert_eq!(&read[512..1536], &[0_u8; 1024][..], "清零盖住封存的层");
    assert_eq!(&read[1536..], &[5_u8; 512][..]);
    fork.write_at(
        DeviceOffsetInBytes(1024),
        &[3_u8; 512],
        WriteDurability::Plain,
    )
    .expect("写");
    fork.read_at(DeviceOffsetInBytes(0), &mut read).expect("读");
    assert_eq!(&read[1024..1536], &[3_u8; 512][..], "清零之后的写读得到");
    assert_eq!(&read[512..1024], &[0_u8; 512][..]);
    let outcome = fork
        .discard_at(DeviceOffsetInBytes(0), 512)
        .expect("discard");
    assert_eq!(outcome, DiscardOutcome::Applied);
    fork.read_at(DeviceOffsetInBytes(0), &mut read).expect("读");
    assert_eq!(&read[..512], &[0_u8; 512][..], "discard 当整段清零");
    assert!(matches!(
        fork.read_at(DeviceOffsetInBytes(100), &mut read),
        Err(BlockDeviceError::Unaligned { .. })
    ));
    assert!(matches!(
        fork.read_at(DeviceOffsetInBytes(1 << 20), &mut read[..512]),
        Err(BlockDeviceError::OutOfRange { .. })
    ));
}

/// A3 与 A4：mkfs 同一个进程里写完第一个文件（3000 字节）之后，第一个数据单元在 50180，每块盘最低落点 50182，
/// 盘 0 上账里记录的最大末槽（不含）是 50253（D3 已定项 10 ⑤ 落点表：树表单元落 50252）。
pub fn check_that_the_first_file_lands_at_50180_and_its_records_end_at_50253() {
    let pool = first_file_pool(GEOMETRY_DEFAULT);
    let first_unit = pool
        .current
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .expect("第一个数据单元");
    assert_eq!(first_unit.slot, SlotNumber(50180));
    for device_map in &pool.allocator.devices {
        assert_eq!(
            device_map.lowest_user_data_slot(pool.allocator.cluster_segments()),
            Some(SlotNumber(50182))
        );
    }
    let largest_end = pool
        .current
        .allocation_records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0))
        .map(|record| record.slot.0 + u64::from(record.span_slots))
        .max();
    assert_eq!(largest_end, Some(50253));
}

/// 第一个文件那次发布：普通分配 6 槽（数据 2 + extent 树根 1 + inode 叶容器 2 + inode 树根 1），固定点 8 槽
/// （分配记录树 5 个节点 + 记账树 1 + 中央映射树 1 + 树表 1，D3 已定项 10 ⑤ 落点表 t5–t12）；两条路径（角色、记录）逐盘相等。
pub fn check_that_the_first_file_publish_has_six_ordinary_and_eight_fixed_point_slots_by_both_paths(
) {
    let pool = first_file_pool(GEOMETRY_DEFAULT);
    let account = fixed_point_account(&pool.current, &device_identities(&pool));
    assert_eq!(account.ordinary_slots, 6);
    assert_eq!(account.by_roles, 8);
    assert_eq!(account.instance_switch_slots, 0);
    assert_eq!(
        account.by_records_per_device,
        vec![(DeviceIdentity(0), 8), (DeviceIdentity(1), 8)]
    );
}

/// B3：本地 ckpt_cost（普通分配删插为 0）与核心层的同名量在第一个文件那一版、一次换写之后的一版上逐个相等，三个几何、两档容量。
pub fn check_that_the_local_checkpoint_cost_equals_the_core_one_on_every_version_measured() {
    let mut content = ContentBuffer::new();
    for (geometry, capacities) in [
        (
            GEOMETRY_DEFAULT,
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
        ),
        (
            GEOMETRY_NARROW,
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
        ),
        (GEOMETRY_SMALL, CodeTwoTreeNodeCapacities::FromTheNodeFormat),
        (GEOMETRY_DEFAULT, SMALL_TREE_NODE_CAPACITIES),
    ] {
        let mut pool = build_pool_after_the_first_file(geometry, capacities);
        for units in [3_usize, 70, 200] {
            let core = core_checkpoint_cost(&pool.current, &pool.allocator, capacities);
            let local = local_checkpoint_cost(
                &pool.current,
                &pool.allocator,
                capacities,
                OrdinaryMappingChanges::default(),
            );
            assert_eq!(local.total, core, "{} 容量 {capacities:?}", geometry.name);
            let outcome = write_units_and_account(&mut pool, units, capacities, &mut content)
                .expect("本地与核心层的 checkpoint 代价对拍过");
            assert!(matches!(outcome, WriteOutcome::Published(_)));
        }
    }
}

/// 这批普通分配的删插：第一个文件那一版有 4 个进映射的普通分配单元（数据、extent 树根、inode 叶容器、inode 树根），
/// 换写成 70 个单元后全部换下（删 4），插的 = 这次重写的角色里进映射的普通分配角色数。
pub fn check_that_a_growth_write_deletes_the_four_first_file_units_and_inserts_every_rewritten_mapped_ordinary_role(
) {
    let mut pool = first_file_pool(GEOMETRY_DEFAULT);
    let mut content = ContentBuffer::new();
    let previous = pool.current.clone();
    let outcome = write_units_and_account(
        &mut pool,
        70,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
        &mut content,
    )
    .expect("对拍过");
    let WriteOutcome::Published(write) = outcome else {
        panic!("70 个单元的写发得成");
    };
    assert_eq!(write.ordinary.deleted, 4);
    let mapped_ordinary_roles: BTreeSet<TransactionUnit> = pool
        .current
        .mapped_units
        .iter()
        .map(|(role, _key)| *role)
        .filter(|role| space_budget_of_role(*role) == SpaceBudgetOfARole::OrdinaryAllocation)
        .collect();
    let rewritten_and_mapped = pool
        .current
        .rewritten
        .iter()
        .filter(|role| mapped_ordinary_roles.contains(role))
        .count();
    assert_eq!(
        write.ordinary.inserted,
        u64::try_from(rewritten_and_mapped).expect("条数")
    );
    assert!(
        write.ordinary.inserted >= 70,
        "至少 70 个数据单元的映射条目"
    );
    assert_eq!(
        ordinary_mapping_changes(&previous, &pool.current),
        write.ordinary
    );
}

/// 压小容量的三层树上，一次加 64 个单元的写：P1 比 ckpt_cost 多（Q3 > 0），因为中央映射树每层多了普通分配的删插；
/// 去掉 P1 里普通分配的删插，P1 就退回 ckpt_cost（Q3 = 0）。
pub fn check_that_a_sixty_four_unit_write_on_the_small_capacity_trees_makes_the_checkpoint_cost_with_ordinary_mapping_changes_exceed_the_checkpoint_cost(
) {
    let mut pool = build_pool_after_the_first_file(GEOMETRY_DEFAULT, SMALL_TREE_NODE_CAPACITIES);
    let mut content = ContentBuffer::new();
    let outcome = write_units_and_account(&mut pool, 65, SMALL_TREE_NODE_CAPACITIES, &mut content)
        .expect("对拍过");
    let WriteOutcome::Published(write) = outcome else {
        panic!("65 个单元的写发得成");
    };
    assert!(
        write.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost() > 0,
        "{write:?}"
    );
    assert_eq!(write.local_checkpoint_cost, write.checkpoint_cost);
    assert!(
        write.checkpoint_cost_with_ordinary_mapping_changes_minus_true() >= 0,
        "带普通分配删插的 checkpoint 代价在这一次写上是上界：{write:?}"
    );
}

/// 小几何、压小容量的树上写了 1500 个单元之后，中央映射树的叶比 ckpt_cost 自己的删插数（K1 加记账树节点，几十把）多得多：
/// 这时多删的普通分配条目、多插的普通分配条目各自都让本地 ckpt_cost 变大（每层可能改的路径数是 min(这一层的节点数, 删 + 插)，插的还多切节点）。
pub fn check_that_extra_deletes_and_inserts_each_raise_the_local_checkpoint_cost_on_a_wide_tree() {
    let mut pool = build_pool_after_the_first_file(GEOMETRY_SMALL, SMALL_TREE_NODE_CAPACITIES);
    let mut content = ContentBuffer::new();
    let outcome =
        write_units_and_account(&mut pool, 1500, SMALL_TREE_NODE_CAPACITIES, &mut content)
            .expect("对拍过");
    assert!(matches!(outcome, WriteOutcome::Published(_)));
    let cost = |extra: OrdinaryMappingChanges| {
        local_checkpoint_cost(
            &pool.current,
            &pool.allocator,
            SMALL_TREE_NODE_CAPACITIES,
            extra,
        )
        .total
    };
    let without_extra = cost(OrdinaryMappingChanges::default());
    let with_deletes = cost(OrdinaryMappingChanges {
        deleted: 1000,
        inserted: 0,
    });
    let with_inserts = cost(OrdinaryMappingChanges {
        deleted: 0,
        inserted: 1000,
    });
    assert!(
        with_deletes > without_extra,
        "多删 1000 条：{with_deletes} 对 {without_extra}"
    );
    assert!(
        with_inserts > without_extra,
        "多插 1000 条：{with_inserts} 对 {without_extra}"
    );
}

pub fn check_that_the_judges_flip_when_the_threshold_moves_between_two_readings() {
    let mut upper = WriteTally::default();
    let mut lower = WriteTally::default();
    let sample = |checkpoint_cost_with_ordinary_mapping_changes_minus_true: i64| PublishedWrite {
        units: 1,
        height_before: 1,
        height_after: 1,
        true_fixed_point: 10,
        true_fixed_point_by_records: Vec::new(),
        checkpoint_cost: 10,
        local_checkpoint_cost: 10,
        checkpoint_cost_with_ordinary_mapping_changes: u64::try_from(
            10 + checkpoint_cost_with_ordinary_mapping_changes_minus_true,
        )
        .expect("非负"),
        ordinary: OrdinaryMappingChanges::default(),
    };
    upper.add(&sample(3));
    lower.add(&sample(0));
    assert_eq!(
        checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(&upper),
        Some(true)
    );
    assert_eq!(
        checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(&lower),
        Some(true)
    );
    assert_eq!(
        checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_given_threshold(&upper, 1),
        Some(true)
    );
    assert_eq!(
        checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_given_threshold(&lower, 1),
        Some(false),
        "门槛挪到两个读数（3 与 0）之间，0 那一边必须翻"
    );
    assert_eq!(
        checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(
            &WriteTally::default()
        ),
        None
    );
    let mut below = WriteTally::default();
    below.add(&sample(-1));
    assert_eq!(
        checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(&below),
        Some(false)
    );
    assert_eq!(
        below.checkpoint_cost_with_ordinary_mapping_changes_below_true_writes,
        1
    );
    assert!(slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(1, 1));
    assert!(!slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(0, 1));
    assert!(
        !slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(1, 2),
        "门槛挪到 1 与 2 之间，1 个状态那一边必须翻"
    );
}

pub fn check_that_slots_enough_and_aligned_pairs_short_holds_only_when_the_slots_are_enough_everywhere_and_the_aligned_pairs_are_short_somewhere(
) {
    let reading = |aligned: [u64; 2], unblocked: [u64; 2]| RoomReading {
        aligned_free_pairs: aligned.to_vec(),
        unblocked_slots: unblocked.to_vec(),
        needed_slots: 400,
        needed_pairs: 1,
    };
    assert!(reading([0, 5], [500, 500]).holds());
    assert!(
        reading([0, 0], [400, 400]).holds(),
        "没挡的槽恰等于要的槽，够"
    );
    assert!(
        !reading([0, 0], [399, 500]).holds(),
        "一块盘的槽差 1 个，不够"
    );
    assert!(
        !reading([1, 1], [500, 500]).holds(),
        "对齐的空槽对恰等于要的个数，够"
    );
    assert!(reading([0, 1], [500, 500]).aligned_pairs_are_short_on_some_device());
    assert!(!reading([1, 2], [500, 500]).aligned_pairs_are_short_on_some_device());
}

pub fn check_that_marking_every_pair_outside_the_clusters_leaves_no_lowest_slot_and_marking_none_leaves_50182(
) {
    let mut pool = first_file_pool(GEOMETRY_DEFAULT);
    let untouched = unequal_landing_reading(&pool);
    assert_eq!(
        untouched.lowest_slot_per_device,
        vec![Some(50182), Some(50182)]
    );
    assert!(!untouched.lowest_slots_differ);
    assert!(untouched.one_data_unit_lands);
    let scan_before = time_the_lowest_user_data_slot(&pool.allocator, 0, 3);
    assert_eq!(scan_before.lowest_slot, Some(50182));
    assert_eq!(
        scan_before.scanned_candidates,
        (50182 - 50176) / 2 + 1,
        "从单元区起点按 2 槽步进到 50182：50176、50178、50180、50182 四个候选"
    );
    mark_the_lowest_pairs_outside_the_clusters(&mut pool.allocator, 100);
    let full = unequal_landing_reading(&pool);
    assert_eq!(
        full.lowest_slot_per_device,
        vec![None, None],
        "开放段里还有空槽对，但落点不进开过的段"
    );
    assert!(
        !full.one_data_unit_lands,
        "段外一对都不剩，一个数据单元也落不下"
    );
    let scan = time_the_lowest_user_data_slot(&pool.allocator, 0, 3);
    assert_eq!(scan.lowest_slot, None);
    assert_eq!(scan.scanned_candidates, 211_968 / 2, "一直扫到单元区末尾");
}

/// 对齐的空槽对数把聚簇段内外都算：开放段 [50240, 50304) 里还空着的槽对多出来。
pub fn check_that_aligned_free_pairs_count_the_open_segment_too() {
    let mut pool = first_file_pool(GEOMETRY_DEFAULT);
    let mut state = session_pool_of(&mut pool).expect("可写挂载");
    let reading = RoomReading::of(&state);
    let device_map = &state.session.allocator.devices[0];
    let mut free_pairs_by_scan = 0;
    let first = device_map.unit_area_start().slot().0;
    let mut slot = first;
    while slot + 1 < first + device_map.unit_area_slots() {
        if device_map.is_free(SlotNumber(slot)) && device_map.is_free(SlotNumber(slot + 1)) {
            free_pairs_by_scan += 1;
        }
        slot += 2;
    }
    assert_eq!(
        reading.aligned_free_pairs[0], free_pairs_by_scan,
        "逐槽扫一遍数出来的"
    );
    let outside = data_unit_pairs_outside_the_clusters_of(&state.session.allocator, 0);
    assert!(
        reading.aligned_free_pairs[0] > outside,
        "开放段里有空槽对：{} 对 {outside}",
        reading.aligned_free_pairs[0]
    );
    assert!(state.created_inodes.is_empty());
    let result = apply_walk_action(&mut state, WalkAction::CreateOneInode).expect("走法");
    assert_eq!(result, "published");
    assert_eq!(state.created_inodes.len(), 1);
}

pub fn check_that_the_seeded_stream_repeats_for_a_seed_and_differs_between_seeds() {
    let mut first = SeededStream::of_seed(1);
    let mut again = SeededStream::of_seed(1);
    let mut other = SeededStream::of_seed(2);
    let drawn: Vec<u64> = (0..8).map(|_| first.below(5)).collect();
    let repeated: Vec<u64> = (0..8).map(|_| again.below(5)).collect();
    let different: Vec<u64> = (0..8).map(|_| other.below(5)).collect();
    assert_eq!(drawn, repeated);
    assert_ne!(drawn, different);
    assert!(drawn.iter().all(|number| *number < 5));
}

pub fn check_that_refusals_are_named_by_the_gate_that_refused() {
    let placement = PublishError::PlacementRefused {
        unit: TransactionUnit::TreeTable,
        refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
    };
    assert_eq!(
        RefusalClass::of(&placement).name(),
        "placement_refused_in_the_fixed_point"
    );
    assert!(passed_the_landing_check_and_the_formula(
        &WriteOutcome::Refused(RefusalClass::of(&placement))
    ));
    assert!(!passed_the_landing_check_and_the_formula(
        &WriteOutcome::Refused(RefusalClass::SpaceAdmission {
            short_devices: String::new(),
            shortfall_bytes: 0
        })
    ));
    assert_eq!(
        variant_name_of("refused_NoSpace(Box { .. })"),
        "refused_NoSpace"
    );
    assert_eq!(signed_difference(3, 5), -2);
}

/// 被拒之后下一次试几个单元：journal 记录数超限减去超出的条数（登记 12.4 的读数：65877 个单元发了 66167 条、上限 65536，超出 631），
/// 缺口按一个单元在一块盘上的 32768 字节折算、向上取整，落点那一道折半；结果夹进 [1, 上一次 − 1]，单元数严格减小。
pub fn check_that_the_units_to_try_after_a_refusal_follow_the_surplus_the_shortfall_or_half() {
    let over_the_limit = RefusalClass::Other {
        description: String::new(),
        journal_records_over_the_limit: Some(631),
    };
    assert_eq!(
        units_to_try_after_a_refusal(&over_the_limit, 65_877),
        65_246
    );
    let without_a_reading = RefusalClass::Other {
        description: String::new(),
        journal_records_over_the_limit: None,
    };
    assert_eq!(units_to_try_after_a_refusal(&without_a_reading, 100), 50);
    let in_placement = RefusalClass::FixedPointPlacement {
        unit: String::new(),
        refusal: String::new(),
    };
    assert_eq!(units_to_try_after_a_refusal(&in_placement, 100), 50);
    let short = |shortfall_bytes: i128| RefusalClass::SpaceAdmission {
        short_devices: String::new(),
        shortfall_bytes,
    };
    assert_eq!(units_to_try_after_a_refusal(&short(32_768), 1000), 999);
    assert_eq!(units_to_try_after_a_refusal(&short(32_769), 1000), 998);
    assert_eq!(units_to_try_after_a_refusal(&short(3 * 32_768), 1000), 997);
    assert_eq!(
        units_to_try_after_a_refusal(&short(0), 1000),
        999,
        "缺口读数不是正数时按一字节算"
    );
    assert_eq!(
        units_to_try_after_a_refusal(&short(40_000_000), 10),
        1,
        "算出负数时夹到 1"
    );
    assert_eq!(
        units_to_try_after_a_refusal(&over_the_limit, 400),
        1,
        "超出的条数比单元数还多时夹到 1"
    );
    assert_eq!(units_to_try_after_a_refusal(&over_the_limit, 2), 1);
    assert_eq!(units_to_try_after_a_refusal(&without_a_reading, 3), 1);
}

/// 填满（登记 12.11）在小几何上：每次发布至多 3000 个单元，所以要好几次发布才填得下；换下的上一批仍占着槽
/// （各盘 defer 队列里至少有最早那次发布的 2 槽 × 单元数）；账里剩下的各盘段外对齐空槽对数等于分配器此刻的读数；
/// 停在「段外一个对齐的空槽对都没剩」或「核心层拒了一个单元的写」：后一种在账里的状态上再发一个单元的写，照样被拒。
pub fn check_that_a_capped_fill_of_the_small_geometry_publishes_several_times_and_ends_with_nothing_left_or_a_refused_single_unit(
) {
    let mut content = ContentBuffer::new();
    let mut pool = first_file_pool(GEOMETRY_SMALL);
    let report = fill_the_data_side(&mut pool, &mut content, 3000).expect("填满不触发停机");
    assert!(report.publishes.len() >= 3, "{report:?}");
    assert!(
        report.publishes.iter().all(|publish| publish.units >= 1
            && publish.units <= publish.units_asked
            && publish.units_asked <= 3000),
        "{report:?}"
    );
    assert_eq!(
        report.remaining_pairs_outside_the_clusters,
        remaining_pairs_outside_the_clusters(&pool),
        "账里剩的槽对数是分配器此刻的读数"
    );
    let slots_of_the_earliest_batch = DATA_UNIT_SLOTS_LOCAL * report.publishes[0].units;
    assert!(
        report
            .deferred_slots
            .iter()
            .all(|deferred| *deferred >= slots_of_the_earliest_batch),
        "换下的最早那一批仍占着槽：{report:?}"
    );
    let fewest = report
        .remaining_pairs_outside_the_clusters
        .iter()
        .copied()
        .min()
        .expect("至少一块盘");
    match &report.stop {
        FillStop::NoAlignedPairOutsideTheClusters => assert_eq!(fewest, 0, "{report:?}"),
        FillStop::Refused { units_asked, .. } => {
            assert!(fewest > 0, "{report:?}");
            assert_eq!(*units_asked, 1, "连一个单元也落不下才停：{report:?}");
            let mut probe = pool.fork();
            let outcome = write_units_and_account(
                &mut probe,
                1,
                CodeTwoTreeNodeCapacities::FromTheNodeFormat,
                &mut content,
            )
            .expect("对拍过");
            assert!(matches!(outcome, WriteOutcome::Refused(_)), "{report:?}");
        }
        FillStop::PublishLimitReached => panic!("发布次数到了保险丝：{report:?}"),
    }
}

fn published_outcome() -> WriteOutcome {
    WriteOutcome::Published(Box::new(PublishedWrite {
        units: 1,
        height_before: 1,
        height_after: 1,
        true_fixed_point: 0,
        true_fixed_point_by_records: Vec::new(),
        checkpoint_cost: 0,
        local_checkpoint_cost: 0,
        checkpoint_cost_with_ordinary_mapping_changes: 0,
        ordinary: OrdinaryMappingChanges::default(),
    }))
}

fn refused_by_space(shortfall_bytes: i128) -> WriteOutcome {
    WriteOutcome::Refused(RefusalClass::SpaceAdmission {
        short_devices: String::new(),
        shortfall_bytes,
    })
}

fn refused_in_the_fixed_point() -> WriteOutcome {
    WriteOutcome::Refused(RefusalClass::FixedPointPlacement {
        unit: String::new(),
        refusal: String::new(),
    })
}

fn refused_without_a_gap() -> WriteOutcome {
    WriteOutcome::Refused(RefusalClass::Other {
        description: String::new(),
        journal_records_over_the_limit: None,
    })
}

/// 合成的盘面：单元数不超过 `edge` 减 3 的写发成，`edge` 减 2 到 `edge` 的三个被拒在固定点（也算放行），更大的被空间准入拒，
/// 缺口 = 超出的单元数 × 32768 × `bytes_per_excess_unit_factor` − 5 字节（减 5 是为了折单元数时向上取整才对得上）。
fn outcome_on_a_synthetic_base_with_a_gap(
    units: usize,
    edge: usize,
    bytes_per_excess_unit_factor: i128,
) -> WriteOutcome {
    if units + 3 <= edge {
        published_outcome()
    } else if units <= edge {
        refused_in_the_fixed_point()
    } else {
        let excess = i128::try_from(units - edge).expect("单元数装得进 i128");
        refused_by_space(excess * 32_768 * bytes_per_excess_unit_factor - 5)
    }
}

/// 同上，被拒的都不带缺口读数（别的拒因）。
fn outcome_on_a_synthetic_base_without_a_gap(units: usize, edge: usize) -> WriteOutcome {
    if units + 3 <= edge {
        published_outcome()
    } else if units <= edge {
        refused_in_the_fixed_point()
    } else {
        refused_without_a_gap()
    }
}

/// 找下界（登记 12.12）：先在填过的底上试填过的单元数，不把它直接当放行；被拒按拒因往下找，到第一个放行的数；
/// 固定点那一道拒也算放行（与二分同一个判法）；一路试到 1 个单元、或试满次数都没放行，交 `NotFound`。
pub fn check_that_the_lower_bound_is_tried_on_the_filled_base_first_and_follows_the_refusals_down_to_a_passing_count(
) {
    {
        let mut tried = Vec::new();
        let found = find_a_lower_bound_that_passes(1000, |units| {
            tried.push(units);
            Ok(published_outcome())
        })
        .expect("试写不出错");
        let LowerBoundSearch::Found {
            units,
            refused_before,
        } = found
        else {
            panic!("填过的单元数放行，该找到");
        };
        assert_eq!((units, refused_before.len()), (1000, 0));
        assert_eq!(tried, vec![1000], "填过的单元数自己先试一次");
    }
    {
        let mut tried = Vec::new();
        let found = find_a_lower_bound_that_passes(1000, |units| {
            tried.push(units);
            Ok(if units > 997 {
                refused_by_space(3 * 32_768)
            } else {
                published_outcome()
            })
        })
        .expect("试写不出错");
        let LowerBoundSearch::Found {
            units,
            refused_before,
        } = found
        else {
            panic!("按缺口一步跳到 997，该找到");
        };
        assert_eq!(units, 997, "缺口 3 个单元：1000 减 3");
        assert_eq!(refused_before.len(), 1);
        assert_eq!(refused_before[0].0, 1000);
        assert_eq!(refused_before[0].1.name(), "space_admission_refused");
        assert_eq!(tried, vec![1000, 997]);
    }
    {
        let found = find_a_lower_bound_that_passes(1000, |_units| Ok(refused_in_the_fixed_point()))
            .expect("试写不出错");
        let LowerBoundSearch::Found {
            units,
            refused_before,
        } = found
        else {
            panic!("被拒在固定点算放行，该找到");
        };
        assert_eq!((units, refused_before.len()), (1000, 0));
    }
    {
        let found = find_a_lower_bound_that_passes(1000, |_units| Ok(refused_without_a_gap()))
            .expect("试写不出错");
        let LowerBoundSearch::NotFound { reason, refused } = found else {
            panic!("一路折半到 1 个单元都被拒，不该找到");
        };
        let refused_units: Vec<usize> = refused.iter().map(|(units, _)| *units).collect();
        assert_eq!(
            refused_units,
            vec![1000, 500, 250, 125, 62, 31, 15, 7, 3, 1],
            "没有缺口读数时每次折半，试到 1 个单元为止"
        );
        assert!(reason.contains("1 个单元"), "{reason}");
    }
    {
        let found = find_a_lower_bound_that_passes(1000, |_units| Ok(refused_by_space(1)))
            .expect("试写不出错");
        let LowerBoundSearch::NotFound { reason, refused } = found else {
            panic!("缺口只有一字节、每次只少 1 个单元，试满次数都没放行，不该找到");
        };
        assert_eq!(refused.len(), 40, "试满 40 次停");
        assert_eq!(
            refused[39].0, 961,
            "1000 起每次少 1 个单元，第 40 次试的是 961"
        );
        assert!(reason.contains("40 次"), "{reason}");
    }
}

/// 二分（登记 12.12）：缺口读数线性时，按最近一次空间准入拒的缺口预测边，一轮就夹住；预测错了（缺口随单元数不是线性）仍收敛到同一个边；
/// 没有缺口读数时退回等分，也收敛。夹住的意思是下界放行、下界 + 1 被拒，两样都是观测到的。
pub fn check_that_the_edge_search_predicts_the_edge_from_the_shortfall_and_closes_it_with_observed_points(
) {
    let edge = 4000;
    {
        let mut tried_one = Vec::new();
        let mut points_of_the_rounds: Vec<Vec<usize>> = Vec::new();
        let search = search_for_the_edge(
            4600,
            5600,
            2,
            |units| {
                tried_one.push(units);
                Ok(outcome_on_a_synthetic_base_with_a_gap(units, edge, 1))
            },
            |points| {
                points_of_the_rounds.push(points.to_vec());
                Ok(points
                    .iter()
                    .map(|units| {
                        (
                            *units,
                            outcome_on_a_synthetic_base_with_a_gap(*units, edge, 1),
                        )
                    })
                    .collect())
            },
        )
        .expect("试写不出错");
        assert_eq!(
            search.end,
            EdgeSearchEnd::Edge {
                boundary_units: 4000
            }
        );
        assert_eq!(search.lower_bound_units, Some(4000));
        assert_eq!(
            tried_one,
            vec![4600, 4000],
            "4600 被拒，缺口 600 个单元，一步跳到 4000"
        );
        assert_eq!(
            points_of_the_rounds,
            vec![vec![4001, 4200]],
            "预测的边 4000 已是下界，试它上面一格 4001，余下的线程等分 (4000, 4600)"
        );
        assert_eq!(
            (
                search.bisection_rounds,
                search.bisection_points_tried,
                search.bisection_points_passed
            ),
            (1, 2, 0)
        );
    }
    {
        let search = search_for_the_edge(
            4600,
            5600,
            2,
            |units| Ok(outcome_on_a_synthetic_base_with_a_gap(units, edge, 2)),
            |points| {
                Ok(points
                    .iter()
                    .map(|units| {
                        (
                            *units,
                            outcome_on_a_synthetic_base_with_a_gap(*units, edge, 2),
                        )
                    })
                    .collect())
            },
        )
        .expect("试写不出错");
        assert_eq!(
            search.end,
            EdgeSearchEnd::Edge {
                boundary_units: 4000
            },
            "缺口随单元数涨得比线性快一倍，预测落在边下面，二分仍夹住同一个边"
        );
        assert_eq!(
            search.lower_bound_units,
            Some(3400),
            "预测错的一步：4600 减 1200"
        );
    }
    {
        let mut points_of_the_rounds: Vec<Vec<usize>> = Vec::new();
        let search = search_for_the_edge(
            4600,
            5600,
            2,
            |units| Ok(outcome_on_a_synthetic_base_without_a_gap(units, edge)),
            |points| {
                points_of_the_rounds.push(points.to_vec());
                Ok(points
                    .iter()
                    .map(|units| {
                        (
                            *units,
                            outcome_on_a_synthetic_base_without_a_gap(*units, edge),
                        )
                    })
                    .collect())
            },
        )
        .expect("试写不出错");
        assert_eq!(
            search.end,
            EdgeSearchEnd::Edge {
                boundary_units: 4000
            },
            "没有缺口读数就等分"
        );
        assert_eq!(
            search.lower_bound_units,
            Some(2300),
            "4600 被拒、折半到 2300 放行"
        );
        assert_eq!(
            points_of_the_rounds[0],
            vec![3066, 3833],
            "(2300, 4600) 三等分"
        );
        assert!(
            points_of_the_rounds.len() > 1
                && points_of_the_rounds.iter().all(|points| points.len() <= 2),
            "等分要好几轮，每轮不超过线程数：{points_of_the_rounds:?}"
        );
    }
}

/// 二分一轮试哪些点：都落在 (下界, 上界) 里，有预测的边时先取预测的边与它上面一格，余下的线程等分。
pub fn check_that_a_bisection_round_tries_the_predicted_edge_and_the_next_count_before_splitting_evenly(
) {
    assert_eq!(
        points_of_a_bisection_round(4000, 4600, 2, Some(4000)),
        vec![4001, 4200]
    );
    assert_eq!(
        points_of_a_bisection_round(3900, 4600, 2, Some(4000)),
        vec![4000, 4001]
    );
    assert_eq!(
        points_of_a_bisection_round(3900, 4600, 1, Some(4000)),
        vec![4000],
        "一个线程只试预测的边"
    );
    assert_eq!(
        points_of_a_bisection_round(4000, 4600, 2, None),
        vec![4200, 4400]
    );
    assert_eq!(
        points_of_a_bisection_round(4000, 4600, 2, Some(3990)),
        vec![4200, 4400],
        "预测的边在下界之下，用不上，等分"
    );
    assert_eq!(
        points_of_a_bisection_round(4000, 4002, 2, None),
        vec![4001],
        "只剩一个点可试"
    );
    assert_eq!(points_of_a_bisection_round(4000, 4002, 1, None), vec![4001]);
}

/// 边没被观测夹住时不交边界（登记 12.12）：一直放行到上限没有被拒的；被拒与放行的次序不一致；下界一个也没找到。
pub fn check_that_an_edge_that_no_observation_brackets_is_reported_as_not_found() {
    {
        let search = search_for_the_edge(
            100,
            110,
            2,
            |_units| Ok(published_outcome()),
            |points| {
                Ok(points
                    .iter()
                    .map(|units| (*units, published_outcome()))
                    .collect())
            },
        )
        .expect("试写不出错");
        let EdgeSearchEnd::NotFound { reason } = &search.end else {
            panic!("一直放行到上限，不该交边界：{search:?}");
        };
        assert!(reason.contains("上限 110"), "{reason}");
    }
    {
        let search = search_for_the_edge(
            3000,
            3010,
            2,
            |_units| Ok(published_outcome()),
            |points| {
                Ok(points
                    .iter()
                    .map(|units| {
                        (
                            *units,
                            if *units == 3003 {
                                refused_without_a_gap()
                            } else {
                                published_outcome()
                            },
                        )
                    })
                    .collect())
            },
        )
        .expect("试写不出错");
        let EdgeSearchEnd::NotFound { reason } = &search.end else {
            panic!("3003 被拒而 3006 放行，次序不一致，不该交边界：{search:?}");
        };
        assert!(reason.contains("3003"), "{reason}");
    }
    {
        let search = search_for_the_edge(
            3,
            100,
            2,
            |_units| Ok(refused_without_a_gap()),
            |_points| panic!("下界没找到，不该再二分"),
        )
        .expect("试写不出错");
        let EdgeSearchEnd::NotFound { reason } = &search.end else {
            panic!("下界一个也没找到，不该交边界：{search:?}");
        };
        assert!(reason.contains("1 个单元"), "{reason}");
        assert_eq!(search.lower_bound_units, None);
        let refused_units: Vec<usize> = search
            .lower_bound_refused_before
            .iter()
            .map(|(units, _)| *units)
            .collect();
        assert_eq!(refused_units, vec![3, 1]);
    }
}

/// 验收（登记 12.12）：小几何填到段外槽对数的六成之后，同样大小的整份换写在这个底上被空间准入那一道拒；
/// 下界不能是填过的单元数，要落在观测到放行的数上：先试的就是填过的单元数、被拒的是空间那一道、边界比它小，
/// 边界在独立的一次试写里放行、边界 + 1 在独立的一次试写里不放行。
pub fn check_that_a_small_base_filled_to_six_tenths_gets_an_edge_below_the_filled_units_that_independent_probes_confirm(
) {
    let mut content = ContentBuffer::new();
    let mut base = first_file_pool(GEOMETRY_SMALL);
    let pairs = data_unit_pairs_outside_the_clusters(&base, 0);
    let filled_units = usize::try_from(pairs * 6 / 10).expect("单元数");
    let filled = write_units_and_account(
        &mut base,
        filled_units,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
        &mut content,
    )
    .expect("对拍过");
    assert!(
        matches!(filled, WriteOutcome::Published(_)),
        "填到六成该发成：{filled:?}"
    );
    base.seal();
    let upper_sentinel = usize::try_from(pairs).expect("槽对数") + 1024;
    let content_bytes: &[u8] = content.of_units(upper_sentinel);
    let search = search_for_the_edge(
        filled_units,
        upper_sentinel,
        2,
        |units| probe_one_write_on_the_sealed_base(&base, units, content_bytes),
        |points| probe_writes_in_parallel(&base, points, content_bytes),
    )
    .expect("试写对拍过");
    let EdgeSearchEnd::Edge { boundary_units } = &search.end else {
        panic!("该找到边：{search:?}");
    };
    let boundary_units = *boundary_units;
    let refusal_at_the_filled_units = search
        .lower_bound_refused_before
        .first()
        .unwrap_or_else(|| panic!("填过的单元数在填过的底上该被拒：{search:?}"));
    assert_eq!(
        refusal_at_the_filled_units.0, filled_units,
        "先试的是填过的单元数：{search:?}"
    );
    assert_eq!(
        refusal_at_the_filled_units.1.name(),
        "space_admission_refused",
        "被拒的是空间准入那一道：{search:?}"
    );
    assert!(boundary_units < filled_units, "{search:?}");
    let lower_bound = search.lower_bound_units.expect("找到了边就有下界");
    assert!(
        lower_bound <= boundary_units && lower_bound < filled_units,
        "{search:?}"
    );
    let at_the_edge =
        probe_one_write_on_the_sealed_base(&base, boundary_units, content_bytes).expect("对拍过");
    assert!(
        passed_the_landing_check_and_the_formula(&at_the_edge),
        "边界 {boundary_units} 在独立的一次试写里该放行：{at_the_edge:?}"
    );
    let above_the_edge =
        probe_one_write_on_the_sealed_base(&base, boundary_units + 1, content_bytes)
            .expect("对拍过");
    assert!(
        !passed_the_landing_check_and_the_formula(&above_the_edge),
        "边界 + 1 在独立的一次试写里该不放行：{above_the_edge:?}"
    );
}

/// 标准错误上被拒那一行的拒因文字与缺口折算（登记 12.12 只改标准错误，不改产物行）。
pub fn check_that_the_refusal_note_names_the_gate_and_the_gap_and_the_shortfall_rounds_up_to_whole_units(
) {
    let space = RefusalClass::SpaceAdmission {
        short_devices: String::new(),
        shortfall_bytes: 98_304,
    };
    assert_eq!(
        space.gate_and_gap_text(),
        "space_admission_refused(shortfall_bytes=98304)"
    );
    assert_eq!(space.space_shortfall_bytes(), Some(98_304));
    let in_placement = RefusalClass::FixedPointPlacement {
        unit: String::new(),
        refusal: String::new(),
    };
    assert_eq!(
        in_placement.gate_and_gap_text(),
        "placement_refused_in_the_fixed_point"
    );
    assert_eq!(in_placement.space_shortfall_bytes(), None);
    let over_the_limit = RefusalClass::Other {
        description: String::new(),
        journal_records_over_the_limit: Some(631),
    };
    assert_eq!(
        over_the_limit.gate_and_gap_text(),
        "other_refusal(journal_records_over_the_limit=631)"
    );
    assert_eq!(over_the_limit.space_shortfall_bytes(), None);
    let without_a_reading = RefusalClass::Other {
        description: String::new(),
        journal_records_over_the_limit: None,
    };
    assert_eq!(without_a_reading.gate_and_gap_text(), "other_refusal");
    assert_eq!(
        units_a_shortfall_is_worth(0),
        1,
        "缺口读数不是正数时按一字节算"
    );
    assert_eq!(units_a_shortfall_is_worth(1), 1);
    assert_eq!(units_a_shortfall_is_worth(32_768), 1);
    assert_eq!(units_a_shortfall_is_worth(32_769), 2, "向上取整");
    assert_eq!(units_a_shortfall_is_worth(98_304), 3);
    let refused_in_placement: Result<TransactionOutput, PublishError> =
        Err(PublishError::PlacementRefused {
            unit: TransactionUnit::TreeTable,
            refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
        });
    assert_eq!(
        progress_line_of_a_write(6000, 1.54, &refused_in_placement),
        "[e181] 一次写 6000 个单元的发布用了 1.5 秒，被拒，拒因 placement_refused_in_the_fixed_point"
    );
    let over_the_record_limit: Result<TransactionOutput, PublishError> =
        Err(PublishError::JournalRecordsOfThePublishExceedTheLimit {
            records_of_the_publish: 66_167,
            record_limit: 65_536,
        });
    assert_eq!(
        progress_line_of_a_write(65_877, 120.0, &over_the_record_limit),
        "[e181] 一次写 65877 个单元的发布用了 120.0 秒，被拒，拒因 other_refusal(journal_records_over_the_limit=631)",
        "登记 12.4 的读数：66167 条、上限 65536，超出 631"
    );
    assert_eq!(
        RefusalClass::of(&PublishError::ContentExceedsDataUnit {
            bytes: 1,
            capacity: 0
        }),
        RefusalClass::Other {
            description: "ContentExceedsDataUnit { bytes: 1, capacity: 0 }".to_string(),
            journal_records_over_the_limit: None
        },
        "别的拒因原样带 Debug 文字，不带超出条数"
    );
}

/// 有一个边历史没找到边，固定点上界那一段停机（带几何名与原因）；都找到了不停。
pub fn check_that_a_geometry_whose_edge_was_not_found_stops_the_fixed_point_upper_bound_section() {
    let found = EdgeReport::default();
    let not_found = EdgeReport {
        edge_not_found_reason: Some("一直放行到上限 110 个单元都没有被拒的试写".to_string()),
        ..EdgeReport::default()
    };
    assert_eq!(
        stop_when_an_edge_was_not_found(&[
            ("narrow", found.clone()),
            ("small_unit_area", found.clone())
        ]),
        Ok(())
    );
    let stopped =
        stop_when_an_edge_was_not_found(&[("narrow", found), ("small_unit_area", not_found)])
            .expect_err("有一个没找到边，该停机");
    assert!(
        stopped.starts_with("没找到边：")
            && stopped.contains("small_unit_area")
            && !stopped.contains("narrow"),
        "{stopped}"
    );
}

/// 准入判法的第一行以 `forced` 起头时才交回摘要（最后一段，空白换成下划线）；满足或没声明时不交。
pub fn check_that_the_preflight_verdict_line_gives_a_summary_only_when_forced() {
    assert_eq!(
        forced_summary_of_the_preflight_verdict("forced\tinputs-unchanged\ta b\tc d"),
        Some("c_d".to_string())
    );
    assert_eq!(forced_summary_of_the_preflight_verdict("met\tabc123"), None);
    assert_eq!(forced_summary_of_the_preflight_verdict("undeclared"), None);
    assert_eq!(forced_summary_of_the_preflight_verdict(""), None);
}

pub fn check_that_local_constants_agree_with_the_crates() {
    check_local_constants_against_the_crates().expect("本地常量回比");
}
