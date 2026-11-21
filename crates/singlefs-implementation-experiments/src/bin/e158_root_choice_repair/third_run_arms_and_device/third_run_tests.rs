use super::*;

fn whole_call_read_failure_at(offset: u64, length: u64) -> ThirdRunFault {
    ThirdRunFault {
        device: DeviceIdentity(0),
        offset,
        length,
        form: UnreadableForm::ReadFails,
        duration: FaultDuration::WholeCall,
        label: format!("probe({offset})"),
    }
}

/// 「读法写死」表「时长」一行：W 每次都坏；T(τ) 坏到虚拟时钟走到 τ；O(m 只) 只坏第 m 次；O(m 起) 第 m 次起都坏（变异 M5、M6 要它红）。
#[test]
fn fault_durations_follow_the_registered_table() {
    assert!(FaultDuration::WholeCall.is_active(1, 0));
    assert!(FaultDuration::WholeCall.is_active(7, 5));
    assert!(FaultDuration::UntilVirtualTime(1).is_active(1, 0));
    assert!(!FaultDuration::UntilVirtualTime(1).is_active(2, 1));
    assert!(FaultDuration::UntilVirtualTime(2).is_active(2, 1));
    assert!(FaultDuration::OnlyTheNthRead(1).is_active(1, 0));
    assert!(!FaultDuration::OnlyTheNthRead(1).is_active(2, 0));
    assert!(!FaultDuration::OnlyTheNthRead(2).is_active(1, 0));
    assert!(!FaultDuration::FromTheNthRead(2).is_active(1, 0));
    assert!(FaultDuration::FromTheNthRead(2).is_active(3, 0));
    assert!(FaultDuration::OnlyTheNthRead(1).is_active_on_the_first_read());
    assert!(!FaultDuration::OnlyTheNthRead(2).is_active_on_the_first_read());
}

/// 虚拟时钟只由等待钩子推进：推一格之后 T(1) 的落点读得出。
#[test]
fn fault_until_one_interval_clears_after_one_wait() {
    let pool = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
    let fault = ThirdRunFault {
        duration: FaultDuration::UntilVirtualTime(1),
        ..whole_call_read_failure_at(0, 4096)
    };
    let call = third_run_call(&pool, &[fault], None, None, |devices| {
        let first = singlefs_core::recovery::PoolReader::read(
            &*devices,
            DeviceIdentity(0),
            DeviceOffsetInBytes(0),
            4096,
        );
        third_run_arm_hooks::advance_one_interval(THIRD_RUN_VIRTUAL_INTERVAL);
        let second = singlefs_core::recovery::PoolReader::read(
            &*devices,
            DeviceIdentity(0),
            DeviceOffsetInBytes(0),
            4096,
        );
        (first.is_some(), second.is_some())
    });
    assert_eq!(call.outcome, (false, true));
    assert_eq!(call.intercepted_reads, vec![1]);
}

/// Q5 的读数：读坏的读也数进读调用次数与读字节数（变异 M14 要它红）。
#[test]
fn failed_reads_are_counted_as_reads() {
    let pool = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
    let call = third_run_call(
        &pool,
        &[whole_call_read_failure_at(0, 4096)],
        None,
        None,
        |devices| {
            singlefs_core::recovery::PoolReader::read(
                &*devices,
                DeviceIdentity(0),
                DeviceOffsetInBytes(0),
                4096,
            )
            .is_some()
        },
    );
    assert!(!call.outcome);
    assert_eq!(call.read_calls, 1);
    assert_eq!(call.read_bytes, 4096);
}

/// V2：点了名而这次调用一次都没读到的落点要报出来（变异 M13 要它红）。
#[test]
fn named_target_never_read_is_reported_as_not_intercepted() {
    let pool = memory_pool_of(&new_devices(IMAGE_BYTES), IMAGE_BYTES);
    let faults = [
        whole_call_read_failure_at(0, 4096),
        whole_call_read_failure_at(1 << 20, 4096),
    ];
    let call = third_run_call(&pool, &faults, None, None, |devices| {
        singlefs_core::recovery::PoolReader::read(
            &*devices,
            DeviceIdentity(0),
            DeviceOffsetInBytes(0),
            4096,
        )
        .is_some()
    });
    assert_eq!(call.unintercepted(&faults), vec![faults[1].label.clone()]);
}

/// K3 / K4 按这次调用设备一层写了几次分，不按错误成员分（变异 M10 要它红）。
#[test]
fn refusal_after_one_write_is_classed_by_its_write_count() {
    let refused: Result<Mounted, singlefs_core::mount::MountError> =
        Err(singlefs_core::mount::MountError::InstanceTableMalformed);
    assert_eq!(third_run_outcome_class(&refused, 1), "K4");
    assert_eq!(third_run_outcome_class(&refused, 0), "K3");
}
