//! 可写挂载在任何写之前拒掉的树与根的内容：账务叶子重复键、树表条目越过树号水位、inode 内部条目的身份与叶子容器不符、
//! 树根出生序号与指针不符、实例表指针落在单元区之前、盘在单元区起点之前就到头。

use super::*;

// ─── 代码审阅第 33 条（与第 24 条同一件）：从盘上重建上一版照冷走读判全 ───

/// 场景：第一个文件那一版的记账树（单个叶根，15 条）里第二条条目整条抄成第一条，两条的 key 相同；叶的两道校验和、树表条目里的根指针、
/// 映射条目、树表、映射树根与根记录的自证校验和都重算。
/// 预期：可写挂载在任何写之前报「叶里的条目不按 key 严格递增」（I-1.1）。重建若只按「拼得成一棵树」读记账树、不判 key 次序，
/// 重复 key 的叶原样交给写行那次发布，规划记账树时在 `transaction` 的断言上 panic。
#[test]
fn an_accounting_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_before_any_write() {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        TREE_KIND_ACCOUNTING,
        |node| {
            let first_entry = node[entry_range_in_node(node, 0)].to_vec();
            let second_entry_range = entry_range_in_node(node, 1);
            node[second_entry_range].copy_from_slice(&first_entry);
        },
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-1.1",
                detail: "叶里的条目不按 key 严格递增",
            })
        ),
        "重复 key 的记账叶判 key 次序：{refusal:?}"
    );
}

/// 场景：最新那条根的树表里退役树那条条目的树 ID 改成根记录的树 ID 水位（第一个文件那一版是 65549），树表与根记录重封。
/// 退役树在发号次序里排最后、号最大（65548），改成 65549 之后树表条目仍按树 ID 严格升序、合发号次序，只违反水位那一条。
/// （D18（块里携带什么信息） 已定项 24 第 6 条之后 deadlist 后面还有四棵，改它就先违反 I-9.16。）
/// 预期：可写挂载在任何写之前报 I-7.8（根记录树 ID 水位不低于全池最大树 ID）「树 ID 不低于水位」。重建若不判它，写行那次发布照抄这张树表。
/// 把被改的条目从 livelist 换成 deadlist：livelist 改成 19 同时违反 I-9.16（树表条目按树 ID 严格升序且合发号次序），
/// 而 I-9.16 那一判排在水位之前（树表排序报告第八节第 3 条），报的是 I-9.16，这条用例就判不到 I-7.8 了。
#[test]
fn a_tree_table_entry_at_or_above_the_tree_identifier_watermark_is_refused_by_the_writable_mount() {
    let (mut devices, first) = pool_after_the_first_file();
    let watermark = first.root.tree_identifier_watermark;
    rewrite_a_tree_table_entry_of_the_newest_root(&mut devices, TREE_KIND_RETIREMENT, |entry| {
        entry[..8].copy_from_slice(&watermark.to_le_bytes());
    });
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-7.8",
                detail: "树 ID 不低于水位",
            })
        ),
        "树表条目的树 ID 判 I-7.8：{refusal:?}"
    );
}

/// 场景：inode 树根第一条内部条目里身份引用的容器出生代加一（条目 = 分隔 key 8 + 身份引用 26 + 子指针 86，容器出生代住条目偏移 26），
/// 与它指着的叶容器头里的四元组不再逐字相等；inode 树根与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报 I-9.2（条目身份与子头相符）「inode 叶头与条目身份引用不符」。重建若取容器头那一份身份、不与条目比，就收下它。
#[test]
fn an_inode_internal_entry_whose_identity_differs_from_its_leaf_container_is_refused_by_the_writable_mount(
) {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(&mut devices, TREE_KIND_INODE, |node| {
        let entry_range = entry_range_in_node(node, 0);
        let container_birth_offset = entry_range.start + 8 + 8 + 2 + 8;
        let container_birth = u64::from_le_bytes(
            node[container_birth_offset..container_birth_offset + 8]
                .try_into()
                .expect("8 字节"),
        );
        node[container_birth_offset..container_birth_offset + 8]
            .copy_from_slice(&(container_birth + 1).to_le_bytes());
    });
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-9.2",
                detail: "inode 叶头与条目身份引用不符",
            })
        ),
        "inode 内部条目的身份引用判 I-9.2：{refusal:?}"
    );
}

