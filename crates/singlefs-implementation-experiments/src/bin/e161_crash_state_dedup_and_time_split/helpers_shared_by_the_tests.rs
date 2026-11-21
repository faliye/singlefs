//! 几份单测共用的帮手。

use super::*;

/// 新池新建文件流的 D1-small（53 个状态）跑一遍 5.1 的四件事；各测试共用这一份搭建的写法。
/// 评之前先核 D1-small 的个数（重跑登记 I-7 的域守卫）：段形或阈值一变，D1-small 会把 1677 万个状态的段整段收进来，
/// 不先核就一路评到撞内存上限。
pub(super) fn evaluate_the_first_stream_small_domain(
    threads: usize,
    timing: TimingControl,
) -> (PreparedStream, StreamDomainTotals, Vec<u64>) {
    let stream = prepare_first_stream();
    let (totals, small) = {
        let table = TornWriteTable::of(&stream);
        let context = EvaluationContext::new(&stream, &table);
        let plan = LocalPlan::new(
            &stream,
            &table,
            domain_expansions(&stream, LocalExpansion::EveryCombination),
        );
        let small = small_domain_ordinals(&stream, &plan);
        assert_eq!(
            small.len(),
            53,
            "域守卫：新池新建文件流 D1-small 是 53 个状态（第 3 次重跑登记 A4），对不上就不评"
        );
        let control: BTreeSet<u64> = small.iter().copied().collect();
        let totals = evaluate_stream_domain(&StreamDomainRun {
            label: "test-first-small".to_string(),
            context: &context,
            plan: &plan,
            ordinals: &small,
            control_ordinals: &control,
            timing,
            threads,
            depth: EvaluationDepth::FourSteps,
        });
        (totals, small)
    };
    (stream, totals, small)
}
