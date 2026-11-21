//! 192、216、264 槽几种池上一块盘在重建那一遍读不出有效根：整环重读一次之后挂上，带活文件的池上只剩一块盘时照旧读得出每条根。

use super::*;

/// 只坏重建那一遍的计划在这几个起点上的预期：照常挂上、重读过一次（钩子恰调一次，在重建那一遍之后），交回与不注错的再挂同一份盘面，
/// 两块盘都在时池级 checker 不判违例，丢盘整读与崩溃之前相同，挂载之前环里还在、不低于 F 的每条自证根丢盘整读与挂载之前相同。
pub fn rereads_and_mounts_like_a_remount_without_faults() -> RemountUnderThePlan {
    RemountUnderThePlan {
        root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
        hook_calls_after_exactly_the_rebuild_pass: vec![true],
        faulted_reads_are_the_named_slots_once: true,
        same_disk_as_a_remount_without_faults: true,
        lone_survivor_reads_what_it_read_before_the_crash: [true, true],
        lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [true, true],
    }
}

/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
/// 预期：盘 0 一条有效根都没读出，整环重读一次（钩子恰调一次），重读读得出，照常可写挂载，交回与不注错的再挂同一份盘面；
/// 两块盘都在时池级 checker 不判违例；丢盘整读与卸载之前相同。
/// 判别力：拿掉「一块盘没读出也重读」（`crates/mutations.tsv`「Q4 rr2…」）时 m 取不到、当场只放到环里最旧有效根，
/// 窄盘上写行取不到落点、取号之前拒，第一项红。
pub fn run_the_scenario_where_on_the_192_slot_pool_after_a_crashed_unmount_device_zero_failing_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
            DEVICE_ZERO_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 0 报读错：整环重读、照常挂上"
    );
}

/// 场景：`192:9:3:1` n = 8，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
/// 预期与判别力同上一条。
pub fn run_the_scenario_where_on_the_192_slot_pool_after_a_crashed_unmount_device_one_failing_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
            DEVICE_ONE_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 1 报读错：整环重读、照常挂上"
    );
}

/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`）。
/// 预期与判别力同上（盘 1 一条有效根都没读出）。
pub fn run_the_scenario_where_on_the_192_slot_pool_after_a_crashed_unmount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上"
    );
}

/// 场景：`192:9:3:1` n = 8，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
/// 预期与判别力同上（盘 0 一条有效根都没读出）。
pub fn run_the_scenario_where_on_the_192_slot_pool_after_a_crashed_unmount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf192SlotsAfterAnUnmountCrashingAfterEightWrites,
            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上"
    );
}

/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
/// 预期与判别力同第一条。
pub fn run_the_scenario_where_on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_failing_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
            DEVICE_ZERO_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 0 报读错：整环重读、照常挂上"
    );
}

/// 场景：`216:7:5:4` n = now，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
/// 预期与判别力同第一条。
pub fn run_the_scenario_where_on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_failing_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
            DEVICE_ONE_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 1 报读错：整环重读、照常挂上"
    );
}

/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`）。
/// 预期与判别力同第一条。
pub fn run_the_scenario_where_on_the_216_slot_pool_crashed_after_its_continued_mount_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上"
    );
}

/// 场景：`216:7:5:4` n = now，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
/// 预期与判别力同第一条。
pub fn run_the_scenario_where_on_the_216_slot_pool_crashed_after_its_continued_mount_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts(
    images: &mut PoolCheckerImages,
) {
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf216SlotsCrashingRightAfterTheContinuedMount,
            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上"
    );
}

/// `264:6:4` 那个起点上同一份读坏计划之前，先核它真有可比的：挂载之前丢任一块盘，只读挂载择到的那一版有活文件，环里还在、不低于 F 的根里
/// 也有带活文件的（不然丢盘整读两项比的又是空表）。
pub fn assert_the_live_file_starting_point_has_live_files_to_compare(plan: RebuildPassPlan) {
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
    );
    // 迭代上界是两块盘；没有提前出口。
    for lost in [DeviceIdentity(0), DeviceIdentity(1)] {
        let live_files =
            live_files_read_whole(&image_without(&before.image_before_the_crash, lost))
                .unwrap_or_else(|failure| {
                    panic!("前提：丢盘 {} 之后只读挂载、整读做得成：{failure}", lost.0)
                });
        assert!(
            live_files.iter().any(|(_, bytes)| !bytes.is_empty()),
            "前提：丢盘 {} 之后剩下那块盘上有一个不空的活文件（{plan:?}）",
            lost.0
        );
        let (compared, compared_with_live_files, _) =
            roots_still_in_the_ring_read_on_a_lone_survivor(
                &before.image_to_remount,
                &before.image_to_remount,
                lost,
            );
        assert!(
            compared > 0 && compared_with_live_files > 0,
            "前提：丢盘 {} 之后环里不低于 F 的根有 {compared} 条读得出，其中 {compared_with_live_files} 条带活文件",
            lost.0
        );
    }
}

