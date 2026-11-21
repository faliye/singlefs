use super::*;

/// 补（等设备的 30 秒只留一处定义）：在这一趟自己起的卡上判完、还显存排队的占位之前那一次等，等的是到此刻为止的全部提交，
/// 上限是卡池借据丢掉时那一次等的同一个常量（它的值由 `gpu_unit_checks` 的测试钉在 E174 登记的 30 秒）；
/// 这一处改回自己写的秒数就红。不要卡。
#[cfg(feature = "gpu")]
#[test]
fn a_place_in_the_memory_queue_is_returned_after_waiting_for_every_submission_as_long_as_a_returned_card_lease(
) {
    match poll_type_before_returning_a_place_in_the_memory_queue() {
            wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(timeout),
            } => assert_eq!(
                timeout,
                crate::gpu_unit_checks::LONGEST_WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE_OR_A_PLACE_IN_THE_MEMORY_QUEUE,
                "还占位之前等的上限与卡池借据丢掉时那一次等的是同一个常量"
            ),
            wgpu::PollType::Wait {
                submission_index: Some(_),
                ..
            }
            | wgpu::PollType::Wait { timeout: None, .. }
            | wgpu::PollType::Poll => {
                panic!(
                    "还占位之前等到此刻为止的全部提交、有上限：{:?}",
                    poll_type_before_returning_a_place_in_the_memory_queue()
                )
            }
        }
}

/// 观察者交回的 `image_count` 张盘照拿到的 `threads_taken` 条线程切出的段，核每段的张数是 `expected_piece_lengths`、
/// 几段升序连着、按次序罩住每一张盘、每张盘恰好落在一段里（断言消息带盘数与条数）。
fn assert_the_images_after_the_observers_are_cut_evenly(
    image_count: usize,
    threads_taken: usize,
    expected_piece_lengths: &[usize],
) {
    let pieces = images_after_the_observers_cut_by_the_threads_taken(image_count, threads_taken);
    assert_eq!(
        pieces
            .iter()
            .map(|piece| piece.end - piece.start)
            .collect::<Vec<usize>>(),
        expected_piece_lengths,
        "{image_count} 张盘、{threads_taken} 条线程：每段的张数：{pieces:?}"
    );
    assert_eq!(
            pieces
                .iter()
                .flat_map(std::clone::Clone::clone)
                .collect::<Vec<usize>>(),
            (0..image_count).collect::<Vec<usize>>(),
            "{image_count} 张盘、{threads_taken} 条线程：几段升序连着、按次序罩住每一张盘，每张盘恰好落在一段里：{pieces:?}"
        );
}

/// 场景：观察者交回的 18 张盘照拿到的 17 条线程切段（盘数比条数多一张；两个数写死，不从环境取）。
/// 预期：切 17 段，头一段 2 张、其余各 1 张；几段连着、按次序罩住每一张盘。
/// 判别力（`crates/mutations.tsv`）：段长向上取整（只切 9 段）、照核对线程数切、一张盘切一段、多的那张放到末段，各让这条红。
#[test]
fn eighteen_images_after_the_observers_are_cut_into_seventeen_pieces_for_seventeen_threads() {
    assert_the_images_after_the_observers_are_cut_evenly(
        18,
        17,
        &[vec![2usize], vec![1usize; 16]].concat(),
    );
}

/// 场景：观察者交回的 31 张盘照拿到的 17 条线程切段（两个数写死，不从环境取）。
/// 预期：切 17 段，前 14 段各 2 张（31 除以 17 余 14）、后 3 段各 1 张；几段连着、按次序罩住每一张盘。
/// 判别力（`crates/mutations.tsv`）：段长向上取整（只切 16 段），这条红。
#[test]
fn thirty_one_images_after_the_observers_are_cut_into_seventeen_pieces_for_seventeen_threads() {
    assert_the_images_after_the_observers_are_cut_evenly(
        31,
        17,
        &[vec![2usize; 14], vec![1usize; 3]].concat(),
    );
}

/// 场景：观察者交回的 5 张盘照拿到的 8 条线程切段（条数多于盘数；两个数写死，不从环境取）。
/// 预期：切 5 段、各 1 张（段数取盘数与条数里小的那个，不切空段）。
/// 判别力（`crates/mutations.tsv`）：段数不取两个数里小的那个（照条数切 8 段、有空段），这条红。
#[test]
fn five_images_after_the_observers_are_cut_into_five_pieces_for_eight_threads() {
    assert_the_images_after_the_observers_are_cut_evenly(5, 8, &[1usize; 5]);
}

