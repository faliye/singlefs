//! 丢后台 GPU、量代价出错、切换时关卡，有提交还在跑的那张整张泄漏并记进进程里的那一份。

use super::*;

/// 丢外壳时，记着一次还在跑的提交（占 600 字节）的那张卡泄漏、另一张照常丢；泄漏了哪一张、占着多少交到调用方拿着的那一份，
/// 并记进进程里的那一份（新一次挂载这张卡先扣掉）。
#[test]
fn dropping_the_background_gpu_reports_the_card_leaked_and_notes_it_in_the_process() {
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
        vec![
            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
                .with_a_submission_running_past_its_wait(600),
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
        ],
        &memory_leaked,
    );
    let gpus_leaked = background_gpu.gpus_leaked_when_dropped();
    assert_eq!(gpus_leaked.gpus_leaked(), None, "还没丢");
    drop(background_gpu);
    assert_eq!(
        gpus_leaked.gpus_leaked(),
        Some(vec![GpuLeakedWhenDropped {
            device: card_identifier("0000:01:00.0"),
            bytes_held_on_the_device: GpuMemoryInBytes(600),
            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
        }]),
        "只泄漏还在跑的那一张"
    );
    assert_eq!(
        (
            memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
            memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0"))
        ),
        (GpuMemoryInBytes(600), GpuMemoryInBytes(0))
    );
}

/// 这个进程里之前几次挂载在「0000:01:00.0」上泄漏了 100 字节的那一份泄漏账。
fn memory_leaked_with_one_hundred_bytes_on_the_card() -> GpuMemoryLeakedInThisProcess {
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
        device: card_identifier("0000:01:00.0"),
        bytes_held_on_the_device: GpuMemoryInBytes(100),
        bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
    }]);
    memory_leaked
}

/// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于开卡：量代价出错时账上记着一次等超过上限、还在跑的提交（这一段占 600 字节，
/// 账从之前泄漏的 100 字节起）——这张卡不丢（替身的析构一次都没走），进程里的泄漏账在这张卡上加这一次的 600 字节、到 700，交回量代价那一次的错，原样。
#[test]
fn the_card_whose_cost_measurement_failed_with_a_submission_still_running_is_leaked_and_noted_in_the_process(
) {
    let card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 100)
        .with_a_submission_running_past_its_wait(600);
    let drops_counted = Arc::clone(&card.drops_counted);
    let memory_leaked = memory_leaked_with_one_hundred_bytes_on_the_card();
    let reported = failure_of_the_measurement_dropping_or_leaking_the_card(
        card,
        failure_of_the_cost_measurement_in_the_test(),
        &memory_leaked,
    );
    assert_eq!(
        reported,
        failure_of_the_cost_measurement_in_the_test(),
        "交回量代价那一次的错"
    );
    assert_eq!(
        drops_counted.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "提交还在跑：这张卡泄漏、不析构"
    );
    assert_eq!(
        memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
        GpuMemoryInBytes(100 + 600),
        "泄漏账加的恰好是这一次占着的 600 字节"
    );
}

/// 同一条的另一格：量代价出错而账上没有还在跑的提交——照常丢（替身析构一次），泄漏账不变，交回量代价那一次的错。
#[test]
fn the_card_whose_cost_measurement_failed_without_a_submission_still_running_is_dropped() {
    let card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 100);
    let drops_counted = Arc::clone(&card.drops_counted);
    let memory_leaked = memory_leaked_with_one_hundred_bytes_on_the_card();
    let reported = failure_of_the_measurement_dropping_or_leaking_the_card(
        card,
        failure_of_the_cost_measurement_in_the_test(),
        &memory_leaked,
    );
    assert_eq!(reported, failure_of_the_cost_measurement_in_the_test());
    assert_eq!(
        drops_counted.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "没有还在跑的提交：照常丢"
    );
    assert_eq!(
        memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
        GpuMemoryInBytes(100),
        "泄漏账不变"
    );
}

