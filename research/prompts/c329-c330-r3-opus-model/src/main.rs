//! C329 / C330 third round, attack leg (Opus). Attack surfaces X2 (the owner row of a switch is
//! overwritten by a later recovery's batch) and X3 (a switch inside the rollback publication, inside
//! a warm-up empty publication, and inside the first publication after R takes effect), against
//! P330'' (owner row, intermediates (i, 0, 0), rollback table based on R_old's) and R.
//! Deterministic, std only. Each scenario writes the durable write set by hand, builds the instance
//! table each arm would hold under the mounted root, and runs the D18 line 879 published predicate.
//! Truth per unit: referenced by the mounted root, and whether its publication is on the mounted
//! root's timeline.
//!   wrongly_published   = predicate says published, not referenced, not on the timeline
//!   wrongly_unpublished = predicate says unpublished, referenced by the mounted root

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnitCode {
    Data,     // code 1: birth <= T_pub or transaction number <= W
    Metadata, // code 2 / 3: birth <= T_pub only
}

#[derive(Clone, Debug)]
struct Unit {
    name: String,
    writer_instance: u32,
    transaction_number: u64,
    birth_txg: u64,
    code: UnitCode,
    referenced_by_mounted_root: bool,
    on_mounted_timeline: bool,
}

fn unit(
    name: &str,
    writer_instance: u32,
    transaction_number: u64,
    birth_txg: u64,
    code: UnitCode,
    referenced_by_mounted_root: bool,
    on_mounted_timeline: bool,
) -> Unit {
    Unit {
        name: name.to_string(),
        writer_instance,
        transaction_number,
        birth_txg,
        code,
        referenced_by_mounted_root,
        on_mounted_timeline,
    }
}

/// Which rule wrote a row: a recovery / switch batch row, the rollback row (flags bit0), or the
/// owner row of a switch's kept transactions (P330'' 3').
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowKind {
    Batch,
    Rollback,
    SwitchOwner,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct InstanceRow {
    instance: u32,
    published_txg: u64,
    max_applied_transaction: u64,
    kind: RowKind,
}

fn batch_row(instance: u32, published_txg: u64, max_applied_transaction: u64) -> InstanceRow {
    InstanceRow { instance, published_txg, max_applied_transaction, kind: RowKind::Batch }
}

fn rollback_row(instance: u32, published_txg: u64) -> InstanceRow {
    InstanceRow { instance, published_txg, max_applied_transaction: 0, kind: RowKind::Rollback }
}

fn owner_row(instance: u32, published_txg: u64, max_applied_transaction: u64) -> InstanceRow {
    InstanceRow { instance, published_txg, max_applied_transaction, kind: RowKind::SwitchOwner }
}

/// Which existing rows a later batch row may not overwrite. D18 line 879 protects the rollback row
/// only; X2 asks whether owner rows should get the same protection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Protection {
    RollbackRowOnly,
    RollbackAndOwnerRows,
}

