
// ---------------------------------------------------------------- 探测代码

fn free_among(allocator: &PoolAllocator, placements: &[PlacementOnDevice]) -> usize {
    placements
        .iter()
        .filter(|p| {
            let map = allocator
                .devices
                .iter()
                .find(|d| d.device == p.device)
                .expect("池里有这块盘");
            (0..p.span).any(|k| map.is_free(SlotNumber(p.slot.0 + k)))
        })
        .count()
}

fn merged(a: &ReadFaultOnChosenSlots, b: &ReadFaultOnChosenSlots) -> ReadFaultOnChosenSlots {
    let mut ranges: Vec<(DeviceIdentity, u64, u64)> = (*a.ranges).clone();
    ranges.extend((*b.ranges).iter().copied());
    ReadFaultOnChosenSlots {
        ranges: Rc::new(ranges),
        is_active: Rc::new(Cell::new(false)),
        zero_mode: Rc::new(Cell::new(false)),
    }
}

fn i74_i48(image: &MemoryPool) -> Vec<String> {
    violations_of_unit_reuse_on(image)
}

fn short(s: String) -> String {
    s.chars().take(300).collect()
}

fn unmount_result(
    pool: &mut PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
) -> (String, Option<PoolAllocator>) {
    let mut session = pool.session.take().expect("会话");
    let r = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match r {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => (
            format!("做成(计数{})", raised.abandoned_roots_unreadable),
            Some(session.allocator),
        ),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => {
            ("无文件版本未写".to_string(), Some(session.allocator))
        }
        Err(e) => (short(format!("拒 {e:?}")), None),
    }
}

/// K2-1 / K2-3：被抛弃根 C 的 H1 / 全零 / 错误 各形。
/// kind: ring（根槽故障）/ acct（账故障）/ both
fn h1_run(
    label: &str,
    ring_slot_fault: bool,
    account_fault: bool,
    zero: bool,
    fault_at_mount: bool,
    lift_after_mount: bool,
    max_steps: usize,
    fill: bool,
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    // 参照：无故障挂载记着的 C 的落点
    let reference: Vec<PlacementOnDevice> = {
        let f = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
        let (_p, rec) = mounted_while_the_abandoned_root_reads(&built, &f);
        rec
    };
    let ring = ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let acct = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
    let fault = match (ring_slot_fault, account_fault) {
        (true, true) => merged(&ring, &acct),
        (true, false) => ring,
        (false, true) => acct,
        (false, false) => merged(&ring, &acct),
    };
    fault.zero_mode.set(zero);
    fault.is_active.set(fault_at_mount);
    let mut devices = devices_with_the_read_fault_on(&built.image, &fault);
    let mounted = match mount_writable_with_space_admission(
        &built.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    ) {
        Ok(m) => m,
        Err(e) => {
            println!("RESULT {label}: 可写挂载被拒: {}", short(format!("{e:?}")));
            return;
        }
    };
    let occ = mounted.allocator.root_ring_occupancy().expect("根环表");
    let recorded_len = occ
        .placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg)
        .map(|v| v.len());
    let free_after_mount = free_among(&mounted.allocator, &reference);
    let mut pool = pool_under_test_after_the_mount(
        &built.parameters,
        built.device_bytes,
        devices,
        mounted,
        built.write_time_seconds + 1000,
    );
    let any_fault = ring_slot_fault || account_fault;
    fault.is_active.set(any_fault && !lift_after_mount);
    let mut lifted_at: Option<usize> = None;
    let mut refused: Option<(usize, String)> = None;
    let mut raises = 0usize;
    let mut first_violation: Option<(usize, Vec<String>)> = None;
    let mut first_free: Option<(usize, usize)> = None;
    let mut steps_done = 0usize;
    let mut hazard_steps: Vec<(usize, usize)> = Vec::new();
    for index in 0..max_steps {
        if lifted_at.is_none() && fault.is_active.get() {
            let still = pool
                .session()
                .allocator
                .root_ring_occupancy()
                .map(|o| o.placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg).is_some())
                .unwrap_or(false);
            if !still {
                fault.is_active.set(false);
                lifted_at = Some(index);
            }
        }
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(p) => {
                steps_done = index + 1;
                raises += p.floor_raises.len();
                let fr = free_among(&pool.session().allocator, &reference);
                let still_now = pool
                    .session()
                    .allocator
                    .root_ring_occupancy()
                    .map(|o| o.placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg).is_some())
                    .unwrap_or(false);
                if still_now && fr > 0 {
                    hazard_steps.push((index + 1, fr));
                }
                if first_free.is_none() && fr > 0 && !p.floor_raises.is_empty() {
                    first_free = Some((index + 1, fr));
                }
                if first_violation.is_none() && index % 3 == 2 {
                    let v = i74_i48(&pool.image());
                    if !v.is_empty() {
                        first_violation = Some((index + 1, v));
                    }
                }
            }
            Err(e) => {
                refused = Some((index + 1, short(format!("{e:?}"))));
                break;
            }
        }
    }
    let mut fill_note = String::from("未做");
    if fill {
        let payload = singlefs_core::unit::data_unit_payload_capacity();
        let mut n = 2usize;
        let mut last_ok = 0usize;
        loop {
            match pool.sequential_write(n * payload) {
                Ok(_) => { last_ok = n; n += 2; }
                Err(e) => { fill_note = format!("顺序写{}个数据单元后第{}个被拒:{}", last_ok, n, short(format!("{e:?}"))); break; }
            }
            if n > 400 { fill_note = format!("到{}个数据单元仍未拒", last_ok); break; }
        }
    }
    let table_still = pool
        .session
        .as_ref()
        .and_then(|s| s.allocator.root_ring_occupancy())
        .map(|o| o.placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg).is_some());
    let v_writes = i74_i48(&pool.image());
    let free_after_writes = free_among(&pool.session().allocator, &reference);
    let (un, alloc_after) = unmount_result(&mut pool);
    let v_un = i74_i48(&pool.image());
    let free_after_un = alloc_after.map(|a| free_among(&a, &reference));
    println!(
        "RESULT {label}: 参照落点数={} 挂载记着={:?} 挂后空闲={free_after_mount} | 会话写至多{max_steps}:完成{steps_done}次 被拒={:?} 推抬F串={raises} 表里还记着C时空闲>0的步={hazard_steps:?} 写满={fill_note} 故障在C被覆写后撤掉于第{lifted_at:?}步 表里还记着C={:?} 首次推抬F后空闲={:?} 逐步首个违例={:?} 空闲(写后)={free_after_writes} 违例(写后)={:?} | 卸载={un} 空闲(卸载后)={:?} 违例(卸载后)={:?}",
        reference.len(), recorded_len, refused, table_still, first_free, first_violation, v_writes, free_after_un, v_un
    );
}