/// 观察者交回的三张盘，基底是一块 1 MiB、没做 mkfs 的空内存盘，没有枚举写、观察者也没发写：中间那张一条不变量都不判，
/// 另两张判全部已实现的不变量（三张的结论可以不同，次序错了比得出来）。
fn three_images_after_the_observers_on_a_blank_pool(
) -> (ObserverFoundation, Vec<ImageToJudgeAfterTheObserver>) {
    let foundation = ObserverFoundation {
        base: MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(0)], 1 << 20),
        enumerated_writes: Vec::new(),
    };
    let image_judging = |invariants_judged| ImageToJudgeAfterTheObserver {
        foundation_digest: sha256_digest(b"blank foundation of the pieces checked on the cpu\n"),
        persisted: Vec::new(),
        writes_on_the_crash_image: Vec::new(),
        invariants_judged,
        counter_when_the_pool_checker_is_clean: "clean".to_string(),
    };
    let images = vec![
        image_judging(InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant),
        image_judging(InvariantsJudgedOnTheImageAfterTheObserver::Only(Vec::new())),
        image_judging(InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant),
    ];
    (foundation, images)
}

/// 同一基底上的这几张盘，去重键各取它在 `images` 里的下标的摘要。
fn images_after_the_observers_on_the_foundation<'judge>(
    foundation: &'judge ObserverFoundation,
    images: &'judge [ImageToJudgeAfterTheObserver],
) -> Vec<ImageAfterTheObserverToJudge<'judge>> {
    images
        .iter()
        .enumerate()
        .map(|(position, image)| ImageAfterTheObserverToJudge {
            deduplication_key: sha256_digest(&position.to_le_bytes()),
            foundation,
            image,
        })
        .collect()
}

/// 把空盘上的三张观察者交回的盘（[`three_images_after_the_observers_on_a_blank_pool`]）照交进来的 `pieces` 在 CPU 上判。
fn judge_three_blank_images_after_the_observers_on_the_cpu_in_these_pieces(
    pieces: &[Range<usize>],
) -> Vec<ImageJudgementRecord> {
    let (foundation, images_to_judge) = three_images_after_the_observers_on_a_blank_pool();
    let images = images_after_the_observers_on_the_foundation(&foundation, &images_to_judge);
    judge_the_images_after_the_observers_on_the_cpu(&images, pieces)
}

/// 场景：观察者交回的三张盘在 CPU 上判，交进来的两段 `0..1`、`2..3` 漏了下标 1 那张盘。
/// 预期：起线程之前 panic，消息点名下标 1 那一段的起止、盘数与前面各段罩到的末尾，并写清依赖的约定。
/// 判别力（`crates/mutations.tsv`）：入口不核交进来的段（漏了的那张盘不判、结论少一条），这条红。
#[test]
#[should_panic(
    expected = "交进来的下标 1 那一段是 2..3，盘数 3：它不接着前面各段的末尾 1 起、是空段或越过了盘数；依赖的约定：调用方交的段要恰好罩全盘的下标"
)]
fn pieces_that_leave_out_an_image_after_the_observers_panic_naming_the_piece() {
    let _ = judge_three_blank_images_after_the_observers_on_the_cpu_in_these_pieces(&[0..1, 2..3]);
}

/// 场景：观察者交回的三张盘在 CPU 上判，交进来的两段 `0..2`、`1..3` 在下标 1 那张盘上重叠。
/// 预期：起线程之前 panic，消息点名下标 1 那一段的起止、盘数与前面各段罩到的末尾。
/// 判别力（`crates/mutations.tsv`）：核段时只要求不越过前面各段的末尾（重叠的段放过），这条红。
#[test]
#[should_panic(expected = "交进来的下标 1 那一段是 1..3，盘数 3：它不接着前面各段的末尾 2 起")]
fn pieces_that_overlap_on_an_image_after_the_observers_panic_naming_the_piece() {
    let _ = judge_three_blank_images_after_the_observers_on_the_cpu_in_these_pieces(&[0..2, 1..3]);
}

