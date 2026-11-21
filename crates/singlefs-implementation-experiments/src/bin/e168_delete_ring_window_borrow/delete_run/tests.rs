//! 删除（5.1 第 4、5 步）：删除的计划、逐步发布、抬 F 与谷值钩子的单测。

use super::*;

fn record_a_fake_root(free_slots_of_each_device: [u64; 2]) {
    FAKE_FREE_SLOTS_RECORDED_AT_EACH_ROOT
        .with(|cell| cell.borrow_mut().push(free_slots_of_each_device.to_vec()));
}

#[test]
fn valley_hook_drops_the_roots_recorded_before_the_delete_and_keeps_those_recorded_after() {
    for free_slots in [[40, 40], [30, 30], [3, 3]] {
        record_a_fake_root(free_slots);
    }
    let valley_hook = ValleyHookDuringTheDelete::opened_at_the_start_of_the_delete(
        take_the_fake_free_slots_recorded_at_each_root,
    );
    assert_eq!(
        valley_hook.take_readings_since_the_last_take(),
        Vec::<Vec<u64>>::new(),
        "建池与此前各格记的三条根在开口时丢掉，第一步之前读到 0 条（2026-10-01 第一段停在 P2-S4、readings=4035 就是没丢）"
    );
    record_a_fake_root([2, 2]);
    assert_eq!(
        valley_hook.take_readings_since_the_last_take(),
        vec![vec![2, 2]],
        "开口之后记的根照取：删除各步之间多出来的读数仍会停机"
    );
    assert_eq!(
        valley_hook.take_readings_since_the_last_take(),
        Vec::<Vec<u64>>::new(),
        "取走之后清空"
    );
}

#[test]
fn valley_keeps_the_roots_recorded_on_the_session_allocator_and_drops_those_the_floor_raise_rehearsal_recorded_on_a_copy(
) {
    let free_on_both_devices = |free_slots: u64| vec![free_slots, free_slots];
    assert_eq!(
        hook_readings_on_the_allocator_of_the_session(
            vec![free_on_both_devices(75)],
            &FloorRaisesOfTheStep::none(),
            true
        ),
        Some(vec![free_on_both_devices(75)]),
        "没推抬 F、发成的一次：钩子读到的那一条就是会话分配器上的"
    );
    let refused_at_the_second_rehearsed_publish = FloorRaisesOfTheStep {
        publishes_of_each_landed_chain: Vec::new(),
        rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 1,
        stop_label: "floor_raise_refused_for_space_in_the_rehearsal_at_publish_2_of_2".to_owned(),
    };
    assert_eq!(
        hook_readings_on_the_allocator_of_the_session(
            vec![free_on_both_devices(3)],
            &refused_at_the_second_rehearsed_publish,
            false
        ),
        Some(Vec::new()),
        "r7 冒烟 P1-S16-K1-Lseq-Fend-N-B-N88-noreclaim 的 t = 7：抬 F 那一串在预演第 2 次被拒，钩子读到的 3 槽是拷贝上第 1 次之后的，\
             不进谷值（谷值退回发布之前的 9 槽，borrow 是 86 − 9 = 77，不是 86 − 3 = 83）"
    );
    let one_landed_chain_of_two = FloorRaisesOfTheStep {
        publishes_of_each_landed_chain: vec![2],
        rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 0,
        stop_label: "none".to_owned(),
    };
    assert_eq!(
        hook_readings_on_the_allocator_of_the_session(
            vec![
                free_on_both_devices(40),
                free_on_both_devices(34),
                free_on_both_devices(41),
                free_on_both_devices(35),
                free_on_both_devices(24),
            ],
            &one_landed_chain_of_two,
            true
        ),
        Some(vec![
            free_on_both_devices(41),
            free_on_both_devices(35),
            free_on_both_devices(24)
        ]),
        "落了盘的一串两次：丢掉预演在拷贝上的前两条，留下真发的两条与发成的那一次"
    );
    let landed_then_refused_in_the_rehearsal = FloorRaisesOfTheStep {
        publishes_of_each_landed_chain: vec![2],
        rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 1,
        stop_label: "floor_raise_refused_for_space_in_the_rehearsal_at_publish_2_of_2".to_owned(),
    };
    assert_eq!(
        hook_readings_on_the_allocator_of_the_session(
            vec![
                free_on_both_devices(20),
                free_on_both_devices(14),
                free_on_both_devices(21),
                free_on_both_devices(15),
                free_on_both_devices(9),
            ],
            &landed_then_refused_in_the_rehearsal,
            false
        ),
        Some(vec![free_on_both_devices(21), free_on_both_devices(15)]),
        "落了一串、再推的一串在预演里被拒：只留第一串真发的两条"
    );
    assert_eq!(
        hook_readings_on_the_allocator_of_the_session(
            vec![free_on_both_devices(3), free_on_both_devices(3)],
            &refused_at_the_second_rehearsed_publish,
            false
        ),
        None,
        "钩子多读了一条：这一套账对不上，交回 None（调用方停机）"
    );
    assert_eq!(
        hook_readings_on_the_allocator_of_the_session(
            Vec::new(),
            &FloorRaisesOfTheStep::none(),
            true
        ),
        None,
        "发成而钩子一条都没读到：对不上"
    );
}

#[test]
fn floor_raises_of_a_refused_step_count_the_rehearsal_roots_and_refuse_a_chain_that_landed_outside_the_refusal(
) {
    use singlefs_core::address::CheckpointTxg;
    use singlefs_core::mount::PublishSequenceFailed;
    use singlefs_core::write_accounting::WritesByStructureKind;
    let some_publish_error = || PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
        checkpoint_txg: CheckpointTxg(9),
        instance: InstanceGeneration(1),
    };
    assert_eq!(
        FloorRaisesOfTheStep::of_a_refused_step(
            &[],
            &FloorRaiseStop::FloorAlreadyAtTheCeiling {
                floor: CheckpointTxg(5),
                ceiling: CheckpointTxg(5)
            }
        ),
        Ok(FloorRaisesOfTheStep {
            publishes_of_each_landed_chain: Vec::new(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 0,
            stop_label: "floor_already_at_the_ceiling".to_owned(),
        })
    );
    assert_eq!(
        FloorRaisesOfTheStep::of_a_refused_step(
            &[],
            &FloorRaiseStop::FloorRaiseRefusedForSpace {
                refusal: Box::new(
                    MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                        refused_publish_in_the_sequence: 2,
                        publishes_in_the_sequence: 2,
                        cause: some_publish_error(),
                    }
                )
            }
        ),
        Ok(FloorRaisesOfTheStep {
            publishes_of_each_landed_chain: Vec::new(),
            rehearsal_root_recordings_of_the_chain_refused_in_the_rehearsal: 1,
            stop_label: "floor_raise_refused_for_space_in_the_rehearsal_at_publish_2_of_2"
                .to_owned(),
        }),
        "预演第 2 次被拒：拷贝上记过 1 条根"
    );
    let landed_one_then_failed = FloorRaiseStop::FloorRaiseRefusedForSpace {
        refusal: Box::new(MountError::RaiseFloorSequencePublishFailed(Box::new(
            PublishSequenceFailed {
                cause: some_publish_error(),
                writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
                writes_of_persisted_publishes: vec![WritesByStructureKind::NOTHING_WRITTEN],
                writes_of_failed_publishes: Vec::new(),
            },
        ))),
    };
    assert!(
        FloorRaisesOfTheStep::of_a_refused_step(&[], &landed_one_then_failed).is_err(),
        "真发途中失败：落了盘的那一次不在 floor_raises 里，交回 Err（调用方停机），不当成没推"
    );
}
