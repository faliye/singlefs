use super::*;
use crate::tests::with_the_rotation_after_the_newest_publish_not_written;

/// 前 n 个写之后的镜像要用的最小历史：mkfs → 可写挂载 → 首个文件 → n1 次覆盖写 → 关闭 C。
fn closed_history(overwrites: u64) -> SecondRunHistory {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut history = second_run_start(&GEOMETRY_PRIMARY, overwrites).expect("历史走得通");
    second_run_close(
        &mut history,
        &parameters,
        Closing::ProcessExit,
        "test",
        "test",
    )
    .expect("关闭 C");
    history
}

/// 抛弃步的落点：最新 k 条根的根槽（一块盘上一个物理块）+ 断链发布的记录两份 / 第一个点名单元两份 / 它的第一份。
#[test]
fn unreadable_ranges_of_the_abandonment_name_the_root_slots_and_the_break() {
    let history = closed_history(1);
    let geometry = independent_geometry(&history.pool).expect("几何");
    let tip = history.tip();
    let (tip_device, tip_offset) =
        root_slot_of(&history.pool, &geometry, tip).expect("最新根的根槽");
    let records = records_of_publish(&history.pool, &geometry, tip);
    assert!(!records.is_empty());
    let every_record = unreadable_ranges_of_the_abandonment(
        &history,
        &geometry,
        1,
        UnreadableForm::ReadFails,
        ReplayBreak::EveryRecord,
    )
    .expect("落点")
    .expect("有记录");
    assert_eq!(every_record.len(), 1 + 2 * records.len());
    assert_eq!(
        (
            every_record[0].device,
            every_record[0].offset,
            every_record[0].length
        ),
        (tip_device, tip_offset, 512)
    );
    assert!(every_record[1..]
        .iter()
        .all(|range| range.length == JOURNAL_RECORD_BYTES));
    assert!(every_record[1..]
        .iter()
        .any(|range| range.device == DeviceIdentity(1)));
    let both_copies = unreadable_ranges_of_the_abandonment(
        &history,
        &geometry,
        1,
        UnreadableForm::ReadsZeros,
        ReplayBreak::FirstNamedUnit,
    )
    .expect("落点")
    .expect("有点名单元");
    assert_eq!(both_copies.len(), 3);
    assert_ne!(both_copies[1].device, both_copies[2].device);
    let first_copy = unreadable_ranges_of_the_abandonment(
        &history,
        &geometry,
        1,
        UnreadableForm::ReadsZeros,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
    )
    .expect("落点")
    .expect("有点名单元");
    assert_eq!(first_copy, both_copies[..2].to_vec());
    let two_roots = unreadable_ranges_of_the_abandonment(
        &history,
        &geometry,
        2,
        UnreadableForm::ReadFails,
        ReplayBreak::EveryRecord,
    )
    .expect("落点")
    .expect("有记录");
    assert_eq!(
        two_roots.len(),
        2 + 2 * records_of_publish(
            &history.pool,
            &geometry,
            history.timeline[history.timeline.len() - 2]
        )
        .len()
    );
}

/// 第 2 次跑修订第 1 条的依据（重放第一份读错就停，第二份一次都没读）在 C694 的 F1_ANY 之后不再整句成立：可写挂载的
/// 「记录有缺口时拒可写」（F1，D23（journal 的角色与格式） 已定项 14，点名单元两份副本都验不过才拒）第一份读坏会去读第二份，
/// 那一份被拦到恰好一次只坏第一份照样断得了链（b = unit_first_copy）。
#[test]
fn a_named_unit_whose_first_copy_reads_bad_has_its_second_copy_read_once_by_the_record_gap_refusal_of_the_writable_mount(
) {
    let history = closed_history(1);
    let geometry = independent_geometry(&history.pool).expect("几何");
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let pool = with_the_rotation_after_the_newest_publish_not_written(&history.pool);
    let both = unreadable_ranges_of_the_abandonment(
        &history,
        &geometry,
        1,
        UnreadableForm::ReadFails,
        ReplayBreak::FirstNamedUnit,
    )
    .expect("落点")
    .expect("有点名单元");
    let run = run_instrumented_mount_writable(&pool, &parameters, &both, None, None);
    assert!(run.intercepted_reads[0] > 0, "根槽拦到了");
    assert!(run.intercepted_reads[1] > 0, "第一份拦到了");
    assert_eq!(
        run.intercepted_reads[2], 1,
        "F1 判两份都验不过才拒：第二份恰好读一次"
    );
    let first = unreadable_ranges_of_the_abandonment(
        &history,
        &geometry,
        1,
        UnreadableForm::ReadFails,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
    )
    .expect("落点")
    .expect("有点名单元");
    let first_copy_run = run_instrumented_mount_writable(&pool, &parameters, &first, None, None);
    let mounted = first_copy_run.result.as_ref().expect("挂载做成");
    assert_eq!(
        root_pair(&mounted.output.chosen_root),
        history.timeline[history.timeline.len() - 2]
    );
    assert_eq!(
        abandoned_roots_on(&first_copy_run.pool_after).expect("S5 不停"),
        BTreeSet::from([tip_of(&history)]),
        "只坏第一份，最新那条根就被抛弃"
    );
}

