
/// 攻方改法 G5（只在攻方副本上量过、被攻过零轮）：新用例那一形（覆盖写 3 次、卸载落下 15 个写就崩），比的槽从「卸载之前现行那一版换下的」
/// 放宽成「盘 1 上最新的根之后每一版换下的」（这一形里盘 1 最新的根 txg 4、现行那一版 txg 6：txg 5、6 两版各自换下的槽）。
#[test]
#[ignore = "攻方改法，只在副本里点名跑"]
fn fix_g5_every_version_after_the_newest_root_on_device_one_released_slots_stay_untouched_in_either_warm_up() {
    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            device_width,
            176,
        );
    let mut released_by_each_version: Vec<(u64, Vec<u64>)> = Vec::new();
    for overwrite_index in 0..OVERWRITES_BEFORE_AN_UNMOUNT_WHOSE_NEXT_MOUNT_WARMS_UP_TWICE {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
        });
        released_by_each_version.push((
            pool.session().current.root().checkpoint_txg.0,
            slots_released_by_the_current_version(&pool),
        ));
    }
    let after_the_unmount = image_after_an_unmount_crashing_after(
        &pool,
        device_width,
        UNMOUNT_WRITES_LANDED_BEFORE_A_MOUNT_THAT_WARMS_UP_TWICE,
    );
    let newest_root_on_device_one = recover(&image_without(&after_the_unmount, DeviceIdentity(0)), JournalPolicy::Consult)
        .effective_root
        .map(|(_, txg)| txg.0)
        .expect("盘 1 单独择得到根");
    let released_after_the_newest_root_on_device_one: Vec<u64> = released_by_each_version
        .iter()
        .filter(|(txg, _)| *txg > newest_root_on_device_one)
        .flat_map(|(_, slots)| slots.iter().copied())
        .collect();
    println!("G5 newest_root_on_device_one={newest_root_on_device_one} versions={:?}", released_by_each_version.iter().map(|(txg, slots)| (*txg, slots.len())).collect::<Vec<_>>());
    let mut remount_devices = plain_devices_on(&after_the_unmount);
    let mounted = mount_writable_through(
        RemountEntry::WithDeletionBatchesOfOne,
        &pool,
        &mut remount_devices,
    )
    .unwrap_or_else(|error| panic!("再可写挂载：{error:?}"));
    assert_eq!(mounted.output.warm_up_publishes.len(), 2);
    let taken: Vec<Vec<u64>> = std::iter::once(&mounted.output.row_publish)
        .chain(&mounted.output.warm_up_publishes)
        .map(|version| {
            version
                .version_of_trees()
                .map(|version| {
                    version
                        .units
                        .iter()
                        .flat_map(|unit| unit.slot.0..unit.slot.0 + unit.identity.span_slots())
                        .filter(|slot| released_after_the_newest_root_on_device_one.contains(slot))
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(taken, vec![Vec::<u64>::new(); 3], "写行与两次暖机都不写进盘 1 最新的根之后各版换下的槽 {released_after_the_newest_root_on_device_one:?}");
}
