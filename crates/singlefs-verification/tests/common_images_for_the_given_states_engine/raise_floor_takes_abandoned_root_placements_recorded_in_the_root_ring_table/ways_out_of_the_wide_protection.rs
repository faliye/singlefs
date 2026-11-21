//! 推环、要出路与核出路的帮手：推一次环、在拷贝上推完再发同一个改动、卸载再挂与推环两条出路的说法、被宽护卡住的那一份报。

use super::*;

/// 推一次环（会话的推环入口），做成时把推出来的每条根记成现行内容。
pub fn push_the_root_ring_of<Device: DeviceOverASparseImage>(
    pool: &mut PoolUnderTest<Device>,
) -> Result<RootRingPushed, RootRingPushRefused> {
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_last_ring_slot_it_reaches_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    if let Ok(pushed) = &pushed {
        for version in &pushed.versions_published {
            pool.note_root_of_the_current_content(&version.root);
        }
    }
    pushed
}

/// 每块盘隔离的槽数（按分配器里盘的次序）。
pub fn isolated_slots_on_each_device_of(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64)> {
    allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect()
}

/// 推环前后每块盘隔离的槽数少了几个（推完多了就当场判红：推环只写掉 C 那一圈的槽、放隔离）。
pub fn isolated_slots_released_between(
    before: &[(DeviceIdentity, u64)],
    after: &[(DeviceIdentity, u64)],
) -> Vec<(DeviceIdentity, u64)> {
    before
        .iter()
        .zip(after)
        .map(|((device, isolated_before), (device_after, isolated_after))| {
            assert_eq!(device, device_after, "前后数的是同一个分配器：盘的次序不变");
            (
                *device,
                isolated_before.checked_sub(*isolated_after).unwrap_or_else(|| {
                    panic!("盘 {} 推完隔离的槽数（{isolated_after}）不多于推之前（{isolated_before}）", device.0)
                }),
            )
        })
        .collect()
}

/// 要出路（会话的 `ways_out_of_the_wide_protection`，盘表交这一刻池里的那一份）；会话不在被拒的那一版上就当场判红（`why` 只用在断言消息里）。
pub fn ways_out_of<Device: DeviceOverASparseImage>(
    pool: &PoolUnderTest<Device>,
    stuck: &StuckBehindTheWideProtection,
    why: &str,
) -> WaysOutOfTheWideProtection {
    pool.session()
        .ways_out_of_the_wide_protection(&pool.devices, stuck)
        .unwrap_or_else(|refusal| {
            panic!("{why}：会话还在被拒的那一版上，要出路做得成：{refusal:?}")
        })
}

/// 照出路二真走一遍（这一刻盘面与会话的一份拷贝上推环、再发同一个改动 `change`），交回再发被拒的那一份；推环被拒或再发做成就当场判红。
pub fn refusal_of_the_same_change_after_pushing_the_root_ring_on_a_copy_of(
    pool: &PoolUnderTest<SparseBlockDevice>,
    change: UserChange<'_>,
    why: &str,
) -> UserChangeRefused {
    let (mut copy, _) = pool_failing_one_changing_call_copied_from(pool, None);
    push_the_root_ring_of(&mut copy)
        .unwrap_or_else(|push_refusal| panic!("{why}：照出路二推环做得成：{push_refusal:?}"));
    let mut session = copy.session.take().expect("挂着的会话");
    match session.publish_user_change(&mut copy.devices, change) {
        Ok(published) => panic!(
            "{why}：预演说推完之后这次改动照旧被拒，照出路二推完再发却做成了：{:?}",
            published.versions_published.len()
        ),
        Err(refusal) => refusal,
    }
}

/// 预演说推完照旧被拒时（`ChangeStillRefusedAfterThePush`），预演交回的拒绝与照出路二真推完、再发同一个改动交回的拒绝 `Debug` 逐字相同
/// （攻方 `research/prompts/m3-batch10z-tu-code-r1-opus-model/` 的 `atk_follow_way_two` 那一判 `dbg_same`：只进内存的盘读不回自己写过的块、
/// 写进内存时块号错一位，列不列推环不变，变的是这一份拒绝）。
pub fn assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
    pool: &PoolUnderTest<SparseBlockDevice>,
    pushing_the_root_ring: &RootRingPushRehearsal,
    change: UserChange<'_>,
    why: &str,
) {
    let RootRingPushRehearsal::ChangeStillRefusedAfterThePush {
        refusal: rehearsed_refusal,
        ..
    } = pushing_the_root_ring
    else {
        panic!("{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}");
    };
    let refusal_after_the_push =
        refusal_of_the_same_change_after_pushing_the_root_ring_on_a_copy_of(pool, change, why);
    assert_eq!(
        format!("{rehearsed_refusal:?}"),
        format!("{refusal_after_the_push:?}"),
        "{why}：预演交回的拒绝与推环之后真再发同一个改动交回的拒绝逐字相同"
    );
}

