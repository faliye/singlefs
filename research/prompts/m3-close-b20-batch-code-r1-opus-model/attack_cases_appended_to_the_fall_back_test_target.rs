
// ===== 攻方用例（只在副本里）=====

/// 池级 checker 在 `image` 上判出的违例名。
fn attack_violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| {
            matches!(
                verdict,
                singlefs_checker::image::InvariantVerdict::Violated(_)
            )
        })
        .map(|(identifier, _)| identifier)
        .collect()
}

/// 错误的 Debug 串只留前 240 个字符。
fn attack_short(text: String) -> String {
    text.chars().take(240).collect()
}

/// 攻方面一与面二：判退那一形（崩在 R 的根槽之后、轮换之前；这一次挂载里 R 的单元一直读回全零、R 的记录读得出 ⇒ R 走读失败、没见证，退到 P），
/// R 的根槽只在第 k 次读读回全零（k 从 1 扫到 16，另有一格不注这一错的对照）。可写挂载做成就立即正常卸载（同一串盘，R 的单元照旧读回全零）。
/// 每格打一行 `ATTACK_UNMOUNT_CELL`：挂载结局、根环表在 R 那一槽记的是什么（有效根的 txg / 读坏或全零）、抬 F 那一串能不能借 C 那一槽（H2）、
/// 卸载结局、R 的单元有没有被写掉、卸载之后池级 checker 的违例。
#[test]
fn attack_unmount_after_the_fall_back_with_the_root_slot_of_r_read_bad_on_one_read() {
    use singlefs_core::mount::{unmount, Unmounted};
    use singlefs_core::root_ring::target_for_publish;
    let recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
    let root_slot_write = recorded.root_slot_write();
    let (instance_of_r, txg_of_r) = recorded.key;
    for bad_read in std::iter::once(None).chain((1..=16u64).map(Some)) {
        let cell = bad_read.map_or_else(|| "control".to_string(), |k| format!("k={k}"));
        let reads = Rc::new(Cell::new(0u64));
        let mut devices = devices_with_the_root_slot_bad_on_chosen_reads(
            &crash_image,
            &ranges,
            root_slot_write,
            true,
            bad_read.unwrap_or(u64::MAX),
            &reads,
        );
        let mut mounted = match mount_writable(&recorded.pool.parameters, &mut devices) {
            Ok(mounted) => mounted,
            Err(refusal) => {
                println!(
                    "ATTACK_UNMOUNT_CELL {cell} mount=refused reads={} refusal={}",
                    reads.get(),
                    attack_short(format!("{refusal:?}"))
                );
                continue;
            }
        };
        let reads_after_the_mount = reads.get();
        let effective = (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg,
        );
        let occupancy = mounted
            .allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表");
        let ring_slot_of_r = target_for_publish(txg_of_r, occupancy.slots_per_region());
        let valid_root_recorded_on_the_slot_of_r =
            occupancy.checkpoint_txg_of_the_valid_root_recorded_on(ring_slot_of_r);
        let slot_of_r_recorded_bad_or_all_zero = occupancy
            .ring_slots_bad_or_reading_back_all_zero_at_mount()
            .contains_key(&ring_slot_of_r);
        let c_holds_the_slot_of_r = occupancy
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .contains_key(&ring_slot_of_r);
        let borrow_of_c_for_r = occupancy
            .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                instance_of_r,
                txg_of_r,
            )
            .map(|placements| placements.len());
        let unmounted = unmount(
            &recorded.pool.parameters,
            &mut devices,
            &mut mounted.allocator,
            &mut mounted.current,
            ShadowLedger::On,
        );
        let unmount_outcome = match &unmounted {
            Ok(Unmounted::FloorRaisedToTheCurrentVersion(_)) => "raised".to_string(),
            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => "nothing_written".to_string(),
            Err(error) => format!("refused:{}", attack_short(format!("{error:?}"))),
        };
        let image_after = image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
            &devices,
            recorded.pool.device_bytes,
        );
        println!(
            "ATTACK_UNMOUNT_CELL {cell} mount=ok reads_after_the_mount={reads_after_the_mount} reads_after_the_unmount={} effective={effective:?} \
             key_of_r={:?} valid_root_recorded_on_the_slot_of_r={valid_root_recorded_on_the_slot_of_r:?} slot_of_r_recorded_bad_or_all_zero={slot_of_r_recorded_bad_or_all_zero} \
             c_holds_the_slot_of_r={c_holds_the_slot_of_r} borrow_of_c_for_r={borrow_of_c_for_r:?} unmount={unmount_outcome} \
             units_of_r_overwritten={} violated={:?}",
            reads.get(),
            (instance_of_r, txg_of_r),
            units_of_r_overwritten_on(&recorded, &image_after).len(),
            attack_violated_invariants(&image_after)
        );
    }
}