/// 场景：inode 树根节点头里的出生序号加一（住 72 + 2k，k = 8），与树表条目里指着它的那条指针不再相同；节点与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报 I-1.2「树根出生序号与指针不符」。重建读 inode 树根若只解节点、不核它的出生身份，就收下它。
#[test]
fn an_inode_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount(
) {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(&mut devices, TREE_KIND_INODE, |node| {
        let birth_sequence_offset = 72 + 2 * usize::from(node[51]);
        let birth_sequence = u32::from_le_bytes(
            node[birth_sequence_offset..birth_sequence_offset + 4]
                .try_into()
                .expect("4 字节"),
        );
        node[birth_sequence_offset..birth_sequence_offset + 4]
            .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
    });
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-1.2",
                detail: "树根出生序号与指针不符",
            })
        ),
        "inode 树根的出生序号判 I-1.2：{refusal:?}"
    );
}

// ─── 代码审阅第 32 条：盘上读来的指针槽号进分配器之前判在单元区里 ───

/// 场景：只做过 mkfs 的池，三个根环区域里第 0 代根的实例表指针两条位置条目都改指槽 100（单元区起点之下、没人用的空当），
/// 实例表单元原样拷一份到两块盘的槽 100：读者按位置条目读得到、校验和对得上、身份 (0, 4, 0, 0) 照样对。
/// 预期：可写挂载在动分配器与任何写之前报「分配记录的槽号落在单元区起点之下」（与盘上分配记录同一道几何判）。
/// mkfs 那一版的两个落点若直接进 `mark_format_time_units`，在 `DeviceFreeMap::index` 的 expect 上 panic。
#[test]
fn a_format_time_instance_table_pointer_below_the_unit_area_is_refused_before_the_allocator_is_touched(
) {
    let mut devices = formatted_pool();
    let genesis_roots = roots_in_the_ring(&devices);
    assert_eq!(genesis_roots.len(), 3, "mkfs 把第 0 代根种进三个区域各一份");
    let instance_table_pointer = genesis_roots[0].1.instance_table;
    let instance_table = read_bytes(
        &devices,
        instance_table_pointer.locations[0].device,
        instance_table_pointer.locations[0].slot.to_device_offset(),
        data_unit_bytes(),
    );
    let mut moved_locations = instance_table_pointer.locations;
    for location in &mut moved_locations {
        location.slot = SLOT_BELOW_THE_UNIT_AREA_NOBODY_USES;
    }
    write_unit_at_both_locations(&mut devices, &moved_locations, &instance_table);
    for (ring_slot, mut root) in genesis_roots {
        root.instance_table.locations = moved_locations;
        write_root_into_the_ring_slot(&mut devices, ring_slot, &root);
    }
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                what: "分配记录的槽号落在单元区起点之下",
            })
        ),
        "mkfs 那一版的落点先过几何判：{refusal:?}"
    );
}

