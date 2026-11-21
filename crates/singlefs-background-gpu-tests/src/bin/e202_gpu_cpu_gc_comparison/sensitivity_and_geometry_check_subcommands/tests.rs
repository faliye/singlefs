use super::*;

/// 登记第八节两点：中档、批大小 16 的交替跑（CPU 臂与 GPU 点名一张卡臂），前台间隔 500 毫秒的并发跑（三条臂）；命令行只认这两个取值。
#[test]
fn the_sensitivity_points_are_the_middle_tier_with_sixteen_unit_batches_and_five_hundred_millisecond_intervals(
) {
    let product = Path::new("sensitivity.out");
    let lockstep = sensitivity_lockstep_command(product, 24);
    assert_eq!(lockstep.tier, UnitAreaTier::FourGibibytes);
    assert_eq!(lockstep.batch_units, 16);
    assert_eq!(lockstep.user_changes, 24);
    assert_eq!(
        lockstep
            .arms_requested
            .iter()
            .map(|arm| arm.name())
            .collect::<Vec<String>>(),
        ["cpu", "gpu_one_card"]
    );
    assert_eq!(
        sensitivity_concurrent_command(product),
        ConcurrentCommand {
            tier: UnitAreaTier::FourGibibytes,
            foreground_interval: Duration::from_millis(500),
            arms_requested: vec![
                ConcurrentArm::Cpu,
                ConcurrentArm::GpuOnTheCardWithTheLowestPciAddress,
                ConcurrentArm::WithoutCompaction
            ],
            product: PathBuf::from("sensitivity.out"),
        }
    );
    assert!(matches!(
        command_of_the_arguments(&arguments_of(&[
            "sensitivity",
            "--batch-units",
            "16",
            "--foreground-interval-milliseconds",
            "500",
            "--product",
            "sensitivity.out"
        ])),
        Ok(CommandOfTheApparatus::Sensitivity {
            user_changes: 24,
            ..
        })
    ));
    assert!(command_of_the_arguments(&arguments_of(&[
        "sensitivity",
        "--batch-units",
        "4",
        "--product",
        "sensitivity.out"
    ]))
    .is_err());
    match command_of_the_arguments(&arguments_of(&[
        "concurrent",
        "--reading",
        "occupancy",
        "--unit-area",
        "4GiB",
        "--product",
        "concurrent.out",
    ])) {
        Ok(CommandOfTheApparatus::Concurrent(command)) => {
            assert_eq!(command.foreground_interval, Duration::from_millis(50));
            assert_eq!(command.arms_requested, CONCURRENT_ARMS.to_vec());
        }
        Ok(_) | Err(_) => panic!("concurrent 认得出、间隔默认 50 毫秒"),
    }
    assert!(command_of_the_arguments(&arguments_of(&[
        "concurrent",
        "--reading",
        "occupancy",
        "--unit-area",
        "4GiB",
        "--foreground-interval-milliseconds",
        "70",
        "--product",
        "concurrent.out",
    ]))
    .is_err());
}
