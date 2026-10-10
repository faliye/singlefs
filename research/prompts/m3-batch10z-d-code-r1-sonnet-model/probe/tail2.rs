
// ---------------------------------------------------------------- H2 扫描

use singlefs_harness::memory_pool::RetainedWrite;

#[derive(Clone, Copy, Debug)]
enum H2Fault {
    Nothing,
    Account,
    RootSlot,
}

fn cells_of_writes(writes: &[&RetainedWrite]) -> Vec<PlacementOnDevice> {
    writes
        .iter()
        .map(|w| PlacementOnDevice {
            device: w.device,
            slot: SlotNumber(w.offset.0 / singlefs_format::SLOT_BYTES),
            span: w.length_in_bytes().div_ceil(singlefs_format::SLOT_BYTES),
        })
        .collect()
}

fn covered_by(p: &PlacementOnDevice, set: &[PlacementOnDevice]) -> bool {
    set.iter().any(|q| q.device == p.device && p.slot.0 >= q.slot.0 && p.slot.0 < q.slot.0 + q.span)
}

fn h2_run(
    label: &str,
    kind: PublishRecordedAsR,
    overwrites_before: u64,
    zero_group: usize,
    fault_kind: H2Fault,
    max_steps: usize,
    fill: bool,
    extra: &str,
) {
    let recorded = record_one_publish_after_overwrites(overwrites_before, kind);
    let groups = recorded.unit_writes_grouped_by_unit();
    if zero_group >= groups.len() {
        println!("RESULT {label}: 组{zero_group} 超出(共{}组)", groups.len());
        return;
    }
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut ranges = RangesReadingBackAllZero::default().with_the_writes(&groups[zero_group]);
    if extra == "az" || extra == "bz" {
        let r = recorded.root();
        for pointer in [r.tree_table, r.mapping_root, r.allocation_record_tree_root] {
            if pointer != NodePointer::empty_root() {
                ranges = ranges.with_the_writes(&recorded.unit_writes_at(&pointer));
            }
        }
    }
    if extra == "rz" || extra == "bz" {
        for ordinal in 1..=recorded.record_count() {
            ranges = ranges.with_the_writes(&recorded.record_writes_of_the_record(ordinal));
        }
    }
    let r_cells = cells_of_writes(&recorded.unit_writes());
    let mut mount_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = match mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut mount_devices,
        SpaceAdmission::JudgedByTheFormula,
    ) {
        Ok(m) => m,
        Err(e) => {
            println!("RESULT {label}: M1 被拒 {}", short(format!("{e:?}")));
            return;
        }
    };
    let selected = (
        mounted.output.effective_root.instance,
        mounted.output.effective_root.checkpoint_txg,
    );
    let selected_is_p = selected == recorded.previous_key;
    let occ = mounted.allocator.root_ring_occupancy().expect("根环表");
    let rec_abandoned = occ
        .placements_recorded_for_the_abandoned_root(root_of_r.instance, root_of_r.checkpoint_txg)
        .map(|v| v.len());
    let c_valid = occ
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        );
    let c_len = c_valid.as_ref().map(|v| v.len());
    let c_set = c_valid.clone().unwrap_or_default();
    let not_in_c: usize = r_cells.iter().filter(|p| !covered_by(p, &c_set)).count();
    let free_after_mount = free_among(&mounted.allocator, &r_cells);
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&mount_devices, recorded.pool.device_bytes);
    let fault = match fault_kind {
        H2Fault::Nothing | H2Fault::Account => ReadFaultOnChosenSlots::on_the_account_of(&root_of_r),
        H2Fault::RootSlot => {
            ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&root_of_r, &recorded.pool.parameters)
        }
    };
    let devices = devices_with_the_read_fault_on(&image_after_the_mount, &fault);
    let mut pool = pool_under_test_after_the_mount(
        &recorded.pool.parameters,
        recorded.pool.device_bytes,
        devices,
        mounted,
        recorded.pool.write_time_seconds + 10_000,
    );
    fault.is_active.set(!matches!(fault_kind, H2Fault::Nothing));
    let mut refused: Option<(usize, String)> = None;
    let mut raises = 0usize;
    let mut first_violation: Option<(usize, Vec<String>)> = None;
    let mut hazard: Vec<(usize, usize, usize)> = Vec::new(); // (步, R 在根环里时空闲数, 被覆盖数)
    let mut steps_done = 0;
    for index in 0..max_steps {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(p) => {
                steps_done = index + 1;
                raises += p.floor_raises.len();
                let fr = free_among(&pool.session().allocator, &r_cells);
                if index % 3 == 2 || fr > 0 {
                    let image = pool.image();
                    let in_ring = root_is_in_the_ring(&image, recorded.key);
                    if in_ring {
                        let over = units_of_r_overwritten_on(&recorded, &image).len();
                        if fr > 0 || over > 0 {
                            hazard.push((index + 1, fr, over));
                        }
                    }
                    if first_violation.is_none() && index % 3 == 2 {
                        let v = i74_i48(&image);
                        if !v.is_empty() {
                            first_violation = Some((index + 1, v));
                        }
                    }
                }
            }
            Err(e) => {
                refused = Some((index + 1, short(format!("{e:?}")).chars().take(150).collect::<String>()));
                break;
            }
        }
    }
    let mut fill_note = String::from("未做");
    if fill && refused.is_none() {
        let payload = singlefs_core::unit::data_unit_payload_capacity();
        let mut n = 2usize;
        let mut last_ok = 0usize;
        loop {
            match pool.sequential_write(n * payload) {
                Ok(_) => { last_ok = n; n += 2; }
                Err(e) => { fill_note = format!("顺序写{last_ok}个数据单元后第{n}个被拒:{}", short(format!("{e:?}")).chars().take(60).collect::<String>()); break; }
            }
            if n > 400 { fill_note = format!("到{last_ok}仍未拒"); break; }
        }
    }
    let image_w = pool.image();
    let in_ring_w = root_is_in_the_ring(&image_w, recorded.key);
    let over_w = units_of_r_overwritten_on(&recorded, &image_w).len();
    let v_w = i74_i48(&image_w);
    let (un, alloc_after) = unmount_result(&mut pool);
    let image_u = pool.image();
    let in_ring_u = root_is_in_the_ring(&image_u, recorded.key);
    let over_u = units_of_r_overwritten_on(&recorded, &image_u).len();
    let v_u = i74_i48(&image_u);
    let free_u = alloc_after.map(|a| free_among(&a, &r_cells));
    println!(
        "RESULT {label}[组{zero_group}/{}组,R单元{}个,不在C里{not_in_c}]: 择P={selected_is_p} 表记被抛弃落点={rec_abandoned:?} C借来={c_len:?} 挂后R单元空闲={free_after_mount} | 会话写{steps_done}/{max_steps} 被拒={refused:?} 推抬F串={raises} R在环里时空闲/被覆盖的步={hazard:?} 逐步首违例={first_violation:?} 写满={fill_note} R在环里(写后)={in_ring_w} 被覆盖(写后)={over_w} 违例(写后)={v_w:?} | 卸载={un} R在环里={in_ring_u} 空闲(卸载后)={free_u:?} 被覆盖={over_u} 违例(卸载后)={v_u:?}",
        groups.len(),
        r_cells.len()
    );
}

