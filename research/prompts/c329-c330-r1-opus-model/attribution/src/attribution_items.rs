// 第二遍的附加项：include! 进 `mod model`，与第一遍搬进来的全部项同在一个模块。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RollbackTableBase {
    /// 回退那次发布的实例表以挂载时最新可读的根引用的那一版为底（第一遍的取法）。
    NewestReadableRootTable,
    /// 以 R_old 自己引用的那一版为底（D23（journal 的角色与格式） 已定项 14「从 R_old 那棵账重新载入」推到实例表上）。
    RollbackTargetTable,
}

/// 与 `run_mount` 同一个过程，只在回退时按 `table_base` 选实例表的底；普通恢复与第一遍取法原样调 `run_mount`。
fn run_mount_with_table_base(rules: &Rules, pool: &mut Pool, plan: &MountPlan, table_base: RollbackTableBase) -> MountOutcome {
    let target_key = match (plan.rollback_to, table_base) {
        (Some(target_key), RollbackTableBase::RollbackTargetTable) => target_key,
        (None, RollbackTableBase::NewestReadableRootTable | RollbackTableBase::RollbackTargetTable)
        | (Some(_), RollbackTableBase::NewestReadableRootTable) => return run_mount(rules, pool, plan),
    };
    let selected = newest_readable(pool, &plan.unreadable);
    let acquired = acquire_instance(pool, &plan.unreadable);
    let target = pool.roots.iter().find(|root| (root.instance, root.txg) == target_key).cloned().expect("回退目标在根环里");
    let ring_highest = pool.roots.iter().filter(|root| is_readable(root, &plan.unreadable)).map(|root| root.txg).max().unwrap_or(0);
    let rows = rollback_rows(&target.table, &target, acquired);
    let txg = first_txg(rules, pool, &plan.unreadable, ring_highest);
    let mut counter = target.covered_counter;
    let mut next_transaction = 1;
    let mut landed = Vec::new();
    let first_publication =
        Publication { label: format!("i{acquired}t{txg}"), data_units: plan.data_units, persistent_nodes: plan.persistent_nodes };
    let (first_root, first_in_memory) =
        publish(pool, &target, acquired, txg, rows.clone(), &mut next_transaction, target.txg, &first_publication, plan.first, &mut counter);
    if let Some(root) = first_root {
        landed.push(root);
    }
    if let (Reach::Complete, Some(second_reach)) = (plan.first, plan.second) {
        let second_publication = Publication { label: format!("i{acquired}t{}", txg + 1), data_units: 0, persistent_nodes: 0 };
        let (second_root, _) = publish(
            pool, &first_in_memory, acquired, txg + 1, rows.clone(), &mut next_transaction, target.txg, &second_publication, second_reach, &mut counter,
        );
        if let Some(root) = second_root {
            landed.push(root);
        }
    }
    MountOutcome { selected, base: target, applied_transaction: 0, acquired, first_txg: txg, rows, landed }
}

/// 与第一遍的 `explore` 同一个枚举，挂载换成 `run_mount_with_table_base`。
#[allow(clippy::too_many_arguments, reason = "枚举的全部状态各有名字")]
fn explore_with(
    rules: &Rules,
    pool: &Pool,
    table_base: RollbackTableBase,
    depth: usize,
    faults: u32,
    script: &mut Vec<String>,
    seen: (bool, bool),
    result: &mut EnumerationResult,
) {
    if depth == ENUMERATED_MOUNTS {
        let mut next = pool.clone();
        let outcome = run_mount(rules, &mut next, &plan(Reach::RootNoSuperblock, None));
        let root = outcome.landed.first().cloned().expect("RootNoSuperblock 必落根");
        let tally = evaluate(rules, &next, &root);
        let line = format!("    next_mount_first_root {}", render_mounted(rules, &next, &root));
        let final_seen = record(result, &tally, seen, faults, script, &line);
        result.histories += 1;
        result.histories_with_u1 += u64::from(final_seen.0);
        result.histories_with_u2 += u64::from(final_seen.1);
        return;
    }
    for rollback in [false, true] {
        for unreadable_count in 0..3usize {
            let unreadable = newest_landed(pool, unreadable_count);
            if unreadable.len() < unreadable_count {
                continue;
            }
            let rollback_to = if rollback {
                let newest = newest_readable(pool, &unreadable);
                match rollback_target(pool, &unreadable, &newest) {
                    Some(target) => Some((target.instance, target.txg)),
                    None => continue,
                }
            } else {
                None
            };
            for (first, second) in STAGES {
                let mount_plan = MountPlan {
                    rollback_to,
                    unreadable: unreadable.clone(),
                    admission_short: false,
                    first,
                    second,
                    data_units: 1,
                    persistent_nodes: 0,
                    skip_rows_when_looks_clean: false,
                };
                let mut next = pool.clone();
                let outcome = run_mount_with_table_base(rules, &mut next, &mount_plan, table_base);
                let mount_faults = faults + u32::try_from(unreadable.len()).expect("至多 2") + 1;
                let mut line = format!("    {}", render_outcome(depth + 2, &mount_plan, &outcome));
                let mut now_seen = seen;
                if let Some(mounted) = outcome.landed.last() {
                    let tally = evaluate(rules, &next, mounted);
                    line.push_str(&format!(" | mounted=({},{}) table={} {}", mounted.instance, mounted.txg, render_rows(&mounted.table), render_tally(&next, &tally)));
                    now_seen = record(result, &tally, seen, mount_faults, script, &line);
                }
                script.push(line);
                explore_with(rules, &next, table_base, depth + 1, mount_faults, script, now_seen, result);
                script.pop();
            }
        }
    }
}

