//! 几份测试共用的辅助：读某条断言的违例数与判定、小改写的常量。

use super::*;

pub(super) fn violations(simulation: &Simulation, name: &str) -> u64 {
    simulation.assertions.violations_of(name)
}

pub(super) fn judged(simulation: &Simulation, name: &str) -> u64 {
    simulation.assertions.judged.get(name).copied().unwrap_or(0)
}

pub(super) const SMALL_REWRITE: HistorySpecification = HistorySpecification::LargeFileRewriteAndDelete {
    data_units: 3,
    rewrites: 5,
    overwrite_before_unlink: true,
    batch: 1,
};