/// 场景：第一个文件那一版之后重开一次（实例 2 写行 (1, 3, W)、暖机），再往 txg 4 按落点公式该住的那一槽（盖掉实例 2 写行那次发布的根；
/// 读根环核落点公式，C743，手摆进别的槽的根照读坏算）放一条实例 1、txg 4 的根——照抄实例 1 暖机那条
/// 树表 0 条的根、只改 txg 与实例表指针：实例表指针两条位置条目都指槽 100（单元区起点之下）。按最新那条根（实例 2 暖机那一条）的实例表，它是被抛弃的根。
/// 预期：这条根的账算「解不开」，它是树表 0 条的一版、从中央映射也现算不成，第二次重开在取号之前拒可写、报出这条根（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算），不 panic。
/// 影子账若只判两条位置条目同槽，槽 100 进 `isolate_abandoned`，在 `DeviceFreeMap::index` 的 expect 上 panic。
#[test]
fn an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking(
) {
    let (mut devices, first) = pool_after_the_first_file();
    let first_remount =
        mount_writable(&parameters(), &mut devices).expect("第一次重开：实例 2 写行、暖机");
    assert_eq!(first_remount.output.instance, InstanceGeneration(2));
    let roots = roots_in_the_ring(&devices);
    let (_, warm_up_root_of_instance_one) = roots
        .iter()
        .find(|(_, root)| {
            root.instance == first.root.instance && root.checkpoint_txg == CheckpointTxg(1)
        })
        .copied()
        .expect("实例 1 暖机那条 txg 1 的根还在环里");
    let system_configuration = system_configuration_of(&devices);
    let mut abandoned = warm_up_root_of_instance_one;
    abandoned.checkpoint_txg = CheckpointTxg(first.root.checkpoint_txg.0 + 1);
    for location in &mut abandoned.instance_table.locations {
        location.slot = SLOT_BELOW_THE_UNIT_AREA_NOBODY_USES;
    }
    let formula_ring_slot = target_for_publish(
        abandoned.checkpoint_txg,
        system_configuration
            .immutable
            .sizes
            .root_ring_slots_per_region,
    );
    assert!(
        roots
            .iter()
            .any(|(ring_slot, root)| *ring_slot == formula_ring_slot
                && root.instance == first_remount.output.instance
                && root.checkpoint_txg == abandoned.checkpoint_txg),
        "前提：txg 4 的落点槽里是实例 2 写行那次发布的根，它之后还有实例 2 暖机那一条更新的根"
    );
    write_root_into_the_ring_slot(&mut devices, formula_ring_slot, &abandoned);
    // 指针槽号不在单元区里的那条被抛弃根算「账读不出」；它是暖机那一版（树表 0 条、映射根是空指针），从中央映射也现算不成，
    // 重读一次仍不成就在取号之前拒可写（C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算那一条的最后一支）——不 panic。
    let refusal = mount_writable(&parameters(), &mut devices)
        .expect_err("被抛弃根的指针不在单元区里、映射也现算不成：拒可写，不 panic");
    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = &refusal else {
        panic!("拒因是「重读一次仍读不出」：{refusal:?}");
    };
    assert!(
        matches!(
            still_unreadable.as_ref(),
            StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { abandoned_root }
                if abandoned_root.instance == first.root.instance
                    && abandoned_root.checkpoint_txg == CheckpointTxg(first.root.checkpoint_txg.0 + 1)
        ),
        "拒因点名那条被抛弃根：{still_unreadable:?}"
    );
}

// ─── 代码审阅第 35 条：盘比单元区起点还短 ───

/// 场景：只做过 mkfs 的池，盘 1 换成一块 700 MiB 的盘（前 700 MiB 与原来逐字节相同）：系统配置、根环都读得到，单元读盘 0 那一份。
/// 预期：可写挂载在读根环之前报盘 1 末尾在单元区起点（784 MiB）之前。走到重建分配器的话，`DeviceFreeMap::new` 里单元区槽数的减法下溢 panic。
#[test]
fn a_device_ending_before_the_unit_area_start_is_refused_by_the_writable_mount_before_any_write() {
    let mut devices = formatted_pool();
    let short_device_bytes = 700 * MEBIBYTE;
    let mut short_device = SparseBlockDevice::new(short_device_bytes, SECTOR_PHYSICAL_BLOCK_SIZE);
    short_device.image = device_mut(&mut devices, DeviceIdentity(1)).image.clone();
    *device_mut(&mut devices, DeviceIdentity(1)) = short_device;
    assert!(
        short_device_bytes < UNIT_AREA_START_SLOT * 16384,
        "700 MiB 在单元区起点 784 MiB 之前"
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart {
                device: DeviceIdentity(1),
                device_bytes,
            }) if device_bytes == short_device_bytes
        ),
        "盘容量下界：{refusal:?}"
    );
}

// ─── 第 35 条续：单元区槽数那一处减法 ───

/// 场景：700 MiB 的盘（单元区起点在 784 MiB）。预期：单元区槽数交回「盘末尾在单元区起点之前」，带盘的字节数与起点槽号；
/// 784 MiB 整的盘单元区是 0 槽。直接做减法的话，`overflow-checks = true` 下下溢 panic。
#[test]
fn the_unit_area_slot_count_of_a_device_ending_before_the_unit_area_start_is_an_error() {
    use singlefs_core::allocator::{
        unit_area_slots_of_device_starting_at, DeviceEndsBeforeTheUnitAreaStart, UnitAreaStart,
    };
    let start = UnitAreaStart::of_the_default_journal_ring();
    assert_eq!(
        unit_area_slots_of_device_starting_at(700 * MEBIBYTE, start),
        Err(DeviceEndsBeforeTheUnitAreaStart {
            device_bytes: 700 * MEBIBYTE,
            unit_area_start: SlotNumber(UNIT_AREA_START_SLOT),
        })
    );
    assert_eq!(
        unit_area_slots_of_device_starting_at(784 * MEBIBYTE, start),
        Ok(0),
        "盘末尾正好在单元区起点：0 槽"
    );
}
