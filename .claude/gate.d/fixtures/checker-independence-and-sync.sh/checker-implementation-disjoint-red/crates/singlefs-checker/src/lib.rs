use singlefs_core::rebuild_accounting;
use implementation::rebuild_accounting as rebuilt_through_alias;
pub fn check() -> u64 { rebuild_accounting() }
