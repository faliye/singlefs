// task-server-gpu-cards: 2
//! 两张卡：校验和核对那两份核在服务交来的同一台两张真卡上按卡的顺序填满（测饱和），每张卡都分到活、
//! 结果与 CPU 参照逐条相同；另有一条活少时只有点名在前的那张分到活。验收「一张卡 1 GiB、两张卡、三张卡，每张卡都能用来分片 gc」
//! 的两张那一格；一张的在 `checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference`，
//! 三张的在 `checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference`。
//! 源头第一行声明要同一台两张卡（`task-server/README.md`「源文件头的声明」），服务照它分、经 `GPU_CARDS_BY_PRIORITY` 交；用例照交来的次序取头两张
//! （`common_discrete_gpus_of_this_host`）。服务没交这一台的卡：每条打 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」；
//! 交来的少于两张、或对不上本机的独显：判红，不降级成一张跑。
//! 只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU。统一任务服务只调配 CPU，任务不拿卡（`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」）：
//! 卡经环境变量 `GPU_CARDS_BY_PRIORITY` 交来，没交来、也没有领卡通道的报「本次未跑」。
//! 对拍分两段、经判定库交接，对拍结论按键入库复用（`common_checksum_batches_over_the_handed_over_gpus` 文件头）：起跑就有卡、不按需领，
//! 库里有两边一致的结论照样复用、不开卡。每条用例的持卡判入口是 `judged_on_the_cards_that_<用例名>`（登记在 `tests/pairing_units_entries.tsv` 的 GPU 一侧那几行）。
#![cfg(feature = "tests-on-a-real-gpu")]

mod common_checksum_batches_over_the_handed_over_gpus;
mod common_discrete_gpus_of_this_host;
mod common_independent_references;
mod common_pairing_input_through_the_library;

use common_checksum_batches_over_the_handed_over_gpus::{
    background_gpu_on_the_declared_count_of_handed_over_gpus_or_report_not_run,
    checksum_units_and_reference_outcomes_read_back,
    checksum_units_concluded_on_the_cards_through_the_pairing_library,
    judged_on_the_declared_count_of_gpus_that_every_gpu_of_a_batch_is_given_work_and_submits,
    units_of_four_kibibytes, units_of_the_fill_over_the_declared_count_of_gpus, units_to_verify,
};
use common_discrete_gpus_of_this_host::MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU;
use common_pairing_input_through_the_library::PairingJudged;
use common_pairing_input_through_the_library::{
    judgement_of_the_assertions_on_the_cards, ComparisonOnTheCards,
};

/// 这一份源头第一行 `// task-server-gpu-cards:` 声明的张数，用例照它取交来的卡；两处要相同（单测
/// `the_two_gpu_target_declares_in_its_header_the_gpu_count_its_tests_ask_for` 核）。
const GPU_CARDS_DECLARED_IN_THE_HEADER: usize = 2;
/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: std::time::Duration = std::time::Duration::from_secs(60);

#[test]
fn the_batch_over_two_named_real_gpus_gives_each_work_and_each_submits() {
    checksum_units_concluded_on_the_cards_through_the_pairing_library(
        "the_batch_over_two_named_real_gpus_gives_each_work_and_each_submits",
        || units_of_the_fill_over_the_declared_count_of_gpus(GPU_CARDS_DECLARED_IN_THE_HEADER),
        judged_on_the_cards_that_the_batch_over_two_named_real_gpus_gives_each_work_and_each_submits,
    );
}

/// 持卡判入口（登记在 `tests/pairing_units_entries.tsv` 的 GPU 一侧那几行）：按卡的顺序填满源头声明的两张卡。
fn judged_on_the_cards_that_the_batch_over_two_named_real_gpus_gives_each_work_and_each_submits(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judged_on_the_declared_count_of_gpus_that_every_gpu_of_a_batch_is_given_work_and_submits(
        test_name,
        GPU_CARDS_DECLARED_IN_THE_HEADER,
        WAIT_LIMIT_OF_THE_GPU_TESTS,
        input_bytes,
        reference_bytes,
    )
}

/// 活少那一头：点名交来的两张、各 1 GiB，一批 1024 个 4096 字节的单元（4 MiB 多，点名在前的那张一次交得下）——
/// 只有点名在前的那张分到活、交一次（不要求每张卡都分到活）。
#[test]
fn the_batch_the_first_named_real_gpu_holds_goes_only_to_that_gpu() {
    checksum_units_concluded_on_the_cards_through_the_pairing_library(
        "the_batch_the_first_named_real_gpu_holds_goes_only_to_that_gpu",
        || units_of_four_kibibytes(1024),
        judged_on_the_cards_that_the_batch_the_first_named_real_gpu_holds_goes_only_to_that_gpu,
    );
}

/// 持卡判入口（登记在 `tests/pairing_units_entries.tsv` 的 GPU 一侧那几行）：开源头声明的两张卡、各 1 GiB，读回的那一批只交给点名在前的那张、交一次，两份核逐条与读回的 CPU 参照比。
fn judged_on_the_cards_that_the_batch_the_first_named_real_gpu_holds_goes_only_to_that_gpu(
    test_name: &str,
    input_bytes: &[u8],
    reference_bytes: &[u8],
) -> PairingJudged {
    judgement_of_the_assertions_on_the_cards(test_name, || {
        let (unit_bytes_and_expected, reference_outcomes) =
            checksum_units_and_reference_outcomes_read_back(input_bytes, reference_bytes);
        let Some((mut background_gpu, named_cards)) =
            background_gpu_on_the_declared_count_of_handed_over_gpus_or_report_not_run(
                test_name,
                GPU_CARDS_DECLARED_IN_THE_HEADER,
                MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU,
                WAIT_LIMIT_OF_THE_GPU_TESTS,
            )
        else {
            return ComparisonOnTheCards::NotRunOnTheseCards;
        };
        let outcomes = background_gpu
            .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
            .expect("健康的卡上一批交得成");
        let submissions = background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification;
        assert_eq!(
            outcomes,
            reference_outcomes
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
        ComparisonOnTheCards::EveryComparisonHeld
    })
}