/// 再挂之后接着覆盖写几次（只读挂载择到的那一版之外，环里更旧的有效根引用的单元要在接着写的时候才会被发出去）。264 槽的窄盘上
/// 第 12 次覆盖写抬过 F 仍放不下（`NoSpaceAfterRaisingTheFloor`，现量），取它之前的 10 次。
pub const OVERWRITES_AFTER_THE_REMOUNT: usize = 10;

/// `264:6:4` 那个起点上照计划只坏重建那一遍再挂，之后接着覆盖写 [`OVERWRITES_AFTER_THE_REMOUNT`] 次（准入关着：窄盘扣不起环窗口，起点就是
/// 关着准入造的，落点是唯一的墙）；交回丢盘 0、丢盘 1 时 [`roots_still_in_the_ring_read_on_a_lone_survivor`] 比的（几条、带活文件的几条、
/// 读得不一样的 txg）——挂载之前环里那几条根，写到这一刻仍在环里、不低于 F 的。
pub fn lone_survivors_after_overwrites_following_the_remount(
    plan: RebuildPassPlan,
) -> [(usize, usize, Vec<u64>); 2] {
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
    );
    let faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
        faulted_slots_of_the_plan(&before.image_to_remount, plan)
            .into_iter()
            .map(|(ring_slot, fault)| {
                (
                    (
                        ring_slot,
                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
                    ),
                    fault,
                )
            })
            .collect();
    let root_ring_reads =
        root_ring_reads_failing(&before.image_to_remount, faulted_reads_of_each_slot);
    let (mounted, image_after_the_remount, _) = remount_through(&before, &root_ring_reads);
    let mounted = mounted.unwrap_or_else(|error| panic!("照计划再可写挂载做得成：{error:?}"));
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    session
        .allocator
        .set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
        parameters: before.parameters.clone(),
        device_bytes: before.image_to_remount.device_size_in_bytes,
        devices: plain_devices_on(&image_after_the_remount),
        session: Some(session),
        write_time_seconds: FIRST_WRITE_TIME_SECONDS + 100_000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    // 迭代上界是 OVERWRITES_AFTER_THE_REMOUNT；没有提前出口。
    for overwrite_index in 0..OVERWRITES_AFTER_THE_REMOUNT {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrite_index} 次覆盖写：{refusal:?}"));
    }
    let image_after_the_overwrites = pool.image();
    [DeviceIdentity(0), DeviceIdentity(1)].map(|lost| {
        roots_still_in_the_ring_read_on_a_lone_survivor(
            &before.image_to_remount,
            &image_after_the_overwrites,
            lost,
        )
    })
}

/// 接着覆盖写之后的预期：丢任一块盘，挂载之前环里还在、不低于 F 的根都比到了（至少一条、其中至少一条带活文件），没有一条读得不一样。
pub fn assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
    compared_on_each_lone_survivor: [(usize, usize, Vec<u64>); 2],
) {
    // 迭代上界是两块盘；没有提前出口。
    for (lost, (compared, compared_with_live_files, read_differently)) in
        [0u32, 1].into_iter().zip(compared_on_each_lone_survivor)
    {
        assert!(
            compared > 0 && compared_with_live_files > 0,
            "丢盘 {lost}：环里还在、不低于 F 的根比了 {compared} 条，其中 {compared_with_live_files} 条带活文件（要比到带活文件的根）"
        );
        assert_eq!(
            read_differently,
            Vec::<u64>::new(),
            "丢盘 {lost}：再挂、接着覆盖写之后，挂载之前环里还在、不低于 F 的根在剩下那块盘上整读与挂载之前相同（读得不一样的根的 txg）"
        );
    }
}

