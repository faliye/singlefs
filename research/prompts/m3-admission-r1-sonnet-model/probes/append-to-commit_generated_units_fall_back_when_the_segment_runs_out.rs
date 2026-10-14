
/// 草稿探针（腿自己的副本，不入库）：C546 剩下那一格，写错之后扣住位还在不在。
#[test]
fn probe_c546_holds_left_in_the_process_after_a_write_failure_in_the_middle_of_the_sequence() {
    let mut counting_pool = five_overwrites_on_fault_injected_devices();
    let before = (counting_pool.allocator.devices[0].free_slots(), counting_pool.allocator.devices[0].unblocked_slots());
    let raised = raise_the_floor_to_four(&mut counting_pool).expect("不注入时这一串做成");
    let after_ok = (counting_pool.allocator.devices[0].free_slots(), counting_pool.allocator.devices[0].unblocked_slots());
    let first = raised.publishes[0].writes.total().write_calls;
    let sysconf = raised.system_configuration_writes_before_the_first_publish.total().write_calls;
    println!("name=c546-probe success before(free,unblocked)={before:?} after(free,unblocked)={after_ok:?} reclaimed={}", raised.reclaimed.len());
    let mut pool = five_overwrites_on_fault_injected_devices();
    pool.fault_plan.arm(FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, sysconf + first + 1));
    let refused = raise_the_floor_to_four(&mut pool);
    assert!(matches!(refused, Err(MountError::RaiseFloorSequencePublishFailed(_))));
    let d = &pool.allocator.devices[0];
    println!("name=c546-probe write_failed_after_one_persisted d0_free_slots={} d0_unblocked_slots={} d0_free_minus_unblocked={} d0_isolated={} frozen_publish_present={}",
        d.free_slots(), d.unblocked_slots(), i128::from(d.free_slots()) - i128::from(d.unblocked_slots()), d.isolated_slots(), pool.allocator.frozen_publish().is_some());
}
