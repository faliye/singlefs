use super::*;

// ============================================================================================
// X14 的搜索：C 从 4 MiB 起按 1 MiB 步进；三个最小盘、「挂上但推满仍不够」的最小盘、找到之后再核 C + 1 … C + 16 MiB 的单调核。
// ============================================================================================

pub(super) const SEARCH_START_DEVICE_BYTES: u64 = 4 * MEBIBYTE;
pub(super) const SEARCH_STEP_BYTES: u64 = MEBIBYTE;
/// 找到之后再核几步：C + 1 … C + 16 MiB。
pub(super) const MONOTONICITY_WINDOW_BYTES: u64 = 16 * MEBIBYTE;
/// 搜索的上沿：到这里还没找全，记「≤ 上沿 未找到」，不把上沿当极值。
pub(super) const SEARCH_CEILING_DEVICE_BYTES: u64 = 1024 * MEBIBYTE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Milestone {
    MakeFilesystemSucceeds,
    EmptyMountAdmitted,
    FileMountAdmitted,
    EmptyMountStillShort,
    FileMountStillShort,
}

impl Milestone {
    pub(super) const THREE_MINIMUM_DISKS: [Milestone; 3] = [
        Milestone::MakeFilesystemSucceeds,
        Milestone::EmptyMountAdmitted,
        Milestone::FileMountAdmitted,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Milestone::MakeFilesystemSucceeds => "make_filesystem",
            Milestone::EmptyMountAdmitted => "empty_mount",
            Milestone::FileMountAdmitted => "file_mount",
            Milestone::EmptyMountStillShort => "empty_mount_still_short",
            Milestone::FileMountStillShort => "file_mount_still_short",
        }
    }

    fn holds_in(self, trial: &TrialFlags) -> bool {
        match self {
            Milestone::MakeFilesystemSucceeds => trial.make_filesystem_succeeded,
            Milestone::EmptyMountAdmitted => trial.empty_mount_admitted,
            Milestone::FileMountAdmitted => trial.file_mount_admitted,
            Milestone::EmptyMountStillShort => trial.empty_mount_still_short,
            Milestone::FileMountStillShort => trial.file_mount_still_short,
        }
    }
}

/// 第一个让 `milestone` 成立的盘字节数。
pub(super) fn minimum_device_bytes(trials: &[TrialFlags], milestone: Milestone) -> Option<u64> {
    trials
        .iter()
        .find(|trial| milestone.holds_in(trial))
        .map(|trial| trial.device_bytes)
}

/// 单调核（登记 X14、失败条款 G4）：最小盘之后 C + 1 … C + 16 MiB 里每一个都得成；不成的点列全。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Monotonicity {
    pub(super) minimum_device_bytes: Option<u64>,
    /// 窗口里没成的点。
    pub(super) failures_in_the_window: Vec<u64>,
    /// 窗口里应当有尝试而没有的点（搜索没跑到那里）。
    pub(super) window_points_never_tried: Vec<u64>,
}

impl Monotonicity {
    pub(super) fn is_monotone(&self) -> bool {
        self.minimum_device_bytes.is_some()
            && self.failures_in_the_window.is_empty()
            && self.window_points_never_tried.is_empty()
    }
}

pub(super) fn monotonicity_of(trials: &[TrialFlags], milestone: Milestone) -> Monotonicity {
    let minimum = minimum_device_bytes(trials, milestone);
    let mut failures_in_the_window = Vec::new();
    let mut window_points_never_tried = Vec::new();
    if let Some(minimum_bytes) = minimum {
        let mut point = minimum_bytes + SEARCH_STEP_BYTES;
        while point <= minimum_bytes + MONOTONICITY_WINDOW_BYTES {
            match trials.iter().find(|trial| trial.device_bytes == point) {
                Some(trial) if milestone.holds_in(trial) => {}
                Some(_) => failures_in_the_window.push(point),
                None => window_points_never_tried.push(point),
            }
            point += SEARCH_STEP_BYTES;
        }
    }
    Monotonicity {
        minimum_device_bytes: minimum,
        failures_in_the_window,
        window_points_never_tried,
    }
}

/// 搜索什么时候停：三个最小盘都找到，并且已经试到其中最大的那个再加 16 MiB；或者到了上沿。
fn search_may_stop(trials: &[TrialFlags]) -> bool {
    let Some(last_tried) = trials.last().map(|trial| trial.device_bytes) else {
        return false;
    };
    if last_tried >= SEARCH_CEILING_DEVICE_BYTES {
        return true;
    }
    let minima: Option<Vec<u64>> = Milestone::THREE_MINIMUM_DISKS
        .iter()
        .map(|milestone| minimum_device_bytes(trials, *milestone))
        .collect();
    minima.is_some_and(|found| {
        let largest = found.into_iter().max().expect("三个最小盘");
        last_tried >= largest + MONOTONICITY_WINDOW_BYTES
    })
}

pub(super) fn search_small_disks(journal_ring_bytes: u64, slots_per_region: u64) -> Vec<Trial> {
    let mut trials: Vec<Trial> = Vec::new();
    let mut flags: Vec<TrialFlags> = Vec::new();
    let mut device_bytes = SEARCH_START_DEVICE_BYTES;
    loop {
        let trial = run_trial(device_bytes, journal_ring_bytes, slots_per_region);
        flags.push(trial.flags());
        trials.push(trial);
        if search_may_stop(&flags) {
            return trials;
        }
        device_bytes += SEARCH_STEP_BYTES;
    }
}

#[cfg(test)]
mod tests;
