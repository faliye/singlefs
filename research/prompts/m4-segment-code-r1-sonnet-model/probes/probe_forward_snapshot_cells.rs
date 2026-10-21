//! 正推腿探针（T1、T3、T4）：
//! ① 快照条目随 NewInodes、推环的空发布、unlink 的意图与各批原样续上；文件集合为空的一版上建快照被拒、删快照放行，删完之后活头前驱回到 0。
//! ② 销毁中的快照：建快照被拒、读快照照读、回退拒、读树表的读者放行、崩溃之后可写挂载续做到删完。
mod common;

use common::{build_pool, file_content, parameters, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration, TreeIdentifier};
use singlefs_core::mount::{
    mount_writable, roll_back_by_a_forward_publish, RollbackError, RollbackTarget,
    VersionOfTheRollback,
};
use singlefs_core::mounted_session::{MountedSession, UserChange, UserChangeRefused};
use singlefs_core::records::{TREE_KIND_INODE, TREE_KIND_SNAPSHOT_INODE_ROOT};
use singlefs_core::recovery::{
    read_the_first_file_of_a_snapshot, user_visible_tree_root_pointers, SnapshotFileReading,
};
use singlefs_core::transaction::{
    publish_create_snapshot, publish_overwrite, publish_snapshot_destruction_intent, FirstFile,
    PoolWriter, PublishError, SnapshotCreationRefusal,
};

const INSTANCE: InstanceGeneration = InstanceGeneration(1);

#[test]
fn probe_forward_snapshot_entries_ride_through_every_kind_of_publish_and_an_emptied_file_set_refuses_only_the_creation() {
    let mut pool = build_pool("probe-forward-snapshot-carry");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("建快照");
    let entries_at_creation = session.current.version_of_trees().expect("有树").snapshot_entries();
    assert_eq!(entries_at_creation.len(), 1);
    let snapshot: TreeIdentifier = entries_at_creation[0].inode_root.tree;
    let live_prev_after_creation = session
        .current
        .version_of_trees()
        .unwrap()
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_INODE)
        .unwrap()
        .previous_snapshot_txg;
    let mut versions_checked = 0usize;
    let mut carried_everywhere = true;
    let mut note = |label: &str, versions: &[singlefs_core::transaction::TransactionOutput]| {
        for version in versions {
            let carried = version.snapshot_entries() == entries_at_creation;
            let live_prev = version
                .tree_table_entries
                .iter()
                .find(|entry| entry.kind == TREE_KIND_INODE)
                .unwrap()
                .previous_snapshot_txg;
            carried_everywhere &= carried && live_prev == live_prev_after_creation;
            versions_checked += 1;
            println!(
                "name=probe_forward_carry kind={label} txg={} pairs_equal={carried} live_head_previous_snapshot_txg_equal={}",
                version.root.checkpoint_txg.0,
                live_prev == live_prev_after_creation
            );
        }
    };
    let published = session
        .publish_user_change(
            &mut devices,
            UserChange::NewInodes { count: 2, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 10 },
        )
        .expect("建 inode");
    note("new_inodes", &published.versions_published);
    match session.push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(&mut devices) {
        Ok(pushed) => note("empty_publishes_of_the_ring_push", &pushed.versions_published),
        Err(refusal) => println!("name=probe_forward_ring_push refused={refusal:?}"),
    }
    let unlinked = session
        .publish_user_change(
            &mut devices,
            UserChange::Unlink { inode: InodeNumber(1), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 20 },
        )
        .expect("unlink 第一个文件");
    note("unlink_intent_and_batches", &unlinked.versions_published);
    for extra_inode in [2_u64, 3] {
        let more = session
            .publish_user_change(
                &mut devices,
                UserChange::Unlink { inode: InodeNumber(extra_inode), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 30 },
            )
            .expect("unlink 新建的空 inode");
        note("unlink_of_a_new_inode", &more.versions_published);
    }
    println!(
        "name=probe_forward_version_kind_after_unlinking_everything is_with_trees_and_no_file={}",
        matches!(session.current, singlefs_core::transaction::PoolVersion::WithTreesAndNoFile(_))
    );
    assert!(carried_everywhere, "每一版都原样续着那一对快照条目与活头前驱");
    // 文件集合为空的一版：建快照拒，删快照放行。
    let creation_on_the_emptied_version = session.publish_user_change(&mut devices, UserChange::CreateSnapshot);
    println!(
        "name=probe_forward_creation_on_an_emptied_file_set result={}",
        match &creation_on_the_emptied_version {
            Ok(_) => "Ok".to_string(),
            Err(UserChangeRefused::NoFileVersionToChange { .. }) => "Err(NoFileVersionToChange)".to_string(),
            Err(other) => format!("Err(other:{other:?})"),
        }
    );
    assert!(matches!(creation_on_the_emptied_version, Err(UserChangeRefused::NoFileVersionToChange { .. })));
    let destroyed = session
        .publish_user_change(&mut devices, UserChange::DestroySnapshot { snapshot })
        .expect("文件集合为空的一版上删快照放行");
    let after = session.current.version_of_trees().expect("有树");
    let live_prev_after_destruction = after
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_INODE)
        .unwrap()
        .previous_snapshot_txg;
    println!(
        "name=probe_forward_destruction_on_an_emptied_file_set versions_published={} snapshot_entries_left={} live_head_previous_snapshot_txg_after={}",
        destroyed.versions_published.len(),
        after.snapshot_entries().len(),
        live_prev_after_destruction.0
    );
    assert!(after.snapshot_entries().is_empty());
    assert_eq!(live_prev_after_destruction, CheckpointTxg(0), "删完最后一个快照，活头前驱传回被删那个的前驱 0");
    pool.devices = Some(devices);
}

