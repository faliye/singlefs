
// ---- 正推腿的覆盖用例（只追加在改后那棵树）：同一串操作，按处打印计数 ----
#[test]
fn forward_leg_coverage_of_the_sites_on_the_same_history() {
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
    let mut total = LookupWorkOfEachSite::default();
    let (created, work) = lookup_work_while(|| session.publish_user_change(&mut devices, UserChange::CreateSnapshot));
    created.expect("建快照");
    total = { let mut t = total; for (k, v) in work.work_of_each_site { let e = t.work_of_each_site.entry(k).or_default(); *e = e.plus(v); } t };
    for (position, inode) in inodes.iter().enumerate() {
        if position % 3 != 0 { continue; }
        let content = content_of_data_units(4, inode.0 + 7_000_001);
        write_time_seconds += 1;
        let work = publish_and_look_at_every_version(&mut session, &mut devices,
            UserChange::SequentialWriteOfAnInode { inode: *inode, content: &content, write_time_seconds }, &mut |_| {});
        for (k, v) in work.work_of_each_site { let e = total.work_of_each_site.entry(k).or_default(); *e = e.plus(v); }
    }
    session.allocator.set_compaction_switches(product_switches_with_the_batch_size(2));
    for _ in 0..COMPACTION_RUNS_AT_MOST {
        let (run, work) = run_compaction_and_count_its_lookups(&mut session, &mut devices);
        for (k, v) in work.work_of_each_site { let e = total.work_of_each_site.entry(k).or_default(); *e = e.plus(v); }
        if run.versions_published.is_empty() { break; }
    }
    for (site, work) in &total.work_of_each_site {
        println!("FWD_COVERAGE site={site:?} work={work:?}");
    }
}
