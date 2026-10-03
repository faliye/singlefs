#!/usr/bin/env bash
# 用法：gen_old.sh <改前 recovery.rs> <输出目录>；改前 recovery.rs 取法：git show fcc4d340:crates/singlefs-core/src/recovery.rs
set -euo pipefail
O=$1; S=$2
mk() {
  name=$1; shift
  { echo '#![allow(clippy::all, dead_code, unused_imports)]'; echo 'use super::*;'; echo 'use crate::journal::JournalRecordOrdinalWithinPublish;'
    sed -n 2941,2953p "$O" | sed 's/^fn readable_record_of_the_same_publish_follows/pub(super) fn readable_record_of_the_same_publish_follows/'
    sed -n 3057,3229p "$O" | sed 's/^pub fn replay_journal(/pub fn replay_journal_before(/' | sed "$@"; } > "$S/$name.rs"
}
mk zz_old -e 's/XXXXNOP//'
mk zz_old_m1 -e 's/\.take(in_flight_limit)//'
mk zz_old_m2 -e 's/if !all_verified {/if !all_verified \&\& false {/'
mk zz_old_m3 -e 's/record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST$/record.ordinal_within_publish.0 > 2/'
mk zz_old_m4 -e 's/report.maximum_applied_transaction = records_of_the_open_publish/report.maximum_applied_transaction = records_of_the_open_publish.iter().take(1)/;s/^            .iter()$//'