/// 攻方面一（影子账那一侧）：先照上一条的判退那一形做一次可写挂载 M1（不注根槽错，退到 P、写行与暖机抛弃 R），M1 交回之后断电（不卸载）。
/// 再挂 M2：最新那条根是 M1 的，它的实例表抛弃 R；R 的单元这一次读得出。R 的根槽只在第 k 次读读回全零（k 从 1 扫到 12，另有对照格）。
/// 每格打一行 `ATTACK_ABANDONED_CELL`：M2 结局、R 是不是读阶段读到的被抛弃根、根环表在 R 那一槽记成被抛弃根没有、影子账隔离了几槽、
/// 之后在 M2 的会话上依次覆盖写（至多 8 次，R 的根还在根环里才核）R 的单元第一次被写掉是第几次。
#[test]
fn attack_abandoned_root_read_by_the_read_stage_and_read_bad_on_one_later_read() {
    use singlefs_core::root_ring::target_for_publish;
    let mut recorded =
        record_one_publish_after_overwrites(OVERWRITES_BEFORE_R, PublishRecordedAsR::Overwrite);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
    let root_slot_write = recorded.root_slot_write().clone();
    let (instance_of_r, txg_of_r) = recorded.key;
    let first_mount_reads = Rc::new(Cell::new(0u64));
    let mut first_mount_devices = devices_with_the_root_slot_bad_on_chosen_reads(
        &crash_image,
        &ranges,
        &root_slot_write,
        true,
        u64::MAX,
        &first_mount_reads,
    );
    let first_mount = mount_writable(&recorded.pool.parameters, &mut first_mount_devices)
        .expect("M1：判退到 P、可写挂载做成");
    assert_eq!(
        (
            first_mount.output.effective_root.instance,
            first_mount.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "M1 建在 P 上"
    );
    for version in std::iter::once(&first_mount.output.row_publish)
        .chain(&first_mount.output.warm_up_publishes)
    {
        let root = version.root();
        recorded.pool.content_of_each_root.insert(
            (root.instance, root.checkpoint_txg),
            recorded.previous_content.clone(),
        );
    }
    let image_after_the_first_mount = image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
        &first_mount_devices,
        recorded.pool.device_bytes,
    );
    drop(first_mount);
    println!(
        "ATTACK_ABANDONED_SETUP key_of_r={:?} r_in_the_ring_after_m1={} units_of_r_overwritten_after_m1={}",
        (instance_of_r, txg_of_r),
        root_is_in_the_ring(&image_after_the_first_mount, recorded.key),
        units_of_r_overwritten_on(&recorded, &image_after_the_first_mount).len()
    );
    for bad_read in std::iter::once(None).chain((1..=12u64).map(Some)) {
        let cell = bad_read.map_or_else(|| "control".to_string(), |k| format!("k={k}"));
        let reads = Rc::new(Cell::new(0u64));
        let mut devices = devices_with_the_root_slot_bad_on_chosen_reads(
            &image_after_the_first_mount,
            &RangesReadingBackAllZero::default(),
            &root_slot_write,
            true,
            bad_read.unwrap_or(u64::MAX),
            &reads,
        );
        let mounted = match mount_writable(&recorded.pool.parameters, &mut devices) {
            Ok(mounted) => mounted,
            Err(refusal) => {
                println!(
                    "ATTACK_ABANDONED_CELL {cell} mount=refused reads={} refusal={}",
                    reads.get(),
                    attack_short(format!("{refusal:?}"))
                );
                continue;
            }
        };
        let occupancy = mounted
            .allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表");
        let ring_slot_of_r = target_for_publish(txg_of_r, occupancy.slots_per_region());
        let slot_of_r_recorded_as_abandoned = occupancy
            .ring_slots_recorded_as_holding_an_abandoned_root()
            .contains_key(&ring_slot_of_r);
        let isolated: u64 = mounted
            .output
            .isolated_slots_per_device
            .iter()
            .map(|(_, slots)| *slots)
            .sum();
        let abandoned_roots_unreadable = mounted.output.abandoned_roots_unreadable;
        let reads_after_the_mount = reads.get();
        let image_after_the_mount = image_of_the_devices_with_the_root_slot_bad_on_chosen_reads(
            &devices,
            recorded.pool.device_bytes,
        );
        let overwritten_by_the_mount =
            units_of_r_overwritten_on(&recorded, &image_after_the_mount).len();
        let mut pool = common_unwitnessed_publish::pool_after_the_mount(
            &recorded,
            &image_after_the_mount,
            mounted,
        );
        let mut first_overwrite_writing_r = None;
        let mut overwrites_done = 0u64;
        for overwrite in 1..=8u64 {
            if pool.overwrite(OVERWRITE_BYTES).is_err() {
                break;
            }
            overwrites_done = overwrite;
            let image = pool.image();
            if root_is_in_the_ring(&image, recorded.key)
                && !units_of_r_overwritten_on(&recorded, &image).is_empty()
            {
                first_overwrite_writing_r = Some(overwrite);
                break;
            }
        }
        println!(
            "ATTACK_ABANDONED_CELL {cell} mount=ok reads_after_the_mount={reads_after_the_mount} slot_of_r_recorded_as_abandoned={slot_of_r_recorded_as_abandoned} \
             isolated={isolated} abandoned_roots_unreadable={abandoned_roots_unreadable} units_of_r_overwritten_by_the_mount={overwritten_by_the_mount} \
             overwrites_done={overwrites_done} first_overwrite_writing_r_while_r_in_the_ring={first_overwrite_writing_r:?}"
        );
    }
}
