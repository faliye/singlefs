// task-server-gpu-cards: 2
//! 两张卡：校验和核对那两份核在服务交来的同一台两张真卡上按卡的顺序填满（用户 2026-11-02 定「按卡的顺序填满」，测饱和），每张卡都分到活、
//! 结果与 CPU 参照逐条相同；另有一条活少时只有点名在前的那张分到活。七·3 验收「一张卡1gb 2张卡 三张卡 每张卡都能用来分片gc」（用户 2026-11-01 定，
//! 原话在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md`）的两张那一格；一张的在 `checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference`，
//! 三张的在 `checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference`。
//! 源头第一行声明要同一台两张卡（`task-server/README.md`「源文件头的声明」），服务照它分、经 `GPU_CARDS_BY_PRIORITY` 交；用例照交来的次序取头两张
//! （`common_discrete_gpus_of_this_host`）。服务没交这一台的卡：每条打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」；
//! 交来的少于两张、或对不上本机的独显：判红，不降级成一张跑。
//! 只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `task-server/task submit` 交（卡与清场由服务定），带 `--test-threads 1`。
#![cfg(feature = "tests-on-a-real-gpu")]

mod common_checksum_batches_over_the_handed_over_gpus;
mod common_discrete_gpus_of_this_host;

use common_checksum_batches_over_the_handed_over_gpus::{
    background_gpu_on_the_declared_count_of_handed_over_gpus_or_report_not_run,
    cpu_reference_outcomes,
    every_handed_over_gpu_of_a_batch_is_given_work_and_submits_on_the_real_gpus,
    units_of_four_kibibytes, units_to_verify,
};
use common_discrete_gpus_of_this_host::MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU;

/// 这一份源头第一行 `// task-server-gpu-cards:` 声明的张数，用例照它取交来的卡；两处要相同（单测
/// `the_two_gpu_target_declares_in_its_header_the_gpu_count_its_tests_ask_for` 核）。
const GPU_CARDS_DECLARED_IN_THE_HEADER: usize = 2;
/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: std::time::Duration = std::time::Duration::from_secs(60);

#[test]
fn the_batch_over_two_named_real_gpus_gives_each_work_and_each_submits() {
    every_handed_over_gpu_of_a_batch_is_given_work_and_submits_on_the_real_gpus(
        "the_batch_over_two_named_real_gpus_gives_each_work_and_each_submits",
        GPU_CARDS_DECLARED_IN_THE_HEADER,
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    );
}

/// 同一定案的活少那一头：点名交来的两张、各 1 GiB，一批 1024 个 4096 字节的单元（4 MiB 多，点名在前的那张一次交得下）——
/// 只有点名在前的那张分到活、交一次（不要求每张卡都分到活）。
#[test]
fn the_batch_the_first_named_real_gpu_holds_goes_only_to_that_gpu() {
    let test_name = "the_batch_the_first_named_real_gpu_holds_goes_only_to_that_gpu";
    let Some((mut background_gpu, named_cards)) =
        background_gpu_on_the_declared_count_of_handed_over_gpus_or_report_not_run(
            test_name,
            GPU_CARDS_DECLARED_IN_THE_HEADER,
            MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU,
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        )
    else {
        return;
    };
    let unit_bytes_and_expected = units_of_four_kibibytes(1024);
    let outcomes = background_gpu
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("健康的卡上一批交得成");
    let submissions = background_gpu
        .take_the_submissions_of_the_batch()
        .unit_checksum_verification;
    assert_eq!(
        outcomes,
        cpu_reference_outcomes(&unit_bytes_and_expected)
            .iter()
            .map(|outcome| (*outcome, *outcome))
            .collect::<Vec<_>>(),
        "两份核逐条与 CPU 参照相同"
    );
    assert_eq!(
        (
            submissions.devices_given_work,
            submissions.submissions_completed
        ),
        (vec![named_cards[0].clone()], 1),
        "只有点名在前的那张分到活、交一次"
    );
}
