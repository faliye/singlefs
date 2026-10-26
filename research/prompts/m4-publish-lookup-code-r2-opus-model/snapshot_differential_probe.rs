
// ── 攻方腿 m4-publish-lookup-code-r2-opus 的差分探针（只拷进副本，改前、改后两份各接在这份测试文件末尾，原样相同）──
// 快照下的一串合法操作：建 S1、覆盖写、建 S2、覆盖写两次、删 S1（意图 + 全部批）、覆盖写、删 S2、再覆盖写。
// 每一版的内存版本（Debug，含每个单元的字节、deadlist 条目、留存的映射条目）与录制写流（Debug）各取一个摘要逐行打出，
// 改前改后的两份输出逐行比。摘要用 `DefaultHasher::new()`（固定键的 SipHash，同一个编译器两边相同）。
fn attack_probe_digest_of<Value: std::fmt::Debug>(value: &Value) -> (u64, usize) {
    use std::hash::{Hash, Hasher};
    let text = format!("{value:?}");
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    (hasher.finish(), text.len())
}

#[test]
fn attack_probe_snapshot_history_digests() {
    let mut pool = build_pool("attack-probe-snapshot-history");
    let mut versions: Vec<(String, TransactionOutput)> = Vec::new();
    let first_snapshot_version = create_snapshot_and_advance(&mut pool);
    let older = snapshot_trees(&first_snapshot_version)[0];
    versions.push(("create S1".into(), first_snapshot_version));
    versions.push(("overwrite 31".into(), overwrite_and_advance(&mut pool, 31)));
    let second_snapshot_version = create_snapshot_and_advance(&mut pool);
    let newer = *snapshot_trees(&second_snapshot_version).last().expect("两个快照");
    versions.push(("create S2".into(), second_snapshot_version));
    versions.push(("overwrite 37".into(), overwrite_and_advance(&mut pool, 37)));
    versions.push(("overwrite 41".into(), overwrite_and_advance(&mut pool, 41)));
    let (intent, batches) = destroy_the_snapshot_and_advance(&mut pool, older);
    versions.push(("destroy S1 intent".into(), intent));
    for (position, batch) in batches.into_iter().enumerate() {
        versions.push((format!("destroy S1 batch {position}"), batch));
    }
    versions.push(("overwrite 43".into(), overwrite_and_advance(&mut pool, 43)));
    let (intent, batches) = destroy_the_snapshot_and_advance(&mut pool, newer);
    versions.push(("destroy S2 intent".into(), intent));
    for (position, batch) in batches.into_iter().enumerate() {
        versions.push((format!("destroy S2 batch {position}"), batch));
    }
    versions.push(("overwrite 47".into(), overwrite_and_advance(&mut pool, 47)));
    for (step, version) in &versions {
        let (digest, bytes) = attack_probe_digest_of(version);
        println!(
            "attack_probe step={step} txg={} deadlist_entries={} kept_mapping_entries={} version_digest={digest:016x} version_debug_bytes={bytes}",
            version.root.checkpoint_txg.0,
            version.deadlist_entries.len(),
            version.mapping_entries_kept_for_the_snapshots.len()
        );
    }
    let (digest, bytes) = attack_probe_digest_of(&pool.stream.operations());
    println!("attack_probe write_stream operations={} digest={digest:016x} debug_bytes={bytes}", pool.stream.operation_count());
    let (digest, bytes) = attack_probe_digest_of(&pool.stream.retained_operations());
    println!("attack_probe retained_stream digest={digest:016x} debug_bytes={bytes}");
}
