use super::*;

fn concurrent_numbers(arm: ConcurrentArm, repeat: u32, latencies: &[f64]) -> ConcurrentRunNumbers {
    ConcurrentRunNumbers {
        arm,
        repeat,
        latencies: latencies.to_vec(),
        publishes_under_the_lock: latencies.iter().map(|latency| latency / 2.0).collect(),
    }
}

/// 量 F：每条臂几次跑合起来一份分布、每次跑各自的中位与第 99 百分位；GPU 点名一张卡臂减 CPU 臂按重复成对（钉绝对值）。
#[test]
fn the_concurrent_tier_judgement_pools_the_runs_and_pairs_the_gpu_arm_against_the_cpu_arm() {
    let numbers = [
        concurrent_numbers(ConcurrentArm::Cpu, 1, &[10.0, 30.0]),
        concurrent_numbers(ConcurrentArm::Cpu, 0, &[20.0, 40.0]),
        concurrent_numbers(
            ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
            0,
            &[25.0, 45.0],
        ),
        concurrent_numbers(
            ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
            1,
            &[5.0, 50.0],
        ),
    ];
    let lines = judgement_of_a_concurrent_tier(
        UnitAreaTier::FourGibibytes,
        Duration::from_millis(50),
        &numbers,
        &[
            ConcurrentArm::Cpu,
            ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
        ],
    );
    let names: Vec<&str> = lines.iter().map(|(name, _fields)| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "latency_distribution",
            "latency_distribution",
            "latency_comparison",
            "latency_comparison"
        ]
    );
    assert!(lines[0].1.contains("arm=cpu "));
    assert!(lines[0].1.contains(
        "latency_nanoseconds_count=4 latency_nanoseconds_median=25 latency_nanoseconds_ninety_ninth_percentile=40 latency_nanoseconds_maximum=40"
    ));
    assert!(lines[0]
        .1
        .contains("publish_under_the_lock_nanoseconds_maximum=20"));
    assert!(lines[0]
        .1
        .contains("median_of_each_run=30,20 ninety_ninth_percentile_of_each_run=40,30"));
    assert!(lines[2].1.contains(
        "statistic=ninety_ninth_percentile gpu_one_card_minus_cpu_nanoseconds=5,20 sign=positive"
    ));
    assert!(lines[3]
        .1
        .contains("statistic=maximum gpu_one_card_minus_cpu_nanoseconds=5,20 sign=positive"));
}

/// 并发跑产物头一行写 `gc_thread_yields_after_unlock=true`（登记第十二节修订第 7 条：GC 线程放锁之后让一下）。
#[test]
fn the_head_line_of_a_concurrent_product_says_the_gc_thread_yields_after_unlock() {
    let fields = command_fields_of_a_concurrent_command(&ConcurrentCommand {
        tier: UnitAreaTier::FourGibibytes,
        foreground_interval: Duration::from_millis(50),
        arms_requested: vec![ConcurrentArm::Cpu],
        product: PathBuf::from("concurrent.out"),
    });
    assert!(
        fields
            .split(' ')
            .any(|field| field == "gc_thread_yields_after_unlock=true"),
        "{fields}"
    );
    assert!(fields.starts_with("command=concurrent reading=occupancy tier=4GiB "));
}