/// 场景：观察者交回的三张盘在 CPU 上判，交进来的两段 `1..3`、`0..1` 罩全了每张盘、次序倒了。
/// 预期：起线程之前 panic，消息点名下标 0 那一段：它不从 0 起。
/// 判别力（`crates/mutations.tsv`）：核段时不要求头一段从 0 起（panic 点名的换成下标 1 那一段），这条红。
#[test]
#[should_panic(expected = "交进来的下标 0 那一段是 1..3，盘数 3：它不接着前面各段的末尾 0 起")]
fn pieces_in_reverse_order_over_the_images_after_the_observers_panic_naming_the_piece() {
    let _ = judge_three_blank_images_after_the_observers_on_the_cpu_in_these_pieces(&[1..3, 0..1]);
}

/// 场景：观察者交回的三张盘在 CPU 上判，交进来的一段 `0..2` 没罩到末一张盘。
/// 预期：起线程之前 panic，消息点名交进来的段止于 2、末一段的起止与盘数。
/// 判别力（`crates/mutations.tsv`）：核段时不核末一段到盘数止，这条红。
#[test]
#[should_panic(
    expected = "交进来的 1 段止于 2（末一段是 Some(0..2)），盘数 3：没罩到末一张盘；依赖的约定：调用方交的段要恰好罩全盘的下标"
)]
fn pieces_that_stop_before_the_last_image_after_the_observers_panic_naming_the_last_piece() {
    let one_piece_that_leaves_out_the_last_image: Vec<Range<usize>> =
        std::iter::once(0..2).collect();
    let _ = judge_three_blank_images_after_the_observers_on_the_cpu_in_these_pieces(
        &one_piece_that_leaves_out_the_last_image,
    );
}

/// 场景：观察者交回的三张盘在 CPU 上判，交进来的两段 `0..1`、`1..3` 恰好罩全每张盘。
/// 预期：照常判完，交回三条 CPU 判的结论，按盘的次序，每条与那张盘单独成一段判出的逐字段相同；没有盘、段表为空时交回空的。
/// 判别力（`crates/mutations.tsv`）：核段时把到盘数止的末一段也当越过了盘数（罩全的段也 panic），这条红。
#[test]
fn pieces_that_cover_every_image_after_the_observers_once_are_judged_in_the_order_of_the_images() {
    let (foundation, images_to_judge) = three_images_after_the_observers_on_a_blank_pool();
    let images = images_after_the_observers_on_the_foundation(&foundation, &images_to_judge);
    let judged_in_pieces = judge_the_images_after_the_observers_on_the_cpu(&images, &[0..1, 1..3]);
    let one_piece_over_the_one_image_judged_alone: Vec<Range<usize>> =
        std::iter::once(0..1).collect();
    let judged_one_by_one: Vec<ImageJudgementRecord> = images
        .iter()
        .flat_map(|image| {
            judge_the_images_after_the_observers_on_the_cpu(
                std::slice::from_ref(image),
                &one_piece_over_the_one_image_judged_alone,
            )
        })
        .collect();
    assert_eq!(
        judged_in_pieces, judged_one_by_one,
        "三张盘分两段判出的结论按盘的次序、与逐张单独判出的逐字段相同"
    );
    assert!(
        judged_in_pieces.iter().all(
            |record| record.judged_on == JudgingDevice::Cpu && record.could_not_judge.is_none()
        ),
        "三条都是 CPU 判出的、没有判不了的：{judged_in_pieces:?}"
    );
    assert_eq!(judged_in_pieces.len(), 3, "交回三条结论");
    assert_eq!(
        judge_the_images_after_the_observers_on_the_cpu(&[], &[]),
        Vec::new(),
        "没有盘、段表为空：照常交回空的"
    );
}

/// CPU 参照那一份结论字（观察者交回的盘那一种只读违例位）：CPU 判器判出违例的每一条不变量，在它于
/// [`IMPLEMENTED_INVARIANTS`] 里的下标处置违例位（位 i 在第 `POOL_CHECKER_VIOLATED_FIRST_WORD + i / 32` 字的第 `i % 32` 位），别的字全 0。
#[cfg(feature = "gpu")]
fn verdict_words_of_the_pool_checker_violations(
    violated_invariants: &[String],
) -> [u32; crate::crash_judge_gpu::VERDICT_WORDS] {
    use crate::crash_judge_gpu::{POOL_CHECKER_VIOLATED_FIRST_WORD, VERDICT_WORDS};
    let mut words = [0u32; VERDICT_WORDS];
    for invariant in violated_invariants {
        let bit = IMPLEMENTED_INVARIANTS
            .iter()
            .position(|implemented| *implemented == invariant.as_str())
            .expect("CPU 判器判出违例的不变量都在已实现的那一张清单里");
        words[POOL_CHECKER_VIOLATED_FIRST_WORD + bit / 32] |= 1 << (bit % 32);
    }
    words
}

