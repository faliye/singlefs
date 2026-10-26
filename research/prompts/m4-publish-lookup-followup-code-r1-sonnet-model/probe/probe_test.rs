// 读数用例（追加在测试文件末尾，只在 instrument.py 改过的副本里存在）：一个文件整份重写 n 个数据单元的一次发布里，
// 发布路径上几处线性找 / 列表 contains 比较了几次。比较次数读 singlefs_core::transaction 里 instrument.py 加的 PROBE_* 计数。
fn reset_probe_counters() {
    use singlefs_core::transaction as t;
    use std::sync::atomic::Ordering::Relaxed;
    for counter in [
        &t::PROBE_UNITS_LEN,
        &t::PROBE_REWRITTEN_LEN,
        &t::PROBE_WRITTEN_FIND,
        &t::PROBE_NAMED_FIND,
        &t::PROBE_ROLES_NAMED_CONTAINS,
        &t::PROBE_KEPT_CONTAINS_RELEASE,
        &t::PROBE_KEPT_CONTAINS_MAPPED_ROLES,
    ] {
        counter.store(0, Relaxed);
    }
}

fn print_probe_counters(scenario: &str, n: usize) {
    use singlefs_core::transaction as t;
    use std::sync::atomic::Ordering::Relaxed;
    println!(
        "PROBE scenario={scenario} n={n} units={} rewritten={} written_find={} named_find={} roles_named_contains={} kept_contains_release={} kept_contains_mapped_roles={}",
        t::PROBE_UNITS_LEN.load(Relaxed),
        t::PROBE_REWRITTEN_LEN.load(Relaxed),
        t::PROBE_WRITTEN_FIND.load(Relaxed),
        t::PROBE_NAMED_FIND.load(Relaxed),
        t::PROBE_ROLES_NAMED_CONTAINS.load(Relaxed),
        t::PROBE_KEPT_CONTAINS_RELEASE.load(Relaxed),
        t::PROBE_KEPT_CONTAINS_MAPPED_ROLES.load(Relaxed),
    );
}

#[test]
fn probe_linear_finds_on_the_publish_path_by_file_size() {
    for n in [16usize, 32, 64, 128, 256] {
        let mut pool = pool_with_files_written_once(1, n);
        let inode = pool.inodes[0];
        reset_probe_counters();
        whole_rewrite_published_once(
            &mut pool,
            inode,
            inode.0 + 1_000_003,
            n,
            common::FIXED_WRITE_TIME_SECONDS + 5_000,
        );
        print_probe_counters("whole_rewrite_no_snapshot", n);
    }
    for n in [16usize, 32, 64, 128, 256] {
        let mut pool = pool_with_files_written_once(1, n);
        pool.session
            .publish_user_change(&mut pool.devices, UserChange::CreateSnapshot)
            .unwrap_or_else(|refusal| panic!("建快照：{refusal:?}"));
        let inode = pool.inodes[0];
        reset_probe_counters();
        whole_rewrite_published_once(
            &mut pool,
            inode,
            inode.0 + 2_000_003,
            n,
            common::FIXED_WRITE_TIME_SECONDS + 7_000,
        );
        print_probe_counters("whole_rewrite_under_a_snapshot", n);
    }
}