fn tip_of(history: &SecondRunHistory) -> TimelineRoot {
    history.tip()
}

/// 第 2 次跑修订第 2 条的依据（H1d 的 n1 = 0 时写行那次发布盖到被藏的首个文件那条根引用的单元）在 C694 的 C 预隔离之后不再成立：
/// 可写挂载把水位之上没施加的记录点名的落点与那次发布的账隔离到写掉那条记录的根环槽（D23（journal 的角色与格式） 已定项 14
/// 「可写挂载时的预隔离」），写行那次发布在它的根落盘之前写的单元，n1 = 0、1 都一处不盖。
#[test]
fn the_row_publish_before_its_root_overwrites_none_of_the_hidden_newest_root_since_the_writable_mount_isolates_its_units(
) {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut overwritten_by_overwrites = Vec::new();
    for overwrites in [0u64, 1] {
        let history = closed_history(overwrites);
        let geometry = independent_geometry(&history.pool).expect("几何");
        let (_, tip_record) = landing_root_on(&history.pool).expect("择根");
        let tip_units = placements_referenced_by_root_on(&history.pool, &tip_record).expect("账");
        let pool = with_the_rotation_after_the_newest_publish_not_written(&history.pool);
        let ranges = unreadable_ranges_of_the_abandonment(
            &history,
            &geometry,
            1,
            UnreadableForm::ReadsZeros,
            ReplayBreak::FirstCopyOfTheFirstNamedUnit,
        )
        .expect("落点")
        .expect("有点名单元");
        let reference = run_instrumented_mount_writable(&pool, &parameters, &ranges, None, None);
        let first_root_write = index_of_the_first_root_slot_write(
            &reference.applied,
            &root_slot_device_offsets(&geometry),
            u64::from(geometry.physical_block_size),
        )
        .expect("写行那次发布写了根");
        let post = image_with_the_first_writes(&pool, &reference.applied, first_root_write);
        overwritten_by_overwrites.push(placements_overwritten_between(&pool, &post, &tip_units));
        assert_eq!(
            landing_root_on(&post).expect("择根").0,
            history.tip(),
            "根没落盘，恢复仍落到被藏的那条"
        );
    }
    assert_eq!(
        overwritten_by_overwrites[0], 0,
        "n1 = 0：{overwritten_by_overwrites:?}"
    );
    assert_eq!(
        overwritten_by_overwrites[1], 0,
        "n1 = 1：{overwritten_by_overwrites:?}"
    );
}

