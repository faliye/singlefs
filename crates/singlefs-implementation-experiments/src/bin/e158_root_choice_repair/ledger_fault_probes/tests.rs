use super::*;
use crate::tests::with_the_rotation_after_the_newest_publish_not_written;
use singlefs_core::address::SlotNumber;
use singlefs_core::pointer::NodePointer;
use singlefs_core::unit::build_index_node;

/// Q1（岔路单第 1 行）：`fault_targets_for` 的三种非空严重度各自选对了 (设备, 偏移)。
#[test]
fn fault_targets_for_both_copies_hits_both_devices() {
    let locations = [
        LocationEntry {
            device: DeviceIdentity(0),
            slot: SlotNumber(10),
            unit_checksum: 0,
        },
        LocationEntry {
            device: DeviceIdentity(1),
            slot: SlotNumber(20),
            unit_checksum: 0,
        },
    ];
    let both = fault_targets_for(&locations, FaultSeverity::Both);
    assert_eq!(
        both,
        vec![
            (DeviceIdentity(0), SlotNumber(10).to_device_offset()),
            (DeviceIdentity(1), SlotNumber(20).to_device_offset()),
        ]
    );
    let disk0 = fault_targets_for(&locations, FaultSeverity::Disk0);
    assert_eq!(
        disk0,
        vec![(DeviceIdentity(0), SlotNumber(10).to_device_offset())]
    );
    let disk1 = fault_targets_for(&locations, FaultSeverity::Disk1);
    assert_eq!(
        disk1,
        vec![(DeviceIdentity(1), SlotNumber(20).to_device_offset())]
    );
    let empty = fault_targets_for(&locations, FaultSeverity::Empty);
    assert!(empty.is_empty(), "空集严重度不该注入任何落点");
}

/// Φ1 的故障数（5.1 原文：两份都读失败 = 2 个故障，只一份 = 1 个，空集 = 0）。
#[test]
fn fault_severity_fault_count_matches_the_prereg_table() {
    assert_eq!(FaultSeverity::Empty.fault_count(), 0);
    assert_eq!(FaultSeverity::Both.fault_count(), 2);
    assert_eq!(FaultSeverity::Disk0.fault_count(), 1);
    assert_eq!(FaultSeverity::Disk1.fault_count(), 1);
}

/// op1 第三种变体：`floor_targets_between` 从 F+1 起，含上限本身。
#[test]
fn floor_targets_between_starts_strictly_above_the_current_floor() {
    let targets = floor_targets_between(CheckpointTxg(2), CheckpointTxg(4));
    assert_eq!(targets, vec![CheckpointTxg(3), CheckpointTxg(4)]);
}

/// 同上：上限本身也是一个合法目标（半开区间是 (F, 上限]，不是 (F, 上限)）。
#[test]
fn floor_targets_between_includes_the_ceiling_itself() {
    let targets = floor_targets_between(CheckpointTxg(2), CheckpointTxg(4));
    assert!(
        targets.contains(&CheckpointTxg(4)),
        "上限本身也是一个合法目标，得到 {targets:?}"
    );
}

/// 上限不高于现行 F 时没有余量可抬——`..=` 起点大于终点天然给出空区间，不用另写判空分支。
#[test]
fn floor_targets_between_is_empty_when_the_ceiling_does_not_exceed_the_current_floor() {
    assert!(floor_targets_between(CheckpointTxg(5), CheckpointTxg(5)).is_empty());
    assert!(floor_targets_between(CheckpointTxg(5), CheckpointTxg(3)).is_empty());
}