macro_rules! h2_tests {
    ($($name:ident: $kind:expr, $fault:expr, $ob:expr, $extra:expr;)*) => {$(
        mod $name {
            use super::*;
            pub fn go(g: usize) { h2_run(stringify!($name), $kind, $ob, g, $fault, 45, false, $extra); }
        }
    )*};
}
h2_tests! {
    ow_none: PublishRecordedAsR::Overwrite, H2Fault::Nothing, 5, "";
    ow_acct: PublishRecordedAsR::Overwrite, H2Fault::Account, 5, "";
    ow_root: PublishRecordedAsR::Overwrite, H2Fault::RootSlot, 5, "";
    sq_none: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Nothing, 5, "";
    sq_acct: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Account, 5, "";
    sq_root: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::RootSlot, 5, "";
    ow_az: PublishRecordedAsR::Overwrite, H2Fault::Account, 5, "az";
    sq_az: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Account, 5, "az";
    ow_rz: PublishRecordedAsR::Overwrite, H2Fault::Account, 5, "rz";
    sq_rz: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Account, 5, "rz";
    ow_bz: PublishRecordedAsR::Overwrite, H2Fault::Account, 5, "bz";
    sq_bz: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Account, 5, "bz";
}
#[test]
fn h2_ow_none_g00() { ow_none::go(0); }
#[test]
fn h2_ow_none_g01() { ow_none::go(1); }
#[test]
fn h2_ow_none_g02() { ow_none::go(2); }
#[test]
fn h2_ow_none_g03() { ow_none::go(3); }
#[test]
fn h2_ow_none_g04() { ow_none::go(4); }
#[test]
fn h2_ow_none_g05() { ow_none::go(5); }
#[test]
fn h2_ow_none_g06() { ow_none::go(6); }
#[test]
fn h2_ow_none_g07() { ow_none::go(7); }
#[test]
fn h2_ow_none_g08() { ow_none::go(8); }
#[test]
fn h2_ow_none_g09() { ow_none::go(9); }
#[test]
fn h2_ow_none_g10() { ow_none::go(10); }
#[test]
fn h2_ow_none_g11() { ow_none::go(11); }
#[test]
fn h2_ow_none_g12() { ow_none::go(12); }
#[test]
fn h2_ow_none_g13() { ow_none::go(13); }
#[test]
fn h2_ow_acct_g00() { ow_acct::go(0); }
#[test]
fn h2_ow_acct_g01() { ow_acct::go(1); }
#[test]
fn h2_ow_acct_g02() { ow_acct::go(2); }
#[test]
fn h2_ow_acct_g03() { ow_acct::go(3); }
#[test]
fn h2_ow_acct_g04() { ow_acct::go(4); }
#[test]
fn h2_ow_acct_g05() { ow_acct::go(5); }
#[test]
fn h2_ow_acct_g06() { ow_acct::go(6); }
#[test]
fn h2_ow_acct_g07() { ow_acct::go(7); }
#[test]
fn h2_ow_acct_g08() { ow_acct::go(8); }
#[test]
fn h2_ow_acct_g09() { ow_acct::go(9); }
#[test]
fn h2_ow_acct_g10() { ow_acct::go(10); }
#[test]
fn h2_ow_acct_g11() { ow_acct::go(11); }
#[test]
fn h2_ow_acct_g12() { ow_acct::go(12); }
#[test]
fn h2_ow_acct_g13() { ow_acct::go(13); }
#[test]
fn h2_ow_root_g00() { ow_root::go(0); }
#[test]
fn h2_ow_root_g01() { ow_root::go(1); }
#[test]
fn h2_ow_root_g02() { ow_root::go(2); }
#[test]
fn h2_ow_root_g03() { ow_root::go(3); }
#[test]
fn h2_ow_root_g04() { ow_root::go(4); }
#[test]
fn h2_ow_root_g05() { ow_root::go(5); }
#[test]
fn h2_ow_root_g06() { ow_root::go(6); }
#[test]
fn h2_ow_root_g07() { ow_root::go(7); }
#[test]
fn h2_ow_root_g08() { ow_root::go(8); }
#[test]
fn h2_ow_root_g09() { ow_root::go(9); }
#[test]
fn h2_ow_root_g10() { ow_root::go(10); }
#[test]
fn h2_ow_root_g11() { ow_root::go(11); }
#[test]
fn h2_ow_root_g12() { ow_root::go(12); }
#[test]
fn h2_ow_root_g13() { ow_root::go(13); }
#[test]
fn h2_sq_none_g00() { sq_none::go(0); }
#[test]
fn h2_sq_none_g01() { sq_none::go(1); }
#[test]
fn h2_sq_none_g02() { sq_none::go(2); }
#[test]
fn h2_sq_none_g03() { sq_none::go(3); }
#[test]
fn h2_sq_none_g04() { sq_none::go(4); }
#[test]
fn h2_sq_none_g05() { sq_none::go(5); }
#[test]
fn h2_sq_none_g06() { sq_none::go(6); }
#[test]
fn h2_sq_none_g07() { sq_none::go(7); }
#[test]
fn h2_sq_none_g08() { sq_none::go(8); }
#[test]
fn h2_sq_none_g09() { sq_none::go(9); }
#[test]
fn h2_sq_none_g10() { sq_none::go(10); }
#[test]
fn h2_sq_none_g11() { sq_none::go(11); }
#[test]
fn h2_sq_none_g12() { sq_none::go(12); }
#[test]
fn h2_sq_none_g13() { sq_none::go(13); }
#[test]
fn h2_sq_acct_g00() { sq_acct::go(0); }
#[test]
fn h2_sq_acct_g01() { sq_acct::go(1); }
#[test]
fn h2_sq_acct_g02() { sq_acct::go(2); }
#[test]
fn h2_sq_acct_g03() { sq_acct::go(3); }
#[test]
fn h2_sq_acct_g04() { sq_acct::go(4); }
#[test]
fn h2_sq_acct_g05() { sq_acct::go(5); }
#[test]
fn h2_sq_acct_g06() { sq_acct::go(6); }
#[test]
fn h2_sq_acct_g07() { sq_acct::go(7); }
#[test]
fn h2_sq_acct_g08() { sq_acct::go(8); }
#[test]
fn h2_sq_acct_g09() { sq_acct::go(9); }
#[test]
fn h2_sq_acct_g10() { sq_acct::go(10); }
#[test]
fn h2_sq_acct_g11() { sq_acct::go(11); }
#[test]
fn h2_sq_acct_g12() { sq_acct::go(12); }
#[test]
fn h2_sq_acct_g13() { sq_acct::go(13); }
#[test]
fn h2_sq_root_g00() { sq_root::go(0); }
#[test]
fn h2_sq_root_g01() { sq_root::go(1); }
#[test]
fn h2_sq_root_g02() { sq_root::go(2); }
#[test]
fn h2_sq_root_g03() { sq_root::go(3); }
#[test]
fn h2_sq_root_g04() { sq_root::go(4); }
#[test]
fn h2_sq_root_g05() { sq_root::go(5); }
#[test]
fn h2_sq_root_g06() { sq_root::go(6); }
#[test]
fn h2_sq_root_g07() { sq_root::go(7); }
#[test]
fn h2_sq_root_g08() { sq_root::go(8); }
#[test]
fn h2_sq_root_g09() { sq_root::go(9); }
#[test]
fn h2_sq_root_g10() { sq_root::go(10); }
#[test]
fn h2_sq_root_g11() { sq_root::go(11); }
#[test]
fn h2_sq_root_g12() { sq_root::go(12); }
#[test]
fn h2_sq_root_g13() { sq_root::go(13); }
#[test]
fn h2_ow_az_g00() { ow_az::go(0); }
#[test]
fn h2_ow_az_g01() { ow_az::go(1); }
#[test]
fn h2_ow_az_g02() { ow_az::go(2); }
#[test]
fn h2_ow_az_g03() { ow_az::go(3); }
#[test]
fn h2_ow_az_g04() { ow_az::go(4); }
#[test]
fn h2_ow_az_g05() { ow_az::go(5); }
#[test]
fn h2_ow_az_g06() { ow_az::go(6); }
#[test]
fn h2_ow_az_g07() { ow_az::go(7); }
#[test]
fn h2_ow_az_g08() { ow_az::go(8); }
#[test]
fn h2_ow_az_g09() { ow_az::go(9); }
#[test]
fn h2_ow_az_g10() { ow_az::go(10); }
#[test]
fn h2_ow_az_g11() { ow_az::go(11); }
#[test]
fn h2_ow_az_g12() { ow_az::go(12); }
#[test]
fn h2_ow_az_g13() { ow_az::go(13); }
#[test]
fn h2_sq_az_g00() { sq_az::go(0); }
#[test]
fn h2_sq_az_g01() { sq_az::go(1); }
#[test]
fn h2_sq_az_g02() { sq_az::go(2); }
#[test]
fn h2_sq_az_g03() { sq_az::go(3); }
#[test]
fn h2_sq_az_g04() { sq_az::go(4); }
#[test]
fn h2_sq_az_g05() { sq_az::go(5); }
#[test]
fn h2_sq_az_g06() { sq_az::go(6); }
#[test]
fn h2_sq_az_g07() { sq_az::go(7); }
#[test]
fn h2_sq_az_g08() { sq_az::go(8); }
#[test]
fn h2_sq_az_g09() { sq_az::go(9); }
#[test]
fn h2_sq_az_g10() { sq_az::go(10); }
#[test]
fn h2_sq_az_g11() { sq_az::go(11); }
#[test]
fn h2_sq_az_g12() { sq_az::go(12); }
#[test]
fn h2_sq_az_g13() { sq_az::go(13); }
#[test]
fn h2_ow_rz_g00() { ow_rz::go(0); }
#[test]
fn h2_ow_rz_g01() { ow_rz::go(1); }
#[test]
fn h2_ow_rz_g02() { ow_rz::go(2); }
#[test]
fn h2_ow_rz_g03() { ow_rz::go(3); }
#[test]
fn h2_ow_rz_g04() { ow_rz::go(4); }
#[test]
fn h2_ow_rz_g05() { ow_rz::go(5); }
#[test]
fn h2_ow_rz_g06() { ow_rz::go(6); }
#[test]
fn h2_ow_rz_g07() { ow_rz::go(7); }
#[test]
fn h2_ow_rz_g08() { ow_rz::go(8); }
#[test]
fn h2_ow_rz_g09() { ow_rz::go(9); }
#[test]
fn h2_ow_rz_g10() { ow_rz::go(10); }
#[test]
fn h2_ow_rz_g11() { ow_rz::go(11); }
#[test]
fn h2_ow_rz_g12() { ow_rz::go(12); }
#[test]
fn h2_ow_rz_g13() { ow_rz::go(13); }
#[test]
fn h2_sq_rz_g00() { sq_rz::go(0); }
#[test]
fn h2_sq_rz_g01() { sq_rz::go(1); }
#[test]
fn h2_sq_rz_g02() { sq_rz::go(2); }
#[test]
fn h2_sq_rz_g03() { sq_rz::go(3); }
#[test]
fn h2_sq_rz_g04() { sq_rz::go(4); }
#[test]
fn h2_sq_rz_g05() { sq_rz::go(5); }
#[test]
fn h2_sq_rz_g06() { sq_rz::go(6); }
#[test]
fn h2_sq_rz_g07() { sq_rz::go(7); }
#[test]
fn h2_sq_rz_g08() { sq_rz::go(8); }
#[test]
fn h2_sq_rz_g09() { sq_rz::go(9); }
#[test]
fn h2_sq_rz_g10() { sq_rz::go(10); }
#[test]
fn h2_sq_rz_g11() { sq_rz::go(11); }
#[test]
fn h2_sq_rz_g12() { sq_rz::go(12); }
#[test]
fn h2_sq_rz_g13() { sq_rz::go(13); }
#[test]
fn h2_ow_bz_g00() { ow_bz::go(0); }
#[test]
fn h2_ow_bz_g01() { ow_bz::go(1); }
#[test]
fn h2_ow_bz_g02() { ow_bz::go(2); }
#[test]
fn h2_ow_bz_g03() { ow_bz::go(3); }
#[test]
fn h2_ow_bz_g04() { ow_bz::go(4); }
#[test]
fn h2_ow_bz_g05() { ow_bz::go(5); }
#[test]
fn h2_ow_bz_g06() { ow_bz::go(6); }
#[test]
fn h2_ow_bz_g07() { ow_bz::go(7); }
#[test]
fn h2_ow_bz_g08() { ow_bz::go(8); }
#[test]
fn h2_ow_bz_g09() { ow_bz::go(9); }
#[test]
fn h2_ow_bz_g10() { ow_bz::go(10); }
#[test]
fn h2_ow_bz_g11() { ow_bz::go(11); }
#[test]
fn h2_ow_bz_g12() { ow_bz::go(12); }
#[test]
fn h2_ow_bz_g13() { ow_bz::go(13); }
#[test]
fn h2_sq_bz_g00() { sq_bz::go(0); }
#[test]
fn h2_sq_bz_g01() { sq_bz::go(1); }
#[test]
fn h2_sq_bz_g02() { sq_bz::go(2); }
#[test]
fn h2_sq_bz_g03() { sq_bz::go(3); }
#[test]
fn h2_sq_bz_g04() { sq_bz::go(4); }
#[test]
fn h2_sq_bz_g05() { sq_bz::go(5); }
#[test]
fn h2_sq_bz_g06() { sq_bz::go(6); }
#[test]
fn h2_sq_bz_g07() { sq_bz::go(7); }
#[test]
fn h2_sq_bz_g08() { sq_bz::go(8); }
#[test]
fn h2_sq_bz_g09() { sq_bz::go(9); }
#[test]
fn h2_sq_bz_g10() { sq_bz::go(10); }
#[test]
fn h2_sq_bz_g11() { sq_bz::go(11); }
#[test]
fn h2_sq_bz_g12() { sq_bz::go(12); }
#[test]
fn h2_sq_bz_g13() { sq_bz::go(13); }