/// 出路里推环那一条预演的结局是「推完放得下」时，交回预演里每块盘放出来的隔离槽数；别的结局交 `None`。
pub fn released_by_the_push_if_the_change_is_admitted_after_it(
    pushing_the_root_ring: &RootRingPushRehearsal,
) -> Option<Vec<(DeviceIdentity, u64)>> {
    match pushing_the_root_ring {
        RootRingPushRehearsal::ChangeAdmittedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
        } => Some(isolated_slots_released_by_the_push_on_each_device.clone()),
        RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
        | RootRingPushRehearsal::PushRefused(_) => None,
    }
}

/// 卸载重挂那一条出路在 R 那一槽记着有效根时：点名 R 那一槽、带 R 的 txg（再挂时它按新实例的实例表判成被抛弃；它的账读不出若是盘上永久坏，
/// 卸载之后再挂会被拒、会话已断开——这几句由调用方照这两个数写）。
pub fn assert_the_way_out_by_unmounting_names_the_recorded_publish_abandoned_and_warns_of_a_permanently_unreadable_account(
    way_out_by_unmounting: &WayOutOfTheWideProtection,
    stuck: &StuckBehindTheWideProtection,
    checkpoint_txg_of_the_root_in_that_ring_slot: CheckpointTxg,
    why: &str,
) {
    assert_eq!(
        *way_out_by_unmounting,
        WayOutOfTheWideProtection::UnmountThenMountWritableAgain {
            ring_slot_held_by_the_wide_protection: stuck.last_ring_slot_the_push_reaches,
            checkpoint_txg_of_the_root_in_that_ring_slot: Some(
                checkpoint_txg_of_the_root_in_that_ring_slot
            ),
        },
        "{why}：头一条出路是卸载重挂，点名 R 那一槽、带 R 的 txg"
    );
}

/// 卸载重挂那一条出路在 R 那一槽没有读得出的有效根时：点名那一槽、不带 txg（再挂之后 C 那一圈不再有这一槽，不说 R 判成被抛弃；
/// 攻方 `research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md` 第一节末：实测卸载再挂之后 C 那一圈直接没了）。
pub fn assert_the_way_out_by_unmounting_says_the_ring_slot_of_the_recorded_publish_holds_no_readable_root(
    way_out_by_unmounting: &WayOutOfTheWideProtection,
    stuck: &StuckBehindTheWideProtection,
    why: &str,
) {
    assert_eq!(
        *way_out_by_unmounting,
        WayOutOfTheWideProtection::UnmountThenMountWritableAgain {
            ring_slot_held_by_the_wide_protection: stuck.last_ring_slot_the_push_reaches,
            checkpoint_txg_of_the_root_in_that_ring_slot: None,
        },
        "{why}：头一条出路是卸载重挂，点名 R 那一槽、那一槽没有读得出的有效根"
    );
}

/// 推环那一条出路：R 那一槽、推几次（就是报里写的那个数，写掉根环里同样多槽）、预演里每块盘放出来的隔离槽数（与预演的数相同）、
/// 推完之后是再发这次改动还是只剩走空（`after_the_push`）。
pub fn assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_the_recorded_publish_and_the_released_slots(
    stuck: &StuckBehindTheWideProtection,
    way_out_by_the_push: &WayOutOfTheWideProtection,
    released_on_each_device: &[(DeviceIdentity, u64)],
    after_the_push: WhatIsLeftAfterThePush,
    why: &str,
) {
    assert_eq!(
        *way_out_by_the_push,
        WayOutOfTheWideProtection::PushTheRootRingUpToTheRingSlotHeldByTheWideProtection {
            ring_slot_held_by_the_wide_protection: stuck.last_ring_slot_the_push_reaches,
            empty_publishes_until_that_ring_slot_is_written: stuck
                .empty_publishes_until_the_last_ring_slot_the_push_reaches_is_written,
            isolated_slots_released_on_each_device: released_on_each_device.to_vec(),
            after_the_push,
        },
        "{why}：第二条出路是推环到 R 那一槽，推的次数与报里的相同、放出来的隔离槽数与预演的相同"
    );
}

/// 一次写被拒、报「被宽护卡住」：这次调用里推了几串抬 F、发成了几版，与报的那一份。
pub struct WriteStuckBehindTheWideProtection {
    pub floor_raises_pushed: usize,
    pub versions_published: usize,
    pub stuck: StuckBehindTheWideProtection,
}

/// 被拒是推满仍不够、停在上限、报「被宽护卡住」（`why` 只用在断言消息里）。
pub fn stuck_behind_the_wide_protection_reported_by(
    refusal: UserChangeRefused,
    why: &str,
) -> WriteStuckBehindTheWideProtection {
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("{why}：被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
        ),
        "{why}：推抬 F 停在上限：{:?}",
        no_space.stop
    );
    let NoSpaceAfterRaisingTheFloor {
        floor_raises,
        versions_published,
        wide_protection_at_the_ceiling,
        ..
    } = *no_space;
    let WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) =
        wide_protection_at_the_ceiling
    else {
        panic!("{why}：F 在上限、C 那一圈还隔离着槽：报「被宽护卡住」，而报的是 {wide_protection_at_the_ceiling:?}");
    };
    WriteStuckBehindTheWideProtection {
        floor_raises_pushed: floor_raises.len(),
        versions_published: versions_published.len(),
        stuck: *stuck,
    }
}
