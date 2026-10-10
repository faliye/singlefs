
/// 在一个卡住的池上扫写的长度：每个 g 发一次长到 g 个单元的顺序写（`total_units` 为真时 g 是整份长度，否则是现在长度加 g），
/// 报「被宽护卡住」的照出路二走、与预演逐格比。交回（报卡住的格数, 对不上的格数）。
fn atk_scan_lengths_and_follow(label: &str, snap: &AtkSnap, grows: &[usize], total_units: bool) -> (usize, usize) {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let size_now = snap.content_now.len().div_ceil(unit_bytes);
    let (mut stuck_cells, mut disagree) = (0, 0);
    for grow in grows {
        let units = if total_units { *grow } else { size_now + grow };
        let mut pool = snap.pool();
        let write = atk_write_of(&mut pool, units);
        let outcome = atk_apply(&mut pool, &write);
        let Some(stuck) = atk_stuck_of(&outcome) else {
            println!("ATK {label} g={grow} first={}", atk_kind(&outcome));
            continue;
        };
        stuck_cells += 1;
        let after_report = AtkSnap::of(&pool);
        let (line, ok) = atk_follow_way_two(&after_report, stuck, &write);
        if !ok {
            disagree += 1;
        }
        println!("ATK {label} g={grow} first=stuck n_pub={} {line}", stuck.empty_publishes_until_the_ring_slot_of_r_is_written);
    }
    (stuck_cells, disagree)
}

/// 实现员三形：卡住的那次覆盖写自己的报照出路二走；再在卡住那一刻起写长到 g 个单元（放开）。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_implementer_forms_rehearsal_vs_real() {
    let grows: Vec<usize> = atk_env_list("ATK_GROWS", "2,5,10,20,30,40,50,60,70,80");
    let forms = [
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack { overwrites_before: 20 },
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation { overwrites_before: 40 },
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion { overwrites_before: 160 },
    ];
    let (mut stuck_cells, mut disagree) = (0, 0);
    for form in forms {
        let PoolStuckBehindTheWideProtection { pool, stuck, .. } = pool_stuck_behind_the_wide_protection(form);
        atk_register_stuck(&pool, &stuck);
        let snap = AtkSnap::of(&pool);
        let overwrite = AtkWrite {
            content: common_unwitnessed_publish::content_of(OVERWRITE_BYTES, pool.write_time_seconds),
            write_time_seconds: pool.write_time_seconds,
            overwrite: true,
        };
        let (line, ok) = atk_follow_way_two(&snap, &stuck, &overwrite);
        stuck_cells += 1;
        if !ok {
            disagree += 1;
        }
        println!("ATK k1impl form={form:?} own_overwrite {line}");
        let (cells, bad) = atk_scan_lengths_and_follow(&format!("k1impl form={form:?}"), &snap, &grows, true);
        stuck_cells += cells;
        disagree += bad;
    }
    println!("ATK k1impl totals stuck={stuck_cells} disagree={disagree}");
}

/// 两槽 C（Q 顺序写、R 建 inode，故障 cfgq,qroot,qlast,racct，挂上就撤）：挂上之后先覆盖写 `pre` 次（放开），再扫写的长度。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_two_c_slots_rehearsal_vs_real() {
    let pre_writes_list: Vec<usize> = atk_env_list("ATK_PRE_WRITES", "0,1,3,6,10");
    let grows: Vec<usize> = (1..=90).collect();
    let (mut stuck_cells, mut disagree) = (0, 0);
    for overwrites_before in [1u64, 3, 8] {
        for pre_writes in &pre_writes_list {
            let recorded = record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(overwrites_before);
            let crash_image = recorded.crash_image_with_the_root_of_r_persisted_and_its_rotation_lost();
            let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
            let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
            let Ok(mounted) = mount_writable_with_space_admission(&recorded.pool.parameters, &mut faulted_devices, SpaceAdmission::JudgedByTheFormula) else {
                println!("ATK k1two o={overwrites_before} pre={pre_writes} mount_refused");
                continue;
            };
            let image_after_the_mount = image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
            let (output, session) = MountedSession::of_the_mount(mounted);
            let content_of_p = recorded.pool.content_of_each_root.get(&recorded.key_of_p).cloned().expect("P");
            let mut contents = recorded.pool.content_of_each_root.clone();
            for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
                let root = version.root();
                contents.insert((root.instance, root.checkpoint_txg), content_of_p.clone());
            }
            let mut pool = PoolUnderTest {
                parameters: recorded.pool.parameters.clone(),
                device_bytes: recorded.pool.device_bytes,
                devices: plain_devices_on(&image_after_the_mount),
                session: Some(session),
                write_time_seconds: recorded.pool.write_time_seconds + 10_000,
                content_of_the_current_version: content_of_p,
                content_of_each_root: contents,
            };
            for _ in 0..*pre_writes {
                let _ = pool.overwrite(OVERWRITE_BYTES);
            }
            let c = atk_c_ring(&pool.session().allocator);
            let label = format!("k1two o={overwrites_before} pre={pre_writes} c={c:?}");
            let (cells, bad) = atk_scan_lengths_and_follow(&label, &AtkSnap::of(&pool), &grows, false);
            stuck_cells += cells;
            disagree += bad;
        }
    }
    println!("ATK k1two totals stuck={stuck_cells} disagree={disagree}");
}

/// C 那一圈：记着落点的槽数、记着的落点罩着的槽这一刻隔离着几个（两盘合计）。
fn atk_c_ring(allocator: &PoolAllocator) -> (usize, u64) {
    let Some(occupancy) = allocator.root_ring_occupancy() else {
        return (0, 0);
    };
    let holding = occupancy.ring_slots_holding_placements_named_by_unapplied_journal_records();
    let placements: Vec<PlacementOnDevice> = holding.values().flatten().copied().collect();
    let isolated: u64 = allocator.isolated_slots_covered_by(&placements).iter().map(|(_, n)| *n).sum();
    (holding.len(), isolated)
}
