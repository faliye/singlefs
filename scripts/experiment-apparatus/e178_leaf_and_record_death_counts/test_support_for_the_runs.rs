//! 几份测试共用的辅助：取写路径的格、在读者表里找位置、按名字取形状、造带桩路径与原路径的模拟器、测项库选项的环境与解析。

use super::*;

pub(super) fn write_path_cell(borrow_arm: BorrowArm, leaf_capacity: u64, merge_threshold: u64) -> WritePathCell {
    WritePathCell { borrow_arm, leaf_capacity, merge_threshold }
}

pub(super) fn position_of(base: BaseArm, shrink: ShrinkArm, effect: UnlinkEffectArm, window: ReadableWindow) -> usize {
    reader_position(Reader { base, shrink, effect, window })
}

pub(super) fn shape_named(name: &str) -> TreeShape {
    catalogue_of_shapes().into_iter().find(|named| named.name == name).expect("形态表里有它").shape
}

// ---------------------------------------------------------------------------------------------
// 第四段（登记修订 2）
// ---------------------------------------------------------------------------------------------

pub(super) fn stub_path_simulation(history: History, write_path: WritePathCell, scanning: bool) -> (Simulation, Option<RollbackOutcome>) {
    let mut simulation = Simulation::start_with(write_path, history.batch(), scanning, ContainerDeathWritePath::ZeroRecordStubs, segment_four_stub_path_readers());
    let outcome = run_history(history, &mut simulation);
    (simulation, outcome)
}

pub(super) fn original_path_simulation(history: History, write_path: WritePathCell, scanning: bool) -> Simulation {
    let mut simulation =
        Simulation::start_with(write_path, history.batch(), scanning, ContainerDeathWritePath::RetirementRecords, segment_four_original_path_readers());
    run_history(history, &mut simulation);
    simulation
}

pub(super) fn position_in(readers: &[Reader], wanted: Reader) -> usize {
    readers.iter().position(|reader| *reader == wanted).expect("读者表里有它")
}

/// 没经 device-code-key.py run 起的样子：环境里没有转交判定库的管道。
pub(super) fn test_environment_without_relay() -> SegmentFourEnvironment {
    SegmentFourEnvironment { store: Err("环境里没有 DEVICE_LIBRARY_REQUEST_FD：读写判定库要经 scripts/experiment-apparatus/device-code-key.py run 起".to_string()) }
}

pub(super) fn parse_with(arguments: &[&str], environment: &SegmentFourEnvironment) -> Result<SegmentFourOptions, String> {
    segment_four_options_from(&arguments.iter().map(|argument| (*argument).to_string()).collect::<Vec<String>>(), environment)
}
