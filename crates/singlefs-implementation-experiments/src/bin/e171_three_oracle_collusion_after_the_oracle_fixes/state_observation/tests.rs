//! 取样态的观测：一个态上判的各块、panic 的态与按序观测取样态的单测。

use super::*;

/// 模型不认的版本读回来记红、认的记绿（X4：崩溃判官的结果丢掉，这里红）。
#[test]
fn read_back_of_a_version_the_model_never_committed_is_red_and_a_committed_one_is_green() {
    let committed_key = model_root_key(InstanceGeneration(1), CheckpointTxg(3));
    let committed = BTreeMap::from([(committed_key, Some(Rc::<[u8]>::from(vec![7u8, 8, 9])))]);
    let green = ObservedReadBack::FileRead {
        root: committed_key,
        content: vec![7, 8, 9],
    };
    assert_eq!(
        crash_judge_verdict(&committed, Some(committed_key), &green),
        CrashJudgeVerdict::Green
    );
    let flipped_last_byte = ObservedReadBack::FileRead {
        root: committed_key,
        content: vec![7, 8, 8],
    };
    assert!(matches!(
        crash_judge_verdict(&committed, Some(committed_key), &flipped_last_byte),
        CrashJudgeVerdict::Red { .. }
    ));
    let never_committed = ObservedReadBack::NoFile {
        root: model_root_key(InstanceGeneration(9), CheckpointTxg(99)),
    };
    assert!(matches!(
        crash_judge_verdict(&committed, None, &never_committed),
        CrashJudgeVerdict::Red { .. }
    ));
}

/// 模型停在第 3 次调用：第 2 次及之前的调用里写的态适用，第 3 次及之后的不适用；一个写都没落的态与模型没停的流都适用
/// （X14：流二每一态都记不适用，这里红）。
#[test]
fn the_crash_judge_is_not_applicable_only_from_the_call_where_the_model_stopped() {
    assert!(crash_judge_is_applicable(Some(2), Some(3)));
    assert!(!crash_judge_is_applicable(Some(3), Some(3)));
    assert!(!crash_judge_is_applicable(Some(5), Some(3)));
    assert!(crash_judge_is_applicable(None, Some(0)));
    assert!(crash_judge_is_applicable(Some(5), None));
}

/// 态上的恢复 panic 不截断（读法 5；V5：退回「交回 Err、后面的态不判」，这里红）：一段 [3, 9) 在序号 5 上 panic，
/// 5 记成恢复 panic 的态、6、7、8 照判；另一段 [20, 22) 照判。
#[test]
fn state_whose_recovery_panics_is_noted_and_the_states_after_it_are_still_judged() {
    let mut judged: Vec<u64> = Vec::new();
    let mut judge_run = |run: std::ops::Range<u64>| -> Result<(), u64> {
        for ordinal in run {
            if ordinal == 5 {
                return Err(ordinal);
            }
            judged.push(ordinal);
        }
        Ok(())
    };
    let mut panicking: Vec<u64> = Vec::new();
    judge_runs_continuing_past_panicking_states(&[3..9, 20..22], &mut judge_run, &mut |ordinal| {
        panicking.push(ordinal);
    });
    assert_eq!(judged, vec![3, 4, 6, 7, 8, 20, 21]);
    assert_eq!(panicking, vec![5]);
}