/// 场景：配了 GPU（多机配置只写 `ENABLE_GPU=1`）、环境里没有服务交的卡；观察者交回一张盘（做完 mkfs 的两块盘、一次持久的系统配置槽写，
/// 判全部已实现的不变量）要判，池级 checker 在头一张要判的盘那一刻照产品取卡的路取卡（取不到），判表交替身服务判
/// （[`packed_tables_stand_in`]：结论字取 CPU 参照那一份，[`verdict_words_of_the_pool_checker_violations`] 照 CPU 判器在这张盘上判出的违例置位）。
/// 预期：交回的结论与在本进程上卡那一支（[`judge_the_images_after_the_observers_on_the_gpu`] 照同一份结论字判出的）逐条相同，违例的不变量与
/// CPU 判器的逐条相同，在哪判的是替身写的那一张卡；替身读到的判表待判行恰好一行：旋钮是产品那一份、态数 1、不强制，
/// 键里内核源码那一段是 `of_the_judge()` 那三份的摘要；判完之后那一行判表待判行不在、判表结论字在。
/// 判别力（`crates/mutations.tsv`）：取卡没成就报错交回（不交服务判），这条红。
#[cfg(feature = "gpu")]
#[test]
fn an_image_after_an_observer_to_judge_without_a_card_is_judged_through_the_gpu_verification_server(
) {
    let (disposable, mut store) =
        an_empty_library_for("observer-image-through-the-gpu-verification-server");
    let configuration = disposable.directory().join("multi-host-with-the-gpu.env");
    std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了配置");
    // 先开库、后拿改环境的锁；守卫清掉了服务交的卡（GPU_CARDS_BY_PRIORITY）与领卡通道，只设多机配置
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        crate::gpu_card_configuration::MULTI_HOST_CONFIGURATION_ENVIRONMENT_VARIABLE,
        configuration.as_os_str(),
    )]);
    // 判段入口那一刻记下的显卡来路：池级 checker 取卡照它，不读取卡那一刻的环境
    let card_sources_of_the_entry =
        CardSourcesOfTheEngineStart::read_from_the_environment_now(configuration.clone());
    assert!(
        gpu_cards_of_this_run_from_the_multi_host_configuration(&card_sources_of_the_entry)
            .is_err(),
        "前提：配置开了 GPU、环境里没有服务交的卡，取卡要报错"
    );
    let foundation = ObserverFoundation {
        base: a_formatted_pool_for_the_judge_tables_of_the_observer_images(),
        enumerated_writes: vec![a_system_configuration_slot_write(0, 0x5A)],
    };
    let image = ImageToJudgeAfterTheObserver {
        foundation_digest: sha256_digest(b"foundation of the image judged through the service\n"),
        persisted: vec![true],
        writes_on_the_crash_image: Vec::new(),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: "clean".to_string(),
    };
    let images = [ImageAfterTheObserverToJudge {
        deduplication_key: sha256_digest(b"image judged through the service\n"),
        foundation: &foundation,
        image: &image,
    }];
    let cpu = judge_the_images_after_the_observers_on_the_cpu(
        &images,
        std::slice::from_ref(&(0..images.len())),
    );
    let words_of_the_cpu =
        verdict_words_of_the_pool_checker_violations(&cpu[0].violated_invariants);
    let rows_seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let pool_checker =
        PoolCheckerOfTheImagesAfterTheObservers::TakingTheGpuCardsAtTheFirstImageToJudge(
            CardsTakenAtTheFirstImageToJudge::new(
                card_sources_of_the_entry,
                WayToTheGpuVerificationServer::StandIn(packed_tables_stand_in(
                    move |_key, _row| vec![words_of_the_cpu],
                    std::sync::Arc::clone(&rows_seen),
                )),
            ),
        );
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let through_the_service = pool_checker
        .judge(
            &library,
            &images,
            crate::verdict_store::ForcedRejudging::NotForced,
        )
        .expect("取卡没成的判表交替身服务判");
    drop(environment);
    let on_a_card_in_this_process = judge_the_images_after_the_observers_on_the_gpu(
        &library,
        &images,
        &mut |_packed_tables_read_back| {
            Ok((
                vec![words_of_the_cpu],
                the_card_of_the_packed_tables_stand_in(),
            ))
        },
    )
    .expect("本进程上卡那一支照同一份结论字判得出");
    assert_eq!(
        through_the_service, on_a_card_in_this_process,
        "经服务那一支与本进程上卡那一支逐条相同"
    );
    assert_eq!(
        through_the_service[0]
            .violated_invariants
            .iter()
            .collect::<BTreeSet<_>>(),
        cpu[0].violated_invariants.iter().collect::<BTreeSet<_>>(),
        "违例的不变量与 CPU 判器的逐条相同"
    );
    assert_eq!(
        (
            through_the_service[0].could_not_judge.clone(),
            through_the_service[0].judged_on.clone()
        ),
        (None, the_card_of_the_packed_tables_stand_in()),
        "判得了、在替身那一张卡上判的"
    );
    let rows_seen = rows_seen
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        rows_seen.len(),
        1,
        "替身读到恰好一行判表待判行：{rows_seen:?}"
    );
    let (key, row) = rows_seen[0];
    assert_eq!(
        (
            row.container_capacity_knobs,
            row.state_count,
            row.forced,
            key.container_capacity_knobs_digest
        ),
        (
            ContainerCapacityKnobs::PRODUCT,
            1,
            crate::verdict_store::ForcedRejudging::NotForced,
            container_capacity_knobs_digest(ContainerCapacityKnobs::PRODUCT)
        ),
        "判表待判行：产品旋钮、一个状态、不强制，键里是产品旋钮的摘要（不带判器的版本）"
    );
    let pending_row_after = library
        .take()
        .read_packed_tables_pending_row(&key)
        .expect("读得了库");
    let words_after = library
        .take()
        .read_packed_tables_verdict_words(&key)
        .expect("读得了库");
    assert_eq!(
        (
            pending_row_after,
            words_after.map(|words| words.verdict_words)
        ),
        (None, Some(vec![words_of_the_cpu])),
        "判完之后判表待判行不在、判表结论字在"
    );
}