/// Rows are unique per instance and a later write overwrites (D18 line 879), except that a
/// protected row is not overwritten by a later batch row.
fn apply_rows(base_table: &[InstanceRow], new_rows: &[InstanceRow], protection: Protection) -> Vec<InstanceRow> {
    let mut table: Vec<InstanceRow> = base_table.to_vec();
    for new_row in new_rows {
        match table.iter().position(|existing| existing.instance == new_row.instance) {
            Some(position) => {
                let existing_is_protected = match table[position].kind {
                    RowKind::Rollback => true,
                    RowKind::SwitchOwner => protection == Protection::RollbackAndOwnerRows,
                    RowKind::Batch => false,
                };
                if existing_is_protected && new_row.kind == RowKind::Batch {
                    continue;
                }
                table[position] = *new_row;
            }
            None => table.push(*new_row),
        }
    }
    table.sort_by_key(|table_row| table_row.instance);
    table
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verdict {
    Published,
    Unpublished,
    Corrupt,
}

/// D18 line 879, the global published predicate, in structure.
fn published_predicate(candidate: &Unit, table: &[InstanceRow], mounted_instance: u32, mounted_txg: u64) -> Verdict {
    if candidate.writer_instance > mounted_instance {
        return Verdict::Corrupt;
    }
    if candidate.writer_instance == mounted_instance {
        return if candidate.birth_txg <= mounted_txg { Verdict::Published } else { Verdict::Unpublished };
    }
    match table.iter().find(|table_row| table_row.instance == candidate.writer_instance) {
        None => Verdict::Published,
        Some(found_row) => {
            let by_birth = candidate.birth_txg <= found_row.published_txg;
            let by_transaction =
                candidate.transaction_number != 0 && candidate.transaction_number <= found_row.max_applied_transaction;
            let published = match candidate.code {
                UnitCode::Data => by_birth || by_transaction,
                UnitCode::Metadata => by_birth,
            };
            if published { Verdict::Published } else { Verdict::Unpublished }
        }
    }
}

struct Evaluation {
    wrongly_published: Vec<String>,
    wrongly_unpublished: Vec<String>,
    corrupt: Vec<String>,
}

fn evaluate(units: &[Unit], table: &[InstanceRow], mounted_instance: u32, mounted_txg: u64) -> Evaluation {
    let mut evaluation = Evaluation { wrongly_published: vec![], wrongly_unpublished: vec![], corrupt: vec![] };
    for candidate in units {
        match published_predicate(candidate, table, mounted_instance, mounted_txg) {
            Verdict::Published => {
                if !candidate.referenced_by_mounted_root && !candidate.on_mounted_timeline {
                    evaluation.wrongly_published.push(candidate.name.clone());
                }
            }
            Verdict::Unpublished => {
                if candidate.referenced_by_mounted_root {
                    evaluation.wrongly_unpublished.push(candidate.name.clone());
                }
            }
            Verdict::Corrupt => evaluation.corrupt.push(candidate.name.clone()),
        }
    }
    evaluation
}

fn format_table(table: &[InstanceRow]) -> String {
    table
        .iter()
        .map(|table_row| {
            let marker = match table_row.kind {
                RowKind::Batch => "",
                RowKind::Rollback => ",rollback",
                RowKind::SwitchOwner => ",owner",
            };
            format!("({},{},{}{})", table_row.instance, table_row.published_txg, table_row.max_applied_transaction, marker)
        })
        .collect::<Vec<_>>()
        .join("")
}

fn report(label: &str, units: &[Unit], table: &[InstanceRow], mounted_instance: u32, mounted_txg: u64) -> (usize, usize) {
    let evaluation = evaluate(units, table, mounted_instance, mounted_txg);
    println!(
        "  {label} mounted=({mounted_instance},{mounted_txg}) table={} wrongly_published={} {:?} wrongly_unpublished={} {:?} corrupt={}",
        format_table(table),
        evaluation.wrongly_published.len(),
        evaluation.wrongly_published,
        evaluation.wrongly_unpublished.len(),
        evaluation.wrongly_unpublished,
        evaluation.corrupt.len()
    );
    (evaluation.wrongly_published.len(), evaluation.wrongly_unpublished.len())
}

/// X2. Round two's S1 switch (instance 2's row-writing publication, txg 4, fails on a fixed-point
/// unit write with a successful probe; switch to 3, which keeps instance 2's transactions 1..3 and
/// writes the owner row (2, 0, 3)). Instance 3 lands only its resend root (3,4) (region 1, disk 1)
/// and crashes before its warm-up. M3: (3,4) is unreadable, recovery 4 selects (1,3) and batches
/// [1, 4); first txg 4 (ties (3,4), instance 4 wins), warm-up (4,5) (4,6). M4: (3,4) reads again,
/// recovery 5 selects (4,6). Control: (3,4) readable at M3, recovery 4 selects it.
fn scenario_owner_row_after_switch_instance_abandoned() -> Vec<(String, usize, usize)> {
    let mut results = vec![];
    let units_m3 = vec![
        unit("i1.t3.data1", 1, 1, 3, UnitCode::Data, true, true),
        unit("i1.t3.node", 1, 0, 3, UnitCode::Metadata, false, true), // superseded by instance 4's fixed point
        unit("i1.t4.accounting", 1, 0, 4, UnitCode::Metadata, false, false),
        unit("i1.t4.data2", 1, 2, 4, UnitCode::Data, false, false),
        unit("i2.t4.data1", 2, 1, 4, UnitCode::Data, false, false),
        unit("i2.t4.data2", 2, 2, 4, UnitCode::Data, false, false),
        unit("i2.t4.data3", 2, 3, 4, UnitCode::Data, false, false),
        unit("i2.t4.fixed", 2, 0, 4, UnitCode::Metadata, false, false),
        unit("i3.t4.fixed", 3, 0, 4, UnitCode::Metadata, false, false),
        unit("i4.t4.node", 4, 0, 4, UnitCode::Metadata, true, true),
        unit("i4.t6.accounting", 4, 0, 6, UnitCode::Metadata, true, true),
    ];
    // The table (3,4) references: instance 3's batch [1, 3) under P330'' = (1,3,0) + owner row (2,0,3).
    let table_of_switch_root =
        apply_rows(&[], &[batch_row(1, 3, 0), owner_row(2, 0, 3)], Protection::RollbackRowOnly);
    // Recovery 4's batch over [1, 4); P330'' 1: non-owner intermediates get (i, 0, 0).
    let recovery_four_rows = vec![batch_row(1, 3, 0), batch_row(2, 0, 0), batch_row(3, 0, 0)];
    let arms: Vec<(&str, Vec<InstanceRow>, Protection)> = vec![
        // The kb structure: the table is the version the selected root points to ((1,3)'s, empty).
        ("versioned_base_is_selected_root_table", vec![], Protection::RollbackRowOnly),
        // Hypothetical: recovery 4 sees (3,4)'s table although (3,4) is unreadable.
        ("hypothetical_global_table_plain_overwrite", table_of_switch_root.clone(), Protection::RollbackRowOnly),
        ("hypothetical_global_table_owner_row_protected", table_of_switch_root.clone(), Protection::RollbackAndOwnerRows),
    ];
    for (arm_name, base_table, protection) in arms {
        let table_m3 = apply_rows(&base_table, &recovery_four_rows, protection);
        let (wrong_published, wrong_unpublished) = report(&format!("M3 {arm_name}"), &units_m3, &table_m3, 4, 6);
        results.push((format!("X2 M3 {arm_name}"), wrong_published, wrong_unpublished));
        let table_m4 = apply_rows(&table_m3, &[batch_row(4, 6, 0)], protection);
        let mut units_m4 = units_m3.clone();
        units_m4.push(unit("i5.t7.accounting", 5, 0, 7, UnitCode::Metadata, true, true));
        let (wrong_published, wrong_unpublished) = report(&format!("M4 {arm_name}"), &units_m4, &table_m4, 5, 7);
        results.push((format!("X2 M4 {arm_name}"), wrong_published, wrong_unpublished));
    }
    let units_control = vec![
        unit("i1.t3.data1", 1, 1, 3, UnitCode::Data, true, true),
        unit("i1.t4.accounting", 1, 0, 4, UnitCode::Metadata, false, false),
        unit("i1.t4.data2", 1, 2, 4, UnitCode::Data, false, false),
        unit("i2.t4.data1", 2, 1, 4, UnitCode::Data, true, true),
        unit("i2.t4.data2", 2, 2, 4, UnitCode::Data, true, true),
        unit("i2.t4.data3", 2, 3, 4, UnitCode::Data, true, true),
        unit("i2.t4.fixed", 2, 0, 4, UnitCode::Metadata, false, false),
        unit("i3.t4.fixed", 3, 0, 4, UnitCode::Metadata, true, true),
        unit("i4.t7.accounting", 4, 0, 7, UnitCode::Metadata, true, true),
    ];
    // Recovery 4 selects (3,4): base = (3,4)'s table, batch [3, 4) = {3}; the owner row is outside it.
    let table_control = apply_rows(&table_of_switch_root, &[batch_row(3, 4, 0)], Protection::RollbackRowOnly);
    let (wrong_published, wrong_unpublished) =
        report("M3 control_switch_root_readable", &units_control, &table_control, 4, 7);
    results.push(("X2 M3 control_switch_root_readable".to_string(), wrong_published, wrong_unpublished));
    results
}

/// X3 (a). Switch inside the rollback publication. Ring: (1,1)..(1,5) of instance 1; instance 2
/// (recovery from (1,5), row (1,5,0)) published (2,6) (rows), warm-up (2,7) (2,8), and (2,9) with its
/// transaction 1. Mount 3: admin rollback to R_old = (1,3) (row (1,5,0) and 3 <= 5, F = 0). The
/// rollback publication (txg 10 = ring max + 1) carries, per P330'' 2, R_old's table (empty) plus the
/// rollback row (1,3,0) and the intermediate (2,0,0), and instance 3's own transactions 1, 2. A
/// fixed-point unit write fails, the probe write succeeds: switch to 4, which resends txg 10 (no
/// root of 3 exists, ring max is still 9), rewrites the fixed point and warms up (4,11) (4,12).
/// Faults: one transient unit-write error; no crash, no unreadable root.
fn scenario_switch_inside_rollback_publication() -> Vec<(String, usize, usize)> {
    let mut results = vec![];
    let units = vec![
        unit("i1.t3.data1", 1, 1, 3, UnitCode::Data, true, true),
        unit("i1.t3.node", 1, 0, 3, UnitCode::Metadata, true, true),
        unit("i1.t4.node", 1, 0, 4, UnitCode::Metadata, false, false),
        unit("i1.t5.data2", 1, 2, 5, UnitCode::Data, false, false),
        unit("i2.t6.node", 2, 0, 6, UnitCode::Metadata, false, false),
        unit("i2.t8.accounting", 2, 0, 8, UnitCode::Metadata, false, false),
        unit("i2.t9.data1", 2, 1, 9, UnitCode::Data, false, false),
        unit("i3.t10.data1", 3, 1, 10, UnitCode::Data, true, true),
        unit("i3.t10.data2", 3, 2, 10, UnitCode::Data, true, true),
        unit("i3.t10.fixed", 3, 0, 10, UnitCode::Metadata, false, false),
        unit("i4.t10.fixed", 4, 0, 10, UnitCode::Metadata, true, true),
        unit("i4.t12.accounting", 4, 0, 12, UnitCode::Metadata, true, true),
    ];
    let resent_rollback_table =
        apply_rows(&[], &[rollback_row(1, 3), batch_row(2, 0, 0)], Protection::RollbackRowOnly);
    let arms: Vec<(&str, Vec<InstanceRow>)> = vec![
        // The switch's selected root = the root the in-flight checkpoint was built on (R_old = (1,3)).
        // Batch [1, 4): R with no replay gives (1,3,0), which loses to the rollback row; (2,0,0) by
        // P330'' 1; owner row (3,0,2) by P330'' 3'.
        ("switch_selected_root_is_in_flight_base_r_old", vec![batch_row(1, 3, 0), batch_row(2, 0, 0), owner_row(3, 0, 2)]),
        // The switch's selected root = what a recovery selects, "newest readable, self-certified root"
        // (D18 line 879) = (2,9). Batch [2, 4): R gives (2, 9, 0) (no record after (2,9)); owner (3,0,2).
        // The rollback's intermediate (2,0,0) is not a rollback row, so it is overwritten.
        ("switch_selected_root_is_newest_readable", vec![batch_row(2, 9, 0), owner_row(3, 0, 2)]),
        // Same reading, and the switch writes only D23 line 691's single row for the old instance
        // (old instance 3, never published: (3, 0, 2) per P330'' 3'); no batch at all.
        ("switch_writes_only_old_instance_row", vec![owner_row(3, 0, 2)]),
    ];
    for (arm_name, switch_rows) in arms {
        let table = apply_rows(&resent_rollback_table, &switch_rows, Protection::RollbackRowOnly);
        let (wrong_published, wrong_unpublished) = report(arm_name, &units, &table, 4, 12);
        results.push((format!("X3a {arm_name}"), wrong_published, wrong_unpublished));
    }
    // Later: instance 4 lands (4,13) and ends; mount 5 recovers from (4,13), batch [4, 5) = {4}.
    // The rollback row and the intermediate row are outside every later batch (I-3.8: rows < 4).
    let mut units_after = units.clone();
    units_after.push(unit("i5.t14.accounting", 5, 0, 14, UnitCode::Metadata, true, true));
    for (arm_name, switch_rows) in [
        ("after_recovery5 in_flight_base", vec![batch_row(1, 3, 0), batch_row(2, 0, 0), owner_row(3, 0, 2)]),
        ("after_recovery5 newest_readable", vec![batch_row(2, 9, 0), owner_row(3, 0, 2)]),
    ] {
        let table_four = apply_rows(&resent_rollback_table, &switch_rows, Protection::RollbackRowOnly);
        let table_five = apply_rows(&table_four, &[batch_row(4, 13, 0)], Protection::RollbackRowOnly);
        let (wrong_published, wrong_unpublished) = report(arm_name, &units_after, &table_five, 5, 14);
        results.push((format!("X3a {arm_name}"), wrong_published, wrong_unpublished));
    }
    results
}

/// X3 (b) and (c). Recovery instance 2 selects (1,3). with_replay: (1,4) is replayed whole
/// (X_after = 4, W = 2; (1,5) landed only its first record and is dropped), so the row-writing
/// publication is the first publication after R takes effect; otherwise nothing is applied
/// (X_after = 3, W = 0). Instance 2's row-writing publication (txg X_after + 1) carries its
/// transactions 1, 2 and a node and lands its root on one disk; the next publication, the first
/// warm-up empty publication (txg X_after + 2), fails on an accounting unit write with a successful
/// probe: switch to 3, which resends the empty publication (no kept transaction) and warms up.
/// Instance 2's transaction 3 had completed in the open checkpoint (born X_after + 3) and is redone
/// by 3 as its transaction 1 (note 4). Faults: one crash in M1, one transient unit-write error.
fn scenario_switch_inside_warm_up_empty_publication(with_replay: bool) -> Vec<(String, usize, usize)> {
    let mut results = vec![];
    let x_after: u64 = if with_replay { 4 } else { 3 };
    let w_of_instance_one: u64 = if with_replay { 2 } else { 0 };
    let row_writing_txg = x_after + 1;
    let warm_up_txg = x_after + 2;
    let open_txg = x_after + 3;
    let mounted_txg = x_after + 4;
    let mut units = vec![
        unit("i1.t3.data1", 1, 1, 3, UnitCode::Data, true, true),
        unit("i1.t3.node", 1, 0, 3, UnitCode::Metadata, false, true),
    ];
    if with_replay {
        units.push(unit("i1.t4.node", 1, 0, 4, UnitCode::Metadata, true, true));
        units.push(unit("i1.t4.data2", 1, 2, 4, UnitCode::Data, true, true));
        units.push(unit("i1.t5.data3", 1, 3, 5, UnitCode::Data, false, false));
    } else {
        units.push(unit("i1.t4.data2", 1, 2, 4, UnitCode::Data, false, false));
    }
    units.push(unit(&format!("i2.t{row_writing_txg}.data1"), 2, 1, row_writing_txg, UnitCode::Data, true, true));
    units.push(unit(&format!("i2.t{row_writing_txg}.data2"), 2, 2, row_writing_txg, UnitCode::Data, true, true));
    units.push(unit(&format!("i2.t{row_writing_txg}.node"), 2, 0, row_writing_txg, UnitCode::Metadata, true, true));
    units.push(unit(&format!("i2.t{warm_up_txg}.accounting"), 2, 0, warm_up_txg, UnitCode::Metadata, false, false));
    units.push(unit(&format!("i2.t{open_txg}.data3"), 2, 3, open_txg, UnitCode::Data, false, false));
    units.push(unit(&format!("i3.t{warm_up_txg}.accounting"), 3, 0, warm_up_txg, UnitCode::Metadata, true, true));
    units.push(unit(&format!("i3.t{open_txg}.redo1"), 3, 1, open_txg, UnitCode::Data, true, true));
    units.push(unit(&format!("i3.t{mounted_txg}.accounting"), 3, 0, mounted_txg, UnitCode::Metadata, true, true));
    for (t_pub_label, t_pub_of_instance_one) in [("r_tpub_after_replay", x_after), ("today_tpub_selected", 3)] {
        if !with_replay && t_pub_label == "today_tpub_selected" {
            continue; // without replay the two T_pub rules coincide
        }
        // The table (2, row_writing_txg) references: instance 2's batch [1, 2) = {1}.
        let table_of_row_writing_root =
            apply_rows(&[], &[batch_row(1, t_pub_of_instance_one, w_of_instance_one)], Protection::RollbackRowOnly);
        let old_instance_row_d23 = batch_row(2, row_writing_txg, 0); // (old instance, last published txg, W = 0)
        let arms: Vec<(&str, Vec<InstanceRow>)> = vec![
            // Selected root of the switch = the old instance's last published root (2, row_writing_txg):
            // batch [2, 3) = {2} = D23 line 691's row; the resend is empty, W = 0.
            ("sel_is_last_published_root", vec![old_instance_row_d23]),
            // Selected root of the switch = the root this mount's recovery selected, (1,3): batch [1, 3);
            // instance 2 is strictly between 1 and 3 and owns no kept transaction => P330'' 1 gives
            // (2,0,0); D23 line 691 gives (2, row_writing_txg, 0). Order within the publication decides.
            ("sel_is_mount_recovery_root_p330_row_written_last",
                vec![old_instance_row_d23, batch_row(1, t_pub_of_instance_one, w_of_instance_one), batch_row(2, 0, 0)]),
            ("sel_is_mount_recovery_root_d23_row_written_last",
                vec![batch_row(1, t_pub_of_instance_one, w_of_instance_one), batch_row(2, 0, 0), old_instance_row_d23]),
            // Note 4's W over an empty resend is max of an empty set; an implementation that takes the
            // old instance's transaction counter (3) instead of 0.
            ("sel_is_last_published_root_w_from_transaction_counter", vec![batch_row(2, row_writing_txg, 3)]),
        ];
        for (arm_name, switch_rows) in arms {
            let table = apply_rows(&table_of_row_writing_root, &switch_rows, Protection::RollbackRowOnly);
            let label = format!("{t_pub_label} {arm_name}");
            let (wrong_published, wrong_unpublished) = report(&label, &units, &table, 3, mounted_txg);
            results.push((format!("X3b replay={with_replay} {label}"), wrong_published, wrong_unpublished));
        }
    }
    results
}

fn main() {
    let mut all_results: Vec<(String, usize, usize)> = vec![];
    println!("X2 owner row (2,0,3) of a switch whose only root becomes unreadable; later recoveries");
    all_results.extend(scenario_owner_row_after_switch_instance_abandoned());
    println!("X3a switch inside the rollback publication (one transient unit-write error)");
    all_results.extend(scenario_switch_inside_rollback_publication());
    println!("X3b switch inside a warm-up empty publication, no replay");
    all_results.extend(scenario_switch_inside_warm_up_empty_publication(false));
    println!("X3c switch inside a warm-up empty publication right after R's row-writing publication (replay)");
    all_results.extend(scenario_switch_inside_warm_up_empty_publication(true));
    println!("SUMMARY arm rows with a wrongly published unit / with a wrongly unpublished unit");
    for (label, wrong_published, wrong_unpublished) in &all_results {
        if *wrong_published > 0 || *wrong_unpublished > 0 {
            println!("  HIT {label} wrongly_published={wrong_published} wrongly_unpublished={wrong_unpublished}");
        }
    }
    let hit_rows = all_results.iter().filter(|(_, published, unpublished)| *published > 0 || *unpublished > 0).count();
    println!("DONE evaluated_arm_rows={} hit_rows={hit_rows}", all_results.len());
}