/// 切换关卡时有提交还在跑的卡照卸载同一个规矩泄漏，另累计进对外读得到的那一份：「0000:41:00.0」上记着一次等超过上限、还在跑的提交（占 600 字节），
/// 切换不再点它——切换关卡的泄漏报告里读得到它与 600 字节。
#[test]
fn the_card_leaked_when_closed_at_a_switch_is_read_from_the_leaks_of_the_switches() {
    let (mut background_gpu, _cards_handed_to_the_opening, _drops_of_the_card_at_forty_one) =
        background_gpu_with_both_cards_opened_at_the_mount(
            opening_that_always_opens,
            |card| card.with_a_submission_running_past_its_wait(600),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
    let leaks_of_the_switches = background_gpu.gpus_leaked_when_closed_at_a_switch();
    assert!(leaks_of_the_switches.gpus_leaked().is_empty());
    let only_the_card_at_one =
        switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
    background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
    assert_eq!(
        leaks_of_the_switches.gpus_leaked(),
        vec![GpuLeakedWhenDropped {
            device: card_identifier("0000:41:00.0"),
            bytes_held_on_the_device: GpuMemoryInBytes(600),
            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
        }]
    );
}

/// 外壳上：同一进程里两次挂载同时开着、都在同一张卡上各有一次等超过上限还在跑的提交（先开的 600 字节、后开的 300 字节，
/// 开卡那一刻都没算上对方的），两次都丢掉——进程里这张卡记 900（按次累加），新一次挂载这张卡的账从 900 起。
#[test]
fn two_mounts_alive_at_once_leaking_on_one_gpu_add_up_through_the_drops_of_the_background_gpu() {
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let card = card_identifier("0000:01:00.0");
    let (mount_leaking_six_hundred_bytes, _present_of_the_mount_leaking_six_hundred_bytes) =
        background_gpu_over_cards_on_the_cpu(
            vec![CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                None,
                memory_leaked.bytes_leaked_on(&card).0,
            )
            .with_a_submission_running_past_its_wait(600)],
            &memory_leaked,
        );
    let (mount_leaking_three_hundred_bytes, _present_of_the_mount_leaking_three_hundred_bytes) =
        background_gpu_over_cards_on_the_cpu(
            vec![CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                None,
                memory_leaked.bytes_leaked_on(&card).0,
            )
            .with_a_submission_running_past_its_wait(300)],
            &memory_leaked,
        );
    drop(mount_leaking_six_hundred_bytes);
    drop(mount_leaking_three_hundred_bytes);
    assert_eq!(
        memory_leaked.bytes_leaked_on(&card),
        GpuMemoryInBytes(900),
        "两次各泄漏 600、300 字节：记 900，不让后丢的盖掉先丢的"
    );
    assert_eq!(
        CardComputedOnTheCpuForTests::of_the_address(
            "0000:01:00.0",
            Some(1 << 20),
            memory_leaked.bytes_leaked_on(&card).0,
        )
        .budget
        .memory_account()
        .bytes_held(),
        GpuMemoryInBytes(900),
        "新一次挂载这张卡的账从 900 起"
    );
}

/// 同一条：一次挂载在一张卡上泄漏 600 字节、丢掉；同一张卡新一次挂载开卡那一刻账从 600 起，再有一次等超过上限还在跑的提交（300 字节），
/// 丢掉时那一刻占着 900——进程里只加这一次自己的 300，记 900，不把开卡时已算上的 600 再加一遍。
#[test]
fn the_later_mount_leaking_again_on_the_same_gpu_adds_only_its_own_bytes_in_the_process() {
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let card = card_identifier("0000:01:00.0");
    let (earlier_mount, _earlier_present) = background_gpu_over_cards_on_the_cpu(
        vec![
            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
                .with_a_submission_running_past_its_wait(600),
        ],
        &memory_leaked,
    );
    drop(earlier_mount);
    assert_eq!(memory_leaked.bytes_leaked_on(&card), GpuMemoryInBytes(600));
    let (later_mount, _later_present) = background_gpu_over_cards_on_the_cpu(
        vec![CardComputedOnTheCpuForTests::of_the_address(
            "0000:01:00.0",
            None,
            memory_leaked.bytes_leaked_on(&card).0,
        )
        .with_a_submission_running_past_its_wait(300)],
        &memory_leaked,
    );
    let gpus_leaked_by_the_later_mount = later_mount.gpus_leaked_when_dropped();
    drop(later_mount);
    assert_eq!(
        gpus_leaked_by_the_later_mount.gpus_leaked(),
        Some(vec![GpuLeakedWhenDropped {
            device: card.clone(),
            bytes_held_on_the_device: GpuMemoryInBytes(900),
            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(600),
        }]),
        "后一次挂载丢掉那一刻占着 900，其中 600 是开卡时已算上的"
    );
    assert_eq!(
        memory_leaked.bytes_leaked_on(&card),
        GpuMemoryInBytes(900),
        "进程里只加这一次自己的 300"
    );
}