#[test]
fn probe_forward_a_snapshot_being_destroyed_refuses_creation_and_rollback_is_still_readable_and_is_continued_by_the_mount() {
    let mut pool = build_pool("probe-forward-snapshot-being-destroyed");
    let publish_parameters = parameters();
    let created = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(&mut writer, &mut pool.allocator, &previous, INSTANCE).expect("建快照")
    };
    pool.output = created.clone();
    let snapshot = created
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
        .expect("一个快照")
        .tree;
    let content: Vec<u8> = (0..2600_u32).map(|index| u8::try_from((index * 5 + 1) % 241).unwrap()).collect();
    let overwritten = {
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &previous,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            INSTANCE,
        )
        .expect("覆盖写")
    };
    pool.output = overwritten.clone();
    let intent = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_snapshot_destruction_intent(&mut writer, &mut pool.allocator, &overwritten, snapshot, INSTANCE)
            .expect("意图")
    };
    pool.output = intent.clone();
    // (a) 建快照被拒。
    let creation = {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_create_snapshot(&mut writer, &mut pool.allocator, &intent, INSTANCE)
    };
    println!("name=probe_forward_creation_while_destroying result={:?}", creation.as_ref().map(|_| "Ok").map_err(|e| format!("{e:?}")));
    assert!(matches!(
        creation,
        Err(PublishError::SnapshotCreationRefused(SnapshotCreationRefusal::SnapshotBeingDestroyed { .. }))
    ));
    // (b) 读快照：销毁中的照样读得出建它那一刻的内容。
    let reading = read_the_first_file_of_a_snapshot(&pool.memory_pool(), &intent.root, snapshot);
    println!("name=probe_forward_read_while_destroying result={}", match &reading {
        Ok(SnapshotFileReading::File(bytes)) => format!("File({} bytes, equals_the_content_at_creation={})", bytes.len(), *bytes == file_content()),
        other => format!("{other:?}"),
    });
    assert_eq!(reading, Ok(SnapshotFileReading::File(file_content())));
    // (c) 回退被拒（现行那一版带销毁中的快照）。
    let rollback = {
        let mut devices = pool.devices.take().expect("镜像还开着");
        let mut current = intent.clone();
        let target = RollbackTarget { instance: INSTANCE, checkpoint_txg: CheckpointTxg(intent.root.checkpoint_txg.0 - 1) };
        let result = roll_back_by_a_forward_publish(&publish_parameters, &mut devices, &mut pool.allocator, &mut current, target);
        pool.devices = Some(devices);
        result
    };
    println!("name=probe_forward_rollback_while_destroying result={:?}", rollback.as_ref().map(|_| "Ok").map_err(|e| format!("{e:?}")));
    assert!(matches!(
        rollback,
        Err(RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { version: VersionOfTheRollback::Current, snapshots: 1 })
    ));
    // (d) 读树表的读者放行码 14 / 15 与 deadlist。
    let pointers = user_visible_tree_root_pointers(&pool.memory_pool(), &intent.root);
    println!("name=probe_forward_user_visible_tree_root_pointers snapshot_tree_roots={:?}", pointers.as_ref().map(|p| p.snapshot_tree_roots.len()));
    assert_eq!(pointers.expect("放行").snapshot_tree_roots.len(), 2);
    // (e) 崩溃之后的可写挂载续做到删完。
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    let after = mounted.current.version_of_trees().expect("有树");
    println!(
        "name=probe_forward_mount_after_the_intent snapshot_entries_left={} snapshots_being_destroyed_left={} deadlist_entries_left={} kept_mapping_entries_left={}",
        after.snapshot_entries().len(),
        after.snapshots_being_destroyed.len(),
        after.deadlist_entries.len(),
        after.mapping_entries_kept_for_the_snapshots.len()
    );
    assert!(after.snapshot_entries().is_empty() && after.snapshots_being_destroyed.is_empty());
    assert!(after.deadlist_entries.is_empty() && after.mapping_entries_kept_for_the_snapshots.is_empty());
    let _ = devices;
}
