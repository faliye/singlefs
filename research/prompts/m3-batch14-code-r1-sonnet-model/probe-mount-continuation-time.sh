#!/usr/bin/env bash
# 探针 2：意图给一个与上一版不同的时间，崩在意图之后，可写挂载续做：续做的几批的记录时间是不是意图那次的值
# （内存里那份 TransactionOutput 的记录，和从盘上重建那一版读出的记录各看一遍）。副本里临时追加一条用例，跑完还原并核 sha256。
set -u
F="$MUT_W/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs"
cp "$F" "$F.bak"
cat >> "$F" <<'RS'

#[test]
fn probe_mount_continuation_copies_the_time_the_intent_wrote() {
    let mut pool = pool_with_a_three_unit_file("probe-qf-continuation");
    let time_before = pool.output.inode_record.expect("记录在").write_time_seconds;
    let time_of_the_intent = time_before + 600;
    {
        let publish_parameters = parameters();
        let previous = pool.output.clone();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let out = publish_unlink_intent(
            &mut writer, &mut pool.allocator, &previous, FIRST_FILE, time_of_the_intent, INSTANCE_OF_THE_PROCESS,
        ).expect("意图");
        pool.output = out;
    }
    let image = pool.memory_pool();
    let stream = SharedStream::new();
    let (mounted, devices) = mount_a_copy_of_the_image_with_batches_of_one(&image, &stream);
    let record_standing = mounted.current.version_of_trees().expect("续做之后有树").record.clone();
    let continued_image = memory_pool_of_sparse_devices(&devices);
    let last = mounted.output.warm_up_publishes.last().unwrap_or(&mounted.output.row_publish).root().checkpoint_txg;
    let mut roots: Vec<_> = disk_snapshot(&continued_image, &stream).readable_roots.into_iter().filter(|r| r.checkpoint_txg > last).collect();
    roots.sort_by_key(|r| r.checkpoint_txg);
    let in_memory: Vec<Option<u64>> = mounted.output.deletion_queue_continuation.iter().map(|s| s.inode_record.map(|r| r.write_time_seconds)).collect();
    let from_disk: Vec<Option<u64>> = roots.iter().map(|root| rebuilt_version_of_trees(&continued_image, root, &record_standing).inode_record.map(|r| r.write_time_seconds)).collect();
    eprintln!("PROBE time_before={time_before} intent={time_of_the_intent} continuation_in_memory={in_memory:?} continuation_from_disk={from_disk:?}");
    assert!(in_memory.iter().flatten().all(|t| *t == time_of_the_intent));
    assert!(from_disk.iter().flatten().all(|t| *t == time_of_the_intent));
    assert!(in_memory.iter().flatten().count() >= 1);
}
RS
cd "$MUT_W" && cargo test --offline -p singlefs-harness --test unlink_idempotence_criteria_and_rollback_to_the_emptied_version probe_ -- --nocapture 2>&1 | grep -E 'PROBE|test result|^error|panicked|^test '
mv "$F.bak" "$F"; sha256sum "$F"