/// 与第一遍 `first_mount(pool, 1, Some((UnitsOnly, 1, 0)))` 同形，只是 (1,3) 不带一直被引用的节点：把「重放施加进来的节点」那一类 U2 摘出去。
fn first_mount_without_nodes(pool: &mut Pool) {
    let acquired = acquire_instance(pool, &[]);
    let mut parent = pool.roots[0].clone();
    let mut counter = 0;
    let mut next_transaction = 1;
    for txg in 1..=3u64 {
        let publication = Publication { label: format!("i{acquired}t{txg}"), data_units: u64::from(txg == 3), persistent_nodes: 0 };
        let (landed, _) =
            publish(pool, &parent, acquired, txg, BTreeMap::new(), &mut next_transaction, 0, &publication, Reach::Complete, &mut counter);
        parent = landed.expect("Complete 必落");
    }
    let publication = Publication { label: format!("i{acquired}t4"), data_units: 1, persistent_nodes: 0 };
    let _ = publish(pool, &parent, acquired, 4, BTreeMap::new(), &mut next_transaction, 0, &publication, Reach::UnitsOnly, &mut counter);
}

/// 择根倒挂：M2 写成 (2,4)..(2,9)；M3 那四条 (2,6)..(2,9) 暂时读不出、选 (2,5)，暖机两次（按区域 = txg mod 3、归属 0 / 1 / 0 看覆盖了哪几块盘）
/// 之后写成一次带码 1 的事务（fsync 已返回），然后崩；M4 全部读得出。
fn selection_inversion(rules: &Rules) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("SELECTION_INVERSION rules={}", rules.name)];
    first_mount(&mut pool, 1, None);
    let second = run_mount(rules, &mut pool, &plan(Reach::Complete, Some(Reach::Complete)));
    let mut parent = second.landed.last().cloned().expect("两次都写成");
    let mut counter = parent.covered_counter;
    let mut next_transaction = 2;
    for txg in parent.txg + 1..=9 {
        let publication = Publication { label: format!("i2t{txg}"), data_units: 1, persistent_nodes: 0 };
        let (landed, _) =
            publish(&mut pool, &parent, 2, txg, parent.table.clone(), &mut next_transaction, second.base.txg, &publication, Reach::Complete, &mut counter);
        parent = landed.expect("Complete 必落");
    }
    lines.push(format!("  M2 acquired=2 rows_written={} landed=(2,4)..(2,{}) all_complete", render_rows(&second.rows), parent.txg));
    let mut third_plan = plan(Reach::Complete, Some(Reach::Complete));
    third_plan.unreadable = vec![(2, 6), (2, 7), (2, 8), (2, 9)];
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
        "  M3 unreadable={:?} selected=({},{}) acquired={} rows_written={} first_txg={} warm_up_roots_on_disks={disks:?} confirmed_publication=({},{confirmed_txg}) crash",
        third_plan.unreadable, third.selected.instance, third.selected.txg, third.acquired, render_rows(&third.rows), third.first_txg, third.acquired
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

pub fn attribution_main() {
    println!("CONFIG second_pass=attribution enumerated_mounts_after_first={ENUMERATED_MOUNTS} stages={}", STAGES.len());
    for rules in c330_rules() {
        selection_inversion(&rules).iter().for_each(|line| println!("{line}"));
    }
    for (base_label, with_nodes) in [("m1_with_nodes", true), ("m1_without_nodes", false)] {
        let mut base = mkfs_pool();
        if with_nodes {
            first_mount(&mut base, 1, Some((Reach::UnitsOnly, 1, 0)));
        } else {
            first_mount_without_nodes(&mut base);
        }
        for table_base in [RollbackTableBase::NewestReadableRootTable, RollbackTableBase::RollbackTargetTable] {
            for rules in c330_rules() {
                let mut result = EnumerationResult::default();
                explore_with(&rules, &base, table_base, 0, 0, &mut Vec::new(), (false, false), &mut result);
                println!(
                    "ENUM2 base={base_label} rollback_table_base={table_base:?} rules={} histories={} evaluations={} u1_units={} u2_units={} histories_with_u1={} histories_with_u2={} evaluations_with_I-3.8_violation={}",
                    rules.name, result.histories, result.evaluations, result.u1_units, result.u2_units, result.histories_with_u1, result.histories_with_u2, result.i38_violations
                );
                for (kind, example) in [("U1", &result.fewest_faults_u1), ("U2", &result.fewest_faults_u2)] {
                    if let Some((faults, history)) = example {
                        println!("ENUM2_EXAMPLE base={base_label} rollback_table_base={table_base:?} rules={} kind={kind} faults={faults}", rules.name);
                        history.iter().for_each(|line| println!("{line}"));
                    }
                }
            }
        }
    }
}
