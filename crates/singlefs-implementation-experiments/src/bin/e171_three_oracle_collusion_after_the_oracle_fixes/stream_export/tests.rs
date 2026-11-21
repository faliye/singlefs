//! 一条流的导出与它的状态数回比 crates的单测。

use super::*;
// 经产品取样路径（`sample_crash_states_on_the_cpu`、给定状态引擎）开全仓那一份的用例开头拿它：同一个二进制里
// `pool_checker_holds_and_violations_through_the_given_states_engine` 设一次性库，不拿的在它设着时开到它那一份、它收尾停服之后连不上
use singlefs_verification::crash_library::TheOneLibraryKeptForThisProcess;

/// 流二第 3 次调用 panic（阳性对照 panic 的造法）：整条流照常导出、取样态照判，不退回「整条流不导出」
/// （登记修订 2 第 2.4 节；V4：panic 那一支退回不导出，这里红）。
#[test]
fn stream_stopped_by_a_panic_is_exported_and_its_sampled_states_are_judged() {
    let _the_one_library = TheOneLibraryKeptForThisProcess::keep_disposable_libraries_out();
    let switches = RunSwitches {
        positive_control: Some(PositiveControl::PanicInTheCall),
        disabled_oracle: None,
        writable_mount_probe: false,
    };
    let stream = prepare_stream(
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
        switches,
        false,
    );
    // 守卫认领导出目录：用例做完、断言失败、panic 都删掉它
    let directory = singlefs_verification::temporary_files::TemporaryPathRemovedOnDrop::claiming(
        std::env::temp_dir().join(format!("e171r1-unit-panic-export-{}", std::process::id())),
    );
    let export = export_prepared_stream(
        &stream,
        &directory,
        false,
        switches,
        StateSelection::RegisteredBatches { third_batch: false },
        Instant::now(),
        0,
        None,
    );
    assert!(
        matches!(
            stream.ending,
            StreamEnding::StoppedByAPanic { call_index: 3, .. }
        ),
        "{:?}",
        stream.ending
    );
    assert_eq!(export.not_exported, None, "panic 截断的流照常导出");
    assert!(!export.observations.is_empty(), "取样态照判");
    assert_eq!(
        stream.calls[..PANIC_CONTROL_CALL]
            .iter()
            .filter(|call| call.model.was_judged_by_the_comparison())
            .count(),
        1,
        "panic 之前对拍判过的调用恰 1 次（第 2 次调用）"
    );
}