/// 场景：`264:6:4`（有活文件，可写挂载做完就崩），重建那一遍盘 0 的每个根环槽报读错（`P5:all/-:e`）。
/// 预期：整环重读一次、照常挂上，与不注错的再挂同一份盘面；丢任一块盘之后剩下那块盘只读挂载整读与崩溃之前相同（这一回比的是一个不空的活文件），
/// 挂载之前环里还在、不低于 F 的每条自证根丢盘整读也与挂载之前相同；再接着覆盖写 [`OVERWRITES_AFTER_THE_REMOUNT`] 次之后，
/// 挂载之前环里仍在、不低于 F 的根丢盘整读还与挂载之前相同。
/// 判别力：重读之后的读数只剩最新那条根（`crates/mutations.tsv`「rr2 用例丢盘整读：重读之后的读数只剩最新那条根…」）时
/// 回收门槛与当场放开越过环里更旧的有效根，接着覆盖写把它们引用的单元写掉，接着写之后那一项先红（它排在前面；后面那一项里两块盘都在时
/// 池级 checker 也判 I-3.1 记账少于遍历）。
pub fn run_the_scenario_where_on_the_264_slot_pool_with_a_live_file_device_zero_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
    images: &mut PoolCheckerImages,
) {
    assert_the_live_file_starting_point_has_live_files_to_compare(DEVICE_ZERO_REPORTS_READ_ERRORS);
    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
        lone_survivors_after_overwrites_following_the_remount(DEVICE_ZERO_REPORTS_READ_ERRORS),
    );
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
            DEVICE_ZERO_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "有活文件的起点、只坏重建那一遍、盘 0 报读错：整环重读、照常挂上，丢盘整读与之前相同"
    );
}

/// 场景：`264:6:4`，重建那一遍盘 1 的每个根环槽报读错（`P5:-/all:e`）。
/// 预期与判别力同上一条。
pub fn run_the_scenario_where_on_the_264_slot_pool_with_a_live_file_device_one_failing_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
    images: &mut PoolCheckerImages,
) {
    assert_the_live_file_starting_point_has_live_files_to_compare(DEVICE_ONE_REPORTS_READ_ERRORS);
    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
        lone_survivors_after_overwrites_following_the_remount(DEVICE_ONE_REPORTS_READ_ERRORS),
    );
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
            DEVICE_ONE_REPORTS_READ_ERRORS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "有活文件的起点、只坏重建那一遍、盘 1 报读错：整环重读、照常挂上，丢盘整读与之前相同"
    );
}

/// 场景：`264:6:4`，重建那一遍盘 0 除最新那条根之外、盘 1 每个根环槽读回全 0（`P5:but1/all:z`；K3）。
/// 预期与判别力同 `…device_zero_failing_the_rebuild_pass…` 那一条（盘 1 一条有效根都没读出，整环重读；变异行
/// 「rr2 用例丢盘整读补全零与乱字节：重读之后的读数只剩最新那条根…」）。
pub fn run_the_scenario_where_on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
    images: &mut PoolCheckerImages,
) {
    assert_the_live_file_starting_point_has_live_files_to_compare(
        DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS,
    );
    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
        lone_survivors_after_overwrites_following_the_remount(
            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS,
        ),
    );
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
            DEVICE_ZERO_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_ZEROS
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "有活文件的起点、只坏重建那一遍、盘 1 读回全 0：整环重读、照常挂上，丢盘整读与之前相同"
    );
}

/// 场景：`264:6:4`，重建那一遍盘 0 每个、盘 1 除最新那条根之外的根环槽读回乱字节（`P5:all/but1:g`）。
/// 预期与判别力同上一条（盘 0 一条有效根都没读出，整环重读）。
pub fn run_the_scenario_where_on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before(
    images: &mut PoolCheckerImages,
) {
    assert_the_live_file_starting_point_has_live_files_to_compare(
        DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE,
    );
    assert_lone_survivors_read_every_root_still_in_the_ring_as_before(
        lone_survivors_after_overwrites_following_the_remount(
            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE,
        ),
    );
    assert_eq!(
        remount_under_the_plan(
            images,
            StartingPoint::UnitAreaOf264SlotsWithALiveFileCrashingRightAfterAWritableMount,
            DEVICE_ONE_KEEPS_ITS_NEWEST_ROOT_AND_THE_REST_READS_BACK_GARBAGE
        ),
        rereads_and_mounts_like_a_remount_without_faults(),
        "有活文件的起点、只坏重建那一遍、盘 0 读回乱字节：整环重读、照常挂上，丢盘整读与之前相同"
    );
}
