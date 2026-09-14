// 第二遍的另一个入口：择根倒挂——读不出的是另一个实例的全部根，重放在实例边界停，接不回来。

/// M1 写成 (1,1)(1,2)(1,3)；M2 选 (1,3)，写成 (2,4)..(2,7)（暖机两次 + 两次带码 1 的发布）；M3 那四条 (2,4)..(2,7) 全部暂时读不出、选 (1,3)，
/// 重放在第一条实例 2 的记录处停；暖机两次（看覆盖了哪几块盘）之后写成一次带码 1 的事务（fsync 已返回），然后崩；M4 全部读得出。
fn inversion_across_instances(rules: &Rules) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("INVERSION_ACROSS_INSTANCES rules={}", rules.name)];
    first_mount(&mut pool, 1, None);
    let second = run_mount(rules, &mut pool, &plan(Reach::Complete, Some(Reach::Complete)));
    let mut parent = second.landed.last().cloned().expect("两次都写成");
    let mut counter = parent.covered_counter;
    let mut next_transaction = 2;
    for txg in parent.txg + 1..=parent.txg + 2 {
        let publication = Publication { label: format!("i2t{txg}"), data_units: 1, persistent_nodes: 0 };
        let (landed, _) =
            publish(&mut pool, &parent, 2, txg, parent.table.clone(), &mut next_transaction, second.base.txg, &publication, Reach::Complete, &mut counter);
        parent = landed.expect("Complete 必落");
    }
    let second_roots: Vec<(u32, u64)> = pool.roots.iter().filter(|root| root.instance == 2).map(|root| (root.instance, root.txg)).collect();
    lines.push(format!("  M2 acquired=2 rows_written={} landed={second_roots:?} all_complete", render_rows(&second.rows)));
    let mut third_plan = plan(Reach::Complete, Some(Reach::Complete));
    third_plan.unreadable = second_roots.clone();
    let third = run_mount(rules, &mut pool, &third_plan);
    let warm = third.landed.last().cloned().expect("两次都写成");
    let disks: Vec<usize> = third.landed.iter().map(|root| [0usize, 1, 0][usize::try_from(root.txg % 3).expect("小于 3")]).collect();
    let mut third_counter = warm.covered_counter;
    let mut third_transactions = 2;
    let confirmed_txg = warm.txg + 1;
    let before = pool.units.len();
    let publication = Publication { label: format!("i{}t{confirmed_txg}.fsync", third.acquired), data_units: 1, persistent_nodes: 0 };
    let _ = publish(
        &mut pool, &warm, third.acquired, confirmed_txg, warm.table.clone(), &mut third_transactions, third.base.txg, &publication, Reach::Complete, &mut third_counter,
    );
    let confirmed_unit = (before..pool.units.len()).find(|index| pool.units[*index].code == UnitCode::Data).expect("一个码 1");
    lines.push(format!(
        "  M3 unreadable={:?} selected=({},{}) after_replay=({},{}) acquired={} rows_written={} first_txg={} warm_up_roots_on_disks={disks:?} confirmed_publication=({},{confirmed_txg}) crash",
        third_plan.unreadable, third.selected.instance, third.selected.txg, third.base.instance, third.base.txg, third.acquired,
        render_rows(&third.rows), third.first_txg, third.acquired
    ));
    let fourth = run_mount(rules, &mut pool, &plan(Reach::Complete, Some(Reach::Complete)));
    let mounted = fourth.landed.last().cloned().expect("写成");
    let unit = &pool.units[confirmed_unit];
    lines.push(format!(
        "  M4 selected=({},{}) acquired={} rows_written={} | confirmed_unit={} referenced_by_mounted={} verdict={:?} | {}",
        fourth.selected.instance, fourth.selected.txg, fourth.acquired, render_rows(&fourth.rows), unit.label,
        mounted.referenced.contains(&confirmed_unit), judge(rules, &mounted, unit), render_mounted(rules, &pool, &mounted)
    ));
    lines
}

pub fn inversion_main() {
    for rules in c330_rules() {
        inversion_across_instances(&rules).iter().for_each(|line| println!("{line}"));
    }
}
