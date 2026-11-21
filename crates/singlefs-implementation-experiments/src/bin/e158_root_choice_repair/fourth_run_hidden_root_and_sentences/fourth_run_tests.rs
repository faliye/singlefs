use super::*;

/// D22（单元原子性怎么合成） 已定项 2「槽序」：(txg mod 3, (txg div 3) mod S)（变异 M21：h = 4 时 (h div 3) mod 8 = 1 ≠ h mod 8 = 4）。
#[test]
fn the_predicted_root_ring_slot_follows_the_slot_order() {
    assert_eq!(predicted_root_ring_slot_of_a_publish(4, 8), (1, 1));
    assert_eq!(predicted_root_ring_slot_of_a_publish(13, 8), (1, 4));
    assert_eq!(predicted_root_ring_slot_of_a_publish(30, 4), (0, 2));
}

/// Q0 去向（「读法写死」表；变异 M16）：s_h 上换成别的键的根、环里没有内容号 ≥ 被藏那一版的 ⇒ 根槽被盖；原样的镜像、没交回 Ok ⇒ 原样留着。
#[test]
fn the_hidden_root_destination_tells_an_overwritten_slot_from_an_untouched_one() {
    let parameters = parameters_for(&GEOMETRY_PRIMARY);
    let mut history = third_run_history_start(&GEOMETRY_PRIMARY, 1).expect("P(1, ·)");
    third_run_close(&mut history, &parameters, Closing::ProcessExit).expect("关闭 C");
    let geometry = independent_geometry(&history.pool).expect("几何");
    let hidden = history.tip();
    let before_hidden = *history.timeline.iter().rev().nth(1).expect("前一条");
    let position = hidden_root_position(&history.pool, hidden).expect("被藏那条根在环里");
    let untouched = hidden_root_destination(
        &position,
        &history.content_number_by_root,
        history.last_confirmed_content_number(),
        &history.pool,
        None,
    );
    assert_eq!(untouched, HiddenRootDestination::LeftAsItWas);
    let (hidden_device, hidden_offset) =
        root_slot_of(&history.pool, &geometry, hidden).expect("s_h");
    let (before_device, before_offset) =
        root_slot_of(&history.pool, &geometry, before_hidden).expect("前一条的槽");
    let length = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    let copied = history
        .pool
        .read(before_device.0, before_offset, length)
        .expect("读得出");
    let mut devices = devices_from_pool(&history.pool, IMAGE_BYTES);
    devices[usize::try_from(hidden_device.0).expect("盘号")]
        .1
        .write_at(
            DeviceOffsetInBytes(hidden_offset),
            &copied,
            WriteDurability::Plain,
        )
        .expect("内存盘写不报错");
    let overwritten_pool = memory_pool_of(&devices, IMAGE_BYTES);
    let overwritten = hidden_root_destination(
        &position,
        &history.content_number_by_root,
        history.last_confirmed_content_number(),
        &overwritten_pool,
        None,
    );
    assert_eq!(overwritten, HiddenRootDestination::RootSlotOverwritten);
    let chosen = hidden_root_destination(
        &position,
        &history.content_number_by_root,
        history.last_confirmed_content_number(),
        &history.pool,
        Some(hidden),
    );
    assert_eq!(chosen, HiddenRootDestination::ChosenOrApplied);
}

/// 条件式句子前件不成立记 vacuous，不算打不中；总判定按 fail > not_constructible > pass。
#[test]
fn conditional_sentences_are_vacuous_when_the_antecedent_does_not_hold() {
    assert_eq!(
        SentenceVerdict::conditional(false, false),
        SentenceVerdict::Vacuous
    );
    assert_eq!(
        SentenceVerdict::conditional(true, false),
        SentenceVerdict::Fail
    );
    let sentences = vec![
        sentence(
            SentenceTag::Measured,
            "first",
            SentenceVerdict::Vacuous,
            "V1",
        ),
        sentence(
            SentenceTag::ArmDefinition,
            "second",
            SentenceVerdict::NotConstructible,
            "F9",
        ),
    ];
    assert_eq!(overall_verdict(&sentences), "not_constructible");
}