/// op1 第三种变体：探测出的 floor 目标落在 [F+1, 上限] 内时，不注入任何故障应当成功
/// （`error_member` 为 `None`）；换成对着某条被抛弃可读根的分配记录树节点注入故障（两份都读失败），
/// `raise_rollback_floor` 必须做成。那条被抛弃根在先不注入的 `mount_writable` 那一刻读得出，根环表把它记成被抛弃根、带着它引用的落点；
/// 抬 F 那一串照记着的落点算影子账、不再从盘上读它的账（D16（发布语义） 已定项 1「根槽这一次读坏」那一行末句，根环线），
/// 所以交回的 `RaisedFloor::abandoned_roots_unreadable` 是 0（注入时与不注入时都是），抬 F 之后它记着的落点一槽都不空闲
/// （隔离着，或现行账里还分配着）——照记着的落点隔离，没被注入的读故障挡住，也没漏掉。
/// 根环表里没记着、账读不出时照读盘那一支（计数 1、从映射现算）由验证包的
/// `an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 的
/// `an_unmount_meeting_an_abandoned_root_not_in_the_root_ring_table_whose_account_is_unreadable_recomputes_it_from_the_mapping`
/// 与 `reuse_after_raising_the_floor.rs` 的 `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable` 钉。
/// 被抛弃根取自第 2 次跑那一节 H1c 的抛弃步（`abandonment_step_nodes`；跑前登记 `research/prompts/e158-r2-prereg.md`
/// 第一节处理表：故障改打 H1c 里一条被抛弃根的分配记录树节点、删掉「或者报了错也算」那一半）。
#[test]
fn mount_writable_then_raise_floor_reaches_the_injected_fault() {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let fixed_geometry = fixed_geometry_for(&GEOMETRY_PRIMARY);
    let mut tested = false;
    for (pool, abandoned) in abandonment_step_nodes() {
        let Ok(pool_geometry) = independent_geometry(&pool) else {
            continue;
        };
        let readable = readable_roots_independent(&pool, &pool_geometry);
        let Some(&abandoned_root) = abandoned.iter().find(|root| readable.contains(root)) else {
            continue;
        };
        let Some(record) = root_record_of(&pool, &pool_geometry, abandoned_root) else {
            continue;
        };
        let plain_devices = devices_from_pool(&pool, IMAGE_BYTES);
        let Ok(allocation_locations) = locate_allocation_record_tree(&plain_devices, &record)
        else {
            continue;
        };
        let node = SimNode {
            pool: pool.clone(),
            session: None,
            timeline: Vec::new(),
            rollback_events: Vec::new(),
            path: Vec::new(),
            content_by_root: BTreeMap::new(),
        };
        let targets = raise_floor_targets_for(&node, &parameters);
        let Some(&floor) = targets.first() else {
            continue;
        };
        let kind = FaultedOperationKind::MountWritableThenRaiseFloorTo(floor);
        let without_fault =
            attempt_faulted_operation(&node, &parameters, fixed_geometry, &kind, &[]);
        assert!(
            without_fault.error_member.is_none(),
            "探测出的 floor 目标不注入任何故障应当成功，得到 {:?}",
            without_fault.error_member
        );
        let fault_targets = fault_targets_for(&allocation_locations, FaultSeverity::Both);
        let with_fault =
            attempt_faulted_operation(&node, &parameters, fixed_geometry, &kind, &fault_targets);
        assert!(
            with_fault.error_member.is_none(),
            "被抛弃根的分配记录树节点两份都读失败时抬 F 照样做成（账读不出只计数），得到 {:?}",
            with_fault.error_member
        );
        assert_eq!(
            (
                with_fault.abandoned_roots_unreadable,
                without_fault.abandoned_roots_unreadable
            ),
            (Some(0), Some(0)),
            "这条被抛弃根挂载那一刻记在根环表里：抬 F 照记着的落点算、不读它的账，注入与不注入都不计（注入时、不注入时）"
        );
        let mut devices_of_the_mount = devices_from_pool(&pool, IMAGE_BYTES);
        let mounted = mount_writable(&parameters, &mut devices_of_the_mount)
            .expect("先不注入的可写挂载做得成（上面那一次就做成了）");
        let recorded_placements = mounted
            .allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .placements_recorded_for_the_abandoned_root(record.instance, record.checkpoint_txg)
            .filter(|placements| !placements.is_empty())
            .expect("挂载那一刻这条被抛弃根读得出：根环表把它记成被抛弃根、带着它引用的落点");
        let PoolVersion::WithFile(current) = mounted.current else {
            panic!("先不注入的可写挂载交回的现行那一版带文件（上面那一次抬 F 做成了）");
        };
        let mut current = std::sync::Arc::unwrap_or_clone(current);
        let mut allocator = mounted.allocator;
        let mut faulted_devices: Vec<(
            DeviceIdentity,
            FaultInjectingBlockDevice<SparseBlockDevice>,
        )> = Vec::new();
        for (identity, device) in devices_of_the_mount {
            let plan = SharedFaultPlan::unarmed(fixed_geometry);
            if let Some((_, offset)) = fault_targets
                .iter()
                .find(|(target_device, _)| *target_device == identity)
            {
                plan.arm(FaultSchedule {
                    fault: InjectedFault::ReadFails,
                    device: FaultDeviceSelector::OnlyDevice(identity),
                    placement: FaultPlacement::OffsetExactly(*offset),
                    counting: FaultCounting::AcrossThePool,
                    occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
                });
            }
            faulted_devices.push((
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan),
            ));
        }
        // 注入确实打在抬 F 那一遍要读的地方（正推 `research/prompts/m3-batch10z-d-code-r1-sonnet-output.md` K1-3）：抬 F 之前，
        // 经同一组带故障的盘读这条被抛弃根的分配记录树节点的每一份（落点照 `locate_allocation_record_tree` 从盘上现找，不取 `fault_targets`），
        // 都读不出——抬 F 那一遍要是读它的账，就撞上注入。注入落空（偏移不对、故障没装上、严重度给成空集）时这里读得出，这一条红。
        for location in &allocation_locations {
            let (_, faulted_device) = faulted_devices
                .iter()
                .find(|(identity, _)| *identity == location.device)
                .expect("池里有这块盘");
            let mut node_bytes = vec![
                0u8;
                usize::try_from(singlefs_format::SLOT_BYTES)
                    .expect("槽宽装得进 usize")
            ];
            assert!(
                faulted_device
                    .read_at(location.slot.to_device_offset(), &mut node_bytes)
                    .is_err(),
                "注入打在这条被抛弃根的分配记录树节点 {location:?} 上：抬 F 那一遍去读它就读不出"
            );
        }
        raise_rollback_floor(
            &parameters,
            &mut faulted_devices,
            &mut allocator,
            &mut current,
            floor,
            ShadowLedger::On,
        )
        .expect("注入同一组故障再抬一次：照样做成");
        let free_recorded_placements: Vec<_> = recorded_placements
            .iter()
            .filter(|placement| {
                allocator
                    .devices
                    .iter()
                    .find(|device_map| device_map.device == placement.device)
                    .expect("池里有这块盘")
                    .is_free(placement.slot)
            })
            .collect();
        assert!(
            free_recorded_placements.is_empty(),
            "抬 F 之后这条被抛弃根记着的落点一槽都不空闲（照记着的落点隔离）：空闲的 {free_recorded_placements:?}"
        );
        tested = true;
        break;
    }
    assert!(
        tested,
        "H1c 抛弃步造出的节点里应当至少有一个有 op1 第三种变体可跑（有余量、有可读被抛弃根）"
    );
}

