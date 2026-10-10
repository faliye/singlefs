
// ---------------------------------------------------------------- K1-1：管理员回退 / 公开上限函数

fn k11_setup(fault_on: bool) -> (PoolWithAnAbandonedRoot, PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>, ReadFaultOnChosenSlots) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault = ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (pool, _rec) = mounted_while_the_abandoned_root_reads(&built, &fault);
    fault.is_active.set(fault_on);
    (built, pool, fault)
}

#[test]
fn k11_admin_rollback_with_abandoned_slot_bad() {
    for fault_on in [false, true] {
        let (built, mut pool, _fault) = k11_setup(fault_on);
        let mut session = pool.session.take().expect("会话");
        let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
        let target = singlefs_core::mount::RollbackTarget {
            instance: built.newest_valid_root.instance,
            checkpoint_txg: built.newest_valid_root.checkpoint_txg,
        };
        let r = singlefs_core::mount::roll_back_by_a_forward_publish(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            target,
        );
        println!("RESULT k11-admin-rollback fault_on={fault_on}: {}", short(format!("{:?}", r.map(|_| ()))));
    }
}

#[test]
fn k11_public_ceiling_function_with_abandoned_slot_bad() {
    for fault_on in [false, true] {
        let (_built, mut pool, _fault) = k11_setup(fault_on);
        let mut session = pool.session.take().expect("会话");
        let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
        let configuration = singlefs_core::recovery::choose_system_configuration(&pool.devices).expect("系统配置");
        let table = singlefs_core::recovery::instance_table_chain_of_root(&pool.devices, &current.root)
            .expect("表")
            .records;
        let r = singlefs_core::mount::rollback_floor_ceiling(
            &pool.devices,
            &configuration,
            current.root.rollback_floor,
            &table,
            &singlefs_core::mount::ring_slots_known_to_hold_a_root_by(&session.allocator),
        );
        println!("RESULT k11-public-rollback_floor_ceiling(known slots, 与 checker 档 on_device_modes.rs 同一写法) fault_on={fault_on}: {}", short(format!("{r:?}")));
        let floor_now = current.root.rollback_floor;
        let r2 = singlefs_core::mount::raise_rollback_floor(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            r.as_ref().map(|c| *c).unwrap_or(floor_now),
            ShadowLedger::On,
        );
        println!("RESULT k11-raise_rollback_floor(到上面的上限) fault_on={fault_on}: {}", short(format!("{:?}", r2.map(|x| x.abandoned_roots_unreadable))));
    }
}

#[test]
fn print_keys_of_p_and_r() {
    for kind in [PublishRecordedAsR::Overwrite, PublishRecordedAsR::SequentialWriteOfThreeDataUnits] {
        let recorded = record_one_publish_after_overwrites(5, kind);
        println!("RESULT keys {kind:?}: P={:?} R={:?}", recorded.previous_key, recorded.key);
    }
}

#[test]
fn print_instance_table_after_m1_bz_and_az() {
    for extra in ["az", "bz"] {
        let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::Overwrite);
        let groups = recorded.unit_writes_grouped_by_unit();
        let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
        let mut ranges = RangesReadingBackAllZero::default().with_the_writes(&groups[3]);
        let r = recorded.root();
        for pointer in [r.tree_table, r.mapping_root, r.allocation_record_tree_root] {
            if pointer != NodePointer::empty_root() {
                ranges = ranges.with_the_writes(&recorded.unit_writes_at(&pointer));
            }
        }
        if extra == "bz" {
            for ordinal in 1..=recorded.record_count() {
                ranges = ranges.with_the_writes(&recorded.record_writes_of_the_record(ordinal));
            }
        }
        let mut devs = devices_reading_back_all_zero_on(&crash_image, &ranges);
        let mounted = mount_writable_with_space_admission(&recorded.pool.parameters, &mut devs, SpaceAdmission::JudgedByTheFormula).expect("M1");
        let current_root = *mounted.current.root();
        let table = singlefs_core::recovery::instance_table_chain_of_root(&devs, &current_root).expect("表").records;
        println!("RESULT table {extra}: effective={:?} current_root=({:?},{:?}) rows={:?}", (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg), current_root.instance, current_root.checkpoint_txg, table);
    }
}

