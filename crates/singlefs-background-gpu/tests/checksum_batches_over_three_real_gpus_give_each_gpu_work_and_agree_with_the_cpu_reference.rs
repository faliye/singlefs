// task-server-gpu-cards: 3
//! 三张卡：校验和核对那两份核在服务交来的同一台三张真卡上按卡的顺序填满（测饱和），每张卡都分到活、
//! 结果与 CPU 参照逐条相同。验收「一张卡 1 GiB、两张卡、三张卡，每张卡都能用来分片 gc」
//! 的三张那一格；一张的在 `checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference`，
//! 两张的在 `checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference`。
//! 源头第一行声明要同一台三张卡（`task-server/README.md`「源文件头的声明」），服务照它分、经 `GPU_CARDS_BY_PRIORITY` 交（哪台凑不出三张由服务记 never-fits）；
//! 用例照交来的次序取头三张（`common_discrete_gpus_of_this_host`）。服务没交这一台的卡：打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」；
//! 交来的少于三张、或对不上本机的独显：判红，不降级成少几张跑。
//! 只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `task-server/task submit` 交（卡与清场由服务定），带 `--test-threads 1`。
#![cfg(feature = "tests-on-a-real-gpu")]

mod common_checksum_batches_over_the_handed_over_gpus;
mod common_discrete_gpus_of_this_host;

use common_checksum_batches_over_the_handed_over_gpus::every_handed_over_gpu_of_a_batch_is_given_work_and_submits_on_the_real_gpus;

/// 这一份源头第一行 `// task-server-gpu-cards:` 声明的张数，用例照它取交来的卡；两处要相同（单测
/// `the_three_gpu_target_declares_in_its_header_the_gpu_count_its_tests_ask_for` 核）。
const GPU_CARDS_DECLARED_IN_THE_HEADER: usize = 3;
/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: std::time::Duration = std::time::Duration::from_secs(60);

#[test]
fn the_batch_over_three_named_real_gpus_gives_each_work_and_each_submits() {
    every_handed_over_gpu_of_a_batch_is_given_work_and_submits_on_the_real_gpus(
        "the_batch_over_three_named_real_gpus_gives_each_work_and_each_submits",
        GPU_CARDS_DECLARED_IN_THE_HEADER,
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    );
}