/// `error_member_of_debug`：跨臂只用字符串比较（A1 副本新变体在「今天」那份编译单元里
/// 根本不存在这个枚举成员，不能直接 `match` 枚举名），首词切分要认得三种括号/空格分隔形态。
#[test]
fn error_member_of_debug_extracts_the_variant_name() {
    assert_eq!(
        error_member_of_debug("AbandonedRootLedgerUnreadable { count: 3 }"),
        "AbandonedRootLedgerUnreadable"
    );
    assert_eq!(
        error_member_of_debug("RollbackTargetNotACandidate { target: X, exclusion: Y }"),
        "RollbackTargetNotACandidate"
    );
    assert_eq!(error_member_of_debug("Recovery(NoValidRoot)"), "Recovery");
    assert_eq!(
        error_member_of_debug("FileVersionWithoutAnyJournalRecord"),
        "FileVersionWithoutAnyJournalRecord"
    );
}

/// `location_key`：去重键只看设备号与槽号，校验和字段不参与比较。
#[test]
fn location_key_ignores_the_checksum_field() {
    let first_copy = LocationEntry {
        device: DeviceIdentity(1),
        slot: SlotNumber(7),
        unit_checksum: 111,
    };
    let second_copy = LocationEntry {
        device: DeviceIdentity(1),
        slot: SlotNumber(7),
        unit_checksum: 222,
    };
    assert_eq!(location_key(&first_copy), location_key(&second_copy));
    assert_eq!(location_key(&first_copy), (1u32, 7u64));
}