/// 单测里调用方交的卡串：写坏了的一项（少了显存额度那一段）。
#[cfg(feature = "gpu")]
fn a_card_string_written_wrong_by_the_caller_of_the_test() -> Option<String> {
    Some("192.0.2.1:0".to_string())
}

/// 单测里调用方没交卡串。
#[cfg(feature = "gpu")]
fn no_card_string_handed_over_by_the_caller_of_the_test() -> Option<String> {
    None
}

/// 单测里取卡那一下：照写坏了的卡串报那一句（不读多机配置、不起卡池）。
#[cfg(feature = "gpu")]
fn cards_refused_for_the_card_string_written_wrong_of_the_test() -> Result<GpuCardsOfThisRun, String>
{
    Err("显卡优先级的一项要写 <HOSTS 里的地址>:<序号>:<显存额度>（例 192.0.2.1:0:32G），实际「192.0.2.1:0」".to_string())
}

/// 单测里取卡那一下：照没经服务给卡报那一句（不读多机配置、不起卡池）。
#[cfg(feature = "gpu")]
fn cards_refused_without_a_card_string_of_the_test() -> Result<GpuCardsOfThisRun, String> {
    Err("没经服务给卡（单测造的那一句）".to_string())
}

/// 场景：配了 GPU 的判段，调用方交了一份写坏了的卡串（取卡那一下报卡串写法不对）；观察者交回一张盘（一块空的内存盘）要判，
/// 取卡没成之后交服务判的那一条路是替身（[`packed_tables_stand_in`]）。
/// 预期：判这张盘报错，错里带取卡报的那一句；替身一行判表待判行都没读到，库里一行判表待判行都没有（交了卡串而取不成的不交服务判）。
/// 判别力（`crates/mutations.tsv`）：交了卡串而取不成的也交服务判，这条红。
#[cfg(feature = "gpu")]
#[test]
fn an_image_after_an_observer_whose_card_string_is_not_taken_is_refused_without_a_pending_row() {
    let (_disposable, mut store) =
        an_empty_library_for("observer-image-whose-card-string-is-not-taken");
    let foundation = ObserverFoundation {
        base: MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(0)], 1 << 20),
        enumerated_writes: Vec::new(),
    };
    let image = ImageToJudgeAfterTheObserver {
        foundation_digest: sha256_digest(
            b"foundation of the image whose card string is not taken\n",
        ),
        persisted: Vec::new(),
        writes_on_the_crash_image: Vec::new(),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: "clean".to_string(),
    };
    let images = [ImageAfterTheObserverToJudge {
        deduplication_key: sha256_digest(b"image whose card string is not taken\n"),
        foundation: &foundation,
        image: &image,
    }];
    let rows_seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let pool_checker =
        PoolCheckerOfTheImagesAfterTheObservers::TakingTheGpuCardsAtTheFirstImageToJudge(
            CardsTakenAtTheFirstImageToJudge::with_the_cards_of_the_test(
                cards_refused_for_the_card_string_written_wrong_of_the_test,
                a_card_string_written_wrong_by_the_caller_of_the_test(),
                WayToTheGpuVerificationServer::StandIn(packed_tables_stand_in(
                    |_key, row| {
                        vec![
                            [0u32; crate::crash_judge_gpu::VERDICT_WORDS];
                            usize::try_from(row.state_count).expect("态数装得进 usize")
                        ]
                    },
                    std::sync::Arc::clone(&rows_seen),
                )),
            ),
        );
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let refusal = pool_checker
        .judge(
            &library,
            &images,
            crate::verdict_store::ForcedRejudging::NotForced,
        )
        .expect_err("交了卡串而取卡没成：报错，不交服务判");
    assert!(
        refusal.contains("实际「192.0.2.1:0」"),
        "错里带取卡报的那一句：{refusal}"
    );
    assert_eq!(
        (
            rows_seen
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len(),
            library
                .take()
                .every_packed_tables_pending_row()
                .expect("扫得了库")
                .len()
        ),
        (0, 0),
        "替身一行判表待判行都没读到，库里一行都没写"
    );
}