/// 7.1 第六行「今天 = 丙」：新实例第一次发布的 txg = 装置按这次读得出的根与记录算的 max + 1——不藏、藏最新根与它的记录、
/// 只藏最新根而记录读得出三种都对得上。
#[test]
fn first_txg_by_the_rule_of_today_matches_the_row_publish_of_the_mount() {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let history = closed_history(1);
    let geometry = independent_geometry(&history.pool).expect("几何");
    let rewound = with_the_rotation_after_the_newest_publish_not_written(&history.pool);
    let mut observed = Vec::new();
    for (pool, ranges) in [
        (&history.pool, Vec::new()),
        (
            &rewound,
            unreadable_ranges_of_the_abandonment(
                &history,
                &geometry,
                1,
                UnreadableForm::ReadFails,
                ReplayBreak::EveryRecord,
            )
            .expect("落点")
            .expect("有记录"),
        ),
        (
            &rewound,
            unreadable_ranges_of_the_abandonment(
                &history,
                &geometry,
                1,
                UnreadableForm::ReadFails,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            )
            .expect("落点")
            .expect("有点名单元"),
        ),
    ] {
        let run = run_instrumented_mount_writable(pool, &parameters, &ranges, None, None);
        let row_txg = run
            .result
            .as_ref()
            .expect("挂载做成")
            .output
            .row_publish
            .root()
            .checkpoint_txg
            .0;
        assert_eq!(
            row_txg,
            first_txg_by_the_rule_of_today(pool, &geometry, &ranges)
        );
        observed.push(row_txg);
    }
    let tip_txg = history.tip().1;
    assert_eq!(observed, vec![tip_txg + 1, tip_txg, tip_txg + 1]);
}

/// Q1-0 的最小复现（前提 1），系统配置没见证到被藏的最新根（`with_the_rotation_after_the_newest_publish_not_written`；C554 乙之后见证只罩最新一次发布之外的，
/// 藏两条根时较旧的那一槽系统配置仍见证第二新那条，可写挂载照拒，这一形造不出）：藏最新那条根、只坏它第一条记录点名的第一个单元的第一份，
/// 挂载做成、最新那条根被按新实例表判抛弃（装置与 crates 一致）；断它自己的全部记录时，新实例第一次发布的 txg 恰与被藏那条相同、
/// 落进同一个根槽，被藏那条就不在了（抛弃集合为空）。
#[test]
fn abandonment_step_abandons_the_hidden_newest_root_unless_the_row_publish_lands_on_its_slot() {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let history = closed_history(1);
    let geometry = independent_geometry(&history.pool).expect("几何");
    let pool = with_the_rotation_after_the_newest_publish_not_written(&history.pool);
    let unit_broken = unreadable_ranges_of_the_abandonment(
        &history,
        &geometry,
        1,
        UnreadableForm::ReadFails,
        ReplayBreak::FirstCopyOfTheFirstNamedUnit,
    )
    .expect("落点")
    .expect("有点名单元");
    let run = run_instrumented_mount_writable(&pool, &parameters, &unit_broken, None, None);
    assert!(run.result.is_ok());
    assert_eq!(
        abandoned_roots_on(&run.pool_after).expect("S5 不停"),
        BTreeSet::from([history.tip()])
    );
    let one = unreadable_ranges_of_the_abandonment(
        &history,
        &geometry,
        1,
        UnreadableForm::ReadFails,
        ReplayBreak::EveryRecord,
    )
    .expect("落点")
    .expect("有记录");
    let one_root_run = run_instrumented_mount_writable(&pool, &parameters, &one, None, None);
    let mounted = one_root_run.result.as_ref().expect("挂载做成");
    assert_eq!(
        mounted.output.row_publish.root().checkpoint_txg.0,
        history.tip().1
    );
    assert!(abandoned_roots_on(&one_root_run.pool_after)
        .expect("S5 不停")
        .is_empty());
}

/// 关闭 U（登记第九节 M16 的反面）：正常卸载把每块盘最新系统配置里的 F 抬到卸载之前现行那一版的 txg，推 2 或 3 次空发布（7.1 第四行）。
#[test]
fn normal_unmount_raises_the_floor_to_the_current_txg_and_pushes_the_unmount_publishes() {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut history = second_run_start(&GEOMETRY_PRIMARY, 1).expect("历史");
    let before = history.timeline.len();
    let before_txg = history.tip().1;
    second_run_close(
        &mut history,
        &parameters,
        Closing::NormalUnmount,
        "test",
        "test",
    )
    .expect("卸载");
    let pushed = history.timeline.len() - before;
    assert_eq!(pushed, if before_txg % 3 == 1 { 3 } else { 2 });
    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    assert_eq!(
        highest_floor_in_the_system_configurations(&history.pool, slot_spacing),
        before_txg
    );
    assert!(history.session.is_none());
}