/// 候选 (b) 分配记录树指称「从内容树反推占用集合」：第一个文件版本上，中央映射树给出的落点
/// 集合非空（至少覆盖那次发布写的数据单元）；mkfs 那一版（`mapping_root` 恒
/// `NodePointer::empty_root()`）读不出——是报错，不是当空集读到。
#[test]
fn allocation_record_tree_reachable_placements_via_central_mapping_reflects_written_content_and_errors_on_an_empty_pointer(
) {
    let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let newest = readable_roots_independent(&node.pool, &geometry_view)
        .into_iter()
        .max()
        .expect("bootstrap 之后至少有一条根");
    let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
    let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let placements =
        allocation_record_tree_reachable_placements_via_central_mapping(&plain_devices, &record)
            .expect("第一个文件版本的中央映射树应当读得出");
    assert!(
        !placements.is_empty(),
        "第一个文件版本至少写了一个数据单元，中央映射树给出的落点集合不该是空集"
    );

    let genesis_record = RootRecord {
        mapping_root: NodePointer::empty_root(),
        ..record
    };
    assert!(
        allocation_record_tree_reachable_placements_via_central_mapping(
            &plain_devices,
            &genesis_record,
        )
        .is_err(),
        "mapping_root 是空指针时应当报错，不是读成空集"
    );
}

/// 候选 (b) 分配记录树指称「从内容树反推占用集合」走全：走全版本必须是只走中央映射
/// 版本的超集——至少多出树表自己的节点与中央映射树自己的节点这两个位置；空指针那一版同样报错
/// （走全版本第一步就调用只走中央映射的版本）。
#[test]
fn allocation_record_tree_reachable_placements_via_all_structural_trees_is_a_superset_of_the_central_mapping_only_version(
) {
    let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let newest = readable_roots_independent(&node.pool, &geometry_view)
        .into_iter()
        .max()
        .expect("bootstrap 之后至少有一条根");
    let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
    let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let central_mapping_only =
        allocation_record_tree_reachable_placements_via_central_mapping(&plain_devices, &record)
            .expect("第一个文件版本的中央映射树应当读得出");
    let all_structural_trees =
        allocation_record_tree_reachable_placements_via_all_structural_trees(
            &plain_devices,
            &record,
        )
        .expect("走全版本在同一段历史上也应当读得出");
    assert!(
        all_structural_trees.is_superset(&central_mapping_only),
        "走全版本必须包含只走中央映射版本给出的每一个落点"
    );
    let mapping_root_own_node = (
        record.mapping_root.locations[0].device.0,
        record.mapping_root.locations[0].slot.0,
    );
    let tree_table_own_node = (
        record.tree_table.locations[0].device.0,
        record.tree_table.locations[0].slot.0,
    );
    assert!(
        all_structural_trees.contains(&mapping_root_own_node),
        "走全版本要包含中央映射树自己的节点，不只是它指向的数据单元"
    );
    assert!(
        all_structural_trees.contains(&tree_table_own_node),
        "走全版本要包含树表自己的节点"
    );

    let genesis_record = RootRecord {
        mapping_root: NodePointer::empty_root(),
        ..record
    };
    assert!(
        allocation_record_tree_reachable_placements_via_all_structural_trees(
            &plain_devices,
            &genesis_record,
        )
        .is_err(),
        "mapping_root 是空指针时走全版本也应当报错（它第一步就调用只走中央映射的版本）"
    );
}

