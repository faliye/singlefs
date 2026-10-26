//! 正推腿的探针（草稿副本，不入库）：把 core 的 truncate 纯函数在一圈边界输入上的答案打印出来，交给 python 另写的模型逐行比。
use singlefs_core::address::CheckpointTxg;
use singlefs_core::records::InodeRecordTruncationField;
use singlefs_core::truncate::{
    data_units_a_size_needs, every_truncation_step_from, tail_data_unit_rewrite_of_a_truncation,
    tail_data_unit_rewrite_of_an_extension, target_size_rounded_up_to_a_whole_data_unit_fits,
    truncation_field_after_the_truncation_intent, TruncationStep,
};
use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};

const P: u64 = 32634;

fn sizes() -> Vec<u64> {
    let mut v = vec![0u64, 1];
    for m in 1..=6u64 {
        v.push(m * P - 1);
        v.push(m * P);
        v.push(m * P + 1);
    }
    v.sort_unstable();
    v.dedup();
    v
}

#[test]
fn probe() {
    let sizes = sizes();
    for existing in 1..=6u64 {
        for &s in &sizes {
            for &n in &sizes {
                if n == s {
                    continue;
                }
                if n < s {
                    let rewrite = tail_data_unit_rewrite_of_a_truncation(existing, s, n);
                    let field = truncation_field_after_the_truncation_intent(
                        InodeRecordTruncationField::written_by_the_truncation_intent_of(CheckpointTxg(5), 123),
                        existing,
                        n,
                        CheckpointTxg(9),
                    );
                    let carried = field == InodeRecordTruncationField::written_by_the_truncation_intent_of(CheckpointTxg(5), 123);
                    println!(
                        "TRUNC existing={existing} s={s} n={n} rewrite={:?} needed={} field_carried={carried} field_txg={} field_n={}",
                        rewrite.map(|r| (r.unit.0, r.declared_length_in_bytes)),
                        data_units_a_size_needs(n),
                        field.truncation_txg().0,
                        field.truncated_size_in_bytes()
                    );
                    for batch in [1u64, 2, 144] {
                        let size = UnlinkBatchSize::CappedForTests(
                            UnlinkBatchDataUnitsCappedForTests::new(batch).expect("1..=144"),
                        );
                        let steps: Vec<String> = every_truncation_step_from(existing, n, size)
                            .into_iter()
                            .map(|s| match s {
                                TruncationStep::Batch { data_units_kept } => format!("B{data_units_kept}"),
                                TruncationStep::FinalBatch { data_units_kept } => format!("F{data_units_kept}"),
                            })
                            .collect();
                        println!("STEPS existing={existing} n={n} batch={batch} steps={}", steps.join(","));
                    }
                } else {
                    let rewrite = tail_data_unit_rewrite_of_an_extension(existing, s, n);
                    println!(
                        "EXT existing={existing} s={s} n={n} rewrite={:?}",
                        rewrite.map(|r| (r.unit.0, r.declared_length_in_bytes))
                    );
                }
            }
        }
    }
    for n in [u64::MAX, u64::MAX - P + 1, u64::MAX - P, (u64::MAX / P) * P, (u64::MAX / P) * P + 1, 0, 1] {
        println!("FITS n={n} fits={}", target_size_rounded_up_to_a_whole_data_unit_fits(n));
    }
}