#[test]
fn h1_ring_error_after_mount() { h1_run("H1 根槽读错(挂后)", true, false, false, false, false, 60, true); }
#[test]
fn h1_ring_zero_after_mount() { h1_run("H1 根槽读回全零(挂后)", true, false, true, false, false, 60, true); }
#[test]
fn h1_ring_and_account_error_after_mount() { h1_run("H1 根槽+账读错(挂后)", true, true, false, false, false, 60, true); }
#[test]
fn h1_account_error_after_mount() { h1_run("账读错(挂后,今天已有的10y形)", false, true, false, false, false, 60, true); }
#[test]
fn k23_ring_zero_at_mount_then_lifted() { h1_run("K2-3 根槽挂载时读回全零(之后恢复)", true, false, true, true, true, 60, true); }
#[test]
fn k23_ring_zero_at_mount_stays() { h1_run("K2-3 根槽挂载时读回全零(一直全零)", true, false, true, true, false, 60, true); }
#[test]
fn k23_ring_error_at_mount_then_lifted() { h1_run("K2-3b 根槽挂载时读错(之后恢复)", true, false, false, true, true, 60, true); }
#[test]
fn k23_ring_error_at_mount_stays() { h1_run("K2-3b 根槽挂载时读错(一直读错)", true, false, false, true, false, 60, true); }
#[test]
fn k23_ring_and_account_zero_at_mount_then_lifted() { h1_run("K2-3c 根槽+账挂载时读回全零(之后恢复)", true, true, true, true, true, 60, true); }
#[test]
fn k23_baseline_no_fault() { h1_run("基线 无故障", false, false, false, false, false, 60, true); }