/// 根因排查②：171 格 `subset_of_pristine` 差集里的两个 (设备, 槽) 就是实例表自己的
/// 物理节点（`RootRecord::instance_table`），不是 LIVELIST/SPARSE_SIDE_TABLE/DEADLIST 的节点、
/// 也不是分配记录树自己的节点——`classify_diff_pair` 要能分辨这三种、且对查无归属的落点如实报告。
#[test]
fn classify_diff_pair_identifies_the_instance_table_self_node() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let newest = readable_roots_independent(&node.pool, &geometry_view)
        .into_iter()
        .max()
        .expect("bootstrap 之后至少有一条根");
    let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
    let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let raw_records =
        allocation_records_under_root(&plain_devices, &record).expect("这段历史的分配记录树读得出");
    let allocation_locations = locate_allocation_record_tree(&plain_devices, &record).ok();

    let instance_table_pair = (
        record.instance_table.locations[0].device.0,
        record.instance_table.locations[0].slot.0,
    );
    assert_eq!(
        classify_diff_pair(
            &plain_devices,
            &record,
            allocation_locations,
            &raw_records,
            instance_table_pair,
        ),
        "instance_table_self_node_not_walked_by_the_all_structural_trees_arm",
        "实例表自己的节点要被认出来，不能落进查无归属的兜底分支"
    );

    let allocation_record_tree_pair = (
        allocation_locations.expect("这段历史的分配记录树有物理位置")[0]
            .device
            .0,
        allocation_locations.expect("同上")[0].slot.0,
    );
    assert_eq!(
        classify_diff_pair(
            &plain_devices,
            &record,
            allocation_locations,
            &raw_records,
            allocation_record_tree_pair,
        ),
        "allocation_record_tree_self_node",
        "分配记录树自己的节点不能被误判成实例表"
    );

    assert_eq!(
        classify_diff_pair(
            &plain_devices,
            &record,
            allocation_locations,
            &raw_records,
            (99, 999_999),
        ),
        "not_found_in_raw_allocation_records",
        "查无归属的落点要如实报告，不能编一个假分类"
    );
}

/// 候选 (b) 树表指称「crates 有路」子分支要用的量：同一实例上更旧的可读根，取的是
/// checkpoint_txg 最大的那一条祖先；不许把自己选成祖先，也不许漏掉更接近目标的候选。
#[test]
fn nearest_older_readable_root_on_same_instance_picks_the_closest_ancestor_on_the_same_instance() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 2).expect("bootstrap 应当成功（含 2 次覆盖写）");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let readable = readable_roots_independent(&node.pool, &geometry_view);
    let newest = *readable.iter().max().expect("bootstrap 之后至少有一条根");
    let target = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
    let (older_root, older_record) =
        nearest_older_readable_root_on_same_instance(&node.pool, &geometry_view, newest, &target)
            .expect("同一实例上还有更旧的可读根（暖机与首个文件那几条）");
    assert_eq!(
        older_root.0, target.instance.0,
        "找到的祖先要与目标同一实例"
    );
    assert!(
        older_record.checkpoint_txg < target.checkpoint_txg,
        "找到的祖先 checkpoint_txg 要严格小于目标"
    );
    let closer_candidate_exists = readable.iter().any(|candidate| {
        candidate.0 == target.instance.0
            && *candidate != older_root
            && candidate.1 > older_root.1
            && candidate.1 < target.checkpoint_txg.0
    });
    assert!(
        !closer_candidate_exists,
        "同一实例上存在 txg 更接近目标、却没被选中的候选"
    );
}