#[test]
fn diag_bz_refusal() {
    let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
    let groups = recorded.unit_writes_grouped_by_unit();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut ranges = RangesReadingBackAllZero::default().with_the_writes(&groups[3]);
    let r = recorded.root();
    for pointer in [r.tree_table, r.mapping_root, r.allocation_record_tree_root] {
        if pointer != NodePointer::empty_root() {
            ranges = ranges.with_the_writes(&recorded.unit_writes_at(&pointer));
        }
    }
    for ordinal in 1..=recorded.record_count() {
        ranges = ranges.with_the_writes(&recorded.record_writes_of_the_record(ordinal));
    }
    let mut devs = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_space_admission(&recorded.pool.parameters, &mut devs, SpaceAdmission::JudgedByTheFormula).expect("M1");
    let image_after = image_of_devices_reading_back_all_zero(&devs, recorded.pool.device_bytes);
    let fault = ReadFaultOnChosenSlots::on_the_account_of(&r);
    let fdevs = devices_with_the_read_fault_on(&image_after, &fault);
    let mut pool = pool_under_test_after_the_mount(&recorded.pool.parameters, recorded.pool.device_bytes, fdevs, mounted, recorded.pool.write_time_seconds + 10_000);
    fault.is_active.set(true);
    for i in 0..30 {
        let before_root = *pool.session().current.root();
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(_) => {}
            Err(e) => {
                let image = pool.image();
                let in_ring = root_is_in_the_ring(&image, recorded.key);
                let mut devices = plain_devices_on(&image);
                let table = singlefs_core::recovery::instance_table_chain_of_root(&devices, &before_root).map(|t| t.records);
                println!("RESULT diag step {} refused; R_in_ring={in_ring} current_root=({:?},{:?}) table={:?}", i + 1, before_root.instance, before_root.checkpoint_txg, table);
                let cfg = singlefs_core::recovery::choose_system_configuration(&devices).expect("cfg");
                let roots = singlefs_core::recovery::readable_roots(&devices, &cfg.immutable.region_devices, &cfg.immutable.sizes, &cfg.immutable.filesystem_identifier);
                println!("RESULT diag roots_in_ring={:?}", roots.iter().map(|r| (r.instance.0, r.checkpoint_txg.0)).collect::<Vec<_>>());
                let _ = &mut devices;
                let session = pool.session.as_ref().expect("s");
                let table2 = singlefs_core::recovery::instance_table_chain_of_root(&pool.devices, session.current.root()).map(|t| t.records);
                println!("RESULT diag session.current.root=({:?},{:?}) floor={:?} table_from_session_current={:?}", session.current.root().instance, session.current.root().checkpoint_txg, session.current.root().rollback_floor, table2);
                let t2 = table2.expect("t");
                let cfg2 = singlefs_core::recovery::choose_system_configuration(&pool.devices).expect("cfg");
                let c = singlefs_core::mount::rollback_floor_ceiling(&pool.devices, &cfg2, session.current.root().rollback_floor, &t2, &singlefs_core::mount::ring_slots_known_to_hold_a_root_by(&session.allocator));
                println!("RESULT diag ceiling_with_session_table={}", short(format!("{c:?}")));
                let rr = singlefs_core::recovery::readable_roots(&pool.devices, &cfg2.immutable.region_devices, &cfg2.immutable.sizes, &cfg2.immutable.filesystem_identifier);
                let valid: Vec<(u32, u64)> = rr.iter().filter(|x| x.checkpoint_txg >= session.current.root().rollback_floor).filter(|x| !singlefs_core::recovery::root_is_abandoned_by_the_instance_table(x, &t2)).map(|x| (x.instance.0, x.checkpoint_txg.0)).collect();
                println!("RESULT diag valid_by_session_table={valid:?}");
                break;
            }
        }
    }
}
