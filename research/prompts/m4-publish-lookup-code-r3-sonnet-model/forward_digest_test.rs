
// ---- 正推腿的差分用例（追加在改前与改后两棵树的同名测试文件末尾，两边一字不差）----
#[test]
fn forward_leg_snapshot_compaction_unlink_destroy_image_digest() {
    let PoolBuilt {
        mut devices,
        mut session,
        inodes,
    } = build_a_pool_of_files(
        PoolOfFiles {
            files: 16,
            data_units_per_file: 4,
        },
        &mut |_| {},
    );
    let mut write_time_seconds = common::FIXED_WRITE_TIME_SECONDS + 2_000_000;
    let mut digests = Vec::new();
    // 建快照
    session
        .publish_user_change(&mut devices, UserChange::CreateSnapshot)
        .expect("建快照");
    digests.push(("after_create_snapshot", digest_of_the_images(&devices)));
    let snapshot = version_of(&session)
        .tree_table_entries
        .iter()
        .find(|entry| entry.kind == singlefs_core::records::TREE_KIND_SNAPSHOT_INODE_ROOT)
        .map(|entry| entry.tree)
        .expect("快照在树表里");
    // 快照下整份重写几个文件（快照判定那一圈进 deadlist 的一支）
    for (position, inode) in inodes.iter().enumerate() {
        if position % 3 != 0 {
            continue;
        }
        let content = content_of_data_units(4, inode.0 + 7_000_001);
        write_time_seconds += 1;
        publish_and_look_at_every_version(
            &mut session,
            &mut devices,
            UserChange::SequentialWriteOfAnInode {
                inode: *inode,
                content: &content,
                write_time_seconds,
            },
            &mut |_| {},
        );
    }
    digests.push(("after_overwrites_under_snapshot", digest_of_the_images(&devices)));
    // 整理推到停（留存的映射条目、整理发批核源落点）
    session
        .allocator
        .set_compaction_switches(product_switches_with_the_batch_size(2));
    let mut batches = 0;
    for _ in 0..COMPACTION_RUNS_AT_MOST {
        let run = session.run_compaction(&mut devices).expect("整理一趟");
        batches += run.batches_published;
        if let CompactionRunOutcome::Failed(failure) = run.outcome {
            panic!("整理失败：{failure:?}");
        }
        if run.versions_published.is_empty() {
            break;
        }
    }
    digests.push(("after_compaction_with_snapshot", digest_of_the_images(&devices)));
    // 再重写、unlink、删快照、再整理
    let content = content_of_data_units(4, 9_000_003);
    write_time_seconds += 1;
    publish_and_look_at_every_version(
        &mut session,
        &mut devices,
        UserChange::SequentialWriteOfAnInode {
            inode: inodes[0],
            content: &content,
            write_time_seconds,
        },
        &mut |_| {},
    );
    write_time_seconds += 1;
    publish_and_look_at_every_version(
        &mut session,
        &mut devices,
        UserChange::Unlink {
            inode: inodes[1],
            write_time_seconds,
        },
        &mut |_| {},
    );
    digests.push(("after_unlink", digest_of_the_images(&devices)));
    session
        .publish_user_change(&mut devices, UserChange::DestroySnapshot { snapshot })
        .expect("删快照");
    digests.push(("after_destroy_snapshot", digest_of_the_images(&devices)));
    for _ in 0..COMPACTION_RUNS_AT_MOST {
        let run = session.run_compaction(&mut devices).expect("整理一趟");
        batches += run.batches_published;
        if run.versions_published.is_empty() {
            break;
        }
    }
    digests.push(("after_final_compaction", digest_of_the_images(&devices)));
    drop(session);
    let violations = violations_of_the_pool_checker(&memory_pool_of(&devices));
    assert!(violations.is_empty(), "池级 checker：{violations:?}");
    for (name, digest) in &digests {
        println!("FWD_DIGEST name={name} sha256={digest}");
    }
    println!("FWD_DIGEST batches={batches}");
}