/// （实十九提醒）：中央映射树条目数超过单节点叶容量
/// （`transaction.rs` 里「中央映射树叶 294」那一句）会长成多层，根节点变成内部节点
/// （`level > 0`）。这里手工构造一个合法但 `level=1` 的假节点，覆盖真实中央映射树根的两份物理
/// 拷贝，验证 `allocation_record_tree_reachable_placements_via_central_mapping` 显式拦下它——
/// 不是靠 `parse_mapping_entry` 的宽度检查侥幸拦住：内部条目宽约 114 字节（`transaction.rs`
/// 同一行「内部 143」按 16384 / 143 反推），比 `MAPPING_ENTRY_BYTES`=55 宽，那条宽度检查拦不住它。
#[test]
fn allocation_record_tree_reachable_placements_via_central_mapping_rejects_a_multi_level_root() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let newest = readable_roots_independent(&node.pool, &geometry_view)
        .into_iter()
        .max()
        .expect("bootstrap 之后至少有一条根");
    let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
    let mut plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);

    let real_bytes = read_node_bytes(&plain_devices, &record.mapping_root.locations)
        .expect("真实的中央映射树根读得出");
    let real_header = parse_index_node(&real_bytes).expect("真实的中央映射树根解得开");
    assert_eq!(real_header.level, 0, "bootstrap 之后中央映射树根应当仍是叶");

    // 内部条目宽的估计值：只要比 MAPPING_ENTRY_BYTES=55 宽，就足够触发要测的那条路
    // （宽度检查拦不住，必须靠显式的 level 检查）。
    let fake_entry_width = 114u16;
    let fake_bytes = build_index_node(
        real_header.tree,
        1,
        real_header.key_width,
        &real_header.smallest_key,
        &real_header.largest_key,
        real_header.birth_txg,
        &FILESYSTEM_IDENTIFIER,
        real_header.instance,
        real_header.birth_sequence,
        fake_entry_width,
        &[vec![0u8; usize::from(fake_entry_width)]],
    );

    for location in &record.mapping_root.locations {
        let (_, device) = plain_devices
            .iter_mut()
            .find(|(identity, _)| *identity == location.device)
            .expect("这块盘在池里");
        device
            .write_at(
                location.slot.to_device_offset(),
                &fake_bytes,
                WriteDurability::Plain,
            )
            .expect("写入假节点应当成功");
    }

    let outcome =
        allocation_record_tree_reachable_placements_via_central_mapping(&plain_devices, &record);
    let error = outcome.expect_err("level=1 的根节点必须被拒绝，不能被当成叶解出假的落点集合");
    assert!(
        error.contains("level=1"),
        "错误信息要点名是哪个 level 拦下的，实际: {error}"
    );
}

/// （`m2-rootchoice-repair-r1-forks.md` 岔路单第 1 行还差项④）：持续故障与瞬时故障
/// 在 op1 起的三次挂载轨迹上必须不同——持续（`persistent=true`）故障不撤，每一步都该继续触发；
/// 瞬时（`persistent=false`）故障只在 op1 那一步有效，撤掉之后的两步该恢复成不触发。用 PC1-a 的挑选条件（按固定次序找第一个满足
/// 『某条被抛弃根的分配记录树节点不共享』的历史）选落点；被抛弃根取自第 2 次跑那一节 H1c 的抛弃步
/// （`abandonment_step_nodes`；跑前登记 `research/prompts/e158-r2-prereg.md` 第一节处理表：第一次跑靠挂载时回退造的
/// 被抛弃根在今天的代码上造不出来，这条单测开工时就红）。
#[test]
fn mount_writable_trajectory_distinguishes_persistent_from_transient_faults() {
    let geometry = &GEOMETRY_PRIMARY;
    let parameters = parameters_for(geometry);
    let fixed_geometry = fixed_geometry_for(geometry);
    for (pool, abandoned) in abandonment_step_nodes() {
        let Ok(pool_geometry) = independent_geometry(&pool) else {
            continue;
        };
        let readable = readable_roots_independent(&pool, &pool_geometry);
        let abandoned_readable: Vec<TimelineRoot> = abandoned
            .iter()
            .copied()
            .filter(|root| readable.contains(root))
            .collect();
        let plain_devices = devices_from_pool(&pool, IMAGE_BYTES);

        for &abandoned_root in &abandoned_readable {
            let Some(record) = root_record_of(&pool, &pool_geometry, abandoned_root) else {
                continue;
            };
            let Ok(allocation_locations) = locate_allocation_record_tree(&plain_devices, &record)
            else {
                continue;
            };
            let allocation_keys: BTreeSet<(u32, u64)> =
                allocation_locations.iter().map(location_key).collect();
            if is_unit_shared(
                &plain_devices,
                &pool,
                &pool_geometry,
                abandoned_root,
                &allocation_keys,
            ) {
                continue;
            }

            let allocation_fault_targets =
                fault_targets_for(&allocation_locations, FaultSeverity::Both);
            let persistent = mount_writable_trajectory(
                &pool,
                &parameters,
                fixed_geometry,
                &allocation_fault_targets,
                true,
                3,
            );
            let transient = mount_writable_trajectory(
                &pool,
                &parameters,
                fixed_geometry,
                &allocation_fault_targets,
                false,
                3,
            );
            assert_eq!(persistent.len(), 3, "轨迹要报满 3 步（跑前登记 8.1 原文）");
            assert_eq!(transient.len(), 3);
            assert_eq!(
                persistent[0], transient[0],
                "op1 这一步两种模式的故障目标相同，第一步的结局必须相同"
            );
            assert!(
                persistent[0].is_some_and(|count| count > 0),
                "PC1-a 已经证明这一格会触发，第一步的计数必须 > 0"
            );
            assert_eq!(
                persistent[1], persistent[0],
                "持续故障不撤，第二步该继续触发（这一格是单一故障不共享的落点，不随挂载次数变化）"
            );
            assert_eq!(persistent[2], persistent[0], "持续故障第三步同理");
            assert_eq!(
                transient[1],
                Some(0),
                "瞬时故障撤掉之后，第二步该恢复成不触发"
            );
            assert_eq!(transient[2], Some(0), "瞬时故障第三步同理");
            return;
        }
    }
    panic!("H1c 抛弃步造出的节点里应当至少有一格满足 PC1-a 的挑选条件（跑前登记 5.2 阳性对照）");
}

