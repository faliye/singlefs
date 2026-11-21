use super::*;
use crate::fourth_run_segment_two::tests::{row_forty_three, shortest, today};

#[test]
fn injected_faults_and_persisted_subsets_round_trip_through_the_result_line() {
    for fault in [
        InjectedFault::WriteFails,
        InjectedFault::WriteIsSwallowed,
        InjectedFault::ReadFails,
        InjectedFault::BarrierFails,
        InjectedFault::BarrierIsSwallowed,
        InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: singlefs_verification::fault_injection::FlippedBit {
                byte_index: 1_234_567,
                bit_index: 5,
            },
        },
    ] {
        assert_eq!(
            parse_injected_fault(&render_injected_fault(fault)),
            Some(fault)
        );
    }
    let persisted = vec![true, false, false, true];
    assert_eq!(render_persisted(&persisted), "1001");
    assert_eq!(parse_persisted("1001"), persisted);
}

/// 装置执行器在今天那一臂上与 harness 执行器逐步同一份镜像、同样多的写与屏障、读不少于 harness（它的稀疏盘答日志环记录槽提示、少读没写过的槽）
/// （S9 末句、S7 的前提；变异 M35 让藏起来的那几段不再读回全 0）。
#[test]
fn device_executor_agrees_with_the_harness_executor_on_the_two_written_histories() {
    for segment in [shortest(), row_forty_three()] {
        let (executor, _) = run_the_main_branch(today(), &segment, PoolCheckerOnEveryStep::Skipped);
        let (compared, mismatches) =
            executor_agreement(&executor, &harness_trace(&segment.history));
        assert!(compared > 0, "{}：harness 一步都没观察到", segment.label);
        assert!(mismatches.is_empty(), "{}：{mismatches:?}", segment.label);
    }
}

/// 比对真的看调用数：把 harness 那一份的写数改大一次，就对不上（变异 M34）；把 harness 那一份的读数改到比装置的多，也对不上。
#[test]
fn executor_agreement_catches_a_different_call_count() {
    let segment = shortest();
    let (executor, _) = run_the_main_branch(today(), &segment, PoolCheckerOnEveryStep::Skipped);
    let mut trace = harness_trace(&segment.history);
    trace[1].2.writes += 1;
    let (_, mismatches) = executor_agreement(&executor, &trace);
    assert_eq!(mismatches.len(), 1, "写数多一次要对不上");
    let mut trace_reading_more_than_the_device = harness_trace(&segment.history);
    trace_reading_more_than_the_device[1].2.reads = u64::MAX;
    let (_, read_mismatches) = executor_agreement(&executor, &trace_reading_more_than_the_device);
    assert_eq!(read_mismatches.len(), 1, "harness 读得比装置多要对不上");
}

/// 崩溃镜像：最后一段整段持久 = 全部写都落了盘 = 执行器此刻的镜像；全不持久 = 前面各段（变异 M33 把挑法反过来）。
#[test]
fn crash_image_applies_exactly_the_persisted_writes_of_the_segment() {
    let segment = shortest();
    let mut executor = RandomExecutor::start(
        today(),
        segment.history.starting_point,
        SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry()),
        PoolCheckerOnEveryStep::Skipped,
    );
    executor.run_the_operations(&segment.history);
    let stream = segmented_stream_of(&executor);
    let last = stream.segments.len() - 1;
    let every_write = vec![true; stream.segments[last].len()];
    assert_eq!(
        image_fingerprint(&stream.crash_image(last, &every_write)),
        image_fingerprint(&executor.image())
    );
    let no_write = vec![false; stream.segments[last].len()];
    let mut earlier = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        history_device_bytes(),
    );
    earlier.apply_writes(&stream.writes[..stream.segments[last][0]]);
    assert_eq!(
        image_fingerprint(&stream.crash_image(last, &no_write)),
        image_fingerprint(&earlier)
    );
}

