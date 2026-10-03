// 把 `crates/singlefs-checker-tier/src/crash_injection.rs` 里 `StreamMarks::read_faults_in_a_crash_state` 的函数体原样搬出来
// （只给它配最小的枚举桩），在它的两条内联用例的输入上跑基线与 mutations.tsv 第 2225、2226 行的改法，看点名的用例红没红。
// 不是 checker 档的测试二进制：独立的 rustc 小程序，只含这一个纯函数。
// 编：rustc --edition 2021 -O read_faults_in_a_crash_state_standalone.rs -o <出口>；
// 变体：--cfg m2225、--cfg m2226。
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReadFaultsInTheCrashState {
    NoneInjected,
    Injected,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HistoryOperationKind {
    CrashRecoveryAbandoningTheNewestRoot,
    PublishWithoutUnits,
    CloseAndMountWritable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepPosition {
    StartingPoint,
    Operation(usize),
}
struct StreamMarks {
    after_make_filesystem: usize,
    after_the_last_finished_step: usize,
    after_each_step: Vec<(StepPosition, usize)>,
}
fn operation_reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
    match kind {
        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,
        HistoryOperationKind::PublishWithoutUnits | HistoryOperationKind::CloseAndMountWritable => {
            false
        }
    }
}

impl StreamMarks {
    fn read_faults_in_a_crash_state(
        &self,
        step_kinds: &[HistoryOperationKind],
        last_stream_index_of_the_crash_window: usize,
    ) -> ReadFaultsInTheCrashState {
        let mut operations_before_the_step = 0;
        for (position, operations_after_the_step) in &self.after_each_step {
            let step_reads_under_injected_faults = match position {
                StepPosition::StartingPoint => false,
                StepPosition::Operation(step_index) => step_kinds
                    .get(*step_index)
                    .copied()
                    .is_some_and(operation_reads_under_injected_faults),
            };
            #[cfg(not(m2226))]
            let step_sent_something_by_the_crash = operations_before_the_step
                < *operations_after_the_step
                && operations_before_the_step <= last_stream_index_of_the_crash_window;
            #[cfg(m2226)]
            let step_sent_something_by_the_crash =
                operations_before_the_step <= last_stream_index_of_the_crash_window;
            #[cfg(not(m2225))]
            let injected = step_reads_under_injected_faults && step_sent_something_by_the_crash;
            #[cfg(m2225)]
            let injected = false && step_reads_under_injected_faults && step_sent_something_by_the_crash;
            if injected {
                return ReadFaultsInTheCrashState::Injected;
            }
            operations_before_the_step = *operations_after_the_step;
        }
        ReadFaultsInTheCrashState::NoneInjected
    }
}

fn main() {
    use HistoryOperationKind::*;
    use ReadFaultsInTheCrashState::*;
    let step_kinds = [CloseAndMountWritable, CrashRecoveryAbandoningTheNewestRoot, PublishWithoutUnits];
    // 用例一：a_crash_state_counts_as_read_faulted_from_the_first_stream_step_of_an_abandoning_recovery_on
    let marks_one = StreamMarks {
        after_make_filesystem: 4,
        after_the_last_finished_step: 40,
        after_each_step: vec![
            (StepPosition::StartingPoint, 10),
            (StepPosition::Operation(0), 20),
            (StepPosition::Operation(1), 25),
            (StepPosition::Operation(2), 40),
        ],
    };
    let case_one = [(19, NoneInjected), (20, Injected), (35, Injected)]
        .iter()
        .all(|(window, expected)| marks_one.read_faults_in_a_crash_state(&step_kinds, *window) == *expected);
    // 用例二：an_abandoning_recovery_that_sent_nothing_leaves_every_crash_state_without_read_faults
    let marks_two = StreamMarks {
        after_make_filesystem: 4,
        after_the_last_finished_step: 40,
        after_each_step: vec![
            (StepPosition::StartingPoint, 10),
            (StepPosition::Operation(0), 20),
            (StepPosition::Operation(1), 20),
            (StepPosition::Operation(2), 40),
        ],
    };
    let case_two = [12, 19, 20, 35]
        .iter()
        .all(|window| marks_two.read_faults_in_a_crash_state(&step_kinds, *window) == NoneInjected);
    println!("用例一 {}；用例二 {}", if case_one { "绿" } else { "红" }, if case_two { "绿" } else { "红" });
}