/// 两条沿用的单测从 H1c 抛弃步造出的节点里挑被抛弃根（跑前登记第一节处理表：「改从 H1c 的 PC1-a 构造取（同一个落点选法）」）。
/// 固定次序与挑法同装置的 `abandonment_step_node_list`（n1、c1、b；挂载做成、落点全拦到、按新实例表判出至少一条被抛弃的根），
/// 只是系统配置没见证到被藏的根（`with_the_rotation_after_the_newest_publish_not_written`）、k 只取 1：C554 乙合进 `crates` 之后，见证在时可写挂载拒，
/// 藏两条以上时较旧那一槽仍见证被藏的根，造不出被抛弃的根。n1 取到 6：k = 1 只抛弃一条，抬 F 的上限要第 4 新的非空有效根
/// （D16（发布语义） 已定项 1），n1 = 1–3 的节点上上限等于现行 F、抬 F 那一种 op1 没有目标（装置的节点表靠 k = 4 那几格才有目标）。
fn abandonment_step_nodes() -> Vec<(MemoryPool, BTreeSet<TimelineRoot>)> {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut nodes = Vec::new();
    for overwrites in 1u64..=6 {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            let mut history = second_run_start(&GEOMETRY_PRIMARY, overwrites).expect("历史");
            second_run_close(&mut history, &parameters, closing, "node-list", "node-list")
                .expect("关闭");
            let pool_geometry = independent_geometry(&history.pool).expect("几何");
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstNamedUnit,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                let Ok(Some(ranges)) = unreadable_ranges_of_the_abandonment(
                    &history,
                    &pool_geometry,
                    1,
                    UnreadableForm::ReadFails,
                    replay_break,
                ) else {
                    continue;
                };
                let pool = with_the_rotation_after_the_newest_publish_not_written(&history.pool);
                let run = run_instrumented_mount_writable(&pool, &parameters, &ranges, None, None);
                if run.result.is_err()
                    || !unintercepted_ranges(&ranges, &run.intercepted_reads).is_empty()
                {
                    continue;
                }
                let Ok(abandoned) = abandoned_roots_on(&run.pool_after) else {
                    continue;
                };
                if !abandoned.is_empty() {
                    nodes.push((run.pool_after, abandoned));
                }
            }
        }
    }
    nodes
}
