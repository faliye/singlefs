//! C329 / C330 second round, attack leg (Opus).
//! Scripted mount histories against the four packages P329, P330, Q330, R.
//! Deterministic, std only: every scenario builds the durable write set by hand,
//! applies each arm's row-writing rule, and runs the D18 line 879 published predicate.
//! Truth per unit: referenced by the mounted root, and whether its publication is on
//! the mounted root's timeline. Violation kinds follow the round-two criteria:
//!   wrongly_published   = predicate says published, not referenced, not on the timeline (V2)
//!   wrongly_unpublished = predicate says unpublished, referenced by the mounted root (V3)

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnitCode {
    Data,     // code 1: judged by birth <= T_pub or transaction number <= W
    Metadata, // code 2 / 3: judged by birth <= T_pub only
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct InstanceRow {
    instance: u32,
    published_txg: u64,
    max_applied_transaction: u64,
    is_rollback: bool,
}

fn row(instance: u32, published_txg: u64, max_applied_transaction: u64) -> InstanceRow {
    InstanceRow { instance, published_txg, max_applied_transaction, is_rollback: false }
}

fn rollback_row(instance: u32, published_txg: u64) -> InstanceRow {
    InstanceRow { instance, published_txg, max_applied_transaction: 0, is_rollback: true }
}

/// Rows are unique per instance, later write overwrites (D18 line 879), except that a
/// rollback row is never overwritten by a later recovery.
fn apply_rows(base_table: &[InstanceRow], new_rows: &[InstanceRow]) -> Vec<InstanceRow> {
    let mut table: Vec<InstanceRow> = base_table.to_vec();
    for new_row in new_rows {
        match table.iter().position(|existing| existing.instance == new_row.instance) {
            Some(position) => {
                if table[position].is_rollback && !new_row.is_rollback {
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

/// D18 line 879, the global published predicate, verbatim in structure.
fn published_predicate(
    candidate: &Unit,
    table: &[InstanceRow],
    mounted_instance: u32,
    mounted_txg: u64,
) -> Verdict {
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
            let by_transaction = candidate.transaction_number != 0
                && candidate.transaction_number <= found_row.max_applied_transaction;
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
            let marker = if table_row.is_rollback { ",rollback" } else { "" };
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

/// S1: the row-writing publication (P329 1: the new instance's first publication) of a
/// recovery instance fails on a unit write after the new instance's own user transactions
/// completed; the probe write succeeds, so the mount switches instance (D23 item 14).
/// The kept transactions (number <= W) belong to the instance that never published a root.
fn scenario_switch_during_row_writing_publication(kept_transaction_count: u64, old_orphan_transaction: u64) -> Vec<(String, usize, usize)> {
    // M1, instance 1: (1,1) (1,2) warm-up, (1,3) carries transactions 1..old_orphan_transaction-1,
    // in-flight (1,4) carries transaction old_orphan_transaction: unit landed, no record, crash.
    let mut units = vec![
        unit("i1.t3.node", 1, 0, 3, UnitCode::Metadata, true, true),
        unit("i1.t4.accounting", 1, 0, 4, UnitCode::Metadata, false, false),
        unit(&format!("i1.t4.data{old_orphan_transaction}"), 1, old_orphan_transaction, 4, UnitCode::Data, false, false),
    ];
    for transaction in 1..old_orphan_transaction {
        units.push(unit(&format!("i1.t3.data{transaction}"), 1, transaction, 3, UnitCode::Data, true, true));
    }
    // M2, instance 2 (recovery): selected (1,3), nothing to replay (W = 0). Its first publication,
    // txg 4, carries its own transactions 1..kept_transaction_count (units landed) and a fixed-point
    // unit; the next fixed-point write fails transiently => switch to instance 3.
    for transaction in 1..=kept_transaction_count {
        units.push(unit(&format!("i2.t4.data{transaction}"), 2, transaction, 4, UnitCode::Data, true, true));
    }
    units.push(unit("i2.t4.fixed", 2, 0, 4, UnitCode::Metadata, false, false));
    // Instance 3 resends the in-flight checkpoint as its first publication (txg 4 under both the
    // today rule and Q330: no record of txg 4 was written), keeps instance 2's transactions
    // 1..kept_transaction_count (note 4: W = max transaction number in the resent checkpoint),
    // rewrites the fixed point, lands (3,4), warms up (3,5) (3,6).
    units.push(unit("i3.t4.fixed", 3, 0, 4, UnitCode::Metadata, true, true));
    units.push(unit("i3.t6.accounting", 3, 0, 6, UnitCode::Metadata, true, true));
    let kept_w = kept_transaction_count;
    let arms: Vec<(&str, Vec<InstanceRow>)> = vec![
        // D18 batch [1, 3) with the switch sentence (old instance, last published txg, W); instance 2
        // never published, so its txg falls back to the selected root's txg.
        ("today", vec![row(1, 3, 0), row(2, 3, kept_w)]),
        // P330 3 as worded: W on the owner of the kept transactions "(the selected root's instance)",
        // P330 1: intermediates strictly between the selected root's instance and the new one get (i,0,0).
        ("p330_as_worded", vec![row(1, 3, kept_w), row(2, 0, 0)]),
        // P330 1 wins and W is dropped (the owner is an intermediate, P330 1 forces (2,0,0)).
        ("p330_intermediate_first", vec![row(1, 3, 0), row(2, 0, 0)]),
        // Owner row fix: W goes to the instance that owns the kept transactions, T_pub 0.
        ("owner_row_fix", vec![row(1, 3, 0), row(2, 0, kept_w)]),
    ];
    let mut results = vec![];
    for (arm_name, new_rows) in arms {
        let table = apply_rows(&[], &new_rows);
        let (wrongly_published, wrongly_unpublished) = report(arm_name, &units, &table, 3, 6);
        results.push((arm_name.to_string(), wrongly_published, wrongly_unpublished));
    }
    results
}

/// S2: two switches in a row; the first switch instance (3) resends the in-flight checkpoint and
/// puts the redo of the open checkpoint's transaction into that resend (the reading P329 3 uses:
/// "the same publication's user-data redo"); its resend fails and it switches to instance 4.
/// keep_previous_switch_redo: whether instance 4 keeps instance 3's completed redo (it sits in the
/// resent checkpoint, so note 4's "number <= W, kept" can cover it) or redoes it again.
fn scenario_double_switch_with_redo_in_resend(keep_previous_switch_redo: bool) -> Vec<(String, usize, usize)> {
    let mut units = vec![
        unit("i1.t3.data1", 1, 1, 3, UnitCode::Data, true, true),
        unit("i2.t5.data1", 2, 1, 5, UnitCode::Data, true, true),
        // in-flight txg 6 of instance 2: transactions 2, 3 landed (kept by both switches)
        unit("i2.t6.data2", 2, 2, 6, UnitCode::Data, true, true),
        unit("i2.t6.data3", 2, 3, 6, UnitCode::Data, true, true),
        unit("i2.t6.fixed", 2, 0, 6, UnitCode::Metadata, false, false),
        // open txg 7 of instance 2: transaction 4 landed, redone by instance 3 as its transaction 1
        unit("i2.t7.data4", 2, 4, 7, UnitCode::Data, false, false),
        unit("i3.t6.redo1", 3, 1, 6, UnitCode::Data, keep_previous_switch_redo, keep_previous_switch_redo),
        unit("i3.t6.fixed", 3, 0, 6, UnitCode::Metadata, false, false),
        unit("i4.t6.fixed", 4, 0, 6, UnitCode::Metadata, true, true),
        unit("i4.t8.accounting", 4, 0, 8, UnitCode::Metadata, true, true),
    ];
    if !keep_previous_switch_redo {
        units.push(unit("i4.t6.redo1", 4, 1, 6, UnitCode::Data, true, true));
    }
    let base_table = vec![row(1, 3, 0)]; // (2,5)'s table: written by instance 2's row-writing publication
    let redo_w_of_instance_three = if keep_previous_switch_redo { 1 } else { 0 };
    let arms: Vec<(&str, Vec<InstanceRow>)> = vec![
        ("today", vec![row(2, 5, 0), row(3, 5, 3)]),
        ("p330_as_worded", vec![row(2, 5, 3), row(3, 0, 0)]),
        ("owner_row_fix", vec![row(2, 5, 3), row(3, 0, redo_w_of_instance_three)]),
    ];
    let mut results = vec![];
    for (arm_name, new_rows) in arms {
        let table = apply_rows(&base_table, &new_rows);
        let (wrongly_published, wrongly_unpublished) = report(arm_name, &units, &table, 4, 8);
        results.push((arm_name.to_string(), wrongly_published, wrongly_unpublished));
    }
    results
}

/// S3: R (T_pub of the selected root's instance = txg of the root after replay) and P330 2
/// (the rollback publication's instance table is based on R_old's table). Replay, then an admin
/// rollback onto the row-writing root, then a recovery after the rollback.
fn scenario_replay_then_rollback_then_recovery() -> Vec<(String, usize, usize)> {
    let mut results = vec![];
    // M1 instance 1: (1,3) root; (1,4) record landed, root not; (1,5) units landed, only the first
    // of its two records landed => the sixth prefix clause drops (1,5) whole.
    let base_units = vec![
        unit("i1.t3.node", 1, 0, 3, UnitCode::Metadata, true, true),
        unit("i1.t3.data1", 1, 1, 3, UnitCode::Data, true, true),
        unit("i1.t4.node", 1, 0, 4, UnitCode::Metadata, true, true),
        unit("i1.t4.data2", 1, 2, 4, UnitCode::Data, true, true),
        unit("i1.t5.node", 1, 0, 5, UnitCode::Metadata, false, false),
        unit("i1.t5.data3", 1, 3, 5, UnitCode::Data, false, false),
    ];
    // M2 instance 2 (recovery): selected (1,3), replays (1,4) whole: X_after = 4, W = 2.
    // First txg: today 5 (after-replay root + 1), Q330 6 (record of (1,5) self-certifies).
    // Instance 2 lands (2,6) (rows), warm-up (2,7) (2,8), then (2,9) with its transaction 1.
    for (arm_name, instance_one_row) in [("today_tpub_selected", row(1, 3, 2)), ("r_tpub_after_replay", row(1, 4, 2))] {
        let mut units = base_units.clone();
        units.push(unit("i2.t6.node", 2, 0, 6, UnitCode::Metadata, true, true));
        units.push(unit("i2.t9.data1", 2, 1, 9, UnitCode::Data, true, true));
        let table_of_instance_two = apply_rows(&[], &[instance_one_row]);
        let (published_wrong, unpublished_wrong) = report(&format!("M2 {arm_name}"), &units, &table_of_instance_two, 2, 9);
        results.push((format!("M2 {arm_name}"), published_wrong, unpublished_wrong));
        // M3 instance 3: admin rollback to R_old = (2,6). P330 2: base = (2,6)'s table, rollback row
        // (2,6,0,rollback); no intermediates. First new root txg = 10 (both rules). Instance 2's
        // transaction 1 (born 9) is on the abandoned part.
        for unit_entry in units.iter_mut() {
            if unit_entry.name == "i2.t9.data1" {
                unit_entry.referenced_by_mounted_root = false;
                unit_entry.on_mounted_timeline = false;
            }
        }
        units.push(unit("i3.t10.node", 3, 0, 10, UnitCode::Metadata, true, true));
        let rollback_table = apply_rows(&table_of_instance_two, &[rollback_row(2, 6)]);
        let (published_wrong, unpublished_wrong) = report(&format!("M3 rollback {arm_name}"), &units, &rollback_table, 3, 10);
        results.push((format!("M3 rollback {arm_name}"), published_wrong, unpublished_wrong));
        // M4 instance 4: recovery after the rollback root and its warm-up (3,11) (3,12) landed and a
        // later publication (3,13) with transaction 1 landed; in-flight (3,14) units only.
        units.push(unit("i3.t13.data1", 3, 1, 13, UnitCode::Data, true, true));
        units.push(unit("i3.t14.data2", 3, 2, 14, UnitCode::Data, false, false));
        units.push(unit("i4.t14.node", 4, 0, 14, UnitCode::Metadata, true, true));
        let recovery_table = apply_rows(&rollback_table, &[row(3, 13, 0)]);
        let (published_wrong, unpublished_wrong) = report(&format!("M4 recovery {arm_name}"), &units, &recovery_table, 4, 14);
        results.push((format!("M4 recovery {arm_name}"), published_wrong, unpublished_wrong));
    }
    results
}

/// S4: the Q330 watermark and the counter rule of D23 item 14 note 3 ("the new instance writes
/// from the prefix end + 1"). The new instance's first record lands on the record slot of the
/// temporarily unreadable root's publication; a crash tears that record write on both mirrors,
/// so the only record with txg 4 is gone and the watermark regresses at the next mount.
#[derive(Clone, Copy)]
struct RecordSlot {
    instance: u32,
    txg: u64,
    self_certified: bool,
}

fn q330_watermark(superblock_txg: Option<u64>, readable_root_txgs: &[u64], ring: &[RecordSlot]) -> u64 {
    let superblock_part = superblock_txg.unwrap_or(0);
    let root_part = readable_root_txgs.iter().copied().max().unwrap_or(0);
    let record_part = ring.iter().filter(|slot| slot.self_certified).map(|slot| slot.txg).max().unwrap_or(0);
    superblock_part.max(root_part).max(record_part)
}

fn scenario_q330_torn_overwrite(superblock_carries_txg: bool) -> Vec<(String, usize, usize)> {
    let mut results = vec![];
    // counters c1..c3: instance 1's records for txg 1..3; superblock txg 3 after (1,3)
    let mut ring = vec![
        RecordSlot { instance: 1, txg: 1, self_certified: true },
        RecordSlot { instance: 1, txg: 2, self_certified: true },
        RecordSlot { instance: 1, txg: 3, self_certified: true },
    ];
    let superblock_txg = if superblock_carries_txg { Some(3) } else { None };
    // M2 instance 2: (2,4) record at c4, root (2,4) FUA landed, crash before the superblock slot.
    ring.push(RecordSlot { instance: 2, txg: 4, self_certified: true });
    // M3 instance 3: (2,4) unreadable. Watermark sees the record of txg 4.
    let watermark_m3 = q330_watermark(superblock_txg, &[1, 2, 3], &ring);
    let first_txg_m3 = watermark_m3 + 1;
    // selected (1,3); replay stops at c4 (instance 2 != 1); prefix end c3; instance 3's first record
    // goes to c4, overwriting (2,4)'s record; the crash tears it on both mirrors.
    ring[3] = RecordSlot { instance: 3, txg: first_txg_m3, self_certified: false };
    // M4 instance 4: (2,4) still unreadable.
    let watermark_m4 = q330_watermark(superblock_txg, &[1, 2, 3], &ring);
    let first_txg_m4 = watermark_m4 + 1;
    println!(
        "  Q330 superblock_carries_txg={superblock_carries_txg} M3 watermark={watermark_m3} first_txg={first_txg_m3} | c4 torn | M4 watermark={watermark_m4} first_txg={first_txg_m4} (collides with unreadable (2,4): {})",
        ring[3].instance == 3 && first_txg_m4 == 4
    );
    // M4 lands only units, crash. M5: everything readable, selected (2,4) (highest txg, no root of 3 or 4).
    let units = vec![
        unit("i2.t4.node", 2, 0, 4, UnitCode::Metadata, true, true),
        unit("i2.t4.data1", 2, 1, 4, UnitCode::Data, true, true),
        unit(&format!("i3.t{first_txg_m3}.node"), 3, 0, first_txg_m3, UnitCode::Metadata, false, false),
        unit(&format!("i3.t{first_txg_m3}.data1"), 3, 1, first_txg_m3, UnitCode::Data, false, false),
        unit(&format!("i4.t{first_txg_m4}.node"), 4, 0, first_txg_m4, UnitCode::Metadata, false, false),
        unit(&format!("i4.t{first_txg_m4}.data1"), 4, 1, first_txg_m4, UnitCode::Data, false, false),
        unit("i5.t5.node", 5, 0, 5, UnitCode::Metadata, true, true),
    ];
    let base_table = vec![row(1, 3, 0)];
    for (arm_name, new_rows) in [
        ("q330_alone_today_rows", vec![row(2, 4, 0), row(3, 4, 0), row(4, 4, 0)]),
        ("q330_plus_p330", vec![row(2, 4, 0), row(3, 0, 0), row(4, 0, 0)]),
    ] {
        let table = apply_rows(&base_table, &new_rows);
        let (published_wrong, unpublished_wrong) = report(arm_name, &units, &table, 5, 5);
        results.push((format!("{arm_name} superblock_carries_txg={superblock_carries_txg}"), published_wrong, unpublished_wrong));
    }
    results
}

/// S4b: the same counter rule against the r1 inversion shape. Instance 2's four roots (2,4)..(2,7)
/// are all unreadable at M3; instance 3's first publication spans four records (a publication may
/// carry several records, D23 item 7), written together between the two barriers of D16 item 7 at
/// counters c4..c7; one crash tears all four on both mirrors.
fn scenario_q330_torn_multi_record_overwrite() {
    let mut ring = vec![
        RecordSlot { instance: 1, txg: 1, self_certified: true },
        RecordSlot { instance: 1, txg: 2, self_certified: true },
        RecordSlot { instance: 1, txg: 3, self_certified: true },
    ];
    for txg in 4..=7 {
        ring.push(RecordSlot { instance: 2, txg, self_certified: true });
    }
    let watermark_m3 = q330_watermark(None, &[1, 2, 3], &ring);
    for position in 3..7 {
        ring[position] = RecordSlot { instance: 3, txg: watermark_m3 + 1, self_certified: false };
    }
    let watermark_m4 = q330_watermark(None, &[1, 2, 3], &ring);
    let first_txg_m4 = watermark_m4 + 1;
    // M4 instance 4: selected (1,3); lands (4,4) (rows), warm-up (4,5) (4,6) cover both disks
    // (region = txg mod 3, ownership 0 / 1 / 0), fsync of a write in (4,6) returns; crash.
    // M5: (2,4)..(2,7) readable again; selection by (txg, instance) max.
    let instance_four_last_txg = first_txg_m4 + 2;
    let selected_is_instance_two = 7 > instance_four_last_txg;
    println!(
        "  Q330_INVERSION M3 watermark={watermark_m3} | c4..c7 torn | M4 watermark={watermark_m4} first_txg={first_txg_m4} acknowledged_root=(4,{instance_four_last_txg}) | M5 selects (2,7) over it: {selected_is_instance_two} (acknowledged write lost; rows do not enter selection, so P330 does not change it)"
    );
}

/// S5: row reclamation under P329 2 (rows on every writable mount). The deletion condition of D18
/// line 879 requires "no root of that instance in the root ring"; the sweep's read-failure clause
/// covers unit locations. A root slot that is temporarily unreadable holds (2,4).
fn scenario_row_reclamation_with_unreadable_root_slot() {
    let root_slot_of_instance_two_readable_at_deletion = false;
    let instance_two_units_all_erased = true; // (2,0,0) judges all of them unpublished; the sweep erased them
    let every_device_online = true;
    let sweep_saw_read_failure_on_unit_locations = false;
    for (arm_name, unreadable_root_slot_blocks_deletion) in [("literal_condition", false), ("unreadable_slot_blocks", true)] {
        let ring_has_root_of_instance_two = if root_slot_of_instance_two_readable_at_deletion {
            true
        } else {
            unreadable_root_slot_blocks_deletion
        };
        let row_deleted = instance_two_units_all_erased
            && every_device_online
            && !sweep_saw_read_failure_on_unit_locations
            && !ring_has_root_of_instance_two;
        // later the slot reads again: rollback candidate set of D23 item 14:
        // (i, T) selectable <=> no row for i, or row (i, Ti, Wi) and T <= Ti; F_effective = 0
        let candidate_selectable = if row_deleted { true } else { 4 <= 0 };
        println!(
            "  RECLAIM {arm_name}: row (2,0,0) deleted={row_deleted} | slot heals | (2,4) in rollback candidate set={candidate_selectable} | its instance-2 units were erased by the sweep={instance_two_units_all_erased}"
        );
    }
}

fn main() {
    println!("S1 switch during the row-writing publication (kept transactions owned by the new instance)");
    let base_results = scenario_switch_during_row_writing_publication(3, 2);
    println!("S1_SWEEP kept_transaction_count x old_orphan_transaction (1..=4 x 2..=5)");
    let mut sweep_cells = 0;
    let mut cells_with_wrongly_published = std::collections::BTreeMap::<String, usize>::new();
    let mut cells_with_wrongly_unpublished = std::collections::BTreeMap::<String, usize>::new();
    for kept_transaction_count in 1..=4u64 {
        for old_orphan_transaction in 2..=5u64 {
            sweep_cells += 1;
            for (arm_name, wrongly_published, wrongly_unpublished) in
                scenario_switch_during_row_writing_publication(kept_transaction_count, old_orphan_transaction)
            {
                *cells_with_wrongly_published.entry(arm_name.clone()).or_insert(0) += usize::from(wrongly_published > 0);
                *cells_with_wrongly_unpublished.entry(arm_name).or_insert(0) += usize::from(wrongly_unpublished > 0);
            }
        }
    }
    for (arm_name, count) in &cells_with_wrongly_published {
        println!(
            "S1_SWEEP_SUMMARY arm={arm_name} cells={sweep_cells} cells_with_wrongly_published={count} cells_with_wrongly_unpublished={}",
            cells_with_wrongly_unpublished[arm_name]
        );
    }
    println!("S2 double switch, redo inside the resend, instance 4 keeps instance 3's completed redo");
    let keep_results = scenario_double_switch_with_redo_in_resend(true);
    println!("S2 double switch, redo inside the resend, instance 4 redoes it again");
    let redo_results = scenario_double_switch_with_redo_in_resend(false);
    println!("S3 replay, rollback onto the row-writing root, recovery after the rollback");
    let replay_results = scenario_replay_then_rollback_then_recovery();
    println!("S4 Q330 with a torn overwrite of the unreadable root's record");
    let q330_results_without_superblock = scenario_q330_torn_overwrite(false);
    let q330_results_with_superblock = scenario_q330_torn_overwrite(true);
    scenario_q330_torn_multi_record_overwrite();
    println!("S5 row reclamation with a temporarily unreadable root slot");
    scenario_row_reclamation_with_unreadable_root_slot();
    let all_results = [base_results, keep_results, redo_results, replay_results, q330_results_without_superblock, q330_results_with_superblock];
    let evaluated_rows: usize = all_results.iter().map(|results| results.len()).sum();
    println!("DONE evaluated_arm_rows={evaluated_rows} sweep_cells={sweep_cells}");
}