/// 场景：配了 GPU 的判段，调用方没交卡串（取卡那一下报没经服务给卡）；观察者交回一张盘（同
/// [`an_image_after_an_observer_to_judge_without_a_card_is_judged_through_the_gpu_verification_server`] 那一张：做完 mkfs 的两块盘、一次持久的系统配置槽写），
/// 交下来的是强制现判（这张盘在复用哨兵抽中强制现判的那一块上），判表交替身服务判。
/// 预期：判得出；替身读到恰好一行判表待判行，那一行是强制现判。
/// 判别力（`crates/mutations.tsv`）：观察者交回的盘交服务判时一律写不强制（不照交下来的那一份），这条红。
#[cfg(feature = "gpu")]
#[test]
fn a_forced_image_after_an_observer_is_handed_to_the_gpu_verification_server_as_forced() {
    let (_disposable, mut store) =
        an_empty_library_for("forced-observer-image-through-the-gpu-verification-server");
    let foundation = ObserverFoundation {
        base: a_formatted_pool_for_the_judge_tables_of_the_observer_images(),
        enumerated_writes: vec![a_system_configuration_slot_write(0, 0x5A)],
    };
    let image = ImageToJudgeAfterTheObserver {
        foundation_digest: sha256_digest(
            b"foundation of the forced image judged through the service\n",
        ),
        persisted: vec![true],
        writes_on_the_crash_image: Vec::new(),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: "clean".to_string(),
    };
    let images = [ImageAfterTheObserverToJudge {
        deduplication_key: sha256_digest(b"forced image judged through the service\n"),
        foundation: &foundation,
        image: &image,
    }];
    let cpu = judge_the_images_after_the_observers_on_the_cpu(
        &images,
        std::slice::from_ref(&(0..images.len())),
    );
    let words_of_the_cpu =
        verdict_words_of_the_pool_checker_violations(&cpu[0].violated_invariants);
    let rows_seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let pool_checker =
        PoolCheckerOfTheImagesAfterTheObservers::TakingTheGpuCardsAtTheFirstImageToJudge(
            CardsTakenAtTheFirstImageToJudge::with_the_cards_of_the_test(
                cards_refused_without_a_card_string_of_the_test,
                no_card_string_handed_over_by_the_caller_of_the_test(),
                WayToTheGpuVerificationServer::StandIn(packed_tables_stand_in(
                    move |_key, _row| vec![words_of_the_cpu],
                    std::sync::Arc::clone(&rows_seen),
                )),
            ),
        );
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    pool_checker
        .judge(
            &library,
            &images,
            crate::verdict_store::ForcedRejudging::Forced,
        )
        .expect("没交卡的判表交替身服务判");
    assert_eq!(
        rows_seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|(_key, row)| row.forced)
            .collect::<Vec<_>>(),
        vec![crate::verdict_store::ForcedRejudging::Forced],
        "替身读到恰好一行判表待判行，是强制现判"
    );
}