/// seg2b 修的装置记账撞键：崩溃分支那次可写挂载写出的根，读回按这次挂载承接的那一版判，不按主路径在同一个 (实例, txg) 上记的那一版判。
/// 主路径的账在分支落到的那个键上改记成另一版（照 -配置 各臂实七-乙 `crash0`：主路径第 8 步那次挂载在同一个键上发布过内容号 1），
/// 读回仍是最后确认那一版、内容号 0（变异 M39 让读回改回拿主路径的账，M40 让分支写出的根一条都不改记）。
/// 崩溃点是第 0 步里暖机第一条根槽写那一段（1 写、没落）：段号 9。C656 两趟写序把 mkfs 开头那一段切成 2 写与 12 写两段、
/// 取号写两轮（D22（单元原子性怎么合成） 已定项 8）让取号自成两段，这两样之前同一条根槽写是段号 7。
#[test]
fn crash_branch_read_back_after_its_mount_goes_by_the_roots_that_branch_wrote() {
    let segment = seventh_crash_history();
    let (mut main, _) = run_the_main_branch(today(), &segment, PoolCheckerOnEveryStep::Skipped);
    let stream = segmented_stream_of(&main);
    let point = CrashPointKey {
        index: 0,
        order_of_the_first_write: 0,
        segment_within_the_step: 9,
        persisted_within_the_segment: vec![false],
    };
    let after_mount_read_back_of = |branch: &BranchResult| -> RandomReadBack {
        let (cell, _) = branch.cells.first().expect("一个崩溃状态一格");
        cell.read_backs
            .iter()
            .find(|(name, _)| name == "after_mount_read_back")
            .map(|(_, read_back)| read_back.clone())
            .expect("挂载之后读回了一次")
    };
    let untouched = after_mount_read_back_of(&run_a_crash_branch(today(), &main, &stream, &point));
    assert_eq!(untouched.literal, "last_confirmed");
    assert_eq!(untouched.content_number, Some(0));
    let landing = untouched.root.expect("读回落到一条根");
    let crash_image = stream.crash_image(
        stream
            .find(
                point.order_of_the_first_write,
                point.segment_within_the_step,
            )
            .expect("今天那一臂上有这一段"),
        &point.persisted_within_the_segment,
    );
    let roots_in_the_crash_image = independent_geometry(&crash_image)
        .map(|geometry| readable_roots_independent(&crash_image, &geometry))
        .expect("崩溃镜像有几何");
    assert!(
        !roots_in_the_crash_image.contains(&landing),
        "落到的根 {landing:?} 是分支那次挂载写出的，崩溃镜像里没有"
    );
    let another_version = main
        .ledger
        .new_content_number(Some(b"another timeline".to_vec()));
    main.ledger.carry(landing, another_version);
    let collided = after_mount_read_back_of(&run_a_crash_branch(today(), &main, &stream, &point));
    assert_eq!(collided.literal, "last_confirmed");
    assert_eq!(collided.content_number, Some(0));
}

/// 注入点的映射：「步内第 k 次」换成这一臂整池第（那一步之前的调用数 + k）次（变异 M29 差一）。
/// 步取这一臂主路径上第一个至少有两次屏障的那一步（C554 乙合进 `crates` 之后，最短复现第 3 步的崩溃恢复拒了、那一步的屏障不够两次）。
#[test]
fn fault_point_fires_at_the_arm_ordinal_of_the_named_call_within_the_step() {
    let segment = shortest();
    let (main, _) = run_the_main_branch(today(), &segment, PoolCheckerOnEveryStep::Skipped);
    let order = (1..main.calls_after_step.len())
        .find(|order| {
            main.calls_after_step[*order].barriers >= main.calls_after_step[*order - 1].barriers + 2
        })
        .expect("最短复现的主路径上有一步至少两次屏障");
    let point = FaultPointKey {
        index: 0,
        fault: InjectedFault::BarrierFails,
        order,
        ordinal_within_the_step: 2,
    };
    let branch = run_a_fault_branch(today(), &segment, &main, &point);
    let barriers_before = main.calls_after_step[order - 1].barriers;
    assert_eq!(branch.fired_call_ordinal, Some(barriers_before + 2));
}
