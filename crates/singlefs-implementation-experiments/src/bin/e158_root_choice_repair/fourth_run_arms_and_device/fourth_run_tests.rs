use super::*;

fn read_failure_at(offset: u64) -> ThirdRunFault {
    ThirdRunFault {
        device: DeviceIdentity(0),
        offset,
        length: 4096,
        form: UnreadableForm::ReadFails,
        duration: FaultDuration::WholeCall,
        label: format!("probe({offset})"),
    }
}

/// 臂名单：十八条，十二条主族；每条臂的候选、N 与 R 照登记 5.2 与主 agent 认定第 1 项。
#[test]
fn the_arm_table_follows_the_registration() {
    assert_eq!(FOURTH_RUN_ARMS.len(), 18);
    let narrow = fourth_run_arm_named("yi-narrow-cfg-carry").expect("乙-窄读-配置续");
    assert_eq!(narrow.candidate, FourthRunCandidate::YiNarrow);
    assert_eq!(narrow.witness, FourthRunWitness::SystemConfigurationCarried);
    assert_eq!(narrow.reread_rounds, 1);
    assert_eq!(
        fourth_run_arm_named("yi-slot-three-rounds")
            .expect("R = 3")
            .reread_rounds,
        3
    );
    assert_eq!(
        fourth_run_arm_named("bing-cfg")
            .expect("丙")
            .refusal_member(),
        "R3RebuildFromRecordsStoppedBelowTheWitness"
    );
    assert_eq!(
        fourth_run_arm_named("yi-narrow-slot")
            .expect("乙-窄读-槽")
            .refusal_member(),
        "R3UnreadableRootRingSlot"
    );
    assert_eq!(
        fourth_run_arm_named("today")
            .expect("今天")
            .refusal_member(),
        "none"
    );
}

/// 时长表（登记 5.3、8.2）：乙 W、O(1 只)、T(δ)、T(2δ)；R = 3 的乙 到 T(4δ)；今天 T(δ) 只跑不断那一格；H1e 的乙 W、T(δ)。
#[test]
fn the_durations_follow_the_registration() {
    let yi = fourth_run_arm_named("yi-cfg").expect("乙");
    let names: Vec<String> = fourth_run_held_once_durations(yi)
        .iter()
        .map(|(duration, _)| duration.name())
        .collect();
    assert_eq!(names, vec!["W", "O1only", "T1", "T2"]);
    let three = fourth_run_arm_named("yi-narrow-cfg-three-rounds").expect("R = 3");
    assert_eq!(fourth_run_held_once_durations(three).len(), 2 + 4);
    assert_eq!(fourth_run_tail_durations(three).len(), 1 + 4);
    let today = fourth_run_arm_named("today").expect("今天");
    assert_eq!(
        fourth_run_held_once_durations(today)[2],
        (FaultDuration::UntilVirtualTime(1), true)
    );
    assert_eq!(
        fourth_run_tail_durations(yi),
        vec![FaultDuration::WholeCall, FaultDuration::UntilVirtualTime(1)]
    );
}

/// 标记前后的读（登记 5.0 第三条、5.4 PC-多读 (iii)；变异 M14 第 4 次跑那一份）：三次读坏、两次读成都数，
/// 第一个标记记下那一刻的读数与已拦下过的落点个数，之后读回全零的另数。
#[test]
fn reads_before_and_after_the_first_wait_count_failed_reads_too() {
    let pool = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
    let state = Rc::new(RefCell::new(ThirdRunDeviceState::new(
        vec![read_failure_at(0)],
        None,
        None,
    )));
    let marker = Rc::new(RefCell::new(FourthRunReadMarker::default()));
    let (identity, inner) = devices_from_pool(&pool, IMAGE_BYTES).remove(0);
    let device = FourthRunDevice {
        device: ThirdRunDevice {
            identity,
            inner,
            state: Rc::clone(&state),
        },
        marker: Rc::clone(&marker),
    };
    let mut buffer = vec![0u8; 4096];
    for _ in 0..3 {
        assert!(device.read_at(DeviceOffsetInBytes(0), &mut buffer).is_err());
    }
    mark_the_first_wait(&marker, &state);
    for _ in 0..2 {
        assert!(device
            .read_at(DeviceOffsetInBytes(1 << 20), &mut buffer)
            .is_ok());
    }
    mark_the_first_wait(&marker, &state);
    let marker = marker.borrow();
    assert_eq!(marker.read_calls, 5);
    assert_eq!(marker.reads_before_the_first_wait, Some(3));
    assert_eq!(marker.faults_intercepted_before_the_first_wait, Some(1));
    assert_eq!(marker.all_zero_reads_after_the_first_wait, 2);
    assert_eq!(state.borrow().read_calls, 5);
}