/// 挂载把同一个位置（盘 0 的系统配置槽 0 号）写了超过判表一个落点的写数上限那么多次：基底枚举写表在那里写了上限加 6 次，一张盘全持久、
/// 观察者又在那里写 3 次（不并是上限加 9 次，越过 [`crate::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION`]，单张盘也建不出判表）；另 40 张盘
/// 各持久前 1 到 40 次，观察者都只发同一次写（盘 1 的系统配置槽，逐字节相同）。并过同一位置之后每张盘都落在某一张建得出的判表里、恰好一次；
/// 交回的写相同的盘共用一份写：41 张盘只排成几张表，每张表里那一次共用的写只有一份；每张盘照判表的掩码按判表写表的次序叠在基底上，
/// 与它自己的次序叠出的盘（[`pool_after_the_observer`]）逐字节相同。
/// 判别力（`crates/mutations.tsv`）：每张盘同一位置的写不并，全持久的那一张建不出判表，这条红；交回的写相同的盘不共用（一张盘一张表），这条红。
#[test]
fn observer_images_writing_one_location_past_the_judge_table_limit_are_merged_and_share_their_writes(
) {
    use crate::crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION;
    const ENUMERATED_REWRITES_OF_THE_SLOT: usize = MAXIMUM_WRITES_PER_LOCATION + 6;
    const PARTLY_PERSISTED_IMAGES: usize = 40;
    let base = a_formatted_pool_for_the_judge_tables_of_the_observer_images();
    let enumerated_writes: Vec<RetainedWrite> = (0..ENUMERATED_REWRITES_OF_THE_SLOT)
        .map(|rewrite| {
            a_system_configuration_slot_write(
                0,
                u8::try_from(rewrite % 255 + 1).expect("取模之后 1 到 255 装得进 u8"),
            )
        })
        .collect();
    let foundation = ObserverFoundation {
        base,
        enumerated_writes,
    };
    let foundation_digest = sha256_digest(b"foundation of the unit test");
    let shared_write = a_system_configuration_slot_write(1, 0xa5);
    let image_to_judge = |persisted_prefix: usize,
                          writes_on_the_crash_image: Vec<RetainedWrite>| {
        ImageToJudgeAfterTheObserver {
            foundation_digest,
            persisted: (0..ENUMERATED_REWRITES_OF_THE_SLOT)
                .map(|write_index| write_index < persisted_prefix)
                .collect(),
            writes_on_the_crash_image,
            invariants_judged:
                InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
            counter_when_the_pool_checker_is_clean: "clean".to_string(),
        }
    };
    let mut images_to_judge = vec![image_to_judge(
        ENUMERATED_REWRITES_OF_THE_SLOT,
        vec![
            a_system_configuration_slot_write(0, 201),
            a_system_configuration_slot_write(0, 202),
            a_system_configuration_slot_write(0, 203),
            shared_write.clone(),
        ],
    )];
    for persisted_prefix in 1..=PARTLY_PERSISTED_IMAGES {
        images_to_judge.push(image_to_judge(persisted_prefix, vec![shared_write.clone()]));
    }
    let images: Vec<ImageAfterTheObserverToJudge<'_>> = images_to_judge
        .iter()
        .enumerate()
        .map(|(position, image)| ImageAfterTheObserverToJudge {
            deduplication_key: sha256_digest(&position.to_le_bytes()),
            foundation: &foundation,
            image,
        })
        .collect();
    let mut planned: Vec<PlannedJudgeTableOfImagesAfterTheObservers> = Vec::new();
    plan_the_judge_tables_of_the_images_after_the_observers(&images, &mut |table| {
        planned.push(table);
        Ok(())
    })
    .expect("收下每一张的那一步不报错");
    let mut tables_built = 0usize;
    let mut times_each_image_is_planned = vec![0usize; images.len()];
    for table in &planned {
        let built = match table {
            PlannedJudgeTableOfImagesAfterTheObservers::Built(built) => built,
            PlannedJudgeTableOfImagesAfterTheObservers::NotBuildableForOneImage {
                position,
                reason,
            } => panic!("第 {position} 张盘并过同一位置上的写之后要建得出判表：{reason}"),
        };
        tables_built += 1;
        assert_eq!(
            built
                .writes
                .iter()
                .filter(|write| **write == shared_write)
                .count(),
            1,
            "交回的写逐字节相同的几张盘在一张表里共用一份写"
        );
        let persisted_sets = match &built.tables.state_source {
            crate::crash_judge_tables::JudgeStateSource::GivenPersistedSets {
                persisted_sets,
                claimed_versions: _,
            } => persisted_sets,
            crate::crash_judge_tables::JudgeStateSource::SegmentExpansions => {
                panic!("观察者交回的盘建的是给定状态的判表")
            }
        };
        assert_eq!(persisted_sets.len(), built.positions.len());
        assert_eq!(
            built.tables.state_count,
            u64::try_from(built.positions.len()).expect("张数装得进 u64"),
            "判表一张盘一个状态"
        );
        for (position, persisted) in built.positions.iter().zip(persisted_sets) {
            times_each_image_is_planned[*position] += 1;
            assert!(
                persisted[built.writes.len()..]
                    .iter()
                    .all(|is_persisted| !*is_persisted),
                "建表接上的撕裂镜像不在任何一张盘的掩码里"
            );
            let mut pool = foundation.base.clone();
            for (write, is_persisted) in built.writes.iter().zip(persisted) {
                if *is_persisted {
                    pool.apply_writes(std::slice::from_ref(write));
                }
            }
            assert!(
                pools_hold_the_same_bytes(&pool, &pool_after_the_observer(&images[*position])),
                "第 {position} 张盘照判表的掩码叠出来与它自己的次序叠出来逐字节相同"
            );
        }
    }
    assert_eq!(
        times_each_image_is_planned,
        vec![1; images.len()],
        "每张盘恰好落在一张判表里"
    );
    assert!(
        tables_built <= 4,
        "{} 张同一基底、交回的写相同的盘只排成几张表，排了 {tables_built} 张",
        images.len()
    );
}

