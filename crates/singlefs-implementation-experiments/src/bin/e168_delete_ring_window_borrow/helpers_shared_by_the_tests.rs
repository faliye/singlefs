//! 几份单测共用的帮手。

use super::*;

thread_local! {
    /// 假的谷值钩子存放处：代替副本里 `allocator::FREE_SLOTS_AT_ROOT_RECORDING_FOR_TESTS`，测试往里推「记根时读到的空闲槽」。
    pub(super) static FAKE_FREE_SLOTS_RECORDED_AT_EACH_ROOT: std::cell::RefCell<Vec<Vec<u64>>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub(super) fn take_the_fake_free_slots_recorded_at_each_root() -> Vec<Vec<u64>> {
    FAKE_FREE_SLOTS_RECORDED_AT_EACH_ROOT.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// 在这条线程上攒行跑 `run`（与切片执行器给一片开的攒行同一个缓冲）：交回攒下的行与停机行（跑完是 None）。
pub(super) fn lines_and_stop_line_of(run: impl FnOnce()) -> (Vec<String>, Option<String>) {
    silence_the_panic_hook_for_stops_inside_slices();
    LINES_OF_THE_SLICE_ON_THIS_THREAD.with(|cell| *cell.borrow_mut() = Some(Vec::new()));
    let result = catch_unwind(AssertUnwindSafe(run));
    let lines = LINES_OF_THE_SLICE_ON_THIS_THREAD
        .with(|cell| cell.borrow_mut().take())
        .expect("开跑时放进去的缓冲");
    let stop_line = match result {
        Ok(()) => None,
        Err(payload) => match payload.downcast::<StopInsideASlice>() {
            Ok(stop) => Some(stop.stop_line),
            Err(payload) => resume_unwind(payload),
        },
    };
    (lines, stop_line)
}

/// 单测不经补丁的设账入口：什么都不做（账恒为第一版的两个 0，即 N 臂的账）。
pub(super) fn leave_the_ledger_as_it_is(_ledger: Option<(u64, u64)>) {}
