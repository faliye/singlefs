use singlefs_format::SAMPLE_BYTES;
use singlefs_checker::check as checker_verdict;
pub fn reference_publish() -> u64 { SAMPLE_BYTES + checker_verdict() }