/// 观察者交回的盘上「GPU 判不了」不是判器的结论（建表那一截在这份文件里、不在判器那一把版本里）：判段这一批判过的盘里只存判器判出来的那几张，
/// 带着一张判不了的那一块不存进库，都判干净的那一块存。判别力（`crates/mutations.tsv`）：判不了的也当结论存，这条红。
#[test]
fn an_image_the_gpu_could_not_judge_is_not_stored_and_its_block_is_not_kept() {
    let foundation = ObserverFoundation {
        base: MemoryPool::with_devices(&[singlefs_core::address::DeviceIdentity(0)], 1 << 20),
        enumerated_writes: Vec::new(),
    };
    let image = ImageToJudgeAfterTheObserver {
        foundation_digest: [0; DIGEST_BYTES],
        persisted: Vec::new(),
        writes_on_the_crash_image: Vec::new(),
        invariants_judged: InvariantsJudgedOnTheImageAfterTheObserver::EveryImplementedInvariant,
        counter_when_the_pool_checker_is_clean: "clean".to_string(),
    };
    let images = [
        ImageAfterTheObserverToJudge {
            deduplication_key: [1; DIGEST_BYTES],
            foundation: &foundation,
            image: &image,
        },
        ImageAfterTheObserverToJudge {
            deduplication_key: [2; DIGEST_BYTES],
            foundation: &foundation,
            image: &image,
        },
    ];
    let judged_clean = ImageJudgementRecord {
        violated_invariants: Vec::new(),
        could_not_judge: None,
        judged_on: JudgingDevice::Gpu {
            card_type: "单测".to_string(),
            driver_version: String::new(),
        },
    };
    let could_not_judge = ImageJudgementRecord {
        violated_invariants: Vec::new(),
        could_not_judge: Some("建不出 GPU 判表：单测".to_string()),
        judged_on: JudgingDevice::Cpu,
    };
    let stored =
        image_judgements_to_store(&images, &[judged_clean.clone(), could_not_judge.clone()]);
    assert_eq!(
        stored
            .iter()
            .map(|(deduplication_key, _)| *deduplication_key)
            .collect::<Vec<_>>(),
        vec![[1; DIGEST_BYTES]],
        "只存判器判出来的那一张"
    );
    assert!(!block_conclusion_is_kept_in_the_library(&[
        Some(judged_clean.clone()),
        Some(could_not_judge),
        None
    ]));
    assert!(block_conclusion_is_kept_in_the_library(&[
        Some(judged_clean),
        None
    ]));
}
