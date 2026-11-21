//! 卸载之后的盘：数据单元没修复的判红，数据单元搬走、树表替换、挂载替换实例表、映射树叶重写之后的判绿。

use super::*;

/// 这张盘卸载之后不放回字节判真盘：卸载把 F 抬到现行那一版、改动之前的根离开候选集，坏的那一份还在盘上而没有候选根引用它。交回判红的不变量；只录那一趟的交 `None`。
fn violated_invariants_of_the_disk_after_the_unmount(
    table_name: &str,
    session: &mut MountedSession,
    devices: &mut Devices,
) -> Option<Vec<&'static str>> {
    unmount(
        &parameters(),
        devices,
        &mut session.allocator,
        &mut session.current,
        session.shadow_ledger,
    )
    .expect("卸载");
    pool_checker_violations_of_the_image(table_name, &memory_pool_of(devices)).map(|violations| {
        violations
            .into_iter()
            .map(|(invariant, _)| invariant)
            .collect()
    })
}

fn assert_the_disk_after_the_unmount_is_green(
    table_name: &str,
    session: &mut MountedSession,
    devices: &mut Devices,
) {
    if let Some(violated) =
        violated_invariants_of_the_disk_after_the_unmount(table_name, session, devices)
    {
        assert_eq!(violated, Vec::<&str>::new(), "{table_name}：判红的不变量");
    }
}

/// 对照：坏的那一份没修、现行那一版还引用它，卸载之后判真盘报 I-2.1 这一族（这一判有牙）。
#[test]
fn the_disk_after_the_unmount_with_the_data_unit_not_repaired_is_red() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let locations = mapped_locations(
        &version,
        &mapping_key_of_role(&version, TransactionUnit::Data(DAMAGED_UNIT)),
    )
    .expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    if let Some(violated) = violated_invariants_of_the_disk_after_the_unmount(
        "disk-after-the-unmount-data-unit-not-repaired",
        &mut session,
        &mut pool.devices,
    ) {
        assert!(
            ["I-2.1", "I-4.8", "I-7.4"]
                .iter()
                .any(|invariant| violated.contains(invariant)),
            "没修的那一份照报 I-2.1 这一族：{violated:?}"
        );
    }
}

/// 数据单元经会话读到、下一次改动之后搬走：卸载之后判真盘（弄坏的字节没放回）一条违例都没有。
/// 判别力：`crates/mutations.tsv`「卸载之后判真盘：搬迁那一批不发（坏的那一份现行版本还引用）」那一行。
#[test]
fn the_disk_after_the_unmount_with_the_data_unit_relocated_is_green() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    damage_the_data_unit_and_read_it_through_the_session(&session, &mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_the_disk_after_the_unmount_is_green(
        "disk-after-the-unmount-data-unit-relocated",
        &mut session,
        &mut pool.devices,
    );
}

/// 树表读到只坏一份、下一次改动换下：卸载之后判真盘一条违例都没有。
#[test]
fn the_disk_after_the_unmount_with_the_tree_table_replaced_is_green() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let table_locations = current_version_of(&session).root.tree_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_NODE);
    whole_first_file_read_through_the_session(&session, &pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_the_disk_after_the_unmount_is_green(
        "disk-after-the-unmount-tree-table-replaced",
        &mut session,
        &mut pool.devices,
    );
}

/// 实例表读到只坏一份（挂载建上一版时）、挂载写行就换了：卸载之后判真盘一条违例都没有。
#[test]
fn the_disk_after_the_unmount_with_the_instance_table_replaced_by_the_mount_is_green() {
    let mut pool = pool_holding(&content_of_units(3));
    let table_locations = pool.version.root.instance_table.locations;
    damage_the_copy_at(&mut pool.devices, &table_locations[0], BYTES_OF_A_DATA_UNIT);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_the_disk_after_the_unmount_is_green(
        "disk-after-the-unmount-instance-table-replaced-by-the-mount",
        &mut session,
        &mut pool.devices,
    );
}

/// 映射树叶挂载读到只坏一份、头一次改动之后那一批重写：卸载之后判真盘一条违例都没有。
#[test]
fn the_disk_after_the_unmount_with_the_mapping_tree_leaf_rewritten_is_green() {
    let mut pool = pool_holding(&content_of_units(
        UNITS_OF_A_FILE_WITH_A_MULTI_LEVEL_MAPPING_TREE,
    ));
    let leaf = leftmost_mapping_tree_leaf_of(&pool.version);
    damage_the_copy_at(&mut pool.devices, &leaf.locations[0], BYTES_OF_A_NODE);
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    assert_the_disk_after_the_unmount_is_green(
        "disk-after-the-unmount-mapping-tree-leaf-rewritten",
        &mut session,
        &mut pool.devices,
    );
}
