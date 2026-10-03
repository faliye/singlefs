#!/usr/bin/env bash
# 用法：gen_old.sh <fcc4d340 的 recovery.rs> <输出目录>；改前 recovery.rs 取法：git show fcc4d340:crates/singlefs-core/src/recovery.rs
# 取改前的 record_ends_its_publish（2932–2937）、readable_record_of_the_same_publish_follows（2941–2953）、
# counter_of_the_last_record_the_root_covers（2962–2987）、replay_journal（3057–3229），改名成 *_before 放进 zz_old*.rs；
# 另给四个改前变异体 m1–m4（同第一轮）。
set -euo pipefail
O=$1; S=$2
mk() {
  name=$1; shift
  { echo '#![allow(clippy::all, dead_code, unused_imports)]'; echo 'use super::*;'; echo 'use crate::journal::{JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish};'
    sed -n 2932,2937p "$O" | sed 's/^fn record_ends_its_publish(/pub(super) fn record_ends_its_publish_before(/'
    sed -n 2941,2953p "$O" | sed 's/^fn readable_record_of_the_same_publish_follows(/pub(super) fn readable_record_of_the_same_publish_follows_before(/' | sed 's/^pub(super) fn/pub(super) fn/'
    sed -n 2962,2987p "$O" | sed 's/^fn counter_of_the_last_record_the_root_covers(/pub(super) fn counter_of_the_last_record_the_root_covers_before(/' | sed 's/record_ends_its_publish(record)/record_ends_its_publish_before(record)/'
    sed -n 3057,3229p "$O" | sed 's/^pub fn replay_journal(/pub fn replay_journal_before(/' \
      | sed 's/counter_of_the_last_record_the_root_covers(root, records)/counter_of_the_last_record_the_root_covers_before(root, records)/; s/record_ends_its_publish(record)/record_ends_its_publish_before(record)/g; s/readable_record_of_the_same_publish_follows(record, records)/readable_record_of_the_same_publish_follows_before(record, records)/' \
      | sed "$@"; } > "$S/$name.rs"
}
mk zz_old -e 's/XXXXNOP//'
mk zz_old_m1 -e 's/\.take(in_flight_limit)//'
mk zz_old_m2 -e 's/if !all_verified {/if !all_verified \&\& false {/'
mk zz_old_m3 -e 's/record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST$/record.ordinal_within_publish.0 > 2/'
mk zz_old_m4 -e 's/report.maximum_applied_transaction = records_of_the_open_publish/report.maximum_applied_transaction = records_of_the_open_publish.iter().take(1)/;s/^            .iter()$//'
